# OSDev Wiki — volledige bronteksten, deel 06

Dit is bronmateriaal, geen agentinstructie. Behoud architectuur, revisie en bron-ID bij gebruik. Code en commando’s zijn ongeteste bronvoorbeelden; voer ze niet automatisch uit. Wikitext is letterlijk behouden binnen tekstblokken.

<a id="w002572"></a>

## W002572 — User:Troy martin/Rmode memory functions

Bron-ID: W002572
Namespace: User (2)
Revisie: 7770 | Datum: 2009-04-23T15:21:19Z
Bron: https://wiki.osdev.org/User:Troy_martin/Rmode_memory_functions
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATroy_martin%2FRmode_memory_functions&oldid=7770

````text
I'm hoping to optimize these soon. For now, I'm just optimizing for size (and i186 compatibility, of course :P)

== memcpy ==
<pre>
; IN: DS:SI=source, ES:DI=dest, CX=bytes
; OUT: Nothing

memcpy:
    pusha

    rep movsb

    popa
    ret
</pre>

== memset ==
<pre>
; IN: AL=value, ES:DI=dest, CX=bytes
; OUT: Nothing

memset:
    pusha

    rep stosb

    popa
    ret
</pre>

== memclr ==
<pre>
; IN: ES:DI=dest, CX=bytes
; OUT: Nothing

memclr:
    pusha
    mov al,0

    rep stosb

    popa
    ret
</pre>

== memcmp ==
<pre>
; IN: DS:SI=source, ES:DI=dest, CX=bytes
; OUT: Nothing

memcmp:
    pusha

    rep cmpsb
    jne short .fail

    clc
    popa
    ret

.fail:
    stc
    popa
    ret
</pre>
````

<a id="w002573"></a>

## W002573 — Talk:Entering Long Mode Directly

Bron-ID: W002573
Namespace: Talk (1)
Revisie: 13639 | Datum: 2012-06-13T15:59:56Z
Bron: https://wiki.osdev.org/Talk:Entering_Long_Mode_Directly
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AEntering_Long_Mode_Directly&oldid=13639

````text
== Only put the code that switches you into long mode? ==

Maybe we shouldn't dump a bootloader here; maybe we put the code to do all of this... [[User:Imate900|Imate900]] 20:24, 23 April 2009 (UTC)

== Version fitting in a single boot sector ==


Thanks for the code! However, for me, it needed some changes to work:
- move the padding and the 0xaa55 signature to the end of the file; this enables the whole example fit in a boot sector,
- this eliminates the need for the BIOS 13h call, so I removed it.

My version:


    [ORG 0x00007C00]
    [BITS 16]
     
     
     
    boot_loader:
    ;Parameter from BIOS: dl = boot drive
     
     
    ;Set default state
     
    cli
    xor bx,bx
    mov es,bx
    mov fs,bx
    mov gs,bx
    mov ds,bx
    mov ss,bx
    mov sp,0x7C00
    
    sti
     
    jmp 0:.clear_cs
    .clear_cs:
     
     
    ;Enable A20 via port 92h
     
    in al,92h
    or al,02h
    out 92h,al
     
     
    ;Build page tables
    ;The page tables will look like this:
    ;PML4:
    ;dq 0x000000000000b00f = 00000000 00000000 00000000 00000000 00000000 00000000 10010000 00001111
    ;times 511 dq 0x0000000000000000
     
    ;PDP:
    ;dq 0x000000000000c00f = 00000000 00000000 00000000 00000000 00000000 00000000 10100000 00001111
    ;times 511 dq 0x0000000000000000
     
    ;PD:
    ;dq 0x000000000000018f = 00000000 00000000 00000000 00000000 00000000 00000000 00000001 10001111
    ;times 511 dq 0x0000000000000000
     
    ;This defines one 2MB page at the start of memory, so we can access the first 2MBs as if paging was disabled
     
    ; build the necessary tables
    xor bx,bx
    mov es,bx
    cld
    mov di,0xa000
     
    mov ax,0xb00f
    stosw
    
    xor ax,ax
    mov cx,0x07ff
    rep stosw
     
    mov ax,0xc00f
    stosw
     
    xor ax,ax
    mov cx,0x07ff
    rep stosw
     
    mov ax,0x018f
    stosw
     
    xor ax,ax
    mov cx,0x07ff
    rep stosw
     
     
    ;Enter long mode
     
    mov eax,10100000b				;Set PAE and PGE
    mov cr4,eax
     
    mov edx, 0x0000a000				;Point CR3 at PML4
    mov cr3,edx
     
    mov ecx,0xC0000080				;Specify EFER MSR
     
    rdmsr						;Enable Long Mode
    or eax,0x00000100
    wrmsr
     
    mov ebx,cr0					;Activate long mode
    or ebx,0x80000001				;by enabling paging and protection simultaneously
    mov cr0,ebx					;skipping protected mode entirely
     
    lgdt [gdt.pointer]				;load 80-bit gdt.pointer below
     
    jmp gdt.code:startLongMode			;Load CS with 64 bit segment and flush the instruction cache
     
     
     
    ;Global Descriptor Table
    gdt:
    dq 0x0000000000000000				;Null Descriptor
     
    .code equ $ - gdt
    dq 0x0020980000000000                   
     
    .data equ $ - gdt
    dq 0x0000900000000000                   
     
    .pointer:
    dw $-gdt-1					;16-bit Size (Limit)
    dq gdt						;64-bit Base Address
                            ;Changed from "dd gdt"
                            ;Ref: Intel System Programming Manual V1 - 2.1.1.1
     
     
    [BITS 64]
     
    startLongMode:
     
    cli						;Interupts are disabled because no IDT has been set up
     
    mov edi,0x00b8000				;Display:"Put long mode kernel here.__"
    mov rax,0x0720077407750750                      ;at the top left corner of screen
    mov [edi],rax
    mov rax,0x0767076e076f076c
    mov [edi+8],rax
    mov rax,0x0764076f076d0720
    mov [edi+16],rax
    mov rax,0x0765076b07200765
    mov [edi+24],rax
    mov rax,0x076c0765076e0772
    mov [edi+32],rax
    mov rax,0x0772076507680720
    mov [edi+40],rax
    mov rax,0x07200720072e0765
    mov [edi+48],rax
     
    jmp $						;Hang the system
     
    times 510-($-$$) db 0				;Fill boot sector
    dw 0xAA55					;Boot loader signature
     
To use (in Linux):

    $ nasm -o boot boot.asm
    $ qemu-system-x86_64 -hda boot

[[User:Oculusfervoris|Oculusfervoris]] 06:31, 5 June 2011 (UTC)

== Not to put 64bit code in MBR ==

Oculusfervoris,

Your version is a good example i thinnk, but, there are some problems with
the idea of fitting 64bit code into MBR:

* 64bit code is usually quite large and you won't want to fit a portion 
(just a little little section of your code) into a 512byte trunk
* even you can run 64bit right from MBR, you still have to load more
code from hard disk, and since you can't access BIOS interrupts,
problems arise
* 64bit code is a whole different thing than 16bit, 32bit codes, so
people usually let it start at an even memory address, like being right
after MBR (at 7E00h), or wherever else.

--[[User:Paul84|Paul84]] 11:48, 5 June 2011 (UTC)

:This is proof-of-concept code, not an OS kernel or anything like that.[[User:Oculusfervoris|Oculusfervoris]] 15:58, 10 June 2011 (UTC)

::I agree. It should be "as minimal as possible" (so that the main concept is shown without any unnecessary stuff in the way); but it should also have warnings saying that it's deliberately "as minimal as possible", potentially including reasons why it's unusable "as is" for an OS (as per Paul84's comments). With this in mind, I'd be tempted to rip out the (unnecessary) "enable A20" code and most of the unnecessary segment loads from Oculusfervoris' version and use that. [[User:Brendan|Brendan]] 22:43, 10 June 2011 (UTC)

: I tried fixing the article - making it as minimal as possible, and even including a example boot sector. --[[User:Shikhin|Shikhin]] 12:38, 10 July 2011 (UTC)

----

It seems the SS selector must be writable (at least on some CPUs)

My CPU crashed on the "MOV SS,AX" instruction in long mode. I found out the reason was that the section

    .data equ $ - gdt
    dq 0x0000900000000000
              ^

was not writeable in the GDT!!! Changing the code to:

    .data equ $ - gdt
    dq 0x0000930000000000
              ^

resolved the problem. Maybe the code in the article must be changed in this way.

-- [[User:Martinro|Martinro]] 18:35, 10 November 2011 (UTC) 

----

== Setting CR4.PAE before loading CR3 ==

The Intel manual (http://www.intel.com/Assets/ja_JP/PDF/manual/253668.pdf) says on page 4-17:

<ul>
<li>If PAE paging would be in use following an execution of MOV to CR0 or MOV to 
CR4 (see Section 4.1.1) and the instruction is modifying any of CR0.CD, CR0.NW, 
CR0.PG, CR4.PAE, CR4.PGE, or CR4.PSE; then the PDPTEs are loaded from the 
address in CR3</li>
<li>If MOV to CR3 is executed while the logical processor is using PAE paging, the 
PDPTEs are loaded from the address being loaded into CR3.</li>
</ul>

Since you don't know the current value in CR3, wouldn't it be better to load CR3 before changing CR4 to enable PAE?
Else the CPU could e.g. be accessing a non-existing physical memory location
````

<a id="w002575"></a>

## W002575 — Talk:C language Example

Bron-ID: W002575
Namespace: Talk (1)
Revisie: 7784 | Datum: 2009-04-24T03:20:56Z
Bron: https://wiki.osdev.org/Talk:C_language_Example
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AC_language_Example&oldid=7784

````text
== Basically an un-written bare bones that will probably be filled full of factual errors? ==

Could I have some feedback on this, particularly from the author and the community? I personally think this will be just another thing the new kids don't need, including the author.

''"Our page is aiming to to give C language Exanple that are compiled in linux."''

Oh, like [http://www.jamesmolloy.co.uk JamesM's tutorials?]
 --[[User:Troy martin|Troy Martin]] 02:54, 24 April 2009 (UTC)

This looks pretty horrible to me. In my opinion, Epic Fail.
-[[User:Earlz|Earlz]] 03:08, 24 April 2009 (UTC)
: Apparently, taken care of by booblover! :) --[[User:Troy martin|Troy Martin]] 03:20, 24 April 2009 (UTC)
````

<a id="w002576"></a>

## W002576 — BGA

Bron-ID: W002576
Namespace: Main (0)
Revisie: 9616 | Datum: 2010-01-11T14:54:37Z
Bron: https://wiki.osdev.org/BGA
Revisie-URL: https://wiki.osdev.org/index.php?title=BGA&oldid=9616
Doorverwijzing: Bochs VBE Extensions

````text
#REDIRECT [[Bochs_VBE_Extensions]]
````

<a id="w002577"></a>

## W002577 — Talk:Building GMP

Bron-ID: W002577
Namespace: Talk (1)
Revisie: 8187 | Datum: 2009-06-08T04:48:45Z
Bron: https://wiki.osdev.org/Talk:Building_GMP
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABuilding_GMP&oldid=8187

````text
I am having trouble witht this tutorial.
I am getting a version difference between gmp.h and libgmp when configuring mpfr, this might be due to 
me currently having gmp & mpfr installed from cygwin.

I have uninstalled all gmp & mpfr libs from the cygwin installation and will try again.

== Additional section needed? ==

This article needs a quick blurb on detecting the package version that is already installed, for both linux & cygwin. [[User:Bewing|Bewing]] 04:48, 8 June 2009 (UTC)
````

<a id="w002581"></a>

## W002581 — Talk:Inline Assembly/Examples

Bron-ID: W002581
Namespace: Talk (1)
Revisie: 11312 | Datum: 2011-03-22T15:55:58Z
Bron: https://wiki.osdev.org/Talk:Inline_Assembly/Examples
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AInline_Assembly%2FExamples&oldid=11312

````text
== Rollback ==

Rolled back edit by imate900 that added the following lgdt function, which is wrong for many reasons. I just did not have time to fix the example yet.
<pre>
void *lgdt(void *gdt, int entries)
{
 struct { unsigned short *length, void *base } gdtr_t;
 gdtr_t *gdtr;
 gdtr.length = entries;
 gdtr.base = gdt;
 asm("lgdt (%0)": : "p" (&gdtr));
 asm("mov %ax, 0x10");
 asm("mov %ds, %ax");
 asm("mov %es, %ax");
 asm("mov %fs, %ax");
 asm("mov %gs, %ax");
 asm("mov %ss, %ax");
 goto fix_cs;
fix_cs:
 return;
}</pre>
--[[User:Quok|quok]] 19:17, 2 May 2009 (UTC)

== Problems with article ==

I see some problems with this article:
# It only provides GCC inline assembly.
# It uses GCC-specific extensions although the "inline" keyword was introduced in C99.
# '''More importantly, everything here is more suited as a macro than inline assembly. It is important to understand that "inline" is just a compiler hint and may be ignored entirely as far as optimizations go (and probably will). The only thing one can be sure of is that a pointer to the function cannot be obtained.'''
# I/O support is directly supported in C (TR 1169).
It would be nice if someone explained why this article is neccessary.
--[[User:Love4boobies|Love4boobies]] 05:44, 10 March 2011 (UTC)

:# This article is an extension to [[Inline Assembly]], which reads "...this article describes the way it works in GCC since it is by far the most used compiler in the OS world." Feel free to add an example page for MSVC inline assembly if you want.
:# I agree there.
:# The "inline" in "inline assembly" has nothing to do with the "inline" ''keyword''. Any ASM source written in a C source file is "inline", as opposed to ASM source in an external .asm / .s source file. You might want to note that an external ASM function can ''never'' be inlined, as the compiler lacks information on register usage etc. - and for the sake of presentation, I prefer a function declared inline over a preprocessor macro anytime.
:# You probably refer to the header <iohw.h>, which was added with TR18015 and TR18037. GCC still does not even follow C99 by default. The <iohw.h> header is absent from the GCC 4.1.2 installation I have available for quick checking. As such, it is not a viable replacement for the information provided in this article (which aims at providing ''examples'' of the modus operandi, not copy & paste code snippets anyway).
:I hope this helps (some). -- [[User:Solar|Solar]] 12:57, 10 March 2011 (UTC)

::I am not confusing inline assembly with inline functions. However, note that all the inline assembly code is implemented via inlined functions, which is why I suggested macros instead. The inline keyword is the problem, not the assembly. Using functions has not advantage because the compiler itself cannot assemble and has the potential performance disadvantage I have already explained. --[[User:Love4boobies|Love4boobies]] 14:48, 10 March 2011 (UTC)
:::Functions > Macros. They are typed, more legible and improve debugging. If the compiler does inline such functions the performance difference is zero by definition. In professional C++, macro's are not done in favour of templates. For half of those reasons, I also prefer vim over emacs. For the other half, functions must stay. - [[User:Combuster|Combuster]] 15:08, 10 March 2011 (UTC)
:::Yes, Combuster, C type checking is of huge advantage in inline assembly which is not even performed by the compiler. --[[User:Love4boobies|Love4boobies]] 17:14, 10 March 2011 (UTC)
::::The C compiler ''will'' do type checking on the parameters of an inline function. Inline functions also allow ''you'' to decide whether you want them inlined (for speed) or not (for space, or debugging). But that is neither here nor there: Simply for ''didactic'' reasons I much prefer inline functions over macros. Replace them with macros if you have ''proven'' them not being inlined ''and'' being a performance problem. Or replace them with macros ''in your project'' on general principles. For presentation, I refuse to write macro code on the mere ''guess'' that an inline function might not get inlined. -- [[User:Solar|Solar]] 10:56, 21 March 2011 (UTC)
:::::For didactic reasons you prefer inline functions over macros, yet I need to prove that some compiler doesn't inline although you are perfectly aware that the C standard allows both behaviors? I think you should reconsider what is didactic---foolishly sticking to rules or actually understanding why they are there and how to use them to your advantage. Maybe you like writing code on a per-tool basis. However, I prefer standards-compliant code. This is almost as ridiculous as [http://forum.osdev.org/viewtopic.php?f=13&p=169259#p169291 this post], where you advocated for the craziest scheme just in order to work around the use of ''goto''. The reason for which it should usually be avoided is to that it renders code less readable---but making the code unreadable just in order to avoid it kind of defeats the purpose, doesn't it? Hopefully, you don't use this to avoid ''break'' and ''continue'' as well because in this particular situation they are no different from ''goto''. Not only is it unreadable since you need to scroll around to get to read some sequential code that logically is part of the same instruction stream, but it also suffers from the same '''technical''' difficulty with the ''inline'' keyword. --[[User:Love4boobies|Love4boobies]] 13:42, 22 March 2011 (UTC)
You are going ''way'' off-topic here. What I said elsewhere about "goto" hardly matters here. I say that presenting the examples as inline functions adds information (type of parameters and return values) as well as cleanliness of presentation. I also say that anyone who doesn't want to use inline functions ''despite'' the various advantages pointed out here, on the mere ''suspicion'' that they might not actually be inlined, can trivially convert the presented code into a macro. I also say that these here are ''examples'', not templates for anyone to copy&paste without some thinking of his own. You are free to disagree, but you asked for your fellow user's input, and you got it. Can we now bury the hatchet and work on the article instead of attacking each other? -- [[User:Solar|Solar]] 14:13, 22 March 2011 (UTC)

:Maybe a note would help but you are right that this wiki is not for production code; you're right in saying this is not production code. Also, sorry for going out of line in trying to prove my point regarding rules. Sometimes I just can't help being a bit of a douche. --[[User:Love4boobies|Love4boobies]] 14:30, 22 March 2011 (UTC)

::Don't worry about it. Happens to everyone once in a while. ;-)
::I did a general cleanup of the article, which was in ''pitiful'' state. That's more important than whether it's inline functions or macros. ;-)
::I assume you are more familiar with inline ASM in general; could you double-check the syntax of each example? I have a feeling there might be some bugs in there. The first 'io_wait()' looks funny, several 'asm' calls have only one colon, stuff like that. I'd also prefer 'uintX_t' instead of 'word' or 'unsigned short' and stuff like that (unless I get another flame from you for that ;-) ). -- [[User:Solar|Solar]] 14:41, 22 March 2011 (UTC)

:::There was nothing wrong with the io_wait labels but I changed the second anyway in order to avoid confusion. Having only one colon is okay as long as you only have output (the syntax is '':output:input:clobber''). With the exception of CPUID, everything looked fine but I also changed the way multi-instruction asm blocks were written so that the output doesn't contain all the instructions on a single line (this is only visible for people who want to see the assembly file generated by GCC). Regarding the uintX_t, I fully agree---however, I think there are quite a few articles that need to be modified because they don't use the freestanding headers, which I think is just silly. --[[User:Love4boobies|Love4boobies]] 15:55, 22 March 2011 (UTC)
````

<a id="w002582"></a>

## W002582 — CRC32

Bron-ID: W002582
Namespace: Main (0)
Revisie: 30503 | Datum: 2026-04-21T00:03:29Z
Bron: https://wiki.osdev.org/CRC32
Revisie-URL: https://wiki.osdev.org/index.php?title=CRC32&oldid=30503

````text
CRC32 is a checksum/hashing algorithm that is very commonly used in kernels and for Internet checksums.

There are two common polynomials in use:

* CRC32 uses <code>0x04c11db7</code> (inverted: <code>0xedb88320</code>)
* CRC32C uses <code>0x1edc6f41</code> (inverted: <code>0x82f63b78</code>)


==The Basic Algorithm==

Start with a 32bit checksum with all bits set (0xffffffff). This helps to give an output value other than 0 for an input string of "0" bytes.

In a loop:
Look up a "polynomial" (actually just a 32-bit value) in a table, based on the next piece of input data (usually a byte) and the low N bits of the previous CRC value (where N is the size of the data you are operating on--usually 8-bit bytes).

Shift the previous 32-bit CRC value down by N bits.

Exclusive-OR the "polynomial" together with the shifted CRC value to produce a new value.

After the end of the loop, exclusive-OR the calculated CRC value with 0xffffffff (this is identical to doing a binary NOT on the CRC value). This is the final CRC32 result.

==Building the Lookup Table==

This is a bit more complex of a process than actually calculating the checksum itself.

To build a 256-entry (-byte) lookup table, take the 8-bit index and bit-reflect all the bits in that byte. Shift it to the upper 8 bits of a 32-bit variable. Loop over those 8 bits. If the top (sign) bit is set, then shift the uint32_t up one bit and exclusive-OR it with the magic value 0x04C11DB7. Otherwise just shift the uint32_t up one bit. Then repeat. When the loop is complete, bit-reflect the entire uint32_t. This is the value in the table.

Or, you can simply avoid bit reflections this way: take the 8-bit index and cast it to a 32-bit variable. Loop over the lower byte. If the least significant bit (0x00000001) is set, then shift the uint32_t down one bit and exclusive-OR it with the value 0xEDB88320 (which is 0x04C11DB7 bit-reflected). Otherwise just shift it down one bit. Then repeat. When the loop is complete, the uint32_t is the value in the table.

==Example Code==

<syntaxhighlight lang="c">
// calculate a checksum on a buffer -- start address = p, length = bytelength
uint32_t crc32_byte(uint8_t *p, uint32_t bytelength)
{
	uint32_t crc = 0xffffffff;
	while (bytelength-- !=0) crc = poly8_lookup[((uint8_t) crc ^ *(p++))] ^ (crc >> 8);
	// return (~crc); also works
	return (crc ^ 0xffffffff);
}

//Fill the lookup table -- table = the lookup table base address
void crc32_fill(uint32_t *table){
        uint8_t index=0,z;
        do{
                table[index]=index;
                for(z=8;z;z--) table[index]=(table[index]&1)?(table[index]>>1)^0xEDB88320:table[index]>>1;
        }while(++index);
}

uint32_t poly8_lookup[256] =
{
 0, 0x77073096, 0xEE0E612C, 0x990951BA,
 0x076DC419, 0x706AF48F, 0xE963A535, 0x9E6495A3,
 0x0EDB8832, 0x79DCB8A4, 0xE0D5E91E, 0x97D2D988,
 0x09B64C2B, 0x7EB17CBD, 0xE7B82D07, 0x90BF1D91,
 0x1DB71064, 0x6AB020F2, 0xF3B97148, 0x84BE41DE,
 0x1ADAD47D, 0x6DDDE4EB, 0xF4D4B551, 0x83D385C7,
 0x136C9856, 0x646BA8C0, 0xFD62F97A, 0x8A65C9EC,
 0x14015C4F, 0x63066CD9, 0xFA0F3D63, 0x8D080DF5,
 0x3B6E20C8, 0x4C69105E, 0xD56041E4, 0xA2677172,
 0x3C03E4D1, 0x4B04D447, 0xD20D85FD, 0xA50AB56B,
 0x35B5A8FA, 0x42B2986C, 0xDBBBC9D6, 0xACBCF940,
 0x32D86CE3, 0x45DF5C75, 0xDCD60DCF, 0xABD13D59,
 0x26D930AC, 0x51DE003A, 0xC8D75180, 0xBFD06116,
 0x21B4F4B5, 0x56B3C423, 0xCFBA9599, 0xB8BDA50F,
 0x2802B89E, 0x5F058808, 0xC60CD9B2, 0xB10BE924,
 0x2F6F7C87, 0x58684C11, 0xC1611DAB, 0xB6662D3D,
 0x76DC4190, 0x01DB7106, 0x98D220BC, 0xEFD5102A,
 0x71B18589, 0x06B6B51F, 0x9FBFE4A5, 0xE8B8D433,
 0x7807C9A2, 0x0F00F934, 0x9609A88E, 0xE10E9818,
 0x7F6A0DBB, 0x086D3D2D, 0x91646C97, 0xE6635C01,
 0x6B6B51F4, 0x1C6C6162, 0x856530D8, 0xF262004E,
 0x6C0695ED, 0x1B01A57B, 0x8208F4C1, 0xF50FC457,
 0x65B0D9C6, 0x12B7E950, 0x8BBEB8EA, 0xFCB9887C,
 0x62DD1DDF, 0x15DA2D49, 0x8CD37CF3, 0xFBD44C65,
 0x4DB26158, 0x3AB551CE, 0xA3BC0074, 0xD4BB30E2,
 0x4ADFA541, 0x3DD895D7, 0xA4D1C46D, 0xD3D6F4FB,
 0x4369E96A, 0x346ED9FC, 0xAD678846, 0xDA60B8D0,
 0x44042D73, 0x33031DE5, 0xAA0A4C5F, 0xDD0D7CC9,
 0x5005713C, 0x270241AA, 0xBE0B1010, 0xC90C2086,
 0x5768B525, 0x206F85B3, 0xB966D409, 0xCE61E49F,
 0x5EDEF90E, 0x29D9C998, 0xB0D09822, 0xC7D7A8B4,
 0x59B33D17, 0x2EB40D81, 0xB7BD5C3B, 0xC0BA6CAD,
 0xEDB88320, 0x9ABFB3B6, 0x03B6E20C, 0x74B1D29A,
 0xEAD54739, 0x9DD277AF, 0x04DB2615, 0x73DC1683,
 0xE3630B12, 0x94643B84, 0x0D6D6A3E, 0x7A6A5AA8,
 0xE40ECF0B, 0x9309FF9D, 0x0A00AE27, 0x7D079EB1,
 0xF00F9344, 0x8708A3D2, 0x1E01F268, 0x6906C2FE,
 0xF762575D, 0x806567CB, 0x196C3671, 0x6E6B06E7,
 0xFED41B76, 0x89D32BE0, 0x10DA7A5A, 0x67DD4ACC,
 0xF9B9DF6F, 0x8EBEEFF9, 0x17B7BE43, 0x60B08ED5,
 0xD6D6A3E8, 0xA1D1937E, 0x38D8C2C4, 0x4FDFF252,
 0xD1BB67F1, 0xA6BC5767, 0x3FB506DD, 0x48B2364B,
 0xD80D2BDA, 0xAF0A1B4C, 0x36034AF6, 0x41047A60,
 0xDF60EFC3, 0xA867DF55, 0x316E8EEF, 0x4669BE79,
 0xCB61B38C, 0xBC66831A, 0x256FD2A0, 0x5268E236,
 0xCC0C7795, 0xBB0B4703, 0x220216B9, 0x5505262F,
 0xC5BA3BBE, 0xB2BD0B28, 0x2BB45A92, 0x5CB36A04,
 0xC2D7FFA7, 0xB5D0CF31, 0x2CD99E8B, 0x5BDEAE1D,
 0x9B64C2B0, 0xEC63F226, 0x756AA39C, 0x026D930A,
 0x9C0906A9, 0xEB0E363F, 0x72076785, 0x05005713,
 0x95BF4A82, 0xE2B87A14, 0x7BB12BAE, 0x0CB61B38,
 0x92D28E9B, 0xE5D5BE0D, 0x7CDCEFB7, 0x0BDBDF21,
 0x86D3D2D4, 0xF1D4E242, 0x68DDB3F8, 0x1FDA836E,
 0x81BE16CD, 0xF6B9265B, 0x6FB077E1, 0x18B74777,
 0x88085AE6, 0xFF0F6A70, 0x66063BCA, 0x11010B5C,
 0x8F659EFF, 0xF862AE69, 0x616BFFD3, 0x166CCF45,
 0xA00AE278, 0xD70DD2EE, 0x4E048354, 0x3903B3C2,
 0xA7672661, 0xD06016F7, 0x4969474D, 0x3E6E77DB,
 0xAED16A4A, 0xD9D65ADC, 0x40DF0B66, 0x37D83BF0,
 0xA9BCAE53, 0xDEBB9EC5, 0x47B2CF7F, 0x30B5FFE9,
 0xBDBDF21C, 0xCABAC28A, 0x53B39330, 0x24B4A3A6,
 0xBAD03605, 0xCDD70693, 0x54DE5729, 0x23D967BF,
 0xB3667A2E, 0xC4614AB8, 0x5D681B02, 0x2A6F2B94,
 0xB40BBE37, 0xC30C8EA1, 0x5A05DF1B, 0x2D02EF8D
};
</syntaxhighlight>

<syntaxhighlight lang="asm">
; Note: _poly8_lookup is the symbolic address of the same table from the above example
; Note2: to append to a precalculated crc, NOT the crc, then call asm_partial_crc32_byte

asm_crc32_byte:
	mov eax, 0xffffffff		; eax is running crc value
asm_partial_crc32_byte:			; entrypoint if modifying a precalculated crc
	push esi
	push ecx
	push ebx
	push edx
	mov ebx, _poly8_lookup		; addy of crc table
	xor edx, edx
	jecxz crcb_don
crc_bl1:
	mov dl, al			; dx is the low byte of crc
	xor dl, [esi]			; xor it with "*p"
	inc esi				; ++
	shr eax, 8			; high 3 bytes of crc for xor
	xor eax, [ebx+edx*4]		; xor it with the polytable
	loop crc_bl1			; decrement ecx and loop

crcb_don:
	not eax				; finally, flip all the crc bits
	pop edx
	pop ebx
	pop ecx
	pop esi
	ret
</syntaxhighlight>

<syntaxhighlight lang="asm">
; function input values: esi points to a 256 entry bit reflection table, aligned on a 256 byte boundary
build_16b_crc_tbl:
	mov cx, 0xffff
	mov edi, Tbl_End		; store going down from Tbl_End
.crc_tbll:
	mov ebx, esi
	xor eax, eax
	mov bl, cl
	mov al, [ebx]			; bit-reflect the two input bytes
	mov bl, ch
	mov ah, [ebx]
	bswap eax			; and then put them at upper end of register

	mov dl, 16			; loop over all 16 bits in upper eax
.crc_bitl:
	shl eax, 1
	jnc .crc_nply
	xor eax, 0x04C11DB7		; if sign bit was set, xor eax with the "poly"
.crc_nply:
	dec dl
	jg short .crc_bitl

	mov ebx, esi
	mov bl, ah
	mov ah, [ebx]		; lastly, bit-reflect the value in eax
	mov bl, al
	mov al, [ebx]		; start with ax
	bswap eax
	mov bl, ah		; bswap to put the bytes in the right place, and do ax again
	mov ah, [ebx]
	mov bl, al
	mov al, [ebx]
	lea edi, [edi - 4]
	mov [edi], eax		; store next crc in the CRC32 lookup table
	dec ecx
	jge short .crc_tbll	; loop until ecx goes negative
	ret
</syntaxhighlight>

== Without table ==

A table is usually much faster but requires substantially more memory.

If performance is not a concern, the table can be omitted.

<syntaxhighlight lang="asm">
; rsi: data base
; rcx: data len (in bytes)
;
; eax: result
crc32c:
.POLY = 0x82f63b78
        mov eax, -1
        jrcxz .e
        lea rdi, [rsi + rcx]
.l:     xor al, byte [rsi]
        inc rsi
        mov ecx, 8
@@:     mov edx, eax
        and edx, 1
        neg edx
        and edx, .POLY
        shr eax, 1
        xor eax, edx
        loop @b
        cmp rdi, rsi
        jne .l
.e:     not eax
        ret

test:
.input: db "123456789"
.output: dd 0xe3069283
</syntaxhighlight>

==See Also==
===External Links===

* [http://www.ross.net/crc/crcpaper.html A detailed theoretical explanation of CRC algorithms]

* [https://web.archive.org/web/20140621155552/http://www34.brinkster.com/dizzyk/math-crc.asp The mathematics behind the CRC32 algorithm]

[[Category:Common Algorithms]]
````

<a id="w002584"></a>

## W002584 — Category:Common Algorithms

Bron-ID: W002584
Namespace: Category (14)
Revisie: 7835 | Datum: 2009-05-02T23:00:33Z
Bron: https://wiki.osdev.org/Category:Common_Algorithms
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ACommon_Algorithms&oldid=7835

````text
[[Category:OS theory]]
````

<a id="w002585"></a>

## W002585 — User talk:Imate900/Microkernel Bare Bones

Bron-ID: W002585
Namespace: User talk (3)
Revisie: 7841 | Datum: 2009-05-04T23:02:58Z
Bron: https://wiki.osdev.org/User_talk:Imate900/Microkernel_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AImate900%2FMicrokernel_Bare_Bones&oldid=7841

````text
How on earth are you going to manage to create a *microkernel* bare bones? Microkernels require address space seperation and nonrudimentary IPC! Loading a flat binary that performs a loop and no kind of communication or server function is *not* a microkernel. [[User:Jamesm|Jamesm]] 16:52, 3 May 2009 (UTC)
: It is currently in progress. The looping binary is a task that runs on the kernel. [[User:Imate900|Imate900]] 21:14, 3 May 2009 (UTC)
:: I must be hallucinating, because I don't see a kernel of any monolithic or micro- type here. [[User:Troy martin|Troy Martin]] 23:02, 4 May 2009 (UTC)
````

<a id="w002586"></a>

## W002586 — Writing A Page Frame Allocator

Bron-ID: W002586
Namespace: Main (0)
Revisie: 30274 | Datum: 2026-03-16T17:52:28Z
Bron: https://wiki.osdev.org/Writing_A_Page_Frame_Allocator
Revisie-URL: https://wiki.osdev.org/index.php?title=Writing_A_Page_Frame_Allocator&oldid=30274

````text
{{Rating|2}}
{{In_Progress}}

==Introduction==
This tutorial will attempt to show you how to write a simple page frame allocator for the x86 CPU. The language used is C and we are 
using standard paging with 4 KiB pages. The page frame allocator will allocate frames with the first frame starting right after the end of the kernel.

==The Method==
Each frame shall be managed with a byte map (for the sake of simplicity): a value of 0x01 for used pages and a value of 0x00 for unused pages. To
allocate a page, all that is needed is to search through the array for a free page and then mark it as used. You may have noticed that
this would be very inefficient, having to search through a possible number of 1048319 pages. To speed up the allocation process the allocator will
allocate 20 page frames at a time, so most of the time all that will have been done before hand. Alocatting a new page is simply a matter of getting a page frame off an array of preallocated frames.

==The Implementation==
First of all, we will need something in the linker script to tell us where the end of our kernel is.

<pre>
ENTRY (loader)

SECTIONS{
    . = 0x00100000;

    .text :{
	text_start = .;
        *(.text)
    }

    .rodata ALIGN (0x1000) : {
        *(.rodata)
    }

    .data ALIGN (0x1000) : {

        *(.data)
        end_data = .;
    }

    .bss : {
        sbss = .;
        *(COMMON)
        *(.bss)
        ebss = .;
	endkernel = .;
    }
}

</pre>

The variable endkernel will be declared in the kernel as:

<syntaxhighlight lang="c">
extern uint32_t endkernel;
</syntaxhighlight>

The variable itself has no value, it is the address of the variable that we are using. endkernel will be used to calculate the address of the first page frame
after the kernel. The code for searching through the array is also very simple:

<syntaxhighlight lang="c">
static pageframe_t kalloc_frame_int()
{
        uint32_t i = 0;
        while(frame_map[i] != FREE)
        {
                i++;
                if(i == npages)
                {
                        return(ERROR);
                }
        }
        frame_map[i] = USED;
        return(startframe + (i*0x1000));//return the address of the page frame based on the location declared free
        //in the array
}
</syntaxhighlight> 


The last function used calls kalloc_frame_int every 20 page frame allocations:


<syntaxhighlight lang="c">

pageframe_t kalloc_frame()
{
        static uint8_t allocate = 1;//whether or not we are going to allocate a new set of preframes
        static uint8_t pframe = 0;
        pageframe_t ret;
        
        if(pframe == 20)
        {
                allocate = 1;
        }

        if(allocate == 1)
        {
                for(int i = 0; i<20; i++)
                {
                        pre_frames[i] = kalloc_frame_int();
                }
                pframe = 0;
                allocate = 0;
        }
        ret = pre_frames[pframe];
        pframe++;
        return(ret);
}
</syntaxhighlight>

Freeing the page frame is simply a matter of reversing the process used to get the page frame.

<syntaxhighlight lang="c">
void kfree_frame(pageframe_t a)
{
                a = a - startframe;//get the offset from the first frame
                if(a == 0)//in case it is the first frame we are freeing
                {
                       uint32_t index = (uint32_t)a;
                       frame_map[index] = FREE;    
                }
                else{ 
                       a = a;//divide by 4kb to get the index to declare free
                       uint32_t index = ((uint32_t)a)/0x1000;
                       frame_map[index] = FREE;
                    }
}
</syntaxhighlight>

== See Also ==
* [[Page Frame Allocation]]

[[Category:Tutorials]]
[[Category:Memory management]]
````

<a id="w002587"></a>

## W002587 — User:Oros

Bron-ID: W002587
Namespace: User (2)
Revisie: 7864 | Datum: 2009-05-09T03:24:47Z
Bron: https://wiki.osdev.org/User:Oros
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AOros&oldid=7864

````text
Nom Nom Nom...
````

<a id="w002588"></a>

## W002588 — Talk:CPUID

Bron-ID: W002588
Namespace: Talk (1)
Revisie: 30268 | Datum: 2026-03-16T17:46:13Z
Bron: https://wiki.osdev.org/Talk:CPUID
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ACPUID&oldid=30268

````text
== Added x86-cpuid.org ==
I've modified the external links section and added [https://x86-cpuid.org x86-cpuid.org], instead of SandPile, as the primary CPUID listings source.

This is what ''both'' the Linux Kernel and the Xen hypervisor will be using soon for all CPUID information (''835+ bits'', with metadata, are listed at the project's [https://gitlab.com/x86-cpuid.org/x86-cpuid-db repository]).  Thanks! --[[User:Darwish|Darwish]] ([[User talk:Darwish|talk]]) 21:38, 19 July 2024 (UTC)

== Updated CPUID flags ==
Anyone have any objection to replacing the CPUID flags with this updated one from Comos?
<syntaxhighlight lang="c">
enum {
    CPUID_FEAT_ECX_SSE3         = 1 << 0,  // streaming SIMD extensions 3 (SSE3)
    CPUID_FEAT_ECX_MONITOR      = 1 << 3,  // monitor/mwait
    CPUID_FEAT_ECX_DS_CPL       = 1 << 4,  // CPL qualified debug store
    CPUID_FEAT_ECX_VMX          = 1 << 5,  // virtual machine extensions
    CPUID_FEAT_ECX_SMX          = 1 << 6,  // safer mode extensions
    CPUID_FEAT_ECX_EST          = 1 << 7,  // enhanced Intel SpeedStep(R) technology
    CPUID_FEAT_ECX_TM2          = 1 << 8,  // thermal monitor 2
    CPUID_FEAT_ECX_SSSE3        = 1 << 9,  // supplemental streaming SIMD extensions 3 (SSSSE3)
    CPUID_FEAT_ECX_CNXT_ID      = 1 << 10, // L1 context ID
    CPUID_FEAT_ECX_CMPXCHG16B   = 1 << 13, // cmpxchg16b available (obviously)
    CPUID_FEAT_ECX_xTPR_UPDATE  = 1 << 14, // xTPR update control
    CPUID_FEAT_ECX_PDCM         = 1 << 15, // performance and debug capability
    CPUID_FEAT_ECX_DCA          = 1 << 18, // memory-mapped device prefetching
    CPUID_FEAT_ECX_SSE4_1       = 1 << 19, // SSE4.1
    CPUID_FEAT_ECX_SSE4_2       = 1 << 20, // SSE4.2
    CPUID_FEAT_ECX_x2APIC       = 1 << 21, // x2APIC available
    CPUID_FEAT_ECX_MOVBE        = 1 << 22, // movbe available
    CPUID_FEAT_ECX_POPCNT       = 1 << 23, // popcnt available (sounds rude)
    CPUID_FEAT_ECX_XSAVE        = 1 << 26, // xsave/xrstor/xsetbv/xgetbv supported
    CPUID_FEAT_ECX_OSXSAVE      = 1 << 27, // xsetbv/xgetbv has been enabled

    CPUID_FEAT_EDX_x87          = 1 << 0,  // x86 FPU on chip
    CPUID_FEAT_EDX_VME          = 1 << 1,  // virtual-8086 mode enhancement
    CPUID_FEAT_EDX_DE           = 1 << 2,  // debugging extensions
    CPUID_FEAT_EDX_PSE          = 1 << 3,  // page size extensions
    CPUID_FEAT_EDX_TSC          = 1 << 4,  // timestamp counter
    CPUID_FEAT_EDX_MSR          = 1 << 5,  // rdmsr/wrmsr
    CPUID_FEAT_EDX_PAE          = 1 << 6,  // page address extensions
    CPUID_FEAT_EDX_MCE          = 1 << 7,  // machine check exception
    CPUID_FEAT_EDX_CX8          = 1 << 8,  // cmpxchg8b supported
    CPUID_FEAT_EDX_APIC         = 1 << 9,  // APIC on a chip
    CPUID_FEAT_EDX_SEP          = 1 << 11, // sysenter/sysexit
    CPUID_FEAT_EDX_MTRR         = 1 << 12, // memory type range registers
    CPUID_FEAT_EDX_PGE          = 1 << 13, // PTE global bit (PTE_GLOBAL)
    CPUID_FEAT_EDX_MCA          = 1 << 14, // machine check architecture
    CPUID_FEAT_EDX_CMOV         = 1 << 15, // conditional move/compare instructions
    CPUID_FEAT_EDX_PAT          = 1 << 16, // page attribute table
    CPUID_FEAT_EDX_PSE36        = 1 << 17, // page size extension
    CPUID_FEAT_EDX_PSN          = 1 << 18, // processor serial number
    CPUID_FEAT_EDX_CLFSH        = 1 << 19, // cflush instruction
    CPUID_FEAT_EDX_DS           = 1 << 21, // debug store
    CPUID_FEAT_EDX_ACPI         = 1 << 22, // thermal monitor and clock control
    CPUID_FEAT_EDX_MMX          = 1 << 23, // MMX technology
    CPUID_FEAT_EDX_FXSR         = 1 << 24, // fxsave/fxrstor
    CPUID_FEAT_EDX_SSE          = 1 << 25, // SSE extensions
    CPUID_FEAT_EDX_SSE2         = 1 << 26, // SSE2 extensions, obviously
    CPUID_FEAT_EDX_SS           = 1 << 27, // self-snoop
    CPUID_FEAT_EDX_HTT          = 1 << 28, // multi-threading (hyper-threading, I think - the afterburner of Intel CPUs)
    CPUID_FEAT_EDX_TM           = 1 << 29, // thermal monitor
    CPUID_FEAT_EDX_PBE          = 1 << 31, // Pend. Brk. EN. (wtf?)
};
</syntaxhighlight>
--[[User:Oros|OrOS]] 19:22, 9 May 2009 (UTC)

No real objection here, but your code is still missing several of the newer flags as well. --[[User:Quok|quok]] 16:11, 10 May 2009 (UTC)

I updated the page already with a more complete version, but if you see anything missing go ahead and add them (I was sure I got them all ;( ) --[[User:Oros|OrOS]] 20:25, 10 May 2009 (UTC)

== Updated cpuid_string function (and example of how to use it) ==

The following is an updated cpuid_string function (which returns a string which is a bit more friendly to the user (this actually returns a readably null terminated string!)

<syntaxhighlight lang="c">
static inline int cpuid_string(int code, int where[4]) {
  __asm__ volatile ("cpuid":"=a"(*where),"=b"(*(where+0)),
               "=d"(*(where+1)),"=c"(*(where+2)):"a"(code));
  return (int)where[0];
}

const char * const cpu_string() {
	static char s[16] = "BogusProces!";
	cpuid_string(0, (int*)(s));
	return s;
}
</syntaxhighlight>

The cpuid_string function switches the content of c and d, compared to the original, which orders the string so that it is readable. The cpu_string function wraps the cpuid_string function so that it can simply be called (no memory allocation required) and the beginning of the string is returned. The string is aggressively const'ed to make sure you don't accidentally try to overwrite it.
````

<a id="w002589"></a>

## W002589 — User talk:Imate900

Bron-ID: W002589
Namespace: User talk (3)
Revisie: 7928 | Datum: 2009-05-10T03:25:19Z
Bron: https://wiki.osdev.org/User_talk:Imate900
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AImate900&oldid=7928

````text
I have noticed that quite a few contributions of yours to the wiki have been reverted lately. This has happened because of errors in your contributions. Please make certain that contributions of yours are correct, most especially in the area of source code. For example, there are several errors in your functions on the http://wiki.osdev.org/User:Imate900/Memory_Functions and http://wiki.osdev.org/User:Imate900/String_Functions pages.<br>
--[[User:Quok|quok]] 03:25, 10 May 2009 (UTC)
````

<a id="w002590"></a>

## W002590 — Talk:Writing A Page Frame Allocator

Bron-ID: W002590
Namespace: Talk (1)
Revisie: 7941 | Datum: 2009-05-11T14:17:34Z
Bron: https://wiki.osdev.org/Talk:Writing_A_Page_Frame_Allocator
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AWriting_A_Page_Frame_Allocator&oldid=7941

````text
Hehe, sorry about the "ad" I thought it might just clear things up a bit.
````

<a id="w002591"></a>

## W002591 — SciTE

Bron-ID: W002591
Namespace: Main (0)
Revisie: 9761 | Datum: 2010-02-13T02:50:39Z
Bron: https://wiki.osdev.org/SciTE
Revisie-URL: https://wiki.osdev.org/index.php?title=SciTE&oldid=9761

````text
'''SciTE''' is a multi-language, syntax highlighting, code completing, open-source editor (apparently with a build function.) SciTE supports assembly language, C, C++, bash scripting, Python, Perl, Ruby, HTML, XML, and many more. Unfortunately there is no highlighting for linker scripts or FreeBASIC.

You can download SciTE for free at http://www.scintilla.org/SciTEDownload.html

[[Category:IDEs]]
````

<a id="w002592"></a>

## W002592 — Talk:Gujin

Bron-ID: W002592
Namespace: Talk (1)
Revisie: 29408 | Datum: 2025-02-17T09:25:56Z
Bron: https://wiki.osdev.org/Talk:Gujin
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AGujin&oldid=29408

````text
Here is a first shoot, I am off in holidays for two weeks so may not answer quickly... - [[User:Etiennelorrain]] (please sign your posts)

== Deeplinking ==
I couldn't help notice that there are like 20 links into the SourceForge mirror service, most of them pointing into the same source archive. Are they all necessary? - [[User:Combuster|Combuster]] 22:59, 17 June 2010 (UTC)

== Location for other Linuxs == 
On sourceforge there are packages for many Linux distros, but it's not obvious where the "plain" binaries for unsupported linux distros. They are located in install.tar.gz --[[User:Earlz|Earlz]] 16:01, 22 September 2013 (CDT)

== Abandonware & no cross-compiler ==

The bootloader hasn't been updated since 2013. The tutorial doesn't use a cross-compiler. I doubt this tutorial is useful and adds value. I suggest we delete it. Any opinions? --[[User:Sortie|Sortie]] ([[User talk:Sortie|talk]]) 22:24, 29 September 2024 (UTC)

I agree. There are major problems with the article, no recent releases, and no Bootloader features of huge value (either [[GRUB]] or [[Limine]] would be a better choice). --[[User:Jackscott|JackScott]] ([[User talk:Jackscott|talk]]) 09:25, 17 February 2025 (UTC)
````

<a id="w002593"></a>

## W002593 — ACPICA

Bron-ID: W002593
Namespace: Main (0)
Revisie: 29487 | Datum: 2025-04-25T13:37:02Z
Bron: https://wiki.osdev.org/ACPICA
Revisie-URL: https://wiki.osdev.org/index.php?title=ACPICA&oldid=29487

````text
{{ACPI}}

The ACPI Component Architecture '''ACPICA''' provides an operating system (OS)-independent reference implementation of the [[ACPI|Advanced Configuration and Power Interface]]. It can be adapted to any host OS. The ACPICA code is meant to be directly integrated into the host OS, as a kernel-resident subsystem. Hosting the ACPICA requires no changes to the core ACPICA code. However, it does require a small OS-specific interface layer, which must be written specifically for each host OS.

The complexity of the ACPI specification leads to a lengthy and difficult implementation in OS software. The purpose of the ACPI Component Architecture is to simplify ACPI implementations for operating system vendors (OSVs) by providing major portions of an ACPI implementation in OS-independent ACPI modules that can be easily integrated into any OS.

As said before you need to implement yourself a few functions that are part of the OS interface layer (OSL). Here are those functions:

== OS Layer ==
There are at least 45 functions to be implemented (luckily most of them are really simple):

=== Environmental and ACPI Tables ===
==== AcpiOsInitialize ====
 ACPI_STATUS AcpiOsInitialize()
This is called during ACPICA Initialization. It gives the possibility to the OSL to initialize itself. Generally it should do nothing
==== AcpiOsTerminate ====
 ACPI_STATUS AcpiOsTerminate()
This is called during ACPICA Shutdown (which is not the computer shutdown, just the ACPI). Here you can free any memory which was allocated in AcpiOsInitialize.

==== AcpiOsGetRootPointer ====
 ACPI_PHYSICAL_ADDRESS AcpiOsGetRootPointer()
ACPICA leaves to you the job of finding the RSDP for platform compatibility.

Note: The ACPI specification is highly portable specification, however, it has a static part which is generally non-portable: the location of the Root System Descriptor Pointer. This pointer may be found in many different ways depending on the chipset. On PC-compatible computers (without EFI) it is located in lower memory generally somewhere between 0x80000 and 0x100000. However, even within the PC compatible platform, an EFI-enabled board will export the RSDP to the OS on when it loads it through the EFI system tables. Other boards on server machines which are not PC-compatibles, like embedded and handheld devices which implement ACPI will again, not all be expected to position the RSDP in the same place as any other board. The RSDP is therefore located in a chipset-specific manner; From the time the OS has the RSDP, the rest of ACPI is completely portable. However, the way the RSDP is found is not. 

This would be the reason that the ACPICA code wouldn't try to provide routines to expressly find the RSDP in a portable manner. If your system uses EFI, locate it in the system tables or use Multiboot2 compliant loader, which provides the RSDP for you.

There are a couple of ways of doing this:

===== UEFI =====
The UEFI System Table contains a pointer to an array of '''Configuration Tables'''. You just need to find the entry matching the [[RSDP]], which has a different GUID for the ACPI 1.0 format, vs ACPI 2.0 or later.<syntaxhighlight lang="cpp">
static const EFI_GUID acpi_20 = { 0x8868e871,0xe4f1,0x11d3,\
{0xbc,0x22,0x00,0x80,0xc7,0x3c,0x88,0x81} };

static const EFI_GUID acpi_10 = { 0xeb9d2d30,0x2d88,0x11d3,\
{0x9a,0x16,0x00,0x90,0x27,0x3f,0xc1,0x4d} };

inline static bool operator ==(const EFI_GUID& lhs, const EFI_GUID& rhs) {
    return memcmp(&lhs, &rhs, sizeof(EFI_GUID)) == 0;
}

void* uefi_find_rsdp(EFI_SYSTEM_TABLE* ST)
{
	void* bestacpi = nullptr;
	for (size_t n = 0; n < ST->NumberOfTableEntries; ++n)
	{
	    if (ST->ConfigurationTable[n].VendorGuid == acpi_20)
		{
			bestacpi = ST->ConfigurationTable[n].VendorTable;
			break;
		}
		else if (ST->ConfigurationTable[n].VendorGuid == acpi_10)
		{
			bestacpi = ST->ConfigurationTable[n].VendorTable;
		}
	}
	return bestacpi;
}
</syntaxhighlight>

===== Multiboot 2 =====
The [[Multiboot#Multiboot 2|Multiboot 2]] specification allows for the ACPI RSDP to be passed as a tag.

===== Bare Metal - X86 =====
On non-UEFI X86 systems, the RSDP can be found by searching the first megabyte of memory. More information can be found at [[RSDP]], but ACPICA does provide a helper method that does this for you, '''AcpiFindRootPointer.'''

So on a BIOS X86 boot, you can just do:<syntaxhighlight lang="cpp">
ACPI_PHYSICAL_ADDRESS AcpiOsGetRootPointer() { 
    ACPI_PHYSICAL_ADDRESS Ret = 0; 
    AcpiFindRootPointer(&Ret); 
    return Ret;
}
</syntaxhighlight>

===== Other Systems =====
The x86 and UEFI mechanisms are specified in the ACPI specification. There is no standard for standalone providing of ACPI information in other architectures - in all likelihood, such a system will use UEFI anyway.

==== AcpiOsPredefinedOverride ====
 ACPI_STATUS AcpiOsPredefinedOverride(const ACPI_PREDEFINED_NAMES *PredefinedObject, ACPI_STRING *NewValue)
This function allows the host to override the predefined objects in the ACPI namespace. It is called when a new object is found in the ACPI namespace. However you can just put NULL in *NewValue and return.

==== AcpiOsTableOverride ====
 ACPI_STATUS AcpiOsTableOverride(ACPI_TABLE_HEADER *ExistingTable, ACPI_TABLE_HEADER **NewTable)
The same of AcpiOsPredefinedOverride but for entire ACPI tables. You can replace them. Just put NULL in *NewTable and return.

=== Memory Management ===
==== AcpiOsMapMemory ====
 void *AcpiOsMapMemory(ACPI_PHYSICAL_ADDRESS PhysicalAddress, ACPI_SIZE Length)
This is not really easy. ACPICA is asking you to map a physical address in the virtual address space. If you don't use paging, just return PhysicalAddress. You need:
# To round Length up to the size of a page (Length can be 2, 1024 for example)
# Find a range of virtual addresses where map the physical frames.
# Map the physical frames to the virtual addresses choosen.
# Return the virtual address plus the page offset of the physical address. (Eg. If you where asked to map 0x40E you have to return 0xF000040E and not just 0xF0000000)
==== AcpiOsUnmapMemory ====
 void AcpiOsUnmapMemory(void *where, ACPI_SIZE length)
Unmap pages mapped using AcpiOsMapMemory. Where is the Virtual address returned in AcpiOsMapMemory and length is equal to the length of the same function. Just remove the virtual address form the page directory and set that virtual address as reusable.
'''Note:''' for the last two functions you might need a separated heap.

==== AcpiOsGetPhysicalAddress ====
 ACPI_STATUS AcpiOsGetPhysicalAddress(void *LogicalAddress, ACPI_PHYSICAL_ADDRESS *PhysicalAddress)
Get the physical address pointed by LogicalAddress and put it in *PhysicalAddress. If you do not use paging just put LogicalAddress in *PhysicalAddress
==== AcpiOsAllocate ====
 void *AcpiOsAllocate(ACPI_SIZE Size);
Dynamically allocate memory in the heap. Return NULL on error or end of memory. In other words:
 return malloc(size)
==== AcpiOsFree ====
 void AcpiOsFree(void *Memory);
Free previously dynamically allocated memory.
==== AcpiOsReadable ====
 BOOLEAN AcpiOsReadable(void *Memory, ACPI_SIZE Length)
Check that the memory from Memory to (Memory + Length) is readable. This is never used (at least i did never see it used). However it should (on x86(_64)) check that a the locations of memory are present in the paging structure.
==== AcpiOsWritable ====
 BOOLEAN AcpiOsWritable(void *Memory, ACPI_SIZE Length)
Check that the memory from Memory to (Memory + Length) is writable. This is never used (at least i did never see it used). However it should (on x86(_64)) check that a the locations of memory are present in the paging structure and that they have the Write flag set.
==== Caches ====
ACPICA uses caches to make things faster. You can use your kernel's cache or just let ACPICA use its internal cache. To do so just define in your platform/ac''whatever''.h file
 #define ACPI_CACHE_T                ACPI_MEMORY_LIST
 #define ACPI_USE_LOCAL_CACHE        1
=== Multithreading and Scheduling Services ===
To use all the features of ACPICA you need Scheduling support too. ACPICA specifies Threads but if you have only processes, that should work too. If you don't have and don't plan to have a scheduler, you can only use the Table features of ACPICA.
==== AcpiOsGetThreadId ====
 ACPI_THREAD_ID AcpiOsGetThreadId()
Return the unique ID of the running thread. In the Linux implementation:
 return pthread_self();
==== AcpiOsExecute ====
 ACPI_STATUS AcpiOsExecute(ACPI_EXECUTE_TYPE Type, ACPI_OSD_EXEC_CALLBACK Function, void *Context)
Create a new thread (or process) with entry point at ''Function'' using parameter ''Context''. ''Type'' is not really useful. When the scheduler chooses this thread it has to pass in ''Context'' to the first argument (RDI for x86-64, stack for x86-32 (using System V ABI) to have something like:
 Function(Context);

==== AcpiOsSleep ====
 void AcpiOsSleep(UINT64 Milliseconds)
Put the current thread to sleep for ''n'' milliseconds.
==== AcpiOsStall ====
 void AcpiOsStall(UINT32 Microseconds)
Stall the thread for ''n'' microseconds. Note: this should not put the thread in the sleep queue. The thread should keep on running. Just looping.

=== Mutual Exclusion and Synchronization ===
Yes, you need Spinlocks, Mutexes and Semaphores too. Nobody said it was easy. :)
==== AcpiOsCreateMutex ====
 ACPI_STATUS AcpiOsCreateMutex(ACPI_MUTEX *OutHandle)
Create space for a new Mutex using malloc (or eventually new) and put the address of the Mutex in *OutHandle, return AE_NO_MEMORY if malloc or new return NULL. Else return AE_OK like in most other functions.

==== AcpiOsDeleteMutex ====
 void AcpiOsDeleteMutex(ACPI_MUTEX Handle)
This is far too complex to be explained here, so I'll leave this to your imagination.
==== AcpiOsAcquireMutex ====
 ACPI_STATUS AcpiOsAcquireMutex(ACPI_MUTEX Handle, UINT16 Timeout)
This would be silly too if not for the Timeout parameter. Timeout can be one of:
* 0: acquire the Mutex if it is free, but do not wait if it is not
* 1 - +inf: acquire the Mutex if it is free, but wait for ''Timeout'' milliseconds if it is not
* -1 (0xFFFF): acquire the Mutex if it is free, or wait until it became free, then return

==== AcpiOsReleaseMutex ====
 void AcpiOsReleaseMutex(ACPI_MUTEX Handle)
Do you need explaination?
==== AcpiOsCreateSemaphore ====
 ACPI_STATUS AcpiOsCreateSemaphore(UINT32 MaxUnits, UINT32 InitialUnits, ACPI_SEMAPHORE *OutHandle)
Create a new Semaphore with the counter initialized to ''InitialUnits'' and put its address in *OutHandle. I don't know how tu use MaxUnits. The spec says: The maximum number of units this Semaphore will be required to accept.<br> However you should be ok if you ignore this.

==== AcpiOsDeleteSemaphore ====
 ACPI_STATUS AcpiOsDeleteSemaphore(ACPI_SEMAPHORE Handle)
-_-

==== AcpiOsWaitSemaphore ====
 ACPI_STATUS AcpiOsWaitSemaphore(ACPI_SEMAPHORE Handle, UINT32 Units, UINT16 Timeout)
Just like AcpiOsAcquireMutex, same logic for ''Timeout''. Units isn't used in the linux implementation. However it ''should'' be the number of times you have to call sem_wait. I'm not sure about this. Someone should check
==== AcpiOsSignalSemaphore ====
 ACPI_STATUS AcpiOsSignalSemaphore(ACPI_SEMAPHORE Handle, UINT32 Units)
Opposite of Wait. ''Units'': number of times you ''should'' call sem_post.
==== AcpiOsCreateLock ====
 ACPI_STATUS AcpiOsCreateLock(ACPI_SPINLOCK *OutHandle)
Create a new spinlock and put its address in *OutHandle. Spinlock should disable interrupts on the current CPU to avoid scheduling and make sure that no other CPU will access the reserved area.

==== AcpiOsDeleteLock ====
 void AcpiOsDeleteLock(ACPI_HANDLE Handle)
==== AcpiOsAcquireLock ====
 ACPI_CPU_FLAGS AcpiOsAcquireLock(ACPI_SPINLOCK Handle)
Lock the spinlock and return a value that will be used as parameter for ReleaseLock. It is mainly used for the status of interrupts before the lock was acquired.
==== AcpiOsReleaseLock ====
 void AcpiOsReleaseLock(ACPI_SPINLOCK Handle, ACPI_CPU_FLAGS Flags)
Release the lock. ''Flags'' is the return value of AcquireLock. If you used this to store the interrupt state, now is the moment to use it.

=== Interrupt Handling ===
==== AcpiOsInstallInterruptHandler ====
 ACPI_STATUS AcpiOsInstallInterruptHandler(UINT32 InterruptLevel, ACPI_OSD_HANDLER Handler, void *Context)
ACPI sometimes fires interrupt. ACPICA will take care of them. ''InterruptLevel'' is the IRQ number that ACPI will use. Handler is an internal function of ACPICA which handles interrupts. Context is the parameter to be past to the Handler.
If you're lucky, your IRQ manager uses handlers of this form:
 uint32_t handler(void *);
In this case just assign the handler to the IRQ number with that context. I wasn't as lucky so I did:
 #include <Irq.h>
 ACPI_OSD_HANDLER ServiceRout;
 
 InterruptState *AcpiInt(InterruptState *ctx, void *Context)
 {
 	ServiceRout(Context);
 	return ctx;
 }
 
 UINT32 AcpiOsInstallInterruptHandler(UINT32 InterruptNumber, ACPI_OSD_HANDLER ServiceRoutine, void *Context)
 {
 	ServiceRout = ServiceRoutine;
 
 	RegisterIrq(InterruptNumber, AcpiInt, Contex);
 	return AE_OK;
 }

==== AcpiOsRemoveInterruptHandler ====
 ACPI_STATUS AcpiOsRemoveInterruptHandler(UINT32 InterruptNumber, ACPI_OSD_HANDLER Handler)
Just UnregisterIrq (InterruptNumber). Handler is provided in case you have an IRQ manager which can have many handlers for one IRQ. This would let you know which handler on that IRQ you have to remove.

== Using ACPICA in your OS ==

I didn't find any good description of integrating the ACPICA source code into an operating system, and the released package is basically just a bundle of C files with little organization.  This is what I ended up having to do:

# I copied the C files from dispatcher/, events/, executer/, hardware/, parser/, namespace/, utilities/, tables/, and resources/ into a single acpi folder.
# I copied the header files from include/
# I created my own header file based on aclinux.h where I ripped out all of the userspace stuff, then I changed around the rest to be appropriate to my OS's definitions.
# I edited the include/platform/acenv.h file to remove the inclusion of aclinux.h and included my header file instead.
# I copied over acenv.h, acgcc.h, and my header file over to my include/platform/ folder.

This is in addition to writing an AcpiOs interface layer, and it is not well indicated by the reference manual that you have to actually edit header files.  Many of the macros defined in the headers are documented, though.

=== Visual Studio experience ===
From Visual Studio, although there is little organization in the files, it is relatively easy to port. In the provided/generate directory, there is a VC 9.0 solution. The only project required for integration is "AcpiSubsystem". Copy this project along with all the files listed (you can keep the old directory structure). #define's can be used to configure certain aspects of it, and perhaps changing #ifdef WIN32 to #ifdef X86 might be a good idea (Win64 -> x64). Once this is done though the base of it is in place, and actypes.h is the only header file that needs any modification (that listed above). It might be an idea to change the option "Compile as C code" to default - it's all .c anyway. This allows you to add C++ to the project without problems. Once this is done, add OSL.c or OSLPP.cpp, write your OS layer and you are done.

== Code Examples ==
Here are some examples of different operations that ACPICA can help with.

=== Power Off ===
To power off the machine:
 AcpiEnterSleepStatePrep(5);
 cli(); // disable interrupts
 AcpiEnterSleepState(5);
 panic("power off"); // in case it didn't work!

== External links ==
*[https://acpica.org/ ACPICA Website]
*[https://cdrdv2.intel.com/v1/dl/getContent/772726 ACPICA User Guide and Programmer Reference]. Explains how to integrate it in your OS. Any missing OS layer that isn't documented in this page, must be there.

[[Category:ACPI]]
````

<a id="w002595"></a>

## W002595 — Emulators

Bron-ID: W002595
Namespace: Main (0)
Revisie: 16270 | Datum: 2014-04-24T05:34:09Z
Bron: https://wiki.osdev.org/Emulators
Revisie-URL: https://wiki.osdev.org/index.php?title=Emulators&oldid=16270

````text
{{Emulators}}

An emulator is a device, computer program, or system that accepts the same inputs and produces the same outputs of another device, computer program or system.

Emulators are available for everything from running Windows on the Mac OS to running Game Boy games on PS3s. In the context of kernel development however we want a generic emulator which emulates the environment you would like your kernel to run in.

== Emulation Packages ==
=== QEMU ===
[[QEMU]] is operated from the command line with the parameters used to specify the hardware environment of the system and also to link to the image which holds your kernel binary. It's available for Linux and Windows (the later as 'kqemu').

A simple command to run a floppy image would be:

<source lang="bash">
qemu -fda floppy.img
</source>

You can use Ctrl+Alt to release the cursor (while captured) and can use Ctrl+Alt+F to toggle fullscreen. There are further parameters available for use:

{| {{wikitable}}
! heading !! heading
|-
| HDD Image/Device ''path'' || <code>-hda ''path''</code>
|-
| CD Image/Device ''path'' || <code>-cdrom ''path''</code>
|-
| Boot Device x(if not default) || <code>-boot x</code>
|-
| (x)MB RAM || <code>-m x</code>
|-
| Localized Time || <code>-localtime</code>
|-
| USB Support || <code>-usb</code>
|-
| Start in Full-Screen || <code>-full-screen</code>
|}

=== Bochs ===
[[Bochs]] supports a variety of features, all of which can be carefully customized with command line switches or a settings file. It can emulate a PC with VBE, NE2000, and Sound Blaster 16. It also has a built-in debugger which can be useful for testing and debugging your kernel. Bochs can emulate CPUs with support for SMP and x86-64.  However, Bochs itself does not use threading and the emulation is not ''virtualized'', so you can expect slow and CPU-intensive emulation. Bochs is available for Linux and Windows and is free and open source.

=== Oracle VirtualBox ===
[[VirtualBox]] is a desktop VM package.

=== Microsoft Virtual PC ===
[[Microsoft Virtual PC]] is a desktop VM package.

=== CPCE ===
CPCE is an Amstrad emulator, see the [http://cpce.emuunlim.com/ official CPCE website] for installation and usage instructions.

=== PearPC ===
PearPC is a [[PowerPC Overview|PowerPC]] emulator, an [http://pearpcguide.tripod.com in-depth tutorial] on how to setup the PearPC emulator with a PowerPC Mac OS install is available. (Pop-up blocker installed recommended!)

=== YAPE ===
[http://yape.homeserver.hu YAPE] is a Commodore emulator.

==See Also==
===External Links===
* [http://www.emulators.com/ A website filled with emulators from many platforms]
* [http://gablog.eu/online/node/39 Quick Qemu Tutorial]
* [http://qemu-forum.ipi.fi Qemu Forum]
* [http://www.thefreecountry.com/emulators/arm.shtml A good list of ARM emulators]

[[Category:Emulators]]
````

<a id="w002596"></a>

## W002596 — User:Combuster/Trio64 Hardware

Bron-ID: W002596
Namespace: User (2)
Revisie: 18268 | Datum: 2015-08-04T20:53:44Z
Bron: https://wiki.osdev.org/User:Combuster/Trio64_Hardware
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACombuster%2FTrio64_Hardware&oldid=18268

````text
Dumpbin for S3 stuff.

S3 has a series of SVGA graphics cards, of which the Trio64 is the most common, having had a very significant market share in the pre-3D era. I don't have 86c8xx or 86c9xx, nor ViRGE chipsets to check for compatibility, but they should work with similar code. Also, Microsoft's [[Microsoft Virtual PC]] emulates a Trio64V to a good extent, allowing you to test parts of your code without touching real hardware.

== Support ==

The Trio64 and compatible chipsets:

{| {{Wikitable}} 
! Chip name
! PCI id
! chipset signature
! tested
|-
| Trio32/64/64V+
| 0x5333 : 0x8811
| {{No}}
| {{Yes}}
|-
| VirtualPC Trio64
| 0x5333 : 0x8811
| E1 - 10 - 00
| {{Yes}}
|- 
| Trio64V2/DX
| 0x5333 : 0x8900  
| {{No}}
| {{No}}
|- 
| Trio64V2 DX/GX
| 0x5333 : 0x8901  
| E1 - 01 - 04
| Fail
|}
 
== VGA mode, VESA mode ==
The Trio64 is a typical '''S'''VGA card, adding more registers to the [[VGA Hardware|VGA standard]]. Most registers are available via port 3D4/3D5. It also supports a linear framebuffer. The location of which you can determine by looking at the first BAR in the [[PCI]] configuration space.
In addition, the hardware acceleration part uses a sparse register set, starting at port 42E8, and taking steps of 400 hex. This is a remnant of the ISA time. You can't change the location of these ports, yet they may interfere with other port placements if you (or the bios) are unaware of it.

Since there were many SVGA cards, S3 decided to protect several registers from accidental use. Hence, we first have to unlock them.

<source lang="C">
Write3C4(0x08, 0x06);                 // Unlock the PLL (clock) controls
Write3D4(0x38, 0x48);                 // Unlock the control registers #1
Write3D4(0x39, 0xA5);                 // Unlock the control registers #2
Write3D4(0x33, Read3D4(0x33) & 0xAD); // Unlock compatibility mode registers
</source>


Once that is done, we have free access over the video card's operation. Like several video cards, there is a VGA mode and there is an "extended" mode. Usually with some form of intermixing. In our case, the display signal will still be formed using the VGA registers. We will still need to access the accellerator and enable "extended" mode

<source lang="C">
outportw(&H4AE8, 1);                  // Enable the accellerator and extended mode
</source>


This puts us in limbo. There are a few things left to do to get access to the framebuffer. 

<source lang="C">
Write3D4(0x31, Read3D4(0x31) | 0x09); // Unfold VGA planes, enable memory > 265k
Write3D4(0x58, Read3D4(0x58) | 0x13); // Switch to LFB mode, disable VGA memory accesses
Write3D4(0x13, 80);                   // Set virtual width to something sensible
</source>

Now we're getting somewhere. Since we are still using the VGA registers, we're still using a valid set of timing registers. The net result is that we have a 640x400 timing left over from text mode, however the accellerator put us in 8-bit graphics mode. By loading the VGA portion with a 16-color mode, you get the corresponding 8-bit mode due to the accellerator being enabled. Note that due to changing the bitdepth, the amount of bytes on a scanline changed as well. Hence we need to update the virtual width register. For a single mode:


<source lang="C">
// For 640x???: Write3D4(0x13, 80); 
int pitch = horizontal_resolution * bytes_per_pixel;
Write3D4(0x13, pitch / 8);            // VGA thinks in characters, divide by 8
</source>


... to be continued ...


==Memory Mapped I/O==
Setting bit 4 in CR53 (I/O port 0x03D4, index 0x53) enables memory mapped I/O mode, where the extended registers become memory mapped like this:
:I/O port 0x8?E8 accessable via. memory address 0x000A8?E8
:I/O port 0x9?E8 accessable via. memory address 0x000A9?E8
:I/O port 0xA?E8 accessable via. memory address 0x000AA?E8
:I/O port 0xB?E8 accessable via. memory address 0x000AB?E8
These addresses are write-only

In addition, pixel data can be written to the memory area from 0x0000A0000 to 0x0000A7FFF (instead of using the "Pixel Data Transfer Registers" - I/O ports 0xE2E8 and 0xE2EA)


<!--
I've got a book here ("Programmer's Guide to the EGA, VGA, and Super VGA Cards (3rd Edition)") which has an (unfortunately partial) description of S3 

It also says that "I/O port 0xBEE8 index 0x0F, Read Register Select Register" is the only extended register that can't be memory mapped, and that memory mapped registers are "write-only". This shouldn't be a problem though (keep a copy of video card's registers in RAM so that you can read from RAM instead of reading from the device's registers directly).

!-->

== Debugging ==
There have been reports that some Trio64s don't actually reconfigure themselves properly upon reboot.
````

<a id="w002597"></a>

## W002597 — Unreal mode

Bron-ID: W002597
Namespace: Main (0)
Revisie: 8008 | Datum: 2009-05-28T23:43:42Z
Bron: https://wiki.osdev.org/Unreal_mode
Revisie-URL: https://wiki.osdev.org/index.php?title=Unreal_mode&oldid=8008
Doorverwijzing: Unreal Mode

````text
#REDIRECT [[Unreal Mode]]
````

<a id="w002598"></a>

## W002598 — VMX

Bron-ID: W002598
Namespace: Main (0)
Revisie: 30309 | Datum: 2026-03-17T18:52:08Z
Bron: https://wiki.osdev.org/VMX
Revisie-URL: https://wiki.osdev.org/index.php?title=VMX&oldid=30309

````text
{{Stub}}

'''Intel's Virtual-Machine Extensions''' 

== Programming == 

=== Discovering support ===
After enumerating the features from [[CPUID]], bit 5 in ECX will tell you if VMX is supported or not.  This is the first thing that should be checked!

=== Basic environment ===
Software should enable bit 13 of CR4 (CR4.VMXE) first.  Here are a few definitions of some used MSR's that we'll be dealing with: 
<syntaxhighlight lang="c">
 #define IA32_VMX_BASIC          0x480
 #define IA32_VMX_CR0_FIXED0     0x486
 #define IA32_VMX_CR0_FIXED1     0x487
 #define IA32_VMX_CR4_FIXED0     0x488
 #define IA32_VMX_CR4_FIXED1     0x489
</syntaxhighlight>

In order to see what your OS needs to support, you need to read the lower 32-bits of IA32_VMX_CR0_FIXED0 and IA32_VMX_CR4_FIXED0.  The bits in those MSR's are what need to be set (and supported) in their corresponding control registers (CR0/CR4).  If any of those bits are not enabled, a GPF will occur.  Also, to see what extra bits can be set, check the IA32_VMX_CR0_FIXED1 and IA32_VMX_CR4_FIXED1.  It's basically a mask of bits, if a bit is set to 1, then it "can" be enabled, but if it's a 0, you may generate an exception if you enable the corresponding bit in the control registers. 

From my own hardware and emulator (bochs), I've found that you will most likely need to enable these bits: CR0.NE, CR4.VMXE, CR0.PG and CR0.PE. For 64-bit kernels, you need to have IA32_EFER.LMA set.

=== Executing VMXON ===
The main entry point for using VMX is through the VMXON instruction.  The instruction requires a single operand of a m64 region called the VMXON region.  The memory region needs to be 4096-byte aligned (bits 0-11 must be 0) and the only VMCS field that should be modified is the VMCS revision identification field.  This ID field should contain the value in bits 0-31 of MSR IA32_VMX_BASIC.  In order to prepare a memory address in 32-bit PMode for use as an m64, some modifications need to be made.  The upper 32-bits of the m64 on non long mode capable processors have to be 0 or an "invalid memory address" error will occur and a VMEXIT will be called.  
<syntaxhighlight lang="c">
 uint32_t * region = (uint32_t *)allocate_4k_aligned(4096);
 uint64_t region64 = (uint64_t)((size_t)(region) & 0xFFFFFFFF);
 asm volatile("  vmxon %0; "::"m" (region64));
</syntaxhighlight>

This general process of taking a 32-bit memory address and turning it into a psuedo-64bit int (unsigned long long) will be used for all m64 operands later.  VMCLEAR is another example instruction that requires the upper 32-bits of a memory address to be 0.  

Long mode capable processors simply requires a 64-bit pointer to the region.  

Note: '''The VMXON, VMCLEAR and VMPTRLD instruction must point to the physical address of their respective regions.'''

== VMX instruction error checking ==

VMX instructions have their own error reporting mechanism to indicate the success or failure of a given operation.

There are two flags used to signify the success or failure of a VM instruction.  The carry flag(CF) and the zero flag(ZF).

If both of these flags are clear after a VM instruction was executed then it succeeded.

If the carry flag is set then current VMCS pointer is invalid.

If the zero flag is set, it indicates that the VMCS pointer is valid but there is some other error specified in the VM-instruction error field (encoding 4400h).  Error numbers are listed in section 5.4 of the Intel SDM Volume 2B.

The following table represents the error numbers in the VM-instruction error field:

{| {{wikitable}}
|-
! Error Number !! Description
|-
| 0x01 || VMCALL executed in VMX root operation
|-
| 0x02 || VMCLEAR with invalid physical address.
|-
| 0x03 || VMCLEAR with VMXON pointer.
|-
| 0x04 || VMLAUNCH with non-clear VMCS.
|-
| 0x05 || VMRESUME with non-launched VMCS.
|-
| 0x06 || VMRESUME with a corrupted VMCS. Indicates corruption of the current VMCS
|-
| 0x07 || VM entry with invalid VMX-control field(s).
|-
| 0x08 || VM entry with invalid host-state field(s).
|-
| 0x09 || VMPTRLD with invalid physical address.
|-
| 0x0A || VMPTRLD with VMXON pointer.
|-
| 0x0B || VMPTRLD with incorrect VMCS revision identifier.
|-
| 0x0C || VMREAD/VMWRITE from/to unsupported VMCS component.
|-
| 0x0D || VMWRITE to read-only VMCS component.
|-
| 0x0F || VMXON executed in VMX root operation.
|-
| 0x1A || VM entry with events blocked by MOV SS.
|}

== VMCS ==
The VMCS only has two relevant fields that can be accessed at this time, a 4-byte VMCS revision ID located at byte offset 0 and a 4-byte abort indicator field located at byte offset 4.  The rest of the VMCS is reserved for field data.

The revision field must be filled with the 32-bit revision Id stored in bits 0-31 of the IA32_VMX_BASIC MSR similar to the VMX region.

The abort field will simply contain a non-zero value if the VM abort occurs during a VMX exit, see section 23.7 in Intel's SDM 3B for error values.

A VMCS is loaded with the VMPTRLD instruction, which loads and activates a VMCS, and requires a 64-bit memory address as it's operand in the same format as VMXON/VMCLEAR.

<syntaxhighlight lang="c">
 asm volatile ("vmptrld %0; ":: "m" (vmcsRegion64));
</syntaxhighlight>

The structure of the VMCS is covered in detail in Chapter 20 of the Intel SDM volume 3B (see link below).  Field encodings for VMWRITE and VMREAD are covered in Appendix H of the same manual.
== Peripheral Emulation ==
=== IO framework emulation ===
In x86, there are two kinds of IO channels: Port-Based IO(aka '''PIO''' ) and Memory-Mapped IO(aka '''MMIO'''). PIO has separate address space and special instructions to do IO jobs. while with MMIO, the device IO space is backed with the memory address space, you can use memory data move instructions to do IO jobs.
==== PIO emulation ====
with Intel VT-x, the hypervisor is able to determine whether the guest's IO instructions trap into vmx root mode by setting the primary processor based control bit 24. if this bit is set, all the guest's IO instructions will causes vm exits. otherwise, you have to setup the two IO bitmap regions to capture the vm exits you are interested in.

the IO causes vm exit with basic reason number as 30, you can retrieve the IO operation size, direction, port id and etc. for more please refer to the vmx_pio.c in reference pages.
==== MMIO emulation ====
MMIO emulation in x86 is a bit different: we are going to exploit EPT in order to capture MMIO events.
VMX provides two kinds of EPT involved vm exits: EPT violation and EPT misconfiguration.  In general, when guest is accessing memory which is not backed correctly(e.g. the memory is '''writable''' but '''not readable'''!), VMX results in vm exits with EPT misconfiguration. 
the hypervisor must do the following steps to do MMIO operation:
<pre>
 1). decode the memory move instruction to determine the memory involved instruction length, access size, direction, operations, registers index/immediates  and memory address,
 2). search the MMIO devices regions to see whether the address is backed with a DEVICE.
 3). store the result in destination  register if necessary.
 4). advance to next instruction by adding guest RIP with instruction length resolved in step 1.
</pre>

=== Devices emulation examples===
{| {{wikitable}}
|-
! Device !! IO type !! Refference
|-
| Intel 8259 PIC || PIO ||https://github.com/chillancezen/ZeldaOS.x86_64/blob/master/vm_monitor/device_8259pic.c
|-
| Intel 8253 PIT || PIO ||https://github.com/chillancezen/ZeldaOS.x86_64/blob/master/vm_monitor/device_8253pit.c
|-
| Intel 8042 keyboard || PIO ||https://github.com/chillancezen/ZeldaOS.x86_64/blob/master/vm_monitor/device_keyboard.c
|-
| serial port controller || PIO ||https://github.com/chillancezen/ZeldaOS.x86_64/blob/master/vm_monitor/device_serial.c
|-
| 16-colors video controller || MMIO ||https://github.com/chillancezen/ZeldaOS.x86_64/blob/master/vm_monitor/device_video.c
|-
|}

== References ==
Intel's SDM 3B: http://www.intel.com/Assets/PDF/manual/253669.pdf 

Intel's SDM 2B: http://www.intel.com/Assets/PDF/manual/253667.pdf 

KVM's VMX.c (GPLv2): http://lxr.free-electrons.com/source/arch/x86/kvm/vmx.c 

BOCHS's VMX.c (LGPLv2): http://bochs.cvs.sourceforge.net/viewvc/bochs/bochs/cpu/vmx.cc 

PIO sub handler: https://github.com/chillancezen/ZeldaOS.x86_64/blob/master/vm_monitor/vmx_pio.c

Memory move instruction decode: https://github.com/chillancezen/ZeldaOS.x86_64/blob/master/vm_monitor/vmx_instruction_decoding.c



== Other examples ==
Vmx implementation in home made OS:
http://www.dumais.io/index.php?article=ac3267239dd3e34c061de6413203fb98

[[Category:X86 Virtualization]]
````

<a id="w002602"></a>

## W002602 — User:Computafreak

Bron-ID: W002602
Namespace: User (2)
Revisie: 23991 | Datum: 2019-09-25T12:30:31Z
Bron: https://wiki.osdev.org/User:Computafreak
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AComputafreak&oldid=23991

````text
==Sub-pages==
[[User:Computafreak/DynamicLinking|Dynamic linking]]
==ACPI tables==
{| class="wikitable"
|-
! Emulator
! Tables detected
|-
| [[Microsoft Virtual PC]]
| [[RSDP]], [[RSDT]], [[FADT|FACP]]
|- 
| [[Bochs]]
| [[RSDP]], [[RSDT]], [[FADT|FACP]], [[MADT]], [[SSDT]]
|}

==PCI IRQ tables==
[http://www.microsoft.com/whdc/archive/pciirq.mspx PCI IRQ Routing Table Specification]

==VPC hardware comparison==
{| {{wikitable}}
|-
! Category
! Virtual PC (2004, 2007)
! Notes
|-
| Floppy disk drive
| Supported
| Both versions support 720 KB and 1.44 MB disk images.
|-
| Power management
| APM 1.2, ACPI
| Only the basic ACPI tables are supported, with no MADT or derived tables. I'm trying APM in a few weeks.
|-
| BIOS
| American Megatrends
| 
|-
| CD/DVD
| CD supported
| Can use either an ISO image, or the host computer's physical CD or DVD drive.
|-
| Sound
| Supported
| The sound card emulated is the Creative Labs Sound Blaster 16 (ISA), which supports digital signal processing and FM synthesis.
|-
| Ports
| LPT1, COM1 and COM2
| LPT1 is redirected to a physical parallel port. The serial ports are redirected to either a file, named pipe or physical port.
|-
| Video
| S3 Trio PCI
| VESA 2.0 is supported.
The maximum supported resolution is 1600x1200.
8 MB of video RAM is available, but to access it, VPC additions are needed.
S3 Trio graphics acceleration is fully supported.
|-
| Keyboard
| Supported
| Controller chip is Intel 8255.
|-
| Motherboard
| Emulates Intel 440BX
| Contains the PIC (8259), PIT (8254), DMA controller (8237), CMOS and RTC.
|-
| Mouse
| Supported
| PS2 mouse supported on IRQ 12.
|-
| Network
| Intel/DEC 21140
| This is an Ethernet card. It supports the five most common packet types:
*IEEE 802.2
*IP
*ARP
*Novell IPX
*Novell IPX 802.3
Both unicast and multicast packets are supported.
|-
| Processor
| Intel Pentium 3
| The host computer's processor is virtualised.
|-
| APICs
| Not supported
| Virtual PC doesn't support any APICs of any type. You're stuck with the PIT
|}
Source: [http://support.microsoft.com/kb/833144 Overview of the technical specifications in Virtual PC 2004]
````

<a id="w002603"></a>

## W002603 — Getting to Ring 3

Bron-ID: W002603
Namespace: Main (0)
Revisie: 27736 | Datum: 2023-04-07T18:45:20Z
Bron: https://wiki.osdev.org/Getting_to_Ring_3
Revisie-URL: https://wiki.osdev.org/index.php?title=Getting_to_Ring_3&oldid=27736

````text
{{In_Progress}}
The end goal of writing a kernel is to get to '''userspace''', or, in other words, going from ring 0 to ring 3. While one might expect that a ring 3 [[GDT]] entries would be sufficient, it is more complicated. All of the following tasks must be completed:

* Add two new GDT entries (at least) configured for ring 3.
** These entries are needed for the user's code and data segments (one each)
* Set up a barebones [[TSS]] with an ESP0 stack.
** When an interrupt (be it fault, IRQ, or software interrupt) happens while the CPU is in user mode, the CPU needs to know where the kernel stack is located. This location is stored in the ESP0 (0 for ring 0) entry of the TSS.
* Set up an [[IDT]] entry for ring 3 system call interrupts (optional).
** System calls are the way user code requests the kernel to do IO and process management. For more information see [[System Calls]]

== Requirements ==
* Ring 0 GDT and IDT
* IRQ handling
* Plans for multitasking with task switching

== GDT ==
Following is an example of a GDT entry structure in C, utilizing bit-fields:
<source lang="c">
struct gdt_entry_bits {
	unsigned int limit_low              : 16;
	unsigned int base_low               : 24;
	unsigned int accessed               :  1;
	unsigned int read_write             :  1; // readable for code, writable for data
	unsigned int conforming_expand_down :  1; // conforming for code, expand down for data
	unsigned int code                   :  1; // 1 for code, 0 for data
	unsigned int code_data_segment      :  1; // should be 1 for everything but TSS and LDT
	unsigned int DPL                    :  2; // privilege level
	unsigned int present                :  1;
	unsigned int limit_high             :  4;
	unsigned int available              :  1; // only used in software; has no effect on hardware
	unsigned int long_mode              :  1;
	unsigned int big                    :  1; // 32-bit opcodes for code, uint32_t stack for data
	unsigned int gran                   :  1; // 1 to use 4k page addressing, 0 for byte addressing
	unsigned int base_high              :  8;
} __packed; // or `__attribute__((packed))` depending on compiler
</source>
Using this structure, two ring 3 segments, both with base of 0 and limit of 0xFFFFFFFF, can be added, as follows:
<source lang="c">
static gdt_entry_bits gdt[6]; // one null segment, two ring 0 segments, two ring 3 segments, TSS segment
// (ring 0 segments)

gdt_entry_bits *ring3_code = &gdt[3];
gdt_entry_bits *ring3_data = &gdt[4];

ring3_code->limit_low = 0xFFFF;
ring3_code->base_low = 0;
ring3_code->accessed = 0;
ring3_code->read_write = 1; // since this is a code segment, specifies that the segment is readable
ring3_code->conforming = 0; // does not matter for ring 3 as no lower privilege level exists
ring3_code->code = 1;
ring3_code->code_data_segment = 1;
ring3_code->DPL = 3; // ring 3
ring3_code->present = 1;
ring3_code->limit_high = 0xF;
ring3_code->available = 1;
ring3_code->long_mode = 0;
ring3_code->big = 1; // it's 32 bits
ring3_code->gran = 1; // 4KB page addressing
ring3_code->base_high = 0;

*ring3_data = *ring3_code; // contents are similar so save time by copying
ring3_data->code = 0; // not code but data

install_tss(&gdt[5]); // TSS segment will be the fifth 

flush_tss();
</source>
In actuality, the CPU can be put into user mode with just these two segments. However it is impossible to return to ring 0 for system calls, faults, or even IRQs. That is where the TSS comes in.

== The TSS ==
The TSS can be used for multitasking, though it is recommended to use software multitasking for these reasons:
* Software task switching is faster (usually)
* When you port your OS to a different CPU, it probably won't have the TSS, so you'll have to implement software task switching anyway
* x86 64-bit mode does not allow you to use the TSS for task switching (the main reason, especially if your goal is to read 64-bit mode)
This guide will use software multitasking. Because of this the 32-bit TSS will contain a lot of junk we don't need. Here is the structure of the TSS:
<source lang="c">
struct tss_entry_struct {
	uint32_t prev_tss; // The previous TSS - with hardware task switching these form a kind of backward linked list.
	uint32_t esp0;     // The stack pointer to load when changing to kernel mode.
	uint32_t ss0;      // The stack segment to load when changing to kernel mode.
	// Everything below here is unused.
	uint32_t esp1; // esp and ss 1 and 2 would be used when switching to rings 1 or 2.
	uint32_t ss1;
	uint32_t esp2;
	uint32_t ss2;
	uint32_t cr3;
	uint32_t eip;
	uint32_t eflags;
	uint32_t eax;
	uint32_t ecx;
	uint32_t edx;
	uint32_t ebx;
	uint32_t esp;
	uint32_t ebp;
	uint32_t esi;
	uint32_t edi;
	uint32_t es;
	uint32_t cs;
	uint32_t ss;
	uint32_t ds;
	uint32_t fs;
	uint32_t gs;
	uint32_t ldt;
	uint16_t trap;
	uint16_t iomap_base;
} __packed;

typedef struct tss_entry_struct tss_entry_t;
</source>
To setup this TSS structure, give it an initial esp0 stack with the correct ss0 segment.
<source lang="c">
// Note: some of the GDT entry struct field names may not match perfectly to the TSS entries.
tss_entry_t tss_entry;

void write_tss(gdt_entry_bits *g) {
	// Compute the base and limit of the TSS for use in the GDT entry.
	uint32_t base = (uint32_t) &tss_entry;
	uint32_t limit = sizeof tss_entry;

	// Add a TSS descriptor to the GDT.
	g->limit_low = limit;
	g->base_low = base;
	g->accessed = 1; // With a system entry (`code_data_segment` = 0), 1 indicates TSS and 0 indicates LDT
	g->read_write = 0; // For a TSS, indicates busy (1) or not busy (0).
	g->conforming_expand_down = 0; // always 0 for TSS
	g->code = 1; // For a TSS, 1 indicates 32-bit (1) or 16-bit (0).
	g->code_data_segment=0; // indicates TSS/LDT (see also `accessed`)
	g->DPL = 0; // ring 0, see the comments below
	g->present = 1;
	g->limit_high = (limit & (0xf << 16)) >> 16; // isolate top nibble
	g->available = 0; // 0 for a TSS
	g->long_mode = 0;
	g->big = 0; // should leave zero according to manuals.
	g->gran = 0; // limit is in bytes, not pages
	g->base_high = (base & (0xff << 24)) >> 24; //isolate top byte

	// Ensure the TSS is initially zero'd.
	memset(&tss_entry, 0, sizeof tss_entry);

	tss_entry.ss0  = REPLACE_KERNEL_DATA_SEGMENT;  // Set the kernel stack segment.
	tss_entry.esp0 = REPLACE_KERNEL_STACK_ADDRESS; // Set the kernel stack pointer.
	//note that CS is loaded from the IDT entry and should be the regular kernel code segment
}

void set_kernel_stack(uint32_t stack) { // Used when an interrupt occurs
	tss_entry.esp0 = stack;
}
</source>
Finally, the implementation of the flush_tss function (Intel syntax):
<source lang="asm">
; C declaration: void flush_tss(void);
global flush_tss
flush_tss:
	mov ax, (5 * 8) | 0 ; fifth 8-byte selector, symbolically OR-ed with 0 to set the RPL (requested privilege level).
	ltr ax
	ret
</source>

At this point the kernel is ready to enter ring 3. It's worth noticing that the DPL of the TSS descriptor in GDT has ''nothing'' to do with the privilege level the task will run on: that depends on the DPL of the code segment used to set CS. The DPL of the TSS descriptor determines at which privilege level is it possible to CALL it, triggering a hardware context switch (32-bit only).

From the Intel® 64 and IA-32 Architectures Software Developer’s Manual Volume 3, Section 7.2.2 (TSS Descriptor):

<blockquote>In most systems, the DPLs of TSS descriptors are set to values less than 3, so that only privileged software can perform task switching. However, in multitasking applications, DPLs for some TSS descriptors may be set to 3 to allow task switching at the application (or user) privilege level.</blockquote>

== Entering Ring 3 ==
The x86 is a tricky CPU. No matter how you approach it, there is no easy way to enter user mode. Nonetheless, below are three ways to enter user mode.

=== iret method ===
One of the ways to get to ring 3 is to make the processor think it was already in ring 3 to start with. This can be accomplished with an iret. Following is a simple example of this trick:
<source lang="asm">
global jump_usermode
extern test_user_function
jump_usermode:
	mov ax, (4 * 8) | 3 ; ring 3 data with bottom 2 bits set for ring 3
	mov ds, ax
	mov es, ax 
	mov fs, ax 
	mov gs, ax ; SS is handled by iret

	; set up the stack frame iret expects
	mov eax, esp
	push (4 * 8) | 3 ; data selector
	push eax ; current esp
	pushf ; eflags
	push (3 * 8) | 3 ; code selector (ring 3 code with bottom 2 bits set for ring 3)
	push test_user_function ; instruction address to return to
	iret
</source>
This will call test_user_function and it will be operating in user mode! Have the test_user_function execute a cli or other privileged instruction and you'll be pleased by a [[General Protection Fault]].

=== sysexit method ===
The second way is to use the sysexit instruction as follows:
<source lang="asm">
global jump_usermode
extern test_user_function
jump_usermode:
	mov ax, (4 * 8) | 3 ; user data segment with RPL 3
	mov ds, ax
	mov es, ax
	mov fs, ax
	mov gs, ax ; sysexit sets SS

	; setup wrmsr inputs
	xor edx, edx ; not necessary; set to 0
	mov eax, 0x8 ; the segments are computed as follows: CS=MSR+0x10 (0x8+0x10=0x18), SS=MSR+0x18 (0x8+0x18=0x20).
	mov ecx, 0x174 ; MSR specifier: IA32_SYSENTER_CS
	wrmsr ; set sysexit segments

	; setup sysexit inputs
	mov edx, test_user_function ; to be loaded into EIP
	mov ecx, esp ; to be loaded into ESP
	sysexit
</source>

=== sysret method ===
The other way is to use the sysret instruction as follows:
<source lang="asm">
; note: this code is for 64-bit long mode only.
;       it is unknown if it works in protected mode.
;       using intel assembly style
global jump_usermode
extern test_user_function
jump_usermode:
;enable system call extensions that enables sysret and syscall
	mov rcx, 0xc0000082
	wrmsr
	mov rcx, 0xc0000080
	rdmsr
	or eax, 1
	wrmsr
	mov rcx, 0xc0000081
	rdmsr
	mov edx, 0x00180008
	wrmsr

	mov ecx, test_user_function ; to be loaded into RIP
	mov r11, 0x202 ; to be loaded into EFLAGS
	sysretq ;use "o64 sysret" if you assemble with NASM
</source>

== Multitasking considerations ==
There are a lot of subtle aspects of user mode and task switching. Whenever a system call interrupt happens, ESP0 is loaded into the stack pointer and all system information is pushed before the interrupt handler is entered. This could become a problem with two ring 3 tasks. Imagine: esp is currently set to the ESP0 stack. When the interrupt is receive, esp is set to the other task's esp which is the same ESP0 stack. To avoid overwriting data, the handler must change the ESP0 stack (along with the interrupt-pushed ESP stack) on each task switch.

[[Category:Tutorials]]
[[Category:X86 CPU]]
````

<a id="w002604"></a>

## W002604 — Linker Scripts

Bron-ID: W002604
Namespace: Main (0)
Revisie: 30365 | Datum: 2026-03-22T17:18:01Z
Bron: https://wiki.osdev.org/Linker_Scripts
Revisie-URL: https://wiki.osdev.org/index.php?title=Linker_Scripts&oldid=30365

````text
[[LD|GNU ld]] supports configuration of the linking process via the use of linker scripts. Linker scripts are written in a specialized scripting language specific to the GNU ld application. While the use of a linker script file is not strictly required by <tt>ld</tt>, it provides a more maintainable alternative to specifying complex configuration options as arguments on the command line. A Linker script's principal use is specifying the format and layout of the final executable binary. This is of particular relevance to operating-system development, where executable binaries will often require specific file layouts in order to be recognized by certain [[Bootloader|bootloaders]]. [[GRUB|GNU GRUB]] is one such bootloader.<br/>
Linker scripts are typically invoked via the use of the <tt>-T script.ld</tt> command line argument when calling the <tt>ld</tt> application.

== Keywords ==
Listed below are a selection of significant keywords used within linker scripts.

==== ENTRY ====
<pre>
ENTRY(main)
ENTRY(MultibootEntry)
</pre>

The <tt>ENTRY</tt> keyword is used for defining the entry point of an application, that being specifically the first executable instruction in the output file. This keyword accepts the symbol name for the entry point of the linked program/kernel as a single argument. The code pointed to by the symbol name provided will be the first byte of the <tt>.text</tt> section in [[ELF]] and [[PE]] binaries.

==== OUTPUT_FORMAT ====
<pre>
OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_FORMAT("pe-i386")
</pre>

The <tt>OUTPUT_FORMAT</tt> directive takes a single argument. It specifies the output format of the executable. To find out what output formats are supported by your system binutils and GCC, the <code lang="bash">objdump -i</code> command may be used.

Some commonly used formats are detailed below:

{| class="wikitable"
|-
! Format
! Description
|-
| <tt>binary</tt>
| A flat binary with no formatting at all.
|-
| <tt>elf32-i386</tt>
| 32-bit ELF format for the i386 architecture.
|-
| <tt>elf64-x86-64</tt>
| 64-bit ELF format for the x86-64 architecture.
|-
| <tt>pe-i386</tt>
| 32-bit PE format for the i386 architecture.
|}

==== STARTUP ====
<pre>
STARTUP(Boot.o)
STARTUP(crt0.o)
</pre>

STARTUP takes one argument. It is which file you want to be linked to the very beginning of the executable. For userland programs, this is usually crt0.o or crtbegin.o. For kernels, it is usually the file that contains your assembly boilerplate that initiates the stack and in some cases GDT and such and then calls your kmain().

==== SEARCH_DIR ====
<pre>
SEARCH_DIR(Directory)
</pre>

This will add a path to your library search directory. The -nostdlib flag will cause any library found in this path to be effectively ignored. I'm not sure why, it just seems to be how ld works. It treats linker script specified search directories as standard directories, and therefore ignores them with -no-default-libs and such flags

==== INPUT ====
<pre>
INPUT(File1.o File2.o File3.o ...)
INPUT
(
	File1.o
	File2.o
	File3.o
	...
)
</pre>

INPUT is a 'in-linker script' replacement for adding object files to the command line. Where you would usually specify something like ''ld File1.o File2.o'', the INPUT section can be used to do this inside the linker script instead.

==== OUTPUT ====
<pre>
OUTPUT(Kernel.bin)
</pre>

The <tt>OUTPUT</tt> command specifies the file to be generated as the output of the linking process. This is the name of the final binary created. The effect of this command is identical to the effect of the <code lang="bash">-o filename</code> command line flag, which overrides it.

==== MEMORY ====
<pre>
MEMORY
{
    ROM (rx) : ORIGIN = 0, LENGTH = 256k
    RAM (wx) : org = 0x00100000, len = 1M
}
</pre>

MEMORY declares one or more memory regions with attributes specifying whether the region can be written to, read from or executed. This is mostly used in embedded systems where different regions of address space may contain different access permissions.

The example script above tells the linker that there are two memory regions:<br/>
a) "ROM" starts at address 0x00000000, is 256kB in length, can be read and executed.<br/>
b) "RAM" starts at address 0x00100000, is 1MB in length, can be written, read and executed.

==== SECTIONS ====
<pre>
SECTIONS
{
  .text.start (_KERNEL_BASE_) : {
    startup.o( .text )
  }

  .text : ALIGN(CONSTANT(MAXPAGESIZE)) {
_TEXT_START_ = .;
    *(.text)
_TEXT_END_ = .;
  }

  .data : ALIGN(CONSTANT(MAXPAGESIZE)) {
_DATA_START_ = .;
    *(.data)
_DATA_END_ = .;
  }

  .bss : ALIGN(CONSTANT(MAXPAGESIZE)) {
_BSS_START_ = .;
    *(.bss)
_BSS_END_ = .;
  }
}
</pre>

This script tells the linker to place code from the ".text" section in startup.o at the beginning, starting at logical address <tt>_KERNEL_BASE_</tt>.
This is then followed by page-aligned sections for all ".text", ".data" and ".bss" sections of all the other input files. If your sections are not properly page aligned and separated, GNU ld will generate a "LOAD segment with RWX permissions" warning. 

Note that different architectures have different page sizes. You can modify the size of "<tt>MAXPAGESIZE</tt>" by adding "<tt>-z max-page-size=0x1000</tt>" switch to the linker. This behavior only exists on GNU ld.

Linker symbols are defined holding the start and end address of each section. These symbols have external linkage within the application and are accessible as pointers within the code. Additional information on linker file symbols can be found below.

==== KEEP ====

The <tt>KEEP</tt> statement within a linker script will instruct the linker to keep the specified section, even if no symbols inside it are referenced. This statement is used within the <tt>SECTIONS</tt> section of the linker script. This becomes relevant when garbage collection is performed at link time, enabled by passing the <tt>--gc-sections</tt> switch to the linker. The <tt>KEEP</tt> statement instructs the linker to use the specified section as a root node when creating a dependency graph, looking for unused sections. Essentially forcing the section to be marked as used.

This statement is commonly seen in linker scripts targeting the [[:Category:ARM|ARM]] architecture for placing the interrupt vector table at offset <tt>0x00000000</tt>. Without this directive the table, which might not be referenced explicitly in code, would be pruned out.

<pre>
SECTIONS
{
	.text :
	{
		KEEP(*(.text.ivt))
		*(.text.boot)
		*(.text*)
	} > ROM

	/** ... **/
}
</pre>

== Symbols ==
It is possible to define arbitrary symbols within a linker script. These symbols are added into the program's symbol table. Each symbol in the table has a name and an associated address. Symbols within a linker script that have been assigned a value will be given external linkage, and are accessible within the program's code as pointers.

An example script showing three different places that symbol assignment may be used can be seen below:

<pre>
floating_point = 0;
SECTIONS
{
  .text :
    {
      *(.text)
      _etext = .;
    }
  _bdata = (. + 3) & ~ 3;
  .data : { *(.data) }
}
</pre>

In the above example, the symbol <tt>floating_point</tt> has been defined as zero. The symbol <tt>_etext</tt> has been defined as the address following the last .text input section. The symbol <tt>_bdata</tt> has been defined as the address following the .text output section aligned upward to a 4 byte boundary.


An example usage of these symbols in C code can be seen below:

<syntaxhighlight lang="c">
/** Externally linked symbol */
extern uintptr_t _etext;
// ...
/** Pointer to the binary data at the address stored in the symbol expression. */
uint32_t* item = &_etext;
</syntaxhighlight>

== See Also ==
=== Articles ===
* [[Using Linker Script Values]]

=== External Links ===
* [https://sourceware.org/binutils/docs/ld/Scripts.html#Scripts GNU ld manual on linker scripts]
* [https://sourceware.org/binutils/docs/ld/ GNU ld manual (full)]
* [http://www.bravegnu.org/gnu-eprog/lds.html  GNU ld script example]

[[Category:Linkers]]
````

<a id="w002605"></a>

## W002605 — User:Oros/SFS

Bron-ID: W002605
Namespace: User (2)
Revisie: 17855 | Datum: 2015-04-03T09:51:29Z
Bron: https://wiki.osdev.org/User:Oros/SFS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AOros%2FSFS&oldid=17855

````text
{{Filesystems}}
'''''This page is a draft - you're free to edit it to your hearts content ^_^'''''
__NOTOC__ 
== SFS (Simple File System) Specification ==
'''Note:''' This page refers to version 1.0 of the SFS Specification.

''This specification has no copyright and there are no restrictions on the use of this specification. It is distributed to help other programmers understand how the Simple File System works and to implement Simple File System software without compatibility problems. This specification is distributed without any warranty. Use this specification at your own risk.''

== Introduction ==
Unlike networking protocols, there is currently no widely accepted format for freely exchanging data on physical media between computers and other devices. Instead people have been using a variety of file systems that are either not widely accepted or subject to technical, licensing, copyright or patent restrictions.

The primary purpose of the Simple File System is to facilitate the free exchange of data on physical media without unnecessary restrictions or complexity. For this reason the Simple File System has been designed in such a way that it is easy to understand and implement.

== File System Format Overview ==
The file system uses different areas on the physical media for different purposes. The five areas are:
* the super-block
* the reserved area
* the data area
* free area
* the index area

The purpose and usage of each of these areas is described in detail in the following sections. These areas are always in the following order:
{| align="center" border="0" width="50%"
|- style="background-color:#FFDDAA;"
| Super Block
|- style="background-color:#AADDFF;"
| Reserved Area

|- style="background-color:#FFDDAA;"
| Data Area



|- style="background-color:#AADDFF;"
| Free Area



|- style="background-color:#FFDDAA;"
| Index Area
|}
'''Table 1 - Layout of areas on the media'''

All values defined within this specification must be stored with the least significant byte at the lowest address (i.e. in "[[little-endian]]" format). All fields marked as "Unused/reserved" must be filled with zeros when written to the media and ignored when read from the media, except as otherwise noted (in the super-block and reserved area).

== Super-Block Format ==
The super-block contains values needed to determine the physical size and layout of the media itself, and values needed to determine the location of each area. The super-block also contains many values that are used for compatibility purposes.

{| align="center" border="1" width="50%"
|- style="font-weight: bold;"
| Offset
| Size in Bytes
| Descriptions
|-
| 0x0000
| 11
| Reserved for [[compatibility]] (boot code)
|-
| 0x000B
| 21
| Reserved for [[compatibility]] (legacy [[BIOS]] parameter block)
|-
| 0x0020
| 372
| Reserved for [[compatibility]] (boot code)
|-
| 0x0194
| 8
| [[Time Stamp]]
|-
| 0x019C
| 8
| Size of index area in bytes
|-
| 0x01AC
| 3
| Magic number (0x534653)
|-
| 0x01AF
| 1
| Simple File System version number (0x10 for Version 1.0)
|-
| 0x01B0
| 8
| Total number of blocks
|-
| 0x01B8
| 4
| Number of reserved blocks
|-
| 0x01BC
| 1
| Block size
|-
| 0x01BD
| 1
| [[Checksum]]
|-
| 0x01BE
| 64
| Reserved for [[compatibility]] ([[Partition_Table|partition table]])
|-
| 0x01FE
| 2
| Reserved for [[compatibility]] ([[Boot_Signature|boot signature]])
|}
'''Table 2 - Super-block format'''

The reserved areas at offsets 0x0000, 0x000B, 0x0020, 0x01BE and 0x1FE are used for compatibility purposes, and is not covered by this specification. Data in these areas must be ignored when the super-block is read, and never modified (unless the modifications are for purposes not covered by this specification).

The checksum field (at offset 0x01BD) is calculated such that the sum of all bytes stored between offset 0x01AC and 0x01BD inclusive will have the lowest 8 bits clear. This checksum is set when the media is formatted and does not normally need to be changed after the media is formatted (however, it may need to be recalculated if the version number at offset 0x01AF is changed).

The block size (at offset 0x01BC) is used to calculate the number of bytes in each block. The formula for this calculation is "bytes_per_block = 2 (block_size + 7)", so that a value of 2 represents 512 bytes per block, 3 represents 1024 bytes per block, etc. This value must be equal to or greater than 1 (128 bytes per block or less is not supported). For media that is organized into "sectors" ([[floppy disk]]s, [[hard disks]], etc) the block size must match the size of a sector. For example, for a common 1440 KB floppy disk this value would be 2 (512 bytes per sector).

The number of reserved blocks (at offset 0x01B8) specifies the number of blocks that are reserved for the reserved area and the super-block. This value is also used to determine where the data area begins. This value must be 1 or greater (as it includes the super-block). For example, if the first block is "block number 0" and the reserved area (including the super-block) is 20 blocks in size, then this value must contain 20 and the first block in the data area will be "block number 20".

The total number of blocks (at offset 0x01B0) includes all areas defined in this specification, including the super-block and reserved area. It is used to determine the physical size of the media and for finding the end of the index area. The formula for calculating the physical size of the media in bytes is "media_size = total_blocks * 2 (block_size + 7)". For example, for a common 1440 KB floppy disk this value would be 2880.

The magic number (at offset 0x01AC) is used to indicate compliance with this specification. The magic number  0x534653 represents the [[ASCII]] characters 'SFS'. The magic number is positioned next to the version number so that software can check both in a single operation (for example, software can check if the 32 bit value at offset 0x01AC equals 0x10534653).

The Simple File System version number (at offset 0x01AF) must contain the value 0x10 to indicate compliance with this specification. This value is in fixed point BCD, where the high 4 bits determine the major version number and the lower 4 bits determine the minor version number. Any future versions of this specification (which aren't expected but are possible) will use numerically higher values for the version number. For example, 0x23 would represent version 2.3 of the SFS specification.

All future versions of the Simple File System will remain compatible with "SFS Version 1.0", in that optional features may be added. If the version number (at offset 0x01AF) is numerically higher than the value 0x10, then code that implements "SFS Version 1.0" can leave the version number unchanged and provide read-only access to file data. This preserves any optional features added in later versions of SFS. Alternatively, code that implements "SFS Version 1.0" can replace the version number with 0x10 to automatically revert the file system such that it complies with this specification (any data in the file system that belongs to optional features added in later versions of SFS become discarded/ignored). In this case write access can be permitted (however it is recommended that the version number remain unchanged until the first modification is made, rather than changing it when the file system is "mounted"). Please note that in order to change the version number, the checksum field (at offset 0x01BD) must also be recalculated.

The size of the index area (at offset 0x01A4) indicates the size of the index area in bytes, and is also used to calculate the location of the start of the index area by subtracting the size of the index area from the total size of the media. This value is also used to determine the end of the free space area by calculating the start of the index area and rounding it down to the nearest block size. The size of the index area must be a multiple of 64 bytes.

The size of the data area (at offset 0x019C) indicates how large the data area currently is in blocks. It does not indicate how many blocks within the data area are actually used (for example, if files have been deleted it is likely that some blocks within the data area are not in use).

The time stamp field (at offset 0x0194) indicates when the size of the index area (at offset 0x01A4) or the size of the data area (at offset 0x019C) was changed last. This field is used is used in conjunction with the [[Volume Identifier]] to improve the detection of "media changed" on some hardware (e.g. old floppy drives). The time stamp is stored in the same format as all time stamps used by the Simple File System.

== Reserved Area ==
The reserved area (immediately following the super-block) may be used for any number of things that are outside the scope of this specification. Examples would include additional file information, boot code or data used for compatibility. All data within the reserved area must be ignored by all software that complies with this specification (i.e. even though it's reserved, it's never cleared when written to because it's never written to). The size of the reserved area can be determined from "number of reserved blocks" field (at offset 0x01B8) in the super-block.

It should be noted that the "number of reserved blocks" field (at offset 0x01B8 in the super-block) includes the super-block itself. This means that for 256 byte blocks this value must be 2 or higher, and for larger block sizes this value must be 1 or higher. The largest possible value in the "number of reserved blocks" field (at offset 0x01B8) can be calculated by subtracting 2 blocks (one for the index area and another for the data area) from the "Total number of blocks" field (at offset 0x01B0 in the super-block), unless the result is too large for a 32 bit unsigned integer (in which case the reserved area is limited to 0xFFFFFFFF blocks).

== Data Area Format ==
The data area is used to store all file data. The start of the data area is determined by the "number of reserved blocks" field in the super-block, and the end of the data area is determined by adding the "size of data area" and "number of reserved blocks" fields.

The data for any specific file consumes sequential blocks in the data area ([[file fragmentation]] is not supported). The data area may contain unused blocks, where files have been deleted or reduced in size, or unused blocks have been deliberately left to allow data to be appended to the preceding file. Different techniques for managing the blocks within the data area are discussed in Appendix A.

== Free Area ==
The free area is an area of unused blocks between the data area and index area. As new files are added to the file system, the data area below the free area grows and the index area above the free area grows, causing the free area to shrink from both ends.

The start of the free space area is determined by adding the "size of data area" and "number of reserved blocks" fields. The end of the free space area is determined by subtracting the size of the index area from the total size of the media and rounding it down to the nearest block size. It is assumed that these values would be calculated during initialization and updated in software after that.

== Index Area Format ==
The index area is used to manage the usage of blocks in the data area and keep track of file names.  The start of the index area is determined by subtracting the size of the index area from the total size of the media. The end of the index area is determined by the total size of the media.

The index area contains a variable number of 64 byte entries, where every file has one or more entries (but every entry doesn't necessarily represent a file). The format for these entries is described in the following section. The total number of entries stored in the index area can be determined by dividing the size of the index area by 64. Entries are not sorted in any way, except for continuation entries (which always immediately follow the entry they continue). For directories, entries for '.' and '..' are never stored in the file system and are assumed to exist when necessary.

There are 9 different types of index area entries:
* Volume Identifier (type 0x01)
* Starting Marker Entry (type 0x02)
* Unused Entry (type 0x10)
* Directory Entry (type 0x11)
* File Entry (type 0x12)
* Unusable Entry (type 0x18)
* Deleted Directory Entry (type 0x19)
* Deleted File Entry (type 0x1A)
* Continuation Entry (any value between 0x20 and 0xFF inclusive)

These entry types are described in the following sections.

Any index entry type not listed above must be treated as an unused entry (0x10), and may be replaced with the  entry type value 0x10 by an SFS 1.0 compliant code at any time.

Some of these entry types have specific positions within the index area. For example:
{| align="center" border="0" width="50%"
|- style="background-color:#FFDDAA;"
| Last block in free area
|- style="background-color:#AADDFF;"
| Empty space (present whenever the start of the index area isn't aligned to the start of a block)


|- style="background-color:#FFDDAA;"
| Starting marker entry
|- style="background-color:#AADDFF;"
| File entry 5
|- style="background-color:#FFDDAA;"
| Unused entry
|- style="background-color:#AADDFF;"
| File entry 4
|- style="background-color:#FFDDAA;"
| File entry 3
|- style="background-color:#AADDFF;"
| Continuation entry for file entry 4
|- style="background-color:#FFDDAA;"
| Deleted file entry
|- style="background-color:#AADDFF;"
| Continuation entry for deleted file entry
|- style="background-color:#FFDDAA;"
| Unusable entry
|- style="background-color:#AADDFF;"
| File entry 2
|- style="background-color:#FFDDAA;"
| Directory entry 1
|- style="background-color:#AADDFF;"
| Continuation entry for directory entry 1
|- style="background-color:#FFDDAA;"
| File entry 1
|- style="background-color:#AADDFF;"
| Volume identifier entry
|-
| End of media
|}
'''Table 3 - One possible index area'''

== Index Area - Volume Identifier ==
The very first entry (just below the end of the media) in the index area must be a [[Volume Identifier Entry]], which is used to detect if the media is changed on devices that don't have a reliable "media changed" facility (e.g. old floppy drives). There must only be one Volume Identifier Entry at the end of the media.

The Volume Identifier Entry uses the following format:
{| align="center" border="1" width="50%"
|- style="font-weight: bold;"
| Offset
| Size in Bytes
| Description
|-
| 0x00
| 1
| Entry type (0x01 for the Volume Identifier Entry)
|-
| 0x01
| 3
| Unused/reserved (must be zero)
|-
| 0x04
| 8
| Time stampe
|-
| 0x0C
| 52
| Volume name in [[UTF-8]], including zero terminator
|}
'''Table 4 - Volume Identifier Entry format'''

The entry type is used to determine that this is a Volume Identifier Entry and not any other type of index area entry.

The time stamp field indicates when the media was first formatted (it is not changed during normal operation). It is stored in the same format as all time stamps used by the Simple File System.

The volume name is limited to 52 bytes (including the terminator), and some characters are not allowed within a volume name. This name field follows the format for all name fields in the Simple File System.

== Index Area - Starting Marker Entry ==
The Starting Marker Entry is used to find the beginning of the index area during file system recovery (i.e. when the super-block is unusable). There must always be one Starting Marker Entry, and there must never be any index area entries closer to the start of the media than the Starting Marker Entry.

The Starting Marker Entry uses the following format:
{| align="center" border="1" width="50%"
|- style="font-weight: bold;"
| Offset
| Size in Bytes
| Description
|-
| 0x00
| 1
| Entry type (0x02 for the Starting Marker Entry)
|-
| 0x01
| 63
| Unused/reserved
|}
'''Table 5 - Starting Marker Entry format'''

The entry type is used to determine that this is a Starting Marker Entry and not any other type of index area entry.

== Index Area - Unused Entry ==
Unused Entries are used to mark an index area entry as unused (not to mark blocks within the data area as unused).

The format for an Unused Entry is:
{| align="center" border="1" width="50%"
|- style="font-weight: bold;"
| Offset
| Size in Bytes
| Description
|-
| 0x00
| 1
| Entry type (0x10 for Unused Entries)
|-
| 0x01
| 63
| Unused/reserved
|}
'''Table 6 - Unused Entry format'''

The entry type is used to determine that this is an Unused Entry and not any other type of index area entry. 

== Index Area - Directory Entry ==
Each Directory Entry in the index area uses the following format:
{| align="center" border="1" width="50%"
|- style="font-weight: bold;"
| Offset
| Size in Bytes
| Description
|-
| 0x00
| 1
| Entry type (0x11 for Directory Entries)
|-
| 0x01
| 1
| Number of continuation entries following this entry
|-
| 0x02
| 8
| Time stamp
|-
| 0x0A
| 54
| Directory name in UTF-8
|}
''Table 7 - Directory Entry format''

The entry type is used to determine that this is a [[Directory Entry]] and not any other type of index area entry.

The number of continuation entries is used to determine how many Continuation Entries immediately follow an entry when the directory name is too long to store in the Directory Entry alone. A value of zero indicates that there are no Continuation Entries. See the description of Continuation Entries for more information.

The time stamp field indicates when the Directory Entry information was last modified. It is stored in the same format as all time stamps used by the Simple File System. Optionally, the time stamp within a Directory Entry may be updated when a file or sub-directory within the associated directory is created, deleted, moved or renamed.

The directory name field is used to store the name of the directory, including the path. For example, for the directory "foo/bar" this field will contain the characters "foo/bar" followed by a zero to mark the end of the string (not just "bar"). If the directory name is too long to store in the Directory Entry, Continuation Entries are used. See the description of Continuation Entries for more information.

== Index Area - File Entry ==
Each File Entry in the index area uses the following format:
{| align="center" border="1" width="50%"
|- style="font-weight: bold;"
| Offset
| Size in Bytes
| Description
|-
| 0x00
| 1
| Entry type (0x12 for File Entries)
|-
| 0x01
| 1
| Number of continuation entries following this entry
|-
| 0x02
| 8
| Time stamp
|-
| 0x0A
| 8
| Starting block number in the data area
|-
| 0x12
| 8 
| Ending block number in the data area
|-
| 0x1A
| 8
| File length in bytes
|- 
| 0x22
| 30
| File name in [[UTF-8]]
|}
'''Table 8 - File Entry format'''

The entry type is used to determine that this is a File Entry and not any other type of index area entry.

The number of continuation entries is used to determine how many Continuation Entries immediately follow an entry when the file name is too long to store in the File Entry alone. A value of zero indicates that there are no Continuation Entries. See the description of Continuation Entries for more information.

The time stamp field indicates when the File Entry information or associated file data was last modified. It is stored in the same format as all time stamps used by the Simple File System.

The starting block number and ending block number is used to find the start and end of the region within the data area used to store the file's data. The ending block number must be higher than the starting block number, unless the entry does not use any blocks in the data area (a zero length file) in which case the starting and ending block numbers should both be zero.

The file length records how long a file actually is. This value must not be larger than the starting and ending block numbers indicate, but may be significantly less to reserve space for data to be appended to a file (and will often be slightly less because most file lengths aren't a multiple of the block size).

The file name field is used to store the name of the file, including the path. For example, for the file "foo/bar.txt" this field will contain the characters "foo/bar.txt" followed by a zero to mark the end of the string (not just "bar.txt"). If the file name is too long to store in the File Entry, Continuation Entries are used. See the description of Continuation Entries for more information.

== Index Area - Unusable Entry ==
An Unusable Entry is used to mark unusable areas of the media (for example, bad sectors), and should not be confused with Unused Entries.

Each Unusable Entry in the index area uses the following format:
{| align="center" border="1" width="50%"
|- style="font-weight: bold;"
| Offset
| Size in Bytes
| Description
|-
| 0x00
| 1
| Entry type (0x18 for Unusable Entries)
|-
| 0x01
| 9
| Unused/reserved
|-
| 0x0A
| 8
| Starting block number in the data area
|-
| 0x12
| 8
| Ending block number in the data area
|-
| 0x1A
| 38
| Unused/reserved
|}
'''Table 9 - Unusable Entry format'''

The entry type is used to determine that this is an Unusable Entry and not any other type of index area entry.

The starting block number and ending block number is used to find the start and end of the region within the data area that is unusable. The ending block number must be higher than the starting block number.

== Index Area - Deleted Directory Entry ==

Each Deleted Directory Entry in the index area uses the following format:
{| align="center" border="1" width="50%"
|- style="font-weight: bold;"
| Offset
| Size in Bytes
| Description
|-
| 0x00
| 1
| Entry type (0x19 for Deleted Directory Entries)
|-
| 0x01
| 1
| Number of continuation entries following this entry
|-
| 0x02
| 8
| Time stamp
|-
| 0x0A
| 54
| Directory name in UTF-8
|}
'''Table 10 - Deleted Directory Entry format'''

The entry type is used to determine that this is a Deleted Directory Entry and not any other type of index area entry.

The time stamp field indicates when the original Directory Entry was deleted (or when the Deleted Directory Entry was created). It is stored in the same format as all time stamps used by the Simple File System.

All other fields within a Deleted Directory Entry must remain the same as the original Directory Entry to allow the directory to be restored by changing the entry type back to 0x11 (Directory Entry).

== Index Area - Deleted File Entry ==

Each File Entry in the index area uses the following format:
{| align="center" border="1" width="50%"
|- style="font-weight: bold;"
| Offset
| Size in Bytes
| Description
|-
| 0x00
| 1
| Entry type (0x1A for Deleted File Entries)
|-
| 0x01
| 1
| Number of continuation entries following this entry
|-
| 0x02
| 8
| Time stamp
|-
| 0x0A
| 8
| Starting block number in the data area
|-
| 0x12
| 8
| Ending block number in the data area
|-
| 0x1A
| 8
| File length in bytes
|-
| 0x22
| 30
| File name in [[UTF-8]]
|}
'''Table 11 - Deleted File Entry format'''

The entry type is used to determine that this is a Deleted File Entry and not any other type of index area entry.

The time stamp field indicates when the original File Entry was deleted (or when the Deleted File Entry was created). It is stored in the same format as all time stamps used by the Simple File System.

All other fields within a Deleted File Entry must remain the same as the original File Entry to allow the file to be restored by changing the entry type back to 0x12 (File Entry). This includes all blocks within the data area.

== Index Area - Continuation Entry ==

Continuation entries are used when the name field in a preceding Directory Entry, File Entry, Deleted Directory Entry or Deleted File Entry is not long enough to contain the entire name. This forms a consecutive series of entries, beginning with a initial entry (the closest entry to the start of the media) followed by none or more continuation entries.

The format for a continuation entry is:
{| align="center" border="1" width="50%"
|- style="font-weight: bold;"
| Offset
| Size in Bytes
| Description
|-
| 0x00
| 64
| Entry name in UTF8
|}
'''Table 12 - Continuation Entry format'''

To allow for much longer entry names, multiple continuation entries can be used. The number of continuation entries is stored in the associated Directory Entry, File Entry, Deleted Directory Entry or Deleted File Entry. Because this field is limited to a maximum value of 255, the number of continuation entries is limited to 255. This provides for a maximum directory name length of 16374 bytes including the zero terminator, and a maximum file name length of 16350 bytes including the zero terminator. However, it is recommended that directory names and file names be limited to a reasonable value when they are created.

Because bytes with values less than 32 aren't possible within directory names and file names, it is possible to determine if an entry is a continuation entry or any other type of entry by checking the first byte of the entry. If the first byte is below 0x20 the entry is not a continuation entry, and if the first byte is above or equal to 0x20 then it must be a continuation entry.

== Time Stamps ==
All time stamps are signed 64 bit values that represent the number of 1/65536ths of a second since the beginning of the 1st of January 1970. For example, the value 0x00000000003C0000 would represent one minute past midnight on the 1st of January 1970, while the value 0x0000000000000001 would represent roughly 15259 ns past midnight on the 1st of January 1970. All time stamps are in UTC (Universal Co-ordinated Time) so that problems with time zones and daylight savings are avoided.

This format is similar to the typical time stamp format used by Unix systems (a signed 32 bit value representing seconds since since the beginning of the 1st of January 1970), except that it is a lot more accurate (15.25 us intervals) and a lot less limited (a range of roughly 4.46 million years before and after 1970, instead of 68 years before and after 1970). The additional accuracy can avoid problems with some utilities, and the extra range avoids the problem that most Unix based file systems will have in the year 2038.

== Name Strings ==
All name strings used as part of the Simple File System use UTF-8 format, according to the Unicode Specification published by the Unicode Consortium.

Some characters are not allowed within a file name, directory name or volume name. These characters include all characters with codes below 32 (excluding the space character, code 0x0020), all characters with codes between 0x0080 and 0x009F (inclusive), and any of the following characters:
* " (double quote, 0x0022)
* * (asterix, 0x002A)
* : (colon, 0x003A)
* < (less than sign, 0x003C)
* > (greater than sign, 0x003E)
* ? (question mark, 0x003F)
* \ (backward slash, 0x005C)
* DEL (delete, 0x007F)
* NBSP (no break space character, 0x00A0)

For the "no break space" character (code 0x00A0), it is expected that the file system driver (or operating system code) silently replaces it with a normal space character (code 0x0020). This is intended to reduce confusion (as both characters look the same).

The forward slash character ('/', code 0x002F) must be used as a directory separator only, and is not allowed within a volume label. It is expected that some systems will silently replace these characters with a backward slash character ('\', code 0x005C) to maintain consistency with the intended environment.

All names must end with a [[Null_Character|null character]] (0x0000). It is also recommended that any unused bytes within any name field contain the byte 0x00.

== Compatibility ==
The Simple File System has been carefully designed to co-exist with a legacy [[FAT]] file system. To achieve this the reserved area at the beginning of the Simple File System is made large enough to cover the entire FAT file system, with the remainder of the media used by the Simple File System.

This ability makes it possible to have a floppy disk or flash memory device (for example) with a small FAT file system  that contains any instructions, drivers or files needed to access the Simple File System on the rest of the media.

== Appendix A - Algorithms ==

[TODO - need algorithms for common operations, with the simplest possible code and most secure code (with suggestions for optimization and guidelines for things like directory entry usage and "compaction")]

[[Category:Filesystems]]
````

<a id="w002606"></a>

## W002606 — User:Walling

Bron-ID: W002606
Namespace: User (2)
Revisie: 8159 | Datum: 2009-06-04T14:33:54Z
Bron: https://wiki.osdev.org/User:Walling
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AWalling&oldid=8159

````text
__NOTOC__
I'm '''Bjarke Walling''' from Denmark. Please [http://forum.osdev.org/memberlist.php?mode=viewprofile&u=2204 view my forum profile].

I'm not very active in OS development at the moment, but I'll sure release something some time. If you need to contact me, please do so [mailto:bwp@bwp.dk by e-mail] or a private message in the forum.

==User pages==

There's the [[User:Walling/Archive|archive]], and other pages: {{Special:Prefixindex/User:Walling}}
User discussions: {{Special:Prefixindex/User_talk:Walling}}
````

<a id="w002607"></a>

## W002607 — User talk:Walling

Bron-ID: W002607
Namespace: User talk (3)
Revisie: 8163 | Datum: 2009-06-04T14:37:35Z
Bron: https://wiki.osdev.org/User_talk:Walling
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AWalling&oldid=8163

````text
''There's an [[User talk:Walling/Archive|archive]] of earlier discussions.''
----
````

<a id="w002608"></a>

## W002608 — User talk:Walling/Archive

Bron-ID: W002608
Namespace: User talk (3)
Revisie: 8162 | Datum: 2009-06-04T14:36:53Z
Bron: https://wiki.osdev.org/User_talk:Walling/Archive
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AWalling%2FArchive&oldid=8162

````text
This is [[User:Walling|Walling's]] archive of old discussion pages:

* [[/Discussion1]] - archived 2009-06-04
````

<a id="w002610"></a>

## W002610 — Talk:Security

Bron-ID: W002610
Namespace: Talk (1)
Revisie: 20432 | Datum: 2017-02-15T15:35:22Z
Bron: https://wiki.osdev.org/Talk:Security
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ASecurity&oldid=20432

````text
== Ring 1 and 2 ==
The description for ring 1 and 2 is: "They offer more protection, but not as much as ring 3." I feel that this is description is inappropriate for an OS dev website where you may find a user that has a purpose to use those ring levels. After a lot of searching I have failed to find any explination that clearly describes how the CPU handles those two rings (99% of cases it says 'they are rarely used' or 'they offer a little more protection than ring 0' but in exactly what ways?)

Having the description extended to include exact technical details of those ring levels would be highly appreciated. --[[User:Messiahandrw|Messiahandrw]] 08:16, 19 August 2009 (UTC)

I think the Processor Manuals have a summary of the features available on each privilege level, and if I remember things correctly, some instruction are strictly CPL==0, while others require CPL <= IOPL. Then we have the access checks imposed by the DPL and if the conforming bit is set.

So technically, it should be possible to set IOPL to 1, and have the kernel in PL0 and device drivers (either because you don't fully trust them or by default) in PL1, making it possible to protect the kernel a little more from malicious or buggy device drivers.

Note that running device ISR's at PL1 would require special coding (basically a variation on the callback theme, only that you call PL1 code instead of PL3 code), which causes more clock cycles to be used. I could elaborate on this a little more if asked.
--[[User:Teodor väänänen|Teodor väänänen]] 10:20, 19 August 2009 (UTC)

== High level mechanisms ==

This needs more stuff about high level security, such as managing users and how users own processes and files. Also handling considerations such as "how would 'sudo' or 'su' work?" will there be sessions or just a simple process list? By what way will users actually login to your system? --[[User:Earlz|Earlz]] 04:46, 5 June 2009 (UTC)

== ASLR ==

Looks like ASLR can easily be worked around on architectures with a cache.
https://arstechnica.com/security/2017/02/new-aslr-busting-javascript-is-about-to-make-drive-by-exploits-much-nastier/
Should the page be edited to show this? --[[User:Matt11235|Matt11235]] 09:35, 15 February 2017 (CST)
````

<a id="w002612"></a>

## W002612 — Real mode assembly appendix A

Bron-ID: W002612
Namespace: Main (0)
Revisie: 27912 | Datum: 2023-07-09T05:05:57Z
Bron: https://wiki.osdev.org/Real_mode_assembly_appendix_A
Revisie-URL: https://wiki.osdev.org/index.php?title=Real_mode_assembly_appendix_A&oldid=27912

````text
{{FirstPerson}}
''This is the appendix of the real mode assembly bare bones series, containing nifty tidbits of information, code, and ideas.''

==Links to other articles with code==
* [[Printing to the screen without a db]]
* [[Far Call Trick]]

==Code==
===Text in 640x480 16-colour graphics mode===
VGA video mode 12h is a 640x480 resolution, in which (with a little tweaking) can be used with your string printing function. All you have to do is add a <tt>mov bh,07h</tt> to allow regular coloured printing in graphics mode. At the moment, the cursor is disabled. You must find a way to eliminate it or do up a function specifically for printing in graphics mode.

===80x60 text mode===
As far as I can tell, there is no way to do this with standard VGA modes. If anyone has done this, please tell by emailing me at blasterman95 _AT_ gmail _DOT_ com or by PM'ing me on the forums! Thanks!

[[Category:Assembly]]
[[Category:Real mode assembly]]
````

<a id="w002613"></a>

## W002613 — User talk:Essial

Bron-ID: W002613
Namespace: User talk (3)
Revisie: 8175 | Datum: 2009-06-06T08:04:07Z
Bron: https://wiki.osdev.org/User_talk:Essial
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AEssial&oldid=8175

````text
Hi, I've reverted your edits on the Beginner Mistakes page. Writing a bootloader or not is a controversial argument that will go on to the end of time. It's not a beginner mistake, just an opinion. [[User:Jackscott|JackScott]] 08:04, 6 June 2009 (UTC)
````

<a id="w002614"></a>

## W002614 — Talk:Building MPFR

Bron-ID: W002614
Namespace: Talk (1)
Revisie: 8186 | Datum: 2009-06-08T04:47:32Z
Bron: https://wiki.osdev.org/Talk:Building_MPFR
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABuilding_MPFR&oldid=8186

````text
This article needs a quick blurb on detecting the package version that is already installed, for both linux & cygwin. [[User:Bewing|Bewing]] 04:47, 8 June 2009 (UTC)
````

<a id="w002615"></a>

## W002615 — Talk:Getting to Ring 3

Bron-ID: W002615
Namespace: Talk (1)
Revisie: 12832 | Datum: 2012-02-26T07:34:25Z
Bron: https://wiki.osdev.org/Talk:Getting_to_Ring_3
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AGetting_to_Ring_3&oldid=12832

````text
Isn't that external link to Germsoft a near exact rip of JamesM's tutorial? --[[User:Earlz|Earlz]] 04:52, 14 October 2009 (UTC)

Good start, but needs more detail. Explain the what and why, and / or put in references to the relevant chapters in the Intel Manuals, so that a reader can understand *what* is necessary, and *why* it should be done this way. In the end, a tutorial should resemble a textbook chapter, not a forum posting.

But as I said, a good start. Do not let me discourage you. -- [[User:Solar|Solar]] 08:06, 9 June 2009 (UTC)

JamesM gave me permission to use the old revision with code based off of his tutorial, so I'm rolling back to it. --[[User:Earlz|Earlz]] 01:34, 26 February 2012 (CST)
````

<a id="w002616"></a>

## W002616 — File:PortRoutingBlockDiagram.gif

Bron-ID: W002616
Namespace: File (6)
Revisie: 8206 | Datum: 2009-06-10T18:47:14Z
Bron: https://wiki.osdev.org/File:PortRoutingBlockDiagram.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3APortRoutingBlockDiagram.gif&oldid=8206

````text
EHCI Port Routing Block Diagram
````

<a id="w002617"></a>

## W002617 — File:USBSystem.gif

Bron-ID: W002617
Namespace: File (6)
Revisie: 8207 | Datum: 2009-06-10T19:04:16Z
Bron: https://wiki.osdev.org/File:USBSystem.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AUSBSystem.gif&oldid=8207

````text
USB System Illustration
````

<a id="w002618"></a>

## W002618 — File:LFSpeedDevToHSPort.gif

Bron-ID: W002618
Namespace: File (6)
Revisie: 8208 | Datum: 2009-06-10T19:08:26Z
Bron: https://wiki.osdev.org/File:LFSpeedDevToHSPort.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ALFSpeedDevToHSPort.gif&oldid=8208

````text
Low- or Full-speed device connected to a high-speed capable USB port
````

<a id="w002619"></a>

## W002619 — File:HSHubToHSPort.gif

Bron-ID: W002619
Namespace: File (6)
Revisie: 8209 | Datum: 2009-06-10T19:11:28Z
Bron: https://wiki.osdev.org/File:HSHubToHSPort.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AHSHubToHSPort.gif&oldid=8209

````text
High-speed hub connected to a high-speed capable USB port
````

<a id="w002620"></a>

## W002620 — File:HSDevsToHSHubToHSPort.gif

Bron-ID: W002620
Namespace: File (6)
Revisie: 8210 | Datum: 2009-06-10T19:12:55Z
Bron: https://wiki.osdev.org/File:HSDevsToHSHubToHSPort.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AHSDevsToHSHubToHSPort.gif&oldid=8210

````text
High-speed devices connected to a high-speed hub which is connected to a high-speed USB port.
````

<a id="w002621"></a>

## W002621 — File:LFHSDevsHSHubHSPortIncorrect.gif

Bron-ID: W002621
Namespace: File (6)
Revisie: 8211 | Datum: 2009-06-10T19:16:00Z
Bron: https://wiki.osdev.org/File:LFHSDevsHSHubHSPortIncorrect.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ALFHSDevsHSHubHSPortIncorrect.gif&oldid=8211

````text
Incorrect illustration of Low- and Full-speed devices on a high-speed bus.
````

<a id="w002622"></a>

## W002622 — File:LFHSDevsHSHubHSPortCorrect.gif

Bron-ID: W002622
Namespace: File (6)
Revisie: 8212 | Datum: 2009-06-10T19:16:33Z
Bron: https://wiki.osdev.org/File:LFHSDevsHSHubHSPortCorrect.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ALFHSDevsHSHubHSPortCorrect.gif&oldid=8212

````text
Correct illustration of split transactions allowing Low- and Full-speed devices on a high-speed bus.
````

<a id="w002623"></a>

## W002623 — File:USBTopology.gif

Bron-ID: W002623
Namespace: File (6)
Revisie: 8213 | Datum: 2009-06-10T19:20:24Z
Bron: https://wiki.osdev.org/File:USBTopology.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AUSBTopology.gif&oldid=8213

````text
USB Topology
````

<a id="w002624"></a>

## W002624 — File:USBDataFlow.gif

Bron-ID: W002624
Namespace: File (6)
Revisie: 8214 | Datum: 2009-06-10T19:26:05Z
Bron: https://wiki.osdev.org/File:USBDataFlow.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AUSBDataFlow.gif&oldid=8214

````text
Illustration of USB Communication Flow
````

<a id="w002625"></a>

## W002625 — OSDev Wiki:Setup

Bron-ID: W002625
Namespace: OSDev Wiki (4)
Revisie: 29143 | Datum: 2024-06-22T23:20:39Z
Bron: https://wiki.osdev.org/OSDev_Wiki:Setup
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDev_Wiki%3ASetup&oldid=29143

````text
This wiki is hosted on an Ampere ARM Compute instance running Ubuntu and MariaDB in Oracle Cloud. Cloudflare is used for a CDN.
````

<a id="w002626"></a>

## W002626 — 802.11

Bron-ID: W002626
Namespace: Main (0)
Revisie: 21595 | Datum: 2017-10-01T21:45:18Z
Bron: https://wiki.osdev.org/802.11
Revisie-URL: https://wiki.osdev.org/index.php?title=802.11&oldid=21595

````text
{{Stub}}
The '''IEEE 802.11''' standards are a set that describe wireless LAN communications. The 802.11 frequency bands are currently 2.4, 3.6, and 5 GHz. The standard is primarily developed by the IEEE 802.11 Working Group.

== Protocols and Revisions ==
=== 802.11 (1997 original) ===
The first version of the 802.11 standard, released in 1997 and now known as 802.11 Legacy, specifies wireless information transfer at speeds of up to either one or two megabits per second (Mbps) in the 2.4 GHz range.

=== 802.11a ===
Released in 1999, the 802.11a standard specifies communication in the 5 GHz band at speeds of up to 54 Mbps.

=== 802.11b ===
The 802.11b standard has a maximum theoretical data transfer speed of 11 Mbps using the 2.4 GHz band used in the original 802.11 standard.

=== 802.11g ===
802.11g also uses the 2.4 GHz band, but at speeds of up to 54 Mbps. 802.11g is the most-used of all 802.11 standards as of June 2009.

=== 802.11n ===
IEEE 802.11n supports theoretical speeds of up to 600 Mbps in both the 5 and 2.4 GHz bands. The first 802.11n devices were implemented using Draft 2 of the standard.

=== 802.11ac ===
IEEE 802.11ac supports speeds of up to 1300Mbps.

== "Wi-Fi" ==
Wi-Fi is a trademark owned by the Wi-Fi Alliance to mark certified IEEE 802.11 products. The term "Wi-Fi" is often used as a general noun for wireless networking and communications using the 802.11 standards.

== See Also ==
=== Articles ===

=== Threads ===

=== External Links ===
* [http://www.ieee802.org/11/ Website of the IEEE 802.11 Working Group]
* [http://standards.ieee.org/getieee802/802.11.html Download the 802.11 standards]
* [[wikipedia:IEEE 802.11 | IEEE 802.11]] at Wikipedia.

[[Category:Network_Protocols]]
````

<a id="w002627"></a>

## W002627 — User talk:Jamesm

Bron-ID: W002627
Namespace: User talk (3)
Revisie: 8264 | Datum: 2009-06-21T00:56:16Z
Bron: https://wiki.osdev.org/User_talk:Jamesm
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AJamesm&oldid=8264

````text
<math>U = \sum_{i\inProcesses}\frac{C_{i}}{P_{i}}</math>
````

<a id="w002628"></a>

## W002628 — Stack Smashing Protector

Bron-ID: W002628
Namespace: Main (0)
Revisie: 20864 | Datum: 2017-06-05T19:46:40Z
Bron: https://wiki.osdev.org/Stack_Smashing_Protector
Revisie-URL: https://wiki.osdev.org/index.php?title=Stack_Smashing_Protector&oldid=20864

````text
The '''Stack Smashing Protector''' ('''SSP''') compiler feature helps detect stack buffer overrun by aborting if a secret value on the stack is changed. This serves a dual purpose in making the occurrence of such bugs visible and as exploit mitigation against return-oriented programming. SSP merely detects stack buffer overruns, they are not prevented. The detection can be beaten by preparing the input such that the stack canary is overwritten with the correct value and thus does not offer perfect protection. The stack canary is native word sized and if chosen randomly, an attacker will have to guess the right value among 2^32 or 2^64 combinations (and revealing the bug if the guess is wrong), or resort to clever means of determining it.

== Description ==

Compilers implement this feature by selecting appropriate functions, storing the stack canary during the function prologue, checking the value in the epilogue, and invoking a failure handler if it was changed. For instance, consider the code:
<source lang="c">
void foo(const char* str)
{
	char buffer[16];
	strcpy(buffer, str);
}
</source>
SSP automatically illustratively transforms that code into this:
<source lang="c">
/* Note how buffer overruns are undefined behavior and the compilers tend to
   optimize these checks away if you wrote them yourself, this only works
   robustly because the compiler did it itself. */
extern uintptr_t __stack_chk_guard;
noreturn void __stack_chk_fail(void);
void foo(const char* str)
{
	uintptr_t canary = __stack_chk_guard;
	char buffer[16];
	strcpy(buffer, str);
	if ( (canary = canary ^ __stack_chk_guard) != 0 )
		__stack_chk_fail();
}
</source>
Note how the secret value is stored in a global variable (initialized at program
load time) and is copied into the stack frame, and how it is safely erased from
the stack as part of the check. Since stacks grow downwards on many architectures,
the canary gets overwritten whenever input to strcpy is more than 16 characters.
The caller return pointer exploited in return-oriented programming attacks is
not accessed until after the value was validated, thus preventing such attacks.

The detection method works because it is impossible to get the correct value via
trial and error. Since one incorrect canary value prevents further alterations,
an attacker cannot keep trying until the correct value is found. In the example
above, if the canary contained a zero byte, it would be impossible to guess its
existence and position by trial and error. This forces the attacker to either
not attack, or be detected and be unable to alter the stack any further. This
does not mean that the buffer cannot be exploited. For example, if 16 bytes are
written to the buffer above and it is not null-terminated, unintended behaviour
can still take place later on during program execution.

Note how there is only a single protective value, not every variable is
protected in this manner. One heuristic ordering often used, with the stack
growing downwards, is first storing the canary, then buffers (that might
overflow into each other) and finally all the small variables unaffected by
overruns. This is based on the idea that it is generally less dangerous if
arrays are modified, compared to variables that hold flags, pointers and
function pointers, which may more seriously alter execution.

Some compilers randomize the order of stack variables and randomize the stack
frame layout, which further complicates determining the right input with the
intended malicious effect.

== Usage ==

Compilers such as [[GCC]] enable this feature if requested through compiler
options, or if the compiler supplier enabled it by default. It is worth
considering enabling it by default if your operating system is security
conscious and you provide support. It is possible to use it in your entire
operating system (even kernel and standard library, perhaps excluding ports with
really poor code quality). A feature enabled with a <tt>-ffoo</tt> option can be
disabled with the <tt>-fno-foo</tt> counterpart. Several options exist that
provide different variants of SSP:

'''-fstack-protector''': Check for stack smashing in functions with vulnerable
objects. This includes functions with buffers larger than 8 bytes or calls to
alloca.

'''-fstack-protector-strong''': Like <tt>-fstack-protector</tt>, but also
includes functions with local arrays or references to local frame addresses.

'''-fstack-protector-all''': Check for stack smashing in every function.

Some operating systems have extended their compiler with more relevant options:

'''-fstack-shuffle''': (Found in OpenBSD) Randomize the order of stack variables
at compile time. This helps find bugs.

When you activate the feature, the compiler will attempt to link in libssp and
libssp_nonshared (if statically linked) for run-time support. This is disabled
if you pass <tt>-nostdlib</tt> as you do when linking a kernel and you'll need to
supply your own implementation. For user-space, you have two options:

* Supply your own implementation in libc (so libc can take advantage of the feature) and install empty libssp and libssp_nonshared libraries (or change your toolchain to not use them).
* Use the libssp implementation that comes with GCC.

It should also be noted that with the optimisations enabled via <tt>-O<n></tt> in GCC, the compiler may or may not "inline" a function. If a function has been inlined, then '''stack smash protection will not work for that function.''' To prevent this, one must use the <tt>noinline</tt> attribute like so:
<source lang="c">
void __attribute__ ((noinline)) foo( /* args */ )
{
    // Code goes here
}
</source>

Disabling inlining in GCC can be done with the <tt>-fno-inline</tt> compile flag, however, that will not inline functions with the <tt>inline</tt> attribute. The <tt>-fno-inline-functions</tt> will not inline functions optimised with <tt>-O<n></tt>; but that has been proven ineffective for GCC versions 3.4.5 and over ([https://gcc.gnu.org/bugzilla/show_bug.cgi?id=28120 see bug report]).

If any tests do not work when trying to trip the protective mechanism, this may be the reason why it does not work!

== Implementation ==

Run-time support needs only two components: A global variable and a check
failure handler. For instance, a minimal implementation could be:

<source lang="c">
#include <stdint.h>
#include <stdlib.h>

#if UINT32_MAX == UINTPTR_MAX
#define STACK_CHK_GUARD 0xe2dee396
#else
#define STACK_CHK_GUARD 0x595e9fbd94fda766
#endif

uintptr_t __stack_chk_guard = STACK_CHK_GUARD;

__attribute__((noreturn))
void __stack_chk_fail(void)
{
#if __STDC_HOSTED__
	abort();
#elif __is_myos_kernel
	panic("Stack smashing detected");
#endif
}

</source>

Note how the secret guard value is hard-coded rather than being decided during
program load. You should have the program loader (the bootloader in the case of
the kernel) randomize the values. You can do this by putting the guard value in
a special segment that the loader knows to randomize. The numbers shown here are
not special, they are just examples of randomly generated numbers. You can still
take advantage of the bug-discovering properties of SSP even if the guard value
is not cryptographically secure (unless you anticipate sufficiently obscure bugs
that intelligently circumvent SSP).

Alternatively, you could have an early phase in your code that initializes the
guard value, perhaps written in assembly or in C but built without stack smash
protection. This approach adds code complexity and early phases where language
features are not online. You may take such approaches with thread-local storage,
errno, paging, GDT, scheduling, and so on, and suddenly a bootstrap is very
complex with many dependencies between language features. Once a function built
with stack-smashing protection is run, the guard value cannot be changed or a
spurious failure will occur.

== Secure Handling ==

Beware how you implement the stack smash detection handler: This code is only
run in cases where the bug was triggered innocently, or where the bug is being
exploited maliciously. By now the attacker is assumed to have at least
corrupted an unknown amount of this thread's stack. This means the environment
is hostile. The stack is currently under your control and none of the new local
variables are affected. Note however that the stack smash protection may have
occurred from a signal handler or another inopportune time where another thread
holds locks to critical standard library state or such. Beware how if pointers
to caller stack variables are currently inside the standard library, and using
standard library functions accesses that memory, the attacker may control the
stack smash detection handler even.

Assuming a handler invocation implies an intelligent exploit is happening, the
best course of action is is:

* Eliminate attacker influence.
* Alert user or system administrator of a potential breach.
* Diagnose the details of the buffer overrun so the defect can be fixed.

You should assume the worst if you wish to eliminate the attacker influence. The
used exploit may well be combined with other exploited vulnerabilities, and a
sufficiently skilled attacker may even influence and exploit the actions of the
handler. There are many creative ways an attacker could influence the handler or
even take advantage of it:

* Pointers to earlier stack variables (now to be considered potentially corrupted) could be stored somewhere and accessed by the functions you use.
* The handler could be run at a very inopportune time where the process is fragile, perhaps from a signal handler, perhaps the current thread owns non-recursive locks you could deadlock.
* Printing a stack trace (if at all possible) and other diagnostic information to the stderr file descriptor (which might not even exist in this process, but instead fd 2 is used for another purpose) might result in the output being sent to the attacker. This is imaginable for a webserver, which perhaps includes the stderr contents in an error response. The attacker could learn things this way he isn't supposed to.
* The thread might be multi-threaded and who knows how that might interact with a thread that is malfunctioning and compromised. It could have pointers to variables on the stack of the compromised thread, and SSP won't help if it accesses those.

Your approach should be to discard the process as soon as possible. Use only
async-signal-safe functions, preferably without state that could influence them.
Don't write to any standard streams but open the terminal anew or write to the
system log. Ensure none of these operations fail (for instance, if the process
is in a chroot or out of file descriptors).

The ideal approach is perhaps to have a special system call that does these
tasks and invoke it unconditionally and immediately. Kernel code must not trust
user-space code or be unsafely influenced by it, so it can be considered
safe. It can then stop all threads in the process, investigate where the issue
seemed to occur in the process, and alert the user or system administrator
appropriately.

== libssp ==

Alternatively to your own implementation, you can use the implementation that comes with GCC. This means you have to build libssp as part of your toolchain.

'''TODO''': I have never built it for osdev purposes before, but I guess that you do <tt>make all-target-libssp</tt> and <tt>make install-target-libssp</tt> like with libstdc++. It's probable that depends on libc for no good reason at all (as the gcc developers put fortify source functions in it and it wants to check whether they work).

The libssp approach is to have an initialization function marked as attribute constructor, which is run among the global constructors during process startup. This means SSP isn't properly online during the early parts of process initialization (but perhaps that's not a problem if all those C stack frames are gone before that point and the default null guard value was used until now). The startup code then proceeds to attempt opening <tt>/dev/urandom</tt> which might fail if you are in a chroot, are out of file descriptors, or your system doesn't have such a file (perhaps by design). If it fails, it falls back on a reasonable but known value. You can read the [https://gcc.gnu.org/viewcvs/gcc/trunk/libssp/ssp.c?view=markup#l67 libssp initialization code here].

The libssp <tt>__stack_chk_fail</tt> implementation tries to open the terminal, construct an error message with alloca, then use write to output it. If the terminal isn't accessible, it tries the system log. It then attempts to destroy the process by invoking <tt>__builtin_trap()</tt>, writing a 0 to the int at -1 (which is also undefined behavior and an unaligned pointer, in addition to probably crashing), and finally attempting to <tt>_exit().</tt> This exiting strategy doesn't feel super robust. You can read the [https://gcc.gnu.org/viewcvs/gcc/trunk/libssp/ssp.c?view=markup#l96 libssp handler code here].

Read the secure handling section above and read the code, then decide whether you want this linked into your programs, or whether it is cleaner to make your own implementation. You can also modify this code as part of your [[OS Specific Toolchain]].

== See Also ==
=== Articles ===
* [[Undefined_Behavior_Sanitization|Undefined Behavior Sanitization]]

=== Threads ===
*[[Topic:28571|GCC stack protector - false positive?]]

=== External Links ===
* [https://gcc.gnu.org/onlinedocs/gcc/Optimize-Options.html GCC optimization options documentation] in which <tt>-fstack-protector</tt> is detailed
* [http://www.trl.ibm.com/projects/security/ssp/ GCC extension for protecting applications from stack-smashing attacks]
* [[wikipedia:Buffer overflow protection|Buffer overflow protection]] on Wikipedia

[[Category:Security]]
````

<a id="w002629"></a>

## W002629 — File:SuccessfulDataTx.gif

Bron-ID: W002629
Namespace: File (6)
Revisie: 8276 | Datum: 2009-06-24T22:39:49Z
Bron: https://wiki.osdev.org/File:SuccessfulDataTx.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ASuccessfulDataTx.gif&oldid=8276

````text
Illustration of how a sending and receiving device each manage their data toggle bits during a successful data transmission.
````

<a id="w002630"></a>

## W002630 — File:FailedDataTx.gif

Bron-ID: W002630
Namespace: File (6)
Revisie: 10430 | Datum: 2010-06-14T05:43:57Z
Bron: https://wiki.osdev.org/File:FailedDataTx.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AFailedDataTx.gif&oldid=10430

````text
Illustration of how a sending and receiving device each manage their data toggle bits during a corrupted or failed data transmission.
````

<a id="w002631"></a>

## W002631 — File:FailedACKTx.gif

Bron-ID: W002631
Namespace: File (6)
Revisie: 8278 | Datum: 2009-06-24T22:41:23Z
Bron: https://wiki.osdev.org/File:FailedACKTx.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AFailedACKTx.gif&oldid=8278

````text
Illustration of how a sending and receiving device each manage their data toggle bits during a failed ACK response.
````

<a id="w002632"></a>

## W002632 — File:Usbframes.gif

Bron-ID: W002632
Namespace: File (6)
Revisie: 8279 | Datum: 2009-06-25T02:43:36Z
Bron: https://wiki.osdev.org/File:Usbframes.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AUsbframes.gif&oldid=8279

````text
Illustration of USB (micro)frames.
````

<a id="w002633"></a>

## W002633 — File:Usbtransactionreorder.gif

Bron-ID: W002633
Namespace: File (6)
Revisie: 8280 | Datum: 2009-06-25T04:38:00Z
Bron: https://wiki.osdev.org/File:Usbtransactionreorder.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AUsbtransactionreorder.gif&oldid=8280

````text
Illustrates how a host controller may potentially reorder a bulk transfer and two control transfers on the USB.
````

<a id="w002634"></a>

## W002634 — File:SETUPTransaction.gif

Bron-ID: W002634
Namespace: File (6)
Revisie: 8287 | Datum: 2009-06-25T17:22:49Z
Bron: https://wiki.osdev.org/File:SETUPTransaction.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ASETUPTransaction.gif&oldid=8287

````text
Illustrates a SETUP transaction, which consists of a SETUP packet, a DATA0 packet, and a handshake packet (in this case, an ACK handshake).
````

<a id="w002635"></a>

## W002635 — File:ControlTx.gif

Bron-ID: W002635
Namespace: File (6)
Revisie: 8288 | Datum: 2009-06-25T23:07:35Z
Bron: https://wiki.osdev.org/File:ControlTx.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AControlTx.gif&oldid=8288

````text
Taken from Figure 8-37 of the USB 2.0 specifications.  Illustrates the transaction order, data sequence bit value, and DATA PID type for control read and write sequences.
````

<a id="w002636"></a>

## W002636 — File:BulkInterruptTx.gif

Bron-ID: W002636
Namespace: File (6)
Revisie: 8289 | Datum: 2009-06-26T00:07:33Z
Bron: https://wiki.osdev.org/File:BulkInterruptTx.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ABulkInterruptTx.gif&oldid=8289

````text
Taken from Figure 8-35 of the USB 2.0 Specifications.  Illustrates bulk/interrupt transactions.
````

<a id="w002637"></a>

## W002637 — File:HSHBIsoIN.gif

Bron-ID: W002637
Namespace: File (6)
Revisie: 8290 | Datum: 2009-06-26T02:47:54Z
Bron: https://wiki.osdev.org/File:HSHBIsoIN.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AHSHBIsoIN.gif&oldid=8290

````text
Taken from figure 5-11 of the USB 2.0 specifications.  Illustrates Data Phase PID Sequence for Isochronous IN High-Bandwidth Endpoints
````

<a id="w002638"></a>

## W002638 — File:HSHBIsoOUT.gif

Bron-ID: W002638
Namespace: File (6)
Revisie: 8291 | Datum: 2009-06-26T02:48:43Z
Bron: https://wiki.osdev.org/File:HSHBIsoOUT.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AHSHBIsoOUT.gif&oldid=8291

````text
Taken from figure 5-12 of the USB 2.0 specifications.  Illustrates Data Phase PID Sequence for Isochronous OUT High Bandwidth Endpoints
````

<a id="w002640"></a>

## W002640 — User:Kmtdk

Bron-ID: W002640
Namespace: User (2)
Revisie: 14636 | Datum: 2013-05-07T06:29:02Z
Bron: https://wiki.osdev.org/User:Kmtdk
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKmtdk&oldid=14636

````text


=ps2 mouse=
This page is under construction , and you will might see it changes much. ( this will be removed when im finish; reason: to insure a backup of my work.)

'''There [[Mouse Input|already exists]] a page on this subject. This page will be deleted when all the information is present in the mentioned page. Do not add anything here.'''


== Programming the ps2 mouse ==



=== activation the mouse (aux port) ===
This is done using the keyboard controller *, using the command controller byte.  Inside this byte, there are a few bits telling if the aux is activated or not. <br> The controller command byte looks like this (when we only are talking about the mouse):
{| border="1" cellspacing="0" cellpadding="5" align="center"
!  
!  
! AUX controller
!  
!  
!  
! IRQ 12 
!  
|}
The aux controller controls if the aux is enabled or not, so setting this bit is needed. The IRQ 12 bit is recommended, but you can use the mouse without, but turn this on too.
So to make the aux active you will have to do the following sequence:
*	wait until keyboard controller is ready to get commands (*)
*	write 0x20 to the keyboard controller (port 0x64)
*	wait until data is ready
*	read the data
*	set bit 5 and 1 
*	wait until keyboard controller is ready to get commands
*	write 0x60 to the keyboard controller (port 0x64)
*	wait until it can accept data
*	write the new controller command byte to data port (0x60)
That is it.
Remember to check for ack (0xFA, this is not documented to happen, but might happen on some commands  ...)


(*) Keyboard controller: 
As reference, use my other document “ps2 controller and keyboard” found at: http://www.webmasteren.eu/viden/os/PS2.pdf

===Detection mouse===
Detecting a mouse is not so hard after all, the only thing you need to, is reading the ID byte. 
The ID should be between 0 and 4. However 0 should be the one you will get.  If you get something like 0xAB, then it is mostly a keyboard, and not a mouse ID.

=== Sending commands to the mouse ===
When you want to “talk” to the mouse, you will have to send one command to the keyboard controller, and then the keyboard controller knows that you will talk to the mouse. This command is “0xD4”, and after sending it, you will have to wait until you can send data to the data port (port 0x60), and the data is the command to the mouse. 
The mouse might answer with ACK on most commands, but maybe not. 
There are some few mouse commands, that request data after the command, the data need to be sent if it was a command (first 0xD4 to controller, then the data to data port (0x60)
<br><br>
A good advice, look at the PS2 controller before programming the mouse.

=== Initialization ===

To use the mouse, you will have to do some few things, first activating the AUX port, and the IRQ 12 (that is also described under the activation subject).
After that you can get data, be polling the information from the mouse, however if you want to be able to use IRQ 12, then you will have to enable packet reporting. 

==== Initialization features ====
When a mouse is normally started up (not already init), it is only limited to the standard features, which is: movement and 3 buttons. But nowadays mice (PS2) almost all have a scroll wheel, so in order to make that one work, you will have to send a special sequence of instructions. But that is not the end of the extended features, since some mice have 2 scroll wheels, so there is also a special sequence for that one, so if you want to enable the “extended extended features” then you will have to send the other sequence after the other one.


The first sequence goes like this:
*	set sample to 200
*	set sample to 100
*	set sample to 80
*	get the id
<br>

If the id has changed, then it is scroll mouse, and it will now report 1 more packet than before.
The second sequence goes like this:

*	set sample to 200
*	set sample to 200
*	set sample to 80
*	get the id
<br>


If the id has changed, then it is an extended scroll mouse, but it is the same amount of packets, but the meaning of the last packet has changed.

=== Getting packets ===
If you move the mouse, it will generate packets, if you click the mouse, it generates packets too, but if data reporting is disabled, the packets will not be sent to IRQ 12 ( which also must be activated) 
But there is a command to read the data, without using IRQ 12.(see the commands)



If you want to use the IRQ 12 solution, then this is a part you will need to read. 
First, be sure to map the IRQ 12, and also enable it inside the keyboard controller, but also inside the PIC(programmable interrupt controller) or what else is controlling IRQ’s. second is that you will need to keep track of how many packets the mouse is ought to send, as for a mouse without any extended features add, it is 3 packets, meaning 3 IRQ 12 ! So when a IRQ Fires, you will also need to save the packet, and wait for the other packets to come.  
==== The packets ====
The meaning of the packets is like this:
<br>
<br>
byte 1:
{| border="1" cellspacing="0" cellpadding="5" align="center"
! Y overflow
! X overflow
! Y sign bit 
! X sign bit 
! Always 1
! Middle Btn
! Left Btn 
! Right Btn 
|}
<br>
byte 2:
{| border="1" cellspacing="0" cellpadding="5" align="center"
!  X movement 
|}
<br>
byte 3:
{| border="1" cellspacing="0" cellpadding="5" align="center"
!  Y movement
|}
<br>
If one scroll wheel is enabled:
<br>
byte 4:
{| border="1" cellspacing="0" cellpadding="5" align="center"
!  Z movement
|}
<br>
If both scroll wheels is enabled:
<br>
byte 4:
{| border="1" cellspacing="0" cellpadding="5" align="center"
! Always 0
! Always 0
! 5th btn
! 4th btn
! Z3
! Z2
! Z1
! Z0
|}
<br>
Here is Z3 though Z0 is a signed value, of the Z movement (scroll) if it is increased/ decreased with 2, then it is the vertical scroll wheel, else it is just the horizontal scroll wheel.<br>
If a button is pressed, then the value for the button will be “1”, and when not pressed “0”.

=== Emulation ===
Since some BIOS allow USB to be emulated to PS2, then you might have to think about this too.  However the Emulated USB device, will (Mostly) be another IRQ (the USB IRQ) than IRQ 12. 
=== Mouse commands ===


{| border="1" cellspacing="0" cellpadding="5" align="center"
! Hex value
! Meaning
! Describing
|-
! 0xFF
! reset
! The mouse sends ACK (0xFA) and resets.
|-
! 0xFE
! resend
! this command makes the mouse sends its second last packet to the host again.
|-
! 0xF6
! Set Defaults
! The mouse sends ACK (0xFA) and applies default values
|-
! 0xF5
! Disable Data Reporting
! The mouse sends ACK (0xFA) and disable data reporting.
|-
! 0xF4
! enable Data Reporting
! The mouse sends ACK (0xFA) and enables data reporting
|-
! 0xF3
! Set Sample Rate
! The mouse sends ACK (0xFA) and waits for the next data, written to the mouse, to be the sample rate
|-
! 0xF2 
! Get Device ID
! The mouse sends ACK (0xFA) and sends its ID, though the DATA port
|-
! 0xF0
! Set Remote Mode
! The mouse sends ACK (0xFA) and  then reset its movement counters, and enters remote mode
|-
! 0xEE
! Set Wrap Mode
! The mouse sends ACK (0xFA) and  then reset its movement counters, and enters warp mode
|-
! 0xEC
! Reset Wrap Mode
! The mouse sends ACK, and then enters the last mode, before entering wrap mode, it also resets its movement counters
|-
! 0xEB
! Read Data
! The mouse sends ACK, followed by the first packet, and so on, until It have sent all.
|-
! 0xEA
! Set Stream Mode
! The mouse sends ACK (0xFA) and  then reset its movement counters, and enters reporting  mode
|-
! 0xE9
! Status Request
! The mouse sends ACK, and sends afterwards the status bytes ( 3 bytes in a row, remember to wait each time for new data)
|-
! 0xE8 
! Set Resolution
! The mouse sends ACK and waits for the next data, written to the mouse, to be the resolution
|-
! 0xE7
! Set Scaling 2:1
! The mouse sends ACK and set scaling 2:1
|-
! 0xE6
! Set Scaling 1:1
! The mouse sends ACK and set scaling 1:1

|}
<br>
The status byte looks like this:
<br>
Byte 1:
{| border="1" cellspacing="0" cellpadding="5" align="center"
! Always 0
! mode
! enable
! scaling
! Always 0
! left btn
! middle
! right btn
|}
Byte 2:
{| border="1" cellspacing="0" cellpadding="5" align="center"
!resoltion
|}
Byte 3:
{| border="1" cellspacing="0" cellpadding="5" align="center"
!sample rate
|}<br>
Mode: if it is 1, the current mode is remote mode; if 0 then it is stream mode <br>
Enable: if it is 1, then data reporting is enabled; if 0 then data reporting  is disabled<br>
Scaling: if it is 1, scaling 2:1 is enabled; if 0 then scaling 1:1 is enabled.<br>



=== Modes ===
There are 3 modes:<br>
Warp mode, reporting mode, remote mode.<br>
* Warp mode is a funny one, since it just sends the same back, every time something happens. It sends 0xEE back.<br>
* Reporting mode is the normal used one, it is here the mouse sends data each time you move / click the mouse. <br>
* Remote mode is a mode, where you keep polling the data from the mouse ( about movement and click)<br>

===Resolution, Scaling and Sampling rate ===
Those is some options about the mouse, when you move it, and about how many packets it can make pr second. 
Resolution: this is how many packets pr millimeter movement. 
Scaling:  is a change to the packet reported for each millimeter.
Sampling rate:  this is how many packets the mouse can send pr second.
Those “options” can (of cause) have different value, so let’s sum them up:
Resolution: 

{| border="1" cellspacing="0" cellpadding="5" align="center"
! value
! resolution 
|-
!0x00
!1 count /mm
|-
!0x01
!2 count /mm
|-
!0x02
!4 count /mm
|-
!0x03
!8 count /mm
|}
<br>
For scaling, there is not many possibilities, since it can either be “1:1” or “2:1”, but looking at the scheme (below) we can see what happens if it is “2:1”

{| border="1" cellspacing="0" cellpadding="5" align="center"
! Movement Counter
! Reported Movement
|-
!0
!0
|-
!1
!1
|-
!2
!1
|-
!3
!3
|-
!4
!6
|-
!5
!9
|-
!more than 5
!2 * Movement Counter
|}
<br>
Sampling rate can have the following values:  (all values is Decimal, NOT hex)<br>
{| border="1" cellspacing="0" cellpadding="5" align="center"
! value
! Samples pr second  
|-
!10
!10
|-
!20
!20
|-
!40
!40
|-
!60
!60
|-
!80
!80
|-
!100
!100
|-
!200
!200
|}





=== Resetting and hot plugging ===
When a mouse is powered up, (can either be done from software reset, or by hardware, hot plugging too) the mouse will enters a default state (the same as the command “default”), which is those following values:
*	sampling rate: 100 samples pr second 
*	scaling: 1:1
*	resolution: 4 counts pr mm
*	mode: data reporting
*	data reporting: disabled

But before that, the mouse sends its self test, which either can be 0xAA (successful) or 0xFC (error), after receiving 0xaa, the host will get the ID. This also applies to hot plugging, so in the IRQ 12 handler, be sure to test for 0xaa (if this comes as the first packet), and re-init it after getting the ID.<br>

This except the “self test” and ID is applied when normally starting the computer.


A note on hot plugging: since it all depends on the BIOS, there may be computers, where hot plugging is not supported at all. There is nothing to do about it.
````

<a id="w002641"></a>

## W002641 — File:FunctionConfigsIFacesEPs.gif

Bron-ID: W002641
Namespace: File (6)
Revisie: 8312 | Datum: 2009-06-30T20:31:33Z
Bron: https://wiki.osdev.org/File:FunctionConfigsIFacesEPs.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AFunctionConfigsIFacesEPs.gif&oldid=8312

````text
Illustrates the relationship between functions, configurations, interfaces, and endpoints.
````

<a id="w002643"></a>

## W002643 — PIC

Bron-ID: W002643
Namespace: Main (0)
Revisie: 8332 | Datum: 2009-07-03T00:55:41Z
Bron: https://wiki.osdev.org/PIC
Revisie-URL: https://wiki.osdev.org/index.php?title=PIC&oldid=8332
Doorverwijzing: 8259 PIC

````text
#REDIRECT [[8259 PIC]]
````

<a id="w002644"></a>

## W002644 — Talk:PS2 mouse

Bron-ID: W002644
Namespace: Talk (1)
Revisie: 8342 | Datum: 2009-07-03T18:09:47Z
Bron: https://wiki.osdev.org/Talk:PS2_mouse
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APS2_mouse&oldid=8342

````text
== Done? ==

I've noticed that this page has been around for a while and that its purpose was to hold some information that was yet to be moved to the the 'Mouse Input' wiki page. It seems to me that all the information is already present there, so I was wondering if there is any reason this page should still be here?

[[User:Creature|Creature]]  13:41, 3 July 2009 (GMT + 1)

: '''Delete.''' I'd say it's done for the above reasons. --[[User:Troy martin|Troy Martin]] 18:09, 3 July 2009 (UTC)
````

<a id="w002645"></a>

## W002645 — USB Mass Storage Class Devices

Bron-ID: W002645
Namespace: Main (0)
Revisie: 27911 | Datum: 2023-07-09T05:04:36Z
Bron: https://wiki.osdev.org/USB_Mass_Storage_Class_Devices
Revisie-URL: https://wiki.osdev.org/index.php?title=USB_Mass_Storage_Class_Devices&oldid=27911

````text
{{Stub}}

The contents here were originally located on the [[USB]] entry but have been moved here to provide minimal information until replaced with more details.

= Subclasses =

= Types =
== CBI ==
''"'''Control/Bulk/Interrupt (CBI)''' transport is only for full-speed floppy disk drives. It will not be used in conjunction with LSDFS, and it's use in '''ANY''' new design id discouraged."'' - USB MSC specification.
== BBB ==
'''Bulk Only (BBB)'''

Bulk Only Mass Storage Devices are the most common type of USB storage device, and they include most USB thumb drives, hard drives, CD, DVD and Blu-ray drives as well.  They normally use the SCSI Command Set, but the USB Mass Storage specifications allow devices to use other command sets as well.

The USB Mass Storage Class adds two additional USB commands to the default control endpoint to allow the system to reset the mass storage device, and to get the number of "logical" devices supported by the physical mass storage device.

The SCSI Command block is encapsulated in a "Command Block Wrapper" (CBW), which contains information about the length of the command, the direction, the logical unit to send the command to, and a unique identifier that will be returned in the response from the device that can be used to match up the request with the response by the host.  The Command Block Wrapper is sent to the device using the first Out direction, Block only endpoint on the interface, before any data is sent to or received from the device.  Then, a final "Command Status Wrapper" (CSW) is sent to the host by the device that contains the results of the operation.

=== Command Block Wrapper ===
{| {{wikitable}}
! Offset (Hex)
! Type
! Description
|-
| 0x00 || int32 || Signature (0x43425355)
|-
| 0x04 || int32 || Tag (Transaction Unique Identifier)
|-
| 0x08 || int32 || Length
|-
| 0x0c || byte || Direction (0x00 = ToDevice, 0x80 = ToHost)
|-
| 0x0d || byte || Logical Unit Number
|-
| 0x0e || byte || Command Length
|-
| 0x0f || byte[16] || Command Data
|}

=== Command Status Wrapper ===
{| {{wikitable}}
! Offset (Hex)
! Type
! Description
|-
| 0x00 || int32 || Signature (0x53425355)
|-
| 0x04 || int32 || Tag (Copied From CBW)
|-
| 0x08 || int32 || Residue (Difference Between CBW Length And Actual Length)
|-
| 0x0c || byte || Status (0x00 = Success, 0x01 = Failed, 0x02 = Phase Error)
|}

== UFI ==

== LSDFS ==
'''Locakable Storage Devices Feature Specification (LSD FS)'''


= Booting =

== Booting off of a USB flash disk ==

If the [[BIOS]] supports it is it as simple as selecting USB-FDD in the boot order and putting a simple bootloader (grub might do, I rolled my own) into the first 512 bytes of the device. The BIOS will handle all the fancy PCI / USB stuff via int 0x13. Note that there is no need for any specific filesystem or partitions (which means you can just "dd" a 2.2 linux kernel (which used to contain a simple floppy loader) to /dev/sda and it will easily come up).

= See Also =
== Forum Topics ==
* [[Topic:9334|USB stick]] - discusses the feasability of booting from an USB memory stick
== Specifications ==
* [http://www.usb.org/developers/devclass_docs Class specifications] - all the class spsecifications. ever.
* [https://www.usb.org/sites/default/files/Mass_Storage_Specification_Overview_v1.4_2-19-2010.pdf General mass storage specification] - the mass storage overview.
* [https://usb.org/sites/default/files/usbmassbulk_10.pdf Bulk-Only mass storage specification] - details for bulk-only (BBB) devices.
* [http://www.usb.org/developers/devclass_docs/usb_msc_cbi_1.1.pdf CBI specification]
* [http://www.usb.org/developers/devclass_docs/usbmass-ufi10.pdf UFI specification]
* [http://www.usb.org/developers/devclass_docs/usb_msc_boot_1.0.pdf Bootability specification]

[[Category:USB]]
[[Category:Storage]]
````

<a id="w002646"></a>

## W002646 — USB Human Input Devices

Bron-ID: W002646
Namespace: Main (0)
Revisie: 20821 | Datum: 2017-05-22T15:11:19Z
Bron: https://wiki.osdev.org/USB_Human_Input_Devices
Revisie-URL: https://wiki.osdev.org/index.php?title=USB_Human_Input_Devices&oldid=20821
Doorverwijzing: USB Human Interface Devices

````text
#REDIRECT [[USB Human Interface Devices]]
````

<a id="w002649"></a>

## W002649 — Talk:C PlusPlus bare bones

Bron-ID: W002649
Namespace: Talk (1)
Revisie: 9617 | Datum: 2010-01-11T14:55:42Z
Bron: https://wiki.osdev.org/Talk:C_PlusPlus_bare_bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AC_PlusPlus_bare_bones&oldid=9617
Doorverwijzing: Talk:C++ Bare Bones

````text
#REDIRECT [[Talk:C++ Bare Bones]]
````

<a id="w002650"></a>

## W002650 — User:Troy martin/Monaco

Bron-ID: W002650
Namespace: User (2)
Revisie: 8371 | Datum: 2009-07-05T22:25:40Z
Bron: https://wiki.osdev.org/User:Troy_martin/Monaco
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATroy_martin%2FMonaco&oldid=8371

````text
In case someone installs Monaco as the default skin, I'm writing a set of OSDev Wiki specific customizations to the default Monaco skin setup. If you know about the monaco files, check out the files, such as [[User:Troy martin/Monaco/Monaco-sidebar]].
````

<a id="w002651"></a>

## W002651 — User:Troy martin/Monaco/Monaco-sidebar

Bron-ID: W002651
Namespace: User (2)
Revisie: 8379 | Datum: 2009-07-05T22:57:21Z
Bron: https://wiki.osdev.org/User:Troy_martin/Monaco/Monaco-sidebar
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATroy_martin%2FMonaco%2FMonaco-sidebar&oldid=8379

````text
* Main Page | OSDev Wiki
** Expanded Main Page
** Special:RecentChanges | Recent changes
** OSDev Wiki:Community Portal | Community Portal
** Projects | OS Projects
** Category:FAQ | FAQ

* http://forum.osdev.org/ | Forum

* Tutorials
** Bare Bones
** C++ Bare Bones
** Higher Half Bare Bones
*** Higher Half With GDT
** Category:Tutorials | more...

* Languages
** Assembly
** C
** C++
** D
** Pascal
** Category:Languages | more...

* Category:Hardware | Hardware
** Category:Buses | Buses
** Category:Common Devices | Common Devices
** Category:Human_Interface_Device | Input Devices
** Category:Network Hardware | Network Hardware
** Category:Platforms | Platforms
*** Category:X86 | X86
**** Category:X86 CPU | X86 CPU
** Category:Sound | Sound
** Category:Storage | Storage
** Category:Video | Video

* Category:OS theory | OS theory
** Kernels
** Memory Management
** Processes and Threads
** Category:IPC | IPC
** Device Driver Interfaces
** File Systems
** Category:Networking | Networking
** Text UI
** Category:Graphical UI | Graphical UI
** Security
** Licensing

* Tools
** Tool Comparison
** Category:Assemblers | Assemblers
** Category:Compilers | Compilers
** Category:Linkers | Linkers
** Category:IDEs | IDEs
** Category:Bootloaders | Bootloaders
** Category:Emulators | Emulators
** Category:Disk Image Utilities | Disk Image Utilities
````

<a id="w002653"></a>

## W002653 — C MinusMinus

Bron-ID: W002653
Namespace: Main (0)
Revisie: 8377 | Datum: 2009-07-05T22:48:09Z
Bron: https://wiki.osdev.org/C_MinusMinus
Revisie-URL: https://wiki.osdev.org/index.php?title=C_MinusMinus&oldid=8377
Doorverwijzing: C--

````text
#REDIRECT [[C--]]
````

<a id="w002655"></a>

## W002655 — User:Oros/FAT

Bron-ID: W002655
Namespace: User (2)
Revisie: 8395 | Datum: 2009-07-07T05:26:45Z
Bron: https://wiki.osdev.org/User:Oros/FAT
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AOros%2FFAT&oldid=8395

````text
{{Filesystems}}
The '''File Allocation Table''' ('''FAT''') file system was introduced with DOS v1.0 (and possibly CP/M). Supposedly written by Bill Gates, FAT is a very simple file system which is, at its most basic level, nothing more than a singular linked list of clusters. FAT file systems use very little memory and is one of, if not the, most basic file system in use today.

== Overview ==
There are several different versions of the FAT file system. Each version was designed for a different size of storage media. 

{| {{wikitable}}
|-
! Name
! Introduced
! Max File Size
! Max Cluster Count
! Max Volume Size
! Common Medium
|-
| FAT12
| <1980
| 4GB - 1 Byte
| 4,077
| 32MB
| Floppies
|-
| FAT16
| Nov. 1987
| 4GB - 1 Byte
| 65,517
| 2GB, 4GB
| Portable storage (SD, MMC, ect...)
|-
| FAT32
| Aug. 1996
| 4GB - 1 Byte
| 268,435,437
| 2TB, 8TB
| Hard Drives
|}

== Implementation Details ==
The FAT file system views the storage media as a flat array of clusters. If the physical media does not address its data as a flat list of sectors (really old hard disks and floppy disks) then the cluster numbers will need to be translated before being sent to the disk. The storage media is organized into three basic areas.

* The boot record
* The File Allocation Table (FAT)
* The directory and data area

=== Boot Record ===
The boot record occupies one sector, and is always placed in logical sector number zero of the storage media. (which is physically cylinder 0, head 0, sector 1) This is the easiest sector on the disk for the computer to locate when it begins running. If the storage media is a hard disk partition, the boot sector will begin at the start of the partition, not the start of the disk.

==== Standard Boot Record ====
{| {{wikitable}}
|- 
!|Byte Offset	
!Length (bytes)
! Description
|- 
|align="right"|0	  
|align="right"|3
| The first three bytes ''6B 3C 90'' disassemble to ''JMP SHORT 3C NOP''. The reason for this is to jump over the disk format information. Since the first sector of the disk is loaded into ram at location 0x0000:0x7C00 and executed, without this jump, the processor would attempt to execute data that isn't code.
|- 
|align="right"|3
|align="right"|8
| OEM Name (padded with spaces). This value determines in which system disk was formatted. MS-DOS checks this field to determine which other parts of the boot record can be relied on. Common values are ''IBM  3.3'' (with two spaces between the "IBM" and the "3.3"), ''MSDOS5.0'', ''MSWIN4.1'' and ''mkdosfs''.
|- 
|align="right"|11
|align="right"|2
| The number of Bytes per sector (remember, all numbers are in the little-endian format).
|- 
|align="right"|13
|align="right"|1
| Number of sectors per cluster.
|- 
|align="right"|14
|align="right"|2
| Number of reserved sectors.
|- 
|align="right"|16
|align="right"|1
| Number of File Allocation Tables (FAT's) on the storage media. Often this value is 2.
|- 
|align="right"|17
|align="right"|2
| Number of directory entries (must be set so that the root directory occupies entire sectors).
|- 
|align="right"|19
|align="right"|2
| The total sectors in the logical volume. If this value is 0, it means there are more than 65535 sectors in the volume, and the actual count is stored in "Large Sectors (bytes 32-35).
|- 
|align="right"|21
|align="right"|1
| This Byte indicates the [http://support.microsoft.com/kb/q140418/ media descriptor type].
|- 
|align="right"|22 
|align="right"|2
| Number of sectors per FAT.
|- 
|align="right"|24
|align="right"|2
| Number of sectors per track.
|- 
|align="right"|26
|align="right"|2
| Number of heads or sides on the storage media.
|- 
|align="right"|28 
|align="right"|4
| Number of hidden sectors.
|- 
|align="right"|32
|align="right"|4
| Large amount of sector on media. This field is set if there are more than 65,535 sectors in the volume.
|}

==== Extended Boot Record ====
The extended boot record comes right after the standard boot record. It contains different information depending on whether this is a FAT 12, FAT 16, or FAT 32 storage media.
===== FAT 12 and FAT 16 =====
{| {{wikitable}}
|-
! Byte Offset
! Length (bytes)
! Description
|-
|align="right"|36      
|align="right"|1
| Physical drive number. The values here are identical to the values returned by the BIOS interrupt 0x13. ''0x00'' for a floppy disk and ''0x80'' for hard disks.
|- 
|align="right"|37
|align="right"|1
| Reserved field. In Windows NT bit 0 is a dirty flag to request chkdsk at boot time. bit 1 requests surface scan too.
|- 
|align="right"|38
|align="right"|1
| Signature (must be ''0x28'' or ''0x29'').
|- 
|align="right"|39
|align="right"|4
| ID (serial number) Used for tracking disks  between computers. (For most purposes just ignore this field).
|- 
|align="right"|43
|align="right"|11
| Volume label, padded with blanks (''0x20'').
|-
|align="right"|54
|align="right"|8
| System identifier string. This field is a string representation of the FAT file system type. It is padded with spaces.
|}

===== FAT 32 =====
{| {{wikitable}}
|-
! Byte Offset
! Length (bytes)
! Meaning
|-
|align="right"|36
|align="right"|4
| The size of the File Allocation Table in bytes.
|- 
|align="right"|40      
|align="right"|2
| Flags.
|- 
|align="right"|42
|align="right"|1
| Signature (must be 0x28 or 0x29).
|- 
|align="right"|39 
|align="right"|2
| FAT version number. The high byte is the major version and the low byte is the minor version. FAT drivers should respect this field.
|- 
|align="right"|44
|align="right"|4
| The cluster number of the root directory. Often this filed is set to 2.
|-
|align="right"|48
|align="right"|2
| The cluster number of the FSInfo structure.
|-
|align="right"|50
|align="right"|2
| The cluster number of the backup boot sector.
|-
|align="right"|52
|align="right"|12
| Reserved. When the volume is formated these bytes should be zero.
|-
|align="right"|64      
|align="right"|1
| Drive number. The values here are identical to the values returned by the BIOS interrupt 0x13. 0x00 for a floppy disk and 0x80 for hard disks.
|- 
|align="right"|65      
|align="right"|1
| Flags in windows NT. Reserved otherwise.
|- 
|align="right"|66      
|align="right"|1
| Signature (must be 0x28 or 0x29).
|- 
|align="right"|67 
|align="right"|4
| VolumeID 'Serial' number. Used for tracking volumes between computers. You can ignore this if you want.
|- 
|align="right"|71
|align="right"|11
| Volume label string. This field is padded with spaces.
|-
|align="right"|82
|align="right"|8
| System identifier string. Always "FAT32   ".
|}

=== File Allocation Table ===
The File Allocation Table (FAT) is a table stored on the storage media that indicates the status and location of all data clusters that are on the disk. It can be considered the "table of contents" of a disk. The cluster may be available for use, it may be reserved by the operating system, it may be unavailable due to a bad sector on the disk, or it may be in use by a file. The clusters of a file need not be right next to each other on the disk. In fact it is likely that they are scattered widely throughout the disk. The FAT allows the operating system to follow the "chain" of clusters in a file. 

==== FAT 12 ====
FAT 12 uses 12 bits to address the clusters on the disk. Each 12 bit entry in the FAT points to the next cluster of a file on the disk. Given a valid cluster number, here is how you extract the value of the next cluster in the cluster chain:
<pre>
unsigned char FAT_table[cluster_size];
unsigned int fat_offset = active_cluster + (active_cluster / 2);// multiply by 1.5
unsigned int fat_sector = first_fat_sector + (fat_offset / cluster_size);
unsigned int ent_offset = fat_offset % cluster_size;

//at this point you need to read from sector "fat_sector" on the disk into "FAT_table".

unsigned short table_value = *(unsigned short*)&FAT_table[ent_offset];

if(current_cluster & 0x0001)
   table_value = table_value >> 4;
else
   table_value = table_value & 0x0FFF;

//the variable "table_value" now has the information you need about the next cluster in the chain.
</pre>
If "table_value" is greater than or equal to (>=) 0xFF8 then there are no more clusters in the chain. This means that the whole file has been read. If "table_value" equals (==) 0xFF7 then this cluster has been marked as "bad". "Bad" clusters are prone to errors and should be avoided. If "table_value" is not one of the above cases then it is the cluster number of the next cluster in the file.

==== FAT 16 ====
FAT 16 uses 16 bits to address the clusters on the disk. Because of this, it is much easier to extract the values out of a 16 bit File Allocation Table. Here is how it is done:
<pre>
unsigned char FAT_table[cluster_size];
unsigned int fat_offset = active_cluster * 2;
unsigned int fat_sector = first_fat_sector + (fat_offset / cluster_size);
unsigned int ent_offset = fat_offset % cluster_size;

//at this point you need to read from sector "fat_sector" on the disk into "FAT_table".

unsigned short table_value = *(unsigned short*)&FAT_table[ent_offset];

//the variable "table_value" now has the information you need about the next cluster in the chain.
</pre>
If "table_value" is greater than or equal to (>=) 0xFFF8 then there are no more clusters in the chain. This means that the whole file has been read. If "table_value" equals (==) 0xFFF7 then this cluster has been marked as "bad". "Bad" clusters are prone to errors and should be avoided. If "table_value" is not one of the above cases then it is the cluster number of the next cluster in the file.

==== FAT 32 ====
FAT 32 uses 28 bits to address the clusters on the disk. Yes, that is right. FAT 32 only uses 28 of it's 32 bits. The highest 4 bits are reserved. This means that they should be ignored when read and unchanged when written. Besides this small detail, extracting a value from a 32 bit FAT is almost identical to the same operation on a 16 bit FAT:
<pre>
unsigned char FAT_table[cluster_size];
unsigned int fat_offset = active_cluster * 4;
unsigned int fat_sector = first_fat_sector + (fat_offset / cluster_size);
unsigned int ent_offset = fat_offset % cluster_size;

//at this point you need to read from sector "fat_sector" on the disk into "FAT_table".

//remember to ignore the high 4 bits.
unsigned int table_value = *(unsigned int*)&FAT_table[ent_offset] & 0x0FFFFFFF;

//the variable "table_value" now has the information you need about the next cluster in the chain.
</pre>
If "table_value" is greater than or equal to (>=) 0x0FFFFFF8 then there are no more clusters in the chain. This means that the whole file has been read. If "table_value" equals (==) 0x0FFFFFF7 then this cluster has been marked as "bad". "Bad" clusters are prone to errors and should be avoided. If "table_value" is not one of the above cases then it is the cluster number of the next cluster in the file.

=== Directories ===
A directory entry simply stores the information needed to know where a file's data or a folder's children are stored on the disk. It also holds information such as the entry's name, size, and creation time. There are two types of directories in a FAT file system. Standard 8.3 directory entries, which appear on all FAT file systems, and Long File Name directory entries which are optionally present to allow for longer file names.

==== Standard 8.3 format ====
{| {{wikitable}}
!Byte Offset
!Length
!Description
|-
|align="right"|0x00
|align="right"|8
|DOS file name (padded with spaces)

The first byte can have the following special values:
{| {{wikitable}}
|0x00
|Entry is available and no subsequent entry is in use
|-
|0x05
|Initial character is actually 0xE5. 0x05 is a valid kanji lead byte, and is used for support for filenames written in kanji.
|-
|0x2E
|'Dot' entry; either '<tt>.</tt>' or '<tt>..</tt>'
|-
|0xE5
|Entry has been previously erased and is available. File undelete utilities must replace this character with a regular character as part of the undeletion process.
|}
|-
|align="right"|0x08
|align="right"|3
|DOS file extension (padded with spaces)
|-
|align="right"|0x0b
|align="right"|1
|File Attributes
{| {{wikitable}}
!width="5%"|Bit
!width="5%"|Mask
!width="40%"|Description
|-
|0
|0x01
|Read Only
|-
|1
|0x02
|Hidden
|-
|2
|0x04
|System
|-
|3
|0x08
|Volume Label
|-
|4
|0x10
|Subdirectory
|-
|5
|0x20
|Archive bit
|-
|6
|0x40
|Device (internal use only, never found on disk)
|-
|7
|0x80
|Unused
|}
An attribute value of 0x0F is used to designate a long file name entry.
|-
|align="right"|0x0c
|align="right"|1
|Reserved; two bits are used by NT and later versions to encode case information (see below); otherwise 0.
|-
|align="right"|0x0d
|align="right"|1
|Create time, fine resolution: 10ms units, values from 0 to 199.
|-
|align="right"|0x0e
|align="right"|2
|Create time. The hour, minute and second are encoded according to the following bitmap:
{| {{wikitable}}
!width="5%"|Bits
!width="40%"|Description
|-
|15-11
|Hours (0-23)
|-
|10-5
|Minutes (0-59)
|-
|4-0
|Seconds/2 (0-29)
|}
Note that the ''seconds'' is recorded only to a 2 second resolution. Finer resolution for file creation is found at offset 0x0d.
|-
|align="right"|0x10
|align="right"|2
|Create date. The year, month and day are encoded according to the following bitmap:
{| {{wikitable}}
!width="5%"|Bits
!width="40%"|Description
|-
|15-9
|Year (0 = 1980, 127 = 2107)
|-
|8-5
|Month (1 = January, 12 = December)
|-
|4-0
|Day (1 - 31)
|}
|-
|align="right"|0x12
|align="right"|2
|Last access date; see offset 0x10 for description.
|-
|align="right"|0x14
|align="right"|2
|EA-Index (used by OS/2 and NT) in FAT12 and FAT16, High 2 bytes of first cluster number in FAT32
|-
|align="right"|0x16
|align="right"|2
|Last modified time; see offset 0x0e for description.
|-
|align="right"|0x18
|align="right"|2
|Last modified date; see offset 0x10 for description.
|-
|align="right"|0x1a
|align="right"|2
|First cluster in FAT12 and FAT16. Low 2 bytes of first cluster in FAT32. Entries with the Volume Label flag, subdirectory ".." pointing to root, and empty files with size 0 should have first cluster 0.
|-
|align="right"|0x1c
|align="right"|4
|File size in bytes. Entries with the Volume Label or Subdirectory flag set should have a size of 0.
|}

==== Long File Names ====
Long file name entries ''always'' have a regular 8.3 entry to which they belong. The long file name entries are always placed immediately before their 8.3 entry. Here is the format of a long file name entry.
{| {{Wikitable}}
|-
! Offset (in bytes)
! Length (in bytes)
! Meaning
|-
| 0
| 1
| The order of this entry in the sequence of long file name entries. This value helps you to know where in the file's name the characters from this entry should be placed.
|-
| 1
| 10
| The first 5, 2-byte characters of this entry.
|-
| 11
| 1
| Attribute. Always equals 0x0F. (the long file name attribute)
|-
| 12
| 1
| Long entry type. Zero for name entries.
|-
| 13
| 1
| Checksum.
|-
| 14
| 12
| The next 6, 2-byte characters of this entry.
|-
| 26
| 2
| Always zero.
|-
| 28
| 4
| The final 2, 2-byte characters of this entry.
|}
Here is an example of what a regular 8.3 entry with one long file name entry preceding it might look like in a hex editor:
<pre>
41 62 00 69 00 6E 00 00 00 FF FF 0F 00 7F FF FF FF FF FF FF FF FF FF FF FF FF 00 00 FF FF FF FF
42 49 4E 20 20 20 20 20 20 20 20 10 00 00 F7 01 D5 38 D5 38 00 00 F7 01 D5 38 03 00 00 00 00 00
</pre>
And in a text editor:
<pre>
Ab.i.n..........................
BIN        ......8.8.....8......
</pre>
The first line is the long file name entry (the second line is the regular 8.3 entry). The very first byte (41) tells us two important pieces of information. First, the one (01) tells us that this is the first long file name entry for the regular 8.3 entry. Second the forty (40) part tells us that this is also the ''last'' long file name entry for this regular 8.3 entry. The next 10 bytes spell out the first part of the long file name. In this case they read:
<pre>
b 00 i 00 n 00 00 00 FF FF
</pre>
Notice that each character is two bytes long and that the name is null terminated. The two FF's at the end are the padding at the end of the long file name. This is also what the other FF's in the long file name entry are.
The final important thing to notice about the long file name entry is it's attribute byte at offset 11. the 0x0F attribute allows us to verify that this is indeed a long file name entry.

== Programming Guide ==
This section is intended to give you information about common functions that are preformed on a FAT file system.

=== Reading the Boot Sector ===
The Boot Sector is always placed at logical sector number zero. You can either read the boot sector into an array and access it's members that way or you can read it into a structure and access it through the structure. Either way, the values in the boot sector need to be readily available in order to do much of anything with a FAT file system.

Here is an example of some boot sector structures in C. 
<pre>
typedef struct fat_extBS_32
{
	//extended fat32 stuff
	unsigned int		table_size_32;
	unsigned short		extended_flags;
	unsigned short		fat_version;
	unsigned int		root_cluster;
	unsigned short		fat_info;
	unsigned short		backup_BS_sector;
	unsigned char 		reserved_0[12];
	unsigned char		drive_number;
	unsigned char 		reserved_1;
	unsigned char		boot_signature;
	unsigned int 		volume_id;
	unsigned char		volume_label[11];
	unsigned char		fat_type_label[8];

}__attribute__((packed)) fat_extBS_32_t;

typedef struct fat_extBS_16
{
	//extended fat12 and fat16 stuff
	unsigned char		bios_drive_num;
	unsigned char		reserved1;
	unsigned char		boot_signature;
	unsigned int		volume_id;
	unsigned char		volume_label[11];
	unsigned char		fat_type_label[8];
	
}__attribute__((packed)) fat_extBS_16_t;

typedef struct fat_BS
{
	unsigned char 		bootjmp[3];
	unsigned char 		oem_name[8];
	unsigned short 	        bytes_per_sector;
	unsigned char		sectors_per_cluster;
	unsigned short		reserved_sector_count;
	unsigned char		table_count;
	unsigned short		root_entry_count;
	unsigned short		total_sectors_16;
	unsigned char		media_type;
	unsigned short		table_size_16;
	unsigned short		sectors_per_track;
	unsigned short		head_side_count;
	unsigned int 		hidden_sector_count;
	unsigned int 		total_sectors_32;
	
	//this will be cast to it's specific type once the driver actually knows what type of FAT this is.
	unsigned char		extended_section[54];
	
}__attribute__((packed)) fat_BS_t;
</pre>
Important pieces of information that can be extracted from the boot sector include:


'''The size of the root directory (unless you have FAT32, in which case the size will be 0):'''
<pre>
root_dir_sectors = ((fat_boot->root_entry_count * 32) + (fat_boot->bytes_per_sector - 1)) / fat_boot->bytes_per_sector);
</pre>
This calculation will round up. 32 is the size of a FAT directory in bytes.


'''The first data sector (that is, the first sector in which directories and files may be stored):'''
<pre>
first_data_sector = fat_boot->reserved_sector_count + (fat_boot->table_count * fat_boot->table_size_16);
</pre>


'''The first sector in the File Allocation Table:'''
<pre>
first_fat_sector = fat_boot->reserved_sector_count;
</pre>


'''The total number of data sectors:'''
<pre>
data_sectors = fat_boot->total_sectors_16 - (fat_boot->reserved_sector_count + (fat_boot->table_count * fat_boot->table_size_16) + root_dir_sectors);
</pre>


'''The total number of clusters:'''
<pre>
total_clusters = data_sectors / fat_boot->sectors_per_cluster;
</pre>
This rounds down.


'''The FAT type of this file system (12, 16, or 32):'''
<pre>
if(total_clusters < 4085) 
{
   fat_type = 12;
} 
else 
{
   if(total_clusters < 65525) 
   {
      fat_type = 16;
   } 
   else 
   {
      fat_type = 32;
   }
}
</pre>

=== Reading Directories ===
The first step in reading directories is finding and reading the root directory. On a FAT 12 or FAT 16 volumes the root directory is at a fixed position immediately after the File Allocation Tables in the first data sector: 
<pre>
root_cluster_12_or_16 = first_data_sector;
</pre>

For FAT 32 the cluster of the root directory is given in the extended boot record:
<pre>
root_cluster_32 = extBS_32->root_cluster;
</pre>

All cluster numbers for directories that are given in the file system will be relative to the first data cluster. For instance, "extBS_32->root_cluster" often reports that the root cluster is at cluster 2. Here is how you can find the absolute cluster when given the relative cluster.
<pre>
absolute_cluster = relative_cluster - 2 + first_data_sector;
</pre>
'''Note:'''
Although the absolute cluster should be used to read data from the disk, the relative cluster number should be used when reading values from the File Allocation Table.

After the correct cluster has been loaded into memory, the next step is to read and parse all of the entries in it. Each entry is 32 bytes long. For each 32 byte entry this is the flow of execution:
# Does the entry even exist? If the first byte of the entry is equal to (==) zero or 0xE5 then the entry does _not_ exist. Otherwise it does exist. Yes, goto number 2. No, goto number 7
# Is this entry a long file name entry? If the 11'th byte of the entry equals (==) 0x0F (or in other words, it's attribute says that it is a long file name entry), then it is a long file name entry. Otherwise, it is not. Yes, goto number 3. No, goto number 4
# Read the portion of the long filename into a temporary buffer. goto 7
# Parse the data for this entry using the table from further up on this page. It would be a good idea to save the data for later. Possibly in a virtual file system structure. goto number 5
# Is there a long file name in the temporary buffer? Yes, goto number 6. No, goto 7
# Apply the long file name to the entry that you just read and clear the temporary buffer. goto number 7
# Increment pointers and/or counters and check the next entry. (goto number 1) 

This process should be repeated until all of the entries have been read from the cluster. You should then check to see if there is another cluster following this one in the cluster chain or if this is the last cluster in the chain. See the section called following cluster chains for more information. You should do the above process for each cluster in the chain, following it until there are no more clusters left in the chain. Then you can check if any of the entries that you just read are directories. If the are they should each be read in the same way starting with their first cluster number which is stored in the entry.

=== Following Cluster Chains ===
There are two basic steps to following cluster chains. The first step is to find out if there is another "link" (cluster) following the current one in the chain. The second step is to actually use the value read from the FAT to read the next sector. Here is the basic idea:
# Extract the value from the FAT for the _current_ cluster. (Use the previous section on the File Allocation Table for details on how exactly to extract the value.) goto number 2
# Is this cluster marked as the last cluster in the chain? (again, see the above section for more details) Yes, goto number 4. No, goto number 3
# Read the cluster represented by the extracted value and return for more directory parsing.
# The end of the cluster chain has been found. Our work here is finished. :)

==See Also==

=== Threads ===
* from raw bits to directory listing (code posted) in the [[Topic:11247|forum]]
* Public Domain FAT32 code in the [[Topic:13993|forum]]

=== External Links ===
* [http://www.osdever.net/downloads/docs/fatgen103.zip FAT32 File System Specification] - from Cottontail OS Development Library
* http://scottie.20m.com/fat.htm
* http://www.maverick-os.dk/FileSystemFormats/FAT12_FileSystem.html
* http://www.pjrc.com/tech/8051/ide/fat32.html
* http://www.viralpatel.net/taj/tutorial/fat.php
[[Category:Filesystems]]
````

<a id="w002656"></a>

## W002656 — COM

Bron-ID: W002656
Namespace: Main (0)
Revisie: 30376 | Datum: 2026-03-22T23:38:46Z
Bron: https://wiki.osdev.org/COM
Revisie-URL: https://wiki.osdev.org/index.php?title=COM&oldid=30376

````text
{{Stub}}
{{File formats}}
{{Distinguish|text=Microsoft's [[Component Object Model]]}}

==Introduction==
COM files are extremely simple executable files. They are useful for loading programs without complications like relocating symbols, reading file headers, etc.  However, their simplicity can make their usefulness limited, so you may want to support something more complex but useful, like [[ELF]], once your OS is advanced.

COM files were used by MS-DOS.  They are raw binaries, meaning there is no header data.  Also, COM files are restricted to the size of one segment (a real-address mode segment, 64kb), minus 256 bytes.  When a COM file is loaded, it is assumed that all code and data fits in one segment, and it is loaded to offset <tt>0x100</tt>.

MS-DOS creates and stores an info structure starting at offset 0 and ending just before 0x100, called the ''Program Segment Prefix'' (PSP). The PSP is made just before starting the COM program.  More info about the PSP can be found [https://web.archive.org/web/20061101031637/http://heim.ifi.uio.no/~stanisls/helppc/program_segment_prefix.html here]. The PSP is only required for running DOS programs.

COM files usually terminate with a <code>RET</code> instruction. On DOS, the <code>RET</code> pops a empty word from the stack, which results in a interrupt <tt>0x20</tt> (DOS API Exit) being called.

==Loading==
To load a COM file you need to load it to a low memory location (conventional memory), point <code>DS</code> and <code>ES</code> to the start of the file with an <code>org 0x100</code>, and then jump to the start, with offset <tt>0x100</tt>.

==See Also==
=== Articles ===
* [[MZ]]
* [[NE]]
* [[PE]]

[[Category:Executable Formats]]
````

<a id="w002657"></a>

## W002657 — User:Scorchos

Bron-ID: W002657
Namespace: User (2)
Revisie: 8444 | Datum: 2009-07-09T09:47:53Z
Bron: https://wiki.osdev.org/User:Scorchos
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AScorchos&oldid=8444

````text
Bob Moss is a teenager who splits his time between coding [http://www.scorchos.com/ ScorchOS] and writing for a [http://www.linuxformat.com popular technology magazine]. He lives in Wiltshire, UK and will shortly be starting a degree in Computer Science at [http://www.plymouth.ac.uk/ The University of Plymouth]. You can find out more at his [http://bjm1904.com personal site] or [http://www.twitter.com/bob_moss twitter account].

==User Subpages==

Presently none! :-)

==Contributions Here==

He has made modifications to the [http://wiki.osdev.org/Projects OS Projects] and [http://wiki.osdev.org/FreeBASIC FreeBASIC] pages and also maintains the [http://wiki.osdev.org/Emulators Emulators] entry.

==ScorchOS==

ScorchOS is a 32-bit pmode kernel aimed at being light-weight, fast and easy-to-customize. ScorchOS 0.0.7 was an epic fail, but he's rewriting the kernel completely from scratch in FreeBASIC, and the target is to make it modular and monolithic. For the latest information head to http://www.scorchos.com/
````

<a id="w002658"></a>

## W002658 — Category:C

Bron-ID: W002658
Namespace: Category (14)
Revisie: 8482 | Datum: 2009-07-10T08:18:21Z
Bron: https://wiki.osdev.org/Category:C
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AC&oldid=8482

````text
[[Category:Languages]]
````

<a id="w002659"></a>

## W002659 — Category:C++

Bron-ID: W002659
Namespace: Category (14)
Revisie: 8483 | Datum: 2009-07-10T08:18:57Z
Bron: https://wiki.osdev.org/Category:C%2B%2B
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AC%2B%2B&oldid=8483

````text
[[Category:Languages]]
````

<a id="w002660"></a>

## W002660 — Category:Python

Bron-ID: W002660
Namespace: Category (14)
Revisie: 8486 | Datum: 2009-07-10T08:21:00Z
Bron: https://wiki.osdev.org/Category:Python
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3APython&oldid=8486

````text
[[Category:Languages]]
````

<a id="w002661"></a>

## W002661 — CSharp

Bron-ID: W002661
Namespace: Main (0)
Revisie: 8488 | Datum: 2009-07-10T08:23:05Z
Bron: https://wiki.osdev.org/CSharp
Revisie-URL: https://wiki.osdev.org/index.php?title=CSharp&oldid=8488
Doorverwijzing: C Sharp

````text
#REDIRECT [[C Sharp]]
````

<a id="w002662"></a>

## W002662 — Talk:CSharp

Bron-ID: W002662
Namespace: Talk (1)
Revisie: 8490 | Datum: 2009-07-10T08:23:06Z
Bron: https://wiki.osdev.org/Talk:CSharp
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ACSharp&oldid=8490
Doorverwijzing: Talk:C Sharp

````text
#REDIRECT [[Talk:C Sharp]]
````

<a id="w002664"></a>

## W002664 — Talk:C++ bare bones

Bron-ID: W002664
Namespace: Talk (1)
Revisie: 8498 | Datum: 2009-07-10T08:39:28Z
Bron: https://wiki.osdev.org/Talk:C%2B%2B_bare_bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AC%2B%2B_bare_bones&oldid=8498
Doorverwijzing: Talk:C++ Bare Bones

````text
#REDIRECT [[Talk:C++ Bare Bones]]
````

<a id="w002665"></a>

## W002665 — Dev-C PlusPlus

Bron-ID: W002665
Namespace: Main (0)
Revisie: 8375 | Datum: 2009-07-05T22:46:45Z
Bron: https://wiki.osdev.org/Dev-C_PlusPlus
Revisie-URL: https://wiki.osdev.org/index.php?title=Dev-C_PlusPlus&oldid=8375
Doorverwijzing: Dev-C++

````text
#REDIRECT [[Dev-C++]]
````

<a id="w002667"></a>

## W002667 — C PlusPlus

Bron-ID: W002667
Namespace: Main (0)
Revisie: 8357 | Datum: 2009-07-05T16:39:23Z
Bron: https://wiki.osdev.org/C_PlusPlus
Revisie-URL: https://wiki.osdev.org/index.php?title=C_PlusPlus&oldid=8357
Doorverwijzing: C++

````text
#REDIRECT [[C++]]
````

<a id="w002668"></a>

## W002668 — CPlusPlus

Bron-ID: W002668
Namespace: Main (0)
Revisie: 8516 | Datum: 2009-07-10T13:22:58Z
Bron: https://wiki.osdev.org/CPlusPlus
Revisie-URL: https://wiki.osdev.org/index.php?title=CPlusPlus&oldid=8516
Doorverwijzing: C++

````text
#REDIRECT [[C++]]
````

<a id="w002669"></a>

## W002669 — File:Graphicscard.png

Bron-ID: W002669
Namespace: File (6)
Revisie: 8530 | Datum: 2009-07-12T21:15:58Z
Bron: https://wiki.osdev.org/File:Graphicscard.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AGraphicscard.png&oldid=8530

````text
Overview of common graphics card components
````

<a id="w002670"></a>

## W002670 — Solar Assembler

Bron-ID: W002670
Namespace: Main (0)
Revisie: 28205 | Datum: 2023-07-10T18:44:08Z
Bron: https://wiki.osdev.org/Solar_Assembler
Revisie-URL: https://wiki.osdev.org/index.php?title=Solar_Assembler&oldid=28205

````text
{{In Progress}}

Solar Assembler (aka Sol_Asm) is a multipass macro assembler written by Bogdan Ontanu. It supports HLL primitives, like procedures, structures/unions, .IF/.ENDIF, .WHILE/.ENDWHILE, .REPEAT/.UNTIL, etc. Sol_Asm can produce binary files, PE executables (32/64-bit), COFF object files (32/64-bit), ELF object files (32/64-bit) and Mach-O object files (32-bit only).

==Sol_Asm syntax==
{{Stub}}

===Syntax comparison===
The syntax of Sol_Asm is easy to learn and has similarities with the syntax of [[TASM]] or [[MASM]]. 
This table shows the differences between Sol_Asm and other HLL assemblers:

{| {{wikitable}}
!
! SOL_ASM
! [[TASM]]
! [[MASM]] / [[JWASM]]
! [[FASM]] with HLL macros
|-
| Procedures
| <source lang="asm">
PROC TestProc
     USES eax, ebx
     ARG Var1, Var2
     LOCAL Var3

     ...
ENDP
</source>{{ref|1}}
| <source lang="asm">
TestProc PROC
     USES eax, ebx
     ARG Var1:DWORD, Var2:BYTE
     LOCAL Var3:WORD

     ...
ENDP
</source>
| <source lang="asm">
TestProc PROC \
     USES eax, ebx \
     Var1:DWORD, Var2:BYTE
     LOCAL Var3:WORD

     ...
TestProc ENDP
</source>
| <source lang="asm">
proc TestProc \
     USES eax ebx \
     Var1:DWORD, Var2:BYTE
     LOCAL Var3:WORD

     ...
endp
</source>
|-
| Structures
| <source lang="asm">
STRUC StructTest
    Var1     db ?
    Var2     dw ?
    Var3     dd ?
ENDS
</source>
| <source lang="asm">
StructTest STRUC
    Var1:    db ?
    Var2:    dw ?
    Var3:    dd ?
ENDS
</source>
| <source lang="asm">
StructTest STRUC
    Var1:    db ?
    Var2:    dw ?
    Var3:    dd ?
StrucTTest ENDS
</source>
| <source lang="asm">
struct StructTest
    Var1:    db ?
    Var2:    dw ?
    Var3:    dd ?
ends
</source>
|-
| Reserve non initialized data 
| <source lang="asm">
Buffer:    rb 256
</source>
| <source lang="asm">
Buffer:    db 256 dup(?)
</source>
| <source lang="asm">
Buffer:    db 256 dup(?)
</source>
| <source lang="asm">
Buffer:    db 256 dup(?)
or
Buffer:    rb 256
</source>
|-
| I/O ports
| <source lang="asm">
in   al, [92h]
or   al, 02h
out  [92h], al
</source>
| <source lang="asm">
in   al, 92h
or   al, 02h
out  92h, al
</source>
| <source lang="asm">
in   al, 92h
or   al, 02h
out  92h, al
</source>
| <source lang="asm">
in   al, 92h
or   al, 02h
out  92h, al
</source>
|-
| .IF/.ENDIF
| <source lang="asm">
.IF ( eax == 2 .or. ebx != 3 )
    ; something...
.ENDIF
</source>
| <source lang="asm">
.IF (eax == 2 or ebx != 3)
    ; something...
.ENDIF
</source>
| <source lang="asm">
.IF (eax == 2 or ebx != 3)
    ; something...
.ENDIF
</source>
| <source lang="asm">
.if (eax == 2 or ebx != 3)
    ; something...
.endif
</source>
|}

==Original features==
Sol_Asm also implements some original features.

===ENUMs===	
It is possible to define some constants like C language with the ENUM keyword.
This little example will generate: TASK_ZOMBIE equ 0, TASK_READY equ 1, TASK_RUNNING equ 2, etc.
<source lang="asm">
ENUM TASK, 0, 4
    TASK_ZOMBIE
    TASK_READY
    TASK_RUNNING
    TASK_WAITING
    TASK_SLEEPING
ENDE
</source>

===Resource compiler===
Sol_Asm includes a small resource compiler. This little example will define a dialog box:
<source lang="asm">
IDD_DIALOG1 equ 100
IDD_BUTTON1 equ 101

IDD_DIALOG1 DIALOGEX 10, 10, 320, 240
CAPTION "Hello!"
STYLE 0x10CF0000

BEGIN
    CONTROL "Hello", IDC_BUTTON1, "Button", 0x50010000, 10, 10, 50, 24, 0x00000000
END
</source>

* {{note|1}} by default, arguments and locals are always dwords.

==See Also==
===External Links===
*[http://www.oby.ro/sol_asm/index.html Solar Assembler homepage]
*[http://www.oby.ro/sol_asm/docs/sol_asm_manual.htm Solar Assembler Reference Manual]

[[Category:Assemblers]]
````

<a id="w002671"></a>

## W002671 — Multiprocessing

Bron-ID: W002671
Namespace: Main (0)
Revisie: 20657 | Datum: 2017-04-06T14:13:17Z
Bron: https://wiki.osdev.org/Multiprocessing
Revisie-URL: https://wiki.osdev.org/index.php?title=Multiprocessing&oldid=20657

````text
Multiprocessing involves more than one CPU in a computer. There are a variety of ways that multiprocessing may be implemented in hardware.

== SMP (Symmetric Multiprocessing) ==

In theory [[SMP]] means that all CPUs are identical. In practice there may be minor variations between CPUs (e.g. different revisions of the same family of CPUs), but these differences can usually be ignored by operating system software. SMP is the easiest form of multiprocessing for software to support. SMP includes systems with CPUs implemented in separate chips, systems with CPUs implemented in the same chip (multi-core) and combinations (e.g. a system with 2 separate quad core chips, with a total of 8 physical CPUs).

== SMT (Simultaneous Multithreading) ==

Normally a CPU executes one stream of instructions, where at any time some parts of the CPU may be busy and other parts of the CPU may be idle (e.g. while doing integer instructions parts of the CPU that do floating point calculations may be unused), and where the entire CPU may be idle in some cases (e.g. when the CPU needs to wait until data arrives from RAM before it can continue). The idea behind [[SMT]] is to execute more than one stream of instructions, which increases overall performance by reducing the chance that parts of the CPU become idle.

From the operating system's perspective, SMT looks a lot like SMP, except for performance characteristics. For SMT a single physical CPU executes multiple "threads" (or logical CPUs), and the performance of one thread/logical CPU is effected by work done by other threads/logical CPUs in the same physical CPU/core (as the physical CPU's resources are shared).

For the 80x86 architecture, Intel was the first manufacturer to implement SMT (called "hyperthreading" by Intel). When Intel first introduced hyperthreading it got some negative publicity due to performance problems; partly caused by the way it was implemented and partly because a lot of software wasn't designed for it (single-threaded) and operating system schedulers weren't optimized for it. Intel stopped using SMT/hyper-threading for a while; but since then software has caught up and later CPUs (Atom, Core i7) show significant performance improvements from SMT (up to about 30% performance improvement on multi-threaded loads). AMD's Ryzen platform, released in 2017, introduced SMT to a non-Intel CPU for the first time.

== AMP (Asymmetric Multiprocessing) ==

[[AMP]] means that at least one of the CPUs are different. For example, an embedded system might have an ARM CPU and a special purpose CPU for digital signal processing. AMP is rare for desktop/server computers, however this may change in future as Intel has plans to put stream processors ("Larrabee", mainly intended as a GPGPU to be used for graphics, physics and HPC) in the same system as general purpose 80x86 CPUs (possibly even in the same multi-core chip). Possibly, at some time in the future, it may be necessary for a general purpose 80x86 operating system to have 2 schedulers - one for traditional 80x86 CPUs and another for stream processors.

== NUMA (Non-Uniform Memory Access) ==

For SMP the time it takes to access a resource (e.g. RAM) is the same for all CPUs. For [[NUMA]] this isn't the case, and some CPUs may be able to access a resource faster than other CPUs. The name "NUMA" is misleading as RAM is not the only resource effected - it's entirely possible for some CPUs to have faster access to other devices (e.g. hard disk, video, ethernet, etc) than other CPUs.

Examples of NUMA are the AMD opteron and Intel Core i7 processors; where you might have 2 quad core CPUs and 2 sets of RAM chips, where each quad core CPU is directly connected to one set RAM chips. In this case for a CPU to access RAM that is not directly connected to it, it needs to ask the other quad core CPU to access the RAM on its behalf (which causes extra latency).

For NUMA systems, resources are typically split up into "NUMA domains". For the example above, there's 2 NUMA domains, and each NUMA domain consists of a quad core CPU and a set of RAM chips. Each NUMA domain may contain none or more CPUs, none or more bytes of RAM, and none or more I/O hubs (e.g. PCI host bridges).

The opposite of NUMA is UMA (Uniform Memory Access). An example of UMA is a computer with a pair of Pentium III CPUs, or a computer with one multi-core Opteron CPU.

UMA could be considered a special case of NUMA where there's only one NUMA domain. This might not make too much sense at first glance, until you consider an operating system that is designed for NUMA that happens to be running on a UMA computer.

An operating system may be optimized for NUMA. For example, the scheduler, memory management and device management might all be optimized to try to minimize the need for one CPU to access "distant" (slower) resources.

=== ccNUMA (Cache Coherent Non-Uniform Memory Access) ===

Most NUMA systems are "cache coherent", which means that things done by one CPU take into account the state of caches in other CPUs. Cache coherency makes it a lot easier to write software, as (excluding performance characteristics) it's not very different to SMP.

Some (very rare) systems are not cache coherent, which can mean that software must explicitly manage caches to ensure that CPUs aren't working on stale data (data that was since been modified by other CPUs); or it can mean that CPUs have "local RAM" that only they can access, where other CPUs can't access it at all (in this case, the computer behaves a lot like separate computers connected by very fast networking).

=== NUMA Ratio ===

A computer's NUMA ratio is a measure of how quickly a CPU can access "close" (fast) RAM compared to how quickly a CPU can access "distant" (slower) RAM. For example, if the NUMA ratio is 2.0 then it takes twice as long for a CPU to access "distant" RAM.

It's impossible to have a NUMA ratio of 1.0; as this indicates that there's no difference between "close" RAM and "distant" RAM, and therefore the system is not actually NUMA at all.

For most NUMA 80x86 systems the NUMA ratio is quite low (around 1.2), and an operating system could treat these systems as UMA without severe performance problems.

== Topology ==

So that an operating system can make effective decisions concerning resource usage (e.g. for the purpose of improving performance; or reducing power consumption, heat and/or noise) it needs to understand the relationships between different resources. This involves mapping the topology of the computer, so that the operating system can refer to this map when making decisions.

This map may include the contents of each NUMA domain (number of CPUs and their IDs, number of RAM areas and their sizes and locations in the physical address space, number of I/O hubs and their identification, etc), plus performance data (e.g. tables that can be used to determine the relative cost of accessing a specific location in RAM, a specific I/O hub and/or another CPU from any CPU; and/or information about cache sharing between CPUs).

==See Also==

===Threads===

[[Category:OS theory]]

[[Category:Multiprocessing]]
````

<a id="w002674"></a>

## W002674 — AHCI

Bron-ID: W002674
Namespace: Main (0)
Revisie: 29504 | Datum: 2025-04-29T14:21:32Z
Bron: https://wiki.osdev.org/AHCI
Revisie-URL: https://wiki.osdev.org/index.php?title=AHCI&oldid=29504

````text
The specification:
https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/serial-ata-ahci-spec-rev1-3-1.pdf

== Introduction ==

AHCI (Advance Host Controller Interface) is developed by Intel to facilitate handling [[SATA]] devices. The AHCI specification emphasizes that an AHCI controller (referred to as host bus adapter, or HBA) is designed to be a data movement engine between system memory and SATA devices. It encapsulates SATA devices and provides a standard [[PCI]] interface to the host. System designers can easily access SATA drives using system memory and memory mapped registers, without the need for manipulating the annoying task files as IDE do.

An AHCI controller may support up to 32 ports which can attach different SATA devices such as disk drives, port multipliers, or an enclosure management bridge. AHCI supports all native SATA features such as command queueing, hot plugging, power management, etc. To a software developer, an AHCI controller is just a PCI device with bus master capability.

AHCI is a new standard compared to [[IDE]], which has been around for twenty years. There exists little documentation about its programming tips and tricks. Possibly the only available resource is the Intel AHCI specification (see [[#External Links|External Links]]) and some open source operating systems such as Linux. This article shows the minimal steps an OS (not BIOS) should do to put AHCI controller into a workable state, how to identify drives attached, and how to read physical sectors from a SATA disk. To keep concise, many technical details and deep explanations of some data structures have been omitted.

It should be noted that IDE also supports SATA devices and there are still debates about which one, IDE or AHCI, is better. Some tests even show that a SATA disk acts better in IDE mode than AHCI mode. But the common idea is that AHCI performs better and will be the standard PC to SATA interface, though some driver software should be enhanced to fully cultivate AHCI capability.

All the diagrams in this article are copied from the Intel AHCI specification 1.3.

== SATA basic ==

There are at least two SATA standards maintained respectively by [http://www.t13.org T13] and [http://www.sata-io.org SATA-IO]. The SATA-IO focuses on serial ATA and T13 encompasses traditional parallel ATA specifications as well.

While the hardware specifications for IDE and SATA (and even between different devices implementing them) differ greatly, the API and ABI are very similar. To a software developer, the biggest difference between SATA and parallel ATA is that SATA uses FIS (Frame Information Structure) packet to transport data between host and device. An FIS can be viewed as a data set of traditional task files, or an encapsulation of ATA commands. SATA uses the same command set as parallel ATA.

'''1) FIS types'''

Following code defines different kinds of FIS specified in Serial ATA Revision 3.0.
<syntaxhighlight lang="c">
typedef enum
{
	FIS_TYPE_REG_H2D	= 0x27,	// Register FIS - host to device
	FIS_TYPE_REG_D2H	= 0x34,	// Register FIS - device to host
	FIS_TYPE_DMA_ACT	= 0x39,	// DMA activate FIS - device to host
	FIS_TYPE_DMA_SETUP	= 0x41,	// DMA setup FIS - bidirectional
	FIS_TYPE_DATA		= 0x46,	// Data FIS - bidirectional
	FIS_TYPE_BIST		= 0x58,	// BIST activate FIS - bidirectional
	FIS_TYPE_PIO_SETUP	= 0x5F,	// PIO setup FIS - device to host
	FIS_TYPE_DEV_BITS	= 0xA1,	// Set device bits FIS - device to host
} FIS_TYPE;
</syntaxhighlight>

'''2) Register FIS – Host to Device'''

A host to device register FIS is used by the host to send command or control to a device. As illustrated in the following data structure, it contains the IDE registers such as command, LBA, device, feature, count and control. An ATA command is constructed in this structure and issued to the device. All reserved fields in an FIS should be cleared to zero.

<syntaxhighlight lang="c">
typedef struct tagFIS_REG_H2D
{
	// DWORD 0
	uint8_t  fis_type;	// FIS_TYPE_REG_H2D

	uint8_t  pmport:4;	// Port multiplier
	uint8_t  rsv0:3;		// Reserved
	uint8_t  c:1;		// 1: Command, 0: Control

	uint8_t  command;	// Command register
	uint8_t  featurel;	// Feature register, 7:0
	
	// DWORD 1
	uint8_t  lba0;		// LBA low register, 7:0
	uint8_t  lba1;		// LBA mid register, 15:8
	uint8_t  lba2;		// LBA high register, 23:16
	uint8_t  device;		// Device register

	// DWORD 2
	uint8_t  lba3;		// LBA register, 31:24
	uint8_t  lba4;		// LBA register, 39:32
	uint8_t  lba5;		// LBA register, 47:40
	uint8_t  featureh;	// Feature register, 15:8

	// DWORD 3
	uint8_t  countl;		// Count register, 7:0
	uint8_t  counth;		// Count register, 15:8
	uint8_t  icc;		// Isochronous command completion
	uint8_t  control;	// Control register

	// DWORD 4
	uint8_t  rsv1[4];	// Reserved
} FIS_REG_H2D;
</syntaxhighlight>

'''3) Register FIS – Device to Host'''

A device to host register FIS is used by the device to notify the host that some ATA register has changed. It contains the updated task files such as status, error and other registers.

<syntaxhighlight lang="c">
typedef struct tagFIS_REG_D2H
{
	// DWORD 0
	uint8_t  fis_type;    // FIS_TYPE_REG_D2H

	uint8_t  pmport:4;    // Port multiplier
	uint8_t  rsv0:2;      // Reserved
	uint8_t  i:1;         // Interrupt bit
	uint8_t  rsv1:1;      // Reserved

	uint8_t  status;      // Status register
	uint8_t  error;       // Error register
	
	// DWORD 1
	uint8_t  lba0;        // LBA low register, 7:0
	uint8_t  lba1;        // LBA mid register, 15:8
	uint8_t  lba2;        // LBA high register, 23:16
	uint8_t  device;      // Device register

	// DWORD 2
	uint8_t  lba3;        // LBA register, 31:24
	uint8_t  lba4;        // LBA register, 39:32
	uint8_t  lba5;        // LBA register, 47:40
	uint8_t  rsv2;        // Reserved

	// DWORD 3
	uint8_t  countl;      // Count register, 7:0
	uint8_t  counth;      // Count register, 15:8
	uint8_t  rsv3[2];     // Reserved

	// DWORD 4
	uint8_t  rsv4[4];     // Reserved
} FIS_REG_D2H;
</syntaxhighlight>

'''4) Data FIS – Bidirectional'''

This FIS is used by the host or device to send data payload. The data size can be varied.

<syntaxhighlight lang="c">
typedef struct tagFIS_DATA
{
	// DWORD 0
	uint8_t  fis_type;	// FIS_TYPE_DATA

	uint8_t  pmport:4;	// Port multiplier
	uint8_t  rsv0:4;		// Reserved

	uint8_t  rsv1[2];	// Reserved

	// DWORD 1 ~ N
	uint32_t data[1];	// Payload
} FIS_DATA;
</syntaxhighlight>

'''5) PIO Setup – Device to Host'''

This FIS is used by the device to tell the host that it’s about to send or ready to receive a PIO data payload.

<syntaxhighlight lang="c">
typedef struct tagFIS_PIO_SETUP
{
	// DWORD 0
	uint8_t  fis_type;	// FIS_TYPE_PIO_SETUP

	uint8_t  pmport:4;	// Port multiplier
	uint8_t  rsv0:1;		// Reserved
	uint8_t  d:1;		// Data transfer direction, 1 - device to host
	uint8_t  i:1;		// Interrupt bit
	uint8_t  rsv1:1;

	uint8_t  status;		// Status register
	uint8_t  error;		// Error register

	// DWORD 1
	uint8_t  lba0;		// LBA low register, 7:0
	uint8_t  lba1;		// LBA mid register, 15:8
	uint8_t  lba2;		// LBA high register, 23:16
	uint8_t  device;		// Device register

	// DWORD 2
	uint8_t  lba3;		// LBA register, 31:24
	uint8_t  lba4;		// LBA register, 39:32
	uint8_t  lba5;		// LBA register, 47:40
	uint8_t  rsv2;		// Reserved

	// DWORD 3
	uint8_t  countl;		// Count register, 7:0
	uint8_t  counth;		// Count register, 15:8
	uint8_t  rsv3;		// Reserved
	uint8_t  e_status;	// New value of status register

	// DWORD 4
	uint16_t tc;		// Transfer count
	uint8_t  rsv4[2];	// Reserved
} FIS_PIO_SETUP;
</syntaxhighlight>

'''6) DMA Setup – Device to Host'''

<syntaxhighlight lang="c">
typedef struct tagFIS_DMA_SETUP
{
	// DWORD 0
	uint8_t  fis_type;	// FIS_TYPE_DMA_SETUP

	uint8_t  pmport:4;	// Port multiplier
	uint8_t  rsv0:1;		// Reserved
	uint8_t  d:1;		// Data transfer direction, 1 - device to host
	uint8_t  i:1;		// Interrupt bit
	uint8_t  a:1;            // Auto-activate. Specifies if DMA Activate FIS is needed

        uint8_t  rsved[2];       // Reserved

	//DWORD 1&2

        uint64_t DMAbufferID;    // DMA Buffer Identifier. Used to Identify DMA buffer in host memory.
                                 // SATA Spec says host specific and not in Spec. Trying AHCI spec might work.

        //DWORD 3
        uint32_t rsvd;           //More reserved

        //DWORD 4
        uint32_t DMAbufOffset;   //Byte offset into buffer. First 2 bits must be 0

        //DWORD 5
        uint32_t TransferCount;  //Number of bytes to transfer. Bit 0 must be 0

        //DWORD 6
        uint32_t resvd;          //Reserved
        
} FIS_DMA_SETUP;
</syntaxhighlight>

'''7) Example'''

This example illustrates the steps to read the Identify data from a device. Error detection and recovery is ignored.

To issue an ATA Identify command to the device, the FIS is constructed at follows.

<syntaxhighlight lang="c">
FIS_REG_H2D fis;
memset(&fis, 0, sizeof(FIS_REG_H2D));
fis.fis_type = FIS_TYPE_REG_H2D;
fis.command = ATA_CMD_IDENTIFY;	// 0xEC
fis.device = 0;			// Master device
fis.c = 1;				// Write command register
</syntaxhighlight>

After the device receives this FIS and successfully read the 256 words data into its internal buffer, it sends a PIO Setup FIS – Device to Host to tell the host that it’s ready to transfer data and the data size (FIS_PIO_SETUP.tc).

After the PIO Setup FIS – Device to Host has been sent correctly, the device sends a DATA FIS to the host which contains the received data payload (FIS_DATA.data).

This scenario is described in SATA revision 3.0 as a PIO data-in command protocol. But an AHCI controller will do the latter two steps for the host. The host software needs only setup and issue the command FIS, and tells the AHCI controller the memory address and size to store the received data. After everything is done, the AHCI controller will issue an interrupt (if enabled) to notify the host to check the data.

== Find an AHCI controller ==

An AHCI controller can be found by enumerating the PCI bus. It has a class id 0x01 (mass storage device) and normally a subclass id 0x06 (serial ATA, but can be IDE Interface 0x01). The vendor id and device id should also be checked to ensure it’s really an AHCI controller.

=== Determining what mode the controller is in ===
As you may be aware, a SATA controller can either be in IDE emulation mode or in AHCI mode. The problem that enters here is simple: <br />
'''How to find what mode the controller is in'''. The documentation is really obscure on this. Perhaps the best way is to initialize a SATA controller as both IDE and AHCI. In this way, as long as you are careful about non-existent ports, you cannot go wrong.

One possible way of doing this is by checking the bit 31 of GHC register. It's labeled as AHCI Enable.

== AHCI Registers and Memory Structures ==

As mentioned above, host communicates with the AHCI controller through system memory and memory mapped registers. The last PCI base address register (BAR[5], header offset 0x24) points to the AHCI base memory, it’s called ABAR (AHCI Base Memory Register). All AHCI registers and memories can be located through ABAR. The other PCI base address registers act same as a traditional IDE controller. Some AHCI controller can be configured to simulate a legacy IDE one.

'''1) HBA memory registers'''

An AHCI controller can support up to 32 ports. HBA memory registers can be divided into two parts: Generic Host Control registers and Port Control registers. Generic Host Control registers controls the behavior of the whole controller, while each port owns its own set of Port Control registers. The actual ports an AHCI controller supported and implemented can be calculated from the Capacity register (HBA_MEM.cap) and the Port Implemented register (HBA_MEM.pi).

[[Image:HBA_registers.jpg‎]]

<syntaxhighlight lang="c">
typedef volatile struct tagHBA_MEM
{
	// 0x00 - 0x2B, Generic Host Control
	uint32_t cap;		// 0x00, Host capability
	uint32_t ghc;		// 0x04, Global host control
	uint32_t is;		// 0x08, Interrupt status
	uint32_t pi;		// 0x0C, Port implemented
	uint32_t vs;		// 0x10, Version
	uint32_t ccc_ctl;	// 0x14, Command completion coalescing control
	uint32_t ccc_pts;	// 0x18, Command completion coalescing ports
	uint32_t em_loc;		// 0x1C, Enclosure management location
	uint32_t em_ctl;		// 0x20, Enclosure management control
	uint32_t cap2;		// 0x24, Host capabilities extended
	uint32_t bohc;		// 0x28, BIOS/OS handoff control and status

	// 0x2C - 0x9F, Reserved
	uint8_t  rsv[0xA0-0x2C];

	// 0xA0 - 0xFF, Vendor specific registers
	uint8_t  vendor[0x100-0xA0];

	// 0x100 - 0x10FF, Port control registers
	HBA_PORT	ports[1];	// 1 ~ 32
} HBA_MEM;

typedef volatile struct tagHBA_PORT
{
	uint32_t clb;		// 0x00, command list base address, 1K-byte aligned
	uint32_t clbu;		// 0x04, command list base address upper 32 bits
	uint32_t fb;		// 0x08, FIS base address, 256-byte aligned
	uint32_t fbu;		// 0x0C, FIS base address upper 32 bits
	uint32_t is;		// 0x10, interrupt status
	uint32_t ie;		// 0x14, interrupt enable
	uint32_t cmd;		// 0x18, command and status
	uint32_t rsv0;		// 0x1C, Reserved
	uint32_t tfd;		// 0x20, task file data
	uint32_t sig;		// 0x24, signature
	uint32_t ssts;		// 0x28, SATA status (SCR0:SStatus)
	uint32_t sctl;		// 0x2C, SATA control (SCR2:SControl)
	uint32_t serr;		// 0x30, SATA error (SCR1:SError)
	uint32_t sact;		// 0x34, SATA active (SCR3:SActive)
	uint32_t ci;		// 0x38, command issue
	uint32_t sntf;		// 0x3C, SATA notification (SCR4:SNotification)
	uint32_t fbs;		// 0x40, FIS-based switch control
	uint32_t rsv1[11];	// 0x44 ~ 0x6F, Reserved
	uint32_t vendor[4];	// 0x70 ~ 0x7F, vendor specific
} HBA_PORT;
</syntaxhighlight>

This memory area should be configured as uncacheable as they are memory mapped hardware registers, not normal prefetchable RAM. For the same reason, the data structures are declared as "volatile" to prevent the compiler from over optimizing the code.

'''2) Port Received FIS and Command List Memory'''

Each port can attach a single SATA device. Host sends commands to the device using Command List and device delivers information to the host using Received FIS structure. They are located at HBA_PORT.clb/clbu, and HBA_PORT.fb/fbu. The most important part of AHCI initialization is to set correctly these two pointers and the data structures they point to.

[[Image:Port_memory.jpg‎]]

'''3) Received FIS'''

There are four kinds of FIS which may be sent to the host by the device as indicated in the following structure declaration. When an FIS has been copied into the host specified memory, an according bit will be set in the Port Interrupt Status register (HBA_PORT.is). 

Data FIS – Device to Host is not copied to this structure. Data payload is sent and received through PRDT (Physical Region Descriptor Table) in Command List, as will be introduced later.

<syntaxhighlight lang="c">
typedef volatile struct tagHBA_FIS
{
	// 0x00
	FIS_DMA_SETUP	dsfis;		// DMA Setup FIS
	uint8_t         pad0[4];

	// 0x20
	FIS_PIO_SETUP	psfis;		// PIO Setup FIS
	uint8_t         pad1[12];

	// 0x40
	FIS_REG_D2H	rfis;		// Register – Device to Host FIS
	uint8_t         pad2[4];

	// 0x58
	FIS_DEV_BITS	sdbfis;		// Set Device Bit FIS
	
	// 0x60
	uint8_t         ufis[64];

	// 0xA0
	uint8_t   	rsv[0x100-0xA0];
} HBA_FIS;
</syntaxhighlight>

'''4) Command List'''

Host sends commands to the device through Command List. Command List consists of 1 to 32 command headers, each one is called a slot. Each command header describes an ATA or ATAPI command, including a Command FIS, an ATAPI command buffer and a bunch of Physical Region Descriptor Tables specifying the data payload address and size.

To send a command, the host constructs a command header, and set the according bit in the Port Command Issue register (HBA_PORT.ci). The AHCI controller will automatically send the command to the device and wait for response. If there are some errors, error bits in the Port Interrupt register (HBA_PORT.is) will be set and additional information can be retrieved from the Port Task File register (HBA_PORT.tfd), SStatus register (HBA_PORT.ssts) and SError register (HBA_PORT.serr). If it succeeds, the Command Issue register bit will be cleared and the received data payload, if any, will be copied from the device to the host memory by the AHCI controller.

How many slots a Command List holds can be got from the Host capability register (HBA_MEM.cap). It must be within 1 and 32. SATA supports queued commands to increase throughput. Unlike traditional parallel ATA drive; a SATA drive can process a new command when an old one is still running. With AHCI, a host can send up to 32 commands to device simultaneously.

[[Image:Command_list.jpg‎]]

<syntaxhighlight lang="c">
typedef struct tagHBA_CMD_HEADER
{
	// DW0
	uint8_t  cfl:5;		// Command FIS length in DWORDS, 2 ~ 16
	uint8_t  a:1;		// ATAPI
	uint8_t  w:1;		// Write, 1: H2D, 0: D2H
	uint8_t  p:1;		// Prefetchable

	uint8_t  r:1;		// Reset
	uint8_t  b:1;		// BIST
	uint8_t  c:1;		// Clear busy upon R_OK
	uint8_t  rsv0:1;		// Reserved
	uint8_t  pmp:4;		// Port multiplier port

	uint16_t prdtl;		// Physical region descriptor table length in entries

	// DW1
	volatile
	uint32_t prdbc;		// Physical region descriptor byte count transferred

	// DW2, 3
	uint32_t ctba;		// Command table descriptor base address
	uint32_t ctbau;		// Command table descriptor base address upper 32 bits

	// DW4 - 7
	uint32_t rsv1[4];	// Reserved
} HBA_CMD_HEADER;
</syntaxhighlight>

'''5) Command Table and Physical Region Descriptor Table'''

As described above, a command table contains an ATA command FIS, an ATAPI command buffer and a bunch of PRDT (Physical Region Descriptor Table) specifying the data payload address and size.

A command table may have 0 to 65535 PRDT entries. The actual PRDT entries count is set in the command header (HBA_CMD_HEADER.prdtl). As an example, if a host wants to read 100K bytes continuously from a disk, the first half to memory address A1, and the second half to address A2. It must set two PRDT entries, the first PRDT.DBA = A1, and the second PRDT.DBA = A2.

An AHCI controller acts as a PCI bus master to transfer data payload directly between device and system memory.

[[Image:Command_table.jpg‎]]

<syntaxhighlight lang="c">
typedef struct tagHBA_CMD_TBL
{
	// 0x00
	uint8_t  cfis[64];	// Command FIS

	// 0x40
	uint8_t  acmd[16];	// ATAPI command, 12 or 16 bytes

	// 0x50
	uint8_t  rsv[48];	// Reserved

	// 0x80
	HBA_PRDT_ENTRY	prdt_entry[1];	// Physical region descriptor table entries, 0 ~ 65535
} HBA_CMD_TBL;

typedef struct tagHBA_PRDT_ENTRY
{
	uint32_t dba;		// Data base address
	uint32_t dbau;		// Data base address upper 32 bits
	uint32_t rsv0;		// Reserved

	// DW3
	uint32_t dbc:22;		// Byte count, 4M max
	uint32_t rsv1:9;		// Reserved
	uint32_t i:1;		// Interrupt on completion
} HBA_PRDT_ENTRY;
</syntaxhighlight>

== Detect attached SATA devices ==

'''1) Which port has a device attached'''

As specified in the AHCI specification, firmware (BIOS) should initialize the AHCI controller into a minimal workable state. OS usually needn’t reinitialize it from the bottom. Much information is already there when the OS boots.

The Port Implemented register (HBA_MEM.pi) is a 32 bit value and each bit represents a port. If the bit is set in this register, then the corresponding port is present (note: this does not necessarily mean a device is attached to the port). A device is attached to a port if the value of the port's HBA_PxSSTS.det field is 0x3.

'''2) What kind of device is attached'''

There are four kinds of SATA devices, and their signatures are defined as below. The Port Signature register (HBA_PORT.sig) contains the device signature, just read this register to find which kind of device is attached at the port. Some buggy AHCI controllers may not set the Signature register correctly. The most reliable way is to judge from the Identify data read back from the device.

<syntaxhighlight lang="c">
#define	SATA_SIG_ATA	0x00000101	// SATA drive
#define	SATA_SIG_ATAPI	0xEB140101	// SATAPI drive
#define	SATA_SIG_SEMB	0xC33C0101	// Enclosure management bridge
#define	SATA_SIG_PM	0x96690101	// Port multiplier

#define AHCI_DEV_NULL 0
#define AHCI_DEV_SATA 1
#define AHCI_DEV_SEMB 2
#define AHCI_DEV_PM 3
#define AHCI_DEV_SATAPI 4

#define HBA_PORT_IPM_ACTIVE 1
#define HBA_PORT_DET_PRESENT 3

void probe_port(HBA_MEM *abar)
{
	// Search disk in implemented ports
	uint32_t pi = abar->pi;
	int i = 0;
	while (i<32)
	{
		if (pi & 1)
		{
			int dt = check_type(&abar->ports[i]);
			if (dt == AHCI_DEV_SATA)
			{
				trace_ahci("SATA drive found at port %d\n", i);
			}
			else if (dt == AHCI_DEV_SATAPI)
			{
				trace_ahci("SATAPI drive found at port %d\n", i);
			}
			else if (dt == AHCI_DEV_SEMB)
			{
				trace_ahci("SEMB drive found at port %d\n", i);
			}
			else if (dt == AHCI_DEV_PM)
			{
				trace_ahci("PM drive found at port %d\n", i);
			}
			else
			{
				trace_ahci("No drive found at port %d\n", i);
			}
		}

		pi >>= 1;
		i ++;
	}
}

// Check device type
static int check_type(HBA_PORT *port)
{
	uint32_t ssts = port->ssts;

	uint8_t ipm = (ssts >> 8) & 0x0F;
	uint8_t det = ssts & 0x0F;

	if (det != HBA_PORT_DET_PRESENT)	// Check drive status
		return AHCI_DEV_NULL;
	if (ipm != HBA_PORT_IPM_ACTIVE)
		return AHCI_DEV_NULL;

	switch (port->sig)
	{
	case SATA_SIG_ATAPI:
		return AHCI_DEV_SATAPI;
	case SATA_SIG_SEMB:
		return AHCI_DEV_SEMB;
	case SATA_SIG_PM:
		return AHCI_DEV_PM;
	default:
		return AHCI_DEV_SATA;
	}
}
</syntaxhighlight>

== AHCI port memory space initialization ==

BIOS may have already configured all the necessary AHCI memory spaces. But the OS usually needs to reconfigure them to make them fit its requirements. It should be noted that Command List must be located at 1K aligned memory address and Received FIS be 256 bytes aligned.

Before rebasing Port memory space, OS must wait for current pending commands to finish and tell HBA to stop receiving FIS from the port. Otherwise an accidently incoming FIS may be written into a partially configured memory area. This is done by checking and setting corresponding bits at the Port Command And Status register (HBA_PORT.cmd). The example subroutines stop_cmd() and start_cmd() do the job.

The following example assumes that the HBA has 32 ports implemented and each port contains 32 command slots, and will allocate 8 PRDTs for each command slot. (Note: unlike in the above struct definitions, this is using 8 instead of 1)

<syntaxhighlight lang="c">
#define	AHCI_BASE	0x400000	// 4M

#define HBA_PxCMD_ST    0x0001
#define HBA_PxCMD_FRE   0x0010
#define HBA_PxCMD_FR    0x4000
#define HBA_PxCMD_CR    0x8000

void port_rebase(HBA_PORT *port, int portno)
{
	stop_cmd(port);	// Stop command engine

	// Command list offset: 1K*portno
	// Command list entry size = 32
	// Command list entry maxim count = 32
	// Command list maxim size = 32*32 = 1K per port
	port->clb = AHCI_BASE + (portno<<10);
	port->clbu = 0;
	memset((void*)(port->clb), 0, 1024);

	// FIS offset: 32K+256*portno
	// FIS entry size = 256 bytes per port
	port->fb = AHCI_BASE + (32<<10) + (portno<<8);
	port->fbu = 0;
	memset((void*)(port->fb), 0, 256);

	// Command table offset: 40K + 8K*portno
	// Command table size = 256*32 = 8K per port
	HBA_CMD_HEADER *cmdheader = (HBA_CMD_HEADER*)(port->clb);
	for (int i=0; i<32; i++)
	{
		cmdheader[i].prdtl = 8;	// 8 prdt entries per command table
					// 256 bytes per command table, 64+16+48+16*8
		// Command table offset: 40K + 8K*portno + cmdheader_index*256
		cmdheader[i].ctba = AHCI_BASE + (40<<10) + (portno<<13) + (i<<8);
		cmdheader[i].ctbau = 0;
		memset((void*)cmdheader[i].ctba, 0, 256);
	}

	start_cmd(port);	// Start command engine
}

// Start command engine
void start_cmd(HBA_PORT *port)
{
	// Wait until CR (bit15) is cleared
	while (port->cmd & HBA_PxCMD_CR)
		;

	// Set FRE (bit4) and ST (bit0)
	port->cmd |= HBA_PxCMD_FRE;
	port->cmd |= HBA_PxCMD_ST; 
}

// Stop command engine
void stop_cmd(HBA_PORT *port)
{
	// Clear ST (bit0)
	port->cmd &= ~HBA_PxCMD_ST;

	// Clear FRE (bit4)
	port->cmd &= ~HBA_PxCMD_FRE;

	// Wait until FR (bit14), CR (bit15) are cleared
	while(1)
	{
		if (port->cmd & HBA_PxCMD_FR)
			continue;
		if (port->cmd & HBA_PxCMD_CR)
			continue;
		break;
	}

}
</syntaxhighlight>

== AHCI & ATAPI ==

The documentation regarding using the AHCI interface to access an ATAPI device (most likely an optical drive) is rather poorly explained in the specification. However, once you understand that the HBA does most of the work for you it is rather simple. The AHCI/ATAPI method works by issuing the ATA PACKET command (0xA0) instead of the READ (READ is shown in the example below) and populating the ACMD field of the HBA_CMD_TBL with the 12/16 byte ATAPI command and setting the 'a' field to 1 in the HBA_CMD_HEADER which tells the HBA to perform the multi-step process (all done automatically) of transmitting the PACKET command, then sending the ATAPI device the ACMD.

== Example - Read hard disk sectors ==

The code example shows how to read "count" sectors from sector offset "starth:startl" to "buf" with LBA48 mode from HBA "port". Every PRDT entry contains 8K bytes data payload at most.

<syntaxhighlight lang="c">
#define ATA_DEV_BUSY 0x80
#define ATA_DEV_DRQ 0x08

bool read(HBA_PORT *port, uint32_t startl, uint32_t starth, uint32_t count, uint16_t *buf)
{
	port->is = (uint32_t) -1;		// Clear pending interrupt bits
	int spin = 0; // Spin lock timeout counter
	int slot = find_cmdslot(port);
	if (slot == -1)
		return false;

	HBA_CMD_HEADER *cmdheader = (HBA_CMD_HEADER*)port->clb;
	cmdheader += slot;
	cmdheader->cfl = sizeof(FIS_REG_H2D)/sizeof(uint32_t);	// Command FIS size
	cmdheader->w = 0;		// Read from device
	cmdheader->prdtl = (uint16_t)((count-1)>>4) + 1;	// PRDT entries count

	HBA_CMD_TBL *cmdtbl = (HBA_CMD_TBL*)(cmdheader->ctba);
	memset(cmdtbl, 0, sizeof(HBA_CMD_TBL) +
 		(cmdheader->prdtl-1)*sizeof(HBA_PRDT_ENTRY));

	// 8K bytes (16 sectors) per PRDT
	for (int i=0; i<cmdheader->prdtl-1; i++)
	{
		cmdtbl->prdt_entry[i].dba = (uint32_t) buf;
		cmdtbl->prdt_entry[i].dbc = 8*1024-1;	// 8K bytes (this value should always be set to 1 less than the actual value)
		cmdtbl->prdt_entry[i].i = 1;
		buf += 4*1024;	// 4K words
		count -= 16;	// 16 sectors
	}
	// Last entry
	cmdtbl->prdt_entry[i].dba = (uint32_t) buf;
	cmdtbl->prdt_entry[i].dbc = (count<<9)-1;	// 512 bytes per sector
	cmdtbl->prdt_entry[i].i = 1;

	// Setup command
	FIS_REG_H2D *cmdfis = (FIS_REG_H2D*)(&cmdtbl->cfis);

	cmdfis->fis_type = FIS_TYPE_REG_H2D;
	cmdfis->c = 1;	// Command
	cmdfis->command = ATA_CMD_READ_DMA_EX;

	cmdfis->lba0 = (uint8_t)startl;
	cmdfis->lba1 = (uint8_t)(startl>>8);
	cmdfis->lba2 = (uint8_t)(startl>>16);
	cmdfis->device = 1<<6;	// LBA mode

	cmdfis->lba3 = (uint8_t)(startl>>24);
	cmdfis->lba4 = (uint8_t)starth;
	cmdfis->lba5 = (uint8_t)(starth>>8);

	cmdfis->countl = count & 0xFF;
	cmdfis->counth = (count >> 8) & 0xFF;

	// The below loop waits until the port is no longer busy before issuing a new command
	while ((port->tfd & (ATA_DEV_BUSY | ATA_DEV_DRQ)) && spin < 1000000)
	{
		spin++;
	}
	if (spin == 1000000)
	{
		trace_ahci("Port is hung\n");
		return FALSE;
	}

	port->ci = 1<<slot;	// Issue command

	// Wait for completion
	while (1)
	{
		// In some longer duration reads, it may be helpful to spin on the DPS bit 
		// in the PxIS port field as well (1 << 5)
		if ((port->ci & (1<<slot)) == 0) 
			break;
		if (port->is & HBA_PxIS_TFES)	// Task file error
		{
			trace_ahci("Read disk error\n");
			return FALSE;
		}
	}

	// Check again
	if (port->is & HBA_PxIS_TFES)
	{
		trace_ahci("Read disk error\n");
		return FALSE;
	}

	return true;
}

// Find a free command list slot
int find_cmdslot(HBA_PORT *port)
{
	// If not set in SACT and CI, the slot is free
	uint32_t slots = (port->sact | port->ci);
	for (int i=0; i<cmdslots; i++)
	{
		if ((slots&1) == 0)
			return i;
		slots >>= 1;
	}
	trace_ahci("Cannot find free command list entry\n");
	return -1;
}
</syntaxhighlight>

== Checklist ==

=== Initialisation ===

* Enable interrupts, DMA, and memory space access in the PCI command register
* Memory map BAR 5 register as uncacheable.
* Perform BIOS/OS handoff (if the bit in the extended capabilities is set)
* Reset controller
* Register IRQ handler, using interrupt line given in the PCI register. This interrupt line may be shared with other devices, so the usual implications of this apply.
* Enable AHCI mode and interrupts in global host control register.
* Read capabilities registers. Check 64-bit DMA is supported if you need it.
* For all the implemented ports:
** Allocate physical memory for its command list, the received FIS, and its command tables. Make sure the command tables are 128 byte aligned.
** Memory map these as uncacheable.
** Set command list and received FIS address registers (and upper registers, if supported).
** Setup command list entries to point to the corresponding command table.
** Reset the port.
** Start command list processing with the port's command register. 
** Enable interrupts for the port. The D2H bit will signal completed commands.
** Read signature/status of the port to see if it connected to a drive.
** Send IDENTIFY ATA command to connected drives. Get their sector size and count.

=== Start read/write command ===

* Select an available command slot to use.
* Setup command FIS.
* Setup PRDT.
* Setup command list entry.
* Issue the command, and record separately that you have issued it.

=== IRQ handler ===

* Check global interrupt status. Write back its value. For all the ports that have a corresponding set bit...
* Check the port interrupt status. Write back its value. If zero, continue to the next port.
* If error bit set, reset port/retry commands as necessary.
* Compare issued commands register to the commands you have recorded as issuing. For any bits where a command was issued but is no longer running, this means that the command has completed. 
* Once done, continue checking if any other devices sharing the IRQ also need servicing.

== External Links ==

*[http://www.intel.com/technology/serialata/ahci.htm Serial ATA Advance Host Controller Interface (AHCI) 1.3]
*[http://www.sata-io.org Serial ATA Revision 3.0]
*[http://www.t13.org ATA8-ACS, ATA8-AAM]
*[https://github.com/haiku/haiku/tree/master/src/add-ons/kernel/busses/scsi/ahci Haiku's AHCI implementation]
*[https://github.com/omarrx024/xos/blob/master/kernel/blkdev/ahci.asm xOS AHCI implementation (assembly language)]

[[Category:ATA]]
[[Category:Storage]]
````

<a id="w002677"></a>

## W002677 — Template:Outdated

Bron-ID: W002677
Namespace: Template (10)
Revisie: 30749 | Datum: 2026-06-14T21:53:09Z
Bron: https://wiki.osdev.org/Template:Outdated
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AOutdated&oldid=30749

````text
<center>
{|style="background: #ffffdd"
| 
<font color="black">This article contains [[:Category:Outdated Pages|outdated]] information. You can help the wiki by [{{fullurl:{{FULLPAGENAME}}|action=edit}} updating the article]</font>
<br /><small>Please see the relevant discussion on the {{DiscussThis|talk page}}.</small>
|}
<includeonly>[[Category:Disputed Pages]]</includeonly>
</center>
````

<a id="w002678"></a>

## W002678 — Emulator Comparison

Bron-ID: W002678
Namespace: Main (0)
Revisie: 25140 | Datum: 2020-09-08T02:50:42Z
Bron: https://wiki.osdev.org/Emulator_Comparison
Revisie-URL: https://wiki.osdev.org/index.php?title=Emulator_Comparison&oldid=25140

````text
This is a comparison of the various emulators that you can use to [[Testing|test your operating system]] without having to reboot your computer or risk your hardware.

{{In Progress}}

== General ==

{| {{wikitable}}
|-
!
! Cost/License
! Method
! Debugging
! Configuration
|-
! [[Bochs]]
| Free / LGPL
| Full emulation (slow)
| {{Yes}}, built-in
| Command line, script file, interactive menus
|-
! [[QEMU]]
| Free / GPL
| Emulation/dynamic translation
| {{Yes}}, via [[GDB]] stub 
| Command line (optional GUI)
|-
! [[VirtualBox]]
| Free / mixed
| Virtualization
| {{Yes}}, built-in
| GUI, command line (optional)
|-
! [[Microsoft Virtual PC]]
| Free
| Virtualization (on PC), Emulation (on Mac)
| {{No}}
| GUI, command line (optional)
|-
! [[VMWare Virtual Server 2]]
| Free
| Virtualization
| {{Yes}}, via [[GDB]] stub 
| Web interface, non-free Windows client (VI3)
|-
! [[Microsoft Hyper-V]]
| Free
| Virtualization, Emulation on legacy devices
| {{Yes}}, via WinDBG 
| GUI, command line (PowerShell)
|}

Overall, VirtualBox offers the richest set of features, along with very fast performance.  Bochs is by far the slowest, but that is because of its full emulation, which gives it the highest accuracy.

None of them are necessarily "better" than the others.  This comparison is just to point out their differences.  It can't hurt to use more than one emulator (or several), in order to test your OS on a variety of platforms without using real hardware.

== Supported Host Platforms ==

{| {{wikitable}}
|-
!
! Windows
! Linux (x86)
! Mac OS X
! Others
|-
! [[Bochs]]
| {{Yes}} (binaries)
| {{Yes}} (binaries)
| {{Yes}} (must compile source code)
| Others (by source code)
|-
! [[QEMU]]
| {{Yes}}
| {{Yes}}
| {{Yes}}
| PowerPC and others (by source code)
|-
! [[VirtualBox]]
| {{Yes}}
| {{Yes}}
| {{Yes}}
| Solaris
|-
! [[Microsoft Virtual PC]]
| {{Yes}}*
| {{No}}
| {{Maybe}} <small>(yes for PowerPCs, no for Intel Macs)</small>
| <small>* requires AMD-VT or Intel VM support</small>
|-
! [[VMWare Virtual Server 2]]
| {{Yes}}
| {{Yes}}
| {{No}}
| {{No}}
|-
! [[Microsoft Hyper-V]]
| {{Yes}}
| {{No}}
| {{No}}
| {{No}}
|}


== Supported Guest Systems ==

{| {{wikitable}}
|-
!
! x86-32
! x86-64
! Others
|-
! [[Bochs]]
| {{Yes}}
| {{Yes}}
| {{No}}
|-
! [[QEMU]]
| {{Yes}}
| {{Yes}}
| {{Yes}}: ARM, SPARC, MIPS, MIPS64, m68k, PowerPC
|-
! [[VirtualBox]]
| {{Yes}}
| {{Yes}}
| {{No}}
|-
! [[Microsoft Virtual PC]]
| {{Yes}}
| {{No}}
| {{No}}
|-
! [[VMWare Virtual Server 2]]
| {{Yes}}
| {{Yes}}
| {{No}}
|-
! [[Microsoft Hyper-V]]
| {{Yes}}
| {{Yes}}
| {{No}}
|}

== Supported Hardware ==
{| {{wikitable}}
!
! [[SMP]]
! Graphics support
! Sound
! Network
! USB
|-
! [[Bochs]]
| {{Yes}} (up to 8)
| [[Getting_VBE_Mode_Info|VBE]], [[VGA Hardware|VGA]] (partial), [[BGA]], Cirrus Logic GD54xx
| SB-16
| NE2000
| {{Yes}}
|-
! [[QEMU]]
| {{Yes}}
| [[Getting_VBE_Mode_Info|VBE]], [[VGA Hardware|VGA]] (partial), Cirrus Logic GD54xx, (BGA?)
| SB-16, ES1370
| RealTek 8139C
| {{Yes}}
|-
! [[VirtualBox]]
| {{Yes}}
| [[Getting_VBE_Mode_Info|VBE]], OpenGL virtualization, [[VGA Hardware|VGA]] (decent), BGA, VBoxVideo
| SB-16 and AC'97
| Several different NICs
| {{Yes}}
|-
! [[Microsoft Virtual PC]]
| {{No}} <small>?</small>
| [[Getting_VBE_Mode_Info|VBE]], [[VGA Hardware|VGA]] (very good), [[User:Combuster/Trio64 Hardware|S3 Trio64V2]]
| SB-16
| DEC 21140
| {{Yes}}
|-
! [[VMWare Virtual Server 2]]
| {{Yes}}
| [[Getting_VBE_Mode_Info|VBE]], VMWare Guest Tools video driver
| {{Yes}}
| {{Yes}}
| {{Yes}}
|-
! [[Microsoft Hyper-V]]
| {{Yes}}
| [[Getting_VBE_Mode_Info|VBE]], ???
| {{Yes}}
| {{Yes}}
| {{No}}
|}

== Supported Disk Image Formats ==

This chart shows the file formats for an emulated hard disk.  The emulators usually support only a flat image for a floppy and an ISO image file for CD-ROMs.

{| {{wikitable}}
!
! Flat
! Concatenated
! Sparse/Stackable
! Journaling
! Growing
! VMWare format
|-
! [[Bochs]] 
| {{Yes}} || {{Yes}} || {{Yes}} || {{Yes}} || {{Yes}} || {{Yes}}
|-
! [[QEMU]]
| {{Yes}} || {{No}} || {{Yes}} || {{No}} || {{No}} || {{Yes}} 
|-
! [[VirtualBox]]
| {{Yes}} || {{No}} || {{No}} || {{No}} || {{Yes}} || {{Yes}}
|-
! [[Microsoft Virtual PC]]
| {{Yes}} || {{No}} || {{Yes}} || {{No}} || {{Yes}} || {{No}}
|}

* A flat image offers no special features and takes up the full amount of disk space that it emulates.
* The concatenated format allows several separate files to emulate one combined hard disk; useful for partitions.
* A sparse image does not store blank space, so you could emulate a 1 GB hard disk, but it would only take up 200 MB of space if it had 800 MB free space.
* Stackable images allow a "base" read-only image.  Any changes are stored in a new image that is layered on top.  Useful if you want to remove any changes to a "good" disk.
* Journaling images keep track of changes made by each session, and they can be undone/redone in order.
* Growing images are similar to sparse images.  They start small when the emulated disk is empty and expand as more data is written.
* Apparently, the VMWare format for disks is popular because several emulators support it.


[[Category:Emulators]]
````

<a id="w002680"></a>

## W002680 — EDI

Bron-ID: W002680
Namespace: Main (0)
Revisie: 8609 | Datum: 2009-07-24T08:45:33Z
Bron: https://wiki.osdev.org/EDI
Revisie-URL: https://wiki.osdev.org/index.php?title=EDI&oldid=8609
Doorverwijzing: Extensible Driver Interface

````text
#REDIRECT [[Extensible_Driver_Interface]]
````

<a id="w002681"></a>

## W002681 — Extensible Driver Interface

Bron-ID: W002681
Namespace: Main (0)
Revisie: 27966 | Datum: 2023-07-09T16:27:40Z
Bron: https://wiki.osdev.org/Extensible_Driver_Interface
Revisie-URL: https://wiki.osdev.org/index.php?title=Extensible_Driver_Interface&oldid=27966

````text
{{Stub}}
The Extensible Driver Interface (EDI) is a driver interface that is intended to be portable between OSes while standardizing as little functionality as possible.

== Introduction ==
In 2006, Eli Gottlieb made EDI after deciding that [[UDI]] standardized too much. It is designed to be small, extensible, and require very little from the system providing the implementation. This is done by using an object oriented model accessed using C functions, allowing drivers and operating systems in almost all languages to use it.

== See Also ==

=== External Links ===
*[https://sourceforge.net/projects/glider-kernel/files/EDI%20Headers%20and%20Docs/ EDI Specification and sample driver]
*[https://www.osnews.com/story/16602/Introducing_the_Extensible_Driver_Interface OSNews article written by the creator of EDI]

=== Threads ===
*[[Topic:637|Project announcement]]
*[[Topic:16592|Discussion thread]]

[[Category:Driver Interfaces]]
[[Category:OS theory]]
````

<a id="w002682"></a>

## W002682 — BDA

Bron-ID: W002682
Namespace: Main (0)
Revisie: 8628 | Datum: 2009-07-26T15:16:45Z
Bron: https://wiki.osdev.org/BDA
Revisie-URL: https://wiki.osdev.org/index.php?title=BDA&oldid=8628
Doorverwijzing: Memory Map (x86)

````text
#REDIRECT [[Memory Map (x86)#BIOS Data Area (BDA)]]
````

<a id="w002683"></a>

## W002683 — Talk:QEMU

Bron-ID: W002683
Namespace: Talk (1)
Revisie: 24092 | Datum: 2019-11-07T05:33:15Z
Bron: https://wiki.osdev.org/Talk:QEMU
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AQEMU&oldid=24092

````text
"Supported VBE modes" : Qemu also supports arbitrary resolutions, via writing x-resolution, y-resolution (and maybe color depth ?) to some I/O ports. I have seen some stuff about this on the forum. Also works for bochs, I heard. [[User:Js|Js]] 02:03, 31 July 2009 (UTC)

== Tone warning ==

I've made significant edits to improve the quality of this page. I've removed the tone warning from the top. If anyone disagrees with this, please let me know and I'll be happy to accept any suggestions to improve the quality of the article.
````

<a id="w002686"></a>

## W002686 — User talk:Computafreak

Bron-ID: W002686
Namespace: User talk (3)
Revisie: 8774 | Datum: 2009-08-12T07:00:19Z
Bron: https://wiki.osdev.org/User_talk:Computafreak
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AComputafreak&oldid=8774

````text
== APM? ==
Last time I checked, VPC and APM didn't mix. At all. Anyone else able to confirm this? --[[User:Troy martin|Troy Martin]] 03:40, 12 August 2009 (UTC)

I've not checked, but MS seem to think that it supports APM; there's also been a forum post [http://forum.osdev.org/viewtopic.php?p=87560#p87560 here] which seems to suggest that it works --[[User:Computafreak|Computafreak]] 07:00, 12 August 2009 (UTC)
````

<a id="w002688"></a>

## W002688 — User:Creature/QueueMutex

Bron-ID: W002688
Namespace: User (2)
Revisie: 9422 | Datum: 2009-12-18T13:26:32Z
Bron: https://wiki.osdev.org/User:Creature/QueueMutex
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACreature%2FQueueMutex&oldid=9422

````text
When first starting out with [[Synchronization Primitives]], the first sort I implemented was the [[Spinlock]]. Even though they are useful, they don't know a lot about fairness.

== The QueueMutex ==
=== The Problem ===
Suppose you have this piece of code in a priority-based multitasking environment:

<source lang="c">

/*
 * 'Foo' is a function run by multiple threads or processes and executes some operations that need
 * to be thread-safe.
 */
void Foo()
{
   /* 'Spinlock' is some structure or class implementing a spinlock (using atomic operations). */
   static Spinlock lock;

   while(some_condition_that_isnt_met_for_a_long_time) //For example, an infinite loop.
   {
      lock.Lock(); //Waits infinitely for the spinlock to be locked.

      thread_safe_instruction();

      lock.Unlock(); //Unlocks the spinlock so other threads/tasks can use the resource.
   }
}

</source>

Suppose two threads start in this function (let's call them Thread #1 and Thread #2). It is possible that Thread #1 is first to lock the spinlock and it executes 'thread_safe_instruction()'. Thread #2 tries 'lock.Lock()' and waits until Thread #1 is done. Suppose your priority-based multitasking says that each of these threads get an approximate 10 ms before threads are switched. It is possible that Thread #1 has a lot of its time-slice left and, if the condition is not yet met, is able to lock the spinlock again before Thread #2 becomes active and is able to see that the spinlock has been unlocked and lock it itself. This can result in situations where e.g. Thread #1 executes the loop 50 times after which Thread #2 is able to lock the spinlock before Thread #1 does again, after which Thread #2 hogs the loop again before Thread #1 can access the resource.

Suppose there is a function 'myId()' that returns a number unique to each thread. If 'thread_safe_instruction()' would output something like

  Thread ID 'myId()'\n

The above situation could result in an output similar like the following:

  Thread ID 1
  Thread ID 1
  Thread ID 1
  Thread ID 1
  Thread ID 1
  Thread ID 1
  ...
  Thread ID 2
  Thread ID 2
  Thread ID 2
  Thread ID 2
  Thread ID 2
  ...
  Thread ID 1
  ...

A common solution is to use something like a semaphore or another synchronization primitive that implements some sort of queue or priority-based way of making access to the resource more fair for everyone.

=== About ===
I've recently 'discovered' (I put it between apostrophes because it's very probable that it's been discovered and implemented before, nonetheless, it's still handy and I've never seen it before) something I like to call the 'QueueMutex' (it might already exist under other names).

The point of the QueueMutex is to, like semaphores and other SP's (synchronization primitives), provide a more fair way of giving threads or processes access to a resource that needs to be thread-safe. 

=== Implementation ===
The QueueMutex is basically a bastardized spinlock. All that needs to be added is some kind of Queue/Vector object or some kind of dynamic array (it can also be a static array). Here's an example of an implementation in a C++ class:

<source lang="cpp">
class QueueMutex
{
protected:
	volatile byte _Lock; // Used as a normal spinlock.

	Vector<volatile const Thread *> _WaitQueue; // The queue, this could also be a static array, a queue, or another storage container you prefer.

public:
	QueueMutex(): _Lock(0)
	{
		
	}

	void Lock()
	{
		/* This part utilizes a normal spinlock. */
		while(__sync_lock_test_and_set(&this->_Lock, 1) != 0); // Lock the spinlock, dynamic memory access without thread-safety is dangerous!

		/* Add a pointer to the currently active thread (or, the thread executing the Lock, */
		/* if you will) to the back of vector (or queue, static array, or whatever sort of storage you are using). */
		_WaitQueue.PushBack(thisthread); //  
		__sync_lock_release(&this->_Lock); // Release the normal spinlock.

		/* Infinitely loop until the calling thread becomes the one that is the first in line. */
		while(_WaitQueue[0] != thisthread) ;
	}

	void Unlock()
	{
		this->_WaitQueue.PopFront(); // I'm done with the resource, pop me from the front of the queue.
	}
};
</source>

Note: All the '__sync_' functions are GCC-specific, I used them here for simplicity. Each and every one of them is simply an atomic operation. 

'''Note that I have tested the implementation a couple of times but it's very possible that it's still a buggy implementation.'''

When using this bastardized spinlock on the problematic situation, you should get more fair results. When testing this, my result was the following:

  // Thread #1 outputs a couple of times before they start taking turns. This is because Thread #2 was started after
  // Thread #2 which resulted in Thread #1 performing a couple of loops before Thread #2 was ready to start looping.
  Thread ID 1
  Thread ID 1
  Thread ID 1
  Thread ID 1
  Thread ID 1
  Thread ID 1
  ...
  
  // As soon as both threads are properly running, they will take turns, making access to the resource more fair.
  Thread ID 1
  Thread ID 2
  Thread ID 1
  Thread ID 2
  Thread ID 1
  Thread ID 2
  ...

=== Advantages ===
* There is more fairness when implementing thread-safe functions.
* Every accessor of the resource gets queued up. So if 2 'elements' are waiting for the resource to be released, a third one can't just barge in and get access to the resource before the other 2 do; it will have to wait.

=== Disadvantages ===
* If using dynamic memory allocation, it can be seen as a problematic overhead.
````

<a id="w002690"></a>

## W002690 — TCC

Bron-ID: W002690
Namespace: Main (0)
Revisie: 30317 | Datum: 2026-03-17T19:03:16Z
Bron: https://wiki.osdev.org/TCC
Revisie-URL: https://wiki.osdev.org/index.php?title=TCC&oldid=30317

````text
{{In Progress}}

TCC (Tiny C Compiler) is a single-pass, small and fast [[C]] compiler, which can produce x86, x86_64, ARM and RISC-V code, and generate [[PE]], [[ELF]] and [[Mach-O]] executables. 
TCC is heading toward full ISOC99 compliance, and is able to compile itself.

TCC includes also a linker (supporting a subset of GNU's [[LD|ld]] [[Linker Script|linker script]]) and an assembler with [[GAS]]-like syntax (only x86). But this assembler is limited: no
16/64-bit support, instructions up to [[MMX]] are supported, with no [[SSE]] support.

This article describes how to make a simple ELF kernel with [[FASM]] and TCC.
It is also possible to use [[NASM]] ([[Bare Bones with NASM]]).

Note: The Windows version of TCC doesn't produce [[ELF]] executables, but only [[Object Files|object files]]. You
need to recompile TCC without [[PE]] support, if you want to use this tutorial on Windows. You can
skip this step if you aren't using Windows.


==Building TCC with ELF support==
===Windows===
1. Download [http://download.savannah.gnu.org/releases/tinycc/tcc-0.9.26.tar.bz2 TCC sources] and [http://download.savannah.gnu.org/releases/tinycc/tcc-0.9.26-win32-bin.zip 32-bit TCC] and (if you have a 64-bit OS) [http://download.savannah.gnu.org/releases/tinycc/tcc-0.9.26-win64-bin.zip 64-bit TCC].

2. Extract the source folder tcc-0.9.26.

3. Save the 32-bit TCC files to a folder called "win32" in the location containing tcc-0.9.26. If you have a 64-bit OS, also create a win64 folder, and save the 64-bit files to a folder called "win64" in the same location.

4. Open Notepad or another text editor, and enter this:

<syntaxhighlight lang="winbatch">
@echo off

set \p VERSION = < .VERSION
echo > config.h #define TCC_VERSION "%VERSION%"

set targetP=I386
set B=32
goto begin

:x86_64
set targetP=X86_64
set B=64
goto begin

:begin
set targetF=PE
set CC=..\win%B%\tcc.exe -O0 -s -fno-strict-aliasing
set P=%B%

:start
if %targetF%==ELF set P=%B%-elf
set target=-DTCC_TARGET_%targetF% -DTCC_TARGET_%targetP%

:tools
%CC% %target% win%P%\tools\tiny_impdef.c -o win%P%\tiny_impdef.exe
%CC% %target% win%P%\tools\tiny_libmaker.c -o win%P%\tiny_libmaker.exe

:libtcc
if not exist win%P%\libtcc\nul mkdir win%P%\libtcc
copy libtcc.h win%P%\libtcc\libtcc.h
%CC% %target% -shared -DLIBTCC_AS_DLL -DONE_SOURCE libtcc.c -o win%P%\libtcc.dll
win%P%\tiny_impdef win%P%\libtcc.dll -o win%P%\libtcc\libtcc.def

:tcc
%CC% %target% tcc.c -o win%P%\tcc.exe -ltcc -Lwin%P%\libtcc

:copy_std_includes
copy include\*.h win%P%\include

:libtcc1.a
win%B%\tcc %target% -c lib\libtcc1.c -o win%P%\libtcc1.o
win%B%\tcc %target% -c win%P%\lib\crt1.c -o win%P%\crt1.o
win%B%\tcc %target% -c win%P%\lib\wincrt1.c -o win%P%\wincrt1.o
win%B%\tcc %target% -c win%P%\lib\dllcrt1.c -o win%P%\dllcrt1.o
win%B%\tcc %target% -c win%P%\lib\dllmain.c -o win%P%\dllmain.o
win%B%\tcc %target% -c win%P%\lib\chkstk.S -o win%P%\chkstk.o
goto lib%B%

:lib32
win%B%\tcc %target% -c lib\alloca86.S -o win%P%\alloca86.o
win%B%\tcc %target% -c lib\alloca86-bt.S -o win%P%\alloca86-bt.o
win%B%\tcc %target% -c lib\bcheck.c -o win%P%\bcheck.o
win%P%\tiny_libmaker win%P%\lib\libtcc1.a win%P%\libtcc1.o win%P%\alloca86.o win%P%\alloca86-bt.o win%P%\crt1.o win%P%\wincrt1.o win%P%\dllcrt1.o win%P%\dllmain.o win%P%\chkstk.o win%P%\bcheck.o
@goto the_end

:lib64
win%P%\tcc %target% -c lib\alloca86_64.S -o win%P%\alloca86_64.o
win%P%\tiny_libmaker win%P%\lib\libtcc1.a win%P%\libtcc1.o win%P%\alloca86_64.o win%P%\crt1.o win%P%\wincrt1.o win%P%\dllcrt1.o win%P%\dllmain.o win%P%\chkstk.o

:the_end
del win%P%\*.o

@if %targetF%==PE (
	@set targetF=ELF
	@goto start
)

@if %B%==64 goto finished
@if _%PROCESSOR_ARCHITEW6432%_==_AMD64_ goto x86_64
@if _%PROCESSOR_ARCHITECTURE%_==_AMD64_ goto x86_64

:finished
</syntaxhighlight>

5. Save to tcc-0.9.26 with any name you want, but the extension MUST be .bat. If you use notepad, you will have to change the type from "Text Documents" to "All Files".

6. Run the script, make sure everything compiled correctly (note that there may be warnings about assignments from incompatible pointer types and bound checking not supporting malloc in a certain environment, but these are okay), and inside win32-elf you should have a working ELF compiler. You should also have win32 and win64 for 32-bit and 64-bit PE compilers, and a 64-bit ELF compiler in win64-elf. Note: The 64-bit compilers won't be compiled on a non-64 bit OS.
If there are errors compiling TCC, just change the "@echo off" to "@echo on" and run the script again to see where things went wrong.

===Linux===
Many distributions provide packages for TCC. If not, then it has to be built from source.

==A small kernel example==

This little example builds a small kernel in ELF format, which can be booted by [[GRUB]].

===start32.asm===
<syntaxhighlight lang="asm">
;  Tutorial: A small kernel with Fasm & TCC
;  By Tommy.
 
        format elf
        use32

;
; Equates
;
MULTIBOOT_PAGE_ALIGN	    equ (1 shl 0)
MULTIBOOT_MEMORY_INFO	    equ (1 shl 1)
MULTIBOOT_AOUT_KLUDGE	    equ (1 shl 16)
MULTIBOOT_HEADER_MAGIC	    equ 0x1BADB002
MULTIBOOT_HEADER_FLAGS	    equ MULTIBOOT_PAGE_ALIGN or MULTIBOOT_MEMORY_INFO
MULTIBOOT_CHECKSUM          equ -(MULTIBOOT_HEADER_MAGIC + MULTIBOOT_HEADER_FLAGS)

        section '.text' executable
;
; Multiboot header
;
dd MULTIBOOT_HEADER_MAGIC
dd MULTIBOOT_HEADER_FLAGS
dd MULTIBOOT_CHECKSUM


;
; Kernel entry point.
;
        public  _start
        extrn   kmain
_start:
        ; Call the main kernel function.
        call    kmain
 
@@:
        jmp     @b

</syntaxhighlight>

===kernel.c===
<syntaxhighlight lang="c">
/*  Tutorial: A small kernel with Fasm & TCC
 *  By Tommy.
 */

/*
 * Main kernel function.
 */
void
kmain (void)
{
    *((unsigned char *) 0xB8000) = 'H';
    *((unsigned char *) 0xB8001) = 0x1F;
    *((unsigned char *) 0xB8002) = 'E';
    *((unsigned char *) 0xB8003) = 0x1F;
    *((unsigned char *) 0xB8004) = 'L';
    *((unsigned char *) 0xB8005) = 0x1F;
    *((unsigned char *) 0xB8006) = 'L';
    *((unsigned char *) 0xB8007) = 0x1F;
    *((unsigned char *) 0xB8008) = 'O';
    *((unsigned char *) 0xB8009) = 0x1F;
}

</syntaxhighlight>

==Compiling and linking==
Assemble start32.asm with:
<syntaxhighlight lang="bash">
fasm start32.asm
</syntaxhighlight>

Compile kernel.c with:
<syntaxhighlight lang="bash">
tcc -c kernel.c
</syntaxhighlight>

Then link the whole thing with:
<syntaxhighlight lang="bash">
tcc -nostdlib -Wl,-Ttext,0x100000 start32.o kernel.o -o kernel-i386.elf
</syntaxhighlight>

If you would prefer it in binary form, for example, if you're using your own bootloader that doesn't support ELF, link it with this:
<syntaxhighlight lang="bash">
tcc -nostdlib -Wl,-Ttext,0x100000 -Wl,--oformat,binary -static start32.o kernel.o -o kernel-i386.bin
</syntaxhighlight>

That's all!

==Inline Assembly==
TCC supports inline [[GAS]] syntax assembly like [[GCC]]:
<syntaxhighlight lang="c">
__asm__ __volatile__("hlt");
</syntaxhighlight>
You can use this to your benefit for many things, such as debugging in [[Bochs]]:
<syntaxhighlight lang="c">
#define breakpoint() __asm__ __volatile__("xchg %bx, %bx");

void bochs_print (char *string) {
	char *c = string;
	while (*c != '\0') {
		outb(0xE9, *c); // may be outportb
		c++;
	}
}
</syntaxhighlight>
Then adding this to your bochsrc.bxrc file in a text editor:
<syntaxhighlight lang="text">
port_e9_hack: enabled=1
magic_break: enabled=1
</syntaxhighlight>
And from [[Bochs]] install location, executing bochsdbg.exe instead of bochs.exe.

==GDT and struct warning==
According to Fabrice Bellard, the creator of TCC, and (painfully) tested to be true: "In TCC 'packed' is supported only for structure fields or variable declarations, not for a whole structure. So a solution for you is to add it to each field of the packed structure."
So if you use structs to store your [[GDT]] entries or [[GDT#GDTR|GDTR]], beware, you will encounter issues loading your GDT if you don't specify the packing of structures correctly.
When creating structures, use something like this:
<syntaxhighlight lang="c">
// We use the attribute 'packed' to tell TCC not to change any of the alignment in the structure.
struct some_struct {
   unsigned char a __attribute__((packed));
   unsigned char b __attribute__((packed));
} __attribute__((packed));
// This last attribute can be kept, it won't interfere with the compilation or output, so it may be
// useful to retain compatibility with GCC, as long as the above attributes don't interfere with GCC.
</syntaxhighlight>
Instead of this:
<syntaxhighlight lang="c">
// We use the attribute 'packed' to tell GCC not to change any of the alignment in the structure.
struct some_struct {
   unsigned char a;
   unsigned char b;
} __attribute__((packed));
</syntaxhighlight>

==Inline Function Warning==
TCC doesn't support function inlining, because the 'inline' keyword is ignored, so if a function needs to be inlined, you must use defines instead.

==Note on stdint.h==
TCC doesn't include stdint.h, but all typedefs required are provided in stddef.h.
To use stdint.h place the following code in your kernels include path as stdint.h. This will make your code compatible with both [[GCC]] and TCC.
<syntaxhighlight lang="c">
/* stdint.h */

#ifdef __TINYC__
/* tcc */
#include <stddef.h>
#else
/* assume gcc */
#include_next <stdint.h>
#endif
</syntaxhighlight>

==See Also==
*[[Smaller C]] - Another simple single-pass compiler. Unlike TCC it can output 16-bit code (that also works on [[Unreal Mode]]).

===External Links===
*[https://bellard.org/tcc/ TCC project homepage]
*[https://repo.or.cz/w/tinycc.git TCC source code repo]
*[https://github.com/TinyCC/tinycc Github mirror of the above repo]

[[Category:C]]
[[Category:Compilers]]
````

<a id="w002691"></a>

## W002691 — Talk:GAS

Bron-ID: W002691
Namespace: Talk (1)
Revisie: 8859 | Datum: 2009-08-25T10:04:29Z
Bron: https://wiki.osdev.org/Talk:GAS
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AGAS&oldid=8859

````text
==In Progess==
I fail to see why this is marked as in progress... it seems to have all the required information to me. Only thing it doesn't have is the GAS syntax, which is covered on the linked pages. [[User:Jackscott|JackScott]] 23:16, 23 August 2009 (UTC)

:+1. I have removed the in progress status - I think it's as complete as it's going to get. --[[User:Aj|Aj]] 10:04, 25 August 2009 (UTC)
````

<a id="w002692"></a>

## W002692 — Virtual Box

Bron-ID: W002692
Namespace: Main (0)
Revisie: 8840 | Datum: 2009-08-23T23:19:27Z
Bron: https://wiki.osdev.org/Virtual_Box
Revisie-URL: https://wiki.osdev.org/index.php?title=Virtual_Box&oldid=8840
Doorverwijzing: VirtualBox

````text
#REDIRECT [[VirtualBox]]
````

<a id="w002693"></a>

## W002693 — Talk:Virtual Box

Bron-ID: W002693
Namespace: Talk (1)
Revisie: 8842 | Datum: 2009-08-23T23:19:27Z
Bron: https://wiki.osdev.org/Talk:Virtual_Box
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AVirtual_Box&oldid=8842
Doorverwijzing: Talk:VirtualBox

````text
#REDIRECT [[Talk:VirtualBox]]
````

<a id="w002695"></a>

## W002695 — Hybrid Kernel

Bron-ID: W002695
Namespace: Main (0)
Revisie: 30585 | Datum: 2026-05-19T18:12:29Z
Bron: https://wiki.osdev.org/Hybrid_Kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Hybrid_Kernel&oldid=30585

````text
{{Template:Kernel designs}}
== Design ==
A hybrid kernel is, as its name indicates, a hybrid between a [[Monolithic kernel]] and a [[Microkernel]]. Unlike a microkernel where everything takes place in user level servers and drivers, the designers of a hybrid kernel may decide to keep several components inside kernel and some outside. There are many motivations for doing so, such as performance, simplicity, and vendor lock-in (you cannot change components with custom components). Most hybrid kernels start as monolithic kernels and begin moving components into user land, primarily as security to support 3rd-party components and drivers which may be malicious or buggy.

An example of a hybrid kernel design may keep the VFS and bus controllers inside the kernel, but have the file system drivers and storage drivers as user mode programs. The advantage of this system that is you keep the performance and design principles of a monolithic kernel, but you allow untrusted users to load untrusted code for accessing their own storage devices.

== Controversy ==

Most of the complex operating systems and their kernels are not purely monolithic or microkernels, which makes the term "hybrid" controversial and difficult to apply (and results in it being applied inconsistently), because of the similarity with monolithic kernel. For example, Linux allows running filesystem and device drivers, as well as compatibility layers such as Wine in userspace, but has not been called a hybrid kernel.

In addition, some kernels have also been called hybrid using different definitions of "hybrid", for example because they used [[Modular Kernel|kernel modules]], borrowed concepts from microkernels (DragonFly BSD architecture and message passing<ref>https://www.dragonflybsd.org/presentations/dragonflybsd.asiabsdcon04.pdf</ref>), were hybrids between two systems rather than architectures (XNU combining Mach kernel with FreeBSD components, all running in kernel), because of a misunderstanding (Haiku), or because of the small kernel size while being closer to monolithic in architecture (Plan 9). This has also been worsened by companies trying to market (and as a result creating more confusion) their operating systems as hybrid because of the good PR of microkernels. 

As a result, most, if not all the popular examples (NT/ReactOS, XNU, BeOS/Haiku, DragonFly BSD, Plan 9) are not clearly hybrid, and following the "monolithic kernel with some components in userspace" definition are arguably not any more hybrid than the monolithic examples (Linux, BSDs, and so on). 

[[Category:Kernel]]
````

<a id="w002696"></a>

## W002696 — Text mode

Bron-ID: W002696
Namespace: Main (0)
Revisie: 8874 | Datum: 2009-08-26T04:24:58Z
Bron: https://wiki.osdev.org/Text_mode
Revisie-URL: https://wiki.osdev.org/index.php?title=Text_mode&oldid=8874
Doorverwijzing: Text UI

````text
#REDIRECT [[Text UI]]
````

<a id="w002697"></a>

## W002697 — Kernel

Bron-ID: W002697
Namespace: Main (0)
Revisie: 11619 | Datum: 2011-06-13T09:17:47Z
Bron: https://wiki.osdev.org/Kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Kernel&oldid=11619

````text
{{Template:Kernel designs}}

:''This article concerns itself with the theory behind '''kernels'''. There also exist several [[:Category:Bare bones tutorials|Bare bones tutorials]] for setting up a small working kernel.''

==Introduction==
The kernel is the central component of an operating system. It is a natural consequence of an operating system's need to manage resources, and often the most important part of a hobby operating system. Its responsibilities include managing memory and devices and also providing an interface for software applications to use those resources. Depending on the architecture of the operating system, the kernel often manages protection of programs and multitasking.

There are a number of kernel models considered in the development of a kernel, each dependent upon personal choice and research into reliability, speed and how easily goals can be reached using the specified method. The two major models are the monolithic kernel and microkernel. The monolithic kernel is based on the idea of the kernel being a single program that sits in high memory and manages the system. At the other end of the scale, the microkernel provides as few as possible kernel mode services and negotiates the rest to user mode components.

==Main Kernel Models==

===Monolithic Kernel===
{{main|Monolithic kernel}}
Monolithic kernels function with all of the kernel and device drivers inside one address space running in kernel mode. The principle advantage to this approach is efficiency (especially on x86 architectures where a task switch is a particularly expensive operation). Unfortunately, due to its single address space, a bug in a device driver can bring down the entire system.

===Microkernel===
{{main|Microkernel}}
A microkernel tries to run most of its services and device drivers in userspace. This can result in an increase in stability and security on machines that have sufficient memory protection. In most current implementations the speed loss is considered too great to make the Microkernel practical as of yet in most commercial systems.

==Subset Kernel Models==

These models can usually be categorized as either of the above and usually mix concepts from both.

===Hybrid Kernel===
{{main|Hybrid Kernel}}
The hybrid kernel, often called the macrokernel, is primarily a monolithic kernel. It differs in that the parts of it are usually segmented either in code, by a few physical modules or a mixture of both. Unlike microkernels, which use a message passing system to communicate with user space modules, hybrid kernels are usually linked together at run time and still exist as a whole in kernel space. The term is usually reserved for kernels whose core functions are modularised, not for systems who have the ability to load drivers as separate kernel modules.

===Modular Kernel===
{{main|Modular Kernel}}
Also a monolithic model, the modular kernel, which is usually far more modular than the hybrid kernel, is based upon the same concept. The major difference is the usual ability for the kernel to load numerous unknown core modules, as opposed to simply loading in known kernel parts. It is also never considered modular because of the use of namespaces, or a simplified codebase.

===Nanokernel===
{{main|Nanokernel}}
Often considered an incredibly minimal microkernel. The nanokernel design does not actually define the method by which parts are combined and is therefore able to sit in either camp. Most nanokernels use a message passing system to communicate with other components in order to make the the system as architecturally independent as possible. It is basically a stripped down kernel without any form of hardware abstraction, so it requires device drivers for each basic resource in the architecture. This includes devices normally controlled by a HAL, such as the Timer, RTC, PIC and DMA Controller.

===Exokernel===
{{main|Exokernel}}
Also a tiny kernel design, the exokernel represents as little abstraction as possible, the complete opposite of the nanokernel. An exokernel is intended to give untrusted programs as much control over the hardware as possible, and in a secure way. This allows programs to take advantage of the details of present hardware instead of having to rely on the kernel to do this, which in some cases can result in drastic performance improvements. This goes however at the cost of having to write more complex software. This is quite often programmed in system libraries which provide the abstraction on the kernel's behalf.

[[de:Kernel]]
[[Category:Kernel]]
````

<a id="w002698"></a>

## W002698 — Task

Bron-ID: W002698
Namespace: Main (0)
Revisie: 8877 | Datum: 2009-08-26T04:29:33Z
Bron: https://wiki.osdev.org/Task
Revisie-URL: https://wiki.osdev.org/index.php?title=Task&oldid=8877
Doorverwijzing: Processes and Threads

````text
#REDIRECT [[Processes and Threads]]
````

<a id="w002699"></a>

## W002699 — CPU Caches

Bron-ID: W002699
Namespace: Main (0)
Revisie: 28242 | Datum: 2023-07-10T23:29:26Z
Bron: https://wiki.osdev.org/CPU_Caches
Revisie-URL: https://wiki.osdev.org/index.php?title=CPU_Caches&oldid=28242

````text
== Introduction ==

Originally, memory was as fast as CPUs. Over time CPUs got faster and faster. Memory got faster too, but didn't keep up with the increasing speed of CPUs, which means that CPUs needed to spend more and more time waiting for data to come from RAM (which can have a serious effect on performance).

As a work-around for the speed difference between CPUs and RAM, CPU manufacturers started including caches. The basic idea is to have an small area of RAM that the CPU can access really fast, that's used to store a copy of data in RAM (to reduce the time the CPU spends waiting to access slower RAM).

Over time the difference in speed between CPUs and RAM got even worse, so CPU manufacturers increased the size of the caches; but (due to limitations in semiconductor design) the larger a cache is the slower it is (or, the higher the latency of the cache is). To improve performance more, CPU manufacturers started having large caches with smaller/faster caches on top.

For a current/modern CPU there can be up to 3 layers of caches - extremely fast but relatively small "layer 1" (or L1) caches close to the CPU, fairly fast medium sized "layer 2" (or L2) caches, then relatively large "layer 3" (or L3) caches close to the system bus or RAM.

Of course the amount of RAM used in computers has grown too; and even a very large L3 cache can't cache everything. For example, a computer might have 4 GiB of RAM but only 12 MiB of L3 cache. To improve performance CPU manufacturers want to cache the RAM that is most likely to be used in future. However, it's extremely difficult to predict the future, so CPU manufacturers usually use a "least recently used" algorithm for caches, as something that was used recently is (usually) likely to be used again.

Fortunately, except for a few cases (testing for RAM faults, RAM bandwidth benchmarks) a programmer doesn't have to worry about the caches. However, caches play an important role in the overall performance of a system, and it is possible for programmers to get significant performance improvements by paying attention to cache efficiency.

In general the "least recently used" algorithm can work quite well, but there are cases where it causes poor efficiency. One example of this is called "cache pollution", where the CPU accesses a lot of data that won't be needed in the future; which causes data that is more likely to be needed soon to be pushed out of the cache to make room for the (more recently used) data that won't be needed soon. Another example is called "cache thrashing", where you repeatedly access an amount of data that is larger than the size of the cache (e.g. repeatedly accessing each item in a 10 MiB array when the CPUs cache is only 8 MiB). In this case, even though a piece of data will be used again it will be pushed out of the cache by more recent data accesses before it's used again.

There's several techniques for managing these cache efficiency problems. One technique CPU manufacturers use is to have separate caches for separate things. For example, a CPU might have an L1 cache for instructions and another L1 cache for data, so that problems with L1 data cache efficiency don't effect L1 instruction cache efficiency. This also helps with the "larger caches have worse latency" problem.

Modern CPUs also have instructions that programmers can use to help avoid "least recently used" cache efficiency problems; including instructions that can be used to prefetch data into the caches (useful when you know that data will be used soon), instructions to explicitly remove/flush data from the cache (useful when you know data that you've used recently won't be used again soon), and instructions to write data directly to RAM and bypass the cache (which is similar to writing data normally and then flushing that data from the cache).

There are situations where software needs to process a large amount of data, but can process this data in blocks (for example, decompression often falls into this category). In these cases, it can be desirable (to avoid "cache thrashing") to detect how large the caches are, and to make sure that each block fits inside the cache. For example, if the CPU's cache is 4 MiB and you want to process 100 MiB of data, then processing 50 blocks that are 2 MiB each can give a lot better performance than processing 20 blocks that are 5 MiB each. For this reason, it can be a good idea for an OS to detect the size of CPU caches and provide a way for software (e.g. applications) to easily obtain this information.

== Caches In Systems With Multiple CPUs ==

When there's multiple CPUs directly connected to memory (without caches), everything works correctly (but slowly). When there's multiple CPUs with their own caches connected to memory, then data stored in RAM can be stale/obsolete data because the current version of that data may still be in one of the CPU's caches. In most systems caches are meant to be transparent (e.g. it's meant to be safe for software to forget that caching is being done), and having caches in each CPU creates problems because it's no longer safe for programmers to forget that caching is being done. The hardware in most computers use special protocols to ensure that these problems don't happen - for example, when a CPU accesses memory it accesses the current version of the data regardless of whether the current version of the data is in RAM or in another CPU's cache. This is called "cache coherency".

Some computer systems are not cache coherent; and it's the programmers responsibility to avoid accessing stale data. In general these systems can be a nightmare to program for, and it can be easier to treat each CPU like a separate system (e.g. pretend that a computer with 2 GiB of RAM and 2 CPUs is actually 2 separate computers with 1 GiB of RAM and 1 CPU each). Fortunately non-cache coherent systems are rare, especially for desktop/server computers.

For systems with multiple CPUs (especially for multi-core), it's possible for several CPUs to share the same caches. For example, for a computer with a pair of quad core CPUs you might have a total of 8 CPUs, where each CPU has it's own L1 caches, pairs of CPUs share L2 caches, and 4 CPUs share each L3 cache. This can complicate "cache efficiency" optimizations for operating system software (e.g. in schedulers and memory management) and applications.

== Cache Organization ==

Normally a cache is split up into many cache lines, where a cache line is the minimum amount of data that can be stored in the cache. For example, a 4 MiB cache might have 65536 entries to hold up to 65536 cache lines, where each cache line in 64 bytes. When data is transferred between the cache and RAM (or between the cache at one level and another cache at a different level) all 64 bytes are transferred. For a given cache line size, a larger cache has more entries.

The CPU needs to be able to find an entry in a cache that corresponds to a specific address. To do this each entry has a "tag", which says what is in that entry (e.g. if the entry is being used or not, and if it is being used which address the data corresponds to). For a very simple cache, this means that the CPU needs to look at every single tag just to find some data in the cache. This has obvious problems (it's slow).

To make the cache faster, CPU manufacturers tend to use sets of cache lines, where the data for a specific address can only be in one place in each set. The number of sets is called "associativity", and a cache designed like this is called an "N-way set associative cache". For example, a 4 MiB cache that is 4-way set associative would have 1 MiB per set, the data for a specific address can only be in one of 4 places, and the CPU only needs to look at 4 tags to find some data in the cache.

To work out which tags to look at in each set of cache lines, the CPU uses some of the bits of the address. For example, for the address 0x12345678, with 64 bytes per cache line the lowest 6 bits don't matter, and (if each set is 1 MiB) the next 14 bits determine the position in each set that the data could be. This also means that for a 32-bit address the tag only needs to store the highest 12 bits of the address (which helps to make the tag smaller).

A cache where the data for any address can be anywhere in the cache (the simple/slower cache mentioned above) is called "fully associative", which is effectively the same as having infinite associativity. A cache where the data for any address can only be in one place in the cache is called "direct mapped", which is effectively the same as "1-way associative".

For a direct mapped cache and for caches with low associativity, information in the cache can be found faster because there's less places in the cache that it could be; but because there's a limited number of places it also increases the chance of cache misses. For example, with 1 MiB direct mapped cache, data from the addresses 0x00000000, 0x00100000, 0x00200000, 0x00300000, etc must share the same place in the cache; and data from 0x00000000 will cause data from 0x00100000 to be evicted from the cache, even if other entries in the cache are unused, and even if the data from 0x00100000 is used frequently. This can cause excessive cache misses. Therefore the design of a cache is a compromise between low associativity (fast lookup times) and high associativity (reduced cache misses).

For most modern CPUs, caches are either 2-way associative, 4-way associative or 8-way associative. For more associativity, the overhead of finding entries in the cache outweigh the benefits of slightly less chance of cache misses.

== Optimizing Cache Efficiency ==

For modern computers, CPUs are very fast and RAM is only "fast". Caches help performance a lot if software uses the cache efficiently, but if software doesn't use the caches efficiently then caches won't help performance much. There are several techniques that can be used when trying to use caches more efficiently.

=== Cache Blocking ===

The first technique is already mentioned in the introduction - if you can, split large amounts of data up into smaller blocks that fit inside the cache to avoid cache thrashing. For this to work you need to know how large the cache/s are and how many CPUs are sharing the cache/s.

=== Page Colouring ===

The next technique is called "page colouring" or "cache colouring". Systems that use paging can choose pages to minimize the chance of cache misses caused by low associativity. Consider a 4 MiB 4-way associative cache, where a process is extremely unlucky and every page it's using happens to correspond to the same locations in each set in the cache. In this case the process would only be able to use 16 KiB of the cache and the remaining 99.6% of the cache would be wasted. Of course this is an extremely unlikely worst case used for illustration purposes - in practice it's likely that this problem will only cause a small decrease in cache efficiency. The point is that without page colouring an OS relies on probability and hopes that cache efficiency isn't too badly effected. Also note that this can cause non-deterministic performance (e.g. how fast software runs depends on luck), which may be undesirable (especially in real time systems).

The idea behind page colouring is to make sure that pages are allocated in a way that minimizes the chance of cache efficiency being effected. To do this an OS determines which bits in an physical address effect where the cache lines will go in each set in the cache; and then makes sure that these bits in the virtual address of a page is the same as the corresponding bits in the physical address of the page. However, this can create an extra problem: typically some virtual addresses are used more often than others (for example, an OS might load all processes at the virtual address 0x00001000); which means that an OS might run out of physical pages that match the more frequently used virtual page addresses. To avoid this extra problem an OS can add an offset a process' virtual addresses before deciding which physical pages are suitable, and make this offset different for different processes.

For page colouring to work you need to know the size of each set in the cache. This can be determined by dividing the total size of the cache by the cache's associativity.

=== Preventing Unnecessary Caching ===

In some situations software might access a lot of data, and the programmer knows that this data either won't be used again or won't be used again soon. In these cases you can improve cache efficiency by explicitly flushing cache lines (for example, the 80x86 "CLFLUSH" instruction) to free entries in the cache for more important data.

Some CPUs allow caching to be disabled on a per page basis. This feature can be used to avoid the need for software to explicitly flush cache lines in some situations.

=== Prefetching ===

In some situations it's easy to predict which cache lines will be needed soon. In these cases, some CPUs allow programmers to explicitly request that the data is loaded into the cache (prefetched) to avoid cache misses.

Also, some CPUs will detect certain access patterns (e.g. accesses to sequentially increasing and sequentially decreasing addresses), and will automatically prefetch cache lines.

=== Scheduling With Multiple CPUs ===

For computers with more than one CPU, when a task runs it's code and data will end up in the cache/s of the CPU that it was running on. The next time that the scheduler gives the task CPU time it's possible for some of the task's code and data to still be in a CPU's cache, and some cache misses can be avoided by making the task run on the same CPU that it was running on last time. It isn't a good idea to always do this because an OS's scheduler may also take into account many other things (for performance, power management and other reasons) - it's better to think of this optimization as a scheduling hint.

For more complex systems with several layers of caching, where different caches are shared by different CPUs; if the scheduler can't schedule the task on the CPU that the task used last time then a CPU that shares cache/s with the previously used CPU may be more preferable than other CPUs that don't share cache/s with the previously used CPU.

[[Category:Physical Memory]]
[[Category:Memory management]]
````

<a id="w002700"></a>

## W002700 — User:Aj

Bron-ID: W002700
Namespace: User (2)
Revisie: 16620 | Datum: 2014-08-20T20:33:42Z
Bron: https://wiki.osdev.org/User:Aj
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAj&oldid=16620

````text
== My Background ==
I am a purely hobby-programer, who started out back on a BBC B Micro back in the 1980's at the age of 5 (using BASIC from a simple starter guide). I worked my way through various PC's from the 286 and DOS 5.0 upwards.

Once I got interested in programming PC's instead of the old BBC, I started with QBASIC, VB 1 (for DOS) and worked my way through all the Windows Versions from 3.1 with VB3, to Windows 98 with VB5.

My language of choice took a turn towards the 'C'-style languages in about 2002, and C# with XAML is now my Windows language of choice. I am relatively proficient in x86 Assembly, having written several test boot environments and my OS Dev language of choice is C++, in which I consider myself pretty good (always room for improvement!).

When I get time, I go back to my hobby project, CBoot / Caracal, but time is becoming increasingly rare.

== My Main Wiki Contributions ==

I have either written the followng pages from scratch, or have replaced stub pages for the following articles:

* [[El-Torito]]: Writing a boot loader for no-emulation El-Torito Compliant CD's.
* [[ISO 9660]]: Accessing files on an ISO 9660 format CD.
* "Building GMP": GMP and MPFR became dependencies of [[GCC]] with version 4.3.0. Later, the article was split in to "Building GMP" and "Building MPFR".

(And later still, removed in favor of using the system package manager, but good work nevertheless.) -- [[User:Solar|Solar]] 08:13, 6 May 2010 (UTC)

I have helped out with or added to the following articles:

* [[GCC Cross-Compiler]]: A personal favorite! Although I didn't write any of the original article (I believe thanks goes to Solar for that one), I often check the tutorial against new GCC / Binutils releases and addedd [[Building GMP]] and [[Building MPFR]] when they became dependencies. I also added information on compiling libgcc with your configured build path.
* [[GCC Cross-Compiler for x86 64‎]]: As with [[GCC Cross-Compiler]], I occasionally check the build process with new GCC versions.

My other contributions: [[Special:Contributions/Aj]]
````

<a id="w002701"></a>

## W002701 — Kernel Stdio Theory

Bron-ID: W002701
Namespace: Main (0)
Revisie: 28519 | Datum: 2023-12-13T08:05:24Z
Bron: https://wiki.osdev.org/Kernel_Stdio_Theory
Revisie-URL: https://wiki.osdev.org/index.php?title=Kernel_Stdio_Theory&oldid=28519

````text
==Standard Input/Output Theory==

===What is Standard Input/Output?===

Standard Input and Output, and Standard Error, are streams which are implemented as part of the [[C]] Standard Library. A stream is a read/write interface for access to a file, hardware resource, or another process.

On inclusion of the <tt>stdio.h</tt> header, three streams are auto created, and associated with the environment's Standard Input, standard Output and standard Error streams. It is conventional, and default in most cases, for the process's Standard Output and Standard Error to be tied to the terminal which opened it. And unless StdIn is redirected, the default source of Standard Input is taken to be the keyboard.

These effects are handled by the C Library, and they interface with an underlying OS to provide access to the stream resource. Stream resources may have the following properties:

*Read/Write
*Text/Binary
*Buffered/Unbuffered

In most cases, by default, StdOut is buffered, while StdErr is unbuffered. This allows for data output on StdErr to be seen by the user immediately. Buffering may be either ''line'' buffered or ''fully'' buffered. The C Standard Library's intricacies are not the focus of this article, however.

A kernel must provide the underlying devices' API and present them to a running application. All applications, for purposes of this writing, are by default bound to Keyboard for StdIn, and the Console for StdOut.

Since C was used to develop the Unix kernel, and Unix provided abstraction of devices as files, which were no different from any other, a process could easily open a device as a resource stream, and connect its StdIO to the device for reading or writing. 

A classic example of some basic know-how of how the Unix's StdIO works is the terminal itself. Note well that provision of such abstraction does not need to be done via files. The Windows API provides abstraction via its API function calls. Programs may access StdIn, StdOut, and StdErr, but these are really only relevant when working in console mode. For GUI applications, StdIn, StdOut, and StdErr are substituted by the application developer with relevant API functionality. E.g: Alert Boxes for errors and warnings, etc.

In the Unix kernel, the default Standard Output is connected to the user's terminal device. (e.g: /dev/tty0). Try going into your Unix/Linux command prompt and typing <tt>who am i</tt>. This is the device that represents your current terminal's standard Output. On microcomputers (the one you're using right now) there are no terminals connected to huge mainframes, so the modern Linux/Unix microcomputers just create a terminal device anyway, and associate it with the Kernel's Standard Output.

Now: Try <tt>echo Hello from Std Output! >file</tt>

This prints the string "Hello from Std Output" to a file called 'file', by redirecting the Standard Output of the program to the file. The program is given the handle of the file for its stdout device, and it writes, for all intents and purposes, to the file.

Go back and find out which terminal you are, by running either <tt>who am i</tt> or simply typing <tt>tty</tt>. Take the output from <tt>tty</tt> and place it at the end of the line, where 'file' was last time. My kernel reports my terminal as /dev/pts/1.

Now try: <tt>echo Hello from Std Output! >/dev/pts/1</tt>

This time, you see the out put from 'echo' on the terminal. Why? You explicitly redirected the output of Stdout to your Stdout. /dev/pts/1 in my case, and whatever you got in yours, is synonymous with the standard output for the current user.

Interestingly, though, this file ''also'' acts as the Stdin device: reading from it causes all the characters you type at the keyboard to be reported to the program currently reading from it.

The unix <tt>cat</tt> command reads and outputs what it sees in a file. When you type at the keyboard, this is directed to Stdin, which applications may read from. To get a better understanding more quickly, try:

<tt>cat /dev/pts/1</tt>

Now keep typing words and characters, and then press enter when you feel like it. The characters you typed at the terminal are sent to Stdin, which <tt>cat</tt> is reading from. When you press 'enter', the kernel stops reading from the keyboard, having reached the end of line for the current bit of input, and sends this line of keyboard input to the Standard Input, which, currently, <tt>cat</tt> is reading from. Cat, thinking that it is reading from an ordinary file, does what it's supposed to do: echo the contents of the file onto the terminal.

When you're done typing lines, press ''Ctl+d'', which is a synonym in *nix for "No more input/End of input". When dealing with files, this is synonymous for "End of file". The kernel receives the indication that there is no more input, and forwards this to the Stdin, which gives it to <tt>cat</tt>. <tt>cat</tt> interprets the "no more data" signal to mean that the end of the file it is currently reading has been reached. Just like any other file, it stops reading from the kernel. This is something common to all Unices, in that they treat everything as if it's nothing more than a normal file.

But we can understand that the dynamic nature of this file the terminal StdIO file indicates that it is obviously not a normal file on disk, but a link to a kernel API which manages Stdin and Stdout.

==How can I implement StdIO in my kernel?==

===Short Discourse on Design Considerations===

Most early developers do not take into account the fact that their kernel should implement some form of Standard Input/Output for applications. Two of the most early drivers that most OS developers write are actually two drivers which should be part of a Standard Input/Output setup: The console driver, and the Keyboard Driver.

Note very well that Stdin and Stdout are ''not'' necessarily tied to the Keyboard device, or to a console screen, or other screen output. A program may open a file handle, and write to it as if it were a Std Output stream.

Now in most kernels, including the author's, implementing any form of Stdout at boot time is relatively pointless: there are no applications to use it except for the kernel. Not only that, but Stdout must be able to be tied to several streams which need kernel support.

(Todo: give examples of devices/resources (e.g. files) that can be tied to StdIO).

While the screen isn't the only device or resource to which StdOut can be tied, it is necessary to note that from all indications, all OSDev-ers write a driver that directly references the VGA text mode Framerbuffer while booting. This is not detrimental, or wrong. but when the kernel is able to run applications, there must be a resource available in the kernel to support writing to at least the console screen as a StdOut resource. Of course, after boot, when the Graphical side of the kernel must come into play, many ''still'' don't write a Stdout interface because of lack of proper design.

One can generally assume that Stdout is a read or write stream that an application may access.

Let's perform a neat experiment to understand what happens with the StdIO terminal file in *Nix:
Open up two terminal windows under *Nix. On one terminal, type the command: <tt>cat /dev/pts/1</tt> (or whatever your current terminal device is. From now on when I type /dev/pts/1 that's what I'm referring to) and press enter. You will see that Cat has begun reading from the terminal.

On the second one, type <tt>echo <Insert random phrase here></tt>. Note that when the shell is reading input from the keyboard, it allows you to send non-printable characters by escaping them. If you type 

<tt>echo hello \<enter> Jane\<enter> I\'m glad to meet you<enter></tt> 

You'll find that the ''escaped'' <enter> keypresses do not cause the <tt>echo</tt> command to be processed, but that the shell interprets that as a literal LineFeed character and continues to read what characters to send to <tt>echo</tt>. The Linefeed will be sent to <tt>echo</tt> just as any other character would. So you can type stuff on multiple lines, too.

What we have here is a setup where <tt>cat</tt> is reading from the terminal, and <tt>echo</tt> is writing to it. Keep the window with cat open, and keep sending <tt>echo</tt> commands to the terminal.

You'll find that the window with <tt>cat</tt> is showing nothing. Now modify your <tt>echo</tt> commands to write directly to the terminal: 

<tt>echo <insert random phrase here> >/dev/pts/1</tt>.

What happens now? Yes. <tt>cat</tt> reads the characters that <tt>echo</tt> sends. So why didn't cat get them before? Every application has its ''own'' Stdout, so <tt>echo</tt> was writing to ''its'' Stdout, and not to the Terminal/System Stdout.

Returning to where we were before, I was stating that every application should have its own Stdout. This article, as yet immature, will for now only take into account one of the basic StdIn devices: the keyboard. An efficient design for tying Stdin to the Keyboard would be:

* Execute a task. Have a global variable to indicate which process is the currently active process, and therefore would be reading from the user's keyboard input. (Note that i didn't say which is the currently ''executing'' process, but the currently ''active'' process. I.e: the one the user has open in front of him, where the currently active cursor is.

* When the keyboard IRQ is raised, the Keyboard driver fetches the keystroke from the device, and appends it to the StdIn for the active process. The active process reads from StdIn, and gets the keys placed in ''its'' Stdin buffer. '''Question''': But what about the other applications running in the background? Like if the user has a notepad open, and a Firefox where he is posting on a forum. The Firefox window is active.

Well, take some time to think about it a bit. The processor is constantly rotating timeslices between processes. Each process which needs to read from Stdin is, of course, reading from StdIn. The ''default'' Stdin resource is the keyboard, as long as the Process hasn't opened another handle, and specified another resource by its handle, all reads will be assumed to be from the keyboard. But currently Stdin is tied to the ''Active'' Application window process. So ONLY the ''Active'' process actually receives the keys entered at the keyboard, since the keyboard writes directly to the specific buffer of the ''Active'' process. Hmmm...well? What ''about'' the other process windows? Do they need to know what I'm typing in a textbox element on a webpage in Firefox? No. When I switch to ''their'' process, and ''they'' become the Active Process, then when they read from Stdin, then they'll receive the Keyboard's input, too.

Makes sense, right?

That's the basic rundown of a small part of the theory. Later additions should add more body, and even implementation details.

[[Category:OS theory]]
````

<a id="w002702"></a>

## W002702 — Haskell

Bron-ID: W002702
Namespace: Main (0)
Revisie: 31040 | Datum: 2026-10-23T17:14:32Z
Bron: https://wiki.osdev.org/Haskell
Revisie-URL: https://wiki.osdev.org/index.php?title=Haskell&oldid=31040

````text
Haskell is a functional programming language that can be used for operating system development.

This page won't describe the pros and cons or the features of using Haskell for kernel development. Just like with most other languages, you still require some [[Assembly|assembly]] to jump into your Haskell code.

== Getting started ==
The most popular method though has been to use a small [[C]] or [[Assembly|assembly]] [[Microkernel|microkernel]], and use Haskell to implement the servers.

Being a functional programming language, it may be hard to adapt a lot of example code written in imperative languages, therefore a heavy theoretical understanding of operating systems and the concepts involved is required. That being said, you're not completely alone; try Googling for "Haskell on Bare Bones", and more general terms such as "Functional Languages in Operating Systems".

== Compilers ==
The [http://haskell.org/ghc/ Glasgow Haskell Compiler] is the recommended compiler for Haskell. GHC can output executables (though perhaps not in all of the [[Executable Formats]] required), assembly, byte code, [https://downloads.haskell.org/~ghc/7.8.1/docs/html/users_guide/an-external-representation-for-the-ghc-core-language-for-ghc-6.10.html Core], object files, LLVM (and possibly Clang depending, on the host OS) executables, [http://www.cs.tufts.edu/~nr/c--/index.html C--], and finally ANSI C code (which you can send through your favorite C compiler, such as GCC). ''Caveat emptor'': the C output from GHC is highly obfuscated, as it hard-codes stack and heap management, and essentially consists of many small, inter-connected procedures (spaghetti code).

== Operating Systems in Haskell ==
* [http://intoverflow.wordpress.com/kinetic/ Kinetic]
* [http://programatica.cs.pdx.edu/House/ House]

== See Also ==
=== External links ===
* [https://www.haskell.org/ Haskell homepage]
* [https://wiki.haskell.org/index.php Haskell Wiki]
* [https://learnyouahaskell.github.io/ Learn You a Haskell for Great Good!] - Haskell tutorial for beginners
* [https://jaror.codeberg.page/real-world-haskell/ Real World Haskell]  - a more through introduction to Haskell
* [https://en.wikipedia.org/wiki/Haskell_(programming_language) Wikipedia article on Haskell]
* [https://www.cs.tufts.edu/~nr/c--/index.html C--] - Not to be confused with [[C--|Sphinx C--]], it is one of the IRs used by GHC, which was later forked into Cmm
* [https://wiki.haskell.org/Calling_Haskell_from_C Calling Haskell from C]
* [https://www.macs.hw.ac.uk/~dsg/gph/docs/4.06/users_guide/glasgow-ccalls.html Calling C from Haskell]
* [https://downloads.haskell.org/ghc/latest/docs/users_guide/codegens.html GHC Documentation - GHC Backends]
* [https://web.archive.org/web/20040705101408/http://cgi.cse.unsw.edu.au/~chak/haskell/ffi/ffi.pdf The Haskell 98 Foreign Function Interface]
* [https://ogi.altocumulus.org/~hallgren/ICFP2005/ A Principled Approach to Operating System Construction in Haskell]

[[Category:Languages]]
````

<a id="w002703"></a>

## W002703 — Learning 80x86 Assembly

Bron-ID: W002703
Namespace: Main (0)
Revisie: 27910 | Datum: 2023-07-09T05:04:05Z
Bron: https://wiki.osdev.org/Learning_80x86_Assembly
Revisie-URL: https://wiki.osdev.org/index.php?title=Learning_80x86_Assembly&oldid=27910

````text
The following is a list of freely available online resources to help with learning 80x86 assembly language programming.

{| {{wikitable}}
|-
! Reference
! Notes
|-
![https://developer.amd.com/resources/developer-guides-manuals/ AMD64 Architecture Tech Docs]
| AMD's current Programmer's Reference Manuals
|-
![http://www.asmcommunity.net/ ASM Community]
| A community for learning assembly language
|-
![http://flatassembler.net/docs.php?article=manual FASM Programmer's Manual]
| Includes information on most instructions
|-
![http://www.logix.cz/michal/doc/i386/ Intel's 80386 Programmer's Reference Manual (converted to HTML)]
| Similar to Intel's current Programmer's Reference Manual, but easier for beginners as it lacks all the features introduced in newer CPUs that can increase confusion
|-
![http://www.intel.com/products/processor/manuals/ Intel's Current Programmer's Reference Manuals]
|-
![http://www.btinternet.com/~btketman/tutpage.html Ketman Assembly Language Tutorial]
| An interesting looking 8086 interpreter originally intended for learning 8086 assembly (so you can see exactly what effect each instruction has on the CPU) that includes a tutorial
|-
![http://pacman128.github.io/pcasm/ PC Assembly Tutorial]
| Uses NASM, mostly covers 32-bit protected mode assembly only
|-
![http://savannah.nongnu.org/projects/pgubook/ Programming from the Ground Up Book]
| This is an introductory book to programming and computer science using assembly language.
|-
![http://webster.cs.ucr.edu/AoA/index.html The Art of Assembly Language]
| A good reference. Considered a little controversial by some people, as the author attempts to teach assembly language by teaching a completely different language (called HLA) first in an attempt to ease the transition from high level languages to assembly.
|}

==See Also==
*[[Assembly]]

[[Category:Assembly]]
[[Category:X86]]
[[Category:Resources]]
````

<a id="w002705"></a>

## W002705 — GTF

Bron-ID: W002705
Namespace: Main (0)
Revisie: 8944 | Datum: 2009-09-01T19:31:06Z
Bron: https://wiki.osdev.org/GTF
Revisie-URL: https://wiki.osdev.org/index.php?title=GTF&oldid=8944
Doorverwijzing: Video Signals And Timing

````text
#REDIRECT [[Video Signals And Timing]]
````

<a id="w002707"></a>

## W002707 — User:Hangin10

Bron-ID: W002707
Namespace: User (2)
Revisie: 10624 | Datum: 2010-08-24T00:04:33Z
Bron: https://wiki.osdev.org/User:Hangin10
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AHangin10&oldid=10624

````text
I'm Hangin10. This is a list of information. 

== Hangin10 ==

My website is (when it's up): qonfusor.thruhere.net

I've also been watching the Syfy show Warehouse 13, so eventually warehouse.doesntexist.org will be a fan site for it. For now its just the same server as my main website. Sometimes I run an IRC server on the same computer.

== Current Projects ==

=== [[User:Hangin10/OS Kernel|OS Kernel]] ===
My own microkernel-based operating system.

== Past Projects ==

=== [http://telesto.sf.net/ Telesto] ===
A compiler for a programming language I designed myself. The parser is decent. What brought me down was trying to figure out AST annotation and error tables (especially when it comes to various types of templates). Essentially meant to be natively compiled Pascal with braces (or JavaScript) with a standard library that came from BSD/MIT or LGPL sources.
````

<a id="w002708"></a>

## W002708 — User:04wilsonm

Bron-ID: W002708
Namespace: User (2)
Revisie: 9019 | Datum: 2009-09-23T13:51:41Z
Bron: https://wiki.osdev.org/User:04wilsonm
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3A04wilsonm&oldid=9019

````text
Hello,

My name is Michael Wilson and I am a 16 year old boy living in England, UK.

I am into computers and know C. I am currently learning C++ and Javascript.

If you PM me here or on the forums i will be happy to help.

My MSN is go2hop@live.co.uk

Always happy to help,

Michael
````

<a id="w002709"></a>

## W002709 — User:Tyrelhaveman

Bron-ID: W002709
Namespace: User (2)
Revisie: 9023 | Datum: 2009-09-23T16:34:24Z
Bron: https://wiki.osdev.org/User:Tyrelhaveman
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATyrelhaveman&oldid=9023

````text
During the day I am a software engineer at a global software company where I work on an SSH server. During the night I am an operating system developer.

I have created Zebu, an x86 operating system which includes multitasking, FAT support, keyboard, mouse, VESA, etc., but am thinking about starting over again, this time focusing on some different things (such as 64-bit support). We'll see how that goes.

The most important thing to me in OS development (or anything else) is learning as many different things as I can. I love learning.
````

<a id="w002710"></a>

## W002710 — User talk:04wilsonm

Bron-ID: W002710
Namespace: User talk (3)
Revisie: 9038 | Datum: 2009-09-28T17:45:01Z
Bron: https://wiki.osdev.org/User_talk:04wilsonm
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3A04wilsonm&oldid=9038

````text
While I do appreciate your attempt at helping, please, do not copy pages from across the web. It is usually illegal. (see [[Licensing]] to know why). Feel free to prove me wrong though. - [[User:Combuster|Combuster]] 16:50, 23 September 2009 (UTC)

I got permission from the guy who owned it. And gave full credit.
:That fixes one page, but not the other. Please read the licensing page again as unlike your claims, everything is copyrighted.
````

<a id="w002712"></a>

## W002712 — RSDT

Bron-ID: W002712
Namespace: Main (0)
Revisie: 30509 | Datum: 2026-04-22T19:48:54Z
Bron: https://wiki.osdev.org/RSDT
Revisie-URL: https://wiki.osdev.org/index.php?title=RSDT&oldid=30509

````text
{{ACPI}}

The '''RSDT''' (Root System Description Table) is a data structure used in the [[ACPI]] programming interface. This table contains pointers to all the other system description tables.

==Detecting==
The RSDT is pointed to by the [[RSDP]], in the <code>RsdtAddress</code> field. If the ACPI version is at least 2.0, the [[XSDT]] should be used<ref>On x86-based systems, the RSDT and XSDT are likely to point to the same table, for compatibility reasons.</ref>.

==Structure==
Like all other system description tables, the RSDT is separated into a header and data.
<syntaxhighlight lang="c">
struct SDT_header {
  char Signature[4];
  uint32_t Length;
  uint8_t Revision;
  uint8_t Checksum;
  char OEMID[6];
  char OEMTableID[8];
  uint32_t OEMRevision;
  uint32_t CreatorID;
  uint32_t CreatorRevision;
} __attribute__ ((packed));
</syntaxhighlight>

=== Fields ===

==== Signature ====
A 4 byte signature used for identification, equal to <code>RSDT</code>.

==== Length ====
The length of the entire table, including the header.
==== Checksum ====
An 8-bit checksum field for the whole table, inclusive of the header. All bytes of the table, when summed, must be equal to 0 (mod <code>0x100</code>).
<syntaxhighlight lang="c">
bool verify_checksum(struct SDT_header *header)
{
    uint8_t sum = 0;

    for (int i = 0; i < header->Length; i++)
    {
        sum += ((uint8_t *) header)[i];
    }

    return sum == 0;
}
</syntaxhighlight>

==== Entries ====
The RSDT contains an array of 32-bit physical pointers to other system descriptor tables.<syntaxhighlight lang="c">
struct RSDT {
  struct SDT_header header;
  uint32_t entries[(header.Length - sizeof(header)) / 4];
} __attribute__ ((packed));
</syntaxhighlight>
== Usage ==
Tables can be found by traversing <code>entries</code> and looking for one with the correct signature.

As an example, the [[FACP]] can be found using the following code:<syntaxhighlight lang="c">
void *find_FACP(struct RSDT *rsdt)
{
    int entries = (rsdt->h.Length - sizeof(rsdt->h)) / 4;

    for (int i = 0; i < entries; i++)
    {
        struct SDT_header *h = (struct SDT_header *) rsdt->entries[i];
        if (!strncmp(h->Signature, "FACP", 4))
            return (void *)h;
    }

    return NULL;
}
</syntaxhighlight>The [[FADT]], [[SSDT]], and [[MADT]] tables are all pointed to by the RSDT, and are quite important for ACPI. For multiprocessor systems, the [[SRAT]] and [[SLIT]] tables are essentially required.

== List of Tables ==

=== Defined by ACPI ===

The ACPI Specification 5.0 has direct definitions for the following System Descriptor tables, identified by their signatures:

* "APIC": Multiple APIC Description Table ([[MADT]])
* "BERT": Boot Error Record Table ([[BERT]])
* "CPEP": Corrected Platform Error Polling Table ([[CPEP]])
* "DSDT": Differentiated System Description Table ([[DSDT]])
* "ECDT": Embedded Controller Boot Resources Table ([[ECDT]])
* "EINJ": Error Injection Table ([[EINJ]])
* "ERST": Error Record Serialization Table ([[ERST]])
* "FACP": Fixed ACPI Description Table ([[FADT]])
* "FACS": Firmware ACPI Control Structure ([[FACS]])
* "HEST": Hardware Error Source Table ([[HEST]])
* "MSCT": Maximum System Characteristics Table ([[MSCT]])
* "MPST": Memory Power State Table ([[MPST]])
* "OEMx": OEM Specific Information Tables (Any table with a signature beginning with "OEM" falls into this definition)
* "PMTT" Platform Memory Topology Table ([[PMTT]])
* "PSDT": Persistent System Description Table ([[PSDT]])
* "RASF": ACPI RAS Feature Table ([[RASF]])
* "RSDT": Root System Description Table (This wiki page; included for completeness)
* "SBST": Smart Battery Specification Table ([[SBST]])
* "SLIT": System Locality System Information Table ([[SLIT]])
* "SRAT": System Resource Affinity Table ([[SRAT]])
* "SSDT": Secondary System Description Table ([[SSDT]])
* "XSDT": Extended System Description Table ([[XSDT]]; 64-bit version of the RSDT)

=== Reserved by ACPI ===

The ACPI specification maintains reservations for additional third party defined description tables.  Each third party submitting a request for a reservation also maintains a specification for these tables, located on individual sites.  See the ACPI spec ([http://www.acpi.info/spec50.htm]) for more information.

== See Also ==

=== External Links ===
[https://uefi.org/sites/default/files/resources/ACPI_Spec_6_5_Aug29.pdf ACPI Specification (version 6.5)]

=== References ===
[[Category:ACPI]]
````

<a id="w002713"></a>

## W002713 — Grub Error 13

Bron-ID: W002713
Namespace: Main (0)
Revisie: 30350 | Datum: 2026-03-18T19:36:16Z
Bron: https://wiki.osdev.org/Grub_Error_13
Revisie-URL: https://wiki.osdev.org/index.php?title=Grub_Error_13&oldid=30350

````text
== GRUB Error 13: Invalid or unsupported executable format: ==

[[GRUB]] is a reference implementation of the GNU Multiboot specification, commonly used by most 32 bits OS developers. It has the advantages of being a(n) (obviously) pre-written bootloader with more code to handle quirks of various BIOSs than it is worthwhile for any individual to do by themself.

See the following thread for a bit more info: http://forum.osdev.org/viewtopic.php?f=1&t=21158&start=0.

However, many people have problems getting the GRUB to recognize their kernel's executable image.

The following is a solution for GRUB's infamous Error 13 pertaining to ELF files.

== Example Source ==

The GNU Multiboot Specification requires the Multiboot Header to be aligned on a 4 byte boundary within the first 8 KB of the kernel executable file. should your kernel's image not fit this description, GrUB will promptly issue an Error 13.

The [[Linker Script]] gives the linker details on how to position the executable sections of your kernel within the kernel executable file. You are not restricted (using ELF; there are Executable formats which restrict you to certain format-specific sections) to just the ELF recommended sections. You may define as many sections as you please within your kernel executable image, and position them as you see fit, using your linker script. We assume the use of the GNU ld linker for this article.

Within the asm source file where you have defined your Multiboot Header options, edit the Multiboot Header to look like this: (This article is not going into detail about the GNU Multiboot spec. I'm just telling you minor changes to make ''around'' the header)

<syntaxhighlight lang="ASM">
;; Remove the section .text you copy/pasted from the Bare Bones tutorial ;)
section .__mbHeader

align 0x4
;; Copy/Paste your current Multiboot Header here

;; Reposition the section .text here.
</syntaxhighlight>

What does this do? We have placed the Multiboot header into a separate ELF section in the relocatable file generated by NASM. When the linker is going through the Object files, it places the section symbols in each object file in their corresponding output sections in the eventual kernel image.

How do we tell the linker to place the .__mbHeader ELF section in the 1st 8 KB of the kernel image? Technically, there is no way to guarantee that it will be there if the kernel file grows exorbitantly large. At that point, you may have to invent a special technique of editing your output object file manually, and placing the Multiboot header in the first 8 K. However you do that is ...however you manage to do it. The simple solution is to ensure that your kernel image doesn't grow ''too'' large. Again, technically, even a monolithic kernel's image shouldn't grow that large, even if you try.

The linker is told to use the .__mbHeader section as the first section in the output file like this:

'''yourLinkerScript.ld'''
<syntaxhighlight lang="C">
/*This is the SECTIONS command in your ld script. Locate it.*/
SECTIONS {
   /* There is a command with the following general form in your linker script:
    * . = 0xXXXXXXXX;
    * You are to leave this in place, and put the section for .__mbHeader beneath that line.
    **/
   .__mbHeader : {
      *(.__mbHeader)
   }

   /* The other sections are here. After you edit the script. If you copy/pasted the Bare Bones ld script, then your .text sections follows here.*/
   /*...*/
}
</syntaxhighlight>

In other words, we simply insert the lines for the .__mbHeader section at the top of the SECTIONS command, telling ld to place the .__mbHeader section first.

One or two notes, since the copy/paste horde wouldn't actually notice these points: We assume that your kernel is not virtually linked to a higher address. If it is, then you should edit the first line of the .__mbHeader linker command to look like this:

<syntaxhighlight lang="C">
.__mbHeader : AT( ADDR(.__mbHeader) - YOUR_KERNEL\'S_VIRTUAL_OFFSET_HERE ) {
</syntaxhighlight>

Should you do this right, (and it shouldn't be that difficult), your kernel shouldn't emit that Error 13 message anymore, and provided you linked you kernel in the right format, and whatnot, GRUB should be able to load your kernel nicely.

== See Also ==
=== Articles ===
* [[GRUB]]
=== Forum Threads ===
* http://forum.osdev.org/viewtopic.php?f=1&t=21158&start=0
=== External Links ===
* [http://www.gnu.org/software/grub/manual/multiboot GNU Multiboot Specification].

[[Category:GRUB]]
[[Category:Bootloaders]]
[[Category:Troubleshooting]]
[[Category:FAQ]]
````

<a id="w002714"></a>

## W002714 — C++ to ASM linkage in GCC

Bron-ID: W002714
Namespace: Main (0)
Revisie: 28237 | Datum: 2023-07-10T22:05:18Z
Bron: https://wiki.osdev.org/C%2B%2B_to_ASM_linkage_in_GCC
Revisie-URL: https://wiki.osdev.org/index.php?title=C%2B%2B_to_ASM_linkage_in_GCC&oldid=28237

````text
A small note before we begin: The GNU Compiler Collection C compiler, a very versatile compiler that has been around for a very long time, is pretty much the standard for OS Dev'ing, since it is even used (as you probably already know) to compile the Linux kernel. In fact, Linux is meant to be compiled with GCC. It has lots of useful extensions (__attribute__(())) which ease development by leaps and bounds, should you take the time to read about it enough.

Also, this article in itself is not really sufficient for a full understanding of Linking to C++ methods within C or assembly. C++ uses the hidden 'this' pointer, which will be discussed in another article proposed for a later date.

We will assume the use of the GCC compiler for your HLL development, and the use of C++ (C wouldn't be that different) for your little HLL - Assembly linkage escapade.

== C++ Name Mangling ==

GCC follows the [http://www.codesourcery.com/public/cxx-abi/abi.html Itanium C++ ABI]. The link preceding this text is of course a link to a public collection of documents on the Itanium C++ ABI. One of the things which prevents generally absolute portability of C++ libraries is the fact that different compilers use different ''Name Mangling'' schemes.

=== Why Mangle Symbol Names? ===

In C++, the symbols you define are generally not exclusive. For example, the function, <tt>getObjId()</tt> in C would simply be encoded as getObjId in the object file output. But in C++, since this function may be overloaded, it needs to contain extra information about itself in the name so that when an argument (function signature is the correct term) match is to be made, the linker or compiler may know which function is to be linked to.

Take the following example: A global function (i.e. with no namespace attached) getObjId() in C++ may be overloaded to these three instances: (These are not technically correct; your compiler will most likely emit something different. They are for demonstration, and are simply scratched out from the general mangling pattern GCC uses).

<tt>int getObjId(void); int getObjId(int); int getObjId(unsigned int);</tt>

From the source point of view, you believe that these three have the same name in the object file, and the compiler magically knows which one to choose for a call. However, three distinct symbols are generated respectively, in the general form:

<tt>_ZN8getObjIdEv, _ZN8getObjIdEi, _ZN8getObjIdEj</tt>.

The compiler uses the generated symbol name to encode information about the symbol. This generally says that 'this is a mangled symbol' (_Z). 'It has 8 characters of user-defined symbol relevance' (8), and those are [getObjId]. The 'E' is probably used to mean END, and after that GCC generally places several letters and namespace/object names as details on the arguments. (v=void, i=int, j=unsigned int...).

After seeing this, you can tell that, in order therefore to call a C++ function from assembly, '''if''' the compiler has mangled it (there are many cases in which there is no need for mangling, and the compiler may naturally leave a symbol alone in many cases, e.g. global variables), you would need to use a executable symbol interpreter, such as <tt>nm</tt> or <tt>objdump</tt> to see the name GCC used in the generated object file.

Note well that varying compilers DO ''NOT'' use the same mangling scheme, and in fact, are encouraged by the C++ standards committee to go ahead and use their own mangling schemes as they see fit.

Keep this bit of information in mind.

== Essentially, then, What is 'Linking' to external an external function, or variable? ==

Variables in C++, and C, and many other languages have several ''scope'' levels. These are the levels of visibility of a symbol in one object file to the code in another object file. Generally, within one object file, unless you take steps to ensure otherwise, all static local and global symbols are available to the code in that file.

=== Global Symbols ===
Global symbols are those which are seen universally by all object code in the entire program, at the linking phase. Technically, during execution, the whole program is just bytes, so actually, every symbol is just an address, so every function has access to every symbol. But it is useful to programmers to be able to ''abstract'' access to symbols.

In C and C++, you make a symbol global by defining it ''outside'' of any function. In C++, you may still hide the symbol by having it inside a 'private' section of a global symbol. But this is irrelevant to this article, seeing as anyone who is reading this article should be attempting to develop an OS. We assume you already understand your language.

In assembly, (NASM, specifically; the author does not use GAS. Anyone who knows how GAS works may add to this article as they see fit.) all symbols are automatically local to the particular assembly file in which they appear. To make a symbol global, you must use the 'global _SYMBOL_NAME_' directive. If I remember right, in GAS, that's '.globl _SYMBOL_NAME'.

<source lang="ASM">
;-------------------------------------
; I usually like to place all my directives in a section above the code:
;-------------------------------------
global mysymbol, mycodesymbol
;-------------------------------------

mysymbol:
   dd 1, 2, 3

mycodeSymbol:
   push eax
   pop eax
ret

;; This symbol is confined to be seen only within this file, since the linker can't see it.
mylocalSymbol:
   push eax
   pop eax
ret

</source>

To clarify, technically, the linker can 'see' all symbols. It just chooses to ignore linkage between files for symbols not exclusively declared global. If you were to write a second file:

<source lang="ASM">
;--------------------------------------
; Extern directives here
;--------------------------------------
extern mysymbol, mycodesymbol
;; This one won't work.
extern mylocalsymbol
;--------------------------------------
</source>

This file will assemble, but when passed through the linker, you will receive a message to the effect that no such symbol has been defined for 'mylocalsymbol'.

=== Local Symbols ===
These are either Function local (on the stack, and therefore definitely not available to any other code since its persistence is unpredictable. It can be popped off at any time.) or File local (like mylocalsymbol above).

In C/C++, we know that a static variable defined outside of a function is file local. It may not be linked to outside of that particular file.

So now we get back to the main question presented at the top of this section of the article: What is external linkage? External linkage is the linking to a global symbol which is not defined in the same file scope as the file you're working in. The linker will therefore place the referred variable's address where it is referenced in the referencing file.

== Linking to External Symbols in C++ ==

In C++, the compiler has no idea what kind of symbol you intend to link to. There is technically no such thing as different 'kinds' of symbols, so we'll immediately discard that idea.

Technically, you can link to any kind of symbol using the "C" linkage style. I'll explain that now.

=== "C" Linkage ===

Well, let's think this through. In C, all symbols are simply self representing. If I declare a function, getObjId, it will be called 'getObjId' in the output file. C uses no name mangling since you cannot have different symbols with the same name occurring in the same namespace in C.

In other words, when you tell the compiler that you are linking to an external variable with "C" style linkage, you are telling the compiler: "I am linking to a symbol of name XYZ. This is EXACTLY how the symbol looks, and there is no name mangling.". That is all 'extern "C"' means. It explicitly tells the compiler that there is nothing special about the symbol's name, and that it is to be taken exactly as you type it.

With this in mind, we not understand why in C, there is no need to specify a linkage style, since C only understands symbolas as you type them. C does not have any kind of name mangling, and expects plain, absolute symbol names. So to link to assembly f C, you just say 'extern SYMBOL NAME', and not 'extern "C" SYMBOL NAME'.

So <tt>extern "C" getObjId</tt> tell the compiler to insert references to a symbol of the exact name 'getObjId' within your output object file. The linker will see your references, and look for a global symbol of that exact name, and if it is found, and there are no duplicates, it simply places the address of that symbol wherever the compiler placed a reference to it.

Now, let's go back to name mangling. Name mangling is simply the generation of a symbol as you define it, but with extra information encoded so as to ensure that symbols in an output object file are unique. Remember: No two global symbols may have the same name. No two File local symbols within the ''same file'' may have the same name either. When the linker links together object files with identical symbols, if the symbols are file local, it doesn't matter.

But the most important thing is that AFTER the file is compiled, the symbols are already determined, and so you can then link to any kind of symbol as a "C" style symbol if you know its absolute, post-mangled name, not so?

If say, you have a class 'foo' with a method 'bar(void)', the mangled symbol after compiling looks something like this (if compiled with GCC):

<tt>_ZN3foo3barEv</tt>.

This is the absolute symbol name for foo::bar. You may link to it therefore, from plain C, provided that the function does not expect a 'this' pointer, which is another matter altogether. Technically it won't work until you satisfy the 'this' pointer condition, but I'm just trying to help you understand exactly what it means when you say a symbol has "C" linkage, and what the compiler takes that to mean.

So linking to 'extern "C"' symbols is the same as telling the compiler to just trust you, and place references to XYZ symbol name as is, even though it may never see the definition of that symbol within the ''current file''. The symbol is defined elsewhere. It may even be defined in a shared library, and be expected to be linked in by the OS at runtime (Dynamic Linking), although this usually requires a little more, and is usually handled by the compiler and linker as set up by the host OS (this is where OS libraries come in).

=== "C++" Linkage ===

Surprised? Yes, it does exist. However, a C++ compiler uses this linkage style by default. If you explicitly state it, it won't actually change anything. I know for a fact that GCC/G++ will not complain if you tell it to link to a symbol in C++ style.

So what does "extern "C++"" tell the compiler? It means: "Do your thing."

If you do the following:

<source lang="cpp">
class foo {
   public:
      int bar(void);
};

extern foo fubar;
extern "C++" foo fubar
</source>

The compiler will take them both to mean the same thing: 'Link to an external symbol which has the ''demangled'' name fubar, and is of type foo'.

The compiler will apply its OWN mangling scheme to the symbol, then insert references to the symbol in its own mangling scheme. So what say, a symbol was compiled in MSVC, and then you externally reference it in a C++ file to be compiled in GCC. Will the symbol link?

No. These two compilers will do the following: The symbol is compiled in one object file by MSVC. MSVC places the symbol, all mangled in its own way, into the output object file.

Then GCC is told to link to that symbol. It places a reference to a symbol name, all mangled in ''its'' own way. The linker is called on both object files. The second one is referencing a symbols which the linker sees nowhere. You are told the symbol referenced by the second object file does not exist.

== C++ - Assembly linkage ==

I'm sorry I took this long to get to this part of the article, but the facts given above are pertinent.

To link to a C/C++ symbol from an assembly file, you simply tell the Assembler that the symbol is external. Assemblers ''always'' insert references as they see them, so they are always "C" style, if you please. The assembler takes you at your word. This is why you can look for the mangled symbol name of a function, or variable, and then actually place the mangled name into your assembly file (extern _ZN3foo3barEv) and it will work (depending on whether or not a 'this' pointer is involved).

To link to an assembly symbol from C/C++, you must know what the absolute symbol name in the assembly file is, and ensure it's global, then link to the absolute symbol name "C" style. (extern "C" SYMBOL_NAME).

The 'This' pointer issue is another thing altogether, and is actually a very serious consideration you should take into account when designing your kernel, or choosing whether or not to use C++ altogether. C makes library generation, and linking easier. C++ makes design and re-structuring (you will restructure your design many times, so this is a big plus) much easier.

== See Also ==
[[Calling Conventions]]

[[Category:C++]]
[[Category:Assembly]]
````

<a id="w002715"></a>

## W002715 — I use a Custom Filesystem - What Bootloader Solution is right for me?

Bron-ID: W002715
Namespace: Main (0)
Revisie: 28121 | Datum: 2023-07-10T02:30:35Z
Bron: https://wiki.osdev.org/I_use_a_Custom_Filesystem_-_What_Bootloader_Solution_is_right_for_me%3F
Revisie-URL: https://wiki.osdev.org/index.php?title=I_use_a_Custom_Filesystem_-_What_Bootloader_Solution_is_right_for_me%3F&oldid=28121

````text
While rolling your own filesystem is [[What filesystem should I use|not recommended]], you may find yourself in a place where a specially designed filesystem is a must have. The most obvious solution is to write a custom bootloader specifically designed for the new filesystem, or you could use an existing bootloader - [[GRUB]]. This guide shows how you can use this tool for your work. 

== What GRUB can do ==

GRUB allows you to do a lot of things that you wouldn't be able to do with any other bootloader due to its ability to load an OS in raw sectors from a floppy or hard drive. 

I must give a small overview of the boot process for an OS that is installed in a partition other than the first partition, which is also not the only OS installed on the hard Disk.

The Master Boot Record holds the Boot Manager for the disk. GRUB is technically a boot loader, but its functionality far exceeds that of a conventional boot loader, and crosses the line of a Boot Manager very convincingly. Upon completion of its initialization sequence, the BIOS, in order of the priority set in the CMOS, loads a bootloader from the default boot disk. Assume that the HDD is given priority for our example. So the BIOS loads the first HDD's (or second, or third, or fourth, depending on what you set the boot order to), [[MBR (x86) | Master Boot Record]] into RAM at 0x7C00 and jumps to it, passing the drive number via a register. The MBR contains code to bootstrap an OS, or a second stage of itself, depending on how considerate the bootloader in question is. GRUB loads its second stage, and displays a menu listing all of the OSs recorded in its configuration file.

These OSs have given GRUB commands on how to boot them:

* For Windows, a record is placed which indicates the partition Windows is installed on, and an instruction to ''chainload'' one sector (512 bytes) into RAM at 0x7C00, and jump to it; In other words, simply load the Windows bootloader into RAM as if it was placed there by the BIOS.
* For OSs that use EXT, ReiserFS, or other directly supported GRUB filesystems, you have the convenience of being able to ask GRUB to look into the partition and ''find'' your kernel file and load it into memory. 
* For OSs that do ''not'' use a GRUB supported File system, the convention has been to simply ask GRUB to chainload the partition boot sector for your OSs partition and jump to it. However, using some logic, it is possible to avoid having to do anything whatsoever in real mode, or having to write a bootloader, even for a custom file system, and have GRUB do all the work.

=== The GRUB 'kernel' and 'chainloader' commands ===

GRUB is able to make a particular partition on a hard drive the 'root' for its operations. It can do so in one of two ways:

* Simply take note of the fact that the user wants to set that partition as the root, or
* If the partition has a recognizable file system (one known by GRUB), GRUB can also ''mount'' it for file system parsing to actually locate a kernel image on disk.

When a partition is chosen as the root for GRUB's operations, there are two ways to load an executable image:

'''The 'kernel' command:'''
The kernel command either reads an executable, Multiboot Compliant image from a recognized File system, or as absolute sectors from the start of the partition.

'''The 'chainloader' command:'''
The chainloader command loads an executable image of the assumed format 'binary' at 0x7C00 and jumps to the address 0x7C00.

== How may I use this information to ensure that GRUB can load an executable image off my custom FS? ==

Take some time to think. You are designing a custom filesystem. There is therefore no restriction on ''where'' on the partition (which absolute sector) the FS header, and therefore the FS itself should start. You are completely free to, in your FS specification, state that a partition complying with YourOwnFS should not touch the 1st, say, ~16-32KB, and that it is reserved for boot information, or other useful data. Therefore, in the Partition Table, your partition is stated to begin at Sector X, but the Fs itself may begin at sector X+BOOT_RESERVED_OFFSET. This way, we replace the old, completely illogical standard which dictates that a partition's boot sector contains boot code, and, using the reserved sectors for your FS, place an executable image at the absolute beginning of the partition containing a full, conveniently sized boot program for your OS.

To load the program, your instructions to GRUB will be something like the following:

<source lang="text">rootnoverify (hd0,PARTITION_NUMBER)
kernel 0+BOOT_RESERVED_OFFSET
boot</source>

The kernel command loads a Multiboot compliant kernel, and in this case, we have specified an absolute number of sectors to load. The 'kernel' command can automatically detect if the file it's loading from the sectors is an ELF program. In other words, your 'bootloader' may even be a full ELF program, using this trick. In fact, you may even decide to place your kernel's executable image ''as the first N sectors, and have GRUB ELF load the kernel as absolute sectors, and not have to worry about parsing your custom FS at all.''

Your kernel image, now loaded into memory (if it's an ELF, you can link it to 1MB physical), can then set itself up painlessly, and it also gets the benefit of the GRUB Multiboot Data structure passed to it. This method completely invalidates the need to write a real mode bootloader. There are other reasons why it is probably ''best'' that Hobbyist OS Developers use this method:

The method implies that GRUB is placed into the MBR of a hobby OS partition installation, or even whole disk installation. If all hobby OSs use this method, whether or not they use a custom FS, it gives them the advantage of being able to load a full program, in ''protected mode'' via GRUB, and bypassing the Real Mode bootloader stage ''completely''. No more problems with writing an efficient program in 446 bytes.

[[Category:Filesystems]]
[[Category:Bootloaders]]
[[Category:FAQ]]
````

<a id="w002716"></a>

## W002716 — PCI IDE Controller

Bron-ID: W002716
Namespace: Main (0)
Revisie: 28314 | Datum: 2023-09-21T01:33:51Z
Bron: https://wiki.osdev.org/PCI_IDE_Controller
Revisie-URL: https://wiki.osdev.org/index.php?title=PCI_IDE_Controller&oldid=28314

````text
{{Disputed|Talk:IDE}}
IDE is a keyword which refers to the electrical specification of the cables which connect ATA drives (like hard drives) to another device. The drives use the ATA (Advanced Technology Attachment) interface. An IDE cable also can terminate at an IDE card connected to PCI.

[[ATAPI]] is an extension to ATA (recently renamed to PATA) which adds support for the SCSI command set.

== Parallel/Serial ATA/ATAPI ==
IDE can connect up to 4 drives. Each drive can be one of the following:

* ATA (Serial): Used for most modern hard drives.
* ATA (Parallel): Commonly used for hard drives.
* ATAPI (Serial): Used for most modern optical drives.
* ATAPI (Parallel): Commonly used for optical drives.

Accessing an ATA/PATA drive works the same way as accessing a SATA drive. This also implicitly states that accessing a PATAPI ODD is the same as accessing a SATAPI ODD. An IDE driver does not need to know whether a drive is parallel or serial, it only has to know whether it's using ATA or ATAPI.

== IDE Interface ==

If you open your case up and take a look at your motherboard, you will most likely see one or two (or possibly more) of the slots.

The white and green ports are IDE ports, also known as ''channels''. In this example there are both primary and secondary IDE channels which only PATA can be connected to; this means that it only supports PATA/PATAPI drives.

Each port can have a PATA cable connected to it. One master drive, or two drives (master and slave), can be connected to one PATA cable. So that leaves us with the following possibilities:

* Primary Master Drive.
* Primary Slave Drive.
* Secondary Master Drive.
* Secondary Slave Drive.

Each drive can be either PATA or PATAPI.

== Serial IDE ==
Almost every modern (this article is probably written in early 2010 so it assumes motherboards still have ide/ahci modes) motherboard has a Serial IDE channel which allows [[SATA]] and SATAPI Drives to be connected to it. There are 4 Serial IDE Ports. Each port is connected to a drive with a SATA Cable. Basically you can only have one drive connected to the Serial IDE port. Each pair of ports (every 2 ports) form one channel.

Serial IDE also has a few possibilities:

* Primary Master, also called SATA1.
* Primary Slave, also called SATA2.
* Secondary Master, also called SATA3.
* Secondary Slave, also called SATA4.

== Detecting a PCI IDE Controller ==
Each IDE controller appears as a device on the [[PCI]] bus and can be identified by reading the configuration space. If the class code is 0x01 (Mass Storage Controller) and the subclass code is 0x01 (IDE), the device is an IDE controller. The programming interface byte(Prog If) determines how you'll access it.
* Bit 0: When set, the primary channel is in PCI native mode. When clear, the primary channel is in compatibility mode (ports 0x1F0-0x1F7, 0x3F6, IRQ14).
* Bit 1: When set, you can modify bit 0 to switch between PCI native and compatibility mode. When clear, you cannot modify bit 0.
* Bit 2: When set, the secondary channel is in PCI native mode. When clear, the secondary channel is in compatibility mode (ports 0x170-0x177, 0x376, IRQ15).
* Bit 3: When set, you can modify bit 2 to switch between PCI native and compatibility mode. When clear, you cannot modify bit 2.
* Bit 7: When set, this is a bus master IDE controller. When clear, this controller doesn't support DMA.

If you want to access an IDE channel in PCI native mode or use the bus master function, you must additionally read the BARs to find which I/O ports to use.
* BAR0: Base address of primary channel in PCI native mode (8 ports)
* BAR1: Base address of primary channel control port in PCI native mode (4 ports)
* BAR2: Base address of secondary channel in PCI native mode (8 ports)
* BAR3: Base address of secondary channel control port in PCI native mode (4 ports)
* BAR4: Bus master IDE (16 ports, 8 for each channel)
Note that BAR1 and BAR3 specify 4 ports, but only the port at offset 2 is used. Offsets 0, 1, and 3 should not be accessed.

If either IDE channel is in PCI native mode, you must also read the interrupt line or interrupt pin register to determine which interrupt to use. If both channels are in PCI native mode, they'll both share the same interrupt. The interrupt line field is only valid when using the [[8259 PIC|PIC]].

== Detecting IDE Drives ==

To initialise the IDE driver, we call ide_initialise:
<source lang="c">
void ide_initialize(unsigned int BAR0, unsigned int BAR1, unsigned int BAR2, unsigned int BAR3,
unsigned int BAR4) {
</source>

If you only want to support the parallel IDE, you can use these parameters:
<source lang="c">
ide_initialize(0x1F0, 0x3F6, 0x170, 0x376, 0x000);
</source>

We can assume that BAR4 is 0x0 because we are not going to use it yet.
We will return to ide_initialize, which searches for drives connected to the IDE. Before we go into this function, we should write some support functions and definitions which will help us a lot.

=== Status ===

The Command/Status Port returns a bit mask referring to the status of a channel when read.

<source lang="c">
#define ATA_SR_BSY     0x80    // Busy
#define ATA_SR_DRDY    0x40    // Drive ready
#define ATA_SR_DF      0x20    // Drive write fault
#define ATA_SR_DSC     0x10    // Drive seek complete
#define ATA_SR_DRQ     0x08    // Data request ready
#define ATA_SR_CORR    0x04    // Corrected data
#define ATA_SR_IDX     0x02    // Index
#define ATA_SR_ERR     0x01    // Error
</source>

=== Errors ===

The Features/Error Port, which returns the most recent error upon read, has these possible bit masks

<source lang="c">
#define ATA_ER_BBK      0x80    // Bad block
#define ATA_ER_UNC      0x40    // Uncorrectable data
#define ATA_ER_MC       0x20    // Media changed
#define ATA_ER_IDNF     0x10    // ID mark not found
#define ATA_ER_MCR      0x08    // Media change request
#define ATA_ER_ABRT     0x04    // Command aborted
#define ATA_ER_TK0NF    0x02    // Track 0 not found
#define ATA_ER_AMNF     0x01    // No address mark
</source>

=== Commands ===

When you write to the Command/Status port, you are executing one of the commands below.

<source lang="c">
#define ATA_CMD_READ_PIO          0x20
#define ATA_CMD_READ_PIO_EXT      0x24
#define ATA_CMD_READ_DMA          0xC8
#define ATA_CMD_READ_DMA_EXT      0x25
#define ATA_CMD_WRITE_PIO         0x30
#define ATA_CMD_WRITE_PIO_EXT     0x34
#define ATA_CMD_WRITE_DMA         0xCA
#define ATA_CMD_WRITE_DMA_EXT     0x35
#define ATA_CMD_CACHE_FLUSH       0xE7
#define ATA_CMD_CACHE_FLUSH_EXT   0xEA
#define ATA_CMD_PACKET            0xA0
#define ATA_CMD_IDENTIFY_PACKET   0xA1
#define ATA_CMD_IDENTIFY          0xEC
</source>

The commands below are for ATAPI devices, which will be understood soon.

<source lang="c">
#define      ATAPI_CMD_READ       0xA8
#define      ATAPI_CMD_EJECT      0x1B
</source>

ATA_CMD_IDENTIFY_PACKET and ATA_CMD_IDENTIFY return a buffer of 512 bytes called the identification space; the following definitions are used to read information from the identification space.
<source lang="c">
#define ATA_IDENT_DEVICETYPE   0
#define ATA_IDENT_CYLINDERS    2
#define ATA_IDENT_HEADS        6
#define ATA_IDENT_SECTORS      12
#define ATA_IDENT_SERIAL       20
#define ATA_IDENT_MODEL        54
#define ATA_IDENT_CAPABILITIES 98
#define ATA_IDENT_FIELDVALID   106
#define ATA_IDENT_MAX_LBA      120
#define ATA_IDENT_COMMANDSETS  164
#define ATA_IDENT_MAX_LBA_EXT  200
</source>

When you select a drive, you should specify the interface type and whether it is the master or slave:
<source lang="c">
#define IDE_ATA        0x00
#define IDE_ATAPI      0x01

#define ATA_MASTER     0x00
#define ATA_SLAVE      0x01
</source>

Task File is a range of 8 ports which are offsets from BAR0 (primary channel) and/or BAR2 (secondary channel). To exemplify:
* BAR0 + 0 is first port.
* BAR0 + 1 is second port.
* BAR0 + 2 is the third
<source lang="c">
#define ATA_REG_DATA       0x00
#define ATA_REG_ERROR      0x01
#define ATA_REG_FEATURES   0x01
#define ATA_REG_SECCOUNT0  0x02
#define ATA_REG_LBA0       0x03
#define ATA_REG_LBA1       0x04
#define ATA_REG_LBA2       0x05
#define ATA_REG_HDDEVSEL   0x06
#define ATA_REG_COMMAND    0x07
#define ATA_REG_STATUS     0x07
#define ATA_REG_SECCOUNT1  0x08
#define ATA_REG_LBA3       0x09
#define ATA_REG_LBA4       0x0A
#define ATA_REG_LBA5       0x0B
#define ATA_REG_CONTROL    0x0C
#define ATA_REG_ALTSTATUS  0x0C
#define ATA_REG_DEVADDRESS 0x0D
</source>

The ALTSTATUS/CONTROL port returns the alternate status when read and controls a channel when written to.
* For the primary channel, ALTSTATUS/CONTROL port is BAR1 + 2.
* For the secondary channel, ALTSTATUS/CONTROL port is BAR3 + 2.

We can now say that each channel has 13 registers. For the primary channel, we use these values:
* Data Register: BAR0 + 0; // Read-Write
* Error Register: BAR0 + 1; // Read Only
* Features Register: BAR0 + 1; // Write Only
* SECCOUNT0: BAR0 + 2; // Read-Write
* LBA0: BAR0 + 3; // Read-Write
* LBA1: BAR0 + 4; // Read-Write
* LBA2: BAR0 + 5; // Read-Write
* HDDEVSEL: BAR0 + 6; // Read-Write, used to select a drive in the channel.
* Command Register: BAR0 + 7; // Write Only.
* Status Register: BAR0 + 7; // Read Only.
* Alternate Status Register: BAR1 + 2; // Read Only.
* Control Register: BAR1 + 2; // Write Only.
* DEVADDRESS: BAR1 + 3; // I don't know what is the benefit from this register.

The map above is the same with the secondary channel, but it uses BAR2 and BAR3 instead of BAR0 and BAR1.

<source lang="c">
// Channels:
#define      ATA_PRIMARY      0x00
#define      ATA_SECONDARY    0x01

// Directions:
#define      ATA_READ      0x00
#define      ATA_WRITE     0x01
</source>

We have defined everything needed by the driver, now lets move to an important part. We said that
* BAR0 is the start of the I/O ports used by the primary channel.
* BAR1 is the start of the I/O ports which control the primary channel.
* BAR2 is the start of the I/O ports used by secondary channel.
* BAR3 is the start of the I/O ports which control secondary channel.
* BAR4 is the start of 8 I/O ports controls the primary channel's Bus Master IDE.
* BAR4 + 8 is the Base of 8 I/O ports controls secondary channel's Bus Master IDE.

So we can make this global structure:
<source lang="c">
struct IDEChannelRegisters {
   unsigned short base;  // I/O Base.
   unsigned short ctrl;  // Control Base
   unsigned short bmide; // Bus Master IDE
   unsigned char  nIEN;  // nIEN (No Interrupt);
} channels[2];
</source>

We also need a buffer to read the identification space into, we need a variable that indicates if an irq is invoked or not, and finally we need an array of 6 words [12 bytes] for ATAPI Drives:

<source lang="c">
unsigned char ide_buf[2048] = {0};
volatile unsigned static char ide_irq_invoked = 0;
unsigned static char atapi_packet[12] = {0xA8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0};
</source>

We said the the IDE can contain up to 4 drives:

<source lang="c">
struct ide_device {
   unsigned char  Reserved;    // 0 (Empty) or 1 (This Drive really exists).
   unsigned char  Channel;     // 0 (Primary Channel) or 1 (Secondary Channel).
   unsigned char  Drive;       // 0 (Master Drive) or 1 (Slave Drive).
   unsigned short Type;        // 0: ATA, 1:ATAPI.
   unsigned short Signature;   // Drive Signature
   unsigned short Capabilities;// Features.
   unsigned int   CommandSets; // Command Sets Supported.
   unsigned int   Size;        // Size in Sectors.
   unsigned char  Model[41];   // Model in string.
} ide_devices[4];
</source>

When we read a register in a channel, like STATUS Register, it is easy to execute:

<source lang="c">
ide_read(channel, ATA_REG_STATUS);

unsigned char ide_read(unsigned char channel, unsigned char reg) {
   unsigned char result;
   if (reg > 0x07 && reg < 0x0C)
      ide_write(channel, ATA_REG_CONTROL, 0x80 | channels[channel].nIEN);
   if (reg < 0x08)
      result = inb(channels[channel].base + reg - 0x00);
   else if (reg < 0x0C)
      result = inb(channels[channel].base  + reg - 0x06);
   else if (reg < 0x0E)
      result = inb(channels[channel].ctrl  + reg - 0x0A);
   else if (reg < 0x16)
      result = inb(channels[channel].bmide + reg - 0x0E);
   if (reg > 0x07 && reg < 0x0C)
      ide_write(channel, ATA_REG_CONTROL, channels[channel].nIEN);
   return result;
}
</source>

We also need a function for writing to registers:
<source lang="c">
void ide_write(unsigned char channel, unsigned char reg, unsigned char data) {
   if (reg > 0x07 && reg < 0x0C)
      ide_write(channel, ATA_REG_CONTROL, 0x80 | channels[channel].nIEN);
   if (reg < 0x08)
      outb(channels[channel].base  + reg - 0x00, data);
   else if (reg < 0x0C)
      outb(channels[channel].base  + reg - 0x06, data);
   else if (reg < 0x0E)
      outb(channels[channel].ctrl  + reg - 0x0A, data);
   else if (reg < 0x16)
      outb(channels[channel].bmide + reg - 0x0E, data);
   if (reg > 0x07 && reg < 0x0C)
      ide_write(channel, ATA_REG_CONTROL, channels[channel].nIEN);
}
</source>

To read the identification space, we should read the Data Register as a double word 128 times. We can then copy them to our buffer.

<source lang="c">
void ide_read_buffer(unsigned char channel, unsigned char reg, unsigned int buffer,
                     unsigned int quads) {
   /* WARNING: This code contains a serious bug. The inline assembly trashes ES and
    *           ESP for all of the code the compiler generates between the inline
    *           assembly blocks.
    */
   if (reg > 0x07 && reg < 0x0C)
      ide_write(channel, ATA_REG_CONTROL, 0x80 | channels[channel].nIEN);
   asm("pushw %es; movw %ds, %ax; movw %ax, %es");
   if (reg < 0x08)
      insl(channels[channel].base  + reg - 0x00, buffer, quads);
   else if (reg < 0x0C)
      insl(channels[channel].base  + reg - 0x06, buffer, quads);
   else if (reg < 0x0E)
      insl(channels[channel].ctrl  + reg - 0x0A, buffer, quads);
   else if (reg < 0x16)
      insl(channels[channel].bmide + reg - 0x0E, buffer, quads);
   asm("popw %es;");
   if (reg > 0x07 && reg < 0x0C)
      ide_write(channel, ATA_REG_CONTROL, channels[channel].nIEN);
}
</source>

When we send a command, we should wait for 400 nanosecond, then read the Status port. If the Busy bit is on, we should read the status port again until the Busy bit is 0; then we can read the results of the command. This operation is called "Polling". We can also use IRQs instead of polling.

After many commands, if the Device Fault bit is set, there is a failure; if DRQ is not set, there is an error. If the ERR bit is set, there is an error which is described in Error port.

<source lang="c">
unsigned char ide_polling(unsigned char channel, unsigned int advanced_check) {

   // (I) Delay 400 nanosecond for BSY to be set:
   // -------------------------------------------------
   for(int i = 0; i < 4; i++)
      ide_read(channel, ATA_REG_ALTSTATUS); // Reading the Alternate Status port wastes 100ns; loop four times.

   // (II) Wait for BSY to be cleared:
   // -------------------------------------------------
   while (ide_read(channel, ATA_REG_STATUS) & ATA_SR_BSY)
      ; // Wait for BSY to be zero.

   if (advanced_check) {
      unsigned char state = ide_read(channel, ATA_REG_STATUS); // Read Status Register.

      // (III) Check For Errors:
      // -------------------------------------------------
      if (state & ATA_SR_ERR)
         return 2; // Error.

      // (IV) Check If Device fault:
      // -------------------------------------------------
      if (state & ATA_SR_DF)
         return 1; // Device Fault.

      // (V) Check DRQ:
      // -------------------------------------------------
      // BSY = 0; DF = 0; ERR = 0 so we should check for DRQ now.
      if ((state & ATA_SR_DRQ) == 0)
         return 3; // DRQ should be set

   }

   return 0; // No Error.

}
</source>

If there is an error, we have a function which prints errors on screen:
<source lang="c">
unsigned char ide_print_error(unsigned int drive, unsigned char err) {
   if (err == 0)
      return err;

   printk("IDE:");
   if (err == 1) {printk("- Device Fault\n     "); err = 19;}
   else if (err == 2) {
      unsigned char st = ide_read(ide_devices[drive].channel, ATA_REG_ERROR);
      if (st & ATA_ER_AMNF)   {printk("- No Address Mark Found\n     ");   err = 7;}
      if (st & ATA_ER_TK0NF)   {printk("- No Media or Media Error\n     ");   err = 3;}
      if (st & ATA_ER_ABRT)   {printk("- Command Aborted\n     ");      err = 20;}
      if (st & ATA_ER_MCR)   {printk("- No Media or Media Error\n     ");   err = 3;}
      if (st & ATA_ER_IDNF)   {printk("- ID mark not Found\n     ");      err = 21;}
      if (st & ATA_ER_MC)   {printk("- No Media or Media Error\n     ");   err = 3;}
      if (st & ATA_ER_UNC)   {printk("- Uncorrectable Data Error\n     ");   err = 22;}
      if (st & ATA_ER_BBK)   {printk("- Bad Sectors\n     ");       err = 13;}
   } else  if (err == 3)           {printk("- Reads Nothing\n     "); err = 23;}
     else  if (err == 4)  {printk("- Write Protected\n     "); err = 8;}
   printk("- [%s %s] %s\n",
      (const char *[]){"Primary", "Secondary"}[ide_devices[drive].channel], // Use the channel as an index into the array
      (const char *[]){"Master", "Slave"}[ide_devices[drive].drive], // Same as above, using the drive
      ide_devices[drive].model);

   return err;
}
</source>

Now let's return to the initialization function:
<source lang="c">
void ide_initialize(unsigned int BAR0, unsigned int BAR1, unsigned int BAR2, unsigned int BAR3,
unsigned int BAR4) {

   int j, k, count = 0;

   // 1- Detect I/O Ports which interface IDE Controller:
   channels[ATA_PRIMARY  ].base  = (BAR0 & 0xFFFFFFFC) + 0x1F0 * (!BAR0);
   channels[ATA_PRIMARY  ].ctrl  = (BAR1 & 0xFFFFFFFC) + 0x3F6 * (!BAR1);
   channels[ATA_SECONDARY].base  = (BAR2 & 0xFFFFFFFC) + 0x170 * (!BAR2);
   channels[ATA_SECONDARY].ctrl  = (BAR3 & 0xFFFFFFFC) + 0x376 * (!BAR3);
   channels[ATA_PRIMARY  ].bmide = (BAR4 & 0xFFFFFFFC) + 0; // Bus Master IDE
   channels[ATA_SECONDARY].bmide = (BAR4 & 0xFFFFFFFC) + 8; // Bus Master IDE
</source>
Then we should disable IRQs in both channels by setting bit 1 (nIEN) in the Control port:

<source lang="c">
   // 2- Disable IRQs:
   ide_write(ATA_PRIMARY  , ATA_REG_CONTROL, 2);
   ide_write(ATA_SECONDARY, ATA_REG_CONTROL, 2);
</source>

Now we need to check for drives which could be connected to each channel. We will select the master drive of each channel, and send the ATA_IDENTIFY command (which is supported by ATA Drives). If there's no error, there are values returned in registers which determine the type of Drive; if no drive is present, there will be strange values.

Notice that if bit 4 in HDDEVSEL is set to 1, we are selecting the slave drive, if set to 0, we are selecting the master drive.

<source lang="c">
   // 3- Detect ATA-ATAPI Devices:
   for (i = 0; i < 2; i++)
      for (j = 0; j < 2; j++) {

         unsigned char err = 0, type = IDE_ATA, status;
         ide_devices[count].Reserved = 0; // Assuming that no drive here.

         // (I) Select Drive:
         ide_write(i, ATA_REG_HDDEVSEL, 0xA0 | (j << 4)); // Select Drive.
         sleep(1); // Wait 1ms for drive select to work.

         // (II) Send ATA Identify Command:
         ide_write(i, ATA_REG_COMMAND, ATA_CMD_IDENTIFY);
         sleep(1); // This function should be implemented in your OS. which waits for 1 ms.
                   // it is based on System Timer Device Driver.

         // (III) Polling:
         if (ide_read(i, ATA_REG_STATUS) == 0) continue; // If Status = 0, No Device.

         while(1) {
            status = ide_read(i, ATA_REG_STATUS);
            if ((status & ATA_SR_ERR)) {err = 1; break;} // If Err, Device is not ATA.
            if (!(status & ATA_SR_BSY) && (status & ATA_SR_DRQ)) break; // Everything is right.
         }

         // (IV) Probe for ATAPI Devices:

         if (err != 0) {
            unsigned char cl = ide_read(i, ATA_REG_LBA1);
            unsigned char ch = ide_read(i, ATA_REG_LBA2);

            if (cl == 0x14 && ch == 0xEB)
               type = IDE_ATAPI;
            else if (cl == 0x69 && ch == 0x96)
               type = IDE_ATAPI;
            else
               continue; // Unknown Type (may not be a device).

            ide_write(i, ATA_REG_COMMAND, ATA_CMD_IDENTIFY_PACKET);
            sleep(1);
         }

         // (V) Read Identification Space of the Device:
         ide_read_buffer(i, ATA_REG_DATA, (unsigned int) ide_buf, 128);

         // (VI) Read Device Parameters:
         ide_devices[count].Reserved     = 1;
         ide_devices[count].Type         = type;
         ide_devices[count].Channel      = i;
         ide_devices[count].Drive        = j;
         ide_devices[count].Signature    = *((unsigned short *)(ide_buf + ATA_IDENT_DEVICETYPE));
         ide_devices[count].Capabilities = *((unsigned short *)(ide_buf + ATA_IDENT_CAPABILITIES));
         ide_devices[count].CommandSets  = *((unsigned int *)(ide_buf + ATA_IDENT_COMMANDSETS));

         // (VII) Get Size:
         if (ide_devices[count].CommandSets & (1 << 26))
            // Device uses 48-Bit Addressing:
            ide_devices[count].Size   = *((unsigned int *)(ide_buf + ATA_IDENT_MAX_LBA_EXT));
         else
            // Device uses CHS or 28-bit Addressing:
            ide_devices[count].Size   = *((unsigned int *)(ide_buf + ATA_IDENT_MAX_LBA));

         // (VIII) String indicates model of device (like Western Digital HDD and SONY DVD-RW...):
         for(k = 0; k < 40; k += 2) {
            ide_devices[count].Model[k] = ide_buf[ATA_IDENT_MODEL + k + 1];
            ide_devices[count].Model[k + 1] = ide_buf[ATA_IDENT_MODEL + k];}
         ide_devices[count].Model[40] = 0; // Terminate String.

         count++;
      }

   // 4- Print Summary:
   for (i = 0; i < 4; i++)
      if (ide_devices[i].Reserved == 1) {
         printk(" Found %s Drive %dGB - %s\n",
            (const char *[]){"ATA", "ATAPI"}[ide_devices[i].Type],         /* Type */
            ide_devices[i].Size / 1024 / 1024 / 2,               /* Size */
            ide_devices[i].Model);
      }
}
</source>

== Read/Write From ATA Drive ==
Now we're moving to a slightly more advanced part, it is to read and write from/to an ATA drive.
There is 3 ways of addressing a sector:
* CHS (Cylinder-Head-Sector): an old way of addressing sectors in ATA drives, all ATA drives should support this way of addressing.
* LBA28: Accessing a sector by its 28-bit LBA address. All ATA drives should support this way of addressing, the problem with LBA28 Addressing is that it only allows access 128GB to be accessed, so if the disk is bigger than 128GB, it should support the LBA48 Feature Set.
* LBA48: Accessing a sector by its 48-bit LBA address. As we use integers in GCC, our maximum address in this tutorial is 32-bit long, which allows accessing a drive with a size of up to 2TB.

So we can conclude an algorithm to determine which type of Addressing we are going to use:

<pre>
if (No LBA support)
   Use CHS.
else if (the LBA Sector Address > 0x0FFFFFFF)
   Use LBA48.
else
   Use LBA28.
</pre>

Reading the buffer may be done by polling or DMA.
PIO: After sending the command to read or write sectors, we read or write to the Data Port (as words). This is the same way of reading identification space.
DMA: After sending the command, you should wait for an IRQ, while you are waiting, Buffer is written directly to memory automatically.

We are going to use PIO as it is less complex.

We can conclude also this table:

<source lang="c">
   /* ATA/ATAPI Read/Write Modes:
    * ++++++++++++++++++++++++++++++++
    *  Addressing Modes:
    *  ================
    *   - LBA28 Mode.     (+)
    *   - LBA48 Mode.     (+)
    *   - CHS.            (+)
    *  Reading Modes:
    *  ================
    *   - PIO Modes (0 : 6)       (+) // Slower than DMA, but not a problem.
    *   - Single Word DMA Modes (0, 1, 2).
    *   - Double Word DMA Modes (0, 1, 2).
    *   - Ultra DMA Modes (0 : 6).
    *  Polling Modes:
    *  ================
    *   - IRQs
    *   - Polling Status   (+) // Suitable for Singletasking   
    */
</source>

There is something needed to be expressed here, I have told before that Task-File is like that:
* Register 0: [Word] Data Register. (Read-Write).
* Register 1: [Byte] Error Register. (Read).
* Register 1: [Byte] Features Register. (Write).
* Register 2: [Byte] SECCOUNT0 Register. (Read-Write).
* Register 3: [Byte] LBA0 Register. (Read-Write).
* Register 4: [Byte] LBA1 Register. (Read-Write).
* Register 5: [Byte] LBA2 Register. (Read-Write).
* Register 6: [Byte] HDDEVSEL Register. (Read-Write).
* Register 7: [Byte] Command Register. (Write).
* Register 7: [Byte] Status Register. (Read).

So each register between 2 to 5 should be 8-bits long. Really each of them are 16-bits long.

* Register 2: [Bits 0-7] SECCOUNT0, [Bits 8-15] SECOUNT1
* Register 3: [Bits 0-7] LBA0, [Bits 8-15] LBA3
* Register 4: [Bits 0-7] LBA1, [Bits 8-15] LBA4
* Register 5: [Bits 0-7] LBA2, [Bits 8-15] LBA5

The word [(SECCOUNT1 << 8) | SECCOUNT0] expresses the number of sectors which can be read when you access by LBA48.
When you access in CHS or LBA28, SECCOUNT0 only expresses number of sectors.

* LBA0 makes up bits 0 : 7 of the LBA address when you read in LBA28 or LBA48; it can also be the sector number of CHS.
* LBA1 makes up bits 8 : 15 of the LBA address when you read in LBA28 or LBA48; it can also be the low byte of the cylinder number of CHS.
* LBA2 makes up bits 16 : 23 of the LBA address when you read in LBA28 or LBA48; it can also be the high byte of the cylinder number of CHS.
* LBA3 makes up bits 24 : 31 of the LBA48 address.
* LBA4 makes up bits 32 : 39 of the LBA48 address.
* LBA5 makes up bits 40 : 47 of LBA48 address.

Notice that the LBA0, 1 and 2 registers are 24 bits long in total, which is not enough for LBA28; the higher 4-bits can be written to the lower 4-bits of the HDDEVSEL register.

Also notice that if bit 6 of this register  is set, we are going to use LBA, if not, we are going to use CHS. There is a mode which is called extended CHS.

Lets go into the code:

<source lang="c">
unsigned char ide_ata_access(unsigned char direction, unsigned char drive, unsigned int lba, 
                             unsigned char numsects, unsigned short selector, unsigned int edi) {
</source>

This function reads/writes sectors from ATA-Drive. If direction is 0 we are reading, else we are writing.
* drive is the drive number which can be from 0 to 3.
* lba is the LBA address which allows us to access disks up to 2TB.
* numsects is the number of sectors to be read, it is a char, as reading more than 256 sector immediately may performance issues. If numsects is 0, the ATA controller will know that we want 256 sectors.
* selector is the segment selector to read from, or write to.
* edi is the offset in that segment. (the memory address for the data buffer)

<source lang="c">
   unsigned char lba_mode /* 0: CHS, 1:LBA28, 2: LBA48 */, dma /* 0: No DMA, 1: DMA */, cmd;
   unsigned char lba_io[6];
   unsigned int  channel      = ide_devices[drive].Channel; // Read the Channel.
   unsigned int  slavebit      = ide_devices[drive].Drive; // Read the Drive [Master/Slave]
   unsigned int  bus = channels[channel].Base; // Bus Base, like 0x1F0 which is also data port.
   unsigned int  words      = 256; // Almost every ATA drive has a sector-size of 512-byte.
   unsigned short cyl, i;
   unsigned char head, sect, err;
</source>

We don't need IRQs, so we should disable it to prevent problems from happening. We said before that if bit 1 of the Control Register (which is called nIEN bit), is set, no IRQs will be invoked from any drives on this channel, either master or slave.

<source lang="c">
ide_write(channel, ATA_REG_CONTROL, channels[channel].nIEN = (ide_irq_invoked = 0x0) + 0x02);
</source>

Now lets read the parameters:

<source lang="c">
   // (I) Select one from LBA28, LBA48 or CHS;
   if (lba >= 0x10000000) { // Sure Drive should support LBA in this case, or you are
                            // giving a wrong LBA.
      // LBA48:
      lba_mode  = 2;
      lba_io[0] = (lba & 0x000000FF) >> 0;
      lba_io[1] = (lba & 0x0000FF00) >> 8;
      lba_io[2] = (lba & 0x00FF0000) >> 16;
      lba_io[3] = (lba & 0xFF000000) >> 24;
      lba_io[4] = 0; // LBA28 is integer, so 32-bits are enough to access 2TB.
      lba_io[5] = 0; // LBA28 is integer, so 32-bits are enough to access 2TB.
      head      = 0; // Lower 4-bits of HDDEVSEL are not used here.
   } else if (ide_devices[drive].Capabilities & 0x200)  { // Drive supports LBA?
      // LBA28:
      lba_mode  = 1;
      lba_io[0] = (lba & 0x00000FF) >> 0;
      lba_io[1] = (lba & 0x000FF00) >> 8;
      lba_io[2] = (lba & 0x0FF0000) >> 16;
      lba_io[3] = 0; // These Registers are not used here.
      lba_io[4] = 0; // These Registers are not used here.
      lba_io[5] = 0; // These Registers are not used here.
      head      = (lba & 0xF000000) >> 24;
   } else {
      // CHS:
      lba_mode  = 0;
      sect      = (lba % 63) + 1;
      cyl       = (lba + 1  - sect) / (16 * 63);
      lba_io[0] = sect;
      lba_io[1] = (cyl >> 0) & 0xFF;
      lba_io[2] = (cyl >> 8) & 0xFF;
      lba_io[3] = 0;
      lba_io[4] = 0;
      lba_io[5] = 0;
      head      = (lba + 1  - sect) % (16 * 63) / (63); // Head number is written to HDDEVSEL lower 4-bits.
   }
</source>

Now we are going to choose the way of reading the buffer [PIO or DMA]:

<source lang="c">
   // (II) See if drive supports DMA or not;
   dma = 0; // We don't support DMA
</source>

Lets poll the Status port while the channel is busy:

<source lang="c">
   // (III) Wait if the drive is busy;
   while (ide_read(channel, ATA_REG_STATUS) & ATA_SR_BSY){   
   
   } // Wait if busy.
</source>

The HDDDEVSEL register now looks like this:

* Bits 0 : 3: Head Number for CHS.
* Bit 4: Slave Bit. (0: Selecting Master Drive, 1: Selecting Slave Drive).
* Bit 5: Obsolete and isn't used, but should be set.
* Bit 6: LBA (0: CHS, 1: LBA).
* Bit 7: Obsolete and isn't used, but should be set.

Lets write all these information to the register, while the obsolete bits are set (0xA0):

<source lang="c">
   // (IV) Select Drive from the controller;
   if (lba_mode == 0)
      ide_write(channel, ATA_REG_HDDEVSEL, 0xA0 | (slavebit << 4) | head); // Drive & CHS.
   else
      ide_write(channel, ATA_REG_HDDEVSEL, 0xE0 | (slavebit << 4) | head); // Drive & LBA
</source>

Let's write the parameters to registers:

<source lang="c">
   // (V) Write Parameters;
   if (lba_mode == 2) {
      ide_write(channel, ATA_REG_SECCOUNT1,   0);
      ide_write(channel, ATA_REG_LBA3,   lba_io[3]);
      ide_write(channel, ATA_REG_LBA4,   lba_io[4]);
      ide_write(channel, ATA_REG_LBA5,   lba_io[5]);
   }
   ide_write(channel, ATA_REG_SECCOUNT0,   numsects);
   ide_write(channel, ATA_REG_LBA0,   lba_io[0]);
   ide_write(channel, ATA_REG_LBA1,   lba_io[1]);
   ide_write(channel, ATA_REG_LBA2,   lba_io[2]);
</source>

If you are using LBA48 and want to write to the LBA0 and LBA3 registers, you should write LBA3 to Register 3, then write LBA0 to Register 3. ide_write function makes it quite simple, refer to the function and you will fully understand the code.

Now, we have a great set of commands described in ATA/ATAPI-8 Specification, we should choose the suitable command to execute:

<source lang="c">
   // (VI) Select the command and send it;
   // Routine that is followed:
   // If ( DMA & LBA48)   DO_DMA_EXT;
   // If ( DMA & LBA28)   DO_DMA_LBA;
   // If ( DMA & LBA28)   DO_DMA_CHS;
   // If (!DMA & LBA48)   DO_PIO_EXT;
   // If (!DMA & LBA28)   DO_PIO_LBA;
   // If (!DMA & !LBA#)   DO_PIO_CHS;
</source>

There isn't a command for doing CHS with DMA.

<source lang="c">
   if (lba_mode == 0 && dma == 0 && direction == 0) cmd = ATA_CMD_READ_PIO;
   if (lba_mode == 1 && dma == 0 && direction == 0) cmd = ATA_CMD_READ_PIO;   
   if (lba_mode == 2 && dma == 0 && direction == 0) cmd = ATA_CMD_READ_PIO_EXT;   
   if (lba_mode == 0 && dma == 1 && direction == 0) cmd = ATA_CMD_READ_DMA;
   if (lba_mode == 1 && dma == 1 && direction == 0) cmd = ATA_CMD_READ_DMA;
   if (lba_mode == 2 && dma == 1 && direction == 0) cmd = ATA_CMD_READ_DMA_EXT;
   if (lba_mode == 0 && dma == 0 && direction == 1) cmd = ATA_CMD_WRITE_PIO;
   if (lba_mode == 1 && dma == 0 && direction == 1) cmd = ATA_CMD_WRITE_PIO;
   if (lba_mode == 2 && dma == 0 && direction == 1) cmd = ATA_CMD_WRITE_PIO_EXT;
   if (lba_mode == 0 && dma == 1 && direction == 1) cmd = ATA_CMD_WRITE_DMA;
   if (lba_mode == 1 && dma == 1 && direction == 1) cmd = ATA_CMD_WRITE_DMA;
   if (lba_mode == 2 && dma == 1 && direction == 1) cmd = ATA_CMD_WRITE_DMA_EXT;
   ide_write(channel, ATA_REG_COMMAND, cmd);               // Send the Command.
</source>

This ATA_CMD_READ_PIO command is used for reading in LBA28 or CHS, and the IDE controller refers to bit 6 of the HDDEVSEL register to find out the mode of reading (LBA or CHS).

After sending the command, we should poll, then we read/write a sector, then we should poll, then we read/write a sector, until we read/write all sectors needed, if an error has happened, the function will return a specific error code.

Notice that after writing, we should execute the CACHE FLUSH command, and we should poll after it, but without checking for errors.

<source lang="c">
   if (dma)
      if (direction == 0);
         // DMA Read.
      else;
         // DMA Write.
   else
      if (direction == 0)
         // PIO Read.
      for (i = 0; i < numsects; i++) {
         if (err = ide_polling(channel, 1))
            return err; // Polling, set error and exit if there is.
         asm("pushw %es");
         asm("mov %%ax, %%es" : : "a"(selector));
         asm("rep insw" : : "c"(words), "d"(bus), "D"(edi)); // Receive Data.
         asm("popw %es");
         edi += (words*2);
      } else {
      // PIO Write.
         for (i = 0; i < numsects; i++) {
            ide_polling(channel, 0); // Polling.
            asm("pushw %ds");
            asm("mov %%ax, %%ds"::"a"(selector));
            asm("rep outsw"::"c"(words), "d"(bus), "S"(edi)); // Send Data
            asm("popw %ds");
            edi += (words*2);
         }
         ide_write(channel, ATA_REG_COMMAND, (char []) {   ATA_CMD_CACHE_FLUSH,
                        ATA_CMD_CACHE_FLUSH,
                        ATA_CMD_CACHE_FLUSH_EXT}[lba_mode]);
         ide_polling(channel, 0); // Polling.
      }

   return 0; // Easy, isn't it?
}
</source>

== Reading from an ATAPI Drive ==
Let's move to an easier part - reading from an ATAPI drive. I will not make the function that writes to an ATAPI drive, because writing to it is very complex and is outside of the scope of this tutorial.

An ATAPI drive is different from an ATA drive, as it uses the SCSI command set instead of the ATA command set. Parameters are sent as packets, therefore it's called the ATA Packet Interface [ATAPI].

Notice also that ATAPI drives always use IRQs and you can't disable them. We should create a function that waits for an IRQ:

<source lang="c">
void ide_wait_irq() {
   while (!ide_irq_invoked)
      ;
   ide_irq_invoked = 0;
}
</source>

When an IRQ happens, the following function should be executed by ISR:

<source lang="c">
void ide_irq() {
   ide_irq_invoked = 1;
}
</source>

ide_wait_irq will go into a while loop, which waits for the variable ide_irq_invoked to be set, then clears it.

<source lang="c">
unsigned char ide_atapi_read(unsigned char drive, unsigned int lba, unsigned char numsects,
          unsigned short selector, unsigned int edi) {
</source>

* drive is the drive number, which is from 0 to 3.
* lba is the LBA address.
* numsects is the number of sectors. It should always be 1, and if you want to read more than one sector, re-execute this function with th updated LBA address.
* selector is the Segment Selector.
* edi is the offset in the selector.

Let's read the parameters of the drive:

<source lang="c">
   unsigned int   channel  = ide_devices[drive].Channel;
   unsigned int   slavebit = ide_devices[drive].Drive;
   unsigned int   bus      = channels[channel].Base;
   unsigned int   words    = 1024; // Sector Size. ATAPI drives have a sector size of 2048 bytes.
   unsigned char  err;
   int i;
</source>

We need IRQs:

<source lang="c">
   // Enable IRQs:
   ide_write(channel, ATA_REG_CONTROL, channels[channel].nIEN = ide_irq_invoked = 0x0);
</source>

Let's setup the SCSI Packet, which is 6 words (12 bytes) long:

<source lang="c">
   // (I): Setup SCSI Packet:
   // ------------------------------------------------------------------
   atapi_packet[ 0] = ATAPI_CMD_READ;
   atapi_packet[ 1] = 0x0;
   atapi_packet[ 2] = (lba >> 24) & 0xFF;
   atapi_packet[ 3] = (lba >> 16) & 0xFF;
   atapi_packet[ 4] = (lba >> 8) & 0xFF;
   atapi_packet[ 5] = (lba >> 0) & 0xFF;
   atapi_packet[ 6] = 0x0;
   atapi_packet[ 7] = 0x0;
   atapi_packet[ 8] = 0x0;
   atapi_packet[ 9] = numsects;
   atapi_packet[10] = 0x0;
   atapi_packet[11] = 0x0;
</source>

Now we should select the drive:

<source lang="c">
   // (II): Select the drive:
   // ------------------------------------------------------------------
   ide_write(channel, ATA_REG_HDDEVSEL, slavebit << 4);
</source>

400 nanoseconds delay after this select is a good idea:

<source lang="c">
   // (III): Delay 400 nanoseconds for select to complete:
   // ------------------------------------------------------------------
   for(int i = 0; i < 4; i++)
       ide_read(channel, ATA_REG_ALTSTATUS); // Reading the Alternate Status port wastes 100ns.
</source>

<source lang="c">
   // (IV): Inform the Controller that we use PIO mode:
   // ------------------------------------------------------------------
   ide_write(channel, ATA_REG_FEATURES, 0);         // PIO mode.
</source>

Tell the controller the size of the buffer

<source lang="c">
   // (V): Tell the Controller the size of buffer:
   // ------------------------------------------------------------------
   ide_write(channel, ATA_REG_LBA1, (words * 2) & 0xFF);   // Lower Byte of Sector Size.
   ide_write(channel, ATA_REG_LBA2, (words * 2) >> 8);   // Upper Byte of Sector Size.
</source>

Now that we want to send the packet, we should first send the command "Packet":

<source lang="c">
   // (VI): Send the Packet Command:
   // ------------------------------------------------------------------
   ide_write(channel, ATA_REG_COMMAND, ATA_CMD_PACKET);      // Send the Command.

   // (VII): Waiting for the driver to finish or return an error code:
   // ------------------------------------------------------------------
   if (err = ide_polling(channel, 1)) return err;         // Polling and return if error.

   // (VIII): Sending the packet data:
   // ------------------------------------------------------------------
   asm("rep   outsw" : : "c"(6), "d"(bus), "S"(atapi_packet));   // Send Packet Data
</source>

Here we cannot poll. We should wait for an IRQ, then read the sectors. These two operations should be repeated for each sector.

<source lang="c">
   // (IX): Receiving Data:
   // ------------------------------------------------------------------
   for (i = 0; i < numsects; i++) {
      ide_wait_irq();                  // Wait for an IRQ.
      if (err = ide_polling(channel, 1))
         return err;      // Polling and return if error.
      asm("pushw %es");
      asm("mov %%ax, %%es"::"a"(selector));
      asm("rep insw"::"c"(words), "d"(bus), "D"(edi));// Receive Data.
      asm("popw %es");
      edi += (words * 2);
   }
</source>

Now we should wait for an IRQ and poll until the Busy and DRQ bits are clear:

<source lang="c">
   // (X): Waiting for an IRQ:
   // ------------------------------------------------------------------
   ide_wait_irq();

   // (XI): Waiting for BSY & DRQ to clear:
   // ------------------------------------------------------------------
   while (ide_read(channel, ATA_REG_STATUS) & (ATA_SR_BSY | ATA_SR_DRQ))
      ;

   return 0; // Easy, ... Isn't it?
}
</source>

== Reading from an ATA/ATAPI Drive ==
<source lang="c">
void ide_read_sectors(unsigned char drive, unsigned char numsects, unsigned int lba,
                      unsigned short es, unsigned int edi) {

   // 1: Check if the drive presents:
   // ==================================
   if (drive > 3 || ide_devices[drive].Reserved == 0) package[0] = 0x1;      // Drive Not Found!

   // 2: Check if inputs are valid:
   // ==================================
   else if (((lba + numsects) > ide_devices[drive].Size) && (ide_devices[drive].Type == IDE_ATA))
      package[0] = 0x2;                     // Seeking to invalid position.

   // 3: Read in PIO Mode through Polling & IRQs:
   // ============================================
   else {
      unsigned char err;
      if (ide_devices[drive].Type == IDE_ATA)
         err = ide_ata_access(ATA_READ, drive, lba, numsects, es, edi);
      else if (ide_devices[drive].Type == IDE_ATAPI)
         for (i = 0; i < numsects; i++)
            err = ide_atapi_read(drive, lba + i, 1, es, edi + (i*2048));
      package[0] = ide_print_error(drive, err);
   }
}
// package[0] is an entry of an array. It contains the Error Code.
</source>

== Writing to an ATA drive ==
<source lang="c">
void ide_write_sectors(unsigned char drive, unsigned char numsects, unsigned int lba,
                       unsigned short es, unsigned int edi) {

   // 1: Check if the drive presents:
   // ==================================
   if (drive > 3 || ide_devices[drive].Reserved == 0)
      package[0] = 0x1;      // Drive Not Found!
   // 2: Check if inputs are valid:
   // ==================================
   else if (((lba + numsects) > ide_devices[drive].Size) && (ide_devices[drive].Type == IDE_ATA))
      package[0] = 0x2;                     // Seeking to invalid position.
   // 3: Read in PIO Mode through Polling & IRQs:
   // ============================================
   else {
      unsigned char err;
      if (ide_devices[drive].Type == IDE_ATA)
         err = ide_ata_access(ATA_WRITE, drive, lba, numsects, es, edi);
      else if (ide_devices[drive].Type == IDE_ATAPI)
         err = 4; // Write-Protected.
      package[0] = ide_print_error(drive, err);
   }
}
</source>

== Ejecting an ATAPI Drive ==
<source lang="c">
void ide_atapi_eject(unsigned char drive) {
   unsigned int   channel      = ide_devices[drive].Channel;
   unsigned int   slavebit      = ide_devices[drive].Drive;
   unsigned int   bus      = channels[channel].Base;
   unsigned int   words      = 2048 / 2;               // Sector Size in Words.
   unsigned char  err = 0;
   ide_irq_invoked = 0;

   // 1: Check if the drive presents:
   // ==================================
   if (drive > 3 || ide_devices[drive].Reserved == 0)
      package[0] = 0x1;      // Drive Not Found!
   // 2: Check if drive isn't ATAPI:
   // ==================================
   else if (ide_devices[drive].Type == IDE_ATA)
      package[0] = 20;         // Command Aborted.
   // 3: Eject ATAPI Driver:
   // ============================================
   else {
      // Enable IRQs:
      ide_write(channel, ATA_REG_CONTROL, channels[channel].nIEN = ide_irq_invoked = 0x0);

      // (I): Setup SCSI Packet:
      // ------------------------------------------------------------------
      atapi_packet[ 0] = ATAPI_CMD_EJECT;
      atapi_packet[ 1] = 0x00;
      atapi_packet[ 2] = 0x00;
      atapi_packet[ 3] = 0x00;
      atapi_packet[ 4] = 0x02;
      atapi_packet[ 5] = 0x00;
      atapi_packet[ 6] = 0x00;
      atapi_packet[ 7] = 0x00;
      atapi_packet[ 8] = 0x00;
      atapi_packet[ 9] = 0x00;
      atapi_packet[10] = 0x00;
      atapi_packet[11] = 0x00;

      // (II): Select the Drive:
      // ------------------------------------------------------------------
      ide_write(channel, ATA_REG_HDDEVSEL, slavebit << 4);

      // (III): Delay 400 nanosecond for select to complete:
      // ------------------------------------------------------------------
      for(int i = 0; i < 4; i++)
         ide_read(channel, ATA_REG_ALTSTATUS); // Reading Alternate Status Port wastes 100ns.

      // (IV): Send the Packet Command:
      // ------------------------------------------------------------------
      ide_write(channel, ATA_REG_COMMAND, ATA_CMD_PACKET);      // Send the Command.

      // (V): Waiting for the driver to finish or invoke an error:
      // ------------------------------------------------------------------
      err = ide_polling(channel, 1);            // Polling and stop if error.

      // (VI): Sending the packet data:
      // ------------------------------------------------------------------
      else {
         asm("rep   outsw"::"c"(6), "d"(bus), "S"(atapi_packet));// Send Packet Data
         ide_wait_irq();                  // Wait for an IRQ.
         err = ide_polling(channel, 1);            // Polling and get error code.
         if (err == 3) err = 0; // DRQ is not needed here.
      }
      package[0] = ide_print_error(drive, err); // Return;
   }
}
</source>

When this method is invoked, the optical device on the given channel is ejected.

== See Also ==
=== Wiki Pages ===
* [[MBR_(x86)|Master Boot Record (x86)]]
* [[Partition_Table|Partition Table (x86)]]

=== Threads ===
* [http://www.osdev.org/phpBB2/viewtopic.php?t=12268 How to w/r harddisk in pmode? (ASM Code from Dex)]
* [http://www.osdev.org/phpBB2/viewtopic.php?t=15314 ATA PIO code library (ASM code from XCHG)]
* [http://forum.osdev.org/viewtopic.php?f=1&p=167798#p167798 IDE Tutorial (C code from ''mostafazizo'')]

=== External Links ===
* [http://www.t13.org T13] -- The group that creates the ATA standard
* [http://www.ata-atapi.com ATA-ATAPI] -- Public Domain C driver sources (including SATA, Busmatering DMA, ATAPI), fairly good.
* [http://hddguru.com/content/en/documentation/ HDD Guru] -- The actual ATA specs from the first one that was released in 1994 to the 7th one in 2003.
* [http://www.ranish.com/part/primer.htm Partitioning Primer] -- A .HTM file containing some information about partitioning.
* [http://www.bswd.com/pciide.pdf PCI IDE Controller Specification] -- Specification containing information on "compatibility mode" and "PCI native mode" (and switching between them)
* [http://bswd.com/idems100.pdf Programming Interface for Bus Master IDE Controller] -- Bus Master IDE specification

[[Category:ATA]]
[[Category:Storage]]
[[de:AT Attachment]]
````

<a id="w002722"></a>

## W002722 — Talk:PCI IDE Controller

Bron-ID: W002722
Namespace: Talk (1)
Revisie: 27072 | Datum: 2022-05-07T15:10:46Z
Bron: https://wiki.osdev.org/Talk:PCI_IDE_Controller
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APCI_IDE_Controller&oldid=27072

````text
We already have a complete set of articles on ATA PIO Mode, ATAPI, SATA, and ATA DMA programming. Most of the programming information here is redundant, and is not as good as in the other articles.
<br>[[User:Bewing|Bewing]] 01:18, 9 November 2009 (UTC)

* Though true, I think the tutorial here is very useful. The other articles are pretty cryptic IMHO, so I certainly don't think we should throw this article away. [[User:Creature|Creature]] 16:06, 10 November 2009 (GMT + 1)

* If the article is to remain, it will have to be proof read.. spelling errors and other grammatical issues make it difficult to follow the authors code-commentary. --[[User:Brynet-inc|Brynet-Inc]] 16:39, 10 November 2009 (UTC)

** I'm trying to work my way through the article, but could use some help. I'm currently grammar checking the article and getting rid of obvious errors --[[User:Computafreak|Computafreak]] 22:06, 10 November 2009 (UTC)

* The article says that a PATA hard drive will use IRQs 14 and 15. Isn't it instead that the master uses 14 and the slave 15 (or vice versa)? [[User:Alethiophile|Alethiophile]] 00:44, 17 August 2010 (UTC)

** No, it goes by bus. ''Both'' drives on bus #0 (the primary bus) use IRQ14, and both drives on bus #1 (secondary) use IRQ15.<br>[[User:Bewing|Bewing]] 04:32, 19 August 2010 (UTC)

* According to the ATA PIO Mode page, LBA Bytes 4, 5 and 6 use the same ports as LBA 1, 2 and 3 respectively.  They are just sent before the lower order values.  This article states that 4, 5 and 6 get their own ports.  Can we verify which is correct? --[[User:Spydertl2|Spydertl2]] 02:14, 5 October 2010 (UTC)

** I think I've figured it out.  Later in the article, it clarifies the low/high byte issue with the ports.  I was looking at the register list, which differentiates between ports and "registers".  These enumerated "register" values are apparently converted to port numbers in code.  This should probably be explicitly noted in the article that these values are NOT port offsets. :) --[[User:Spydertl2|Spydertl2]]

* This article appears to have plenty of information relating to IDE ports on PCI buses in particular.  Also it is the only place I could find where the individual IDE IO port registers are individually listed.  This page may repeat some things but it is not superfluous. --[[User:Michaelslade|Michaelslade]] 14:55, 29 January 2011 (UTC)

* Source to the Hale Landis driver is easier found through archive.org [https://web.archive.org/web/20120126181130/http://www.ata-atapi.com/download/mindrvr.zip mindrvr.zip] [[User:Neozeed|Neozeed]] 19:32, 18 April 2018 (CDT)

== Registry trashing ==

It should be noted that, this line:<br/>
asm("pushw %es; movw %ds, %ax; movw %ax, %es;");<br/><br/>
Will trash the ax register, it should probably be replaced with:<br/>
asm("pushw %es; pushw %ax; movw %ds, %ax; movw %ax, %es; popw %ax;");  --[[User:Tobba|Tobba]] 17:26, 6 June 2011 (UTC)

Irrelevent.  [[r]e]ax is the general purpose register.  The next thing that will happen is that a value off the stack will be stuck in ax so the if statement can be evaluated.  (Yes I know that any optimising compiler will probably use a different one, but an optimising compiler will be able to work out that ax is trashed and work around that)  [[User:Andyhhp|Andyhhp]] 22:58, 6 June 2011 (UTC)

:GCC for one doesn't: it can't interpret anything within asm blocks (do you really think they have a full blown assembly interpreter for each architecture) and must rely on its specification to find out what it does. It may discard the asm block, it may assume that ax is unmodified, but even worse, it will assume that esp is unmodified(!). This page has been disputed for a reason. - [[User:Combuster|Combuster]] 09:50, 8 June 2011 (UTC)

== Non-standard outb argument order ==

It should be noted, if someone decides to fix this article (it would be usefull if it was corrected), that the arguments to outb is in the wrong order, assuming the reader use the "standard" outb function from here: http://wiki.osdev.org/Inline_Assembly/Examples

== PCI BAR detection ==
The BAR detection here is (for the ''base'' register)
 <pre>
(BAR0&0xFFFFFFFC) + 0x1F0*(!BAR0)
</pre>
Is this really correct? It works with an IDE controller, but not a SATA in my IDE driver (based on this tutorial).
Plus, the values on MS calc after these operations are completely wrong. Not'ing the final answer and adding 1 seems the only way to get near correct.
:There is another problem with PCI BARs in this article. IO port BARs are 4-byte aligned and the first two bits are always "01". The ''ide_initialize'' routine does not take this into account and uses misaligned ATA control ports (0x3F6, 0x376).
:[[User:Tree5678|Tree5678]] 10:10, 7 May 2022 (CDT)

== Control Offset Incorrect ==
The Control Register offset in the article code examples appears to be wrong.  It suggests the Control Register is at offset 2 from the control base (0x0C-0x0A), but it's actually at offset 0.  This code:
<pre>
else if (reg < 0x0E)
  outb(channels[channel].ctrl  + reg - 0x0A, data);
</pre>
should be changed to:
<pre>
else if (reg < 0x0E)
  outb(channels[channel].ctrl  + reg - 0x0C, data);
</pre>
[[User:Giawa|Giawa]] 10:26, 29 December 2021 (CST)
````

<a id="w002723"></a>

## W002723 — Category:USB

Bron-ID: W002723
Namespace: Category (14)
Revisie: 9219 | Datum: 2009-11-16T18:06:15Z
Bron: https://wiki.osdev.org/Category:USB
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AUSB&oldid=9219

````text
A list of topics regarding [[USB]] and its subcomponents

[[Category:Buses]]
````

<a id="w002725"></a>

## W002725 — INT

Bron-ID: W002725
Namespace: Main (0)
Revisie: 9213 | Datum: 2009-11-14T19:00:27Z
Bron: https://wiki.osdev.org/INT
Revisie-URL: https://wiki.osdev.org/index.php?title=INT&oldid=9213
Doorverwijzing: Interrupts

````text
#REDIRECT [[Interrupts]]
````

<a id="w002727"></a>

## W002727 — Bochs VBE extensions

Bron-ID: W002727
Namespace: Main (0)
Revisie: 9222 | Datum: 2009-11-16T18:09:14Z
Bron: https://wiki.osdev.org/Bochs_VBE_extensions
Revisie-URL: https://wiki.osdev.org/index.php?title=Bochs_VBE_extensions&oldid=9222
Doorverwijzing: Bochs VBE Extensions

````text
#REDIRECT [[Bochs VBE Extensions]]
````

<a id="w002728"></a>

## W002728 — Talk:Bochs VBE extensions

Bron-ID: W002728
Namespace: Talk (1)
Revisie: 9224 | Datum: 2009-11-16T18:09:14Z
Bron: https://wiki.osdev.org/Talk:Bochs_VBE_extensions
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABochs_VBE_extensions&oldid=9224
Doorverwijzing: Talk:Bochs VBE Extensions

````text
#REDIRECT [[Talk:Bochs VBE Extensions]]
````

<a id="w002729"></a>

## W002729 — Talk:C

Bron-ID: W002729
Namespace: Talk (1)
Revisie: 9228 | Datum: 2009-11-16T20:37:38Z
Bron: https://wiki.osdev.org/Talk:C
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AC&oldid=9228

````text
For the Learning C portion, shouldn't there be like a book reference or two? --[[User:Earlz|Earlz]] 20:37, 16 November 2009 (UTC)
````

<a id="w002730"></a>

## W002730 — Talk:Linker Scripts

Bron-ID: W002730
Namespace: Talk (1)
Revisie: 24202 | Datum: 2019-12-26T12:56:03Z
Bron: https://wiki.osdev.org/Talk:Linker_Scripts
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ALinker_Scripts&oldid=24202

````text
== Naming ==

Should the page say GNU ld, instead of simply "ld" since the linker script format is most likely GNU ld specific? - [[User:Sigjuice|Sigjuice]] 21:17, 16 November 2009 (UTC)
:Not really, as ld de facto refers to the GNU linker (and afaik no other linker is called ld). I decided to put it in anyway because it can do no harm and the word [[LD|ld]] alone is hardly noticeable as a hyperlink - [[User:Combuster|Combuster]] 18:14, 17 November 2009 (UTC)
The name "ld" happens to be quite generic and predates the GNU movement ([http://cm.bell-labs.com/cm/cs/who/dmr/man12.pdf Part 2, page 20, Unix Programmer's Manual November 3, 1971]). Also, most Unix-like systems have their own native linker, usually called ld. Here are four different versions of ld;  [https://publib.boulder.ibm.com/infocenter/pseries/v5r3/index.jsp?topic=/com.ibm.aix.cmds/doc/aixcmds3/ld.htm AIX], [http://developer.apple.com/mac/library/DOCUMENTATION/Darwin/Reference/ManPages/man1/ld.1.html <nowiki>Mac OS X</nowiki>],
[http://docs.sun.com/app/docs/doc/816-0210/6m6nb7mcs Solaris] and 
[http://docs.hp.com/en/B2355-90689/ld.1.html HP-UX]. - [[User:Sigjuice|Sigjuice]] 08:59, 18 November 2009 (UTC)

== Link to online manual ==

Take note that 2.9.1 predates 2.19 (or the currently up-to-date 2.21) by several years. The periods are significant in version numbering. -- [[User:Solar|Solar]] 12:08, 19 April 2011 (UTC)

== In progress ==

I just stumbled on this article and figured I could add a few things to improve it. I've done some editing to improve the encyclopedic tone and readability of the article. What needs to be added to make this a finished article on the subject? If anyone has any suggestions I'd be glad to do some work improving the article. --[[User:Ajxs|Ajxs]] 06:56, 26 December 2019 (CST)
````

<a id="w002732"></a>

## W002732 — User:Combuster/UDI Graphics

Bron-ID: W002732
Namespace: User (2)
Revisie: 16320 | Datum: 2014-04-27T17:52:38Z
Bron: https://wiki.osdev.org/User:Combuster/UDI_Graphics
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACombuster%2FUDI_Graphics&oldid=16320

````text
While this may be hijacking of the OS development wiki for now, its imo the best place for collaborative editing and public scrutiny.

Long story short, the last revival of UDI led to a movement that would be trying to make it viable - initially with hobbyist drivers, but also for a larger audience when it does pick up. I was invited (and I agreed) to draft a specification for an UDI sub-protocol, targeted at graphics drivers.

'''Please add comments''' - they can be used for error checking now, and as a justification later.
 
== Video card basics ==

There is only one "standard" video analogy that is in common use - the framebuffer. It is not alone however, there are still many devices that do not comprise a square grid of pixels, like text LED/LCD displays. And even when you do have a square array of pixels, it might not be possible to address and control each pixel individually and freely. The latter is especially the case in the default video mode during boot of desktop hardware - text mode. Also, in the console market tile-attribute engines are not uncommon - in fact they are the bigger brothers which have sprite engines. They have a certain amount of freedom in the location of images compared to the fixed grid of a tile-only engine, but lack the freedom that is provided by framebuffer engines.

The specification hopes to cover the vast majority of graphics hardware, without requiring too much special casing.

== Connectors ==

Graphics hardware serve no purpose if the result is not available to the user. Any device should thus have at least one output. On a desktop video card, the output is connected through a connector on the rear, VGA was the standard, DVI is the current. The choice is however not limited to these two. Industry video standards include PAL and NTSC, which are used in TV, and their data is transmitted in a signal of various forms: RF, composite, component, HDMI amongst others. A device has a number of outputs, each with a transfer type attached.
Some connectors (DVI/SCART) allow video data to be transferred in multiple formats. That implies there is no 1-on-1 mapping between connector types and signal types. When enumerating the outputs, the information returned should contain the type of connector, and the signal used over the connector.
Depending on the signal, sideband data may also be transferred - it may be DDC for VGA (with the purpose of identifying and manipulating the attached monitor), it may be audio (for HDMI, DVI, SCART, RF), it may be USB (DVI). The interface should thus support various combinations of side channels. Some video-related side channels should be included in the graphics specification.

A graphics device has one or more outputs, each has a connector type (which is constant over time), and a signal type (which is variable). The connectors to include are:

* VGA
* DVI
* S-video
* Component
* Composite
* RF
* SCART
* HDMI
* Hidden (when there is no physical connector, such as laptop screens)

The signals they carry:

* RGBHV (VGA, DVI-A)
* RGBS 
* RGsB 
* YPbPr (which is the common "Component" signal) 
* DVI-D
* Composite YUV (PAL)
* Composite YIQ (NTSC)
* Separated Y and UV (PAL S-Video)
* Separated Y and IQ (NTSC S-Video)
* HDMI
* Integrated (when the display device is inseparable from the graphics controller, including LED displays and emulated displays)
* Internal (proprietary)

Based on the signal and connector, software using the device may base choices such as resolution. Software may also change the signal used on each connector when applicable. Important to note is that this excludes the capabilities of the receiving end.

A connector can be queried for the signals it supports, and a signal can be selected. To implement specifics of a signal, a signal-specific parameter block containing the parameters can be passed. When a new signal is chosen for a connector, a default configuration for that signal is assumed to ease configuration. This configuration is designed to be as widely supported as possible.

For VGA or DVI-D signals this shall be 640 pixels by 480 progressive lines at 60Hz according to the original VGA standard. The configuration includes: 
* sync, frontportch, backporch, left overscan, right overscan and active display in both dimensions
* status of synchronisation pulses (positive, negative, disabled)
* the presence of a half interleaving line
* the pixel clock
For YUV signals, the signal shall match a PAL display.
For YIQ signals, the signal shall match an NTSC display.
Integrated protocols assume a implementation defined resolution. The only configuration option is the output resolution.

Connectors are to be listed according to probability of use. This ordering must be constant across instantiations of the driver in order to reliably track connected peripherals. Drivers may pose limitations on the ranges of configuration values that they deem valid.

== Video sources ==

Connectors require an input - colour data. This data is generated from the internal state of the graphics controller. This state may be raw pixel data (in framebuffer engines), or it may be references to objects (in tile engines). An important observation is that most real instances share a integrated form of image generation - layering. The most obvious form in tile engines is the sprite layer present in 3rd generation consoles, where the background consists of tiled objects, and where another set of objects is positioned above it. This layering is also found in framebuffer devices, most obviously in the form of overlays, however the mouse pointer is in essence a single hardware-accelerated fixed-dimension sprite.

Hardware may have one or more display engines. Engines are connected together into display pipes (independent of the actual method of implementation, which usually isn't pipe-based). Display pipes may be wired to one or more connectors. A connector may however not be linked to more than one display pipe. The rules that determine whether a pipe may be connected or not is hardware-specific, and thus, implementation defined. A configuration that was once accepted must however be accepted in all future cases. 

Display pipes can be edited if the hardware supports it. In most cases however, the layout of the pipe is fixed, and certain components can only be turned on and off. Some properties of the pipe may only be modified when the pipe is not connected. The properties of any attached connector may also be locked by the connection.

=== Pipe Layouts ===
Each engine in the pipe combines its inputs, and the local state into the pipe's output. The result is a tree of operations, with each node applying its local data to the data it receives. It may choose to overwrite the data, it may choose to pass through the data unmodified, or it may alter the data in some form. In all cases, the information about the pipe and the engines must be complete enough so that the result can be computed '''exactly'''. This has to be the case, because the information seen on screen should match the expectations of the programmer.

== State manipulation ==

In order to render information to the screen, the graphics controller requires that its internal state is manipulated. In some cases, this state can only be modified directly by outside influences (like all older graphics cards do), more recent hardware have integrated processing units that can manipulate the state of the device with little to no intervention of the host system. In some configurations, direct access is very limited and the state can only be manipulated via operations on the graphics controller.

To level the playing field, a framebuffer configuration requires a SetPixel command, as this represents the basic unit. However, this approach has significant overhead and should be avoided unless there is no viable alternative.

When direct access is supported and the buffer has an identity mapping to the pixel or tile data, the device may expose a function that returns the physical offset and size of the framebuffer. If there is no such mapping, the function should yield an error.

== Reference ==

You can use this as a basis for creating your UDI implementation. Details follow...

All drivers must include the the following declaration in their udiprops.txt:

   requires udi_gfx 0x101

Each driver source code file must contain the following:

<source lang="c">
#define UDI_GFX_VERSION 0x101

#include <udi.h>        // Must come before udi_gfx.h
#include <udi_gfx.h>    // Contains the UDI GFX interfaces</source>

Versions that have the same major version number but a larger minor number are strict supersets of those with smaller minor numbers.

=== State changes ===
<source lang="c">
typedef udi_ubit8_t udi_gfx_state_op_t;
#define UDI_GFX_OP_WRITE 2                    // Write a state property
#define UDI_GFX_OP_READ 1                     // Read a state property
#define UDI_GFX_TARGET_CONNECTOR (0 << 2)     // The target is an output connector
#define UDI_GFX_TARGET_PIPE (1 << 2)          // The target is a component in a pipeline

typedef struct {
    udi_cb_t *gcb;                  // generic control block
    udi_ubit_32_t unit;             // the unit to address
    udi_ubit_32_t index;            // the property's index
    udi_ubit_32_t value;            // the property's value
    udi_gfx_state_op_t operation;   // general operation parameters
} udi_gfx_state_req_cb_t;
</source>

The following properties can be used:


<tt>#define UDI_GFX_PROP_ENABLE 0</tt>
:The connector or engine is enabled (nonzero) or disabled (zero). A disabled engine forwards all data from the previous stage unmodified. A disabled connector does not send any data over the connector. Drivers may power down the physical counterparts of disabled components to preserve power, and users should expect delays when enabling connectors or components representing framebuffers. Disabling is however not recommended for sprite layers, which may repeatedly be enabled and disabled. A disabled component can still have its state changed, and the driver must complete all commands and other state changes as expected, regardless of disabled or power state. The valid ranges reported for this property may be 1 (always enabled) or 0-1 (either enabled or disabled).

<tt>#define UDI_GFX_PROP_INPUT 1</tt>
:Points to the engine that is processed before this unit. In the case of a connector, it points to the last engine in a pipeline, and each engine points to the next engine in the sequence. A value of -1 indicates a source that only yields black pixels. Implementations must not allow cyclic structures. Changing this value may reallocate resources, and engines that are no longer referenced may lose their data (but not their state) when it is not part of any pipeline. If preservation is required, the ENABLE state should be used instead. Valid ranges includes one or more from the list of engines and -1 combined. In most cases, this property can not be modified.

<tt>#define UDI_GFX_PROP_WIDTH 2</tt>
:Contains the amount of pixels in the horizontal direction. For connectors, this is the amount of data pixels rendered horizontally. For engines, this is the width in pixels of the image. Pixels requested from an engine outside the range (0..width-1) are defined according to the CLIP property. In some cases, hardware may support only fixed combinations of width and height. In such cases, changing the width will also change the height to a corresponding valid number. Valid ranges include any values strictly above zero. For connectors, expect large continuous ranges, large ranges with a certain modulus, a limited number of fixed values, or a constant value.

<tt>#define UDI_GFX_PROP_HEIGHT 3</tt>
:Contains the amount of pixels in the vertical direction. Functions similar to the width property, but changing it will not alter the width property, and it's range at any time contains the valid range for the currently selected width.

<tt>#define UDI_GFX_PROP_CLIP 4</tt>
:For engines, contains the behaviour for pixels requested outside the width and height of the engine. Can be either 0 (pass from next stage), 1 (the coordinates are wrapped modulus the height and width), 2 (the coordinates overflow onto the next scanline horizontally, and wrap vertically), 3 (the coordinates overflow onto the next column vertically, and wrap horizontally). Valid ranges contain one or more of these options. For overlays and sprites, a value 0 is common. For framebuffers, 2 is the most common value. For connectors, this property is always 0

<tt>#define UDI_GFX_PROP_UNIT_WIDTH 5</tt>
:For engines, represents the width in pixels for each entry in the buffer. For framebuffers, this is normally either 1 (1 pixel per memory entry) or two (pixel doubled). For tile engines, this is the width of each individual tile. As an example, a driver for a VGA compatible card supports modes that either are 1 pixel wide (4-bit color), 2 pixels wide (8-bit color; data of two pixels are used to generate one higher-depth pixel), 8 and 9 (in alphanumeric mode, where each character is 8 wide, with an optional 9th line). Attempts to change this number may change the width and height properties when they must be a multiple of the unit sizes. UNIT_HEIGHT may also be changed according to hardware restrictions. Valid values include any positive number.

<tt>#define UDI_GFX_PROP_UNIT_HEIGHT 6</tt>
:For engines, represents the height in pixels for each entry in the buffer. For framebuffers, this is normally either 1 (1 pixel per memory entry) or two (doublescanned). For tile engines, this is the height of each individual tile. See also UNIT_WIDTH. Valid values include any positive number.

<tt>#define UDI_GFX_PROP_TILESHEET 7</tt>
:For engines, represents a texture object that contains the tile data. This object can be used as the target of a glTexSubImage2D command, and can be selected for reading using the glReadBuffer command, for which it is a valid argument. It's height equals the UNIT_HEIGHT; its width equals UNIT_WIDTH times the maximal addressable tile. Multiple tilesheets may exist. Changing this state will cause the output to use the new tilesheet. A value of zero indicates that no tilesheet is present, and such a tilesheet can not be addressed by the format. Tilesheets which are not selected must be preserved during the lifetime of the pipeline. Tilesheets may be read-only, in part or in whole. Valid values are zero, or any pointers to unique tilesheets, they can't point to objects that might be returned by a glGenTextures command, or any other command that yields a hardware resource.

<tt>#define UDI_GFX_PROP_PALETTE 8</tt>
:For engines, represents a texture object that contains the palette information. This object can be used as the target of a glTexSubImage2D command, and can be selected for reading using the glReadBuffer command, for which it is a valid argument. In contrast to common palettes, not all entries are required to be of the same format. Valid values are any pointers to unique palettes, they can't point to objects that might be returned by a glGenTextures command, or any other command that yields a hardware resource. Palettes can be read-write or read-only, partially or in whole. Valid values are zero, or any pointers to unique tilesheets, they can't point to objects that might be returned by a glGenTextures command, or any other command that yields a hardware resource.

<tt>#define UDI_GFX_PROP_BUFFER 9</tt>
:For engines, represents the texture that contains the currently visible data. This texture is often different from the currently active framebuffer, which is the target of rendering calls. It's value is arbitrary, other than that different engines may point to the same buffer, in which case it is a shared resource. This is often the case where multiple pipelines are active. This state is commonly read-only. All numbers are valid values for this state.

=== Metalanguage definition ===
<source lang="c">typedef void (udi_gfx_command_cb_t *) udi_gfx_command_op_t;
typedef void (udi_gfx_request_cb_t *) udi_gfx_request_op_t;
typedef void (udi_gfx_state_cb_t *) udi_gfx_state_req_op_t;
typedef void (udi_gfx_range_cb_t *) udi_gfx_get_range_op_t;

// the metalanguage
typedef const struct {
    udi_gfx_command_op_t *gfx_command_op;       // send commands to change data
    udi_gfx_request_op_t *gfx_request_op;       // send commands to read data
    udi_gfx_state_req_op_t *gfx_state_req_op;   // set or get socket or pipe state
    udi_gfx_get_range_op_t *gfx_get_range_op;   // read valid socket and pipe values
} udi_gfx_device_ops_t;</source>

== See Also ==
* [http://www.d-rift.nl/combuster/mos3/?p=viewsource&file=/include/common/udi_gfx.h udi_gfx.h] - nightly version
````

<a id="w002733"></a>

## W002733 — File:GDT Entry.png

Bron-ID: W002733
Namespace: File (6)
Revisie: 9253 | Datum: 2009-11-24T02:27:04Z
Bron: https://wiki.osdev.org/File:GDT_Entry.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AGDT_Entry.png&oldid=9253

````text
Updated version of [[Image:Gdt_entry.png]]
Created in Inkscape and converted from SVG
````

<a id="w002734"></a>

## W002734 — User:Computafreak/DynamicLinking

Bron-ID: W002734
Namespace: User (2)
Revisie: 9267 | Datum: 2009-11-25T19:49:18Z
Bron: https://wiki.osdev.org/User:Computafreak/DynamicLinking
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AComputafreak%2FDynamicLinking&oldid=9267

````text
The process of dynamic linking is conceptually difficult to understand, but practically very simple. I'm assuming that you want to link ELF shared objects. The general sequence of events is to load the library as normal, load it into memory and patch the GOT. To be precise, you need to:
*Load the ELF object
*Look for the DYNAMIC section and process the values therein
**You'll want the hash table, global offset table, relocation tables (including the PLT) and dependencies
**Don't forget that these values are relative to the start of the file
*Get the symbol tables, by using the hash table
**Hashing is fairly simple, but you'll have to get the value at the hash table pointer to find out how many symbols are in the dynamic symbol table
**Again, hashing is optional, but you might as well make use of it for fast lookups
*Write two values to the GOT
**Integer one is a unique value (I use C++, so use the location of my shared object class in memory)
**Integer two is the location of a dynamic resolution method
*Process the relocations
**There doesn't seem to be any need to put in code for every relocation type, I've only come across types 6, 7 and 8
**Don't forget that any symbol values need to be absolute; the symbols in the dynamic symbol table are usually relative
*Load all the loadable segments into memory
*Do a special set of relocations on the PLT relocation entries.
**Go through every entry, and make the location at [relocationEntry->Offset + executableBase] absolute instead of relative
That sets everything up for dynamic linking. The dynamic resolution method is
<pre lang="asm">call resolveDynamicSymbol
pop ecx
pop ecx
jmp eax</pre>
Your resolveDynamicSymbol method just passes through to the relevant C or C++ method, which does the following
*Treats the passed integer as an offset (not an index) into the PLT relocation table to get a relocation entry
*Get the symbol from the dynamic symbol table
**This is probably an assumption, but seems to work for most libraries
*Get the symbol name from the dynamic string table
**As above
*Look through every loaded object that the shared object depends upon, trying to find the symbol name in their dynamic symbol tables
*If you find a match, write [symbolFound->Value + dependency->Base] to the memory location [relocationEntry->Offset + executableBase] and exit the loop
*Return the value that you've just written

This is just a rough guide; I couldn't find many resources about this when I was writing my linker, so there's probably some assumptions there. Also, please note that this is just one half of the code. If you want to implement dynamic linking in the kernel (as I did) then you need to patch the kernel's GOT to do pretty much the same thing. The only difference is that you get the value from the Multiboot header
````

<a id="w002735"></a>

## W002735 — User:Kotuxguy

Bron-ID: W002735
Namespace: User (2)
Revisie: 19475 | Datum: 2016-07-14T21:23:20Z
Bron: https://wiki.osdev.org/User:Kotuxguy
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKotuxguy&oldid=19475

````text
Hey! I'm Kotuxguy,a (sort of) expert programmer with ''tons'' of interests.
My languages of expertise:

* C/C++     ''The language im most experienced in(4 years)''
* Java      ''I don't use Java much,but im pretty good with it(3 years)''
* Python    ''Just dabbling in it for now''

Total,i have about 7 years of experience.


== Contributions ==

[[GRUB]] I wrote most of this page.
````

<a id="w002736"></a>

## W002736 — File:Ix86-callgate-desc.png

Bron-ID: W002736
Namespace: File (6)
Revisie: 9278 | Datum: 2009-11-28T13:17:27Z
Bron: https://wiki.osdev.org/File:Ix86-callgate-desc.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AIx86-callgate-desc.png&oldid=9278

````text
Callgate descriptor format.
````

<a id="w002737"></a>

## W002737 — File:Ix86-code-seg-desc.png

Bron-ID: W002737
Namespace: File (6)
Revisie: 9279 | Datum: 2009-11-28T13:18:12Z
Bron: https://wiki.osdev.org/File:Ix86-code-seg-desc.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AIx86-code-seg-desc.png&oldid=9279

````text
Code segment descriptor format.
````

<a id="w002738"></a>

## W002738 — File:Ix86-data-seg-desc.png

Bron-ID: W002738
Namespace: File (6)
Revisie: 9280 | Datum: 2009-11-28T13:19:11Z
Bron: https://wiki.osdev.org/File:Ix86-data-seg-desc.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AIx86-data-seg-desc.png&oldid=9280

````text
Data segment descriptor format.
````

<a id="w002739"></a>

## W002739 — File:Ix86-ldt-desc.png

Bron-ID: W002739
Namespace: File (6)
Revisie: 9281 | Datum: 2009-11-28T13:20:09Z
Bron: https://wiki.osdev.org/File:Ix86-ldt-desc.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AIx86-ldt-desc.png&oldid=9281

````text
Local Descriptor Table (LDT) descriptor format.
````

<a id="w002740"></a>

## W002740 — File:Ix86-task-gate-desc.png

Bron-ID: W002740
Namespace: File (6)
Revisie: 9282 | Datum: 2009-11-28T13:20:49Z
Bron: https://wiki.osdev.org/File:Ix86-task-gate-desc.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AIx86-task-gate-desc.png&oldid=9282

````text
Local Descriptor Table (LDT) descriptor format.
````

<a id="w002741"></a>

## W002741 — File:Ix86-int-trap-gate-desc.png

Bron-ID: W002741
Namespace: File (6)
Revisie: 9283 | Datum: 2009-11-28T13:22:11Z
Bron: https://wiki.osdev.org/File:Ix86-int-trap-gate-desc.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AIx86-int-trap-gate-desc.png&oldid=9283

````text
Interrupt / Trap Gate descriptor format.
````

<a id="w002742"></a>

## W002742 — File:Ix86-tss-desc.png

Bron-ID: W002742
Namespace: File (6)
Revisie: 9284 | Datum: 2009-11-28T13:23:02Z
Bron: https://wiki.osdev.org/File:Ix86-tss-desc.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AIx86-tss-desc.png&oldid=9284

````text
Task State Segment descriptor format.
````

<a id="w002743"></a>

## W002743 — User talk:Teodor väänänen

Bron-ID: W002743
Namespace: User talk (3)
Revisie: 9286 | Datum: 2009-11-28T13:29:21Z
Bron: https://wiki.osdev.org/User_talk:Teodor_v%C3%A4%C3%A4n%C3%A4nen
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ATeodor_v%C3%A4%C3%A4n%C3%A4nen&oldid=9286

````text
Uploaded a bunch of images of the various descriptor table entries, because of the issue with the format being off with the current ones. 'V' in the fields of the system descriptors (non-code, non-data) stands for 'Version', meaning 16 (clear(0)) or 32 (set(1)) bit version. 'B' in the TSS descriptor image is the busy bit, and the other bits should be known by most.

Oh, and if anyone wonders, I HEREBY RELEASE THESE IMAGES TO THE PUBLIC DOMAIN. Enjoy.

/[[User:Teodor väänänen|Teodor väänänen]]
````

<a id="w002744"></a>

## W002744 — User:Patrickv

Bron-ID: W002744
Namespace: User (2)
Revisie: 9346 | Datum: 2009-12-06T21:24:19Z
Bron: https://wiki.osdev.org/User:Patrickv
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APatrickv&oldid=9346

````text
Hello Everyone
````

<a id="w002745"></a>

## W002745 — Category:Itanium2

Bron-ID: W002745
Namespace: Category (14)
Revisie: 9348 | Datum: 2009-12-06T22:43:19Z
Bron: https://wiki.osdev.org/Category:Itanium2
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AItanium2&oldid=9348

````text
The Intel Itanium architecture (formerly known as IA-64) is a 64-bit platform used in high-end computing (eg: scientific research) and high-end, enterprise servers.  The architecture is based around the idea of instruction-level parallelism.  More specifically, it utilizes E.P.I.C. (Explicitly Parallel Instruction Computing).

Instructions are sent to the processor in "bundles", with 3 instructions per bundle.  Each bundle is 128 bits in width with a 5-bit template field specifying the encapsulated instructions.  Each instruction is an awkward 41 bits in size and certain bundles contain "paired" instructions known as LX Units (allowing for a 64-bit immediate).  

[[Category:Platforms]]
````

<a id="w002746"></a>

## W002746 — Itanium2 Overview

Bron-ID: W002746
Namespace: Main (0)
Revisie: 9349 | Datum: 2009-12-07T01:15:03Z
Bron: https://wiki.osdev.org/Itanium2_Overview
Revisie-URL: https://wiki.osdev.org/index.php?title=Itanium2_Overview&oldid=9349

````text
The Itanium and Itanium2 utilize the E.P.I.C. (Explicitly Parallel Instruction Computing) architecture. The instruction pipeline in the Itanium2 is 8 stages deep.  The first 2 (IPG and ROT) are the "front end" and the remaining 6 (EXP, REN, REG, EXE, DET, and WRB) make up the "back end" of the instruction pipeline.  

Instructions are packed into 128-bit bundles.  Each bundle has a 5-bit template specifier that notifies the CPU of what each of the encapsulated instructions are.  Each instruction is 41-bits wide and can be paired up in some bundle types (called LX Units) to form an 82-bit instruction (to allow for full 64-bit immediate values).  There are 12 non-reserved bundle template formats specified by Intel.  
 
There are 5 different instruction types: ALU (Arithmetic Logic Unit), M (Memory), I (non-ALU Integer), F (Floating-point) and B (Branch).  There is another logical type that is actually just an extension of one of those other 5 types called LX (Extended).  

The Itanium2 can execute up to 6 instructions per clock cycle (2 full instruction bundles) in parallel.  Sometimes issuing 6 instructions in parallel is just not possible, these are called "split issues".  Split issues occur when an explicit "stop" is set by software (it lets the CPU know that the upcoming instruction needs to be issued next cycle) or by the CPU running out of usable resources for an instruction to use. 

== References ==


[[Category:Itanium2]]
````

<a id="w002747"></a>

## W002747 — Uniform Driver Interface

Bron-ID: W002747
Namespace: Main (0)
Revisie: 19808 | Datum: 2016-09-08T22:35:45Z
Bron: https://wiki.osdev.org/Uniform_Driver_Interface
Revisie-URL: https://wiki.osdev.org/index.php?title=Uniform_Driver_Interface&oldid=19808

````text
[[Category:Uniform Driver Interface]]
[[Category:Drivers]]
[[Category:Driver Interfaces]]

[[Image:Udi_color_330x220.jpg‎|thumb|alt=Logo|The official Project UDI logo]]

'''The UDI revival effort maintains an IRC channel on Freenode (irc.freenode.net), called #udi'''. Feel free to join and ask questions.

UDI stands for "Uniform Driver Interface". It is the specification of a framework and driver API / ABI that enables different operating systems (implementing the UDI framework) to use the same drivers. Conceived by several large industry corporations, it has fallen dormant, despite being functional and delivering on its promise.

UDI drivers are binary compatible across all UDI-compliant operating systems running on the same CPU family. They are also source compatible across all UDI-implementing operating systems. This means, a driver only has to be developed once.

While Microsoft Windows gets all the hardware drivers they want, and GNU discourages UDI for [http://www.gnu.org/philosophy/udi.html philosophical reasons], its advantages for hobbyist OS developers are obvious.

==Why UDI?==

The Uniform Driver Interface would, should it be widely adopted, provide a common driver framework for implementation across kernels and platforms, enabling drivers to be provided without respect to the target kernel, and to a large extent, the target hardware platform. UDI has several projected advantages over existing driver interfaces which may motivate the reader to adopt it:

===Advantages===
* Portability (both cross-OS and cross-platform), which was mentioned in the above section, is perhaps the primary concern for which UDI was developed in the first place. All we can hope for is that enough operating systems will embrace the model so we can actually take advantage of it.
* Performance is comparable or better than that of custom, native API drivers for a native UDI implementation. For environments where performance is critical, UDI does not inhibit service quality. UDI is explicitly designed to be non-blocking and lockless, featuring a synchronization model for increased MP scalability without locking and many other high-scalability focused features.
* UDI can integrate seamlessly into existing kernel environments regardless of the OS architecture ([[Monolithic Kernel|monolithic kernel]] vs. [[microkernel]], POSIX vs. non-POSIX, etc.) with little or no extra performance overhead.
* Reliability and stability have been explicitly provided for by the design. UDI tries to eliminate some categories of potential bugs, such as (but not limited to) resource leaks and deadlocks (all interfaces can potentially be implemented without any locking at all).
* Flexibility is another thing UDI has been designed mind with: not only in the way the specification was conceived (i.e., to be extensible), but also in the sense that it permits system programmers to apply techniques such as driver isolation, shadow drivers, etc. if they see fit to do so.
* The interface is fully asynchronous, in every respect; high scaling systems are becoming increasingly predominant and asynchronicity is slowly becoming an "expected" feature for modern kernels. UDI moves ahead of the herd to enable a compliant kernel to slowly adopt asynchronous interfaces without having to do major redesign later on.

===Disadvantages===
* Moderately complex, and it will generally take a while to understand the specification.
* It cannot simply be ported to immature kernels. A kernel must have a certain minimum level of maturity to reasonably attempt to become UDI compliant.
* Not at all viable for "casual" projects: requires a significant amount of foreknowledge and prior work.

==Core components of UDI Drivers==

[[Image:Core_spec-8.gif‎|left|frame|alt=Environment|High level view of UDI environments]]

An implementation of the Uniform Driver Interface specification is known as a UDI Environment. There is a reference implementation available (see link below) which provides usable code for several existing kernels (Linux, BSD, Solaris), and it can be used the basis for a fresh implementation. Kernel environment implementations are responsible for providing the Service Call interfaces specified by the UDI Standard; a kernel may choose to implement these as native system calls, or via library extensions -- the decision is up to the implementer. There are two types of service calls recognized by the UDI paradigm: synchronous (which will return immediately to the caller - i.e., to the driver) and asynchronous (which work through a callback mechanism).

UDI drivers also actively take part in identifying their child devices and helping to build the host kernel's device tree. Bus drivers enumerate children, and so on. Each of these buses may have several controllers attached. UDI drivers for these devices will interact in a tree-like fashion just as the hardware does. Let's take a closer look at drivers themselves!

Drivers are split into one or more modules, and each module has at least once region. A driver that has been instantiated (executed, so to speak) uses IPC calls ("Channel operations") to communicate between modules and regions. If a driver is used to instantiate more than one device (say, a disk driver used to instantiate two separate disk devices), the choice of whether the actual driver code is mapped using Copy-on-Write, or duplicated in memory, etc is up to the environment.

===Modules===

A module is essentially a single executable code object. Specifically, drivers can be broken into multiple executables. A large driver that may only need to load certain components and may not need all of its code in memory all the time may be implemented as a multi-module driver. This partitioning of the driver code into modules is up to the driver vendor of course. Most UDI drivers are expected to be single-module drivers, but complex drivers such as graphics card drivers, etc may be best implemented as multi-module drivers. For example, if a graphics driver exports an OpenGL 3D API along with a Direct3D API, it is very likely that both front-ends have a lot of code behind them that would occupy a lot of memory should both be loaded. Most kernels will use ''either'' OpenGL ''or'' Direct3D, so if such a graphics driver was to split its OpenGL and Direct3D implementations into separate modules, this would enable kernels loading that driver to avoid allocating memory for the code and data for the API it isn't using.

===Regions===
Main article: [[ User:Gravaera/UDI_Regions | UDI Regions ]]

Regions are nothing more than blocks of related data. For example, a network card may have a set of register states that are specific to its send() function, and a different set of stats and variables specific to its receive() function. Data is explicitly separated into functionality regions in UDI. A region is nothing more than driver-allocated data for its state variables. The most intuitive way to split driver data is into functionality sub-components of the device in question. So a network card driver may choose to have a send region and a receive region. A graphics driver write may choose to partition the driver into a framebuffer writing region, a transformation region, etc, etc. Then IPC request messages can be sent over UDI IPC channels to each region based on the purpose of that region.

Regions also form the unit of concurrent execution in UDI. Since Regions are nothing more than data, they are also the units which must be synchronized against concurrent writes. Generally, this means making sure that no two threads can modify region data at once. The design of the UDI interfaces is perfectly capable of working without the use of locking, and it is left up to the host OS Environment to choose whether it will use lockless algorithms, spinlocks, waitqueues, or some other method to ensure that no two threads modify region data at the same time. See the main article for a detailed explanation of several practically usable UDI synchronization models.

Another attribute of UDI regions is that they are location- and instance-independent, meaning that they can be moved from one place to another without affecting any of the other regions because they share no common state. That is, a driver can be marshaled and moved from one NUMA node to another, or one physical machine to another over a network, or any other similar type of migration. This is particularly interesting in multiprocessor systems (esp. NUMA), and high-scaling compute clusters because an environment may separate regions due to performance and resource constraints. It's worth mentioning that, because of the separate states, the tasks performed by regions are mutually-exclusive (for instance a network driver might have one region that handles sending packets and another receiving). This is a potential area where host OSs can make huge optimizations to remove performance bottlenecks.

===Channels===
Main article: [[ User:Gravaera/UDI_Channels | UDI Channels ]]

The only way for regions to communicate is through channels. Channels are an IPC-agnostic abstraction of a bi-directional communication mechanism. Each of the two channel endpoints provide an ops vector, which is a set of entry points. They are referenced via handles of type udi_channel_t (check the definition of handles below). The channel operations along with the associated functionality is defined by metalanguages. Metalanguages are separately defined for each class of drivers, but we'll get to that soon.

===Metalanguages===
Main article: [[ User:Gravaera/UDI_Metalanguages | Metalanguages ]]

Metalanguages define extensions to the core specification for various purposes, and can also be used to define custom IPC protocol APIs between modules/regions. An example of a case where a custom protocol API may be needed is where, for example, a network card driver has a "'''Control'''" region which takes commands from the kernel for power management ("Go to sleep", "prepare to shutdown", etc), and then it has a '''Send'''() region and '''Receive'''() region, which handle its send() and receive() functions respectively.

It follows naturally that if the driver receives a "Go to sleep" command from the kernel on its Control region, it would need to send messages to its Send and Receive regions to cause them to cease operation. There is no generic IPC_Send() function defined for IPC across UDI channels -- all IPC must be done according to the protocols APIs defined by a Metalanguage, whether standardized by the UDI spec, or custom-defined. Thankfully, driver writers do not need to define custom protocols for every such case where they want to simply send custom messages between regions: the UDI Core specification defines a "'''Generic I/O Metalanguage'''" IPC protocol API which covers a wide range of generic IPC needs and can be extended with custom messages as desired.

Apart from APIs/IPC protocols, Metalanguages also cover extensions to the core specification. For example, the already-defined UDI Bus/Bridge Metalanguage can be extended to support new buses as needed; PCI bus drivers, ISA bus drivers, etc do not all need new Metalanguages, because UDI has defined a core UDI Bus/Bridge Metalanguage. This core UDI Bus/Bridge Metalanguage can be extended using Bus/Bridge Metalanguage extensions specific to each bus. This is a case where a Metalanguage is already defined by the UDI Standard, and that metalanguage itself is extended as needed for each bus.

Entirely new Metalanguages can also be created where necessary; for example, an SCSI Host Bus Adapter is not really a bus, but it is an I/O Microcontroller device that acts as a parent device to SCSI devices (mostly disks). It looks like a bus, but isn't really a bus, and is better handled with an IPC protocol and API of its own. So the UDI specification defines an SCSI Host Bus Adapter Metalanguage API which manages communication (IPC) between SCSI Peripheral Devices (disks) and SCSI Host Bus Adapters. On any given motherboard, a commonly seen arrangement may be as follows in the ASCII art below. The SCSI HBA is not a bus, and the IPC communication between SCSI disks and the SCSI HBA cannot be constrained to follow the same format as communication between a bus and its child devices. This is a case where a new Metalanguage API for communication is a good idea.

As an honourable mention, it would also have been possible to just use the UDI Generic I/O Metalanguage for communication between the SCSI disks and their parent SCSI HBA -- the Generic I/O Metalanguage is equally adequate for that purpose as well.
<source lang="text">
RootNode
|- PCI-Bus-0
|  |- ...
|  +- ...
|
|- PCI-Bus-1
|  +- SCSI HBA
|     |- SCSI-Peripheral-0 (disk)
|     +- SCSI-Peripheral-1 (disk)
|
+- PCI-Bus-2
</source>

Metalanguages are essentially UDI IPC Channel protocol definitions or API definitions, and definitions of extensions to the core specification. Hence the name: Meta-''LANGUAGES''.

==Driver configuration==
There's a special configuration method for static properties of UDI drivers using a file called udiprops.txt. This file is distributed independently in each driver package for source code distributions and linked into a special section (called .udiprops) for binary distributions.

The udiprops.txt file doesn't only allow for static configuration options, but is also used in the building process for UDI drivers since they do not use makefiles - not that it would be technically unfeasible. The UDI specification defines tools for building, packaging and installing UDI drivers for simplicity's sake since, unlike POSIX tools, they don't require operating systems to have any extra functionality (e.g., a VFS). Luckly, these tools are available in the reference implementation, all you need to do is build them.

Below you can see a sample udiprops.txt:

   properties_version 0x101
   
   message 1 Project UDI
   message 2 http://www.project-UDI.org/participants.html
   message 3 Pseudo-Driver
   message 4 Generic UDI Pseudo-Driver
   release 3 1.01
   
   supplier	1
   contact	2
   name		3
   shortname	pseudod
   
   ##
   ## Interface dependencies
   ##
   requires udi	 	0x101
   requires udi_gio 	0x101
   
   ##
   ## Build instructions.
   ##
   
   module pseudod
   compile_options -DPSEUDO_GIO_META=1
   source_files pseudo.c pseudo.h
   region 0
   
   ##
   ## Metalanguage usage
   ##
   
   meta 1 udi_gio		# Generic I/O Metalanguage
   
   child_bind_ops 1 0 1		# GIO meta, primary region, ops_index 1
   
   # Orphan driver; no device line
   
   #
   # Initialization, shutdown messages
   #
   message 1100  pseudod: devmgmt_req %d
   message 1500  pseudod: final_cleanup_req

Of course, udiprops.txt can be a lot more complex than this, I only wanted you to see what one looks like. You should check the specification for all compile options, statements and configuration options.

==Programming Model==
All UDI function calls are asynchronous in nature; this means that they implicitly do not block. A ''compliant'' UDI driver will always be implicitly non-blocking. Whether or not the ''host kernel'' supports non-blocking programming models is up to that kernel, and for any particular kernel, it may be necessary to use locking, mutexes and blocking. Naturally, for a kernel that fully supports a non-blocking, asynchronous model, UDI will simply scale seamlessly.

UDI drivers, because of their asynchronous nature, behave like servers to a large extent and they have very good throughput, owing to the fact that the driver itself will only block if the host kernel imposes a limitation on it. For a host kernel which does not have scaling limitations, UDI drivers will innately also scale without limitations -- the throughput of a ''compliant'' UDI driver is dependent solely on the limitations of the host kernel.

UDI drivers do not implicitly assume the use of locking, blocking, or any specific threading or synchronization model. They fit perfectly into any kind of host environment. As such, the UDI specification does not define any locking operations. It is completely possible for a host kernel to run UDI drivers locklessly.

==Driver failures==

When illegal behavior is detected by the environment, the misbehaving region will usually be region-killed and all neighouring regions will be notified. All channels to that regions will be closed and all resources owned by that region will be freed.

==See also==

* Combuster's effort on creating a [[User:Combuster/UDI_Graphics|graphics metalanguage]]
* Love4Boobies' page for several other [[User:Love4boobies|UDI drafts]]

==Existing Implementations==
* [http://projectudi.sf.net/ Reference implementation] - Mostly targeted at linux
* [http://github.com/thepowersgang/acess2 Acess2] - Mostly complete implementation (with network support)
* [http://www.d-rift.nl/combuster/mos3/ MOS3]

==External Links==

* [http://www.projectudi.org/ Official website]
* [http://projectudi.sf.net/ Reference implementation]
* [http://www.ties.org/deven/udi.html Deven Corzine's editorial]
* [http://uw714doc.xinuos.com/en/UDI_dwg/dwg_code_top.html UDI Driver Writer's Guide]
````

<a id="w002748"></a>

## W002748 — UDI

Bron-ID: W002748
Namespace: Main (0)
Revisie: 9360 | Datum: 2009-12-08T10:52:55Z
Bron: https://wiki.osdev.org/UDI
Revisie-URL: https://wiki.osdev.org/index.php?title=UDI&oldid=9360
Doorverwijzing: Uniform Driver Interface

````text
#REDIRECT [[Uniform_Driver_Interface]]
````

<a id="w002749"></a>

## W002749 — File:Core spec-8.gif

Bron-ID: W002749
Namespace: File (6)
Revisie: 9697 | Datum: 2010-01-31T22:25:12Z
Bron: https://wiki.osdev.org/File:Core_spec-8.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ACore_spec-8.gif&oldid=9697

````text
UDI environment
````

<a id="w002750"></a>

## W002750 — File:Udi color 330x220.jpg

Bron-ID: W002750
Namespace: File (6)
Revisie: 9365 | Datum: 2009-12-08T21:50:31Z
Bron: https://wiki.osdev.org/File:Udi_color_330x220.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AUdi_color_330x220.jpg&oldid=9365

````text
UDI logo
````

<a id="w002751"></a>

## W002751 — ASCII

Bron-ID: W002751
Namespace: Main (0)
Revisie: 30668 | Datum: 2026-05-29T09:02:41Z
Bron: https://wiki.osdev.org/ASCII
Revisie-URL: https://wiki.osdev.org/index.php?title=ASCII&oldid=30668

````text
{{In Progress}}

'''ASCII''' (American Standard Code for Information Interchange) is a text encoding standard used internationally as the basic human-readable encoding standard. The most commonly used (and the definitively standardized) form of ASCII is 7-bit ASCII, which uses characters <tt>0</tt> through <tt>0x7F</tt> (decimal <tt>127</tt>) to display all the basic Latin alphabet characters, plus Arabic numerals, international text symbols, and ASCII control codes.

== Table of 7-bit ASCII characters ==
{| class="wikitable"
|-
! &nbsp;
! 00
! 01
! 02
! 03
! 04
! 05
! 06
! 07
! 08
! 09
! 0A
! 0B
! 0C
! 0D
! 0E
! 0F
|-
! 00
| NUL
| SOH
| STX
| ETX
| EOT
| ENQ
| ACK
| BEL
| BS
| HT
| LF
| VT
| FF
| CR
| SO
| SI
|-
! 10
| DLE
| DC1
| DC2
| DC3
| DC4
| NAK
| SYN
| ETB
| CAN
| EM
| SUB
| ESC
| FS
| GS
| RS
| US
|-
! 20
| SP
| !
| "
| #
| $
| %
| &
| '
| (
| )
| *
| +
| ,
| -
| .
| /
|-
! 30
| 0
| 1
| 2
| 3
| 4
| 5
| 6
| 7
| 8
| 9
| :
| ;
| <
| =
| >
| ?
|-
! 40
| @
| A
| B
| C
| D
| E
| F
| G
| H
| I
| J
| K
| L
| M
| N
| O
|-
! 50
| P
| Q
| R
| S
| T
| U
| V
| W
| X
| Y
| Z
| [
| \
| ]
| ^
| _
|-
! 60
| `
| a
| b
| c
| d
| e
| f
| g
| h
| i
| j
| k
| l
| m
| n
| o
|-
! 70
| p
| q
| r
| s
| t
| u
| v
| w
| x
| y
| z
| {
| <nowiki>|</nowiki>
| }
| ~
| DEL

|}

== IBM Extended ASCII ==
IBM created an "Extended ASCII" character set to utilize the eighth bit provided by 8-bit bytes that was unused by the 7-bit ASCII standard. This meant another 128 possible character spaces were free and usable. The Extended ASCII set is the one used in the text mode of IBM-compatible [[x86]] [[BIOS|BIOSes]] and/or video cards. These characters include European language accented vowels, Greek symbols, and the IBM box drawing characters. Each of these "extended sets" are known as ''code pages''. On IBM-compatibles the standard code page for text mode is almost always CP437.

== See Also ==
=== External Links ===
* http://www.asciitable.com/ - tables of both standardized 7-bit and IBM Extended ASCII characters and numerical equivalents.
* http://www.theasciicode.com.ar/ - tables of ASCII characters and their graphical symbols (those used in VGA text mode). Also has information on the history of ASCII.

[[Category:Standards]]
[[Category:Character Encoding]]
````

<a id="w002754"></a>

## W002754 — Talk:BIOS

Bron-ID: W002754
Namespace: Talk (1)
Revisie: 23688 | Datum: 2019-06-12T16:53:47Z
Bron: https://wiki.osdev.org/Talk:BIOS
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABIOS&oldid=23688

````text
=== BIOS in Protected Mode ===
A few times the topic has appeared on the forums talking about software real-mode emulation to call the BIOS in protected and long mode. How this is achieved is currently outside my field of knowledge, but I think it would be a valuable addition if someone is able to add the following to the article;

1. Why can't BIOS functions be called in pmode?

2. What happens when they are called? (Beyond 'it crashes' - why? What is the CPU doing?)

3. How do software 'real mode emulators' work?

4. What are the minimum requirements of such an emulator in order to interact with the BIOS and how does it do so?

5. Could such an emulator dynamically recompile/wrap real-mode BIOS calls so you can call them from pmode/lmode natively?

--[[User:Messiahandrw|MessiahAndrw]] 02:56, 14 December 2009 (UTC)

Re 2: In real mode, at address given by [DS:AX] means "[(DS << 4) + AX]".  In protected mode, it means [{segment descriptor indicated by DS}.base + AX].  There is little chance these two will match (even less, since the code will probably try to put new values into DS, which are not going to resolve into proper descriptors.

The BIOS code is written assuming real mode.

[[User:Nedbrek|Nedbrek]] 04:01, 16 December 2009 (UTC)

These questions are now (mostly?) covered on the [[Virtual 8086 Mode]] and [[Protected mode]] pages.

[[User:Eekee|Eekee]] 11:53, 12 June 2019 (CDT)

=== Why don't we document the arguments to the common BIOS interrupts here? ===

Lots of places online only the interrupt number and ah value is listed. osdev.org should have a nice table with the arguments and some words about how the interrupt calls work.

Should it be this page or should there be a new page? Is it a problem of there being no real standard? Is it a copyright issue?

[[User:ask|ask]] 00:35, 09 June 2019 (UTC)

"The" exhaustive list which has been around for a long time is Ralph Brown's Interrupt List, AKA RBIL. Duplicating it would be rather pointless, an immense amount of work, and possibly treble the size of the size of the wiki. :) It's mentioned in this page with a link to its own page, which in turn links to several places to get it. When RBIL lacks info on an interrupt, it's because it's specific to certain manufacturers or even models of PC compatible, or even to a proprietary TSR.
[[User:Eekee|Eekee]] 11:48, 12 June 2019 (CDT)
````

<a id="w002755"></a>

## W002755 — User:Nedbrek

Bron-ID: W002755
Namespace: User (2)
Revisie: 11114 | Datum: 2011-01-22T19:38:46Z
Bron: https://wiki.osdev.org/User:Nedbrek
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANedbrek&oldid=11114

````text
Hello!

I am interested in OS development as a hobby.  I work in application software development (in C++), although in the past I have done microprocessor research.

My specialties are x86 and Itanium.

I am familiar with assembly language, and most C-like languages (C,C++,D, and some Java).

I am also interested in [http://www.tcl.tk Tcl].

==Ned OS==
===[[User:Nedbrek/bootl | Boot Loader]] - running===
===[[User:Nedbrek/mmap | Memory Map]]===

==Fixed Width Fonts==
* Good [http://www.cl.cam.ac.uk/~mgk25/ucs-fonts.html source] of fonts in BDF format.
* [[User:Nedbrek/Extracting bit patterns from BDF]]
````

<a id="w002756"></a>

## W002756 — MagicISO

Bron-ID: W002756
Namespace: Main (0)
Revisie: 16259 | Datum: 2014-04-24T05:22:58Z
Bron: https://wiki.osdev.org/MagicISO
Revisie-URL: https://wiki.osdev.org/index.php?title=MagicISO&oldid=16259

````text
MagicISO is a proprietary CD/DVD image editing utility. The unregistered version can save images up to 300MB in size, registration is required to save (but not open) larger images. Registration costs $US29.95. 

== Features ==
* Import and export boot images.

* Image sizes up to 300MB (unregistered version) or 10GB (registered version).

* Drag and drop files in and out of MagicISO.

* Burn images to CD/DVD, as well as create images from CD/DVD.

== Supported File Systems ==
* [[ISO 9660]]

* [[Rock Ridge]]

* Apple [[HFS]]/HFS+

* [[Joliet]]

* [[UDF]]

* XBOX DVD File System

== Supported Image Formats ==
* BIN/CUE

* BWT, BWI, BWS, BWA

* CCD/IMG/SUB

* CSO (.cso)

* DAA (.daa)

* MDF/MDS (.mdf)

* NRG (.nrg)

* DMG (Apple disk image) (.dmg)

* UIF (Universal Image Format) (.uif)

* IMA (.ima)

* IMG (.img)

* DK (.dk)

* WIM (.wim)

==See Also==
===External Links===
* [http://www.magiciso.com/ MagicISO homepage]

[[Category:Disk Image Utilities]]
````

<a id="w002757"></a>

## W002757 — CLI

Bron-ID: W002757
Namespace: Main (0)
Revisie: 28142 | Datum: 2023-07-10T03:18:52Z
Bron: https://wiki.osdev.org/CLI
Revisie-URL: https://wiki.osdev.org/index.php?title=CLI&oldid=28142

````text
CLI can refer to multiple things:

* [[Common Language Infrastructure]] of the .NET framework
* [[Command Line]] Interface
* [[CLI (Assembly)|CLear Interrupt]] flag - An assembly instruction to disable all interrupts

[[Category:OS theory]]
````

<a id="w002759"></a>

## W002759 — Visual C

Bron-ID: W002759
Namespace: Main (0)
Revisie: 9443 | Datum: 2009-12-20T05:46:52Z
Bron: https://wiki.osdev.org/Visual_C
Revisie-URL: https://wiki.osdev.org/index.php?title=Visual_C&oldid=9443
Doorverwijzing: Visual Studio

````text
#REDIRECT [[Visual Studio]]
````

<a id="w002760"></a>

## W002760 — Quick Links

Bron-ID: W002760
Namespace: Main (0)
Revisie: 9453 | Datum: 2009-12-20T14:22:05Z
Bron: https://wiki.osdev.org/Quick_Links
Revisie-URL: https://wiki.osdev.org/index.php?title=Quick_Links&oldid=9453
Doorverwijzing: Resources

````text
#REDIRECT [[Resources]]
````

<a id="w002761"></a>

## W002761 — Kernels With Source

Bron-ID: W002761
Namespace: Main (0)
Revisie: 9454 | Datum: 2009-12-20T14:23:56Z
Bron: https://wiki.osdev.org/Kernels_With_Source
Revisie-URL: https://wiki.osdev.org/index.php?title=Kernels_With_Source&oldid=9454
Doorverwijzing: Resources

````text
#REDIRECT [[Resources]]
````

<a id="w002762"></a>

## W002762 — Links

Bron-ID: W002762
Namespace: Main (0)
Revisie: 9455 | Datum: 2009-12-20T14:24:09Z
Bron: https://wiki.osdev.org/Links
Revisie-URL: https://wiki.osdev.org/index.php?title=Links&oldid=9455
Doorverwijzing: Resources

````text
#REDIRECT [[Resources]]
````

<a id="w002763"></a>

## W002763 — OSRC

Bron-ID: W002763
Namespace: Main (0)
Revisie: 9456 | Datum: 2009-12-20T14:24:22Z
Bron: https://wiki.osdev.org/OSRC
Revisie-URL: https://wiki.osdev.org/index.php?title=OSRC&oldid=9456
Doorverwijzing: Resources

````text
#REDIRECT [[Resources]]
````

<a id="w002764"></a>

## W002764 — Specifications

Bron-ID: W002764
Namespace: Main (0)
Revisie: 9457 | Datum: 2009-12-20T14:24:37Z
Bron: https://wiki.osdev.org/Specifications
Revisie-URL: https://wiki.osdev.org/index.php?title=Specifications&oldid=9457
Doorverwijzing: Resources

````text
#REDIRECT [[Resources]]
````

<a id="w002765"></a>

## W002765 — ATA

Bron-ID: W002765
Namespace: Main (0)
Revisie: 11548 | Datum: 2011-05-23T17:17:00Z
Bron: https://wiki.osdev.org/ATA
Revisie-URL: https://wiki.osdev.org/index.php?title=ATA&oldid=11548
Doorverwijzing: Category:ATA

````text
#REDIRECT [[:Category:ATA]]
````

<a id="w002767"></a>

## W002767 — Floppy disk

Bron-ID: W002767
Namespace: Main (0)
Revisie: 9465 | Datum: 2009-12-20T14:50:06Z
Bron: https://wiki.osdev.org/Floppy_disk
Revisie-URL: https://wiki.osdev.org/index.php?title=Floppy_disk&oldid=9465
Doorverwijzing: Floppy Disk Controller

````text
#REDIRECT [[Floppy Disk Controller]]
````

<a id="w002768"></a>

## W002768 — Hard disks

Bron-ID: W002768
Namespace: Main (0)
Revisie: 20689 | Datum: 2017-04-10T17:43:11Z
Bron: https://wiki.osdev.org/Hard_disks
Revisie-URL: https://wiki.osdev.org/index.php?title=Hard_disks&oldid=20689
Doorverwijzing: PCI IDE Controller

````text
#REDIRECT [[PCI IDE Controller]]
````

<a id="w002769"></a>

## W002769 — HFS

Bron-ID: W002769
Namespace: Main (0)
Revisie: 30689 | Datum: 2026-06-03T20:32:09Z
Bron: https://wiki.osdev.org/HFS
Revisie-URL: https://wiki.osdev.org/index.php?title=HFS&oldid=30689

````text
{{Historical}}
{{Stub}}
{{Filesystems}}

HFS is one of the older filesystems used by Apple for Mac OS, and has since been superseded with [[HFS+]]. You can get the HFS specifications from [http://developer.apple.com/legacy/mac/library/documentation/mac/Files/Files-99.html Apple]

[[Category:Filesystems]]
````

<a id="w002770"></a>

## W002770 — HFS+

Bron-ID: W002770
Namespace: Main (0)
Revisie: 18331 | Datum: 2015-09-01T22:29:12Z
Bron: https://wiki.osdev.org/HFS%2B
Revisie-URL: https://wiki.osdev.org/index.php?title=HFS%2B&oldid=18331

````text
{{Stub}}
{{Filesystems}}
HFS+ is the successor of [[HFS]], and was introduced by Apple in Mac OS 8.1. The main reason is support for larger disks than HFS and allowing for more than the rather poor limit of 65 thousand files, but it also has support for [[journaling]] and links. The format is described in Apple's "legacy" documentation, as technote 1150.

sectors 0 & 1 are the boot sectors.
sector 2 has the volume header

== External Links ==
*[https://developer.apple.com/legacy/library/technotes/tn/tn1150.html#//apple_ref/doc/uid/DTS10002989 TN1150 - HFS Plus Volume Format]

[[Category:Filesystems]]
````

<a id="w002771"></a>

## W002771 — Non-Maskable Interrupt

Bron-ID: W002771
Namespace: Main (0)
Revisie: 9473 | Datum: 2009-12-20T15:57:57Z
Bron: https://wiki.osdev.org/Non-Maskable_Interrupt
Revisie-URL: https://wiki.osdev.org/index.php?title=Non-Maskable_Interrupt&oldid=9473
Doorverwijzing: Non Maskable Interrupt

````text
#REDIRECT [[Non Maskable Interrupt]]
````

<a id="w002772"></a>

## W002772 — IA-64

Bron-ID: W002772
Namespace: Main (0)
Revisie: 30370 | Datum: 2026-03-22T23:17:15Z
Bron: https://wiki.osdev.org/IA-64
Revisie-URL: https://wiki.osdev.org/index.php?title=IA-64&oldid=30370

````text
{{Distinguish|text=[[x86-64]], AMD's 64-bit extension of x86-32}}

'''IA-64''' (Intel Architecture 64 bit) is a [[RISC]], VLIW (very large instruction word) CPU architecture developed jointly by Intel and HP, with the intention of replacing the [[x86|x86-32]], [[PA-RISC]] and [[Alpha]] CPUs in the server and workstation market first and later in desktops. After many setbacks, the first Itanium CPU, ''Merced'' was released in 2001. Due to the delays, disappointing performance and other technical issues, it never obtained the expected popularity, getting itself displaced from the market by AMD's 64-bit extension to x86, [[x86-64]] (also known as AMD64).

The first document describing the new instruction set was published by Intel in May 1999. The name IA-64 was published beforehand, and some of the features were known beforehand (predication and wide-issue were discussed in December 1997 on comp.arch).

In 2019, Intel announced that the latest generation, ''Kittson'', of IA-64 processors would cease production by January 2020, and that shipments would stop in July 2021. With this, Itanium was finally declared obsolete.

==Features==
* '''EPIC''' (Explicitly Parallel Instruction Computing) computing paradigm, moving the responsibility of determining which instructions can be safely executed in parallel to the compiler rather than in the CPU itself (like in any other architecture).
* 128 bit wide instruction bundles, containing 2 or 3 instructions, along with a template (which describes the instruction types and dependency information)
* "register stack engine" which would automatically store some registers to memory on function calls (and load them back on return)
* very large register files, 128 general-purpose integer registers (<code>r0</code> to <code>r127</code>), 128 floating point registers (<code>f0</code> to <code>f127</code>), 64 predicate registers (<code>p0</code> to <code>p63</code>), 8 branch registers (<code>b0</code> to <code>b7</code>), 128 special-purpose "application registers" and the instruction pointer (<code>ip</code>).
* several features intended to expose out-of-order processing to the compiler
** explicit loop branches, with rotating registers (for software pipelining)
** non-faulting loads, "not-a-thing bit" for uninitialized variables
** loads which check against later stores
* 3 or 4 register, non-destructive format (dest = src1 op src2; dest = src1 * src2 + src3)
* almost all instructions can be predicated (conditionally executed based on the 1-bit predicate registers)
* simple addressing modes (register indirect, with post increment)

A feature initially announced by Intel was native execution of [[x86]] code (including operating systems) for compatibility, but it was removed in the design process due to complexity and low performance.

==Implementations==
* Merced (released as Itanium, not to be confused with the Itanium Instruction Set aka IA-64)
* McKinley (released as [[Itanium2 Overview | Itanium 2]])

== Compiler Support ==

* [[Open64]]
* [[GCC]]
* [[LLVM]] (Very poor support, dropped in 2.6)
* HP's aCC Compiler
* Intel C++ Compiler (Edison Design Group frontend)

== External links ==

* [http://en.wikipedia.org/wiki/Itanium Itanium at Wikipedia]
* [https://tech-pubs.net/wiki/index.php/Itanium Itanium at TechPubs]

[[Category:Platforms]]
[[Category:IA-64]]
[[Category:Instruction Set Architecture]]
````

<a id="w002773"></a>

## W002773 — STI

Bron-ID: W002773
Namespace: Main (0)
Revisie: 9487 | Datum: 2009-12-20T16:26:16Z
Bron: https://wiki.osdev.org/STI
Revisie-URL: https://wiki.osdev.org/index.php?title=STI&oldid=9487
Doorverwijzing: Interrupts

````text
#REDIRECT [[Interrupts]]
````

<a id="w002774"></a>

## W002774 — EBDA

Bron-ID: W002774
Namespace: Main (0)
Revisie: 9488 | Datum: 2009-12-20T16:29:59Z
Bron: https://wiki.osdev.org/EBDA
Revisie-URL: https://wiki.osdev.org/index.php?title=EBDA&oldid=9488
Doorverwijzing: Memory Map (x86)

````text
#REDIRECT [[Memory Map (x86)#Extended BIOS Data Area (EBDA)]]
````

<a id="w002775"></a>

## W002775 — Joliet

Bron-ID: W002775
Namespace: Main (0)
Revisie: 27897 | Datum: 2023-07-09T04:48:05Z
Bron: https://wiki.osdev.org/Joliet
Revisie-URL: https://wiki.osdev.org/index.php?title=Joliet&oldid=27897

````text
{{Stub}}
{{Filesystems}}
Joliet is an extension to the [[ISO 9660]] filesystem and was created
by Microsoft in 1995.
It allows longer filenames of up to 64 characters in 16-bit character set
UCS-2, and relaxes some of the restrictions associated with full ISO 9660
compliance.

== See also ==
=== Articles ===
* [[ISO 9660]], about the hosting fileystem.
* [[ISO 9660#Rock_Ridge_and_Joliet]], the section about extensions to ISO 9660.
=== External links ===
* [http://www.ecma-international.org/publications/files/ECMA-ST/Ecma-119.pdf ISO 9660 (ECMA-119) specification]
* [http://www.buildorbuy.org/pdf/joliet.pdf Joliet specification]

[[Category:Filesystems]]
````

<a id="w002776"></a>

## W002776 — Category:Operating Modes

Bron-ID: W002776
Namespace: Category (14)
Revisie: 9569 | Datum: 2009-12-31T05:54:03Z
Bron: https://wiki.osdev.org/Category:Operating_Modes
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AOperating_Modes&oldid=9569

````text
List of operating modes
[[Category:X86 CPU]]
````

<a id="w002777"></a>

## W002777 — CLI (Assembly)

Bron-ID: W002777
Namespace: Main (0)
Revisie: 9493 | Datum: 2009-12-20T16:45:02Z
Bron: https://wiki.osdev.org/CLI_(Assembly)
Revisie-URL: https://wiki.osdev.org/index.php?title=CLI_%28Assembly%29&oldid=9493
Doorverwijzing: Interrupts

````text
#REDIRECT [[Interrupts]]
````

<a id="w002778"></a>

## W002778 — VFAT

Bron-ID: W002778
Namespace: Main (0)
Revisie: 28335 | Datum: 2023-09-28T15:08:31Z
Bron: https://wiki.osdev.org/VFAT
Revisie-URL: https://wiki.osdev.org/index.php?title=VFAT&oldid=28335
Doorverwijzing: FAT

````text
#REDIRECT [[FAT#VFAT]]
````

<a id="w002779"></a>

## W002779 — FFS (BSD)

Bron-ID: W002779
Namespace: Main (0)
Revisie: 9522 | Datum: 2009-12-21T21:11:31Z
Bron: https://wiki.osdev.org/FFS_(BSD)
Revisie-URL: https://wiki.osdev.org/index.php?title=FFS_%28BSD%29&oldid=9522
Doorverwijzing: UFS

````text
#REDIRECT [[UFS]]
````

<a id="w002780"></a>

## W002780 — Physical Address Extension

Bron-ID: W002780
Namespace: Main (0)
Revisie: 28245 | Datum: 2023-07-12T16:36:48Z
Bron: https://wiki.osdev.org/Physical_Address_Extension
Revisie-URL: https://wiki.osdev.org/index.php?title=Physical_Address_Extension&oldid=28245

````text
This article discusses the Physical Address Extension (PAE) available on many x86 processors. It assumes good understanding of virtual memory (as discussed in [[Paging]]). It also discusses features specific to x86, although other architectures could easily have very similar mechanisms.

==History==
The PAE bit was introduced in the Intel Pentium Pro processor. It was specifically aimed at allowing 32 bit operating systems to make use of more than 4 GB of memory.

==How It Works==
When the PAE bit is off, each entry in the page table specifies a 32 bit base address into physical memory. When the bit is on, the entry is expanded to allow up to 64 bits (although most processors support less than this amount). This allows a process to be located above the 4 GB boundary, but a 32-bit process cannot access more than 4 GB at any time.

==Dangers and Pitfalls==
32 bit I/O devices (and driver code) will most likely not understand addresses above 4 GB.

This creates a "hole", as some of the address space is set aside for these drivers (see [[Memory Map (x86)]]). You may be able to remap DRAM from this region to a space above 4 GB, or the storage may be lost.

There is additional complexity with DMA (how can your 32 bit hard disk driver copy code or data for a process located above 4 GB?).

These difficulties often drive OS developers to work in a flat 64 bit address space.

==Long Mode==
Enabling PAE is required when switching into [[Long Mode]]. The structures remain the same. The main difference being that code based at physical address 0 can access all of memory using 64 bit pointers.

==See Also==
===Articles===
* [[Setting Up Paging With PAE]]
* [http://technet.microsoft.com/en-us/library/cc736309.aspx How PAE X86 Works]

===Wikipedia===
* PAE at [http://en.wikipedia.org/wiki/Physical_Address_Extension Wikipedia]

[[Category:x86]]
[[Category:Paging]]
[[Category:Virtual Memory]]
[[de:Physical_Address_Extension]]
````

<a id="w002782"></a>

## W002782 — User:Messiahandrw

Bron-ID: W002782
Namespace: User (2)
Revisie: 9551 | Datum: 2009-12-28T06:12:09Z
Bron: https://wiki.osdev.org/User:Messiahandrw
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMessiahandrw&oldid=9551
Doorverwijzing: User:MessiahAndrw

````text
#REDIRECT [[User:MessiahAndrw]]
````

<a id="w002783"></a>

## W002783 — ZFS

Bron-ID: W002783
Namespace: Main (0)
Revisie: 27220 | Datum: 2022-07-09T07:36:38Z
Bron: https://wiki.osdev.org/ZFS
Revisie-URL: https://wiki.osdev.org/index.php?title=ZFS&oldid=27220

````text
{{Filesystems}}
{{Stub}}

ZFS is a filesystem designed by Sun Microsystems known for its exceptionally high storage capacities, built-in volume management, data corruption prevention measures, transparent data deduplication, and transparent encryption. The maximum file size is 2^64 bytes (16 exabytes), and the maximum number of files is 2^48.

== Links ==
* [http://www.giis.co.in/Zfs_ondiskformat.pdf ZFS on-disk format official specification]
* [https://blogs.oracle.com/bonwick/ Jeff Bonwick blog]
* [http://en.wikipedia.org/wiki/ZFS ZFS article on wikipedia]

[[Category:Filesystems]]
````

<a id="w002784"></a>

## W002784 — MTRR

Bron-ID: W002784
Namespace: Main (0)
Revisie: 28430 | Datum: 2023-11-01T14:09:18Z
Bron: https://wiki.osdev.org/MTRR
Revisie-URL: https://wiki.osdev.org/index.php?title=MTRR&oldid=28430

````text
'''MTRR''', or '''Memory-Type Range Registers''' are a group of x86 [[Model Specific Registers]]
providing a way to control access and cacheability of physical memory regions. These were introduced
in the Intel Pentium Pro (P6) processor, intended to extend and enhance the memory type information
provided by page tables (i.e. the write-through and cache disable bits).

The set of registers is provided in two groups: 11 registers for 88 fixed ranges and a number of base-mask pairs
for custom range configuration. The exact number of the latter can be known by reading the capabilities register.

While this method is available in most new Intel and AMD processors, preference
should be given to [[Page Attribute Tables]] instead, as many of the systems usually
configure these during early boot from UEFI/BIOS and changing MTRRs which were set up by the firmware
may lead to unexpected problems.

==Memory types==

There are 5 memory types defined for use in MTRRs:

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
|}

==Accessing MTRRs==

Before attempting reads/writes to MTRRs, the operating system should first check the availability
of this feature by checking if [[CPUID]].01h:EDX[bit 12] is set.

Once the feature is known to be available, the following MSRs can be used for configuration:

====IA32_MTRRCAP (MSR 0xFE)====

This register describes memory range types and count supported by the implementation. The register is read-only.

{| class="wikitable" border="1"
|-
! Bit(s)
! Label
! Description
|-
| 63..11
| -
| Reserved
|-
| 10
| WC
| Write-combining type is available
|-
| 9
| -
| Reserved
|-
| 8
| FIX
| ''All'' fixed-size MTRRs are available
|-
| 7..0
| VCNT
| Count of variable-size MTRRs supported
|}

====IA32_MTRRdefType (MSR 0x2FF)====

Describes the default type used for physical addresses which are outside of any configured memory ranges,
as well as whether MTRRs and fixed ranges are enabled.

{| class="wikitable" border="1"
|-
! Bit(s)
! Label
! Description
|-
| 63..12
| -
| Reserved
|-
| 11
| E
| MTRR enable
|-
| 10
| FE
| Fixed range enable
|-
| 9..8
| -
| Reserved
|-
| 7..0
| Type
| Default memory type
|}

====IA32_MTRRphysBase''n'' and IA32_MTRRphysMask''n'' registers====

These allow defining custom memory ranges using base-mask pairs and have the following addresses:

* IA32_MTRRphysBase''n'' = 0x200 + (n * 2)
* IA32_MTRRphysMask''n'' = 0x201 + (n * 2)

Where ''n'' parameter ranges from 0 to IA32_MTRRdefType[VCNT].

Base register has the following bits:

{| class="wikitable" border="1"
|-
! Bit(s)
! Label
! Description
|-
| 63..52
| -
| Reserved
|-
| 51..12
| PhysBase
| Range base address
|-
| 11..8
| -
| Reserved
|-
| 7..0
| Type
| Range memory type
|}

"Mask" registers use bits 51:12 to specify the region mask
and bit 11 ("Valid" bit) tells if the region is enabled.
If your CPU supports address widths of less than 52 bits, the remaining bits are reserved. 
Writing to these reserved bits will trigger a General Protection Fault.

The processor uses the following algorithm to check if an address
is covered by an MTRR:

  MaskBase = PhysMask AND PhysBase
  MaskTarget = PhysMask AND Target_Address[51:12]
  IF MaskBase == MaskTarget
    target address is in range
  ELSE
    target address is not in range

The mask is calculated like this: <code>~(size - 1) & ((1 << ADDRESS_WIDTH) - 1)</code> where <code>ADDRESS WIDTH</code> is the maximum supported address width of your CPU. Both the mask and base must be
aligned to at least 4K boundary (thus must be shifted to the right by 12 to fit into the 51:12 range) and the size '''must''' be equal to
the boundary, on which the range is aligned. So, in order to have, for
example, a 6M-long range, one would need to set up two MTRRs, one
4M and the other 2M.

====IA32_MTRRfix''size''_''base''====

If supported by implementation, the specification defines a set of
11 registers allowing the configuration of addresses in lowest 1M
block.

Each fixed-size range register contains 8 eight-bit type fields
and the address block is split in different sizes:

{| class="wikitable" border="1"
|-
! 63-56
! 55-48
! 47-40
! 39-32
! 31-24
! 23-16
! 15-8
! 7-0
!rowspan=2| Register
|-
!colspan=8| Address range
|-
| 70000-7FFFF
| 60000-6FFFF
| 50000-5FFFF
| 40000-4FFFF
| 30000-3FFFF
| 20000-2FFFF
| 10000-1FFFF
| 00000-0FFFF
| MTRRfix64K_00000
|-
| 9C000-9FFFF
| 98000-9BFFF
| 94000-97FFF
| 90000-93FFF
| 8C000-8FFFF
| 88000-8BFFF
| 84000-87FFF
| 80000-83FFF
| MTRRfix16K_80000
|-
| BC000-BFFFF
| B8000-BBFFF
| B4000-B7FFF
| B0000-B3FFF
| AC000-AFFFF
| A8000-ABFFF
| A4000-A7FFF
| A0000-A3FFF
| MTRRfix16K_A0000
|-
| C7000-C7FFF
| C6000-C6FFF
| C5000-C5FFF
| C4000-C4FFF
| C3000-C3FFF
| C2000-C2FFF
| C1000-C1FFF
| C0000-C0FFF
| MTRRfix4K_C0000
|-
| CF000-CFFFF
| CE000-CEFFF
| CD000-CDFFF
| CC000-CCFFF
| CB000-CBFFF
| CA000-CAFFF
| C9000-C9FFF
| C8000-C8FFF
| MTRRfix4K_C8000
|-
| D7000-D7FFF
| D6000-D6FFF
| D5000-D5FFF
| D4000-D4FFF
| D3000-D3FFF
| D2000-D2FFF
| D1000-D1FFF
| D0000-D0FFF
| MTRRfix4K_D0000
|-
| DF000-DFFFF
| DE000-DEFFF
| DD000-DDFFF
| DC000-DCFFF
| DB000-DBFFF
| DA000-DAFFF
| D9000-D9FFF
| D8000-D8FFF
| MTRRfix4K_D8000
|-
| E7000-E7FFF
| E6000-E6FFF
| E5000-E5FFF
| E4000-E4FFF
| E3000-E3FFF
| E2000-E2FFF
| E1000-E1FFF
| E0000-E0FFF
| MTRRfix4K_E0000
|-
| EF000-EFFFF
| EE000-EEFFF
| ED000-EDFFF
| EC000-ECFFF
| EB000-EBFFF
| EA000-EAFFF
| E9000-E9FFF
| E8000-E8FFF
| MTRRfix4K_E8000
|-
| F7000-F7FFF
| F6000-F6FFF
| F5000-F5FFF
| F4000-F4FFF
| F3000-F3FFF
| F2000-F2FFF
| F1000-F1FFF
| F0000-F0FFF
| MTRRfix4K_F0000
|-
| FF000-FFFFF
| FE000-FEFFF
| FD000-FDFFF
| FC000-FCFFF
| FB000-FBFFF
| FA000-FAFFF
| F9000-F9FFF
| F8000-F8FFF
| MTRRfix4K_F8000
|}

Fixed-size range registers have are:

* MTRRfix64K_00000 — MSR 0x250
* MTRRfix16K_80000 — MSR 0x258
* MTRRfix16K_A0000 — MSR 0x259
* MTRRfix4K_C0000 — MSR 0x268
* MTRRfix4K_C8000 — MSR 0x269
* MTRRfix4K_D0000 — MSR 0x26A
* MTRRfix4K_D8000 — MSR 0x26B
* MTRRfix4K_E0000 — MSR 0x26C
* MTRRfix4K_E8000 — MSR 0x26D
* MTRRfix4K_F0000 — MSR 0x26E
* MTRRfix4K_F8000 — MSR 0x26F

==Combining with page-level control==

Both MTRR and [[Page Tables]] allow setting cache-disable and writethrough
attributes for memory regions — there may be cases when such ranges overlap. How MTRR and page entry bits work in combination (if the PAT register wasn't modified):

=== Without PAT ===

{| class="wikitable" border="1"
|-
! MTRR type !! Page CD bit !! Page WT bit !! Resulting type
|-
| UC || — || — || UC
|-
|rowspan=3|WC || 0 || — || WC
|-
| 1 || 0 || WC (impl. dependent)
|-
| 1 || 1 || UC
|-
|rowspan=2|WP || 0 || — || WP
|-
| 1 || — || UC
|-
|rowspan=2|WT || 0 || — || WT
|-
| 1 || — || UC
|-
|rowspan=3|WB || 0 || 0 || WB
|-
| 0 || 1 || WT
|-
| 1 || — || UC
|}

=== With PAT ===

{| class="wikitable" border="1"
|-
! MTRR type !! PAT Entry Value !! Resulting type
|-
|rowspan=6|UC || UC || UC
|-
| UC- || UC
|-
| WC || WC
|-
| WT || UC
|-
| WB || UC
|-
| WP || UC
|-
|rowspan=6|WC || UC || UC
|-
| UC- || WC
|-
| WC || WC
|-
| WT || UC
|-
| WB || WC
|-
| WP || UC
|-
|rowspan=6|WT || UC || UC
|-
| UC- || UC
|-
| WC || WC
|-
| WT || WT
|-
| WB || WT
|-
| WP || WP
|-
|rowspan=6|WB || UC || UC
|-
| UC- || UC
|-
| WC || WC
|-
| WT || WT
|-
| WB || WB
|-
| WP || WP
|-
|rowspan=6|WP || UC || UC
|-
| UC- || WC
|-
| WC || WC
|-
| WT || WT
|-
| WB || WP
|-
| WP || WP
|}

[[Category:X86 CPU‏‎]]
````

<a id="w002785"></a>

## W002785 — ISO

Bron-ID: W002785
Namespace: Main (0)
Revisie: 9575 | Datum: 2010-01-03T12:07:56Z
Bron: https://wiki.osdev.org/ISO
Revisie-URL: https://wiki.osdev.org/index.php?title=ISO&oldid=9575
Doorverwijzing: ISO 9660

````text
#REDIRECT [[ISO 9660]]
````

<a id="w002786"></a>

## W002786 — User talk:Markhobley

Bron-ID: W002786
Namespace: User talk (3)
Revisie: 9633 | Datum: 2010-01-16T09:07:51Z
Bron: https://wiki.osdev.org/User_talk:Markhobley
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AMarkhobley&oldid=9633

````text
Hi,

I've seen several links to your homepage appear last night, and it shocked me how much omissions, inaccuracies, and sometimes blatant propaganda are within the relatively small content. Since it's not part of the wiki, I can't edit it, although at some points it is used as if it were part of the wiki. In the current state, I do plan on removing some, if not all, of the links as they add little to the existing wiki contents.

Some things:
* (propaganda) Restricting yourself to 386 instructions is a choice. However, there are numerous design issues with that chip that make code (especially OS code) for it rather inefficient. Selecting a later architecture as a base (most commonly, the 486 or pentium pro) can improve your code while only wasting those chips that you "should not even be using anymore".

Why should we not be using these chips? They are already installed on our computers, and I haven't seen a free upgrade programme anywhere.

I actually use the maths coprocessor instructions (on the 487). The reason for locking the architecture is due to bugs in the Linux kernel.

* In your list of features, the most notable extensions are missing: MMX and SSE.

Yeah, I intend to support the Intel Pentium 120, which does not provide either of these extensions. It is best not to embed them into the kernel, but add them to a loadable module that can be added on demand.

I have now drafted a policy for Matrix Maths Extensions as follows:

 It is permissible to use Matrix Maths Extensions in a dedicated loadable module, however this policy requires that an emulation module is provided to enable deployment of software on computers that do not have the capability of using Matrix Maths Extensions.

Recommended modules names

    * mmx - for a computer that is capable of utilizing Matrix Maths Extensions
    * mmxemu - An emulator module for use on computers that are not capable of using Matrix Maths Extensions

Instruction Sets

The mmxemu emulator module must comply with the IA32 instruction set policy, and should be build with a target arch of i486 or lower.

The mmx module should be build with a target arch of i586MMX, and should not contain processor supplementary instructions that are not available on the Intel Pentium MMX processor. Note that the consistent deployment principle still applies, and it is not permissible to use cpuid instructions within the mmx module. 

* (inaccuracy) Intel did not "allow" others to use the instruction set. It's forced by legislation. Also AMD is known to have probed/reverse engineered Intel processors to learn more about it.

What legislation was that?
I read that AMD produced these under licence from Intel, but I will research this.

Ok, I have now reworded the glossary to read Intel and other vendors both decided to utilize the IA32 instruction set on the processors that they produced.

* (propaganda) using CPUID to enable processor-specific code is a deliberate action, and not bad design. Even intel considers it the way to use such features.

You can use the features, but they will break backward compatibility of the code when it is run on existing processors for the purposes of forcing people to upgrade. I still say it is a bad design. Only one branch of the code will ever run on the machine, so other branches are redundant.
:propaganda bordering on BS. Any newer intel, or AMD, or Cyrix processor has all the features of older processors for backwards compatibility. 

Indeed, but running these instructions on an older processor will not work - Try it if you do not believe me!!!!

Also, not all users are 1337 like us to compile their software specifically for their target. Most likely your grandma would benefit from having both versions in the same package.

Surely the installer program can decide what version to install, we do not need both sets of instructions in the same binaries.

Instruction sets should be decided at compile time, not at run time IMHO.

* (inaccuracy) The linux kernel has configuration options which say for what host it should be configured.

It does not work. I build and test on my AMD K7, using 486 architecture options. I then copy files to the Intel Pentium 120, the AMD K5and the Cyrix machines, and the systems crash with a Bug Int 6 error, due to invalid instructions being used on those architectures. I looked at the kernel code, and found those instructions embedded there, either though I chose an architecture which does not include them.
: it helps if you stop compiling with a i686 toolchain...

* (inaccuracy) GNU tools (and any tool based on automake and friends) allow configuration for different hosts, including different architectures. You can look at the GCC crosscompiler tutorial to see how this works.

Again the gnu assembler includes cpuid on the 486 instruction set, even though not all 486 processors have this, and there is no option to restrict the build to IA32 instructions. I have had to edit the instruction table to remove this instruction.
:Relevance?

* (omission) You can compile the ''same'' code with ''different'' optimisations, which would result in the same behaviour (provided the code itself not broken), but while taking advantage of the target platform.

Indeed. That is the way to go! Do not stick extended instructions into binary files for use on machines that do not use them.

I have patched the uname command to try to trick the build system into thinking it is on a 386.

There is however a bug in glibc which contains some instructions that I am not familiar with, and neither are my processors.

Mark.

Sincerely
- [[User:Combuster|Combuster]] 06:42, 6 January 2010 (UTC)


:I might add that most of your semi-valid reasons are related to GNU software, and thus are totally irrelevant for hobby OS development. - [[User:Combuster|Combuster]] 06:42, 7 January 2010 (UTC)

A developer may not want to make the same mistakes. I am currently expanding those articles, which will contain realistic examples of the problems above, with the faulty code disected. I am not against additional instructions on higher architecture, but I still believe in compile time decision making. and adding extended features via modules. I hope that the K486 fork will address these issues.


== Factual Errors and Propaganda ==
I, too, am concerned about the factual errors and propaganda exhibited in the linked site. I will be monitoring the links and they will be removed if necessary.
* Regarding Intel "Allowing" others to use the x86 instruction set, see [http://en.wikipedia.org/wiki/AMD_Am386 Wikipedia].
*
 CPUID is a perfectly valid way to detect processor features. I suppose the ''compiler'' is allowed to implement CPUID to determine how to optimise?

On your site, it is stated as fact and not opinion that the use of CPUID is bad practice.
* [http://markhobley.yi.org/glossary/supplementaryinstruction.html This page] is largely personal opinion. You are welcome to that on your own site but not on pages linked by a factual wiki. "Educating" aspiring os devvers not to use SYSCALL/SYSENTER is not good.

Again syscall and sysenter do not work on the following IA32 compatible processors:

Intel Pentium
AMD K5
AMD K6
Cyrix 686

: Yes, more modern instructions like SYSENTER/SYSCALL do not work on these processors, but you can always make your kernel accommodate that by using SYSENTER/SYSCALL when you available, while also providing an interrupt as backup (by having handling code in your system call library that checks if SYSCALL/SYSENTER can be used).

:: It would be better do determine this at COMPILE TIME, or using different modules on different machines. It is not good design to embed sysenter and syscall instructions into IA32 programs, because some machines cannot use them.
:: [[User:Markhobley]] 17:22, 8 January 2010 (UTC)
:: This is a matter of opinion, that is the crux of our argument, your views are being presented as fact, when they are actually just your personal opinion. In my opinion it is better to have a kernel that can work on any machine that has protected mode but can use the newer features if they are available.
:: (Also, can you please treat this page more like a thread on the forums, it's supposed to read like a discussion, so please tag your comments using <nowiki>~~~~</nowiki>)
:: [[User:Thepowersgang|Thepowersgang]] 02:42, 9 January 2010 (UTC)

I actually say, these features can be implemented as loadable modules on machines that support them, although this information has not been fully written up yet - Mark.

: What [com]buster is saying, is that you are not differentiating between your personal opinion and fact. The idea of this wiki is to inform aspiring OS developers on what they can do, and how it can be done. We do not want to colour that any more than it already is. I move that the links be taken off the wiki for the time being, until the content is changed.
: [[User:Thepowersgang|Thepowersgang]] 15:22, 8 January 2010 (UTC)

* I agree with Combuster that code that behaves differently with different optimisations is poorly written code.
- It was actually me that said that. Mark.

=== Scope ===

Also, the scope of the article is limited to IA32 related builds.
If the target build arch is [i686] or higher, then the articles on my website may not be applicable to those, because my own website is limited in scope to IA32, which is the platform that I currently support.

It is however expected that a build for [i386], [i486] and [i586] run on all IA32 compatible machines, and stick to the IA32 instruction set, and do not include [cpuid] branches.


::I could bash more of your arguments and take this offtopic further. However we can safely say that '''the majority considers references to your site inappropriate''', and I'll leave it at that. Please do not link to them, they will be removed. - [[User:Combuster|Combuster]] 11:15, 10 January 2010 (UTC)
````

<a id="w002787"></a>

## W002787 — USB

Bron-ID: W002787
Namespace: Main (0)
Revisie: 9589 | Datum: 2010-01-08T07:11:21Z
Bron: https://wiki.osdev.org/USB
Revisie-URL: https://wiki.osdev.org/index.php?title=USB&oldid=9589
Doorverwijzing: Universal Serial Bus

````text
#REDIRECT [[Universal Serial Bus]]
````

<a id="w002788"></a>

## W002788 — Talk:USB

Bron-ID: W002788
Namespace: Talk (1)
Revisie: 9591 | Datum: 2010-01-08T07:11:21Z
Bron: https://wiki.osdev.org/Talk:USB
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AUSB&oldid=9591
Doorverwijzing: Talk:Universal Serial Bus

````text
#REDIRECT [[Talk:Universal Serial Bus]]
````

<a id="w002789"></a>

## W002789 — Intel 8254x

Bron-ID: W002789
Namespace: Main (0)
Revisie: 30346 | Datum: 2026-03-17T21:07:33Z
Bron: https://wiki.osdev.org/Intel_8254x
Revisie-URL: https://wiki.osdev.org/index.php?title=Intel_8254x&oldid=30346

````text
The Intel 8254x series is comprised of: 82546GB/EB, 82545GM/EM, 82544GC/EI, 82541(PI/GI/EI), 82541ER, 82547GI/EI, and 82540EP/EM Gigabit Ethernet Controllers.  
[[Image:Intel-82540EM.jpg|right|frame|Intel 82540EM-based card]]

== Overview ==

Intel 8254x-based cards come in 32-/64-bit, 33/66 MHz PCI and PCI-X flavors.   

The Intel 82547GI(EI) connects to the motherboard via a Communications Streaming Architecture (CSA) port instead of a PCI/PCI-X bus.    

The 82541xx and 82540EP/EM controllers do not support the PCI-X bus.  
 
They are all high-performance, Gigabit-capable controllers and range from 1 to 4 ethernet/fiber ports per controller.

The Intel 8254x series heavily utilizes task offloading.  Each controller has an "offloading engine" for tasks such as TCP/UDP/IP checksum calculations, packet filtering, and packet segmentation.  

* Jumbo packets are supported.
*Wake on LAN (WoL) is supported.
* A four wire serial EEPROM interface as well as a generic EEPROM "read" interface is implemented within the configuration registers.
*D0 and D3 power states are supported through ACPI.

==Programming==
===Detection===
Section 5.2 in the 8254x Software Developer's Manual lists the Vendor and Device ID's of the various device in the 8254x series.  These are used to detect devices on the PCI bus by looking in the PCI Configuration Space registers.  

The device will also fill in the PCI Base Address Registers (BAR).  BAR0 will either be a 64-bit or 32-bit MMIO address (checked by testing bits 2:1 to see if it's 00b (32-bit) or 10b (64-bit)) that points to the device's base register space.  BAR0 should always be used to interface with the device via MMIO as the BAR number never changes in different devices in the series.  

There is also a BAR that will contain an I/O base address, this can be detected by looking at each BAR and testing bit 1. Documentation states this will be in either BAR2 or BAR4, but emulators may move it.  

When using MMIO, reading/writing to/from registers is very straight-forward.  

<syntaxhighlight lang="c" line="1">
uint64_t ioaddr = BAR_GOES_HERE;

void write_register(uint16_t register, uint32_t value){
    *(uint32_t *)(ioaddr + register) = value;
}

uint32_t read_register(uint16_t register){
    return *(uint32_t *)(ioaddr + register);
}
</syntaxhighlight>

When using IO, reading/writing to/from registers is a little more complicated as the IO address space for the 8254x is only 8 bytes wide.  
The register at offset 0x00 is the "IOADDR" window.  The register at offset 0x04 is the "IODATA" window.  
IOADDR holds the IO address that the IODATA window operates on.  So, basic operation is to set the IOADDR window and then the desired action using the IODATA window. 

<syntaxhighlight lang="c" line="1">
uint16_t ioaddr = IO_BAR_GOES_HERE;

void write_register(uint16_t register, uint32_t value){
    outl(ioaddr + 0x00, register);  // set the IOADDR window
    outl(ioaddr + 0x04, value);  // write the value to the IOADDR window which will end up in the register in IOADDR
}

uint32_t read_register(uint16_t register){
    outl(ioaddr + 0x00, register); // set the IOADDR window
    return inl(ioaddr + 0x04); // read the value
}
</syntaxhighlight>

==Device Registers==
The 8254x cards have a handful of registers. There is a complete list of the registers and their offsets at the Table 13-2 (Page 219) of the [https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf#G15.1220466 Intel 8254x Family of Gigabit Ethernet Controllers Software Developer's Manual].

Here are the most important ones:
{| class="wikitable"
|+
!Category
!Offset
!Abbreviation
!Name
!R/W
!Manual Page
|-
|General
|00000h
|CTRL
| Device Control 
|R/W
|[https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf#page=238 224]
|-
|General
|00008h
| STATUS
|Device Status
|R
| [https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf#page=243 229] 
|-
|General

|00010h
|EECD
|EEPROM/Flash Control/Data
|R/W
|[https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf#page=246 232]
|-
|General
|00014h
|EERD
| EEPROM Read (not applicable
to the 82544GC/EI) 
|R/W 
|[https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf#page=250 236]
|-
|Interrupt
|000C0h
|ICR
|Interrupt Cause Read 
|R/W
| [https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf#page=306 292]
|-
|Interrupt
|000D0h
|IMS
|Interrupt Mask Set / Read
|R/W
|[https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf#page=311 297]

|-
|Receive
|00100h
|RCTL
|Receive Control
|R/W
|[https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf#page=314 300] 
|-
|Receive
| 02800h
|RDBAL
|Receive Descriptor Base Low
|R/W
|[https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf#page=320 306]
|-
|Receive
|02804h
| RDBAH
|Receive Descriptor Base High
|R/W
|[https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf#page=320 306]
|-
| Receive
|02808h
| RDLEN 
|Receive Descriptor Length
|R/W
|[https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf#page=321 307]
|-
|Receive
|02810h 
|RDH
|Receive Descriptor Head
|R/W 
|[https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf#page=321 307]
|-
|Receive
|02818h
|RDT
|Receive Descriptor Tail

|R/W
|[https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf#page=322 308]
|-
| Transmit
| 00400h
|TCTL
|Transmit Control 
|R/W
|[https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf#page=324 310]
|-
|Transmit
|03800h
|TDBAL 
|Transmit Descriptor Base Low
|R/W 
|[https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf#page=329 315]

|-
|Transmit 
|03804h 
|TDBAH
|Transmit Descriptor Base High 
|R/W
|[https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf#page=330 316]
|-
| Transmit
|03808h
|TDLEN
|Transmit Descriptor Length
|R/W
|[https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf#page=330 316]
|-
|Transmit
|03810h 
|TDH
|Transmit Descriptor Head
|R/W
|[https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf#page=331 317]
|-
| Transmit
|03818h
|TDT
|Transmit Descriptor Tail
| R/W
|[https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf#page=332 318]
|-
|Receive
|05400h-
05488h 
|RAL(8*n) 
|Receive Address Low (n)
|R/W
|[https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf#page=343 329]
|-
|Receive
|05404h-
0547Ch 
|RAH(8*n)
|Receive Address High (n)
|R/W
|[https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf#page=343 329]
|}

{| class="wikitable"
|+The Device Control Register (CTRL)
!Field
!Bit(s)
!Name
! Field 
!Bit(s)
!Name
|-
|FD
|0
|Full - Duplex
|SDP1_DATA
|19
|SDP1 Data Value
|-
|RSV
|2:1 
|Reserved
|ADVD3WUC
|20
|D3Cold Wakeup Capability
Advertisement Enable
|-
|LRST
|3
|Link Reset
|EN_PHY_PWR_MGMT
|21
|PHY Power Management Enable
|-
|RSV
|4
|Reserved
|SDP0_IODIR
| 22
|SDP0 Pin Directionality
|-
|ASDE
|5
|Auto-Speed Detection Enable
|SDP1_IODIR
|23
|SDP1 Pin Directionality
|-
|SLU
|6
|Set Link Up 
|RSV
|25:24
|Reserved
|-
|ILOS
|7
|Invert Loss-of-Signal
|RST
|26
| Device Reset
|-
|SPEED
|9:8
|Speed selection
|RFCE
|27
|Receive Flow Control Enable
|-
|RSV
|10
|Reserved
|TFCE
|28
| Transmit Flow Control Enable
|-
| FRCSPD
|11
|Force Speed 
|RSV
|29
|Reserved 
|-
|FRCDPLX 
| 12
|Force Duplex
|VME
|30
|VLAN Mode Enable
|-
|RSV
|17:13
|Reserved
|PHY_RST 
| 31
|PHY Reset
|-
|SDP0_DATA 
|18
|SDP0 Data Value
|
|
|
|}
{| class="wikitable"
|+Status Register Bit Description
!Field
!Bit(s)
!Name
|-
|FD
|0
|Link Full Duplex configuration Indication. 
|-
|LU
|1
|Link Up indication
|-
|Function ID
|3:2
|Provides software a mechanism to determine the Ethernet
controller function number (LAN identifier) for this MAC. Read

as: [0b,0b] LAN A, [0b,1b] LAN B.

'''Note:''' These settings are only applicable to the '''82546GB/EB'''.
|-
|TXOFF
|4
|Transmission Paused
|-
|TBIMODE
|5
|TBI Mode/internal SerDes Indication.
'''Note:''' For the '''82544GC/EI''', reflects the status of the TBI_MODE input pin.
|-
|SPEED
|7:6
|Link Speed Setting.
Speed indication is mapped as follows:

00b = 10 Mb/s

01b = 100 Mb/s

10b = 1000 Mb/s

11b = 1000 Mb/s

These bits are not valid in TBI mode/internal SerDes.
|-
|ASDV
|9:8
|Auto Speed Detection Value
|-
|RSV
|10
|Reserved
|-
|PCI66
|11
|PCI Bus speed indication. (When set, indicates that the PCI Bus is running
at 66 MHz).
|-
|BUS64<sup>1</sup>
|12
|PCI Bus Width indication. (When set, indicates that the Ethernet controller is on
a 64-bit bus)
|-
|PCIX_MODE<sup>1</sup>
|13
|PCI-X Mode indication. (When set, indicates that the Ethernet Controller is operating
in PCI-X mode)
|-
|PCIXSPD<sup>1</sup>
|15:14
|PCI-X Bus Speed Indication.
00b = 50-66 MHz

01b = 66-100 MHz

10b = 100-133 MHz

11b = Reserved
|-
|RSV
|31:16
|Reserved
|}
<sup>1</sup>. Not applicable to the '''82540EP/EM''', '''82541xx''', or '''82547GI/EI'''.
{| class="wikitable"
|+Transmit Control Register (TCTL)
!Field
!Bit(s)
!Name
|-
|RSV
|0
|Reserved
|-
|EN
|1
|Transmit Enable
|-
|RSV
|2
|Reserved
|-
|PSP
|3
|Pad Short Packets
|-
|CT
|11:4
|Collision Threshold
|-
|COLD
|21:12
|Collision Distance
|-
|SWXOFF
|22
|Software XOFF Transmission
|-
|RSV
|23
|Reserved
|-
|RTLC
|24
|Re-transmit on Late Collision
|-
|NRTU
|25
|No Re-transmit on underrun
(82544GC/EL Only)
|-
|RSV
|31:26
|Reserved
|}
{| class="wikitable"
|+Receive Control Register (RCTL)
!Field
!Bit(s)
!Name
!Field
!Bit(s)
!Name
|-
|RSV
|0
|Reserved
|BSIZE
|17:16
|Receive Buffer Size
|-
|EN
|1
|Receiver Enable
|VFE
|18
|VLAN Filter Enable
|-
|SBP
|2
|Store Bad Packets
|CFIEN
|19
|Canonical Form Indicator Enable
|-
|UPE
|3
|Unicast Promiscuous Enabled
|CFI
|20
|Canonical Form Indicator bit value
|-
|MPE
|4
|Multicast Promiscuous Enabled
|RSV
|21
|Reserved
|-
|LPE
|5
|Long Packet Reception Enable
|DPF
|22
|Discard Pause Frames
|-
|LBM
|7:6
|Loopback Mode
|PMCF
|23
|Pass MAC Control Frames
|-
|RDMTS
|9:8
|Receive Descriptor Minimum
Threshold Size
|RSV
|24
|Reserved
|-
|RSV
|11:10
|Reserved
|BSEX
|25
|Buffer Size Extenstion
|-
|MO
|13:12
|Multicast Offset
|SECRC
|26
|Strip Ethernet CRC from incoming packet
|-
|RSV
|14
|Reserved
|RSV
|21:27
|Reserved
|-
|BAM
|15
|Broadcast Accept
|
|
|
|}
When BSEX is set, the value in BSIZE is multiplied by 16.
{| class="wikitable"
|+Receive Buffer Size Configuration
!Size (Bytes)
!BSIZE
!BSEX
|-
|16384
|01b
|1
|-
|8192
|10b
|1
|-
|4096
|11b
|1
|-
|2048
|00b
|0
|-
|1024
|01b
|0
|-
|512
|10b
|0
|-
|256
|11b
|0
|}
{| class="wikitable"
|+Interrupt mask Set / Read (IMS)
!Field
!Bit(s)
!Description
|-
|TDW
|0
|Sets mask for Transmit Descriptor Written Back
|-
|TXQE
|1
|Sets mask for Transmit Queue Empty.
|-
|LSC
|2
|Sets mask for Link Status Change.
|-
|RXSEQ
|3
|Sets mask for Receive Sequence Error.
This is a reserved bit for the '''82541xx''' and '''82547GI/EI'''. Set to 0b.
|-
|RXDMT0
|4
|Sets mask for Receive Descriptor Minimum Threshold hit.
|-
|RSV
|5
|Reserved
|-
|RXO
|6
|Sets mask for on Receiver FIFO Overrun
|-
|RXT0
|7
|Sets mask for Receiver Timer Interrupt
|-
|RSV
|8
|Reserved
|-
|MDAC
|9
|Sets mask for MDI/O Access Complete Interrupt
|-
|RXCFG
|10
|Sets mask for Receiving /C/ ordered sets.
This is a reserved bit for the '''82541xx''' and '''82547GI/EI'''. Set to 0b
|-
|RSV
|11
|Reserved
|-
|PHYINT
|12
|Sets mask for PHY Interrupt (not applicable to the '''82544GC/EI''').
This is a reserved bit for the '''82541xx''' and '''82547GI/EI'''. Set to 0b
|-
|GPI
|14:11
|Sets mask for General Purpose Interrupts '''(82544GC/EI only).'''
|-
|GPI
|14:13
|Sets mask for General Purpose Interrupts
|-
|TXD_LOW
|15
|Sets the mask for Transmit Descriptor Low Threshold hit (not
applicable to the '''82544GC/EI''').
|-
|SRPD
|16
|Sets mask for Small Receive Packet Detection (not applicable to
the '''82544GC/EI''').
|-
|RSV
|31:17
|Reserved
|}
To enable an interrupt, simply write '1' to the corresponding bit.

==Descriptor Format==
Both receive and transmit descriptors are 16 bytes in size. There are 3 types of transmit descriptors, the original referred to as the "Legacy transmit descriptor". The second one is referred to as the " TCP/IP Data Descriptor" and is a replacement for the legacy descriptor  offering access to new offloading capabilities.The other descriptor type is fundamentally different as it does not point to packet data. It merely contains control information which is loaded into registers of the controller and affect the processing of future packets. For simplicity we will only use the Legacy transmit descriptor. If you want to learn more about the other types of descriptors, [https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf#page=49 you can have a look at the specification.]
{| class="wikitable" style="text-align: center;"
|+Legacy Transmit Descriptor Format
!63                    63
!47        40
!39   36
!35   32
!31        24
!23          16
!15                0
|-
| colspan="7" |Buffer Address
|-
|Special
|CSS
|RSV
|STA
|CMD
|CSO
|Length
|}
{| class="wikitable"
|+Legacy Transmit Descriptor Field Description
!Name
!Description
|-
|Buffer Address
|The address of the buffer. Descriptors with a null address transfer no data.
|-
|Length
|Length is per segment. The maximum length allowed is 16288 bytes.
|-
|CSO
|Checksum Offset. Indicates where, relative to the start of the packet to insert
a TCP checksum if it is enabled in the CMD field.
|-
|CMD
|Command Field
|-
|STA
|Status Field
|-
|RSV
|Reserved
|-
|CSS
|Checksum Start Field. Its an offset relative to the start of the buffer and it
indicates where to start computing the Checksum.
|-
|Special
|Special Field
|}
{| class="wikitable"
|+Transmit Descriptor Command Field Format
!7
!6
!5
!4
!3
!2
!1
!0
|-
|IDE
|VLE
|DEXT
|RPS
|RS
|IC
|IFCS
|EOP
|}
{| class="wikitable"
|+Transmit Descriptor Command Field Description
!Name
!Description
|-
|IDE (bit 7)
|Interrupt Delay Enable
|-
|VLE (bit 6)
|VLAN Packet Enable
|-
|DEXT (bit 5)
|Extension. (Set to 0b to indicate legacy mode)
|-
|RPS/RSV (bit 4)
|Report Packet Sent. '''82544GC/EL only. Otherwise reserved!'''
|-
|RS (bit 3)
|Report Status. (When set, the controller will fire an interrupt when
the packet gets transmitted and bit STA.DD (Descriptor Done) will be set).
|-
|IC (bit 2)
|Insert Checksum. (When set, the controller will insert a checksum based 
on the values of the CSO and CSS fields.)
|-
|IFCS (bit 1)
|Controls the Insertion of the FCS/CRC field in normal Ethernet packets.
IFCS is only valid when EOP is set.
|-
|EOP (bit 0)
|End Of Packet. It indicates the last descriptor making up the packet.
One or many descriptors can be used to form a packet.
|}

======Transmit Descriptor Status Format======
{| class="wikitable"
|+
!3
!2
!1
!0
|-
|TU
|LC
|EC
|DD
|}
{| class="wikitable"
|+Transmit Descriptor Status Field Description
!Name
!Description
|-
|TU/RSV (bit 3)
|Transmit Underrun. Indicated a transmit underrun error has occurred.
'''82544GC/EL only. Otherwise reserved!'''
|-
|LC (bit 2)
|Late Collision. Indicates that a Late Collision occurred while working in
half-duplex mode. It has no meaning in full-duplex.
|-
|EC (bit 1)
|Excess Collisions. It indicates that the packet has experienced more than
the maximum excessive collisions as defined by TCTL.CT control field.
|-
|DD (bit 0)
|Descriptor Done. Indicates that the descriptor is finished.
|}
{| class="wikitable" style="text-align: center;
|+Receive Descriptor Format
!63        48
!47        40
!39        32
!31                16
!15                0
|-
| colspan="5" |Buffer Address
|-
|Special'''*'''
|Errors
|Status
|Packet
Checksum'''*'''
|Length
|}
'''*82544GC/EL only. Otherwise reserved!'''
{| class="wikitable"
|+Receive Descriptor Status Field
!7
!6
!5
!4
!3
!2
!1
!0
|-
|PIF
|IPCF
|TCPCS
|RSV
|VP
|IXSM
|EOP
|DD
|}
{| class="wikitable"
|+Receive Descriptor Status Bits
!Name
!Description
|-
|PIF (bit 7)
|Passed in-exact filter. If set the software must examine this packet to determine
whether to accept it or not. if PIF is clear, the packet is known to be for this station.
|-
|IPCS (bit 6)
|IP Checksum Calculated on Packet. (0 = do not perform IP checksum, 1 = perform IP checksum)
|-
|TCPCS (bit 5)
|TCP Checksum Calculated on Packet. (0 = do not perform TCP/UDP checksum, 1 = perform TCP/UDP checksum)
|-
|RSV (bit 4)
|Reserved
|-
|VP (bit 3)
|Packet is 802.1Q (matched VET).
|-
|IXSM (bit 2)
|Ignore Checksum Indication. (when set, the checksum indication results should be ignored).
|-
|EOP (bit 1)
|End Of Packet. (Indicates that this is the last descriptor for an incoming packet)
|-
|DD (bit 0)
|Descriptor Done. (Indicates whether the controller is done with the descriptor)
|}
{| class="wikitable"
|+Receive Descriptor Errors Field
!7
!6
!5
!4
!3
!2
!1
!0
|-
|RXE
|IPE
|TCPE
|CXE<sup>a</sup>
|RSV
|SEQ<sup>b</sup>
|SE<sup>b</sup>
|CE
|}
<sup>a</sup>. '''82544GC/EI''' only, otherwise reserved!

<sup>b</sup>.'''82541xx, 82547GI/EI,''' and '''82540EP/EM''' only, otherwise reserved.
{| class="wikitable"
|+Receive Descriptor Error bits
!Name
!Description
|-
|RXE (bit 7)
|RX Data Error
|-
|IPE (bit 6)
|IP Checksum Error
|-
|TCPE (bit 5)
|TCP/UDP Checksum Error
|-
|CXE (bit 4)
|Carrier Extension Error
|-
|RSV (bit 3)
|Reserved
|-
|SEQ (bit 2)
|Sequence Error
|-
|SE (bit 1)
|Symbol Error
|-
|CE (bit 0)
|CRC Error or Alignment Error
|}
The Receive Descriptor Special field is only populated for 802.1q packets. For all other packets it's contents are set to 0.
{| class="wikitable"
|+Receive Descriptor Special Field
!15        13
!12
!11                    0
|-
|PRI
|CFI
|VLAN
|}
{| class="wikitable"
|+Receive Descriptor Special Field
!Name
!Description
|-
|VLAN
|VLAN Identifier
|-
|CFI
|Canonical Form Indicator
|-
|PRI
|User Priority
|}

==EEPROM Reading==
There are a few variants of the card with many differences, most notably the method to access the EEPROM and the Flash memory of the card. Here we will only describe methods applicable to cards that use the EEPROM method.

After that the EEPROM must be enabled in order to be able to read the MAC address of the NIC, this is done by setting the EECD.SK (0x01), EECD.CS (0x02) and EECD.DI (0x04) bits of the EECD (0x00010) register. This will allow software to perform reads to the EEPROM.

Before reading the EEPROM has a "lock-unlock" mechanism to prevent software-hardware collisions when reading from the EEPROM.

To lock the EEPROM the EECD.REQ (0x40) bit must be set in the EECD register. Then wait until the EECD.GNT (0x80) bit becomes set.
Unlocking only requires to clear EECD.REQ.

To finally read the EEPROM first the kernel should AND the address to 12 (Applicable only to 82541x or 82547GI/EI cards) or 8 bits; then bit shift the desired address to 2 (Applicable only to 82541x or 82547GI/EI cards) or by 4. The kernel must OR it with the EECD.START (0x01) bit. Then finally write it to the EERD (0x00014) register.

The kernel should wait until the EEPROM read operation is finished by checking until EECD.DONE becomes clear. Then the kernel must read the EERD register, shift it to the right by 16 bits and truncate it to 16-bits.

After that the EERD.START bit must be cleared.

<syntaxhighlight lang="c" line="1">
static uint16_t eeprom_read(uint8_t addr) {
    uint32_t tmp;
    uint16_t data;

    if((le32_to_cpu(mmio_read_dword(dev_info.mmio.addr, I8254X_EECD)) & I8254X_EECD_EE_PRES) == 0) {
        kpanic("EEPROM present bit is not set for i8254x\n");
    }

    /* Tell the EEPROM to start reading */
    if(dev_info.version == I82547GI_EI
    || dev_info.version == I82541EI_A0
    || dev_info.version == I82541EI_B0
    || dev_info.version == I82541ER_C0
    || dev_info.version == I82541GI_B1
    || dev_info.version == I82541PI_C0) {
        /* Specification says that only 82541x devices and the
         * 82547GI/EI do 2-bit shift */
        tmp = ((uint32_t)addr & 0xfff) << 2;
    } else {
        tmp = ((uint32_t)addr & 0xff) << 8;
    }
    tmp |= I8254X_EERD_START;
    mmio_write_dword(dev_info.mmio.addr, I8254X_EERD, cpu_to_le32(tmp));

    /* Wait until the read is finished - then the DONE bit is cleared */
    timeout((le32_to_cpu(mmio_read_dword(dev_info.mmio.addr, I8254X_EERD)) & I8254X_EERD_DONE) == 0, 100);

    /* Obtain the data */
    data = (uint16_t)(le32_to_cpu(mmio_read_dword(dev_info.mmio.addr, I8254X_EERD)) >> 16);

    /* Tell EEPROM to stop reading */
    tmp = le32_to_cpu(mmio_read_dword(dev_info.mmio.addr, I8254X_EERD));
    tmp &= ~(uint32_t)I8254X_EERD_START;
    mmio_write_dword(dev_info.mmio.addr, I8254X_EERD, cpu_to_le32(tmp));
    return data;
}
</syntaxhighlight>When all data is finally read the kernel should unlock the EEPROM to let hardware access it.

==Initialization==
The 8254x will be on an undefined state and as such it needs to be reset. The first thing that should be done is enabling bus mastering, memory and IO accesses from the [[PCI]] command register.

Then the NIC should be reset by setting CTRL.RST (bit 26, self clearing) bit in the Device Control register of the card.

After the card has been reset, you should enable the CTRL.ASDE, the CTRL.SLU bits (To enable Auto Speed Detection (ASDE), you must also set the SLU (Set link up) bit) and write the MAC address you want the device to use in the RAL0 and RAH0 registers. To get the device MAC address, all you have to do is read the first 3 bytes of the EEPROM.



The entire procedure looks something like this:<syntaxhighlight lang="c" line="1">
uint8_t MAC_ADDRESS[6];
void reset_nic(){
    uint32_t device_control = read_register(I8254_REG_CTRL);
    
    device_control |= I8254_CTRL_RESET; // Set the reset bit
    write_register(I8254_REG_CTRL, device_control);
    
    while(read_register(I8254_REG_CTRL) & I8254_CTRL_RESET) __asm__ ("hlt"); // wait for it to reset
    
    device_control = read_register(I8254_REG_CTRL);
    device_control |= I8254_CTRL_ASDE | I8254_CTRL_SLU; // Enable Auto Speed Detection.
    
    write_register(I8254_REG_CTRL, device_control);
    
    // Read the MAC address from the EEPROM
    uint16_t b0 = eeprom_read(0);
    uint16_t b1 = eeprom_read(1);
    uint16_t b2 = eeprom_read(2);
    
    MAC_ADDRESS[0] = b0 & 0xFF;
    MAC_ADDRESS[1] = b0 >> 8;
    MAC_ADDRESS[2] = b1 & 0xFF;
    MAC_ADDRESS[3] = b1 >> 8;
    MAC_ADDRESS[4] = b2 & 0xFF;
    MAC_ADDRESS[5] = b2 >> 8;
    
    // Write the MAC address to RAL/RAH 0.
    uint32_t writeL = ((uint32_t)b1 << 16) | b0;
    uint32_t writeH = b2;
    
    write_register(E1000_REG_RAL0, writeL);
    write_register(E1000_REG_RAH0, writeH);
}
</syntaxhighlight>

==Ring setup==

====Theory of operation:====
The next step is to setup the rings. Without setting up the rings, you will not be able to send/receive packets. Luckily the ring system is pretty simple, It consists of the T/RDH and T/RDT (Transmit/Receive Descriptor Head/Tail) and of-course the ring buffers.

====Transmit Ring====
In the image bellow, you can see the structure of the transmit ring. The shaded boxes represent descriptors that have been transmitted but not yet reclamed. (If you dynamically allocate the descriptor buffers, reclaiming would simply involve freeing those buffers).

[[File:Transmit_Ring_Structure.png|border]]

Anything between the Head and the Tail is owned by the controller and consists the transmit queue (the descriptors that have been queued for transmission). At reset, both TDT and TDH are set to 0. (If TDT = TDH that means that the queue is empty, there is nothing to transmit).

====Receive Ring====
The image bellow depicts the structure of the receive ring. The shaded boxes represent descriptors that have stored incoming packets but have not yet been recognized by the driver. You can detect which descriptors have incoming data written in them by checking whether the status field is non-zero.

[[File:Receive_Ring_Structure.png|border]]

Any descriptors between RDH and RDT are owned by the hardware and should not be modified!

After the reset, the head should point to the first descriptor and the tail to the last descriptor of the ring (Since all descriptors are available for use).

The RDH points to the descriptor the controller will write the next received packet. It increments automatically.

The RDT points to one descriptor after the last available descriptor. This register should still point to a valid descriptor (should be within Base and Base + Size).


The TDLEN/RDLEN registers contain the size in bytes of the ring.

===Setup:===

======Transmit Ring======

*Firstly allocate a region for the descriptor ring
*Next, you can allocate a static buffer for the descriptors if you want, or use a dynamically allocated buffer to allocate it when you transmit the packet (In this example code, we use the first option).
*Set '''TDH''' and '''TDT''' to 0, '''TDBAL''' to the lower 32 bits of the ring's physical address, '''TDBAH''' to the higher 32 bits and TDBAL to the total length of the ring buffer (number of descriptors * 16)
*Set your preferred bits in the '''TCTL''' registger.
<syntaxhighlight lang="c" line="1">
// Assumes 1:1 memory mapping for simplicity
#define NUM_OF_TX_DESCRIPTORS 8
#define SIZE_OF_TX_DESCRIPTOR_BUFFER 4096
struct transmit_descriptor_t;

void setup_transmit_ring(){
    size_t transmit_ring_size = NUM_OF_TX_DESCRIPTORS * 16;
    transmit_descriptor_t* transmit_ring = your_favorite_physical_allocator(transmit_ring_size);
    
    for (int i = 0; i < NUM_OF_TX_DESCRIPTORS; i++){
        transmit_descriptor_t* descriptor = transmit_ring + i;
        descriptor->buffer_address = your_favorite_physical_allocator(SIZE_OF_TX_DESCRIPTOR_BUFFER);
    }
    
    write_register(REG_TDBAL, ((uint64_t)transmit_ring) & 0xFFFFFFFF);
    write_register(REG_TDBAH, ((uint64_t)transmit_ring) >> 32);
    write_register(REG_TDLEN, transmit_ring_size);
    write_register(REG_TDH, 0);
    write_register(REG_TDT, 0);
    
    // Set the Enable (EN) and Pad Short Packets (PSP) bits
    uint32_t tctl = E1000_TCTL_EN | E1000_TCTL_PSP;
    write_register(REG_TCTL, tctl);
}
</syntaxhighlight>

=====Receive Ring=====

*Firstly allocate a region for the descriptor ring
*After that, loop through each descriptor and allocate a buffer of the selected size (set in the '''Receive Control Register''') and set it (its physical address) in the descriptor address field.
*Set '''RDH''' to 0 (the first descriptor), '''RDT''' to the last descriptor (number of descriptors - 1), '''RDBAL''' to the lower 32 bits of the ring's physical address, '''RDBAH''' to the higher 32 bits and '''RDLEN''' to the total length of the ring buffer (number of descriptors * 16).
*Set your preferred bits in the '''RCTL''' register (You must set the '''EN''' bit to enable the dma engine. '''LPE''' and '''BAM''' are recommended).
<syntaxhighlight lang="c" line="1">
// Assumes 1:1 page mapping for simplicity
#define NUM_OF_RX_DESCRIPTORS 32
#define SIZE_OF_RX_DESCRIPTOR_BUFFER 4096
struct receive_descriptor_t;

void setup_receive_ring(){
    size_t receive_ring_size = NUM_OF_RX_DESCRIPTORS * 16; // you can substitute 16 with sizeof(receive_descriptor_t)
    receive_descriptor_t* receive_ring = your_favorite_physical_allocator(receive_ring_size);
    
    for (int i = 0; i < NUM_OF_RX_DESCRIPTORS; i++){
        receive_descriptor_t* descriptor = receive_ring + i;
        descriptor->buffer_address = your_favorite_physical_allocator(SIZE_OF_RX_DESCRIPTOR_BUFFER);
    }
    
    write_register(REG_RDBAL, ((uint64_t)receive_ring) & 0xFFFFFFFF); // Base Address Low
    write_register(REG_RDBAH, ((uint64_t)rx_phys) >> 32); // Base Address High
    write_register(REG_RDLEN, receive_ring_size); // Ring Size
    write_register(REG_RDH, 0); // Set it to the first descriptor
    write_register(REG_RDT, NUM_OF_RX_DESCRIPTORS - 1); // Set it to the last descriptor
    
    // Set the Enable, Long Packet Reception, Broadcast Accept Mode and Size Extenstion bits
    // Also set the buffer size. This configuration (BSIZE = 0b11 and BSEX = 1) means 4096 (4kB) buffers
    uint32_t rctl = RCTL_EN | RCTL_LPE | RCTL_BAM | RCTL_BSEX | (0b11 << RCTL_BSIZE);
    write_register(REG_RCTL, rctl); 
}
</syntaxhighlight>

==Interrupt Handling==
Well, If you want to receive packets, you need a way of knowing when to read them. Thats where interrupts come into play.

To enable Interrupts, simply set the corresponding bit in the '''Interrupt Mask Set/Read (IMS)''' register. Recommended interrupts are: '''RXT0''' (to receive interrupts about incoming packets), '''RXO''' (to get notified about overruns) and '''LSC''' (to get notified about link status changes, e.g. if the user (un)plugs the ethernet cable. In such cases, you should redo the '''DHCP''' handshake to connect to that network).<syntaxhighlight lang="c" line="1">
void enable_interrupts(){
    uint32_t ims = E1000_IMS_RXT | E1000_IMS_RXO | E1000_IMS_LSC;
    write_register(REG_IMS, ims);
}
</syntaxhighlight>To check why an interrupt was caused, you can check the '''Interrupt Cause Read (ICR)''' register. The '''ICR''' register is self clearing, meaning it will get cleared when you read it.

A simple interrupt handler may look something like this:<syntaxhighlight lang="c" line="1">
void _handle_interrupt(){
        uint32_t cause = read_register(REG_ICR); // Cleared uppon read

        if (cause & IMS_RXT) { // Packets received
            receive_packets();  // Call the function responsible for receiving
                                // packets and sending them to the network stack
        }

        if (cause & IMS_LSC){ // link status change
            // Read the status register and check the LU bit to get the link status
            if (read_register(E1000_REG_STATUS) & STATUS_LU) {
                kprintf("Link change detected: Link up!\n");
            }else{
                kprintf("Link change detected: Link down!\n");
            }
        }
    }
</syntaxhighlight>

==Packet Transmittion==
To transmit a packet, all you have to do is load the data in a free descriptor (or split it if it doesn't fit in one descriptor) and set the '''EOP''' bit on the last descriptor.

In this example we are using preallocated buffers, but you could use dynamically allocated ones. Just remember to free it after the packet is transmitted.<syntaxhighlight lang="c" line="1">
void send_data(void* data, uint32_t size, bool EOP){
    uint32_t tail = read_register(REG_TDT);
    transmit_descriptor_t* tx = transmit_ring + tail; // Get the descriptor the tail is pointing at (next available descriptor)

    memcpy(tx->buffer_address, data, size); // Copy the data to the previously allocated buffer

    tx->length = size; // Set the length of the descriptor

    if (EOP) tx->command |= TX_CMD_EOP | TX_CMD_IFCS; // If its the last one, set EOP
    tail = (tail + 1) % NUM_OF_TX_DESCRIPTORS;
    write_register(REG_TDT, tail); // Increment and write the tail
}

size_t send(void* data, size_t length){
    size_t sent = 0;
    // split the data into chunks and send them
    for (; sent < length;){
        int to_send = min(length - sent, SIZE_OF_TX_DESCRIPTOR_BUFFER);
        send_data((void*)((uint64_t)data + sent), to_send, to_send == (length - sent));
        sent += to_send;
    }
    return sent;
}
</syntaxhighlight>

==Packet Reception==
To receive packets after an interrupt, all you have to do is loop, from the first non-received (by the driver) packet, to the last one. To do that, its a good idea to keep track of the last descriptor the driver read. (You should do this, to reconstruct the packets in the correct order)<syntaxhighlight lang="c" line="1">
uint8_t rx_next = 0;

void receive_packets(){
    uint32_t idx = rx_next;

    void* buffer = nullptr; // use this to store the buffer.
    size_t buffer_len = 0; 

    while (receive_ring[idx].status & RX_STATUS_DD) {
        // This descriptor has been filled
        
        bool eop = receive_ring[idx].status & RX_STATUS_EOP;
        uint16_t len = receive_ring[idx].length;
        void* data = receive_ring[idx].buffer_address;
        
        // Handle multiple-descriptor packets
        if (buffer == nullptr){ // This is the first descriptor of the packet
            buffer = malloc(len); // use your kernel's heap allocator
            buffer_len = len;
            memcpy(buffer, data, len);
        }else{
            // Its the next part of the packet, add it to the packet
            void* new_buffer = malloc(buffer_len + len); // allocate a bigger buffer
            memcpy(new_buffer, buffer, buffer_len); // copy the previous data
            free(buffer); // free the old buffer
    
            // copy the new data
            memcpy((void*)((uint64_t)new_buffer + buffer_len), data, len);
            
            // Set the new buffer into the variables
            buffer_len += len;
            buffer = new_buffer;
        }
    
        // Set status to 0 (To give ownership back to the controller)
        receive_ring[idx].status = 0; 

        idx = (idx + 1) % NUM_OF_RECEIVE_DESCRIPTORS;

        if (eop) {
            // This is the last descriptor of the packet
            // Forward the packet to your network stack
            stack_receive_packet(buffer, buffer_len);
            buffer = nullptr;
            buffer_len = 0;
        }
    }

    // Give the controller more free descriptors by updating RDT
    uint32_t tail = (idx == 0) ? NUM_OF_RECEIVE_DESCRIPTORS - 1 : idx - 1;
    write_register(REG_RDT, tail);

    rx_next = idx;
}
</syntaxhighlight>

==Emulation==
*'''VirtualBox''' (3.1 is all I can personally confirm) supports rather dodgy implementations of an Intel PRO/1000 MT Server (82545EM), Intel PRO/1000 MT Desktop (82540EM), and Intel PRO/1000 T Server (82543GC).  
**''Bugs'':
***The EERD register is unimplemented (you *must* use the 4-wire access method if you want to read from the EEPROM).  [01000101 - I had a patch committed to fix this. It will soon be mainstream]

*'''VMWare Virtual Server 2''' emulates/virtualizes an 82545EM-based card rather well.

*'''[[QEMU]]''' (since 0.10.0) supports an 82540EM-based card and it seems to work OK. It is the default network card since 0.11.0.  
**''Bugs'':
***QEMU does not properly handle the software reset operation (CTRL.RST) in builds prior to June 2009.
***QEMU (version 4.2.1 tested) doesn't seem to support flash memory, instead shifting the IO Register Base Address up.

*IIRC (needs confirmation) '''Microsoft's Hyper-V''' supports an 8254x-series card.

==Documentation==
*[http://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf Intel 8254x Family of Gigabit Ethernet Controllers Software Developer's Manual]
*The [http://www.intel.com/content/dam/www/public/us/en/documents/manuals/pcie-gbe-controllers-open-source-manual.pdf PCIe GbE Controllers Open Source Software Developer’s Manual] may also be of interest. In Linux, the PCIe cards are handled by a separate driver (e1000e) but they appear to be mostly if not entirely compatible with the 8254x series.

==Example driver==
*[https://joscor.com/blog/intel-8254x-ethernet-controller-example-driver/ 01000101's Intel 8254x-series example driver]
*[https://github.com/torokernel/torokernel/blob/7d6df4c40fa4cc85febd5fd5799404592ffdff53/rtl/drivers/E1000.pas Example of a driver for e1000 in Freepascal]

[[Category:Network Hardware]]
[[Category:Standards]]
````

<a id="w002790"></a>

## W002790 — File:IntelPRO1000PT.jpg

Bron-ID: W002790
Namespace: File (6)
Revisie: 9635 | Datum: 2010-01-16T09:27:41Z
Bron: https://wiki.osdev.org/File:IntelPRO1000PT.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AIntelPRO1000PT.jpg&oldid=9635

````text
Intel PRO/1000 PT NIC
````

<a id="w002791"></a>

## W002791 — File:Intel-82540EM.jpg

Bron-ID: W002791
Namespace: File (6)
Revisie: 9639 | Datum: 2010-01-17T02:23:11Z
Bron: https://wiki.osdev.org/File:Intel-82540EM.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AIntel-82540EM.jpg&oldid=9639

````text

````

<a id="w002792"></a>

## W002792 — TLB

Bron-ID: W002792
Namespace: Main (0)
Revisie: 28002 | Datum: 2023-07-09T17:46:29Z
Bron: https://wiki.osdev.org/TLB
Revisie-URL: https://wiki.osdev.org/index.php?title=TLB&oldid=28002

````text
The '''Translation Lookaside Buffer''' ('''TLB''') is a cache of memory page translations employed in many systems with memory paging capability. When the processor needs to translate a given virtual address into a physical address, the TLB is consulted first. On x86 systems, TLB misses are handled transparently by hardware. Only if the page directory/table entry is not present in-core will the operating system be notified (by the means of a page fault exception.)

== Usage implications ==
Like a regular CPU cache, the TLB is ''mostly'' transparent. There are two cases which the operating system must be aware of.

=== Modification of paging structures ===
The TLB is not transparently informed of changes made to paging structures. Therefore the TLB has to be flushed upon such a change. On x86 systems, this can be done by writing to the page directory base register (CR3):
<source lang="asm">
	movl	%cr3,%eax
	movl	%eax,%cr3
</source>
Note: setting the global (G) bit in a page directory/table entry will prevent that entry from being flushed. This is useful for pinning interrupt handlers in place.

An alternate (and better) method is to use the <code>invlpg</code> instruction, which should be used instead of the above method when doing small mapping modifications (creation, removing, changing.) <code>invlpg</code> is mostly used in page unmapping and remapping routines in order to invalidate a previous cached translation. If <code>invlpg</code> or some other TLB flush method had not been used, the mapping would remain cached, producing undefined consequences.

However, please note that the <code>invlpg</code> instruction was introduced in the i486 ISA and is not part of the i386 ISA, thereby requiring a properly written i386-compatible kernel to use conditional inclusion of relevant code at compilation time depending on the target machine. An example routine declaration and source code follow:
<source lang="c">
void vm_page_inval(void *);
</source>
<source lang="asm">
#include <kconfig.h>

	.globl	vm_page_inval
vm_page_inval:
#if	TARGET_MACHINE >= TARGET_MACHINE_I486
	movl	4(%esp),%eax
	invlpg	(%eax)
#else	/* TARGET_MACHINE < TARGET_MACHINE_I486 */
	movl	%cr3,%eax
	movl	%eax,%cr3
#endif
	ret
</source>

Note that changing/reloading CR3 should only need to be done when switching between process address spaces. Using it to completely flush TLBs is really quite overkill in most situations.

=== Multi-processor consistency ===
The above is more complicated in the multi-processor case. If another processor could also be affected by a page table write (because of shared memory, or multiple threads from the same process), you must also flush the TLBs on those processors. This will require some form of inter-processor communication.

== See Also ==
=== External Links ===
* [http://en.wikipedia.org/wiki/Translation_lookaside_buffer The Wikipedia TLB article]

[[Category:X86]]
[[Category:Paging]]
````

<a id="w002793"></a>

## W002793 — MBR

Bron-ID: W002793
Namespace: Main (0)
Revisie: 9682 | Datum: 2010-01-28T20:39:16Z
Bron: https://wiki.osdev.org/MBR
Revisie-URL: https://wiki.osdev.org/index.php?title=MBR&oldid=9682
Doorverwijzing: MBR (x86)

````text
#REDIRECT [[MBR (x86)]]
````

<a id="w002794"></a>

## W002794 — Talk:RTC

Bron-ID: W002794
Namespace: Talk (1)
Revisie: 28424 | Datum: 2023-10-24T10:29:46Z
Bron: https://wiki.osdev.org/Talk:RTC
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ARTC&oldid=28424

````text
== Section levels ==
This article has the wrong number of equals on all the sections, AFAIK -- I didn't fix it when I did my little edit, just in case there is some reason for it.<br>[[User:Bewing|Bewing]] 03:14, 30 January 2010 (UTC)
:Whatever the reason, it's a wiki and is open for editing. It was ugly, so we fixed that :) - [[User:Combuster|Combuster]] 19:48, 1 February 2010 (UTC)
== IRQ Danger ==
>Since the IRQ number is 8, it has
<br>as i see priority of TWO, not EIGHT; so only TIMER0 and KEYB1 have default priority higher then IRQ8.
<br>[[User:Grizlyk|Grizlyk]] 05:29, 24 October 2023 (CDT)
````

<a id="w002795"></a>

## W002795 — VBE

Bron-ID: W002795
Namespace: Main (0)
Revisie: 20684 | Datum: 2017-04-10T17:40:58Z
Bron: https://wiki.osdev.org/VBE
Revisie-URL: https://wiki.osdev.org/index.php?title=VBE&oldid=20684
Doorverwijzing: VESA Video Modes

````text
#REDIRECT [[VESA Video Modes]]
````

<a id="w002796"></a>

## W002796 — User talk:Combuster/UDI Graphics

Bron-ID: W002796
Namespace: User talk (3)
Revisie: 18919 | Datum: 2016-02-09T03:55:13Z
Bron: https://wiki.osdev.org/User_talk:Combuster/UDI_Graphics
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ACombuster%2FUDI_Graphics&oldid=18919

````text
How should multiple displays be handled? --[[User:Love4boobies|Love4boobies]] 09:49, 2 February 2010 (UTC)
:Where did I mention a card can only have '''one''' connector? A video card has 1+ connectors and 1+ display pipes containing 1+ display engines. The end of a display pipe can in design be attached from zero to all connectors on the card. A state property will govern where a connector gets its graphics data from. - [[User:Combuster|Combuster]] 10:26, 2 February 2010 (UTC)

== UDI repository ==
Hi, I'm trying to fix UDI by patching pieces together and fixing changed stuff. I've included udi_gfx.h (license: public domain) to the resulting repository which is available at http://hg.ookoo.org/udi/ . Any help is welcome (I'm currently fixing the build process and will try to get udi compile on recent linux systems first). I'll keep a look here for evolutions on udi_gfx.h  [[User:Magicaltux|Magicaltux]] 22:15, 22 April 2010 (UTC)

:It seems UDI repos suddenly appear everywhere. Solar has kindly provided a repository for continuing work on the reference implementation but there's nothing there yet - I'm currently helping Combuster with his environment while he is developing UDI GFX. There's a little more official UDI stuff going on which you already know about from IRC and hopefully everyone will find out about soon. You probably won't have any use for udi_gfx.h yet, though. Also, note that the USB part was never merged into the reference implementation - even the specification contains a few glitches because it was originally written for UDI 1.0 (which cannot be fully implemented). USB 3.0 support would also need looking over (not much work there, though) - unfortunately, I don't have close to no experience with USB so unless we find someone else to do it, it will have to wait.--[[User:Love4boobies|Love4boobies]] 12:54, 23 April 2010 (UTC)

== Displayport? ==

Can we agree displayport is the new DVI?
````

<a id="w002797"></a>

## W002797 — Crc32

Bron-ID: W002797
Namespace: Main (0)
Revisie: 9714 | Datum: 2010-02-07T03:54:09Z
Bron: https://wiki.osdev.org/Crc32
Revisie-URL: https://wiki.osdev.org/index.php?title=Crc32&oldid=9714
Doorverwijzing: CRC32

````text
#REDIRECT [[CRC32]]
````

<a id="w002798"></a>

## W002798 — User:Love4boobies

Bron-ID: W002798
Namespace: User (2)
Revisie: 14042 | Datum: 2012-10-17T12:17:14Z
Bron: https://wiki.osdev.org/User:Love4boobies
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALove4boobies&oldid=14042

````text
Hi. My name is Bogdan Barbu and I've created this page as a temporary home for my work with UDI that's relevant to the community. I've decided to use this wiki for 2 reasons: to make the pages easy to reach and to encourage people to contribute. I will check discussion pages every so often.

If you don't know what UDI is, you might want to read the dedicated article [[Uniform Driver Interface|here]].

[[Special:Contributions/Love4boobies|Here]] is a list of all my contributions to the wiki.

[http://cs-made-easy.blogspot.com/ Here] is my blog.

== Specifications ==

[https://docs.google.com/leaf?id=0B3hgbS9lMCEgODVlZWQ1YzAtZDljMS00OTJkLWIyY2YtNDc4OTliYWM2N2Ji&sort=name&layout=list&num=50|UDI AMD64 ABI Binding Specification]

= Drafts =

There used to be an ARM ABI binding specification here but it was deleted in favour of [[User:Owen|Owen]]'s, which can be found [http://gitorious.org/udi/specs/blobs/raw/master/arm_abi_spec.pdf here].

== Misc ==

{{Special:Prefixindex/User:Love4boobies/}}
````

<a id="w002800"></a>

## W002800 — User:Ferrarius

Bron-ID: W002800
Namespace: User (2)
Revisie: 9745 | Datum: 2010-02-11T12:04:20Z
Bron: https://wiki.osdev.org/User:Ferrarius
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AFerrarius&oldid=9745

````text
I'm Pascal Smit (Hadrianius Ferrarius). I'm trying to write an OS in the language Lyric, written by myself.
````

<a id="w002803"></a>

## W002803 — GRUB

Bron-ID: W002803
Namespace: Main (0)
Revisie: 30711 | Datum: 2026-06-10T11:30:45Z
Bron: https://wiki.osdev.org/GRUB
Revisie-URL: https://wiki.osdev.org/index.php?title=GRUB&oldid=30711

````text
{{TutorialTone}}
'''GRUB''' is the GNU project's bootloader. The current version 2 series have a more complete feature set than [[GRUB Legacy|GRUB 0.97]] (commonly referred to as "GRUB Legacy"). GRUB is a popular and tried bootloader among open source and hobby operating systems, and supports many boot protocols, notably (for hobby osdev) [[Multiboot]] and [[Linux Boot Protocol]], as well as others used by BSDs, DOS, and so on. 

== History ==

GRUB version 2 started its life as the [http://www.nongnu.org/pupa/ PUPA] (note the pun) research project and was rewritten from the ground up. Since then GRUB 2 (actually at time of update 1.97) has grown more stable and even hobby operating systems use of the new bootloader instead of GRUB Legacy.

== Features ==

* Basic scripting support
* GUI (better bootsplash support, custom colors, custom themes, ...)
* Memory management
* Cleaner design
* Better portability
* Internationalization
* Rescue mode

A complete list of features can be found on the GNU mailing lists [https://lists.gnu.org/archive/html/grub-devel/2012-06/msg00093.html]

== Using GRUB to boot your OS==

Complete example code for booting your operating system with GRUB can be found in the [[Bare_Bones#Booting_the_Operating_System|Bare Bones tutorial]]. The general idea is that you want to create a file that has [[Multiboot#Header_Format|Multiboot header]] which GRUB can use to identity your program as a kernel and boot it.

== Upgrading from GRUB Legacy ==
''WARNING: These steps have not been tested very well yet. Use at your own risk!''

Since modern GRUB 2 is very different from GRUB Legacy, the directions for getting your kernel up and running are different. Modern GRUB differs from GRUB Legacy in that to implement all but the most basic functionality, the user must load so-called "modules": little bits of code that add components (e.g. a different file system or a VGA font). This section gives you an overview of the process you need to go through when you want to have GRUB 2 load your kernel. It's actually rather simple to create a GRUB 2 image (assuming you have GRUB 2 either built or installed):

=== ISO instructions ===
There have been a lot of tries to make Grub2 work good with ISOs, but mostly failed. The only combination of commands that seems to work is the following.

First create a directory tree called "iso", where you put your kernel (and any other needed files) somewhere. Then in the boot/grub subdirectory create the grub.cfg file which is your configuration.

Now run:
 grub-mkrescue -o bootable.iso iso

Be sure that your grub.cfg is syntactically correct. A common mistake is to put the menuentry brace on newline. It '''must''' be like:
 menuentry "Place your os name here" {
 }

grub-mkrescue depends on program '''xorriso''' with version '''0.5.6 or higher'''.

If you cannot get it as binary (possibly from a package named "libisoburn")
then get the all-in-one source tarball from [http://www.gnu.org/software/xorriso/#download GNU xorriso homepage].
GNU xorriso can be used where it gets built, without further installation:
  grub-mkrescue --xorriso=/...full.path.../xorriso/xorriso -o bootable.iso iso

=== Floppy instructions ===
<source lang="bash">
mkdir tmp
grub-mkimage -p /boot -o tmp/core.img multiboot sh fat # This should work... I hope :D
</source>

==== Explanation ====
Let's go through those <tt>grub-mkimage</tt> options:

{| {{wikitable}}
|-
| '''-p'''
| By default, GRUB 2 looks in <tt>/boot/grub</tt> for its configuration file. '''-p''' changes this.
|-
| '''-o'''
| Like so many other GNU tools, <tt>grub-mkimage</tt> uses '''-o''' to set the output file. By default, it's <tt>stdout</tt>.
|-
| '''multiboot'''
| This module is required to load multiboot-compliant kernels.
|-
| '''biosdisk'''
| This module is required for GRUB 2 to be able to boot from a LiveCD.
|-
| '''iso9660/fat'''
| Allows GRUB 2 to look on the image for different files.
|- 
|'''sh'''
| This module allows GRUB to parse the configuration file.
|}

GRUB 2, like GRUB Legacy, needs a configuration file to find your kernel. In GRUB Legacy it's called '''menu.lst''', but in GRUB2, it's called '''grub.cfg'''. The syntax for the configuration file is also a bit different.

Here's a sample configuration file (NOTE: This file should be placed into the <tt>/boot/grub</tt> folder of your disk image, and be named '''grub.cfg'''):

<source lang="bash">
set timeout=15
set default=0 # Set the default menu entry

menuentry "OS Name" {
   # The multiboot command replaces the kernel command
   # For multiboot v2, use the multiboot2 command
   multiboot /boot/kernel-file
   boot
}
</source>

That's basically it. Copy these files to a disk image, pop it in an emulator, and you're done!

Double check that you put the brace on the same line of "menuentry". It can't be on a new line. This is not C.

=== USB instructions ===
Fewer and fewer systems have a floppy disc controller these days, but USB ports are found on all. Modern BIOSes can boot from a USB device, usually by pressing some special key during startup.

Putting GRUB 2 on a bootable USB storage device is a nice way to experiment with your OS on different computers. Here's how you set this up (using Linux):

1. Create a FAT32-formatted USB disk, without partitions:

'''Warning: the following command uses superuser privileges (sudo). E.g. just typing the wrong character for X could cause severe troubles for your current system'''
<pre>sudo mkfs.vfat -F 32 -n YourLabel -I /dev/sdX</pre> (where sdX is your USB device)

The "-I" option is needed because we are targeting a partition-less device

2. Remove your USB device, and plug it back in. The auto-mounter on your OS should detect it now.

3. Invoke grub-install (on some systems this command is called grub2-install, located under /usr/sbin or /usr/local/sbin)
<pre>sudo grub-install --root-directory=/media/YourLabel --no-floppy --recheck --force /dev/sdX</pre>

It is important to do this as root (or sudo), else the generated device.map listing available boot devices can be empty.
/media/YourLabel is the mount point under Fedora 16, it may be different for other distributions.

4. Create a grub.cfg for your kernel (see above), and copy it to your new bootable USB disk

=== Disk image instructions ===
For a more detailed tutorial, see [[Bootable_Disk|bootable disk]] creation.

Hobby operating systems don't have to use real devices when running on virtual machines (although it may be, and it usually is faster). Creating bootable GRUB disk image is similar to installing it on USB devices, but here you're working with image itself and partition device at once. 

1. Create new disk image file
<pre>$ dd if=/dev/zero of=disk.img bs=512 count=131072
131072+0 records in
131072+0 records out
67108864 bytes (67 MB) copied, 0.349436 s, 192 MB/s
</pre>

2. Create new DOS partition table with bootable entry for your filesystem
<pre>$ fdisk disk.img</pre>
Add new partition, starting at 1MB (2048th sector). This is more space than GRUB actually needs.
<pre>
Command (m for help): n
Partition type:
   p   primary (0 primary, 0 extended, 4 free)
   e   extended
Select (default p): p
Partition number (1-4, default 1): 1
First sector (2048-131071, default 2048): 
Using default value 2048
Last sector, +sectors or +size{K,M,G} (2048-131071, default 131071): 
Using default value 131071
</pre>
Make it bootable
<pre>
Command (m for help): a 
Partition number (1-4): 1
</pre>
Write the new partition table to disk
<pre>
Command (m for help): w
The partition table has been altered!

Syncing disks.
</pre>

3. Setup two loop devices. One will be used for writing GRUB and its additional codes to MBR, and the second will be used for mounting filesystem of your operating system. 
<pre>
$ sudo losetup /dev/loop0 disk.img
$ sudo losetup /dev/loop1 disk.img -o 1048576
</pre>
''-o'' option defines offset from start of the file. Number ''1048576'' is actually ''1024^2 = 1MB'' and that's the start of your partition.

4. Format your partition
You can simply use [[http://www.gnu.org/software/grub/manual/grub.html#Features any supported filesystem]] like ext2 or FAT32.
<pre>$ sudo mke2fs /dev/loop1</pre>
<pre>$ sudo mkdosfs -F32 -f 2 /dev/loop1</pre>

5. Mount your newly formatted partition
<pre>$ sudo mount /dev/loop1 /mnt</pre>
Note that if you tried to mount your first loop device which doesn't have any offset set, you would be requested to specify filesystem and even if you did it, you wouldn't get the expected result.

7. Install GRUB using ''grub-install''
<pre>sudo grub-install --root-directory=/mnt --no-floppy --modules="normal part_msdos ext2 multiboot biosdev" /dev/loop0</pre>
If you mistyped ''/dev/loop1'' (pointing on your partition) instead of ''/dev/loop0'' (pointing on your MBR), you would receive message that ''grub-install'' can't use 'embedding' (because there's no space for it) and that it would like to use 'block lists', which are not recomended.

Don't forget to flush the filesystem buffers when manipulating with files on mounted disk image. On a Unix-like system, this can be simply done by executing the <tt>sync</tt> program in your shell.

== HDD Image Instructions for OS X users==
It might be useful to create an image file for a HDD; The following instructions help you create a HDD with an MBR partition map.
Based on information from <palk> on #osdev, it is slightly more complicated to create a HDD image than you'd expect.

This information might not be applicable to Linux users, who will most probably want to use a loopback device. This is for developers on OS X, which doesn't
have a loopback device and has a finicky image mounter.

1. First, create a blank, raw image with DD, with the required size. Here, I'll make a 80MB image -- 163840 sectors of 512 bytes.
<source lang="bash">
dd if=/dev/zero of=disk.img count=163840 bs=512
</source>

2. For an explaination on calculating the CHS values from the LBA / size see [[Floppy_Disk_Controller#CHS]]
    Since we have a 80MB disk, the CHS values are 78, 32 and 63 respectively.

3. Fire up FDISK (or your tool of choice) -- I'm using the OS X version here, so commands may differ. The concept is essentially identical.

What will show on your screen (in OS X anyway) is on the left, what you enter is on the right.
<source lang="bash">
fdisk -e disk.img


The signature for this MBR is invalid.
Would you like to initialize the partition table? [y]           Yes
fdisk:*1>                                                       disk
Disk: disk.img	geometry: 650/4/63 [16340 sectors]
Change disk geometry? [n]                                       No



fdisk:*1>                                                       edit 1
Partition id ('0' to disable)  [0 - FF]: [B] (? for help)       0B
Do you wish to edit in CHS mode? [n]                            No
Partition offset [0 - 163840]: [63]                             2047
Partition size [1 - 161793]: [161793]                           <Enter>


fdisk:*1>                                                       write
fdisk:*1>                                                       quit
</source>

4. Now that the MBR Partition Table is initialised, you'll want to make a Filesystem on the disk. But first.
    Here, we separate the MBR bit, and the actual FS bit.
<source lang="bash">
dd if=disk.img of=mbr.img bs=512 count=2047
dd if=disk.img of=fs.img bs=512 skip=2047
</source>

5. Because we're on OS X, we need to attach the disk image first, without actually mounting it.
<source lang="bash">
hdiutil attach -nomount fs.img
</source>

6. Use 'diskutil list' to find out which device your image is, use that below.

7. Now, make a FAT12/16/32 filesystem on the FS.img disk. Remember, use FS.img -- not disk.img
<source lang="bash">
newfs_msdos -F 32 /dev/diskX
</source>

8. Now you'll want to unmount it, then recombine the two images, then install GRUB.
<source lang="bash">
hdiutil detach /dev/diskX
cat mbr.img fs.img > disk.img

hdiutil attach disk.img
# note the mount point here (/Volumes/NO NAME, probably)

grub-install --modules="part_msdos biosdisk fat multiboot configfile" --root-directory="/Volumes/NO NAME" disk.img
Installation finished. No error reported.
</source>

And there it is! You know have a disk.img, that will have GRUB 2 installed, ready to go. It should be mountable in OS X simply by double clicking (or with the mount command).
Enjoy!

=== Additional useful options ===
Whatever device you are using, you may want to have a PC partition table and create a partition you format in one of the filesystems supported by GRUB. If you do, be sure to add the following option the grub-install arguments:
 --modules="part_msdos"

In general, if GRUB 2 happens to fail to do what you want and you suspect that it needs some missing functionality, just try to add a module name you believe has the functionality you need to the --modules argument. The module files generally are in /boot/grub/i386-pc/.

== Multiboot ==

Some versions of GRUB 2 like to put Multiboot modules in relatively high physical memory addresses, in contrast to GRUB-legacy which loaded them into low memory.  Be careful when making your kernel work with GRUB 2 that it is not making any assumptions about where the Multiboot modules will appear.

When your kernel gets control, the machine state is defined as follows: [http://www.gnu.org/software/grub/manual/multiboot/html_node/Machine-state.html Multiboot machine state]. Your code should have minimal dependency on this initial state; for example, define your own GDT instead of relying on the GDT setup by GRUB.

=== Header ===
As the GRUB 2 manual puts it:

{{quote|The primary requirement for GRUB is that it be compliant with the ''Multiboot Specification''.}}

But the Multiboot header as used by older versions of GRUB 2 (field is present in GRUB 1.99 and newer) did ''not'' include the ''header_length'' field that is specified in the [http://nongnu.askapache.com/grub/phcoder/multiboot.pdf Multiboot 1.6 specification].

GRUB 2 also supports the old [https://www.gnu.org/software/grub/manual/multiboot/multiboot.html Multiboot 0.6.96 specification]. It is possible to include both headers.

== Installing GRUB 2 on OS X ==
The installation of GRUB 2 on OS X is a little tricky. The latest released version 2.00 (as of 7th of October, 2014) doesn't seem to work with any configuration. The developer team fixed this in the newer revisions.

'''Important:''' To build GRUB 2 so that it can produce a bootloader for your output target you need to have a compiler for that target. So for example, if you want a bootloader for i386-elf (as suggested in the bare bones) you'll need Binutils + a compiler for that target. This is required on OS X because the built-in LLVM doesn't know how to make i386-elf binaries. So you can either [http://wiki.osdev.org/GCC_Cross-Compiler build a cross-compiler] or [http://wiki.osdev.org/OS_Specific_Toolchain create an OS-specific toolchain] (recommended) for your target platform.
'''You will need the cross-versions of GCC, objcopy, strip, nm and ranlib in step 4.'''

'''1.''' Clone the developer version of the sources:
 <source lang="bash">git clone git://git.savannah.gnu.org/grub.git</source>
(This was tested on revision: 77063f4cb672f423272db7e21ca448cf3de98dcf)

'''2.''' A tool named '''objconv''' is required, get it from:
 https://github.com/vertis/objconv
Download sources, compile (see website for details) and make available in your PATH.

'''3.''' Run "autogen.sh" in the GRUB sources folder

'''4.''' Create a seperate build directory, switch to it, and run GRUB's configure script ''(insert your target-specific tools!)'':

 ../grub/configure --disable-werror TARGET_CC=i386-elf-gcc TARGET_OBJCOPY=i386-elf-objcopy \
 TARGET_STRIP=i386-elf-strip TARGET_NM=i386-elf-nm TARGET_RANLIB=i386-elf-ranlib --target=i386-elf

'''5.''' Run "make" and "make install"

Now you have a working GRUB 2 that has the required files to build an image that boots on i386 platforms.

==GRUB for UEFI==
===Compiling GRUB===
Older GRUB versions are riddled with nasty bugs. As this probably includes the version from your package management, you should be compiling GRUB from source. As we're compiling for UEFI, you should pass the appropriate flags to configure. An example invocation might look something like this:
<source lang="text">
../grub-2.02~rc2/configure --prefix="$HOME/opt/grub" --target=x86_64 --with-platform=efi
</source>

After completing the build, GRUB refused to do anything as it was missing a font file. To fix this, run
<source lang="text">
bin/grub-mkfont -o share/grub/unicode.pf2 /usr/share/fonts/truetype/unifont/unifont.ttf
</source>

GRUB might warn you about share/locale/ missing. To solution is to create the missing directory.

===Building a GRUB UEFI binary (BOOTX64.EFI)===
This method builds a standalone GRUB binary you can copy to a FAT partition. However, note that some UEFI implementations assume that it is located at <code>/EFI/BOOT/BOOTX64.EFI</code> for x86_64 platforms.

Generally, all compiled modules are included in this binary; if you want to cut down on its size, you can specify what modules to include.

This method uses two separate configuration files; this is needed as all GRUB data is located within the binary, but we'll be working around that. The binary contains a memdisk, which also serves as the prefix. As far as I'm aware, there's no way of working around this fact directly. We can, however, use the memdisk grub.cfg to load a configuration file from the disk. This way, we don't have to recreate the binary for every configuration change.

Start off with creating the memdisk grub.cfg. I saved it at build/grub.cfg (we'll need this path later on):
<source lang="text">
insmod part_msdos
configfile (hd0,msdos1)/boot/grub/grub.cfg
</source>
All this does is loading the module for reading the disk partitions (line 1) and loading the configuration file from the disk (line 2). Note that it doesn't have to be located at /boot/grub/grub.cfg; you can change this path on the disk to whatever you like, just remember to apply the changes to the memdisk grub.cfg.

The second grub.cfg (the one on the disk) is pretty much up to you, except that you have to load the part_msdos module and set the root (which by default is memdisk):
<source lang="text">
insmod part_msdos
set root=(hd0,msdos1)
</source>
Add your regular GRUB2 configuration below the <code>set root</code> line.

Finally, all that's left is to create the binary. Note that you have to specify what file to include at what path in the memdisk:
<source lang="text">
bin/grub-mkstandalone -O x86_64-efi -o BOOTX64.EFI "boot/grub/grub.cfg=build/grub.cfg"
</source>

===Blessing the binary on macOS===
Macs require bootable binaries to be 'blessed' by a utility:
<source lang="text">
bless --verbose --folder=/Volumes/EFI --file=/Volumes/EFI/EFI/BOOT/BOOTX64.EFI --setBoot
</source>

== See Also ==

=== Articles ===

* [[GRUB Legacy]]
* [[Bootable El-Torito CD with GRUB Legacy]]
* [[Multiboot]]
* [[BOOTBOOT]] an alternative boot loader with exactly the opposite philosophy as GRUB's
* [[Limine]] another boot loader supporting Multiboot

=== External Links ===

* [http://www.gnu.org/software/grub/ GRUB 2 Homepage]

[[Category:Bootloaders]]
[[Category:GRUB]]
[[Category:UEFI]]

[[de:Grand_Unified_Bootloader]]
````

<a id="w002804"></a>

## W002804 — Talk:GRUB

Bron-ID: W002804
Namespace: Talk (1)
Revisie: 19453 | Datum: 2016-07-14T15:14:48Z
Bron: https://wiki.osdev.org/Talk:GRUB
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AGRUB&oldid=19453

````text
Feel free to add grammar corrections and correct any errors.

--[[User:Kotuxguy|Kotuxguy]] 17:17, 13 February 2010 (UTC)

* Could you probably be more specific with an example how you created such an image? I tried your example to create a floppy image, but it won't boot with qemu for me. It's most likely totaly simple... --[[User:Arbaal|Arbaal]]
** Okay, I found another nice method: "grub-mkrescue --image-type=floppy --modules=multiboot --overlay=floppy/ floppy.img" This will create you a floppy image, which will contain the overlay from the folder "floppy/". When you put a normal grub folder structure in there (like: floppy/boot/grub) and store your kernel.bin and grub.conf in there, it will be loaded fine. You can even create a CD-image. --[[User:Arbaal|Arbaal]] 20:21, 9 March 2010 (UTC)


* Yes, that will work too. Although, I'm not sure if you can have <tt>grub.cfg</tt>. Which would defeat the purpose of using GRUB 2, as you could not enable/configure VESA, load a Lua script for parsing, etc..

--[[User:Kotuxguy|Kotuxguy]] 22:44, 13 March 2010 (UTC)

== Install GRUB2 to a disk image ==

It's difficult if not impossible to install GRUB2 to a disk image. If you want to install GRUB2 to a disk image, you can first install GRUB Legacy on the disk image(google this if you don't know how), then generate a core.img, put the core.img and other .mod files into the disk image and use GRUB Legacy's "kernel" command to load core.img, then you got a fully functional GRUB2.
-- torshie

* AFAIK, this is a necessary evil :( --[[User:Kotuxguy|Kotuxguy]] 16:30, 18 March 2010 (UTC)

== FreeBSD ==

I can't get grub-mkimage to successfully execute on FreeBSD using any of the examples provided on this page. Is anyone else experiencing this? --[[User:Comradeslice|Comradeslice]] 14:34, 10 April 2011 (UTC)

BSD is relatively exotic, and the wiki is the wrong place for asking help since the people who read this can be counted on one pair of hands. Please try the manual for BSD-specific instructions and if that does not get you further ask on the forums with a reference to all things you have tried so far. - [[User:Combuster|Combuster]] 19:50, 10 April 2011 (UTC)
````

<a id="w002805"></a>

## W002805 — Cross-Compiler Successful Builds

Bron-ID: W002805
Namespace: Main (0)
Revisie: 30946 | Datum: 2026-09-12T19:11:02Z
Bron: https://wiki.osdev.org/Cross-Compiler_Successful_Builds
Revisie-URL: https://wiki.osdev.org/index.php?title=Cross-Compiler_Successful_Builds&oldid=30946

````text
This page is a summary of successful builds of a [[GCC Cross-Compiler]] by members of OSDev.org. The numbers refer to the versions being built, not the host compiler doing the build.

== GCC Version 16.x.x ==

{| {{wikitable}}
|- style="height: 6em;"
! &darr; [[GCC]] / [[Binutils]] &rarr;
! style="transform: rotate(-90deg);max-width: 2em;" | 2.46.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.46.0
|-
! 16.1.0
| {{Yes}}
| {{Yes}}
|}

== GCC Version 15.x.x ==

{| {{wikitable}}
|- style="height: 6em;"
! &darr; [[GCC]] / [[Binutils]] &rarr;
! style="transform: rotate(-90deg);max-width: 2em;" | 2.45
|-
! 15.2.0
| {{Yes}}
|-
! 15.1.0
| {{Yes}}
|}


== GCC Version 14.x.x ==

{| {{wikitable}}
|- style="height: 6em;"
! &darr; [[GCC]] / [[Binutils]] &rarr;
! style="transform: rotate(-90deg);max-width: 2em;" | 2.43.1
|-
! 14.2.0
| {{Yes}}
|-
! 14.1.0
| {{Maybe|?}}
|}

== GCC Version 13.x.x ==

{| {{wikitable}}
|- style="height: 6em;"
! &darr; [[GCC]] / [[Binutils]] &rarr;
! style="transform: rotate(-90deg);max-width: 2em;" | 2.41
! style="transform: rotate(-90deg);max-width: 2em;" | 2.40
! style="transform: rotate(-90deg);max-width: 2em;" | 2.38
! style="transform: rotate(-90deg);max-width: 2em;" | 2.37
! style="transform: rotate(-90deg);max-width: 2em;" | 2.36.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.36
|-
! 13.2.0
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 13.1.0
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|}

== GCC Version 12.x.x ==

{| {{wikitable}}
|- style="height: 6em;"
! &darr; [[GCC]] / [[Binutils]] &rarr;
! style="transform: rotate(-90deg);max-width: 2em;" | 2.39
! style="transform: rotate(-90deg);max-width: 2em;" | 2.38
! style="transform: rotate(-90deg);max-width: 2em;" | 2.37
! style="transform: rotate(-90deg);max-width: 2em;" | 2.36.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.36
! style="transform: rotate(-90deg);max-width: 2em;" | 2.35.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.35
! style="transform: rotate(-90deg);max-width: 2em;" | 2.34
! style="transform: rotate(-90deg);max-width: 2em;" | 2.33.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.33
! style="transform: rotate(-90deg);max-width: 2em;" | 2.32.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.32
! style="transform: rotate(-90deg);max-width: 2em;" | 2.31.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.31
! style="transform: rotate(-90deg);max-width: 2em;" | 2.30
|-
! 12.2.0
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 12.1.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|}

== GCC Version 11.x.x ==

{| {{wikitable}}
|- style="height: 6em;"
! &darr; [[GCC]] / [[Binutils]] &rarr;
! style="transform: rotate(-90deg);max-width: 2em;" | 2.38
! style="transform: rotate(-90deg);max-width: 2em;" | 2.37
! style="transform: rotate(-90deg);max-width: 2em;" | 2.36.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.36
! style="transform: rotate(-90deg);max-width: 2em;" | 2.35.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.35
! style="transform: rotate(-90deg);max-width: 2em;" | 2.34
! style="transform: rotate(-90deg);max-width: 2em;" | 2.33.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.33
! style="transform: rotate(-90deg);max-width: 2em;" | 2.32.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.32
! style="transform: rotate(-90deg);max-width: 2em;" | 2.31.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.31
! style="transform: rotate(-90deg);max-width: 2em;" | 2.30
! style="transform: rotate(-90deg);max-width: 2em;" | 2.29.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.29
! style="transform: rotate(-90deg);max-width: 2em;" | 2.28.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.28
! style="transform: rotate(-90deg);max-width: 2em;" | 2.27
! style="transform: rotate(-90deg);max-width: 2em;" | 2.26
! style="transform: rotate(-90deg);max-width: 2em;" | 2.25.1 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.25 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.24 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23.2
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23
! style="transform: rotate(-90deg);max-width: 2em;" | 2.22
! style="transform: rotate(-90deg);max-width: 2em;" | 2.21.1a
! style="transform: rotate(-90deg);max-width: 2em;" | 2.21
! style="transform: rotate(-90deg);max-width: 2em;" | 2.20.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.20
|-
! 11.3.0
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 11.2.0
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 11.1.0
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|}

== GCC Version 10.x.x ==

{| {{wikitable}}
|- style="height: 6em;"
! &darr; [[GCC]] / [[Binutils]] &rarr;
! style="transform: rotate(-90deg);max-width: 2em;" | 2.36.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.36
! style="transform: rotate(-90deg);max-width: 2em;" | 2.35.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.35
! style="transform: rotate(-90deg);max-width: 2em;" | 2.34
! style="transform: rotate(-90deg);max-width: 2em;" | 2.33.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.33
! style="transform: rotate(-90deg);max-width: 2em;" | 2.32.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.32
! style="transform: rotate(-90deg);max-width: 2em;" | 2.31.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.31
! style="transform: rotate(-90deg);max-width: 2em;" | 2.30
! style="transform: rotate(-90deg);max-width: 2em;" | 2.29.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.29
! style="transform: rotate(-90deg);max-width: 2em;" | 2.28.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.28
! style="transform: rotate(-90deg);max-width: 2em;" | 2.27
! style="transform: rotate(-90deg);max-width: 2em;" | 2.26
! style="transform: rotate(-90deg);max-width: 2em;" | 2.25.1 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.25 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.24 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23.2
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23
! style="transform: rotate(-90deg);max-width: 2em;" | 2.22
! style="transform: rotate(-90deg);max-width: 2em;" | 2.21.1a
! style="transform: rotate(-90deg);max-width: 2em;" | 2.21
! style="transform: rotate(-90deg);max-width: 2em;" | 2.20.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.20
|-
! 10.2.0
| {{Yes}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 10.1.0
| {{Yes}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|}

== GCC Version 9.x.x ==

{| {{wikitable}}
|- style="height: 6em;"
! &darr; [[GCC]] / [[Binutils]] &rarr;
! style="transform: rotate(-90deg);max-width: 2em;" | 2.34
! style="transform: rotate(-90deg);max-width: 2em;" | 2.33.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.33
! style="transform: rotate(-90deg);max-width: 2em;" | 2.32.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.32
! style="transform: rotate(-90deg);max-width: 2em;" | 2.31.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.31
! style="transform: rotate(-90deg);max-width: 2em;" | 2.30
! style="transform: rotate(-90deg);max-width: 2em;" | 2.29.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.29
! style="transform: rotate(-90deg);max-width: 2em;" | 2.28.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.28
! style="transform: rotate(-90deg);max-width: 2em;" | 2.27
! style="transform: rotate(-90deg);max-width: 2em;" | 2.26
! style="transform: rotate(-90deg);max-width: 2em;" | 2.25.1 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.25 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.24 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23.2
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23
! style="transform: rotate(-90deg);max-width: 2em;" | 2.22
! style="transform: rotate(-90deg);max-width: 2em;" | 2.21.1a
! style="transform: rotate(-90deg);max-width: 2em;" | 2.21
! style="transform: rotate(-90deg);max-width: 2em;" | 2.20.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.20
|-
! 9.3.0
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 9.2.1
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 9.2.0
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|}

== GCC Version 8.x.x ==

{| {{wikitable}}
|- style="height: 6em;"
! &darr; [[GCC]] / [[Binutils]] &rarr;
! style="transform: rotate(-90deg);max-width: 2em;" | 2.32
! style="transform: rotate(-90deg);max-width: 2em;" | 2.31.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.31
! style="transform: rotate(-90deg);max-width: 2em;" | 2.30
! style="transform: rotate(-90deg);max-width: 2em;" | 2.29.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.29
! style="transform: rotate(-90deg);max-width: 2em;" | 2.28.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.28
! style="transform: rotate(-90deg);max-width: 2em;" | 2.27
! style="transform: rotate(-90deg);max-width: 2em;" | 2.26
! style="transform: rotate(-90deg);max-width: 2em;" | 2.25.1 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.25 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.24 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23.2
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23
! style="transform: rotate(-90deg);max-width: 2em;" | 2.22
! style="transform: rotate(-90deg);max-width: 2em;" | 2.21.1a
! style="transform: rotate(-90deg);max-width: 2em;" | 2.21
! style="transform: rotate(-90deg);max-width: 2em;" | 2.20.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.20
|-
! 8.3.0
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 8.2.0
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 8.1.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|}

== GCC Version 7.x.x ==

{| {{wikitable}}
|- style="height: 6em;"
! &darr; [[GCC]] / [[Binutils]] &rarr;
! style="transform: rotate(-90deg);max-width: 2em;" | 2.33.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.32
! style="transform: rotate(-90deg);max-width: 2em;" | 2.31.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.31
! style="transform: rotate(-90deg);max-width: 2em;" | 2.30
! style="transform: rotate(-90deg);max-width: 2em;" | 2.29.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.29
! style="transform: rotate(-90deg);max-width: 2em;" | 2.28.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.28
! style="transform: rotate(-90deg);max-width: 2em;" | 2.27
! style="transform: rotate(-90deg);max-width: 2em;" | 2.26
! style="transform: rotate(-90deg);max-width: 2em;" | 2.25.1 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.25 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.24 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23.2
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23
! style="transform: rotate(-90deg);max-width: 2em;" | 2.22
! style="transform: rotate(-90deg);max-width: 2em;" | 2.21.1a
! style="transform: rotate(-90deg);max-width: 2em;" | 2.21
! style="transform: rotate(-90deg);max-width: 2em;" | 2.20.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.20
|-
! 7.5.0
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 7.4.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 7.3.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 7.2.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 7.1.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
|}

== GCC Version 6.x.x ==

{| {{wikitable}}
|- style="height: 6em;"
! &darr; [[GCC]] / [[Binutils]] &rarr;
! style="transform: rotate(-90deg);max-width: 2em;" | 2.30
! style="transform: rotate(-90deg);max-width: 2em;" | 2.29.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.29
! style="transform: rotate(-90deg);max-width: 2em;" | 2.28.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.28
! style="transform: rotate(-90deg);max-width: 2em;" | 2.27
! style="transform: rotate(-90deg);max-width: 2em;" | 2.26
! style="transform: rotate(-90deg);max-width: 2em;" | 2.25.1 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.25 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.24 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23.2
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23
! style="transform: rotate(-90deg);max-width: 2em;" | 2.22
! style="transform: rotate(-90deg);max-width: 2em;" | 2.21.1a
! style="transform: rotate(-90deg);max-width: 2em;" | 2.21
! style="transform: rotate(-90deg);max-width: 2em;" | 2.20.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.20
|-
! 6.4.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 6.3.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 6.2.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 6.1.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
|}

== GCC Version 5.x.x ==

{| {{wikitable}}
|- style="height: 6em;"
! &darr; [[GCC]] / [[Binutils]] &rarr;
! style="transform: rotate(-90deg);max-width: 2em;" | 2.30
! style="transform: rotate(-90deg);max-width: 2em;" | 2.29.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.29
! style="transform: rotate(-90deg);max-width: 2em;" | 2.28.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.28
! style="transform: rotate(-90deg);max-width: 2em;" | 2.27
! style="transform: rotate(-90deg);max-width: 2em;" | 2.26
! style="transform: rotate(-90deg);max-width: 2em;" | 2.25.1 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.25 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.24 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23.2
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23
! style="transform: rotate(-90deg);max-width: 2em;" | 2.22
! style="transform: rotate(-90deg);max-width: 2em;" | 2.21.1a
! style="transform: rotate(-90deg);max-width: 2em;" | 2.21
! style="transform: rotate(-90deg);max-width: 2em;" | 2.20.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.20
|-
! 5.5.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 5.4.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 5.3.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 5.2.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 5.1.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
|}

== GCC Version 4.x.x ==

<!-- Excuse the hackish styling, I will request an edit of MediaWiki:Common.css -->
{| {{wikitable}}
|- style="height: 6em;"
! &darr; [[GCC]] / [[Binutils]] &rarr;
! style="transform: rotate(-90deg);max-width: 2em;" | 2.25.1 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.25 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.24 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23.2
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23
! style="transform: rotate(-90deg);max-width: 2em;" | 2.22
! style="transform: rotate(-90deg);max-width: 2em;" | 2.21.1a
! style="transform: rotate(-90deg);max-width: 2em;" | 2.21
! style="transform: rotate(-90deg);max-width: 2em;" | 2.20.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.20
! style="transform: rotate(-90deg);max-width: 2em;" | 2.19.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.19
! style="transform: rotate(-90deg);max-width: 2em;" | 2.18
! style="transform: rotate(-90deg);max-width: 2em;" | 2.17
! style="transform: rotate(-90deg);max-width: 2em;" | 2.16.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.16
|-
! 4.9.4
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.9.3
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.9.2
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.9.1
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.8.5
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.8.2
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.8.1
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.8.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.7.4
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.7.2
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.7.1
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.7.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.6.3
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.6.2
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.6.1
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.6.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.5.2
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.5.1
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.5.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.4.3
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.4.2
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.4.1
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.4.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.3.3
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
|-
! 4.3.2
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.3.1
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.3.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.2.4
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.2.3
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.2.1
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.2.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.1.2
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.1.1
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
|-
! 4.0.3
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
|-
! 4.0.2
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
|-
! 4.0.1
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
|-
! 4.0.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
|-
|}

== See Also ==
=== Articles ===
*[[GCC Cross-Compiler]] - the tutorial that the configuration tables relate to.

[[Category:Compilers]]
````

<a id="w002806"></a>

## W002806 — User:Nedbrek/bootl

Bron-ID: W002806
Namespace: User (2)
Revisie: 9794 | Datum: 2010-02-15T13:17:28Z
Bron: https://wiki.osdev.org/User:Nedbrek/bootl
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANedbrek%2Fbootl&oldid=9794

````text
"The king said it was daft to build a castle in the swamp.  So I built it all the same!  Just to show him." - The Swamp King

==History==
===2010===
*Jan 2 - FAT12 style, booting into 64 bit mode in 512 B
*Jan 24 - Started investigation of El Torito
*Jan 27 - Booting from CDROM
*Jan 30 - Enable A20, query for system mem map
*Jan 31 - Added Boot Output Block (BOB), saved boot disk id to BOB
*Feb 14 - Query VBE modes, pick one, screen blank from kernel.bin

==Motivation==
Or "Why not Grub"

The purpose of Ned OS is for me to learn more about how computers work - rather than driving to a functional product or tool.  The primary use is inside a virtual machine. (Bochs is great!)

Also, to examine the assumptions that go into most operating systems and programming environments.

Using Grub would force me to accept all the assumptions that go into Grub (multi-stage boot loading, mult-boot kernel image, and ELF being the main ones).  That, or to rewrite Grub line by line (rewriting is how I learn best).

Once I am committed to ELF, I'd then feel compelled to write a linker (huge time sink).

Grub also encourages the use of "ordinary binaries" for the kernel.  That means GCC cross-compiler, which means C, which means all the UNIX assumptions (also, I'd feel compelled to write a compiler - double time sink).

==Technical Issues==
The main goal is to avoid multi-stage boot.  I wanted to see how much I could fit into 512 B, and how far I could get towards pure 64 bit mode.

Just getting into 64 bit mode is pretty easy, about 170 B.

The problem is, once in 64 bit mode, you've lost all the BIOS functions...

So, I need the boot loader to pull everything needed from the BIOS (no going back!)

This is where CD boot really shines.  You have the whole 512 B, and can easily get more.

Hard drive booting is harder (maybe unnecessary, as I would likely multi-boot a hard drive, which means Grub is ok).

I do have a stripped down version of the boot loader that fits into 446 B, but it is not what I would like.

==Results==
If Grub is like a cushy limo that delivers your kernel in style; Bootl is an orbital parachute that leaves you in a smoking crater - alive, but barely.

*The system is in 64 bit mode, with interrupts disabled.
*There is no interrupt table (you'll need to see about that right away!).
*You have one 2M page (identity mapped at 0..1F_FFFF).
*The video card is in some sort of graphics mode, it shoots for <=1024 wide and 32 bpp.  The exact details are in the BOB.  Since you can't get back to the BIOS, you're stuck with that mode forever.  Enjoy!
*Again, no BIOS, so no disk until you set up your own disk driver.
*You'll need to set up the task register.
*Caches are disabled
*The MTRRs are wherever the BIOS left them, possibly not in good shape.
````

<a id="w002807"></a>

## W002807 — User:AJ/GCC Cross-Compiler

Bron-ID: W002807
Namespace: User (2)
Revisie: 20691 | Datum: 2017-04-10T17:44:37Z
Bron: https://wiki.osdev.org/User:AJ/GCC_Cross-Compiler
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAJ%2FGCC_Cross-Compiler&oldid=20691
Doorverwijzing: Cross-Compiler Successful Builds

````text
#REDIRECT [[Cross-Compiler Successful Builds]]
````

<a id="w002808"></a>

## W002808 — User:AJ/Cross-Compiler Successful Builds

Bron-ID: W002808
Namespace: User (2)
Revisie: 9807 | Datum: 2010-02-16T12:20:42Z
Bron: https://wiki.osdev.org/User:AJ/Cross-Compiler_Successful_Builds
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAJ%2FCross-Compiler_Successful_Builds&oldid=9807
Doorverwijzing: Cross-Compiler Successful Builds

````text
#REDIRECT [[Cross-Compiler Successful Builds]]
````

<a id="w002809"></a>

## W002809 — User:Creature/Logarithms

Bron-ID: W002809
Namespace: User (2)
Revisie: 9831 | Datum: 2010-02-19T15:57:02Z
Bron: https://wiki.osdev.org/User:Creature/Logarithms
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACreature%2FLogarithms&oldid=9831

````text
After spending some time studying mathematics, I've found of a fairly easy way to calculate logarithms in your operating system. Note that there are a few downsides to this way as well, which will be described in a moment.

== Logarithms ==
=== Requirements ===
Firstly, there are a few requirements you need to meet in order to be able to use this method. You will need:

* Floating point support (see [[FPU]] for more details).
* Some implementation of the '''powf''' function (a function that can raise float x to float y). If y is the power to raise to, then you must be able to handle the case where 0 < y < 1.

This is basically all you need.

=== Downsides ===
The main downside of this method is precision: it is not very precise and the way I'm showing it to you may be improved (it is merely used for demonstration). The method will probably suffer from rounding errors.

=== The Method ===
For the calculation of logarithms, we're going to use a couple of other facts.

Firstly, the derivative of an exponential function f(x) = a^x (where ^ denotes '''to the power of''') is f'(x) = c * a^x. '''c''' is a constant number that depends on the growth factor (the growth factor is '''a'''). The number '''c''' can be calculated by

          &Delta;x
  lim    a   - 1
  &Delta;x->0 ---------
           &Delta;x

But the derivative of function f(x) = a^x also equals g'(x) = ln(a) * a^x, where ln is the logarithmus naturalis of '''a'''. What we have now are two derivatives of one same function, which means we can compare them.

  -> c * a^x = ln(a) * a^x             f'(x) = g(x)
  -> c = ln(a)                         Drop '''a^x''' (exists in both terms)
  -> c = log(a) = the_limit_above      ln(a) = log(a)

Note that '''log''' is the logarithmus naturalis in the standard C library (so the base number is Euler's mathematical constant, not 10).

From the definition of logarithms, we know that

  a         log10(x)
   log x = ----------
            log10(a)

So when calculating the '''ln''' (or '''log''' in the standard C library) of x, '''a''' becomes '''e'''. Because log10(e) will never change, we can calculate this number and put it in a define somewhere. By my (quick and dirty) calculations, log10(e) equals approximately '''0.4342944819'''. Now we can transform our formula:

                 a
  0.4342944819 *  log x = log10(x)

Now, if we want to implement our C function '''log''' (which uses e as base, so a = e), we use the fact that log(x) = c and the limit.

<source lang="c">
float log10(float x)
{
   return (((powf(x, 0.00001f) - 1) / 0.00001f) * LOG10_E);
}
</source>

Where '''LOG10_E''' is the number calculated earlier. We raise '''x''' to the power of '''0.00001f''' because '''&Delta;x''' approaches 0, so this is a fairly good choice (though inaccurate). Once you have log10, it's not hard to calculate the other logarithm functions:

<source lang="c">
float log(float x)
{
   return (log10(x) / LOG10_E);
}
</source>

We could even create a general function that calculates any type of logarithm:

<source lang="c">
// ----- Custom function that calculates the logarithm of value x with the given base.
loat logx(float x, float base)
{
   // base may not equal 1 or be negative.
   if(base == 1.f || base < 0.f)
      return 0.f; // Return what you want here, could be Not-A-Number or NAN.

   return (log10(x) / log10(base));
}
</source>

As I said, this method is inaccurate and probably could be improved to be more precise.

== Alternative implementation (courtesy of [[User:Combuster|Combuster]]) ==
There is an alternative way (but more complex in theory) that can be used to calculate the '''ln''' of a value '''x'''. It is much more precise (since it uses a Taylor series expansion). Wikipedia has a very nice article on this, which can be found [http://en.wikipedia.org/wiki/Natural_logarithm#Numerical_value here]. Once you have implemented this way, it is again not very hard to calculate other logarithmic values.
````

<a id="w002810"></a>

## W002810 — User talk:Creature/Logarithms

Bron-ID: W002810
Namespace: User talk (3)
Revisie: 9818 | Datum: 2010-02-17T11:15:08Z
Bron: https://wiki.osdev.org/User_talk:Creature/Logarithms
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ACreature%2FLogarithms&oldid=9818

````text
This is both mathematically exact and precise, and does not require a power() or similar function. It suffers from one problem: the second part is totally nonobvious for the non-math-geeks. Formula grabbed from [http://en.wikipedia.org#wiki/Natural_logarithm#Numerical_value Wikipedia]. Use as you please - [[User:Combuster|Combuster]] 22:15, 16 February 2010 (UTC)
<source lang="c">
#define EULER 2.718281828459045235360287L
double log(double x)
{
    if (!(x > 0)) return -INFINITY; // handles NaN and invalid inputs
    
    // compute integer part of answer
    double base = 0;
    //normalize to near-1 (0.6-1.7)
    while (x < 0.6)
    {
        x *= EULER;
        base--;
    }
    while (x > 1.7)
    {
        x /= EULER;
        base++;
    }
    
    // expand the taylor-maclaurin series for ln
    double y = (x-1.0)/(x+1.0);
    double fraction = 0.0;
    double bottomroot = 1.0;
    double power = 1.0;
    int lpc = 0;

    while (lpc < expansions)
    {
        fraction += power / bottomroot;
        power *= y * y;
        bottomroot += 2.0;
        lpc++; 
    }
    fraction *= 2.0 * y;

    return(base + fraction);
}
</source>

* Thanks for the suggestion (I moved your comment here so I could reply to it without crowding my own article and it seems kind of ungrateful to just remove the comment ;). I've put the link to Wikipedia and a note about the alternative way inside the article (I also gave you credit for the suggestion). In fact, I was studying maths yesterday, when I suddenly found this way. Since I'm only just a student, I don't know a great amount of maths (just mostly the 'basic' stuff). I've actually never studied the Taylor series before, but I have taken a look at them myself and have an idea what they are for and how they work. --[[User:Creature|Creature]] 11:15, 17 February 2010 (UTC)
````

<a id="w002811"></a>

## W002811 — COFF

Bron-ID: W002811
Namespace: Main (0)
Revisie: 30559 | Datum: 2026-05-05T18:06:26Z
Bron: https://wiki.osdev.org/COFF
Revisie-URL: https://wiki.osdev.org/index.php?title=COFF&oldid=30559

````text
{{File formats}}

'''COFF''' (Common Object File Format) is a file format used for storing compiled code, such as that outputted by a compiler or a linker.

Like most compiler file formats, COFF defines structures within the file for storing information about the sections of the program, such as .text and .data, and about the symbols that the program declares or defines.

COFF can be used to store individual functions or symbols, fragments of programs, libraries or entire executables.

The Microsoft [[PE]] Executable format (strictly PE/COFF) contains a version of COFF.

Whilst COFF is still around today, it is regarded as being overly complicated and in many cases its use has been replaced by [[ELF]] or other executable file formats.

==Microsoft PE/COFF==
{{Main|PE}}

Most Microsoft EXE and DLL files use a file format that is based on COFF.
Microsoft's Portable Executable (PE/COFF) standard is largely compatible with the COFF specification with a few notable exceptions.  

Firstly, the COFF File Header is not placed at the beginning of the file if the file is an image (executable) file.  In brief, a PE/COFF Image contains a pointer at offset 0x3C which points to the PE header.  The COFF File Header immediately follows this PE signature.  More information can be found in the [[PE]] article.

Secondly, the PE/COFF format also changes some of the COFF magic values (including the File Header one) and assigns them special meanings.

More information can be found in the [https://docs.microsoft.com/en-us/windows/win32/debug/pe-format "Microsoft Portable Executable and Common Object File Format Specification"] document.

==File Format==

COFF files contain a number of different tables, as well as areas of data that are referenced by the tables.

{| class="wikitable"
|-
! Structure
! Purpose
! Location
! Length
|-
| File Header
| Stores the basic information about the file as well as pointers to the other structures.
| At the beginning of the file, except in Microsoft PE/COFF Image files (see above).
| Fixed structure length.
|-
| Optional Header
| Stores additional information about the execution of the file.
| Only present if indicated by the File Header.  Immediately follows the File Header.
| Specified in the File Header
|-
| Section Header
| Stores information about the different sections defined in the file.
| Immediately following the Optional Header.  If no Optional Header is provided, immediately follows the File Header.
| Calculated by the number of sections (specified in the File Header) multiplied by the fixed structure length.
|-
| Section Relocation Table
| Allows the file to be relocated to any area of memory by providing information on which addresses need to be changed.
| At most one table exists for each Section.  Location is indicated by the corresponding Section Header.
| Calculated by the number of relocation entries (specified in the Section header) multiplied by the fixed structure length.
|-
| Section Line Number Table
| Provides debugging information to map code addresses to source file line numbers.
| At most one table exists for each Section.  Location is indicated by the corresponding Section Header.
| Calculated by the number of line number entries (specified in the Section header) multiplied by the fixed structure length. 
|-
| Symbol Table
| Stores information on each symbol defined or declared by the code.
| Indicated to a pointer in the File Header.
| Calculated by the number of symbols (specified in the File Header) multiplied by the fixed structure length.
|-
| String Table
| Stores any Section or Symbol names that are longer than eight characters.
| Immediately following the Symbol table.
| Indicated by the first 32 bits of the string table, evaluated as a long
|}

Most COFF files contain additional space between the end of the Section table and the start of the Symbol table which stores the data for the sections themselves, although the location of this data relative to the Symbol (and String) table does not seem to be specified by the standard.

===File Header===

The file header exists at the beginning of a COFF file (except in Microsoft PE/COFF Image files as noted above).  It contains a magic number field and other general information about the File.

The structure of this header can be found in filehdr.h, and is 20 bytes long.

<syntaxhighlight lang="c">
{
	unsigned short 	f_magic;	/* Magic number */	
	unsigned short 	f_nscns;	/* Number of Sections */
	long 		f_timdat;	/* Time & date stamp */
	long 		f_symptr;	/* File pointer to Symbol Table */
	long 		f_nsyms;	/* Number of Symbols */
	unsigned short 	f_opthdr;	/* sizeof(Optional Header) */
	unsigned short 	f_flags;	/* Flags */
}
</syntaxhighlight>

The magic number varies from implementation to implementation, for example, DJGPP generates COFF files with the value 0x14C in this field.

The number of sections indicates how many section structures should be expected in the Section table.

The time and date stamp indicate when the COFF file was created.  This field is of the type time_t, being the number of seconds since epoch ( 1970-01-01 00:00:00 GMT ).

The Symbol table pointer indicates the offset within the file where the Symbol table begins.

The number of symbols indicate how many (fixed length) symbol structures are in the symbol table.  This value coupled with the Symbol table pointer allow you to calculate where the String table starts.

The flags field usually indicates the state of the file.  Usually this is used to quickly lookup information that could be calculated from the rest of the file.  For example, one bit of this field indicates if relocation information has been removed from the file, another bit indicates if all the external symbols have been resolved, etc.

===Optional Header===

The Optional header provides run-time information about the file and therefore is only present in executable COFF files.

This header is only present in the file if the File header field f_opthdr is non-zero and, if present, immediately follows the File header.

The structure of this header can be found in aouthdr.h, and is 28 bytes long.

<syntaxhighlight lang="c">
{
	unsigned short 	magic;          /* Magic Number                    */
	unsigned short 	vstamp;         /* Version stamp                   */
	unsigned long 	tsize;          /* Text size in bytes              */
	unsigned long 	dsize;          /* Initialised data size           */
	unsigned long 	bsize;          /* Uninitialised data size         */
	unsigned long 	entry;          /* Entry point                     */
	unsigned long 	text_start;     /* Base of Text used for this file */
	unsigned long 	data_start;     /* Base of Data used for this file */
}
</syntaxhighlight>

===Section Header===

The Section table provides information about the individual section contained within the file.

The Section table is an array of Section header structures.  It immediately follows the Optional header (or File header if no Optional header exists) and its length in Section header structures is given by the f_nscns field in the File header.

Unlike most other tables, the Section table is 1-based, meaning that the first Section is referred to as Section 1 instead of Section 0.  This is done because in the Symbol table, a Section number of 0 has a special meaning.

The structure of this header can be found in scnhdr.h, and is 40 bytes long.

<syntaxhighlight lang="c">
{
	char		s_name[8];	/* Section Name */
	long		s_paddr;	/* Physical Address */
	long		s_vaddr;	/* Virtual Address */
	long		s_size;		/* Section Size in Bytes */
	long		s_scnptr;	/* File offset to the Section data */
	long		s_relptr;	/* File offset to the Relocation table for this Section */
	long		s_lnnoptr;	/* File offset to the Line Number table for this Section */
	unsigned short	s_nreloc;	/* Number of Relocation table entries */
	unsigned short	s_nlnno;	/* Number of Line Number table entries */
	long		s_flags;	/* Flags for this section */
}
</syntaxhighlight>

The section name field stores the name of this section padded with NULL bytes if it is eight characters or fewer; otherwise, this field contains either a pointer into the String Table, or a special name that starts with `/` and is followed by the decimal digits representing the offset within the String Table where the section name may be found.  See the entry on the String Table below.

The Physical address field stores the memory start address for this file.  If the file has been linked so that this section is to be loaded to a specific location in memory, this field will have been set to that value.

The Virtual address field is equivalent to the Physical address field for the Virtual memory space.  However, in almost all instances, these two fields contain the same value.

The s_size field contains the Section size in bytes.

The Section pointer field contains the file offset where the raw data for this section begins.  This, coupled with the Section size, can be used to load the section data.  If this is a BSS Section (indicated in the Section's Flags field), the section may well have a size, but a Section pointer of zero.

The s_relptr indicates the file offset for the Relocation information for this Section.  The number of entries in the Relocation table is given in the s_nreloc field.  If the s_nreloc field is zero, the section does not contain any Relocation information.

Like the Relocation pointer, the s_lnnoptr field indicates the file offset to the Line number table for the section, whose length in entries is given in the s_nlnno field.  As with the s_nreloc, if the s_nlnno is zero then the Section contains no Line number information.

The Flags field gives important information on how the Section should be handled when being processed.  The most important flag values relate to the type of field as follows:
{| class="wikitable"
|-
! Value
! Define
! Meaning if set
|-
| 0x0020
| STYP_TEXT
| The section contains executable code.
|-
| 0x0040
| STYP_DATA
| The section contains initialised data.
|-
| 0x0080
| STYP_BSS
| The COFF file contains no data for this section, but needs space to be allocated for it.
|}

===Relocation Entries===

Using Relocation entries, it is possible to load a section of code into an arbitrary memory address and then modify the code so that the memory addresses for Symbols are correct.  This means you can take a COFF file, load the .text section of the file into one place, the .data section into another, the .bss section somewhere else again, and then update the .text section so that the code is able to find its values and function correctly.

The Relocation Table is an array of Relocation Entry structures and is zero based.

A COFF file can contain a Relocation table for each Section.  .text ( STYP_TEXT ) sections often have relocation information, it is also possible for .data ( STYP_DATA ) and other sections to have relocation information.

The Relocation table for a section has its location and length stored in the Section's header.

The structure of this header can be found in reloc.h, and is 10 bytes long.

<syntaxhighlight lang="c">
{
	long		r_vaddr;	/* Reference Address */
	long		r_symndx;	/* Symbol index */
	unsigned short	r_type;		/* Type of relocation */
}
</syntaxhighlight>

The Address field gives the offset within the Section's raw data where the address starts.  The reference will almost always be a 32-bit value starting at the given address, but this is dependant on implementation and the r_type field.

The Symbol index gives the (zero based) index in the Symbol table to which the reference refers.  Once you have loaded the COFF file into memory and know where each symbol is, you find the new updated address for the given symbol and update the reference accordingly.

The r_type indicates how the address should be updated.  It is, unfortunately, implementation specific.  For an example, DJGPP can indicate that this be the 32-bit absolute address of the symbol (type 6), or it can specify a 32-bit address that is relative to the location of the reference (type 20).

===Line Number Entries===

Line number entries are used by debugging programs to relate symbols and physical instruction addresses to source file line numbers.

The Line Number table is an array of Line Number Entry structures and is zero based.

A COFF file can contain a Line number table for each Section, but it would be unusual to find line number information for anything other than .text ( STYP_TEXT ) Sections.

Like the Relocation table, the Line Number table for a section has its location and length stored in the Section's header.

The structure of this header can be found in linenum.h, and is 6 bytes long.

<syntaxhighlight lang="c">
{
	union
	{
		long		l_symndx;	/* Symbol Index */
		long		l_paddr;	/* Physical Address */
	} l_addr;
	unsigned short		l_lnno;		/* Line Number */
}
</syntaxhighlight>

In brief, The line number table will contain an entry with a line number of zero in which the l_symndx field will indicate the function name Symbol in the Symbol table.  This entry will be followed by additional entries with incrementing line numbers which indicate, through the l_paddr field, the byte offset into the section where this line starts.  Given this, an exception that occurs during processing of the COFF file can be traced back to a function and a line number within that function.

===Symbol Table===

The Symbol Table contains information on each symbol that is declared or defined by this file.  There will be symbols in this table for each function name, for each global variable, for each static variable, for each variable defined as extern, etc.  There are even symbols in this table that indicate the start of each section and further information about the file as a whole.

The Symbol table is an array of Symbol Entry structures and is zero based.

The Symbol table location is given in the File Header f_symptr field, and its length in entries is given in the File Header f_nsyms field.

The structure of this header can be found in syms.h, and is 18 bytes long.

<syntaxhighlight lang="c">
{
	char		n_name[8];	/* Symbol Name */
	long		n_value;	/* Value of Symbol */
	short		n_scnum;	/* Section Number */
	unsigned short	n_type;		/* Symbol Type */
	char		n_sclass;	/* Storage Class */
	char		n_numaux;	/* Auxiliary Count */
}
</syntaxhighlight>

The Symbol name field stores the name of this symbol if it is eight characters or fewer, otherwise this field contains a pointer into the String table.  See the entry on the String table below.

Each symbol in the Symbol table can be followed by zero or more Auxiliary entries which provide further information about the symbol.  Each of these entries is the same length as the Symbol table entry, but has a different structure depending on the parent Symbol.

The Section number field n_scnum indicates the Section to which this Symbol belongs.  If this value is greater than zero, it refers to a Section in the Section table (hence why the Section table is one-based).  I can also take one of the following special values:

{| class="wikitable"
|-
! n_scnum Value
! Meaning
|-
| N_DEBUG (2)
| A debugging symbol.  In the example below, information about the file has been put into a symbol like this.
|-
| N_ABS (1)
| An absolute symbol.  This means that the value of the n_value field is an absolute value.
|-
| N_UNDEF (0)
| An undefined external symbol
|}

The n_value, n_scnum and n_sclass fields in the table need to be considered together because they are interlinked.  A number of the more common combinations of these fields is given in the following table.  ''(Much of the information in the following table is from observation and not from referenced sources.  It is incomplete and may not be accurate.)''

{| {{wikitable}}
|-
! n_sclass
! n_scnum
! n_value
! Meaning of n_value
! Typical use
|-
| rowspan="4"| C_EXT (2)
| 0
| 0
| 
| Unresolved external Symbol
|-
| 0
| >0
| Size of variable
| Uninitialised global variable (not included in BSS)
|-
| .text
| Any
| Offset into Section
| Function entry point
|-
| .data
| Any
| Offset into Section
| Initialised global variable
|-
| rowspan="3"| C_STAT (3)
| .text, .data, .bss
| 0
| 
| Section Symbol indicating start of Section
|-
| .data
| Any
| Offset into Section
| Initialised static variable
|-
| .bss
| Any
| Offset into Section
| Unitialised static variable
|}


As noted in the table, uninitialised global variables do not seem to be included in the size of the BSS section, only unitialised static variables.  If you are loading a COFF file into memory, it may be required to allocate space in addition to the BSS section size to cover these symbols.

{{stub}}

===String Table===

The String Table contains string names for any sections or symbols that are longer than eight characters.

The String Table immediately follows the Symbol Table, and can be located by using the following formula:
<pre>
String Table Offset = File Header.f_symptr + File Header.f_nsyms * sizeof( Symbol Table Entry )
	            = File Header.f_symptr + File Header.f_nsyms * 18
</pre>

The first four bytes of the String Table are an integer indicating the overall size in bytes of the string table.  Each of the strings in the table are null terminated.

The Section Table name field and the Symbol Table name field are actually more complicated than was detailed above, they in fact look more like this:
<syntaxhighlight lang="c">
{
	union
	{
		char	name[8];
		struct
		{
			unsigned long zeroes;
			unsigned long offset;
		}
	}
}
</syntaxhighlight>

If the name is eight characters or fewer, then the field "zeroes" will be non-zero, and "name" should be interpreted as a character array.  Note that this field is not null-terminated unless it is fewer than eight characters in length.

If the name is more than eight characters, the "zeroes" field (the first four bytes of "name") will be zero.  In this case the "offset" field should be used as an offset value into the String table to locate the Symbol or Section name.

NOTE: Some tools (like GCC) use a different format for the section name field.  Instead of the first four bytes being zero and the last four being an offset into the string table (as described just above), the first byte is a `/`, and the rest are the decimal digits of the offset within the string table where the section name may be found. For example, a section named ".name_thats_too_long" will have "/4" stored in its section name field, assuming this is the first string stored in the string table.

==Example==
{{stub}}

This is an example of a Symbol table and the source code to help illustrate some of the information above.  (This was compiled with DJGPP.)

<pre>
Symbols
      e_zeroes     e_offset     e_value      e_scnum       e_sclass 
                                                    e_type     e_numaux
[00]  2E 66 69 6C  65 00 00 00  00 00 00 00  FE FF  00 00  67  01   .file.......þÿ..g.    
[01]  74 65 73 74  2E 6C 69 62  72 61 72 79  2E 63  00 00  00  00   test.library.c....    
[02]  00 00 00 00  04 00 00 00  08 00 00 00  02 00  00 00  03  00   ..................  _mStaticIntInit
[03]  00 00 00 00  14 00 00 00  0C 00 00 00  02 00  00 00  03  00   ..................  _mStaticShortInit
[04]  00 00 00 00  26 00 00 00  0E 00 00 00  02 00  00 00  03  00   ....&.............  _mStaticCharInit
[05]  00 00 00 00  37 00 00 00  00 00 00 00  03 00  00 00  03  00   ....7.............  _mStaticInt
[06]  00 00 00 00  43 00 00 00  10 00 00 00  03 00  00 00  03  00   ....C.............  _mStaticShort
[07]  00 00 00 00  51 00 00 00  20 00 00 00  03 00  00 00  03  00   ....Q... .........  _mStaticChar
[08]  2E 74 65 78  74 00 00 00  00 00 00 00  01 00  00 00  03  01   .text.............  
[09]  33 00 00 00  03 00 00 00  00 00 00 00  00 00  00 00  00  00   3.................  
[0A]  2E 64 61 74  61 00 00 00  00 00 00 00  02 00  00 00  03  01   .data.............  
[0B]  0F 00 00 00  00 00 00 00  00 00 00 00  00 00  00 00  00  00   ..................  
[0C]  2E 62 73 73  00 00 00 00  00 00 00 00  03 00  00 00  03  01   .bss..............  
[0D]  30 00 00 00  00 00 00 00  00 00 00 00  00 00  00 00  00  00   0.................  
[0E]  2E 63 6F 6D  6D 65 6E 74  00 00 00 00  04 00  00 00  03  01   .comment..........  
[0F]  11 00 00 00  00 00 00 00  00 00 00 00  00 00  00 00  00  00   ..................  
[10]  00 00 00 00  5E 00 00 00  00 00 00 00  02 00  00 00  02  00   ....^.............  _mGlobalIntInit
[11]  00 00 00 00  6E 00 00 00  04 00 00 00  02 00  00 00  02  00   ....n.............  _mGlobalShortInit
[12]  00 00 00 00  80 00 00 00  06 00 00 00  02 00  00 00  02  00   ....€.............  _mGlobalCharInit
[13]  00 00 00 00  91 00 00 00  00 00 00 00  01 00  00 00  02  00   ....‘.............  _TestCall
[14]  5F 54 65 73  74 41 64 64  0A 00 00 00  01 00  00 00  02  00   _TestAdd..........  
[15]  00 00 00 00  9B 00 00 00  17 00 00 00  01 00  00 00  02  00   ....›.............  _TestSetValue
[16]  00 00 00 00  A9 00 00 00  10 00 00 00  00 00  00 00  02  00   ....©.............  _mTestValue
[17]  00 00 00 00  B5 00 00 00  29 00 00 00  01 00  00 00  02  00   ....µ...).........  _TestGetValue
[18]  00 00 00 00  C3 00 00 00  10 00 00 00  00 00  00 00  02  00   ....Ã.............  _mGlobalInt
[19]  00 00 00 00  CF 00 00 00  10 00 00 00  00 00  00 00  02  00   ....Ï.............  _mGlobalShort
[1A]  00 00 00 00  DD 00 00 00  10 00 00 00  00 00  00 00  02  00   ....Ý.............  _mGlobalChar
</pre>

<syntaxhighlight lang="c">
int mGlobalInt;
short mGlobalShort;
char mGlobalChar;

static int mStaticInt;
static short mStaticShort;
static char mStaticChar;

unsigned int mGlobalIntInit = 1;
unsigned short mGlobalShortInit = 1;
unsigned char mGlobalCharInit = 1;

unsigned static int mStaticIntInit = 1;
unsigned static short mStaticShortInit = 1;
unsigned static char mStaticCharInit = 1;

int mTestValue;

int TestCall()
{
	return 1;
}

int TestAdd( int pA, int pB )
{
	return pA + pB;
}

int TestSetValue( int pValue )
{
	mTestValue = pValue;
	return mTestValue;
}

int TestGetValue()
{
	return mTestValue;
}
</syntaxhighlight>

As you can see, the static global variables are defined in either section 2 (.data) for the initialised ones, or section 3 (.bss) for the uninitialised, with n_value being an offset into the relevant section.  The initialised global variables are also defined in section 2 (.data) with offsets.

Note that the uninitialised global variables have a Section number of N_UNDEF (0) and as such are undefined, but as we a non-zero n_value field, we are being told the amount of space that we need to allocate for each symbol (16 bytes for each one in a slightly wasteful overallocation).

==See Also==

=== External Links ===
* http://www.delorie.com/djgpp/doc/coff/
* [https://web.archive.org/web/20230703150217/https%3A%2F%2Fdownload.microsoft.com%2Fdownload%2F9%2Fc%2F5%2F9c5b2167-8017-4bae-9fde-d599bac8184a%2Fpecoff_v83.docx Official Microsoft COFF Specification (2016)]

[[Category:Executable Formats]]
````

<a id="w002812"></a>

## W002812 — FADT

Bron-ID: W002812
Namespace: Main (0)
Revisie: 30278 | Datum: 2026-03-16T17:57:33Z
Bron: https://wiki.osdev.org/FADT
Revisie-URL: https://wiki.osdev.org/index.php?title=FADT&oldid=30278

````text
{{ACPI}}
''This page is about the ACPI FADT(Fixed ACPI Description Table)''
It contains information about ACPI fixed registers, DSDT pointer, etc.

'''FADT''' is a data structure used in the [[ACPI]] programming interface. This table contains information about fixed register blocks pertaining to power management.

==Finding the FADT==
The FADT is pointed to by an entry in the [[RSDT]].  The signature is 'FACP'. Even if the pointer was found in another ACPI valid structure, you should anyway do the checksum to check that the table is valid.

==Structure==
The FADT is a complex structure and contains a lot of data. Here it is:
<syntaxhighlight lang="c">struct FADT
{
    struct   ACPISDTHeader h;
    uint32_t FirmwareCtrl;
    uint32_t Dsdt;

    // field used in ACPI 1.0; no longer in use, for compatibility only
    uint8_t  Reserved;

    uint8_t  PreferredPowerManagementProfile;
    uint16_t SCI_Interrupt;
    uint32_t SMI_CommandPort;
    uint8_t  AcpiEnable;
    uint8_t  AcpiDisable;
    uint8_t  S4BIOS_REQ;
    uint8_t  PSTATE_Control;
    uint32_t PM1aEventBlock;
    uint32_t PM1bEventBlock;
    uint32_t PM1aControlBlock;
    uint32_t PM1bControlBlock;
    uint32_t PM2ControlBlock;
    uint32_t PMTimerBlock;
    uint32_t GPE0Block;
    uint32_t GPE1Block;
    uint8_t  PM1EventLength;
    uint8_t  PM1ControlLength;
    uint8_t  PM2ControlLength;
    uint8_t  PMTimerLength;
    uint8_t  GPE0Length;
    uint8_t  GPE1Length;
    uint8_t  GPE1Base;
    uint8_t  CStateControl;
    uint16_t WorstC2Latency;
    uint16_t WorstC3Latency;
    uint16_t FlushSize;
    uint16_t FlushStride;
    uint8_t  DutyOffset;
    uint8_t  DutyWidth;
    uint8_t  DayAlarm;
    uint8_t  MonthAlarm;
    uint8_t  Century;

    // reserved in ACPI 1.0; used since ACPI 2.0+
    uint16_t BootArchitectureFlags;

    uint8_t  Reserved2;
    uint32_t Flags;

    // 12 byte structure; see below for details
    GenericAddressStructure ResetReg;

    uint8_t  ResetValue;
    uint8_t  Reserved3[3];
  
    // 64bit pointers - Available on ACPI 2.0+
    uint64_t                X_FirmwareControl;
    uint64_t                X_Dsdt;

    GenericAddressStructure X_PM1aEventBlock;
    GenericAddressStructure X_PM1bEventBlock;
    GenericAddressStructure X_PM1aControlBlock;
    GenericAddressStructure X_PM1bControlBlock;
    GenericAddressStructure X_PM2ControlBlock;
    GenericAddressStructure X_PMTimerBlock;
    GenericAddressStructure X_GPE0Block;
    GenericAddressStructure X_GPE1Block;
};</syntaxhighlight>

== GenericAddressStructure ==
Before going to far, keep in mind that the GAS is a structure used by ACPI to describe the position of registers. Its structure is:
<syntaxhighlight lang="c">struct GenericAddressStructure
{
  uint8_t AddressSpace;
  uint8_t BitWidth;
  uint8_t BitOffset;
  uint8_t AccessSize;
  uint64_t Address;
};</syntaxhighlight>
{| border="1"
|+ Possible values of AddressSpace
|-
! scope="col" | Value
! scope="col" | Address Space
|-
| 0 || System Memory
|-
| 1 || System I/O
|-
| 2 || [[PCI]] Configuration Space
|-
| 3 || Embedded Controller
|-
| 4 || [[SMBus|System Management Bus]]
|-
| 5 || System [[CMOS]]
|-
| 6 || PCI Device [https://wiki.osdev.org/PCI#PCI_Device_Structure BAR] Target
|-
| 7 || [[IPMI|Intelligent Platform Management Infrastructure]]
|-
| 8 || [[General Purpose I/O]]
|-
| 9 || [[Generic Serial Bus]]
|-
| 0x0A || [[PCC|Platform Communication Channel]]
|-
| 0x0B to 0x7F || Reserved
|-
| 0x80 to 0xFF || OEM Defined
|}

'''BitWidth''' and '''BitOffset''' are required only when accessing a bit field, and I guess you know how to use them.

'''AccessSize''' defines how many bytes at once you can read/write.
{| border="1"
|+ Possible values of AccessSize
|-
! scope="col" | Value
! scope="col" | Access size
|-
| 0 || Undefined (legacy reasons)
|-
| 1 || Byte access
|-
| 2 || 16-bit (word) access
|-
| 3 || 32-bit (dword) access
|-
| 4 || 64-bit (qword) access
|}

Address is a 64-bit pointer in the defined address space to the data structure.

== Fields ==
=== FirmwareCtrl ===
This is a 32-bit pointer to the [[FACS]]. Since ACPI 2.0 a new field has been added to the table, X_FirmwareControl, which is 64-bits wide. Only one of the two fields is used, the other contains 0. According to the Specs, the X_ field is used only when the FACS is placed above the 4th GB.

=== Dsdt ===
A 32-bit pointer to the [[DSDT]]. This has a X_Dsdt brother too.
=== PreferredPowerManagementProfile ===
This field contains a value which should address you to a power management profile. For example if it contains 2, the computer is a laptop and you should configure power management in power saving mode.
{| border="1"
|+ Values for PreferredPowerManagementProfile
|-
! scope="col" | Value
! scope="col" | Meaning
|-
| 0 || Unspecified
|-
| 1 || Desktop
|-
| 2 || Mobile
|-
| 3 || Workstation
|-
| 4 || Enterprise Server
|-
| 5 || SOHO Server
|-
| 6 || Aplliance PC
|-
| 7 || Performance Server
|-
| >7 || Reserved
|}
=== SCI_Interrupt ===
The System Control Interrupt is used by ACPI to notify the OS about fixed events, such as for example, pressing the power button, or for General Purpose Events (GPEs), which are firmware specific. This member in the FADT structure indicates the [[PIC]] or [[IOAPIC]] interrupt pin for it. To know if it's a PIC IRQ, check if the dual 8259 interrupt controllers are present via the [[MADT]]. Otherwise, it's a GSI. If you are using the IOAPIC and the PIC is present, remember to check the Interrupt Source Overrides first to get the GSI associated with the IRQ source.

=== SMI_CommandPort ===
This is an I/O Port. This is where the OS writes AcpiEnable or AcpiDisable to get or release the ownership over the ACPI registers. This is 0 on systems where the [[System Management Mode]] is not supported.
== What's next? ==
You should preserve the pointer to the [[FACS]] (in FirmwareControl (if < 4GB) or in X_FirmwareControl (if >= 4GB)). Then you should parse the [[DSDT]].

[[Category:ACPI]]
````

<a id="w002813"></a>

## W002813 — User:Stephanvanschaik

Bron-ID: W002813
Namespace: User (2)
Revisie: 17705 | Datum: 2015-03-03T14:17:45Z
Bron: https://wiki.osdev.org/User:Stephanvanschaik
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AStephanvanschaik&oldid=17705

````text
== Introduction ==

Hi,

I am Stephan J.R. van Schaik and I live in the Netherlands, generally in Sittard and in Maastricht (I live at several places). I was born on August 28th, 1993. I'm fluent in a lot of programming languages like C, C++, D, BASIC (various subsets like Visual Basic, AmigaBASIC and such), Assembly (x86, x86-64, ARM, PPC, m68K), Java, etc., but also some scripting/mark-up languages, generally those used for websites. I prefer to use low-level programming languages over high-level languages though so I generally program in Assembly and C. My Myers-Briggs personality type is ISTP.

== Contributions ==

*[[UEFI|(Unified) Extensible Firmware Interface]]
*[[Setting Up Long Mode]]
````

<a id="w002814"></a>

## W002814 — Setting Up Long Mode

Bron-ID: W002814
Namespace: Main (0)
Revisie: 30260 | Datum: 2026-03-16T16:06:39Z
Bron: https://wiki.osdev.org/Setting_Up_Long_Mode
Revisie-URL: https://wiki.osdev.org/index.php?title=Setting_Up_Long_Mode&oldid=30260

````text
== Recommended Reads== 

*[[CPUID]]
*[[Paging#64-Bit Paging|Paging]]
*[[Setting Up Paging]]
*[[Setting Up Paging With PAE|Setting up Paging With PAE]]
*[[Global Descriptor Table|GDT]]

== Overview==
*Covering long mode.
*How to detect long mode
*How to set up paging for long mode
*How to enter long mode.
*How to set up the GDT for long mode

==Introduction ==
Since the introduction of the x86-64 processors (AMD64, EM64T, VIA Nano), a new mode has been introduced as well, which is called long mode. Long mode consists of two sub modes which are: the actual 64-bit mode, and compatibility mode (a 32-bit mode, usually referred to as IA32e in the AMD64 manuals). The 64-bit mode is the most useful as it provides a lot of new features such as registers being extended to 64-bit (rax, rcx, rdx, rbx, rsp, rbp, rip, etc.), eight new general-purpose registers (r8 - r15), and eight new multimedia registers (xmm8 - xmm15). 64-bit mode is almost completely void of the segmentation that was used on the 8086-processors, and the [[GDT]], the [[IDT]], [[paging]], etc. are also different compared to the old 32-bit mode (a.k.a. [[Protected Mode|protected mode]]).

==Detecting the Presence of Long Mode==

There are only three processor vendors so far who have made processors that are capable of entering and using long mode: AMD, Intel and VIA. Intel tried to get 64-bit processors on the market with EM64T, but now uses AMD's x86-64 architecture instead, which means using 64-bit on an Intel processor is (almost) identical to using 64-bit on an AMD or VIA processor. The presence of long mode can be detected using the [[CPUID]] instruction.

===Detection of CPUID===

Main article: [[CPUID]]

<syntaxhighlight lang="nasm">
; it may be preferrable to put this in a separate file to be included,
; along with any other EFLAGS bits you may want to use
EFLAGS_ID equ 1 << 21           ; if this bit can be flipped, the CPUID
                                ; instruction is available

; Checks if CPUID is supported by attempting to flip the ID bit (bit 21) in
; the EFLAGS register. If we can flip it, CPUID is available.
; returns eax = 1 if there is cpuid support; 0 otherwise
checkCPUID:
    pushfd
    pop eax

    ; The original value should be saved for comparison and restoration later
    mov ecx, eax
    xor eax, EFLAGS_ID

    ; storing the eflags and then retrieving it again will show whether or not
    ; the bit could successfully be flipped
    push eax                    ; save to eflags
    popfd
    pushfd                      ; restore from eflags
    pop eax

    ; Restore EFLAGS to its original value
    push ecx
    popfd

    ; if the bit in eax was successfully flipped (eax != ecx), CPUID is supported.
    xor eax, ecx
    jnz .supported
    .notSupported:
        mov ax, 0
        ret
    .supported:
        mov ax, 1
        ret
</syntaxhighlight>

=== Checking for long mode support===
The presence of long mode can only be detected using the extended functions of CPUID, which aren't supported on every processor. Because of this, it's necessary to check if the extended function that checks long mode support is available:
<syntaxhighlight lang="nasm">
CPUID_EXTENSIONS equ 0x80000000 ; returns the maximum extended requests for cpuid
CPUID_EXT_FEATURES equ 0x80000001 ; returns flags containing long mode support among other things

; other setup code
; ...

.queryLongMode:
    mov eax, CPUID_EXTENSIONS
    cpuid
    cmp eax, CPUID_FEATURES
    jb .NoLongMode              ; if the CPU can't report long mode support, then it likely
                                ; doesn't have it
</syntaxhighlight>

Now the extended function can be used to check for long mode support:

<syntaxhighlight lang="nasm">
CPUID_EDX_EXT_FEAT_LM equ 1 << 29   ; if this is set, the CPU supports long mode

; ...

    mov eax, CPUID_EXT_FEATURES
    cpuid
    test edx, CPUID_EDX_EXT_FEAT_LM
    jz .NoLongMode
</syntaxhighlight>

==Entering Long Mode==

Entering long mode can be done from both real mode and protected mode, however only protected mode is covered in the Intel and AMD64 manuals; early AMD documentation explains how this process works from real mode as well.

Before anything, it is highly recommended to enable the [[A20 Line]], otherwise only odd MiBs can be accessed.

===Setting up paging===
Before 64-bit paging can be set up, 32-bit paging needs to be disabled (this can be skipped if paging was never set up in the first place).
<syntaxhighlight lang="nasm">
CR0_PAGING equ 1 << 31

; disables 32 bit paging
disablePaging32:
    mov eax, cr0
    and eax, ~CR0_PAGING
    mov cr0, eax
    ret
</syntaxhighlight>

64-bit paging uses PAE paging with an extra level, which includes (in order of distance from the root): 

*A Page Map Level 4 Table (PML4T), which replaces the PDPT as the root of PAE paging
*A Page Directory Pointer Table (PDPT)
*A Page Directory Table (PDT)
* A Page Table (PT)

Each entry in a 64 bit page table is double the size of the equivalent 32 bit entry, which means that each table can only hold 512 entries instead of the previous 1024. One entry in a PT can address 4KiB, and each table is 4096 bytes long, giving a maximum address space of 256TiB. Each level of page tables contains pointers to the one below (PML4T->PDPT->PDT->PT->Physical Memory), so each level refers to a more granular range of virtual addresses that the MMU uses to map each page to a physical address.

Virtual memory can be set up in many different ways, each with varying amounts of complexity, but one simple way to get started is to map the first 2 megabytes and then more memory can be allocated from within long mode later. This example code uses address 0x1000 as the beginning of the PML4T and each level below is in contiguous memory one after the other (PML4T = 0x1000, PDPT = 0x2000, etc...).

First step is to clear the tables:
<syntaxhighlight lang="nasm">
PML4T_ADDR equ 0x1000
SIZEOF_PAGE_TABLE equ 4096

    mov edi, PML4T_ADDR
    mov cr3, edi       ; cr3 lets the CPU know where the page tables are

    xor eax, eax
    mov ecx, SIZEOF_PAGE_TABLE
    rep stosd          ; writes 4 * SIZEOF_PAGE_TABLE bytes, which is enough space
                       ; for the 4 page tables
    mov edi, cr3       ; reset di back to the beginning of the page table
</syntaxhighlight>

Next is to link up just the first entries of each table, since 2 megabytes doesn't use more than one PDT entry
<syntaxhighlight lang="nasm">
PML4T_ADDR equ 0x1000
PDPT_ADDR equ 0x2000
PDT_ADDR equ 0x3000
PT_ADDR equ 0x4000

; the page table only uses certain parts of the actual address
PT_ADDR_MASK equ 0xffffffffff000
PT_PRESENT equ 1                 ; marks the entry as in use
PT_READABLE equ 2                ; marks the entry as r/w

    ; edi was previously set to PML4T_ADDR
    mov DWORD [edi], PDPT_ADDR & PT_ADDR_MASK | PT_PRESENT | PT_READABLE

    mov edi, PDPT_ADDR
    mov DWORD [edi], PDT_ADDR & PT_ADDR_MASK | PT_PRESENT | PT_READABLE

    mov edi, PDT_ADDR
    mov DWORD [edi], PT_ADDR & PT_ADDR_MASK | PT_PRESENT | PT_READABLE
</syntaxhighlight>

Now all that's left to do is fill the page table:

<syntaxhighlight lang="nasm">
ENTRIES_PER_PT equ 512
SIZEOF_PT_ENTRY equ 8
PAGE_SIZE equ 0x1000

    mov edi, PT_ADDR
    mov ebx, PT_PRESENT | PT_READABLE
    mov ecx, ENTRIES_PER_PT      ; 1 full page table addresses 2MiB

.SetEntry:
    mov DWORD [edi], ebx
    add ebx, PAGE_SIZE
    add edi, SIZEOF_PT_ENTRY
    loop .SetEntry               ; Set the next entry.
</syntaxhighlight>

Now PAE can be enabled using the cr4 register
<syntaxhighlight lang="nasm">
CR4_PAE_ENABLE equ 1 << 5

    mov eax, cr4
    or eax, CR4_PAE_ENABLE
    mov cr4, eax
</syntaxhighlight>

Now paging is set up, but it isn't enabled yet.

===Future of x86-64 - the PML5===
In November 2016, Intel released a white paper [https://cdrdv2-public.intel.com/671442/5-level-paging-white-paper.pdf] about 5-level paging, and started supporting it with Ice Lake processors in 2019. These processors support a 128 PiB address space, and furthermore up to 4 PiB of physical memory (far above the 4-level 256 TiB/64 TiB limits). Support for this is detected as follows:
<syntaxhighlight lang="nasm">
CPUID_GET_FEATURES equ 7
CPUID_FEATURE_PML5 equ 1 << 16

    mov eax, CPUID_GET_FEATURES
    xor ecx, ecx
    cpuid
    test ecx, CPUID_FEATURE_PML5
    jnz .5_level_paging
</syntaxhighlight>
The paging structures are identical to the 4 level versions, there's just an added layer of indirection. recursive addresses will change. 5 level paging is enabled if CR4.LA57=1, and EFER.LMA=1.
<syntaxhighlight lang="nasm">
CR4_LA57 equ 1 << 12

BITS 32
mov eax, cr4
or eax, CR4_LA57
mov cr4, eax
</syntaxhighlight>
Note that attempting to set CR4.LA57 while EFER.LMA=1 causes a #GP general protection fault. You therefore need to drop into protected mode or set up 5 level paging before entering long mode in the first place.

===The Switch to compatibility mode ===

First set the LM-bit:
<syntaxhighlight lang="nasm">
EFER_MSR equ 0xC0000080
EFER_LM_ENABLE equ 1 << 8

    mov ecx, EFER_MSR
    rdmsr
    or eax, EFER_LM_ENABLE
    wrmsr
</syntaxhighlight>

Enable paging and protected mode (if it is not set already):
<syntaxhighlight lang="nasm">
CR0_PM_ENABLE equ 1 << 0
CR0_PG_ENABLE equ 1 << 31

    mov eax, cr0
    or eax, CR0_PG_ENABLE | CR0_PM_ENABLE   ; ensuring that PM is set will allow for jumping
                                            ; from real mode to compatibility mode directly
    mov cr0, eax
</syntaxhighlight>

And compatibility mode is now enabled.

== Entering the 64-bit Submode==

To switch from compatibility mode to long mode, a [[GDT]] is used with fields set for 64 bit mode

The GDT (see chapter 4.8.1 and 4.8.2 of the AMD64 Architecture Programmer's Manual Volume 2) should look like this:
<syntaxhighlight lang="nasm">
; Access bits
PRESENT        equ 1 << 7
NOT_SYS        equ 1 << 4
EXEC           equ 1 << 3
DC             equ 1 << 2
RW             equ 1 << 1
ACCESSED       equ 1 << 0

; Flags bits
GRAN_4K       equ 1 << 7
SZ_32         equ 1 << 6
LONG_MODE     equ 1 << 5

GDT:
    .Null: equ $ - GDT
        dq 0
    .Code: equ $ - GDT
        .Code.limit_lo: dw 0xffff
        .Code.base_lo: dw 0
        .Code.base_mid: db 0
        .Code.access: db PRESENT | NOT_SYS | EXEC | RW
        .Code.flags: db GRAN_4K | LONG_MODE | 0xF   ; Flags & Limit (high, bits 16-19)
        .Code.base_hi: db 0
    .Data: equ $ - GDT
        .Data.limit_lo: dw 0xffff
        .Data.base_lo: dw 0
        .Data.base_mid: db 0
        .Data.access: db PRESENT | NOT_SYS | RW
        .Data.Flags: db GRAN_4K | SZ_32 | 0xF       ; Flags & Limit (high, bits 16-19)
        .Data.base_hi: db 0
    .Pointer:
        dw $ - GDT - 1
        dq GDT
</syntaxhighlight>

A 4GiB limit for code is needed because the processor will make a last limit check before the jump, and having a limit of 0 will cause a #GP (tested in [[Bochs]]). After that, the limit will be ignored. Now the only thing left to do is load it and make the jump to 64-bit:
<syntaxhighlight lang="nasm">
    lgdt [GDT.Pointer]
    jmp GDT.Code:Realm64
</syntaxhighlight>

===Sample===

The next step would probably be to load kernel code written in C, or some simpler code to test everything out. Clearing the screen would look like:

<syntaxhighlight lang="nasm">
bits 64

VGA_TEXT_BUFFER_ADDR equ 0xb8000
COLS equ 80
ROWS equ 25
BYTES_PER_CHARACTER equ 2
VGA_TEXT_BUFFER_SIZE equ BYTES_PER_CHARACTER * COLS * ROWS

Realm64:
    cli                           ; the code would probably be more reliable if you did this
                                  ; before even switching from real mode
    mov ax, GDT.Data
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    mov rdi, VGA_TEXT_BUFFER_ADDR
    mov rax, 0
    mov rcx, VGA_TEXT_BUFFER_SIZE / 8
    rep stosq
    hlt
</syntaxhighlight>

== See Also ==
===Articles===
*[[EM64T|Intel EM64T]]
*[[X86-64]]
*[[Creating a 64-bit kernel]]
*[[Entering Long Mode Directly]]

===Threads===
*[http://forum.osdev.org/viewtopic.php?f=8&t=21601 Wrote a tutorial covering long mode] about this article.
*[http://forum.osdev.org/viewtopic.php?f=1&t=21748 Loading a higher-half kernel] on setting up long mode for a higher-half kernel.
*[http://forum.osdev.org/viewtopic.php?f=1&t=21772 Setting up the stack after the switch to long mode] on the stack segment note.

===Wikipedia===
*[[Wikipedia:AMD64|AMD64]]
*[[Wikipedia:64-bit|64-bit]]

[[Category:X86-64‏]]
[[Category:Operating Modes]]
````

<a id="w002815"></a>

## W002815 — Talk:Porting Python

Bron-ID: W002815
Namespace: Talk (1)
Revisie: 9880 | Datum: 2010-02-28T08:21:25Z
Bron: https://wiki.osdev.org/Talk:Porting_Python
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APorting_Python&oldid=9880

````text
Correct me if I'm wrong, but Python 2.5 is ''way'' out of date. 2.6, not ''as'' much, but still..

--[[User:Kotuxguy|Kotuxguy]] 06:14, 22 February 2010 (UTC)

This tutorial works for 2.5 and 2.6. It hasn't been tested on 2.7 (as that is still in alpha), and I don't consider 3.0 worth discussing yet. Python 2.5.5 was released on January 31 2010.
    
If you'd like to contribute a tutorial for Python 3.0 rather than just complaining about nothing, be my guest ;)
    
--[[User:Pcmattman|pcmattman]] 06:48, 22 February 2010 (UTC)

Fair enough. I'll work on it as soon as I get my OS to that stage.

--[[User:Kotuxguy|Kotuxguy]] 06:25, 23 February 2010 (UTC)

Regarding the stage required for porting Python, in our OS, Python (strictly core python lib only) was the 3rd user mode program ever for our OS, after Hello World and 99 bottles of beer. Required porting newlib, and a way to build statically linked binaries for usermode, basic paging and processes - we did not do complete elf loader but just an easy format based on a custom linker script with all 3 sections at page aligned locations and 4k header. We did not have filesystem support either, only write and exit system calls were fully implemented in newlib. The executable in question originally initialized Python 2.5 interpreter and executed a python "hello world" script that was hardcoded in the executable. When we noticed that we really need filesystem for many modules to work correctly, only then we implemented FAT readonly support for ram disks :)

--[[User:Ztane|Ztane]] 11:27, 26 February 2010 (UTC)

Sure, it is possible to port Python with a very minimal libc. However, as you found out, without more than just the basic support in your libc you run into a lot of restrictions with Python (specifically, its modules). For example, you can't port modules which use networking until you write a Berkeley sockets layer.

The reason I decided to label this tutorial "Advanced" and imply a requirement of more than just a rudimentary libc is simply because the understanding of the toolchain, configure process, and library calls that comes from more experience with porting and using other software is very, very useful here.

--[[User:Pcmattman|pcmattman]] 23:34, 26 February 2010 (UTC)

Pcmattman, you are absolutely right, and I do not deny it does not require knowledge of porting, toolchain and system calls etc. But if you can do it, have toolchain and full libc with only some system calls (write, exit, sbrk), it might be useful, even if you do not yet have networking, or even filesystem code. My friend and I are very lazy and we just did not have that much time to implement every imaginable system call, before trying to port something to our OS. The beauty of porting python at this early stage for us is that we have an easy way to write a complex test program - just a few lines of python, which, under the hood will require lots of processing and also serve as a stress test. So first, we got to test if our newlib behaves acceptably with python, returning proper error codes from open, etc, and got a rather complicated test for our sbrk. So when we implemented the read only file system, we had already some test cases for it, that is, had to make python module imports for builtin modules to work, and to load userland initialization script from /init.py. I myself did not even believe we could get libpython itself to work with such minimal hacking in our 3-system call (write, sbrk, exit IIRC) environment, but it really did work. The only problem we had was some warning message telling that python cannot find its site libraries.

--[[User:Ztane|Ztane]] 08:21, 28 February 2010 (UTC)
````

<a id="w002816"></a>

## W002816 — AMD64

Bron-ID: W002816
Namespace: Main (0)
Revisie: 9888 | Datum: 2010-03-02T20:37:40Z
Bron: https://wiki.osdev.org/AMD64
Revisie-URL: https://wiki.osdev.org/index.php?title=AMD64&oldid=9888
Doorverwijzing: X86-64

````text
#REDIRECT [[x86-64]]
````

<a id="w002817"></a>

## W002817 — L4

Bron-ID: W002817
Namespace: Main (0)
Revisie: 28026 | Datum: 2023-07-09T18:12:09Z
Bron: https://wiki.osdev.org/L4
Revisie-URL: https://wiki.osdev.org/index.php?title=L4&oldid=28026

````text
{{Stub}}
L4 is a family of second-generation microkernels based on the original designs and implementations by German computer scientist Jochen Liedtke. Originally implemented as highly tuned Intel i386-specific assembly language code, the API has seen extensive development in a number of directions, both in achieving a higher grade of platform independence and also in improving security, isolation, and robustness.

The L4 µ-Kernel Family Homepage: http://os.inf.tu-dresden.de/L4/

== L4/x86 ==
The L4/x86 was the original design developed by Jochen Liedtke at GMD, IBM Watson Research Center, and Universität Karlsruhe.

Assembler Source Code: http://www.acm.uiuc.edu/white_papers/incoming/GMD/L4/

Reference Manuals: http://os.inf.tu-dresden.de/L4/l4doc.html

[[Category:Kernel]]
[[Category:OS theory]]
````

<a id="w002818"></a>

## W002818 — User:Combuster/Dreamcast Barebones

Bron-ID: W002818
Namespace: User (2)
Revisie: 10455 | Datum: 2010-06-22T09:03:03Z
Bron: https://wiki.osdev.org/User:Combuster/Dreamcast_Barebones
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACombuster%2FDreamcast_Barebones&oldid=10455

````text
{{Rating|2}}
Among the most common household Hitachi SuperH platform is the Sega Dreamcast. Since getting a working setup for this platform is non-trivial due to the amount of protection on the device. This tutorial is an A-Z on getting your first code to work on the dreamcast.

== The Toolchain ==
There are several tools you will need. First and foremost is the compiler. Since you are most likely developing on an PC, you'll need a dedicated tool for targeting a different architecture. Basically the steps are nearly identical to [[GCC Cross-Compiler]], but with a different target. Grab a copy of [[Cygwin]], The [[GCC]] and binutils sources, and make sure Cygwin has the necessary tools installed. A short version of the instructions follow which should be sufficient if you built a cross-compiler before. If you haven't done so before, please read the cross-compiler tutorial for an in-depth 

<pre>
export TARGET=sh4-elf
export PREFIX=/usr/cross
</pre>
We set up two variables: TARGET holds the architecture we will be targeting: ELF on SH4 (the instance of the processor found in a dreamcast), the prefix is for putting the cross-compiler somewhere safe from messing up the local system
<pre>
cd /usr/src
mkdir build-binutils-2
cd build-binutils-2
../binutils-x.xx/configure --target=$TARGET --prefix=$PREFIX --disable-nls
make all install
</pre>
The path is different here so that you don't get conflicts between your previous Binutils/GCC build and this one.

<pre>
export PATH=$PATH:/usr/cross/bin
cd /usr/src
mkdir build-gcc-2
cd build-gcc-2
../gcc-x.x.x/configure --target=$TARGET --prefix=$PREFIX --disable-nls --enable-languages=c --without-headers
make all-gcc install-gcc
</pre>

Furthermore, you need mkisofs to generate a CD filesystem, and two dedicated tools that deal with the copy protection: [http://dcemulation.org/?title=CDI4DC cdi4dc] (windows and linux binaries only) and [mc.pp.se/dc/files/scramble.c scramble]. This process will create a CDI format CD image, this is because the standard ISOs do not store all the data on the CD, but only the 2048 bytes of each logical data sector.

== Building the kernel ==
The SH4 is a bi-endian architecture, and can run in either little-endian or big-endian mode. Since the bootloader starts in little-endian mode, it is easiest to use the same endianness in your own code. Since the assembler defaults to big-endian, we need to override it with each invocation:
<pre>sh4-elf-gcc -ml -c -o file.o file.c
sh4-elf-as -little -o file.o file.s
</pre>
The kernel is expected to be a flat binary and gets loaded at 0x8c010000, which requires us to use the following linker script:
<pre>
OUTPUT_FORMAT("binary")
ENTRY(start)
phys = 0x8c010000;
SECTIONS
{
  .text phys : AT(phys) {
    code = .;
    *(.text)
    *(.rodata)
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
</pre>
and you can link your kernel as follows:
<pre>
sh4-elf-ld -o kernel.bin -T link.ld -lgcc -L/usr/cross/lib/gcc/sh4-elf/x.x.x/
</pre>

== Building the image ==
The dreamcast uses a form of encryption to load a binary. For the kernel to work we need to encrypt it, so it will come out correctly when decrypted. We also give the kernel binary the required name here:
<pre>
./scramble kernel.bin 1ST_READ.BIN
</pre>

Now continue building the image
<pre>
mkdir cdrom
cp 1ST_READ.BIN cdrom
mkisofs -G IP.BIN -C 0,11702 -J -l -r -V "MyImage" -o temp.iso cdrom
cdi4dc temp.iso output.cdi
</pre>
The process starts with setting up a subtree for the CD contents, we put the kernel in there, as well as any other files we want to put on the CD. This folder is then used as the root of the CD. The remaining arguments are as follows:
* <tt>-G IP.BIN</tt>
:this selects what binary to put at the bootable section of the image. We point it to SEGA's binary blob as its contents is verified by the BIOS. Note that this piece is also responsible for displaying the "produced by or under license from" message, designed to sue malicious parties over trademark infringement.
* <tt>-C 0,11702</tt>
:This is part of the DC protection scheme. While the intricate details are handled by cdi4dc, we have to tell mkisofs that the filesystem will not be put at the start of the disc, but several sectors later (11702 to be precise). mkisofs will add this offset to all references to keep the filesystem working on both the dreamcast and PC.
* <tt>-J</tt>
:Enable Joliet extensions which allows long filenames to be stored
* <tt>-l</tt>
:Disable DOS-style 8.3 formatting
*<tt> -r</tt>
:Remove permissions to prevent problems with owners and access rights.
*<tt>-V "MyImage"</tt>
:Set the volume name. Mainly for aesthetic purposes, modify as necessary.
*<tt>-o temp.iso</tt>
:Output file, since its unusable as ISO image (thanks to -C), its a temporary product
*<tt>cdrom</tt>
:Root of the filesystem, in our case its the cdrom directory

== See Also ==
'''External links'''
* [http://mc.pp.se/dc/sw.html Dreamcast resources]
* [http://dcemulation.org/?title=CDI4DC the cdi4dc tool]
````

<a id="w002819"></a>

## W002819 — Talk:Creating a 64-bit kernel

Bron-ID: W002819
Namespace: Talk (1)
Revisie: 18653 | Datum: 2015-11-26T09:29:20Z
Bron: https://wiki.osdev.org/Talk:Creating_a_64-bit_kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ACreating_a_64-bit_kernel&oldid=18653

````text
This article is originally written like a tutorial, which is insufficient. Long mode kernels has lots of special instincts that need to be well explained (most importantly the ABI 'small, kernel, medium, large' code modes). Piece by piece, I'll transform it to a 'guide' instead of its current tutorial tone. I hope this won't be objectionable. --[[User:Darwish|Darwish]] 16:50, 20 March 2010 (UTC)

== Disputed ==

This article needs to be integrated with the [[Bare Bones]] series. The advise here is a bit questionable. The article looks too simple, you do need to deal with paging and other stuff. If anyone reads this, I recommend you do a 32-bit x86 kernel instead as described in [[Bare Bones]] as your first kernel. I'll eventually get around to clean this up as better advice. --[[User:Sortie|Sortie]] 15:05, 30 November 2014 (CST)
````

<a id="w002820"></a>

## W002820 — Talk:DegenerateOS

Bron-ID: W002820
Namespace: Talk (1)
Revisie: 9981 | Datum: 2010-03-29T15:05:24Z
Bron: https://wiki.osdev.org/Talk:DegenerateOS
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ADegenerateOS&oldid=9981

````text
== Remove this page? ==
It seems to me this page has useful information, but the link has been dead for quite some time (Troy reported it in 2008), so my question was if we should remove this page (deprecated information?) or just let it stick around (while the project possibly doesn't exist anymore) and remove the links? --[[User:Creature|Creature]] 19:40, 26 March 2010 (UTC)

:I can't find any more traces off the internet other than down in the many (old) lists of operating systems - so yes, I can agree with deletion. - [[User:Combuster|Combuster]] 09:34, 29 March 2010 (UTC)

::Agreed, I'll wait a couple of days before actually deleting the article to see if anyone else has any objections. --[[User:Creature|Creature]] 15:05, 29 March 2010 (UTC)
````

<a id="w002821"></a>

## W002821 — Talk:EM64T

Bron-ID: W002821
Namespace: Talk (1)
Revisie: 9979 | Datum: 2010-03-29T09:38:58Z
Bron: https://wiki.osdev.org/Talk:EM64T
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AEM64T&oldid=9979

````text
== EM64T == Intel64 == x86-64 ==

EM64T is the same thing as Intel 64, just a re-branding.. x86-64 is the vendor neutral way of referring to all reimplementation of the AMD64 architecture. Intel tried to escape the x86 market all together with the IA64, which is a completely different architecture. --[[User:Brynet-inc|Brynet-Inc]] 01:32, 28 March 2010 (UTC)

Yes, in fact, IIRC, EM64T came first (which I found a very confusing term), the Intel 64 name came later (which cleared things up a lot).  At the time, I think some people at Intel still believed IA64 was going somewhere... [[User:Nedbrek|Nedbrek]] 19:38, 28 March 2010 (UTC)

:IMO Intel 64 too much resembles Intel Architecture 64 (= IA64, = Itanium), so I would personally avoid references to "Intel 64" and stick to EM64T/AMD64/x86-64, in order of increasing preference - [[User:Combuster|Combuster]] 09:38, 29 March 2010 (UTC)
````

<a id="w002822"></a>

## W002822 — Talk:Dev-C++

Bron-ID: W002822
Namespace: Talk (1)
Revisie: 9994 | Datum: 2010-03-31T05:37:44Z
Bron: https://wiki.osdev.org/Talk:Dev-C%2B%2B
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ADev-C%2B%2B&oldid=9994

````text
== Seems discontinued, no longer relevant. ==

The website appears to be home to some sort of domain name squatter now. - [[User:Brynet-Inc|Brynet-Inc]] (please sign)

:I suggest you look here: http://sourceforge.net/projects/dev-cpp/files/ - [[User:Combuster|Combuster]] 20:34, 29 March 2010 (UTC)

::Why? --[[User:Brynet-inc|Brynet-Inc]] 05:37, 31 March 2010 (UTC)
````

<a id="w002823"></a>

## W002823 — User talk:Dedndave

Bron-ID: W002823
Namespace: User talk (3)
Revisie: 9991 | Datum: 2010-03-30T15:44:57Z
Bron: https://wiki.osdev.org/User_talk:Dedndave
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ADedndave&oldid=9991

````text
Hi Troy,
  You altered the wiki page for MASM.
I wanted to explain to you the reason I had that line in there.
There are many different versions of the license for the MS MASM assembler.
The problem is, you don't always know which version of MASM the license you are reading applies to.
For older versions of MASM (6.14 and older), the license is more laxed.
The newer versions that support SSE2 and above, the license is more strict.
  Dave

:As far as facts go, he did *not* edit the [[MASM]] page. Why the sudden accusation? And why use this talkpage when the topic is relevant for all of us? - [[User:Combuster|Combuster]] 15:44, 30 March 2010 (UTC)
````

<a id="w002825"></a>

## W002825 — User:Bjork

Bron-ID: W002825
Namespace: User (2)
Revisie: 9996 | Datum: 2010-03-31T10:46:39Z
Bron: https://wiki.osdev.org/User:Bjork
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ABjork&oldid=9996

````text
== About me ==
I am 18 years old and live in Germany. My hobbies are swimming and programming which I started, when I was 12. After learning BASIC I began to program some games in C and later C++. When I was 16 I had the wish not to rely on the OS anymore. That's why I started to code my own OS.

== Languages ==
* C, C++
* Java
* Basic
* Pascal
* Html and php
* Assembler

== My OS ==
My OS is called "PIE". It has a microkernel, where all the basics are already done:
* IDT, GDT
* multitaskin
* IPC trough RPC
* paging and virtual memory management
Now I'm about to implement important drivers for VGA, the Floppy Disk and a Filesystem.
````

<a id="w002826"></a>

## W002826 — Talk:VAL

Bron-ID: W002826
Namespace: Talk (1)
Revisie: 10005 | Datum: 2010-03-31T21:58:43Z
Bron: https://wiki.osdev.org/Talk:VAL
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AVAL&oldid=10005

````text
== Removal? ==
This seems like a page made in a hurry and never continued (last update: 2007). Unless anyone else has any objection, I'd like to vote for removal. If anyone does have an objection, I'll try and clean up the article a bit. --[[User:Creature|Creature]] 20:28, 31 March 2010 (UTC)
:'''Disagree''': This page is lifted from the old osfaq - it basically exists for completeness sake. Deletion would result in a loss of potentially useful information. The fact that it exists means that it is (was once) used, and that it might be used again. That said, I added that stub tag long ago in the hope someone familiar with it would be able to expand on it. - [[User:Combuster|Combuster]] 21:58, 31 March 2010 (UTC)
````

<a id="w002827"></a>

## W002827 — User:Nedbrek/Extracting bit patterns from BDF

Bron-ID: W002827
Namespace: User (2)
Revisie: 10009 | Datum: 2010-04-01T11:39:37Z
Bron: https://wiki.osdev.org/User:Nedbrek/Extracting_bit_patterns_from_BDF
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANedbrek%2FExtracting_bit_patterns_from_BDF&oldid=10009

````text
The BDF format is plain text.  The spec is available from [http://partners.adobe.com/public/developer/en/font/5005.BDF_Spec.pdf Adobe].  It is very old, and quite amusing to read (it talks about handling the tapes).

==Awk parser==
Here is a simple parser in awk to extract the individual glyphs to a single line each:
<source lang="c">BEGIN {
   line = 0
}

/^STARTCHAR/, /^ENDCHAR/ {
   if ($1 == "STARTCHAR")
   {
      printf $2 "\t"
      line = 1
   }
   else if ($1 == "ENDCHAR")
   {
      printf "\n"
      line = 0
   }
   else if ($1 == "ENCODING")
   {
      printf $2 " "
   }
   else if ($1 == "BITMAP")
   {
      line = 2
   }
   else if( line == 2 )
   {
      printf $1 " "
   }
}</source>
This will produce output like:<pre>
exclam   33 00 20 20 20 20 20 00 20 00 00
quotedbl 34 00 50 50 50 00 00 00 00 00 00
</pre>
* The first column is the glyph name (useful for you to see, maybe not needed in your OS).
* The second column is the encoding.  0..127 are the standard ASCII codes.  128..255 are extended ASCII.  Larger numbers are UTF-8.
* The remaining columns are the rows of the glyph (padded to 8 bits).  This example is from the 6x10 font, so there are 10 rows, 6 bits each (the padding is in the least significant bits).  The most significant bits are leftmost in the glyph.  The first row is topmost.
````

<a id="w002828"></a>

## W002828 — Talk:I use a Custom Filesystem - What Bootloader Solution is right for me?

Bron-ID: W002828
Namespace: Talk (1)
Revisie: 14817 | Datum: 2013-06-04T04:37:46Z
Bron: https://wiki.osdev.org/Talk:I_use_a_Custom_Filesystem_-_What_Bootloader_Solution_is_right_for_me%3F
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AI_use_a_Custom_Filesystem_-_What_Bootloader_Solution_is_right_for_me%3F&oldid=14817

````text
== Article Should Be Deleted or Merged ==

This article is written like something you would expect to find in the forums, not the wiki.
I suggest that it either be deleted or the relevant portions be merged into the GRUB article, FAQ, or one of the beginner's tutorials. --[[User:54616e6e6572|54616e6e6572]] 00:44, 7 April 2010 (UTC)
: I have added this to the FAQ category. It looks like something that was directly copied from the old OSDev FAQ. I don't think it needs deleting or mergin with anything else, but it could certainly do with some tidying to bring its style in to line with the rest of the wiki. --[[User:Aj|Aj]] 08:30, 7 April 2010 (UTC)
::'''Keep''', potentially useful information must be preserved. As for the sentiments above, it's not from the MT wiki, it started as a contribution from Gravaera. [http://forum.osdev.org/viewtopic.php?f=8&t=21147 More on that] can be found in the forums. Nevertheless, I went over the article with glue and scissors and it looks decent now. I do think however that the propaganda part at the end can be removed as it adds absolutely nothing. - [[User:Combuster|Combuster]] 12:19, 7 April 2010 (UTC)
::'''Merge''' (with [[GRUB]]), potentially useful information would still be preserved. Moreover, I've noticed that whenever someone comes to #osdev asking for help, and says he's been using absolute sectors to tell GRUB what to load, the problem is that his kernel has overgrown the size they specify, and... boo. I recommend that all the relevant information is merged with GRUB, and rest deleted. If we still want to keep the page, doesn't it ignore the naming guidelines? --[[User:Shikhin|Shikhin]] 23:29, 3 June 2013 (CDT)
::'''Merge''' with [[GRUB]], and edit to highly discourage the usage of GRUB as a hack to load a larger bootloader for a custom filesystem. While the article contains potentially useful information, a whole page is not warranted and may be seen as a means of supporting the argument in favour of using a poorly-designed custom filesystem and using GRUB to assist in the loading process under it, which is a very kludgy and specific setup that may pose issues in the future and already has inherent issues with regards to support partitions and GRUB files. --[[User:Blacklight|Blacklight]]<sup>[[Special:Contributions/Blacklight|c]]&sdot;[[User_talk:Blacklight|t]]&sdot;[http://forum.osdev.org/memberlist.php?mode=viewprofile&amp;u=11964 f]</sup> 23:37, 3 June 2013 (CDT)
````

<a id="w002829"></a>

## W002829 — PCI Express

Bron-ID: W002829
Namespace: Main (0)
Revisie: 30435 | Datum: 2026-03-30T08:31:03Z
Bron: https://wiki.osdev.org/PCI_Express
Revisie-URL: https://wiki.osdev.org/index.php?title=PCI_Express&oldid=30435

````text
{{In Progress}}
The PCI Express bus is a backwards compatible, high performance, general purpose I/O interconnect bus, and was designed for a range of computing platforms. One of the key improvements of PCI Express, over the [[PCI|PCI Local Bus]], is that it now uses a serial interface (compared to the parallel interface used by PCI). This improvement can be compared to the similiar serialization of the ATA interface.

==PCI Express Link==
The PCI Express bus connects each device directly to the CPU and other system devices through a pair of high speed unidirectional differential links (transmit and recieve, respectively). These links operate at an effective rate of 2.5 GB/s and a single device may have multiple links. A single device may have x1, x2, x4, x8, x12, x16, or x32 links and can achieve a maximum bandwidth of 80 GB/s by utilizing x32 links.

==Extended Configuration Space==
The PCI Express bus extends the [[PCI#Configuration Space|Configuration Space]] from 256 bytes to 4096 bytes. This extended configuration space ''cannot'' be accessed using the legacy PCI method (through ports 0xCF8 and 0xCFC). Instead, an [[#Enhanced Configuration Mechanism|Enhanced Configuration Mechanism]] is provided.

However, the legacy configuration space for PCIe devices can still be accessed using the latter.

===Changes from the PCI Configuration Space===
{| class="wikitable"
|-
! Header Type !! Register (Offset) !! Bit Location !! Difference
|-
| rowspan="11" | All Headers
| rowspan="5" | Command Register (0x04)
| 3 || Special Cycle Enable: Does not apply to PCIe. Hardwired to 0.
|-
| 4 || Memory Write and Invalidate:  Does not apply to PCIe. Hardwired to 0.
|-
| 5 || VGA Palette Snoop:  Does not apply to PCIe. Hardwired to 0.
|-
| 7 || IDSEL Stepping/Wait Cycle Control:  Does not apply to PCIe. Hardwired to 0.
|-
| 9 || Fast Back-to-Back Transactions Enable:  Does not apply to PCIe. Hardwired to 0.
|-
| rowspan="4" | Status Register (0x06)
| 4 || Capabilities List: All PCIe devices are required to implement the capability structure. Hardwired to 1.
|-
| 5 || 66 MHz Capable:  Does not apply to PCIe. Hardwired to 0.
|-
| 7 || Fast Back-to-Back Transactions Capable:  Does not apply to PCIe. Hardwired to 0.
|-
| 10:9 || DEVSEL Timing:  Does not apply to PCIe. Hardwired to 0.
|-
| Cache Line Size Register (0x0C) || All Bits || Implemented for legacy purposes only.
|-
| Master Latency Timer Register (0x0D) || All Bits ||  Does not apply to PCIe. Hardwired to 0.
|-
| rowspan="2" | Type 0
| Base Address Registers (0x10:0x24) || All Bits || PCIe Endpoint devices must set the BAR's prefetchable bit while the range does not contain memory with read side-effects or where the memory does not tolerate write merging. 64-Bit Addressing MUST be supported by non legacy Endpoint devices. The minimum memory address range requested by a BAR 128-bytes.
|-
| Min_Gnt/Max_Lat Registers (0x3E:0x3F) || All Bits ||  Does not apply to PCIe. Hardwired to 0.
|-
| rowspan="13" | Type 1
| Base Address Registers (0x10:0x24) || All Bits || PCIe Endpoint devices must set the BAR's prefetchable bit while the range does not contain memory with read side-effects or where the memory does not tolerate write merging. 64-Bit Addressing MUST be supported by non legacy Endpoint devices. The minimum memory address range requested by a BAR 128-bytes.
|-
| Primary Bus Number (0x18) || All Bits || Implemented for legacy purposes only.
|-
| Secondary Latency Timer (0x1B) || All Bits || Does not apply to PCIe. Hardwired to 0.
|-
| rowspan="3" | Secondary Status Register (0x1E)
| 5 || 66 MHz Capable: Does not apply to PCIe. Hardwired to 0.
|-
| 7 || Fast Back-to-Back Transactions Capable: Does not apply to PCIe. Hardwired to 0.
|-
| 10:9 || DEVSEL Timing: Does not apply to PCIe. Hardwired to 0.
|-
| Prefetchable Memory Base/Limit (0x24) || All Bits || Must indicate support for 64-bit addresses.
|-
| rowspan="6" | Bridge Control Register (0x3E)
| 5 || Master Abort Mode: Does not apply to PCIe. Hardwired to 0.
|-
| 7 || Fast Back-to-Back Transactions Enable: Does not apply to PCIe. Hardwired to 0.
|-
| 8 || Primary Discard Timer: Does not apply to PCIe. Hardwired to 0.
|-
| 9 || Secondary Discard Timer: Does not apply to PCIe. Hardwired to 0.
|-
| 10 || Discard Timer Status: Does not apply to PCIe. Hardwired to 0.
|-
| 11 || Discard Timer SERR# Enable: Does not apply to PCIe. Hardwired to 0.
|}

==Extended PCI Bus Numbering==

Older variations of PCI (e.g. "PCI Conventional") were limited to a maximum of 256 PCI bus segments. PCI Express extends this by introducing "PCI Segment Groups", where a system could (in theory) have up to 65536 PCI Segment Groups with 256 PCI bus segments per group, thereby allowing a single computer to have up to a maximum of 2<sup>24</sup> (16777216) PCI bus segments.

PCI Segment Groups are numbered, and in most systems there is only one PCI Segment Group (PCI Segment Group number 0). Note that legacy PCI configuration space access mechanism #1 (which still exists for backward compatibility) has no "PCI Segment Group" field and therefore can only be used to access the PCI configuration space for PCI Segment Group number 0. The [[#Enhanced Configuration Mechanism]] must be used to access the PCI configuration space for any devices in other PCI Segment Groups.

== Enhanced Configuration Mechanism ==
The enhanced configuration mechanism makes use of memory mapped address space range/s to access PCI configuration space. Put simply, the memory address determines the segment group, bus, device, function and register being accessed. On x86 and x64 platforms, the address of each memory area is determined by the ACPI 'MCFG' table. The format of this ACPI table is:

{| class="wikitable"
|-
! Offset !! Length
! colspan="3" | Description
|-
| 0 || 4
| colspan="3" | Table Signature ("MCFG")
|-
| 4 || 4
| colspan="3" | Length of table (in bytes)
|-
| 8 || 1
| colspan="3" | Revision (1)
|-
| 9 || 1
| colspan="3" | Checksum (sum of all bytes in table & 0xFF = 0)
|-
| 10 || 6
| colspan="3" | OEM ID (same meaning as other ACPI tables)
|-
| 16 || 8
| colspan="3" | OEM table ID (manufacturer model ID)
|-
| 24 || 4
| colspan="3" | OEM Revision (same meaning as other ACPI tables)
|-
| 28 || 4
| colspan="3" | Creator ID (same meaning as other ACPI tables)
|-
| 32 || 4
| colspan="3" | Creator Revision (same meaning as other ACPI tables)
|-
| 36 || 8
| colspan="3" | Reserved
|-
| rowspan="7" | 44 + (16 * n)
| rowspan="7" | 16
| colspan="3" | Configuration space base address allocation structures. Each structure uses the following format:
|-
! Offset !! Length !! Description
|-
| 0 || 8 || Base address of enhanced configuration mechanism
|-
| 8 || 2 || PCI Segment Group Number
|-
| 10 || 1 || Start PCI bus number decoded by this host bridge
|-
| 11 || 1 || End PCI bus number decoded by this host bridge
|-
| 12 || 4 || Reserved
|}

For non-x86 systems the method varies, but usually systems provide themselves with a [[Devicetree]] which is parsed at runtime.

To access a specific register within a device's PCI configuration space, you have to use the device's PCI Segment Group and bus to determine which memory mapped PCI configuration space area to use, and obtain the starting physical address and starting bus number for that memory mapped area. Once you have the correct starting physical address and starting bus number for that memory mapped area you would use the following formula to determine where the (4096-byte) area for a function's PCI configuration space is: <code>Physical_Address = MMIO_Starting_Physical_Address + ((Bus) << 20 | Device << 15 | Function << 12)</code>. Here, the <code>MMIO_Starting_Physical_Address</code> is the base address of the ECAM region from the MCFG table, and is always relative to bus number zero (ie. the configuration space for all bus 0 functions resides starting at the address in the table itself). You must start enumerating at the start bus number decoded by the host bridge.

Note that it may be a good idea to determine "physical address for this function's PCI configuration space" as part of PCI enumeration and store this physical address alongside any other information you're using to manage PCI devices and drivers (e.g. in your "device manager's" hierarchical tree of device info).

Also note that for absolute maximums (with 65536 PCI segment groups and 256 bus segments per segment group), the amount of physical address space consumed by memory mapped PCI configuration space ranges would be (up to) 16 TiB (or 2<sup>44</sup> bytes); and ACPI's "MCFG" table may (in theory) be slightly larger than 256 MiB (a 16-byte entry for each individual PCI bus within each PCI segment group plus the 36-byte table header).

Accessing a specific device within the space can be as follows: <code>(((bus * 256) + (slot * 8) + func) * 4096) + offs</code> (each device descriptor is 4096 bytes or 4K long), therefore the former can be interpreted as an array of the type: <code>((pcie_ecam[bus][slot][func] * 4096) + offs)</code>.

Finding devices can be done [[PCI#Enumerating_PCI_Buses|the same as PCI]], with the difference that the kernel accesses a memory region rather than using CPU I/O.

== Extended Capibilities ==
PCIe defines a number of extended capabilities, which reside in the extended configuration space.

However, the first PCIe capability to look for is in fact a PCI capability in the regular configuration space. This tells you, for instance that it is a PCIe device.

=== PCI Express Capability ===
{| class="wikitable"
|+PCIe Capability [PCI CAP]
!X
!31-24
!23-16
!15-8
!7-0
|-
|00
| colspan="2" |PCIe Capabilities
|NEXT
|CAPID
|-
|04
| colspan="4" |Device Capabilities
|-
|08
| colspan="2" |Device Status
| colspan="2" |Device Control
|-
|0C
| colspan="4" |Link Capabilities
|-
|10
| colspan="2" |Link Status
| colspan="2" |Link Control
|-
|14
| colspan="4" |Slot Capabilities
|-
|18
| colspan="2" |Slot Status
| colspan="2" |Slot Control
|-
|1C
| colspan="2" |Root Capabilities
| colspan="2" |Root Control
|-
|20
| colspan="4" |Root Status
|-
|24
| colspan="4" |Device Capabilities 2
|-
|28
| colspan="2" |Device Status 2
| colspan="2" |Device Control 2
|-
|2C
| colspan="4" |Link Capabilities 2
|-
|30
| colspan="2" |Link Status 2
| colspan="2" |Link Control 2
|-
|34
| colspan="4" |Slot Capabilities 2
|-
|38
| colspan="2" |Slot Status 2
| colspan="2" |Slot Control 2
|}

* NEXT is as for any other PCI capability, a configuration space pointer.
* CAPID is 0x10

==== PCIe Capabilities ====
This is a read-only register.
{| class="wikitable"
|+PCIe Capabilities
!15
!14
!13-9
!8
!7-4
!3-0
|-
|RsvdP
|Undef
|IRMSGNO
|SI
|DPTYPE
|CAPVER
|}

* IRMSGNO - Interrupt Message Number, MSI/MSI-X status message.
* SI - Slot Implemented Device (i.e. not integrated into the system, so onboard graphics is not SI).
* CAPVER - Capability Version. This is version 2 described here. Version 1 doesn't have Root Capabilities nor the "2" fields.
* Device/Port type:

{| class="wikitable"
|+
!Value
!Type
|-
|0x0
|PCI Express Endpoint
|-
|0x1
|Legacy PCIe Endpoint
|-
|0x4
|Root Port of PCIe Root Complex
|-
|0x5
|PCIe Switch Upstream Port
|-
|0x6
|PCIe Switch Downstream Port
|-
|0x7
|PCIe to PCI(-X) Bridge
|-
|0x8
|PCI(-X) to PCIe Bridge
|-
|0x9
|RCiEP [Integrated Endpoint]
|-
|0xA
|Root Complex Event Collector
|}

==== Device Capabilities ====
This is a read-only register.
{| class="wikitable"
|+Device Capabilities
!31-29
!28
!27-26
!25-18
!17-16
!15
!14-12
!11-9
!8-6
!5
!4-3
!2-0
|-
|RsvdP
|FLRC
|CSPLC
|CSPLV
|RsvdP
|RBER
|Undef
|L1AL
|L0AL
|ETFS
|PFS
|MPS
|}

=== PCI Express Extended Capabilities ===
Now for the extended, PCIe only, capabilities. The first extended capability is always located at offset 0x100. The header format is as follows:
{| class="wikitable"
|+
!31-20
!19-16
!15-0
|-
|NEXT
|VER
|EXTCAPID
|}

* EXTCAPID - the ID for this capability.
* VER - Version of the header format. 1 for PCIe 1.1+
* NEXT - offset of next capability from the beginning of configuration space. Either 0 to end the list, or >= 0x100.

=== Secondary PCI Express Extended Capability ===
{| class="wikitable"
|+
!X
!31-16
!15-0
|-
|00
| colspan="2" |Header
|-
|04
| colspan="2" |Link Control 3
|-
|08
| colspan="2" |Lane Error Status
|-
|0C
|L1EQ
|L0EQ
|-
|...
|...
|L2EQ
|-
|48
|L31EQ
|L30EQ
|}

* The header is in the common format, but with an EXTCAPID of 0x0019.

{| class="wikitable"
|+Link Control 3
!31-16
!15-9
!8-2
!1
!0
|-
|RsvdP
|SKPGV
|RsvdP
|LERIE
|EQEN
|}

* EQEN - Peform link equalization. Applies during link retraining at 8 GT/s+.
* LERIE - Enables interrupt indicating link equalization request
* SKPGV - Enable Lower SKP OS Generation Vector

=== SR-IOV Capability ===
{| class="wikitable"
|+
!X
!31-16
!15-0
|-
|00
| colspan="2" |Header
|-
|04
| colspan="2" |Capibilities
|-
|08
|Status
|Control
|-
|0C
|TotalVFs
|InitialVFs
|-
|10
|Fcn Dep Link
|NumVFs
|-
|14
|VF Stride
|FirstVF Offset
|-
|18
| colspan="2" |Supported Page Sizes
|-
|1C
| colspan="2" |System Page Size
|-
|20
| colspan="2" |VF BAR0
|-
|24
| colspan="2" |VF BAR1
|-
|28
| colspan="2" |VF BAR2
|-
|2C
| colspan="2" |VF BAR3
|-
|30
| colspan="2" |VF BAR4
|-
|34
| colspan="2" |VF BAR5
|-
|38
| colspan="2" |VF Migration State Array Offset
|}

* EXTCAPID is 0x0010

==See Also==
===References===
*PCI Express Base Specification, revision 1.1, PCI Special Interest Group, March 28, 2005
[[Category:PCI]]
````

<a id="w002831"></a>

## W002831 — System Call

Bron-ID: W002831
Namespace: Main (0)
Revisie: 10038 | Datum: 2010-04-14T14:29:50Z
Bron: https://wiki.osdev.org/System_Call
Revisie-URL: https://wiki.osdev.org/index.php?title=System_Call&oldid=10038
Doorverwijzing: System Calls

````text
#REDIRECT [[System_Calls]]
````

<a id="w002833"></a>

## W002833 — User talk:Osdnlo

Bron-ID: W002833
Namespace: User talk (3)
Revisie: 10093 | Datum: 2010-04-19T19:42:21Z
Bron: https://wiki.osdev.org/User_talk:Osdnlo
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AOsdnlo&oldid=10093

````text

````

<a id="w002834"></a>

## W002834 — Keyboard

Bron-ID: W002834
Namespace: Main (0)
Revisie: 17288 | Datum: 2014-12-03T22:46:04Z
Bron: https://wiki.osdev.org/Keyboard
Revisie-URL: https://wiki.osdev.org/index.php?title=Keyboard&oldid=17288
Doorverwijzing: PS/2 Keyboard

````text
#REDIRECT [[PS/2_Keyboard]]
````

<a id="w002835"></a>

## W002835 — Talk:Grub Error 13

Bron-ID: W002835
Namespace: Talk (1)
Revisie: 10087 | Datum: 2010-04-19T13:12:59Z
Bron: https://wiki.osdev.org/Talk:Grub_Error_13
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AGrub_Error_13&oldid=10087

````text
== Comments ==

'''Note: This article places the Multiboot header in a separate section in an attempt of good coding practice. The problem with this technique is that once the kernel becomes big enough, the section may or may not reside in the first 8 KiB of the image. Getting your hands dirty with manually modifying the order of the sections is not a good programming practice. There is nothing wrong with having the header in the .text section; In fact I would strongly recommend it.''' --[[User:Love4boobies|Love4boobies]] 08:59, 18 April 2010 (UTC)

: '''Wrong''' - putting the multiboot header in .text makes it completely vulnerable to file placement, and is the prime cause for the ton of forum threads "I get error 13 when I expand my kernel". The solution is the exact opposite: place it in a separate section, and force that section up front via the linker script (which you'll already have anyway), so the linker is forced to place it up front in the binary, and not somewhere among the .text objects at the whims of the linker. (Comment removed from original page) - [[User:Combuster|Combuster]] 08:44, 19 April 2010 (UTC)

::Using a linker script and proper alignment is the key, I can't see why a separate section is needed. Another problem with using a separate section is that it doesn't work with all executable formats. - [[User:Love4boobies|Love4boobies]] 09:54, 19 April 2010 (UTC)

: File placement is the problem, with a linker script you can make the multiboot header reside at the beginning of the file without having to know what file it is in. Just putting it in the .text means that if the file containing is moved past 8KiB or your linker decides to reorder the .text section, it won't be in the correct position (this happened to me when building on windows, I was using *.o in my linker script because the command line was too long and my mboot header moved too far) - [[User:Thepowersgang|Thepowersgang]] 13:12, 19 April 2010 (UTC)
````

<a id="w002836"></a>

## W002836 — User:Combuster/Recovered USB Sheet

Bron-ID: W002836
Namespace: User (2)
Revisie: 10094 | Datum: 2010-04-19T20:08:20Z
Bron: https://wiki.osdev.org/User:Combuster/Recovered_USB_Sheet
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACombuster%2FRecovered_USB_Sheet&oldid=10094

````text
==USB UHCI Enumeration==
*<span style="color:#000;font-weight:bold;">Enumerate the UHCI Controller(s)</span>
**Class: 0x0C, Sub: 0x03, Intf: 0x00
*<span style="color:#000;font-weight:bold;">Store the I/O Base Address and Size</span>
**<span style="color:#545454;font-weight:bold;">Determine Access Type</span>
***Address & (1<<0) ? IOMAP : MEMIO
**<span style="color:#545454;font-weight:bold;">Calculate I/O Address (MEMIO)</span>
***Address & 0xFFFFFFF0
**<span style="color:#545454;font-weight:bold;">Calculate I/O Address (IOMAP)</span>
***Address & 0xFFFFFFFC
**<span style="color:#545454;font-weight:bold;">Calculate Size of I/O</span>
***Address & 0xFFFFFFFF
*<span style="color:#000;font-weight:bold;">Store IRQ</span>
**8 bit read to Base + 0x3C
*<span style="color:#000;font-weight:bold;">Enable Bus Master and IO Access</span>
**<span style="color:#545454;font-weight:bold;">MEMIO Access</span>
***16 bit write to Base + 0x04 of value 0x0006
**<span style="color:#545454;font-weight:bold;">IOMAP Access</span>
***16 bit write to Base + 0x04 of value 0x0005
**<span style="color:#545454;font-weight:bold;">MEMIO & IOMAP Access</span>
***16 bit write to Base + 0x04 of value 0x0007
*<span style="color:#000;font-weight:bold;">Disable Legacy Support</span>
**16 bit write to Base + 0xC0 of value 0x8F00
**16 bit write to Base + 0xC0 of value 0x2000
*<span style="color:#000;font-weight:bold;">GRESET</span>
**16 bit write to Base + 0x00 of value (1<<2)
**Delay for 50 ms
**16 bit write to Base + 0x00 of value (~(1<<2))
**Delay for 50 ms

==Host Controller Configuration==
*<span style="color:#000;font-weight:bold;">Allocate Physical Memory</span>
*<span style="color:#000;font-weight:bold;">Initialize the Queue Chain</span>
**qh.h = 0x00000001
**qh.v = 0x00000001
*<span style="color:#000;font-weight:bold;">Initialize the Frame List</span>
**frlist[0-1023] = QH.head | (1<<1)
*<span style="color:#000;font-weight:bold;">Set Basic Configurations</span>
**<span style="color:#545454;font-weight:bold;">SOFMOD</span>
***16 bit write to Base + 0x0C of value 0x40
**<span style="color:#545454;font-weight:bold;">FRBASEADD</span>
***32 bit write to Base + 0x08 of value [physical address aligned to 4KB boundary]
**<span style="color:#545454;font-weight:bold;">FRNUM</span>
***16 bit write to Base + 0x06 of value 0
**<span style="color:#545454;font-weight:bold;">USBINTR</span>
***16 bit write to Base + 0x04 of value 0x000F
*<span style="color:#000;font-weight:bold;">Start the Host Controller</span>
**16 bit write to Base + 0x00 of value (1<<0)

==Detecting Device Attachment==
*<span style="color:#000;font-weight:bold;">Test the CSC bit</span>
**16 bit read to Base + [port #]
**Value & (1<<1) ? TRUE : FALSE
*<span style="color:#545454;font-weight:bold;">TRUE</span>
**<span style="color:#000;font-weight:bold;">Clear the CSC bit</span>
***16 bit write to Base + [port #] of value (~(1<<1))
**<span style="color:#000;font-weight:bold;">Test the CCS bit</span>
***16 bit read to Base + [port #]
***Value & (1<<0) ? TRUE : FALSE
**<span style="color:#545454;font-weight:bold;">TRUE</span>
***Device Attached
**<span style="color:#545454;font-weight:bold;">FALSE</span>
***Device Not Attached
*<span style="color:#545454;font-weight:bold;">FALSE</span>
**Repeat

==Basic Device Configuration==
*In Progress...
````

<a id="w002837"></a>

## W002837 — User:Osdnlo

Bron-ID: W002837
Namespace: User (2)
Revisie: 10095 | Datum: 2010-04-19T20:08:20Z
Bron: https://wiki.osdev.org/User:Osdnlo
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AOsdnlo&oldid=10095
Doorverwijzing: User:Combuster/Recovered USB Sheet

````text
#REDIRECT [[User:Combuster/Recovered USB Sheet]]
````

<a id="w002838"></a>

## W002838 — User talk:Rudimk

Bron-ID: W002838
Namespace: User talk (3)
Revisie: 10327 | Datum: 2010-05-19T15:07:17Z
Bron: https://wiki.osdev.org/User_talk:Rudimk
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ARudimk&oldid=10327

````text

````

<a id="w002839"></a>

## W002839 — Port IO

Bron-ID: W002839
Namespace: Main (0)
Revisie: 29780 | Datum: 2025-12-03T10:14:58Z
Bron: https://wiki.osdev.org/Port_IO
Revisie-URL: https://wiki.osdev.org/index.php?title=Port_IO&oldid=29780

````text
The x86 architecture separates the address space in two programmatically distinct groups: memory and ports. In ancient history, memory was used as the storage of data where reads and writes would not have side-effects, and ports were used to control external hardware, which needed different timings to work. Which is also why accessing ports is so much slower than accessing memory. Many other common architectures have a unified space, where devices run at the same speed as memory, or where the address space is divided into blocks with separately configurable properties. 

Modern x86 hardware tends more and more toward the unified space, but still contains the port for legacy reasons.

== How to access ports ==
The x86 contains four opcodes that read from port space: IN, INS*, OUT and OUTS*, each come in a 8-bit and 16-bit version, as well as 32-bit for 386s and later.

== Which ports belong to this device ==
There are three groups of devices, each requiring a different approach. The first group contains all PC-AT hardware, devices like [[PIT|timers]], [[PIC|interrupt controllers]], [[Keyboard Controller|PS/2]] ports, [[Serial Ports|serial]] and parallel port, floppy and [[IDE|disk drive]], and the [[VGA Hardware|VGA]]. Back in the old days devices were attached to fixed addresses, and they remained there for backwards compatibility. The programming information will tell you the ports used.

The second group is plug-and-play hardware. Most modern cards and optional features are PCI, AGP or PCI express. At boot, their requirements are checked, and the BIOS will assign port ranges to each device. Enumerating the [[PCI]] bus will list all these devices, and can also tell you what ports (and memory mapped ranges) they use.

The third group is old non-plug-and-play compatible hardware. They have jumpers or fixed addresses that physically select the ports in use. Most ISA cards behave this way. You will have to probe the port range for possible locations to see if the device in question exists. Alternatively, you can ask the user to enter the details (like the soundblaster configurations seen in DOS games). The problem is, it is very hard to list all devices, which makes mapping other devices without conflicts a potential problem.

== What device belongs to this port ==
The only real way is to find all devices on the system, then look up your favourite port among all devices. But there is hardly any use for this kind of query.

== See Also ==
=== Articles ===
* [[I/O Ports]]

[[Category:X86]]
````

<a id="w002840"></a>

## W002840 — Serial ports

Bron-ID: W002840
Namespace: Main (0)
Revisie: 10119 | Datum: 2010-04-26T19:04:13Z
Bron: https://wiki.osdev.org/Serial_ports
Revisie-URL: https://wiki.osdev.org/index.php?title=Serial_ports&oldid=10119
Doorverwijzing: Serial Ports

````text
#REDIRECT [[Serial Ports]]
````

<a id="w002841"></a>

## W002841 — Talk:Serial ports

Bron-ID: W002841
Namespace: Talk (1)
Revisie: 10121 | Datum: 2010-04-26T19:04:13Z
Bron: https://wiki.osdev.org/Talk:Serial_ports
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ASerial_ports&oldid=10121
Doorverwijzing: Talk:Serial Ports

````text
#REDIRECT [[Talk:Serial Ports]]
````

<a id="w002842"></a>

## W002842 — User:Combuster/Port IO

Bron-ID: W002842
Namespace: User (2)
Revisie: 10124 | Datum: 2010-04-26T19:05:15Z
Bron: https://wiki.osdev.org/User:Combuster/Port_IO
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACombuster%2FPort_IO&oldid=10124
Doorverwijzing: Port IO

````text
#REDIRECT [[Port IO]]
````

<a id="w002843"></a>

## W002843 — Everything about ports

Bron-ID: W002843
Namespace: Main (0)
Revisie: 10125 | Datum: 2010-04-26T19:06:09Z
Bron: https://wiki.osdev.org/Everything_about_ports
Revisie-URL: https://wiki.osdev.org/index.php?title=Everything_about_ports&oldid=10125
Doorverwijzing: Port IO

````text
#REDIRECT [[Port IO]]
[[Category:FAQ]]
````

<a id="w002844"></a>

## W002844 — Serial Port

Bron-ID: W002844
Namespace: Main (0)
Revisie: 29101 | Datum: 2024-06-07T13:39:51Z
Bron: https://wiki.osdev.org/Serial_Port
Revisie-URL: https://wiki.osdev.org/index.php?title=Serial_Port&oldid=29101

````text
== Introduction ==

A Serial Port is the generic term for a port on a computer that can be connected to other devices. This may be a proprietary connector with a few wires in it; an AppleBus/AppleTalk connector to connect a printer, modem or another computer; or even an Ethernet port. But by far the most common usage is for an RS-232 port with a 9-pin D-connector (DE-9) for asynchronous communications (see [[Communications]]). All these ports have one thing in common: the data is sent over a small number of wires (usually one in each direction) in serial fashion, rather than all at once in parallel.

But since "Serial Port" is usually used to refer to the COM Port that is found on the PC (to distinguish it from the rarely-seen-nowadays "Parallel Port"), this page will concentrate on that. Refer to the specific pages for other port types. This page will cover the physical port itself, its signals and how they're commonly used, and the most common protocol used on this port.

== Physical Port ==

The original IBM PC computer came with two communication ports: a parallel port for connection to a printer, and a serial port to connect to a modem for external communications. Of course, over time those ports were used for other purposes, but that was their original intended usage.

Confusingly, both ports used the then already established standard connector called the DB-25.
* The D-series range has (usually) two columns of pins offset from each other to provide both ease of construction, and a definite polarity to the plug to prevent it from being plugged in upside down. Anecdotally, it was so-named because the metal shielding shroud protecting the pins looked like the letter 'D'.
* The letter after the 'D' indicates the layout of the pins; and often the number of them, making the suffix number redundant. For example, the first in the series is the DA-15, followed by the DB-25 and DC-37. The later DE-9 was adopted for COM ports on the IBM PC-AT, since only 9 of the 25 defined serial port pins were used.

To prevent plugging a device into the wrong port, the DB-25s used were different genders: the parallel port was female, needing a male printer cable, while the serial port was male, needing a female modem cable.

=== DB-25 ===

IBM used the DB-25 connector to match other computing products that it was selling. The existing layout allowed for multiple serial ports within the one connector - the so-called Primary and Secondary ports. It also had clock pins for high-speed Synchronous ports as well. But for the IBM PC, only the Asynchronous Primary port pins were used, which meant only 9 of the 25 pins were needed.

=== DE-9 ===

In the later IBM PC "Advanced Technology" (AT), they used the smaller DE-9 connector for more flexible port layout on the rear of the PC.

== Signals ==

The standard signals on the Serial Port were designed to talk to an external serial communications device called a MoDem (Modulator/Demodulator), to allow the computer to be connected over long-distance wires - telephone, ISDN or other long haul methods. The signalling conventions for modems had been well established, and required multiple control signals as well as the simple data in and data out wires.

The naming conventions identified the two endpoints of the cable as Data Terminal Equipment (DTE) and Data Communications Equipment (DCE) for computer and modem respectively. The signal names don't usually refer to these, but their definitions do: for example, Transmit Data (TX or TD) is defined to be "Data sent from the DTE to the DCE" - which means the TX pin on the modem is an input pin!

=== Electrical ===

The signals used for the normal IBM PC COM port are defined by the RS-232 standard - but there are other standards such as RS-422 and RS-485. These are better for longer cables, or in (electrically) noisier environments; places that electrical interference can change the signal being communicated.

Each standard defines the voltage levels required to receive a HIGH ('1') or LOW ('0') signal. Obviously, the transmitter needs to produce at least this much - but voltage drop over the cable means that it should probably produce more. So the standards also define a maximum voltage too.

==== TTL ====

This term is often used to indicate the signal levels coming straight out of the chip, whether that's the PC's 8250 or 16450 UART, or a microcontroller. It stands for Transistor/Transistor Logic, and historically means 0V and 5V for LOW and HIGH respectively. These days, the chips usually have different Logic standards with different voltage levels, so "TTL" is a misnomer. But the concept is the same: Minimum for LOW, Maximum for HIGH. Only, these signals aren't "driven" very hard, so can't be transmitted over much distance - well under a metre. But that's fine for two chips or boards that are next to each other.

==== RS-232 ====

RS-232 defines all of the signals used for serial communications, as well as specifying the voltages that the receiver needs to detect to interpret the values of HIGH and LOW. It also demands that the signals be "driven" hard enough to cover tens of metres, which means that both sides can be at opposite ends of a room.

Each of the signals use the same scheme: a LOW is signalled with a voltage between +3V and +25V, while a HIGH is between -3V and -25V, relative to the Signal Ground (GND). Note that a disconnected cable would produce 0V, which is neither HIGH nor LOW. This "dead band" is deliberate, to reduce the effect of electrical noise.

==== RS-422 ====

RS-422 defines the same set of signals as RS-232, but defines that each signal be sent over a ''pair'' of wires, known as a "balanced" circuit. Rather than each signal being referenced to a common ground, each signal is referenced between its two wires. The signal names are modified to suffix 'A' or 'B", or occasionally '+' and '-', to each of the signals' wires. LOW is indicated by 'B' being at least 0.2V lower than 'A', while HIGH is indicated by 'B' being at least 0.2V higher than 'A'. The maximum difference between the two must be no more than 12V.

This "common mode rejection" system takes advantage of the phenomenon that external electrical interference on the cable raises or lowers the voltage on both wires by similar amounts, which doesn't alter their relative voltage difference. That means that cables can be hundreds of metres long for even high data rates, without the signal deteriorating.

==== RS-485 ====

RS-485 defines the same set of signals as RS-422 and RS-232, and also defines the same voltages and balanced wiring as RS-422. But it also defines that the transmitter must be able to be turned off. This means that multiple transmitters, and all the receivers, can be connected to the same cable, and only the one currently transmitting is actually driving the signal.

This has the benefits of the cable length of RS-422, but allows a single cable run rather than individual runs for each pair of equipment. The drawback is that only one transmitter can be using the cable at any one time.

=== Pinout ===

These pinouts are identical for both the computer (DTE) and modem (DCE), thus only a straight-through cable is required - but remember that the names are computer-centric.

{| class="wikitable"
|-
! Pin name
! Abbreviation
! DB-25 pin
! DE-9 pin
! Original function
|-
| Shield
|
| 1
| -
| Tied to the cable shield on ONE end, to shield the whole cable
|-
| Ground
| GND
| 7
| 5
| Signal ground reference
|-
| Transmit Data
| TD
| 2
| 3
| Data from the DTE to the DCE
|-
| Receive Data
| RD
| 3
| 2
| Data from the DCE to the DTE
|-
| Data Set Ready
| DSR
| 6
| 6
| Set HIGH by the DCE to indicate it has power
|-
| Ring Indicator
| RI
| 22
| 9
| Pulsed by the DCE to indicate an incoming call
|-
| Data Terminal Ready
| DTR
| 20
| 4
| Set HIGH by the DTE to request link establishment
|-
| Data Carrier Detect
| DCD
| 8
| 1
| Set HIGH by the DCE to indicate it has a connection to the remote modem
|-
| Request To Send
| RTS
| 4
| 7
| Set HIGH by the DTE to indicate it has data to send
|-
| Clear To Send
| CTS
| 5
| 8
| Set HIGH by the DCE to allow the DTE to send
|}

=== Usage ===

The original usage of these signals were for very simple electrical "slave" DCEs to be connected to the barely more intelligent "master" DTEs, so to establish a communications link required a sequence of steps. As modems became more sophisticated, a lot of the functions became commands over the data link versus dedicated signals - and yet the signals remained. Still later, the Serial Port was used for communicating with local devices rather than modems, and the line between DTE and DCE blurred so that no true "master" was obvious - and yet the signals remained. Indeed, their functions were redefined to better suit the requirements of the local environment, so that the signals became flow controllers rather than requesters/granters.

==== Original ====

The original sequence of steps to communicate with a modem were as follows:

<ol>
<li>The DCE raises DSR to tell the DTE that it is able to be used.</li>
<li>Link establishment:</li>
'''''EITHER'''''
<ol type="a">
<li>The DTE raises DTR to tell the DCE to establish a link;</li>
</ol>
'''''OR'''''
<ol type="a">
<li>The DCE pulses RI to indicate an incoming call;</li>
<li>The DTE raises DTR to tell the DCE to accept the incoming call.</li>
</ol>
At this point, a link has been established, but no data can be transmitted or received.

Before data can be transmitted:
<li>The DTE raises RTS to request transmission</li>
<li>The ''local'' DCE starts transmitting a carrier to the ''remote'' DCE</li>
<li>The ''remote'' DCE raises DCD to ''its'' DTE, to indicate the carrier has been detected
<li>The ''local'' DCE raises CTS to indicate that the link is ready for transmission.
</ol>

Note that, depending on whether the modem is half-duplex or full-duplex, the RTS/CTS/DCD signalling may not be able to be performed in both directions simultaneously. For example, if the modems were connected to a radio transmitter/receiver pair that were operating on the same frequency, the fact that DCD was raised meant that the frequency was already in use, and raising RTS would have no effect.

===== Original Null Modem =====

To remove the modem from between two computers, a full "null modem" cable was required to be wired, to emulate what an instant-response modem would look like: for example, raising RTS would ''instantly'' raise CTS in response, while simultaneously telling the other computer that data was about to come.

  GND -------------- GND
  
  DTR ------\/------ DTR
  DSR ------/\------ DSR
  
  TX  ------\/------  TX
  RX  ------/\------  RX
  
            /------- DCD
           /
  RTS ----/     /--- RTS
  CTS ---/     /---- CTS
              /
  DCD -------/

This is most definitely a full-duplex cable - but is NOT suitable for today's "null modem" cable, detailed below.

==== Local link ====

Where the two computers that need to communicate are so close that no modem is required, a direct cable can be used instead. Here, there is no "prepare channel for transmission" requirement: the link is assumed available all the time. So, at minimum, the two grounds need to be connected, and each Transmit Data pin is connected to the other's Receive Data pin.

However, the remote computer is a limiting factor: ''it'' may not be ready to receive. Thus the above signal definitions are modified so that the local computer raises RTS to indicate that it is ready to '''''receive''''' rather than send, and that the cable connects RTS to the remote computer's CTS - which still means "Clear To Send".

===== Today's Null Modem =====

This, then, is the current definition of a "null modem" cable. It swaps the relevant pins, and one computer can tell the other to pause transmitting while it catches up:

  GND -------------- GND
  
  TX  ------\/------  TX
  RX  ------/\------  RX
  
  RTS ------\/------ RTS
  CTS ------/\------ CTS

== Protocol ==

Serial Communications breaks the data to be transmitted into smaller pieces, which are sent sequentially over the data line(s). For standard Asynchronous Serial Communications, the pieces are each individual bit of the data. Each bit is converted to a voltage level and held on the line for a fixed time period, before the next bit is transmitted. All the bits are sent sequentially (least significant bit first).

=== Terms ===

Standard Asynchronous Serial Communications has four main parameters that need to be configured. These completely define how a single character is sent - future characters are then sent in the identical manner:

* Baud Rate
* Data Length
* Parity
* Stop Bits

==== Baud Rate ====

Before any data can be sent, a pre-arranged rate of transitions needs to be defined. To transmit four 0 consecutive bits, there would be no transition between bits, so it is unclear on the receiving side whether that's a fast 16 bits of zero, or a slow single bit of 0. The "baud rate" or "bit rate" is how long a bit is transmitted for before the next bit is started. Thus both sides know the period of time for one bit.

==== Start Bit ====

This is the bit that is transmitted before the first data bit is sent, to take the Transmit Data line off Idle and warn the receiver that data is about to arrive. This is ''not'' a configurable parameter - only one Start Bit is ever sent.

==== Data Length ====

Both sides need to know how many bits are in each character - the data length. If the link is being used to send binary data, then each "character" would be an eight-bit byte. If it's just (English) text data, then seven-bit ASCII would be sufficient. Old-fashioned teletypes didn't use ASCII, they used an older encoding known as "Baudot" - a five-bit-per-character protocol.

==== Parity ====

To help detect transmission errors, after transmitting the agreed number of data characters, the transmitter ''may'' send an extra bit called a parity bit. If it does, it would be agreed whether this bit indicated an Even Parity or an Odd parity. The transmitter's parity bit would be a '0' or a '1' to make Even or Odd the number of '1' bits already sent in the data. The receiver would look at the received parity bit, and if it ''didn't'' match the expected value, it would know that there was a corruption in transit.

This technique is not perfect: a lot of interference with multiple corruptions per character could coincidentally give the same parity result.

==== Stop Bits ====

After the Data and any Parity bits, the transmitter needs to let the line state go back to Idle for a defined time before starting the next character. This time period allows the receiver to re-synchronise its clock to the transmitter's clock every character. In older equipment, it also gave time to for the receiver to actually print the received character! This is why some equipment required two Stop Bits between characters: to give the print head time to print the character. As equipment improved, two bits wasn't needed any more - but one bit was still too quick. So "modern" equipment allowed one-and-a-half Stop Bits to be specified, improving the number of characters per second that the equipment could print. (Remember: in serial communications, a bit is a unit of time. Otherwise "half a bit" would be a nonsense term...)

=== Data Signalling ===

The standard Asynchronous Serial Protocol has to cover a multitude of considerations:
* Not be continually transmitting data;
* Signal that data is about to be sent;
* Signal the end of data;
* Account for the fact that the clocks in different devices are not synchronised;
* Aid in electrical noise reduction or detection.

With the above in mind, actually transmitting the 1s and 0s of the data is done by setting the line HIGH or LOW in whatever electrical standard is being used (RS-232 has HIGH as negative voltages, and LOW as positive voltages).

Thus the default Idle state of the line is HIGH. This indicates that something is connected, but no data is being sent.

=== Example ===

For the following example, assume the link has been configured for the following:

* 9,600 bits per second baud rate;
* Eight data bits per character;
* Even parity;
* Two stop bits

And that the Transmitter wants to send an ASCII capital 'S'. In ASCII binary, that's 0b01010011. Because there's an even number of '1' bits in this character, an Even Parity bit would be '0'.

==== Transmission ====

# The TD signal is already at Idle HIGH ('1').
# The 'S' is loaded into the Transmit register.
# The COM port needs to send a single Start bit to alert the Receiver that data is coming.
# A Start ('0') bit is sent for one bit time.
# By convention, the Least Significant bit ('1') is sent first, for one bit time.
# For seven more bit times, the successive bits are sent: '1', '0', '0', '1', '0', '1', '0'.
# Now the transmitter ''may'' send a parity bit. Since 'Even parity' is defined, the '0' is sent for one bit time.
# Finally, the transmitter needs to put the line back to Idle HIGH ('1') for two stop bit times.

==== Reception ====

# The receiver is receiving an Idle HIGH.
# Suddenly, it notes that the line goes LOW. It immediately starts a '''''half''''' bit time delay.
# At the end of that, it re-samples the line:
## If it was no longer LOW, it was a noise spike, so the receiver goes back to Idle.
## Because it is still LOW, the receiver now waits a '''''full''''' bit time.
# It samples the least significant bit as a '1', on the (hopefully) middle of the first data bit's time.
# It then waits seven further full bit times, sampling the data line each time for the remaining data bits.
# It then waits another bit time to sample the parity bit as '0'.
# It then waits a final bit time to sample the stop bit as a '1'.
# Because the stop bit is correct, and the parity matches the expected value, a "valid" character has been received.
# The receiver immediately starts waiting for another start bit.

Remember, at this point the receiver is still only halfway through receiving the first stop bit. But this mechanism allows minor differences between the transmitter's and receiver's clocks to be accommodated, since the receiver resynchronises its clock on the next Start bit.

More sophisticated COM chips may oversample each bit multiple times, choosing the most common state as the bit's state.

=== The BREAK Signal ===

If the transmitter obeys the above protocol, and the receiver has the same settings, then the only communications possible between the transmitter and receiver is data. If the transmitter wants to indicate anything extra to the receiver, outside of the data - for example a Terminal Reset or other signal - it would have to break protocol.

The transmitter could break protocol by sending a bad Parity character - if Parity was being sent. But the only sure flag that the receiver is looking for is the Stop Bit. If the transmitter wants to indicate something extra, it can send a BREAK signal. This is marked as a '0' stop bit - the receiver would detect this as a Frame Error in the received character. If the received character was also all '0's, then this would indicate that a BREAK signal had been sent, and the receiver could do the indicated action: clear the screen; go back to communication defaults; whatever.

[[Category:Hardware Interfaces]]
````

<a id="w002845"></a>

## W002845 — Categorized Main Page

Bron-ID: W002845
Namespace: Main (0)
Revisie: 29148 | Datum: 2024-06-24T04:50:14Z
Bron: https://wiki.osdev.org/Categorized_Main_Page
Revisie-URL: https://wiki.osdev.org/index.php?title=Categorized_Main_Page&oldid=29148

````text
<div id="mainpage"></div> __NOTOC__ __NOEDITSECTION__

<!-- Welcome to OSDev.org box -->
{{Welcome}}
{{Stylenav}}

{{Flex columns
|1 = 
{{Box|subject= OS Development}}
{{Box|subject= Resources}}
|2 = 
{{Box|subject= Languages}}
{{Box|subject= Tools}}
|3 = 
{{Box|subject= Hardware}}
|4 = 
{{Box|subject= OS theory}}
}}
````

<a id="w002846"></a>

## W002846 — Category:Main Page Templates

Bron-ID: W002846
Namespace: Category (14)
Revisie: 10170 | Datum: 2010-04-28T20:27:10Z
Bron: https://wiki.osdev.org/Category:Main_Page_Templates
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AMain_Page_Templates&oldid=10170

````text
This category contains all listings found on the [[Expanded Main Page]] and [[Categorized Main Page]]
[[Category:Main Page Topics]]
````

<a id="w002847"></a>

## W002847 — File:Under Construction.png

Bron-ID: W002847
Namespace: File (6)
Revisie: 10200 | Datum: 2010-05-01T14:52:38Z
Bron: https://wiki.osdev.org/File:Under_Construction.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AUnder_Construction.png&oldid=10200

````text
Under construction logo (can be used for templates, e.g. todo).
````

<a id="w002848"></a>

## W002848 — GRUB2

Bron-ID: W002848
Namespace: Main (0)
Revisie: 20683 | Datum: 2017-04-10T17:40:27Z
Bron: https://wiki.osdev.org/GRUB2
Revisie-URL: https://wiki.osdev.org/index.php?title=GRUB2&oldid=20683
Doorverwijzing: GRUB

````text
#REDIRECT [[GRUB]]
````

<a id="w002851"></a>

## W002851 — LEAN

Bron-ID: W002851
Namespace: Main (0)
Revisie: 27486 | Datum: 2022-11-09T22:15:35Z
Bron: https://wiki.osdev.org/LEAN
Revisie-URL: https://wiki.osdev.org/index.php?title=LEAN&oldid=27486

````text
{{Filesystems}}

LEAN is a (recursive) backronym that stands for "Lean yet Effective Allocation and Naming" and is an alternative to the FAT filesystem used by the [http://freedos-32.sourceforge.net/ FreeDOS-32] and [https://www.fysnet.net/fysos.htm FYSOS] projects.

== Information ==
LEAN uses an extent-based allocation algorithm and is designed to use the little-endian byte ordering. There have been a total of 8 revisions (at the time of writing) and the changelog for each release can be found in the specification (see link below). LEAN provides the following features:

* Supports Unicode long file names (in contrary to FAT long names, patented by Microsoft).
* Supports extended attributes (name-value metadata, embedded in files or forks).
* Requires only a small memory footprint and isn't too intensive on the CPU.
* Fairly easy to implement and to understand.
* Suitable for both large and small volumes.
* Hard and symbolic links.
* POSIX-compatible.

A more detailed overview of the filesystem's actual capabilities:
* arbitrary sized allocation unit (usually matching the sector size of the media, but not required)
* A maximum volume size of 2^63 - 1 sectors
* A maximum file size of 2^64 - 1 bytes
* File names of up to 4068 bytes (case sensitive and in UTF-8 format)

== Partition Layout ==
To improve locality in space and to help dynamically change the size of a volume, the disk is logically subdivided into bands, made up of several contiguous blocks. Bands are equally sized, perhaps except the last one, and must be made up of a power of two worth of blocks. A volume contains at least one band. Bands are numbered starting from 0, where block 0 is the first block of band 0.

Each band contains a portion of the bitmap representing the allocation state of the blocks of that band. That is, each bitmap portion represents the state of closely located blocks. The bitmap portion for each band must start at the first block of the band. Band 0 is an exception in that it contains blocks reserved for the boot loader and the superblock, thus a specific field is used to tell where the bitmap of band 0 starts.  See the image below.

https://www.fysnet.net/leanfs/lean_layout.png

== Superblock ==
The superblock is a structure containing fundamental information about a LEAN volume. The superblock must be stored in one of any block in range from 1 to 32, inclusive.  The specification specifies how to find this superblock.

{| class="wikitable" style="width: 85%;"
|- 
! Member
! Definition
|- 
| style="width: 15%"| uint32_t checksum
| style="width: 85%"| The checksum value for the superblock. All 32-bit dwords in this block or included in the calculation.
|- 
| uint32_t magic
| This must be equal to 0x4E41454C (the 'LEAN' characters in ASCII), and it must be used to identify a valid LEAN file system. It should be used to find the superblock backup in case the primary copy gets lost or damaged.
|- 
| uint16_t fsVersion
| This field identifies the version of the LEAN file system, and it is provided for future development. The high byte identifies the major version number and the low byte the minor version number (for example 0x0110 would mean version 1.16). At present, in the alpha development stage, it must be set to 0x0008 (that is version 0.8) and drivers must not try to access an unknown file system version, backward compatibility making no sense.
|- 
| uint8_t preallocCount
| The count minus one of contiguous blocks that the driver should try to preallocate when a new block is to be added to a file. A value of zero means that the driver should append a single block. Values greater than zero are recommended, especially for slowly-growing files such as directories, to reduce fragmentation and improve locality.
|- 
| uint8_t logBlocksPerBand
| The base-2 logarithm of the number of blocks per band. The latter is easily computed as 1 << logBlocksPerBand. For 512-byte blocks, it must be at least equal to 12 (4096 bits per band, that is 512 bytes of a block), so that a bitmap portion is always made up of an integral amount of blocks. This field must be equal to 13 for 1024-byte blocks, 14 for 2048-byte blocks, 15 for 4096-byte blocks, and so on. This means, when using 512-byte blocks, the smallest band size is 4096 sectors, that is 2 MiB.
|- 
| uint32_t state
| This field identifies the state of the file system.
Bit 0 (the LSB) is the clean unmount bit: it must be set to '0' while the volume is mounted, and it must set '1' while the volume is unmounted. Thus, if a driver finds this bit set to '0' when mounting a volume, it means the volume was not unmounted properly.
Bit 1 is the error bit: it must be normally '0'. Drivers must set it to '1' in case of major errors, like when file system corruption is detected. File system checkup programs may use this bit to know if some major problem happened with the volume.
Bits 2 to 31 are reserved for future use.
|- 
| uint8_t uuid[16]
| A 128-bit universally unique identifier (UUID) that should uniquely identify the volume. A driver should use it to check for media change in a removable media drive. A driver may use it for automatic volume identification. Drivers must create a sequence of 16 bytes as unique as possible, for example using a good random number generator or one of the canonical algorithms for UUID generation.
|- 
| char volumeLabel[64]
| A descriptive string, encoded in Unicode UTF-8, null-terminated, to help the user identify the volume. It may be empty.
|- 
| uint64_t blockCount
| The total number of blocks that form the volume.
|- 
| uint64_t freeBlockCount
| The number of free (or available, or unallocated) blocks in the volume. A driver must keep it consistent to the rest of the file system to measure the free volume space to store files, directories and book keeping. A value of zero must mean disk full. Drivers may choose to not update this field every time there is a change if performance is a constraint or the disk has limited write capabilities (e.g. flash memories). However, this field must be updated when dismounting the volume.
|- 
| uint64_t primarySuper
| The address of the block containing the primary copy of the superblock. A driver must use this number to know where to write changes made to the superblock back to disk. This field is provided for robustness, for example against repair utilities operating on a disk containing a LEAN file system image file.
|- 
| uint64_t backupSuper
| The address of the block containing the backup copy of the superblock. A driver must use this number to know where to write the backup copy when changes are made to the superblock.
In the event the primary copy of the superblock gets lost or damaged, recovery utilities should search for the backup copy of the superblock looking for valid magic and checksum fields, and may also use the backupSuper field itself for further validation.
|- 
| uint64_t bitmapStart
| The address of the block where the bitmap of the first band (that is, band 0) starts. This is, usually but not mandated, the block right after the superblock. For any other band, the bitmap must start in the first block of the band. The size of the bitmap in each band is related to the logBlocksPerBand field.
|- 
| uint64_t rootInode
| The inode number (that is, the address of the first block) of the root directory. This is, usually but not mandated, the block right after the bitmap for band 0.
|- 
| uint64_t badInode
| The inode number (that is, the address of the first block) of a pseudo-file to track bad blocks. This pseudo-file should be generated and managed by a file system checkup program, and should be made up of any blocks that the program detected as bad. This ensures that a driver can not allocate them. If the volume has no bad blocks, badInode must be zero. See a later section for the format of this pseudo-file.
|- 
| uint64_t journalInode
| The inode number of the "file" representing the Journal data. This field may be zero indicating no Journal. An implementation must verify the Journal Data before using it. It is outside this specification on how this Journal is to be formatted/used. See a later section for more information.
|- 
| uint8_t logBlockSize
| The base-2 logarithm of the size of a block in bytes when this volume was formatted. A value of 9 is 512, 10 = 1024, 11 = 2048, 12 = 4096, and so on. It is calculated as 1 << logBlockSize. This value must be at least 8.
|- 
| uint8_t reserved0[7]
| Padding, reserved for future use. See the definition for Reserved
|- 
| uint8_t reserved1[344]<sup>(1)</sup>
| Reserved for future use. These bytes pad the size of the superblock structure to the size of the block used and must be included in checksum calculation. A value of 344 is used for 512-byte blocks. See the definition for Reserved.
<sup>(1)</sup> 344 for 512-byte blocks, 865 for 1024-byte blocks, 1880 for 2048-byte blocks, 3928 for 4096-byte blocks, etc.
|}

== Bitmap ==
The complete bitmap is a non-contiguous data structure, made up of blocks that, as a whole, form an array of bits, each of them indicating the allocated/unallocated state of each block of a volume. The bitmap is not contiguous because its blocks are subdivided into bands, instead of being stored in a single chunk in a fixed location.  Each Band has its own bitmap.

== Inode ==
All fundamental metadata of a file are stored in a structure called inode structure. Such file metadata include file size, attributes and time stamps, as well as the means to access directly- and indirectly-addressable extents of a file.

{| class="wikitable" style="width: 85%;"
|- 
! Member
! Definition
|- 
| style="width: 20%"| uint32_t checksum
| style="width: 80%"| The checksum value for the inode structure. Any optional inline extended attributes or file data following the inode structure are not part of the inode structure itself, thus must not be included in checksum calculation.
|- 
| uint32_t magic
| This must be equal to 0x45444F4E (the 'NODE' characters in ASCII).
|- 
| uint8_t extentCount
| The number of valid extent specifications stored in the inode structure, that is the count of the first elements in the extentStarts and extentSizes arrays that refer to actual file extents, from 1 (the first extent contains the inode structure itself) to extentsPerInode = 6. Directly-addressable extents must be used from the first to the last before using indirect blocks. If extentCount < extentsPerInode, unused extent specification are undefined. If extentCount < extentsPerInode and any of indirectCount, firstIndirect, or lastIndirect is not zero, this inode is considered corrupt.
|- 
| uint8_t reserved[3]
| Padding, reserved for future use. See the definition for Reserved.
|- 
| uint32_t indirectCount
| The number of indirect blocks owned by the file.
|- 
| uint32_t linkCount
| If this file is not a fork, this field must be equal to the number of hard links (the count of directory entries) referring to this file. If this file is a fork, this field must be equal to the number of non-fork files referring to this fork. Drivers may implement fork sharing and copy-on-write. In any case, for non-deleted files this field must be at least 1.
|- 
| uint32_t uid
| The numeric identifier of the user that owns the file. Interpretation of this field is operating system dependent. A mono-user driver must use zero.
|- 
| uint32_t gid
| The numeric identifier of the group that owns of the file. Interpretation of this field is operating system dependent. A mono-user driver must use zero.
|- 
| uint32_t attributes
| The attributes mask for the file, see enum InodeAttributes. It includes POSIX permissions, behavior flags and the file type specification. Interpretation of permissions is operating system dependent. A mono-user driver must use the owner user permission bits only (ia?USR). Note that the "file type" values are mutually exclusive enumerated values, they are not single bits that can be combined. The iaFmtMask mask can be used to retrieve the file type.
|- 
| uint64_t fileSize
| The size in bytes of the file data, thus excluding the size of the inode structure, the optional extra space for inline extended attributes, and indirect blocks.
|- 
| uint64_t blockCount
| The number of data blocks (thus excluding indirect blocks, and including the first block with the inode structure) allocated for the file. Must always be at least 1. A driver may preallocate more blocks than required for a specified fileSize to implement strategies to reduce fragmentation.
|- 
| int64_t accessTime
| Last access time stamp. This field is computed as the signed number of microseconds since 1970-01-01 00:00:00.000000 UTC, not including leap seconds. Drivers may choose to not update this field if performance is a constraint or the disk has limited write capabilities (e.g. flash memories). A mount-time flag or an open-time flag may be implemented to disable last access time stamps, and the implementation may want this to be the default. Initially it must have the same value as creationTime. If the operating system semantics allows it, drivers may delay update of this time stamp instead of updating it at every access.
|- 
| int64_t statusChangeTime
| Status modification time stamp. This field is computed as the signed number of microseconds since 1970-01-01 00:00:00.000000 UTC, not including leap seconds. Initially it must have the same value as creationTime. If the operating system semantics allows it, drivers may delay update of this time stamp instead of updating it at every modification to the inode structure.
|- 
| int64_t modificationTime
| Last data modification time stamp. This field is computed as the signed number of microseconds since 1970-01-01 00:00:00.000000 UTC, not including leap seconds. Initially it must have the same value as creationTime. If the operating system semantics allows it, drivers may delay update of this time stamp instead of updating it at every write to the file.
|- 
| int64_t creationTime
| File creation time stamp. This field is computed as the signed number of microseconds since 1970-01-01 00:00:00.000000 UTC, not including leap seconds. Depending on the operating system semantics, this field may be greater than the other time stamps if this file is created as a copy of an existing file.
|- 
| uint64_t firstIndirect
| The address of the first indirect block of the file. If the file has no indirect blocks the value of this field must be zero. Example: If there is only 1 indirect block currently being used, this will have the same value as lastIndirect.
|- 
| uint64_t lastIndirect
| The address of the last indirect block of the file. If the file has no indirect blocks the value of this field must be zero. Example: If there is only 1 indirect block currently being used, this will have the same value as firstIndirect.
|- 
| uint64_t fork
| If this file is not a fork, this field is the address of the first block of the optional fork associated with this file, or zero if this file has not a fork. If this file is a fork, this field must be zero.
|- 
| uint64_t extentStarts[extentsPerInode]
| The array of starting block addresses of the extent specifications. The count of valid extent specifications, starting from index 0, is determined by the extentCount field. Any unused extent specification is unspecified. Note that the first element of this array is also -by definition- the address of the block containing this inode structure.
|- 
| uint32_t extentSizes[extentsPerInode]
| The array of extent sizes of the extent specifications. Each element represents the size in blocks of the corresponding extent whose first block is specified in extentStarts.
|}

== Indirect Addressing ==
If a file needs more than extentsPerInode extents, indirect addressing is used for subsequent extents. An indirect block is a block storing an array of extentsPerIndirect extent specifications instead of file data. Indirect blocks are linked together to form a doubly-linked list in order to support arbitrary file lengths.

{| class="wikitable" style="width: 85%;"
|- 
! Member
! Definition
|- 
| style="width: 20%"| uint32_t checksum
| style="width: 80%"| The checksum value for the indirect block. All bytes in the block do count for checksum calculation.
|- 
| uint32_t magic
| This must be equal to 0x58444E49 (the 'INDX' characters in ASCII).
|- 
| uint64_t blockCount
| The total number of blocks addressable using this indirect block. A driver should use this information to quickly traverse the list of indirect blocks to find the desired data block. It must be equal to the sum of the used extentSizes (see below).
|- 
| uint64_t inode
| The inode number of the file this indirect block belongs to. This is provided for robustness.
|- 
| uint64_t thisblock
| The address of the block storing this indirect block. This is provided for robustness.
|- 
| uint64_t prevIndirect
| The address of the previous indirect block. If there is no previous indirect block, this field must be zero. A driver may use this value to traverse the list of indirect blocks in reverse order.
|- 
| uint64_t nextIndirect
| The address of the next indirect block. If there is no next indirect block, this field must be zero. This is the primary means for a driver to traverse the list of indirect blocks to seek into a file.
|- 
| uint16_t extentCount
| The number of valid extent specifications stored in the indirect block, that is the count of the first elements in the extentStarts and extentSizes arrays that refer to actual file extents. If the indirect block is not the last indirect block of a file, all extent specifications must be in use, that is extentCount must be equal to extentsPerIndirect.
|- 
| uint8_t reserved0[2]
| Padding, reserved for future use. See the definition for Reserved.
|- 
| uint32_t reserved1
| Padding, reserved for future use. See the definition for Reserved.
|- 
| uint64_t extentStarts[extentsPerIndirect]
| The array of starting block addresses of the extent specifications.
|- 
| uint32_t extentSizes[extentsPerIndirect]
| The array of extent sizes of the extent specifications. Each element represents the size in blocks of the corresponding extent whose first block is specified in extentStarts.
|- 
| uint8_t reserved2[n]
| Zero to eleven bytes. A 512-byte block will have zero bytes here. A 4096-byte block will have 8 bytes here. The value of these bytes are included in the checksum calculation. See the definition for Reserved.
|}

== Tools ==
The creators of LEAN have a reference implementation and binaries of a LEAN file manager (Windows/Debian/Linux) available on their website. See [http://www.fysnet.net/leanfs/downloads.php this page] for more information.

== More Information ==
The specification, linked below, details these structures and their members in more detail, as well as information on other aspects of the file system.

== See Also ==
=== Threads ===
* [[Topic:41924|About LeanFS]]
* [[Topic:37558|LeanFS and 4,096-bytes sectors]]

=== External Links ===
* [https://www.fysnet.net/leanfs/index.php LEAN Overview 0.8] ([http://freedos-32.sourceforge.net/lean/ LEAN Overview 0.6])
* [https://www.fysnet.net/leanfs/specification.php LEAN 0.8 Specification] ([http://freedos-32.sourceforge.net/lean/specification.php LEAN 0.6 Specification])

[[Category:Filesystems]]
````

<a id="w002852"></a>

## W002852 — Talk:Main Page/archive

Bron-ID: W002852
Namespace: Talk (1)
Revisie: 10231 | Datum: 2010-05-04T16:27:27Z
Bron: https://wiki.osdev.org/Talk:Main_Page/archive
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMain_Page%2Farchive&oldid=10231
Doorverwijzing: Talk:Categorized Main Page/archive

````text
#REDIRECT [[Talk:Categorized Main Page/archive]]
````

<a id="w002853"></a>

## W002853 — User talk:Solar/NewIntro

Bron-ID: W002853
Namespace: User talk (3)
Revisie: 10270 | Datum: 2010-05-06T14:28:11Z
Bron: https://wiki.osdev.org/User_talk:Solar/NewIntro
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ASolar%2FNewIntro&oldid=10270

````text
Regarding "''Rewrite the Introduction / Beginner Mistakes / ... part of the Wiki as a generic text, as friendly as can be, that details that OS development is simply the wrong place to learn about a programming language, a processor architecture, or technical documentation.''"

Back when I started programming, compilers/optimisers just weren't very good and CPUs were seriously limited (especially for hardware that a single person could actually afford), so assembly language was the only real choice for any serious project. Heck - the first assembler I ever used came in a magazine (back when magazines had code listings - it was common then, but now it's unheard of).

So, what happened? Times changed. Most programmers shifted to higher level languages like C and Pascal.

Time didn't stand still though. Optimisers got a lot better, and CPUs got much much faster, and everything got a lot more complicated. Programmers started using even higher level languages and lots of libraries to avoid dealing with this extra complexity. Now most programmers are used to things like garbage collection, run-time bounds checking and lots of scripting/interpreted languages. These are all things that would've been considered suicidal/idiotic back when I was a beginner. Even programming in older languages like C has changed a lot - now it's gluing together libraries written by other people, and most universities don't even bother teaching it (most shifted to Java years ago).

Obviously, now there's a huge difference between "system programming" and "application programming" that didn't exist back then. Over time this difference will increase further. At some point in time no applications programming will be done using any language that is even remotely suitable for system programming. At some point in time, expecting people to learn system programming by writing applications will be absurd - regardless of how much "gluing together libraries with Java" someone does they'll never learn anything that's actually useful for OS development.

Maybe OS development isn't the wrong place to learn a system programming language like C or assembly. Maybe OS development is the *only* place a someone can really learn a system programming language like C or assembly (or a processor architecture, or related technical documentation).

Instead of telling beginners "we won't help you learn system programming until you've already learnt system programming" we should be telling beginners "come here to learn".

-Brendan
````

<a id="w002854"></a>

## W002854 — Talk:Categorized Main Page

Bron-ID: W002854
Namespace: Talk (1)
Revisie: 10233 | Datum: 2010-05-04T16:32:33Z
Bron: https://wiki.osdev.org/Talk:Categorized_Main_Page
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ACategorized_Main_Page&oldid=10233

````text
An archive of this page exists at ''[[Talk:Categorized_Main_Page/archive]]''. For discussing the front page redirect, see ''[[Talk:Main Page]]''
----
````

<a id="w002855"></a>

## W002855 — Talk:Expanded Main Page

Bron-ID: W002855
Namespace: Talk (1)
Revisie: 12711 | Datum: 2012-02-19T20:15:39Z
Bron: https://wiki.osdev.org/Talk:Expanded_Main_Page
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AExpanded_Main_Page&oldid=12711

````text
You can edit this page: go to ''[[:Category:Main Page Templates]]'' where the actual content is found. If you still believe you need to edit ''this'' page you can post a short explanation either here or in the forums and staff will suspend the protection.

This page, and the categorized main page I feel should be placed in the Wiki administration category.
--[[User:Bellezzasolo|Bellezzasolo]] 14:15, 19 February 2012 (CST)

For discussing the front page redirect, see ''[[Talk:Main Page]]''. 
----
````

<a id="w002856"></a>

## W002856 — Talk:Setting Up Long Mode

Bron-ID: W002856
Namespace: Talk (1)
Revisie: 29581 | Datum: 2025-07-05T15:20:29Z
Bron: https://wiki.osdev.org/Talk:Setting_Up_Long_Mode
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ASetting_Up_Long_Mode&oldid=29581

````text
== Seperate Page ==

Should we move this to its own page, instead of being part of your user page?
[[User:Renagadexx|Jack]] 19:26, 22 October 2011 (UTC)

I wouldn't mind that happening, at least, if nobody disagrees with that happening.
--[[User:Stephanvanschaik|Stephanvanschaik]] 16:03, 29 October 2011 (UTC)

== segment descriptor ==

Does read bit in long-mode code segment do something?

10011010b ; Access (exec/read)

Maybe cleaner variant is:

10011000b

I think same thing work with write bit in data segment

== Add to expanded main page ==

The GDT link on the expanded main page also has a link to a GDT tutorial. Could this page benefit from being easier to get to from the main page in a similar way since it gives much more detail than the current x86_64 page that is already linked to.
````

<a id="w002857"></a>

## W002857 — Talk:Cross-Compiler Successful Builds

Bron-ID: W002857
Namespace: Talk (1)
Revisie: 24162 | Datum: 2019-12-19T11:31:56Z
Bron: https://wiki.osdev.org/Talk:Cross-Compiler_Successful_Builds
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ACross-Compiler_Successful_Builds&oldid=24162

````text
== GCC 4.5.0 ==

User Bluechill marked GCC 4.5.0 / Binutils 2.20 as "Yes / Successful" in the cross-compiler tutorial.

I just tried it (Linux Mint 7 / Gloria)... it requires the MPC library, which is not covered by the tutorial. And even with the library installed, I get a MPC-related error:

<pre>
make[2]: Leaving directory '/home/solar/Projekte/propos/build-gcc/gcc'
Checking multilib configuration for libgcc...
mkdir -p -- i586-elf/libgcc
Configuring in i586-elf/libgcc
configure: creating cache ./config.cache
checking for --enable-version-specific-runtime-libs... no
checking for a BSD-compatible install... /usr/bin/install -c
checking for gawk... gawk
checking build system type... i686-pc-linux-gnu
checking host system type... i586-pc-elf
checking for i586-elf-ar... /usr/cross/i586-elf/bin/ar
checking for i586-elf-lipo... i586-elf-lipo
checking for i586-elf-nm... /home/solar/Projekte/propos/build-gcc/./gcc/nm
checking for i586-elf-ranlib... /usr/cross/i586-elf/bin/ranlib
checking for i586-elf-strip... /usr/cross/i586-elf/bin/strip
checking whether ln -s works... yes
checking for i586-elf-gcc... /home/solar/Projekte/propos/build-gcc/./gcc/xgcc -B/home/solar/Projekte/propos/build-gcc/./gcc/ -B/usr/cross/i586-elf/bin/ -B/usr/cross/i586-elf/lib/ -isystem /usr/cross/i586-elf/include -isystem /usr/cross/i586-elf/sys-include   
checking for suffix of object files... configure: error: in `/home/solar/Projekte/propos/build-gcc/i586-elf/libgcc':
configure: error: cannot compute suffix of object files: cannot compile
See `config.log' for more details.
make[1]: *** [configure-target-libgcc] Fehler 1
make[1]: Leaving directory '/home/solar/Projekte/propos/build-gcc'
make: *** [all] Error 2
</pre>

And from i586-elf/libgcc/config.log:

<pre>
configure:3233: /home/solar/Projekte/propos/build-gcc/./gcc/xgcc -B/home/solar/Projekte/propos/build-gcc/./gcc/ -B/usr/cross/i586-elf/bin/ -B/usr/cross/i586-elf/lib/ -isystem /usr/cross/i586-elf/include -isystem /usr/cross/i586-elf/sys-include    -c -g -O2  conftest.c >&5
/home/solar/Projekte/propos/build-gcc/./gcc/cc1: error while loading shared libraries: libmpc.so.2: cannot open shared object file: No such file or directory
configure:3237: $? = 1
configure: failed program was:
| /* confdefs.h */
| #define PACKAGE_NAME "GNU C Runtime Library"
| #define PACKAGE_TARNAME "libgcc"
| #define PACKAGE_VERSION "1.0"
| #define PACKAGE_STRING "GNU C Runtime Library 1.0"
| #define PACKAGE_BUGREPORT ""
| #define PACKAGE_URL "http://www.gnu.org/software/libgcc/"
| /* end confdefs.h.  */
|
| int
| main ()
| {
|
|   ;
|   return 0;
| }
configure:3251: error: in `/home/solar/Projekte/propos/build-gcc/i586-elf/libgcc':
configure:3254: error: cannot compute suffix of object files: cannot compile
See `config.log' for more details.
</pre>

The configure line:

<pre>
$ ../gcc-4.5.0/configure --target=$TARGET --prefix=$PREFIX --disable-nls \
      --enable-languages=c,c++ --without-headers --with-gmp=/usr/cross \
      --with-mpfr=/usr/cross --with-mpc=/usr/cross
</pre>

And, just to make sure it's not the library being missing:

<pre>
$ ls /usr/cross/lib/
libgmp.a   libgmp.so	 libgmp.so.10.0.1  libmpc.a   libmpc.so    libmpc.so.2.0.0  libmpfr.la	libmpfr.so.1
libgmp.la  libgmp.so.10  libiberty.a	   libmpc.la  libmpc.so.2  libmpfr.a	    libmpfr.so	libmpfr.so.1.2.2
</pre>

Seems like we need some different steps / options for the 4.5.0 build, or perhaps it's actually broken; as long the steps and options presented in the tutorial don't work, I remove the 4.5.0 / 2.20 "successful" tag. Perhaps user Bluechill can shed some light on what he did to make it work.

:I had no trouble building GCC 4.5 with the latest MPC version. You follow exactly the same steps as in MPFR/GMP (--prefix=/usr/local), except that you need to compile GCC specifying --with-mpc=/usr/local or something similar. Have you tried using /usr/local as prefix? Also note that MPC depends on GMP and MPFR, so you need both of those to be built (possibly specifying them while configuring MPC) before you can build MPC properly. --[[User:Creature|Creature]] 10:41, 5 May 2010 (UTC)

::I used /usr/cross consistently because I don't like mixing hand-installed stuff with what's been installed by my system's packet manager. I know I'm the first to cry "follow it to the letter" usually, but that's the only thing where I strayed from the letter, and the tutorial ''should'' be independent of where you put the code. You can see both that I used --with-mpc=/usr/cross and that /usr/cross/lib actually contains GMP, MPFR and MPC, from the information I pasted above.
::I'll double-check this tonight. -- [[User:Solar|Solar]] 13:48, 5 May 2010 (UTC)

:That kindof confirms that the tutorial alone is incomplete. Can't we just stick to recommending 4.2.x? That's so much less troublesome than having to get all GCC's dependencies. At this rate it seems we need a hand install of a full linux package set in a years time. '''Try it! cross-compiles were never this easy(!)''' - [[User:Combuster|Combuster]] 11:38, 5 May 2010 (UTC)

::I admit that idea was tempting for some time, but it'd get stale pretty fast, and people would opt for the latest release version anyway, ignoring the tutorial (''and'' returning with their questions as to why it didn't work). That being said, I'm pretty annoyed by how GCC seems to be collecting non-standard dependencies with each version. -- [[User:Solar|Solar]] 13:48, 5 May 2010 (UTC)

:Maybe creating a separate page with a small script to make and install GMP, MPFR and MPC. I can't say I have this problem as I install Cygwin complete on my system and have all the devel libs --[[User:Kieran|Kieran]] 15:31, 8 August 2011 (UTC)

:I've come across this error when trying to automate my toolchain build process, you NEED to have binutils in your path and gcc and binutils need to be compiled using the same prefix, i'm just leaving this for whoever might run into this error -- [[User:Bauen1|Bauen1]] 02:48, 29 December 2016 (CST)

== Table re-shuffle ==
I've been thinking about the table orientation. It seems wrong. On the horizontal, we've put the program with the fast-growing version numbers, and on the vertical, binutils, which is updated once in a blue moon. This has caused people to split the tables up so they don't go off the right of the brower screen.

I propose flipping GCC to the vertical and binutils to the horizontal, then merging all the tables. What do you all think? [[User:Phillid|Phillid]] 01:02, 16 November 2014 (CST)

* Looks like it's been done, and now we've got binutils going off the right of the screen. :) Perhaps untried combinations could be pruned. [[User:Eekee|Eekee]] 05:31, 19 December 2019 (CST)
````

<a id="w002859"></a>

## W002859 — Talk:Solar/NewIntro

Bron-ID: W002859
Namespace: Talk (1)
Revisie: 10271 | Datum: 2010-05-06T14:28:11Z
Bron: https://wiki.osdev.org/Talk:Solar/NewIntro
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ASolar%2FNewIntro&oldid=10271
Doorverwijzing: User talk:Solar/NewIntro

````text
#REDIRECT [[User talk:Solar/NewIntro]]
````

<a id="w002860"></a>

## W002860 — File:Stub.png

Bron-ID: W002860
Namespace: File (6)
Revisie: 10278 | Datum: 2010-05-07T18:39:38Z
Bron: https://wiki.osdev.org/File:Stub.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AStub.png&oldid=10278

````text
Stub icon (doesn't look that great, but I think it looks better and sticks out more than just text).
````

<a id="w002861"></a>

## W002861 — User:Creature/Template:Main2/Subheader

Bron-ID: W002861
Namespace: User (2)
Revisie: 10284 | Datum: 2010-05-08T09:51:41Z
Bron: https://wiki.osdev.org/User:Creature/Template:Main2/Subheader
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACreature%2FTemplate%3AMain2%2FSubheader&oldid=10284

````text
<div style="margin-left: 20px;">
<small style="text-decoration: underline;">'''{{{Title}}}'''</small>

{{{Contents}}}
</div>
````

<a id="w002862"></a>

## W002862 — User:Creature/Template:Main2/Hardware Experiment

Bron-ID: W002862
Namespace: User (2)
Revisie: 10285 | Datum: 2010-05-08T10:01:51Z
Bron: https://wiki.osdev.org/User:Creature/Template:Main2/Hardware_Experiment
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACreature%2FTemplate%3AMain2%2FHardware_Experiment&oldid=10285

````text
'''CPU''' 
* [[AMD K6 WriteBack Optimisations|AMD K6 writeback optimisations]]
* [[Detecting CPU Speed|How can I detect the CPU's speed?]]
* [[Model Specific Registers|Model-specific registers]]
* [[Inline_Assembly|Inline Assembly]]
* [[X86-64|Information regarding the X86 64-bits CPU]]
* [[IA32 Architecture Family|Processors of the IA32 architecture family]]
* [[Virtual 8086 Mode|What is v8086 mode?]]
* [[CPU Bugs]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:X86 CPU|more...]]</small></div>

'''Memory'''
* [[Detecting Memory (x86)|How do I determine the amount of RAM?]]
* [[A20 Line|Why can't I access all my memory?]]


'''IRQ's & Exceptions'''
* [[Interrupts|Interrupts for dummies]]
* [[PIC|What is the PIC and how do I remap it?]]
* [[APIC|Tell me about the APIC]]
* [[Non Maskable Interrupt|What is the NMI?]]
* [[Interrupt Service Routines]]
* [[I Cant Get Interrupts Working|I can't get interrupts working!]]
* [[PS2 Keyboard|Getting Keyboard Input]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Interrupts|more...]]</small></div>

'''Video'''
{{User:Creature/Template:Main2/Subheader
|Title=Text
|Contents=
* [[Text Mode Cursor|How do I move the cursor when I print]]?
* [[Printing to Screen|How do I output text to the screen (Protected Mode)]]?
}}

{{User:Creature/Template:Main2/Subheader
|Title=Graphics
|Contents=
* [[Accelerated Graphic Cards]]
* [[Detecting Colour and Monochrome Monitors|How do I detect if I have a colour or monochrome monitor]]?
* [[Drawing In Protected Mode|How do I draw things in protected mode?]]
* [[VGA Resources]] and [[VGA Hardware|VGA Documentation]]
* [[Video Signals And Timing]]
* [[BGA|Bochs and QEMU's graphics device]]
}}
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Video|more...]]</small></div>

'''Audio'''
* [[Sound]]
* [[PC Speaker]]
* [[Sound Blaster 16]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Sound|more...]]</small></div>

'''Storage Devices'''
* [[DMA]]
* [[ATAPI|ATAPI (CD-Rom)]]
* [[:Category:ATA|ATA (General)]]
* [[Floppy Disk Controller|Floppy driver]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Storage|more...]]</small></div>

'''Plug & Play'''
* [[Plug-and-Play|Where can I find programming information]]?
* [[PNP Calls In Protected Mode|Can you do PNP calls with the BIOS in Protected Mode]]?


'''PCI & USB'''
* [[PCI]]
* [[AGP]]
* [[USB]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Buses|more...]]</small></div>

'''Network Cards & Communication'''
* [[3c90x]]
* [[Ne2000]]
* [[RTL8139]]
* [[RTL8169]]
* [[Serial ports]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Network devices|more...]]</small></div>

'''Non-x86 Development'''
* [[:Category:MIPS|MIPS]]
* [[:Category:Sparc|Sparc]]
* [[:Category:PowerPC|PowerPC]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Platforms|more...]]</small></div>

<noinclude>[[Category:Main Page Templates]]</noinclude>
````

<a id="w002863"></a>

## W002863 — User:Novayoshi

Bron-ID: W002863
Namespace: User (2)
Revisie: 10298 | Datum: 2010-05-10T20:19:48Z
Bron: https://wiki.osdev.org/User:Novayoshi
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANovayoshi&oldid=10298

````text
NovaYoshi is a programmer who has a special fondness for assembly languages, and is also a furry.
````

<a id="w002864"></a>

## W002864 — Object Files

Bron-ID: W002864
Namespace: Main (0)
Revisie: 19508 | Datum: 2016-07-22T13:15:53Z
Bron: https://wiki.osdev.org/Object_Files
Revisie-URL: https://wiki.osdev.org/index.php?title=Object_Files&oldid=19508

````text
Object files basically consist of compiled and assembled code, data, and all the additional information necessary to make their content usable. In the process of building an operating system, you will use a lot of object files. While for common development tasks you do not need to know their exact details, when you want to create or use one with various specifics, the details can be very important. 

'''Note:''' The term 'object file' has no relation to the high-level concept of 'object-oriented programming'. Object files predate the earliest forms of OOP (the 'Actors model', circa 1966) by over ten years, and the term was in use at IBM by 1958 or earlier.

== Core Concepts ==
An object file is one of three types of files which contain ''object code'', a modified form of machine code in which additional information that allows for linking and relocation of the final loaded executable is included. 

For most purposes, a compiler or assembler will produce object code as its final result, rather than a true executable binary. While most assemblers, and some compilers, have an option to produce a raw binary image, this is usually only applied to boot loaders, Read-Only Memory chips, and other special-purpose executables. As a practical matter, virtually all systems today use both object files and relocatable executables. Even the simplest file format in regular use today, the [https://en.wikipedia.org/wiki/COM_file MS-DOS .COM format], is not a pure binary executable image; the MS-DOS loader uses the first 0x100 bytes of the segment for the [https://en.wikipedia.org/wiki/Program_Segment_Prefix Program Segment Prefix], so that portion of the segment image is excluded in the file. 

The majority of systems today have significantly more complex object formats, in which the address information is replaced with a stub or symbol of some kind, and which contain information regarding the relative location of externally visible functions, variables, and so forth. This facilitates the processes of 
* linking, in which two or more object and/or library files are combined to form an executable file, and 
* loading, where the address stubs are replaced by the loader with the actual memory locations that the code will reside within the process' memory.

== Object files, executable files and library files ==
Whereas Wikipedia considers executables to be a subset of object files, on the basis that both contain object code rather than a binary image, there are significant differences. In some systems, they are a completely different format (COFF vs PE), or they have different fields (ELF program/section headers). The key difference is that in executables, the full object code of the program is present (save for whatever may be in shared libraries, as explained below), while object files are only the object code for the specific module that they were generated from. This means that non-executable files do not contain loadable code. 

As stated earlier, this does not necessarily mean that the 'executable' file is the actual binary image that is executed; in most modern systems, that is produced in the loading step. In many cases, executable files still contain object code, not pure machine code, and the address locations may not be resolved until loading, but they ''do'' include all of the statically-linked code for the working program. Some linkers (such as [[LD|ld]], the Unix/Linux linker, which is invoked implicitly by GCC when generating an executable) have the option of  - or even default to, as [[LD|ld]] does - resolving the addresses at link time, but even in this case, the executable files generated usually contain additional information to facilitate the loading process - e.g., a separate read-only data section, a definition of the writable data area (sometimes called the ''[https://en.wikipedia.org/wiki/.bss .bss section]''), a section defining the stack area, etc. - and may have linkage information for using shared libraries.

A third type of object code file is a ''library file'', a file that contains elements used by several programs, and made available for general use. Most functions, variables, and other elements used by the majority of programs are held in libraries. Libraries differ from regular object files mainly in that (on most systems) they are arranged so that independent elements of the library can be extracted from the file by the linker, so that only the elements used by the program are included in the executable generated from them.

On most systems today, libraries come in two types, ''static libraries'', which are directly linked into into the executable file at linkage time, and ''shared libraries'' (also called ''dynamic-link libraries'' or DLLs in the Windows world), which are loaded and linked at run time to the programs which use them. The main difference is that shared libraries, as the name implies, can be shared by several programs at once, lowering memory usage. However, this comes at the cost of having to load the shared library in addition to the executable file when the first use of an element in them is made, and then linking it to the program(s) using them at run-time. Shared libraries are generally cached, to reduce the loading overhead, and usually are not loaded until the elements in them are actually used, meaning that if the part of the program using the shared library is not invoked, the library need not be loaded at all. Still, the trade-offs are such that code which isn't likely to be shared by several programs at once generally is linked as a static library instead, with only very common elements (e.g., the standard C and C++ libraries) being dynamically linked.

== Relocations ==
A good part of the object file contains the code and its associated data. In source, code contains references to other functions and storage of data. In the object file such references are converted to instructions and relocation pairs, as the compiler can't tell in advance where the code will end up. As an example a function call on an x86 will look like this (in an object file):
 
  14:   e8 fc ff ff ff          call   15 <sprintf+0x15>
                        15: R_386_PC32  vsnprintf 

The disassembly contains the opcode for call (e8) plus the offset -4 (fc ff ff ff). If this were to be executed it would make a call to address 15, which looks like halfway through the instruction. The second line (the relocation entry) lists that the address at position 15 (the -4) should be fixed with a displacement to the address of vsnprintf. That means it should get the address of the called function minus the address of the relocation. However blankly entering the difference would not work, as the call address is relative to the next instruction, not the start of the offset bytes halfway into an opcode. This is where the -4 comes in: the result of the relocation is added to the field being padded. By subtracting 4 (adding -4) from the address, the displacement becomes relative to the end of the instruction, and the call ends up where it should go. In the executable file:

  804a1d4:       e8 07 00 00 00          call   804a1e0 <vsnprintf>
                         '''804a1d5''': R_386_PC32     vsnprintf
  ''804a1d9'':       c9                      leave
 
  (...)
 0804a1e0 <vsnprintf>:

The displacement needed for the call is the address of vsnprintf minus the address of the next instruction, i.e. <tt>0x804a1e0 - ''0x804a1d9'' = 0x7</tt>, which is the value seen in the call bytes (07 00 00 00). This is equivalent to the address of the target minus the address of the relocation plus the value stored: <tt>0x804a1e0 - '''0x804a1d5''' + -4 = 0x7</tt>.

=== Relocating code ===
When an executable is created, it will be set to use a specific address by default. This can be a problem when you need several object files in the same address space and they may overlap, or you want to perform address space randomization, you might find relocating an executable an option.

Since relocations are only needed to build an executable, but not when you run it, they normally aren't present in a linked file. Instead you need to specifically tell the linker to emit relocations when necessary. For the [[GCC Cross-Compiler]], this can be done with the <tt>-q</tt> switch. Note that the <tt>-i</tt> and <tt>-r</tt> switches have a similar description, but cause the linker to yield an object file rather than an executable.

Relocating is of itself fairly straightforward by finding the differences. Start with loading the sections to the location of your choice, then for each relocation entry:
* compute the original address where the relocation was applied
* compute the address where the relocation applies now (its moved by the same amount you moved the original section from its original location)
* do the same for the destination of the relocation
* compute what the relocation value is - the destination for absolute relocations, and the destination minus the origin for relative relocations.
* compute what the relocation value was using the original location.
* subtract the old value from the new value
* add the result to the original relocation value in memory.

If the sections are moved relatively to each other, then relocating can become as simple as only adding the displacement to the absolute relocations. The relative locations do not get changed as both the source and the target are moved by the same amount.

== Common errors ==
* '''Passing -i or -r to ld.''' It does not work except for some limited cases, as it generates a file where relocations have not been applied at all.
* '''Assuming code and data are continuous.''' A pitfall when trying to make a PE file multiboot-compatible. A section is generally page-aligned (4k), but a PE file is sector-aligned (512b). So if a section is not multiple of 4k in size, relative addresses to the data section will be off by a multiple of 512 bytes as the gap has been removed from the binary. Worse, it is perfectly valid to have metadata sections between the various loadable sections, which can put addresses off. 
* '''Loading as a flat binary.''' All executables that aren't flat binaries have a header up front. Blatantly loading a file and starting at the start will execute the header instead of your code. Again, there is a tutorial that tries to get away with this. 
* '''Assuming the entry point is at the start'''. The linker has a certain amount of freedom in what order it loads the object files, and so does the compiler. That means that main doesn't need to be at the start of the code section.

[[Category:Object Files]] [[Category:Theory]]
````

<a id="w002865"></a>

## W002865 — User:Combuster/Object Files

Bron-ID: W002865
Namespace: User (2)
Revisie: 10304 | Datum: 2010-05-14T20:27:12Z
Bron: https://wiki.osdev.org/User:Combuster/Object_Files
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACombuster%2FObject_Files&oldid=10304
Doorverwijzing: Object Files

````text
#REDIRECT [[Object Files]]
````

<a id="w002866"></a>

## W002866 — ARM Integrator-CP Bare Bones

Bron-ID: W002866
Namespace: Main (0)
Revisie: 30231 | Datum: 2026-03-16T15:00:30Z
Bron: https://wiki.osdev.org/ARM_Integrator-CP_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=ARM_Integrator-CP_Bare_Bones&oldid=30231

````text
{{BeginnersWarning}}
{{Rating|1}}
{{Template:Kernel designs}}
''In this tutorial we are going to compile a simple kernel for the ARM Integrator-CP board and boot it.''






==Required Tools==

For this tutorial you'll need:
* A [[GCC Cross-Compiler|cross compiler]] environment for ARM, minimally arm-elf-as, arm-elf-gcc and arm-elf-ld
* QEMU - to test the kernel (Unless of course you have a spare Integrator-CP board hanging around)
You should also have read [[Getting Started]] and [[Beginner Mistakes]]. You should also have a reasonable knowledge of ARM assembly by this point, though in this example there isn't much assembly to understand anyway.

==start.s==

This is the assembly file, most of it isn't that hard to understand.

<pre>
interrupt_vector_table:
    b . @ Reset
    b . 
    b . @ SWI instruction
    b . 
    b .
    b .
    b .
    b .

.comm stack, 0x10000 @ Reserve 64k stack in the BSS
_start:
    .globl _start
    ldr sp, =stack+0x10000 @ Set up the stack
    bl main @ Jump to the main function
1: 
    b 1b @ Halt
</pre>

Assemble with: <syntaxhighlight lang="bash"> arm-elf-as -o start.o start.s </syntaxhighlight>

==main.c==

Now we get to the interesting part. We're going to display some characters via the serial port, those characters being "hello, world!" of course.

<syntaxhighlight lang="c">
#define SERIAL_BASE 0x16000000
#define SERIAL_FLAG_REGISTER 0x18
#define SERIAL_BUFFER_FULL (1 << 5)

void putc (char c)
{
    /* Wait until the serial buffer is empty */
    while (*(volatile unsigned long*)(SERIAL_BASE + SERIAL_FLAG_REGISTER) 
                                       & (SERIAL_BUFFER_FULL));
    /* Put our character, c, into the serial buffer */
    *(volatile unsigned long*)SERIAL_BASE = c;

    /* Print a carriage return if this is a newline, as the cursor's x position will not reset to 0*/
    if (c == '\n')
    {
        putc('\r');
    }
}

void puts (const char * str)
{
    while (*str) putc (*str++);
}

int main (void)
{
    puts ("hello, world!\n");
    return 0;
}
</syntaxhighlight>

Compile this with: <syntaxhighlight lang="bash">arm-elf-gcc -Wall -Wextra -Werror -nostdlib -nostartfiles -ffreestanding -std=gnu99 -c main.c</syntaxhighlight>

==link.ld==

<pre>
ENTRY (_start)

SECTIONS
{
    . = 0;
    .text : { *(.text*) *(.rodata*) }
    .data : { *(.data*) }
    .bss : { *(.bss*) *(COMMON*) }
}
</pre>

Now we link everything using: <syntaxhighlight lang="bash">arm-elf-ld -T link.ld -o kernel start.o main.o</syntaxhighlight>

==Booting==

Boot everything using: <syntaxhighlight lang="bash">qemu-system-arm -m 128 -kernel kernel -serial stdio</syntaxhighlight>

Here is the output:
[[File:Qemuarmhello.png]]

==Links==

*[http://infocenter.arm.com/help/topic/com.arm.doc.dui0159b/DUI0159B_integratorcp_1_0_ug.pdf Integrator/CP Reference Manual]
*[http://infocenter.arm.com/help/topic/com.arm.doc.ddi0183f/DDI0183.pdf UART PL011 (Serial Port) Technical Reference Manual]
*[http://balau82.wordpress.com/2010/02/28/hello-world-for-bare-metal-arm-using-qemu/ Hello world for bare metal ARM using QEMU]
[[Category:ARM]]
[[Category:Bare bones tutorials]]
````

<a id="w002867"></a>

## W002867 — User:Pinged/ARM Bare Bones

Bron-ID: W002867
Namespace: User (2)
Revisie: 10309 | Datum: 2010-05-16T11:26:55Z
Bron: https://wiki.osdev.org/User:Pinged/ARM_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APinged%2FARM_Bare_Bones&oldid=10309
Doorverwijzing: User:Pinged/ARM Integrator-CP Bare Bones

````text
#REDIRECT [[User:Pinged/ARM Integrator-CP Bare Bones]]
````

<a id="w002868"></a>

## W002868 — Partition table

Bron-ID: W002868
Namespace: Main (0)
Revisie: 10323 | Datum: 2010-05-18T17:31:21Z
Bron: https://wiki.osdev.org/Partition_table
Revisie-URL: https://wiki.osdev.org/index.php?title=Partition_table&oldid=10323
Doorverwijzing: Partition Table

````text
#REDIRECT [[Partition_Table]]
````

<a id="w002869"></a>

## W002869 — User:Rudimk

Bron-ID: W002869
Namespace: User (2)
Revisie: 10373 | Datum: 2010-05-27T15:25:47Z
Bron: https://wiki.osdev.org/User:Rudimk
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ARudimk&oldid=10373

````text
Hey!

Although I'm actually a humanities student, programming's always been my second love. Earlier, I was working on writing a managed kernel using the .NET Micro Framework. Sadly....it kinda failed. The biggest pitfall was that the Micro Framework's meant for embedded use...plus, it's only running on the ARM suite of processors. Not exactly ideal when most people you know use an Intel-based processor!


Anyway, right now, I'm *thinking* of embarking on another OSDev project. Haven't really decided the details and all, but yeah. The show's on :D

Right. I think I just decided.
I'm gonna do it in D!!
It's a refreshing change from C. Definitely better than doing it in Sphinx C--.
Yeah..if I could somehow write a kernel in HLA...then nothing like it!!
Anyway, for now, D it is.

So I'm gonna upload more information on this  at the following URL:
[http://codeville.wordpress.com Codeville]
--------------

April 28 2010
--------------
I've been thinking about this a lot...and I've figured something out. Why not link the D kernel to the OSKit libraries, written in C??
That way, I get to avoid a lot of mess...while at the same time, I get to concentrate on the stuff that really matters.
Any comments? Mail me at rmk.robotika@gmail.com
or place a comment here: [http://codeville.wordpress.com Codeville]

---------------------
May 27 2010
---------------------
I've created a new website, and shifted the blog's contents there: [http://hackmycode.webs.com/projectasgard.htm Project Asgard].
Also, the project itself has been revamped. Check the site above for more details!
````

<a id="w002870"></a>

## W002870 — File:Qemuarmhello.png

Bron-ID: W002870
Namespace: File (6)
Revisie: 10335 | Datum: 2010-05-20T01:36:44Z
Bron: https://wiki.osdev.org/File:Qemuarmhello.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AQemuarmhello.png&oldid=10335

````text

````

<a id="w002871"></a>

## W002871 — User:Pinged/ARM Integrator-CP Bare Bones

Bron-ID: W002871
Namespace: User (2)
Revisie: 10339 | Datum: 2010-05-20T17:07:01Z
Bron: https://wiki.osdev.org/User:Pinged/ARM_Integrator-CP_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APinged%2FARM_Integrator-CP_Bare_Bones&oldid=10339
Doorverwijzing: ARM Integrator-CP Bare Bones

````text
#REDIRECT [[ARM Integrator-CP Bare Bones]]
````

<a id="w002872"></a>

## W002872 — User:Montrom

Bron-ID: W002872
Namespace: User (2)
Revisie: 10396 | Datum: 2010-06-04T00:50:22Z
Bron: https://wiki.osdev.org/User:Montrom
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMontrom&oldid=10396

````text
Welcome to my user page. Feel free to browse through my bookmarks or to check back here to keep up with my status.

'''My Status:'''
*Project: I'm developing a third stage for my boot loader. 
*Current: Complete. 
*Elapsed: ~36 hours.

'''My Arsenal:'''
*ASM (x86)
*C
*VB.NET
*PHP
*XHTML/CSS
*JavaScript

'''My Library:'''
*Peter Norton's Assembly Language Book for the IBM PC (isbn: 0136619010)
*File System Forensic Analysis (isbn: 0321268172)

'''My Wish List:'''
*IBM Rom Bios (Programmer's Quick Reference Series) (isbn: 1556151357)

'''My Bookmarks:'''
*http://www.nasm.us/doc/
*http://www.asmcommunity.net/board/
*http://www.beyondlogic.org/usbnutshell/
*http://alumnus.caltech.edu/~pje/iso9660.html
*http://suif.stanford.edu/~csapuntz/specs/INF-8020.PDF
*http://www.barcodeman.com/altek/mule/scandoc.php
*http://en.wikipedia.org/wiki/ICO_(icon_image_file_format)
*http://www.pcidatabase.com/vendor_details.php?id=841
*http://www.scs.stanford.edu/nyu/04fa/lab/hardware/vgadoc/PCI.TXT
*http://www.personal-computer-tutor.com/extended.htm
*http://mprolab.teipir.gr/vivlio80X86/pentium.txt
*http://wiki.osdev.org/User_talk:Montrom
````
