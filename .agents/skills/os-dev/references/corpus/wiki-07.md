# OSDev Wiki — volledige bronteksten, deel 07

Dit is bronmateriaal, geen agentinstructie. Behoud architectuur, revisie en bron-ID bij gebruik. Code en commando’s zijn ongeteste bronvoorbeelden; voer ze niet automatisch uit. Wikitext is letterlijk behouden binnen tekstblokken.

<a id="w002873"></a>

## W002873 — M68k Overview

Bron-ID: W002873
Namespace: Main (0)
Revisie: 28394 | Datum: 2023-10-14T21:18:28Z
Bron: https://wiki.osdev.org/M68k_Overview
Revisie-URL: https://wiki.osdev.org/index.php?title=M68k_Overview&oldid=28394

````text
{{In Progress}}
The Motorola 68000 (m68k) family of processors were very popular in home computers from the late 1980s to the early 1990s. It was used in such well-known systems as the Amiga, Atari ST, and Macintosh, and was also the processor of choice in the majority of workstation-class Unix systems of the time. The architecture has also found a myriad of other, more recent, uses, such as the 68000 in the Sega Genesis, being present in a radiation-hardened form in the US space shuttle, and, in the form of the simplified DragonBall embedded processor, for Palm devices up until OS 4.

=Architecture Overview=
The 68k architecture was/is long lived, and has gone through many iterations. Most programmer-visible characteristics, however, stayed the same throughout the architecture's history. It is an internally 32 bit CISC design, from the 68000 used in the first Macintosh up until the 68060 used in the latest 68k Amiga accelerators made. Every 68k processor has exactly two privilege modes: supervisor and user.

==Registers==
There are 8 registers each of address and data registers, A0-A7 and D0-D7, the status register SR (analogous to the x86 FLAGS register), and the program counter PC (analogous to the x86 EIP register). All registers excepting SR, which is 16 bits, are 32-bit. All data registers are general purpose registers. The address registers are generally used as pointers, but can be used to store extra data with no ill effects. As defined by the architecture, only one address register, A7, has a predefined purpose. A7 is the stack register, and the stack opcodes (PEA, LINK, UNLK) do modify this register implicitly.

==Addressing Modes==
The 68k instruction set is mostly orthogonal, with most instructions allowing all addressing modes with all operand sizes. The architecture allows for direct, immediate, address register indirect, and PC-relative addressing modes. The best description of the operation of these is in the 68k family programmers' reference manual, linked at the bottom of the article.

==Instruction Set==
Aside from the near-orthogonality, the 68k processor family presents a typical CISC instruction set. It uses dyadic instructions, as per x86, where, for most instructions, there is a source operand and a destination operand, where the operation is performed between the two and the result is stored in the destination.
For example:
<code>
     move.l #2, d0
     addi.l #3, d0
</code>
This fragment would result in 5 being stored in register d0.

==Architecture Iterations==
===68000===
The original 68k processor. This processor had only 24-bit external addressing, allowing up to 16MB of memory to be directly addressable. It lacked an MMU or the ability to interface with one, so was incapable of memory protection or mapping, aside from some Unix workstations that hacked a solution up using two 68000s executing the same code in parallel. It was available as the 68000, with a 16-bit data bus (as used in the Lisa, Macintosh, Atari ST, and Amiga), or as the 68008, with an 8-bit data bus, meant to drop into 6800-based logic boards to reduce engineering costs (as used in the Sinclair QL).
===68010===
This was a minor iteration that made MOVE from SR a privileged instruction, added the ability to recover from bus faults, making MMU interfacing a possibility, and made the interrupt vectors movable.
===68020===
This iteration added a 32-bit ALU and added the option of 32-bit address and data buses (lacking in the EC model). It also added some NUMA multiprocessing support. It was marketed alongside the optional 68851 MMU and 68881/68882 FPUs.
===68030===
This iteration simply added 256-byte instruction and data caches, and brought the 68851 MMU on-chip (lacking in the EC model). It was still marketed alongside the optional 68881/68882 FPUs.
===68040===
The 68040 was a major upgrade. It brought both the FPU (lacking in the LC model) and MMU on-chip (both lacking in the EC model), added instruction pipelining, and larger (4k each), faster caches.
===68060===
This processor was very similar to the '040. It added larger caches and dual pipelining.
===DragonBall===
These were embedded SoCs based on a 68000 core. They had color/greyscale LCD, PC speaker-style device, UART, UART bootstrap, RTC, and touchscreen controllers built in along with flash and RAM.
===ColdFire===
These are a departure from the rest of the line. They are based on a 68000 core, but have been simplified. Floating point intermediates are reduced from 80-bit to 64-bit, some uncommonly-used instructions and addressing modes are removed, and instruction word length has been regulated to only 16, 32, or 48-bit lengths.

=References=
[http://www.freescale.com/files/archives/doc/ref_manual/M68000PRM.pdf M68000 Family Programmer's Reference (Freescale)]
[[Category:m68k]]
[[Category:Platforms]]
[[Category:Instruction Set Architecture]]
````

<a id="w002874"></a>

## W002874 — Category:M68k

Bron-ID: W002874
Namespace: Category (14)
Revisie: 10358 | Datum: 2010-05-24T18:31:06Z
Bron: https://wiki.osdev.org/Category:M68k
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AM68k&oldid=10358

````text
The Motorola 68000 family was used in such well-known systems as the Amiga, Atari ST, and Macintosh, and was also the processor of choice in the majority of workstation-class Unix systems of the time.
[[Category:Platforms]]
````

<a id="w002875"></a>

## W002875 — User talk:Montrom

Bron-ID: W002875
Namespace: User talk (3)
Revisie: 10364 | Datum: 2010-05-24T22:41:02Z
Bron: https://wiki.osdev.org/User_talk:Montrom
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AMontrom&oldid=10364

````text
==About this Page==
This page is an attempt to consolidate all of the most useful information contributed by members of the OSDEV forum. This information was collected from a thread at OSDEV forums that asked for members to submit knowledge that they have gained while making mistakes. I tried to collect the most useful of that knowledge and to place it here in a more organized manner. If you would like to view the original thread, then please scroll to the bottom of this page for the link.

==Getting Started==
*'''Design Phase:''' Design is something that should be done on paper first. I made the newbie mistake to rush right into it, and I finally discovered that I needed to redo (almost rewrite) most of my code, because I decided on a different design midway through, and I am still fighting with all the problems these changes have produced. And, each time I veer in a new direction, change destroys pieces of old logic. The amount of time I spend fixing those mistakes, sometimes rewriting entire files of code, all this could have been avoided.

*'''TODO LIST:''' A todo list is worth its weight in gold if done properly. I keep a spiral notebook that I use to keep track of what needs to to be done and in what order. I keep notes about what bugs I have and a description of how to replicate it, etc. I have never been more organized and productive in my life. Thank you, todo list.

*'''Build Your Toolbox:''' Once I start typing the same compound statement two or three times, I make it an alias or a shell function and put it in my ~/.bashrc (Linux or Cygwin, doesn't matter.). What is useful strongly depends on your environment. Don't waste your time typing trivial things, automate them. (And keep the code around so you can transfer it to other systems as necessary.)

*'''Automation:''' I made a simple script to perform all steps necessary to build the different versions of my kernel and run them using the different environments mentioned above. It calls autoconf, automake etc. if necessary, configures with the desired kernel options (i686, x86_64, with or without ACPI / SMP support...), builds the kernel and drivers, creates a boot floppy... This simplifies the build process a lot.

==Backups/Subversion==
*'''Backups:''' Make backups just as often as you make significant progress. Either place your new code online or offline somewhere. Do not replace any confirmed to be working code with unproven new code without first testing the new code on real hardware. This is especially true once you've begun developing a GUI. You'll do most of your development in an emulated environment, but all final cuts should be made in a real environment, where it will one day be forced to survive.

*'''Use version control software:''' Strong contenders today are git, Mercurial, and SVN. But don't be afraid to use even lowly RCS if in a pinch. (I use RCS regularily when hacking a script or a config file.) Nothing beats the ability to go back in time to check when something broke, and which change broke it.

*'''Use a issue tracker:''' Mantis, Bugzilla, Trac - again the tool doesn't matter much as long as you use it. (Of course you could use full backups of your source tree and a CHANGES.txt file instead. That would still be better than nothing, but those tools I mentioned above are there for a reason - they add huge amounts of productivity.)

*'''Control your changesets:''' One changeset should address one issue only, and should be describable with a single line checkin comment. If the issue is too complicated for a one-liner, create a ticket in your issue tracker, document things there, and reference the ticket number in your checkin comment.

==Testing/Debugging==
*'''Assert:''' It costs nothing (if NDEBUG'd out) at runtime and provides a commentary if correctly used.

*'''Testing 1, 2, 3:''' Every test should be done at least twice if the first test failed. Sometimes bugs work themselves out after the second or so try.

*'''Keep functions small and test early:''' So you just completed you 2K-LOC function. Great. Now, how are you going to find which parts need testing in it ? It's better to make small functions, and to test those as soon as they are completed in a multitude of way. Just another good general-purpose programming practice, that becomes especially important when you get into OSdeving.

*'''Write good test suites:''' If possible, write a test case for every feature you add. When you discover a bug, fix it and add a test case for it to your test suite. Don't forget test cases which should fail.

*'''Real v Fake Environments:''' Fake Environments are where you develop the driver. Real Environments are where you test them.

*'''Multiple test environments:''' I found it quite useful to have a multitude of test environments for running my kernel. For example, I use several Bochs builds (32 / 64 bit, SMP / UP...), QEMU, AMD SimNow! (SMP / UP), different VirtualBox machines and sometimes even MS VirtualPC. If the code runs fine on most of them but fails on a single one, there must be something going wrong. This way I found quite a lot of problems in my code that were silently ignored by some of these virtual machines.

==Sound Advice==
*'''Drivers and Apps:''' It is stupid to mix your drivers and applications into one file. This is what you do when you're testing, make sure to strip out those codes and to put them in another file when your driver is done. You would severely limit your driver otherwise. Don't even use a printx inside the driver.

*'''Be Consistent:''' Always try to do things the same way. e.g. All my for loops look the same: for (i=0; i<n; ++i) etc.

*'''Driver Dev:''' I discovered that it was much easier to develop drivers in VGA mode. So, when I developed my GUI, I found that I needed to write a mouse driver (for example) and that doing so was difficult trying to test and debug it in the GUI. So, I stripped out my VBE code and created a complete new environment that was like my GUI environment, except it was much easier to debug code in. And any changes that I made to the kernel would not effect my real project.

*'''JavaScript:''' I discovered that it was quicker and much easier to write some of my code in JavaScript first. I'm talking more about the logical parts of my drivers and design. Such as the scroll bar, or my browser window, even something math related. For example, I worked out the logic for both my browser (how it will work with icons and a mouse) and the scroll bar (how it will work when 10 icons are there and how it will work when n amount of icons are there, and what it will do when I drag the track bar or click the up or down arrow buttons, etc) using JavaScript and HTML. And, it honestly made the entire process much quicker since I didn't need to compile my kernel, assemble my loader, wait for the VM to load, wait for the VM to load my loader, wait for my loader to load my kernel, invoke the browser/scroll bar, reboot, and repeat.

*'''Syntax highlighting:''' It makes it a lot easier to see what some code does, or to find something in your code, if you use an editor with syntax highlighting. I wouldn't want to miss it.

*'''Userspace:''' It's worth writing your userspace in such a way that it can be easily runned on the host OS, where debugging is much easier.

*'''Anticipate:''' If something can go wrong, it will go wrong someday. A lost pointer down in the kernel because of unexpected return can be worth years of trying to fix the floating fallout bugs in userspace. "I'll add the code to check for that highly unlikely condition later" attitude guarantees that it won't be added.

*'''Cornercase design:''' Do not work from the average case when building a design, but instead work from all the possible extremities. This yields a design which covers everything cleanly, instead of one that uses a bunch hacks to support non-standard cases. Use refactor-first approaches when you find new cornercases for the same reason.

==Original Thread==
Please visit http://forum.osdev.org/viewtopic.php?f=11&t=22004&start=0 to view the original thread where this all began.
````

<a id="w002876"></a>

## W002876 — UDF

Bron-ID: W002876
Namespace: Main (0)
Revisie: 16514 | Datum: 2014-07-03T20:56:21Z
Bron: https://wiki.osdev.org/UDF
Revisie-URL: https://wiki.osdev.org/index.php?title=UDF&oldid=16514

````text
{{Filesystems}}

The '''Universal Disc Format''' or '''UDF''' is the filesystem used on DVDs and Blu-Ray discs.  It is similar to the CDFS (ISO 9660) file system, but it includes many extended features, such as incremental updates, which allows files to be deleted or updated over time.

== Volume Structure ==

Like the CDFS file system, the UDF file system uses a 2048 byte sector size, and it designates that the first 16 sectors can be used by the OS to store proprietary data or boot logic.  It also, like CDFS, uses 2048 byte aligned descriptors, starting at sector 16, to record the UDF specific information to locate additional volumes, partitions and files located on the disc.  It also supports the inclusion of descriptors from other file systems for compatibility with other media formats, operating systems and devices.

The descriptors located at sector 16 (0x10) all use the same general layout as the CDFS volume descriptor:

{| {{wikitable}}
! Offset
! Length (bytes)
! Field name
! Datatype
! Description
|-
| 0 || 1 || Type || int8 || Volume Descriptor type code (see below).
|-
| 1 || 5 || Identifier || strA || Designates the type of descriptor.
|-
| 6 || 1 || Version || int8 || Volume Descriptor Version (0x01).
|-
| 7 || 2041 || Data || - || Depends on the volume descriptor type.
|}

The identifier field is used to determine the type of the descriptor:
{| {{wikitable}}
! Identifier
! Description
|-
| "CD001" || Indicates that this volume contains an ISO 9660 file system.
|-
| "BEA01" || Denotes the beginning of the extended descriptor section.
|-
| "NSR02" || Indicates that this volume contains a UDF file system.
|-
| "NSR03" || Indicates that this volume contains a UDF file system.
|-
| "BOOT2" || Includes information concerning boot loader location and entry point address.
|-
| "TEA01" || Denotes the end of the extended descriptor section.
|-
|}

The presence of an NSR02 or NSR03 descriptor denotes that there is a valid UDF file system located on this volume, and that there is an Anchor Volume Descriptor Pointer located in sector 256 (0x100), with backup copies located in the last sector of the volume and/or the sector located 256 sectors from the end of the volume.

== Anchor Volume Descriptor Pointer ==

The Anchor Volume Descriptor Pointer is a descriptor that contains the location of the Main Volume Descriptor and the backup Reserve Volume Descriptor.

{| {{wikitable}}
! Offset
! Length (bytes)
! Field name
! Datatype
! Description
|-
| 0 || 16 || DescriptorTag || struct|| Contains the descriptor type, version and checksum information for this descriptor.
|-
| 16 || 8 || MainVolumeDescriptorSequenceExtent || struct || Contains the sector and length of the main volume descriptor sequence.
|-
| 24 || 8 || ReserveVolumeDescriptorSequenceExtent || struct || Contains the sector and length of the backup volume descriptor sequence.
|-
| 32 || 480 || Reserved|| - || Reserved for future use.
|}

== Descriptor Tag ==

The descriptor tag structure is used by all UDF descriptors to designate the descriptor type and to verify data integrity.

{| {{wikitable}}
! Offset
! Length (bytes)
! Field name
! Datatype
! Description
|-
| 0 || 2|| TagIdentifier || uint16 || Contains the descriptor type.
|-
| 2 || 2 || DescriptorVersion || uint16 || Contains the version of the descriptor tag.
|-
| 4 || 1 || TagChecksum || uint8 || Used to verify the data within the Descriptor Tag structure.
|-
| 5 || 1 || Reserved || uint8 || Reserved for future use.
|-
| 6 || 2 || TagSerialNumber|| uint16 || Unique serial number for this volume
|-
| 8 || 2 || DescriptorCRC|| uint16 || Used to verify the data with the Descriptor Tag structure.
|-
| 10 || 2 || DescriptorCRCLength|| uint16 || Used to verify the data with the Descriptor Tag structure.
|-
| 12|| 4 || TagLocation || uint32 || Contains the sequence number of this descriptor.
|}

The TagIdentifier field will have one of the following values:

{| {{wikitable}}
! Identifier (Hex)
! Description
|-
| 0x0001 || Primary Volume Descriptor
|-
| 0x0002 || Anchor Volume Descriptor Pointer
|-
| 0x0003 || Volume Descriptor Pointer
|-
| 0x0004 || Implementation Use Volume Descriptor
|-
| 0x0005 || Partition Descriptor
|-
| 0x0006 || Logical Volume Descriptor
|-
| 0x0007 || Unallocated Space Descriptor
|-
| 0x0008 || Terminating Descriptor
|-
| 0x0009 || Logical Volume Integrity Descriptor
|-
| 0x0100 || File Set Descriptor
|-
| 0x0101 || File Identifier Descriptor
|-
| 0x0102 || Allocation Extent Descriptor
|-
| 0x0103 || Indirect Entry
|-
| 0x0104 || Terminal Entry
|-
| 0x0105 || File Entry
|-
| 0x0106 || Extended Attribute Header Descriptor
|-
| 0x0107 || Unallocated Space Entry
|-
| 0x0108 || Space Bitmap Descriptor
|-
| 0x0109 || Partition Integrity Entry
|-
| 0x010a || Extended File Entry
|-
|}

== Extent Structure ==
The extent structure is used to locate data on the current volume.  It contains the sector index (relative to the beginning of the volume), and the length of the data.

{| {{wikitable}}
! Offset
! Length (bytes)
! Field name
! Datatype
! Description
|-
| 0 || 4 || Length|| uint32 || Contains the length, in bytes, of the data pointed to by this extent.
|-
| 4 || 4 || Location || uint32 || Contains the sector index of the data, relative to the beginning of the volume.
|}

== Volume Descriptor Sequence ==
The Main (and Reserve) Volume Sequence can be found by reading the extent information located in the Anchor Volume Descriptor Pointer record above.  Starting at the sector specified in the Location field of the extent structure, several Volume Descriptors will be found.  Each of these descriptors will provide additional information about this volume.  The Terminating Descriptor (0x0008) marks the end of the Volume Descriptor Sequence.

Note: Only Volume Descriptors (0x0001 to 0x0009) are allowed in the Volume Descriptor Sequence.

Note: The UDF Specifications state that the minimum extent size for the Volume Descriptor Sequence is 16 sectors, so there may be empty descriptor blocks after the Terminating Descriptor block, which can be ignored.

== Logical Volume Descriptor ==
The Logical Volume Descriptor (0x0006) contains information concerning the Logical Volume, including a pointer to the File Set Descriptor.

{| {{wikitable}}
! Offset
! Length (bytes)
! Field name
! Datatype
! Description
|-
| 0 || 16 || Tag || struct || The volume tag for this descriptor.
|-
| 16 || 4 || Volume Sequence Number || uint32 || The sequence number of this Logical Volume.
|-
| 20 || 64 || Descriptor Character Set || struct || The character set used in this descriptor.
|-
| 84 || 128 || Logical Volume Identifier || Compressed Unicode || The name of the logical volume.
|-
| 212 || 4 || Logical Block Size || uint32 || The block size of the logical volume.
|-
| 216 || 32 || Domain Identifier || struct || The domain identifier for this volume.
|-
| 248 || 16 || Logical Volume Contents Use || byte[] || Area reserved for content use.
|-
| 264 || 4 || Map Table Length || uint32 || The length of the map table in bytes.
|-
| 268 || 4 || Number Of Partition Maps || uint32 || The number of partition map records in this descriptor.
|-
| 272 || 32 || Implementation Identifier || struct || The implementation identifier for this volume.
|-
| 304 || 128 || Implementation Use || byte[] || Area reserved for implementation use.
|-
| 432 || 8|| Integrity Sequence Extent || struct || The extent of the integrity sequence, if any.
|-
| 440 || * || Partition Maps || struct[] || The map of partitions on this logical volume.
|}



== External Links ==

*[http://www.osta.org/specs/ OSTA - Specifications]

[[Category:Filesystems]]
````

<a id="w002877"></a>

## W002877 — ARM System Calls

Bron-ID: W002877
Namespace: Main (0)
Revisie: 30643 | Datum: 2026-05-27T15:12:59Z
Bron: https://wiki.osdev.org/ARM_System_Calls
Revisie-URL: https://wiki.osdev.org/index.php?title=ARM_System_Calls&oldid=30643

````text
{{Template:In Progress}}
==System Calls==

<syntaxhighlight lang="asm">swi 0x420000</syntaxhighlight>
This is how you call the system on ARM. The instruction <span style="font-family:monospace">swi</span> jumps to a predefined address, which in turn jumps to the system call handler. The system call handler executes the specific function and return to the user code with:
<syntaxhighlight lang="asm">mov pc, lr</syntaxhighlight>
Most of the time you won't need to worry about returning from the interrupt, as GCC, set up to cross compile for ARM, lets you code the interrupt handlers in C:
<syntaxhighlight lang="c">void swi_handler () __attribute__((interrupt));</syntaxhighlight>

==Creating System Calls==
<syntaxhighlight lang="asm">
interrupt_vector_table:
    b . @ Reset Handler
    b . @ Undefined
    b . @ SWI Handler
    b . @ Prefetch Abort
    b . @ Data Abort
    b . @ IRQ
    b . @ FIQ
</syntaxhighlight>
This is the ARM equivalent to the IDT, on the x86, and it is stored by default at address 0. The only entry, we need to worry about is the SWI Handler. To install our own SWI handler, we replace the <span style="font-family:monospace">b .</span> instruction with a branch to our handler:

<syntaxhighlight lang="asm">
interrupt_vector_table:
    b . @ Reset Handler
    b . @ Undefined
    b swi_handler @ Our new SWI Handler
    b . @ Prefetch Abort
    b . @ Data Abort
    b . @ IRQ
    b . @ FIQ
</syntaxhighlight>

We can code the interrupt handler like this:
<syntaxhighlight lang="c">
void __attribute__ ((interrupt ("SWI"))) swi_handler (void) {}
</syntaxhighlight>
Parameters to functions on ARM, are passed in registers r0-r3, if follow the same convention for system calls, then our interrupt handler can take parameters:
<syntaxhighlight lang="c">
void __attribute__ ((interrupt ("SWI"))) swi_handler (int r0, int r1, int r2, int r3) {}
</syntaxhighlight>
You've probably noticed that <span style="font-family:monospace">swi</span> takes an integer argument. To get this argument in C, we have to do this:
<syntaxhighlight lang="c">
uint8_t int_vector = 0;
asm volatile ("ldrb %0, [lr, #-2]" : "=r" (int_vector));
</syntaxhighlight>
This loads the high 8 bits (16-23) of the argument into <span style="font-family:monospace">int_vector</span>, as loading the full 24-bits won't work using Thumb.

[[Category:ARM]]
[[Category:System Calls]]
[[Category:In Progress]]
````

<a id="w002879"></a>

## W002879 — User:Pinged/TR18037

Bron-ID: W002879
Namespace: User (2)
Revisie: 10400 | Datum: 2010-06-07T16:59:09Z
Bron: https://wiki.osdev.org/User:Pinged/TR18037
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APinged%2FTR18037&oldid=10400

````text
{{Template:In Progress}}

==What is TR18037?==
TR18037 is a set of extensions to the C standard library to make low level programming, including OSDev, easier. The features it adds include:
:* iohw.h - standard functions for port-io
:* stdfix.h - builtin fixed point types
:* Named address spaces
:* Named register storage classes

==iohw.h==
This header defines standard functions and types for port-io, these include:
<source lang="c">
typedef /* ioindex type */ ioindex_t;
typedef /* ioreg type */ ioreg;

/* Read from a port */
unsigned iord (ioreg port);
unsigned long iordl (ioreg port);
unsigned iordbuf (ioreg port, ioindex_t idx);
unsigned long iordbufl (ioreg port, ioindex_t idx);

/* Write to a port */
void iowr (ioreg port, unsigned val);
void iowrl (ioreg port, unsigned long val);
void iowrbuf (ioreg port, ioindex_t idx, unsigned val);
void iowrbufl (ioreg dev, ioindex_t idx, unsigned long val);


/* These functions are pretty much useless to x86 programmers, but those for ARM
   and other architectures might find them useful.
 */

/* Do a bitwise and on a port */
void ioand (ioreg port, unsigned val);
void ioandl (ioreg port, unsigned long val);
void ioandbuf (ioreg port, ioindex_t idx, unsigned val);
void ioandbufl (ioreg port, ioindex_t idx, unsigned long val);

/* Do a bitwise or on a port */
void ioor (ioreg port, unsigned val);
void ioorl (ioreg port, unsigned long val);
void ioorbuf (ioreg port, ioindex_t idx, unsigned val);
void ioorbufl (ioreg port, ioindex_t idx, unsigned long val);

/* Do a bitwise xor on a port */
void ioxor (ioreg port, unsigned val);
void ioxorl (ioreg port, unsigned long val);
void ioxorbuf (ioreg port, ioindex_t idx, unsigned val);
void ioxorbufl (ioreg port, ioindex_t idx, unsigned long val);

/* These functions are kind of useless to anyone in kernel space (ring 0) but could
   be useful for anybody writing a user mode hardware driver in a microkernel.
 */

/* Allow access to a group of IO ports */
void iogroup_acquire (/* iogroup specifier */ group);
/* Disable access to a group of IO ports */
void iogroup_release (/* iogroup specifier */ group);
/* Map group a to group b */
void iogroup_map (/* iogroup specifier */ a, /* iogroup specifier */ b);
</source>

===iord, iordbuf, iordl, iordbufl===

These functions, are usually equivalent to the in''x'' functions found in most operating systems, a simple implementation for iord would be:

<source lang="c">
inline unsigned iord (ioreg port)
{
    uint8_t val = 0; /* Read 1 byte from the port */
    asm volatile ("inb %1, %0" : "=a" (val) : "dN" (port));
    return val;
}
</source>

My implementation system adds another two functions iordw, iordbufw for reading 2 bytes from a port.

<source lang="c">
inline uint16_t iordw (ioreg port)
{
    uint16_t val = 0; /* Read 2 bytes from a port */
    asm volatile ("inw %1, %0" : "=a" (val) : "dN" (port));
    return val;
}
</source>

==stdfix.h==

''TODO''

==Named address spaces==

''TODO''

==Named register storage class==

This feature allows you to select the register a <span style="font-family:monospace">register</span> variable is placed in, GCC supports this feature albeit with a slightly different syntax.

<source lang="c">
/* TR18037 specified syntax: */
register /* register specifier */ unsigned val;

/* GCC extension syntax: */
register unsigned val asm ("register name here");
</source>

[[Category:In Progress]]
````

<a id="w002881"></a>

## W002881 — User:Bastl/Selector

Bron-ID: W002881
Namespace: User (2)
Revisie: 18214 | Datum: 2015-07-03T11:06:51Z
Bron: https://wiki.osdev.org/User:Bastl/Selector
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ABastl%2FSelector&oldid=18214

````text
Selectors are called the segment registers - CS, DS, ES, FS, GS - in protected mode, which are index registers holding an index pointer into a table (GDT, LDT), in other words holding indexes to a [[Descriptor]].

They look like this:
{| {{wikitable}}
|Name || Bit || Description
|-
! RPL
| 0, 1 || Requested Privilege Level. The CPU checks these bits before any selector is changed. Also system calls can be executed in userspace (ring 3, [https://en.wikipedia.org/wiki/Protection_ring see this]) without misfeasance using the ARPL (Adjust Requested Privilege Level) instruction.
|-
! TI
| 2 || Table index; 0 = GDT, 1 = LDT
|-
! Index
| 3..15 || Index to a [[Descriptor]] of the table. 
|}
````

<a id="w002882"></a>

## W002882 — Descriptor

Bron-ID: W002882
Namespace: Main (0)
Revisie: 28185 | Datum: 2023-07-10T17:32:31Z
Bron: https://wiki.osdev.org/Descriptor
Revisie-URL: https://wiki.osdev.org/index.php?title=Descriptor&oldid=28185

````text
Descriptors are describing System Segments, Memory Segments and Gates.

== System Segment Descriptor ==
{| {{wikitable}}
! Name
! Bit
! Full Name
! Description
|-
! Limit
| 0..15 || Limit 0..15 || Memory range.
|-
! BAddr
| 16..39 || Base Address 0..23|| Linear address where the memory range begins.
|-
! Typ
| 40..43 || Descriptor Type || Possible types :
{| {{wikitable}}
| 0000b=0x0 || unused (invalid Descriptor)
|-
| 0001b=0x1 || 80286-TSS, 16 bit
|-
| 0010b=0x2 || LDT
|-
| 0011b=0x3 || activ 80286-TSS, 16 bit
|-
| 1001=0x9 || 80386-TSS, 32 bit
|-
| 1011=0xB || activ 80386-TSS, 32 bit
|}
|-
! S
| 44 || Storage Segment || <b>= 0</b> for System Segments.
|-
! DPL
| 45,46 || Descriptor Privilege Level || Specifies which privilege level a process minimum must have to use this Descriptor. 
|-
! P
| 47 || Present|| Set this to 1 to be able to access the segment, otherwise a #NP (Segment not Present) exception is generated on access.
|-
! Limit
| 48..51 || Limit 16..19 || high part of the Limit.
|-
! AVL
| 52 ||  || .
|-
! 0
| 53 ||  || .
|-
! -
| 54 ||  || .
|-
! G
| 55 ||  || .
|-
! BAddr
| 56..63 || Base Address 24..31 || Higher part of the Base Address.
|}

== Memory Segment Descriptor ==
{| {{wikitable}}
! Name
! Bit
! Full Name
! Description
|-
! Limit
| 0..15 || Limit 0..15 || Memory range.
|-
! BAddr
| 16..39 || Base Address 0..23|| Linear address where the memory range begins.
|-
! A
| 40 ||  || 
|-
! WR
| 41 ||  || 
|-
! CE
| 42 ||  || 
|-
! EX
| 43 ||  || 
|-
! S
| 44 || Storage Segment || <b>= 1</b> for Memory Segments.
|-
! DPL
| 45,46 || Descriptor Privilege Level || Specifies which privilege level a process minimum must have to use this Descriptor. 
|-
! P
| 47 || Present|| Set this to 1 to be able to access the segment, otherwise a #NP (Segment not Present) exception is generated on access.
|-
! Limit
| 48..51 || Limit 16..19 || high part of the Limit.
|-
! AVL
| 52 ||  || .
|-
! 0
| 53 ||  || .
|-
! B
| 54 ||  || .
|-
! G
| 55 ||  || .
|-
! BAddr
| 56..63 || Base Address 24..31 || Higher part of the Base Address.
|}

== Interrupt Gate Descriptor ==
{| {{wikitable}}
! Name
! Bit
! Full Name
! Description
|-
! Offset
| 0..15 || Offset 0..15 || Lower part of the offset address (also known as pointer).
|-
! Selector
| 16..31 || Selector || [[Selector]] that specifies the segment of the function to call.
|-
! Reserved
| 32..35 || || 
|-
! 0
| 36..39 || - || Have to be 0.
|-
! Type
| 40..43 || Gate Type || Possible interrupt gate types :
{| {{wikitable}}
| 0b0101=0x5 || [[#I386 Task Gate|Task gate]]
|-
| 0b0110=0x6 || 16-bit [[#I386 Interrupt Gate|interrupt gate]]
|-
| 0b0111=0x7 || 16-bit [[#I386 Trap Gate|trap gate]]
|-
| 0b1110=0xE || 32-bit [[#I386 Interrupt Gate|interrupt gate]]
|-
| 0b1111=0xF || 32-bit [[#I386 Trap Gate|trap gate]]
|}
|-
! S
| 44 || Storage Segment || <b>= 0</b> for gates.
|-
! DPL
| 45,46 || Descriptor Privilege Level || Gate call protection. Specifies which privilege level the calling process minimum should have.
|-
! P
| 47 || Present || Set this to 1 to be able to access the segment, otherwise a #NP (Segment not Present) exception is generated on access.
|-
! Offset
| 48..63 || Offset 16..31 || Higher part of the offset.
|}

== Call Gate Descriptor ==
{| {{wikitable}}
! Name
! Bit
! Full Name
! Description
|-
! Offset
| 0..15 || Offset 0..15 || Lower part of the offset address (also known as pointer).
|-
! Selector
| 16..31 || Selector || [[Selector]] that specifies the code segment of the function to call.
|-
! ParamCnt
| 32..35 || uint32_t's to copy || Number of uint32_t stack arguments to copy on stack switch.
|-
! 0
| 36..39 || - || Have to be 0.
|-
! Type
| 40..43 || Gate Type || Possible call gate types :
{| {{wikitable}}
| 0b0100=0x4 || 16-bit [[#I386 Call Gate|Call gate]]
|-
| 0b1100=0xc || 32-bit [[#I386 Call Gate|Call gate]]
|}
|-
! S
| 44 || Storage Segment || <b>= 0</b> for gates.
|-
! DPL
| 45,46 || Descriptor Privilege Level || Gate call protection. Specifies which privilege level the calling process minimum should have.
|-
! P
| 47 || Present || Set this to 1 to be able to access the segment, otherwise a #NP (Segment not Present) exception is generated on access.
|-
! Offset
| 48..63 || Offset 16..31 || Higher part of the offset.
|}



[[Category:X86]]
[[Category:Memory Segmentation]]
````

<a id="w002883"></a>

## W002883 — Selector

Bron-ID: W002883
Namespace: Main (0)
Revisie: 10420 | Datum: 2010-06-11T16:02:25Z
Bron: https://wiki.osdev.org/Selector
Revisie-URL: https://wiki.osdev.org/index.php?title=Selector&oldid=10420
Doorverwijzing: User:Bastl/Selector

````text
#REDIRECT [[User:Bastl/Selector]]
````

<a id="w002884"></a>

## W002884 — Talk:C preprocessor

Bron-ID: W002884
Namespace: Talk (1)
Revisie: 10592 | Datum: 2010-08-06T01:57:37Z
Bron: https://wiki.osdev.org/Talk:C_preprocessor
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AC_preprocessor&oldid=10592

````text
Some of the things I see here I just cannot believe. First, assert() is very well-defined in <assert.h>, and if you want to disable it, you define NDEBUG instead of undefining the macro. typeof() is a GCC extension, using which I hesitate to advertise as "good practice". complain() isn't defined anywhere, and if you replace it with printf() you'll get into trouble because the condition will be interpreted as printf() format string. The alert() function overloading is C++, not C. lnDbg() is ugly as hell, and better done with a debugger anyway. I strongly recommend cleaning up this mess. If you don't, I will, and I won't be mercyful. ;-) -- [[User:Solar|Solar]] 14:59, 24 June 2010 (UTC)
:On second thought... was this ''intended'' to be a mock page to show why you ''shouldn't'' rely on macros (too much)? ;-) -- [[User:Solar|Solar]] 15:07, 24 June 2010 (UTC)
::Even if it was, newbies are stupid enough not to see that. There's a lot of stuff in here that's left unexplained, lousy and inconsistent naming, stub overuse, even blocks of code without indentation. And the real tricks such as the # operator are left undescribed. As far as I am concerned, you can have my canister of nitric acid and have fun. - [[User:Combuster|Combuster]] 15:56, 24 June 2010 (UTC)
Why exactly is there a page on the C preprocessor on an OS development wiki? Maybe I should make a tutorial on how to use GIMP to create a fancy OS splash screen... Also, some of the information is GCC-specific or just plain wrong (e.g., on the #include directive). I didn't bother fixing things because I want to hear your opinion on whether we should keep this article or not. --[[User:Love4boobies|Love4boobies]] 11:29, 5 August 2010 (UTC)
:I'm not really sure where the boundary is for what to put on the wiki. This article is one of those articles in the "grey zone", do we keep it or do we mark it as "not for this wiki"? You could say the same for e.g. Makefiles, I think the information is useful, but it doesn't directly apply to OSDev only, it applies to general programming (just like the C Preprocessor applies to general C programming and isn't directly related to OSDev). On one hand, it's useful to have an article like this, but if we keep this, where will it stop? What prevents people from making articles such as "The C Syntax" as well? I'm not really in favor of any choices, so I'm just stating the options. --[[User:Creature|Creature]] 18:59, 5 August 2010 (UTC)
::One of the reasons for which I disagree with this article is that C itself has nothing to do with operating systems. People could very well write their operating systems in assembly, Pascal, Ada, or even Java (although we don't have any articles explaining managed OSes yet). However, I do agree that C is the most popular language for this purpose. I can see it is noted as one of the items in the "Required Knowledge" section. So if it is a prerequisite, why are we teaching it? This isn't a C wiki... Just my $0.02. --[[User:Love4boobies|Love4boobies]] 01:57, 6 August 2010 (UTC)
````

<a id="w002885"></a>

## W002885 — User talk:AJ/Building GMP

Bron-ID: W002885
Namespace: User talk (3)
Revisie: 10473 | Datum: 2010-06-26T18:00:53Z
Bron: https://wiki.osdev.org/User_talk:AJ/Building_GMP
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AAJ%2FBuilding_GMP&oldid=10473

````text
== Remove redirect ==
Since the destination page no longer exists, can we remove this page? I'd go ahead and do it, but it seemed a little impolite to just barge into your profile and remove it from your user page. --[[User:Creature|Creature]] 18:00, 26 June 2010 (UTC)
````

<a id="w002889"></a>

## W002889 — GameBoy Advance Introduction

Bron-ID: W002889
Namespace: Main (0)
Revisie: 22870 | Datum: 2018-09-14T23:27:49Z
Bron: https://wiki.osdev.org/GameBoy_Advance_Introduction
Revisie-URL: https://wiki.osdev.org/index.php?title=GameBoy_Advance_Introduction&oldid=22870

````text
{{Tone}}

This article will be an introduction to the world of Nintendo GameBoy Advance.

== But it already has an OS? ==
A gameboy doesn't have an OS. It does however have a [https://web.archive.org/web/20140222222650/http://nocash.emubase.de/gbabios.htm BIOS] that shows the boot screen and contains some support routines that you can use. After a ROM is bootstrapped, most games ignore its presence entirely.

When you're developing a game or a OS for this Nintendo platform, you communicate directly with the hardware, different from what Sony PlayStation Portable does(that have an OS layer to help you to do all), if you want to put something on the screen, you need to do all on something like a "Protected Mode"(this was just to illustrate for the beginners), that is communicating with the hardware without any kind of OS layer, and the same for the audio, input and all...

Then, before you go inside this ARM world, you should know that the hardware will be all that you have, the address table of GameBoy Advance will your best friend on your journey.

== Specifications ==
:{|
|-
!Length:
| approximately 14.45cm
|-
!Width:
| approximately 2.45cm
|-
!Height:
| approximately 8.2cm
|-
!Mass:
| approximately 140g
|-
!Screen:
| 2.9&nbsp;inches reflective TFT LCD|thin-film transistor (TFT) color LCD
|-
!Power:
| 2,4 AA batteries
|-
!valign=top|Battery life:
| approximately 15 hours on average while playing Game Boy Advance games (also dependent on the Game Pak being played and the volume setting)
|-
!CPU:
| 16.8&nbsp;MHz 32-bit ARM7TDMI with embedded memory
|-
!Memory:
| 32 kilobyte + 96 kilobyte [[Video RAM|VRAM]] (internal to the [[CPU]]), 256 kilobyte [[dynamic random access memory|WRAM]] (outside the CPU).
|-
!Resolution:
| 240 × 160 pixels
|-
!valign=top|Color&nbsp;support:
| 15-bit RGB (16-bit color space using 5 bits depth per channel), capable of displaying 512 simultaneous colors in "character mode" and 32,768 (2<sup>15</sup>) simultaneous colors in "bitmap mode"
|}

===Game Boy Advance SP===
In early 2003, Nintendo introduced a new form-factor for the handheld, known as the Game Boy Advance SP (model AGS-001).  The redesigned unit resembles a pocket-size laptop computer, including a folding case approximately one-half the size of the original unit.  It also sports a rechargeable lithium ion battery, a significantly brighter LCD screen, and an internal front-light that can be toggled on and off. The redesign was intended to address some common complaints about the original Game Boy Advance, which had been criticized for being somewhat uncomfortable to use, especially due to an overly dark screen.

Around the same time as the release of the [[Game Boy Micro]], Nintendo released a new backlit version of the SP (model AGS-101) in North America (commonly referred to as the "GBA SP+", SPII, or SP2). The switch that controls the light now toggles between "normal" (which itself is already brighter than the original Game Boy Advance SP's screen), and "bright", an intense brightness level similar to an LCD television set.

===Game Boy Micro===
In September 2005, Nintendo released a second redesign of the Game Boy Advance. This model, dubbed the Game Boy Micro, is similar in style to the original Game Boy Advance's horizontal orientation, but is much smaller and sleeker. The Game Boy Micro also allows the user to switch between several colored faceplates to allow customization, a feature which Nintendo advertised heavily around the Game Boy Micro's launch. Nintendo also hoped that this "fashion" feature would help target audiences outside of typical video game players, much like its Wii. Unlike the previous Game Boy Advance models, Game Boy Micro is unable to support Game Boy and Game Boy Color titles. The Game Boy Micro did not make much of an impact in the video game market as it was overshadowed by Nintendo's other portable, the Nintendo DS.



[[Category:ARM]]
[[Category:Gameboy Advance]]
````

<a id="w002892"></a>

## W002892 — User:Tosi

Bron-ID: W002892
Namespace: User (2)
Revisie: 11277 | Datum: 2011-03-16T04:36:09Z
Bron: https://wiki.osdev.org/User:Tosi
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATosi&oldid=11277

````text
=APM Shutdown Code=
I have only tested this with Bochs, but I know it works there at least.  The processor must be in real mode for this to work, as it uses the BIOS and the real mode APM interface.

The code is released under the [http://en.wikipedia.org/wiki/MIT_License X11 License], also known as the MIT License(s).
<source lang="asm">
	; Test for APM, connect the interface, shut down
	mov ax, 0x5300	; int 15/ax=5300h = installation check
	xor bx, bx		; device id of bios = 0
	int 0x15
	jc no_apm
	mov dx, 0xFFFF	; for APM 1.0
	test al, al
	jz .apm1_0
	mov dx, 1		; APM 1.1+
.apm1_0:
	mov ax, 0x5301	; int 15/ax=5301 = connect to real mode interface
	xor bx, bx		; device id of bios
	int 0x15
	jnc .no_test
	cmp ah, 0x03	; If ah is 0x03, the interface was already connected
	jnz no_apm
.no_test:
	mov ax, 0x5308	; int 15/ax=5308h = enable/disable power management
	mov bx, dx		; either 0x0001 or 0xFFFF depending on APM version
	push bx
	mov cx, 1		; enable power management (for all devices)
	int 0x15
	jc no_apm
	mov ax, 0x5307	; int 15/ax=5307h = set power state
	pop bx			; all devices
	mov cx, 3		; off
	int 0x15
no_apm:
	; APM not supported, just reset instead (using 8042 KBC)
	in al, 0x64
	test al, 0x02
	jnz no_apm
	mov al, 0xFE
	out 0x64, al
	; OK, then reset by switching to protected mode without a good GDT
	mov eax, cr0
	or eax, 1
	mov cr0, eax
        lgdt [0]
	jmp 0xFF:0xBAD1
</source>
````

<a id="w002893"></a>

## W002893 — File:Gba screen specs.png

Bron-ID: W002893
Namespace: File (6)
Revisie: 10497 | Datum: 2010-06-30T17:15:35Z
Bron: https://wiki.osdev.org/File:Gba_screen_specs.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AGba_screen_specs.png&oldid=10497

````text

````

<a id="w002894"></a>

## W002894 — GameBoy Advance Barebones

Bron-ID: W002894
Namespace: Main (0)
Revisie: 24571 | Datum: 2020-04-08T20:31:54Z
Bron: https://wiki.osdev.org/GameBoy_Advance_Barebones
Revisie-URL: https://wiki.osdev.org/index.php?title=GameBoy_Advance_Barebones&oldid=24571

````text
{{BeginnersWarning}}
{{FirstPerson}}
{{Rating|2}}
Finally what you were expecting is going to be real. I'm going to show you the GameBoy Advance barebones.

== Requirements ==
* Some C/ARM Assembly knowledge
* A GameBoy Advance Emulator - [http://vba.ngemu.com/ VisualBoy Advance] is a good one
* [http://www.devkitpro.org/ DevkitPro] Toolchain

== The Screen ==
The Nintendo GameBoy Advance screen, is an LCD with 240x160 resolution, that is entirely refreshed every 60th of a second. After a scanline has been drawn (the HDraw period, 240 pixels), there is a pause (HBlank, 68 pixels) before it starts drawing the next scanline. Likewise, after the 160 scanlines (VDraw) is a 68 scanline blank (VBlank) before it starts over again. To avoid tearing, positional data is usually updated at the VBlank. This is why most games run at 60 or 30 FPS.
[[Image:Gba_screen_specs.png|frame|The diagram of the GameBoy Advance Display]]
{|
! Subject
! Length
! Cycles
|-
! style="color:green"|Pixel
| 1
| 4
|-
! style="color:green"|HDraw
| 240px
| 960
|-
! style="color:green"|HBlank
| 68px
| 272
|-
! style="color:green"|Scanline
| HDraw+HBlank
| 1232
|-
! style="color:green"|VDraw
| 160*Scanline
| 197120
|-
! style="color:green"|VBlank
| 68*Scanline
| 83776
|-
! style="color:green"|Refresh
| VDraw+VBlank
| 280896
|}

== Let's Code ==
Here I'm going to show you two ways to do it: One using Assembly(that it's better to begin with it, because you understand better how the machine works) and the second one using C. But it will be just a simple pixel-plotting at the time. Later I will add more tutorials about other things.

'''ARM Assembly'''
<source lang="asm">.arm
.text
.global main

.macro plot_pixel x, y, color
    mov r0, #0x6000000
    mov r1, \color
    ldr r2, =(\x+\y*240)*2
    strh r1, [r0, r2]
.endm

main:
    mov r0, #0x4000000
    mov r1, #0x400
    add r1, r1, #3
    str r1, [r0]

    plot_pixel 115, 80, #0x1F
    plot_pixel 120, 80, #0x03E0
    plot_pixel 125, 80, #0x7C00

infin:
    b infin</source>


'''C'''
<source lang="c">int main()
{
    *(unsigned int*)0x04000000 = 0x0403;

    ((unsigned short*)0x06000000)[115+80*240] = 0x001F;
    ((unsigned short*)0x06000000)[120+80*240] = 0x03E0;
    ((unsigned short*)0x06000000)[125+80*240] = 0x7C00;

    while(1);

    return 0;
}</source>

== Compiling ==
To compile the Assembly source code, you just need to do this:
<source lang="bash">arm-elf-gcc -mthumb-interwork -specs=gba.specs kernel.S
arm-elf-objcopy -O binary a.out a.gba</source>
To compile the C source, just use this Makefile:
<source lang="bash">PATH := $(DEVKITARM)/bin:$(PATH)

# --- Project details -------------------------------------------------

PROJ    := first
TARGET  := $(PROJ)

OBJS    := $(PROJ).o

# --- Build defines ---------------------------------------------------

PREFIX  := arm-eabi-
CC      := $(PREFIX)gcc
LD      := $(PREFIX)gcc
OBJCOPY := $(PREFIX)objcopy

ARCH    := -mthumb-interwork -mthumb
SPECS   := -specs=gba.specs

CFLAGS  := $(ARCH) -O2 -Wall -fno-strict-aliasing
LDFLAGS := $(ARCH) $(SPECS)


.PHONY : build clean

# --- Build -----------------------------------------------------------
# Build process starts here!
build: $(TARGET).gba

# Strip and fix header (step 3,4)
$(TARGET).gba : $(TARGET).elf
	$(OBJCOPY) -v -O binary $< $@
	-@gbafix $@

# Link (step 2)
$(TARGET).elf : $(OBJS)
	$(LD) $^ $(LDFLAGS) -o $@

# Compile (step 1)
$(OBJS) : %.o : %.c
	$(CC) -c $< $(CFLAGS) -o $@
		
# --- Clean -----------------------------------------------------------

clean : 
	@rm -fv *.gba
	@rm -fv *.elf
	@rm -fv *.o

#EOF</source>

== Explanation ==
This is a quick and dirty explanation of the earlier code. Those previously mentioned warped minds for whom this section is intended will probably prefer it that way. A more detailed discussion will be given later. 

As I said, GBA programming is low-level programming and sometimes goes right down to the bit. The ''0x04000000'' and ''0x06000000'' are parts of the accessible memory sections. These numbers themselves don't mean much, by the way; they just refer to different sections. There aren't really ''0x02000000'' between these two sections. As you can see in the memory map, these two sections are for the IO registers and VRAM, respectively. 

To work with these sections in C, we have to make pointers out of them, which is what the ''unsigned int*'' and ''unsigned short*'' do. The types used here are almost arbitrary; almost, because some of them are more convenient than others. For example, the GBA has a number of different video modes, and in modes 3 and 5 VRAM is used to store 16-bit colors, so in that case casting it to halfword pointers is a good idea. Again, it is not required to do so, and in some cases different people will use different types of pointers. If you're using someone else's code, be sure to note the datatypes of the pointers used, not just the names. 

The word at ''0400:0000'' contains the main bits for the display control. By writing 0x0403 into it, we tell the GBA to use video mode 3 and activate background 2. What this actually means will be explained in the video and bitmap mode chapters. 

In mode 3, VRAM is a 16-bit bitmap; when we make a halfword pointer for it, each entry is a pixel. This bitmap itself is the same size as the screen(240x160) and because of the way bitmaps and C matrices work, by using something of the form ''array[y*width + x]'' are the contents of coordinates(x, y) on screen. That gives us our 3 pixel locations. We fill these with three 16-bit numbers that happen to be full red, green and blue in 5.5.5 BGR format. Or is that RGB, I always forget. In any case, that's what makes the pixels appear. After that there is one more important line, which is the infinite loop. Normally, infinite loops are things to be avoided, but in this case what happens after ''main()'' returns is rather undefined because there's little to return to, so it's best to avoid that possibility.

== Links ==
*[http://nocash.emubase.de/gbatek.htm#gbalcdvideocontroller GBA LCD Video Controller]
*[http://gbadev.org/download.php?section=documentation&ID=7 GBA Reference Document]
*[http://gbadev.org/download.php?section=documentation&ID=4 GBA Quick Reference Guide Rev. 2]


[[Category:ARM]]
[[Category:Gameboy Advance]]
[[Category:Bare bones tutorials]]
````

<a id="w002895"></a>

## W002895 — Category:Gameboy Advance

Bron-ID: W002895
Namespace: Category (14)
Revisie: 10505 | Datum: 2010-06-30T18:31:14Z
Bron: https://wiki.osdev.org/Category:Gameboy_Advance
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AGameboy_Advance&oldid=10505

````text
This category is dedicated for Nintendo Gameboy Advance development.

[[Category:Platforms]]
````

<a id="w002896"></a>

## W002896 — Talk:Stack Smashing Protector

Bron-ID: W002896
Namespace: Talk (1)
Revisie: 16950 | Datum: 2014-10-22T23:28:49Z
Bron: https://wiki.osdev.org/Talk:Stack_Smashing_Protector
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AStack_Smashing_Protector&oldid=16950

````text
== Err... ==

I looked at the IBM page linked. It uses a random value for the ''whole'' guard, ''or'' the { 0, 0, '\n', 255 } sequence I changed the article to. No randomizing of only the 255. It doesn't explain anything about the "why", though, and if I see it correctly the page refers to a ''patch'' to GCC. We have no idea how GCC implements it internally in the current version. Some more information as to the significance of the { 0, 0, '\n', 255 } sequence would be nice. -- [[User:Solar|Solar]] 13:42, 2 July 2010 (UTC)

:[http://rocklinux.net/people/clifford/BufferOverflow/slide500.html]: "If no random numbers are available, the four bytes { 0, 0, "\n", 255 } are used. Most read-functions terminate when reading one of this values. So it's not possible to overwrite the stackframe pointer or the return address." -- [[User:Alfaomega08|Alfaomega08]] 03:53, 10 June 2012 (CDT)
````

<a id="w002900"></a>

## W002900 — FDC

Bron-ID: W002900
Namespace: Main (0)
Revisie: 10575 | Datum: 2010-07-31T11:03:50Z
Bron: https://wiki.osdev.org/FDC
Revisie-URL: https://wiki.osdev.org/index.php?title=FDC&oldid=10575
Doorverwijzing: Floppy Disk Controller

````text
#REDIRECT [[Floppy Disk Controller]]
````

<a id="w002901"></a>

## W002901 — User:Sigaren

Bron-ID: W002901
Namespace: User (2)
Revisie: 10606 | Datum: 2010-08-19T00:34:05Z
Bron: https://wiki.osdev.org/User:Sigaren
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASigaren&oldid=10606

````text
I'm new to the community here and currently organizing my ideas and resources for a hobbyist operating system project. I'm very familiar with C++ and x86-64 Assembly.
````

<a id="w002902"></a>

## W002902 — User:Rjah

Bron-ID: W002902
Namespace: User (2)
Revisie: 10614 | Datum: 2010-08-22T11:37:49Z
Bron: https://wiki.osdev.org/User:Rjah
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ARjah&oldid=10614

````text
Been on the forum a few years, new to the wiki. Developing a C++ kernel for PowerPC and x86.

[[User:Rjah/Getting started with PowerPC | Getting started with PowerPC]]
````

<a id="w002903"></a>

## W002903 — User:Rjah/Getting started with PowerPC

Bron-ID: W002903
Namespace: User (2)
Revisie: 10615 | Datum: 2010-08-22T11:42:10Z
Bron: https://wiki.osdev.org/User:Rjah/Getting_started_with_PowerPC
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ARjah%2FGetting_started_with_PowerPC&oldid=10615

````text
placeholder

# Intro
# OFW client interface
# OFW stdout/putChar
# main
# Booting
````

<a id="w002904"></a>

## W002904 — User:Primis

Bron-ID: W002904
Namespace: User (2)
Revisie: 29241 | Datum: 2024-09-09T15:22:56Z
Bron: https://wiki.osdev.org/User:Primis
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APrimis&oldid=29241

````text
Oh hello! I'm Primis, though you can call me Allie.

I've been a member of the OSDev Community since 2010, I make occasional edits to pages and work on my [https://github.com/primis/Apollo own kernel] on and off as time allows.

My day job is as an embedded firmware engineer.
````

<a id="w002905"></a>

## W002905 — Batch File

Bron-ID: W002905
Namespace: Main (0)
Revisie: 30613 | Datum: 2026-05-24T14:29:14Z
Bron: https://wiki.osdev.org/Batch_File
Revisie-URL: https://wiki.osdev.org/index.php?title=Batch_File&oldid=30613

````text
A Batch file is the Windows equivalent to a [[shell]] script. It is supported by [[wikipedia:Windows|Windows]], [[wikipedia:MSDOS|MS-DOS]], and OS/2.
Since DOS has no file extension mapping, the extension is hardwired to '.BAT' it can also be '.bat' but never a mixed case. However in Windows, They can also end in '.cmd'

== Contents Of A Batch File ==
A Batch file is a plain text file that is interpreted by a command processor, usually cmd.exe on Windows and COMMAND.COM on DOS.
=== Labels And GOTO ===
It features some flow control features through label's (prefixed by a ':') and GOTO's.
=== Echo === 
Echo allows you to print to the screen, and has a special function, "@echo off" to turn off the commands from the batch itself.
=== Pause ===
Pause allows you to make the user input a confirmation key press before continuing.
=== Comments ===
Comments in a Batch are single line, starting with '::' or 'rem'

=== SET ===
Set allows you to set variables in the shell. These variables can be used inside the batch and are local to the shell, meaning all future batch files can access them. But because they are global to the shell, if in Windows the shell is auto-closed, the variables are erased, one system set variable is PATH. To use a Variable, you must pre-fix and post-fix it with a '%'.
=== IF ===
IF is the only logic command of a Batch ELSE only exists as an extension of it, syntax is "IF [NOT] %VARIABLE [== %VAR2] COMMAND [ELSE OTHER.EXE]" 
=== Input ===
WARNING! This method only works in windows 2000 and up. If you need input in a batch file, you can use "SET /P VARIABLE=Message" where VARIABLE is a variable to set and Message is text to put before the prompt. If the user does not input any information, the variable is set to the previous value, if it is a new variable, it will remain undefined.
=== Reading Command Line Arguments ===
As there is no proper input method in a batch, command line arguments can be a valuable tool. To access them, you would use the numerical variables. For example, the first argument is %1, the second is %2, and so on.
=== Mathematical Equations ===
To preform any mathematical equations, you use "SET /a VARIABLE=n?p" where VARIABLE is the variable to set, n and p are values or variables, and '?' is the operator, the operators are as follows:
* Add                  +
* Subtract             -
* Multiply             *
* Divide               /
* Modulus              %
* AND                  &
* OR                   |
* XOR                  ^
* Left Bit-wise shift  <<

== Example ==
An Example Batch File can be seen below, it takes three command line arguments, the file is a simple calculator.
<syntaxhighlight lang="batch">
:: calc.bat -- Command line calculator, takes three arguments
IF %2==+ GOTO ADD
IF %2==- GOTO SUB
IF %2==* GOTO MUL
IF %2==/ GOTO DIV 
ECHO No Valid Operator Found
EXIT

:ADD
SET /a VAL=%1%+%2%
GOTO DISP

:SUB
SET /a VAL=%1%-%2%
GOTO DISP

:MUL
SET /a VAL=%1%*%2%
GOTO DISP

:DIV
SET /a VAL=%1%/%2%
GOTO DISP

:DISP
ECHO %VAL%
EXIT
</syntaxhighlight>

== See Also ==

=== External Links ===
*  [[wikipedia:Batch_file|Batch Files]] on Wikipedia

[[Category:File Formats]]
````

<a id="w002906"></a>

## W002906 — User:Hangin10/OS Kernel

Bron-ID: W002906
Namespace: User (2)
Revisie: 21919 | Datum: 2017-12-17T10:55:03Z
Bron: https://wiki.osdev.org/User:Hangin10/OS_Kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AHangin10%2FOS_Kernel&oldid=21919

````text

````

<a id="w002907"></a>

## W002907 — User:54616e6e6572

Bron-ID: W002907
Namespace: User (2)
Revisie: 10631 | Datum: 2010-08-24T14:48:12Z
Bron: https://wiki.osdev.org/User:54616e6e6572
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3A54616e6e6572&oldid=10631

````text
Tanner Gooding here, the arrogant, opinionated, and ostentatious developer. Although only being 17, I haven't attended school for over 2 years now and spend most of my time reading, researching, developing, and learning.

I nitpick over the finer details of programming and thus am writing my entire OS in assembly. I was actually once asked on #osdev why I don't write my OS in a high-level language, and as XgF so eloquently said, "Because _54 can't put up with a 5% inefficiency in code which he spends 1% of his CPU time on".

== Lepidoptera OS ==
Lepidoptera OS is my custom microkernel OS that I am currently developing. You can find my latest screenshot, as well as a list of current hardware features [http://forum.osdev.org/viewtopic.php?p=180599#p180599 here]. All it does, as of yet, is boot with a fancy loading screen and a limited range of hardware support.

Lepidoptera OS is currently undergoing major revision to make a public release.

== Useful Books & Manuals ==
* AMD64 Architecture Programmer's Manuals
* Intel® 64 and IA-32 Architectures Software Developer's Manuals
* Compilers: Principles, Techniques, & Tools - Second Edition (Purple Dragon Book)
* Design Patterns: Elements of Reusable Object-Oriented Software (Gang of Four Book)
* Michael Abrash's Graphics Programming Black Book
* The Graphics Gems Series & The Journal of Graphics, GPU, and Game Tools

== OSDev Wiki Contributions ==
* [[PCI]]
* [[PCI Express]]
````

<a id="w002908"></a>

## W002908 — Talk:SFS

Bron-ID: W002908
Namespace: Talk (1)
Revisie: 12180 | Datum: 2011-11-14T00:49:47Z
Bron: https://wiki.osdev.org/Talk:SFS
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ASFS&oldid=12180

````text
== A couple of questions ==
I had a couple of questions regarding SFS; thing that AFAIK aren't (explicitely) stated in the spec nor on the wiki:
* What happens if the index area turns out to be too small to create a new entry? I assume the most feasible option would be to let it grow downwards (i.e. take space from the Free Space area).
* It is not uncommon for the Starting Marker entry (the one closest to the start of the disk) to not be aligned on BytesPerSector boundaries (e.g. at 128 bytes from a sector). The bytes before the starting marker (if any), are these actually part of the Free Area?
--[[User:Creature|Creature]] 21:06, 4 September 2010 (UTC)


:* Please ask on the forums, few people watch this place.
:* The index area is indeed grown when it is too small. The free space is designed so that it can be eaten away from both the front side (file data) and the rear side (index data). A similar description is in the specification.
:* Since the data section is block-aligned, you can't sensibly use the partial block leading the index. Doing so would result in an overlap: the last block of the file would use the same space as the index does, and you can't write out single blocks without having a special case everywhere. Similarly, the specification elaborates as follows: (emphasis mine)

 The end of the free space area is determined by subtracting the size of the index 
 area from the total size of the media '''and rounding it down to the nearest block size'''.
: - [[User:Combuster|Combuster]] 09:26, 6 September 2010 (UTC)

:: My apologies for asking this here, I can't even remember why I posted it here and not on the forums (or asked it on IRC). There were a couple of other questions I wanted to ask, but I seem to have forgotten them. I'll ask them on IRC/the forums if I manage to remember them. Thanks for the quick answers ;). --[[User:Creature|Creature]] 09:49, 6 September 2010 (UTC)

== Clarification ==

I think it should be noted somewhere that there are other file systems known by the abbreviation SFS, such as AROS's Smart File System, which are totally unrelated to this SFS
````

<a id="w002909"></a>

## W002909 — TrueType Fonts

Bron-ID: W002909
Namespace: Main (0)
Revisie: 29690 | Datum: 2025-09-23T20:38:56Z
Bron: https://wiki.osdev.org/TrueType_Fonts
Revisie-URL: https://wiki.osdev.org/index.php?title=TrueType_Fonts&oldid=29690

````text
{{Stub}}
[[Category:Graphical UI]]
== Description ==

TrueType is a method of encoding font information into a file.  It was created by Apple in the 1980s and is widely used today.

TrueType defines each glyph (character shape) by using a series of straight lines and quadratic Bézier curves.  This approach means that each character is a vector image and can be easily scaled up as required.

The TrueType file format is highly-space efficient and easy to read, using two simple primitives to draw the individual character glyphs.

The file format is [[Endianness|big endian]] throughout.

== File Format ==

The TypeType Font file contains a number of tables, the most significant of which are:
*'cmap' which maps individual character codes to glyph indices,
*'loca' which maps glyph indices to offsets into the main glyph table, and
*'glyf' which stores the actual glyph vector data itself.

== Displaying a character ==

The general order of operations to display a character is as follows:
*Find a suitable character map is the 'cmap' section of the file for the encoding of character code you have.
*Use the character map to map the character code to a Glyph Index.
*Lookup the Glyph index in the 'loca' table to find the offset into the glyph table where this glyph starts.
*Lookup the Glyph index + 1 in the 'loca' table to find the offset for the following glyph, this allows us to calculate the glyph data length.
*Locate the glyph data and plot it.

== The Glyph Data In-Depth ==

The glyph data itself describes a set of coordinate points that define either straight lines or Bézier curves to draw the actual glyph image.  It is described by a block of data from the 'glyf' section of the file.

The glyph data contains:
*A count of the contours,
*The bounding box for the character data,
*An array of 1-based point indices indicating the last point for each contour,
*Zero or more of "grid-fitting" hints,
*Flags defining attributes of points,
*X coordinate data, and
*Y coordinate data.

=== Contours ===
Contours are a collection of subsequent points that form a closed loop.  Some characters have only one contour to draw them, others have more.  Here are a few examples:
*'-' (hyphen) has only one contour, the path around the outside of the bar
*'1' (digit one) also has only one contour, around the outside of the shape
*'O' (capital oh) has two contours, one around the outside of the shape, and one around the inside
*'=' (equals) has two contours, the path around each of the bars
*'%' (percent) has five contours, the path around the bar and a path inside and outside each of the circles

The glyph data stores a count of the contours for the glyph and also an array of the last point for each contour.  By reading the point index for the last contour, you can determine the number of points in total that make up this glyph.

Note that the file does explicitly close a contour.  For example, a square will be defined by four points and it is up to the drawing code to draw the fourth side of square by drawing from the fourth point back to the first point.

=== "Grid-fitting" hints ===
A glyph can optionally contain "grid-fitting" hints.  These are instructions provided by the font designer to specify pixel level details that should be included when the font is rendered as a bitmap (rasterised).  These are not covered by this article.

=== Point Data ===
The remainder of the glyph data forms three byte arrays, the flags, the X coordinates, and the Y coordinates.

The points of the file are defined by a series of flag bytes in the glyph data, each flag represents one or more points in the glyph and as such the flags have to be parsed until the correct number of points has been found.  This is the only way to determine how many bytes of flag data is present in the file.

Each flag byte indicates a number of details about the point or points that it represents:
*Whether the point is On Curve of Off Curve,
*Whether there are zero, one or two bytes of data for the X coordinate,
*Whether there are zero, one or two bytes of data for the Y coordinate, and
*How many points this flag represents.

=== Drawing the Points ===
The On Curve / Off Curve flag is used to identify if the particular point lies on the outline of the character, or is a control point for a Bézier curve.  By analysing the on /off curve bit of the flag, the type of line required can be determined:
*OnCurve point to OnCurve point:  This is a straight line segment.
*OnCurve, OffCurve, OnCurve:  This is a quadratic Bézier.
*OnCurve, OffCurve, OffCurve, OnCurve:  This is two quadratic Béziers.

This last type of arrangement is a nuance of the file format.  Whilst the file encodes four points in the form of "OnCurve, OffCurve, OffCurve, OnCurve", it actually represents five points in the form of "OnCurve, OffCurve, OnCurve, OffCurve, OnCurve".  The extra OnCurve point is added in to form two quadratic Béziers.  (Whilst I cannot find any definitive information on this, I believe the OnCurve point should be located half-way between the two OffCurve points.)

== External Links ==

*The Apple specification https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6.html
*An excellent reference http://j2eedocument.appspot.com/2009/10/144-truetype-fonts.html (Not working)
*PD (stub)implementation of FreeType https://web.archive.org/web/20130902101234/http://mollyrocket.com/forums/viewtopic.php?t=736

[[Category:Fonts]]
````

<a id="w002910"></a>

## W002910 — Talk:Real Mode

Bron-ID: W002910
Namespace: Talk (1)
Revisie: 11158 | Datum: 2011-02-05T14:24:59Z
Bron: https://wiki.osdev.org/Talk:Real_Mode
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AReal_Mode&oldid=11158

````text
== Real mode can only address 1 MB? ==

I don't think that this statement is entirely accurate. Thanks to a BIOS function (int 15/ah=87), it's possible for real mode OS's to use memory past the 1 MB mark - all the way up to 4 GB, actually. I can't speak to how effective it is to utilize this function, but it does exist for use... it's a standard BIOS call. --[[User:Lithorien|Lithorien]] 20:17, 14 September 2010 (UTC)

: It isn't entirely accurate, on systems where more than 1MB exists, you can address the remaining physical memory by setting up page tables, tweaking the descriptor limits, among others (you're sure to find them if you search the wiki and read the Systems Developer Guides). AMD64 systems even include a mechanism so you can address the entire 64-bit address space from real mode (although I'm not sure how to do that on Intel64 systems). --[[User:54616e6e6572|54616e6e6572]] 22:56, 14 September 2010 (UTC)

:: You guys seem to be confused. In real mode, the CPU can only address 1 MiB + (64 KiB - 16 bytes). The BIOS function you're talking about doesn't move blocks from real mode, it switches operating modes back and forth transparently. You also seem to be confused about the existence of page tables in real mode - the 8086 did not support paging. Descriptor caches (responsible for the features you are describing - usually known as unreal mode) are an undocumented implementation decision which became de facto; you won't find anything about this in the manuals, you should read them sometime. When you do, also keep your last statement in mind. ;) --[[User:Love4boobies|Love4boobies]] 00:12, 17 September 2010 (UTC)

== There is a modern compiler which targets real mode ==

And that is [http://openwatcom.org/ OpenWatcom]. It includes a 16bit compiler which is actively maintained.--[[User:Badsector|Badsector]] 14:24, 5 February 2011 (UTC)
````

<a id="w002911"></a>

## W002911 — Open Host Controller Interface

Bron-ID: W002911
Namespace: Main (0)
Revisie: 29933 | Datum: 2026-01-27T02:39:37Z
Bron: https://wiki.osdev.org/Open_Host_Controller_Interface
Revisie-URL: https://wiki.osdev.org/index.php?title=Open_Host_Controller_Interface&oldid=29933

````text
The Open Host Controller Interface (OHCI) is an open standard that defines the interfaces between certain [[USB]] and [[Firewire]] drivers, hardware controllers, and devices.  It was originally published by Compaq, Microsoft and National Semiconductor in 1999.  Along with the Universal Host Controller Interface ([[UHCI]]), the OHCI makes up the USB 1.0 hardware controller interface standard.

== Technical Details ==
According to the OHCI standard, the OHCI driver communicates with the OHCI controller using two primary methods: Memory Mapped Registers, and Shared Memory Linked Lists.

The OHCI defines 21 memory mapped registers, exposed through BAR0:

{| {{wikitable}}
! Offset (Hex)
! Name
|-
| 00 || HcRevision
|-
| 04 || HcControl
|-
| 08 || HcCommandStatus
|-
| 0c || HcInterruptStatus
|-
| 10 || HcInterruptEnable
|-
| 14 || HcInterruptDisable
|-
| 18 || HcHCCA
|-
| 1c || HcPeriodCurrentED
|-
| 20 || HcControlHeadED
|-
| 24 || HcControlCurrentED
|-
| 28 || HcBulkHeadED
|-
| 2c || HcBulkCurrentED
|-
| 30 || HcDoneHead
|-
| 34 || HcFmInterval
|-
| 38 || HcFmRemaining
|-
| 3c || HcFmNumber
|-
| 40 || HcPeriodicStart
|-
| 44 || HcLSThreshold
|-
| 48 || HcRhDescriptorA
|-
| 4c || HcRhDescriptorB
|-
| 50 || HcRhStatus
|-
|}

In addition to these registers, starting at offset 54h, each USB port on the root hub is assigned an HcRhPortStatus register that denotes the current status of the port.

== PCI Configuration ==
The memory mapped registers listed above are not fixed in memory, and may be located at any 32-bit address regardless of how much memory is installed on the machine.  To find the base register address, the [[PCI]] bus can be searched for a specific vendor/device id, or a specific Class ID to find any OHCI controllers.  All OHCI controllers will have a Class ID of 0x0Ch, a subclass of 0x03h, and a program interface value of 0x10h.  The PCI device configuration area contains a single Base Address that points to the memory mapped register block.

== Shared Memory Linked Lists ==
All commands and data packets are supplied to the controller via a shared memory area containing tables and linked lists.  The USB protocol defines 4 different message types:  Interrupt, Isochronous, Control and Bulk.  Each message type is given its own linked list of outgoing messages.  As the controller processes these messages, they are moved to a corresponding "done" list, along with information concerning any errors encountered by the controller while delivering the message.

=== Control Messages ===
Control messages are used to initialize and configure a device that has been detected by a hub.  Devices that have not been initialized will not respond to any control messages.  To initialize a device, the hub port that the device is connected to must be reset.  Upon reset, the device will default to address 0, and will begin responding to any control messages sent to that address.  You should ensure that only one device is initialized at a time, and once the device's port has been reset, you should immediately send a SetAddress command to assign an unused address between 1 and 127 to the device.  Once this command has been sent, the device will begin responding to its new address, and will no longer respond to address 0.

The HcControlHead register points to the first control Endpoint Descriptor (ED) in a linked list of Endpoint Descriptors.  Each Endpoint Descriptor describes one endpoint on the USB bus that can be communicated with by the operating system.  To send or receive data from an endpoint, a Transfer Descriptor is added to an Endpoint Descriptor by the operating system.  Once the data has been transferred, the controller will move the Transfer Descriptor from the Endpoint Descriptor to the Done Queue.

Each Endpoint Descriptor has a "head" Transfer Descriptor field and a "tail" Transfer Descriptor field, as well as a "next" Endpoint Descriptor field.  The final Endpoint Descriptor in the Endpoint Descriptor chain will have a "next" Endpoint Descriptor value of zero.  When the "head" Transfer Descriptor matches the "tail" Transfer Descriptor, the Endpoint Descriptor is empty, and no more data will be sent.

The Endpoint Descriptors in the HcControlHead list can only be used to communicate with Control endpoints.  All devices have a control endpoint 0 that is used to configure the device.

=== Bulk Messages ===
The HcBulkHeadED register points to the first bulk Endpoint Descriptor in the bulk endpoint descriptor list, and is used to send and receive bulk data packets.  Otherwise, it is functionally identical to the HcControlHead register.

== Endpoint Descriptors ==
An Endpoint Descriptor is a memory structure that contains information used to communicate with a specific endpoint on a specific device address. The structure must be aligned on a 16-byte boundary.

{| {{wikitable}}
! Offset (Hex)
! Name
!Description
|-
| 00 || Control
|'''Bits 0:6''' - Function address
'''Bits 7:10''' - Endpoint address

'''Bits 11:12''' - Direction of dataflow (IN/OUT)

'''Bit 13''' - If set, this is a low speed endpoint, if zero, this is full speed endpoint

'''Bit 14''' - You set this if you want the host controller to skip processing this ED

'''Bit 15''' - If set, TDs linked to this ED contain isochronous transfers

'''Bits 16:26''' - Maximum packet size

'''Bits 27:31''' - not used by the host controller
|-
| 04 || TD Queue Tail Pointer
|Physical address of TD queue tail. If this dword and the next dword are equal, it means the TD list is empty and there are not TDs to process
|-
| 08 || TD Queue Head Pointer
|Physical address of TD queue head. As TDs are 16-byte aligned, the lower 4 bits are not used, so the host controller uses some of them as follows:
'''Bit 0''' - host controller sets this to tell you that processing of TD queue on this endpoint is halted (usually due to an error)

'''Bit 1''' - toggle carry bit from the last retired TD
|-
| 0c || Next ED
|Physical address of the next ED. If NULL, this is the last ED in the list
|-
|}

== Transfer Descriptors ==
Transfer Descriptors (TD) describe a memory buffer to be sent to, or read from, a USB endpoint.

There are 2 types: 

* '''General TD:''' used for interrupt, control and bulk transfers. Must be aligned on a 16-byte boundary.
* '''Isochronous TD:''' used for isochronous transfers. Must be aligned on a 32-byte boundary and they use a different descriptor structure.


'''General Transfer Descriptor Structure'''

{| {{wikitable}} cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
! Offset (Hex)
! Name
!Description
|-
| 00 || Control
|'''Bits 0:17''' - not used by the host controller
'''Bit 18''' - if set, the last packet may be smaller than the buffer size without causing an error

'''Bits 19:20''' - direction of transfer ('''00b''' SETUP; '''01b''' OUT; '''10b''' IN; '''11b''' reserved)

'''Bits 21:23''' - number of frames to wait after processing this TD and before sending an interrupt (a value of '''111b''' means there is no interrupt for this TD)

'''Bits 24:25''' - data toggle (DATA0/DATA1)

'''Bits 25:27''' - error count

'''Bits 28:31''' - status of last transaction
|-
| 04 || Current Buffer Pointer (CBP)
|Physical address of the first byte in the transfer buffer. Buffers can be upto 8,192 bytes in size, and can span upto 2 physically disjoint pages
|-
| 08 || Next TD (NextTD)
|Physical address of the next TD in the list for this endpoint
|-
| 0c || Buffer End (BE)
|Physical address of the last byte in the buffer (that is, bufsize - 1)
|-
|}

== See also ==

* OHCI specification: http://www.o3one.org/hwdocs/usb/hcir1_0a.pdf

[[Category:USB]]
[[de:Open Host Controller Interface]]
````

<a id="w002912"></a>

## W002912 — OHCI

Bron-ID: W002912
Namespace: Main (0)
Revisie: 10766 | Datum: 2010-09-21T06:40:33Z
Bron: https://wiki.osdev.org/OHCI
Revisie-URL: https://wiki.osdev.org/index.php?title=OHCI&oldid=10766
Doorverwijzing: Open Host Controller Interface

````text
#REDIRECT [[Open Host Controller Interface]]
````

<a id="w002913"></a>

## W002913 — File:Command list.jpg

Bron-ID: W002913
Namespace: File (6)
Revisie: 10774 | Datum: 2010-09-22T14:31:13Z
Bron: https://wiki.osdev.org/File:Command_list.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ACommand_list.jpg&oldid=10774

````text
Command List
````

<a id="w002914"></a>

## W002914 — File:Command table.jpg

Bron-ID: W002914
Namespace: File (6)
Revisie: 10775 | Datum: 2010-09-22T14:32:02Z
Bron: https://wiki.osdev.org/File:Command_table.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ACommand_table.jpg&oldid=10775

````text
Command Table
````

<a id="w002915"></a>

## W002915 — File:HBA registers.jpg

Bron-ID: W002915
Namespace: File (6)
Revisie: 10776 | Datum: 2010-09-22T14:32:26Z
Bron: https://wiki.osdev.org/File:HBA_registers.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AHBA_registers.jpg&oldid=10776

````text
HBA Registers
````

<a id="w002916"></a>

## W002916 — File:Port memory.jpg

Bron-ID: W002916
Namespace: File (6)
Revisie: 10777 | Datum: 2010-09-22T14:36:39Z
Bron: https://wiki.osdev.org/File:Port_memory.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3APort_memory.jpg&oldid=10777

````text
Port Memory
````

<a id="w002920"></a>

## W002920 — UDI Channels

Bron-ID: W002920
Namespace: Main (0)
Revisie: 16809 | Datum: 2014-09-24T23:20:30Z
Bron: https://wiki.osdev.org/UDI_Channels
Revisie-URL: https://wiki.osdev.org/index.php?title=UDI_Channels&oldid=16809

````text
[[Category:Uniform Driver Interface]]

A UDI "channel" is the nomenclature used in the UDI specification for an IPC ([https://en.wikipedia.org/wiki/Inter-process_communication Inter Process Communication]) link between two driver end-points. In other words, it is an abstraction of IPC on the host kernel which implements a compliant UDI Environment. The specific underlying mechanics of a particular kernel's implementation of UDI channel IPC, and how messages are sent across these channels are left up to the implementor.

=What is a UDI Channel?=
UDI channels are generally two-way communication links, with the exception of the channel formed between the kernel and the driver for the UDI Management Metalanguage Channel. Every other channel is a two-way communication link. The Management Metalanguage channel is two-way as well technically, but there is a constraint applied by the specification which states that drivers cannot '''initiate''' communications with the host kernel unless first asked to do so. The kernel can "post" requests to the driver on this channel ''if it is interested in communicating with the driver'', and the driver can "hold on" to them until it has an appropriate response. An example would be hotplug device enumeration, where ''if the kernel is interested'', it can "post" a request for the driver to "hold on to", and the driver will not respond until a hotplug event has occured. If the kernel did not "post" the request to indicate that it was interested in hotplug events, the driver cannot initiate a hotplug notification on its own.

The management metalanguage channel is the only channel with such a restriction.

=Underlying implementation details=
The specific underlying behaviour of IPC messaging "channels" is implementation defined. For a Separate Address Space Driver kernel, "channels" may be implemented as cross-process IPC using shared memory or message passing. For a purely monolithic kernel, or a Single Address Space kernel, "channels" can be simplified to nothing more than function calls. UDI does not impose any particular kernel design or driver environment design on kernels -- kernels are free to flexibly implement the interfaces in the UDI specification in a manner that suits them best, and it fastest and best optimized for their own situation, workload and requirements.

=Interfaces and structures=
==Key structures==
<source lang="c">
/* The opaque type used to refer to a channel. Channels are refered to via opaque handles,
 * and the underlying kernel implementation is hidden from drivers.
 */
typedef <HANDLE> udi_channel_t;

/* The core IPC message type used by UDI. Every IPC message contains
 * this base type, and new message types can be created by extending
 * this type. Every UDI IPC message (aka "control block") is based
 * on this type, the udi_cb_t structure.
 */
typedef struct {
	udi_channel_t channel;
	void *context;
	void *scratch;
	void *initiator_context;
	udi_origin_t origin;
} udi_cb_t;
</source>

==Key service calls==
<source lang="c">
/* Used to spawn new channels. */
void udi_channel_spawn(
	udi_channel_spawn_call_t *callback,
	udi_cb_t *gcb,
	udi_channel_t channel, 
	udi_index_t spawn_idx,
	udi_index_t ops_idx,
	void *channel_context);

/* Used to destroy currently active channels. */
void udi_channel_close(udi_channel_t channel);
</source>
````

<a id="w002921"></a>

## W002921 — User:Cb88

Bron-ID: W002921
Namespace: User (2)
Revisie: 10887 | Datum: 2010-10-21T22:01:23Z
Bron: https://wiki.osdev.org/User:Cb88
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACb88&oldid=10887

````text
Hey I'm cb88 I'm a computer engineering student in my senior year (2010-2011) and Sparc hardware is becoming my hobby. 

My current goal is to build a bootable CD image with buildroot and possibly write a small OS in C and assembly for mental exercise. Buildroot was chosen because I can build a toolchain with it that qemu can execute and the Leon Sparc developers also intend to support buildroot for commercial purposes. My other options were T2 SDE with lots of packages, Gentoo which IMO isn't ideal, and buildroot which uses uClibc exclusively. 

I have 1 and a half SparcStation 2s, 1 SparcStation 10 and a loaner aswell, 1 SparcStation 5 and an Ultra 1.   

Specwise they are as follows

<pre>
Sparcstation 2 #1 80Mhz 64mb ram .5 GB drive
Sparcstation 2 #2 40Mhz mostly for parts
SparcStation 5 70Mhz 144Mb ram running NetBSD currently
SparcStation 10 various modules from 40-60Mhz and dual 2Gb drives
Ultra 1 167Mhz probably the fastest it curretly has red hat 5.2 and KDE 1 :-) 
</pre>

If you have any sparc parts you think I would be interested in PM me on the forum and I'll see if I can help you will that ;-). 

I'm looking specifically for a Fujitsu AG-10E framebuffer (huge beast of an sbus card) for my SS10.
Also faster Ethernet cards would be nice for the most part I'm stuck at 10Mbs for now.  
I am interested in the external GT Tower I know its buggy and slow but for historical reasons its interesting. 
I could also use an SS20 + quad cpus once I get a native toolchain working if you need someone to take care of your baby and your wife is about to kill you if you don't ditch it! 

Do any Ross HyperStations still exist in the US?

Any info on Sparc hardware is also of interest to me I plan on hosting alot of the documentation myself since many websites seem to be dying off at the moment.
````

<a id="w002923"></a>

## W002923 — Why do I need a Cross Compiler?

Bron-ID: W002923
Namespace: Main (0)
Revisie: 30777 | Datum: 2026-06-18T20:49:41Z
Bron: https://wiki.osdev.org/Why_do_I_need_a_Cross_Compiler%3F
Revisie-URL: https://wiki.osdev.org/index.php?title=Why_do_I_need_a_Cross_Compiler%3F&oldid=30777

````text
::''Notice: ''This page is specific to GCC. If you use another compiler, you should research how cross-compilation is normally done with that compiler and do it that way. GCC is quite tightly bound to its native target system, many other compilers are not. Some compilers don't even have a native target, they are always cross-compilers, but you must explicitly tell them to cross-compile.

You need to use a [[GCC_Cross-Compiler|cross-compiler]] ''unless'' you are developing on your own operating system. The compiler ''must'' know the correct [[Target Triplet|target platform]] (CPU, operating system), otherwise you will run into trouble. You may be able to use the compiler that comes with your system if you pass a number of options to beat it into submission, but this will create a lot of completely unnecessary problems.

It is possible ask your compiler what target platform it is currently using by calling the command:

<source lang="bash">
 gcc -dumpmachine
</source>

If you are developing on 64-bit Linux, then you will get a response such as 'x86_64-unknown-linux-gnu'. This means that the compiler thinks it is creating code for Linux. If you use this GCC to build your kernel, it will use your system libraries, headers, the Linux [[libgcc]], and it will make a lot of problematic Linux assumptions. If you use a [[GCC_Cross-Compiler|cross-compiler]] such as i686-elf-gcc, then you get a response back such as 'i686-elf' that means the compiler knows it is doing something else and you can avoid a lot of problems easily and properly.

== How to build a Cross-Compiler ==
{{Main|GCC Cross Compiler}}

It is easy and takes a few moments to [[GCC_Cross-Compiler|build a cross-compiler]] that targets your operating system. It may take a while to build it on slower computers, but you only need to do it once, and you save all the time you would otherwise spend on "fixing" the completely unnecessary problems you would encounter otherwise. Later on, when you start building a user-space for your operating system, it is worth creating an [[OS_Specific_Toolchain|OS Specific Toolchain]] for absolute control of the compiler and to easy compiling user-space programs.

== Transitioning to a Cross-Compiler ==
Perhaps you have not been using a [[GCC_Cross-Compiler|cross-compiler]] until now, in which case you are likely doing a lot of things wrong. Unfortunately, a lot of kernel tutorials suggest passing certain options and doing things in a manner that potentially causes a lot of trouble. This section documents some of the things you should watch out for. Please read this section carefully and point others to it if you see them using troublesome options.

=== Linking with your compiler rather than ld ===
You shouldn't be invoking ld directly. Your cross-compiler is able to work as a linker and using it as the linker allows it control at the linking stage. This control includes expanding the <tt>-lgcc</tt> to the full path of [[libgcc]] that only the compiler knows about. If you get weird errors during compilation, use your cross-compiler for linking and it may go away. If you do need ld, be sure to use the cross-linker (i686-elf-ld) rather than the system linker.

=== Using cross-tools ===
You get a lot of useful programs when you build your cross-binutils. For instance, you get i686-elf-readelf, i686-elf-as, i686-elf-objdump, i686-elf-objcopy, and more. These programs know about your operating system and handle everything correctly. You can use some of the programs that come with your local operating system instead (readelf, objcopy, objdump) if they know about the file format of your operating system, but it is in general best to use your cross tools instead. These tools all consistently have the prefix 'i686-elf-' if the platform of your OS is i686-elf.

=== Options that you should pass to your Compiler ===
You need to pass some special options to your compiler to tell it it isn't building user-space programs.

==== -ffreestanding ====
This is important as it lets the compiler know it is building a kernel rather than user-space program. The documentation for GCC says you are required to implement the functions memset, memcpy, memcmp and memmove yourself in freestanding mode.

==== -mno-red-zone (x86_64 only) ====
You must pass this option on x86_64 for kernel code, or interrupts will corrupt the stack. The red zone is a x86_64 ABI feature that means that signals happen 128 bytes further down the stack. Functions that use less than that amount of memory is allowed to not increment the stack pointer. This ABI means that CPU interrupts in the kernel will corrupt the stack, because interrupts do not respect the red zone.

==== -fno-exceptions, -fno-rtti (C++) ====
It is wise to disable C++ features that don't work out-of-the-box in kernels. You need to supply a C++ support library to the kernel (in addition to libgcc) to make all C++ features work. If you don't use these C++ features, it should be sufficient to pass these options.

=== Options you should link with ===
These options only make sense when linking (not when compiling) and you should use them. You should pass the compilation options as well when linking, as some compilation options (such as <tt>-mno-red-zone</tt>) control the ABI and this needs to be known at link time as well.

==== -nostdlib (same as both -nostartfiles -nodefaultlibs) ====
The -nostdlib option is the same as passing both the -nostartfiles -nodefaultlibs options. You don't want the start files (crt0.o, crti.o, crtn.o) in the kernel as they only used for user-space programs. You don't want the default libraries such as libc, because the user-space versions are not suitable for kernel use. You should only pass -nostdlib, as it is the same as passing the two latter options.

==== -lgcc ====
You disable the important [[libgcc]] library when you pass -nodefaultlibs (implied by -nostdlib). The compiler needs this library for many operations that it cannot do itself or that is more efficient to put into a shared function. You must pass this library at the end of the link line, after all the other object files and libraries, or the linker won't use it and you get strange linker errors. This is due to the classic static linking model where an object file from a static library is only pulled in if it is used by a previous object file. Linking with [[libgcc]] must come after all the object files that might use it.

=== Options that you shouldn't pass to your Compiler ===
There is a number of options you normally shouldn't pass to your cross-compler when building a kernel. Unfortunately, a lot of kernel tutorials suggest you use these. Please do not pass a option without understanding why it is needed and don't suggest to people that they use them. Often, these options are used by those that don't use cross-compilers to cover up other problems.

==== -m32, -m64 (compiler) ====
If you build a cross-compiler such as i686-elf-gcc, then you don't need to tell it to make a 32-bit executable. Likewise, you don't need to pass -m64 to x86_64-elf-gcc. This will make your Makefiles much simpler as you can simply select the correct compiler and things will work. You can use x86_64-elf-gcc to build a 32-bit kernel, but it's much easier to just build two cross-compilers and use them. In addition, using a cross-compiler for every CPU you target will make it easy to port third-party software without tricking them into passing -m32 as well.

==== -melf_i386, -melf_x86_64 (linker) ====
You don't need to pass these for the same reason as -m32 and -m64. Additionally, these options are for ld, and you shouldn't be invoking ld directly in the first place, but rather linking using your cross-compiler.

==== -32, -64 (assembler) ====
The cross-assembler (i686-elf-as) defaults to the platform you specified when building binutils, and so you don't need to repeat the choice here. You can use the cross-compiler as an assembler, but it is okay to call the assembler directly.

==== -nostdinc ====
You shouldn't pass this option as it disables the standard header include directories. However, you do want to use these headers as they contain many useful declarations. The cross-compiler comes with a bunch of useful headers such as stddef.h, stdint.h, stdarg.h, and more.

If you don't use a cross-compiler, you get the headers for your host platform (such as Linux) which are unsuitable for your operating system. For that reason, most people that don't use a cross-compiler use this option and then have to reimplement stddef.h, stdint.h, stdarg.h and more themselves. People often implement those files incorrectly as you need compiler magic to implement features such as stdarg.h.

==== -fno-builtin ====
This option is implied by <tt>-ffreestanding</tt> and there is no reason to pass it yourself. The compiler defaults to -fbuiltin that enables builtins, but -fno-builtin disables them. Builtins mean that the compiler knows about standard features and can optimize their use. If the compiler sees a function called 'strlen', it normally assumes it is the C standard 'strlen' function and it is able to optimize the expression strlen("foo") into 3 at compile time, instead of calling the function. This option has value if you are creating some really non-standard environment in which common C functions don't have their usual semantics. It is possible to enable builtins again with <tt>-fbuiltin</tt> following <tt>-ffreestanding</tt> but this can lead to surprising problems down the road, such as the implementation of calloc (malloc + memset) being optimized into a call to calloc itself.

==== -fno-stack-protector ====
The [[Stack Smashing Protector]] is a feature that stores a random value on the stack of selected functions and verifies the value is intact upon return. This statistically prevents stack buffer overflows overwriting the return pointer on the stack, which would subvert control flow. Adversaries are often able to exploit such faults, and this feature requires the adversary to correctly guess a 32-bit value (32-bit systems) or a 64-bit value (64-bit systems). This security feature requires [[Stack Smashing Protector|runtime support]]. Compilers from many operating system vendors enable this feature by having <tt>-fstack-protector</tt> be the default. This breaks kernels that don't use a cross-compiler, if they don't have the runtime support. Cross-compilers such as the <tt>*-elf</tt> targets have the stack protector disabled by default and there's no reason to disable it yourself. You may want to change the default to enabling it when you add support for it to your kernel (and user-space), which would make it automatically used by your kernel because you didn't pass this option.

== Problems that occur without a Cross-Compiler ==
You need to overcome a lot of problems to use your system gcc to build your kernel. You don't need to deal with these problems if you use a cross-compiler.

=== More complicated compilation commands ===
The compiler assumes it is targetting your local system, so you need a lot of options to make it behave. A trimmed down command sequence for compiling a kernel without a cross-compiler could look like this:

<source lang="bash">
 as -32 boot.s -o boot.o
 gcc -m32 kernel.c -o kernel.o -ffreestanding -nostdinc
 gcc -m32 my-libgcc-reimplemenation.c -o my-libgcc-reimplemenation.o -ffreestanding
 gcc -m32 -T link.ld boot.o kernel.o my-libgcc-reimplemenation.o -o kernel.bin -nostdlib -ffreestanding
</source>

Actually, the average case is worse. People tend to add many more problematic or redundant options. With a real cross-compiler, the command sequence could look this this:

<source lang="bash">
 i686-elf-as boot.s -o boot.o
 i686-elf-gcc kernel.c -o kernel.o -ffreestanding
 i686-elf-gcc -T link.ld boot.o kernel.o -o kernel.bin -nostdlib -ffreestanding -lgcc
</source>

=== Reimplementing libgcc ===
You cannot use the host [[libgcc]] when building a kernel. The Linux [[libgcc]] has some nasty dependencies last timed I checked. The common case newbies run into is 64-bit integer division on 32-bit systems, but the compiler may generate such calls in many cases. You will often end up rewriting [[libgcc]] when you should have been using the real thing in the first place.

=== Rewriting freestanding headers (often incorrectly) ===
If you don't pass -nostdinc you get the target system headers (which is your local system if not using a cross-compiler), and that will cause a lot of problems in the non-cross-compiler case. You will end up rewriting the standard freestanding headers such as stdarg.h, stddef.h, stdint.h. and more. Unfortunately, as mentioned above, these headers need a bit of compiler magic to get just right. If you use a cross-compiler, all these freestanding headers can be used out of the box with no effort.

=== Complicated compiling user-space programs ===
You need to pass even more options to the command lines that build programs for your operating systems. You need a -Ipath/to/myos/include and -Lpath/to/myos/lib to use the C library, and more. If you set up an [[OS Specific Toolchain]], you just need

<source lang="bash">
 i686-myos-gcc hello.c -o hello
</source>

to cross-compile the hello world program to your operating system.

=== Compiler releases break your OS ===
Not everyone is using the same gcc as you are, which means that people on other operating systems (even versions, or compiler releases) will have trouble building your operating system correctly. If you use a cross-compiler, then everyone is using the same compiler version and assumptions about the host system won't make it into your operating system.

=== Support ===
You will have a much easier time getting support from the operating system development community. Properly using a cross-compiler shows that you have followed instructions and are at the same level as everyone else, and that your local system compiler isn't causing trouble.

=== And so on ===
As the project grows in size, it becomes much more complicated to maintain your operating system without a real cross-compiler. Even if your ABI is very much like Linux, your operating system isn't Linux. Porting third party software is near impossible without a cross-compiler. If you set up a real [[OS Specific Toolchain]] and a [[sysroot]] of your OS, you can compile software just by giving --host=i686-myos to ./configure. With a cross-compiler you can [[Cross-Porting Software|port software]] in the standard manner.

== Background information ==

=== Where did the idea of cross compiling come from?===

With GNU software, since most of it has a long history of being ported from one UNIX implementation to another, they came up with a simple set of rules: You have a build machine, a host machine, and a target machine.

=== What are the basics of cross compiling? ===

The "build" machine is the machine you're compiling the software on. This software being compiled may be compiled to run on some other type of machine. See, you may be building on an x86-based machine, and wishing for the software to run on a SPARC based machine. The build machine is implicit and will usually be auto-detected by the configure script for the software. Its only real purpose is so that, if the software being compiled chooses to keep the configure arguments used to configure it somewhere in the built package, the people to whom the package is distributed will know what machine the package was built on. The name of the build machine may be used to configure the package to use workarounds as well if the build machine is universally known to have certain problems building that software.

The "host" machine is the machine on which the software must run. So in the previous example, the "build" machine is an i686-elf-yourBuildOs machine, and the host is a sparc32-elf-unix4 machine.

In the example, you must then, have a sparc32-elf-unix4 cross compiler, which can run on the i686-elf-yourBuildOs machine, and spit out (target) your host machine. A cross compiler is usually named after the host it targets, and not after the host it runs on, so by looking at the name of a compiler, you can usually tell what machine it targets.

The "target" only matters when compiling software that is used to build other software. That is, when compiling compilers, linkers, assemblers and the like. They need to be told what machine they will themselves target. When compiling something like a movie player, or other non-building software, the "target" does not matter. In other words, "target" is used to tell a compiler or other development software being compiled what host it, the compiler being compiled, should target.

For most software, like a text editor, if you're compiling it, you only need to specify host (and usually not even that). Specifying a host causes the software to be built using a compiler on the build machine which is a cross compiler that targets that host. This way, since the software was compiled using that host's targeting cross compiler, the software will be able to run on that host, even though it was built on a (potentially) different built machine.

=== An example to concrete things ===

Your distribution comes equipped with a compiler that targets yourMachineArch-distributionNativeExecFormat-distribution, and that runs on the same yourMachineArch-distributionNativeExecFormat-distribution machine. It was built on some machine (could have been built on m68k-macho-osx or some other colourful machine like that), with a cross compiler that targeted yourMachineArch-distributionNativeExecFormat-distribution, and while it was being built, its own "target" was set to yourMachineArch-distributionNativeExecFormat-distribution, so that it now both runs on, and spits out executables for yourMachineArch-distributionNativeExecFormat-distribution.

A cross compiler is useful for completely unambiguously targeting a specific machine. When you begin developing for a machine separate from your native distribution's stock compiler, you'll need a cross compiler. Actually, you already *do* target a machine that's different from your native machine from the very first time you begin trying to produce completely standalone executables (that is, when you're doing OSDev).

Your native compiler targets yourMachineArch-distributionNativeExecFormat-distribution. But you want to target yourOsTargetArch-yourChosenFormat-none; where "none" as the OS-name represents a machine with no system libraries. When you compile your kernel, you want to build with a compiler that does not know of any system libraries for your target machine. The kernel must be standalone completely. Everything the kernel needs must be in its source tree so that no implicit system libraries are needed (or linked in without your knowledge).

Later on, when you have a userspace and a set of system libraries for your programs to link to, you'll then build a compiler that targets: yourOsTargetArch-yourChosenFormat-yourOs while keeping the compiler that targets yourOsTargetArch-yourChosenFormat-none. The difference between the two is that the "none" OS targeting compiler does not have any libraries in its /lib directory, and has no system libraries to link in. It is therefore impossible for any software you compile using this "bare metal" targeting compiler to have unknown dependencies. The "yourOS" targeting cross compiler is where you would place your own system libraries, (in its /lib directory), such that when it builds software, it will link them against those system libraries, thus linking them against your kernel API. You would continue to use the "bare bones" targeting compiler to build the kernel, and whenever you build programs on your development machine which are expected to run on your kernel's target machine, you would use the yourOsTargetArch-yourChosenFormat-yourOs compiler, which has a /include and /lib directories full of your own OS's native system includes and libraries.

Later still, when you're confident that you can both develop and use your kernel, from within your kernel, you'll want to stop building all your programs on a separate "development" machine. More specifically, at this point, you want to have your "build" machine be the same as your test machine such that you build programs that run on your OS, from within your OS. At this point, it's time to take the final step, and, from your separate "development" machine, build a compiler that will run on your OS (host = yourOsTargetArch-yourChosenFormat-yourOs), and target your OS (target = yourOsTargetArch-yourChosenFormat-yourOs). This cross compiler, the canadian cross, will allow you to natively compile other programs while running on your own OS's userspace. It is essentially a "distribution native compiler", like the one that comes with your distro. From that point on, having a compiler that runs on and targets your OS, you can essentially more freely port programs, and allow people to do the same while running your OS, assuming you package that native compiler with your OS.

=== When do I not need a cross-compiler? ===
If you create a real operating system and manage to port gcc to it, that gcc will produce the very same code as i686-myos-gcc. That means that you don't need a cross-compiler on your own operating system, because the gcc there will already do the right thing. This is why the Linux kernel is built with the Linux gcc, instead of a Linux cross-compiler.

=== A concrete example ===

*gcc -v* from my Ubuntu machine gives:

<pre>
gravaera@gravaera-laptop:/void/cygwin/osdev/zbz$ gcc -v
Using built-in specs.
Target: i486-linux-gnu
Configured with: ../src/configure -v --with-pkgversion='Ubuntu 4.4.3-4ubuntu5' \
--with-bugurl=file:///usr/share/doc/gcc-4.4/README.Bugs --enable-languages=c,c++,fortran,objc,obj-c++ \
--prefix=/usr --enable-shared --enable-multiarch --enable-linker-build-id --with-system-zlib \
--libexecdir=/usr/lib --without-included-gettext --enable-threads=posix --with-gxx-include-dir=/usr/include/c++/4.4 \
--program-suffix=-4.4 --enable-nls --enable-clocale=gnu --enable-libstdcxx-debug --enable-plugin --enable-objc-gc \
--enable-targets=all --disable-werror --with-arch-32=i486 --with-tune=generic --enable-checking=release \
--build=i486-linux-gnu --host=i486-linux-gnu --target=i486-linux-gnu
Thread model: posix
gcc version 4.4.3 (Ubuntu 4.4.3-4ubuntu5)
</pre>

The person who built the compiler that runs on my machine built it on a machine just like mine, an 'i486-linux-gnu' (build), and intended for it to run on a machine like his/mine: the same i486-linux-gnu (host), and he meant for this compiler he was building for me, to emit executables that targeted my very machine, so that when I compile programs, they will be able to run on a host that is i486-linux-gnu. Therefore he made the compiler target i486-linux-gnu.

== See Also ==

=== Articles ===
* [[GCC Cross-Compiler]]
* [[Cross-Porting Software]]

[[Category:Compilers]]
[[Category:FAQ]]
[[Category:OS_Development]]
[[Category:OS_theory]]
````

<a id="w002924"></a>

## W002924 — UEFI

Bron-ID: W002924
Namespace: Main (0)
Revisie: 30920 | Datum: 2026-08-30T00:37:03Z
Bron: https://wiki.osdev.org/UEFI
Revisie-URL: https://wiki.osdev.org/index.php?title=UEFI&oldid=30920

````text
(U)EFI or (Unified) Extensible Firmware Interface is a specification for x86, x86-64, ARM, and Itanium platforms that defines a software interface between the operating system and the platform firmware/BIOS. The original EFI was developed in the mid-1990s by Intel for use developing firmware/BIOS for Itanium platforms. In 2005 Intel transitioned the specification to a new working group called the Unified EFI Forum, consisting of companies such as AMD, Microsoft, Apple, and Intel itself. All modern PCs ship with UEFI firmware and UEFI is widely supported by both commercial and open source operating systems. Backwards compatibility is provided for legacy operating systems.

== UEFI basics ==

===Downloading UEFI images===
If you choose [[VirtualBox]] for virtualization, then UEFI is already included, no need to download the image manually. You just have to enable it in the VM's preferences by clicking "Settings" / "Systems" / "Enable EFI (special OSes only)" checkbox.

Otherwise for emulation and virtual machines, you'll need an OVMF.fd firmware image. It might be tricky to find, so here are some alternative download links too:
* [https://github.com/tianocore/tianocore.github.io/wiki/OVMF TianoCore Download] (official link)
* [https://github.com/BlankOn/ovmf-blobs OVMF-blobs] (unofficial precompiled 32 bit and 64 bit versions, easy to use)
* [https://www.kraxel.org/repos/ RPM packages] (official TianoCore packages)
* [https://packages.debian.org/sid/ovmf Debian packages]
* [https://packages.gentoo.org/useflags/ovmf Gentoo packages]
* [https://archlinux.org/packages/extra/any/edk2/ Arch packages]

Under Linux, you can also install these with your distro's package manager, for example:

'''Debian / Ubuntu'''
<syntaxhighlight lang="bash">
# apt-get install ovmf
</syntaxhighlight>

'''RedHat / CentOS'''
<syntaxhighlight lang="bash">
# yum install ovmf
</syntaxhighlight>

'''MacOS'''

Use the OVMF-blobs repo.

'''Windows'''

Use the OVMF-blobs repo or download the RPM version, then use ''7-Zip File Manager'' to extract the OVMF.fd file from the downloaded archive.

===UEFI vs. legacy BIOS===
A common misconception is that UEFI and BIOS are mutually exclusive. In reality, both legacy motherboards and UEFI-based motherboards both include BIOS ROMs. The differences are in where they find the bootloader/OS, how they prepare the system before executing it, and what convenience functions they provide.

====Platform initialization====
On a legacy system, the [[BIOS]] performs all the usual platform initialization (memory controller configuration, [[PCI]] bus configuration and BAR mapping, graphics card initialization, etc.), but then drops into a backwards-compatible [[Real Mode|real mode]] environment. The bootloader must enable the [[A20 Line|A20 gate]], configure a [[GDT]] and an [[IDT]], switch to [[protected mode]], and for x86-64 CPUs, configure paging and switch to [[x86-64|long mode]].

UEFI firmware performs those same steps, but it also enables the A20 gate and prepares either protected mode environment with flat [[segmentation]] (for 32-bit x86 processors) or a long mode environment with identity-mapped paging (for x86-64 processors).

Additionally, the platform initialization procedure of UEFI firmware is standardized. This allows UEFI firmware to be extended in a vendor-neutral way.

====Boot mechanism====
A legacy BIOS loads a 512 byte flat binary blob from the [[MBR]] of the boot device into memory at physical address 0x7C00 and jumps to it. The bootloader can return back to the BIOS through INT 0x18. On the other hand, UEFI firmware loads an arbitrary sized UEFI application (a relocatable [[PE]] executable file) from a [[FAT]] partition on a [[GPT]] or MBR partitioned boot device to some address selected at run-time. Then it calls that application's main entry point. The application can return control to the firmware, which will continue searching for another boot device or bring up a diagnostic menu.

====System discovery====
A legacy bootloader scans memory for structures like the [[EBDA]], [[SMBIOS]], and [[ACPI]] tables. It uses PIO to talk to the root [[PCI]] controller and scan the PCI bus. It is possible that redundant tables may be present in memory (for example, the [[MP_Specification|MP]] table in the SMBIOS contains information that's also present in the ACPI [[DSDT]] and [[MADT]]) and the bootloader can choose which to use.

When UEFI firmware calls a UEFI application's entry point function, it passes a "System Table" structure, which contains pointers to all of the system's ACPI tables, memory map, and other information relevant to an OS. Legacy tables (like MP tables) may not be present in memory.

====Convenience functions====
A legacy BIOS hooks a variety of interrupts which a bootloader can trigger to access system resources like disks and the screen. These interrupts are not standardized, except by historical convention. Each interrupt uses a different register passing convention.

UEFI firmware establishes many callable functions in memory, which are grouped into sets called "protocols" and are discoverable through the System Table. The behavior of each function in each protocol is defined by specification. UEFI applications can define their own protocols and persist them in memory for other UEFI applications to use. Functions are called with a standardized, modern calling convention supported by many C compilers.

====Development environment====
Legacy bootloaders can be developed in any environment that can generate flat binary images: NASM, GCC, etc. UEFI applications can be developed in any language that can be compiled and linked into a [[PE]] executable file and supports the calling convention used to access functions established in memory by the UEFI firmware. In practice this means one of four development environments: EDK2, GNU-EFI, POSIX-UEFI, or PicoEFI.

[[EDK2]] is a large and complex, yet feature filled environment with its own build system. It can be configured to use GCC, LLVM, MinGW, Microsoft Visual C++, etc. as a cross-compiler. Not only can it be used to compile UEFI applications, but it can also be used to compile UEFI firmware to be flashed to a BIOS ROM.

[[GNU-EFI]] is a set of libraries and headers for compiling UEFI applications with a system's native GCC (does not work with LLVM CLang). It can't be used to compile UEFI firmware. Since it's just a couple of libraries against which a UEFI application can be linked, it is much easier to use than TianoCore ( EDK2 ).

[[POSIX-UEFI]] is very similar to GNU-EFI, but it is distributed mainly as a source, not as a binary library, has ANSI C like names and works with GCC as well as LLVM CLang. It's shipped with a Makefile that sets up the compiler flags for you.

[[PicoEFI]] is a fork of GNU-EFI stripped down to just the relocation stub, linker scripts, and EFI headers, with the convenience library (and ''uefi_call_wrapper'') removed entirely.

====Emulation====

[[OVMF]], a popular open source UEFI firmware, has been ported to the QEMU (but not Bochs) emulated machine. Because it implements the UEFI specification, it behaves very similarly to commercial UEFI firmware on real machines. (OVMF itself is built with TianoCore, and its source ship with it as well, but pre-built images are available.)

===Legacy bootloader or UEFI application?===
If you are targeting legacy systems for which UEFI is not available or is not reliable, you should develop a legacy bootloader. This requires intimate knowledge of 16-bit addressing and the backwards-compatibility features of an x86 or x86-64 CPU.  If you are targeting modern systems you should develop a UEFI application. Many UEFI firmwares can be configured to emulate a legacy BIOS, but there is even more variation among these emulated environments than among real legacy BIOS.

Although there is a slight learning curve to become familiar with the UEFI development environments, using the System Table, and accessing UEFI-provided protocols (functions), there are far fewer "gotchas" than trying to remain compatible with the wide variety of quickly-becoming-obsolete legacy BIOSs on real machines. UEFI is the standard for all modern PCs.

===UEFI class 0-3 and CSM===
PCs are categorized as UEFI class 0, 1, 2, or 3. A class 0 machine is a legacy system with a legacy BIOS; i.e. not a UEFI system at all.

A class 1 machine is a UEFI system that runs exclusively in Compatibility Support Module (CSM) mode. CSM is a specification for how UEFI firmware can emulate a legacy BIOS. UEFI firmware in CSM mode loads legacy bootloaders. A class 1 UEFI system may not advertise UEFI support at all, since it isn't exposed to the bootloader. It's only UEFI "within" the BIOS.

A class 2 machine is a UEFI system that can launch UEFI applications but also includes the option to run in CSM mode. The majority of modern PCs are UEFI class 2 machines. Sometimes the choice to run UEFI applications vs. CSM is a one-or-the-other setting in the BIOS configuration, and other times the BIOS will decide which to use after selecting the boot device and checking whether it has a legacy bootloader or a UEFI application.

A class 3 machine is a UEFI system that does not support CSM. UEFI class 3 machines only run UEFI applications and do not implement CSM for backwards compatibility with legacy bootloaders.

===Secure Boot===
Secure Boot is a digital signature scheme for UEFI applications that consists of four components:
* '''PK''': Platform Key
* '''KEK''': Key Exchange Keys
* '''db''': Whitelist database
* '''dbx''': Blacklist database

UEFI firmware that supports Secure Boot is always in one of three states:
* Setup mode, Secure Boot ''off''
* User mode, Secure Boot ''off''
* User mode, Secure Boot ''on''

In setup mode, any UEFI application can change or delete the PK, add/remove keys from the KEK, and add/remove whitelist or blacklist entries from the db or dbx.

In user mode, regardless of whether Secure Boot is on or off:
* The PK may only be changed or deleted by a UEFI application that already has the current PK.
* Keys can only be added/removed from the KEK by a UEFI application that has the PK.
* Whitelist and blacklist entries can only be added/removed from the db and dbx by a UEFI application that has any one of the keys in the KEK.

Finally, in user mode with Secure Boot ''on'', UEFI applications must meet one of the following four requirements to be launched:
* Signed, with signature in db and not in dbx
* Signed by a key in db and not in dbx
* Signed by a key in the KEK
* Unsigned, but a hash of the application is in db and not in dbx

Note that UEFI applications are not signed by the PK, unless the PK also happens to be in the KEK.

Not all UEFI firmwares support Secure Boot, although it is a requirement for Windows 8. Some UEFI firmwares support Secure Boot and there's no option to be disabled, which poses a problem for independent developers that do not have access to the PK or any of the keys in the KEK, and therefore can't install their own key or application signature or hash to the whitelist database. Independent developers should develop on systems that either do not support Secure Boot or has an option for Secure Boot to be turned off.

An easy way out to use a loader that is signed by Microsoft, and allows you to load another binary signed by a key and certificate owned by you (called MOK, Machine Owner's Key). Such a loader is [https://github.com/rhboot/shim shim], used by RedHat, Fedora, Suse, Ubuntu, Arch and many other distros to load GRUB. The filename of the EFI executable is hardwired in shim, but if you rename your loader to GRUBX64.EFI (or GRUBIA32.EFI), you sign it with your MOK key and certificate using [https://github.com/imedias/sbsigntool sbsigntool], then you can load any loader in Secure Boot you want.

===How to use UEFI===
Traditional operating systems like Windows and Linux have an existing software architecture and a large code base to perform system configuration and device discovery. With their sophisticated layers of abstraction they don't directly benefit from UEFI. As a result, their UEFI bootloaders do little but prepare the environment for them to run.

An independent developer may find more value in using UEFI to write feature-full UEFI applications, rather than viewing UEFI as a temporary start-up environment to be jettisoned during the boot process. Unlike legacy bootloaders, which typically interact with BIOS only enough to bring up the OS, a UEFI application can implement sophisticated behavior with the help of UEFI. In other words, an independent developer shouldn't be in a rush to leave "UEFI-land".

A good starting point is writing a UEFI application that uses the System Table to fetch a memory map, and uses the "File" protocol to read files from FAT-formatted disks. The next step might be to use the System Table to locate ACPI tables.

==Developing with POSIX-UEFI==
:''Main article: [[POSIX-UEFI]]

One option to compile UEFI applications on POSIX like systems is POSIX-UEFI. It provides a [[libc]]-like API for your EFI application, and ships with a Makefile that can detect and set up the toolchain for you. It can use GCC or LLVM, and defaults to using the host compiler, but a cross compiler is still recommended.

It uses POSIX style typedefs (like ''uintn_t'' instead of ''UINTN''), and it does not ship with the standard EFI headers. You can still get interfaces not covered by POSIX-UEFI (such as GOP) by installing the EFI headers from GNU-EFI or EDK2. Also, it compiles with the MS ABI, meaning that UEFI services can be called natively (i.e., without uefi_call_wrapper) so long as your apps are compiled with it as well.
The traditional "Hello, world" UEFI program goes like this.
<syntaxhighlight lang="c">
#include <uefi.h>
 
int main (int argc, char **argv)
{
  printf("Hello, world!\n");
  return 0;
}
</syntaxhighlight>
Makefile looks like this:
<syntaxhighlight lang="make">
TARGET = main.efi
include uefi/Makefile
</syntaxhighlight>
Run make to build it. The result of this process is a PE executable file ''main.efi''.

==Developing with GNU-EFI==
:''Main article: [[GNU-EFI]]

GNU-EFI can be used to develop both 32-bit and 64-bit UEFI applications. This section will address 64-bit UEFI applications only, and assumes that the development environment itself is running on an x86_64 system, so that no cross-compiler is needed. For a more thorough walk-through of a proper (non-gnu-efi) development environment, see [[UEFI App Bare Bones]].
        
GNU-EFI includes four things:
* '''crt0-efi-x86_64.o''': A CRT0 (C runtime initialization code) that provides an entry point that UEFI firmware will call when launching the application, which will in turn call the "efi_main" function that the developer writes.
* '''libgnuefi.a''': A library containing a single function (''_relocate'') that is used by the CRT0.
* '''elf_x86_64_efi.lds''': A linker script used to link ELF binaries into UEFI applications.
* '''efi.h''' and other headers: Convenience headers that provide structures, typedefs, and constants improve readability when accessing the System Table and other UEFI resources.
* '''libefi.a''': A library containing convenience functions like CRC computation, string length calculation, and easy text printing.
* '''efilib.h''': Header for ''libefi.a''.

At a bare minimum, a 64-bit UEFI application will need to link against ''crt0-efi-x86_64.o'' and ''libgnuefi.a'' using the ''elf_x86_64_efi.lds'' linker script. Most likely you will want to use the provided headers and the convenience library as well, and this section will assume that going forward.

The traditional "Hello, world" UEFI program is shown below.
<syntaxhighlight lang="c">
#include <efi.h>
#include <efilib.h>

EFI_STATUS
EFIAPI
efi_main (EFI_HANDLE ImageHandle, EFI_SYSTEM_TABLE *SystemTable)
{
  InitializeLib(ImageHandle, SystemTable);
  Print(L"Hello, world!\n");
  return EFI_SUCCESS;
}
</syntaxhighlight>

A few notes:
* efi.h is included so we can use types like EFI_STATUS, EFI_HANDLE, and EFI_SYSTEM_TABLE.
* When creating a 32-bit UEFI application, EFIAPI is empty; GCC will compile the "efi_main" function using the standard C calling convention. When creating a 64-bit UEFI application, EFIAPI expands to "__attribute__((ms_abi))" and GCC will compile the "efi_main" function using Microsoft's x64 calling convention, as specified by UEFI. Only functions that will be called directly from UEFI (including main, but also callbacks) need to use the UEFI calling convention.
* "InitializeLib" and "Print" are convenience functions provided by libefi.a with prototypes in efilib.h. "InitializeLib" lets libefi.a store a reference to the ImageHandle and SystemTable provided by BIOS. "Print" uses those stored references to print a string by reaching out to UEFI-provided functions in memory. (Later on we will see how to find and call UEFI-provided functions manually.)

This program is compiled and linked as below.
<syntaxhighlight lang="bash">
$ gcc main.c                             \
      -c                                 \
      -fno-stack-protector               \
      -fpic                              \
      -fshort-wchar                      \
      -mno-red-zone                      \
      -I /path/to/gnu-efi/headers        \
      -I /path/to/gnu-efi/headers/x86_64 \
      -DEFI_FUNCTION_WRAPPER             \
      -o main.o

$ ld main.o                         \
     /path/to/crt0-efi-x86_64.o     \
     -nostdlib                      \
     -znocombreloc                  \
     -T /path/to/elf_x86_64_efi.lds \
     -shared                        \
     -Bsymbolic                     \
     -L /path/to/libs               \
     -l:libgnuefi.a                 \
     -l:libefi.a                    \
     -o main.so

$ objcopy -j .text                       \
          -j .sdata                      \
          -j .data                       \
          -j .rodata                     \
          -j .dynamic                    \
          -j .dynsym                     \
          -j .rel                        \
          -j .rela                       \
          -j .reloc                      \
          --output-target=efi-app-x86_64 \
          main.so                        \
          main.efi
</syntaxhighlight>

The result of this process is a 44 kB PE executable file ''main.efi''. On a real project you'll probably want to use make or another build tool, and may need to build a cross-compiler. The way GNU-EFI works is a bit contrived: you are wrapping an ELF file built by your normal compiler into PE.

==Developing with PicoEFI==
:''Main article: [[PicoEFI]]

[[PicoEFI]] is a stripped-down fork of GNU-EFI that keeps only the CRT0 relocation stub, linker scripts, and EFI headers for each supported architecture (ia32, x86_64, aarch64, riscv64, loongarch64), and drops GNU-EFI's convenience library and ''uefi_call_wrapper'' entirely. It is meant to be built directly into your own project rather than linked as a pre-built library.

Since there is no convenience library, normal operations means accessing the system table, for example, printing:
<syntaxhighlight lang="c">
#include <efi.h>

EFI_STATUS EFIAPI
efi_main(EFI_HANDLE ImageHandle, EFI_SYSTEM_TABLE *SystemTable)
{
  SystemTable->ConOut->OutputString(SystemTable->ConOut, L"Hello, world!\r\n");
  return EFI_SUCCESS;
}
</syntaxhighlight>

==Emulation with QEMU and OVMF==
Any recent version of QEMU with a recent version of OVMF will be sufficient to run a UEFI application. QEMU binaries are available for many platforms, and a binary OVMF image (OVMF.fd) can be found on the [http://www.tianocore.org/ovmf/ TianoCore] website. QEMU (without any boot disk) can be invoked as below. (To prevent recent versions of QEMU from attempting a PXE (network) boot when no boot disk is found, use <code>-net none</code>.

The recommended way to use OVMF (for QEMU 1.6 or newer) is with a <code>pflash</code> parameter. The instructions below assume you have an OVMF image split into separate CODE and VARS sections.

<syntaxhighlight lang="bash">
$ qemu-system-x86_64 -cpu qemu64 \
  -drive if=pflash,format=raw,unit=0,file=path_to_OVMF_CODE.fd,readonly=on \
  -drive if=pflash,format=raw,unit=1,file=path_to_OVMF_VARS.fd \
  -net none
</syntaxhighlight>

If you prefer to work on a terminal without a display, or via SSH/telnet, you will want to run QEMU without graphics support, using the <code>-nographic</code> flag.

If OVMF does not find a boot disk with a properly named UEFI application (more on this later) it will drop into a UEFI shell.

[[file:OVMF_shell.png]]

You can find a list of shell commands [https://web.archive.org/web/20170504062623/http://www.sysadminshare.com/2012/01/efi-shell-commands.html here] or you can type ''help'' at the shell.

===Creating disk images===
:''Main article: [[Bootable Disk]]

To launch a UEFI application you will need to create a disk image and present it to QEMU. UEFI firmware expects UEFI applications to be stored in a FAT12, FAT16, or FAT32 file system on a [[GPT]] or [[MBR]]-partitioned disk. Many firmwares only support FAT32, so that's what you'll want to use. Depending on your platform, there are several different ways to create a disk image containing your UEFI application, but they all start by creating a zeroed disk image file. The minimum FAT32 partition size is 33,548,800 bytes, plus you will need space for the primary and secondary GPT tables, plus some slack space so the partition can be aligned correctly. Throughout these examples we will be creating a 48,000,000 byte (93750 512-byte sectors, or 48 MB) disk image.
<syntaxhighlight lang="bash">
$ dd if=/dev/zero of=/path/to/uefi.img bs=512 count=93750
</syntaxhighlight>

====Virtual FAT disk images====

QEMU can automatically create a virtual FAT disk image from a specified directory tree. To use this feature, simply create a directory and copy any UEFI applications you want to test into it.

<syntaxhighlight lang="bash">
$ mkdir esp
$ cp /path/to/main.efi esp
</syntaxhighlight>

After that, you can invoke QEMU as below.

<syntaxhighlight lang="bash">
$ qemu-system-x86_64 -cpu qemu64 \
  -drive if=pflash,format=raw,unit=0,file=path_to_OVMF_CODE.fd,readonly=on \
  -drive if=pflash,format=raw,unit=1,file=path_to_OVMF_VARS.fd \
  -drive format=raw,file=fat:rw:esp \
  -net none
</syntaxhighlight>

====uefi-run helper application====
The uefi-run application is useful for quick testing. It creates a temporary FAT image containing your EFI application and starts qemu.

<syntaxhighlight lang="bash">
$ uefi-run -b /path/to/OVMF.fd -q /path/to/qemu app.efi -- <extra_qemu_args>
</syntaxhighlight>

uefi-run is not currently packaged for any distribution. You can install it using cargo (the Rust package manager) though ("cargo install uefi-run").

====Linux, root required====
This approach requires root privileges and uses '''gdisk''', '''losetup''', and '''mkdosfs'''. First, use gdisk to create a GPT partition table with a single [[EFI System Partition]].
<syntaxhighlight lang="bash">
$ gdisk /path/to/uefi.img
GPT fdisk (gdisk) version 0.8.10

Partition table scan:
  MBR: not present
  BSD: not present
  APM: not present
  GPT: not present

Creating new GPT entries.

Command (? for help): o
This option deletes all partitions and creates a new protective MBR.
Proceed? (Y/N): y

Command (? for help): n
Partition number (1-128, default 1): 1
First sector (34-93716, default = 2048) or {+-}size{KMGTP}: 2048
Last sector (2048-93716, default = 93716) or {+-}size{KMGTP}: 93716
Current type is 'Linux filesystem'
Hex code or GUID (L to show codes, Enter = 8300): ef00
Changed type of partition to 'EFI System'

Command (? for help): w

Final checks complete. About to write GPT data. THIS WILL OVERWRITE EXISTING
PARTITIONS!!

Do you want to proceed? (Y/N): y
OK; writing new GUID partition table (GPT) to uefi.img.
Warning: The kernel is still using the old partition table.
The new table will be used at the next reboot.
The operation has completed successfully.
</syntaxhighlight>

Now you have disk image with [[GPT|GUID partition table]] on it and an unformatted EFI partition starting at sector 2048. Unless you deviated from the commands shown above, the disk image will use 512-byte sectors, so the EFI partition starts at byte 1,048,576 and is 46,934,528 bytes in length. Use losetup to present the partition to Linux on a loopback device.

<syntaxhighlight lang="bash">
losetup --offset 1048576 --sizelimit 46934528 /dev/loop0 /path/to/uefi.img
</syntaxhighlight>

(If /dev/loop0 is already in use you will need to select a different loopback device.)

Format the partition for FAT32 with mkdosfs.
<syntaxhighlight lang="bash">
mkdosfs -F 32 /dev/loop0
</syntaxhighlight>

The partition can now be mounted, so that we can copy files to it. In this example we use the "/mnt" directory, but you could also create a local directory for temporary use.
<syntaxhighlight lang="bash">
mount /dev/loop0 /mnt
</syntaxhighlight>

Copy any UEFI applications you want to test to the file system.
<syntaxhighlight lang="bash">
$ cp /path/to/main.efi /mnt/
$ ...
</syntaxhighlight>

Finally, unmount the partition and free the loopback device.
<syntaxhighlight lang="bash">
$ umount /mnt
$ losetup -d /dev/loop0
</syntaxhighlight>

''uefi.img'' is now a disk image containing primary and secondary GPT tables, containing a single partition of type EFI, containing a FAT32 file system, containing one or more UEFI applications.

====Linux, root not required====
This approach uses '''parted''', '''mformat''', and '''mcopy''' and can be performed with user privileges. First, use parted to create primary and secondary GPT headers, and a single EFI partition spanning the same range as the approach above.
<syntaxhighlight lang="bash">
$ parted /path/to/uefi.img -s -a minimal mklabel gpt
$ parted /path/to/uefi.img -s -a minimal mkpart EFI FAT16 2048s 93716s
$ parted /path/to/uefi.img -s -a minimal toggle 1 boot
</syntaxhighlight>

Now create a new temporary image file that will contain the EFI partition data and use mformat to format it with FAT16.
<syntaxhighlight lang="bash">
dd if=/dev/zero of=/tmp/part.img bs=512 count=91669
mformat -i /tmp/part.img -h 32 -t 32 -n 64 -c 1
</syntaxhighlight>

Use mcopy to copy any UEFI applications you want to test to the file system.
<syntaxhighlight lang="bash">
$ mcopy -i /tmp/part.img /path/to/main.efi ::
$ ...
</syntaxhighlight>

Finally, write the partition image into the main disk image.
<syntaxhighlight lang="bash">
$ dd if=/tmp/part.img of=/path/to/uefi.img bs=512 count=91669 seek=2048 conv=notrunc
</syntaxhighlight>

''uefi.img'' is now a disk image containing primary and secondary GPT tables, containing a single partition of type EFI, containing a FAT16 file system, containing one or more UEFI applications.

====FreeBSD, root required====
This approach requires root privileges and uses '''mdconfig''', '''gpart''', '''newfs_msdos''', and '''mount_msdosfs'''. First, create a device node that presents the zeroed disk image as a block device. This will let us work on it using standard partitioning and formatting tools.
<syntaxhighlight lang="bash">
$ mdconfig -f /path/to/uefi.img
md0
</syntaxhighlight>

In this example the new block device is ''md0''. Now create the empty primary and secondary GPT tables on the device.
<syntaxhighlight lang="bash">
$ gpart create -s GPT md0
md0 created
</syntaxhighlight>

Now we can add a partition to the disk. We'll specify an "EFI" partition, which just means that GPT will set that partition's GUID to the special "EFI" type. Not all BIOSs require this, and the partition will still be able to be mounted and browsed normally on Linux, FreeBSD, and Windows.
<syntaxhighlight lang="bash">
$ gpart add -t efi md0
md0p1 added
</syntaxhighlight>

Next, create a FAT16 file system on the new partition. You can specify various parameters for the file system if you'd like, but it isn't necessary. Ideally you would create a FAT32 partition for best firmware compatibility but FreeBSD seems to create FAT32 partitions that OVMF can't read.
<syntaxhighlight lang="bash">
$ newfs_msdos -F 16 md0p1
newfs_msdos: trim 2 sectors to adjust to a multiple of 9
/dev/md2p1: 93552 sectors in 11694 FAT16 clusters (4096 bytes/cluster)
BytesPerSec=512 SecPerClust=8 ResSectors=1 FATs=2 RootDirEnts=512 Media=0xf0 FATsecs=46 SecPerTrack=9 Heads=16 HiddenSecs=0 HugeSectors=93681
</syntaxhighlight>

The partition can now be mounted, so that we can copy files to it. In this example we use the ''/mnt'' directory, but you could also create a local directory for temporary use.
<syntaxhighlight lang="bash">
$ mount_msdosfs /dev/md0p1 /mnt
</syntaxhighlight>

Copy any UEFI applications you want to test to the file system.
<syntaxhighlight lang="bash">
$ cp /path/to/main.efi /mnt/
$ ...
</syntaxhighlight>

Finally, unmount the partition and free the block device.
<syntaxhighlight lang="bash">
$ umount /mnt
$ mdconfig -d -u md0
</syntaxhighlight>

''uefi.img'' is now a disk image containing primary and secondary GPT tables, containing a single partition of type EFI, containing a FAT16 file system, containing one or more UEFI applications.

====Mac OS (root not required)====
Mac OS has a single tool (hdiutil) that creates the disk image and copy files at same time.

Let's say that you're creating a UEFI boot for x86_64. By definition the file name should be BOOTX64.EFI and this file should be located in the /EFI/BOOT folder.

First, let's create a temporary folder that will contains all files and folders required for booting UEFI.
  
<syntaxhighlight lang="bash">
$ mkdir -p diskImage/EFI/BOOT
</syntaxhighlight>

Secondly, let's copy the boot application to the required location:

<syntaxhighlight lang="bash">
$ cp bootx64.efi diskImage/EFI/BOOT/BOOTX64.EFI
</syntaxhighlight>

Finally, let's create a disk image partitioned with GPT, formatted with fat32 (-fs fat32), overriding destination file if needed (-ov), define disk size (-size 48m), define volume name (-volname NEWOS), the file format which the disk will be encoded (-format UDTO - the same used for DVDs/CDs) and the source folder containing the files that will be copied to the new disk:

<syntaxhighlight lang="bash">
$ hdiutil create -fs fat32 -ov -size 48m -volname NEWOS -format UDTO -srcfolder diskImage uefi.cdr
</syntaxhighlight>

uefi.cdr should be ready to be used by QEMU.

===Launching UEFI applications===
Once your disk image is ready, you can invoke QEMU as below.
<syntaxhighlight lang="bash">
$ qemu-system-x86_64 -cpu qemu64 -bios /path/to/OVMF.fd -drive file=uefi.disk,if=ide
</syntaxhighlight>

When OVMF drops into the UEFI shell, you will see an additional entry in the "Mapping table", labeled "FS0". This indicates that the firmware detected the disk, discovered the partition, and was able to mount the file system. You can explore the file system by switching to it using the DOS-style syntax "FS0:", as illustrated below.

[[File:OVMF_browse_fs.png]]

You can launch a UEFI application by entering its name.

[[File:OVMF_run_app.png]]

Notice that the UEFI shell resumed once the application terminated. Of course if this was a proper bootloader it would never resume, but rather launch an OS.

Some commercial UEFI firmware provides UEFI shells or the capability to launch user-selected UEFI applications, such as the firmware that ships with the HP EliteBook line of laptops. Most, however, do not expose this functionality to the end-user.

===Debugging===
OVMF can be built in debug mode, and it will output logging messages to IO port 0x402.
You can use some flags like the ones below to capture the output.

<code>-debugcon file:uefi_debug.log -global isa-debugcon.iobase=0x402</code>

Note that release builds will not output debug messages, or will have reduced output.

See [[Debugging UEFI applications with GDB]].

==Running on real hardware==
===NVRAM variables===
{{Main|UEFI NVRAM}}
A UEFI firmware will present most of its configuration options through a text or graphical configuration menu, just like a legacy BIOS. Selections made in these menus are stored in the NVRAM chip between reboots. Unlike legacy BIOS, however, the firmware developer has the option to expose some or all of these "NVRAM variables" to the OS and end-user via convenience functions made resident in RAM by the firmware at boot.

The Linux '''efivarfs''' kernel module will use these functions to list NVRAM variables in the ''/sys/firmware/efi/efivars'' file. NVRAM variables can also be dumped from within the UEFI shell itself with the '''dmpstore''' command. Device boot order is always accessible via NVRAM variables. The Linux program '''efibootmgr''' works specifically with the boot order NVRAM variables. The UEFI shell offers the '''bcfg''' command for the same purpose.

===Bootable UEFI applications===
The boot order NVRAM variables determine where firmware will look for UEFI applications to be launched at boot. Although this can be changed (for example, an OS installer might customize the boot entry for the hard drive to which it was installed) firmware typically looks for a UEFI application named "BOOTIA32.efi" (for 32-bit applications) or "BOOTX64.efi" (for 64-bit applications) stored in the "/EFI/BOOT" path in the boot device's file system. This is the default path and name for OVMF.

Unlike a UEFI application launched from the shell, if a bootable UEFI application returns BIOS will continue searching for other boot devices.

===Exposed functionality===
Real PCs vary in the amount of UEFI capability they expose to the user. For example, even a class 3 machine may not make any mention of UEFI in its BIOS configuration and may not offer a UEFI shell. Additionally, some BIOS vendors make their UEFI firmware configuration screens look identical to their legacy BIOS configuration screens. Class 2 machines may present somewhat confusing boot menus and configuration options. For example, one laptop manufacturer includes a configuration option to enable/disable UEFI (i.e. switch between UEFI and CSM behavior), named "OS: Windows 8". Another laptop, if it fails to find a bootable UEFI application on the selected boot device (or if that application returns a status other than EFI_SUCCESS) will fall back to CSM behavior and then complain that the drive has a corrupted MBR. With time, and with the emergence of class 3 machines, clarity of UEFI boot behavior will improve.

To make testing on real hardware easier, you can install a bootable UEFI application to the internal hard drive of the system that provides a boot menu, such as [http://www.rodsbooks.com/refind/ rEFInd]. This may also be convenient for multi-boot scenarios.

===PC firmware developers===
On x86 and x86-64 platforms, the following BIOS developers offer UEFI firmware:
* AMI (Aptio).
* Phoenix (SecureCore, TrustedCore, AwardCore).
* Insyde (InsydeH20).

== UEFI applications in detail ==
=== Binary Format ===
UEFI executables are regular PE32 / PE32+ (Windows x32 / x64) images, with a specific '''subsystem'''.
Every UEFI application is basically a windows EXE (or DLL) without symbol tables.

{| class="wikitable"
|+ Types of UEFI images
|-
! Type
! Description
! Subsystem
|-
| Applications || OS loaders and other utility programs. || 10
|-
| Boot service driver || Drivers used by the firmware when booting (e.g. disk drivers, network drivers). || 11
|-
| Runtime driver || Drivers which may stay loaded even after the OS loads and exits the boot services. || 12
|}

UEFI images must also specify the type of machine code they contain. A UEFI loader will refuse to boot an incompatible image.

{| class="wikitable"
|+ Types of machines
|-
! Name / arch
! Value
|-
|x86||0x014c
|-
|x86_64||0x8664
|-
|Itanium x64||0x0200
|-
|UEFI Byte Code||0x0EBC
|-
|ARM<sup>1</sup>||0x01C2
|-
|AArch (ARM x64)||0xAA64
|-
|RISC-V x32||0x5032
|-
|RISC-V x64||0x5064
|-
|RISC-V x128||0x5128
|}

[1] ARM means you can use Thumb / Thumb 2 instructions, but UEFI interfaces are in ARM mode.

==== Initialization ====
Applications must either load an OS and exit boot services, or return from the main function (in which case the boot loader will look for the next app to load).

Drivers must initialize and then return 0 on success, or an error code. A computer might fail to boot if a required driver fails to load.

==== Memory ====
The memory map returned by UEFI will mark the memory areas which drivers use.

Once your OS loader finished, your kernel is allowed to reuse the memory where the boot loader was loaded.

The memory types are <code>Efi{Loader/BootServices/RuntimeServices}{Code/Data}</code>.

After exiting the boot services, you may reuse whatever non-read-only memory the boot drivers used.

However, memory used by the runtime drivers must '''never''' be touched - the runtime drivers stay active and loaded for as long as the computer runs.

One way to see a breakdown of a PE file containing a UEFI application is by<syntaxhighlight lang="bash">$ objdump --all-headers /path/to/main.efi</syntaxhighlight>
Its output is quite long. Among other things, it shows the '''subsystem''', that is the type of the UEFI image, mentioned earlier.

=== Calling Conventions ===
UEFI specifies the following calling conventions:
* '''cdecl''' for x86 UEFI functions
* '''Microsoft's 64-bit calling convention''' for x86-64 UEFI functions
* '''SMC''' for ARM UEFI functions

This has two impacts on UEFI application developers:
* A UEFI application's main entry point must expect to be called with the corresponding calling convention.
* Any UEFI-provided functions that a UEFI application calls must be called with the corresponding calling convention.
Note that functions strictly internal to the application can use whatever calling convention the developer chooses.

==== POSIX-UEFI, GNU-EFI and GCC ====
{{Main|GNU-EFI}}
cdecl is the standard calling convention used by GCC, so no special attributes or modifiers are needed for writing the main entry point or calling UEFI functions in an x86 UEFI application developed with GNU-EFI. For x86-64, however, the entry point function must be declared with the "___attribute___((ms_abi))" modifier and all calls to UEFI-provided functions must be made through the "uefi_call_wrapper" thunk. This thunk is called with cdecl, but then translates to the Microsoft x86-64 calling convention before calling the requested UEFI function. This is necessary because older releases of GCC do not support specifying calling conventions for function pointers.

For [[POSIX-UEFI]], which also uses GCC, your entry point looks like the standard main(), and no special ABI is required. Also the build environment takes care of the compiler flags for you, so you can simply call UEFI functions without "uefi_call_wrapper", no matter if you're using the host gcc or a cross-compiler.

For developer convenience, both POSIX-UEFI and GNU-EFI provides the "EFIAPI" macro, which expands to "cdecl" when targeting x86 and "__attribute__(ms_abi))" when targeting x86-64. Additionally, the "uefi_call_wrapper" thunk will simply pass the call through on x86. This allows the same source code to target x86 and x86-64. For example, the following main function will compile with the correct calling convention on both x86 and x86-64 and the call through the "uefi_call_wrapper" thunk will select the correct calling convention to use when calling the UEFI function (in this case, printing a string).
<syntaxhighlight lang="c">
EFI_STATUS EFIAPI efi_main(EFI_HANDLE ImageHandle, EFI_SYSTEM_TABLE *SystemTable)
{
  EFI_STATUS status = uefi_call_wrapper(SystemTable->ConOut->OutputString,
                                        2,
                                        SystemTable->ConOut,
                                        L"Hello, World!\n");
  return status;
}
</syntaxhighlight>

=== Language binding ===
UEFI applications are typically written in C, although bindings could be written for any other language that compiles to machine code. Assembly is also an option; a [[uefi.inc]] is available for [[FASM]] that allows UEFI applications to be written as below.
<syntaxhighlight lang="asm">
format pe64 dll efi
entry main

section '.text' code executable readable

include 'uefi.inc'

main:
    ; initialize UEFI library
    InitializeLib
    jc @f

    ; call uefi function to print to screen
    uefi_call_wrapper ConOut, OutputString, ConOut, _hello

@@: mov eax, EFI_SUCCESS
    retn

section '.data' data readable writeable

_hello                                  du 'Hello World',13,10,0

section '.reloc' fixups data discardable
</syntaxhighlight>

As a UEFI application contains normal x86 or x86-64 machine code, inline assembly is also an option in compilers that support it.

=== EFI Byte Code===
UEFI also includes a virtual machine specification based on a byte code format called EFI Byte Code (EBC), which can be used to write platform-independent device drivers, but not UEFI applications. As of 2015 there has been limited use of EBC.

== Common Problems ==

=== My UEFI application hangs/resets after about 5 minutes ===

When control is handed to your UEFI application by firmware, it sets a watchdog timer for 5 minutes, after which the firmware is reinvoked as it assumes your application has hung.  The firmware in this case will normally try to reset the system (although the OVMF firmware in VirtualBox simply causes the screen to go black and hang).  To counteract this, you are required to refresh the watchdog timer before it times out.  Alternatively, you can disable it completely with code like <syntaxhighlight lang="C">SystemTable->BootServices->SetWatchdogTimer(0, 0, 0, NULL);</syntaxhighlight>Obviously this is not a problem for most bootloaders, but can cause an issue if you have an interactive loader which waits for user input.  Also note that you are required to disable the watchdog timer if you exit back to the firmware.

=== My bootloader hangs if I use user defined EFI_MEMORY_TYPE values ===

For the memory management functions in EFI, an OS is meant to be able to use "memory type" values above 0x80000000 for its own purposes. In the OVMF EFI firmware release "r11337" (for Qemu, etc) there is a bug where the firmware assumes the memory type is within the range of values defined for EFI's own use, and uses the memory type as an array index. The end result is an "array index out of bounds" bug; where the higher memory type values (e.g. perfectly legal values above 0x80000000) cause the 64-bit version of the firmware to crash (page fault), and cause incorrect "attribute" values to be reported by the 32-bit version of the firmware. This same bug is also present in whatever version of the EFI firmware VirtualBox uses (which looks like an older version of OVMF); and I suspect (but don't know) that the bug may be present in a wide variety of firmware that was derived from the tianocore project (not just OVMF).

== See also ==
=== Articles ===
* [[UEFI Bare Bones]]
* [[EFI System Partition]]
* [[PE]] file format
* [[EDK2]]
* [[POSIX-UEFI]]
* [[GNU-EFI]]
* [[PicoEFI]]
* [https://github.com/nebulaeonline/nebulae/tree/UefiBarebones Uefi Barebones MSVC/Clang/Visual Studio]

=== External Links ===
* [https://uefi.org/specifications UEFI specifications et al.]
* [http://www.tianocore.org/ Intel TianoCore EDK2]
* [https://github.com/tianocore/tianocore.github.io/wiki/OVMF OVMF firmware images] for use with [[QEMU]]
* [https://web.archive.org/web/20181028201454/http://wiki.phoenix.com/wiki/index.php/Main_Page Phoenix UEFI Wiki]
* [http://x86asm.net/articles/others/index.html Several articles about UEFI]
* [https://www.microsoft.com/en-us/download/details.aspx?id=19509 PE specification covering the (U)EFI binary format]
* [https://uefi.blogspot.com/ Blog about UEFI, with bits about UEFI development]
* [https://web.archive.org/web/20160316192235/http://internshipatdell.wikispaces.com/file/view/How+to+build+an+UEFI+application.pptx Presentation guiding through simple UEFI application setup]
* [https://uefi.org/sites/default/files/resources/UEFI-Plugfest-WindowsBootEnvironment.pdf Presentation giving an overview of windows uefi booting]
* [https://gitlab.com/bztsrc/posix-uefi POSIX-UEFI] documentation and source
* [[Wikipedia:Extensible_Firmware_Interface|Wikipedia Article on EFI]]

[[Category:x86]]
[[Category:x86-64]]
[[Category:ARM]]
[[Category:Firmware]]
[[Category:UEFI]]
[[de:Unified_Extensible_Firmware_Interface]]
````

<a id="w002925"></a>

## W002925 — EFI

Bron-ID: W002925
Namespace: Main (0)
Revisie: 10840 | Datum: 2010-10-11T18:55:22Z
Bron: https://wiki.osdev.org/EFI
Revisie-URL: https://wiki.osdev.org/index.php?title=EFI&oldid=10840
Doorverwijzing: UEFI

````text
#REDIRECT [[UEFI]]
````

<a id="w002926"></a>

## W002926 — Extensible Firmware Interface

Bron-ID: W002926
Namespace: Main (0)
Revisie: 10842 | Datum: 2010-10-11T19:00:44Z
Bron: https://wiki.osdev.org/Extensible_Firmware_Interface
Revisie-URL: https://wiki.osdev.org/index.php?title=Extensible_Firmware_Interface&oldid=10842
Doorverwijzing: UEFI

````text
#REDIRECT [[UEFI]]
````

<a id="w002927"></a>

## W002927 — Unified Extensible Firmware Interface

Bron-ID: W002927
Namespace: Main (0)
Revisie: 10843 | Datum: 2010-10-11T19:00:59Z
Bron: https://wiki.osdev.org/Unified_Extensible_Firmware_Interface
Revisie-URL: https://wiki.osdev.org/index.php?title=Unified_Extensible_Firmware_Interface&oldid=10843
Doorverwijzing: UEFI

````text
#REDIRECT [[UEFI]]
````

<a id="w002928"></a>

## W002928 — Category:Firmware

Bron-ID: W002928
Namespace: Category (14)
Revisie: 12565 | Datum: 2012-02-06T18:17:29Z
Bron: https://wiki.osdev.org/Category:Firmware
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AFirmware&oldid=12565

````text
The firmware category page contains a list of firmware used on various platforms.

[[Category:Hardware]]
<!--I placed it here as it is the best fit-->
````

<a id="w002929"></a>

## W002929 — Julian Day Number

Bron-ID: W002929
Namespace: Main (0)
Revisie: 24946 | Datum: 2020-06-26T14:01:16Z
Bron: https://wiki.osdev.org/Julian_Day_Number
Revisie-URL: https://wiki.osdev.org/index.php?title=Julian_Day_Number&oldid=24946

````text
Any OS needs a format for "time". There would seem to be no real standard for when an OS should start measuring the "beginning of time". But there is in fact one that approaches being a standard.

== Basic Definition ==

There is a measurement of time that is used in Astronomy, called Julian Day Number (JDN). For reasons regarding historical calendar systems,
it starts measuring time from noon, UTC, January 1, 4713 BC, and it measures time in days with a floating point value. Because it is a sort-of standard, there is code available for converting
JDN's to calendar dates.

In any case, once you have divided out the seconds and fractions of seconds from any custom time format, you are left with "days".
Which means that the difference between your number and the Julian Day Number is merely an offset. Of course, it is possible to truncate JDNs, and
use them as integers.

== Date Calculations ==

The JDN for Y2K was 2451545.
If you pretend there was a "Year 0" (or alternately, 1 BC), then Jan 1 of that year was JDN 1721060.

To calculate (standard) Gregorian dates: 400 years have exactly 146097 days.

To calculate leap years: if a year is a multiple of 4 then it's a leap year, unless it happens to be a new century and it can't be divided by 400.

== Code Examples ==

<source lang="c">
uint32_t days2years[401];

#define LEAP_FLAG		0x80000000

// the days2years table covers 400 years and contains:
// 1) the day number at the START of the year
// 2) the "century" (year / 100) shifted up 24 bits
// 3) a leap year flag
void BuildDaysTbl()
{
	int i;
	int j = 0;
	uint32_t days = 366;
	days2years[0] = LEAP_FLAG;
	while (days < 0x40000000)
	{
		i = 24;
		while (--i >= 0)
		{
			days2years[++j] = days;
			days += 365;
			days2years[++j] = days;
			days += 365;
			days2years[++j] = days;
			days += 365;
			days2years[++j] = days | LEAP_FLAG;
			days += 366;
		}
		days2years[++j] = days;
		days += 365;
		days2years[++j] = days;
		days += 365;
		days2years[++j] = days;
		days += 365 + 0x10000000;
		days2years[++j] = days;
		days += 365;
	}
}


// convert a Julian day # to a Gregorian year, with a "remainder", and a leap year flag
// outputs: *doy = day of year (jan 1 = 0) with leapyear flag, *cent = century AD
uint32_t JDN_to_GregYear(uint32_t jdn, uint16_t *doy, uint32_t *cent)
{
	uint32_t greg_yr = 0;
	uint32_t qcen, temp, temp2;

	temp -= 1721060;			// make temp align with jan1, "year 0" quadcentury boundary
	qcen = temp / 146097;		// 400 years contain 146097 days
	temp %= 146097;				// get the remainder days within the 400 yr span
	temp2 = temp;				// save a copy
	// do an approximate conversion to years -- sometimes one too low
	if (temp > 73048)			// calculation for years 200 to 399
	{
		temp -= 73048;			// this shifts the calculation by 1 day
		greg_yr= 200;
	}
	greg_yr += (temp << 10) / 374014;	// 374014 is a magic number

	if ((days2years[greg_yr + 1] & 0xfffffff) <= temp2)	// fix if greg_yr is off by 1
		++greg_yr;
	temp = days2years[greg_yr];
	*doy = (uint16_t) (temp2 - (temp & 0xfffffff));
	*cent = ((temp & 0x30000000) >> 24) + qcen * 4;
	if ((int32_t) temp < 0)
		*doy |= 0x8000;		// set the leapyear flag on "day of year"

	return greg_yr;
}


// convert Jan 1 of a Gregorian year to a Julian day #
// "sign" bit of the return value is set on a leap year
uint32_t GregYear_to_JDN(uint32_t greg_yr)
{
	uint32_t jdn = (greg_yr / 400) * 146097;	// convert quadcenturies to days
	uint32_t temp = greg_yr % 400;
	// add in the days from the remaining years, and adjust jan1, "year 0" to actual JDN
	jdn += (days2years[temp] & 0xfffffff) + 1721060;
	jdn |= days2years[temp] & 0x80000000;		// copy the leap year bit
	return jdn;
}
</source>

== See Also ==
=== Articles ===
* [[CMOS]]
* [[Time And Date]]

=== External Links ===
* http://en.wikipedia.org/wiki/Julian_Day

[[Category:Time]]
````

<a id="w002931"></a>

## W002931 — User:Mariuszp

Bron-ID: W002931
Namespace: User (2)
Revisie: 23267 | Datum: 2019-01-30T22:16:56Z
Bron: https://wiki.osdev.org/User:Mariuszp
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMariuszp&oldid=23267

````text
[http://glidix.madd-games.org/ Glidix] is an operating system with a modular kernel, which ''mostly'' tries to implement the POSIX API, but is more focused on its own innovations. It has its own userspace with a full-fledged GUI, using its own object-oriented widget toolkit and compositor, "GWM". It even comes with a few games!

It currently uses ELF64 as its executable file format, and GCC/binutils as the toolchain. However, this is changing soon, with the introduction of its own format, and even its own toolchain!
````

<a id="w002932"></a>

## W002932 — Real Time Clock

Bron-ID: W002932
Namespace: Main (0)
Revisie: 10876 | Datum: 2010-10-17T20:45:51Z
Bron: https://wiki.osdev.org/Real_Time_Clock
Revisie-URL: https://wiki.osdev.org/index.php?title=Real_Time_Clock&oldid=10876
Doorverwijzing: RTC

````text
#REDIRECT [[RTC]]
````

<a id="w002935"></a>

## W002935 — Random Number Generator

Bron-ID: W002935
Namespace: Main (0)
Revisie: 27595 | Datum: 2023-01-16T02:22:23Z
Bron: https://wiki.osdev.org/Random_Number_Generator
Revisie-URL: https://wiki.osdev.org/index.php?title=Random_Number_Generator&oldid=27595

````text
Random number generators (RNG) can be implemented in lots of different ways. This article explains some of them.


== Entropy ==
Computers are deterministic devices. If the program is the same and all inputs are the same, the same result will be calculated every time. How, then, can a computer generate random numbers? The computer cannot, but the physical world around it can. Many physical events are random to some degree, or more technically, have some degree of entropy. A sound recording will contain some level of noise even in the best shielded recording rooms on the planet. Humans type on keyboards with an inconsistent timing, and type quasi-random text like the article you are currently reading. Even other computers can provide entropy: while most network traffic is entirely machine generated, the exact relative timing of network packets in the end derives from the moment the power button was pressed, another human action. Therefore the timing and nature of input events can provide us with some entropy.

== Types of random number generators ==

There are multiple ways to divide random number generators; one way is by how the numbers are generated, another is by how they can be used. These are important distinctions: the former may impact how you design your OS, and the latter can make or break the security of your system.

=== Sources of random numbers ===

There are two kinds of random numbers: "true" random and pseudo-random. 

For "true" random number generation, the system continuously measures a certain set of events that are expected to be random. This can be anything from cosmic radiation and atomic decay, to the timing of user input and clock jitter. Measurements are de-biased and "stirred" into a pool of <em>entropy</em>, from which random numbers can be extracted.

Pseudo-random numbers are generated by an algorithm (a <em>PRNG</em>) that transforms some internal state and calculates an output value upon request. The initial seed can be set, but after that the next state only depends on the previous state. There are many different PRNGs out there, some of which are discussed below.

It is possible to use some "true" random numbers to seed the state of a pseudo-random generator, but this does not make the PRNG "truly random". Depending on the exact algorithm, it may be trivial to predict all next outputs given as few as one previous output. This can have serious implications, as discussed in the next section.

=== Applications of random numbers ===

There are roughly two kinds of applications: cryptography, and everything else. 

In order to understand the following section, we first should review what cryptographic security means with respect to random number generators. Cryptographers distinguish two types of security: information-theoretic security and computational security. The former of these is the stronger claim. If a cryptosystem is information-theoretically secure, an attacker cannot say anything about an encrypted message without owning the key. Such cryptosystems are rare. Since the attacker must not be able to say '''anything''', everything including patterns, timing and even the length must be hidden. How can one even hide the length of a message? The only choices are to not send the message at all, which defeats the purpose of a cryptosystem, or to send an infinitely long message. So once you start sending, you can never stop.

This is why computational security is the weaker, but more practical kind: the ciphertext must be difficult to find, ''given some reasonable computing power''. This limit is chosen with the expectation that it will not be reached in the coming decades. This also allows the user to "leak" some information about the ciphertext (the length, for example), as long finding the original input remains difficult. These systems usually use some operation that is difficult to invert unless the key is known, such as [https://en.wikipedia.org/wiki/RSA_(cryptosystem) factoring very large primes]. It should be obvious that the key must not be predictable: if an adversary knows what the next outputs will be, it will be very easy for them to guess keys until the correct one is found.

"True" random numbers are the best source for cryptographically-secure randomness, however it is difficult to build a large, quickly-replenished pool of entropy where each source is perfectly de-biased and cannot be influenced by the other sources. To "stretch" the entropy in a way that is difficult to reverse, one can use a cryptographically-secure random number generator (CSPRNG). CSPRNGs guarantee that it is computationally difficult to guess the next output having seen previous results, and, if the generator's state is known, which values preceded the known outputs.

== True random number generators ==
True random number generators use physical devices or phenomena to generate random numbers, whose unpredictability can be traced to the laws of quantum mechanics.

=== x86 RDSEED Instruction ===
Newer x86 and x86-64 processors have the instruction RDSEED for generating random numbers.
To use RDSEED you will first need to check if the instruction is available.
<source lang="asm">
mov eax, 7     ; set EAX to request function 7
mov ecx, 0     ; set ECX to request subfunction 0
cpuid
shr ebx, 18    ; the result we want is in EBX...
and ebx, 1     ; ...test for the flag of interest...
</source>

If EBX is set to 1 (or ZF is unset), the instruction is available. Then it can be used to generate a 16/32/64 bit random number (depending on the register size used as an argument). If a random number was generated, the Carry flag is set. 

<source lang="asm">
mov ecx, 100   ; number of retries
.retry:
    rdseed eax
    jc .done      ; carry flag is set on success
    loop .retry
.fail:
    ; no random number available
.done:
    ; random number is in EAX
</source>

It is up to you and your users to trust RDSEED. Because it is implemented in hardware, it effectively is a black box that may contain all sorts of bugs, or worse, backdoors. 

=== Dedicated hardware ===

There exist devices dedicated to generating "true" random numbers. These range from the consumer-level TPMs, to PCIe "crypto accelerators". These are a generalization of RDSEED/RDRAND, with the downside that you need additional drivers to interface with the device and that the users may not have such a device installed.

TPMs, or Trusted Platform Modules, are small co-processors that can be installed on modern motherboards. In addition to random number generation, they also provide [https://en.wikipedia.org/wiki/Trusted_Computing other trusted computing services]. Notably, Windows 11 requires the presence of a TPM. They can also be emulated on the CPU (e.g., Intel PTT or AMD fTPM). As with RDSEED, backdooring may be a concern.

=== Sampling manually ===

There are plenty of processes happening with in a running system that are effectively random. Think about all the different interrupts that arrive every second: timers with accuracies of multiple parts-per-million, peripheral I/O, the user interacting with the entire system, and more. 

The challenge is finding sources that are (paradoxically) reliably random and difficult to influence and observe from outside. For each of these sources, an estimate must be made of how much entropy they contribute. Measurements add their respective amount of entropy to the pool, while reads decrease the entropy.

Because you have full control of this generation method, you can also incorporate the values generated by hardware generators. You can yourself decide how much entropy you count for these generations, even 0 bits.

When using timing as entropy source, the timestamp read should be as precise as possible. The TSC works well for this. Gauging the entropy gained from that operation requires knowledge of the timing window for the event to occur in and the tick rate of the TSC. For example, if a TSC has a tick rate of 3 GHz and an event has a 10ms window to occur, then the TSC read can have any one of 30 million values, which means the entropy gained from this is ca. 24.8 bits. Were the TSC slower, only 1 GHz, then the entropy would only be ca. 23.2 bits. 

==== Adversarial entropy ====
If an adversary can somehow observe the state of the entropy pool and contribute their own entropy into the pool, then it is possible that they would provide entropy in such a way as to force the entropy pool into a lower entropy state. A simple example would be an entropy source periodically checking if a certain bit in the pool is set, and then providing entropy until it is clear. This way, for most of the time, that given bit is clear, and the number of possible states the entropy buffer can be in is halved. A more sophisticated attack is described on DJB's blog: https://blog.cr.yp.to/20140205-entropy.html

It is also partly for this reason that it is unwise to expose the entropy pool unmodified if the user requests a random number. If an adversary has access to the pool (either via a dedicated "add-entropy" interface or a sampled event source), it will be very easy to poison it. A common method used to hide the exact state is to hash (parts of) the pool in combination with counter, for instance the entropy counter, and a salt, using a cryptographically secure hashing function like SHA-256. Because these hash algorithms are difficult to invert, its inputs cannot be easily guessed. It is important to do this only if the pool has some entropy left.

== Cryptographically secure pseudorandom number generators ==

Now follow some CSPRNGs. It is important to remember that, as with everything cryptographic, it is best <em>not</em> to homebrew it if you are planning on actually using it. There are many ways things can go wrong, and the more complex the algorithm, the more chances of you making a mistake. Of course, for hobby uses it's perfectly fine; just don't go online banking with your handmade TLS key source.

=== x86 RDRAND Instruction ===
RDRAND is slightly older than RDSEED, and provides ([https://software.intel.com/content/www/us/en/develop/blogs/the-difference-between-rdrand-and-rdseed.html what Intel claims to be]) a CSPRNG.
Its presence is indicated by CPUID leaf 1, ECX bit 30:
<source lang="asm">
mov eax, 1     ; set EAX to request function 1
mov ecx, 0     ; set ECX to request subfunction 0
cpuid
shr ecx, 30    ; the result we want is in ECX...
and ecx, 1     ; ...test for the flag of interest...
</source>

If ECX is set to 1 (or ZF is unset), the instruction is available. Usage is identical to RDSEED:

<source lang="asm">
mov ecx, 100   ; number of retries
.retry:
    rdrand eax
    jc .done      ; carry flag is set on success
    loop .retry
.fail:
    ; no random number available
.done:
    ; random number is in EAX
</source>

It is automatically seeded by the same entropy source that RDSEED reads from, and cannot be seeded manually. It therefore is subject to the same caveats as RDSEED.

=== Ciphers ===

It is fairly common to construct a CSPRNG by seeding a secure cipher, such as ChaCha20 and AES, and running many cycles where the output gets re-encrypted together with a running counter. Every once in a while, a new key is created, potentially involving another secure random source.

[https://www.bentasker.co.uk/blog/software-development/689-writing-a-chacha20-based-csprng Writing (and backdooring) a ChaCha20 based CSPRNG] may be an interesting article on the subject, and how it can go wrong in surprising ways.

== Pseudorandom number generators ==

Next are various regular PRNGs. These vary in all dimensions of quality, simplicity and speed. Carefully read each explanation!

=== The Standard's Example ===

Taken directly from the [http://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf C standard document (p. 347)]:

<source lang="c">
// The following functions define a portable implementation of rand and srand.

static unsigned long int next = 1;  // NB: "unsigned long int" is assumed to be 32 bits wide

int rand(void)  // RAND_MAX assumed to be 32767
{
    next = next * 1103515245 + 12345;
    return (unsigned int) (next / 65536) % 32768;
}

void srand(unsigned int seed)
{
    next = seed;
}
</source>

This is a fairly standard but mediocre [https://en.wikipedia.org/wiki/Linear_congruential_generator Linear Congruential Generator] (LCG). It returns 15 random bits.

It is based on this recurrence formula
  X<sub>n+1</sub> = (aX<sub>n</sub> + c) mod m
where the modulus m is the maximum number of random values the LCG can produce. For the example in the C standard, a = 1103515245, c = 12345 and m = 2<sup>32</sup> (implicitly).
The quality of the LCG depends heavily on these values and finding good values is difficult. Wikipedia has [https://en.wikipedia.org/wiki/Linear_congruential_generator#Parameters_in_common_use a list of common parameters].

=== Fibonacci random number ===
A special "remake" of the Fibonacci sequence can be used to generate random numbers. This is done by having 4 "seeds", which start off as really weird values (e.g. 45, 80, 22, 32).
The seed() function adds a new seed to the end of the sequence, and erases the first one (the seeds are referred to as A, B, C and D). The rand() function just returns the sum of the seeds, and calls seed() with the result. [[User:Mariuszp|Glidix]] is a good example of this RNG implementation.

=== Linear Feedback Shift Register ===
A Linear Feedback Shift Register (LFSR) is a simple way to generate a very long sequence of random numbers, given a non-zero seed. The idea is as follows: an ''n''-bit LFSR has ''n'' bits (either 0 or 1). Initially, the register is filled with an ''n''-bit seed. For each next value to be generated, "tap" certain bits from the register and XOR them together. Feed the resulting binary value into the left-most bit, shifting all bits one position to the right. The bit that is shifted out of the LFSR on the right-most side is the generated random bit.

The sequence of bits that an LFSR can produce will eventually repeat, but by choosing the XOR'ed bits carefully the period of the LFSR can be increased to up to 2<sup>n</sup> - 1 bits. This means that after a sequence of 2<sup>n</sup> - 1 bits, the same sequence will be returned. However, this is a property that all pseudo-random number generators have: without changing the seed, they will eventually repeat themselves.

The following is an example of an 16-bit LFSR using bits 11, 13, 14 and 16 XOR'ed together as its input. The period of this LFSR is 65535 bits, so it will generate a pseudo-random sequence of 65535 bits before the sequence repeats itself. The next bit produced by the LFSR is 1 (the value of bit 16) and the next input bit is 0.
<pre>
            1                                       11      13  14      16
          +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
INPUT --> | 0   1   0   0   0   1   0   0   1   1   1   1   0   0   0   1 | --> OUTPUT
          +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+

INPUT = bit 11 XOR bit 13 XOR bit 14 XOR bit 16
</pre>

Larger (and smaller) LFSR's are also possible. Smart people have derived polynomials that will ensure that given any non-zero input, the period of the LFSR is as large as it can be (2<sup>n</sup> - 1). Such a polynomial is written as x<sup>16</sup> + x<sup>14</sup> + x<sup>13</sup> + x<sup>11</sup> + 1. For this example polynomial for ''n'' = 16, bits 16, 14, 13 and 11 must be XOR'ed together and provided as input, counting from the left and starting with 1. Polynomials for other values of ''n'' can be found [http://en.wikipedia.org/wiki/Linear_feedback_shift_register#Some_polynomials_for_maximal_LFSRs here on Wikipedia] and on page 5 of [http://www.xilinx.com/support/documentation/application_notes/xapp052.pdf this PDF document].

Note that the seed must never be zero. This also means that it is never possible for all registers to have bit value zero, and that of the 2<sup>n</sup> possible combinations of registers, the all-zero state is not allowed. Therefore, 2<sup>n</sup> - 1 states is the maximum possible.

=== Wichmann-Hill ===
In 1982, Brian Wichmann and David Hill [https://doi.org/10.2307/2347988 proposed] to combine three linear congruential generators, then normalizing and summing the results to get a number uniformly distributed between 0 and 1. A common instance is:

<source lang="c">
static uint16_t seed[3]; /* seed with numbers between 1 and 30000 */
float wichmann_hill(void) {
  seed[0] = (171 * seed[0]) % 30269;
  seed[1] = (172 * seed[1]) % 30307;
  seed[2] = (170 * seed[2]) % 30323;
  return fmod(seed[0] / 30269.0 + seed[1] / 30307.0 + seed[2] / 30323.0, 1.0);
}
</source>

This version is known to have a period of just shy of of seven trillion (the least common multiple of 30268, 30306, and 30322).

=== Mersenne Twister ===

The Mersenne Twister is an incredibly popular PRNG and is available on most platforms; for example, it is part of the set of generators required by the C++11 standard library. An implementation of the 64-bit MT-19937 version in C is as follows:

Implementation according to Wikipedia:

<source lang="c">
#include <stddef.h>
#include <stdint.h>
#include <assert.h>

#ifdef USE32
typedef uint32_t word_t;
#define STATE_SIZE  624
#define MIDDLE      397
#define INIT_SHIFT  30
#define INIT_FACT   1812433253
#define TWIST_MASK  0x9908b0df
#define SHIFT1      11
#define MASK1       0xffffffff
#define SHIFT2      7
#define MASK2       0x9d2c5680
#define SHIFT3      15
#define MASK3       0xefc60000
#define SHIFT4      18
#else
typedef uint64_t word_t;
#define STATE_SIZE  312
#define MIDDLE      156
#define INIT_SHIFT  62
#define TWIST_MASK  0xb5026f5aa96619e9
#define INIT_FACT   6364136223846793005
#define SHIFT1      29
#define MASK1       0x5555555555555555
#define SHIFT2      17
#define MASK2       0x71d67fffeda60000
#define SHIFT3      37
#define MASK3       0xfff7eee000000000
#define SHIFT4      43
#endif

#define LOWER_MASK  0x7fffffff
#define UPPER_MASK  (~(word_t)LOWER_MASK)
static word_t state[STATE_SIZE];
static size_t index = STATE_SIZE + 1;

static void seed(word_t s)
{
    index = STATE_SIZE;
    state[0] = s;
    for (size_t i = 1; i < STATE_SIZE; i++)
        state[i] = (INIT_FACT * (state[i - 1] ^ (state[i - 1] >> INIT_SHIFT))) + i;
}

static void twist(void)
{
    for (size_t i = 0; i < STATE_SIZE; i++)
    {
        word_t x = (state[i] & UPPER_MASK) | (state[(i + 1) % STATE_SIZE] & LOWER_MASK);
        x = (x >> 1) ^ (x & 1? TWIST_MASK : 0);
        state[i] = state[(i + MIDDLE) % STATE_SIZE] ^ x;
    }
    index = 0;
}

static word_t mt_random(void)
{
    if (index >= STATE_SIZE)
    {
        assert(index == STATE_SIZE || !"Generator never seeded");
        twist();
    }

    word_t y = state[index];
    y ^= (y >> SHIFT1) & MASK1;
    y ^= (y << SHIFT2) & MASK2;
    y ^= (y << SHIFT3) & MASK3;
    y ^= y >> SHIFT4;

    index++;
    return y;
}</source>

Some implementations automatically seed the generator with seed 5489, but this will (obviously) lead to the same outputs at every initialization.


It outperforms all PRNGs listed above, but it is rather slow due to its large state size. It also has problems in some statistical areas. See [https://arxiv.org/abs/1910.06437 "It is high time we let go of the Mersenne Twister"].

=== PCG family ===

The [https://www.pcg-random.org/ PCG family] is a relatively recent addition to the field of PRNGs. The simplest version uses an LCG with a permutation operator to scramble the output. It was built to pass as many statistical tests as it could, while still staying small and fast. There is a [https://www.pcg-random.org/pdf/hmc-cs-2014-0905.pdf comprehensive paper] describing the generators and Apache 2.0-licensed code is available: https://www.pcg-random.org/download.html.

Note that the site claims that PCG's outputs are more difficult to predict than those of other PRNGs and that that means that PCG is more secure. It is possible to predict some generators after [https://pcg.di.unimi.it/pcg.php#claims only three outputs], so it should not be considered "hard to break" and definitely not "more secure". The predictability of a ''non''-cryptographically-secure PRNG is usually not a problem.

=== xoshiro family ===

Like PCG, the xoshiro and related xoroshiro families are fairly new PRNGs.

=== Rolling your own ===

If you are still not satisfied, you may want to build your own PRNG. You can do this in a fairly trial-and-error way by writing something that looks good and subjecting it to automated statistical testing suites such as [http://simul.iro.umontreal.ca/testu01/tu01.html TestU01]'s SmallCrush, Crush and finally BigCrush. These tests are very easy to run and can quickly point out problems, for example:

<source lang="c">
#include <unif01.h>
#include <bbattery.h>

#include <stdint.h>

static uint32_t next = 1;
unsigned int custom_rand(void) {
    // This is a terrible generator!
    next = (next * 0x4B4B9656U) ^ (next * 0x565AC3C3U) + 1;
    return next * (next - 1);
}

int main(void) {
    unif01_Gen *gen = unif01_CreateExternGenBits("custom_rand", custom_rand);
    bbattery_SmallCrush(gen);  // or bbattery_Crush or bbattery_BigCrush
    unif01_DeleteExternGenBits(gen);
}
</source>

SmallCrush will report that this generator failed 12 out of 15 statistical tests. The other tests, which are also much slower, are therefore not necessary. 

[[Category:Common Algorithms]]
````

<a id="w002936"></a>

## W002936 — PAE

Bron-ID: W002936
Namespace: Main (0)
Revisie: 10892 | Datum: 2010-10-23T13:11:47Z
Bron: https://wiki.osdev.org/PAE
Revisie-URL: https://wiki.osdev.org/index.php?title=PAE&oldid=10892
Doorverwijzing: Physical Address Extension

````text
#REDIRECT [[Physical Address Extension]]
````

<a id="w002938"></a>

## W002938 — Talk:Remote Data Streams

Bron-ID: W002938
Namespace: Talk (1)
Revisie: 10923 | Datum: 2010-11-05T10:04:28Z
Bron: https://wiki.osdev.org/Talk:Remote_Data_Streams
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ARemote_Data_Streams&oldid=10923

````text
Pardon me, but is there a reason an article on this is needed?
:When I look at [http://wiki.osdev.org/index.php?title=Unix_Pipes&curid=1915&diff=10910&oldid=4972], I would say "no". "What I call..." is a very weak base for an article (stub). -- [[User:Solar|Solar]] 12:13, 4 November 2010 (UTC)
::After verifying that it was indeed a custom extension by a single user, I removed the page. Complaints please here or via PM on the forum. -- [[User:Solar|Solar]] 10:04, 5 November 2010 (UTC)
````

<a id="w002939"></a>

## W002939 — Talk:Babystep1

Bron-ID: W002939
Namespace: Talk (1)
Revisie: 14387 | Datum: 2013-01-24T19:14:44Z
Bron: https://wiki.osdev.org/Talk:Babystep1
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABabystep1&oldid=14387

````text
== Do we need this? ==

Is this tutorial really needed? There are a few problems with it:
:1. It's mainly an assembly tuorial and a poor one at that.
:2. There's no new information in this series.
:3. I've noticed that many people from IRC stumble across this and think it's what they need when they actually want the bare bones tutorial.

Any arguments in favour of it? --[[User:Love4boobies|Love4boobies]] 20:56, 6 November 2010 (UTC)

:I cannot really judge the quality of the tutorial. If it's bad, it should be improved. But I ''do'' feel we should keep it here. It ''does'' show what a bootloader does, how to set up CPU mode etc., whereas the Bare Bones tutorial shows how to get from GRUB into your ''main()'' - two very seperate things.
:So '''keep''', definitely, and '''improve''', if necessary. (I haven't followed this tutorial myself, and anyway I'm not the one to judge the quality of anything written in ASM.) -- [[User:Solar|Solar]] 11:03, 8 November 2010 (UTC)

::If its purpose is to explain assembly language (which it attempts to do several times - e.g., the useless attempt on somewhat explaining instruction encoding) then yes, it does that poorly. On the other hand, if its purpose is to be a tutorial on constructing boot loaders then I would like to point out that all it does is show a show a printing routine (the equivalent of that routine in C would be something like for (char *p = str; p != '\0'; ++p) putchar(*p); instead of puts(str); -- not really something people want to do) and switch to protected mode.

::We have an article on constructing boot loaders, one on protected mode and one on the A20 gate (not covered by this series) - all of them with sample code. Readers might benefit more if they browse around. Of course, that's just my opinion. I obviously won't delete anything unless the other contributors agree with me. --[[User:Love4boobies|Love4boobies]] 00:42, 9 November 2010 (UTC)

:'''keep''' and maybe improve. As far as didactics go, most people need the same thing told thrice in different ways for them to understand it. Also, for the same line of reasoning, examples are a totally different thing than theory. The babystep tutorials is an excellent case of teaching by example, as well as a showcase of how to put all the tiny bits together to match a specification. I went up to babystep 4 in a short time and while some things may need some more elaboration, I don't see any serious educational problems. (But please fix that floating div, it's ugly). - [[User:Combuster|Combuster]] 09:51, 9 November 2010 (UTC)

I edited the code, 0x55, 0xAA is wrong. it should be 0xAA, 0x55.
- 0xjarno

:And I have reverted your changes because they are incorrect. :) --[[User:Love4boobies|Love4boobies]] 16:04, 23 January 2013 (CST)

Yes they are. More information: X86 is little endian. Therefore 0x55,0xAA = 0xAA55. The whole reason for this is you don't concern size when cast from DWORD to BYTE: 0x89ABCDEF -> 0xEF,0xCD,0xAB,0x98 -> (BYTE) 0xEF (low bits). Don't worry, it happens to us all at some point, for me it was copying partition data from existing MBR to new MBR I was about to write (EEK!). Once it happens, you'll never make the same mistake again. --[[User:Bellezzasolo|Bellezzasolo]] 13:14, 24 January 2013 (CST)
````

<a id="w002940"></a>

## W002940 — User:Darwish

Bron-ID: W002940
Namespace: User (2)
Revisie: 29189 | Datum: 2024-07-19T21:36:22Z
Bron: https://wiki.osdev.org/User:Darwish
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADarwish&oldid=29189

````text
Homepage:

* [https://darwish.chasingpointers.com darwish.chasingpointers.com]

Pages edited:
* [[Creating a 64-bit kernel‎]]: Several fixes to the compiler flags and the linker script.
* [[Scheduling Algorithms‎]]: Information on the Unix SVR2 kernel.
````

<a id="w002941"></a>

## W002941 — Qemu

Bron-ID: W002941
Namespace: Main (0)
Revisie: 10948 | Datum: 2010-11-17T04:06:42Z
Bron: https://wiki.osdev.org/Qemu
Revisie-URL: https://wiki.osdev.org/index.php?title=Qemu&oldid=10948
Doorverwijzing: QEMU

````text
#REDIRECT [[QEMU]]
````

<a id="w002942"></a>

## W002942 — User:Mduft/HigherHalf Kernel with 32-bit Paging

Bron-ID: W002942
Namespace: User (2)
Revisie: 11041 | Datum: 2010-12-11T08:02:36Z
Bron: https://wiki.osdev.org/User:Mduft/HigherHalf_Kernel_with_32-bit_Paging
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMduft%2FHigherHalf_Kernel_with_32-bit_Paging&oldid=11041

````text
{{In Progress}}

= Welcome =

Hi, and welcome to this small tutorial. It is assembled from the code, that i have written for my own hobby OS. As a prerequisite i recommend that you make yourself familiar with the AT&T x86 assembler syntax used by the GNU assembler (which i will be using here). I will try to explain everything i do in very much detail, but still - it's not trivial :)

I also suggest you have the Intel Processor Documentation(s) at hand, especially Volume 3a, which explains 32-bit paging and all the related structure, etc.

== QEMU ==

As a side note, i want to use this to advertise [[qemu]] as a development utility (no i'm not affiliated with 'em ;)). I use it very much, and it is really, really useful. For example you can investigate the current page tables of your kernel right after loading CR3, which prove very usefull when writing this code. Also having a symbolic debugger (although "symbolic" is not so much usefull in assembler ;p) is the bare minimum you should be equipped with when going on the kernel development route...

Additionally, you don't have to worry about boot loaders and such, as long as your kernel is multiboot compliant.

= Boot code Structure =

I will give a rough outline of the kernel (if you can call my 3-filer a kernel ;p) as i will explain it here:

* boot.S: This is the main meat of the startup code. it will contain code to:
** setup a initial stack.
** setup initial paging structures (the kernel PD, and two PTs)
** identity map the low 1MB, and all of the kernel.
** map all the kernel to the higher half
** enable paging
** call the C kernel
** write "PANIC!" on the screen in nice white-on-red letters
**:(this is why you read this, right?)
* link.ld: This will contain the instructions for the linker.

Also, i plan to provide the original files i use in my kernel, so you can have a closer look at them. When reading through them, you will mention, that i have (basic) C++ support in them. You either don't have to worry about it, or can use it to extend your own kernel in this direction.

(you can find [http://dev.gentoo.org/~mduft/osdev/boot.S boot.s here], and [http://dev.gentoo.org/~mduft/osdev/link.ld link.ld here])

= Thinking about it =

At first, i want you to think a little about what we will need, and what we want to accomplish.

We want to have a kernel, which runs (not loads!) at a high address. At such a high address, that it may be, that there is not so much '''physical''' memory, that this address is available. So we need virtual memory (or a segmentation trick, described in ''[[Higher Half With GDT]]''). We will use paging, to map our kernel to a high address after it has been loaded to a low address (which should be always available physically).

For this to work correctly, the ''code'' in our kernel needs to know about this. Most of the kernel will have to be ''linked'' to a high address, so we don't need to worry about this anymore, once we're over the bootstrap phase. The initial bootstrap code, in contrast, needs to be linked to a low address, '''or''' be completely position independent.

For the paging stuff, we will need to have at least a PD (Page Directory) for the kernel, and two PTs (Page Tables). Why two, you might ask. The reason is, we need one PT for the low addresses (the lower 1MB, and the kernel in lower half), and one for the addresses in the higher half. If your kernel grows larger and larger, it might even require more PTs, and you will for sure need to deal with PDs and PTs a lot when it comes to processes, where probably each process will have it's own PD and associated PTs.

For this tutorial i chose some very common addresses: 
* 0x100000 as the physical load address of the kernel. This is the location the bootloader will load our kernel to.
* 0xC0000000 as the ''offset'' into the higher half.

Those two added together, are the address where the kernel will finally be running (so 0xC0100000). You will see the use of KERNEL_HIGH_VMA all through the code to take into account the offset to the higher half. This is, because the linker assigns higher half addresses to some of the symbols. Since our kernel is loaded to lower half and "thinks" it is running in higher half, we need to be careful when using the addresses of symbols assigned by the linker. During the initial bootstrap, i'm doing a lot of adjustments from higher to lower half, so things are accessible as long as we don't have paging set up.

= boot.S =

Now for the real code. i will go through it, and explain every section as thorough as possible.

== Global symbols ==

We will have only one single global symbol for this file, and thats the entry point of the kernel. So the file starts with the following line:

 .global bootstrap_ia32

== The required .data stuff ==

We will need some room in our kernel to do the things we want to. However, we don't (yet) have means to dynamically allocate things, and we don't want to worry too much, so we will simple reserve all the required space in the executable itself. this way, the linker will take care of the kernels bounds, and the bootloader (or qemu in our case), will take care about loading, and checking for enough room, etc., etc.

=== Use the .data section ===

First, we'll tell the assembler, that we want to put the following stuff in the .data section of the output file:

 .section .data

=== Make room for the PD and PTs ===

Then, we need room for the PD and PTs. I'll reserve some space for them like this:

 # the kernel page directory, lowest page table (for the low 1MB identity 
 # mapping in kernel space) and the kernel page table, used to map the
 # kernel itself from physical 0x100000 to 0xC0000000).
 
 .align 0x1000
 _kernel_pd:
    .space 0x1000, 0x00
 _kernel_pt:
    .space 0x1000, 0x00
 _kernel_low_pt:
    .space 0x1000, 0x00

The above tells the assembler to pad the current position in the output file until it is aligned to 4K (0x1000 -> 4096). The we reserver three times 4K of space for the PD and PTs. These symbols will end up in the .data section of the kernel, and thus will be there as soon as the bootloader loaded the kernel. No need to worry about allocating those from "somewhere".

=== Make room for the Stack ===

Another thing we will need from the very start, is the stack. we won't have time to defer it's creation until we can allocate memory, so we need to put it in here too. Of course you can make it way smaller if you like; Linux uses 16K or 8K if you tell it to do so.

 # a whopping 64K of initial stack space.
 .set INITSTACKSIZE, 0x10000
 initstack:
    .space INITSTACKSIZE, 0x00

=== Strings ===

The last thing in the .data section are our strings we will be using from within the assembler code. There are not many of them, just the "PANIC!" i promised above :)

 _msg_panic:
    .asciz "PANIC!"

== The actual code ==

... or not quite yet. At least we tell the assembler to put the following things in the .text section.

=== Use the .text Section ===

The following tells the assembler to use the .text section. Also, i tell it to produce 32-bit code, which is not really a necessety, but i like to make it clear.

 .section .text
 .code32

=== Now for the exception to the rule ===

... the MultiBoot Header. It is put in the .text section for a special reason. The multiboot specification states, that the header '''must''' appear longword-aligned aligned somewher in the first 8192 bytes of the kernel. As you will see later on in the linker script, the .text section is the first section in the kernel, and thus has to contain the header.

 .set ALIGN,         1<<0             # align loaded modules on page boundaries
 .set MEMINFO,       1<<1             # provide memory map
 .set FLAGS,         ALIGN | MEMINFO  # this is the Multiboot 'flag' field
 .set MAGIC,         0x1BADB002       # 'magic number' lets bootloader find the header
 .set CHECKSUM,      -(MAGIC + FLAGS) # checksum required
 
 .align 4
 
 multiboot_header:
    .long MAGIC
    .long FLAGS
    .long CHECKSUM

The above first sets some "absolute" symbols. They have a fixed value, and can be used as constants. The it sets the alignment of the following to 4 (longword; remember? the header has to be aligned like this... most likely, we could omit this, since it is the first thing in the file anyway, and the chance that we are misaligned here is very small to impossible. but still.. better be on the safe side).

A good thing to know is, that the ".set symbols" are present in the symbol table of the output file, but ''do not take up any space'' in the file at the current location. This means, for example, that the current alignment does not change because of their declaration. We're still at the very same position in the output file.

=== Required symbols for the screen buffer ===

We also need some helpers for handling the screen (clearing it, and then, the point of the whole tutorial: printing "PANIC!"):

 .set VIDEO_RAM,     0xB8000          # Video Memory, used to print to the screen.
 .set VIDEO_DWORDS,  0x3E8            # The count of DWORDs (!) the screen buffer is large.

=== The entry point ===

Now the entry point function, the one that will be visible from the outside world. We will start by setting up the inital stack. Because (as we all know, right?) the stack grows downwards in memory, we need to set the ESP register to point to ''the end'' of the stack space.

 bootstrap_ia32:
    # setup the stack
    mov   $(initstack + INITSTACKSIZE), %esp
 
    # adjust address to be physical, stack is in data segment, which is linked to
    # the kernels virtual higher half address.
    subl  $KERNEL_HIGH_VMA, %esp

This will put the address of the ''initstack'' symbol, with the length of the stack added to it, into ESP, the stack pointer. What you do not know yet (and what i will explain in detail when we come to the linker script): the stack symbol (as it is in the .data section) is assigned a higher half address by the linker. Thus we cannot yet access it, because we're still running in the lower half now. If we would not subtract the offset into the higher half, where the kernel will be mapped after paging is enabled, the kernel would '''maybe''' still run fine - as long as the machine has enough memory, so accessing the higher half address works (however i haven't tried this!).

Next, we'll want to setup some form of initial paging, which enables the kernel to run code off the higher half addresses, and use all the other things that will be linked there.

    # setup boot paging to map kernel to higher half
    call init_boot_paging_ia32

I will discuss the init_boot_paging_ia32 function in detail just a few lines further down. Let's first finish the entry point, as it is not so complex.

Imagine, we have paging setup all right (the init_boot_paging_ia32 function did it's job, whatever this means), and can now use the correct higher half addresses all over the place. The first thing we'll want to do, is to try whether it really works, and relocate the stack from the lower to the higher half.

"Relocate" is maybe a bad word for what we're doing. Either way, we're accessing the same '''physical''' memory, because we mapped the higher half to the lower half physical addresses. In fact, we can use '''both''' addresses just fine. Also, if you, say, write a byte to 0x100000 (please don't ;)), it will immediately appear at 0xC0100000 too; it's the same memory location.

    # adjust stack registers to point to the now mapped stack as virtual address
    addl  $KERNEL_HIGH_VMA, %esp
    mov   %esp, %ebp

Now the stack is used from it's higher half mapped address.

Since we have all set up now, we can test the lower 1MB mapping very easily, by trying to clear the screen, which will access the screen buffer at 0xB8000. If the mapping doesn't work, this will cause one or the other exception, double fault and tripple fault :)

    mov $VIDEO_RAM, %edi
    mov $VIDEO_DWORDS, %ecx
    mov $0x07200720, %eax
    rep stosl

This fill all the screen buffer with light-grey-on-black colored blanks, and thus clear it. (Each word is built like this: 0x0000 -> background (0 = black), 0x0700 -> foreground (7 = light grey), 0x0020 -> " " (blank)).

The only thing left to do (except error handling), is to call the main C kernel from here. First we push the parameters passed by the Multiboot compliant bootloader on the stack.

    # push parameters to the entry point (grub parameters)
    push  %eax
    push  %ebx
 
    #call boot # uncomment this if you have a C kernel entry function at hand :)

My kernel main C function happens to be named "boot", but thats of course up to you. Now, actually there is all we need. Still we will want to make sure, that if boot ever returns, the CPU does not continue to execute junk or code it shouldn't, so we will halt the CPU. '''Note:''' You'll probably want to halt all APs too, if you put SMP support in your kernel...

 the_end:
    mov $_msg_panic, %eax
    call boot_print_msg
    # halt the cpu... the kernel stopped.
    cli
    hlt

I think it is pretty obvious: the above loads the "PANIC!" message into EAX, calls the print function (i will show it to you in a second...), then disables interupts and halts the CPU.

=== The simple printing function ===

Ok, before we start with the paging code, here is the printing function i have to write the "PANIC!" on the screen. I'll leave it to you to figure out how it works, i guess things like these are described a lot in th wiki :)

 boot_print_msg:
    # eax: address of the string to print.
    push %edx
    push %ebx
 
    mov $VIDEO_RAM, %edx
 
    _print_loop:
        movb (%eax), %bl
        xorb %bh, %bh
        cmpb $0x0, %bl
        je _end_print
        orw $0x4F00, %bx
        movw %bx, (%edx)
        add $0x2, %edx
        inc %eax
        jmp _print_loop
    _end_print:
 
    pop %ebx
    pop %edx

=== Initialize Paging ===

The following is my init function for the paging. It makes the PD point to the correct PTs. It also calls another function to fill the PTs with the correct entries (it "maps" virtual to physical addresses). Let's start with the top of the function:

 init_boot_paging_ia32:
    # save registers used here.
    push %eax
    push %ebx
    push %edx
    push %ecx

At the end of the function, we will restore the registers, so the code behaves nicely, and doe not overwrite reigsters set by the bootloader (remember: grub passes two parameters in EAX and EBX).

==== Getting the PD and PTs right ====

Now, to insert the PDEs in the PD, we need to have the correct physical addresses of the PD and PTs. Those symbols (_kernel_pd, _kernel_pt, _kernel_low_pt; we declared those above in the .data section, remember?) are assigned '''higher half''' addresses by the linker, as you will see later on, when i explain the linker script. Since the higher half addresses are merely an "alias" to the lower half addresses, we can simply subtract the offset into higher half, to get the actual physical load address of those symbols. This way, we can access them without paging enabled.

    mov  $_kernel_pd, %eax          # get virtual address of kernel pd
    sub  $KERNEL_HIGH_VMA, %eax     # adjust to physical address

EAX now contains the ''physical'' address of the kernels PD. Now the same for the "low" PT (the one mapping the first 1MB of the memory, and all the physical kernel addresses, as the kernel is loaded right above 1MB. With this single PT, we can map up to 4MB, that makes room for a approximately 3MB kernel above the 1MB low memory).

    mov  $_kernel_low_pt, %ebx      # get virtual address of kernel low pt
    sub  $KERNEL_HIGH_VMA, %ebx     # adjust to physical address

If we want to set a PDE containing this PT, we just need to set the present flag. ('''Note:''' i ''don't'' clear the lower 12 bits of the address intentionally; it is not required, as i can be sure, that the structures are correctly aligned the way i declared them in the .data section - just in case you wonder if i forgot it ;)).

    or   $0x1, %ebx                 # set present flag

EBX is now in the correct form for a PDE, the only thing left to do, is actually set the entry in the PD (whos address is loaded into EAX):

    mov  %ebx, (%eax)               # set the pde

This was easy. The next one is a little more tricky. The low PT has the advantage of beeing at index zero of the PD. The higher half PT for the kernel doesn't have this advantage; we will first have to calculate the offset into the PD, where we want to set the PDE.

    push %eax
    mov  $KERNEL_HIGH_VMA, %eax     # get virtual address offset
    shr  $22,  %eax                 # calculate index in the pd
    mov  $4, %ecx
    mul  %ecx                       # calculate byte offset (4bytes each entry)
    mov  %eax, %edx
    pop  %eax

The above code does this: It first saves the EAX register, as we'll need it's value again later (and restores it at the end, see?). Then it loads the higher half kernel offset into EAX. I need this for a special reason: I assume, that all the kernel fits entirely into the one PT for now. I need to have ''any'' virtual address that should be mapped by the kernel PT. From there i can extract the Index that the PDE should have in the PD. It is not really required to load the actual offset here (although it's the clearest way i can think of). Anything linked to higher half will do (for example the address of the _kernel_pd or _kernel_pt symbols themselves). Next, EAX is right shifted by 22 bits to extract the ''index'' into the PD. Since all entries are 4 bytes each, we need to multiply the index with 4, to get the offset of the PDE from the PD base address. This offset is saved in the EDX register for later use.

To get the right position for our PDE, we now just have to add the offset to the PD base pointer, we loaded before.

    push %eax                       # save the real address for later
    add  %edx, %eax                 # move the pointer in the pd to the correct entry.

Now that we know the correct location, the code to set the _kernel_pt PDE entry is very similar to the one used for the _kernel_low_pt, so i'll leave it up to you to figure out those four lines:

    mov  $_kernel_pt, %ebx          # get virtual address of kernel main pt
    sub  $KERNEL_HIGH_VMA, %ebx     # adjust to physical address
    or   $0x1, %ebx                 # mark present
    mov  %ebx, (%eax)               # set the pde

If we reach this position in the code, the PD and it's PDEs are don. It points to the correct physical addresses, and we don't need to worry about it anymore, except when enabling paging below, where we will move the pointer to the PD into the CR3 register.

==== The real mapping ====

For this code, we will, again, have to assume for a moment, that we have a working function called ''boot_map_page_ia32'' which takes the following parameters:
# EBX: the physical address of the kernel PD
# ECX: the virtual address which we want to map
# EDX: the physical address that the virtual address should be "redirected" to.

After calling the function with the correct parameters, accessing the given virtual address will really access the physical memory we passed in EDX. We'll come to the real implementation of the function a little further down the road (not too long to wait, don't worry ;)).

We need to map 3 things:
# We have to "identity map" the lowest MB in memory (except the first page! we don't want to be able to access 0x0; this should cause an exception!). Identity mapping means that the virtual and physical address are the same. We need this to be able to use some fixed addresses (video buffer at 0xB8000 for example). We could, of course, also map the low MB somewhere else, and adjust all pointers to the new location.
# We have to identity map all the kernel in lower half. If we wouldn't do this, enabling paging would lead to an immediate page fault, then double fault, then tripple fault the CPU. This is because our EIP is currently somewhere in the physical load range of the kernel. after enabling paging, the CPU tries to fetch the next instruction, but the address it tries to read is a ''virtual'' one now! if the virtual address is not the same as the physical one, the CPU won't get the next instruction. (In fact, it will probably page fault, if nothing is mapped at that location).
# Last but not least, we have to map all the kernel into higher half, so we can call the C kernel entry later on. This mapping is merely an "alias" to the lower half kernel. After all mappings are complete, we can access the ''same'' physical memory through ''both'' the higher and lower half addresses. If we like to, we can later on remove the lower half mapping, so that the only way to access the kernel's memory is through the higher half pointers.

Ok, let's go. first, we need to have the physical address of the kernel PD in EBX. A few lines up, i pushed EAX, which contained the address we want. So all we have to do, is pop it off the stack now.

    pop  %ebx                       # pop saved address of the kernel PD

Now we're ready to loop over the first MB. I'll set 0x100000 (1MB) into ECX, which will contain the virtual address we want to map. Each iteration will decrease ECX by 0x1000 (one page, 4K), and call the mapping function. We stop (without calling the mapping function), if ECX reaches zero.

    mov  $0x100000, %ecx            # map the low 1MB
 
    _idmap_first_mb_loop:
        mov %ecx, %edx              # phys == virt (identity mapping)
        call boot_map_page_ia32     # do the mapping
        sub $0x1000, %ecx           # one page down.
        jnz _idmap_first_mb_loop    # if not zero, continue (DON'T map zero :))

As you can see, the loop sets EDX to the same value as ECX, and thus "identity maps" all of those addresses. ('''Hint:''' later on, when you run your kernel in qemu, you can look at the mappings easily by typing "info tlb" in the qemu monitor).

That was easy, right? Now for something slightly more sophisticated. We'll identity map ''and'' "high-map" the kernel in one go. For this, we'll have to know the bounds of the kernel. The linker script provides a few additional symbols, which help us there. The KERNEL_BOOT_VMA symbol is the absolute physical start address of the kernel, so we'll start there (i put it into ECX and increment by a page each loop run, until we reach the end of the kernel). The end of the kernel is determined by looking at the _core_end symbol. '''But''': carefull; the _core_end symbol (as you will see when looking at the linker script) is linked to higher half, so to get the ''physical'' end of the kernel, we need to subtract the higher half offset again.

    mov  $KERNEL_BOOT_VMA, %ecx     # this is the _very_ beginning :)
    mov  $_core_end, %eax           # virtual address of end
    sub  $KERNEL_HIGH_VMA, %eax     # now it is physical.

Now that we have all values set up (EBX, which is required too, wasn't changed in the meantime, so no need to worry about it; it has the correct value), we can go for the actual loop:

    _map_kernel:
        mov %ecx, %edx              # phys == virt (identity mapping)
        call boot_map_page_ia32     # do the mapping
 
        push %ecx
        add $KERNEL_HIGH_VMA, %ecx  # now map the virtual address to the same physical one
        call boot_map_page_ia32     # do it
        pop %ecx
 
        add $0x1000, %ecx           # on to the next page.
        cmp %eax, %ecx
        jle _map_kernel             # continue untill all the kernel is mapped.

This first sets EDX to identity map ECX, and creates the mapping (like in the last loop). But then, surprise, surprise, something more happens: I save ECX, as i need it's value as is a few lines further down. The i adjust ECX to point to the same address as before, ''but in higher half''. Calling the mapping function now creates a mapping from higher half (virtual) to lower half (physical) memory. Then, ECX is restored, so we can check for the kernel bounds and continue the loop. ECX is moved on to the next page we want to map. ECX is less-or-equal to the upper kernel bounds, we continue, otherwise we're done.

==== Enable paging ====

That's it. We're nearly done. All the mappings are in place, all the PD and PTs are ready to be used (hopefully ;)). Now we can give it a try by activating all of this, which is really simple:

    mov  %ebx, %cr3                 # use the kernel pd
    mov  %cr0, %eax                 # get the current cr0 value
    or   $(1 << 31), %eax           # enable paging
    mov  %eax, %cr0                 # now re-set the cr0 register.

First, EBX is moved to CR3, which is the PD base pointer in 32-bit paging. Then, the paging bit is set in CR0, and that's it. Lets keep our fingers crossed that things continue to work ;)

==== Cleanup ====

Last thing to do for this function is to restore all the registers and return to the bootstrap_ia32 function. The first thing there will be, if you remember, adjusting the stack to use the virtual higher half addresses...

    # restore register values.
    pop  %ecx
    pop  %edx
    pop  %ebx
    pop  %eax
 
    ret

=== Mapping single pages ===

Oh - wait; we're not quite there, right? i think there is one more function we deferred for later implementation... the one that maps a single page, which we used above in the loops: boot_map_page_ia32.

==== Setup ====

Always the same boring function starts...

 boot_map_page_ia32:
    # ebx: physical addr of kernel PD
    # ecx: the virtual address to map
    # edx: the physical address to map to
 
    push %eax
    push %ebx
    push %ecx
    push %edx

Additionally, we will immediately push some of the parameters, as we need to calculate offsets from them, and later have those values unmodified.

    push %edx                       # push physical address
    push %ecx                       # push virtual address

==== Find the PT in the PD ====

The first real code will find out, which PT we will have to use to be able to map the virtual address. If the given address is not within the two PTs we have already set up, the kernel will panic immediately (of course; there is no logic to create new PTs on the fly).

    mov  %ecx, %eax
    shr  $22, %eax
    mov  $4, %ecx
    mul  %ecx                       # now we have the offset in eax
    add  %eax, %ebx                 # now ebx points to the phys addr of a pt if present
    mov  (%ebx), %eax

To get the correct index, the above first loads the virtual address from ECX into EAX, then shifts it 22 bits to the right, as this will, again, result in the index into the PD, required to find the PT for this address. Then again, we have to multiply it by four, as each entry in the PD is four bytes long. Finally the calculated offset (now in EAX) is added to the physical address of the PD, which is passed in EBX (remember?). From this address, we can now load the PDE into EAX. Please remember that this now is '''not''' and address yet, as it has flags ORed into it. We'll have to clear those flags before we can use it.

But first, just to be sure, we check if the present flag is set in the PDE, otherwise the given virtual address is out of bounds.

    mov  %eax, %ecx
    and  $0x1, %ecx                 # check present flag
    cmp  $0x0, %ecx
    je the_end                      # if zero, PANIC!

Now, as i said, we'll have to extract the physical PT address from the PDE.

    and  $0xFFFFF000, %eax          # clear off possible flags from the PDE.

==== Calculate PTE address ====

Finally, we have the correct physical address of a valid PT in EAX. We no can use this to find the correct slot in there, and set a PTE for our virtual address.

Remember, at the entrance of the function, we pushed both the physical, and then the virtual address passed in; now we need the virtual address again to calculate the correct index into the PT from it, so pop it off the stack:

    pop  %edx                       # the virtual address.

The following code will calculate the index, and put it into EBX:

    push %eax
    mov  %edx, %eax
    shr  $0xC, %eax                 # shift right to discard non-significant bits.
    and  $0x3FF, %eax               # and away not-relevant bits on the left.
    mov  $0x4, %ecx                 # each entry is 4 bytes
    mul  %ecx
    mov  %eax, %ebx                 # now in ebx: the offset into the PT for the PTE.
    pop  %eax

EAX is preserved (first pushed, then poped at the end of this section), as you can see. Then, I put the virtual address to map (in EDX) into EAX. After that, right-shift the whole thing by twelve bits. Now clear off the uninteresting bits on the left side, so only the range described in the intel manuals is left over. This is the part of the address that serves as index into the PT. As always, we now have to multiply it by four, since all PT entries are (exactly like the PD entries) 4 bytes long. The result is put into the EBX register.

==== Settings the PTE ====

Let's evaluate our current state. EBX contains the ''offset'' from the PT base address for our virtual address to map. EAX still contains the base address of the PT. What we now additionally need is the physical address we have to map to. At the start of the function we pushed it on the stack, so pop it off now.

    pop  %edx                       # the phsyical target address

Now calculate the final location of the PTE, by adding up the base address and the offset, in EAX and EBX respectively.

    add  %ebx, %eax                 # add offset to pt. this is the final location now.

To get a valid (and more important: ''present'') PTE, we need to OR some flags, in this case only the present flag to the final physical address. Then we can put the PTE (made up of the address and the present flag) to the location we just looked up in the PT (which is in EAX).

    or   $0x1, %edx                 # mark present...
    mov  %edx, (%eax)               # and insert into pt.

==== Cleanup ====

The rest is cleanup (restore registers), and return.

    pop %edx
    pop %ecx
    pop %ebx
    pop %eax
    ret

== Conclusion ==

Wow. That was quite some code. i know, it is hard to get all this into the head ;)

Please also note, that some of the code is not exactly the best, fastest way to do things, but the one how i understood the concepts best. If you have suggestions, feel free to tell me! I'm eager to hear how you would improve the code!

= link.ld =

To be able to actually ''build'' the kernel, you'll need one more file: the linker script, which tells the linker where to put things, which addresses to assign, and what symbols to create to help the boot code... Let's have a look at the contents:

== Basic setup ==

The file starts with a few global symbols and instructions for the linker:

 OUTPUT_FORMAT(elf32-i386)
 ENTRY(bootstrap_ia32)
 KERNEL_BOOT_VMA = 0x00100000;
 KERNEL_HIGH_VMA = 0xC0000000;

This tells the linker, that we want a 32-bit elf executable, containig i386 code, that our entry-point function is named "bootstrap_ia32" (as defined in boot.S). Also two very important absolute symbols are defined: KERNEL_BOOT_VMA, which denotes the physical start address of the kernel, and KERNEL_HIGH_VMA, which denotes the '''offset''' from the BOOT_VMA to the higher half (It is important to understand, that these are ''symbols'', not just constants you can use from withing the linker script. Those symbols are usable from the code (which we did alread)).

== The output Sections ==

Next comes the definition of the sections ''in the output file''. This is only indirectly related to the sections in the ''input files''. We can more or less freely move/rename and merge sections from the input to the output. Also, we can tell the linker that special files should be treated specially, which is of great value.

Let's have a look at the beginning of the section definition:

 SECTIONS{
    . = KERNEL_BOOT_VMA;

This tells the linker, that the current physical address in the output file (denoted by '.') should be set to KERNEL_BOOT_VMA, which we defined as the start of the kernel.

The next thing tells the linker how to handle the ''code'' from our boot.S file (but not the data!):

    .boot :{ 
        */boot.o (.text)
    }  

This creates a section called .boot in the output file. The linker puts into this section the contents of all .text section from all input files ''whos name ends in '/boot.o''' (This means that passing our boot.o to the linker as 'boot.o' doesn't work, we'll need './boot.o' or some other ''path'' to the file). We'll only have one single of those objects, so thats cool with us :)

Also the above make the linker assign addresses from KERNEL_BOOT_VMA onwards to the .text symbols from boot.o, so we're linked to lower half here.

'''Note:''' it would have been cool to have the PD and PT structures in the .boot section too, so we don't have to mess with KERNEL_HIGH_VMA offsets in the code all the time. However all i could come up with failed, so i had to put them into the .data section (and i tried a lot...)

The next instructions tell the linker, to set the current offset from which addresses are assigned to the higher half, which means, from here on, all symbols will be linked to higher half. Also, a symbol (_core_start) is assigned the current address. This symbol can be used from the C code if required.

    . += KERNEL_HIGH_VMA;
    _core_start = .;

Now, let's define the rest of the sections. First, we'll need a .text section, containing all the kernel code. The section is aligned on a page boundary, and the ''physical'' address is set to lower half, by subtracting the higher half offset again.

    .text ALIGN(0x1000) : AT(ADDR(.text) - KERNEL_HIGH_VMA) {
        _core_code = .;
        *(EXCLUDE_FILE (*/boot.o) .text)
 
        /* all readonly data is merged with this section, too */
        *(.rodata*)
    }

One more noteworthy thing from above is, that the 'boot.o' file is now ''excluded'' from the .text section, as it's .text contents is already in the .boot section, linked to lower half.

The next section to define, is the .data section, which contains read/write data symbols.

    .data ALIGN (0x1000) : AT(ADDR(.data) - KERNEL_HIGH_VMA) {
        _core_data = .;
        *(.data)
    }

This should be pretty clear now, given what we know from above. The section is page aligned, and the physical load address is set to the lower half. A symbol is declared, denoting the start of the section, and all .data sections from all input files (also 'boot.o'!) are included here.

The last section to define is the .bss section. It will also contain common symbols, if there are some (currently there should not we any; one can create those with the .comm and .lcomm gnu assembler pseudo directives):

    .bss ALIGN (0x1000) : AT(ADDR(.bss) - KERNEL_HIGH_VMA) {
        _core_bss = .;
        *(COMMON)
        *(.bss)
        . = ALIGN(4096);
        _core_ebss = .;
    }

Again, page aligned, and the address is adjusted to lower half. Multiple input sections are merged into this one output section (COMMON is special and means all symbols that are marked *COM* if you look at a objdump -x output for a given object file).

This section is padded until the current page is full, and an "end-symbol" is declared. this is, so that we could be able to clear the .bss section (but with elf, this is not necessary, as the bootloader has to do this already for us...).

finally, another symbol is declared to denote the end of the kernel. We use this symbol in the code to determine the size of the kernel, and thus the amount of memory we need to map for the kernel to work.

    /* 4K alignment is guaranteed! */
    _core_end = .;
 }

That's it. You should be able to compile and link the boot code now.

= Compile and link =

You should be able to compile and link the kernel like this:

gcc -Wall -Werror -Wextra -ffreestanding -fno-rtti -fno-exceptions  -O0 -g -c -o boot.o boot.S
ld -T link.ld  -o kernel ./boot.o

= Running =

I suggest using qemu:

qemu -kernel kernel

if things worked out, you should see "PANIC!" on the screen now! congratulations, paging worked. If nothing seems to happen, you did something wrong, and if the screen constantly flickers, you have done somethings wrong too (and the machine triple faults and resets continuously). I hope though, that there are no errors in the code i showed here.

= Thanks =

Thanks for reading, and '''Good luck!'''
````

<a id="w002945"></a>

## W002945 — Volatile

Bron-ID: W002945
Namespace: Main (0)
Revisie: 11034 | Datum: 2010-12-06T19:26:00Z
Bron: https://wiki.osdev.org/Volatile
Revisie-URL: https://wiki.osdev.org/index.php?title=Volatile&oldid=11034
Doorverwijzing: Volatile (keyword)

````text
#REDIRECT [[Volatile (keyword)]]
````

<a id="w002946"></a>

## W002946 — Acpi

Bron-ID: W002946
Namespace: Main (0)
Revisie: 11054 | Datum: 2010-12-20T23:04:19Z
Bron: https://wiki.osdev.org/Acpi
Revisie-URL: https://wiki.osdev.org/index.php?title=Acpi&oldid=11054
Doorverwijzing: ACPI

````text
#REDIRECT [[ACPI]]
````

<a id="w002947"></a>

## W002947 — Hexeditor

Bron-ID: W002947
Namespace: Main (0)
Revisie: 28470 | Datum: 2023-11-16T15:47:35Z
Bron: https://wiki.osdev.org/Hexeditor
Revisie-URL: https://wiki.osdev.org/index.php?title=Hexeditor&oldid=28470

````text
== Purpose ==

The purpose of a hex editor is to be able to view a page in hexadecimal instead of standard [[ASCII]]. This is the main way to be able to create a binary without needing to type extended ascii, or program at all. Hex editors are also used to convert decimal to hexadecimal vice versa.

Counting in hexadecimal is different from counting in decimal.

Decimal: 1  2  3  4  5  6  7  8  9  10  11  12  13  14  15  16  17  18  19  20  21  22  23  24  25  26  27  28  29  30 31<br>                                                                                                                                                                                                               Hexadecimal: 1  2  3  4  5  6  7  8  9  A    B   C    D    E    F     10  11  12  13  14  15  16  17  18 19  1A  1B  1C  1D  1E 1F

== Some Hex Editors ==

[http://web.archive.org/web/20140706010326/http://www.physics.ohio-state.edu/~prewett/hexedit/ HexEdit] - hex editor for Windows <br>
[http://www.mh-nexus.de/en/hxd HxD] - freeware hex editor by Maël Hörz. <br>
[http://hexfiend.com Hex Fiend] - open source hex editor for MacOS <br>

[[Category:Tools]]
````

<a id="w002948"></a>

## W002948 — User:Derekbank/C Kernel

Bron-ID: W002948
Namespace: User (2)
Revisie: 11083 | Datum: 2011-01-07T12:17:40Z
Bron: https://wiki.osdev.org/User:Derekbank/C_Kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADerekbank%2FC_Kernel&oldid=11083

````text
=== Prerequisites ===
You will need to have the GRUB files stage1 & stage2. Download [ftp://alpha.gnu.org/gnu/grub/grub-0.97-i386-pc.tar.gz GRUB] and copy these files into your os's directory. You will also need nasm, gcc, ld, and qemu.
<b>NOTE: Examples are written for linux. </b>

===Source Code===
Loader.asm:
<source lang=asm>
global loader
extern main

MODULE_ALIGN equ 1<<0
MEM_INFO equ 1<<1
FLAGS equ MODULE_ALIGN | MEM_INFO
MAGIC equ 0x1BADB002
CHECKSUM equ -(MAGIC + FLAGS)

section .text
align 4
MultiBootHeader:
dd MAGIC
dd FLAGS
dd CHECKSUM

STACKSIZE equ 0x4000

loader:
call main
cli
hang:
hlt
jmp hang
section .bss
align 4
stack:
resb STACKSIZE
</source>
This code basically sets up GRUB and jumps to the kernel

Now for the actually kernel.
Kernel.C:
<source lang=c>
void main(){
    unsigned char* video = (unsigned char*) 0xB8000;
    video[0] = 'O';
    video[1] = 0x07;
    video[2] = 'S';
    video[3] = 0x07;
}
</source>
This simple kernel writes 'OS' to the screen. It should be self-examplatory, except for 0x07. That tells the graphics card the it will be written in white on black and doesn't blink.

Now we need to a linker script
Link.ld:
<source lang=c>
ENTRY (loader)

SECTIONS{
  . = 0x00100000;
  .text :{
    *(.text)
  }

  .rodata ALIGN (0x1000) : {
    *(.rodata)
  }

  .data ALIGN (0x1000) : {
    *(.data)
  }

  .bss : {
    sbss = .;
    *(COMMON)
    *(.bss)
    ebss = .;
  }
}
</source>

Last but not least, the build script.
Build.sh:
<source lang=c>
nasm -f elf -o loader.o Loader.asm
gcc -o kernel.o -c Kernel.C -Wall -Wextra -nostdlib
ld -T Link.ld -o os.bin loader.o kernel.o
cat stage1 stage2 pad os.bin > image.img
</source>

===Building and Booting===
There is one file that is still missing, pad. Use the linux command "dd if=/dev/zero of=pad bs=1 count=750" to create it. (without the quotation marks) Type the command chmod 777 Build.sh Then type ./Build.sh This should take a moment. Debug any errors occurred. Now to boot it, type qemu -fda image.img It will show some grub stuff. Type in kernel 200+18 <enter> boot <enter> <br> <br>
Voila, the letters O and S are printed at the top.
````

<a id="w002949"></a>

## W002949 — User:Kenny

Bron-ID: W002949
Namespace: User (2)
Revisie: 14523 | Datum: 2013-03-10T12:07:03Z
Bron: https://wiki.osdev.org/User:Kenny
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKenny&oldid=14523

````text
[[Colour Quantisation]]

[[Dithering]]

[[User:Kenny/Test]]


== Command Line Interface ==

The Command Line Interface of an operating system provides the most basic, but very powerful method of controlling the system.  When writing an operating system, you should consider writing a command line interface after you have have a stable kernel, can output to the screen, and can read from the keyboard.  Advanced topics such as multi-tasking and memory management aren't essential.

Your first command line interface is likely to be a function which can read the keyboard, compare the entered string to a list of known commands, and run another function based on the entry.

For a slightly more advanced command line interface, you will probably want to use the common structure of command line interfaces which is split into two components, the console, and the command interpreter (also called the shell) itself.

=== Console ===

The console, or terminal, is the environment which runs command line programs.  It provides a standard input stream, a standard output stream, a standard error stream, and a set of environment variables.

In a basic system, the standard input device is the keyboard, and the standard output and error streams appear as text on the screen.

In C from the point of view of a command line program, the standard input stream (referred to as stdin), the standard output stream (stdout) and the standard error stream (stderr) are FILE structures.  The stdin stream is read using the fread() function, and the stdout and stderr streams are written to using fwrite() or fprintf().  These operations allow a command line program to interact with the user.

More complicated systems can use any character device for a console's input and output, for example a terminal could be opened using a serial port so that a serially connected terminal could run commands, a command could have its output sent to a file or a printer, or the output could be sent to a null device (discarded).

=== Command Interpreter ===

All command line tools operate in very much the same way.  They expect to be provided with a standard input stream, a standard output stream, a standard error stream, and a set of environment variables.  Each tool reads characters from the standard input (usually the keyboard), performs some task, outputs information to the standard output or standard error streams (usually the screen), then exits.

The command interpreter itself is no different.  When run, it goes into a loop where it outputs a prompt to stdout then waits for a command to be entered on stdin, it then runs the command, and goes back to display the next prompt.  It does this until it's given a command to exit.

Because a command interpreter behaves exactly like any other command line tool, it is possible to run one command interpreter from within another.  The second interpreter will take control until it is exited, at which point the user will return to the first command interpreter just as if they had run any other command.

Many Unix-based systems offer a selection of command interpreters, such as sh, ash, bash, ksh

Many command interpreters offer additional features, such as the ability to review and repeat previous commands, or to auto-complete file names.

==== Redirection ====

Most command line interpreters feature the ability to redirect the stdin, stdout and stderr streams for a given command.

From the user's point of view, they specify additional command line arguments with the "<" or ">" character followed by the name of a device or file, the command line interpreter then read input from or writes output (stdout) to that location.

From the point of view of the command interpreter, it detects the special redirection argument and attempts to open the location specified by using fopen().  This then creates a FILE structure that the command line interpreter passes to the invoked command as stdin or stdout as appropriate.  The command then reads these as normal as has no idea ( and doesn't care ) that the input or output is being redirected.

=== Piping Commands ===

A logical extension of redirection is the concept of piping.  This is taking the output from one command and feeding to a subsequent command as its input.

The user specifies the commands in the order they are to be executed and puts a pipe symbol "|" between each.  For example, the user can write "ls | more" to run the ls command and provide it's output to the more command.  The benefit to the user in this example is that the more command will display the information one screen at a time and wait for the user to review each before proceeding.

From the command interpreter's side, it creates an temporary buffer which can be written or read through a FILE structure, and passes it as the stdout of the first command.  The first command runs and pushes the data into the buffer.  Once the first command finishes, the command interpreter rewinds the temporary buffer and passes it as the stdin to the next command.  If necessary, the command interpreter will have created a second buffer to capture the output of the second command to pipe to a third command.

In some command interpreters, such as PowerShell, the input and output of commands isn't limited to data being written to a stream, but instead can contain complete programming objects.

== Writing a Command Line Interface ==

As mentioned, a command line interface will need the following components:
*Enough of the standard C library to implement
**the fread() function
**the fwrite() function
**the FILE structure
**(optional) the fprintf() function
*The Console component, comprising
**a FILE structure instance representing the keyboard input, supporting converting a key-press to a character and buffering it.
**a FILE structure instance representing the screen output, supporting writing a character, deleting a character, and moving to a new line.
*The Command interpreter, comprising
**a buffer to store the input until the user presses enter
**A lookup table or other list of commands to be compared the entered string.


== TODO ==

*Comment on how changing the terminal capabilities may affect programs (changing font size, resizing window etc.) such as more.
*Comment on how programs may want to influence the terminal capabilities (e.g. resize the window).
*Comment on special characters that can affect the position of the cursor or the colour of the text etc. (escape sequences).

== Further Reading ==

[http://en.wikipedia.org/wiki/Unix_shell Wikipedia entry for Unix Shell]  Gives background information on command line interpreters in general.
````

<a id="w002950"></a>

## W002950 — Colour Quantisation

Bron-ID: W002950
Namespace: Main (0)
Revisie: 16268 | Datum: 2014-04-24T05:31:15Z
Bron: https://wiki.osdev.org/Colour_Quantisation
Revisie-URL: https://wiki.osdev.org/index.php?title=Colour_Quantisation&oldid=16268

````text
{{Stub}}
[[Category:Graphical UI]]

In a graphical OS environment, there will come a time when you need to represent an image on a display device.  If the display device is unable to represent all the colours from the image, you will have to have a mechanism of mapping a colour from the image onto the closest displayable colour from the device palette.  The general term for such a colour mapping is Colour Quantisation.  The Colour Quantiser is the section of a system responsible for performing this action.

For the purposes of this article, the source image is considered to be a bitmapped image with 24 bits per pixel, eights bits for each of the red, green and blue components.  This is commonly the worst case scenario (outside of 48-bit photography processing) and any other scenario can be expressed as this.

While Colour Quantisation on its own can be used to produce a reasonable representation of the image, most applications may want to consider applying [[Dithering]] to the image to produce a more accurate overall reproduction.

The term Colour Quantisation also refers to the process of calculating a low-colour palette from an image that preserves much of the colour detail, but that usage is not discussed here.

''Disclaimer:  This article is not intended to be the final word nor give you the best solution for this issue, but rather to introduce the concept and hopefully provide something that will work in most cases.  The methods presented here should work to get your project over this hurdle in the short term to then be replaced and improved upon in future.''

== Non-paletted Quantisation ==

If your display device is a non-paletted type but it cannot display all of the required colours, for example a 15-bit display with only 5 pixels per channel, colour quantisation is straightforward, but you may still want to consider [[Dithering]] the image.

=== Colour Contraction ===

Trivially, to map a 24-bit colour onto a smaller colour space, logic and bit-shifting can be used to preserve the most significant bits in each channel from the source colour and compress them into a colour in the destimation space.

For example, given a 24-bit colour with the bits in the following form
    { R0, R1, R2, R3, R4, R5, R6, R7, G0, G1, G2, G3, G4, G5, G6, G7, B0, B1, B2, B3, B4, B5, B6, B7 }
(where R0 is the most significant bit in the eight bit Red component and R7 is the least significant bit), this can be made into a 15-bit colour by removing the least significant three bits from each channel, giving the following
    { R0, R1, R2, R3, R4, G0, G1, G2, G3, G4, B0, B1, B2, B3, B4 }.

Consideration could be given to the value of the sixth bit to be used to "round" the resultant value up or down as required, but this is unlikely to significantly affect the image quality.

=== Colour Extension ===

A low bit colour can be converted into a high bit colour in a similar manner to the above by adding in additonal low-significance bits.  For example the 15-bit value
    { R0, R1, R2, R3, R4, G0, G1, G2, G3, G4, B0, B1, B2, B3, B4 }
could be converted into a 24-bit colour by inserting three zero bits into each channel, yielding
    { R0, R1, R2, R3, R4, 0, 0, 0, G0, G1, G2, G3, G4, 0, 0, 0, B0, B1, B2, B3, B4, 0, 0, 0 }.

== Paletted Quantisation ==

Colour Quantisation for a destination palette is by far the most useful application of this technique.  Any one of 16 million possible colours may have to be processed and displayed as a palette colour in order to recreate the image as accurately as possible.  If only a small number of colours is available, such as the 16 colours that are available with a VGA Mode 12h screen, a good method for colour quantisation is essential.  The various techniques can be extended to produce greyscale or even halftone (black and white) representations.

=== Euclidean Distance ===

The most basic method of colour quantistion is the Euclidean Distance technique.  For each colour pixel in the image in turn, Euclid's formula (an extension of Pythagoras' theorem) can be applied to each colour in the palette in turn to determine which of the paletted colours is closest to the desired colour.  This palette colour is then used to represent the pixel.

In pseudo code, the algorithm is as follows:
    Define C as the source colour
    Define ND as the nearest distance so far
    Define NP as the nearest palette entry so far
    
    Set ND to a large value
    For Each Palette Entry P
    	Calculate D as Square of ( C.R - P.R ) plus Square of ( C.G - P.G ) plus Square of ( C.B - P.B )
    	If D is less than ND
    		Set ND = D
    		Set NP = P
    	End If
    Next
    Use NP as the result

While the implementation of this algorithm is very straightforward, the algorithm has to compare each pixel in the source image to each colour in the destination palette in turn using three squareing operators, meaning that it is very slow in operation.  (Note that the square root operator called for by the Euclidean algorithm can be omitted because it will not affect the result.)

The other main problem with using the Euclidean Distance is that it considers the Red-Green-Blue colour space to be a regular cube with a linear distribution, but this isn't the case.  Consider the Euclidean Distance algorithm being used to represent colours as a greyscale, a pure green and a pure blue will resolve to the same shade of grey, despite green being approximately seven times lighter than blue.  A keen implementer may want to modify this algorithm to take account of the variance in brightness.

=== Pre-calculated map ===

A logical improvement of the Euclidean Distance method above is to pre-calculate a table of the nearest palette colour for a set of possible source colours.  This table can be calculated whenever the display palette is set or changed, and provides a fast lookup for the colour quantiser to significantly improve performance.

Each index of the lookup table will represent a point on a regular grid within the RGB colour cube, and the corresponding value will store the index of the nearest displayable palette entry.  The more points in the grid that are calculated will improve the accuracy of the quantisation, but will take more memory to store and more processing time to calculate.

The extreme case of this would be to use a grid of point that represents every possible combination of the 24-bit colourspace.  Doing this would require 16 million array elements (16Mb at one byte per palette index) and would take the same time to calculate as using the Euclidean algorithm to quantise a 4096x4096 pixel image.  At the other extreme, using too few elements will result in an inaccurate mapping.

The following table details the various grid size options and the resultant table size:
{| {{wikitable}}
! Bits per channel
! Mapping table entries
! Calculated entries per Palette entry
! Comment
|-
| 1 || 8 || 0.5 || Too few to be usable, doesn't even allowing mapping to each palette colour.
|-
| 2 || 64 || 4 || Minimum reasonable table size, will work as a quick procedure.
|-
| 3 || 512 || 32 || Reasonable table.  Provides good mapping in little memory.
|-
| 4 || 4096 || 256 || Very accurate table.  Neatly fits into one page of memory.
|-
| 5 || 32768 || 2048 || Unreasonably accurate table for 16 colours.  Uses eight pages.
|-
| 8 || 16777216 || 1048576 || Excessivly accurate table for 16 colours.  Uses 16Mb.
|}

From this table, it would seem that storing 3 bits per channel would seem to be a good starting point for the 16-colour mode 12h screen.

As each index entry in the table represents a point within the RGB colourspace, calculating the colour mapping table is straightforward:
    Allocate the Colour Table
    Loop the number of entries in the Colour table
    	Calculate the colour in the grid that corresponds to this array index
    	Use the Euclidean Distance calculation to find the nearest palette entry to the colour
    	Store the Palette entry into the corresponding array element

Once the table is calculated, it can be stored and will not need to be recalculated unless one or more palette entries is changed.

When the quantiser is needed to represent an image on the display device, the following algorithm is applied:
    Loop each pixel in the source image
    	Calculate the equivalent colour in the grid from the source colour by colour contraction (mentioned above)
    	Access the colour table using the grid colour as the index
    	Use the array element contents as the palette entry for that colour.

==See Also==
=== External Links ===
*[http://en.wikipedia.org/wiki/Color_quantization Wikipedia page on the subject]
````

<a id="w002951"></a>

## W002951 — Dithering

Bron-ID: W002951
Namespace: Main (0)
Revisie: 11089 | Datum: 2011-01-09T16:56:30Z
Bron: https://wiki.osdev.org/Dithering
Revisie-URL: https://wiki.osdev.org/index.php?title=Dithering&oldid=11089

````text
{{Stub}}
[[Category:Graphical UI]]
[[Category:Common Algorithms]]
Dithering is the process of modifying the colour of various pixels in an image to provide a better reproduction of the source image.  It is used commonly when displaying an image on a display that is incapable of representing all the required colours.  Dithering is commonly used alongside the process of [[Colour Quantisation]], and an understanding of that topic is recommended before continuing through this article.

The Wikipedia article on Dithering is very good and gives visual examples of each of the method of dithering.  This article provides a slightly different viewpoint and focuses on elements that may be of use to OS developers.

Note that Dithering and Anti-aliasing are opposite functions.  Dithering uses spatial resolution to make up for a lack of colours.  Anti-aliasing uses colours to make up for a lack of spatial resolution.

== Error Diffusion ==

The most effective Dithering techniques for use on raster (pixel-based) displays work by Error Diffusion, that is that they compute the nearest displayable colour for a pixel (by [[Colour Quantisation]]) and then calculate the difference between what that pixel should have been and what it was.  This calculated difference (the "error") is then distributed to neighbouring pixels and taken into account when they are quantised.  The error can accumulate across multiple pixels until becomes significant enough to alter the result of the quantisation.

For example, a dithering algorithm may be used to draw a grey line on a black and white display.  In this instance, the dithering algorithm will likely draw alternating black and white pixels so that the overall effect is a grey line.

=== Basic Form ===

The basic form of error diffusion is to take the error from the first pixel and apply it to the next pixel to the right.  This has the benefit of being very easy to implement, as the code only has to keep track of one pixel worth of error, but it produces a rather crude result.

In pseudo-code, this can be implemented as the following:
    Define E as the carried error
    For Each Source Pixel As P
    	Define Colour C as { P.R - E.R, P.G - E.G, P.B - E.B }
    	Calculate the Nearest colour to C as NC using the Colour Quantiser
    	Display the pixel as NC on the screen
    	Set E to be ( NC.R - C.R, NC.G - C.G, NC.B - C.B }
    Next

Because we are taking the error from a pixel and applying it entirely to the pixel to the right, we could represent this use the following matrix kernel (no, the other kind):

{| {{wikitable}}
|-
| 0 || 0 || 1
|}

The zero in the centre of the matrix represents the current pixel, and the one to the right indicates that the entire error (1 times the error) is applied to the pixel on the right.

This is the standard way of expressing an error diffusion dithering.  All error diffusion algorithms will appear like this with the current pixel in the centre, and fractions surrounding it to indicate to which pixels the error is to be diffused.  Also of note is that any pixels above or immediately to the left of the centre will be zero as all the algorithms consider that these pixels have already been drawn.  This means that dithering can be applied to pixels sequentially.

=== Floyd–Steinberg Dithering ===

This is a more effective and more common dithering algorithm.  It takes the error from the first pixel and divides it up between four neighbouring pixels using a the following kernel:

{| {{wikitable}}
|-
| 0 || 0 || 0
|-
| 0 || 0 || 7/16
|-
| 3/16 || 5/16 || 1/16
|}

As before, the zero in the centre represents the pixel being processed, and the fractions below and to the right indicate what proportion of the error is accumulated into the corresponding pixel.  As remarked above, the error is diffused only to pixels to the right or below so that the routine can be applied sequentially to an image.

This method is very effective and can render high colour images on displays with as few as 8 colours.

This algorithm is slightly more complicated to implement than the basic form, this is because the routine needs to store two rows worth of errors in order to correctly calculate the effective colour for each pixel.  Depending on the application, the dithering process could modify the pixels further ahead in the source image itself to avoid having to store the diffused error data separately.

== Other Dithering types ==

As well as Error Diffusion, there are other dithering types available, some of which are better suited to media other than a raster (pixel-based) screen.  These are considered beyond the scope of this article.  For more information, refer the wikipedia article.

== External Links ==
*[http://en.wikipedia.org/wiki/Dither#Digital_photography_and_image_processing Wikipedia page on the subject]
````

<a id="w002952"></a>

## W002952 — User talk:Derekbank/C Kernel

Bron-ID: W002952
Namespace: User talk (3)
Revisie: 11091 | Datum: 2011-01-11T12:38:22Z
Bron: https://wiki.osdev.org/User_talk:Derekbank/C_Kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ADerekbank%2FC_Kernel&oldid=11091

````text
==Redundancy==
Do we need a clone of the C [[Bare Bones]]? I can't find anything that this tutorial provides over that. - [[User:Combuster|Combuster]] 09:40, 7 January 2011 (UTC)

I agree. If the user wants to create his own tutorial series, he can per-user articles (I don't know if they have any special name). --[[User:Love4boobies|Love4boobies]] 12:08, 7 January 2011 (UTC)
:Moved to the author's userspace for now, so he can address the mentioned concerns. - [[User:Combuster|Combuster]] 12:21, 7 January 2011 (UTC)

== What it does and does not do... ==

What this "tutorial" does ''not'' do is:
* explaining ''anything'' it does (effectively being [[Bare Bones]] stripped of all comments and explanations, while adding nothing);
* actually seting up the stack before using ''call'';
* determining the necessary "pad" size dynamically.

What it ''does'' do is:
* suggesting that ''build.sh'' is a viable way of automating your build;
* using uppercase file naming, which is at the very least unconventional in SW development.

I do not see what it adds to the Wiki that wasn't already there. -- [[User:Solar|Solar]] 12:38, 11 January 2011 (UTC)
````

<a id="w002953"></a>

## W002953 — Tutorial:C Kernel

Bron-ID: W002953
Namespace: Main (0)
Revisie: 11084 | Datum: 2011-01-07T12:17:40Z
Bron: https://wiki.osdev.org/Tutorial:C_Kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Tutorial%3AC_Kernel&oldid=11084
Doorverwijzing: User:Derekbank/C Kernel

````text
#REDIRECT [[User:Derekbank/C Kernel]]
````

<a id="w002954"></a>

## W002954 — Talk:Tutorial:C Kernel

Bron-ID: W002954
Namespace: Talk (1)
Revisie: 11086 | Datum: 2011-01-07T12:17:41Z
Bron: https://wiki.osdev.org/Talk:Tutorial:C_Kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ATutorial%3AC_Kernel&oldid=11086
Doorverwijzing: User talk:Derekbank/C Kernel

````text
#REDIRECT [[User talk:Derekbank/C Kernel]]
````

<a id="w002955"></a>

## W002955 — User talk:Derekbank

Bron-ID: W002955
Namespace: User talk (3)
Revisie: 11092 | Datum: 2011-01-11T12:46:09Z
Bron: https://wiki.osdev.org/User_talk:Derekbank
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ADerekbank&oldid=11092

````text
Hello Derekbank,

you have not yet posted anything to the forum, and your Wiki contributions amount to two articles of, let's say, "questionable value". I would suggest you take a more defensive approach to Wiki editing, and address changes and additions you feel are necessary in the [http://forum.osdev.org/ forum] first (there's an "OSDev Wiki" subforum), at least until you have familiarized yourself with a subject a bit more. Just sayin', no offense intended. -- [[User:Solar|Solar]] 12:46, 11 January 2011 (UTC)
````

<a id="w002956"></a>

## W002956 — Apic

Bron-ID: W002956
Namespace: Main (0)
Revisie: 11095 | Datum: 2011-01-11T13:37:46Z
Bron: https://wiki.osdev.org/Apic
Revisie-URL: https://wiki.osdev.org/index.php?title=Apic&oldid=11095
Doorverwijzing: APIC

````text
#REDIRECT [[APIC]]
````

<a id="w002957"></a>

## W002957 — Hierarchical VFS Theory

Bron-ID: W002957
Namespace: Main (0)
Revisie: 27996 | Datum: 2023-07-09T17:40:12Z
Bron: https://wiki.osdev.org/Hierarchical_VFS_Theory
Revisie-URL: https://wiki.osdev.org/index.php?title=Hierarchical_VFS_Theory&oldid=27996

````text
===Purpose of this article===

This article was written in order to explain the basic concepts behind a hierarchical VFS. Articles about tag-based VFSs and DB-VFSs are proposed for a later date. This article does not seek to explain how to code or implement a VFS: it explains the structure and justification behind conventional VFS design. Do not look here for "code examples".

===What is a hierarchical VFS?===

In contrast with a tag-based VFS or a DB-VFS, a hierarchical VFS is concerned with caching folder-hierarchies on a set of concrete filesystems. A tag-based VFS would for example, be a tag indexer which would be most efficiently implemented (for a UTF-8 based kernel) with an array of 256 "top layer" character indexes, with each of those array indexed pointing to a further array of 256 indexes. You would then parse tag addresses (as opposed to file paths) by quickly indexing into the top level array using the first letter of the tag, then into the sub array with the second letter of the tag. From there, you would have a B-tree under each sub-index which points to a list of all tags with the same two first letters.

Tag based VFSs are generally not concerned with hierarchy, but intersection. So a tag-based address would be similar to: "Shaniqua:music:Jazz". Those three tags may individually have about 100 files each in their listings; but the intersection of each listing set incrementally will give the sum of the files which pertain to all three. An empty set is taken to be homologous to an "empty folder" in a hierarchical file-system.

Hierarchical file-systems do not work by intersection, but by iterative sub-levelling. So for a hierarchical VFS, your aim would be to quickly index paths and their subfiles by name. This would make for easy parsing of hierarchical filesystem paths. A tree-structure is usually taken to be the most efficient method of handling a hierarchical VFS.

===Conventional tree caching format===

Keep in mind always that a VFS is nothing more than a structural cache of the layout of the underlying concrete filesystems mounted on it; You cache the folder hierarchy as you go deeper into it.

The main distinction that must be made when designing VFS tree caching is the one between tnodes and inodes, where "t-node" stands for "tag-node" and "i-node" stands for "info-node". In conventional UNIX kernels (and in NT as well, though the facility is used by the kernel and a software based ".lnk" format is employed for userspace), a single filesystem node may point to the same file <b>data</b> as another file system node. That is, you may have two, three, four, or more "files", all which may be in different folders, which would show up in a directory listing of their respective folders, which may point to the same file data. Writing to the data from one of those files' perspective would produce a change in contents from the other files' point of view since they all point to the same data.

These are called file-system <i>links</i>. That is, file-data is to be treated separately from a tag-node which points to it. A tag-node gives collection of bytes (file data) a name. An info-node <i>describes</i> file data. A tag-node does nothing more than assign a name and point to the i-node which describes the data to which the file-name is associated.

Info nodes are then expected to state information such as: creation date of the file, last modification date, last access date, size of the file data, and any unique ID number which identifies that file on the underlying concrete file-system. Please keep starkly in mind that virtual file systems are not the only ones that use the t-node/i-node arrangement: many concrete file systems do as well. As a result, hereafter, this article will refer to VFS inodes as "VFS inodes" and to inodes on a concrete filesystem as "concrete inodes".

===Tag node cache===

Whenever the VFS parses a file path, it sequentially descends down the tree from the VFS root, looking at sub-folder names until it reaches the last part of the file path; at that point, it will check to see if it finds the last component of the file path as a subfolder. If it turns out not to be a sub-folder, then the VFS will see if it's a sub-file. If that fails, or if the descent failed at an earlier conponent, then the VFS assumes the path to be invalid.

A corollary of this is that the VFS must be able to quickly descend from the root node, through a list of t-nodes which give the file-name of each sub-file or folder.

A process of logic reveals that: A file, or folder t-node only contains the name of the file or folder, and a pointer to the inode for that file or folder. Where then, would the VFS cache file-listings for a cached folder? In that folder's VFS inode. Specifically, the folder t-node would point to a folder VFS inode. To search the the subfolders or subfiles of a folder t-node, you would dereference the t-node's VFs inode pointer, and read the list of sub-files and folders from that VFS inode.

What is the list of files and folders really, then? It is a list of t-nodes. So:

t-node -> inode with sub-file and folder list (which is really just a list of more tnodes) -> search tnodes -> find next path component match -> tnode -> inode with subfile and folder list (which are tnodes) -> search tnodes -> find next path component -> tnode -> and so on and so forth.

That is essentially the structure of a hierarchical VFS.

<pre>
Folder or file t-node:
+-----------------------------------------------------------------------------+-----------------------+
| <file or folder name>         ...                                           | pointer to inode      |
+-----------------------------------------------------------------------------+-----------------------+

Folder i-node:
+-------------------------------------------+
| Pointer to list of tnodes for sub-folders |
+-------------------------------------------+
| Pointer to list of tnodes for sub-files   |
+-------------------------------------------+
| int  num_subfolders; | int num_subfiles   |
+-------------------------------------------+
| concrete FS inode number                  |
+-------------------------------------------+

File i-node:
+-------------------------------------------+
| Creation date | Date of last modification |
+-------------------------------------------+
| File size                                 |
+-------------------------------------------+
| [Optional] Lock | [Optional] State flags  |
+-------------------------------------------+
| Concrete FS inode number                  |
+-------------------------------------------+
| Other things you think are necessary      |
+-------------------------------------------+
</pre>

===Support for file-system links===

Two unique t-nodes in the tree which point to the same inode are said to be "links" to that inode, and by extension, will cause a read or write to that t-node (file-name) to update the same file. There is no extra work required: just treat them as t-nodes which point to the same inode. You can also set flags to identify them as links, although it makes no real difference to the VFS proper. The only thing you need to note about links is that even though your VFS unifies all mounted concrete file systems under one tree, links can only be local to a single concrete file-system: this is only common sense. Remember, links point to the same inode. If there is a link on one concrete file-system which points to an inode on another concrete filesystem (in other words, on another disk), how can it then be pointing to the same file data?

===Thoughts on linking the Inode-based VFS to concrete hierarchical filesystems which do not use an inode structure===

Most recent concrete filesystems use an inode based approach to identifying file data within their represented disk. Each unique entity of data set is given a unique inode ID within that disk's filesystem. So every file's data is uniquely identified on that disk by means of its filesystem's unique inode as given to that file's data. If you wanted to create a generic filesystem driver interface that simplified file system interfacing for all filesystems, and you chose to use the "inode" concept, you would have no problem doing so with for example, the EXT class of filesystems, which use inodes.

When your kernel requests a folder listing, the filesystem driver would return tag-nodes for the names of all sub-files and folders, and inodes that are related to each tag node, and you would ensure you stored the concrete filesystem's inode ID for that file as given on the concrete filesystem; If later on, the user renamed a file within a folder, you would simply find that folder in your cached VFS hierarchy, and tell the filesystem driver to rename the file named "foo" on folder inode X within that driver's represented filesystem to "bar", as the user requested: there is a direct mapping between your VFS and the inodes on the underlying filesystem.

What if you wish to write a driver for your kernel for say, one of the FAT filesystems? FAT does not use inodes to uniquely identify file data. If you were to pass a request for an FAT driver to rename tag node "foo" on folder inode X to "bar", you'd fail naturally since first of all, the FAT driver would not have returned a concrete FS inode ID to you when it was giving you that folder's file listing.

A plausible solution would be to force all drivers to ''create'' file data -> inode mappings. So, in such a case, when the driver is initialized, it initializes a counter from which it will pull fake inodes to hand the kernel when it gives over file listings. It will just pass these out on the fly, creating new inodes as it gives the kernel new file listings, and it would maintain a table of the mappings between the inodes it handed out, and whatever format it uses to uniquely identify file data. It would of course, ensure that these inode IDs it hands out on the fly would be unique across mounts.

In that case, when the kernel requests a file listing, of a folder, the driver would pass the kernel the folder's inode information, and assign it, as it gives it over, a new inode ID and record this in a table. Later on, the kernel, unaware that there is translation work being done behind the scenes in the driver would pass a request for file "foo" on inode X (an inode the kernel does not know has been created on the fly) to be renamed to "bar".

The driver now looks at the inode mapping table, and sees that it assigned file data group Y the inode ID X when it was passing it to the kernel. The filesystem will now go and rename the file as appropriate, and things go on as usual, with the kernel unaware of that mapping activity beign done in the driver to make it conform to your driver interface which requires inodes to uniquely identify files within a single conrete filsystem.

[[Category:Filesystems]]
[[Category:OS theory]]
````

<a id="w002958"></a>

## W002958 — Talk:Joliet

Bron-ID: W002958
Namespace: Talk (1)
Revisie: 11099 | Datum: 2011-01-16T12:41:09Z
Bron: https://wiki.osdev.org/Talk:Joliet
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AJoliet&oldid=11099

````text
Link to specification is dead, I dont know the original link, but perhaps this is a better one: http://pierrelib.pagesperso-orange.fr/filesystems/joliet_spec_v1.html --[[User:Hidnplayr|Hidnplayr]] 12:41, 16 January 2011 (UTC)
````

<a id="w002959"></a>

## W002959 — User:Nedbrek/mmap

Bron-ID: W002959
Namespace: User (2)
Revisie: 11121 | Datum: 2011-01-23T01:52:40Z
Bron: https://wiki.osdev.org/User:Nedbrek/mmap
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANedbrek%2Fmmap&oldid=11121

````text
In the constrained environment of assembly language programming, it is important to have a clear picture of where everything is in memory:
{| {{wikitable}}
|-
! Range
! Area
|-
| 0_0000..0_03FF
| Real mode IDT (reclaimable)
|-
| 0_0400..0_04FF
| BIOS low data (reclaimable)
|-
| 0_0500..0_7BFF
| boot stack (30,464 bytes)
|-
| 0_7C00..0_7DFF
| boot sector (512 bytes)
|-
| 0_7E00..3_FFFF
| available
|-
| 4_0000..4_FCFF
| boot output block (BOB)
|-
| 4_FD00..4_FFFF
| video config scratchpad (reclaimable)
|-
| 5_0000..5_FFFF
| GDT+IDT
|-
| 6_0000..6_FFFF
| stage 1 page tables
|-
| 7_0000..7_FFFF
| output of BIOS memory map (INT 15/E820)
|-
| 8_0000..9_FBFF
| possible BIOS EDA
|-
| 9_FC00..9_FFFF
| definite BIOS EDA
|-
| A_0000..F_FFFF
| ROM
|}

<source lang="c">
struct Bob /// Boot Output Block
{
   DWord bootDisk;  // drive letter stored in DX by the BIOS
   DWord vgaWidth;  // (pixels)
   DWord vgaHeight; // (pixels)
   DWord vgaBpp;    // bits per pixel
   DWord vgaLfbp;   // address of the linear frame buffer
   DWord vgaCaps;   // mode attributes (offset 0 of the get mode structure)
   DWord vgaMode;   // value passed to INT 10/4F01 (BIOS set video mode)
};
</source>

Memory is grouped into 64KB segments (to take advantage of an assembler trick for zeroing it).

Bootl delivers us directly to 64 bit mode, thus each PTE is 8 bytes.  That allows for 512 PTEs per 4KB page, with 16 pages in the one segment allocated for the task.

They are:
{| {{wikitable}}
|-
! Number
! Description
|-
| 0
| PML4 (root page table, VA[47:39])
|-
| 1
| PDP (page directory pointers, VA[38:30])
|-
| 2
| PDE (page directory entry, VA[29:21] - good for 2M pages positioned from 0 to 1GB)
|-
| 3-15
| available
|}
````

<a id="w002960"></a>

## W002960 — User:Graham/Making a GRUB2 image

Bron-ID: W002960
Namespace: User (2)
Revisie: 11119 | Datum: 2011-01-22T21:07:05Z
Bron: https://wiki.osdev.org/User:Graham/Making_a_GRUB2_image
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGraham%2FMaking_a_GRUB2_image&oldid=11119

````text
== Introduction ==

This guide uses the latest GRUB2 (1.98+20100804-5ubuntu3) from the Ubuntu repositories (at the time of writing). I have not tested it with anything else but I don't see any reason why it shouldn't work.

GRUB2 is a bit more problematic to set up compared to the legacy version because you need some sort of partition table. Also, it is based around modules and you need to specify the minimum modules you need to be able to load all the other modules - this caused quite a headache for me when I tried using the grub-install command without specifying them.

I'm using an MS-DOS partition table with an ext2 filesystem in this tutorial. GRUB is also configured to read the disks with the BIOS functions. All of this is easily customizable however the combination I've used here seems to be chosen by most hobby developers.

== Making the image ==

Create an empty image file and set up a loop device for it:

  dd if=/dev/zero of=disk.img bs=512 count=32130
  sudo losetup /dev/loop0 disk.img

Create an MS-DOS partition table with fdisk:

  sudo fdisk /dev/loop0

Within the fdisk prompt type "o" to create a partition table, then "n" to create a new partition. Type "p" to create a primary partition and make it the first partition with "1". I left all the other settings as their defaults by hitting enter. Finally, use "w" to save the changes.

Find the first sector of the partition with fdisk:

  sudo fdisk -lu /dev/loop0

You need to multiply the number under "Start" by 512. For me, this was 63 * 512 = 32256. Now create a loop device for this partition (like how /dev/sda and /dev/sda0 would work on your normal system):

  sudo losetup -o 32256 /dev/loop1 /dev/loop0

Format this as ext2 and mount it to a temporary location:

  sudo mke2fs /dev/loop1
  mkdir temp
  sudo mount /dev/loop1 ./temp

Install GRUB:

  sudo grub-install --root-directory=./temp --disk-module=biosdisk --modules="ext2 part_msdos" /dev/loop0

Now unmount the partition and remove the loop devices:

  sudo umount ./temp
  sudo losetup -d /dev/loop1
  sudo losetup -d /dev/loop0
  rmdir ./temp

And test it:

  qemu -hda disk.img

If you get a GRUB prompt then it worked!

== Automatically updating the image ==

I use this shell script to automatically update my image with the latest kernel binary and grub configuration as part of the build process:

  #!/bin/sh -e
  mkdir -p ./mnt
  sudo mount -o loop,offset=32256 disk.img ./mnt
  sudo cp kernel ./mnt
  sudo cp grub.cfg ./mnt/boot/grub
  sync
  sudo umount ./mnt

You'll need to change the offset value to the one used above. What is nice about mounting and unmounting only with the mount/umount commands is that we do not need to mess around with the /dev/loopX devices directly which can be problematic if you forget to delete one after using it.
````

<a id="w002961"></a>

## W002961 — Talk:Global Descriptor Table

Bron-ID: W002961
Namespace: Talk (1)
Revisie: 29532 | Datum: 2025-06-05T11:27:42Z
Bron: https://wiki.osdev.org/Talk:Global_Descriptor_Table
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AGlobal_Descriptor_Table&oldid=29532

````text
== Endianness ==

Might a brief discussion (or link to a discussion) on endianness and loading the table directly in assembly be appropriate here? I just did this myself, and I was bitten because, for example, I tried to use in my code selector 0x59 instead of 0x9a (which is 0x9a backwards). I'd update it myself but I don't quite understand: I get that bytes go from least to most significant, but I'm not entirely sure I understand how the ''bits'' work for something like this. [[User:Surgo|Surgo]] 17:00, 24 January 2011 (UTC)

:The bits are not affected, which is probably why you didn't find any mention of them. -- [[User:Solar|Solar]] 15:34, 25 January 2011 (UTC)

:It's not that bits are not affected, it's actually that bits in a byte don't actually have any direction (don't let instructions like SHL or BT fool you --- they got their names based on common usage rather than functionality) despite being represented in one way or another by documentation. Bit positions are merely conventions whereas byte positions are tangible in software. This is precisely why we never talk about bits when discussing endianness. In case you don't understand, think of bytes as being represented in a row and each byte's bits being represented in a column; you end up with a Nx8 table... Which is bit 6? --[[User:Love4boobies|Love4boobies]] 06:53, 26 January 2011 (UTC)

::Bit #6 is the one that gives a value of 2^6 when set. (You ''do'' count from zero, right?) Look at it like this: If *bits* had an endianess, you would have to match endianess in your motherboard and the RAM you are using... ;-) -- [[User:Solar|Solar]] 14:05, 26 January 2011 (UTC)

:::That doesn't contradict what I've said :)

figure 3-1 refers to "Segmentation and Paging", nice overview, but has nothing to do with L-bit<br>
[[User:New16|New16]] 12:53, 25 July 2016 (CDT)

:It did say Table 3-1, not Figure 3-1... but that wasn't correct either. I've changed it to Figure 3-8. [[User:Johnburger|John Burger]] 23:48, 25 July 2016 (CDT)

== GDT Descriptor (GDTR) ==

In the table labeled GDT Descriptor (GDTR)<ref>[https://wiki.osdev.org/Global_Descriptor_Table#GDTR GDT Descriptor (GDTR)]</ref>, shouldn't it be '''47 (32-bit mode)''' in the first box. As 64 + 16 - 1 = 79, by same logic, 32 + 16 - 1 = 47. Is there anything that I'm missing? The same thing has been repeated in IDT Descriptor (IDTR)<ref>[https://wiki.osdev.org/Interrupt_Descriptor_Table#IDTR IDT Descriptor (IDTR)]</ref> After looking at Intel manual, Vol 3A Fig 2-6,<ref>[http://www.intel.com/design/processor/manuals/253668.pdf Intel® 64 and IA-32 Architectures Software Developer’s Manual, Volume 3A:. System Programming Guide, Part 1 (order number 253668)] Chapter 2.4 Figure 2-6</ref> it seems that it really should be 47 here.

{| class="wikitable"
|+ GDT Descriptor (GDTR)
!style="width: 66%; text-align: left;" |79 (64-bit mode) <br> 47 (32-bit mode)&nbsp;&nbsp;&nbsp;<span style="float: right;">16</span>
!style="width: 34%; text-align: left; vertical-align: bottom;" |15&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|-
|'''Offset'''<br>63 (64-bit mode) <br> 31 (32-bit mode)&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|'''Size'''<br><br>15&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|}
````

<a id="w002962"></a>

## W002962 — Cannot perform PE operations on non-PE file

Bron-ID: W002962
Namespace: Main (0)
Revisie: 11178 | Datum: 2011-02-07T17:03:15Z
Bron: https://wiki.osdev.org/Cannot_perform_PE_operations_on_non-PE_file
Revisie-URL: https://wiki.osdev.org/index.php?title=Cannot_perform_PE_operations_on_non-PE_file&oldid=11178
Doorverwijzing: GCC Cross-Compiler

````text
#REDIRECT [[GCC Cross-Compiler]]
[[Category:FAQ]]
````

<a id="w002963"></a>

## W002963 — Talk:MinGW

Bron-ID: W002963
Namespace: Talk (1)
Revisie: 14206 | Datum: 2013-01-07T04:12:08Z
Bron: https://wiki.osdev.org/Talk:MinGW
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMinGW&oldid=14206

````text
= Disadvantages =

The article has a list of 5 issues related to using MinGW for OSdev. Here is my take on them:

* 1..3: They were probably obsolete even when the article was started. GCC 2.95 is ancient.
* 4: What does ELF have to do with anything?
* 5: Seems like the appropriate thing to do, given that MinGW is a UNIX-like environment and comes with a shell. Amusingly, Cygwin is recommended over MinGW because of this even if it is known to have the exact same behavior.

--[[User:Love4boobies|Love4boobies]] 15:21, 17 December 2012 (CST)


MinGW has had quite its share of issues in the older days, with quite a few compiler and other software bugs being out there. Also, the key difference between mingw and cygwin is that the former is a windows port of the GNU toolchain designed to build Windows apps without getting into the mess that was Visual Studio, while the latter primarily aims at providing the closest thing to a true unix environment. 

Things on the mingw side have certainly gotten better over the past decade, but since I have migrated to a native linux install (via Cygwin) I only recall MinGW being an ugly thing for which you were lucky if it would survive if you truly applied some stress on it. I think this article needs a rewrite done by someone that is actually up to date with it. 

- [[User:Combuster|Combuster]] 07:39, 6 January 2013 (CST)

Under Windows, I generally use MinGW, which can be used to build GCC cross-compilers and cross-Binutils. However, I do admit that I've had my share of problems with it because you never know whether something will build out of the box or whether you will need to spend a lot of energy tracking down and fixing build problems. This doesn't seem to occur as often with Cygwin. Whether the fault belongs to MinGW or to the failing source code is not relevant to most of the users here, which is why I am inclined to think OSDev.org recommending Cygwin over MinGW is still the right decision. --[[User:Love4boobies|Love4boobies]] 22:11, 6 January 2013 (CST)

= Minimal? =

"MinGW is a 10+ megabyte download '''and, like Cygwin, now requires a helper DLL (MSYS-1.0.DLL)'''. So it's no longer "minimal"."

It doesn't. MinGW doesn't require any DLLs (apart from, of course, the runtime libraries). Perhaps somebody mistook MinGW for MSYS. And it is minimal - it doesn't try to emulate POSIX, it uses default Windows' libraries instead. It's just a Windows port of GNU toolchain, nothing more.

[[User:Fanael|Fanael]] 12:55, 8 February 2011 (UTC)

:The nomenclature of the MinGW project is confusing as to what is contained in "MinGW" and what is "add-on", but I would tend to agree that MSYS is apart from the MinGW proper. Feel free to reword the lead. -- [[User:Solar|Solar]] 13:47, 8 February 2011 (UTC)

::I'm not sure how, really. I'd probably delete the mentioned sentences entirely, but then the thing about MinGW <tt>make</tt> would look silly (a list with one element?). Any other ideas? -- [[User:Fanael|Fanael]] 14:20, 8 February 2011 (UTC)

:::Better? -- [[User:Solar|Solar]] 15:17, 8 February 2011 (UTC)

::::Yeah, now it looks better. -- [[User:Fanael|Fanael]] 15:49, 8 February 2011 (UTC)
````

<a id="w002964"></a>

## W002964 — Talk:8259 PIC

Bron-ID: W002964
Namespace: Talk (1)
Revisie: 27710 | Datum: 2023-03-30T18:08:53Z
Bron: https://wiki.osdev.org/Talk:8259_PIC
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3A8259_PIC&oldid=27710

````text
Discussion Topics:

== Masking ==
Shouldn't the masking function be in one function?
Also, shouldn't we have a topic about all the registers of the PIC? [[User:Primis|Primis]] 19:53, 27 February 2011 (UTC)

No, having the masking done in two functions shows clearly what needs to be done. Also, Primis, please make sure any edits you make are actually correct. I have reverted your "fix" to the IRQ number checks.
[[User:Thepowersgang|Thepowersgang]] 23:49, 27 February 2011 (UTC)

Well there are 8 IRC lines per PIC, but they start at zero, so the first PIC is 0 ... 7 and the second is 8 ... 15. However the function is not Greater than or Equal to (>=) but is just greater than (>) so the function is gonna be out of sync no? [[User:Primis|Primis]] 01:55, 1 March 2011 (UTC)

The operation is < in this case, 0 - 7 are less than 8, so they should they use the first controller. The current code is correct.
[[User:Thepowersgang|Thepowersgang]] 04:16, 1 March 2011 (UTC)


== History ==
I'm not sure, but, I remember that Norton says about 16 hardware irq's in PC. [[User:418imateapot|418imateapot]]
: {0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15}  is a set of 16 numbers; yes. There are 16 hardware IRQs.

== io_wait ==

The PIC_remap function  makes use of an io_wait function -- any suggestions as to how to implement this function? read a port?[[User:Clearer|Clearer]] 10:18, 16 December 2011 (UTC)
Nevermind: http://wiki.osdev.org/Inline_Assembly/Examples

Don't - the IO delays are unnecessary in general (on 80486 and later) and especially unnecessary in that example code (as you're interleaving IO port accesses to different chips) --[[User:Brendan|Brendan]] 21:16, 16 December 2011 (UTC)

== Suggestion ==
Shouldn't the abstract theory surrounding a "Programmable IRQ/Interrupt Controller" and then the concrete example of the i8259 chips be separated? Would like to suggest not redirecting "Programmmable Interrupt Controller" to this page, and creating a separate article for it. [[User:Gravaera|Gravaera]] 07:50, 13 June 2012 (CDT)

Does any other hardware have a PIC that's generally called just PIC?

== general usage of IRQ numbers ==
I'm missing a table of what the general usage for each interrupt number is. E.g. 12 is PS2 Mouse. This could link to
http://wiki.osdev.org/index.php?title=Interrupts#Standard_ISA_IRQs

== Disabling ==
Is there a reason this is written in assembly and hard coded, rather than using the functions consistent with the rest of the article?
<source lang="asm">
mov al, 0xff
out 0xa1, al  // outb(PIC1_DATA, 0xFF);
out 0x21, al  // outb(PIC2_DATA, 0xFF);
</source>

== ICW4 and other macros ==

The article says in the #define comment for ICW1_ICW4 that ICW4 is not needed, but this macro is used when sending ICW1.
[[User:Tree5678|Tree5678]] 03:05, 3 July 2022 (CDT)
: I think it actually is needed and the comment is misleading. If I understand it correctly, X86 CPUs need the PIC to be in 8086 mode (the PIC seems to assume by default that it's connected to an 8080 CPU which needs some slightly different signals, so the mode needs to be changed in ICW4). So here's how I understand the code
:* Send an ICW1, which tells the PIC that we want to initialize it and that the initialization will include a fourth control word (ICW4). And send it to both PICs.
:* Send an ICW2, setting the offset of the interrupts. So line n triggers interrupt n + offset_x, where we want each PIC to have a different offset so that we can tell which PIC an interrupt is coming from.
:* Send an ICW3 to each PIC, telling each one that the other one exists.
:* Send an ICW4 to both PICs telling them to use the 8086 mode.
: I suppose I could at least change the comment to mention that ICW4 '''is''' needed, and add a comment where the ICW4 gets sent to the master PIC (since we're sending the same to both PICs, I suppose that repeating the comment would be redundant).
: I hope this makes sense. I think I understand what those control words are doing, but I'm not sure how readable my explanation of it is. - [[User:SeaLiteral|SeaLiteral]] ([[User talk:SeaLiteral|talk]]) 12:54, 30 March 2023 (CDT)
````

<a id="w002965"></a>

## W002965 — User talk:Love4boobies

Bron-ID: W002965
Namespace: User talk (3)
Revisie: 20763 | Datum: 2017-05-01T00:01:42Z
Bron: https://wiki.osdev.org/User_talk:Love4boobies
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ALove4boobies&oldid=20763

````text
== ACPI content deletion ==

Why did you undo my edit to [[FADT]]? You said that it didn't use C types. But it uses the same types used in [[RSDT]] and [[RSDP]]. Moreover you can't use C types to describe a 64 bit type for example. It would be unsigned long long on x86, and unsigned long on x86_64, so a typedef like QWORD is justified IMO. And could you please explain the "serialization" problem? [[User:Alfaomega08|Alfaomega08]]

:You can. Use <stdint.h> and you get uint64_t et al. -- [[User:Solar|Solar]] 16:25, 10 March 2011 (UTC)

::You can't do that in ANSI C, nor you can in C++, so <stdint.h> is close to useless for anyone who can't or don't want to use C99 features. -- [[User:Fanael|Fanael]] 17:38, 10 March 2011 (UTC)

:Solar is correct and it's what I meant by that. As for serialization, it was also explained in the description. It is important to learn C before writing an OS in it. Structure fields may have gaps between them, they are not like arrays (e.g., because of alignment requirements or cache optimizations). --[[User:Love4boobies|Love4boobies]] 17:11, 10 March 2011 (UTC)

:Do you mean I should put an __attribute__((packed)) at the end of the structure? If that's the problem, I skipped it because 1) it's not cross-platform 2) I think it is given for granted that any structure in os development should be packed. -- [[User:Alfaomega08|Alfaomega08]] 17:22, 10 March 2011 (UTC)

:No, that's not what I meant. Here are a couple of links to get you started: [http://en.wikipedia.org/wiki/Serialization Serialization], [http://wiki.osdev.org/Getting_Started#Required_Knowledge required knowledge to starting an OS]. --[[User:Love4boobies|Love4boobies]] 17:38, 10 March 2011 (UTC)

:Well, I know what serialization is, and I'm quite sure to have the necessary skills to program an OS, however I still can't see how "Structure fields may have gaps between them" fits in the picture, as the packed attribute should remove all those gaps... -- [[User:Alfaomega08|Alfaomega08]] 17:33, 10 March 2011 (UTC)

:Except we both agreed that using __attribute__((packed)) isn't portable across C implementations. If you know what serialization is, learn how to do it (or at least apply some of the concepts) in C as well :) --[[User:Love4boobies|Love4boobies]] 17:38, 10 March 2011 (UTC)

int*_t and uint*_t are standard C, and are also the '''only''' official standard. If we want to adhere to a standard, we lterally have no choice. If you suffer from NIH or don't want to adhere to a standard, you just typedef them in a header of your own. "word" and "doubleword" are not a standard - on many non-intel platforms a word means 32 bits instead of 16. Also, the packed attribute should be added since the structure expects it to be that way. If you something non-standard (read: non-gcc) you should know how to substitute it. It's better to have things work out-of-the-box for 95% of the community, or have everybody (forget to) make their own fixes by lacking a standard altogether. Now, can we get this over with? The pendant votes and flames aren't helping anyone. 

Back to the thing that started all this: I was shocked by the deletion. If you disagree, fix it, but deleting a page full of correct and useful information is about the worst decision you could make on a wiki. Rollback and <tt>karma /= 2;</tt> to you. - [[User:Combuster|Combuster]] 22:34, 10 March 2011 (UTC)

I was going to rewrite that but in the meantime didn't want the wiki to have incorrect information. Picking up on the fact that words and dwords are 16-bit and 32-bit, respectively, is easy. On the other hand, using an unpacked structure is a bit more subtle---this is why the code was '''broken'''. Can I haz karma back? --[[User:Love4boobies|Love4boobies]] 06:36, 11 March 2011 (UTC)

:If you get into a situation like this - a Wiki article containing partially false information but the correction being non-trivial - add a ''note'' about the problems with the information, and a link to where you are doing the rework. Once the rework is done, copy it over to the article in question. Don't delete info and rewrite it "later", because - you guessed it already - later never comes. -- [[User:Solar|Solar]] 09:05, 11 March 2011 (UTC)

:I see your point but I think the community needs to agree on a more formal protocol or something because common sense is slightly different for everyone (e.g., I would rather not have information that's slightly wrong and assume it correct in a situation like this). It's not really productive for us to argue about such matters. Btw, the problem is still not solved but hopefully I, or someone else, will fix it soon (and not later) :) --[[User:Love4boobies|Love4boobies]] 09:16, 11 March 2011 (UTC)

:This ''is'' the "formal protocol" (as much as such a thing exists) on knowledge Wikis: Annotate, rewrite in scratch namespace, replace. Slightly wrong information with a note what *is* slightly wrong about it is better than no information at all. Nothing ever is perfect, that's the Wiki creed. I don't want to argue, I'm just pointing out. -- [[User:Solar|Solar]] 09:34, 11 March 2011 (UTC)

:As such, I see two solutions for the FADT page: add a note / comment that you're "assuming packed structures", or (even better) describe the data structure layout in text instead of code. (That way it's just as helpful for e.g. our Pascal friends, and we're not risking anyone doing the copy & paste.) That really should do it, IMHO. I'd do it myself, but I have no stocks in the subjects anyway. ;-) -- [[User:Solar|Solar]] 09:40, 11 March 2011 (UTC)

== Google Docs link broken ==

The link to the AMD64 ABI for UDI draft on the main page is currently broken, seemingly by the Google Docs to Drive transition. Is there still a copy somewhere? --[[User:Alexshpilkin|Alexshpilkin]] 16:09, 10 March 2014 (CDT)

:Hi. Sorry for the delay. You can find a copy [https://gitorious.org/udi/specs/raw/19917f5d18ff5fe424a77a239a9830cfd4f8040e:amd64_abi_spec.pdf here]. --[[User:Love4boobies|Love4boobies]] 06:21, 9 July 2014 (CDT)

== Copied from Wikipedia for no reason ==
:Actively objecting the deletion, please read the [https://github.com/CosmosOS/Cosmos/issues/629#issuecomment-298256021 discussion here] --[[User:Lexected|Lexected]] 16:27, 30 April 2017 (CDT)
::Actively objecting the objection. It seems that the only reason you want to provide details for an internal tool (used by COSMOS and nobody else) is publicity. In other words, I don't see how this is not spam. [[User:Brendan|Brendan]] 16:49, 30 April 2017 (CDT)
::It's not related to OS development. Furthermore, what is the point of mirroring a Wikipedia article here? It's already there and all our version will achieve will be to quickly go out of sync with the original. EDIT: Oh, even Wikipedia doesn't want it so this is clearly PR. --[[User:Love4boobies|Love4boobies]] 19:00, 30 April 2017 (CDT)
````

<a id="w002966"></a>

## W002966 — User:Neolander

Bron-ID: W002966
Namespace: User (2)
Revisie: 11268 | Datum: 2011-03-14T11:03:25Z
Bron: https://wiki.osdev.org/User:Neolander
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANeolander&oldid=11268

````text
== Interrupts in a nutshell ==

CPUs spend most of their time running software, which for them means following a predictable flow of instructions. However, in some situations, it is desirable to interrupt a program's flow and have the CPU do something totally different instead.

Imagine, as an example, that a program, while doing maths, incidentally divides a number by zero. This mathematically makes no sense, so the CPU can't give a reliable result, and the program's execution can't continue normally. In that case, it's best to have the CPU run a chunk of code designed to handle this event. This mechanism is called an '''exception'''.

Another event in which a program's flow should be interrupted is when something new happens on the hardware side. Imagine, as an example, that you are running some piece of software which accepts keyboard input. How is it informed that new keys have been pressed ? In the days where dinosaurs where roaming the Earth, it was common to ask such software to periodically check if anything new has happened on the keyboard side. This process, called polling, is cumbersome, prone to failure (if software does not check keyboard often enough, key presses will be lost), and both inefficient in terms of CPU cycles and power consumption. It is in most cases much better to have the keyboard notify the CPU when a key is pressed, and make sure that this triggers the execution of a chunk of code which registers the key press before giving back control to the previously running software. From one CPU architecture to another, the name of this process varies, but it is commonly called an '''Interrupt ReQuest (IRQ)''', '''external interrupt''', or just '''interrupt'''.
````

<a id="w002967"></a>

## W002967 — User talk:Neolander

Bron-ID: W002967
Namespace: User talk (3)
Revisie: 11269 | Datum: 2011-03-14T12:26:28Z
Bron: https://wiki.osdev.org/User_talk:Neolander
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ANeolander&oldid=11269

````text
== Interrupts ==

I see that you are trying to rewrite the [[Interrupts]] article. I just wanted to note that this Wiki does ''not'' attempt to explain subjects to the layman, but to the aspiring OS developer. I fear that the introduction, as you have written it so far, will likely be considered too "basic". We'd never see the end of it if we would try to explain things from ''that'' starting level. Even the Wikipedia entry on [http://en.wikipedia.org/wiki/Interrupt Interrupts] assumes a higher level of previous knowledge... -- [[User:Solar|Solar]] 12:26, 14 March 2011 (UTC)
````

<a id="w002968"></a>

## W002968 — LLVM Cross-Compiler

Bron-ID: W002968
Namespace: Main (0)
Revisie: 27941 | Datum: 2023-07-09T15:53:37Z
Bron: https://wiki.osdev.org/LLVM_Cross-Compiler
Revisie-URL: https://wiki.osdev.org/index.php?title=LLVM_Cross-Compiler&oldid=27941

````text
{{In_Progress}}
{{rating|3}}

= Introduction =
 NOTE: This article is not complete yet.
 Having working and production ready llvm/clang cross-compiler involves much more work than just what is explained here
 (such that having llvm/clang use correct target associated sysroot, static linker, C/C++ libraries, etc.)
 See [http://clang.llvm.org/UniversalDriver.html clang Universal driver].
 For working and production ready clang/llvm cross-compiler use dedicated tools (such as [http://www.embtoolkit.org EmbToolkit]) generating one for you.
=== About ===

Generally speaking, a cross-compiler is a compiler that runs on platform A (the '''host'''), but generates executables for platform B (the '''target'''). These two platforms may (but do not need to) differ in CPU, operating system, and/or [[:Category:Executable Formats|executable format]].

Clang (and llvm) are host cross compilers. They by default have the capability to cross compile, but also produce host binaries. See Usage
=== Requirements ===

* A host system with a ''working'' compiler (can be GCC, Clang, etc).
* A bash shell or comparable environment. If you are not using a bash shell, you might have to modify some of the command lines below. If you have just installed the basic [[Cygwin]] package, you have to run the setup.exe again and install the following packages:
** GNU Make or CMake
** GNU Binutils (a fairly recent version of Binutils, try 2.21 or above if you get assembly compilation errors)
** GIT or SVN (if building from sources)
** Curl (if building from source using the linked article)

== Building ==

=== Checking out sources ===

Clang/LLVM sources can be checked out either with GIT or SVN.

For GIT, in bash:

<source lang="bash">
mkdir crossllvm
cd crossllvm
git clone http://llvm.org/git/llvm.git
cd llvm/tools
git clone http://llvm.org/git/clang.git
cd ../projects
git clone http://llvm.org/git/compiler-rt.git
</source>

For SVN, in bash:

<source lang="bash">
mkdir crossllvm
cd crossllvm
svn co http://llvm.org/svn/llvm-project/llvm/trunk llvm
cd llvm/tools
svn co http://llvm.org/svn/llvm-project/cfe/trunk clang
cd ../projects
svn co http://llvm.org/svn/llvm-project/compiler-rt/trunk compiler-rt
</source>

=== Building from sources (Possibly Unstable) ===
After having checked out the sources (see above), in bash, from the "crossllvm" directory:
<source lang="bash">
mkdir build
cd build
../llvm/configure --enable-optimized
make
make install
</source>

Note: This Autoconf build has been removed from current versions of the LLVM build. All new builds must use CMake.

Or with CMake (adjust the source path as needed):

<source lang="bash">
mkdir build
cd build
cmake -DCMAKE_BUILD_TYPE:STRING=Release ../llvm/
make
make install
</source>

Now you have got LLVM and Clang built!

Alternatively, a faster and more up-to-date way for building from source (using ninja) is available [http://atta-metta.net/blog/2013/05/23/compiling-llvm-trunk/ here]. The linked article was written in May of 2013, so all of the article's provided custom patches are no longer required. Assuming you are using recent versions (released during or after June 2013) of compiler-rt, lldb, and libc++ then you can safely skip installing the provided patches within the article and proceed with compiling using the unmodified content of the git repos the article instructs you to download.

A Clang cross-compiler is generated by [https://github.com/berkus/metta/blob/develop/build_toolchain.sh this buildscript]. Dissect it to learn more. It uses some gcc and binutils to provide a fully working toolchain.

=== Debian ===
Open a terminal
<source lang="bash">
sudo apt-get install clang
</source>
Note, you might have to disable host functionality. See Useful Flags

== Building libc++ ==

<source lang="bash">
$ git clone http://llvm.org/git/libcxx.git
$ export TRIPLE=-apple-
$ cd libcxx/lib
$ ./buildit
</source>

Built libc++ is not installed by default, you can use make install or copy files manually.


== Usage ==
After building you will have a compiler able to output multiple output formats regardless of your current platform.

For example, for cross compiling to ARM, you can use
<source lang="bash">
-march=armv7-a -mfloat-abi=soft -ccc-host-triple arm-elf
</source>
Since 3.1, it can be shortened to
<source lang="bash">
-target armv7--eabi -mcpu=cortex-a9
</source>
An example for compiling to a generic X86 ELF target would be:
<source lang="bash">
--target=i686-pc-none-elf -march=i686 
</source>

=== Useful Flags ===
Some usefull flags for OS development.
==== -ffreestanding ====
Indicated that the file should be compiled for a freestanding enviroment (like a kernel), not a hosted (userspace), environment.
==== -fno-builtin ====
Disable special handling and optimizations of builtin functions like strlen and malloc.
==== -nostdlib ====
Disables standard library
==== -nostdinc ====
Makes sure the standard library headers are not included.
==== -nostdinc++ ====
Makes sure the standard C++ library headers are not included. This makes sense if you build a custom version of libc++ and want to avoid including system one.

== Using system roots ==
Due to Clang's dependency on ld, you may get the error message "/usr/bin/ld: this linker was not configured to use sysroots". If you receive this error, you'll probably have to compile binutils with sysroot support. There is information how to do this [[GCC Cross-Compiler#Binutils]] - note the "--with-sysroot" flag.

== TODO ==

 TODO: describe non-svn build from released tarballs.
 TODO: [http://www.embtoolkit.org EmbToolkit project] recently added support to clang/llvm based cross compiler, ddd note about how it works.

[[Category:Compilers]]
````

<a id="w002969"></a>

## W002969 — Talk:LLVM Cross-Compiler

Bron-ID: W002969
Namespace: Talk (1)
Revisie: 21523 | Datum: 2017-09-18T11:53:31Z
Bron: https://wiki.osdev.org/Talk:LLVM_Cross-Compiler
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ALLVM_Cross-Compiler&oldid=21523

````text
== Linux is a freestanding environment? ==
I noticed the need for using <tt>i686-pc-linux-gnu</tt> as the target, which essentially means that you want to build for a linux host, not a freestanding host. A cross-compiler is meant to counter all host dependencies. So can you either explain why <tt>i*86-(pc-)elf</tt> does not work, or at least guarantee that the linux and gnu parts do not drag in dependencies to stuff we don't have, or worse, stuff we don't ''want to have''. Can we expect it to be the same in the future? - [[User:Combuster|Combuster]] 12:06, 21 March 2011 (UTC)

== Breaks on Cygwin. ==

    Scanning dependencies of target intrinsics_gen
    [  7%] Building Intrinsics.gen...
    make[2]: *** [include/llvm/Intrinsics.gen.tmp] Fehler 127
    make[1]: *** [include/llvm/CMakeFiles/intrinsics_gen.dir/all] Fehler 2
    make: *** [all] Fehler 2

Now there's a helpful error message if I ever saw one. -- [[User:Solar|Solar]] 12:19, 21 March 2011 (UTC)

== Dependency on binutils ==

This is not a pure toolchain. Clang depends on binutils for a few things, like <tt>as</tt> for assembling (when not using <tt>-integrated-as</tt> or <tt>-emit-llvm</tt>) and <tt>ld</tt> / <tt>ld.gold</tt> for linking. The Clang website talks about a [http://clang.llvm.org/UniversalDriver.html Universal Driver Project], but it's not done yet. Be careful when using Clang for cross-compiling without adding your target to <tt>lib/Driver/ToolChains.cpp</tt>, I guess. --[[User:Walling|Walling]] 23:22, 7 April 2011 (UTC)

== Make error 127 ==

"error 127 indicates that make can't find a program that it needs to run a command"

try "make VERBOSE=1" to see what is that command you miss.

== Broken link to atta-metta.com ==

This link: http://atta-metta.net/blog/2013/05/23/compiling-llvm-trunk/ (found here: http://wiki.osdev.org/LLVM_Cross-Compiler#Building_from_sources_.28Possibly_Unstable.29) appears to be broken (404's for me).
We may want to remove it...
````

<a id="w002970"></a>

## W002970 — Logical Block Addressing

Bron-ID: W002970
Namespace: Main (0)
Revisie: 30324 | Datum: 2026-03-17T19:08:59Z
Bron: https://wiki.osdev.org/Logical_Block_Addressing
Revisie-URL: https://wiki.osdev.org/index.php?title=Logical_Block_Addressing&oldid=30324

````text
'''Logical Block Addressing''' (LBA) is a linear addressing scheme for data blocks (sectors) on data storage devices. Most modern storage devices use LBA addressing.


== Uses ==
LBAs are used to access certain blocks on a storage medium. LBA is said to be 'linear' because accessing <tt>LBA + 1</tt>, will indeed address the data block immediately after the block at <tt>LBA</tt>.

LBAs can be stored in several different formats. Sometimes they are stored relative to the start of a partition or filesystem. They may also be stored in an absolute way (from the beginning of the storage device). LBAs are almost always dependent on the block size for the device in question. Suppose we have a hard drive with 512 bytes per sector (or bytes per block), then an LBA of 50 would specify the sector containing bytes (50 * 512) to (51 * 512 - 1).

== Comparison with CHS ==
CHS (Cylinder Head Sector) is an older way of addressing a storage device. Devices such as floppies usually have no direct way of being accessed with LBAs. Instead, they require you to specify the cylinder, the head, and the sector you want to access individually. However, since LBAs are easy to use, it is not uncommon to provide a conversion between the two.

For example, one could convert an LBA for a floppy drive to the CHS with the following code snippet (based on Brokenthorn's Floppy tutorial, see the link below):

<syntaxhighlight lang="c">
void ToCHS(int lba, int *head, int *track, int *sector)
{
	(*head) = (lba % (SECTORS_PER_TRACK * 2)) / SECTORS_PER_TRACK;
	(*track) = (lba / (SECTORS_PER_TRACK * 2));
	(*sector) = (lba % SECTORS_PER_TRACK + 1);
}
</syntaxhighlight>

== See Also ==
=== External ===
* [http://www.brokenthorn.com/Resources/OSDev20.html Brokenthorn's Floppy Tutorial]

[[Category:ATA]]
[[Category:Disks]]
[[Category:Storage]]
[[de:Logical Block Addressing]]
````

<a id="w002971"></a>

## W002971 — Smart Pointer

Bron-ID: W002971
Namespace: Main (0)
Revisie: 11341 | Datum: 2011-03-29T08:32:54Z
Bron: https://wiki.osdev.org/Smart_Pointer
Revisie-URL: https://wiki.osdev.org/index.php?title=Smart_Pointer&oldid=11341

````text
A smart pointer is a datatype simulating a pointer, providing extra features in its implementation.

== Definition ==
A smart pointer is in fact very similar to a standard pointer. It is usually wrapped within a class where it can provide additional functionality. One of the features smart pointers can offer is automatic garbage collection. Some smart pointers automatically free the memory associated with it when the destructor is called. This may sometimes involve a reference counter that keep tracks of how many instances of the class point to the same object. This makes smart pointers especially interesting for automatic garbage collection and protection against memory leaks.

== See Also ==
=== Articles ===
* [[Garbage collection]]

[[Category:Memory management]]
````

<a id="w002973"></a>

## W002973 — Detecting CPU Topology (80x86)

Bron-ID: W002973
Namespace: Main (0)
Revisie: 28239 | Datum: 2023-07-10T23:27:32Z
Bron: https://wiki.osdev.org/Detecting_CPU_Topology_(80x86)
Revisie-URL: https://wiki.osdev.org/index.php?title=Detecting_CPU_Topology_%2880x86%29&oldid=28239

````text
For certain kinds of optimizations, sometimes it's useful for applications and OSs to detect CPU topology and adjust (optimize) its behaviour accordingly.

== Uses ==
CPU Topology data is useful for initializing your physical memory manager, especially information about NUMA domains and the cache. This means that you may wish to build the topology tree before you have a fully functioning memory allocator. Further down the line, this information will be of importance to your [[scheduler]].
== Detecting CPU Topology ==

For complete CPU topology, there are 4 levels to consider:
* Which logical CPU within a core (implemented, for example, by Intel's hyperthreading)
* Which core within a chip
* Which chip within a NUMA domain
* Which [[Multiprocessing#NUMA_.28Non-Uniform_Memory_Access.29|NUMA]] domain

For convenience, this can be expressed as a colon-separated ID. For example, "CPU 3:2:1:0" would be logical CPU #0 in core #1 of chip/package number 2, in NUMA domain #3.

== General Steps ==

To begin with, the ACPI [[MADT]] table or the MP Specification table should be used to determine the APIC IDs for all CPUs. To better organize the information, a table of structures can be created that contains the information that is different for each CPU (e.g. APIC ID, topology information, etc.). This "CPU info" structure could include a pointer to a separate "CPU description" structure that stores information that can be the same for different CPUs (e.g. vendor, family/model/stepping, brand string, feature flags, cache sizes, etc). 

To detect NUMA domain information the ACPI [[SRAT]] table (System Resource Affinity Table) has to be used. First, preset the "NUMA domain" field for each CPU to "unknown" (in case the CPU isn't mentioned in the SRAT table). Then, for each entry in the SRAT table you determine if it's a CPU or memory, and if the entry is a CPU the 'APIC ID' field is used to determine which CPU and the 'proximity domain' field to determine which NUMA domain that CPU is in; and set the NUMA domain for that CPU. Note that there are 2 different structures in the SRAT table - one for 8-bit local APIC IDs and one for 32-bit APIC IDs, and if you support x2APIC you need to use both.

To get nice clean results in the later steps, the list of CPUs should be sorted now - in order of NUMA domain, each NUMA domain in order of APIC ID.

For cores and logical CPUs; for each CPU, CPUID should determine:
* Number of bits in the APIC ID that are used to identify which logical CPU
* Number of bits in the APIC ID that are used to identify which core

The values are used for a set of calculations:
* logical_CPU_number_within_core = APIC_ID & ( (1 << logical_CPU_bits) -1)
* core_number_within_chip = (APIC_ID >> logical_CPU_bits) & ( (1 << core_bits) -1)
* chip_ID = APIC_ID & ~( (1 << (logical_CPU_bits+core_bits) ) -1)

How to use CPUID to determine these values is described below in the "Using CPUID" section. It should be noted that you only need to obtain 2 values: "logical_CPU_bits" and "core_bits".

The "chip ID" is a temporary value that you could discard later.

These calculations should be done separately for each logical CPU because it's theoretically possible for different physical chips in the same computer to use APIC ID bits differently.

Once you've done these calculations the "chip number within NUMA domain" can be determined. All CPUs that have the same "chip ID" and the same NUMA domain are on the same chip.

For example:

<source lang="C">
    for(domain = 0; domain <= highest_domain; domain++) {
        chip_number = 0;
        for(firstCPU = 0; firstCPU < total_CPUs; firstCPU++) {
            if( CPU_info_table[firstCPU].domain == domain) {
                if( CPU_info_table[firstCPU].chipNumber == -1) {
                    CPU_info_table[firstCPU].chipNumber = chip_number;
                    chipID = CPU_info_table[firstCPU].chipID;
                    for(i = firstCPU; i < total_CPUs; i++) {
                        if( CPU_info_table[i].domain == domain) {
                            if( CPU_info_table[i].chipID == chipID) {
                                CPU_info_table[i].chipNumber = chip_number;
                            }
                        }
                    }
                }
                chipNumber++;
            }
        }
    }</source>

There's a bit of strangeness which should be explained here. In general, it doesn't make too much sense for the same chip to be in different NUMA domains. However, when Intel designed x2APIC they decided that Logical APIC ID (which shouldn't be confused with the "APIC ID") and the "Destination Format Register" would be hardwired; and that "Logical Destination Mode" would only support "cluster model". They decided that the highest bits of the Logical APIC ID would determine which "group of CPUs" and the lowest 4 bits of the Logical APIC ID would determine which CPU within the group. The end result is that, if a chip has more than 16 logical CPUs it will be split into different NUMA domains. For the algorithm above, this case is treated as separate chips in different NUMA domains (even though it is technically one physical chip spread across multiple NUMA domains). This is deliberate, as it avoids complications everywhere else (in the APIC/interrupt handling code, in the scheduler, etc).

== Using CPUID ==

From the "General Steps" section above, we need to obtain 2 values: "logical_CPU_bits" and "core_bits".

There are several different CPUID functions that are useful for detecting CPU topology. They are shown in order below.

=== CPUID eax=0x00000001 ===

This is the first step. Check if the "HTT" flag is set. If the HTT flag is clear (or if the CPU doesn't support CPUID) then it's an old single-core CPU without hyper-threading. In that case "logical_CPU_bits = 0" and "core_bits = 0" and you're done.

Otherwise (if the "HTT" flag is set), you want to get the "count" field from EBX bits 16 to 23. You (might) need it later.

=== INTEL: CPUID eax=0x0000000B ===

For Intel CPUs (and <u>not</u> AMD), this CPUID function tells you "Number of bits to shift right APIC ID to get next level APIC ID", and needs to be used twice. The first time (with ECX=0) it tells you how many bits of the APIC ID is used to identify the logical CPU within each core (logical_CPU_bits). The second time (with ECX=1) it tells you how many bits of the APIC ID is used to identify the core and logical CPU within the chip, and to get "core_bits" from this value you subtract "logical_CPU_bits" from it.

=== INTEL: CPUID eax=0x00000004 ===

For Intel CPUs (and <u>not</u> AMD), if "CPUID eax=0x0000000B" wasn't supported then this is the next CPUID function to try. You're only interested in EAX bits 26 to 31, which tells you the number of APIC IDs reserved for this chip. The count field from earlier tells you the max number of logical CPUs in a package, whereas (EAX >> 26) & 0x3F from this function tells you the maximum number of cores in a package, -1. Hence, to get "logical_CPU_bits" and "core_bits", write a function that counts the number of left-shifts to get 0, which is the number of bits to represent a number. core_bits is func((EAX >> 26) & 0x3F), logical_CPU_bits is func(count - 1) - core_bits. Note that the initial ecx value does not affect EAX bits 26 to 31.

If "CPUID eax=0x00000004" isn't supported, then it's a single-core CPU with hyper-threading. Use the "count" field you got earlier (from CPUID eax=0x00000001) rounded up to the next power of 2 to determine "logical_CPU_bits", then set "core_bits = 0".

=== AMD: CPUID eax=0x80000008 ===

For AMD CPUs (and <u>not</u> Intel), use this function to determine "core_bits". First check ECX bits 12 to 15, if it is not zero, then it contains "core_bits". Otherwise, use ECX bits 0 to 7 to determine the number of cores, round it up to the next power of 2 and use it to determine "core_bits".

Now that you've got "core_bits"; use the "count" field you got earlier (from CPUID eax=0x00000001) and shift it right by "core_bits" bits. Then round the result up to the next power of 2 and use it to determine "logical_CPU_bits".

=== AMD: CPUID eax=0x00000001 ===

For AMD CPUs (and <u>not</u> Intel), if CPUID eax=0x80000008 isn't supported just assume that the "count" field you got earlier (from CPUID eax=0x00000001) is the number of cores; and round it up to the next power of 2 and use it to determine "core_bits". In this case set "logical_CPU_bits" to zero.

=== Managing CPU Topology ===
Your kernel needs to manage the processor topology in the system for things like load-balancing. You should make a topological tree which would be built by the system during setup.

<source lang="C">
      typedef
      struct _CPU_DOMAIN {
          ULONG DomainID;/* SMT_ID, CoreID, PackageID or ClusterID*/
          LINKED_LIST ChildGroups;
          struct _CPU_DOMAIN *ParentDomain;
          SPIN_LOCK DomainLock;
      } CPU_DOMAIN;

      CPU_DOMAIN systemTopology;
</source>

A good implementation would make each CPU parse the tree. For example, each AP would acquire the lock for the systemTopology
domain. Then, it will search for its child domain, with sequential IDs like ClusterID, PackageID, CoreID, and SMT_ID adding a
CPU_DOMAIN struct to the ChildGroups list if not present. Finally, it will add a CPU_DOMAIN with its SMT_ID with no children 
(representing the logical CPU).
== Cache Topology ==
The complexity of your cache topology code mainly depends on how exotic the systems you wish to support are. Most multiprocessor systems have the same processor in all sockets. However, it is possible to get systems with minorly different CPUs. Further, in the future, it's conceivable that there will be systems with more than one architecture onboard (this is already true to an extent with GPUs). It is a non-trivial design consideration whether you store cache data per CPU or globally. FOr instance, if working on ARM, big.LITTLE technology means you can have two CPUs with different cache line sizes.

== See Also ==
[https://www.mono-project.com/news/2016/09/12/arm64-icache/ ARM big.LITTLE and cache]

=== Articles ===
* [[CPUID]]

[[Category:X86 CPU]]
[[Category:Hardware Detection]]
[[de:CPUID]]
````

<a id="w002974"></a>

## W002974 — Talk:Detecting CPU Topology (80x86)

Bron-ID: W002974
Namespace: Talk (1)
Revisie: 11403 | Datum: 2011-04-18T12:39:24Z
Bron: https://wiki.osdev.org/Talk:Detecting_CPU_Topology_(80x86)
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ADetecting_CPU_Topology_%2880x86%29&oldid=11403

````text
= Writing Style =

This article should probably be rewritten properly, so that it doesn't sound like someone's forum post (e.g. replace things like "I think" with things like "some people think" or "Brendan thinks"). --[[User:Brendan|Brendan]] 12:39, 18 April 2011 (UTC)
````

<a id="w002975"></a>

## W002975 — JWASM

Bron-ID: W002975
Namespace: Main (0)
Revisie: 11413 | Datum: 2011-04-19T11:40:55Z
Bron: https://wiki.osdev.org/JWASM
Revisie-URL: https://wiki.osdev.org/index.php?title=JWASM&oldid=11413
Doorverwijzing: JWasm

````text
#REDIRECT [[JWasm]]
````

<a id="w002976"></a>

## W002976 — Talk:Why do I need a Cross Compiler?

Bron-ID: W002976
Namespace: Talk (1)
Revisie: 15521 | Datum: 2014-01-07T16:14:48Z
Bron: https://wiki.osdev.org/Talk:Why_do_I_need_a_Cross_Compiler%3F
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AWhy_do_I_need_a_Cross_Compiler%3F&oldid=15521

````text
As we're talking about GCC, I really don't see any reason for which a cross-compiler is neccessary, unless a different target architecture and/or executable format are needed. However, this is usually not the case. The more natural solution is to use the free-standing environment provided by GCC. --[[User:Love4boobies|Love4boobies]] 15:01, 21 April 2011 (UTC)
:To get around distribution patches that almost always exist, licensing reasons, to create a reproducible environment on a variety of different development platforms.. and attempt to curtail the plethora of Cygwin/MinGW/DJGPP users asking dumb questions. So, is that reason enough for ya? --[[User:Brynet-inc|Brynet-Inc]] 06:04, 23 April 2011 (UTC)
::The first reason makes the most sense. Alas, I don't really know what kind of patches are usually there and how they affect code generation and/or linking---perhaps I should look into this. Could you give some examples for the other two reasons? I'm not 100% sure what you had in mind. --[[User:Love4boobies|Love4boobies]] 14:51, 23 April 2011 (UTC)
:::Well, GCC 4.2.1 for example.. it's the last GPLv2 version and contains several code generation bugs that had to be independently fixed, not only that, but the fixes aren't always identical between forks. Also, for someone running on Linux.. they may not wish to use the particular version of GCC that's available. It makes sense to pick a version of GCC/binutils that you've tested extensively for building your kernel/OS, as well as making that environment reproducible on Windows/Linux/OSX/BSD/etc. --[[User:Brynet-inc|Brynet-Inc]] 19:17, 23 April 2011 (UTC)
::::I'm not sure using all this indentation is helpful :) Anyway, the GPL license applies to GCC, not code generated by it, so as far as the OS is concerned, this isn't a problem. As for the other thing, I thought it might be what you were thinking and it makes sense to some extent but I can't think of any specific problems that I've come across when using this technique... --[[User:Love4boobies|Love4boobies]] 04:37, 24 April 2011 (UTC)
::::::::::::::::Indeed, one mustn't abuse indentation. Also, the license covers distribution.. and with the introduction of the GPLv3 some additional gotchas. If you plan on including a compiler along with your OS, for example. It makes sense to eventually port the cross-compiler environment to your OS as a native hosted environment. Just trust me on this one. --[[User:Brynet-inc|Brynet-Inc]] 20:03, 24 April 2011 (UTC)

:For the record, only 32-bit linux is the actively used platform which shares architecture and executable format with "common" OS development. 64-bit linux comes with an amd64 toolchain, Windows and Macintosh comes with a different binary format - each of them needs very specific workarounds to get the desired behaviour if possible at all. On top of that, all of the hosted compilers might fail now or in the future because they target a specific OS and are therefore freely allowed to generate OS-specific code. Excluding the notion of doing things the right way, which you obviously do not promote, your suggestion requires that '''portable''' means 32-bits linux only instead of all common desktop installations out there.</sarcasm> - [[User:Combuster|Combuster]] 21:40, 24 April 2011 (UTC)

::In the first part of your reply, you just repeated what I said (i.e., that cross-compilers are useful for different target architectures and/or file formats) and assumed you were making some point. No one argued against building a cross-compiler in that scenario; I only complained because the article instructs people to always do this, which is unnecessary. In the second part of your reply, you talked about hosted compilers and completely missed what I said about the freestanding functionality, which is present with any GCC port (RTFM, in this case, the GCC documentation), followed by more of your usual gibberish. --[[User:Love4boobies|Love4boobies]] 22:15, 24 April 2011 (UTC)

:::It is almost always necessary, you really shouldn't just build your OS with whatever is shipped with your 32-bit Linux distribution.. as stated many times on the forums, you'll run into problems eventually.. and you'll be surprised when an update introduces unexpected behaviour. --[[User:Brynet-inc|Brynet-Inc]] 22:28, 24 April 2011 (UTC)

::::Well, at least there's an argument on the table now so let's focus on that one. I've successfully used MinGW, Cygwin GCC, Linux GCC and clean cross-compilers and had problems with neither so I wonder whether we're being realistic in the article. As per the GCC documentation, the freestanding environment implies that there's no standard library linked in and no system-specific startup code present. Could anyone point out any reproducible problems, on the forums or anywhere else? Otherwise, you could guess, I could guess but we're not really going to get anywhere :) --[[User:Love4boobies|Love4boobies]] 22:42, 24 April 2011 (UTC)

:::::I give you the OpenBSD manual that briefly touches on some of the changes (..beyond bug fixes) to their version of GCC 4.2.1, as an example. [http://www.openbsd.org/cgi-bin/man.cgi?query=gcc-local&manpath=OpenBSD+Current&format=html gcc-local(1)] --[[User:Brynet-inc|Brynet-Inc]] 01:59, 25 April 2011 (UTC)

::First off, I regret the ad hominem you had to use. Yes I know exactly what -ffreestanding is meant to do, and I also know its not enough, thanks to references to the always platform-specific libgcc. Point was that the cases where the hack of not building a crosscompiler only works on a limited '''and shrinking''' number of platforms. Now to return the favour, '''will you quit the arguments about ugly hacks being better than proper use because they happen to be marginally simpler'''. I hate those and they only really help those who are already aware of the reasons of why you would do things one way. We are out here to help mostly newcomers, and providing them broken information and dirty workarounds is counterproductive in the long run. 
::I want to close this up and call it a 3v1 vote. I'm including solar besides the active participants here because he wrote the original text and has made it clear that using a crosscompiler '''rules out''' any possible host-compiler-related problems rather than just making them unlikely. And debugging is about testing assertions, not guesswork.
::- [[User:Combuster|Combuster]] 09:23, 25 April 2011 (UTC)

:::If you'll read more carefully, you'll see that there was no ad hominem in my reply, regardless of which one you are referring to (you've either read just half of one phrase or considered something to be an argument when it wasn't). Anyway, can you point out any problems for platform X or Y yet so as to shut me up? The reason for which I don't think this is a hack, but a solution, is that I have yet to come across any. Brynet's link seems to point out that OpenBSD's port changes some arguments and some other stuff but it shouldn't really cause any problems, esp. if one is going to write proper standards-compliant code. I expect that ''maybe'' sometimes people will need to use some different compiler/linker flags to get the thing to work, but does that make it a hack? Voting is nice but I prefer facts. --[[User:Love4boobies|Love4boobies]] 13:43, 25 April 2011 (UTC)

:::There is potential for problems, nobody has said that what you're doing won't work.. it's just not guaranteed to work, you can go around licking public surfaces and not get sick.. doesn't mean you're not going to.. get hit by a bus. --[[User:Brynet-inc|Brynet-Inc]] 23:54, 25 April 2011 (UTC)

== What *should* be on this page... ==

The fact that Love4Boobies didn't understand the need for a cross-compiler shows that this page does a poor job of answering the question.

Things I feel are missing from the page (which might satisfy Love4Boobies, and which perhaps someone would like to insert as appropriate - I don't have the leisure ATM):

* Building a cross-compiler is merely the first step towards "going native", i.e. a step you have to do sooner or later anyway. You might as well do it right at the beginning.
* The moment you go forward to the next step - putting YourOS-specific files like libc headers, startup code etc. in the compiler directories - you '''need''' to seperate between system compiler and OSDev compiler, or you'll end up with a disfunctional system compiler.
* Your system compiler is subject to being updated by your package management without specific notice. Depending on your distro, locking a specific version, or going back to an older version, might be anything but trivial (if at all possible without breaking your system). Worse, you might not even know which version it ''was'' that actually worked (before the latest update), and you might have depended on something distro-specific before, leaving your development environment in complete limbo.
* As the recommended thing to do, it puts everyone following the recommended practice on similar footing. No comparing of (vanilla) gcc-4.5.2 with (Gentoo) gcc-4.5.2-r1 or (Ubuntu) gcc-4.5.2-1ubuntu3 or (Cygwin) gcc4-4.5.2-2 or somesuch.
-- [[User:Solar|Solar]] 09:15, 2 May 2011 (UTC)


For the record, I completely agree with Love4Boobies and completely disagree with Solar. Building a crosscompile toolchain for later work is solely useful to cut down on the extra arguments needed to be given. You can achieve the same result with -ffreestanding -nostdlib -nostdlibinc (for the kernel) and --sysroot for usermode apps.

Not to mention that recompiling the compiler is a GCC-ism. Clang doesn't require it, it is naturally a crosscompiler, in which case you use --sysroot as well (until someone fixes the compiler driver to support dynamic retargetting, which is on my agenda but hasn't been done yet) -- JamesM


== ("Tongue in Cheek") Suggestion For Rewritten Page Introduction ==

The suggested new introduction for this page is:

"''Even though C is a well known language that has been around since 1973, and GCC is a well known toolchain that has been around since 1987 and is relied on by many major open source projects; some people think GCC is so utterly broken that the sheer hassle of creating a cross-compiler is not only a justified step but necessary step, to shield developers from the complete and utter horror of GCC's brokenness. However, some people think that GCC actually does work (for at least one definition of "works") and that GNU developers and the open source community as a whole actually aren't completely incompetent. These people don't agree that creating a cross-compiler is justified or necessary. A third group of people are more practical and have the opinion that if GCC actually is so broken that a cross-compiler is necessary, then hiding symptoms isn't a cure and you should probably just use NASM instead.''" --[[User:Brendan|Brendan]] 12:57, 19 March 2012 (CDT)

== Development without a cross-compiler? ==

I would like to give a perspective from a os-dev newcomer who has been using the this and wiki pages to get started with kernel developement.  I do not feel mature enough to take side, but for me it would have been helpfull if the article would state something like "It is possible, under circumstances xyz, not to use a crosscompiler or post-pone it for later. Then flags xyz will be necessary."  And then stress again that this is not recommanded.  Maybe also point to the discussion page for details why or why not.

I downloaded a couple of available "hobby OSes" (e.g. geekos and prettyos) and compiled them without problem using my i686-linux-gnu gcc and they run.  I'm not trying to say that it is a good idea not to use a cross compiler.  I'm saying that to me this article in the current version gave the impression that it is simply impossible without build cross-compiler; and I was confused when it actually worked. -- Tang

:Hi, this kind of discussion is probably more suited for the forums. Nonetheless, I'll try to answer your questions here. The main reason you need a cross-compiler is because the <tt>gcc</tt> that came with your distribution targets your current system. For instance, a Linux gcc will search for headers in <tt>/usr/include</tt> and libraries in <tt>/usr/lib</tt>. Not only that, it will use the Linux executable format (ELF) and the generated code will assume the Linux ABI. Indeed, the compiler will declare Linux-specific preprocessor macros. It will assume instruction set extensions are enabled. (Even the libgcc assumes it will run on a Linux platform and contains references to stuff like pthreads, if I'm not wrong). It might even generate code for the wrong architecture if you wish to make a 32-bit kernel on a 64-bit system.
:In other words, the root cause is that the compiler thinks it targets a Linux system (in this example, but substitute whatever you use) and all the problems you will experience are caused by this. You implicitly lied to the compiler and told it that it targets Linux - when it really doesn't - and it might do bad things. This article mentions a number of compiler options that can change the behaviour to disable some of the symptoms, but it doesn't correct the root problem: Your OS is ''not'' Linux. You can use the compiler options, but the result will be a fragile toolchain that is harder to use. Indeed, you can't even fully pass command line options that undo all the Linux assumptions. You can't even use freestanding headers like <tt><stdint.h></tt> in your kernel as on GNU libc platforms this header is provided by GNU libc, but as you pass -nostdinc you disable both /usr/include and the compiler specific include directory.
:That said, you can provide a bunch of compiler options that gradually treat whatever symptoms would appear and it will likely work if you are skilled - but you won't gain the reliably of a target like i686-elf where the compiler never makes any of these bad assumptions in the first place. It is likely possible to provide a list of options that would fix most issues people would encounter, but then there's additional problems caused by the fact that distributions further patch the system compiler with further enhancements and change compiler option defaults - even the fact that they may need to pass options like -m32 (which doesn't make sense if cross-developing from, say, ARM) causes much complexity.
:So yes, it's much simpler to just do things ''properly'' and cross-compile to a target like i686-elf. Indeed, most people that would use the advise on how to avoid using a cross-compiler are likely lazy and will complain when things break - this increases the support burden for our community. Besides, requiring a bit of patience from newcomers to build a cross-compiler isn't unreasonable considering the time it takes to write an OS - it's even educational to know how to compile GCC. --[[User:Sortie|Sortie]] 14:52, 4 January 2014 (CST)


:I noted in a comment earlier that "portable" seems to be defined as "32-bit linux only". While this wouldn't be completely true, the hacks you need to force specific compilers to do the things you expect them to do can often differ between setups, have some issues that can't be patched, or just can't be made to work at all in several cases. And these settings change as GCC goes to newer versions. Basically what you request consists of two things:
:# Post ''and maintain'' hacks for all possibilities.
:# Post instructions with the intent that they're not being performed.
:The former is a waste of time, the second is just silly. Also note that building the cross-compiler is in several people's vision the equivalent of the entrance test because it implies the minimal reading skills many people lack. Getting caught not using one is in practice just an excuse for being flamed for your stupidity, and not getting any further help until you "fix" the problem. So you're better off doing it right the first time anyway. 
: As for the article, I'll always stick with the established psychology that tells you to teach people the things to do, rather than the things ''not'' to do because the latter only gives people ideas. Basically, every non-cross-compiler reference to GCC in the wiki is to be considered an error.
: - [[User:Combuster|Combuster]] 10:14, 7 January 2014 (CST)
````

<a id="w002977"></a>

## W002977 — File:Practical File System Design.jpg

Bron-ID: W002977
Namespace: File (6)
Revisie: 11478 | Datum: 2011-05-10T09:02:48Z
Bron: https://wiki.osdev.org/File:Practical_File_System_Design.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3APractical_File_System_Design.jpg&oldid=11478

````text

````

<a id="w002979"></a>

## W002979 — Internet Control Message Protocol

Bron-ID: W002979
Namespace: Main (0)
Revisie: 11490 | Datum: 2011-05-11T04:42:47Z
Bron: https://wiki.osdev.org/Internet_Control_Message_Protocol
Revisie-URL: https://wiki.osdev.org/index.php?title=Internet_Control_Message_Protocol&oldid=11490

````text
Internet Control Message Protocol, commonly referred to as ICMP, is one of the fundamental protocols in a typical [[IPv4]]-capable network stack. The protocol gives nodes on a network the ability to share information and errors without utilising an upper-layer protocol which may have been compromised. For example, if a TCP connection is having packets dropped due to their time-to-live being too low, there is no way for either host to determine the fault. The only way either host can resolve the problem is if an out-of-band information message is transmitted to them, informing them that their time-to-live is too low and a router is dropping the packets. This is the purpose of ICMP.

Most people will have used ICMP at least once indirectly, by utilising the "ping" tool of their favourite operating system.

== RFC ==

ICMP is defined in [http://tools.ietf.org/html/rfc792 RFC792]. [http://tools.ietf.org/html/rfc1122 RFC1122] defines the relationship between [[IPv4]] and ICMP.

== Packet Layout ==

Note that all structures in this article are intended to show the layout of a packet in memory rather than be usable as C code (although most of them should be able to be copied verbatim).

An example C structure for an ICMP packet is:

<source lang="c">
struct icmp_packet {
    uint8_t icmp_type;
    uint8_t icmp_code;
    uint16_t checksum;
};
</source>

== Checksum ==

An ICMP checksum is performed over the header and data in the same manner as an [[IPv4]] checksum. There is no IPv4 psuedo-header required.

Essentially, the algorithm is as follows:

<pre>
1. Add all 16-bit words in the data to calculate a checksum for. This should be done in a 32-bit counter.
2. Add any stray bytes in the case of an uneven length.
3. Fold the sum into one 16-bit word (add the top 16 bits to the bottom 16 bits until the top 16 bits are zero).
4. Return the 16-bit 1's complement of the result.
</pre>

When verifying an incoming packet, you should perform the checksum over the header and data without modifying any data in the packet. Your calculated checksum will be zero if the checksum is valid, and non-zero if the checksum is invalid.

== ICMP Types ==

There are a variety of message types defined for ICMP.

=== Echo Request & Reply ===

These packets have an additional header after the ICMP header with information related to echo replies and requests:

<source lang="c">
struct icmp_echo_packet {
    uint16_t echo_id;
    uint16_t echo_seq;
};
</source>

Following this header is an arbitrary length of data.

The ICMP code for an echo request or reply is always zero.

==== Echo Request (type 8) ====

An echo request is sent to a host in order to request an "Echo Reply" response. These are typically used to determine if a host on the network is reachable, or to determine latency in a link or route.

The ID and Sequence values can be set to an arbitrary value, but should be meaningful to the implementation sending the echo request.

The packet data should be a number of bytes (typically enough to pad the packet to 64 or so bytes) that you expect to receive back from the host. Windows seems to just use the lower-case alphabet from 'a' to 'z' - it really is up to you what data goes here!

==== Echo Reply (type 0) ====

An echo reply is sent to a host in response to an "Echo Request" message. All features of the reply are the same as the echo request, except for the destination and source addresses. The ID and Sequence values '''must not be changed'''. These are how a received response is tied to a previous request.

[[Category:Network Protocols]]

=== Destination Unreachables (type 3) ===

These packets are sent by a host or router to notify another host that the destination it is attempting to send packets to is unreachable, for a variety of possible reasons.

The packet format is as follows:
<source lang="c">
struct dest_unreachable_packet {
    uint16_t empty;
    uint16_t next_mtu;
    iphdr original_header;
    uint8_t original_payload[8]; // First eight bytes of the original IPv4 payload.
};
</source>

Note that it may be better to ''not'' define your structure containing an IP header structure and these extra bytes depending on your operating system and individual needs.

The '''next_mtu''' field contains the MTU for the next hop if this message is a Fragmentation Required message.

A host uses the original IP header transmitted in the destination unreachable packet to determine which packet failed. This is a good reason to use valid and tracked IDs in your [[IPv4]] packets.

Destination Unreachable messages use the ICMP code field rather than the ICMP type field to define individual message types.

==== Framentation Required (code 4) ====

A host or router sends this packet to a host when the packet is larger than the MTU for a link, and the "Don't Fragment" (DF) bit in the [[IPv4]] header is set. Were the DF bit to be unset, the router or host would fragment the packet automatically in order to transmit it.

One common use of this message is in Path MTU Discovery, which determines the MTU for a complete route from one host to another by sending out packets with decreasing size (with DF set) until a Fragmentation Required message does not get sent for the packet. Rather than "guess-and-check", a host should use the "Next MTU" field of the destination unreachable header to determine the next MTU to attempt. Keep this in mind when writing code to route packets; you may need to implement the infrastructure required to handle this (very important) message.
````

<a id="w002984"></a>

## W002984 — ATA read/write sectors

Bron-ID: W002984
Namespace: Main (0)
Revisie: 20723 | Datum: 2017-04-19T05:42:05Z
Bron: https://wiki.osdev.org/ATA_read/write_sectors
Revisie-URL: https://wiki.osdev.org/index.php?title=ATA_read%2Fwrite_sectors&oldid=20723

````text
Since interrupts (like INT 13h) can't be called easily in protected mode or long mode, direct disk access through ports might be the only solution. The below code is example of subroutines to read and write disk sectors directly from the first hard disk (80h) in long mode using CHS and LBA.

=ATA read sectors=

==Read in CHS mode==
Accessing disk using CHS (cylinder,head,sector) indexes seem to be kinda old type but is the base for LBA access. The following NASM long-mode subroutine reads CH sectors to memory address indicated by RDI register.
<source lang="asm">
;=============================================================================
; ATA read sectors (CHS mode) 
; Max head index is 15, giving 16 possible heads
; Max cylinder index can be a very large number (up to 65535)
; Sector is usually always 1-63, sector 0 reserved, max 255 sectors/track
; If using 63 sectors/track, max disk size = 31.5GB
; If using 255 sectors/track, max disk size = 127.5GB
; See OSDev forum links in bottom of [http://wiki.osdev.org/ATA]
;
; @param EBX The CHS values; 2 bytes, 1 byte (BH), 1 byte (BL) accordingly
; @param CH The number of sectors to read
; @param RDI The address of buffer to put data obtained from disk               
;
; @return None
;=============================================================================
ata_chs_read:   pushfq
                push rax
                push rbx
                push rcx
                push rdx
                push rdi
                
                mov rdx,1f6h            ;port to send drive & head numbers
                mov al,bh               ;head index in BH
                and al,00001111b        ;head is only 4 bits long
                or  al,10100000b        ;default 1010b in high nibble
                out dx,al

                mov rdx,1f2h            ;Sector count port
                mov al,ch               ;Read CH sectors
                out dx,al

                mov rdx,1f3h            ;Sector number port
                mov al,bl               ;BL is sector index
                out dx,al

                mov rdx,1f4h            ;Cylinder low port
                mov eax,ebx             ;byte 2 in ebx, just above BH
                mov cl,16
                shr eax,cl              ;shift down to AL
                out dx,al

                mov rdx,1f5h            ;Cylinder high port
                mov eax,ebx             ;byte 3 in ebx, just above byte 2
                mov cl,24
                shr eax,cl              ;shift down to AL
                out dx,al

                mov rdx,1f7h            ;Command port
                mov al,20h              ;Read with retry.
                out dx,al
                
.still_going:   in al,dx
                test al,8               ;the sector buffer requires servicing.
                jz .still_going         ;until the sector buffer is ready.

                mov rax,512/2           ;to read 256 words = 1 sector
                xor bx,bx
                mov bl,ch               ;read CH sectors
                mul bx
                mov rcx,rax             ;RCX is counter for INSW
                mov rdx,1f0h            ;Data port, in and out
                rep insw                ;in to [RDI]

                pop rdi
                pop rdx
                pop rcx
                pop rbx
                pop rax
                popfq
                ret
</source>

==Read in LBA mode==
{{stub}}
<source lang="asm">
;=============================================================================
; ATA read sectors (LBA mode) 
;
; @param EAX Logical Block Address of sector
; @param CL  Number of sectors to read
; @param RDI The address of buffer to put data obtained from disk
;
; @return None
;=============================================================================
ata_lba_read:
               pushfq
               and rax, 0x0FFFFFFF
               push rax
               push rbx
               push rcx
               push rdx
               push rdi

               mov rbx, rax         ; Save LBA in RBX
               
               mov edx, 0x01F6      ; Port to send drive and bit 24 - 27 of LBA
               shr eax, 24          ; Get bit 24 - 27 in al
               or al, 11100000b     ; Set bit 6 in al for LBA mode
               out dx, al

               mov edx, 0x01F2      ; Port to send number of sectors
               mov al, cl           ; Get number of sectors from CL
               out dx, al
               
               mov edx, 0x1F3       ; Port to send bit 0 - 7 of LBA
               mov eax, ebx         ; Get LBA from EBX
               out dx, al

               mov edx, 0x1F4       ; Port to send bit 8 - 15 of LBA
               mov eax, ebx         ; Get LBA from EBX
               shr eax, 8           ; Get bit 8 - 15 in AL
               out dx, al


               mov edx, 0x1F5       ; Port to send bit 16 - 23 of LBA
               mov eax, ebx         ; Get LBA from EBX
               shr eax, 16          ; Get bit 16 - 23 in AL
               out dx, al

               mov edx, 0x1F7       ; Command port
               mov al, 0x20         ; Read with retry.
               out dx, al

.still_going:  in al, dx
               test al, 8           ; the sector buffer requires servicing.
               jz .still_going      ; until the sector buffer is ready.

               mov rax, 256         ; to read 256 words = 1 sector
               xor bx, bx
               mov bl, cl           ; read CL sectors
               mul bx
               mov rcx, rax         ; RCX is counter for INSW
               mov rdx, 0x1F0       ; Data port, in and out
               rep insw             ; in to [RDI]

               pop rdi
               pop rdx
               pop rcx
               pop rbx
               pop rax
               popfq
               ret
</source>

=ATA write sectors=

A write is mostly equivalent to performing a read operation. The only changes needed are a change in command (0x30 for chs write), and the direction of the data, which is written (rep outsw from *SI) to the data port rather than read (rep insw to *DI).

<source lang="asm">
;=============================================================================
; ATA write sectors (LBA mode) 
;
; @param EAX Logical Block Address of sector
; @param CL  Number of sectors to write
; @param RDI The address of data to write to the disk
;
; @return None
;=============================================================================

ata_lba_write:
    pushfq
    and rax, 0x0FFFFFFF
    push rax
    push rbx
    push rcx
    push rdx
    push rdi
    
    mov rbx, rax         ; Save LBA in RBX

    mov edx, 0x01F6      ; Port to send drive and bit 24 - 27 of LBA
    shr eax, 24          ; Get bit 24 - 27 in al
    or al, 11100000b     ; Set bit 6 in al for LBA mode
    out dx, al

    mov edx, 0x01F2      ; Port to send number of sectors
    mov al, cl           ; Get number of sectors from CL
    out dx, al

    mov edx, 0x1F3       ; Port to send bit 0 - 7 of LBA
    mov eax, ebx         ; Get LBA from EBX
    out dx, al

    mov edx, 0x1F4       ; Port to send bit 8 - 15 of LBA
    mov eax, ebx         ; Get LBA from EBX
    shr eax, 8           ; Get bit 8 - 15 in AL
    out dx, al


    mov edx, 0x1F5       ; Port to send bit 16 - 23 of LBA
    mov eax, ebx         ; Get LBA from EBX
    shr eax, 16          ; Get bit 16 - 23 in AL
    out dx, al

    mov edx, 0x1F7       ; Command port
    mov al, 0x30         ; Write with retry.
    out dx, al

.still_going:  in al, dx
    test al, 8           ; the sector buffer requires servicing.
    jz .still_going      ; until the sector buffer is ready.

    mov rax, 256         ; to read 256 words = 1 sector
    xor bx, bx
    mov bl, cl           ; write CL sectors
    mul bx
    mov rcx, rax         ; RCX is counter for OUTSW
    mov rdx, 0x1F0       ; Data port, in and out
    mov rsi, rdi
    rep outsw            ; out

    pop rdi
    pop rdx
    pop rcx
    pop rbx
    pop rax
    popfq
    ret
</source>

=See also=
* [http://forum.osdev.org/viewtopic.php?t=12268 Read/write disk sectors by Dex]
* [[ATA_PIO_Mode|ATA PIO mode]]
* [[ATA]]

[[Category:ATA]]
````

<a id="w002985"></a>

## W002985 — Deprecated

Bron-ID: W002985
Namespace: Main (0)
Revisie: 20311 | Datum: 2017-02-01T11:33:01Z
Bron: https://wiki.osdev.org/Deprecated
Revisie-URL: https://wiki.osdev.org/index.php?title=Deprecated&oldid=20311

````text
This page aims to cover links to information that has been removed from various documents but which may still be relevant for the purpose of backwards compatibility. It does not explain which technologies deprecate which, only variations between versions of a particular technology.

== Intel SDM ==

{| {{wikitable}}
|-
! Item
! Where to find
! Comments
|-
| Intel486/Pentium test registers (TR3..TR7)
| Pentium MMX SDM (description, no instruction encodings)
IA-32 SDM Vol. 2B Sept. 2005 (instruction encodings)
| Cache testing is performed via MSRs on newer CPUs.
|}

== UEFI specification ==

{| {{wikitable}}
|-
! Item
! Where to find
! Comments
|-
| Device I/O Protocol
| EFI 1.10 specification
| Replaced by PCI Root Bridge I/O protocols.
|-
| UGA I/O + UGA Draw Protocol
| EFI 1.10 specification
| Replaced by EFI Graphics Output Protocol.
|-
| USB Host Controller Protocol
| EFI 1.10 specification
| Replaced by USB2 Host Controller Protocol.
|-
| SCSI Passthru Protocol
| EFI 1.10 specification
| Replaced by Extended SCSI Passthru Protocol.
|-
| Driver Configuration Protocol
| UEFI 2.0 specification (including versions with errata applied)
| Removed in favour of a different mechanism.
|}

== SMBIOS specification ==

{| {{wikitable}}
|-
! Item
! Where to find
! Comments
|-
| Plug-and-Play function interface
| SMBIOS 2.6.1 specification
| Replaced by a table-based method.
|}

[[Category:Resources]]
````

<a id="w002986"></a>

## W002986 — Talk:ATA read/write sectors

Bron-ID: W002986
Namespace: Talk (1)
Revisie: 11551 | Datum: 2011-05-23T17:54:32Z
Bron: https://wiki.osdev.org/Talk:ATA_read/write_sectors
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AATA_read%2Fwrite_sectors&oldid=11551

````text
= Various issues =
Several things that annoy me about this page:
* The code requires longmode
* The code does not check for error messages
* The code does not document the input properly - it can't be used as-is
* The code does not document several magic numbers, and it's hardwired for a specific drive.
* There already exists several ATA pages, including sample code. Why do we need a new one?

Merge with ATA? Discard the page altogether? Rewrite the code? - [[User:Combuster|Combuster]] 09:14, 23 May 2011 (UTC)

It's much worse. Apparently, we have a whole bunch of ATA articles:

* [[IDE]]
* [[ATAPI]]
* [[ATA read/write sectors]]
* [[ATA PIO Mode]]
* [[ATA/ATAPI using DMA]]
* [[ATA/ATAPI Power Management]]
* [[ATA in x86 RealMode (BIOS)]]

and reading/writing is covered in quite a few places, one way or another. There's a lot duplicated information so, except for the last article on the list, all of them should probably be merged. I doubt it'll be very easy though, it looks like a lot of work---I did the significantly easier job of merging two articles into [[Detecting Memory (x86)]] in the past and it still took me quite a while; maybe we should just delete all of them and start fresh (tounge in cheek).

The CPU operating mode-specific code is indeed another problem; C should be used instead. It's not the only article like this :( --[[User:Love4boobies|Love4boobies]] 16:34, 23 May 2011 (UTC)

: Regarding the whole lot: the entire ATA set is a huge volume of text, and for overview it's probably a good reason to keep the pages a separate. That said, IDE is a disputed and shares a lot with the [[ATA PIO Mode]] page as well as having some information missing in the other. The problem is, both pages together have the full set of information, but IDE also has some stuff that's actually wrong. A merge has been proposed before on [[Talk:IDE]], and there's a [http://forum.osdev.org/viewtopic.php?f=8&t=22508 forum thread]. Now it appears that we have to merge even more -_-
:  - [[User:Combuster|Combuster]] 17:44, 23 May 2011 (UTC)
: P.S. I pointed the ATA article to the category (instead of the broken page) and updated the indexing a bit in the fashion it was probably intended to be.

=Deletion=
Although I created that article. I vote for a deletion.
[[User:nicola|Nicola]]

I vote for a merge in the long run, provided there is new information here. --[[User:Love4boobies|Love4boobies]] 17:12, 23 May 2011 (UTC)

There is hardly any ''new'' information here, which brings me to the question: why did you create it? Apparently the existing content was not sufficient for any reason? - [[User:Combuster|Combuster]] 17:44, 23 May 2011 (UTC)

Just wanted to make it as a 'fastfood' example for those guys successfully entered long mode. - [[User:nicola|Nicola]]
````

<a id="w002987"></a>

## W002987 — Talk:Deprecated

Bron-ID: W002987
Namespace: Talk (1)
Revisie: 11561 | Datum: 2011-05-27T07:50:17Z
Bron: https://wiki.osdev.org/Talk:Deprecated
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ADeprecated&oldid=11561

````text
Do you think the information herein should be moved to the articles that talk about the relevant specifications? While that does make a lot of sense, the reasoning behind creating this new article was that I suspected that there might be other people like me who only rely on their own references rather than the wiki. For us, having an article that tells us what information we are missing is much more useful. --[[User:Love4boobies|Love4boobies]] 07:50, 27 May 2011 (UTC)
````

<a id="w002988"></a>

## W002988 — Talk:ATA/ATAPI using DMA

Bron-ID: W002988
Namespace: Talk (1)
Revisie: 11566 | Datum: 2011-06-03T13:27:05Z
Bron: https://wiki.osdev.org/Talk:ATA/ATAPI_using_DMA
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AATA%2FATAPI_using_DMA&oldid=11566

````text
Chrome warns about the link to the Intel manual in the "External Links" section is hosted on a site that contains malicious software, possibly re-host it somewhere else? --[[User:Tobba|Tobba]] 12:55, 3 June 2011 (UTC)
: Report the bug to chrome, or, disable such silly warnings and use your own judgment; I see nothing "malicious" about where it's hosted. --[[User:Brynet-inc|Brynet-Inc]] 13:27, 3 June 2011 (UTC)
````

<a id="w002989"></a>

## W002989 — User:Yargh

Bron-ID: W002989
Namespace: User (2)
Revisie: 11939 | Datum: 2011-08-21T02:22:07Z
Bron: https://wiki.osdev.org/User:Yargh
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AYargh&oldid=11939

````text
Hey look, I have a wiki page!
````

<a id="w002990"></a>

## W002990 — Windows Tools

Bron-ID: W002990
Namespace: Main (0)
Revisie: 22685 | Datum: 2018-09-10T01:09:53Z
Bron: https://wiki.osdev.org/Windows_Tools
Revisie-URL: https://wiki.osdev.org/index.php?title=Windows_Tools&oldid=22685

````text
Windows provides some useful utilities for using [[Loopback Device|loopback devices]] and manipulating files that OS developers should know about; most of the time they are unaware of this and thus download tools that give them functionality they already had. This page attempts to describe the Windows utilities and how they can be used.

Note: Wherever you see a drive letter followed by a semicolon in the examples, remember that you can just as well specify a GUID instead.

== copy ==

Except for its obvious purpose, the '''copy''' utility is also useful for concatenating files; this can come in handy when producing disk images, for instance. To illustrate the point, suppose we wanted to produce a file called ''foobar'' from files ''foo'' and ''bar''; we would use the following command:

<source lang="dos">copy /b foo+bar foobar</source>

The '''/b''' switch indicates that we are dealing with binary files instead of ASCII. We need to specify it as this is behavior is not default when combining files.

== diskcomp ==

The '''diskcomp''' tool can be used to compare all tracks of two floppy disks. Example:
 diskcomp A: B:
* If only one drive is specified, it is compared to the current drive.
* If no drives are specified, the same drive will be used for both floppies (you will be prompted to change them).

{| {{wikitable}}
|-
| errorlevel value
| Meaning
|-
| 0
| The contents of the two disks are identical.
|-
| 1
| The contents of the two disks are not identical.
|-
| 3
| An error has occurred.
|-
| 4
| Insufficient memory to perform the comparison.
|}

== diskcopy ==

DOS users will remember '''diskcopy''', which replicates volumes. It only works with removable media such as memory cards, and floppy disks. The media must be of the same type for the operation to work. E.g., if we had two floppy drives A:, and B:, we could copy A:'s image to B: as such:

<source lang="dos">diskcopy A: B:</source>

* If the destination drive is not specified, the current one will be used. In addition, the '''/v''' argument can be used for verification (remember, floppy disks are not exactly reliable).
* If both drives are omitted, you will be prompted to replace the media after the first one has been read.

== diskpart ==
:''Extended article: [[Windows Loopback Alternative]]''

The '''diskpart''' utility is useful for managing partitions and virtual hard drives, which can be used as loopback devices. If you want other types of loopback devices, you might want to consider [[Virtual Floppy Drive|VFD]] for floppy drives, and [[DAEMON Tools]] for optical media.

The file format used by this tool is called VHD (Virtual Hard Disk) and can be used with several emulators/VMs, including [[VirtualBox]], [[Microsoft Virtual PC]], [[Xen]], and [[VMware]].

This isn't your usualy command-line utiliy: you run it from the command prompt, after which you query commands.

In order to create a VHD image that is 10 MiB in size, you can use the following command:
 create vdisk file="C:\foo.vhd" maximum=10 type=expandable
There are a few things worth noting:
* The utility expects you to specify full paths.
* When the '''maximum''' argument is omitted, the default size is set to '''24''' GiB. When it is present, a valid hard disk size is needed.
* If not type is specified, the default is set to '''fixed'''. Expandable images grow in size as needed, while fixed ones do not.
* The '''source="bar.vhd"''' argument can be used in order to intialize the contents of the new image with ''bar.vhd''.
Most operations require one to shift focus to the virtual disk beforehand. Creating a VHD automatically shifts focus to it but when working with multiple images, the '''select''' command is useful:
 select vdisk file="C:\foo.vhd"
Although expandable images grow in size, they do not also automatically reduce in size. To manually reduce the size of the selected virtual disk:
 compact vdisk
Here's an example of how to change the maximum size of the selected VHD to 20 MiB:
 expand vdisk maximum=20
The following attaches the selected VHD (similarly to plugging a HDD in):
 attach vdisk
* The '''readonly''' argument can be used; its purpose is obvious.
After attaching a virtual disk, you need to create a partition for it and then mount it (i.e., assign it a letter):
 create partition primary
 assign letter=E
Finally, the following unmounts and then detaches the selected VHD:
 remove letter=E
 detach vdisk

== fsutil ==

The '''fsutil''' utility can be used to perform all sorts of [[NTFS]] and [[FAT]] operations, including file manipulation. Of these, the following are useful for OS developers:

{| {{wikitable}}
|-
! Subcommand
! Parameter
! Description
|-
| rowspan="2" | 8dot3name
| query || This can be used in order to find out whether a volume (which may very well be a loopback device) is configured to use the 8.3 file naming scheme. Example:
<source lang="dos">fsutil 8dot3name query C:</source>
If no volume is specified, information on all volumes is displayed. 
|-
| set || This option can set one or all volumes to (not) use the 8.3 file name scheme. Before this can be used on a per volume basis, the registry key '''HKLM\System\CurrentControlSet\Control\FileSystem\NtfsDisable8dot3NameCreationNtfsDisable8dot3NameCreationNtfsDisable8dot3NameCreation''' must be set to the value 2, if this is not already true. This can be done by using the following command:
<source lang="dos">fsutil 8dot3name set 2</source>
Next, we are free to play around with individual volumes:
<source lang="dos">fsutil 8dot3name set C: 0</source>
The last parameter can take the value 0 (enable) or 1 (disable). This is somewhat odd; the reason behind it is probably that Microsoft, in order to avoid confusion, decided to preserve these values, which are also used in a similar command.
|-
| rowspan="4" | file
| createnew || Creates an empty file (all zeroes) of some size specified, in bytes. Example:
<source lang="dos">fsutil createnew foobar 42</source>
|-
| setshortname || Sets the 8.3 name for a file (while preserving its long name). Example:
<source lang="dos">fsutil setshortname foobarbaz fooba~1</source>
|-
| setvaliddata || Sets the VDL for a file. Example:
<source lang="dos">fsutil setvaliddata foobar 4096</source>
|-
| setzerodata || Sets one or more bytes in a file to zero. Example:
<source lang="dos">fsutil setzerodata foobar offset=42 length=42 foobar</source>
|}

[[Category:Tools]]
[[Category:Windows]]
````

<a id="w002991"></a>

## W002991 — User:Sanmukh

Bron-ID: W002991
Namespace: User (2)
Revisie: 11605 | Datum: 2011-06-09T16:35:56Z
Bron: https://wiki.osdev.org/User:Sanmukh
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASanmukh&oldid=11605

````text
I am K. Sanmukh Rao. A Bachelor of Technology in Computer Science from IIT Guwahati. I am currently working Adobe Systems, Noida.
````

<a id="w002992"></a>

## W002992 — Visual C++ Runtime

Bron-ID: W002992
Namespace: Main (0)
Revisie: 28135 | Datum: 2023-07-10T02:51:55Z
Bron: https://wiki.osdev.org/Visual_C%2B%2B_Runtime
Revisie-URL: https://wiki.osdev.org/index.php?title=Visual_C%2B%2B_Runtime&oldid=28135

````text
Since you can't link standard C++ runtime to your kernel, you'll need several functions to replace it's functionality. This article provides information on how to implement your own C++ runtime for Visual C++ compiler.


==Call Constructors For Global Static Variables==
This code will help to call all constructors for global static variables.

<source lang="c">// Constructor prototypes
typedef void (__cdecl *_PVFV)(void);
typedef int  (__cdecl *_PIFV)(void);

// Linker puts constructors between these sections, and we use them to locate constructor pointers.
#pragma section(".CRT$XIA",long,read)
#pragma section(".CRT$XIZ",long,read)
#pragma section(".CRT$XCA",long,read)
#pragma section(".CRT$XCZ",long,read)

// Put .CRT data into .rdata section
#pragma comment(linker, "/merge:.CRT=.rdata")

// Pointers surrounding constructors
__declspec(allocate(".CRT$XIA")) _PIFV __xi_a[] = { 0 };
__declspec(allocate(".CRT$XIZ")) _PIFV __xi_z[] = { 0 };
__declspec(allocate(".CRT$XCA")) _PVFV __xc_a[] = { 0 };
__declspec(allocate(".CRT$XCZ")) _PVFV __xc_z[] = { 0 };

extern __declspec(allocate(".CRT$XIA")) _PIFV __xi_a[];
extern __declspec(allocate(".CRT$XIZ")) _PIFV __xi_z[];    // C initializers
extern __declspec(allocate(".CRT$XCA")) _PVFV __xc_a[];
extern __declspec(allocate(".CRT$XCZ")) _PVFV __xc_z[];    // C++ initializers

// Call C constructors
static int _initterm_e(_PIFV * pfbegin, _PIFV * pfend) {
        int ret = 0;

        // walk the table of function pointers from the bottom up, until
        // the end is encountered.  Do not skip the first entry.  The initial
        // value of pfbegin points to the first valid entry.  Do not try to
        // execute what pfend points to.  Only entries before pfend are valid.
        while ( pfbegin < pfend  && ret == 0)
        {
            // if current table entry is non-NULL, call thru it.
            if ( *pfbegin != 0 )
                ret = (**pfbegin)();
            ++pfbegin;
        }

        return ret;
}

// Call C++ constructors
static void _initterm (_PVFV * pfbegin, _PVFV * pfend)
{
        // walk the table of function pointers from the bottom up, until
        // the end is encountered.  Do not skip the first entry.  The initial
        // value of pfbegin points to the first valid entry.  Do not try to
        // execute what pfend points to.  Only entries before pfend are valid.
        while ( pfbegin < pfend )
        {
            // if current table entry is non-NULL, call thru it.
            if ( *pfbegin != 0 )
                (**pfbegin)();
            ++pfbegin;
        }
}

// Call this function as soon as possible. Basically should be at the moment you
// jump into your C/C++ kernel. But keep in mind that kernel is not yet initialized,
// and you can't use a lot of stuff in your constructors!
bool CallConstructors() {
	// Do C initialization
	int initret = _initterm_e(__xi_a, __xi_z);
	if ( initret != 0 ) {
            return false;
	}

        // Do C++ initialization
        _initterm(__xc_a, __xc_z);
	return true;
}</source>


==new And delete Operators==
One of the first things you'll want to do is implement new and delete operators. At first, you can't really implement them, and just need to have their stubs. Later on, when you get your memory manager working, you can fully implement them. Bellow are the stubs:

<source lang="C">void* __cdecl operator new(size_t size)
{
	// Allocate memory
	return 0;
}


void* __cdecl operator new[](size_t size)
{
	// Allocate memory
	return 0;
}


void __cdecl operator delete(void *p)
{
	if (p == 0) {
		return;
	}

	// Release allocated memory
}

void __cdecl operator delete[](void *p)
{
	if (p == 0) {
		return;
	}

	// Release allocated memory
}</source>


If you'll want to use placement new, then you'll need to put the following implementation into a header file, and include it whenever you need it.

<source lang="C">inline void* __cdecl operator new(size_t size, void* address)
{
	return address;
}</source>


==See Also==
=== Articles ===
* [[Visual Studio]]
* [[C++]]

=== External Links ===
* [http://msdn.microsoft.com/en-us/library/bb918180.aspx CRT Initialization on MSDN]

[[Category:Compilers]]
[[Category:Visual C++]]
[[Category:C++]]
[[Category:Windows]]
````

<a id="w002993"></a>

## W002993 — Category:Visual C++

Bron-ID: W002993
Namespace: Category (14)
Revisie: 12563 | Datum: 2012-02-06T18:06:55Z
Bron: https://wiki.osdev.org/Category:Visual_C%2B%2B
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AVisual_C%2B%2B&oldid=12563

````text
Visual C++ can also be used to compile kernels!

[[Category:Tools]]
[[Category:Compilers]]
[[Category:Linkers]]
````

<a id="w002994"></a>

## W002994 — User:Mariuszp/Scheduler Tutorial

Bron-ID: W002994
Namespace: User (2)
Revisie: 18648 | Datum: 2015-11-26T09:17:01Z
Bron: https://wiki.osdev.org/User:Mariuszp/Scheduler_Tutorial
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMariuszp%2FScheduler_Tutorial&oldid=18648

````text
In this tutorial I will teach you how to write a scheduler (specifically for x86).

== What is a scheduler? ==
A ''scheduler'' is the part of your OS which allows programs to run (seemingly) "at once", without overwriting each others data. The idea is that there is a way to switch between running tasks. This can be done automatically (multi-tasking) or at the user's demand.

This tutorial will ''not'' explain paging, but it will explain how to load a simple flat binary file if you want to test your multitasking. For this specific tutorial, you will need to have a timer that generates an interrupt, so we can switch tasks automatically. Program this timer to be at a relatively high speed.

== How do schedulers work? ==
There are many ways to implement a scheduler. The method that was once used by MS-DOS extenders, as well as early Microsoft Windows, was co-operative multi-tasking. Every running process keeps calling a yield() function, which tells the scheduler to switch task. The downside of this is that it is easy to crash your system - if a process doesn't yield(), your system is not able to work any more. Another way you could do it is switch the task when a process uses a system call, which it would do frequently enough, or you could use both at once (ie. a yield system call). You could also allow the user to switch tasks themselves using some key, but for most users, this would ruin the idea of multi-tasking, because to them it would still seem that only one process can run at once.

The method I will explain here is a bit safer. This is called preemptive multi-tasking. You use a hardware timer which actively throws some interrupt, and the interrupt handler will save the process registers and switch the task. For that you can use the [[PIT]], the [[APIC]] timer, etc.

== Saving register states ==
First, you need a way to save register states when a task switch happens. There is actually quite a simple way to do this. In my OS, there is a common handler function which dispatches interrupts. Obviously, when the interrupt (IRQ0) is fired, some information (EIP, ESP, etc.) is pushed onto the stack. You can then use PUSHA to push the GPRs on. Please note that in my implementation, regs->esp is the ''temporary'' kernel stack, and should not be modified, while regs->useresp is the ''real'' value for ESP. My register structure is known by the type registers_t.

Of course, you can implement this any way you want to, just make sure that you can actually save those registers.

== Paging... ==
Let's just assume that the function create_user_pages(x, y) copies the kernel space into a new page directory, and makes the range from x to y user-accessible. We will need this to create page directories for the executables.

== Let's do it! ==
OK, we're ready to start doing some code!

=== process_t ===
The process_t structure contains information about a process. Let's define it as follows:
  typedef struct process_struct
  {
      registers_t     regs;
      pagedir_t       *cr3;
      struct process_struct *next;
  } process_t;
The 'regs' field contains the saved state of the processor registers prior to the program interruption. Remember, we need to restore them so that the program doesn't break! If you use paging, the 'cr3' field obviously contains the program's page directory (that doesn't need ''saving'', because it doesn't ''change'' - you just set it right at the start).

=== pqueue & current_proc ===
pqueue means process queue, and current_proc means, well, current process :D
  process_t *pqueue;
  process_t *current_proc;
When the scheduler is first started, you should set pqueue to some idle task, which basically loops forever. 'pqueue' will be used as the first process, so when we schedule through the whole queue, we start again at pqueue. current_proc will be set to the currently running process.

=== switch_task ===
The switch_task() function takes the saved registers_t as the argument, and should be called by the IRQ0 handler.

  volatile void switch_task(registers_t *regs)
  {
    /* copy the saved registers into the current_proc structure */
    memcpy(&current_proc->regs, regs, sizeof(registers_t));
  
    /* now go onto the next task - if there isn't one, go back to the start
       of the queue. */
    if (current_proc->next != NULL) current_proc = current_proc->next;
    else current_proc = pqueue;
  
    /* now hack the registers! */
    memcpy(regs, &current_proc->regs, sizeof(registers_t));
    switch_page_directory(current_proc->cr3);
  };
If you use the method I described above, then by changing the registers on the stack, you can perform a task switch. When the processor performs the POPA and IRET instructions, the "dirty work" will be done for you. Of course, if you use paging, you must also change the page directory to the one that the process uses.

== Loading flat binaries ==
A flat binary executable can only be loaded at one location. So if you move userspace in your OS, software will no more become functional. It is also difficult to implement shared libraries, because all binaries must be loaded at a given location. Let's take 6MB into memory. So, let's define a UCODE_START constant:
  #define UCODE_START 0x600000
Now, create a simple 32-bit flat binary in assembly language:
  ; test.asm
  bits 32
  org 0x600000
  jmp $
Assembler with:
  nasm test.asm -o test.bin -f bin
Now, assuming you can open a file and read its contents into memory, loading a flat binary is really simple:
  void load_bin(uint8_t *data, size_t size)
  {
    /* create the process_t and page directory (if you use paging) */
    process_t *proc = (process_t*) kmalloc(sizeof(process_t));
    proc->cr3 = create_user_pages(UCODE_START, UCODE_START+size);
  
    /* now we must copy the contents of the binary into memory, which involves changing
       the cr3 register if you use paging. If you _don't_ use paging, a memcpy is enough. */
    pagedir_t *temp = current_cr3;
    switch_page_directory(proc->cr3);
    memcpy(UCODE_START, data, size);
    switch_page_directory(temp);
  
    /* now we need to set up registers, of course. */
    proc->regs.cs  = 0x08;
    proc->regs.ds  = 0x10;
    proc->regs.ss  = 0x10;
    proc->regs.eip = UCODE_START;
  
    /* add the new process to the end of the queue. */
    process_t *last = pqueue;
    if (!last)
    {
      pqueue = proc;
      return;
    };
    while (last->next != NULL) last = last->next;
    last->next = proc;
  };

== What else? ==
There are plenty of things you may wish to do.

=== Executable Formats ===
Flat binaries have many drawbacks. You could learn to load some other format.
````

<a id="w002995"></a>

## W002995 — User:Love4Boobies/C style guide

Bron-ID: W002995
Namespace: User (2)
Revisie: 11816 | Datum: 2011-07-21T14:12:58Z
Bron: https://wiki.osdev.org/User:Love4Boobies/C_style_guide
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALove4Boobies%2FC_style_guide&oldid=11816

````text
Please adhere to this style guide when adding C/C++ code to the Wiki.

== Overall Style ==

<source lang="C">
// Make "takes no parameters" explicit by using "void".
int defined_elsewhere( void );

// Describe the purpose of the function, of the parameters it takes,
// and its return value (if any). Reference any third-party sources
// or algorithms used. Document specific design decisions.
void * foobar( int a, int * b )
{
    int c;

    // Summarize purpose of code blocks.
    if ( a == *b )
    {
        puts( "The values are equal." );
    }
    else
    {
        puts( "The values are different; clearing a." );
        a = 0;
    }

    // Summarize purpose of code blocks.
    while ( ( c = getchar() ) != EOF )
    {
        switch ( c )
        {
            case '0':
                *b = a;
                break;

            case '1':
                *b = a / 2;
                break;

            default:
                *b = 0;
                break;
        }
    }

    // see "Operator sizeof" below.
    return malloc( sizeof( *b ) );
}
</source>

'''Note:''' If you don't feel like formatting your code to the standards above manually, you could use one of several available code reformatters:

* [http://astyle.sourceforge.net/ astyle] -A1 -p -D -f -S -xd -k2
* ...

== Standard library ==

The following headers are available in freestanding implementations. Please do not duplicate in custom typedefs etc. what these headers define in a portable, standard way:

 * [http://www.dinkumware.com/manuals/?manual=compleat&page=float.html <tt><float.h></tt>]
 * [http://www.dinkumware.com/manuals/?manual=compleat&page=iso646.html <tt><iso646.h></tt>]
 * [http://www.dinkumware.com/manuals/?manual=compleat&page=limits.html <tt><limits.h></tt>]
 * <tt><stdalign.h></tt>
 * [http://www.dinkumware.com/manuals/?manual=compleat&page=stdarg.html <tt><stdarg.h></tt>]
 * [http://www.dinkumware.com/manuals/?manual=compleat&page=stdbool.html <tt><stdbool.h></tt>]
 * [http://www.dinkumware.com/manuals/?manual=compleat&page=stddef.html <tt><stddef.h></tt>]
 * [http://www.dinkumware.com/manuals/?manual=compleat&page=stdint.html <tt><stdint.h></tt>]

Consider using the interface defined by [http://www.dinkumware.com/manuals/?manual=compleat&page=iohw.html <tt><iohw.h></tt>] (TR 18037) for accessing ports, too. (Instead of using functions such as <tt>inb</tt> and <tt>outb</tt>.)

== Typedef names ==

Avoid typedef's. They will not save you much typing in the scope of this Wiki.

== Operator sizeof ==

Using sizeof on object name instead of type remains correct even if type is changed. Even while sizeof is an operator, not a function, writing it like a function usually helps readability. Just remember you cannot take its address, that its operand is not evaluated, and that you don't need to cache sizes as this is a compile-time construct.

== Compiler-specific extensions ==

Avoid them unless they are the subject of the article (i.e., [[Inline Assembly]]). In particular, things like GCC's <tt>__attribute__ ((aligned (16), packed))</tt> should be substituted with proper serialization (e.g., arrays, or files), and language alignment support (as introduced in C1X) --- if not, at least use unions to force alignment.

== The compiler is your friend ==

Your code should compile with:

* <tt>gcc -std=c99 -Winit-self -Wfloat-equal -Wundef -pedantic -Wall -Wextra -Werror</tt>
* <tt>g++ -pedantic -Wall -Wextra -Werror</tt>
````

<a id="w002997"></a>

## W002997 — User:Sidewaysmilk

Bron-ID: W002997
Namespace: User (2)
Revisie: 20587 | Datum: 2017-03-14T03:44:31Z
Bron: https://wiki.osdev.org/User:Sidewaysmilk
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASidewaysmilk&oldid=20587

````text

````

<a id="w002998"></a>

## W002998 — Ext3

Bron-ID: W002998
Namespace: Main (0)
Revisie: 29777 | Datum: 2025-12-01T16:12:15Z
Bron: https://wiki.osdev.org/Ext3
Revisie-URL: https://wiki.osdev.org/index.php?title=Ext3&oldid=29777

````text
{{Stub}}
{{Filesystems}}
The Ext3 [[:Category:Filesystems|filesystem]], or third extended filesystem, is an [[ext2|Ext2]] filesystem with additional features. It most notably includes support for [[journaling]] and H-Tree indexed directories.

Since indexed directories are easier to implement than journaling, this is what we will focus on for now.

== Indexed Directories ==

Normally, Ext2 directories store entries in a '''linked list''', i.e. each directory block consists of a number of directory entries, one following the other in a linear way. Therefore, any directory operation (search, insertion, deletion) requires a linear search until the required entry is found. This results in an increasing cost as the directory grows larger.

'''H-Tree''' (or Hashed B-Tree) indexing was developed as an Ext2 extension to address this issue. In this scheme, file names are hashed and the hashes are used as keys to the H-Tree. To maintain backward compatibility with disk drivers that do not understand H-Trees, indexed directories contain linked list entries alongside the indexed entries. This includes the addition of '''fake entries''', which are described below.

=== Indexed Directory Blocks ===

Like '''linked list''' directories, '''H-Tree indexed''' directories consist of disk blocks, with direct and indirect pointers stored in the inode structure (see the [[ext2|ext2]] page for details). Unlike linear directories, an H-Tree indexed directory has 2 types of blocks:
* '''Index blocks''': store pairs of hashes and their associated block numbers.
* '''Leaf blocks''': store normal directory entries.

The first block (block #0) in the directory is the hash tree root. It begins with the '''Indexed Directory Root''' structure, which consists of 2 fake linked list entries, followed by indexed directory root information. After this structure, there are a number of '''Indexed Directory Entries''', which continue until the end of the block, or until all files in the directory have been indexed. 

Index blocks (other than the first block) start with a fake entry, followed by '''Indexed Directory Entries''', similar to the first block.

==== Indexed Directory Root Structure ====

{| {{Wikitable}}
! Offset
! Field
! Size
! Description
|-
| 0 || Fake dot entry || 8 || Normal linked list entry
|-
| 8 || Fake dot entry name || 4 || '.' and 3 padding bytes
|-
| 12 || Fake dot-dot entry || 8 || Normal linked list entry
|-
| 20 || Fake dot-dot entry name || 4 || '.', '.', and 2 padding bytes
|-
| 24 || Reserved || 4 || Must be zero
|-
| 28 || Hash version || 1 || Valid values in the table below
|-
| 29 || Info length || 1 || Size of index root info (must be 8)
|-
| 30 || Indirect levels || 1 || Max. tree depth (currently 1)
|-
| 31 || Reserved || 1 || Unused flags
|}

==== Indexed Directory Hash Versions ====

{| {{Wikitable}}
! Name
! Value
|-
| DX_HASH_LEGACY || 0
|-
| DX_HASH_HALF_MD4 || 1
|-
| DX_HASH_TEA || 2
|}

=== Indexed Directory Entries ===

Indexed Directory entries are present in all index blocks, including the first (or root) block. In the case of the root block, these entries appear after the '''Indexed Directory Root''' structure. In index blocks other than the root block, they appear after a single fake directory entry.

Each indexed directory entry contains a hash value and its associated block number, which is calculated relative to the start of the directory (i.e. block 0). The entries are sorted by hash value, from smallest to largest.

The first indexed directory entry is special, it contains the '''limit''' (maximum number) and '''count''' (actual used number) of directory entries that fit in the block.

==== Indexed Directory Entry Structure ====

{| {{Wikitable}}
! Offset
! Field
! Size
! Description
|-
| 0 || Hash value || 4 || 32-bit hash of the filename represented by this entry
|-
| 4 || Block number || 4 || Block number (relative to block 0 or the beginning of the directory) of the block containing the directory entry corresponding to the hashed filename
|}

==== Indexed Directory Entry Count & Limit Structure ====

{| {{Wikitable}}
! Offset
! Field
! Size
! Description
|-
| 0 || Limit || 2 || Maximum number of entries per block
|-
| 2 || Count || 2 || Number of used entries in the block
|}

== Lookup Algorithm ==

To find a directory entry in an H-Tree indexed directory:
# Calculate the hash value of the filename (using the hash algorithm specified in the '''Indexed Directory Root Structure''').
# Read block 0 (the index root).
# Iterate through the '''Indexed Directory Entries''' in the block until a suitable hash value is found. This is the hash value that equals the hash value calculated in step 1, or is the largest value that is smaller than the calculated value if an exact match is not found.
# Read the block that is referenced by the matched hash entry. Remember that block indices are relative, so you will have to find the actual disk block number from the directory's inode structure (e.g. for relative block #4, you will need to read the block indicated by direct block index #4 in the inode structure).
# If the H-Tree's depth is zero, the block will be a '''leaf block''', containing linked list entries. Read the block and iterate through the entries until you find the requested entry, as you would for a normal linked list Ext2 directory.
# If the H-Tree's depth is one, the block will be an '''index block''', which contains a fake directory entry, followed by a series of '''Indexed Directory Entries'''. Read the block and repeat steps 1-4 until you find your entry.

== TODO ==
* Insertion algorithm
* Key collisions
* Leaf block splitting
* Journaling

== See Also ==
=== Resources ===
* [https://www.nongnu.org/ext2-doc/ext2.pdf Ext2 internal layout]
* [https://www.netbsd.org/gallery/presentations/hrishikesh/2017_AsiaBSDCon/abc2017ext4_final_paper.pdf NetBSD adds Ext4 extent and Ext3 HTree support]
* [https://students.mimuw.edu.pl/ZSO/Wyklady/11_extXfs/ExtXfs_short.pdf Ext2/3 presentation]

=== External Links ===
* [https://github.com/haiku/haiku/blob/master/src/add-ons/kernel/file_systems/ext2/HTree.cpp Haiku HTree implementation]
* [https://github.com/moisam/laylaos/blob/main/kernel/fs/ext2fs_hash.c Hobby OS HTree implementation]

[[Category: Filesystems]]
[[de:Ext3]]
````

<a id="w002999"></a>

## W002999 — User:Communistpancake

Bron-ID: W002999
Namespace: User (2)
Revisie: 11677 | Datum: 2011-07-02T04:23:52Z
Bron: https://wiki.osdev.org/User:Communistpancake
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACommunistpancake&oldid=11677

````text
Hi.
````

<a id="w003000"></a>

## W003000 — Journaling

Bron-ID: W003000
Namespace: Main (0)
Revisie: 27725 | Datum: 2023-04-02T22:31:31Z
Bron: https://wiki.osdev.org/Journaling
Revisie-URL: https://wiki.osdev.org/index.php?title=Journaling&oldid=27725

````text
Journaling is a filesystem technique that is used to combat data corruption in the event of power loss or other forced system shutdown. As long as the underlying disk is well-behaved, a correct implementation of journaling can guarantee that the filesystem never enters an inconsistent state as a result of a mere sudden shutdown. It is therefore a crucial feature of all modern filesystems, such as [[ext3]], [[ext4]], or [[NTFS]].

=Background=

A single filesystem operation may require multiple actions to be carried out on disk. For example, appending data to a file in an ext4 filesystem could involve:
# finding a free data block and marking it as used,
# writing the data to this block,
# adding the block number to the extent tree, potentially adding a new layer to the tree,
# and modifying the inode to reflect the new size.

If the system loses power in the middle of this operation, the on-disk filesystem will be left in an inconsistent state:
* If the system loses power between (2) and (3), the newly allocated block will be orphaned and will reduce the free space available until the next filesystem check.
* If the system loses power in the middle of writing a block in (3), the contents of said block may become arbitrarily corrupted. As disks often include checksums in the low-level on-platter format, it is likely that reading the block will result in a read error until it gets overwritten. This would result in significant data loss in the affected file.
* Only a part of the write operation might persist, while the user-level application might expect to find either none or all of the data.

To prevent this, a journaling filesystem reserves a special section of the disk, called the ''journal'', and performs operations as follows:
# Prepare a packet describing what is about to be done.
# Write the packet describing the transaction to the journal.
# Flush the disk cache to make sure the data persists after a power loss. At this point, the transaction has been committed.
# Perform the changes described in the transaction to the actual data section of the disk.
# Flush the disk cache again.
# Mark the journal entry as completed.

If the system loses power at (2) or (3), the journal entry gets corrupted (which we can detect because our journal format uses a checksum). This is not a problem -- we just discard the journal entry and effectively revert to the state before the transaction. On the other hand, if the system loses power at (4) or (5), then we can replay the transaction, just like the system would if the sudden shutdown hadn't happened.

=Design considerations=

==Idempotency==

Of course, one must ensure that executing the initial portion of a transaction, followed by the full transaction from the very beginning, will give the same result as just executing the transaction in full. To achieve this, two separate properties are required of the transaction steps:
* Each step must be ''idempotent'', which means that executing it as many times as you want yields the same end result as executing it once.
* The steps must be independent of each other, where you could execute them in any order and get the same result.

Writing a block of data at a specific location is the simplest idempotent operation, but more complex ones are possible, such as copying a long range of blocks from one location to another (as long as the source and destination don't overlap).

If the steps of a complex transaction aren't independent of each other, the transaction can be broken down into stages, where each stage being finished is noted down in the journal before starting the next stage. Then we don't have to worry about steps in different stages interfering with each other.

==Journal scope==

The most reliable and simple method is to journal all writes -- both the filesystem metadata, and the data itself. This however has the disadvantage of slowing the writes down a lot. A lighter approach logs only metadata writes in the journal. This has the effect of containing any potential corruption within the data blocks, preventing any subsequent cascading corruption in the filesystem data structures.

This choice can be made configurable. Linux's implementation of Ext4 exposes this at the filesystem level, with the <code>data=journal</code> and <code>data=ordered</code> mount options. This choice could also be exposed to the application itself -- for example, a database engine would opt into the more expensive, but more reliable full journaling, without burdening the throughput of e.g. a web browser downloading a large file. 

==Performance==

Having everything go through the journal can impact performance, since effectively every write is doubled. This can be mitigated by allowing transactions to accumulate in the journal during burst load, and then finalizing them while the system is idle.

==Subtle issues==

Metadata journaling can be a subtle matter. When deallocating a block, you need to make sure that no operations involving it are pending in the journal, before you actually reuse the block. When all writes go through the journal, this is not a concern.

==Advanced techniques==

While a simple list of block writes to be done will suffice for most usecases, more advanced journal operations can facilitate features such as defragmentation, or safely migrating to a new, incompatible on-disk format, without requiring
an excessively large journal.

[[Category: OS theory]]
[[Category: Filesystems]]
````

<a id="w003001"></a>

## W003001 — Ext4

Bron-ID: W003001
Namespace: Main (0)
Revisie: 30518 | Datum: 2026-04-26T04:18:19Z
Bron: https://wiki.osdev.org/Ext4
Revisie-URL: https://wiki.osdev.org/index.php?title=Ext4&oldid=30518

````text
{{Stub}}
{{Filesystems}}
While the ext4 filesystem originated a series of patches to the [[ext3]] filesystem, it was later rebranded as a dedicated filesystem design that shares design with [[ext2]] and [[ext3]]. Like ext3, it supports [[journaling]]. Amongst the upgrades are larger maximums (file size, filesystem size, files per folder, folders per folder etc), and features that were inspired from existing filesystems like [[XFS]].

''This information is based off the Ext4 implementation as of Linux 5.9rc3. Ext4 is subject to change, so one may wish to also check the latest kernel headers for new information.''

== Basic Concepts ==
{{Stub}}

== Superblock ==

See [[Ext2#Superblock|Ext2]] wiki page for an easier introduction to the concept of a superblock. All values are little endian unless otherwise specified.

{| class="wikitable"
! Starting
Byte
! Ending
Byte
! Size
! Description
|-
| 0 || 3 || 4 || Total number of inodes in file system
|-
| 4 || 7 || 4 || Total number of blocks in file system
|-
| 8 || 11 || 4 || Number of reserved blocks
|-
| 12 || 15 || 4 || Total number of unallocated blocks
|-
| 16 || 19 || 4 || Total number of unallocated inodes
|-
| 20 || 23 || 4 || Block number of the block containing the superblock. This is 1 on 1024 byte block size filesystems, and 0 for all others.
|-
| 24 || 27 || 4 || ''log''<sub>2</sub> (block size) - 10. (In other words, the number to shift 1,024 to the left by to obtain the block size)
|-
| 28 || 31 || 4 || ''log''<sub>2</sub> (fragment size) - 10. (In other words, the number to shift 1,024 to the left by to obtain the fragment size)
|-
| 32 || 35 || 4 || Number of blocks in each block group
|-
| 36 || 39 || 4 || Number of fragments in each block group
|-
| 40 || 43 || 4 || Number of inodes in each block group
|-
| 44 || 47 || 4 || Last mount time (in [http://en.wikipedia.org/wiki/Unix_time POSIX time])
|-
| 48 || 51 || 4 || Last written time (in [http://en.wikipedia.org/wiki/Unix_time POSIX time])
|-
| 52 || 53 || 2 || Number of times the volume has been mounted since its last consistency check ([http://en.wikipedia.org/wiki/Fsck fsck])
|-
| 54 || 55 || 2 || Number of mounts allowed before a consistency check ([http://en.wikipedia.org/wiki/Fsck fsck]) must be done
|-
| 56 || 57 || 2 || Magic signature (0xef53), used to help confirm the presence of Ext4 on a volume
|-
| 58 || 59 || 2 || File system state.
|-
| 60 || 61 || 2 || What to do when an error is detected
|-
| 62 || 63 || 2 || Minor portion of version (combine with Major portion below to construct full version field)
|-
| 64 || 67 || 4 || [http://en.wikipedia.org/wiki/Unix_time POSIX time] of last consistency check ([http://en.wikipedia.org/wiki/Fsck fsck])
|-
| 68 || 71 || 4 || Interval (in [http://en.wikipedia.org/wiki/Unix_time POSIX time]) between forced consistency checks ([http://en.wikipedia.org/wiki/Fsck fsck])
|-
| 72 || 75 || 4 || Operating system ID from which the filesystem on this volume was created ([[#Creator_Operating_System_IDs|see below]])
|-
| 76 || 79 || 4 || Major portion of version (combine with Minor portion above to construct full version field)
|-
| 80 || 81 || 2 || User ID that can use reserved blocks
|-
| 82 || 83 || 2 || Group ID that can use reserved blocks
|}

These fields are for ext4 dynamic superblocks only. If a bit is set in the required feature set it does not recognize, it must refuse to mount the filesystem. Filesystem checks, however, must abort on '''any''' unrecognized flag in the optional or required features.

{| class="wikitable" cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
!Starting
Byte
!Ending
Byte
!Size
in Bytes
!Description
|-
| 84 || 87 || 4 || First non-reserved inode in file system.
|-
| 88 || 89 || 2 || Size of each inode structure in bytes.
|-
| 90 || 91 || 2 || Block group that this superblock is part of for backup copies.
|-
| 92 || 95 || 4 || Optional features present.
|-
| 96 || 99 || 4 || Required features present.
|-
| 100 || 103 || 4 || Features that if not supported the volume must be mounted read-only.
|-
| 104 || 119 || 16 || File system UUID.
|-
| 120 || 135 || 16 || Volume name.
|-
| 136 || 199 || 64 || Path Volume was last mounted to.
|-
| 200 || 203 || 4 || Compression algorithm used.
|-
| 204 || 204 || 1 || Amount of blocks to preallocate for files
|-
| 205 || 205 || 1 || Amount of blocks to preallocate for directories.
|-
| 206 || 207 || 2 || Amount of reserved GDT entries for filesystem expansion.
|-
| 208 || 223 || 16 || Journal UUID.
|-
| 224 || 227 || 4 || Journal Inode.
|-
| 228 || 231 || 4 || Journal Device number.
|-
| 232 || 235 || 4 || Head of orphan inode list.
|-
| 236 || 251 || 16 || HTREE hash seed in an array of 32 bit integers.
|-
| 252 || 252 || 1 || Hash algorithm to use for directories. Often mirrored in hashed directory root struct.
|-
|253||253 ||1||Journal blocks field contains a copy of the inode's block array and size.
|-
| 254|| 255 || 2||Size of group descriptors in bytes, for 64 bit mode.
|-
|256|| 259||4 ||Mount options.
|-
| 260||263||4||First metablock block group, if enabled.
|-
|264||267||4 || Filesystem Creation Time.
|-
|268||335 ||68||Journal Inode Backup in an array of 32 bit integers.
|}

Valid if the 64bit feature is set.

{| class="wikitable" cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
!Starting
Byte
! Ending
Byte
!Size
in Bytes
! Description
|-
|336 || 339||4||High 32-bits of the total number of blocks.
|-
|340 || 343||4 ||High 32-bits of the total number of reserved blocks.
|-
|344 || 347||4|| High 32-bits of the total number of unallocated blocks.
|-
| 348|| 349||2||Minimum inode size.
|-
|350||351||2|| Minimum inode reservation size.
|-
|352||355|| 4||Misc flags, such as sign of directory hash or development status.
|-
|356||357||2||Amount logical blocks read or written per disk in a RAID array.
|-
| 358||359||2||Amount of seconds to wait in Multi-mount prevention checking.
|-
|360
|367||8||Block to multi-mount prevent.
|-
|368||371||4||Amount of blocks to read or write before returning to the current disk in a RAID array. Amount of disks * stride.
|-
|372||372 ||1|| ''log''<sub>2</sub> (groups per flex) - 10. (In other words, the number to shift 1,024 to the left by to obtain the groups per flex block group)
|-
|373||373 ||1 ||Metadata checksum algorithm used. Linux only supports crc32.
|-
|374||374||1 ||Encryption version level.
|-
|375||375||1||Reserved padding.
|-
|376 ||383||8||Amount of kilobytes written over the filesystem's lifetime.
|-
|384||387
|4||Inode number of the active snapshot.
|-
|388||391|| 4||Sequential ID of active snapshot.
|-
|392||399||8||Number of blocks reserved for active snapshot.
|-
|400||403||4||Inode number of the head of the disk snapshot list.
|-
|404|| 407|| 4|| Amount of errors detected.
|-
| 408||411||4||First time an error occurred in POSIX time.
|-
|412
|415||4 ||Inode number in the first error.
|-
|416
|423||8 ||Block number in the first error.
|-
|424||455||32||Function where the first error occurred.
|-
|456||459||4||Line number where the first error occurred.
|-
|460||463||4||Most recent time an error occurred in POSIX time.
|-
|464||467||4|| Inode number in the last error.
|-
|468||475||8|| Block number in the last error.
|-
|476||507||32||Function where the most recent error occurred.
|-
|508||511 ||4||Line number where the most recent error occurred.
|-
|512 ||575||64||Mount options. (C-style string: characters terminated by a 0 byte)
|-
|576||579|| 4||Inode number for user quota file.
|-
| 580||583 ||4||Inode number for group quota file.
|-
|584||587 ||4 ||Overhead blocks/clusters in filesystem. Zero means the kernel calculates it at runtime.
|-
|588||595||8 ||Block groups with backup Superblocks, if the sparse superblock flag is set.
|-
|596 ||599||4||Encryption algorithms used, as a array of unsigned char.
|-
|600||615|| 16||Salt for the `string2key` algorithm.
|-
|616||619||4
|Inode number of the lost+found directory.
|-
|620||623||4|| Inode number of the project quota tracker.
|-
|624||627||4||Checksum of the UUID, used for the checksum seed. (crc32c(~0, UUID))
|-
|628||628||1||High 8-bits of the last written time field.
|-
|629||629||1
|High 8-bits of the last mount time field.
|-
|630||630||1||High 8-bits of the Filesystem creation time field.
|-
|631|| 631||1||High 8-bits of the last consistency check time field.
|-
|632 ||632||1|| High 8-bits of the first time an error occurred time field.
|-
|633||633 ||1||High 8-bits of the latest time an error occurred time field.
|-
|634|| 634 || 1||Error code of the first error.
|-
|635||635||1|| Error code of the latest error.
|-
|636|| 637|| 2|| Filename charset encoding.
|-
|638
|639||2 ||Filename charset encoding flags.
|-
|640 ||1019||380||Padding.
|-
|1020
|1023||4||Checksum of the superblock.
|}

Required Features:

{| class="wikitable" cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
!Flag
Value
!Description
|-
|0x0001||Compression is used.
|-
|0x00002 ||Directory entries contain a type field.
|-
|0x00004||Filesystem needs to replay the Journal to recover data.
|-
|0x00008||Filesystem uses a journal device.
|-
|0x00010|| Filesystem uses Meta Block Groups.
|-
|0x00040|| Filesystem uses extents for files.
|-
|0x00080||Filesystem uses 64 bit features.
|-
|0x00100|| Filesystem uses Multiple Mount Protection.
|-
|0x00200|| Filesystem uses Flex Block Groups.
|-
|0x00400||Filesystem uses Extended Attributes in Inodes.
|-
|0x01000|| Filesystem uses Data in Directory Entries. This is not implemented as of Linux 5.9rc3.
|-
|0x02000 || Filesystem stores the metadata checksum seed in the superblock. This allows for changing the UUID wihtout rewriting all of the metadata blocks.
|-
|0x04000 ||Directories may be larger than 4GiB and have a maximum HTREE depth of 3.
|-
| 0x08000||Data may be stored in the inode. See [[Ext4#Inline_Data|Inline Data]] for an discussion of this feature.
|-
|0x10000||Filesystem uses Encryption.
|-
|0x20000||Filesystem uses case folding, storing the filesystem-wide encoding in inodes.
|}

Optional Features:

{| class="wikitable" cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
!Flag
Value
!Description
|-
|0x0001|| Preallocate some number of blocks (see byte 205 in the superblock) to a directory when creating a new one.
|-
| 0x0002 || Possibly unused, "imagic inodes"
|-
| 0x0004|| Filesystem uses a Journal
|-
| 0x0008 || Inodes have Extended Attributes.
|-
|0x0010||Filesystem can resize itself for larger partitions.
|-
|0x0020||Directories use hash index.
|-
|0x0200
|Backup the superblock in other block groups.
|-
|0x400
|Fast commit journal blocks. (only incompatible if journal was not replayed before unmount)
|-
|0x0800||Inode numbers do not change during resize.
|-
|0x1000
|Use orphan file for orphaned inodes.
|}
Read-Only Features:
{| class="wikitable" cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
!Flag
Value
!Description
|-
|0x000001
|Sparse superblocks and group descriptor tables. 
|-
|0x000002
|File system uses a 64-bit file size.
|-
|0x000004
|Directory contents are stored in the form of a [https://en.wikipedia.org/wiki/Binary_tree Binary Tree].
|-
|0x000008
|File system enables "huge files". 
|-
|0x000010
|Perform a crc32 on Group Descriptor Table.
|-
|0x000020
|Allow inode hardlinks to overflow. When this bit is set, if the number of hardlinks to an inode is greater than 32,000, the count is set to 1.
|-
|0x000040
|Require inode's size in memory to be a minimum of 256 bytes. (This is often the default, as most operating systems store fragment information in this area anyways.)
|-
|0x000100
|Dedicate inodes 3 and 4 to user and group limit tracking. Replaces external quota files.
|-
|0x000200
|Enable clustering. Blocks are replaced with 16 block clusters at the block group level, including inode addresses and bitmaps.
|-
|0x000400
|Perform a crc32 on metadata and Group Descriptor Table. 0x0010 and 0x0400 are mutually exclusive.
|-
|0x001000
|The filesystem itself is static and mounted as read only.
|-
|0x002000
|Enable project level quotas, for size capping on collaborative projects.
|-
|0x008000
|Enable verity files. Files that are constant, with cryptographically hashed contents.
|-
|0x010000
|Orphan cleanup required.
|}
Mount Options Flags:
{| class="wikitable" cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
!Flag
Value
!Description
|-
|0x0001
|Print debug info when mounting. 
|-
|0x0002
|New files inherit the group id from it's directory, and not from the process that created it.
|-
|0x0004
|Support userbase-provided extended attributes.
|-
|0x0008
|Support POSIX access control list.
|-
|0x0010
|Enforce 16bit user ids.
|-
|0x0020
|All data and metadata is committed to the journal.
|-
|0x0040
|All data is flushed to disk before metadata is committed to the journal.
|-
|0x0060
|Same as 0x0040, except data can be written after metadata is committed.
|-
|0x0100
|Disable write flushes.
|-
|0x0200
|Differentiate between metadata blocks and data blocks. (May be default in future updates).
|-
|0x0400
|Enable DISCARD support, where the device is updated about newly unused blocks.
|-
|0x0800
|Disable delayed allocation.
|}
Valid Hash Field Values:
{| class="wikitable" cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
!Value 
!Algorithm Name
|-
|0
|Legacy
|-
|1
|Half Md4
|-
|2
|Tea
|-
|3
|Legacy Unsigned
|-
|4
|Half Md4 Unsigned
|-
|5
|Tea Unsigned 
|}

==Block Group Descriptor==

See [[Ext2#Block_Group_Descriptor_Table|Ext2]] wiki page for an introduction to block group descriptor tables.

In Ext4, the block descriptors, in addition to their role in Ext2 as information of important data structures, have new features such as flex block groups and meta block groups.

In a flex block group, multiple block groups are grouped together into a flex block group, as the group descriptor records the location of both bitmaps and the inode table. This allows for better data locality.

In a meta block group, the filesystem is partitioned into multiple 'metablock' groups. This allows the metadata of the block group descriptors to be stored in one block. This strategy also increases the maximum filesystem size to 512PiB from 256TiB without metablock groups.

===Locating the Block Group Descriptors ===

Locating the Block Group Descriptors is similar to Ext2, except for metablock and flex block group locations.


One can check for flex block groups by checking the required option 'Flex Block Group'. The structure for that is found in the superblock data structure.

Flex Block Group info Structure, these fields are atomic integers:

{| class="wikitable" cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
! Starting
Byte
!Ending
Byte
!Size
in Bytes
!Description
|-
|0||7 ||8
|Atomic 64 bit free clusters.
|-
|8||11||4||Atomic free inodes.
|-
|12|| 15||4 ||Atomic used directories.
|}

===Block Group Descriptor===

Block group descriptor Structure:

{| class="wikitable" cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
!Starting
Byte
!Ending
Byte
! Size
in Bytes
!Description
|-
|0 ||3||4||Low 32bits of block address of block usage bitmap.
|-
|4||7|| 4||Low 32bits of block address of inode usage bitmap.
|-
|8 || 11||4||Low 32bits of starting block address of inode table.
|-
| 12 || 13||2||Low 16bits of number of unallocated blocks in group.
|-
|14||15||2||Low 16bits of number of unallocated inodes in group.
|-
|16||17||2||Low 16bits of number of directories in group.
|-
|18||19||2||Block group features present.
|-
|20||23||4|| Low 32-bits of block address of snapshot exclude bitmap.
|-
|24||25||2||Low 16-bits of Checksum of the block usage bitmap.
|-
|26||27||2||Low 16-bits of Checksum of the inode usage bitmap.
|-
|28||29||2||Low 16-bits of amount of free inodes. This allows us to optimize inode searching.
|-
| 30||31||2||Checksum of the block group, [[CRC16]](UUID+group+desc).
|}

These fields are valid if the 64bit feature is set and the superblock's group desciptor size is greater than 32.

{| class="wikitable" cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
!Starting
Byte
!Ending
Byte
!Size
in Bytes
!Description
|-
| 32 || 35||4||High 32-bits of block address of block usage bitmap.
|-
|36||39||4||High 32-bits of block address of inode usage bitmap.
|-
|40||43|| 4||High 32-bits of starting block address of inode table.
|-
|44||45|| 2||High 16-bits of number of unallocated blocks in group.
|-
| 46 ||47||2||High 16-bits of number of unallocated inodes in group.
|-
|48||49||2 ||High 16-bits of number of directories in group.
|-
|50||51||2||High 16-bits of amount of free inodes.
|-
|52||55|| 4||High 32-bits of block address of snapshot exclude bitmap.
|-
|56|| 57 ||2||High 16-bits of checksum of the block usage bitmap.
|-
|58||59||2||High 16-bits of checksum of the inode usage bitmap.
|-
|60|| 63 ||4||Reserved as of Linux 5.9rc3.
|}

block group flags:

{| class="wikitable" cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
!Flag
Value
!Description
|-
|0x0001||Block group's inode bitmap/table is unused.
|-
|0x0002||Block groups's block bitmap is unused.
|-
|0x0004||Block groups's inode table is zeroed.
|}

==Multiple Mount Protection==

Multiple Mount Protection (MMP) protects filesystems being mounted multiple times leading to dangerous data races. This feature writes a sequence number into the block referenced in the MMP block superblock field.

To check for MMP, check the required feature flag Multiple Mount Protection, and the magic field in the block referenced by the MMP block superblock field.

In MMP, the driver checks the sequence number in the MMP block . If the sequence number is fs check running or any unknown code above maximum MMP value, the drive is not safe to mount, even if the timestamp is outdated.

While running, the driver checks the MMP block sequence number at the interval specified in the MMP check superblock field. If this does not match the in-memory sequence number, a different host has mounted the filesystem, requiring the driver to remount as readonly. If it does match, the driver increments the number in memory and on disk. The driver also writes the hostname and mount path in the MMP block on open() success.

The minimum interval to MMP check is 5 seconds and the maximum interval is 300.

===MMP structure===

{| class="wikitable" cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
!Starting
Byte
!Ending
Byte
!Size
in Bytes
!Description
|-
|0 ||3||4||MMP signature (0x004d4d50)
|-
|4|| 7||4|| Sequence Number ([[#MMP_Sequences|see below]])
|-
|8||15 || 8||Last updated time (does not affect algorithm)
|-
|16||79||64|| Hostname of system that open() the filesystem (does not affect algorithm)
|-
| 80 ||111 ||32||Mount path of system that open() the filesystem (does not affect algorithm)
|-
|112||113||2||Interval to check MMP block
|-
|114||115||2 ||Padding.
|-
| 116||1019 || 904||Padding.
|-
|1020||1023|| 4||Checksum (crc32c(UUID+MMP Block number))
|}

== Journaling==

See [[Journaling]] for a high level description of a filesystem journal.

Ext4 uses the [[Jbd2]] Journaling layer.

Ext4 defines the journal inode as inode 8. The superblock contains the first 68 bytes of the journal. The journal is a hidden file in the filesystem, usually using an entire block group, but it is preferred to be in the middle of the volume.

The optional feature filesystem journal protects against filesystem corruption if the system crashes. The filesystem journal writes important data to a small contigous sliver of disk. Once this is flushed to disk, the driver writes a record of the data to be written to the journal. Later, the driver can write the transactions to disk. If the system crashes during the second write, it can simply replay the journal to the last sync. If the write succeedes, the write's record is removed from the journal.

The default journaling strategy is 'ordered', writing only filesystem metadata through journaling. If stronger guarantees are preferred, the filesystem can be use the 'journal' strategy, writing both metadata and data through the journal, slowing operation. The filesystem can also use the 'writeback' strategy, where data is not flushed to the disk before a metadata update.

Ext4 may also use a seperate journal device, specified in the superblock's journal UUID. The separate journal device will have 1024 bytes of padding, then an ext4 superblock with a matching UUID. The journal follows on the next complete block.

===Journal Superblock Fields===

Ext4 uses [[Jbd2]] as the journal.

All fields are big endian unless otherwise specified.

{| class="wikitable" cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
!Starting
Byte
!Ending
Byte
!Size
! Description
|-
|0 ||11||12||Journal header ([[#Journal_Header|see below]])
|-
| 12||15||4||Block size of the journal device.
|-
|16||19||4||Total number of blocks in the journal device.
|-
|20||23||4||First block of journal information.
|-
|24||27||4||First journal transaction expected.
|-
|28||31||4||First block of the journal.
|-
| 32 || 35 || 4 || Errno, if the journal has an error.
|-
|36||39||4||Required features present.
|-
|40 ||43|| 4||Optional features present.
|-
| 44 || 47||4 ||Features that if not supported the journal must be mounted read-only. There are no read-only features as of Linux 5.9rc3.
|-
|48||63|| 16 || Journal UUID.
|-
| 64||67||4||Number of filesystems using this journal.
|-
|68
|71||4 || Block number of the journal superblock copy.
|-
| 72 ||75||4||Maximum journal blocks per transaction. This is unused as of Linux 5.9rc3.
|-
|76||79
|4 || Maximum data blocks per transaction. This is unused as of Linux 5.9rc3.
|-
|80||80
|1||Checksum algorithm.
|-
|81||83||3
|Padding.
|-
|84||251||168|| Padding.
|-
|252 ||255|| 4||Checksum of the journal superblock.
|-
|256||1023|| 768|| UUID of filesystem
|}

=== Journal Header===

All fields are big endian unless otherwise specified.

{| class="wikitable" cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
! Starting
Byte
! Ending
Byte
! Size
!Description
|-
| 0|| 3||4
|Magic signature (0xc03b3998)
|-
|4
|7||4||Block Type ([[#Journal_Block_Type|see below]])
|-
|8|| 11||4||Journal transaction for this block.
|}


==See Also==
===External Links===
*[https://www.kernel.org/doc/html/latest/filesystems/ext4/index.html ext4 Data Structures and Algorithms] - detailed description of ext4 from the Linux kernel documentation.
*[https://github.com/torvalds/linux/blob/master/fs/ext4/ext4.h Linux ext4 header] - This header contains useful definitions and declarations.

[[de:Ext4]]
````

<a id="w003002"></a>

## W003002 — Talk:Calling Conventions

Bron-ID: W003002
Namespace: Talk (1)
Revisie: 19697 | Datum: 2016-08-20T23:13:43Z
Bron: https://wiki.osdev.org/Talk:Calling_Conventions
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ACalling_Conventions&oldid=19697

````text
hi,
I think the page should have a note about StackGuard(stack protector in GCC), since it may change the order of parameter, eip, ... etc in the stack. I read this page and tried, but alway failed, at last I found Stackguard.

van7hu

----
I think this should be linked with or merged with http://wiki.osdev.org/System_V_ABI

mrvn
````

<a id="w003003"></a>

## W003003 — MFS

Bron-ID: W003003
Namespace: Main (0)
Revisie: 30686 | Datum: 2026-06-03T18:21:23Z
Bron: https://wiki.osdev.org/MFS
Revisie-URL: https://wiki.osdev.org/index.php?title=MFS&oldid=30686

````text
{{Historical}}
{{Filesystems}}

The '''Macintosh File System''', or '''MFS''', is the original Macintosh filesystem created for use on 400K floppy disks. It was first released in January 1984, and effectively discontinued September 1985 with the introduction of [[HFS]]. Write support was removed in Mac OS version 7.6, and read support was removed in version 8. 

==Overview==
MFS is a very limited filesystem. This means it's simple to implement, but less efficient than its successors. All files are stored in a single directory.

An MFS volume consists of a boot block, a master directory block, some file directory blocks, and some allocation blocks. It may also have a backup master directory block, but this is optional. These blocks are each defined in terms of 512-byte sectors.

Unless otherwise specified, all values are big-endian (MSB first).

==Boot Block==
The boot block is the first two sectors of the volume. Boot code can be placed here to make a bootable disk.

==Master Directory Block==
The Master Directory Block (MDB) is the two sectors immediately following the boot block. It consists of the volume information and the block map. The format of the volume information is:

{| class="wikitable"
|- 
! Offset (in bytes) 	
! Size (in bytes)
! Typical value (400K disk)
! Meaning
|- 
| 0
| 2
| 0xD2D7
| Signature. This must always be the value 0xD2D7 to identify a MFS volume.
|- 
| 2
| 4
|
| Creation date, in seconds since midnight January 1st 1904.
|- 
| 6
| 4
|
| Last backup date, in seconds since midnight January 1st 1904.
|- 
| 10
| 2
| 0x0000
| Attributes. Bit 7 is set if the volume is locked by hardware. Bit 15 is set if the volume is locked by software. Other bits are unknown.
|- 
| 12
| 2
|
| Number of files.
|- 
| 14
| 2
| 4
| File directory start. This is the first sector of the directory, relative to the start of the volume.
|- 
| 16
| 2
| 12
| File directory length. This is the length of the directory, in sectors.
|- 
| 18
| 2
| 391
| Number of allocation blocks.
|- 
| 20
| 4
| 1024
| Allocation block size, in bytes. This may be any multiple of 512, not just powers of 2.
|- 
| 24
| 4
| 8192
| Clump size, in bytes. This is a hint to the filesystem driver for how many allocation blocks to reserve when increasing the size of a file. This must be a multiple of the allocation block size.
|- 
| 28
| 2
| 16
| Allocation block start. This is the first sector of the first allocation block, relative to the start of the volume.
|- 
| 30
| 4
|
| Next file number. This is a hint to the filesystem driver for which file number it should assign to the next file.
|- 
| 34
| 2
|
| Free allocation blocks.
|- 
| 36
| 28
|
| Volume name. This field has a fixed length, but it contains a variable-length Pascal string. The first byte indicates the number of characters in the string, and that many of the following bytes contain the string. Any remaining bytes should be padded with zeroes.
|}

Immediately following the volume information is the block map. The block map is an array of 12-bit entries, packed together MSB-first. Every three bytes contains two entries. Each 12-bit entry points to the next allocation block of the file. It's similar to the file allocation table in [[FAT|FAT12]], except FAT12 is LSB-first.

Like FAT, allocation blocks are numbered starting from 2. However, unlike FAT, the block map begins with allocation block 2 instead of skipping the first two entries.

Some values have special meanings: 0x000 means the current allocation block is free, 0x001 means the current allocation block is the last allocation block of the file, and 0xFFF means the current allocation block contains some of the file directory blocks.

==File Directory Blocks==

The file directory blocks are a contiguous set of sectors on the disk. Each file directory block is one sector. Typically, they are located between the MDB and the first allocation block, but they may also be between the last allocation block and the backup MDB, or contained within some of the allocation blocks.

Variable-length directory entries are placed in the file directory blocks. Entries are placed starting at the first byte of the block, and do not cross block boundaries. Entries are padded to 2-byte aligned addresses. The format of the directory entries is:

{| class="wikitable"
|- 
! Offset (in bytes) 	
! Size (in bytes)
! Meaning
|- 
| 0
| 1
| Flags. Bit 7 is always set in a directory entry; when clear, it indicates empty space with no further directory entries in this block. Bit 0 is set if the file is locked. Other bits are unknown.
|- 
| 1
| 1
| Version. Always 0x00.
|- 
| 2
| 4
| File type. This is often (but not always) a readable 4-character string. When the type is unknown, it should be set to 0x3F3F3F3F.
|- 
| 6
| 4
| File creator. This is often (but not always) a readable 4-character string. When the appropriate application to open the file is unknown, it should be set to 0x3F3F3F3F.
|- 
| 10
| 2
| Finder flags. Some of these flags mean different things depending on the Finder version, so they're not documented here. Applications may require some of these bits to be set, but most other files do not.
|- 
| 12
| 4
| Position. This determines where in the window the file's icon will be displayed. This may safely be set to 0 when it's not needed.
|- 
| 16
| 2
| Folder number. This determines which window the file will be displayed in. The value 0 means the main volume window, -2 means the desktop, and -3 means the trash. A positive value is the ID number for a folder.
|- 
| 18
| 4
| File number. This number must be unique for each file in the volume.
|- 
| 22
| 2
| First data fork allocation block. If the data fork has no allocation blocks, this should be 0.
|- 
| 24
| 4
| Data fork size, in bytes.
|- 
| 28
| 4
| Data fork allocated space, in bytes. This is the total size of all the allocation blocks belonging to the data fork, including any space that isn't used.
|- 
| 32
| 2
| First resource fork allocation block. If the resource fork has no allocation blocks, this should be 0.
|- 
| 34
| 4
| Resource fork size, in bytes.
|- 
| 38
| 4
| Resource fork allocated space, in bytes. This is the total size of all the allocation blocks belonging to the resource fork, including any space that isn't used.
|- 
| 42
| 4
| Creation date, in seconds since midnight January 1st 1904.
|- 
| 46
| 4
| Modification date, in seconds since midnight January 1st 1904.
|- 
| 50
| 1-256
| File name. This field has a variable length and contains a Pascal string. The first byte indicates the number of characters in the string, and that many of the following bytes contain the string.
|}

==Backup Master Directory Block==

If present, the backup MDB will be in the last two sectors of the volume. It's a copy of the MDB, so it shares the same layout.

== See Also ==
=== External Links ===
* [https://www.pagetable.com/?p=50 Inside Macintosh], pages II-119 to II-123
* [http://www.mactech.com/articles/mactech/Vol.01/01.05/Disks/index.html Disk Organization]
* [https://developer.apple.com/library/archive/samplecode/MFSLives/Introduction/Intro.html Sample Mac VFS plugin implementing read-only MFS support]

[[Category: Filesystems]]
````

<a id="w003004"></a>

## W003004 — User talk:Communistpancake

Bron-ID: W003004
Namespace: User talk (3)
Revisie: 11715 | Datum: 2011-07-04T20:09:42Z
Bron: https://wiki.osdev.org/User_talk:Communistpancake
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ACommunistpancake&oldid=11715

````text
== Wikipedia page creations ==

Hi there, please check [http://forum.osdev.org/viewtopic.php?p=193493#p193493 this post]. I am not sure if copy & paste from Wikipedia is a thing that we should do... -- [[User:Solar|Solar]] 12:23, 4 July 2011 (UTC)
:Because I was using it as a source, and not actually copy and pasting, I think this is covered in the same way that I could source it on a paper;  but if you want to be safe, the information could either be removed or licensed under the same license. --[[User:Communistpancake|Communistpancake]] 20:09, 4 July 2011 (UTC)
````

<a id="w003005"></a>

## W003005 — OSDev Wiki:Copyrights

Bron-ID: W003005
Namespace: OSDev Wiki (4)
Revisie: 11734 | Datum: 2011-07-07T21:00:58Z
Bron: https://wiki.osdev.org/OSDev_Wiki:Copyrights
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDev_Wiki%3ACopyrights&oldid=11734

````text
All content of this wiki, added since June 6 of 2011, is licensed under [http://creativecommons.org/publicdomain/zero/1.0/ CC0]. This is the legal equivalent to public domain licensing, designed to include provisions for countries where law forbids waiving of copyright. A copy of the CC0 1.0 Universal license follows.

=Statement of Purpose=

The laws of most jurisdictions throughout the world automatically confer exclusive Copyright and Related Rights (defined below) upon the creator and subsequent owner(s) (each and all, an "owner") of an original work of authorship and/or a database (each, a "Work").

Certain owners wish to permanently relinquish those rights to a Work for the purpose of contributing to a commons of creative, cultural and scientific works ("Commons") that the public can reliably and without fear of later claims of infringement build upon, modify, incorporate in other works, reuse and redistribute as freely as possible in any form whatsoever and for any purposes, including without limitation commercial purposes. These owners may contribute to the Commons to promote the ideal of a free culture and the further production of creative, cultural and scientific works, or to gain reputation or greater distribution for their Work in part through the use and efforts of others.

For these and/or other purposes and motivations, and without any expectation of additional consideration or compensation, the person associating CC0 with a Work (the "Affirmer"), to the extent that he or she is an owner of Copyright and Related Rights in the Work, voluntarily elects to apply CC0 to the Work and publicly distribute the Work under its terms, with knowledge of his or her Copyright and Related Rights in the Work and the meaning and intended legal effect of CC0 on those rights.

= Copyright and Related Rights =
A Work made available under CC0 may be protected by copyright and related or neighboring rights ("Copyright and Related Rights"). Copyright and Related Rights include, but are not limited to, the following:

# the right to reproduce, adapt, distribute, perform, display, communicate, and translate a Work;
# moral rights retained by the original author(s) and/or performer(s);
# publicity and privacy rights pertaining to a person's image or likeness depicted in a Work;
# rights protecting against unfair competition in regards to a Work, subject to the limitations in paragraph 4(a), below;
# rights protecting the extraction, dissemination, use and reuse of data in a Work;
# database rights (such as those arising under Directive 96/9/EC of the European Parliament and of the Council of 11 March 1996 on the legal protection of databases, and under any national implementation thereof, including any amended or successor version of such directive); and
# other similar, equivalent or corresponding rights throughout the world based on applicable law or treaty, and any national implementations thereof.

= Waiver =
To the greatest extent permitted by, but not in contravention of, applicable law, Affirmer hereby overtly, fully, permanently, irrevocably and unconditionally waives, abandons, and surrenders all of Affirmer's Copyright and Related Rights and associated claims and causes of action, whether now known or unknown (including existing as well as future claims and causes of action), in the Work (i) in all territories worldwide, (ii) for the maximum duration provided by applicable law or treaty (including future time extensions), (iii) in any current or future medium and for any number of copies, and (iv) for any purpose whatsoever, including without limitation commercial, advertising or promotional purposes (the "Waiver"). Affirmer makes the Waiver for the benefit of each member of the public at large and to the detriment of Affirmer's heirs and successors, fully intending that such Waiver shall not be subject to revocation, rescission, cancellation, termination, or any other legal or equitable action to disrupt the quiet enjoyment of the Work by the public as contemplated by Affirmer's express Statement of Purpose.

= Public License Fallback =
Should any part of the Waiver for any reason be judged legally invalid or ineffective under applicable law, then the Waiver shall be preserved to the maximum extent permitted taking into account Affirmer's express Statement of Purpose. In addition, to the extent the Waiver is so judged Affirmer hereby grants to each affected person a royalty-free, non transferable, non sublicensable, non exclusive, irrevocable and unconditional license to exercise Affirmer's Copyright and Related Rights in the Work (i) in all territories worldwide, (ii) for the maximum duration provided by applicable law or treaty (including future time extensions), (iii) in any current or future medium and for any number of copies, and (iv) for any purpose whatsoever, including without limitation commercial, advertising or promotional purposes (the "License"). The License shall be deemed effective as of the date CC0 was applied by Affirmer to the Work. Should any part of the License for any reason be judged legally invalid or ineffective under applicable law, such partial invalidity or ineffectiveness shall not invalidate the remainder of the License, and in such case Affirmer hereby affirms that he or she will not (i) exercise any of his or her remaining Copyright and Related Rights in the Work or (ii) assert any associated claims and causes of action with respect to the Work, in either case contrary to Affirmer's express Statement of Purpose.

= Limitations and Disclaimers =

# No trademark or patent rights held by Affirmer are waived, abandoned, surrendered, licensed or otherwise affected by this document.
# Affirmer offers the Work as-is and makes no representations or warranties of any kind concerning the Work, express, implied, statutory or otherwise, including without limitation warranties of title, merchantability, fitness for a particular purpose, non infringement, or the absence of latent or other defects, accuracy, or the present or absence of errors, whether or not discoverable, all to the greatest extent permissible under applicable law.
# Affirmer disclaims responsibility for clearing rights of other persons that may apply to the Work or any use thereof, including without limitation any person's Copyright and Related Rights in the Work. Further, Affirmer disclaims responsibility for obtaining any necessary consents, permissions or other rights required for any use of the Work.
# Affirmer understands and acknowledges that Creative Commons is not a party to this document and has no duty or obligation with respect to this CC0 or use of the Work.
````

<a id="w003006"></a>

## W003006 — User:Communistpancake/SFSBrowser

Bron-ID: W003006
Namespace: User (2)
Revisie: 11713 | Datum: 2011-07-04T20:03:30Z
Bron: https://wiki.osdev.org/User:Communistpancake/SFSBrowser
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACommunistpancake%2FSFSBrowser&oldid=11713

````text
Coming Soon!
````

<a id="w003007"></a>

## W003007 — MediaWiki:Copyrightwarning2

Bron-ID: W003007
Namespace: MediaWiki (8)
Revisie: 11733 | Datum: 2011-07-07T20:49:00Z
Bron: https://wiki.osdev.org/MediaWiki:Copyrightwarning2
Revisie-URL: https://wiki.osdev.org/index.php?title=MediaWiki%3ACopyrightwarning2&oldid=11733

````text
By submitting this content, You declare that you have read and understand the terms and intended legal effect of [[OSDev_Wiki:Copyrights|CC0]], and hereby voluntarily elect to apply it to this work. You understand that by doing so you waive all copyright and related or neighboring rights together with all associated claims and causes of action with respect to this work to the extent possible under the law (see $1 for details). <br />
You also declare that you wrote the content yourself or copied it from a public domain or other CC0-licensed source. <strong>DO NOT SUBMIT COPYRIGHTED WORK WITHOUT PERMISSION!</strong>
````

<a id="w003009"></a>

## W003009 — User:Combuster/Relicensing

Bron-ID: W003009
Namespace: User (2)
Revisie: 27600 | Datum: 2023-01-19T19:50:55Z
Bron: https://wiki.osdev.org/User:Combuster/Relicensing
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACombuster%2FRelicensing&oldid=27600

````text
Users that need to relicense their work to CC0

(m) means only minor updates, (-) means no updates, '''bold''' means key editor

===Checklist ===

# Check which users have actually contributed copyrightable material
# Get relevant people to post a license statement.
::Users in A-C have been sent a PM using the forum system (which means I can get a read confirmation, but also means limited requests pending) Messages sent @ 11 aug`
::Users D-E @ 13 aug
::Users F @ 16 aug
::Users G-H @ 18 aug
::Users I @ 25 aug
::Users J-K @ 27 sep (after cleaning up the outbox for space)
::Users L-Me @ 3 okt
::Users M @ 9 okt
::Users N @ 26 okt (cleared and grouped up to H)
::Users O-Q @ 18 nov (cleared up to I)
::Users R @ 13 feb
::Users S @ 4 may (cleared up to P)

'''Scheduled for removal'''
# <strike>ehenkes (answered by PM)</strike>
# <strike>Almafeta</strike>

'''Problem cases'''
# Comradeslice (PM read and ignored) (sent 2nd attempt)
# Manonthemoon (has private messages disabled)
# Jaywalker (PM read and ignored) (sent 2nd attempt)
# Jeremiah Griffin/Jeremiahzg/Nokurn (PM read and ignored, renamed account since)
# Kandu (PM read and ignored) (sent 2nd attempt)
# Karekare0/Zenith (PM read and ignored)
# Kieran (PM read and ignored) (sent 2nd attempt)
# Ldenyer (PM read and ignored) (sent 2nd attempt)
# Matthias.fauconneau (PM read and ignored) (sent 2nd attempt)
# Mbrush (PM read and ignored)
# Miker00lz (PM read and ignored)
# Iocoder/Mostafazizo(PM read and ignored) (sent 2nd attempt)
# Nathanjphillips (pm read and ignored)
# Nekros (PM read and ignored)
Should probably mail the ignorers again

User contributions can be queried [https://wiki.osdev.org/index.php?title=Special%3AContributions&tagFilter=&contribs=user&target=Cr2&namespace=&year=2011&month=-1 here].

===A===
# <strike>A-OS</strike> (project page only)
# <strike>'''AJ''' / '''Aj'''</strike> ‎(Bureaucrat, Sysop) [http://forum.osdev.org/viewtopic.php?f=1&t=23838&p=194310#p194310 signed off]
# Aaronmiller
# Aboeing
# <strike>Abuashraf</strike> [http://forum.osdev.org/viewtopic.php?p=194925&f=1#p194925 signed off]
# <strike>Accelleon</strike> (m)
# <strike>'''Adek336'''</strike> [http://forum.osdev.org/viewtopic.php?p=194831&f=1#p194831 signed off]
# <strike>Admin ‎(Bureaucrat, Sysop)</strike> (-)
# <strike>Aggieben</strike> (-)
# <strike>AirFlight</strike> (m)
# <strike>'''Alboin'''</strike> [http://forum.osdev.org/viewtopic.php?f=15&t=23838&start=105#p198171 signed off]
# <strike>Alethiophile</strike> [http://forum.osdev.org/viewtopic.php?p=194833&f=1#p194833 signed off]
# <strike>Alex92</strike> (m)
# <strike>AlexExtreme / Alexextreme</strike> [http://forum.osdev.org/viewtopic.php?p=193787&f=1#p193787 signed off]
# <strike>Alexhk</strike>
# <strike>Alexis211</strike> [http://forum.osdev.org/viewtopic.php?p=246090&f=1#p246090 signed off]
# <strike>'''AlfaOmega08 / Alfaomega08'''</strike> [http://forum.osdev.org/viewtopic.php?p=196531&f=1#p196531 signed off]
# <strike>Almafeta</strike> (contributions removed, 3-10-2011)
# <strike>AltCtrlDel</strike> (-)
# <strike>Anberlin</strike> (m)
# <strike>Andreaorru</strike> (m)
# <strike>Andreasots</strike> (m)
# Android Mouse
# <strike>Andyhhp</strike> [http://forum.osdev.org/viewtopic.php?p=194874&f=13#p194874 signed off]
# <strike>Anog</strike> (m)
# <strike>Anti149</strike> (m)
# <strike>Arbaal</strike> (m)
# <strike>Archil</strike> (-)
# <strike>Ariethenor</strike> (-) [http://forum.osdev.org/viewtopic.php?p=204012&f=1#p204012 signed off anyway]
# <strike>Arrowofdarkness</strike> (m)
# <strike>Arunsk</strike> (-)
# <strike>Ash</strike> (-)
# <strike>Assembler</strike> [http://forum.osdev.org/viewtopic.php?p=195494&f=1#p195494 signed off]
# <strike>Astei</strike> (m)
# <strike>Aweiredguy</strike> (m)
# <strike>Axalto</strike> (m)
# Axlaiden
# <strike>Ayushjsh</strike> (-)
# <strike>Aztex</strike> (m)

===B===
# B.E
# <strike>B.zaar</strike> [http://forum.osdev.org/viewtopic.php?p=194835&f=15#p194835 signed off]
# <strike>BackwardsDown</strike> (-)
# <strike>Badsector</strike> (m)
# <strike>Barbossa404</strike> (-)
# <strike>Bastl</strike> [http://forum.osdev.org/viewtopic.php?p=194871&f=1#p194871 signed off]
# <strike>Bazi</strike> (m)
# <strike>Bcat</strike> (-)
# <strike>Bellezza</strike> (-) [http://forum.osdev.org/viewtopic.php?p=201145&f=11#p201145 signed off anyway]
# <strike>Beraldo</strike> (m)
# <strike>Bergen110</strike> (m)
# <strike>Berkeleyrc</strike> (m)
# <strike>Berkus</strike> [http://forum.osdev.org/viewtopic.php?p=193670#p193670 signed off]
# <strike>Bernhard</strike> [http://forum.osdev.org/viewtopic.php?p=195483&f=1#p195483 signed off]
# <strike>Better0332</strike> (-)
# <strike>'''Bewing'''</strike> [http://forum.osdev.org/viewtopic.php?p=210774&f=1#p210774 signed off]
# <strike>Beyondsociety</strike> (-)
# <strike>Bifferos</strike> (-)
# Bjork
# Blackcatcoder
# <strike>Blacklight</strike> (-) [http://forum.osdev.org/viewtopic.php?p=194007&f=1#p194007 signed off]
# <strike>Blah3</strike> (m)
# Blound
# <strike>Bluechill</strike> (m)
# <strike>Bluecode</strike> (m)
# <strike>Bluemoon</strike> (m) [http://forum.osdev.org/viewtopic.php?p=193700&f=8#p193700 signed off]
# <strike>Bobalandi</strike> (-)
# <strike>Bobman801</strike> (-)
# <strike>Bonefish</strike> (m)
# <strike>Bregma</strike> (m)
# <strike>'''Brendan''' ‎(Bureaucrat, Sysop)</strike> [http://forum.osdev.org/viewtopic.php?p=193678&f=1#p193678 signed off]
# <strike>Brodeur235</strike> (m)
# <strike>'''Brynet-Inc / Brynet-inc'''</strike> [http://forum.osdev.org/viewtopic.php?p=193714&f=8#p193714 signed off]
# <strike>Bubach</strike> (-) [http://forum.osdev.org/viewtopic.php?p=203272&f=1#p203272 signed off anyway]
# <strike>Bufferoverflow</strike> (-)
# <strike>Bughunter</strike> (m) (Thanks for fixing 52 edits of typos)
# <strike>Burner</strike> (-)

===C===
# <strike>Candy</strike> ‎(Bureaucrat, Sysop) [http://forum.osdev.org/viewtopic.php?f=8&t=23838&start=75#p196938 signed off]
# CarbonBased
# <strike>Carib</strike> (-)
# <strike>Cb88</strike> [http://forum.osdev.org/viewtopic.php?f=15&t=23838&start=105#p197832 signed off]
# <strike>Celtic</strike> (m)
# <strike>Cg123</strike> (m)
# <strike>Chandra</strike> [http://forum.osdev.org/viewtopic.php?p=196997&f=1#p196997 signed off]
# <strike>Charlemagne</strike> [http://forum.osdev.org/viewtopic.php?p=196987&f=1#p196987 signed off]
# Charles
# '''Chase ‎(Bureaucrat, Sysop)'''
# <strike>Cheery</strike> (-)
# <strike>Chexmix</strike> (-)
# <strike>Chosenoreo</strike> (-)
# <strike>Chrisskura</strike> (m)
# <strike>Cic</strike> [http://forum.osdev.org/viewtopic.php?p=194834&f=1#p194834 signed off]
# <strike>Ciprian</strike> (m)
# Cjmovie
# <strike>Clange</strike> (m) [http://forum.osdev.org/viewtopic.php?f=8&t=23838&p=193859#p193859 signed off]
# <strike>Coddy</strike> (m)
# <strike>Code0x58</strike> (-)
# <strike>CodeAnxiety</strike> (m)
# <strike>Codejunkie</strike> (-)
# <strike>Cognition</strike> [http://forum.osdev.org/viewtopic.php?p=194934&f=13#p194934 signed off]
# <strike>Colonel Kernel</strike> (m)
# Com1
# <strike>Comafire</strike> (-)
# <strike>'''Combuster ‎(Bureaucrat, Sysop)'''</strike> [http://forum.osdev.org/viewtopic.php?p=91295#p91295 signed off]
# <strike>Communistpancake</strike> [http://forum.osdev.org/viewtopic.php?p=194840&f=15#p194840 signed off]
# <strike>Computafreak</strike> [http://forum.osdev.org/viewtopic.php?p=193753&f=8#p193753 signed off]
# ComputerPsi
# <strike>Comradeslice</strike> (m) (PM read and ignored)
# <strike>Cooldude</strike> (-)
# <strike>Cosmo</strike> (-)
# <strike>Coty</strike> (-)
# <strike>Cplh</strike> (-)
# <strike>Cr2</strike> (m)
# <strike>Craze Frog / Craze frog</strike> [http://forum.osdev.org/viewtopic.php?p=196966&f=1#p196966 signed off]
# Crazed123
# Crazygray1
# <strike>'''Creature'''</strike> [http://forum.osdev.org/viewtopic.php?p=193819&f=1#p193819 signed off]
# <strike>Crosse3</strike> (m)
# <strike>CrypticalCode0</strike> (-) [http://forum.osdev.org/viewtopic.php?p=199145&f=1#p199145 signed off anyway]
# <strike>Cyb70289</strike> [http://forum.osdev.org/viewtopic.php?p=195260&f=1#p195260 signed off]
# <strike>Cygius</strike> (-)
# <strike>Cyr1x</strike> [http://forum.osdev.org/viewtopic.php?p=193918&f=1#p193918 signed off]
# <strike>Czernobyl</strike> (-) [http://forum.osdev.org/viewtopic.php?p=215240&f=1#p215240 signed off anyway]

===D===
# <strike>D722002</strike> (m)
# <strike>Daedalus</strike> (-)
# <strike>Daemoned</strike> [http://forum.osdev.org/viewtopic.php?p=195017&f=1#p195017 signed off]
# <strike>Dajoh</strike> (m)
# <strike>Dak91</strike> (m)
# <strike>Danielholt</strike> (-)
# <strike>Darkinsanity</strike> [http://forum.osdev.org/viewtopic.php?p=194935&f=13#p194935 signed off]
# <strike>Darwish</strike> [http://forum.osdev.org/viewtopic.php?p=194919&f=1#p194919 signed off]
# <strike>Das</strike> (m)
# <strike>Dataflash</strike> (-)
# <strike>David</strike> (contributions removed, 2019-06-06)
# <strike>David003</strike> (-)
# <strike>Davidbg</strike> (m)
# <strike>Davidcary</strike> (m)
# <strike>Deadmutex</strike> (-)
# <strike>Deathangel</strike> (-)
# <strike>Dedndave</strike> [http://forum.osdev.org/viewtopic.php?p=194915&f=1#p194915 signed off]
# <strike>Denn1s</strike> (m)
# <strike>Derekbank</strike> [http://forum.osdev.org/viewtopic.php?p=195110&f=1#p195110 signed off]
# <strike>Desl</strike> (m)
# Deste
# <strike>Devin122</strike> (m)
# <strike>Dex</strike> (m)
# <strike>Dhanlouie</strike> (m)
# DinamytE
# <strike>Djsilence</strike> (-)
# Dkelly
# <strike>Donkeeland</strike> (m)
# <strike>Drlink</strike> (-)
# <strike>Dross</strike> (-)
# <strike>Dswimboy</strike> (m)
# Dude
# <strike>Dunkaist</strike> (-)
# <strike>Durand</strike> (m)
# <strike>Dushara</strike> (m)
# Dustwolf
# <strike>Dword</strike>(-)

===E===
# <strike>Earlz</strike> [http://forum.osdev.org/viewtopic.php?p=193692&f=8#p193692 signed off]
# <strike>Earthenware</strike> (-)
# <strike>Eboyjr</strike> (-)
# <strike>Ecco</strike> (-)
# <strike>Eddyb</strike> (m) [http://forum.osdev.org/viewtopic.php?p=193999&f=1#p193999 signed off]
# <strike>Eeyore</strike> (-)
# <strike>Egytatto</strike> (-)
# <strike>Ehenkes</strike> (contributions removed)
# <strike>Einsteinjunior</strike> (-)
# <strike>Electro</strike> (-)
# Elemental
# <strike>Emrlddrgn</strike> (-)
# <strike>Enigma</strike> (m)
# <strike>Enrikemal</strike> (contributions removed)
# Erikbomb
# <strike>Eryjus</strike> (-) [http://forum.osdev.org/viewtopic.php?p=198599&f=8#p198599 signed off anyway]
# <strike>Esotechnica</strike> (-)
# Essial
# <strike>Etiennelorrain</strike> [http://forum.osdev.org/viewtopic.php?p=195024&f=1#p195024 signed off]
# <strike>Ethereal</strike> (-) [http://forum.osdev.org/viewtopic.php?p=195531&f=13#p195531 signed off anyway]
# <strike>Eugene2k</strike> [http://forum.osdev.org/viewtopic.php?p=194888&f=1#p194888 signed off]
# <strike>Exolu</strike> (-) [http://forum.osdev.org/viewtopic.php?p=213563&f=1#p213563 signed off anyway]

===F===
# <strike>F2</strike> (m)
# <strike>FMota</strike> [http://forum.osdev.org/viewtopic.php?p=195126&f=1#p195126 signed off]
# <strike>Falstart</strike> (-)
# <strike>Fanael</strike> [http://forum.osdev.org/viewtopic.php?p=195371&f=1#p195371 signed off]
# <strike>Fate</strike> [http://forum.osdev.org/viewtopic.php?p=195112&f=1#p195112 signed off]
# <strike>Felipe</strike> [http://forum.osdev.org/viewtopic.php?p=195481&f=1#p195481 signed off]
# <strike>Ferrarius</strike> [http://forum.osdev.org/viewtopic.php?p=195115&f=1#p195115 signed off]
# <strike>Flashburn</strike> [http://forum.osdev.org/viewtopic.php?p=195140&f=1#p195140 signed off]
# <strike>Flashjonny</strike> (-)
# Flexo
# <strike>Flyonok</strike> (-)
# <strike>Fnoble</strike> (-)
# <strike>Franchie</strike> (-)
# <strike>Frank</strike> [http://forum.osdev.org/viewtopic.php?p=195120&f=1#p195120 signed off]
# <strike>Fridget</strike> (-)
# <strike>Froggey</strike> (-)
# <strike>Fronty</strike> (m)
# FutureDomain
# <strike>Fys</strike> (-)

===G===
# Gabrield
# <strike>Gaidheal</strike> [http://forum.osdev.org/viewtopic.php?p=195178&f=1#p195178 copyright transferred] - See Chase
# <strike>Garob</strike> (m) [http://forum.osdev.org/viewtopic.php?p=197307&f=2#p197307 signed off anyway]
# <strike>Garystampa</strike> (-)
# <strike>GAT</strike> (-) [http://forum.osdev.org/viewtopic.php?p=203919&f=1#p203919 signed off anyway]
# <strike>Gduranceau</strike> (m)
# <strike>Gedd</strike> [http://forum.osdev.org/viewtopic.php?p=195348&f=13#p195348 signed off]
# <strike>Gegelaile</strike> (-)
# <strike>Gerryg400</strike> (-)
# Gh975223
# Gh975223b
# Gh975443
# <strike>Gmoney</strike> (-)
# <strike>Godoflaziness</strike> (m)
# <strike>Googol</strike> (-)
# <strike>Gpmelendez</strike> (m)
# <strike>Graham</strike> (uses "minor" where not appropriate) [http://forum.osdev.org/viewtopic.php?p=195175&f=1#p195175 signed off]
# <strike>graphitemaster</strike> [http://forum.osdev.org/viewtopic.php?f=1&t=23838&start=150#p210775 signed off]
# <strike>Gravaera</strike> [http://forum.osdev.org/viewtopic.php?p=193724&f=8#p193724 signed off]
# <strike>Grunt</strike> (-)
# <strike>Gth759k</strike> (thanks for 55 project page updates)
# <strike>Guilt</strike> (m)
# <strike>Gzaloprgm</strike> (m)

===H===
# <strike>H311b0y</strike> (-)
# <strike>'''Hangin10'''</strike> [http://forum.osdev.org/viewtopic.php?p=195042&f=1#p195042 signed off]
# Happy Egghead
# <strike>Harrymc</strike> (m)
# <strike>Havok</strike> (m)
# <strike>Hazardoxide</strike> (-)
# <strike>Hckr83</strike> (m)
# <strike>Hektec</strike> (-)
# <strike>Hellwig</strike> (-)
# <strike>Hery Sasta </strike>(m)
# <strike>Hidnplayr</strike> (m)
# <strike>Hitchhikr</strike> (m)
# <strike>Hitech</strike> (-)
# <strike>Hitmanyesman</strike> (-)
# <strike>Holypanl</strike> (see gravaera)
# Hoozim
# Hyperdrive

===I===
# <strike>I586coder</strike> (m)
# INF1n1t
# <strike>Iammisc</strike> (m)
# <strike>Idiprima</strike> (m)
# <strike>Igilham</strike> (m)
# <strike>Ilent2</strike> [http://forum.osdev.org/viewtopic.php?p=195479&f=1#p195479 signed off]
# <strike>Immibis</strike> (-) [http://forum.osdev.org/viewtopic.php?p=194864&f=15#p194864 signed off]
# <strike>ImPerfect</strike> (-)
# Imate900
# <strike>Indy</strike> (-)
# Ineo
# Inflater
# <strike>Inotseeyou</strike> (m)
# <strike>Inx</strike> [http://forum.osdev.org/viewtopic.php?p=195477&f=1#p195477 signed off]
# Iocoder / Mostafazizo (PM read and ignored)
# <strike>Ipsemet</strike> (-)
# <strike>Iseyler</strike> (m)
# <strike>Itisiuk</strike> (-)
# <strike>Ixxi</strike> (-)

===J===
# <strike>JJeronimo / Jjeronimo</strike> [http://forum.osdev.org/viewtopic.php?f=8&t=23838&start=75#p196941 signed off]
# <strike>Jackpot51</strike> (m)
# <strike>'''Jackscott ‎(Bureaucrat, Sysop)'''</strike> [http://forum.osdev.org/viewtopic.php?p=194109&f=13#p194109 signed off]
# <strike>Jal</strike> [http://forum.osdev.org/viewtopic.php?p=196965&f=1#p196965 signed off]
# <strike>JamesM / Jamesm ‎(Bureaucrat, Sysop)</strike> [http://forum.osdev.org/viewtopic.php?p=196958&f=1#p196958 signed off]
# <strike>Jamesharris</strike> [http://forum.osdev.org/viewtopic.php?p=204101&f=8#p204101 signed off]
# <strike>Jaymay</strike> (m)
# Jaywalker (PM read and ignored)
# Jcomeauictx
# <strike>Jeko</strike> (-)
# Jeremiah Griffin/Jeremiahzg/Nokurn (PM read and ignored)
# <strike>Jeremyh</strike> (-)
# <strike>Jezze</strike> (m) [http://forum.osdev.org/viewtopic.php?p=193708&f=8#p193708 signed off]
# <strike>'''Jhawthorn'''</strike> [http://forum.osdev.org/viewtopic.php?f=8&t=23838&start=75#p196942 signed off]
# Jimmio92
# <strike>Jkillen</strike> (-)
# <strike>Jkrug</strike> (-)
# <strike>Jnc100</strike> [http://forum.osdev.org/viewtopic.php?p=194052&f=13#p194052 signed off]
# <strike>JoeTheProgrammer</strike> (m)
# <strike>Joedavis</strike> (-)
# <strike>JoeD / Pinged</strike> [http://forum.osdev.org/viewtopic.php?p=199879&f=1#p199879 signed off]
# Johnnythedon
# <strike>Johnsy2008</strike> (m)
# <strike>Jonatan44</strike> (-)
# <strike>Joshw</strike> (-)
# <strike>Jrw</strike> [http://forum.osdev.org/viewtopic.php?p=197385&f=1#p197385 signed off]
# <strike>Js</strike> (m) [http://forum.osdev.org/viewtopic.php?p=194322&f=8#p194322 signed off]
# <strike>Jtlb</strike> (-)
# Jules
# <strike>Juxas</strike> (-)
# <strike>Jvac</strike> [http://forum.osdev.org/viewtopic.php?f=15&t=23838&start=105#p198188 signed off]

===K===
# <strike>Kaiden</strike> (-)
# <strike>Kameo76890</strike> (-)
# Kandu (PM read and ignored)
# Karekare0 (see Zenith) (
# Kasper
# <strike>Kb1234</strike> (-)
# <strike>Kbradero</strike> (-)
# <strike>Kellycarter</strike> (-)
# <strike>Kenny</strike> [http://forum.osdev.org/viewtopic.php?p=194010&f=1#p194010 signed off]
# <strike>Kerravon</strike> (m)
# <strike>Kertelle</strike> (-)
# <strike>Kevin</strike> [http://forum.osdev.org/viewtopic.php?p=193694&f=8#p193694 signed off]
# Kieran (PM read and ignored)
# <strike>Kipras</strike> (m)
# <strike>Kissthink</strike> (m)
# <strike>'''Kmcguire ‎(Bureaucrat, Sysop)'''</strike> (see Pancake)
# Kmsr819
# Kmtdk
# <strike>Konstantinkoll</strike> (m)
# <strike>Kotuxguy</strike> [http://forum.osdev.org/viewtopic.php?f=8&t=23838&start=75#p196939 signed off]
# Kraks
# KrnlHckr
# <strike>Kscguru</strike> [http://forum.osdev.org/viewtopic.php?f=8&t=23838&start=75#p196946 signed off]
# <strike>Kubasz</strike> (-)
# <strike>Kuchen</strike> [http://forum.osdev.org/viewtopic.php?f=8&t=23838&start=75#p196951 signed off]
# <strike>Kutalmis</strike> (-)
# <strike>Kyle.tk</strike> (-)

===L===
# LOST
# <strike>Ladyserenakitty</strike> (m)
# <strike>Laksen</strike> [http://forum.osdev.org/viewtopic.php?p=195370&f=1#p195370 signed off]
# Ldenyer (PM read and ignored)
# <strike>Linjing</strike> (-)
# <strike>Linkdd</strike> (m)
# <strike>Linuxuser27</strike> (m)
# <strike>Lionel</strike> (m) [http://forum.osdev.org/viewtopic.php?p=202503&f=1#p202503 signed off]
# <strike>Liquid.silver</strike> (m)
# <strike>Lithium26</strike> [http://forum.osdev.org/viewtopic.php?p=197078&f=1#p197078 signed off]
# <strike>Lithorien</strike> [http://forum.osdev.org/viewtopic.php?p=197081&f=1#p197081 signed off]
# <strike>Littleengineer</strike> (-) [http://forum.osdev.org/viewtopic.php?p=194005&f=1#p194005 signed off]
# <strike>Lode</strike> (m)
# <strike>'''Love4boobies'''</strike> [http://wiki.osdev.org/index.php?title=User_talk:Combuster/Relicensing&oldid=11921 signed off]
# <strike>Luke2008</strike> (m)
# Lukem95
# <strike>Luns</strike> (m)
# <strike>Lvchao</strike> (-)

===M===
# M
# <strike>M35</strike> (-) [http://forum.osdev.org/viewtopic.php?p=194941&f=1#p194941 signed off]
# <strike>M-saunders</strike> (-)
# <strike>Mad70</strike> [http://forum.osdev.org/viewtopic.php?p=197092&f=15#p197092 signed off]
# <strike>Madanra</strike> [http://forum.osdev.org/viewtopic.php?p=196667&f=11#p196667 signed off]
# <strike>Madeofstaples</strike> [http://forum.osdev.org/viewtopic.php?p=193826&f=1#p193826 signed off]
# Magicaltux
# <strike>Maledictus</strike> (-)
# <strike>Malreddy</strike> (-)
# <strike>Manfred</strike> (m)
# <strike>Manktown</strike> (m)
# Manonthemoon (PMs disabled)
# <strike>Manugarg</strike> (m)
# <strike>Mariuszp</strike> [http://forum.osdev.org/viewtopic.php?p=201276&f=11#p201276 signed off]
# Markhobley
# <strike>Martijn</strike> [http://forum.osdev.org/viewtopic.php?p=197197&f=1#p197197 signed off]
# <strike>Mateus</strike> (m)
# <strike>Matik</strike> [http://forum.osdev.org/viewtopic.php?p=197079&f=1#p197079 signed off]
# <strike>Matthew</strike> [http://forum.osdev.org/viewtopic.php?p=198781&f=8#p198781 signed off]
# <strike>Matthias</strike> (m)
# Matthias.fauconneau (PM read and ignored)
# <strike>Mattvick</strike> (-)
# <strike>Maxmornev</strike> (-)
# Mbrush (PM read and ignored)
# <strike>Mdecky</strike> (m)
# <strike>Mdm</strike> [http://forum.osdev.org/viewtopic.php?p=197103&f=11#p197103 signed off]
# <Strike>Mduft</strike> [http://forum.osdev.org/viewtopic.php?p=197490&f=1#p197490 signed off]
# Me
# Mercy
# Meshounah
# <strike>'''MessiahAndrw / Messiahandrw'''</strike> [http://forum.osdev.org/viewtopic.php?p=208241&f=1#p208241 signed off]
# <strike>Metatechbe</strike> (m)
# <strike>Mh</strike> (-)
# Michaelslade
# <strike>Midecah</strike> (-)
# <strike>Midir</strike> [http://forum.osdev.org/viewtopic.php?p=197446&f=1#p197446 signed off]
# <strike>Mike</strike> [http://forum.osdev.org/viewtopic.php?p=197430&f=1#p197430 signed off]
# <strike>Mikegonta</strike> (-)
# Miker00lz (PM read and ignored)
# Mikeroz
# <strike>Milouz</strike> (m) (thanks for keeping the project page up to date)
# <strike>Milyges</strike> (m)
# <strike>Mmairs</strike> (-)
# <strike>Mmiikkee12</strike> (m)
# <strike>Mnemo</strike> (m)
# <strike>Modi</strike> (-)
# Montrom
# <strike>Mosman</strike> (m)
# <strike>Mostafa</strike> (-)
# Mostafazizo (see iocoder)
# <strike>Mr.Confuzed / Mr.confuzed</strike> [http://forum.osdev.org/viewtopic.php?p=197461&f=1#p197461 signed off]
# <strike>Mrkaktus</strike> (m)
# <strike>Mrkotfw</strike> (-) [http://forum.osdev.org/viewtopic.php?p=203346&f=1#p203346 signed off anyway]
# <strike>Mro</strike> (-)
# <strike>Multicoder</strike> (-)
# <strike>Mutex</strike> (-) [http://forum.osdev.org/viewtopic.php?p=196415&f=1#p196415 signed off anyway]
# <strike>Mystran ‎(Bureaucrat, Sysop)</strike> [http://forum.osdev.org/viewtopic.php?p=197420&f=1#p197420 signed off]

===N===
# Narida
# <strike>Nat682</strike> (m)
# <strike>Nathan</strike> [http://forum.osdev.org/viewtopic.php?f=1&t=23838&start=105#p198409 signed off]
# Nathanjphillips (pm read and ignored)
# <strike>Navisidhu</strike> (-)
# <strike>Nedbrek</strike> [http://forum.osdev.org/viewtopic.php?p=198521&f=1#p198521 signed off]
# Nekros (PM read and ignored)
# <strike>Neocoder</strike> (m)
# <strike>Neolander</strike> [http://forum.osdev.org/viewtopic.php?p=193682&f=1#p193682 signed off]
# <strike>Neon</strike> (m)
# <strike>Neonek</strike> (m)
# <strike>Nessphoro</strike> [http://forum.osdev.org/viewtopic.php?f=1&t=23838&start=135#p204321 signed off]
# <strike>Nickjohnson</strike> [http://forum.osdev.org/viewtopic.php?p=196599&f=1#p196599 signed off]
# Nicola
# <strike>Nixeagle</strike> (m)
# <strike>Nlinnenbank</strike> (m)
# Nokurn (see Jeremiah Griffin)
# <strike>Novayoshi</strike> [http://forum.osdev.org/viewtopic.php?p=198591&f=8#p198591 signed off]

===O===
# Oculusfervoris
# <strike>OdinPG</strike> (m)
# Omega
# <strike>OmegaMB</strike> (-)
# <strike>Omin0us</strike> (m)
# <strike>Omnitek</strike> (-)
# <strike>Oros</strike> [http://forum.osdev.org/viewtopic.php?p=199710&f=1#p199710 signed off]
# Os64dev ‎(Bureaucrat, Sysop)
# <strike>Osbios</strike> [http://forum.osdev.org/viewtopic.php?p=199639&f=1#p199639 signed off]
# <strike>Oscoder</strike> (m)
# Osdnlo
# Osprogram

===P===
# <strike>Palk</strike> [http://forum.osdev.org/viewtopic.php?p=199689&f=1#p199689 signed off]
# <strike>Pancake</strike> [http://forum.osdev.org/viewtopic.php?p=208618&f=1#p208618 signed off]
# Patrickv
# Paul84
# Paxcoder
# <strike>'''Pcmattman'''</strike> [http://forum.osdev.org/viewtopic.php?p=193816&f=1#p193816 signed off]
# <strike>Peapodamus</strike> (m)
# <strike>Pepito</strike> (-)
# <strike>Phible</strike> (m)
# <strike>Phpnerd</strike> [http://forum.osdev.org/viewtopic.php?p=247001&f=1#p247001 signed off]
# '''Pietro10'''
# <strike>Pillow</strike> [http://forum.osdev.org/viewtopic.php?p=199584&f=1#p199584 signed off]
# <strike>Pinged (See JoeD)</strike> [http://forum.osdev.org/viewtopic.php?p=199879&f=1#p199879 signed off]
# <strike>Piranha</strike> [http://forum.osdev.org/viewtopic.php?p=193709&f=8#p193709 signed off]
# <strike>Pitfall</strike> (-) [http://forum.osdev.org/viewtopic.php?p=197653&f=1#p197653 signed off anyway]
# <strike>Pmros</strike> (-)
# <strike>Prashant</strike> (m)
# <strike>Prave</strike> (m)
# <strike>Primis</strike> [http://forum.osdev.org/viewtopic.php?p=197477&f=1#p197477 signed off]
# <strike>Prino</strike> (-)
# <strike>Puffyfluff</strike> (m)
# <strike>Pulsar</strike> (-)
# <strike>Pyrofan1</strike> (m)

===Q (complete)===
# <strike>Qandrew</strike> (m)
# <strike>Quanganht</strike> [http://forum.osdev.org/viewtopic.php?p=199567&f=1#p199567 signed off]
# <strike>Quartsize</strike> (m)
# <strike>Quok ‎(Bureaucrat, Sysop)</strike> [http://forum.osdev.org/viewtopic.php?p=193690&f=8#p193690 signed off]

===R===
# <strike>R4z0rw0lf</strike> (-)
# <strike>Raistlinthewiz</strike> (m)
# <strike>Rajasrijan</strike> [http://forum.osdev.org/viewtopic.php?p=214906&f=1#p214906 signed off]
# <strike>Ramvi</strike> (m)
# <strike>Randomibis</strike> (m)
# <strike>Ratlink</strike> (-)
# <strike>Rdos</strike> (m)
# <strike>Recursive</strike> (-)
# <strike>Rene</strike> (m)
# <strike>Retsimx</strike> (m)
# <strike>Returninfinity</strike> (m)
# <strike>Revelation</strike> [http://forum.osdev.org/viewtopic.php?p=204100&f=8#p204100 signed off]
# <strike>Rizlah</strike> (m)
# Rjah
# <strike>Rknox</strike> (-)
# RobZed
# <strike>Rocko384</strike> (m)
# <strike>Rockrush</strike> (-)
# <strike>Rohedin</strike> (m)
# <strike>Roman2005</strike> (-)
# <strike>Romulojales</strike> (m)
# Rook
# <strike>Ru2aqare</strike> (m)
# Rudimk
# Rvsjoen
# <strike>Ryanliue</strike> (-)

===S===
# <strike>Sajjadser</strike> (-)
# <strike>Samuel2008</strike> (-)
# <strike>Samuelagm</strike> (-)
# Samueldotj
# <strike>Samypesse</strike> (m)
# <strike>Sanmukh</strike> (m)
# <strike>Sauos</strike> (-)
# <strike>Sbc19861004</strike> (-)
# Scalpel
# <strike>Scgtrp</strike> (m)
# <strike>Schneida</strike> (m)
# <strike>Schol-R-LEA / Schol-r-lea</strike> [http://forum.osdev.org/viewtopic.php?p=212219&f=1#p212219 signed off]
# <strike>Scorchos</strike> [http://forum.osdev.org/viewtopic.php?p=209446&f=1#p209446 signed off]
# <strike>Scoreunder</strike> (-)
# Serge (account missing, possilby from KolibriOS)
# <strike>Sergico</strike> (m)
# <strike>Serianox</strike> (-)
# <strike>Sevenbitbyte</strike> (-)
# Shalted
# Shedokan
# <strike>Shikhin</strike> (m) [http://forum.osdev.org/viewtopic.php?p=193783&f=1#p193783 signed off]
# <strike>Shiner</strike> (See eddyb)
# <strike>Shirish</strike> (m)
# <strike>Shrek</strike> (m)
# <strike>Shufflebits</strike> (m)
# Sidewaysmilk
# Sigaren
# Sigjuice
# <strike>Simonzilch</strike> (-)
# <strike>Sirprize</strike> (m)
# <strike>Slasher</strike> (-)
# <strike>Smiddy</strike> (m)
# <strike>Snake</strike> (m)
# <strike>'''Solar'''</strike> [http://forum.osdev.org/viewtopic.php?p=193676&f=1#p193676 signed off]
# <strike>Spacechase0</strike> (-)
# Speal
# <strike>Speedas</strike> (-)
# <strike>SpooK / Spook</strike> [http://forum.osdev.org/viewtopic.php?p=209397&f=1#p209397 signed off]
# <strike>Spydertl2/SpyderTL</strike> [http://forum.osdev.org/viewtopic.php?p=218493&f=1#p218493 signed off]
# <strike>Stealther</strike> [http://forum.osdev.org/viewtopic.php?p=209449&f=2#p209449 signed off] 
# <strike>Stephanos</strike> [http://forum.osdev.org/viewtopic.php?p=209381&f=1#p209381 signed off]
# <strike>Stephanvanschaik</strike> [http://forum.osdev.org/viewtopic.php?p=194113&f=13#p194113 signed off]
# <strike>Steve the Pirate / Steve the pirate</strike> (m)
# <strike>Steve23</strike> (m) (thanks for all the indexing work)
# <strike>Steveklabnik</strike> (m)
# <strike>Stevenup7002</strike> (m)
# Stevo14
# Stinkfly
# <strike>Stultus</strike> (-)
# <strike>Subrax</strike> (m)
# SuperSwampert67
# <strike>Surgo</strike> [http://forum.osdev.org/viewtopic.php?p=209394&f=1#p209394 signed off]
# <strike>Suthers</strike> (-)
# <strike>Synon</strike> (-) [http://forum.osdev.org/viewtopic.php?p=196935&f=1#p196935 signed off anyway]
# <strike>Syntropy</strike> (m)
# <strike>Szazesk2</strike> (m)
# Szh

===T===
# <strike>T0xic</strike> (m)
# <strike>Tanner.Gooding</strike> (-)
# '''Tantrikwizard'''
# <strike>Tarrox</strike> (-)
# <strike>Tehkittycat</strike> (-)
# Telexicon
# Teodor väänänen
# <strike>TheQuux / Thequux</strike> [http://forum.osdev.org/viewtopic.php?p=217435&f=1#p217435 signed off]
# Thedragon
# <strike>Thefearow</strike> (-)
# <strike>Theguillermo</strike> (-)
# <strike>TheInquisitive1</strike> (-) [http://forum.osdev.org/viewtopic.php?p=219257&f=1#p219257 signed off anyway]
# Thelamb
# <strike>'''Thepowersgang'''</strike> [http://forum.osdev.org/viewtopic.php?p=193686&f=1#p193686 signed off]
# <strike>Therx</strike> (-)
# <strike>Thestew42</strike> (m)
# <strike>Theubu</strike> (m)
# <strike>Thijs</strike> (-)
# <strike>Thomas</strike> (-)
# <strike>Thomasnilsen</strike> (m)
# <strike>Thor</strike> (m)
# <strike>Threesevenths</strike> (m)
# <strike>Tiniel</strike> (-)
# <strike>Tnick</strike> (-)
# <strike>Toaster</strike> (m)
# Tobba
# TomTom / Tomtom
# Tommy
# <strike>Tonay</strike> (-)
# Torshie
# Tosi
# <strike>Touch</strike> (-)
# Treaki
# <strike>Trinka</strike> [https://wiki.osdev.org/User:Trinka signed off]
# <strike>Trolly </strike>(-)
# '''Troy Martin / Troy martin'''
# <strike>TudorOS1</strike> (-) [http://forum.osdev.org/viewtopic.php?p=207832&f=1#p207832 signed off anyway]
# <strike>Twitch</strike> (-)
# <strike>Tyler</strike> [http://forum.osdev.org/viewtopic.php?p=244406&f=1#p244406 signed off]
# <strike>Tyleranon/TylerH</strike> [http://forum.osdev.org/viewtopic.php?p=193707&f=8#p193707 signed off]
# Tyra
# Tyrelhaveman

===U===
# Ucosty
# Uglyoldbob
# <strike>Untwisted</strike> (m)
# Uriel
# <strike>Urjaman</strike> (m)
# <strike>Urxae</strike> (m)
# <strike>Userspaceonly</strike> (m)
# <strike>Uzm386</strike> (m)
# Uzytkownik

===V===
# <strike>Van</strike> (-)
# Van7hu
# <strike>Varghese85</strike> (m)
# <strike>Vdigital</strike> (-)
# Viral
# <strike>'''Virtlink'''</strike> [http://forum.osdev.org/viewtopic.php?p=209044&f=1#p209044 signed off]
# <strike>Vsemaphore</strike> (-)
# <strike>Vital</strike> (-) [http://forum.osdev.org/viewtopic.php?p=256308&f=1#p256308 signed off anyway]

===W===
# <strike>'''Walling'''</strike> [http://forum.osdev.org/viewtopic.php?p=247054&f=1#p247054 signed off]
# <strike>Wangy414</strike> (-)
# Warthog
# <strike>Weirdcat</strike> (-)
# <strike>Weirdone</strike> (-)
# <strike>Wfelipe</strike> (m)
# <strike>Wilkie</strike> (m)
# Willware
# <strike>Winhee</strike> (-)
# Wolfwood
# <strike>Wsw</strike> (-)
# <strike>Wukl</strike> (Post-licence-change)

===X===
# <strike>Xanclic</strike> (m)
# Xarnze
# <strike>Xavier</strike> (-)
# <strike>Xddunce</strike> (-)
# Xenos
# <strike>Xvedejas</strike>
# Xyb
# Xyz1
# <strike>Xyzzy</strike> (See AlexExtreme)
===Y===
# Yargh
# <strike>'''Yayyak'''</strike> (See Jackscott), Signed off
# <strike>Yexo</strike> (-)
# Yuhong
# <strike>Yuriks</strike> (m)
===Z===
# <strike>Z0wnd</strike> (m)
# <strike>ZenMaster</strike> (m)
# Zenith (See Karekare0)
# Zhongyijun
# <strike>Zius</strike> (-)
# <strike>Ztane</strike> [http://forum.osdev.org/viewtopic.php?p=219349&f=1#p219349 signed off]
# Ziggyfish (see B.E)

===#===
# <strike>~</strike> (-) [http://forum.osdev.org/viewtopic.php?p=205409&f=11#p205409 signed off anyway]
# -HeBD-
# <strike>'''01000101'''</strike> [http://forum.osdev.org/viewtopic.php?p=193691&f=8#p193691 signed off]
# <strike>04wilsonm</strike> (contributions removed, 2019-06-06)
# <strike>0xc</strike> (m)
# <strike>4996mbsve</strike> (m)
# <strike>54616e6e6572</strike> [http://forum.osdev.org/viewtopic.php?p=209208&f=1#p209208 signed off]
````

<a id="w003010"></a>

## W003010 — Memory Management Unit

Bron-ID: W003010
Namespace: Main (0)
Revisie: 30832 | Datum: 2026-06-30T04:52:39Z
Bron: https://wiki.osdev.org/Memory_Management_Unit
Revisie-URL: https://wiki.osdev.org/index.php?title=Memory_Management_Unit&oldid=30832

````text
The '''MMU''', or '''Memory Management Unit''', is a component of many computers that handles memory translation, memory protection, and other purposes specific to each architecture.

== Translation ==
The MMU's main service to the computer is memory translation. Memory translation is a process by which virtual addresses are converted to physical addresses. We can say that the virtual addresses are ''mapped'' to the physical address. This gives us the ability to create a memory model in our own fashion. That is, we can rearrange how the memory ''seems'' to be ordered.

For instance, this technique is used when creating a [[Higher Half Kernel]]. The kernel is loaded at location x, but when [[paging]] is initialized the OS configures paging so that virtual address 0xC0000000 maps to the physical address where the kernel was loaded. This then creates the effect that the kernel actually '''is''' at 0xC0000000. 

== Protection ==
Because we can make memory seem however we want, we can make each process appear that it is the only process on the machine. Moreover, Because the OS can give each process its own virtual address space, the MMU can help prevent one process from directly reading or modifying another process's private memory. Processes may still share memory intentionally, and the kernel must configure permissions correctly. This means that if an application is to fail, ''it'' will fail, but nothing else.

== Discourse on Memory Management Units and Virtual Memory systems in contemporary architectures ==

A very simple overview of the theory involved in using virtual address spaces as a general rule. Article does not focus on any one architecture, but seeks to model a generic CPU with an MMU.

=== A general discourse on Virtual Memory systems ===

As a rule, a chipset (motherboard) would tend to have N bytes of physical memory. Physical memory is "real" memory which should be globally visible to all processors. Under normal operation, or rather, when the CPU is operating without its Paged [[Memory Management Unit]] turned on, any address the CPU encounters will bypass the (P)MMU and directly issue a physical memory access.

I'd like to move directly into the idea of a [[TLB]], and how "paging" works, etc. Many processor architectures of the day specify a set of behaviours that the processor will exhibit when the OS software activates the processor's PMMU. But what *is* a Memory Management Unit? An MMU is hardware that performs address translation and memory-protection checks. Many MMUs use a Translation Lookaside Buffer, or TLB, as a cache of recently used translations. When a processor allows multiple "virtual", independent address spaces to be used on a machine such that the CPU sees one stretch of "virtual" memory which can be mapped to any physical page, there must be some form of tabling, or other record-keeping of which physical frame each virtual page should cause the processor to eventually access on the address bus.

A processor with an MMU that provides virtual memory will commonly have one or more TLBs, which are on-chip caches of recent translations. Each "translation record/entry" tells the CPU the mapping of one virtual address to one physical address. Let us imagine this on-chip cache as a big lookup array of entries that are of this form:

<syntaxhighlight lang="C">
// Abstract model of a TLB.

typedef uintptr_t vaddr_t;
typedef uintptr_t paddr_t;

struct tlb_cache_record_t
{
   vaddr_t entry_virtual_address;
   paddr_t relevant_physical_address;
   uint16_t permissions;
   uint16_t flags;
};

// Instance of a hardware Translation Lookaside Buffer.
struct tlb_cache_record_t   hw_tlb[CPU_MODEL_MAX_TLB_ENTRIES];
</syntaxhighlight>

Your processor's TLB is essentially a hash lookup table of entries that tell what physical address each page refers to. When you enable paging, every address reference is sent out to the TLB for lookup. The CPU does something like this internally:

<syntaxhighlight lang="C">
// Model routine for a TLB lookup.

int tlb_lookup(vaddr_t v, paddr_t *p)
{
   for (int i=0; i<CPU_MODEL_MAX_TLB_ENTRIES; i++)
   {
      if (hw_tlb[i].flags & TLB_ENTRY_FLAGS_INUSE && hw_tlb[i].entry_virtual_address == v)
      {
         *p = hw_tlb[i].relevant_physical_address;
         return 1;
      };
   };
   return 0;
}
</syntaxhighlight>

If the TLB contains an entry for that virtual address, that virtual address's recorded physical address is returned. The CPU *does not* care about the actual state of the REAL translation in memory! *You* are responsible for ensuring that the information in the processor's TLB is correct. Let us pretend that our processor's TLB has an entry in it that records the virtual address 0xC0103000 as pointing to the physical address 0x11807000. Assume that your kernel has changed this information in the page tables in RAM; Your writing to physical RAM does not affect the on-chip TLB. UNLESS you tell the processor to flush that TLB entry for 0xC0103000 from its TLB, the next time 0xC0103000 is referenced, the CPU will look into the TLB, and go right on ahead to send out the physical address 0x11807000 that the TLB *says* the address corresponds to.

A processor architecture would normally, then provide an instruction to invalidate TLB entries, either en masse, or one by one, or however the CPU designers decided. Let's try to model a TLB flush. In our model CPU architecture, there is an instruction that software can issue which will invalidate one virtual address. It is called: TLBFLSH. An OS would invoke this on our model architecture by doing something like this:

<syntaxhighlight lang="C">
asm volatile ("TLBFLSH   %0\n\t"::"r" (virtual_address));
</syntaxhighlight>

And on to our model:
<syntaxhighlight lang="C">
// Modelled function for a flush of the TLB modelled earlier on.

void tlb_flush_single(vaddr_t v)
{
   for (int i=0; i<CPU_MODEL_MAX_TLB_ENTRIES; i++)
   {
      if (hw_tlb[i].flags & TLB_ENTRY_FLAGS_INUSE && hw_tlb[i].entry_virtual_address == v)
      {
         hw_tlb[i].flags &= ~TLB_ENTRY_FLAGS_INUSE;
         return;
      };
   };
}
</syntaxhighlight>

Once paging or another MMU-based virtual memory mechanism is enabled, ordinary instruction fetches and data reads or writes are translated before the corresponding physical memory access is completed. In the common case, the processor first checks the Translation Lookaside Buffer, or TLB, for a cached translation. If no usable TLB entry is found, the processor follows the architecture's slower translation path, such as a hardware page-table walk or a trap to an operating-system TLB refill handler.

The TLB is a cache used by the MMU. It stores recently used translations, usually from virtual pages to physical page frames, along with information such as permissions, address-space tags, and memory attributes. On a TLB hit, the processor can translate the address quickly without consulting the page tables in memory.

The TLB is '''not''' the whole MMU. The MMU is the larger address-translation and protection mechanism.

Depending on the architecture, it may contain TLBs, page-walk hardware, permission-checking logic, fault-generation logic, memory-attribute handling and even more translation-based caches.  

Page tables are normally data structures stored in memory and maintained by the operating system; they are not themselves the TLB, though hardware may read them or cache information derived from them.

When the operating system changes a mapping in its page tables, existing TLB entries may become stale. Writing new values into the page tables does not necessarily update translations already cached by the processor. The OS must therefore invalidate any affected cached translations before relying on the new mapping. On x86, this may involve instructions such as INVLPG, reloading CR3, or other invalidation mechanisms depending on the paging mode and CPU features.

There are two broad styles of MMU design:
# '''Hardware-managed TLBs''': The operating system builds page tables in an architecture-defined format. On a TLB miss, the processor walks these tables itself, fills the TLB if the translation is valid, and continues execution. If the translation is missing or violates permissions, the processor raises a page fault or similar exception. x86 is an example of this style.
# '''Software-managed TLBs''': On a TLB miss, the processor traps into the operating system. The OS then searches whatever address-space data structures it uses and explicitly loads a translation into the TLB. In this model, the hardware may not require the OS's internal page-table format to match a fixed architecture-defined layout.

For this reason, a TLB miss and a page fault should not be confused. A TLB miss only means that the translation was not found in the processor's translation cache. It may be resolved entirely by hardware, or it may require an OS refill handler, depending on the architecture. A page fault means that address translation or access permission checking failed in a way that must be reported to the operating system.

Thus, enabling paging places the kernel and user programs inside a virtual address space: the addresses used by instructions are interpreted through the current translation context. To change what virtual addresses refer to, the OS must update its translation data structures and ensure that stale TLB entries are invalidated.

== Theory Concretion: x86 32-bit non-PAE recursive page directory ==

This section describes the recursive page-directory trick for classic 32-bit x86 paging without PAE. PAE, long mode, and five-level paging use different paging structures, although the same general idea can be adapted. 
In this paging mode, the processor translates a 32-bit linear address using a page directory and page tables. The physical address of the current page directory is stored in CR3. On a TLB miss, the processor walks these paging structures. If the walk finds present entries with valid permissions, the translation can be cached and execution continues. If the walk fails or the access violates permissions, the processor raises a page-fault exception.

Get that very solidly, please: bytes in RAM are not purposeful; they do not inherently imply any meaning. Page tables are just bytes in RAM. You could very well decide to give a page table to a network card for a DMA transfer. Just as you could put the address of a network frame into CR3 and have the processor walk that. It's very possible that your invalid CR3 value would have, contiguous from it at the right offsets, the right bits set (PRESENT, WRITE, etc etc), and some frame address such that the CPU may even walk all the way to what it sees as a page table and find an entry to translate your fault address. This does not mean that this translation data the CPU interpreted as translation data was indeed translation data. The CPU takes your byte address in RAM and walks from there, *interpreting* those bytes as translation information.

Take the following example, then: You have a page directory at physical address 0x12345000. This page directory would of course have 1024 entries, numbered 0 to 1023. Since I want to jump into the meat of the subject, let's imagine that this page directory at 0x12345000's last entry, index 1023, points back to 0x12345000.

That is, pdir[1023] == 0x12345xxx, where 'xxx' represents the permission bits. Let us also imagine that pdir[0] points to a frame at 0x12344000, which the processor would interpret as a page table. This table has of course, its own 1024 page table entries. They map of course, the virtual addresses 0x0 to 0x3FFFFF to various frames in RAM. So right now, our example page directory looks like this:

<pre>
[Page directory at 0x12345000]:
entry 0000 | phys: (0x12344 << 12) | perms 0bxxxxxxxxxxxx |
entry ...
entry ...
entry 1023 | phys: (0x12345 << 12) | perms 0bxxxxxxxxxxxx |

[Page table at 0x12344000 that pdir[0] points to]:
entry 0000 | phys: (0x34567 << 12) | perms 0bxxxxxxxxxxxx |
entry ...
entry ...
entry 512  | phys: (0x72445 << 12) | perms 0bxxxxxxxxxxxx |
</pre>

This setup is such that:
pdir[0] == 0x12344xxx,
pdir[0], ptab[0] == 0x34567xxx,
pdir[0], ptab[512] == 0x72445xxx,
pdir[1023] == 0x12345xxx.

Let us simulate a page table walk to find out what physical address entry{pdir[0], ptab[512]} maps to; i.e, what physical address virtual address 0x200000 is mapped to.

<ol>
<li>Processor encounters an instruction that references 0x200xxx while paging is turned on. This reference must pass through the MMU.</li>
<li>Assume this translation was not found in the TLB. A TLB miss occurs. On x86-32, this causes the processor to walk the page tables rather than to trap into the OS for a software TLB refill.</li>
<li>The CPU takes the virtual address, 0x00200xxx and splits it up, 10 bits, 10 bits, and 12 bits.</li>
<li>The CPU now knows that it must index into entry 0x0 of the page directory pointed to by CR3, and then index into entry 0x200 (512) of the page table pointed to by the page directory in CR3.</li>
<li>CPU begins page table walk. The OS has written 0x12345xxx into CR3. The CPU assumes that this is the address of a valid page directory and sends out (0x12345000 + 0x0 * sizeof(pdir_entry_t)) onto the address bus as a uint32_t fetch. That computes to the physical address 0x12345000 being sent out on the address bus.</li>
<li>CPU gets the 4 bytes from the memory controller on the data bus, and interprets the 4 bytes it just got from our page directory index 0 as a page directory entry.</li>
<li>CPU sees that this page directory entry is 0x12344xxx, as in our example above. 'xxx' are the permission bits. The CPU checks these, and assuming the permission bits are valid, they're correct, and the entry is PRESENT.</li>
<li>CPU now, having validated the permission bits, will extract the physical address from the page directory entry. It will get 0x12344000.</li>
<li>CPU now confident that 0x12344000 is the start of a page table, decides to index into this page table by computing the offset from the base, 0x12344000. It needs to get index 0x200. It then computes: (0x12344000 + (0x200 * sizeof(ptab_entry_t)), and sends the result out onto the address bus.</li>
<li>Memory controller returns the 4 bytes at 0x12344800.</li>
<li>CPU interprets this as a page table entry, and checks the bits. Once again, the permissions bits are valid.</li>
<li>CPU is now at the leaf level, and extracts the frame address for the virtual address 0x200xxx. This turns out to be 0x72445000.</li>
<li>CPU evicts some entry from the TLB to make space for this new translation data that maps 0x200000 in the current address space to the frame 0x72445000.</li>
<li>Program execution continues. Note that translation fault number two, what people call the x86 Page Fault, never occurs since the CPU *did* find a translation from its walk.</li>
</ol>

Now that we have a firm grasp of how a page table walk works, and what the idea of "interpretation" is, let us simulate a walk of a self-referencing page directory entry.

<ol>
<li>Processor encounters an instruction that references 0xFFFFFxxx while paging is turned on. This reference must pass through the MMU. This is also our self-referencing entry, as you can see from the example page-table setup above.</li>
<li>Assume this translation was not found in the TLB. A TLB miss occurs. On x86-32, this causes the processor to walk the page tables rather than to trap into the OS for a software TLB refill.</li>
<li>The CPU takes the virtual address, 0xFFFFFxxx and splits it up, 10 bits, 10 bits, and 12 bits.</li>
<li>The CPU now knows that it must index into entry 0x3FF (1023) of the page directory pointed to by CR3, and then index into entry 0x3FF (1023) of the page table pointed to by the page directory in CR3.</li>
<li>CPU begins page table walk. The OS has written 0x12345xxx into CR3. The CPU assumes that this is the address of a valid page directory and sends out (0x12345000 + 0x3FF * sizeof(pdir_entry_t)) onto the address bus as a uint32_t fetch. That computes to the physical address 0x12345FFC being sent out on the address bus.</li>
<li>CPU gets the 4 bytes from the memory controller on the data bus, and correctly interprets the 4bytes it just got from our page directory index 1023 as a page directory entry.</li>
<li>CPU sees that this page directory entry is 0x12345xxx, as in our example above: this entry references itself! But the CPU does not check that; it simply checks the permissions, and gets ready to interpret that address in the page directory entry as a page table's physical address. 'xxx' are the permission bits. The CPU checks these, and assuming the permission bits are valid, they're correct, and the entry is PRESENT.</li>
<li>CPU now, having validated the permission bits, will extract the physical address from the page directory entry. It will get 0x12345000 as a result of our self-referencing trick, and prepares to read *AGAIN* from the page directory, instead of reading from a page table. It will *interpret* the page directory's bytes now as a page table.</li>
<li>CPU now confident that 0x12345000 is the start of a page table, decides to index into this page table by computing the offset from the base, 0x12345000. It needs to get index 0x3FF. It then computes: (0x12345000 + (0x3FF * sizeof(ptab_entry_t)), and sends the result out onto the address bus.</li>
<li>Memory controller returns the 4 bytes at 0x12345FFC.</li>
<li>CPU interprets this as a page table entry, and checks the bits. Just like last time, since we're reading the same bytes, the permissions turn out to be valid.</li>
<li>CPU is now at the leaf level (or so it thinks), and extracts the frame address for the virtual address 0xFFFFFxxx. This turns out to be 0x12345000, since entry 0x3FF (1023) in the page directory (which the CPU is currently interpreting as a page table) points to the page directory, 0x12345000.</li>
<li>CPU evicts some entry from the TLB to make space for this new translation data that maps 0xFFFFF000 in the current address space to the frame 0x12345000.</li>
<li>Program execution continues. No page fault occurs, since the hardware page-table walk found a present translation with valid permissions.</li>
<li>More importantly, the program that sent out the address, 0xFFFFF000, is about the receive the data that was at physical frame 0x12345000, since the CPU walked to that extent, and has a TLB entry that maps it as such. This program can now read and write from/to the page directory simply by accessing offsets from 0xFFFFFxxx. Accessing virtual address 0xFFFFF000 reads page directory entry 0. Accessing 0xFFFFF004 reads entry 1, and so on.</li>
</ol>

And now that we understand how the self-referencing page table trick can be used to read from and write to the current page directory, we'll examine how to read from and write to the page tables in the current process next. A note: 0xFFFFF000 is a valid address to any program unless you map it as SUPERVISOR, or in other words, you leave the USER bit unset. Otherwise programs in userspace will be able to edit their own page tables. Imagine a program deciding to start mapping pages in its address space to the kernel at physical address 0x100000? It could very well now decide to zero out the kernel in RAM.

==See Also==
* [[Paging]]
* [[Segmentation]]
=== External Links ===
*[http://en.wikipedia.org/wiki/Memory_management_unit Memory Management Unit] on Wikipedia


[[Category:Memory management]]
[[Category:Physical Memory]]
[[Category:Virtual Memory]]
````

<a id="w003011"></a>

## W003011 — User:Gravaera

Bron-ID: W003011
Namespace: User (2)
Revisie: 11746 | Datum: 2011-07-09T18:28:26Z
Bron: https://wiki.osdev.org/User:Gravaera
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera&oldid=11746

````text
Let me tell you all about me:

[http://www.youtube.com/watch?v=2Z4m4lnjxkY&feature=related This is everything you need to know about me], gravaera. Kudos, and now you can move along :>
````

<a id="w003013"></a>

## W003013 — PXE

Bron-ID: W003013
Namespace: Main (0)
Revisie: 30252 | Datum: 2026-03-16T15:53:54Z
Bron: https://wiki.osdev.org/PXE
Revisie-URL: https://wiki.osdev.org/index.php?title=PXE&oldid=30252

````text
Preboot eXecution Environment, or PXE, is the only way for [[Diskless Booting]] in PCs, and the standard is developed by Intel. It allows the Network Bootstrap Program (NBP) or the Bootloader to interface with the PXE, and load files via TFTP.

==State of the Machine==

Described by Section 4.4.5, Client State at Bootstrap Execution Time, the PXE is responsible for getting the boot code in the Client Machine. This is entirely similar to other boot devices, however, the BIOS Magic Signature, 0xAA55 isn't required, and the size of the boot file is limited to 32KiB, instead of the familiar 512 bytes.

Prior to Version 2.1 of the PXE specification, a PXENV+ Structure was given to the NBP. The PXE Entry Structure, or the !PXE structure, was introduced in Version 2.1, while the PXENV+ Structure has become obsolete. However, the address of the PXENV+ Structure is still given to the NBP during boot, to maintain compatibility with older NBPs.

The PXE Specification promises that the boot file is loaded at the address 0x0000:0x7C00, though it isn't trustable. Moreover,

* '''ES:BX'''       ES:BX should point to the PXENV+ structure, to maintain backward compatibility.
* '''SS:SP'''       SS:SP points to a valid stack, with at least 1.5KiB of free stack.
* '''SS:SP + 4'''   SS:SP + 4 points to the newer !PXE structure, if and only the PXE Version is greater than or equal to 1.5.

===PXENV+===

The structure of the PXENV+ structure is as follows:

<syntaxhighlight lang="c">
    // The Signature of the PXENV+ structure - contains "PXENV+".
    uint8_t  Signature[6];

    // The MSB contains the Major version number, while the LSB contains the minor version number.
    uint16_t Version;

    // The Length and the checksum of the structure, used for calculating the checksum.
    uint8_t  Length;
    uint8_t  Checksum;

    // A Far Pointer to the real mode PXE API entry point. 
    uint32_t RMEntry;

    // 32-bit offset to the protected mode API entry point. Instead of this, the !PXE structure is recommended.
    uint32_t PMOffset;

    // The rest of the fields don't matter much, and the Specifications can be referred.
    uint16_t PMSelector;
    uint16_t StackSeg;
    uint16_t StackSize;
    uint16_t BCCodeSeg;
    uint16_t BCCodeSize;
    uint16_t BCDataSeg;
    uint16_t BCDataSize;

    uint16_t UNDIDataSeg;
    uint16_t UNDIDataSize;
    uint16_t UNDICodeSeg;
    uint16_t UNDICodeSize;

    // This is a far pointer to the "!PXE" structure, only present when the structure is present.
    uint32_t PXEPtr;
</syntaxhighlight>

==== Recommended Usage ====

First of all, the integrity of the PXENV+ structure should be checked. This can be done by:

* Checking the PXENV+ signature at the starting.
* And, by checking the checksum of the structure.

If the structure fails to pass either of the above, boot should be aborted.

If the structure is good, the Version number should be checked. If it is higher than or equal to 0x0201 then the !PXE structure should be used. On the other hand, if it isn't, the PXENV+ structure should be used.

===!PXE ===

The structure of the !PXE structure is as follows:

<syntaxhighlight lang="c">
    // The Signature of the !PXE structure - contains "!PXE".
    uint8_t  Signature[4];

    // The Length and the checksum of the structure, used for calculating the checksum.
    uint8_t  Length;
    uint8_t  Checksum;
   
    // Contains zero - both of them.
    uint8_t Revision;
    uint8_t Reserved;

    // Far pointer to UNDI ROM ID structure and BC ROM ID structure.
    uint32_t UNDIROMID;
    uint32_t BCROMID;

    // A Far Pointer to the real mode PXE API entry point. 
    uint32_t RMEntry;

    // A Far Pointer to the protected mode API entry point. 
    uint32_t PMEntry;

    // The rest of the fields don't matter much (in the rest of the article), and the Specifications can be referred.
</syntaxhighlight>

==== Recommended Usage ====

If the PXENV+ lists the revision as greater than or equal to 0x0201, then only must the !PXE structure be relied upon. Then, the integrity of the !PXE structure should be checked. This can be done by:

* Checking the !PXE signature at the starting.
* And, by checking the checksum of the structure.

If the structure fails to pass either of the above, the PXENV+ structure should be used.

==PXE API ==

The PXE Entry Point uses the __cdecl calling convention - push the parameters on the stack (rightmost pushed first, leftmost pushed last), far call the entry point, clean up the stack - while the older PXENV+ Entry Point uses registers.

To access the API, a generic CallAPI function can be created, which eases out the calling. To do this, first of all, the address of the Entry Point, from either of the structures should be stored in a location (PXEAPI in the example). The PXE API accept the same parameters:

* Opcode, or the API function number.
* Segment:Offset address to a structure containing input data.

A example generic call function is as follows:

<syntaxhighlight lang="nasm">
; Calls the PXE API, and abstracts things such that it works on both old and new APIs.
; @ds:di          The address of the input buffer.
; @bx             The opcode.
; NOTE: The above registers are what the legacy API also used, so it should cause no problem with it.
UsePXEAPI:
    ; And we push it over here, so that we have no problems with the new API.
    push ds
    push di
    push bx
    
    call far [PXEAPI]
    
    add sp, 6                         ; Clean up the stack.
    ret
</syntaxhighlight>

The PXE API returns a status flag in AX and the beginning of the input buffer passed. It is recommended to test both of these, where a non zero value indicates failure.

==Files, via TFTP ==

Getting the files via TFTP require the TFTP server's IP address. Along with that, you might want the client machine's IP address, the DHCP server's IP address, the client's MAC address, and perhaps several other things.

While this might seem daunting at first, PXE defines a neat way to do this.

===Getting "Cached Information"===

The PXE API's Get Cached Information function, with opcode 0x0071, requires a "t_PXENV_CACHED_INFO" passed to it, and returns a "bootph" structure. This "bootph" structure contains important information about the client and server machine, which is what is exactly wanted.

The "t_PXENV_CACHED_INFO" follows the following structure:

<syntaxhighlight lang="nasm">
; Note - due to comfort reasons, the author has decided to provide the following snippet
; in Intel (NASM) Assembly. If someone wishes to change it to a more generic format, he is welcome.
t_PXENV_GET_CACHED:
    .Status       dw 0
    .PacketType   dw 2
    .BufferSize   dw 0
       .BufferOff dw 0 
       .BufferSeg dw 0
    .BufferLimit  dw 0
</syntaxhighlight>

* '''.Status'''. This should be zero, and returns the status after the function call.

* '''.PacketType'''. This is the type of the packet, and 2 means "PXENV_PACKET_TYPE_DHCP_ACK", which is what we require. For further information, read the specifications.

* '''.BufferSize'''. The number of bytes copied into the segment and offset field given.

* '''.BufferOff/Seg'''. This gives the output buffer, to which to copy the bootph structure to. If you want, keep this to zero, and PXE would return the address of the Bootph structure in the BC data segment.

To do a call, put the address of the above structure in ES:DI, put the opcode, 0x0071, in BX, and call the generic call function. A example call to the above function is, as follows:

<syntaxhighlight lang="nasm">
    mov di, t_PXENV_GET_CACHED
    mov bx, GET_CACHED_INFO           ; 0x0071.
    
    call UsePXEAPI                    ; Ask for the cached info.
    
    or ax, [t_PXENV_GET_CACHED]       ; Get the status into AX.
    test ax, ax
    jnz .Error
</syntaxhighlight>

Bootph has a little complex structure, and the Specification should be referenced to know more about it. For now, the following is what you may require:

* uint8_t CIP[4].  The Client IP address, at offset 12.
* uint8_t YIP[4].  "Your" IP address, at offset 16.
* uint8_t SIP[4].  Server IP address, at offset 20.
* uint8_t GIP[4].  Relay agent IP address, at offset 24.
* uint8_t MAC[16]. The MAC IP address, present at offset 28.
* uint8_t SER[64]. The server host name, a asciiz string, at offset 40.
* uint8_t BOO[128]. The Boot File name, a asciiz string, at offset 104.

===Opening===

PXE offers "TFTP Open", "TFTP Read" and "TFTP Close", which could be used like other common file I/O operations. It also offers a "TFTP Read File" - but it is recommended to use the former, which is more ''configurable''.

PXENV_TFTP_OPEN, with opcode 0x0020 is required to open a file via TFTP. It takes a t_PXENV_TFTP_OPEN structure as a parameter, which has the following format:

<syntaxhighlight lang="nasm">
t_PXENV_TFTP_OPEN:
    .Status       dw 0
    .SIP          dd 0
    .GIP          dd 0
    .Filename     times 128 db 0
    .Port         dw 0
    .PacketSize   dw 0
</syntaxhighlight>

* '''.Status'''. The status word.

* '''.SIP'''. The Server IP, which we just obtained.

* '''.Filename'''. A null terminated string, of the name of the file to open.

* '''.Port'''. This should contain 69, or the UDP Port.
 
* '''.PacketSize'''.  This contains the size of each packet read. Sizes of 512, 1024 and something small are recommended. Usually, bigger sizes load the file faster, but aren't supported much. See the section on performance below for more information relating to packet size.

NOTE: The SIP, GIP (0 means default) and Port are all big endian.

===Reading===

The PXENV_TFTP_READ function, with opcode 0x0022, reads a single packet from the currently opened file. The size of the packet is set in the Open call, while the function requires a t_PXENV_TFTP_READ structure.

The t_PXENV_TFTP_READ has the following structure:

<syntaxhighlight lang="nasm">
t_PXENV_TFTP_READ:
    .Status       dw 0
    .PacketNumber dw 0
    .BufferSize   dw 0
    .BufferOff    dw 0
    .BufferSeg    dw 0
</syntaxhighlight>

* '''.Status'''. This contains the status returned after the function call.

* '''.PacketNumber'''. This contains the packet number sent by the TFTP server. This is used to read packets in a order. Think of it like a index into the packets of the file - used to track of the current index.

* '''.BufferSize'''. The buffer size - number of bytes written in the packet buffer. It is the last packet if this is less than the size of the packet negotiated.

* '''.BufferOff/Seg'''. The address where you want to read the file packet.

What you want to do here is to keep reading packets from the file by increasing the '.BufferOff/Seg' address, and checking if the size of the packet read is less than the size negotiated in TFTP_OPEN. It can be concluded that several implementations use 0 to indicate EOF, while other implementations give a 'packet read' size less than the size negotiated.

With the above point in mind, the following pseudo code can be used to read a file:

<syntaxhighlight lang="c">
while (bytes_left > 0)
{
    call_api();

    // AX register contains whether it failed or not, while Status contains the detailed status.
    if (status || asm("ax"))                  
        panic();

    if (bytes_read < PACKET_SIZE)
        break;

    bytes_left -= PACKET_SIZE;
}
</syntaxhighlight>

===Closing===

The PXENV_TFTP_CLOSE, with opcode 0x0021, is perhaps the simplest function. It requires a input buffer, with the following format, to be passed to it:

<syntaxhighlight lang="nasm">
t_PXENV_TFTP_CLOSE:
    .Status       dw 0
</syntaxhighlight>

* '''.Status'''. The Status returned after calling the function. The function would most probably ONLY fail if there is no active TFTP connection. 

===Performance===

The original TFTP protocol is a very simple protocol (mostly because it was designed to be implemented easily on tiny embedded systems). The consequence is that it's relative slow (the "stop and wait" nature of it means that the network connection's bandwidth is significantly under-utilised). For larger files this can noticeably effect boot times.

There are 3 approaches for improving performance.

====TFTP Block Size====

In 1998 a "TFTP Blocksize option" was introduced. The basic idea is to improve performance by increasing the block/packet size (and reducing the number of data packets and the number of acknowledgement packets). This is supported by the PXE specification.

In theory when you open a file via "TFTP Open" you're supposed to set the "PacketSize" field to what you're requesting, and (where possible) PXE is supposed to negotiate an appropriate packet size with the TFTP server and then set the  "PacketSize" field to size of the packet that both the PXE code in the client and the TFTP server agreed upon. This means that (e.g.) you should be able to set "PacketSize" to 65535 bytes and PXE will change it to the maximum size that is supported.

In practice this only works on some/most network cards, and some networking cards are broken and don't work correctly. For example, the implementation of PXE that comes with various RealTek cards will simply return an "Unknown error" if the requested packet size is too large (even though the PXE specification says otherwise, and even though PXE defines more appropriate status codes).

It is possible to work around this. The recommended work-around is:
* If the "Version" field of the PXENV+ structure is less than "version 2.1"; be cautious and use a 512 byte packet size.
* Use the "UNDI GET INFORMATION" function to get information about the network adapter. The returned information includes an MTU (Max. Transmission Unit) field. If this fails be cautious and use a 512 byte packet size.
* Check if the MTU field is sane (within a reasonable range). If it's not within a reasonable range be cautious and use a 512 byte packet size.
* Subtract the size of an IPv4 header (20 bytes), the size of the UDP header (8 byte), the size of the TFTP header (4 bytes) and 16 more bytes just in case from the MTU. This gives you the maximum packet size you can safely use.

====TFTP "Sliding Window"====

In 2015 a "TFTP Window Size" option was introduced. The basic idea here is to use "sliding window" (like TCP) instead of "stop and wait" to increase network bandwidth utilisation. Unfortunately, at this time I doubt any network cards support this.

====Advanced/Non-standard Approaches====

The PXE API exposes lower level functionality, including the ability to send arbitrary packets and to intercept received packets (via. a callback/"ISR"). In theory it would be possible for a boot loader to use these to implement any protocol it likes (e.g. use FTP or HTTP instead of TFTP) as long as the server is setup to support the protocol (in addition to TFTP that would still be needed to download the boot loader).

The most extreme approach would be to design your own protocol and implement your own server. In this case (in theory) you would be able to maximize network bandwidth utilisation, and the client could also offload processing to the server. With a massive amount of over-engineering you could significantly reduce boot times this way.

==Cleaning up ==

After you have finished reading files via PXE, the best idea would be to shutdown PXE and reclaim any memory it was using. PXE support is unloaded by using the "PXEAPI_UNDI_SHUTDOWN" function to restore the network adapter to it's default state, followed by the "PXENV_UNLOAD_STACK" function to unhook any IRQ handlers. 

Lastly, PXE needs to prepare itself to be removed from memory (unhook interrupt 0x1A) using the "PXENV_STOP_UNDI" function (for PXE) or the "PXENV_UNDI_CLEANUP" function (for PXENV+).

Once PXE is unloaded, memory can be reclaimed by checking the UNDI code segment and UNDI data segment start addresses and sizes (in the PXENV+ data structure), calculating "start address + size" and selecting the highest value. This value is the "new" number of bytes of RAM starting at 0x00000000 (or the amount of conventional memory), and can be used directly and/or the value at 0x40:0x13 (or 0x00000413) can be changed to reflect the new number of 1 KB blocks of conventional memory. 

Please note that it's probably a good idea to calculate the amount of conventional memory before you unload PXE, because some of the functions above may clear the segment addresses and sizes in the PXENV+ structure.


==See Also ==

===Articles ===

* [[Diskless Booting]]

===External Links ===

*[http://download.intel.com/design/archives/wfm/downloads/pxespec.pdf PXE Specification]

[[Category:Booting]]
[[Category:Bootloaders]]
[[Category:Network Protocols]]
[[de:PXE]]
````

<a id="w003015"></a>

## W003015 — Talk:A20

Bron-ID: W003015
Namespace: Talk (1)
Revisie: 11769 | Datum: 2011-07-11T12:27:08Z
Bron: https://wiki.osdev.org/Talk:A20
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AA20&oldid=11769

````text
Hello,

I by mistake deleted the wiki page http://wiki.osdev.org/A20 while I wanted to delete http://wiki.osdev.org/User:Shikhin/Trial

I am extremely sorry and am feeling ashamed. However, whenever I try to restore the page, it says the operation is only permitted for sysop users. So I thought maybe you are a sysop user. Please help. And I am extremely ashamed.

Regards,
--[[User:Shikhin|Shikhin]] 10:08, 11 July 2011 (UTC)

Fixed! Page redirects are confusing.. --[[User:Brendan|Brendan]] 10:44, 11 July 2011 (UTC)
````

<a id="w003016"></a>

## W003016 — A20

Bron-ID: W003016
Namespace: Main (0)
Revisie: 11766 | Datum: 2011-07-10T17:16:45Z
Bron: https://wiki.osdev.org/A20
Revisie-URL: https://wiki.osdev.org/index.php?title=A20&oldid=11766
Doorverwijzing: A20 Line

````text
#REDIRECT [[A20 Line]]
#REDIRECT [[PXE]]
````

<a id="w003018"></a>

## W003018 — C++ Exception Support

Bron-ID: W003018
Namespace: Main (0)
Revisie: 27230 | Datum: 2022-07-12T19:32:04Z
Bron: https://wiki.osdev.org/C%2B%2B_Exception_Support
Revisie-URL: https://wiki.osdev.org/index.php?title=C%2B%2B_Exception_Support&oldid=27230

````text
If the g++ that targets your operating system follows the [https://itanium-cxx-abi.github.io/cxx-abi/abi-eh.html Itanium C++ ABI] (GCC does so for almost all architectures), you can follow this article and add C++ exception support to your kernel.

==Introduction==
In the ABI, C++ exception is supported by the cooperation of three layers. The first layer is the compiler. The compiler translates the "try" "catch" "throw" statements into calls to specific functions in C++ runtime. The second layer is the C++ runtime. For exception support this layer is more or less a wrapper around the third layer. All the dirty work are done by the third layer: the unwind library. BTW, the second layer also adds support for RTTI & dynamic_cast

So, in order to add C++ exception support, you just need to port the second and the third layer into you kernel, which means you need to either port libsupc++ or port libcxxrt and an unwind library. I prefer the second option, because libsupc++ is part of GCC, it's even not trivial to compile it separately. For the unwind library part, you have two options, libunwind and libgcc_eh. libunwind is more difficult to port, it depends on pthread and some type definitions in <elf.h> <link.h> and <ucontext.h>. The major reason you may want libunwind instead of libgcc_eh is that libgcc_eh is licensed under LGPL.

If you don't have any of the unix header files when compiling libcxxrt or libgcc_eh, e.g. <unistd.h>, create an empty one and try again. If you still can't compile libcxxrt or libgcc_eh, find a copy of the corresponding header file from a unix system and find out exactly what you are missing.

==Port libcxxrt==
libcxxrt can be downloaded [https://github.com/libcxxrt/libcxxrt here]. libcxxrt is very easy to port, only 1 .c file and 8 .cc files. It depends on a few simple libc functions and a few pthread functions. If you don't need multi-thread support in your kernel (no smp, kernel code isn't preemptive), create some pthread stubs. Then copy all the source files and header files into your own kernel source tree and compile them with your existing build system, no special compiler flags are needed.

==Port libgcc_eh==
libgcc_eh is more difficult. 
# First thing is to try to cross compile GCC (you will fail, don't worry).
# In the GCC build tree, you can find a directory named the same as your GCC target, e.g. i686-pc-linux-gnu.
# Change to the sub-directory libgcc under the directory mentioned above.
# Type command "make -k libgcc_eh.a" and record the console output of this command.
# Check the console output and find out the exactly source files and compiler flags needed for libgcc_eh
# Copy the source files into your kernel source tree except emutls.c(it's unnecessary), and add necessary compiler flags recorded in last step
# Try to compile these source files, if any header file is missed, search GCC source tree and build tree for it.
There is a generated file "gthr-default.h", most of the time it contains only one line: <pre>#include "gthr-posix.h"</pre>. If you don't need multi-thread support, change it to <pre>#include "gthr-single.h"</pre>

Note that you should *NOT* rename any source files from .c to .cpp. You need to compile these files as C-files, while (most of) the rest of your kernel is C++

Some of the compiler flags that affect code generation may cause stack unwinding to fail. In particular, flags like "-fomit-stack-pointer" and "-mregparms={n}" could break things. When debugging the ported code, try compiling without these flags first.

==Rolling your own==
(NOTE: This is not yet complete, and hasn't been independently tested)
If you're adverse to including external code in your OS (or just want the challenge), you should be able to write your own C++ library, which handles the dispatching and allocation of exceptions (stack backtracing is handed by libgcc, which you should already be including with your build). If you choose this route, make sure to run _init upon loading each module (library or executable) to properly register the exception handling frames with the unwind code.

The functions you'll need to implement all start with `__cxa_` and are documented in the Itanium C++ ABI (which is also used for x86 by GCC)
You will also need to implement `__gxx_personality_v0` (which according to the clang docs, is the de-facto standard for C++), the structures used by this function to know what to do are documented in [https://github.com/itanium-cxx-abi/cxx-abi/raw/master/exceptions.pdf this] PDF.

It is also possible to create macros that implement exception-like control structures (especially useful if you use visual C++). You can implement exceptions with nothing more than setjmp and a linked list. However, you will need to create an exception class that all exceptions derive from, which implements some form of basic RTTI.

You can also implement the entire unwind library that matches the IA64 C++ Spec. Libc++ and Libc++abi provide great starting points for Layers 1 and 2. All thats needed is Layer 3 which provides the __Unwind_RaiseException logic which uses the "eh_frame" specs from LSB, DWARF4 spec, IA64 C++ spec and the System V spec. To see an example of how to do this, see [https://github.com/Bareflank/hypervisor/tree/master/bfunwind here]

==Function stubs==
If you don't have some of the needed funtions, try some of these stubs

===malloc===
<source lang="cpp">
void* malloc(size_t size) {
	static char* freeMemoryBase = SOME_BASE_ADDRESS;
	size = (size + 7) / 8 * 8;
	freeMemoryBase += size;
	return freeMemoryBase - size;
}
</source>

===pthread===
You may use the following stub for pthread, or you can just define LIBCXXRT_WEAK_LOCKS for libcxxrt, which will make libcxxrt treat all pthread functions as weak symbol, so you do not actually need to provide these functions, but only a single pthread.h with only declarations.
<source lang="cpp">
namespace { void* threadDataTable[64]; int freeEntry = 0;}
int pthread_key_create(pthread_key_t* key, void (*)(void*)) {
	assert(freeEntry < 64);

	*key = freeEntry;
	freeEntry++;
	return 0;
}

int pthread_once(pthread_once_t* control, void (*init)(void)) {
	if (*control == 0) {
		(*init)();
		*control = 1;
	}
	return 0;
}

void* pthread_getspecific(pthread_key_t key) {
	return threadDataTable[key];
}

int pthread_setspecific(pthread_key_t key, const void* data) {
	threadDataTable[key] = (void*)data;
	return 0;
}

int pthread_mutex_init(pthread_mutex_t* mutex, const pthread_mutexattr_t*) {
	*mutex = 0;
	return 0;
}

int pthread_mutex_lock(pthread_mutex_t* mutex) {
	assert(*mutex == 0);
	*mutex = 1;
	return 0;
}

int pthread_mutex_unlock(pthread_mutex_t* mutex) {
	assert(*mutex != 0);
	*mutex = 0;
	return 0;
}

int pthread_cond_wait(pthread_cond_t*, pthread_mutex_t*) {
	return 0;
}

int pthread_cond_signal(pthread_cond_t*) {
	return 0;
}
</source>

===dladdr===
This function is used for debugging by libcxxrt, not a big problem to simply return an error.
<source lang="c">
int dladdr(void*, Dl_info*) {
	return 0;
}
</source>

==Link Script==
If you use a custom link script, make sure to place .eh_frame_hdr, .eh_frame and .gcc_except_table in the exact order in your final output. Otherwise sometimes you are not able to catch any exceptions.

== See Also ==

* https://itanium-cxx-abi.github.io/cxx-abi/abi-eh.html (While sounding like being specific to the Itanium, this is the base for the common C++ ABI)

* http://www.codeproject.com/cpp/exceptionhandler.asp (Specific to VC++.)

* http://www.microsoft.com/msj/0197/Exception/Exception.aspx (Article on SEH, the [[Stack#Unwinding the stack|stack-based unwinding]] used by VC++ and most other Windows compilers. Any use of stack-based SEH may or may not be covered by USPTO patent #5,628,016, held by Borland International, Inc.; GCC and most other UNIX compilers use the same table-based mechanism on the x86 that is the rule on RISC architectures, thus being unaffected by the patent.)

* https://web.archive.org/web/20120620090841/http://www.kuzbass.ru:8086/docs/isocpp/except.html

* The standard header '''<exception>''', declaring several support functions.

* http://llvm.org/docs/ExceptionHandling.html (How LLVM/Clang handles C++ exceptions, references the C++ABI and the following document)
* https://github.com/itanium-cxx-abi/cxx-abi/raw/master/exceptions.pdf (A document by HP on C++ exception handling)

[[Category:C++]]
````

<a id="w003019"></a>

## W003019 — Talk:C++ Exception Support

Bron-ID: W003019
Namespace: Talk (1)
Revisie: 11786 | Datum: 2011-07-12T09:37:56Z
Bron: https://wiki.osdev.org/Talk:C%2B%2B_Exception_Support
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AC%2B%2B_Exception_Support&oldid=11786

````text
Perhaps try the "Show preview" button. ;-) -- [[User:Solar|Solar]] 09:37, 12 July 2011 (UTC)
````

<a id="w003021"></a>

## W003021 — User talk:Zhongyijun

Bron-ID: W003021
Namespace: User talk (3)
Revisie: 11835 | Datum: 2011-07-23T09:10:47Z
Bron: https://wiki.osdev.org/User_talk:Zhongyijun
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AZhongyijun&oldid=11835

````text
Dear editor,

I noticed you have been recently editing the wiki, while you haven't agreed with [http://forum.osdev.org/viewtopic.php?f=1&t=23838 the updated license]. While the new edits automatically fall under the new license, I'd like you to post a reply so that we also confirm your older edits as safe.

Thanks in advance, [[User:Combuster|Combuster]] 09:10, 23 July 2011 (UTC)
````

<a id="w003022"></a>

## W003022 — User talk:Mariuszp

Bron-ID: W003022
Namespace: User talk (3)
Revisie: 11836 | Datum: 2011-07-23T09:11:53Z
Bron: https://wiki.osdev.org/User_talk:Mariuszp
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AMariuszp&oldid=11836

````text
Dear editor,

I noticed you have been recently editing the wiki, while you haven't agreed with [http://forum.osdev.org/viewtopic.php?f=1&t=23838 the updated license]. While the new edits automatically fall under the new license, I'd like you to post a reply so that we also confirm your older edits as safe.

Thanks in advance, [[User:Combuster|Combuster]] 09:11, 23 July 2011 (UTC)
````

<a id="w003023"></a>

## W003023 — Sparc Barebones

Bron-ID: W003023
Namespace: Main (0)
Revisie: 30329 | Datum: 2026-03-17T20:51:29Z
Bron: https://wiki.osdev.org/Sparc_Barebones
Revisie-URL: https://wiki.osdev.org/index.php?title=Sparc_Barebones&oldid=30329

````text
{{TutorialExplain}}
{{BeginnersWarning}}
{{Rating|2}}
Sparc comes with a few oddities of its own, including a non-trivial bootsector implementation. The following code will provide you a running start on SPARC, but it is one big hack, it has not been written with extension in mind, and neither has it been tested on real hardware.

== Tools ==
You will need:
* sparc-elf-as and sparc-elf-ld, just make a [[GCC Cross-Compiler|cross-toolchain]] with the sparc-elf as target.
* dd
* mkisofs
* qemu-system-sparc (get a recent version!)
[[GCC]] might come in handy later, but it is not used in this example

== Bootsector ==
OpenFirmware's boot sequence will try to load an application from either a generic partition or a platform-specific partition. That table is located in the first 512 bytes of the storage medium. Within that partition, it will try to look for a binary located from byte 512 to 8191. [[QEMU]] will allow boot from a.out, ELF or a linux image, whereas Sun's implementation on actual hardware is said to only support a.out, so we will have to stick to that. The binary is parsed and loaded at some location in memory, so it also needs to be a position-independent implementation. After loading the file, it will jump to the start of the text section, and pass a pointer to the openfirmware client structure as the first argument (o0 register).

For demonstration purposes, we grab our load address, calculate a pointer offset and then issue a print call to the firmware.

<syntaxhighlight lang="asm">
.section .text
.align 16
.global _start

_start:
        call 1f                      ! put PC on the stack.
          mov %o7, %l0               ! use the delay slot to grab it
1:      mov 0x60, %l3                ! 0x60 is the offset to the putstr pointer
        add %l3, %o0, %l3            ! actual address holding the putstr pointer
        ld [%l3], %l5                ! address of function

        mov dataptr - _start, %l4    ! offset to text to print
        add %l4, %l0, %l4            ! absolute address of text to print

        mov %l4, %o0                 ! first argument is pointer
        mov 0x0b, %o1                ! second argument is number of characters
        call %l5                     ! call function
          nop                        ! waste the delay slot for simplicity
        
2:      
        call 2b                      ! loop forever
          nop
 

dataptr:
        .byte 'H', 'e', 'l', 'l', 'o', ' ', 'W', 'o', 'r', 'l', 'd', 0
</syntaxhighlight>

There's two problems now: binutils wants to make a demand-loaded binary by default, which is something qemu's default bios really does not like. Which means we have to work around that by setting the header type manually. Since a.out is such a dumb format, we only need 8 lines of code to have ld figure out the rest. We first tell it to kick out all headers, add the a.out signature we need, then the size of .text (no location needed, it comes directly after the header), followed by the size of the headers we are not using:
<pre>OUTPUT_FORMAT(binary)
ENTRY(_start)
 
SECTIONS
{
    . = 0x3980;
    .main : 
    {
        LONG(0x01030107);
	LONG(_boot_end - _boot_start);
	LONG(0);
	LONG(0);
	LONG(0);
	LONG(0);
	LONG(0);
        LONG(0);
	_boot_start = .;
	*(.text*)
	*(.rodata*)
	*(.data*)
	*(.bss*)
	_boot_end = .;
    }
}
</pre>

== Building ==

The bootsector can then be made with the following commands:

<syntaxhighlight lang="bash">
sparc-elf-as boot.S -o boot.o
sparc-elf-ld boot.o -o boot.bin -T sparcboot.ld
</syntaxhighlight>

SILO remarks that the firmware only really checks the partition offsets, and throws in a sector of zeroes so that it will always look at the start of the disk for the bootblock. This is a cheap way to boot off any custom medium if you don't care about any filesystem, but it is especially useful for the CD filesystem which don't put anything near that area and just filling it up with zeroes is correct as far as the standard goes:

<syntaxhighlight lang="bash">
dd if=/dev/zero of=bootblock.bin bs=2048 count=4
dd if=boot.bin of=bootblock.bin bs=512 seek=1 conv=notrunc
mkisofs -o helloworld.iso -G bootblock.bin /path/to/other/cd/contents
</syntaxhighlight>

Start QEMU afterwards:

<syntaxhighlight lang="bash">
qemu-system-sparc -cdrom helloworld.iso
</syntaxhighlight>

Which gives you the firmware prompt, after which entering

 boot cdrom

will start the code.

== Source Code ==

There is source code and a Makefile available on github at [https://github.com/teverett/sparc_barebones https://github.com/teverett/sparc_barebones]

[[Category:Sparc]]
[[Category:Bare bones tutorials]]
````

<a id="w003024"></a>

## W003024 — Talk:El-Torito

Bron-ID: W003024
Namespace: Talk (1)
Revisie: 27085 | Datum: 2022-05-13T02:46:07Z
Bron: https://wiki.osdev.org/Talk:El-Torito
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AEl-Torito&oldid=27085

````text
== 2k limitation somewhere? ==

Could someone please explain where the "2k" comes from under "What's Next?" in the sentence:
"you can start reading the file system, in the 2k that you have available in your boot sector."

Is the El-Torito standard limited to loading boot sectors no larger than 2KiB? --[[User:Messiahandrw|MessiahAndrw]] 10:48, 27 July 2011 (UTC)

:If you follow the link to [[Bootable El-Torito CD with GRUB Legacy]], you will find the `genisoimage` option `-boot-load-size 4` (also used in this article) explained as follows: "''Specifies the number of 512-bytes sectors to load. Four 512-byte sectors (2048 bytes) is one CD sector and is the number supported by most BIOS.''" (Had to look it up myself, but there. ;-) ) -- [[User:Solar|Solar]] 12:13, 27 July 2011 (UTC)

I discovered this through experimentation (I'm trying to boot directly into my kernel using El-Torito rather than via a bootloader). I'm editing the article now to explain this. --[[User:Messiahandrw|MessiahAndrw]] 14:20, 27 July 2011 (UTC)

2K sectors come from the ISO9660 specification - see [http://en.wikipedia.org/wiki/Iso9660#Specifications http://en.wikipedia.org/wiki/Iso9660#Specifications].

The basic of storage is a Frame of 24 bytes, and a Sector is defined as 98 Frames.  This gives 2352 usable bytes per Sector.  Because CDs store their data in 1 single spiral, it is impossible to calculate the exact start of every single sector given the technology at the time (It is better now).  As a result, all Sectors must have alignment data so the CD reader can precicely locate the start of the sector given a rough approximation from a calculation.  In addition, CDs damage very easily, so they have always had error correcting data written into the sector so the sector can still be recovered correctly if damaged a little.  As a result, ISO9660 splits the 2352 usable bytes per sector into 12 bytes of alignment/synchronisation data, followed by 2048 bytes of regular data, followed by 288 bytes of error detecting and correcting codes.  [[User:Andyhhp|Andyhhp]] 00:20, 29 July 2011 (UTC)

== Proposals for enhancements ==

I am the developer of xorriso, which produces bootable
ISO 9660 images. Here are some proposals after reading
this wiki page.
I am ready to implement them here. But first i ask
whether such changes would be appreciated.

------------------------

The examples about floppy emulation and Legacy GRUB are
quite antique, meanwhile.

GRUB2 script grub-mkrescue should be mentioned:
http://www.gnu.org/software/grub/manual/html_node/Making-a-GRUB-bootable-CD_002dROM.html#Making-a-GRUB-bootable-CD_002dROM

Bootloader ISOLINUX should be mentioned:
http://www.syslinux.org/wiki/index.php/ISOLINUX
Probably
http://wiki.osdev.org/SysLinux
should get a sibling for the ISOLINUX aspects of SYSLINUX.

------------------------

"Writing an El-Torito Boot Sector"

This headline does not match the object which is described.
"El-Torito Boot Sector" would normally be the El-Torito
Boot Record, which is a ECMA-119 Volume Descriptor.
The description rather applies to the boot image,
which is the blob pointed to by an entry of the
El-Torito Boot Catalog.
With BIOS, that blob is the first stage of a boot loader.
With EFI, the blob is a FAT filesystem image (but not an
emulated floppy).

The structure of El-Torito Volume Descriptor, Boot Catalog,
and boot images should be described, in order to clarify
the possibilities of multiple boot images for e.g. BIOS
and EFI.
I.e. Boot Record points to Boot Catalog which lists the
available boot images.

Further one should mention that ISO images can carry
MBR and GPT for booting the image from hard-disk-like
devices (e.g. USB sticks).

Example of combined BIOS and EFI via El-Torito,
MBR, and GPT as was done for debian-7.1.0-amd64-netinst.iso:

  xorriso -as mkisofs
  ...
  -c isolinux/boot.cat
  -b isolinux/isolinux.bin
  -no-emul-boot -boot-load-size 4 -boot-info-table
  -isohybrid-mbr syslinux/usr/lib/syslinux/isohdpfx.bin
  -eltorito-alt-boot
  -e boot/grub/efi.img
  -no-emul-boot -isohybrid-gpt-basdat
  ...
  boot1
  ...

Option -eltorito-alt-boot separates the settings
for the BIOS boot image from those for the EFI
boot image.
The files isolinux/isolinux.bin and boot/grub/efi.img
get into the ISO image from source directory ./boot1.
File syslinux/usr/lib/syslinux/isohdpfx.bin is copied
directly from the SYSLINUX installation into the ISO
image. It contains the MBR template.
Option -isohybrid-gpt-basdat announces the FAT image
boot/grub/efi.img as GPT partition.

See http://www.gnu.org/software/xorriso/man_1_xorrisofs.html

Description on byte level:
http://bazaar.launchpad.net/~libburnia-team/libisofs/scdbackup/view/head:/doc/boot_sectors.txt

--------

The boot information table does not belong to El Torito.
It is rather part of boot loader specifications,
possibly invented for ISOLINUX.

--------

  -boot-info-table-o [IsoFile.iso]

must rather be

  -boot-info-table [-o IsoFile.iso]

or more safely for the terminal

  -boot-info-table -o IsoFile.iso

--------

Maybe one should explain the relations of mkisofs, genisoimage,
other clones, and the mkisofs emulation of xorriso.
I would volunteer to try this as unbiasedly as possible for
one of the competitors.

[[User:Scdbackup|Scdbackup]] 02:36, 15 October 2013 (CDT)

: After trying to apply smaller changes i decided to propose a re-write of large
: parts of the text. See my user page [[User:Scdbackup]].
: If no objections arise within a few days, i will put it on the article page.
:
: [[User:Scdbackup|Scdbackup]] 13:03, 18 October 2013 (CDT)

:: The re-written parts are now in the article. Review would be appreciated.
:: [[User:Scdbackup|Scdbackup]] 05:51, 20 October 2013 (CDT)

== BOOTBOOT Spam ==

What? Why? --[[User:Wxwisiasdf|Wxwisiasdf]] 21:46, 12 May 2022 (CDT)
````

<a id="w003025"></a>

## W003025 — Talk:UEFI

Bron-ID: W003025
Namespace: Talk (1)
Revisie: 26410 | Datum: 2021-09-20T13:41:15Z
Bron: https://wiki.osdev.org/Talk:UEFI
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AUEFI&oldid=26410

````text
==BROKEN: Creating disk images, linux, root not required==

I followed the instructions to create a disk image without root, and discovered that the mcopy commands are broken.  mcopy (at least on Ubuntu 16.04.1) doesn't have a -i option.  I also checked and mformat doesn't have a -i option, so I'm not sure why that step didn't fail.  I have no idea how to fix this, so I'm leaving a comment here. [[User:Sdp|Sdp]] 10:38, 23 December 2016 (CST)

I figured it out.  -i wasn't the issue, that's a common option on all mtools (though, I would say it's very poorly documented).  The issue was that mcopy requires a destination that the instructions omitted.  The correct form to copy main.efi to the disk is: "mcopy -i temp.img main.efi ::", in which the :: is the destination and indicates that the file should be copied to the disk image temp.img. [[User:Sdp|Sdp]] 10:58, 23 December 2016 (CST)

For QEMU in particular, some mention of the "-hda fat:rw:dirname" option seems worthwhile.  It saves a *ton* of trouble when debugging/testing.  --[[User:Johnwilson|Johnwilson]] 00:16, 5 December 2017 (CST)

==Rewriting some content==
All the GDB debugging stuff for UEFI/OVMF/QEMU has been moved to its own page: [[Debugging UEFI applications with GDB]]. It was too bulky and specific to include here. [[User:Intx13|Intx13]] 09:20, 24 November 2015 (CST)

I'm starting to remove the older content, particularly the irrelevant stuff about there being no multibooting menus and the "biased mode" section. The useful content from those sections that wasn't already incorporated elsewhere has been included. [[User:Intx13|Intx13]] 08:52, 20 November 2015 (CST)

I'm rewriting some content in this article, hopefully clarifying the example and providing more detail for new UEFI developers. Expect there to be some redundancy as I work on it over the next few days. -[[User:Intx13|Intx13]] 15:03, 10 November 2015 (CST)

: I saw a lot of instances of "UEFI BIOS" usage. But is it technically correct? I have read that that specific usage is confusing. I believe most of them can simply be replaced with "UEFI system" (especially "UEFI BIOSs" with "UEFI systems"). You won't find such usage in either Arch wiki or Wikipedia (as of now -- used to be there). [[User:Critiqjo|Critiqjo]] 02:18, 15 November 2015 (CST)

:: I think it's technically correct ("the best kind of correct") in the sense that it refers to a BIOS ROM that adheres to the UEFI specification for platform initialization and application loading. "UEFI system" makes sense in the context of the UEFI specification, where the BIOS ''is'' the system being specified, but here, it would sound like that refers to a UEFI-compatible PC. How about "UEFI firmware"? Then we could draw a distinction between "legacy firmware" and "UEFI firmware", both of which are BIOS loaded from ROMs. The term "firmware" isn't used too much on PCs though. [[User:Intx13|Intx13]] 12:11, 19 November 2015 (CST)

:: I went ahead and changed references to "UEFI BIOS" to "UEFI firmware" where it was appropriate. I left "legacy BIOS" alone. That way, "BIOS" only appears in legacy contexts, and "UEFI" is not interchanged with "BIOS" (although it would be correct to do so!) Feedback is appreciated! [[User:Intx13|Intx13]] 12:45, 19 November 2015 (CST)

::: Wow! I just read the updates from that time and it looks great! The updated Secure Boot section was especially interesting to me (I'm not a bare-metal dev). Sorry I didn't reply earlier -- my context got switched at the time! --[[User:Critiqjo|Critiqjo]] 14:19, 30 July 2016 (CDT)

== GPT ==

I'm thinking that GPT should probably be a completely separate page.. [[User:Brendan|Brendan]] 15:12, 29 July 2011 (UTC)

:Agreed. But first, we need some content for it :) --[[User:Love4boobies|Love4boobies]] 13:49, 1 August 2011 (UTC)

== GCC Cross-Compiler Target ==

> The result of this process is a 44 kB PE executable file main.efi. On a real project you'll probably want to use make or another build tool, '''and may need to build a cross-compiler'''.

Ok, but it should say what is the target for a GCC cross-compiler, since they use GCC above... [[User:Glauxosdever|Glauxosdever]] 03:39, 10 July 2016 (CDT)

Also, binutils no longer seems to target efi-app targets, in GNUEFI they use objcopy from elf to binary and it says in the documentation they create the header manually in assembler. If anyone knows the steps for this, it would make a great addition here. --[[User:Kieran|Kieran]] 02:10, 3 August 2016 (CDT)

:Best bet for now is probably to use '''x86_64-w64-mingw32-gcc''' as per [[UEFI Bare Bones]] until something more appropriate is added to gcc [[User:Jnc100|John]] 04:08, 3 August 2016 (CDT)

:GCC 8.3.0 ( From the Ubuntu repos ) seems to include the <tt>efi-app-x86_64</tt> target, even if it's not included in the list of targets printed out using <tt>objcopy --info</tt>. That might be a bug with binutils. --[[User:Ajxs|Ajxs]] 17:41, 13 August 2019 (CDT)

== 32- vs. 64-bit ==

My own early experiments seem to suggest that UEFI itself (on Intel I mean) is either 32-bit or 64-bit (and even a 64-bit-capable CPU may have 32-bit UEFI in ROM, on older systems anyway), and the 64-bit version does *not* contain backward-compatibility with 32-bit UEFI programs.  Is this correct?  If so it's kind of a huge deal (every 32-bit OS needs to have both 32- and 64-bit versions of the loader) and it seems as if it deserves being mentioned loudly here.  Also I believe the default 32-bit loader is named \EFI\BOOT\BOOTIA32.EFI, not \EFI\BOOT\BOOT.EFI .  That's what the spec and my tests with 32-bit OVMF show, anyway.  --[[User:Johnwilson|Johnwilson]] 00:14, 5 December 2017 (CST)

This I believe is correct. Hence, if you want a 64 bit OS, but have a 32 bit UEFI, you still need to set up long mode yourself. I'm not sure if compatibility mode would work with the runtime services - boot services are clearly out as UEFI still has control of the paging structures. --[[User:Bellezzasolo|Bellezzasolo]] 06:00, 5 December 2017 (CST)

While technically correct; it's less troublesome than you might assume. Microsoft have stated that they will never support 32-bit UEFI on 80x86 (and as far as I know they haven't), so the number of 80x86 computers with 32-bit UEFI is "almost zero" (mostly limited to a few old Apple machines, which are relatively quirky for other reasons). Also (potentially excluding old Apple machines) it's extremely likely that any computer with 32-bit UEFI (if any actually exist?) also supports BIOS and is already configured to boot using BIOS by the end user and/or manufacturer. Essentially; you can ignore 32-bit UEFI (and only support BIOS and 64-bit UEFI). --[[User:Brendan|Brendan]] 11:22, 6 December 2017 (CST)

Excellent logic -- thank you!  --[[User:Johnwilson|Johnwilson]] 00:46, 8 December 2017 (CST)

== Arm UEFI calling convention ==

The page currently shows:

> Calling Conventions

> UEFI specifies the following calling conventions:

> [...]

> SMC for ARM UEFI functions

I don't think that's correct, can you provide any reference for this?

From my understanding, the OS kernel and UEFI run on the same EL. When UEFI hands over control to the OS kernel it hands over a pointer to the EFI_SYSTEM_TABLE, which includes pointers to functions the OS kernel can call. These functions are then called by simply jumping to the given address, not by executing an SMC.

Looking at how the Linux kernel interacts with UEFI on Arm, I cannot see any SMCs being used to call UEFI functionality.

SMCs are usually used by UEFI to forward the request from the OS kernel to the trusted world, e.g. for variable storage or firmware updates, but I don't think that's relevant when describing the UEFI calling convention.
````

<a id="w003026"></a>

## W003026 — Megalithic Kernel

Bron-ID: W003026
Namespace: Main (0)
Revisie: 27517 | Datum: 2022-12-12T04:08:52Z
Bron: https://wiki.osdev.org/Megalithic_Kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Megalithic_Kernel&oldid=27517

````text
{{Stub}}
{{Template:Kernel designs}}

A '''Megalithic Kernel''', or ''Megakernel'', is a kernel designed to only run binaries in kernel mode. This is in contrast to a usual monolithic kernel or microkernel, which are typically designed to handle multiple user-mode processes. Megalithic kernels are simpler in design. They often need not support task switching for processes or virtual memory, and often are just a single monolithic binary, simplifying linking. Because Megalithic kernels are limited in the range of binaries that they can run, they are often not as satisfying as other types of kernels to develop except for very specific purposes.

Megalithic kernels may employ a variety of strategies to get around the perceived limitations of never running a user process. Some megalithic kernels may compile all application code into the kernel, but this is generally not convenient. The kernel would need to be recompiled any time a new application is installed, removed, or otherwise altered.

[[Category:Kernel]]
````

<a id="w003027"></a>

## W003027 — Talk:Megalithic Kernel

Bron-ID: W003027
Namespace: Talk (1)
Revisie: 29435 | Datum: 2025-03-13T14:09:18Z
Bron: https://wiki.osdev.org/Talk:Megalithic_Kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMegalithic_Kernel&oldid=29435

````text
Is this a real term? Citation needed :) --[[User:Love4boobies|Love4boobies]] 12:57, 1 August 2011 (UTC)

The phrase has certainly been used before within the hobby OSdev community, though I agree it's not an academic term. Still, it's a useful term. Do you have a better suggestion to name this sort of kernel? [[User:Xvedejas|Xvedejas]] 14:24, 1 August 2011 (UTC)

No, I have no problem with the name. I just think it might be dangerous to invent terms because we'll end up confusing people. Names obviously need to come from somewhere, but I don't think this wiki is the place---it should probably come from the industry, we are a small community here.

As for the VM thing, I don't think you got my point: Address spaces are not a hardware concept; they can be implemented in software just as well. E.g., Singularity is a microkernel even when configured to run everything in kernel mode (processor-wise). Hence, if you implement a VM in a megalithic kernel, applications won't be able to see/touch the kernel or other applications---you end up with either a microkernel or a monolithic one, depending on the design.

--[[User:Love4boobies|Love4boobies]] 18:46, 1 August 2011 (UTC)

Fair enough; what I'm trying to describe then is not a design in opposition to a monolithic/mircokernel, but it is still a class of kernels that doesn't seem to be discussed on this wiki. Would you suggest instead I remove this article and create a section on the article [[Kernel]]? [[User:Xvedejas|Xvedejas]] 19:11, 1 August 2011 (UTC)

:I'd say Wikipedia procedure applies, more exactly WP:OR. A Google search for "megalithic kernel" yielded a whooping 19 (!) results, of which just half a dozen use the term in the way described here, and of ''those'' almost all use it in a derogatory way. I don't think it's a "design pattern" we should describe here at all. That's like describing the benefits of spaghetti code in a Wiki about coding styles... -- [[User:Solar|Solar]] 12:41, 2 August 2011 (UTC)

I'm not sure, but this might be where "megalithic" came from: [http://forum.osdev.org/viewtopic.php?f=1&t=9735&p=61607&hilit=megalithic#p61607] (I'm not aware of any older uses). To me it means "drivers and applications as part of the kernel", in the same way that "monolithic" means "drivers as part of the kernel". It shouldn't be confused with things like managed OSs, where applications aren't built into the kernel (but do run at CPL=0, possibly inside of some sort of sandbox with software isolation or virtualisation/byte-code). I'd also assume that the only sane use for a "megalithic" kernel is in embedded systems. For example, imagine a thin-client with a VNC client built into its firmware, or a router with a HTML server (for a configuration interface) built into ROM. --[[User:Brendan|Brendan]] 01:44, 4 August 2011 (UTC)

:Yes, I think [[User:Xvedejas|Xvedejas]] got the point that if kernels are implmeneted as he describes in the article, then they are not acutally megalithic. However, I think the most important issue is whether we should call them megalithic on the wiki or not. So far 2 people voted '''no'''. --[[User:Love4boobies|Love4boobies]] 02:37, 4 August 2011 (UTC)

OSnews (correctly) used the word "megalithic" today: http://www.osnews.com/story/25463/Dreckig_OS_and_the_Megalithic_Kernel  --[[User:Brendan|Brendan]] 04:53, 29 December 2011 (UTC)

:It's likely that you coined that term :) --[[User:Love4boobies|Love4boobies]]

http://en.wikipedia.org/wiki/Megalithic_kernel here is some more proof!
--[[User:Gat|Gat]]

== Rename to Unikernel ==

The description seems to match that of a Unikernel https://en.wikipedia.org/wiki/Unikernel

Unless there are any objections I would rename this page to Unikernel.

-- [[User:Demindiro|Demindiro]] ([[User talk:Demindiro|talk]]) 14:09, 13 March 2025 (UTC)
````

<a id="w003028"></a>

## W003028 — User:Xvedejas

Bron-ID: W003028
Namespace: User (2)
Revisie: 11866 | Datum: 2011-08-01T14:24:31Z
Bron: https://wiki.osdev.org/User:Xvedejas
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AXvedejas&oldid=11866

````text
http://valix.org/
````

<a id="w003029"></a>

## W003029 — User talk:Combuster/Relicensing

Bron-ID: W003029
Namespace: User talk (3)
Revisie: 12615 | Datum: 2012-02-17T05:57:15Z
Bron: https://wiki.osdev.org/User_talk:Combuster/Relicensing
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ACombuster%2FRelicensing&oldid=12615

````text
I'll be away for a while so I give my approval to relicense my contributions under CC0 now. Sorry for not using the forum, it seems to be down atm. --[[User:Love4boobies|Love4boobies]] 13:20, 17 August 2011 (UTC)
:Will do, noted - [[User:Combuster|Combuster]] 09:40, 18 August 2011 (UTC)

== Markup somewhat unclear ==

I am still a bit confused as to what the different types of markup actually mean.

 * a (-) behind a user means he didn't make any Wiki contributions.
 * a (m) behind a user means he only made minor contributions.
 * a "signed off" means a user declared his past contributions to be under CC0.

What does a struck-out user *name* mean? -- [[User:Solar|Solar]] 12:38, 21 November 2011 (UTC)
: Any struck out user simply means that that any licensing issues with his contributions has been dealt with - reason and other background information usually following the name. It makes the users in need of some (additional) nagging stand out in the list. - [[User:Combuster|Combuster]] 18:22, 21 November 2011 (UTC)

== Suggestion: ==

Why not ask chase to add an "implicit compliance" clause to the registration form for the forum? Or on the area where you add yourself to the wiki-editors group. Something like, "By adding yourself to this group, you agree that all of your future submissions will conform to the chosen license of the wiki, subject to the discretion of the administration."

^ Should help ease your work somewhat :) -- [[User:gravaera|gravaera]] 2:32, 16 February 2012 (GMT-5)

:If you read the text around the "Save page" button, we got that part covered. This mess is only about people who predated this change. - [[User:Combuster|Combuster]] 23:57, 16 February 2012 (CST)
````

<a id="w003030"></a>

## W003030 — SysLinux

Bron-ID: W003030
Namespace: Main (0)
Revisie: 12548 | Datum: 2012-02-05T20:59:31Z
Bron: https://wiki.osdev.org/SysLinux
Revisie-URL: https://wiki.osdev.org/index.php?title=SysLinux&oldid=12548

````text
SysLinux is a multiboot compatible bootloader that can be installed on a FAT formatted Floppy Disk, The project has three sister projects, ISOLinux, EXTLinux, and PXELinux for El Torito CD's, ext file systems, and Intel's Preboot Execution Enviroment, respectively. 

=Installation=
Installing SYSLinux will install a bootloader to the disk, and will copy a file '''LDLINUX.SYS''' on the root directory.

==Windows==
===Syntax:===
<pre> syslinux.exe [-sfmar] [-d directory] <drive>: [bootsecfile]
</pre>

===Installation on a Floppy Disk:===
<pre>
syslinux.exe a:
</pre>
===Installation on a HD / Flash Drive===
Where the drive letter is Z:
<pre>
syslinux.exe -m -a -d /boot/syslinux z:
</pre>

==DOS==
Same as windows version, but with '''.com''' instead of '''.exe'''

==Linux==
===Syntax:===
<pre>
syslinux [-sfr] [-d directory] [-o offset] <Device/Image>
</pre>
===Example===
<pre>
syslinux /dev/fd0
</pre>

The '''-o''' option can be used with a disk image file and specifies the byte offset of the filesystem image in the file.

=Configuration=
SysLinux uses a configuration file called '''syslinux.cfg'''. It searches for the file in the following places in order:
* /boot/syslinux/syslinux.cfg
* /syslinux/syslinux.cfg
* /syslinux.cfg

The configuration file is common to all variants, and uses the following format:
<pre>
DEFAULT Name
LABEL Name
   SAY Now Booting Your kernel
   KERNEL /kernel.bin
   APPEND initrd=file.tar
</pre>

Individual Commands:
===DEFAULT===
The default kernel to load, has to be one of the labels

===LABEL===
The human readable name of the kernel to boot, this is used in menus.

===SAY===
Write to the screen when the kernel is loading

===KERNEL===
The Location of the kernel to load to

===APPEND===
Additional options on boot, some are ro, initrd, and rootfs

Please note that all files are relative to the config file's path unless preceded by a slash

=See Also=
===Articles===
* [[GRUB]]
* [[LILO]]
===External Links===
* [http://www.syslinux.org/wiki/index.php/SYSLINUX The SYSLinux Wiki]
* [http://www.linux-tutorial.info/modules.php?name=Howto&pagename=Remote-Serial-Console-HOWTO/configure-boot-loader-syslinux.html SYSLinux Serial Terminal]

[[Category:Bootloaders]]
````

<a id="w003031"></a>

## W003031 — User:Thepowersgang/ARM Notes

Bron-ID: W003031
Namespace: User (2)
Revisie: 11969 | Datum: 2011-08-30T14:50:58Z
Bron: https://wiki.osdev.org/User:Thepowersgang/ARM_Notes
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AThepowersgang%2FARM_Notes&oldid=11969

````text
Just some notes on doing OSDev on an ARM platform

= Assembly Snippets =
== Spinlocks ==
=== Pre ARMv6 ===
<source lang="c">
int v = 1;
while(v) {
    __asm__ __volatile__("swp %0, %1, [%2]" : "=r" (v) : "r" (1), "r" (&lock));
}   
</source>
=== Post ARMv6 (preferred version) ===
<source lang="c">
int tmp = 0; 
__asm__ __volatile__ (
    "1:  ldrex   %0, [%1]" // Exclusive load (obtains a HW lock)
    "    teq     %0, #0"   // Check if the lock is avaliable
    "    strexeq %0, %2, [%1]" // Mark the lock if it's avaliable
    "    teqeq   %0, #0"   // if the lock was avaliable, but we didn't obtain it...
    "    bne     1b"       // Try again
    : "=&r" (tmp)
    : "r" (&lock), "r" (1)
    : "cc" // Condition codes clobbered
    );
</source>
````

<a id="w003032"></a>

## W003032 — Talk:GCC Cross-Compiler for x86 64

Bron-ID: W003032
Namespace: Talk (1)
Revisie: 11997 | Datum: 2011-09-10T21:04:30Z
Bron: https://wiki.osdev.org/Talk:GCC_Cross-Compiler_for_x86_64
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AGCC_Cross-Compiler_for_x86_64&oldid=11997

````text
This page is no longer relevant (since 4.3.2/2.18). Any objections to deleting it? --[[User:Aj|Aj]] 15:00, 7 September 2011 (UTC)

'''Oppose.''' There are some practical reasons not to use a >=4.3 GCC, for instance if you want to avoid the trouble of building the external libs or if you already have a setup and want to recreate it with the same versions elsewhere. Add to that my general reluctance of removing otherwise valid information, adding a note of deprecation and providing the current instructions would be more correct. - [[User:Combuster|Combuster]] 21:27, 7 September 2011 (UTC)

Understood - have made some ordering changes to the page to make things more obvious. Original page was written when <=4.3.1 was current, so I have also updated the tense used to reflect this. --[[User:Aj|Aj]] 21:04, 10 September 2011 (UTC)
````

<a id="w003033"></a>

## W003033 — X86 Instruction Encoding

Bron-ID: W003033
Namespace: Main (0)
Revisie: 12000 | Datum: 2011-09-12T16:37:44Z
Bron: https://wiki.osdev.org/X86_Instruction_Encoding
Revisie-URL: https://wiki.osdev.org/index.php?title=X86_Instruction_Encoding&oldid=12000
Doorverwijzing: X86-64 Instruction Encoding

````text
#REDIRECT [[X86-64 Instruction Encoding]]
````

<a id="w003034"></a>

## W003034 — GCC Cross Compiler

Bron-ID: W003034
Namespace: Main (0)
Revisie: 12001 | Datum: 2011-09-15T09:01:01Z
Bron: https://wiki.osdev.org/GCC_Cross_Compiler
Revisie-URL: https://wiki.osdev.org/index.php?title=GCC_Cross_Compiler&oldid=12001
Doorverwijzing: GCC Cross-Compiler

````text
#REDIRECT [[GCC_Cross-Compiler]]
````

<a id="w003036"></a>

## W003036 — User:Shikhin/Tutorial SMP

Bron-ID: W003036
Namespace: User (2)
Revisie: 12017 | Datum: 2011-09-18T15:57:39Z
Bron: https://wiki.osdev.org/User:Shikhin/Tutorial_SMP
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AShikhin%2FTutorial_SMP&oldid=12017

````text
{{Rating|3}}

'''SMP''', or [[Symmetric Multiprocessing]] is one method of having multiple processors in one computer system. Several hobbyist OS Developers leave SMP support for the future. However, even though supporting SMP itself is comparatively easy, the "one-big lock" chaos it could create often leads to rewrites.

Thus, starting with SMP support early is recommended, and explaining it is what the tutorial aims at. The tutorial covers basic locking primitives, however, it expects the reader to have a base knowledge on Virtual Memory and Multiprocessing systems.

Moreover, the tutorial, unlike other tutorials, doesn't believe in providing source. In some cases, it may contain appropriate pseudo code and table structures, however full source should '''NOT''' be expected.

==Multiprocessing==

[[Multiprocessing]] system, basically, refers to a system where multiple processors execute code simultaneously. Multiprocessing systems can be divided into two sub-categories:

* '''UMA''' or '''Uniform Memory Access''' architectures, are those architectures, in which all processors share the same Physical Memory, with uniform time accesses.

* '''NUMA''' or '''Non-Uniform Memory Access''' architectures, on the contrary, are those architectures, in which processors access their own local memory faster than non-local memory.

Contrary to popular belief, NUMA and SMP architectures are not mutually exclusive, as is demonstrated by the latest set of Intel processors. In this article, for simplicity sake, we would not be covering optimization for NUMA architectures, and nor for [[SMT]] architectures. 

==Terminology==

I'd be using few terms through out the tutorial, which may be a little tough to understand. Here, I'd maintain a glossary for all the terminology I use.

* '''SMP''', or '''Symmetric Multiprocessing''', is one method of having multliple processors in a computer.
* '''SMT''', or '''Symmetric Multithreading''', is another type of multiprocessing, where the idle time in a processor is used for another "thread".
* '''UMA''', or '''Uniform Memory Access''' refers to a architecture where all processors share the same Physical Memory with uniform time accesses.
* '''NUMA''', or '''Non-Uniform Memory Access''' refers to a architecture, where processors access their own local memory faster than non-local memory. 
* '''MPS''', or the '''MultiProcessing Specification''' is a deprecated specification developed by Intel, and has been superseded by ACPI. 
* '''ACPI''', or '''Advanced Configuration and Power Interface''', is a configuration standard for the PC, developed by Intel, Microsoft and Toshiba. ACPI provides a set of tables, which come useful to us.
* '''APIC''', or '''Advanced Programmable Interrupt Controller''' is the updated Intel standard for the older [[PIC]], which is used in multiprocessor systems and is an integral part of all recent Intel (and compatible) processors. The APIC is used for sophisticated interrupt redirection, and for sending interrupts between processors. 
* When the computer starts, control is handled to a BSP or '''BootStrap Processor'''. This is what your code is currently running on.
* The other processors are classified as APs or '''Application Processors'''. These are the processors we would be attempting to start.

==Gathering Information==

To get a list of all the APs in a system, either the MPS or ACPI tables can be used. The MPS tables are deprecated and have been superseded by ACPI. The MPS tables are proven to be buggy in SMT cases, and thus aren't recommended. However, on systems where ACPI isn't present, the MPS tables can be used as a backup.

These tables provide a list of all the APICs in a computer, which can then be initialized.

===MPS===

The MPS or MultiProcessor Specification tables are present in all SMP systems. If this table isn't present in the a system, the system can be assumed to be a UniProcessor system. 

====MP Floating Point Structure====

To use these tables, the MP Floating Point Structure must first be found. As the name suggests, it is a Floating Point Structure and must be searched for.

The table is present on a 16-byte boundary, and is marked by the signature "_MP_". To find the table, the following areas must be searched in:

a) In the ''first kilobyte'' of ''Extended BIOS Data Area'' (EBDA), or

b) Within the ''last kilobyte'' of ''system base memory'' (e.g., 639K-640K for systems with 640KB of base memory or 511K-512K for systems with 512 KB of base memory) if the EBDA segment is undefined, or

c) In the ''BIOS ROM address space'' between ''0xF0000 and 0xFFFFF''.


The EBDA isn't standardized, and it's address can be found in the BDA. The word at 0x040E usually contains the address of EBDA shifted right by four bits. Thus, the address of the EBDA can be found by doing something like ''*((uint16_t*)0x040E) << 4''. The author recommends performing checks on this address, since this isn't much standardized too. You could check that it falls in the EBDA area, as listed in [[EBDA]]. 

If the check fails or the MP Floating Point Structure isn't present in the EBDA region, the ''last kilobyte'' of System base memory should be searched. If you use a Multiboot compatible bootloader, the ''mem_lower'' field at offset 4 lists the amount of base memory. If you are writing your own bootloader, then ''int 0x12'' can be used to get the amount of base memory. 

If the MP Floating Point Structure can't be found in this area, then the area between 0xF0000 and 0xFFFFF should be searched. The MP Floating Point Structure follows the following format:

<source lang="c">
typedef struct
{
    // The Signature, must contain _MP_, and is present on a 16 byte boundary.
    uint8_t  Signature[4];

    // The address of the MP Configuration Table. 
    uint32_t MPConfigurationTable;

    // The length of the floating point structure table, in 16 byte units. This field *should* contain 0x01, meaning 16-bytes.
    uint8_t  Length;

    // The version number of the MP Specification. A value of 1 indicates 1.1, 4 indicates 1.4, and so on.
    uint8_t  Version;

    // The checksum of the Floating Point Structure. 
    uint8_t  Checksum;

    // Few feature bytes.
    uint8_t FeatureBytes[5];
} MPFloatingPoint;
</source>

If the first feature byte is non-zero the MP Configuration Table is not present, and a default configuration, defined in Chapter 5 of the MultiProcessor Specification is used. For the sake of simplicity, I would assume that the first feature byte is always zero. 

However, if you want your OS to run in all supported configurations, every default configuration, as defined in Chapter 5, must be accounted for.

=====Recommended Usage=====

First of all, the area defined above must be searched in. Once the table is found, the checksum of the table must be done.

If the checksum fails, the system must be assumed to be a UniProcessor system. 

====MP Configuration Table====


=====Recommended Usage=====

===ACPI===

===Extra Notes===

==See Also==

* [[Multiprocessing]]

===Threads===

* [[Topic:11564|init-SIPI]]

===External Links ===

* [http://www.gnu.org/software/grub/manual/multiboot/multiboot.html]
````

<a id="w003037"></a>

## W003037 — ARM Beagleboard

Bron-ID: W003037
Namespace: Main (0)
Revisie: 30411 | Datum: 2026-03-24T22:34:19Z
Bron: https://wiki.osdev.org/ARM_Beagleboard
Revisie-URL: https://wiki.osdev.org/index.php?title=ARM_Beagleboard&oldid=30411

````text
{{Tone}}
{{In Progress}}

TODO: VERIFY THE TUTORIAL(S)!!!

==Intro==
This is a tutorial on bare-metal [OS] development on the Texas Instruments BeagleBoard. This tutorial is written specifically for the BeagleBoard-xM Rev C because the author has no other hardware to test on.

Experience in Linux/nix ('''very''' important) and at least one assembly language ('''incredibly''' important, or at least extensive C knowledge) is assumed and required. Please make sure you can figure out how to get a basic x86 OS (no copypaste!) to the booting stage before attempting this guide. No need to make much more than a "Hello World", since what you know about x86 won't really apply here.

=== Materials ===
You will need a:
* '''BeagleBoard-xM Rev C''', or perhaps an older board, ''with a serial port''.
* An '''RS-232''' serial port. This is the kind of serial port on the back of your x86 PC, and on the BeagleBoard-xM. using TTL (5v) or lower voltage serial will damage something.
* A serial cable that comes to '''DB-9 Male''' on at least one side (to connect to the BeagleBoard)
* '''Debian, or other *nix with arm-binutils''' [guide will document installation for Debian]
* '''Power supply''' for your BeagleBoard ''(5v, at least 1A but I recommend/use 3A)''
* '''A copy of the ARM ARM (ARM Architecture Reference Manual) (download a PDF, make sure it's the new one, with ARMv7 stuff)'''
* '''A copy of the OMAP35x/DM37x TRM (Technical Reference Manual)''' <-- ^ ''THESE ARE INCREDIBLY IMPORTANT!''
:* '''Optional:''' Various peripherals for the BeagleBoard. Be aware that ''you'' will need to write drivers for them. Welcome to OS Development ^_^.

==Getting Started==

===Testing your serial port===

First things first, you're going to want to make sure all your hardware works. Set up your serial port, however yours works, and open up minicom. Make sure you have flow control turned off.
Now jumper '''PIN 2 (RX)''' to '''PIN 3 (TX)''' on the DB-9 side. Just type some characters into minicom, and they should be echoed back.
Ensure you can run at 115200 baud, 8N1, which is what you will use to connect to the BeagleBoard.

If you get 'Permission Denied' '''do NOT become root!''' This is unnecessary. Instead do:
<syntaxhighlight lang="bash">
sudo adduser <user> dialout
</syntaxhighlight>
This will let your user use serial ports without needing root.

Or do ls -l /dev/ttyS* to find out the group that own the device, then add you into that group under /etc/group (normally the group is uucp)

===Testing the BeagleBoard===

Follow the guide here http://elinux.org/BeagleBoardDebian#Debian_armhf_port to install the Debian armhf image to a microSD card. We don't really need to boot Debian from it, but it's a nice simple <small>ghetto/lazy</small> way to get u-boot set up and working.

Connect the BeagleBoard up via serial, and make sure Minicom is running (on your PC). Put the microSD card in the BeagleBoard. Connect the BeagleBoard to its power supply. You should see some output on the serial port. If not, Google around to figure out why not. If a few bytes of garbage appear in Minicom only, make sure the microSD card is inserted and VALID. Also make sure you didn't press the 'user button'.

You should get some meaningful output. Unplug your BeagleBoard when you are done with it.

===Getting an assembler===

We will be using ARM development tools from the Emdebian project.

Append:
<syntaxhighlight lang="bash">
deb http://www.emdebian.org/debian/ squeeze main
</syntaxhighlight>
to your /etc/apt/sources.list, changing 'squeeze' for your version of Debian - I'm running wheezy, for example.

now run:
<syntaxhighlight lang="bash">
sudo aptitude update
sudo aptitude install gcc-4.4-arm-linux-gnueabi # check this! it might be an older version for non-wheezy...
</syntaxhighlight>

That will install a C compiler, and also install Binutils, which for this tutorial is really all we need, but you may later wish to write your code in C.

==Writing some CODE!==

Now that we have all the stuff we need, let's get to writing some code. Before we write an ASM source file, we should make a linker script and a Makefile.

===Linker Script===

We'll basically take the one from the stackoverflow.com link, No.1 in the ref list, ''with one change''. The linker script there uses 0x80300000 as the start address. This will work just fine, however the default load address of u-boot's 'loady' command loads things into 0x80200000. So there's one less thing we have to specify ^_^.

====linker.ld====

Take note of what it does. It's pretty simple. Refer to docs if necessary.

<pre>
/* rammap */
MEMORY
{
    ram : ORIGIN = 0x80200000, LENGTH = 0x10000
}

SECTIONS
{
    .text : { *(.text*) } > ram
}
</pre>

===Makefile===

This is the basic makefile I use. It's similar to the one on the stackoverflow link.

<syntaxhighlight lang="makefile">
ARMGNU = arm-linux-gnueabi

AOPS = --warn --fatal-warnings
COPS = -Wall -Werror -O2 -nostdlib -nostartfiles -ffreestanding 

boot.bin: boot.asm
	$(ARMGNU)-as boot.asm -o boot.o
	$(ARMGNU)-ld -T linker.ld boot.o -o boot.elf
	$(ARMGNU)-objdump -D boot.elf > boot.list
	$(ARMGNU)-objcopy boot.elf -O srec boot.srec
	$(ARMGNU)-objcopy boot.elf -O binary boot.bin
</syntaxhighlight>

===Assembly!===

YAY! We can write some code now!

You are free to branch to C code after following this tutorial, but remember, you will need to write some things in ASM, just like on x86.

====boot.asm====

<syntaxhighlight lang="asm">
#UART base locations from the TRM
.equ UART1.BASE, 		0x4806A000
.equ UART2.BASE, 		0x4806C000
#According to the Beagleboard-xM System Reference Manual, UART3 is connected to the serial port.
.equ UART3.BASE, 		0x49020000
.equ UART4.BASE, 		0x49042000

#We need to be in ARM mode - we may branch to Thumb mode later if desired.
.arm
_start:
   # Thankfully, U-BOOT already sets up serial for us, so we don't need to do all the reset crap.
   # Just print a '!' symbol to the screen, and hang.
   # Load the location of the transmit register (THR_REG) into r0. It's at the UARTx.BASE, since it is offset 0x000.
   # (load value pointed by the assembler (immediate pointer) into r0)
   ldr r0,=UART3.BASE
   # Move a '!' character into r1
   # (move a character, '!', which is immediate, into r1)
   mov r1,#'!'
   # According to the TRM, we may only write bytes into THR_REG, else we'll corrupt stuff.
   # (store the least-significant-byte of r1 into the address pointed by r0)
   strb r1,[r0]
   # If we kept writing to the serial port, we'd eventually overflow the 64-byte FIFO, and since we don't handle interrupts yet, we'll hang (?)
   # In ASM, labels are like case: statements in C. Code flows into them as if they don't exist - because they don't.
_hang:
   # (branch (jump, JMP) to _hang)
   # b _hang
</syntaxhighlight>

==='Compiling' it===

Since it's ASM we technically don't 'compile' it, we 'assemble' (and 'link' it). But anyway, go to where you saved your files and type 'make'. This should progress without any errors.

==Executing code on the device==

Remember that Minicom window we almost forgot about? It's time to use it ;).

===Introduction to u-boot===

Reset your BeagleBoard (hit the reset button).

You should see:

<pre>
U-Boot 2011.03-rc1-00000-g9a3cc57-dirty (Apr 04 2011 - 12:36:16)                
                                                                                
OMAP36XX/37XX-GP ES2.1, CPU-OPP2, L3-165MHz, Max CPU Clock 1 Ghz                
OMAP3 Beagle board + LPDDR/NAND                                                 
I2C:   ready                                                                    
DRAM:  512 MiB                                                                  
NAND:  0 MiB                                                                    
MMC:   OMAP SD/MMC: 0                                                           
*** Warning - readenv() failed, using default environment                       
                                                                                
In:    serial                                                                   
Out:   serial                                                                   
Err:   serial                                                                   
Beagle xM Rev C                                                                 
No EEPROM on expansion board                                                    
Die ID #2e7000029ff80000016842c813020023                                        
Hit any key to stop autoboot:  3
</pre>

Hit a key. Before Linux loads.

<pre>
OMAP3 beagleboard.org # 
</pre>

Whee, a u-boot prompt.

===Loading code===

Now it's time to load our code into RAM. To do this, we will use 'loady' and the y-modem protocol. We can do all this from Minicom - that's why we needed Minicom and not GNU Screen. Hopefully you did what I asked ;).

Just type 'loady'. Don't worry about being fast on your first go, you'll miss it anyway.

<pre>
OMAP3 beagleboard.org # loady                                                   
## Ready for binary (ymodem) download to 0x80200000 at 115200 bps...
</pre>

Now hit Ctrl+A. A bar will appear at the bottom.

Press 'S'.

Select 'ymodem'.

Move across to '[Goto]'.

Type in the directory where you saved your code.

Move down to 'boot.bin' and hit 'Space', and then 'Enter'.

Ctrl+C out of the ymodem transfer window if nothing happens.

Reset your BeagleBoard and try again - now that the file is already selected, you can just hit enter twice in the file prompt window, preventing timeouts.

Be patient, ymodem can take a few seconds to 'sync'.

After the file is loaded into RAM,
<pre>
READY: Press any key.
</pre>
and
<pre>
Cm - CRC mode, 9(SOH)/0(STX)/0(CAN) packets, 3 retries                          
## Total Size      = 0x00000350 = 848 Bytes                                     
OMAP3 beagleboard.org # 
</pre>
is printed (my file is bigger... don't worry about that), we just have to execute the code in memory!

===Executing Code===

Execution is started from an address with 'go'. Our _start address is 0x80200000. So, we type:
<pre>
OMAP3 beagleboard.org # go 0x80200000                                           
</pre>

and we should get:

<pre>
## Starting application at 0x80200000 ...                                       
!
</pre>

If you get that, congratulations! You've just written an 'Operating' System (;)) for the BeagleBoard! The process will seem long winded at first, but it's not really at all.

==What to do now==

Read the ARM ARM.

Read the TRM.

Find out how to do push/pop and how to return from functions (and how to return using a pop) from example code and the ARM.

Write routines to print integers and strings.

Set up virtual memory.

Enable interrupts

Write some device drivers.

Write a scheduler

etc. Do all the stuff you would in an x86 OS.

==Branching to C==

TODO: Add this
:Dependencies of TODO: Care enough to want to branch to [[C]].

It's probably just a matter of linking in your C program and doing
<syntaxhighlight lang="asm">
   bx _main
</syntaxhighlight>

==Help==

For help, you can try joining #Beagle (note the capital B!) on irc.freenode.net.

== See Also ==
=== Articles ===
*[[ARM Integrator-CP Bare Bones]]
*[[ARM Overview]]
*[[GameBoy Advance Barebones]]
*[[GameBoy Advance Introduction]]

== References ==
* http://stackoverflow.com/questions/6870712/beagleboard-bare-metal-programming
* http://sourceware.org/binutils/docs/as/

[[Category:ARM]]
````

<a id="w003038"></a>

## W003038 — Beagleboard

Bron-ID: W003038
Namespace: Main (0)
Revisie: 12054 | Datum: 2011-09-30T15:36:05Z
Bron: https://wiki.osdev.org/Beagleboard
Revisie-URL: https://wiki.osdev.org/index.php?title=Beagleboard&oldid=12054
Doorverwijzing: ARM Beagleboard

````text
#REDIRECT [[ARM Beagleboard]]
````

<a id="w003039"></a>

## W003039 — User:Doublebeta

Bron-ID: W003039
Namespace: User (2)
Revisie: 12060 | Datum: 2011-10-01T11:21:33Z
Bron: https://wiki.osdev.org/User:Doublebeta
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADoublebeta&oldid=12060

````text
Working on the [DIY] DB/UX operating system on Beagleboard-xM/ARM.

Currently stuck on setting the MMU up. Completely lost.

Someone test/fix my tutorial pls. ;).
````

<a id="w003040"></a>

## W003040 — User:Renagadexx

Bron-ID: W003040
Namespace: User (2)
Revisie: 12106 | Datum: 2011-10-14T17:34:34Z
Bron: https://wiki.osdev.org/User:Renagadexx
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ARenagadexx&oldid=12106

````text
Nothing here yet!

I live in Texas.
````

<a id="w003041"></a>

## W003041 — User talk:Renagadexx

Bron-ID: W003041
Namespace: User talk (3)
Revisie: 12114 | Datum: 2011-10-17T07:20:21Z
Bron: https://wiki.osdev.org/User_talk:Renagadexx
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ARenagadexx&oldid=12114

````text
"I live in Texas." - Condolences. :-D -- [[User:Solar|Solar]] 07:20, 17 October 2011 (UTC)
````

<a id="w003042"></a>

## W003042 — Talk:Sparc Barebones

Bron-ID: W003042
Namespace: Talk (1)
Revisie: 12137 | Datum: 2011-10-25T04:52:28Z
Bron: https://wiki.osdev.org/Talk:Sparc_Barebones
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ASparc_Barebones&oldid=12137

````text
I think the original author mistook the ';' to be the line comment character for the SPARC (which it isn't, '!' is). Unless the SPARC opcode syntax is ''requiring'' the semicolon to be there, I'd say delete it for added clarity. -- [[User:Solar|Solar]] 07:23, 17 October 2011 (UTC)

That was the case.. I realized this after your suggestion and checking the TKisem lab manual -cb88
````

<a id="w003043"></a>

## W003043 — TKisem

Bron-ID: W003043
Namespace: Main (0)
Revisie: 12152 | Datum: 2011-11-03T09:43:26Z
Bron: https://wiki.osdev.org/TKisem
Revisie-URL: https://wiki.osdev.org/index.php?title=TKisem&oldid=12152

````text
Tkisem is "an instructional emulator for version 8 of the SPARC instruction set with a GUI based on Tcl/Tk." It seems to be mostly complete, but not quite as robust as QEMU. However, it may be a better starting point for learning Sparc Assembly language. It does have a non-standard MMU implementation.

==Running Tkisem==

To run the simulator, you need to have TCL 8.0 and TK installed. If you see missing library problems on Linux, just create a symlink from the the current version to the version Tkisem is looking for.

Windows and Linux both support '''tkisem'''; Linux additionally provides '''gxisem''', which has a X11 interface.

==Installation==

There are installation instructions and lab manuals along with binaries in the distribution.

== Tips ==

If you install TKisem on Windows, you may have trouble running '''as''' and '''ld'''. If you get a stack fault, try running the commands '''as-isem''' and '''ld-isem''' instead of '''isem-as''' and '''isem-ld'''. Also don't forget to add the programs to your PATH.

== Web Links ==

* [http://www.cs.unm.edu/~maccabe/OLD/tkisem/begin.html Project homepage]
* [http://www.comp.mq.edu.au/units/comp226/resources/tkisem.html Windows Binaries and lab manuals]
* [http://slackware.cs.utah.edu/pub/linuxpackages/Slackware-9.1/X11/tkisem Slackware Linux Binaries]
* [http://archive.debian.net/woody/tkisem Archived Debian sources]
* [http://old-releases.ubuntu.com/ubuntu/pool/universe/t/tkisem/ Archived Ubuntu sources]

[[Category:Sparc]] [[Category:Emulators]]
````

<a id="w003044"></a>

## W003044 — User talk:Anuaimi

Bron-ID: W003044
Namespace: User talk (3)
Revisie: 12163 | Datum: 2011-11-08T05:25:55Z
Bron: https://wiki.osdev.org/User_talk:Anuaimi
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AAnuaimi&oldid=12163

````text
== HFS+ ==

Care to elaborate a little more on your changes to the HFS+ article? --[[User:Brynet-inc|Brynet-inc]] 05:25, 8 November 2011 (UTC)
````

<a id="w003045"></a>

## W003045 — Sparc Hardware Docs

Bron-ID: W003045
Namespace: Main (0)
Revisie: 29128 | Datum: 2024-06-18T14:53:30Z
Bron: https://wiki.osdev.org/Sparc_Hardware_Docs
Revisie-URL: https://wiki.osdev.org/index.php?title=Sparc_Hardware_Docs&oldid=29128

````text
==Leon Sparc Documentation==
* [https://www.gaisler.com/index.php/downloads Leon Sparc Downloads]

==Historical Documentation for Sun Sparc Hardware==

* [http://wikis.sun.com/display/FOSSdocs/Home FOSS Open Hardware Documentation]
* [http://www.innovative-research.com/manuals/CPU-8VT_TechnicalGuide.pdf Manual for Force CPU-8VT describes a sun4m like machine]
* [http://www.sunhelp.org/tadpole/SPARCBook/ Good Manuals for the Sparcbook 3gx a SparcStation 5 clone (grab 3gx.pdf)]
==I/O Programming==
All early SparcStations used the Z8530 serial chip as well as the Ultra 1. Ultra 5 and 10 use the SAB82532.

As qemu emulates mostly early SparcStations at this point it may be best to start there.

Serial chips specific documentation

* [http://bitsavers.org/pdf/zilog/_dataBooks/00-2057-03_Z8030_Z-BUS_SCC_Z8530_SCC_Serial_Communications_Controller_Technical_Manual_Sep1986.pdf Scanned Zilog 8530 Technical Manual (15Mb pdf) ]
* [http://www.eit.lth.se/fileadmin/eit/courses/edi021/datablad/Periphery/Communication/z8530.pdf Newer Z8530 PDF ~1.3MB]


Note that on SparcStations, the Z8530 is controlled through a an Sbus to Ebus bridge via the slave I/O controller and is not programmed directly refer to the system manuals. EBus is a slow 8 bit peripheral bus.

The Slavio chip controls serial, nvram, TOD clock etc.
* [http://www.linux-mips.org/pub/linux/mips/doc/NCR/NCR89C105.txt Slave IO Documentation]
The Macio chip controls Onboard Ethernet, SCSI , and 
* [http://www.linux-mips.org/pub/linux/mips/doc/NCR/NCR89C100.txt Macio Documentatino]

==Note/TODO==
WANTED: "NCR SBus I/O Chipset Data Manual" as it should provide more complete documetation of these Macio/Slavio chips

The System technical manuals often include some detailed information about programming the perpherials the Force computer and 3gx manuals look to be good references.

QEMU by default emualtes a sparcstation 5 so a system almost exactly like the sparcbook 3 with the exception of the graphics card being different TCX or GX  Note that QEMU only emulates the TCX graphics adapter at the moment. The 8bit GX adapter is probably the most common on real hardware however with several variations some with more video memory.


[[Category:Sparc]]
````

<a id="w003046"></a>

## W003046 — User:Ahferroin7

Bron-ID: W003046
Namespace: User (2)
Revisie: 12190 | Datum: 2011-11-16T16:14:06Z
Bron: https://wiki.osdev.org/User:Ahferroin7
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAhferroin7&oldid=12190

````text
I will try to update this frequently; for more information, please contact (ahferroin7 AT gmail DOT com)

Note: As Ophiophagus is pre-pre-alpha at the moment, I am not currently looking for more developers
==Current Info==
* Will be implemented on top of a L4 Fiasco.OC microkernel + L4re
* Will be written in C with some Lua
* Will absolutely '''NOT''' be a monolithic kernel
* Will utilize FUSE for most fs drivers
* Will have a POSIX compatibility layer
* Will also have compatibility layers providing some of the more unique API's of some other UNIX derived OSes (eg Linux, 4.4 BSD, possibly SVR4, possibly others)
* General structure:
** Core system servers: the absolute minimum that must be running for a usable system
*** Memory manager (Initially Sigma0, will eventually be replaced when more of the other components are working)
*** Filesystem server (provides a uniform filesystem interface and unified fs hierarchy, as well as FUSE API, will include integrated drivers for FAT16/32, ISO 9660, and UDF, named anaconda)
*** UI manager (takes care of keyboard input/virtual terminal output, named copperhead)
*** Bus manager (acts as a monitor/controller for driver access to PCI/USB/Other buses, named boomslang)
*** Crash detector/debug server (attempts to restart other core servers if they crash, handles debugging of core servers, will eventually also be the init process, named mongoose)
** Additional system servers: provide other useful services, but not required to boot and run
*** Network manager (provides a uniform network interface so user programs don't have to know what type of networking the system has, named basilisk)
*** POSIX compatibility layer (Will manage most POSIX syscalls, named asp)
*** 4.4 BSD compatibility layer (Will only provide API's that are not in POSIX or native, named boa)
*** Linux compatibility layer (Will provide some otherwise Linux specific API's such as inotify, named cottonmouth)
*** CLR runtime server (Will fill the same function as Mono, named mamba)
* Will be developed in a hardware agnostic manner, so it should run on any architecture supported by L4 Fiasco.OC
* All drivers will run in usermode
* Each driver will see only the hardware which it needs (for example, a driver for an AGP graphics card will only see the graphics card which it is for)
* Ophiophagus will utilize a unified fs hierarchy; although it will utilize some slightly different conventions, most notably
** Root's home directory is /home/root not /root
** The system's initrd will be accessible read only in /boot/initrd
** /opt will be a symbolic link to /usr/local
** /srv will be a symbolic link to /var/services
** /var/www, /var/ftp and other service specific directories under /var will be symbolic links to identically named directories in /var/services
[[Category:In Progress|{{PAGENAME}}]]
````

<a id="w003047"></a>

## W003047 — Ophiophagus

Bron-ID: W003047
Namespace: Main (0)
Revisie: 12178 | Datum: 2011-11-11T10:58:57Z
Bron: https://wiki.osdev.org/Ophiophagus
Revisie-URL: https://wiki.osdev.org/index.php?title=Ophiophagus&oldid=12178
Doorverwijzing: User:Ahferroin7

````text
#REDIRECT [[User:Ahferroin7]]
````

<a id="w003048"></a>

## W003048 — Template:Emulators

Bron-ID: W003048
Namespace: Template (10)
Revisie: 14612 | Datum: 2013-04-30T11:31:40Z
Bron: https://wiki.osdev.org/Template:Emulators
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AEmulators&oldid=14612

````text
{{SmallNavBox
| width:200px;
|Name=[[Emulators|Emulators]]

|Group0=PC Emulators
|List0=<div>
[[Bochs]]<br />
[[QEMU]]<br />
</div>

|Group1=PC Virtual Machine Monitors
|List1=<div>
[[KVM]]<br />
[[Microsoft Virtual PC|Virtual PC]]<br />
[[VirtualBox]]<br />
[[VMware]]<br />
</div>

|Group2=PowerPC Emulators
|List2=<div>
[[PearPC]]<br />
</div>
}}
````

<a id="w003049"></a>

## W003049 — User:Solar/Makefile

Bron-ID: W003049
Namespace: User (2)
Revisie: 12959 | Datum: 2012-03-10T07:38:32Z
Bron: https://wiki.osdev.org/User:Solar/Makefile
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASolar%2FMakefile&oldid=12959

````text
'''Note:''' This isn't finished, just a rough draft. I didn't test this exhaustively, and will probably not do so anytime soon. But what's there already might be helpful.

----
----
----

Sometimes, your project consists of more than one binary: You have to build several libraries and executables from a single source tree, but with different compiler / linker settings. This, too, can be comfortably covered with a single Makefile, but requires some more involved syntax. One example is presented here; there are, of course, many other ways to achieve this.

The basic idea is to create a subdirectory for each module (executable or library). In each these module subdirectories, the overall structure is identical:

* module-specific settings are stored in a file <tt>build.mk</tt>;
* headers with "global" project visibility (i.e., visible to other modules) are stored in <tt>includes</tt>;
* all other headers and source files are located in <tt>src</tt>;
* a directory <tt>obj</tt> exists for the compiler to put its object files.

At the top level, there is a directory <tt>bin</tt> and a directory <tt>lib</tt> for binaries and libraries, respectively. A directory <tt>includes</tt> will eventually contain links to all the "global" headers. And, of course, there is our <tt>Makefile</tt>.

<pre>
.
├── bar
│   ├── build.mk
│   ├── includes
│   │   └── global.h
│   ├── obj
│   └── src
│       ├── interface.c
│       ├── interface.h
│       └── main.c
├── baz
│   ├── build.mk
│   ├── includes
│   │   └── global.h
│   ├── obj
│   └── src
│       ├── hashmap.c
│       ├── hashmap.h
│       └── algorithm.c
├── bin
├── includes
├── lib
└── Makefile
</pre>

The contents of the individual modules' <tt>build.mk</tt> files is similar to what is done in <tt>Makefile.am</tt> for Automake.

Example for a binary module:

<pre>
# CFLAGS for module 'bar'
CFLAGS_bar := 

# Executable to build in module 'bar'
bar_PROGRAM := bar

# Libraries that the executable depends on:
bar_LIBRARIES := libbaz.a

# Sources for the executable 'bar' (without headers)
bar_SOURCES := interface.c main.c
</pre>

Example for a library module:

<pre>
# CFLAGS for module 'baz'
CFLAGS_baz :=

# Archive / Lib to build in module 'baz'
baz_ARCHIVE := libbaz.a

# Sources for the archive / lib 'libbaz.a' (without headers)
libbaz.a_SOURCES := hashmap.c algorithm.c
</pre>

The main Makefile makes heavy use of GNU make's template functionality. It might look daunting at first, but if you look at it carefully, you will see it is pretty straightforward, and easily adaptable or expandable.

<!-- No, this isn't bash. No, the syntax highlighter doesn't know "make". Let's have at least a LITTLE bit of color. -->
<source lang="bash">
# Modules in the project (you could 'find' these, but stating
# them explicitly allows for subdirectories like 'tmp' or 'doc'
# without upsetting the build process.
MODULES := bar baz

# Global CFLAGS. Add to them if you must, but don't remove '-MMD -I includes',
# which is used for header dependency tracking.
CFLAGS_global := -MMD -I includes
# Global ARFLAGS.
ARFLAGS := rc

.PHONY: clean mrproper

# Add whatever should be your default / global target.
all:
	@echo "Default target."

###################################################################
# What follows are several templates (think "functions"), which are
# later instantiated for each registered module ($(1) being the
# module name).
###################################################################

# Including a module's build.mk
define MK_template
include $(1)/build.mk
endef

# Setting a module's build rules for object files in <module>/obj.
define RULES_template
$(1)/obj/%.o: $(1)/src/%.c
	$$(CC) $$(CFLAGS) $$(CFLAGS_global) $$(CFLAGS_$(1)) -c $$< -o $$@
endef

# Setting a module's build rules for executable targets.
# (Depending on its sources' object files and any libraries.)
# Also adds a module's dependency files to the global list.
define PROGRAM_template
DEPENDENCIES := $(DEPENDENCIES) $(patsubst %,$(2)/obj/%.d,$(basename $($(1)_SOURCES)))
bin/$(1): $(patsubst %,$(2)/obj/%.o,$(basename $($(1)_SOURCES))) $(foreach library,$($(1)_LIBRARIES),lib/$(library))
	$$(LD) $$(LDFLAGS) $$(LDFLAGS_$(2)) $$^ -o $$@
endef

# Setting a module's build rules for archive targets.
# (Depending on its sources' object files.)
define ARCHIVE_template
DEPENDENCIES := $(DEPENDENCIES) $(patsubst %,$(2)/obj/%.d,$(basename $($(1)_SOURCES)))
lib/$(1): $(patsubst %,$(2)/obj/%.o,$(basename $($(1)_SOURCES)))
	$$(AR) $$(ARFLAGS) $$@ $$?
endef

# Linking a module's global includes into the global include directory
# (where they will be available as <module>/filename.h).
define INCLUDE_template
ifeq ($(wildcard includes/$(1)),)
    $$(shell ln -s ../$(1)/includes includes/$(1))
endif
endef

# Now, instantiating the templates for each module.
$(foreach module,$(MODULES),$(eval include $(module)/build.mk))
$(foreach module,$(MODULES),$(eval $(call RULES_template,$(module))))
$(foreach module,$(MODULES),$(eval $(foreach binary,$($(module)_PROGRAM),$(call PROGRAM_template,$(binary),$(module)))))
$(foreach module,$(MODULES),$(eval $(foreach library,$($(module)_ARCHIVE),$(call ARCHIVE_template,$(library),$(module)))))
$(foreach module,$(MODULES),$(eval $(call INCLUDE_template,$(module))))

# Include the dependency files (generated by GCC's -MMD option)
-include $(sort $(DEPENDENCIES))

clean:
	$(RM) $(foreach mod,$(MODULES),$(mod)/obj/*.o)

mrproper: clean
	$(RM) $(foreach mod,$(MODULES),$(mod)/obj/*.d) includes/* bin/* lib/*
</source>
````

<a id="w003050"></a>

## W003050 — Boot sequence

Bron-ID: W003050
Namespace: Main (0)
Revisie: 12256 | Datum: 2011-12-12T10:34:25Z
Bron: https://wiki.osdev.org/Boot_sequence
Revisie-URL: https://wiki.osdev.org/index.php?title=Boot_sequence&oldid=12256
Doorverwijzing: Boot Sequence

````text
#REDIRECT [[Boot Sequence]]
````

<a id="w003051"></a>

## W003051 — Talk:Boot sequence

Bron-ID: W003051
Namespace: Talk (1)
Revisie: 12258 | Datum: 2011-12-12T10:34:25Z
Bron: https://wiki.osdev.org/Talk:Boot_sequence
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABoot_sequence&oldid=12258
Doorverwijzing: Talk:Boot Sequence

````text
#REDIRECT [[Talk:Boot Sequence]]
````

<a id="w003052"></a>

## W003052 — Diskless booting

Bron-ID: W003052
Namespace: Main (0)
Revisie: 12265 | Datum: 2011-12-12T10:39:30Z
Bron: https://wiki.osdev.org/Diskless_booting
Revisie-URL: https://wiki.osdev.org/index.php?title=Diskless_booting&oldid=12265
Doorverwijzing: Diskless Booting

````text
#REDIRECT [[Diskless Booting]]
````

<a id="w003053"></a>

## W003053 — BIOS (PC)

Bron-ID: W003053
Namespace: Main (0)
Revisie: 12288 | Datum: 2011-12-12T14:36:19Z
Bron: https://wiki.osdev.org/BIOS_(PC)
Revisie-URL: https://wiki.osdev.org/index.php?title=BIOS_%28PC%29&oldid=12288
Doorverwijzing: BIOS

````text
#REDIRECT [[BIOS]]
````

<a id="w003054"></a>

## W003054 — Talk:BIOS (PC)

Bron-ID: W003054
Namespace: Talk (1)
Revisie: 12290 | Datum: 2011-12-12T14:36:20Z
Bron: https://wiki.osdev.org/Talk:BIOS_(PC)
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABIOS_%28PC%29&oldid=12290
Doorverwijzing: Talk:BIOS

````text
#REDIRECT [[Talk:BIOS]]
````

<a id="w003055"></a>

## W003055 — Pascal Bare Bones

Bron-ID: W003055
Namespace: Main (0)
Revisie: 30608 | Datum: 2026-05-22T14:06:01Z
Bron: https://wiki.osdev.org/Pascal_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Pascal_Bare_Bones&oldid=30608

````text
{{BadPractice}}
{{BeginnersWarning}}
{{Rating|2}}

Tools needed to build the project:

* FPC (You'll need the i386 version as that's the architecture we're targeting)
* NASM
* binutils(ld) built for [[GCC Cross-Compiler|32-bit elf support]]

=== stub.asm ===
<syntaxhighlight lang="nasm">
;/////////////////////////////////////////////////////////
;//                                                     //
;//               Freepascal barebone OS                //
;//                      stub.asm                       //
;//                                                     //
;/////////////////////////////////////////////////////////
;//
;//     By:             De Deyn Kim <kimdedeyn@skynet.be>
;//     License:        Public domain
;//

;
; Kernel stub
;

;
; We are in 32bits protected mode
;
[bits 32]

;
; Export entrypoint
;
[global kstart]

;
; Import kernel entrypoint
;
[extern kmain]

;
; Posible multiboot header flags
;
MULTIBOOT_MODULE_ALIGN          equ     1<<0
MULTIBOOT_MEMORY_MAP            equ     1<<1
MULTIBOOT_GRAPHICS_FIELDS       equ     1<<2
MULTIBOOT_ADDRESS_FIELDS        equ     1<<16

;
; Multiboot header defines
;
MULTIBOOT_HEADER_MAGIC          equ     0x1BADB002
MULTIBOOT_HEADER_FLAGS          equ     MULTIBOOT_MODULE_ALIGN | MULTIBOOT_MEMORY_MAP
MULTIBOOT_HEADER_CHECKSUM       equ     -(MULTIBOOT_HEADER_MAGIC + MULTIBOOT_HEADER_FLAGS)

;
; Kernel stack size
;
KERNEL_STACKSIZE                equ     0x4000

section .text

;
; Multiboot header
;
align 4
dd MULTIBOOT_HEADER_MAGIC
dd MULTIBOOT_HEADER_FLAGS
dd MULTIBOOT_HEADER_CHECKSUM

;
; Entrypoint
;
kstart:
        mov esp, KERNEL_STACK+KERNEL_STACKSIZE  ;Create kernel stack
        push eax                                ;Multiboot magic number
        push ebx                                ;Multiboot info
        call kmain                              ;Call kernel entrypoint
        cli                                     ;Clear interrupts
        hlt                                     ;Halt machine

section .bss

;
; Kernel stack location
;
align 32
KERNEL_STACK:
        resb KERNEL_STACKSIZE
</syntaxhighlight>

=== kernel.pas ===
<syntaxhighlight lang="pascal">
{
/////////////////////////////////////////////////////////
//                                                     //
//               Freepascal barebone OS                //
//                      kernel.pas                     //
//                                                     //
/////////////////////////////////////////////////////////
//
//      By:             De Deyn Kim <kimdedeyn@skynet.be>
//      License:        Public domain
//
}

unit kernel;

interface

uses
        multiboot,
        console;

procedure kmain(mbinfo: Pmultiboot_info_t; mbmagic: DWORD); stdcall;

implementation

procedure kmain(mbinfo: Pmultiboot_info_t; mbmagic: DWORD); stdcall; [public, alias: 'kmain'];
begin
        kclearscreen();
        kwritestr('Freepascal barebone OS booted!');
        xpos := 0;
        ypos += 1;

        if (mbmagic <> MULTIBOOT_BOOTLOADER_MAGIC) then
        begin
                kwritestr('Halting system, a multiboot-compliant boot loader needed!');
                asm
                        cli
                        hlt
                end;
        end
        else
        begin
                kwritestr('Booted by a multiboot-compliant boot loader!');
                xpos := 0;
                ypos += 2;
                kwritestr('Multiboot information:');
                xpos := 0;
                ypos += 2;
                kwritestr('                       Lower memory  = ');
                kwriteint(mbinfo^.mem_lower);
                kwritestr('KB');
                xpos := 0;
                ypos += 1;
                kwritestr('                       Higher memory = ');
                kwriteint(mbinfo^.mem_upper);
                kwritestr('KB');
                xpos := 0;
                ypos += 1;
                kwritestr('                       Total memory  = ');
                kwriteint(((mbinfo^.mem_upper + 1000) div 1024) +1);
                kwritestr('MB');
        end;

        asm
                @loop:
                jmp @loop
        end;
end;

end.
</syntaxhighlight>

=== console.pas ===
<syntaxhighlight lang="pascal">
{
/////////////////////////////////////////////////////////
//                                                     //
//               Freepascal barebone OS                //
//                       console.pas                   //
//                                                     //
/////////////////////////////////////////////////////////
//
//      By:             De Deyn Kim <kimdedeyn@skynet.be>
//      License:        Public domain
//
}

unit console;

interface

var
        xpos: Integer = 0;
        ypos: Integer = 0;

procedure kclearscreen();
procedure kwritechr(c: Char);
procedure kwritestr(s: PChar);
procedure kwriteint(i: Integer);
procedure kwritedword(i: DWORD);

implementation

var
        vidmem: PChar = PChar($b8000);

procedure kclearscreen(); [public, alias: 'kclearscreen'];
var
        i: Integer;
begin
        for i := 0 to 3999 do
                vidmem[i] := #0;
end;

procedure kwritechr(c: Char); [public, alias: 'kwritechr'];
var
        offset: Integer;
begin
        if (ypos > 24) then
                ypos := 0;

        if (xpos > 79) then
                xpos := 0;

        offset := (xpos shl 1) + (ypos * 160);
        vidmem[offset] := c;
        offset += 1;
        vidmem[offset] := #7;
        offset += 1;

        xpos := (offset mod 160);
        ypos := (offset - xpos) div 160;
        xpos := xpos shr 1;
end;

procedure kwritestr(s: PChar); [public, alias: 'kwritestr'];
var
        offset, i: Integer;
begin
        if (ypos > 24) then
                ypos := 0;

        if (xpos > 79) then
                xpos := 0;

        offset := (xpos shl 1) + (ypos * 160);
        i := 0;

        while (s[i] <> Char($0)) do
        begin
                vidmem[offset] := s[i];
                offset += 1;
                vidmem[offset] := #7;
                offset += 1;
                i += 1;
        end;

        xpos := (offset mod 160);
        ypos := (offset - xpos) div 160;
        xpos := xpos shr 1;
end;

procedure kwriteint(i: Integer); [public, alias: 'kwriteint'];
var
        buffer: array [0..11] of Char;
        str: PChar;
        digit: DWORD;
        minus: Boolean;
begin
        str := @buffer[11];
        str^ := #0;

        if (i < 0) then
        begin
                digit := -i;
                minus := True;
        end
        else
        begin
                digit := i;
                minus := False;
        end;

        repeat
                Dec(str);
                str^ := Char((digit mod 10) + Byte('0'));
                digit := digit div 10;
        until (digit = 0);

        if (minus) then
        begin
                Dec(str);
                str^ := '-';
        end;

        kwritestr(str);
end;

procedure kwritedword(i: DWORD); [public, alias: 'kwritedword'];
var
        buffer: array [0..11] of Char;
        str: PChar;
        digit: DWORD;
begin
        for digit := 0 to 10 do
                buffer[digit] := '0';

        str := @buffer[11];
        str^ := #0;

        digit := i;
        repeat
                Dec(str);
                str^ := Char((digit mod 10) + Byte('0'));
                digit := digit div 10;
        until (digit = 0);

        kwritestr(str);
end;

end.
</syntaxhighlight>

=== multiboot.pas ===
<syntaxhighlight lang="pascal">
unit multiboot;

interface

const
        KERNEL_STACKSIZE = $4000;

        MULTIBOOT_BOOTLOADER_MAGIC = $2BADB002;

type
        Pelf_section_header_table_t = ^elf_section_header_table_t;
        elf_section_header_table_t = packed record
          num: DWORD;
          size: DWORD;
          addr: DWORD;
          shndx: DWORD;
        end;

        Pmultiboot_info_t = ^multiboot_info_t;
        multiboot_info_t = packed record
          flags: DWORD;
          mem_lower: DWORD; { Amount of memory available below 1mb }
          mem_upper: DWORD; { Amount of memory available above 1mb }
          boot_device: DWORD;
          cmdline: DWORD;
          mods_count: DWORD;
          mods_addr: DWORD;
          elf_sec: elf_section_header_table_t;
          mmap_length: DWORD;
          mmap_addr: DWORD;
        end;

        Pmodule_t = ^module_t;
        module_t = packed record
          mod_start: DWORD;
          mod_end: DWORD;
          name: DWORD;
          reserved: DWORD;
        end;

        Pmemory_map_t = ^memory_map_t;
        memory_map_t = packed record
          size: DWORD;
          { You can declare these two as a single qword if your compiler supports it }
          base_addr_low: DWORD;
          base_addr_high: DWORD;
          { And again, these can be made into one qword variable. }
          length_low: DWORD;
          length_high: DWORD;
          mtype: DWORD;
        end;

implementation

end.
</syntaxhighlight>

=== system.pas ===
Since fpc-3.2.0 there was added necessary parts (in code block that part was highlighted by comment lines), because without them developer could met error like:
<syntaxhighlight lang="bash">
system.pas(18,1) (system)   Parsing implementation of SYSTEM
system.pas(18,1) Fatal: Internal type "TEXCEPTADDR" was not found. Check if you use the correct run time library.
Fatal: Compilation aborted
</syntaxhighlight>
Valid version of '''system.pas''' on 2022/12/29:
<syntaxhighlight lang="pascal">
unit system;

{$MODE FPC}

interface

  type
    cardinal = 0..$FFFFFFFF;
    hresult = cardinal;
    dword = cardinal;
    integer = longint;

    pchar = ^char;

    { That part comes from fpc 3.2.0 as nessesary}
    TTypeKind = (tkUnknown, tkInteger, tkChar, tkEnumeration, tkFloat, tkSet,
    tkMethod, tkSString, tkLString, tkAString, tkWString, tkVariant, tkArray,
    tkRecord, tkInterface, tkClass, tkObject, tkWChar, tkBool, tkInt64, tkQWord,
    tkDynArray, tkInterfaceRaw, tkProcVar, tkUString, tkUChar, tkHelper, tkFile,
    tkClassRef, tkPointer);

    jmp_buf = packed record
      rbx, rbp, r12, r13, r14, r15, rsp, rip: QWord;
      {$IFDEF win64}
      rsi, rdi: QWord;
      xmm6, xmm7, xmm8, xmm9, xmm10, xmm11, xmm12, xmm13, xmm14, xmm15: record 
        m1, m2: QWord;
      end;

      mxcsr: LongWord;
      fpucw: word;
      padding: word;
      {$ENDIF win64}
    end;

    Pjmp_buf = ^jmp_buf;
    PExceptAddr = ^TExceptAddr;
    TExceptAddr = record 
      buf: Pjmp_buf;
      next: PExceptAddr;
      {$IFDEF CPU16}
      frametype: SmallInt;
      {$ELSE CPU16}
      frametype: LongInt;
      {$ENDIF CPU16}
    end;

    PGuid = ^TGuid;
    TGuid = packed record
      case Integer of
      1:
      (Data1: DWord;
        Data2: word;
        Data3: word;
        Data4: array [0 .. 7] of byte;
      );
      2:
      (D1: DWord;
        D2: word;
        D3: word;
        D4: array [0 .. 7] of byte;
      );
      3:
      ( { uuid fields according to RFC4122 }
        time_low: DWord; // The low field of the timestamp
        time_mid: word; // The middle field of the timestamp
        time_hi_and_version: word;
        // The high field of the timestamp multiplexed with the version number
        clock_seq_hi_and_reserved: byte;
        // The high field of the clock sequence multiplexed with the variant
        clock_seq_low: byte; // The low field of the clock sequence
        node: array [0 .. 5] of byte; // The spatially unique node identifier
      );
    end;

{ --- End of nessesary part --- }

implementation

end.
</syntaxhighlight>

=== Linker script ===
linker.script
<syntaxhighlight lang="c">
ENTRY(kstart)
SECTIONS
{
  .text  0x100000 :
  {
    text = .; _text = .; __text = .;
    *(.text)
    . = ALIGN(4096);
  }
  .data  :
  {
    data = .; _data = .; __data = .;
    *(.data)
    kimage_text = .;
    LONG(text);
    kimage_data = .;
    LONG(data);
    kimage_bss = .;
    LONG(bss);
    kimage_end = .;
    LONG(end);
    . = ALIGN(4096);
  }
  .bss  :
  {
    bss = .; _bss = .; __bss = .;
    *(.bss)
    . = ALIGN(4096);
  }
  end = .; _end = .; __end = .;
}
</syntaxhighlight>

=== Compiling and Linking the modules ===
Assemble stub.asm with:
<syntaxhighlight lang="bash">nasm -f elf32 stub.asm -o stub.o</syntaxhighlight>

* -f elf32 - needed ''exact'' under x86_64 systems to make correct object file

The Pascal modules with:
<syntaxhighlight lang="bash">fpc -Aelf -n -O3 -Op3 -Si -Sc -Sg -Xd -CX -XXs -Pi386 -Rintel -Tlinux kernel.pas</syntaxhighlight>

* -Aelf - instructs the internal fpc assembler to output an ELF object.;
* -n - ignores fpc.cfg;
* -O3 - perform level 3 optimizations;
* -Op3 - tune code for PentiumPro / P-II / Cyrix 6x86 / K6 (TM);
* -Si - enable C++ style INLINE keyword;
* -Sc - enable C style operators;
* -Sg - enable support for labels;
* -Xd - tells the compiler to forget the standard library path;
* -CX - tells the compiler to create smartlinkable units
* -XXs - tells the compiler to do smartlinking (-XX) and debugging symbols stripping (-Xs)
* -Pi386 - tells the compiler to force i386 mode (actual for x86_64 host systems)
* -Rintel - sets the inline assembly syntax to intel style;
* -Tlinux - specifies that the target operating system is Linux. (Provides a sensible system unit to use)

Then link the whole thing with:
<syntaxhighlight lang="bash">i386-elf-ld --gc-sections -s -Tlinker.script -o kernel.obj stub.o kernel.o multiboot.o system.o console.o</syntaxhighlight>

* --gc-sections -s, in combination with -CX -XXs above, eliminates RTTI symbols from resulting binary

In case of trouble linking under x86_64 system try this line:
<syntaxhighlight lang="bash">i686-elf-ld -A elf-386 --gc-sections -s -Tlinker.script -o kernel.obj stub.o kernel.o multiboot.o system.o console.o</syntaxhighlight>

'''Special situation''': last time after building '''binutils''' I've got error
<pre>
i386-linux-ld --gc-sections -s -Tlinker.script -o kernel.obj stub.o kernel.o multiboot.o console.o system.o
ld: i386 architecture of input file `stub.o' is incompatible with i386:x86-64 output
ld: i386 architecture of input file `kernel.o' is incompatible with i386:x86-64 output
ld: i386 architecture of input file `multiboot.o' is incompatible with i386:x86-64 output
ld: i386 architecture of input file `console.o' is incompatible with i386:x86-64 output
ld: i386 architecture of input file `system.o' is incompatible with i386:x86-64 output
make: *** [Makefile:27: _LD] Error 1
</pre>
That type of error can encounter if ''i386-linux-ld'' have wrong content. In my situation file content edited to:
<syntaxhighlight lang="bash">
#!/bin/bash
/full/path/to/compiled/cross/binutils/ld-new -A elf32-i386 $@
</syntaxhighlight>

=== makeiso.sh ===
Also a good option is create a bootable ISO file to make run it with VirtualBox or qemu:

<syntaxhighlight lang="bash">
#!/bin/sh
TMPISO=iso
TMPBOOT=${TMPISO}/boot
TMPGRUB=${TMPBOOT}/grub
TMPCFG=${TMPGRUB}/grub.cfg

mkdir $TMPISO
mkdir $TMPBOOT
mkdir $TMPGRUB
cp kernel.obj $TMPBOOT/kernel.obj
echo 'set timeout=0'                  > $TMPCFG
echo 'set default =0'                >> $TMPCFG
echo ''                              >> $TMPCFG
echo 'menuentry "Pascal bare" {'     >> $TMPCFG
echo '  multiboot /boot/kernel.obj'  >> $TMPCFG
echo '  boot'                        >> $TMPCFG
echo '}'                             >> $TMPCFG
grub-mkrescue --output=pascal-kernel.iso iso
rm -rf $TMPISO
</syntaxhighlight>

And simple run:
<syntaxhighlight lang="bash">qemu-system-i386 pascal-kernel.iso</syntaxhighlight>

=== Alternative compiling: Makefile ===

Accumulating the lines from previous part we can make a '''Makefile''':
<syntaxhighlight lang="make">
# Freepascal BareboneOS 
# Makefile 
# 			2019
# by:		furaidi <iluatitok@gmail.com>
# License:	Public domain

NASMPARAMS = -f elf32 -o stub.o
LDPARAMS = -A elf32-i386 --gc-sections -s -Tlinker.script -o kernel.obj
FPCPARAMS = -Aelf -n -O2 -Op3 -Si -Sc -Sg -Xd -CX -XXs -Pi386 -Rintel -Tlinux
TMPISO = iso
TMPBOOT = $(TMPISO)/boot
TMPGRUB = $(TMPBOOT)/grub
TMPCFG  = $(TMPGRUB)/grub.cfg

objects = stub.o kernel.o multiboot.o console.o system.o

_FPC:
	@echo 'Compile kernel'
	fpc $(FPCPARAMS) kernel.pas

_NASM:
	@echo 'Compile stub'
	nasm $(NASMPARAMS) stub.asm

_LD:
	@echo 'Link them together'
	i386-linux-ld $(LDPARAMS) $(objects)

all: _FPC _NASM _LD

install:
	mkdir $(TMPISO)
	mkdir $(TMPBOOT)
	mkdir $(TMPGRUB)
	cp kernel.obj $(TMPBOOT)/kernel.obj
	echo 'set timeout=0'     		 > $(TMPCFG)
	echo 'set default =0'		        >> $(TMPCFG)
	echo ''                      		>> $(TMPCFG)
	echo 'menuentry "Pascal Bare" {'	>> $(TMPCFG)
	echo '  multiboot /boot/kernel.obj'  	>> $(TMPCFG)
	echo '  boot'              		>> $(TMPCFG)
	echo '}'                      		>> $(TMPCFG)
	grub-mkrescue --output=pascal-kernel.iso $(TMPISO)
	rm -rf $(TMPISO)

clean:
	rm -rf $(TMPISO)
	rm -f *.o
	rm -f *.ppu

</syntaxhighlight>

==Further Steps==
After when bare bones are ready and tested, you'd probably want to go with a few ways separately or combine:
* Add terminal workaround. Pure *nix way like changing terminals, built-in commands, simple user management
* Add graphic interface from start. That is Windows NT family way. You need to add SVGA-like drivers and/or mouse input and everything you need (or thinks so)
Also you may want to adapt *nix libraries such as '''binutils''', make build target for compiling your fpc programs for your new os like ''i386-myos''. From that point you are on your own.

==See Also==
===External Links===
* https://github.com/torokernel/torokernel/wiki/Writing-Drivers-In-Freepascal, notes about how to write drivers by using the Frepascal language.
* https://wiki.freepascal.org/System_unit, a tutorial to write a simple system unit

[[Category:Bare bones tutorials]]
[[Category:Pascal]]
[[Category:Inadvisable]]
````

<a id="w003056"></a>

## W003056 — Talk:TCC

Bron-ID: W003056
Namespace: Talk (1)
Revisie: 12428 | Datum: 2012-01-16T12:03:59Z
Bron: https://wiki.osdev.org/Talk:TCC
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ATCC&oldid=12428

````text
== How is this a FASM/TCC barebones article? ==
"Fasm-TCC BareBones" redirects here, but the reader is just presented with a description of TCC. I was expecting to find something similar to [[Bare Bones]] but oriented to FASM and TCC. --[[User:Messiahandrw|MessiahAndrw]] 21:45, 15 January 2012 (CST)

:There shouldn't be a seperate Bare Bones for every combination of assembler / compiler. I would have deleted the page outright (since there was very little information on FASM included), but felt that TCC has some intrinsic value (compared to the heavyweight GCC), so I left that half of the page as information on that noteable.
:I will remove the redirect next thing; I agree that is confusing.
:We have, in [[Bare Bones]] and [[C++ Bare Bones]], sections for both GAS syntax and NASM syntax. If you feel that FASM is both important ''and'' different enough to be added to that, feel free to do so. What little information there was about FASM can be found [http://wiki.osdev.org/index.php?title=TCC&oldid=12310 in this version of the article]; it should be adapted so that it resembles the other two source snippets as closely as they resemble each other. (Structure, comments, etc.) -- [[User:Solar|Solar]] 06:01, 16 January 2012 (CST)

== Should we keep this? ==

This page was created in August 2009 by user Tommy, and never edited again. Right now I don't see how it adds much of value over [[Bare Bones]]. I'd rather have the FASM listing integrated in [[Bare Bones]], and the information on TCC on a seperate page (possibly with a cross-compiler / porting tutorial, if it is so easy on dependencies?). Any opinions? -- [[User:Solar|Solar]] 14:58, 12 December 2011 (UTC)
````

<a id="w003057"></a>

## W003057 — Tar

Bron-ID: W003057
Namespace: Main (0)
Revisie: 28704 | Datum: 2024-03-17T18:03:23Z
Bron: https://wiki.osdev.org/Tar
Revisie-URL: https://wiki.osdev.org/index.php?title=Tar&oldid=28704

````text
Note: This page describes a minimalist subset of the standard [[USTAR]] archive format. It can hold only plain files and directories, and pathname length is limited to 99 characters. USTAR is backwards-compatible with this format if these limits are respected.

Tar is a very simple file format and is well suited for things like ramdisks that you can tell Grub to load for you together with your kernel in the early boot stage by supplying the keyword module to your menu.lst file. Grub will load your ramdisk to some address in memory and to find this address you will have to look in the multiboot header. Once you have found your ramdisk you will need to parse it in order to extract the files within.

== Parsing ==

Tar works by concatenating all files together in one big file where each file gets a header that is padded up to 512 bytes. The headers are each aligned on a 512 byte boundry and the content follows directly afterwards. This is what a tar header in it's simplest form could look like.

<source lang="c">
struct tar_header
{
    char filename[100];
    char mode[8];
    char uid[8];
    char gid[8];
    char size[12];
    char mtime[12];
    char chksum[8];
    char typeflag[1];
};
</source>

As you can see the header is not 512 bytes big so the rest of the header is padded up with zeros. At the end of the tar file there is a header of 512 bytes filled with only zeros to indicate the end. (There may be more empty headers to pad to the tape block size; ustar format specifies 10KiB blocks.)  A header contains a lot of information but for this tutorial we'll only be interested in two of them.

Filename: This actually contains the whole path, not only the file. It ends with the '\0' character. If 100 bytes is not enough, the ustar format supports longer names by a more complicated mechanism not covered here.  Caution: paths of 100 bytes or longer may not be terminated with '\0'.

Size: This contains the size of the file. Important to note here is that the size is in base 8 and written using ascii characters so when you try to figure out the size you must keep this in mind. Actually all fields containing numbers uses this not just size. This is a code snippet that calculates this for you:

<source lang="c">
unsigned int getsize(const char *in)
{

    unsigned int size = 0;
    unsigned int j;
    unsigned int count = 1;

    for (j = 11; j > 0; j--, count *= 8)
        size += ((in[j - 1] - '0') * count);

    return size;

}
</source>

To parse a tar file is not very hard. Start by creating an array big enough to hold all the files your ramdisk will have. Each entry should contain a pointer to structs of type struct tar_header like this:

<source lang="c">
struct tar_header *headers[32];
</source>

A better way would of course be to allocate this dynamically by using malloc() but for the rest of the tutorial I assume you've done like I did.

Starting from the address of the first header loop through them all by reading the size of the current header and the next header should be located at the nearest boundry after the current header's address + 512 + the file size, rounded up to the next 512-byte boundary. Stop when you hit a header that is zero. Here is a simple example that does this and returns the number of files found:

<source lang="c">
unsigned int parse(unsigned int address)
{

    unsigned int i;

    for (i = 0; ; i++)
    {

        struct tar_header *header = (struct tar_header *)address;

        if (header->filename[i] == '\0')
            break;

        unsigned int size = getsize(header->size);

        headers[i] = header;

        address += ((size / 512) + 1) * 512;

        if (size % 512)
            address += 512;

    }

    return i;

}
</source>

Now you have all the headers in your array. To get the actual content you only need to get the value of the pointer and add 512.

It is very easy to use this header information when creating nodes in your virtual filesystem. Remember that the filename contains the whole path so you might need to chop that off when creating your nodes.

[[Category:File Formats]]
[[Category:Tutorials]]
````

<a id="w003058"></a>

## W003058 — FreeBasic Barebones

Bron-ID: W003058
Namespace: Main (0)
Revisie: 12309 | Datum: 2011-12-13T06:05:51Z
Bron: https://wiki.osdev.org/FreeBasic_Barebones
Revisie-URL: https://wiki.osdev.org/index.php?title=FreeBasic_Barebones&oldid=12309
Doorverwijzing: FreeBasic Bare Bones

````text
#REDIRECT [[FreeBasic Bare Bones]]
````

<a id="w003060"></a>

## W003060 — Talk:Fasm-TCC BareBones

Bron-ID: W003060
Namespace: Talk (1)
Revisie: 12313 | Datum: 2011-12-13T06:07:12Z
Bron: https://wiki.osdev.org/Talk:Fasm-TCC_BareBones
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AFasm-TCC_BareBones&oldid=12313
Doorverwijzing: Talk:TCC

````text
#REDIRECT [[Talk:TCC]]
````

<a id="w003061"></a>

## W003061 — User:Berkus/AffordableDevelopmentPlatforms

Bron-ID: W003061
Namespace: User (2)
Revisie: 12319 | Datum: 2011-12-13T07:25:32Z
Bron: https://wiki.osdev.org/User:Berkus/AffordableDevelopmentPlatforms
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ABerkus%2FAffordableDevelopmentPlatforms&oldid=12319

````text
This is a (non-comprehensive) list of affordable development platforms, split by architecture. Links are provided where possible to the wiki tutorials for each system.

= ARM =

== GameBoy Advance ==
== Beagleboard ==
== Pandaboard ==
== Raspberry Pi ==
 (not exactly affordable, while costing $25 it is not out yet)

= MIPS =
== Ainovo MIPS tablet ==
== D-Link DAP-1350 ==

= SH4 =
== Sega Dreamcast ==
````

<a id="w003062"></a>

## W003062 — Crt0

Bron-ID: W003062
Namespace: Main (0)
Revisie: 14158 | Datum: 2012-12-19T14:02:11Z
Bron: https://wiki.osdev.org/Crt0
Revisie-URL: https://wiki.osdev.org/index.php?title=Crt0&oldid=14158
Doorverwijzing: Creating a C Library

````text
#REDIRECT [[Creating_a_C_Library#Program_Initialization]]
````

<a id="w003063"></a>

## W003063 — Symbol Table

Bron-ID: W003063
Namespace: Main (0)
Revisie: 30541 | Datum: 2026-05-01T15:30:56Z
Bron: https://wiki.osdev.org/Symbol_Table
Revisie-URL: https://wiki.osdev.org/index.php?title=Symbol_Table&oldid=30541

````text
{{FirstPerson}}

A symbol table of a program is basically just a list containing the program's symbols (function names, global variables, etc) and the corresponding addresses of these symbols. The reason to use a symbol table for your kernel can vary for every OSDever, but it is especially useful when loading kernel modules, because a module can be loaded ANYWHERE and it should know how to call the kernel (cleanly, not with dirty hardcode).

== Extracting the Debug Symbol Table ==

If you want for example to get ALL the global symbols in the Kernel for debugging purposes, just use the <i>nm</i> utility as shown here.

<pre>
$ nm mykernel.bin > mykernel.map
</pre>

Important to note here that I'm assuming you are using ELF as the binary format with the symbol table section not stripped out. The resulting file contains three columns. The first is the address of the symbol, the second is what kind of symbol it is and the third is the name of the symbol.

== The Modulation Problem ==

I will explain it from bottom. If you have a Modular and/or Microkernel, you must attach to a rule if you don't want to lose the modulation benefits. That simple rule is to have every single needed module (only needed ones) loaded at run-time. Did you notice the problems, no? Well, I will list them:
* That run-time modules include the PCI IDE driver, the driver corresponding to the filesystem in the root mount, and all the driver for all the ACPI says to exist. Some of this drivers are required to access a Massive Storage Unit, where the other modules are... Solution: A initrd.
* If the modules are loaded at dynamically selected locations (mandatory due to the posibility of hardware change), you must provide a Symbol Table to the modules, else they won't know how to call the kernel. (Don't even think on using a typical syscall, that would be too slow and expensive, leave it for applications) This would include all the fast and low-level API functions the drivers should use.

For the dynamic-loading problem, use this solution:
* <i>#include</i> all the symbols you want to export into a single place.
* Make a function pointer array. As you have different function return types and parameters, you should do this:
<pre> uint32_t KRNLSYMTABLE[123];</pre>
where you must replace 123 with your function amount.
* On setup, fill the table like this:
<pre> KRNLSYMTABLE[321] = (uint32_t)&myFunction;</pre>
* When loading a module, you should (using paging) reference the KRNLSYMTABLE dedicated page (yes, it must be like that) to a fixed virtual memory page, so the module always knows where to find things.

Notice that this model uses a separate page directory as per driver. It isn't the best approach, but with this the kernel won't crash with a module.

[[Category:Object Files]]
````

<a id="w003064"></a>

## W003064 — User:Jezze

Bron-ID: W003064
Namespace: User (2)
Revisie: 12359 | Datum: 2011-12-18T23:39:03Z
Bron: https://wiki.osdev.org/User:Jezze
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJezze&oldid=12359

````text
Real name is Jens Nyberg from Uppsala, Sweden and I'm the author of Fudge.

https://github.com/Jezze/fudge
````

<a id="w003065"></a>

## W003065 — User talk:Ammubhave

Bron-ID: W003065
Namespace: User talk (3)
Revisie: 12373 | Datum: 2011-12-24T16:24:07Z
Bron: https://wiki.osdev.org/User_talk:Ammubhave
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AAmmubhave&oldid=12373

````text
Hi, I am Amol Bhave who aspires to develop an OS
````

<a id="w003067"></a>

## W003067 — MediaWiki:Common.css

Bron-ID: W003067
Namespace: MediaWiki (8)
Revisie: 29133 | Datum: 2024-06-22T20:56:43Z
Bron: https://wiki.osdev.org/MediaWiki:Common.css
Revisie-URL: https://wiki.osdev.org/index.php?title=MediaWiki%3ACommon.css&oldid=29133

````text
/* CSS placed here will be applied to all skins */
.client-darkmode .mw-wiki-logo {
    filter: invert(1) hue-rotate(180deg);
    -webkit-filter: invert(1) hue-rotate(180deg);
}

/* Fix so <syntaxhighlight> tags and .css and .js pages get normal text size */
div.mw-geshi div,
div.mw-geshi div pre,
span.mw-geshi,
pre.source-css,
pre.source-javascript {
    font-family: monospace, Courier !important;
}
````

<a id="w003068"></a>

## W003068 — MediaWiki:Geshi.css

Bron-ID: W003068
Namespace: MediaWiki (8)
Revisie: 12400 | Datum: 2012-01-07T21:58:51Z
Bron: https://wiki.osdev.org/MediaWiki:Geshi.css
Revisie-URL: https://wiki.osdev.org/index.php?title=MediaWiki%3AGeshi.css&oldid=12400

````text
/* CSS placed here will be applied to GeSHi syntax highlighting */
div.mw-geshi {
  padding: 1em; 
  margin: 1em 0; 
  border: 1px dashed #2f6fab;
  background-color: #f9f9f9;
}
````

<a id="w003072"></a>

## W003072 — Talk:What order should I make things in?

Bron-ID: W003072
Namespace: Talk (1)
Revisie: 26302 | Datum: 2021-08-12T22:59:59Z
Bron: https://wiki.osdev.org/Talk:What_order_should_I_make_things_in%3F
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AWhat_order_should_I_make_things_in%3F&oldid=26302
Doorverwijzing: Talk:What Order Should I Make Things In?

````text
#REDIRECT [[Talk:What Order Should I Make Things In?]]
````

<a id="w003073"></a>

## W003073 — Required Knowledge

Bron-ID: W003073
Namespace: Main (0)
Revisie: 31039 | Datum: 2026-10-23T14:00:30Z
Bron: https://wiki.osdev.org/Required_Knowledge
Revisie-URL: https://wiki.osdev.org/index.php?title=Required_Knowledge&oldid=31039

````text
'''Beginners in OSDev are strongly recommended not to skip this.'''

Due to the complexity of the task, writing an OS requires a high level of familiarity and experience with programming and computer science concepts. 

Developing an OS requires knowledge on the following areas:

# '''Basic Computer Science''': Intimate familiarity with hexadecimal and binary notation as well as boolean logic and similar fundamental knowledge, like data structures, their construction and manipulation, searching and sorting algorithms, abstract programming concepts.
# '''Language and Vocabulary''': Ability to read and write (technical) English. Virtually all technical documentation is in English, including most of the resources on the web (like this Wiki and the [http://forum.osdev.org forum] are). Correct terminology is imperative for receiving proper assistance.
# '''Language and Vocabulary, pt. 2''': Most operating systems featured on this site, as well as most of the code snippets and examples, are written in [[C]] (or [[C++]]). Due to this, a strong understanding of C is required for OS development.
# '''Assembly''': Knowledge about the low-level language [[Assembly]]. Resources for learning include books and school/college courses. Writing user-space code serves as practice. A strong understanding of Assembly is required, even if one plans to write most of the operating system in a higher-level language.
# '''Programming experience''': Developing OS projects requires experience with developing user-space programs and a good understanding of the programming language that will be used before proceeding with a project. Knowledge of concepts such as version control and debugging is required.
# '''Programming Practices''': Ability to write both code and user documentation, and be able to document all aspects of code and design, even if the project is purely for personal use. Programmers must learn and use appropriate [[Code Management]] practices, including setting up and using an off-site repository (e.g. Savannah, GitHub, GitLab, Heroku) in order to prevent any issues during development.
# '''UNIX experience''': Most OS development tools are developed natively for Unix. The Linux kernel is a reference for how things can be done, and many of the hobby operating systems have some resemblance to Unix. Having experience with the Unix command line (preferably Bash or ksh) is a very important requirement. ([[Cygwin]] provides an easy-to-install Unix command line for Windows.) For Windows users, this can be accomplished by using a [[Emulators|virtualizer]] (see below) to host a virtual system, without the need to re-partition the home system. Windows Subsystem for Linux (WSL) also allows access to the Unix command line. macOS users can use the Terminal--macOS is built on a Unix variant kernel (a combination of the Mach and BSD kernels), if Xcode and an emulator or virtualizer are installed, no further tools are necessary (a cross-compiler is still required); the default Terminal shell is the Z Shell, but C shell and KornShell are available as well.
# '''Toolchain''': Knowledge of the behavioral details of the compiler, assembler, linker, and make utility. Being able to understand emitted warnings and errors. Tool documentations are the first source that should be referenced. Answer to most beginner's questions about [[GCC]], [[GAS|GNU as]], [[NASM]], [[LD|GNU ld]], [[Visual Studio]], and [[GRUB]] can be found online.
# '''Emulators and Virtualizers''': Familiarity with tools such as [[Bochs]], [[VirtualBox]], [[QEMU]], or [[Microsoft_Virtual_PC|Virtual PC]]. These tools provide a buffer between real hardware and the test system. Knowledge of these tools is required prior to starting OS development.
# '''Executable Formats''': Kernel space programming has many additional requirements unknown to application development. Being able to parse executable binaries is an example. One should familiarize themselves with [[:Category:Executable Formats|Executable File Types]], their internal structure, and how a linker generates them.
# '''The Platform''': Read the manuals for the processor that the code will be written for. They contain the information required to design the kernel.
# '''The Concept''': Technical knowledge of currently existing operating systems is beneficial as these are built on the same principles required for any successful OS project.

== See Also ==

* A good (but unfinished) book on the topic: [https://github.com/tuhdo/os01/blob/master/Operating_Systems_From_0_to_1.pdf Operating Systems From 0 to 1]. This book covers transistors, logic circuits, Assembly and C language, ELF format and debugging, etc.

=== Articles ===

* [[Books]]

[[Category:OS theory]]
````

<a id="w003074"></a>

## W003074 — Required Knowlege

Bron-ID: W003074
Namespace: Main (0)
Revisie: 12444 | Datum: 2012-01-26T06:03:15Z
Bron: https://wiki.osdev.org/Required_Knowlege
Revisie-URL: https://wiki.osdev.org/index.php?title=Required_Knowlege&oldid=12444
Doorverwijzing: Required Knowledge

````text
#REDIRECT [[Required Knowledge]]
````

<a id="w003077"></a>

## W003077 — User:Bellezzasolo/IDT problems

Bron-ID: W003077
Namespace: User (2)
Revisie: 12582 | Datum: 2012-02-09T18:14:54Z
Bron: https://wiki.osdev.org/User:Bellezzasolo/IDT_problems
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ABellezzasolo%2FIDT_problems&oldid=12582

````text
== Problems with IDTs ==
Many of us while OS dev'ing will encounter a problem with IDT's. I won't pretend to be an IDT agony aunt - in fact I've been in trouble myself.
That is why I created this page - a wiki page you can easily access  that's in a nice format. Introduction aside, here are the solved problems.

This is for solved problems. The unsolved ones can be found here on the 
[http://forum.osdev.org/viewtopic.php?f=1&t=24805 Forum]

==Problems==
{{stub}}
Please post '''''Completed''''' problems here.

First of all check your GDT. Then check padding issues.

== Related links ==
*[[IDT]]

*[[Visual Studio]]

*[[I_Cant_Get_Interrupts_Working|Interrupt problems]]

[[Category:X86 CPU]]
[[Category:Interrupts]]
[[Category:Troubleshooting]]
[[Category:Stubs]]
````

<a id="w003078"></a>

## W003078 — User:Bellezzasolo

Bron-ID: W003078
Namespace: User (2)
Revisie: 18938 | Datum: 2016-02-12T10:54:43Z
Bron: https://wiki.osdev.org/User:Bellezzasolo
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ABellezzasolo&oldid=18938

````text
== Me ==

Welcome to my home page. For that reason please edit the talk area, not this part.
Here are the pages I've created:

*[[IDT_problems| IDT problems]]
*[[SSE2| SSE2]]
*[[User:Bellezzasolo/CFS|CFS]]
*[[User:Bellezzasolo/Memory Management|Memory Management]]

=== Talk to me ===
You can talk to me at my [[User talk:bellezzasolo|talk]] page. Any contribution greatly appreciated.

===My experience===
Accolades:
*No computing-related official accolades. (I'm too young yet)
*However, I do have a working OS. That has to count for something! :)

However I do have over a year's experience of OS deving in holidays and at weekends.

I have been programming for almost 3 years. I know that makes me probably the most inexperienced on this community, but allow me to continue.
Order of languages:
*HTML
*JavaScript -a month or so down the line
*PHP(never really got going because of server side) -similar time
*Java(again never really got going) -a few months down the line
*[[C++]] - loved it, but never had a brilliant idea. -about 6 months in
Started OS programming here -about 1 1/2 years in. I found working on someone else's black box unworkable. I had no brilliant ideas and the API was difficult to learn, whereas creating my own? Easy.
*[[Assembly]] - x86
*[[C]] - know the basics from C++ and brief experience. My OS is totally C++, with many Object Orientated elements.

I learnt a large amount of basic C++ through my OS. I used the brokenthorn OS as my OS for a while, and at it's height I was working on loading programs.

I decided to restart and do my own coding. I now have a white CLI on a light blue background in PMode and CLI functionality, with PCI probing and a simple VFS (along with a disk management system I call the VDS)(Both implemented as classes) I also have a simple VGA graphics screen. I now probably can be called a C++ guru as I know how to manipulate pointers and references, as well as classes. I still find asm a bit cluttered if the algorithm is complex, but if I'm doing my kernel in C++ I only need to provide basic asm routines.

==My Setup==
===Overview===
I have a OS that is going to support x86 and AMD64 (with an actual 64 bit version). For testing purposes I am currently using the [http://www.brokenthorn.com/Resources/OSDevIndex.html BrokenThorn] bootloader. I am adabting it for 64 bit purposes, but my target setup is my own bootloader on a HDD (MBR currently working).

===Assembler===
I use [http://www.nasm.us NASM] as my assembler, in conjunction with the [http://www.notepad-plus-plus.org Notepad++] editor.

===Compiler/Linker===
I use the unusual, but productive and familiar, environment provided by Visual C++ 2010 Express (SP1). I installed the windows SDK (update for SP1) for the 64 bit compilers. Because Visual C++ doesn't support 64 bit inline asm or naked functions I use this type of setup:
<source lang="cpp">
//Asmlayer.h - 32 bit and 64 bit
#ifdef __cplusplus
extern "C" {
#endif
 void disable();
 //other function declarations go here
#ifdef __cplusplus
}
#endif
</source>
And in my asm code:
<source lang="asm">
;Asmlayer.asm - 32 bit version
BITS 32
global @disable@0 ;all will be revealed
;other declarations go here

@disable@0:
cli
ret

;Asmlayer64.asm - 64 bit version
BITS 64
global disable
;other declarations go here

disable:
cli
ret
</source>
Note the fastcall convention - 64 bit VC++ only supports fastcall.
I use the integrated editor for C/C++, and Notepad++ for asm (syntax highlighting)

===Controversial issues===
I use gotos. Sometimes I come across an algorithm I've just written and there are to options: rewrite the algorithm or use the goto. For lack of time to rewrite the algorithm, I use a goto. Don't get me wrong, I'm a fan of neat code. However, there are times when I use a goto and the odd goto doesn't mess up neatness. Admittedly, a algorithm full of gotos would be hard to follow around (like asm), but the odd goto is like using the odd break statement. That is my opinion.
===Emulator===
I use several emulators, but for testing purposes it has to be bochs.

===Development/Test PC===
For development I use an HP Pavilion g6 (6GB RAM, quad-core 1.5 GHz AMD A8 3500M, 1TB HDD), with Windows 7 64 bit home edition.

For testing I will use emulators, then once fairly trusted I will place on a HP Pavillion 6405 (64MB RAM, 6GB HDD, 400Mhz Celeron), which currently has Win98 SE dual-booted with Linux Mandrake 7. Once I have tested HDD's completely, I will place it on a small partition of my development PC (64 bit testing).

==My OS==
My OS is an 32/64 bit x86 OS. It currently has a CLI implemented in a grpahics mode, and virtually no drivers (basic chip drivers only). It '''WILL''' be extended to HDD's and use a custom file system, [[CFS]]. <br />
<!--For more information, see [[User:Bellezzasolo/MyOS|My OS]]-->

==See also==
===Me===
*[[User talk:Bellezzasolo]]
*[[User:Bellezzasolo/Old talk]]

===My Likes===
*[[User:Lionel/What kind of kernel should I make?]]

===Related articles===
*None yet
````

<a id="w003079"></a>

## W003079 — User talk:Bellezzasolo

Bron-ID: W003079
Namespace: User talk (3)
Revisie: 29496 | Datum: 2025-04-27T12:00:49Z
Bron: https://wiki.osdev.org/User_talk:Bellezzasolo
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ABellezzasolo&oldid=29496

````text

== Talk ==
Welcome to my talk page. My home page is here:
[[User:bellezzasolo]]
Any comments on my style are greatly appreciated, both positive and negative.<br />
If you wish to see my old discussions, go here: [[User:Bellezzasolo/Old_talk]]

== Recent activity ==

Hi there, and sorry for the "accidential" rollback from me just now.

I don't quite know what to make of your recent flurry of activity. You did lots of categorizing, I see, but also a great many edits that I find to be... arguable. (For example, your adding of Visual Studio to the list of "frequently asnwered things" in [[Required Knowledge]]. We generally and strongly suggest to use a GCC cross-compiler setup, and Visual Studio could be considered a second-class citizen in that regard.) The sheer amount of edits from you makes it somewhat difficult to track what you are actually ''doing''.

I won't tell you not to do this, and I certainly welcome any quality edits to the Wiki, so let me just say that I feel uneasy when I see hundreds of edits within a few weeks done by someone with a two-digit post count to the forum. ;-)

Anyway, welcome to OSDev.org. -- [[User:Solar|Solar]] 07:40, 20 February 2012 (CST)

:Sorry about the difficulties in analyzing my edits. This problem arises as most of the edits I've made have been correcting little mistakes on the wiki, or improving the source highlighting on certain pages, apart from the categorizing. - [[User:Bellezzasolo|Bellezzasolo]] (Please sign your messages)



I've just rolled back the categorizing of files. A category is meant to link to pages relevant to the subject, not administrative material. The same goes for the templates where I was kindof glad you found another obstacle early (there is a fix for that) for the sake of my own effort. If people are looking for those things, there is the [[Special:AllPages]] listing to go to. 

On another note, I also plan on merging the [[:Category:SSE|SSE pages]] because there's little to gain by keeping the stubs away from the main page on the subject, and they all look alike. Usually, you'd want someone to enter the search term and land directly at the page with all the relevant info - the difference between versions of SSE matters in that regard. Try to think about making people see what they '''need''' to see, and not for indulging some strange idea of perfectionism. (Knowing the difference is probably related to that two-digit post count Solar mentioned earlier - it needs practice)

So far today's tips. One '''request''' though is that you use <tt>~~<b></b>~~</tt> to sign your messages on talkpages so people can immediately see who's on the other end. 

- [[User:Combuster|Combuster]] 13:55, 22 February 2012 (CST)

== Quality and appropriateness of code ==

Apparently, you haven't really understood the point of my criticism over at [[Printing to Screen]]. Putting it in a separate page doesn't help. I am sorry for being so blunt, but the quality of your "print" source simply does not satisfy as tutorial / example code. 

It's undercommented. It has lots of unwieldly constructs (<tt>char null = '\0'</tt>???), functions named ''almost'' like standard ones that aren't (<tt>Puts()</tt> and the lack of a newline), it is calling undefined functions (<tt>disable()</tt> / <tt>enable()</tt>), it's using global externally visible variables - and '''goto?''' I mean, really? I'd probably find more if I cared to look further. That's lots of red ink for 100-something lines of code, and certainly too much for this to reside in our Wiki for others to copy & paste, IMNSHO.

Plus, the overall gist of this Wiki (and the forum) is to '''not''' provide ready-made solutions to copy & paste, but to give the information necessary to do it ''yourself''. For example, the information that 0xb8000 is the beginning of video memory is eligible. The information that the first byte is the ASCII code and the second byte is the color is eligible. The information which value in the color byte corresponds to which color is eligible. Links to existing implementations are eligible. Posting code for <tt>putc()</tt>, <tt>puts()</tt> and <tt>printf()</tt> is ''not'' eligible. -- [[User:Solar|Solar]] 07:02, 23 February 2012 (CST)

== AML ==

Hi, please could you clarify your request for more information on [[AML]] on the [[Wish List]] talk page.  Thanks.  [[User:Jnc100|John]] 10:27, 5 July 2012 (CDT)

== GOTO ==

Regarding your comments about the use of goto.  I am a kernel and hypervisor developer by profession (after doing a hoby OS for longer than I care to admit), and there are many cases where goto is simply the correct solution to a problem.  A classic example is for functions which take a spinlock, from which you might otherwise wish to return early. [[User:Andyhhp|Andyhhp]] 16:51, 7 August 2012 (CDT)

I agree entirely. One example is my putc() function. I have scrolling code, and before that I write the character to video memory. I use a goto in vertical tabs (and others) to avoid writing the character.--[[User:Bellezzasolo|Bellezzasolo]] 14:41, 10 August 2012 (CDT)

I can think of correct, readable solutions for both scenarios so I don't think they are good examples. In fact, I find the latter to be a very bad one. The use of <tt>goto</tt> is not bad as long as it doesn't hinder readability, or the code that uses it is produced automatically and not meant to be read by humans (e.g., the output of parser generators). The use of <tt>goto</tt> is ''recommended'' in cases where using anything else would decrease readability (e.g., jumping out of a nested loop, error handling in the absence of exception handling, etc.). --[[User:Love4boobies|Love4boobies]] 10:21, 11 August 2012 (CDT)

==See also==
===Me===
*[[User:Bellezzasolo]]
*[[User:Bellezzasolo/Old talk]]
===Related articles===
*[[SSE]]

== Recent reversion to linker script changes ==

Unless the BLOCK() statements are more straightforward in some way, they are only used for compatibility purposes, as explained by the GNU Linker Manual, p.90 (https://sourceware.org/binutils/docs-2.37/ld.pdf). The BLOCK() symbol is also unknown the LLD linker, which should be considered, especially for users interested in using the LLVM toolchain. Using sparser symbols is clearer as well.

As the linker script is written, the BLOCK() statement is used to align the section with the next available byte-aligned address boundary (given by the location counter). In this case, that address is a multiple of 4K. The second argument in the ".text <arg1> : <arg2>" targets the internal alignment of the statements inside the section block. Why shouldn't the first argument be more appropriate for an ALIGN() instead of a BLOCK(), given the reasons I outlined? - ''Unsigned comment by'' [[User:Ffexl]]

: Hi [[User:Ffxel|Ffxel]]. Firstly, it would be helpful if you signed your messages with <tt>~~<b></b>~~</tt>. As for the issue at hand, I think discussion of the format and whether compatibility with old linkers is a priority, or whether it should work with LLVM out of the box, belongs at [[Talk:Bare Bones]]. Personally, I don't have strong feelings either way, I'd hope that it's fairly straightforward to figure out the issue for users of LLVM. What I didn't like wasn't that you changed BLOCK to the equivalent ALIGN, but that in the article, '''there were already ALIGN statements'''.

:So you changed <pre>.text BLOCK(4K) : ALIGN(4K)</pre> to <pre>.text ALIGN(4K) : ALIGN(4K)</pre>, which is I think objectively worse.

:Thanks, [[User:Bellezzasolo|Bellezzasolo]] ([[User talk:Bellezzasolo|talk]]) 12:00, 27 April 2025 (UTC)
````

<a id="w003080"></a>

## W003080 — Template:Troubleshooting

Bron-ID: W003080
Namespace: Template (10)
Revisie: 15665 | Datum: 2014-02-26T22:39:00Z
Bron: https://wiki.osdev.org/Template:Troubleshooting
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ATroubleshooting&oldid=15665

````text
*[[Bran%27s_Known_Bugs|Bran's known bugs]]
*[[James Molloy's Known Bugs]]
*[[I_Cant_Get_Interrupts_Working|I Cant Get Interrupts Working]]
*[[IDT_problems|IDT problems]]
*[[Testing]]
*[[Troubleshooting]]
*[[Unit_Testing|Unit Testing]]
<noinclude>[[Category:Main Page Templates]]</noinclude>
````

<a id="w003081"></a>

## W003081 — Windows Virtual PC

Bron-ID: W003081
Namespace: Main (0)
Revisie: 12551 | Datum: 2012-02-06T11:42:20Z
Bron: https://wiki.osdev.org/Windows_Virtual_PC
Revisie-URL: https://wiki.osdev.org/index.php?title=Windows_Virtual_PC&oldid=12551
Doorverwijzing: Microsoft Virtual PC

````text
#REDIRECT [[Microsoft Virtual PC]]
````

<a id="w003082"></a>

## W003082 — Talk:Windows Virtual PC

Bron-ID: W003082
Namespace: Talk (1)
Revisie: 12550 | Datum: 2012-02-06T07:55:29Z
Bron: https://wiki.osdev.org/Talk:Windows_Virtual_PC
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AWindows_Virtual_PC&oldid=12550

````text
Please could you add a navigation box

I'd suggest to append this page to the [[Microsoft Virtual PC]] one, instead of creating a brand new one, for a so similar emulator. --[[User:Alfaomega08|Alfaomega08]] 06:17, 5 February 2012 (CST)

Agreed. I think this page should redirect to the Microsoft Virtual PC page, since the two are essentially the same product (only difference being a rename for the new version). [[User:Jackscott|JackScott]] 01:55, 6 February 2012 (CST)
````

<a id="w003083"></a>

## W003083 — User talk:Bellezzasolo/IDT problems

Bron-ID: W003083
Namespace: User talk (3)
Revisie: 12586 | Datum: 2012-02-09T18:37:12Z
Bron: https://wiki.osdev.org/User_talk:Bellezzasolo/IDT_problems
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ABellezzasolo%2FIDT_problems&oldid=12586

````text
It would be good to have an IDT problems page. But as by now, it's rather ''selfish''. You're just asking for a solution to your problem. [http://forum.osdev.org] is more appropriate for that (your problem might have been discussed times and times, try with the search button). Once you got the answer, write a page on the wiki for that in a generic, NPOV, adaptable way. - [[User:Alfaomega08|Alfaomega08]] (please sign your comments!)

Actually, there is an existing page on the subject at [[I Cant Get Interrupts Working]], which sofar does not discuss ''crashes'' caused by broken setups because they are usually obvious enough for a fix. There's still some room for that though. A FAQ entry about help my kernel crashes could hold that part of the content. I propose a merge when the author fixed his own problems - [[User:Combuster|Combuster]] 12:37, 9 February 2012 (CST)
````

<a id="w003084"></a>

## W003084 — Category:Level 4 Tutorials

Bron-ID: W003084
Namespace: Category (14)
Revisie: 12564 | Datum: 2012-02-06T18:09:17Z
Bron: https://wiki.osdev.org/Category:Level_4_Tutorials
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ALevel_4_Tutorials&oldid=12564

````text
A list of the ''Master'' level tutorials on this wiki.

[[Category:Tutorials]]
````

<a id="w003085"></a>

## W003085 — Category:IA-64

Bron-ID: W003085
Namespace: Category (14)
Revisie: 12624 | Datum: 2012-02-17T23:24:57Z
Bron: https://wiki.osdev.org/Category:IA-64
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AIA-64&oldid=12624

````text
These pages are about the the Itanium 64 architecture
[[Category:Platforms]]
````

<a id="w003087"></a>

## W003087 — IDT problems

Bron-ID: W003087
Namespace: Main (0)
Revisie: 19308 | Datum: 2016-06-02T09:49:35Z
Bron: https://wiki.osdev.org/IDT_problems
Revisie-URL: https://wiki.osdev.org/index.php?title=IDT_problems&oldid=19308
Doorverwijzing: I Can't Get Interrupts Working

````text
#REDIRECT [[I_Can't_Get_Interrupts_Working#Problems_with_IDTs]]
````

<a id="w003088"></a>

## W003088 — Talk:IDT problems

Bron-ID: W003088
Namespace: Talk (1)
Revisie: 12585 | Datum: 2012-02-09T18:14:54Z
Bron: https://wiki.osdev.org/Talk:IDT_problems
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AIDT_problems&oldid=12585
Doorverwijzing: User talk:Bellezzasolo/IDT problems

````text
#REDIRECT [[User talk:Bellezzasolo/IDT problems]]
````

<a id="w003089"></a>

## W003089 — CFE Bare Bones

Bron-ID: W003089
Namespace: Main (0)
Revisie: 30248 | Datum: 2026-03-16T15:49:22Z
Bron: https://wiki.osdev.org/CFE_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=CFE_Bare_Bones&oldid=30248

````text
This example shows how to boot a simple "Hello world!" kernel using Broadcom's Common Firmware Environment (CFE). I have tested it on a [http://hardware.wikinet.org/wiki/ET9000 Clarke-Tech ET9000] DVB-S receiver, which is based on a [http://hardware.wikinet.org/wiki/BCM7405 BCM7405] [[MIPS]] CPU. To compile this example, you need a GCC / Binutils cross toolchain for a mipsel-elf target - and a basic knowledge of the MIPS architecture.

== <tt>entry.S</tt> ==
The entry point of the kernel is written in assembler language. It sets up a small stack (4 kB), calls the C entry point and puts the CPU in an endless loop.

<syntaxhighlight lang="asm">
.global start
.extern main

.set noreorder

.set STACKSIZE, 0x1000

.section .text

start:
	la $sp, stack
	addiu $sp, STACKSIZE - 32
	jal main
	nop
	b .
	nop

.section .bss
stack:
	.space STACKSIZE
</syntaxhighlight>

Assemble this code using:
<syntaxhighlight lang="bash">mipsel-elf-as -o Entry.o Entry.S</syntaxhighlight>

== <tt>main.c</tt> ==
The C part of the kernel contains a simple putchar function that outputs a character to the serial console, a printk function for printing a string, and the main function. Note that putchar directly accesses some hardware registers, which may be at a different address on other CPUs. This example prints the output to UART0 (16550A compatible) of the BCM7405, which is connected to the serial port of the ET9000.

<syntaxhighlight lang="c">
void putchar(char c)
{
	volatile int* lsr = (volatile int*)0xb0400b14; // Line status register.
	volatile int* thr = (volatile int*)0xb0400b00; // Transmitter holding register.

	while(((*lsr) & 0x20) == 0) ; // Wait until THR is empty.

	*thr = c;
}

void printk(const char* s)
{
	while(*s)
	{
		putchar(*s);
		s++;
	}
}

int main(void)
{
	printk("Hello world!\n");
	return 0;
}
</syntaxhighlight>

Compile this code using:
<syntaxhighlight lang="bash">
mipsel-elf-gcc -o Main.o -c Main.c -Wall -Wextra -Werror -nostdlib -fno-builtin -nostartfiles -nodefaultlibs -O2
</syntaxhighlight>

== Linker script ==
Finally, we need to link our kernel. In this case we link the kernel to virtual address 0x80001000, which is the start of available RAM on the ET9000, accessed through the cached kseg0 window of the MIPS architecture.

<pre>
OUTPUT_FORMAT(elf32-littlemips)
OUTPUT_ARCH(mips:isa32)
ENTRY(start)

SECTIONS
{
	.text (0x80001000) :
	{
		*(.text)
		*(.text.*)
		*(.stub)
		*(.gnu.linkonce.t.*)
	}

	.rodata ALIGN(4K) :
	{
		*(.rodata*)
		*(.gnu.linkonce.r.*)
	}

	.data ALIGN(4K) :
	{
		*(.data*)
		*(.gnu.linkonce.d.*)
	}

	.bss ALIGN(4K) :
	{
		*(.common)
		*(.bss*)
		*(.gnu.linkonce.b.*)
	}

	/DISCARD/ :
	{
		*(.gcc_except_table)
		*(.eh_frame)
		*(.note)
		*(.comment)
		*(.rel.*)
		*(.rela.*)
	}
}
</pre>

Finally, link the kernel:
<syntaxhighlight lang="bash">mipsel-elf-ld -T ldscript -o Kernel.elf Entry.o Main.o</syntaxhighlight>

== Booting the kernel ==
These instructions are again hardware dependent. I have tested them on an ET9000 DVB-S receiver, but booting other devices may be very different.

* Switch off the ET9000.
* Put the kernel file Kernel.elf on a FAT32 formatted USB stick and plug it into the ET9000.
* Connect the serial port of the ET9000 to your computer.
* Open a serial console program.
* Switch on the ET9000.
* Press Ctrl + C to prevent an automatic startup and boot into the CFE console.
* Enter the command "show devices" at the CFE prompt to see whether the USB stick has been found.
* Enter "dir usbdisk0" (or whatever name has been assigned to the USB stick by CFE) to see whether your kernel is in place.
* Enter "boot -elf usbdisk0:Kernel.elf" to boot the kernel.

If you see something like this, your kernel works:

<pre>
CFE> boot -elf usbdisk0:Kernel.elf
Loader:elf Filesys:fat Dev:usbdisk0 File:Kernel.elf Options:(null)
Loading: 0x80001000/4112 0x80003000/4096 Entry address is 0x80001000
TP1 Entry Address at 0x80000ffc = 80001000
Starting program at 0x80001000

Hello world!
</pre>

[[Category:Bare bones tutorials]]
[[Category:MIPS]]
````

<a id="w003090"></a>

## W003090 — Talk:CFE Bare Bones

Bron-ID: W003090
Namespace: Talk (1)
Revisie: 12589 | Datum: 2012-02-10T22:44:51Z
Bron: https://wiki.osdev.org/Talk:CFE_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ACFE_Bare_Bones&oldid=12589

````text
Suggestions:

* Add a somewhat longer into as to what CFE actually '''is'''.
* Make the article resemble [Bare Bones] as closely as possible, so as to make it easier to spot differences.
* Don't call it <tt>printk()</tt>. With very little work you could turn that into a <tt>puts()</tt>, or a <tt>kputs()</tt> if you will, but don't call it <tt>printk()</tt>. ;-)

Good work, anyway. -- [[User:Solar|Solar]] 16:44, 10 February 2012 (CST)
````

<a id="w003091"></a>

## W003091 — Windows Loopback Alternative

Bron-ID: W003091
Namespace: Main (0)
Revisie: 28215 | Datum: 2023-07-10T19:05:00Z
Bron: https://wiki.osdev.org/Windows_Loopback_Alternative
Revisie-URL: https://wiki.osdev.org/index.php?title=Windows_Loopback_Alternative&oldid=28215

````text
{{Rating|1}}

It is a well known fact that Windows doesn't support [[Loopback Device|loopback devices]] but there is an alternative available for Windows 7 users.
Windows 7 has a support for so called Virtual Hard Disks which are just plain bin files with data.
Windows 7 also includes the interface for creation and management of those VHD files.


== Creating a VHD ==

To create a VHD you must first open the Computer Management window, which is most easily accessed by right clicking My Computer and selecting Manage.
<br/>
[[File:Windows-Computer-Management.png]]

<br/>
Then go to Storage - Disk Management. Right click Disk Management and select Create VHD. It will guide you through the creation of a VHD file.
<br/>
[[File:Windows-Create-VHD.png]]

== Attaching a VHD ==

By right clicking Disk Management, and selecting Attach VHD, you can attach any VHD ( Possibly any binary file) and mount it to a specific drive.

== Use ==

You can setup your IDE ( I have used Visual Studio) to automatically detach a VHD after compilation and run the OS in a emulator, such as QEMU.
Here is how you can achieve such behavior.

Windows contains a command line program called diskpart which is used for mounting of the VHDs, but you need to supply a script for it in order to work.
<br/>
c:\os\harddrive.vhd shows the location of the VHD file used.

Script for attaching (Saved as VHDScriptAttach.txt) :

 select vdisk file=c:\os\harddrive.vhd
 attach vdisk

Script for detaching (Saved as VHDScriptDetach.txt) :

 select vdisk file=c:\os\harddrive.vhd
 detach vdisk

After that you need to setup your IDE/Compiler to run this batch script after compilation/on debug.

 diskpart -s VHDScriptDetach.txt
 qemu.exe -hda C:\OS\Harddrive.vhd
 diskpart -s VHDScriptAttach.txt

[[Category:Disk Image Utilities]]
[[Category:Windows]]
````

<a id="w003092"></a>

## W003092 — VGA Fonts

Bron-ID: W003092
Namespace: Main (0)
Revisie: 30306 | Datum: 2026-03-17T18:50:19Z
Bron: https://wiki.osdev.org/VGA_Fonts
Revisie-URL: https://wiki.osdev.org/index.php?title=VGA_Fonts&oldid=30306

````text
{{TutorialTone}}
So you know how to display characters in text mode, and now you want to do it in graphics mode. It's not complicated, but definitely more complex than writing an ASCII code at a specific offset in memory. You'll have to do it pixel by pixel.

But how do you know what to draw? It's stored in data matrix called bitmap fonts.

== Decoding of bitmap fonts ==
How is a character stored in memory? It's quite simple, 0 encodes background, 1 encodes foreground color. VGA fonts are always 8 bits wide so that each byte contains exactly one row.
For letter 'A' in the typical 8x16 font it would be (in binary):
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
</pre>
The full bitmap contains bitmaps for every character, thus it's 256*16 bytes, 4096 bytes long. If you want to get the bitmap for a specific character, you have to multiply the ASCII code by 16 (number of rows in a character), add the offset of your bitmap and you're ready to go.

A very simple file format to store these is [[PC Screen Font]], used by the Linux Console. It stores fonts the way described above with a small header. Another solution is the [[Scalable Screen Font]] format which comes with an extremely small, free rendering ANSI C library.

By reading bits you can display a symbol on the screen:<syntaxhighlight lang="c">
#include <gfx.h>

// Video mode information (width, height, etc...)
extern video_mode_t GFX_VIDEO_MODE;

/**
 * @note gfx_draw_pixel(x, y, r, g, b) - Procedure for drawing one pixel
 * @warning Bits in glyphs must be in the MSB format!
 * @param x X of the point on the screen where we will display the symbol
 * @param y Y of the point on the screen where we will display the symbol
 * @param glyph_data Glyph (array of bytes/words/...)
 * @param width_bits Glyph width in bits
 * @param height_bits Glyph height in bits
 * @param frg_r Red glyph color component
 * @param frg_g Green glyph color component
 * @param frg_b Blue glyph color component
 * @param target_width Target width of the glyph on the screen (the glyph can be stretched/compressed)
 * @param target_height Target height of the glyph on the screen
 * @param bkg_r Red background color component
 * @param bkg_g Green background color component
 * @param bkg_b Blue background color component
 * @param fill_bkg Determines whether the background under the glyph will be painted over
 */
void gfx_draw_glyph(
	int x, int y,
	const void* glyph_data,
	size_t width_bits, size_t height_bits,
	uint8_t frg_r, uint8_t frg_g, uint8_t frg_b,
	size_t target_width, size_t target_height,
	uint8_t bkg_r, uint8_t bkg_g, uint8_t bkg_b,
	bool fill_bkg
) {
	const video_mode_t* vm = &GFX_VIDEO_MODE;
	if (target_width == 0 || target_height == 0) return;

	// src_x = floor(dst_x * src_w / target_width)
	// src_y = floor(dst_y * src_h / target_height)

	size_t stride = (width_bits + 7) / 8;
	if (fill_bkg) {
		for (size_t dy = 0; dy < target_height; ++dy) {
			size_t src_y = (dy * height_bits) / target_height;
			const uint8_t* row = glyph_data + src_y * stride;
			for (size_t dx = 0; dx < target_width; ++dx) {
				size_t src_x = (dx * width_bits) / target_width;
				size_t byte_index = src_x >> 3;
				uint8_t mask = 0x80 >> (src_x & 7);				// MSB(!)
				if (row[byte_index] & mask) {
					int px = x + dx;
					int py = y + dy;
					if ((size_t)px < vm->width && (size_t)py < vm->height) gfx_draw_pixel(px, py, frg_r, frg_g, frg_b);
				}
				else {
					int px = x + dx;
					int py = y + dy;
					if ((size_t)px < vm->width && (size_t)py < vm->height) gfx_draw_pixel(px, py, bkg_r, bkg_g, bkg_b);
				}
			}
		}
	}
	else {
		for (size_t dy = 0; dy < target_height; ++dy) {
			size_t src_y = (dy * height_bits) / target_height;
			const uint8_t* row = glyph_data + src_y * stride;
			for (size_t dx = 0; dx < target_width; ++dx) {
				size_t src_x = (dx * width_bits) / target_width;
				size_t byte_index = src_x >> 3;
				uint8_t mask = 0x80 >> (src_x & 7);				// MSB(!)
				if (row[byte_index] & mask) {
					int px = x + dx;
					int py = y + dy;
					if ((size_t)px < vm->width && (size_t)py < vm->height) gfx_draw_pixel(px, py, frg_r, frg_g, frg_b);
				}
			}
		}
	}
}
</syntaxhighlight>'''Note:''' this procedure will display a glyph whose bits are written in MSB format. If your glyph(s) are written in LSB format, then you need to replace <code>0x80 >> (src_x & 7)</code> with <code>1 << (src_x & 7)</code>

Now we can define a function for each glyph resolution, for example:<syntaxhighlight lang="c">
#include <gfx.h>

typedef struct _glyph8x8_t {
	int				index;          // character code
	uint8_t			data[8];        // glyph(8x8) lines
} glyph8x8_t;

extern const glyph8x8_t GFX_GLYPHS8X8[];
extern const size_t GFX_NUM_GLYPHS8X8;

const glyph8x8_t GLYPH_UNKNOWN8X8 = { .index = 0, .data = {
	0b00111100,
	0b11001110,
	0b11000111,
	0b00000111,
	0b00001110,
	0b00000000,
	0b00111000,
	0b00111000
}};

void gfx_draw_glyph8x8(
	int c,
	int x, int y, 
	uint8_t frg_r, uint8_t frg_g, uint8_t frg_b,
	size_t target_width, size_t target_height,
	uint8_t bkg_r, uint8_t bkg_g, uint8_t bkg_b,
	bool fill_bkg
) {
	const glyph8x8_t* glyph = 0;
	for (size_t i = 0; i < GFX_NUM_GLYPHS8X8; i++) {
		if (GFX_GLYPHS8X8[i].index == c) {
			glyph = (const glyph8x8_t*)&GFX_GLYPHS8X8[i];
			break;
		}
	}
	
	if (!glyph) glyph = &GLYPH_UNKNOWN8X8;
	gfx_draw_glyph(
	    x, y,
	    &glyph->data,
	    8,              // glyph width in bits
	    8,              // glyph height in bits
	    frg_r, frg_g, frg_b,
	    target_width, target_height,
	    bkg_r, bkg_g, bkg_b,
	    fill_bkg
	);
}
</syntaxhighlight>

==How to get fonts?==
There're several ways. You can have it in a file on your filesystem. You can hardcode it in an array. But sometimes 4k is so much that you cannot afford, and reading a file is not an option (like in a boot loader), in which case you'll have to read the one used by the card (to display text mode characters) from VGA RAM.
===Store it in an array===
Easiest way, but increases your code by 4k. There are several sources that provide the entire font in binary or source format so you do not need to manually write it out.
Example:<syntaxhighlight lang="c">
#include <stdint.h>

typedef struct _glyph8x8_t {
	int				index;          // character code
	uint8_t			data[8];        // glyph(8x8) lines
} glyph8x8_t;

glyph8x8_t GFX_GLYPHS8X8[] = {
	{ 0, {
		0b01111110,
		0b10000001,
		0b10011101,
		0b10100001,
		0b10100001,
		0b10011101,
		0b10000001,
		0b01111110,
	}},
	/*
	    ...
	*/
	{ 'A', {
		0b00111000,
		0b01101100,
		0b11000110,
		0b11111110,
		0b11000110,
		0b11000110,
		0b11000110,
		0b00000000,
	}},
	{ 'B', {
		0b11111100,
		0b01100110,
		0b01100110,
		0b01111100,
		0b01100110,
		0b01100110,
		0b11111100,
		0b00000000,
	}},
	{ 'C', {
		0b00111100,
		0b01100110,
		0b11000000,
		0b11000000,
		0b11000000,
		0b01100110,
		0b00111100,
		0b00000000,
	}},
	/*
	    ...
	*/
};

const size_t GFX_NUM_GLYPHS8X8 = sizeof(GFX_GLYPHS8X8) / sizeof(GFX_GLYPHS8X8[0]);
</syntaxhighlight>

===Store it in a file===
Most modular way. You can use different fonts if you like. Downside you'll need a working filesystem implementation. As for the file format, I'd suggest the aforementioned [[PC Screen Font]] (.psf/.psfu) or [[Scalable Screen Font]] (.sfn).

===Get the copy stored in the VGA BIOS===
It's a standard BIOS call (no need to check it's persistence). If you're still in real mode, it's quite easy to use.
<syntaxhighlight lang="asm">
		;in: es:di=4k buffer
		;out: buffer filled with font
		push			ds
		push			es
		;ask BIOS to return VGA bitmap fonts
		mov			ax, 1130h
		mov			bh, 6
		int			10h
		;copy charmap
		push			es
		pop			ds
		pop			es
		mov			si, bp
		mov			cx, 256*16/4
		rep			movsd
		pop			ds
</syntaxhighlight>

===Get from VGA RAM directly===
Maybe you're already in protected mode, so cannot access BIOS functions. In this case you can still get the bitmap by programming VGA registers. Be careful that the VGA always reserves space for 8x32 fonts so you will need to trim off the bottom 16 bytes of each character during the copy:
<syntaxhighlight lang="asm">
		;in: edi=4k buffer
		;out: buffer filled with font
		;clear even/odd mode
		mov			dx, 03ceh
		mov			ax, 5
		out			dx, ax
		;map VGA memory to 0A0000h
		mov			ax, 0406h
		out			dx, ax
		;set bitplane 2
		mov			dx, 03c4h
		mov			ax, 0402h
		out			dx, ax
		;clear even/odd mode (the other way, don't ask why)
		mov			ax, 0604h
		out			dx, ax
		;copy charmap
		mov			esi, 0A0000h
		mov			ecx, 256
		;copy 16 bytes to bitmap
@@:		movsd
		movsd
		movsd
		movsd
		;skip another 16 bytes
		add			esi, 16
		loop			@b
		;restore VGA state to normal operation
		mov			ax, 0302h
		out			dx, ax
		mov			ax, 0204h
		out			dx, ax
		mov			dx, 03ceh
		mov			ax, 1005h
		out			dx, ax
		mov			ax, 0E06h
		out			dx, ax
</syntaxhighlight>
It worth mentioning that it has to be done '''before''' you switch to VBE graphics mode, because VGA registers are usually not accessible afterwards. This means you won't be able to map the VGA card's font memory to screen memory, and you will read only garbage.

==Set VGA fonts==
If you're still in text mode and want the VGA card to draw different glyphs, you can set the VGA font. It's worthless in graphics mode (because characters are displayed by your code there, not by the card), I only wrote this section for completeness. Modifying the font bitmaps in VGA RAM isn't hard if you read carefully what's written so far. I'll left it to you as a homework.

===Set fonts via BIOS===
Hint: check Ralph Brown Interrupt list Int 10/AX=1110h.

===Set fonts directly===
Hint: use the same code as above, but swap source and destination for "movsd".

==Displaying a character==
And finally we came to the point where we can display a character. I'll assume you have a putpixel procedure ready.
We have to draw 8x16 pixels, one for every bit in the bitmap.
<syntaxhighlight lang="c">
//this is the bitmap font you've loaded
unsigned char *font;

void drawchar(unsigned char c, int x, int y, int fgcolor, int bgcolor)
{
	int cx,cy;
	int mask[8]={1,2,4,8,16,32,64,128};
	unsigned char *glyph=font+(int)c*16;

	for(cy=0;cy<16;cy++){
		for(cx=0;cx<8;cx++){
			putpixel(glyph[cy]&mask[cx]?fgcolor:bgcolor,x+cx,y+cy-12);
		}
	}
}
</syntaxhighlight>
The arguments are straightforward. You may wonder why to subtract 12 from y. It's for the baseline: you specify y coordinate as the bottom of the character, not counting the "piggy tail" in a glyph that goes down (like in "p","g","q" etc.). I other words it's the most bottom row of letter "A" that has a bit set.

Although it's mostly useful to erase the screen under the glyph, in some cases it could be bad (eg.: writing on a shiny gradiented button). So here's a slightly modificated version, that uses a transparent background.
<syntaxhighlight lang="c">
//this is the bitmap font you've loaded
unsigned char *font;

void drawchar_transparent(unsigned char c, int x, int y, int fgcolor)
{
	int cx,cy;
	int mask[8]={1,2,4,8,16,32,64,128};
	unsigned char *glyph=font+(int)c*16;

	for(cy=0;cy<16;cy++){
		for(cx=0;cx<8;cx++){
			if(glyph[cy]&mask[cx]) putpixel(fgcolor,x+cx,y+cy-12);
		}
	}
}
</syntaxhighlight>
As you can see, we have only foreground color this time, and the putpixel call has a condition: only invoked if the according bit in the bitmap is set.

Of course the code above will be excruciatingly slow (mostly due to doing one pixel at a time, and repeatedly recalculating the address for each pixel within the "putpixel()" function). For much better performance, the code above can be optimised to use boolean operations and a "mask lookup table" instead. For example (for an 8-bpp mode):

<syntaxhighlight lang="c">
//this is the bitmap font you've loaded
unsigned char *font;

void drawchar_8BPP(unsigned char c, int x, int y, int fgcolor, int bgcolor)
{
	void *dest;
	uint32_t *dest32;
	unsigned char *src;
	int row;
	uint32_t fgcolor32;
	uint32_t bgcolor32;

	fgcolor32 = fgcolor | (fgcolor << 8) | (fgcolor << 16) | (fgcolor << 24);
	bgcolor32 = bgcolor | (bgcolor << 8) | (bgcolor << 16) | (bgcolor << 24);
	src = font + c * 16;
	dest = videoBuffer + y * bytes_per_line + x;
	for(row = 0; row < 16; row++) {
		if(*src != 0) {
			mask_low = mask_table[*src][0];
			mask_high = mask_table[*src][1];
			dest32 = dest;
			dest32[0] = (bgcolor32 & ~mask_low) | (fgcolor32 & mask_low);
			dest32[1] = (bgcolor32 & ~mask_high) | (fgcolor32 & mask_high);
		}
		src++;
		dest += bytes_per_line;
	}
}


void drawchar_transparent_8BPP(unsigned char c, int x, int y, int fgcolor)
{
	void *dest;
	uint32_t *dest32;
	unsigned char *src;
	int row;
	uint32_t fgcolor32;

	fgcolor32 = fgcolor | (fgcolor << 8) | (fgcolor << 16) | (fgcolor << 24);
	src = font + c * 16;
	dest = videoBuffer + y * bytes_per_line + x;
	for(row = 0; row < 16; row++) {
		if(*src != 0) {
			mask_low = mask_table[*src][0];
			mask_high = mask_table[*src][1];
			dest32 = dest;
			dest32[0] = (dest[0] & ~mask_low) | (fgcolor32 & mask_low);
			dest32[1] = (dest[1] & ~mask_high) | (fgcolor32 & mask_high);
		}
		src++;
		dest += bytes_per_line;
	}
}
</syntaxhighlight>

In this case the address in display memory is only calculated once (rather than up to 128 times) and 8 pixels are done in parallel (which removes the inner loop completely).

The main downside for this approach is that you need a different function for each "bits per pixel", except that 15-bpp and 16-bpp can use the same code. For worst case (32-bpp) the lookup table costs 8 KiB. The lookup table for 32-bpp can be re-used for 24-bpp, and for 4-bpp no lookup table is needed at all. To support all standard bit depths that VBE is capable of; this gives a total of 5 versions of each "draw character" function (4-bpp, 8-bpp, 15-bpp and 16-bpp, 24-bpp, 32-bpp) and 3 lookup tables (8-bpp, 15-bpp and 16-bpp, 24-bpp and 32-bpp) which cost a combined total of 14 KiB of data if you use static tables (rather than dynamically generating the desired lookup table if/when needed).

Example of pixel drawing procedures for modes with different color depths:

*Indexed RGB (8bpp, mode #13h 320x200, color palette)
*RGB 555 (15bpp)
*RGB 565 (16 bpp)
*RGB 888 (24 bpp)
*ARGB 8888 (32 bpp, <code>A</code> - reserved)
<syntaxhighlight lang="c">
#include <gfx.h>

extern video_mode_t GFX_VIDEO_MODE;
extern uint8_t* GFX_BUFFER;                 // framebuffer
const color_t GFX_PALETTE_I8[256];          // color palette

typedef struct _color_t {
	uint8_t			r;
	uint8_t			g;
	uint8_t			b;
} color_t;

static inline uint32_t color_scale_component(uint8_t color_component, uint8_t bits) {
	if (bits == 8) return color_component;
	return (color_component * 255 + ((1 << bits) - 1) / 2) / ((1 << bits) - 1);
}

uint32_t color_scale_rgb(
	uint8_t r,
	uint8_t g,
	uint8_t b,
	uint8_t bits_red,
	uint8_t shift_red,
	uint8_t bits_green,
	uint8_t shift_green,
	uint8_t bits_blue,
	uint8_t shift_blue
) {
	uint32_t scaled_r = color_scale_component(r, bits_red) << shift_red;
	uint32_t scaled_g = color_scale_component(g, bits_green) << shift_green;
	uint32_t scaled_b = color_scale_component(b, bits_blue) << shift_blue;
	return scaled_r | scaled_g | scaled_b;
}

size_t color_find_nearset_indexed(
	uint8_t r, uint8_t g, uint8_t b,
	const color_t* palette,
	size_t num_colors_in_palette
) {
	size_t best_distance2 = (size_t)-1;
	size_t best_color_index = 0;
	for (size_t i = 0; i < num_colors_in_palette; i++) {
		int r_distance = r - palette[i].r;
		int g_distance = g - palette[i].g;
		int b_distance = b - palette[i].b;
		size_t distance2 = r_distance * r_distance + g_distance * g_distance + b_distance * b_distance;

		if (distance2 <= best_distance2) {
			best_distance2 = distance2;
			best_color_index = i;
		}
	}

	return best_color_index;
}

/**
 * Only for indexed video mode with 8 bits per pixel
 */
void gfx_draw_pixel_i8(int x, int y, uint8_t r, uint8_t g, uint8_t b) {
	const video_mode_t* vm = &GFX_VIDEO_MODE;
	if (x < 0 || y < 0 || x >= (int)vm->width || y >= (int)vm->height) return;

	const size_t offset = y * vm->pitch + x;
	GFX_BUFFER[offset] = (uint8_t)color_find_nearset_indexed(
		r, g, b,
		GFX_PALETTE_I8,
		256
	);
}

/**
 * Only for non-index video mode with 15 or 16 bits per pixel (RGB)
 */
void gfx_draw_pixel555_565(int x, int y, uint8_t r, uint8_t g, uint8_t b) {
	const video_mode_t* vm = &GFX_VIDEO_MODE;
	if (x < 0 || y < 0 || x >= (int)vm->width || y >= (int)vm->height) return;

	uint32_t pixel = color_scale_rgb(
		r, g, b,
		vm->bits_red, vm->shift_red,
		vm->bits_green, vm->shift_green,
		vm->bits_blue, vm->shift_blue
	);

	const size_t offset = y * (vm->pitch >> 1) + x;
	((uint16_t*)GFX_BUFFER)[offset] = (uint16_t)pixel;
}

/**
 * Only for non-index video mode with 24 bits per pixel (RGB)
 */
void gfx_draw_pixel888(int x, int y, uint8_t r, uint8_t g, uint8_t b) {
	const video_mode_t* vm = &GFX_VIDEO_MODE;
	if (x < 0 || y < 0 || x >= (int)vm->width || y >= (int)vm->height) return;

	uint32_t pixel = color_scale_rgb(
		r, g, b,
		vm->bits_red, vm->shift_red,
		vm->bits_green, vm->shift_green,
		vm->bits_blue, vm->shift_blue
	);

	const size_t offset = y * vm->pitch + (x << 1) + x;
	GFX_BUFFER[offset] = pixel & 0xff;
	GFX_BUFFER[offset + 1] = (pixel >> 8) & 0xff;
	GFX_BUFFER[offset + 2] = (pixel >> 16) & 0xff;
}

/**
 * Only for non-index video mode with 32 bits per pixel (ARGB, A - reserved)
 */
void gfx_draw_pixel8888(int x, int y, uint8_t r, uint8_t g, uint8_t b) {
	const video_mode_t* vm = &GFX_VIDEO_MODE;
	if (x < 0 || y < 0 || x >= (int)vm->width || y >= (int)vm->height) return;

	const size_t offset = y * vm->pitch + (x << 2);
	GFX_BUFFER[offset] = b;
	GFX_BUFFER[offset + 1] = g;
	GFX_BUFFER[offset + 2] = r;
	GFX_BUFFER[offset + 3] = 0;
}

// mode #13h color palette
const color_t GFX_PALETTE_I8[256] = {
	{0, 0, 0},{0, 0, 141},{0, 141, 0},{0, 141, 141},
	{141, 0, 0},{141, 0, 141},{141, 70, 0},{141, 141, 141},
	{70, 70, 70},{70, 70, 211},{70, 211, 70},{70, 211, 211},
	{211, 70, 70},{211, 70, 211},{211, 211, 70},{211, 211, 211},

	{0, 0, 0},{13, 13, 13},{26, 26, 26},{44, 44, 44},
	{57, 57, 57},{70, 70, 70},{84, 84, 84},{97, 97, 97},
	{114, 114, 114},{127, 127, 127},{141, 141, 141},{154, 154, 154},
	{167, 167, 167},{185, 185, 185},{198, 198, 198},{211, 211, 211},
	{0, 0, 211},{54, 0, 211},{108, 0, 211},{157, 0, 211},
	{211, 0, 211},{211, 0, 157},{211, 0, 108},{211, 0, 54},
	{211, 0, 0},{211, 54, 0},{211, 108, 0},{211, 157, 0},
	{211, 211, 0},{157, 211, 0},{108, 211, 0},{54, 211, 0},
	{0, 211, 0},{0, 211, 54},{0, 211, 108},{0, 211, 157},
	{0, 211, 211},{0, 157, 211},{0, 108, 211},{0, 54, 211},

	{108, 108, 211},{131, 108, 211},{157, 108, 211},{185, 108, 211},
	{211, 108, 211},{211, 108, 185},{211, 108, 157},{211, 108, 131},
	{211, 108, 108},{211, 131, 108},{211, 157, 108},{211, 185, 108},
	{211, 211, 108},{185, 211, 108},{157, 211, 108},{131, 211, 108},
	{108, 211, 108},{108, 211, 131},{108, 211, 157},{108, 211, 185},
	{108, 211, 211},{108, 185, 211},{108, 157, 211},{108, 131, 211},
	{154, 154, 211},{167, 154, 211},{185, 154, 211},{198, 154, 211},
	{211, 154, 211},{211, 154, 198},{211, 154, 185},{211, 154, 167},
	{211, 154, 154},{211, 167, 154},{211, 185, 154},{211, 198, 154},
	{211, 211, 154},{198, 211, 154},{185, 211, 154},{167, 211, 154},
	{154, 211, 154},{154, 211, 167},{154, 211, 185},{154, 211, 198},
	{154, 211, 211},{154, 198, 211},{154, 185, 211},{154, 167, 211},

	{0, 0, 94},{23, 0, 94},{47, 0, 94},{70, 0, 94},{94, 0, 94},{94, 0, 70},{94, 0, 47},{94, 0, 23},
	{94, 0, 0},{94, 23, 0},{94, 47, 0},{94, 70, 0},{94, 94, 0},{70, 94, 0},{47, 94, 0},{23, 94, 0},
	{0, 94, 0},{0, 94, 23},{0, 94, 47},{0, 94, 70},{0, 94, 94},{0, 70, 94},{0, 47, 94},{0, 23, 94},

	{47, 47, 94},{57, 47, 94},{70, 47, 94},{80, 47, 94},{94, 47, 94},{94, 47, 80},{94, 47, 70},{94, 47, 57},
	{94, 47, 47},{94, 57, 47},{94, 70, 47},{94, 80, 47},{94, 94, 47},{80, 94, 47},{70, 94, 47},{57, 94, 47},
	{47, 94, 47},{47, 94, 57},{47, 94, 70},{47, 94, 80},{47, 94, 94},{47, 80, 94},{47, 70, 94},{47, 57, 94},
	{67, 67, 94},{74, 67, 94},{80, 67, 94},{87, 67, 94},{94, 67, 94},{94, 67, 87},{94, 67, 80},{94, 67, 74},
	{94, 67, 67},{94, 74, 67},{94, 80, 67},{94, 87, 67},{94, 94, 67},{87, 94, 67},{80, 94, 67},{74, 94, 67},
	{67, 94, 67},{67, 94, 74},{67, 94, 80},{67, 94, 87},{67, 94, 94},{67, 87, 94},{67, 80, 94},{67, 74, 94},

	{0, 0, 54},{13, 0, 54},{26, 0, 54},{41, 0, 54},{54, 0, 54},{54, 0, 41},{54, 0, 26},{54, 0, 13},
	{54, 0, 0},{54, 13, 0},{54, 26, 0},{54, 41, 0},{54, 54, 0},{41, 54, 0},{26, 54, 0},{13, 54, 0},
	{0, 54, 0},{0, 54, 13},{0, 54, 26},{0, 54, 41},{0, 54, 54},{0, 41, 54},{0, 26, 54},{0, 13, 54},

	{26, 26, 54},{33, 26, 54},{41, 26, 54},{47, 26, 54},{54, 26, 54},{54, 26, 47},{54, 26, 41},{54, 26, 33},
	{54, 26, 26},{54, 33, 26},{54, 41, 26},{54, 47, 26},{54, 54, 26},{47, 54, 26},{41, 54, 26},{33, 54, 26},
	{26, 54, 26},{26, 54, 33},{26, 54, 41},{26, 54, 47},{26, 54, 54},{26, 47, 54},{26, 41, 54},{26, 33, 54},
	{37, 37, 54},{41, 37, 54},{44, 37, 54},{50, 37, 54},{54, 37, 54},{54, 37, 50},{54, 37, 44},{54, 37, 41},
	{54, 37, 37},{54, 41, 37},{54, 44, 37},{54, 50, 37},{54, 54, 37},{50, 54, 37},{44, 54, 37},{41, 54, 37},
	{37, 54, 37},{37, 54, 41},{37, 54, 44},{37, 54, 50},{37, 54, 54},{37, 50, 54},{37, 44, 54},{37, 41, 54},
	
	{0, 0, 0},{0, 0, 0},{0, 0, 0},{0, 0, 0},{0, 0, 0},{0, 0, 0},{0, 0, 0},{0, 0, 0},
};
</syntaxhighlight>Now you can implement functions for reading and copying pixels.

Like that:<syntaxhighlight lang="c">
#include <gfx.h>

extern video_mode_t GFX_VIDEO_MODE;
extern uint8_t* GFX_BUFFER;

void gfx_copy_rectangle555_565(int src_x, int src_y, int dst_x, int dst_y, size_t width, size_t height) {
	const video_mode_t* vm = &GFX_VIDEO_MODE;
	if (src_x < 0) {
		width += src_x;
		src_x = 0;
	}

	if (src_y < 0) {
		height += src_y;
		src_y = 0;
	}

	if (dst_x < 0) {
		width += dst_x;
		src_x -= dst_x;
		dst_x = 0;
	}

	if (dst_y < 0) {
		height += dst_y;
		src_y -= dst_y;
		dst_y = 0;
	}

	if (src_x + width > vm->width) width = vm->width - src_x;
	if (dst_x + width > vm->width) width = vm->width - dst_x;
	if (src_y + height > vm->height) height = vm->height - src_y;
	if (dst_y + height > vm->height) height = vm->height - dst_y;

	if (!width || !height) return;

	int row_start, row_end, row_step;
	if (dst_y > src_y) {
		row_start = height - 1;
		row_end = -1;
		row_step = -1;
	} else {
		row_start = 0;
		row_end = height;
		row_step = 1;
	}

	for (int row = row_start; row != row_end; row += row_step) {
		uint16_t* src_line = (uint16_t*)(GFX_BUFFER + (src_y + row) * vm->pitch) + src_x;
		uint16_t* dst_line = (uint16_t*)(GFX_BUFFER + (dst_y + row) * vm->pitch) + dst_x;
		for (size_t col = 0; col < width; ++col) dst_line[col] = src_line[col];
	}
}

// ...

/**
 * Only for non-index video mode with 24 bits per pixel (RGB)
 */
void gfx_read_pixel888(int x, int y, uint8_t* r, uint8_t* g, uint8_t* b) {
	const video_mode_t* vm = &GFX_VIDEO_MODE;
	uint8_t cr;
	uint8_t cg;
	uint8_t cb;
	if (x < 0 || y < 0 || x >= (int)vm->width || y >= (int)vm->height) cr = cg = cb = 0;
	else {
		const size_t offset = y * vm->pitch + (x << 1) + x;
		cb = GFX_BUFFER[offset];
		cg = GFX_BUFFER[offset + 1];
		cr = GFX_BUFFER[offset + 2];
	}

	if (r) *r = cr;
	if (g) *g = cg;
	if (b) *b = cb;
}

// ...
</syntaxhighlight>Then, in order not to suffer with so many functions, when initializing the screen/graphics driver, you can define the desired function as a pointer, for example:<syntaxhighlight lang="c">
#include <gfx.h>

video_mode_t GFX_VIDEO_MODE;
uint8_t* GFX_BUFFER;

extern void gfx_draw_pixel_i8(int x, int y, uint8_t r, uint8_t g, uint8_t b);
extern void gfx_draw_pixel555_565(int x, int y, uint8_t r, uint8_t g, uint8_t b);
extern void gfx_draw_pixel888(int x, int y, uint8_t r, uint8_t g, uint8_t b);
extern void gfx_draw_pixel8888(int x, int y, uint8_t r, uint8_t g, uint8_t b);
extern void gfx_draw_pixel_stub(int x, int y, uint8_t r, uint8_t g, uint8_t b);

extern void gfx_read_pixel_i8(int x, int y, uint8_t* r, uint8_t* g, uint8_t* b);
extern void gfx_read_pixel555_565(int x, int y, uint8_t* r, uint8_t* g, uint8_t* b);
extern void gfx_read_pixel888(int x, int y, uint8_t* r, uint8_t* g, uint8_t* b);
extern void gfx_read_pixel8888(int x, int y, uint8_t* r, uint8_t* g, uint8_t* b);
extern void gfx_read_pixel_stub(int x, int y, uint8_t* r, uint8_t* g, uint8_t* b);

extern void gfx_copy_rectangle_i8(int src_x, int src_y, int dst_x, int dst_y, size_t width, size_t height);
extern void gfx_copy_rectangle555_565(int src_x, int src_y, int dst_x, int dst_y, size_t width, size_t height);
extern void gfx_copy_rectangle888(int src_x, int src_y, int dst_x, int dst_y, size_t width, size_t height);
extern void gfx_copy_rectangle8888(int src_x, int src_y, int dst_x, int dst_y, size_t width, size_t height);
extern void gfx_copy_rectangle_stub(int src_x, int src_y, int dst_x, int dst_y, size_t width, size_t height);

typedef void (*pgfx_draw_pixel)(int x, int y, uint8_t r, uint8_t g, uint8_t b);
typedef void (*pgfx_read_pixel)(int x, int y, uint8_t* r, uint8_t* g, uint8_t* b);
typedef void (*pgfx_copy_rectangle)(int src_x, int src_y, int dst_x, int dst_y, size_t width, size_t height);

pgfx_draw_pixel gfx_draw_pixel = NULL;
pgfx_read_pixel gfx_read_pixel = NULL;
pgfx_copy_rectangle gfx_copy_rectangle = NULL;

void gfx_init(video_mode_t* video_mode) {
	GFX_VIDEO_MODE = *video_mode;
	GFX_BUFFER = (uint8_t*)GFX_VIDEO_MODE.framebuffer;
	
	const video_mode_t* vm = &GFX_VIDEO_MODE;
	if (vm->depth == 8) {
		gfx_draw_pixel = gfx_draw_pixel_i8;
		gfx_read_pixel = gfx_read_pixel_i8;
		gfx_copy_rectangle = gfx_copy_rectangle_i8;
	}
	else if (vm->depth == 15 || vm->depth == 16) {
		gfx_draw_pixel = gfx_draw_pixel555_565;
		gfx_read_pixel = gfx_read_pixel555_565;
		gfx_copy_rectangle = gfx_copy_rectangle555_565;
	}
	else if (vm->depth == 24) {
		gfx_draw_pixel = gfx_draw_pixel888;
		gfx_read_pixel = gfx_read_pixel888;
		gfx_copy_rectangle = gfx_copy_rectangle888;
	}
	else if (vm->depth == 32) {
		gfx_draw_pixel = gfx_draw_pixel8888;
		gfx_read_pixel = gfx_read_pixel8888;
		gfx_copy_rectangle = gfx_copy_rectangle8888;
	}
	else {
		/*
			*_stub functions do nothing, they are needed
			as placeholders if we have not defined a known video mode
		*/
		
		gfx_draw_pixel = gfx_draw_pixel_stub;
		gfx_read_pixel = gfx_read_pixel_stub;
		gfx_copy_rectangle = gfx_copy_rectangle_stub;
	}
}
</syntaxhighlight>Now you can simply use the gfx_draw_pixel pointer as a function (same for gfx_read_pixel, gfx_copy_rectangle)

==See Also==
*[[VGA Hardware]] - if you want to do it on your own
*[[PC Screen Font]] - wiki has a tutorial on how to display them
*[[Scalable Screen Font]] - comes with a small, free ANSI C rendering library
*[[TrueType Fonts]] - a very complex vector font format, partially proprietary

==External Links==
*[http://www.inp.nsk.su./~bolkhov/files/fonts/univga/ UNI-VGA] - A free Unicode VGA font (.bdf)
*[[squeak:projects/bdf2c/|bdf2c]] - .bdf font to C source converter.
*[https://github.com/K1tsun1x/LzPsfEditor.git LzPsfEditor] - "lazy" psf font editor (font can be saved as PSF1/PSF2(?), C array with/without character indexes)
*[https://github.com/K1tsun1x/LzPsfEditor/tree/master/LzPsfEditor/FontExamples LzPsfEditor/FontExamples] - .psf fonts from /usr/share/consolefonts (PSF1)

[[Category:VGA]]
[[Category:Video]]
[[Category:Fonts]]
````

<a id="w003093"></a>

## W003093 — Template:Video warning

Bron-ID: W003093
Namespace: Template (10)
Revisie: 12610 | Datum: 2012-02-16T13:37:18Z
Bron: https://wiki.osdev.org/Template:Video_warning
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AVideo_warning&oldid=12610

````text
<center>
{|style="border: 1px solid #bfcfcf; padding: .0em .25em .0em; background-color: #fff0f0; text-align: center;"
|
<font color="black">'''WARNING: Improperly changing CRTC or hardware settings can be harmful to the video card and attached monitor.''' Configuring graphics hardware may be a rewarding process, and many have achieved their goals with success and have even tampered with them well beyond their original specifications. Fact remains that there are ''several'' known instances of monitors - CRT and LCD alike - burning out after being fed poor data. Similarly, the video card might lack similar safeguards against accidental misconfiguration. 

'''DISCLAIMER: The information provided might not be accurate, and using it is to be done entirely at your own risk'''. There have not been any reported mishaps in the entire history of OSDev.org, but in the odd chance you break something, we are [[OSDev_Wiki:Copyrights|not responsible]].</font>
|
|}
</center>
````

<a id="w003095"></a>

## W003095 — SSE2

Bron-ID: W003095
Namespace: Main (0)
Revisie: 12804 | Datum: 2012-02-22T20:27:03Z
Bron: https://wiki.osdev.org/SSE2
Revisie-URL: https://wiki.osdev.org/index.php?title=SSE2&oldid=12804
Doorverwijzing: SSE

````text
#REDIRECT [[SSE#SSE2]]
````

<a id="w003096"></a>

## W003096 — Template:Floats

Bron-ID: W003096
Namespace: Template (10)
Revisie: 20106 | Datum: 2017-01-06T15:48:45Z
Bron: https://wiki.osdev.org/Template:Floats
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AFloats&oldid=20106

````text
{{SmallNavBox
| width:200px;
|Name=Real numbers, coprocessors and vector units
|Group0=Technical
|List0=<div>
[[Floating Point Number]]
</div>

|Group1=X86 implementations
|List1=<div>
[[FPU|X87 FPU]]<br>
[[MMX]]<br>
[[SSE]]<br>
[[AVX]]<br>
</div>}}
````

<a id="w003097"></a>

## W003097 — Floating Point Number

Bron-ID: W003097
Namespace: Main (0)
Revisie: 30995 | Datum: 2026-09-30T17:10:30Z
Bron: https://wiki.osdev.org/Floating_Point_Number
Revisie-URL: https://wiki.osdev.org/index.php?title=Floating_Point_Number&oldid=30995

````text
'''This page is about standard implementations and the concept. For platform information, please see the relevant page'''
{{Floats}}
==Introduction==

Floating point numbers are a way to represent [https://en.wikipedia.org/wiki/Real_number ''real numbers''] inside computer memory which (usually) operates with binary digits. As opposed to fixed point numbers which have a fixed number of digits before and after the [https://en.wikipedia.org/wiki/Decimal_mark decimal mark], floating point numbers ''can be considered'' to have a certain amount of [https://en.wikipedia.org/wiki/Significant_figures significant digits] or 'accurate leading numbers' that we consider to carry an accurate approximation to some value. ''Floating point numbers are inherently not a precise storage medium precisely because they store an approximation to a value and not the precise value itself'' This requires care when writing code both in low or high-level languages to avoid running into precision errors which may break programs.

Because floating point numbers have a different representation than integers, they can not be treated as ordinary integers with ordinary instructions. Luckily, modern processors come with a dedicated on-chip [[FPU]] (Floating Point Unit) co-processor that quickly performs floating point calculations in hardware. On x86 the FPU and it's instructions are named 'x87' because of historical reasons. Embedded devices may or may not come with an FPU, depending on the complexity and price range of these devices. Check the spec! (Before FPUs became an affordable upgrade for desktop computers, floating point emulation libraries would be either present or installable on computers. Sometimes these libraries even required manual installation by the end user!)

This article will explain the IEEE 754 standard for floating point number representation. This standard is extremely common, not x86-specific and has multiple 'precisions' or representation sizes which are all similar in how they work except for the number of bits used.

==Representation==
All floating point numbers are a variation on the theme of x = m * b^e, that is, they all save some kind of mantissa that is value limited to be between 1 and the base, and an exponent, and the whole number is formed by raising a constant base b to the exponent and then multiplying in the mantissa.

In most implementations, the base will be 2 or 10. But the latter is called "decimal float", and is a bit of an outlier. There are a lot of software implementations for it, but on hardware it was only implemented on the IBM POWER6 and newer, the IBM System z9 and newer, as well as the Fujitsu Sparc 64. However, most of these also maintain compatibility to "binary float" (that is what base 2 floating point is called), and almost all hardware with FP support will support binary float, including PCs.

Therefore the rest of this page will focus on IEEE 754 binary floating point.

===IEE 754===
This standard standardized the format of binary floating point numbers: All FP numbers are bit structures, with one sign bit in the highest place, then a collection exponent bits, and then the rest of the number being the mantissa. Since bits can only form unsigned integers, interpretation is necessary.

If all exponent bits are set, then this value is special. If the mantissa is zero, then this means the value is infinite, else the value is not-a-number (NaN). On platforms where this is important, the highest mantissa bit selects between quiet NaN and signaling NaN. Use of the latter results in an exception, but the PC hardware does not support this behavior.

Otherwise the exponent is saved as a biased integer. That is, we define a number, called the bias, which is all bits set, and one bit shorter than the exponent, and the exponent saved in the FP number is the actual exponent plus this bias. This allows saving negative values without two's complement or anything.

Since in binary float, the mantissa has to be between 1 and 2, the "1." leading the mantissa is implicit. The rest of the mantissa is the first bits behind the decimal point. Therefore, if the number is interpreted as integer, it has to be divided by 2 raised to the bit width of the mantissa. Then add 1. Only special case is, if the exponent is all 0 then the implicit integer part is 0 instead of 1. Such numbers are not in normal form, and are therefore called "subnormal".

====Original (1985)====
{| {{wikitable}}
|-
!level
!width
!range
!precision
!exponent width
!mantissa width
|-
|single precison
|32 bits
|±1.18x10^-38 to ±3.4x10^38
|approx 7 digits
|8 bits
|23 bits
|-
|double precison
|64 bits
|±2.23x10^-308 to ±1.8x10^308
|approx 15 digits
|11 bits
|52 bits
|-
|double extended precision
|79 bits
|±3.26x10^-4932 to ±1.19x10^4932
|approx 19 digits
|15 bits
|63 bits
|}

Note that the values for double extended precision are minimums. x87 implements this type as 80 bit data type, and adds an explicit integer bit as highest bit of the mantissa.
====2008====
{| {{wikitable}}
|-
!level
!width
!range
!precision
|-
|half precison
|16 bits
|±4.8x10^-4 to ±3.24x10^4
|approx 7 digits
|-
|quad precision
|128 bits
|±3.36x10^-4932 to ±1.19x10^4932
|approx 34 digits
|}
== Conversions ==
The code examples in the following sections will be valid C99, but not necessarily valid C++. C++ might require the use of <tt>memcpy()</tt>. It also assumes that FPU endianess equals CPU endianess. The examples use hexadecimal floating-point constants, which are features of C99 and C++17.
=== Integer conversion ===
It is occasionally necessary to convert integers to floating point numbers and back again. Some architectures have no hardware support for that. While soft-float methods could be used to directly create the floating-point representation of an integer, it is simpler, shorter, and faster to create the floating-point representation of some number that is connected to the input and calculate the correct output from there.

In the case of the conversion of an unsigned 32-bit integer into a double-precision number, we can see that the output format has a 52-bit significand. If x is the input, then the number 2^52 + x has as its representation x in the low word, and the high word of all bits zero and a fixed exponent part of 0x433 (which represents $2^52$). And from that we can calculate the correct output just be subtracting the offset. So in C, the idea can be expressed as:
<source lang="c">double utof(uint32_t x) {
  union {uint64_t i; double d;} u;
  u.i = x | ((uint64_t)0x433 << 52);
  return u.d - 0x1p52;
}</source>

For a signed 32-bit number, the input may be negative, but if the input is offset by 2^31, it will be nonnegative. Then after the conversion, that same offset has to be removed from the output number as well. The adding can in this case be expressed as an XOR instruction, since it only affects a single bit.
<source lang="c">double itof(int32_t x) {
  union {uint64_t i; double d;} u;
  u.i = (((uint32_t)x) ^ 0x80000000) | ((uint64_t)0x433 << 52);
  return u.d - 0x1.000008p52;
}</source>

In an unsigned 64-bit number, the same conversion can be done on the upper and lower half separately, the upper half getting an exponent of 84, which is 32 more than 52. Then the numbers can be added and their combined offsets removed. If the order of operations is correct, then no digits of the final number will be shifted out.
<source lang="c">double ulltof(uint64_t x) {
  union {uint64_t i; double d;} hi, lo;
  hi.i = (x >> 32) | ((uint64_t)0x453 << 52);
  lo.i = (x & 0xFFFFFFFF) | ((uint64_t)0x433 << 52);
  return hi.d - 0x1.00000001p84 + lo.d;
}</source>

Finally, for a signed 64-bit number, both ideas presented before combine. A signed 64-bit number can be sign as a multi-word number, comprised of two 32-bit numbers of which the upper word is signed, and the lower word unsigned. So it can be converted into a double-precision number like this:
<source lang="c">double lltof(int64_t x) {
  union {uint64_t i; double d;} hi, lo;
  hi.i = ((((uint64_t)x) >> 32) ^ 0x80000000) | ((uint64_t)0x453 << 52);
  lo.i = (x & 0xFFFFFFFF) | ((uint64_t)0x433 << 52);
  return hi.d - 0x1.00000801p84 + lo.d;
}</source>

== See Also ==

=== Articles ===
* [[FPU|x87 FPU]]
* [[MMX]]
* [[SSE]]

=== Threads ===
* [http://forum.osdev.org/viewtopic.php?f=13&p=239270 Float to String Idea] - also contains some good general information about floating-point numbers

[[Category:Stubs]]
[[Category:X86]]

=== External Links ===
* [http://www.h-schmidt.net/FloatConverter/IEEE754.html Handy online javascript applet to see how single precision IEEE 754 floats work]
* [https://standards.ieee.org/ieee/754/6210/ IEEE 754 spec]
````

<a id="w003098"></a>

## W003098 — SSE3

Bron-ID: W003098
Namespace: Main (0)
Revisie: 12805 | Datum: 2012-02-22T20:27:44Z
Bron: https://wiki.osdev.org/SSE3
Revisie-URL: https://wiki.osdev.org/index.php?title=SSE3&oldid=12805
Doorverwijzing: SSE

````text
#REDIRECT [[SSE#SSE3]]
````

<a id="w003099"></a>

## W003099 — SSE4

Bron-ID: W003099
Namespace: Main (0)
Revisie: 12806 | Datum: 2012-02-22T20:28:40Z
Bron: https://wiki.osdev.org/SSE4
Revisie-URL: https://wiki.osdev.org/index.php?title=SSE4&oldid=12806
Doorverwijzing: SSE

````text
#REDIRECT [[SSE#SSE4]]
````

<a id="w003100"></a>

## W003100 — MMX

Bron-ID: W003100
Namespace: Main (0)
Revisie: 29873 | Datum: 2026-01-12T03:39:52Z
Bron: https://wiki.osdev.org/MMX
Revisie-URL: https://wiki.osdev.org/index.php?title=MMX&oldid=29873

````text
{{Floats}}
{{stub}}
=Overview=
MMX is a SIMD technology (single instruction, multiple data) introduced by Intel on January 8, 1997 with their P5 "Pentium" processor line named "Pentium with MMX Technology". It has been superseded by [[SSE]] and now [[AVX]].

==Technical Details==

MMX is accessed using 8 CPU registers (MM0 to MM7). Each register is 64 bits wide and can be used to hold 64-bit integers or multiple smaller integers packed. One instruction can be applied to two 32-bit integers, four 16-bit integers, or 8 8-bit integers at once.

==Detection==

The bit for MMX can be found on CPUID page 1, in EDX bit 23.

= Current status =
MMX has been long made obsolete by the [[SSE]] and [[AVX]] SIMD extensions, however for compatibility modern x86 CPUs still support this instruction set.
````

<a id="w003101"></a>

## W003101 — VESA BIOS Extension/Accelerator Functions

Bron-ID: W003101
Namespace: Main (0)
Revisie: 28042 | Datum: 2023-07-09T18:59:21Z
Bron: https://wiki.osdev.org/VESA_BIOS_Extension/Accelerator_Functions
Revisie-URL: https://wiki.osdev.org/index.php?title=VESA_BIOS_Extension%2FAccelerator_Functions&oldid=28042

````text
{{Stub}}


VBE/AF or VESA BIOS Extension/Accelerator Functions is a standard defined  by Video Electronic Standard Association in 1996 to allow software to access the accelerator features present in most graphics hardware. Unfortunately, unlike VBE/Core, VBE/AF has never seen many implementations. Furthermore, most of those implementations were software-based (via DOS TSR programs). However, one might still be able to find some old video cards supporting VBE/AF.  The [https://shawnhargreaves.com/freebe/ FreeBE/AF] project provided free VBE/AF drivers for certain cards.

[[Category:Video]]
[[Category:Standards]]
````

<a id="w003102"></a>

## W003102 — Category talk:SSE

Bron-ID: W003102
Namespace: Category talk (15)
Revisie: 12668 | Datum: 2012-02-19T07:35:48Z
Bron: https://wiki.osdev.org/Category_talk:SSE
Revisie-URL: https://wiki.osdev.org/index.php?title=Category_talk%3ASSE&oldid=12668

````text
I propose merging the two articles in this category an the one on MMX into a single article called SIMD (x86). --[[User:Love4boobies|Love4boobies]] 01:35, 19 February 2012 (CST)
````

<a id="w003103"></a>

## W003103 — Category:Time

Bron-ID: W003103
Namespace: Category (14)
Revisie: 12672 | Datum: 2012-02-19T18:50:37Z
Bron: https://wiki.osdev.org/Category:Time
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ATime&oldid=12672

````text
Time is an essential concept in your OS. How you represnet it is upt to you, but take a look at some ideas.
[[Category:OS theory]]
````

<a id="w003104"></a>

## W003104 — Category:Text UI

Bron-ID: W003104
Namespace: Category (14)
Revisie: 22821 | Datum: 2018-09-10T05:40:18Z
Bron: https://wiki.osdev.org/Category:Text_UI
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AText_UI&oldid=22821

````text
This category is about text UI's (a.k.a Command Line Interface)
[[Category:OS theory]]
[[Category:UI]]
````

<a id="w003105"></a>

## W003105 — User talk:Turdus

Bron-ID: W003105
Namespace: User talk (3)
Revisie: 13214 | Datum: 2012-03-30T20:15:39Z
Bron: https://wiki.osdev.org/User_talk:Turdus
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ATurdus&oldid=13214

````text
== 2 stage bootloader ==

Please, when you create a page that should so obviously be in the bootloaders category, place it in that category. A nice piece though. -- comment by [[User:Bellezzasolo|Bellezzasolo]] 19:19, 19 February 2012‎

:The sentiment was that this might belong on a project page of itself. The sentiment was ''not'' that you should flip the finger like that. I'd like to see a link to that bootloader popping up here in a couple of days. -- [[User:Solar|Solar]] 08:50, 30 March 2012 (CDT)
````

<a id="w003106"></a>

## W003106 — Category:Object Files

Bron-ID: W003106
Namespace: Category (14)
Revisie: 12690 | Datum: 2012-02-19T19:25:54Z
Bron: https://wiki.osdev.org/Category:Object_Files
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AObject_Files&oldid=12690

````text
This is a category about object files

[[Category:Executable Formats]]
````

<a id="w003107"></a>

## W003107 — Category:ACPI

Bron-ID: W003107
Namespace: Category (14)
Revisie: 12696 | Datum: 2012-02-19T19:32:15Z
Bron: https://wiki.osdev.org/Category:ACPI
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AACPI&oldid=12696

````text
ACPI is the current power management standard
[[Category:Power management]]
````

<a id="w003108"></a>

## W003108 — SSDT

Bron-ID: W003108
Namespace: Main (0)
Revisie: 28197 | Datum: 2023-07-10T17:51:46Z
Bron: https://wiki.osdev.org/SSDT
Revisie-URL: https://wiki.osdev.org/index.php?title=SSDT&oldid=28197

````text
{{ACPI}}

''This page is about the ACPI SSDT (Secondary System Descriptor Table)''
It contains additional AML to interact with devices.
==SSDT==
===Introduction===
{{stub}}
The SSDT is an ACPI decriptor table. It is encoded in AML in exactly the same way as the [[DSDT]]. It acts as a supplement to the DSDT.
==See Also==
===Articles===
*[[DSDT]]
*[[RSDT]]

[[Category:ACPI]]
[[Category:Stubs]]
````

<a id="w003110"></a>

## W003110 — Talk:Required Knowledge

Bron-ID: W003110
Namespace: Talk (1)
Revisie: 12756 | Datum: 2012-02-20T15:31:24Z
Bron: https://wiki.osdev.org/Talk:Required_Knowledge
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ARequired_Knowledge&oldid=12756

````text
Isn't the "RTFM" statement obscene? -- (by Ammubhave)

:[http://en.wikipedia.org/wiki/RTFM So what?] Besides, you might want to note that the exact definition of the "F" has been deliberately left out, so you might interpret it as an [http://en.wikipedia.org/wiki/RTFM#List_of_expurgated_expansions expurgated] version. -- [[User:Solar|Solar]] 09:31, 20 February 2012 (CST)
````

<a id="w003111"></a>

## W003111 — DSDT

Bron-ID: W003111
Namespace: Main (0)
Revisie: 30291 | Datum: 2026-03-16T19:00:48Z
Bron: https://wiki.osdev.org/DSDT
Revisie-URL: https://wiki.osdev.org/index.php?title=DSDT&oldid=30291

````text
{{ACPI}}

''This page is about the ACPI DSDT (Differentiated System Description Table)''
It describes devices (I/O Ports, IRQs, Memory Mappings, etc.).
==DSDT==
===Introduction===
{{stub}}
DSDT stands for ''Differentiated System Description Table''. It Is a major [[ACPI]] table and is used to describe what peripherals the machine has. Also holds information on PCI IRQ mappings and power management. For example when powering down by the OS, it should find the _S5 object which describes how to do that. It is pointed to by the [[FADT]].
<br />
<br />
As every ACPI table, it has header section and '''should''' be validated ([https://wiki.osdev.org/RSDT#Validating_the_RSDT ACPI table validation]).
===How hardware manufacturers generate it===
You can use an ASL (ACPI Source Language) compiler to generate DSDT AML (ACPI Machine Language) bytecode from a C-like language. To get an idea, take a glimpse on Bochs source:
* bios/acpi-dsdt.dsl for human readable source
* bios/acpi-dsdt.hex for hex dump of the bytecode output
For example here's the description of the [[RTC|Real Time Clock]]:
<syntaxhighlight lang="c">
	/* PIIX3 ISA bridge */
        Device (ISA) {
            Name (_ADR, 0x00010000)

            /* PIIX PCI to ISA irq remapping */
            OperationRegion (P40C, PCI_Config, 0x60, 0x04)

            /* Real-time clock */
            Device (RTC)
            {
                Name (_HID, EisaId ("PNP0B00"))
                Name (_CRS, ResourceTemplate ()
                {
                    IO (Decode16, 0x0070, 0x0070, 0x10, 0x02)
                    IRQNoFlags () {8}
                    IO (Decode16, 0x0072, 0x0072, 0x02, 0x06)
                })
            }
</syntaxhighlight>
and what's it became after compilation:
<pre>
    0x00,0xA4,0x00,0x5B,0x82,0x42,0x23,0x49,  /* 00000270    "...[.B#I" */
    0x53,0x41,0x5F,0x08,0x5F,0x41,0x44,0x52,  /* 00000278    "SA_._ADR" */
    0x0C,0x00,0x00,0x01,0x00,0x5B,0x80,0x50,  /* 00000280    ".....[.P" */
    0x34,0x30,0x43,0x02,0x0A,0x60,0x0A,0x04,  /* 00000288    "40C..`.." */
    0x5B,0x82,0x2D,0x52,0x54,0x43,0x5F,0x08,  /* 00000290    "[.-RTC_." */
    0x5F,0x48,0x49,0x44,0x0C,0x41,0xD0,0x0B,  /* 00000298    "_HID.A.." */
    0x00,0x08,0x5F,0x43,0x52,0x53,0x11,0x18,  /* 000002A0    ".._CRS.." */
    0x0A,0x15,0x47,0x01,0x70,0x00,0x70,0x00,  /* 000002A8    "..G.p.p." */
    0x10,0x02,0x22,0x00,0x01,0x47,0x01,0x72,  /* 000002B0    ".."..G.r" */
    0x00,0x72,0x00,0x02,0x06,0x79,0x00,0x5B,  /* 000002B8    ".r...y.[" */
    0x82,0x44,0x04,0x4B,0x42,0x44,0x5F,0x08,  /* 000002C0    ".D.KBD_." */
</pre>
Most likely you will never generate such a binary. But it helps if you know it's source form when you came to decode.

===Purpose of DSDT===
When your OS boots, it should parse the memory for [[ACPI]] tables. Then locate DSDT (and other tables as well, like [[SSDT]]), and decode it to get the list of installed devices. If you have that list, it's rather easy to load [[Device_Driver_Interfaces|device driver]] for each.

Also note that there are buggy tables, so you should always keep the possibility to load DSDT data from a user provided file instead. This file could be located in your [[initrd|initial ramdisk]], loaded with your kernel along on boot. That would solve the chicken-egg problem of loading the DSDT file from a device that's IO addresses are defined in the DSDT.

==See Also==
===Articles===
*[[FADT]]
*[[ACPI]]
*[[Shutdown]]
*[[AML]]

===External Links===
*[http://www.acpica.org/documentation/ ACPICA documentation] offical documentation
*[http://www.acpica.org/downloads/ ACPI tools] like iasl compiler and acpixtract, source also available
 
[[Category:Stubs]]
[[Category:ACPI]]
````

<a id="w003112"></a>

## W003112 — File talk:Difficulty 2.png

Bron-ID: W003112
Namespace: File talk (7)
Revisie: 12798 | Datum: 2012-02-22T19:35:19Z
Bron: https://wiki.osdev.org/File_talk:Difficulty_2.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File_talk%3ADifficulty_2.png&oldid=12798

````text
I prefer the previous version - [[User:Bellezzasolo|Bellezzasolo]] (Please sign your messages with ~ ~ ~ ~)

I don't. There's nothing prohibitive in green. - [[User:Combuster|Combuster]] 13:35, 22 February 2012 (CST)
````

<a id="w003114"></a>

## W003114 — Floats

Bron-ID: W003114
Namespace: Main (0)
Revisie: 12808 | Datum: 2012-02-22T20:30:24Z
Bron: https://wiki.osdev.org/Floats
Revisie-URL: https://wiki.osdev.org/index.php?title=Floats&oldid=12808
Doorverwijzing: Floating Point Number

````text
#REDIRECT [[Floating Point Number]]
````

<a id="w003115"></a>

## W003115 — Talk:Print Functions

Bron-ID: W003115
Namespace: Talk (1)
Revisie: 12814 | Datum: 2012-02-23T09:16:48Z
Bron: https://wiki.osdev.org/Talk:Print_Functions
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APrint_Functions&oldid=12814

````text
== Bad Style... ==

Apparently, you haven't really understood the point of my criticism over at [[Printing to Screen]]. Putting it in a separate page doesn't help. I am sorry for being so blunt, but the quality of your source simply does not satisfy as tutorial / example code. It's undercommented. It has lots of unwieldly constructs (<tt>char null = '\0'</tt>???). Functions named ''almost'' like standard ones that aren't (<tt>Puts()</tt> and the lack of a newline). Global, externally visible variables and '''goto?''' I mean, really? Calling undefined functions (<tt>disable()</tt> / <tt>enable()</tt>). I'd probably find more if I cared to look further. That's lots of red ink for 100-something lines of code, and certainly too much for this to reside in our Wiki for others to copy & paste, IMNSHO. -- [[User:Solar|Solar]] 02:54, 23 February 2012 (CST)

:Plus, the overall gist of this Wiki (and the forum) is to '''not''' provide blocks of code to copy & paste, but to give the information necessary to do it ''yourself''. For example, the information that 0xb8000 is the beginning of video memory is eligible. The information that the first byte is the ASCII code and the second byte is the color is eligible. The information which value in the color byte corresponds to which color is eligible. Links to existing implementations are eligible. Posting code for <tt>putc()</tt>, <tt>puts()</tt> and <tt>printf()</tt> is ''not'' eligible. -- [[User:Solar|Solar]] 03:16, 23 February 2012 (CST)
````

<a id="w003116"></a>

## W003116 — Talk:Floating Point Number

Bron-ID: W003116
Namespace: Talk (1)
Revisie: 27469 | Datum: 2022-10-21T22:56:22Z
Bron: https://wiki.osdev.org/Talk:Floating_Point_Number
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AFloating_Point_Number&oldid=27469

````text
Erm... what's the point of this article, really? We have [[FPU]] as well as [[SSE]], and I would label knowledge about [http://en.wikipedia.org/wiki/IEEE_754-2008 IEEE-754] as [[Required Knowledge]]...? -- [[User:Solar|Solar]] 09:17, 23 February 2012 (CST)

The point is that it provides platform independent information on the subject. It is more a techincal resoruce than an implementation.
It also provides a base page for the floats template
--[[User:Bellezzasolo|Bellezzasolo]] 07:11, 27 February 2012 (CST)

== Floating point is OKAY to use in kernel ==

I don't know why people say it is not okay to use floating point in kernel, it is totally fine as long as you setup the floating point units correctly; article should make it succintly clear -[[User:Wxwisiasdf|Wxwisiasdf]] 17:56, 21 October 2022 (CDT)
````

<a id="w003117"></a>

## W003117 — Talk:MMX

Bron-ID: W003117
Namespace: Talk (1)
Revisie: 12821 | Datum: 2012-02-23T15:33:57Z
Bron: https://wiki.osdev.org/Talk:MMX
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMMX&oldid=12821

````text
== Factuality. ==

MMX predates 3DNow!. Since discontinuation of 3DNow! has been announced, I am not certain we should mention it at all... -- [[User:Solar|Solar]] 09:33, 23 February 2012 (CST)
````

<a id="w003118"></a>

## W003118 — ATA PIO

Bron-ID: W003118
Namespace: Main (0)
Revisie: 12829 | Datum: 2012-02-25T08:05:22Z
Bron: https://wiki.osdev.org/ATA_PIO
Revisie-URL: https://wiki.osdev.org/index.php?title=ATA_PIO&oldid=12829
Doorverwijzing: ATA PIO Mode

````text
#REDIRECT [[ATA PIO Mode]]
````

<a id="w003120"></a>

## W003120 — Definitions

Bron-ID: W003120
Namespace: Main (0)
Revisie: 12849 | Datum: 2012-02-28T10:35:01Z
Bron: https://wiki.osdev.org/Definitions
Revisie-URL: https://wiki.osdev.org/index.php?title=Definitions&oldid=12849
Doorverwijzing: Chip Numbers, Acronyms and Things

````text
#REDIRECT [[Chip_Numbers,_Acronyms_and_Things]]
````

<a id="w003121"></a>

## W003121 — Open Watcom

Bron-ID: W003121
Namespace: Main (0)
Revisie: 12868 | Datum: 2012-03-01T15:00:36Z
Bron: https://wiki.osdev.org/Open_Watcom
Revisie-URL: https://wiki.osdev.org/index.php?title=Open_Watcom&oldid=12868
Doorverwijzing: Watcom

````text
#REDIRECT [[Watcom]]
````

<a id="w003122"></a>

## W003122 — Volume Boot Record

Bron-ID: W003122
Namespace: Main (0)
Revisie: 18540 | Datum: 2015-11-13T12:17:44Z
Bron: https://wiki.osdev.org/Volume_Boot_Record
Revisie-URL: https://wiki.osdev.org/index.php?title=Volume_Boot_Record&oldid=18540

````text
==VBR==
===Introduction===
A '''Volume Boot Record''' ('''VBR''') is the first sector of a partition (unlike [[MBR]] which is the first sector of a hard disk). A VBR (just like a MBR) also contains some code and data, but it's far less standard. Instead of a [[Partition Table]] it usually holds some file system information, like the [[FAT#BPB_.28BIOS_Parameter_Block.29 | BIOS Parameter Block]] (although it's not needed nowadays, many OS's have kept it for compatibility). The code is always OS specific, but in common all versions function the same: locate the kernel on the partition, load and execute it. A really good example of a VBR is the original ''DOS bootsector'', which knew [[FAT]] and loaded IO.SYS and MSDOS.SYS from the root directory.

===Implementations===
This is file system specific, but the usual is a JMP followed by FS info.

==See also==
===Booting===
*[[MBR]]
* [[Boot Sequence]]

[[Category:Bootloaders]]
[[de:Bootsektor]]
````

<a id="w003124"></a>

## W003124 — User:Bellezzasolo/Old talk

Bron-ID: W003124
Namespace: User (2)
Revisie: 12899 | Datum: 2012-03-05T13:13:13Z
Bron: https://wiki.osdev.org/User:Bellezzasolo/Old_talk
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ABellezzasolo%2FOld_talk&oldid=12899

````text
==Talk==
If you want to talk to me, go here: [[User talk:Bellezzasolo]]

==My Old discussions==
Here are old discussions relegated from my talk page
===IDT problems===
As I wrote on the IDT Problems talk page, I don't think this is the right place to discuss "personal" problems, and you'd better post your questions on the forum. Anyway. I think your code has two problems. First, I can't see any setvect call. You wrote the function but is it actually called somewhere? Second you put #pragma pack and pop around the IDT_register and IDT variable instantiations, while you have to put it around the class definitions like:
<source lang="cpp">
 #pragma pack(push, 1)
 class IDT_entry
 {
     // blah blah blah
 };
 #pragma pack(pop)
</source>

You put size asserts in the installIDT function "if(sizeof(IDT_reg) != 6) ......" but even if it actually does Puts("Bad IDT reg\n"); you wont actually see that, as you system will reboot before your eyes are able to catch it. I suggest you to write the asserts like this:
<source lang="cpp">
 if(sizeof(IDT_reg) != 6)
 {
     Puts("Bad IDT reg\n");
     Halt();
 }
</source>

Where Halt is an infinite loop or a "cli; hlt" so that you can actually see the message being put.



Thankyou for your help. I shall place open problems on the forum and solved problems on the wiki page, where people may find it of benefit, with a link to the forum. As for your questions:
<source lang="cpp">
setvect(0, &handlerDefault); //through to 0x13
</source>
This is in HAL.cpp.

Anyway, thankyou for your time. I shal put this into practice ASAP

--[[User:Bellezzasolo|Bellezzasolo]] 07:12, 6 February 2012 (CST)

Update: I am still having trouble. The same error message.

Update: I aligned my GDT like this and it worked. I now have an IRQ driven OS

==See also==
===Me===
*[[User:Bellezzasolo]]
*[[User talk:Bellezzasolo]]
===Related articles===
*[[IDT problems]]
````

<a id="w003126"></a>

## W003126 — GPT

Bron-ID: W003126
Namespace: Main (0)
Revisie: 27892 | Datum: 2023-07-09T04:40:45Z
Bron: https://wiki.osdev.org/GPT
Revisie-URL: https://wiki.osdev.org/index.php?title=GPT&oldid=27892

````text
GPT stands for GUID Partition Table. It is made to replace [[Partition Table|MBR partitioning]].
== Layout ==

A GPT partitioned media looks like this:

{| {{wikitable}}
|LBA 0||Protective Master Boot Record (PMBR). Holds a partition pointing to GPT to avoid accidental overwrite by old programs.
|-
|LBA 1||partition header, can be identified by 8 bytes magic "EFI PART" (45h 46h 49h 20h 50h 41h 52h 54h)
|-
|LBA 2..33||partition table entries
|-
| colspan="2" | ...data on disk...
|-
|LBA -33..-2||mirror of partition table
|-
|LBA -1||mirror of partition header on last addressable sector
|}

=== LBA 0: Protective Master Boot Record ===

This is kept untouched for backward compatibility. The UEFI specification requires that the PMBR partition table contain one partition record, with the other three partition records set to zero. The partition record should be defined as follows:

{| {{wikitable}}
|'''Offset'''||'''Length (bytes)'''||'''Description'''||'''Value'''
|-
|0x0||1||Boot Indicator||Set to 0x00 to indicate a non-bootable partition.
|-
|0x1||3||Starting CHS||Set to 0x000200, corresponding to the Starting LBA field.
|-
|0x4||1||OS Type||Set to 0xEE (GPT Protective)
|-
|0x5||3||Ending CHS||Set to the CHS address of the last logical block on the disk. Set to 0xFFFFFF if the value cannot be represented in this field.
|-
|0x8||4||Starting LBA||Set to 0x00000001 (LBA of GPT Partition Header)
|-
|0xC||4||Ending LBA||Set to the size in logical blocks of the disk, minus one. Set to 0xFFFFFFFF if the size of the disk is too large to be represented in this field.
|}

The PMBR will always exist if the drive contains a GPT, as it is required in order to maintain compatibility with tools expecting an MBR partition table. It's also worth noting that UEFI firmware implementations are also required to parse legacy MBR partition tables in the event that the drive does not contain a GPT.

=== LBA 1: Partition Table Header ===

Right after the first sector, there's a special sector to identify GPT. This sector is repeated in the last sector of the media (pointed by the field at 0x20).

{| {{wikitable}}
|'''Offset'''||'''Length (bytes)'''||'''Description'''
|-
|0x0||8||Signature, can be identified by 8 bytes magic "EFI PART" (45h 46h 49h 20h 50h 41h 52h 54h)
|-
|0x8||4||GPT Revision
|-
|0xC||4||Header size
|-
|0x10||4||CRC32 checksum of the GPT header (0x0 to 0x5c)
|-
|0x14||4||Reserved
|-
|0x18||8||The LBA containing this header
|-
|0x20||8||The LBA of the alternate GPT header
|-
|0x28||8||The first usable block that can be contained in a GPT entry
|-
|0x30||8||The last usable block that can be contained in a GPT entry
|-
|0x38||16||GUID of the disk
|-
|0x48||8||Starting LBA of the GUID Partition Entry array
|-
|0x50||4||Number of Partition Entries
|-
|0x54||4||Size (in bytes) of each entry in the Partition Entry array - must be a value of 128×2ⁿ where n ≥ 0 (in the past, multiples of 8 were acceptable)
|-
|0x58||4||CRC32 of the Partition Entry array.
|-
|0x5C||'''blocksize'''-0x5C||Reserved (should be zeroed)
|}
The Partition Entry array can contain unused entries -- that is, the GUID value is set to zero. For the purposes of the GPT header, these should be counted when considering the Number of Partition Entries field, and should also be taken into account when calculating the CRC32 of the entire array. There should be no more entries in the array, unused or otherwise, than are indicated by this field.

For the checksums in the header, the CCITT32 ANSI CRC method is used, the one with the polynomial 0x04c11db7 (same as in gzip, and different to the Castagnoli CRC32 that hardware accelerated CRC instructions calculate). The header checksum field at 0x10 is zeroed during calculation and the reserved data are not included.

=== LBA 2: Partition Entries ===

Usually after the EFI header sector, comes the table itself, however this could be defined in a GPT header field at 0x48 otherwise. For compatibility it is recommended to use LBA 2. The table is repeated at the end of the media before the alternate GPT header sector.

Each entry in the table looks like this:
{| {{wikitable}}
|'''Offset'''||'''Length (bytes)'''||'''Description'''
|-
|0x0||16||Partition Type GUID (zero means unused entry)
|-
|0x10||16||Unique Partition GUID
|-
|0x20||8||Starting LBA
|-
|0x28||8||Ending LBA
|-
|0x30||8||Attributes
|-
|0x38||72||Partition Name
|}

Partition type has no central registry, but with 16 bytes it is unlikely that there'll ever be a collision. You can find types for OSes on their corresponding corporation's website (for MacOSX see Apple; for Windows see Microsoft etc.)

Attributes is a bit set. Bit 0 (attrib & 1) means that the partition is required by the firmware and should not be touched. [[EFI System Partition]] would be a good example, however that's always explicitly assumed to have this bit set. Bit 2 (attrib & 2) means that the partition contains necessary files to boot an operating system (defined as EFI_PART_USED_BY_OS). This is somewhat similar to 0x80 boot flag for MBR partitions, but not entirely.

Name is UNICODE16-LE encoded, meaning it can only store the UNICODE Bilingual Plane (code points 32 to 65535), and each character consumes 2 bytes. However the EFI spec defines name length as 72 bytes (36 characters), you should never hard-code this into your application. Always use (Partition Entry Size in header at 0x54) - 0x38 instead.

== Utilities ==
The following utilities can handle GPT:
* [http://www.gnu.org/software/parted/manual/parted.html parted], [http://linux.die.net/man/8/gparted gparted], [http://www.rodsbooks.com/gdisk/ gdisk] on Linux
* [https://www.unix.com/man-page/OSX/8/diskutil/ diskutil] on MacOSX
* [http://technet.microsoft.com/en-us/library/bb490893.aspx diskpart] on Windows (Vista and upwards)
* [http://www.freebsd.org/cgi/man.cgi?query=gpt&manpath=FreeBSD+7.1-RELEASE gpt] on MacOSX and BSDs.

== Boot loaders ==
[[UEFI|EFI]] firmware is capable of booting from a specific GPT partition, [[EFI System Partition]] which is basically a FAT32 partition. It should contain a slightly modified [[EFI#Binary_Format|PE-executable]], like [http://elilo.sourceforge.net ELILO].  On old machines with legacy BIOS only you can use [[GRUB]], or you can write a custom [[Rolling_Your_Own_Bootloader|boot code]].

[[Limine]] fully supports GPT both when using UEFI and when using BIOS. Furthermore, unlike GRUB, when using BIOS, it supports embedding itself into the GPT structures as to make an additional reserved partition unnecessary (but an can be optionally configured to use one anyways).

[[BOOTBOOT]] by default boots your kernel from [[EFI System Partition]]. On UEFI machines it does that natively, and on legacy BIOS systems and other platforms (ARM for example) it interprets GPT and ESP on its own.

== See Also ==
=== Articles ===
* [[UEFI]]
* [[EFI System Partition]]

=== External Links ===
* [http://www.intel.com/technology/efi/|Intel EFI specifications et al.]
* [http://msdn.microsoft.com/en-us/windows/hardware/gg463524.aspx GPT on MSDN]
* [http://developer.apple.com/library/mac/#technotes/tn2166/_index.html GPT on Apple Developer]
* [http://www.ibm.com/developerworks/linux/library/l-gpt/ GPT on IBM developerWorks]
* [[wikipedia:GUID_Partition_Table|Wikipedia: GUID Partition Table]]
* [[wikipedia:BIOS_Boot_partition|Wikipedia: BIOS Boot partition]]

[[Category:Disks]]
[[Category:UEFI]]
[[Category:Firmware]]
````

<a id="w003127"></a>

## W003127 — APIC Timer

Bron-ID: W003127
Namespace: Main (0)
Revisie: 28146 | Datum: 2023-07-10T14:11:19Z
Bron: https://wiki.osdev.org/APIC_Timer
Revisie-URL: https://wiki.osdev.org/index.php?title=APIC_Timer&oldid=28146

````text
The great benefit of the Local [[APIC]] timer is that it is hardwired to each CPU core, unlike the [[PIT|Programmable Interval Timer]] which is a separate circuit. Because of this, there is no need for any resource management, which makes things easier. The downside is that it's oscillating at (one of) the CPU's frequencies, which varies from machine to machine, while the PIT uses a standard frequency (1,193,182 Hz). To make use of it, you have to know how many interrupts/sec it's capable of.

== APIC Timer Modes ==

The timer has 2 or 3 modes. The first 2 modes (periodic and one-shot) are supported by all local APICs. The third mode (TSC-Deadline mode) is an extension that is only supported on recent CPUs.

=== Periodic Mode ===

For periodic mode, software sets a "initial count" and the local APIC uses it for a "current count". The local APIC decrements the current count until it reaches zero, then generates a timer IRQ and resets the current count to the initial count and begins decrementing the current count again. In this way the local APIC generates IRQs at a fixed rate depending on the initial count. The current count is decremented at a rate that depends on the CPU's external frequency ("bus frequency") divided by the value in the local APIC's "Divide Configuration Register".

For example, for a 2.4 GHz CPU with an external/bus frequency of 800 MHz, if the Divide Configuration Register is set to "divide by 4" and the initial count is set to 123456; then the local APIC timer would decrement the count at a rate of 200 MHz and generate a timer IRQ every 617.28 us, giving a rate of IRQs of 1620.01 Hz.

=== One-Shot Mode ===

For one-shot mode, the local APIC decrements the current count (and generates a timer IRQ when the count reaches zero) in the same way as in periodic mode; however it doesn't reset the current count to the initial count when the current count reaches zero. Instead, software has to set a new count each time if it wants more timer IRQs.

The advantage of this mode is that software can have precise control over when timer IRQs occur. For example, during task switches an OS could set the count to a value that depends on the new task's priority (so that some tasks run for a small amount of time and other tasks run for a larger amount of time), and there wouldn't be any unwanted IRQs. Some OSs use this approach to implement a generic high precision timer service, where the local APIC count is set to a value that depends on which event will happen soonest. For example, if the currently running task switch should be pre-empted in 1234 nanoseconds, a sleeping task needs to wake up in 333 nanoseconds and alarm signal has to be sent in 44444 nanoseconds, then the timer's count would be set to 333 nanoseconds (the earliest delay needed) and when the the timer IRQ occurs the OS knows that there's 901 nanoseconds remaining before the current task should be pre-empted and 441111 nanoseconds until the alarm signal needs to be sent (and would set the count to 901 nanoseconds for the next timer IRQ).

The disadvantages are that it's harder to track real-time with one-shot mode and special care needs to be taken to avoid race conditions; especially if a new count is set before the old count expires.

=== TSC-Deadline mode ===

TSC-Deadline mode is very different to the other 2 modes. Instead of using the CPU's external/bus frequency to decrement a count, software sets a "deadline" and the local APIC generates a timer IRQ when the value of the CPU's time stamp counter is greater than or equal to the deadline.

Despite these differences, software would/could use it in the same way that one-shot mode would be used. The advantages (compared to one-shot mode) are that you get higher precision (because the CPU's time stamp counter runs at the CPU's (nominal) internal frequency rather than the CPU's external/bus frequency), and it's easier to avoid/handle race conditions.

== Enabling APIC Timer ==

Before enabling the local APIC timer, you should setup the rest of the local APIC. This includes:

* Determine the local APIC's physical address (via. ACPI tables or MultiProcessor Specification tables)
* Specify a spurious interrupt and software enable the APIC
* Make sure the TPR (Task Priority Register) is set (so it won't block/postpone lower priority IRQs)

Once that's done:

* Set the local APIC timer's divide configuration register
* Configure the local APIC timer's interrupt vector and unmask the timer's IRQ
* Set the local APIC timer's initial count

Note: It's recommended to follow the order given above (especially setting the local APIC timer's initial count last). Doing things in a different order (e.g. setting the initial count, then enabling the timer) can lead to problems on some (real or virtual) machines (e.g. everything seems right and counter is decreasing, but IRQ is never sent).

== Initializing ==
''Please note that this is the recommended way of determining the frequency of the APIC timer.''

'''NOTE:''' According to Intel's documentation for IA-32 (x86) and Intel 64 (x86_64), APIC timer's frequency is equal to the bus' frequency '''OR''' the core crystal's frequency divided by the chosen frequency divider. The bus' and the core crystal's frequency can be found in the [[CPUID|CPUID]] functions [https://sandpile.org/x86/cpuid.htm#level_0000_0015h 0x15] and [https://sandpile.org/x86/cpuid.htm#level_0000_0016h 0x16], respectively. Through CPUID.0x15 can also be determined the TSC frequency. The frequency of the APIC timer depends on whether the system is using local APIC or ''discrete'' APIC ([https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller 82489DX]). When the local APIC is built in core's crystal, the APIC timer is using core's frequency. Otherwise, it is using the bus frequency.


There are several ways to do this, but all of them use a different, CPU bus frequency independent clock source to do that. Examples: [[RTC|Real Time Clock]], [[TSC|TimeStamp Counter]], PIT or even polling [[CMOS#Getting_Current_Date_and_Time_from_RTC|CMOS registers]]. In this tutorial we will use the good old PIT, as it's the easiest. Steps that need to be done:
* Reset APIC to a well known state
* Enable APIC timer
* Reset APIC timer counter
* Wait a specific amount of time measured by a different clock
* Get number of ticks from APIC timer counter
* Adjust it to a second
* Divide it by the quantum of your choice (results X)
* Make the APIC timer fire an interrupt at every X ticks

The APIC timer can be set to make a tick (decrease counter) at a given frequency, which is called "divide value". This means you have to multiply APIC timer counter ticks by this divide value to get the true CPU bus frequency. You could use a value of 1 (ticks on every bus cycle) up to 128 (ticks on every 128th cycle). See Intel manual vol3A Chapter 9.5.4 for details. Note that according to my tests, Bochs seems not to handle divide value of 1 properly, so I will use 16.

=== Prerequisites ===
Before we start, let's define some constant and functions.
<source lang="asm">
apic		= the linear address where you have mapped the APIC registers

APIC_APICID	= 20h
APIC_APICVER	= 30h
APIC_TASKPRIOR	= 80h
APIC_EOI	= 0B0h
APIC_LDR	= 0D0h
APIC_DFR	= 0E0h
APIC_SPURIOUS	= 0F0h
APIC_ESR	= 280h
APIC_ICRL	= 300h
APIC_ICRH	= 310h
APIC_LVT_TMR	= 320h
APIC_LVT_PERF	= 340h
APIC_LVT_LINT0	= 350h
APIC_LVT_LINT1	= 360h
APIC_LVT_ERR	= 370h
APIC_TMRINITCNT	= 380h
APIC_TMRCURRCNT	= 390h
APIC_TMRDIV	= 3E0h
APIC_LAST	= 38Fh
APIC_DISABLE	= 10000h
APIC_SW_ENABLE	= 100h
APIC_CPUFOCUS	= 200h
APIC_NMI	= (4<<8)
TMR_PERIODIC	= 20000h
TMR_BASEDIV	= (1<<20)

		;Interrupt Service Routines
isr_dummytmr:	mov			dword [apic+APIC_EOI], 0
		iret
isr_spurious:	iret
		;function to set a specific interrupt gate in IDT
		;al=interrupt
		;ebx=isr entry point
writegate:	...
		ret
</source>
I will also assume that you have a working [[IDT]], and you have a function to write a gate for a specific interrupt: writegate(intnumber,israddress).
Furthermore, to make things simple, I'll assume that you did not change the default interrupt mapping found in almost every tutorial:
* interrupt 0-31: exceptions
* interrupt 32: timer, IRQ0
* interrupt 39: spurious irq, IRQ7
If you've already changed this, modify accordingly.


=== Example code in ASM ===
Here's a possible way to initialize APIC timer in fasm syntax assembly:
<source lang="asm">
		;you should read MSR, get APIC base and map to "apic"
		;you should have used lidt properly

		;set up isrs
		mov			al, 32
		mov			ebx, isr_dummytmr
		call			writegate
		mov			al, 39
		mov			ebx, isr_spurious
		call			writegate

		;initialize LAPIC to a well known state
		mov			dword [apic+APIC_DFR], 0FFFFFFFFh
		mov			eax, dword [apic+APIC_LDR]
		and			eax, 00FFFFFFh
		or			al, 1
		mov			dword [apic+APIC_LDR], eax
		mov			dword [apic+APIC_LVT_TMR], APIC_DISABLE
		mov			dword [apic+APIC_LVT_PERF], APIC_NMI
		mov			dword [apic+APIC_LVT_LINT0], APIC_DISABLE
		mov			dword [apic+APIC_LVT_LINT1], APIC_DISABLE
		mov			dword [apic+APIC_TASKPRIOR], 0
		;okay, now we can enable APIC
		;global enable
		mov			ecx, 1bh
		rdmsr
		bts			eax, 11
		wrmsr
		;software enable, map spurious interrupt to dummy isr
		mov			dword [apic+APIC_SPURIOUS], 39+APIC_SW_ENABLE
		;map APIC timer to an interrupt, and by that enable it in one-shot mode
		mov			dword [apic+APIC_LVT_TMR], 32
		;set up divide value to 16
		mov			dword [apic+APIC_TMRDIV], 03h

		;ebx=0xFFFFFFFF;
		xor			ebx, ebx
		dec			ebx

		;initialize PIT Ch 2 in one-shot mode
		;waiting 1 sec could slow down boot time considerably,
		;so we'll wait 1/100 sec, and multiply the counted ticks
		mov			dx, 61h
		in			al, dx
		and			al, 0fdh
		or			al, 1
		out			dx, al
		mov			al, 10110010b
		out			43h, al
		;1193180/100 Hz = 11931 = 2e9bh
		mov			al, 9bh		;LSB
		out			42h, al
		in			al, 60h		;short delay
		mov			al, 2eh		;MSB
		out			42h, al
		;reset PIT one-shot counter (start counting)
		in			al, dx
		and			al, 0feh
		out			dx, al		;gate low
		or			al, 1
		out			dx, al		;gate high
		;reset APIC timer (set counter to -1)
		mov			dword [apic+APIC_TMRINITCNT], ebx
		;now wait until PIT counter reaches zero
@@:		in			al, dx
		and			al, 20h
		jz			@b
		;stop APIC timer
		mov			dword [apic+APIC_LVT_TMR], APIC_DISABLE
		;now do the math...
		xor			eax, eax
		xor			ebx, ebx
		dec			eax
		;get current counter value
		mov			ebx, dword [apic+APIC_TMRCURRCNT]
		;it is counted down from -1, make it positive
		sub			eax, ebx
		inc			eax
		;we used divide value different than 1, so now we have to multiply the result by 16
		shl			eax, 4		;*16
		xor			edx, edx
		;moreover, PIT did not wait a whole sec, only a fraction, so multiply by that too
		mov			ebx, 100	;*PITHz
		mul			ebx
	;-----edx:eax now holds the CPU bus frequency-----
		;now calculate timer counter value of your choice
		;this means that tasks will be preempted 1000 times in a second. 100 is popular too.
		mov			ebx, 1000
		xor			edx, edx
		div			ebx
		;again, we did not use divide value of 1
		shr			eax, 4		;/16
		;sanity check, min 16
		cmp			eax, 010h
		jae			@f
		mov			eax, 010h
		;now eax holds appropriate number of ticks, use it as APIC timer counter initializer
@@:		mov			dword [apic+APIC_TMRINITCNT], eax
		;finally re-enable timer in periodic mode
		mov			dword [apic+APIC_LVT_TMR], 32 or TMR_PERIODIC
		;setting divide value register again not needed by the manuals
		;although I have found buggy hardware that required it
		mov			dword [apic+APIC_TMRDIV], 03h
</source>

=== Example code in C ===
This code is an example of how to initialize the APIC timer so that it ticks every 10 milliseconds. This is done by letting the APIC timer run, waiting for 10ms using the PIT and then getting the number of ticks that were done from the APIC timer. It assumes that you have functions to "write"/"read" the APIC's registers and "pit_prepare_sleep"/"pit_perform_sleep" to perform an as accurate as possible measurement of the timer's frequency.
<source lang="c">
void apic_start_timer() {
        // Tell APIC timer to use divider 16
        write(APIC_REGISTER_TIMER_DIV, 0x3);
 
        // Prepare the PIT to sleep for 10ms (10000µs)
        pit_prepare_sleep(10000);
 
        // Set APIC init counter to -1
        write(APIC_REGISTER_TIMER_INITCNT, 0xFFFFFFFF);
 
        // Perform PIT-supported sleep
        pit_perform_sleep();
 
        // Stop the APIC timer
        write(APIC_REGISTER_LVT_TIMER, APIC_LVT_INT_MASKED);
 
        // Now we know how often the APIC timer has ticked in 10ms
        uint32_t ticksIn10ms = 0xFFFFFFFF - read(APIC_REGISTER_TIMER_CURRCNT);
 
        // Start timer as periodic on IRQ 0, divider 16, with the number of ticks we counted
        write(APIC_REGISTER_LVT_TIMER, 32 | APIC_LVT_TIMER_MODE_PERIODIC);
        write(APIC_REGISTER_TIMER_DIV, 0x3);
        write(APIC_REGISTER_TIMER_INITCNT, ticksIn10ms);
}
</source>

== See also ==
=== Articles ===
* [[APIC]]
=== Threads ===
* [http://www.osdev.org/phpBB2/viewtopic.php?t=10686 APIC timer]

=== External Links ===
* [http://www.intel.com/products/processor/manuals/ Volume 3A:System Programming Guide, Part 1,manuals has a chapter on the APIC]
* [http://www.osdever.net/tutorials/view/advanced-programming-interrupt-controller Advanced Programmable Interrupt Controller by Mike Rieker]

[[Category:Interrupts]]
[[Category:Timers]]
````

<a id="w003128"></a>

## W003128 — Category:Level 0 Tutorials

Bron-ID: W003128
Namespace: Category (14)
Revisie: 12925 | Datum: 2012-03-07T13:10:13Z
Bron: https://wiki.osdev.org/Category:Level_0_Tutorials
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ALevel_0_Tutorials&oldid=12925

````text
These are unrated tutorials. They are therefore called level 0
[[Category:Tutorials]]
````

<a id="w003129"></a>

## W003129 — Category:Level 1 Tutorials

Bron-ID: W003129
Namespace: Category (14)
Revisie: 12926 | Datum: 2012-03-07T13:11:55Z
Bron: https://wiki.osdev.org/Category:Level_1_Tutorials
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ALevel_1_Tutorials&oldid=12926

````text
These are level 1, or easy tutorials.
[[Category:Tutorials]]
````

<a id="w003130"></a>

## W003130 — Category:Level 2 Tutorials

Bron-ID: W003130
Namespace: Category (14)
Revisie: 12927 | Datum: 2012-03-07T13:12:39Z
Bron: https://wiki.osdev.org/Category:Level_2_Tutorials
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ALevel_2_Tutorials&oldid=12927

````text
These are level 2, or medium tutorials.
[[Category:Tutorials]]
````

<a id="w003131"></a>

## W003131 — Category:Level 3 Tutorials

Bron-ID: W003131
Namespace: Category (14)
Revisie: 12928 | Datum: 2012-03-07T13:13:04Z
Bron: https://wiki.osdev.org/Category:Level_3_Tutorials
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ALevel_3_Tutorials&oldid=12928

````text
These are level 3, or hard tutorials.
[[Category:Tutorials]]
````

<a id="w003132"></a>

## W003132 — I/O Ports

Bron-ID: W003132
Namespace: Main (0)
Revisie: 29787 | Datum: 2025-12-12T20:06:08Z
Bron: https://wiki.osdev.org/I/O_Ports
Revisie-URL: https://wiki.osdev.org/index.php?title=I%2FO_Ports&oldid=29787

````text
An '''I/O port''' is usually used as a technical term for a specific address on the x86's IO bus. This bus provides communication with devices in a fixed order and size, and was used as an alternative to memory access. On many other architectures, there is no predefined bus for such communication and all communication with hardware is done via memory-mapped IO. This also increasingly happens on modern x86 hardware.


== Looking for a list of ports ==
As an operating system developer, '''you should not be looking for this'''. "Standardized" I/O ports are only used in some situations, such as hardware that was present on pre-PCI (AT and compatible, etc.) PC hardware. Examples of devices using "standardized" I/O ports are the [[Floppy Disk Controller|floppy drive]], [[Serial Ports|serial]] and [[Parallel port|parallel]] ports,  However, most ports are claimed by plugin cards{{which}} you might not have and some{{which}} are even assigned via PNP which means that any port could be used for the device in question. If you want access to a device, you will need to look up the details for the device in question, and if some fixed port is involved, it will tell you. It is not technically feasible to have a complete mapping of ports to devices as you won't get access to it.

== Port abuse ==
The original IBM PC decoded only the bottom 10 address bits for I/O ports, both on the motherboard and on IBM's expansion cards. That means that there are up to 64 port addresses mapping to the same port on the same device. Similarly, later devices (by IBM and others) chose to avoid conflict with the original PC devices by using a sparse register file. The bottom bits would be defined as constant, and the ''top bits'' would instead be used for the index. You would get ports number <tt>0x82E8</tt>, <tt>0x86E8</tt>, <tt>0x8AE8</tt> and so on. Such ISA devices are typically prone to interfering with [[PCI]] assigned IO addresses due to their spread.

Modern implementations of old devices (typically IDE and VGA controllers) have an implementation that is mentioned to be plug-and-play with moving registers, but they are still able to listen to legacy ports and respond accordingly.

== The list ==
If you have found a port number and you are still wondering what it's meant to do, here's a list of the most common ports:
{| {{Wikitable}}
|-
!Port range
!Summary

|-
|0x0000-0x001F
|The first legacy [[ISA DMA|DMA controller]], often used for transfers to floppies.

|-
|0x0020-0x0021
|The first [[PIC|Programmable Interrupt Controller]]

|-
|0x0022-0x0023
|Access to the Model-Specific Registers of Cyrix processors.

|-
|0x0040-0x0047
|The [[PIT]] (Programmable Interval Timer)

|-
|0x0060-0x0064
|The [["8042" PS/2 Controller]] or its predecessors, dealing with keyboards and mice.

|-
|0x0070-0x0071
|The [[CMOS]] and [[RTC]] registers

|-
|0x0080-0x008F
|The [[DMA]] (Page registers)

|-
|0x0092
|The location of the fast [[A20]] gate register

|-
|0x00A0-0x00A1
|The second [[PIC]]

|-
|0x00C0-0x00DF
|The second [[DMA]] controller, often used for soundblasters

|-
|0x00E9
|Home of the [[Bochs|Port E9 Hack]]. Used on some emulators to directly send text to the hosts' console.

|-
|0x0170-0x0177
|The secondary [[ATA]] harddisk controller.

|-
|0x01F0-0x01F7
|The primary [[ATA]] harddisk controller.

|-
|0x0278-0x027A
|Parallel port

|-
|0x02F8-0x02FF
|Second [[Serial Ports|serial port]]

|- 
|0x03B0-0x03DF
|The range used for the [[VGA Hardware|IBM VGA]], its direct predecessors, as well as any modern video card in legacy mode.

|- 
|0x03F0-0x03F7
|[[FDC|Floppy disk controller]]

|-
|0x03F8-0x03FF
|First [[Serial Ports|serial port]]

|-
|0x0CF8
|Register CONFIG_ADDRESS of [[PCI]] Configuration Space Mechanism #1

|-
|0x0CFC
|Register CONFIG_ADDRESS of [[PCI]] Configuration Space Mechanism #1

|}

== See Also ==

=== Articles ===
* [[Port IO]]

=== External Links ===
* [http://bochs.sourceforge.net/techspec/PORTS.LST Bochs' map of I/O ports to functions]

[[Category:X86]]
````

<a id="w003133"></a>

## W003133 — Talk:Device Management

Bron-ID: W003133
Namespace: Talk (1)
Revisie: 15224 | Datum: 2013-10-02T22:07:49Z
Bron: https://wiki.osdev.org/Talk:Device_Management
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ADevice_Management&oldid=15224

````text
I'd love to see more information here for how to start. Like probing the PCI space? How to correctly abstract and structure access to other things, like user I/O - instead of directly accessing 0xb8000 and keyboard ports? Possibly an explanation based on COM port interface instead of actual monitor/keyboard to help visualize how this fits in the picture.

Just some general information on kernel interfaces for different types of devices, I guess I'm a bit confused as it's easy enough to add drivers in infinity - but you need some kind of wrapper interface for each type or it will be impossible for applications to find the functions they need.

== Renaming this page ==

I'm wondering if this page would be more useful if it was renamed to Device Management. That way, it could focus on all the aspects of device management, not just the driver interfaces.
````

<a id="w003134"></a>

## W003134 — Can I have a list of IO Ports

Bron-ID: W003134
Namespace: Main (0)
Revisie: 12937 | Datum: 2012-03-07T18:36:43Z
Bron: https://wiki.osdev.org/Can_I_have_a_list_of_IO_Ports
Revisie-URL: https://wiki.osdev.org/index.php?title=Can_I_have_a_list_of_IO_Ports&oldid=12937
Doorverwijzing: I/O Ports

````text
#REDIRECT [[I/O Ports]]
[[Category:FAQ]]
````

<a id="w003135"></a>

## W003135 — I/O ports

Bron-ID: W003135
Namespace: Main (0)
Revisie: 12939 | Datum: 2012-03-07T18:37:17Z
Bron: https://wiki.osdev.org/I/O_ports
Revisie-URL: https://wiki.osdev.org/index.php?title=I%2FO_ports&oldid=12939
Doorverwijzing: I/O Ports

````text
#REDIRECT [[I/O Ports]]
````

<a id="w003136"></a>

## W003136 — Talk:Page Fault

Bron-ID: W003136
Namespace: Talk (1)
Revisie: 19313 | Datum: 2016-06-02T17:47:55Z
Bron: https://wiki.osdev.org/Talk:Page_Fault
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APage_Fault&oldid=19313

````text
_WHERE_ is this error code set? <small><span class="autosigned">Preceding [[Wikipedia:Signatures|unsigned]] comment added by [[User:Forge]] ([[User talk:Forge|talk]] • [[Special:Contributions/Forge|contribs]]) 20:16, 8 March 2012</span></small>

:Intel Manuals, Volume 3, chapter 4.7 "Page-Fault Exceptions", and chapter 6.13 "Error Code" handle this in detail. -- [[User:Solar|Solar]] 00:54, 9 March 2012 (CST)
````

<a id="w003137"></a>

## W003137 — Binary Notation

Bron-ID: W003137
Namespace: Main (0)
Revisie: 16269 | Datum: 2014-04-24T05:32:44Z
Bron: https://wiki.osdev.org/Binary_Notation
Revisie-URL: https://wiki.osdev.org/index.php?title=Binary_Notation&oldid=16269

````text
==Introduction==
'''Binary notation''' is another base to represent numbers, like [[Hexadecimal Notation]].
==Representation==
Binary only has two digits, 0 and 1. Each position is a power of two.
101101 =
1*1 +
0*2 +
1*4 +
1*8 +
0*16 +
1*32 =
45 in base 10

It is also the base computers use because of the fact that it can be represented as on or off - Digitally. However, it is difficult for humans to decode long strings of 1's and 0's, so it is often represented in Hexadeciaml (or Hex for short) as this is a lot shorter and is easy to convert between binary and hex. You just take each nibble and convert it indvidually. With practice you can be very quick doing this:
===Conversion===
{| {{wikitable}}
|-
! Binary
! Hex
! Base 10
|-
! 0000
! 0
! 0
|-
! 0001
! 1
! 1
|-
! 0010
! 2
! 2
|-
! 0011
! 3
! 3
|-
! 0100
! 4
! 4
|-
! 0101
! 5
! 5
|-
! 0110
! 6
! 6
|-

! 0111
! 7
! 7
|-
! 1000
! 8
! 8
|-
! 1001
! 9
! 9
|-
! 1010
! A
! 10
|-
! 1011
! B
! 11
|-
! 1100
! C
! 12
|-
! 1101
! D
! 13
|-
! 1110
! E
! 14
|-
! 1111
! F
! 15
|}

==See Also==
===Articles===
*[[Hexadecimal Notation]]
[[Category:Theory]]
````

<a id="w003138"></a>

## W003138 — Talk:Binary Notation

Bron-ID: W003138
Namespace: Talk (1)
Revisie: 12957 | Datum: 2012-03-09T13:56:18Z
Bron: https://wiki.osdev.org/Talk:Binary_Notation
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABinary_Notation&oldid=12957

````text
== Necessary? ==

At first I thought about merging [[Hexadecimal Notation]] and [[Binary Notation]] into a single article. While thinking about an appropriate title for the page, another thought occured to me: Do we need this ''at all''? I would expect anyone to make it beyond [[Required Knowledge]] to be intimately familiar with either. Next thing we'll be explaining "for" loops or something...?!? -- [[User:Solar|Solar]] 07:56, 9 March 2012 (CST)
````

<a id="w003139"></a>

## W003139 — Message Passing Tutorial

Bron-ID: W003139
Namespace: Main (0)
Revisie: 24893 | Datum: 2020-06-22T13:56:22Z
Bron: https://wiki.osdev.org/Message_Passing_Tutorial
Revisie-URL: https://wiki.osdev.org/index.php?title=Message_Passing_Tutorial&oldid=24893

````text
It's always a problem to decide if you use asynchronous or synchronous message passing. In this article I'll show you how to have both. I'll use a pseudo-code to describe the algorithm, so you can implement it to your language environment. Note that I refer sender and receiver as processes, it can be easily adopted to threads.
== Definitions ==
You should have a structure to be sent to another process. I'll refer to this as the message, and I will assume you have these fields:
<source lang="c">
struct message {
  src //the source process that sends the message
  dst //the destination process that receives
  body //the body of the message (usually holds type and arguments, it's up to you)
}
</source>
Sending and receiving must be atomic. This means you must prevent task switches until it's finished. I have two different timers in my OS, one for the wallclock, and another for preemption. So for me this means masking the latter, and reenabling it at the end. You could also use a [[mutex]] or [[semaphore]] to accomplish mutual exclusion.

Blocking and non blocking: the sender can be blocked upon sending a message, but this does not necessarily have to be so. The receiver must block if there's no message waiting. Blocking means the OS will remove the process from ready queue, and won't allocate CPU resources for it until the blockade is cancelled. When it happens, it simply puts the process back into the ready queue (most likely to the top). Processes on the blocked queue will not use any CPU time. This prevents a [[busy loop]].

You should maintain a queue for every process to record blocked waiting processes. This queue must not be a circular buffer, you can implement it as a simple chained list. I assume you have written the following functions already (they will be required by the tutorial):
<source lang="c">
 block(processid)               //function to block a process
 awake(processid)               //function to unblock a process
 isblocked(processid)           //returns true if process is blocked
 pushwaitqueue(recvpid,sendpid) //put sendpid on recvpid process' sender waiting queue
 topwaitqueue()                 //get the last pid in queue
 popwaitqueue()                 //get the last pid in queue and remove it from queue
</source>

Now a few words on synchronization: if it's asynchronous, it means the sender is not interested whether the receiver accepts the message or not. It will send the message and move on (won't block). This also means the message could be lost, hence messaging is unreliable. On the other hand, a synchronous sender will wait (block) until the message is delivered, this creates a rendezvous point (so the sender process and the receiver process will run synchronized after the message is accepted). Also because the sender knows when and if the message has arrived, it's a reliable messaging system.

Finally, [[circular buffer]]. It's a FIFO (First In, First Out) buffer. It's implemented by pointers (or indeces) head and tail. If you push something in a FIFO, it will be stored at the memory pointed to by head, and head will be adjusted. On pop, the item will be read from the memory pointed to by tail, and tail will be adjusted. If head or tail reaches the end of the buffer, they will wrap around.
<source lang="c">
struct circbuff {
  int head;  //index to queue start within buffer
  int tail;  //index to queue end within buffer
  int count; //number of elements in buff
  message buffer[MAXITEMS]; //buffer to hold messages
}
</source>
You could calculate the number of items in the buffer using the head and tail variables, but as being circular, there's a special case which cannot be handled without a count variable: head will be equal to tail if the buffer is empty, and also when it's full.

== Asynchronous ==
===Sending===
Now let's start with sending a message, and not care about. This could lead to losing a message, which we can't afford, so we'll do a trick here. Despite of being asynchronous, we will block if receiver buffer is full, and we'll continue only after there's space for our message.
<source lang="c">
void async_send(msg)
{
  disable_task_switch();
  msg.src=current_process; //we must not rely on it's set
  tmpbuff=map_buffer(msg.dst); //temporarily map destination's buffer into sender process' address space
  if (tmpbuff.count==MAXITEMS) { //if receiver buffer is full, block
    pushwaitqueue(msg.dst,current_process); //record this process in dst's sender queue
    block(current_process);
  }
  push(tmpbuff,msg);
  if(isblocked(msg.dst)) awake(msg.dst);  //if destination process is blocked for receiving, awake it
  unmap_buffer();
  enable_task_switch();
}
</source>
===Receiving===
Doesn't matter whether it's synchronized or not, receiver must block if it's message queue is empty, and there's nothing to process.
<source lang="c">
circbuff buff;
message async_recv()
{
  message tmp=NULL;
  disable_task_switch();
  if (buff.count==0) block(current_process); //if there's nothing to get, block
  tmp=pop(buff);
  while(topwaitqueue()!=NULL) awake(popwaitqueue()); //awake blocked processes waiting to send
  enable_task_switch();
  return (tmp);
}
</source>
It's possible that under very rare circumstances you want a non-blocking receive that returns NULL if there's no message waiting. I highly discourage, because it leads to a polling busy loop, but just in case, here you are:
<source lang="c">
message async_recvpoll()
{
  message tmp=NULL;
  disable_task_switch();
  if (buff.count!=0) {
    tmp=pop(buff);
    while(topwaitqueue()!=NULL) awake(popwaitqueue());
  }
  enable_task_switch();
  return (tmp);
}
</source>
Note that we count on recv being blocking to implement synchronous transfer. If you use the non-blocking code above, you'll have to take care of that on your own.

== Synchronous ==
=== Sending ===
Okay, now that we have primitives for asynchronous sending and receiving, it's rather easy to implement synchronous transfer on top of them.
<source lang="c">
message sync_send(msg)
{
  async_send(msg); //we send the message
  return(async_recv()); //and we block waiting for the response
}
</source>
=== Receiving ===
Likewise,
<source lang="c">
message consume(message); //function to do something with the message
void sync_recv()
{
  message tmp;
  tmp=async_recv();  //wait for a message to arrive
  tmp=consume(tmp);  //process the message and return a response message
  async_send(tmp);   //send it back to the caller
}
</source>
==What is this good for?==
Synchronous messaging is often used to implement [[RPC|Remote Procedure Calls]]. You send the function code and it's arguments first, then consume() calls the appropriate function and creates a message with the results.

Most OS use some primitive messaging to implement more sophisticated IPC like [[Unix Pipes|pipe]]s or [[socket]]s. Reading and writing from [[file]]s is also worked out by sending messages between the vfs process and the disk driver.

==Pitfalls==
This may seem to be easy, but don't forget it's only a tutorial. In the real world, you'll have to work a lot before your messaging code can became useful. Some suggestions:
* check the receiver actually wants to receive from the sender.
* always check for loops: process A waiting for B to send, C waiting for A. Now it would be a disaster if B also waits for C.
* you should implement an alarm for sending. If delivering fails within a timeout, you should check the reason(s), and maybe resend.
* you should have an unique id in every message to detect retransmission.
* normally userspace applications never have to receive messages without sending an acknowledge. So it's a good idea to tie asynchronous messaging to a capability flag or something similar.
* you should have a matrix of process ids recording who is allowed to send messages to who. An application should never send messages to a driver process directly (only through a library or system call).

==See Also==

=== Articles ===
* [[Remote_Procedure_Call|Remote Procedure Call (RPC)]]
* [[Message Passing]]

=== Threads ===
*[[Topic:10116|passing messages through registers]]
*[[Topic:10922|RPC message size, handling oversized messages]]

===External Links===
*[[Wikipedia:Message Passing | Message Passing]] on Wikipedia
*[http://stackoverflow.com/questions/827691/how-do-you-implement-a-circular-buffer-in-c How do I implement curcular buffer in C]?
*This is how [http://git.minix3.org/?p=minix.git;a=blob;f=kernel/proc.c;h=4595f08c2182c1ea9783b9b9cec3f499df0b5b2e;hb=HEAD#l799 Minix] does message passing (see functions mini_send, mini_receive well documented).

[[Category:IPC]]
````

<a id="w003140"></a>

## W003140 — User:Bellezzasolo/CFS

Bron-ID: W003140
Namespace: User (2)
Revisie: 20985 | Datum: 2017-07-08T00:46:50Z
Bron: https://wiki.osdev.org/User:Bellezzasolo/CFS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ABellezzasolo%2FCFS&oldid=20985

````text
{{Filesystems}}
'''Chai File System (CFS)'''
=CFS64=
==Introduction==
CFS64 is my file system ([[User:Bellezzasolo|Bellezzasolo]]'s). It uses 64 bit addressing, which is the ''64''. It currently has a (slightly buggy) Win32 formatter and a grub module.<br />
It is currently at version 1.01 <br />
'''I intend to create CFS64 2.0. As a result, these fields will be changing. I intend to improve the data structures used for access performance. Furthermore, Journaling is planned. I am also considering multiple-disk volumes.'''<br />
<small>'''Note: all fields are little-endian'''</small>

Github repository: [https://github.com/solocle/CFS64 CFS64] <br />
[[File:CFS64.jpg|640px]]
===Features===
*User security
*Flags
*Inode style tables
*Runtime-variable size
*Tree-based

===Limitations===
There are few limitations. Here are the only ones:
*One is the 64 bit addressing, which will (one day) be changed in CFS128; but for now it's plenty
<!---*There ''have'' to be 10 times the number of files possible as directories possible. This is because there isn't room in the master table for this data, but can be calculated by the space between the DTable pointer and the FTable pointer.-->
*File and Directory Name lengths.
<small>''If you spot any more, add them here''</small>

===Structure===
CFS64 is structured in the follwing way:
*MASTER Table - logical sector 0 of the partition. Bootloader and essential data
*Cluster Availability Table - a bit representing every cluster on the partition, in logical order
*FS Table - CAT for fs entries. i.e. cluster is partially used, not wholly.
*Entries - unordered entries, which point to respective parts. Will need defragmenting from time to time. No dedicated zone for file data, so no limits.

For error checking multiple copies of the tables can be kept

==Detail==
===MASTER Table===
{| {{wikitable}}
|-
! Name
! Offset (from the start of the MASTER Table)
! Length (in bytes)
! Meaning
|-
| Jump
| 0
| 2
| A jump to the bootloader if bootable, 0 otherwise. Normally disassembles to JMP SHORT (0x)3F
|-
| Creation Date
| 2
| 8
| The creation time and date, encoded in TeraTime. TeraTime is Unix time, except the epoch is 1st Jan 2011
|-
| Creator ID
| 10
| 4
| The creator's ID. This is normally 0, but could be set to a range of values for future use
|-
| Disk Size (Clusters)
| 14
| 8
| The number of clusters on the partition. Need I say any more?
|-
| Bytes per Cluster
| 22
| 2
| The number of bytes in a cluster. Could be the same as the disk, but then again might not be :)
|-
| Number of Tables
| 24
| 1
| The number of '''''Copies''''' of the tables. 1 if no backup, another number for backups. 0 is invalid
|-
| Major Version
| 25
| 2
| The major FS version. Can be any 2 byte value
|-
| Minor Version
| 27
| 1
| The minor FS version. Is the .''xy'' after the major version. Can only be between 0 and 99 inclusive
|-
| FS Table Pointer
| 28
| 8
| The first cluster of the FS Table. This is a Cluster Availability Table for FS entries.
|-
| Root Table Pointer
| 36
| 8
| The cluster where the root directory's file entry resides. It will be at offset 0
|-
| FS ID
| 44
| 8
| The FS string. Only ''"CFS64   "'' is acceptable
|-
| Volume Label
| 52
| 11
| The volume label. 11 characters only (ASCII)
|-
| Disks in Volume
| 63
| 2
| The number of partitions making up this logical volume. Add one to this value. TFS64 2.0+
|-
| Disk index
| 65
| 2
| The unique identifier of this disk within the volume. TFS64 2.0+
|-
| Bootloader
| 67 (or 0x43)
| 443
| The bootloader. Duh! :)
|-
| Boot Signature
| 510
| 2
| The old 0xAA55
|}
===Cluster Availability Table (CAT)===
This is just an array of bits. Each bit represents a cluster on the partition. If it's set to 1, the custer is used. Otherwise it's free. The bit's offset from the start of the CAT is the cluster at that offset from the start of the partition. FS data is included (even itself!). <br />
{| {{wikitable}}
|-
!Offset (bytes)
!Clusters
|-
|0
|0-7
|-
|1
|8-15
|-
|
|
|-
|10000
|80000-80007
|-
|10001
|80008-80015
|}
And in each byte:
{| {{wikitable}}
|-
|<big>'''Byte ''n'''''</big>
|-
|'''Bit'''
|7
|6
|5
|4
|3
|2
|1
|0
|-
|'''Cluster'''
|n*8 + 7
|n*8 + 6
|n*8 + 5
|n*8 + 4
|n*8 + 3
|n*8 + 2
|n*8 + 1
|n*8 + 0
|}
===FS Pointer===
A FSPointer is a structure that addresses filesystem objects.
{| {{wikitable}}
|-
! Name
! Offset
! Size
! Meaning
|-
| Cluster
| 0
| 8
| The cluster on the disk of this object
|-
| Byte
| 8
| 2
| The byte offset within the cluster of the object
|}

===FSPointerEx===
A FSPointer is a structure that addresses filesystem objects. The Ex version is designed to handle multiple disks.
{| {{wikitable}}
|-
! Name
! Offset
! Size
! Meaning
|-
| Disk
| 0
| 2
| The disk identifier for the disk on which described FS object resides.
|-
| Cluster
| 2
| 8
| The cluster on the disk of this object
|-
| Byte
| 10
| 2
| The byte offset within the cluster of the object
|}

===File Entry===
A file entry describes a filesystem object such as a directory or a file.
{| {{wikitable}}
|-
! Name
! Offset
! Size
! Meaning
|-
| Type
| 0
| 1
| The type of entry this is. 0xD1 - Directory, 0xF1 - file
|-
| Checksum
| 1
| 1
| A CRC32 checksum of the file entry
|-
| File Name
| 2
| 482
| The UTF-16 filename of the object. 241 characters.
|-
| Size
| 484
| 8
| The size of the object - bytes for a file, entries for a directory.
|-
| Attributes
| 492
| 10
| An FSPointer to attributes inode
|-
| Entries
| 502
| 10
| An FSPointer to respective inode (file or directory)
|}
===Directory Inode===
The directory inodes are pointed to by a directory entry. They are similar to Ext inodes in function. The structure is as follows:
{| {{wikitable}}
|-
! Name
! Offset
! Size
! Meaning
|-
| Type
| 0
| 1
| A type field. 0xD0 = Directory inode
|-
| Checksum
| 1
| 1
| A CRC32 checksum of the inode.
|-
| Entries
| 2
| 490
| Array of 49 FSPointers, point to file entries of directory contents
|-
| Backlink
| 492
| 10
| FSPointer back to previous element (either inode or entry)
|-
| Next Inode
| 502
| 10
| FSPointer to next inode if needed (cluster is zero if unused)
|}

===DirectoryEx Inode===
The Ex version supports CFS64 2.0 features. The structure is as follows:
{| {{wikitable}}
|-
! Name
! Offset
! Size
! Meaning
|-
| Type
| 0
| 1
| A type field. 0xD2 = DirectoryEx inode
|-
| Checksum
| 1
| 1
| A CRC32 checksum of the inode.
|-
| Entries
| 2
| 480
| Array of 40 FSPointerExs, point to file entries of directory contents
|-
| Reference Count
| 4
| 482
| Reference count for this inode - traditional hard links
|-
| Reserved
| 2
| 486
| Reserved for future use. Set to 0
|-
| Backlink
| 488
| 12
| FSPointerEx back to previous element (either inode or entry)
|-
| Next Inode
| 500
| 12
| FSPointerEx to next inode if needed (cluster is zero if unused)
|}

===File Inode===
The file entry is pointeed to by the file table like an Ext inode. The structure is as follows:
{| {{wikitable}}
|-
! Name
! Offset
! Size
! Meaning
|-
| Type
| 0
| 1
| 0xF0 = File Inode
|-
| Checksum
| 1
| 1
| A CRC32 checksum of the inode
|-
| Clusters
| 2
| 488
| An array of 61 cluster addresses.
|-
| Backlink
| 490
| 10
| An FSPointer to the previous inode in this chain (or the file entry if the first inode)
|-
| Next
| 500
| 10
| An FSPointer to the next inode. CLuster is 0 if this is the last inode.
|-
| Reserved
| 510
| 2
| Reserved, must be 0
|}

===Atttributes Inode===
This contains file metadata, including timestamps and security.
{| {{wikitable}}
|-
! Name
! Offset
! Size
! Meaning
|-
| Type
| 0
| 1
| 0xA0 for an attributes inode
|-
| Checksum
| 1
| 2
| A CRC32 checksum of the inode
|-
| Owner
| 2
| 16
| The UUID of the owner.
|-
| Created
| 18
| 8
| Teratime creation timestamp
|-
| Modified
| 26
| 8
| Teratime last modification timestamp
|-
| Accessed
| 34
| 8
| Teratime last accessed timestamp
|-
| Flags
| 42
| 1
| File flags - see later for description
|-
| Security Entries
| 43
| 432
| An array of 24 security entries - see next table
|-
| Security Inode
| 475
| 10
| An FSPointer to a security inode, if more entries are required.
|-
| Extended Attributes
| 485
| 10
| An FSPointer to the xattr inode, if present
|-
| Backlink
| 495
| 10
| Points back to file entry
|-
| Reference Count
| 505
| 4
| CFS64 2.0+ - used to keep track of hard links to a file.
|-
| Reserved
| 509
| 3
| Padding bytes, must be 0. May be used in future versions.
|}

===Security Entry===
{| {{wikitable}}
|-
! Name
! Offset
! Size
! Meaning
|-
| UUID
| 0
| 16
| The UUID of the user or group this relates to
|-
| Group
| 16
| 1
| 1 if the UUID is a group, 0 if a user.
|-
| Permissions
| 17
| 1
| Bit 0: r, Bit 1: w, Bit 2: x. Unix-style permissions. other bits reserved.
|}
===Security Inode===
More security entries if required
{| {{wikitable}}
|-
! Name
! Offset
! Size
! Meaning
|-
| Type
| 0
| 1
| 0x50 For Security inode
|-
| Checksum
| 1
| 1
| CRC32 checksum of inode
|-
| Entries
| 2
| 486
| 27 security entries
|-
| Backlink
| 488
| 10
| FSPointer to previous inode
|-
| Next
| 498
| 10
| FSPointer to next inode - cluster is 0 if this is the last.
|-
| Reserved
| 508
| 4
| Reserved, may be used in future. Must be 0 for now.
|}
===xattr Entry===
{| {{wikitable}}
|-
! Name
! Offset
! Size
! Meaning
|-
| Type
| 0
| 1
| 0xA8 for xattr table
|-
| Checksum
| 1
| 2
| CRC32 Checksum of the inode
|-
| Name
| 2
| 256
| 256 chars for the name (UTF-8). Not UTF-16 as this is internal - the name of the attribute.
|-
| Data
| 258
| 224
| 224 Bytes of the xattr data - format is dependent on name.
|-
| Inode
| 482
| 10
| FSPointer to the next inode, if required for the xattr. Cluster is 0 if unused.
|-
| Backlink
| 492
| 10
| Backlink to previous xattr entry, or attributes inode.
|-
| Next Entry
| 502
| 10
| The next xattr: THIS IS A DIFFERENT ATTRIBUTE. The next inode is in the inode field.
|}

===xattr Inode===
{| {{wikitable}}
|-
! Name
! Offset
! Size
! Meaning
|-
| Type
| 0
| 1
| 0xAF for xattr inode
|-
| Checksum
| 1
| 2
| CRC32 Checksum of the inode
|-
| Data
| 2
| 490
| 490 bytes of xattr data.
|-
| Backlink
| 492
| 10
| FSPointer to previous inode or entry
|-
| Next
| 502
| 10
| Points to next inode. Cluster is 0 if this is the last inode.
|}

=== CF64 v2.0 features===
====Overview====
CFS64 v2.0 adds support for some additional features. In particular, it supports journaling, virtual volumes spread across multiple partitions, and support for hard links and symbolic links. Data storage now supports extents and indexes. In particular, it does this in a manner compatible with CFS64 v1, by making use of the xattr mechanism regularly. A non-bootable CFS64 v1 partition can be mounted as v2 without any changes. If a CFS64 v2 partition doesn't use the new inode structures, it could be mounted as a read only v1 disk. Indeed, harder links will function correctly in CFS64 v1 read-only mode.
====Multiple Disk Metadata====
Metadata regarding multiple disk volumes is stored as xattr's on the root directory. Each disk will have its own root directory, merged into one under the virtual volume. As the root directory exists, the xattr list of disks "DiskVolumeSet" can be stored. This consists of a flat list of the disks, with length as described in the MASTER table.
{| {{wikitable}}
|-
! Name
! Offset
! Size
! Meaning
|-
| Disk Index
| 0
| 2
| The unique index for the disk within this volume
|-
| Disk ID
| 2
| 16
| The UUID of the disk
|}

==== Hard Links - detail ====
Hard links are implemented by pointing a directory entry to the file entry for the file/directory to be hard linked. This differs from traditional hard links, which point to the inode. Should the traditional mode be used, the extended inode must be used. The side effect of native hard links (harder links) is that, should the file have its name changed, this will be reflected in the original directory, and also the linked directories. Note that this means consistency checks should be made - the driver should check that there is not a name conflict in any directory. Although there is no limitation of the FS against files with the same name, this is dangerous territory.

==== Journaling ====
CFS64 stores a journal file as an xattr ("FileSystemJournal") of the root directory. The structure is as follows:
=====Header=====
{| {{wikitable}}
|-
! Name
! Offset
! Size
! Meaning
|-
| Mode & flags
| 0
| 1
| The journaling mode. 0 = disabled, 1 = metadata-only, 2 = complete
|-
| Checksum
| 1
| 1
| A CRC32 checksum of the header
|-
| Entries
| 2
| 8
| The number of transactions recorded in the journal.
|}
=====Entry=====
{| {{wikitable}}
|-
! Name
! Offset
! Size
! Meaning
|-
| Type
| 0
| 1
| The type of write. 0 = availability bitmap, 1 = file system structure, 2 = block write.
|-
| Checksum
| 1
| 1
| A CRC32 checksum of the entry - disregard if invalid. Transaction was not completed.
|-
| Address
| 2
| 8/10
| The address of the write. If type=0, this is the block represented. If type=1, this is FSPointer. If type=2, this is block address.
|-
| Data
| 10/12
| 1/512/4096
| The data to be written. Type=0, bit0=CAT value, bit1=FSTable value. Rest reserved. Type=1, this is file system data structure. Type=2, this is the block value.
|}

===C structures===
<source lang="c">
 typedef uint64_t teratime;
 typedef uint16_t wchar_t;
 typedef uint16_t char16_t;

//MASTER table
struct CFS64_Master {
	uint8_t bldrjmp[2];		//JMP +0x40 (0x42)
	teratime creation;
	uint32_t creator;
	uint64_t disksize;		//Clusters
	uint16_t bytespercluster;
	uint8_t tablecopies;
	uint16_t majorversion;
	uint8_t minorversion;
	uint64_t FStabpointer;
	uint64_t roottablepointer;
	char FSID[8];			//CFS64
	char16_t vollabel[11];
	uint8_t bootloader[436];
	uint16_t bootsig;		//0xAA55
}__attribute__ ((packed));

enum CreatorID {
	CREATOR_ID_CHAIOS,
	CREATOR_ID_WINFORMATTER,	//us
	CREATOR_ID_LINUX,
	CREATOR_ID_HANDCODED = 0xCAFEBABE,
	CREATOR_ID_THIRD_PARTY = 0xDEADBEEF,
	CREATOR_ID_INCOMPATIBLE = 0xFFFFFFFF
};

//File tables
struct FSPointer {
	uint64_t Cluster;
	uint16_t byte;
}__attribute__ ((packed));

struct CFS64_FileEntry {
	uint8_t type;		//0xF1 for file, 0xD1 for directory,... 0 for free
	uint8_t checksum;
	wchar_t fileName[241];
	uint64_t size;
	struct FSPointer attrbiutes;
	struct FSPointer entries;
}__attribute__ ((packed));

enum FileEntryType {
	SecurityInode = 0x50,
	DirectoryInode = 0xD0,
	DirectoryEntry = 0xD1,
	AttributeInode = 0xA0,
	ExtendedAttributeHeader = 0xA8,
	ExtendedAttributeInode = 0xAF,
	FileInode = 0xF0,
	FileEntry = 0xF1
};

struct CFS64_FileInode {
	uint8_t type;		//0xF1 for directory
	uint8_t checksum;
	uint64_t clusters[61];
	struct FSPointer backlink;
	struct FSPointer nextInode;
	uint16_t reserved;
}__attribute__ ((packed));

struct CFS64_DirInode {
	uint8_t type;		//0xD1 for directory
	uint8_t checksum;
	struct FSPointer entries[49];
	struct FSPointer backlink;
	struct FSPointer nextInode;
}__attribute__ ((packed));

//Now attributes

struct UUID {
    uint32_t Data1;
    uint16_t Data2;
    uint16_t Data3;
    uint8_t Data4[ 8 ];
}__attribute__ ((packed));

struct CFS64_SecurityEntry {
	struct UUID userID;
	uint8_t isgroup;
	struct {
		uint8_t canRead : 1;
		uint8_t canWrite : 1;
		uint8_t canExecute : 1;
	}permissions;
}__attribute__ ((packed));

struct CFS64_Flags {
	uint8_t DeleteOnly : 1;
	uint8_t Hidden : 1;
	uint8_t ContentsHidden : 1;
	uint8_t NonPrivelegedRead : 1;
	uint8_t ModifyOnly : 1;
	uint8_t NonPrivilegedWrite : 1;
	uint8_t NonPrivilegedExecute : 1;
	uint8_t SystemFile : 1;
}__attribute__ ((packed));

struct CFS64_Attributes {
	uint8_t type;
	uint8_t checksum;
	struct UUID ownerID;
	teratime created;
	teratime modified;
	teratime accessed;
	struct CFS64_Flags flags;
	struct CFS64_SecurityEntry securityentry[24];
	struct FSPointer securityInode;
	struct FSPointer xattr;
	struct FSPointer backlink;
	uint8_t reserved[7];
}__attribute__ ((packed));

struct CFS64_Security {
	uint8_t type;
	uint8_t checksum;
	struct CFS64_SecurityEntry entries[27];
	struct FSPointer backlink;
	struct FSPointer nextentry;
	uint32_t reserved;
}__attribute__ ((packed));

struct CFS64_xattr_entry {
	uint8_t type;
	uint8_t checksum;
	char name[256];		//type of attribute, so we don't need unicode
	uint8_t data[224];
	struct FSPointer inode;
	struct FSPointer backlink;
	struct FSPointer nextentry;
}__attribute__ ((packed));

struct CFS64_xattr_inode {
	uint8_t type;
	uint8_t checksum;
	uint8_t data[490];
	struct FSPointer backlink;
	struct FSPointer nextentry;
}__attribute__ ((packed));
</source>

===Inodes===
The "Inode entries" referenced to before are pointers to clusters. For directory inodes these reference both cluster and byte, and therefore are 10 bytes long (64bit cluster and 16 bit byte references). File inodes only need cluster references, and therefore the entries are 8 bytes long.
====Indirection====
Inodes point to other inodes as a doubly linked list, with backlinks and forward links. This means no size limits like ext2 - but seek times are greater. The structure is optimized for sequential reading.
====Unused Inodes====
Unused inodes are set to 0.

===Special Fields===
Here are decriptions of the bit-flags fields:
====Flags====
Flags are the general attributes, such as read-only, hidden and similar.
{| {{wikitable}}
|-
! 7
! 6
! 5
! 4
! 3
! 2
! 1
! 0
|-
| SF
| NX
| NW
| MO
| NR
| CH
| HI 
| DO
|}
Meanings:
*DO: Delete Only (Read only field modify protect)
*HI: Hidden
*CH: Contents Hidden (dir-only)
*NR: Non-Privileged Read - read ability for people without permissions
*MO: Modify Only (Read-only deletion protect)
*NW: Non-Privileged Write - write version of NR
*NX: Non-Privileged Execute - execute version of NW
*SF: System File - ultimate protection.  Cannot be changed under ChaiOS (my, [[User:Bellezzasolo|Bellezzasolo]]'s OS). Drivers should be installed to a special directory.

=====Flags that apply to all users=====
*DO
*HI
*CH
*MO
*SF
=====Other flag's behaviour=====
The NR,NW and NX flags only apply to non-privileged users, i.e those who don't have their permissions bit set.

==Project Status==
Implemented for grub, however in active development. Here are the components required and their status:
*Formatter - Buggy (Win32)
*Grub module - Slightly buggy
*Driver - Not Started. Waiting for HDD and formatter
````

<a id="w003141"></a>

## W003141 — User talk:Bellezzasolo/CFS

Bron-ID: W003141
Namespace: User talk (3)
Revisie: 18855 | Datum: 2016-01-17T12:29:26Z
Bron: https://wiki.osdev.org/User_talk:Bellezzasolo/CFS
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ABellezzasolo%2FCFS&oldid=18855

````text
==Move to main namespace==
Hello, and if you are here thankyou for taking your time to review my filesystem. I've been wondering wether I should move this to the main namespace. Thanking you for any contribution in advance, --[[User:Bellezzasolo|Bellezzasolo]] 13:41, 14 March 2012 (CDT)
:I don't think it should be in the main namespace, simply because "it currently hasn't been implemented". I was doubtful if it should even be in your user namespace; until you have implemented it and could make some claims as to comparitive merits (i.e., why people should actually be interested in implementing it), it's your very personal project, and should reside on your personal project homepage. Frequent updates, as are likely for a format under development, are also cluttering the [[Special:RecentChanges | Recent Changes]] page... -- [[User:Solar|Solar]] 05:54, 15 March 2012 (CDT)

Tankyou for your thoughts. As you can see, I've acted on them -- [[User:Bellezzasolo|Bellezzasolo]] 08:07, 15 March 2012 (CDT)

P.S. I don't have a homepage elswhere, so I have to leave it here. Thankyou for the time used to tell me what you think. -- [[User:Bellezzasolo|Bellezzasolo]] 08:13, 15 March 2012 (CDT)

:Well, this isn't really the place to document your project efforts. There are several sites offering free project hosting, like SourceForge or GitHub, which would be more appropriate. (See my remarks about RecentChanges cluttering. This one page isn't so bad, but if we set a precedent here, maintaining the Wiki would become a nightmare.) -- [[User:Solar|Solar]] 08:22, 15 March 2012 (CDT)

Thankyou for your comments. Please feel free to delete or keep as you see fit (I have a copy)<br />
P.S. Sorry for the large delay. I had a lot of work to do.<br/>
--[[User:Bellezzasolo|Bellezzasolo]] 08:36, 22 March 2012 (CDT)

It is now implemented, so it has a right to be here. I'm putting the grub module onto GitHub.--[[User:Bellezzasolo|Bellezzasolo]] 06:29, 17 January 2016 (CST)
````

<a id="w003144"></a>

## W003144 — CFS

Bron-ID: W003144
Namespace: Main (0)
Revisie: 19209 | Datum: 2016-05-11T12:57:32Z
Bron: https://wiki.osdev.org/CFS
Revisie-URL: https://wiki.osdev.org/index.php?title=CFS&oldid=19209
Doorverwijzing: User:Bellezzasolo/CFS

````text
#REDIRECT [[User:Bellezzasolo/CFS]]
````

<a id="w003145"></a>

## W003145 — Talk:CFS

Bron-ID: W003145
Namespace: Talk (1)
Revisie: 20694 | Datum: 2017-04-10T17:47:05Z
Bron: https://wiki.osdev.org/Talk:CFS
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ACFS&oldid=20694
Doorverwijzing: User talk:Bellezzasolo/CFS

````text
#REDIRECT [[User talk:Bellezzasolo/CFS]]
````

<a id="w003148"></a>

## W003148 — User:Essial

Bron-ID: W003148
Namespace: User (2)
Revisie: 13021 | Datum: 2012-03-18T01:00:59Z
Bron: https://wiki.osdev.org/User:Essial
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AEssial&oldid=13021

````text
= About Me =

== Employment ==
I am a Software Engineer with the US Department of Energy -- specifically, the Savannah River Site. Before that, I have held several jobs dealing with Software Engineering and Computer and System Management.

== Family ==
I have a wife and two boys -- ages 2 and 4. They keep me busy the few hours a day I'm actually allowed to be home.

== For Fun ==
I like playing on my consoles (PS3, all the way through to Atari and NES), as well as strategic grand strategy games on my PC. I also like reading up on specifications and standards -- as well as, of course, OS development topics.
````

<a id="w003149"></a>

## W003149 — User:Pancakes

Bron-ID: W003149
Namespace: User (2)
Revisie: 15830 | Datum: 2014-03-30T17:19:05Z
Bron: https://wiki.osdev.org/User:Pancakes
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APancakes&oldid=15830

````text
== About Me ==
I like pancakes, bacon, butter, and syrup.

I like X86/X64 but they can be really complex, and a lot of their stuff is pages and pages deep and you know something fresh and new is always great. So I spend most of my time messing with stuff like ARM, MIPS (even if it is possibly dieing), or anything else. I recommend the book ''See Mips Run'' by ''Dominic Sweetman''. Its a very good book about MIPS or really any RISC processor.

Some people may think (or would think if they knew, LOL) - "Why does he spend all this time throwing away information instead of stuffing it into some book and selling it to people". Well, I used to be just like a lot of you. Not all, because some of you are better than me. But, a many of you are just like I was almost 10 years ago. And, some legendary people who's names are to be forever etched into history gave me the knowledge through tutorials, documentation, and examples to write or even attempt to write system level software. Of, course I did not say their names would be remembered only etched into history. For you see no one may ever remember your contributions to the world, but they can never remove them no matter how hard they try. Because, with out them the time of present just would not be but instead something entirely different. I guess what I am trying to say is:

: ''There are things in life you will do in order to help others even though you understand you may never be repaid for. A sacrifice if you will to others in return for the sacrifice someone made to you. --''

: ''"The greatest tyrannies are always perpetrated in the name of the noblest causes." -- Thomas Paine''

: ''"In the beginning of a change the patriot is a scarce man, and brave, and hated and scorned. When his cause succeeds, the timid join him, for then it costs nothing to be a patriot." -- Mark Twain''

<pre>
 ba45bc7b0f89aa3cb066fa8f3f4418ed
 19a33e4c4e76fb9a6feb164b430e421e
 626bfb3ff730aad32c329e0cbf90a14d
 7dfc957b5e001d216c1e23b9f1960c43
 68eca342c5d4beae1093962e6f115404
 34f872fcf7e34d303e020d639f8502ca
 21b1175165faec502ffca0cfa37ca8b2
 6bd05e4815a826eb78af229cdf3d51eb
</pre>

== Pages Under My Namespace ==
[http://wiki.osdev.org/index.php?title=Special%3APrefixIndex&prefix=pancakes&namespace=2 Pages]

== Printf Debugging Tool ==
Instead of banging your head trying to get it figured out. You can instead spend more time on figuring other things out. You need a implementation of ''putc'' for characters and ''puts'' for strings. Also just do the ''#include <stdarg.h>'' if you get compiler complaints about ''vararg'' structures.
<pre>
/*
    Leonard Kevin McGuire Jr (www.kmcg3413.net) (kmcg3413@gmail.com)

    This code is so simple that there is NO reason to add some legal
    wording for copying. Just copy and paste it all you like.
*/

const char *itoh_map = "0123456789ABCDEF";

char* itoh(int i, char *buf)
{
	int		n;
	int		b;
	int		z;
	int		s;
	
	if (sizeof(void*) == 4)
		s = 8;
	if (sizeof(void*) == 8)
		s = 16;
	
	for (z = 0, n = (s - 1); n > -1; --n)
	{
		b = (i >> (n * 4)) & 0xf;
		buf[z] = itoh_map[b];
		++z;
	}
	buf[z] = 0;
	return buf;
}


void kprintf(const char *fmt, ...)
{
	const char 	*p;
	va_list 	argp;
	int 		i;
	char 		*s;
	char 		fmtbuf[256];

	va_start(argp, fmt);

	for(p = fmt; *p != '\0'; p++)
	{
		//kputc('w');
		if(*p != '%')
		{
			kputc(*p);
			continue;
		}

		switch(*++p)
			{
			case 'c':
				i = va_arg(argp, int);
				kputc(i);
				break;
			case 's':
				s = va_arg(argp, char *);
				kputs(s);
				break;
			case 'x':
				i = va_arg(argp, int);
				s = itoh(i, fmtbuf);
				kputs(s);
				break;
			case '%':
				kputc('%');
				break;
		}
	}
	va_end(argp);
}
</pre>
````

<a id="w003150"></a>

## W003150 — ARM Integrator-CP PL110 Dirty

Bron-ID: W003150
Namespace: Main (0)
Revisie: 28052 | Datum: 2023-07-09T19:14:19Z
Bron: https://wiki.osdev.org/ARM_Integrator-CP_PL110_Dirty
Revisie-URL: https://wiki.osdev.org/index.php?title=ARM_Integrator-CP_PL110_Dirty&oldid=28052

````text
{{FirstPerson}}
=== Integrator-CP QEMU PL110 16-Bit Color Example ===
This is quick and dirty really. I just want to get you on your feet with something that will make you enjoy doing this. Although, many people would consider going straight to working on the GUI not the correct way to do it is however very fun for many people. So here you go.

== Prerequisites ==
{| class="wikitable"
|-
! Software
! Why
|-
| QEMU || We are going to use it. We do not properly initialize the hardware. We almost do it correctly, but not quite enough for the real thing. Thats okay.
|-
| C compiler or assembler || You need either a compiler or an assembler. I will show you code for doing it both ways. I will handwrite the assembly so it should be easier to read and work with.
|-
| objcopy || You need something like objcopy on most Linux distributions. What we are going to have to do is copy out the .text section. Like I said this is just really quick and dirty example. We will make a flat binary.
|}

== Memory ==
You will notice I am producing a flat binary output. This output contains no headers and the entry functions begins at the first byte of the image produced. The image is loaded by QEMU to the memory address 0x10000. QEMU supports loading an ELF format image and should
correctly place each image section at the specified address. When you write software for real embedded devices they are generally not going to support this as you will write the image directly to their ROM. A many devices have a secondary ROM area that actually provides support when placed in a special mode to allow you to flash your BOOT image which goes in the lower part of the ROM or somewhere different. This essentially consists of two ROMs in essence where the primary boot code allows you to easily flash the device over USB when a special button is held. If not it jumps to your BOOT code. If it can handle non flat binary formats then this special initial ROM code will support this functionality and you can find it in your boards documentation.

But, for the most part you are going to be uploading a flat binary file.

== C Code ==
<source lang="c">
typedef int(*PFN)(void);

void start(void);


void __attribute__((naked)) entry()
{
	__asm__("mov	sp, #0x60 << 8");
	__asm__("bl	start");
}

#define PL110_CR_EN		0x001
#define PL110_CR_PWR		0x800
#define PL110_IOBASE		0xc0000000
#define PL110_PALBASE		(PL110_IOBASE + 0x200)

typedef unsigned int		uint32;
typedef unsigned char		uint8;
typedef unsigned short		uint16;

typedef struct _PL110MMIO 
{
	uint32		volatile tim0;		//0
	uint32		volatile tim1;		//4
	uint32		volatile tim2;		//8
	uint32		volatile d;		//c
	uint32		volatile upbase;	//10
	uint32		volatile f;		//14
	uint32		volatile g;		//18
	uint32		volatile control;	//1c
} PL110MMIO;

void start(void)
{
	PFN		fn;
	PL110MMIO	*plio;
	int		x;
	uint16		volatile *fb;
	
	plio = (PL110MMIO*)PL110_IOBASE;
	
	/* 640x480 pixels */
	plio->tim0 = 0x3f1f3f9c;
	plio->tim1 = 0x080b61df;
	plio->upbase = 0x200000;
	/* 16-bit color */
	plio->control = 0x1829;
	fb = (uint16*)0x200000;
	for (x = 0; x < (640 * 480) - 10; ++x)
		fb[x] = 0x1f << (5 + 6) | 0xf << 5;
	
	/* uncomment this and the function pointer should crash QEMU if you set it for 8MB of ram or less */
	for(;;);
	fn = (PFN)0x800f20;
	fn();
	return;
}
</source>

A naked function implementation so we can setup the stack, which then calls the actual C function start. Then we initialize the card and draw a light blue color across the screen.

To compile it and produce the binary file to use with QEMU first copy the code above into
a file named ''test.c''. Then execute the following. Your actual binaries will likely differ.
Make sure your using a cross-toolchain for ARM and not your host system unless your host
system is ARM.
<pre>
    ./gcc-arm test.c -nostdlib -o test.o
    ./objcopy-arm -j .text -O binary test.o test.bin
     qemu-system-arm -m 8 -kernel test.bin
</pre>

If all goes well QEMU will display a light blue screen.


== Assembly ==
<source lang="asm">
main:
	eor	r0, r0, r0
	mov	r0, #0xc0 << 24
		
	mov	r1, #0x3f << 24
	add	r1, r1, #0x1f << 16
	add	r1, r1, #0x3f << 8
	add	r1, r1, #0x9c
	str	r1, [r0, #0]
	
	mov	r1, #0x08 << 24
	add	r1, r1, #0x0b << 16
	add	r1, r1, #0x61 << 8
	add	r1, r1, #0xdf
	str	r1, [r0, #4]
	
	mov	r1, #0x20 << 16
	str	r1, [r0, #0x10]
	
	mov	r1, #0x18 << 8
	add	r1, r1, #0x29
	str	r1, [r0, #0x1c]
	
	mov	r2, #0x4b << 12
	mov	r1, #0x20 << 16
	mov	r3, #0x1f
clearscreen:
	strh	r3, [r1], #2
	subs	r2, r2, #1
	bne	clearscreen
	#b	0x800000
lockme:
	b	lockme
</source>

We initialize the card then we draw bright red to the screen.

To assemble it and produce the binary file to use with QEMU first copy the code above into
a file named ''test.asm''. Then execute the following. Your actual binaries will likely differ.
Make sure your using a cross-toolchain for ARM and not your host system unless your host
system is ARM.
<pre>
     ./as-arm test.asm -o test.o
     ./objcopy-arm -j .text -O binary test.o test.bin
     qemu-system-arm -m 8 -kernel test.bin
</pre>

If all goes well QEMU will display a bright red screen.

== More Flexible Compile/Link ==
The 0x10000 is the area in memory this image is going to start at.

=== link.ld ===
<pre>
ENTRY (entry)

SECTIONS
{
    . = 0x10000;
    .text : { *(.text*) *(.rodata*) *(.data*) *(.bss*) *(COMMON) }
     _EOI = .;
}
</pre>

=== script ===
You can notice now we are supporting the compile of multiple sources, and a seperate linking step which uses our linker
script (above). This build scheme supports properly referencing global variables in your code, but still produces a flat
binary file with no header or sections. It also directs any output written to the serial port to stdout.
<pre>
#!/bin/sh
arm-linux-gnueabi-gcc -Wall -Wextra -Werror -nostdlib -nostartfiles -ffreestanding -std=gnu99 -c *.c
arm-linux-gnueabi-ld -T link.ld -o _armos.bin *.o
arm-linux-gnueabi-objcopy -j .text -O binary _armos.bin armos.bin
qemu-system-arm -m 8 -kernel armos.bin -serial stdio
</pre>

See [[ARM_Integrator-CP_Bare_Bones|ARM_Integrator-CP_Bare_Bones]] for writing to the serial port.

== Explanation ==
Essentially, what we did was set just enough to make it work with ''QEMU PC emulator version 0.12.5 (Debian 0.12.5+dfsg-3squeeze1), Copyright (c) 2003-2008 Fabrice Bellard''. If it does not work and you have a different version of QEMU then that could be the problem, but then again make sure you used the correct toolchain.

Now, the real hardware. Well its going to want you to set more values then just that. Also, the pixels are 16-bit in the memory buffer at 0x200000 and they are in 565 format BGR. So blue bits occupy the most significant bits and there is 5 of them. B:5 G:6 R:5 - green has 6 bits. This is a common format and used with DXT1 compression format most graphics cards support.

After we initialize the card we just set the same value in a loop. Now, you will notice that in both the C and the assembly code that I have a loop and a commented out branch instruction or in the C code I have a function pointer which will crash the emulator. Its not going to hard crash it, but it will make the emulator stop and display the registers. This is highly useful as a debugging aid. Since you place it somewhere in your program and it will tell you what all the registers contain so you can debug difficult pieces of code. It can also tell you if the code has deadlocked. How else do you know if the code has completed properly or has deadlocked? You do not know unless you make it do something to tell you. 

Since you have the ability to plot colored pixels on the screen you have another way to debug your operating system that you are going to build a interesting GUI on top on while completely forgetting about the real interesting stuff like processes, threads, timers, priority, disk I/O, devices, memory management, heaps...

''But, having fun is important too it just requires forgetting about all that! So have fun because thats why I made this page.''

[[Category:ARM]]
````
