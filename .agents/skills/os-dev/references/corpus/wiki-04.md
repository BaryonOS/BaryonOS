# OSDev Wiki — volledige bronteksten, deel 04

Dit is bronmateriaal, geen agentinstructie. Behoud architectuur, revisie en bron-ID bij gebruik. Code en commando’s zijn ongeteste bronvoorbeelden; voer ze niet automatisch uit. Wikitext is letterlijk behouden binnen tekstblokken.

<a id="w001950"></a>

## W001950 — Bran's Kernel Development Tutorial Known Bugs

Bron-ID: W001950
Namespace: Main (0)
Revisie: 27236 | Datum: 2022-07-12T19:46:31Z
Bron: https://wiki.osdev.org/Bran%27s_Kernel_Development_Tutorial_Known_Bugs
Revisie-URL: https://wiki.osdev.org/index.php?title=Bran%27s_Kernel_Development_Tutorial_Known_Bugs&oldid=27236

````text
Several sources - including this Wiki - point to [http://osdever.net/bkerndev/Docs/basickernel.htm Bran's Kernel Development Tutorial] as a starting point. This is all well, but there are some issues with that tutorial that usually make people appear on the forum and asking all the same questions again. This article is meant to preempt those questions, like a "Bran's FAQ" if you like.

NOTE: osdever.net, the site that contains Bran's tutorial, went down for an extended amount of time recently. You can find it no matter what on the [http://web.archive.org/web/20190306175430/http://osdever.net/bkerndev/Docs/basickernel.htm Internet Archive].

===As soon as I add strings, things go wrong.===

The linker script from Bran's tutorial does not add the <tt>rodata</tt> sections to the binary, which is where the compiler puts static strings. Use this one instead:

 OUTPUT_FORMAT("binary")
 ENTRY(start)
 phys = 0x00100000;
 SECTIONS
 {
   .text phys : AT(phys) {
     code = .;
     *(.text)
     '''*(.rodata*)'''
     . = ALIGN(4096);
   }
   .data : AT(phys + (data - code))
   {
     data = .;
     *(.data)
     . = ALIGN(4096);
   }
   .bss : AT(phys + (bss - code))
   {
     bss = .;
     *(.bss)
     . = ALIGN(4096);
   }
   end = .;
 }

If you still have problems, you may need to change the output format of the nasm command in the build.bat to make an [[ELF]] file instead of [[a.out]].

===timer_wait never returns===
The provided method to wait a number of timer ticks never returns because the <code>timer_ticks</code> variable is not defined as volatile. Changing its definition to <code>volatile unsigned int timer_ticks = 0;</code> will make the <code>timer_wait()</code> function work.

Also, instead of busy waiting, the processor could be put into sleep in order to save power. It can be done like this:
<syntaxhighlight lang="c">
void timer_wait(int ticks)
{
    unsigned int eticks;

    eticks = timer_ticks + ticks;
    while(timer_ticks < eticks) 
    {
        __asm__ __volatile__ ("sti//hlt//cli");
    }
}
</syntaxhighlight>

===Does not properly set up PIT===
The tutorial says the PIT setup code snippet it provides is "optional", when in reality it isn't.

===Does not use Cross-Compiler===
Bran's Kernel Tutorial uses DJGPP and NASM (standard install) on windows.<br />
To allow the code to link on a cross-compiler you must first remove all leading underscores from Global and External ASM Labels.

===Divide by zero interrupt does not get raised===
After implementing the IDT and ISRs, the tutorial explains that dividing by zero will now raise the exception 'divide by zero' and consequently execute the fault handler. The tutorial provides the following C code:
<syntaxhighlight lang="c">
int i = 10 / 0;
</syntaxhighlight>
The fault handler on this exception, however, will not be raised and nothing happens. Compilers usually optimize and insert some extra code (so there is actually no divide by zero). The only way to guarantee this exception is raised is to write some in-line assembly like:
<syntaxhighlight lang="c">
__asm__  ("div %0" :: "r"(0));
</syntaxhighlight>

===Compiler error: unknown type name ‘size_t'===
GCC produces the following errors: ''unknown type name 'X'''. The tutorial source code doesn't include any standard definitions for types like ''size_t'' or ''uint32_t''. Including the following headers in each source file:
<syntaxhighlight lang="c">
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
</syntaxhighlight>

===References===
* http://forum.osdev.org/viewtopic.php?f=8&t=18302

[[Category:OS Development]]
[[Category:Troubleshooting]]
[[Category:FAQ]]
````

<a id="w001951"></a>

## W001951 — Talk:Physical And Virtual Addresses

Bron-ID: W001951
Namespace: Talk (1)
Revisie: 4283 | Datum: 2007-07-11T20:13:22Z
Bron: https://wiki.osdev.org/Talk:Physical_And_Virtual_Addresses
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APhysical_And_Virtual_Addresses&oldid=4283

````text
[[Memory management]] has the "Paging" section of this article. -[[User:Jhawthorn|Jhawthorn]] 21:57, 9 March 2007 (CST)
:...and? Pull it to one article and link from the other? Merge the articles entirely? - [[User:Combuster|Combuster]] 08:47, 11 March 2007 (CDT)
:What about creating [[Paging]] and moving 'Virtual Memory' and 'Memory Protection' to it, along with creating a more in-depth article about paging in general? I think having [[Segmentation]] without [[Paging]] is a little uneven. Following that, this page could be deleted. --Alboin
::Sounds OK to me. - [[User:Combuster|Combuster]] 15:13, 11 July 2007 (CDT)
````

<a id="w001952"></a>

## W001952 — User:Kmcguire/Quick And Dirty Heap Algorithm

Bron-ID: W001952
Namespace: User (2)
Revisie: 15831 | Datum: 2014-03-30T17:29:58Z
Bron: https://wiki.osdev.org/User:Kmcguire/Quick_And_Dirty_Heap_Algorithm
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKmcguire%2FQuick_And_Dirty_Heap_Algorithm&oldid=15831

````text
=Quick And Dirty Heap Algorithms=

This page is obsolete. A better page has been created, 
[[Heap|Here (Heap)]].
````

<a id="w001953"></a>

## W001953 — Category:Executable Formats

Bron-ID: W001953
Namespace: Category (14)
Revisie: 3292 | Datum: 2007-03-11T05:08:38Z
Bron: https://wiki.osdev.org/Category:Executable_Formats
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AExecutable_Formats&oldid=3292

````text
{{main|Executable Formats}}

[[Category:OS theory]]
````

<a id="w001954"></a>

## W001954 — Executable Formats

Bron-ID: W001954
Namespace: Main (0)
Revisie: 27885 | Datum: 2023-07-09T04:33:20Z
Bron: https://wiki.osdev.org/Executable_Formats
Revisie-URL: https://wiki.osdev.org/index.php?title=Executable_Formats&oldid=27885

````text
An executable format is the file format created by the compiler and linker, and that is then loaded into memory by the operating system's loader. The number of different executable file types is as many and varied as the number of different image and sound file formats. This page gives an introduction to the theory behind these formats.

== Definitions ==
* TEXT is the actual executable code area,
* DATA is "initialized" data,
* BSS is "un-initialized" data. 

The BSS (Block Started by Symbol) needn't to be present in an executable file. At load-time, the loader will still allocate memory for it and ''wipes'' this memory with zeros (this is assumed by C programs, for instance).

== See Also ==
=== Articles ===
* [[:Category:Executable Formats|List of executable formats]]
=== External Links ===
* [http://www.wotsit.org/ Wotsit's Programmer's File Format Collection] 
* The [http://www.iecc.com/linker/ Linkers and Loaders] online book... 
* [http://perso.wanadoo.fr/pierrelib/exec_formats/index.html Pierre's Library]

[[Category:Executable Formats]]
[[Category:OS theory]]
````

<a id="w001955"></a>

## W001955 — A.out

Bron-ID: W001955
Namespace: Main (0)
Revisie: 30550 | Datum: 2026-05-02T19:07:46Z
Bron: https://wiki.osdev.org/A.out
Revisie-URL: https://wiki.osdev.org/index.php?title=A.out&oldid=30550

````text
{{Stub}}
{{File formats}}

'''<tt>a.out</tt>''' is the "original" binary format for Unix machines. It is considered obsolete today because of several shortcomings. It was superseded by System V [[ELF]]. However, as it is extremely simple and supported by many compilers/assemblers, it may be a good choice if you're willing to develop your own format or have more information than a raw binary for your bootloader.

==Structure==
An a.out binary file consists of up to 7 sections.  In order, these sections are:
{| class="wikitable"
|-
! Section
! Description
|-
|exec header
|Contains parameters used by the kernel to load a binary file into memory and execute it, and by the link editor ld to combine a binary file with other binary files. 	       This section is the only mandatory one.
|-
|text segment
|Contains machine code and related data that are loaded into memory when a program executes. Should be loaded read-only.
|-
|data segment
|Contains initialized data; always loaded into writable memory.
|-
|text relocations
|Contains records used by the link editor to update pointers in the text segment when combining binary files.
|-
|data relocations
|Like the text relocation section, but for data segment pointers.
|-
|symbol table
|Contains records used by the link editor to cross reference the addresses of named variables and functions (`symbols') between binary files.
|-
|string table
|Contains the character strings corresponding to the symbol names.
|}
===The exec header===
Every binary file begins with an exec structure, which is 32 bytes in size. Some later versions of a.out are modified to allow 64 bit code execution. Their header is 64 bytes in size, however.

32 bit executable header:
<syntaxhighlight lang="c">
struct exec
{
	uint32_t   a_midmag;
	uint32_t   a_text;
	uint32_t   a_data;
	uint32_t   a_bss;
	uint32_t   a_syms;
	uint32_t   a_entry;
	uint32_t   a_trsize;
	uint32_t   a_drsize;
};
</syntaxhighlight>

The fields have the following functions:
{| class="wikitable"
|-
! Position (32 bit)
! Position (64 bit)
! Field
! Description
|-
|0-3
|0-7
|a_midmag
|This field is stored in network byte-order so that binaries for machines with alternative byte orders can be distinguished. It has a number of sub-components accessed by the macros N_GETFLAG(), N_GETMID(), and N_GETMAGIC(), and set by the macro N_SETMAGIC().
|-
|4-7
|8-15
|a_text
|Contains the size of the text segment in bytes.
|-
|8-11
|16-23
|a_data
|Contains the size of the data segment in bytes.
|-
|12-15
|24-31
|a_bss
|Contains the number of bytes in the `bss segment'. The kernel loads the program so that this amount of writable memory appears to follow the data segment and initially reads as zeroes.
|-
|16-19
|32-39
|a_syms
|Contains the size in bytes of the symbol table section.
|-
|20-23
|40-47
|a_entry
|Contains the address in memory of the entry point of the program after the kernel has loaded it; the kernel starts the execution of the program from the machine instruction at this address.
|-
|24-27
|48-55
|a_trsize
|Contains the size in bytes of the text relocation table.
|-
|28-31
|56-63
|a_drsize
|Contains the size in bytes of the data relocation table.
|}

===The relocation info structure===
Relocation records have a standard format which is described by the relocation_info structure:

<syntaxhighlight lang="c">
struct relocation_info
{
    int32_t  r_address;

    uint32_t r_symbolnum : 24,
             r_pcrel     : 1,
             r_length    : 2,
             r_extern    : 1,
             r_baserel   : 1,
             r_jmptable  : 1,
             r_relative  : 1,
             r_copy      : 1;
};
</syntaxhighlight>

The fields have the following functions:
{| class="wikitable"
|-
! Position in bits (32 bit)
! Field
! Description
|-
|0-31
|r_address
| Contains the byte offset of a	pointer	that needs to be link-edited.  Text relocation offsets are reckoned from the start of the text segment, and data relocation offsets from the start of the data segment. The link editor adds the value that is already stored at this offset into the new value that it computes using this relocation record.
|-
|32-55
|r_symbolnum
|Contains the ordinal number of a symbol structure in the symbol table (it is not a byte offset).
|-
|56
|r_pcrel
|If this is set, the link editor assumes that it is updating a pointer that is part of a machine code instruction using pc-relative addressing. The address of the relocated pointer is implicitly added to its value when the running program uses it.
|-
|57-58
|r_length
|Contains the log base	2 of the length	of the pointer in bytes.
|-
|59
|r_extern
|Set if this relocation requires an external reference.
|-
|60
|r_baserel
|If set, the symbol is to be relocated to an offset into the GOT.
|-
|61
|r_jmptable
|If set, the symbol is to be relocated to an offset into the PLT.
|-
|62
|r_relative
|If set, this relocation is relative to the load address of the image this object file is going to be a part of.
|-
|63
|r_copy
|If set, this relocation record identifies a symbol whose contents should be copied to the location given in r_address.
|}

== See Also ==
=== External Links ===
*[https://web.archive.org/web/20120301032406/http://www.nondot.org/sabre/os/files/Executables/OpenBSD-a.out.txt OSRC #1]
*[https://web.archive.org/web/20120301023848/http://www.nondot.org/sabre/os/files/Executables/a.out.txt OSRC #2]
*[https://man.openbsd.org/OpenBSD-5.4/a.out.5 OpenBSD-5.4 man page on a.out]
*[https://www.freebsd.org/cgi/man.cgi?query=a.out FreeBSD man page on a.out]
*[https://man.netbsd.org/a.out.5 NetBSD man page on a.out]

[[Category:Executable Formats]]
````

<a id="w001956"></a>

## W001956 — MZ

Bron-ID: W001956
Namespace: Main (0)
Revisie: 30558 | Datum: 2026-05-05T18:04:16Z
Bron: https://wiki.osdev.org/MZ
Revisie-URL: https://wiki.osdev.org/index.php?title=MZ&oldid=30558

````text
{{File formats}}

The MS-DOS EXE format, also known as MZ after its signature (the initials of Microsoft engineer Mark Zbikowski), was introduced with PC-DOS 1.0 (pre-release version 0.90 only sported the simple [[COM]] format; note in DOS 1.x the DOS kernel doesn't support EXEs and the EXE loader is in COMMAND.COM; in DOS 2.x and later it was moved into the DOS kernel). It is designed as a relocatable executable running under real mode. As such, only DOS and Windows 9x/Me can use this format natively, but there are several free DOS emulators (e.g., [http://www.dosbox.com/ DOSBox]) that support it and that run under various operating systems (e.g., Linux, Amiga, Windows NT, etc.). Although they can exist on their own, MZ executables are embedded in all [[NE]], [[LE]], and [[PE]] executables, usually as stubs so that when they are ran under DOS, they display:
<pre>
This program cannot be run in MS-DOS mode.
</pre>

However, they can also be used so that a single executable can provide 2 ports of the same application (e.g. one for DOS and one for Windows). Windows 9x will run the MZ executable if the program is started from the command line prompt, or the PE executable if started normally. In the case of boot loaders, they can help provide a DOS version, especially since UEFI requires the PE format, which contains a MZ executable.

==MZ File Structure==

MZ executables only consists of 2 structures: the header and the relocation table.  The header, which is followed by the program image, looks like this:

{| class="wikitable"
|-
! colspan=2|Offset
! Field
! Size
! Description
|-
| 0
| 0x00
| Signature
| word
| <tt>0x5A4D</tt> ([[ASCII]] for 'M' and 'Z')
|-
| 2
| 0x02
| Extra bytes
| word
| Number of bytes in the last page.
|-
| 4
| 0x04
| Pages
| word
| Number of whole/partial pages.
|-
| 6
| 0x06
| Relocation items
| word
| Number of entries in the relocation table.
|-
| 8
| 0x08
| Header size
| word
| The number of paragraphs taken up by the header. It can be any value, as the loader just uses it to find where the actual executable data starts. It may be larger than what the "standard" fields take up, and you may use it if you want to include your own header metadata, or put the relocation table there, or use it for any other purpose.
|-
| 10
| 0x0A
| Minimum allocation
| word
| The number of paragraphs '''required''' by the program, excluding the PSP and program image. If no free block is big enough, the loading stops.
|-
| 12
| 0x0C
| Maximum allocation
| word
| The number of paragraphs '''requested''' by the program. If no free block is big enough, the biggest one possible is allocated.
|-
| 14
| 0x0E
| Initial SS
| word
| Relocatable segment address for SS.
|-
| 16
| 0x10
| Initial SP
| word
| Initial value for SP.
|-
| 18
| 0x12
| Checksum
| word
| When added to the sum of all other words in the file, the result should be zero.
|-
| 20
| 0x14
| Initial IP
| word
| Initial value for IP (aka ''entry point'').
|-
| 22
| 0x16
| Initial CS
| word
| Relocatable segment address for CS.
|-
| 24
| 0x18
| Relocation table
| word
| The (absolute) offset to the relocation table.
|-
| 26
| 0x1A
| Overlay
| word
| Value used for overlay management. If zero, this is the main executable.
|-
| 28
| 0x1C
| Overlay information
| N/A
| Files sometimes contain extra information for the main's program overlay management.
|}

A paragraph is 16 bytes in size. A page (or block) is 512 bytes long.

If both the minimum and maximum allocation fields are cleared, MS-DOS will attempt to load the executable as high as possible in memory. Otherwise, the image will be loaded just above the 256-byte PSP structure, in low memory.

===Relocations===

After loading the executable into memory, the program loader goes through every entry in relocation table. For each relocation entry, the loader adds the start segment address into word value pointed to by the segment:offset pair. So, for example, a relocation entry 0001:001A will make the loader add start segment address to the value at offset 1*0x10+0x1A=0x2A within the program data.

Each pointer in the relocation table looks as such:

{| class="wikitable"
|-
! colspan=2|Offset
! Field
! Size
! Description
|-
| 0
| 0x00
| Offset
| word
| Offset of the relocation within provided segment.
|-
| 2
| 0x02
| Segment
| word
| Segment of the relocation, relative to the load segment address.
|}

CS and SS registers are relocated in a similar fashion.

==Initial Program State==
* ES and DS registers both point to the segment containing the PSP structure.
* CS equals value specified in the header, relocated by adding the start segment address to it.
* IP equals value specified in the header. Note, that unlike in [[COM]] executables, MZ programs don't start at offset 0x100.
* SS equals value specified in the header, relocated, just like CS.
* SP equals value specified in the header.
* AL is 0x00 if the first FCB in the PSP has a valid drive identifier, 0xFF otherwise.
* AH is the same as AL, but for the second FCB in the PSP.
* All other registers may, or may not be set to 0. You should consider them undefined.

==PE Extension==

The MZ format was suitable for larger DOS executables, however, was extended with the introduction of Windows 1.0 in 1985, and the multitasking MS-DOS 4.0 in 1986. This new executable format was called the [[NE]], for New Executable.

A couple of new fields were added, primarily, e_lfanew.

{| class="wikitable"
|-
! colspan=2|Offset
! Field
! Size
! Description
|-
| 28
| 0x1C
| Reserved
| qword
|
|-
| 36
| 0x24
| OEM identifier
| word
| Defined by name but no other information is given; typically zeroes
|-
| 38
| 0x26
| OEM info
| word
| Defined by name but no other information is given; typically zeroes
|-
| 40
| 0x28
| Reserved
| 20 bytes
|
|-
| 60
| 0x3C
| PE header start
| dword
| Starting address of the extended header
|}
This same extension is used by the [[PE]] format, as well as [[LE]], but with correspondingly different signatures in the extended header. In principle it can be used for ''any'' executable format which is defined as an extension of MZ.

==See Also==
* [https://web.archive.org/web/20120204110033/http://www.nondot.org/sabre/os/files/Executables/EXE.txt EXE format description at OSRC]
* [https://www.delorie.com/djgpp/doc/exe/ D.J. Delorie: MZ Header format]
* [http://www.pinvoke.net/default.aspx/Structures.IMAGE_DOS_HEADER IMAGE_DOS_HEADER]
* [https://web.archive.org/web/20200730062804/https://marcin-chwedczuk.github.io/a-closer-look-at-portable-executable-msdos-stub A closer look at Portable Executable MS-DOS Stub]
* [https://github.com/FDOS/kernel/blob/33f17b0bd7f168484e6ec490c1f87b9991f8b764/kernel/task.c#L625 FreeDOS kernel's MZ loader source code (GPL 2.0-licensed)]

[[Category:Executable Formats]]
````

<a id="w001957"></a>

## W001957 — NE

Bron-ID: W001957
Namespace: Main (0)
Revisie: 29636 | Datum: 2025-08-31T18:33:08Z
Bron: https://wiki.osdev.org/NE
Revisie-URL: https://wiki.osdev.org/index.php?title=NE&oldid=29636

````text
{{File formats}}
==Introduction==
The '''NE''' ('''New Executable''') format was introduced with Windows 1.0. Like its predecessor [[MZ]], it is a 16-bit format, but it lifts some of its maximum size restrictions and adds new features, such as references to other files.

All numbers in the NE file's structures are in little-endian byte order.

==Support==
NE files are the native executable format of Windows 1.01 through 3.xx, Windows NT 3.1, as well as OS/2 1.x. The format can be run on later 32-bit Windows systems (in a Virtual DOS Machine), such as Windows 95 and Windows NT 3.5, but their native format is [[PE]].

Because 64-bit mode on x86 processors lacks virtual 8086 support, NE executables cannot be run on 64-bit Windows.

==Inside the NE file==
===DOS Stub===
Every NE executable is also a valid [[MZ]] executable. This enables the developer to package both an MS-DOS and Win16 version of the program in one file, but for most programs, the MZ executable only prints "This Program requires Microsoft Windows" and exits, so the MZ part is often referred to as the ''DOS Stub''.

To detect whether an MZ executable is also an NE executable, the following must be done:

# Read the <tt>e_lfarlc</tt> field (uint16_t at offset 0x18). If it does not equal 0x0040, the executable is not an NE executable.
# Read the <tt>e_lfanew</tt> field (uint32_t at offset 0x3C). Seek to that location in the file and read 2 bytes. If these bytes are <tt>'N'</tt> and <tt>'E'</tt>, the executable is an NE executable. (Other common possibilities at that location are <tt>'L'</tt> and <tt>'E'</tt> for [[LE]], and <tt>'P'</tt> and <tt>'E'</tt> for [[PE]] executables.)

===NE Header===
The NE header is a relatively large structure.

<syntaxhighlight lang="cpp">
struct NE_Header {
    char sig[2];                 // {'N', 'E'}
    uint8_t MajLinkerVersion;    //The major linker version
    uint8_t MinLinkerVersion;    //The minor linker version (also known as the linker revision)
    uint16_t EntryTableOffset;   //Offset of entry table from start of NE_Header
    uint16_t EntryTableLength;   //Length of entry table in bytes
    uint32_t FileLoadCRC;        //32-bit CRC of entire contents of file
    uint16_t FlagWord;           // Uses the FlagWord enum
    uint16_t AutoDataSegIndex;   //The automatic data segment index
    uint16_t InitHeapSize;       //The initial local heap size
    uint16_t InitStackSize;      //The initial stack size
    uint32_t EntryPoint;         //CS:IP entry point, CS is index into segment table
    uint32_t InitStack;          //SS:SP initial stack pointer, SS is index into segment table
    uint16_t SegCount;           //Number of segments in segment table
    uint16_t ModRefs;            //Number of module references (DLLs)
    uint16_t NoResNamesTabSiz;   //Size of non-resident names table in bytes
    uint16_t SegTableOffset;     //Offset of segment table from start of NE_Header
    uint16_t ResTableOffset;     //Offset of resources table from start of NE_Header
    uint16_t ResidNamTable;      //Offset of resident names table from start of NE_Header
    uint16_t ModRefTable;        //Offset of module reference table from start of NE_Header
    uint16_t ImportNameTable;    //Offset of imported names table from start of NE_Header
    uint32_t OffStartNonResTab;  //Offset of non-resident names table from start of file (!)
    uint16_t MovEntryCount;      //Count of moveable entry point listed in entry table
    uint16_t FileAlnSzShftCnt;   //File alignment size shift count (0=9(default 512 byte pages))
    uint16_t nResTabEntries;     //Number of resource table entries (often inaccurate!)
    uint8_t targOS;              //Target OS

    //
    // The rest of these are not defined in the Windows 3.0 standard and
    // appear to be specific to OS/2.

    uint8_t OS2EXEFlags;         //Other OS/2 flags
    uint16_t retThunkOffset;     //Offset to return thunks or start of gangload area - what is gangload?
    uint16_t segrefthunksoff;    //Offset to segment reference thunks or size of gangload area
    uint16_t mincodeswap;        //Minimum code swap area size
    uint8_t expctwinver[2];      //Expected windows version (minor first)
};

//
// In 16-bit DOS/Windows terminology, DGROUP is a segment class that referring
// to segments that are used for data.
//
// Win16 used segmentation to permit a DLL or program to have multiple
// instances along with an instance handle and manage multiple data
// segments. This allowed one NOTEPAD.EXE code segment to execute
// multiple instances of the notepad application.
//
enum FlagWord {
    // how is data handled?
    NOAUTODATA = 0x0000,
    SINGLEDATA = 0x0001, // shared among instances of the same program
    MULTIPLEDATA = 0x0002, // separate for each instance of the same program

    // additional flags:
    LINKERROR = 0x2000, // Linker error, module cannot load
    LIBMODULE = 0x8000, // if this flag is set, this is a DLL;
                        // see the "Dynamic Libraries" section below
};

#define GLOBINIT 1<<2     //global initialization
#define PMODEONLY 1<<3    //Protected mode only
#define INSTRUC86 1<<4    //8086 instructions
#define INSTRU286 1<<5    //80286 instructions
#define INSTRU386 1<<6    //80386 instructions
#define INSTRUx87 1<<7    //80x87 (FPU) instructions

//Application flags
//Application type
enum apptype {
    none,
    fullscreeen,    //fullscreen (not aware of Windows/P.M. API)
    winpmcompat,    //compatible with Windows/P.M. API
    winpmuses       //uses Windows/P.M. API
};

// #define OS2APP 1<<3    //OS/2 family application
// //bit 4 reserved?
// #define IMAGEERROR 1<<5    //errors in image/executable
// #define NONCONFORM 1<<6    //non-conforming program?
// #define DLL        1<<7

//Target Operating System
enum targetos {
    unknown,    //Obvious ;)
    os2,        //OS/2 (as if you hadn't worked that out!)
    win,        //Windows (Win16)
    dos4,       //European DOS  4.x
    win386,     //Windows for the 80386 (Win32s). 32 bit code.
    BOSS        //The boss, a.k.a Borland Operating System Services
};
//Other OS/2 flags
#define LFN 1        //OS/2 Long File Names (finally, no more 8.3 conversion :) )
#define PMODE 1<<1   //OS/2 2.x Protected Mode executable
#define PFONT 1<<2   //OS/2 2.x Proportional Fonts
#define GANGL 1<<3   //OS/2 Gangload area
</syntaxhighlight>

Which types of processor mode are supported can be ascertained from the following fields:

{| class="wikitable"
|-
! <code>expctwinver[1]</code>
! <code>FlagWord &amp; 0x0008</code>
! <code>OS2EXEFlags &amp; 0x04</code>
! real mode
! protected mode
|-
| &lt; 3 || any || 0 || yes || no
|-
| &lt; 3 || any || != 0 || yes || yes
|-
| 3 || 0 || any || yes || yes
|-
| 3 || != 0 || any || no || yes
|}

In the following examples, it is assumed that <tt>neHeader</tt> is a variable of type <tt>struct NE_Header</tt>.

=== Segment Table ===

The segment table is similar to a section table in newer executable formats. Each segment can contain code, data, relocation information, etc.

It is found at <tt>neHeader.SegTableOffset</tt> from the beginning of <tt>neHeader</tt> and occupies <tt>neHeader.SegCount</tt> entries, each 8 bytes long.

<syntaxhighlight lang="cpp">
// The type is a 3-bit integer or'ed together with the other flags.
#define SEGFLAGS_TYPE_CODE  0
#define SEGFLAGS_TYPE_DATA  1
#define SEGFLAGS_TYPE_MASK  0x0007

#define SEGFLAGS_MOVABLE    0x0010
#define SEGFLAGS_PRELOAD    0x0040
#define SEGFLAGS_HAS_RELOCS 0x0100
#define SEGFLAGS_DISCARD    0xF000

// Segment table entry
typedef struct {
    uint16_t    SectorBase; // offset in sectors from beginning of file;
                            // byte offset: segEnt.SectorBase * (1 << neHeader.FileAlnSzShftCnt)
    uint16_t    SegBytes;   // length of segment in file, in bytes
    uint16_t    SegFlags;   // see SEGFLAGS_*
    uint16_t    MinAlloc;   // minimum number of bytes to allocate
} NE_SegEnt;
</syntaxhighlight>

=== Resource Table ===

The resource table contains structures such as icons and version information.

It is found at <tt>neHeader.ResTableOffset</tt> from the beginning of <tt>neHeader</tt>. Its length can only be determined by decoding it.

It begins with a short header:
<syntaxhighlight lang="cpp">
typedef struct {
    uint16_t    AlignmentShiftCount;
} NE_ResourceTableHeader;
</syntaxhighlight>

Then follows a table of resource types and resources.

<syntaxhighlight lang="cpp">
typedef struct {
    uint16_t    TypeID;
    uint16_t    ResourceCount;
    uint32_t    Reserved;
} NE_ResourceTypeHeader;

#define RESOURCE_FLAGS_MOVEABLE 0x0010
#define RESOURCE_FLAGS_PURE     0x0020
#define RESOURCE_FLAGS_PRELOAD  0x0040

typedef struct {
    uint16_t    FileOffset;     // byte offset of resource from start of file:
                                // resource.FileOffset * (1 << resourceTableHeader.AlignmentShiftCount)
    uint16_t    ResourceLength; // actual number of bytes:
                                // resource.ResourceLength * (1 << resourceTableHeader.AlignmentShiftCount)
    uint16_t    ResourceFlags;  // see RESOURCE_FLAGS_*
    uint16_t    ResourceID;
    uint32_t    Reserved;
} NE_Resource;
</syntaxhighlight>

These have to be decoded in a loop:

# Read a <tt>uint16_t TypeID;</tt>.
# If <tt>TypeID</tt> is 0x0000, this is the end of the resource table; break out of the loop.
# Read the rest of <tt>NE_ResourceTypeHeader</tt>.
# Loop <tt>resourceTypeHeader.ResourceCount</tt> times to read the <tt>NE_Resource</tt> for each resource of the given type.

==== Resource Type IDs and Resource IDs ====

Resource type IDs and resource IDs are either integers or strings.

If the top bit of the ID is set (0x8000), the ID is an integer ID.

If the top bit is not set, the ID is an offset, in bytes relative to the beginning of the resource table, to the type string.

Type strings are encoded as Pascal strings: one <tt>uint8_t</tt> specifying the length, followed by as many bytes containing the characters. Make sure to add the NUL terminator at the end of the string if you are using Pascal strings with C string functions!

=== Resident-Name Table ===

The resident-name table contains the name of the module at index 0 followed by the module's exported procedures that stay resident in memory.

It is found at <tt>neHeader.ResidNamTable</tt> from the beginning of <tt>neHeader</tt>. Its length depends on the length of the strings contained within.

The structure of a resident-name table entry is roughly this:

<syntaxhighlight lang="cpp">
typedef struct {
    uint8_t  NameLength;
    char     Name[NameLength];
    uint16_t OrdinalNumber;
} NE_ResidentNameEntry;
</syntaxhighlight>

As is often the case with NE structures, the resident-name table must be read in a loop:

# Read <tt>uint8_t NameLength</tt>.
# If <tt>NameLength</tt> is 0x00, this is the end of the table; break out of the loop.
# Read <tt>NameLength</tt> bytes to obtain the <tt>Name</tt>.
# Read <tt>uint16_t OrdinalNumber</tt>.

=== Module-Reference Table ===

The module-reference table is an additional layer of indirection when looking up module names in the imported names table.

It is found at <tt>neHeader.ModRefTable</tt> from the beginning of <tt>neHeader</tt>. Its entries are all of type <tt>uint16_t</tt> and its length in bytes is <tt>neHeader.ModRefs</tt> times 2 (the size of <tt>uint16_t</tt>).

The usage of the module reference table is described below, in the section concerning relocations.

=== Imported-Name Table ===

The imported name table contains the names of modules and procedures that are being imported by the executable. Its entries are referenced by the module-reference table and the relocation tables of segments.

It is found at <tt>neHeader.ImportNameTable</tt> from the beginning of <tt>neHeader</tt>. Its entries are all Pascal strings:

<syntaxhighlight lang="cpp">
typedef struct {
    uint8_t  NameLength;
    char     Name[NameLength];
} NE_ImportedNameTableEntry;
</syntaxhighlight>

i.e. one <tt>uint8_t</tt> of length followed by that many bytes of characters. (Make sure to add the NUL terminator at the end of the string if you are using Pascal strings with C string functions!)

The usage of the imported name table is described below, in the section concerning relocations.

=== Entry Table ===

The entry table contains information about the entry points in the executable and is organized in bundles.

It is found at <tt>neHeader.EntryTableOffset</tt> from the beginning of <tt>neHeader</tt>. Its format differs depending on the segment indicator of each bundle. Let's begin with a few definitions:

<syntaxhighlight lang="cpp">
#define ENTRY_FLAGS_EXPORTED   0x01
#define ENTRY_FLAGS_GLOBALDATA 0x02

typedef struct {
  uint8_t  EntryFlags;       // see ENTRY_FLAGS_*
  uint16_t EntryPointOffset; // entry point offset within segment
} NE_FixedSegmentEntry;

typedef struct {
  uint8_t  EntryFlags;       // see ENTRY_FLAGS_*
  uint16_t Int3fh;           // constant value (interrupt opcode)
  uint8_t  SegmentNumber;    // number of the movable segment
  uint16_t EntryPointOffset; // entry point offset within segment
} NE_MovableSegmentEntry;
</syntaxhighlight>

The entry table can be decoded bundle for bundle in a loop as follows:

# Read <tt>uint8_t EntryCount</tt>.
# If <tt>EntryCount</tt> is 0x00, this is the end of the entry table; break out of the loop.
# Read <tt>uint8_t SegmentIndicator</tt>.
# If <tt>SegmentIndicator</tt> is 0x00, this is a bundle of unused entries. Read no further bytes for this bundle, mark the next <tt>EntryCount</tt> ordinals as unused and continue with step 1.
# If <tt>SegmentIndicator</tt> is 0xFF, these are movable segment entries. Read <tt>EntryCount</tt> structures of type <tt>NE_MovableSegmentEntry</tt>, then continue with step 1.
# If <tt>SegmentIndicator</tt> is any other value, these are fixed segment entries. Read <tt>EntryCount</tt> structures of type <tt>NE_FixedSegmentEntry</tt> (using <tt>SegmentIndicator</tt> as the segment number), then continue with step 1.

=== Nonresident-Name Table ===

The nonresident-name table contains a description of the module at index 0 followed by the module's exported procedures that do not stay resident in memory.

It is found at <tt>neHeader.OffStartNonResTab</tt> from the '''beginning of the executable file''', in contrast to all other NE tables. Its length depends on the length of the strings contained within.

Its format is the same as that of the resident-name table, described in a previous section.

=== Relocation Data ===

If a segment has the <tt>SEGFLAGS_HAS_RELOCS</tt> in <tt>SegFlags</tt> set, its content is followed directly by its relocation table.

To access a segment's relocation data, seek to <tt>segEnt.SectorBase * (1 << neHeader.FileAlnSzShftCnt) + segEnt.SegBytes</tt>. The structure is as follows:

<syntaxhighlight lang="cpp">
#define ENTRY_FLAGS_EXPORTED   0x01
#define ENTRY_FLAGS_GLOBALDATA 0x02

typedef struct {
  uint16_t RelocEntryCount;
  NE_SegRelocationEntry Entries[RelocEntryCount];
} NE_SegRelocationInfo;

typedef struct {
  uint8_t  Source;            // see RELOC_SOURCE_*
  uint8_t  FlagsAndTarget;    // one of RELOC_TARGET_* OR'ed with any number of RELOC_FLAGS_*
  uint16_t SourceChainOffset;
  union { // variant depends on target within FlagsAndTarget (see RELOC_TARGET_*)
    struct {
      uint8_t  SegmentNumber; // 0xFF == movable segment, other value == fixed segment
      uint8_t  Zero;          // always 0
      uint16_t SegmentIndex;  // if fixed segment: offset into segment;
                              // if movable segment: ordinal number index into Entry Table
    } InternalRef;
    struct {
      uint16_t ModRefTableIndex; // index into module reference table
      uint16_t ProcNameOffset;   // offset from start of imported-names table to procedure name string
    } ImportName;
    struct {
      uint16_t ModRefTableIndex; // index into module reference table
      uint16_t OrdinalNumber;    // ordinal number of the procedure
    } ImportOrdinal;
    struct {
      uint16_t OSFixupType; // see OS_FIXUP_TYPE_*
      uint16_t Zero;        // always 0
    } OSFixup;
  } Value;
} NE_SegRelocationEntry;

#define RELOC_SOURCE_LOW_BYTE 0x00
#define RELOC_SOURCE_SEGMENT 0x02
#define RELOC_SOURCE_FAR_ADDR 0x03 /* (32-bit pointer) */
#define RELOC_SOURCE_OFFSET 0x05 /* (16-bit offset) */

#define RELOC_TARGET_INTERNAL_REF 0x00
#define RELOC_TARGET_IMPORT_ORDINAL 0x01
#define RELOC_TARGET_IMPORT_NAME 0x02
#define RELOC_TARGET_OS_FIXUP 0x03
#define RELOC_TARGET_MASK 0x03

#define RELOC_FLAGS_ADDITIVE 0x04
#define RELOC_FLAGS_MASK 0xFC

#define OS_FIXUP_TYPE_FIARQQ_FJARQQ 0x0001
#define OS_FIXUP_TYPE_FISRQQ_FJSRQQ 0x0002
#define OS_FIXUP_TYPE_FICRQQ_FJCRQQ 0x0003
#define OS_FIXUP_TYPE_FIERQQ        0x0004
#define OS_FIXUP_TYPE_FIDRQQ        0x0005
#define OS_FIXUP_TYPE_FIWRQQ        0x0006
</syntaxhighlight>

To find the module name for an <tt>IMPORT_NAME</tt> or <tt>IMPORT_ORDINAL</tt> relocation, do the following:

# Remember your current location in the file.
# Calculate the offset <tt>startOfNeHeader + neHeader.ModRefTable + 2 * (segRelocationEntry.ImportName.ModRefTableIndex - 1)</tt> and seek to it.
# Read <tt>uint16_t moduleNameOffset</tt>.
# Calculate the offset <tt>startOfNeHeader + neHeader.ImportNameTable + moduleNameOffset</tt> and seek to it.
# Read the Pascal string (one <tt>uint8_t</tt> of length, that many bytes of character data) at that location. (Make sure to add the NUL terminator at the end of the string if you are using Pascal strings with C string functions!)
# Seek to the location remembered in the first step.

(You may wish to store the names in a lookup table mapping <tt>ModRefTableIndex</tt> to module names to speed up repeated lookups of the same index.)

To find the procedure name for an <tt>IMPORT_NAME</tt> relocation, do the following:

# Remember your current location in the file.
# Calculate the offset <tt>startOfNeHeader + neHeader.ImportNameTable + segRelocationEntry.ImportName.ProcNameOffset</tt> and seek to it.
# Read the Pascal string (one <tt>uint8_t</tt> of length, that many bytes of character data) at that location. (Make sure to add the NUL terminator at the end of the string if you are using Pascal strings with C string functions!)
# Seek to the location remembered in the first step.

== Dynamic Libraries ==

If the module is a DLL, the CS:IP in the header is instead the load procedure of the DLL and SS:SP is unused (since libraries do not get their own stacks). It takes the module handle in AX, which is an opaque data type that represents an EXE or a DLL when loaded in memory. On return it outputs a AX!=0 for SUCCESS and AX==0 for failure, which is unusual but very important.

==See Also==
===NE===
*[http://benoit.papillault.free.fr/c/disc2/exefmt.txt Executable-File Header Format]
*[http://www.fileformat.info/format/exe/corion-ne.htm fileformat.info]
===Similar Formats===
*[[MZ]]
*[[PE]]

[[Category:Executable Formats]]
````

<a id="w001958"></a>

## W001958 — LE

Bron-ID: W001958
Namespace: Main (0)
Revisie: 30374 | Datum: 2026-03-22T23:32:17Z
Bron: https://wiki.osdev.org/LE
Revisie-URL: https://wiki.osdev.org/index.php?title=LE&oldid=30374

````text
{{Stub}}
{{File formats}}

'''LE''' (Linear Executable) is a executable file format designed by Microsoft for IBM's OS/2 operating system. It supports both 16 and 32 bit segments in a single file. A 32-bit only variant named "LX" was introduced for OS/2 later.

== Operating System Support ==
* OS/2
* [[Watcom|Watcom Compiler/Extender]] (DOS)
* Windows 3.x and 9x VxD drivers

== See Also ==
* [https://web.archive.org/web/20150810172739/https://hobbes.nmsu.edu/download/pub/os2/dev/info/lxexe.doc Linear Executable Specification] (DOC format)

[[Category:Executable Formats]]
````

<a id="w001959"></a>

## W001959 — PE Binaries

Bron-ID: W001959
Namespace: Main (0)
Revisie: 3302 | Datum: 2007-03-11T05:21:58Z
Bron: https://wiki.osdev.org/PE_Binaries
Revisie-URL: https://wiki.osdev.org/index.php?title=PE_Binaries&oldid=3302
Doorverwijzing: PE

````text
#REDIRECT [[PE]]
````

<a id="w001960"></a>

## W001960 — Rawwrite

Bron-ID: W001960
Namespace: Main (0)
Revisie: 20231 | Datum: 2017-01-23T00:58:39Z
Bron: https://wiki.osdev.org/Rawwrite
Revisie-URL: https://wiki.osdev.org/index.php?title=Rawwrite&oldid=20231

````text
RawWrite is a tool for writing and reading raw images from/to floppy disks. The Windws version is called 'Rawwritewin' 

== Using RawWrite ==
=== Command Line ===
There are two command lines, one for reading and one for writing.
To read the content of a floppy into an image file, use:
 rawwritewin --read --drive driveno file.img
To write an image file to floppy, us:
 rawwritewin --write --copies n --drive driveno file.img
The drives are indexed, where 0 indicates the first/default drive.

=== GUI ===
I think there is no need to explain the use of the GUI because it is self-explanatory.

== See also ==
=== External links ===
*http://www.chrysocome.net/rawwrite - download RawWrite for Windows

[[Category:Disk Image Utilities]]
````

<a id="w001961"></a>

## W001961 — System Calls

Bron-ID: W001961
Namespace: Main (0)
Revisie: 30333 | Datum: 2026-03-17T20:54:10Z
Bron: https://wiki.osdev.org/System_Calls
Revisie-URL: https://wiki.osdev.org/index.php?title=System_Calls&oldid=30333

````text
System Calls are used to call a kernel service from user land. The goal is to be able to switch from user mode to kernel mode, with the associated privileges. Provided system calls depend on the nature of your [[Kernels|kernel]].

==Possible methods to make a System Call  ==

===Interrupts===
The most common way to implement system calls is using a software [[interrupt]]. It is probably the most portable way to implement system calls. Linux traditionally uses interrupt 0x80 for this purpose on x86. Other systems may have a fixed system call vector (e.g. PowerPC or Microblaze).

To do this, you will have to create your interrupt handler in Assembly. This is because your system call ABI will likely not correspond to the normal ABI the compiler supports. It is therefore necessary to translate from one to the other.

For example, on i386, the Linux kernel gets its arguments in <code>eax, ebx, ecx, edx, esi, edi, and ebp</code> in that order. The ABI however places all arguments in reverse order on the stack. Linux proceeds to construct a <code>pt_regs</code> structure on the stack and passes a pointer to it to a C function to handle the call itself. This can be simplified into something like this:

<syntaxhighlight lang="asm">
Int128Handler:
    ; already on stack: ss, sp, flags, cs, ip.
    ; need to push ax, gs, fs, es, ds, -ENOSYS, bp, di, si, dx, cx, and bx
    push eax
    push dword gs
    push dword fs
    push dword es
    push dword ds
    push dword -ENOSYS
    push ebp
    push edi
    push esi
    push edx
    push ecx
    push ebx
    push esp
    call do_syscall_in_C
    add esp, 4
    pop ebx
    pop ecx
    [...]
    pop es
    pop fs
    pop gs
    add esp, 4
    iretd
</syntaxhighlight>

Many protected mode OSes use EAX to hold the function code. DOS uses the AX register to store the function code — AH for the service and AL for functions of the service, or AH for the functions if there are no services. For example, let's say you have read() and write(). The codes are 1 for read() and 2 for write() from the interrupt 0A9h (an arbitrary choice, possibly wrong). You can write
<syntaxhighlight lang="asm">
 IntA9Handler:
     CMP AH, 1
     JNE .write
     CALL _read
     JMP .done
 .write:
     CMP AH, 2
     JNE .badcode
     CALL _write
     JMP .done
 .badcode:
     MOV EAX, 0FFFFFFFFh
 .done:
     IRETD
</syntaxhighlight>

However, if all function codes are small contiguous numbers, a better option might be a function table, such as:

<syntaxhighlight lang="asm">
dispatch_syscall:
    cmp eax, NR_syscalls
    ja .badcode
    jmp [syscall_table+4*eax]
.badcode:
    mov eax, -ENOSYS
    ret
</syntaxhighlight>

Note that this assumes the syscall table to be NULL free. If there is a hole in the table, fill it with a pointer to a function returning an error code!

===Sysenter/Sysexit (Intel)===
{{Main|Sysenter}}

On Intel CPU, starting from the Pentium II, a new instruction pair sysenter/sysexit has appeared. It allows a faster switch from user mode to kernel mode, by limiting the overhead of changing mode. The sysenter entry point will have the kernel stack set already. However, sysenter does absolutely no state saving, so the user stack pointer and return address both have to either be well-known values, or have to be saved by the user-space code leading up to the sysenter. Also, besides unconditionally clearing the interrupt and VM flags, sysenter will not modify any flags.

A similar instruction pair has been created by AMD: Syscall/Sysret. However the behaviour of these instructions are different from Intel's. The syscall entry point will still have the user space stack loaded, and will have to save it and load the kernel stack. The only reasonable way to do this is by way of a CPU-local variable: By way of the <code>swapgs</code> instruction, a CPU-local pointer can be loaded, behind which the user stack pointer can be saved, before overwriting the stack pointer with the kernel value (which also can be saved among the CPU-local variables). In 32-bit mode you get a bit of a chicken-and-egg scenario: You can't save any register to stack, since the stack in question is user stack and thus not to be trusted or modified, and in an SMP system, you can't use any global state, either. And you need to save pretty much all registers, so you can't modify them. So, possibly avoid syscall in 32-bit mode.

In 64-bit mode, the flags register can be modified by way of the SFMASK MSR. The original RFLAGS value will be saved in r11.

Note that, although these instructions did appear in pairs, there is no actual need to keep these instructions paired. With a properly constructed stack-frame, a system call that was started with <code>syscall</code> can be ended with <code>iret</code>.

The kernel can specify which registers are preserved and which registers are lost on SYSENTER or SYSCALL (with the exception of r11 in 64-bit mode, which is always lost) as part of its syscall ABI. It then does not need to save all registers but only those specified as being preserved. Most commonly the C calling conventions in use are followed. By using a tiny assembler stub that calls SYSENTER or SYSCALL the C compiler will safeguard caller saved registers. The kernel entry point for SYSENTER or SYSCALL can then be another small assembler stub that avoids changing any callee saved register before calling a C function for the syscall. That way only the user space stack pointer (and r11 in 64-bit mode) need to be saved as everything else is either preserved by the C compiler or allowed to be destroyed.

Note that for security reasons the kernel should zero or restore all the registers that are not preserved across SYSENTER or SYSCALL so no information is accidentally leaked from kernel to userspace.

===Trap===
Some OSes implement system calls by triggering a CPU [[Trap]] in a determined fashion such that they can recognize it as a system call. This solution is adopted on some hardware by Solaris, by L4, and probably others.

For example, L4 use a "LOCK NOP" instruction on x86. Since it is not permitted to perform a lock on the "NOP" instruction a trap is triggered. The problem with this approach is that there is no guarantee the "LOCK NOP" will have the same behavior on future x86 CPU. They should probably have used the "UD2" instruction, since it is defined for this purpose.

===Call Gates (Intel)===
The 80386 family of processors offer various call gates as part of the [[GDT]]. The call gate is a far pointer that can be called similar to calling a normal function. Very few operating systems use call gates.

To use a call gate in 32-bit mode, an entry has to be added to the GDT. Assuming the standard two-tier architecture (kernel is ring 0, user is ring 3, and rings 1 and 2 are unused), the DPL of that segment needs to be 3, the first two bytes and last two bytes of the descriptor (i.e. the limit, flags, and high base fields) need to be set to a pointer to the handler function, the second two bytes (the low base field) needs to be set to the kernel code segment selector, the mid-base field can contain a parameter count (up to 31 DWORDs can be copied from user to kernel stack), and the access byte has to be set up like this: Pr must be 1, Privl must be 3, the bit the [[GDT]] page claims must be 1 (which is called S in the AMD documentation) must actually be 0, and below that a value of 1100 causes the segment to be seen as a 32-bit call-gate.

To use the gate, user-space code must use a far-call instruction. The offset will be ignored. Assuming the gate is the first entry in the GDT, segment 0x0b will have to be requested (offset 8 and RPL 3):

<syntaxhighlight lang="asm">
call far 0x0b:0
</syntaxhighlight>

In 64-bit mode, the descriptor size is doubled, with the high half of the handler address directly after the rest of the descriptor described above. Also, the argument count has to be zero, and the second DWORD of the second descriptor has to be all zeros. Otherwise, no changes.

==Passing Arguments==

===Registers===
The easiest way to pass arguments to a System Call handler are the registers. The [[BIOS]] takes arguments this way.

Pros:
*very fast
Cons:
*limited to the number of available registers
*caller has to save/restore the used registers if it needs their old values after the System Call
*insecure (if the caller passes more/less arguments than the callee assumes to get)

===Stack===
It is also possible to pass arguments through the [[stack]].

Pros:
*nested System Calls are possible
*it is easy to implement a System Call handler in C because C uses the stack to pass arguments to functions, too
*not limited
Cons:
*insecure (if the caller passes more/less arguments than the callee assumes to get)

===Memory===
The last common way to pass arguments is to store them in memory. Before making the System Call the caller must store a pointer to the argument's location in a register for the System Call handler. (assuming this location is not fixed)

Pros:
*not limited
*secure
Cons:
*one register is still needed
*nested System Calls are not possible without copying arguments
*insecure (if the caller passes more/less arguments than the callee assumes to get)

==Security/safety implications==
Since the kernel is running at higher privilege than the user mode code calling it, it is imperative to check everything. This is not merely paranoia for fear of malicious programs, but also to protect your kernel from broken applications. It is therefore necessary to check all arguments for being in range, and all pointers for being actual user land pointers. The kernel can write anywhere, but you would not want a specially crafted <code>read()</code> system call to overwrite the credentials of some process with zeroes (thus giving it root access).

As for making sure that pointers are in range, checking if they point to user or kernel memory can be difficult to do efficiently unless you are writing a [[Higher Half Kernel]]. For checking all user space accesses for being valid, you can either check with your [[Page Frame Allocation|Virtual Memory Manager]] to see if the requested bytes are mapped, or else you can just access them and handle the resulting page faults. Linux switched to doing the latter from version 2.6 onwards.

==On the user land side==
While the developer can trigger a system call manually, it is probably a good idea to provide a library to encapsulate such call. Therefore you will be able to switch the system call technique without impacting user applications.

Another way is to have a stub somewhere in memory that the kernel places there, then once your registers are set up, call that stub to do the actual system call for you. Then you can swap methods at load time rather than compile time.

Note that whatever library you provide, you cannot assume the user to call the system with that stub. They can, and will, call the system directly if given half the chance.

==Strategies Conclusion==
The system call strategy depends on the platform. You may want to use different strategy depending on the architecture, and even switch strategy depending on the hardware performance. You might also need more parameter copying on a [[Microkernel|microkernel]] than you will need on a [[Monolithic_Kernel|monolithic]] one.

==See Also==
===Threads===

* [[Topic:13186|System call implementation]]

===External Links===
* [[Wikipedia:System call|System Call]] on Wikipedia.
* [http://www.sandpile.org/post/msgs/20003633.htm sandpile.org - Re: SYSENTER SYSCALL] - An explanation on using sysenter / sysexit / syscall.
* [http://www.pagetable.com/?p=9 Asking the kernel how to make a syscall] - Just the same, with notes on how the L4 microkernel is impacted by this.
* [http://manugarg.appspot.com/systemcallinlinux2_6.html Sysenter Based System Call Mechanism in Linux 2.6] - Explains the how and why of why Linux changed their System Call procedure for Pentium II+ machines.
* [http://www.freebsd.org/doc/en_US.ISO8859-1/books/developers-handbook/x86-system-calls.html FreeBSD Developers' Handbook - System Calls] - Discusses System Calls in FreeBSD from the usermode perspective.

[[Category:System Calls]]
[[Category:OS theory]]
````

<a id="w001962"></a>

## W001962 — RAWRITE

Bron-ID: W001962
Namespace: Main (0)
Revisie: 3316 | Datum: 2007-03-11T13:40:29Z
Bron: https://wiki.osdev.org/RAWRITE
Revisie-URL: https://wiki.osdev.org/index.php?title=RAWRITE&oldid=3316
Doorverwijzing: Rawwrite

````text
#REDIRECT [[Rawwrite]]
````

<a id="w001963"></a>

## W001963 — Talk:Makefile

Bron-ID: W001963
Namespace: Talk (1)
Revisie: 19778 | Datum: 2016-09-05T11:09:21Z
Bron: https://wiki.osdev.org/Talk:Makefile
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMakefile&oldid=19778

````text
== rm ==

if [ -f $$file ]; then rm $$file; fi;

what about using rm -f, or $(RM) which is by default defined as rm -f ?

Also, are you very sure you can mark an explanation of your own makefile as a tutorial for makefiles? It seems quite arrogant.

-- Candy

: The "mark an explanation of your own makefile as a tutorial for makefiles" does seem a little weird. Maybe you just got that strange naming convention and programmers block I get sometimes. .. maybe all the time. Look I am doing it now. "Makefile: Tips And Tricks". <Have mercy on me..>
: -- [[User:kmcguire|kmcguire]] 10:40 AM CST 11 March 2007

:Everything is relative, so is arrogance. A practical example may be a good starting point, even if it is your own code. As long as it serves its purpose and people learn from it, why should we care?. You have an alternative? ;) - [[User:Combuster|Combuster]] 03:05, 12 March 2007 (CDT)

Sorry for taking so long to reply, I'm not a "regular" to the new Wiki (yet?). Candy, you are of course right about the $(RM) implicit variable. The "if"-wrapper is being done to keep the Makefile quiet even if a file from the list does not exist (which is not, in fact, an error, and thus shouldn't bother the user). I know I had problems with "only" rm -f to this end, but would have to do some testing to give an exact explanation.
As for the "arrogance" part... you can't write "the" tutorial about makefiles, because make is far too diverse. You can use them to run everything and the kitchen sink. I thought the PDCLib Makefile - being AFAIK correct, non-trivial, and yet not overly long / complex - would be a good example for others to learn a few tricks about Makefiles, especially the dependency handling and non-recursiveness. I shall make the wording a bit more general, and add a note that this is meant as a ''starting-point'', with feedback / requests for extensions being very welcome. Sorry if it sounded too arrogant. [[User:Solar|Solar]] 03:02, 28 March 2007 (CDT)

:Sorry but right now I can't be bothered to do this. I have more or less dropped osdev.org completely, and am spending my time on other projects. I might return later, but for now feel free to rewrite this any way you like. [[User:Solar|Solar]] 23:31, 22 April 2007 (CDT)

Looks better to me. I especially liked the fact that I found a neat section on extracting the ''TODO:''s from a source tree which I just never have thought about doing.

--[[User:Kmcguire|Kevin]] 18:22, 29 April 2007 (CDT)

I did a rework today, including some explanation on why I used the actual PDCLib Makefile instead of giving general tipps. I hope you like it. [[User:Solar|Solar]] 04:20, 21 January 2008 (CST)

== $(RM) wildcarding ==

-@for file in $(OBJFILES) $(DEPFILES) $(TSTFILES) $(REGFILES) pdclib.a pdclib.tgz; do if [ -f $$file ]; then rm $$file; fi; done

Wow, that is ugly, and IMO needlessly complicated. A much better rule would be:

@$(RM) $(wildcard $(OBJFILES) $(DEPFILES) $(TSTFILES) $(REGFILES) pdclib.a pdclib.tgz)

The wildcard function returns a list of only the files that exist, so the if statement and for loop are no longer necessary. Likewise, the - to ignore failures from $(RM) is unneccessary because all files are guaranteed to exist. Although if you don't have permission to delete them, then
a failure will stop any other rules that may be in that target from being run, but that's correct bahavior anyway.
--[[User:Quok|quok]] 07:05, 26 August 2008 (UTC)

:Very nice. -- [[User:Solar|Solar]] 08:43, 26 August 2008 (UTC)

== Recursive Make ==

I'm not an expert in makefiles and I do understand the overhead inolved in recursive invocations of "make". However, parallel builds are impossible without recursivity as far as I can tell. Feel free to correct me.--[[User:Love4boobies|Love4boobies]] 00:44, 10 April 2010 (UTC)

Make will build anything in parallel, as long as you use -j[N]. [[User:Nedbrek|Nedbrek]] 10:47, 12 April 2010 (UTC)

:And it's actually the other way around - recursive Makefiles are always in danger of not representing the whole dependency graph, and the more processes you run in parallel, the higher the chances to break something. Monolithic Makefiles don't suffer from this problem.
:The parallelity doesn't come from multiple ''Makefiles'' being executed in parallel, but in multiple ''rules'' being run. (You have 100 .c files to be turned into .o files, so you can run 100 compiler processes if you feel like it.) -- [[User:Solar|Solar]] 14:26, 13 April 2010 (UTC)

::You are right, of course. Thank you for clearing up how parallel building actually works, Solar.--[[User:Love4boobies|Love4boobies]] 09:22, 14 April 2010 (UTC)

== Extended todolist... ==

I disagree with Virtlink's edit [http://wiki.osdev.org/index.php?title=Makefile&diff=6210&oldid=6209]. Indeed in the PDCLib makefile that was the "template" for this article, I have two different targets, one for TODO: comments and one for FIXME: comments, as the two represent very different levels of urgency; I only presented one of the two as I felt it would be trivial to adapt this to personal choice. You usually don't want TODOs in a FIXME-list, and vice versa. The -F option may make the grep faster, but I feel it's a bit non-KISS. And while we're at it printing file names, why not line numbers to go? Suggestion:

<pre>
todolist:
-@for file in $(ALLFILES); do grep -Hn TODO $$file; done; true
</pre>
This will print a list of all ''TODO'' comments from my files, complete with file name and line number. Nice to be remembered what's still missing before you do a release.



:'fgrep' seems simple enough, and has advantages, especially when you have a big project to build. On the other hand, '-Hn' (or '-H' for that matter) seems also a bit non-KISS to me, however it may be useful. And it is not that trivial to add another keyword, because you need the '-e' switch for that. To me (any possibly to you too), FIXME's are more urgent than TODO's (in the spirit of 'what's missing before you do a release') so maybe we should use those in the example instead. I suggest the following (or with 'fixme' replaced by 'todo'): [[User:Virtlink|Virtlink]] 16:03, 20 August 2008 (UTC)
<pre>
fixmelist:
    -@for file in $(ALLFILES); do fgrep -Hn -e FIXME $$file; done; true
</pre>
:This will ''grep'' all those ''FIXME'' comments from the files and display them in the terminal. It's nice to be remembered of what's still missing before you do a release. To add another keyword, just add another <code>-e keyword</code>. Don't forget to add <code>fixmelist</code> to your <code>.PHONY</code> list.

== Bug in Dependencies, pt. 3? ==

Should 

    %.o: %.c Makefile
            @$(CC) $(CFLAGS) -DNDEBUG -MMD -MP -MT "$*.d $*.t" -g -std=c99 -I./includes -I./internals -c $< -o $@

not be 

    %.o: %.c Makefile
            @$(CC) $(CFLAGS) -DNDEBUG -MMD -MP -MT "$*.o $*.t" -g -std=c99 -I./includes -I./internals -c $< -o $@

instead?
--[[User:Mikeroz|Mikeroz]] 01:13, 1 August 2009 (UTC)

Not quite. The $*.d is needed to recreate the dependency file itself. But you are absolutely right that the $*.o was missing. Funny that no-one else pointed this out (and a shame that I didn't figure this one out earlier, especially since I've got a corresponding bug sitting in my PDCLib tracker for ages...). -- [[User:Solar|Solar]] 14:27, 19 June 2010 (UTC)

== Added "What Is A" and a basic example to aid complete beginners to makefiles ==

I have added two sections into this article and felt they required a brief spot of justification.  Whilst the prerequisites for this wiki include an in-depth knowledge of C, not all flavours of C (maily the Microsoft flavours) use makefiles and some people (like myself) can have survived their whole life of C experience never even having laid eyes on a Makefile.  The first section, "What is a Makefile?", is my attempt to bridge this gap and hopefully convey some of the benefits of having one of these.
Moving on, I have added the most basic of basic examples that I could think of to address the fact that the first actual look at a makefile appears more than half way down this fairly sizable beginner tutorial document.
I hope I'm not treading on anyone's toes here, the tutorial is very good, but speaking from the starting point of my experience it needed this little intro.
Feel free to edit or remove.--[[User:Kenny|Kenny]] 19:12, 19 July 2010 (UTC)

== Which make to target? ==

Should this article target GNU make or POSIX make? I think it wouldn't be bad idea to rewrite this article to conform POSIX, because those features can be trusted to be available unlike some GNU extensions (try to look for pattern rules in pmake, for example), and if someone wants to know if his/her specific make know some fancier tricks, the manual is available. Maybe some tricks could be even shortly listed in the end of the article. After quick skimming I noticed two things which would need changing, pattern rules and usage of <tt>:=</tt> to create macros. IMO at least it should be noted that this article talks about GNU make.

And by the way, usage of <tt>=</tt> to create macros can also be sign of trying to be portable instead of an error... [[User:Fronty|Fronty]] 20:58, 6 April 2011 (UTC)

:It ''does'' say it is targeting GNU make, right in the intro of the Tutorial section. There are several reasons I wrote it like this.
:First, I did not write this tutorial in limbo; the example Makefile presented closely reflects the Makefile I use for PDCLib. Any error I find there gets corrected here, too, which likely wouldn't happen if this tutorial were not connected to a "live" Makefile like that. (And there have been a couple of errors over time, like a complete screw-up of dependency handling I would never have discovered if it had been tutorial code only.)
:All platforms I have access to, both privately and professionally, have a GNU make installation on them, or could be fitted with one in ''no'' time (make being one of the few tools having ''zero'' dependencies and making true of the <tt>./configure && make install</tt> promise). And figuring out ''one'' make manual was quite enough for me. Two reasons I don't feel like digging through other make's manuals to find out the common denominator...
:Then there's the thing with common denominators... <tt>=</tt> may be more portable, but it is ''also'' highly ''inefficient'' (as the expression gets re-evaluated every time it appears in a rule). There is no such thing as the <tt>.PHONY</tt> target in POSIX make (I just checked), so you're open for faulty behaviour on that flank.
:That's not touching on where I simply prefer the GNU notation (e.g. "%.o: %c") over the POSIX one (".c.o:") for expressiveness.
:Bottom line, if you insist on re-structuring this tutorial to target POSIX instead of GNU make, I won't be starting an edit war over it, :but I think GNU make has some distinct advantages. -- [[User:Solar|Solar]] 13:15, 7 April 2011 (UTC)
:I kind of like the idea of portable makefiles but to be fair, it's better to be practical sometimes. The POSIX ''make'' is sort of rudimentary; I don't even know any projects actually trying to achieve ''make'' compatibility. Furthermore, most people are turning away from make (IMHO, for good reason). --[[User:Love4boobies|Love4boobies]] 15:59, 7 April 2011 (UTC)
::"Most people are turning away from make"? Not in the C/C++ world, they don't. (CMake, Automake, they all use make in the end, and I haven't heard of any non-make alternative making the mainstream.) -- [[User:Solar|Solar]] 05:15, 8 April 2011 (UTC)
:::Fair enough but in the same sense that CMake uses make, C compilers usually use assembly. What I meant was that makefiles tend to become very cumbersome for larger projects and that makes people want to learn something else instead. --[[User:Love4boobies|Love4boobies]] 08:22, 8 April 2011 (UTC)
:I don't understand how I could miss that GNU make really is mentioned. Maybe I really should grep more thoroughly before complaining. :-)
:Yes, I missed .PHONY target. That isn't very big problem, though. The trick is to add a dummy dependency which does nothing to a phony target. Maybe it isn't beautiful, but .PHONY IMO is quite ugly too.
:I checked out PDCLib to see how much worse would the build time be if all the <tt>:=</tt> would be changed to <tt>=</tt> when built with GNU make on Ubuntu 10.04 live cd (my FreeBSD installation has some attitude problems right now). Worst differences were about 0.5s, best differences under 0.1s. I got similar results with version Ubuntu stock make (version 3.81) and 3.75 built from source, so I think this isn't due to some relatively new optimization. I don't think that is highly inefficient.
:So... Looks like we're going to live rest of our lives happily with our opinions on efficiency and aesthetics and compatibility and hope we don't stomp on other's feet. [[User:Fronty|Fronty]] 16:50, 9 April 2011 (UTC)
::This whole page is for people who want to make a good Makefile, not for comparing build systems in general. Make is ''quite'' up to the task, it's just that few people ever bothered to actually ''learn'' how to use make efficiently. Oh, and your "trick" for .PHONY doesn't work, if you think about it for a second. (What happens when a file with the name of the dependency exists?) ''And'' you can hardly argue with "getting cumbersome for large projects" on the one hand and the comparatively simple PDCLib not making much of a difference on the other hand. Hey, just buy a computer that's twice as fast, and you can get away with crappy algorithms? I don't think so. -- [[User:Solar|Solar]] 08:14, 11 April 2011 (UTC)
:::This whole page is about that, and this whole discussion is about what does a good Makefile mean, and I think we have found out what is public's opinion on that.
:::Yes, I know the .PHONY "trick" isn't completely foolproof, but IMO double check is good enough in this kind of situation. What the heck are some extra files doing in the source/build directory anyway? This can be always argued, though.
:::Yes, I understand that PDClib is quite simple project. However, that shows well enough that the impact of the type of macro doesn't have that big impact on build speed that I would care. I don't care if my nightly builds are ready a little earlier or later. Full build won't be once-in-an-hour event while developing, and I'll be making tea or something while my computer is installing something and I don't think someone who has already chosen the slow path will want to stare the compilation the whole time. You won't have anything magical in the macros, maybe to the macros containing list of source and object files will be a bit fancier, but they aren't used very much.
:::Can I get away with crappy algorithms with faster computer? No. Can I use maybe a little slower algorithm with some good characteristics if the speed difference won't make any difference in that situation and I fancy the good characteristics? Yes. [[User:Fronty|Fronty]] 13:56, 11 April 2011 (UTC)

== Proper use of CFLAGS ==

I know that using CFLAGS like I did is frowned upon, as it overwrites any environment variable the user might have set. Since I don't feel the pressure in PDCLib, and don't have any other C/C++ project using Makefiles at the moment... does anyone have a canon / working alternative? -- [[User:Solar|Solar]] 03:01, 25 May 2012 (CDT)

:My personal solution is to have a different named set of flags and append CFLAGS as appropriate:

 CPARAMS := "-m32 -DNEEDED_DEFINE -DOTHER_DEFINE -Wall -Wextra -fno-omit-frame-pointer"
 (...)
 %.o: %.c
     $(CC) $(CFLAGS) $(CPARAMS) -c -o $@ $<

:Which under many circumstances has exactly the intended effect, but it does little to prevent conflicting params/flagsets other than hoping they override in order. - [[User:Combuster|Combuster]] 15:00, 25 May 2012 (CDT)



:Oops, somehow I made an earlier comment vanish O.o - [[User:Combuster|Combuster]] 03:06, 26 May 2012 (CDT)
How about CFLAGS += ...? e.g.
<pre>
CFLAGS += -g -Wall

make
gcc -g -Wall -o test test.c

CFLAGS=-O3 make
gcc -O3 -g -Wall -o test test.c
</pre>
:Obviously this can lead to errors if the user-specified flags conflict with those you specify.  [[User:Jnc100|John]] 06:02, 25 May 2012 (CDT)

== Commands Prefixed with @ ==

Is it really a good idea to cause <tt>make(1)</tt> not to print out the executed commands?

When debugging it's probably better to know which command failed.

When not debugging you can pass the <tt>-s</tt> to <tt>make(1)</tt>.

[[User:Glauxosdever|Glauxosdever]] 06:09, 5 September 2016 (CDT)
````

<a id="w001964"></a>

## W001964 — User:Kmcguire/Quick And Dirty Virtual Address Space Scheme

Bron-ID: W001964
Namespace: User (2)
Revisie: 15635 | Datum: 2014-02-25T03:32:21Z
Bron: https://wiki.osdev.org/User:Kmcguire/Quick_And_Dirty_Virtual_Address_Space_Scheme
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKmcguire%2FQuick_And_Dirty_Virtual_Address_Space_Scheme&oldid=15635

````text
= The Quick And Dirty Virtual Address Space Scheme =
This quick and dirty virtual address space scheme will take you step by step through a entire kernel sub-system that will provide the virtual address space for every user space process and the kernel while shadowing all page directories and tables by using direct mapping from physical into the virtual address space. All implementations are broken down into parts, and step by step we examine them together to try and provide not just a copy and paste bucket of code but instead a actual understanding.

== Target Audience ==
If you have never read any of the Intel or AMD documentation for the 80386 and higher generation processors, comprehended a overview of the 80386 and higher virtual address space mechanism, or know the different in a virtual and physical address then you are most likely not a target for this book. Preferably someone who has attempted or successfully implemented a memory management scheme and would like to improve or continue it and needs a idea or some in depth explanation will benefit highly from this book. However you must realize that even if you are not a targeted audience this book could still help you, provide some valuable insight into you're future work, or allow to to realize if you are a targeted audience.

== The Overview ==
I am going to try to lead you as gently as possible through two complete memory management schemes: physical and virtual. The first part of this document is about physical page management. The second part is about virtual page management. Both parts will be covered by explanation and implementation. The only part where you may could become confused is when we implement a limitation in our physical memory management.

I will try to lead you as gently as possible through two complete memory management schemes like I lead my own self when writing something. So not only should you gain a understanding of the contents, but the ability to understand how to break a solution into layers, especially for programming. You should know that this book is broken into two major parts: Physical Page Management, and Virtual Address Space Management. The physical page management will hopefully help you solve problems by giving a in depth and step by step examination of a basic allocator, while the virtual address space management is going to shed light on a scheme that is very successful at providing performance and less debugging.

== Physical Page Management ==
This will provide some ground work, which is essential to the understanding of the virtual page management. We will discuss a few different approaches and generalize what a physical page management implementation should do. I will also use the byte map allocation technique since I feel it is very simple, and the memory requirements are simplistic at best.

Physical memory management is like having a lot, and I mean a lot, of places to put things and of course each of these places, boxes, cupboards, bins, or rooms that need to be kept track of. Imagine having a large warehouse with millions of boxes, and having to walk for thirty to sixty minutes just to reach a box in this building. First to take note of is that this would be literally insane. Second to also note is that the contents of each box is so similar to the other million boxes that just by looking inside one you could never deduce what it represents. This can be the equivalent of a physical memory, not the management since you have to walk in this warehouse to each box to check it. So literally no management exists, and upon that fact it is also just simply impossible to do with out management unless you liked living in a warehouse looking at boxes twenty four seven 

Our management information will help us know things about the place, boxes, cupboards, bins, or rooms if you wish so that we can better find what we are looking for. Luckily the matter at hand in this book and for this chapter is just to keep track of weather something is actually in the box or not. I know the example above is strange in the sense but truly imagine searching through memory looking at the data contained on the page, and trying to determine if it is being used or is free to use just by the data contents. We could keep pages that can be used, which are unused by anything else, zeroed but the possibility arises that something may need to write all zeros thus the example should make sense that of these million boxes there is no way to determine what their contents are for when everything looks extremely similar.

We will provide some ground work here, but nothing overly complicated. A few nuts and bolts will be inserted towards the end, but overall do not expect anything that requires actually engaging the brain into overdrive. We really want to save that for later on. Included in the ground work is information and implementation for the byte map and the encapsulating function calls to work with this byte map by enslaving it to do you're evil bidding and take over the world. Ok. Well. I suppose that would be too much for it to handle, but it does allocate physical memory pages!

=== Discussing The Algorithm ===
There are many alternatives to the technique I will discuss in this section. You are welcomed to try new stuff, and use something even drastically different then what I use here. The only reason not to do something different is if it will keep you from understanding the sections here and forth.

A byte map consists of a sequential list of bytes, where each byte in this list - maps information to a page address. The page addresses we will be mapping to are the physical ones. Each byte holds a value that represents attributes, or just a attribute in our case, about that page. The byte index within the byte map is used to compute the actual page address.

Many algorithms exist that can provide us a allocation mechanism but some have very complex internals, little documentation, or other unnecessary functions and uses which would could cause bloating of the explanation by a exponential factor, decrease likely hood of comprehension, and turn away some people who are not trying to change the world with a memory management scheme. With these in mind I have decided to use the most basic design I could think of. Now this implementation is by no means the most efficient or the fastest but it does provide good all round performance and will keep the bugs down which means less debugging time. From my experience debugging time can make a project become abandoned, and with good reason.

A few algorithms are the [[Page Frame Allocation| Buddy Allocation System]] which is used by the Linux kernel, [[http://www.google.com/search?hl=en&q=Linked+List&btnG=Search| Linked Lists]], [[http://www.google.com/search?hl=en&q=AVL+tree&btnG=Search| AVL Trees]], Map Allocators (which are used here in this book), and many more. You can find more references and links at the ending of this book.

=== The Byte Map Approach ===
The implementation of the allocator algorithm consists of three major parts, and a voodoo doll. No the doll is not included and it must be purchased separately to aid in some religious debugging rituals when everything fails to operate as intended but that should not happen. The initialization, allocation, and deallocation form the three major parts and are in reality very simple or can be extremely complex if you choose to do a different implementation later on so be aware.

[[Image:Kmcguirebitmapnumbered.png|frame|left|The mathematical relationship of the map to memory pages is simple.]]The parts in our byte map allocator are very simple in understanding and implementing, and by implementing I mean the processing of putting some code into effect using a plan which you will find our as we move along. I thought I would save some time for the impatient and do both at the same time. For initialize we need to get things ready. Set default values, or use provided values to configure and optimize. Of course this allocator will do a little less than optimize it's self since I consider that a advanced and a not relevant topic for a quick and dirty virtual address space scheme. A allocation is when you set something apart or designate it for a special purpose. The reason we need to set something apart is to keep other independent parts of our system from meddling with what we what to use while we use it. Our allocation will be written in C and made callable as a function that will return a not designated or free physical page memory address, and right before this record it as being allocated or designated.The deallocation reverses the allocation procedure, and turns this designated or allocated physical page into a unallocated and not designated state. While I have just used the word state I would like you to know that in basic terms we are simply changing the state of a physical memory page.

(Left) A depiction of a byte map and the memory pages associated with each byte. Also for worth noting the byte map could be a bit map which would use bits instead of bytes to represent the state of a page. The address coropsonding to the index in the byte map is used to produce the address in memory by shifting the address to the left by twelve. In the C language a shift operation is denoted by ''(index<<12)'' and thus such as in the depiction is used as text inside each page on the bottom. Also the color represents that byte in the map having a different value. This value is used to set if the byte is allocated or not. A byte can have two-hundred-fifty-six different values due to the eight bits being used to form a byte which range from zero to two-hundred-fifty-five. We only need two values for this simplistic design, but from a tutorial stand point bytes are easier to work with. Try creative uses for the byte map such as storing other information about the page such as: what it is allocated for, who allocated it, what permissions for allocation does it have, and many other possibilities.

=== Implementation Global Data ===
We will need two pieces of information to be declared global. This will prevent us from having to constantly pass this variable to every function call, and indirectly decrease the chance of passing the wrong argument in it's place. It is like accidentally passing degrees to a function when it expects radians since you kept the conversion in the same variable. I hope you would not have much to keep in the same variables (below) but that is another story. 
<pre>

	unsigned char *g_pmm_map = 0;
	unsigned int g_pmm_reqpages;
</pre>

=== The Map Of Bits ===
The map of bits is pointed to by ''g_pmm_map'', which also in turn treats this continuous area of memory as a array of eight bit bytes. As a quick run through we will use ''g_pmm_map'' to access and modify attributes for each page of memory we track. For example to get the attributes for a certain page in memory using any address we shift the address to the right twelve bits and use the result as a index into ''g_pmm_map''. If you are not quite sure you understand why we make the bit shift to the right you need to do a search with [[http://www.google.com/search?hl=en&q=4kb+page+80386&btnG=Google+Search|4kb+page+80386]] on google and then come back. It will not take long. I promise.
<pre>
	attributes = g_pmm_map[address>>12];
</pre>
Of course you may notice that using this calculation we are considering our map starting at page address zero. This will be the default case through the remaining of this writing since: I want to keep the algorithm as simple as possible, and I need the implementation to handle the entire four gigabyte address space if requested.

=== Initialization ===
Lets first prototype the two initialization functions we will need, and I know I said there is only three parts with one of them being initialization. I just did not say how many items are in that section. Do not get discouraged this is a lot easier than you think just read on.
<pre>
	void pmm_setmaxmem(unsigned int max);
	void pmm_init(unsigned int page, unsigned int count);
</pre>
You might know we needed a initialization function such as: ''pmm_init''. But, the ''setmaxmem'' is most likely closer to throwing a monkey wrench in the works - do not worry we can work this out. The ''setmaxmem'' function will help configure the ''pmm'' to support the specified amount of memory. For example if we only need to support at maximum sixteen megabytes of memory, and instead we had no ''setmaxmem'' a default configuration would have to be used which might always initialize for four gigabytes instead. There would have to be a significant amount of wasted memory, and even though we like to waste memory at the beginning it does not make sense to go overboard and drown in clam waters.

Here we need to find out how much of the memory we are about to manage - we need ourselves to do the actual managing. It seems as silly as saying I want to buy something and part of what it costs should pay for it too. Although that sounds like it could confuse the nice store clerk for several seconds I imagine it would not work not because it is impossible since our system of currency would fail. Fortunately we are the money makers or rather the memory owners and we can do what we like which is simply using some of something for that something.
<pre>
	pageCount = (maxmem>>12)+1;
</pre>
Using the above formula we can determine from the maximum amount of memory we wish to support that we will have ''((maxmem>>12)+1)'' pages at most. This directly gives us the required bitmap length in bytes (or bits for a alternative implementation). Also we shift to the right once again. Since ''maxmem'' will be a thirty-two bit value that should represent the amount of memory detected in the system it indirectly holds the number of four kilobyte pages present too. I also add one in the case of a partial page which could be something like 0x1002 (four-thousand-ninety-eight) which is a page and one partial page. You could devise a more advanced mechanism here to detect a partial page and then add one if needed, but I decided to keep things simple.

Lets see if we can work out a simple implementation for this ''pmm_setmaxmem'' function. The function argument is the amount of memory detected in the system, or at least what we want to think it is.
<pre>
void pmm_setmaxmem(unsigned int max){
	g_pmm_reqpages = (max>>12)+1;
}
</pre>
We just set a global variable to hold the result. All we wanted to do is provide a nice interface in the form of a function call to keep the distracting odd names and cryptic variables names out of the calling code.

Now, lets try to implement our ''pmm_init'' function in a layered manner or little steps to help you comprehend actually what is going on instead of a bag of jumbled code that looks more like someone someone never wanted you to read.
<pre>
void pmm_init(unsigned int page, unsigned int count){
	for(x = 0; x < count; ++x){
		g_pmm_map[(page>>12)+x] = 0;
	}
	return;
}
</pre>
We take the range and make sure all bytes for each page in the range are set to zeros to represent a unallocated state or free. We have a problem unfortunately and it lies in the fact that we still need to figure out where our map is in memory. We already know how big it needs to be so the simplest method would be to hope that the first memory range given can hold it. This inherently will introduce problems, but since I wanted a simple base to use to write our virtual memory management implementation I thought we could slip by here and if you like you can come up with something better. To understand how to use it properly see the end of this book in the examples section.
<pre>
void pmm_init(unsigned int page, unsigned int count){
	if(g_pmm_map == 0){
		if(count >= g_pmm_reqpages){
			g_pmm_map = (unsigned char*)page;
			page += (count<<12);
			count -= g_pmm_reqpages;
			pmm_strip((unsigned int)g_pmm_map, g_pmm_reqpages);
		}else{
			return;
		}
	}
	for(x = 0; x < count; ++x){
		g_pmm_map[(page>>12)+x] = 0;
	}
	return;
}
</pre>
Now, you will notice we see if the ''g_pmm_map'' does not exist then check if the first range passed is big enough. The reason we can check the existence of it is because its global declaration gives it a initialization value of zero or null. I include no error handling if the initial memory range, when ''g_pmm_map'' is zero, is not large enough since I feel the actual determination can be made outside of the ''pmm'' which is before the ''pmm_init'' call. There is also a unknown function included just ignore it until a little later but for a little information it is the same as using what something costs to pay for it's self.

[[Image:KmcguireBytemap.png|thumb|center|Various.]] [[Image:KmcguireBytemapPmmMap.png|thumb|center|Various.]]

=== Allocation ===
We will produce a prototype for the allocation function which will be responsible for returning a pointer to one page of memory.
<pre>
	unsigned int pmm_alloc(void);
</pre>

The map of bits is pointed to by ''g_pmm_map'', which also in treats this continuous area of memory as a array of eight bit bytes. As a quick run through we will use ''g_pmm_map'' to access and modify attributes for each page of memory we track. For example to get the attributes for a certain page in memory, using any address we take the address and shift it to the right twelve bits. If you are not quite sure you understand why we make the bit shift to the right you need to do a search with [[http://www.google.com/search?hl=en&q=4kb+page+80386&btnG=Google+Search|4kb+page+80386]] on google and then come back. It will not take long. I promise.
<pre>
	attributes = g_pmm_map[address>>12];
</pre>
Of course you may notice that I have already said this. I wanted to bring it up to the front of you brain so that when I say, "You can also start at array index zero and increment while you a less than ''g_pmm_reqpages''." Lets add a layer that does just this:
We need to walk through the bitmap, and we know that the first page at memory address zero has it's attributes stored at ''g_pmm_map[0]'' since I told you earlier that you may notice that using this calculation we are considering our map starting at page address zero. This will be the default case through the remaining of this writing since: I want to keep the algorithm as simple as possible, and I need the implementation to handle the entire four gigabyte address space if requested. So with that in mind we still have a problem or knowing at what array index to stop? Well the question can be answered by reviewing the ''pmm_setmax'' function and understanding that we can determine from the maximum amount of memory we wish to support that we will have ''((maxmem>>12)+1)'' pages at most. This directly gives us the required bitmap length in bytes (or bits for a alternative implementation). So since we are walking the bytes in the map and we know that each byte represents one page of memory we should deduce that ''g_pmm_reqpages'' used in the ''pmm_setmax'' function holds the number of pages and bytes that our map tracks. Lets walk until we are right below that number since our array starts at index zero not one.
<pre>
	...
	unsigned int x;
	for(x = 0; x < g_pmm_reqpages; ++x){
		// This g_pmm_map[x] is valid.
	}
	...
</pre>

This is extremely useful for one major reason. We can now search the bitmap for pages that are unallocated since we know that there is a direct mapping between the array index and the page address with ''g_pmm_map[address>>12]''. With this new found searching ability we should now be able to satisfy the request for one free page.
<pre>
	...
	unsigned int x;
	for(x = 0; x < g_pmm_reqpages; ++x){
		if(g_pmm_map[x] == 0){
			g_pmm_map[x] = 1;
			return (x<<12);
		}
	}
	...
</pre>
You should notice that we shifted ''x'' left twelve bits. This is because the index is created from a address by shifting that address to the right twelve bits and the reverse will produce the whole address. Taking that ''address == ((address>>12)<<12)''. We simply got the index of a address with ''(address>>12)'' and turned it back into ''address'' using ''(address<<12)''. The only other switch is ''address'' becomes the index ''[]'' and then back.

Lets see if we can write out the entire implementation for the alloc function, and produce a completed ''pmm_alloc'' implementation with no bugs!
<pre>
unsigned int pmm_alloc(void){
	unsigned int x;
	for(x = 0; x < g_pmm_reqpages; ++x){
		if(g_pmm_map[x] == 0){
			g_pmm_map[x] = 1;
			return (x<<12);
		}
	}
}
</pre>

=== Striping ===
[[Image:Kmcguirebitmapstrip.png|frame|left|Some map entries do not contain the same value. This distinguishes between allocated and unallocated, and other states.]] When we strip we are talking about removing a entire physical address range from our allocator. A perfectly good example is where the kernel is loaded, and even where our ''g_pmm_map'' is located. If you remember we left a function call in our ''pmm_init'' function called ''pmm_strip'' and if you think about it long enough you might realize that strip is short for stripping just for brain exercise. Lets try implementing this here starting with our prototype.

(Left) A depiction of green square representing byte in the map that are in a unallocated state, while the red show ones that have been set as allocated. A strip is exactly like a allocation except no address is returned. It just reserves the pages or removes them from ever being allocated by the allocator out of the byte map.
<pre>
void pmm_strip(unsigned int page, unsigned int count){
	unsigned int x;
	for(x = 0; x < count; ++x){
		g_pmm_map[(page>>12)+x] = 1;
	}
	return;
}
</pre>
Its just a matter of running through the ''count'' in a loop, while adding the offset as page) shifted down twelve bits to it and setting a one there. Since nothing should ever allocate it. It should never be freed unless a bug crawls along and buries its self into your code. Ouch. And what good reason would you have for code overwriting the kernels ''.data'' and ''.text'' sections.

=== Deallocation ===
We have seen how to allocate a page by setting the appropriate byte in our map, and as you might have guess we deallocate it by setting this byte back to zero or what ever may be the deallocated value. Lets try our hands at writing it.
<pre>
	void pmm_free(unsigned int address){
		g_pmm_map[address>>12] = 0;
	}
</pre>
Very simple. I know. Since there is a direct mapping between the array index and the page address we just exploit that fact by shifting the address to the right twelve bits, and setting the value to zero to represent a unallocated page.

=== We Need Limitations ===
We need limitations in order to make a slip fit with our virtual memory manager. Some of the careless allocation that we perform above just will not sit well with our virtual page management. I am going to avoid explaining exactly why, but as you read this section you may figure out why before I actually get there. These limitations are in the form of ranges. What we need to do is to break the contiguous memory we are working with into parts, or ranges. In our implementation we only need one range at the moment.

A range is a offset with a count. A range also needs to have a dynamic state during the initialization of our physical memory manager. So we should be able to set a range and leave it for good - in our early form at least! We are only going to use one range right now, and the form will be in a global variable which is accessible through a function call.
<pre>
	unsigned int g_pmm_vmm_offset;
	unsigned int g_pmm_vmm_count;
</pre>

The functions, which need to be self explanatory.
<pre>
	void pmm_set_vmm_offset(unsigned int offset){
		g_pmm_vmm_offset = offset;
		return;
	}
	void pmm_set_vmm_count(unsigned int count){
		g_pmm_vmm_count = count;
		return;
	}
</pre>

=== A Limitation Implementation ===
I know the suspense is killing you, but wait the truth is here. We need to make our implementation understand that it should never allocate pages inside this range that we have given it with ''(g_pmm_vmm_offset)'' to ''(g_pmm_vmm_offset+g_pmm_vmm_count)''. If it does there will be lots of sparks if you use the virtual implementation coming up.

Before we get that far we need to find a way to keep our allocation from ever getting inside this range to even check the byte for a state. Lets take a look at our old culprit.
<pre>
	unsigned int pmm_alloc(void){
		unsigned int x;
		for(x = 0; x < g_pmm_reqpages; ++x){
			if(g_pmm_map[x] == 0){
				g_pmm_map[x] = 1;
				return (x<<12);
			}
		}
		return 0;
	}
</pre>

You will notice we just move through all the bytes until we find one that is a zero, and because the value (x) is a direct mapping to the page address we know what to return to the requester. This is where we need to insert a sanity check of: Is this inside our range?

<pre>
		.....
		for(x = 0; x < g_pmm_reqpages; ++x){
			if((x < g_pmm_vmm_offset) || (x >= (g_pmm_vmm_offset+g_pmm_vmm_count))){
				if(g_pmm_map[x] == 0){
					g_pmm_map[x] = 1;
					return (x<<12);
				}
			}
		}
		.....
</pre>

What this says is, "If ''x'' is less than ''(g_pmm_vmm_offset)'' then we have not made it to the range starting point yet.". It also states that, "If ''x'' is greater than or equal to ''(g_pmm_vmm_offset+g_pmm_vmm_count)'' then we are past the entire range". If either of these two conditions are true, hence the OR operator, then we should continue checking if this page is allocated or unallocated.

The final result will be a contiguous range of pages that are never allocated. The number of pages will be static for now to keep things simple.

<pre>
unsigned int pmm_alloc(void){
	for(x = 0; x < g_pmm_reqpages; ++x){
		if((x < g_pmm_vmm_offset) || (x >= (g_pmm_vmm_offset+g_pmm_vmm_count))){
			if(g_pmm_map[x] == 0){
				g_pmm_map[x] = 1;
				return (x<<12);
			}
		}
	}
	return 0;
}
</pre>

=== Getting Creative ===
Getting creative is going to involve coming up with a better allocation and deallocation algorithm, and even better figuring out a method to dynamically change the ''g_pmm_vmm_*'' range above. But, just for the beginner lets try to focus on making a better allocator rather then a better limitation range until you read the next sections.

== Virtual Page Management ==

This will provide the actual implementation and explanation on top of the previous physical page management ground work. If you have altered or changed the original one to suit needs you have to be sure you can do a nice slip and fit or at least read how I implemented the previous one.

The major disadvantage when working with page directories and tables when you are working in a virtual addressing mode is if the addresses in the directory and tables point to pages that are not directly mapped right into our current address, space since the processors internal mechanism only understands physical pages not virtual ones, it can cause a explosion of sparks or lots of data looking like scrambled eggs.

The obvious fix is to simply made correction to your calculations such as: performing a reverse lookup of the table to find where it is mapped in the current address space, searching a precomputed list of translations just for directories and tables, and others. The reasoning I have not to do either of the two later methods is if you have already mapped the pages into the current address space to even think about performing a reverse lookup then why not have just mapped them in with direct mapping such as mapping physical address 0x183000 to virtual address 0x183000 in the first place? This should save clock cycles from the calculations and just make working with page directories and tables much easier.

[[Image:kmcguirephyandvirt.png|frame|left|The physical and virtual page type colored.]] (Left) A depiction of the virtual and physical address spaces laid out in pages. Of course in the picture above the machine looks like it has less than twenty pages of memory, but it is only for a example. You will notice that are four colors: blue, red, orange, and green. The blue is the direct mapped page directories and tables area in memory, red is miscellaneous kernel allocated pages, orange repersents the actual kernel ''.text'' and ''.data'' sections and such other and also the IDT and GDT type tables which are also direct mapped just like the blue, and green which repersents free memory that can be used by user space processes and even the kernel (which would convert green to red for the kernel and I use no color in the depiction to represent user space allocated pages). You should also take into mind that in the top virtual color bar green is only found to the right. This is where the virtual memory has been split with kernel on the bottom and user space on the top. On the bottom color bar representing physical memory you can see that the green and red pages mix together some which allows the power of the 80386 paging mechanism. You will note too that the blue are required to be direct mapped and this forms the entire premise and maybe encompasses the entire ending part of this tutorial. How do we get blue direct mapped, spit the virtual address space, and let the physical address space mix.

Also we are going to split the virtual address space into two parts. For the sake of keeping things simple I am using the lower part of the four gigabyte address space for the kernel, while the remaining is used for the user space processes. If you do something different it should be trival to understand the modifications necessary.

=== Internal Allocation ===
Our scheme will use two allocation functions. One shall be used internally for allocation of directories and tables while the other will be our original ''pmm_alloc''. We will try to produce a implementation of a internal one first, then work towards the second implementation or external if you will.

If you went through the section for the physical page management you will remember that we talked about limitations. We also implemented a range that the allocator was never supposed to touch. That range is where this internal ''vmm'' allocator will get pages from to use. The reasoning is we need to try to keep from clobbering up random pages in the upper memory which will be used by user space applications, and from clobbering all the available physical memory that has direct mapping potential.

Lets try writing a implementation that will provide internal allocations. We are going to try to use the three global variables from the physical memory manager. If you have written a different physical management scheme you will need to alter this entire function to use you're method.
<pre>
unsigned int pmm_vmm_alloc(void){
	for(x = g_pmm_vmm_offset; (x < (g_pmm_vmm_offset+g_pmm_vmm_count)) && (x < g_pmm_reqpages) ; ++x){
		if(g_pmm_map[x] == 0){
			g_pmm_map[x] = 1;
			return (x<<12);
		}
	}	
	return 0;
}
</pre>

If you notice we provide two conditions. The first says if ''x'' is less than our range's end and if ''x'' is less than the last page in our map then keep going. If not we need to exit with a zero or null. 

=== Helpful Macros ===
Now that we have the start of our implementation written lets design two macros to assist when accessing our page directory and tables. 
<pre>
	#define VMMDIR(t, x) ((unsigned int *)t[(x>>12)])
	#define VMMTABLE(t, x) VMMDIR(t, x)[t>>22]
</pre>

This allows use to specify ''VMMTABLE(vdir, address)'' with ''vdir'' being a unsigned int pointer to our virtual directory, and address being the virtual page address we wish to access inside the table. Of course there are a few holes which you will fall in such as: What if the directory exists but not that certain table? We will cover that next, and even find a example of this happening.

=== Finding Free Pages ===
[[Image:KmcguireFindFreeVirt.png|frame|left|A silly depiction of finding a free virtual page using the page directory and tables.]]Finding free pages is not all too hard. If you are concerned with performance you just might never finish the project, but for the most part keeping a level head and having forward progress is enough to understand this implementation since there is always time for optimization. The first part is our function prototype.
<pre>
unsigned int vmm_findfree(unsigned int *vdir, unsigned int count, unsigned int index4mb, unsigned int icount4mb);
</pre>

Here we want to pass a unsigned int pointer of our page directory, ''count'' as the number of continuous pages we need, ''index4mb'' as the four mega byte page address we may start our search at (pre shifted down twenty two bits), and ''icount4mb'' as the number of four megabyte pages starting at index4mb. This gives us flexibility to tell the function, "Hey, I want this many pages allocated in the kernel's half of memory." or "Hey, I want this many pages allocated in the user processes' half of memory.".

The second part is building a loop so we can move through each table entry of the directory, and if a table exists each of it's entries.
<pre>
	...
	unsigned int x;
	for(x = index4mb; x < (index4mb+icount4mb) ; ++x){
		// check directory entry
		if(vdir[x] != 0){
			for(y = 0; y < 1024; ++y){
				// check table entry
			}
		}
	}
	...
</pre>

Now, this is great - but we are still missing a critical element. We must keep count of how many free pages we find. Lets try that.
<pre>
	...
	unsigned int *table;
	unsigned int x;
	unsigned int cfreepage = 0;
	for(x = index4mb; x < (index4mb+icount4mb) ; ++x){
		if(vdir[x] != 0){
			table = (unsigned int*)(vdir[x] & 0xFFFFF000);
			for(y = 0; y < 1024; ++y){
				if(table[y] != 0){
					++cfreepage;	
				}
			}
		}
	}
	...
</pre>

You will notice I create another unsigned int pointer to hold the table address just to simplify the code. I also used 0xFFFFF000 to zero the lower twelve bits of each directory entry since they only hold attributes for the page. I named ''cfreepage'' to represent the number of consecutive free pages which is flawed, of course! 

Take note that the above implementation is wrong if we encounter a non-free page. This is because instead we should reset on encountering non-free pages. Also flawed is if a table does not exist we are completely ignoring it instead of counting one thousand and twenty four pages. It was holding all those pages and was not allocated right? Lets try a rewrite:

<pre>
	...
	unsigned int *table;
	unsigned int x;
	unsigned int cfreepage = 0;
	for(x = index4mb; x < (index4mb+icount4mb) ; ++x){
		if(vdir[x] != 0){
			table = (unsigned int*)(vdir[x] & 0xFFFFF000);
			for(y = 0; y < 1024; ++y){
				if(table[y] != 0){
					++cfreepage;	
				}else{
					// reset the count
					cfreepage = 0;
				}
			}
		}else{
			// count completely blank tables
			cfreepage += 1024;
		}
	}
	...
</pre>

A working directory and table walking function with the ability to find a consecutive range of free virtual pages will fit nicely in our next sections for a utility function. Lets get a complete version written real quick.

'''THIS FUNCTION IS BROKEN''' 
''The concept will work but it was coded wrong!''
<pre>
unsigned int vmm_findfree(unsigned int *vdir, unsigned int count, unsigned int index4mb, unsigned int icount4mb){
	unsigned int *table;
	unsigned int x;
	unsigned int cfreepage = 0;
	unsigned int opage = index4mb << 22;
	for(x = index4mb; x < (index4mb+icount4mb) ; ++x){
		if(vdir[x] != 0){
			table = (unsigned int*)(vdir[x] & 0xFFFFF000);
			for(y = 0; y < 1024; ++y){
				if(table[y] != 0){
					++cfreepage;
					if(cfreepage >= count){
						return opage;
					}
				}else{
					// reset the count
					cfreepage = 0;
					// this should end up being the first page notice the (y+1) and currently y=y+0.
					opage = (x << 22) + ((y+1) << 12);
				}
			}
		}else{
			// count completely blank tables
			cfreepage += 1024;
		}
	}
	return 0;
}
</pre>

I added the ''opage'' to keep track of the original page we start counting at. There are only two reasons we starting counting from zero. We reach a page that is current in use, or we have just called into the function. Note that ''opage'' equals ''index4mb << 22'', which is the very first four megabyte page we start at (or table). Lets see about writing a mapping implementation.

=== Mapping ===
Mapping is extremely easy except for the now slightly complicated and dangerous corner case scenario we have created with the direct mapping, but.. it is worth it. Lets venture forward with creating our prototype for the first function.
<pre>
void vmm_map_single(unsigned int *vdir, unsigned int phy, unsigned int virt, unsigned int flags){
	VMMTABLE(vdir, virt) = (phy & 0xFFFFF000) | flags;
	return;
}
</pre>

Not bad huh, Surprised? You should be since using that would lead to a explosion of bits and bytes guaranteed. It completely ignores the fact that the table the virtual address resides in might not exists at all. Lets add some code to check for this and try to correct it.
<pre>
	....
	unsigned int table;
	if(VMMDIR(vdir, virt) == 0){
		table = pmm_vmm_alloc();
		VMMDIR(vdir, virt) = table | flags;

		for(x = 0; x < 1024; ++x){
			((unsigned int*)VMMDIR(vdir, virt))[x] = 0;
		}
	}
	VMMTABLE(vdir, virt) = (phy & 0xFFFFF000) | flags;
	....
</pre>

I first check if the table exists. If not I create it and then proceed to clearing it by converting the unsigned int value into a pointer and looping through all one thousand and twenty four entries. Then we set the appropriate entry with the macro ''VMMTABLE''. We have still forgotten one important element to the entire process. That is to keep a direct mapping of the directories and tables. Lets try once again to get this right!

<pre>
	....
	unsigned int table;
	if(VMMDIR(vdir, virt) == 0){
		table = pmm_vmm_alloc();
		VMMDIR(vdir, virt) = table | flags;

		// mapping table
		vmm_map_single(vdir, table, table, flags);

		for(x = 0; x < 1024; ++x){
			if(
				((table >> 22 << 10) == (virt >> 22 << 10)) &&
				((table << 10 >> 10 >> 12) == (x))
				){
				// skip it
			}else{
			
				((unsigned int*)VMMDIR(vdir, virt))[x] = 0;
			}
		}

	}
	VMMTABLE(vdir, virt) = (phy & 0xFFFFF000) | flags;
	....
</pre>
Wait. What is this? A recursive call it must be, and you are right. We need to keep a direct mapping, and for those of you not quite up to date this should do the trick in example. The table can now be referenced in both physical and virtual address modes. You will also notice some mess of shift operators. I am first saying if ''table'' is going to need to be mapped into the same table as ''virt'' and the current (x) references the same entry we are currently on then skip it. This is a corner case scenario that I had to learn the hard way.. ouch. Afterwards, we set the table entrie with the macro ''VMMTABLE'' like usual or so we thought from the get go.

There does exist a case in which every ''pmm_vmm_alloc'' call could return a address that resides on a different table. This could cause a severe performance hit if it happens, but the likely hood is low and the all around saying could be that you would have had to allocate some more tables anyway so why not do it early on? [humor]

Lets try to tie this together into one complete function implementation.
<pre>
void vmm_map_single(unsigned int *vdir, unsigned int phy, unsigned int virt, unsigned int flags){
	unsigned int table;
	if(VMMDIR(vdir, virt) == 0){
		table = pmm_vmm_alloc();
		VMMDIR(vdir, virt) = table | flags;

		// mapping table
		vmm_map_single(vdir, table, table, flags);

		for(x = 0; x < 1024; ++x){
			if(
				((table >> 22 << 10) == (virt >> 22 << 10)) &&
				((table << 10 >> 10 >> 12) == (x))
				){
				// skip it
			}else{
			
				((unsigned int*)VMMDIR(vdir, virt))[x] = 0;
			}
		}

	}
	VMMTABLE(vdir, virt) = (phy & 0xFFFFF000) | flags;
}
</pre>

=== Allocation ===
Allocation is a rather simple process but does involve one pitfall. This, is inherently, that we need to know what the pass the ''findfree'' function so we can keep kernel and user processes' virtual memory separated. You will find out when we write our initialization function why this is so. Lets get started with our prototype.

<pre>
unsigned int vmm_alloc(unsigned int *vdir, unsigned int vindex4mb, unsigned int vcount4mb, unsigned int flags);
</pre>
The ''vdir'' contains a unsigned int pointer to our page directory, ''vindex4mb'' is a four megabyte index or table index, ''vcount4mb'' is the count of four mega byte blocks we wish to allocate with in starting at ''vindex4mb'', and flags are the attributes we wish to assign this allocation. Lets see if we can write a some code to acquire a free page and then allocate it.

<pre>
	...
	unsigned int virtpage = vmm_findfree(vdir, 1, vindex4mb, vcount4mb);
	unsigned int phypage = pmm_alloc();
	vmm_map_single(vdir, phypage, virtpage, flags);
	return virtpage;
	...
</pre>
You should notice we first search for a free virtual page spot with count one. Next, we get a free physical page from our physical memory management function we wrote a while back. Finally, we actually map the physical page to the virtual page, and return the virtual address contained in ''virtpage''. You might have noticed that we used a ''vindex4mb'' and ''vcount4mb''. In our next two functions you will see why.

=== Kernel And User Allocations ===
In basic terms we need two separate ranges to be used and called by different functions to provide a kernel space and a user process space. You will understand why when we arrive at implementing our initialization function. Until then lets define our prototypes.

<pre>
	unsigned int vmm_ualloc(void);
	unsigned int vmm_kalloc(void);
</pre>

And, the implementation which uses two constants that split our kernel and user space into two (two gigabyte) regions.
<pre>
	unsigned int vmm_ualloc(unsigned int *vdir, unsigned int flags){
		return vmm_alloc(vdir, 0x1FFFFFFF>>22, (0xFFFFFFFF>>22) - 1, flags);
	}
	unsigned int vmm_kalloc(unsigned int *vdir, unsigned int flags){
		return vmm_alloc(vdir, 0x0, (0x1FFFFFFF>>22) - 1, flags);
	}
</pre>
This is a point in which you may wish to change these constants, or convert them into a more complex system that could suit you better. For development they should be ok.

=== Directory Creation ===
Here is the king of all function implementations. This one ties them all together, and even attaches a little ribbon and teleports you to another dimension... Well not quite that much but it does tie everything together! This time we need to define a little more than just a function prototype.

== Data Structures ==
<pre>
struct tvmmDir{
	unsigned int	*vdir;
	struct tvmmDir  *next;
};
</pre>
This is our linked list item, and will be used to track all page directories in use by the kernel.

==== Global Data ====
<pre>
struct tvmmDir		*g_vmm_dir = 0;
</pre>
I want you to use this global variable to keep a hook to the first entrie in the link list at all times.

==== Implementation ====
And, of course our actual implementation.

<pre>
unsigned int* vmm_create(unsigned int *vdir, unsigned int flags);
</pre>

A little trickier here so pay attention. The ''vdir'' is the current virtual address space we are executing in, it could be null or zero as I will explain later. The flags argument are normally going to be the default kernel attributes you wish to specify. I left it as a argument in case you were debugging and wish to write a quick user space application to access kernel memory (including the page directory page). Lets dive a little deeper, by exploring the code to create this initial page to be used as a directory.

<pre>
	...
	unsigned int *dir = (unsigned int*)pmm_vmm_alloc();
	// add new dir to global dir link list
	nl = (struct tvmmDir*)<YOUR MALLOC IMPLEMENTATION>;
	nl->vdir = dir;
	if(g_vmm_dir == 0){
		nl->next = 0;
		g_vmm_dir = nl;
	}else{
		nl->next = g_vmm_dir;
		g_vmm_dir = nl->next;
	}
	....
</pre>
Here we allocate a page from a special function pmm_vmm_alloc that returns a physical page address from that speacil range we created earlier. If you implemented your own then it should also return a page that can be directly mapped. Next, we either start the link list or add a new item to it keeping the next pointers updated correctly. Lets see about doing direct mapping on this directory page so it can be accessed in the current address space.
<pre>
	...
	struct tvmmDir *nl;
	unsigned int *dir = (unsigned int*)pmm_vmm_alloc();
	// add new dir to global dir link list
	nl = (struct tvmmDir*)<YOUR MALLOC IMPLEMENTATION>;
	nl->vmm = dir;
	if(g_vmm_dir == 0){
		nl->next = 0;
		g_vmm_dir = nl;
	}else{
		nl->next = g_vmm_dir;
		g_vmm_dir = nl->next;
	}
	// map into current virtual address space for accessing
	vmm_map_single(vdir, (unsigned int)dir, (unsigned int)dir, flags);
	// clear entire directory
	for(x = 0; x < 1024; ++x){
		dir[x] = 0;
	}
	....
</pre>

We first map it so the processor will not generate a page fault exception, then we clear each entrie being one thousand and twenty four of them. Now, lets create a sync implementation so the changes are commited across all page directories.

===== Sync Implementation =====
Lets during the middle of implementing our intialization function go ahead and create a propagation implementation. What happens is we store a individual page directory for each process, but we shared the bottom two gigabytes with each process using tables. Yet, sometimes like in this instance we need to modify another process' page directory and tables so the solution becomes to simply keep every directory and table always mapped into the kernel's virtual space. To do this we must also make sure we keep the other directories syncronized.

<pre>
void vmm_sync(unsigned int *vdir);
</pre>
We pass our current virtual page directory from the calling vmm_create function we are currently working on.

<pre>
void vmm_sync(unsigned int *vdir){
	struct tvmmDir *cl;
	for(cl = g_vmm_dir; cl != 0; cl = cl->next){
		if(cl != vdir){
			for(x = 0; x < ((0x1FFFFFFF>>22) - 1); ++x){
				cl->vdir[x] = vdir[x];
			}
		}
	}
	return;
}
</pre>
Since the current address space is could be considered our working copy, and it should contain the same as every other directory at least when we first started execution in this address space. All we should do is update all others from this one to reflect any changes made. You will almost notice that annoying constant repersenting the kernel only using the lower two gigabytes of the entire four gigabyte address space. Remember, we do not want to copy a different processes tables onto another as that would create wierd events..can you imagine?

==== Create (Continued) ====
Here is where we left from.
<pre>
	...
	struct tvmmDir *nl;
	unsigned int *dir = (unsigned int*)pmm_vmm_alloc();
	// add new dir to global dir link list
	nl = (struct tvmmDir*)<YOUR MALLOC IMPLEMENTATION>;
	nl->vmm = dir;
	if(g_vmm_dir == 0){
		nl->next = 0;
		g_vmm_dir = nl;
	}else{
		nl->next = g_vmm_dir;
		g_vmm_dir = nl->next;
	}
	// map into current virtual address space for accessing
	vmm_map_single(vdir, (unsigned int)dir, (unsigned int)dir, flags);
	// clear entire directory
	for(x = 0; x < 1024; ++x){
		dir[x] = 0;
	}
	....
</pre>
Now, that we have implemented a syncronization function lets use it!
<pre>
unsigned int* vmm_create(unsigned int *vdir, unsigned int flags){
	struct tvmmDir *nl;
	unsigned int *dir = (unsigned int*)pmm_vmm_alloc();
	// add new dir to global dir link list
	nl = (struct tvmmDir*)<YOUR MALLOC IMPLEMENTATION>;
	nl->vmm = dir;
	if(g_vmm_dir == 0){
		nl->next = 0;
		g_vmm_dir = nl;
	}else{
		nl->next = g_vmm_dir;
		g_vmm_dir = nl->next;
	}
	// map into current virtual address space for accessing
	vmm_map_single(vdir, (unsigned int)dir, (unsigned int)dir, flags);
	// clear entire directory
	for(x = 0; x < 1024; ++x){
		dir[x] = 0;
	}
	// syncronize
	vmm_sync(vdir);
	return dir;
}
</pre>
Now, we have a complete vmm_create function but we are missing something. The ability to cope if vdir is null. Lets make some changes.
<pre>
unsigned int* vmm_create(unsigned int *vdir, unsigned int flags){
	struct tvmmDir *nl;
	unsigned int *dir = (unsigned int*)pmm_vmm_alloc();
	// add new dir to global dir link list
	nl = (struct tvmmDir*)<YOUR MALLOC IMPLEMENTATION>;
	nl->vmm = dir;
	if(g_vmm_dir == 0){
		nl->next = 0;
		g_vmm_dir = nl;
	}else{
		nl->next = g_vmm_dir;
		g_vmm_dir = nl->next;
	}
	if(vdir != 0){
		// map into current virtual address space for accessing
		vmm_map_single(vdir, (unsigned int)dir, (unsigned int)dir, flags);
		// clear entire directory
		for(x = 0; x < 1024; ++x){
			dir[x] = 0;
		}
		// syncronize
		vmm_sync(vdir);
	}else{
		for(x = 0; x < 1024; ++x){
			dir[x] = 0;
		}
		vmm_map_single(dir, (unsigned int)dir, (unsigned int)dir, flags);
		// no syncronization since this is the first ever page directory.
	}
	return dir;
}
</pre>
Now, you will notice we still add a item to the link list, but we branch and just clear and make a mapping to it's self.

== Finishing Up ==
To finsh up lets try to run down a example scenario with some code. For starters lets say the kernel just booted up, and you need to get the memory managment going. Here is what I would round about do.
<pre>
pmm_setmaxmem(AQUIRE_MAXMEM_INSTALLED);
pmm_init(FIRST_CALL_WITH_BIGGEST_FREE_MEMORY_RANGE);
for(;;){
	// LOOK THROUH ANY OTHER MEMORY RANGES
	pmm_init(....);
}
// STIP THE KERNEL'S LOADED ADDRESS OUT.
pmm_strip(KERNEL_START, KERNEL_PAGECOUNT);

cr3 = vmm_create(0);
vmm_map_range(cr3, KERNEL_START, KERNEL_START, KERNEL_PAGECOUNT, VMM_KERNEL);
vmm_map_range(cr3, 0xb8000, 0xb8000, 1, VMM_KERNEL);
vmm_map_range(cr3, GDT_AND_IDT_START, GDT_AND_IDT_START, GDT_AND_IDT_PAGECOUNT, VMM_KERNEL);
// TURN ON PAGING

// ................................
// ... create processes virtual memory ...
childcr3 = vmm_create(cr3);
// call vmm functions with childcr3... get childcr3 ready..
// syncronize all other page directories..
vmm_sync(cr3);
// jump to child thread..
// ..............................
</pre>

== Extras ==
Here are some extras I have.

<pre>
#define VMM_SUPER 0x4
#define VMM_RW	0x2
#define VMM_PRESENT 0x1
#define VMM_KERNEL (VMM_SUPER|VMM_RW|VMM_PRESENT)
</pre>
Flags for using with the flag argument for the vmm_ function calls.

== Feedback ==
You can provide feedback at <kmcg3413@gmail.com>
````

<a id="w001965"></a>

## W001965 — File:KmcguireBytemap.png

Bron-ID: W001965
Namespace: File (6)
Revisie: 3346 | Datum: 2007-03-16T02:07:59Z
Bron: https://wiki.osdev.org/File:KmcguireBytemap.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AKmcguireBytemap.png&oldid=3346

````text
A depiction of a byte map.
````

<a id="w001966"></a>

## W001966 — File:KmcguireBytemapPmmMap.png

Bron-ID: W001966
Namespace: File (6)
Revisie: 3902 | Datum: 2007-05-19T02:39:29Z
Bron: https://wiki.osdev.org/File:KmcguireBytemapPmmMap.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AKmcguireBytemapPmmMap.png&oldid=3902

````text
A depiction of the unsigned char *g_pmm_map variable in Quick And Dirty Virtual Address Space Scheme.

Copyright held by [[User:Kmcguire|Kmcguire]]
````

<a id="w001967"></a>

## W001967 — Talk:GCC Cross-Compiler

Bron-ID: W001967
Namespace: Talk (1)
Revisie: 23887 | Datum: 2019-09-02T12:38:31Z
Bron: https://wiki.osdev.org/Talk:GCC_Cross-Compiler
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AGCC_Cross-Compiler&oldid=23887

````text
== Windows Subsystem for Linux ==
Checking in for the first time in a couple of years :) Just followed the instructions for apt-get based system on WSL (Bash on Ubuntu on Windows) and it works perfectly for gcc 6.2.0 with Binutils 2.27. I'll be switching to that system over Cygwin. --[[User:Aj|Aj]] 06:23, 25 November 2016 (CST)

=== Update to WSL ===
WSL has been non-beta for a while now and I'd suggest it as the go-to method for anybody using Windows 10 now. Compatibility with builds (grub tools etc...) is now ''so'' much better than Cygwin64 and I've been using it as my main dev environment on Windows 10 since the start of the Beta. I'm happy to add the relevant notes on the main wiki page. Objections? --[[User:Aj|Aj]] 05:54, 12 January 2018 (CST)

== Repeatable Error with Cygwin 64? ==

I got a chance to have a play with my cross-compilers today (as you do...) and had a go at following the tutorial on the 64 bit version of Cygwin on Windows 8.1. Binutils version is 2.24 and GCC is 4.9.1, target is x86_64-elf. On "make install-gcc", I get the error:

<source lang="bash">Verify that you have permission to grant a GFDL license for all
new text in tm.texi, then copy it to ../../gcc-4.9.0/gcc/doc/tm.texi.</source>

I'm obviously not the first to get this error ([http://stackoverflow.com/questions/23839851/gcc-cross-compiler-cygwin64-gfdl-license-tm-texi reference]). This seems to be some sort of file version control issue. In my case, copying the file as suggested by the error message did ''not'' work. What ''did'' work was to re extract the tar file containing the original GCC source, overwriting the GCC source files (I assume this resets date stamps). After this, I changed back to my build-gcc folder, ran 'make install-gcc' and the process went swimmingly. I can therefore confirm that the tutorial works OK in Cygwin64 with the above proviso. I've not added this to the list of common errors in the article as I don't have time to test repeatability and would like others to confirm that this is a real issue first. (PS: Diffutils also required as the build process uses cmp at this point) --[[User:Aj|Aj]] 16:49, 25 July 2014 (CDT)

PS: It also seems like in Cygwin64, the dependency libintl-devel is also not installed by default and needs to be added to Cygwin for successful compilation, even with --disable-nls. --[[User:Aj|Aj]] 06:29, 28 July 2014 (CDT)

==Unwieldy Configurations Table Size==

I've just added another working config and the table of "known good" configurations is now getting a little big when viewed on a 1024x768 4:3 monitor. The situation is only going to get worse, so could I make some suggestions:

# Remove GCC versions < 4.0.0 ''and/or''
# Make the table reverse-chronological, so GCC 4.5.0 / Binutils 2.20 would be at the top left and version numbers would decrease rightwards and downwards, so no horizontal scroling for the latest version. ''and/or''
# Something else... ''and/or''
# Club together and buy me a nice widescreen monitor ;o)
Any thoughts, before I make such a big change? I favour option 2 (or 4!) so that all information is left intact --[[User:Aj|Aj]] 12:29, 11 February 2010 (UTC)

: I support this, there should be a minimal resolution that all the articles should target.. most of my monitors are <= 1024x768, so that's my biased recommendation. --[[User:Brynet-inc|Brynet-Inc]] 03:00, 13 February 2010 (UTC)

: I would go with option 1, since GCC4 is (AFAIK) fairly standard and mature now. If we go with option 1 however, I would also suggest dumping the other information regarding GCC3 from the article. Option 2 is an idea, but it won't decrease the size of the table and it's getting a bit big IMHO. --[[User:Creature|Creature]] 11:46, 13 February 2010 (UTC)

: I don't agree with option 1, there still may be people using that version of the compiler. --[[User:Brynet-inc|Brynet-Inc]] 20:42, 13 February 2010 (UTC)

: What if we were to split the table into two tables: one for GCC3 and one for GCC4? That would fix the exaggerated width and make it a little less overwhelming. --[[User:Creature|Creature]] 11:18, 14 February 2010 (UTC)

: I have copied the table section to [[User:AJ/GCC_Cross-Compiler]] so that I can play around with the layouts. I won't do anything to the main article yet. I think this is such a well-used article that I will start a topic on the forums once I have a suggested replacement. --[[User:Aj|Aj]] 09:22, 15 February 2010 (UTC)

==Building Target-libgcc==
At the end of cross-compiler creation, I always run make all-target-libgcc install-target-libgcc for long long integer support and so on. I wonder if this should be added to the Cross-Compiler tutorial, as the sensible time to do this is when you already have a configured build directory. If so, how should it be added; perhaps just as a footnote or should it actually be listed as an (optional) additional build command at the end of the GCC build? --[[User:Aj|Aj]] 10:56, 3 August 2009 (UTC)

:+1 for adding this to the tutorial. I think it would be best to keep it away from the bulk of the tutorial and leave it as a "Now you're done" sort of thing. That way people (especially those new to this kind of stuff) will be happy to leave with a cross-compiler without libgcc, and those who are interested in the additional features libgcc provides will read further. My 2c. --[[User:Pcmattman|pcmattman]] 12:02, 3 August 2009 (UTC)

: I have inserted this information before the troubleshooting section. The idea is that the libgcc targets should be built while the configured gcc directory still exists, so I hope to catch people before they close Bash and delete their temporary build-gcc directory. I have also added a link to a summary of libgcc and have deleted the (dead) link to bill gaitliff's site - he used to offer a free advice page, but now seems to provide commercial cross-compiler services under a different URL. --[[User:Aj|Aj]] 10:04, 25 August 2009 (UTC)

==Compilation of x86_64-elf Target==
I have just tried with GCC 4.3.1 and 4.3.2 and it appears that GCC no longer requires the x86_64-elf target adding manually for configuration (it now works in exactly the same way as i586-elf. I am going to make a few changes to the article to reflect this, but where does this leave the specific x86_64 Cross compiler article? Deleted? Added as a cross-reference for people building against older versions? --[[User:Aj|Aj]] 10:41, 29 September 2008 (UTC)
:I would put that information in the x86_64 crosscompiler article, rather than adding it here, since people are told to go there anyway. - [[User:Combuster|Combuster]] 10:58, 29 September 2008 (UTC)

==Cross-Reference Suggestion==
I was thinking of referencing the Porting Newlib article from Step 2 of this tutorial and/or the Articles section at the end of the page - any problems with that? --[[User:AJ|AJ]] 07:38, 1 October 2007 (CDT)
:No problem at all. - [[User:Combuster|Combuster]] 10:51, 1 October 2007 (CDT)

==Binutils requires Flex & Bison==
I'm editing the article because flex and bison are still required for at least binutils 4.2.2. [[User:CodeAnxiety|CodeAnxiety]] 01:45, 31 January 2008 (CST)

==What does it Do?==
After reading through part 1, i fail to see what is done that cannot be done by GCC before creation of this cross compiler. It seems to be a waste of time, and nothing is specified about the platform that can't be accomplished by all other versions of GCC. --[[User:Tyler|Tyler]] 22:50, 15 March 2007 (CDT)

Either your answer is in [[GCC_Cross-Compiler#Why_an_OS_developer_should_build_a_cross-compiler]], otherwise, could you please be more clear in ''what'' the problem is exactly? - [[User:Combuster|Combuster]] 04:03, 16 March 2007 (CDT)

My problem is... this tutorial does not show you how to "solve" any of these problems. It tells you how to recompile GCC to make only ELF files... something GCC can do anyway, ridding you of the problems. --[[User:Tyler|Tyler]] 08:25, 16 March 2007 (CDT)

The main difference is that the GCC you are building is a Stage 1 toolchain, not a bootstrapped toolchain. (Not ELF vs COFF like you think) A dedicated crosscompiler does not think its compiling for windows/linux and as such does not fill in the gaps for you. It does just what you ask and only what you ask and nothing more, which is something you cant say of a stock GCC. Some of these 'features' you can override with commandline parameters, but some will stubbornly persist. - [[User:Combuster|Combuster]] 04:45, 19 March 2007 (CDT)

:Actually, the ''main'' difference is that, to my knowlegde, Cygwin GCC doesn't allow you to generate ELF binaries without the help of objcopy. Several other problems - like careless inclusion of system headers, error messages being posted to the forum that cannot be reproduced by people using a different host OS, and more - are also solved by this tutorial. [[User:Solar|Solar]] 09:57, 3 April 2007 (CDT)

:Like [[User:Solar|Solar]] says, Cygwin GCC can't create ELF binaries by default. Creating a cross compiler fixes that, and more, and is pretty easily done. - [[User:Quok|Quok]] 20:30, 3 April 2007 (CDT)

== Texinfo ==

Has anyone else had problems building Binutils/GCC when Texinfo is not installed? I am not sure what it is used for, but for some reason when not installed the Make fails. -- [[User:Tyler|Tyler]] 14:02, 13 September 2007 (CDT)

: AFAIK texinfo should only be used to make the documentation.  On my setup, running 'make all-gcc' does not, by default, attempt to create the documentation and 'make install-gcc' does not attempt to install it, so texinfo shouldn't be required.  Having said that, I do have texinfo installed, so I might not have noticed a call to it.  [[User:Jnc100|John]] 14:41, 13 September 2007 (CDT)

:: I've never got as far as GCC without Texinfo, so i wouldn't know, it dies in the Binutils build, during a make all. Does Make all create documentation? -- [[User:Tyler|Tyler]] 15:25, 13 September 2007 (CDT)

::: Yes, I'm afraid it does.  There is a 'make all-binutils' command, but that also seems to try building in the bfd/doc directory.  Apparently running configure with the --without-docdir option stops it trying to install the documentation.  I don't know if it stops it trying to make it however.  [[User:Jnc100|John]] 16:35, 13 September 2007 (CDT)

::::I checked my cygwin setup, and noticed texinfo is installed without me specifically selecting it, so it either is a dependency of one of the other packages I do have, or its installed by default. Since solar's regression test of removing, reinstalling, selecting the basic install + the packages listed always results in an environment that can build the crosscompiler, I don't think its an error in the tutorial. - [[User:Combuster|Combuster]] 02:55, 14 September 2007 (CDT)

::::For those having problems building binutils without texinfo installed, run the configure step as normal. When that finishes, do this:
::::echo "MAKEINFO = :" >> Makefile
::::That command overrides the MAKEINFO variable in the Makefile, setting it to something that does not exist. This causes make to skip attempting to build (and subsequently install) the documentation. --[[User:Quok|quok]] 06:20, 26 August 2008 (UTC)

== Version Junkies... ==

Pretty impressive to see how GCC 4.5.0 is listed in the "compatibility matrix", when it has not even hit ftp.gnu.org yet... someone *is* eager to use the latest... ;-) -- [[User:Solar|Solar]] 07:52, 12 February 2010 (UTC)
* I was thinking exactly the same thing... --[[User:Creature|Creature]] 17:19, 12 February 2010 (UTC)

== GMP, MPFR and MPC ==

See [[Talk:Cross-Compiler_Successful_Builds]] and corresponding [http://forum.osdev.org/viewtopic.php?f=8&p=176033#p176033 forum thread]. These supporting libraries are general-purpose and not specific to your cross-compiling target, so it is best to install them the way your system usually installs packages, instead of compiling them from source. Which you can do properly only when you're root and remember to install to /usr/local, while no SysAdmin should have anything against installing them on a shared system you might be using.

== RE: New "Note 2" ==

Quote: "Cygwin users should use the default Windows 'terminal'. Else you may get the error message <path-to-as>/as.exe: Bad Address". Can someone explain this one to me? This tutorial works fine in the Cygwin bash shell, as well as from the Windows 'cmd' shell (if the Cygwin /bin is in your PATH). This note seems misleading and misguided. --[[User:Pcmattman|pcmattman]] 05:20, 13 December 2010 (UTC)

:I cannot picture why this should be, either. I did send the user (whose only activity on either the Wiki and the forum was this edit) a PM on the forum asking him to elaborate, and reverted his edit until he does. -- [[User:Solar|Solar]] 10:03, 13 December 2010 (UTC)

== On Mac OS X Lion ==

I was building a cross-compiler (binutils 2.22, gcc 4.6.2) for i386-elf on Mac OS X 10.7.2 with Xcode 4.1 using this tutorial (from memory at first, but I double checked to make sure that I wasn't screwing things up), when I came across a rather peculiar issue. The toolchain would successfully compile and install, but when I went to use it, it would start spewing out internal compiler errors. Below are the commands I used when building the toolchain, as well as a reduction for reproducing the errors.
<source lang="bash">export PREFIX=/usr/local/cross
export TARGET=i386-elf
export MAKEFLAGS="-j 16"
mkdir -p ~/src/build-{binutils,gcc}/
curl ftp://ftp.gnu.org/gnu/binutils/binutils-2.22.tar.bz2 | tar -x -f - -C ~/src/
curl ftp://ftp.gnu.org/gnu/gcc/gcc-4.6.2/gcc-core-4.6.2.tar.bz2 | tar -x -f - -C ~/src/
curl ftp://ftp.gnu.org/gnu/gmp/gmp-5.0.2.tar.bz2 | tar -x -f - -C ~/src/gcc-4.6.2/
curl ftp://ftp.gnu.org/gnu/mpfr/mpfr-3.1.0.tar.bz2 | tar -x -f - -C ~/src/gcc-4.6.2/
curl http://www.multiprecision.org/mpc/download/mpc-0.9.tar.gz | tar -x -f - -C ~/src/gcc-4.6.2/
cd ~/src/build-binutils/
../binutils-2.22/configure --prefix=$PREFIX --target=$TARGET --disable-nls
make all
sudo make install
export PATH=$PATH:$PREFIX/bin
cd ~/src/build-gcc/
mv ../gcc-4.6.2/gmp-5.0.2/ ../gcc-4.6.2/gmp/
mv ../gcc-4.6.2/mpfr-3.1.0/ ../gcc-4.6.2/mpfr/
mv ../gcc-4.6.2/mpc-0.9/ ../gcc-4.6.2/mpc/
../gcc-4.6.2/configure --prefix=$PREFIX --target=$TARGET --disable-nls --without-headers --with-mpfr-include=$HOME/src/gcc-4.6.2/mpfr/src/ --with-mpfr-lib=$HOME/src/build-gcc/mpfr/src/.libs/
make all-gcc
sudo make install-gcc</source>

Source file (reduction.c):
<source lang="c">void _start()
{
}</source>

Built using:
<source lang="bash">i386-elf-gcc -ffreestanding -c reduction.c</source>

Diagnostics:
<pre>reduction.c:3:1: internal compiler error: in execute_ipa_pass_list, at passes.c:1921
Please submit a full bug report,
with preprocessed source if appropriate.
See <http://gcc.gnu.org/bugs.html> for instructions.</pre>

Preprocessed source:
<source lang="c"># 1 "reduction.c"
# 1 "<built-in>"
# 1 "<command-line>"
# 1 "reduction.c"
void _start()
{
}</source>

After researching for a bit, I discovered that the issue is caused by Xcode 4.1's side-by-side installation of gcc, clang, and llvm-gcc.
<pre>$ readlink /usr/bin/gcc /usr/bin/g++
llvm-gcc-4.2
llvm-g++-4.2</pre>

By default, "gcc" will invoke llvm-gcc, which triggers some sort of bug in gcc's code (I didn't bother figuring out what specifically). To work around this, I set these variables at the beginning (along with PREFIX, TARGET, and MAKEFLAGS):
<source lang="bash">export CC=/usr/bin/gcc-4.2
export CXX=/usr/bin/g++-4.2
export CPP=/usr/bin/cpp-4.2
export LD=/usr/bin/gcc-4.2</source>

And repeated the rest of the process as-is. Doing so creates a functional toolchain.

The reason I am posting this is because I suggest that these instructions be added to the tutorial, possibly in the Troubleshooting section. I'd go ahead and add it but I'm pretty new to the wiki and don't know if there's a change approval process or what. :) --[[User:Nokurn|Nokurn]] 05:47, 29 November 2011 (UTC)

:Very good description of the problem. Thank you!
:But I am confused. Your cross-compiler should reside in /usr/local/cross/bin, named i386-elf-gcc. (As your example call shows.) Whatever /usr/bin/gcc is or points to shouldn't affect it in the least. Neither should your CC / CXX / ... exports solve the problem...?!? -- [[User:Solar|Solar]] 07:11, 29 November 2011 (UTC)
::$CC & co. are used when compiling the cross-compiler. Something about using llvm-gcc to produce the cross-compiler triggers a bug of some sort in the gcc source code, so you get a defective i386-elf-gcc. At least, that's what I've come to think. I don't know how else to explain why the host compiler would affect the resulting cross-compiler. I'd dig into the gcc source code to try to figure it out but I've heard so many horror stories about that codebase that I'm breaking into a cold sweat just by entertaining the idea... --[[User:Nokurn|Nokurn]] 07:18, 29 November 2011 (UTC)
:::Ah, I see - you set those exports ''while compiling the cross-compiler'', not while using it (as I initially understood). That's definitely worth a bug report to upstream (GCC)... Funny enough, a quick Google shows that [http://llvm.org/docs/GettingStarted.html#brokengcc the animosities are mutual]. :-D I'll extend the tutorial ASAP. -- [[User:Solar|Solar]] 14:58, 29 November 2011 (UTC)
::::I'll look into putting together a bug report for them. Thanks for adding it. Great tutorial by the way. --[[User:Nokurn|Nokurn]] 22:28, 29 November 2011 (UTC)

== Multiple Versions Simultaneously ==
You can have multiple versions installed simultaneously by setting the --program-prefix and --program-suffix options during configuration. In my case I do --program-suffix=-$GCC_OR_BINUTILS_VERSION --program-prefix=$TARGET- where $GCC_OR_BINUTILS_VERSION is the version of gcc or binutils you are installing.      - ChosenOreo

== ./gthr-default.h:35:10: fatal error: pthread.h: No such file or directory AND TARGET=arm-linux-gnueabihf ==

I'm building the cross-compiler to start with for the bare target as this article outlines. I ran into a problem with the "make all-target-libgcc" step with the title error: ./gthr-default.h:35:10: fatal error: pthread.h: No such file or directory, occurring at some signification point in completion. I believe I've followed the steps verbatim. and; after research, even attempted to reconfigure with --disable-threads option. There's very little additional recommendation on this error that I've found beyond, using one of the many cross-compilers somebody else has generated. I think it would be great if this talk section could have comments And after a solution is found, a section in the main page added for the corrective actions. Thanks for any feedback. If an answer appears, I'd have no problem adding a minor change to the article assuming that's permitted.
````

<a id="w001972"></a>

## W001972 — File:ISBN 0201479508.jpg

Bron-ID: W001972
Namespace: File (6)
Revisie: 13539 | Datum: 2012-06-13T02:44:20Z
Bron: https://wiki.osdev.org/File:ISBN_0201479508.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0201479508.jpg&oldid=13539

````text
[[Category:Book Covers]]
````

<a id="w001974"></a>

## W001974 — Talk:VMware

Bron-ID: W001974
Namespace: Talk (1)
Revisie: 14602 | Datum: 2013-04-30T11:22:02Z
Bron: https://wiki.osdev.org/Talk:VMware
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AVMware&oldid=14602

````text
== Opinionated ==
As per the quote:

 I found VMware emulates very accurate real hardware behaviors, at good speed (especially with Intel-VT or AMD-V)

This has more promotional value than factual value. Define "good speed" - is it worse or better than QEmu, worse or better than VirtualPC, worse or better than VirtualBox? Similarly define "very accurate emulation" - VMWare used to have rather poor graphics abilities, enough to put it below Bochs and Qemu on that matter (which have a nice collection of issues of themselves), and thus way beyond VirtualPC which so far proved to be the most accurate in that area. Is all that still the case? I think some facts should be added, or alternatively, these phrases be removed. - [[User:Combuster|Combuster]] 16:02, 18 February 2010 (UTC)

:As far as facts go, I ran my VGA test suite on VMWare and compared the results:
:#VMWare doesn't support all resolutions
:#VMWare has very poor accuracy, only enough to support standard VGA modes. In quality it barely breaks even with Bochs, and is way behind VirtualPC
:#VMWare emulates the VGA extremely slow, far worse than my default Bochs configuration. Extrapolated, Bochs would have to be set to emulate a ~900KHz processor in order to match VMWare on my Athlon64 machine (and yes, the units are correct!).
:Based on the above and the lack of replies, I will now remove the propaganda. - [[User:Combuster|Combuster]] 23:31, 22 February 2010 (UTC)

== VMWare on 64-bit windows ==
I removed this line:
:'''Note:''' On 64-bit Windows systems using AMD processors, a patch is required for virtual machines to function correctly.  The patch can be downloaded from [http://www.microsoft.com/downloads/details.aspx?familyid=CEA2832E-7CFE-4A6A-A8E2-9F666AEC5BC0&displaylang=en#QuickInfoContainer Microsoft]
I would like it to be put back in the article but I would like some clarifications first. First it should mention AMD-V. Secondly, will VMWare function incorrectly? or just not use hardware-assisted virtualization? If incorrectly I'm curious to how so? -[[User:Jhawthorn|Jhawthorn]] 03:26, 17 March 2007 (CDT)

By "function incorrectly," I meant the program will crash when a virtual machine is started (oddly enough, this only happens about half the time).  VMWare isn't using machine-assisted virtualization without this patch, but my understanding is that it relies on it for some of its functionality on the AMD64 platform.  After installing the patch, I no longer had stability issues, and observed an enormous performance boost. --[[User:speal|speal]] 03:52, 20 March 2007 (CDT)

I haven't had a chance to write up a detailed description of the problem, but I stumbled onto an article at [http://www.pagetable.com/?p=25 pagetable.com].  It seems to describe the exact issue I had, and explains why I was able to run several other hobby OSes under vmware without any performance or stability issues. --[[User:speal|speal]]
````

<a id="w001975"></a>

## W001975 — Talk:APIC

Bron-ID: W001975
Namespace: Talk (1)
Revisie: 5683 | Datum: 2008-06-19T12:16:20Z
Bron: https://wiki.osdev.org/Talk:APIC
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AAPIC&oldid=5683

````text
The original quick conversion was a bit of a mess. This is my 1st whack at cleaning it up a bit. I think it looks much better now. ;) I can't find JHawthorn's instructions on how to fix the Forum links in the See Also section -- sorry.
[[User:Bewing|Bewing]] 01:56, 19 March 2007 (CDT)

FYI: [[Post:83433]] ;) - [[User:Combuster|Combuster]] 04:52, 19 March 2007 (CDT)

Thanks for that link -- OK, I fixed those See Also links. [[User:Bewing|Bewing]] 21:49, 5 April 2007 (CDT)

== APIC in uniprocessor x86 machines... ==

The article seems to focus on APIC being multiprocessor machines, but in fact many older AMD/Intel machines have an option in BIOS that allows PIC/APIC selection. - [[User:Brynet-Inc|Brynet-Inc]] (Please sign your posts)

The idea is, you can simply forget about the APIC when dealing with uniprocessor machines, even though some of its features are very useful to the developer. Most of the useful contents is completely independent of a multiprocessor machine anyway. - [[User:Combuster|Combuster]] 15:32, 8 July 2007 (CDT)

==Sanity APIC Timer Check==
So, I attempted to condense Brendan's timer using CPU speed detection code in English. Could anyone check it over for mistakes? 

*Init the PIT at 4hz and setup the APIC timer.
*Wait for the PIT to start.
*Set the APIC timer count to 0xFFFFFFFF, and start.
*Wait for next PIT IRQ.
*Stop the APIC and PIT timers.
*((0xFFFFFFFF - APIC's current count) * 32) / frequency / divider =  New APIC count
--[[User:Alboin|Alboin]] 18:18, 9 June 2008 (CDT)

Well, as far as mistakes goes ... you can't possibly mean 4hz. 400Hz? 4KHz?<br>
[[User:Bewing|Bewing]] 15:42, 11 June 2008 (CDT)
:(The code I'm referring to is on the main APIC in the links section.) According to Brendan's comment: ";Configure the RTC or PIT timer for an IRQ every 250 mS (4 Hz)"--[[User:Alboin|Alboin]] 15:53, 11 June 2008 (CDT)
::However, "You might also want to play with how long the duration of the fixed time value is - 250 mS is relatively long considering the speed of even the slowest CPU's bus." -- Would changing that affect the rest of the process? (eg. the 32, or the 128/4)--[[User:Alboin|Alboin]] 15:55, 11 June 2008 (CDT)
:: -- Well, I don't know what Brendan was thinking ... but the absolute lowest possible PIT frequency is 18.2Hz. You can't get to 4. So I'd suggest using a different number. [[User:Bewing|Bewing]] 22:59, 12 June 2008 (CDT)
:The only real remark I have is that Bus frequency = APIC frequency ''!= CPU frequency''. Other than that, it's a good impression. - [[User:Combuster|Combuster]] 16:15, 11 June 2008 (CDT)
By the by, where do you think a short bit on determining the bus frequency should go? I'm not feeling love with it in here...(Cat:x86?)--[[User:Alboin|Alboin]] 16:47, 12 June 2008 (CDT)
:[[Detecting CPU Speed]], maybe? - [[User:Combuster|Combuster]] 12:16, 19 June 2008 (UTC)
````

<a id="w001977"></a>

## W001977 — Talk:PS/2 Keyboard

Bron-ID: W001977
Namespace: Talk (1)
Revisie: 17517 | Datum: 2015-02-02T22:00:23Z
Bron: https://wiki.osdev.org/Talk:PS/2_Keyboard
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APS%2F2_Keyboard&oldid=17517

````text
==Todo==
Would benefit from code to initialize the keyboard  [[User:Makerimages|Makerimages]]

Add info on: scan codes, changing led lights, handle shift/caps etc, etc... - [[User:Jhawthorn|Jhawthorn]]
Maybe change name to "Keyboard programming" or similar, since it's not only about getting a key.
:How about changing the name to just 'Keyboards'? I've noticed that this is the only page on the wiki dealing with just keyboards, so it would make some sense. --Alboin
::Would this be okay? --Alboin
:::I'd call it [[PS2 Keyboard]] - [[User:Combuster|Combuster]] 08:15, 14 July 2007 (CDT)
::::Moved and done. --Alboin
:::::Heh - I broke stuff! This should be a (platform neutral) "PS/2 Keyboards" page, and shouldn't have any information about "PS/2 Controller" or other types of keyboard (e.g. we should have a "USB Keyboard" or "USB HID" page). We probably also need a generic "User Input" page too. --[[User:Brendan|Brendan]] 23:05, 24 March 2012 (CDT)

==Broken forum link==
The link to the thread about "loading Arabic fonts" takes me to a thread titled "what is sequence to be followed for developing ones own OS?" instead. --[[User:Messiahandrw|Messiahandrw]] 15:02, 15 August 2009 (UTC)
:Looks like a MT post number. I'd have to find the conversion table again to find the new number - [[User:Combuster|Combuster]] 10:02, 19 August 2009 (UTC)
::PS. the file has gone amiss. It will be september when I get back to the machine where there should be a copy... http://forum.osdev.org/viewtopic.php?p=83433#p83433

==Escaped Scancodes==

In the example, shouldn't:

 if (escaped) new_scan_code += 256;

be more something like this ?

 if (escaped)
 {
  new_scan_code += 256;
  escaped = 0;
 }
- [[User:Hitchhikr|Hitchhikr]] 04:01, 12 October 2010 (UTC)
:It is indeed better, fixed - [[User:Combuster|Combuster]] 09:20, 15 October 2010 (UTC)
````

<a id="w001978"></a>

## W001978 — Licensing Issues

Bron-ID: W001978
Namespace: Main (0)
Revisie: 3404 | Datum: 2007-03-20T02:20:17Z
Bron: https://wiki.osdev.org/Licensing_Issues
Revisie-URL: https://wiki.osdev.org/index.php?title=Licensing_Issues&oldid=3404
Doorverwijzing: Licensing

````text
#REDIRECT [[Licensing]]
````

<a id="w001979"></a>

## W001979 — Talk:Journey To The Protected Land

Bron-ID: W001979
Namespace: Talk (1)
Revisie: 21645 | Datum: 2017-10-22T21:42:34Z
Bron: https://wiki.osdev.org/Talk:Journey_To_The_Protected_Land
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AJourney_To_The_Protected_Land&oldid=21645

````text
Is this information elsewhere in the wiki? I think I've seen it before... - [[User:Alboin|Alboin]] ''(please sign your messages)''

Not in a knock-down-the-dungeon-door style like this, but some parts have been described more thoroughly in [[Protected Mode]] and [[GDT]]. The wiki doesn't have an protected mode tutorial however, which to some extent is the piece that is filled by this article. It can be a bit elaborated though... - [[User:Combuster|Combuster]] 14:42, 21 March 2007 (CDT)

I vote to merge this page with [[Protected Mode]]. That would...

A. Make it easier to access this tutorial (This page is orphaned)

B. Reduce the number of clicks it takes to get here (yes, some users do have dial-up internet)

C. This tutorial would no longer be on an orphaned page

D. Help organize the wiki

...any problems with this? - [[User:Cr2|cr2]]

:No problems IMO. -- [[User:Solar|Solar]] 06:44, 19 September 2008 (UTC)
:I agree. --[[User:Davidaylaian|Davidaylaian]] 16:42, 22 October 2017 (CDT)
````

<a id="w001980"></a>

## W001980 — Talk:How kernel, compiler, and C library work together

Bron-ID: W001980
Namespace: Talk (1)
Revisie: 4194 | Datum: 2007-07-05T20:31:05Z
Bron: https://wiki.osdev.org/Talk:How_kernel%2C_compiler%2C_and_C_library_work_together
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AHow_kernel%2C_compiler%2C_and_C_library_work_together&oldid=4194

````text
== Non-matching symbols ==
The page references to both ___main and __main as being the same things. Probably, one of them has a _ too much or too few. My guess would be that it needs two, but could someone confirm that? - [[User:Combuster|Combuster]] 02:51, 3 July 2007 (CDT)

Actually in my kernel both _alloca and _main only have 1 underscore. But after a little research on the internet it seems that the number of underscores varies between platforms and file types. For example a.out has 2 and elf has 1. But then again the person that wrote this could have been thinking about how some C compilers prefix underscores to their functions. I don't know. I think that we should just pick one of them and make it consistent. --[[User:Frank|Frank]] 08:54, 3 July 2007 (CDT)

The text refers to PE binaries. So its either MinGW or MSVC that created the code. Still I doubt that either generates ''3'' underscores, although you never know with Microsoft. If its indeed compiler-dependent we should indeed pick a default (Not unlikely, as my ELF toolchain doesn't prefix underscores to main while yours does). Could someone post the MSVC variant for reference? - [[User:Combuster|Combuster]] 06:12, 5 July 2007 (CDT)

Well I was talking about the fact the linker throws an error about not having _main even though I have main declared. My ELF toolchain doesn't prefix underscores on my functions. You just have to include a stub _main for it to link properly. I have been playing around with MS visual studio for a while now and it doesn't seem to want to call _main or __main before it calls main, but it does prefix all symbols with an underscore when in c mode. What do you think? --[[User:Frank|Frank]] 15:31, 5 July 2007 (CDT)



== Unformatted Text ==
I can't figure out why the text in a few sections around 'Static Linking' are unformatted.....Any ideas? - [[User:Alboin|Alboin]] ''(please sign your posts)''

They are prefixed with whitespace, causing them to be interpreted as a code block. - [[User:Combuster|Combuster]] 02:47, 22 March 2007 (CDT)
````

<a id="w001981"></a>

## W001981 — Nwcc

Bron-ID: W001981
Namespace: Main (0)
Revisie: 28073 | Datum: 2023-07-09T20:43:58Z
Bron: https://wiki.osdev.org/Nwcc
Revisie-URL: https://wiki.osdev.org/index.php?title=Nwcc&oldid=28073

````text
==Nils Weller's C Compiler==
nwcc is a small C compiler written by Nils Weller in his spare time. It can output gas, yasm, and nasm assembly output, and supports several architectures. Currently, it is not yet complete, and is missing several features including some that may be needed by osdev. (ie. bitfields) However, it does work and can compile itself. Moreover, it assembles to straight assembly; possibly giving the osdever more control over how his code is created. Note, however, that nwcc currently has not optimizations and therefore will not produce code as esteemed as [[GCC]].

==Using With Osdev==
Using nwcc for osdev is surprisingly easy. First, you have to compile your C sources like so:

<code>nwcc -arch=x86 kernel.c -nostdinc</code>

This instructs nwcc to cross-compile kernel.c to the x86 architecture and not to include any standard includes. (Although, the source seems identical without -nostdinc, but just to be safe.) Now, an important point here. When you instruct nwcc to cross-compile something, it does not invoke an assembler, so you are free to assemble yourself with whatever assembler nwcc is currently set to. (This can be adjusted with the -asm=x flag.) Being as such, this command will output kernel.asm; just waiting to be assembled. 

<code>yasm -f elf kernel.asm -o kernel.o</code>

''Note: On my system. (An x86-64 Linux box.) nwcc '''only''' supports yasm and gas. So you may want to adjust some stuff here if you are using nasm.''

Yasm takes nwcc's output and assembles it to an object file. Not much to be said here.

<code>ld -m elf_i386 -T kernel.ld k_entry.o kernel.o</code>

''Note: k_entry.o is assumed to be a kernel entry from your boot loader. Also, omit '-m elf_i386' if not using a 64bit [[GCC]]''

This then assembles and links your kernel according to your [[LD]] linker script and entry code. 

==That's it?==

Yeah, that's it. nwcc is looking to be a promising C compiler, without all of the bloat. **cough**[[GCC]]**cough** If you can get past some of its more unique features, you should be able to use it to make a small, clean os.

==See Also==
===External Links===
*http://nwcc.sourceforge.net
*http://nwbintools.sourceforge.net

[[Category:C]]
[[Category:Compilers]]
````

<a id="w001982"></a>

## W001982 — File:Kmcguirebitmapnumbered.png

Bron-ID: W001982
Namespace: File (6)
Revisie: 3906 | Datum: 2007-05-19T02:41:17Z
Bron: https://wiki.osdev.org/File:Kmcguirebitmapnumbered.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AKmcguirebitmapnumbered.png&oldid=3906

````text
A depiction of a byte map, and the association with each page.

Copyright held by [[User:Kmcguire|Kmcguire]]
````

<a id="w001983"></a>

## W001983 — File:Kmcguirebitmapstrip.png

Bron-ID: W001983
Namespace: File (6)
Revisie: 3905 | Datum: 2007-05-19T02:41:04Z
Bron: https://wiki.osdev.org/File:Kmcguirebitmapstrip.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AKmcguirebitmapstrip.png&oldid=3905

````text
A depiction of some byte map entries having different values for a purpose.

Copyright held by [[User:Kmcguire|Kmcguire]]
````

<a id="w001984"></a>

## W001984 — File:Kmcguirephyandvirt.png

Bron-ID: W001984
Namespace: File (6)
Revisie: 3904 | Datum: 2007-05-19T02:40:56Z
Bron: https://wiki.osdev.org/File:Kmcguirephyandvirt.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AKmcguirephyandvirt.png&oldid=3904

````text
The physical and virtual memory layout.

Copyright held by [[User:Kmcguire|Kmcguire]]
````

<a id="w001985"></a>

## W001985 — File:KmcguireFindFreeVirt.png

Bron-ID: W001985
Namespace: File (6)
Revisie: 3439 | Datum: 2007-03-23T09:28:47Z
Bron: https://wiki.osdev.org/File:KmcguireFindFreeVirt.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AKmcguireFindFreeVirt.png&oldid=3439

````text
A silly depiction of locating a free virtual (not physical) page.
````

<a id="w001987"></a>

## W001987 — Talk:Memory Management2

Bron-ID: W001987
Namespace: Talk (1)
Revisie: 4616 | Datum: 2007-09-16T22:10:38Z
Bron: https://wiki.osdev.org/Talk:Memory_Management2
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMemory_Management2&oldid=4616

````text
I feel this page has no useful information. If someone has a more favorable opinion of it please speak up so I don't delete it. -[[User:Jhawthorn|Jhawthorn]] 01:20, 25 March 2007 (CDT)

Some background info on the MMU could be retained, the rest is just distracting petty talk. - [[User:Combuster|Combuster]] 13:53, 25 March 2007 (CDT)
:Should a page for the [[MMU]] be created, or would a section in [[Paging]] be enough? --Alboin
::[[MMU]] is a wanted page. Some general information on the MMU should go there as it deals with more than just paging (read: x86 segmentation), while you can keep x86-specific things in [[Paging]], pretty much like [[segmentation]] does it. Just remember, not all MMUs work like the x86's - [[User:Combuster|Combuster]] 16:04, 11 July 2007 (CDT)
:::Okay. I wrote a short article on the MMU. I tried to make it as generic as possible, so I only include the two things that pretty much all MMU's I've read about do. That is, Memory Protection and Memory Translation. Is there any reason not to delete this article now? --Alboin
::::I've read over all the relevant articles. I see nothing here that isn't in another page. [[User:Yayyak|Yayyak]] 19:49, 15 September 2007 (CDT)
:::::There is some basic info on the workings on paging (be it incorrect) for the x86 and sparc, which are not available elsewhere. The [[paging]] article is listed as a work in progress, and sparc-related pages are pretty much absent. In favour of retaining the knowledge I suggest dropping notes on the relevant (talk)pages before deleting the article. - [[User:Combuster|Combuster]] 17:10, 16 September 2007 (CDT)
````

<a id="w001990"></a>

## W001990 — User:Kmcguire/Quick And Dirty Device Management

Bron-ID: W001990
Namespace: User (2)
Revisie: 3897 | Datum: 2007-05-19T02:37:01Z
Bron: https://wiki.osdev.org/User:Kmcguire/Quick_And_Dirty_Device_Management
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKmcguire%2FQuick_And_Dirty_Device_Management&oldid=3897

````text
=The Quick And Dirt Device Management Scheme=
==Target Audience==
==Overview==
I will throw magical energy balls, containing all the information you need to know, through your video screen and into your head. A word of warning, however. If your head is not turned correctly the energy balls may cause head injuries, head aches, and slight brain damage. Please take caution, and know that we are not responsible for any of the aforementioned symptoms. Also remember that none of the previous sentences should be taken seriously, and that your belief in magic energy balls is completely up to you. Thank you for your time. 
==Terms==
Here are some terms for clarification in this article that I will be using.
* Connection - ''A connection is the physical or conceptual identification, for which a device or componet of the computer system is connected to another componet.''
* Major Identifier - ''A class of interfaces that contain similar minor interfaces.''
* Minor Identifier - ''A specifier for a interface that also resides in a class represented by the Major Identifier.''
* Device - ''A single function board that exports interfaces that are related by their major identifier.''
* Motherboard Helper - ''Similar to a driver except the primary purpose is to aid the kernel device subsystem in loading the devices present on the motherboard.''
* Multifunction Board - ''A multifunction board is one in which contains the potential of having multiple distinct devices. Hence it is referenced as a board instead of a device.''
* Driver Instance - ''A driver which has employed to control a physical device residing on a bus. There may be multiple instances of the same driver.''
* Module - ''A binary file containing executable code and required data which can contain one or more driver headers.''
* Device Subsystem - ''A kernel subsystem responsible for loading the motherboard helper and providing the framework for device management.''

==What To Load?==
The kernel has no idea what to load instead the motherboard helper, a external subsystem of the kernel device subsystem, as a run-time or compile-time module will make assumptions and decisions on what to load for the system. This motherboard helper is of a generic or specialized type. The specialized motherboard helpers are used in circumstances where a specialized computer system may be used such as a proprietary board using a MIPS processor, or even in trival situations the future may hold where certain motherboard manufactures support specialized features that are provided by the motherboard.

Once the motherboard helper initializes and in a general case it will initialize the common devices such as the 8253 Programmable Interrupt Timer, 8259 Programmable Interrupt Controller, and even make decisions to load more advanced chips like the APIC which replaces the 8259 and 8253 chips.
==What Do We Need?==
What we need to know to load a driver is only relevant to a bus driver or motherboard helper and the associated drivers for the motherboard helper or bus driver. What I mean is a driver or the PCI bus driver for a PCI bus needs to know nothing about what a driver for a ISA bus needs to know from drivers for devices on a ISA bus. Instead the PCI bus driver only needs to understand what it wants from PCI device drivers and nothing else. In special circumstances a driver or motherboard helper may require knowledge about what a PCI bus driver and a ISA bus driver need from their drivers, however this is only a what-if condition clause and should be avoided if possible. Just so you can make a mental note this information is references as ''connectionData'' in the remaining of this article, but before we actually get to explaining this ''connectionData'' we need to talk about a little before it.
==Driver Major Parts==
The major parts of a driver are: one or more interfaces, driver header, bus initialization handler, and the actual driving code for the device. The driver header is what ties the other major parts together. The bus initialization handler will be explained a little later, but right now we will take a exploration of interfaces which are the first major part of a driver.
A interface is a group of functions that are manifested by a standard which is referenced using two identifiers. The major and minor identifiers form a pair that references a standard, or a manifestation of the standard known here as a interface. This standard however has to only be published in the kernel headers, but it could be published by other means which are up to you're creative imagination. This standard is made in the following form inside the kernel header(s):
<pre>
#define DEV_PICTURE			0x1
	#define DEV_PICTURE_PICTURE		0x1
	#define DEV_PICTURE_VGA			0x2
#define DEV_IO				0x2
	#define DEV_IO_CONTROL			0x1
#define DEV_CLOCK			0x4
	#define DEV_CLOCK_CLOCK			0x1
	#define DEV_CLOCK_TIMER			0x2
#define DEV_PIC				0x5
	#define DEV_PIC_PIC			0x1
</pre>
Just for a example and not for usage in a actual production environment is the above interface major and minor values. This implementation is completely user controlled. Meaning someone out in No Man's Land could download you're kernel and decide he wants to include a new interface for a driver or motherboard helper to export. He could just select a free major and minor identifier which would be completely okay. The problem would arises if he later downloads a new version of you're kernel which ended up through official channels picking the major and minor identifiers he used. This would cause his kernel to malfunction because in the main stream kernel version the major and minor used the same major and minor he decided to use in a older main stream kernel version. This problem luckily would only affect those that decided to venture out alone in the world of standardized interfaces for drivers. The main point is that as long there is a official mediator of the major and minor identifiers there should never be any wide scale problems or at least not introduced by the kernel development team. In the worst case scenario the drivers would have to be simply recompiled once the major and minor values were changed and hopefully introduced into the main stream through the official communication channels.
===The Point Of Interfaces===
Interfaces allow us to realize our goal of allowing the devices to become managed by dividing them into major and minor categories. For example a VGA device needs to provide to same functionality as a SVGA device, as well as any other video output device needs to also provide the same. If not then a developer would be forced to write different code for each of these devices. So we provide the solution of exporting a list of functions that always: stay in the same order, have the same return values, and accept the same arguments. This creates a standard is which should be intended to remain for as long as possible. This standard is manifested in the interface in other words. Now that a interface is defined the developers can feel comfortable that their code will work properly on a VGA, SVGA, or any other video output device for many years.
At the same time the SVGA device may have extra functionality that is not present in the VGA device. To include this extra functionality we build another interface and manifest this standard onto it. This also allows developers to prove if the device supports this extra functionality only by looking for the interface instead of checking for each single part of extra functionality. All interface functions should try to provide a error mechanism, if needed, for proper handling by the calling software if it is unsupported. This could be a case in some circumstances where a functionality was not present, but is not critical to the software which would allow it to gracefully continue. A fault tolerant method is to have documentation for the interfaces that specify their behavior which allows developers to plan for the what-ifs. 
In some cases one type of device may share a common functionality with another different type of device. To make development easier it could be more rational to split the common functionality into it's own interface that is shared among the drivers.
===The Interface Look And Feel===
Lets use a example driver for a generic VGA driver.
<pre>
struct tBitmapInfo{
	unsigned int 	address;
	unsigned short 	width;
	unsigned short	height;
	unsigned int	format;
};
typedef unsigned int POS;
int BltBlt(struct tBitmapInfo *info, POS fromX, POS fromY, POS fromWidth, POS fromHeight, POS toX, POS toY, POS toWidth, POS toHeight);
</pre>
Here we depict a function that copies video data from somewhere in memory to the actual video card allowing a specification of the source format, source offset, source dimensions, and the destination offset. This would be a generic function that should be supported by all video devices.
<pre>
int Clear(POS x, POS y, POS width, POS height);
</pre>
Might we want to clear the screen we should have a function to do this which should be supported by all video devices. I want to let you know I am not trying to tell you how to write a VGA driver but rather using a simple one as a example. So with that said these two functions should be basic enough to actually create a interface.
<pre>
struct tOurPictureInterface{
	int (*BltBlt)(struct tBitmapInfo *info, POS fromX, POS fromY, POS fromWidth, POS fromHeight, POS toX, POS toY, POS toWidth, POS toHeight);
	int (*Clear)(POS x, POS y, POS width, POS height);	
};
</pre>
Now we have created a standard that is manifested in a interface form. Once matured it can be published so that developers can reply on its functionality being present or if not the supporting code for the interface can emulate it if possible, or return a error value. We can also create other interfaces to represent functionality for a VGA, SVGA, or both and others.
===Driver Header===
The major parts of a driver are: one or more interfaces, driver header, bus initialization handler, and the actual driving code for the device. The interfaces were discussed a little above and will be futher discussed a little later along with the bus initialization handler. For the time being we will explore a little of the glue that I like to call the driver header.

The driver header glues together the major parts of a driver. We have just discussed interfaces but before we discuss the initialization handler and actual driver code we will cover the driver header in detail. A driver header needs to include some information such as: what type of connection in the system do we have, the interfaces we export, our connection dependent data, and our initialization routine for the connection. The type of connection could be explained by showing a list of connection types defined in our device subsystem's source.
<pre>
#define DEV_CONNECTION_DEV		0x0
#define DEV_CONNECTION_MOTHERBOARD	0x1
#define DEV_CONNECTION_PCI		0x2
</pre>
These are the connection types with ''DEV_CONNECTION_DEV'' being identification for the motherboard helper to the kernel device subsystem, ''DEV_CONNECTION_MOTHERBOARD'' being support for common board components such as the 8253 Programmable Interrupt Timer; 8259 Programmable Interrupt Controller; PCI, and ''DEV_CONNECTION_PCI'' identifying the connection between PCI drivers and the PCI bus driver. Of course more can be listed here, but we only need enough to help you gain a understanding of what is going on.

The connection dependent data and our initialization routine are actually combined in this device management scheme. Let me show you a three data formats with each one being for a connection in the order of: ''DEV_CONNECTION_DEV'', ''DEV_CONNECTION_MOTHERBOARD'', ''DEV_CONNECTION_PCI''.
<pre>
#define DEV_CONNECTION_DEV_VERSION 0x00000000
struct tdevBusData{
	u16		vendor;
	u16		board;
	i32 		(*Init)(void);
}; typedef struct tdevBusData DEV_CONNECTION_DATA; typedef struct tdevBusData* PDEV_CONNECTION_DATA;
#define DEV_CONNECTION_MOTHERBOARD_VERSION 0x00000000
struct tdevBusDataMobo{
	u32		type;
	i32 		(*Init)(void);
}; typedef struct tdevBusDataMobo DEV_CONNECTION_MOTHERBOARD_DATA; typedef struct tdevBusDataMobo* PDEV_CONNECTION_MOTHERBOARD_DATA;
#define DEV_CONNECTION_PCI_VERSION	0x00000000
struct tdevBusDataPci{
	u32 vendor;
	u32 device;
	i32 (*Init)(u32 bus, u32 slot);
}; typedef struct tdevBusDataPci DEV_CONNECTION_PCI_DATA; typedef struct tdevBusDataPci* PDEV_CONNECTION_PCI_DATA;
</pre>
Here we have defined three structures for converying minimal scope information about each device to the appropriate bus, or the motherboard helper. You should note that these may be added to or changed just like interfaces. There only serve to present a standard or a language that can be understood between the bus or motherboard helper and the drivers. You may also notice that the ''DEV_CONNECTION_PCI_DATA'' connection data provides space for a vendor and device identifier. It would be wasteful for us to include this information into every driver header becoming the point to making the a customized data area for drivers and their buses or their motherboard helper.

Let us take a peek at the actual driver header structure.
<pre>
struct tdevDriver{
	u8 		*szName;
	u16		connection;
	u16		connectionVersion;
	void 		*connectionData;			///
	void		*priv;
	u16		ifaceCount;
	DEVINTERFACE	iface[];
}; typedef struct tdevDriver DRIVER; typedef struct tdevDriver* PDRIVER; 
typedef struct tdevDriver MOTHERBOARDHELPER; typedef struct tdevDriver* PMOTHERBOARDHELPER;
</pre>
We can first see that we have a field ''szName'' which is a user defined null terminated string. The ''connection'' field provides a value from our ''DEV_CONNECTION_*'' constants: ''DEV_CONNECTION_DEV,DEV_CONNECTION_MOTHERBOARD,DEV_CONNECTION_PCI''. The ''connectionVersion'' field provides a fail safe mechanism that helps developers or users catch drivers compiled with older ''connectionData'' structures used and a listing which we seen but did not cover is: ''DEV_CONNECTION_DEV_VERSION,DEV_CONNECTION_MOTHERBOARD_VERSION,DEV_CONNECTION_PCI_VERSION''. The ''connectionData'' field provides a pointer to the structure that contains the language that can be understood between the bus driver or motherboard helper and drivers that fit under these appropriately and some of the structures which we gave as a examples were: ''DEV_CONNECTION_DATA,DEV_CONNECTION_MOTHERBOARD_DATA,DEV_CONNECTION_PCI_DATA''. The ''priv'' field is a speacil kernel writable only field and any value placed here should become over writen once a instance of the driver is created. We will cover instances later but for now the ''ifaceCount'' field contains the number of ''DEVINTERFACE'' structures that follow. The ''DEVINTERFACE'' structures are access by ''(iface[index])''. Lets see if a example of this information put into the real world could help you grasp what is going on.
Here is a motherboard helper structure, which is exactly the same as the driver structure.
<pre>
static DEV_CONNECTION_DATA connectionData = {
	.vendor = 0,
	.board = 0,
	.Init = &mobo_generic_init	
};
MOTHERBOARDHELPER mobo_generic = {
	.szName = "MOTHERBOARD GENERIC",
	.connection = DEV_CONNECTION_DEV,
	.connectionVersion = DEV_CONNECTION_DEV_VERSION,
	.connectionData = &connectionData,
	.priv = 0,
	.ifaceCount = 0,
	{
		// no interfaces defined or needed by the generic motherboard.
		// -- however this is up to you and you're own kernel ideas!
	}
};
</pre>
You can see that the ''connection'' field is specified as ''DEV_CONNECTION_DEV'' which repersents the kernel device subsystem. The ''connectionVersion'' field also represents the version value used from the kernel sources when the driver.. I mean motherboard helper was compiled. The ''connectionData'' is a important field by showing the relationship between the ''DEV_CONNECTON_DATA'' structure and the driver header.

Here is a PCI driver structure.
<pre>
static DEV_CONNECTION_MOTHERBOARD_DATA connectionData = {
	.type = DEV_MOTHERBOARD_PCI,
	.Init = &pci_generic_init
};
DRIVER pci_generic = {
	.szName = "PCI GENERIC",
	.connection = DEV_CONNECTION_MOTHERBOARD,
	.connectionVersion = DEV_CONNECTION_MOTHERBOARD_VERSION,
	.connectionData = &connectionData,
	.priv = 0,
	.ifaceCount = 0,
};
</pre>
The ''connection'' field in this driver specifies the ''DEV_CONNECTION_MOTHERBOARD'' value which says this driver has a ''connectionData'' field with relevant information to a driver that implements the ''DEV_CONNECTION_MOTHERBOARD'' mechanisms. In normal circumstances the motherboard helper will implement the ''DEV_CONNECTION_MOTHERBOARD'' mechanisms or language. Before I get too far I want to show a example list of constants that a generic motherboard helper would understand. These constants are specified in the ''DEV_CONNECTION_MOTHERBOARD_DATA.type'' field.
<pre>
#define DEV_MOTHERBOARD_VGA 	0x1
#define DEV_MOTHERBOARD_PCI 	0x2
#define DEV_MOTHERBOARD_CLOCK	0x3
#define DEV_MOTHERBOARD_PIC	0x4
#define DEV_MOTHERBOARD_APIC	0x5
</pre>
Just to note that this is just a design, and it is not absolutely required that you follow this method but at least understand what is going on so you have a idea of how you might like to change or reimplement this in your own way. We can see that the PCI driver is using the constant ''DEV_MOTHEROAD_PCI'' value for it's ''type'' field in the ''DEV_CONNECTION_MOTHEROARD_DATA'' structure. This is primary used in my example here to allow the motherboard helper to search for a PCI driver by looking through all the drivers that have the ''connection'' field value set to ''DEV_CONNECTION_MOTHERBOARD'' and have the ''DEV_CONNECTION_MOTHERBOARD_DATA.type'' field set to ''DEV_MOTHERBOARD_PCI''. If for example there existed a PCI bus on a PCI bus you could end up with a driver header looking like this.
<pre>
DRIVER pci_generic = {
	.szName = "PCI To PCI Bridge Generic",
	.connection = DEV_CONNECTION_PCI,
	.connectionVersion = DEV_CONNECTION_PCI_VERSION,
	.connectionData = &connectionData,
	.priv = 0,
	.ifaceCount = 0,
};
</pre>
You can notice I set the ''connection'' field to ''DEV_CONNECTION_PCI'' instead of ''DEV_CONNECTION_MOTHERBOARD'' in the previous example. You might also allow one module contain two driver headers by using something of the such.
<pre>
DRIVER pci_generic = {
	.szName = "PCI GENERIC",
	.connection = DEV_CONNECTION_MOTHERBOARD,
	.connectionVersion = DEV_CONNECTION_MOTHERBOARD_VERSION,
	.connectionData = &connectionDataPci,
	.priv = 0,
	.ifaceCount = 0,
};
DRIVER pcitopci_generic = {
	.szName = "PCI To PCI Bridge Generic",
	.connection = DEV_CONNECTION_PCI,
	.connectionVersion = DEV_CONNECTION_PCI_VERSION,
	.connectionData = &connectionDataPciToPci,
	.priv = 0,
	.ifaceCount = 0,
};
</pre>
This (above) could allow each to have their own interfaces or even share the exact same interfaces and driver device driving code, but provide functionality for the motherboard helper and the PCI bus driver. This gives a lot of flexibility when writing drivers not only for today's systems but tomorrow's possibilities. Lets take one last look at a example of a driver header, but this time we will see a VGA driver's driver header.
<pre>
static DEV_CONNECTION_PCI_DATA connectionDataPci = {
	.vendor = 0x1013,
	.device = 0xB8,
	.Init = &video_vga_pci_init
};

static DEV_CONNECTION_MOTHERBOARD_DATA connectionDataMotherboard = {
	.type = DEV_MOTHERBOARD_VGA,
	.Init = &video_vga_generic_init
};

static IPICTURE_PICTURE ifacePicturePicture = {
	.BltBlt = &BltBlt,			// basic picture functions not related to vga
	.SetFormat = &SetFormat,
	.Clear = &Clear
};
static IPICTURE_VGA ifacePictureVga = {
						// vga speacilized functions
	0 /* just a example but no interface functions shown */
};

DRIVER clgd5446_vga = {
	.szName = "CIRRUSVGA",
	.connection = DEV_CONNECTION_PCI,
	.connectionVersion = DEV_CONNECTION_PCI_VERSION,
	.connectionData = &connectionDataPci,
	.priv = 0,
	.ifaceCount = 2,
	.iface = {
		{
			.szName = "PICTURE.PICTURE",
			.major = DEV_PICTURE,
			.minor = DEV_PICTURE_PICTURE,
			.iface = &ifacePicturePicture
		},
		{
			.szName = "PICTURE.VGA",
			.major = DEV_PICTURE,
			.minor = DEV_PICTURE_VGA,
			.iface = &ifacePictureVga
		},
	}
};

DRIVER generic_vga = {
	.szName = "VGA",
	.connection = DEV_CONNECTION_MOTHERBOARD,
	.connectionVersion = DEV_CONNECTION_MOTHERBOARD_VERSION,
	.connectionData = &connectionDataMotherboard,
	.priv = 0,
	.ifaceCount = 2,
	.iface = {
		{
			.szName = "PICTURE.PICTURE",
			.major = DEV_PICTURE,
			.minor = DEV_PICTURE_PICTURE,
			.iface = &ifacePicturePicture
		},
		{
			.szName = "PICTURE.VGA",
			.major = DEV_PICTURE,
			.minor = DEV_PICTURE_VGA,
			.iface = &ifacePictureVga
		},
	}
};
</pre>
Here we can see that in a example VGA driver that might be used in development of our kernel we actually have two drivers in one module. The first is a driver for the '''Cirrus CLGD5446''' video card, and the second is a generic VGA driver. Each includes the appropriate values so that in reality this same module with two drivers could support two physical video cards in the system. One being a generic VGA adapter and only being a PCI card inserted into the PCI bus for the '''Cirrus CLGD5446'' chip set. I hope I have given you a good understanding of the driver header. Lets continue by exploring the interface structures just a bit.
====Initialization Handler====
The initialization handler will provide the ability for a bus driver or the motherboard helper to create a instance of their driver which will immediantly start to control to device they were intended for. The language to tell the driver what device to control in a system is implemented in the ''connectionData'' field's custom data pointed to such as: ''DEV_CONNECTION_DEV_VERSION,DEV_CONNECTION_MOTHERBOARD_VERSION,DEV_CONNECTION_PCI_VERSION'' for examples. It is dependent on these structures being able to convey the information needed to be passed. For example lets take a look at the ''DEV_CONNECTION_PCI_DATA'' structure.
<pre>
#define DEV_CONNECTION_PCI_VERSION	0x00000000
struct tdevBusDataPci{
	u32 vendor;
	u32 device;
	i32 (*Init)(u32 bus, u32 slot);
}; typedef struct tdevBusDataPci DEV_CONNECTION_PCI_DATA; typedef struct tdevBusDataPci* PDEV_CONNECTION_PCI_DATA;
</pre>
Here we can see that it defined that the driver that wishes to use it as a connection must export a function that takes the arguments ''bus'' and ''slot''. This enables to driver to perform a query back to the PCI bus driver for what resources it needs to control by identifying it's self as ''bus->slot'' when making calls. If more information was need it should be defined here by the developers of the PCI bus system for the kernel.

The example above for the ''Init'' function is called a initialization handler. All drivers should export some sort of initialization mechanism even if it is not in the ordinary function call way. In specialized cases which I can not think of it could be possible to not need a initialization handler, however that would be completely out of scope of this article but we do leave room for it. Lets take a look at a motherboard helper initialization handler.
<pre>
i32 mobo_generic_init(void){
	/// do a sanity check if possible or needed
	......
		/// sanity check failed then exit and hopefully no harm done.
		.......
	/// create a instance of ourselves
	.......
	///  check if the APIC is avaible and use that instead of the PIC.
	.....
		/// find and get the PIC running
		......
	/// find and get any generic clock running that is possible.
	.....
	/// find and get pci bus running.
	......
	return 1;
}
static DEV_CONNECTION_DATA connectionData = {
	.vendor = 0,
	.board = 0,
	&mobo_generic_init	
};
</pre>
Here is a general outline of what you might do in a generic motherboard helper.

1.0 Do a sanity check if possible or needed.
: ''Here we might make a final determination of weather the driver should actually load. This is normally the second part of a two part system. The first part was where for instance the PCI bus driver associated a vendor and device identifier with this driver header. However in some real world cases this driver might not actually be able to handle this device and this is the chance it gets to make a determination and exit before creating a instance.''
: 1.1. If sanity check failed then exit.
:: ''The sanity check failed because the driver's initialization handler determined there was a insane condition present.''
2.0 Create a instance.
: ''We create a instance and the kernel device subsystem performs the necessary loading of the interfaces and properties.''
3.0 Check if the APIC is avalible and use that instead of the PIC.
: ''For a generic motherboard helper we might try to see if the APIC exists and use that instead of the PIC.''
4.0 Find and get the PIC running.
: ''The APIC did not exit so lets find a driver for the PIC, or in a similar case the APIC did exist and we find a driver for it instead.''
5.0 Find and get any generic clock running that is possible.
: ''We look for any clocks that we can support and try to initialize them. ''
6.0 Find and get PCI bus running.
: ''Get the PCI bus up and running. Some devices might need a clock or other common devices so we load it last.''

Of course that is simple a rough outline of what you might or could do. I make no assertion that you should or should not do these things or in this order but rather assume you will use it to better your idea of what you want to do. This exact same case could be applied to a PCI driver such that.
<pre>
int pci_init(void){
	// do a sanity check..
	// create a instance..
	// scan the PCI bus
		// search for a driver for each device found
		// try to initialize the driver and let it create a instance if needed.
	// exit function
	return 1;
}
</pre>
The motherboard helper initialization handler would actually end up making a direct call to this ''pci_init'' initialization handler, and once it runs it might make more calls into the various drivers it might load which would terminate with returns and arrive hopefully if all goes well back at the original caller which would be the kernel device subsystem.
==The Device Subsystem==
The device subsystem in the kernel source tree is responsible for providing the basic structures or framework on which the drivers are built from. It also provides some general services to the drivers such as: searching, hardware resource management, instance creation and destruction, and system calls. To help explain some of the next sections lets implement some basic framing for a kernel device subsystem to work from.
<pre>
// This is just a list of drivers known by the subsystem. This drivers may reside in memory by being staticaly compiled with the
// kernel, or on disk.
OLIST dev_drivers = 0;
// This is a list also, except it repersents instances of the list dev_drivers. When a instance is created the actual driver is known
// to be placed into a running state driving the device. This list allows searching of availible drivers.
PDEVINSTANCE dev_instances = 0;
PDEVINSTANCE dev_CreateInstance(PDRIVER driver);
// The entry point called by the kernel's main flow when booting.
void dev_init(void);
// A resource allocation linked list.
struct tdevres *dev_resl = 0;
// The function to allocate resources to the link list.
int dev_res_alloc(PPRIVATE priv, u8 type, u16 start, u16 count);
</pre>
You can see that I maintain a list of known drivers as ''dev_drivers'' which of these could live in memory from being staticly linked with the kernel during the build, or drivers which have been detected by different means not only during boot by also during run-time of the kernel. The drivers in the ''dev_drivers'' are never actually executed but instead remain there in a sort of database form for future or current usage.

The future of current usage happens when through the chain of booting of drivers, they are called and the driver creates a instances of it's self or when the system user specifies a command to the device subsystem to perform a rescan and attempt to load missing drivers. This could caused by the kernel being booted and the user noticing a driver missing for some hardware. The user downloading the driver and then wanting to load and have this driver picked up with out restarting the kernel.

The resource allocation is performed after a driver creates a instance of it's self. Once a instance is created the driver will attempt to allocate any resources such as: memory ranges (physical and virtual), I/O ports, and any other type of resource. The structure for holding a resource allocation and some common resource types are:
<pre>
struct tdevres{
	unsigned char		type;
	unsigned int		value[2];
	struct tdevres		*next;
};
#define DEV_RES_IO	1
#define DEV_RES_REGION	2
</pre>
You is completely up to you how to impelment this, but for a example I have listed two resouces types with will be stored in the ''type'' field of the ''struct tdevres'' structure. The first is ''DEV_RES_IO'' which repersents a I/O port range that can be reserved by a driver. The second is ''DEV_RES_REGION'' with repersents a region of memory that can be reserved by a driver. If I wanted I could include a value that repersented a region of memory being allocated as physical or virtual if it meet the needs of my kernel. Once a resource allocation is made no other drivers or software in the system should ever have direct access to this resources except the driver that made the allocation of them.
===Searching Services===
The searching services allow drivers, kernel space software, or user space software to query for certain types of drivers or instances. This ability for user space software to access query services is encapsulated by the system calls provided by the device sub system too. The simple reasoning behind having the subsystem implement the searching services is to keep drivers from having to implement the same exact code over and over again. Instead lets maintain this functionality in one place instead of multiple places.
===Hardware Resource Management Services===
The hardware resource management services allow drivers to allocate and free resources. This provides a means for devices to determine if the resources they need are currently in use, and if it is possible for them to configure to use a different set of resouces then they can do so with minimal user intervention. This also prevents drivers from unintentionaly trying to use the same resources which give a high fail safe mechanism. Drivers should never use a resource before first allocating it and checking for a error condition being present on the allocation call.
===Instance Creation And Destruction Services===
The instance creation and destruction services help drivers perform a common task which is to actually employ themselves to control a device and become registered in the kernel as a loaded, initialized, and running device driver. This also allows changes in the core kernel device mangement subsystem to happen with out changing and recompiling all drivers since they ended up using their own instance creation and destruction routines. It becomes a much better idea to allow this to become centralized rather than bulking up every driver when this exact same code group has to be implemented each time instead of just once using the instance creation and destruction services provided by the kernel device management subsystem.
===System Call Services===
The device subsystem does not implement the actual system call mechanism, but serves as a entrance for user space software to access the subsystem through the usage of system calls. You might implement system calls in numerous ways however it is most prabobale that you're implementation supports a naming convention or call function with arguments and a result. These three things are most likely exactly what the system call into the device subsystem would need also: a function name, arguments, a result.
===Overview===
The device subsystem is comprised of searching services, hardware resource management, instance creation and destruction services, basic structures and framework for drivers, and system call services. 
==Driver Module Format==
The driver module format will cover a stand alone binary file that contains the driver(s) machine code, and the driver(s) data. From this point you can move to compile-time linking or run-time linking for the drivers. I prefer to first implement the compile-time linking since it is a little easier with less overhead requirements from you're kernel. Such modules that contain generic disk drivers and other critical things are most likely best done in a compile-time linking fashion while leaving all the other non-critical ones left to be loaded during run-time. However this is simply a suggestion and it is up to you how to implement this into you're own kernel.
===Compile-Time Linking===
Compile-time linking of drivers is when they are included into the kernel build using a static or automated method. A static method is when you directly add the source and supporting source and headers into the source tree with out a isolation method to prevent the compilation of the driver with out editing the actual source. A automated method involves using a external tool that will staticly include the drivers into the source tree during a compile, but the drivers will remain out of the source tree when the automated mechanism is disabled or removed. A example of a automated method could be the when using Makefiles and having a driver directory with the first Makefile inside. This Makefile could be responsible for triggering a action that results in a static include of the driver into the kernel source for the current compilation. This makefile would be the mechanism for enabling or disabling the automated compile-time linking of drivers into the kernel.

I prefer compile-time linking personally since it has requires less overhead in the kernel at compile and run time. It does take a little effort to produce a quality automated mechanism however I will try to guide you into a quick and dirty one that will work for a small kernel project.
====The Makefile And Driver Directory====
===Run-Time Linking===
==Kernel Abstractions==
===IRQ Routing Support===
===Clock Support===
````

<a id="w001992"></a>

## W001992 — User talk:Almafeta

Bron-ID: W001992
Namespace: User talk (3)
Revisie: 3479 | Datum: 2007-03-26T03:27:24Z
Bron: https://wiki.osdev.org/User_talk:Almafeta
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AAlmafeta&oldid=3479

````text
The <tt>=</tt>'s you were wondering about should be replaced by <tt><nowiki><tt></nowiki></tt> and <tt><nowiki></tt></nowiki></tt>. Also <tt><nowiki>'''</nowiki></tt> is the MediaWiki equivalent of <tt><nowiki><b></nowiki></tt> and <tt><nowiki></b></nowiki></tt> -[[User:Jhawthorn|Jhawthorn]] 22:27, 25 March 2007 (CDT)
````

<a id="w001993"></a>

## W001993 — User talk:Kmcguire/Quick And Dirty Device Management

Bron-ID: W001993
Namespace: User talk (3)
Revisie: 17810 | Datum: 2015-03-15T08:32:22Z
Bron: https://wiki.osdev.org/User_talk:Kmcguire/Quick_And_Dirty_Device_Management
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AKmcguire%2FQuick_And_Dirty_Device_Management&oldid=17810

````text
== Conflicting Edits ==
Okay, what was wrong with my finishing of your overview? I thought it was a nice finish, since you started with the magic energy balls and stuff. Shouldn't a wiki have some humor? - Alboin

You edited it while I was editing it. I pushed edit and a few minutes later you edited, and about twenty minutes later I pushed save and it over wrote it.   I only noticed it after it happened, but figured I would stick it back in on the next save if you had not. I still have a lot more to push onto the end of it. - [[User:Kmcguire|kmcguire]]

Oh......sorry then.......this is why I shouldn't edit the wiki at midnight.....bad things happen..... If you have more to add to the end you can just toss mine then and do what you were planning. I wasn't aware that anyone else was working on it, so I figured it needed more than a sentence. - Alboin

== Use of "Motherboard" ==
The document suggests that you need a driver for the motherboard (which pretends to be a bus). This is rather confusing as:
* the motherboard is actually a collection of devices and buses
* the motherboard should be transparent to the kernel (and for most x86s, it is)
The processor is hooked up to the system bus according to the intel architecture, so kernels do not need to worry about the top-level bus (or buses, if you want to include the processor's on chip buses) as it is defined. Most buses beyond that are made transparent by the bios and need no specific modifications. Hence what is referred to as 'the motherboard driver' is for one part integral to the kernel, while the second part is a collection of chip drivers (PIT, APIC, PC speaker, PCI, etc, etc). I'd recommend removing the motherboard=bus analogy as its bad for the reasons above - [[User:Combuster|Combuster]] 06:32, 26 March 2007 (CDT)

You're recommendation is accepted by me, and as the original author I will made amendments to my local article from this copy then and push it back. - [[User:Kmcguire|kmcguire]]
````

<a id="w001994"></a>

## W001994 — FreeBASIC

Bron-ID: W001994
Namespace: Main (0)
Revisie: 12296 | Datum: 2011-12-12T14:47:28Z
Bron: https://wiki.osdev.org/FreeBASIC
Revisie-URL: https://wiki.osdev.org/index.php?title=FreeBASIC&oldid=12296

````text
FreeBASIC is the name of a modern Basic language variant, as well as the corresponding compiler. Whereas classic Basic can not do native OS development, Freebasic comes with lots of features useful for OS Development. 

== Overview ==
Apart from being mostly QuickBasic compatible, it offers the following useful features:
* Unsigned data types
* Inline assembly
* Pointers
* Callbacks
* A portable runtime.

The major downside is that most of the BASIC-specific instructions require the runtime which can not be easily used in an freestanding environment. The consequence is that several typical BASIC constructs can not be used until runtime support is included, making development very tricky.

== Usage ==
FreeBASIC has a command line syntax similar to that of [[GCC]]:
 fbc -c -o kernel.o kernel.bas
to link the output:
 ld -T script.ld -o kernel.bin kernel.o

To get a running start, there is a [[FreeBasic Bare Bones]] tutorial that ends in a working "hello world" kernel.

== Runtime Library ==
In an freestanding environment, the runtime is unavailable. Programmers should take care not to use any of the following:
* Strings
* Dynamic arrays (only local, fixed-size arrays are allowed)
* Globals of any kind. All information must be passed in arguments.
* All non-primitive built-in functions, i.e. anything more complex than + - * / and the logic operators.
* IO functions, including port outs and ins, peek and poke.

You'll need to provide a runtime init to allow the use of globals, you'll need a malloc() to get strings and arrays working (the rest of the dependencies are cross-platform ctype+stdlib functions and don't pose much of a problem).

If you sit down and think about it, writing a kernel in C, and writing the same thing in FreeBASIC is just about the same. Of course, there's that huge language difference, what with FreeBASIC's syntax resembling English a little more than C...but to do anything useful, both strategies need a C runtime and library. With the FB kernel, after you've managed to port a C library, you could think about porting the FB runtime on top. What's more, FB can link into existing C libraries, so porting a C library isn't a pain.

== See Also ==
=== Articles ===
* [[FreeBasic Bare Bones]] tutorial
* Alternative [[Languages]] in general.

=== External Links ===
* FreeBasic [http://www.freebasic.net/ homepage]
* The [http://www.freebasic.net/wiki/wikka.php?wakka=DocToc FreeBasic Manual]
* FreeBASIC tutorial: [http://www.ascii-world.com/fb-to-gedit-howto syntax highlighting in gedit]

[[de:FreeBASIC]]
[[Category:Compilers]] [[Category:Languages]]
````

<a id="w001995"></a>

## W001995 — Talk:Main Page

Bron-ID: W001995
Namespace: Talk (1)
Revisie: 10367 | Datum: 2010-05-25T08:51:06Z
Bron: https://wiki.osdev.org/Talk:Main_Page
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMain_Page&oldid=10367

````text
:For discussing a specific main page, please see ''[[Talk:Categorized_Main_Page|Short view]]  ([[Talk:Categorized_Main_Page/archive|archive]])'', or ''[[Talk:Expanded_Main_Page|Expanded view]]''
----


== Expanded main page up front ==

Suggestions have been recently (and in the past) made to make the [[Expanded Main Page]] the default welcome page. I've updated the expanded page with some of the more recent additions, and made a copy of the current main page at [[Categorized Main Page]]. Most of the reasons can be found [http://forum.osdev.org/viewtopic.php?f=6&p=175469#p175469 here]. Opinions and comments wanted, since this is the main page three votes will not really suffice. - [[User:Combuster|Combuster]] 19:40, 26 April 2010 (UTC)

* '''Pro''' - I voiced my reasons in the linked thread already. -- [[User:Solar|Solar]] 09:36, 27 April 2010 (UTC)

* '''Yea''' - I find myself getting annoyed when I have to click through to the expanded page to find the information I'm looking for. The generic main page wasn't really useful at all. --[[User:Lithorien|Lithorien]] 10:48, 27 April 2010 (UTC)

* '''Agreed''' - The short view is good, but the referenced pages show too little of what the entire wiki has to offer. I vote for the expanded main page.--[[User:Creature|Creature]] 14:28, 27 April 2010 (UTC)

* '''Sure''' - Why not, there is no sense hiding anything. --[[User:Brynet-inc|Brynet-Inc]] 18:16, 28 April 2010 (UTC)

* '''Yes''' - It's been ages after I came to this site until I noticed there was an expanded main page.--[[User:Love4boobies|Love4boobies]] 21:02, 28 April 2010 (UTC)

"Thee votes will not really suffice" - well, we got five. ;-) Do we pick up the discussion again (seing how the linked thread derailed and was closed), or do we consider this 5-to-null vote to be sufficient? (I'm itching to get working on the Wiki, and this *is* the starting point.) -- [[User:Solar|Solar]] 08:19, 4 May 2010 (UTC)
:Well it's not unanimous but 5 vs 1 (see heading below), but that is still about as much as you'd normally see on a wiki vote. Will perform the update later today when I've ran out of appointments ;) - [[User:Combuster|Combuster]] 12:50, 4 May 2010 (UTC)
::And... updated - [[User:Combuster|Combuster]] 16:35, 4 May 2010 (UTC)
:::It's a bit annoying that it shows "(Redirected from Main Page)" - I couldn't see a "move" link. Perhaps making a copy called Main_page and deleting this one will do the trick? --[[User:Love4boobies|Love4boobies]] 19:40, 21 May 2010 (UTC)
::::The main pages have the protect flag on to prevent erroneous edits (edit the templates instead). - [[User:Combuster|Combuster]] 08:51, 25 May 2010 (UTC)

== More info needed ==

The main reason I like the short form is that it's short. I'm just looking for an entrypoint to the wiki, and I don't need 3 screenfuls of info. On the other hand, the short page AFAIK is entirely formatted via a template. I have no idea how to even edit that template. So (at least the last time I thought about doing it) the short main page is basically uneditable to the uninitiated. Which is probably one reason why it is not rationally organized, and does not really represent what's in the wiki, currently. Especially the OS Theory block.<BR>
In some sense, it's a good thing to discourage tinkering at the topmost level of the wiki by novices. But it makes the main page ''severely'' out of date.<BR>
So the main question in my mind is -- is the expanded view any better? Is it still a template and not a normal page? And since it's so expanded, why not have a "Categories" section in it, that gives direct access to the Category system, and thus incorporates the functionality of the short view? [[User:Bewing|Bewing]] 14:26, 28 April 2010 (UTC)<BR>
:The many ''"more..."'' links cover the category tree very well. It is still based around templates for redundancy reasons. You can find the relevant templates much easier than for the current main page, as they're all named as [http://wiki.osdev.org/index.php?title=Special%3AAllpages&from=Main2&namespace=10 Template:Main2/(...)] (If you have the name, you can find [http://wiki.osdev.org/index.php?title=Special%3AAllpages&from=&namespace=10 the category templates] for the current main page). I can try to categorize the main page templates for easier locating if that's what you want. - [[User:Combuster|Combuster]] 18:51, 28 April 2010 (UTC)
````

<a id="w001996"></a>

## W001996 — Talk:ISA DMA

Bron-ID: W001996
Namespace: Talk (1)
Revisie: 13333 | Datum: 2012-05-07T00:32:29Z
Bron: https://wiki.osdev.org/Talk:ISA_DMA
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AISA_DMA&oldid=13333

````text
== rewrite ==
Two thirds of this article is about generic ISA DMA programming. That part of it does not belong in the "Storage" category at all.
One third of it is about DMA Floppy Driver programming. This very much belongs where it is.
The article desperately needs to be busted into two pieces. ISA DMA programming goes somewhere else (and NOT just named "DMA" either!). The remaining article links to the ISA DMA article, and gets renamed "Floppy DMA", I think.
And the Genesis thing is amusing, but certainly overdone. And it makes baby Brynet cry.
[[User:Bewing|Bewing]] 03:01, 23 March 2008 (CDT)
:Actually, I'd like to see a split into three parts: [[DMA]] (theory), [[PC DMA]], [[Floppy Programming]]. Doing so would make a complete series on the matter (what is dma, how does it work, and how you can use it). Unfortunately, there's little in this article to provide the first. A stub is however easily made. (and I don't mind adding a bit of common sense to it when I get the time.). WRT Genesis, I kinda like it <font color="white">(and who cares about what brynet thinks anyway `o.o´)</font> - [[User:Combuster|Combuster]] 04:10, 25 March 2008 (CDT)
::On that topic, I feel as if this article does more bashing than adding actual information. Yes we get it, DMA sucks and should be dead and buried, but do we have to beat it down with a shovel? [[User:Primis|Primis]] 19:35, 29 December 2011 (EST)
::Actually DMA is still alive. ISA is what has been bashed on time and time again and yet it's kept on life support for an equal period of time. Any specific points of improvement you are suggesting (other than that the DMA theory page is still missing) - [[User:Combuster|Combuster]] 11:46, 30 December 2011 (UTC)
::I think that, at the very least, we should write some example code in C, Organize the information better. I also think we should have more information about busmastering DMA, since, this article is about DMA, not just ISA. - [[User:Primis|Primis]] 16:00, 31 December 2011 (EST)

==rewrite==
This page could use a rewrite. -Mystran 2007-04-02
:I definitely agree that it needs a lot of work. I really dislike, among other things, the "DMA Genesis" section. That said, I think it's better to improve (3rd person, more informative, to the point) the current article until it meets expectations than to restart it. The "Floppy Disk DMA Programming" could be very helpful (I have seen endless forum posts asking for help on floppy drivers). This section should be cleaned up, written in C and possibly split into another article. -[[User:Jhawthorn|Jhawthorn]] 21:27, 2 April 2007 (CDT)


''I've added the following to what was orignally here, some of it has been re-written a great deal of it has been written from personal experience at swearing at the little DMA chip or it's functional equivalent. Still more to come, there's a lot of thoery to understand before simply writing code. The DMA chip has a lot of 'interesting' and 'unexpected' features. Failing to take these into acount will result in nothing working.            You have been warned :-). XARDFIR''

== Actual 8237 information ==

I don't know if you've ever thumbed through the 8237 datasheet, but I wanted to point out, the IBM implementation of the chip is only missing one signal, /EOP. This technically can interrupt the processor upon DMA completion. There are no other signals on this chip not brought out on the ISA bus.
````

<a id="w001997"></a>

## W001997 — User:Jhawthorn/Pages To Fix

Bron-ID: W001997
Namespace: User (2)
Revisie: 3536 | Datum: 2007-04-03T02:28:53Z
Bron: https://wiki.osdev.org/User:Jhawthorn/Pages_To_Fix
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJhawthorn%2FPages_To_Fix&oldid=3536

````text
*[[DMA]]
*[[Floppy Driver]]
````

<a id="w001998"></a>

## W001998 — User talk:Combuster/Manual Of Style draft

Bron-ID: W001998
Namespace: User talk (3)
Revisie: 3540 | Datum: 2007-04-03T07:34:35Z
Bron: https://wiki.osdev.org/User_talk:Combuster/Manual_Of_Style_draft
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ACombuster%2FManual_Of_Style_draft&oldid=3540

````text
===Things I would like to see specified===
* kB or KiB
* Intel or AT&T assembler syntax. Probably can't get consensus on this one.
I will add more when I think of them -[[User:Jhawthorn|Jhawthorn]] 21:39, 2 April 2007 (CDT)


* all units of bytes should go in powers of two - that means KiB, MiB, GiB are correct usage. However, KB, MB and GB are more common in usage and in most contexts here describe the same amount. [http://en.wikipedia.org/wiki/Binary_prefix See Wikipedia]. Adding a note of preference for the KiB variant might be ok.
* Forcing intel or att syntax will only cause holy wars.
* There's also a request for the "See Also" style in the main page archive. 
IMO you should ask these questions in the manual of style thread on the forum where people read it, get consensus on it, then merging it into the official manual of style. 

Speaking of which, I think it is time the current version is merged into the wiki before I add above remarks - [[User:Combuster|Combuster]] 02:34, 3 April 2007 (CDT)
````

<a id="w001999"></a>

## W001999 — OSDev Wiki:Manual Of Style

Bron-ID: W001999
Namespace: OSDev Wiki (4)
Revisie: 30262 | Datum: 2026-03-16T16:12:52Z
Bron: https://wiki.osdev.org/OSDev_Wiki:Manual_Of_Style
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDev_Wiki%3AManual_Of_Style&oldid=30262

````text
This page is a guideline for those writing new articles or contributing to existing ones. Being a guideline, these rules are in no case binding, though we expect that the articles follows the concepts as appropriate. Hence these rules can be overridden by common sense.

== Article Naming Conventions ==
* Titles start with a capital letter (as if it were the start of a sentence)
* Words are separated with spaces, each new word starting with another capital
* Further capitalizing and use of non-character symbols is to be done according to the English language.

examples:
* kbc commands -> KBC Commands
* gcc cross compiler -> GCC Cross-Compiler

bad titles:
* hardware (rule 1: should start with a capital: [[:Category:Hardware|Hardware]])
* FileSystems (rule 2: words are separated with spaces: should be [[File Systems]])
* Cpuid (rule 3: The Intel manual defines this in all-caps: should be [[CPUID]])


== Article Contents ==
Articles should be informative in style. FAQ style is discouraged. Tutorials are allowed, but try to keep instructional content separate from encyclopedic content.

=== Code Snippets ===
Code snippets can be used on any page where useful. 

Code snippets should be readable and understandable:
* Proper indentation: little trouble for a largely increased readability. Things like whether you put your {}'s on the same line or on the next is subject to your taste
* Avoid long lines: these break the layout as well as making things very difficult to read. 40 characters should be fine.
* Comments: Use common sense. Short snippets might not need any, longer ones are more likely. Assembly code can in general use more comments than higher level languages.
* Language: The preferred language is either C or Assembly, as most programmers know these. Try however to minimize mixing of languages (inline assembly).

Code blocks can be marked with a leading space in front of each line or by using the '''pre''' tags. However, the wiki supports the '''syntaxhighlight''' tags, which are preferred since they provide syntax highlighting for several languages. For a list of supported languages, see [https://pygments.org/languages/].

<pre>
// C code formatted using pre tags.
int main(void)
{
    return 0;
}
</pre>

<syntaxhighlight lang="c">
// C code formatted using source tags with lang="c".
int main(void)
{
    return 0;
}
</syntaxhighlight>

=== Tables ===
Tables should be created like this:
<pre>
{| {{wikitable}}
! Header 1
! Header 2
|-
| Value 1 || Othervalue 1.1
|-
| Value 2 || Othervalue 2.1
|-
| rowspan="2" | Value 3 || Othervalue 3.1
|-
| Othervalue 3.2
|}
</pre>
The table will look like this:
{| {{wikitable}}
! Header 1
! Header 2
|-
| Value 1 || Othervalue 1.1
|-
| Value 2 || Othervalue 2.1
|-
| rowspan="2" | Value 3 || Othervalue 3.1
|-
| Othervalue 3.2
|}
Additionally you can use the <nowiki>{{Yes}} and {{No}}</nowiki> for comparison tables. You can create notes in tables with the following codes (the first to insert a reference, the second is the reference text):
<pre>
<sup>[[#Note1|1]]</sup>

<small id="Note1">Note 1: Some description here.</small>
</pre>

=== Sections with additional info ===
The layout of additional info sections should be like this:
<pre>
== See Also ==

=== Articles ===
* [[Some Article]]
* ...

=== Threads ===
* [[Topic:18304|Name of forum thread]] - Write what you'll find in the thread here
* ...

=== External Links ===
* [[wikipedia:Operating system|Operating system]] on Wikipedia
* [http://www.osdever.net/ Name of homepage] - Write what you'll find on the homepage
* ...
</pre>

== Discussing articles ==
Issues that might need a discussion should be posted on the talk backpages. when you do, please use four tildes to sign your message: <nowiki>~~~~</nowiki>

More important issues should be posted on the wiki forum, as well anything not directly related to any single article, as these are checked by more people.


== Sandbox ==
Each member has his own userpage, in which he is free to try things as needed. Please try to minimize Category:xxx links as they cause links to appear to your article. You can link to a category by inserting a colon in the beginning of the wikilink like this: <nowiki>[[:Category:Tutorials]]</nowiki>.

You can have a look at some other people's user pages for ideas.

== Discussing the Manual of Style ==
To request changes, please post a message to the [[Topic:12334|Manual of Style thread]]

[[Category:Wiki administration|{{PAGENAME}}]]
````

<a id="w002000"></a>

## W002000 — Floppy Driver

Bron-ID: W002000
Namespace: Main (0)
Revisie: 4100 | Datum: 2007-06-17T11:44:02Z
Bron: https://wiki.osdev.org/Floppy_Driver
Revisie-URL: https://wiki.osdev.org/index.php?title=Floppy_Driver&oldid=4100
Doorverwijzing: Floppy Disk Controller

````text
#REDIRECT [[Floppy Disk Controller]]
````

<a id="w002001"></a>

## W002001 — Talk:Floppy Driver

Bron-ID: W002001
Namespace: Talk (1)
Revisie: 6223 | Datum: 2008-08-20T16:11:33Z
Bron: https://wiki.osdev.org/Talk:Floppy_Driver
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AFloppy_Driver&oldid=6223
Doorverwijzing: Talk:Floppy Disk Controller

````text
#REDIRECT [[Talk:Floppy Disk Controller]]
````

<a id="w002002"></a>

## W002002 — Talk:X86 Interrupts

Bron-ID: W002002
Namespace: Talk (1)
Revisie: 30757 | Datum: 2026-06-15T16:30:27Z
Bron: https://wiki.osdev.org/Talk:X86_Interrupts
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AX86_Interrupts&oldid=30757

````text
==Renaming==
Could this be renamed just 'Interrupts' and have [[I Cant Get Interrupts Working]] added to the end as a 'Troubleshooting' section? --Alboin
:The rename is Ok. I wouldn't merge a FAQ page with a informational page however. - [[User:Combuster|Combuster]] 08:21, 14 July 2007 (CDT)

::There just has to be copyright and junk on the "For Dummies" title.. A rename would be advisable.. --[[User:Brynet-Inc|Brynet-Inc]] 11:08, 14 July 2007 (CDT)
:::Done. --Alboin

==Contributors==
NotTheCHEAT, MikeWikiUser, PypeClicker

== Updating the page/moving the x86 specific stuff into a separate article ==

This page assumes x86, and for some reason also talks about S390, and I think that this page could perhaps be reworked to be more generic, and the x86 and S390 stuff could be moved to their own appropriate pages (to existing or new ones)...

The second issue is that the x86 stuff is again outdated (I have no experience with S390); it is assuming PIC (not even present in new PCs), talks about keyboard controller (also not present in many PCs), IDT (which is getting replaced by FRED), ISA interrupts (great information for 30 years old PCs, but you should get that stuff from ACPI or whatever), all of this stuff should be updated.
--[[User:Mishakov|Mishakov]] ([[User talk:Mishakov|talk]]) 21:38, 14 June 2026 (UTC)


Yes, this article needs to be generalized (there are other articles that need that too like the paging article). Split the arch-specific parts to their own articles.
Next, regarding the "outdated" contents, being outdated doesn't make the info worthless. It will still be useful for working with older hardware, so either that information should be moved to a new article or explicitly marked as old.

You also mention the FRED in x86, but it is still too new to declare the IDT obsolete. Give it a decade or so.

--[[User:Tocariimaa|Tocariimaa]] ([[User talk:Tocariimaa|talk]]) 23:12, 14 June 2026 (UTC)
````

<a id="w002003"></a>

## W002003 — Talk:Beginner Mistakes

Bron-ID: W002003
Namespace: Talk (1)
Revisie: 30981 | Datum: 2026-09-28T13:14:56Z
Bron: https://wiki.osdev.org/Talk:Beginner_Mistakes
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABeginner_Mistakes&oldid=30981

````text
:''ToDo: Extend this by misconceptions encountered around the web and in the forum.''

Do you think this page is ready for removal out of "Pages for Review"? Or is there anything else that needs to be added that has come up recently on the forum? --Alboin
:It is '''NOT''' at all ready. It still references pages by their old name (ex. "using some language other than C"), is still in FAQ format, isn't formated correctly, links missing, section headers too long, uncategorized... -[[User:Jhawthorn|Jhawthorn]] 00:10, 6 April 2007 (CDT)
:: '''Can you please''' tell me what FAQ format is? I don't really know what you mean by that. I see your point about formating and such, but I'll wait to attempt to fix it though until I see what this FAQ formatting things about. Thanks. --Alboin
:::We seem to have taken a different style of naming sections on the new articles. Example: On the OSFAQ we'd see something like "I have to write an OS for my studies, deadline is in three months..." and on this new wiki we'd likely have something like "Deadlines" or "Time Constraints". -[[User:Jhawthorn|Jhawthorn]] 18:21, 6 April 2007 (CDT)

ATM, we should find a way to make it reachable from the rest of the wiki. This includes a category (which one is a difficult question), and some direct links from other articles. Once that's done the PFR tag can be removed - [[User:Combuster|Combuster]] 16:49, 10 April 2007 (CDT)
:What about in Category:OS_development? --Alboin
::Will do, adding it - [[User:Combuster|Combuster]] 02:03, 11 April 2007 (CDT)
:What about directly linking it to [[Troubleshooting]] and [[Getting Started]]? --Alboin

About "OSDEV will probably never land you a job." Is this really true? I'm sure that such low level experience comes in handy in the driver development and embedded fields. --[[User:Earlz|Earlz]] 08:50, 24 April 2010 (UTC)
:In one of its ambiguous meanings it's true: you probably won't get a job as an OS developer more than what you do with regular education. A masters degree often earns you more than some code. And even though I got a job in the embedded market, I wonder how much the "OS" in "I made this nice (OS) project" made a difference. - [[User:Combuster|Combuster]] 08:51, 26 April 2010 (UTC)
::It's not just that -- most beginners will probably misinterpret the word OS: they will think about something like Windows, Linux or OS X. The word "never" has a certain amount of gravity that is probably more suited. They can figure it out for themselves later. :) --[[User:Love4boobies|Love4boobies]] 11:13, 26 April 2010 (UTC)

Forth was suggested; however information on Forth OS dev is sparse from this site or the web. This seems pretty advanced and out of scope for a beginner. --[[User:Eve|Eve]] 20:55
Sunday, 19 February 2023 (UTC)

This page seems a bit harsh, I was able to start OSDev without much programming knowledge at all (outside of basic C syntax and types), though it was very hard to learn over the past ~3 years or so. I think that it may be more appropriate to make it a bit less harsh, any thoughts? --[[User:anotheridiot|Anotheridiot]] 13:14, 28 September 2026 (UTC)
````

<a id="w002004"></a>

## W002004 — Talk:Symmetric Multiprocessing

Bron-ID: W002004
Namespace: Talk (1)
Revisie: 7601 | Datum: 2009-04-07T10:15:01Z
Bron: https://wiki.osdev.org/Talk:Symmetric_Multiprocessing
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ASymmetric_Multiprocessing&oldid=7601

````text
This page is a CollectedKnowledge about multi-processors systems.
----
Contributors: Brendan
````

<a id="w002005"></a>

## W002005 — Talk:Mouse Input

Bron-ID: W002005
Namespace: Talk (1)
Revisie: 19246 | Datum: 2016-05-18T17:56:53Z
Bron: https://wiki.osdev.org/Talk:Mouse_Input
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMouse_Input&oldid=19246

````text
==Todo==
* Basic Packet Format
* Explain usage of 0x60 and 0x64 port.
* Explain whole mouse driver better. I can't understand since i readed it 571 times. 
* Extended Modes (wheel and five-button mode)
* Cursor display
* Anything else...?

== Reverted ==

Oops, forgot to mention why I had to revert the last couple of changes. It was a direct copypasta from an article that wasn't our own. Copyright is an important issue.

== Did the first two ==

Could still put in a pretty wiki graphic to show the bits in first packet byte. Fourth packet byte is so variable that a graphic seems impractical.

And I didn't cover cursor display at all. That seems like it doesn't belong in a hardware article?
[[User:Bewing|Bewing]] 01:25, 15 November 2007 (CST)

You do not understand him, Bewing. He said that this wiki page could have some graphics bitmap instead of tables for packets. This is good idea because then users would better understand that packet.--[[User:Lukaandjelkovic|Lukaandjelkovic]] 12:55, 18 May 2016 (CDT)

== today ==
add the graphic wiki style ( however it will be good, if someone who can "extend" those bitmaps ( Z movement and those with only 1 ..)

a cursor is a "GUI" thing, however explaning about movement direction would might be a good ider ? 

KMT dk

== Packet Information? ==

At the end of the 'Mouse Packet Info' header, it says:
"The bottom 3 bits of the first byte indicate whether the middle, right, or left mouse buttons are currently being held down, if the respective bit is set. Middle = bit 2 (value=4), right = bit 1 (value=2), left = bit 0 (value=1)."

This indicates that:
  * Bit 0: Left Mouse Button
  * Bit 1: Right Mouse Button
  * Bit 2: Middle Mouse Button

Whilst the table of 'byte 1' says that the Right Mouse Button is bit 0. Is it just me or is there something wrong with the table or the text?

EDIT: I'll change this in a moment, just tested it and bit 0 is left mouse button, bit 1 is right mouse button and bit 2 is the middle mouse button.

-- [[User:Creature|Creature]] 12:51, 2 July 2009 (GMT + 1)
````

<a id="w002006"></a>

## W002006 — Talk:Chip Numbers, Acronyms and Things

Bron-ID: W002006
Namespace: Talk (1)
Revisie: 5874 | Datum: 2008-07-09T21:48:57Z
Bron: https://wiki.osdev.org/Talk:Chip_Numbers%2C_Acronyms_and_Things
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AChip_Numbers%2C_Acronyms_and_Things&oldid=5874

````text
Gawd, I really hate the title of this page. It is so badly named that nothing even links to it -- and it's supposed to contain all of the commonly used definitions and acronyms! I want to put a definition into it, and then link to that definition -- but I can't even do that, because the page does not use equal sign separators, so I can't use a # to locate a particular definition.

I'd still like to take the current [[Definitions]] article, move it into "theory" or someplace, rename that one a bit, and then rename THIS article to "Definitions" -- and then put equal sign separators on each definition, so that they can be linked!
<br>[[User:Bewing|Bewing]] 21:48, 9 July 2008 (UTC)
````

<a id="w002007"></a>

## W002007 — Talk:Definitions

Bron-ID: W002007
Namespace: Talk (1)
Revisie: 3586 | Datum: 2007-04-06T03:16:53Z
Bron: https://wiki.osdev.org/Talk:Definitions
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ADefinitions&oldid=3586

````text
Please see my comment on this thread: http://www.osdev.org/wiki/Category_talk:Definitions
[[User:Bewing|Bewing]] 22:16, 5 April 2007 (CDT)
````

<a id="w002008"></a>

## W002008 — Talk:Text Mode Cursor

Bron-ID: W002008
Namespace: Talk (1)
Revisie: 20594 | Datum: 2017-03-19T06:24:33Z
Bron: https://wiki.osdev.org/Talk:Text_Mode_Cursor
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AText_Mode_Cursor&oldid=20594

````text
I retagged this as needing review. It needs '''at least''' a new title first. -[[User:Jhawthorn|Jhawthorn]] 00:11, 6 April 2007 (CDT)
: How about [[Text Mode Cursor]]? - [[User:Combuster|Combuster]] 02:53, 6 April 2007 (CDT)
:: Sounds good to me. Although, I think I'm slightly confused. Why "'''at least'''" a new title? What else is wrong with it? --Alboin
::: I see some minor things wrong: unnecessary capitalization ("ASSUMES"), using "h" suffix rather than "0x" prefix specified in [[User:Combuster/Manual Of Style draft]]. You know, things like that. -[[User:Jhawthorn|Jhawthorn]] 18:15, 6 April 2007 (CDT)
::: Made said changes. '''I''' no longer oppose the tag being removed. -[[User:Jhawthorn|Jhawthorn]] 18:21, 6 April 2007 (CDT)
:::: Moved to Video Category. --Alboin

The "Source in C" section uses DOS-style "outb", with the port first.  This seems inappropriate and unnecessarily confusing, as most of the wiki (and I assume the OSDev community) seems geared towards Unix-like operating systems.  Does anyone else agree?
````

<a id="w002011"></a>

## W002011 — Talk:OSRC

Bron-ID: W002011
Namespace: Talk (1)
Revisie: 12065 | Datum: 2011-10-03T12:27:18Z
Bron: https://wiki.osdev.org/Talk:OSRC
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AOSRC&oldid=12065

````text
Why do we have this page? Wouldn't it be enough just to put a link in quicklinks or something? --Alboin
:Comment removed due to copyright - [[User:Combuster|Combuster]] 12:27, 3 October 2011 (UTC)

==Merger==
This page was slated for merger. Here: [[Talk:Resources]]. Should it be moved back into PFR? (I'm thinking it doesn't really matter, but nevertheless. I thought I'd make note of it just in case someone was to revamp it.) --Alboin
:Sorry, didn't see that. I support the mergers. [[User:Yayyak|Yayyak]] 00:46, 31 August 2007 (CDT)
````

<a id="w002013"></a>

## W002013 — Talk:Quick Links

Bron-ID: W002013
Namespace: Talk (1)
Revisie: 4269 | Datum: 2007-07-10T17:33:25Z
Bron: https://wiki.osdev.org/Talk:Quick_Links
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AQuick_Links&oldid=4269

````text
Can we remove this page? I moved all the information to [[Specifications]] on the main page. (Most of it, that is. There were some dead links, and I removed all of the forum links. (How does one find those on the new forum?)) --Alboin

I think a wider discussion is needed as there are several pages involved: [[Quick Links]] [[Specifications]] [[OSRC]] [[Resources]] and probably more. I think merging the lot into one [[Resources]] page with the appropriate subheaders would be good. Many off-site links are already present in various articles so having several pages dedicated to external links is not good for organisation. Having [[Specifications|specs]] and [[Resources|other resources]] in separate pages is arguable though. - [[User:Combuster|Combuster]] 17:08, 10 April 2007 (CDT)
:[[Resources]] is everything that is not a pure hardware specification, which go in [[Specifications]]. I think there's a clear distinction between the two. I am not sure about merging them... --Albion
I'm liking your idea about just a [[Resources]] page. Being, if someone's looking for information on Video hardware, they can just search for video and go to the links at the bottom of the page. Having a page '''just''' for [[Specifications]] is redundant. I think just a short and sweet [[Resources]] page would do, with non specific abstract links like osdev and OSRC. --Alboin
:Just noticed [[Links]]. --Alboin
:Also [[Kernels With Source]] --Alboin

I do not think Specifications redundant. Having a list with properly documentated hardware helps picking an OS developers' next goal. Since a large merge seems a good thing, I move this discussion to [[Talk:Resources]] where everything should end up. - [[User:Combuster|Combuster]] 12:33, 10 July 2007 (CDT)
````

<a id="w002015"></a>

## W002015 — Category:Graphical UI

Bron-ID: W002015
Namespace: Category (14)
Revisie: 22822 | Datum: 2018-09-10T05:40:40Z
Bron: https://wiki.osdev.org/Category:Graphical_UI
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AGraphical_UI&oldid=22822

````text
[[Category:OS theory]]
[[Category:UI]]
````

<a id="w002016"></a>

## W002016 — User:Quok

Bron-ID: W002016
Namespace: User (2)
Revisie: 6248 | Datum: 2008-08-27T18:30:21Z
Bron: https://wiki.osdev.org/User:Quok
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AQuok&oldid=6248

````text
===My OS Project===

I'm slowly working on QuokOS (temporary name, I'm sure). The goal is to create a POSIX and possibly SUS compliant UNIX-like OS. The one major change I'm planning on is a more sane (that is, modern) filesystem layout, and application directories (like Mac OS). Of course, that does most likely mean I can't meet full POSIX and SUS compliance.

QuokOS is developed in C, using GCC and GAS on all platforms. It is a monolithic kernel, with support for module loading (implemented now) and unloading planned in the future.

Currently not too much is supported, some basic hardware detection, multiple virtual consoles, some SMP code, and that's about it. While there is multitasking code in QuokOS, I don't currently launch any tasks; I'm waiting on implementing my own ATA driver and some filesystem code (Ext2FS).

QuokOS is currently licensed under its own license, named the QuokOS Public License. However, I am considering switching the license to the ISC license.

QuokOS has been tested under Bochs, VMWare, and Qemu. It also runs on real hardware: I've booted it successfully on some IBM Thinkpad laptops, a dual cpu IBM Netfinity 5500, my AMD64 workstation (in 32-bit mode), a macbook pro, and a 16 way x86_64 based server with over 32GB of RAM. Sure, it booted there, but was it usable? I don't know, it's not usable anywhere right now!

QuokOS is currently X86 (32 bit) only. A X86-64 version is planned, and work is slowly starting to progress there.

===Pages In My Namespace===
Here's links to WIP pages in my namespace.

[[User:Quok/Silicon_Image_Datasheets|Silicon Image Datasheets]]
````

<a id="w002017"></a>

## W002017 — User:Alboin

Bron-ID: W002017
Namespace: User (2)
Revisie: 5022 | Datum: 2008-01-02T23:21:40Z
Bron: https://wiki.osdev.org/User:Alboin
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAlboin&oldid=5022

````text
I noticed this page was outdated, so now all I have is this message.

...

I think it works.
````

<a id="w002018"></a>

## W002018 — Task State Segment

Bron-ID: W002018
Namespace: Main (0)
Revisie: 28134 | Datum: 2023-07-10T02:49:00Z
Bron: https://wiki.osdev.org/Task_State_Segment
Revisie-URL: https://wiki.osdev.org/index.php?title=Task_State_Segment&oldid=28134

````text
A '''Task State Segment''' ('''TSS''') is a binary data structure specific to the [[IA32_Architecture_Family|IA-32]] and [[X86-64|x86-64]] architectures. It holds information about a task. In '''[[Protected Mode]]''' the '''TSS''' is primarily suited for '''[[Context Switching#Hardware Context Switching|Hardware Task Switching]]''', where each individual '''Task''' has its own '''TSS'''. For use in [[Context Switching#Software Context Switching|software multitasking]], one or two are also generally used, as they allow for entering '''Ring 0''' code after an interrupt. In '''[[Long Mode]]''', the '''TSS''' has a separate structure and is used to change the '''Stack Pointer''' after an interrupt or permission level change. You'll have to update the '''TSS''' yourself in the multitasking function, as it apparently does not save registers automatically.

== Protected Mode ==

For its use in hardware task switching, a '''TSS''' contains a program's state, including '''[[CPU Registers x86#General Purpose Registers|General Purpose Registers]]''', '''[[Segment Selector|Segment Selectors]]''', the '''[[CPU Registers x86#General Pointer Registers|Instruction Pointer]]''', the '''[[CPU Registers x86#EFLAGS Register|EFLAGS Register]]''' and '''[[CPU Registers x86#CR3|Control Register 3]]'''. It contains certain other fields described below.

{|class="wikitable"
|-
!style="width: 20%;"| 0x3
!style="width: 20%;"| 0x2
!style="width: 20%;"| 0x1
!style="width: 20%;"| 0x0
!style="width: 20%;"| Offset
|-
|colspan=4 style="text-align: center;"| '''SSP'''
!0x68
|-
|colspan=2 style="text-align: center;"| '''IOPB'''
|colspan=2 style="text-align: center;"| Reserved
!0x64
|-
|colspan=2 style="text-align: center;"| Reserved
|colspan=2 style="text-align: center;"| '''LDTR'''
!0x60
|-
|colspan=2 style="text-align: center;"| Reserved
|colspan=2 style="text-align: center;"| '''GS'''
!0x5C
|-
|colspan=2 style="text-align: center;"| Reserved
|colspan=2 style="text-align: center;"| '''FS'''
!0x58
|-
|colspan=2 style="text-align: center;"| Reserved
|colspan=2 style="text-align: center;"| '''DS'''
!0x54
|-
|colspan=2 style="text-align: center;"| Reserved
|colspan=2 style="text-align: center;"| '''SS'''
!0x50
|-
|colspan=2 style="text-align: center;"| Reserved
|colspan=2 style="text-align: center;"| '''CS'''
!0x4C
|-
|colspan=2 style="text-align: center;"| Reserved
|colspan=2 style="text-align: center;"| '''ES'''
!0x48
|-
|colspan=4 style="text-align: center;"| '''EDI'''
!0x44
|-
|colspan=4 style="text-align: center;"| '''ESI'''
!0x40
|-
|colspan=4 style="text-align: center;"| '''EBP'''
!0x3C
|-
|colspan=4 style="text-align: center;"| '''ESP'''
!0x38
|-
|colspan=4 style="text-align: center;"| '''EBX'''
!0x34
|-
|colspan=4 style="text-align: center;"| '''EDX'''
!0x30
|-
|colspan=4 style="text-align: center;"| '''ECX'''
!0x2C
|-
|colspan=4 style="text-align: center;"| '''EAX'''
!0x28
|-
|colspan=4 style="text-align: center;"| '''EFLAGS'''
!0x24
|-
|colspan=4 style="text-align: center;"| '''EIP'''
!0x20
|-
|colspan=4 style="text-align: center;"| '''CR3'''
!0x1C
|-
|colspan=2 style="text-align: center;"| Reserved
|colspan=2 style="text-align: center;"| '''SS2'''
!0x18
|-
|colspan=4 style="text-align: center;"| '''ESP2'''
!0x14
|-
|colspan=2 style="text-align: center;"| Reserved
|colspan=2 style="text-align: center;"| '''SS1'''
!0x10
|-
|colspan=4 style="text-align: center;"| '''ESP1'''
!0x0C
|-
|colspan=2 style="text-align: center;"| Reserved
|colspan=2 style="text-align: center;"| '''SS0'''
!0x08
|-
|colspan=4 style="text-align: center;"| '''ESP0'''
!0x04
|-
|colspan=2 style="text-align: center;"| Reserved
|colspan=2 style="text-align: center;"| '''LINK'''
!0x00
|}

* '''LINK:''' Previous Task Link Field. Contains the '''Segment Selector''' for the '''TSS''' of the previous task.
* '''SS0, SS1, SS2:''' The '''Segment Selectors''' used to load the stack when a privilege level change occurs from a lower privilege level to a higher one.
* '''ESP0, ESP1, ESP2:''' The '''Stack Pointers''' used to load the stack when a privilege level change occurs from a lower privilege level to a higher one.
* '''IOPB:''' I/O Map Base Address Field. Contains a 16-bit offset from the base of the '''TSS''' to the '''I/O Permission Bit Map'''.
* '''SSP:''' Shadow Stack Pointer.

== Long Mode ==

In '''[[Long Mode]]''', the '''TSS''' does not store information on a task's execution state, instead it is used to store the '''Interrupt Stack Table'''.

{|class="wikitable"
|-
!style="width: 20%;"| 0x3
!style="width: 20%;"| 0x2
!style="width: 20%;"| 0x1
!style="width: 20%;"| 0x0
!style="width: 20%;"| Offset
|-
|colspan=2 style="text-align: center;"| '''IOPB'''
|colspan=2 style="text-align: center;"| Reserved
!0x64
|-
|colspan=4 style="text-align: center;"| Reserved
!0x60
|-
|colspan=4 style="text-align: center;"| Reserved
!0x5C
|-
|colspan=4 style="text-align: center;"| '''IST7''' (High)
!0x58
|-
|colspan=4 style="text-align: center;"| '''IST7''' (Low)
!0x54
|-
|colspan=4 style="text-align: center;"| '''IST6''' (High)
!0x50
|-
|colspan=4 style="text-align: center;"| '''IST6''' (Low)
!0x4C
|-
|colspan=4 style="text-align: center;"| '''IST5''' (High)
!0x48
|-
|colspan=4 style="text-align: center;"| '''IST5''' (Low)
!0x44
|-
|colspan=4 style="text-align: center;"| '''IST4''' (High)
!0x40
|-
|colspan=4 style="text-align: center;"| '''IST4''' (Low)
!0x3C
|-
|colspan=4 style="text-align: center;"| '''IST3''' (High)
!0x38
|-
|colspan=4 style="text-align: center;"| '''IST3''' (Low)
!0x34
|-
|colspan=4 style="text-align: center;"| '''IST2''' (High)
!0x30
|-
|colspan=4 style="text-align: center;"| '''IST2''' (Low)
!0x2C
|-
|colspan=4 style="text-align: center;"| '''IST1''' (High)
!0x28
|-
|colspan=4 style="text-align: center;"| '''IST1''' (Low)
!0x24
|-
|colspan=4 style="text-align: center;"| Reserved
!0x20
|-
|colspan=4 style="text-align: center;"| Reserved
!0x1C
|-
|colspan=4 style="text-align: center;"| '''RSP2''' (High)
!0x18
|-
|colspan=4 style="text-align: center;"| '''RSP2''' (Low)
!0x14
|-
|colspan=4 style="text-align: center;"| '''RSP1''' (High)
!0x10
|-
|colspan=4 style="text-align: center;"| '''RSP1''' (Low)
!0x0C
|-
|colspan=4 style="text-align: center;"| '''RSP0''' (High)
!0x08
|-
|colspan=4 style="text-align: center;"| '''RSP0''' (Low)
!0x04
|-
|colspan=4 style="text-align: center;"| Reserved
!0x00
|}

* '''RSP0, RSP1, RSP2:''' The '''Stack Pointers''' used to load the stack when a privilege level change occurs from a lower privilege level to a higher one.
* '''IST#:''' Interrupt Stack Table. The '''Stack Pointers''' used to load the stack when an entry in the '''[[Interrupt Descriptor Table#Long Mode|Interrupt Descriptor Table]]''' has an '''IST''' value other than 0.
* '''IOPB:''' I/O Map Base Address Field. Contains a 16-bit offset from the base of the '''TSS''' to the '''I/O Permission Bit Map'''.

== TSS in software multitasking ==

For each CPU which executes processes possibly wanting to do system calls via interrupts, one TSS is required. The only interesting fields are SS0 and ESP0. Whenever a system call occurs, the CPU gets the SS0 and ESP0-value in its TSS and assigns the stack-pointer to it. So one or more kernel-stacks need to be set up for processes doing system calls. Be aware that a thread's/process' time-slice may end during a system call, passing control to another thread/process which may as well perform a system call, ending up in the same stack. Solutions are to create a private kernel-stack for each thread/process and re-assign esp0 at any task-switch or to disable scheduling during a system-call (see also [[Kernel Multitasking]]).

Setting up a TSS is straight-forward. An entry in the [[Global Descriptor Table]] is needed (see also the [[GDT_Tutorial#Flat_Setup|GDT Tutorial]]), specifying the TSS' address as "base", TSS' size as "limit", 0x89 (Present|Executable|Accessed) as "access byte" and 0x40 (Size-bit) as "flags".
In the TSS itself, the members "SS0", "ESP0" and "IOPB offset" are to be set:
* SS0 gets the kernel datasegment descriptor (e.g. 0x10 if the third entry in your GDT describes your kernel's data)
* ESP0 gets the value the stack-pointer shall get at a system call
* IOPB may get the value sizeof(TSS) (which is 104) if you don't plan to use this io-bitmap further (according to mystran in http://forum.osdev.org/viewtopic.php?t=13678)

The actual loading of the TSS must take place in protected mode and after the GDT has been loaded. The loading is simple as:
<source lang="asm">
mov ax, 0x??  ;The descriptor of the TSS in the GDT (e.g. 0x28 if the sixths entry in your GDT describes your TSS)
ltr ax        ;The actual load
</source>

== See Also ==
=== Articles ===
*[[GDT_Tutorial#Flat_Setup|GDT Tutorial]]
*[[System Calls]]
*[[Getting to Ring 3]]
=== Forum Threads ===
*[[Topic:13678|Do I need a TSS?]]
=== External Links ===
*[[wikipedia:Task State Segment|Task State Segment]] on Wikipedia

[[Category:X86 CPU]]
[[Category:Multitasking]]
[[de:Task State Segment]]
````

<a id="w002019"></a>

## W002019 — Talk:Getting Started

Bron-ID: W002019
Namespace: Talk (1)
Revisie: 13497 | Datum: 2012-06-04T10:56:57Z
Bron: https://wiki.osdev.org/Talk:Getting_Started
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AGetting_Started&oldid=13497

````text
==Idle comment==
Suggestion removed due to copyright claims - [[User:Combuster|Combuster]] 12:32, 3 October 2011 (UTC)
:You mean create a [[Toolsets]] page and discuss the options there, while generalizing [[Getting Started]]? Sounds good to me. --Alboin
::At one side, that is politically correct. However, there is by my knowledge no substitute being as complete, powerful, tried, and universally available as gcc/binutils. I think we can fairly say that a different C compiler is "use at your own risk". I don't expect people to figure out the issues with FreeBasic as osdev targeted compiler. I think we should look for a middle road - [[User:Combuster|Combuster]] 12:38, 12 April 2007 (CDT)

==Merging in [[Prerequisites]]==
It seems to me that the above page could become a section within this page, in a sort of 'mental requirements' section, like the '''Getting motivated''' bit we already have here. [[User:Yayyak|Yayyak]] 06:21, 17 August 2007 (CDT)
If nobody has any objections, I'll do it tomorrow (Sept 3 2007). [[User:Yayyak|Yayyak]] 21:40, 1 September 2007 (CDT)

== GNU slant ==

This page is misleading (as many on this site are) slanted toward a disadvantage of Windows users while suggesting a requirement of GNU tools for OSDEV.  This is accurate for cross architecture development but not x86 development where 99% of Windows developers will be working.  Most any Windows based native compiler can be used for OSDEV, compiling is not the problem, linking is the difficulty.  I've ported my OS for compilation with Borland tools, MSVC and Watcom.  Open Watcom's wlink is the best linker I've encountered for Windows.  It's not a cross architecture compiler but does produce flat binaries, ELF, PE, MZ, Phar Lap, etc and will link the output from nearly any Windows based compiler.  JLOC is another option but it doesn't like borland's OMF format. 

''This unsigned comment was made by [[User:Tantrikwizard]]; please remember to sign your posts using four tildes:'' <tt><nowiki>~~~~</nowiki></tt>

:Having a bias towards GNU tools does not automatically mean that it is misleading. Next to that, there are good reasons why that bias actually exists - Visual Studio ''alone'' has alwas had issues with generating the necessary formats to make a bootable image - there were always third party tools needed to make it actually work. As for watcom, it has about the smallest userbase of the common C compilers so if there are problems with it, you won't be able to readily expect answers about that compiler. If you read the article you'll notice that it is explained where that bias comes from. If you really feel that that should change, you have a reputation to make for the compiler suite or combination of tools in question as being better than GNU on Windows. Just claiming something's bad doesn't help when there's no good alternative.
:- [[User:Combuster|Combuster]] 16:18, 26 November 2008 (UTC)
::The other thing: because most people know GCC, they can answer newbie's questions about gcc when they come to the forum with error messages. There are a number of cases where people come to the forum with "I use my friend's dad's programmer friend's linker, and I have this error message. What does it mean? What do I need to do?". There are a lot less ums and ahs when they ask about gcc. If you're going to be supporting something, recommend the something you know about.
:: [[User:Jackscott|Jackscott]] 12:49, 27 November 2008 (UTC)
:::The reasons for a certain bias ''pro'' GNU toolchain are the same as for using C/C++, Intel syntax ASM (sadly, IMHO), or for this Wiki being in English: It certainly improves your chances of getting help (''and'' makes life easier for the OSDev citizenry).
:::That's what is being said at the beginning of the chapter, too - "The GNU toolset is generally recognised as the best for OSDev".
:::That being said, I don't see a problem with the article as it is now. -- [[User:Solar|Solar]] 14:22, 27 November 2008 (UTC)

== Common starting points ==

The section says, "Some of the best introductory material is included in the tutorial section at the bottom of the page." However, there is no "tutorial section" at the bottom of the page. Should this contain a summary of some material from [[Tutorials]]? I find it confusing. [[User:Sidewaysmilk|Sidewaysmilk]] 04:08, 27 June 2011 (UTC)

:Done. (Why did this remain unaddressed for so long?) -- [[User:Solar|Solar]] 05:56, 4 June 2012 (CDT)
````

<a id="w002020"></a>

## W002020 — File:Tree based.png

Bron-ID: W002020
Namespace: File (6)
Revisie: 3662 | Datum: 2007-04-12T07:42:16Z
Bron: https://wiki.osdev.org/File:Tree_based.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ATree_based.png&oldid=3662

````text

````

<a id="w002021"></a>

## W002021 — Talk:Page Frame Allocation

Bron-ID: W002021
Namespace: Talk (1)
Revisie: 12831 | Datum: 2012-02-25T11:58:30Z
Bron: https://wiki.osdev.org/Talk:Page_Frame_Allocation
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APage_Frame_Allocation&oldid=12831

````text
==Bitmap==
:-- any pointer to a PD reference implementation is welcome /PypeClicker

==Stack/List of pages==
I removed this line
 Note that when all the memory is free, up to N*4 bytes are required to store the states.
Lated I (or someone else) should explain why this isn't always true. -[[User:Jhawthorn|Jhawthorn]] 10:45, 12 April 2007 (CDT)

==Page frame/Page pool confusion?==
Umm, if I recall my theory correctly, page frames are contiguous slices of the virtual address space, whereas a page pool is the allocation mechanism to provide physical page addresses to map in to a page frame, no? The page seems a little confused about the distinction .... --[[User:Childoftechno|Childoftechno]] 02:38, 25 February 2012 (CST)

[[wikipedia:Page_table|Wikipedia]] disagrees with you.
I have never heard of the term page pool before you mentioned it, and a google search doesn't show anything except the windows Paged/Unpaged memory pools.

[[User:Thepowersgang|Thepowersgang]] 05:58, 25 February 2012 (CST)
````

<a id="w002022"></a>

## W002022 — File:Flat list.png

Bron-ID: W002022
Namespace: File (6)
Revisie: 3669 | Datum: 2007-04-12T18:09:22Z
Bron: https://wiki.osdev.org/File:Flat_list.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AFlat_list.png&oldid=3669

````text
based off of this diagram:

<pre>
                                   address space
                        _______      ______
  Start--------------->|       |    |  (A) |
  Size--------------\  | free  | -> | free |
  Block_before       \ | space |    |------|
,-Block_after         \|_______|    |\\\\\\|
|                                   |\used\|
|                       _______     |\\\\\\|
| Start--------------->|       |    |------|
| Size--------------\  | free  |    |   (B)|
'-Block_before       \ | space | -> | free |
,-Block_after         \|_______|    |      |
|                                   |------|
|                       _______     |\used\|
| Start--------------->|       |    |---(C)|
| Size--------------\  | free  | -> | free |
'-Block_before       \ | space |    |------|
,-Block_after         \|_______|    |\\\\\\|
|                                   |\used\|
|                       _______     |\\\\\\|
| Start--------------->|       |    |------|
| Size--------------\  | free  |    |   (D)|
'-Block_before       \ | space | -> | free |
  Block_after         \|_______|    |______|
</pre>
````

<a id="w002023"></a>

## W002023 — File:Sized Portion Scheme.png

Bron-ID: W002023
Namespace: File (6)
Revisie: 3672 | Datum: 2007-04-12T18:25:27Z
Bron: https://wiki.osdev.org/File:Sized_Portion_Scheme.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ASized_Portion_Scheme.png&oldid=3672

````text
based off of this diagram:
 <---------------------------------------->
 |8kb....|4kb|4kb||8kb....|4kb|4kb|
 <---------------------------------------->
````

<a id="w002024"></a>

## W002024 — Manual Of Style

Bron-ID: W002024
Namespace: Main (0)
Revisie: 6601 | Datum: 2008-11-04T19:25:30Z
Bron: https://wiki.osdev.org/Manual_Of_Style
Revisie-URL: https://wiki.osdev.org/index.php?title=Manual_Of_Style&oldid=6601
Doorverwijzing: OSDev Wiki:Manual Of Style

````text
#REDIRECT [[OSDev Wiki:Manual Of Style]]
````

<a id="w002025"></a>

## W002025 — Talk:Interrupt Descriptor Table

Bron-ID: W002025
Namespace: Talk (1)
Revisie: 29701 | Datum: 2025-09-29T21:52:02Z
Bron: https://wiki.osdev.org/Talk:Interrupt_Descriptor_Table
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AInterrupt_Descriptor_Table&oldid=29701

````text
==Old content==
This used to be on the page. I replaced it with a C struct. It should be considered for re-inclusion at a later date.
 Byte:
        +---------------+---------------+---------------+---------------+
    0   |  Offset 0:7   |  Offset 8:15  | Selector 0:7  | Selector 7:15 |
        +---------------+---------------+---------------+---------------+
 
        +---------------+-----+---------+---------------+---------------+
    4   |0 0 0 0 0 0 0 0|Flags|GateType | Offset 16:23  | Offset 24:31  |
        +---------------+-----+---------+---------------+---------------+
-[[User:Jhawthorn|Jhawthorn]] 17:11, 21 April 2007 (CDT)


==Merge==
Since I copied the last details from [[IDT2]], I think there are no reasons to keep the WIP and merge tags. Anybody having any objections? - [[User:Combuster|Combuster]] 05:41, 14 November 2007 (CST)

==64bit==
How are we doing 64bit info with articles like this? I'm reading up on initializing 64bit and such, and thought I might fill out some. --[[User:Alboin|Alboin]] 20:32, 23 January 2008 (CST)

Personally I'd put it on a separate page and add links to navigate from the one to the other - [[User:Combuster|Combuster]] 01:45, 24 January 2008 (CST)

== Crappy Quality ==

Looking through this article, this appears to be nothing more than a massive pile of randomly sawn-together pieces of information. This needs some serious reordering, or even better, rework. I might look into doing this, however I can and will not guarantee this. -- [[User:No92|no92]] 10:51, 10 May 2017 (CDT)

== What is the Storage Section field ==

I looked into the latest IA32 System Programming Guide and the old 80386 programmer manual, none mention the Section Storage field. In both manuals, that field is just set to 0 and sort of "part" of the gate type field.

Where did that name come from? Was it just made up?

== *should* exceptions be trap gates?? ==

The page advises exceptions should be trap gates, but the reasoning isn't fully explained. I'm not convinced that having hardware interrupts fire in the middle of handling an exception is actually desired behaviour

Per my own experiments (limited to QEMU), and unless I'm very much mistaken, on an x86 clearing the interrupt flag does NOT prevent exceptions from firing - only IRQs. (This means the parts about "disabling interrupts" are also slightly misleading) As such I see no reason why exceptions should be implemented as trap gates - if anything it should be the other way around, with IRQ handlers set as trap gates in order that the PIC can interrupt them in order of priority... [[User:Invalidopcode|Invalidopcode]] ([[User talk:Invalidopcode|talk]]) 14:30, 14 September 2025 (UTC)

: Hi [[User:Invalidopcode|Invalidopcode]], just to note that you'd posted the same question 3 times. I presume this was because they were awaiting moderation, so I've removed the duplicates. --[[User:Bellezzasolo|Bellezzasolo]] ([[User talk:Bellezzasolo|talk]]) 07:42, 15 September 2025 (UTC)

Thanks Bellezzasolo, the error message was quite vague and I had no idea I was multiple posting! [[User:Invalidopcode|Invalidopcode]] ([[User talk:Invalidopcode|talk]]) 21:52, 29 September 2025 (UTC)
````

<a id="w002026"></a>

## W002026 — User:Ldenyer/Memory Management First Fit

Bron-ID: W002026
Namespace: User (2)
Revisie: 10063 | Datum: 2010-04-16T10:56:31Z
Bron: https://wiki.osdev.org/User:Ldenyer/Memory_Management_First_Fit
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALdenyer%2FMemory_Management_First_Fit&oldid=10063

````text
'''This page will have a memory management tutorial for the first fit algorithm soon!'''
````

<a id="w002027"></a>

## W002027 — User:Bernhard

Bron-ID: W002027
Namespace: User (2)
Revisie: 5076 | Datum: 2008-02-04T08:36:57Z
Bron: https://wiki.osdev.org/User:Bernhard
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ABernhard&oldid=5076

````text
My Name is Bernhard Treutwein and I am around with computers since the late 1970ies. I started programming with around 1974 on a TR-440. This was a german host computer running a german dialogue-oriented operating system, which went a radically different way than the other OS which had its roots at a similar time: *ix i.e. Unix, Irix, Aix, etc. From there I came the long way via several CDC host systems over to PDP11s under RT11 and then further down to PCs (MSDos, DRDos) and then up again to Unix workstations (Sparc Solaris), and finally via OS/2 to Linux. I'm switching freely between Windows (mainly XP, but sometimes also 9x), Linux (mainly Debian/Ubuntu, but sometimes also Suse), and Native Oberon (sometimes experimenting with BlueBottle/Aos).

Having learned Algol-60 as my first programming language, I was sticky in the sense that I am still in the Algol-branch of the programming-language tree, i.e. my favorite languages are currently Component Pascal [http://www.oberon.ch/blackbox.html] and Oberon-2 [http://www.oberon.ethz.ch/].
````

<a id="w002028"></a>

## W002028 — Talk:Pascal

Bron-ID: W002028
Namespace: Talk (1)
Revisie: 3719 | Datum: 2007-04-26T16:37:20Z
Bron: https://wiki.osdev.org/Talk:Pascal
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APascal&oldid=3719

````text
== Topic split ==
Would it be cleaner to fork off the [[:Category:Bare bones tutorials|Barebones]] to an separate article? - [[User:Combuster|Combuster]] 11:37, 26 April 2007 (CDT)
````

<a id="w002029"></a>

## W002029 — OSDev Wiki:Privacy policy

Bron-ID: W002029
Namespace: OSDev Wiki (4)
Revisie: 16682 | Datum: 2014-09-04T02:51:13Z
Bron: https://wiki.osdev.org/OSDev_Wiki:Privacy_policy
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDev_Wiki%3APrivacy_policy&oldid=16682

````text
=Privacy Policy=
# Where possible OSDev.org will attempt to respect your privacy. 
# No information collected will be shared with third parties. This includes such data as server logs and the information not publicly shared by authors and editors. 
# OSDev.org is located in the United States, specifically in Houston, TX, and may be required to comply with legal requests to identify people if they commit illegal activities on this site. 
# Please, no copyright infringement, virus writing, OS exploiting, or links to those types of activities. 
# Please do not add your private information unless you are sure you want it shared as deleting content in the wiki does not permanently remove it. 
# Do not post other people's private information.

[[Category:Wiki administration|{{PAGENAME}}]]
````

<a id="w002030"></a>

## W002030 — OSDev Wiki:General disclaimer

Bron-ID: W002030
Namespace: OSDev Wiki (4)
Revisie: 6586 | Datum: 2008-11-04T15:33:26Z
Bron: https://wiki.osdev.org/OSDev_Wiki:General_disclaimer
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDev_Wiki%3AGeneral_disclaimer&oldid=6586

````text
=No warranty of any kind=

OSDev.org makes no guarantee of validity of the information contained or copied from this site. This site contains low level source code and information on hardware devices which may be used in the creation of software. The information contained here is for entertainment purposes and should be verified by yourself as accurate before use. Any software created by the information contained here or software located at this site has the following disclaimer:

<pre>
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
</pre>

[[Category:Wiki administration|{{PAGENAME}}]]
````

<a id="w002031"></a>

## W002031 — OSDev Wiki:License

Bron-ID: W002031
Namespace: OSDev Wiki (4)
Revisie: 11735 | Datum: 2011-07-07T21:04:05Z
Bron: https://wiki.osdev.org/OSDev_Wiki:License
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDev_Wiki%3ALicense&oldid=11735

````text
=Wiki License=
As from 03-07-2011, all new content has to be strictly compatible with [[OSDev Wiki:Copyrights|Public Domain licensing]]. All authors of existing material will be attempted to be informed of the new official license declaration. Any author not in agreement of the new license(s) can have their content removed or choose to dual license their work. This does not apply to forum content in which all posts are owned by the author.

That means that currently '''the contents of this wiki is of mixed licensing'''.

[[Category:Wiki administration|{{PAGENAME}}]]
````

<a id="w002033"></a>

## W002033 — File:ISBN 0201544288.jpg

Bron-ID: W002033
Namespace: File (6)
Revisie: 13542 | Datum: 2012-06-13T03:09:52Z
Bron: https://wiki.osdev.org/File:ISBN_0201544288.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0201544288.jpg&oldid=13542

````text
[[Category:Book Covers]]
````

<a id="w002034"></a>

## W002034 — Volatile (keyword)

Bron-ID: W002034
Namespace: Main (0)
Revisie: 15381 | Datum: 2013-11-04T19:05:12Z
Bron: https://wiki.osdev.org/Volatile_(keyword)
Revisie-URL: https://wiki.osdev.org/index.php?title=Volatile_%28keyword%29&oldid=15381

````text
The <code>volatile</code> keyword gives an indication to the compiler/optimizer that it should always perform a read or write to a variable or memory without caching it locally. It also ensures that the order of changes is preserved and not altered by the optimizer, and that apparently redundant code is not optimized away. The keyword however doesn't guarantee that memory access isn't reordered by the processor on runtime. To fix possible issues due to this, make sure you use [[atomic operation]]s or [[memory barriers]] to manipulate memory.

== The basics ==

The <code>volatile</code> keyword can be used in type definitions as well as function arguments, variable definitions and typecasts. Similar to the <code>const</code> keyword, it either makes the variable itself or the data it points to <code>volatile</code>, or both.

<source lang="c">
/* Ensures the changes to/reads from x are always performed */
volatile int x;

/* Ensures that changes to/reads from the data that ptr points to
   (but not the value itself) are always performed */
volatile int *ptr;
int volatile *ptr;

/* Ensures that changes to/reads from the pointer ptr (but not the
   data it points to) are always performed */
int * volatile ptr;

/* Ensures that changes to/reads from the pointer ptr and also
   the data it points to are always performed */
volatile int * volatile ptr;
int volatile * volatile ptr;
</source>

== Examples ==

=== Loops ===

This is an example of a function that is supposed to create a very short delay, but might be optimized away by the compiler entirely because to the optimizer this code seems redundant:

<source lang="c">
static void some_delay(void)
{
    int i;
    /* Should be declared as '''volatile int i;''' to prevent
       the loop or even the entire function from being optimized
       away */
    
    for (i = 0; i < 100; i++)
    {
        /* Do nothing, just wait a little while so that
           some hardware can respond. Using this technique
           is rarely neccessary and should be avoided! */
    }
}
</source>

This example is supposed to poll a byte until it is <code>0</code> (e.g. waiting for a [[Spinlock]] to be released). Obviously another thread is supposed to change that memory, but since the compiler has no clue about this, we need to ensure that the code isn't optimized away in any case:

<source lang="c">
typedef unsigned char * spin_lock;
/* Should be defined as '''typedef volatile unsigned char * spin_lock;'''
   to prevent reading/writing to spin_lock variables being cached
   in CPU registers! */

static void poll_spinlock(spin_lock lock)
{
    while (*lock != 0)
    {
        /* Do nothing, just poll... */
    }
}

/* Alternatively, if spin_lock wasn't defined as a pointer to volatile data
   you can use it in a typecast instead: */
static void poll_spinlock(spin_lock lock)
{
    while (*(volatile unsigned char *)lock != 0)
    {
        /* Do nothing, just poll... */
    }
}
</source>

=== Dereferencing memory ===

This is an example of code that is supposed to touch a piece of memory (e.g. to make it become resident or to check if it's valid and doesn't raise a page fault):

<source lang="c">
static void touch_mem(void *ptr)
{
    char *data = (char *)ptr;
    /* Should be declared as '''volatile char *data = (volatile char *)ptr;'''
       to prevent the code from being optimized away */
    
    /* It is assumed that the memory is not used by other threads at the same time */
    *data = *data;
}
</source>

Also see the [[APIC#IO_APIC_Configuration|APIC example code]] where the <code>volatile</code> keyword is crucial (reading/writing to hardware registers).

=== Memory shared by multiple threads ===

Suppose you have one thread "A" that loops until another thread "B" wants thread "A" to terminate gracefully. They share a structure with a field "terminate" that thread "A" polls to see if another thread wants it to terminate. Unless that field or the entire structure is declared <code>volatile</code>, there's no guarantee that the code works as expected because the compiler/optimizer has no clue about that field being touched by someone else while looping. So it could generate code that reads the value once and caches it, resulting in an infinite loop. Use <code>volatile</code> to prevent wrong code being generated:

<source lang="c">
typedef struct {
    int terminate; /* Should be volatile int terminate; */
} shared_thread_data;

/* This is the code run by thread "B" to get thread "A" to terminate gracefully */
void stop_worker_thread(shared_thread_data *thread)
{
    thread->terminate = 1;
    /* Techni<cally incorrect, you should use a function that guarantees
       an atomic operation (e.g. lock xchg)
}

/* This is the thread "A" loop */
void worker_thread_body(shared_thread_data *thread)
{
    while (thread->terminate == 0)
    {
        /* Do some work here... */
    }
}
</source>

== External Links ==
[http://www.mjmwired.net/kernel/Documentation/volatile-considered-harmful.txt Volatile considered harmful]

[[Category:C]]
[[Category:C++]]
````

<a id="w002035"></a>

## W002035 — Talk:How Do I Determine The Amount Of RAM

Bron-ID: W002035
Namespace: Talk (1)
Revisie: 5466 | Datum: 2008-05-17T20:35:39Z
Bron: https://wiki.osdev.org/Talk:How_Do_I_Determine_The_Amount_Of_RAM
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AHow_Do_I_Determine_The_Amount_Of_RAM&oldid=5466

````text
== OK for rewrite? ==
There are several typos on this page. Part of it is copied verbatim from another website. The formatting of many comments is bad. The pieces that are lifted directly out of RBIL are not as helpful as they could be.

The article belongs in Hardware/Platforms/X86 -- not in "Common Devices". It should be in two pieces. One piece would be an X86 Memory Map, including the BDA and EBDA. The other would be what remains of this current article -- I think it should be renamed "Detecting Memory (x86)". With extensions about CMOS, and including some of Brendan's commentary on the subject over the years.

Do I hear objections about me rewriting it?
[[User:Bewing|Bewing]] 08:56, 19 April 2008 (CDT)

I agree completely. As with anything, feel free as long as no information is lost. Just beware that this title is an entry in the FAQ category, and should stay that way (don't delete or move this page!). Instead, please replace it with the following snippet once you've completed the new article:
<pre><nowiki>
#REDIRECT [[Your New Page]]
[[Category:FAQ]]</nowiki></pre>
- [[User:Combuster|Combuster]] 16:44, 20 April 2008 (CDT)


OK, done.<br>
[[User:Bewing|Bewing]] 15:35, 17 May 2008 (CDT)

== Volatile or ASM clobber ==
Unedited 'request' from [[User:TomTom|TomTom]]: use of volatile keyword instead of asm("":::"memory"). Two reasons: it was marked as minor while it was a serious change in code, and IMO it didnt improve understandability. Apart from that, the algorithm is fundamentally flawed as it can't cope with residual bus voltages. There's a thread regarding good checking of memory whose contents would provide for a better way of checking memory - we should create a new algorithm based on the concepts there. - [[User:Combuster|Combuster]] 15:53, 9 May 2007 (CDT)
````

<a id="w002036"></a>

## W002036 — Talk:Volatile (keyword)

Bron-ID: W002036
Namespace: Talk (1)
Revisie: 7053 | Datum: 2009-01-12T15:06:56Z
Bron: https://wiki.osdev.org/Talk:Volatile_(keyword)
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AVolatile_%28keyword%29&oldid=7053

````text
Unless we are planning to document the whole of the C language this really is over the top. There is nothing special to this keyword that makes it more useful than others in OS Development... and just because you found one code listing where you think volatile was neccesary does not mean you should be setting of on a mission to teach the world about volatile. -- [[User:Tyler|Tyler]] 16:58, 9 May 2007 (CDT)

:Agreed, but I do think it has some worth. What about creating a Programming Section for articles like this? --Alboin

:Reading the logs, it was more than just ''one'' code snippet. Although I think that mentioning it is a good thing, dedicating an entire page to it is over the top. Mentioning the issue on the [[C]] and [[C PlusPlus|C++]] pages should IMO suffice.

: The Linux kernel documentation devotes an entire text file to volatile. See [http://www.mjmwired.net/kernel/Documentation/volatile-considered-harmful.txt]. Linus generally considers declaring data as 'volatile' to be a bad thing, particularly for shared data. IMO the arguments made in the document are sound... perhaps its worth talking about them on this page?

:To be honest, I don't consider the Linux kernel maintainers to be the last instance in good programming. I also don't like blanket statements like "volatile considered harmful" (erm... then again, I based the [[Makefile]] tutorial on a paper "recursive make considered harmful", so that's a bit of a double standard there... :-D ). There are indeed some points made in the text that should be massaged into this article, but I'd also note that it refers to the ''Linux kernel sources'', where support code for mutexes, memory I/O accessors etc. is already in place. -- [[User:Solar|Solar]] 07:57, 12 January 2009 (UTC)

:The bottom line of that document is not the blanket "don't use volatile" but it rather reads "Don't use volatile because too many of you are too stupid to apply it correctly". And it is that underlying reason that should IMO be addressed. That partly includes the examples of when and when not, but primarily about what volatile itself is about so that people can make their own educated decisions instead of randomly being told that it is bad. - [[User:Combuster|Combuster]] 11:23, 12 January 2009 (UTC)

Whether you like it or not, the point of this discussion is teaching about what this statement does and when it may be neccessary. I wouldn't be surprised if 95% of all hobby OS's work fine as long as they are compiled without optimizations, but as soon as they try to compile with e.g. -O3 they wonder why nothing works anymore. There's instances and uses where volatile definitely is neccessary (especially I/O related code), and also highly optimized synchronization algorithms typically use them as well. Just because Linux is trying to avoid it doesn't mean it's incorrect or wrong. It is a very powerful keyword if the programmer knows when and how to use it properly. My article intends to provide the basic education. I personally do not think the Linux kernel is the best example of good source code, unless you're trying to write a POSIX compliant operating system. I agree with some points of that article against the use of volatile, but like I said there's many occasions where it definitely is important. And yes, "volatile" does not mean a write/read operation becomes atomic!
````

<a id="w002037"></a>

## W002037 — Hardware Abstraction Layer

Bron-ID: W002037
Namespace: Main (0)
Revisie: 29462 | Datum: 2025-04-07T15:58:17Z
Bron: https://wiki.osdev.org/Hardware_Abstraction_Layer
Revisie-URL: https://wiki.osdev.org/index.php?title=Hardware_Abstraction_Layer&oldid=29462

````text
{{Stub}}

== Hardware Abstraction Layer ==

A "Hardware Abstraction Layer" or HAL is an abstraction of the hardware, so as to present a consistent interface to software even if the underlying hardware changes or different models of the same device vary in their implementation and interface. A HAL allows programmers to write device drivers in a consistent and largely model or brand agnostic manner, it might isolate the kernel from much of the hardware in the system. Operating systems which make use of a HAL will rarely, permit user-level software to interface directly with hardware devices and usually will not allow device drivers (which frequently run with privileges) to do so either. [[Device Driver Interfaces]] are one of the most important compoments in a HAL.

[[de:Hardware Abstraction Layer]]
[[Category:Kernel]]
````

<a id="w002038"></a>

## W002038 — Talk:Hardware Abstraction Layer

Bron-ID: W002038
Namespace: Talk (1)
Revisie: 3910 | Datum: 2007-05-19T21:04:05Z
Bron: https://wiki.osdev.org/Talk:Hardware_Abstraction_Layer
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AHardware_Abstraction_Layer&oldid=3910

````text
...o...k... why has there to be pages in russian while the vast majority here speaks english? - [[User:Combuster|Combuster]] 02:52, 14 May 2007 (CDT)
:p.s. the author created a [[Topic:13962|thread]] discussing the idea. IMO it was a bit premature to contribute before the request has gotten replies. Anyway the forum thread suggests deletion or move to user space, I'd go for the latter to preserve the content until a translation has been made. Comments? - [[User:Combuster|Combuster]] 16:59, 14 May 2007 (CDT)
````

<a id="w002039"></a>

## W002039 — Chat

Bron-ID: W002039
Namespace: Main (0)
Revisie: 30127 | Datum: 2026-03-04T19:15:53Z
Bron: https://wiki.osdev.org/Chat
Revisie-URL: https://wiki.osdev.org/index.php?title=Chat&oldid=30127

````text
<center>
{|style="border: 1px solid #cfcfbf; margin-top: 25px; margin-bottom: 25px; background-color: #f0f0ff; text-align: center;"
|
This page is protected to prevent malicious edits to Discord invite links or IRC channel names. If you would like to propose an additional chat resource, please contact one of the wiki administrators.
|
|}
</center>

==Discord==

There is an active and welcoming OS development server on Discord with thousands of members. 

Invite link: https://discord.gg/RnCtsqD

They also have alternate platforms:

Matrix space: [https://matrix.to/#/#osdev:matrix.org #osdev:matrix.org]

Fluxer: https://fluxer.gg/eFPqIzIs

==IRC==

There is a well-populated OS Development IRC channel on Libera (irc.libera.chat) at '''#osdev'''. Several regular contributors to the OSDev.org wiki and forums are usually present in the channel, and the channel has over twenty years of history from its previous iteration on Freenode - it even predates this wiki by several years!

* Historical logs and channel statistics: http://osdev-logs.qzx.com
* Today's log: https://libera.irclog.whitequark.org/osdev

===IRC client suggestions===

Make your own! IRC is one of the simpler protocols to implement, to the point where it could be the first thing you implement after TCP and Telnet. If there's anything tricky about it, it's calculating the maximum length of the actual text, but you can just use a fixed maximum which allows room for all the other fields at their maximum lengths. Or just ignore it, Freenode seems to handle long messages well. In any case, the standard is [https://tools.ietf.org/html/rfc1459 RFC 1459]

Freenode offers an [http://webchat.freenode.net/ AJAX IRC web client] in case you don't have or want to make one.

Other clients:
* [https://tools.suckless.org/ii/ ii] is a file- and FIFO-based IRC client for POSIX systems in under 1000 lines of source code
* [https://irssi.org/ IRSSI] is a popular and powerful text-mode IRC client.
* [https://pidgin.im/ Pidgin] supports a vast [https://pidgin.im/plugins/?publisher=all&query=&type= range] of different chat services in one client
* [https://riot.im Riot] has Matrix and IRC in one package
* [https://tools.suckless.org/sic/ sic] is under 250 lines of C for POSIX systems
* [https://weechat.org/ WeeChat] is a somewhat popular rival to IRSSI, but contrary to its name, it has been known to use many times the memory and CPU of IRSSI
* [http://felloff.net/usr/cinap_lenrek/wircrc wircrc] is a multi-window IRC client in under 300 lines of Plan 9 shell script, which shows what can be done with operating system support
* [http://xchat.org/ xchat] a X11 based IRC chat client, somewhat user friendly and available in many Linux distributions.

See also Wikipedia's [https://en.wikipedia.org/wiki/Comparison_of_Internet_Relay_Chat_clients Comparison of Internet Relay Chat clients].

Somewhat related: [https://www.bitlbee.org/main.php/news.r.html Bitlbee] is an open-source gateway from IRC to many other chat services. It could be a shortcut to using these other services with your OS. It uses the same libpurple as Pidgin, but obviously cannot support images or video.

[[Category:Main Page Topics]]
[[Category:Resources]]
````

<a id="w002040"></a>

## W002040 — Talk:Chat

Bron-ID: W002040
Namespace: Talk (1)
Revisie: 30128 | Datum: 2026-03-04T19:16:49Z
Bron: https://wiki.osdev.org/Talk:Chat
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AChat&oldid=30128

````text
I created the Chat page just as a test to see how well it worked. Should I keep the page, move it to another namespace, or move it outside the wiki all together? -[[User:Chase|Chase]] 08:38, 16 May 2007 (CDT)

I can't say I'm the biggest fan of Java applets, Can't this page just list OSDev related IRC(<i>Or others..</i>) chatrooms and make recommendations for IRC clients? --[[User:Brynet-Inc|Brynet-Inc]] 14:07, 18 May 2007 (CDT)

I agree with [[User:Brynet-Inc|Brynet-Inc]]. It should be changed to a page with a list of places to go. [[User:Yayyak|Yayyak]] 02:06, 31 August 2007 (CDT)

== On the migration from Freenode ==

The '''#osdev''' channel formerly on Freenode has moved to Libera in response to recent events. The decision to move was unanimous among the staff of the channel and (only a few days later) user numbers are at a significant percentage of what we had on Freenode - understandably, some idlers were lost in the shuffle. In moving to Libera, a discussion was held on the relationship between #osdev and osdev.org. While it was fully recognized that there was no official relationship between the two while the channel was on Freenode, in consideration for the overlap in staff and in the interest of maintaining a viable claim as a "project" channel under the policies Libera inherited from Freenode, we have made the decision to found the new channel jointly between the wiki/forum and the former Freenode channel. While not necessarily an enthusiastic decision, the staff were all in agreement and the sentiment from general members was positive. Personally, I am glad to see twenty years of will-they-won't-they finally resolved, and this will lead to far less confusion or pronoun juggling when discussing the wiki in conversations on IRC ;)

@chase: You, of course, still have right-of-refusal to the joint operation of the channel, and it is unfortunate that the rapidity with which these events unfolded made it difficult to get all parties on board. Regardless, we should probably change the profile entry on the forum for freenode usernames, as I suspect it will cause confusion regardless moving forward.

--[[User:Klange|klange]] 20:23, 26 May 2021 (CDT)

== Update Discord ==
"Discord with hundreds of members" should be replaced with "Discord with thousands of members". :^) --[[User:Lukflug|Lukflug]] 09:55, 3 June 2022 (CDT)

* "Welcoming and friendly" should also be replaced with "Very welcoming and friendly" :^) --[[User:Wxwisiasdf|Wxwisiasdf]]

My previous request still stands: we now have well over 13k members. In addition, the link is no longer a Matrix room, but a Matrix space. As an alternative to Discord, we also offer a Fluxer community: https://fluxer.gg/eFPqIzIs. --[[User:Lukflug|Lukflug]] ([[User talk:Lukflug|talk]]) 16:34, 22 February 2026 (UTC)

* done [[User:Eekee|Eekee]] ([[User talk:Eekee|talk]]) 19:16, 4 March 2026 (UTC)
````

<a id="w002041"></a>

## W002041 — User:Com1

Bron-ID: W002041
Namespace: User (2)
Revisie: 3836 | Datum: 2007-05-16T19:50:27Z
Bron: https://wiki.osdev.org/User:Com1
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACom1&oldid=3836

````text
Hi, Im COM!
````

<a id="w002060"></a>

## W002060 — RU:Hardware Abstraction Layer

Bron-ID: W002060
Namespace: Main (0)
Revisie: 3909 | Datum: 2007-05-19T21:04:05Z
Bron: https://wiki.osdev.org/RU:Hardware_Abstraction_Layer
Revisie-URL: https://wiki.osdev.org/index.php?title=RU%3AHardware_Abstraction_Layer&oldid=3909
Doorverwijzing: Hardware Abstraction Layer

````text
#REDIRECT [[Hardware Abstraction Layer]]
````

<a id="w002061"></a>

## W002061 — Talk:RU:Hardware Abstraction Layer

Bron-ID: W002061
Namespace: Talk (1)
Revisie: 3911 | Datum: 2007-05-19T21:04:05Z
Bron: https://wiki.osdev.org/Talk:RU:Hardware_Abstraction_Layer
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ARU%3AHardware_Abstraction_Layer&oldid=3911
Doorverwijzing: Talk:Hardware Abstraction Layer

````text
#REDIRECT [[Talk:Hardware Abstraction Layer]]
````

<a id="w002062"></a>

## W002062 — Continuation Systems

Bron-ID: W002062
Namespace: Main (0)
Revisie: 22715 | Datum: 2018-09-10T02:20:34Z
Bron: https://wiki.osdev.org/Continuation_Systems
Revisie-URL: https://wiki.osdev.org/index.php?title=Continuation_Systems&oldid=22715

````text
{{Tone}}
Time-sharing works by interrupting the running program and saving its run-time state (such as program counter and registers) into a data structure (often the kernel-level stack). Theoretically speaking, that saved state constitutes a '''one-shot continuation''' of the interrupted program. Indeed, switching threads in a thread-based system involves saving the continuation of the old thread into that thread's data structure, and then loading and ''throwing to'' a new continuation from the new thread.  Adding a ''copy'' operation for continuations allows user-programs to decide when to use expensive multi-shot continuations.  Finally, adding an operation to suspend the current continuation and pass it as a value to a specified new continuation enables continuations to communicate -- throwing to a continuation sends it a value that may include its invoker.

Because no saved continuation can represent the currently-running program, the idea of '''virtual processors''' is required.  These represent the current continuation by storing the kernel's status information for the current computation and abstracting over machine processors.  The kernel can use virtual processors to support multiprocessor or multi-core machines by creating a virtual processor abstraction for each logical processor.

One can easily see an analogy: virtual processors look and behave like running threads, while saved continuations look and behave like suspended threads.

Scheme and ML programmers have known for years how to use continuations of coroutines to create cooperative threading systems, but a preemptive system (like most operating-system creators desire) requires some mechanism to preempt a running program.  Adding an '''event''' abstraction accomplishes this.  These events can contain any number of saved continuations for "handlers", and support an operation to "fire" or "trigger" the event. When something fires an event, it dequeues all of its handlers and hands them to a scheduler that only needs to decide what kind of event-handler continuations can preempt what kind of running virtual processors.  When the scheduler decide to preempt, it simply passes the preempted continuation to the preempting continuation as a value, possibly checking the return value it receives when the preempting continuation finishes for error codes.  Giving the handler the preempted continuation allows it to perform higher-level scheduling operations, possibly moving a function that even L4 kept in the kernel out of it.

Note that events can serve as a convenient abstraction over machine interrupts.  Using them as such provides a convenient way to translate hardware support for preemption into operating-system support for preemption.


However, adding a continuation as an event handler with the current set of operations will run far too slowly for real use.  Instead, events gain a new operation that adds the ''current continuation'' to the given event's set of handlers and then invokes a ''given continuation'' with a ''given value''. This primitive can run much more efficiently.


Finally, adding a single additional primitive operation will serve to bring this concurrency system into the truly parallel world of multi-core and multi-processor machines. Since a continuation cannot terminate itself without throwing to another continuation, a virtual processor can even run itself forever once started. One only needs a primitive to start an inactive virtual processor with a given continuation and value.

In conclusion, continuations; virtual processors; and events can be used to build complex concurrency and parallelism models on top of normal processes by giving user-space or higher levels of the kernel control over the multiplexing of processors.

An interesting side question: Can events be used to implement standard synchronization objects, assuming that the kernel makes events work in a parallel-safe manner?  It can easily be shown that events will sufficiently serve to implement barrier variables, but what about semaphores?

== See Also ==
=== External Links ===
* [https://citeseer.ist.psu.edu/reppy90asynchronous.html "Asynchronous Signals in Standard ML" by John H. Reppy, (C) 1990]

[[Category:Processes and Threads]]
````

<a id="w002064"></a>

## W002064 — Writing a memory manager

Bron-ID: W002064
Namespace: Main (0)
Revisie: 20837 | Datum: 2017-06-03T09:33:27Z
Bron: https://wiki.osdev.org/Writing_a_memory_manager
Revisie-URL: https://wiki.osdev.org/index.php?title=Writing_a_memory_manager&oldid=20837

````text
{{Rating|2}}
{{In_Progress}}
== First fit MM ==
It's not that hard to implement a basic first fit MM. By memory manager, I don't mean [[Paging]] (you just keep a list of free/used pages), but rather a simple library you can use in userspace and in your kernel (or globally if you have a flat memory model).

What I'm going to talk about assumes you know where the chunk of free memory is you can play with. For a flat memory model, this can be from the end of your kernel code (say the 1/2MB mark) to the end of the system memory (GRUB can give you this info). For a paging kernel, this can be the beginning of the page to the end of the page.

=== malloc() ===

Okay, the first step is to get out a pen and paper - I don't mean a keyboard and text editor since you can't draw diagrams (a stylus/tablet is okay with your favorite-graphics-program/OneNote/Windows-Journal).

Anyway, my point is. Do you understand the basic concept? Sketch how the memory will be laid out on paper. Don't start writing your allocate function straight away. That should be one of the last things you should do. Actually design how you want your memory to be managed, where you want to store the information (in a header before, in a global bitmap, etc). What do you want to store in the header/bitmap? If it's a header, what sort of data will be saved in it (pointer to the next free space, if it's free or used).

Remember this is your operating system! Plan it how you would like.

The important thing is to start from an OVERALL view. Not sitting at a keyboard with a blank malloc function expecting to know what do do.

Then write a basic structure for the header/bitmap.

When you know how memory is stored, then you can go on to your allocation function. Close that text editor, we're not done with that pen yet  :) Again, start with a top down view. What, exactly, does a memory allocator do?
# Find free block
# Mark it as used
# Return its address

Then break it up.

How do we find a free block? (this is for a first fit method with a header, but for a basic buddy system all you need is some imagination)
* Start with a pointer to the beginning of the free memory
* Does this header have an End Of Memory status?
** Yes
*** Allocate a new page
** No
*** return an error (such as a pointer to 0)
* Does this header say this memory is free
** Yes
*** Is (address of next header - (address of this header - size of header)) larger than the size wanted
**** Yes?
***** WE FOUND A FREE BLOCK!!!!
**** No?
***** Point to the next block and go back to step 2
** No
*** Point to the next block and go back to step 2

Once you have found one, set it's used bit to used. If the space found is significantly larger than the space needed, then it is generally a good idea to split it up (cut it in half, then half again, or add another header right after the used space and update the pointers. use your imagination!)  ;] Then return the address!

=== free() ===
Don't program it yet! Now you need to write your free function. You need to basically go through the same process. You need a way to find out where about 
in your bitmap or which header is associated with the address you're given. With a header it's simply address - size of header. Set it's bit to free.

The next step is to scan through the memory and merge free blocks (e.g. if next pointer's header is free, search the next pointer's header, then the next, until you find a used block, then set the current block's next pointer to that used block, skipping over the free blocks - if any of that made sense :D).

Just keep breaking it into smaller, and smaller blocks. Make sure you understand 100% how each process works. The key is to understand it fully before you start coding! Write down every equation for everything. It is essential you know how your code works, and have it in readable form on paper, so when an error occurs it's easy to debug.

=== Testing ===
Only then, you can start coding. Once it's written, you should try it out in a simple sandbox environment. Such as allocate around about 5 random sizes, deallocate a few from the middle, re allocate them, and get them to print out all their memory addresses at each step, then take the time to work it out on paper to see if it's doing it correctly. The main problem I've had is with the freeing (combining free blocks together) which I had to print out every step to the screen to find the bug (I forget what it was, a = instead of a == I think).

Don't copy someone else's code! You can copy their basic theory, but plan how you want it to work in YOUR OS on paper, and implement it YOUR way. That way, you know exactly how it works, and you can debug it and extend on it yourself.

Good luck with your MM.  :]

== Best fit MM ==
A basic idea of a best fit memory manager is same as a first fit one. Having blocks in memory. But the problem with first fit is the fragmentation which occurs when splitting blocks way bigger than we need, when there are smaller fitting blocks.

=== Keeping track of free blocks ===
You need to know where each free block is in memory, and how big it is. You can store this in an array, or binary tree, or whatever.
The way you sort the list is important. If we sort the list on size, small to big, we can look for free block starting at first entry, and as long as the current node is not big enough, going to next node. This way you will find the best fitting block. If no block is found, you need to ask the virtual memory manager for more memory.

For making this work you can make some nice functions. Those functions (add for example) finds the best place to put a node (free block), and then moves all other nodes a bit for making place for new node. Removing a block is easier. Just find the given block in the list, remove it and move all other blocks to fill up the hole.

=== Merging and splitting blocks ===
Merging and splitting free blocks becomes a bit more work because you have to keep your free block list updated.

Good luck making a best fit memory allocator!

[[Category:Memory management]]
[[Category:Tutorials]]
````

<a id="w002065"></a>

## W002065 — Template:OSProjectEntry

Bron-ID: W002065
Namespace: Template (10)
Revisie: 21851 | Datum: 2017-12-13T15:34:59Z
Bron: https://wiki.osdev.org/Template:OSProjectEntry
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AOSProjectEntry&oldid=21851

````text
<br />
* '''{{{OSName}}}'''  - {{{OSDescription}}} '' {{{Note|}}} ''
** [[Image:Icon_phone.png]] Contact: <tt> ''{{{Contact|No Contact Information}}}'' </tt>
** [[Image:Icon_license.png]] License: <tt> ''{{{License|No License Information}}}'' </tt>
** [[Image:Icon_site.png]] URL: <tt> ''{{if|{{{URL|}}}|[{{{URL}}} {{{URL}}}]|No URL}}'' {{{URLEx|}}} </tt>
** [[Image:Icon_clock.png]] Status: <tt> ''{{Month|{{{LastReleaseMonth}}}}} {{if|{{{LastReleaseDay|{{{LastReleaseMonth|}}}}}}|{{{LastReleaseDay}}},&nbsp;}}{{{LastReleaseYear|}}}'' {{if|{{{LastReleaseDescription|}}}|-- ''{{{LastReleaseDescription}}}''}} {{if|{{{Status|}}}|-- ''{{{Status|}}}''}} </tt>
**[[Image:Icon_floppy.png]] ''{{{Floppy availability|Unknown}}}''<br>
{{{CloserInspection|}}}
````

<a id="w002066"></a>

## W002066 — File:Time.png

Bron-ID: W002066
Namespace: File (6)
Revisie: 3949 | Datum: 2007-05-27T19:35:28Z
Bron: https://wiki.osdev.org/File:Time.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ATime.png&oldid=3949

````text
A picture of a clock with a stop symbol in front representing a stopped clock.
````

<a id="w002072"></a>

## W002072 — File:Icon magnify warning.png

Bron-ID: W002072
Namespace: File (6)
Revisie: 3963 | Datum: 2007-05-27T20:47:07Z
Bron: https://wiki.osdev.org/File:Icon_magnify_warning.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AIcon_magnify_warning.png&oldid=3963

````text
A magnify icon with warning color.
````

<a id="w002073"></a>

## W002073 — Template:CloserInspectionValidity

Bron-ID: W002073
Namespace: Template (10)
Revisie: 12736 | Datum: 2012-02-19T20:33:03Z
Bron: https://wiki.osdev.org/Template:CloserInspectionValidity
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ACloserInspectionValidity&oldid=12736

````text
** [[Image:Icon_magnify_warning.png]] '''The validity of this entry is questionable;'''<br \> ''{{{1}}}''
````

<a id="w002074"></a>

## W002074 — File:Icon info.png

Bron-ID: W002074
Namespace: File (6)
Revisie: 3984 | Datum: 2007-05-27T22:36:54Z
Bron: https://wiki.osdev.org/File:Icon_info.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AIcon_info.png&oldid=3984

````text
A info icon.
````

<a id="w002075"></a>

## W002075 — User:Kmcguire

Bron-ID: W002075
Namespace: User (2)
Revisie: 15602 | Datum: 2014-02-21T07:44:46Z
Bron: https://wiki.osdev.org/User:Kmcguire
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKmcguire&oldid=15602

````text
== User Pages ==
[http://wiki.osdev.org/index.php?title=Special%3APrefixIndex&prefix=kmcguire&namespace=2 User Pages]

=== Reading CLR PE32 File ===
==Useful Defines, External API, References==
<source lang="c">
#define UINT8_AT(x) (*((uint8_t*)(x)))
#define UINT16_AT(x) (*((uint16_t*)(x)))
#define UINT32_AT(x) (*((uint32_t*)(x)))
</source>
Also the memcpy, memset, malloc, strcmp, and free from the C standard library are used. I also use printf but
only for diagnostic purposes. 

Also, the page here has been very useful: http://ntcore.com/files/dotnetformat.htm.
==Portable Executable 32 File Header And NT Header Structure ==
<source lang="c">
typedef struct {
	uint32_t	signature;
	uint16_t	cputype;
	uint16_t	objcnt;
	
	uint32_t	tdstamp;
	uint32_t	reserved1;
	
	uint32_t	reserved2;
	uint16_t	nthdrsize;
	uint16_t	flags;
	
	uint16_t	reserved3;
	uint16_t	lmajor;
	uint16_t	lminor;
	uint16_t	reserved4;
	
	uint32_t	reserved5;
	uint32_t	reserved6;
	
	uint32_t	entrypointrva;
	uint32_t	reserved7;
	
	uint32_t	reserved8;
	uint32_t	imagebase;
	
	uint32_t	objectalign;
	uint32_t	filealign;
	
	uint16_t	osmajor;
	uint16_t	osminor;
	uint16_t	usermajor;
	uint16_t	userminor;
	
	uint16_t	subsysmajor;
	uint16_t	subsysminor;
	uint32_t	reserved9;
	
	uint32_t	imagesize;
	uint32_t	hdrsize;
	
	uint32_t	filechecksum;
	uint16_t	subsystem;
	uint16_t	dllflags;
	
	uint32_t	stackreservesize;
	uint32_t	stackcommitsize;
	
	uint32_t	heapreservesize;
	uint32_t	heapcommitsize;
	
	uint32_t	reserved10;
	uint32_t	_rvasizes;
	
	struct {
		uint32_t	rva;
		uint32_t	size;
	} tables[];
	
} win32_pe32_fhdr;

#define WIN32_PE32_TABLE_EXPORT		0x00
#define WIN32_PE32_TABLE_IMPORT		0x01
#define WIN32_PE32_TABLE_RESOURCE	0x02
#define WIN32_PE32_TABLE_EXCEPTION	0x03
#define WIN32_PE32_TABLE_SECURITY	0x04
#define WIN32_PE32_TABLE_FIXUP		0x05
#define WIN32_PE32_TABLE_DEBUG		0x06
#define WIN32_PE32_TABLE_IMAGEDESC	0x07
#define WIN32_PE32_TABLE_MACHSPEC	0x08
#define WIN32_PE32_TABLE_THREADLOC	0x09
#define WIN32_PE32_TABLE_UNKNOWN1	0x0A
#define WIN32_PE32_TABLE_UNKNOWN2	0x0B
#define WIN32_PE32_TABLE_IMPORTADDR	0x0C
#define WIN32_PE32_TABLE_UNKNOWN3	0x0D
#define WIN32_PE32_TABLE_CLRRTHDR	0x0E
</source>
==Common Language Runtime Header==
<source lang="c">
typedef struct {
	uint32_t	rva;
	uint32_t	size;
} win32_clr20_datadir;

/* http://ntcore.com/files/dotnetformat.htm */
typedef struct {
	uint32_t			cb;
	uint16_t			rtmajor;
	uint16_t			rtminor;
	win32_clr20_datadir		meta;
	uint32_t			flags;
	uint32_t			entrypointrva;	/* entry point rva */
	win32_clr20_datadir		resources;	/* resources */
	win32_clr20_datadir		strongnamsig;	/* strong name signature */
	win32_clr20_datadir		codemantab;	/* code manager table */
	win32_clr20_datadir		vtabfixups;	/* vtable fixups */
	win32_clr20_datadir		exaddrtabjumps;	/* export address table jumps */
} win32_clr20_hdr;
</source>
==CLR Meta-Data Container Structures==
These are custom structures that I made to hold the data as I read and process it. They
are not defined inside the PE32 file nor by the standard. These structures are just a
natural and hopefully simple way to hold and access the data.

The #~ stream (win_clr20_meta_stream) is converted into an array of tables (win_clr20_meta_table). I
am not sure why I prefixed them with "win_" I would remove it if I did it over, but for now I will
just leave it there.

The win_clr20_module is the main structure which also holds the win_clr20_meta structure. In most
of my functions I like to hold on to either the meta or module structure since I can access
anything I need.
<source lang="c">
typedef struct {
/*
	The meta-data contains streams. There are:
	#~
	#GUID
	#US
	#Strings
	
	Each of these is stored in this structure.
*/
	uint8_t					*name;		/* ASCII name */
	uint8_t					*data;		/* if non-zero is buffer */
	uint32_t				size;		/* size in bytes */
} win_clr20_meta_stream;

typedef struct {
/*
	This structure holds the table once it has been
	read from the meta-data #~ stream. It holds the
	rows in a fixed static grid. You can get the size
	of each row (which is constant), and the size of
	each field in the row. However, there is no naming
	per field.
*/
	uint32_t				rowcnt;
	uint8_t					rsize;		/* row size in bytes */
	uint8_t					*fsize;		/* size of each field from left to right in bytes */
	uint8_t					*rows;		/* array of pointers to row data for each row */
} win_clr20_meta_table;

typedef struct {
/*
	This is the main structure for the meta-data. It
	should allow access to other information from this
	structure.
*/
	uint16_t				strmcnt;			/* the number of array items below */
	win_clr20_meta_stream	*strms;				/* an array just like they are read in */
	win_clr20_meta_stream	strmbyid[5];		/* copy of strms, but indexed by WIN_CLR20_META_ */
	
	uint8_t					heapoffsetsizes;	/* heap offset sizes */
	uint32_t				valid[2];			/* qword(64bit) tells which tables are present */
	uint32_t				sorted[2];			/* qword(64bit) tells which tables are sorted */

	win_clr20_meta_table	tables[64];			/* all the known tables that are supported */
} win_clr20_meta;

/*
	A quick bit guide to find the right one.
*/
#define CLR20_META_HEAPOFFSETSIZES_STRING		0x01
#define CLR20_META_HEAPOFFSETSIZES_GUID			0x02
#define CLR20_META_HEAPOFFSETSIZES_BLOB			0x03

/* 
	The identifier for each stream. This is
	specific to win_clr20_meta. These values
	are internal to this code.
*/
#define WIN_CLR20_META_TABLES	0x00
#define WIN_CLR20_META_USTRINGS	0x01
#define WIN_CLR20_META_STRINGS	0x02
#define WIN_CLR20_META_BLOB		0x03
#define WIN_CLR20_META_GUID		0x04

typedef struct {
	uintptr_t				rva;
	uintptr_t				size;
	uint8_t					*data;
} win_clr20_chunk;

typedef struct {
/*
	A container for the meta-data and
	for any other structures.
	
	Also included are the object sections
	like .text, .reloc, ect. These are kept
	because the meta data will reference
	into this sections. So we need to keep
	them close, but they are not actual 
	meta data so they are outside of the
	meta structure.
*/
	win_clr20_meta			meta;

	uint32_t				chunkcnt;
	win_clr20_chunk			*chunks;
} win_clr20_module;
</source>

==Accessing CLR Header Through PE32 Headers==
Here, I access the CLR header by walking the PE32 file and optional DLL headers. I also have to peek
into the data dictionaries. The file sections (.text, ...) are copied into what I named chunks which
are saved and act just like sections just using a different name. The CLR meta-data will point into
these chunks at a later time.

<source lang="c">
int cilvm_pe32_buffer(uint8_t *buffer, uint32_t len, win_clr20_module *clrmodule)
/*
	Will read a CLR module in the PE32 format into a memory structure.
*/
{
	win32_pe32_fhdr		*fhdr;
	win32_pe32_ohdr		*ohdr;
	
	uint32_t			pe_off;
	uint32_t			opt_off;
	uint16_t			opt_magic;
	win_clr20_hdr		*clr_hdr;
	uint32_t			clr_rva;
	uint32_t			clr_size;
	uint32_t			clr_off;
	uint32_t			meta_off;
	uint32_t			meta_rva;
	uint32_t			meta_size;
	uint32_t 			x;
	uint32_t			y;
	uint8_t				*tb;

	/* get pe32 header offset */
	pe_off = ((uint32_t*)(buffer + 0x3c))[0];
	printf("peoff:%x\n", pe_off);
	
	if(memcmp(buffer + pe_off, "PE\0\0", 4) == 0)
	{
		printf("Yes, this is a PE header.\n");
	}
	
	fhdr = (win32_pe32_fhdr*)(buffer + pe_off);
	
	printf("imagebase: %x\n", fhdr->imagebase);
	
	/* get clr runtime header */
	clr_rva = fhdr->tables[WIN32_PE32_TABLE_CLRRTHDR].rva;
	clr_size = fhdr->tables[WIN32_PE32_TABLE_CLRRTHDR].size;
	printf("CLRRTHDR_RVA:%x\nCLRRTHDR_SIZE:%x\n", clr_rva, clr_size);
	
	/* 
		The array of objects follow the headers. But, my header does
		not include the size for the data dictionary array. So to
		include that we get the number of entries in the data dictionary
		and multiple this by eight since each one is eight bytes in PE32.
	*/
	ohdr = (win32_pe32_ohdr*)(buffer + pe_off + sizeof(win32_pe32_fhdr) + (fhdr->_rvasizes * 8));

	/*
		I also copy each section into a chunk. I used a different name incase
		we start reading from stuff other than PE32 files and the word chunk
		seemed cool to use.
	*/
	clrmodule->chunkcnt = fhdr->objcnt;
	clrmodule->chunks = (win_clr20_chunk*)malloc(sizeof(win_clr20_chunk) * fhdr->objcnt);
	for(x = 0; x < fhdr->objcnt; ++x)
	{
		clrmodule->chunks[x].rva = ohdr[x].rva;
		clrmodule->chunks[x].size = ohdr[x].psize;
		clrmodule->chunks[x].data = (uint8_t*)malloc(ohdr[x].psize);
		memcpy(clrmodule->chunks[x].data, buffer + ohdr[x].poff, ohdr[x].psize);
	}

	for(x = 0; x < fhdr->objcnt; ++x)
	{
		printf("object rva %x %s\n", ohdr[x].rva, &ohdr[x].name[0]);
		if(clr_rva >= ohdr[x].rva)
		{
			/* calculating relative to file offset not memory */
			clr_off = ohdr[x].poff + (clr_rva - ohdr[x].rva);
			printf("found object for CLRRTHDR, with rva %x\n", ohdr[x].rva);
			break;
		}
	}
	clr_hdr = (win_clr20_hdr*)(buffer + clr_off);

	/* try to read in the meta data */
	printf("clr-metatable:%x [size:%x]\n", clr_hdr->meta.rva, clr_hdr->meta.size);
	meta_rva = clr_hdr->meta.rva;
	meta_size = clr_hdr->meta.size;
	
	/* find the object that the metadata is in and point header there */
	for(x = 0; x < fhdr->objcnt; ++x)
	{
		printf("object rva %x %s\n", ohdr[x].rva, &ohdr[x].name[0]);
		if(meta_rva >= ohdr[x].rva)
		{
			/* calculating relative to file offset not memory */
			meta_off = ohdr[x].poff + (meta_rva - ohdr[x].rva);
			printf("found object for CLR-METADATA, with rva %x\n", ohdr[x].rva);
			break;
		}
	}
	
	/* call routine to read meta-data header */
	cilvm_clr_meta_read(buffer + meta_off, meta_size, &clrmodule->meta);
	/* after reading in the meta-data we need to process it into a structure */
	cilvm_clr_meta_tread(&clrmodule->meta);

	return 1;
}
</source>
The cilvm_clr_meta_read and cilvm_clr_meta_tread are not yet shown, but there each perform a small
part in the eventual construction of a high level structure where the common intermediate language
instructions can be accessed and processing performed.

==Read CLR Meta-Data Streams Into Memory==
<source lang="c">
int cilvm_clr_meta_read(uint8_t *buffer, uint32_t len, win_clr20_meta *meta)
/*
	This will read the meta-data streams.
*/
{
	uint16_t	ver_major;
	uint16_t	ver_minor;
	uint16_t	ver_strlen;
	uint16_t	strmcnt;
	uint32_t	x;
	uint32_t	y;
	uint16_t	strm_off;
	uint16_t	strm_size;
	uint8_t		*_buffer;
	
	_buffer = buffer;

	if(UINT32_AT(buffer) != 0x424A5342)
	{
		printf("invalid clr metadata signature\n");
		return 0;
	}
	
	ver_major = UINT16_AT(buffer + 4 + 0);
	ver_minor = UINT16_AT(buffer + 4 + 2);
	
	printf("meta data ver:%x.%x\n", ver_major, ver_minor);
	
	ASSERT((ver_major == 1) && (ver_minor == 1));
		
	/* jump over signature, version, and reserved */
	buffer += 4 + 4 + 4;
	
	/* display and jump over version string */
	ver_strlen = UINT32_AT(buffer);
	ASSERT((ver_strlen & 0x3) == 0x0);			/* has to be multiple of 4 */
	printf("ver_strlen:%u\n", ver_strlen);
	printf("ver:%s\n", buffer + 4);
	buffer += 4 + ver_strlen;
		
	/* jump over flags; 16 bit field; supposed to always be zero;
	   but apparently it is not always zero oddly enough..
	ASSERT(UINT16_AT(buffer) != 0x0); */
	buffer += 2;
	
	strmcnt = UINT16_AT(buffer);
	buffer += 2;
	
	printf("MetaDataStreamCount:%u\n", strmcnt);
	
	/* read each metadata stream header */
	meta->strmcnt = strmcnt;
	meta->strms = (win_clr20_meta_stream*)malloc(sizeof(win_clr20_meta_stream) * strmcnt);
	for(x = 0; x < strmcnt; ++x)
	{
		/* read offset and size then copy into newly allocated buffer */
		printf("off:%x,size:%x,name:%s\n", UINT32_AT(buffer), UINT32_AT(buffer + 4), buffer + 8);
		meta->strms[x].data = (uint8_t*)malloc(UINT32_AT(buffer + 4));
		memcpy(meta->strms[x].data, _buffer + UINT32_AT(buffer), UINT32_AT(buffer + 4));
		buffer += 4 + 4;
		/* read stream name (stop at zero byte) */
		y = 0;
		while(buffer[y] != 0x00)
		{
			++y;
		}
		++y;
		y = ((y & 3) > 0 ? (y & (~3)) + 4 : y);
		meta->strms[x].name = (uint8_t*)malloc(y + 1);
		memcpy(meta->strms[x].name, buffer, y + 1);
		/* advance to next offset and size pair */
		buffer += y;
	}

	/* give each stream a home at a consistent index */
	for(x = 0; x < meta->strmcnt; ++x)
	{
		if(strcmp(meta->strms[x].name, "#~") == 0)
		{
			meta->strmbyid[WIN_CLR20_META_TABLES] = meta->strms[x];
		}
		if(strcmp(meta->strms[x].name, "#US") == 0)
		{
			meta->strmbyid[WIN_CLR20_META_USTRINGS] = meta->strms[x];
		}
		if(strcmp(meta->strms[x].name, "#Strings") == 0)
		{
			meta->strmbyid[WIN_CLR20_META_STRINGS] = meta->strms[x];
		}
		if(strcmp(meta->strms[x].name, "#Blob") == 0)
		{
			meta->strmbyid[WIN_CLR20_META_BLOB] = meta->strms[x];
		}
		if(strcmp(meta->strms[x].name, "#GUID") == 0)
		{
			meta->strmbyid[WIN_CLR20_META_GUID] = meta->strms[x];
		}
	}
	return 1;
}
</source>

==Meta-Data #~ Stream Table Field Descriptions==
This allows us to write a function to interpret the table data that follows the
header in the #~ meta-data stream. The indexes can vary in size by what tables
they reference so we have to create a letter for each specific index which refers
to specific tables so that during run-time we can calculate the index size needed.

In my opinion this was done to conserve space, but I am not sure.
<source lang="c">
/*
	The best way I could think of was to create a large table
	that has 64 entries. Since at most with 2.0 there can be
	a maximum of 64 tables. At this time not all of these are
	used. So I only populate the indexes that are for a specific
	table.
	
	What I do is describe the fields in a row for each table. Since
	each table has different fields per row each entry is different.
	Also, most of the fields are index types. And since the index
	size can change to, apparently conserve space, I use these identifiers
	so that when I actually go to use them I calculate their size before
	hand. The '1', '2', and '4' are always this size.

	1 - 8 bit field
	2 - 16 bit field
	4 - 32 bit field
	S - index to String table
	G - index to GUID table
	R - resolution scope index (Module, ModuleRef, AssemblyRef, TypeRef)
	T - TypeDefOrRef index (TypeRef, TypeDef, TypeSpec)
	F - Field table index
	M - MethodDef table index
	B - Blob table index
	P - Param table index
	D - TypeDef table index (different from T)
	E - MemberRefParent index (TypeRef, ModuleRef, MethodDef, TypeSpec, TypeDef)
	H - HasConstant index (Param, Field, Property)
	C - HasCustomAttribute index (**AnyTable** Except It's Self)
	I - CustomAttributeType index (MethodDef, MethodRef)
	O - HasFieldMarshal index (Field, Param)
	A - HasDesclSecurityIndex (TypeDef, MethodDef, Assembly)
	V - Event table index
	Y - Property table index
	S - HasSemantics index (Event, Property)
	U - MemberForwarded index (Field, MethodDef)
	B - Implementation index (File, AssemblyRef, ExportedType)
	X - CustomAttributeType index (MethodDef, MethodRef)
	W - ResolutionScope index (Module, ModuleRef, AssemblyRef, TypeRef)
	N - ModuleRef table index
	J - AssemblyRef table index
	K - TypeOrMethodDef index (TypeDef, MethodDef)
	L - GenericParam table index
*/

/*
	This is the table's ID.
*/
#define CLR20_META_TMODULE				0
#define CLR20_META_TTYPEREF				1
#define CLR20_META_TTYPEDEF				2
#define CLR20_META_TFIELD				4
#define CLR20_META_TMETHODDEF			6
#define CLR20_META_TPARAM				8
#define CLR20_META_TIFACEIMPL			9	/* interface implementation table */
#define CLR20_META_TMEMBERREF			10
#define CLR20_META_TCONSTANT			11
#define CLR20_META_TCUSTOMATTRI			12
#define CLR20_META_TFIELDMARSHAL		13
#define CLR20_META_TDECLSECURITY		14
#define CLR20_META_TCLASSLAYOUT			15
#define CLR20_META_TFIELDLAYOUT			16
#define CLR20_META_TSTANDALONGSIG		17
#define CLR20_META_TEVENTMAP			18
#define CLR20_META_TEVENT				20
#define CLR20_META_TPROPERTY			23
#define CLR20_META_TMETHODSEMAN			24
#define CLR20_META_TMETHODIMPL			25
#define CLR20_META_TMODULEREF			26
#define CLR20_META_TIMPLMAP				28
#define CLR20_META_TFIELDRVA			29
#define CLR20_META_TASSEMBLY			32
#define CLR20_META_TASSEMBLYPROC		33
#define CLR20_META_TASSEMBLYOS			34
#define CLR20_META_TASSEMBLYREF			35
#define CLR20_META_TASSEMBLYREFPROC		36
#define CLR20_META_TASSEMBLYREFOS		37
#define CLR20_META_TFILE				38
#define CLR20_META_TEXPORTEDTYPE		39
#define CLR20_META_TMANIFESTRES			40
#define CLR20_META_TNESTEDCLASS			41
#define CLR20_META_TGENERICPARAM		42
#define CLR20_META_TGENERICPARAMCONST	44	/* generic parameter contraint */
#define CLR20_META_TPROPERTYMAP			21
#define CLR20_META_TTYPESPEC			27
#define CLR20_META_TMETHODREF			63	/* WHERE IS THIS ONE??? */

/*
	This is the field description structure. It is created at
	compile time and is accessed during run-time to determine
	the row length so that it can be copied into a more suitable
	structure for dynamic access.
*/
uint8_t *WIN_CLR20_META_TFIELDS[64] = 
{
	[CLR20_META_TMODULE] = 			"2SGGG",
	[CLR20_META_TTYPEREF] = 		"RSS",			/* META_TTYPEREF */
	[CLR20_META_TTYPEDEF] = 		"4SSTFM",		/* META_TTYPEDEF */
	[CLR20_META_TFIELD] = 			"2SB",			/* META_TFIELD */
	[CLR20_META_TMETHODDEF] = 		"422SBP",		/* META_TMETHODDEF */
	[CLR20_META_TPARAM] = 			"22S",			/* META_TPARAM */
	[CLR20_META_TIFACEIMPL] = 		"DT",			/* META_TIFACEIMPL */
	[CLR20_META_TMEMBERREF] = 		"ESB",			/* META_TMEMBERREF */
	[CLR20_META_TCONSTANT] = 		"11HB",			/* META_TCONSTANT */
	[CLR20_META_TCUSTOMATTRI] = 	"CIB",			/* META_TCUSTOMATTRI */
	[CLR20_META_TFIELDMARSHAL] = 	"OB",			/* META_TFIELDMARSHAL */
	[CLR20_META_TDECLSECURITY] = 	"2AB",			/* META_TDECLSECURITY */
	[CLR20_META_TCLASSLAYOUT] = 	"24D", 			/* META_TCLASSLAYOUT */
	[CLR20_META_TFIELDLAYOUT] = 	"4F",
	[CLR20_META_TSTANDALONGSIG] =	"B",
	[CLR20_META_TEVENTMAP] = 		"DV",
	[CLR20_META_TEVENT] = 			"2ST",
	[CLR20_META_TPROPERTYMAP] =		"DY",
	[CLR20_META_TPROPERTY] =		"2SB",
	[CLR20_META_TMETHODSEMAN] =		"2MS",
	[CLR20_META_TMETHODIMPL] = 		"DII",
	[CLR20_META_TMODULEREF] =		"S",
	[CLR20_META_TTYPESPEC] =		"B",
	[CLR20_META_TIMPLMAP] =			"2USN",
	[CLR20_META_TFIELDRVA] =		"4F",
	[CLR20_META_TASSEMBLY] = 		"422224BSS",
	[CLR20_META_TASSEMBLYPROC] =	"4",
	[CLR20_META_TASSEMBLYOS] = 		"444",
	[CLR20_META_TASSEMBLYREF] = 	"22224BSSB",
	[CLR20_META_TASSEMBLYREFPROC] = "4J",
	[CLR20_META_TASSEMBLYREFOS] = 	"444J",
	[CLR20_META_TFILE] = 			"4SB",
	[CLR20_META_TEXPORTEDTYPE] = 	"44SSB",
	[CLR20_META_TMANIFESTRES] = 	"44SB",
	[CLR20_META_TNESTEDCLASS] = 	"DD",
	[CLR20_META_TGENERICPARAM] =	"22KS",
	[CLR20_META_TGENERICPARAMCONST]="LT"
};
</source>
==Reading Meta-Data #~ Stream Tables==
A wise point to note is that although the code below can handle all the
major tables defined by the ECMA-335 standard for the CLI there do exist
other tables that can be present which may not be standard. If one of these
tables exists then the code below may fail and measures should be taken
to catch this situation. I have included a comment at the bottom of this
function which shows where this check needs to be. Since if this unknown
table happens between some known tables then there is no way that I know
of to skip it because you have to know the size of each row to know it's
total size.
<source lang="C">
int cilvm_clr_meta_tread(win_clr20_meta *meta)
/*	
	Processes the CLR meta-data by inflating the table stream 
	into a binary structure which can be more easily accessed 
	later.
	
	All of the meta-data is stored in a stream like format. It
	is difficult to randomly access the data in this format as
	it lacks any block like structure. This function will read
	the stream and create a block like structure which can be
	easily accessed in a random manner.	
*/
{
	uint32_t		x;
	uint32_t		y;
	uint32_t		z;
	uint32_t		w;
	uint32_t		v;
	uint8_t 		*tbuf;			/* table stream buffer */
	uint8_t			ver_major;		/* major version */
	uint8_t			ver_minor;		/* minor version */
	uint8_t			fsize[26];		/* field size for each type in bytes */
	
	memset(fsize, 0, sizeof(fsize));
	memset(meta->tables, 0, sizeof(meta->tables));

	/* We are going to convert the stream into a structure */	
	tbuf = meta->strmbyid[WIN_CLR20_META_TABLES].data;
	
	/* The first DWORD is supposed to be reserved and zero. */
	ASSERT(UINT32_AT(tbuf) == 0);
	tbuf += 4;
	
	/* The major and minor version. */
	ver_major = UINT8_AT(tbuf + 0);
	ver_minor = UINT8_AT(tbuf + 1);
	tbuf += 2;
	
	/* http://ntcore.com/files/dotnetformat.htm
	   The heap offset sizes. This field is very important, it's a byte that 
	   tells us the size that indexes into the "#String", "#GUID" and "#Blob" 
	   streams will have. I paste you the description and the bit mask from 
	   the SDK 
	   
		Bit mask	Description
		0x01		Size of “#String” stream >= 2^16.
		0x02		Size of “#GUID” stream >= 2^16
		0x04		Size of “#Blob” stream >= 2^16.
		
		If zero index are 16 bits and if one index are 32 bits.
	*/
	meta->heapoffsetsizes = UINT8_AT(tbuf++);
	
	/* Reserved byte which should be zero.. */
	/* ASSERT(UINT8_AT(tbuf) == 0); */
	++tbuf;
	
	/*  http://ntcore.com/files/dotnetformat.htm
	    It's a bitmask-qword that tells us which MetaData Tables are present in
	    the assembly. Of course, since this is a qword the maximum number of 
	    tables is 64. However, most of the tables aren't defined yet. So, the
	    high bits of this qword are always 0.
	*/
	meta->valid[0] = UINT32_AT(tbuf + 0);
	meta->valid[1] = UINT32_AT(tbuf + 4);
	tbuf += 8;

	printf("TablesValid:%04x%04x\n", meta->valid[0], meta->valid[1]);

	/*  Also a bitmask-qword. It tells us which tables are sorted. */
	meta->sorted[0] = UINT32_AT(tbuf + 0);
	meta->sorted[1] = UINT32_AT(tbuf + 4);
	tbuf += 8;
	
	/*  http://ntcore.com/files/dotnetformat.htm
	    Following there's an array of dwords with the number of rows for each 
	    present table. Ok this has to be explained. For every table there can
	    be n rows. Let's say we have three tables: A, B and C. And the Valid 
	    mask tells us that the B table is not present, but A and C are. In 
	    this case there will be 2 dwords (not three), one for the rows in the 
	    A table and one for the C table. The B table rows are skipped since 
	    there is no B table in the assembly.
	*/
	for(y = 0; y < 2; ++y)
	{
		for(x = 0; x < 32; ++x)
		{
			if(((meta->valid[y] >> x) & 0x1) == 0x1)
			{
				/* This table is present. */
				printf("Table[%u] Present With %u Rows\n", x + (y * 32), UINT32_AT(tbuf));
				meta->tables[x + (y * 32)].rowcnt = UINT32_AT(tbuf);
				tbuf += 4;
			}else{
				meta->tables[x + (y * 32)].rowcnt = 0;
			}
		}
	}
	/* I need to calculate the size of any indexes that are used by
	   the dynamic field format table defined during compile-time.
	   Once the size of these indexes are defined then we can continue
	   by loading the actual data for the meta-data tables.
	*/
	fsize['1'] = 1;
	fsize['2'] = 2;
	fsize['4'] = 4;
	/* Either a 4 byte or 2 byte index needed depending on bit set. */
	fsize['S'] = (meta->heapoffsetsizes & CLR20_META_HEAPOFFSETSIZES_STRING) ? 4 : 2;
	fsize['G'] = (meta->heapoffsetsizes & CLR20_META_HEAPOFFSETSIZES_GUID) ? 4 : 2;
	fsize['B'] = (meta->heapoffsetsizes & CLR20_META_HEAPOFFSETSIZES_BLOB) ? 4 : 2;
	/* We or the row counts and if we have left something larger than
	   0xffff then this means that one of the tables is needs a 32 bit
	   index instead of a 16 bit one.
	*/
	fsize['R'] = (meta->tables[CLR20_META_TMODULE].rowcnt |
				 meta->tables[CLR20_META_TMODULEREF].rowcnt |
				 meta->tables[CLR20_META_TASSEMBLYREF].rowcnt |
				 meta->tables[CLR20_META_TTYPEREF].rowcnt) > 0xffff ? 4 : 2;
	fsize['T'] = (meta->tables[CLR20_META_TTYPEREF].rowcnt |
				  meta->tables[CLR20_META_TTYPEDEF].rowcnt |
				  meta->tables[CLR20_META_TTYPESPEC].rowcnt) > 0xffff ? 4 : 2;
	fsize['F'] = meta->tables[CLR20_META_TFIELD].rowcnt > 0xffff ? 4 : 2;
	fsize['M'] = meta->tables[CLR20_META_TMETHODDEF].rowcnt > 0xffff ? 4 : 2;
	fsize['P'] = meta->tables[CLR20_META_TPARAM].rowcnt > 0xffff ? 4 : 2;
	fsize['D'] = meta->tables[CLR20_META_TTYPEDEF].rowcnt > 0xffff ? 4 : 2;
	fsize['E'] = (meta->tables[CLR20_META_TTYPEREF].rowcnt |
				  meta->tables[CLR20_META_TMODULEREF].rowcnt |
				  meta->tables[CLR20_META_TMETHODDEF].rowcnt |
				  meta->tables[CLR20_META_TTYPESPEC].rowcnt |
				  meta->tables[CLR20_META_TTYPEDEF].rowcnt) > 0xffff ? 4 : 2;
	fsize['H'] = (meta->tables[CLR20_META_TPARAM].rowcnt |
				  meta->tables[CLR20_META_TFIELD].rowcnt |
				  meta->tables[CLR20_META_TPROPERTY].rowcnt) > 0xffff ? 4 : 2;
	/* A bit of a special case here. It can reference any table but it's self. */
	for(x = 0, y = 0; x < 64; ++x)
	{
		if(x != CLR20_META_TCUSTOMATTRI)
		{
			y = y | meta->tables[x].rowcnt;
		}
	}
	fsize['C'] = y > 0xffff ? 4 : 2;
	/* Oddly, the document says there is a MethodRef table? Where? If there
	   does exist one then it should be considered for 'K' here. Since at
	   the moment I have no idea what to do but just ignore it.
	*/
	fsize['I'] = (meta->tables[CLR20_META_TMETHODDEF].rowcnt |
				  meta->tables[CLR20_META_TMETHODREF].rowcnt) > 0xffff ? 4 : 2;
	fsize['O'] = (meta->tables[CLR20_META_TFIELD].rowcnt |
				  meta->tables[CLR20_META_TPARAM].rowcnt) > 0xffff ? 4 : 2;
	fsize['A'] = (meta->tables[CLR20_META_TTYPEDEF].rowcnt |
				  meta->tables[CLR20_META_TMETHODDEF].rowcnt |
				  meta->tables[CLR20_META_TASSEMBLY].rowcnt) > 0xffff ? 4 : 2;
	fsize['V'] = meta->tables[CLR20_META_TEVENT].rowcnt > 0xffff ? 4 : 2;
	fsize['Y'] = meta->tables[CLR20_META_TPROPERTY].rowcnt > 0xffff ? 4 : 2;
	fsize['S'] = (meta->tables[CLR20_META_TEVENT].rowcnt |
				  meta->tables[CLR20_META_TPROPERTY].rowcnt) > 0xffff ? 4 : 2;
	fsize['U'] = (meta->tables[CLR20_META_TFIELD].rowcnt |
				  meta->tables[CLR20_META_TMETHODDEF].rowcnt) > 0xffff ? 4 : 2;
	fsize['B'] = (meta->tables[CLR20_META_TFILE].rowcnt |
				  meta->tables[CLR20_META_TASSEMBLYREF].rowcnt |
				  meta->tables[CLR20_META_TEXPORTEDTYPE].rowcnt) > 0xffff ? 4 : 2;
	fsize['X'] = (meta->tables[CLR20_META_TMETHODDEF].rowcnt |
				  meta->tables[CLR20_META_TMETHODREF].rowcnt) > 0xffff ? 4 : 2;
	fsize['W'] = (meta->tables[CLR20_META_TMODULE].rowcnt |
				  meta->tables[CLR20_META_TMODULEREF].rowcnt |
				  meta->tables[CLR20_META_TASSEMBLYREF].rowcnt |
				  meta->tables[CLR20_META_TTYPEREF].rowcnt) > 0xffff ? 4 : 2;
	fsize['N'] = meta->tables[CLR20_META_TMODULEREF].rowcnt > 0xffff ? 4 : 2;
	fsize['J'] = meta->tables[CLR20_META_TASSEMBLYREF].rowcnt > 0xffff ? 4 : 2;
	fsize['K'] = (meta->tables[CLR20_META_TTYPEDEF].rowcnt |
				 meta->tables[CLR20_META_TMETHODDEF].rowcnt) > 0xffff ? 4 : 2;
	fsize['L'] = meta->tables[CLR20_META_TGENERICPARAM].rowcnt > 0xffff ? 4 : 2;
	
	
	/* http://ntcore.com/files/dotnetformat.htm
	   As you can see, some numbers are missing, that's because some tables, 
	   as I said before, are not defined yet. It's important you understand 
	   how the tables are stored. A table is made of an array of rows; a row 
	   is a structure (let's call it this way for the moment to make things 
	   easier). After the rows of a given table end, the rows of the next 
	   table follow. The problem with a row (remember, think of it like a 
	   structure) is that some of its fields aren't always of the same size 
	   and they change from assembly to assembly, so you have to calculate 
	   them dynamically.
	*/
	for(y = 0; y < 2; ++y)
	{
		for(x = 0; x < 32; ++x)
		{
			if(((meta->valid[y] >> x) & 0x1) == 0x1)
			{
				/* the table identifier */
				w = x + (y * 32);
                                /* 
                                   Here is where a check needs
                                   to be made to ensure that
                                   this is not a unknown table. 

                                   A unknown table will have no
                                   row field string so you might
                                   also get a crash here...
                                */
				/* calculate the size of each row */
				for(z = 0; WIN_CLR20_META_TFIELDS[w][z] != 0; ++z);
				meta->tables[w].fsize = (uint8_t*)malloc(z);
				for(v = 0, z = 0; WIN_CLR20_META_TFIELDS[w][z] != 0; ++z)
				{
					v += fsize[WIN_CLR20_META_TFIELDS[w][z]];
					meta->tables[w].fsize[z] = fsize[WIN_CLR20_META_TFIELDS[w][z]];
				}
				meta->tables[w].fcnt = z;
				/* read the number of rows specified by row size */
				meta->tables[w].rsize = v;
				meta->tables[w].rows = (uint8_t*)malloc(meta->tables[w].rowcnt * v);
				memcpy(meta->tables[w].rows, tbuf, meta->tables[w].rowcnt * v);
				tbuf += meta->tables[w].rowcnt * v;
				printf("read %u rows of %u bytes each for a total of %u.\n", 
						meta->tables[w].rowcnt, v, meta->tables[w].rowcnt * v);
			}
		}
	}
	
	return 1;
}
</source>
The tables have now been read into their own structure. The data for all the rows is pointed to by meta->tables[w].rows.

==Meta-Data Method Definition Table Field Indexes==
<source lang="c">
/*
	ECMA-335 4th Edition Section 22
	Table Field Indexes

	These are the actual index of each
	field in the row for each table denoted.

	Although, each field has a certain size
	and most index type fields can actually
	vary in size from one module to the next.

	This normally serves as a constant to pass
	to the actual helper function which can
	access fields in a specified table and row.

	So now you have a textual name to reference
	instead of having to remember arbitray index.
*/
#define CLR20_TASSEMBLY_HASHALGID		0
#define CLR20_TASSEMBLY_VERMAJOR		1
#define CLR20_TASSEMBLY_VERMINOR		2
#define CLR20_TASSEMBLY_BUILDNUM		3
#define CLR20_TASSEMBLY_REVISION		4
#define CLR20_TASSEMBLY_PUBLICKEY		5
#define CLR20_TASSEMBLY_NAME			6
#define CLR20_TASSEMBLY_CULTURE			7
#define CLR20_TASSEMBLYOS_OSPLATFORMID		0
#define CLR20_TASSEMBLYOS_OSMAJORVERSION	1
#define CLR20_TASSEMBLYOS_OSMINORVERSION	2
#define CLR20_TASSEMBLYPROC_PROCESSOR		0
#define CLR20_TASSEMBLYREF_VERMAJOR		0
#define CLR20_TASSEMBLYREF_VERMINOR		1
#define CLR20_TASSEMBLYREF_BUILDNUM		2
#define CLR20_TASSEMBLYREF_REVISION		3
#define CLR20_TASSEMBLYREFOS_OSPLATFORMID	0
#define CLR20_TASSEMBLYREFOS_OSVERMAJOR		1
#define CLR20_TASSEMBLYREFOS_OSVERMINOR		2
#define CLR20_TASSEMBLYREFOS_ASSEMBLYREF	3
#define CLR20_TASSEMBLYREFPROC_PROCESSOR	0
#define CLR20_TASSEMBLYREFPROC_ASSEMBLYREF	1
#define CLR20_TCLASSLAYOUT_PACKINGSIZE		0
#define CLR20_TCLASSLAYOUT_CLASSSIZE		1
#define CLR20_TCLASSLAYOUT_PARENT		2
#define CLR20_TCONSTANT_TYPE			0
#define CLR20_TCONSTANT_PARENT			1
#define CLR20_TCONSTANT_VALUE			2
#define CLR20_TCUSTOMATTRI_PARENT		0
#define CLR20_TCUSTOMATTRI_TYPE			1
#define CLR20_TCUSTOMATTRI_VALUE		2
#define CLR20_TDECLSECURITY_ACTION		0
#define CLR20_TDECLSECURITY_PARENT		1
#define CLR20_TDECLSECURITY_PERMISSIONSET	2
#define CLR20_TEVENTMAP_PARENT			0
#define CLR20_TEVENTMAP_EVENTLIST		1
#define CLR20_TEXPORTEDTYPE_FLAGS		0
#define CLR20_TEXPORTEDTYPE_TYPEDEFID		1
#define CLR20_TEXPORTEDTYPE_TYPENAME		2
#define CLR20_TEXPORTEDTYPE_TYPENAMESPACE	3
#define CLR20_TEXPORTEDTYPE_IMPLEMENTATION	4
#define CLR20_TFIELD_FLAGS			0
#define CLR20_TFIELD_NAME			1
#define CLR20_TFIELD_SIGNATURE			2
#define CLR20_TFIELDLAYOUT_OFFSET		0
#define CLR20_TFIELDLAYOUT_FIELD		1
#define CLR20_TFIELDMARSHAL_PARENT		0
#define CLR20_TFIELDMARSHAL_NATIVETYPE		1
#define CLR20_TFIELDRVA_RVA			0
#define CLR20_TFIELDRVA_FIELD			1
#define CLR20_TFILE_FLAGS			0
#define CLR20_TFILE_NAME			1
#define CLR20_TFILE_HASHVALUE			2
#define CLR20_TGENERICPARAM_NUMBER		0
#define CLR20_TGENERICPARAM_FLAGS		1
#define CLR20_TGENERICPARAM_OWNER		2
#define CLR20_TGENERICPARAM_NAME		3
#define CLR20_TGENERICPARAMCONST_OWNER		0
#define CLR20_TGENERICPARAMCONST_CONSTRAINT	1
#define CLR20_TIMPLMAP_MAPPINGFLAGS		0
#define CLR20_TIMPLMAP_MEMBERFORWARDED		1
#define CLR20_TIMPLMAP_IMPORTNAME		2
#define CLR20_TIMPLMAP_IMPORTSCOPE		3
#define CLR20_TIFACEIMPL_CLASS			0
#define CLR20_TIFACEIMPL_INTERFACE		1
#define CLR20_TMANIFESTRES_OFFSET		0
#define CLR20_TMANIFESTRES_FLAGS		1
#define CLR20_TMANIFESTRES_NAME			2
#define CLR20_TMANIFESTRES_IMPLEMENTATION	3
#define CLR20_TMEMBERREF_CLASS			0
#define CLR20_TMEMBERREF_NAME			1
#define CLR20_TMEMBERREF_SIGNATURE		2
#define CLR20_TMETHODDEF_RVA			0
#define CLR20_TMETHODDEF_IMPLFLAGS		1
#define CLR20_TMETHODDEF_FLAGS			2
#define CLR20_TMETHODDEF_NAME			3
#define CLR20_TMETHODDEF_SIGNATURE		4
#define CLR20_TMETHODDEF_PARAMLIST		5
#define CLR20_TMETHODIMPL_CLASS			0
#define CLR20_TMETHODIMPL_METHODBODY		1
#define CLR20_TMETHODIMPL_METHODDECLARATION	2
#define CLR20_TMETHODSEMAN_SEMANTICS		0
#define CLR20_TMETHODSEMAN_METHOD		1
#define CLR20_TMETHODSEMAN_ASSOCIATION		2
#define CLR20_TMETHODSPEC_METHOD		0
#define CLR20_TMETHODSPEC_INSTANTIATION		1
#define CLR20_TMODULE_GENERATION		0
#define CLR20_TMODULE_NAME			1
#define CLR20_TMODULE_MVID			2
#define CLR20_TMODULE_ENCID			3
#define CLR20_TMODULE_ENCBASEID			4
#define CLR20_TMODULEREF_NAME			0
#define CLR20_TNESTEDCLASS_NESTEDCLASS		0
#define CLR20_TNESTEDCLASS_ENCLOSINGCLASS	1
#define CLR20_TPARAM_FLAGS			0
#define CLR20_TPARAM_SEQUENCE			1
#define CLR20_TPARAM_NAME			2
#define CLR20_TPROPERTY_FLAGS			0
#define CLR20_TPROPERTY_NAME			1
#define CLR20_TPROPERTY_TYPE			2
#define CLR20_TPROPERTYMAP_PARENT		0
#define CLR20_TPROPERTYMAP_PROPERTYLIST		1
#define CLR20_TSTANDALONGSIG_SIGNATURE		0
#define CLR20_TTYPEDEF_FLAGS			0
#define CLR20_TTYPEDEF_TYPENAME			1
#define CLR20_TTYPEDEF_TYPENAMESPACE		2
#define CLR20_TTYPEDEF_EXTENDS			3
#define CLR20_TTYPEDEF_FIELDLIST		4
#define CLR20_TTYPEDEF_METHODLIST		5
#define CLR20_TTYPEREF_RESOLUTIONSCOPE		0
#define CLR20_TTYPEREF_TYPENAME			1
#define CLR20_TTYPEREF_TYPENAMESPACE		2
#define CLR20_TTYPESPEC_SIGNATURE		0
</source>

==Displaying The Method Definition Table==
Now that you have located the PE32 headers which allowed you to find the CLR run-time header which you then used to
read the meta-data streams and finally the meta-data tables from the #~ stream you can now display the method definition
table.

I wrote a utility function to access a row and field in each table. The function works like below:
<source lang="C">
	win_clr20_meta_table	*tmp;

	tmp = &meta->tables[CLR20_META_TMETHODDEF];
	for(x = 0; x < tmp->rowcnt; ++x)
	{
		td = tmp->rows + (x * tmp->rsize);
		cilvm_clr_treadrowfield(tmp, x, CLR20_TMETHODDEF_RVA, &z);
		printf("rva:%x ", z);
		cilvm_clr_treadrowfield(tmp, x, CLR20_TMETHODDEF_IMPLFLAGS, &z);
		printf("implflags:%x ", z);
		cilvm_clr_treadrowfield(tmp, x, CLR20_TMETHODDEF_FLAGS, &z);
		printf("flags:%x ", z);
		cilvm_clr_treadrowfield(tmp, x, CLR20_TMETHODDEF_NAME, &z);
		printf("Name:%x ", z);
		cilvm_clr_treadrowfield(tmp, x, CLR20_TMETHODDEF_SIGNATURE, &z);
		printf("Signature:%x ", z);
		cilvm_clr_treadrowfield(tmp, x, CLR20_TMETHODDEF_PARAMLIST, &z);
		printf("ParamList:%x ", z);
		printf("\n");
	}
</source>
But, looks like this:
<source lang="c">
int cilvm_clr_treadrowfield(win_clr20_meta_table *table, uint32_t rndx, uint32_t fndx, void *ret)
/*
	This will read a field and row both specified by an index starting from zero.
	
	To check this function for success you should ensure that the return value is non-zero. This
	function may fail if you try to access past the last row or last field. Or, somehow pass a
	negative index.
*/
{
	uint8_t		x;
	uint8_t		*o;
	
	if(rndx >= table->rowcnt)
	{
		return 0;
	}
	o = table->rows + (table->rsize * rndx);

	if(fndx >= table->fcnt)
	{
		return 0;
	}
	for(x = 0; x < fndx; ++x)
	{
		o += table->fsize[x];
	}
	
	memcpy(ret, o, table->fsize[fndx]);
	return 1;
}
</source>

==Display Name And Signature For Each Method Defintion==
You can also easily display the method definition name, signature, and find the chunk (section) by getting
a pointer to where the method is located in the chunk.
<source lang="C">
	uint32_t				x;
	uint32_t				y;
	uint32_t				z;
	win_clr20_meta_table	*tmp;
	uint8_t					*mptr;
	
	tmp = &clrmodule->meta.tables[CLR20_META_TMETHODDEF];
	for(x = 0; x < tmp->rowcnt; ++x)
	{	
		/* find the name */
		cilvm_clr_treadrowfield(tmp, x, CLR20_TMETHODDEF_NAME, &z);
		mptr = clrmodule->meta.strmbyid[WIN_CLR20_META_STRINGS].data;
		printf("%s\n", &mptr[z]);
		
		/* find the signature */
		cilvm_clr_treadrowfield(tmp, x, CLR20_TMETHODDEF_SIGNATURE, &z);
		mptr = clrmodule->meta.strmbyid[WIN_CLR20_META_BLOB].data;
		printf("siglen:%x\nsig:", mptr[z]); 
		for(y = 0; y < mptr[z]; ++y)
		{
			printf("%02x-", mptr[z + 1 + y]);
		}
		printf("\n");
		
		/* find the chunk that holds the method */
		cilvm_clr_treadrowfield(tmp, x, CLR20_TMETHODDEF_RVA, &z);
		if(cilvm_clr_getchunkptrbyrva(clrmodule, z, (void**)&mptr))
		{
			printf("found chunk holding method specified by rva\n");
		}else{
			printf("failed to find chunk holding rva specified by method.\n");
		}
	}	
</source>

==Display Type Names And Namespaces With Fields==
<source lang="c">
int test2(win_clr20_module *clrmodule)
{
	win_clr20_meta_table	*mtp_typedef;
	win_clr20_meta_table	*mtp_field;
	uint32_t		x;	
	uint32_t		y;
	uint32_t		z;
	uint32_t		w;
	uint32_t		tdef_flags;
	uint8_t			*tdef_typename;
	uint8_t			*tdef_typenamespace;
	uint32_t		tdef_fieldlist;
	uint32_t		tdef_methodlist;
	uint8_t			*clrm_strings;
	uint16_t		tfield_flags;
	uint8_t			*tfield_name;
	uint8_t			*tfield_signature;

	clrm_strings = clrmodule->meta.strmbyid[WIN_CLR20_META_STRINGS].data;
	mtp_typedef = &clrmodule->meta.tables[CLR20_META_TTYPEDEF];
	mtp_field = &clrmodule->meta.tables[CLR20_META_TFIELD];

	for(x = 0; x < mtp_typedef->rowcnt; ++x)
	{
		tdef_typename = 0;
		tdef_typenamespace = 0;
		tdef_fieldlist = 0;
		cilvm_clr_treadrowfield(mtp_typedef, x, CLR20_TTYPEDEF_FLAGS, &tdef_flags);
		cilvm_clr_treadrowfield(mtp_typedef, x, CLR20_TTYPEDEF_TYPENAME, &tdef_typename);
		cilvm_clr_treadrowfield(mtp_typedef, x, CLR20_TTYPEDEF_TYPENAMESPACE, &tdef_typenamespace);
		cilvm_clr_treadrowfield(mtp_typedef, x, CLR20_TTYPEDEF_FIELDLIST, &tdef_fieldlist);
		cilvm_clr_treadrowfield(mtp_typedef, x, CLR20_TTYPEDEF_METHODLIST, &tdef_methodlist);
		tdef_typename = &clrm_strings[(uintptr_t)tdef_typename];
		tdef_typenamespace = &clrm_strings[(uintptr_t)tdef_typenamespace];
		printf(".type %s.%s\n", tdef_typenamespace, tdef_typename);
		/* figure out where the fields stop for this one */
		if((x + 1) < mtp_typedef->rowcnt)
		{
			/* 
			   According to ECMA-335-4th-S22.37, the field list for this type
			   stops where the field list for the next type def starts, so
			   look ahead one..
			*/
			z = 0;
			cilvm_clr_treadrowfield(mtp_typedef, x + 1, CLR20_TTYPEDEF_FIELDLIST, &z);
		}else{
			/* This is the last type definition. So read until last row. */
			z = mtp_field->rowcnt + 1;
		}
		/* read the fields for this type */
		printf("tdef_fieldlist:%x\n", tdef_fieldlist);
		for(y = tdef_fieldlist - 1; y < (z - 1); ++y)
		{
			tfield_flags = 0;
			tfield_name = 0;
			tfield_signature = 0;
			cilvm_clr_treadrowfield(mtp_field, y, CLR20_TFIELD_FLAGS, &tfield_flags);
			cilvm_clr_treadrowfield(mtp_field, y, CLR20_TFIELD_NAME, &tfield_name);
			cilvm_clr_treadrowfield(mtp_field, y, CLR20_TFIELD_SIGNATURE, &tfield_signature);
			tfield_name = &clrm_strings[(uintptr_t)tfield_name];
			printf(".field[%u] %s\n", y, tfield_name);
			
		}
	}

	return 1;
}
</source>
````

<a id="w002076"></a>

## W002076 — User talk:Kmcguire

Bron-ID: W002076
Namespace: User talk (3)
Revisie: 3992 | Datum: 2007-05-27T23:42:09Z
Bron: https://wiki.osdev.org/User_talk:Kmcguire
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AKmcguire&oldid=3992

````text
I like using a template for the [[Projects]] page. Good work. -[[User:Jhawthorn|Jhawthorn]] 18:42, 27 May 2007 (CDT)
````

<a id="w002077"></a>

## W002077 — Template:CloserInspectionIncomprehensible

Bron-ID: W002077
Namespace: Template (10)
Revisie: 12735 | Datum: 2012-02-19T20:32:46Z
Bron: https://wiki.osdev.org/Template:CloserInspectionIncomprehensible
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ACloserInspectionIncomprehensible&oldid=12735

````text
** [[Image:Icon_magnify_warning.png]] '''Failure at comprehension of this entry.;'''<br \> ''{{{1}}}''
````

<a id="w002079"></a>

## W002079 — TSS

Bron-ID: W002079
Namespace: Main (0)
Revisie: 4009 | Datum: 2007-05-28T05:41:05Z
Bron: https://wiki.osdev.org/TSS
Revisie-URL: https://wiki.osdev.org/index.php?title=TSS&oldid=4009
Doorverwijzing: Task State Segment

````text
#REDIRECT [[Task State Segment]]
````

<a id="w002080"></a>

## W002080 — User:Jhawthorn/TLB

Bron-ID: W002080
Namespace: User (2)
Revisie: 30213 | Datum: 2026-03-13T12:01:52Z
Bron: https://wiki.osdev.org/User:Jhawthorn/TLB
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJhawthorn%2FTLB&oldid=30213

````text
The '''Translation Lookaside Buffer''' (TLB) is a cache used to speed up virtual address translation.

==TLB Flush==
There are certain instances which will cause the TLB will be flushed. That is, some or all of the entries will be invalidated.

These cases will cause full TLB flush, emptying all pages from the cache:
* modifying the PE or PG flags in CR0
* modifying the PSE, PGE, or PAE flags in CR4
* writing to an MTTR

These cases will cause a partial TLB flush, where everything but global pages are empty:
* any ''write'' to CR3
* hardware task switches which ''change'' CR3

A single page can be invalidated through the <tt>[[Paging#INVLPG|INVLPG]]</tt> instruction

==See Also==
===Threads===
*[[Topic:13988 | Cost of enabling and disabling pages]]

<!-- [[Category:x86 CPU]] -->
````

<a id="w002082"></a>

## W002082 — Talk:Bootable CD

Bron-ID: W002082
Namespace: Talk (1)
Revisie: 4031 | Datum: 2007-05-30T15:26:05Z
Bron: https://wiki.osdev.org/Talk:Bootable_CD
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABootable_CD&oldid=4031

````text
I think that this page needs to be re-written for the El-torito standard, or maybe have booting with El-torito added on. -Steve the Pirate
:This method already creates an El-Torito CD, so why you are requesting a rewrite is beyond me - [[User:Combuster|Combuster]] 10:26, 30 May 2007 (CDT)
````

<a id="w002083"></a>

## W002083 — User talk:Xyz1/Describe new OS project

Bron-ID: W002083
Namespace: User talk (3)
Revisie: 4029 | Datum: 2007-05-30T15:13:07Z
Bron: https://wiki.osdev.org/User_talk:Xyz1/Describe_new_OS_project
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AXyz1%2FDescribe_new_OS_project&oldid=4029

````text
''...kevin slowly looks around...then steps out and asks...'' - What sort of interesting and magical land is this? --[[User:Kmcguire|Kevin]] 10:13, 30 May 2007 (CDT)
````

<a id="w002084"></a>

## W002084 — Talk:GRUB Legacy

Bron-ID: W002084
Namespace: Talk (1)
Revisie: 19450 | Datum: 2016-07-14T15:13:01Z
Bron: https://wiki.osdev.org/Talk:GRUB_Legacy
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AGRUB_Legacy&oldid=19450

````text
Could someone add a heading about loading modules and using them in your kernel please?--[[User:MessiahAndrw|MessiahAndrw]] 04:57, 1 June 2007 (CDT)


Would anyone disagree with moving the '''GRUB HOWTO''' section into a separate article in the tutorial section? -[[User:Jhawthorn|Jhawthorn]] 17:01, 1 June 2007 (CDT)


== Grub for EFI ==

This section had been added underneath the external links section and so I have moved it in to the main body of the page. This seems quite a major section and I almost wonder if it could do with its own page? No time to do this at present but what do you think - should it stay here or be moved elsewhere? --[[User:Aj|Aj]] 20:41, 2 September 2011 (UTC)
````

<a id="w002085"></a>

## W002085 — Template:Eq

Bron-ID: W002085
Namespace: Template (10)
Revisie: 28982 | Datum: 2024-06-06T17:23:28Z
Bron: https://wiki.osdev.org/Template:Eq
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AEq&oldid=28982

````text
{{Eq1|_reserved={{{1}}}|{{{2}}}=1}}
````

<a id="w002086"></a>

## W002086 — Template:Eq1

Bron-ID: W002086
Namespace: Template (10)
Revisie: 12737 | Datum: 2012-02-19T20:33:37Z
Bron: https://wiki.osdev.org/Template:Eq1
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AEq1&oldid=12737

````text
{{{{{{_reserved}}}|}}}
````

<a id="w002087"></a>

## W002087 — Template:Month

Bron-ID: W002087
Namespace: Template (10)
Revisie: 4075 | Datum: 2007-06-08T20:43:45Z
Bron: https://wiki.osdev.org/Template:Month
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AMonth&oldid=4075

````text
{{If|{{Eq|{{{1}}}|1}}|January}}{{If|{{Eq|{{{1}}}|2}}|February}}{{If|{{Eq|{{{1}}}|3}}|March}}{{If|{{Eq|{{{1}}}|4}}|April}}{{If|{{Eq|{{{1}}}|5}}|May}}{{If|{{Eq|{{{1}}}|6}}|June}}{{If|{{Eq|{{{1}}}|7}}|July}}{{If|{{Eq|{{{1}}}|8}}|August}}{{If|{{Eq|{{{1}}}|9}}|September}}{{If|{{Eq|{{{1}}}|10}}|October}}{{If|{{Eq|{{{1}}}|11}}|November}}{{If|{{Eq|{{{1}}}|12}}|December}}
````

<a id="w002088"></a>

## W002088 — Template:IntelManualURL

Bron-ID: W002088
Namespace: Template (10)
Revisie: 30067 | Datum: 2026-02-23T22:00:28Z
Bron: https://wiki.osdev.org/Template:IntelManualURL
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AIntelManualURL&oldid=30067

````text
https://pdos.csail.mit.edu/6.828/2018/readings/i386/{{if|{{{2|}}}|s{{{1}}}_{{{2}}}|c{{{1}}}}}.htm
````

<a id="w002089"></a>

## W002089 — Template:IntelManualLink

Bron-ID: W002089
Namespace: Template (10)
Revisie: 4089 | Datum: 2007-06-17T07:14:47Z
Bron: https://wiki.osdev.org/Template:IntelManualLink
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AIntelManualLink&oldid=4089

````text
[{{IntelManualURL|{{{1|}}}|{{{2|}}}}} Intel 80386 Reference Programmer's Manual Chapter {{{1}}} {{if|{{{2|}}}|Section {{{2}}}}}]
````

<a id="w002090"></a>

## W002090 — Advanced Integrated Peripheral

Bron-ID: W002090
Namespace: Main (0)
Revisie: 19821 | Datum: 2016-09-17T15:43:43Z
Bron: https://wiki.osdev.org/Advanced_Integrated_Peripheral
Revisie-URL: https://wiki.osdev.org/index.php?title=Advanced_Integrated_Peripheral&oldid=19821

````text
Advanced Integrated Peripheral (AIP) is a multi-chip that provides the functionality of several chips in one chip on AT-Compatible PCs.

An example of this kind of chip is Intel 82091AA.

== See Also ==

=== Emulated Chipsets ===
* [[Floppy Disk Controller]] (8272A/82078/82077SL/82077AA)
* [[UART]] (8250/8250A/8250B/8250/82450/16450/16550/16550A/16550AF/16550AFN)
* [[Parallel port]] (IEEE Standard 1284) SPP/EPP/ECP
* [[:Category:ATA|ATA]]/IDE (Advanced Technology Attachment or Integrated Drive Electronics)
* [[DMA]] (8237)

=== Related Chips ===
* [[PS2 Keyboard|Keyboard Controller]] (8042)
* [[PIT]]
* [[PIC]]
* [[APIC]]
* [[OPIC]]
* [[APM]]
* [[ISA]]

=== External Links ===
* [http://www.intel.com/design/archives/periphrl/docs/29048603.htm Intel 82091AA Datasheet]
* [http://www.intel.com/design/archives/periphrl/docs/7549.htm Intel Technical Questions and Answers]

[[Category:Common Devices]]
````

<a id="w002091"></a>

## W002091 — UART

Bron-ID: W002091
Namespace: Main (0)
Revisie: 23885 | Datum: 2019-09-02T12:28:59Z
Bron: https://wiki.osdev.org/UART
Revisie-URL: https://wiki.osdev.org/index.php?title=UART&oldid=23885

````text
A universal asynchronous receiver-transmitter (UART) is a hardware device for asynchronous serial communication commonly found in personal computers and embedded devices. The term UART refers specifically to the hardware responsible for the transmission and receiving of data over a serial port interface, the most well-known implementation of which is the [[Serial_Ports|serial port]] common to x86 computers. This hardware is usually found in the form of an IC integrated onto a motherboard or system-on-chip.

This article provides a theoretical introduction to the topic of UARTs and their relevance to operating system development. It does not intend to provide a comprehensive guide to programming any specific implementation, rather to introduce the reader to a high-level understanding of the topic of UART hardware and their use in serial communication.

== Common Implementations ==
The most common implementation of a UART is the [[wikipedia:16550_UART|16650 UART]], developed in 1987 by National Semiconductor. It is frequently used to implement the serial port interface for [[wikipedia:IBM_Personal_Computer|IBM PC]] compatible personal computers, where it is commonly connected to an [[wikipedia:RS-232|RS-232]] port. The implementation in [[x86|x86/64]] is known as the COM interface, which is accessible via standard [[I/O_Ports|IO Ports]] mapped to addresses from <tt>0x3F8</tt> through <tt>0x3FF</tt>.

Hardware serial interfaces can be found on nearly every personal computer manufactured prior to 2005. Even systems without physical serial ports will typically include drivers for interfacing with compatible hardware.
For the purposes of operating system development on x86 computers, the 16550 UART is of particular relevance. Its support is nearly guaranteed in PC emulators, as well as being particularly well documented and easy to develop drivers for.
Most consumer embedded systems feature an integral UART or USART implementation, often accessible via a GPIO interface. UART implementations in embedded systems vary somewhat between vendors, however implementations tend to be reasonably mutually intelligible.

UART hardware typically includes a clock generator responsible for regulating the [[wikipedia:Symbol_rate|baud rate]] of communications. As well as multiple memory-mapped registers, commonly referred to as 'shift registers', responsible for holding data to be transmitted or received.
For a more detailed description of an individual hardware implementation, refer to its datasheet. A datasheet explaining the individual hardware components of the 16550D UART manufactured by Texas Instruments can be obtained [http://www.ti.com/product/PC16550D here].

== Serial Communication ==
Serial communication is facilitated by the UART hardware taking bytes of data and transmitting individual bits in a sequential fashion, hence the use of the term 'serial'. At the destination, the receiving UART reassembles the stream of bits into the complete data payload. This is facilitated by the use of the aforementioned 'shift registers', which will typically interface with the driver via the generation of [[interrupts]].

''Note: What follows is a technical explanation of the line protocol used by UART devices to transmit and receive data. In depth knowledge of the electrical characteristics of UART communication is not necessary for the implementation of UART drivers, but is useful background knowledge.''

The UART hardware is configured to transmit and receive at a predetermined [[wikipedia:Symbol_rate|baud rate]]. This value corresponds to the number of bits that are transmitted across the line per second. Both the sender and receiver UART will need to use the same UART to ensure that communication is synchronised. Configuration of the baud rate is typically accomplished via populating a 'clock divisor' register to limit the baud rate to a quotient of the device's maximum. For example: if the device has a maximum baud rate of 115200 ( A common default value ), populating the clock divisor register with a divisor of 1 will give 115200 baud, a divisor of 2 will give 57600 baud, 3 will give 38400 baud, etc.

The idle, no data state on a serial line is typically a high-logic state. Transmission of an individual character over a serial line consists of each character being framed with a logic-low start bit, an arbitrary number of data bits, an optional parity bit and one or more stop bits.
In most UART implementations the least significant bit is transmitted first.

It is important to note that the bit-width of a character in a particular serial line protocol might not necessarily be 8. In most UART implementations this is a configurable setting, which the end-user may tailor to the individual requirements of the system. 

The logic-low start bit signals to the receiver that a new character is pending. The next bits, which will vary in length depending on the configuration, represent the character payload. If a parity bit is used, it will be placed after all of the data bits. The next one or two bits, called the stop bits, are always in the high-logic state. They signal to the receiver that character transmission is complete. These predictable bits can be used by the hardware to test whether the transmitter and receiver are in phase.

If the line is held in the logic low condition for longer than the period necessary to transmit a character, this is a 'break condition' that can be detected by the UART, typically via the generation of an interrupt.

The use of a [[wikipedia:Parity_bit|parity bit]] in UART communication is a method of detecting errors in transmission. When a parity bit is used with a serial port, an extra data bit is sent along with each transmitted character. The parity bit has a logical value corresponding to whether the number of 1's in the binary character being transmitted is even or odd. There are two variants of parity bits: ''even'' and ''odd'' parity bits. In the case of an even parity bit, for a given set of bits, the occurrences of bits whose value is 1 is counted. If that count is odd, the parity bit value is set to 1, making the total count of occurrences of 1's in the whole set (including the parity bit) an even number. If the count of 1's in a given set of bits is already even, the parity bit's value is 0. In the case of an odd parity bit, the value is reversed, so that the value of the parity bit will ensure that the total number of 1's in the transmission is an odd number. The transmitting and receiving UART hardware must have a matching parity bit configuration or their signals will be mutually unintelligible.

The commonly used standard for serial configuration is known as [[wikipedia:8-N-1|8N1]]: eight (8) data bits, no (N) parity bit, and one (1) stop bit.

== Uses ==
Prior to the 2000s, serial ports were a common way for personal computers to interface with external consumer peripherals. Despite being more-or-less entirely superseded by [[USB]] for communicating with peripherals in personal computers, UART remains in more common use in embedded systems for communicating with hardware peripherals.

Serial programming is of special relevance to operating system development, as it provides an easy to implement and robust interface between an operating system and external logging mechanisms. In systems where video hardware may not be present, or may not be suitable for the printing of debugging information, a serial driver can easily be implemented to fulfill this need. Serial port interfaces to the host operating system are widely available in common PC emulators such as [[QEMU]] and [[bochs]].

== See Also ==
=== Related Articles ===
[[Serial_Ports|Serial Ports]]
=== Related Links ===
* [http://en.wikibooks.org/wiki/Serial_Programming A Wikibook about serial device programming]

[[Category:Network Hardware]]
[[Category:Common Devices]]
````

<a id="w002092"></a>

## W002092 — Floppy Drive Controller

Bron-ID: W002092
Namespace: Main (0)
Revisie: 4094 | Datum: 2007-06-17T11:32:14Z
Bron: https://wiki.osdev.org/Floppy_Drive_Controller
Revisie-URL: https://wiki.osdev.org/index.php?title=Floppy_Drive_Controller&oldid=4094
Doorverwijzing: Floppy Disk Controller

````text
#REDIRECT [[Floppy Disk Controller]]
````

<a id="w002093"></a>

## W002093 — Talk:Floppy Drive Controller

Bron-ID: W002093
Namespace: Talk (1)
Revisie: 4096 | Datum: 2007-06-17T11:32:14Z
Bron: https://wiki.osdev.org/Talk:Floppy_Drive_Controller
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AFloppy_Drive_Controller&oldid=4096
Doorverwijzing: Talk:Floppy Disk Controller

````text
#REDIRECT [[Talk:Floppy Disk Controller]]
````

<a id="w002094"></a>

## W002094 — Parallel port

Bron-ID: W002094
Namespace: Main (0)
Revisie: 30349 | Datum: 2026-03-18T04:22:33Z
Bron: https://wiki.osdev.org/Parallel_port
Revisie-URL: https://wiki.osdev.org/index.php?title=Parallel_port&oldid=30349

````text

The '''IEEE 1284 parallel port''' is an external 8-bit parallel computer bus common from the late 1970s to early 2000s for connecting printers, scanners, and external storage devices to home and office computers (functioning as a much cheaper, slower and worse alternative to [[SCSI]]). The IEEE 1284 standard defines 5 operating modes for this interface, as well as 3 different connectors.

The parallel port is also referred to as the '''Centronics port''' (after the first manufacturer to implement the interface) or '''Line Printer Terminal (LPT)''' port.

== IEEE 1284 Connectors ==

There are three different connectors:
* 25-pin DB-25 D-Sub Connector (Common on PC Side)
* 36-pin 'Centronics' Connector (Common on Printer Side)
* 36-pin Miniature Connector (Very Rare)

For the purposes of programming the parallel port, the connector used is irrelevant.

== IEEE 1284 Operating Modes ==

* '''Standard Parallel Port (SPP)''' - Also referred to as '''Compatibility Mode''' in IEEE 1284.
* '''Nibble Mode'''
* '''Bi-Directional Mode'''
* '''Enhanced Parallel Port (EPP)'''
* '''Extended Capabilities Port (ECP)'''

==Registers & Common Address==
Common base addresses are:
* LPT1: 0x378 (or occasionally 0x3BC) (IRQ 7)
* LPT2: 0x278 (IRQ 6)
* LPT3: 0x3BC (IRQ 5)

The [[Memory Map (x86)#BIOS_Data_Area_.28BDA.29|BIOS Data Area]] should contain the IO base addresses of any existing parallel ports

==Parallel Port Software Interface==
{{Stub}}

{{NoteBox|<b>NOTE:</b> Before doing anything, make sure to set the parallel ports mode to Standard or Normal in the BIOS instead of ECP/EPP if anything fails from your programming efforts, preferably before exhausting your options}}


Each parallel port has three IO port registers, Data, Status and Control.  Their addresses are relative to the base address of the parallel port in question.

===Data Register===
<tt>Address = Base Address + 0</tt>

Any byte written to this register is put on pins 2 through 9 of the port.
Any read from this register reflects the state of the port.

===Status Register===
<tt>Address = Base Address + 1</tt>
{| class="wikitable"
|+
! 7 || 6 || 5 ||4 || 3 || 2 || 1 || 0
|-
| Busy || ACK || Paper Out || Select In || Error || [[IRQ]] || Reserved || Reserved
|}

The ''Error'', ''ACK'' and ''Busy'' signals are active low when reading from the IO port.

===Control Register===
Address = Base Address + 2
{| class="wikitable"
|+
! 7 || 6 || 5 ||4 || 3 || 2 || 1 || 0
|-
| Unused || Unused || BIDI || IRQACK || Select || Initialize || Auto LF || Strobe
|}

The ''Initialize'' signal is active low when writing to the IO port.

The ''Strobe'' signal is for handshaking and alerts the printer to data being ready at the data port.

''Auto LF'' is the Automatic Line-Feed signal. If this is set and the printer receives a Carriage-Return character (<tt>0x0D</tt>), the printer will automatically perform a Line-Feed (character <tt>0x0A</tt>) as well.

''Initialize'', sometimes called ''Prime'', alerts the device that data transfer is about to start. This signal may result in a printer performing a reset and buffers being flushed.

==Standard Parallel Port Mode==

This is the most basic of the parallel modes.  The other modes are EPP and ECP.  All systems should support this mode and it may well be the default at boot time.  Some BIOSes also support setting the default mode of the parallel port.  In this mode, the Data register and the Control register are Write-Only and the Status register is Read-Only.

In Standard (or Compatibility) mode, data is sent using a mechanism called Centronics Handshaking, described below.

This mode requires communuication handshaking to be performed by software which limits the maximum data throughput of the port.  This means that a standard mode has a maximum transfer rate of around 1000 bytes per second, depending on the timings of the computer and receiving device.  The more advanced types (or modes) of parallel ports, EPP and ECP, reduce this by providing hardware-based handshaking.  Relieving software of this requirement reduces CPU load and increases the port's maximum potential speed.

For a line printer, this method should be enough to get things going, simply sending characters using this method to the parallel port while the printer is online should get the print head moving, assuming no buffers are in the way to store the values.

Depending on the connected device, you may have to raise the INITIALISE signal before data transmission to ready the device, and possibly again after in order to flush any buffers.

===Centronics Handshaking===

In Standard (or Compatibility) mode, data is sent to the connected device by writing the byte to the data port, then pulsing the STROBE signal.  This pulse informs the device that data is ready to be read.  The device will respond by raising its BUSY signal and then reading the data and performing some processing on it.  Once this processing is complete, the device will lower the Busy signal and may raise a brief ACK signal to indicate that it has finished.

<syntaxhighlight lang="c">
 // Sends a byte to the printer
void Parallel_SendByte( unsigned char pData )
{
	unsigned char lControl;
 
 	// Wait for the printer to be receptive
 	while ( ! inb( 0x379 ) & 0x80 )
 	{
 		Timer_Delay( 10 );
 	}
 
 	// Now put the data onto the data lines
 	outb( 0x378, pData );
 
 	// Now pulse the strobe line to tell the printer to read the data
 	lControl = Ports_In8( 0x37A);
 	outb( 0x37A, lControl | 1 );
 	Timer_Delay( 10 );
 	outb( 0x37A, lControl );
 
 
	// Now wait for the printer to finish processing
	while ( ! inb( 0x379 ) & 0x80 )
	{
		Timer_Delay( 10 );
	}
}
</syntaxhighlight>

<tt>Timer_Delay()</tt> pauses processing for the specified number of milliseconds. <tt>inb()</tt> and <tt>outb()</tt> read and write a byte of data to/from the IO port.  You can find the IO functions in the [[Inline_Assembly/Examples#OUTx|Inline Assembly Examples page]].



[http://devel.archefire.org/TextRecordings/lpthandler.tar <b><code>lpthandler.tar:</code></b> Code and binary for "LPT Handler for Windows"]

[http://www.youtube.com/watch?v=sJRbGohmA3M <b>YouTube video:</b> Basics of Parallel Port Programming Under Windows, Part 1 of 2]

[http://www.youtube.com/watch?v=zAxeCLuWZCk <b>YouTube video:</b> Basics of Parallel Port Programming Under Windows, Part 2 of 2]

[[File:Win32_lpthandler_exe_0000.jpg]]

<syntaxhighlight lang="c">
/*
 In short we only need:

 outl(LPT_Base_Address+2, 0);          //Reset Control port with 0
 outl(LPT_Base_Address,   0xFF-0x00);  //Write Data Port with any byte value
*/



//Here the LPT_Base_Address can be:
//
//          0x3BC -- LPT1
//          0x378 -- LPT1
//          0x278 -- LPT2
//          0x3BC -- LPT3
///

//We need to reset the Control Port writing a 0
//to configure their options, controlled for each of
//their bits:
///
outl(LPT_Base_Address+2, 0);

//Now send an value betwewen 0 and 255 to the data port:
///
outl(LPT_Base_Address, 0xFF-0x00);
</syntaxhighlight>

== See Also ==
=== Threads ===
* [[Topic:30279|Easy Parallel Port Programming Tests (Under Windows)]]



[[Category:Common Devices]]
[[Category:Hardware Interfaces]]
````

<a id="w002095"></a>

## W002095 — Keyboard Controller

Bron-ID: W002095
Namespace: Main (0)
Revisie: 17781 | Datum: 2015-03-11T18:35:02Z
Bron: https://wiki.osdev.org/Keyboard_Controller
Revisie-URL: https://wiki.osdev.org/index.php?title=Keyboard_Controller&oldid=17781
Doorverwijzing: PS/2 Keyboard

````text
#REDIRECT [[PS/2 Keyboard#Commands]]
````

<a id="w002097"></a>

## W002097 — VirtualBox

Bron-ID: W002097
Namespace: Main (0)
Revisie: 23255 | Datum: 2019-01-23T18:58:56Z
Bron: https://wiki.osdev.org/VirtualBox
Revisie-URL: https://wiki.osdev.org/index.php?title=VirtualBox&oldid=23255

````text
{{Emulators}}

'''VirtualBox''' is a virtual machine monitor produced by Oracle (previously Sun Microsystems). It is largely open-source (GPL) with a few feature packs that are closed source. It is very easy to use and has good support for many host and guest platforms.

==Features==
*It has very good documentation so it is very easy to use out of the box.
*VirtualBox supports booting from real media or images of floppies and CD/DVD-ROM's.
*It also [http://www.virtualbox.org/manual/ch12.html#id356590 supports debugging].
*The ability to access host USB drives.

You are able to also use features of the host operating system where applicable. Some features make using the guest OS easier. For example, "pointer integration" (which allows you to move the cursor between the host OS and certain guest OS's seamlessly).

==64-bit guest==
To run a 64-bit guest OS on a 32-bit or 64-bit host, the CPU ''must'' support virtualization (AMD-V or Intel VT-x) and nested paging (AMD RVI or Intel EPT), and must have these features enabled in the BIOS and the VirtualBox machine configuration. The VirtualBox developers didn't implement software virtualization for 64-bit hosts since most modern CPUs support hardware virtualization, and sofware virtualization would be too slow and memory consuming (especially on 32-bit hosts). If your CPU doesn't support virtualization or nested paging, then you can use [[QEMU]], [[Bochs]] or [[VMware|VMware Player]] as these support 64-bit guests without hardware virtualization.

To see which virtualization features are present and enabled for VirtualBox, run your virtual machine and hover with the mouse over the V icon in the bottom bar.

==Virtualized Hardware==
VirtualBox supports the virtualization of the following hardware:

===Audio===
*[[AC97|Intel AC'97]]
*[[Intel High Definition Audio|Intel HD Audio]]
*[[Sound Blaster 16|Creative Sound Blaster 16]]

===Chipset===
*Intel PIIX3
*Intel ICH9

===Networking===
*[[AMD PCNET|AMD PCNet PCI II (AM79C970A)]]
*[[AMD PCNET|AMD PCNet FAST III (AM79C973)]]
*[[Intel 8254x|Intel PRO/1000 MT Desktop (82540EM)]]
*[[Intel 8254x|Intel PRO/1000 T Server (82543GC)]]
*[[Intel 8254x|Intel PRO/1000 MT Server (82545EM)]]
*Paravirtualized networking (through [[KVM|KVM's]] 'virtio' networking drivers)

==Guest Additions==
{{Main|VirtualBox_Guest_Additions}}
VirtualBox offers many features to integrate the user experience of the guest operating system more easily into the host operating system. These features include;
* Shared drag and drop capability.
* Mounting shared directories on the host in the client.
* Seamless mouse pointer integration.
* Seamless window blending.
* Shared clipboard.
* Detecting the size of the VirtualBox window.
* 2D and 3D acceleration.
All of the official VirtualBox documentation either targets end-users installing guest operating systems or people developing VirtualBox. Documentation for people developing for VirtualBox's virtual hardware is virtually non existent.
The best reference you have available is the source code for the guest additions for FreeBSD, Linux, OS2, Solaris, and Windows. The source code can be found on the VirtualBox SVN respository under [https://www.virtualbox.org/browser/vbox/trunk/src/VBox/Additions/ /vbox/trunk/src/VBox/Additions].

The ability to enable certain guest features depends on the value of the "Guest OS" property. For example, the option to enable 3d acceleration doesn't become available unless the "Guest OS" property is set to Windows. Changing the "Guest OS" merely changes the recommended settings and what extended features are available; it does not change the behaviour of the virtual machine.

==See Also==
===External Links===
*[http://www.virtualbox.org VirtualBox HomePage]
*[[Wikipedia:VirtualBox|Wikipedia page on VirtualBox]]

===Issues===
*[http://forum.osdev.org/viewtopic.php?t=24932 User turdus identified a bug in long mode]
*More recently, [http://9front.org/ 9front] users found strange behaviour: Some emulated hardware (e.g. NIC, important to 9front,) works in one revision, but not in the next.  (E.g. it works in version X.Y.39 but not X.Y.87.)  9front works fine with real hardware of the same type.

[[Category:Emulators]]
[[de:VirtualBox]]
````

<a id="w002098"></a>

## W002098 — Talk:VirtualBox

Bron-ID: W002098
Namespace: Talk (1)
Revisie: 13259 | Datum: 2012-04-27T22:02:20Z
Bron: https://wiki.osdev.org/Talk:VirtualBox
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AVirtualBox&oldid=13259

````text
how do i get rid of the odd formatting?
:Thank for for adding another useful tool to the wiki. The strange formatting existed because you added spaces before each paragraph. I have fixed this and changed the formatting so it conforms to the style of the other articles in the wiki. -[[User:Jhawthorn|Jhawthorn]] 17:21, 25 June 2007 (CDT)

Is there any information on VirtualBox's guest hardware? For example accelerated video, finding the size of the VirtualBox screen, synchronizing clipboards, shared folders, etc? I've looked on the VirtualBox website and the manuals are targeted to end users or to VirtualBox developers, but not for OS developers. --[[User:Messiahandrw|MessiahAndrw]] 17:02, 27 April 2012 (CDT)
````

<a id="w002100"></a>

## W002100 — MASM

Bron-ID: W002100
Namespace: Main (0)
Revisie: 22961 | Datum: 2018-09-25T20:28:49Z
Bron: https://wiki.osdev.org/MASM
Revisie-URL: https://wiki.osdev.org/index.php?title=MASM&oldid=22961

````text
The '''Microsoft Macro Assembler''' is an x86 architecture assembler for MS-DOS and Microsoft Windows. While the name MASM has earlier usage as the Unisys OS 1100 Meta-Assembler, it is commonly understood in more recent years to refer to the Microsoft Macro Assembler. It is an archetypal MACRO assembler for the x86 PC market that is owned and maintained by a major operating system vendor and since the introduction of MASM version 6.0 in 1991 has had a powerful preprocessor that supports pseudo high level emulation of variety of high level constructions including loop code, conditional testing and has a semi-automated system of procedure creation and management available if required. Version 6.11d was 32 bit object module capable using a specialised linker available in the WinNT 3.5 SDK but with the introduction of binary patches that upgraded version 6.11d, all later versions were 32 bit Portable Executable console mode application that produced both OMF and COFF object modules for 32 bit code.

'''NOTE: Using MASM for operating system development is not prohibited in the license agreement although you may sometimes hear that. This is because people often confuse the MASM and MASM32 licenses; they are 2 unrelated projects.'''

While recent versions of MASM only come with Visual Studio, its syntax is in widespread use in existing code and is also used as a guideline in the development of other assemblers, such as [[JWASM]] and Pelle's PoAsm assembler.

== History ==
The Microsoft Assembler has been in production since 1981 and is upgraded by Microsoft on a needs basis to reflect technology changes in both operating systems and processor hardware capacity. The copyright string from the 1991 version of ML.EXE is as follows.

    Microsoft (R) Macro Assembler Version 6.00
    Copyright (C) Microsoft Corp '''1981-1991'''. All rights reserved.

The last freestanding commercial version of the Mirosoft assembler was version '''6.11d''' released in 1993 and retailed through the middle to late 1990s. With the release of 32 bit versions of Windows with both the OEM win95 and WinNT version 4.0, Microsoft developed ML.EXE mainly for internal use as an operating system vendor and it was mainly available only through MSDN subscription for the development of device drivers    but Microsoft developed patches for the last commercial version of ML.EXE that upgraded it from a 16 bit MZ executable to a proper 32 bit portable executable file that ran natively on the 32 bit Windows platforms. With the release of the 6.14 patch, ML.EXE became a very reliable tool that supported Intel opcodes up to the early SSE instruction set.

In middle 2000 Microsoft re-integrated ML.EXE back into their VC98 commercial software development package with the processor pack as the downloadable file VCPP5.EXE which was licenced so that licenced end users of VC98 could redistribute the processor pack to other licenced end users of VC98 (VCPP5.EXE EULA) and all versions of Microsoft Visual C and Visual Studios have contained ML.EXE as a component since that time. The ML.EXE version supplied in the VCPP5 pack was ML.EXE version 6.15 which added support for the SSE2 Intel instruction set. Successive versions of ML.EXE have been developed on a needs basis to include later Intel opcodes. Later in Visual C++ 2005, a seperate 64-bit version of MASM appeared under the file name '''ml64.exe'''.

Although MASM is no longer a freestanding commercial product, it has since 2000 been a component of the Microsoft commercial development environment Visual Studio but Microsoft have also made it availabe in many different packages for device development and more recently in the free downloadable versions of Visual Studio.

Version 7.0 was included with Visual C++ .NET 2002. Version 7.1 was included with Visual C++ .NET 2003. Version 8.0 was included with Visual C++ 2005 which also includes a version that can assemble x64 code. Version 9.0 is included with Visual C++ 2008. Some of the newer versions of MASM are also included in various Microsoft SDKs and DDKs.

ML.EXE is typically an internal usage industrial tool maintained by a major operating system vendor to serve their own purpose without the need to make it a particularly "user friendly" application as the vast majority of its users are experienced programmers who have used it for many years. Microsoft have tended to use assembler code in the very low levels of their operating systems where even the best C compilers do not deliver sufficiently optimised code for the intended purpose. This is evident for programmers with enough technical experience who have some need to disassemble occasional OS components for research and compatibility purposes like '''NTOSKRNL.EXE''' and '''HAL.DLL''' and the tell tale indication of code written by hand in ML.EXE is the use of the trailing LEAVE mnemonic at the end of a procedure with a stack frame.

The two current ML versions as of January 2010 are as follows,

2009 ML version copyright string
    Microsoft (R) Macro Assembler Version 9.00.30729.01
    Copyright (C) Microsoft Corporation.  All rights reserved.

2009 ML64 version copyright string<br />
    Microsoft (R) Macro Assembler (x64) Version 9.00.30729.207
    Copyright (C) Microsoft Corporation.  All rights reserved.

==Usage==
The Microsoft assembler has been the main vehicle for preserving the earlier Intel assembler notation and it can still be written as a fully specified language, a format that many dis-assemblers produce. The most common notation of this type are the data size specifiers,

    '''BYTE PTR'''  The data size specifier for the target being 8 bit.
    '''WORD PTR'''  The data size specifier for the target being 16 bit.
    '''DWORD PTR''' The data size specifier for the target being 32 bit.

===Addressing Notation===
ML.EXE maintains the historical distinction between transient stack addressing and fixed data addressing by using the notation '''OFFSET''' to denote data in either the initialised or uninitialised data sections. Transient stack addressing is handled by a number of methods. With a procedure that uses a stack frame, named LOCAL variables are used for readability purposes and where the address of the variable is required it can be accessed eith by the '''LEA''' mnemonic or in an INVOKE call by the '''ADDR''' operator. The '''LOCAL''' variables are [EBP] stack addresses when used within a procedure. Procedures written without a stack frame are generally written purely in mnemonics using direct [ESP] based argument addressing. To maintain compatibility with the historical MASM method of pseudo high level notation, there is a notation to turn off the stack frame generation for the procedure on a needs basis.

===Square Brackets===
ML.EXE does not require the addition of square brackets around a named variable but will tolerate such notation deviation by ignoring the notation. This notation difference has at time been a source of confusion for programmers familiar with other assemblers that use the square brackets to denote an address.

    '''mov eax, local_var'''     ; Standard ML.EXE notation
    '''mov eax, [local_var]'''   ; Square brackets are ignored by ML.EXE in this context.

There is no Intel mnemonic that will produce the extra level of indirection implied by placing un-necessary square brackets around a named variable and the practice leads to confusion of programmers who are experienced in using other x86 assemblers that use the square bracket notation differently. ML.EXE uses square brackets around direct mnemonic code to perform the dereferencing operation as in the following example.

    '''lea eax, variable_name'''   ; load the address of a stack variable into the EAX register
    '''mov eax, [eax]'''           ; dereference the '''CONTENT''' of the variable and copy it into the EAX register

ML.EXE will allow the following.

    '''mov eax, [eax+ebx]'''
    '''mov eax, [eax][ebx]'''

In this context the second pair of square brackets perform the '''ADDITION''' function within the complex addressing notation. It is a useful technique to use when writing procedures that do not have a stack frame as the second pair of brackets can be used to contain the stack displacement that changes with PUSH and POP mnemonics.

===Notation Abreviation===
Over a long period a form of shorthand notation developed as the parsers in early versions improved and generally if the assembler can determine the size of the data the data size specifier is not necessary although it still can be used. This shorthand has confused some users who have used other assemblers which are not by default data size specified tools. It can lead to problems when the user is not familiar with the default data size specifiers while using the shorthand notation.

    movzx eax, [esi]            ; generates an error - data SIZE cannot be determined by the assembler
    movzx eax, '''BYTE PTR''' [esi]   ; zero extend a BYTE into the 32 bit EAX register

===Limited Type Checking===
From at least version 6.0, ML.EXE has supported a pseudo high level notation for creating procedures that perform argument size and count checking. It is part of a system using the '''PROC ENDP PROTO''' and '''INVOKE''' operators. The '''PROTO''' operator is used to define a function prototype that has a matching '''PROC''' that is terminated with the '''ENDP''' operator. The prototyped procedure can then be called with the '''INVOKE''' operator which is protected by the limited size and argument count checking. There is additional notation at a more advanced level for turning off the automatically generated stack frame for the procedure where stack overhead in the procedure call may have an effect with very small procedures. ML.EXE is also capable of being written completely free of the pseudo high level notation using only bare Intel mnemonics.

Using an example prototype from the 32 bit Windows API function set,

    SendMessage '''PROTO''' STDCALL :DWORD,:DWORD,:DWORD,:DWORD
    SendMessage equ <SendMessageA>

The code to call this function using the '''INVOKE''' notation is as follows.

    '''invoke SendMessage,hWin,WM_COMMAND,wParam,lParam'''

Which is translated exactly to,

    '''push lParam'''
    '''push wParam'''
    '''push WM_COMMAND'''
    '''push hWin'''
    '''call SendMessage'''

The advantage of the '''INVOKE''' method is that it tests the size of the data types and the argument count and generates an assembly time error if the arguments do not match the prototype.

Note that ML64.EXE does not currently support the '''INVOKE''' notation and may not in the future. Based off Microsoft's history of updating earlier versions of ML.EXE on a needs basis for their own internal usage, this feature set may not be developed unless they have a need to add it for their own usage.

===Calling Conventions===
ML.EXE supports a number of different calling conventions on both the 16 bit real mode DOS operating system, the 16 bit Windows versions and the later 32 bit versions. ML.EXE supports the '''C''', '''SYSCALL''', '''STDCALL''', '''BASIC''', '''FORTRAN''' and '''PASCAL''' calling conventions.

===Pseudo High Level Emulation===
ML.EXE provides a notation to emulate a variety of high level control and loop structures.<br />
It supports the '''.IF''' block structure,

  '''.if'''
    -
  '''.elseif'''
    -
  '''.else'''
    -
  '''.endif'''

It also supports the '''.WHILE''' loop structure,

  '''.while eax > 0'''
    '''sub eax, 1'''
  '''.endw'''

And the '''.REPEAT''' loop structure.

  '''.repeat'''
    '''sub eax, 1'''
  '''.until eax''' < 1

The high level emulation also supports C runtime comparison operators that work according to the same rules as Intel mnemonic comparisons. For the .IF block notation the distinction between SIGNED and UNSIGNED data is handles with a minor data type notation variation where the storage size '''DWORD''' which is by default UNSIGNED can also be specified as '''SDWORD''' for SIGNED comparison. This data type distinction is only appropriate for the pseudo high level notation as it is unused at the mnemonic level of code where the distinction is determined by the range of conditional evaluation techniques available in the Intel mnemonics.

The combined pseudo high level emulation allows MASM to more easily interface with the later current operating systems that use a C style application programming interface. Generally the pseudo high level interface is used for non-speed critical code where clarity and readability are the most important factors, speed critical code is usually written directly in mnemonics.

Note that ML64.EXE does not support all of the earlier pseudo high level notation and may not in the future. Based off Microsoft's history of updating earlier versions of ML.EXE on a needs basis for their own internal usage, this feature set may not be developed unless they have a need to add it for their own usage.

=== Pre-Processor ===
The Microsoft assembler has a very powerful pre-processor that has considerable more functionality than modern C compilers which is consistent with its designation as a '''macro assembler''' and it has been designed from the introduction of ML.EXE version 6.0 with C style pseudo high level functionality for programmers who prefer to use this style of notation for not speed critical code. On the down side the pre-processor is an old design that is known to be quirky in its operation and reasonably difficult to use without a lot of experience when writing macros that are of a more complex nature.

At their simplest macros written for the ML.EXE pre-processor are useful for automating many different simple tasks.

    ; ----------------------------
    ; memory to memory assignment
    ; ----------------------------
      m2m MACRO M1, M2
        push M2
        pop  M1
      ENDM

    ; --------------------------------------------------
    ; memory to memory assignment using the EAX register
    ; --------------------------------------------------
      mrm MACRO m1, m2
        mov eax, m2
        mov m1, eax
      ENDM

Using the '''EXITM <return item>''' notation a macro can return a value or register in a way that can used similar to a high level function call. Using a very simple example,

    addregs32 MACRO reg1, reg2
      add reg1, reg2
      EXITM <reg1>
    ENDM

In the '''.CODE''' section.

    mov ecx, 16
    mov edx, 8
    
    mov eax, addregs32(ecx, edx)

Which disassembles exactly to the following mnemonics.

    0040102B B910000000             mov     ecx,10h
    00401030 BA08000000             mov     edx,8
    00401035 03CA                   add     ecx,edx
    00401037 8BC1                   mov     eax,ecx

At a slightly more complex level the pre-processor can be used to emulate higher level languages which allows non-critical code to be simplified for higher programming throughput.

    '''fn''' MessageBox,0,'''str$'''(eax),"Title",MB_OK

In this working example '''fn''' is a macro that encapsulates the INVOKE notation and adds functionality so that quoted text can be inserted directly into the API function call in much the same way as a high level language. The '''str$()''' macro is an emulation of traditional basic for converting numeric data as either a memory operand or a register into string data for display. The example is taken from the MASM32 Project main macro file.

== Object Module Compatibility ==
The 32 bit versions of ML.EXE introduced with the patches for the last commercial version onwards produce object modules in both the older OMF format and the Microsoft version of the '''Portable Executable''' specification COFF format. The object module format is compatible with modern Microsoft C compilers and object modules produced by either ML.EXE or CL.EXE can be routinely intermixed and linked into applications written either with ML.EXE or CL.EXE.

=== COFF Object Modules Reference ===
* http://download.microsoft.com/download/e/b/a/eba1050f-a31d-436b-9281-92cdfeae4b45/pecoff.doc
* http://www.microsoft.com/whdc/system/platform/firmware/PECOFF.mspx

== Compatible Linkers ==
For building 16 bit MS-DOS applications a Microsoft OMF linker is required as the 32 bit linkers build 32 bit PE code. The original linkers provided with commercial versions of MASM work correctly and for later versions, Microsoft have had various links over time for their last OMF 16 bit linker that have been available on their website.

== Microsoft Reference For ML.EXE ==

=== Publication ===
* Mirosoft Macro Assembler Programmerss Guide (c) 1991 The Microsoft Corporation. (Supplied with MASM version 6.0)
* Programmer's Guide (c) 1992 The Microsoft Corporation. (Supplied with MASM version 6.1)

=== Online ===
* http://msdn.microsoft.com/en-us/library/afzk3475(VS.71).aspx

== Microsoft Licence ==
MASM has been available over a very long period and has been subject to a number of licencing methods over that time. The different licences break down into two different types, the commercial versions of MASM as either a seperate product or a later component of the Visual C development environment may be used for any purpose including developing non-Microsoft operating system components but almost exclusively the version made available from Microsoft at no cost are restricted to use on Microsoft operating systems and specifically exclude the production of '''Open Source''' code. With the wide range of different licences over the products lifetime the individual licence should be read in detail to determine what it can be used for and what if any restrictions apply in its usage.

== MASM Compatible Assemblers ==
For assembler programmers who are unable to use the Microsoft assembler for licencing reasons there are two directly MASM compatible assemblers that can build almost all MASM code apart from the more complex macros developed under MASM.

* '''Pelle's Macro Assembler''' is a component of the Pelles C development environment and comes complete with detailed documentation supplied with Pelles C. For the assembler language programmer, Pelles C comes with a compatible linker and resource compiler as well as a mature C compiler and an IDE that also contains its own resource editor. It is both mnemonic and operator compatible with MASM but diverges in its macro capacity. It is both 32 bit and 64 bit capable.
* http://www.smorgasbordet.com/pellesc/download.htm

* '''JWASM Macro Assembler''' is a MASM clone that has a very high level of compatibility with the original Microsoft Macro Assembler. It is currently supplied as both source code and a working binary and it comes close to assembling almost all MASM code including the vast majority of MASM macros. It will build MASM code for 16, 32 and 64 bit platforms. It is burdened by having no supporting documentation and relies wholly on the availability of documentation from Microsoft for MASM. It is available under the Sybase Open Watcom EULA.
* http://www.japheth.de/JWasm.html

[[Category:Assemblers]]
````

<a id="w002101"></a>

## W002101 — TASM

Bron-ID: W002101
Namespace: Main (0)
Revisie: 28483 | Datum: 2023-11-28T07:05:58Z
Bron: https://wiki.osdev.org/TASM
Revisie-URL: https://wiki.osdev.org/index.php?title=TASM&oldid=28483

````text
TASM, [[Borland | Borland's]] Turbo Assembler, is an x86 assembler, able to create both 16 and 32 bit programs.

==Advantages==
There are several, if not many, advantages to using the Turbo Assembler. One of which is the "Ideal" mode syntax which many ASM developers prefer over the "MASM" syntax.  TASM can also compile [[MASM]] source by using its "MASM mode". Its highly compatible with other Borland Turbo Products, namely [[Turbo C]], [[Turbo Pascal]], and [[Turbo Basic]] and has powerful macro support.

==Disadvantages==
Some of the disadvantages to using TASM is that it is not open source. Another disadvantage, especially to today's typical developer, is that TASM only runs on Windows and DOS.
Windows user can use GUI Turbo Assembler which is an IDE that comes packed with Turbo Assembler, Linker and Debugger for writing assembly codes.

==Demise==
It has been left (by the developer community) partially because assembly language has been all but left in favor of [[High Level Languages]], and, as mentioned above, it is not open source or cross-compatible. These disadvantages were taken advantage of somewhere in the open source movement by the authors of [[NASM]]. NASM, with its cross-platform compatibility and open source GPL (now BSD) license, in combination with the fact that it supports Intel Syntax, effectively brought about the demise of the once omnipotent Turbo Assembler.

==See Also==
=== Articles ===
* [[LZASM]] - A freeware TASM compatible assembler that supports more modern x86 instructions

=== Threads ===

===External Links===
*[https://web.archive.org/web/20090126151846/http://info.borland.com:80/borlandcpp/cppcomp/tasmfact.html Archive of TASM Homepage]
*[https://web.archive.org/web/20050723074324/http://webster.cs.ucr.edu:80/Articles/X86FAQ/tasm.html Archive of TASM FAQ]
*[https://sourceforge.net/projects/guitasm8086/ GUI Turbo Assembler]

[[Category:Assemblers]]
````

<a id="w002102"></a>

## W002102 — YASM

Bron-ID: W002102
Namespace: Main (0)
Revisie: 29992 | Datum: 2026-02-13T17:50:50Z
Bron: https://wiki.osdev.org/YASM
Revisie-URL: https://wiki.osdev.org/index.php?title=YASM&oldid=29992

````text
{{Stub}}

The Yasm Modular Assembler is a relatively new assembler built with the intention to support many instruction sets and syntax formats. At the moment it can assemble intel syntax and AT&T syntax and has support for 64 bit instructions, and is almost fully compatible with both [[NASM]] and [[GAS]].

== See Also ==
=== Articles ===
* [[Assembly]]
* [[Tool Comparison]]

=== External Links ===
* [https://web.archive.org/web/20240804081159/http://yasm.tortall.net/ YASM Homepage]
* [https://github.com/yasm/yasm YASM repository at GitHub]

[[Category:Assemblers]]
````

<a id="w002103"></a>

## W002103 — Talk:Higher Half x86 Bare Bones (Backup)

Bron-ID: W002103
Namespace: Talk (1)
Revisie: 23156 | Datum: 2018-12-27T22:32:10Z
Bron: https://wiki.osdev.org/Talk:Higher_Half_x86_Bare_Bones_(Backup)
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AHigher_Half_x86_Bare_Bones_%28Backup%29&oldid=23156

````text
== Where? ==

Where did this C kernel tutorial which I am seeing missing in several pages go to?
--[[User:Frank|Frank]] 22:20, 1 July 2007 (CDT)

:I think it refers to the "C kernel [[bare bones]] tutorial", which has been renamed a few times - [[User:Combuster|Combuster]] 09:25, 2 July 2007 (CDT)
:Yes, that is what the links should be. That was messy editing on my part. Sorry. -[[User:Jhawthorn|Jhawthorn]] 14:40, 2 July 2007 (CDT)

Intrepid Ibex repositories have the 2.03 which is ok for this. Can someone check the hardy repos please? <- Someone posted in the middle of the article [[User:Oros|Oros]]


According to http://forum.osdev.org/viewtopic.php?p=243327#p243327, you do not need to substract KERNEL_VIRTUAL_BASE from _loader  . I deleted the _loader symbol, used only loader, and it works fine for me

== Disputing ==

There are two obvious issues with this article. First, I needed to use ENTRY(_loader) and not ENTRY(loader), that is I needed to use the virtual entry address. Attempting to do otherwise resulted in the GRUB error described in the page. Second, it is '''not guaranteed''' that GRUB will put the multiboot structure in the lower 4 MiB.

After these, I can see no mention about mapping only the kernel (and not the whole lower 4 MiB), setting permissions to .text, .data, .rodata, .bss, etc, or setting the WP bit.

[[User:Glauxosdever|Glauxosdever]] 13:55, 3 June 2016 (CDT)

== NASM style is conflicting with the original Bare Bones tutorial ==

This 'tutorial' references the [[Bare Bones]] section, but is not compatible because it is using NASM style assembly, while the original Bare Bones is using GAS/AT&T style assembly. I propose that the assembly code on this page is rewritten to be compatible with the original Bare Bones tutorial. Of course, there are a few issues with the code that need to be fixed first (see above).

[[User:Basdp|Basdp]] 04:12, 25 August 2016 (CDT)


There is a new tutorial being prepared in order to replace this.

[[User:Glauxosdever|Glauxosdever]] 05:08, 25 August 2016 (CDT)

== Video memory offset ==

Using kernel.c from the [[Bare Bones]] tutorial will fault when it reaches the terminal_writestring() call because the tutorial's terminal driver does not account for the higher-half offset.  I added a small point to the end of the tutorial to accommodate this. [[User:Sigtau|Sigtau]] 00:50, 2 October 2016 (CDT)

== NASM Higher Half Bare Bones ==

The NASM Code here could (with updates, if needed/wanted) be linked in the normal HH Bare bones page like the NASM version of the Bare Bones assembly code.
--[[User:Electrodeyt|Electrodeyt]] 15:17, 26 December 2018 (CST)
````

<a id="w002104"></a>

## W002104 — User:Frank

Bron-ID: W002104
Namespace: User (2)
Revisie: 19535 | Datum: 2016-07-22T21:00:32Z
Bron: https://wiki.osdev.org/User:Frank
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AFrank&oldid=19535

````text
== Stuff I have learned ==
=== How to create a topic link: ===
<pre>
[[Topic:TOPIC_ID|Title]]
[[Topic:TOPIC_ID]]
</pre>

== Pages I have fixed ==
* [[How kernel, compiler, and C library work together]]
* [[Semaphores]]
* [[How Do I Use A Debugger With My OS]]
* [[Higher Half x86 Bare Bones]]
* [[Bare Bones]]
* [[Babystep7]]
* [[Interrupts For Dummies]]
* [[James T. Klik]]
* [[Modular Kernel]]
* [[Boot Sequence]]
* [[Rolling Your Own Bootloader]]
````

<a id="w002105"></a>

## W002105 — Talk:Kernel Debugging

Bron-ID: W002105
Namespace: Talk (1)
Revisie: 14821 | Datum: 2013-06-04T05:11:40Z
Bron: https://wiki.osdev.org/Talk:Kernel_Debugging
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AKernel_Debugging&oldid=14821

````text
Broken links under "Using a serial null-modem between GDB and your OS" both seem to be broken (KernelHacking.org is now a landing page, and the gdb spits a 404) maybe it's time for some spring cleaning on this page? -- [[User:Primis|Primis]] 18:32, 15 September 2012 (CDT)

I fixed all of the links and formating. I just don't know which category to place this in. Other than that I think that it can be taken out of pages for review. --[[User:Frank|Frank]] 12:37, 2 July 2007 (CDT)
:It's a [[:Category:FAQ|FAQ]] page, so it goes in that category. I fixed up some last things and removed the review tag. Thanks for your help - [[User:Combuster|Combuster]] 02:32, 3 July 2007 (CDT)

Is such a long name for the article appropriate? "How Do I Use A Debugger With My OS" is more of a question than a title. Other articles are not labeled "What is an ELF file", just "ELF", so I think it would be appropriate to rename this article to "Kernel Debugging". The same could apply to other articles in the [http://wiki.osdev.org/Category:FAQ FAQ] category. --[[User:Messiahandrw|MessiahAndrw]] 03:06, 22 December 2009 (UTC)
:+1 for this in principal. Only one thing - articles in the FAQ category should perhaps appear as questions. I would prefer the title "Kernel Debugging", but it may annoy some people that FAQ's are not actually posed as questions. You may find there is some debate - perhaps you would like to pose the question on the forums? --[[User:Aj|Aj]] 11:21, 22 December 2009 (UTC)
The FAQ can still remain in the form of questions. For example an article link can still say "How Do I Use A Debugger With My OS?" but direct you to the the "Kernel Debugging" article. --[[User:Messiahandrw|MessiahAndrw]] 02:16, 27 July 2010 (UTC)

I think we should avoid using personalized speech in the Wiki (except on user pages). For example under the heading "Use a virtual machine" someone stated "my favorite is Bochs". I changed it to a more appropriate wording; "Bochs is favored amongst the OS community." --[[User:Messiahandrw|MessiahAndrw]] 02:16, 27 July 2010 (UTC)
````

<a id="w002106"></a>

## W002106 — Talk:Atomic Action

Bron-ID: W002106
Namespace: Talk (1)
Revisie: 4155 | Datum: 2007-07-02T17:39:52Z
Bron: https://wiki.osdev.org/Talk:Atomic_Action
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AAtomic_Action&oldid=4155

````text
Can't we just delete this page and change all of the links to [[Atomic operation]]? --[[User:Frank|Frank]] 12:39, 2 July 2007 (CDT)
````

<a id="w002107"></a>

## W002107 — Talk:Rolling Your Own Bootloader

Bron-ID: W002107
Namespace: Talk (1)
Revisie: 28678 | Datum: 2024-03-01T07:51:54Z
Bron: https://wiki.osdev.org/Talk:Rolling_Your_Own_Bootloader
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ARolling_Your_Own_Bootloader&oldid=28678

````text
Are we keeping this page and fixing it up or are we going to move its contents to somewhere else? It would be nice to know before I go and fix it up. Thanks. --[[User:Frank|Frank]] 09:46, 3 July 2007 (CDT)

:I already rewrote parts of the article a while before this version was moved in with the fast merge. The main difference is that the other article is more theoretical but lacks some information that was present. I think we should decide here which way to go:
:# fix this article and remove [[Bootloader]]
:# copy the remaining information to [[Bootloader]] and remove this article
:# move all remaining theory to bootloader and keep all practical info here in a separate article.
:My choice would be either option 2 or 3, largely depending on wether keeping two articles is useful. One could add some extra information to the practical aspects, at which point #3 becomes the better one. At the very least we should compare the articles and see what information is missing from either one. - [[User:Combuster|Combuster]] 06:25, 5 July 2007 (CDT)

:: I think that I like 3 better. Moving of this information would just clutter up the theory article. This one is more of a how-to anyways. --[[User:Frank|Frank]] 15:36, 5 July 2007 (CDT)

== Broken link in 'How do I actually load bytes' section ==
The broken link is the 'see OSRC for more info'. I've been perusing the [http://www.nondot.org/sabre/ root site] but I never saw the original link, so I'm not sure exactly what I'm looking for, I just know there is no 'os' section anymore. I'll keep looking, but I wanted to make a note to see if anyone had more luck than me.

-- [[User:Jflopezfernandez|Jflopezfernandez]] 12:35, 12 August 2019 (CDT)

== Its very biased ==

Rolling your own bootloader is very biased and has a very negative tone. That's what I got from it. The neg tone set from the start from the author made it hard to concentrate on the actual page written. There are a lot of people who prefer to learn and write their own stuff, it makes them feel like they accomplished something, even its something little. There are a lot of people who don't care to use tools. It was said that if a bootloader source code was presented then someone would be using that premade bootloader instead of trying to make one and that is the not case at all.

There are quite a few people who actually study the source code to understand it even if they never use the product. I mean how many times have you found your self re-reading something until you finally understood it? Tracing labels and their connections. That's not possible if nothing is presented. How else are you to ever become one of the old greats, if you never study, different sources of code? Maybe someone would like to compare styles and learn new techniques from what they read. How else are you suppose to write things from scratch and learn the more difficult things if only tools are being shoved down your throat. All of these things contribute to making better coders.

At what point does the art become lost?? Idk. I like os dev but starting to hate all the biased pages with limited and narrow views. What is the point of even writing a page if you are not going to properly cover the topic? If you have the knowledge, on a knowledge giving/providing site, but are refusing to share the knowledge. That makes no sense. Is this not a site to come and learn how to code the things you want to learn to code? I almost felt the reason the author didn't provide the bootloader, was because he himself has become too reliant on tools, that he was unable to provide the bootloader from lack of knowledge. Not trying to start a fight, not trying to be rude, or anything like that. Its just what I personally took from the page. I am not completely against tools either, because I use some myself, that I find help the workflow. I am merely writing my view point on this page because it has irked me every time I have read it. This was/is an introductory page, is this really the tone, we want to set for new visitors when they read from the beginning?
````

<a id="w002108"></a>

## W002108 — Talk:Boot Sequence

Bron-ID: W002108
Namespace: Talk (1)
Revisie: 12257 | Datum: 2011-12-12T10:34:25Z
Bron: https://wiki.osdev.org/Talk:Boot_Sequence
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABoot_Sequence&oldid=12257

````text
Where did Things You Cannot Do With C go? --[[User:Frank|Frank]] 09:51, 3 July 2007 (CDT)
: Never mind I found it --[[User:Frank|Frank]] 10:00, 3 July 2007 (CDT)

== Merging with System Initialization ==

Rewrite in progress. Please make a note here if this article is modified in the next few days.
[[User:Bewing|Bewing]] 14:18, 8 July 2008 (UTC)<br>
Almost all of the content of this article has now been moved to other articles. As soon as that is finished, I intend to delete this article. (It sux, anyway.) [[User:Bewing|Bewing]] 14:12, 22 July 2008 (UTC)
````

<a id="w002109"></a>

## W002109 — Talk:Link Archiver

Bron-ID: W002109
Namespace: Talk (1)
Revisie: 4207 | Datum: 2007-07-08T20:19:59Z
Bron: https://wiki.osdev.org/Talk:Link_Archiver
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ALink_Archiver&oldid=4207

````text
Is there anything else anyone can think of to add to this article? If not, I think it's ready to moved out of review. --Alboin

As the tag says, it should be categorized. However, there is no good matching category for ''ar'' in existance. [[:Category:Linkers]] is about the closest. The main alternative is to create another category like [[:Category:Other Utilities]]. The consequence of the latter is that it implies making a change on the main page, which is something I'd like to have approved by someone else before I do so. - [[User:Combuster|Combuster]] 15:19, 8 July 2007 (CDT)
````

<a id="w002111"></a>

## W002111 — Talk:BootProg

Bron-ID: W002111
Namespace: Talk (1)
Revisie: 4222 | Datum: 2007-07-09T09:02:55Z
Bron: https://wiki.osdev.org/Talk:BootProg
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABootProg&oldid=4222

````text
Anyone have any experience with BootProg? (i.e. to expand upon the article.) If not, I'd say this page is ready for removal from PFR. --Alboin

:I changed the status from PFR to stub, since the former doesn't depend on any future additions - [[User:Combuster|Combuster]] 04:02, 9 July 2007 (CDT)
````

<a id="w002113"></a>

## W002113 — Talk:Descriptors

Bron-ID: W002113
Namespace: Talk (1)
Revisie: 4235 | Datum: 2007-07-09T18:24:27Z
Bron: https://wiki.osdev.org/Talk:Descriptors
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ADescriptors&oldid=4235

````text
== Pages for Review ==
Checking - can this article be removed from PFR? - [[User:Combuster|Combuster]] 05:39, 9 July 2007 (CDT)
*Looks good here. --Alboin
::Ok - removed tag - [[User:Combuster|Combuster]] 13:24, 9 July 2007 (CDT)

== Todo's ==
With the review I filtered some todo's. This article doesn't explain:
* (non)conforming code segments
* long mode descriptors
* IDT desciptors and system descriptors
Could somebody with spare time do those? - [[User:Combuster|Combuster]] 05:39, 9 July 2007 (CDT)
````

<a id="w002114"></a>

## W002114 — Talk:Non Maskable Interrupt

Bron-ID: W002114
Namespace: Talk (1)
Revisie: 4265 | Datum: 2007-07-10T15:33:39Z
Bron: https://wiki.osdev.org/Talk:Non_Maskable_Interrupt
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ANon_Maskable_Interrupt&oldid=4265

````text
==Quote Breakup?==
What should we do with Brendan's quote? Also, how does one update a forum link like that? I couldn't find how, although I think I've seen it somewhere before. --Alboin
:[[Post:83433]] for the links. As for the quote, I don't know enough about the NMI to rewrite things - [[User:Combuster|Combuster]] 10:33, 10 July 2007 (CDT)

==Comments==
:Is it really wise to turn off NMI? okay, if you get an NMI while you're switching from RealMode to ProtectedMode, you could get a [http://en.wikipedia.org/wiki/Triple_fault Triple Fault], which would reset the system, but isn't a system reset wished when content from memory is unreliable by that time ? -- Pype.

:Is there much you can do if an NMI occurs? I guess if you got the error while reading from something that was copied from a disk at some point in the past and not modified since then you could read it from the disk again and continue with the (hopefully) good copy, but if it's something that has changed or been created dynamically then you don't really have an easy way to recover other than essentially starting from scratch. -- TheKemp

:Of course, you're assuming that the kernel code hasn't been corrupted, if the kernel is damaged then you'll most likely triple fault anyway. The best course of action is probably to request the user perform a RAM diagnostic with ~MemTest or something (and hope you don't crash before you can get that far). Then again... Windows keeps a "Hardware Damaged" flag for every physical page of RAM, unfortunately this would mean aborting the program that was running when the NMI occured then checking each page the program was using at the time for what are basically "bad sectors" and flaging them so they aren't used again -- AR

:So if an NMI occurs you should assume it'll continue happening at that location in future rather than it being a freak occurrence? -- TheKemp

:I'm not an engineer so I wouldn't know for certain, but I would be inclined to think that if an NMI occurs on a page when you retest the program that caused the original NMI then it would seem rather likely that the chip is faulty, I don't know if Windows persists the flags across reboots but I doubt it since AFAIK it's impossible to tell if the RAM has been replaced while the computer was off. If the NMI doesn't occur again while checking the pages then the fault isn't severe and could be written off, just if it is a persistent problem then the page should probably be disabled (in the interest of preventing random program crashes) -- AR

:Added Brendan's information from the forums, could probably do with being organised better but at least it's here for now. -- TheKemp
````

<a id="w002115"></a>

## W002115 — Talk:LE

Bron-ID: W002115
Namespace: Talk (1)
Revisie: 4257 | Datum: 2007-07-10T02:56:18Z
Bron: https://wiki.osdev.org/Talk:LE
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ALE&oldid=4257

````text
Hey! I'm pretty excited about this. I think I've found that the LE\LX specification is PD! (IBM apparently released it.) It's [http://hobbes.nmsu.edu/pub/os2/dev/info/lxexe.doc Here]. What do you want to do with it? A link? Or should I format it all pretty like and post it on the wiki? --Alboin
: Wow that's a long file. Never mind, I think I'll just add a link, but if someone was '''really''' bored I guess they could still do it.;) --Alboin
````

<a id="w002116"></a>

## W002116 — Talk:Detecting Colour and Monochrome Monitors

Bron-ID: W002116
Namespace: Talk (1)
Revisie: 15467 | Datum: 2013-12-20T00:34:10Z
Bron: https://wiki.osdev.org/Talk:Detecting_Colour_and_Monochrome_Monitors
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ADetecting_Colour_and_Monochrome_Monitors&oldid=15467

````text
How can this be improved? (Why is it in PFR?) Or can it just be removed from PFR? --Alboin
:I think the 'only roughly converted' was enough :) - [[User:Combuster|Combuster]] 11:35, 10 July 2007 (CDT)

== integration ==

could this short article be integrated into a larger article on VGA or Text mode, or both? [[User:Jackscott|JackScott]] 17:39, 18 February 2009 (UTC)

Well, we can rule out the VGA article - the VGA and friends cannot detect what kind of monitor is connected. Since this comes straight from the BDA, I assume it's a bios option if not defaulted to a colour monitor. The text mode article isn't good either since the monitor properties apply to both text and graphics mode. If it should be merged, it should be done with some article on EDID or similar. Right now, it IMO fits best here, alone. - [[User:Combuster|Combuster]] 18:36, 18 February 2009 (UTC)

== Video mode setup on linux ==
Here's a page on how linux implements this detection, maybe someone with time can integrate and find more source material to strengthen?

http://lxr.linux.no/linux+v3.3.5/drivers/video/console/vgacon.c#L433
[[User:Chazzeromus|Chazzeromus]] 19:20, 7 May 2012 (CDT)

:I feel this to be an artifact from long, *loooong* ago. BIOS is on its way to the "legacy" booth, and I haven't seen a (working) monochrome monitor in decades. I don't think this article needs "strengthening". Let it sit here gathering dust, and maybe some time someone takes a heart and relieves it of its digital existence... ;-) -- [[User:Solar|Solar]] 09:38, 8 May 2012 (CDT)

== ISO C doubts ==

I tried to compile this along with my OS. Apparently there was a complain that USHORT is not a variable type. A little research tells me to use unsigned int as USHORT can be a typedef of USHORT, but apparently doing so seems to break the program. Can someone do a check and reprogram the function to use ISO C types? Or is something wrong with my OS? --hometue

:Uh, <tt>USHORT</tt> is really just a bad coding practice that expands to <tt>unsigned short</tt>. Note however that this code likely meant an unsigned 16-bit value, so <tt>uint16_t</tt> is probably a better choice. Then comes endian trouble and such, but since this is x86-specific anyway, I think it'll do. I'll just fix the code, seems trivial. --[[User:Sortie|Sortie]] 18:34, 19 December 2013 (CST)
````

<a id="w002117"></a>

## W002117 — Talk:Testing

Bron-ID: W002117
Namespace: Talk (1)
Revisie: 4459 | Datum: 2007-08-31T05:50:07Z
Bron: https://wiki.osdev.org/Talk:Testing
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ATesting&oldid=4459

````text
I can't really think of a wiki-like name for this article. Any ideas? --Alboin
:If the page was expanded to be more general (and I get the feeling that was the original intention way back when) it could be renamed ''Testing'', and merged with the information [Getting_Started#Testing_your_operating_system]. That part of Getting Started could then become a link to this page? Or something like that. [[User:Yayyak|Yayyak]] 22:35, 30 August 2007 (CDT)
::As for the link, something like the Paging and Segmentation sections in [[Memory_management]] would be nice. Name wise, Testing sounds good to me. --Alboin
:::Yes. Let's do it then. [[User:Yayyak|Yayyak]] 00:49, 31 August 2007 (CDT)
````

<a id="w002118"></a>

## W002118 — Talk:USB Tutorial

Bron-ID: W002118
Namespace: Talk (1)
Revisie: 4262 | Datum: 2007-07-10T03:18:41Z
Bron: https://wiki.osdev.org/Talk:USB_Tutorial
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AUSB_Tutorial&oldid=4262

````text
I believe this can be deleted as it has already been moved to [[USB]]. --Alboin
````

<a id="w002119"></a>

## W002119 — Talk:And what about USB

Bron-ID: W002119
Namespace: Talk (1)
Revisie: 4273 | Datum: 2007-07-10T18:25:54Z
Bron: https://wiki.osdev.org/Talk:And_what_about_USB
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AAnd_what_about_USB&oldid=4273

````text
I think merging this article and [[USB]] is a good idea. Something like:

*Adding USB (From this article.)
*Low Level (The stuff currently in [[USB]])
*High Level (USB keyboards/mice in this article.)
*Booting (From this article.)
*See Also

What do you think?
-Alboin

I agree with the merge. I preserved the original order however as it seems more fitting. (First the things you can do without inside knowledge on the USB system, then continuing for those who are still interested). As far as I am concerned this article can be deleted now (please copy the talkpage before doing so) - [[User:Combuster|Combuster]] 13:25, 10 July 2007 (CDT)
````

<a id="w002120"></a>

## W002120 — Talk:Universal Serial Bus

Bron-ID: W002120
Namespace: Talk (1)
Revisie: 9590 | Datum: 2010-01-08T07:11:21Z
Bron: https://wiki.osdev.org/Talk:Universal_Serial_Bus
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AUniversal_Serial_Bus&oldid=9590

````text
==Merger==
I think merging this article and USB is a good idea. Something like:

    * Adding USB (From this article.)
    * Low Level (The stuff currently in USB)
    * High Level (USB keyboards/mice in this article.)
    * Booting (From this article.)
    * See Also 

What do you think? -Alboin

I agree with the merge. I preserved the original order however as it seems more fitting. (First the things you can do without inside knowledge on the USB system, then continuing for those who are still interested). As far as I am concerned this article can be deleted now (please copy the talkpage before doing so) - Combuster 13:25, 10 July 2007 (CDT)
:Done --Alboin

== [[Special:Longpages|Longest page]] on the wiki! ==

Congratz on making this article the largest one on the entire wiki!

Now if only the rest were like this... :) --[[User:Troy martin|Troy Martin]] 18:16, 26 June 2009 (UTC)
````

<a id="w002121"></a>

## W002121 — Talk:VGA Text Mode

Bron-ID: W002121
Namespace: Talk (1)
Revisie: 30589 | Datum: 2026-05-21T06:16:16Z
Bron: https://wiki.osdev.org/Talk:VGA_Text_Mode
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AVGA_Text_Mode&oldid=30589

````text
Anyone object to moving this out of PFR? I'm not quite sure about the Troubleshooting section... --Alboin

== The fine line of "example" code ==

I don't think that we should fill our Wiki with "building block" code, especially not something so trivial as putc() / puts(). ''Everybody'' meeting [[Required Knowledge]] should be able to write this in less than five minutes. Moreover, '''if''' we provide example code, it should be a ''shining'' example - and using <tt>UCHAR</tt> and a <tt>Puts()</tt> (sp!) that doesn't do a line feed isn't. -- [[User:Solar|Solar]] 07:44, 22 February 2012 (CST)


Sorry, but I was cut off before I had completed writing the code - [[User:Bellezzasolo|Bellezzasolo]] (Once again, please sign).

== Merge ==

I propose merging this article with [[Text UI]], as the information contained is virtually identical. -- [[User:Solar|Solar]] 09:52, 23 February 2012 (CST)

== Renaming and merging ==

Adding on what was suggested above, this article should also be renamed to "VGA Text Mode" and merged with parts of the [[Text UI]] article. --[[User:Tocariimaa|Tocariimaa]] ([[User talk:Tocariimaa|talk]]) 17:01, 27 February 2026 (UTC)
````

<a id="w002122"></a>

## W002122 — X86 Paging

Bron-ID: W002122
Namespace: Main (0)
Revisie: 30739 | Datum: 2026-06-12T16:52:38Z
Bron: https://wiki.osdev.org/X86_Paging
Revisie-URL: https://wiki.osdev.org/index.php?title=X86_Paging&oldid=30739

````text
{{Disputed}}
[[image:Paging_Structure.gif|right|thumb|600x350px|x86 Paging Structure]]

Paging is a system which allows each process to see a full virtual address space, without actually requiring the full amount of physical memory to be available or present. 32-bit x86 processors support 32-bit virtual addresses and 4-GiB virtual address spaces, and current 64-bit processors support 48-bit virtual addressing and 256-TiB virtual address spaces. Intel has released [https://en.wikipedia.org/wiki/Intel_5-level_paging documentation] for a extension to 57-bit virtual addressing and 128-PiB virtual address spaces. Currently, implementations of x86-64 have a limit of between 4 GiB and 256 TiB of physical address space (and an architectural limit of 4 PiB of physical address space).

In addition to this, paging introduces the benefit of page-level protection. In this system, user processes can only see and modify data which is paged in on their own address space, providing hardware-based isolation. System pages are also protected from user processes. On the x86-64 architecture, page-level protection now completely supersedes [[Segmentation]] as the memory protection mechanism. On the IA-32 architecture, both paging and segmentation exist, but segmentation is now considered 'legacy'.

Once an Operating System has paging, it can also make use of other benefits and workarounds, such as linear framebuffer simulation for memory-mapped IO and paging out to disk, where disk storage space is used to free up physical RAM.

== 32-bit Paging (Protected Mode) ==

=== MMU ===

Paging is achieved through the use of the [[Memory Management Unit]] (MMU). On the x86, the MMU maps memory through a series of [[Page Tables|tables]], two to be exact. They are the paging directory (PD), and the paging table (PT).

Both [[Page Tables| tables]] contain 1024 4-byte entries, making them 4 KiB each. In the page directory, each entry points to a page table. In the page table, each entry points to a 4 KiB physical page frame. Additionally, each entry has bits controlling access protection and caching features of the structure to which it points. The entire system consisting of a page directory and page tables represents a linear 4-GiB virtual memory map.

Translation of a virtual address into a physical address first involves dividing the virtual address into three parts: the most significant 10 bits (bits 22-31) specify the index of the page directory entry, the next 10 bits (bits 12-21) specify the index of the page table entry, and the least significant 12 bits (bits 0-11) specify the page offset. The then MMU walks through the paging structures, starting with the page directory, and uses the page directory entry to locate the page table. The page table entry is used to locate the base address of the physical page frame, and the page offset is added to the physical base address to produce the physical address. If translation fails for some reason (entry is marked as not present, for example), then the processor issues a page fault.

=== Page Directory ===
The topmost paging structure is the page directory. It is essentially an array of page directory entries that take the following form.

[[Image:Page_directory_entry.png|frame|A Page Directory Entry]]

When PS=0, the page table address field represents the '''physical''' address of the page table that manages the four megabytes at that point. Please note that it is very important that this address be 4-KiB aligned. This is needed, due to the fact that the last 12 bits of the 32-bit value are overwritten by access bits and such. Similarly, when PS=1, the address must be 4-MiB aligned.

* PAT, or ''''P'''age '''A'''ttribute '''T'''able'. If [https://en.wikipedia.org/wiki/Page_attribute_table PAT] is supported, then PAT along with PCD and PWT shall indicate the memory caching type.  Otherwise, it is reserved and must be set to 0.
* G, or ''''G'''lobal' tells the processor not to invalidate the TLB entry corresponding to the page upon a MOV to CR3 instruction. Bit 7 (PGE) in CR4 must be set to enable global pages.
* PS, or ''''P'''age '''S'''ize' stores the page size for that specific entry. If the bit is set, then the PDE maps to a page that is 4 MiB in size. Otherwise, it maps to a 4 KiB page table. Please note that 4-MiB pages require PSE to be enabled.
* D, or ''''D'''irty' is used to determine whether a page has been written to.
* A, or ''''A'''ccessed' is used to discover whether a PDE or PTE was read during virtual address translation. If it has, then the bit is set, otherwise, it is not. Note that, this bit will not be cleared by the CPU, so that burden falls on the OS (if it needs this bit at all).
* PCD, is the 'Cache Disable' bit. If the bit is set, the page will not be cached. Otherwise, it will be.
* PWT, controls Write-Through' abilities of the page. If the bit is set, write-through caching is enabled. If not, then write-back is enabled instead.
* U/S, the ''''U'''ser/Supervisor' bit, controls access to the page based on privilege level. If the bit is set, then the page may be accessed by all; if the bit is not set, however, only the supervisor can access it. For a page directory entry, the user bit controls access to all the pages referenced by the page directory entry. Therefore if you wish to make a page a user page, you must set the user bit in the relevant page directory entry as well as the page table entry.
* R/W, the ''''R'''ead/'''W'''rite' permissions flag. If the bit is set, the page is read/write. Otherwise when it is not set, the page is read-only. The WP bit in CR0 determines if this is only applied to userland, always giving the kernel write access (the default) or both userland and the kernel (see Intel Manuals 3A 2-20). The R/W bit of the parent tables is also checked: if any are 0, the page is treated as read-only.
* P, or ''''P'''resent'. If the bit is set, the page is actually in physical memory at the moment. For example, when a page is swapped out, it is not in physical memory and therefore not 'Present'. If a page is called, but not present, a page fault will occur, and the OS should handle it. (See below.)

The remaining bits 9 through 11 (if PS=0, also bits 6 & 8) are not used by the processor, and are free for the OS to store some of its own accounting information. In addition, when P is not set, the processor ignores the rest of the entry and you can use all remaining 31 bits for extra information, like recording where the page has ended up in swap space. When changing the accessed or dirty bits from 1 to 0 while an entry is marked as present, it's recommended to invalidate the associated page. Otherwise, the processor may not set those bits upon subsequent read/writes due to TLB caching. 

[[Image:Page table entry.png|frame|A Page Table Entry]]

Setting the PS bit makes the page directory entry point directly to a 4-MiB page. There is no paging table involved in the address translation.
Note: With 4-MiB pages, whether or not bits 20 through 13 are reserved depends on PSE being enabled and how many PSE bits are supported by the processor (PSE, PSE-36, PSE-40). [[CPUID]] should be used to determine this. Thus, the physical address must also be 4-MiB-aligned. Physical addresses above 4 GiB can only be mapped using 4 MiB PDEs.

=== Page Table ===
In each page table, as it is, there are also 1024 entries. These are called page table entries, and are '''very''' similar to page directory entries.

The first item, is once again, a 4-KiB aligned physical address. Unlike previously, however, the address is not that of a page table, but instead a 4 KiB block of physical memory that is then mapped to that location in the page table and directory. Note that the PAT bit is bit 7 instead of bit 12 as in the 4 MiB PDE.

=== Example ===

Say the kernel is loaded to 0x100000. However, it needed to be remapped to 0xC0000000. After loading the kernel, it'll initiate paging, and set up the appropriate tables. (See [[Higher Half Kernel]]) After [[Identity Paging]] the first megabyte, it'll need to create a second table (ie. at entry #768 in the paging directory.) to map 0x100000 to 0xC0000000. The code may be like:

<syntaxhighlight lang="ASM">
 mov eax, 0x0
 mov ebx, 0x100000
 .fill_table:
      mov ecx, ebx
      or ecx, 3
      mov [table_768+eax*4], ecx
      add ebx, 4096
      inc eax
      cmp eax, 1024
      jne .fill_table
</syntaxhighlight>

== 64-Bit Paging ==
[[Image:64-bit page tables1.png|thumb|Page map table entry structure (non-page-sized)]]

Paging in [[x86-64|long mode]] is similar to that of 32-bit paging, except [[PAE|Physical Address Extension]] (PAE) is required. Registers CR2 and CR3 are extended to 64 bits. Instead of just having to utilize 3 levels of page maps: page directory pointer table, page directory, and page table, a fourth page-map table is used: the level-4 page map table (PML4). This allows a processor to map 48-bit virtual addresses to 52-bit physical addresses. If level-5 page maps are supported and enabled, then a fifth page-map table, the level-5 page map table (PML5), allows the processor to map 57-bit virtual addresses to 52-bit physical addresses. Both the PML4 and PML5 contain 512 64-bit entries of which each may point to a lower-level page map table. Do note that with each additional level of paging, virtual addressing becomes slower, especially in the case of TLB cache misses.

Virtual addresses in 64-bit mode must be '''canonical''', that is, the upper bits of the address must either be all 0s or all 1s. For systems supporting 48-bit virtual address spaces, the upper 16 bits must be the same, and for systems supporting 57-bit virtual addresses, the upper 7 bits must match. Although 32-bit code running in [[x86-64|long mode]] (compatibility mode) is still limited to 32-bit virtual addresses, they can still map to a 52-bit physical addresses.

=== Page Map Table Entries ===
[[Image:64-bit page tables2.png|thumb|Page map table entry structure (page-sized)]]

New bits have been added to page map table entries for long-mode paging:

* XD, or ''''E'''xecute '''D'''isable'. If the NXE bit (bit 11) is set in the [[CPU_Registers_x86-64#IA32_EFER|EFER register]], then instructions are not allowed to be executed at addresses within the page whenever XD is set. If EFER.NXE bit is 0, then the XD bit is reserved and should be set to 0.

* PK, or ''''P'''rotection '''K'''ey'. The protection key is a 4-bit corresponding to each virtual address that is used to control user-mode and supervisor-mode memory accesses. If the PKE bit (bit 22) in CR4 is set, then the PKRU register is used for determining access rights for user-mode based on the protection key. If the PKS bit (bit 24) is set in CR4, then the PKRS register is used for determining access rights for supervisor-mode based on the protection key. A protection key allows the system to enable/disable access rights for multiple page entries across different address spaces at once.

* AVL, or ''''Av'''ai'''l'''able'. These bits are ignored by the processor. However, an [[IOMMU]] may use some of these bits (e.g. [[AMD-Vi_IOMMU#I/O_page_tables_2|AMD-Vi IOMMU]])

M signifies the physical address width supported by a processor using PAE. Currently, up to 52 bits are supported, but the actual supported width may be less.

Bits marked as reserved must all be set to 0, otherwise, a page fault will occur with a reserved error code.

Support for 1 GiB pages, (NX) execute disable, (PKS/PKU) protection keys for supervisor-mode and user-mode pages, shadow stack pages, (M) physical address width, virtual address width, (PAT) page attribute table, (PCID) process context identifiers, and (LA57) 5-level paging can be determined with the [[CPUID|CPUID]] instruction (EAX:0x01; EAX:0x07, ECX=0x00; EAX:0x80000001; EAX:0x80000008).

=== Process Context Identifiers ===
If process context ids (PCID) are supported, then bits 0-11 of CR3 specify the process context id. Otherwise, bit 3 is PWT for PML4, and bit 4 is PCD for PML4. PCIDs are used to control TLB caching across multiple address spaces. The INVPCID instruction uses PCIDs to allow more control over page invalidation.

== Enabling ==
=== 32-bit Paging ===
Enabling paging is actually very simple. All that is needed is to load CR3 with the address of the page directory and to set the paging (PG) and protection (PE) bits of CR0.

<syntaxhighlight lang="ASM">
 mov eax, page_directory
 mov cr3, eax
 
 mov eax, cr0
 or eax, 0x80000001
 mov cr0, eax
</syntaxhighlight>

Note: setting the paging flag when the protection flag is clear causes a [[Exceptions#General_Protection_Fault|general protection exception]]. Also, once paging has been enabled, any attempt to enable long mode by setting LME (bit 8) of the [[CPU_Registers_x86-64#IA32_EFER|EFER register]] will trigger a [[Exceptions#General_Protection_Fault|GPF]]. The CR0.PG must first be cleared before EFER.LME can be set.

If you want to set pages as read-only for both userspace and supervisor, replace 0x80000001 above with 0x80010001, which also sets the WP bit.

To enable PSE (4 MiB pages) the following code is required.

<syntaxhighlight lang="ASM">
 mov eax, cr4
 or eax, 0x00000010
 mov cr4, eax
</syntaxhighlight>

=== 64-bit Paging ===
Enabling paging in long mode requires a few more additional steps. Since it is not possible to enter long mode without paging with PAE active, the order in which one enables the bits are important. Firstly, paging must not be active (i.e. CR0.PG must be cleared.) Then, CR4.PAE (bit 5) and EFER.LME (bit 8 of MSR 0xC0000080) are set. If 57-bit virtual addresses are to be enabled, then CR4.LA57 (bit 12) is set. Finally, CR0.PG is set to enable paging.

<syntaxhighlight lang="ASM">
  ; Skip these 3 lines if paging is already disabled
  mov ebx, cr0
  and ebx, ~(1 << 31)
  mov cr0, ebx

  ; Enable PAE
  mov edx, cr4
  or  edx, (1 << 5)
  mov cr4, edx

  ; Set LME (long mode enable)
  mov ecx, 0xC0000080
  rdmsr
  or  eax, (1 << 8)
  wrmsr

  ; Replace 'pml4_table' with the appropriate physical address (and flags, if applicable)
  mov eax, pml4_table
  mov cr3, eax

  ; Enable paging (and protected mode, if it isn't already active)
  or ebx, (1 << 31) | (1 << 0)
  mov cr0, ebx

  ; Now reload the segment registers (CS, DS, SS, etc.) with the appropriate segment selectors...

  mov ax, DATA_SEL
  mov ds, ax
  mov es, ax
  mov fs, ax
  mov gs, ax
  
  ; Reload CS with a 64-bit code selector by performing a long jmp

  jmp CODE_SEL:reloadCS

[BITS 64]
reloadCS:
  hlt   ; Done. Replace these lines with your own code
  jmp reloadCS
</syntaxhighlight>

Once paging has been enabled, you cannot switch from 4-level paging to 5-level paging (and vice-versa) directly. The same is true for switching to legacy 32-bit paging. You must first disable paging by clearing CR0.PG before making changes. Failure to do so will result in a [[Exceptions#General_Protection_Fault|general protection fault]].

== Physical Address Extension ==
All Intel processors since Pentium Pro (with exception of the Pentium M at 400 Mhz) and all AMD since the Athlon series implement the [[PAE|Physical Address Extension]] (PAE). This feature allows you to access up to 4 PiB (2<sup>52</sup>) of RAM. You can check for this feature using [[CPUID|CPUID]]. Once checked, you can activate this feature by setting bit 5 in CR4.

For legacy 32-bit PAE, the CR3 register points to a page directory pointer table (PDPT) of 4 64-bit entries, each one pointing to a page directory made of 4096 bytes (like in normal paging), divided into 512 64-bit entries, each pointing to a 4096-byte page table, divided into 512 64bit page entries. Keep in mind that virtual addresses are still limited to 4 GiB (2<sup>32</sup> bytes).

For 4-level and 5-level PAE, as used in compatibility mode and [[x86-64|long mode]], the CR3 register points to the top-level page map table: the PML4 table and PML5 table, respectively. Each of the page map tables: PML5 table, PML4 table, page directory pointer table, page directory, page table, contain 512 64-bit entries.

If paging is enabled then PAE must also be enabled before entering long mode. Attempting to enter long mode with CR0.PG set and CR4.PAE cleared will trigger a general protection fault.

== Usage ==
Due to the simplicity in the design of paging, it has many uses.

=== Virtual Address Spaces ===
In a paged system, each process may execute in its own area of memory, without any chance of affecting any other process's memory, or the kernel's. Two or more processes may opt to share memory by mapping the same physical page(s) to addresses in their own address spaces. The virtual address of each mapping do not need to be the same. Consequently, a virtual address in one address space won't point to the same data in other address spaces, in general. 
[[Image:Virtual memory.png|frame|none|paging illustrated: two process with different views of the same physical memory]]

=== Virtual Memory ===
Because paging allows for the dynamic handling of unallocated page tables, an OS can swap entire pages, not in current use, to the hard drive where they can wait until they are called. In the mean time, however, the  physical memory that they were using can be used elsewhere. In this way, the OS can manipulate the system so that programs actually seem to have more RAM than there actually is. 

''More...''

== Manipulation ==

The CR3 value, that is, the value containing the address of the page directory, is in physical form. Once, then, the computer is in paging mode, only recognizing those virtual addresses mapped into the paging tables, how can the tables be edited and dynamically changed?

Many prefer to map the last PDE to itself. The page directory will look like a page table to the system. To get the physical address of any virtual address in the range 0x00000000-0xFFFFF000 is then just a matter of:

<syntaxhighlight lang="C">
void *get_physaddr(void *virtualaddr) {
    unsigned long pdindex = (unsigned long)virtualaddr >> 22;
    unsigned long ptindex = (unsigned long)virtualaddr >> 12 & 0x03FF;

    unsigned long *pd = (unsigned long *)0xFFFFF000;
    // Here you need to check whether the PD entry is present.

    unsigned long *pt = ((unsigned long *)0xFFC00000) + (0x400 * pdindex);
    // Here you need to check whether the PT entry is present.

    return (void *)((pt[ptindex] & ~0xFFF) + ((unsigned long)virtualaddr & 0xFFF));
}
</syntaxhighlight>

To map a virtual address to a physical address can be done as follows:

<syntaxhighlight lang="C">
void map_page(void *physaddr, void *virtualaddr, unsigned int flags) {
    // Make sure that both addresses are page-aligned.

    unsigned long pdindex = (unsigned long)virtualaddr >> 22;
    unsigned long ptindex = (unsigned long)virtualaddr >> 12 & 0x03FF;

    unsigned long *pd = (unsigned long *)0xFFFFF000;
    // Here you need to check whether the PD entry is present.
    // When it is not present, you need to create a new empty PT and
    // adjust the PDE accordingly.

    unsigned long *pt = ((unsigned long *)0xFFC00000) + (0x400 * pdindex);
    // Here you need to check whether the PT entry is present.
    // When it is, then there is already a mapping present. What do you do now?

    pt[ptindex] = ((unsigned long)physaddr) | (flags & 0xFFF) | 0x01; // Present

    // Now you need to flush the entry in the TLB
    // or you might not notice the change.
}
</syntaxhighlight>

Unmapping an entry is essentially the same as above, but instead of assigning the <code>pt[ptindex]</code> a value, you set it to 0x00000000 (i.e. not present). When the entire page table is empty, you may want to remove it and mark the page directory entry 'not present'. Of course you don't need the 'flags' or 'physaddr' for unmapping.

== Page Faults ==
A [[Exceptions#Page_Fault|page fault]] exception is caused when a process is seeking to access an area of virtual memory that is not mapped to any physical memory, when a write is attempted on a read-only page, when accessing a PTE or PDE with the reserved bit or when permissions are inadequate. A [[Exceptions#Page_Fault|page fault]] can either be pure, which occurs when the faulting process has permission to access the page, or invalid, which is due to a protection violation. Pure [[Exceptions#Page_Fault|page faults]] aren't errors, but are resolved through the page fault handler by performing the appropriate map operation and/or page swap.

=== Handling ===
The CPU pushes an error code on the stack before firing a [[Exceptions#Page_Fault|page fault exception]].  The error code must be analyzed by the exception handler to determine how to handle the exception.  The following bits are the only ones used, all others are reserved.
 Bit 0 (P) is the Present flag.
 Bit 1 (R/W) is the Read/Write flag.
 Bit 2 (U/S) is the User/Supervisor flag.
 Bit 3 (RSVD) indicates whether a reserved bit was set in some page-structure entry
 Bit 4 (I/D) is the Instruction/Data flag (1=instruction fetch, 0=data access)
 Bit 5 (PK) indicates a protection-key violation
 Bit 6 (SS) indicates a shadow-stack access fault
 Bit 15 (SGX) indicates an [https://en.wikipedia.org/wiki/Software_Guard_Extensions SGX violaton]

The combination of these flags specify the details of the page fault and indicate what action to take:
 US RW  P - Description
 0  0  0 - Supervisory process tried to read a non-present page entry
 0  0  1 - Supervisory process tried to read a page and caused a protection fault
 0  1  0 - Supervisory process tried to write to a non-present page entry
 0  1  1 - Supervisory process tried to write a page and caused a protection fault
 1  0  0 - User process tried to read a non-present page entry
 1  0  1 - User process tried to read a page and caused a protection fault
 1  1  0 - User process tried to write to a non-present page entry
 1  1  1 - User process tried to write a page and caused a protection fault

When the CPU fires a page-not-present exception the CR2 register is populated with the linear address that caused the exception.  The upper 10 bits specify the page directory entry (PDE) and the middle 10 bits specify the page table entry (PTE).  First check the PDE and see if it's present bit is set, if not setup a page table and point  the PDE to the base address of the page table, set the present bit and iretd.  If the PDE is present then the present bit of the PTE will be cleared.  You'll need to map some physical memory to the page table, set the present bit and then iretd to continue processing.

== INVLPG ==

INVLPG is an instruction available since the i486 that invalidates a single page in the TLB. Intel notes that this instruction may be implemented differently on future processors, but that this alternate behavior must be explicitly enabled. INVLPG modifies no flags.

NASM example:

<syntaxhighlight lang="ASM">
 invlpg [0]
</syntaxhighlight>

Inline assembly for GCC (from Linux kernel source):

<syntaxhighlight lang="C">
static inline void __native_flush_tlb_single(unsigned long addr) {
   asm volatile("invlpg (%0)" ::"r" (addr) : "memory");
}
</syntaxhighlight>

This only invalidates the page on the current processor. If you're using SMP, you'll need to send an IPI to the other processors so that they can also invalidate the page (this is called a TLB shootdown; it's very slow), making sure to avoid any nasty race conditions. You may only want to do this when removing a mapping, and just make your page fault handler invalidate a page if it you didn't invalidate a mapping addition on that processor by looking through the page directory, again avoiding race conditions.

When you modify an entry in the page directory, rather than just a page table, you'll need to invalidate each page in the table. Alternatively, you could  reload CR3 which will invalidates the whole directory, but this may be slower. (TODO time this)

== Paging Tricks ==
The processor always fires a page fault exception when the present bit is cleared in the PDE or PTE regardless of the address.  This means the contents of the PTE or PDE can be used to indicate a location of the page saved on mass storage and to quickly load it.  When a page gets swapped to disk, use these entries to identify the location in the paging file where they can be quickly loaded from then set the present bit to 0. Similarly, blocks from disk can be mapped to memory this way. When a process accesses the memory-mapped region, a page fault occurs. The fault handler reads the appropriate tables, loads the disk block(s) into a page, and maps it. The process can then read/write to memory as if it were accessing the device directly. The contents of the page would then be written back to disk to save the changes.

For memory efficiency, two or more processes can share pages as read-only. If one process were to write to its page, then a page fault would occur and the system could duplicate the page and then mark it as read-write. This is known as copy-on-write (COW). Copy-on-write allows the system to delay memory allocation until a process actually requires it, preventing unnecessary copying.

== PAT ==
The Page Attribute Table determines caching attributes on a page granularity. This is similar to [[MTRR]]s, but those apply to physical addresses and are more limited.

The PAT is set via the IA32_PAT_MSR [[MSR]] (0x277). It has 8 entries, taking the low order 3 bits of each byte, in standard little endian order. So the high byte is PAT7, low byte is PAT0.

The following are the different caching types.
{| class="wikitable" border="1"
|-
! Number
! Name
! Description
|-
| 0
| UC — Uncacheable
| All accesses are uncacheable. Write combining is not allowed. Speculative accesses are not allowed.
|-
| 1
| WC — Write-Combining
| All accesses are uncacheable. Write combining is allowed. Speculative reads are allowed.
|-
| 4
| WT — Writethrough
| Reads allocate cache lines on a cache miss. Cache lines are not allocated on a write miss.
Write hits update the cache and main memory.
|-
| 5
| WP — Write-Protect
| Reads allocate cache lines on a cache miss. All writes update main memory.
Cache lines are not allocated on a write miss. Write hits invalidate the cache
line and update main memory.
|-
| 6
| WB — Writeback
| Reads allocate cache lines on a cache miss, and can allocate to either the shared,
exclusive, or modified state. Writes allocate to the modified state on a cache miss.
|-
| 7
| UC- — Uncached
| Same as uncacheable, ''except'' that this can be overriden by Write-Combining MTRRs.
|}

The PAT has a reset value of 0x0007040600070406. This ensures compatibility with non-PAT usage. This corresponds to the following:
{| class="wikitable" border="1"
|-
|UC
|UC-
|WT
|WB
|UC
|UC-
|WT
|WB
|}

The PAT is indexed by the three page table bits:
{| class="wikitable" border="1"
|-
|PAT
|PCD
|PWT
|}
The PAT bit is reserved when there isn't a PAT, and the default value of the MSR ensures backwards comaptibility with the PCD and PWT bit.

You will need to modify the PAT if you want Write-Combining cache, which is very useful for framebuffers.

== See Also ==
*[[X86 Page Tables]]

=== Articles ===
*[[Identity Paging]]
*[[Page Frame Allocation]]
*[[Setting Up Paging]]
*[[Page Tables]]
*[[Memory Management]]
*[[Memory Management Unit]]
*[https://forum.osdev.org/viewtopic.php?p=282061 Page Coloring]

=== External Links ===
*[https://forum.osdev.org/viewtopic.php?f=1&t=18222 INVLPG thread]
*[http://www.dumaisnet.ca/index.php?article=ff3b7adb128cb438ac1e306b3fbe37e7 Process Context ID]
*[https://en.wikipedia.org/wiki/Intel_5-level_paging 5-Level Paging]

[[Category:Memory management]]
[[Category:Paging]]
[[Category:Virtual Memory]]
[[Category:Security]]
[[de:Paging]]
[[Category:X86]]
````

<a id="w002123"></a>

## W002123 — Talk:X86 Paging

Bron-ID: W002123
Namespace: Talk (1)
Revisie: 30737 | Datum: 2026-06-12T16:50:24Z
Bron: https://wiki.osdev.org/Talk:X86_Paging
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AX86_Paging&oldid=30737

````text
[[Memory management]] has the "Paging" section of this article. -[[User:Jhawthorn|Jhawthorn]] 21:57, 9 March 2007 (CST)
:...and? Pull it to one article and link from the other? Merge the articles entirely? - [[User:Combuster|Combuster]] 08:47, 11 March 2007 (CDT)
:What about creating [[Paging]] and moving 'Virtual Memory' and 'Memory Protection' to it, along with creating a more in-depth article about paging in general? I think having [[Segmentation]] without [[Paging]] is a little uneven. Following that, this page could be deleted. --Alboin
::Sounds OK to me. - [[User:Combuster|Combuster]] 15:13, 11 July 2007 (CDT)
:::Done. --Alboin

This is really x86-specific, shouldn't a Paging article be more or less generic, with specific pages made for paging on x86 and other architectures? --[[User:Pcmattman|pcmattman]] 03:11, 27 July 2009 (UTC)

== Really... ==

"Paging is a memory scheme that breaks up memory in groups of pages that are constantly swapped between hard disk and computer" - That is utter bullshit. It should be rephrased. [[User:Jal|Jal]] 19:41, 29 November 2009 (UTC)
:Agreed and I've made a start on fixing it - a proof-read (and possible rewording) would be much appreciated. --[[User:Aj|Aj]] 17:09, 30 November 2009 (UTC)
::Indeed much better. Perhaps I'd make it architecture neutral in the introduction, so at least the ring 3 reference should be avoided (or maybe with an added "on x86" or the like). [[User:Jal|Jal]] 11:16, 2 December 2009 (UTC)
:::OK - because the whole thing is very x86 focussed, I've left in most of the references but made them more obvious. Feel free to tinker with the wording if you like. I've also added a reference to Segmentation in the protection part - just for completeness. --[[User:Aj|Aj]] 19:16, 2 December 2009 (UTC)

== Page Attribute Table (PAT) ==

I'm missing the PAT index bit in the discussion of the page directory (4MB pages only) and the page table. Perhaps it should be added (I've got no time atm). [[User:Jal|Jal]] 11:18, 2 December 2009 (UTC)
````

<a id="w002124"></a>

## W002124 — Descriptor Cache

Bron-ID: W002124
Namespace: Main (0)
Revisie: 26974 | Datum: 2022-02-16T18:25:12Z
Bron: https://wiki.osdev.org/Descriptor_Cache
Revisie-URL: https://wiki.osdev.org/index.php?title=Descriptor_Cache&oldid=26974

````text
To allow for fast accesses to segmented memory, the x86 processor keeps a copy of each segment descriptor in a special descriptor cache. This saves the processor from accessing the [[GDT]] for every memory access made. The workings of this cache allows for some interesting side effects.

== The cache ==
The cache for each segment selector (CS, SS, DS, ES, FS, GS) contains all of the bits and fields you would also find in the [[GDT]]. These include descriptor type, access rights, base and limit. In [[Protected Mode|protected mode]] these fields are filled from the GDT or LDT whenever there is data written to a segment selector (both implicitly and explicitly). In real mode, the processor generates entries internally as there is no GDT involved in this mode. Interestingly, not all fields are updated in real mode. Most notably that includes the segment limit.

== Unreal Mode ==
[[Unreal Mode]] is a 'mode' where the processor runs in real mode while the segment limit does not equal 64KB (in most cases, its 4GB). Since real mode doesn't update the limit field, this state persists across segment register loads. Entering this mode is achieved easily by entering protected mode (where the limit can be changed), load the desired limit into the descriptor cache, then switch back to real mode.

== Other cache settings ==
Other settings for the descriptor cache are also used in the x86 architecture. When the processor boots it enters real mode, but it starts executing at 0xfffffff0 by having a high base value loaded in the descriptor cache for CS. System management mode also loads non-standard values into the cache.

== Modifying the GDT ==
Since each segment register holds a copy of the GDT's contents, updating the GDT will not change the cache even if that selector is currently in use. Therefore one needs to reload the segment registers to make sure the correct values are stored in the cache. This independence can also be used to discard the GDT without breaking the system. GRUB will for instance leave you with a known contents in the descriptor caches, but without GDT. To make sure things remain safe, interrupts will have to be disabled since an interrupt or exception will implicitly reload CS. If the GDT has been overwritten with other data, you are almost guaranteed to get a [[Triple Fault]]

[[Category:X86 CPU]]
````

<a id="w002125"></a>

## W002125 — MMU

Bron-ID: W002125
Namespace: Main (0)
Revisie: 24592 | Datum: 2020-04-11T16:54:22Z
Bron: https://wiki.osdev.org/MMU
Revisie-URL: https://wiki.osdev.org/index.php?title=MMU&oldid=24592
Doorverwijzing: Memory Management Unit

````text
#REDIRECT [[Memory Management Unit]] {{R from merge}}
````

<a id="w002126"></a>

## W002126 — Porting Newlib

Bron-ID: W002126
Namespace: Main (0)
Revisie: 28218 | Datum: 2023-07-10T20:21:11Z
Bron: https://wiki.osdev.org/Porting_Newlib
Revisie-URL: https://wiki.osdev.org/index.php?title=Porting_Newlib&oldid=28218

````text
{{Rating|3}}
Newlib is a C library intended for use on embedded systems available under a free software license. It is known for being simple to port to new operating systems. Allegedly, it's coding practices are sometimes questionable. This tutorial follows [[OS Specific Toolchain]] and completes it using newlib rather than using another [[C Library]] such as [[Creating a C Library|your own]].

Porting newlib is one of the easiest ways to get a simple C library into your operating system without an excessive amount of effort. As an added bonus, once complete you can port the toolchain (GCC/binutils) to your OS - and who wouldn't want to do that?

''This article was written with x86 in mind. It has been extended to armv8 through tips and notes in the troubleshooting section.''

== Introduction ==

I decided that after an incredibly difficult week of trying to get newlib ported to my own OS that I would write a tutorial that outlines the requirements for porting newlib and how to actually do it. I'm assuming you can already load binaries from somewhere and that these binaries are compiled C code. I also assume you have a syscall interface setup already. Why wait? Let's get cracking!

== Preparation ==

Download newlib source (I'm using 2.5.0) from [ftp://sources.redhat.com/pub/newlib/index.html this ftp server].

=== Download source code of Automake and Autoconf ===

Acquire Automake (v1.11) and Autoconf (v2.65) from here:
[https://ftp.gnu.org/gnu/automake/automake-1.11.tar.gz]
[https://ftp.gnu.org/gnu/autoconf/autoconf-2.65.tar.gz]

''Note'': The newlib source is organized using "Cygnus style," which is unsupported in Automake versions 1.12 and beyond.
Therefore, to be able to build newlib, you need a version less than or equal to 1.11.

Untar both of the archives:
<source lang="bash">
tar xvf automake-1.11.tar.gz
tar xvf autoconf-2.65.tar.gz
</source>

Create a destination folder:
<source lang="bash">
mkdir ~/bin
</source>

Create a build folder:
<source lang="bash">
mkdir build
cd build
</source>

Configure automake first:
<source lang="bash">
../automake-1.11/configure --prefix="~/bin"
</source>

Make and install

<source lang="bash">
make && make install
</source>
Now lets configure autoconf
<source lang="bash">
../autoconf-2.65/configure --prefix=~/bin
</source>
Then make and install:
<source lang="bash">
make && make install
</source>

You should now have the proper binaries in ~/bin!

To add these binaries to your path temporarily
<source lang="bash">
export PATH=~/bin:$PATH
</source>

== System Calls ==

First of all you need to support a set of 17 system calls that act as 'glue' between newlib and your OS. These calls are the typical "_exit", "open", "read/write", "execve" (et al).
See the [http://sourceware.org/newlib/libc.html#Syscalls Red Hat newlib C library] documentation for an overview of necessary calls.

Newlib uses a very specific hierarchy of syscalls, many of which can be supplied by more than one file.
This can quickly lead to symbol redefinition or symbol missing errors when linking with the library.
The normal way that newlib expects you to define syscalls, which you may see elsewhere is to define the underscored symbols (e.g. _open instead of open).
In this case, newlib will call the underscored versions using wrappers defined in newlib/libc/syscalls/.
Our different (simplified) approach is to define the syscalls directly. No wrappers.
To do this the newlib_cflags variable must be set to "" in configure.host (default for some platforms, like x86), which will prevent the wrappers from being compiled.

Implementing the syscalls is usually quite trivial, my kernel exposes all the system calls on interrupt 0x80 (128d) so I just had to put a bit of inline assembly into each stub to do what I needed it to do.
It's up to you how to implement them in relation to your kernel.

== Porting Newlib ==

=== config.sub ===

Same as for binutils in [[OS Specific Toolchain]].

=== newlib/configure.host ===

Tell newlib which system-specific directory to use for our particular target.  In the section starting 'Get the source directories to use for the host ... case "${host}" in', add a section:

<source lang="bash">
i[3-7]86-*-myos*)
    sys_dir=myos
    ;;
</source>

configure.host contains two switch clauses, make sure that your variables are not overwritten later!
For example, for aarch64 platforms it sets the syscall_dir variable after us, breaking the library.

=== newlib/libc/sys/configure.in ===

Tell the newlib build system that it also needs to configure our myos-specific host directory.  In the <tt>case ${sys_dir} in</tt> list, simply add

<source lang="autoconf">
  myos) AC_CONFIG_SUBDIRS(myos) ;;
</source>

'''Note:''' After this, you need to run <tt>autoconf (precisely version 2.64)</tt> in the libc/sys directory.

=== newlib/libc/sys/myos ===

This is a directory that we need to create where we put our OS-specific extensions to newlib.  We need to create a minimum of 4 files.  You can easily add more files to this directory to define your own os-specific library functions, if you want them to be included in libc.a (and so linked in to every application by default).

=== newlib/libc/sys/myos/crt0.c ===

This file creates crt0.o, which is included in every application.  It should define the symbol _start, and then call the main() function, possibly after setting up process-space segment selectors and pushing argc and argv onto the stack.  A simple implementation is:

<source lang="C">
#include <fcntl.h>

extern void exit(int code);
extern int main ();

void _start() {
    int ex = main();
    exit(ex);
}
</source>

'''Note:''' add in argc and argv support based on how you handle them in your OS

=== newlib/libc/sys/myos/syscalls.c ===

This file should contain the implementations for each glue function newlib requires.

<source lang="C">
/* note these headers are all provided by newlib - you don't need to provide them */
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/fcntl.h>
#include <sys/times.h>
#include <sys/errno.h>
#include <sys/time.h>
#include <stdio.h>

void _exit();
int close(int file);
char **environ; /* pointer to array of char * strings that define the current environment variables */
int execve(char *name, char **argv, char **env);
int fork();
int fstat(int file, struct stat *st);
int getpid();
int isatty(int file);
int kill(int pid, int sig);
int link(char *old, char *new);
int lseek(int file, int ptr, int dir);
int open(const char *name, int flags, ...);
int read(int file, char *ptr, int len);
caddr_t sbrk(int incr);
int stat(const char *file, struct stat *st);
clock_t times(struct tms *buf);
int unlink(char *name);
int wait(int *status);
int write(int file, char *ptr, int len);
int gettimeofday(struct timeval *p, struct timezone *z);
</source>

'''Note''': You may split this up into multiple files, just don't forget to link against all of them in Makefile.am.

=== newlib/libc/sys/myos/configure.in ===

Configure script for our system directory.

<source lang="bash">
AC_PREREQ(2.59)
AC_INIT([newlib], [NEWLIB_VERSION])
AC_CONFIG_SRCDIR([crt0.c])
AC_CONFIG_AUX_DIR(../../../..)
NEWLIB_CONFIGURE(../../..)
AC_CONFIG_FILES([Makefile])
AC_OUTPUT
</source>

=== newlib/libc/sys/myos/Makefile.am ===

A Makefile template for this directory:

<source lang="make">
AUTOMAKE_OPTIONS = cygnus
INCLUDES = $(NEWLIB_CFLAGS) $(CROSS_CFLAGS) $(TARGET_CFLAGS)
AM_CCASFLAGS = $(INCLUDES)

noinst_LIBRARIES = lib.a

if MAY_SUPPLY_SYSCALLS
extra_objs = syscalls.o # add more object files here if you split up
else                    # syscalls.c into multiple files in the previous step
extra_objs =
endif

lib_a_SOURCES =
lib_a_LIBADD = $(extra_objs)
EXTRA_lib_a_SOURCES = syscalls.c crt0.c # add more source files here if you split up
lib_a_DEPENDENCIES = $(extra_objs)      # syscalls.c into multiple files
lib_a_CCASFLAGS = $(AM_CCASFLAGS)
lib_a_CFLAGS = $(AM_CFLAGS)

if MAY_SUPPLY_SYSCALLS
all: crt0.o
endif

ACLOCAL_AMFLAGS = -I ../../..
CONFIG_STATUS_DEPENDENCIES = $(newlib_basedir)/configure.host
</source>

'''Note''':  After this, you need to run <tt>autoconf</tt> in the newlib/libc/sys/ directory, and <tt>autoreconf</tt> in the newlib/libc/sys/myos directory.

=== Signal handling ===

Newlib has two different mechanisms for dealing with UNIX signals (see the man pages for signal()/raise()).  In the first, it provides its own emulation, where it maintains a table of signal handlers in a per-process manner.  If you use this method, then you will only be able to respond to signals sent from within the current process.  In order to support it, all you need to do is make sure your crt0 calls '_init_signal' before it calls main, which sets up the signal handler table.

Alternatively, you can provide your own implementation.  To do this, you need to define your own version of signal() in syscalls.c.  A typical implementation would register the handler somewhere in kernel space, so that issuing a signal from another process causes the corresponding function to be called in the receiving process (this will also require some nifty stack-playing in the receiving process, as you are basically interrupting the program flow in the middle).  You then need to provide a kill() function in syscalls.c which actually sends signals to another process.  Newlib will still define a raise() function for you, but it is just a stub which calls kill() with the current process id.  To switch newlib to this mode, you need to #define the SIGNAL_PROVIDED macro when compiling.  A simple way to do this is to add the line:

<source lang="bash">
newlib_cflags="${newlib_cflags} -DSIGNAL_PROVIDED"
</source>

to your host's entry in <tt>configure.host</tt>.  It would probably also make sense to provide sigaction(), and provide signal() as a wrapper for it.  Note that [http://pubs.opengroup.org/onlinepubs/9699919799/functions/sigaction.html the Open Group's] definition of sigaction states that 1) sigaction supersedes signal, and 2) an application designed shouldn't use both to manipulate the same signal.

== Compiling ==

You can build newlib in this manner:
Newlib is very pesky about the compiler, and you probably haven't built your own i686-myos-gcc toolchain yet, meaning that configure will not be happy when you set target to i686-myos. So use this hack to get it to work (it worked fine for me).

'''Note:''' there must be a better way then this.

<source lang="bash">
# newlib setup
CURRDIR=$(pwd)

# make symlinks (a bad hack) to make newlib work
cd ~/cross/bin/ # this is where the bootstrapped generic cross compiler toolchain (i686-elf-xxx) is installed in,
                # change this based on your development environment.
ln i686-elf-ar i686-myos-ar
ln i686-elf-as i686-myos-as
ln i686-elf-gcc i686-myos-gcc
ln i686-elf-gcc i686-myos-cc
ln i686-elf-ranlib i686-myos-ranlib

# return
cd $CURRDIR
</source>

Then run the following commands to build newlib

<source lang="bash">
mkdir build-newlib
cd build-newlib
../newlib-x.y.z/configure --prefix=/usr --target=i686-myos
make all
make DESTDIR=${SYSROOT} install
</source>

'''Note:''' SYSROOT is where all your OS-specific toolchains will be installed in. It will look like a miniature version of the Linux filesystem, but have your OS-specific toolchains in; I am using ~/myos as my SYSROOT directory.

'''Note:''' By default, newlib is configured to not support %lld/u format specifiers in printf()/scanf() (i.e. it assumes %lld to mean the same as %ld). In order to override this, should it matter, one must add --enable-newlib-io-long-long to the configure invocation

For some reason, the newer versions of newlib (at least for me) didn't put the libraries in a location where other utilities like binutils could find.
So here's another hack to fix this:

<source lang="bash">
cp -ar $SYSROOT/usr/i686-myos/* $SYSROOT/usr/
</source>

After building all of this, your freshly built libc will be installed in your SYSROOT directory! Now you can progress to building your own [[OS Specific Toolchain]].

'''Important Note:''' I found that for newlib to properly work, you have to link against libc, libg, libm, and libnosys - hence when porting gcc, in

<source lang="C">
#define LIB_SPEC ...
</source>

in gcc/config/myos.h,

make sure you put

<source lang="C">
#define LIB_SPEC "-lc -lg -lm -lnosys"
</source>

at the bare minimum.

I highly recommend rebuilding the library with your [[OS Specific Toolchain]] after you are done porting one. (don't forget to remove the symlinks, too.)

== Conclusion ==

Well, you've done it. You've ported newlib to your OS! With this you can start creating user mode programs with ease! You may now also add in new functions to newlib, such as dlopen(), dlclose(), dlsym(), and dlerror() for dynamic linking support. Your operating system has a bright road ahead! You can now port the toolchain and run binutils and GCC on your own OS. Almost self-hosting, how do you feel?

Good luck!

Last Updated by '''0fb1d8''' for compatibility with newer versions of newlib and the [[OS Specific Toolchain]] tutorial.

'''Note:''' I used a lot of hacks in this article, if you find a better way to do something, please contribute to the page. Thank you.

== Troubleshooting ==

=== Autotools ===
* Whenever you modifiy configure.ac/in files, or not auto-generated Makefile.in files, you must run the appropriate autoconf/automake/autoreconf command
* autoreconf is a tool that automatically calls autotools as required to process the present working working directory. Autoreconf will use the versions of autotools in your path, so make sure to prepend(!) your custom build of autotools to the path variable. There are also supposedly environment variables that can be set.
* *.ac or *.in files are modified by pattern substitution. Lone spaces or tabs WILL cause issues later, since Makefiles are whitespace sensitive and the whitespace is never removed

=== Library Implementation ===
* In some situations, the <code>-DMISSING_SYSCALL_NAMES</code> flag must be set in `newlib_cflags` so that certain functions can call your syscalls as the underscore variant. I.e. sbrk() instead of _sbrk() in your syscalls.c. Otherwise at compile time, the symbol _sbrk will be reported as missing.
* The crt0.o object is overridden by the one generated by <code>libgloss/aarch64</code>. To use your crt file, make sure to override the existing one in your sysroot with the version from the <code>$NEWLIB_BUILD_DIR/$TARGET/newlib/</code> folder after installing newlib.

=== Build System Tips ===
* When porting aarch64 (and perhaps other platforms), not having *-elf at the end of your target string can lead to a circular dependency in one of the Makefiles, causing the build to fail on copying the .spec files. Make sure to add a match statement in the <code>case "${target}" in</code> case in `libgloss/aarch64/configure.in`.
* The build system was rewritten in later versions of NEWLIB. The instructions regarding the automake files will no longer apply. For the most part, it just means leaving out those parts, but be prepared to do your own detective work!

=== General Tips ===
* Copy-pasting may introduce aforementioned missing newlines or spurious spaces.
* Don't move your build artifacts. This may break dependencies.
* If you are stuck, try taking a look at other NEWLIB ports, such as for Jin Xue's https://github.com/Jimx-/lyos, for example.

== See Also ==

=== Articles ===
* [[GCC Cross-Compiler]]
* [[OS Specific Toolchain]]

[[Category:Porting]]
[[Category:C]]
[[Category:Standard Libraries]]
[[Category:Tutorials]]
````

<a id="w002127"></a>

## W002127 — User:Pcmattman

Bron-ID: W002127
Namespace: User (2)
Revisie: 19309 | Datum: 2016-06-02T10:35:34Z
Bron: https://wiki.osdev.org/User:Pcmattman
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APcmattman&oldid=19309

````text
== Online Presence ==

* Github: [https://github.com/miselin miselin]
* Website: [https://ideasandcode.xyz https://ideasandcode.xyz]

== My OS Projects ==

=== Pedigree ===

[https://www.pedigree-project.org https://www.pedigree-project.org]

Features:
* x86, x86_64, and ARM targets
* POSIX compatibility
* Full TCP/IP stack (with IPv6)
* bash, GCC/binutils, coreutils, Apache, and many other programs ported
* Python port

=== FORGE ===

Recently revived from an old git checkout.

[https://github.com/miselin/forgeos https://github.com/miselin/forgeos]

=== Rustic ===

No longer actively maintained, was an initial attempt at an OS in Rust (during the very early days of Rust - things have changed a lot!)

[https://github.com/miselin/rustic https://github.com/miselin/rustic]
````

<a id="w002128"></a>

## W002128 — Interrupts For Dummies

Bron-ID: W002128
Namespace: Main (0)
Revisie: 4317 | Datum: 2007-07-14T16:14:23Z
Bron: https://wiki.osdev.org/Interrupts_For_Dummies
Revisie-URL: https://wiki.osdev.org/index.php?title=Interrupts_For_Dummies&oldid=4317
Doorverwijzing: Interrupts

````text
#REDIRECT [[Interrupts]]
````

<a id="w002129"></a>

## W002129 — Talk:Interrupts For Dummies

Bron-ID: W002129
Namespace: Talk (1)
Revisie: 4319 | Datum: 2007-07-14T16:14:24Z
Bron: https://wiki.osdev.org/Talk:Interrupts_For_Dummies
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AInterrupts_For_Dummies&oldid=4319
Doorverwijzing: Talk:Interrupts

````text
#REDIRECT [[Talk:Interrupts]]
````

<a id="w002130"></a>

## W002130 — Talk:MASM

Bron-ID: W002130
Namespace: Talk (1)
Revisie: 9740 | Datum: 2010-02-10T03:20:14Z
Bron: https://wiki.osdev.org/Talk:MASM
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMASM&oldid=9740

````text
== Illegal? ==

Combuster... i Read the license at that link and though it prohibits writing software for non-Microsoft Operating System's it does not prohibit it's use in Operating System Development. Which part of the license are you Refering to? -- [[User:Tyler|Tyler]] 11:33, 23 July 2007 (CDT)
:''4. You cannot use the MASM32 Project to write software for Non-Microsoft Operating Systems.'' Because an operating system does not run on Windows, it's illegal. --Alboin
::Yes... as i said in my original question, and i quote, "it prohibits writing software for non-Microsoft Operating System's". Using it to write an Operating System (at least the Kernel) is not using it to write software for a non-Microsoft Operating System. Clearly no Operating System, is not a Non-Microsoft Operating System, and therefore no problem. -- [[User:Tyler|Tyler]] 19:53, 24 July 2007 (CDT)
:::The kernel is a piece of software, and it is part of an operating system. Arguably, you could see the OS as an abstract concept, and the kernel the most basic piece of software for that OS. Besides, you should know from the formulation that they ''intend'' to prohibit kernel development with that clause as well, which might be cause for the judge to favor microsoft anyway. Disclaimer: If you want to be sure seek legal council - I'm not a lawyer. Your best bet is still not to use MASM anyway - [[User:Combuster|Combuster]] 08:16, 27 July 2007 (CDT)
::::Yeah, that would seem logical, except Microsoft can afford lawyers, and there is no such thing as intention in a license. So i asked a software lawyer and he says (by my definition of Kernel, we didn't check the legal one) that the license does not prohibit use. I still wouldn't use it though! -- [[User:Tyler|Tyler]] 19:23, 27 July 2007 (CDT)
::Considering that the MASM project itself != the MASM32 project, there may be a loophole in that rule. As long as you don't use the MASM32 project itself, you might not be in the realm of illegality. --[[User:Troy martin|Troy Martin]] 03:20, 10 February 2010 (UTC)
There is a stipulation to the legality issue; If you purchase a copy, the rules are different. I cannot be precise about what they are, as there are so many different versions of the licence out there, and they do not specify which assembler version they apply to. Of course, if you purchase a copy, you will have the correct licence to go with it, and you may interpret it as you see fit. More than likely, it would alter the "open source" rule. Still, MASM may be used to write utilities that may be used for OS development (an example). I edited it, more or less for the syntax. If someone wants to convert a piece of MASM source to some other language, it may help. [[User:DednDave|DednDave]]

== MASM vs. MASM32 ==
Aren't [http://en.wikipedia.org/wiki/MASM MASM] and [http://en.wikipedia.org/wiki/MASM32 MSAM32] <i>entirely</i> different things? [http://forum.osdev.org/viewtopic.php?p=155820#p155820 This forum post] seems to imply that MASM is just fine to use with OS development! 

The link for the MASM homepage also links to the MASM32 homepage, further adding to the confusion. --[[User:Mikeroz|Mikeroz]] 20:26, 9 August 2009 (UTC)

MASM is, of course, the assembler from Microsoft. MASM32 is a project started by Steve Hutchesson that includes libraries and macros, as well as tutorials, reference material, examples, and a forum.[[User:DednDave|DednDave]]
````

<a id="w002131"></a>

## W002131 — Talk:Bare Bones

Bron-ID: W002131
Namespace: Talk (1)
Revisie: 30937 | Datum: 2026-09-02T14:52:31Z
Bron: https://wiki.osdev.org/Talk:Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABare_Bones&oldid=30937

````text
== Renaming ==

Shouldn't this page get renamed?
Because every other language's BareBone is called [Language Name] BareBone.
Only the C/C++ BareBone site is called BareBone.
I think it is reasonable, to rename it to C/C++ BareBone.
Then we could use the BareBone page for other things, like explaining why there are BareBones and so on.
[[User:Columbus|Columbus]] 08:50, 14 September 2014 (CDT)

I would've also thought that 'x86 Bare Bones' might also be more appropriate, since that is what you're actually building here. --[[User:Ajxs|Ajxs]] 23:59, 23 April 2017 (CDT)

== main -> kmain ==

I was the one who originally decided to use main() as function name. I changed my mind, because of the possibility of brain-dead compiler option choices resulting in a main()-related linker error, which might be less confusing if the function in the code is called kmain() instead. [[User:Solar|Solar]] 07:01, 27 February 2008 (CST)

== 64 bits ==

Hello from a new user.

I tried to assemble stuff from this article, but it was no go:
 loader.s:21: Error: suffix or operands invalid for `push'
 loader.s:22: Error: suffix or operands invalid for `push'
While browsing, I encountered indications it might be my architecture (64bit AMD). I know that cross-compiler would do the job (for 32 bit), but couldn't the code instead be written in such way, so that it is cross-architecture instead? That way, I'd have the solution for both architectures (no need for replacement code to this), and possibly wouldn't need to build a cross compiler at this point (very, very early).

Also, it would be very good if someone could explain, line by line, what's happening here.
<br>In any case, looks like a small, but a very important rewrite is in order.

<br>So, in short:
*Is it architecture specific?
*Any ideas about a workaround?
*Can someone explain the (new!) code?

--[[User:Paxcoder|Paxcoder]] 21:57, 5 January 2009 (UTC)



First of all, questions belong on the forum, not in some talkpage few people ever bother to read.

What you have is an assembler that defaults to 64-bit code, and assembly that is written for 32-bit code. To GAS those two are completely different architectures (i386 vs x86-64). There is however a command-line switch that would tell GAS to assemble for the correct architecture and as such needs no change in this code.
- [[User:Combuster|Combuster]] 22:45, 5 January 2009 (UTC)

::I hope you understand what you offered is not a solution. I want it to be able to run on 64 bit processors. Basically, you said very little. Nevermind. I'll use forum. --[[User:Paxcoder|Paxcoder]] 20:42, 6 January 2009 (UTC)


Added question at the bottom of the page with the options I use to build on my 64-bit Ubuntu 10.04 laptop. -- [[User:Mbrush|Mbrush]] 03:04, 21 May 2010 (UTC)

== Revert of -fno-leading-underscore ==

fixed minor mistake-meshounah
:Depending on the compiler, leading underscores are either required or wrong. Also, the summary field was designed for these messages - [[User:Combuster|Combuster]] 17:42, 11 August 2007 (CDT)
::We should add either <tt>-fleading-underscore</tt> or <tt>-fno-leading-underscore</tt> to the gcc command. I'd personally prefer <tt>-fno-leading-underscore</tt> and removing all underscores from the NASM and GAS code. Any opinions on the matter? -[[User:Jhawthorn|Jhawthorn]] 19:04, 15 August 2007 (CDT)
:::I'd go for suggesting that a [[GCC Cross-Compiler|cross-compiler]] is the only guarantee for success - it also fixes several other things one's system compiler might try to annoy the user (things commandline arguments won't fix). Since the crosscompiler described does not add leading underscores, and the use of having them is purely legacy, I'd go for no underscores anyway. - [[User:Combuster|Combuster]] 04:34, 17 August 2007 (CDT)

I reverted an edit from Hangin10 regarding -fno-leading-underscore. One of the advantages of the [[GCC Cross-Compiler]] is that it makes any tweaks regarding this option unnecessary. IMHO, if we start "leaking" such tweaks into this tutorial, we end up with a "Bare bones for Windows" and a "Bare bones for Linux" again. Or has there been a reason for this edit that I am unaware of? -- [[User:Solar|Solar]] 08:26, 19 January 2009 (UTC)
:'''Agreed''' If you use a windows oriented compiler you get stuck with all its windows oriented dependencies. Better not to silently allow a timebomb to be constructed on a windows platform by disabling leading underscores, and just have everybody use the crosscompiler from day one. - [[User:Combuster|Combuster]] 23:59, 19 January 2009 (UTC)

== FPU Support ==

I reverted an edit from Hangin10 where the 'finit' instruction was added. IMO, FPU initialization doesn't belong in Bare Bones, and a single 'finit' doesn't even begin to touch on anything else that is needed for proper FPU support. - [[User:Quok|quok]] 19:13, 29 September 2009 (UTC)

== Disk image ==

Why does this tutorial still use the kernel 200+18  method of loading?
Wouldn't it be better to teach how to create a FAT/ext2 GrUB boot disk?, hence simplifying debugging (and allowing the same code to be expanded to load multiboot modules, and be accessed by the kernel)
:People may not want to implement either of these file systems. They may not even want file systems at all. I find the current approach to be more general. --[[User:Love4boobies|Love4boobies]] 01:06, 27 November 2010 (UTC)

== Linkage ==

Is everyone comfortable with switching to external linkage rather than pushing things on the stack and then picking them up as arguments? The current way seems less natural, less readable, and is dependent on the calling convention. --[[User:Love4boobies|Love4boobies]] 04:17, 15 May 2012 (CDT)

:Back then, I thought it a good idea to show how stuff pushed by ASM can be popped by C. It was also closer to the example code provided by GRUB. But that was back when there ''was'' no x64 calling convention. In that light, I think external linkage ''would'' be better today. But then, try to make sure that you make the chance in ''all'' "Bare Bones" articles. We should strive to have them so things as similar as possible, the only differences being those demanded by the language. -- [[User:Solar|Solar]] 07:21, 15 May 2012 (CDT)

::I have updated the C and C++ tutorials but I'd rather someone else take care of the Pascal and FreeBasic tutorials. It's been about 10 years since I've last used Pascal and FreeBasic I do not know at all. --[[User:Love4boobies|Love4boobies]] 05:06, 28 May 2012 (CDT)

== Stack top & bottom ==

As far as I know, bottom of the stack is were the first element is added. In case of x86, it is the highest address. Compare with current content:

# Currently the stack pointer register (esp) points at anything and using it may
# cause massive harm. Instead, we'll provide our own stack. We will allocate
# room for a small temporary stack by creating a symbol at the '''bottom''' of it,
# then allocating 16384 bytes for it, and finally creating a symbol at the '''top'''.
.section .bootstrap_stack, "aw", @nobits
stack_'''bottom''':
.skip 16384 # 16 KiB
stack_'''top''':

I think all these occurrences of words bottom and top should be swapped. Do I miss something?

== Comment style ==
I've changed the commenting style from the old # ones to the new C-style ones using '''/*''' and '''*/''', hopefully this is OK, I thought it was a good idea since the # style is deprecated and is used to override line number / file name when errors are reported by the assembler. That said, if people can give a good reason as to why it should be changed back, by all means, go ahead, I'm just trying to help avoid headaches in the future. Also, I used my own commenting style (a single /* and */ before the first line and after the last line of a comment block), but I know people like other styles, so if you really feel strongly about one and think it helps readability, go ahead and use that. -- [[User:Mikumiku747|Mikumiku747]] 05:10, 19 July 2017 (CDT)

== Loading at 2M ==

A change was recently made to this article to switch the load address from 1M to 2M. This was based on the change author's real-world experiences with UEFI systems that did not mark the 1M region as available.

The change author's experience was not unique. Grub developers encountered the same problem, and added a feature to Multiboot2 specifically to address it: https://git.savannah.gnu.org/cgit/grub.git/commit/grub-core/loader/multiboot_mbi2.c?id=a620876e3b32e4ea0c9a7b3541fcb9a4dd4fb9eb

As the code in the barebones is not affected by this load offset, we should keep the updated 2M address. --[[User:Klange|klange]] 22:08, 11 November 2023 (CST)

== Deprecation of the tutorial ==

I would like to start a discussion on the deprecation of this article (I have added a Historical template, but it has been reverted). The motivation for doing that is that while this tutorial is "correct for what it is", it is using stuff that has been outdated for 30 or more years (multiboot 1, VGA text, no virtual memory...), and at this point, in my opinion, it is not even a good base for anything (unless you only target i386 retro hardware or something), and it is damaging and steers the people away from osdev by being ''the'' bare bones tutorial once they hit the inevitable roadblocks of it. I'm sorry for putting my frustration (not only with this tutorial, but the general stagnation of the wiki) out here, but the world has moved on, nobody has an Intel 386 with ancient VGA hardware in their PC (which would maybe warrant Multiboot 1 over Multiboot 2), and this tutorial, as it, is not even usable on most modern PCs (I am typing this on a not even that new of a machine, which has no VGA text mode).

I am of the opinion that at least having the Historical template is warranted for this page (this page was probably fine when it was written in 2009, but it is no longer so in 2026).

I also have personally been wanting and preparing to write a multiboot 2 barebones tutorials (for people who for some reason insist on using multiboot over a plethora of arguably better protocols), which would at least be a usable base for modern/64-bit PCs and can actually run outside of QEMU on hardware that people have, but have not had time to do so yet. --[[User:Mishakov|mishakov]] 2:42, 11 June 2026 (CST)

:I have been planning to upgrade this tutorial to Multiboot 2. There is no particular reason that it should be using Multiboot 1, actually. If it gains e.g. multiboot 2 + framebuffer support, then it would work quite well on modern EFI systems if loaded via GRUB 2. Font rendering and graphics are more complicated topics though.
:
:I might just go ahead and upgrade Bare Bones and Meaty Skeleton to Multiboot 2. I wanted to do it half a year ago, but I had some trouble with getting Multiboot 2 working for Sortix, but I should have some golden code now that can be a good reference.
:
:Note how VGA hardware is still commonly available via Qemu emulation, which is much recommended for initial testing. If we start bundling 64-bit / paging into the initial bare bones tutorial, then in my opinion, it becomes even harder to understand as a newcomer. I generally recommend that people complete this 32-bit tutorial before going 64-bit or tackling paging. I think the major problem is that we should have a good follow up tutorial to Bare Bones with these more advanced topics, and connect those tutorials together in a nice intended experience.
:
:You cannot deprecate this tutorial and consider it historical before you have a *replacement* available. Please feel free to write a new tutorial, or improve existing ones, but *please* *please* **please** get your changes peer reviewed by the wider community. There is a lot of misinformation and bad advice in almost all tutorials out there, and the osdev wiki base tutorials are different, because we have iterate on them for decades to get them perfect (for what they are). Please pop into the #osdev channel on irc.libera.chat and feel free to ping me if you want me to review your change, and I'll be very happy to lend a hand. :) --[[User:Sortie|Sortie]] ([[User talk:Sortie|talk]]) 20:35, 18 June 2026 (UTC)

Imo the [[Limine Bare Bones]] could be the indirect replacement; while limine protocol is somewhat opinionated (which for me for example doesn't make it very ergonomic), it does hide some amount of complexity from the beginners (hence why I added historical, there are alternatives already). I am personally not sure about the utility of starting with 32 bits either, I have seen people with different opinions on this. (but I agree that this whole thing, especially the follow-ups is not straightforward and does require a lot of discussion)

I have semi-recently added multiboot 2 to my kernel in preparation for the new tutorial (I had Limine and Ultra protocols), for both the AMD64 and IA32 ports of it, so I do think I have the working code that can be the base for the mb2 tutorial, that maps/gets the kernel to the higher half and enters C++.

However, the framebuffer situation is a bit sticky, since the other protocols map it for you and make it easier with libraries like franterm, but since multiboot doesn't (and considering that the framebuffer is sometimes not at <4GB), I feel like people have to do it themselves (my framebuffer/terminal driver is in userspace, so that my kernel doesn't even really touch it during the init so it's not a problem for me), but for QEMU in particular, COM0 could perhaps be used (which you would probably want for a kernel anyway), until the kernel can initialize the framebuffer...

I'm a bit busy right now so I'll probably come back to it in a week or so.
--[[User:Mishakov|Mishakov]] ([[User talk:Mishakov|talk]]) 23:05, 18 June 2026 (UTC)

I think a better formatted Limine Bare Bones that includes Flanterm could be a perfectly reasonable base for Bare Bones and meaty skeleton, it will start you on x86_64, make UEFI a possibility (unlike with multiboot1 where it can boot but its kinda, terrible), and also works in general on modern hardware far better, and makes SMP far easier. --[[User:Anotheridiot|Anotheridiot]]
````

<a id="w002132"></a>

## W002132 — Talk:Gas Intel Howto

Bron-ID: W002132
Namespace: Talk (1)
Revisie: 4346 | Datum: 2007-08-11T22:40:27Z
Bron: https://wiki.osdev.org/Talk:Gas_Intel_Howto
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AGas_Intel_Howto&oldid=4346

````text
== Copyright ==
Okay I have a couple of questions, Can we move that copyright notice to the bottom and it is okay to even have this page in the wiki since it apparently has a copyright? --[[User:Frank|Frank]] 11:34, 11 August 2007 (CDT)
:I should say that, if I was in the mind enough to care, I would rewrite this page just to remove the copyright. (The tutorial itself is quite small IMO, so it wouldn't pose much of a problem.) --Alboin

== Category ==
Also I am thinking about adding this page to tutorials. --[[User:Frank|Frank]] 11:51, 11 August 2007 (CDT)
:No objections. --Alboin
````

<a id="w002134"></a>

## W002134 — User:Meshounah

Bron-ID: W002134
Namespace: User (2)
Revisie: 4353 | Datum: 2007-08-15T22:52:42Z
Bron: https://wiki.osdev.org/User:Meshounah
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMeshounah&oldid=4353

````text
well hello my frisky little monkeys
````

<a id="w002135"></a>

## W002135 — User talk:Meshounah

Bron-ID: W002135
Namespace: User talk (3)
Revisie: 4354 | Datum: 2007-08-15T22:53:03Z
Bron: https://wiki.osdev.org/User_talk:Meshounah
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AMeshounah&oldid=4354

````text
talk about me all you want ;)
````

<a id="w002136"></a>

## W002136 — User:Jackscott

Bron-ID: W002136
Namespace: User (2)
Revisie: 29416 | Datum: 2025-02-19T05:24:58Z
Bron: https://wiki.osdev.org/User:Jackscott
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJackscott&oldid=29416

````text
Many moons ago I helped edit System Call Magazine: https://github.com/JackScottAU/SystemCallMagazine

There was only ever one edition produced: https://github.com/JackScottAU/SystemCallMagazine/raw/master/2008-01/2008-01.pdf

I'm the lead software developer at a software consultancy, https://loopfoundry.com.au

I'm writing my own little OS: [https://github.com/JackScottAU/Amethyst Amethyst]

I also do some work with old minicomputer history: [https://github.com/JackScottAU/SNOCOM SNOCOM]

=== Test Bed Machines ===

==== "Gresley" ====

* '''CPU:''' Intel Pentium II 266MHz (SL28L)
* '''MBD:''' Gigabyte GA-686LX4 (Chipsets: i440LX, PIIX4)
* '''RAM:''' 320MiB SDRAM
* '''GFX:''' NVidia Riva TNT2 Model 64 (AGP, 32MB)
** Also available: S3 Trio 3D/2X (AGP, 8MB) and ATi Rage 128 Pro (AGP, 16MB)
* '''NET:''' Realtek RTL8139C 10/100 (PCI)
* '''SND:''' ESS AudioDrive ES1868 (ISA)
* '''Storage:'''
** 3.5" 1.44MB Floppy Drive
** IDE 4.3GB Quantum Fireball ST
** IDE CD-ROM Drive

= Amethyst =
{{Infobox Project
|name=Amethyst

|image=Amethyst-Screenshot.png
|caption=Early Boot Process

|developers=[[User:Jackscott|Jack Scott]]

|kerneltype=Microkernel
|platform=x86
|taskmodel=Preemptive multitasking
|language=C, C++, Assembly

|website=[http://aquilaos.com aquilaos.com]
|github=jackscottau/amethyst

|license=ISC
}}
````

<a id="w002137"></a>

## W002137 — Talk:History

Bron-ID: W002137
Namespace: Talk (1)
Revisie: 4361 | Datum: 2007-08-16T04:22:20Z
Bron: https://wiki.osdev.org/Talk:History
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AHistory&oldid=4361

````text
Alrighty. I've been working on the History page for a while now. I've still got to edit the timeline, and add some more quotes and usenet announcements, etc. I hope nobody minds what I've done. [[User:Yayyak|Yayyak]] 23:22, 15 August 2007 (CDT)
````

<a id="w002139"></a>

## W002139 — Talk:Keyboard Input

Bron-ID: W002139
Namespace: Talk (1)
Revisie: 17656 | Datum: 2015-03-01T15:12:20Z
Bron: https://wiki.osdev.org/Talk:Keyboard_Input
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AKeyboard_Input&oldid=17656

````text

````

<a id="w002142"></a>

## W002142 — User:Stinkfly

Bron-ID: W002142
Namespace: User (2)
Revisie: 4474 | Datum: 2007-08-31T09:05:03Z
Bron: https://wiki.osdev.org/User:Stinkfly
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AStinkfly&oldid=4474

````text
<div style="border: 2px solid black; background: rgb(200, 200, 200); padding: 2px; margin: 2px; width: 100%;">
<div style="margin: 2px; padding: 5px; background: rgb(230, 230, 230); font-size: 300%; text-align: center">
Stinkfly
</div>
<p style="text-align: center">
Hello, i'm Stinkfly (not really, but wish I was), a [[wikipedia:Omnitrix#Stinkfly|Lepidopterran]] off the Cartoon Network show Ben 10.
</p>
'''About me'''
<table width="100%" style="background: none;"><tr><td width="50%">
I like:
* Werewolves
* Ben 10
* Tux + Linux
* My Girlfriend (don't ask)
* Goop
* Programming (PHP, C, Assemmbly *new*, C++)
* Dogs
</td><td>
I hate:
* Bad things
* Glasses
* Evil people
* Perl/Python
</td></tr></table>
<p style="text-align: center; font-size: 300%;">
[http://stinkflyos.wordpress.com My Site] [[Special:Contributions/Stinkfly|My Contribs]]
</p>
</div>
````

<a id="w002143"></a>

## W002143 — Talk:Executable Formats

Bron-ID: W002143
Namespace: Talk (1)
Revisie: 4419 | Datum: 2007-08-28T00:45:03Z
Bron: https://wiki.osdev.org/Talk:Executable_Formats
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AExecutable_Formats&oldid=4419

````text
== Pages For Review ==
I opt for removing the PFR tag and adding a Stub tag. Comments? - [[User:Combuster|Combuster]] 17:12, 27 August 2007 (CDT)
:By all means. Another page out of PFR is another fairy gaining her wings. --Alboin
````

<a id="w002144"></a>

## W002144 — Talk:Time And Date

Bron-ID: W002144
Namespace: Talk (1)
Revisie: 7836 | Datum: 2009-05-02T23:19:50Z
Bron: https://wiki.osdev.org/Talk:Time_And_Date
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ATime_And_Date&oldid=7836

````text
== Pages For Review ==
As far as I can see, this page needs to be categorized and checked before removal from PFR. My suggestion is [[:Category:OS theory]], agreed? - [[User:Combuster|Combuster]] 05:37, 28 August 2007 (CDT)
:It does seem to fit in nicely with that group, so yup, agreed. --Alboin

== Original Discussion ==
ToDo: This is a first draft and should probably be checked. It might be a good idea to move all of the RTC details into a seperate "RTC Hardware" page and add more info ... -- Brendan 

There is also the CMOS page - maybe details about accessing it should be moved there? It's a bit lacking, too (the CMOS page, I mean). You just raised some interesting issues, like the danger of reading CMOS during updates.. --ZomgTest

The RTC info is completely x86 specific, and needs to be separated and moved somewhere into the x86 hardware category.<br>
Also, other time/date formats with a more scientific basis (such as Julian Day Numbers) need to be mentioned. [[User:Bewing|Bewing]] 23:19, 2 May 2009 (UTC)
````

<a id="w002146"></a>

## W002146 — How Do I Test

Bron-ID: W002146
Namespace: Main (0)
Revisie: 4458 | Datum: 2007-08-31T05:50:07Z
Bron: https://wiki.osdev.org/How_Do_I_Test
Revisie-URL: https://wiki.osdev.org/index.php?title=How_Do_I_Test&oldid=4458
Doorverwijzing: Testing

````text
#REDIRECT [[Testing]]
````

<a id="w002147"></a>

## W002147 — Talk:How Do I Test

Bron-ID: W002147
Namespace: Talk (1)
Revisie: 4460 | Datum: 2007-08-31T05:50:07Z
Bron: https://wiki.osdev.org/Talk:How_Do_I_Test
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AHow_Do_I_Test&oldid=4460
Doorverwijzing: Talk:Testing

````text
#REDIRECT [[Talk:Testing]]
````

<a id="w002148"></a>

## W002148 — Category:In Progress

Bron-ID: W002148
Namespace: Category (14)
Revisie: 4899 | Datum: 2007-11-19T16:15:55Z
Bron: https://wiki.osdev.org/Category:In_Progress
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AIn_Progress&oldid=4899

````text
A list of pages that are currently being worked on.

[[Category:Wiki administration]]
````

<a id="w002149"></a>

## W002149 — Talk:Unit Testing

Bron-ID: W002149
Namespace: Talk (1)
Revisie: 13802 | Datum: 2012-07-10T15:08:52Z
Bron: https://wiki.osdev.org/Talk:Unit_Testing
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AUnit_Testing&oldid=13802

````text
Should this page be merged into a section within [[Testing]]? It could be put in the advanced matters section, but it could also be argued that Unit testing is a seperate enough topic to deserve it's own page. [[User:Yayyak|Yayyak]] 01:56, 31 August 2007 (CDT)
:For now, keep them separate. This article is as large as the entire testing page and merging now would give the idea that two articles are crammed into one page. - [[User:Combuster|Combuster]] 03:20, 31 August 2007 (CDT)

Regarding [[User:Combuster|Combuster]]'s last two edits, I don't think that's what the original author of the article had in mind. Notice that the two bullet points explain how unit testing can be used to improve the maintainability of a project, not how to develop or maintain the actual unit tests. IMO, the previous version did a better job at emphasizing what unit tests are useful for. The options are:

* To leave it as-is.
* To revert to the version before.
* To revert but also have a section that covers the use of unit tests (however, is this the place?).

--[[User:Love4boobies|Love4boobies]] 10:08, 10 July 2012 (CDT)
````

<a id="w002153"></a>

## W002153 — FreeBasic

Bron-ID: W002153
Namespace: Main (0)
Revisie: 4486 | Datum: 2007-08-31T12:50:30Z
Bron: https://wiki.osdev.org/FreeBasic
Revisie-URL: https://wiki.osdev.org/index.php?title=FreeBasic&oldid=4486
Doorverwijzing: FreeBASIC

````text
#REDIRECT [[FreeBASIC]]
````

<a id="w002155"></a>

## W002155 — FreeBasic Bare Bones

Bron-ID: W002155
Namespace: Main (0)
Revisie: 30662 | Datum: 2026-05-28T15:43:25Z
Bron: https://wiki.osdev.org/FreeBasic_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=FreeBasic_Bare_Bones&oldid=30662

````text
{{BadPractice}}
{{BeginnersWarning}}
{{Rating|2}}
While the forum has several flamewars about BASIC, it is a turing-complete language. FreeBasic adds functionality that makes it suitable for OS development. This tutorial provides a working kernel in Basic, together with the pitfalls associated with it.

== About FreeBasic and the Runtime ==
[[FreeBasic]] is a BASIC compiler with syntax compatible to QuickBasic, with several improvements that make it a viable compiler for Operating System Development. However, normal Basic programs rely heavily on part of the language that is called the Runtime. The Runtime consists of all operations that cannot be performed by a set of basic processor instructions. When engaging in operating systems development, you must be aware that you can not use anything that is part of the Runtime. That includes:
* String operations. You can define strings, but not use them in any regular way.
* Dynamic-sized arrays. Only arrays with a fixed size can be used, and even then some operations won't work on them.
* Any input or output statements. Normally the program communicates with the OS about these things, but since we are ''writing'' the OS, these things can't be used.
When you get messages about undefined symbols starting with fb_, it means you are using parts of the Runtime.

To compensate, FreeBasic provides pointers that can be used to perform functionality the runtime would provide. Inline assembly can be used as well.


== Pointers for Beginners ==
Pointers can be difficult to understand. Many modern languages do not work with pointers, while if you know a language like C, C++ or Assembly, you will probably know what they do. Since you will need to use them, a quick introduction in how they work in FreeBasic.

The computer uses a set of registers, and memory. Memory is divided into bytes, each byte has its own location number. Big numbers, strings, types and arrays use up multiple bytes. These bytes are stored next to each other. A Long will hold 4 bytes, and will for example occupy locations 239394, 239395, 239396 and 239397. In the computer, these location numbers are called addresses (like the address when sending letters)

Pointers hold these location numbers. For bytes, they hold the exact address, for larger objects, it will hold the address of the lowest address (you can determine the other addresses as they will immediately follow the first)

In FreeBasic, a pointer is defined by adding the Ptr keyword.
<syntaxhighlight lang="vbnet">
 Dim mypointer As Byte Ptr
</syntaxhighlight>
This one holds the location of a byte of memory.
<syntaxhighlight lang="vbnet">
 Dim mypointer As Long Ptr
</syntaxhighlight>
This one points to the first of 4 bytes of memory, which together form the number.

You can not use pointers straight away. Consider sending a letter with no address on it will not do any good. So we must first put an address in there. That leads to the question: how do we get an address. Some things have fixed addresses, like the video card. 
You can also ask variables for their addresses.
<syntaxhighlight lang="vbnet">
 Dim variable as Byte
 Dim pointer as Byte Ptr
 pointer = @variable
</syntaxhighlight>
The @ returns the address of the variable that follows it.
<syntaxhighlight lang="vbnet">
 Dim pointer as Byte Ptr
 Dim address as Long
 address = 12345
 pointer = CPtr(Byte Ptr, address)
</syntaxhighlight>
Here CPtr (Convert to Pointer) is used to create pointers. You give it a type (Byte Ptr) and the address. You can also change pointers:
<syntaxhighlight lang="vbnet">
 Dim pointer1 as Byte Ptr
 Dim pointer2 as Long Ptr
 pointer2 = CPtr(Long Ptr, pointer1)
</syntaxhighlight>
Be careful when you do so: a byte occupies one location in memory, a long occupies four. if we would use this pointer, we would use three locations in memory of which we know nothing about. Sometimes, they are necessary, like when we want to work with strings without having the Runtime.

You can access pointers with an index:
<syntaxhighlight lang="vbnet">
 Dim value as Long
 Dim pointer as Long Pointer
 pointer = @value
 ' value = pointer[0]
</syntaxhighlight>
when we want to know what is behind the pointer, we ask for the memory at its location with the [ ], and then we add a number which tells us how many locations further to look. When we use [0] it simply means that we do not want to do anything with the address stored in the pointer. The result is the value stored in the memory locations pointed to by the pointer. For byte pointers, this will look at the address stored and return the result. For other pointers, it will look at a series of locations and return what those values represent.

Strings are useful, but tedious. A string is stored as a series of bytes in consecutive addresses. You can build a pointer to a string as well:
<syntaxhighlight lang="vbnet">
 Dim s as String
 Dim pointer as String Ptr
 pointer = @s
</syntaxhighlight>
But since we do not have the Runtime, we have to use something that does not use it.
<syntaxhighlight lang="vbnet">
 Const s = "Text"
 Dim pointer as Byte Ptr
 pointer = CPtr(Byte Ptr, @s)
 ' pointer[0] = Asc("T")
 ' pointer[1] = Asc("e")
 ' pointer[2] = Asc("x")
 ' pointer[3] = Asc("t")
 ' pointer[4] = 0
</syntaxhighlight>
The first example uses the Runtime which will not compile. The second example shows the only working method of using strings: Define one using Const, then ask for a pointer to that. Since a string is a sequence of characters (bytes), we change the type of the pointer in the process. Next we can ask for individual characters as if it were an array. However, they now appear as numbers. The ASCII codes to be precise, and the same as when you use Chr$() and Asc().

Basic uses an borrowed trick to tell us the end of the string. After the last character, there will always be a 0. So if we read a string in order, we can tell when it has ended.

<syntaxhighlight lang="vbnet">
 Sub PrintString(src As Byte Ptr, x As Long, y As Long)
 
    Dim dst as Byte Ptr
    Dim counter as Long
 
    dst = CPtr(Byte Ptr, &HB8000 + y * 160 + x * 2)
 
    counter = 0
 
    while src[counter] <> 0
        dst[2 * counter] = src[counter]
        dst[2 * counter + 1] = 15
        counter = counter + 1
    wend
 End Sub
</syntaxhighlight>

To conclude, this function prints a string (a converted Byte Ptr string). It creates a pointer that is aimed at the [[Text UI|video card]] (it occupies among others a range starting from address B8000 hex), and we pick a location in there. Next we take a character from the string, check if its 0, and if it isn't, copy it to the video card and go to the next character. Due to the way the video card works, we add a color (15) as well.

== Tuning FreeBasic for OS Development ==
A normal install of FreeBasic is in most cases configured to build for the system it was installed on. That means that it will try to compile programs for linux or windows OSes, rather than your own. 

To fix this, you should build at the very least a [[GCC Cross-Compiler|crosscompiling binutils]] (2.17 or later recommended). While you're at it, you can also build GCC, which comes in handy once you start porting software written in C (that includes the runtime).

FreeBasic stores its auxiliary binaries in the bin directory, or a subdirectory thereof. To get started quickly, replace <code>ld</code>, <code>ar</code> and <code>as</code> with the versions built in the previous step. Note that this stops FreeBasic from working on its previous host.
Under windows, the bin directory is subdivided into platforms, and you can add a new one rather than modifying the existing one. To use this, create the 'linux' directory, copy <code>i586-elf-ld</code>, <code>i586-elf-ar</code> and <code>i586-elf-as</code>, and rename them to <code>ld</code>, <code>ar</code> and <code>as</code>. When running freebasic you can use the <code>-t linux</code> command line switch to compile for your OS instead of windows, allowing you to continue using Freebasic normally for non-os development. The Linux version does not come with this target switch - here you'll have to replace the bundled binaries with your own versions.

== The code ==
Here is the entire project for reference

=== kernel.bas ===
<syntaxhighlight lang="vbnet">
Declare Sub PrintString(src As Byte Ptr, x As Long, y As Long)
Declare Sub main ()

Sub multiboot ()
   Asm

     'setting up the Multiboot header - see GRUB docs for details
    
     .set ALIGN,    1<<0             
     .set MEMINFO,  1<<1             
     .set FLAGS,    ALIGN | MEMINFO  
     .set MAGIC,    0x1BADB002       
     .set CHECKSUM, -(MAGIC + FLAGS) 

     .align 4
     .long MAGIC
     .long FLAGS
     .long CHECKSUM

     .set STACKSIZE, 0x4000          
     .comm stack, STACKSIZE, 32      
    
     .global loader                  

     loader:
       lea   esp, stack + STACKSIZE
       push  eax                       
       push  ebx                       

       call  MAIN

       cli
       hlt                    
   End Asm

End Sub

Sub main ()
    Const s = "Hello World"

    PrintString CPtr(Byte Ptr, @s), 35, 12

End Sub

Sub PrintString(src As Byte Ptr, x As Long, y As Long)

    Dim dst as Byte Ptr
    Dim counter as Long

    dst = CPtr(Byte Ptr, &HB8000 + y * 160 + x * 2)

    counter = 0
 
    while src[counter] <> 0
        dst[2 * counter] = src[counter]
        dst[2 * counter + 1] = 15
        counter = counter + 1
    wend
End Sub
</syntaxhighlight>

=== link.ld ===
<pre>
OUTPUT_FORMAT("elf32-i386")
ENTRY (loader)

SECTIONS{
    . = 0x00100000;

    .text :{
        KERNEL_START = .;

        *(.text)
    }

    .rodata ALIGN (0x1000) : {
        *(.rodata)
        _CTORS = .;
        *(.ctors)
        _CTORS_END = .;       
    }

    .data ALIGN (0x1000) : {
        *(.data)
    }

    .bss : {
        SBSS = .;
        *(COMMON)
        *(.bss)
        EBSS = .;

        KERNEL_END = .;
    }
}
</pre>

=== Build instructions ===
<syntaxhighlight lang="shell-session">
$ fbc -c kernel.bas -o kernel.o
$ i586-elf-ld -T link.ld -o kernel.bin kernel.o
</syntaxhighlight>

<tt>kernel.bin</tt> can then be loaded by [[GRUB]]

== Getting the Runtime to work ==
Once you progress with your kernel, you can try to get runtime functionality to work.

The Runtime is built on top of the C library. FreeBasic provides sources to the Runtime (which are written in C). You'll need to provide the C library. Since many parts of the kernel rely heavily upon each other, you will probably add the Runtime in steps.
Some functions compile cleanly with minimal effort. When you compile the CType, String and Stdlib (excluding malloc, calloc, and free) parts of the runtime can be compiled on top of those. 
Before you can use the string and array instructions, you will need to have [[Memory Management|memory management]] implemented. That will allow you to compile malloc, calloc, free from the C library, after which the string and some of the array functions can be compiled.

[[Category:Bare bones tutorials]]
[[Category:Inadvisable]]
````

<a id="w002156"></a>

## W002156 — Talk:Other Compilers

Bron-ID: W002156
Namespace: Talk (1)
Revisie: 24920 | Datum: 2020-06-23T10:14:12Z
Bron: https://wiki.osdev.org/Talk:Other_Compilers
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AOther_Compilers&oldid=24920

````text
I'm making this into articles in Category:Compilers --[[User:Stinkfly|Stinkfly]] 04:10, 31 August 2007 (CDT)
:'''Stop''' Please discuss before passing through a complete reorganisation. This is IMHO not necessary yet, please work on the pages for review status first. - [[User:Combuster|Combuster]] 07:55, 31 August 2007 (CDT)
::Sorry, won't do it agian! --[[User:Stinkfly|Stinkfly]] 08:12, 31 August 2007 (CDT)

This seems to me to be an echo of the other languages part of [[Languages]]. I don't believe they should be merged, but is there some way we can reduce the amount of duplicate content? [[User:Yayyak|Yayyak]] 19:27, 31 August 2007 (CDT)
:What about keeping the list of viable compilers and move (merge) the rest to [[languages]]? - [[User:Combuster|Combuster]] 15:08, 4 September 2007 (CDT)

== Proposal to rename & slightly repurpose ==

I propose renaming this page to "List of Compilers" and including a very brief summary of every compiler. Other lists could be added on the page so we keep the present content as Summaries and add terser lists for language, target architecture, host architecture, and perhaps license. My goal is to make a quick-reference to save time and effort when choosing a compiler. This would ease maintenance a little because we wouldn't be trying to exclude things in the compilers category. I see it presently includes some compilers which have their own page and are or should be in the category. Another compiler ([[LCC]]) which is in the category but not this page has a one-line page more suitable for this list. Regarding "very brief", my own 6-sentence summary of "kencc" is perhaps a longer than desirable. I'll make a page for it at some point. [[User:Eekee|Eekee]] 05:14, 23 June 2020 (CDT)
````

<a id="w002157"></a>

## W002157 — User talk:Stinkfly

Bron-ID: W002157
Namespace: User talk (3)
Revisie: 4490 | Datum: 2007-08-31T13:09:04Z
Bron: https://wiki.osdev.org/User_talk:Stinkfly
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AStinkfly&oldid=4490

````text
Will you please [[Talk:Other Compilers|discuss]] things before turning things upside down, thanks - [[User:Combuster|Combuster]] 07:59, 31 August 2007 (CDT)
:Also, please read http://en.wikipedia.org/wiki/Help:Minor_edit - [[User:Combuster|Combuster]] 08:09, 31 August 2007 (CDT)
````

<a id="w002158"></a>

## W002158 — Talk:WinImage

Bron-ID: W002158
Namespace: Talk (1)
Revisie: 4497 | Datum: 2007-09-01T00:48:55Z
Bron: https://wiki.osdev.org/Talk:WinImage
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AWinImage&oldid=4497

````text
On WinImage 8.0.8000 there's a bug and you can continue using it past it's expiry date.
--[[User:MessiahAndrw|MessiahAndrw]] 19:48, 31 August 2007 (CDT)
````

<a id="w002159"></a>

## W002159 — Category talk:Tools

Bron-ID: W002159
Namespace: Category talk (15)
Revisie: 4520 | Datum: 2007-09-04T21:40:44Z
Bron: https://wiki.osdev.org/Category_talk:Tools
Revisie-URL: https://wiki.osdev.org/index.php?title=Category_talk%3ATools&oldid=4520

````text
== Template to install ==

<!-- toolbox -->
<div style="float: right; border: 2px solid black; background: rgb(200, 200, 200); padding: 2px; margin: 2px;">
'''Install NASM'''<br />
<div style="padding: 2px; background: rgb(300, 100, 100);">
'''Debian based systems: '''<pre>apt-get install nasm</pre>
</div><br />
<div style="padding: 2px; background: rgb(300, 100, 100);">
'''Source Code: (get [http://404.com here] and extract)'''<pre>./configure
make
sudo make install</pre>
</div>
</div>
Hi, do you think it's a good idea to make a template showing how to install each tool on each system (e.g debian: apt-get install mytoolname) It could look like the toolbox i've constructed, displayed on the right. --[[User:Stinkfly|Stinkfly]] 05:45, 2 September 2007 (CDT)

:I think the users of a given system know ''very well'' how they should install things. This mainly because you need to have some experience on the matter to be elegible for os development in the first place. Its hardly a frequently asked question either. - [[User:Combuster|Combuster]] 15:15, 4 September 2007 (CDT)
::I have to agree with Combuster. This wiki's not here to teach how to use an OS, but how to make one. ;) --Alboin
````

<a id="w002160"></a>

## W002160 — Libsupcxx

Bron-ID: W002160
Namespace: Main (0)
Revisie: 17502 | Datum: 2015-01-28T12:50:16Z
Bron: https://wiki.osdev.org/Libsupcxx
Revisie-URL: https://wiki.osdev.org/index.php?title=Libsupcxx&oldid=17502

````text
{{In_Progress}}
{{Disputed|Talk:Libsupcxx}}
{{Rating}}
Libsupc++ is a support library for g++ that contains functions dealing with run-time type information (RTTI) and exception handling.  If you attempt to use either exceptions or RTTI in a C++ kernel you have compiled with a [[GCC Cross-Compiler]] you will also need the libsupc++ library.  In general, you should be able to use the one provided as part of a Linux distribution.  If, however, you run into problems and need to compile your own, you can follow these steps.

== Compiling libsupc++ ==

=== Create a working [[GCC Cross-Compiler]]. ===
This tutorial assumes it is entitled 'i686-elf-gcc'

=== Configure gcc ===
Enter the gcc source directory, run

    ./configure --target=i686-elf --prefix=/usr/local/cross --enable-languages=c,c++ \
        --without-headers --disable-nls
    cd libstdc++-v3

=== Edit the libstdc++ configure script ===
Now you need to edit the configure file in the libstdc++-v3 directory.  Open it up in the editor of your choice (which preserves unix style line endings) and find a section similar to (it is around line 108,000 in gcc 4.2.1, searching for 'combination' is probably the easiest way to find it):

    { { echo "$as_me:$LINENO: error: No support for this host/target combination." >&5
    echo "$as_me: error: No support for this host/target combination." >&2;}
    { (exit 1); exit 1; }; }
    ;;

and alter the third line so that it reads:

    { { echo "$as_me:$LINENO: error: No support for this host/target combination." >&5
    echo "$as_me: error: No support for this host/target combination." >&2;}
    }
    ;;

=== Configure and make libsupc++ ===

    CPP=i686-elf-cpp ./configure --host=i686-elf --prefix=/usr/local/cross --disable-hosted-libstdcxx \
        --disable-nls
    cd include
    make
    make install
    cd ../libsupc++
    make
    make install

=== Usage ===
Libsupc++ should now be installed into /usr/local/cross/lib.  To use it, you will need to add

    -L/usr/local/cross/lib -lsupc++

to your linker command line.

=== Additional requirements ===
Libsupc++ also requires that [[libgcc]].a be included in your link as well.  This is usually found (if you followed the cross compiler directions) in /usr/local/cross/lib/gcc/i686-elf/<gcc version>.  Finally, it has a number of dependencies which your kernel must provide, including (but not limited to) malloc, free, abort and strlen.

=== Tested on ===
These steps were tested on g++ 4.2.1 under Cygwin with a cross compiler targeting i686-elf

== Full C++ Runtime Support Using libgcc And libsupc++ ==

The following description is valid for i386, GCC 3.2 and [[libgcc]]/libsupc++ compiled for Linux/glibc (you can use the static libgcc/libsupc++ libraries compiled for your Linux for your kernel).

If you want Exceptions, RTTI, new and delete altogether, you also could use libgcc and libsupc++. libgcc contains the unwinder (for exceptions), while libsupc++ contains the C++ support. These functions look very complex (gcc_sources/gcc/unwind*, gcc_sources/libstdc++-v3/libsupc++/*), so it might be better to port them instead of trying to write them yourself.

To get full C++ support, you only have to do the following:

* Provide some libc functions (e.g. abort, malloc, free, ...) because libsupc++ needs them. There are even more functions you could support, like pthread_*, but since these are weak symbols, you don't need to define them.
* There's also a strange function dl_iterate_phdrs. You don't need this so let it simply return -1. It's usually used to find exception frames for dynamically linked objects. You could also remove calls to this function from the library.
* To make use of exception handling, you also have to tell libsupc++ where the '''.eh_frame''' section begins. Before you throw any exception: <verbatim>__register_frame(address_of_eh_frames); </verbatim>.
* Terminate the '''.eh_frame''' section with 4 bytes of zeros (somehow). If you forget this, libsupc++ will never find the end of '''.eh_frame''' and generate stupid page faults.

Please note that you still have to call the constructors/destructors by yourself as documented in [[Calling Global Constructors]].

== Linking a kernel with libsupc++ ==

You can use your libsupc++ to get exception handling and RTTI in a C++ kernel (no more passing -fno-exceptions -fno-rtti to g++!) so you can use things like throw and dynamic_cast<>.  Libsupc++ depends upon [[libgcc]] for stack unwinding support.  Passing the -nostdlib option to gcc when linking caused libgcc.a and libsupc++.a to not be included, so you need to specify -lgcc -lsupc++ on the command line (no need to specify the directories; gcc knows where it installed them to).  In addition, you need to include a .eh_frame section in your linker script and terminate it with 32 bits of zeros (QUAD(0) is a useful linker script command).  The symbol start_eh_frame should point to the start of the eh_frame section, and it should be aligned by 4.  In addition you need to include your constructors and destructors in the link (see [[C++]] for details).  You also need to provide __register_frame() (or call the function provided by libgcc with the start of your .eh_frame section), void *__dso_handle;, __cxa_atexit() and __cxa_finalize (again see [[C++]]).  Something along the lines of
  #include <reent.h>
  static struct _reent global_reent;
  struct _reent *_impure_ptr = &global_reent;
somewhere in your kernel will keep libgcc happy, because it expects these bits to be provided by the standard library (which you aren't linking into your kernel - but you can provide them in your libk).  Libgcc expects a number of (simple) C library functions to be provided by your kernel, including abort, malloc, free, memcpy, memset and strlen.  Libsupc++ also requires write, fputs, fputc, fwrite, strcpy and strcat for debugging output.

[[Category:Tutorials]]
````

<a id="w002161"></a>

## W002161 — Category:Interrupts

Bron-ID: W002161
Namespace: Category (14)
Revisie: 4525 | Datum: 2007-09-06T04:33:36Z
Bron: https://wiki.osdev.org/Category:Interrupts
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AInterrupts&oldid=4525

````text
Information about interrupts on x86 systems.
PIC, IRQ, IDT, etc are covered here. (or they will be in future).

[[Category:Hardware]]
[[Category:X86]]
````

<a id="w002163"></a>

## W002163 — Category talk:Tutorials

Bron-ID: W002163
Namespace: Category talk (15)
Revisie: 4542 | Datum: 2007-09-08T21:20:30Z
Bron: https://wiki.osdev.org/Category_talk:Tutorials
Revisie-URL: https://wiki.osdev.org/index.php?title=Category_talk%3ATutorials&oldid=4542

````text
I am not sure exactly who the target of the tutorials category is aimed at, nor at to what the content is expected to be.  I have added a few tutorials on cross compiling binutils and libsupc++, and there is also a porting newlib topic here.  These are compeltely different types of tutorials from the barebones kernel tutorials, and I was wondering whether it would make more sense to have different categories, e.g. beginner's kernel tutorials (getting started, protected mode, printing to the screen), more advanced tutorials (scheduling, ipc etc) and then user mode tutorials which the libsupc++, newlib and possible binutils tutorials could go to.
[[User:Jnc100|John]] 16:20, 8 September 2007 (CDT)
````

<a id="w002164"></a>

## W002164 — OS Specific Toolchain

Bron-ID: W002164
Namespace: Main (0)
Revisie: 30281 | Datum: 2026-03-16T18:01:39Z
Bron: https://wiki.osdev.org/OS_Specific_Toolchain
Revisie-URL: https://wiki.osdev.org/index.php?title=OS_Specific_Toolchain&oldid=30281

````text
{{Rating|3}}

This tutorial will guide you through creating a toolchain comprising Binutils and GCC that specifically targets your operating system. The instructions below teach Binutils and GCC how to create programs for a hypothetical OS named 'MyOS'. 

Until now you have been using a [[GCC Cross-Compiler|cross-compiler]] configured to use an existing generic bare target. This is very convenient when starting out as you get a reliable target and the compiler doesn't make any bad assumptions because it thinks it is targeting an existing operating system. However, when you proceed it becomes useful if the compiler knows it is targeting your operating system and what its customs are. For instance, you can make the compiler define a <tt>__myos__</tt> preprocessor macro, know which directories to search for include files in, what special <tt>crt*.o</tt> files are used when linking against libc, and so on. It also becomes much easier to cross-compile software to your OS when you simply have to invoke <tt>x86_64-myos-gcc hello.c -o hello</tt> to cross-compile a program. Additionally part of the instructions here can be applied to other software packages that also use the GNU build system, which will help you [[Cross-Porting Software|port existing software]].

This tutorial teaches you how to set up a cross-compiler that specifically targets your OS. This is actually the first step of ''porting Binutils and GCC to your operating system'': Any information you give GCC about your OS will help it run on your OS. Once your OS Specific Toolchain has been set up and you have built your OS with it, you can continue by using the cross-compiler to cross-compile the compiler itself to your OS, assuming your libc and kernel is powerful enough. For more information see [[Porting GCC to your OS]].

== Introduction ==

You need the following before you get started:

* A build environment that can successfully build a [[GCC Cross-Compiler]].
* autoconf (exactly version 2.69)
* automake (exactly version 1.15.1)
* libtool
* The latest Binutils source code (2.39 at time of writing).
* The latest GCC source code (12.2.0 at time of writing).
* Knowledge of the internals of Binutils and GCC.
* Knowledge of autoconf and automake.
* The dependencies of Binutils and GCC as detailed in [[GCC Cross-Compiler]].

If you're compiling a later version of GCC/binutils, find versions of autoconf and automake that were released just prior to your version of GCC/binutils.  See the [[Cross-Compiler Successful Builds]] page for known compatible versions of GCC and binutils.

Additionally you will need a [[C Library]] as described in a later section. As detailed in [[Hosted GCC Cross-Compiler]], it doesn't need to support much and the functionality can be stubbed, but libgcc will need to believe you have a libc.

You should decide exactly what targets you'll add to Binutils and GCC. If you have been using a generic <tt>i686-elf</tt> or <tt>x86_64-elf</tt> or such target, you'll simply want to swap <tt>-elf</tt> with <tt>-myos</tt> and get <tt>i686-myos</tt> and <tt>x86_64-myos</tt>. Naturally, don't actually write myos, but rather use the name of your OS converted to lower case. See [[Target Triplet]].

This tutorial currently only have instructions for adding a new x86 and x86_64 target for myos, but it serves as a good enough example that it should be trivial to add more processors by basing it on these instructions and what other operating systems have done.

== Making your changes reproducible ==
Since at some point you might have a contributor that wants participate in your project, you likely want to make your toolchain setup reproducible. Instead of downloading the tar-balls, it is therefore a good idea to clone the repositories of binutils (https://sourceware.org/git/binutils-gdb.git) and gcc (https://gcc.gnu.org/git/gcc.git) locally.

All release versions are tagged within these repositories, so you can use <tt>"git checkout binutils-2_39"</tt> and <tt>"git checkout releases/gcc-12.2.0"</tt> to switch to the correct versions respectively. After making the described changes, you can easily create a patch using <tt>"git diff"</tt> which you can reuse later.
Another option would be to fork one of the mirror repositories on GitHub.

== Modifying Binutils ==

=== config.sub ===

This is a file you will modify in the same way for each package.  It is a GNU standard file produced by including the line 'AC_CANONICAL_SYSTEM' in a configure.ac that is processed by autoconf, and is designed to convert a canonical name of the form <tt>i686-pc-myos</tt> into separate variables for the processor, vendor and OS, and also rejects systems it doesn't know about.  We simply need to add 'myos' to the list of acceptable operating systems.  Find the section that begins with the comment "<tt>Now accept the basic system types</tt>" (it begins '<tt>-gnu*</tt>') and add '<tt>-myos*</tt>' to the list.  Find a line with some free room and add your entry there.

=== bfd/config.bfd ===

This file is part of the configuration for libbfd, the back-end to Binutils which provides a consistent interface for many object file formats.  Generally, each platform-specific version of Binutils contains a libbfd which only supports the object files normally in use on that system, as otherwise the library would be massive (libbfd can support a _lot_ of object types).  We need to associate our os with some particular object types.  There is a long list starting 'WHEN ADDING ENTRIES TO THIS MATRIX' with the first line as 'case "${targ}" in'.  We need to add our full canonical name to this list, by adding some cases such as:

<syntaxhighlight lang="bash">
  i[3-7]86-*-myos*)
    targ_defvec=i386_elf32_vec
    targ_selvecs=
    targ64_selvecs=x86_64_elf64_vec
    ;;
#ifdef BFD64
  x86_64-*-myos*)
    targ_defvec=x86_64_elf64_vec
    targ_selvecs=i386_elf32_vec
    want64=true
    ;;
#endif
</syntaxhighlight>

'''Note:''' If using binutils-2.24 or older, change <i>i386_elf32_vec</i> to <i>bfd_elf32_i386_vec</i> and <i>x86_64_elf64_vec</i> to <i>bfd_elf64_x86_64_vec</i>.

Be sure to follow the instructions in the comment block above the list and add your entry beneath the comment "<tt>#START OF targmatch.h</tt>". If you like, you could support different object formats (look at other entries in the list, and the contents of 'bfd' for hints) and also provide more than one to the <tt>targ_selvecs</tt> line. For instance, you can support coff object files if you add <tt>i386coff_vec</tt> to the <tt>targ_selvecs</tt> list. All the <tt>x86_64</tt> entries in the file file are wrapped in <tt>#ifdef BFD64</tt>, as when targmatch.sed processes this file to turn it into to targmatch.h, it will add <tt>#ifdef BFD64</tt> so that the relevant code is only compiled when targeting a 64 bit platform

=== gas/configure.tgt ===

This file tells the gnu assembler what type of output to generate for each target.  It automatically matches the i686 part of your target and generates the correct output for that.  We just need to tell it what type of object file to generate for myos.  In the section starting '<tt>Assign object format ... case ${generic_target} in</tt>' you need to add a line like

<syntaxhighlight lang="bash">
  i386-*-myos*)    fmt=elf ;;
</syntaxhighlight>

You should use '<tt>i386</tt>' in this line even if you are targeting x86_64.  This is the only file where you should do it.  It is basically because the variable 'generic_target' is not your canonical target name, but rather a variable generated further up in the configure.tgt file, and it sets the first part to <tt>i386</tt> for any <tt>i[3-7]86</tt> or <tt>x86_64</tt>.

Note: this will use the 'generic' emulation. One side-effect is that gas will interpret slash ('/') as a comment, not as a division operator. This will break any code like "<tt>movl $(ADDRESS/PAGE_SIZE), %eax</tt>". Using "<tt>fmt=elf em=gnu ;;</tt>" or "<tt>fmt=elf em=linux ;;</tt>" will disable slash as a comment character.

=== ld/configure.tgt ===

This file tells the gnu linker what 'emulation' to use for each target.  An emulation is basically a combination of linker script and executable file format.  We are going to define our own emulation called <tt>elf_i386_myos</tt>.  We need to add an entry to the case statement here after '<tt>Please try to keep this table more or less in alphabetic order ... case "${targ}</tt>" in':

<syntaxhighlight lang="bash">
i[3-7]86-*-myos*)
			targ_emul=elf_i386_myos
			targ_extra_emuls=elf_i386
			targ64_extra_emuls="elf_x86_64_myos elf_x86_64"
			;;
x86_64-*-myos*)
			targ_emul=elf_x86_64_myos
			targ_extra_emuls="elf_i386_myos elf_x86_64 elf_i386"
			;;
</syntaxhighlight>

* '''elf_i386_myos''' is a 32-bit target for your OS.
* '''elf_x86_64_myos''' is a 64-bit target for your OS.
* '''elf_i386''' is a bare 32-bit target as you had with <tt>i686-elf</tt>.
* '''elf_x86_64''' is a bare 64-bit target for your OS as you had with <tt>x86_64-elf</tt>.

This setup provides you with a 32-bit toolchain that also can produce 64-bit executables, and a 64-bit toolchain that can also produce 64-bit executables. This comes in handy if you use objcopy, for instance. You can also add <tt>targ_extra_emuls</tt> entries to specify other targets ld should support. See the <tt>ld/configure.tgt</tt> file for examples.

=== ld/emulparams/elf_i386_myos.sh ===

Now we need to actually define our emulation. There is a generic file called <tt>ld/genscripts.sh</tt> which creates the required linker scripts for our target (you need more than one, depending on shared object usage and the like: I have 13 for a single target). It uses a linker script template (from the ld/scripttempl directory) to do this, and it creates the actual emulation C file from an emulation template (from the ld/emultempl directory). These templates are customised by running a script in the ld/emulparams directory which sets various variables. You are welcome to define your own emulation and linker templates, but I find the ELF ones adequate, given that they can be customised by simply adding a file to the emulparams directory. This is what we are going to do now. The content of the file could be something like:

<syntaxhighlight lang="bash">
source_sh ${srcdir}/emulparams/elf_i386.sh
TEXT_START_ADDR=0x08000000
</syntaxhighlight>

This script is included by <tt>ld/genscripts.sh</tt> to customize its behavior through shell variables. We include the base <tt>elf_i386.sh</tt> script as it sets reasonable defaults. Finally, we override the variables whose defaults we disagree with.

There are a large number of variables that can be set here to customize your toolchain. Read the documentation and look at existing emulations for further information. These are some of the variables that can be set:

* '''GENERATE_SHLIB_SCRIPT'''=''yes|no'' Whether to generate a linker script for shared libraries. We enable this as you might want it later.
* '''GENERATE_PIE_SCRIPT'''=''yes|no'' Whether to generate a linker script for position independent executables. We enable this as you might want it later.
* '''SCRIPT_NAME'''=''name'' Controls which ld/scripttempl/''name''.sc script generates our linker scripts.
* '''TEMPLATE_NAME'''=''name'' Controls which ld/emultempl/''name''.em script generates our bfd emulation C implementation.
* '''OUTPUT_FORMAT'''=''name'' The name of the BFD output target we use.
* '''TEXT_START_ADDR'''=0x''value'' Controls where the executable begins in memory.

You can read the base <tt>elf_i386.sh</tt> script for the defaults of these variables, you can then decide for yourself if you wish to override them for your operating system.

=== ld/emulparams/elf_x86_64_myos.sh ===

This file is just like the above <tt>ld/emulparams/elf_i386_myos.sh</tt> but for x86_64.

<syntaxhighlight lang="bash">
source_sh ${srcdir}/emulparams/elf_x86_64.sh
</syntaxhighlight>

=== ld/Makefile.am ===

We now just need to tell make how to produce the emulation C file for our specific emulation. Putting the '<tt>targ_emul=elf_i386_myos</tt>' line into <tt>ld/configure.tgt</tt> above implies that your host linker will try to link your target ld executable with an object file called <tt>eelf_i386_myos.o</tt>. There is a default rule to generate this from <tt>eelf_i386_myos.c</tt>, so we just need to tell it how to make this <tt>eelf_i386_myos.c</tt> file.  As stated above, we let the genscripts.sh file do the hard work. You need to add <tt>eelf_i386_myos.c</tt> to the <tt>ALL_EMULATION_SOURCES</tt> list; you also need to add <tt>eelf_x86_64_myos.c</tt> to the <tt>ALL_64_EMULATION_SOURCES</tt> list if applicable.

'''Note''': You ''must'' run <tt>automake</tt> in the <tt>ld</tt> directory after you modify <tt>Makefile.am</tt> to regenerate <tt>Makefile.in</tt>.

== Modifying GCC ==

=== config.sub ===

Similar modification to config.sub in Binutils.

=== gcc/config.gcc ===

This file defines what needs to be built for each particular target and what to include in the final executable.  There are two main sections: one which defines generic options for your operating system, and those which define options specific to your operating system on each individual machine type. 

For the first part, find the '<tt>case ${target} in</tt>' line just after '<tt># Common parts for widely ported systems</tt>' (around line 688) and add something like:

<syntaxhighlight lang="bash">
*-*-myos*)
  extra_options="$extra_options gnu-user.opt"
  gas=yes
  gnu_ld=yes
  default_use_cxa_atexit=yes
  use_gcc_stdint=provide
  ;;
</syntaxhighlight>

* '''extra_options="$extra_options gnu-user.opt"''' our operating system has options such as -pthread
* '''gas=yes''' our operating system by default uses the GNU assembler
* '''gnu_ld=yes''' out operating system by default uses the GNU linker
* '''default_use_cxa_atexit=yes''' We will provide ''__cxa_atexit'' (You will need to provide this in your standard library)
* '''use_gcc_stdint=provide'''  This instructs gcc to provide you with a ''stdint.h'' appropiate for your target.  Change <tt>provide</tt> to <tt>wrap</tt> if you have your own ''stdint.h'', to make GCC wrap yours. Alternatively use <tt>none</tt> to make gcc not install such a header and let libc supply it.

The second section we need to add to is the architecture-specific one. Find the '<tt>case ${target} in</tt>' line just before '<tt>tm_file="${tm_file} elfos.h newlib-stdint.h"</tt>' (around line 1094) and add something like:

<syntaxhighlight lang="bash">
i[34567]86-*-myos*)
    tm_file="${tm_file} i386/unix.h i386/att.h elfos.h glibc-stdint.h i386/i386elf.h myos.h gnu-user.h"
    ;;
x86_64-*-myos*)
    tm_file="${tm_file} i386/unix.h i386/att.h elfos.h glibc-stdint.h i386/i386elf.h i386/x86-64.h myos.h gnu-user.h"
    ;;
</syntaxhighlight>

This defines which target configuration header files gets used. You can make <tt>i386/myos32.h</tt> and <tt>i386/myos64.h</tt> files if desired.

=== gcc/config/myos.h ===

This header allows you to customize your toolchain using preprocessor macros. The relevant parts of GCC will include this header (as controlled by <tt>gcc/config.gcc</tt>) and modify the behavior according to your customizations. 

You can explore <tt>gcc/defaults.h</tt> for a full list of things you can modify, and more importantly, the assumptions GCC will make about various aspects of your target. For instance, if <tt>PID_TYPE</tt> is not defined in <tt>myos.h</tt>, then GCC will default it to <tt>int</tt>, which can be problematic if that's not what your <tt>pid_t</tt> is defined as.

<syntaxhighlight lang="C">
/* Useful if you wish to make target-specific GCC changes. */
#undef TARGET_MYOS
#define TARGET_MYOS 1

/* Default arguments you want when running your
   i686-myos-gcc/x86_64-myos-gcc toolchain */
#undef LIB_SPEC
#define LIB_SPEC "-lc" /* link against C standard library */

/* Files that are linked before user code.
   The %s tells GCC to look for these files in the library directory. */
#undef STARTFILE_SPEC
#define STARTFILE_SPEC "crt0.o%s crti.o%s crtbegin.o%s"

/* Files that are linked after user code. */
#undef ENDFILE_SPEC
#define ENDFILE_SPEC "crtend.o%s crtn.o%s"

/* Additional predefined macros. */
#undef TARGET_OS_CPP_BUILTINS
#define TARGET_OS_CPP_BUILTINS()      \
  do {                                \
    builtin_define ("__myos__");      \
    builtin_define ("__unix__");      \
    builtin_assert ("system=myos");   \
    builtin_assert ("system=unix");   \
    builtin_assert ("system=posix");   \
  } while(0);
</syntaxhighlight>

=== libstdc++-v3/crossconfig.m4 ===

This file describes how the libstdc++ configure file will examine your operating system and adjust the provided features of libstdc++ accordingly. Add a case similar to

<syntaxhighlight lang="autoconf">
  *-myos*)
    GLIBCXX_CHECK_COMPILER_FEATURES
    GLIBCXX_CHECK_LINKER_FEATURES
    GLIBCXX_CHECK_MATH_SUPPORT
    GLIBCXX_CHECK_STDLIB_SUPPORT
    ;;
</syntaxhighlight>

'''Note''': You need to run <tt>autoconf</tt> in the <tt>libstdc++-v3</tt> directory.

=== libgcc/config.host ===

Find the '<tt>case ${host} in</tt>' just prior to '<tt>extra_parts="$extra_parts crtbegin.o crtend.o crti.o crtn.o"</tt>' (around line 368) and add the cases:

<syntaxhighlight lang="bash">
i[34567]86-*-myos*)
	extra_parts="$extra_parts crti.o crtbegin.o crtend.o crtn.o"
	tmake_file="$tmake_file i386/t-crtstuff t-crtstuff-pic t-libgcc-pic"
	;;
x86_64-*-myos*)
	extra_parts="$extra_parts crti.o crtbegin.o crtend.o crtn.o"
	tmake_file="$tmake_file i386/t-crtstuff t-crtstuff-pic t-libgcc-pic"
	;;
</syntaxhighlight>

=== fixincludes/mkfixinc.sh ===

You should disable fixincludes for your operating system. Find the case statement and add a pattern for your operating system. For instance:

<syntaxhighlight lang="bash">
# Check for special fix rules for particular targets
case $machine in
    *-myos* | \
    *-*-myos* | \
    i?86-*-cygwin* | \
	# (... snip ...)
    powerpcle-*-eabi* )
	#  IF there is no include fixing,
	#  THEN create a no-op fixer and exit
	(echo "#! /bin/sh" ; echo "exit 0" ) > ${target}
        ;;
</syntaxhighlight>

A number of operating systems (especially older and obscure ones) provide troublesome systems headers that fail to strictly comply with various standards. The GCC developers consider it their job to fix these headers. GCC will look into your system root, apply a bunch of patterns to detect headers it doesn't like, then it copies that header into a private GCC system directory (that overrides your standard system directory) and attempts to fix the header. Sometimes fixincludes even break working headers (some people refer to it as breakincludes).

This is rather inconvenient as your libc will likely happen to trigger these patterns (and false positives often happens). Any time you change your system headers, you have to rebuild your compiler so the fixed versions get updated. The first time you encounter this, it will show up as a system header that does nothing different even though you edit it.

This addition to the <tt>mkfixinc.sh</tt> file forcefully disables fixincludes for your operating system. It's your job to provide working system headers, not the compiler developers'.

== Further Customization ==

'''TODO''': Document more various tips and tricks for further customization of OS specific toolchains.

=== Changing the Default Include Directory ===

If you wish to change the default include directory from <tt>/usr/include</tt>, you can override the <tt>native_system_header_dir</tt> variable in <tt>gcc/config.gcc</tt> in the case for your OS.

=== Changing the Default Library Directory ===

If you wish to change the default library directory from <tt>/usr/lib</tt>, you can change it to <tt>/lib</tt> by adding the following block of code to the case just below the declaration of <tt>NATIVE_LIB_DIRS</tt> in <tt>binutils/ld/configure.tgt</tt> (around line 1056).

<syntaxhighlight lang="c">
*-*-myos*)
  NATIVE_LIB_DIRS='/lib /local/lib'
  ;;
</syntaxhighlight>

=== Start Files Directory ===

You can modify which directory GCC looks for the crt0.o, crti.o and crtn.o in. The path to that directory is stored in <tt>STANDARD_STARTFILE_PREFIX</tt>. For instance, if you change the library directory to <tt>/lib</tt> in Binutils and want GCC to match, you can add the following to <tt>gcc/config/myos.h</tt>:

<syntaxhighlight lang="C">
#undef STANDARD_STARTFILE_PREFIX
#define STANDARD_STARTFILE_PREFIX "/lib/"
</syntaxhighlight>

Note that the trailing slash is important as the raw crt*.o names are appended without first adding a slash.


=== Dynamic linking ===
If you want to support dynamic linking with your toolchain, you need to...
* Pass the <tt>"--enable-shared"</tt> flag to the <tt>./configure</tt> scripts of both binutils and gcc
* Set <tt>LINK_SPEC</tt> in your "gcc/config/myos.h" so the gcc driver will pass the correct flags to the linker:

<syntaxhighlight lang="C">
#undef LINK_SPEC
#define LINK_SPEC "%{shared:-shared} %{static:-static} %{!shared: %{!static: %{rdynamic:-export-dynamic}}}"
</syntaxhighlight>

=== Linker Options ===

You can modify the arguments passed to the linker using <tt>LINK_SPEC</tt>. This can be used to force 4KB alignment of sections on 64 bit systems, as ld defaults to 2MB alignment.

<syntaxhighlight lang="C">
/* Tell ld to force 4KB pages*/
#undef LINK_SPEC
#define LINK_SPEC "-z max-page-size=4096"
</syntaxhighlight>

== Selecting a C Library ==

{{Main|C Library}}

At this point, you have to decide which [[C Library]] to use. You have options:

* [[Creating a C_Library|Create your own C library]].
* Pick an existing [[C Library]] such as [[Porting Newlib|Newlib]].

== Building ==

{{Main|Hosted GCC Cross-Compiler}}

Your OS specific toolchain is built differently from the introductory <tt>i686-elf</tt> toolchain as it has a user-space and standard library. In particular, you need to ensure your libc meets the minimum requirements for libgcc. You need to install the standard library headers into your [[Meaty_Skeleton#System_Root|System Root]] before building the cross-compiler. You need to tell the cross-binutils and cross-gcc where the system root is via the configure option <tt>--with-sysroot=/path/to/sysroot</tt>. You can then build your libc with your cross-compiler and then finally libstdc++ if desired.

== Conclusion ==

You now have a <tt>i686-myos</tt> toolchain that can be used instead of your old <tt>i686-elf</tt> toolchain. Your new toolchain is effectively just a renamed <tt>i686-elf</tt> with customizations. You should switch all your operating system build scripts to use this new compiler, even the kernel and libk, as your new compiler is capable of providing a freestanding environment.

You will certainly wish to package up your custom toolchain (and be able to create a diff between the upstream version and your custom version for others to audit). Contributors should be able to download tarballs of your myos-binutils and myos-gcc packages, so they can build themselves your custom toolchain. 

== Common errors ==

=== Whitespaces ===

Some files need tabs, some files need spaces and some files accept happily any mixture. Use an editor that can display special chars such as tabs and spaces, to be sure you use the right form. Whitespace errors may result in 'make' reporting missing separators. Some editors will replace a tab with four spaces, which will also cause invalid separator issues.

=== Autoconf ===

There are several steps that conclude in running '<tt>autoconf</tt>' or '<tt>automake</tt>', '<tt>autoreconf</tt>', be sure you did not miss them. The order of autoconf/-reconf calls in a package is important. These errors may result in missing subdirectories of the build-* directory and/or 'make' reporting missing targets.

== See Also ==

=== Articles ===

* [[GCC]]

[[Category:Tools]]
[[Category:Tutorials]]
````

<a id="w002165"></a>

## W002165 — User:Jnc100

Bron-ID: W002165
Namespace: User (2)
Revisie: 4550 | Datum: 2007-09-10T22:16:08Z
Bron: https://wiki.osdev.org/User:Jnc100
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJnc100&oldid=4550

````text
{{Special:Prefixindex/User:Jnc100/}}
````

<a id="w002166"></a>

## W002166 — Talk:Specifications

Bron-ID: W002166
Namespace: Talk (1)
Revisie: 4553 | Datum: 2007-09-11T05:33:04Z
Bron: https://wiki.osdev.org/Talk:Specifications
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ASpecifications&oldid=4553

````text
Did anyone happen to save copies of the mouse info? 3 out of the 5 links are dead. [[User:Bewing|Bewing]] 00:33, 11 September 2007 (CDT)
````

<a id="w002167"></a>

## W002167 — Talk:OS Specific Toolchain

Bron-ID: W002167
Namespace: Talk (1)
Revisie: 27466 | Datum: 2022-10-11T19:00:38Z
Bron: https://wiki.osdev.org/Talk:OS_Specific_Toolchain
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AOS_Specific_Toolchain&oldid=27466

````text
'''Proposal:''' I think we should simplify this page and remove the binutils part about the emulparams. It is not really necessary to create custom emulparams since those existing for the target should be sufficient (elf_i386/elf_x86_64 in this case). This makes the whole setup simpler since automake'ing isn't required anymore either. What do you think? --[[User:Max|Max]] 14:00, 11 October 2022 (CDT)


----


'''Excellent''' work! As for how to continue with this... I was originally intending what is the GCC Cross-Compiler article today to become a multi-part tutorial, starting with the cross-compiler setup and ending with a self-supporting OS. You can see the stubs for the later steps in the article.
Of course I would prefer that tutorial to include PDCLib instead of newlib, but newlib is available today. Maybe it's time to expand the cross-compiler tut. into the multi-part thing it was originally meant to be?
Opinions? [[User:Solar|Solar]] 06:21, 13 September 2007 (CDT)


:I'm not familiar enough with the inner workings of gcc and binutils to say something meaningful about it, but I wonder to what extent a stage 2/3 cross-compiler would be buildable with a plain unmodified i586-elf target. If it is, this tutorial is just a sidestep for the geeks who see the uses of a specifically named toolchain (or just want gcc to be named after them) and the cross-compiler tutorial is mostly independent. 
:On the other hand, the cross-compiler tutorial is for most people to just get a stage 1 compiler which is suitable for kernel development. It could be wise to use the crosscompiler page for just that, and build a stage 2/3 and stage 4 crosscompiler in separate articles (something like, [[User:Jnc100/OS Specific Toolchain|this article]] and [[Porting GCC]]).
:I posted a PM to the author in the meantime in which I said that this article is very well suited for inclusion in the main namespace. Independent of what we are to do with it, I don't think we should withhold other uses from finding and using this article. At worst somebody discover errors, which isn't even too bad since we can fix that later.
:@Solar: I tried once building a stage 2 gcc with PDCLib but it failed with conflicting types for size_t so I guess newlib is for now the way to go. 
:- [[User:Combuster|Combuster]] 10:46, 13 September 2007 (CDT)


:: Okay, its moved.  GCC and binutils are quite interlinked so should remain in the same section.  I don't, however, see any problem with splitting the newlib bit into a separate section (although then the overlap with [[Porting Newlib]] is quite apparent), so we could just have links at the bottom of the GCC/binutils bit to the newlib page and also to a (future) PDCLib page.  Unfortunately, I don't have much experience with PDCLib.  Originally, I tried with glibc (glutton for punishment) but eventually gave up trying to understand their definition of 'unix' and 'posix' (they implement some aspects of the Single Unix Specification, and you implement the rest.  Unfortunately, they don't explain anywhere what you are supposed to implement, and adding each function tended to require a complete rebuild, including reconfiguring every directory).  I chose newlib late one night because pcmattman had already demonstrated [[Porting Newlib|proof-of-principle]].
:: I'm all for expanding the cross compiler tutorial, but have concerns that we might make it too complex beyond the level of instructing new visitors on the methods for building a cross compiler.  Basically, I don't want to fill up the forum with lots of simple questions from people not following the instructions precisely.  It should be made extremely clear that this article is not intended for beginners.
:: As I've now moved this article to the main namespace, I'll remove the 'Creating a new binutils target' which was my forerunner to this article and is really just a less-well written and designed version of the first section.
:: Many thanks for your kind comments. [[User:Jnc100|John]] 12:39, 13 September 2007 (CDT)

:: Also, I'm adding this to the tutorials category, but think it might be wise to maybe create a subcategory on tutorials for creating user-space tools or something, because otherwise it might become one of the first ports of call for people looking for an 'OS development tutorial'.  I've made a comment to that effect on the [[Category talk:Tutorials|Tutorials talk page]]. [[User:Jnc100|John]] 12:44, 13 September 2007 (CDT)


::: If needs be, you can overwrite my version of the [[Porting Newlib]] article with your description, which is more in-depth than mine. Specifically, it covers adding a new system as your newlib target, which I couldn't find out how to do properly. I'll be referring to this tutorial later when I get around to porting the toolchain to my OS - it's really well done.
::: - [[User:Pcmattman|Pcmattman]] 23:30, 14 September 2007 (CDT)


: I've spent a lot of time reading through the GCC source and various docs and think I have a very good understanding of how to 'properly' specify target specific behavior when adding a new OS target to both binutils and GCC. For now, I'm writing up some documentation on this offline, and will eventually post it under my user namespace. I'll post here (and on the forum) when I have that done. If this really is the proper place for that kind of document (there doesn't seem to be a consensus here), I'll happily post the 'finished' doc here, but right now I think that [[Porting GCC]] is a more likely place for it.
: --[[User:Quok|Quok]] 06:08, 26 August 2008 (UTC)



I am working on some modifications to this guide that will allow initializers (_init and _fini) to work. Also I will be working on some stuff that will allow these initializers to work in the kernel.
--[[User:uglyoldbob|uglyoldbob]] 13 Oct 2008

== Grid of Successful Configurations ==
As suggested on [http://www.osdev.org/phpBB2/viewtopic.php?p=111217#111217 the forum], I am submitting a successful configuration for following this tutorial. Perhaps once there are enough of these, we can sort out how to format the information. For my part:
 cygwin-based:
 i586-pc-caracal - based strictly on the tutorial using binutils 2.18 and GCC (with g++) 4.2.2.
 x86_64-pc-caracal - same version of GCC and binutils using x86_64 as a template.
                   - Extra emulation target added: i386 elf and coff.

Thanks for the excellent tutorial and help on the forum!

The following quokos targets now exist as patches in the QuokOS svn repo:
  i[3-7]86-pc-quokos - Loosely based on this tutorial with other additions.
    emulations: elf32_i386, efi_app_ia32
    64-bit emulations: elf64_x86_64
  x86_64-pc-quokos - Based on the above target, with the same additions plus x86_64 specific bits.
    emulations: elf64_x86_64, efi_app_x86_64
    32-bit emulations: elf32_i386, efi_app_ia32
The quokos target patchset was initially based on GCC 4.2.4 and Binutils 2.18, but has been applied
to different versions and configurations of GCC from 4.0+ and Binutils 2.16, 2.17, and 2.18. For the
GCC 4.2.4 / Binutils 2.18 combination specifically, the build succeeds on openSUSE 10.3, 11.0, Debian Etch,
Cygwin, OSX 10.5, and OpenBSD 4.3.  If anyone wants the details of what else I changed, or wants the
patches themselves for some reason, just PM me. --[[User:Quok|Quok]] 06:08, 26 August 2008 (UTC)

== Implementation notes ==
This tutorial has worked for me with GCC 4.5.0 and Newlib 1.18.0. I came across two errors when compiling Newlib. The first was exactly the same as in [forum:17777 this thread], but this was because there were compilation errors in my system call stubs; my best guess is that Make would try to compile them, fail, abort the entire compile process, then continue onwards assuming that it had succeeded. The second error was when I ran 'make install'. /usr/bin/install couldn't stat newlib/libc/sys/myos/netinet/*.h. For lack of a better way to disable networking (I presume) functionality, I simply created the folder structure as indicated and created an empty file, Net.h. This could probably be done better, but for a quick and dirty hack it seems to work. --[[User:Computafreak|Computafreak]] 09:41, 30 August 2010 (UTC)

I tried this with bleeding edge binutils-2.22, gcc-4.6.2 and newlib-1.20.0. I had a few issues, the most important one is that gettimeofday() syscall is wrong. Removing the row made it compile but should probably be changed. Also I had to add --without-headers and --with-newlib when running configure for gcc --[[User:Jezze|Jezze]] 22:27, 6 January 2012 (CST)

== Binutils not building GNU gold linker ==

Even when adding <tt>--enable-gold --enable-lto --enable-plugins</tt> to the Binutils configure script, the gold linker is not being built. It is built for the target <tt>i686-pc-elf</tt>, but not <tt>i686-pc-myos</tt>. I'm not sure whether you have to enable gold for the ''myos'' target and where. It seems that <tt>$SRC/gold/configure.tgt</tt> does not require any changes. I just wanted to mention this. Using binutils-2.21. --[[User:Walling|Walling]] 21:57, 7 April 2011 (UTC)

:I found the answer. You have to enable the gold linker for your target. It is easily done in the main configure.ac file. Look for <tt>Handle --enable-gold, --enable-ld</tt> and find the line <tt># Check for ELF target.</tt> Now add your target in the <tt>case "${target}"</tt>. I added <tt>*-*-myos*</tt> after <tt>*-*-irix6*</tt>, because there was some space left. Then run <tt>autoconf</tt> in the same directory as the configure.ac file. It will generate a new configure script. Now you're able to configure (in a build directory) and make. --[[User:Walling|Walling]] 13:24, 8 April 2011 (UTC)

== Newlib sys/myos Makefile.am problem ==

It appears that in newer versions of Newlib (I'm trying to use 2.1.0), if you create the Makefile.am for your system as suggested below:

  if MAY_SUPPLY_SYSCALLS
    extra_objs = syscalls.o
  else
    extra_objs = 
  endif

than, the extra_objs assignment is added after the rest of the assignments in the actual Makefile. So, for example, in the Makefile (after configuration) it would look something like this:

  lib_a_SOURCES =
  lib_a_LIBADD = $(extra_objs)
  EXTRA_lib_a_SOURCES = syscalls.c crt0.S
  lib_a_DEPENDENCIES = $(extra_objs)
  lib_a_CCASFLAGS = $(AM_CCASFLAGS)
  lib_a_CFLAGS = $(AM_CFLAGS)
  ... # A lot of other stuff, approximately 300 lines
    extra_objs = syscalls.o
  # extra_objs = 

This makes lib_a_LIBADD empty, and therefore never compiles your syscalls. The solution is to remove the conditional and make it simply:

  extra_objs = syscalls.o

It was a rather annoying bug to track down. I'm not sure of the ramifications of ignoring the MAY_SUPPLY_SYSCALLS macro, but I am currently ignoring it and newlib is compiling happily. I assume this is happening because the configure script/automake makes two passes. First evaluating assignments and adding them all to the script, then second evaluating all conditionals and adding remaining assignments to the script. So on pass one, all the variables are dumped into the Makefile, then the if block is tested and extra_objs is placed inside the Makefile after all the other assignments (which in this case is a problem). --[[User:Caleb1994|Caleb1994]] 17:11, 4 September 2014 (CDT)
````

<a id="w002168"></a>

## W002168 — User:Jnc100/OS Specific Toolchain

Bron-ID: W002168
Namespace: User (2)
Revisie: 4565 | Datum: 2007-09-13T17:21:59Z
Bron: https://wiki.osdev.org/User:Jnc100/OS_Specific_Toolchain
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJnc100%2FOS_Specific_Toolchain&oldid=4565
Doorverwijzing: OS Specific Toolchain

````text
#REDIRECT [[OS Specific Toolchain]]
````

<a id="w002169"></a>

## W002169 — User talk:Jnc100/OS Specific Toolchain

Bron-ID: W002169
Namespace: User talk (3)
Revisie: 4567 | Datum: 2007-09-13T17:21:59Z
Bron: https://wiki.osdev.org/User_talk:Jnc100/OS_Specific_Toolchain
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AJnc100%2FOS_Specific_Toolchain&oldid=4567
Doorverwijzing: Talk:OS Specific Toolchain

````text
#REDIRECT [[Talk:OS Specific Toolchain]]
````

<a id="w002170"></a>

## W002170 — Plug-and-Play

Bron-ID: W002170
Namespace: Main (0)
Revisie: 30134 | Datum: 2026-03-05T02:48:31Z
Bron: https://wiki.osdev.org/Plug-and-Play
Revisie-URL: https://wiki.osdev.org/index.php?title=Plug-and-Play&oldid=30134

````text
{{Historical}}
{{Stub}}

'''Plug-and-Play''' (PnP) is a set of hardware and software specifications designed to reduce or eliminate user configuration for PC hardware by facilitating the discovery of devices and matching drivers as well as resource management (DMA, IRQ, etc.). As of 2001, [[ACPI]] has assumed the role of PnP BIOS and [[APM]], and [[PCI]] has inherent plug-and-play functionality through the configuration space, thus making PnP obsolete.


== Goal of Plug-and-Play ==

In the early days of the IBM PC platform, [[IRQ|IRQs]], [[DMA]] channels, ports, and memory addresses had to be manually configured by flipping DIP switches and jumping pins on the main board. Conflicts always caused errors because the ISA bus did not have any way of arbitrating shared resources (e.g. there was no way to know where an IRQ came from). Drivers would have to be set up manually. PnP ISA was developed for the standard 16-bit PC bus as a hardware extension. It did not allow for IRQ sharing but did allow resources to be reallocated through a configuration space to avoid conflicts.

To assist the OS in hardware configuration, the PnP BIOS specification was published. It provides a 16-bit [[Protected Mode|protected]] and [[Real Mode|real mode]] interface with various functions like obtaining extended system configuration data and the number of [[ISA]] PnP cards, amongst other features.

== PnP device specifications ==
There are PnP specifications for the following:
* Serial port [https://web.archive.org/web/20160322040318/http://www.osdever.net/documents/PNP-ExternalSerial-v1.00.pdf]
* Parallel port [https://web.archive.org/web/20200926105816/http://www.osdever.net/documents/PNP-Parellel-v1.0b.pdf?the_id=55]
* ISA cards [https://web.archive.org/web/20160423133722/http://download.intel.com/support/motherboards/desktop/sb/pnpbiosspecificationv10a.pdf]
* BIOS [https://web.archive.org/web/20220317101935/https://www.intel.com/content/dam/support/us/en/documents/motherboards/desktop/sb/pnpbiosspecificationv10a.pdf]
* SCSI [https://web.archive.org/web/20180116205211/http://ftpmirror.your.org/pub/misc/ftp.microsoft.com/developr/drg/Plug-and-Play/Pnpspecs/PNPSCSI.EXE]

== Resources ==
* [https://web.archive.org/web/20180116205211/http://ftpmirror.your.org/pub/misc/ftp.microsoft.com/developr/drg/Plug-and-Play/Pnpspecs/ Official Plug-and-Play specifications from Microsoft] - Note: the files are self extracting Zip files in a [[MZ|MS-DOS EXE]] format.
* [https://web.archive.org/web/20030207142304/http://www.microsoft.com:80/hwdev/tech/PnP/default.asp Plug and Play Technology]
* [https://web.archive.org/web/20080829130750/http://members.datafast.net.au/dft0802 Craig's PCI & PnP Stuff Pages]
* [https://web.archive.org/web/20160418224610/https://linux-sxs.org/programming/interfac.html ISAPNP-Programming-Interface]

== See Also ==
=== Articles ===
* [[PNP Calls In Protected Mode]]
* [[ACPI]], the standard superseding PnP

[[Category:Hardware Detection]]
[[Category:X86]]
````

<a id="w002171"></a>

## W002171 — Talk:Plug-and-Play

Bron-ID: W002171
Namespace: Talk (1)
Revisie: 4612 | Datum: 2007-09-16T21:24:48Z
Bron: https://wiki.osdev.org/Talk:Plug-and-Play
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APlug-and-Play&oldid=4612

````text
I moved the contents of "Where Can I Find More Info On PNP" to the Plug-and-Play article under the headline "Resources". This is to retain consistancy throughout the wiki.--[[User:MessiahAndrw|MessiahAndrw]] 16:24, 16 September 2007 (CDT)
````

<a id="w002172"></a>

## W002172 — Talk:Porting Newlib

Bron-ID: W002172
Namespace: Talk (1)
Revisie: 25022 | Datum: 2020-07-24T20:14:28Z
Bron: https://wiki.osdev.org/Talk:Porting_Newlib
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APorting_Newlib&oldid=25022

````text
==Neutral==
Really? 'Neutral'? For something to be neutral, I was under the impression that there had to be a disagreement about it. How, by any meaning of the word, is using 'Genesis' not appropriate as a synonym for 'Beginning'? Moreover, in [[DMA]], using Genesis makes the article a little more together due to it's style. I should say it's perfectly acceptable. Such edits seem to cause quite the opposite of the intent. --Alboin

How is using a religious terminology at all related to what we do on this site? I made sure that my choice of words were clear and descriptive. "Genesis" might not be very clear to non-English speaking Wiki browsers. Change it back if you disagree. I personally think the [[DMA]] article and any others should not contain such "speculative" information, Audits anyone? ;-) --[[User:Brynet-Inc|Brynet-Inc]] 18:15, 16 September 2007 (CDT)

The [[DMA]] article is written like it is a bible story, so it should be Genesis. It fits, and it works. Non-English speakers should have some idea of what Genesis means (the bible is the most read book, after all). I don't see any offence towards non-Christians, either. It's a humourous reference, not like it's being preached. On the other hand, in Porting Newlib, it should be Preparation. It's not Genesis, it doesn't fit, and it's not at the beginning. [[User:Yayyak|Yayyak]] 18:33, 16 September 2007 (CDT)

Preparation does not fit the chapter either, so I took the liberty to change it to "Compiling" (basically, the preparation is done in the support calls section, all you are doing within that chapter is configuring and compiling newlib). - [[User:Combuster|Combuster]] 15:09, 17 September 2007 (CDT)


should we have like a see also thing? cause I found http://neptune.billgatliff.com/newlib.html which has some nice info about porting newlib to a very minimal OS(and how to cope with such things as lack of filesystem) --[[User:Earlz|Earlz]] 15:17, 1 June 2009 (UTC)

== Building newlib with ARM cross-compiler on Fedora 10 ==

To get this to work, I had to apply the patch at http://thread.gmane.org/gmane.comp.lib.newlib/5612, see the attachment to the second message, from Nick Clifton. On Fedora, I couldn't easily get patch to figure out what to do with it, possibly because of my inexperience using patch or my general laziness. It's a simple fix and easy enough to apply by hand. Then newlib built for me.

The rest of the toolchain went together using a procedure modified slightly from the [[GCC Cross-Compiler]] page, and appears to have built and installed correctly.
 sudo mkdir /usr/local/arm-elf
 export PREFIX=/usr/local/arm-elf
 export TARGET=arm-elf
 cd build-binutils/
 ../binutils-2.20/configure --target=$TARGET --prefix=$PREFIX --disable-nls
 make all
 sudo make install
 cd ../build-gcc
 ls ..
 export PATH=$PATH:$PREFIX/bin
 ../gcc-4.3.2/configure --target=$TARGET --prefix=$PREFIX --disable-nls \
           --enable-languages=c,c++ --without-headers
 make all-gcc
 sudo make install-gcc
-- [[User:Willware|Willware]] 05:31, 1 November 2009 (UTC)

== Automake 1.14 ? ==

Since Cygnus style trees was removed from automake after release 1.12, recommend to use automake 1.14 to build newlib should fail ? At least, it fail on my distro (Fedora 23) with autoconf 2.68 (latest newlib 26/2/16) and automake 1.14.

== Hack at end ==

In the section where it shows making all those symbolic links, wouldn't it make more sense to build a first pass OS toolchain, the port Newlib, and then build a second pass toolchain? That's how I'm doing it. -- Nexos
````

<a id="w002173"></a>

## W002173 — User:MessiahAndrw

Bron-ID: W002173
Namespace: User (2)
Revisie: 25669 | Datum: 2021-02-13T02:48:19Z
Bron: https://wiki.osdev.org/User:MessiahAndrw
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMessiahAndrw&oldid=25669

````text

````

<a id="w002174"></a>

## W002174 — Talk:OSFAQ Foreword

Bron-ID: W002174
Namespace: Talk (1)
Revisie: 4836 | Datum: 2007-11-14T07:03:40Z
Bron: https://wiki.osdev.org/Talk:OSFAQ_Foreword
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AOSFAQ_Foreword&oldid=4836

````text
== Merge and deletion ==
Most of the contents contained here is also present in [[Getting Started]], be it that this page is far more aggressive on what the readers are to know. For that there are a few points to discuss:
* Should we need to spice up [[Getting Started]] to some extent as it is very mild opposed to the original edition of the requirements page (this one)? I think that we should make sure that the requirements listed are minimal, rather than just recommended.
* Should we add some requirement about knowledge regarding the toolchain? (IMO that is a must if you use something other than GCC / binutils / assembler)
* Any objections for deleting this article (after reaching consensus on the previous items)
- [[User:Combuster|Combuster]] 09:07, 17 September 2007 (CDT)

I think that's a good idea (TM). Spicing up Getting Started in some sections would certainly not be objectionable, especially the areas about required knowledge. That list for required knowledge (on the GS page) also needs to be extended a lot. IMHO, Linux and UNIX experience could be seperated as well. One of those items should definitely be knowledge about toolsets/utilities. I say go for it. [[User:Yayyak|Yayyak]] 17:59, 17 September 2007 (CDT)

As the original author, I have no objections to a merge / deletion. It was a "spice up" to [[Getting Started]] because of the high numbers of kids "oh yeee, /me going to write own op sys" in the forum back on MT. If some of that "spice" makes it into [[Getting Started]], the objective has been reached. [[User:Solar|Solar]] 04:10, 20 September 2007 (CDT)

I've thrown [[Getting Started|a few spices]] into the mix, your thoughts? - [[User:Combuster|Combuster]] 07:25, 20 September 2007 (CDT)
:So......Can this page be deleted? --Alboin

Yes, IMHO. --[[User:Yayyak|Yayyak]] 01:03, 14 November 2007 (CST)
````

<a id="w002175"></a>

## W002175 — User:Solar

Bron-ID: W002175
Namespace: User (2)
Revisie: 13888 | Datum: 2012-07-30T07:09:53Z
Bron: https://wiki.osdev.org/User:Solar
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASolar&oldid=13888

````text
Former user that left for other pastimes. Major contributions were [[GCC Cross-Compiler]] and [[Makefile]] as well as [http://pdclib.rootdirectory.de PDCLib].
````

<a id="w002176"></a>

## W002176 — RTL8139

Bron-ID: W002176
Namespace: Main (0)
Revisie: 30434 | Datum: 2026-03-29T22:49:11Z
Bron: https://wiki.osdev.org/RTL8139
Revisie-URL: https://wiki.osdev.org/index.php?title=RTL8139&oldid=30434

````text
{{In Progress}}

The RTL8139 Network Chip is used on many old and budget Ethernet Network Devices. It supports 10 and 100 MBit.  

While the original chip lacks the features of newer Gigabit devices, it is a commonly emulated device in virtualization environments, due to its simplicity (fewer IO operations means faster traps to the Hypervisor), and extensive OS support.  

It also makes a for a simple driver (and so is an excellent first choice for OS development hobbyists).

[[Image:rtl8139.JPG|right|frame|A PCI RTL8139]]

NB If you find your driver suddenly freezes and stops receiving interrupts and you're using [[KVM]]/[[QEMU]]. Try changing the option -machine kernel_irqchip=[on|off]

The PCI vendor ID is 0x10EC and the device ID is 0x8139.

== Overview ==

RTL8139 is configured via registers. The IO base address and the [[IRQ]] number for the device is taken from [[PCI]] configuration.


=== Registers ===

These are the registers used for using RTL8139.

{| class="wikitable"
! Offset (from IO base)
! Size
! Name
|-
| 0x00 || 6 || MAC0-5
|-
| 0x08 || 8 || MAR0-7
|-
| 0x30 || 4 || RBSTART
|-
| 0x37 || 1 || CMD
|-
|0x38
|2
|CAPR
|-
| 0x3C || 2 || IMR
|-
| 0x3E || 2 || ISR
|}

== Programming Guide ==

=== PCI Bus Mastering ===

First, you need to enable [[PCI]] Bus Mastering for this device. This allows the NIC to perform [[DMA]]. To do it, you can read the Command Register from the device's PCI Configuration Space, set bit 2 (bus mastering bit) and write the modified Command Register. Note this Command Register is that of the PCI Configuration Space and has nothing to do with the Command Register that will be evoked in the following sections: the latter is specific to the RTL8139, whereas every PCI device (not only NICs) have a PCI Configuration Space with a Command Register.

Some BIOS may enable Bus Mastering at startup, but some versions of QEMU don't. You should thus be careful about this step.

=== Turning on the RTL8139 === 

Send 0x00 to the CONFIG_1 register (0x52) to set the LWAKE + LWPTN to active high.
this should essentially *power on* the device.

<syntaxhighlight lang="c">
 outportb( ioaddr + 0x52, 0x0);
</syntaxhighlight>

==== Software Reset! ====

Next, we should do a software reset to clear the RX and TX buffers and set everything
back to defaults. Do this to eliminate the possibility of there still being garbage
left in the buffers or registers on power on. 

Sending 0x10 to the Command register (0x37) will send the RTL8139 into a software reset. 
Once that byte is sent, the RST bit must be checked to make sure that the chip has finished
the reset.  If the RST bit is high (1), then the reset is still in operation. 

NB: There is a minor bug in QEMU. If you check the command register before performing a
soft reset, you may find the RST bit is high (1). Just ignore it and carry on with the 
initialization procedure. 

<syntaxhighlight lang="c">
 outportb( ioaddr + 0x37, 0x10);
 while( (inb(ioaddr + 0x37) & 0x10) != 0) { }
</syntaxhighlight>

==== Init Receive buffer ====

For this part, we will send the chip a memory location to use as its receive buffer start 
location.  One way to do it, would be to define a buffer variable and send
that variables memory location to the RBSTART register (0x30). 

<syntaxhighlight lang="c">
 //ioaddr is obtained from PCI configuration
 outportd(ioaddr + 0x30, (uintptr_t)rx_buffer); // send uint32_t memory location to RBSTART (0x30)
</syntaxhighlight>

Note that 'rx_buffer' needs to be a pointer to a '''physical address'''. In this case a size of 8192 + 16 (8K + 16 bytes) is recommended, see below.

=== Set IMR + ISR ===

The Interrupt Mask Register (IMR) and Interrupt Service Register (ISR) are responsible
for firing up different IRQs. The IMR bits line up with the ISR bits to work in sync.
If an IMR bit is low, then the corresponding ISR bit with never fire an IRQ when the
time comes for it to happen. The IMR is located at 0x3C and the ISR is located at 0x3E.

To set the RTL8139 to accept only the Transmit OK (TOK) and Receive OK (ROK) interrupts,
we would have the TOK and ROK bits of the IMR high and leave the rest low.  That way when 
a TOK or ROK IRQ happens, it actually will go through and fire up an IRQ. 

<syntaxhighlight lang="c">
 outportw(ioaddr + 0x3C, 0x0005); // Sets the TOK and ROK bits high
</syntaxhighlight>

=== Configuring receive buffer (RCR) ===

Before hoping to see a packet coming to you, you should tell the RTL8139 to '''accept''' packets based on various rules. The configuration register is RCR.

You can enable different "matching" rules:
* '''AB - Accept Broadcast''': Accept broadcast packets sent to mac ff:ff:ff:ff:ff:ff
* '''AM - Accept Multicast''': Accept multicast packets.
* '''APM - Accept Physical Match''': Accept packets send to NIC's MAC address.
* '''AAP - Accept All Packets'''. Accept all packets (run in promiscuous mode).

Another bit, the WRAP bit, controls the handling of receive buffer wrap around.  

If WRAP is 0, the Rx buffer is treated as a traditional ring buffer: if a packet is being written near the end of the buffer and the RTL8139 knows you've already handled data before this (thanks to CAPR), the packet will continue at the beginning of the buffer.

If WRAP is 1, the remainder of the packet will be written contiguously (overflowing the actual receive buffer) so that it can be handled more efficiently. This means the buffer must be an additional 1500 bytes (to hold the largest potentially overflowing packet).

You can also tell the size of your RX buffer here, however if you use a 8k + 16 buffer as described before, writing zeroes is enough.   To use the WRAP=1 bit, an 8K buffer must in fact be 8k+16+1500 bytes.

<syntaxhighlight lang="c">
 outportl(ioaddr + 0x44, 0xf | (1 << 7)); // (1 << 7) is the WRAP bit, 0xf is AB+AM+APM+AAP
</syntaxhighlight>

=== Enable Receive and Transmitter === 

Now is the time to start up the RX and TX functions.  
This is quite an easy piece, and should (in my opinion) only be done after all of the 
configurations to the RTL8139's registers have been set to what is desired.  The
RE (Receiver Enabled) and the TE (Transmitter Enabled) bits are located in the Command
Register (0x37).  Starting up the RE and TE is pretty straight-forward, but lets go 
through it anyways.  

To enable the RTL8139 to accept and transmit packets, the RE and TE bits must go high.  
Once this is completed, then the card will start allowing packets in and/or out. 

<syntaxhighlight lang="c">
 outportb(ioaddr + 0x37, 0x0C); // Sets the RE and TE bits high 
</syntaxhighlight>

=== Transmitting Packets ===
The RTL8139 NIC uses a round robin style for transmitting packets. It has four transmit buffer (a.k.a. transmit start) registers, and four transmit status/command registers. The transmit start registers are each 32 bits long, and are in I/O offsets 0x20, 0x24, 0x28 and 0x2C. The transmit status/command registers are also each 32 bits long and are in I/O offsets 0x10, 0x14, 0x18 and 0x1C. Each pair of transmit start and status registers work together (i.e. registers 0x20 and 0x10 work together, 0x24 and 0x14 work together, etc.)

After performing a controller reset as mentioned above, the controller ignores all transmit registers except pair zero (i.e. register 0x20 for the transmit start and register 0x10 for the status/command.) After software transmits a packet using those registers, the round robin counter increments, to use pair one (i.e. register 0x24 for the transmit start and register 0x14 for the status/command.) This continues until pair number three, which is the last transmit register pair, and the counter then overflows and goes back to pair number zero. When the controller is using a specific transmit register pair, all other transmit registers are ignored, and cannot be used to transmit packets at all. Software is responsible for keeping track which pair is being used.

Each transmit start register contains the 32-bit physical address of the packet which is to be sent over the network.

Each transmit status/command register contains various bit fields that control the transmission of the packet. The table below only gives a brief about these bits. For the full format, take a look at the RTL8139 specification under "External links" below.

{| class="wikitable" width="70%" border="1"
|+Status/Command Register Format
! Bit Range
! Description
|-
|0-12
|'''Data size.''' This field contains the total size of the data to be sent across the network. The maximum value for this field is 1792, and larger numbers will cause an error.
|-
|13
|'''Own bit.''' When this bit is set to 1, the controller is idle and not performing any DMA transfers on this descriptor. The driver sets this bit to zero to tell the controller to start transmitting the packet. The bit then remains zero until the entire packet has been copied from RAM into the controller's internal memory via DMA. The default value after a reset is one. Setting this bit to zero also clears all other status bits in this register.
|-
|15
|'''Transmit OK.''' After the own bit has been set by the hardware, indicating the DMA transfer has completed, the hardware will start to transmit the packet across the actual network. This bit will be set to one after the network transmission has completed.
|}

=== Receiving Packets ===
The RTL8139 NIC uses a ring buffer to receive packages. The physical address of the buffer gets defined when you write to the RBSTART at setup.  When a packet comes into the NIC it first get stored int he RX FIFO and gets copied into the ring buffer once the early receive threshold is met. The status of a receiving packet is stored infront of it in a 16-Bit packet header
{| class="wikitable"
|+Packet Header Format
!Bit 
Range
!Name
!Description
|-
|0
|ROK
|'''Receive OK''': When set, indicates that a good packet is received.
|-
|1
|FAE
|'''Frame Alignment Error''': When set, indicates that a frame alignment error occurred on this received packet.
|-
|2
|CRC
|'''CRC Error''': When set, indicates that a CRC error occurred on the received packet
|-
|3
|LONG
|'''Long Packet''': Set to 1 indicates that the size of the received packetexceeds 4k bytes.
|-
|4
|RUNT
|'''Runt Packet Received''': Set to 1 indicates that the received packet length is smaller than 64 bytes ( i.e. media header + data + CRC < 64 bytes )
|-
|5
|ISE
|'''Invalid Symbol Error''': ''(100BASE-TX only)'' An invalid symbol was encountered during the reception of this packet if this bit set to 1.
|-
|6 - 12
| -
|'''Reserved'''
|-
|13
|BAR
|'''Broadcast Address Received''': Set to 1 indicates that a broadcast packet is received. BAR, MAR bit will not be set simultaneously.
|-
|14
|PAM
|'''Physical Address Matched''': Set to 1 indicates that the destination address of this packet matches the value written in ID registers.
|-
|15
|MAR
|'''Multicast Address Received''': Set to 1 indicates that a multicast packet is received.
|}
You know the start address of the current packet by reading the CAPR register, which contains the Current Address of Packet Read.

== ISR Handler ==
When you handle an interrupt, you ''have'' to write the bit corresponding to the interrupt to reset it. The datasheet says reading the register is enough to reset the buffer to zero and writing has no effect. ''This is not the case on QEMU'', and probably on some/most hardware too.

'''Note that it is important you write to this register ''before you read any packets from your buffers'', or the write to the register will have no effect, and any other packets than the first will not be delivered to your ISR.'''

For example, this is tested and works on QEMU:

<syntaxhighlight lang="c">
void rtl8139_handler(uint8_t isr, uint64_t error, uint64_t irq) {
	uint16_t status = inw(io_base + 0x3e);
	outw(io_base + 0x3E, 0x05);
	if(status & TOK) {
		// Sent
	}
	if (status & ROK) {
		// Received
		receive_packet();
	}
}
</syntaxhighlight>

== See Also ==
=== Articles ===
* [[RTL8169]] - The successor chipset

=== External Links ===
* [https://github.com/omarrx024/xos/tree/master/rtl8139 xOS RTL8139 Driver] (MIT license)
* [https://web.archive.org/web/20241228143504/https://people.freebsd.org/~wpaul/RealTek/spec-8139cp(150).pdf Datasheet for the RTL8139C]
* [http://www.cs.usfca.edu/~cruse/cs326f04/RTL8139D_DataSheet.pdf Datasheet for the RTL8139D], has more information
* [http://www.cs.usfca.edu/~cruse/cs326f04/RTL8139_ProgrammersGuide.pdf Programming guide for the RTL8139]
* [http://www.jbox.dk/sanos/source/sys/dev/rtl8139.c.html Example Driver implementation] (GPL Licensed)

[[Category:Network Hardware]]
[[Category:Standards]]
[[de:RTL8139]]
````

<a id="w002177"></a>

## W002177 — Talk:Resources/Merge Archive

Bron-ID: W002177
Namespace: Talk (1)
Revisie: 4690 | Datum: 2007-09-26T21:11:57Z
Bron: https://wiki.osdev.org/Talk:Resources/Merge_Archive
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AResources%2FMerge_Archive&oldid=4690

````text
== Archived discussions from OSRC ==
Why do we have this page? Wouldn't it be enough just to put a link in quicklinks or something? --Alboin
:Mostly legacy.  Once the two wikis are converted to the new format, their content will likely be merged together and reorganized.  [[User:Almafeta|Almafeta]] 18:18, 9 April 2007 (CDT)

===Merger===
This page was slated for merger. Here: [[Talk:Resources]]. Should it be moved back into PFR? (I'm thinking it doesn't really matter, but nevertheless. I thought I'd make note of it just in case someone was to revamp it.) --Alboin
:Sorry, didn't see that. I support the mergers. [[User:Yayyak|Yayyak]] 00:46, 31 August 2007 (CDT)


== Archived discussion from Quick Links ==
Can we remove this page? I moved all the information to [[Specifications]] on the main page. (Most of it, that is. There were some dead links, and I removed all of the forum links. (How does one find those on the new forum?)) --Alboin

I think a wider discussion is needed as there are several pages involved: [[Quick Links]] [[Specifications]] [[OSRC]] [[Resources]] and probably more. I think merging the lot into one [[Resources]] page with the appropriate subheaders would be good. Many off-site links are already present in various articles so having several pages dedicated to external links is not good for organisation. Having [[Specifications|specs]] and [[Resources|other resources]] in separate pages is arguable though. - [[User:Combuster|Combuster]] 17:08, 10 April 2007 (CDT)
:[[Resources]] is everything that is not a pure hardware specification, which go in [[Specifications]]. I think there's a clear distinction between the two. I am not sure about merging them... --Albion
I'm liking your idea about just a [[Resources]] page. Being, if someone's looking for information on Video hardware, they can just search for video and go to the links at the bottom of the page. Having a page '''just''' for [[Specifications]] is redundant. I think just a short and sweet [[Resources]] page would do, with non specific abstract links like osdev and OSRC. --Alboin
:Just noticed [[Links]]. --Alboin
:Also [[Kernels With Source]] --Alboin

I do not think Specifications redundant. Having a list with properly documentated hardware helps picking an OS developers' next goal. Since a large merge seems a good thing, I move this discussion to [[Talk:Resources]] where everything should end up. - [[User:Combuster|Combuster]] 12:33, 10 July 2007 (CDT)

== Archived discussion from Specifications ==
Did anyone happen to save copies of the mouse info? 3 out of the 5 links are dead. [[User:Bewing|Bewing]] 00:33, 11 September 2007 (CDT)
````

<a id="w002178"></a>

## W002178 — Talk:Resources/Archive

Bron-ID: W002178
Namespace: Talk (1)
Revisie: 4689 | Datum: 2007-09-26T21:00:49Z
Bron: https://wiki.osdev.org/Talk:Resources/Archive
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AResources%2FArchive&oldid=4689

````text

==Operating System Programming Tools==
I suggest we insure that everything in this section is in [[:Category:Tools]], and then remove the section. Anyone else have an opinion? -[[User:Jhawthorn|Jhawthorn]] 12:54, 26 June 2007 (CDT)

:Yes, I agree. The tool listing does not belong in the Resources page. - [[User:Combuster|Combuster]] 16:57, 26 June 2007 (CDT)

::It seems [[YASM]] ended up missing. Created the article as part of the double-check. Everything looks fine now - [[User:Combuster|Combuster]] 02:55, 27 June 2007 (CDT)

== Image tools ==
I'd like to see some more disk-image tools. The only one I know of that supports floppy disks is WinImage.
--[[User:MessiahAndrw|MessiahAndrw]] 23:44, 24 June 2007 (CDT)
:Have you checked [[:Category:Disk Image Utilities]]? -[[User:Jhawthorn|Jhawthorn]] 00:06, 25 June 2007 (CDT)

==Organization==
Things to think about for organisation:

* Add pascal/ada/basic/whatever compilers?
* split compilers into realmode, pmode and longmode?
* Other architectures

how about tagging each compiler with what architecture (subset) it supports

''- Combuster''

== Compiler Comparison ==

I think the compiler details like architecture supported should go in [[Tool_Comparison]]
''- Chase''
````

<a id="w002179"></a>

## W002179 — NMI

Bron-ID: W002179
Namespace: Main (0)
Revisie: 4697 | Datum: 2007-09-26T21:43:19Z
Bron: https://wiki.osdev.org/NMI
Revisie-URL: https://wiki.osdev.org/index.php?title=NMI&oldid=4697
Doorverwijzing: Non Maskable Interrupt

````text
#REDIRECT [[Non Maskable Interrupt]]
````

<a id="w002181"></a>

## W002181 — User:Ucosty

Bron-ID: W002181
Namespace: User (2)
Revisie: 4718 | Datum: 2007-10-02T17:24:07Z
Bron: https://wiki.osdev.org/User:Ucosty
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AUcosty&oldid=4718

````text
I'm currently working on gridOS. I should put the sources on a website or something at some stage...
````

<a id="w002182"></a>

## W002182 — Talk:How To Ask Questions

Bron-ID: W002182
Namespace: Talk (1)
Revisie: 4731 | Datum: 2007-10-16T22:41:36Z
Bron: https://wiki.osdev.org/Talk:How_To_Ask_Questions
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AHow_To_Ask_Questions&oldid=4731

````text
The OSID database link (http://oslib.codymays.net/index.php?page=4) appears to be dead for me. --[[User:MessiahAndrw|MessiahAndrw]] 09:13, 15 October 2007 (CDT)
:Same here. --Alboin
::It's been down for a few years. I think that the Projects page could be suggested as a replacement? [[User:Yayyak|Yayyak]] 01:16, 16 October 2007 (CDT)
:::Yeah. It's probably more up to date anyway. ;) --Alboin
````

<a id="w002184"></a>

## W002184 — User:Combuster/FAQ Rebuild

Bron-ID: W002184
Namespace: User (2)
Revisie: 5507 | Datum: 2008-06-01T02:37:20Z
Bron: https://wiki.osdev.org/User:Combuster/FAQ_Rebuild
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACombuster%2FFAQ_Rebuild&oldid=5507

````text
''This is an attempt at rebuild a faq-style index based off MT ideas. - [[User:Combuster|Combuster]] 04:45, 23 October 2007 (CDT)''

==Introduction==
'''Basic Information'''
* Foreword? <- this is IMPORTANT!
* Introduction
* Avoid beginner mistakes
* Getting started
* What order should I make things in?
* What is a kernel--shell--GUI--etc.

'''Environment'''
* How kernel, compiler, and C library work together
* Things you can not do with C (See also Interrupt Service Routines!)
* Can I use some language other than C?
* Doing a kernel in C++
* Doing a kernel in Pascal
* Sample inline functions
* Using the Bios or not using the Bios ? ...
* Using inline assembly

'''Booting and Setup'''

* Boot Sequence
* Diskless Booting
* The BabyStep tutorial
* GDT For Dummies
* Interrupts For Dummies
* Getting Keyboard Input

'''Testing and debuging'''

* How do I test (and avoid crashing my disk while doing so)
* How do I use a debugger with my OS
* Is it possible to use Unit Testing with my OS
* Help! My kernel crashed! (guru meditation in a nutshell)

== Design Considerations ==
'''[[Kernel]] Models'''
* [[Monolithic Kernel]]
* [[Microkernel]]
* [[Exokernel|Exokernels]] and their kind (including Nanokernels, Cache Kernels, Virtualizing Kernels, ...)
* [[Modular Kernel]]

'''Task Models'''

* Monotasking Systems
* Multitasking Systems
* Real-Time Systems

'''Memory and Resource Management'''

* Physical, Virtual, Paging, help?!?
* What segments are About?
* Allocating and freeing memory
* File management
* Algorithms and tips for memory management

'''Scheduling'''

* Tasks, processes and threads
* Context switching
* Scheduling algorithms
* Multiprocessor scheduling
* Sleeping and waiting processes

'''Process Sychronization / Inter-Process Communication'''

* Semaphores, mutexes, spinlocks et al.
* Signals
* Message passing
* Shared memory
* Remote procedure calls

==Tools==
'''[[Bootloaders]]'''

* [[GRUB]]
* [[LILO]]
* [[XOSL]] - Extended Operating System Loader
* System Commander
* Boot Magic
* Rolling your own Bootloader

'''[[:Category:Compilers|Compilers]]'''

* Watcom C/C++
* Visual C/C++
* [[GCC]]
** Using GCC Under Unix and Linux
** Using GCC Under Windows
** DJGPP
** Cygwin
** MinGW and Dev-C++
** Building a [[GCC Cross-Compiler]]
** Porting GCC to Your OS
** Calling conventions of GCC for x86 Systems
* OtherCompilers

'''Assemblers'''

* [[GAS|GNU as]]
* [[FASM]]
* [[NASM]]
* Differences Between AT&T and Intel assembler formats

'''Linkers'''
* ld
* ar

'''Emulators'''
* VMWare PC Emulator
* [[Bochs]] (i386) PC emulator
* SimICS
* [[QEMU]]
* Microsoft Virtual PC

'''Working with Disk Images'''
* ...under Linux
* ...under FreeBSD
* ...under MacOS X
* ...under Windows

==Reference Information==
'''Executable File Types'''

'''Filesystems'''
* [[FAT|FAT12 Details]]
* AmigaFFS Details

===Hardware ===
'''CPU'''
* AMD K6 writeback optimisations
* How can I tell CPU speed ?
* Model-specific register
* Sample inline functions
* Tell me about x86 64 bits CPU ...
* The IA32 architecture family
* What is v8086 mode?
* [[CPU Bugs]]

'''Memory'''
* How do I determine the amount of RAM?
* Why cant I access all my memory?

'''IRQ's and Exceptions, PIC, NMI, APIC, OPIC'''
* Can I remap the PIC?
* Getting Keyboard Input
* Help!? I can't get interrupts working
* How do I know if an IRQ or exception is firing?
* Interrupt service routines
* Interrupts for dummies
* So whats the [[Non-Maskable Interrupt|NMI]] then?
* Tell me about the APIC
* Tell me about the OPIC
* What is the PIC?

'''Video'''
* [[Accelerated Graphic Cards|Accelerated graphic cards]]
* How do I detect if I have a colour or monochrome monitor?
* How do I draw things in protected mode?
* How do I move the cursor when I print?
* How do I output text to the screen in protected mode?
* VGA [[VGA Resources|Links]] and [[VGA Hardware|Documentation]]
  
'''Storage devices'''
* ATAPI CDRom
* DMA
* Floppy driver

'''Plug and Play'''
* I heard you can do PNP calls with the BIOS in Protected Mode?
* Where can I find programming info on PNP?

'''PCI, USB and things like this ...'''
* [[AGP]] information
* And what about [[USB]] ?
* I heard you can do PCI calls with the BIOS in Protected Mode?
* PCI Section of Pentium VME
* [[USB Tutorial]]
* Where can I find programming info on [[PCI]]?

'''Network cards and communicating devices'''
* [[3c556]]
* [[Serial ports]]
* [[Ne2000]]

'''Non-ix86 Development'''
* [[:Category:PowerPC|PowerPC]]

== Resources ==

* Books on OS Development
* Assembler language
* Some small kernels with source
* BareBones - a minimalistic example of how to get from GRUB into C space.
* BareBonesC++ - well, guess. :-)
* Disk Image with preinstalled GRUB
* Chip Numbers, Acronyms and Things
* [[Resources|Links]]

== Forum Information etc. ==

* [[How To Ask Questions|How to ask questions]]
````

<a id="w002185"></a>

## W002185 — AGP Information

Bron-ID: W002185
Namespace: Main (0)
Revisie: 4747 | Datum: 2007-10-23T10:07:33Z
Bron: https://wiki.osdev.org/AGP_Information
Revisie-URL: https://wiki.osdev.org/index.php?title=AGP_Information&oldid=4747
Doorverwijzing: AGP

````text
#REDIRECT [[AGP]]
````

<a id="w002187"></a>

## W002187 — ISA

Bron-ID: W002187
Namespace: Main (0)
Revisie: 30189 | Datum: 2026-03-11T02:45:58Z
Bron: https://wiki.osdev.org/ISA
Revisie-URL: https://wiki.osdev.org/index.php?title=ISA&oldid=30189

````text
{{In Progress}}
- Industry Standard Architecture, ISA may also refer to [[Instruction Set Architecture]].

The '''Industry Standard Architecture''' (ISA) bus was created for the original IBM PC back in 1981. At that stage, it was an 8-bit, 5MHz bus (2.39MB/s), but was later upgraded to 16-bits at 8MHz (8.33MB/s). Today, the ISA bus is archaic and incredibly slow, but it is still commonly found in older machines, and many of the most common, basic devices are connected to it. For this reason, it is still supported by many operating systems. It is slowly being replaced by the Super I/O chip that is common amongst modern machines.


== I/O Ports and Initiation ==

There are three 8-bit ports specified by the ISA PnP standard that are used for accessing configuration spaces.

{| class="wikitable" style="margin:left"
|+ ISA Plug-and-Play (PnP 1.0A 7)
|-
! Address !! Name !! R/W
|-
| 0x279 (Same is LPT status, but W) || ADDRESS|| W
|-
| 0xA79 || WRITE_DATA|| W
|-
| 0x203-0x3FF (Relocatable) || READ_DATA || R
|}

'''Note: Each of these ports use 12-bit decode'''

The ADDRESS port is an 8-bit port that takes two bytes. The first byte send is the CSN, or card select number. The second is the offset in the configuration space. The address is the same as the printer status, but ports are bidirectional on PCs.

Cards in PnP mode are disabled on startup and are in a "Wait for Key" state. Once configured, they should be returned to this state. The Initiation Key is a 32-byte sequence sent to the ADDRESS port that will put the cards in configuration mode. Send zero twice to clear any previous address before sending this key.
<syntaxhighlight lang="asm">
; Warning: not tested
SendKey:
	pushfd            ; Save IF
	cli
	cld
	mov    dx,ADDRESS
	mov    esi,Key
	mov    ecx,32+2
	rep    outsb
	popfd             ;  Restore IF, nothing else changed
	ret
</syntaxhighlight>

The key is defined as:
<syntaxhighlight lang="asm">
Key:
	; Zero bytes to clear ADDRESS
	DW     0
	; The actual key
	DB     0x6A,0xB5,0xDA,0xED,0xF6,0xFB,0x7D,0xBE
	DB     0xDF,0x6F,0x37,0x1B,0x0D,0x86,0xC3,0x61
	DB     0xB0,0x58,0x2C,0x16,0x8B,0x45,0xA2,0xD1
	DB     0xE8,0x74,0x3A,0x9D,0xCE,0xE7,0x73,0x39
</syntaxhighlight>

AMD recommends (PCnet-ISA II 3) disabling interrupts while performing the IO operation to prevent anything else from interfering (as shown above). No other ports should be accessed. Although not mentioned in the original specification, it is also recommended by AMD to '''send it twice''' in case the key is not received the first time.

== Isolation and CSNs ==

The specification details isolation of cards. BIOS is supposed to perform this procedure. The term "plug-and-play software" in the specification may refer to the OS and firmware. On startup, the BIOS will send the initiation key, perform the serial isolation procedure, and take the cards out of isolation or configuration mode and have them ''wait for key''. Each card is assigned a sequential handle starting from 1 called a CSN. The number of CSNs that the BIOS assigned, as well as the READ_DATA port that the BIOS decided can be found with PnP function 40h.

Despite the BIOS performing isolation itself, the PnP BIOS specification says that the OS should not rely on the BIOS at all and do it again. 

== Card Control Space ==

Appendix A specifies the card-global configuration space. Here is an overview of what the card control section looks like:
{| class="wikitable" style="margin:left"
|+ ISA Plug-and-Play Register Space (PnP 1.0A 7)
|-
! Offset !! Name !! What it does
|-
| 0x00 || Set READ_DATA   || Set the READ_DATA address
|-
| 0x01 || Serial Isolation|| (TODO: Add this)
|-
| 0x02 || Config Control || See below for spec
|-
| 0x03 || Wake[CSN] || Writing here wakes that CSN
|-
| 0x04 || Resource Data|| Reading fetches the next byte
|-
| 0x05 || Status || If bit 0 is set, it is safe to read data
|-
| 0x06 || Card Select Number || The CSN, can be changed
|-
| 0x07 || Logical Device || Selects a logical device, read-only zero if none
|-
| 0x08-1F||Card reserved ||   ~
|-
| 0x20-0x2F||Vendor defined|| ~
|}

'''Note: Some operations appear to be global in scope'''
=== Waking Up Cards ===
If it is not clear, writing to the Wake[CSN] port sets the actual CSN to perform the wake operation. This is required in the isolation procedure.

=== Configuring READ_DATA ===
The Set READ_DATA register (also referred to as RD_DATA) is an 8-bit value that relocates the READ_DATA port to the range specified above. This value is practically shifted left by three and or'ed with "11" in binary (Page 15) to create a 10-bit address slanted by 3. The value written to this register would be:

 Set_READ_DATA = (PhysicalPort & 0x3F8) >> 3 /* Make sure first bits are 11 */
E.g. writing 0x40 to this port will place the READ_DATA port at 0x203

Reading this register does nothing, the programmer must assign this port and check for conflicts. There is only one for all cards.

=== Resource Data ===

Resource data is read from the dedicated register for the logical device selected. Bit zero of the status register must be polled until it is one before reading the resource data. The resource data register is READ ONLY. This resource data is not to be confused with the resource configuration registers in the card control space. It defines what resources the card '''is able''' to use. If a fully PnP card uses any memory address range, the static resource entry will be 0x00000000-0xFFFFFFFF (AFAIK), and the same applies to ports, DMA, and IRQs. If only one DMA or IRQ is implemented, that device must use it. For IO ports, the tag can be a fixed descriptor or regular one. If the tag is a fixed descriptor, the old 10-bit ISA decode is used for that range.

The PnP ISA specification allows for 32-bit data and addresses. This is definitely not used by any ISA cards because the ISA bus is 16-bit and only supports 24-bit addresses. Despite this, the OS should support this by indirection for consistency.

The PnP BIOS uses the same format for reporting devices connected to the mainboard.

== Logical Device Local Control ==

The logical card control registers are found at 0x30-0x3F. This space is present on all cards, even if they do not have any logical devices. Selecting the logical device swaps out this space. To check if the card is multi-function, write a value other than zero to the logical device register and try to write it back. If a read of LDS still generates a zero, the device has no functions. Otherwise, write back the original value (zero i.g.).

=== Activation ===

The activate register turns on the card or the logical device belonging to that card when the first bit is on. If there are no resources for a card, the OS will have to disable the card by setting the register to zero (the user should be notified). IO range check should be disabled while activating.

== Resource Configuration ==

The resource data is stored in the logical card space. The memory information starts at 0x40 or 0x76 and is seen at Table A-3. Memory descriptors are four bytes for 24-bit addressing and nine for 32-bit. The specifics are detailed in A.3.1.

== Device Conflicts ==
There are broadly four types of resources that cause device conflicts (listed in the order problems are generally experienced):
* Memory addresses
* [[Interrupts|IRQ]] requests
* [[DMA]] channels
* I/O port addresses
Managing these resources becomes the responsibility of the operating system. Fortunately, Microsoft has created a standard called [[Plug-and-Play]] (PnP) that helps with this. Not all ISA cards support it, however, so some luck is required in that the person building the system has set the correct jumpers.

== Decode Problems ==
The ISA bus can support x86 16-bit port addresses, but many older ISA cards only support 10-bit address decode. This presents a major problem that can lead to conflict. When the processor tries to access a certain resource on the ISA bus, it will signal all ISA devices in the hopes that only one responds. If a card can only handle ten of the address bits, it can respond to IOR and IOW signals when it should not be, because it does not see the upper bits.

So if a device is located at port 100H, that 10-bit address with every possible combination for the upper 6 bits will map to the same device (e.g port 0xFF00). To make this totally impossible, one can simply restrict PnP port addresses to be within the 1024-byte range to make such conflicts impossible, though this would be a limitation. It is also important to note that PCI uses 0CF8-0xCFF, which is above the 10-bit range and cannot be reassigned. Built-in devices reported by the PnP BIOS such as the PCI host bridge should the address range with the top 6 bits masked out. This way, ISA cards cannot be configured in a conflicting manner.

Conflicts are not restricted to IO and should apply to memory too, but any mainboard with an ISA bus and more than 16MB of RAM will probably be smart enough to check addresses before sending them to the ISA bus. They may check IO port addresses too, but it is always best to verify rather than trust 90's era hardware that sometimes had dubious standard compliance.

The decode of legacy devices like the LPT, COM, etc. that are typically embedded in the mainboard are probably 16-bit on PnP boards, but checking with the BIOS is the only way to be sure.

== ISA Plug-and-Play ==

The original ISA bus, or more specifically, the 16-bit PC/AT bus used static non-sharable resource assignments and had no isolation between cards. There was no way to program it from software. A number of buses appeared on the PC market such as [[VLB]], [[PCI]], and [[EISA]]. These architectures were incompatible with the original design. Plug-and-play ISA was introduced to get around the original limitations while maintaining compatibility and improving user friendliness. The following is a general overview of the ISA Plug-and-play specification. The goal of this section is to clarify the documentation as there are many forward references to the appendices. See the original document (ISA PnP 1.0A) for more authoritative information (including timeouts).

Developers looking for robust support for the 1990's generation of hardware may want to add plug-and-play ISA support.

=== Supporting ISA PnP ===

[[PnP]] ISA devices can suport varrying levels of configurability. For example, a card that implements a parallel port may only be configurable to three standard addresses for PC compatibility. The static resource data mentioned previously reports what data a device supports, but not necessarily what it will use.

The plug-and-play management code of the OS must be able to handle every possible bus, and combinations of buses, as some computers have [[PCI]] and ISA. Reserving resources should be supported so that less-so or non-configurable devices can work.

The following specifics of ISA PnP must be implemented in the operating system:
* Memory address ranges supported
* IO port ranges
* IRQs
* Address and port decode
* Data width (8/16/32-bit)

== See Also ==
=== External Links ===
* [http://www.osdever.net/documents/PNP-ISA-v1.0a.pdf Tutorial on how to enable PCnet-ISA II PnP]
* [https://www.amd.com/system/files/TechDocs/PCnet-ISA_II.pdf PnP ISA Specification]
* [[wikipedia:Industry Standard Architecture|ISA article on Wikipedia]]

[[Category:Buses]]
[[de:Industry_Standard_Architecture]]
````

<a id="w002188"></a>

## W002188 — DP8390

Bron-ID: W002188
Namespace: Main (0)
Revisie: 22650 | Datum: 2018-09-09T23:36:25Z
Bron: https://wiki.osdev.org/DP8390
Revisie-URL: https://wiki.osdev.org/index.php?title=DP8390&oldid=22650

````text
{{Stub}}

The first link is a great general reference for other NIC chipsets as well because it goes through step-by-step instructions on how to do things like IRQ and DMA handling. 

== See Also ==
=== External Links ===
* [http://osdever.net/documents/WritingDriversForTheDP8390.pdf?the_id=56 Writing Drivers for the DP8390 NIC Family of Ethernet Controllers]
* [http://osdever.net/documents/DP8390Details.pdf?the_id=46 DP8390D/NS32490D NIC Network Interface Controller, Architecture Datasheets and Guides for the DP8390 chipset family.] 

[[Category:Network Hardware]]
[[Category:Standards]]
````

<a id="w002189"></a>

## W002189 — OSDev Wiki talk:About

Bron-ID: W002189
Namespace: OSDev Wiki talk (5)
Revisie: 4795 | Datum: 2007-10-29T14:36:46Z
Bron: https://wiki.osdev.org/OSDev_Wiki_talk:About
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDev_Wiki_talk%3AAbout&oldid=4795

````text
== Still running Solaris? ==

Didn't chase recently switch from Solaris to Linux? Netcraft seems to think so.. [http://uptime.netcraft.com/up/graph?site=www.osdev.org See Here] --[[User:Brynet-Inc|Brynet-Inc]] 14:04, 28 October 2007 (CDT)
:http://www.osdev.org/phpBB2/viewtopic.php?t=13865&highlight=linux I'd like to know which version before I edit the About page. Maybe Chase knows? [[User:Yayyak|Yayyak]] 22:35, 28 October 2007 (CDT)

For now the system is running CentOS 4.4 with OpenVZ. Inside of OpenVZ I'm running Fedora Core 6 which is being used to host the site. -[[User:Chase|Chase]] 09:36, 29 October 2007 (CDT)
````

<a id="w002190"></a>

## W002190 — User talk:A-OS

Bron-ID: W002190
Namespace: User talk (3)
Revisie: 4810 | Datum: 2007-11-02T07:18:42Z
Bron: https://wiki.osdev.org/User_talk:A-OS
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AA-OS&oldid=4810

````text

````

<a id="w002191"></a>

## W002191 — Talk:IDT2

Bron-ID: W002191
Namespace: Talk (1)
Revisie: 4865 | Datum: 2007-11-14T21:43:25Z
Bron: https://wiki.osdev.org/Talk:IDT2
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AIDT2&oldid=4865

````text
It seems that [[IDT]] has all the contents of this article (and this article looks kindof looted), however a formal discussion is missing. After completing [[IDT]], I think that everything contained here is present in that article, so this article can be euthanised. Confirmations welcome - [[User:Combuster|Combuster]] 05:39, 14 November 2007 (CST)
:Oh, by all means! Anything to remove the last page from PFR! :) --Alboin (By the way, this is a confirmation.)
::Confirmation. BTW, nice work on removing all the other pages from PFR! I was trying to do one a day... and then BAM! --[[User:Yayyak|Yayyak]] 15:43, 14 November 2007 (CST)
````

<a id="w002193"></a>

## W002193 — File:Page dir.png

Bron-ID: W002193
Namespace: File (6)
Revisie: 26486 | Datum: 2021-10-14T22:37:04Z
Bron: https://wiki.osdev.org/File:Page_dir.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3APage_dir.png&oldid=26486

````text
Page Directory Format
````

<a id="w002194"></a>

## W002194 — File:Page table.png

Bron-ID: W002194
Namespace: File (6)
Revisie: 4879 | Datum: 2007-11-15T20:12:27Z
Bron: https://wiki.osdev.org/File:Page_table.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3APage_table.png&oldid=4879

````text
Page Table Image
````

<a id="w002195"></a>

## W002195 — File:Gdt entry.png

Bron-ID: W002195
Namespace: File (6)
Revisie: 4883 | Datum: 2007-11-16T01:27:51Z
Bron: https://wiki.osdev.org/File:Gdt_entry.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AGdt_entry.png&oldid=4883

````text
GDT entry diagram for [[GDT]]
````

<a id="w002196"></a>

## W002196 — File:Gdtr.png

Bron-ID: W002196
Namespace: File (6)
Revisie: 4886 | Datum: 2007-11-16T01:39:22Z
Bron: https://wiki.osdev.org/File:Gdtr.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AGdtr.png&oldid=4886

````text
GDTR Diagram
````

<a id="w002197"></a>

## W002197 — File:Gdt bits.png

Bron-ID: W002197
Namespace: File (6)
Revisie: 4887 | Datum: 2007-11-16T02:06:17Z
Bron: https://wiki.osdev.org/File:Gdt_bits.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AGdt_bits.png&oldid=4887

````text
GDT Bits Diagram
````

<a id="w002198"></a>

## W002198 — Template:Conversion

Bron-ID: W002198
Namespace: Template (10)
Revisie: 4905 | Datum: 2007-11-20T09:37:39Z
Bron: https://wiki.osdev.org/Template:Conversion
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AConversion&oldid=4905
Doorverwijzing: Template:Help Us

````text
#REDIRECT [[Template:Help Us]]
````

<a id="w002200"></a>

## W002200 — Category:MIPS

Bron-ID: W002200
Namespace: Category (14)
Revisie: 4914 | Datum: 2007-11-20T10:20:16Z
Bron: https://wiki.osdev.org/Category:MIPS
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AMIPS&oldid=4914

````text
The MIPS CPU architecture is used in computer architectures like SGI O2 and Octane systems, Nintendo N64 as well as the Sony Playstation and Playstation 2.

[[Category:Platforms]]
````

<a id="w002202"></a>

## W002202 — Talk:Hexadecimal Notation

Bron-ID: W002202
Namespace: Talk (1)
Revisie: 4949 | Datum: 2007-11-26T09:47:03Z
Bron: https://wiki.osdev.org/Talk:Hexadecimal_Notation
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AHexadecimal_Notation&oldid=4949

````text
== Replacing This Article ==
I have worked on a new hexadecimal article at [[User:AJ/Hexadecimal Notation]] which is a bit more in-depth than the current version (which seems to be put on the wiki a while ago now). Could I get opinions on whether I could replace the existing article (after some tidying up)? I also plan to do binary (with bitwise compares etc...) and possibly decimal versions of the article too. If it's best, I'll ask this in the forums. --[[User:AJ|AJ]] 06:18, 23 November 2007 (CST)

: What is missing in ''your'' version is how to convert between decimal and hex, you currently say nothing about getting the decimal value of a hex number, and you refer to printf for "converting" decimal to hex, which people will not always be able to use. For these reasons I say no for the time being. - [[User:Combuster|Combuster]] 15:16, 25 November 2007 (CST)

: Fair enough - I'll work on it and add something about conversion to decimal. I'm sure I can't see where I suggest using printf to convert hex to decimal, though. The only place I see a reference to printf() is where I distinguish between printing Hex numbers with either capital or lowercase letters. --[[User:AJ|AJ]] 03:47, 26 November 2007 (CST)
````

<a id="w002204"></a>

## W002204 — Bootf

Bron-ID: W002204
Namespace: Main (0)
Revisie: 8657 | Datum: 2009-07-28T04:21:01Z
Bron: https://wiki.osdev.org/Bootf
Revisie-URL: https://wiki.osdev.org/index.php?title=Bootf&oldid=8657

````text
'''Bootf''' is a small [[FAT12]] floppy [[bootloader|bootloader]] for protected mode OS images, created by John Fine. It is also a coding sample for cramming lots of operations into very little code. The boot code on a FAT diskette or FAT partition is limited to 0x1C0 bytes (0x3E bytes are reserved at the beginning and 0x2 at the end of a 0x200 byte sector). BOOTF.ASM fits with 0x1A bytes to spare.

== Features ==
* Enables the A20 gate.
* Switches to big real mode.
* Computes the locations of the FAT, the root directory and the first cluster.
* Reads in the root directory.
* Scans the root directory for a specific file (KERNEL.BIN).
* Reads in the FAT.
* Reads the file (KERNEL.BIN) to RAM starting at physical 1Mb.
* Builds two page tables and one page directory mapping:
** The first 4Mb linear to the first 4Mb physical.
** Linear FF800000 (where all my protected mode images start) to physical RAM starting at 1Mb.
** Self map the page directory to the end of linear memory.
* Switch to protected mode with paging turned on.
* JMP to KERNEL.BIN at linear FF800000.

== Limitations ==
* Runs only on a 386+.
* The FAT structure must be the first thing on the drive (no partitions). It just uses "reserved" sectors before the FAT, not "hidden" etc.
* The sector size must be 512 bytes.
* There are less than 256 sectors per fat (safe, since FAT12 has less than 4096*1.5 bytes per FAT).
* LBN of root directory less than 65536 (safe in unpartitioned FAT12).

== See Also ==
=== External Links ===
* [http://files.osdev.org/mirrors/geezer/johnfine/bootf02.zip Bootf02.zip] (Mirror)
* [http://files.osdev.org/mirrors/geezer/johnfine/index.htm 'Official' Site] (Mirror)
* [http://redir.no-ip.org/mirrors/my.execpc.com/~geezer/ Chris Giese OSDev] (Mirror)

[[Category:Bootloaders]]
````

<a id="w002205"></a>

## W002205 — User:Viral

Bron-ID: W002205
Namespace: User (2)
Revisie: 4951 | Datum: 2007-11-28T09:34:30Z
Bron: https://wiki.osdev.org/User:Viral
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AViral&oldid=4951

````text
Hello, I am Viral Patel from India. I have made an operating system called Taj Operating System in C++. Following is my home page:
* [http://www.viralpatel.net Viral Patel's Home Page]
````

<a id="w002207"></a>

## W002207 — List of editors

Bron-ID: W002207
Namespace: Main (0)
Revisie: 26190 | Datum: 2021-07-17T08:44:37Z
Bron: https://wiki.osdev.org/List_of_editors
Revisie-URL: https://wiki.osdev.org/index.php?title=List_of_editors&oldid=26190

````text
== Editors for Windows ==
* [http://gonix.ten.lt/ SEG]
* [http://notepad-plus.sourceforge.net/uk/site.htm Notepad++]
* [http://www.flos-freeware.ch/notepad2.html Notepad2]
* [http://syn.sourceforge.net/ Syn Text Editor]
== Editors for OS X ==
* [https://developer.apple.com/xcode/ Xcode]
== Cross-platform editors ==
=== GTK ===
* [http://www.gnome.org/projects/gedit/ Gedit]
* [http://geany.uvena.de/ Geany]
* [https://tarot.freeshell.org/leafpad/ Leafpad]
* [https://github.com/tsujan/FeatherPad FeatherPad]
* [https://github.com/codebrainz/mousepad Mousepad]
=== Qt ===
* [http://kate-editor.org/kate Kate]
=== Other ===
* [http://www.nedit.org/ Nedit] (Windows-like)
* [http://www.sublimetext.com/ Sublime Text]
* [http://www.atom.io/ Atom]
* [[SciTE]]
* [https://code.visualstudio.com/ Visual Studio Code]
* [http://www.codeblocks.org/ Code::Blocks]
=== Pseudographical ===
* [http://mitglied.lycos.de/albkleine/ e3]
* [http://www.nano-editor.org/ Nano]
=== Pseudographical/graphical ===
* [http://www.gnu.org/software/emacs/ Emacs]
* [http://www.vim.org/ Vim]

[[Category:Tools]]
````

<a id="w002208"></a>

## W002208 — Talk:List of editors

Bron-ID: W002208
Namespace: Talk (1)
Revisie: 6990 | Datum: 2009-01-05T13:32:09Z
Bron: https://wiki.osdev.org/Talk:List_of_editors
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AList_of_editors&oldid=6990

````text
IMHO the [[:Category:Editors]] exists for just this subject. If the sole purpose is to put up links then the [[resources]] page would be more appropriate.
:Agreed. Any disagreement? [[User:Jackscott|Jackscott]] 22:14, 24 November 2008 (UTC)
::I agree. There is no wikilinks to this page, it's not categorized (anymore) and it just contains links. Not really that useful IMHO. Everyone have their favorite editor. On the other hand it should be mentioned if there's an editor with special features geared towards developing operating systems. - [[User:Walling|Walling]] 13:32, 5 January 2009 (UTC)
````

<a id="w002209"></a>

## W002209 — Linux Kernel Primer

Bron-ID: W002209
Namespace: Main (0)
Revisie: 28235 | Datum: 2023-07-10T22:02:54Z
Bron: https://wiki.osdev.org/Linux_Kernel_Primer
Revisie-URL: https://wiki.osdev.org/index.php?title=Linux_Kernel_Primer&oldid=28235

````text
== Introduction ==

The Linux kernel source is an excellent case study for a modern, used, and tested operating system design.  It was not, however, created as a teaching tool. The source is often not commented and provides very little hand holding.  That said, there are a great many places and things to learn.  A full description of the linux kernel source could span multiple books.

== Obtaining the Source  ==

http://www.kernel.org is the major source for all that is the linux kernel source code.
http://lxr.free-electrons.com lets you browse the source online. It also has a very nice search engine.

== Version Numbers ==

Basic history of the major functional changes:

* 1.0 and prior - x86 only, a.out binary format
* 1.2 - adds support for a few more CPU types, ELF binary format, modules
* 2.0 - adds SMP support

At 2.2 functionality reached a point where improvements became much more iterative.  The 2.2 kernel implements most of the major features required of a modern kernel.  Using anything below 2.2 as a reference or basis for a fork could easily result in serious architectural challenges for your derived code or design. Core improvements to the kernel have been made after 2.2, and the kernel developers had good reason for them.

The size of the kernel can be attributed to the plethora of file systems, device drivers, architecture supports, and kernel services and features.  While 2.6 is a massive code base, removing the non-core parts reduces the size. Drivers take up over 100MB, non-i386 architectures, around 50MB, file systems, around 20MB. As a general rule, you won't need to venture into the various subsystems and drivers that you are not using.

== Browsing and Using The Source ==

It's better to not try and follow the whole of the Linux kernel source as a single process flow.  Different architectures start in different ways, but the generic portions can be found in init/.  Keep in mind that an operating system kernel is very different from a user space program - the kernel is responding and servicing to user space requests.

Instead of trying to figure out a process flow for the entire kernel, try to figure out what's going on for a single event or system call. System calls can easily be found by searching the code for functions beginning with "sys_".  These generally have descriptive names that correspond to the kernel requirements for a C library call.

Looking at a driver source in Linux can tell you a great deal about a given piece of hardware functionality. Keep in mind that the Linux kernel source isn't always "correct" about how it handles specific hardware.  Many drivers were written with little or no documentation or standards information.  

Unless you modify large portions of your kernel to use the same names and conventions of the Linux kernel, Linux kernel source is not easily ported away from Linux.  Also, the source is GPL, so your kernel source is going to require proper licensing if you use anything.  There are some exceptions to this rule, mostly in code shared between projects located in the Linux kernel tree.

The amount of infrastructure in Linux for basic system tasks - interrupt handlers, virtual memory, swap, block io - will make a lot of that code difficult to read if you are just starting.  It might be better to read other tutorials here before looking at how a "real" kernel does things, so that you can recognize what is infrastructure setup and what is required to perform a given task.  The interesting side effect is that big picture kernel logic, obscured in other small example kernels, becomes very clear in Linux.  The scheduler algorithm, for instance, is fairly visible and easy to read in a single source file with little of the task swapping details obscuring how it works.

As the Linux kernel already provides tested mechanisms for many different tasks, creating a kernel module can allow you test your own kernel code.  The downside to attempting this is that you'll have to figure out how to "play nicely" with the portions of Linux you are trying to use...

== Important Source Files ==

This list is by no means complete. These files provide common functionality likely to be required in any operating system kernel.


* Locking, Synchronization
** kernel/mutex.c, include/linux/mutex.h - kernel space mutex implementation
** kernel/futex.c - kernel level support for 'fast' userspace mutual exclusion
** kernel/spinlock.c, include/linux/spinlock.h - kernel spinlock implementation
* Block devices
** block/elevator.c - support for the various IO scheduling algorithms
** block/noop-iosched.c - the easiest to read and understand IO scheduler (noop)
** block/ll_rw_blk.c - where block requests meet a block device queue
* File system
** fs/read_write.c - read, write system calls
** fs/file.c - management of file handles
** fs/inode.c - functionality for inodes (file information structs, sometimes like kernel file handles)
** fs/open.c - contains several of the major file operations 
** fs/cramfs/inode.c, fs/cramfs/uncompress.c - a very small filesystem with easy to read/find code* 
* Process Management
** kernel/sched.c - the linux scheduler, schedule() is the function to look at here
** kernel/workqueue.c - a workqueue that provides a means for doing work
** arch/i386/kernel/process.c - large portion of process handling for x86
* Binary formats
** fs/binfmt_elf.c - ELF loading
* Memory Management:
** mm/slab.c - slab allocator, also home for kmalloc
** mm/vmalloc.c - virtual contiguous memory allocator
** mm/page_alloc.c - buddy allocation

== Finding what you are looking for ==
The arch/ directory contains architecture dependent files.  If you want to learn about x86 development, the best place to look is arch/i386. Most any hardware specific functionality is contained in drivers/.  Keep in mind that things like PCI are used by multiple architectures, and so they are located under drivers/ as well.  Sound and Networking are treated differently and reside under their own directory sound/ and net/ with drivers in subdirectories there.  Core kernel functionality lives in kernel/, and lib/.

There is "Documentation" in the Documentation directory.  This directory is not what you think it is.  It is more of a collection of tidbits of knowledge than any decent information on the kernel.

== Resources ==
Websites:

* https://kernelnewbies.org/ - guides for new Linux kernel developers

* http://www.tldp.org/HOWTO/KernelAnalysis-HOWTO.html - guide to the Linux kernel organized by subsystems

* https://syscalls.kernelgrok.com/ - a reference list of Linux syscalls

Books:

* Linux Kernel Development, by Robert Love - a very decent 'overview' book of the Linux kernel.  It doesn't go into too much detail, but provides enough of a big picture and detail view to really get started on a Linux kernel project.

* Understanding the Linux Kernel, David Bovet - more detailed than Linux Kernel Development.  If you want to do more work than browsing the kernel, this has more of the detail required.

[[Category:Kernel]]
[[Category:Linux]]
````

<a id="w002210"></a>

## W002210 — User talk:Crazygray1

Bron-ID: W002210
Namespace: User talk (3)
Revisie: 5036 | Datum: 2008-01-04T20:19:00Z
Bron: https://wiki.osdev.org/User_talk:Crazygray1
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ACrazygray1&oldid=5036

````text
== Many-edit ==
Would you please try and use the preview version instead of saving the pages lots of times until you got it right? Also, would you mind adding a meaningful summary each time you make a change. Every modification will appear on the changes list, and I think you can understand how 5 items on the same page without subject looks a lot like spam on my management tool. 

Thanks in advance. - [[User:Combuster|Combuster]] 14:27, 1 January 2008 (CST)
:Sorry about that. I'm kind of new to editing wiki's. --Crazygray1
::Generally on these talk pages headers are used to seperate different discussions, and messages are tabbed with ':'s. Also, it's nice to sign your messages. --[[User:Alboin|Alboin]] 14:19, 4 January 2008 (CST)
````

<a id="w002211"></a>

## W002211 — ISO 9660

Bron-ID: W002211
Namespace: Main (0)
Revisie: 29375 | Datum: 2025-02-06T23:03:34Z
Bron: https://wiki.osdev.org/ISO_9660
Revisie-URL: https://wiki.osdev.org/index.php?title=ISO_9660&oldid=29375

````text
{{Filesystems}}
'''ISO 9660''' is the standard file system for CD-ROMs. It is also widely used
on DVD and BD media and may as well be present on USB sticks or hard disks. 
Its specifications are available for free under the name '''ECMA-119'''.

== Overview and caveats ==
ISO 9660 is not a complex file system, but has a few quirks that are worth remembering. It seems that some operating systems also create non-compliant CDs, so beware! The main example of this is the character set that is available for file names. Strictly, filenames may only consist of uppercase letters A-Z, digits,
dots, and underscores. Further there is a semicolon which separates the visible file name from its version number suffix. Many operating systems also allow lower case letters and other characters. Linux's [[VFS]] displays lower case filenames to the user despite the CD contents actually containing upper case characters.

=== Sector size ===
An ISO 9660 sector is normally 2 KiB long. Although the specification allows for alternative sector sizes, you will rarely find anything other than 2 KiB.

=== Numerical formats ===
Another quirk of the system is that it has several numbering formats and multi-byte numbers are often represented in ''both-endian'' format. The ISO 9660 standard specifies three ways to encode 16 and 32-bit integers, using either little-endian (least-significant byte first), big-endian (most-significant byte first), or a combination of both (little-endian followed by big-endian). Both-endian (LSB-MSB) fields are therefore twice as wide. For this reason, 32-bit LBA's often appear as 8 byte fields. Where a both-endian format is present, the x86 architecture makes use of the first little-endian sequence and ignores the big-endian sequence.

{| {{wikitable}}
! Encoding
! Description
|-
| int8 || Unsigned 8-bit integer.
|-
| sint8 || Signed 8-bit integer.
|-
| int16_LSB || Little-endian encoded unsigned 16-bit integer.
|-
| int16_MSB || Big-endian encoded unsigned 16-bit integer.
|-
| int16_LSB-MSB || Little-endian followed by big-endian encoded unsigned 16-bit integer.
|-
| sint16_LSB || Little-endian encoded signed 16-bit integer.
|-
| sint16_MSB || Big-endian encoded signed 16-bit integer.
|-
| sint16_LSB-MSB || Little-endian followed by big-endian encoded signed 16-bit integer.
|-
| int32_LSB || Little-endian encoded unsigned 32-bit integer.
|-
| int32_MSB || Big-endian encoded unsigned 32-bit integer.
|-
| int32_LSB-MSB || Little-endian followed by big-endian encoded unsigned 32-bit integer.
|-
| sint32_LSB || Little-endian encoded signed 32-bit integer.
|-
| sint32_MSB || Big-endian encoded signed 32-bit integer.
|-
| sint32_LSB-MSB || Little-endian followed by big-endian encoded signed 32-bit integer.
|}


=== Date/time format ===

The date/time format used in the ''Primary Volume Descriptor'' is denoted as ''dec-datetime'' and uses ASCII digits to represent the main parts of the date/time:

{| {{wikitable}}
!Offset
!Size
!Datatype
!Description
|-
| 0 || 4 || strD || Year from 1 to 9999.
|-
| 4 || 2 || strD || Month from 1 to 12. 
|-
| 6 || 2 || strD || Day from 1 to 31.
|-
| 8 || 2 || strD || Hour from 0 to 23.
|-
| 10 || 2 || strD || Minute from 0 to 59.
|-
| 12 || 2 || strD || Second from 0 to 59.
|-
| 14 || 2 || strD || Hundredths of a second from 0 to 99.
|-
| 16 || 1 || int8 || Time zone offset from GMT in 15 minute intervals, starting at interval -48 (west) and running up to interval 52 (east). So value 0 indicates interval -48 which equals GMT-12 hours, and value 100 indicates interval 52 which equals GMT+13 hours.
|}

All fields except for the offset from GMT are in ASCII digits. When the date and time is not specified, all string fields are ASCII '0' (for a total of 16 ASCII zeroes) and the last field is binary zero.

=== String format ===
Character strings are encoded with ASCII encoding. The specification does not permit all characters. It defines two sets of characters: 'a-characters' and 'd-characters'. You will see these terms used in the descriptor tables throughout this article. The character sets are:

<pre>
a-characters: A B C D E F G H I J K L M N O P Q R S T U V W X Y Z 0 1 2 3 4 5 6 7 8 9 _ 
              ! " % & ' ( ) * + , - . / : ; < = > ?
</pre>
<pre>
d-characters: A B C D E F G H I J K L M N O P Q R S T U V W X Y Z 0 1 2 3 4 5 6 7 8 9 _
</pre>

{| {{wikitable}}
! Encoding
! Description
|-
| strA || String with only ASCII a-characters, padded to the right with spaces.
|-
| strD || String with only ASCII d-characters, padded to the right with spaces.
|}

Note that not all CDs strictly adhere to the character sets specified in ISO 9660.

=== Filenames ===
Filenames must use d-character encoding (strD), plus dot and semicolon
which have to occur exactly once per filename.
Filenames are composed of a File Name, a dot, a File Name Extension,
a semicolon; and a version number in decimal digits. The latter two
are usually not displayed to the user.

There are three Levels of Interchange defined. Level 1 allows filenames
with a File Name length of 8 and an extension length of 3 (like MS-DOS).
Levels 2 and 3 allow File Name and File Name Extension to have a combined
length of up to 30 characters.

The ECMA-119 Directory Record format can hold composed names of up to 222
characters. This would violate the specs but must nevertheless be handled
by a reader of the filesystem.

=== Size Limitations ===
ISO 9660 filesystems can have up to 2 exp 32 blocks, i.e. 8 TiB.
Normally they will be restricted to the size of optical media.
(Currently up to 100 GiB with 4-layer BD-R.)

The maximum size of data files depends on the Level of Interchange
that is intended for the ISO filesystem. Levels 1 and 2 allow for
4 GiB - 1, because a single Directory Record can claim up to that 
number of bytes. Level 3 allows to have multiple consequtive
Directory Records with the same name. They all are to be concatenated
to a single data file. This means that a single data file can nearly
fill up the full 8 TiB of image size.

== System Area ==
An ISO 9660 filesystem begins by 32 KiB which may be used for arbitrary
data. This is often used to store boot information for the case that
the ISO 9660 filesystem is not stored on optical media, but rather on
a hard-disk-like device, e.g. on a USB stick.

So be prepared to find at that location a Master Boot Record
(MBR, for BIOS), a GUID Partition Table (GPT, for EFI), or an
Apple Partition Map (APM).

== Volume Descriptors ==
When preparing to mount a CD, your first action will be reading the volume descriptors (specifically, you will be looking for the ''Primary Volume Descriptor'').

Since sectors 0x00-0x0F of the CD are reserved as System Area,
the Volume Descriptors can be found starting at sector 0x10 (16). The format of the volume descriptors is as follows:

{| {{wikitable}}
! Offset
! Length (bytes)
! Field name
! Datatype
! Description
|-
| 0 || 1 || Type || int8 || Volume Descriptor type code (see below).
|-
| 1 || 5 || Identifier || strA || Always 'CD001'.
|-
| 6 || 1 || Version || int8 || Volume Descriptor Version (0x01).
|-
| 7 || 2041 || Data || - || Depends on the volume descriptor type.
|}

This means that each volume descriptor is therefore one sector (2 KiB) long.

=== Volume Descriptor Type Codes ===
The Volume Descriptor Type field specifies the type of Volume Descriptor:
{| {{wikitable}}
! Value
! Description
|-
| 0 || Boot Record
|-
| 1 || Primary Volume Descriptor
|-
| 2 || Supplementary Volume Descriptor
|-
| 3 || Volume Partition Descriptor
|-
| 4-254 || Reserved
|-
| 255 || Volume Descriptor Set Terminator
|}

When starting out with a basic CD, we are going to be interested in the Primary Volume Descriptor, which points us to the root directory and path tables, which both allow us to find any file on the CD. Using the path table is ideal for minimal implementations which do not wish to search the directory hierarchy node by node. This is slower (string comparisons across the entire file system) but easier to implement.

=== The Boot Record ===
The first type of Volume Descriptor is the "Boot Record". The descriptor format is as follows:

{| {{wikitable}}
! Offset
! Length (bytes)
! Field name
! Datatype
! Description
|-
| 0 || 1 || Type || int8 || Zero indicates a boot record.
|-
| 1 || 5 || Identifier || strA || Always "CD001".
|-
| 6 || 1 || Version || int8 || Volume Descriptor Version (0x01).
|-
| 7 || 32 || Boot System Identifier || strA || ID of the system which can act on and boot the system from the boot record.
|-
| 39 || 32 || Boot Identifier || strA || Identification of the boot system defined in the rest of this descriptor.
|-
| 71 || 1977 || Boot System Use || - || Custom - used by the boot system.
|}

The most common Boot System Use specification is [[El-Torito|El Torito]].
It records at bytes 71 to 74 as little-endian 32-bit number the 
block address of the El Torito Boot Catalog. This catalog lists the 
available boot images, which serve as starting points of booting systems.

=== The Primary Volume Descriptor ===
This is a lengthy descriptor, but it contains some very useful information for reading the rest of the file system.

{| {{wikitable}}
! Offset
! Length (bytes)
! Field name
! Datatype
! Description
|-
| 0 || 1 || Type Code || int8 || Always 0x01 for a Primary Volume Descriptor.
|-
| 1 || 5 || Standard Identifier || strA || Always 'CD001'.
|-
| 6 || 1 || Version || int8 || Always 0x01.
|-
| 7 || 1 || Unused || - || Always 0x00.
|-
| 8 || 32 || System Identifier || strA || The name of the system that can act upon sectors 0x00-0x0F for the volume.
|-
| 40 || 32 || Volume Identifier || strD || Identification of this volume.
|-
| 72 || 8 || Unused Field || - || All zeroes.
|-
| 80 || 8 || Volume Space Size || int32_LSB-MSB || Number of Logical Blocks in which the volume is recorded.
|-
| 88 || 32 || Unused Field || - || All zeroes.
|-
| 120 || 4 || Volume Set Size || int16_LSB-MSB || The size of the set in this logical volume (number of disks).
|-
| 124 || 4 || Volume Sequence Number || int16_LSB-MSB || The number of this disk in the Volume Set.
|-
| 128 || 4 || Logical Block Size || int16_LSB-MSB ||  The size in bytes of a logical block. NB: This means that a logical block on a CD could be something other than 2 KiB!
|-
| 132 || 8 || Path Table Size || int32_LSB-MSB || The size in bytes of the path table.
|-
| 140 || 4 || Location of Type-L Path Table || int32_LSB || LBA location of the path table. The path table pointed to contains only little-endian values.
|-
| 144 || 4 || Location of the Optional Type-L Path Table || int32_LSB || LBA location of the optional path table. The path table pointed to contains only little-endian values. Zero means that no optional path table exists.
|-
| 148 || 4 || Location of Type-M Path Table || int32_MSB || LBA location of the path table. The path table pointed to contains only big-endian values.
|-
| 152 || 4 || Location of Optional Type-M Path Table || int32_MSB || LBA location of the optional path table. The path table pointed to contains only big-endian values. Zero means that no optional path table exists.
|-
| 156 || 34 || Directory entry for the root directory || - || Note that this is not an LBA address, it is the actual Directory Record, which contains a single byte Directory Identifier (0x00), hence the fixed 34 byte size.
|-
| 190 || 128 || Volume Set Identifier || strD || Identifier of the volume set of which this volume is a member.
|-
| 318 || 128 || Publisher Identifier || strA || The volume publisher. For extended publisher information, the first byte should be 0x5F, followed by the filename of a file in the root directory. If not specified, all bytes should be 0x20.
|-
| 446 || 128 || Data Preparer Identifier || strA || The identifier of the person(s) who prepared the data for this volume. For extended preparation information, the first byte should be 0x5F, followed by the filename of a file in the root directory. If not specified, all bytes should be 0x20.
|-
| 574 || 128 || Application Identifier || strA || Identifies how the data are recorded on this volume. For extended information, the first byte should be 0x5F, followed by the filename of a file in the root directory. If not specified, all bytes should be 0x20.
|-
| 702 || 37 || Copyright File Identifier || strD || Filename of a file in the root directory that contains copyright information for this volume set. If not specified, all bytes should be 0x20.
|-
| 739 || 37 || Abstract File Identifier || strD || Filename of a file in the root directory that contains abstract information for this volume set. If not specified, all bytes should be 0x20.
|-
| 776 || 37 || Bibliographic File Identifier || strD || Filename of a file in the root directory that contains bibliographic information for this volume set. If not specified, all bytes should be 0x20.
|-
| 813 || 17 || Volume Creation Date and Time || dec-datetime || The date and time of when the volume was created.
|-
| 830 || 17 || Volume Modification Date and Time || dec-datetime || The date and time of when the volume was modified.
|-
| 847 || 17 || Volume Expiration Date and Time || dec-datetime || The date and time after which this volume is considered to be obsolete. If not specified, then the volume is never considered to be obsolete.
|-
| 864 || 17 || Volume Effective Date and Time || dec-datetime || The date and time after which the volume may be used. If not specified, the volume may be used immediately.
|-
| 881 || 1 || File Structure Version || int8 || The directory records and path table version (always 0x01).
|-
| 882 || 1 || Unused || - || Always 0x00.
|-
| 883 || 512 || Application Used || - || Contents not defined by ISO 9660.
|-
| 1395 || 653 || Reserved || - || Reserved by ISO.
|}

=== Volume Descriptor Set Terminator ===

The Volume Descriptor Set Terminator does not currently define bytes 7-2047 of its Volume Descriptor. This means that the only fields in use for the volume set terminator are the type code (255), the standard identifier ('CD001') and the descriptor version (0x01).

{| {{wikitable}}
! Offset
! Length (bytes)
! Field name
! Datatype
! Description
|-
| 0 || 1 || Type || int8 || 255 indicates a Volume Descriptor Set Terminator.
|-
| 1 || 5 || Identifier || strA || Always "CD001".
|-
| 6 || 1 || Version || int8 || Volume Descriptor Version (0x01).
|}

== The Path Table ==
The Path Table contains a well-ordered sequence of records describing every directory extent on the CD. There are some exceptions with this: the Path Table can only contain 65536 records, due to the length of the "Parent Directory Number" field. If there are more than this number of directories on the disc, some CD authoring software will ignore this limit and create a non-compliant CD (this applies to some earlier versions of Nero, for example). If your file system uses the path table, you should be aware of this possibility. Windows uses the Path Table and will fail with such non-compliant CD's (additional nodes exist but appear as zero-byte). Linux, which uses the directory tables is not affected by this issue.

The location of the path tables can be found in the Primary Volume Descriptor. There are two table types - the L-Path table (relevant to x86) and the M-Path table. The only difference between these two tables is that multi-byte values in the L-Table are LSB-first and the values in the M-Table are MSB-first.

The structure of a Path Table Entry is as follows:
{| {{wikitable}}
!Offset
!Size
!Description
|-
| 0 || 1 || Length of Directory Identifier
|-
| 1 || 1 || Extended Attribute Record Length
|-
| 2 || 4 || Location of Extent (LBA). This is in a different format depending on whether this is the L-Table or M-Table (see explanation above).
|-
| 6 || 2 || Directory number of parent directory (an index in to the path table). This is the field that limits the table to 65536 records.
|-
| 8 || (variable) || Directory Identifier (name) in d-characters.
|-
| (variable) || 1 || Padding Field - contains a zero if the Length of Directory Identifier field is odd, not present otherwise. This means that each table entry will always start on an even byte number.
|}

The path table is in ascending order of directory level and is alphabetically sorted within each directory level.

== Directories ==
At some point when reading from an ISO 9660 CD, you will need a directory record to locate a file, even if you generally use the path table to locate the directory initially. Unlike the path tables, there is only one version of each directory table, and multi byte numbers are in both-endian format. Every directory will start with 2 special entries: an empty string, describing the "." entry, and the string "\1" describing the ".." entry. A directory record is laid out as follows:

{| {{wikitable}}
!Offset
!Size
!Type
!Description
|-
| 0 || 1 || int8 || Length of Directory Record.
|-
| 1 || 1 || int8 || Extended Attribute Record length.
|-
| 2 || 8 || int32_LSB-MSB || Location of extent (LBA) in both-endian format.
|-
| 10 || 8 || int32_LSB_MSB || Data length (size of extent) in both-endian format.
|-
| 18 || 7 || see format below || Recording date and time.
|-
| 25 || 1 || see below || File flags.
|-
| 26 || 1 || int8 || File unit size for files recorded in interleaved mode, zero otherwise.
|-
| 27 || 1 || int8 || Interleave gap size for files recorded in interleaved mode, zero otherwise.
|-
| 28 || 4 || int16_LSB-MSB || Volume sequence number - the volume that this extent is recorded on, in 16 bit both-endian format.
|-
| 32 || 1 || int8 || Length of file identifier (file name). This terminates with a ';' character followed by the file ID number in ASCII coded decimal ('1').
|-
| 33 || (variable) || strD || File identifier.
|-
| (variable) || 1 || -- || Padding field - zero if length of file identifier is even, otherwise, this field is not present. This means that a directory entry will always start on an even byte number.
|-
| (variable) || (variable) || -- ||
System Use - 
The remaining bytes up to the maximum record size of 255 may be used
for extensions of ISO 9660. The most common one is the System Use Share
Protocol (SUSP) and its application, the Rock Ridge Interchange Protocol
(RRIP).
|}

Even if a directory spans multiple sectors, the directory entries are not permitted to cross the sector boundary (unlike the path table). Where there is not enough space to record an entire directory entry at the end of a sector, that sector is zero-padded and the next consecutive sector is used. Some of the above fields need explanation. Unfortunately, the date/time format is different from that used in the Primary Volume Descriptor. The Date/Time format is:

{| {{wikitable}}
!Offset
!Size
!Description
|-
| 0 || 1 || Number of years since 1900.
|-
| 1 || 1 || Month of the year from 1 to 12.
|-
| 2 || 1 || Day of the month from 1 to 31.
|-
| 3 || 1 || Hour of the day from 0 to 23.
|-
| 4 || 1 || Minute of the hour from 0 to 59.
|-
| 5 || 1 || Second of the minute from 0 to 59.
|-
| 6 || 1 || Offset from GMT in 15 minute intervals from -48 (West) to +52 (East).
|}

This is quite a contrast to the PVD which contains ASCII encoded decimal values, but this format is presumably used to save disc space over a large number of entries.

The other field that needs some explanation is the File Flags field. This is represented by one bit flags as follows:
{| {{wikitable}}
!Bit
!Description
|-
| 0 || If set, the existence of this file need not be made known to the user (basically a 'hidden' flag.
|-
| 1 || If set, this record describes a directory (in other words, it is a subdirectory extent).
|-
| 2 || If set, this file is an  "Associated File".
|-
| 3 || If set, the extended attribute record contains information about the format of this file.
|-
| 4 || If set, owner and group permissions are set in the extended attribute record.
|-
| 5 & 6 || Reserved
|-
| 7 || If set, this is not the final directory record for this file (for files spanning several extents, for example files over 4GiB long.
|}

== Locating Data on the CD == 
By now, you should be able to see that there are two main ways to navigate to a file record. You an either search the path table, or you can search the full directory structure. You may find it more convenient and faster to cache the path table, loading directories only when necessary.

=== Searching the Path Table ===
If you are using the Path Table method, you will still need to know about Directory Records to find the file you are looking for. Basically, you search the path in a reverse order, following the "Parent Directory" links in the Path Table. Once you have located the directory containing the file you want, load that Directory and scan it for the appropriate file name.

=== Recursing from the Root Directory ===
Alternatively, you can ignore the Path Table and just cache the root directory from the Primary Volume Descriptor. You then load each directory in turn. For example, for the path '/BOOT/MYLOADER/STAGE2.BIN'

# Read the PVD in to memory. Bytes 156-189 contain the root directory entry.
# Load the root directory by reading the LBA and Length values in this root directory entry.
# Scan the directory entry identifiers for 'BOOT;1'.
# If found, use the LBA and length values to load the 'BOOT' directory in to memory.
# Repeat steps 3 and 4 for the file identifier 'MYLOADER;1'.
# Scan the 'MYLOADER' directory for 'STAGE2.BIN;1'. If found, you can now use the LBA value to load your file in to memory.

== Rock Ridge and Joliet ==
There are two enhancements for ISO 9660 which make it more suitable for
the worlds of Unix and of MS-Windows. Both can be combined in the same
filesystem. So the reader often has the choice between three file name
spaces: Plain ISO, Rock Ridge, Joliet.

ISO and Rock Ridge will show the same tree of files but with different names.
Joliet can show a completely different tree than ISO.

Rock Ridge allows for file names of up to 255 characters of 8 bit. Only
the 0-byte and the slash ("/") may not be used. Further it adds the
file attributes which are specified by POSIX (owner, group, permissions,...)
and it allows for symbolic links.

Rock Ridge is an application of SUSP. It may be accompanied by other
SUSP applications like zisofs (compression of data files, Linux
specific), Apple ISO 9660 Extensions, Amiga AS entries, or Arbitrary
Attribute Interchange Protocol (AAIP: Extended Attributes and ACLs).
A reader of SUSP entries shall simply ignore all entry types
which it does not expect.

Joliet was defined by Microsoft Inc. to allow for filenames with
up to 64 UCS-2 characters (16 bit). It is implemented as separate
tree of Directory Records which begins by a root record in a
Supplementary Volume Descriptor. That descriptor is similar to a
Primary Volume Descriptor, but has a Type Code of 2.

== See Also ==
=== Articles ===
* [[El-Torito]], a standard for creating bootable CD-ROMs
* [[Mkisofs]], about ISO 9660 producing programs: mkisofs, genisoimage, xorriso
* [[Optical Drive]], an overview about how to operate optical drives and media

=== External links ===
* [https://www.ecma-international.org/wp-content/uploads/ECMA-119_2nd_edition_december_1987.pdf ISO 9660 (ECMA-119) specification]
* [[wikipedia:ISO 9660|ISO 9960 on Wikipedia]]
* [https://dev.lovelyhq.com/libburnia/libisofs/raw/master/doc/boot_sectors.txt  Boot entry points in ISO 9660 filesystems]
* [ftp://ftp.ymi.com/pub/rockridge/susp112.ps SUSP 1.12 (entries CE , PD , SP , ST , ER , ES)]
* [ftp://ftp.ymi.com/pub/rockridge/rrip112.ps Rock Ridge: RRIP 1.12 (SUSP entries PX , PN , SL , NM , CL , PL , RE , TF , SF , obsolete: RR)]
* [http://www.estamos.de/makecd/Rock_Ridge_Amiga_Specific Amiga SUSP entry AS]
* [https://dev.lovelyhq.com/libburnia/libisofs/raw/master/doc/susp_aaip_2_0.txt libisofs SUSP application AAIP (SUSP entry AL)]
* [http://www.buildorbuy.org/pdf/joliet.pdf Joliet addon-on specifications]

[[de:ISO9660]]
[[Category:Filesystems]]
````

<a id="w002212"></a>

## W002212 — SFS

Bron-ID: W002212
Namespace: Main (0)
Revisie: 26882 | Datum: 2022-01-24T01:49:00Z
Bron: https://wiki.osdev.org/SFS
Revisie-URL: https://wiki.osdev.org/index.php?title=SFS&oldid=26882

````text
{{Filesystems}}

There are several filesystems existing that are abbreviated '''SFS''', but the one discussed here is the '''Simple File System'''  by Brendan Trotter, designed to be very easy to implement. It has a minimal set of features. Officially it is still in its draft stage, but hasn't changed for some time.

SFS is designed to be used as a simple filesystem for use on floppy disks and other removable media to transfer data between computers.

== Format Details ==
=== Layout ===
There are five key areas in a SimpleFS partition: The Super-block, Reserved Area, Data Area, Free Area and Index Area.
=== Super-block ===
The Super-block is located in the first sector of a disk or partition. For floppy disks that means it is contained in the boot sector. The superblock contains structural information about the partition. The SFS data of the super-block is located at offset 0x0194 and is 58 bytes long, leaving space for boot code and a partition table.
{| class="wikitable"
|-
! Offset
! Size
! Contents
|-
| 0x0194
| 8
| Last alteration time of super-block values (Data size or Index Size)
|-
| 0x019C
| 8
| Size of Data Area in blocks
|-
| 0x01A4
| 8
| Size of Index Area in <u>bytes</u>
|-
| 0x01AC
| 3
| Magic number (0x534653 'SFS')
|-
| 0x01AF
| 1
| Version number in BCD (0x10 = 1.0)
|-
| 0x01B0
| 8
| Total number of blocks in volume
|-
| 0x01B8
| 4
| Size of Reserved Area and Super-block in blocks
|-
| 0x01BC
| 1
| Block size given by ''BytesPerBlock = 2<sup>(BlockSize + 7)</sup>''
|-
| 0x01BD
| 1
| Checksum of bytes between 0x1AC and 0x1BC such that the lower 8 bits are zero
|}

=== Reserved Area ===
The reserved area is a segment of disk that is untouched by drivers conforming to the current specification. This allows ancillary information or data such as a kernel image or a small EXT2/FAT partition to be stored on the same volume.

=== Data Area ===
The Data Area contains file data and starts at the end of the Reserved Area. The files in the data area can only occupy sequential areas as there is no way of linking chains of blocks for a single file.

=== Free Area ===
The Free Area is the currently unused space in the volume that can be used by either the Data Area growing downwards or the Index Area growing upwards.

=== Index Area ===
''Note: Because the Index Area is situated at the end of the disk the offsets are referred to in an opposite fashion to what is typical. So, the first entry is actually the closest to the end of the disk''

The Index Area contains the tracking information for the Data Area and is made of a variable number of 64 byte entries extending from the end of the volume towards the beginning.
The entries in the Index Area follow a few simple rules.
The "first" (at the end of the disk) entry must be a Volume Identifier entry, this is used to detect if a disk has been swapped when auto-detection is unavailable. The "last" (closest to the start of the disk) must be a Starting Marker entry, this is used for file-system recovery if the super-block is damaged.
Continuation entries are used to allow longer file names than can be placed in a single directory or file entry and immediately precede their respective file/directory entry.

=== Embedding ===
The GUID currently in use for GPT partitions is 4ebf0e06-11bf-450c-1a06-534653534653

== Information Formatting ==
=== Timestamps ===
All SFS timestamps are 64-bit signed values that count the number of 1/65536<sup>ths</sup> of a second before or after the UNIX Epoch (1<sup>st</sup> of January 1970) and are in UTC.
=== Names ===
Name Strings in SFS are in UTF-8 and must be null terminated. Characters can include any character in the ranges 0x20-0x7E and 0xA1-0xFF excluding a double quote ("), asterisk (*), colon (:), less than or greater then sign (< or >), question mark (?) or backward slash (\). The forwards slash (/) is admissible only as a path separator. 
Note that UTF-8 means that the characters above 0x80 take up multiple bytes rather than one, which could confuse ASCII based systems.

== See Also ==
=== Threads ===
* [[Topic:11833|SFS Review thread]]
=== External Links ===
* [https://www.d-rift.nl/combuster/vdisk/sfs.html Official Specification] ([https://web.archive.org/web/20090923015653/http://dimensionalrift.homelinux.net/combuster/vdisk/sfs.html web.archive.org])
* [https://www.d-rift.nl/combuster/vdisk/ A windows utility for SFS images] ([https://web.archive.org/web/20111111113756/http://dimensionalrift.homelinux.net/combuster/vdisk/ web.archive.org])
* [https://www.fysnet.net/blog/files/sfs.pdf Ben Lunt's specification] (updated 13 Jan 2022) (not 100% compatible with the original, has more features) (version 1.10)

[[Category:Filesystems]]
[[de:SFS]]
````

<a id="w002213"></a>

## W002213 — What filesystem should I use

Bron-ID: W002213
Namespace: Main (0)
Revisie: 5034 | Datum: 2008-01-03T13:27:27Z
Bron: https://wiki.osdev.org/What_filesystem_should_I_use
Revisie-URL: https://wiki.osdev.org/index.php?title=What_filesystem_should_I_use&oldid=5034
Doorverwijzing: File Systems

````text
#REDIRECT [[File Systems#File systems for OSDevers]]
[[Category:FAQ]]
````

<a id="w002214"></a>

## W002214 — User talk:Chase

Bron-ID: W002214
Namespace: User talk (3)
Revisie: 29165 | Datum: 2024-06-28T19:38:52Z
Bron: https://wiki.osdev.org/User_talk:Chase
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AChase&oldid=29165

````text

````

<a id="w002215"></a>

## W002215 — Category:Human Interface Device

Bron-ID: W002215
Namespace: Category (14)
Revisie: 5064 | Datum: 2008-01-28T04:43:09Z
Bron: https://wiki.osdev.org/Category:Human_Interface_Device
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AHuman_Interface_Device&oldid=5064

````text
[[Category:Hardware]]
````

<a id="w002216"></a>

## W002216 — User:Assembler

Bron-ID: W002216
Namespace: User (2)
Revisie: 24548 | Datum: 2020-04-04T04:54:27Z
Bron: https://wiki.osdev.org/User:Assembler
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAssembler&oldid=24548

````text
== Who am I ? ==
'''Name:''' Aslan<br>
'''Degree:''' PhD in Electrial and Computer Engineering<br>
'''Job:''' Postdoctoral Research Associate<br>
'''WWW:''' http://www.sce.carleton.ca/~maslan<br>
'''Projects:''' http://www.sce.carleton.ca/~maslan/projects.html<br>
'''Favorite Quote:''' "Do you pine for the nice days of Minix-1.1, when men were men and wrote their own device drivers?" -- Linus Torvalds<br>
== About Me ==
I am currently a postdoc at the department of systems and computer engineering at Carleton university.<br>
I've been involved in operating system development since year 1999, where I wrote a boot loader, and a simple 8086 real mode kernel.<br>
Later, I used to work on [http://git.savannah.gnu.org/cgit/osdk.git LibOSDK] during my free time,<br>
Nowadays, I'm very interested in the [http://www.openbsd.org OpenBSD] kernel development,<br>
I've also been attracted to the virtualization technology specially paravirtualization and the [http://www.xen.org Xen] project.<br/>
In addition, I like to read about the [http://www.l4ka.org L4] kernel design and API.<br>
````

<a id="w002217"></a>

## W002217 — SILO

Bron-ID: W002217
Namespace: Main (0)
Revisie: 20389 | Datum: 2017-02-11T20:51:26Z
Bron: https://wiki.osdev.org/SILO
Revisie-URL: https://wiki.osdev.org/index.php?title=SILO&oldid=20389

````text
== What is SILO ==
SILO stands for "Sparc Improved boot LOader", it is a Linux [[bootloader]] on [[SPARC]] and [[UltraSPARC]] platforms.
SILO can load Linux, Solaris from [[ext2]], [[ext3]], [[FFS_(BSD)]], [[ISO_9660]] and ROMFS

SILO is capable of booting uncompressed or gzipped kernel images from the filesystems it supports. 

== SILO usage ==
When SILO is loaded at the boot prompt the path to the image to be booted can be passed manually or via an alias from the silo.conf file. Pressing tab lists aliases in the silo.conf file. Typing "halt" sends you back to the boot prom. Pressing "enter" or "return" boots the defualt image. 

== Installation ==
To create a bootable cdrom image for [[QEMU]] or real hardware. Note that second.b (the secondary bootblock) and silo.conf need to be edited to reflect your target filesystem and should be in the boot directory of the target filesystem.
<pre>mkisofs -G /boot/isofs.b -B ... -r -o IMAGENAME DIRECTORY_TREE</pre>

For floppy drive images.
The following should be done with silo on an actual [[SPARC]] or emulated machine. Another option is to try and get intelsilo to work. 
[[genromfs]] should be downloaded and installed if you have not already done so.

<pre>
genromfs -d floppy/ -f /dev/ram -A 2048,/.. -a 512 -V 'My installation disk'
mount -t romfs /dev/ram /mnt
silo -r /mnt -F -i /boot/fd.b -b /boot/second.b -C /etc/silo.conf
umount /mnt
dd if=/dev/ram of=floppy.img bs=1440k count=1
</pre>

==Caveats==
SILO can only load images within the first 1GB of a disk. This means that partitions and or files beyond 1GB are not bootable.

== Links ==
* [http://lfs-matrix.com/pub/clfs/clfs-packages/svn/silo-1.4.14.tar.bz2 SILO Source code mirrored by CLFS] 
* [http://tldp.org/HOWTO/SPARC-HOWTO-17.html http://tldp.org/HOWTO/SPARC-HOWTO-17.html]
* [http://www.gentoo.org/doc/en/handbook/handbook-sparc.xml?part=1&chap=10 http://www.gentoo.org/doc/en/handbook/handbook-sparc.xml?part=1&chap=10]
* [[Sparc Overview]]

[[Category:Bootloaders]]
````

<a id="w002218"></a>

## W002218 — User:Pietro10

Bron-ID: W002218
Namespace: User (2)
Revisie: 5154 | Datum: 2008-02-25T23:05:04Z
Bron: https://wiki.osdev.org/User:Pietro10
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APietro10&oldid=5154

````text
Hi. I'm Pietro Gagliardi, and I am working on two operating systems.
* One, which I'm writing myself, is going nowhere fast (ha). There's quite a bit of code from a tutorial that I want to change to handwritten code. I don't know how fast that will be. Then, I want to actually get multitasking <i>working</i>!
* An operating system called MMURTL, which many people don't recommend, is my second project. I intend to rewrite the Assembly parts and make them into C. That'll wait a while.
````

<a id="w002220"></a>

## W002220 — Talk:System Calls

Bron-ID: W002220
Namespace: Talk (1)
Revisie: 29764 | Datum: 2025-11-16T13:49:32Z
Bron: https://wiki.osdev.org/Talk:System_Calls
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ASystem_Calls&oldid=29764

````text
== Syscall/Sysret ==

I noticed this on the page:
"A similar instruction pair has been created by AMD: Syscall/Sysret. However the behaviour of these instructions are different from Intel's."

This is not entirely correct.  AMD and Intel both have Sysenter and Sysexit.  In long mode on an AMD 64 bit processor, you must use Syscall/Sysret, but in legacy mode (not compatibility) either will work.  I haven't looked up how this works on an intel machine.  I assume it's similar for long mode as the MSRs required to configure Sysenter/Sysexit only leave room for 32 bit addresses.

--[[User:Speal|Speal]] 09:11, 20 February 2008 (CST)

:I fail to see why that argument makes that line of text on the page incorrect. Intel introduced sysenter/sysexit. AMD introduced syscall/sysret. Both share the goal of getting in and out of kernel mode as fast as possible. Only the way it is done differs. That both may or may not be implemented on one processor isn't at all the point that's being made, no? - [[User:Combuster|Combuster]] 16:24, 20 February 2008 (CST)

::The text seems to suggest that AMD doesn't have sysenter/sysexit, which is not true.  The distinction is important since sysenter/sysexit don't work in long mode.  I'd be willing to draft an addition for the syscall/sysret use in long mode in the next few days. --[[User:Speal|Speal]] 20:34, 20 February 2008 (CST)


Are you going to include argument passing methods? Such as using the stack, or registers, etc etc
--[[User:Earlz|Earlz]] 14:02, 9 April 2009 (UTC)

Note about read() vs write() being used to stop kernel structures: read() provides a userspace controlled pointer to which the kernel writes data, write() provides a userspace controlled pointer from which the kernel reads data. Therefore a pointer check failure on write() can not be used to overwrite kernel data, but a pointer check failure on read() can.

[[User:Dormito|Dormito]] 16:39, 16 February 2021 (CST)

== "The most common way to implement system calls is using a software interrupt." ==

This might have been true when it was written, but most architectures seems to provide dedicated instructions now:

* sysenter/sysexit in x86 protected mode
* syscall/sysret in x86 long mode
* ecall in RISC-V
* syscall in MIPS

ARM still uses software interrupts, apparently (swi).

[[User:Demindiro|Demindiro]] ([[User talk:Demindiro|talk]]) 13:49, 16 November 2025 (UTC)
````

<a id="w002222"></a>

## W002222 — Category:IDEs

Bron-ID: W002222
Namespace: Category (14)
Revisie: 5169 | Datum: 2008-02-28T18:10:14Z
Bron: https://wiki.osdev.org/Category:IDEs
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AIDEs&oldid=5169

````text
A list of IDEs that can be used for OS Development

[[Category:Tools]]
````

<a id="w002224"></a>

## W002224 — User talk:Abuashraf

Bron-ID: W002224
Namespace: User talk (3)
Revisie: 5171 | Datum: 2008-02-28T18:17:27Z
Bron: https://wiki.osdev.org/User_talk:Abuashraf
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AAbuashraf&oldid=5171

````text
==Deleting pages, and editing in general==
'''PLEASE''', the Delete option is is only for content that is no longer needed. It shouldn't '''EVER''' be used because something doesnt work and you want to rewrite a page.

Also, please read the [[Manual Of Style]], and look at other articles on how to make things look the way they should. If you're unsure about things you should not bulldoze over everything but instead ask on the forum.

I cleaned up what could be cleaned up. The page histories however are gone, which is a Bad Thing.

:- [[User:Combuster|Combuster]] 12:17, 28 February 2008 (CST)
````

<a id="w002225"></a>

## W002225 — SMBIOS

Bron-ID: W002225
Namespace: Main (0)
Revisie: 5210 | Datum: 2008-03-06T22:00:57Z
Bron: https://wiki.osdev.org/SMBIOS
Revisie-URL: https://wiki.osdev.org/index.php?title=SMBIOS&oldid=5210
Doorverwijzing: System Management BIOS

````text
#REDIRECT [[System Management BIOS]]
````

<a id="w002226"></a>

## W002226 — System Management BIOS

Bron-ID: W002226
Namespace: Main (0)
Revisie: 30972 | Datum: 2026-09-26T18:03:52Z
Bron: https://wiki.osdev.org/System_Management_BIOS
Revisie-URL: https://wiki.osdev.org/index.php?title=System_Management_BIOS&oldid=30972

````text
SMBIOS (System Management [[BIOS]]) is a standard developed by [https://www.dmtf.org DMTF] whose purpose is to deliver information about the hardware, by relying on the system firmware. Once booted, SMBIOS will let the OS access a table that contains a series of entries with various hardware information, separated in different structures of variable length.

It was first designed for Intel, and released in 1995. Nowadays it supports most desktop architectures, and its last specification was published the 21 of July, 2023.

== Entry Point Structure ==

SMBIOS provides a searchable (or querryable) structure called Entry Point Structure (or EPS) that contains a pointer to the SMBIOS Structure Table and some additional information, like its length, version, or number of structures.

=== Format ===

{| align="center" cellspacing="2"
| width="50%" valign="top" |

{| {{wikitable}}
|+ 32-bit Entry Point Structure Format
! Offset
! Name
! Size
|-
| 0x00 || Anchor String || 4 BYTEs
|-
| 0x04 || Entry Point Checksum || BYTE
|-
| 0x05 || Entry Point Lenght || BYTE
|-
| 0x06 || SMBIOS Major Version || BYTE
|-
| 0x07 || SMBIOS Minor Version || BYTE
|-
| 0x08 || SMBIOS Structure Maximum Size || WORD
|-
| 0x0A || Entry Point Revision || BYTE
|-
| 0x0B || Formatted Area || 5 BYTEs
|-
| 0x10 || Intermediate Anchor String || 5 BYTEs
|-
| 0x15 || Intermediate Checksum || BYTE
|-
| 0x16 || Structure Table Length || WORD
|-
| 0x18 || '''Structure Table Address''' || DWORD
|-
| 0x1C || Number of Structures || WORD
|-
| 0x1E || BCD Revision || BYTE
|}

| width="10%" valign="top" |

| width="50%" valign="top" |

{| {{wikitable}}
|+ 64-bit Entry Point Structure Format
! Offset
! Name
! Size
|-
| 0x00 || Anchor String || 5 BYTEs
|-
| 0x05 || Entry Point Checksum || BYTE
|-
| 0x06 || Entry Point Lenght || BYTE
|-
| 0x07 || SMBIOS Major Version || BYTE
|-
| 0x08 || SMBIOS Minor Version || BYTE
|-
| 0x09 || SMBIOS Docrev || BYTE
|-
| 0x0A || Entry Point Revision || BYTE
|-
| 0x0B || Reserved || BYTE
|-
| 0x0C || Structure Table Maximum Size || DWORD
|-
| 0x10 || '''Structure Table Address''' || QWORD
|}

|}

=== Locating the entry point structure ===

==== Non-UEFI systems ====

Under systems without UEFI, the Entry Point Structure is located somewhere in physical memory from address 0x000F0000 to 0x00FFFFF, with a 16 byte alignment.

* In 32-bit architectures, the SMBIOS EPS first contains (with offset 0) a string with the value "_SM_". This is what will help us search the location of the EPS.

* In 64-bit architectures, as in 32-bit, it first contains a string, but with the value "_SM3_" (The 3 stands for SMBIOS 3, which is the version in 64-bit machines).

This example C code searches the entry point in a 64-bit architecture. All it does is iterate for every 16-byte aligned memory addresses (from 0x000F0000 to 0x000FFFFF) and comparing 4 bytes from that address with the string "_SM3_". To acomplish that, it is needed to verify the checksum value (add all bytes and see if the lowest 8 bits of the result are zero):

<source lang="c">

    /* Start address */
    char *eps = 0x000F0000;

    int length, i;
    uint8_t checksum = 0;

    while (eps <= (char*) 0x000FFFFF) {

        /* Check Anchor String (64-bit version) */
        if (!memcmp(eps, "_SM3_", 5)) {

            length = eps[5];
            checksum = 0;

            /* Add all bytes */
            for (i = 0; i < length; i++)
                checksum += eps[i];

            if (checksum == 0)
                /* Done! */
                break;
        }

        /* Next 16-byte-aligned address */
        eps += 16;
    }

</source>

Now, ''eps'' contains the address of the Entry Point Structure. Some systems may not have the SMBIOS, so an error check may be a good idea:

<source lang="c">
    if ((unsigned int) eps == 0x00100000) {
        /* Error, SMBIOS could not be located */
        return -1;
    }
</source>

==== UEFI systems ====

On UEFI systems, the search-for-a-string method is not used to obtain the EPS. Instead, it is located by looking in the EFI Configuration Table for the SMBIOS Version 3 GUID (SMBIOS3_TABLE_GUID), which will contain a pointer to the structure.

This C code shows how that could get implemented using [[GNU-EFI]]:
<source lang="c">

    /* Will contain the address of the Entry Point Structure */
    void *SMBIOS_Pointer = NULL;
    UINTN status = LibGetSystemConfigurationTable(&SMBIOS3TableGuid, (void **)(&SMBIOS_Pointer));

    /* Check all posible errors (maybe is not needed?) */
    if (status != EFI_SUCCESS || SMBIOS_Pointer == NULL ||
        CompareMem(SMBIOS_Pointer, "_SM3_", 5) ) {

        /* Error, SMBIOS could not be located */
        return -1;

    }

</source>

== Structure Table ==

Once the EPS is located in memory, information about the SMBIOS table can already be obtained. The most important one is the '''Structure Table Adress''', which is the address of the table that contains all the SMBIOS structures. The structures are located directly adjacent to each other in memory, with a new structure beginning as soon as another one ends. Each structure is composed of a header, a structure specific table, and a string table of variable length. 

The first SMBIOS header is located at the '''Structure Table Adress'''. The value of ''type'' indicates what element the structure contains information about. ''length'' indicates the size of '''header + data table'''. The strings are not included in the length. 

<source lang="c">
struct SMBIOSHeader {
    uint8_t type;
    uint8_t length;
    uint16_t handle;
};
</source>

Immediately after the end of the header, is the data table. At the end of the data table (header address + length), the strings section starts. Each string is NULL terminated and is limited to 64 characters. Strings are referenced within tables by using an index into the string table (index 0 means that the string is effectively a NULL pointer and should be ignored). The first string begins immediately after the data, and the second string begins immediately after that, and so on. The string section itself is terminated by two consecutive zero bytes.

So, the end (and therefore the length) of the SMBIOS structure can be calculated by finding the two [[Null Character]]s in the string section. Your code might look like:
<source lang="c">
size_t smbios_struct_len(struct SMBIOSHeader *hd)
{
    size_t i;
    const char *strtab = (char *)hd + hd->len;
    // Scan until we find a double zero byte
    for (i = 1; strtab[i - 1] != '\0' || strtab[i] != '\0'; i++)
        ;
    return hd->len + i + 1;
}
</source>

The final table is denoted by a ''type'' field of value 127.

As an example, the BIOS Struct (Type 0) might look like this:

<source lang="asm">
 db 0 ; Indicates BIOS Structure Type            |
 db 13h ; Length of information in bytes         | HEADER
 dw ? ; Reserved for handle                      |

 db 01h ; String 1 is the Vendor Name            |
 db 02h ; String 2 is the BIOS version           |
 dw 0E800h ; BIOS Starting Address               |
 db 03h ; String 3 is the BIOS Build Date        | DATA
 db 1 ; Size of BIOS ROM is 128K (64K * (1 + 1)) |
 dq BIOS_Char ; BIOS Characteristics             |
 db 0 ; BIOS Characteristics Extension Byte 1    |

 db ‘System BIOS Vendor Name’,0 ;                |
 db ‘4.04’,0 ;                                   | STRINGS
 db ‘00/00/0000’,0 ;                             |

 db 0 ; End of structure
</source>

== Header Types ==
{| class="wikitable"
|-
! Code
! Description
|-
| 0
| BIOS Information
|-
| 1
| System Information
|-
| 2
| Mainboard Information
|-
| 3
| Enclosure/Chasis Information
|-
| 4
| Processor Information
|-
| 7
| Cache Information
|-
| 9
| System Slots Information
|-
| 16
| Physical Memory Array
|-
| 17
| Memory Device Information
|-
| 19
| Memory Array Mapped Address
|-
| 20
| Memory Device Mapped Address (optional as of SMBIOS 2.5)
|-
| 32
| System Boot Information
|}

More details can be found in the Specification (see External Links)


==See Also==

===Articles===
* [[ACPI]]
* [[DMI]]

===Forum Threads===
* http://www.osdev.org/phpBB2/viewtopic.php?t=16687 Info (saved here through lack of info)

===External Links===
* [http://www.dmtf.org/standards/smbios/ SMBIOS Specification]

[[Category:X86]]
[[Category:Standards]]
[[Category:Hardware Detection]]
````

<a id="w002227"></a>

## W002227 — Talk:Continuation Systems

Bron-ID: W002227
Namespace: Talk (1)
Revisie: 5176 | Datum: 2008-03-05T01:23:24Z
Bron: https://wiki.osdev.org/Talk:Continuation_Systems
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AContinuation_Systems&oldid=5176

````text
While not exactly identical, this article describes a system very much like Scheduler Activations.  These are described in [http://www.cs.washington.edu/homes/tom/pubs/sched_act.pdf this paper].  This system has actually been implemented, while continuation system have not (to the best of my knowledge).  I think the paper, while it is 27 pages long, gives a clearer picture of a very similar system.  There is also a quantitative discussion of the benefits of this system - something only possible if it's actually been implemented.

If anyone has actually implemented continuations, perhaps you could post some benchmarks.  Otherwise it seems like we should include this as a subsection on a Scheduler Activations page.
--[[User:Speal|Speal]] 19:23, 4 March 2008 (CST)
````

<a id="w002228"></a>

## W002228 — Talk:Synchronization Primitives

Bron-ID: W002228
Namespace: Talk (1)
Revisie: 5601 | Datum: 2008-06-04T19:53:14Z
Bron: https://wiki.osdev.org/Talk:Synchronization_Primitives
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ASynchronization_Primitives&oldid=5601

````text
This page looks like it's actually about lock-based synchronization.  There is also a Semaphore page which gives details on binary and counting semaphores.  I think we should change this page's title to Lock-Based Synchronization.
--[[User:Speal|Speal]] 19:26, 4 March 2008 (CST)
````

<a id="w002229"></a>

## W002229 — Long Mode

Bron-ID: W002229
Namespace: Main (0)
Revisie: 5180 | Datum: 2008-03-05T01:55:25Z
Bron: https://wiki.osdev.org/Long_Mode
Revisie-URL: https://wiki.osdev.org/index.php?title=Long_Mode&oldid=5180
Doorverwijzing: X86-64

````text
#REDIRECT [[x86-64]]
````

<a id="w002230"></a>

## W002230 — User:Speal

Bron-ID: W002230
Namespace: User (2)
Revisie: 5181 | Datum: 2008-03-05T02:05:52Z
Bron: https://wiki.osdev.org/User:Speal
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASpeal&oldid=5181

````text
I'm a college student studying Computer Science at the University of Minnesota.

I am currently working on a 64 bit (x86) microkernel operating system in [http://www.digitalmars.com/d/ D]
You can look at the source, download an ISO, and follow the development at [http://www.devlime.com my website].
````

<a id="w002231"></a>

## W002231 — Bochs VBE Extensions

Bron-ID: W002231
Namespace: Main (0)
Revisie: 30571 | Datum: 2026-05-12T15:39:18Z
Bron: https://wiki.osdev.org/Bochs_VBE_Extensions
Revisie-URL: https://wiki.osdev.org/index.php?title=Bochs_VBE_Extensions&oldid=30571

````text
The [[Bochs]] VGA BIOS supports, to an extent, the [[VBE]] specification. Since Bochs only emulates a VGA card down to the hardware level (and a Cirrus graphics card if you enable it, but that is not tied in with the Bochs VBE extensions), it emulates very simple graphics hardware that the VBE BIOS can drive. The advantage of this is that if you are running your OS in Bochs (or [[QEMU]], which uses the Bochs VGA BIOS, or even [[VirtualBox]]), you can use this emulated hardware to directly set video modes without using VBE (which would require real mode or v86).

== Overview ==
The Bochs emulated graphics hardware (henceforth called BGA for Bochs Graphics Adaptor) is accessed via two 16-bit IO-ports. The first one is an index port, the second one a data port (comparable to how the VGA handles its sets of registers). Via these ports it is possible to enable or disable the VBE extensions, change the screen resolution and bit depth, and manage a larger virtual screen. There are six versions of the BGA (0xB0C0 through 0xB0C5), but if you use the latest version of Bochs you only need to concern yourself with the latest one (0xB0C5). QEMU (with the <tt>-std-vga</tt> command line argument) also uses the latest version.

The Bochs sources define in vga.h, located in the subdirectory iodev/, a number of defines that are useful for programming the BGA. The names of these defines all start with VBE_DISPI. They are used in the sections below, with their numerical value between parentheses.

== BGA versions ==
As Bochs has evolved, so has the BGA. Six versions of the BGA have existed, of which 0xB0C5 is the current version (as of 2009, Bochs version 2.4). The main features of each version:
* 0xB0C0 - setting X and Y resolution and bit depth (8 BPP only), banked mode
* 0xB0C1 - virtual width and height, X and Y offset
* 0xB0C2 - 15, 16, 24 and 32 BPP modes, support for linear frame buffer, support for retaining memory contents on mode switching
* 0xB0C3 - support for getting capabilities, support for using 8 bit DAC
* 0xB0C4 - VRAM increased to 8 MB
* 0xB0C5 - [http://marc.info/?l=bochs-cvs&m=123287460501516 VRAM increased to 16 MB?] [TODO: verify and check for other changes]
[TODO: if and when 4BPP modes are supported]
[TODO: if and when VGA attribute controller (AC) is supported]

== Programming the BGA ==
=== Writing registers ===
To write an index/data pair to one of the BGA registers, first write its index value to the 16-bit IO-port VBE_DISPI_IOPORT_INDEX (0x01CE), followed by writing the data value to the 16-bit IO-port VBE_DISPI_IOPORT_DATA (0x01CF). The BGA supports 10 different index values (0 through 9):
* VBE_DISPI_INDEX_ID (0)
* VBE_DISPI_INDEX_XRES (1)
* VBE_DISPI_INDEX_YRES (2)
* VBE_DISPI_INDEX_BPP (3)
* VBE_DISPI_INDEX_ENABLE (4)
* VBE_DISPI_INDEX_BANK (5)
* VBE_DISPI_INDEX_VIRT_WIDTH (6)
* VBE_DISPI_INDEX_VIRT_HEIGHT (7)
* VBE_DISPI_INDEX_X_OFFSET (8)
* VBE_DISPI_INDEX_Y_OFFSET (9)

In order to change the contents of registers 1-3 (VBE_DISPI_INDEX_XRES, VBE_DISPI_INDEX_YRES, VBE_DISPI_INDEX_BPP) the VBE extensions must be disabled first. To do so, write the value VBE_DISPI_DISABLED (0x00) to VBE_DISPI_INDEX_ENABLE (4). The changes are not visible until the VBE extensions are enabled again. To do so, write the value VBE_DISPI_ENABLED (0x01) to the same register (see also note below on enabling the LFB).

=== Reading registers ===
To read a register, first write the index value to VBE_DISPI_IOPORT_INDEX (0x01CE), then read the 16-bit value from VBE_DISPI_IOPORT_DATA (0x01CF). The value returned depends on the specific register that is queried. [TODO: check and describe, especially _ID and _ENABLE, others seem trivial]

=== Checking availability ===
To check whether the BGA is available, read the value from VBE_DISPI_INDEX_ID (0). If it equals VBE_DISPI_ID5 (0xB0C5) the latest version of the BGA is present. If it returns a value of 0xB0C0 through 0xB0C3, you have an old version of Bochs and/or the Bochs VGA BIOS.

If for some reason you want Bochs to emulate an older version of the BGA, you can write the desired version to VBE_DISPI_INDEX_ID (0). If succesful, reading the register again will return the value just set. This is used by the Bochs VGA BIOS to ensure it is run with the right version of Bochs. If done from an application (or your OS), this will break compatibility with the Bochs VBE BIOS, which expects the latest version.

=== Setting display resolution and bit depth ===
Most likely, setting the display resolution and bit depth is all you need. To do so, disable the VBE extensions (see above), write the X resolution, Y resolution and BPP to their respective registers (VBE_DISPI_INDEX_XRES (1), VBE_DISPI_INDEX_YRES (2) and VBE_DISPI_INDEX_BPP (3)) and enable the VBE extensions. Since the BGA is not real hardware, X and Y resolutions can be set at will up to a maximum horizontal resolution (depending on your version of Bochs) of either 1024 or 1600 (VBE_DISPI_MAX_XRES) and a maximum vertical resolution of either 768 or 1200 (VBE_DISPI_MAX_YRES). It seems that both Bochs and QEMU can use about any resolutions with a few restrictions. In QEMU the X resolution must be divisible by 8. [TODO: Describe how Bochs handle different X resolutions. It can crash with segmentation fault on various X resolutions, so it is difficult to test]. The Y resolution can be any number from 1 to maximum in both Bochs and QEMU. The bit depth needs to be one of the following:
* VBE_DISPI_BPP_4 (0x04)
* VBE_DISPI_BPP_8 (0x08)
* VBE_DISPI_BPP_15 (0x0F)
* VBE_DISPI_BPP_16 (0x10)
* VBE_DISPI_BPP_24 (0x18)
* VBE_DISPI_BPP_32 (0x20)

If you try to set an invalid resolution (greater than the maximum or an irregular X resolution) or bit depth, usually nothing happens. The current video mode is preserved. In some cases Bochs will crash with a segmentation fault if you set an invalid X resolution, so only use standard ones. [TODO: Check latest CVS version of Bochs and search their bug tracker]. You can always read the current resolution and bit depth from their respective registers. When you have set a resolution it is a good idea to read it back and check whether your new resolution is really set. You should do that before you enable the BGA to minimize the chance of your emulator crashing. [TODO: Check what happens when the first resolution you set is invalid.]

=== Memory layout of video modes ===

In all modes the first byte represents the top left of the screen. A formula to calculate the video memory offset from a pixel coordinate is: <tt>offset = (Y * X-resolution + X) * <some-factor></tt>. The factor varies using the different bit depths. The 4 and 8 BPP modes are palette modes. Read more about programming the attribute controller and DAC palettes in [[:Category:VGA|VGA documents]]. The other modes are using the colour value directly.

In 4 BPP you have 16 colours. The pixel colour is used an index to the attribute controller, which again points to the DAC, which in turn gives an 18 bit colour (6 bits each for red, green and blue) that is shown. The memory is layout like this: The first byte is the colour bit 0 of pixels 0-7. The second byte is the colour bit 1 of pixels 0-7, and so forth. You can access 8 pixels at a time using longwords (32 bits).

[[Image:Linear video memory layout 4bpp.png]]

In 8 BPP you have 256 colours. The pixel colour is sent to the DAC, which gives an 18 bit colour that is shown. The memory layout is very simple. Each pixel is exactly one byte.

[[Image:Linear video memory layout 8bpp.png]]

In 15 BPP each pixel is easiest accessed as words (16 bits). There are 5 bits each colour component and the last bit is ignored.

[[Image:Linear video memory layout 15bpp.png]]

In 16 BPP each pixel is easiest accessed as words (16 bits). There are 5 bits for the red and blue components and 6 bits for green. This makes sense, because the human eye is more sensitive to green colours.

[[Image:Linear video memory layout 16bpp.png]]

In 24 BPP each pixel is 3 bytes. There is one byte for each component. The colour components is blue first, then green, then red.

[[Image:Linear video memory layout 24bpp.png]]

In 32 BPP each pixel is 4 bytes and easiest accessed as longwords (32 bits). The fourth byte is ignored. The colour components is layout like in 24 BPP. Accessing pixels as longwords the colour should be defined as 0x00RRGGBB.

[[Image:Linear video memory layout 32bpp.png]]

=== Using banked mode ===
When using banked mode, the BGA uses a 64Kb bank size (VBE_DISPI_BANK_SIZE_KB) starting at address 0xA0000 (VBE_DISPI_BANK_ADDRESS). Banked mode is the default mode, so when enabling the VBE extensions without explicitly telling the BGA to use a linear frame buffer, the BGA enables banked mode. To set the bank to use, write the bank number to the bank register (VBE_DISPI_INDEX_BANK (5)). <!--[TODO: check whether Bochs does a sanity check, or wrap-around, or whatever and describe]-->

=== Using a linear frame buffer (LFB) ===
When using a linear framebuffer, the BGA exposes all of the video memory in a single linearly addressable section of memory. The address of the framebuffer is not fixed, and must be read from the first [[PCI]] base address register (BAR 0 of device 0x1234:0x1111). To enable the linear framebuffer, use the VBE_DISPI_LFB_ENABLED flag (0x40) when enabling the BGA in conjunction with the VBE_DISPI_ENABLED flag. <!--[TODO: check behaviour for 4 BPP modes]-->

Unlike Bochs, QEMU does not necessarily pay attention to the VBE_DISPI_LFB_ENABLED flag with respect to banked memory access, allowing both the linear framebuffer and banked memory to be used at all times. Bochs will not honour requests to change the memory bank when the linear framebuffer is enabled, and it will similarly ignore any writes made to the memory bank.

'''Note:''' In older versions of Bochs and QEMU, the framebuffer was fixed at 0xE0000000, and modern versions will use that address when emulating ISA-only systems. '''It is highly inadvisable to make assumptions about the address of the linear framebuffer.''' It should always be read from the BGA's PCI BAR0.

=== Clearing display memory ===
When enabling the VBE extensions, Bochs clears the video memory (i.e. sets all bytes to 0). To prevent this from happening, use the VBE_DISPI_NOCLEARMEM flag (0x80) when enabling the VBE extensions (so write a value of VBE_DISPI_ENABLED | VBE_DISPI_NOCLEARMEM (0x81) for banked mode and VBE_DISPI_ENABLED | VBE_DISPI_LFB_ENABLED | VBE_DISPI_NOCLEARMEM (0xC1) for LFB).

=== Finding out capabilities ===

Based on source code examination for Bochs (iodev/vga.cc) setting VBE_DISPI_GETCAPS in VBE_DISPI_INDEX_ENABLE makes the VBE_DISPI_INDEX_ (XRES / YRES / BPP) fields return their maximum values when read instead of the current values.

=== 8-bit DAC ===

The default palette DAC is a 3x6 bit dac; it returns a value between 0 and 63 for each color. Setting the VBE_DISPI_8BIT_DAC bit in VBE_DISPI_INDEX_ENABLE changes this to a 3x8 bit dac and converts the palette to the appropriate values. Resetting the bit moves them back again.

=== Virtual display ===
The Bochs adapter allows a virtual display that is larger than the physical one. The video memory is normally your screen width wide and at least as high as your vertical screen size, the height. The actual video memory is larger and the remainder is seen as a vertical extension.

Having this as a horizontal extension can be very useful. You can use this to make horizontal and vertical scrolling effects very cheap. You can also make a virtual display, double buffering and lots of other ideas.

The mechanism is used that the memory is a virtual display starting at location (0,0) with a size specified by the virtual width. The height is implicitly as large as it can be within the video memory. Suppose you set the virtual width to be 1024 in a 32-bpp mode (4 bytes per pixel) on a 16-meg Bochs card. That makes the vertical height 4096.

You can then specify where the video card should start reading the memory using the X and Y offset. 

The variables taking care of this:
* VBE_DISPI_INDEX_VIRT_WIDTH is the virtual width.
* VBE_DISPI_INDEX_VIRT_HEIGHT is the virtual height, currently not implemented. Reasoning is above.
* VBE_DISPI_INDEX_X_OFFSET is the X offset for displaying.
* VBE_DISPI_INDEX_Y_OFFSET is the Y offset for displaying.

=== Example code ===

<syntaxhighlight lang="c">
void BgaWriteRegister(unsigned short IndexValue, unsigned short DataValue)
{
    outpw(VBE_DISPI_IOPORT_INDEX, IndexValue);
    outpw(VBE_DISPI_IOPORT_DATA, DataValue);
}

unsigned short BgaReadRegister(unsigned short IndexValue)
{
    outpw(VBE_DISPI_IOPORT_INDEX, IndexValue);
    return inpw(VBE_DISPI_IOPORT_DATA);
}

int BgaIsAvailable(void)
{
    return (BgaReadRegister(VBE_DISPI_INDEX_ID) == VBE_DISPI_ID4);
}

void BgaSetVideoMode(unsigned int Width, unsigned int Height, unsigned int BitDepth, int UseLinearFrameBuffer, int ClearVideoMemory)
{
    BgaWriteRegister(VBE_DISPI_INDEX_ENABLE, VBE_DISPI_DISABLED);
    BgaWriteRegister(VBE_DISPI_INDEX_XRES, Width);
    BgaWriteRegister(VBE_DISPI_INDEX_YRES, Height);
    BgaWriteRegister(VBE_DISPI_INDEX_BPP, BitDepth);
    BgaWriteRegister(VBE_DISPI_INDEX_ENABLE, VBE_DISPI_ENABLED |
        (UseLinearFrameBuffer ? VBE_DISPI_LFB_ENABLED : 0) |
        (ClearVideoMemory ? 0 : VBE_DISPI_NOCLEARMEM));
}

void BgaSetBank(unsigned short BankNumber)
{
    BgaWriteRegister(VBE_DISPI_INDEX_BANK, BankNumber);
}
</syntaxhighlight>

== See Also ==
=== External Links ===
* [http://cvs.savannah.nongnu.org/viewvc/*checkout*/vgabios/vgabios/vbe_display_api.txt Specification]
* [https://raw.githubusercontent.com/bochs-emu/VGABIOS/refs/heads/master/vgabios/vbe_display_api.txt Alternate specification link]
* [https://github.com/bochs-emu/VGABIOS Bochs VGABIOS Repository]

[[Category:Video]]
[[Category:VGA]]
````

<a id="w002232"></a>

## W002232 — Talk:Bochs VBE Extensions

Bron-ID: W002232
Namespace: Talk (1)
Revisie: 29711 | Datum: 2025-10-15T16:57:07Z
Bron: https://wiki.osdev.org/Talk:Bochs_VBE_Extensions
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABochs_VBE_Extensions&oldid=29711

````text
Ok, since this topic keeps coming up on the forum, I decided to make a Wiki page for the subject. There's still a lot of TODOs, but mainly on how Bochs treats out of bound values, and some miscellaneous stuff that's probably not used that much. [[User:Jal|Jal]] 05:44, 5 March 2008 (CST)

: I made some additions with the knowledge I gained during my experiments with BGA in both Bochs and QEMU. I hope you like it. Actually in some cases they behave very differently even if they implement the same protocol. I'm still making some experiments, so at some point I might add more. --[[User:Walling|Walling]] 06:40, 26 October 2008 (UTC)

I tried to follow this document. It seems there might be a problem with the section [[Bochs_VBE_extensions#Using_a_linear_frame_buffer_.28LFB.29|Using a linear frame buffer]]. With version 2.4.1 of bochs, I do an Output Word on the Index IO Port with VBE_DISPI_INDEX_ENABLE and then Output Word on the data port VBE_DISPI_ENABLED | VBE_DISPI_LFB_ENABLED. In the document, it specifies that VBE_DISPI_INDEX_ENABLE | VBE_DISPI_LFB_ENABLED should be sent to the index and then VBE_DISPI_ENABLED on the data port. Can someone confirm this? --[[User:Threesevenths|Threesevenths]] 16:59, 6 August 2009 (UTC)

:You are right, that was a stupid typo. It should read VBE_DISP_ENABLED instead of VBE_DISPO_INDEX_ENABLE. Thanks for noticing it (you can see the code example does it right). [[User:Jal|Jal]] 09:17, 21 October 2009 (UTC)

== Writing 0x20 to VGA_ATT_W ==

For a custom QEMU BIOS I found that it is necessary to write the value 0x20 to the VGA_ATT_W register (<code>0x3c0</code>, or the first register in the [https://www.qemu.org/docs/master/specs/standard-vga.html#mmio-area-spec VGA section of the MMIO region]). I don't understand what it does however, as I just copied it from EDK2. Perhaps this article could clarify what it does and why it is necessary?

I've skimmed [[VGA Hardware]] but it does not seem to contain information about this particular value.

[[User:Demindiro|Demindiro]] ([[User talk:Demindiro|talk]]) 16:57, 15 October 2025 (UTC)
````

<a id="w002234"></a>

## W002234 — File:VGA gc write3.gif

Bron-ID: W002234
Namespace: File (6)
Revisie: 5224 | Datum: 2008-03-12T15:12:59Z
Bron: https://wiki.osdev.org/File:VGA_gc_write3.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AVGA_gc_write3.gif&oldid=5224

````text
VGA Write mode 3 schematics
````

<a id="w002236"></a>

## W002236 — Category:Sound

Bron-ID: W002236
Namespace: Category (14)
Revisie: 5234 | Datum: 2008-03-22T22:38:31Z
Bron: https://wiki.osdev.org/Category:Sound
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ASound&oldid=5234

````text
This category includes articles about sound cards; the different varieties, how to program them and articles concerning driver development and implementation.
{{main|Sound}}

[[Category:Hardware]]
````

<a id="w002237"></a>

## W002237 — Sound Blaster 16

Bron-ID: W002237
Namespace: Main (0)
Revisie: 30642 | Datum: 2026-05-27T15:09:52Z
Bron: https://wiki.osdev.org/Sound_Blaster_16
Revisie-URL: https://wiki.osdev.org/index.php?title=Sound_Blaster_16&oldid=30642

````text
{{Historical}}

The Sound Blaster series is a family of sound cards made by Creative. For many years they were the standard audio cards on IBM PC's.

==Introduction==
The Sound Blaster 16 is the successor to the Sound Blaster Pro audio card found in many older computers. It was first brought out in 1992. The main improvement over the Pro is that it supports 16-bit digital audio sampling (hence the name). Most SB16 cards are [[PCI]] compatible, as Creative made this improvement shortly after releasing the card, upgrading from [[ISA]].

==Digital Signal Processor==
The DSP built into the Sound Blaster 16 supports playing and recording audio in 8-bit and 16-bit PCM encoded samples, along with playing several other formats (ADPCM, etc.).  The base I/O register address can be found using the [[PCI]] bus for PCI models, or by detecting the presence of an older [[ISA]] Sound Blaster by issuing a Get Version command to one of several common I/O port addresses (0x220, 0x240, etc.) and waiting for a response.

{| class="wikitable"
! Port
! Description
|-
| 0x224 || DSP Mixer port
|-
| 0x225 || DSP Mixer data port
|-
| 0x226 || DSP Reset
|-
| 0x22A || DSP Read
|-
| 0x22C || DSP Write
|-
| 0x22E || DSP Read Status (Read this port to acknowledge 8-bit interrupt)
|-
| 0x22F || DSP 16-bit Interrupt Acknowledge (Read this port to acknowledge 16-bit interrupt)  (DSP Version 4.0+ Only)
|-
|}

{| class="wikitable"
! Command for DSP Write
! Description
! Output/input for command
|-
| 0x40 || Set time constant || 8 bit value
|-
| 0x41 || Set Output Sample Rate || high bit/low bit
|-
| 0xD1 || Turn speaker on ||
|-
| 0xD3 || Turn speaker off ||
|-
| 0xD0 || Stop playing 8 bit channel ||
|-
| 0xD4 || Resume playback of 8 bit channel ||
|-
| 0xD5 || Stop playing 16 bit channel ||
|-
| 0xD6 || Resume playback of 16 bit channel ||
|-
| 0xE1 || Get DSP version || major version/minor version
|-
|}

{| class="wikitable"
! Command for Mixer port
! Description
! Mixer data port
|-
| 0x22 || Master volume || 0xLR L=left volume R=right volume min=0x0 max=0xF (default value is 0xCC or 0x11)
|-
| 0x80 || Set IRQ || See below
|-
|}

There are two modes for transferring data. First is single mode - data from buffer are played, interrupt is fired and playing is stopped. It make lower quality of sound. Second is auto mode - data from buffer are playing forever and interrupt is fired after play buffer. You should use interrupt for re-filling buffer.

==Operations==

===Reseting DSP (detecting DSP)===
# Send 1 to DSP reset port
# Wait 3 microseconds
# Send 0 to DSP reset port
# Now should be in DSP read port 0xAA.
Some SB-compatible cards returned 0xAA on first check but on second check they reported their specific version.

===Setting IRQ===
# Send 0x80 to Mixer port
# Send value of your IRQ to Mixer data port
0x01=IRQ 2 0x02=IRQ 5 0x04=IRQ 7 0x08=IRQ 10 Usually is used IRQ 5. You can read this port too for get actual IRQ.

===Programming DMA===
You can get more info about programming DMA on page [http://homepages.cae.wisc.edu/~brodskye/sb16doc/sb16doc.html] .

Programming 8 bit transfers through channel 1:
# Disable channel by writing to port 0x0A value 0x05 (channel number + 0x04)
# Write value to flip-flop port 0x0C (any value e.g. 1)
# Send transfer mode to 0x0B (0x48 for single mode/0x58 for auto mode + channel number)
# Send page number to 0x83(page port of channel 1) For example if you have sound data at 0x100450, page is 0x10.
# Send low bits of position to port 0x02(addr. port of channel 1) For example(see above) is 0x50.
# Send high bits of position to port 0x02(addr. port of channel 1) For example(see above) is 0x04.
# Send low bits of length of data to port 0x03(count port of channel 1) For example if is length 0x0FFF, send 0xFF
# Send high bits of length of data to port 0x03(count port of channel 1) For example if is length 0x0FFF, send 0x0F
# Enable channel by writing channel number to port 0x0A

Programming 16 bit transfers through channel 5 ('''channel number''' is still 1):
# Disable channel by writing to port 0xD4 value 0x05 (channel number + 0x04)
# Write value to flip-flop port 0xD8 (any value e.g. 1)
# Send transfer mode to 0xD6 (0x48 for single mode/0x58 for auto mode + channel number)
# Send page number to 0x8B(page port of channel 5) For example if you have sound data at 0x100450, page is 0x10.
# Send low bits of position to port 0xC4(addr. port of channel 5) For example(see above) is 0x50.
# Send high bits of position to port 0xC4(pos. port of channel 5) For example(see above) is 0x04.
# Send low bits of length of data to port 0xC6(count port of channel 5) For example if is length 0x0FFF, send 0xFF
# Send high bits of length of data to port 0xC6(count port of channel 5) For example if is length 0x0FFF, send 0x0F
# Enable channel by writing channel number to port 0xD4

===Writing transfer mode to DSP===

Usually values are 0xB0 for 16 bit playing sound or 0xC0 for 8 bit playing sound.

{| class="wikitable"
! Bit 7-4
! Bit 3
! Bit 2
! Bit 1
! Bit 0
|-
| 0xB=16 bit transfer 0xC=8 bit transfer || 0=playing sound 1=recording sound || 0 || 0=FIFO off 1=FIFO on || 0
|-
|}

===Writing type of sound data to DSP===

You must write type of sound data after write transfer mode.

{| class="wikitable"
! Bit 7
! Bit 6
! Bit 5
! Bit 4
! Bit 3
! Bit 2
! Bit 1
! Bit 0
|-
| 0 || 0 || 0=mono 1=stereo || 0=unsigned 1=signed || 0 || 0 || 0 || 0
|-
|}

==Playing sound==
# Reset DSP
# Load sound data to memory
# Set master volume
# Turn speaker on
# Program ISA DMA to transfer
# Set time constant. Notice that the Sound Blaster 16 is able to use sample rates instead of time constants using command 0x41 instead of 0x40. <br> You can calculate the time constant like this: Time constant = 65536 - (256000000 / (channels * sampling rate))
# Set output sample rate
# Write transfer mode to DSP
# Write type of sound data
# Write data length to DSP(Low byte/High byte) (You must calculate LENGTH-1 e.g. if is your real length 0x0FFF, you must send 0xFE and 0x0F)

==Code==
<syntaxhighlight lang="nasm">
  %macro OUTB 2
    mov dx, %1
    mov al, %2
    out dx, al
  %endmacro

  %macro INB 1
    mov dx, %1
    in al, dx
  %endmacro


  ;SOUND BLASTER 16 driver in real mode

  ;reset sound blaster
  OUTB 0x226, 1 ;reset port
  mov ah, 86h
  mov cx, 0x0000
  mov dx, 0xFFFF
  int 15h ;wait
  OUTB 0x226, 0 ;reset port

  ;turn speaker on
  OUTB 0x22C, 0xD1

  ;DMA channel 1
  OUTB 0x0A, 5 ;disable channel 1 (number of channel + 0x04)
  OUTB 0x0C, 1 ;flip flop
  OUTB 0x0B, 0x49 ;transfer mode
  OUTB 0x83, 0x01 ;PAGE TRANSFER (EXAMPLE POSITION IN MEMORY 0x[01]0F04) - SET THIS VALUE FOR YOU
  OUTB 0x02, 0x04 ;POSITION LOW BIT (EXAMPLE POSITION IN MEMORY 0x010F[04]) - SET THIS VALUE FOR YOU
  OUTB 0x02, 0x0F ;POSITON HIGH BIT (EXAMPLE POSITION IN MEMORY 0x01[0F]04) - SET THIS VALUE FOR YOU
  OUTB 0x03, 0xFF ;COUNT LOW BIT (EXAMPLE 0x0FFF) - SET THIS VALUE FOR YOU
  OUTB 0x03, 0x0F ;COUNT HIGH BIT (EXAMPLE 0x0FFF) - SET THIS VALUE FOR YOU
  OUTB 0x0A, 1 ;enable channel 1

  ;program sound blaster 16
  OUTB 0x22C, 0x40 ;set time constant
  OUTB 0x22C, 165 ;10989 Hz
  OUTB 0x22C, 0xC0 ;8 bit sound
  OUTB 0x22C, 0x00 ;mono and unsigned sound data
  OUTB 0x22C, 0xFE ;COUNT LOW BIT - COUNT LENGTH-1 (EXAMPLE 0x0FFF SO 0x0FFE) - SET THIS VALUE FOR YOU
  OUTB 0x22C, 0x0F ;COUNT HIGH BIT - COUNT LENGTH-1 (EXAMPLE 0x0FFF SO 0x0FFE) - SET THIS VALUE FOR YOU

  ;now transfer start - don't forget to handle irq
</syntaxhighlight>

==QEMU support==
[[QEMU]] is one of the few hypervisors/emulators that support this sound card.
To run QEMU with Sound Blaster 16 emulation, use the <code>-soundhw sb16</code> option.

'''Warning''': recent versions of QEMU (>= 4.0) have a broken support for
this sound card. See bug: [https://bugs.launchpad.net/qemu/+bug/1873769].
Briefly, when QEMU's GTK UI is used and audio is playing, you'll experience
the QEMU window freezing. In addition, there will be flickering in the audio
as well.

===Workarounds===

# Use an older version of QEMU. With QEMU 2.11 the problem simply does not exist.
# Use [https://virt-manager.org virt-manager] which connects to QEMU using Spice. The problem does not exist in this case because QEMU's GTK UI is not used. BUT, configuring QEMU to emulate Sound Blaster 16 through virt-manager requires some tricky settings:
## Add any sound card to the VM
## In virt-manager's Edit -> Preferences menu check the "Enable XML editing" box.
## Open VM's hardware and after selecting the sound card, click on the XML tab and replace its contents with:<syntaxhighlight lang="xml"><sound model="sb16"/></syntaxhighlight>
# Run the OS using QEMU's <code>-curses</code> option or use QEMU's <code>-kernel</code> and <code>-nographic</code> (serial console): in this case, there will be no "freeze", but there still be some flickering in the audio.

==See Also==
===External Links===
* http://www.qzx.com/pc-gpe/
* https://web.archive.org/web/20190613202127/http://homepages.cae.wisc.edu/~brodskye/sb16doc/sb16doc.html
* [http://ftp.lanet.lv/ftp/mirror/x2ftp/msdos/programming/mxinfo/ Some documents at the bottom (.zip)]

[[Category:Sound]]
````

<a id="w002238"></a>

## W002238 — Sound

Bron-ID: W002238
Namespace: Main (0)
Revisie: 28960 | Datum: 2024-06-06T17:17:53Z
Bron: https://wiki.osdev.org/Sound
Revisie-URL: https://wiki.osdev.org/index.php?title=Sound&oldid=28960

````text
{{In Progress}}

[[Image:Old_Sound_Card.jpg|right|frame|An example of an older-style PCI sound card]]

Sound is never noticed by users, until it isn't there. It forms an integral part of the user feedback experience. For example, the beeps produced by Windows Explorer, KDE and other desktop environments when users click on the wrong thing. It is also becoming a far more dominant part of the gaming and multimedia experience; with many games and movies now featuring 5.1 channel soundtracks. As such, it is becoming more important for hobby operating systems to support sound playback, at the very least.

==A Short History of PC Sound==
The original sound device on PCs was the [[PC Speaker]], available as far back as the IBM PC (introduced in 1981). In the late 1980s several manufacturers started producing add-in sound devices, notably Creative (the Game Blaster) and AdLib (the AdLib Music Synthesizer Card). Of these, Creative was far more successful, and the [[Sound Blaster 16]] became a de-facto standard for more than half a decade. Almost all cards today still have some backwards compatibility with the Sound Blaster 16.

In 1997 Intel specified a new standard, [[AC97]] (short for ''Audio Codec 1997''), that virtually replaced the Sound Blaster standard by specifying higher quality sampling. In 2004 Intel produced yet another standard, this time the [[Intel High Definition Audio]] standard (codenamed ''Azalia''), which specifies yet another improvement on previous standards' audio quality.

Sound cards and standards for the professional (and audiophile) markets have evolved separately from the mainstream cards. The important feature in these cards is not fancy features, but very low latency (<5ms) and high quality sampling (24 bit sampling at 96KHz not being uncommon). [[MIDI]] is an important part of professional audio today, and has been since the early 1990s. Cards can be from 2 channels through to 32 and more.

==Programming for Sound==
Programming sound support into your operating system is not simply a case of writing a driver for a fairly generic card (such as the [[Sound Blaster 16]]) and letting your userspace applications deal with it; it is a case of putting together an architecture that will support sound generation, transport (to and from different devices, applications and even computers), mixing, output, and control, all the while retaining low latencies and high quality. In this, there are several key problems:
* Your sound transport architecture should be lossless, that is, 32 bit samples should be carried at 32 bits. Carrying these samples at 16 bits (for example) will lead to sound degradation, and is to be avoided.
* The same transport mechanism should also provide support for varied sample rates. 44100Hz is the standard for CDs, but DVDs carry sound at a sample rate of 48000Hz. Treating these two as the same will lead to very strange noises being emitted from the speakers. In addition, these will need conversion to a common sample rate before being output.
* Two applications '''will''' need access to the sound devices simultaneously. For example, notifications from an Instant Messaging program as well as the user's MP3 player. The operating system has the job of mixing these sound streams together and producing an intelligible result. 
* Today's audio is not stereo, it can be any number of channels. Simpler applications still use mono sound, while DVD and High definition movies have 5.1 and 7.1 channel sound tracks. Your operating system will need to be able to route sound to any of these channels, and decide where to place mono and stereo sounds in the larger channel set. In addition, [[MIDI]] audio also needs to be routed correctly to and from devices and applications.
* Sound, like gaming graphics, must be completed in real time. Skipping audio caused by random hard disk or CPU usage is ''very'' noticeable. Unfortunately, simply increasing buffer size is not always the best solution, as this causes people listening to freshly-recorded audio ('monitoring' in the industry speak) to suffer delays.
Some (but not all) sound cards will have features allowing some of this processing to be done in hardware. Simple mixing and sample processing are the most common things to have dedicated hardware.

Some applications, such as games, will try to use the maximum number of channels possible. Older versions of Windows allowed applications to have 'exclusive' control over the sound hardware for low latency sound with guaranteed channels. However, since Windows Vista all sound (including volume) is software mixed supporting an unlimited number of channels.

Software mixing all sounds has several advantages for very little CPU overhead:
* Easier to develop sound card drivers - all they must do is stream a buffer to the device rather than worry about channels, channel effects, channel volume, and what-not.
* Provides a consistent environment across hardware - a game will know it will not run out of channels and all channel effects (echo, reverb, etc) will be supported.

===The Sound Architecture===
{{In Progress}}
====The Simple Approach====
[[Image:Really_Simple_Audio_Architecture.png]]

This is the simplest of all architectures: a single application dealing with a single device through a single device driver. This would work well with a UNIX-based philosophy: read(), write() and ioctl() all have immediate and obvious effects. The application can read a number of samples from the sound device, process them (perhaps recording them to disk, adding a reverb, etc), and then write the resulting sound samples back out onto the device's speaker port.

====The Ring/Hub Approach====
A more complex example of an audio architecture is based around a ring concept. The ring is responsible for transport of audio signals around the system, and each module plugged on to the ring acts on the signal in some way.

Each application, input or output, mixer, filter, or plugin of some sort could be represented on the ring, and audio travels around being modified by some or all of the other ring modules. This has been successfully implemented in hardware on Creative's line of X-Fi sound cards; see [http://www.pcper.com/article.php?aid=177&type=expert&pid=2 here] for more detail.

In a kernel this need not be implemented strictly as a ring: a hub architecture achieves roughly the same effect in a different manner.

'''ToDo: Describe additional methods, research and flesh out these ones more. See what is being used in today's kernels (information on how ALSA and co work is sketchy without reading the source).'''

==Resources==
* [[wikipedia:Digital audio|An overview of digital audio]]
* [[wikipedia:MIDI|An overview of MIDI]]
* [http://www.alsa-project.org/ The ALSA project] has documentation that may be useful in designing a sound system, as well as GPLed drivers for many different sound cards.

[[Category:Sound]]
````

<a id="w002239"></a>

## W002239 — ATA PIO Mode

Bron-ID: W002239
Namespace: Main (0)
Revisie: 27609 | Datum: 2023-01-31T21:30:32Z
Bron: https://wiki.osdev.org/ATA_PIO_Mode
Revisie-URL: https://wiki.osdev.org/index.php?title=ATA_PIO_Mode&oldid=27609

````text
According to the ATA specs, PIO mode must always be supported by all ATA-compliant drives as the default data transfer mechanism.

PIO mode uses a tremendous amount of CPU resources because every byte of data transferred between the disk and the CPU must be sent through the CPU's [[Inline Assembly/Examples#I\O access|IO port bus]] (not the memory). On some CPUs, PIO mode can still achieve actual transfer speeds of 16MB per sec, but no other processes on the machine will get any CPU time.

However, when a computer is just beginning to boot, there are no other processes. So PIO mode is an excellent and simple interface to utilize during bootup until the system goes into multitasking mode.

Please note that this article deals with what are now styled PATA hard disks, as opposed to SATA hard disks.



==Hardware==

The ATA disk specification is built around an older specification called ST506. With ST506, each disk drive was connected to a controller board by two cables -- a data cable, and a command cable. The controller board was plugged into a motherboard bus. The CPU communicated with the controller board through the CPU's IO ports, which were directly connected to the motherboard bus.

What the original IDE specification did was to detach the disk controller boards from the motherboard, and stick one controller onto each disk drive, permanently. When the CPU accessed a disk IO port, there was a chip that shorted the CPU's IO bus pins directly onto the IDE cable -- so the CPU could directly access the drive's controller board. The data transfer mechanism between the CPU and the controller board remained the same, and is now called PIO mode.
(Nowadays, the disk controller chips just copy the electrical signals between the IO port bus and the IDE cable, until the drive goes into some other mode than PIO.)



==Master/Slave Drives==

There is only one wire dedicated to selecting which drive on each bus is active. It is either electrically "high" or "low", which means that there can never be more than two devices operational on any ATA bus. They are called the master and the slave devices, for no particular reason. The terms 'master' and 'slave' have largely been abandoned as they inaccurately portray the master drive as having some kind of superiority over the slave drive, or that the latter is dependent on the master. However, these terms will be used in this document. The functionality of the master and slave drives is almost completely identical. There is a special IO port bit that allows a driver to select either drive as the target drive for each command byte.

==Primary/Secondary Bus==

Current disk controller chips almost always support two ATA buses per chip. There is a standardized set of IO ports to control the disks on the buses. The first two buses are called the Primary and Secondary ATA bus, and are almost always controlled by IO ports 0x1F0 through 0x1F7, and 0x170 through 0x177, respectively (unless you change it). The associated Device Control Registers/Alternate Status ports are IO ports 0x3F6, and 0x376, respectively. The standard IRQ for the Primary bus is IRQ14 and IRQ15 for the Secondary bus.

If the next two buses exist, they are normally controlled by IO ports 0x1E8 through 0x1EF, and 0x168 through 0x16F, respectively. The associated Device Control Registers/Alternate Status ports are IO ports 0x3E6, and 0x366.

The actual control registers and IRQs for each bus can often be determined by enumerating the PCI bus, finding all the disk controllers, and reading the information from each controller's PCI Configuration Space. So, technically, PCI enumeration should be done before ATA device detection. However, this method is not exactly reliable.

When the system boots, according to the specs, the PCI disk controller is supposed to be in "Legacy/Compatibility" mode. This means it is supposed to use the standardized IO port settings. You may have no real choice but to rely on that fact.



==400ns delays==

The method suggested in the ATA specs for sending ATA commands tells you to check the BSY and DRQ bits before trying to send a command. This means that you need to read a Status Register (Alternate Status is a good choice) <i>for the proper drive</i> before sending the next command. Which means that you need to select the correct device <i>first</i>, before you can read that status (and then send all the other values to the other IO ports).
Which means that a drive select may always happen <i>just before</i> a status read. This is bad. Many drives require a little time to respond to a "select", and push their status onto the bus. The suggestion is to read the Status register <b>FIFTEEN TIMES</b>, and only pay attention to the value returned by the last one -- after selecting a new master or slave device. The point being that you can assume an IO port read takes at least 30ns, so doing the first fourteen creates a 420ns delay -- which allows the drive time to push the correct voltages onto the bus.

Reading IO ports to create delays wastes a lot of CPU cycles. So, it is actually smarter to have your driver remember the last value sent to each Drive Select IO port, to avoid doing unneeded drive selections, if the value did not change. If you do not send a drive select, then you only have to read the Status Register <b>once</b>.

Alternately, you never want to send new commands to a drive that is already servicing a previous command, anyway. Your driver <b>always</b> needs to block if the current device is actively modifying BSY/DRQ/ERR, and your device driver always already knows that the device is in that condition (because the driver just sent the command to the device, and it hasn't been marked "complete" yet). Once a drive has actually completed a command, it will always clear BSY and DRQ. You can simply verify this, <i>before</i> your next Device Select command -- that the previously selected device cleared BSY and DRQ properly at command completion. Then you will never have to check if they are clear <b>after</b> a Device Select -- so you will not have to read the Status Register after the Device Select at all.

There is a similar problem after writing the Command Register, with the ERR/DF bits. They are two slightly different kinds of errors that can terminate a command. BSY and DRQ will be cleared, but ERR or DF remain set <i>until just after you write a new command to the Command Register</i>. If you are using polling (see below), you should account for the fact that your first four reads of the Status Register, after sending your command byte, may have the ERR or DF bits still set accidentally. (If you are using IRQs, the Status will always be correct by the time the IRQ is serviced.)



==Cache Flush==

On some drives it is necessary to "manually" flush the hardware write cache after every write command. This is done by sending the 0xE7 command to the Command Register (then waiting for BSY to clear). If a driver does not do this, then subsequent write commands can fail invisibly, or "temporary bad sectors" can be created on your disk.



==Bad Sectors==

For practical purposes, there are three different types of bad sectors on an ATA disk.
* Sectors that can't be written (permanent)
* Sectors that can't be read (permanent)
* Sectors that can't be read (temporary)

Some disk manufacturers have a feature that allows a small supply of "spare" sectors on the disk to be remapped onto permanent bad sectors. However, that feature is non-standard and completely manufacturer-specific. In general, an OS/filesystem will need to keep a "bad sector list" for each partition of each drive, and work around the bad sectors.

As said above, there are also "temporary bad sectors". When you read them you will get a hardware error, just like for a permanently bad sector. If you write to that sector, however, the write will work perfectly and the sector will turn back into a good sector. Temporary bad sectors can happen as a result of unflushed write caches, power spikes, or power failures.



==Detection and Initialization==

===Floating Bus===

The disk that was selected last (by the BIOS, during boot) is supposed to maintain control of the electrical values on each IDE bus. If there is no disk connected to the bus at all, then the electrical values on the bus will all go "high" (to +5 volts). A computer will read this as an 0xFF byte -- this is a condition called a "floating" bus. This is an excellent way to find out if there are no drives on a bus. '''Before sending any data to the IO ports,''' read the Regular Status byte. The value 0xFF is an illegal status value, and indicates that the bus has no drives. The reason to read the port before writing anything is that the act of writing can easily cause the voltages of the wires to go screwy for a millisecond (since there may be nothing attached to the wires to control the voltages!), and mess up any attempt to measure "float".

Measuring "float" is a shortcut for detecting that drives do not exist. Reading a non-0xFF value is not completely definitive. The definitive test for detecting drives is the [[#IDENTIFY command]].



===Detecting Controller IO Ports===

Detecting controller IO ports is probably a waste of time. During boot, the IO ports assigned to the ATA bus are supposed to be located at standardized addresses. If they are not there, your only chance of finding them is to enumerate the disk controllers on the PCI bus. Alternately, if the ports <i>are</i> at the standard addresses, then "detecting" them gains you nothing. However, you will see controller detection code around, and it is a good thing to be able to recognize it, and know what it is for. If you suspect you know the location of a set of ATA controller IO ports, and there is at least one drive attached to that bus, then you can "detect" those IO ports. ATA controller IO ports are mostly read/write ports. This means that if you write a value to (for example) the "SectorCount" IO port, then you are supposed to be able to read the same value back again, to see what it is set to. This read/write function is performed by the master drive on the bus, unless the slave drive both exists and is selected. One caveat is that if you write a value to a "non-existent" IO port, you may be able to read that value back off the bus, if you read it immediately. So, generally, the way ATA IO port detection software works is to write a byte to one suspected ATA IO port, write a different byte to a different ATA IO port, then read back and verify the two values written to the two ports. If both bytes verify, then the IO ports are read/write ports, and they can be presumed to be ATA controller IO ports. On the Primary bus, ports 0x1F2 through 0x1F5 should all be read/write.



===Standard and Non-standard Detection===

All current BIOSes have standardized the use of the IDENTIFY command to detect the existence of all types of ATA bus devices ... PATA, PATAPI, SATAPI, SATA.

There are two other nonstandard techniques that are not recommended. The first is to select a device (then do a 400ns delay) then read the device's Status Register. For ATA devices that are not "sleeping", the RDY bit will always be set. This should be detectable, just so long as you have already tested for float (where <i>all</i> the bits are always set). If there is no device, then the Status value will be 0. This method does not work for detecting ATAPI devices -- their RDY bit is always <b>clear</b> (until they get their first PACKET command).

The other method is to use the Execute Device Diagnostics command (0x90). It supposedly sets bits in the Error Register (0x1F1 on the Primary bus) to show the existence of master and slave devices on the bus.



===IDENTIFY command===

To use the IDENTIFY command, select a target drive by sending 0xA0 for the master drive, or 0xB0 for the slave, to the "drive select" IO port. On the Primary bus, this would be port 0x1F6. Then set the Sectorcount, LBAlo, LBAmid, and LBAhi IO ports to 0 (port 0x1F2 to 0x1F5). Then send the IDENTIFY command (0xEC) to the Command IO port (0x1F7).
Then read the Status port (0x1F7) again. If the value read is 0, the drive does not exist. For any other value: poll the Status port (0x1F7) until bit 7 (BSY, value = 0x80) clears. Because of some ATAPI drives that do not follow spec, at this point you need to check the LBAmid and LBAhi ports (0x1F4 and 0x1F5) to see if they are non-zero. If so, the drive is not ATA, and you should stop polling. Otherwise, continue polling one of the Status ports until bit 3 (DRQ, value = 8) sets, or until bit 0 (ERR, value = 1) sets.

At that point, if ERR is clear, the data is ready to read from the Data port (0x1F0). Read 256 16-bit values, and store them.



===="Command Aborted"====

ATAPI or SATA devices are supposed to respond to an ATA IDENTIFY command by immediately reporting an error in the Status Register, rather than setting BSY, then DRQ, then sending 256 16 bit values of PIO data. These devices will also write specific values to the IO ports, that can be read. Seeing ATAPI specific values on those ports after an IDENTIFY is definitive proof that the device is ATAPI -- on the Primary bus, IO port 0x1F4 will read as 0x14, and IO port 0x1F5 will read as 0xEB. If a normal ATA drive should ever happen to abort an IDENTIFY command, the values in those two ports will be 0. A SATA device will report 0x3c, and 0xc3 instead. See below for a code example.

'''However''', at least a few real ATAPI drives do not set the ERR flag after aborting an ATA IDENTIFY command. So do not depend completely on the ERR flag after an IDENTIFY.



====Interesting information returned by IDENTIFY====

* uint16_t 0: is useful if the device is not a hard disk.

* uint16_t 83: Bit 10 is set if the drive supports LBA48 mode.

* uint16_t 88: The bits in the low byte tell you the supported UDMA modes, the upper byte tells you which UDMA mode is active. If the active mode is         not the highest supported mode, you may want to figure out why. ''Notes: The returned uint16_t should look like this in binary: 0000001 00000001. Each bit corresponds to a single mode. E.g. if the decimal number is 257, that means that only UDMA mode 1 is supported and running (the binary number above) if the binary number is 515, the binary looks like this, 00000010 00000011, that means that UDMA modes 1 and 2 are supported, and 2 is running.   This is true for every mode. If it does not look like that, e.g 00000001 00000011, as stated above, you may want to find out why. The formula for finding out the decimal number is 257 * 2 ^ position + 2 ^position - 1.''

* uint16_t 93 from a master drive on the bus: Bit 11 is supposed to be set if the drive detects an 80 conductor cable. ''Notes: if the bit is set then 80 conductor cable is present and UDMA modes > 2 can be used; if bit is clear then there may or may not be an 80 conductor cable and UDMA modes > 2 shouldn't be used but might work fine. Because this bit is "master device only", if there is a slave device and no master there is no way information about cable type (and would have to assume UDMA modes > 2 can't be used).''

* uint16_t 60 & 61 taken as a uint32_t contain the total number of 28 bit LBA addressable sectors on the drive. (If non-zero, the drive supports LBA28.)

* uint16_t 100 through 103 taken as a uint64_t contain the total number of 48 bit addressable sectors on the drive. (Probably also proof that LBA48 is supported.)

==Addressing Modes==

Currently there are three addressing modes to select particular sectors to read or write on a disk. They are 28 bit LBA, 48 bit LBA, and CHS. CHS mode is obsolete, but is discussed quickly below. The number of bits in the LBA modes refer to the number of significant bits in the sector "address", called an LBA. In 28 bit mode, LBAs from 0 to 0x0FFFFFFF are legal. This gives a total of 256M sectors, or 128GB of addressable space. So 28 bit LBA mode is also obsolete for many current drives. However, 28 bit PIO mode is faster than 48 bit addressing, so it may be a better choice for drives or partitions that do not violate the maximum LBA value limitation.



===Absolute/Relative LBA===

All the ATA commands that use LBA addressing require "absolute" LBAs (ie. the sector offset from the very beginning of the disk -- completely ignoring [[Partition Table|partition]] boundaries). At first glance, it might seem most efficient to store the LBA values in this same format in your OS. However, this is not the case. It is always necessary to validate the LBAs that are passed into your driver, as truly belonging to the partition that is being accessed. It ends up being smartest to use partition-relative LBA addressing in your code, because you then never need to test if the LBA being accessed is "off the front" of your current partition. So you only need to do half as many tests. This makes up for the fact that you need to add the absolute LBA of the beginning of the current partition to every "relative" LBA value passed to the driver. At the same time, doing this can give you access to one additional LBA address bit. (See the [[#ATA Driver|"33 bit LBA" driver]] code below.)



===Registers===

An ATA bus typically has ten I/O ports which control its behavior. For the primary bus, these I/O ports are usually 0x1F0 (the "I/O" port base) through 0x1F7 and 0x3F6 (the "Control" port base) through 0x3F7. For the secondary bus, they are usually 0x170 through 0x177 and 0x376 through 0x377. Some systems may have non-standard port locations for the ATA busses, in which case it may be helpful to consult the section on PCI to determine how to retrieve port addresses for various devices in the system.

The values in these tables are relative to the so-called port base; a port offset of 1 from the I/O port base actually refers to port 0x1F1 (0x1F0 + 1 = 0x1F1). This is done because the base port may vary depending on the hardware in any given system. Also, some of these I/O ports map to different registers based on whether they are being read from or written to.

{| {{wikitable}}
|-
! Offset from
"I/O" base
! Direction
! Function
! Description
! Param. size LBA28/LBA48
|-
| 0
| R/W
| Data Register
| Read/Write PIO '''data''' bytes
| 16-bit / 16-bit
|-
| 1
| R
| Error Register
| Used to retrieve any error generated by the last ATA command executed.
| 8-bit / 16-bit
|-
| 1
| W
| Features Register
| Used to control command specific interface features.
| 8-bit / 16-bit
|-
| 2
| R/W
| Sector Count Register
| Number of sectors to read/write (0 is a special value).
| 8-bit / 16-bit
|-
| 3
| R/W
| Sector Number Register (LBAlo)
| This is CHS / LBA28 / LBA48 specific.
| 8-bit / 16-bit
|-
| 4
| R/W
| Cylinder Low Register / (LBAmid)
| Partial Disk Sector address.
| 8-bit / 16-bit
|-
| 5
| R/W
| Cylinder High Register / (LBAhi)
| Partial Disk Sector address.
| 8-bit / 16-bit
|-
| 6
| R/W
| Drive / Head Register
| Used to select a drive and/or head. Supports extra address/flag bits.
| 8-bit / 8-bit
|-
| 7
| R
| Status Register
| Used to read the current status.
| 8-bit / 8-bit
|-
| 7
| W
| Command Register
| Used to send ATA commands to the device.
| 8-bit / 8-bit
|-
|}



{| {{wikitable}}
|-
! Offset from
"Control" base
! Direction
! Function
! Description
! Param. size LBA28/LBA48
|-
| 0
| R
| Alternate Status Register
| A duplicate of the Status Register which does not affect interrupts.
| 8-bit / 8-bit
|-
| 0
| W
| Device Control Register
| Used to reset the bus or enable/disable interrupts.
| 8-bit / 8-bit
|-
| 1
| R
| Drive Address Register
| Provides drive select and head select information.
| 8-bit / 8-bit
|-
|}



====Error Register====

{| {{wikitable}}
|-
! Bit
! Abbreviation
! Function
|-
| 0
| AMNF
| Address mark not found.
|-
| 1
| TKZNF
| Track zero not found.
|-
| 2
| ABRT
| Aborted command.
|-
| 3
| MCR
| Media change request.
|-
| 4
| IDNF
| ID not found.
|-
| 5
| MC
| Media changed.
|-
| 6
| UNC
| Uncorrectable data error.
|-
| 7
| BBK
| Bad Block detected.
|}



====Drive / Head Register (I/O base + 6)====

{| {{wikitable}}
|-
! Bit
! Abbreviation
! Function
|-
| 0 - 3
| 
| In CHS addressing, bits 0 to 3 of the head. In LBA addressing, bits 24 to 27 of the block number.
|-
| 4
| DRV
| Selects the drive number.
|-
| 5
| 1
| Always set.
|-
| 6
| LBA
| Uses CHS addressing if clear or LBA addressing if set.
|-
| 7
| 1
| Always set.
|}



====Status Register (I/O base + 7)====

{| {{wikitable}}
|-
! Bit
! Abbreviation
! Function
|-
| 0
| ERR
| Indicates an error occurred. Send a new command to clear it (or nuke it with a Software Reset).
|-
| 1
| IDX
| Index. Always set to zero.
|-
| 2
| CORR
| Corrected data. Always set to zero.
|-
| 3
| DRQ
| Set when the drive has PIO data to transfer, or is ready to accept PIO data.
|-
| 4
| SRV
| Overlapped Mode Service Request.
|-
| 5
| DF
| Drive Fault Error ('''does not set ERR''').
|-
| 6
| RDY
| Bit is clear when drive is spun down, or after an error. Set otherwise.
|-
| 7
| BSY
| Indicates the drive is preparing to send/receive data (wait for it to clear). In case of 'hang' (it never clears), do a software reset.
|}

Technically, when BSY is set, the other bits in the Status byte are meaningless. It is also generally a Bad Idea to test the "Seek Complete" (DSC) bit, because it has been deprecated and replaced by the newer SRV bit.



====Alternate Status Register (Control base + 0)====

Reading the Device Control Register port gets you the value of the Alternate Status Register, instead.
The value of Alternate Status is always the same as the Regular Status port (0x1F7 on the Primary bus), but reading the Alternate Status port does not affect interrupts. (See Preempting IRQs, below).



====Device Control Register (Control base + 0)====

There is an additional IO port that changes the behavior of each ATA bus, called the Device Control Register (on the Primary bus, port 0x3F6). Each ATA bus has its own Control Register.

{| {{wikitable}}
|-
! Bit
! Abbreviation
! Function
|-
| 0
| n/a
| Always set to zero.
|-
| 1
| nIEN
| Set this to stop the current device from sending interrupts.
|-
| 2
| SRST
| Set, then clear (after 5us), this to do a "Software Reset" on all ATA drives on a bus, if one is misbehaving.
|-
| 3
| n/a
| Reserved.
|-
| 4
| n/a
| Reserved.
|-
| 5
| n/a
| Reserved.
|-
| 6
| n/a
| Reserved.
|-
| 7
| HOB
| Set this to read back the High Order Byte of the last LBA48 value sent to an IO port.
|}

All other bits are reserved and should always be clear. In general, you will want to leave HOB, SRST, and nIEN cleared. Set each Device Control Register to 0 once, during boot.

====Drive Address Register (Control base + 1)====

{| {{wikitable}}
|-
! Bit
! Abbreviation
! Function
|-
| 0
| DS0
| Drive 0 select. Clears when drive 0 selected.
|-
| 1
| DS1
| Drive 1 select. Clears when drive 1 selected.
|-
| 2 - 5
| HS0 - HS3
| One's compliment representation of the currently selected head.
|-
| 6
| WTG
| Write gate; goes low while writing to the drive is in progress.
|-
| 7
| n/a
| Reserved for compatibility with floppy drive controllers which may use this bit.
|}

==Resetting a drive / Software Reset==

For non-ATAPI drives, the only method a driver has of resetting a drive after a major error is to do a "software reset" on the bus. Set bit 2 (SRST, value = 4) in the proper Control Register for the bus. This will reset <b>both</b> ATA devices on the bus. Then, you have to clear that bit again, yourself. The master drive on the bus is automatically selected. ATAPI drives set values on their LBA_LOW and LBA_HIGH IO ports, but are not supposed to reset or even terminate their current command.



==IRQs==

Note: When a command terminates with an error, it does <b>not</b> generate an IRQ. It is smart to check the Alternate Status Register a few times per second to see if the ERR bit has set. Otherwise, you will not know until your command times out.



===Handling an IRQ===

In the early days, the only intent of an IRQ was to inform the IRQ handler that the drive was ready to send or accept data. The expectation was that the IRQ handler itself would perform a PIO based data transfer of the next data block, immediately.
Now things are not so simple. One or both of the drives on the bus may be in DMA mode, or have data block sizes other than 256 16-bit values.
Also, there is more emphasis now on returning as quickly as possible out of the IRQ handler routine. So the question is: what is the minimal set of operations that an IRQ handler needs to do?

If you are using IRQ sharing, you will need to check the PCI Busmaster Status byte, to verify that the IRQ came from the disk.
If it did, it is necessary to read the Regular Status Register once, to make the disk clear its interrupt flag.
If the ERR bit in the Status Register is set (bit 0, value = 1), you may want to read and save the "error details" value from the Error IO port (0x1F1 on
the Primary bus).

If the transfer was a READ DMA operation, you <i>must</i> read the value from the Busmaster Status Register. Since the IRQ handler probably doesn't know whether the operation was a DMA operation or not, you will probably end up checking the Busmaster Status byte after all IRQs (if the bus is controlled by a PCI controller at all -- which it almost certainly is).
If that byte has its ERR bit set (bit 1, value = 2), you may want to save the current values in the disk's LBA IO ports -- they can tell you which sector on the drive generated the error. You will also need to clear the error bit, by writing a 2 to it.

You will also need to send EOI (0x20) to both PICs, to clear their interrupt flags. Then you need to set a flag to "unblock" the driver, and let it know that another IRQ has occurred -- so the driver can do any necessary data transfer.

Note: if you are still in singletasking mode, and polling the Regular Status Register in PIO mode only, then the only thing the IRQ handler needs to do is send EOI to the PICs. You may even want to set the Control Register's nIEN bit, to try to shut off disk IRQs completely.



===Polling the Status vs. IRQs===

When a driver issues a PIO read or write command, it needs to wait until the drive is ready before transferring data. There are two ways to know when the drive is ready for the data. The drive will send an IRQ when it is good and ready. Or, a driver can poll one of the Status ports (either the Regular or Alternate Status).

There are two advantages to polling, and one gigantic disadvantage.
Advantages: Polling responds more quickly than an IRQ. The logic of polling is much simpler than waiting on an IRQ.

The disadvantage: In a multitasking environment, polling will eat up all your CPU time. However, in singletasking mode this is not an issue (the CPU has nothing better to do) -- so polling is a good thing, then.

<b>How to poll</b> (waiting for the drive to be ready to transfer data):
Read the Regular Status port until bit 7 (BSY, value = 0x80) clears, and bit 3 (DRQ, value = 8) sets -- or until bit 0 (ERR, value = 1) or bit 5 (DF, value = 0x20) sets. If neither error bit is set, the device is ready right then.



===Preempting/Preventing IRQs from firing:===
If a driver ever reads the Regular Status port after sending a command to a drive, the "response" IRQ may never happen. If you <i>want</i> to receive IRQs, then always read the Alternate Status port, instead of the Regular Status port. But sometimes IRQs are just wasteful, and it is a good idea to make them go away.

A much more complete way to prevent ATA IRQs from happening is to set the nIEN bit in the Control Register of a particular <i>selected drive</i>. This should prevent the drive on the bus from sending any IRQs at all, until you clear the bit again. <b>However, it may not always work!</b> Several programmers have reported problems making nIEN work.
Drives only respond to <i>newly written values</i> of nIEN when they are the selected drive on the bus. That is, if a drive is selected, and you set nIEN, then select the other drive with the Drive Select Register, then clear nIEN -- then the first drive should "remember" forever that it was told not to send IRQs -- until you select it again, and write a 0 to the nIEN bit in the Control Register.



===Read/Write Multiple===

One way of trying to reduce the number of IRQs in <b>multitasking</b> PIO mode is to use the READ MULTIPLE (0xC4), and WRITE MULTIPLE (0xC5) commands. These commands make the drive buffer "blocks" of sectors, and only send one IRQ per block, rather than one IRQ per sector. See uint16_t 47 and 59 of the IDENTIFY command, to determine the number of sectors in a block. You can also try to use the SET MULTIPLE MODE (0xC6) command, to change the sectors per block.

<b>NOTE:</b> Overall, PIO mode is a slow transfer method. Under real working conditions, almost any drive should be controlled by a DMA driver, and should not be using PIO. Trying to speed up PIO mode by preempting IRQs (or any other method) is mostly a waste of time and effort. ATA drives that are 400MB or smaller may not support Multiword DMA mode 0, however. If you want to support drives that size, then perhaps a <i>little</i> effort spent on PIO mode drivers is worthwhile.



==x86 Directions==

===28 bit PIO===
Assume you have a sectorcount byte and a 28 bit LBA value. A sectorcount of 0 means 256 sectors = 128K.

Notes: When you send a command byte and the RDY bit of the Status Registers is clear, you may have to wait (technically up to 30 seconds) for the drive to spin up, before DRQ sets. You may also need to ignore ERR and DF the first four times that you read the Status, if you are polling.

An example of a 28 bit LBA PIO mode read on the Primary bus:
# Send 0xE0 for the "master" or 0xF0 for the "slave", ORed with the highest 4 bits of the LBA to port 0x1F6: outb(0x1F6, 0xE0 | (slavebit << 4) | ((LBA >> 24) & 0x0F))
# Send a NULL byte to port 0x1F1, if you like (it is ignored and wastes lots of CPU time): outb(0x1F1, 0x00)
# Send the sectorcount to port 0x1F2: outb(0x1F2, (unsigned char) count)
# Send the low 8 bits of the LBA to port 0x1F3: outb(0x1F3, (unsigned char) LBA))
# Send the next 8 bits of the LBA to port 0x1F4: outb(0x1F4, (unsigned char)(LBA >> 8))
# Send the next 8 bits of the LBA to port 0x1F5: outb(0x1F5, (unsigned char)(LBA >> 16))
# Send the "READ SECTORS" command (0x20) to port 0x1F7: outb(0x1F7, 0x20)
# Wait for an IRQ or poll.
# Transfer 256 16-bit values, a uint16_t at a time, into your buffer from I/O port 0x1F0. (In assembler, REP INSW works well for this.)
# Then loop back to waiting for the next IRQ (or poll again -- see next note) for <i>each successive sector</i>.

Note for polling PIO drivers:
After transferring the last uint16_t of a PIO data block to the data IO port, give the drive a 400ns delay to reset its DRQ bit (and possibly set BSY again, while emptying/filling its buffer to/from the drive).

Note on the "magic bits" sent to port 0x1f6: Bit 6 (value = 0x40) is the LBA bit. This must be set for either LBA28 or LBA48 transfers. It must be clear for CHS transfers. Bits 7 and 5 are obsolete for <b>current</b> ATA drives, but must be set for backwards compatibility with very old (ATA1) drives.

===Writing 28 bit LBA===
To write sectors in 28 bit PIO mode, send command "WRITE SECTORS" (0x30) to the Command port. Do <b>not</b> use <b>REP</b> OUTSW to transfer data. There must be a tiny delay between each OUTSW output uint16_t. A jmp $+2 size of delay. Make sure to do a Cache Flush (ATA command 0xE7) after each write command completes.



===48 bit PIO===
Reading sectors using 48 bit PIO is very similar to the 28 bit method:

(Notes: A sector count of 0 means 65536 sectors = 32MB. Try not to send bytes to the same IO port twice in a row. Doing so is <b>much</b> slower than doing two outb() commands to <b>different</b> IO ports. The important thing is that the high byte of the sector count, features and LBA bytes 4, 5, & 6 go to their respective ports <b>before the low bytes</b>.)

Assume you have a sectorcount uint16_t and a 6 byte LBA value. Mentally number the LBA bytes as 1 to 6, from low to high.
Send the 2 byte sector count to port 0x1F2 (high byte first), and the six LBA byte pairs to ports 0x1F3 through 0x1F5 in some appropriate order.

An example:
# Send 0x40 for the "master" or 0x50 for the "slave" to port 0x1F6: outb(0x1F6, 0x40 | (slavebit << 4))
# outb (0x1F2, sectorcount high byte)
# outb (0x1F3, LBA4)
# outb (0x1F4, LBA5)
# outb (0x1F5, LBA6)
# outb (0x1F2, sectorcount low byte)
# outb (0x1F3, LBA1)
# outb (0x1F4, LBA2)
# outb (0x1F5, LBA3)
# Send the "READ SECTORS EXT" command (0x24) to port 0x1F7: outb(0x1F7, 0x24)

Note on the "magic bits" sent to port 0x1f6: Bit 6 (value = 0x40) is the LBA bit. This must be set for either LBA28 or LBA48 transfers. It must be clear for CHS transfers. Any drive that can support LBA48 will ignore all other bits on this port for an LBA48 command. You can set them if it will make your code cleaner (to use the same magic bits as LBA28).

To write sectors in 48 bit PIO mode, send command "WRITE SECTORS EXT" (0x34), instead. (As before, do not use REP OUTSW when writing.) And remember to do a Cache Flush after each write command completes.

After the command byte is sent, transfer each sector of data in exactly the same way as for a 28 bit PIO Read/Write command.



==CHS mode==

Cylinder, Head, Sector mode is completely obsolete, but there are a few things to know about it, for legacy reasons.

The oldest drives had many glass "platters", and two read/write "heads" per platter. The heads are always lined up, vertically. One head of one of the platters was usually used for "timing". As all the platters rotated, each head traced out a circle, and all the heads together traced out a "cylinder." Each circle traced out by each head was subdivided into some number of "sectors." Each sector could be used for storing 512 bytes of data. Selecting the Cylinder, Head, and Sector became an addressing mode.

Changing the cylinder meant moving the whole head assembly, which was to be avoided if possible.

But the important point is that none of this information has been true for the last 20 years -- except that computers kept accessing data via artificial CHS addressing.

In CHS mode, every drive has a "geometry" -- the legal ranges for the CHS values. The typical maximum legal values are Cyl = 0 to 1023, Head = 0 to 15, Sector = 1 to 63.

<b>Please note that Sector = 0 is always illegal! That is a common cause of errors.</b> (It is also possible that some hardware / drives will accept Cylinder values as high as 65537.)

Converting CHS addressing to LBA is straightforward: (Cylinder * TotalHeads + SelectedHead) * SectorsPerTrack + (SectorNum - 1). Sometimes programs will ask for a CHS address, and you will need to do that calculation by hand.

Accessing sectors in CHS mode is basically identical to doing 28 bit LBA reads and writes, except that you leave the LBA bit (value = 0x40) turned off when writing the Bit Flags port, and you send various CHS bytes instead of LBA bytes to the IO ports.

An example:
# Send 0xA0 for the "master" or 0xB0 for the "slave", ORed with the Head Number to port 0x1F6: outb(0x1F6, 0xA0 | (slavebit << 4) | Head Number)
# outb (0x1F2, bytecount/512 = sectorcount)
# outb (0x1F3, Sector Number -- the S in CHS)
# outb (0x1F4, Cylinder Low Byte)
# outb (0x1F5, Cylinder High Byte)
# Send the "READ SECTORS" command (0x20) to port 0x1F7: outb(0x1F7, 0x20)

To write, send command "WRITE SECTORS" (0x30).

Note on the "magic bits" sent to port 0x1f6: Bit 6 (value = 0x40) is the LBA bit. This must be clear for CHS transfers, as said above. Bits 7 and 5 are obsolete for <b>current</b> ATA drives, but must be set for backwards compatibility with very old (ATA1) drives.



==x86 Code Examples==

===Detecting device types===
(Using a Software Reset -- adapted from PypeClicker)

<source lang="c">
/* on Primary bus: ctrl->base =0x1F0, ctrl->dev_ctl =0x3F6. REG_CYL_LO=4, REG_CYL_HI=5, REG_DEVSEL=6 */
int detect_devtype (int slavebit, struct DEVICE *ctrl)
{
	ata_soft_reset(ctrl->dev_ctl);		/* waits until master drive is ready again */
	outb(ctrl->base + REG_DEVSEL, 0xA0 | slavebit<<4);
	inb(ctrl->dev_ctl);			/* wait 400ns for drive select to work */
	inb(ctrl->dev_ctl);
	inb(ctrl->dev_ctl);
	inb(ctrl->dev_ctl);
	unsigned cl=inb(ctrl->base + REG_CYL_LO);	/* get the "signature bytes" */
	unsigned ch=inb(ctrl->base + REG_CYL_HI);

	/* differentiate ATA, ATAPI, SATA and SATAPI */
	if (cl==0x14 && ch==0xEB) return ATADEV_PATAPI;
	if (cl==0x69 && ch==0x96) return ATADEV_SATAPI;
	if (cl==0 && ch == 0) return ATADEV_PATA;
	if (cl==0x3c && ch==0xc3) return ATADEV_SATA;
	return ATADEV_UNKNOWN;
}
</source>



===ATA Driver===
A complete singletasking (polling) PIO mode driver for reading a hard disk
(Note: the following routines should all include some form of OS-specific timeout.)

<source lang="asm">
; do a singletasking PIO ATA read
; inputs: ebx = # of sectors to read, edi -> dest buffer, esi -> driverdata struct, ebp = 4b LBA
; Note: ebp is a "relative" LBA -- the offset from the beginning of the partition
; outputs: ebp, edi incremented past read; ebx = 0
; flags: zero flag set on success, carry set on failure (redundant)
read_ata_st:
	push edx
	push ecx
	push eax
	test ebx, ebx			; # of sectors < 0 is a "reset" request from software
	js short .reset
	cmp ebx, 0x3fffff		; read will be bigger than 2GB? (error)
	stc
	jg short .r_don
	mov edx, [esi + dd_prtlen]	; get the total partition length (sectors)
	dec edx				; (to avoid handling "equality" case)
	cmp edx, ebp			; verify ebp is legal (within partition limit)
	jb short .r_don			; (carry is set automatically on an error)
	cmp edx, ebx			; verify ebx is legal (forget about the ebx = edx case)
	jb short .r_don
	sub edx, ebx			; verify ebp + ebx - 1 is legal
	inc edx
	cmp edx, ebp			; (the test actually checks ebp <= edx - ebx + 1)
	jb short .r_don
	mov dx, [esi + dd_dcr]		; dx = alt status/DCR
	in al, dx			; get the current status
	test al, 0x88			; check the BSY and DRQ bits -- both must be clear
	je short .stat_ok
.reset:
	call srst_ata_st
	test ebx, ebx			; bypass any read on a "reset" request
	jns short .stat_ok
	xor ebx, ebx			; force zero flag on, carry clear
	jmp short .r_don
.stat_ok:
; preferentially use the 28bit routine, because it's a little faster
; if ebp > 28bit or esi.stLBA > 28bit or stLBA+ebp > 28bit or stLBA+ebp+ebx > 28bit, use 48 bit
	cmp ebp, 0xfffffff
	jg short .setreg
	mov eax, [esi + dd_stLBA]
	cmp eax, 0xfffffff
	jg short .setreg
	add eax, ebp
	cmp eax, 0xfffffff
	jg short .setreg
	add eax, ebx
	cmp eax, 0xfffffff
.setreg:
	mov dx, [esi + dd_tf]		; dx = IO port base ("task file")
	jle short .read28		; test the flags from the eax cmp's above
.read48:
	test ebx, ebx		; no more sectors to read?
	je short .r_don
	call pio48_read		; read up to 256 more sectors, updating registers
	je short .read48	; if successful, is there more to read?
	jmp short .r_don
.read28:
	test ebx, ebx		; no more sectors to read?
	je short .r_don
	call pio28_read		; read up to 256 more sectors, updating registers
	je short .read28	; if successful, is there more to read?
.r_don:
	pop eax
	pop ecx
	pop edx
	ret


;ATA PI0 28bit singletasking disk read function (up to 256 sectors)
; inputs: ESI -> driverdata info, EDI -> destination buffer
; BL = sectors to read, DX = base bus I/O port (0x1F0, 0x170, ...), EBP = 28bit "relative" LBA
; BSY and DRQ ATA status bits must already be known to be clear on both slave and master
; outputs: data stored in EDI; EDI and EBP advanced, EBX decremented
; flags: on success Zero flag set, Carry clear
pio28_read:
	add ebp, [esi + dd_stLBA]	; convert relative LBA to absolute LBA
	mov ecx, ebp			; save a working copy
	mov al, bl		; set al= sector count (0 means 256 sectors)
	or dl, 2		; dx = sectorcount port -- usually port 1f2
	out dx, al
	mov al, cl		; ecx currently holds LBA
	inc edx			; port 1f3 -- LBAlow
	out dx, al
	mov al, ch
	inc edx			; port 1f4 -- LBAmid
	out dx, al
	bswap ecx
	mov al, ch		; bits 16 to 23 of LBA
	inc edx			; port 1f5 -- LBAhigh
	out dx, al
	mov al, cl			; bits 24 to 28 of LBA
	or al, byte [esi + dd_sbits]	; master/slave flag | 0xe0
	inc edx				; port 1f6 -- drive select
	out dx, al

	inc edx			; port 1f7 -- command/status
	mov al, 0x20		; send "read" command to drive
	out dx, al

; ignore the error bit for the first 4 status reads -- ie. implement 400ns delay on ERR only
; wait for BSY clear and DRQ set
	mov ecx, 4
.lp1:
	in al, dx		; grab a status byte
	test al, 0x80		; BSY flag set?
	jne short .retry
	test al, 8		; DRQ set?
	jne short .data_rdy
.retry:
	dec ecx
	jg short .lp1
; need to wait some more -- loop until BSY clears or ERR sets (error exit if ERR sets)

.pior_l:
	in al, dx		; grab a status byte
	test al, 0x80		; BSY flag set?
	jne short .pior_l	; (all other flags are meaningless if BSY is set)
	test al, 0x21		; ERR or DF set?
	jne short .fail
.data_rdy:
; if BSY and ERR are clear then DRQ must be set -- go and read the data
	sub dl, 7		; read from data port (ie. 0x1f0)
	mov cx, 256
	rep insw		; gulp one 512b sector into edi
	or dl, 7		; "point" dx back at the status register
	in al, dx		; delay 400ns to allow drive to set new values of BSY and DRQ
	in al, dx
	in al, dx
	in al, dx

; After each DRQ data block it is mandatory to either:
; receive and ack the IRQ -- or poll the status port all over again

	inc ebp			; increment the current absolute LBA
	dec ebx			; decrement the "sectors to read" count
	test bl, bl		; check if the low byte just turned 0 (more sectors to read?)
	jne short .pior_l

	sub dx, 7		; "point" dx back at the base IO port, so it's unchanged
	sub ebp, [esi + dd_stLBA]	; convert absolute lba back to relative
; "test" sets the zero flag for a "success" return -- also clears the carry flag
	test al, 0x21		; test the last status ERR bits
	je short .done
.fail:
	stc
.done:
	ret


;ATA PI0 33bit singletasking disk read function (up to 64K sectors, using 48bit mode)
; inputs: bx = sectors to read (0 means 64K sectors), edi -> destination buffer
; esi -> driverdata info, dx = base bus I/O port (0x1F0, 0x170, ...), ebp = 32bit "relative" LBA
; BSY and DRQ ATA status bits must already be known to be clear on both slave and master
; outputs: data stored in edi; edi and ebp advanced, ebx decremented
; flags: on success Zero flag set, Carry clear
pio48_read:
	xor eax, eax
	add ebp, [esi + dd_stLBA]	; convert relative LBA to absolute LBA
; special case: did the addition overflow 32 bits (carry set)?
	adc ah, 0			; if so, ah = LBA byte #5 = 1
	mov ecx, ebp			; save a working copy of 32 bit absolute LBA

; for speed purposes, never OUT to the same port twice in a row -- avoiding it is messy but best
;outb (0x1F2, sectorcount high)
;outb (0x1F3, LBA4)
;outb (0x1F4, LBA5)			-- value = 0 or 1 only
;outb (0x1F5, LBA6)			-- value = 0 always
;outb (0x1F2, sectorcount low)
;outb (0x1F3, LBA1)
;outb (0x1F4, LBA2)
;outb (0x1F5, LBA3)
	bswap ecx		; make LBA4 and LBA3 easy to access (cl, ch)
	or dl, 2		; dx = sectorcount port -- usually port 1f2
	mov al, bh		; sectorcount -- high byte
	out dx, al
	mov al, cl
	inc edx
	out dx, al		; LBA4 = LBAlow, high byte (1f3)
	inc edx
	mov al, ah		; LBA5 was calculated above
	out dx, al		; LBA5 = LBAmid, high byte (1f4)
	inc edx
	mov al, 0		; LBA6 is always 0 in 32 bit mode
	out dx, al		; LBA6 = LBAhigh, high byte (1f5)

	sub dl, 3
	mov al, bl		; sectorcount -- low byte (1f2)
	out dx, al
	mov ax, bp		; get LBA1 and LBA2 into ax
	inc edx
	out dx, al		; LBA1 = LBAlow, low byte (1f3)
	mov al, ah		; LBA2
	inc edx
	out dx, al		; LBA2 = LBAmid, low byte (1f4)
	mov al, ch		; LBA3
	inc edx
	out dx, al		; LBA3 = LBAhigh, low byte (1f5)

	mov al, byte [esi + dd_sbits]	; master/slave flag | 0xe0
	inc edx
	and al, 0x50		; get rid of extraneous LBA28 bits in drive selector
	out dx, al		; drive select (1f6)

	inc edx
	mov al, 0x24		; send "read ext" command to drive
	out dx, al		; command (1f7)

; ignore the error bit for the first 4 status reads -- ie. implement 400ns delay on ERR only
; wait for BSY clear and DRQ set
	mov ecx, 4
.lp1:
	in al, dx		; grab a status byte
	test al, 0x80		; BSY flag set?
	jne short .retry
	test al, 8		; DRQ set?
	jne short .data_rdy
.retry:
	dec ecx
	jg short .lp1
; need to wait some more -- loop until BSY clears or ERR sets (error exit if ERR sets)

.pior_l:
	in al, dx		; grab a status byte
	test al, 0x80		; BSY flag set?
	jne short .pior_l	; (all other flags are meaningless if BSY is set)
	test al, 0x21		; ERR or DF set?
	jne short .fail
.data_rdy:
; if BSY and ERR are clear then DRQ must be set -- go and read the data
	sub dl, 7		; read from data port (ie. 0x1f0)
	mov cx, 256
	rep insw		; gulp one 512b sector into edi
	or dl, 7		; "point" dx back at the status register
	in al, dx		; delay 400ns to allow drive to set new values of BSY and DRQ
	in al, dx
	in al, dx
	in al, dx

; After each DRQ data block it is mandatory to either:
; receive and ack the IRQ -- or poll the status port all over again

	inc ebp			; increment the current absolute LBA (overflowing is OK!)
	dec ebx			; decrement the "sectors to read" count
	test bx, bx		; check if "sectorcount" just decremented to 0
	jne short .pior_l

	sub dx, 7		; "point" dx back at the base IO port, so it's unchanged
	sub ebp, [esi + dd_stLBA]	; convert absolute lba back to relative
; this sub handles the >32bit overflow cases correcty, too
; "test" sets the zero flag for a "success" return -- also clears the carry flag
	test al, 0x21		; test the last status ERR bits
	je short .done
.fail:
	stc
.done:
	ret


; do a singletasking PIO ata "software reset" with DCR in dx
srst_ata_st:
	push eax
	mov al, 4
	out dx, al			; do a "software reset" on the bus
	xor eax, eax
	out dx, al			; reset the bus to normal operation
	in al, dx			; it might take 4 tries for status bits to reset
	in al, dx			; ie. do a 400ns delay
	in al, dx
	in al, dx
.rdylp:
	in al, dx
	and al, 0xc0			; check BSY and RDY
	cmp al, 0x40			; want BSY clear and RDY set
	jne short .rdylp
	pop eax
	ret
</source>



==Comments==
ATA Execute Drive Diagnostic command (0x90) may not work right in Bochs.



==See Also==
===Wiki Pages===
* [[ATA Command Matrix|ATA Command Matrix]]
* [[MBR_(x86)|Master Boot Record (x86)]]
* [[Partition_Table|Partition Table (x86)]]
* [[PCI IDE Controller|IDE Controller]]

===Threads===
*[http://www.osdev.org/phpBB2/viewtopic.php?t=12268 How to w/r harddisk in pmode? (ASM Code from Dex)]
*[http://www.osdev.org/phpBB2/viewtopic.php?t=15314 ATA PIO code library (ASM code from XCHG)]
*[https://forum.osdev.org/viewtopic.php?f=1&t=33152&p=285620#p285620 problem with reading and writing to hard disc (C code for reading/writing with PIO28)]

===External Links===
* http://www.t13.org -- T13, the group that creates the ATA standard
* http://www.ata-atapi.com -- Public Domain C driver sourcecode, including SATA, Busmatering DMA, ATAPI -- not perfect, but good.
* [http://hddguru.com/content/en/documentation/ HDD Guru] -- The actual ATA specs from the first one that was released in 1994 to the 8th one in 2006.
* An example of [http://msdn.microsoft.com/en-us/library/windows/hardware/ff559006(v=vs.85).aspx the structure of the data returned by the IDENTIFY Command] (in case you wanted to know what most of the fields were for).
* [http://www.fysnet.net/media_storage_devices.htm A book] written by somebody on the forum about ATA and SATA.
* [https://github.com/omarrx024/xos/blob/master/kernel/blkdev/ata.asm ATA PIO driver in assembly.]

[[Category:ATA]]
[[de:AT Attachment]]
````

<a id="w002240"></a>

## W002240 — Category:ATA

Bron-ID: W002240
Namespace: Category (14)
Revisie: 23196 | Datum: 2019-01-12T07:37:08Z
Bron: https://wiki.osdev.org/Category:ATA
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AATA&oldid=23196

````text
This page contains all pages related to the ATA specifications. To get started quickly, try one of the following pages:
* [[ATA PIO Mode]] 
: Describes the oldest hardware mode for accessing devices. Start here for all your driver needs.
* [[ATAPI]]
: ATA Packet Interface - Add SCSI commands to the protocol. CD-ROM drives are usually ATAPI devices. You will need some [[ATA PIO Mode|ATA]] parts for setup.
* [[ATA in x86 RealMode (BIOS)]]
: If you want to make disk accesses using the legacy BIOS.

===IDE===

The name IDE is often used interchangeably with ATA, but "IDE" actually refers to only the electrical specifications of the signals on the 40 / 80 pin disk cable. ATA is the proper name for the entire specification.

===SATA and PATA===

Recently a new standard for ATA data transmission has emerged. It was named [[SATA]], and the previous well-known forms of ATA were retroactively renamed PATA, "to reduce confusion" -- while certainly causing it.



[[Category:Storage]]
[[Category:Buses]]
[[de:AT_Attachment]]
````

<a id="w002241"></a>

## W002241 — ATA/ATAPI using DMA

Bron-ID: W002241
Namespace: Main (0)
Revisie: 19394 | Datum: 2016-07-02T20:51:22Z
Bron: https://wiki.osdev.org/ATA/ATAPI_using_DMA
Revisie-URL: https://wiki.osdev.org/index.php?title=ATA%2FATAPI_using_DMA&oldid=19394

````text
==ISA DMA==

It is very important to understand that there are several different types of DMA on a machine. The DMA that is used for ATA drives is called PCI Busmastering DMA. It is very fast. It is completely different from ISA DMA, which is limited to 4MB per second. Anything you may have read about 8bit or 16bit DMA channels 0 through 7, <b>does not apply to disk DMA at all.</b>
ATA singleword DMA, ATA multiword DMA, UDMA, and [[ADMA]] are all PCI Busmastering DMA types.

==UDMA==

From the point of view of the OS, there is only a tiny difference between "regular" (Single/Multiword) DMA and any of the UDMA modes.
Most of the actual differences involve precisely how, and how fast, data is transferred between the disk drive and the disk controller. OS software has no direct control over that part of the process. The driver software issues exactly the same commands, in exactly the same way, for all the DMA methods (except ADMA).

The only control the rest of the system has, lies in the initialization of the disk drive and disk controller to mutually compatible DMA modes, during bootup. This will hopefully always be done correctly by the BIOS.

If it is not done correctly by the BIOS, there is a chance that an OS can do the job after bootup. But it is a slim chance, and it is an ugly process. If you want to try it, then read the Intel ATA Controller chip manual linked below.

The one place where UDMA makes a difference to the driver is the concept of a UDMA CRC error. These happen when data blocks get corrupted between the drive and the PCI controller. No other form of DMA uses CRC. In case of an error during the DMA transfer, the driver needs to check if the error was CRC based, and retry the transfer at least once if it was.

===Verifying CRC errors===

Read the PCI Configuration Space uint16_t at offset 6 (Device Status) to get the error code.

==The PRD Table==

You must set up at least one Physical Region Descriptor Table (PRDT) in memory per ATA bus, which contains some number of Physical Region Descriptors (PRDs).
(The PRDT must be uint32_t aligned, contiguous in physical memory, and cannot cross a 64K boundary.)
Then you need to store the physical address of the current PRDT in the Bus Master Register, of the Bus Mastering ATA Disk Controller on the PCI bus (see below).

===PRD setup===

Half of each DMA transfer is encoded in one uint64_t PRD entry (8 bytes) in the table.
(Why is it only half of a DMA transfer? Because the PRD does not contain any information about which LBAs to read from/write to the disk.
That information is sent in an entirely separate way to the proper disk -- see below.)

The low uint32_t is a '''physical''' memory address of a data buffer. (Keep in mind on 64 bit systems that this address must fit in a uint32_t!)
The next uint16_t is a '''byte''' count (not a sector count!) of the transfer size (64K maximum per PRD transfer).
A byte count of 0 means 64K. The next uint16_t is reserved (should be 0) except for the MSB. If the MSB is set, that indicates that this PRD is the last entry in the PRDT, and the entire set of transfers is complete.
The data buffers cannot cross a 64K boundary, and must be contiguous in physical memory (ie. they can't be "virtual" data buffers).
The byte count on the data buffers must match the number of sectors transferred by the disk. If not, the controller will error out in various ways.

==The Bus Master Register==

The "address" of the Bus Master Register is stored in BAR4, in the PCI Configuration Space of the disk controller.
The Bus Master Register is generally a set of 16 sequential IO ports. It can also be a 16 byte memory mapped space.

Format:
<pre>byte offset		function
(Primary ATA bus)
0x0			Command (byte)
0x2			Status (byte)
0x4-0x7			PRDT Address (uint32_t)
(Secondary ATA bus)
0x8			Command (byte)
0xA			Status (byte)
0xC-0xF			PRDT Address (uint32_t)
</pre>

===The Command Byte===
The command byte has only 2 operational bits. All the rest should be 0.
Bit 0 (value = 1) is the Start/Stop bit.
Setting the bit puts the controller in DMA mode for that ATA channel, and it starts at the beginning of the respective PRDT.
Clearing the bit terminates DMA mode for that ATA channel. If the controller was in the middle of a transfer, the remaining data is thrown away.
Also, the controller does not remember how far it got in the PRDT. That information is lost, if the OS does not save it. The bit must be cleared when a transfer completes.

Bit 3 (value = 8) is the Read/Write bit. This bit is a huge problem. The disk controller does not automatically detect whether the next disk operation is a read or write. You have to <i>tell</i> it, in advance, by setting this bit. Note that when reading from the disk, you must set this bit to 1, and clear it when writing to the disk. You must first stop DMA transfers (by clearing bit 0) before you can change the Read/Write bit! Please note all the bad consequences of clearing bit 0, above! The controller loses its place in the PRDT.

In essence, this means that each PRDT must consist exclusively of either read or write entries. You set the Read/Write bit in advance, then "use up" the entire PRDT -- before you can do the opposite operation.

===The Status Byte===
The bits in the status byte are not usually useful. However, you are required to read it after every IRQ on disk reads anyway.
Reading this byte may perform a necessary final cache flush of the DMA data to memory.

* Bit 7 (Simplex operation only) is completely obsolete. However, if you want your OS to support mid 90's era hardware, this bit means you can only use one ATA bus at a time for DMA. The other can only use PIO.
* Perhaps bits 5 and 6 (values 0x20 and 0x40) will provide useful information if examined once. They are supposedly set by the BIOS if the Master or Slave drive on the bus (respectively) are capable of, and initialized for DMA operation. They are there only for your use. They serve no other purpose.
* If bit 2 (value = 4) is not set after the OS receives an IRQ, then some other device sharing the IRQ generated the IRQ -- not the disk. (You might want to reset this bit after every IRQ, by writing a 4 to the Status Byte IO port.)
* Bit 1 (value = 2) is set if any DMA memory transfer failed for any reason in this PRDT.
* Bit 0 (value = 1) is set when the bus goes into DMA mode. It is cleared when the last PRD in the table has been used up. Generally, the OS should already know that this has happened, by receiving the proper number of IRQs from the disk.

==ATA/ATAPI Commands==

The PCI disk controller only handles the memory half of the DMA transfer, by interpreting the PRDT.
The device driver must separately tell the drive to do its half of the work.

To use DMA with an ATAPI drive, write a 1 to the Features IO port (0x1F1 on the Primary bus), and the maxbytecount should be 0 -- when issuing the PACKET command to the drive.

For ATA, for each PRD entry in the PRDT, the driver must issue a Read/Write DMA command to the disk; specifying a StartLBA and a sector count.

When the drive completes each command it sends an IRQ (possibly also when transfer in progress), to let the driver know that it is ready to transfer the next batch of data to/from the disk controller. The driver should then read the Bus Master Register Status byte.

The formats of the commands are precisely the same as for the 28 and 48 bit PIO mode Read and Write commands, except for the value sent to the "Command" IO Port.
<pre>
Command byte:		Function:
0xC8			Read DMA (28 bit LBA)
0x25			Read DMA (48 bit LBA)
0xCA			Write DMA (28 bit LBA)
0x35			Write DMA (48 bit LBA)
</pre>

==Handling Errors==

After a transfer, if the Bus Master Status byte has its ERR bit (bit 1, value = 2) set, then clear it by writing a 2 to that port.
You can then read the LBA IO ports for the bus (0x1F3 through 0x1F6) to find out which sector failed.

For any error except a UDMA CRC error, it is recommended to do a Software Reset on the bus, in order to force all the devices to take the bus out of DMA mode.

==Standard Order of Sending Commands==

* Prepare a PRDT in system memory.
* Send the '''physical''' PRDT address to the Bus Master PRDT Register.
* Set the direction of the data transfer by setting the Read/Write bit in the Bus Master Command Register.
* Clear the Error and Interrupt bit in the Bus Master Status Register.
* Select the drive.
* Send the LBA and sector count to their respective ports.
* Send the DMA transfer command to the ATA controller.
* Set the Start/Stop bit on the Bus Master Command Register.
* When an interrupt arrives (after the transfer is complete), respond by resetting the Start/Stop bit.
* Read the controller and drive status to determine if the transfer completed successfully.
==Comments==

==See Also==
===Articles===
* [[ADMA]]
* [[DMA]]
===External Links===
* [http://bos.asmhackers.net/docs/ata/docs/29860001.pdf Intel manual with DMA / UDMA / PCI configuration space details]
* [http://www.osdever.net/downloads/docs/idems100.zip Very simple original Busmaster DMA spec with PRDT spec]

{{stub}}

[[Category:ATA]]
````

<a id="w002242"></a>

## W002242 — ATA/ATAPI Power Management

Bron-ID: W002242
Namespace: Main (0)
Revisie: 23759 | Datum: 2019-07-16T04:20:40Z
Bron: https://wiki.osdev.org/ATA/ATAPI_Power_Management
Revisie-URL: https://wiki.osdev.org/index.php?title=ATA%2FATAPI_Power_Management&oldid=23759

````text
==ATA STANDBY mode==
Spins down the drive, but leaves the controller board awake. The drive is supposed to accept the next command normally; but it will take lots of time to complete the command (and return to Active mode). The maximum time limit in the ATA spec is 30 seconds, but most drives spin up in a second or two. Use the STANDBY (0xE2) or STANDBY IMMEDIATE (0xE0) commands to put the drive in this mode.

==ATA SLEEP mode==
If you want to put the whole machine to sleep, you can also put the drives into SLEEP mode, with the SLEEP (0xE6) command. You will need to set bit 2 (SRST, value = 4) in the Control Register of each bus to get all the drives to spin up again, afterward.

==ATAPI power management==

Many ATAPI drives support the above STANDBY IMMEDIATE and SLEEP commands. There may also be supported power management commands in the ATAPI command set itself.

==APM==
The BIOS is supposed to enable APM on any drive that supports it, if the system also supports it. APM runs independently of the OS, and invisibly, for the most part. If the BIOS fails to turn on the APM functionality of a device, a driver can do it with the SET FEATURES command (0xEF).

==ACPI==

(stubbed)
{{stub}}

[[Category:ATA]]
[[Category:Power management]]
````

<a id="w002243"></a>

## W002243 — SATA

Bron-ID: W002243
Namespace: Main (0)
Revisie: 30508 | Datum: 2026-04-22T09:38:37Z
Bron: https://wiki.osdev.org/SATA
Revisie-URL: https://wiki.osdev.org/index.php?title=SATA&oldid=30508

````text
'''SATA''' (Serial AT Attachment) was a major interface standard for mass-storage devices such as hard drives, solid-state drives, optical media and more from roughly 2005 onwards, and used as the primary storage interface in most desktop computers until roughly 2020. It succeeded the older [[ATA|(Parallel) ATA]] interface and has in turn been succeeded by [[NVMe]]. Practically all SATA drives can be used using either [[AHCI]] or [[ATA PIO Mode|ATA]].


== Introduction ==

SATA is a bus interface for communicating with mass storage devices (such as disk drives, optical drives, etc). The composition is mostly identical to PATA, where one controller connects to several devices using SATA cables. SATA wiring is also one-on-one instead of allowing more than one device to be connected to a single cable. When SATA was introduced, almost all host controllers were configured to masquerade as an PATA controller, needing no change from the existing disk drivers. The majority of SATA controllers also support [[AHCI]], which offers connecting more devices than PATA allows, and significant other advantages over the legacy protocol. 

Computers that are currently shipped often are [[AHCI]] only, and backwards compatibility mode, if present, has to be manually enabled from the BIOS - a change that often comes with the cost of making the installed OSes unbootable.

The actual SATA specification is split into three distinct protocol 'layers'. 

* Physical Layer - Information relating to the physical connectors, etc. Not especially relevant here. 
* Link Layer
* Transport Layer

At the time of writing, there were three supposedly definitive, conflicting standards for SATA. It may be best to wait for the industry to shake out a little more completely, before trying to write SATA drivers.

If you want to try anyway, then get the ATA7, and ATA8 technical docs from [http://www.t13.org www.t13.org]. Then maybe try paying $25 to get the technical specs from www.sata-io.org. Hopefully, the discrepancies between the standards aren't too bad.

== Detection and Initialization ==

Send a standard IDENTIFY command to the drive (0xEC). The drive should respond with an error in the ERR bit of the Status Register, and a pair of "signature bytes". On the Primary ATA bus, you get the signature bytes by reading IO ports 0x1F4 and 0x1F5, and you should see values of 0x3C and 0xC3.

== See Also ==
=== Articles ===
* [[AHCI]]

=== External Links ===
* [https://sata-io.org/system/files/specifications/SerialATA_Revision_3_1_Gold.pdf Serial ATA Revision 3.1 - Gold Revision]
*[http://www.sata-io.org/ The Serial ATA International Organization]
* http://www.ata-atapi.com -- Public Domain C driver sourcecode, including SATA, Busmatering DMA, ATAPI -- not perfect, but good.


[[Category:ATA]]
[[Category:Disks]]
[[Category:Storage]]
[[Category:Standards]]
````

<a id="w002244"></a>

## W002244 — ADMA

Bron-ID: W002244
Namespace: Main (0)
Revisie: 20357 | Datum: 2017-02-11T16:20:32Z
Bron: https://wiki.osdev.org/ADMA
Revisie-URL: https://wiki.osdev.org/index.php?title=ADMA&oldid=20357

````text
ADMA is a feature of some PCI ATA controllers, ADMA eliminates the need for the driver to send the READ and WRITE commands to the drive. The Bus Mastering Controller sets up <b>both</b> sides of every DMA transfer, and initiates the transfer itself. All the driver needs to do is set up a table of "commands" (a CPB), set some pointers, and then turn on ADMA mode. This greatly reduces excessive IO operations, and frees up precious CPU time.

An ADMA capable controller maps the standard ATA IO and ADMA registers at a 64-bit or 32-bit address (...PCI BAR 4/5, 20h-27h). The size of this area is 1024 bytes.

ADMA supports Queued and Overlapped ATA command sets, but neither is necessary to support ADMA. Both the ATA and ATAPI commands sets are supported.

==Detection and Initialization==

The ATA/ATAPI Host Adapters Standard specifies a PCI Class Code, details initialization, and operation of ADMA devices.

7.4.1.6 PCI Class Code / Table 14 – ADMA PCI Class Code
* Programming Interface Code: 20h – Single Stepping / 30h – Continuous Operation
* Sub-class Code: 05h – ATA
* Base-Class Code: 01h – Mass Storage

Several vendors have chosen ADMA mode as the basis for their SATA controller implementations, however, often these implementations contain proprietary extensions. For this reason, the PCI Class may be different... driver "quirks" may be required to support these devices.

==See Also==
===Articles===
* [[ATA/ATAPI using DMA]]
===External Links===
* http://www.t13.org/Documents/UploadedDocuments/technical/e00149r0.pdf -- Initial specification
* http://www.t13.org/Documents/UploadedDocuments/project/d1510r1-Host-Adapter.pdf -- Latest draft of "ATA/ATAPI Host Adapters Standard"

{{stub}}

[[Category:ATA]]
````

<a id="w002245"></a>

## W002245 — Disk access using the BIOS (INT 13h)

Bron-ID: W002245
Namespace: Main (0)
Revisie: 30242 | Datum: 2026-03-16T15:43:42Z
Bron: https://wiki.osdev.org/Disk_access_using_the_BIOS_(INT_13h)
Revisie-URL: https://wiki.osdev.org/index.php?title=Disk_access_using_the_BIOS_%28INT_13h%29&oldid=30242

````text
The [[BIOS]] provides a set of disk access routines using the INT 0x13 family of BIOS functions. Until an appropriate driver is implemented, these BIOS functions are the only way to access disks.

There are two basic INT 0x13 call families to use for disk access. One uses Cylinder, Head, Sector addressing (CHS), and the other uses [[LBA]] addressing. There is a third command set to access [[ATAPI]] drives using the PACKET command set.

Note: These BIOS INT calls should completely preserve all the registers (except AX). However, some older versions of the [[Bochs]]/[[QEMU]] BIOS destroyed the upper 16-bit values of some of the 32-bit registers.

==CHS==

Remember that there are severe addressing limitations with CHS addressing. Typically only the first 8 GB of the media can be accessed, at most. Sometimes USB drives might be using floppy emulation, so the maximum accessable size becomes only 1.44M.

===Converting LBA to CHS===
The addresses of sectors on a disk are almost always calculated or stored as LBAs, but some drives (especially floppies and USB drives with floppy emulation) cannot use LBA addressing. So your code must translate the address, and use a CHS read call. This also applies if you are trying to read floppies and hard disks with the same code.

====Quick Explanation of the Algorithm====
You can think of a CHS address as the digits of a 3-digit number. The sectors are the low digit, the heads are the middle digit, and cylinders are the high digit. As an analogy, think of the decimal number 345. To extract the low (sectors) digit, you take the modulo with 10. 345 % 10 = 5. You also need the integer result of 345 / 10 to calculate the heads and cylinders. 345 / 10 = 34. Then %10 again gets the head, and /10 gets the cylinder. The nice thing is that all CPU chips have "div" opcodes that give you <b>both</b> the result <b>and</b> the modulus for every operation.

====The Algorithm====
LBA is the input value,
*Temp = LBA / (Sectors per Track)
*Sector = (LBA % (Sectors per Track)) + 1
*Head = Temp % (Number of Heads)
*Cylinder = Temp / (Number of Heads)

'''
Note: Always remember that sector is 1-based, and not 0-based ... this detail causes many problems.
'''
====Getting Sectors/Track, Total Head values====
There is only one real place where you can get the "Sectors per Track" and "Number of Heads" values for the previous LBA->CHS calculation. All modern BIOSes use automatic CHS to LBA conversions internally, with internal hardcoded conversion values. They <b>do not use</b> the "real" CHS values that are written on the drive's label. Also, if you perhaps have a FAT formatted drive, it will claim to have "Sectors per Track" and "Number of Heads" information stored in the BPB. These values are almost always <b>wrong</b>.

If the 0x80 bit is set for the BIOS drive number, then you have no real choice other than to use the INT13h AH=8 BIOS function to get the "drive geometry".
*Set AH to 8, DL to the BIOS drive number, and execute INT 0x13.
*The value returned in DH is the "Number of Heads" -1.
*AND the value returned in CL with 0x3f to get the "Sectors per Track".

Note: INT 0x13 AH=8 does not work well with floppy drives, or emulated floppy drives. It may be best to use default values in that case.

===Reading sectors with a CHS address===

Cylinder = 0 to 1023 (maybe 4095), Head = 0 to 15 (maybe 254, maybe 255), Sector = 1 to 63
*Set AH = 2
*AL = total sector count (0 is illegal) -- cannot cross ES page boundary, <b>or a cylinder boundary</b>, and must be < 128; in some rare cases, even crossing track boundaries in a single request may not be allowed
*CH = cylinder & 0xff
*CL = Sector | ((cylinder >> 2) & 0xC0);
*DH = Head -- may include two more cylinder bits
*ES:BX -> buffer
*Set DL = "drive number" -- typically 0x80, for the "C" drive
*Issue an INT 0x13.

The carry flag will be clear if the read operation completed successfully and set if there is any error during the transfer. AH should be set to 0 on success and non-zero on error.

'''Warning''': Some BIOSes will also set AL to the sectors successfully read, either only on error, or on both success and error. In case of success, it would be equal to the sectors requested. In case of error, a non-zero sector count would indicate a partial read. However, not all BIOSes adhere to this convention and, even if AL is set accordingly, it shouldn't be relied upon for handling partial reads because it cannot be reliably determined which parts of the request have actually succeeded due to possible out-of-order completion of DMA transfers. For all practical purposes, all errors including partial reads should always be treated as errors. AL should always be treated as undefined/destroyed, regardless of success or error, and any data previously residing in the designated read buffer up to the read length should be treated as destroyed even if a read error occurs.

'''Note''': The limitation about not crossing cylinder (and rarely track) boundaries is very annoying, especially when combined with the 127 sector limit -- because the arithmetic for the length and "start CHS" of the <i>next</i> consecutive read or write gets messy. Not all BIOSes have these two limitations, of course, but it is necessary to program for the "lowest common denominator". The simplest workaround is to read or write only one sector at a time in CHS mode. A slightly more advanced but possibly more efficient solution is to split requests on track boundaries, while making sure that all 63 sectors maximum for a track will fit in the rest of the ES segment after ES:BX.

To write to a disk, set AH = 0x03 instead.

==LBA in Extended Mode==

To use [[LBA]] addressing with INT 0x13, you need to use a command in the "INT13h Extensions" aka "BIOS Enhanced Disk Drive Services" (EDD).
Every BIOS since the mid-90's supports the extensions, but you may want to verify that they are supported anyway.

* Set AH = 0x41
* BX = 0x55AA
* DL = 0x80
* Issue an INT 0x13.

Upon success

The carry flag will be unset, BX will have its bytes swapped to 0xAA55, AH and DH will contain some version information.

Lastly CX will contain flags for the supported commands in AH.

For this part of the guide bit 0 should be set, which indicates support for the extended read/write services.

To read or write, first you need to set up a "Disk Address Packet Structure" in memory, on a uint32_t (4 byte) boundary.

{| class="wikitable"
|+ Format of the Disk Address Packet
! Offset
! Size
! Description
|-
| 0
| 1
| Size of packet (16 bytes, 0x10)
|-
| 1
| 1
| Always 0
|-
| 2
| 2
| Number of sectors to transfer (max 127 on some BIOSes)
|-
| 4
| 4
| Transfer buffer (16 bit segment:16 bit offset) (see note #1)
|-
| 8
| 8
| LBA address of sector to read from disk
|}

Notes: 

(1) The 16 bit segment value ends up at an offset of 6 from the beginning of the structure (i.e., when declaring segment:offset as two separate 16-bit fields, place the offset first and then follow with the segment because x86 is little-endian). 

(2) If the disk drive itself does not support LBA addressing, the BIOS will automatically convert the LBA to a CHS address for you -- so this function still works. 

(3) The transfer buffer should be 16-bit (2 byte) aligned.


To read a disk:

*Set the proper values in the Disk Address Packet structure
*Set DS:SI -> Disk Address Packet in memory
*Set AH = 0x42
*Set DL = "drive number" -- typically 0x80 for the "C" drive
*Issue an INT 0x13.

The carry flag will be clear if the read operation completed successfully and set if there is any error during the transfer. AH should be set to 0 on success and to non-zero on error. 

'''Warning''': The "numbers of sectors to transfer" field in the Disk Address Packet may be set to the sectors successfully read, but it shouldn't be relied upon for handling partial reads, similar to AL in CHS mode as described above. For all practical purposes, all errors including partial reads should always be treated as errors. The "numbers of sectors to transfer" should always be treated as undefined/destroyed, regardless of success or error, and any data previously residing in the designated read buffer up to the read length should be treated as destroyed even if a read error occurs.

To write to a disk, set AH = 0x43 instead.

=== 64-bit Addressing Extensions ===

The BIOS Enhanced Disk Drives Services version 3.0 specifies the possibility of loading to flat 64-bit addresses. Then the format of disk address packet looks like:
{| class="wikitable"
|+ Format of the Disk Address Packet (EDD 3.0 version)
! Offset
! Size
! Description
|-
| 0
| 1
| Size of packet (24 bytes, 0x18)
|-
| 1
| 1
| Always 0
|-
| 2
| 2
| Number of sectors to transfer (max 127 on some BIOSes)
|-
| 4
| 4
| Transfer buffer. Must be set to <tt>0xFFFF:0xFFFF</tt> in this case.
|-
| 8
| 8
| LBA address of sector to read from disk
|-
| 16
| 8
| 64-bit, linear load address of the transfer buffer.
|}

'''Warning''': 64-bit addressing extensions are not implemented by most BIOSes, therefore they are very unpractical. Even the BIOSes of most virtual machines don't implement them.

==x86 Examples==

* Reading 16 sectors from LBA #1 to physical address 0x7C00
<syntaxhighlight lang="nasm">
DAPACK:
	db	0x10
	db	0
blkcnt:	dw	16			; int 13 resets this to # of blocks actually read/written
db_add:	dw	0x7C00		; memory buffer destination address (0:7c00)
	dw	0				; in memory page zero
d_lba:	dd	1			; put the lba to read in this spot
	dd	0				; more storage bytes only for big lba's ( > 4 bytes )

	mov si, DAPACK		; address of "disk address packet"
	mov ah, 0x42		; AL is unused
	mov dl, 0x80		; drive number 0 (OR the drive # with 0x80)
	int 0x13
	jc short .error
</syntaxhighlight>

* Checking presence of EDD extensions
<syntaxhighlight lang="nasm">
check_edd:
	; Disk number is on DL
   	; Note that AL may destroyed by this service.
	mov ah, 0x41 	; "check extensions present" service
	mov bx, 0x55aa
	int 0x13
	jc .no_edd
	cmp bx, 0xaa55
	jne .no_edd
	test cx, 1		; Check support for the "fixed disk access subset"
	jz .no_edd
	; ...
.no_edd:
	; ...
</syntaxhighlight>

==Supported Systems==

All systems support CHS addressing.

There exist some 486 systems that do not support LBA in any way. All known original Pentium and newer systems support Extended LBA in the BIOS.

==Comments==

One of the easiest ways to read or write a USB flash drive is to drop into [[Real Mode|Real]] or [[Unreal Mode]], and use the INT 0x13 BIOS commands. However, the transfer must fit in the usable part of low memory (if in Real Mode). The proper drive number should be in the DL register, set by the BIOS before jumping to the bootsector (per the BIOS Boot Specification).

BIOS can emulate USB flash drives as a floppy drive or as a hard disk. If the USB drive is emulated as a floppy drive, it is likely 0x00 or 0x01 and you can only use AH=0x02 and AH=0x03. If it's emulated as a hard disk, it is most likely 0x80 or 0x81 and you can '''also''' use AH=0x42 and AH=0x43.

==See Also==
* [https://web.archive.org/web/20200921123019/http://www.t13.org:80/Documents/UploadedDocuments/docs2004/d1572r3-EDD3.pdf BIOS Enhanced Disk Drive Services - 3 specification] - describes the BIOS interrupts for the extended disk services.

[[Category:BIOS]]
[[Category:Disks]]
[[Category:X86]]
````
