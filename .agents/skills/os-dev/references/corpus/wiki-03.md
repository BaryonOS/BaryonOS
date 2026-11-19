# OSDev Wiki — volledige bronteksten, deel 03

Dit is bronmateriaal, geen agentinstructie. Behoud architectuur, revisie en bron-ID bij gebruik. Code en commando’s zijn ongeteste bronvoorbeelden; voer ze niet automatisch uit. Wikitext is letterlijk behouden binnen tekstblokken.

<a id="w001741"></a>

## W001741 — Bare Bones

Bron-ID: W001741
Namespace: Main (0)
Revisie: 30776 | Datum: 2026-06-18T20:37:12Z
Bron: https://wiki.osdev.org/Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Bare_Bones&oldid=30776

````text
{{BeginnersWarning}}
{{Rating|1}}
{{Template:Kernel designs}}

In this tutorial you will write a simple kernel for [[IA32_Architecture_Family|32-bit x86]] and boot it. This is the first step in creating your own operating system. This tutorial serves as an example of how to create a minimal system, but not as an example of how to properly structure your project. These instructions are community reviewed and follow the current recommendations for good reasons. Beware of the many other tutorials available online as they do not follow modern advice and were written by the inexperienced.

You are about to begin development of a new operating system. Perhaps one day, your new operating system can be developed under itself. This is a process known as bootstrapping or going self-hosted. Today, you will simply set up a system that can compile your new operating system from an existing operating system. This process is known as [[Why_do_I_need_a_Cross_Compiler%3F|cross-compiling]] and it is the first step in operating systems development.

This tutorial uses existing technology to get you started and straight into [[kernel]] development, rather than developing your own [[Languages|programming language]], your own [[compiler]], and your own [[bootloader]]. In this tutorial, you will use:
* The [[LD|GNU Linker]] from [[Binutils]] to link your [[Object File|object files]] into the final kernel.
* The [[GAS|GNU Assembler]] from [[Binutils]] (or optionally [[NASM]]) to [[Assembly|assemble instructions]] into object files containing machine code.
* The [[GCC|GNU Compiler Collection]] to compile your high level code into assembly.
* The [[C]] programming language (or optionally [[C++]]) to write the high level parts of your [[kernel]].
* The [[GRUB]] bootloader to [[Bootloader|bootload]] your kernel using the [[Multiboot]] boot protocol that loads us into 32-bit protected mode with [[paging]] disabled.
* The [[ELF]] as the [[Executable Formats|executable format]] that gives us control of where and how the kernel is loaded.

This article assumes you are using a Unix-like operating system such as Linux which supports operating systems development well. Windows users should be able to complete it from a [[WSL]], [[MinGW]], or [[Cygwin]] environment.

Succeeding at operating systems development requires becoming an expert, having patience, and reading all the instructions very carefully. You need to read everything in this article before proceeding. If you run into problems, you need to reread the article even more carefully and then do it thrice more for good measure. If you still have issues, the OSDev community is experienced and will gladly help at the [http://forum.osdev.org forums] or on [[Chat|IRC]].

== Building a Cross-Compiler ==
:''Main article: [[GCC Cross-Compiler]], [[Why do I need a Cross Compiler?]]

The first thing you should do is set up a [[GCC Cross-Compiler]] for '''i686-elf'''. You have not yet modified your compiler to know about the existence of your operating system, so you will use a generic target called i686-elf, which provides you with a toolchain targeting the System V ABI. This setting is well tested and understood by the osdev community and will allow you to easily set up a bootable kernel using GRUB and Multiboot. (Note that if you are already using an ELF platform, such as Linux, you may already have a GCC that produces ELF programs. This is not suitable for osdev work, as this compiler will produce programs for Linux, and your operating system is <b>not</b> Linux, no matter how similar it is. You will certainly run into trouble if you don't use a cross-compiler.)

You will ''not'' be able to correctly compile your operating system without a cross-compiler.

You will ''not'' be able to correctly complete this tutorial with a x86_64-elf cross-compiler, as GRUB is only able to load 32-bit multiboot kernels. If this is your first operating system project, you should do a 32-bit kernel first. If you use a x86_64 compiler instead and somehow bypass the later sanity check, you will end up with a kernel that GRUB doesn't know how to boot.

== Overview ==

By now, you should have set up your [[GCC Cross-Compiler|cross-compiler]] for i686-elf (as described above). This tutorial provides a minimal solution for creating an operating system for x86. It doesn't serve as a recommend skeleton for project structure, but rather as an example of a minimal kernel. In this simple case, you just need three input files:

* boot.s - kernel entry point that sets up the processor environment
* kernel.c - your actual kernel routines
* linker.ld - for linking the above files

== Booting the Operating System ==

To start the operating system, an existing piece of software will be needed to load it. This is called the bootloader and in this tutorial you will be using [[GRUB]]. Writing your own bootloader is an advanced subject, but it is commonly done. We'll later configure the bootloader, but the operating system needs to handle when the bootloader passes control to it. The kernel is passed a very minimal environment, in which the stack is not set up yet, virtual memory is not yet enabled, hardware is not initialized, and so on.

The first task you will deal with is how the bootloader starts the kernel. OSDevers are lucky because there exists a Multiboot Standard, which describes an easy interface between the bootloader and the operating system kernel. It works by putting a few magic values in some global variables (known as a multiboot header), which is searched for by the bootloader. When it sees these values, it recognizes the kernel as multiboot compatible and it knows how to load us, and it can even forward us important information such as memory maps, but you won't need that yet.

Since there is no stack yet and you need to make sure the global variables are set correctly, you will do this in assembly.

=== Bootstrap Assembly ===
:''Alternatively, you can use [[Bare Bones with NASM|NASM]] as your assembler.''

You will now create a file called boot.s and discuss its contents. In this example, you are using the GNU assembler, which is part of the cross-compiler toolchain you built earlier. This assembler integrates very well with the rest of the GNU toolchain.

The very most important piece to create is the multiboot header, as it must be very early in the kernel binary, or the bootloader will fail to recognize us.

<syntaxhighlight lang="asm">
/* Declare constants for the multiboot header. */
.set ALIGN,    1<<0             /* align loaded modules on page boundaries */
.set MEMINFO,  1<<1             /* provide memory map */
.set FLAGS,    ALIGN | MEMINFO  /* this is the Multiboot 'flag' field */
.set MAGIC,    0x1BADB002       /* 'magic number' lets bootloader find the header */
.set CHECKSUM, -(MAGIC + FLAGS) /* checksum of above, to prove we are multiboot */

/* 
Declare a multiboot header that marks the program as a kernel. These are magic
values that are documented in the multiboot standard. The bootloader will
search for this signature in the first 8 KiB of the kernel file, aligned at a
32-bit boundary. The signature is in its own section so the header can be
forced to be within the first 8 KiB of the kernel file.
*/
.section .multiboot
.align 4
.long MAGIC
.long FLAGS
.long CHECKSUM

/*
The multiboot standard does not define the value of the stack pointer register
(esp) and it is up to the kernel to provide a stack. This allocates room for a
small stack by creating a symbol at the bottom of it, then allocating 16384
bytes for it, and finally creating a symbol at the top. The stack grows
downwards on x86. The stack is in its own section so it can be marked nobits,
which means the kernel file is smaller because it does not contain an
uninitialized stack. The stack on x86 must be 16-byte aligned according to the
System V ABI standard and de-facto extensions. The compiler will assume the
stack is properly aligned and failure to align the stack will result in
undefined behavior.
*/
.section .bss
.align 16
stack_bottom:
.skip 16384 # 16 KiB
stack_top:

/*
The linker script specifies _start as the entry point to the kernel and the
bootloader will jump to this position once the kernel has been loaded. It
doesn't make sense to return from this function as the bootloader is gone.
*/
.section .text
.global _start
.type _start, @function
_start:
	/*
	The bootloader has loaded us into 32-bit protected mode on a x86
	machine. Interrupts are disabled. Paging is disabled. The processor
	state is as defined in the multiboot standard. The kernel has full
	control of the CPU. The kernel can only make use of hardware features
	and any code it provides as part of itself. There's no printf
	function, unless the kernel provides its own <stdio.h> header and a
	printf implementation. There are no security restrictions, no
	safeguards, no debugging mechanisms, only what the kernel provides
	itself. It has absolute and complete power over the
	machine.
	*/

	/*
	To set up a stack, we set the esp register to point to the top of the
	stack (as it grows downwards on x86 systems). This is necessarily done
	in assembly as languages such as C cannot function without a stack.
	*/
	mov $stack_top, %esp

	/*
	This is a good place to initialize crucial processor state before the
	high-level kernel is entered. It's best to minimize the early
	environment where crucial features are offline. Note that the
	processor is not fully initialized yet: Features such as floating
	point instructions and instruction set extensions are not initialized
	yet. The GDT should be loaded here. Paging should be enabled here.
	C++ features such as global constructors and exceptions will require
	runtime support to work as well.
	*/

	/*
	Enter the high-level kernel. The ABI requires the stack is 16-byte
	aligned at the time of the call instruction (which afterwards pushes
	the return pointer of size 4 bytes). The stack was originally 16-byte
	aligned above and we've pushed a multiple of 16 bytes to the
	stack since (pushed 0 bytes so far), so the alignment has thus been
	preserved and the call is well defined.
	*/
	call kernel_main

	/*
	If the system has nothing more to do, put the computer into an
	infinite loop. To do that:
	1) Disable interrupts with cli (clear interrupt enable in eflags).
	   They are already disabled by the bootloader, so this is not needed.
	   Mind that you might later enable interrupts and return from
	   kernel_main (which is sort of nonsensical to do).
	2) Wait for the next interrupt to arrive with hlt (halt instruction).
	   Since they are disabled, this will lock up the computer.
	3) Jump to the hlt instruction if it ever wakes up due to a
	   non-maskable interrupt occurring or due to system management mode.
	*/
	cli
1:	hlt
	jmp 1b

/*
Set the size of the _start symbol to the current location '.' minus its start.
This is useful when debugging or when you implement call tracing.
*/
.size _start, . - _start
</syntaxhighlight>

You can then assemble boot.s using:
<syntaxhighlight lang="bash">i686-elf-as boot.s -o boot.o</syntaxhighlight>

== Implementing the Kernel ==

So far you have written the bootstrap assembly stub that sets up the processor such that high level languages such as C can be used. It is also possible to use other languages such as C++.

=== Freestanding and Hosted Environments ===

If you have done C or C++ programming in user-space, you have used a so-called Hosted Environment. Hosted means that there is a C standard library and other useful runtime features. Alternatively, there is the Freestanding version, which is what you are using here. Freestanding means that there is no C standard library, only what you provide yourself. However, some header files are actually not part of the C standard library, but rather the compiler. These remain available even in freestanding C source code. In this case you use <stdbool.h> to get the bool datatype, <stddef.h> to get size_t and NULL, and <stdint.h> to get the intx_t and uintx_t datatypes which are invaluable for operating systems development, where you need to make sure that the variable is of an exact size (if you used a short instead of uint16_t and the size of short changed, your VGA driver here would break!). Additionally you can access the <float.h>, <iso646.h>, <limits.h>, and <stdarg.h> headers, as they are also freestanding. GCC actually ships a few more headers, but these are special purpose.

=== Writing a kernel in C ===

The following shows how to create a simple kernel in C. This kernel uses the VGA text mode buffer (located at <code>0xB8000</code>) as the output device. It sets up a simple driver that remembers the location of the next character in this buffer and provides a primitive for adding a new character. Notably, there is no support for line breaks ('\n') (and writing that character will show some VGA-specific character instead) and no support for scrolling when the screen is filled up. Adding this will be your first task. Please take a few moments to understand the code.

<b>IMPORTANT NOTE</b>: the VGA text mode (as well as the BIOS) is deprecated on newer machines, and UEFI only supports pixel buffers. For forward compatibility you might want to start with that. Ask [[GRUB]] to set up a framebuffer using appropriate Multiboot flags or call [[Vesa|VESA VBE]] yourself. Unlike VGA text mode, a framebuffer has pixels, so you have to draw each glyph yourself. This means you'll need a different <code>terminal_putchar</code>, and you'll need a font (bitmap images for each character). All Linux distro ships [[PC Screen Font]]s that you can use, and the wiki article has a simple putchar() example. Otherwise everything else described here still stands (you have to keep track of the cursor position, implement line breaks and scrolling etc.)

<syntaxhighlight lang="c">
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

/* Check if the compiler thinks you are targeting the wrong operating system. */
#if defined(__linux__)
#error "You are not using a cross-compiler, you will most certainly run into trouble"
#endif

/* This tutorial will only work for the 32-bit ix86 targets. */
#if !defined(__i386__)
#error "This tutorial needs to be compiled with a ix86-elf compiler"
#endif

/* Hardware text mode color constants. */
enum vga_color {
	VGA_COLOR_BLACK = 0,
	VGA_COLOR_BLUE = 1,
	VGA_COLOR_GREEN = 2,
	VGA_COLOR_CYAN = 3,
	VGA_COLOR_RED = 4,
	VGA_COLOR_MAGENTA = 5,
	VGA_COLOR_BROWN = 6,
	VGA_COLOR_LIGHT_GREY = 7,
	VGA_COLOR_DARK_GREY = 8,
	VGA_COLOR_LIGHT_BLUE = 9,
	VGA_COLOR_LIGHT_GREEN = 10,
	VGA_COLOR_LIGHT_CYAN = 11,
	VGA_COLOR_LIGHT_RED = 12,
	VGA_COLOR_LIGHT_MAGENTA = 13,
	VGA_COLOR_LIGHT_BROWN = 14,
	VGA_COLOR_WHITE = 15,
};

static inline uint8_t vga_entry_color(enum vga_color fg, enum vga_color bg) 
{
	return fg | bg << 4;
}

static inline uint16_t vga_entry(unsigned char uc, uint8_t color) 
{
	return (uint16_t) uc | (uint16_t) color << 8;
}

size_t strlen(const char* str) 
{
	size_t len = 0;
	while (str[len])
		len++;
	return len;
}

#define VGA_WIDTH   80
#define VGA_HEIGHT  25
#define VGA_MEMORY  0xB8000 

size_t terminal_row;
size_t terminal_column;
uint8_t terminal_color;
uint16_t* terminal_buffer = (uint16_t*)VGA_MEMORY;

void terminal_initialize(void) 
{
	terminal_row = 0;
	terminal_column = 0;
	terminal_color = vga_entry_color(VGA_COLOR_LIGHT_GREY, VGA_COLOR_BLACK);
	
	for (size_t y = 0; y < VGA_HEIGHT; y++) {
		for (size_t x = 0; x < VGA_WIDTH; x++) {
			const size_t index = y * VGA_WIDTH + x;
			terminal_buffer[index] = vga_entry(' ', terminal_color);
		}
	}
}

void terminal_setcolor(uint8_t color) 
{
	terminal_color = color;
}

void terminal_putentryat(char c, uint8_t color, size_t x, size_t y) 
{
	const size_t index = y * VGA_WIDTH + x;
	terminal_buffer[index] = vga_entry(c, color);
}

void terminal_putchar(char c) 
{
	terminal_putentryat(c, terminal_color, terminal_column, terminal_row);
	if (++terminal_column == VGA_WIDTH) {
		terminal_column = 0;
		if (++terminal_row == VGA_HEIGHT)
			terminal_row = 0;
	}
}

void terminal_write(const char* data, size_t size) 
{
	for (size_t i = 0; i < size; i++)
		terminal_putchar(data[i]);
}

void terminal_writestring(const char* data) 
{
	terminal_write(data, strlen(data));
}

void kernel_main(void) 
{
	/* Initialize terminal interface */
	terminal_initialize();

	/* Newline support is left as an exercise. */
	terminal_writestring("Hello, kernel World!\n");
}
</syntaxhighlight>

Notice how in the code you wished to use the common C function <tt>strlen</tt>, but this function is part of the C standard library that you don't have available. Instead, you relied on the freestanding header <stddef.h> to provide <tt>size_t</tt> and you simply declared your own implementation of <tt>strlen</tt>. You will have to do this for every function you wish to use (as the freestanding headers only provide macros and data types).

Compile using:
<syntaxhighlight lang="bash">i686-elf-gcc -c kernel.c -o kernel.o -std=gnu99 -ffreestanding -O2 -Wall -Wextra</syntaxhighlight>

Note that the above code uses a few extensions and hence you build as the GNU version of C99.

=== Writing a kernel in C++ ===

Writing a kernel in [[C++]] is easy. Note that not all features from the language is available. For instance, exception support requires special runtime support and so does memory allocation. To write a kernel in C++, simply adopt code above: Add an extern "C" declaration to the main method. Notice how the kernel_main function has to be declared with C linkage, as otherwise the compiler would include type information in the assembly name (name mangling). This complicates calling the function from your above assembly stub and you therefore use C linkage, where the symbol name is the same as the name of the function (with no additional type information). Save the code as kernel.c++ (or what your favorite C++ filename extension is). 

You can compile the file kernel.c++ using:

<syntaxhighlight lang="bash">i686-elf-g++ -c kernel.c++ -o kernel.o -ffreestanding -O2 -Wall -Wextra -fno-exceptions -fno-rtti</syntaxhighlight>

Note that you must have also built a cross C++ compiler for this work.

== Linking the Kernel ==

You can now assemble boot.s and compile kernel.c. This produces two object files that each contain part of the kernel. To create the full and final kernel you will have to link these object files into the final kernel program, usable by the bootloader. When developing user-space programs, your toolchain ships with default scripts for linking such programs. However, these are unsuitable for kernel development and you need to provide your own customized linker script. Save the following in linker.ld:

<syntaxhighlight lang="c">
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

With these components you can now actually build the final kernel. We use the compiler as the linker as it allows it greater control over the link process. Note that if your kernel is written in C++, you should use the C++ compiler instead. 

You can then link your kernel using:

<syntaxhighlight lang="bash">i686-elf-gcc -T linker.ld -o myos -ffreestanding -O2 -nostdlib boot.o kernel.o -lgcc</syntaxhighlight>

Note: Some tutorials suggest linking with i686-elf-ld rather than the compiler, however this prevents the compiler from performing various tasks during linking.

The file myos is now your kernel (all other files are no longer needed). Note that we are linking against [[libgcc]], which implements various runtime routines that your cross-compiler depends on. Leaving it out will give you problems in the future. If you did not build and install [[libgcc]] as part of your cross-compiler, you should go back now and build a cross-compiler with [[libgcc]]. The compiler depends on this library and will use it regardless of whether you provide it or not.

== Verifying Multiboot ==

If you have [[GRUB]] installed, you can check whether a file has a valid [[Multiboot]] version 1 header, which is the case for your kernel. It's important that the Multiboot header is within the first 8 KiB of the actual program file at 4 byte alignment. This can potentially break later if you make a mistake in the boot assembly, the linker script, or anything else that might go wrong. If the header isn't valid, GRUB will give an error that it can't find a Multiboot header when you try to boot it. This code fragment will help you diagnose such cases:

<syntaxhighlight lang="bash">
grub-file --is-x86-multiboot myos
</syntaxhighlight>

<tt>grub-file</tt> is quiet but will exit 0 (successfully) if it is a valid multiboot kernel and exit 1 (unsuccessfully) otherwise. You can type <tt>echo $?</tt> in your shell immediately afterwards to see the exit status. You can add this grub-file check to your build scripts as a sanity test to catch the problem at compile time. Multiboot version 2 can be checked with the <tt>--is-x86-multiboot2</tt> option instead. If you invoke the <tt>grub-file</tt> command manually in a shell, it is convenient to wrap it in a conditional to easily see the status. This command should work now:

<syntaxhighlight lang="bash">
if grub-file --is-x86-multiboot myos; then
  echo multiboot confirmed
else
  echo the file is not multiboot
fi
</syntaxhighlight>

== Booting the Kernel ==
In a few moments, you will see your kernel in action.

=== Building a bootable cdrom image ===
You can easily create a bootable CD-ROM image containing the GRUB bootloader and your kernel using the program <tt>grub-mkrescue</tt>. You may need to install the GRUB utility programs and the program <tt>xorriso</tt> (version 0.5.6 or higher). First you should create a file called grub.cfg containing the contents:

<syntaxhighlight lang="c">
menuentry "myos" {
	multiboot /boot/myos
}
</syntaxhighlight>

Note that the braces must be placed as shown here. You can now create a bootable image of your operating system by typing these commands:
<syntaxhighlight lang="bash">
mkdir -p isodir/boot/grub
cp myos isodir/boot/myos
cp grub.cfg isodir/boot/grub/grub.cfg
grub-mkrescue -o myos.iso isodir
</syntaxhighlight>

Congratulations! You have now created a file called myos.iso that contains your Hello World operating system. If you don't have the program <tt>grub-mkrescue</tt> installed, now is a good time to install GRUB. It should already be installed on Linux systems. Windows users will likely want to use a Cygwin variant if no native grub-mkrescue program is available.

'''Warning:''' GNU GRUB, the bootloader used by <tt>grub-mkrescue</tt>, is licensed under the GNU General Public License. Your iso file contains copyrighted material under that license and redistributing it in violation of the GPL constitutes copyright infringement. The GPL requires you publish the source code corresponding to the bootloader. You need to get the exact source package corresponding to the GRUB package you have installed from your distribution, at the time <tt>grub-mkrescue</tt> is invoked (as distro packages are occasionally updated). You then need to publish that source code along with your ISO to satisfy the GPL. Alternative, you can build GRUB from source code yourself. Extract it somewhere, then build GRUB from it, and install it in a isolated prefix in your PATH to ensure its <tt>grub-mkrescue</tt> program is used to produce your iso. You can then publish the official GRUB tarball along with your OS release. You're not required to publish the source code of your OS at all, only the code for the bootloader that's inside the iso.

=== Testing your operating system (QEMU) ===
Virtual Machines are very useful for development operating systems, as they allow you to quickly test your code and have access to the source code during the execution. Otherwise, you would be in for an endless cycle of reboots that would only annoy you. They start very quickly, especially combined with small operating systems such as yours.

In this tutorial, we will be using QEMU. You can also use other virtual machines if you please. Simply adding the ISO to the CD drive of an empty virtual machine will do the trick.

Install QEMU from your repositories, and then use the following command to start your new operating system.

<syntaxhighlight lang="bash">qemu-system-i386 -cdrom myos.iso</syntaxhighlight>

This should start a new virtual machine containing only your ISO as a cdrom. If all goes well, you will be met with a menu provided by the bootloader. Simply select myos and if all goes well, you should see the happy words "Hello, Kernel World!" followed by some mysterious character.

Additionally, QEMU supports booting multiboot kernels directly without bootable medium:

<syntaxhighlight lang="bash">qemu-system-i386 -kernel myos</syntaxhighlight>

=== Testing your operating system (Real Hardware) ===

The program grub-mkrescue is nice because it makes a bootable ISO that works on both real computers and virtual machines. You can then build an ISO and use it everywhere. To boot your kernel on your local computer you can install myos to your /boot directory and configure your bootloader appropriately.

Or alternatively, you can burn it to an USB stick (erasing all data on it!). To do so, simply find out the name of the USB block device, in my case /dev/sdb but this may vary, and using the wrong block device (your harddisk, gasp!) may be disastrous. If you are using Linux and /dev/sdx is your block name, simply:

<syntaxhighlight lang="bash">sudo dd if=myos.iso of=/dev/sdx && sync</syntaxhighlight>

Your operating system will then be installed on your USB stick. If you configure your BIOS to boot from USB first, you can insert the USB stick and your computer should start your operating system.

Alternatively, the .iso is a normal cdrom image. Simply burn it to a CD or DVD if you feel like wasting one of those on a few kilobytes large kernel.

== Moving Forward ==
Now that you can run your new shiny operating system, congratulations! Of course, depending on how much this interests you, it may just be the beginning. Here's a few things to get going.

=== Adding Support for Newlines to Terminal Driver ===
The current terminal driver does not handle newlines. The VGA text mode font stores another character at the location, since newlines are never meant to be actually rendered: they are logical entities. Rather, in terminal_putchar check if c == '\n' and increment terminal_row and reset terminal_column.

=== Implementing Terminal Scrolling ===
In case the terminal is filled up, it will just go back to the top of the screen. This is unacceptable for normal use. Instead, it should move all rows up one row and discard the upper most, and leave a blank row at the bottom ready to be filled up with characters. Implement this.

=== Rendering Colorful ASCII Art ===
Use the existing terminal driver to render some pretty stuff in all the glorious 16 colors you have available. Note that only 8 colors may be available for the background color, as the uppermost bit in the entries by default means something other than background color. You'll need a real VGA driver to fix this.

=== Calling Global Constructors ===
{{Main|Calling Global Constructors}}
This tutorial showed a small example of how to create a minimal environment for C and C++ kernels. Unfortunately, you don't have everything set up yet. For instance, C++ with global objects will not have their constructors called because you never do it. The compiler uses a special mechanism for performing tasks at program initialization time through the <tt>crt*.o</tt> objects, which may be valuable even for C programmers. If you combine the <tt>crt*.o</tt> files correctly, you will create an <tt>_init</tt> function that invokes all the program initialization tasks. Your boot.o object file can then invoke <tt>_init</tt> before calling <tt>kernel_main</tt>.

=== Meaty Skeleton ===
{{Main|Meaty Skeleton}}
This tutorial is meant as a minimal example to give impatient beginners a quick hello world operating system. It is deliberately minimal and doesn't show the best practices on how to organize your operating system. The Meaty Skeleton tutorial shows an example of how to organize a minimal operating system with a kernel, room for a standard library to grow, and prepared for a user-space to appear.

=== Going Further ===
{{Main|Going Further on x86}}
This guide is meant as an overview of what to do, so you have a kernel ready for more features, without actually redesigning it radically when adding them.

=== Bare Bones <tt>II</tt> ===

Make your operating system self-hosting and then complete Bare Bones under your own operating system while following all the instructions. This is a five star exercise and you may need a couple of years to solve it. Extra credit if you host all of the osdev.org infrastructure on your own operating system.

== Frequently Asked Questions ==

<!--
TODO: People might need mformat(1) (part of mtools) installed to run grub-mkrescue on UEFI systems. At least my build documentation says so and I remember having such issues once. Haven't heard that issue much since though. This needs to be researched and mentioned above if it's the case, and with an error message down here to help people catch it. I somehow thought that was already mentioned in the article, but isn't right now, so check the article history to see if it was removed-
-->

;Why the Multiboot header? Wouldn't a pure [[ELF]] file be loadable by GRUB anyway?
:GRUB is capable of loading a variety of formats. However, in this tutorial we are creating a Multiboot compliant kernel that could be loaded by any other compliant bootloader. To achieve this, the multiboot header is mandatory.

;Is the AOUT kludge required for my kernel?
:The AOUT kludge is not necessary for kernels in ELF format: a multiboot-compliant loader will recognize an ELF executable as such and use the program header to load things in their proper place. You can provide an AOUT kludge with your ELF kernel, in which case the headers of the ELF file are ignored. With any other format, such as AOUT, COFF or PE kernels, the AOUT kludge it is required, however.

;Can the Multiboot header be anywhere in the kernel file, or does it have to be in a specific offset?
:The Multiboot header must be in the first 8kb of the kernel file and must be aligned to a 32-bit (4 byte) boundary for GRUB to find it. You can ensure that this is the case by putting the header in its own source code file and passing that as the first object file to LD.

;Will GRUB wipe the BSS section before loading the kernel?
:Yes. For ELF kernels, the .bss section is automatically identified and cleared (despite the Multiboot specification being a bit vague about it). For other formats, if you ask it politely to do so, that is if you use the 'address override' information from the Multiboot header (flag #16) and give a non-zero value to the bss_end_addr field. Note that using "address override" with an ELF kernel will disable the default behavior and do what is described by the "address override" header instead.

;What is the state of registers/memory/etc. when GRUB calls my kernel?
:GRUB is an implementation of the Multiboot specification. Anything not specified there is "undefined behavior", which should ring a bell (not only) with C/C++ programmers... Better check the Machine State section of Multiboot documentation, and assume nothing else.

;I still get <tt><nowiki>Error 13: Invalid or unsupported executable format from GRUB</nowiki></tt>...
:Chances are the Multiboot header is missing from the final executable, or it is not at the right location.
:If you are using some other format than ELF (such as PE), you should specify the AOUT kludge in the Multiboot header. The <tt>grub-file</tt> program describe aboveand "objdump -h" should give you more hints about what is going on.
:It may also happen if you use an ELF object file instead of an executable (e.g. you have an ELF file with unresolved symbols or unfixable relocations). Try to link your ELF file to a binary executable to get more accurate error messages.
:A common problem when your kernel size increases, is that the Multiboot header does no longer appear at the start of the output binary. The common solutions is to put the Multiboot header in a separate section and make sure that section is first in the output binary, or to include the Multiboot header itself in the linker script.

;I get <tt><nowiki>Boot failed: Could not read from CD-ROM (code 0009)</nowiki></tt> when trying to boot the iso image in QEMU
:If your development system is booted from EFI it may be that you don't have the PC-BIOS version of the grub binaries installed anywhere.  If you install them then grub-mkrescue will by default produce a hybrid ISO that will work in QEMU.  On Ubuntu this can be achieved with: <b>apt-get install grub-pc-bin</b>.

== See Also ==

=== Articles ===

* [[Books]]
* [[Limine Bare Bones]]
* [[BOOTBOOT]]

=== External Links ===

* [https://github.com/limine-bootloader/limine/blob/trunk/PROTOCOL.md Limine Boot Protocol Specification]
* [https://www.gnu.org/software/grub/manual/multiboot/multiboot.html Multiboot Specification]
* [https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html Multiboot2 Specification]
* [https://gitlab.com/bztsrc/bootboot/raw/master/bootboot_spec_1st_ed.pdf BOOTBOOT Specification]
* [https://pubs.opengroup.org/onlinepubs/9799919799/ The POSIX Standard]

[[Category:Bare bones tutorials]]
[[Category:C]]
[[Category:C++]]
````

<a id="w001742"></a>

## W001742 — C++ Bare Bones

Bron-ID: W001742
Namespace: Main (0)
Revisie: 14706 | Datum: 2013-05-18T16:45:01Z
Bron: https://wiki.osdev.org/C%2B%2B_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=C%2B%2B_Bare_Bones&oldid=14706
Doorverwijzing: Bare Bones

````text
#REDIRECT [[Bare Bones]]
````

<a id="w001743"></a>

## W001743 — User:Xyz1/OS projects

Bron-ID: W001743
Namespace: User (2)
Revisie: 2746 | Datum: 2007-01-10T08:17:20Z
Bron: https://wiki.osdev.org/User:Xyz1/OS_projects
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AXyz1%2FOS_projects&oldid=2746

````text
* [[Describe new OS project|OS Projects Instructions]]
* [[Innovations|Innovative Features]]
* [[Search OS|(Extended) OS Search]]
* [[NewProjects|(New) OS Projects]]
* [[Projects|Old OS Projects list]]
````

<a id="w001744"></a>

## W001744 — User:Xyz1/Describe new OS project

Bron-ID: W001744
Namespace: User (2)
Revisie: 2744 | Datum: 2007-01-10T08:16:38Z
Bron: https://wiki.osdev.org/User:Xyz1/Describe_new_OS_project
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AXyz1%2FDescribe_new_OS_project&oldid=2744

````text
Notes, about:

- '''activity''':

&nbsp; -- distro [last update, about some components (diagram)]

&nbsp; -- get help: author: [YES/NO] forum: [YES/NO] documentation: [good/bad].

- '''feautures''':

&nbsp; -- programming language: [C/assembler/...]

&nbsp; -- license: [public domain/other:.../gpl/closed code]

&nbsp; -- GUI: [YES/NO] [...features...]

- '''using''':

&nbsp; -- install: [easy/difficult]

&nbsp; -- (download) size: [kernel/all/+applications]

&nbsp; -- live demo: [YES/NO] [...diagram, when it is avaible...]

&nbsp; -- emulators: Bochs: [YES/NO] qemu: [YES/NO] [...other...]

- '''development''':

&nbsp; -- name: ''name''; start in os: [YES/NO]

&nbsp; -- kernel structure: ???

&nbsp; -- layers between different parts of an OS: [YES/NO/Describe when]

- '''other information''':

&nbsp; -- project homepage: [url]

&nbsp; -- developers: main & other

&nbsp; -- news in the press

&nbsp; -- special features
````

<a id="w001745"></a>

## W001745 — Higher Half x86 Bare Bones (Backup)

Bron-ID: W001745
Namespace: Main (0)
Revisie: 23056 | Datum: 2018-10-27T19:39:13Z
Bron: https://wiki.osdev.org/Higher_Half_x86_Bare_Bones_(Backup)
Revisie-URL: https://wiki.osdev.org/index.php?title=Higher_Half_x86_Bare_Bones_%28Backup%29&oldid=23056

````text
{{Disputed|Talk:Higher_Half_x86_Bare_Bones}}
{{FirstPerson}}
{{TutorialExplain}}
{{BeginnersWarning}}
{{Rating|2}}

Here is some sample code for a kernel that is loaded by GRUB and is mapped in the upper half of memory. In this case, the kernel is loaded at 1MB in the physical address space (0x00100000), but is mapped at 3GB + 1MB in the virtual address space (0xC0100000). It is recommended that you have a firm grasp of the contents within the [[Bare bones|Bare bones tutorial]] and [[Paging]] before attempting this.

==loader.asm==
This piece of code is taking over control from the Multiboot bootloader. It sets up a page directory with page table entries that identity map the first 4MB, and also map the first 4MB to virtual 3GB. After setting up paging, it unmaps the identity mapping so that the kernel is entirely in the higher half and jumps into the kernel proper.

<source lang="asm">
global _loader                          ; Make entry point visible to linker.
extern _main                            ; _main is defined elsewhere

; setting up the Multiboot header - see GRUB docs for details
MODULEALIGN equ  1<<0             ; align loaded modules on page boundaries
MEMINFO     equ  1<<1             ; provide memory map
FLAGS       equ  MODULEALIGN | MEMINFO  ; this is the Multiboot 'flag' field
MAGIC       equ    0x1BADB002     ; 'magic number' lets bootloader find the header
CHECKSUM    equ -(MAGIC + FLAGS)  ; checksum required

; This is the virtual base address of kernel space. It must be used to convert virtual
; addresses into physical addresses until paging is enabled. Note that this is not
; the virtual address where the kernel image itself is loaded -- just the amount that must
; be subtracted from a virtual address to get a physical address.
KERNEL_VIRTUAL_BASE equ 0xC0000000                  ; 3GB
KERNEL_PAGE_NUMBER equ (KERNEL_VIRTUAL_BASE >> 22)  ; Page directory index of kernel's 4MB PTE.


section .data
align 0x1000
BootPageDirectory:
    ; This page directory entry identity-maps the first 4MB of the 32-bit physical address space.
    ; All bits are clear except the following:
    ; bit 7: PS The kernel page is 4MB.
    ; bit 1: RW The kernel page is read/write.
    ; bit 0: P  The kernel page is present.
    ; This entry must be here -- otherwise the kernel will crash immediately after paging is
    ; enabled because it can't fetch the next instruction! It's ok to unmap this page later.
    dd 0x00000083
    times (KERNEL_PAGE_NUMBER - 1) dd 0                 ; Pages before kernel space.
    ; This page directory entry defines a 4MB page containing the kernel.
    dd 0x00000083
    times (1024 - KERNEL_PAGE_NUMBER - 1) dd 0  ; Pages after the kernel image.


section .text
align 4
MultiBootHeader:
    dd MAGIC
    dd FLAGS
    dd CHECKSUM

; reserve initial kernel stack space -- that's 16k.
STACKSIZE equ 0x4000

; setting up entry point for linker
loader equ (_loader - 0xC0000000)
global loader

_loader:
    ; NOTE: Until paging is set up, the code must be position-independent and use physical
    ; addresses, not virtual ones!
    mov ecx, (BootPageDirectory - KERNEL_VIRTUAL_BASE)
    mov cr3, ecx                                        ; Load Page Directory Base Register.

    mov ecx, cr4
    or ecx, 0x00000010                          ; Set PSE bit in CR4 to enable 4MB pages.
    mov cr4, ecx

    mov ecx, cr0
    or ecx, 0x80000000                          ; Set PG bit in CR0 to enable paging.
    mov cr0, ecx

    ; Start fetching instructions in kernel space.
    ; Since eip at this point holds the physical address of this command (approximately 0x00100000)
    ; we need to do a long jump to the correct virtual address of StartInHigherHalf which is
    ; approximately 0xC0100000.
    lea ecx, [StartInHigherHalf]
    jmp ecx                                                     ; NOTE: Must be absolute jump!

StartInHigherHalf:
    ; Unmap the identity-mapped first 4MB of physical address space. It should not be needed
    ; anymore.
    mov dword [BootPageDirectory], 0
    invlpg [0]

    ; NOTE: From now on, paging should be enabled. The first 4MB of physical address space is
    ; mapped starting at KERNEL_VIRTUAL_BASE. Everything is linked to this address, so no more
    ; position-independent code or funny business with virtual-to-physical address translation
    ; should be necessary. We now have a higher-half kernel.
    mov esp, stack+STACKSIZE           ; set up the stack
    push eax                           ; pass Multiboot magic number

    ; pass Multiboot info structure -- WARNING: This is a physical address and may not be
    ; in the first 4MB!
    push ebx

    call  _main                  ; call kernel proper
    hlt                          ; halt machine should kernel return


section .bss
align 32
stack:
    resb STACKSIZE      ; reserve 16k stack on a uint64_t boundary
</source>

==linker.ld==
This is a little trickier than it was for the [[Bare bones|C kernel tutorial]], since you need to distinguish between virtual addresses (which will be in the higher half) and load addresses, which GRUB needs to decide where to put your kernel.

<source lang="c">
ENTRY(loader)
OUTPUT_FORMAT(elf32-i386)

SECTIONS {
   /* The kernel will live at 3GB + 1MB in the virtual
      address space, which will be mapped to 1MB in the
      physical address space. */
   . = 0xC0100000;

   .text : AT(ADDR(.text) - 0xC0000000) {
       *(.text)
       *(.rodata*)
   }

   .data ALIGN (0x1000) : AT(ADDR(.data) - 0xC0000000) {
       *(.data)
   }

   .bss : AT(ADDR(.bss) - 0xC0000000) {
       _sbss = .;
       *(COMMON)
       *(.bss)
       _ebss = .;
   }
}
</source>

Note that we use loader (and not _loader) as our entry point. This is due to the fact that _loader's address is approximately 0xC0100000, if we try to set our eip to that address it will not find our loader function.
Also note our entry point is not being converted to physical address. GRUB does this conversion when calculating starting value of EIP, and if you attempt to do the translation, you may get your execution when you don't want it or get "entry point isn't in a segment" error.

==kernel.c==

Using the [[Bare bones#Writing_a_kernel_in_C|kernel.c code]] from the original bare bones tutorial will work fine, with one small change.  On the fourth line in terminal_initialize(), change:

<source lang="c">
terminal_buffer = (uint16_t*) 0xB8000;
</source>

to

<source lang="c">
terminal_buffer = (uint16_t*) 0xC00B8000;
</source>

This accomodates for the kernel's new offset into higher-half space.  Any direct memory access by the kernel at this point should take place with respect to this offset where necessary.

==Troubleshooting==
;I got a page fault (#PF) when accessing my GRUB Multiboot info structure
The address passed by loader.s is physical, you have to make it virtual to add your virtual base to it. For example, in your loader.s:
<source lang="asm">
   add ebx, KERNEL_VIRTUAL_BASE ; make the address virtual
   push ebx ; push it on the stack for _main()
</source>
Don't forget to make all addresses pointing to memory locations in the Multiboot info structure also virtual.

==See Also==
===Articles===
*[[Bare bones|Bare bones]]
*An [[User:Glauxosdever/Higher_Half_x86_Bare_Bones|alternate]] higher half Bare Bones.

[[Category:Bare bones tutorials|Higher Half bare bones]]
````

<a id="w001746"></a>

## W001746 — DJGPP

Bron-ID: W001746
Namespace: Main (0)
Revisie: 11721 | Datum: 2011-07-05T11:33:26Z
Bron: https://wiki.osdev.org/DJGPP
Revisie-URL: https://wiki.osdev.org/index.php?title=DJGPP&oldid=11721

````text
The DJGPP toolkit is a port of several basic GNU utilities to MS-DOS, using the 32-bit DPMI system extender, created by DJ Delorie. It included versions of GCC, binutils, and EMACS. It also includes several DOS and Windows specific utilities.

While DJGPP has been widely used for OS development in the past, it's close connection to MS-DOS and compatibility problems with Windows XP have caused most Windows users to shift to [[Cygwin]] or [[MinGW]]. DJGPP must be considered obsolete by now.

==ELF files==
It is fairly simple to produce ELF files with DJGPP. What you will need is a good installation of DJGPP, you will need sed, bison, flex, bash, etc. You will also need to do it from a dosbox inside Windows95/98 etc so you can use long file names.

First, download the latest binutils. (I used <tt>binutils-2.9.1.tar.gz</tt>). You need the plain binutils, not the binutils for Linux, not the binutils for dos, etc, but the plain binutils from any GNU archive site.

Once you have downloaded binutils, if you have LESS than 64mb of memory, edit the properties of your dos box and manually enter 65535 into the box provided for EMS memory, XMS memory and DPMI memory (using the selectors in the box will not let you enter 65535 so it must be entered by hand).

use DJTARX to unpack the binutils into your root directory (its easier).

<pre>
C:
CD \
DJTARX \download\binutils-2.9.1.tar.gz
</pre>

Once its unpacked you will have to make some very minor modifications to you DJGPP installation (nothing drastic!)

Enter into the DJGPP\bin directory and COPY <tt>flex.exe</tt> <tt>lex.exe</tt>, <tt>bash.exe</tt> to <tt>sh.exe</tt> and make a new directory in your root called <tt>\bin</tt> and copy <tt>sh.exe</tt> to <tt>\bin\sh.exe</tt>

<pre>
C:
CD \
COPY flex.exe lex.exe
COPY bash.exe sh.exe
MD \bin
COPY \djgpp\bin\bash.exe \bin\sh.exe
</pre>

The <tt>\bin</tt> directory must be on the SAME DRIVE as the one you unpacked binutils onto, which CAN be a different drive to what DJGPP is installed onto.

Now enter the binutils directory (<tt>cd binutils-2.9.1</tt>) and edit the "configure" script.

Near the top of the script are three lines that read

<pre>
remove
hard_link
symbolic_link
</pre>

alter these lines so they read

<pre>
remove=rm
hard_link=cp
symbolic_link=cp
</pre>

Save the modified configure script.

Execute the bash shell by typing bash, your prompt should change to "<tt>bash$ </tt>".

Now you have to setup bash to work with djgpp, enter these commands.

<pre>
export CC="c:/djgpp/bin/gcc.exe"
export LD="c:/djgpp/bin/ld.exe"
export BISON="c:/djgpp/bin/bison.exe"
export YACC="c:/djgpp/bin/bison.exe -y"
</pre>

Now you are ready to build your new set of [[ELF]] utils. First you have to run configure with some options.

You can choose what targets you want built into your new set of binutils, myself, I built in all targets, just in case I later need something other than elf (some a.out formats for BSD, etc!).

<pre>
./configure --enable-targets=all --with-gnu-as --with-gnu-ld --with-stabs
</pre>

After a very long process, if all goes well, you will find no errors, and will be back at the "<tt>bash$ </tt>" prompt ready to build your new utils.

Now... some more tinkering... the configure process is not 100%, you have to edit the "libtool" file that's in each directories, bfd/libtool, as/libtool, gprof/libtool, ld/libtool and opcodes/libtool. So exit from bash ("exit") and use an editor to open those files, and just re-save them (make no modifications, just save). This seems to fix the crlf/lf/cr unix/dos mechanisms that is a bit broken in libtool.

Re-enter bash and type <tt>make</tt> and it will build your new utils. If you have enabled all targets the resulting <tt>.exe</tt> files will be about 6 to 7mb EACH, so strip off the debugging info (<tt>strip --strip-all *.exe</tt>) and then use the DJP compressor on them and they should come out to about 600 to 900kb each.

Alternatively, if you did not want to build all targets you can run configure without the "<tt>--enable-targets=all</tt>" option and replace with "--target=i386-unknown-elf".

GCC itself does not output [[ELF]]/[[COFF]]/[[a.out]] files, <tt>GCC</tt> outputs assembler that gets passed to <tt>AS</tt> which assembles it to whatever output format is default.


==See Also==
===Articles===
*[[GCC]]
*[[Cygwin]]
*[[MinGW]]
===External Links===
*http://www.delorie.com/djgpp/ - DJGPP website

[[Category:Compilers]]
````

<a id="w001747"></a>

## W001747 — MinGW

Bron-ID: W001747
Namespace: Main (0)
Revisie: 28040 | Datum: 2023-07-09T18:57:25Z
Bron: https://wiki.osdev.org/MinGW
Revisie-URL: https://wiki.osdev.org/index.php?title=MinGW&oldid=28040

````text
{{Stub}}
'''MinGW''' (Minimalist GNU for Windows) is a port of several GNU utilities to the Windows console environment, including the GCC / binutils toolchain. It is less sophisticated than Cygwin, but more compact with a lower overhead. It is most widely used in conjunction with the [[Dev-C++]] and CodeBlocks [[:Category:IDEs|IDE]].

==See Also==
===Articles===
*[[GCC]]
*[[Cygwin]]
===Forum===
*[[Topic:10089|Osdeving with MinGW]]
===External Links===
*http://www.mingw.org/

[[Category:Compilers]]
[[Category:Windows]]
````

<a id="w001748"></a>

## W001748 — Getting Started

Bron-ID: W001748
Namespace: Main (0)
Revisie: 29110 | Datum: 2024-06-07T13:41:28Z
Bron: https://wiki.osdev.org/Getting_Started
Revisie-URL: https://wiki.osdev.org/index.php?title=Getting_Started&oldid=29110

````text
First of all, developing an operating system is probably one of the most challenging things you can do on a computer (next to killing the final boss in Doom on Nightmare difficulty level). Composing an operating system requires a lot of knowledge about several complex areas within computer science. You need to understand how hardware works and be able to read and write the complex assembly language as well as a higher-level language (such as C, C++, or [[Pascal]]). Your mind has to be able to wrap itself around abstract theory and hold a myriad of thoughts. Feel discouraged yet? Don't fear! Because all of these things are also the things that makes OS programming fun and entertaining.

There is nothing like the feeling of accomplishment when you, finally, after hours of struggling, solve the problem. And after some time you are able to look back and see all of the things you've created from scratch. Your handwritten system is able to boot, performs magic against the hardware, and gives the user a user interface and programs to play with.

There is no absolute path you have to take when creating an OS. Once you get your initial system up and running (and you do this by finding appropriate tutorials), you choose the path you want to take next. Your OS is exactly that--yours. You have ultimate control, and the sky's the limit!

==The Hard Truth==
Hopefully the basic fact that operating system development is a complicated and ongoing process does not discourage you. The truth is, operating system development is truly unparalleled since it requires the utmost amount of patience and careful code design, and it returns very little to no "instant gratification" you get from the development of things like games and web-based scripting.

You have been fairly warned of the hard work ahead, but if you are still interested then proceed forward into the realm of the operating system programmer. Prepare yourself for occasional bouts of confusion, discouragement, and for some of us...temporary insanity. In time, and with enough dedication, you will find yourself among the elite few who have contributed to a working operating system. If you do get discouraged along the way, refresh yourself with the content of this book. Hopefully it will remind you why you started such an insane journey in the first place.

At this stage, it would also pay to read the [[Beginner Mistakes]] page. Users on the forum have noticed a lot of these mistakes getting repeated over time, and avoiding them is a great way to not make a fool of yourself.

==Responsibility==
People tend to claim that it is OK to write inefficient software, stating that computer systems are so fast these days, that you won't see the impact. This type of mentality is dangerous in operating system design. It might be OK to write sloppy code when making a simple application, but when it comes to critical code that may get called thousands of times per second, you need to take out all the overhead you can. The operating system should supply the computer as a basic resource to the running applications, with as little complication, abstraction, and overhead as possible.

People who design operating systems in this day and age tend to have the "everything but the kitchen sink" mentality. They take it upon themselves to account for everything, which of course is good, but it shouldn't be done at the expense of allowing poorly-written programs to flourish. There are many things that go on "under the hood" when program errors occur. Poorly-written programs cost precious execution time and involve task switches that are expensive in both memory and frequency. We encourage you to discourage poorly-written software.

== Required Knowledge ==
{{Main|Required Knowledge}}

'''If you think you can skip this, it's just for you.'''

''This section has been [[Required Knowledge | moved to a separate page]] because it is referred to so often in forum discussions.''

==Organize your plans==
Before proceeding, consider what it is you want to get out of writing an operating system. What are your motivations in taking on this project? There are many possible reasons for taking on a hobby OS project, and most os-devers have more than one. Even just saying, "I just want to" can be enough, though the more you consider and clarify your goals and motives, the more you can focus on what you really want.

Be honest with yourself, as well. There's no shame in having larger ambitions for your project, even (or especially) if they aren't the primary objective. Try to acknowledge ''all'' of your goals, not just the one you think is your main purpose.

Try to settle on which aspects of OS design you are most interested in or see a need to work on. Most of what goes into OS dev, especially early on, is kernel design and development, but the kernel itself is only a small part of most operating systems; if your primary interest is in UX, or networking, or driver programming, you should think about whether you really need (now or in the future) to write your own OS at all or if you would be just as satisfied developing those things on an existing kernel. More than a few people have gone into OS dev when they really wanted to design a desktop environment, so this is a very important question to ask yourself.

Try to think of any non-OS projects you might want to take on first, or at the same time, especially ones which might serve as practice or preparation for the OS project. There's usually no need to work on the OS project ''right now'', and the more you have prepared ahead of time, the better off you'll be (up to a point, at least--preparation is one thing, procrastination is something else).

Similarly, if you mean to work on forking an existing design to experiment with, or to modify for some specific purpose, focus on that rather than general development issues. Consider what part of the existing code base you will need, and which ones you want to change.

Try to work out some of your specific project goals, and be prepared to plan out separate projects if it helps to do so. If you are simply intending to putter around and see where it takes you, that's fine; if your intent is to overthrow Microsoft, that's fine (if probably unrealistic), too. Once you know what you want to do, you can break down the details of it into specific objectives, and work out what it would take to reach them. Don't try to force too many divergent objectives into one project--if you have different things you want to try with contradictory goals, split them into different projects.

It may help if you write out an overview of your planned OS design, with any specific requirements or details you feel are notable or which could clarify what you need help with, and add it to your public repository if you can. This not only will make it easier for others to help you, it will help organize and stabilize your plans, much like writing an outline for a story or paper. Be prepared to maintain it as your goals and plans change, but keep a copy of older versions (or better still, keep the document under version control) so you can see how your work develops over time.

Finally, review the time and resources which the project will require, and decide if they are feasible. If you know that you only have a certain amount of time to devote to the project, take that into account, and whatever you do, ''don't'' commit to an outside deadline even if you are certain you can reach it. OS dev takes time--a '''lot''' of time--and trying to finish a full OS project in a semester isn't realistic.

==Choosing your development environment==
You need a platform to develop your new system on. Following the trends of general computing, the most popular is GNU/Linux, but many use Windows too. Developers using a GNU/Linux system have a slight advantage in availability of tools, but this can be solved on Windows using a system such as [[Cygwin]] or [[MinGW]].

* '''Binutils''': Fundamental tools for manipulation of object files.
* '''[[GCC]]''': The GNU Compiler Collection. GCC contains compilers for C, C++, Fortran, and Ada, amongst others.
* '''[[Makefile|Make]]''': For automating the build process, which becomes really helpful once you have more than a handful of files.
* '''Grep''' and '''sed''': For doing more powerful searches and search-and-replaces (helpful when filling out tables with data).
* '''Diffutils''': Incredibly useful for showing the differences between two files.
* '''[[Wikipedia:Perl|Perl]]''' or '''[[Wikipedia:Python_(programming_language)|Python]]''': One of these two scripting languages should be installed. Useful for string manipulation, amongst other things. Perl used to be the recommendation, but Python is now quite mature and is possibly easier to learn. Both have hundreds of packages/modules available for doing various tasks.
* '''[[Which assembler should I use?(x86)|An Assembler]]''': For example NASM or GAS. This varies depending on your target CPU architecture.
* '''[[List of editors|An editor]]''': For writing your Assembly, C, and other (code) files.

You might not use all of these tools, but it is best to have them on hand "just in case," and know how to use them, even at a basic level. But if you decided to use another [[Languages | language]] then the tooling is mostly up to you and maybe the list above just won't help you in any way. Below is the information mostly related to the C/C++ or Assembly developers.

===GNU/Linux===

The most recommended system for OS development is GNU/Linux. When using GNU/Linux, most of the GNU development tools are probably already present. If not, use your distribution's package management tools (APT, RPM, Portage, Pacman, Apk, etc.) to install them as needed. Again, making a [[GCC Cross-Compiler | cross-compiler]] is required, so as not to link in the development system's runtime files.

Common editors are [[Wikipedia:Vim (text editor)|Vim]], [[Wikipedia:Emacs|Emacs]], [[Wikipedia:KDevelop|KDevelop]], [[Wikipedia:Komodo_Edit|Komodo Edit]], etc. Some prefer lightweight editors instead of an IDE, such as [[Wikipedia:gedit|gedit]], [[Wikipedia:Geany|Geany]] and [[Wikipedia:SciTE|SciTE]]. Many like [[Wikipedia:Midnight_Commander|Midnight Commander]] which has a [[Text UI]] and a built-in editor (mcedit) and therefore extremely lightweight and lightning fast.

About which distributions you should use, consult the [https://en.wikipedia.org/wiki/List_of_Linux_distributions list of Linux distributions]. They come in all shapes and sizes, but as long as they're relatively general-purpose, they should be fine.

If you are unsure, try Ubuntu, Fedora or Linux Mint.

===Windows===

In order to get the tools necessary, you should install the [[Cygwin]] environment. [[MinGW]] or [[DJGPP]] are alternatives, but [https://www.msys2.org/ MSYS2] is strongly suggested as it is the most complete and compatible environment, and also includes a package manager to install libraries and tools.

Microsoft has recently (as of writing) released the Windows Subsystem for Linux as an optional feature for Windows 10. It is basically a real Ubuntu command line distribution running on top of Windows WITHOUT the use of a VM. The latest GCC and Binutils (6.1.0 and 2.27 as of writing) compile and work correctly in this environment. Using the Bash shell, you can access your Windows hard disks through /mnt/<drive letter>. The advantage of this solution is that you can work with whichever Windows or Linux tools that you require, without having to find out if they work in Cygwin. Many of the needed tools can be installed using "apt-get".

For all of the above, it is strongly suggested to build a cross-compiler, not only because the default compilers target different [[Executable Formats|executable format]]s, but because it's generally a good idea. Check the [[GCC Cross-Compiler]] page for details and instructions.

You will also need an editor. Using Notepad will work, but it's easier if you have a more complete editor. For example, [http://notepad-plus-plus.org/ Notepad++] or [http://www.flos-freeware.ch/notepad2.html Notepad2] are used by many people. If you are comfortable with Unix editors, you can pick one from the choice Cygwin provides (which includes e.g. Vim and Emacs, which take some getting-used-to but are very powerful).

It is also possible to use [[Visual Studio]], or the freely downloadable [http://www.microsoft.com/express/ Visual C++ Express Edition], to write and compile your operating system. You will require a special configuration file, and you will certainly be in the minority, but it does work quite well. You can even install the Windows SDK on top, enabling 64 bit development. The only pitfall is this doesn't support Inline Assembly.

Other tools such as [[Watcom]] or [[Borland]] can be used, too, but they each have specific requirements of their own, and are not widely used for this kind of work.

Another consideration is that you will probably have as a goal for your OS to be self-hosting, that is, you can compile your operating system using your operating system. If your OS is written in C, your minimal requirements will therefore be a C compiler and C library. If you intend for your OS to be a Windows clone rather than another POSIX-compliant OS, you will need a C library that does Windows calls instead of POSIX calls, and you will need a C compiler that uses just the C library instead of doing POSIX calls. [http://pdos.org GCCWIN + PDPCLIB] fits this bill.

===MacOS===

Because under the hood it uses FreeBSD's userland, it is fully POSIX compatible. All the usual tools are available (vi, bash, dd, cat, sed, tar, cpio, etc.) Almost every tutorial works out-of-the-box. The missing tools are mostly file system related: no [[Loopback_Device|loopback device]], no fdisk, no mkfs.vfat nor mtools. But you can use diskutil for these purposes, or use [https://brew.sh/ brew] or [https://www.macports.org/ macports] to install those missing tools.

To get gcc, you used to have an mpkg on the 2nd Installation DVD for the older versions. Newer MacOS versions (10.13 and up) can install command line XCode (not the IDE, just the toolchain) by running "xcode-select --install" from a Terminal. This will install gcc, binutils and make. This gcc is actually a masquaraded CLang, but featurefull enough to build your own [[GCC_Cross-Compiler|cross-compiler]] without problems. It is preferred to use the official compiler for bootstraping gcc than to install one from brew or macports.

==Testing your operating system==
{{Main|Testing}}
The above article goes into a lot of depth about choosing how to test your operating system and how to integrate that with your development process. Both physical and emulated testing environments are discussed.

==Protecting your code==
During your code building you will write hundreds, even thousands, of lines of code. You'll spend an unmentionable number of hours, and sit up late at night coding when you really should go to bed. The last thing you need is a disk crash or a poorly written 'rm' or 'format' command throwing all your work away.

What you need is a [https://en.wikipedia.org/wiki/Version_control version control system]. [http://www.nongnu.org/cvs/ CVS] has been used for a number of years, but has gotten a lot of competition from [http://subversion.apache.org/ Subversion], [http://bazaar-vcs.org/ Bazaar], [https://www.mercurial-scm.org// Mercurial], and [http://git-scm.com/ Git] lately. If you can, you should set up a remote computer or server as a version control server, but if you do not have such a machine available you can also host the version control system on your local development computer. Just remember to backup your code to CD or FTP once in a while.

We cannot stress this point strongly enough: if you are not using source control already, you should start doing so immediately. You only need make a serious mistake in your code once to realize the importance of having your code securely versioned and easily retrievable. While it may seem like overkill for a small, private hobby project, once you get into the habit of using revision control, you'll wonder how you ever did without it.

For Git you can create your project on [https://github.com/ GitHub] or [https://bitbucket.org Bitbucket]. Both come with free, private repositories.

An additional benefit of using version control on a network-accessible repository is that it makes it a lot easier to collaborate with and get help from others. This can be quite useful, especially in the forums, as it can avoid the need for constantly posting updated versions of your code to a message thread--you simply point the conversation towards your repository, and the others in the thread will have direct access to your most current changes. It is also crucial if, as the project grows, you begin to work with other developers on the project (just [[Beginner Mistakes#Teamwork|don't expect that to happen overnight]]).

==Common starting points==

The easiest way to get a "Hello World" 64-bit [[Higher Half Kernel|higher half kernel]] going is the [[Limine Bare Bones]] tutorial. A different approach would be to learn how the computer itself starts up, on the [[Boot Sequence]] page.

There are many other [[Tutorials]] available, too.

==Obtaining further knowledge==
There is an amazing amount of knowledge about operating system development available on the Internet today. It's just a matter of finding it. First of all, there is this wiki itself. Amongst others we have lots of [[:Category:Tutorials|Tutorials]]. Since you're here, you've probably already found it. Also on this site is the [http://www.osdev.org/phpBB2/ forum], where many developers hang out and can help you (but make sure you read [[How_To_Ask_Questions|How To Ask Questions]] first). Quite a few books on operating system development have been written. A number of these are featured on our [[Books]] page, and more over at [http://www.osdever.net/tutorials/ osdever.net] as well.

== See Also ==

=== Articles ===
* [[:Category:Bare bones tutorials |Bare bones tutorials]]
* [[Resources]]
* [[What order should I make things in]]?

=== Threads ===

=== External Links ===
* [http://www.osdever.net/ Bona Fide OS Development] contains several tutorials and documentation.
* [http://www.informit.com/articles/article.aspx?p=26396 Implementing an Operating System] by Andrew S. Tanenbaum.

[[Category:OS Development]]
````

<a id="w001750"></a>

## W001750 — Babystep

Bron-ID: W001750
Namespace: Main (0)
Revisie: 2790 | Datum: 2007-01-13T01:22:54Z
Bron: https://wiki.osdev.org/Babystep
Revisie-URL: https://wiki.osdev.org/index.php?title=Babystep&oldid=2790
Doorverwijzing: Category:Babystep

````text
#REDIRECT [[:Category:Babystep]]
````

<a id="w001751"></a>

## W001751 — Category:Tutorials

Bron-ID: W001751
Namespace: Category (14)
Revisie: 6683 | Datum: 2008-11-18T11:36:45Z
Bron: https://wiki.osdev.org/Category:Tutorials
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ATutorials&oldid=6683

````text
This category contain articles that are tutorials. A tutorial shows how to solve a concrete problem or do something concrete. You can also see the [[Tutorials|tutorials with ratings]].

[[Category:OS Development]]
````

<a id="w001752"></a>

## W001752 — Semaphore

Bron-ID: W001752
Namespace: Main (0)
Revisie: 11881 | Datum: 2011-08-06T15:34:51Z
Bron: https://wiki.osdev.org/Semaphore
Revisie-URL: https://wiki.osdev.org/index.php?title=Semaphore&oldid=11881

````text
A '''semaphore''' is a [[:Category:Synchronization|synchronization]] primitive data type.  From the programmer's perspective, it is an opaque data type with two defined operations, usually called '''wait''' and '''signal'''.  There are two types of semaphore -- the '''binary semaphore''' and the '''counting semaphore'''.

== Counting semaphores ==
The most general type of semaphore is the counting semaphore.  It can be thought of as a single integer variable.  When the '''signal''' method is called the variable is [[atomic operation|atomically]] incremented.  When the '''wait''' method is called, the process waits until the variable is non-zero, and then atomically decrements it before returning.

== Binary semaphores ==
A binary semaphore, also known as Mutex, is like a counting semaphore, except that the '''signal''' method sets the value of the semaphore to 1, rather than incrementing it.

== Usage ==
Binary semaphores are most often used to guarantee [[Mutual Exclusion|mutual exclusion]]. In this case, the semaphore value is initialized to 1, and an application about to enter a mutually exclusive section of code calls '''wait''', and then calls '''signal''' after exiting the code.

Counting semaphores can be used as the building blocks to implement many synchronization models. The typical example of this is a statically allocated queue, as in the example below.

<source lang="cpp">
Message queue[N];
Semaphore slots = new Semaphore(N);
Semaphore messages = new Semaphore(0);
int last_read = 0
int last_written = 0;

Message get()
{
   Message m;
   messages.wait();
   
   m = queue[last_read];
   last_read = (last_read+1)%N;
  
   slots.signal();
   return m;
}

void put(Message m)
{
   slots.wait();
   
   queue[last_written] = m;
   last_written = (last_written+1)%N;
   
   messages.signal();
}
</source>

In the example above, the functions could use error codes to tell the caller when the array is out of space and when the queue is empty (or an assertion that terminated the program/crashed the kernel on lack of space and trying to get() from an empty queue). However, in many situations you simply want the current thread to block until the actual request can be performed. These are the cases where you want a semaphore.

There are many (many many many) algorithms that use semaphores as synchronization primitives. When the semaphores are used to synchronize multiple threads of the same process, it's common for this kind of algorithms to combine binary (mutexes) and counting semaphores. This is because the code that manipulates the data structures still has to be mutual exclusive, even when semaphores are used to control resource usage.

== Implementation ==
While the description of the semaphore is very simple, implementation is not as easy as it might at first sound.  First of all, ensuring the two methods operate atomically is not always easy; secondly, for the purpose of efficiency, a thread calling '''wait''' on a semaphore whose value is 0 should be removed from the scheduler queue, and only added to it again when the semaphore is signalled.

Therefore, the usual implementation of a semaphore is as a kernel-level object which uses another more primitive form of locking (e.g. a [[spinlock]]) to ensure that only one thread may access it at once.  In addition to the integer variable, it also contains a queue of threads that are waiting for access.  Whenever '''signal''' is called, the thread at the front of the queue is woken and rescheduled immediately after incrementing the variable (or, in some cases, the variable is never incremented at all, and the woken thread skips the stage of decrementing it; this ensures that the woken thread progresses rather than some new arrival).  Whenever '''wait''' cannot immediately progress, the current thread is suspended and added to the queue at an appropriate point (e.g. at the end, or in a position correct for the thread's priority).

NB There is classic race condition here called the lost wake-up problem. For a solution to this problem please read this article http://www.linuxjournal.com/article/8144 .

== See Also ==

=== Articles ===

=== Threads ===

=== External Links ===
* [[wikipedia:Semaphore_(programming)|Semaphore]] on Wikipedia

[[Category:Synchronization]]
[[Category:IPC]]
````

<a id="w001753"></a>

## W001753 — Talk:Serial Ports

Bron-ID: W001753
Namespace: Talk (1)
Revisie: 24029 | Datum: 2019-10-05T03:48:19Z
Bron: https://wiki.osdev.org/Talk:Serial_Ports
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ASerial_Ports&oldid=24029

````text
==Detection==
This section needs details on exactly how to detect serial ports using legacy device techniques. It also doesn't mention IRG 3/4, if im not mistaken... do these not report incoming data or is that a BIOS thing? --[[User:Tyler|Tyler]] 06:51, 9 January 2007 (CST)

== Baud rate ==
I'm not SerialPorts guru, but bauds is usually symbols per second on the wire, and afaik, each symbol on a RS-232 line will only carry 1 bit, so the baud rate (here) is more likely to be the bitrate than the byte-per-second speed ... note however than modems typically send many bits per baud (because you may have more than 2 waveforms used), but not necessarily 8 of them ... Someone should check the specs and confirm/correct. - [[User:PypeClicker|PypeClicker]] ''(copy from osfaq - [[User:Combuster|Combuster]] 04:05, 20 January 2007 (CST))''

== Pages to convert ==
PLEASE mark the page as converted next time, you had me do almost everything a second time. - [[User:Combuster|Combuster]] 04:40, 20 January 2007 (CST)

== Category ==

This page current only has one category - network hardware... The page doesn't just cover networking with serial ports though. Suggest adding another relevant category? --[[User:Pcmattman|pcmattman]] 03:36, 4 April 2009 (UTC)
:I added it to Common Devices. --[[User:Troy martin|Troy Martin]] 16:35, 4 April 2009 (UTC)

== Sending Data Example ==

Under the "Example Code" section, under "Sending Data," it has <code>return inb(PORT + 5) & 0x20;</code> as the body for <code>is_transmit_empty</code> - is that meant to be <code>return inb(PORT + 5) & 0x2;</code>? --[[User:Retrocat|Retrocat]] 21:29, 26 May 2018 (CDT)

== Should the article be updated to be a little more generalised? ==

Should this article be updated to provide a more general view of serial/uart communication hardware? It seems to be very x86-centric and doesn't provide a very generalised view of the topic.
For my personal perspective, I've put in a little more time working with UART communication in embedded systems than with serial ports in X86 systems, but I've created simple 8250 compatible drivers before. 'uart' redirects to this article, should it not be the other way around? It doesn't tell the reader much about what specific kind of controller hardware that x86 uses or where to find datasheets for it. My suggestions specifically is just that the wording be changed slightly to reflect that the article deals specifically with the 8250/16550 UART on x86 systems. I think this would be helpful not only because it is more technically correct, but because it will encourage the reader to think in the correct terms: That what they are doing is writing a driver for a specific piece of hardware.
I'm happy to make the changes myself if anyone agrees. --[[User:Ajxs|Ajxs]] 04:00, 27 August 2019 (CDT)

+1, although "changed slightly" may not be the best way. This page starts out covering serial ports in general, but then gets 8250-specific, and then x86 specific. Instead, it should be three separate pages - I was going to do precisely this, especially 1) below, but then it got all too big:

<ol>
<li>Serial ports in general:</li>
<ol type="a">
<li>What a serial port is for;</li>
<li>Synchronous vs Asynchronous;</li>
<li>Common standards, including RS-232, RS-422 and RS-485.</li>
</ol>
<li>The 8250/16450/16550/16650/16x60 UARTs:</li>
<ol type="a">
<li>The various registers (some hat-tip to x86 port convention is appropriate here);</li>
<li>The FIFOs in the later versions</li>
</ol>
<li>The standard COM ports on the x86</li>
<ol type="a">
<li>Their conventional port assignments</li>
<li>When the PIC should get the EOI</li>
</ol>
</ol>

--[[User:Johnburger|John Burger]] 22:04, 26 August 2019 (CDT)

: As stated above, I'd be happy to help. Perhaps we could divide the work among ourselves.
: I'd be happy to begin work on a generalised 'uart' page if you think that would be a helpful place to start. You seem very knowledgeable on the subject, if you could suggest any points to be sure that I addressed that would be appreciated. I think that a 'uart' page should address the basic concept of what a UART is. I feel addressing the topic within the context of hardware would be a good place to start. Then providing some helpful practical information on how this relates to x86 osdev. --[[User:Ajxs|Ajxs]] 04:00, 27 August 2019 (CDT)

:: By all means. To that end, I've started a page called "Communications". I envisage this to be a "Level 0" page used to descend down the complex hierarchy of communications standards. We can link the "Serial Ports" and "UART" pages to this. I'll be writing a more general "Serial Port" page soon. --[[User:Johnburger|John Burger]] 00:42, 28 August 2019 (CDT)

::: I've written a draft UART page, which can be found here [[User:Ajxs/uart]]. Any review would be appreciated. I've decided not to include any specific programming information, preferring to introduce more specific articles related to programming an individual implementation. Currently I haven't done any such introduction, simply linked to this article in the 'related articles' section. If you're happy with this, I'll move this to the [[UART]] page. --[[User:Ajxs|Ajxs]] 00:44, 31 August 2019 (CDT)

:::: A concise summary of the topic. By all means submit it! --[[User:Johnburger|John Burger]] 07:23, 2 September 2019 (CDT)

::::: Thank you! Done! From here I'll try and formulate a good way to rewrite the programming parts of this page to introduce the 16550 specifically. I'll make any edits in a personal page to allow for more possibility of review. Thanks again from another Australian osdever! --[[User:Ajxs|Ajxs]] 07:35, 2 September 2019 (CDT)

== Modem control register use in initialization code ==

The initialization code provided sets the RTS and DSR bits in the Modem control register. Is there any significance of this to the article? The article does not go into any detail about modem control, so this could potentially be removed.
The comment also suggests: "IRQs enabled, RTS/DSR set". From reading the 16550D datasheet, none of the bits in the modem control register seem to correspond to IRQ generation. Can this line be removed? It seems pretty harmless but it looks like it's been copied from a different context, and risks being blindly copied into others. --[[User:Ajxs|Ajxs]] 22:48, 4 October 2019 (CDT)
````

<a id="w001754"></a>

## W001754 — Bootable CD

Bron-ID: W001754
Namespace: Main (0)
Revisie: 30479 | Datum: 2026-04-12T16:55:30Z
Bron: https://wiki.osdev.org/Bootable_CD
Revisie-URL: https://wiki.osdev.org/index.php?title=Bootable_CD&oldid=30479

````text
This brief tutorial is about how to create your own bootable CD. For information about bootable CDs see [[El-Torito]] and [[ISO 9660]]
{{Rating|1}}

== Requirements ==
* [[mkisofs]]
* A CD burning program
* A 1.44M floppy image, 1474560 bytes to be exact. See [[Disk Images]] for a list of programs that can help on this.

== How it works ==

First, lets go to the directory where your floppy image (floppy.img) is located:
<syntaxhighlight lang="shell-session">
$ cd ~/myosname
</syntaxhighlight>

First we create a directory to store the contents of the CD:
<syntaxhighlight lang="shell-session">
$ mkdir cdcontents
</syntaxhighlight>

Next we copy everything we want on the CD into here
<syntaxhighlight lang="shell-session">
$ cp files/* cdcontents
</syntaxhighlight>

You can create extra subdirectories if necessary. After that, we need to copy our floppy image in there as well
<syntaxhighlight lang="shell-session">
$ cp floppy.img cdcontents
</syntaxhighlight>

Now our directory stores all we need. We can continue creating the CD image:
<syntaxhighlight lang="shell-session">
$ mkisofs -o mybootcd.iso -V MyOSName -b floppy.img cdcontents
</syntaxhighlight>

This will create the CD image:
{| class="wikitable"
|-
| -o mybootcd.iso
| send output to mybootcd.iso
|-
| -V MyOSName
| the CD's volume label is MyOSName
|-
| -b floppy.img
| this is the boot image. Note that its both located in the current directory and in the CD's directory
|-
| cdcontents
| this is the directory where all the files are to be taken from
|}

If you have done everything correct, a CD image is created in the current directory. You can burn this image to CD with your favorite CD burner application.

== See Also ==
=== Articles ===
* [[Bootable El-Torito CD with GRUB Legacy]] - how to create a "no emulation" bootable CD

[[Category:Booting]]
[[Category:Disk Image Utilities]]
````

<a id="w001757"></a>

## W001757 — User talk:Xyz1

Bron-ID: W001757
Namespace: User talk (3)
Revisie: 2748 | Datum: 2007-01-10T08:19:30Z
Bron: https://wiki.osdev.org/User_talk:Xyz1
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AXyz1&oldid=2748

````text
I have moved your pages to [[User:Xyz1/Describe new OS project]] and [[User:Xyz1/OS projects]] as they currently aren't working with the rest of the wiki.   Let me know if you have any questions. -[[User:Jhawthorn|Jhawthorn]] 02:19, 10 January 2007 (CST)
````

<a id="w001758"></a>

## W001758 — Category:Synchronization

Bron-ID: W001758
Namespace: Category (14)
Revisie: 2766 | Datum: 2007-01-11T11:50:52Z
Bron: https://wiki.osdev.org/Category:Synchronization
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ASynchronization&oldid=2766

````text
[[Category:Processes and Threads]]
````

<a id="w001759"></a>

## W001759 — Calling Conventions

Bron-ID: W001759
Namespace: Main (0)
Revisie: 29385 | Datum: 2025-02-08T13:33:55Z
Bron: https://wiki.osdev.org/Calling_Conventions
Revisie-URL: https://wiki.osdev.org/index.php?title=Calling_Conventions&oldid=29385

````text
Calling external functions in C, and calling C functions from other languages, is a common issue in OS programming, especially where the other language is assembly. This page will concentrate primarily on the latter case, but some consideration is made for other languages as well.

Some of what is described here is imposed by the x86 architecture, some is special to the GNU [[GCC]] toolchain. Some is configurable, and you could be  making your own GCC target to support a different calling convention. Currently, this page makes no effort of differentiating which is what.

==Basics==
As a general rule, a function which follows the C calling conventions, and is appropriately declared (see below) in the C headers, can be called as a normal C function. Most of the burden for following the calling rules falls upon the assembly program.

== Cheat Sheets ==

Here is a quick overview of common calling conventions. Note that the calling conventions are usually more complex than represented here (for instance, how is a large struct returned? How about a struct that fits in two registers? How about <code>va_list</code>'s?). Look up the specifications if you want to be certain. It may be useful to write a test function and use <code>gcc -S</code> to see how the compiler generates code, which may give a hint of how the calling convention specification should be interpreted.

{| {{wikitable}}
! Platform
! Return Value
! Parameter Registers
! Additional Parameters
! Stack Alignment
! Scratch Registers
! Preserved Registers
! Call List
|-
| System V i386 || eax, edx || none || stack (right to left)<sup>[[#Note1|1]]</sup> || || eax, ecx, edx || ebx, esi, edi, ebp, esp || ebp
|-
| System V X86_64<sup>[[#Note2|2]]</sup> || rax, rdx || rdi, rsi, rdx, rcx, r8, r9 || stack (right to left)<sup>[[#Note1|1]]</sup>  || 16-byte at call<sup>[[#Note3|3]]</sup> || rax, rdi, rsi, rdx, rcx, r8, r9, r10, r11 || rbx, rsp, rbp, r12, r13, r14, r15 || rbp
|-
| Microsoft x64,<br>UEFI X64 || rax || rcx, rdx, r8, r9 || stack (right to left)<sup>[[#Note1|1]]</sup>  || 16-byte at call<sup>[[#Note3|3]]</sup> || rax, rcx, rdx, r8, r9, r10, r11 || rbx, rdi, rsi, rsp, rbp, r12, r13, r14, r15 || rbp
|-
| ARM (32-bit) || r0, r1 || r0, r1, r2, r3 || stack || 8 byte<sup>[[#Note4|4]]</sup> || r0, r1, r2, r3, r12 || r4, r5, r6, r7, r8, r9, r10, r11, r13, r14 ||
|}

<small id="Note2">Note 1: The called function is allowed to modify the arguments on the stack and the caller must not assume the stack parameters are preserved. The caller should clean up the stack.</small>

<small id="Note2">Note 2: There is a 128 byte area below the stack called the 'red zone', which may be used by leaf functions without increasing %rsp. This requires the kernel to increase %rsp by an additional 128 bytes upon signals in user-space. This is <em>not</em> done by the CPU - if interrupts use the current stack (as with kernel code), and the red zone is enabled (default), then interrupts will silently corrupt the stack. Always pass <code>-mno-red-zone</code> to kernel code (even support libraries such as libc's embedded in the kernel) if interrupts don't respect the red zone.</small>

<small id="Note3">Note 3: Stack is 16 byte aligned at time of call. The call pushes %rip, so the stack is 16-byte aligned again if the callee pushes %rbp.</small>

<small id="Note4">Note 4: Stack is 8 byte aligned at all times outside of prologue/epilogue of function.</small>

== System V ABI ==
{{Main|System V ABI}}

The System V ABI is one of the major ABIs in use today and is virtually universal among Unix systems. It is the calling convention used by toolchains such as <tt>i686-elf-gcc</tt> and <tt>x86_64-elf-gcc</tt>.

==External References==
In order to call a foreign function from C, it must have a correct C prototype. Thus, is if the function <tt>fee()</tt> takes the arguments fie, foe, and fum, in C calling order, and returns an integer value, then the corresponding header file should have the following prototype:

<syntaxhighlight lang="c">
int fee(int fie, char foe, double fum);
</syntaxhighlight>

Similarly, an global variables in the assembly code must be declared <tt>extern</tt>:

<syntaxhighlight lang="c">
extern int frotz;
</syntaxhighlight>

C functions in assembly or other languages must be declared as appropriate for the language. For example, in NASM, the C function

<syntaxhighlight lang="c">
int foo(int bar, char baz, double quux);
</syntaxhighlight>

would be declared

<syntaxhighlight lang="c">
extern foo
</syntaxhighlight>

Also, in most assembly languages, a function or variable that it to be exported must be declared global:

<syntaxhighlight lang="asm">
global foo
global frotz
</syntaxhighlight>

==Name Mangling==

In some object formats ([[a.out]]), the name of a C function is automagically mangled by prepending it with an underscore ("_"). Thus, to call a C function <tt>foo()</tt> in assembly with such a format, you must define it as <tt>extern _foo</tt> instead of <tt>extern foo</tt>. This requirement does not apply to most modern formats such as [[COFF]], [[PE]], and [[ELF]].

C++ name mangling is much more severe, as the C++ compiler encodes the type information from the parameter list into the symbol. (This is what enables function overloading in C++ in the first place.) The Binutils package contains the tool <tt>c++filt</tt> that can be used to determine the correct mangled name.

==Registers==
The general register <tt>EBX</tt>, <tt>ESI</tt>, <tt>EDI</tt>, <tt>EBP</tt>, <tt>DS</tt>, <tt>ES</tt>, and <tt>SS</tt>, must be preserved by the called function. If you use them, you must save them first and restore them afterwards. Conversely, <tt>EAX</tt> and <tt>EDX</tt> are used for return values, and thus should not be preserved. The other registers do not need to be saved by the called function, but if they are in use by the calling function, then the calling function should save them before the call is made, and restored afterwards.

==Passing Function Arguments==
GCC/x86 passes function arguments on the stack. These arguments are pushed in reverse order from their order in the argument list. Furthermore, since the x86 protected-mode stack operations operate on 32-bit values, the values are always pushed as a 32-bit value, even if the actual value is less than a full 32-bit value. Thus, for function <tt>foo()</tt>, the value of <tt>quux</tt> (a 48-bit FP value) is pushed first as two 32-bit values, low-32-bit-value first; the value of <tt>baz</tt> is pushed as the first byte of in 32-bit value; and then finally <tt>bar</tt> is pushed as a 32-bit value.

To pass arguments to a C function, the calling function must push the argument values as described above. Thus, to call foo() from a [[NASM]] assembly program, you would do something like this

<syntaxhighlight lang="asm">
push eax   ; low 32-bit of quux
push edx   ; high 32-bit of quux
push bl    ; baz
push ecx   ; bar
call foo
</syntaxhighlight>

==Accessing Function Arguments==
In the GCC/x86 C calling convention, the first thing any function that accepts formal arguments should do is push the value of <tt>EBP</tt> (the frame base pointer of the calling function), then copy the value of <tt>ESP</tt> to <tt>EBP</tt>. This sets the function's own frame pointer, which is used to track both the arguments and (in C, or in any properly reentrant assembly code) the local variables.

To access arguments passed by a C function, you need to use the <tt>EBP</tt> an offset equal to 4 * (n + 2), where n is the number of the parameter in the argument list (not the number in the order it was pushed by), zero-indexed. The + 2 is an added offset for the calling function's saved frame pointer and return pointer (pushed automatically by <tt>CALL</tt>, and popped by <tt>RET</tt>).

Thus, in function <tt>fee</tt>, to move <tt>fie</tt> into <tt>EAX</tt>, <tt>foe</tt> into <tt>BL</tt>, and <tt>fum</tt> into <tt>EAX</tt> and <tt>EDX</tt>, you would write (in NASM):

<syntaxhighlight lang="asm">
mov ecx, [ebp + 8]  ; fie
mov bl,  [ebp + 12] ; foe
mov edx, [ebp + 16] ; low 32-bit of fum
mov eax, [ebp + 20] ; high 32-bit of fum
</syntaxhighlight>

As stated earlier, return values in GCC are passed using <tt>EAX</tt> and <tt>EDX</tt>. If a value exceeds 64 bits, it must be passed as a pointer.

==See Also==
===External Links===
*[http://www.delorie.com/djgpp/doc/ug/asm/calling.html DJGPP FAQ: GCC calling conventions]
*[http://gul.ime.usp.br/Docs/docs/howto/other-formats/html/HOWTO-INDEX-html/Assembly-HOWTO-5.html Linux Assembly Language HOWTO chapter 5]
*http://files.osdev.org/mirrors/geezer/osd/libc/index.htm

[[Category:ABI]]
[[Category:C]]
[[de:Aufrufkonventionen]]
````

<a id="w001760"></a>

## W001760 — User:Xyz1/Main Page

Bron-ID: W001760
Namespace: User (2)
Revisie: 2755 | Datum: 2007-01-11T05:56:29Z
Bron: https://wiki.osdev.org/User:Xyz1/Main_Page
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AXyz1%2FMain_Page&oldid=2755

````text
<div id="mainpage"></div> __NOTOC__ __NOEDITSECTION__

<!-- Welcome to OSDev.org box -->
{{Welcome}}

<!-- Conversion from OSFAQ wiki to MediaWiki box -->
{{Conversion}}

<!-- Table stuff -->
{| style="width: 100%; margin: 0; padding: 0; border: 0; border-collapse: collapse;"
| style="padding: 0; width: 25%; vertical-align: top;" |

<!-- First column -->
{{Box|subject= OS development}}
{{Box|subject= Resources (Test)}}
{{Box|subject= OS projects}} 

| style="padding: 0 0 0 10px; width: 25%; vertical-align: top;" |

<!-- Second column -->
{{Box|subject= Languages}}
{{Box|subject= Tools}}

| style="padding: 0 0 0 10px; width: 25%; vertical-align: top;" |

<!-- Third column -->
{{Box|subject= Hardware}}

| style="padding: 0 0 0 10px; width: 25%; vertical-align: top;" |

<!-- Forth column -->
{{Box|subject= OS theory}}

|}
````

<a id="w001763"></a>

## W001763 — Remote Procedure Call

Bron-ID: W001763
Namespace: Main (0)
Revisie: 13503 | Datum: 2012-06-05T09:18:12Z
Bron: https://wiki.osdev.org/Remote_Procedure_Call
Revisie-URL: https://wiki.osdev.org/index.php?title=Remote_Procedure_Call&oldid=13503

````text
== What is RPC ==

Remote Procedure Calls, or RPC's for short, are a means to have certain services executed by some code that lies in an different process (maybe even on a machine located somewhere in the network), instead of being available locally. This is a common practice, employed e.g. by application servers or web applications.

Popular protocols for RPC's are COM+, Corba, or SOAP.

== Remote Procedure Calls in a nutshell ==

An RPC system relies on a compiler that will generate helper code from a high-level interface description (usually IDL or ''Interface Description Language''). The 'client' code that wishes to call a service will actually call a proxy function locally, passing it the arguments for the remote code. The proxy then marshalls (or serializes) the arguments according so that they fit in a flat buffer and send that buffer to the server code (for instance involving a UDP packet or some local message queue) and waits for response.

The server will suddenly receive the marshalled packet and pass it to a stub that will recreate the data (unmarshall) and call the real service procedure, retrieving the results and marshall them back in a message to the client.

<pre>
 ( client )                                    ( server )
      |                                            ^
      | sayHello("World")                          | serverSayHello("World")
      V                                            |
   +------\     -  -  - <channel>-  -  -  -   \-------+
   | PROXY > ->  iXX mYY Str'W o r l d \0'  -> > STUB |
   +------/     -  -  -  -  - |-  -  -  -     /-------+
                              |
                   marshalled message
       ^                                           ^
      /_\                                         /_\
       :                                           :
       :                                           :
       :               |------------|\             :
       :               | .idl file  L_\            :
       +  - compiler - | (describing  | - compiler +
                       |  interface)  |
                       +--------------+
</pre>

[[Category:IPC]]
````

<a id="w001764"></a>

## W001764 — RPC

Bron-ID: W001764
Namespace: Main (0)
Revisie: 2758 | Datum: 2007-01-11T10:19:20Z
Bron: https://wiki.osdev.org/RPC
Revisie-URL: https://wiki.osdev.org/index.php?title=RPC&oldid=2758
Doorverwijzing: Remote Procedure Call

````text
#REDIRECT [[Remote Procedure Call]]
````

<a id="w001765"></a>

## W001765 — Shared Memory

Bron-ID: W001765
Namespace: Main (0)
Revisie: 28212 | Datum: 2023-07-10T18:54:12Z
Bron: https://wiki.osdev.org/Shared_Memory
Revisie-URL: https://wiki.osdev.org/index.php?title=Shared_Memory&oldid=28212

````text
Shared memory allows other programs to influence each other by sharing a piece of memory

== Concept ==

The concept of memory protection has become a well-accepted one in OS design. A special CPU component, the [[MMU]] (Memory Management Unit), provides each user-space application with a "virtual" address space. Each application "thinks" it has the whole address space for itself.

In reality, the MMU maps each "virtual" address to a "physical" one, usually in units of "pages" (4k or 2 or 4MB on the IA32 architecture). Page 0 of one application is mapped to physical 0x0007 0000, while page 0 of another application is mapped to physical 0x0008 0000. You get the idea.

This means that applications cannot communicate with (pass data to) each other directly, because they are "virtually alone" in their address space: They have to ask the OS [[kernel]] to pass a message (data). (Only the kernel and services in the kernel address space can see the real address spaces)

== Advantages ==

Now, [[Message Passing]] can be costly, in terms of clock cycles. If two [[Process|processes]] intend to closely cooperate, the penalty of passing messages back and forth can become significant. An alternative would be to share memory.

To do this, the kernel is asked to map a certain address range of process A to the same physical address as a similar sized address range of process B. This address range is thus shared between process A and process B.

== Considerations ==

There are many things to consider here: Authentication, for example. Or that, if the shared physical memory is not mapped to identical virtual address ranges in process A and process B, any pointers in the shared memory structures must be offsets instead of absolute pointers, or they will no longer point to the same thing in the other process' address space. Also, on a multiprocessor system, checking to see whether the other process is accessing the memory at the same time is important.

[[Category:IPC]]
[[Category:Memory management]]
[[Category:Virtual Memory]]
````

<a id="w001766"></a>

## W001766 — Atomic operation

Bron-ID: W001766
Namespace: Main (0)
Revisie: 24056 | Datum: 2019-10-13T12:47:58Z
Bron: https://wiki.osdev.org/Atomic_operation
Revisie-URL: https://wiki.osdev.org/index.php?title=Atomic_operation&oldid=24056

````text
An '''atomic operation''' is an operation that will always be executed without any other process being able to read or change state that is read or changed during the operation.  It is effectively executed as a single step, and is an important quality in a number of algorithms that deal with multiple independent processes, both in synchronization and algorithms that update shared data without requiring synchronization.

== Methods for ensuring an operation is atomic ==

=== Single processor-single core systems ===

On a single processor system, if an operation is implemented in a single CPU instruction, it is always atomic.  Therefore it is safe to assume that operations like XCHG or INC are atomic on such systems.

If an operation requires multiple CPU instructions, then it may be interrupted in the middle of executing.  If this results in a context switch (or if the interrupt handler refers to data that was being used) then atomicity could be compromised.  It is possible to use any standard locking technique (e.g. a [[spinlock]]) to prevent this, but may be inefficient.  If it is possible, disabling interrupts may be the most efficient method of ensuring atomicity (although note that this may increase the worst-case interrupt latency, which could be problematic if it becomes too long).

=== Multiprocessor or multicore systems ===

On multiprocessor systems, ensuring atomicity exists is a little harder.  It is still possible to use a lock (e.g. a [[spinlock]]) the same as on single processor systems, but merely using a single instruction or disabling interrupts will not guarantee atomic access.  You must also ensure that no other processor or core in the system attempts to access the data you are working with.  The easiest way to achieve this is to ensure that the instructions you are using assert the 'LOCK' signal on the bus, which prevents any other processor in the system from accessing the memory at the same time.  On x86 processors, some instructions automatically lock the bus (e.g. 'XCHG') while others require you to specify a 'LOCK' prefix to the instruction to achieve this (e.g. 'CMPXCHG', which you should write as 'LOCK CMPXCHG op1, op2').

== Implementation ==

A few of the [[Synchronization Primitives]] (sub)pages have implementations or examples of atomic operations. Besides that, GCC can make your life easier because it has built-in functions for them. You can take a look at the [http://gcc.gnu.org/onlinedocs/gcc/Atomic-Builtins.html GCC Documentation].

[[Category:Synchronization]]
````

<a id="w001767"></a>

## W001767 — Category talk:Synchronization

Bron-ID: W001767
Namespace: Category talk (15)
Revisie: 2769 | Datum: 2007-01-12T05:32:07Z
Bron: https://wiki.osdev.org/Category_talk:Synchronization
Revisie-URL: https://wiki.osdev.org/index.php?title=Category_talk%3ASynchronization&oldid=2769

````text
== Category hierarchy ==
I think this category should be a subclass to [[:Category:Processes and Threads|Processes and Threads]] rather than to [[:Category:IPC|IPC]] as it influences far more than just IPC. I moved it, but if you disagree you can revert it and post your opinion here. - [[User:Combuster|Combuster]] 05:49, 11 January 2007 (CST)
:Agree 100% I considered but avoided changing it simply because the OSFAQ has it under [[:Category:IPC|IPC]]. -[[User:Jhawthorn|Jhawthorn]] 23:32, 11 January 2007 (CST)
````

<a id="w001768"></a>

## W001768 — Talk:NTFS

Bron-ID: W001768
Namespace: Talk (1)
Revisie: 12845 | Datum: 2012-02-28T08:02:52Z
Bron: https://wiki.osdev.org/Talk:NTFS
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ANTFS&oldid=12845

````text
"For the sake of performance and compatibility with some (especially Japanese) machines, NTFS allows sectors of other sizes."
What's so different about Japanese machines? Same hardware, right?
--[[User:MessiahAndrw|MessiahAndrw]] 19:09, 11 January 2007 (CST)

== Field sizes ==

I pause a bit when looking at the perpetuation of "unsigned short, 2 byte". It's itching me to change this shebang into "proper" types (i.e., uint16_t). However, I have positively <tt>no</tt> idea if "byte" should be signed or not...?!? {suppressing maniacal laughter} -- [[User:Solar|Solar]] 01:55, 28 February 2012 (CST)

:Never mind. I feel better now. -- [[User:Solar|Solar]] 02:02, 28 February 2012 (CST)
````

<a id="w001769"></a>

## W001769 — Babystep1

Bron-ID: W001769
Namespace: Main (0)
Revisie: 25013 | Datum: 2020-07-15T13:33:11Z
Bron: https://wiki.osdev.org/Babystep1
Revisie-URL: https://wiki.osdev.org/index.php?title=Babystep1&oldid=25013

````text
__NOTOC__

{{Tone}}
{{Rating|1}}

{{Infobox Tutorial
| name=Babystep1: Your first boot sector
| next=[[Babystep2]]
}}

=== Your first boot sector. ===

==Code==
The following code is the smallest possible example of booting code from a disk.

<source lang="asm">
; boot.asm
hang:
    jmp hang

    times 512-($-$$) db 0
</source>
The CPU starts in real mode and the [[BIOS]] loads this code at address 0000:7c00. "times 512-($-$$) db 0" is NASM's way of saying fill up 512 bytes with zeroes, and partcopy is going to expect that (200 in Hex = 512 in Decimal). Change it and partcopy will likely fail.

There is often a boot signature (0xAA55) at the end. Older versions of some BIOSes looked for this in order to identify a boot sector on a disk. It is evidently unnecessary nowadays, unless you're running the code on a legacy BIOS, or in QEMU. If it's needed, the last line would be replaced with (or some version of it):
<source lang="asm">
; boot.asm
hang:
    jmp hang

    times 510-($-$$) db 0 ; 2 bytes less now
    db 0x55
    db 0xAA
</source>

Once you've booted, and the cursor is happily blinking on a blank screen, the disk's motor will turn off and you can now press Ctrl-Alt-Del to reboot. This is because [[interrupts]] are still being generated.

Try clearing the interrupts flag:
<source lang="asm">
;boot.asm
     cli
 hang:
     jmp hang

     times 510-($-$$) db 0
     db 0x55
     db 0xAA
</source>
You may notice that the motor doesn't turn off and you can't reboot with Ctrl-Alt-Del.

Removing the loop and merely padding out the sector with zeroes will usually cause the BIOS to throw an error on boot. On most machines, it will say "Operating System Not Found".

==Creating disk image==
The code is assembled in [[NASM]] and copied to floppy (outdated), disk or USB-sticks using partcopy, dd, or debug. Then you simply boot from that disk.

For a more detailed description, see the [[Bootable Disk]] page.

===Windows===
<source lang="bash">
nasmw boot.asm -f bin -o boot.bin
partcopy boot.bin 0 200 -f0 
OR
debug boot.bin
-W 100 0 0 1
-Q</source>
===Unix===
<source lang="bash">
nasm boot.asm -f bin -o boot.bin
dd if=boot.bin of=/dev/fd0
</source>
To write to a hard drive or USB-stick, use
<source lang="bash">
nasm boot.asm -f bin -o boot.bin
dd if=boot.bin of=/dev/sda
</source>
where replace "sda" with sdb, sdc etc. according to your configuration.

==Running the binary in QEMU==
If you don't have an old machine with floppy drive you can emulate one using QEMU (note "'''f'''da").

<source lang="bash">
qemu-system-i386 -fda boot.bin
</source>

But it is advisable to forget about floppies altogether, and focus on USB-sticks instead. Also if you're afraid to test your code on your development machine (that would be wise), you can use QEMU (note "'''h'''da").

<source lang="bash">
qemu-system-i386 -hda boot.bin
</source>

Use the QEMU monitor command to send Ctrl-Alt-Del to the VM:
<source lang="bash">
sendkey ctrl-alt-delete
</source>

Because of how fast emulation has become, you might need to slow down emulation speed to 1% to notice the reboots.

== References ==
* Instruction Set from the horse's mouth: [http://www.intel.com/design/pentiumII/documentation.htm]
* Easier to read: [http://www.baldwin.cx/386htm/toc.htm]
* NASM asembler - docs incl instruction set: [http://sourceforge.net/projects/nasm]
* Partcopy - download pcopy02.zip (new link): [http://www.osdever.net/downloads.php]
* Interrupts by number: [http://www.osdever.net/downloads.php]
* Randall Hyde's look into the bowels of the PC: [http://webster.cs.ucr.edu/]
* QEMU [https://www.qemu.org]

[[Category:Babystep]]
[[Category:Bootloaders]]
````

<a id="w001770"></a>

## W001770 — Babystep2

Bron-ID: W001770
Namespace: Main (0)
Revisie: 26162 | Datum: 2021-06-28T19:56:03Z
Bron: https://wiki.osdev.org/Babystep2
Revisie-URL: https://wiki.osdev.org/index.php?title=Babystep2&oldid=26162

````text
__NOTOC__

{{Infobox Tutorial
| name=Babystep2: Writing a message using the BIOS
| prev=[[Babystep1]]
| next=[[Babystep3]]
}}

=== Writing a message using the BIOS ===

Quick review:
# Boot sector loaded by BIOS is 512 bytes
# The code in the boot sector of the disk is loaded by the BIOS at 0000:7c00
# Machine starts in [http://www.osdev.org/wiki/Real_Mode Real Mode]
# Be aware that the CPU is being interrupted unless you issue the [[CLI]] assembly command

Many (but not all) BIOS interrupts expect DS to be filled with a Real Mode segment value. This is why many BIOS interrupts won't work in protected mode. So if you want to use int 10h/ah=0eh to print to the screen, then you need to make sure that your seg:offset for the characters to print is correct.

In real mode, addresses are calculated as segment * 16 + offset. Since offset can be much larger than 16, there are many pairs of segment and offset that point to the same address. For instance, some say that the bootloader is is loaded at 0000:7C00, while others say 07C0:0000. This is in fact the same address: 16 * 0x0000 + 0x7C00 = 16 * 0x07C0 + 0x0000 = 0x7C00.

It doesn't matter if you use 0000:7c00 or 07c0:0000, but if you use ORG you need to be aware of what's happening. By default, the start of a raw binary is at offset 0, but if you need it you can change the offset to something different and make it work. For instance the following snippet accesses the variable msg with segment 0x7C0.

Asm Example:
<source lang="asm">
; boot.asm
   mov ax, 0x07c0
   mov ds, ax

   mov si, msg
   cld
ch_loop:lodsb
   or al, al ; zero=end of str
   jz hang   ; get out
   mov ah, 0x0E
   mov bh, 0
   int 0x10
   jmp ch_loop

hang:
   jmp hang

msg   db 'Hello World', 13, 10, 0
   times 510-($-$$) db 0
   db 0x55
   db 0xAA
</source>

Here is the ORG version. This time, msg is accessed with segment 0. Note that you still need to tell DS what to be as it can initialize to any value.

<source lang="asm">
[ORG 0x7c00]

   xor ax, ax ; make it zero
   mov ds, ax

   mov si, msg
   cld
ch_loop:lodsb
   or al, al  ; zero=end of string
   jz hang    ; get out
   mov ah, 0x0E
   mov bh, 0
   int 0x10
   jmp ch_loop

hang:
   jmp hang

msg   db 'Hello World', 13, 10, 0

   times 510-($-$$) db 0
   db 0x55
   db 0xAA
</source>

=== Procedures ===

To save on writing space, the typical 'procedures' are often separated from the code using CALL/RET like the following:
<source lang="asm">
[ORG 0x7c00]
   xor ax, ax  ;make it zero
   mov ds, ax
   cld

   mov si, msg
   call bios_print

hang:
   jmp hang

msg   db 'Hello World', 13, 10, 0

bios_print:
   lodsb
   or al, al  ;zero=end of str
   jz done    ;get out
   mov ah, 0x0E
   mov bh, 0
   int 0x10
   jmp bios_print
done:
   ret

   times 510-($-$$) db 0
   db 0x55
   db 0xAA
</source>

For some inexplicable reason, loading SI '''then''' jumping to the procedure always bugged me. Fortunately for psychos like me NASM's macros let you pretend that you are passing a parameter (macro definitions has to go before it's being called).

<source lang="asm">
%macro BiosPrint 1
                mov si, word %1
ch_loop:lodsb
   or al, al
   jz done
   mov ah, 0x0E
   int 0x10
   jmp ch_loop
done:
%endmacro

[ORG 0x7c00]
   xor ax, ax
   mov ds, ax
   cld

   BiosPrint msg

hang:
   jmp hang

msg   db 'Hello World', 13, 10, 0

   times 510-($-$$) db 0
   db 0x55
   db 0xAA
</source>

And in case your code is becoming long and unreadable, you can break it up into different files, then include the files at the beginning of you main code. Like so:
<source lang="asm">
jmp main

%include "othercode.inc"

main:
   ; ... rest of code here
</source>
Don't forget the jmp main at the start - otherwise some random other procedure will get called.

[[Category:Babystep]]
````

<a id="w001771"></a>

## W001771 — Babystep3

Bron-ID: W001771
Namespace: Main (0)
Revisie: 19080 | Datum: 2016-04-18T06:00:53Z
Bron: https://wiki.osdev.org/Babystep3
Revisie-URL: https://wiki.osdev.org/index.php?title=Babystep3&oldid=19080

````text
__NOTOC__

{{Infobox Tutorial
| name=Babystep3: A look at machine code
| prev=[[Babystep2]]
| next=[[Babystep4]]
}}

=== A look at machine code (opcodes, prefix, etc) ===


<source lang="asm">
; nasmw encode.asm -f bin -o encode.bin

mov cx, 0xFF
times 510-($-$$) db 0
db 0x55
db 0xAA
</source>
Don't partycopy to disk. Just open this in DEBUG (for Windows, Hexdump will be nice for Linux users)

 C:\osdev\debug encode.bin

Type in 'd' after the '-' to see the binary file. ('?' will give you help, 'q' will quit). You will see something like this:

 0AE3:0100 B9 FF 00 00 00 00 etc...

Look up the opcode for MOV here: [http://www.baldwin.cx/386htm/MOV.htm http://www.baldwin.cx/386htm/MOV.htm]
See Section "17.2.2.1 Opcode" here: [http://www.baldwin.cx/386htm/s17_02.htm http://www.baldwin.cx/386htm/s17_02.htm]

In other words, there is a unique register number (CX=1) added to the base opcode value 'B8' to give 'B9', which you see in the dump.

But watch what happens when you replace CX with ECX:

<source lang="asm">
mov ecx, 0xFF
times 510-($-$$) db 0
db 0x55
db 0xAA
</source>
 0AE3:0100 66 B9 FF 00 00 00 00 etc...

The '66' is an Operand Size Override Prefix generated by the assembler when there is a discrepancy with the default mode, which when NASM assembles binary files, it is 16-bit. The same thing happens if you use the BITS directive to change the mode, but it differs from the size of the operand:

<source lang="asm">
[BITS 32]
   mov cx, 0xFF
   times 510-($-$$) db 0
   db 0x55
   db 0xAA
</source>

This doesn't actually change the mode of the processor, but it does help it interpret the subsequent bytes.

==== Addresses ==== 

Address encoding is a bit more complicated

<source lang="asm">
 mov cx, [temp]

temp db 0x99
   times 510-($-$$) db 0
   db 0x55
   db 0xAA
</source>

 0AE3:0100 8B 0E 04 00 99 00 00 00 etc...

* '8B' is the opcode
* '0E' is a ModR/M byte which help the opcode interpretation

See Section "17.2.1 ModR/M and SIB Bytes" here: [http://www.baldwin.cx/386htm/s17_02.htm http://www.baldwin.cx/386htm/s17_02.htm]

The rules for interpreting this byte are complicated, because it contains different fields (see Fig. 17-2), but fortunately Table 17-2 makes it easier. Look up '0E' and you will see at the left it says "disp16" which means that the operand will be interpreted as a 16-bit offset.

'04 00' is the 16-bit offset. If you are confused why 0x0004 is backwards, it's because the Intel processor is "little endian". The "little" end of the number comes first.

'99' is of course the value of the byte at 0x0004 (8B is at 0x0000)

Be aware of another prefix called the Address size Override Prefix '67' which the assembler generates when there is a discrepancy just like with '66' above.

This stuff matters for a bunch of reasons, but since we will be making the switch from 16-bit real mode to 32-bit protected mode, our code is going to also change. And being aware of what a dump looks like can prevent a lot of grief.

[[Category:Babystep]]
````

<a id="w001772"></a>

## W001772 — Category:Babystep

Bron-ID: W001772
Namespace: Category (14)
Revisie: 29109 | Datum: 2024-06-07T13:41:27Z
Bron: https://wiki.osdev.org/Category:Babystep
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ABabystep&oldid=29109

````text
BabyStep is a series of tutorials intended for novice programmers that need help in writing simple bootloaders, and range from a simple 'Hello World' bootsector to [[Unreal Mode]] and [[Protected mode]] switch & display. They were started by CrazyBuddah in the old forum at mega-tokyo.com.

[[Category:Tutorials]]
[[Category:Bootloaders]]
````

<a id="w001774"></a>

## W001774 — Babystep4

Bron-ID: W001774
Namespace: Main (0)
Revisie: 22352 | Datum: 2018-04-27T02:12:30Z
Bron: https://wiki.osdev.org/Babystep4
Revisie-URL: https://wiki.osdev.org/index.php?title=Babystep4&oldid=22352

````text
__NOTOC__

{{Infobox Tutorial
| name=Babystep4
| prev=[[Babystep3]]
| next=[[Babystep5]]
}}

=== Printing to the screen without the BIOS ===

I know this is starting to look like a half-baked tutorial in assembly, but there's actually a reason behind my madness. Namely, solving as many problems as possible before switching to [[Protected mode]] etc. will lessen the confusion a great deal.

This example prints a string and the contents of a memory location (which is the first letter of the string in video memory). It is meant to demonstrate printing to screen in text mode without using [[BIOS]], as well as converting hex so it can be displayed -- so we can check register and memory values.

A [[stack]] is included, but only used by <tt>call</tt> and <tt>ret</tt>.

<source lang="asm">
;=====================================
; nasmw boot.asm -f bin -o boot.bin
; partcopy boot.bin 0 200 -f0

[ORG 0x7c00]      ; add to offsets
   xor ax, ax    ; make it zero
   mov ds, ax   ; DS=0
   mov ss, ax   ; stack starts at 0
   mov sp, 0x9c00   ; 2000h past code start

   cld

   mov ax, 0xb800   ; text video memory
   mov es, ax

   mov si, msg   ; show text string
   call sprint

   mov ax, 0xb800   ; look at video mem
   mov gs, ax
   mov bx, 0x0000   ; 'W'=57 attrib=0F
   mov ax, [gs:bx]

   mov  word [reg16], ax ;look at register
   call printreg16

hang:
   jmp hang

;----------------------
dochar:   call cprint         ; print one character
sprint:   lodsb      ; string char to AL
   cmp al, 0
   jne dochar   ; else, we're done
   add byte [ypos], 1   ;down one row
   mov byte [xpos], 0   ;back to left
   ret

cprint:   mov ah, 0x0F   ; attrib = white on black
   mov cx, ax    ; save char/attribute
   movzx ax, byte [ypos]
   mov dx, 160   ; 2 bytes (char/attrib)
   mul dx      ; for 80 columns
   movzx bx, byte [xpos]
   shl bx, 1    ; times 2 to skip attrib

   mov di, 0        ; start of video memory
   add di, ax      ; add y offset
   add di, bx      ; add x offset

   mov ax, cx        ; restore char/attribute
   stosw              ; write char/attribute
   add byte [xpos], 1  ; advance to right

   ret

;------------------------------------

printreg16:
   mov di, outstr16
   mov ax, [reg16]
   mov si, hexstr
   mov cx, 4   ;four places
hexloop:
   rol ax, 4   ;leftmost will
   mov bx, ax   ; become
   and bx, 0x0f   ; rightmost
   mov bl, [si + bx];index into hexstr
   mov [di], bl
   inc di
   dec cx
   jnz hexloop

   mov si, outstr16
   call sprint

   ret

;------------------------------------

xpos   db 0
ypos   db 0
hexstr   db '0123456789ABCDEF'
outstr16   db '0000', 0  ;register value string
reg16   dw    0  ; pass values to printreg16
msg   db "What are you doing, Dave?", 0
times 510-($-$$) db 0
db 0x55
db 0xAA
;==================================
</source>

[[Category:Babystep]]
````

<a id="w001775"></a>

## W001775 — Babystep5

Bron-ID: W001775
Namespace: Main (0)
Revisie: 23448 | Datum: 2019-03-09T22:48:07Z
Bron: https://wiki.osdev.org/Babystep5
Revisie-URL: https://wiki.osdev.org/index.php?title=Babystep5&oldid=23448

````text
__NOTOC__

{{Infobox Tutorial
| name=Babystep5
| prev=[[Babystep4]]
| next=[[Babystep6]]
}}

=== Interrupts ===

This code is meant to show how the hardware [[Interrupts|interrupt]] generated when you press a key can be handled by replacing the seg:offset specified in the [[Interrupt Vector Table|IVT]] (interrupt vector table). This normally points to a [[BIOS]] routine. To find the entry in the IVT, multiply the interrupt number by 4 (which is the size of each entry).

This key handler just displays the scan code without conversion to [[ASCII]], buffering, or handling extended keys. The reason for doing this is to not muddle up the basic idea, which is to provide input, as well as output, in its most simple form.

I will not go into the hows and whys of reading the ports involved in a key press. Suffice it to say that you are communicating with actual chips (or parts of chips), not some software intermediary. I personally feel it is good to remember that, no matter what level of abstraction you work at, you are ultimately telling hardware what to do.

I will point out the turning the keyboard on/off through port 0x61 is given in its complete form, some of which might not be needed, depending on the system.

<source lang="asm">
;==========================================
; nasmw boot.asm -f bin -o boot.bin
; partcopy boot.bin 0 200 -f0

[ORG 0x7c00]      ; add to offsets
   jmp start

   %include "print.inc"

start:   xor ax, ax   ; make it zero
   mov ds, ax   ; DS=0
   mov ss, ax   ; stack starts at 0
   mov sp, 0x9c00   ; 200h past code start

   mov ax, 0xb800   ; text video memory
   mov es, ax

   cli      ;no interruptions
   mov bx, 0x09   ;hardware interrupt #
   shl bx, 2   ;multiply by 4
   xor ax, ax
   mov gs, ax   ;start of memory
   mov [gs:bx], word keyhandler
   mov [gs:bx+2], ds ; segment
   sti

   jmp $      ; loop forever

keyhandler:
   in al, 0x60   ; get key data
   mov bl, al   ; save it
   mov byte [port60], al

   in al, 0x61   ; keybrd control
   mov ah, al
   or al, 0x80   ; disable bit 7
   out 0x61, al   ; send it back
   xchg ah, al   ; get original
   out 0x61, al   ; send that back

   mov al, 0x20   ; End of Interrupt
   out 0x20, al   ;

   and bl, 0x80   ; key released
   jnz done   ; don't repeat

   mov ax, [port60]
   mov  word [reg16], ax
   call printreg16

done:
   iret

port60   dw 0

   times 510-($-$$) db 0  ; fill sector w/ 0's
   dw 0xAA55        ; req'd by some BIOSes
;==========================================
</source>

=== print.inc ===

Just in case it wasn't clear, we have moved the printing routines from part 4 (''dochar'', ''sprint'', ''cprint'', and ''printreg16'') into an include file, which we will name '''print.inc'''.

<source lang="asm">
;==========================================
; print.inc
;----------------------



dochar:   call cprint         ; print one character
sprint:   lodsb      ; string char to AL
   cmp al, 0
   jne dochar   ; else, we're done
   add byte [ypos], 1   ;down one row
   mov byte [xpos], 0   ;back to left
   ret
 
cprint:   mov ah, 0x0F   ; attrib = white on black
   mov cx, ax    ; save char/attribute
   movzx ax, byte [ypos]
   mov dx, 160   ; 2 bytes (char/attrib)
   mul dx      ; for 80 columns
   movzx bx, byte [xpos]
   shl bx, 1    ; times 2 to skip attrib
 
   mov di, 0        ; start of video memory
   add di, ax      ; add y offset
   add di, bx      ; add x offset
 
   mov ax, cx        ; restore char/attribute
   stosw              ; write char/attribute
   add byte [xpos], 1  ; advance to right
 
   ret
 
;------------------------------------
 
printreg16:
   mov di, outstr16
   mov ax, [reg16]
   mov si, hexstr
   mov cx, 4   ;four places
hexloop:
   rol ax, 4   ;leftmost will
   mov bx, ax   ; become
   and bx, 0x0f   ; rightmost
   mov bl, [si + bx];index into hexstr
   mov [di], bl
   inc di
   dec cx
   jnz hexloop
 
   mov si, outstr16
   call sprint
 
   ret

;------------------------------------
 
xpos   db 0
ypos   db 0
hexstr   db '0123456789ABCDEF'
outstr16   db '0000', 0  ;register value string
reg16   dw    0  ; pass values to printreg16


</source>

==See Also==
===External Links===
* hardware fun http://chip.ms.mff.cuni.cz/pcguts/ (https://web.archive.org/web/20020801235709/http://chip.ms.mff.cuni.cz/~pcguts/)
* Intel's Summer Reading List http://developer.intel.com/vtune/cbts/refman.htm
* John Fine links to hardware programming http://www.geocities.com/SiliconValley/Peaks/8600/device.html (http://web.archive.org/web/20090723073639/http://www.geocities.com/SiliconValley/Peaks/8600/device.html)


[[Category:Babystep]]
````

<a id="w001776"></a>

## W001776 — Babystep6

Bron-ID: W001776
Namespace: Main (0)
Revisie: 22878 | Datum: 2018-09-14T23:35:03Z
Bron: https://wiki.osdev.org/Babystep6
Revisie-URL: https://wiki.osdev.org/index.php?title=Babystep6&oldid=22878

````text
__NOTOC__

{{Infobox Tutorial
| name=Babystep6
| prev=[[Babystep5]]
| next=[[Babystep7]]
}}

=== Entering protected mode ===

Entering [[Protected mode]] is actually done by switching a single bit in a special control register (cr0). (All the other stuff, like [[A20Line]], tasks, [[IDT]], call gates, etc. is additional stuff.)

However, before switching to pmode, you have to use the LGDT instruction to load another special register (gdtr) with the location of a table of data structures called descriptors that tell the process how to access memory.

==Descriptors==

Overview of bytes in the descriptor:
  +0 +1 +2 +3  +4 +5 +6 +7
  l0 l1 b0 b1  b2 TT Fl b3

Descriptor bytes arranged from lowest memory location to highest:
{| {{wikitable}}
|-
| <tt>0</tt>
| <tt>0x00</tt> lowest byte of Limit
|-
| <tt>1</tt>
| <tt>0x00</tt> next byte of Limit
|-
| <tt>2</tt>
| <tt>0x00</tt> lowest byte of Base Addr
|-
| <tt>3</tt>
| <tt>0x00</tt> next byte of Base Addr
|-
| <tt>4</tt>
| <tt>0x00</tt> third byte of Base Addr
|-
| <tt>5</tt>
| <tt>0x00</tt> = (bits) 0 - 00 - 0 - 0000 = P - DPL - S - Type
|-
| <tt>6</tt>
| <tt>0x00</tt> = (bits) 0 - 0 - 0 - 0 - 0000 = G - D/B - R - AVL - Size
|-
| <tt>7</tt>
| <tt>0x00</tt> fourth and highest byte of Base Addr
|}

===Bits in Type (byte #5)===
;P
:Present (1 bit) = 1 means segment is in memory (accessing a non-present segment will raise an exception)

;DPL
:Descriptor Privilege Level (2 bits) = 0 is most privileged and 3 is least.

;S
:System (1 bit) = must be 0 in descriptors for Task State Segments (TSS), Interrupt Gate, Trap Gate, Task Gate, Call Gates. Otherwise, for code/data/stack segment descriptors, it will be 1.

;Type
:Type (4 bits) = interpretation of these depends on whether S (above) is set or not. For S=0, the interpretation will be covered in specific instances of gates etc.

;Type bit 3
:If S=1, then if high bit is 1, it's a code segment, otherwise it's a data segment.

;Type bit 2
:The next highest bit depends on the highest bit. If code segment, this next bit indicates whether the segment is 'Conforming' or not. This allows programs somewhere else that are LESS privileged to access this segment, then this segment conforms to the privilege level of the calling program. If it's a data segment, this bit specifies "Expand (up or down)" for when the segment is used as a stack. Expand-up (bit=0) is your normal stack behavior. [[Expand_Down|Expand-down]] is used to prevent problems in stacks that are resized.

;Type bit 1
:The subsequent bit specifies permission to Read/Write. For data segments, 0 means read-only and 1 is r/w. For code segments, 0 means you can't read from it (e.g. using MOV) and 1 means you can.

;Type bit 0
:The lowest bit means that the segment has been accessed already (1) or not.

===Bits in Flags (byte #6)===
;G
:Granularity (1 bit) = segment Size specified in bytes (0) or 4K pages (1)

;D/B
:Default (code seg) / Big (data seg) = (1 bit) In a code segment (see "Type" above), this bits says default operand/address size is 32-bit (1) or 16-bit (0). For a data segment, it means stack pointer is 32-bit (1) or 16-bit (0). Also means something for expand-down stacks (see "Type" above), but we don't care.

;R
:Reserved (1 bit) = belongs to the Intel of the future.

;AVL
:Available (1 bit) = For your use. Go crazy.

;Size
:Top Nibble of Size (4 bits) = The size of the segment is 20-bits. This is the final four. Whether it means the highest possible segment size is 1 meg or 4 Gigs depends on Granularity above.

==See Also==
*[[Global Descriptor Table]]
*[[Segmentation]]

[[Category:Babystep]]
````

<a id="w001777"></a>

## W001777 — Babystep7

Bron-ID: W001777
Namespace: Main (0)
Revisie: 19020 | Datum: 2016-03-26T03:34:55Z
Bron: https://wiki.osdev.org/Babystep7
Revisie-URL: https://wiki.osdev.org/index.php?title=Babystep7&oldid=19020

````text
__NOTOC__

{{Infobox Tutorial
| name=Babystep7
| prev=[[Babystep6]]
| next=[[Babystep8]]
}}

==Unreal Mode==
:'''a.k.a Big Real or voodoo mode'''

While this code is largely just a party trick, understanding it gives a gentle intro to protected mode concepts and possibly avoids some headaches later on 'cause you skipped over this kind of stuff.

The single descriptor in the global descriptor table at the bottom is laid out to match [[Babystep6|Babystep6]]. The 'size' given is 1 MB, the base address is 0x0, and the bit fields you can work out yourself.

The reason for doing this is to enable 32-bit offsets in real mode. However, you won't be able to go past 1 meg quite yet.

In protected mode, the bits 3-15 in the segment register are an index into the descriptor table. That's why in this code 0x08 = 1000b gets you the 1 entry.

When this register given a "selector", a "segment descriptor cache register" is filled with the descriptor values, including the size (or limit). After the switch back to real mode, these values are not modified, regardless of what value is in the 16-bit segment register. So the 64k limit is no longer valid and 32-bit offsets can be used with the real-mode addressing rules (i.e. shift segment 4 bits, then add offset).

Finally, note that IP is unaffected by all this, so the code itself is still limited to 64k.

<source lang="asm">
AsmExample:

;==========================================
; nasmw boot.asm -o boot.bin
; partcopy boot.bin 0 200 -f0

[ORG 0x7c00]      ; add to offsets

start:   xor ax, ax   ; make it zero
   mov ds, ax   ; DS=0
   mov ss, ax   ; stack starts at 0
   mov sp, 0x9c00   ; 2000h past code start

   cli      ; no interrupt
   push ds      ; save real mode

   lgdt [gdtinfo]   ; load gdt register

   mov  eax, cr0   ; switch to pmode by
   or al,1         ; set pmode bit
   mov  cr0, eax

   mov  bx, 0x08   ; select descriptor 1
   mov  ds, bx   ; 8h = 1000b

   and al,0xFE     ; back to realmode
   mov  cr0, eax   ; by toggling bit again

   pop ds      ; get back old segment
   sti

   mov bx, 0x0f01   ; attrib/char of smiley
   mov eax, 0x0b8000 ; note 32 bit offset
   mov word [ds:eax], bx

   jmp $      ; loop forever

gdtinfo:
   dw gdt_end - gdt - 1   ;last byte in table
   dd gdt         ;start of table

gdt        dd 0,0  ; entry 0 is always unused
flatdesc    db 0xff, 0xff, 0, 0, 0, 10010010b, 11001111b, 0
gdt_end:

   times 510-($-$$) db 0  ; fill sector w/ 0's
   db 0x55          ; req'd by some BIOSes
   db 0xAA
;==========================================
</source>

==See Also==
===Articles===
*[[Unreal Mode]]

[[Category:Babystep]]
````

<a id="w001778"></a>

## W001778 — Babystep8

Bron-ID: W001778
Namespace: Main (0)
Revisie: 7946 | Datum: 2009-05-13T13:27:57Z
Bron: https://wiki.osdev.org/Babystep8
Revisie-URL: https://wiki.osdev.org/index.php?title=Babystep8&oldid=7946

````text
__NOTOC__

{{Infobox Tutorial
| name=Babystep8
| prev=[[Babystep7]]
}}

==32-bit printing==

Here is the same non-BIOS screen print AsmExample as before, but adjusted to use 32-bit registers and offsets. The 'complex' string instructions have been replaced.

<source lang="asm">
;----------------------
dochar:
    call cprint              ; print one character
sprint:
    mov eax, [esi]          ; string char to AL
    lea esi, [esi+1]
    cmp al, 0
    jne dochar               ; else, we're done
    add byte [ypos], 1       ; down one row
    mov byte [xpos], 0       ; back to left
    ret

cprint:
    mov ah, 0x0F             ; attrib = white on black
    mov ecx, eax             ; save char/attribute
    movzx eax, byte [ypos]
    mov edx, 160             ; 2 bytes (char/attrib)
    mul edx                  ; for 80 columns
    movzx ebx, byte [xpos]
    shl ebx, 1               ; times 2 to skip attrib

    mov edi, 0xb8000         ; start of video memory
    add edi, eax             ; add y offset
    add edi, ebx             ; add x offset

    mov eax, ecx             ; restore char/attribute
    mov word [ds:edi], ax
    add byte [xpos], 1       ; advance to right

    ret

;------------------------------------

printreg32:
    mov edi, outstr32
    mov eax, [reg32]
    mov esi, hexstr
    mov ecx, 8               ; eight nibbles

hexloop:
    rol eax, 4               ; leftmost will
    mov ebx, eax             ; become rightmost
    and ebx, 0x0f ;
    mov bl, [esi + ebx]      ; index into hexstr
    mov [edi], bl
    inc edi
    dec ecx
    jnz hexloop

    mov esi, outstr32
    call sprint

    ret

;------------------------------------

xpos db 0
ypos db 0
hexstr db '0123456789ABCDEF'
outstr32 db '00000000', 0    ; register value
reg32 dd 0                   ; pass values to printreg32

;------------------------------------
</source>

[[Category:Babystep]]
````

<a id="w001779"></a>

## W001779 — User:Jhawthorn/ASCII

Bron-ID: W001779
Namespace: User (2)
Revisie: 7241 | Datum: 2009-02-15T20:30:12Z
Bron: https://wiki.osdev.org/User:Jhawthorn/ASCII
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJhawthorn%2FASCII&oldid=7241

````text
{{In Progress}}

<tt>
{| {{wikitable}}
|-
|
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
</tt>
````

<a id="w001780"></a>

## W001780 — Category talk:Babystep

Bron-ID: W001780
Namespace: Category talk (15)
Revisie: 2795 | Datum: 2007-01-13T10:07:28Z
Bron: https://wiki.osdev.org/Category_talk:Babystep
Revisie-URL: https://wiki.osdev.org/index.php?title=Category_talk%3ABabystep&oldid=2795

````text
==Name==
Is "Babystep" still an appropriate name for these tutorials? (I don't object to it but it's not very precise on the contents) -[[User:Jhawthorn|Jhawthorn]] 04:07, 13 January 2007 (CST)
````

<a id="w001781"></a>

## W001781 — Win Image

Bron-ID: W001781
Namespace: Main (0)
Revisie: 2804 | Datum: 2007-01-13T22:19:22Z
Bron: https://wiki.osdev.org/Win_Image
Revisie-URL: https://wiki.osdev.org/index.php?title=Win_Image&oldid=2804
Doorverwijzing: WinImage

````text
#REDIRECT [[WinImage]]
````

<a id="w001782"></a>

## W001782 — Template:Infobox Tutorial

Bron-ID: W001782
Namespace: Template (10)
Revisie: 3888 | Datum: 2007-05-18T03:19:12Z
Bron: https://wiki.osdev.org/Template:Infobox_Tutorial
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AInfobox_Tutorial&oldid=3888

````text
__NOEDITSECTION__
<table align="right" cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; width: 30em; font-size: 90%; text-align: left;">
<tr><td colspan="2" style="text-align: center; font-size: 130%; font-weight: bold;">
{{{name}}}
</td></tr>
{{if | {{{image}}} | <tr><td colspan="2">[[Image:{{{image}}}|300px|]]</td></tr>}}
<tr><td colspan="2" style="text-align: center; font-size: 83%; font-weight: bold;">
<h4>[[:Category:Tutorials|Tutorial]]</h4>
</td></tr>
{{if | {{{prev|{{{next|}}}}}} | <tr><td>'''Previous'''</td><td>'''Next'''</td></tr><tr><td>{{{prev|}}}</td><td>{{{next|}}}</td></tr>}}
</table>
````

<a id="w001784"></a>

## W001784 — Talk:CMOS

Bron-ID: W001784
Namespace: Talk (1)
Revisie: 12323 | Datum: 2011-12-14T09:15:01Z
Bron: https://wiki.osdev.org/Talk:CMOS
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ACMOS&oldid=12323

````text
==Usage of GCC inline assembler instead of VC?==
Technically the code could be converted to code that would compile in GCC, Considering a majority of code examples here are in Assembly or C in a syntax compatible with GCC. --[[User:Brynet-Inc|Brynet-Inc]] 17:17, 14 January 2007 (CST)

The goal is to get the point across. To that extent intel assembly is easier to read than AT&T syntax. Besides, this is 16-bit stuff which fits in neither GCC nor MSVC. - [[User:Combuster|Combuster]] 17:13, 15 January 2007 (CST)

== Reading and sanitizing the RTC ==

I use the following code to read and sanitize the RTC:

<pre>
char time[10];
void out_byte(int port, int value);
int in_byte(int port);
enum {
	cmos_address = 0x70,
	cmos_data    = 0x71
};
int cmos_ready() {
	out_byte(cmos_address, 10);
	return (in_byte(cmos_data) & 0x80);
}

void read_rtc() {
	while (cmos_ready());
	out_byte(cmos_address, 0x00);	time[0] = in_byte(cmos_data);
	out_byte(cmos_address, 0x02);	time[1] = in_byte(cmos_data);
	out_byte(cmos_address, 0x04);	time[2] = in_byte(cmos_data);
	out_byte(cmos_address, 0x06);	time[3] = in_byte(cmos_data);
	out_byte(cmos_address, 0x07);	time[4] = in_byte(cmos_data);
	out_byte(cmos_address, 0x08);	time[5] = in_byte(cmos_data) - 1;
	out_byte(cmos_address, 0x09);	time[6] = in_byte(cmos_data);
	out_byte(cmos_address, 0x32);	time[7] = in_byte(cmos_data);
	out_byte(cmos_address, 0x0a);	time[8] = in_byte(cmos_data);
	out_byte(cmos_address, 0x0b);	time[9] = in_byte(cmos_data);
	// 12 hour clock
	if (!(time[9] & 2) && (time[2] & 0x80)) {
		time[2] = ((time[2] & 0x80) + 12) % 24;
	}
	// decimal stuff
	if (!(time[9] & 4)) {
		time[0] = (time[0] & 0xf) + ((time[0] / 16) * 10);
		time[1] = (time[1] & 0xf) + ((time[1] / 16) * 10);
		time[2] = (time[2] & 0xf) + ((time[2] / 16) * 10);
		time[5] = (time[5] & 0xf) + ((time[5] / 16) * 10);
	}
	time[3] = (time[3] + 5)	% 7;
}
</pre>

This produces a much saner version of the CMOS bytes for the RTC; weeks start on Mondays (international standard, used pretty much everywhere but the US) with Monday = 0, Sunday = 6. January = 0 (not 1), December = 11, hour is always [0..23] and minutes/seconds is always [0..59] (no hexing). Naturally, I can't say that the "century" is a correct representation of the century.

The code assumes that interrupts are disabled before calling it.

I hope this is helpful. (Posted 2011-12-13 by Clearer, edited by Solar.)
````

<a id="w001785"></a>

## W001785 — Makefile

Bron-ID: W001785
Namespace: Main (0)
Revisie: 30883 | Datum: 2026-07-11T22:19:14Z
Bron: https://wiki.osdev.org/Makefile
Revisie-URL: https://wiki.osdev.org/index.php?title=Makefile&oldid=30883

````text
{{FirstPerson}}
{{Rating|1}}

A Makefile is a file which controls the 'make' command.  Make is available on virtually every platform, and is used to control the build process of a project.  Once a Makefile has been written for a project, make can easily and efficiently build your project and create the required output file(s).

Make reads dependency information from your Makefile, figures out which output files need (re-)building (because they are either missing or older than their corresponding input files), executes the necessary commands to actually ''do'' the (re-)building. This compares favourably to "build batchfiles" that always rebuild the whole project.

In doing this, make is not limited to any specific set of languages. Any tool that takes some input file and generates an output file can be controlled via make.

A Makefile can offer several different "targets", which can be invoked by 'make <target>'. A frequently-used target is 'make clean', which removes any temporary files, or 'make install', which installs the project in its target location - but you can also give a specific output file as a target, and have make process only that file. Invoking make without a target parameter builds the default target (which is usually to build the whole project).

But 'make' requires a well-written Makefile to do these things efficiently and reliably.

== Makefile tutorial ==

The 'make' manual will tell you about the hundreds of things you can do with a Makefile, but it doesn't give you an example for a ''good'' Makefile. The following examples are mostly shaped after the real-life [http://pdclib.e43.eu PDCLib] Makefile I used at the time, and shows some of the "tricks" used therein that may not be that obvious to the make beginner.

The Makefile creates only one project-wide linker library, but it should be easy to expand it for multiple binaries/libraries.

(Note: There are several different flavours of make around, and POSIX defines a common denominator for them. This tutorial specifically targets GNU make. See the [[Talk:Makefile#Which_make_to_target.3F | discussion page]] for further info.) 

=== Basics ===

It is best practice to name your Makefile <code>Makefile</code> (without an extension), because when make is executed without any parameters, it will by default look for that file in the current directory. It also makes the file show up prominently on top of directory listings, at least on Unix machines.

Generally speaking, a Makefile consists of ''definitions'' and ''rules''.

A ''definition'' declares a variable, and assigns a value to it. Its general syntax is <code>VARIABLE := Value</code>.

'''Note:''' Frequently, you will see definitions using "=" instead of ":=". Such a definition will result in the assignment being evaluated ''every time the variable is used'', while ":=" evaluates only once at the startup of make, which is usually what you want. Don't go changing other people's Makefiles, though - ":=" is a GNU extension to the make syntax.

A rule defines a ''target'', 0..n ''dependencies'', and 0..n ''commands''. The general idea is that ''make'' checks if the target (file) is there; if it isn't, or any of the dependencies is newer than the target, the commands are executed. The general syntax is:

<syntaxhighlight lang="make">
target: dependency
	command
</syntaxhighlight>

Both ''dependency'' and ''command'' are optional. There might be more than one ''command'' line, in which case they are executed in sequence.

Note that commands ''must be tab-indented''. If your editor environment is set to replace tabs with spaces, you have to undo that setting while editing a Makefile.

What makes Makefiles so hard to read, for the beginner, is that it is not procedural in syntax (i.e., executed top-down), but functional: 'make' reads the ''whole'' Makefile, building a dependency tree, and then resolves the dependencies by hopping from rule to rule as necessary until the target you gave it on the command line has successfully been resolved.

But let's not go into internal details of 'make'. This is a tutorial, not a man page, so it will show you how a real-life Makefile could be built, and the ideas behind each line.

=== Automated Testing ===

As the Makefile presented in this tutorial takes care of automated testing in a special way, this approach will be explained up front, so the related parts of the tutorial will make sense to you.

As stated above, this tutorial is mainly derived from work done on the PDCLib project - which builds a single linker library. In that project, there is strictly one library function per source file. In each such source file, there is a test driver attached for conditional compilation, like this:
<syntaxhighlight lang="C">
#ifdef TEST
int main()
{
    /* Test function code here */
    return NUMBER_OF_TEST_ERRORS;
}
#endif
</syntaxhighlight>
Thus, when that source is compiled with <code>gcc -c</code>, it results in an object file with the library function code; when compiled with <code>gcc -DTEST</code>, it gives a test driver executable for that function. Returning the number of errors allows to do a grand total of errors encountered (see below).

=== File Lists ===

First, various "file lists" are assembled which are needed later in the Makefile.

==== Non-Source Files ====

<syntaxhighlight lang="make">
# This is a list of all non-source files that are part of the distribution.
AUXFILES := Makefile Readme.txt
</syntaxhighlight>

Further down we will have a target ''"dist"'', which packages all required files into a tarball for distribution. Lists of the sources and headers are created anyway. But there are commonly some auxiliary files, which are not referenced anywhere else in the Makefile but should still end up in the tarball. These are listed here.

==== Project Directories ====
<syntaxhighlight lang="make">
PROJDIRS := functions includes internals
</syntaxhighlight>
Those are subdirectories holding the actual sources. (Or rather, searched for source files automatically, see below.) These could be subprojects, or whatever. We could simply search for source files starting with the current working directory, but if you like to have temporary subdirectories in your project (for testing, keeping reference sources etc.), that wouldn't work.

''Note that this is not a recursive approach; there is no Makefile in those subdirectories. Dependencies are hard enough to get right if you use one Makefile. At the bottom of this article is a link to a very good paper on the subject titled "recursive make considered harmful"; not only is a single Makefile easier to maintain (once you learned a couple of tricks), it can also be more efficient!''

==== Sources and Headers ====
<syntaxhighlight lang="make">
SRCFILES := $(shell find $(PROJDIRS) -type f -name "\*.c")
HDRFILES := $(shell find $(PROJDIRS) -type f -name "\*.h")
</syntaxhighlight>
It should be obvious to see what these two lines do. We now have a list of all source and header files in our project directories.

==== Object Files and Test Driver Executables ====
<syntaxhighlight lang="make">
OBJFILES := $(patsubst %.c,%.o,$(SRCFILES))
TSTFILES := $(patsubst %.c,%_t,$(SRCFILES))
</syntaxhighlight>
''OBJFILES'' should be clear - a list of the source files, with ''*.c'' exchanged for ''*.o''. ''TSTFILES'' does the same for the filename suffix ''*_t'', which we will use for our test driver executables.

:''Note: This tutorial initially used ''*.t'' instead of ''*_t'' here; this, however, kept us from handling dependencies for test driver executables separately from those for library object files, which was necessary. See next section.''

==== Dependencies, pt. 1 ====
Many people edit their Makefile every time they add/change an ''#include'' somewhere in their code. (Or forget to do so, resulting in some scratching of heads.)

But most compilers - including [[GCC]] - can do this automatically for you! Although the approach looks a little backward. For each source file, ''GCC'' will create a ''dependency file'' (which is canonically made to end in ''*.d''), which contains a Makefile dependency rule listing that source file's includes. (And more, but see below.)

We need two separate sets of dependency files; one for the library objects, and one for the test driver executables (which usually have additional includes, and thus dependencies, not needed for the OBJFILES).
<syntaxhighlight lang="make">
DEPFILES    := $(patsubst %.c,%.d,$(SRCFILES))
TSTDEPFILES := $(patsubst %,%.d,$(TSTFILES))
</syntaxhighlight>

==== Distribution Files ====
The last list is the one with all sources, headers, and auxiliary files that should end up in a distribution tarball.
<syntaxhighlight lang="make">
ALLFILES := $(SRCFILES) $(HDRFILES) $(AUXFILES)
</syntaxhighlight>

=== .PHONY ===
The next one can take you by surprise. When you write a rule for ''make clean'', and there happens to be a file named ''clean'' in your working directory, you might be surprised to find that ''make'' does nothing, because the "target" of the rule ''clean'' already exists. To avoid this, declare such rules as ''phony'', i.e. disable the checking for a file of that name. These will be executed every time:
<syntaxhighlight lang="make">
.PHONY: all clean dist check testdrivers todolist
</syntaxhighlight>

=== CFLAGS ===
If you thought ''-Wall'' does tell you everything, you'll be in for a rude surprise now. If you don't even use ''-Wall'', shame on you. ;)
<syntaxhighlight lang="make">
WARNINGS := -Wall -Wextra -pedantic -Wshadow -Wpointer-arith -Wcast-align \
            -Wwrite-strings -Wmissing-prototypes -Wmissing-declarations \
            -Wredundant-decls -Wnested-externs -Winline -Wno-long-long \
            -Wconversion -Wstrict-prototypes
CFLAGS := -g -std=gnu99 $(WARNINGS)
</syntaxhighlight>
It is suggested to add new warning options to your project one at a time instead of all at once, to avoid getting swamped in warnings. ;) These flags are merely recommendations for C work. If you use C++, you need different ones. Check out the GCC manual; each major compiler update changes the list of available warning options.

=== Rules ===

Now come the rules, in their typical backward manner (top-level rule first). The topmost rule is the default one (if 'make' is called without an explicit target). It is common practice to have the first rule called "all".

==== Top-Level Targets ====
<syntaxhighlight lang="make">
all: pdclib.a

pdclib.a: $(OBJFILES)
	@ar r pdclib.a $?

clean:
	-@$(RM) $(wildcard $(OBJFILES) $(DEPFILES) $(TSTFILES) pdclib.a pdclib.tgz)

dist:
	@tar czf pdclib.tgz $(ALLFILES)
</syntaxhighlight>
The ''@'' at the beginning of a line tells ''make'' to be quiet, i.e. not to echo the executed commands on the console prior to executing them. The Unix credo is "no news is good news". You don't get a list of processed files with ''cp'' or ''tar'', either, so it's completely beyond me why developers chose to have their Makefiles churn out endless lists of useless garbage. One very practical advantage of shutting up ''make'' is that you actually get to ''see'' those compiler warnings, instead of having them drowned out by noise.

The ''-'' at the beginning of a line tells ''make'' to continue even in case an error is encountered (default behaviour is to terminate the whole build).

The ''$(RM)'' in the ''clean'' rule is the platform-independent command to remove a file.

The ''$?'' in the ''pdclib.a'' rule is an internal variable, which ''make'' expands to list all dependencies to the rule ''which are newer than the target''.

==== Automated Testing, pt. 2 ====
<syntaxhighlight lang="make">
check: testdrivers
	-@rc=0; count=0; \
        for file in $(TSTFILES); do \
            echo " TST     $$file"; ./$$file; \
            rc=`expr $$rc + $$?`; count=`expr $$count + 1`; \
        done; \
	echo; echo "Tests executed: $$count  Tests failed: $$rc"

testdrivers: $(TSTFILES)
</syntaxhighlight>
Call it crude, but it works beautifully for me. The leading ''-'' means that 'make' will not abort when encountering an error, but continue with the loop.

The ''echo " TST    $$file"'' in there is useful in case you get a ''SEGFAULT'' or something like that from one of your test drivers. (Without echoing the drivers as they are executed, you would be at a loss as to which one crashed.)

==== Dependencies, pt. 2 ====
<syntaxhighlight lang="make">
-include $(DEPFILES) $(TSTDEPFILES)
</syntaxhighlight>
Further below, you will see how dependency files are ''generated''. Here, we ''include'' all of them, i.e. make the dependencies listed in them part of our Makefile. Never mind that they might not even exist when we run our Makefile the first time - the leading "-" again suppresses the errors.

==== Extracting TODO Statements ====
<syntaxhighlight lang="make">
todolist:
	-@for file in $(ALLFILES:Makefile=); do fgrep -H -e TODO -e FIXME $$file; done; true
</syntaxhighlight>
Taking all files in your project, with exception of the Makefile itself, this will ''grep'' all those ''TODO'' and ''FIXME'' comments from your files, and display them in the terminal. It is nice to be remembered of what is still missing before you do a release. To add another keyword, just insert another <code>-e keyword</code>.

:'''Note:''' ''$(ALLFILES:Makefile=) is a list of everything in $(ALLFILES), except for the string "Makefile" which is replaced with nothing (i.e., removed from the list). This avoids a self-referring match, where the string "TODO" in the grep command would find itself. (Of course, it also avoids finding any *real* TODO's in the Makefile, but there's always a downside. ;-) )''

==== Dependencies, pt. 3 ====

Now comes the dependency magic I talked about earlier. Note that this needs GCC 3.3 or newer.
<syntaxhighlight lang="make">
%.o: %.c Makefile
	@$(CC) $(CFLAGS) -MMD -MP -c $< -o $@
</syntaxhighlight>
Isn't it a beauty? ;-)

The ''-MMD'' flag generates the dependency file (%.d), which will hold (in Makefile syntax) rules making the ''generated'' file (%.o in this case) depend on the source file ''and any non-system headers it includes''. That means the object file gets recreated automatically whenever relevant sources are touched. If you want to also depend on system headers (i.e., checking them for updates on each compile), use ''-MD'' instead. The ''-MP'' option adds empty dummy rules, which avoid errors should header files be removed from the filesystem.

Compiling the object file actually looks like a side effect. ;-) 

Note that the dependency list of the rule includes the Makefile itself. If you changed e.g. the ''CFLAGS'' in the Makefile, you want them to be applied, don't you? Using the $< macro ("first dependency") in the command makes sure we do not attempt to compile the Makefile as C source.

Of course we also need a rule for generating the test driver executables (and their dependency files):

<syntaxhighlight lang="make">
%_t: %.c Makefile pdclib.a
	@$(CC) $(CFLAGS) -MMD -MP -DTEST $< pdclib.a -o $@
</syntaxhighlight>

Here you can see why test driver executables get a ''*_t'' suffix instead of a ''*.t'' extension: The ''-MMD'' option uses the basename (i.e., filename without extension) of the ''compiled'' file as basis for the dependency file. If we would compile the sources into ''abc.o'' and ''abc.t'', the dependency files would both be named ''abc.d'', overwriting each other.

Other compilers differ a bit in their support for this, so look up their specifics when using something else than ''GCC''.

==== Backups ====

Backing up your files is a perfect candidate of a task that should be automated. However, this is generally achieved through a [[Code_Management#Version_Control_Systems|version control system]].

Below is an example of a ''backup'' target, which creates a dated 7-Zip archive of the directory where the Makefile resides.

<syntaxhighlight lang="make">
THISDIR := $(shell basename `pwd`)
TODAY := $(shell date +%Y-%m-%d)
BACKUPDIR := projectBackups/$(TODAY)

backup: clean
	@tar cf - ../$(THISDIR) | 7za a -si ../$(BACKUPDIR)/$(THISDIR).$(TODAY)_`date +%H%M`.tar.7z
</syntaxhighlight>

=== Summary ===

A well-written Makefile can make maintaining a code base much easier, as it can wrap complex command sequences into simple 'make' invocations. Especially for large projects, it also cuts back on compilation times when compared to a dumb "build.sh" script. And, once written, modifications to a well-written Makefile are seldom necessary.

== Advanced Techniques ==

Some of the stuff we did above already ''was'' pretty advanced, and no mistake. But we needed those features for the basic yet convenient setup. Below you will find some even trickier stuff, which might not be for everybody but is immensely helpful ''if'' you need it.

=== Conditional Evaluation ===

Sometimes it is useful to react on the existence or content of certain environment variables. For example, you might have to rely on the path to a certain framework being passed in FRAMEWORK_PATH. Perhaps the error message given by the compiler in case the variable is not set is not helpful, or it takes long until 'make' gets to the point where it actually detects the error. You want to fail early, and with a meaningful error message.

Luckily, 'make' allows for conditional evaluation and manual error reporting, quite akin to the C preprocessor:

<syntaxhighlight lang="make">
ifndef FRAMEWORK_PATH
    $(error FRAMEWORK_PATH is not set. Please set to the path where you installed "Framework".)
endif

ifneq ($(FRAMEWORK_PATH),/usr/lib/framework)
    $(warning FRAMEWORK_PATH is set to $(FRAMEWORK_PATH), not /usr/lib/framework. Are you sure this is correct?)
endif
</syntaxhighlight>

=== Multi-Target ===

Preliminary work has been done to write an improved Makefile, which allows generating multiple binaries with individual settings while still being easy to configure and use. While this is not yet in "tutorial" format, commented source can be found [[User:Solar/Makefile | here]].

== See Also ==
=== Articles ===
* [[User:Solar/Makefile]], A more complex example capable of building multiple executables and libraries from a single Makefile.

=== External Links ===
* [https://github.com/DevSolar/jaws JAWS], a pre-configured [http://www.cmake.org CMake] setup which, while not geared toward OS development, is a definite step forward from "naked" Makefiles.
* [https://aegis.sourceforge.net/auug97.pdf Recursive Make Considered Harmful] by Peter Miller<br />A paper detailing why the traditional approach of recursive make invocations harms performance and reliability, proposing a modular Makefile approach instead.
* [https://web.archive.org/web/20150115042103/http://evbergen.home.xs4all.nl/nonrecursive-make.html Implementing non-recursive make] by Emile van Bergen<br />Further input on the subject of non-recursive, low-maintenance Makefile creation.
* [https://www.gnu.org/software/make/manual/ Manual for GNU make]
* [https://www.goodreads.com/book/show/583690.Managing_Projects_with_GNU_Make Managing Projects with GNU Make], the O'Reilly book on GNU Make

[[Category:Tutorials]]
[[Category:Build Tools]]

[[de:Makefile]]
````

<a id="w001790"></a>

## W001790 — Category:Bare bones tutorials

Bron-ID: W001790
Namespace: Category (14)
Revisie: 2840 | Datum: 2007-01-18T06:05:21Z
Bron: https://wiki.osdev.org/Category:Bare_bones_tutorials
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ABare_bones_tutorials&oldid=2840

````text
[[Category:Tutorials]]
````

<a id="w001791"></a>

## W001791 — Category talk:Bare bones tutorials

Bron-ID: W001791
Namespace: Category talk (15)
Revisie: 2843 | Datum: 2007-01-18T06:10:29Z
Bron: https://wiki.osdev.org/Category_talk:Bare_bones_tutorials
Revisie-URL: https://wiki.osdev.org/index.php?title=Category_talk%3ABare_bones_tutorials&oldid=2843

````text
== Kernel Tutorials or Barebones ==
Voicing [[User:Solar|Solar]]'s opinion (with which i agree, see forum), these aren't real kernels, but barebones. I'd opt for a rename as such - [[User:Combuster|Combuster]] 10:33, 16 January 2007 (CST)
:Go for it -[[User:Jhawthorn|Jhawthorn]] 11:41, 16 January 2007 (CST)
:Actually, I made the mess, I'll fix it. -[[User:Jhawthorn|Jhawthorn]] 23:58, 17 January 2007 (CST)
````

<a id="w001794"></a>

## W001794 — Template:Disputed

Bron-ID: W001794
Namespace: Template (10)
Revisie: 22865 | Datum: 2018-09-14T23:21:50Z
Bron: https://wiki.osdev.org/Template:Disputed
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ADisputed&oldid=22865

````text
<center>
{|style="border: 1px solid #cfcfbf; padding: .0em .25em .0em; background-color: #f0f0ff; text-align: center;"
|
<font color="black">The factual accuracy of this article or section is [[:Category:Disputed Pages|disputed]].</font><br /><small><font color="red">Please see the relevant discussion on the {{DiscussThis|talk page}}.</font></small>
|
|}
<includeonly>[[Category:Disputed Pages]]</includeonly>
</center>
````

<a id="w001795"></a>

## W001795 — Category:Disputed Pages

Bron-ID: W001795
Namespace: Category (14)
Revisie: 2853 | Datum: 2007-01-20T10:23:26Z
Bron: https://wiki.osdev.org/Category:Disputed_Pages
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ADisputed_Pages&oldid=2853

````text
Lists all articles using the [[Template:Disputed|disputed]] template.

[[Category:Wiki administration]]
````

<a id="w001796"></a>

## W001796 — Category talk:Hardware

Bron-ID: W001796
Namespace: Category talk (15)
Revisie: 9479 | Datum: 2009-12-20T16:05:08Z
Bron: https://wiki.osdev.org/Category_talk:Hardware
Revisie-URL: https://wiki.osdev.org/index.php?title=Category_talk%3AHardware&oldid=9479

````text
== Revamp ==

Let me spam you guys first with an expansion of the hardware tree:

::Category:Architectures|Architectures
::'''[[PowerPC]]'''
::[[:Category:X86|X86]]
:::'''[[AMD K6 WriteBack Optimisations]]'''
:::'''[[Bootloader]]'''
:::'''[[CPU Bugs]]'''
:::Category:Operating Modes|Operating Modes
::::Category:Protected Mode|Protected Mode
:::::'''[[Global Descriptor Table]]'''
:::::'''[[Interrupt Descriptor Table]]'''
:::::'''[[Protected Mode]]'''
:::::'''[[Virtual 8086 Mode]]'''
:::::'''[[Virtual Monitor]]'''
::::Category:Real Mode|Real Mode
:::::'''[[Interrupt Vector Table]]'''
:::::'''[[Real Mode]]'''
::::'''[[Unreal Mode]]'''
:::'''Category:System Initialization (x86)|System Initialization (x86)'''
::::'''[[MBR (x86)]]'''
:::'''Tutorial:GDT'''
:::Category:X86 Instructions|X86 Instructions
::::'''[[CLI]]'''
::::'''[[CPUID]]'''
::::'''[[INT]]'''
::::'''INVLPG'''
::::'''STI'''
::::'''XCHG'''
:::'''[[X86-64]]'''
:'''[[BIOS]]'''
:Category:CPU|CPU
::'''[[AMD K6 WriteBack Optimisations]]'''
::'''[[CPU Bugs]]'''
::'''[[Detecting CPU Speed]]'''
::'''[[X86-64]]'''
:Category:HardwareComm|HardwareComm
::'''[[3c556]]'''
::'''[[Ne2000]]'''
::'''[[Serial ports]]'''
:[[:Category:Interrupts|Interrupts]]
::'''[[Interrupt Service Routines]]'''
::'''[[PIC]]'''
:Category:Memory|Memory
::'''[[A20 Line]]'''
::'''[[CMOS]]'''
:[[:Category:Networking|Networking]]
::'''[[3c556]]'''
::'''[[Ne2000]]'''
:'''[[PCI]]'''
:Category:Peripheral Buses
::'''[[USB]]'''
:'''[[PIC]]'''
:'''Category:PowerManagement'''
::'''[[ACPI]]'''
::'''[[APM]]'''
::'''[[RSDP]]'''
:'''[[Programmable Interval Timer]]'''
:[[:Category:Storage|Storage]]
::'''[[ATA]]'''
::'''[[DMA]]'''
::'''[[Floppy Driver]]'''
::'''[[Storage]]'''
:[[:Category:Video|Video]]
::'''[[Text UI]]'''
::[[:Category:VGA|VGA]]
:::'''[[VGA Hardware]]'''
:::'''[[VGA Resources]]'''

What I notice that there are a lot of topics out of place, topics in categories where they don't belong, duplicate categories, categories with same-named pages, single-topic categories, categories with a full article's contents, and naming inconsistencies: IMO one big mess.

Id like to propose a reorganisation of the category structure as well as some sort of manual of style related issues:
At the root, i'd suggest having only concrete devices and directly related standards. (most definately no theory related pages) Like the following:

:Category:Architectures|Architectures
:[[:Category:Common Devices|Common Devices]] (motherboard resources, timers, RNGs, memory: DMA PIC PIT and APIC would go here)
:Category:Communication|Communication (merge of hardwarecomm and networking)
:Category:Input Devices|Input Devices (nonexistant, but useful. A good KBC article would go here)
:Category:Peripheral Buses|Peripheral Buses
:[[:Category:Sound|Sound]] (nonexistant)
:[[:Category:Storage|Storage]]
:[[:Category:Video|Video]]
as subcategories, and either no topics, or a list of ALL devices (for allowing a quick lookup of a device being documented) for which some wikipedia magic could be useful?

For the architectures category:
:X86 (category)
:: Operating Modes (as category, containing only PM, RM, UM, v8086 mode as articles, no subs)
:: Instructions (as category)
:: all other x86-specific topics would go as normal articles here, as well as the articles under CPU
and similarly for other architectures
:X86-64 (as category)
:: [[X86-64]] (as article)
:: ...
:PowerPC (as category)
::[[PowerPC]] (as article)
:: ... 

The philosophy behind this is that it should be consistent, and obvious. (a lot of subclasses from x86 have been removed, and other architectures get their own category for any future expansion).

For the remaining topics:
BIOS goes to X86 (possibly renamed as X86 BIOS)
Text UI goes to OS Theory.
ACPI and APM are a bit doubtful.

suggestions welcome

- [[User:Combuster|Combuster]] 06:28, 20 January 2007 (CST)
````

<a id="w001797"></a>

## W001797 — Blocking Process

Bron-ID: W001797
Namespace: Main (0)
Revisie: 24004 | Datum: 2019-09-25T13:09:58Z
Bron: https://wiki.osdev.org/Blocking_Process
Revisie-URL: https://wiki.osdev.org/index.php?title=Blocking_Process&oldid=24004

````text
A process is described as blocking or waiting when it is set inactive until a specific event occurs.

== Blocking Process ==
A blocking process is usually waiting for an event such as a semaphore being released or a message arriving in its message queue. In multitasking systems, such processes are expected to notify the scheduler with a system call that it is to wait, so that they can be removed from the active scheduling queue until the event occurs. A process that continues to run while waiting (i.e., continuously polling for the event in a tight loop) is said to be [[Busy_loop|busy-waiting]], which is undesirable as it wastes clock cycles which could be used for other processes.

== Sleeping ==

A special case of waiting is sleeping, which is when a process is set inactive for a given period of time (e.g., 20 milliseconds). This is usually handled separately for ordinary waiting, primarily for efficiency reasons; since the clock interrupt is frequent, and keeping track of individual ticks is infeasible, the usual approach is to use a system of relative counters to mark when a given process will be awakened.

A commonly used data structure for tracking sleeping processes is the delta queue, an ordered list of sleeping processes in which each process has a counter which is offset relative to last process in the list before it; for example, if process A is sleeping for 3 ticks, process B for 5 ticks, process C for 5 ticks, and process D for 8 ticks, then the list would be

 {A, 3} -> {B, 2} -> {C, 0} -> {D, 3}

At each clock tick, the counter for the topmost process is decremented, like so:

 {A, 2} -> {B, 2} -> {C, 0} -> {D, 3}

If at this point process E is added which is to wait 6 ticks, then it would be inserted before D, and both E and D would be updated appropriately:

 {A, 2} -> {B, 2} -> {C, 0} -> {E, 2} -> {D, 1}

If the topmost process reaches zero, then it -- and any processes immediately following it that are also zero -- are placed back into the active scheduling queue. Thus, after 2 more ticks, A is set as active; 2 ticks after that, both B and C are set as active; and so on. This approach has the advantage of minimizing the number of changes required in a given clock tick.

== Interruptible and Uninterruptible ==

It is generally a good idea to differentiate between interruptible wait/sleep and uninterruptible wait/sleep. Depending on your system, you will probably want most wait states interruptible, though you almost definitely will need some uninterruptible states. You might want to consider building interruptible wait on top of uninterruptible wait, instead of using special flags.

[[Category:Processes and Threads]]
[[Category:OS theory]]
````

<a id="w001798"></a>

## W001798 — User:Combuster/notepad

Bron-ID: W001798
Namespace: User (2)
Revisie: 18280 | Datum: 2015-08-12T12:02:26Z
Bron: https://wiki.osdev.org/User:Combuster/notepad
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACombuster%2Fnotepad&oldid=18280

````text
These are my notes from experience. PD code, No warranty, might be inaccurate in places etc - even I mess up things more than once ;). If you feel parts should be in the main namespace, feel free to do so

== UEFI ==
A bunch of notes regarding UEFI and how people have tried hard to make things stupid:

=== Official documentation ===
Those morons put a contract on the documentation, so the only one you can download straight from google is an older 2.0 specification. And it's got a fair share of pitfalls.

==== ABI ====
The EFI ABI for x86_64 is severely limtedly documented in the UEFI specification, which leads to issues where you are not expecting them. Notable problems:
* You need to "push" all arguments on the stack - including all arguments that you also pass in a register. The function should use the register version, so you can leave anything in those holes, but many functions will simply write back the arguments to their locations. If you're not reserving space, that means your stack gets overwritten.
* It does not state that you should maintain 16-byte alignment.
* The documentation gives you 4K stack minimum, but doesn't describe ownership for anything past the top of the stack. In fact, there's no red zone and interrupts will simply jump to the firmware and overwrite anything you put past the stackpointer. If you internally resort to the AMD64 ABI, make sure you pass <tt>-mno-red-zone </tt>
Basically, it boils down to the fact that the EFI ABI follows exactly the Microsoft x86_64 ABI, including everything it didn't copy over from the [http://msdn.microsoft.com/en-us/library/ew5tede7%28v=vs.80%29.aspx original documentation] (and all associated surprises thereof)

==== EFI_BOOT_SERVICES ====
In the table, there's a missing pointer, which means everything you use past it has wrong offsets and therefore calls the wrong function
<pre>EFI_UNINSTALL_PROTOCOL_INTERFACE UninstallProtocolInterface;
EFI_HANDLE_PROTOCOL HandleProtocol;
EFI_REGISTER_PROTOCOL_NOTIFY RegisterProtocolNotify;</pre>
This should be something like
<pre>EFI_UNINSTALL_PROTOCOL_INTERFACE UninstallProtocolInterface;
void * PossiblyHandleProtocol_1;
void * PossiblyHandleProtocol_2;
EFI_REGISTER_PROTOCOL_NOTIFY RegisterProtocolNotify;</pre>

=== gnu-efi ===
This is the biggest hack and worst teaching material in the book. What it does is employs a gigantic series of hacks to make linux binaries run on the EFI ABI. It strips relocations, forges fake relocations to make EFI believe the binary is relocatable, then performs manual ELF loading and the relevant self-modifying-code hacks to get all the addresses back in order when it runs. It also has wrappers to toggle between AMD64 and Microsoft ABIs

Symptoms are broken output binaries for x86_64, and intermediate files that cause buffer overflows in objdump so you can't properly inspect them, and way too much support code needed to even get an empty binary working

Instead, use a PE-targeted cross-compiler. i386-pe works for 32-bits; x86_64-pe gives a functional binutils but no gcc; a mingw/cygwin compiler is technically wrong, but probably works better already. For both platforms, just passing "-shared -dll --subsystem 10" arguments is sufficient to get the expected output. But don't use it together with gnu-efi or the proper relocations get messed up with the fake ones and the firmware suddenly stops being able to read the file (as well as objdump).

=== emu woes ===
There has been a broken amd64 build of the OVMF bios posted on the website. Right now it works. Most tutorials around the web are well out of date though: you only get one binary instead of separate firmware and plug-in roms (so there's no VGA rom!)

I've seen both work with both Bochs and qemu. In bochs, you seem to need
* The right bios image
* 440FX support compiled and enabled
* Cirrus Logic compiled and enabled (although not necessarily with the newer bioses)

In qemu:
* "-bios path/to/ovmf"
* "-vga cirrus"
* "-no-kvm" where needed
* "-net none" to cut off minutes of boot time waiting for netboot
* "-hda fat:(...)" to create virtual harddisks without building entire images

=== HD construction ===

This can perfectly be automated. No superuser access needed! Tools: mtools, parted, dd
(clean up and copy instructions here)

== Graphics Card design ==

[[Image:Graphicscard.png]]

A graph I made to show many of the components you might find in a video card. Note that the data routing is in many cases much more specialized, with dedicated routes between certain components (like direct connections between the GPU and dedicated texture memory)

Considerations for graphics development:
* Several chips have fixed pixel clocks, which use a fixed frequency. (VGA, older adapters)
* There may be many kind of devices connected to the output, VGA and vga-capable are common, but digital outputs are not uncommon
* LCDs have a native, if not fixed resolution
* Not everything has a refresh rate (Terminals, Non-VGA LCDs, The [[BGA]], etc.)
* Not everything supports different refresh rates (VGA, TV standards), and those who do have a limit (All CRTs and VGA/DVI LCDs)
* Not all chips and modes output a single pixel per dot clock cycle (VGA does 1 clock per 4 bits, 1 clock/pixel in 16 color, 2 clock/pixel in 256 color, similar effects with SVGA controllers)
* Not everything is a framebuffer, including tile engines (VGA text, VT), sprite engines (console hardware), up to DSP-like engines.
* Video cards have a n:m mapping to display outputs - video cards have zero(!) to multiple outputs, SLI'ed cards have effects on adapters connected not to itself.
* It is 1 am - do you know which display engine(s) on which card(s) are currently pretending to be VGA?

== E820 woes ==
I recently discovered a broken bios that did not fill in the mandatory part of the structure (but left parts of it intact). Turns out GRUB doesn't fix this behaviour and that all multiboot kernels would consequently get incorrect data about the memory map.

A more complete list on bugproofing calling 0xE820:
* EAX = '''0x0000'''E820 ''(Ralph Brown: call may be ignored if it isn't)''
* The list can be terminated by either EBX=0 (where you need to parse the provided result) or CF=1 (where you should not)
* reserve '''at least''' 20 bytes ''(Ralph Brown: some bioses will always write 20 bytes)''
* make sure the area receiving the structure is '''zeroed''' before use. ''(Undocumented)''

== Porting the FreeBasic Runtime ==
To get past deprecated versions and everything, I updated my FB installation from 0.15 to 0.20(!)
As a consequence, I got the updated library and since I couldn't be bothered to merge all the differences, I had a go from scratch. I decided afterwards to record the general process here.

===starting off===
#unpacked libfb-0.20b to a temp directory, copied the rtlib directory over to my workspace
#got rid of the automake garbage :)
#performed some shell tricks to get a list of c files into the directories Makefile, so that I only needed to add that bit of building rules
#integrated the stuff into the build system.
Well, to start things off, I decided to compile the first c file in the list to see where things got awry, and was greeted by several screens full of errors.

===Shaddap, I'm developing an OS here O.o===
Starting at the top to work things out, there was a nice #error message, telling me that the platform was unsupported. No duh. Edited fb.h and removed that silly statement.

Then came the garbage from the C library - wide character support was *not* enabled and FB can get pretty nasty over that. So I stripped the fb_unicode.h header, and all the system-specific stuff not available in freestanding land and ended up with fb_file.h fb_device.h fb_serial.h fb_thread and fb_printer.h commented out. Next I went down the other files and commented out the remaining declarations with wide characters.

Some remaining functions were found and removed, and at some point the first function compiled.

===Getting to the bone===
Obviously, I tried next to compile all parts of the runtime library, which obviously didn't like the fact that lots of pieces were disabled. And of course the missing OS support.
Some 50% of the files refused compilation, and to test for sanity, I shoved the remaining stuff into a single binary to see what was actually working. Of course the linker protested and more files were disabled

After disabling 99% of the source files, making mental notes of what was needed, some basic memory moving stuff was working. most string functions were failing with a undefined reference to __fb_ctx, the array functions with error handling code.

=== __fb_ctx and fb_hInit()===
after a nice grep I located the definition in libfb_init.c this file compiled, but didn't link because of an OS dependency to fb_hInit(), with a comment belonging to host specific initialisation.

Since I needed nothing that the C runtime already provided for me, I got away easy with a small support file.

 #include "../fb.h"
 
 void fb_hInit( void )
 {
     return;
 }

afterwards, most of the string functions could be enabled.

=== Error numbers ===
The other keystone to the language are the array functions. The required source file that went about this was libfb_error_getset.c, which would provide the calls, but refused to compile from scratch, due to a missing ERROR constant. Simply setting ERROR to a constant resulted in a missing definition, which was resolved by re-enabling fb_thread in fb.h which in turn (what else) gave trouble. After removing parts of an enum with more missing macro's, all that was left was again a depency to what appeared to be a host support function. This time dealing with thread local storage.

Since I didn't want to bother with the threading support, I added a stub that would fix the messages and do something useful. Basically it returns the error context for this specific thread.

 #include "../fb.h"
 
 FB_ERRORCTX __fb_errorctx;
 
 FBCALL void *fb_TlsGetCtx (int index, int len )
 { 
     switch(index)
     {
         case FB_TLSKEY_ERROR:
             return(&__fb_errorctx);
             break;
 
         default:
             while(1);
     }
     
     while(1);
     return(NULL);
 }

Since the ERROR macro made no sense, I changed that too adding this into fb_error.h (replacing the previous ERROR macro I put there):

 #define ERROR FB_TLSKEY_ERROR

Afterwards, most array functions were operational, and some other random functions started working too. With runtime support for the two most used primitives, I decided to call it a day and commit the changes.


== DIV test ==
originally devised to tell apart amd/intel/cyrix/whatever 486s and 386s that didn't have cpuid, this still works on current hardware.
You can detect spoofs (emulators?) by verifying the div test signature with the expected one.

recent intels maintain all flags after DIV, as do bochs and qemu by default. cyrices clear most flags, AMDs set some and clear some.

 PUSHFD
 POP EAX
 ; either clear flags for first test, or set flags for second test. pick one:
 ; AND AX, (0xffff - 0x8D5)
 ; OR AX, 0x8D5
 PUSH EAX
 MOV AX, 5
 MOV CL, 2
 POPFD
 DIV CL 
 PUSHFD
 POP EAX
 AND EAX, 0x8D5
 ;AX contains signature

in 16-bit mode, use popf and ax instead of popfd and eax

results I've gathered:

{| {{Wikitable}}
|-
! CPU
! Div test with all flags clear
! Div test with all flags set

|-
| Intel 686
| 0x000
| 0x8D5

|-
| AMD K6-II
| 0x010
| 0x811

|-
| AMD Athlon64
| 0x010
| 0x811

|-
| Cyrix Cx6x86 / M1
| 0x000
| 0x801

|-
! Emulator
! Div test with all flags clear
! Div test with all flags set

|-
| Bochs
| 0x000
| 0x8D5

|-
| Qemu (without kqemu)
| 0x000
| 0x8D5

|}
Still looking for information on the 386 and 486 generation...

== Bochs messages ==
A list of messages and likely causes:

 interrupt(): gate descriptor is not valid sys seg
You have not loaded an IDT, or the IDT is corrupt

 interrupt(): SS selector null
* You have no TSS
* You haven't set SS0 / ESP0 in the TSS

 CR0 = 0xe0000001 CR2 = 0xe0000001
* Your page tables are not page-aligned 
* Your page tables do not point to the correct parts of memory

 MOV_EwSw: can't use this segment register 1
* MOV CS, ... is not a valid instruction, except on an 8086 where you don't want it either because it doesn't set IP.

 prefetch: getHostMemAddr vetoed direct read, pAddr=0x0000000a0000
* You jumped to the wrong address.
* Your page tables do not point to the correct parts of memory
* You're trying to run code in VGA memory, which is not allowed

== HPPA ==
That big noisy RISC machine from HP. Oh well. Needs some patching to get a elf toolchain built.

apparently there are two file formats at work here: ELF and SOM. if you mix up those two you get
no end of assembly messages (unknown pseudo-op .nsubspa etc). the following got me a successfully
built gcc. I have not tested its actual operations for sanity. Just doccing it here so I can
reproduce the results on a computer that's closer to my HPPA machine.

add to gcc/config.gcc
<pre>hppa-*-elf*)
	target_cpu_default="MASK_PA_11|MASK_NO_SPACE_REGS"
	tm_file="${tm_file} pa/pa32-regs.h dbxelf.h elfos.h pa/elf.h \
		 pa/pa-pro-end.h libgloss.h"
	tmake_file="pa/t-pro"
	use_collect2=yes
	gas=yes
	;;</pre>

run

 export PREFIX=/usr/cross
 export TARGET=hppa-elf
 mkdir build-gcc-hppa
 mkdir build-binutils-hppa
 cd build-binutils-hppa
 ../binutils-x.x/configure --target=$TARGET --prefix=$PREFIX --disable-nls
 make
 make install
 cd ..
 export PATH=$PATH:/usr/cross/bin
 cd build-gcc-hppa
 ../gcc-x.x.x/configure --target=$TARGET --prefix=$PREFIX --without-headers
             --disable-nls --with-newlib --enable-languages=c
 make all-gcc
 make install-gcc

works with binutils 2.17/gcc 4.2.2
fails on binutils 2.17/gcc 4.1.1 

The 15 minutes spent trying to get palo (the opensource bootloader used with linux) to compile resulted in an asm error in crt0.s
and several missing system headers (guess what, PA-RISC + LILO = PALO. linux.h missing. d'oh). 

And that was one step out of the many that need be taken...
````

<a id="w001799"></a>

## W001799 — Category:Sparc

Bron-ID: W001799
Namespace: Category (14)
Revisie: 3004 | Datum: 2007-02-14T22:29:19Z
Bron: https://wiki.osdev.org/Category:Sparc
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ASparc&oldid=3004

````text
SPARC (the Scalable Processor ARChitecture) is a RISC-based processor mainly used in high-end workstations and servers; in particular, Sun's SparcStation and many of Sun's Ultra machines are SPARC based.

[[Category:Platforms]]
````

<a id="w001800"></a>

## W001800 — User:Mr.Confuzed

Bron-ID: W001800
Namespace: User (2)
Revisie: 3623 | Datum: 2007-04-08T21:19:11Z
Bron: https://wiki.osdev.org/User:Mr.Confuzed
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMr.Confuzed&oldid=3623

````text
The following is the NASM source code for my OS (actually, it's just a boot program).  It boots into protected mode and lets you type stuff on the text mode screen with a custom cursor.  Really basic, but it now uses a GDT and an IDT.

I have some fairly lofty goals for this, but it is a hobby OS and I'm trying to keep it simple specifically for now while I'm still learning.  I'm currently thinking about adding memory management.

Mr.Confuzed - Feb. 19, 2007



<pre>
[BITS 16]
[ORG 0x7C00]



 Boot_loader:
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

 jmp 0:Clear_cs
 Clear_cs:

;Load kernel from floppy disk
;Int 0x13 Function 0x02 (Load sectors)
;ah = Function 0x02
;al = Number of sectors
;cx = Cylinder and sector
;dx = Head and drive number
;es:bx = Destination
mov ax,0x020F
mov bx,0x7E00
mov cx,0x0002
xor dh,dh
int 0x13

;Set screen to 80x50 text mode (8x8 font)
mov ax,0x1112
xor bl,bl
int 0x10

;Enable A20 via keyboard controller
call Wait_for_keyboard_controller
;Out A20 control
mov al,0xD1
out 0x64,al
call Wait_for_keyboard_controller
;Out Enable A20
mov al,0xDF
out 0x60,al

;Enter protected mode
lgdt[Reference_GDT]
mov eax,cr0
or al,0x01
mov cr0,eax

;Jump to kernel
 jmp Code_GDT:Kernel



Wait_for_keyboard_controller:
mov cx,0xFFFF
Loop_Wait_for_keyboard_controller:
in al,0x64
test al,2
loopnz Loop_Wait_for_keyboard_controller
 ret



;Global Descriptor Table reference:
;16 Length
;32 Physical Address

;GDT descriptor format:
;16 Limit
;24 Base
;1 Present (1)
;2 Privilege (ring)
;1 (1)
;1 Executable
;1 Direction/Conforming
;1 Readable/Writable
;1 Accessed (0)
;1 Limit granularity (0=byte,1=4kilobyte)
;1 Size (1)
;2 (0)
;4 Limit
;8 Base

;Note that dd produces data in little endian order (bytes are backwards).

 Reference_GDT:
dw End_GDT - GDT - 1
dd GDT
 GDT:
dd 0x00000000
dd 0x00000000
 Code_GDT equ $ - GDT
dd 0x0000FFFF
dd 0x00CF9A00
 Data_GDT equ $ - GDT
dd 0x0000FFFF
dd 0x00CF9200
 Video_GDT equ $ - GDT
dd 0x80001F3F
dd 0x0000920B
 End_GDT:



;Fix boot sector size
times 510-($-$$) db 0
;Boot loader signature
dw 0xAA55



[BITS 32]



 Kernel:

;Set default state
cli
mov ax,Data_GDT 
mov ds,ax
mov es,ax
mov fs,ax
mov ss,ax
mov ax,Video_GDT
mov gs,ax
mov esp,0x00007C00

call Clear_screen

;Display Hello World in protected mode
mov esi,Hello_World
xor edi,edi
call Write_string
mov word [ds:Insertion_Point],di

;Program Programmable Interrupt Controllers:
;Source:Nanos
;IRQ 0-15 = Int 0x20-0x2F
;ICW1 (Initialize)
mov al,0x15
out 0x20,al
out 0xA0,al
;ICW2 (Vector offset)
mov al,0x20
out 0x21,al
add al,8
out 0xA1,al
;ICW3
mov al,4
out 0x21,al
mov al,2
out 0xA1,al
;ICW4
mov al,0x01
out 0x21,al
out 0xA1,al
;Enable IRQs at PICs (Keyboard)
mov al,11111101b
out 0x21,al
mov al,11111111b
out 0xA1,al

lidt[Reference_IDT]
sti

hlt

 jmp $



 Clear_screen:

xor edi,edi
.Loop_Clear_screen
mov dword [gs:edi],0x07000700
add edi,4
cmp edi,8000
jb .Loop_Clear_screen

 ret


 Write_string:
;Edits dl,esi,edi

;Get cursor location... or not!
;Initial write location parameter stored in DI
;Initial read location parameter stored in SI

mov byte dl,[ds:esi]
or dl,dl
jz .Exit_Write_string
mov byte [gs:edi],dl
inc esi
add edi,2
jmp Write_string
.Exit_Write_string

 ret



Hello_World db 'Protected mode successful. ',0



;Interrupt Descriptor Table
;Gate structure:
;16 Offset
;16 Selector
;8 cleared
;1 Present (1)
;2 Privilege (ring)
;1 cleared
;4 Gate type
;  Interrupt: 0xE
;  Task: 0x5
;  Trap: 0xF
;16 Offset

;Exceptions:
;Source:Nanos
;0 Divide Error
;1 Debug Exception
;2 NMI  Interrupt
;3 Breakpoint
;4 INTO - overflow
;5 BOUND Range exceeded
;6 Invalid opcode
;7 Device not available
;8 Double Fault
;9 Co-processor segment overrun
;A Invalid TSS
;B Segment Not Present
;C Stack Fault
;D General Protection Fault
;E Page Fault
;F
;10  Floating Point Error
;11  Alignment Check
;12-1F Reserved

;Source:Wikipedia.org
;IRQ 0 - System timer. Reserved for the system. Cannot be changed by a user.
;IRQ 1 - Keyboard. Reserved for the system. Cannot be altered even if no keyboard is present or needed.
;IRQ 2 - Second IRQ controller. See below for explanation.
;IRQ 3 - COM 2(Default) COM 4(User)
;IRQ 4 - COM 1(Default) COM 3(User)
;IRQ 5 - Sound card (Sound Blaster Pro or later) or LPT2(User)
;IRQ 6 - Floppy disk controller
;IRQ 7 - LPT1(Parallel port) or sound card (8-bit Sound Blaster and compatibles)
;IRQ 8 - Real time clock
;IRQ 9 - ACPI SCI or ISA MPU-401
;IRQ 10 - Free / Open interrupt / Available / SCSI
;IRQ 11 - Free / Open interrupt / Available / SCSI
;IRQ 12 - PS/2 connector Mouse / If no PS/2 connector mouse is used, this can be used for other peripherals
;IRQ 13 - Math co-processor. Cannot be changed
;IRQ 14 - Primary IDE. If no Primary IDE this can be changed
;IRQ 15 - Secondary IDE

 Reference_IDT:
dw End_IDT - IDT - 1
dd IDT
 IDT:
;Exceptions, not supported
dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000

dd 0x00000000
dd 0x00000000
;not supported
 Timer_IDT equ $ - IDT
dd 0x00000000
dd 0x00000000
;supported soon
 Keyboard_IDT equ $ - IDT
dw Interrupt.IRQ1
dw Code_GDT
dw 0x8E00
dw 0x0000
 End_IDT:



;Interrupt Service Routines:
 Interrupt.IRQ1
pusha

xor eax,eax

;get keycode
 read_kbd:
in al,0x64
and al,0x01
jz read_kbd
in al,0x60
call Scancode_To_ASCII

;send EOI to PIC
mov al,0x20
out 0x20,al
popa
 iret



 Display_Keypress:
xor ebx,ebx
mov word bx,[ds:Insertion_Point]
mov byte [gs:ebx],dl
add bx,2
call Move_Insertion_Point
 ret



 Scancode_To_ASCII:
;Uses ASCII db
cmp al,0x39
ja Above_Scancode_To_ASCII
add eax,ASCII
mov byte dl,[ds:eax]
call Display_Keypress
jmp End_Scancode_To_ASCII
Above_Scancode_To_ASCII:
call Virtual_Mouse
 End_Scancode_To_ASCII:
 ret



 Virtual_Mouse:
xor ebx,ebx
mov word bx,[ds:Insertion_Point]

cmp al,0x48
je Up_Virtual_Mouse
cmp al,0x4C
je Down_Virtual_Mouse
cmp al,0x50
je Down_Virtual_Mouse
cmp al,0x4B
je Left_Virtual_Mouse
cmp al,0x4D
je Right_Virtual_Mouse

jmp End_Virtual_Mouse

 Up_Virtual_Mouse:
cmp bx,160
jb End_Virtual_Mouse
sub bx,160
call Move_Insertion_Point
jmp End_Virtual_Mouse

 Down_Virtual_Mouse:
cmp bx,7840
jae End_Virtual_Mouse
add bx,160
call Move_Insertion_Point
jmp End_Virtual_Mouse

 Left_Virtual_Mouse:
cmp bx,0
je End_Virtual_Mouse
sub bx,2
call Move_Insertion_Point
jmp End_Virtual_Mouse

 Right_Virtual_Mouse:
cmp bx,7998
je End_Virtual_Mouse
add bx,2
call Move_Insertion_Point

 End_Virtual_Mouse
 ret



 Move_Insertion_Point:
;bx = new location
xor edi,edi
mov word di,[ds:Insertion_Point]
inc di
mov byte dl,[gs:edi]
xor dl,0xFF
mov byte [gs:edi],dl

mov word [ds:Insertion_Point],bx
inc bx
mov byte dl,[gs:ebx]
xor dl,0xFF
mov byte [gs:ebx],dl

 ret



Insertion_Point dw 0x0000
ASCII db 0,27,49,50,51,52,53,54,55,56,57,48,45,61,8,9,81,87,69,82,84,89,85,73,79,80,91,93,10,0,65,83,68,70,71,72,74,75,76,59,39,96,0,92,90,88,67,86,66,78,77,44,46,47,0,42,0,32



;Fix floppy image size
times 1474560-($-$$) db 0
</pre>
````

<a id="w001801"></a>

## W001801 — System Initialization (Sparc)

Bron-ID: W001801
Namespace: Main (0)
Revisie: 28031 | Datum: 2023-07-09T18:42:28Z
Bron: https://wiki.osdev.org/System_Initialization_(Sparc)
Revisie-URL: https://wiki.osdev.org/index.php?title=System_Initialization_%28Sparc%29&oldid=28031

````text
At boot time, a SPARC system will load the OpenBoot firmware, which then initializes basic hardware (such as the framebuffer, serial ports, keyboard, and disks), tests system functioning, and, optionally, (if the auto-boot? option is set to true) loads a primary bootloader in some way. Otherwise, it gives the user an "ok" prompt, at which to type initialization commands or FORTH instructions. 

In theory, one could load an operating system from any device they wish, regardless of whether the device is directly supported by OpenBoot (eg, by typing in FORTH words to stick the raw bytes of the operating system code in memory). In practice, however, an operating system is generally loaded from the hard disk, a floppy, or the network.

== Hard disk ==
Typing "boot disk" or "b sd(0,3,0)" for older boot proms at the boot prompt loads a bootloader from the hard disk, starting with sector 1 (512 bytes from the beginning of the disk) and continuing for 15 more sectors (7.5 Kb), assuming that the disk has a proper Sun Disklabel on it. Alternatively, "load disk" loads the bootloader into memory but does not execute it.

== CD/DVD ==
Typing "boot cdrom" or "b sd(0,6,0)" for older boot proms will load the boot loader from the system architecture specific partition of the cdrom. This allows different code to be executed on the various sun architectures sun-4 sun4c sun4m sun4d etc... In the case linux and bsd the partitions are all mapped to the same bootblock. The SILO bootloader documentation covers this in some detail. 
== Floppy ==
Somebody who has tried this, please fill this in. Theoretically, because disk-label provides the load command, it should work the same way, but who knows?

If you have a v2 prom a the prom type
<pre>boot floppy</pre>
or if you have a v0 prom
<pre>b fd()</pre>

BSD boot floppies work at least on v2 proms tested on Sparc Station 5 and 10 and supposedly work on v0 boot proms as well though initial testing showed it isn't as reliable. Also see the [[SILO]] page. 

== Network ==
Loading from the network uses RARP to configure an IP address. It then connects to the TFTP server on the machine that responded to the RARP request, and downloads a file whose name is the machine's IP address in hex (with ".SUN4C" or ".SUN4M" appended if it's a V7 or V8 SPARC, respectively). This is then used as the bootloader.


== See Also ==
* [[SILO]]
=== Articles ===
* [[Sparc Overview]]

=== Links ===
* [http://www.cheesecake.org/sac/os-supp/ Writing an Operating System for SPARC-Based Computers]

[[Category:Sparc]]
[[Category:Booting]]
````

<a id="w001802"></a>

## W001802 — Talk:VGA Hardware

Bron-ID: W001802
Namespace: Talk (1)
Revisie: 12345 | Datum: 2011-12-16T08:46:56Z
Bron: https://wiki.osdev.org/Talk:VGA_Hardware
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AVGA_Hardware&oldid=12345

````text
== Things that still need to be done: ==
* I need to reverse engineer more documentation for the sequencer.
* I need to test some additional GC bits for effects.
* Read Modes 0 and 1, Write modes 1,2,3 (this is easy as this documentation is rather complete. I Have not tested all details of this stuff)
* DAC Mask Register (the latest spec doesnt document it)
* Color Logic (There's more to this than the eye meets. I can write this from info out of Abrash's book, but i should still test it anyway. (It partially depends on the Sequencer)
* Sequencer: byte word doubleword mode, doublescanning, why 256-color modes have halved horizontal resolutions. Some of this depends on the sequencer operation...
- [[User:Combuster|Combuster]] 17:15, 3 January 2007 (CST)

== What you can do: ==
* Proof-read it, check for sanity.
* Comment on n00b-friendliness
* check registers and timings
* probably a lot more :)
- [[User:Combuster|Combuster]] 16:55, 27 December 2006 (CST)


I'm proof-reading it :-)<br />
Let me know if you'd rather me use British English instead of American English.<br />
- [[User:Renagadexx|Jack]] 17:23, 14 October 2011 (UTC)

== Detection ==
You mention checking for VGA compatability before use, but you make no mention of how to check for VGA compatability. I have never programmed for VGA so could not write this myself. --[[User:Tyler|Tyler]] 18:28, 1 February 2007 (CST)

Well, I don't have any detection code either, because I know of no standard way existing of doing probes. You can deduce an probing method from the info listed, however, but it is not guaranteed to leave non-vga machines intact  (see forum thread on ATI Mobile cards). You may want to google for it and contribute whatever you find - [[User:Combuster|Combuster]] 03:38, 2 February 2007 (CST)

Well i have had a good search. Unfortunately it really is all so device independent that it is impossible to tell. I do recall once reading some kernel source code that verified a colour system was being used in it's initialisation routine. Unfortunately i have misplaced the code so if anyone comes across it, i think it may be used as such. I will keep searching. --[[User:Tyler|Tyler]] 10:08, 2 February 2007 (CST)
: For PCI cards you can probe the class and subclass; if the device class is 0x03 and the subclass is 0x01, then it is VGA compatible; there are no guarantees for any other subclasees (i.e. it might be VGA compatible or it might not).[[User:Clearer|Clearer]] 08:46, 16 December 2011 (UTC)

== Memory layout stuff ==

Should we turn this section into a link to Bochs_VBE_extensions#Memory_layout_of_video_modes and copy the fleshed-out tidbits to the aforementioned article? Or the other way around? --[[User:Troy martin|Troy Martin]] 00:06, 27 December 2008 (UTC)

== "Memory Layout in 4-bit modes" ==

Surely the memory layout in 4-bit modes would be exactly the same as the memory layout in 16-colour modes? :-)

I assume this title was meant to be "Memory Layout in 4-colour (2-bit) modes". In this case AFAIK it's very similar to 4-bit/16-colour modes, except that only 2 planes are used (instead of 4 planes). The same applies to monochrome (1-bit, 2-colour) modes, where only one plane is used.

== Detection ==

There's at least 3 different "hints" that could be used to help VGA detection.

The first hint is the class code in the device's PCI configuration space. If the class code is 0x03 (display device), the sub-class code is 0x00 and the interface code is 0x00, then (according to PCI standards) the device is: ''"VGA-compatible controller. Memory addresses 0A0000h through 0BFFFFh. I/O addresses 3B0h to 3BBh and 3C0h to 3DFh and all aliases of these addresses."''.

Other class codes define other types of display devices. Example, 0x030000 is VGA, 0x030001 is "8514-compatible controller", 0x030100 is "XGA controller", 0x030200 is "3D controller" and 0x038000 is "Other display controller". Also, class code 0x00???? means that "Device was built before Class Code definitions were finalized".

The second hint is VBE. For "Function 00h - Return VBE Controller Information", Bit 1 in the Capabilities field will be either set (Controller is not VGA compatible) or clear (Controller is VGA compatible).

The third hint is BIOS function Int 0x10, ax = 0x1A00 ("GET DISPLAY COMBINATION CODE (PS,VGA/MCGA)"). I'll let Ralph Brown describe it (see http://www.ctyme.com/intr/rb-0219.htm). I'm assuming that if this function is supported and returns 0x07 or 0x08 then it's VGA compatible (however, if this function is supported and returns 0xFF then the controller may or may not be VGA compatible).

Of course I call these "hints" because I don't think any of them are necessarily definitive. A better idea might be to get the PCI vendor and device IDs and implement a "compatible enough to work" white list and a "not compatible enough" black list (and then rely on the hints above for video cards that aren't in either list).


== Old Chain-4 discussion ==
'' Been testing the effect of Chain 4 on memory writes and output, and the results aren't consistent with one another. Chain 4 is located in the Sequencer which would mean setting/clearing it would have effect on video output. Furthermore I have been testing whether plane enable has effect in chain 4 writes. '' - Combuster


'' This section needs a rewrite. Chain4 mode is perfectly normal function of VGA that standard BIOS 0x13 mode relies on for it's "linear" addressing. Basicly, when chain4 is set, lowest 2 bits of memory access address select the plane, and the address is shifted 2 bits down. The memory organization (as far as display goes) is always planar (like Mode-X) but the special mapping that is chain4 mode makes it appear linear. As such, I find it highly unlikely that the data about chain4 support below is correct. [[User:Mystran|Mystran]] 09:25, 9 December 2007 (CST)


'' Here's the catch: all systems emulate mode 0x13 and mode-x up to the extent most programmers do expect. When in mode 0x13, video memory seems linear due to the chain 4 bit. On the other end, selecting the corresponding byte-mode, word-mode and dword-mode make the display come up as appropriate, i.e. dword mode for 0x13 and byte mode for mode-x. It means that when you use only one mode there's nothing wrong. However, bochs does not support the b/w/d-mode bits and instead uses the chain-4 bit to determine the selection. QEmu doesn't support these bits either, but always uses byte mode and redirects the writes in chain-4 enabled mode to the location needed to have byte word work as expected. All the tested emulators have their quirks in this area, assuming that the real cards are the most VGA compatible. (MSVPC being the closest to a real card).''

''The process to verify the behaviour:''
* Enter mode-x, clear screen (I added rulers but they won't interfere with the actual process.)
* Enable chain-4 bit (but leave the BWD bits in their mode-x state) - this should not change the screen (it does however on bochs)
* write a test pattern to the screen
* Visually check the output. On real hardware you'll notice 4 coloured pixels followed by 12 blank ones, 4 pixels, 12 blanks etc.
''(you could also clear chain-4 and read out the actual planes to detect qemu writing pixels without blanks)''

''I have just verified it again. The write address is not shifted as you state, it is ANDed. The 12-pixel gaps appear supporting the stated hypothesis. If you have '' '''evidence''' '' of the opposite I would gladly see it. The purpose is after all to provide information.''

''If you look at existing mode-X code you will see that it will change the two Byte/Word/Doubleword bits besides just the chain4 bit.''

''I have in the meantime learned that the chip ordering isn't as logically distinct as you expect. Chain-4 is located at the GC end of the sequencer and should not affect the output at the CRTC/AC end. I added a stub instead of the comment above to at least make some effort regarding the rewrite'' - [[User:Combuster|Combuster]] 17:12, 9 December 2007 (CST)

''p.s. would you mind using the talkpage next time''
````

<a id="w001803"></a>

## W001803 — C

Bron-ID: W001803
Namespace: Main (0)
Revisie: 22953 | Datum: 2018-09-25T20:25:59Z
Bron: https://wiki.osdev.org/C
Revisie-URL: https://wiki.osdev.org/index.php?title=C&oldid=22953

````text
C is a minimalistic programming language which is platform independent and requires no runtime. Because of these qualities it has become by far the most used language in the field of OS development and system programming. It can be compared to a "portable [[Assembly]]", as it was initially designed and used for.

== Compilers ==

A complex compiler is not required to compile C because of its minimalistic qualities. As a result many various compilers have been written for C and many will work for OS development. Today the two most common are [[GCC]] and Microsoft VC++. GCC being the far most common in open source development.

== Libraries ==
{{Main|C Library}}

The C library implements the standard C functions (i.e., the things declared in <stdlib.h>, <math.h>, <stdio.h> etc.) and provides them in binary form suitable for linking with user-space applications.

In addition to standard C functions (as defined in the ISO standard), a C library might (and usually does) implement further functionality, which might or might not be defined by some standard. The standard C library says nothing about networking, for example. For Unix-alike systems, the POSIX standard defines what is expected from a C library; other systems might differ fundamentally.

It should be noted that, in order to implement its functionality, the C library must call kernel functions through system calls. So, for your own OS, you can of course take a ready-made C library and just recompile it for your OS - but that requires that you tell the library how to call your kernel functions, and your kernel to actually provide those functions.

It should also be noted that you can not use a usermode C library directly in your kernel as kernel code needs to be compiled specially. Things such as malloc, free, memcpy need to implemented by you before being used.

== Things C can't do==

Because of the platform independent nature of C some parts of OS development can only be done in Assembly. Others can be implemented using inline Assembly, but still require knowledge outside the realm of your high-level language of choice.

The canonical example of code that must be written in pure Assembly is the first-stage bootloader. If you choose to write your own instead of using an existing solution such as GRUB, it must be written in Assembly, as it requires direct manipulation of certain registers; specifically, the segment selectors and the stack pointer, which would not be possible in C itself.

Other functions, such as loading the Global Descriptor Table (on the IA32 architecture), also require special opcodes which are not available within C language (but can be implemented in [[Inline Assembly]]). In the (unlikely) case your compiler does not support inline Assembly, you have the option of writing 'support functions' in a separate assembly file. Note that we have a page with [[Inline Assembly/Examples|examples]] for basic operations using GCC.

[[Interrupt Service Routines]] (ISRs) also require some special handling, because they are called directly by the CPU, not by the C environment.

== Things you should know about optimizations ==

At some point you might want to compile your code with optimizations enabled. This however can lead to unpredictable results if your C code wasn't written properly. You might end up wasting days or weeks trying to find out why your code works flawlessly without optimizations enabled, but as soon as you compile it with optimizations it just doesn't work anymore or even crashes or locks up your OS. Operating systems are usually multi-threaded environments these days, however the optimizer of compilers generally assume that algorithms and memory changes are single-threaded and so only visible to one instance. So if the optimizer determines that your code does something that is redundant and could be optimized away, it might remove code or change it's logic any way it wants. These are legit changes to the code, but in some instance it's not what you want and it will simply break it. To prevent such things from happening, use the [[volatile (keyword)|volatile]] keyword where neccessary.

To fix possible issues due to optimizations performed by processors on runtime (e.g. reordering instructions and memory accesses, caching memory) make sure you use [[atomic operation]]s or [[memory barriers]] where neccessary. This however isn't an optimization specific to the C/C++ language but rather platform and hardware specific.

However, although optimizations increase the severity and apparent number of small bugs in C code, they will never break correct C, assuming the compiler is stable and multi-threading issues are accounted for. Because of this, it is sometimes best to debug with optimizations enabled at all different levels, to make sure that code is absolutely correct. If optimizations are turned off, the bugs are still there, just less noticeable. A release-quality project should work while enabling any combination of optimizations, although it is much harder to make progress while doing so.

== See also ==

* [[Books#C|Learning C]]
* [[C preprocessor]]
* [[C++]]
* [[C MinusMinus|C--]]
* [[Bare Bones]]

[[Category:C]]
[[Category:Languages]]
[[de:C]]
````

<a id="w001804"></a>

## W001804 — Talk:Assembly

Bron-ID: W001804
Namespace: Talk (1)
Revisie: 11404 | Datum: 2011-04-18T14:17:46Z
Bron: https://wiki.osdev.org/Talk:Assembly
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AAssembly&oldid=11404

````text
==0x123==
In the AT&T Syntax section the 123 which has been used in the rest of the document has been changed to 0x123. I believe that this is misleading. The values in this section should match those in the others. -[[User:Jhawthorn|Jhawthorn]] 00:05, 8 February 2007 (CST)

I quickly ran a test, using plain numbers work both as immediates and memory operands in GAS, and this creates the same output as in the NASM counterpart. Hence 0x123 is not just misleading, its incorrect. Fixing it - [[User:Combuster|Combuster]] 06:12, 8 February 2007 (CST)

== Rollbacks / undos on 2011-04-18 ==

Sorry for the messed-up history, I confused rollback and undo. On the subject: The discussions of the pros and cons of Intel vs. AT&T syntax have been numerous and heated. I would prefer to keep any "advantages / disadvantages" discussion out of the article, and have everybody make up his own mind. -- [[User:Solar|Solar]] 08:53, 18 April 2011 (UTC)

Wasn't trying to list any of the subjective advantages/disadvantages, only the facts; nor favour any of the two syntaxes. I just wanted to list the more obvious ones (e.g., the % prefix in AT&T works great for extensibility, since it lets you access the register namespace) so people know what they get into when they make their decision. --[[User:Love4boobies|Love4boobies]] 14:17, 18 April 2011 (UTC)
````

<a id="w001807"></a>

## W001807 — System Initialization (SGI Mips)

Bron-ID: W001807
Namespace: Main (0)
Revisie: 29801 | Datum: 2025-12-25T05:18:23Z
Bron: https://wiki.osdev.org/System_Initialization_(SGI_Mips)
Revisie-URL: https://wiki.osdev.org/index.php?title=System_Initialization_%28SGI_Mips%29&oldid=29801

````text
{{Stub}}

At boottime, a SGI system will load the ARCS firmware which then initializes basic hardware (such as the framebuffer, serial ports, keyboard, and disks), test system functionality and optionally loads a primary bootloader or kernel. Otherwise, it gives the user a graphical menu if a keyboard has been attached.

The ARCS firmware was made in both 32 bit and 64 bit versions. The 32 bit versions are loosely based on the [[ARC]] firmware specification, but as with all ARC systems there were several deviations. The differences between the 32 bit and 64 bit version are not well documented. A system with a 64 bit processor does not necessarily have a 64 bit firmware - the Power Indigo 2, Indigo 2 R10000, Origin, Onyx 2, and Octane are 64 bit ARCS, for example, but the Indy, which has a 64 bit processor, runs a 32 bit version of ARCS.

The 32 bit and early 64 bit versions of the ARCS firmware expect a boot payload in ECOFF format, an extended variant of [[COFF]]. Later 64 bit versions of the firmware (as well as a later 32 bit firmware for the R5000 Indy) will also accept [[ELF]] binaries.

== Pre-ARCS Firmware SGIs ==
All SGIs under the Personal IRIS, Power Series, Professional IRIS and IRIS 4D (but not the IRIS Indigo) use a PROM firmware distinct from the later ARCS firmware. 

== ARCS Firmware SGIs ==
Starting with the SGI IRIS Indigo ARCS is the firmware used with a full graphical interface for startup.<ref>https://tech-pubs.net/wiki/index.php/ARCS</ref>

== ARCS Environment Variables ==
The following variables are relevant in terms of getting ARCS to boot:
 OSLoadPartition	The disk partition where the operating system kernel is located. This is also used as the default root partition and is set automatically at system startup. Takes the format: dksc(0,1,0) (first number is controller, second is device, third is partition number)
 
 OSLoader The operating system loading program. By default, this is sash (the stand-alone shell). This is set automatically at system startup.
 
 SystemPartition	The disk partition where the operating system loading program is found. This is set automatically at system startup. Takes the format: dksc(0,1,0) (first number is controller, second is device, third is partition number)
 
 OSLoadFilename The filename of the operating system kernel. By default, this is /unix. This variable is automatically set at system startup.
 
 OSLoadOptions This variable specifies options to the boot command used to load the operating system.
 
 AutoLoad This variable specifies whether the operating system will boot automatically after a reset or power cycle. This variable supersedes bootmode and can be set to yes or no. This variable interacts with the rebound variable and the reboot_on_panic kernel tunable parameter.
Further reading can be had at [https://www.tech-pubs.net/SGI_EndUser/books/IRIX_EnvVar/sgi_html/ch02.html this guide]
[[Category:MIPS]]
[[Category:Booting]]
````

<a id="w001808"></a>

## W001808 — User:Gh975223/Xtalk

Bron-ID: W001808
Namespace: User (2)
Revisie: 3202 | Datum: 2007-03-07T05:20:17Z
Bron: https://wiki.osdev.org/User:Gh975223/Xtalk
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGh975223%2FXtalk&oldid=3202

````text
Xtalk, pronounced crosstalk,otherwise known as XIO. Xtalk is a bus used in SGI systems like Octane
````

<a id="w001812"></a>

## W001812 — Category:PowerPC

Bron-ID: W001812
Namespace: Category (14)
Revisie: 3003 | Datum: 2007-02-14T22:29:10Z
Bron: https://wiki.osdev.org/Category:PowerPC
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3APowerPC&oldid=3003

````text
[[Category:Platforms]]
````

<a id="w001819"></a>

## W001819 — User talk:Gh975223

Bron-ID: W001819
Namespace: User talk (3)
Revisie: 3007 | Datum: 2007-02-14T22:36:33Z
Bron: https://wiki.osdev.org/User_talk:Gh975223
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AGh975223&oldid=3007

````text
== Wiki Etiquette ==
First of all, I'd like to thank you for your interest in this wiki. However, would you mind not changing people's posts unless its really necessary, thx - [[User:Combuster|Combuster]] 14:05, 11 February 2007 (CST)

==WIP pages==
I've moved some work in progress pages that you have written under the [[Special:Prefixindex/User:Gh975223|User:Gh975223 prefix]]. When you have finished them feel free to move them back into the main namespace. Also, the general consensus is that the content of these pages should not be categories. -[[User:Jhawthorn|Jhawthorn]] 16:36, 14 February 2007 (CST)
````

<a id="w001820"></a>

## W001820 — Win95 DOS Call

Bron-ID: W001820
Namespace: Main (0)
Revisie: 28471 | Datum: 2023-11-16T15:49:20Z
Bron: https://wiki.osdev.org/Win95_DOS_Call
Revisie-URL: https://wiki.osdev.org/index.php?title=Win95_DOS_Call&oldid=28471

````text
{{Stub}}

Windows 95, while not precisely a disk image utility itself, gives a programmer an easy access method for reading and writing data from/into disk image files. This comes from the total lack of security features in the OS. The method is the DOS interrupt call: INT21h, ax=0x7305. This might make Windows 95 a better choice for an OS development platform than Windows 98.

==Usage==
The main reason to be working with disk images when doing OS development is that they are needed for [[emulators]] such as [[Bochs]]. If you are using a non-standard [[filesystem]] in the emulator, then you must use a disk [[hexeditor]], or read and write sectors in your disk image file directly, because no host OS will recognize your non-standard filesystem.

This interrupt call gives a programmer a method to read and write logical disk sectors. You need to use some utility to find the sector location on your disk partition of your disk image.

Create a 10 byte "DISKIO" structure in memory, consisting of the base sector number (4 bytes), number of sectors to read/write (2 bytes), and a pointer to your data buffer (4 bytes). (I'm not sure why it's a 4 byte pointer, as the rest of the call seems to be in real mode?)

Set si to 0x6001 for a write, or 0 for a read. cx = 0xffff, ds:bx points to the DISKIO structure, dl = logical drive letter (1= A, 2= B, ... ), ax = 0x7305. Then do an INT 0x21.

[[Category:Disk Image Utilities]]
[[Category:Windows]]
````

<a id="w001821"></a>

## W001821 — Norton Diskedit

Bron-ID: W001821
Namespace: Main (0)
Revisie: 14304 | Datum: 2013-01-16T05:29:00Z
Bron: https://wiki.osdev.org/Norton_Diskedit
Revisie-URL: https://wiki.osdev.org/index.php?title=Norton_Diskedit&oldid=14304

````text
Norton Diskedit is a [[hexeditor]] for logical and physical disk drives on all Windows [[File Systems|file systems]]. It is an undocumented utility that comes along with the standard Norton Utilities package for Windows. (There may also be a version that supports Apple filesystem formats on the Mac version of Norton Utilities?)

==Features==
Diskedit is able to do some kind of magic to read and modify sectors on any logical or physical disk attached to a PC, whether the disk is formatted or not. It runs under DOS, either in a window, or in singletasking mode. When running in a DOS window, it reverts to "Read Only" mode, but can still read any sector on any physical or logical disk, and analyze the directory and allocation structure of the disk. In singletasking DOS mode, it is able to copy sectors from one disk to another, files from one disk to another, or modify individual bits on individual sectors. It can be used to find the cluster number, physical sector, or logical sector where a file is stored.

==Usage==
Diskedit is very useful as a disk image utility. It will analyze a filesystem's structure to help you locate the position of the disk image precisely on your hard drive. If you have data or binary images stored on a disk, you can open them and then store them into the disk image (once you know its location). Since it can copy raw sectors, it is not necessary for the disk image to be in any recognizable filesystem format.

==Helpful Hints==
Diskedit tries to scan your entire filesystem when it is started, to locate every file, and verify filesystem integrity. So it is nice to start it from a small ramdrive that contains the files you are interested in. If it is possible, somehow, to break the Read Only "feature" in Diskedit it would become an even better tool. A programmer could run Bochs in one DOS window, compile kernels in another, and transfer them into the Bochs disk image immediately with Diskedit in another. As it is, it is necessary to shut Windows down to DOS singletasking mode in order to do the Diskedit transfer, and reboot Windows.


{{Stub}}

[[Category:Disk Image Utilities]]
````

<a id="w001824"></a>

## W001824 — GCC Canadian Cross

Bron-ID: W001824
Namespace: Main (0)
Revisie: 22688 | Datum: 2018-09-10T01:13:02Z
Bron: https://wiki.osdev.org/GCC_Canadian_Cross
Revisie-URL: https://wiki.osdev.org/index.php?title=GCC_Canadian_Cross&oldid=22688

````text
{{Stub}}
A Canadian Cross is to first build a cross-compiler for system Y (the host) on system X (the build-system). You would then use that cross-compiler, setting <code>--build=<where this compiler runs> --host=<where the compiler you're building shall run> --target=<where the executables the compiler you're building will build shall run></code>. This explanation alone should be sufficient to show why this is both non-trivial and error-prone.

==="Minor" Canadian Cross===

Building a cross-compiler takes some time, so you might want to use a faster machine (like, a PIV) to build a cross-compiler for a lesser machine (like, a Pentium laptop). You don't really need the initial cross-compiler for this, as the system compiler on the build machine (PIV) can generate binaries for the host machine (Pentium) alright.

[[Category:Compilers]]
````

<a id="w001825"></a>

## W001825 — Chip Numbers, Acronyms and Things

Bron-ID: W001825
Namespace: Main (0)
Revisie: 17416 | Datum: 2015-01-01T19:25:29Z
Bron: https://wiki.osdev.org/Chip_Numbers%2C_Acronyms_and_Things
Revisie-URL: https://wiki.osdev.org/index.php?title=Chip_Numbers%2C_Acronyms_and_Things&oldid=17416

````text
;6845:Graphics controller chip used in many many graphic adaptors
;8042:Controller chip in the AT keyboard
;8048:Controller chip in the XT keyboard
;82072A:Controller chip in the AT floppy disk drive
;82077:Controller chip in PS/2 floppy disk drive. Successor to the 82072A
;82284:The 80286 clock generator
;82288:The 80286 bus controller chip
;8237A:An 8=bit DMA controller in the XT. Two chips in the AT and beyond, with four DMA channels per chip. Chips can be cascaded.
;16450:UART chip in the AT, successor to the 8250.
;82489DX:The original Advanced Programmable Interrupt Controller (APIC) used in 486 and early Pentium multiprocessor systems. Later Pentiums integrate the local APIC onto the processor itself. The successor to the 8259A PIC.
;8253:Programmable Interval Timer ([[PIT]]) chip in the XT. Has three independent timers.
;8254:Programmable Interval Timer ([[PIT]]). Used in AT's and EISA/MCA machines. Just an improved version of the 8253.
;8259A:Programmable Interrupt Controller ([[PIC]]) used in all non-SMP PC's that predate the Pentium III/Athlon (current systems still include it for backward compatibility). Each chip has 8 interrupt lines. AT and beyond have two chips.
;8284:Clock generator in the 8088/8086/80186
;8288:Bus controller in the 8088/8086/80186
;8741:Common controller chip in AT keyboards
;8742:Controller chip in PS/2 keyboards
;ABIOS:Advanced BIOS. PS/2 BIOS that can be called in both Protected Mode and Real Mode.
;ARLL:Advanced RLL. Method used in Hard Disks. MFM, RLL, etc.
;ASPI:Advanced SCSI Programming Interface. An Adaptec thing, its a common interface for programming SCSI devices.
;ATA:AT Attachment. A bus standard for connecting hard disks and CD-ROMs.
;BIST:Built-in Self Test.
;Booting:The loading of the BIOS and kicking in the bootsector to bootstrap an OS loader.
;Bootstrap:The bootstrap is a small program that loads an operating system. (Usually synonymous with the bootsector/MBR). Also used to describe the process of setting up a compiler on a new system not having one already.
;Bus Master:Where a device can control the BUS autonomously. Examples of this are the CPU and DMA chips. Other controller chips can also do this (ala some PCI devices, etc).
;Call Gate:Much like an software driven interrupt, a call gate allows access to other code from a different privilege level.
;Combicontroller:Combine a Floppy disk controller chip and a Hard disk controller chip and you have a combicontroller. It is not limited to just floppy+hard drive chips. Usually meant to refer to the actual plug-in card where you have two floppy and two hd IDE ports.
;CPL:Current privilege level. The current [[Protected Mode|ring]] in which code is being executed.
;Descriptor:An 8 byte structure that describes a segment/gate/task in protected mode.
;DMA:[[DMA|Direct Memory Access]]. Allows peripheral devices to access main memory directly, bypassing the CPU.
;DPL:Descriptor privilege level. The privilege needed to use a descriptor in the [[GDT]] or [[IDT]]
;EISA:Extended ISA. Basically extends the ISA bus from being 16-bit to being 32-bit. Designed for 386 + 486 systems.
;ESDI:Enhanced Small Device Interface. A hard disk controlling mechanism. Successor to the ST506/412 interface. Can handle pumping data out at 24mbits (3mb a second)
;Exception:CPU error. Usually hardware triggered, but can also be software triggered (some debug breakpoints, etc)
;Expanded Memory:Memory used by an EMS driver. Located beyond 1mb mark.
;Extended Memory:Memory used by an XMS driver. Located beyond the 1mb mark. The first 64kb of XMS is known as HMA (High Memory Area).
;Fault:An exception that is recognized by the processor before the CPU executes the code. eg: trying to access memory swapped out to disk triggers a fault BEFORE the memory is accessed so it can be swapped into memory.
;FDC:Floppy Disk Controller
;FIFO:First In First Out. FIFO buffers are common in newer UART chips for communications (16550A)
;GDT:Global Descriptor Table. This table contains descriptors that are potentially available to all programs in protected mode.
;HAL:Hardware Abstraction Layer. Used by Windows NT and some other operating systems.
;HDC:Hard Disk Controller
;IDE:Intelligent Drive Electronics. Another name for ATA.
;IDT:Interrupt Descriptor Table. Table of 8-byte entries that describe interrupts, traps, exceptions and fault handlers to the CPU.
;Interrupt:Interrupts can be software or hardware generated. When an interrupt occurs, the CPU jumps to code assigned to be ran whenever that interrupt is triggered.
;Interrupt Gate:A descriptor for calling an interrupt.
;IRQ:Interrupt Request. Hardware signal to the CPU from an external peripheral.
;ISA:Industry Standard Architecture. The defined bus standard for AT's.
;LDT:Local Descriptor Table. A table of descriptors that can only be accessed by the task that owns the LDT and none other.
;MC146818:CMOS RAM and Real Time Clock chip in the AT.
;MMU:Memory Management Unit. The MMU is often contained within the CPU but can be external to the CPU. eg: old 68k CPU's did not have an MMU but one could be attached externally. The MMU is responsible for doing things like address segmentation translation and paging.
;PD765:The floppy controller chip in the XT
;Multitasking:Where the CPU 'appears' to be running several tasks all at the same time.
;Nibble,Nybble:Group of 4 bits. Half a byte.
;NMI:[[Non Maskable Interrupt]]. An interrupt request sent to the CPU that must be dealt with immediately.
;Page:Section of memory that is classed as a single entity. eg: x86 in pmode has pages of 1byte, 4kb and 4mb in size.
;Page Directory:Holds the entries for the paging table mechanism. Unlike Page Tables, the Page Directory can't be swapped out of memory.
;PCI:Peripheral Component Interconnect. A Local Bus standard running at 32bits at 33mhz (can go higher). PCI has replaced ISA as the most common bus interface for desktop computers.
;PCMCIA:Personal Computer Memory Card International Association. Small credit card sized units that plug into portable computer PCMCIA slot and provide things like extra memory, modems, etc.
;PIC:Programmable Interrupt Controller. The PIC manages hardware interrupts.
;PIT:Programmable Interval Timer. The PIT is like a stopwatch with periodic alarm...
;POST:Power On Self Test. Your computer does this when you turn it on in order to assess that its 100% A-OK and can boot up without error.
;RPL:Requested Privilege Level. Used in [[Protected Mode]] to pretend running at a lower privilege level for accessing a descriptor.
;RTC:Real Time Clock
;SCSI:Small Computer Systems Interface. A different method from ATA/IDE for connecting devices to your computer.
;Selector:An index into a descriptor table.
;ST506/412:Physical interface between a hard disk and computer.
;Task:AKA a process or job. A task is just another program.
;Task Switch:Changing from one task to another in an multi-tasking operating system.
;Trap:An exception which occurs AFTER the error has occurred instead of before (see Fault).
;TSS:Task State Segment. A data structure that holds the state of the CPU when for that task. Used by multitasking OS.
;VESA Local Bus (VLB):Bus system for 486 machines. CPU speed specific.

[[Category:Resources]]
````

<a id="w001827"></a>

## W001827 — User:Gh975223/i960

Bron-ID: W001827
Namespace: User (2)
Revisie: 2991 | Datum: 2007-02-14T22:18:41Z
Bron: https://wiki.osdev.org/User:Gh975223/i960
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGh975223%2Fi960&oldid=2991

````text
'''Registers'''
{| {{wikitable}} width="75%"
|-
! Name
! Number
! Function
! Callee must preserve?
|-
! R0-R15
| 0-15
| 
| 
|-
! G0-G15
| 16-31
| 
| 
|-
! sp
| 1
| 
| 
|-
! fp
| 31
| 
| 
|}


{{In Progress}}
````

<a id="w001828"></a>

## W001828 — User:Gh975223/IA32

Bron-ID: W001828
Namespace: User (2)
Revisie: 2992 | Datum: 2007-02-14T22:19:36Z
Bron: https://wiki.osdev.org/User:Gh975223/IA32
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGh975223%2FIA32&oldid=2992

````text
8 bit registers AH AL BH BL CH CL DH DL

16 bit Registers AX BX CX DX BP SP SI DI IP FLAGS CS SS DS ES FS GS

32 bit Registers EAX EBX ECX EDX EBP ESP ESI EDI EIP EFLAGS MXCSR

48 bit Registers

8x 64 bit Registers

8 x 80 bit Registers

8 x 128 bit Registers
````

<a id="w001829"></a>

## W001829 — User:Gh975223/68K

Bron-ID: W001829
Namespace: User (2)
Revisie: 2993 | Datum: 2007-02-14T22:21:11Z
Bron: https://wiki.osdev.org/User:Gh975223/68K
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGh975223%2F68K&oldid=2993

````text
Mortorola 68K series of processors are used in computer architecture such as Atari, Amiga and the Apple MacIntosh. The instruction set is noted for being orthogonal.

'''Registers'''
{| {{wikitable}} width="75%"
|-
! Name
! Number
! Function
! Callee must preserve?
|-
| D0-D7
| 
| General purpose Data registers
| 
|-
| A0-A7
| 
| General purpose Address registers
| 
|-
|PC 
| 
| Program counter
| 
|}

{{In Progress}}
````

<a id="w001830"></a>

## W001830 — User:Gh975223/ARM

Bron-ID: W001830
Namespace: User (2)
Revisie: 2994 | Datum: 2007-02-14T22:23:13Z
Bron: https://wiki.osdev.org/User:Gh975223/ARM
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGh975223%2FARM&oldid=2994

````text
The ARM architecture, is currently used in legacy Acorn Archimedes, Acorn RiscPC and PDA computers.


{{In Progress}}


'''Registers'''
{| {{wikitable}} width="75%"
|-
! Name
! Number
! Function
! Callee must preserve?
|-
! 
| R0-R12
| general purpose registers
| 
|-
! SP
| R13
| stack pointer
|
|-
! 
| R14
| subroutine link register
|
|-
! PC
| R15
| program counter
|
|-
! CPSR
| R16
| state register
|
|}


'''Register Set'''
{| {{wikitable}} width="75%"
|-
! System & User
! FIQ
! Supervisor
! Abort
! IRQ
! Undefined
|-
| R1
| R1
| R1
| R1
| R1
| R1
|-
| R2
| R2
| R2
| R2
| R2
| R2
|-
| R3
| R3
| R3
| R3
| R3
| R3
|-
| R4
| R4
| R4
| R4
| R4
| R4
|-
| R5
| R5
| R5
| R5
| R5
| R5
|-
| R6
| R6
| R6
| R6
| R6
| R6
|-
| R7
| R7
| R7
| R7
| R7
| R7
|-
| R8
| R8_fiq  &dagger;
| R8
| R8
| R8
| R8
|-
| R9
| R9_fiq  &dagger;
| R9
| R9
| R9
| R9
|-
| R10
| R10_fiq &dagger;
| R10
| R10
| R10
| R10
|-
| R11
| R11_fiq &dagger;
| R11
| R11
| R11
| R11
|-
| R12
| R12_fiq &dagger;
| R12
| R12
| R12
| R12
|-
| R13
| R13_fiq &dagger;
| R13_svc &dagger;
| R13_abt &dagger;
| R13_irq &dagger;
| R13_und &dagger;
|-
| R14
| R14_fiq &dagger;
| R14_svc &dagger;
| R14_abt &dagger;
| R14_irq &dagger;
| R14_und &dagger;
|-
| R15
| R15
| R15
| R15
| R15
| R15
|-
| CPSR
| CPSR
| CPSR
| CPSR 
| CPSR 
| CPSR
|-
| 
| SPSR_fiq &dagger;
| SPSR_svc &dagger;
| SPSR_abt &dagger;
| SPSR_irq &dagger;
| SPSR_und &dagger;
|}
&dagger; =Banked register


'''ARM710 CPO Register Set'''
{| {{wikitable}} width="75%"
|-
! Register
! Function
! R/W
|-
| 0
| Processor identification
| R
|-
| 1
| Control
| W
|-
| 2
| Translation Table Base
| W
|-
| 3
| Domain Access Control
| W
|-
| 4
| Reserved
| n/a
|-
| 5
| Page fault status / TLB flush
| R/W
|-
| 6
| Data fault address / TLB purge
| R/W
|-
| 7
| IDC flush
| W
|-
| 8-15
| Reserved
| n/a
|}
````

<a id="w001831"></a>

## W001831 — User:Gh975223/Alpha

Bron-ID: W001831
Namespace: User (2)
Revisie: 2995 | Datum: 2007-02-14T22:23:46Z
Bron: https://wiki.osdev.org/User:Gh975223/Alpha
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGh975223%2FAlpha&oldid=2995

````text
The DEC Alpha microprocessor was developed as a replacement for VAX microprocessor.
````

<a id="w001832"></a>

## W001832 — MIPS Overview

Bron-ID: W001832
Namespace: Main (0)
Revisie: 29859 | Datum: 2026-01-11T19:10:23Z
Bron: https://wiki.osdev.org/MIPS_Overview
Revisie-URL: https://wiki.osdev.org/index.php?title=MIPS_Overview&oldid=29859

````text
{{In Progress}}

MIPS (Microprocessor without Interlocked Pipelined Stages) is a RISC CPU architecture used in systems like SGI O2 and Octane systems, Nintendo N64 as well as the Sony Playstation, Playstation 2 and Playstation Portable.

== General Registers ==
{| {{wikitable}} width="75%"
|-
! Name
! Number
! Function
! Callee must preserve?
|-
! $zero
| $0
| constant 0
| n/a
|-
! $at
| $1
| assembler temporary
| no
|-
! $v0–$v1
| $2–$3
| values for function returns and expression evaluation
| no
|-
! $a0–$a3
| $4–$7
| function arguments
| no
|-
! $t0–$t7
| $8–$15
| temporaries
| no
|-
! $s0–$s7
| $16–$23
| saved temporaries
| yes
|-
! $t8–$t9
| $24–$25
| temporaries
| no
|-
! $k0–$k1
| $26–$27
| reserved for OS kernel
| no
|-
! $gp
| $28
| global pointer
| yes
|-
! $sp
| $29
| stack pointer
| yes
|-
! $fp/$s8
| $30
| frame pointer
| yes
|-
! $ra 
| $31
| return address
| n/a
|}
'''Note:''' All except registers on the MIPS except $zero, HI and LO are general registers; the listed usage is per convention and not enforced by the processor or the assembler. The register name $s8 is a synonym for $fp used in some assemblers, in systems where the frame pointer is not regularly used.

== Arithmetic Registers ==
{| {{wikitable}} width="50%"
! Register
! Multiplication
! Division
|- 
| HI
| Multiplicand Upper word
| Quotient
|-
| LOW
| Multiplicand Lower word
| Remainder
|}

== Coprocessor 0 Registers ==
{| {{wikitable}} width="75%"
|-
! Name
! Number
! Function
! Callee must preserve?
|-
| c0_index
| cop0 $0
| TLB entry index register
| n/a
|-
| c0_random
| cop0 $1
| TLB randomized access register
| n/a
|-
| c0_entrylo
| cop0 $2
| Low-order word of "current" TLB entry
| n/a
|-
| c0_context
| cop0 $4
| Page-table lookup address
| n/a
|-
| c0_vaddr
| cop0 $8
| Virtual address associated with certain exceptions
| n/a
|-
| c0_entryhi
| cop0 $10
| High-order word of "current" TLB entry
| n/a
|-
| c0_status
| cop0 $12
| Processor status register
| n/a
|-
| c0_cause
| cop0 $13
| Exception cause register
| n/a
|-
| c0_epc
| cop0 $14
| PC at which exception occurred
| n/a
|}

== Instruction fields ==
{| {{wikitable}} width="75%"
! Field
! Size
! Position
! Op Types
! Description
|-
! op
| 6
| 26-31
| R, I, J
| opcode for the instruction or group of instructions.
|-
! rs
| 5
| 21-25
| R, I
| Source register for store operations, destination for all other operations.
|-
! rt
| 5
| 16-20
| R, I
| First operand register. 
|-
! rd
| 5
| 11-15
| R
| Second operand register.
|-
! shift
| 5
| 6-10
| R
| Immediate operand for shift and rotate instructions. 
|-
! func
| 6
| 0-5
| R
| Extended opcode.
|- 
! imm
| 16
| 0-15
| I
| Half-word immediate operand.
|-
! address
| 
| 0-25
| J
| 26-bit address field for unconditional jump operations.
|}

== Addressing modes ==
{| {{wikitable}} width="80%"
|-
! Type
! Assembly Format
! Opcode format
! Comments
|-
| Register
| inst rs, rd, rt
| op rs, rd, rt, shift, func
| The opcode represents a group of operations rather than a specific instruction; the func field contains the actual operation. The shift field is only used in shift and rotate operations.
|-
| Immediate (I-type)
| inst rs, rt, imm
| opcode rs, rt, imm
| Immediate operation use a 16-bit immediate value from the instruction word itself. 
|-
| Load (I-type)
| inst rs, imm(rt)
| op rs, rt, imm
| Load/Store operations are a special case of immediate, where the offset is the immediate operand.
|-
| Store (I-type)
| inst rs, offset(rt)
| op rs, rt, imm
| Unlike in most other operations, the rs register is the data source.
|-
| Cond. Branch(I-type)
| inst rs, rt, label
| op rs, rt, imm
| Conditional branches have a 16-bit relative range.
|-
| Jump {J-type)
| inst label
| op address
| the J and JAL operations have a 26-bit relative range.
|}
'''Note:''' The assembly formats given are those from the official MIPS Technologies documentation. Other assemblers (e.g., gas) may use different formats. 

== See Also ==

=== Articles ===
* [[System Initialization (SGI Mips)]]

=== References ===
* [http://en.wikibooks.org/wiki/MIPS_Assembly/Instruction_Formats Wikibooks: MIPS Assembly - Instruction Formats]
* [https://web.archive.org/web/20070920180642/https://www.d.umn.edu/~gshute/spimsal/talref.html MIPS Instruction Coding]
* [http://www.student.cs.uwaterloo.ca/~isg/res/mips/opcodes MIPS Encoding Reference]

[[Category:MIPS]]
[[Category:Instruction Set Architecture]]
````

<a id="w001833"></a>

## W001833 — Sparc Overview

Bron-ID: W001833
Namespace: Main (0)
Revisie: 27547 | Datum: 2022-12-25T05:49:21Z
Bron: https://wiki.osdev.org/Sparc_Overview
Revisie-URL: https://wiki.osdev.org/index.php?title=Sparc_Overview&oldid=27547

````text
SPARC (the Scalable Processor ARChitecture) is a RISC-based processor mainly used in high-end workstations and servers; in particular, Sun's SparcStation and many of Sun's Ultra machines are SPARC based. Notable features of Sun's implementation include:
* [[OpenBoot]] (aka OpenFirmware, IEEE 1275)

As Sparc hardware generally is highly expensive, few hobby OS's target it; however, as more and more older SPARCs are being replaced, an interested developer can often find them at swap meets and in discard piles, where they are available at little cost.

As a side effect of their high cost, little documentation is available for Sun's workstations at a hardware level, and what is available from Sun often requires a support contract. Fortunately, several free operating systems support the Sparc, such as Linux and *BSD.

The Sparc processor has several versions:
* V8: 32-bit, used in Sun's SparcStations
* V9: 64-bit, used in Sun's ULTRA workstations. Also known as UltraSparc.


'''Registers'''
{| {{wikitable}} width="75%"
|-
! Name
! Number
! Function
! Callee must preserve?
|-
! %g0
| $0
| constant 0
| 
|-
! %g1-%g7
| $1-$7
| global registers 1-7
|
|-
! %o0-%o5
| $8-$13
| output registers 0-5
|
|-
! %sp
| $14
| stack pointer
|
|-
! %o7
| $15
| subroutine return address
|
|-
! %l0-%l7
| $16-$23
| local registers 0-7
|
|-
! %i0-%i5
| $24-$29
| input registers 0-5
|
|-
! %fp
| $30
| frame pointer
|
|-
! %i7
| $31
| function return address
|
|}

== See Also ==

=== Articles ===
* [[System Initialization (Sparc)]]
* [[Sparc Hardware Docs]]

=== Links ===
* [http://web.archive.org/web/20090506092708/http://www.owlnet.rice.edu/~comp320/2007/html/sparc/ http://www.owlnet.rice.edu/~comp320/2007/html/sparc/]

[[Category:Sparc]]
````

<a id="w001834"></a>

## W001834 — PowerPC Overview

Bron-ID: W001834
Namespace: Main (0)
Revisie: 29665 | Datum: 2025-09-18T21:46:23Z
Bron: https://wiki.osdev.org/PowerPC_Overview
Revisie-URL: https://wiki.osdev.org/index.php?title=PowerPC_Overview&oldid=29665

````text
The PowerPC CPU architecture is significantly different from the IA32. Yet still, the architecture of your OS need not differ too much: While the way you address memory on the lowest levels might be different, or the way your SIMD unit operates, you still have a bootloader, a scheduler, a dispatcher, a memory manager etc. etc.

You will be able to transcribe most of the documentation 1:1 to the PowerPC, unless they handle the low levels (interrupt handling, real vs. protected mode etc.).

A good place to get started with PowerPC is via the POWER instruction set manual which can be found [https://wiki.raptorcs.com/w/images/f/f5/PowerISA_public.v3.1.pdf here] . This document contains a complete documentation of the POWER/PowerPC architecture including MMU and SIMD/Vector extensions.


But CPU docs won't suffice. You also need info on the motherboard / chipset / boot sequence etc. - and that's where it gets tricky, since there is no such thing as "the PowerPC architecture" - Apple doesn't like giving away that information, and there are many other platforms that might or might not be compatible.

A Linux-on-PPC project exists, and is probably a good place to look for PowerPC info...

<pre>
/usr/src/linux/arch/ppc/boot/prep/head.S
/*
* Boot loader philosophy:
*      ROM loads us to some arbitrary location
*      Move the boot code to the link address (8M)
*      Call decompress_kernel()
*        Relocate the initrd, zimage and residual data to 8M
*        Decompress the kernel to 0
*      Jump to the kernel entry
*            -- Cort
*/
</pre>
* http://penguinppc.org/ looks like a neat place to start.
* https://wiki.netbsd.org/ports/macppc/ also might be of some interest.

The BIOS in latest Apple Macintosh is named "Open Firmware". Open Firmware was initially a SPARC-stuff targeted at Sun stations. Extension cards and other hardware of the like that are OpenFirmware compliant should carry a Forth-written initialization code in their ROM. I suppose the main boot rom then scan memory for such ROMs and then interpret the FORTH code on them.

* http://www.firmworks.com/www/ofw.htm
* http://www.openfirmware.org/
* http://www.firmworks.com/open_firmware/literature/ofdesc.pdf (overview of OpenFirmware)

==PowerPC Emulators==
* [[QEMU]]
* [http://pearpc.sourceforge.net/ PearPC]

[[Category:PowerPC]]
[[Category:Instruction Set Architecture]]
````

<a id="w001835"></a>

## W001835 — Category:Platforms

Bron-ID: W001835
Namespace: Category (14)
Revisie: 3006 | Datum: 2007-02-14T22:30:06Z
Bron: https://wiki.osdev.org/Category:Platforms
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3APlatforms&oldid=3006

````text
[[Category:Hardware]]
````

<a id="w001840"></a>

## W001840 — System Initialization (x86)

Bron-ID: W001840
Namespace: Main (0)
Revisie: 27975 | Datum: 2023-07-09T17:20:20Z
Bron: https://wiki.osdev.org/System_Initialization_(x86)
Revisie-URL: https://wiki.osdev.org/index.php?title=System_Initialization_%28x86%29&oldid=27975

````text
When an x86-based computer is turned on (assuming that it turns on successfully), it begins a complex path to get to the stage
where control is transferred to your kernel's "main()" routine. The exact sequence of steps depends on what kind of device the computer
decides to boot from, and whether it uses the legacy [[BIOS]] boot method, or the new [[UEFI]] method. The new UEFI method completely
changes the entire boot process, and is covered in [[UEFI|another article]]. The intention is that the UEFI boot process will someday
completely replace the BIOS method in all new computers, but it may not succeed.

Described below is the traditional BIOS-based system initialization process.

==BIOS initialization==

The very first thing that happens when the machine is turned on, is that the CPU starts trying to run a program at the very end of the
4Gb memory area. At that location must be some ROM which contains a BIOS initialization program. The initialization
code can be large, and the ROM may be as large as 256Kb in size. An OS programmer cannot modify or control this stage of the process in any way.

* RAM detection -- the BIOS must use some RAM in order to perform its functions. To use the RAM, the BIOS must first detect the type and quantity of RAM chips installed. This can only be done (using chipset-specific methods) while the CPU is running code that is stored in <b>ROM</b>, specifically. Once the RAM has been detected, the BIOS may perform a simple memory test on it, and then the BIOS loads data and code into several memory areas in RAM: the [[Memory Map (x86)#BIOS Data Area (BDA)|BDA]], the [[Memory Map (x86)#Extended BIOS Data Area (EBDA)|EBDA]], and the 64K "BIOS area" at physical address 0xF0000 to 0xFFFFF. The BIOS also sets up a tiny stack somewhere in memory, and sets up the Real Mode IVT from physical address 0 to 0x3FF. Some of the physical memory between address 0xA0000 and 0xFFFFF is then set to "read only" mode using chipset-specific methods.

* Hardware detection/Initialization -- the BIOS detects, enumerates, configures, and initializes every bus, and almost every piece of hardware on the system, using values that the BIOS chooses. It stores a great deal of information about all of this hardware, for the OS to later parse. If the BIOS finds any ROM chips on any hardware, they are mapped (not loaded) into physical memory at addresses that the BIOS chooses. It is important to note that many BIOSes do a rather bad job of configuring, sometimes. An OS may well need to reconfigure the [[MTRR]]s, or the [[PCI]] bus, or the mapping of some PCI devices. The BIOS is supposed to always bring all the hardware into a functional state, but that state may not be optimal, or even technically "legal" according to the specs.

* "Boot sequence" -- at this point, the BIOS is done with its initialization. Now it tries to transfer control to the next stage of the bootloader process; so the BIOS must choose the "boot device". There is a list stored in [[CMOS]], called the "boot sequence", that tells the BIOS which devices to test, and in what order, to see if they exist and are bootable. The BIOS may try to boot from a floppy disk, hard disk "C:", a USB flash memory device, a CD, a network, or something else. All of these devices can have some type of "bootsector", and there is a flag that the BIOS can check to see if the bootsector is valid. The BIOS will transfer control to the first valid bootsector that it finds, as it searches through the boot sequence. If the BIOS never finds a valid bootsector, it will lock up with an error message.

The BIOS transfers 512 bytes of data from each device that exists, into physical memory starting at address 0x7c00. If the last two bytes
transferred are 0x55, and then 0xAA, then the BIOS considers this to be a valid bootsector, and starts running the code that now begins at 0x7c00.

==Transfer of Control to Bootsector==

In most circumstances (except for CDs/DVDs) the bootsector will be LBA 0 of the device (or equivalently, CHS 0,0,1) -- see the specs of the
particular device to find out what addressing mode it uses. For CDs/DVDs, the bootsector is LBA 17. Traditionally, there is one additional
complication. Hard disks have been standardized as having "partitioning". That is, all operating systems allow you to subdivide a hard disk
into smaller "partitions" or "volumes". The way this is done is by adding an additional "layer" of standardized bootsectors, called MBRs
(Master Boot Records).

What this means in reality is that the BIOS boots some devices directly into your OS-specific bootloader, and the BIOS boots other
devices (any hard disk, or anything emulating a hard disk) into an MBR -- which in turn boots your OS-specific bootloader. So the
system always ends up running your OS-specific bootloader -- but it may happen directly, or there may be an intervening step.

This has implications if you want to write your own bootloader, and/or if you want to write your own MBR.

===System "Environment"===

There are very few things that are standardized about the state of the system, when the BIOS transfers control to the bootsector. The only things
that are (nearly) certain are that the bootsector code is loaded and running at physical address 0x7c00, the CPU is in 16-bit [[Real Mode]], the
CPU register called DL contains the "drive number", and that only 512 bytes of the bootsector have been loaded.

Note: there are apparently a tiny number of ancient BIOSes that actually put the system in Protected Mode, instead of Real Mode.
The recommendation is: do not support those BIOSes.

===OS-specific Bootloaders===

Every OS is expected to have its own bootloader, stored on the media with the kernel. The bootloader is expected to be stored at a
particular location, so that the BIOS or MBR can find it, load it, and run it. If the boot device is a hard disk (or something else
emulating a hard disk) then the bootloader is expected to be stored as the very first "block" of the partition. As said above, for
all other types of devices, the bootloader is stored at LBA 0, CHS 0,0,1, or LBA 17 (for CDs and DVDs).

The main function of these bootloaders is to find the kernel, wherever it is on the media, load it, and run it. Additionally, the
bootloaders need to set up a known environment for the kernel (which often includes switching to [[Protected Mode]]). The bootloaders
also might collect some system data for the kernel to use (some data is much easier to get while the system is still in Real Mode).

For more bootloader "theory" see the [[bootloader]] article.

There are [[:Category:Bootloaders|generic bootloaders]] available, or you can [[Rolling Your Own Bootloader|create your own]].

Either way, however, the OS-specific bootloader completes the process of system initialization.

===MBR Bootsectors===

Bootsectors for devices that can have multiple partitions have a standard [[MBR (x86)|format]].
Such devices always include hard disks, and can include USB Flash drives, or remote Network drives. The MBR bootsector is
created on the drive when the drive is partitioned by partitioning software (such as FDISK, under DOS/Windows).

The BIOS loads the entire MBR (both the Partition Table and Bootstrap code), and executes the bootstrap code. (See [[#System "Environment"|System "Environment"]]
above for additional details.)

The function of the [[MBR (x86)#MBR Bootstrap|MBR Bootstrap]] code is (usually) to search the [[Partition Table]] for the partition marked with an "active" flag
(flag byte equals 0x80), then load and run the bootsector (that is, the OS-specific Bootloader) of that partition (the very first sector).
This is what "generic" MBR Bootstraps do, such as the one that FDISK writes to a newly partitioned disk. The bootstrap code is expected to
pass a pointer (in DS:SI) to a memory copy of the booted partition table entry, to the OS-specific bootloader -- as well as reproduce the
above System "Environment" that the OS-specific bootloader expects to see.

Fancier MBR bootstraps can perform "Dual Booting". That is, they present a choice of disk drives and disk partitions to the user, and allow the
user to select a particular partition to boot -- rather than just automatically choosing the "active partition". These fancy MBR bootstraps often
use tricks to overcome that fact that they are limited to a little over 400 bytes of code.

See the [[MBR (x86)]] article for information on writing your own MBR.

==Notes==


==See Also==


[[Category:Booting]]
[[Category:X86]]
````

<a id="w001841"></a>

## W001841 — Category:X86 CPU

Bron-ID: W001841
Namespace: Category (14)
Revisie: 3030 | Datum: 2007-02-14T22:53:42Z
Bron: https://wiki.osdev.org/Category:X86_CPU
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AX86_CPU&oldid=3030

````text
[[Category:X86]]
````

<a id="w001842"></a>

## W001842 — Category:Power management

Bron-ID: W001842
Namespace: Category (14)
Revisie: 3039 | Datum: 2007-02-15T01:13:16Z
Bron: https://wiki.osdev.org/Category:Power_management
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3APower_management&oldid=3039

````text
[[Category:X86]]
````

<a id="w001844"></a>

## W001844 — Reboot

Bron-ID: W001844
Namespace: Main (0)
Revisie: 26084 | Datum: 2021-06-04T22:16:55Z
Bron: https://wiki.osdev.org/Reboot
Revisie-URL: https://wiki.osdev.org/index.php?title=Reboot&oldid=26084

````text
There are several methods for rebooting, including

# ACPI Reset command
# load a 0-sized IDT and issue an interrupt (that'll triple fault and reset)
# Use the 8042 keyboard controller to pulse the CPU's RESET pin
# Use the ResetSystem runtime [[UEFI]] service, if available.
# Far jump to the reset vector (0xFFFF:0000)/ Triple fault

== ACPI reset command ==
ACPI 2.0+ defines a "reset register" that can be used to reset the PC. According to the ACPI specification, ''' ''all'' ''' hardware must be reset after using this mechanism.
In the FADT, there are several General Address Structures. Take a look at the [[FADT#GenericAddressStructure|FADT]] page for details. The FADT structure contains a generic address structure called ResetReg, followed by a byte called ResetValue. This is an optional feature and software must check support for it by checking if the FADT is version 2 or newer, and then testing if bit 10 of the Flags field in the FADT is set.
After that, software can reset the system by writing the value in ResetValue into the area pointed to by ResetReg. ACPI says the ResetReg can only be located in I/O bus, PCI bus or memory-mapped. If it is in the I/O bus, simply do:
<source lang="c">
outportb(FADT->ResetReg.Address, FADT->ResetValue);
</source>
If it is memory-mapped, simply map the memory pointed by ResetReg->Address somewhere in the virtual address space, and write ResetValue there as an 8-bit operation. Like:
<source lang="c">
vmm_map(FADT->ResetReg.Address, 1); // Map 1 byte
uint8_t *reg = (uint8_t *)((uintptr_t)FADT->ResetReg.Address);
*reg = FADT->ResetValue;
</source>

But if it is on the PCI Configuration Space, you need to write ResetValue as a byte on:
* Segment: 0
* Bus: 0  
* Slot: (address >> 32) & 0xFFFF
* Function: (address >> 16) & 0xFFFF
* Offset: address & 0xFFFF
That operation in C looks like:
<source lang="c">
pci_writeb(0, 0, (FADT->ResetReg.Address >> 32) & 0xFFFF, (FADT->ResetReg.Address >> 16) & 0xFFFF, FADT->ResetReg.Address & 0xFFFF, FADT->ResetValue);
</source>

== Short code to do a 8042 reset ==
<source lang="c">
void reboot()
{
    uint8_t good = 0x02;
    while (good & 0x02)
        good = inb(0x64);
    outb(0x64, 0xFE);
    halt();
}
</source>

== Annotated code for reboot() ==
<source lang="c">
/* keyboard interface IO port: data and control
   READ:   status port
   WRITE:  control register */
#define KBRD_INTRFC 0x64

/* keyboard interface bits */
#define KBRD_BIT_KDATA 0 /* keyboard data is in buffer (output buffer is empty) (bit 0) */
#define KBRD_BIT_UDATA 1 /* user data is in buffer (command buffer is empty) (bit 1) */

#define KBRD_IO 0x60 /* keyboard IO port */
#define KBRD_RESET 0xFE /* reset CPU command */

#define bit(n) (1<<(n)) /* Set bit n to 1 */

/* Check if bit n in flags is set */
#define check_flag(flags, n) ((flags) & bit(n))

void reboot()
{
    uint8_t temp;

    asm volatile ("cli"); /* disable all interrupts */

    /* Clear all keyboard buffers (output and command buffers) */
    do
    {
        temp = inb(KBRD_INTRFC); /* empty user data */
        if (check_flag(temp, KBRD_BIT_KDATA) != 0)
            inb(KBRD_IO); /* empty keyboard data */
    } while (check_flag(temp, KBRD_BIT_UDATA) != 0);

    outb(KBRD_INTRFC, KBRD_RESET); /* pulse CPU reset line */
loop:
    asm volatile ("hlt"); /* if that didn't work, halt the CPU */
    goto loop; /* if a NMI is received, halt again */
}
</source>
== Far jump to the reset vector/Triple fault == 
Probably the easiest way.
<source lang="asm">
JMP 0xFFFF:0
</source>
It'll far jump to the reset vector if you're in real mode, and triple faults if you're in protected mode and removed/haven't setup the gpf handler before.
[[Category:Power management]]
````

<a id="w001845"></a>

## W001845 — Shutdown

Bron-ID: W001845
Namespace: Main (0)
Revisie: 30327 | Datum: 2026-03-17T20:50:27Z
Bron: https://wiki.osdev.org/Shutdown
Revisie-URL: https://wiki.osdev.org/index.php?title=Shutdown&oldid=30327

````text
In order to shutdown the computer, you need to use some sort of power management. Either [[APM]] or [[ACPI]].

== APM ==
This is the basic sequence of APM commands that must be given in order to shut down a computer. For details on exactly how to implement these steps, see the [[APM]] article.

* Perform an installation check.
* Check if the APM version is '''at least''' 1.1
* Disconnect any existing APM interface.
* Connect the Real Mode interface.
* Tell the APM that your driver supports version 1.1
* Enable power management for all devices.
* Set the power state for all devices to "Off" (03h).

If the APM version is 1.0 (or the APM isn't told that your code supports version 1.1, so it will run as 1.0 for legacy purposes) it isn't possible to set the power state for all devices (maybe it's possible to shut each device down individually)

== ACPI ==
[[Topic:16990|ACPI Shutdown code with good explanation in C]]

A summary of ACPI shutdown from the above forum post:
<blockquote>
The ACPI shutdown is technically a really simple thing all that is needed is a <code>outw(PM1a_CNT, SLP_TYPa | SLP_EN );</code> and the computer is powered off. The problem lies in the gathering of these values especialy since the <code>SLP_TYPa</code> is in the <code>\_S5</code> object which is in the <code>DSDT</code> and therefore AML encoded. 
</blockquote>

'''Warning:''' the code above shouldn't be used in production. There are many things the author skips, like calling the _PTS method. A real AML interpreter is required for that, like for example, ACPICA.

On many hardware calling _PTS is required and failing to call it will cause shutdown to fail, or cause hw to freeze in some half-shutdown phase. This phenomenon mostly occurs on laptops, but also occurs on desktops.

== Emulator-specific methods ==

In some cases (such as testing), you may want a poweroff method, but not need it to work on real hardware.

In Bochs, and older versions of QEMU(than 2.0), you can do the following:

<syntaxhighlight lang="c">
outw(0xB004, 0x2000);
</syntaxhighlight>

In newer versions of QEMU, you can do shutdown with:

<syntaxhighlight lang="c">
outw(0x604, 0x2000);
</syntaxhighlight>

In Virtualbox, you can do shutdown with:

<syntaxhighlight lang="c">
outw(0x4004, 0x3400);
</syntaxhighlight>

In Cloud Hypervisor, you can do shutdown with:

<syntaxhighlight lang="c">
outw(0x600, 0x34);
</syntaxhighlight>

=== QEMU with custom BIOS (and Q35) ===

If you use a custom BIOS with QEMU you'll notice that nothing is present on port <code>0x604</code>. This is because shutdown is managed by the ICH9 LPC, which needs to be initialized.

The general steps are:

# Scan the PCI bus for the LPC. Note that MMIO won't be accessible either until it set up by the MCH, so you will have to either:
#* Set a base address for the PCIe configuration registers by writing <code>addr[63:32]</code> and <code>addr[31:0]</code> to <code>0x64</code> and <code>0x60</code> to the MCH configuration space (through I/O ports).
#* Only use I/O ports and ignore MMIO.
# Enable port I/O access for the LPC.
# Set <code>PMBASE</code> to <code>0x600</code>, located at <code>0x40</code> in the PCI configuration space.
# Set <code>ACPI_EN</code> in <code>ACPI_CNTL</code>, located at bit 7 at <code>0x44</code> in the PCI configuration space.

Given that the location of the LPC is fixed, you can also hardcode the addresses, if you wish.

== See Also ==

=== Articles ===
* [[Reboot]]

=== External Links ===

* [https://www.intel.com/content/dam/doc/datasheet/io-controller-hub-9-datasheet.pdf ICH9 datasheet]

[[Category:Power management]]
````

<a id="w001848"></a>

## W001848 — Category:Buses

Bron-ID: W001848
Namespace: Category (14)
Revisie: 3053 | Datum: 2007-02-15T01:25:57Z
Bron: https://wiki.osdev.org/Category:Buses
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ABuses&oldid=3053

````text
[[Category:Hardware]]
````

<a id="w001849"></a>

## W001849 — Category:Common Devices

Bron-ID: W001849
Namespace: Category (14)
Revisie: 3056 | Datum: 2007-02-15T01:28:42Z
Bron: https://wiki.osdev.org/Category:Common_Devices
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ACommon_Devices&oldid=3056

````text
[[Category:Hardware]]
````

<a id="w001851"></a>

## W001851 — Template:NeedsReview

Bron-ID: W001851
Namespace: Template (10)
Revisie: 3091 | Datum: 2007-02-27T14:02:19Z
Bron: https://wiki.osdev.org/Template:NeedsReview
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ANeedsReview&oldid=3091

````text
<center>
{|style="border: 1px solid #cfcfbf; padding: .0em .25em .0em; background-color: #f0f0ff; text-align: center;"
|
<font color="red"><b>This article or section needs to be reviewed according to the discussion [[{{{1|}}}|here]].</b></font>
|}
<includeonly>[[Category:Disputed Pages]]</includeonly>
</center>
````

<a id="w001852"></a>

## W001852 — Introduction

Bron-ID: W001852
Namespace: Main (0)
Revisie: 30898 | Datum: 2026-07-22T01:54:03Z
Bron: https://wiki.osdev.org/Introduction
Revisie-URL: https://wiki.osdev.org/index.php?title=Introduction&oldid=30898

````text
== Welcome ==
Welcome to operating system development; the great frontier. Not all "make it" in this field, many don't even pass the "Hello World" of OS development, but perhaps you will go further and create the next Linux? Or Windows? Or are your goals lower - MenuetOS? Or even CP/M?

Whatever your goals, OSDev'ing is the great pinnacle of programming. But, you're not alone. In fact, this entire website, including the [http://forum.osdev.org forums] and this Wiki, are dedicated to OSDev'ing. This is not only about great programming skills, but is about community and developing friendships. Be those friendships between fellow forum members or IRQs and processes.

What do you need to succeed in OSDev'ing? You should read the [[Getting Started]] article. If you are going to use C/C++ as your language of choice, you are required to build a [[GCC Cross-Compiler]] first. And if you prefer to use other [[Languages]] then it is important to have some similar tools (e.g. compiler) or if there's no such tools it is often only your efforts that can help you. But going along with your preferred language can add some motivation and excitement during your work.

Good luck!

== What is an Operating System? ==

An operating system is a software controlling the operation of a computer system and its resources. Among other things, there's one very important criteria common to all operating systems:<blockquote>Capable of loading and executing user programs providing standardized (hardware-independent) input / output interface for them.</blockquote>Major functions of operating systems may include:

* Managing memory and other system resources.
* Imposing security and access policies.
* Scheduling and multiplexing processes and threads.
* Launching and closing user programs dynamically.
* Providing a basic user interface and application programmer interface.

Not all operating systems provide all of these functions. Single-tasking systems like MS-DOS would not schedule processes, while embedded systems like eCOS may not have a user interface, or may work with a static set of user programs.

An operating system is '''not''':

* The computer hardware.
* A specific application such as a word processor, web browser or game.
* A suite of utilities (like the GNU tools, which are used in many Unix-derived systems).
* A development environment (though some OSes, such as UCSD Pascal or Smalltalk-80, incorporate an interpreter and IDE).
* A graphical user interface (though many modern operating systems incorporate a GUI as part of the OS).

While most operating systems are distributed with such tools, they are not themselves a necessary part of the OS. Some operating system families, such as GNU/Linux, come in several packaged forms called distributions. While they all share the Linux kernel, distributions may include different suites of system utilities, package managers, and software. Nonetheless, because they rely on the same core kernel architecture, they belong to the same operating system family.

== What is a kernel? ==

The kernel of an operating system is something you will never see. It basically enables any other programs to execute. It handles events generated by hardware (called ''interrupts'') and software (called ''system calls''), and manages access to resources.

The hardware event handlers (''interrupt handlers'') will for instance get the number of the key you just pressed, and convert it to the corresponding character stored in a buffer so some program can retrieve it.

The ''system calls'' are initiated by user-level programs, for opening files, starting other programs, etc. Each system call handler will have to check whether the arguments passed are valid, then perform the internal operation to complete the request.

Most user programs do not directly issue system calls (except for asm programs, for instance), but instead use a ''standard library'' which does the ugly job of formatting arguments as required by the kernel and generating the system call. (For example, the C function ''fopen()'' eventually calls a kernel function that actually opens the file.)

The kernel usually defines a few ''abstractions'' like files, processes, sockets, directories, etc. which correspond to an internal state it remembers about last operations, so that a program may issue a session of operation more efficiently.

== What is a shell? ==

A shell is a special program that is usually integrated in an OS distribution, and which offers humans an interface with the computer. The way it appears to users may vary from system to system (command line, file explorer, etc), but the concept is always the same:

* Allow the user to select a program to be started, and optionally give it session-specific arguments.
* Allow trivial operation on the local storage like listing the content of directories, moving and copying files across the system.

In order to complete those actions, the shell may have to issue numerous system calls, like "open file 'x'; open file 'y' and create it if it doesn't exist; read content from X, write into Y, close both files, write 'done' to standard output".

The shell may also be used by programs that want to start other programs but do not want to do this themselves (e.g. completing file patterns like "*.mp3", retrieving the exact path of the program, etc.).

Modern shells can also have various extra features, such as the following:

* '''Auto-Completion''': By pressing the TAB (or any preferred) key, the word the user is typing will be completed to a valid shell command, a file, directory, or something else. Pressing the auto-complete key multiple times cycles through other completion possibilities.
* '''Character Insertion''': The user can move around in what they entered by using the arrow keys. When typing new characters in the middle of a sentence, characters will get "inserted".
* '''Shell History''': By using the up and down arrow keys, the user can scroll through previous input.
* '''Scrolling''': When there are more lines than the console is high, save the output in a buffer and allow the user to scroll up and down in the console.
* '''Scripting''': Some shells have custom scripting languages. Examples of scripting languages are Bash or DOS batch.
* etc.

== What is a GUI about? ==

The graphical user interface (GUI) is the most visible part of any operating system that has one. Its role goes beyond a simple drawing library; it must also be able to:

* Catch user input events (keyboard, mouse, etc.) and dispatch them to the proper object.
* Update the internal information of what is to be displayed where on the screen, determining which parts of the screen need to be redrawn.
* Update the ''visible'' screen contents, redrawing the necessary parts.
* Do so in a way that feels natural, intuitive, and responsive to the user.

=== Desktop Environment, Window Manager, Widget Library ===

When you are launching into a KDE or Windows session, that is a ''desktop environment'', i.e. a graphical shell providing the functional environment for all lower-level functions.

The part of the system responsible for organizing the windows of the various running programs, their resizing/closing gadgets, window borders, scrollbars etc. is the ''Window Manager''.

Finally you have the subsystem that does the drawing of control elements, rendering documents on screen etc.; this is commonly called the ''widget library''. However, there are alternatives to widget libraries, usually in the form of declarative languages (e.g., Mozilla's XUL, Qt's QML).

=== GUI-related forum threads ===

* [[Topic:8783|Will you implement a GUI?]]
* [[Topic:9432|GUI design]]
* [[Topic:9448|Handling input events in GUI]]

== Why develop an OS? ==

There are various reasons why people choose to develop an operating system. Each individual developer may have their own, but some reasons are common among some (if not most) developers:

* '''Having complete control over the machine.''' When developing an application or other userspace program, the developer has to take the code written by others into consideration: the operating system, the libraries, other programs, etc. It is a powerful feeling for the only code to be running on a machine to be your own.

* '''Research.''' Quite a few operating system projects are started as homework or research projects. While starting an operating system as a homework assignment in a pre-tertiary or first-year environment is generally considered a bad idea (due to short deadlines), a long-term project is quite fine. Research projects are usually undertaken to improve on existing operating systems. A common beginner mistake, however, is to underestimate the time needed to write an operating system from scratch.

* '''To replace the currently available operating systems.''' Maybe they don't have a particular feature that the developer wants. Maybe they just generally suck (Linux is bloated, Windows is unstable, etc.). This may be for profit; although any returns are likely to be a long time away.

* '''Because it's fun.''' Low level programming is a fun and exciting task, because you have to do everything. This may seem more difficult (it is, don't worry) but by the same reasons more fun. You know how everything works, how it all fits, and the inner-most workings of your program.

Unfortunately, many operating system projects are undertaken for the wrong reasons. See [[Beginner Mistakes]] for a list.

== See Also ==
=== Articles ===

* [[Required Knowledge]]
* [[Beginner Mistakes]]

[[Category:OS Development]]
````

<a id="w001853"></a>

## W001853 — User:Combuster/Manual Of Style draft

Bron-ID: W001853
Namespace: User (2)
Revisie: 21775 | Datum: 2017-11-27T02:49:53Z
Bron: https://wiki.osdev.org/User:Combuster/Manual_Of_Style_draft
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACombuster%2FManual_Of_Style_draft&oldid=21775

````text
''' This is a draft and hence NOT official. The official version is located [[Manual Of Style|here]] '''

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
Articles should be informative in style. FAQ style is discouraged. Tutorials are allowed, but try to keep instructional content separate from enyclopedic content. 

=== Code Snippets ===
Code snippets can be used on any page where useful. 

Code snippets should be readable and understandable:
* Proper identation: little trouble for a largely increased readability. By convention Allman style (the opening { appears unindented on the next line) is used. 
* Avoid long lines: these break the layout as well as making things very difficult to read. 40 characters should be fine.
* Comments: Use common sense. Short snippets might not need any, longer ones are more likely. Assembly code can in general use more comments than higher level languages.
* Language: The preferred language is either C or Assembly, as most programmers know these. Try however to minimize mixing of languages (inline assembly).

Code blocks can be marked with <nowiki><pre>...</pre></nowiki> tags or a leading space for each line:

 int main(void)
 {
     return(0);
 }

=== Tables ===
use 
<pre><nowiki>
{| {{wikitable}}
|-
! bold text 1
! bold text 2
|-
| normal text 1
| normal text 2
|}
</nowiki></pre>
for tables. Additionally you can use the <nowiki>{{Yes}} and {{No}}</nowiki> for comparison tables

=== Numbers ===
* 1 KiB = 1024 bytes, 1 MiB = 1024 KiB, 1GiB = 1024 Mib, etc. Use of these units is recommended over KB, MB and GB
* KB, MB, and GB should in most cases equal KiB, MiB and GiB, unless there's a compelling reason not to (e.g. the standard size of 1.44 MB floppies, where 1 MB equals 1000*1024 bytes)
* Hexadecimal numbers should be formatted using the 0x prefix rather than the h suffix: 0x1234CDEF
* Avoid octal numbers and numbers with other uncommon bases

== Discussing articles ==
Issues that might need a discussion should be posted on the talk backpages. When you do, use four tildes to sign your message: <nowiki>~~~~</nowiki>.

More important issues should be posted on the wiki forum, as well anything not directly related to any single article, as these are checked by more people.


== References and Links ==

The format for further references is as follows:
==See Also==

===Articles===
*Article1
*Wikipedia:Article2

===Threads===
*Thread1
*Thread2

===External Links===
*<nowiki>http://www.link1.net</nowiki>
*<nowiki>http://www.link2.net</nowiki>


== Sandbox ==
Each member has his own userpage, in which he is free to try things as needed. Please try to minimize Category:xxx links as they cause links to appear to your article.

You can have a look at some other people's user pages for ideas.

== Minor Edits ==
Minor edits are only used to inform of changes that will not need any checking from other people. Things that can pass as minor edits are, among others, fixing of typographic errors, rewording sentences, and to some extent the reordering of blocks, as these edits do not alter the informational content of the article. Be careful with the minor edits when altering code blocks as even small changes can change the net effect of algorithms.
````

<a id="w001862"></a>

## W001862 — Opcode syntax

Bron-ID: W001862
Namespace: Main (0)
Revisie: 31002 | Datum: 2026-10-04T08:29:27Z
Bron: https://wiki.osdev.org/Opcode_syntax
Revisie-URL: https://wiki.osdev.org/index.php?title=Opcode_syntax&oldid=31002

````text
The AT&T syntax (as understood by GAS, the GNU assembler) is the standard syntax on most non-Intel platforms, but remains rare on x86 platforms.
However, AT&T syntax is the default for GCC [[Inline Assembly]], and it is what objdump will provide you with when debugging your kernel.

NASM and FASM use Intel syntax, and Intel syntax is what the [[Bochs]] debugger will provide you when debugging your kernel.

== Important Details ==

There are some substantial differences between the AT&T syntax and the Intel syntax, which a programmer intending to use the GNU tools should be aware of.
Here are a few key things to look for, when moving to AT&T syntax:

*'''Case Sensitivity:''' MOVL is not the same as movl.
*'''Numerical Base:''' expressed as in C: 1 for decimal, 01 for octal, 0x01 for hex. (The Intel postfix-h syntax is not supported.) 
*'''Escapes:''' Special characters are written as C-style escapes (\n, \", \#, \\, ...). 
*'''Comments:''' Either C-style ( /* ... */ ) or shell style (# ...). 
*'''Directive syntax:''' Directives begin with a period (".align 4" to align on a 32-bit boundary, ".word 0x1234" is the equivalent of "DW 1234h"). 
*'''Strings:''' Defined using special directives, .ascii (or .asciz for a zero-terminated string). Example: msg: .ascii "Hello, World!\n" 
*'''Current location address:''' Indicated by a period (".", equivalent to Intel syntax "$"). 
*'''Initializing Memory:''' Done with .fill (roughly equivalent to Intel syntax 'times db'). Example: .fill 0x1fe - (. - START) , 1, 0 (where '1' is the size fill mask in bytes and START is a label marking the entry point of the code) is equal to Intel syntax times 1FEh - ($-$$) db 0. (The .skip and .space directives can be used in a similar manner.) 
*the '''code counter''' can be set multiple times, using the .org directive (as in .org 0x1fe + START, where START is a label marking the entry point of the code. The location-assignment directive, '.=', can be used in the same manner. 
*'''16/32 bit code''' can be generated by stating .code16 or .code32 (equivalent to Intel syntax [BITS 16] and [BITS 32], respectively). 
*'''Target CPU:''' Set with the .arch directive. It is a Good Idea to set it, even if you are sure that the default is 'i386'. 
*'''Label Declarations:''' Always end in a colon. 
*a '''new identifier''' appearing at the beginning of a line, and not ending in a colon, is assumed to be part of an equivalence statement, and must be followed by an equals sign and an assigned value. Example: FOO = 0xF00 
*'''End of Instruction:''' Designated either by a newline or with a semi-colon; the latter is primarily seen in macros, to allow multiple lines of code. 
*'''Line Continuation:''' As in C, with a backslash ('\') as last character in a line. This also is mostly used in macros. 
*'''Registers:''' Always prefixed with a percent sign: %eax, %cs, %esp, etc. 
*'''Source, Destination:''' Move, load, store, and similar operations always have operands in the order 'source, destination', which is very unlike Intel syntax. Thus, "movl %eax, %ebx" moves the value of %eax into %ebx. This is the part that seems to confuse people the most, since it is nearly the opposite of the intel theme: <pre>Opcode    Register/Memory-being-modified, Data, Data</pre>
*'''Operand Size Suffixes:''' Always appended to instructions (with the exception of ljmp, lcall, and lret on the x86): movb for "move byte", movw for "move word", movl for "move long", etc.
*'''Direct-address Operands:''' are not prefixed. Thus, "movl foo, %eax" moves the contents of memory location "foo" into %eax. 
*'''Immediate Operands:''' are prefixed with a dollar sign ($): "pushl $4" pushes 0x00000004 onto the stack. This applies to labels as well: "movl $foo, %eax" moves the value of the label foo (that is, the address of variable foo) into %eax. 
*'''Indexed / Indirect Operands:''' are used in the format segment:displacement (base, index, scale), like so: movl %eax, %ss:8(%ebp, 2, 3) (which is equivalent to Intel syntax mov dword [ss:ebp + 2 * 3 + 8], eax, that is, it moves the value of %eax to offset (%ebp + (2 *3) + 8) in segment %ss). Any of the five operands of an indirect address may be omitted. 
*'''Relative Addressing:''' Used by default in all jump and call instructions. To use absolute addressing, the operand must be prefixed with an asterisk (*). 
*'''Far jumps / calls / returns:''' Use the special opcodes 'ljmp', 'lcall' and 'lret'.

The AT&T syntax format for macros:

<syntaxhighlight lang="asm">
.macro <name> <args>
<operations>
.endm
</syntaxhighlight>

Example:

<syntaxhighlight lang="asm">
.macro write string
   movw string, %si
   call printstr
.endm
</syntaxhighlight>

This would be equivalent to the NASM macro:

<syntaxhighlight lang="asm">
%macro write 1
   mov si, %1
   call printstr
%endmacro
</syntaxhighlight>

Additionally, the cpp and [[M4]] macro preprocessors are often used for macro handling.

== Converting small snippets of code from Intel syntax to AT&T ==

You can use the following script to convert short snippets of code (one liners) from Intel syntax to AT&T syntax:
<syntaxhighlight lang="bash">#!/bin/bash
set -e

# Usage:
#
# ./inteltoatt [16|32|64] "mov eax, eax \n xor ecx, edx"
#

case "$1" in
16|32|64)
	bits="$1"
	shift ;;
*)
	bits="32" ;;
esac
code="$1"

nasm="$(mktemp)"
obj="$(mktemp)"
objdump="$(mktemp)"

case "$bits" in
	16) m="i8086"       ;;
	32) m="i386"        ;;
	64) m="i386:x86-64" ;;
esac

echo -e "BITS $bits\n$code" > "$nasm"

nasm "$nasm" -o "$obj"
objdump -D -b binary -m $m -Maddr${bits},data${bits} "$obj" > "$objdump"

lineno="$(egrep -m 1 -n '<\.data>\:$' "$objdump" | cut -d':' -f1)"
lineno=$((lineno+1))

tail -n +$lineno "$objdump"
</syntaxhighlight>

== See Also ==
=== External Links ===
*[http://www.delorie.com/djgpp/v2faq/faq17_1.html DJGPP AT&T Assembly Tutorial]
*[http://asm.sourceforge.net//howto/Assembly-HOWTO.html Linux Assembly HOWTO]
*[https://savannah.nongnu.org/projects/gas-user/ GAS/AS End User Help Project]
*[https://savannah.nongnu.org/projects/pgubook/ Programming from the Ground Up]

[[Category:Assembly]]
````

<a id="w001863"></a>

## W001863 — Talk:Inline Assembly

Bron-ID: W001863
Namespace: Talk (1)
Revisie: 15583 | Datum: 2014-02-13T10:21:21Z
Bron: https://wiki.osdev.org/Talk:Inline_Assembly
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AInline_Assembly&oldid=15583

````text
The following text came from the OSFAQ on AT&T vs Intel syntax differences -- but it seems to me that it belongs here, instead:

You can switch to intel syntax mode in the inline assembler with a pseudo op, and switch back at the end of the block.
-- MirAbile

Page with the contents on it: http://www.osdev.org/osfaq2/index.php/GasIntelHowtoMirror
-- DasCandy

Under "Input Operands" I dont think the example code:
<source lang="c">
asm("mov %%eax,%%ebx": : "a" (amount));//useless but it gets the idea
</source>
matches the text, namely I think it should be :
<source lang="c">
int dummy;
asm("add %%eax,%%eax": "=a" (dummy)) : "0" (amount));// the "0" indicate that it's using the same register as the first parameter
</source>
--[[User:Dormito|Dormito]] 04:21, 13 February 2014 (CST)

[[User:Bewing|Bewing]] 00:20, 28 February 2007 (CST)
````

<a id="w001864"></a>

## W001864 — Template:Convert

Bron-ID: W001864
Namespace: Template (10)
Revisie: 12732 | Datum: 2012-02-19T20:31:36Z
Bron: https://wiki.osdev.org/Template:Convert
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AConvert&oldid=12732

````text
<includeonly>
<center>
{|style="border: 1px solid #cfcfbf; padding: .0em .25em .0em; background-color: #f0f0ff; text-align: center;"
|
This page has only been roughly converted. It may need its syntax fixed, its links fixed, categorization, etc...
|
|}
</center>
[[Category:Pages For Review]]
</includeonly>

<noinclude>
==Usage==
<pre>
{{Convert}}
</pre>

==Example==
<center>
{|style="border: 1px solid #cfcfbf; padding: .0em .25em .0em; background-color: #f0f0ff; text-align: center;"
|
This page has only been roughly converted. It may need its syntax fixed, its links fixed, categorization, etc...
|
|}
</center>
</noinclude>
````

<a id="w001867"></a>

## W001867 — Mutual Exclusion

Bron-ID: W001867
Namespace: Main (0)
Revisie: 11335 | Datum: 2011-03-28T11:24:45Z
Bron: https://wiki.osdev.org/Mutual_Exclusion
Revisie-URL: https://wiki.osdev.org/index.php?title=Mutual_Exclusion&oldid=11335

````text
Mutual exclusion is a word commonly used in the context of synchronization (e.g. between threads). It is used to ensure that only one instance can access a resource.

== What ==
Mutual exclusion refers to the fact that multiple instances can attempt to access one resource, but only one of them will be granted
access at a time. The word 'instance' here refers to something attempting to gain access to a resource, e.g. a thread.

== Mutex ==
The principle of mutual exclusion is commonly used in the '''mutex''' locking mechanism. A mutex is also referred to as a ''binary semaphore'', because it uses a boolean instead of an integer to test its lock. This lock can then be used in i.e. multi-threaded systems to make a section of the code safe to run on multiple threads.

Suppose multiple threads have to access the same file. You could then use a mutex to determine whether the file is already in use or not. If it is in use, the mutex will be locked and the thread in question will have to wait for it to become available. The other thread will unlock the mutex once it is done with the file. When the first thread notices that the mutex has become unlocked, it immediately locks it and proceeds accessing the file. This way, both threads are certain that they will not intefere with each other or try to access the file at the exact same moment.

Suppose one has to check for the existance of a specific file. 

== See Also ==
=== Articles ===
* [[Synchronization Primitives]]

[[Category:Synchronization]]
````

<a id="w001868"></a>

## W001868 — Higher Half Kernel

Bron-ID: W001868
Namespace: Main (0)
Revisie: 30637 | Datum: 2026-05-27T07:15:31Z
Bron: https://wiki.osdev.org/Higher_Half_Kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Higher_Half_Kernel&oldid=30637

````text
{{Template:Kernel designs}}

Higher half kernel is a traditional and generally recommended kernel memory layout, used by all major operating systems, which places the kernel into the higher half, leaving the lower half to userspace applications. With it, the kernel code is usually mapped into every process in the same place (by copying the top level page table mappings), and the lower half changes when the current process is switched. As such, when writing new kernels, using this layout is strongly advised, unless there is a good reason to not do it ([[Monotasking Systems]], Hypervisors, hardware without MMU, and other specific reasons).

As the "lower and higher halves" name implies, the available address space is split in two (for example, 0x00000000 - 0x7fffffff for userspace and 0x80000000 - 0xffffffff for kernel, 2GB/2GB split) equal halves. However, on 32 bit architectures, the more usual split is 3GB for userspace and 1GB for the kernel (0 - 0xbfffffff, 0xc0000000 - 0xffffffff, accordingly), since it leaves more available address space for userspace. On 64 bit architectures, the split is usually equal, because the address space is very large, and because of the canonical gap.

In addition, some ISAs ([[MIPS Overview|MIPS]], [[AArch64|ARM]], LoongArch, x86 with new protections, and others) partly force the layout. On MIPS and ARM systems, as well as on x86 with LASS enabled, addresses using the high bit (either bit 31 or bit 63, depending on the system word width) are reserved for use in Supervisor mode, and are exception trapped when in User mode. Finally, some architectures, like LoongArch, use split page tables (one for kernel and another one for userspace), which make the kernel's job much easier (and faster) with the higher/lower half split.

Higher half kernels provide many advantages:

* The memory management and system calls become much easier, since the kernel can know if the pointer is coming from userspace by looking at its address.
* Userspace applications do not depend on the kernel memory layout, making the ABI much nicer.
* Creating new processes/address spaces is much easier (on architectures whith use paging, e.g. x86), since the kernel can be fully mapped into the different processes by just copying the top page table mappings.
* On some architectures, you can use additional hardware protections with it (since it is the assumed modern memory layout).
* If your OS is 64-bit, then 32-bit applications will be able to use the full 32-bit address space.
* It's easier to set up [[Virtual 8086 Mode|VM86]] processes since the region below 1 MB is userspace.
* 'Mnemonic' invalid pointers such as ''0xCAFEBABE'', ''0xDEADBEEF'', ''0xDEADC0DE'', etc. can be used.

== Bootloader support ==
To make things easier, many newer [[Bootloaders]] natively support higher half kernels, by directly loading and mapping a kernel to the higher half in virtual memory. For example:

* [[BOOTBOOT]] only supports higher half kernels by design. It has example Hello World kernels written in [[C]], [[Pascal]], [[Rust]] and [[Go]]
* The [[Limine]] protocol also only support higher half kernels by design. See [[Limine Bare Bones]] for a tutorial on how to write a simple 64-bit higher half kernel using Limine.
* [[Ultra protocol]] also only support higher half kernels, allowing to boot 32 and 64 bit x86, and aarch64 kernels directly into the higher half.

== Initialization ==
To set up a higher half kernel, you have to map your kernel to the appropriate virtual address. When using a boot protocol which supports higher half kernels directly, such as [[BOOTBOOT]], [[Limine]], or [[Ultra protocol]] your kernel will already be properly mapped.

How to do this basically depends on '''when''' you'd like your kernel to believe it's in the higher end, and '''when''' you set up paging. Without the boot loader help (rolling bootloader yourself, or using an older/simplier protocol, like multiboot2), you'll need a small trampoline code which runs in lower half, sets up higher half paging and jumps. While this can be done by the kernel for itself, a use of a [[Prekernel]] to do this task (as well as for other initialization duties) might be an easier option.

== See Also ==

=== Articles ===
* [[Higher Half x86 Bare Bones]]

=== Threads ===
* [[Topic:11160|I wrote a simple HigherHalf kernel]]

[[Category:Kernel]]
````

<a id="w001870"></a>

## W001870 — Boot Magic

Bron-ID: W001870
Namespace: Main (0)
Revisie: 6475 | Datum: 2008-10-29T09:37:51Z
Bron: https://wiki.osdev.org/Boot_Magic
Revisie-URL: https://wiki.osdev.org/index.php?title=Boot_Magic&oldid=6475

````text
{{Stub}}

'''BootMagic''' is the [[bootloader]] that comes with the Norton PartitionMagic suite.

== See Also ==
=== External Links ===
* About [http://www.symantec.com/norton/partitionmagic PartitionMagic]

[[Category:Bootloaders]]
````

<a id="w001871"></a>

## W001871 — Boot Sequence

Bron-ID: W001871
Namespace: Main (0)
Revisie: 30544 | Datum: 2026-05-02T10:58:50Z
Bron: https://wiki.osdev.org/Boot_Sequence
Revisie-URL: https://wiki.osdev.org/index.php?title=Boot_Sequence&oldid=30544

````text
== POST ==

When a computer is switched on or reset, it runs through a series of diagnostics called POST - '''P'''ower-'''O'''n '''S'''elf-'''T'''est. This sequence culminates in locating a bootable device, such as a Floppy disk, CD-ROM or a Hard disk in the order that the firmware is configured to.

== Master Boot Record ==

The (legacy) BIOS checks bootable devices for a boot signature, a so called magic number. The boot signature is in a boot sector (sector number 0) and it contains the byte sequence 0x55, 0xAA at byte offsets 510 and 511 respectively. When the BIOS finds such a boot sector, it is loaded into memory at <tt>0x0000:0x7c00</tt> (segment 0, address 0x7c00). (However, some BIOS' load to <tt>0x7c0:0x0000</tt> (segment 0x07c0, offset 0), which resolves to the same physical address, but can be surprising. A good practice is to enforce CS:IP at the very start of your boot sector.)

Execution is then transferred to the freshly loaded boot record. On a floppy disk, all 512 bytes of the boot record (save the last two signature bytes) may contain executable code. On a hard drive, the [[MBR (x86)|Master Boot Record]] (MBR) holds executable code at offset 0x0000 - 0x01bd, followed by table entries for the four [[Partition Table|primary partitions]], using sixteen bytes per entry (0x01be - 0x01fd), and the two-byte signature (0x01fe - 0x01ff).

== Early Environment ==

This early execution environment is highly implementation defined, meaning the implementation of your particular BIOS. ''Never'' make any assumptions on the contents of registers: They might be initialized to 0, but they might contain a spurious value just as well. This includes the FLAGS register and the SP register, you may not have a valid stack either! The only thing that's certain, is that the DL register holds the drive code from where your boot code was loaded.

The CPU is currently in [[Real Mode]]. (Unless you are running on one of those rare BIOS' which believe they're doing you a favor by activating [[Protected Mode]] for you. Which means you not only have to write code for activating protected mode on any ''other'' hardware, but should also add a test condition if it's already activated.)

== Kernel ==

Finally, the bootloader loads the kernel into memory and passes control to it.

== Loading ==

Now we know ''what'' we have to load, let's see ''how'' we get it loaded.

If booting from hard drive, you have only 446 bytes available for your boot record. Looking at the list of things to do before your kernel image can run, you will agree that this is not much:

* determine which partition to boot from (either by looking for the active partition, or by presenting the user with a selection of installed operating systems to chose from);
* determine where your kernel image is located on the boot partition (either by interpreting the file system, or by loading the image from a fixed position);
* load the kernel image into memory (requires basic disk I/O);
* enable protected mode;
* preparing the runtime environment for the kernel (e.g. setting up stack space);

You don't have to do things in this order, but all of this has to be done before you can call <tt>kmain()</tt>.

To make things worse, GCC generates protected mode executables only, so the code for this early environment is one of the [[C#Things C can't do|Things You Cannot Do With C]].

There are several approaches to this problem:

* '''Geek loading''': Squeeze everything from the above list into the boot record. This is next to impossible, and does not leave room for any special-case handling or useful error messages.
* '''One-stage loading''': Write a stub program for making the switch, and link that in front of your kernel image. Boot record loads kernel image (below the 1mb memory mark, because in real mode that's the upper memory limit!), jumps into the stub, stub makes the switch to Protected Mode and runtime preparations, jumps into kernel proper.
* '''Two-stage loading''': Write a ''separate'' stub program which is loaded below the 1mb memory mark, and does everything from the above list.

=== The Traditional Way ===

Traditionally, the MBR relocates itself to <tt>0x0000:0x0600</tt>, determines the active partition from the partition table, loads the first sector of that partition (the "partition boot record") to <tt>0x0000:0x7c00</tt> (hence the previous relocation), and jumps to that address. This is called "chain loading". If you want your self-written boot record to be capable of dual-booting e.g. Windows, it should mimic this behaviour.

=== Easy Way Out ===

Unless you really want to be [[Rolling Your Own Bootloader]] (record / stubs) for the educational value, we recommend using readily available [[:Category:bootloaders|bootloaders]].

The most prominent one is [[GRUB]], a two-stage bootloader that not only provides a boot menu with chainloading capability, but initializes the early environment to a well-defined state (including [[Protected Mode]] and reading various interesting information from the BIOS), can load generic executables as kernel images (instead of requiring flat binaries like most other bootloaders), supports optional kernel modules, various file systems, and if <tt>./configure</tt>'d correctly, [[Diskless Booting]].

=== Some methods ===
There are many possible variants to boot. Below is a list of methods but it is possible that there are even more methods:
* You could take an unused partition and load the stage 2 "raw"
* You could place the stage 2 between MBR and start of the first partition
* You could (as Lilo did) write a kernel file, then use a tool to detect the sectors (or clusters). Then let stage 1 load the sectors from the list.
* DOS and Windows do it this way: Create an empty filesystem (format it) and then place the kernel in the first file, and the shell in the second file in the empty rootdir. So the loader simply loads the first entry in the rootdir and then the second.
* Old Linux was booting from floppy disk. The first sector ("boot") loaded the second stage in "raw" mode = without filesystem (The second stage was "setup", in the sectors directly behind "boot") The second stage did setup the system (video mode, memory map, etc.) and then loaded the real kernel image (usually compressed using gzip, bzip2 etc.).
* Several years ago here was a bootloader (called "nuni") which switched to pmode and loaded a file, all in one bootsector

== See Also ==

=== External Links ===
* [http://duartes.org/gustavo/blog/post/how-computers-boot-up Jun 2008: How Computers Boot Up] by Gustavo Duarte.
* [http://duartes.org/gustavo/blog/post/kernel-boot-process Jun 2008:The Kernel Boot Process] by Gustavo Duarte.
* IBM developerWorks' [https://web.archive.org/web/20190402174801/https://developer.ibm.com/articles/l-linuxboot/ Inside the Linux boot process] a very good, illustrated overview from BIOS to userspace.

[[Category:Booting]]
[[Category:Bootloaders]]
[[Category:X86]]
[[de:Bootprozess auf x86]]
````

<a id="w001872"></a>

## W001872 — Diskless Booting

Bron-ID: W001872
Namespace: Main (0)
Revisie: 29089 | Datum: 2024-06-07T13:38:42Z
Bron: https://wiki.osdev.org/Diskless_Booting
Revisie-URL: https://wiki.osdev.org/index.php?title=Diskless_Booting&oldid=29089

````text
"Diskless Booting" is a synonym for booting across a network. The kernel and its modules are downloaded from a computer on the network. This can be very useful for large projects where Bochs is too slow or one has to use a floppy disk, and is used in some corporate environments to enable centralized OS updates.

In order to boot up your kernel by network, you need a DHCP server, a TFTP server, and a program acting as client on the other computer.

== The GRUB Legacy Way ==

First, you have to create a floppy with [[GRUB Legacy]] configured for net support. You can download a current source release of [[GRUB Legacy]] and <tt>./configure</tt> it with support for your NIC.

Although this is the simplest way, GRUB Legacy doesn't seem to support all network cards.

== The PXELINUX Way ==

Compile syslinux; a <tt>pxelinux.0</tt> file will be created. It is a PXE binary of a simple bootloader-over-tftp, which can be booted by the client computer (not the one with the TFTP server). After setting up DHCP and TFTP accordingly so the file boots, you can use pxelinux to load "memdisk", which comes with syslinux as well.

This file is loaded with a <tt>memdisk initrd=grub.ima</tt> syntax, which will cause pxelinux to load memdisk and grub.ima through TFTP. Memdisk will hook interrupt 0x13, and boot the disk image that way. (However, not all [[GRUB Legacy]] disk images seem to access floppies through bios. If you've got such an image you're stuck.)

You should get a <tt>pxelinux.0</tt> file, which can be loaded by, for example, etherboot. Many modern computers allow booting from NICs so you only need the TFTP and DHCP server up.

At this point, you can make changes to the grub.ima disk image, and put a [[GRUB Legacy]] config file and your kernel's binaries there.

Try <tt>mount /tftpboot/grub.ima /mnt/fpy -o loop</tt> under linux, for example.

== The gPXE + GRUB Way ==

[[GRUB]] since version 2 can piggy-back on [http://www.etherboot.org/wiki/start gPXE]'s network support, unlike GRUB Legacy that does not support newer network cards. The gPXE project is a currently-maintained, open source, free network bootloader. It is easy to get gPXE ISO, disk, or USB disk images from their website, but there is a workaround you need to apply in order to get GRUB to successfully load.

gPXE supports multiboot, but if it detects a multiboot image then it will not provide PXE services.  Unfortunately, it detects your generated GRUB image as multiboot, and the only way I found to get around this was to recompile gPXE without multiboot support.  Recompiling gPXE is easy: unpack it, <tt>cd src; make</tt>.  Before you do that, you will want to edit <tt>src/config/defaults/pcbios.h</tt> and comment out the line that defines <tt>IMAGE_MULTIBOOT</tt>.  After compiling you should be left with <tt>bin/gpxe.{dsk,iso,usb}</tt> which you can write to disk or CD.

To create a GRUB PXE bootable image, you can follow the advice in the [http://www.gnu.org/software/grub/manual/grub.html#Network GRUB manual's Network chapter]. Several of the options to <tt>grub-mkimage</tt> did not exist in past versions and you need to upgrade if they aren't supported. If you compile from source, you can get all the <tt>*.{lst,mod,img}</tt> files you need in the source directory.  In the case you compile your own GRUB you do not need to install it, just do commands like this:

<source lang="text">
./grub-mkimage -d . --format=i386-pc --output=core.img --prefix="(pxe)/boot/grub" pxe pxecmd
cat pxeboot.img core.img > grub2pxe
</source>

The final thing you need to do is setup a DHCP/BOOTP/TFTP server.  I used <tt>dnsmasq</tt> which came preinstalled on my workstations, and seems to be widely available in distributions.  It can easily be configured on the command line or in <tt>/etc/dnsmasq.conf</tt> which uses the same syntax as the long-form command-line options but without the leading dashes.  You will need the following options:

<source lang="text">
interface=...                # be careful what interface the dhcp server runs on!
bind-interfaces              # *really* only bind that interface
dhcp-range=a.b.c.d,e.f.g.h   # whatever your private network uses
dhcp-boot=boot/grub/grub2pxe # tells machine to boot grub
dhcp-no-override             # some kind of workaround that gpxe needs
enable-tftp
tftp-root=/tftp              # or wherever
</source>

and there are other options to explore as well.  Now make sure that you take <tt>grub2pxe,*.lst,*.mod</tt> from the GRUB source and put them in <tt>/tftp/boot/grub</tt> or equivalent.  Also put your <tt>grub.cfg</tt> file there.  The format of that is fairly simple.  Here's the essence of what I use:

<source lang="text">
set timeout=0
set default=0
menuentry "MY OS" {
  set root=(pxe)
  multiboot /kernel
  module    /shell
  module    /test
}
</source>

Make sure your kernel and modules appear in the tftp root, and you should be set to boot using your gPXE media, over a private network connection hooked up between workstations.

==The Direct Way==

{{Main|PXE}}

Both of the options above involve using someone else's code to do the dirty work, which may be undesirable in some situations - licensing conflicts, technical problems (e.g. for "memdisk" the interrupt 0x13 hook won't work in protected mode) and possibly personal pride. Fortunately, writing your own PXE boot code isn't as difficult as it sounds.

At first glance the PXE specification can look rather daunting, however most of it relates to BIOS and network cards and can be safely ignored. The important part is in chapter 3, the PXE API.

The PXE API is capable of doing raw UDP connections among several other things, however, for a start, you might just want to read files using TFTP.

==See Also==
===External Links===
*[http://osdev.berlios.de/netboot.html Tutorial on network booting]
*[http://home.dei.polimi.it/fornacia/progetto_minazzi/index.html#HOW%20TO%20INSTALL%20A%20TFTP%20SERVER Setting up a TFTP server]
*[http://tftpd32.jounin.net/ TFTP32 server]
*[http://download.intel.com/design/archives/wfm/downloads/pxespec.pdf PXE Specification] 

[[Category:Booting]]
[[Category:Bootloaders]]
````

<a id="w001873"></a>

## W001873 — Porting GCC to your OS

Bron-ID: W001873
Namespace: Main (0)
Revisie: 28221 | Datum: 2023-07-10T20:22:23Z
Bron: https://wiki.osdev.org/Porting_GCC_to_your_OS
Revisie-URL: https://wiki.osdev.org/index.php?title=Porting_GCC_to_your_OS&oldid=28221

````text
{{Rating|4}}
{{In Progress}}

* Read [[GCC Cross-Compiler]].
* Make a [[OS Specific Toolchain]].
* Read [[Hosted GCC Cross-Compiler]].
* Have a sufficient [[C Library]]. GCC is fairly portable and needs the C standard library, and some extensions from POSIX. It needs fork and exec, for instance, to run the assembler and linker. You will need a C++ standard library (such as libstdc++) as GCC is now written in [[C++]].
* Cross-compile libgmp, libmpc, libmpfr and other dependencies with instructions in [[Cross-Porting Software]].
* Likewise cross-compile [[Binutils]] and [[GCC]], but pass the special configure options <tt>--with-build-sysroot=/your/sysroot</tt> and <tt>--with-sysroot=/</tt>. The option <tt>--with-build-sysroot</tt> option uses that [[sysroot]] for the duration of the build, but the final program does not remember it. <tt>--with-sysroot=/</tt> should not be needed or should be set to the empty string, but some binutils/gcc versions are buggy and require it, which has the disadvantage the final toolchain might use <tt>//foo</tt> instead of <tt>/foo</tt>. You should ''only'' pass these options to the Binutils and GCC builds, some other packages also have a <tt>--with-sysroot</tt> option, but it instead means a libtool feature you should ''not'' use.
* Become self-hosting and complete [[Bare_Bones#Bare_Bones_II|Bare Bones II]]. Congratulations.

== See Also ==
* [[Binutils]]
* [[Cross-Porting Software]]
* [[GCC]]
* [[GCC Cross-Compiler]]
* [[Hosted GCC Cross-Compiler]]
* [[OS Specific Toolchain]]
* [[Sysroot]]

[[Category:Porting]]
[[Category:Compilers]]
````

<a id="w001874"></a>

## W001874 — X86 Interrupts

Bron-ID: W001874
Namespace: Main (0)
Revisie: 30830 | Datum: 2026-06-29T19:40:14Z
Bron: https://wiki.osdev.org/X86_Interrupts
Revisie-URL: https://wiki.osdev.org/index.php?title=X86_Interrupts&oldid=30830

````text
'''Interrupts''' are signals from a device, such as a keyboard or a hard drive, to the CPU, telling it to immediately stop whatever it is currently doing and do something else. For example, a keyboard controller can send an interrupt when a character key was pressed. Then the OS can display the character on screen immediately, even if the CPU was doing something completely unrelated before, and return to what it was doing afterwards. When a specific interrupt arises, the CPU looks up an entry for that specific interrupt from a table provided by the OS. In [[Protected Mode|x86 protected mode]] the table is called the [[IDT|Interrupt Descriptor Table (IDT)]] and can have up to 256 entries, but the name of this table and the maximum number of entries it can have can differ based on the CPU architecture. After the CPU finds the entry for the interrupt, it jumps to the code the entry points to. This code that is run in response to the interrupt is known as a [[Interrupt Service Routines|interrupt service routine (ISR) or an interrupt handler]].

== Types of Interrupts ==

There are generally three classes of interrupts on most platforms:
* '''[[Exceptions]]''': These are generated internally by the CPU and used to alert the running kernel of an event or situation which requires its attention. On x86 CPUs, these include exception conditions such as Double Fault, Page Fault, General Protection Fault, etc.

* '''Interrupt Request (IRQ) or Hardware Interrupt''': This type of interrupt is generated externally by the chipset, and it is signaled by latching onto the #INTR pin or equivalent signal of the CPU in question. There are two types of IRQs in common use today.
** ''IRQ Lines, or Pin-based IRQs'': These are typically statically routed on the chipset. Wires or lines run from the devices on the chipset to an [[IRQ controller]] which serializes the interrupt requests sent by devices, sending them to the CPU one by one to prevent races. In many cases, an IRQ Controller will send multiple IRQs to the CPU at once, based on the priority of the device. An example of a very well known IRQ Controller is the [[8259 PIC|Intel 8259]] controller chain, which is present on all IBM-PC compatible chipsets, chaining two controllers together, each providing 8 input pins for a total of 16 usable IRQ signaling pins on the legacy IBM-PC.
** ''[[Message Signaled Interrupts]]'': These are signaled by writing a value to a memory location reserved for information about the interrupting device, the interrupt itself, and the vectoring information. The device is assigned a location to which it writes either by firmware or by the kernel software. Then, an IRQ is generated by the device using an arbitration protocol specific to the device's bus. An example of a bus which provides message based interrupt functionality is the PCI Bus.

* '''Software Interrupt''': This is an interrupt signaled by software running on a CPU to indicate that it needs the kernel's attention. These types of interrupts are generally used for [[System Calls]]. On x86 CPUs, the instruction which is used to initiate a software interrupt is the "INT" instruction. Since the x86 CPU can use any of the 256 available interrupt vectors for software interrupts, kernels generally choose one. For example, many contemporary Unixes use vector 0x80 on the x86 based platforms.

As a rule, where a CPU gives the developer the freedom to choose which vectors to use for what (as on x86), one should refrain from having interrupts of different types coming in on the same vector. Common practice is to leave the first 32 vectors for exceptions, as mandated by Intel. However you partition of the rest of the vectors is up to you.

==From the keyboard's perspective==
Basically, when a key is pressed, the keyboard controller tells a device called the [[PIC|Programmable Interrupt Controller]], or PIC, to cause an interrupt. Because of the wiring of keyboard and PIC, IRQ #1 is the keyboard interrupt, so when a key is pressed, IRQ 1 is sent to the PIC. The role of the PIC will be to decide whether the CPU should be immediately notified of that IRQ or not and to translate the IRQ number into an ''interrupt vector'' (i.e. a number between 0 and 255) for the CPU's table.

The OS is supposed to handle the interrupt by talking to the keyboard, via <tt>in</tt> and <tt>out</tt> instructions (or <tt>inportb</tt>/<tt>outportb</tt>, <tt>inportw</tt>/<tt>outportw</tt>, and <tt>inportd</tt>/<tt>outportd</tt> in C, see [[Inline Assembly/Examples]]), asking what key was pressed, doing something about it (such as displaying the key on the screen, and notifying the current application that a key has been pressed), and returning to whatever code was executing when the interrupt came in. Indeed, failure to read the key from the buffer will prevent any subsequent IRQs from the keyboard.

==From the PIC's perspective==
There are actually two PICs on most systems, and each has 8 different inputs, plus one output signal that's used to tell the CPU that an IRQ occurred. The slave PIC's output signal is connected to the master PIC's third input (input #2); so when the slave PIC wants to tell the CPU an interrupt occurred it actually tells the master PIC, and the master PIC tells the CPU. This is called "cascade". The master PIC's third input is configured for this and not configured as a normal IRQ, which means that IRQ 2 can't happen.

A device sends a PIC chip an interrupt, and the PIC tells the CPU an interrupt occurred (either directly or indirectly). When the CPU acknowledges the "interrupt occurred" signal, the PIC chip sends the interrupt number (between 00h and FFh, or 0 and 255 decimal) to the CPU. When the system first starts up, IRQs 0 to 7 are set to interrupts 08h to 0Fh, and IRQs 8 to 15 are set to interrupts 70h to 77h. Therefore, for IRQ 6 the PIC would tell the CPU to service INT 0Eh, which presumably has code for interacting with whatever device is connected to the master PIC chip's "input #6". Of course, there can be trouble when two or more devices share an IRQ; if you wonder how this works, check out [[Where Can I Find Info On PNP|Plug and Play]]. Note that interrupts are handled by priority level: 0, 1, 2, 8, 9, 10, 11, 12, 13, 14, 15, 3, 4, 5, 6, 7. So, if IRQ 8 and IRQ 3 come in simultaneously, IRQ 8 is sent to the CPU. When the CPU finishes handling the interrupt, it tells the PIC that it's OK to resume sending interrupts:

<syntaxhighlight lang="asm">
mov al,20h
out 20h,al
</syntaxhighlight>

or if the interrupt came from the slave PIC:
<syntaxhighlight lang="asm">
mov al, 20h
out A0h, al
out 20h, al
</syntaxhighlight>

and the PIC sends the interrupt assigned to IRQ 3, which the CPU handles (using the IDT to look up the handler for that interrupt).

Alert readers will notice that the CPU has reserved interrupts 0-31, yet IRQs 0-7 are set to interrupts 08-0Fh. Now the reserved interrupts are called when, for example, a dreadful error has occurred that the OS must handle. Now when the computer first starts up, most errors of this type won't occur. However, when you enter protected mode, these errors may occur at any time, and the OS needs to be able to handle them. How's the OS going to tell the difference between INT 9, Exception: Coprocessor segment overrun, and INT 9: IRQ 1? Well, it can ask the device whether there is really an interrupt for that device. But this is slow, and hackish, and not all devices are able to do this type of thing. The best way to do it is to tell the PIC to map the IRQs to ''different'' interrupts, such as INT 78h-7Fh. For information on this, see the [[PIC]] FAQ. Note that IRQs can only be mapped to INTs that are multiples of 08h: 00h-07h, 08h-0Fh, 10h-17h, 17h-1Fh. And you probably want to use 20h-27h, or greater, since 00h-1Fh are reserved by the CPU. Also, each PIC has to be programmed separately. You can tell the Master PIC to map IRQs 0-7 to INTs 20h-27h, but IRQs 8-F will still be INTs 70h-77h, unless you tell the Slave PIC to put them elsewhere as well.

See [[PIC#Programming the PIC chips|programming the PIC chips]] for detailed information.

==From the CPU's perspective==

Every time the CPU is done with one machine instruction, it will check if the PIC's pin has notified an interrupt. If that's the case, it stores some state information on the stack (so that it can return to whatever it is doing currently, when the INT is done being serviced by the OS) and jumps to a location pointed to by the IDT. The OS takes over from there.
The current program can, however, prevent the CPU from being disturbed by interrupts by means of the ''interrupt flag'' (IF in status register). As long as this flag is cleared, the CPU ignores the PIC's requests and continues running the current program. Assembly instructions <tt>cli</tt> and <tt>sti</tt> can control that flag.

==From the OS's perspective==
When an interrupt comes in, the [[IDT]] (which is setup by the OS in advance) is used to jump to code portion of the OS, which handles the interrupt (and therefore called the "interrupt handler" or "[[Interrupt Service Routines]]"). Usually the code interacts with the device, then returns to whatever it was doing previously with an <tt>iret</tt> instruction (which tells the CPU to load the state information it saved, from the stack). Before the <tt>iret</tt>, this code is executed, to tell the PIC that it's OK to send any new or pending interrupts, because the current one is done. The PIC doesn't send any more interrupts until the cpu acknowledges the interrupt:

<syntaxhighlight lang="asm">
mov al,20h
out 20h,al
</syntaxhighlight>

In the case of the [[PS/2_Keyboard|keyboard input]], the interrupt handler asks the keyboard which key was pressed, does something with the information, then acknowledges and return:

<syntaxhighlight lang="asm">
push eax    ;; make sure you don't damage current state
in al,60h   ;; read information from the keyboard

mov al,20h
out 20h,al  ;; acknowledge the interrupt to the PIC
pop eax     ;; restore state
iret        ;; return to code executed before.
</syntaxhighlight>

Whatever the CPU was previously doing is then resumed (unless another INT was received by the PIC while servicing this one, in which case the PIC tells the CPU about it and a new interrupt handler is executed, once the CPU saves state information on the stack again).

== So how do I program this stuff?==

Step by step, now that you've grabbed the whole thing and know what's to be done:
* Make space for the interrupt descriptor table
* Tell the CPU where that space is (see [[GDT Tutorial]]: <tt>lidt</tt> works the very same way as <tt>lgdt</tt>)
* Tell the PIC that you no longer want to use the BIOS defaults (see [[PIC#Programming the PIC chips|Programming the PIC chips]])
* Write a couple of ISR handlers (see [[Interrupt Service Routines]]) for both IRQs and exceptions
* Put the addresses of the ISR handlers in the appropriate descriptors (in [[Interrupt Descriptor Table]])
* Enable all supported interrupts in the IRQ mask (of the PIC)

== General IBM-PC Compatible Interrupt Information ==
=== Standard [[ISA]] IRQs ===
{| class="wikitable"
|-
! IRQ !! Description
|-
| 0 || Programmable Interrupt Timer Interrupt
|-
| 1 || Keyboard Interrupt
|-
| 2 || Cascade (used internally by the two PICs. never raised)
|-
| 3 || COM2 (if enabled)
|-
| 4 || COM1 (if enabled)
|-
| 5 || LPT2 (if enabled)
|-
| 6 || Floppy Disk
|-
| 7 || LPT1 / Unreliable [[8259_PIC#Spurious_IRQs|"spurious" interrupt]] (usually)
|-
| 8 || CMOS real-time clock (if enabled)
|-
| 9 || Free for peripherals / legacy SCSI / NIC
|-
| 10 || Free for peripherals / SCSI / NIC
|-
| 11 || Free for peripherals / SCSI / NIC
|-
| 12 || PS2 Mouse
|-
| 13 || FPU / Coprocessor / Inter-processor
|-
| 14 || Primary ATA Hard Disk
|-
| 15 || Secondary ATA Hard Disk
|}

=== Default PC Interrupt Vector Assignment ===
{| class="wikitable"
|-
! Int !! Description
|-
| 0-31 || [[Protected Mode]] Exceptions (Reserved by Intel)
|-
| 8-15 || Default mapping of IRQ0-7 by the BIOS at bootstrap
|-
| 70h-78h || Default mapping of IRQ8-15 by the BIOS at bootstrap
|}

=== Ports ===
{| class="wikitable"
|-
! Port !! Description
|-
| 20h & 21h || control/mask ports of the master PIC
|-
| A0h & A1h || control/mask ports of the slave PIC
|-
| 60h || data port from the keyboard controller
|-
| 64h || command port for keyboard controller - use to enable/disable keyboard interrupts, etc.
|-
|}

== See Also ==
* [[S390 Interrupts]]

=== Articles ===
* [[Ralf Brown's Interrupt List]]
* [[Interrupts tutorial]]

=== Threads ===

=== External Links ===
* [[wikipedia:Interrupts|Interrupts Wikipedia Article]]

[[Category:Interrupts]]
[[Category:x86]]
````

<a id="w001877"></a>

## W001877 — Descriptors

Bron-ID: W001877
Namespace: Main (0)
Revisie: 16219 | Datum: 2014-04-17T18:42:21Z
Bron: https://wiki.osdev.org/Descriptors
Revisie-URL: https://wiki.osdev.org/index.php?title=Descriptors&oldid=16219

````text
Descriptors are one of the key structures of Protected Mode. They are used to hold information about various system objects and are used as entries in the three system tables - [[GDT]], LDT and [[IDT]].

== Common properties ==

All descriptors are 8 bytes in size. Different types may have different format.

One thing they all have in common is the "type_attr" field (byte at offset 5) that contains descriptor type and some common attributes. Thus, if you have a descriptor of yet unknown type, you should look at the "type_attr" byte to find out the type. (Maybe you want to analyze and display the contents of some table)

<source lang="c">
// a "general" structure of a descriptor whose type we don't know,
// only for the sake of discussion - you will want to use a more specific structure
// according to the type you're dealing with, or define a union for all types
struct Descr{
    uint8_t bytes[8];

    // --- accessor methods
    uint8_t type_attr(){ return bytes[5]; };
};
</source>

"type_attr" has the following bit fields:
* bit 7: "P" (Present) flag
:Indicates whether the descriptor is being used. Accessing an object through a descriptor which has P=0 will cause a <del>General Protection Fault</del> Segment Not Present Fault (Int 0Bh) - or, if it's trying to be used as a Stack Segment, a Stack Fault (Int 0Ch).
* bits 5..6: DPL - Descriptor Privilege Level (2 bits, 0..3)
:Used to control access to the object described by the descriptor
* bit 4: "S" - whether this is a data/code segment (S=1) or a system segment (some other object) (S=0).
:One can think of it as "(data/code) Segment" flag. This, combined with the "type" code, can be used to determine the descriptor type and structure.
* bits 0..3: type (4 bits, 0..0xF)

Here are the formulas:

{| {{wikitable}}
|-
! type
| (type_attr & 0xF)
|- 
! S
| ((type_attr>>4) & 1)
|-
! DPL
| ((type_attr>>5) & 3)
|-
! P
| ((type_attr>>7) & 1)
|}

== Code/Data Segment Descriptors ==

These are descriptors in the GDT that have S=1. Bit 3 of "type" indicates whether it's (0) Data or (1) Code. The interpretation of the other bits are given further down this page. 

The structure for a code/data descriptor is as follows:

<source lang="c">
struct SegDescr{
    uint16_t limit_1;   // limit, bits 0..15
    uint16_t base_1;    // base, bits 0..15
    uint8_t base_2;     // base, bits 16..23
    uint8_t type_attr;  // type_attr
    uint8_t lim_attr;
      //^ bits 0..3: limit, bits 16..19
      //^ bits 4..7: additional data/code attributes
    uint8_t base_3;     // base, bits 24..31
};
</source>

When concatenating all chunks, base has 32 bits and limit has 20 bits. To allow for a wide range of sizes, the G bit influences how the 20 bits of the limit field are converted to a 32-bit limit
* when using G=0 (granularity: bytes), the amount of accessible bytes can be from 1b to 1Mb (bytes 0x0 to limit. byte 0 will always be writable)
* when using G=1 (granularity: pages), the amount of accessible bytes can be from 4Kb to 4Gb (= 0x1000 * limit + 0xfff)

Additional attributes from lim_attr (bits 4..7):

{| {{wikitable}}
|-
! bit 7
! G
! Granularity
| 0 (limit is in bytes)<br>1 (limit is in pages of 4096 bytes)
|-
! bit 6
! D/B 
! Default operand size/Big
| 0 for 16-bit segments<br>1 for 32-bit segments
|-
! bit 5
! L
! 64-bit code segment
| 0 normally<br>1 if this is a 64-bit code segment in IA-32e mode
|-
! bit 4
! AVL
! Available
| For use by system software<br>(your OS can use this as you choose)
|}

The following snippet calculates the base and 20-bit limit from a descriptor:

<source lang="c">
// computing base and limit values - putting the bits together

uint32_t SegDescr::base(){
 return
   base_1 |
   (base_2<<16) |
   (base_3<<24);
};

uint32_t SegDescr::limit(){
 return
  limit_1 |
  ((lim_attr&0xf)<<16);
};
</source>

=== Type bits for Data segments ===

{| {{wikitable}}
|-
! bit 3
! Data/Code
| 0 (data)
|-
! bit 2
! Expand-down
| 0 (normal) <br> 1 ([[Expand_Down|expand-down]])
|-
! bit 1
! Writable
| 0 (read-only) <br> 1 (read-write)
|-
! bit 0
! Accessed
| 0 (hasn't been accessed) <br> 1 (has been accessed)
|}

=== Type bits for Code segments ===

{| {{wikitable}}
|-
! bit 3
! Data/Code
| 1 (code)
|-
! bit 2
! Conforming
| 0 (non-conforming) <br> 1 (conforming)
|-
! bit 1
! Readable
| 0 (execute-only) <br> 1 (executable and readable)
|-
! bit 0
! Accessed
| 0 (hasn't been accessed) <br> 1 (has been accessed)
|}

== See Also ==
* [[Protected Mode]]
* [[Babystep6]]

[[Category:X86 CPU]]
````

<a id="w001878"></a>

## W001878 — Rolling Your Own Bootloader

Bron-ID: W001878
Namespace: Main (0)
Revisie: 30336 | Datum: 2026-03-17T20:59:00Z
Bron: https://wiki.osdev.org/Rolling_Your_Own_Bootloader
Revisie-URL: https://wiki.osdev.org/index.php?title=Rolling_Your_Own_Bootloader&oldid=30336

````text
{{rating|2}}
Some people prefer to use their own software for everything, or wish to try their hand at coding a bootloader. This page attempts to describe what steps to take when you write your own bootloader. Before you start writing one, it is best that you know the background [[Bootloader|theory]].

== What and Why ==
=== Disclaimer ===

Okay. You are here because you don't want to use a mainstream bootloader. You may also want to code your own bootloader as a learning experience to better understand how they function. We also have pages on many [[Bootloaders]] developed by this community, there's the [[Bare bones]] but still people complain we don't have a page explaining how the bootloader can be coded.

I won't try to give you full code that works because if that was what you were looking for, you'd be using one of the [[:Category:Bootloaders|premade bootloaders]] instead. This page plans to tell you what is needed and what could be wished in a bootloader, and optionally points at parts of the FAQ that can help you achieving the goals.

Whether or not you'll use your own bootloader or reuse an existing tool is completely up to you. If you get the feeling you don't understand a thing, make sure you read our page about the [[Boot Sequence]] first.<br />

A good reason to have a custom bootloader would be a custom filesystem, though you could add support for your filesystem to [[GRUB]] or perhaps some other bootloader. (GRUB is designed to accommodate such additions, other bootloaders may not be.)

=== What you need to do ===

The bootloader ultimately has to bring the kernel (and all the kernel needs to bootstrap) in memory, switch to an environment that the kernel will like and then transfer control to the kernel.

As the scope of this article is protected mode C kernels, I'll assume that "an environment the kernel will like" means [[Protected Mode]], with kernel and additional components being stored at their 'favorite', compile-time known locations, with a wide-enough stack ready and BSS section cleared.

=== What you could wish to add ===

Since the bootloader runs in [[Real Mode]], it has easier access to BIOS resources and functions. Therefore it's a good place to perform [[How Do I Determine The Amount Of RAM|memory map detection]], [[Getting VBE Mode Info|detecting available video modes]], loading additional files etc. The bootloader will collect this information and present it in a way the kernel will be able to understand

== Loading... Please wait... ==

=== Where will you load your kernel? ===

You will have to decide where in memory you are going to load your kernel. Your kernel generally depends on it.

In Real Mode, the easiest is to stay below the 1MB barrier, which means you practically have 512KB of memory to load things. You may wish the kernel to be loaded at a well-known position, say 0x10000 physical (es=0x1000, bx=0 when calling INT13h).

If your kernel is bigger (or is expected to grow bigger) than this, you'll probably prefer to have the kernel above the 1MB barrier, which means you need to activate [[A20 Line|A20 gate]] and switch to [[Unreal Mode]] to load the kernel (with A20 alone, you cannot have more than 64K above 1MB).

Note that BIOS will still be unable to write to memory above 1MB, so you need to read stuff in a buffer below 1MB and then perform a rep movsd to place the data where they ultimately should go.

=== How will you find your kernel? ===

The bits of your kernel are somewhere on some disk (presumably the [[Bootable Disk|booting disk]], but this is not mandatory). Question is: where on the disk? Is it a regular file on a [[FAT|FAT-formatted floppy]]? is it a collection of consecutive sectors in the "reserved area" of the FAT12 floppy (in which case you may need a dedicated tool to format the disk and install the kernel on it)? Or is the floppy simply left unformatted and kernel pasted directly with a [[Disk Images|disk image tool]].

All the above options are possible. Maybe the one I'd choose myself would be to reserve enough space on a FAT12 floppy to store the ''list of sectors'' used by the kernel file. The "advantage" of being fully-FAT12 is that you don't need to re-write the bootsector every time you rewrite the kernel.

=== What else could you need to load? ===

That mainly depends on what's in your kernel. Linux, for instance, requires an additional 'initrd' file that will contain the 'initialization process' (as user level). If your kernel is modular and if [[File Systems]] are understood by some modules, you need to load the modules along with the kernel. Same goes for 'microkernel services' like disk/files/memory services, etc.

=== What if I get beyond the 512 bytes of the boot sector? ===

Make sure the first 512 bytes are able to load the rest of your loader and you're safe. Some do this with a separate "second stage" loader, others by really inserting a '512-bytes' break in their ASM code, making sure the rest of the loader is put after the bootsector (that is, starting at 0x7e00).

=== What if I wish to offer the user the option to boot several OSes? ===

The easiest way to boot another OS is a mechanism called ''chainloading''. Windows stores something akin to a second-stage bootloader in the boot sector of the ''partition'' it was installed in. When installing Linux, writing e.g. LILO or GRUB to the ''partition'' boot sector instead of the MBR is also an option. Now, the thing your MBR bootsector can do is to ''relocate'' itself (copying from 0x0000:0x7c00 to, traditionally, 0x0060:0x0000), parse the partition table, display some kind of menu and let the user chose which partition to boot from. Then, your (relocated) MBR bootsector would load that ''partition'' boot sector to 0x0000:0x7c00, and jump there. The partition boot sector would be none the wiser that there already was a bootsector loaded ''before'', and could actually load yet ''another'' bootsector - which is why it's called ''chainloading''. It doesn't really matter where you decide to relocate the boot sector as long as you don't overwrite the [[IVT]] (if IF in EFLAGS is set), the [[Memory Map (x86)#BIOS Data Area (BDA)|BDA]] or the [[Memory Map (x86)#Extended BIOS Data Area (EBDA)|EBDA]].

You see that with displaying a menu in some intelligible way and accepting keystrokes, such a multi-option bootloader can get quite complex rather quickly. We didn't even touch the subject of booting from extended partitions, which would require sequentially reading and parsing multiple extended partition tables before printing the menu.

Taken to the extreme, boot managers like that can become as complex as a simple OS (much like GRUB is, which offers reading from various filesystems, booting [[Multiboot]] kernels, chainloading, loading initrd ramdisks etc. etc. - such internals will not be addressed here.

=== How do I actually load bytes ===

[[BIOS]] interrupt 13h. Get info about it at [[Ralf Brown's Interrupt List]], make sure you know (the now outdated) floppies may fail one or two times, that you cannot read more than a track at once, you have to use CHS addressing and you're done.

To read from the hard drive (which is the preferred way these days, also used by CDROMs and USB-sticks), you probably want int 13h, ah=0x42, drive number 0x80 that uses simple [[LBA]] addressing. Details in the interrupt list.

If you need guidance, feel free to check [http://clicker.cvs.sourceforge.net/clicker/c32-lxsdk/kernel/src/sosflppy/lowlevel.asm?view=log lowlevel.asm]


Note also that most [[File Systems]] involve some conversion between allocation units (blocks/clusters) and physical "Cylinder:Head:Sector" values. Those conversions are simple once you know the ''sectors-per-track'' and ''heads'' counts. Check out [http://www.nondot.org/sabre/os/articles OSRC] for additional info. This is only relevant for outdated floppies; everything else, like hard drives, CDROMs, USB-sticks all use the simple LBA addressing scheme.

<pre>
> Does anyone have a formula for converting DOS Sectors to
> Physical Sectors (Head, Cylinder, Sector) such as used in
> INT 13h?

DOS_sector_num = BIOS_sector_num - 1 + Head_num*Sectors_per_track
		+ Track_num*Sectors_per_track*Total_heads

BIOS_sector_num = 1 + (DOS_sector_num MOD Sectors_per_track)
BIOS_Head_num   = (DOS_sector_num DIV Sectors_per_track) MOD Total_heads
BIOS_Track_num  = (DOS_sector_num DIV Sectors_per_track) DIV Total_heads
</pre>

If you're loading above 1MB, you should proceed in 2 steps: first using BIOS to load in the "conventional" area, and then performing a <tt>rep movsd</tt> to place the data where they ultimately should go.

== Loaded. Gathering Information ==

The next step consist of collecting as much information as you can/need: [[How Do I Determine The Amount Of RAM|amount of installed RAM]], available [[Getting VBE Mode Info|video modes]] and things alike are easier to do in real mode, so better do them while in [[Real Mode]] than trying to come back to real mode for a trip later. Of course the exact requirements depend on your kernel.

A very simple solution here is to organize your information as a flat table (ala [[BIOS|BIOS data area]]). An alternative could be to add those information as a structured flow: you keep an index at a well-known address (or at some address you'll pass to the kernel when loaded) and that index gives for each "key" the address of the corresponding data structure. E.g.

<pre>
  organization           lookup code (eax == signature)
  +------+------+          mov esi, well_known_index_address
  | RAM. | 1234 |        .loop:
  | VBE. | 5678 |          cmp [esi],'END.'
  | MODS | 9ABC |          je .notfound
  | DISK | DEF0 |          add esi,8
  | END. | ---- |          cmp [esi-4],eax
  +------+------+          jne .loop
                           mov eax,[esi]
                           ret
</pre>

== Ready. Entering [[Protected Mode]] ... ==

To enter protected mode you should first disable interrupts and set global descriptor table. After it set PE bit of CR0:

<syntaxhighlight lang="asm">
mov eax,cr0
or eax,1
mov cr0,eax
</syntaxhighlight>

After it set registers and do a far jump to kernel.
If data selector is 10h, code selector is 8 and kernel offset is 10000h do:
<syntaxhighlight lang="asm">
mov ax,10h
mov ds,ax
mov es,ax
mov fs,ax
mov gs,ax
mov ss,ax
jmp 8:10000h
</syntaxhighlight>

Notes:
* that in this case, the GDT will be ''temporary''. Indeed, the loader has no idea of what the kernel wants to do with the GDT, so all it can do is providing a minimal and let the kernel reload GDTR with an appropriate GDT later.
* it's common for the loader to keep interrupts disabled (the kernel will enable them later when an IDT is properly set up)
* give yourself the time about thinking whether you'll enable paging now or not. Keep in mind that debugging paging initialization code without the help of exception handlers may quickly become a nightmare!
* it is possible to perform more initialization once protected mode is enabled and before kernel is loaded. This will, however, require that you mix 16 bits and 32bits code in a single object file, which can quickly become a nightmare too...
* it is very likely that your kernel does not start with an executable code, rather it has an ELF or PE header at 10000h. You'll have to parse that to get the entry point to jump to.

== You have long way to continue... ==
Now, you are very distant from using extern and call function for C Code. You will need to enable [[A20]], make something which reads images (so you can actually boot any .bin or .sys file), and so on.

== Help I'm Stuck! ==
The only way to figure out what's wrong with your boot loader is to debug it in a VM. You could probably print out variables, but the limited space makes this uneccesarily hard.
Also reading [[My_Bootloader_Does_Not_Work|common mistakes and gotchas]] may give you ideas about your issue.

[[Category:OS Development]]
[[Category:Tutorials]]
[[Category:Bootloaders]]
[[de:Eigener Bootloader]]

== A list of things you might want to do ==

* Setup 16-bit segment registers and stack
* Print startup message
* Check presence of PCI, CPUID, MSRs
* Enable and confirm enabled A20 line
* Load GDTR
* Inform BIOS of target processor mode
* Get memory map from BIOS
* Locate kernel in filesystem
* Allocate memory to load kernel image
* Load kernel image into buffer
* Enable graphics mode
* Check kernel image ELF headers
* Enable long mode, if 64-bit
* Allocate and map memory for kernel segments
* Setup stack
* Setup COM serial output port
* Setup IDT
* Disable PIC
* Check presence of CPU features (NX, SMEP, x87, PCID, global pages, TCE, WP, MMX, SSE, SYSCALL), and enable them
* Assign a PAT to write combining
* Setup FS/GS base
* Load IDTR
* Enable APIC and setup using information in ACPI tables
* Setup GDT and TSS
````

<a id="w001884"></a>

## W001884 — Canadian Cross

Bron-ID: W001884
Namespace: Main (0)
Revisie: 3158 | Datum: 2007-03-06T13:32:52Z
Bron: https://wiki.osdev.org/Canadian_Cross
Revisie-URL: https://wiki.osdev.org/index.php?title=Canadian_Cross&oldid=3158
Doorverwijzing: GCC Canadian Cross

````text
#REDIRECT [[GCC Canadian Cross]]
````

<a id="w001893"></a>

## W001893 — Dev-C++

Bron-ID: W001893
Namespace: Main (0)
Revisie: 28244 | Datum: 2023-07-10T23:38:35Z
Bron: https://wiki.osdev.org/Dev-C%2B%2B
Revisie-URL: https://wiki.osdev.org/index.php?title=Dev-C%2B%2B&oldid=28244

````text
Dev-C++ is an advanced, freely distributed integrated development environment for Windows, created by Bloodshed Software. It is based on the [[MinGW]] implementation of [[GCC]]. A similar IDE, [http://www.bloodshed.net/devpascal.html Dev-Pascal], is available and works with both Free Pascal and GNU Pascal.

Perhaps the greatest feature of Dev-C++ for OSDev purposes is that it can be used with the [[Cygwin]] toolset as well as (or instead of) the MinGW one.

The last update to Dev-C++ from Bloodshed Software was made in 2005. However, in June 2011 Dev-C++ was forked by [http://orwelldevcpp.blogspot.be/ Orwell] to continue its development and since that time several bugs were fixed and new features added.

There is also [http://wxdsgn.sourceforge.net/ wxDev-C++], a Dev-C++ fork specifically targetted at users developing with the wxWidgets framework.

== Using Cygwin with Dev-C++ ==

It is possible to use a full (not minimalistic) GCC compiler with Dev-C++ by using Cygwin. To do this, you have to edit some settings of the IDE:

# Go to Tools -> Compiler Options.
# Go to the Directories tab, then add the appropriate Cygwin paths.
# You can use a custom Makefile by going to Project -> Project Options, selecting the Makefile tab and replacing the Makefile.

A more detailed and illustrated tutorial can be found at [http://forums.qj.net/guides-psp-development/55782-dev-c-w-working-cygwin-tutorial.html].

== See Also ==

=== External Links ===
* [https://dev-cpp.com Dev-C++ official website]

[[Category:C]]
[[Category:C++]]
[[Category:IDEs]]
````

<a id="w001895"></a>

## W001895 — EM64T

Bron-ID: W001895
Namespace: Main (0)
Revisie: 28117 | Datum: 2023-07-09T23:15:46Z
Bron: https://wiki.osdev.org/EM64T
Revisie-URL: https://wiki.osdev.org/index.php?title=EM64T&oldid=28117

````text
Intel Extended Memory 64 Technology ([http://www.intel.com/technology/64bitextensions/ Intel EM64T]) enables 64-bit computing on the server/workstation and desktop platforms when combined with supporting software. Intel EM64T improves performance by allowing the system to address more than 4 GB of both virtual and physical memory.

Technically speaking, EM64T offers to Intel processors what [[x86-64]] offers to AMD processors. Ancient disagreements such as 3Dnow-vs-MMX still holds, but the rest should be exactly the same. It is supported by latest XEON processors.

Intel EM64T provides support for:

* 64-bit flat virtual address space
* 64-bit pointers
* 64-bit wide general purpose registers (16 - added R8-R15)
* 64-bit integer support
* Up to 1 tebibyte (TiB)  of platform address space

Currently physical addresses are restricted to 40 bits, and linear addresses are restricted to 48 bits, sign-extended.

== See Also ==
=== Articles ===
* [[X86-64]]
* [[Creating a 64-bit kernel]]
* [[User:Stephanvanschaik/Setting_Up_Long_Mode|Setting up long mode]]

=== External Links ===
* [[Wikipedia:EM64T|EM64T]] on Wikipedia.
* [ftp://download.intel.com/design/Pentium4/manuals/25366816.pdf XEON manual]

[[Category:X86 CPU]]
[[Category:X86-64]]
````

<a id="w001896"></a>

## W001896 — How kernel, compiler, and C library work together

Bron-ID: W001896
Namespace: Main (0)
Revisie: 27935 | Datum: 2023-07-09T15:07:12Z
Bron: https://wiki.osdev.org/How_kernel%2C_compiler%2C_and_C_library_work_together
Revisie-URL: https://wiki.osdev.org/index.php?title=How_kernel%2C_compiler%2C_and_C_library_work_together&oldid=27935

````text
==Kernel==

The kernel is the core of an operating system. In a traditional design, it is responsible for memory management, I/O, interrupt handling, and various other things. And even while some modern designs like [[Microkernel]]s or [[Exokernel]]s move several of these services into user space, this matters little in the scope of this document.

The kernel makes its services available through a set of system calls; how they are called and what they do exactly differs from kernel to kernel.

==C Library==

:''Main Articles: See [[C Library]], [[Creating a C Library]]''

One thing up front: When you begin working on your kernel, you do not have a C library available. You have to provide everything yourself, except a few pieces provided by the compiler itself. You will also have to port an existing C library or write one yourself.

The C library implements the standard C functions (i.e., the things declared in <stdlib.h>, <math.h>, <stdio.h> etc.) and provides them in binary form suitable for linking with user-space applications.

In addition to standard C functions (as defined in the ISO standard), a C library might (and usually does) implement further functionality, which might or might not be defined by some standard. The standard C library says nothing about networking, for example. For Unix-like systems, the POSIX standard defines what is expected from a C library; other systems might differ fundamentally.

It should be noted that, in order to implement its functionality, the C library must call kernel functions. So, for your own OS, you can of course take a ready-made C library and just recompile it for your OS - but that requires that you tell the library how to call your kernel functions, and your kernel to actually provide those functions.

A more elaborate example is available in [[Library Calls]] or, you can use an existing [[C Library]] or [[Creating a C Library|create your own C Library]].

==Compiler / Assembler==

An Assembler takes (plaintext) source code and turns it into (binary) machine code; more precisely, it turns the source into ''object'' code, which contains additional information like symbol names, relocation information etc.

A compiler takes higher-level language source code, and either directly turns it into object code, or (as is the case with GCC) turns it into Assembler source code and invokes an Assembler for the final step.

The resulting object code does ''not'' yet contain any code for standard functions called. If you <tt>include</tt>d e.g. <tt><stdio.h></tt> and used <tt>printf()</tt>, the object code will merely contain a ''reference'' stating that a function named <tt>printf()</tt> (and taking a <tt>const char *</tt> and a number of unnamed arguments as parameters) must be linked to the object code in order to receive a complete executable.

Some compilers use standard library functions ''internally'', which might result in object files referencing e.g. <tt>memset()</tt> or <tt>memcpy()</tt> even though you did not include the header or used a function of this name. You will have to provide an implementation of these functions to the linker, or the linking will fail. The GCC freestanding environment expects only the functions <tt>memset()</tt>, <tt>memcpy()</tt>, <tt>memcmp()</tt>, and <tt>memmove()</tt>, as well as the [[libgcc]] library. Some advanced operations (e.g. 64-bits divisions on a 32-bits system) might involve ''compiler-internal'' functions. For [[GCC]], those functions are residing in libgcc. The content of this library is agnostic of what OS you use, and it won't taint your compiled kernel with licensing issues of whatever sort.

==Linker==

A linker takes the object code generated by the compiler / assembler, and ''links'' it against the C library (and / or libgcc.a or whatever link library you provide). This can be done in two ways: static, and dynamic.

===Static Linking===

When linking statically, the linker is invoked during the build process, just after the compiler / assembler run. It takes the object code, checks it for unresolved references, and checks if it can resolve these references from the available libraries. It then adds the binary code from these libraries to the executable; after this process, the executable is ''complete'', i.e. when running it does not require anything but the kernel to be present.

On the downside, the executable can become quite large, and code from the libraries is duplicated over and over, both on disk and in memory.

===Dynamic Linking===

When linking dynamically, the linker is invoked during the ''loading'' of an executable. The unresolved references in the object code are resolved against the libraries currently present in the system. This makes the on-disk executable much smaller, and allows for in-memory space-saving strategies such as ''shared libraries'' (see below).

On the downside, the executable becomes dependent on the presence of the libraries it references; if a system does not have those libraries, the executable cannot run.

===Shared Libraries===

A popular strategy is to ''share'' dynamically linked libraries across multiple executables. This means that, instead of attaching the binary of the library to the executable image, the references in the executable are tweaked, so that all executables refer to the same in-memory representation of the required library.

This requires some trickery. For one, the library must either not have any ''state'' (static or global data) at all, or it must provide a separate ''state'' for each executable. This gets even trickier with multi-threaded systems, where one executable might have more than one simultaneous control flow.

Second, in a virtual memory environment, it is usually impossible to provide a library to all executables in the system at the same virtual memory address. To access library code at an arbitrary virtual address requires the library code to be ''position independent'' (which can be achieved e.g. by setting the -PIC command line option for the [[GCC]] compiler). This requires support of the feature by the binary format (relocation tables), and can result in slightly less efficient code on some architectures.

===ABI - Application Binary Interface===

The ABI of a system defines how library function calls and kernel system calls are actually done. This includes whether parameters are passed on the stack or in registers, how function entry points are located in libraries, and other such concerns.

When using static linkage, the resulting executable depends on the kernel using the same ABI as the one the executable was built for; when using dynamic linkage, the executable depends on the libraries' ABI staying the same.

===Unresolved Symbols===

The linker is the stage where you will find out about stuff that has been added without your knowledge, and which is not provided by your environment. This can include references to <tt>alloca()</tt>, <tt>memcpy()</tt>, or several others. This is usually a sign that either your toolchain or your command line options are not correctly set up for compiling your own OS kernel - or that you are using functionality that is not yet implemented in your C library / runtime environment! You will most certainly run into trouble if you are not using a [[GCC Cross-Compiler|cross-compiler]] and the libgcc library and have implementations of memcpy, memmove, memset and memcmp.

Other symbols, such as _udiv* or __builtin_saveregs, are available in [[libgcc]]. If you get errors about missing such symbols, remember that you need to link with libgcc.

[[Category:Kernel]]
[[Category:Compilers]]
[[Category:Linkers]]
[[Category:ABI]]
````

<a id="w001897"></a>

## W001897 — IA32 Architecture Family

Bron-ID: W001897
Namespace: Main (0)
Revisie: 22159 | Datum: 2018-02-21T05:15:22Z
Bron: https://wiki.osdev.org/IA32_Architecture_Family
Revisie-URL: https://wiki.osdev.org/index.php?title=IA32_Architecture_Family&oldid=22159

````text
The following tables and notes constitute an overview of the x86-based processors produced (most of which are still available in some form today). These tables are intended as a guide only, the most reliable way to determine CPU features (amongst newer CPUs at least) is by using [[CPUID]].

The table is fairly easy to read, but a note on some of the values would be helpful. Values marked with '''Yes''' are available in all CPUs in that series, no exceptions. Items marked with '''No''' are not available at all. Items marked with '''Maybe''' are available in some of the CPUs (maybe higher spec'd machines, for instance the 486DX, or in later processor steppings). Items marked with a '''?''' are yet to be researched or confirmed. If you have some information, let us know!

Most of the information in this table comes from [[Wikipedia:Main_Page|Wikipedia]], with some coming from the Intel and AMD processor manuals.

==Intel Processors==
These processors from Intel use the CPUID string "GenuineIntel".

{| {{wikitable}}
|-
!
! Release Date
! [[FPU|FPU (80x87)]]
! [[Protected Mode]]
! [[SMP]]
! [[MMX]]
! [[PAE]]
! [[SSE]]
! Hyper-threading
! [[EM64T]]/[[AMD64]]
! Notes
|-
! 8086
| 1978
| {{Maybe|Optional}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}, 16-bit only
| First processor of the long-lasting x86 ISA, only supporting 16-bit real mode and 64KB segmentation. Eight 16-bit general-purpose registers, four 16-bit segment registers, a 16-bit instruction pointer, and a 16-bit flags register. 256 interrupts available and a 64KB I/O space. 
|-
! 80186
| 1982
| {{Maybe|Optional}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}, 16-bit only
| Designed and intended for embedded systems. First x86 processor with ENTER/LEAVE instructions, as well as PUSHA/POPA, a few other instructions, and immediate modes for PUSH, IMUL, and shift instructions. Exception 06h (Invalid Opcode, #UD) introduced with the UD2 "instruction".
|-
! 80286
| 1982
| {{Maybe|Optional}}
| {{Maybe|16-bit only}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}, 16-bit only
| First x86 processor with a "protected mode" and a 24-bit address bus and can not go from pmode to real mode without a CPU reset. It has better performance in real mode than the previous 16-bit Intel processors. Designed for multitasking and multi-user systems.
|-
! 80386
| 1985
| {{Maybe|Optional}}
| {{Yes}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| Successor to the 80286, the Intel 386 is the first processor of the IA32 architecture. It has 32 bit wide registers, supports 4 kByte paging, and a flat memory model in addition to the segmented memory model of the 80286. 
|-
! 80486
| 1989
| {{Maybe|Optional}}
| {{Yes}}
| {{Yes}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| The 486 integrates a 80x87 FPU on-chip (not the 486SX though), and supports power saving functions (System Management Mode, Stop Clock, Auto Halt Powerdown). It also supports SMP with an external APIC (though rare), and adds pipelining and on-chip caches, as well as related opcodes such as INVD, INVLPG, XADD and CMPXCHG. 
|-
! Pentium
| 1993
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| The Pentium integrates an APIC (which may be permanently disabled by the BIOS), and supports PSE (4 MiB pages). It also supports 2-way multiprocessing. 
|-
! Pentium Pro
| 1995
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{No}}
| {{Yes}}
| {{No}}
| {{No}}
| {{No}}
| The Pentium Pro supports PAE (36 bit physical address space with 2 MiB and/or 4 KiB pages), but does not have the MMX registers of the Pentium. 
|-
! Pentium MMX
| 1996
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| The Pentium MMX is very similar to the original Pentium CPU, but includes MMX SIMD registers (single instruction, multiple data). It doesn't seem logical, but the Pentium MMX was released after the Pentium Pro.
|-
! Pentium II
| 1997
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{No}}
| {{No}}
| {{No}}
| The Pentium II again supports MMX (as well as PAE), as well as additional low-power states: AutoHALT, Stop-Grant, Sleep, and ~DeepSleep. 
|-
! Pentium II Xeon
| 1998
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{No}}
| {{No}}
| {{No}}
| The Xeon supports 4/8/+ way multiprocessing. 
|-
! Pentium III
| 1999
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| SSE
| {{No}}
| {{No}}
| Available in speeds from 450MHz to 1400MHz, the Pentium III  was the first to support SSE (128 bit packed single FP SIMD). Other than this it was widely similar to the Pentium II Deschutes.
|-
! Pentium III Xeon
| 1999
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| SSE
| {{No}}
| {{No}}
| 
|-
! Pentium IV
| 2000
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| SSE2*
| {{Yes}}
| {{Maybe}}
| Intel added both SSE3 and "EMT64/Intel 64" to the Prescott series.
|-
! Pentium M
| 2003
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| SSE2
| {{No}}
| {{No}}
| 
|-
! Core
| 2003
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| SSE3
| {{No}}
| {{No}}
| 
|-
! Xeon 51xx
| 2006
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| SSSE3
| {{Yes}}
| {{Yes}}
| 
|-
! Core 2
| 2006
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| SSE4.1
| {{No}}
| {{Yes}}
| 
|-
! Xeon 54xx
| 2007
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| SSE4.1
| {{No}}
| {{Yes}}
| 
|-
! Core i-series
| 2008
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| SSE4.2
| {{Yes}}
| {{Yes}}
| 
|-
! Atom 200/300
| 2008
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| SSSE3
| {{Yes}}
| {{Yes}}
| 
|-
! Atom N-series
| 2008
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| SSSE3
| {{Yes}}
| {{Maybe}}
| Intel omitted 64-bit mode from the Diamondville (N-2xx) series, but included it with Pineview (N-4xx).
|-
! Atom Z-series
| 2008
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| SSSE3
| {{Yes}}
| {{No}}
|}

==Advanced Micro Device Intel-compatible Processors==
AMD has been developing integrated circuits since the early 70's, they originally licensed the 80286 from Intel and branded it as the Am286. The company later went on to release its first Intel 386 clone, the Am386, in 1991.

The CPUID identifier string is "AuthenticAMD".

It is important to note that the "SSE" used by AMD and the "SSE" used by Intel may not be entirely compatible.

{| {{wikitable}}
|-
!
! Release Date
! [[FPU|FPU (80x87)]]
! [[Protected Mode]]
! [[SMP]]
! [[MMX]]
! [[3DNow!]]
! [[PAE]]
! [[SSE]]
! Hyper-threading
! [[EM64T]]/[[AMD64]]
! Notes
|-
! Am386
| 1991
| {{Maybe|Optional}}
| {{Yes}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| AMD's first clone of the 32-bit i386 architecture, FPU was optional.
|-
! Am486
Am5x86
| 1994
1995
| {{Yes}}
| {{Yes}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| AMD's 486 clone, 2x the cache size of most of Intel's 486 chips.
|-
! K5
| 1996
| {{Yes}}
| {{Yes}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| AMD's first try at a Pentium-compatible CPU.
|-
! K6
| 1997
| {{Yes}}
| {{Yes}}
| {{No}}
| {{Yes}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| Actually designed by NexGen (taken over by AMD), the K6 is a fully Pentium-compatible CPU. One notable instruction was the LOOPcc instruction, which executed in 2 cycles compared to a Pentium's 18, causing timing problems.
|-
! K6-2
| 1998
| {{Yes}}
| {{Yes}}
| {{No}}
| {{Yes}}
| {{Yes}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| AMD added 16 wait states to the execution of the LOOPcc and thus caused it to slow to the speed of a Pentium. They added a special case (speculation, might be coincidence) for the DEC (E)CX; Jcc combination, which is semantically equivalent with the LOOPcc instruction; since LOOPcc was faster on Intels, nobody used the DEC/Jcc combo there. So AMD kept the original speed for this combo, and specified in their optimization manuals that this was the preferred method over the LOOPcc instruction.

The K6-2 also featured the 3DNOW! technology, which was "MMX using floating point numbers", and multiplexed (again) on the floating point registers. It was largely compatible with the P2.
|-
! K6-3
| 1999
| {{Yes}}
| {{Yes}}
| {{No}}
| {{Yes}}
| {{Yes}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| This design was fully P2 compatible.

The K6-3 suffered from a bottleneck at the instruction decode unit (which converts the x86 instructions to native instructions). While it did have 3 execution units of each type (ALU / MMX / loadstore), they were not used much at all since the instruction decode unit could not keep up.
|-
! Athlon
| 1999
| {{Yes}}
| {{Yes}}
| {{No}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{No}}
| {{No}}
| {{No}}
| 
|-
! Athlon XP/MP
| 2001
| {{Yes}}
| {{Yes}}
| {{Maybe}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| SSE
| {{No}}
| {{No}}
| Athlon XP (starting with Palomino) introduced SSE. SMP capable chips were branded as Athlon MP.
|-
! Athlon 64
| 2003
| {{Yes}}
| {{Yes}}
| {{No}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| SSE3
| {{No}}
| {{Yes}}
| 
|-
! Athlon 64 X2
| 2005
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| SSE3
| {{No}}
| {{Yes}}
| 
|-
! Phenom
| 2007
| {{Yes}}
| {{Yes}}
| ?
| {{Yes}}
| {{Yes}}
| {{Yes}}
| SSE4a
| {{No}}
| {{Yes}}
|
|-
! Ryzen Series
| 2017
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{No}}
| {{Yes}}
| SSE4.2
| {{Yes}}
| {{Yes}}
|
|-
|}

===AMD64 based CPU's===
These processors all support the entire IA32 family natively. AMD created a new processor, with 48-bit memory addressing and 64-bit calculations, being very compatible with the old style CPU's. So compatible, that the core for 32-bit and 64-bit is essentially identical, aside from the size of calculations and the support of a few encodings that were effectively redundant. They removed a few 1-byte opcodes (about 20 in total, including all 1-byte INC and 1-byte DEC instructions) to make room for a new REX prefix. They modified the core to use 16 registers instead of 8, added a load of new names, got the old software working, and optimized the 32-bit performance.

== Other CPU vendors making similar chips==

===Cyrix===

Cyrix was a well-known CPU vendor from the 386 years (and slightly before), up to Pentium II times, when it more or less vanished inside Via. Via now uses the name as a CPU name (not making it clearer), but this section is about the original Cyrix CPU's. The processors supporting [[CPUID]] call it "CyrixInstead".

==== Cyrix 387 ====

This isn't actually a processor but an FPU. It was the fastest coprocessor to the 386 to be found, and was even very usable aside a 486-SX.

==== Cyrix 4x86 ====

A processor that had the abilities of a 486. The first generation plugged into 386 sockets giving greater speeds without requiring extra hardware updates. Later editions could also be used on 486 motherboards.

==== Cyrix 5x86 ====

A processor that performed as a 486 and was socket-compatible. It features some of the Pentium's abilities, but not all (such as cmpxchg8b).

==== Cyrix 6x86 / M1 ====

This processor is, even though the name suggests otherwise, compatible with the 586 (Pentium). It didn't contain any of the MMX or PPro features. It performed slightly better per cycle compared to the Pentium Pro, and was thus given ratings. The performance of floating point operations was lower than that of the Pentium Pro.

==== Cyrix 6x86MX / M2 ====

Was a Pentium MMX compatible processor, also using ratings. It was again socket-compatible to the Pentium MMX and the older Pentiums (without MMX). It supported a few features from the Pentium Pro, among which the very usable CMOVcc set.

==== MediaGX ====

This was a chip that, apart from the CPU, included several peripherals on-chip including graphics and audio devices. 

The company was bought by National Semiconductor, who sold the trademark to VIA. The MediaGX was developed further and was eventually bought by AMD who marketed it as the Geode.

=== Rise Technologies ===

A company reputedly producing Pentium-compatible chips, without MMX. Little detail is known, but the CPUID identifier string was "RiseRiseRise", or the same in all 3 dwords (making a search for it very easy).

==See Also==

=== Articles ===

===Threads===
[[Topic:10030|AT,XT and PC]]

=== External Links ===

* [[wikipedia:X86|x86]] on Wikipedia


[[Category:X86]]
````

<a id="w001900"></a>

## W001900 — PS/2 Keyboard

Bron-ID: W001900
Namespace: Main (0)
Revisie: 30840 | Datum: 2026-07-02T19:01:15Z
Bron: https://wiki.osdev.org/PS/2_Keyboard
Revisie-URL: https://wiki.osdev.org/index.php?title=PS%2F2_Keyboard&oldid=30840

````text

== Overview ==

The PS/2 Keyboard is a device that talks to a PS/2 controller using [[PS/2|serial communication]]. Ideally, each different type of PS/2 controller driver should provide some sort of standard/simple "send byte/receive byte" interface, and the PS/2 Keyboard driver would use this interface without caring about lower level details (like what type of PS/2 controller the device is plugged into).

The PS/2 Keyboard accepts commands and sends responses to those commands, and also sends scan codes indicating when a key was pressed or released. Keep in mind that these scan codes (from the tables below) may be altered (i.e. translated) by the PS/2 controller, depending on the controller's settings.

== Commands ==

A PS/2 Keyboard accepts many types of commands. A command is one byte. Some commands have data byte/s which must be sent after the command byte. The keyboard typically responds to a command by sending either an "ACK" (to acknowledge the command) or a "Resend" (to say something was wrong with the previous command) back. Don't forget to wait between the command, the data and the response from keyboard.

The commands that a PS/2 Keyboard accepts are:

{| class="wikitable"
|-
! Command Byte
! Meaning
! Data Byte/s
! Response
|-
| 0xED
| Set LEDs
| LED states:
    {| class="wikitable"
    |-
    ! Bit
    ! Use
    |-
    | 0
    | ScrollLock
    |-
    | 1
    | NumberLock
    |-
    | 2
    | CapsLock
    |}
Note: Other bits may be used in international keyboards for other purposes (e.g. a Japanese keyboard might use bit 4 for a "Kana mode" LED).
| 0xFA (ACK) or 0xFE (Resend)
|-
| 0xEE
| Echo (for diagnostic purposes, and useful for device removal detection)
| None
| 0xEE (Echo) or 0xFE (Resend)
|-
| 0xF0
| Get/set current scan code set
| Sub-command:
    {| class="wikitable"
    |-
    ! Value
    ! Use
    |-
    | 0
    | Get current scan code set
    |-
    | 1
    | Set scan code set 1
    |-
    | 2
    | Set scan code set 2
    |-
    | 3
    | Set scan code set 3
    |}
| 0xFA (ACK) or 0xFE (Resend) if scan code is being set; 0xFA (ACK) then the scan code set number, or 0xFE (Resend) if you're getting the scancode. If getting the scancode the table indicates the value that identify each set:
    {| class="wikitable"
    |-
    ! Raw
    ! Translated
    ! Use
    |-
    | 1
    | 0x43
    | Scan code set 1
    |-
    | 2
    | 0x41
    | Scan code set 2
    |-
    | 3
    | 0x3F
    | Scan code set 3
    |}

|-
| 0xF2
| Identify keyboard
| None
| 0xFA (ACK) followed by none or more ID bytes (see [["8042" PS/2 Controller#Detecting PS.2F2 Device Types|"Detecting Device Types"]])
|-
| 0xF3
| Set typematic rate and delay
| Typematic byte:
    {| class="wikitable"
    |-
    ! Bit/s
    ! Meaning
    |-
    | 0 to 4
    | Repeat rate (00000b = 30 Hz, ..., 11111b = 2 Hz)
    |-
    | 5 to 6
    | Delay before keys repeat (00b = 250 ms, 01b = 500 ms, 10b = 750 ms, 11b = 1000 ms)
    |-
    | 7
    | Must be zero
    |}
| 0xFA (ACK) or 0xFE (Resend)
|-
| 0xF4
| Enable scanning (keyboard will send scan codes)
| None
| 0xFA (ACK) or 0xFE (Resend)
|-
| 0xF5
| Disable scanning (keyboard won't send scan codes)
Note: May also restore default parameters
| None
| 0xFA (ACK) or 0xFE (Resend)
|-
| 0xF6
| Set default parameters
| None
| 0xFA (ACK) or 0xFE (Resend)
|-
| 0xF7
| Set all keys to typematic/autorepeat only (scancode set 3 only)
| None
| 0xFA (ACK) or 0xFE (Resend)
|-
| 0xF8
| Set all keys to make/release (scancode set 3 only)
| None
| 0xFA (ACK) or 0xFE (Resend)
|-
| 0xF9
| Set all keys to make only (scancode set 3 only)
| None
| 0xFA (ACK) or 0xFE (Resend)
|-
| 0xFA
| Set all keys to typematic/autorepeat/make/release (scancode set 3 only)
| None
| 0xFA (ACK) or 0xFE (Resend)
|-
| 0xFB
| Set specific key to typematic/autorepeat only (scancode set 3 only)
| Scancode for key
| 0xFA (ACK) or 0xFE (Resend)
|-
| 0xFC
| Set specific key to make/release (scancode set 3 only)
| Scancode for key
| 0xFA (ACK) or 0xFE (Resend)
|-
| 0xFD
| Set specific key to make only (scancode set 3 only)
| Scancode for key
| 0xFA (ACK) or 0xFE (Resend)
|-
| 0xFE
| Resend last byte
| None
| Previously sent byte or 0xFE (Resend)
|-
| 0xFF
| Reset and start self-test
| None
| 0xFA (ACK) or 0xFE (Resend) followed by 0xAA (self-test passed), 0xFC or 0xFD (self test failed)
|}

== Special Bytes ==

The keyboard sends bytes to the system. Some of these bytes have special meaning (e.g. responses from the commands above). The bytes the keyboard may send are:

{| class="wikitable"
|-
! Response Byte
! Meaning
|-
| 0x00
| Key detection error or internal buffer overrun
|-
| 0xAA
| Self test passed (sent after "0xFF (reset)" command or keyboard power up)
|-
| 0xEE
| Response to "0xEE (echo)" command
|-
| 0xFA
| Command acknowledged (ACK)
|-
| 0xFC and 0xFD
| Self test failed (sent after "0xFF (reset)" command or keyboard power up)
|-
| 0xFE
| Resend (keyboard wants controller to repeat last command it sent)
|-
| 0xFF
| Key detection error or internal buffer overrun
|}

All other bytes sent by the keyboard are scan codes, where interpretation depends on the currently selected scan code set.

== Driver Model ==

=== Command Queue and State Machine ===

Commands must be sent one at a time (e.g. if your driver is interrupt driven, you can't start sending a command within the IRQ handler because code outside the IRQ handler may be in the middle of sending a command). The command isn't completed until you've received an ACK for it. For example, if you send a command and the keyboard responds with "0xFE (resend)" then you have to send the command again (possibly limited to 3 retries before you give up and assume the keyboard doesn't support the command you're sending or there's been a hardware failure). Finally, sometimes you want to send several commands at once. For example, you might have a "reinitialise()" function that sets the scan code set, sets the typematic byte, sets the LEDs and enables scanning.

The simplest way to achieve this is for the driver to maintain a queue of commands. When you add a command to the queue, if the queue is empty you start sending the command; otherwise you append the command to the queue. When you receive an "0xFA (ACK)" from the keyboard you discard the command at the head of the queue and start sending the next command in the queue (if any). If you receive an "0xFE (Resend)" from the keyboard you can resend the command at the head of the queue.

The remainder of the driver should be a kind of state machine. The state machine moves into a different state when some commands are successfully completed, and when various bytes are received from the keyboard. For example, the driver might be in a default state and receive a break code that puts it into a "waiting for scan code after receiving break code" state. Then it might receive the first byte of a multi-byte scan code and shift to a "waiting for second byte of scan code after receiving break code" state. Finally it might receive the second/last byte of the scan code and then switch back to the default state.


=== Scan Code Sets, Scan Codes and Key Codes ===

A scan code set is a set of codes that determine when a key is pressed or repeated, or released. There are 3 different sets of scan codes. The oldest is "scan code set 1", the default is "scan code set 2", and there is a newer (more complex) "scan code set 3". ''Note: Normally on PC compatible systems the keyboard itself uses scan code set 2 and the keyboard controller translates this into scan code set 1 for compatibility. See [["8042"_PS/2_Controller#Translation|"8042"_PS/2_Controller]] for more information about this translation.''

Modern keyboards should support all three scan code sets, however some don't. Scan code set 2 (the default) is the only scan code set that is guaranteed to be supported. In theory (I haven't tried it) it should be possible to attempt to set scan code set 1 or scan code set 3, and then ask the keyboard which scan code it is currently using and see if it actually is using the requested scan code set. In this way it may be possible to determine which scan code sets the keyboard does support.

Scan codes themselves are sequences of one or more bytes. In some cases the sequence can be as many as 6 bytes (e.g. the Pause/Break key in scan code set 1 
generates the sequence 0xE1, 0x1D, 0x45, 0xE1, 0x9D, 0xC5 when pressed). This situation isn't really ideal. In general (for later processing) you want to convert these "one or more byte sequences" into a single integer that uniquely identifies a specific key, that can be used effectively in things like lookup tables (without having sparsely used "many GiB" lookup tables).

There is no standard for "key codes" - it's something you have to make up or invent for your OS. I personally like the idea of having an 8-bit key code where the highest 3 bits determine which row on the keyboard and the lowest 5 bits determine which column (essentially, the keyboard is treated as a grid of up to 8 rows and up to 32 columns of keys). Regardless of what you choose to use for your key codes, it should be something that is used by all keyboard drivers (including USB Keyboards) and could possibly also be used for other input devices (e.g. left mouse button might be treated as "key code 0xF1").

Basically, when the keyboard driver's state machine knows it has received a complete scan code, the next step is to convert the "one or more byte" scan code into a key code.

=== Key Codes, Key States and Key Mappings ===

Once you've got key codes, then next step is to keep track of which keys are currently being pressed. Imagine a computer game that uses the "WASD" keys for player movement - when the 'A' key is being pressed the player rotates clockwise. How does the game know if the 'A' key is currently being pressed? For this you'd want an array of flags, where each flag corresponds to a key code. There is a hidden bonus here - the keyboard driver itself can use the same "array of flags" to determine if a shift key, control key, alt key, etc is down, which can be quite useful when trying to convert the key code into an actual ASCII character or Unicode code point. For example, if the user presses the 'a' key then it might correspond to 'a' or 'A' (depending on capslock state and whether or not a shift key is being held down at the time) or might not correspond to a valid character at all (e.g. "control-a" or "alt-a").

Also note that (for example) a "WASD" game doesn't care if the keys are 'W', 'A', 'S' and 'D'. The game wants to know about keys in a specific "T-shaped" pattern on the left of the keyboard. If the keyboard happens to be something different, then the keys in the same location may be completely different (e.g. they would be '<', 'A', 'O' and 'E' keys on a Dvorak keyboard). This helps to explain my preference of having an 8-bit key code where the highest 3 bits determine which row on the keyboard and the lowest 5 bits determine which column (it's easy for a game to ask about the state of the third key on the left of the third row).

Once you're able to keep track of which keys are currently being pressed, the next step is to (attempt to) convert the key into an ASCII character or Unicode code point. At this point you need to know what type of keyboard the user has - is it "US QWERTY", or "French AZERTY", some form of Dvorak, or perhaps it's Arabic. To handle many different keyboard layouts, the keyboard driver needs to use tables to convert key codes into ASCII characters or Unicode code points; so that you only need to load a different "Key Mapping" table to support different keyboard layouts.

However, it's not quite that simple. Different keyboard layouts can have different meta keys, different status LEDs, etc. The Key Mapping table has to sort these differences out too. This is why you don't want to detect if the keyboard LEDs have changed earlier, but want to send the "set LEDs" command (if necessary) *after* you've found the entry for the key code in your key map table.

The final step of processing is to combine all relevant information into some sort of "keypress packet" structure, and send it to whomever (e.g. GUI). The entire "keypress packet" might include the following:
* Unicode code point (if applicable)
* Key code
* Pressed/released flag
* Various other key states (shift, alt, control, etc)
* Various "toggle" states (CapsLock, ScrollLock, NumberLock, etc)

== Scan Code Sets ==

As there are 3 different scan code sets, there are 3 different tables (one for each scan code set). Some of the scan codes correspond to extra keys that have been added over time and have become "relatively standard". To help keep track scan codes have been categorized and tagged in the tables below. The tags used are:

{| class="wikitable"
|-
! Tag
! Meaning
|-
| (keypad)
| A key that is on the numerics keypad (typically found on the right hand side of the keyboard).
|-
| (ACPI)
| A key that is part of the "ACPI" group of keys (typically found near the top of the keyboard). A lot of modern keyboards don't actually have these keys (if I remember right, they were fashionable in the late 1990's but have become less common since).
Note: Don't let the name fool you - these keys have nothing to do with ACPI at all and do behave like any other normal key (but could be useful for an OS that supports power management).
|-
| (multimedia)
| A key that is part of the multimedia group of keys (typically found near the top of the keyboard). A lot of modern keyboards do have at least some of these keys. Some of these keys are intended to be used for media players (volume control, play/pause, next track, previous track, etc), some are intended for web browsing (previous web page, next web page, refresh, favourites/bookmarks, etc), and and some are intended for launching applications (e.g. starting an email client, starting a calculator, opening "my computer", etc).
|}


=== Scan Code Set 1 ===

The following table shows which scan codes correspond to which keys when using scan code set 1 (for a "US QWERTY" keyboard only):

Note that scancodes with extended byte (E0) generates two different interrupts: the first containing the E0 byte, the second containing the scancode
{| class="wikitable"
|-
! Scan code
! Key
! style="background: #E8E8E8" | Scan code
! style="background: #E8E8E8" | Key
! Scan code
! Key
! style="background: #E8E8E8" | Scan code
! style="background: #E8E8E8" | Key
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0x01
| style="background: #E8E8E8" | escape pressed
| align="right" | 0x02
| 1 pressed
| style="background: #E8E8E8" align="right" | 0x03
| style="background: #E8E8E8" | 2 pressed
|-
| align="right" | 0x04
| 3 pressed
| style="background: #E8E8E8" align="right" | 0x05
| style="background: #E8E8E8" | 4 pressed
| align="right" | 0x06
| 5 pressed
| style="background: #E8E8E8" align="right" | 0x07
| style="background: #E8E8E8" | 6 pressed
|-
| align="right" | 0x08
| 7 pressed
| style="background: #E8E8E8" align="right" | 0x09
| style="background: #E8E8E8" | 8 pressed
| align="right" | 0x0A
| 9 pressed
| style="background: #E8E8E8" align="right" | 0x0B
| style="background: #E8E8E8" | 0 (zero) pressed
|-
| align="right" | 0x0C
| - pressed
| style="background: #E8E8E8" align="right" | 0x0D
| style="background: #E8E8E8" | = pressed
| align="right" | 0x0E
| backspace pressed
| style="background: #E8E8E8" align="right" | 0x0F
| style="background: #E8E8E8" | tab pressed
|-
| align="right" | 0x10
| Q pressed
| style="background: #E8E8E8" align="right" | 0x11
| style="background: #E8E8E8" | W pressed
| align="right" | 0x12
| E pressed
| style="background: #E8E8E8" align="right" | 0x13
| style="background: #E8E8E8" | R pressed
|-
| align="right" | 0x14
| T pressed
| style="background: #E8E8E8" align="right" | 0x15
| style="background: #E8E8E8" | Y pressed
| align="right" | 0x16
| U pressed
| style="background: #E8E8E8" align="right" | 0x17
| style="background: #E8E8E8" | I pressed
|-
| align="right" | 0x18
| O pressed
| style="background: #E8E8E8" align="right" | 0x19
| style="background: #E8E8E8" | P pressed
| align="right" | 0x1A
| [ pressed
| style="background: #E8E8E8" align="right" | 0x1B
| style="background: #E8E8E8" | ] pressed
|-
| align="right" | 0x1C
| enter pressed
| style="background: #E8E8E8" align="right" | 0x1D
| style="background: #E8E8E8" | left control pressed
| align="right" | 0x1E
| A pressed
| style="background: #E8E8E8" align="right" | 0x1F
| style="background: #E8E8E8" | S pressed
|-
| align="right" | 0x20
| D pressed
| style="background: #E8E8E8" align="right" | 0x21
| style="background: #E8E8E8" | F pressed
| align="right" | 0x22
| G pressed
| style="background: #E8E8E8" align="right" | 0x23
| style="background: #E8E8E8" | H pressed
|-
| align="right" | 0x24
| J pressed
| style="background: #E8E8E8" align="right" | 0x25
| style="background: #E8E8E8" | K pressed
| align="right" | 0x26
| L pressed
| style="background: #E8E8E8" align="right" | 0x27
| style="background: #E8E8E8" | ; pressed
|-
| align="right" | 0x28
| ' (single quote) pressed
| style="background: #E8E8E8" align="right" | 0x29
| style="background: #E8E8E8" | ` (back tick) pressed
| align="right" | 0x2A
| left shift pressed
| style="background: #E8E8E8" align="right" | 0x2B
| style="background: #E8E8E8" | \ pressed
|-
| align="right" | 0x2C
| Z pressed
| style="background: #E8E8E8" align="right" | 0x2D
| style="background: #E8E8E8" | X pressed
| align="right" | 0x2E
| C pressed
| style="background: #E8E8E8" align="right" | 0x2F
| style="background: #E8E8E8" | V pressed
|-
| align="right" | 0x30
| B pressed
| style="background: #E8E8E8" align="right" | 0x31
| style="background: #E8E8E8" | N pressed
| align="right" | 0x32
| M pressed
| style="background: #E8E8E8" align="right" | 0x33
| style="background: #E8E8E8" | , pressed
|-
| align="right" | 0x34
| . pressed
| style="background: #E8E8E8" align="right" | 0x35
| style="background: #E8E8E8" | / pressed
| align="right" | 0x36
| right shift pressed
| style="background: #E8E8E8" align="right" | 0x37
| style="background: #E8E8E8" | (keypad) * pressed
|-
| align="right" | 0x38
| left alt pressed
| style="background: #E8E8E8" align="right" | 0x39
| style="background: #E8E8E8" | space pressed
| align="right" | 0x3A
| CapsLock pressed
| style="background: #E8E8E8" align="right" | 0x3B
| style="background: #E8E8E8" | F1 pressed
|-
| align="right" | 0x3C
| F2 pressed
| style="background: #E8E8E8" align="right" | 0x3D
| style="background: #E8E8E8" | F3 pressed
| align="right" | 0x3E
| F4 pressed
| style="background: #E8E8E8" align="right" | 0x3F
| style="background: #E8E8E8" | F5 pressed
|-
| align="right" | 0x40
| F6 pressed
| style="background: #E8E8E8" align="right" | 0x41
| style="background: #E8E8E8" | F7 pressed
| align="right" | 0x42
| F8 pressed
| style="background: #E8E8E8" align="right" | 0x43
| style="background: #E8E8E8" | F9 pressed
|-
| align="right" | 0x44
| F10 pressed
| style="background: #E8E8E8" align="right" | 0x45
| style="background: #E8E8E8" | NumberLock pressed
| align="right" | 0x46
| ScrollLock pressed
| style="background: #E8E8E8" align="right" | 0x47
| style="background: #E8E8E8" | (keypad) 7 pressed
|-
| align="right" | 0x48
| (keypad) 8 pressed
| style="background: #E8E8E8" align="right" | 0x49
| style="background: #E8E8E8" | (keypad) 9 pressed
| align="right" | 0x4A
| (keypad) - pressed
| style="background: #E8E8E8" align="right" | 0x4B
| style="background: #E8E8E8" | (keypad) 4 pressed
|-
| align="right" | 0x4C
| (keypad) 5 pressed
| style="background: #E8E8E8" align="right" | 0x4D
| style="background: #E8E8E8" | (keypad) 6 pressed
| align="right" | 0x4E
| (keypad) + pressed
| style="background: #E8E8E8" align="right" | 0x4F
| style="background: #E8E8E8" | (keypad) 1 pressed
|-
| align="right" | 0x50
| (keypad) 2 pressed
| style="background: #E8E8E8" align="right" | 0x51
| style="background: #E8E8E8" | (keypad) 3 pressed
| align="right" | 0x52
| (keypad) 0 pressed
| style="background: #E8E8E8" align="right" | 0x53
| style="background: #E8E8E8" | (keypad) . pressed
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" | 
| 
| 
| style="background: #E8E8E8" align="right" | 0x57
| style="background: #E8E8E8" | F11 pressed
|-
| align="right" | 0x58
| F12 pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" | 
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" | 
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0x81
| style="background: #E8E8E8" | escape released
| align="right" | 0x82
| 1 released
| style="background: #E8E8E8" align="right" | 0x83
| style="background: #E8E8E8" | 2 released
|-
| align="right" | 0x84
| 3 released
| style="background: #E8E8E8" align="right" | 0x85
| style="background: #E8E8E8" | 4 released
| align="right" | 0x86
| 5 released
| style="background: #E8E8E8" align="right" | 0x87
| style="background: #E8E8E8" | 6 released
|-
| align="right" | 0x88
| 7 released
| style="background: #E8E8E8" align="right" | 0x89
| style="background: #E8E8E8" | 8 released
| align="right" | 0x8A
| 9 released
| style="background: #E8E8E8" align="right" | 0x8B
| style="background: #E8E8E8" | 0 (zero) released
|-
| align="right" | 0x8C
| - released
| style="background: #E8E8E8" align="right" | 0x8D
| style="background: #E8E8E8" | = released
| align="right" | 0x8E
| backspace released
| style="background: #E8E8E8" align="right" | 0x8F
| style="background: #E8E8E8" | tab released
|-
| align="right" | 0x90
| Q released
| style="background: #E8E8E8" align="right" | 0x91
| style="background: #E8E8E8" | W released
| align="right" | 0x92
| E released
| style="background: #E8E8E8" align="right" | 0x93
| style="background: #E8E8E8" | R released
|-
| align="right" | 0x94
| T released
| style="background: #E8E8E8" align="right" | 0x95
| style="background: #E8E8E8" | Y released
| align="right" | 0x96
| U released
| style="background: #E8E8E8" align="right" | 0x97
| style="background: #E8E8E8" | I released
|-
| align="right" | 0x98
| O released
| style="background: #E8E8E8" align="right" | 0x99
| style="background: #E8E8E8" | P released
| align="right" | 0x9A
| [ released
| style="background: #E8E8E8" align="right" | 0x9B
| style="background: #E8E8E8" | ] released
|-
| align="right" | 0x9C
| enter released
| style="background: #E8E8E8" align="right" | 0x9D
| style="background: #E8E8E8" | left control released
| align="right" | 0x9E
| A released
| style="background: #E8E8E8" align="right" | 0x9F
| style="background: #E8E8E8" | S released
|-
| align="right" | 0xA0
| D released
| style="background: #E8E8E8" align="right" | 0xA1
| style="background: #E8E8E8" | F released
| align="right" | 0xA2
| G released
| style="background: #E8E8E8" align="right" | 0xA3
| style="background: #E8E8E8" | H released
|-
| align="right" | 0xA4
| J released
| style="background: #E8E8E8" align="right" | 0xA5
| style="background: #E8E8E8" | K released
| align="right" | 0xA6
| L released
| style="background: #E8E8E8" align="right" | 0xA7
| style="background: #E8E8E8" | ; released
|-
| align="right" | 0xA8
| ' (single quote) released
| style="background: #E8E8E8" align="right" | 0xA9
| style="background: #E8E8E8" | ` (back tick) released
| align="right" | 0xAA
| left shift released
| style="background: #E8E8E8" align="right" | 0xAB
| style="background: #E8E8E8" | \ released
|-
| align="right" | 0xAC
| Z released
| style="background: #E8E8E8" align="right" | 0xAD
| style="background: #E8E8E8" | X released
| align="right" | 0xAE
| C released
| style="background: #E8E8E8" align="right" | 0xAF
| style="background: #E8E8E8" | V released
|-
| align="right" | 0xB0
| B released
| style="background: #E8E8E8" align="right" | 0xB1
| style="background: #E8E8E8" | N released
| align="right" | 0xB2
| M released
| style="background: #E8E8E8" align="right" | 0xB3
| style="background: #E8E8E8" | , released
|-
| align="right" | 0xB4
| . released
| style="background: #E8E8E8" align="right" | 0xB5
| style="background: #E8E8E8" | / released
| align="right" | 0xB6
| right shift released
| style="background: #E8E8E8" align="right" | 0xB7
| style="background: #E8E8E8" | (keypad) * released
|-
| align="right" | 0xB8
| left alt released
| style="background: #E8E8E8" align="right" | 0xB9
| style="background: #E8E8E8" | space released
| align="right" | 0xBA
| CapsLock released
| style="background: #E8E8E8" align="right" | 0xBB
| style="background: #E8E8E8" | F1 released
|-
| align="right" | 0xBC
| F2 released
| style="background: #E8E8E8" align="right" | 0xBD
| style="background: #E8E8E8" | F3 released
| align="right" | 0xBE
| F4 released
| style="background: #E8E8E8" align="right" | 0xBF
| style="background: #E8E8E8" | F5 released
|-
| align="right" | 0xC0
| F6 released
| style="background: #E8E8E8" align="right" | 0xC1
| style="background: #E8E8E8" | F7 released
| align="right" | 0xC2
| F8 released
| style="background: #E8E8E8" align="right" | 0xC3
| style="background: #E8E8E8" | F9 released
|-
| align="right" | 0xC4
| F10 released
| style="background: #E8E8E8" align="right" | 0xC5
| style="background: #E8E8E8" | NumberLock released
| align="right" | 0xC6
| ScrollLock released
| style="background: #E8E8E8" align="right" | 0xC7
| style="background: #E8E8E8" | (keypad) 7 released
|-
| align="right" | 0xC8
| (keypad) 8 released
| style="background: #E8E8E8" align="right" | 0xC9
| style="background: #E8E8E8" | (keypad) 9 released
| align="right" | 0xCA
| (keypad) - released
| style="background: #E8E8E8" align="right" | 0xCB
| style="background: #E8E8E8" | (keypad) 4 released
|-
| align="right" | 0xCC
| (keypad) 5 released
| style="background: #E8E8E8" align="right" | 0xCD
| style="background: #E8E8E8" | (keypad) 6 released
| align="right" | 0xCE
| (keypad) + released
| style="background: #E8E8E8" align="right" | 0xCF
| style="background: #E8E8E8" | (keypad) 1 released
|-
| align="right" | 0xD0
| (keypad) 2 released
| style="background: #E8E8E8" align="right" | 0xD1
| style="background: #E8E8E8" | (keypad) 3 released
| align="right" | 0xD2
| (keypad) 0 released
| style="background: #E8E8E8" align="right" | 0xD3
| style="background: #E8E8E8" | (keypad) . released
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" | 
| 
| 
| style="background: #E8E8E8" align="right" | 0xD7
| style="background: #E8E8E8" | F11 released
|-
| align="right" | 0xD8
| F12 released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" | 
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" | 
|- 
| align="right" | 0xE0, 0x10
| (multimedia) previous track pressed
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
| 
| 
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
|- 
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0x19
| style="background: #E8E8E8" | (multimedia) next track pressed
| 
| 
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
|- 
| align="right" | 0xE0, 0x1C
| (keypad) enter pressed
| style="background: #E8E8E8" align="right" | 0xE0, 0x1D
| style="background: #E8E8E8" | right control pressed
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" | 
|- 
| align="right" | 0xE0, 0x20
| (multimedia) mute pressed
| style="background: #E8E8E8" align="right" | 0xE0, 0x21
| style="background: #E8E8E8" | (multimedia) calculator pressed
| align="right" | 0xE0, 0x22
| (multimedia) play pressed
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
|- 
| align="right" | 0xE0, 0x24
| (multimedia) stop pressed
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
| 
| 
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
|- 
| 
| 
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
| align="right" | 0xE0, 0x2E
| (multimedia) volume down pressed
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
|- 
| align="right" | 0xE0, 0x30
| (multimedia) volume up pressed
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
| align="right" | 0xE0, 0x32
| (multimedia) WWW home pressed
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
|- 
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0x35
| style="background: #E8E8E8" | (keypad) / pressed
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" | 
|-
| align="right" | 0xE0, 0x38
| right alt (or altGr) pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" | 
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" | 
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" | 
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0x47
| style="background: #E8E8E8" | home pressed
|-
| align="right" | 0xE0, 0x48
| cursor up pressed
| style="background: #E8E8E8" align="right" | 0xE0, 0x49
| style="background: #E8E8E8" | page up pressed
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0x4B
| style="background: #E8E8E8" | cursor left pressed
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0x4D
| style="background: #E8E8E8" | cursor right pressed
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0x4F
| style="background: #E8E8E8" | end pressed
|-
| align="right" | 0xE0, 0x50
| cursor down pressed
| style="background: #E8E8E8" align="right" | 0xE0, 0x51
| style="background: #E8E8E8" | page down pressed
| align="right" | 0xE0, 0x52
| insert pressed
| style="background: #E8E8E8" align="right" | 0xE0, 0x53
| style="background: #E8E8E8" | delete pressed
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" | 
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0x5B
| style="background: #E8E8E8" | left GUI pressed
|-
| align="right" | 0xE0, 0x5C
| right GUI pressed
| style="background: #E8E8E8" align="right" | 0xE0, 0x5D
| style="background: #E8E8E8" | "apps" pressed
| align="right" | 0xE0, 0x5E
| (ACPI) power pressed
| style="background: #E8E8E8" align="right" | 0xE0, 0x5F
| style="background: #E8E8E8" | (ACPI) sleep pressed
|- 
| 
| 
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0x63
| style="background: #E8E8E8" | (ACPI) wake pressed
|- 
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0x65
| style="background: #E8E8E8" | (multimedia) WWW search pressed
| align="right" | 0xE0, 0x66
| (multimedia) WWW favorites pressed
| style="background: #E8E8E8" align="right" | 0xE0, 0x67
| style="background: #E8E8E8" | (multimedia) WWW refresh pressed
|- 
| align="right" | 0xE0, 0x68
| (multimedia) WWW stop pressed
| style="background: #E8E8E8" align="right" | 0xE0, 0x69
| style="background: #E8E8E8" | (multimedia) WWW forward pressed
| align="right" | 0xE0, 0x6A
| (multimedia) WWW back pressed
| style="background: #E8E8E8" align="right" | 0xE0, 0x6B
| style="background: #E8E8E8" | (multimedia) my computer pressed
|- 
| align="right" | 0xE0, 0x6C
| (multimedia) email pressed
| style="background: #E8E8E8" align="right" | 0xE0, 0x6D
| style="background: #E8E8E8" | (multimedia) media select pressed
| 
| 
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
|- 
| align="right" | 0xE0, 0x90
| (multimedia) previous track released
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
| 
| 
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
|- 
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0x99
| style="background: #E8E8E8" | (multimedia) next track released
| 
| 
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
|- 
| align="right" | 0xE0, 0x9C
| (keypad) enter released
| style="background: #E8E8E8" align="right" | 0xE0, 0x9D
| style="background: #E8E8E8" | right control released
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" | 
|- 
| align="right" | 0xE0, 0xA0
| (multimedia) mute released
| style="background: #E8E8E8" align="right" | 0xE0, 0xA1
| style="background: #E8E8E8" | (multimedia) calculator released
| align="right" | 0xE0, 0xA2
| (multimedia) play released
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
|- 
| align="right" | 0xE0, 0xA4
| (multimedia) stop released
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
| 
| 
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
|- 
| 
| 
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
| align="right" | 0xE0, 0xAE
| (multimedia) volume down released
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
|- 
| align="right" | 0xE0, 0xB0
| (multimedia) volume up released
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
| align="right" | 0xE0, 0xB2
| (multimedia) WWW home released
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
|- 
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0xB5
| style="background: #E8E8E8" | (keypad) / released
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" | 
|-
| align="right" | 0xE0, 0xB8
| right alt (or altGr) released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" | 
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" | 
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" | 
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0xC7
| style="background: #E8E8E8" | home released
|-
| align="right" | 0xE0, 0xC8
| cursor up released
| style="background: #E8E8E8" align="right" | 0xE0, 0xC9
| style="background: #E8E8E8" | page up released
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0xCB
| style="background: #E8E8E8" | cursor left released
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0xCD
| style="background: #E8E8E8" | cursor right released
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0xCF
| style="background: #E8E8E8" | end released
|-
| align="right" | 0xE0, 0xD0
| cursor down released
| style="background: #E8E8E8" align="right" | 0xE0, 0xD1
| style="background: #E8E8E8" | page down released
| align="right" | 0xE0, 0xD2
| insert released
| style="background: #E8E8E8" align="right" | 0xE0, 0xD3
| style="background: #E8E8E8" | delete released
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" | 
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0xDB
| style="background: #E8E8E8" | left GUI released
|-
| align="right" | 0xE0, 0xDC
| right GUI released
| style="background: #E8E8E8" align="right" | 0xE0, 0xDD
| style="background: #E8E8E8" | "apps" released
| align="right" | 0xE0, 0xDE
| (ACPI) power released
| style="background: #E8E8E8" align="right" | 0xE0, 0xDF
| style="background: #E8E8E8" | (ACPI) sleep released
|-
| 
| 
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0xE3
| style="background: #E8E8E8" | (ACPI) wake released
|- 
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0xE5
| style="background: #E8E8E8" | (multimedia) WWW search released
| align="right" | 0xE0, 0xE6
| (multimedia) WWW favorites released
| style="background: #E8E8E8" align="right" | 0xE0, 0xE7
| style="background: #E8E8E8" | (multimedia) WWW refresh released
|- 
| align="right" | 0xE0, 0xE8
| (multimedia) WWW stop released
| style="background: #E8E8E8" align="right" | 0xE0, 0xE9
| style="background: #E8E8E8" | (multimedia) WWW forward released
| align="right" | 0xE0, 0xEA
| (multimedia) WWW back released
| style="background: #E8E8E8" align="right" | 0xE0, 0xEB
| style="background: #E8E8E8" | (multimedia) my computer released
|- 
| align="right" | 0xE0, 0xEC
| (multimedia) email released
| style="background: #E8E8E8" align="right" | 0xE0, 0xED
| style="background: #E8E8E8" | (multimedia) media select released
| 
| 
| style="background: #E8E8E8" | 
| style="background: #E8E8E8" | 
|- 
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" | 
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0x2A, 0xE0, 0x37
| style="background: #E8E8E8" | print screen pressed
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" | 
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0xB7, 0xE0, 0xAA
| style="background: #E8E8E8" | print screen released
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0xE1, 0x1D, 0x45, 0xE1, 0x9D, 0xC5
| style="background: #E8E8E8" | pause pressed
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" | 
|}


Note: There is no scan code for "pause key released" (it behaves as if it is released as soon as it's pressed)

=== Scan Code Set 2 ===

The following table shows which "make" scan codes correspond to which keys when using scan code set 2 (for a "US QWERTY" keyboard only):

{| class="wikitable"
|-
! Scan code
! Key
! style="background: #E8E8E8" | Scan code
! style="background: #E8E8E8" | Key
! Scan code
! Key
! style="background: #E8E8E8" | Scan code
! style="background: #E8E8E8" | Key
|-
|
|
| style="background: #E8E8E8" align="right" | 0x01
| style="background: #E8E8E8" | F9 pressed
|
|
| style="background: #E8E8E8" align="right" | 0x03
| style="background: #E8E8E8" | F5 pressed
|-
| align="right" | 0x04
| F3 pressed
| style="background: #E8E8E8" align="right" | 0x05
| style="background: #E8E8E8" | F1 pressed
| align="right" | 0x06
| F2 pressed
| style="background: #E8E8E8" align="right" | 0x07
| style="background: #E8E8E8" | F12 pressed
|-
|
|
| style="background: #E8E8E8" align="right" | 0x09
| style="background: #E8E8E8" | F10 pressed
| align="right" | 0x0A
| F8 pressed
| style="background: #E8E8E8" align="right" | 0x0B
| style="background: #E8E8E8" | F6 pressed
|-
| align="right" | 0x0C
| F4 pressed
| style="background: #E8E8E8" align="right" | 0x0D
| style="background: #E8E8E8" | tab pressed
| align="right" | 0x0E
| ` (back tick) pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0x11
| style="background: #E8E8E8" | left alt pressed
| align="right" | 0x12
| left shift pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0x14
| left control pressed
| style="background: #E8E8E8" align="right" | 0x15
| style="background: #E8E8E8" | Q pressed
| align="right" | 0x16
| 1 pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| align="right" | 0x1A
| Z pressed
| style="background: #E8E8E8" align="right" | 0x1B
| style="background: #E8E8E8" | S pressed
|-
| align="right" | 0x1C
| A pressed
| style="background: #E8E8E8" align="right" | 0x1D
| style="background: #E8E8E8" | W pressed
| align="right" | 0x1E
| 2 pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0x21
| style="background: #E8E8E8" | C pressed
| align="right" | 0x22
| X pressed
| style="background: #E8E8E8" align="right" | 0x23
| style="background: #E8E8E8" | D pressed
|-
| align="right" | 0x24
| E pressed
| style="background: #E8E8E8" align="right" | 0x25
| style="background: #E8E8E8" | 4 pressed
| align="right" | 0x26
| 3 pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0x29
| style="background: #E8E8E8" | space pressed
| align="right" | 0x2A
| V pressed
| style="background: #E8E8E8" align="right" | 0x2B
| style="background: #E8E8E8" | F pressed
|-
| align="right" | 0x2C
| T pressed
| style="background: #E8E8E8" align="right" | 0x2D
| style="background: #E8E8E8" | R pressed
| align="right" | 0x2E
| 5 pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0x31
| style="background: #E8E8E8" | N pressed
| align="right" | 0x32
| B pressed
| style="background: #E8E8E8" align="right" | 0x33
| style="background: #E8E8E8" | H pressed
|-
| align="right" | 0x34
| G pressed
| style="background: #E8E8E8" align="right" | 0x35
| style="background: #E8E8E8" | Y pressed
| align="right" | 0x36
| 6 pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| align="right" | 0x3A
| M pressed
| style="background: #E8E8E8" align="right" | 0x3B
| style="background: #E8E8E8" | J pressed
|-
| align="right" | 0x3C
| U pressed
| style="background: #E8E8E8" align="right" | 0x3D
| style="background: #E8E8E8" | 7 pressed
| align="right" | 0x3E
| 8 pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0x41
| style="background: #E8E8E8" | , pressed
| align="right" | 0x42
| K pressed
| style="background: #E8E8E8" align="right" | 0x43
| style="background: #E8E8E8" | I pressed
|-
| align="right" | 0x44
| O pressed
| style="background: #E8E8E8" align="right" | 0x45
| style="background: #E8E8E8" | 0 (zero) pressed
| align="right" | 0x46
| 9 pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
|
|
| style="background: #E8E8E8" align="right" | 0x49
| style="background: #E8E8E8" | . pressed
| align="right" | 0x4A
| / pressed
| style="background: #E8E8E8" align="right" | 0x4B
| style="background: #E8E8E8" | L pressed
|-
| align="right" | 0x4C
| ; pressed
| style="background: #E8E8E8" align="right" | 0x4D
| style="background: #E8E8E8" | P pressed
| align="right" | 0x4E
| - pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| align="right" | 0x52
| ' pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0x54
| [ pressed
| style="background: #E8E8E8" align="right" | 0x55
| style="background: #E8E8E8" | = pressed
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0x58
| CapsLock pressed
| style="background: #E8E8E8" align="right" | 0x59
| style="background: #E8E8E8" | right shift pressed
| align="right" | 0x5A
| enter pressed
| style="background: #E8E8E8" align="right" | 0x5B
| style="background: #E8E8E8" | ] pressed
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0x5D
| style="background: #E8E8E8" | \ pressed
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" | &nbsp;
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| align="right" | 0x66
| backspace pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0x69
| style="background: #E8E8E8" | (keypad) 1 pressed
| 
| 
| style="background: #E8E8E8" align="right" | 0x6B
| style="background: #E8E8E8" | (keypad) 4 pressed
|-
| align="right" | 0x6C
| (keypad) 7 pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0x70
| (keypad) 0 pressed
| style="background: #E8E8E8" align="right" | 0x71
| style="background: #E8E8E8" | (keypad) . pressed
| align="right" | 0x72
| (keypad) 2 pressed
| style="background: #E8E8E8" align="right" | 0x73
| style="background: #E8E8E8" | (keypad) 5 pressed
|-
| align="right" | 0x74
| (keypad) 6 pressed
| style="background: #E8E8E8" align="right" | 0x75
| style="background: #E8E8E8" | (keypad) 8 pressed
| align="right" | 0x76
| escape pressed
| style="background: #E8E8E8" align="right" | 0x77
| style="background: #E8E8E8" | NumberLock pressed
|-
| align="right" | 0x78
| F11 pressed
| style="background: #E8E8E8" align="right" | 0x79
| style="background: #E8E8E8" | (keypad) + pressed
| align="right" | 0x7A
| (keypad) 3 pressed
| style="background: #E8E8E8" align="right" | 0x7B
| style="background: #E8E8E8" | (keypad) - pressed
|-
| align="right" | 0x7C
| (keypad) * pressed
| style="background: #E8E8E8" align="right" | 0x7D
| style="background: #E8E8E8" | (keypad) 9 pressed
| align="right" | 0x7E
| ScrollLock pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" align="right" | 0x83
| style="background: #E8E8E8" | F7 pressed
|-
| align="right" | 0xE0, 0x10
| (multimedia) WWW search pressed
| style="background: #E8E8E8" align="right" | 0xE0, 0x11
| style="background: #E8E8E8" | right alt pressed
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0xE0, 0x14
| right control pressed
| style="background: #E8E8E8" align="right" | 0xE0, 0x15
| style="background: #E8E8E8" | (multimedia) previous track pressed
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0xE0, 0x18
| (multimedia) WWW favourites pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0x1F
| style="background: #E8E8E8" | left GUI pressed
|-
| align="right" | 0xE0, 0x20
| (multimedia) WWW refresh pressed
| style="background: #E8E8E8" align="right" | 0xE0, 0x21
| style="background: #E8E8E8" | (multimedia) volume down pressed
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0x23
| style="background: #E8E8E8" | (multimedia) mute pressed
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0x27
| style="background: #E8E8E8" | right GUI pressed
|-
| align="right" | 0xE0, 0x28
| (multimedia) WWW stop pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0x2B
| style="background: #E8E8E8" | (multimedia) calculator pressed
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0x2F
| style="background: #E8E8E8" | apps pressed
|-
| align="right" | 0xE0, 0x30
| (multimedia) WWW forward pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| align="right" | 0xE0, 0x32
| (multimedia) volume up pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0xE0, 0x34
| (multimedia) play/pause pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0x37
| style="background: #E8E8E8" | (ACPI) power pressed
|-
| align="right" | 0xE0, 0x38
| (multimedia) WWW back pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| align="right" | 0xE0, 0x3A
| (multimedia) WWW home pressed
| style="background: #E8E8E8" align="right" | 0xE0, 0x3B
| style="background: #E8E8E8" | (multimedia) stop pressed
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0x3F
| style="background: #E8E8E8" | (ACPI) sleep pressed
|-
| align="right" | 0xE0, 0x40
| (multimedia) my computer pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0xE0, 0x48
| (multimedia) email pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| align="right" | 0xE0, 0x4A
| (keypad) / pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0x4D
| style="background: #E8E8E8" | (multimedia) next track pressed
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0xE0, 0x50
| (multimedia) media select pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| align="right" | 0xE0, 0x5A
| (keypad) enter pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| align="right" | 0xE0, 0x5E
| (ACPI) wake pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0x69
| style="background: #E8E8E8" | end pressed
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0x6B
| style="background: #E8E8E8" | cursor left pressed
|-
| align="right" | 0xE0, 0x6C
| home pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0xE0, 0x70
| insert pressed
| style="background: #E8E8E8" align="right" | 0xE0, 0x71
| style="background: #E8E8E8" | delete pressed
| align="right" | 0xE0, 0x72
| cursor down pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0xE0, 0x74
| cursor right pressed
| style="background: #E8E8E8" align="right" | 0xE0, 0x75
| style="background: #E8E8E8" | cursor up pressed
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| align="right" | 0xE0, 0x7A
| page down pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0x7D
| style="background: #E8E8E8" | page up pressed
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
|
|
| style="background: #E8E8E8" align="right" | 0xF0, 0x01
| style="background: #E8E8E8" | F9 released
|
|
| style="background: #E8E8E8" align="right" | 0xF0, 0x03
| style="background: #E8E8E8" | F5 released
|-
| align="right" | 0xF0, 0x04
| F3 released
| style="background: #E8E8E8" align="right" | 0xF0, 0x05
| style="background: #E8E8E8" | F1 released
| align="right" | 0xF0, 0x06
| F2 released
| style="background: #E8E8E8" align="right" | 0xF0, 0x07
| style="background: #E8E8E8" | F12 released
|-
|
|
| style="background: #E8E8E8" align="right" | 0xF0, 0x09
| style="background: #E8E8E8" | F10 released
| align="right" | 0xF0, 0x0A
| F8 released
| style="background: #E8E8E8" align="right" | 0xF0, 0x0B
| style="background: #E8E8E8" | F6 released
|-
| align="right" | 0xF0, 0x0C
| F4 released
| style="background: #E8E8E8" align="right" | 0xF0, 0x0D
| style="background: #E8E8E8" | tab released
| align="right" | 0xF0, 0x0E
| ` (back tick) released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0xF0, 0x11
| style="background: #E8E8E8" | left alt released
| align="right" | 0xF0, 0x12
| left shift released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0xF0, 0x14
| left control released
| style="background: #E8E8E8" align="right" | 0xF0, 0x15
| style="background: #E8E8E8" | Q released
| align="right" | 0xF0, 0x16
| 1 released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| align="right" | 0xF0, 0x1A
| Z released
| style="background: #E8E8E8" align="right" | 0xF0, 0x1B
| style="background: #E8E8E8" | S released
|-
| align="right" | 0xF0, 0x1C
| A released
| style="background: #E8E8E8" align="right" | 0xF0, 0x1D
| style="background: #E8E8E8" | W released
| align="right" | 0xF0, 0x1E
| 2 released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0xF0, 0x21
| style="background: #E8E8E8" | C released
| align="right" | 0xF0, 0x22
| X released
| style="background: #E8E8E8" align="right" | 0xF0, 0x23
| style="background: #E8E8E8" | D released
|-
| align="right" | 0xF0, 0x24
| E released
| style="background: #E8E8E8" align="right" | 0xF0, 0x25
| style="background: #E8E8E8" | 4 released
| align="right" | 0xF0, 0x26
| 3 released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0xF0, 0x29
| style="background: #E8E8E8" | space released
| align="right" | 0xF0, 0x2A
| V released
| style="background: #E8E8E8" align="right" | 0xF0, 0x2B
| style="background: #E8E8E8" | F released
|-
| align="right" | 0xF0, 0x2C
| T released
| style="background: #E8E8E8" align="right" | 0xF0, 0x2D
| style="background: #E8E8E8" | R released
| align="right" | 0xF0, 0x2E
| 5 released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0xF0, 0x31
| style="background: #E8E8E8" | N released
| align="right" | 0xF0, 0x32
| B released
| style="background: #E8E8E8" align="right" | 0xF0, 0x33
| style="background: #E8E8E8" | H released
|-
| align="right" | 0xF0, 0x34
| G released
| style="background: #E8E8E8" align="right" | 0xF0, 0x35
| style="background: #E8E8E8" | Y released
| align="right" | 0xF0, 0x36
| 6 released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| align="right" | 0xF0, 0x3A
| M released
| style="background: #E8E8E8" align="right" | 0xF0, 0x3B
| style="background: #E8E8E8" | J released
|-
| align="right" | 0xF0, 0x3C
| U released
| style="background: #E8E8E8" align="right" | 0xF0, 0x3D
| style="background: #E8E8E8" | 7 released
| align="right" | 0xF0, 0x3E
| 8 released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0xF0, 0x41
| style="background: #E8E8E8" | , released
| align="right" | 0xF0, 0x42
| K released
| style="background: #E8E8E8" align="right" | 0xF0, 0x43
| style="background: #E8E8E8" | I released
|-
| align="right" | 0xF0, 0x44
| O released
| style="background: #E8E8E8" align="right" | 0xF0, 0x45
| style="background: #E8E8E8" | 0 (zero) released
| align="right" | 0xF0, 0x46
| 9 released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
|
|
| style="background: #E8E8E8" align="right" | 0xF0, 0x49
| style="background: #E8E8E8" | . released
| align="right" | 0xF0, 0x4A
| / released
| style="background: #E8E8E8" align="right" | 0xF0, 0x4B
| style="background: #E8E8E8" | L released
|-
| align="right" | 0xF0, 0x4C
| ; released
| style="background: #E8E8E8" align="right" | 0xF0, 0x4D
| style="background: #E8E8E8" | P released
| align="right" | 0xF0, 0x4E
| - released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| align="right" | 0xF0, 0x52
| ' released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0xF0, 0x54
| [ released
| style="background: #E8E8E8" align="right" | 0xF0, 0x55
| style="background: #E8E8E8" | = released
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0xF0, 0x58
| CapsLock released
| style="background: #E8E8E8" align="right" | 0xF0, 0x59
| style="background: #E8E8E8" | right shift released
| align="right" | 0xF0, 0x5A
| enter released
| style="background: #E8E8E8" align="right" | 0xF0, 0x5B
| style="background: #E8E8E8" | ] released
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0xF0, 0x5D
| style="background: #E8E8E8" | \ released
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" | &nbsp;
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| align="right" | 0xF0, 0x66
| backspace released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0xF0, 0x69
| style="background: #E8E8E8" | (keypad) 1 released
| 
| 
| style="background: #E8E8E8" align="right" | 0xF0, 0x6B
| style="background: #E8E8E8" | (keypad) 4 released
|-
| align="right" | 0xF0, 0x6C
| (keypad) 7 released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0xF0, 0x70
| (keypad) 0 released
| style="background: #E8E8E8" align="right" | 0xF0, 0x71
| style="background: #E8E8E8" | (keypad) . released
| align="right" | 0xF0, 0x72
| (keypad) 2 released
| style="background: #E8E8E8" align="right" | 0xF0, 0x73
| style="background: #E8E8E8" | (keypad) 5 released
|-
| align="right" | 0xF0, 0x74
| (keypad) 6 released
| style="background: #E8E8E8" align="right" | 0xF0, 0x75
| style="background: #E8E8E8" | (keypad) 8 released
| align="right" | 0xF0, 0x76
| escape released
| style="background: #E8E8E8" align="right" | 0xF0, 0x77
| style="background: #E8E8E8" | NumberLock released
|-
| align="right" | 0xF0, 0x78
| F11 released
| style="background: #E8E8E8" align="right" | 0xF0, 0x79
| style="background: #E8E8E8" | (keypad) + released
| align="right" | 0xF0, 0x7A
| (keypad) 3 released
| style="background: #E8E8E8" align="right" | 0xF0, 0x7B
| style="background: #E8E8E8" | (keypad) - released
|-
| align="right" | 0xF0, 0x7C
| (keypad) * released
| style="background: #E8E8E8" align="right" | 0xF0, 0x7D
| style="background: #E8E8E8" | (keypad) 9 released
| align="right" | 0xF0, 0x7E
| ScrollLock released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" align="right" | 0xF0, 0x83
| style="background: #E8E8E8" | F7 released
|-
| align="right" | 0xE0, 0x12, 0xE0, 0x7C
| print screen pressed
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0xE0, 0xF0, 0x10
| (multimedia) WWW search released
| style="background: #E8E8E8" align="right" | 0xE0, 0xF0, 0x11
| style="background: #E8E8E8" | right alt released
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0xE0, 0xF0, 0x14
| right control released
| style="background: #E8E8E8" align="right" | 0xE0, 0xF0, 0x15
| style="background: #E8E8E8" | (multimedia) previous track released
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0xE0, 0xF0, 0x18
| (multimedia) WWW favourites released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0xF0, 0x1F
| style="background: #E8E8E8" | left GUI released
|-
| align="right" | 0xE0, 0xF0, 0x20
| (multimedia) WWW refresh released
| style="background: #E8E8E8" align="right" | 0xE0, 0xF0, 0x21
| style="background: #E8E8E8" | (multimedia) volume down released
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0xF0, 0x23
| style="background: #E8E8E8" | (multimedia) mute released
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0xF0, 0x27
| style="background: #E8E8E8" | right GUI released
|-
| align="right" | 0xE0, 0xF0, 0x28
| (multimedia) WWW stop released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0xF0, 0x2B
| style="background: #E8E8E8" | (multimedia) calculator released
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0xF0, 0x2F
| style="background: #E8E8E8" | apps released
|-
| align="right" | 0xE0, 0xF0, 0x30
| (multimedia) WWW forward released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| align="right" | 0xE0, 0xF0, 0x32
| (multimedia) volume up released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0xE0, 0xF0, 0x34
| (multimedia) play/pause released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0xF0, 0x37
| style="background: #E8E8E8" | (ACPI) power released
|-
| align="right" | 0xE0, 0xF0, 0x38
| (multimedia) WWW back released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| align="right" | 0xE0, 0xF0, 0x3A
| (multimedia) WWW home released
| style="background: #E8E8E8" align="right" | 0xE0, 0xF0, 0x3B
| style="background: #E8E8E8" | (multimedia) stop released
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0xF0, 0x3F
| style="background: #E8E8E8" | (ACPI) sleep released
|-
| align="right" | 0xE0, 0xF0, 0x40
| (multimedia) my computer released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0xE0, 0xF0, 0x48
| (multimedia) email released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| align="right" | 0xE0, 0xF0, 0x4A
| (keypad) / released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0xF0, 0x4D
| style="background: #E8E8E8" | (multimedia) next track released
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0xE0, 0xF0, 0x50
| (multimedia) media select released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| align="right" | 0xE0, 0xF0, 0x5A
| (keypad) enter released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| align="right" | 0xE0, 0xF0, 0x5E
| (ACPI) wake released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0xF0, 0x69
| style="background: #E8E8E8" | end released
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0xF0, 0x6B
| style="background: #E8E8E8" | cursor left released
|-
| align="right" | 0xE0, 0xF0, 0x6C
| home released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0xE0, 0xF0, 0x70
| insert released
| style="background: #E8E8E8" align="right" | 0xE0, 0xF0, 0x71
| style="background: #E8E8E8" | delete released
| align="right" | 0xE0, 0xF0, 0x72
| cursor down released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| align="right" | 0xE0, 0xF0, 0x74
| cursor right released
| style="background: #E8E8E8" align="right" | 0xE0, 0xF0, 0x75
| style="background: #E8E8E8" | cursor up released
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| align="right" | 0xE0, 0xF0, 0x7A
| page down released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" align="right" | 0xE0, 0xF0, 0x7D
| style="background: #E8E8E8" | page up released
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| align="right" | 0xE0, 0xF0, 0x7C, 0xE0, 0xF0, 0x12
| print screen released
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
|-
| 
| 
| style="background: #E8E8E8" |
| style="background: #E8E8E8" |
| 
| 
| style="background: #E8E8E8" align="right" | 0xE1, 0x14, 0x77, 0xE1, 0xF0, 0x14, 0xF0, 0x77
| style="background: #E8E8E8" | pause pressed
|}

Note: There is no scan code for "pause key released" (it behaves as if it is released as soon as it's pressed)

=== Scan Code Set 3 ===

The following table shows which scan codes correspond to which keys when using scan code set 3 (for a "US QWERTY" keyboard only):

{| class="wikitable"
|-
!Key
!Code for Scan code set 3
|-
|A || 1C
|-
|B || 32
|-
|C || 21
|-
|D || 23
|-
|E || 24
|-
|F || 2B
|-
|G || 34
|-
|H || 33
|-
|I || 43
|-
|J || 3B
|-
|K || 42
|-
|L || 4B
|-
|M || 3A
|-
|N || 31
|-
|O || 44
|-
|P || 4D
|-
|Q || 15
|-
|R || 2D
|-
|S || 1B
|-
|T || 2C
|-
|U || 3C
|-
|V || 2A
|-
|W || 1D
|-
|X || 22
|-
|Y || 35
|-
|Z || 1A
|}

[https://web.archive.org/web/20170108131104/http://www.computer-engineering.org/ps2keyboard/scancodes3.html Scan code]

==See Also==
*[[PS/2]]
*[["8042" PS/2 Controller]]
*[[PL050 PS/2 Controller | PL050 PS/2 Controller (ARM)]]
*[[PS/2 Mouse]]
===Forum Threads===
*[[Topic:9746|Keyboard input]]
*[[Topic:9761|Up or down press?]]
*[[Topic:9711|Change typerate]]
*[[Topic:9648|Converting the scancodes]]
*[[Topic:9448|Discussion about keyboard input in a GUI]]
*[[Topic:9590|Scroll-lock LED]]
*[[Topic:10153|Keyboard LEDs (asm source)]]
*[[Topic:10053|Keyboard LEDs (C source)]]

===External Links===
*[http://www.win.tue.nl/~aeb/linux/kbd/scancodes.html Keyboard scancodes] - A complete reference on all scancodes you might encounter.
*[https://web.archive.org/web/20030621203107/http://www.microsoft.com/whdc/hwdev/tech/input/Scancode.mspx USB HID to PS/2 Translation Table] - Microsoft's table of scancodes and USB equivalents
*[https://web.archive.org/web/20190301075756/http://download.microsoft.com/download/1/6/1/161ba512-40e2-4cc9-843a-923143f3456c/scancode.doc Keyboard Scan Code Specification] - Microsoft's specification for scancodes
*[https://www.youtube.com/playlist?list=PLUZozxlhse-NUto5JeJ0EDXEUFloWBdAj PS/2 keyboard interface playlist] - A video playlist of building a PS/2 keyboard interface on bread boards, by Ben Eater.

==== Implementations ====
* [http://lxr.linux.no/#linux+v3.5.4/drivers/input/keyboard/atkbd.c Linux] (C,GPL)
* [https://github.com/Stichting-MINIX-Research-Foundation/minix/blob/master/minix/drivers/hid/pckbd/pckbd.c Minix]

[[Category:Human Interface Device]]
[[Category:Common Devices]]
[[de:Keyboard Controller]]
````

<a id="w001901"></a>

## W001901 — Direct Mapping

Bron-ID: W001901
Namespace: Main (0)
Revisie: 30754 | Datum: 2026-06-15T09:50:02Z
Bron: https://wiki.osdev.org/Direct_Mapping
Revisie-URL: https://wiki.osdev.org/index.php?title=Direct_Mapping&oldid=30754

````text
Direct Mapping is a special mapping used with [[Paging]], where some or all of the virtual memory is mapped with a fixed offset to the physical memory, creating a window into it. The direct mapping provides a fast and easy way to access physical memory (since the CPU can't usually directly do so after Paging is enabled), and as such is oftentimes very useful or necessary during the initialization of the paging, and might still be a handy mechanism to access physical memory later (with the alternatives being Recursive Mapping, or creating temporary mappings as needed).

Identity Mapping is a common form of direct mapping, where the virtual addresses are mapped to their same physical pages (with offset 0). Otherwise, due to the problems of Identity Mapping, many operating systems (including Linux) choose to place and use the direct map in their address space once they are initialized, which is usually the higher half, creating that way what it commonly referred to (by many bootloaders, including limine) as Higher Half Direct Map (HHDM), or just the Direct Map (referring to a mapping in the kernel's address range).

== Identity Mapping ==

Identity Paging, Identity Mapped Paging and 1:1 Paging are terms often used on the forum to describe a design choice where a portion of virtual addresses are mapped to physical addresses that have the same value. This means that if paging is enabled with identity paging, <tt>0xb8000</tt> '''is''' <tt>0xb8000</tt>, as long as that area is identity mapped.

Because of their disadvantages, identity maps are generally not recommended for typical designs (for [[Multitasking Systems]]), and are usually not kept by kernels after finishing the initialization (except for some special cases).

===Advantages===
When enabling the paging, the 1:1 mapping region doesn't care of whether paging is enabled or disabled, since the addresses stay the same between the two. Mapping the necessary memory regions, or even all of the memory (first few megabytes/gigabytes for it, for example all 4GB on x86 before enabling long mode) allows to enable the paging without headaches, and treat the physical memory addresses given by the bootloader ([[Multiboot]] is the first thing that comes to mind) or the firmware as pointers.

=== Disadvantages ===
The main disadvantage is that the identity map location clashes with the typical address ranges used by userspace (with [[Higher Half Kernel]] design), leading to all the disadvantages implied by that (the need to have relocatable binaries, difficulties with programming, and so on). In addition, all of the general disadvantages of having a direct map still apply here (notably, the limited address space on 32-bit systems).

===Example===
Let's say you decide to use Identity Paging in the lowest 1MB. In this case vaddr <tt>00000000</tt>..<tt>00000fff</tt> are mapped to frame #00000, vaddr <tt>00001000</tt>..<tt>00001fff</tt> are mapped to frame #00001, and so on. (vaddr <tt>000ff000</tt>..<tt>000fffff</tt> are mapped to frame #000ff)

You can easily do this with a loop filling the page table:
<source lang="c">
 void idpaging(uint32_t *first_pte, vaddr from, int size) {
    from = from & 0xfffff000; // discard bits we don't want
    for(; size>0; from += 4096, size -= 4096, first_pte++){
       *first_pte = from | 1;     // mark page present.
    }
 }
</source>

== Higher Half Direct Map ==
Higher Half Direct Map (or Mapping), or simply a Direct Mapping is a mapping of some or all of the memory, but in the kernel address space in [[Higher Half Kernel|higher half kernels]]. With it, the kernel can keep accessing physical memory similarly to having an identity map (but by adding the offset to the beginning of the mapping to get the virtual address, instead of treating the physical addresses as pointers to virtual memory).

Unlike the identity map, having it in the kernel address space makes it not collide with userspace (removing the biggest disadvantage), while all the benefits of it still hold. Because of that, the higher half direct maps are used by many operating systems (especially to access things like [[Page Tables]], or to save on having to map in kernel heap on each allocation). Some newer/more ergonomic [[Bootloaders|boot protocols]], such as [[Limine]] may even set it up for the kernel, saving the need to use a different mechanism to access physical memory during boot.

== Advantages of direct maps ==
Direct maps (in higher half, but also identity maps) provide several big advantages, which make them popular and recommended for 64 bit kernels.

=== Fast memory access ===
Since the mappings cover large contiguous memory regions, it is trivial to use huge pages when creating them (for example, 2MB/4MB/1GB pages on x86). In addition, some architectures (for example, LoongArch) even provide direct maps on the architecture level, bypassing page tables all together. This reduces the [[TLB]] misses, and usually makes this the fastest way to access physical memory (than having to create temporary mappings all the time, or walking the page tables through recursive mappings, unless specifically designed around).

In addition to having less TLB misses, the address arithmetic is also easier and faster, since getting the address is just a matter of adding an offset (in case all the RAM is accessible through it), and getting a physical address from the pointer in a direct map is also easier.

=== Faster memory allocation ===
If the mappings are always present, the kernel might choose to place some parts or all of the heap into its direct map, saving the need to explicitly map and unmap memory on allocation and deallocation. This is especially helpful if the [[Page Frame Allocation|page frame allocator]]/memory manager can quickly allocate more than one page, since that way most of the allocations can go to direct map (unless the physical memory is very fragmented, in which case the kernel may choose to fall back to allocating a virtual memory region outside direct map, and map the pages into there). 

== Problems with direct mapping ==
=== Address space consumption ===
[[File:Windows9xRunningOutVirtualMemory.jpg|thumb|The result of trying to map 16GB physical memory into a 2GB kernel virtual address space.]]
The direct mappings consume memory, in the form of physical memory (relatively small amount of it), and virtual address space. While the address space is very large on 64 bit systems, which makes it a non-issue on them, the 32 bit systems (especially x86 and other later 32 bit architectures, where the system might have more than 4GB of physical memory) are a different story. For example, if the CPU can only address 4 GB of virtual memory, even if the system only has 2GB of RAM, mapping all of that at once would eat a significant chink of virtual address space of userspace. This has lead to some creative (or dubious, opinions differ) solutions, such as sliding physical memory windows in Linux.

Older DOS-based versions of Windows, including 3.11 and 9x, did not address this problem. They therefore will fail with the rather ironic "Insufficient memory to initialize Windows" message on modern x86-64 machines, or indeed any machine with gigabytes of RAM. The kernel tries to map the entirety of physical memory into the higher half, and runs out of ''virtual'' memory'''.'''

=== Mapping too much on x86 ===
While it might be convenient and tempting to just map all of the memory (both RAM and MMIO), in reality it is problematic. Notably, x86 does not allow having the same memory be mapped with different cache bits to different virtual addresses. Mapping more than RAM creates precisely this situation, and even when such mappings are never accessed by the kernel, the processor might still access them through different speculative execution problems. This manifests in receiving unexpected MCEs in the best case, and unexplainable crashes or hangs in the worst. 

=== Security concerns ===
While this is not an issue in itself (if someone ends up accessing the kernel memory, there are bigger issues than this), access to the kernel's memory immediately gives the attacker access to all the memory in the system. To combat this, advanced kernels choose to use various mechanisms, such as kernel address space isolation. However, this is not a very big risk (something has to break before someone can read the kernel memory) and probably not a realistic scenario for the majority of operating systems. 
==See Also==
===Articles===
* [[Memory Management]]
* [[Paging]]

[[Category:Memory management]]
````

<a id="w001902"></a>

## W001902 — UltraSPARC

Bron-ID: W001902
Namespace: Main (0)
Revisie: 30665 | Datum: 2026-05-28T15:47:13Z
Bron: https://wiki.osdev.org/UltraSPARC
Revisie-URL: https://wiki.osdev.org/index.php?title=UltraSPARC&oldid=30665

````text
The UltraSPARC architecture is a 64-bit extension of the SPARC (Scalable Processor Architecture) architecture. The processor is used in many machines made by Sun, and is a RISC processor. It contains a IEEE 1275 compliant PROM (a.k.a ~OpenBoot PROM) (with a FORTH interpreter).

The latest Sun machine running a UltraSPARC processor is the UltraSPARC IV, released in 2004 (UltraSPARC V and VI were canceled earlier this year). A UltraSPARC processor called 'Rock', with multiple cores on one chip, is planned for 2006. The current revision of the SPARC arch is version 9.

==Registers==

All registers are 64 bits. There are any number of registers, from 40 to 520 registers, but only 32 registers are available at one time, due to 'registers windows'. A SPARC processor can have 2 to 32 windows (but 7 is the most common number). At any given time however, only one register window can be in use.

There are 8 temporary registers, labeled L0 to L7 (L = local here), meant for temporary data in a function.
There are 8 global registers for global data (G0 to G8). However, the G0 register is special because it is always zero. That means a 'nop' can be performed by setting the g0 register (and this is how it's done as a meta-instruction in many UltraSPARC assemblers).

==Boot Sequence==
The machine boots into a Open Firmware boot ROM (which can be interactive), and the ROM searches for a valid boot device. If a valid bootsector is found, it loads 512 bytes into the memory location 0x4000 (according to the FreeBSD source code), and passes the Open Firmware function call/base address in the register O4.
The bootloader must then access the PROM to find out which drive it was booted from, and load the appropriate sectors into memory. The UltraSPARC Linux loader, SILO, loads the kernel to 0x4000 (after loading a second stage somewhere else).

==Printing to the console==

Unlike the [[x86]] architecture, printing to screen is not done by poking a word into memory, instead, all input/output requests must go through the PROM, which has a special calling convention.

==See Also==
===External Links===
* [http://www.academic.marist.edu/~~jzbv/architecture/Projects/S2002/SPARC/intro.html SPARC Architecture Overview]
* [http://www.sparc.com/standards/SPARCV9.pdf SPARC V9 Architecture Standard]
* [http://en.wikipedia.org/wiki/SPARC Wikipedia definition]

[[Category:Sparc]]
````

<a id="w001903"></a>

## W001903 — Time And Date

Bron-ID: W001903
Namespace: Main (0)
Revisie: 30770 | Datum: 2026-06-17T10:09:27Z
Bron: https://wiki.osdev.org/Time_And_Date
Revisie-URL: https://wiki.osdev.org/index.php?title=Time_And_Date&oldid=30770

````text
Users, the filesystem, the scheduler, system applications, and some user applications will all need to know the date and/or time to various accuracies.
Some applications will need to be started, sent signals, or sent messages when the clock strikes a certain time.
To provide these services, the OS needs to obtain the current time during boot and maintain this time.

There are three important subdivisions of "time" in a system: human time, filesystem time, and scheduler timeslice "ticks". Human
time can be measured in seconds (or even days, for longer times), where "ticks" must always be .01 seconds or smaller.
An important design decision is whether to keep these time formats separate, or whether to merge them together.

== Maintaining The Time Internally ==
{{Main|Timer Interrupt Sources}}

During operation, a typical OS will use a local hardware timer to drive its own internal time keeping code -- often called the "system clock".
For example, x86 systems (and other architectures) have several different internal [[Timer Interrupt Sources]], depending on the system, that may produce an interrupt (or may even be polled) at regular intervals to inform the OS of the passage of time.
Using at least one of these timers is probably necessary to control and generate scheduler timeslice ticks.
It takes very little additional code in the driver to also use that timer to update the system clock. The OS may only need to
know how often the chosen timer ticks during one second, and keep a counter -- or the system clock may be designed to tick at
the same speed as the chosen timer.

However, some of these timers are only accurate to within a handful of seconds a day, so it may be desirable to benchmark the system
clock to a reference, periodically, even while the system is running.

It is also possible not to use a timer to update the system clock at all, and only use an "accurate" external reference when the system tries to
access the system clock. Depending on the reference used, this method may only be accurate to 1 second, it may be slow, it may be needed often
(for file system accesses, especially), and the format used often isn't what the OS wants.

== Obtaining The Initial Time And Date ==

When the computer is turned off, obviously system software cannot keep an internal clock updated using a timer. So when the computer is rebooted,
the OS needs to use some other method for obtaining an initial time and date. There are really only two automated methods. If those methods fail,
the only fallback method that exists is to ask a user to enter the date/time.

=== Battery-Backed Clock ===

For x86 PCs, there's a special "Real Time Clock" (RTC), which is "combined" with the system's [[CMOS]]. It has its own battery
so that it keeps running when the computer is turned off and the contents of its memory are not lost. See the [[CMOS]] article for
information on reading the time and date from the RTC.

Almost any other system besides x86 will also have some kind of battery-backed Date/Time clock.

The downside to this is that the battery will always eventually die, and may not get replaced. It is a good idea to check the results
of reading any battery-backed clock for "sane" values.

=== Network Time ===
Getting some kind of network time is a superior method for achieving accuracy and consistency across machines. The downside is that the end-user's computer may not be (currently) connected to a network, or the server may go down.

==== Internet ====
[http://ntp.org/ NTP] is a good protocol with a very large [https://www.ntppool.org/en/ pool] of free servers which may be automatically selected by DNS. It uses UDP on port 123. ([http://en.wikipedia.org/wiki/Network_Time_Protocol Wikipedia article])

The [https://en.wikipedia.org/wiki/Time_Protocol TIME protocol] is extremely simple: attach a TCP socket to port 37 on a [http://tf.nist.gov/tf-cgi/servers.cgi NIST server], read a 32bit bigendian value (seconds since midnight, Jan 1, 1900, UTC), and close the socket. This has caveats: it's not as accurate as NTP; the value will roll over in 2036; and as stated on the server page, using the complete machinery of TCP to retrieve only 32 bits is not kind to the server's bandwidth. The Unix rdate command uses the TIME protocol.

=== LAN ===
The official NTP [http://ntp.org/downloads.html software] can easily be configured to serve NTP on a LAN (or indeed the Internet).

It's very easy to write a TIME server, (it's as easy as a server can be,) and using TCP to retrieve 32 bits often doesn't matter on a LAN.

It's just as easy to write your own protocol for LAN use. You can make resolution and range better than NIST, but it may be hard to equal NTP's accuracy. There is at least one professional precedent: Diskless Plan 9 computers get the time from their file server.

== Internal OS Time Formats ==

Choosing a good time format, and writing the code to support this format can be more complex than it seems at first. As said above, the humans,
the filesystem, and the scheduler all have somewhat different needs when it comes to time. They operate on different timescales. You can create
a separate format for each, or, it can make more sense for the OS to maintain a single universal time format with higher accuracy.
All possible choices have downsides, either in complexity or in wasted computer resources. It is important to note that '''there is no standard yet''',
and there may be no best choice to make.

=== Human Timescales ===

In the short term, humans are comfortable dealing with seconds. A time format that is specified to times shorter than seconds is partially wasted
on humans. In fact, presenting users with ''too much'' accuracy can confuse them and decrease their productivity.

On the other hand, once a file (for example) is more than a year old, a user will no longer care about what second it was created. So in the longer
term, users are going to be much more interested in a time specified in days.

This may argue for a system time that is kept in seconds -- or perhaps a more flexible format that is initially in seconds and then switches to something
like [[Julian Day Number]] for longer periods of time. Or for a system that is specified with shorter intervals than seconds, but that only displays
a limited amount of the actual time information available.

=== Filesystem Timestamps ===

Almost every filesystem uses a predefined time format. Once again, there is no standard. If your OS only supports one filesystem, it may be smart
to match your OS time format to the one that the filesystem uses, so that you never need to do conversions. Many filesystems use time formats that
are specified in seconds, which can cause "less than perfect" results for utilities such as "make" -- if the utility is badly designed. When you
design an OS, you will need to make a decision about whether to coddle the bad design decisions of other people, to help make their software work.

=== Scheduler Timeslices ===

On a multitasking OS, the length of time that each thread is allowed to run is often based on a little piece of time called a "timeslice".
This is an '''extremely''' important function of an OS, so it is quite important to have some kind of counter available that measures time
on this timescale. One major obstacle to having a universal time format for your OS is if your timeslices are variable-length, because this
may make it very difficult to establish the minimum length of time that your universal time format needs to represent.

=== Historical Dates ===

Keep in mind that Accounting packages, Databases, and other programs may need to store dates from the past century, such as birthdates. If you
try hard to imagine such a thing, it's even possible to imagine wanting to store actual historical dates from many centuries ago. So, once again,
you may need to make a less-than-optimal choice for your time format, in order to support this feature.

=== Example Time Formats ===

The *NIX time format keeps track of 32bit seconds since the start of year 1970. This value becomes obsolete around the year 2100, and cannot store
some historical dates.

The Windows time format uses a 64-bit unsigned value of the number of 100-nanosecond (100 x 10<sup>-9</sup> second) intervals since January 1, 1601.
This value becomes obsolete in about 50 thousand years. Why 1601? It can be considered the start of a 400-year cycle of leap years, which makes conversion
into a date simpler. It is also very close to the actual beginning of the use of the Gregorian Calendar (the one most people use).
The slight drawback is that you need 8 bytes of storage, everywhere you want to store a date. Which is a lot of places. Which you could use for
storing things such as file version numbers, instead.

BCOS uses something very similar, except using signed milliseconds from Y2K, rather than unsigned 100 nanosecond intervals from 1601.

=== "Scientific" Format ===

There is in fact a measurement of time that is used in Astronomy, called [[Julian Day Number]]. For reasons regarding historical calendar systems,
it starts measuring time from noon, UTC, January 1, 4713 BC, and it measures time in days with a floating point value. At the very least, you can
consider it to be a standard, in an area that is very lacking in standards. Because it is a standard, there is code available for converting
JDN's to calendar dates. In any case, once you have divided out the seconds and fractions of seconds from any time format, you are left with "days".
Which means that the difference between your number and the Julian Day Number is merely an offset. Of course, it is possible to truncate JDNs, and
use them as integers. See the [[Julian Day Number]] article for some date calculation code examples.


== Which Time? ==
Once you've determined what format to use for keeping track of time, it's important to decide '''which''' time you'll keep track of. In general there are
three different times: the user's "wall clock" time, local standard time and UTC ("Universal Co-ordinated Time").

At any instant, UTC is the same everywhere around the world. Local standard time depends on which time zone you're in (for example, my local standard time
is always UTC + 9.5 hours). "Wall clock time" is the same as local standard time unless daylight savings is in effect (for example, my wall clock time is
UTC + 9.5 hours except during summer where it becomes UTC + 10.5 hours).

OSes that were created before the internet existed were built assuming that the user would set the computer's battery-backed clock to wall-clock time,
so that the computer could easily function as a clock for the computer's owner. The OS assumed that it may need to take responsibility for adjusting
the computer's actual battery-backed clock for Daylight Savings Time. Old versions of Windows do this.
This can cause problems when the computer dual-boots two OSes that both expect to adjust the battery-backed clock for daylight savings time
(so that it's accidentally changed by 2 hours instead of one). See http://www.cl.cam.ac.uk/~mgk25/mswish/ut-rtc.html for a
detailed discussion of this problem.

With distributed computing, however, comes the need to synchonize multiple computer clocks. The obvious way to do that is to use UTC for setting the clocks
on all systems. The OS has the responsibility of converting UTC to wall clock time (including all the complexity of Daylight Savings) before displaying
the time to the user. This is what Linux does. If you have a system that dual-boots both types of OSes, and both of them try to read the battery-backed
clock, then obviously you will have an unfixable conflict between them.

Some solutions:
* do not use the battery-backed clock
* make using the battery-backed clock optional
* allow the superuser to set a flag for using the battery-backed clock either way

The '''stupid''' thing to do is to force the clock to be only one way or the other. As an indication of how unworkable this is, even Windows can use the clock either way. [https://www.howtogeek.com/323390/how-to-fix-windows-and-linux-showing-different-times-when-dual-booting/ (But, of course, it doesn't support its non-default very well.)]

== Complexities ==

Regardless of what you do you will eventually need to convert the time format that your OS uses into other time formats. Unfortunately time itself is not a simple thing, and conversion between time formats can be quite complex (especially if done accurately). Also there are some problems maintaining the OSs time that don't involve conversions. The things you may need to watch out for include:

;Time Zones: This is mostly for converting between local standard time and UTC. Most OSs have a database so that each time zone can be given a name or location (for example, "Adelaide, South Australia" rather than "UTC +9.5 hours"). A map of timezones and their locations can be found here: [http://upload.wikimedia.org/wikipedia/commons/e/e7/Timezones2008.png] (1.23MB)
;Daylight Savings: This is a nightmare. Some countries have different time zone rules for each state, some countries have different time zone rules for each local area within a state (the USA is particularly messed up). Worse still is that some areas decide what they are going to do each year so that it's impossible to work it out in advance, and for most areas the daylight savings rules are subject to the whims of politicians. For some OSs daylight savings information is also kept within the same database as time zone information, so that a user can tell the OS where they are and the OS can figure out the appropriate time zone and daylight savings rules from this. Daylight savings can be especially problematic for dates in the past.
;Leap Years: As you all know, a year isn't exactly 365 days. The Gregorian leap year rules are that if a year is a multiple of 4 then it's a leap year, unless it happens to be a new century and it can't be divided by 400. For example, the years 2004, 1996 and 1968 are leap years, the years 1700, 1800, and 1900 are not leap years, but years 2000, 1600 and 2400 are. This keeps the date in synchronization with the seasons.
;Leap Seconds: Due to standards bodies, atomic clocks, and gravitational influences, a day is not exactly 86400 seconds long (and on average each day is slightly longer than the last). To account for this an extra leap second is added (roughly one second each 5 years). A list of when leap seconds have been added can be found at http://tf.nist.gov/pubs/bulletin/leapsecond.htm. This is beyond the timekeeping accuracy of any computer clock, but may be an issue if you want to create very accurate answers when subtracting one time from another in your format.
;Calendars: Most of the world uses the Gregorian calendar, but some people don't and some use other calendars in conjunction with the Gregorian calendar. If you intend to make your OS international, or if you convert your time format into Gregorian dates before 1920 then you may want to research other calendars and (for past dates) the history of calendars. An excellent starting point for this can be found at [http://www.tondering.dk/claus/calendar.html].
;Fixing Drift: Any timer in a system may run slow or fast, and that may be detectable -- either by the user or the OS. It may be desirable to add or subtract a small extra amount of time on each timer tick.
;Accuracy
: Unfortunately, the electronics in PCs isn't as accurate as it could be, and over time (regardless of everything else) the computer's time will become inaccurate. Some OSs ignore this problem and allow the user to change or adjust the time whenever they like. This causes problems with some utilities (if you've ever got a "modification time in the future" error message from 'make' you'll know why). For other OSs (often the OSs that are designed for servers that are never turned off) there's a way of adjusting the time in a more subtle way, with many tiny changes rather than a larger sudden change. An example of this is the 'adjtimex' utility on *nix systems. However, enforcing such subtle changes may make life difficult for users if the internal clock is far wrong. I (eekee) once had a situation where the system clock was off by 8 years, and ntpd was patched to ignore its option to allow sudden changes. ntpd made tiny changes effectively accelerating the time to about 3 times normal speed, so the computer's clock wouldn't have been correct for over 2.5 years! If an OS doesn't set its idea of the time from an authoritative external source on boot, (and sometimes you can't because there's no network,) then it must either query the user on boot, or allow the user to adjust the time whenever they like. And some users will get very annoyed with having to manually enter the time every boot.

== See Also ==
=== Articles ===
* [[Timer Interrupt Sources]]
* [[Programmable Interval Timer]]
* [[CMOS]]
* [[RTC]]
* [[APIC]]
* [[Julian Day Number]]

=== Threads ===
*[[Topic:11485|Time: How do you store time in your OS?]]

=== External Links ===

[[Category:Time]]
````

<a id="w001904"></a>

## W001904 — VESA Video Modes

Bron-ID: W001904
Namespace: Main (0)
Revisie: 30513 | Datum: 2026-04-26T04:10:17Z
Bron: https://wiki.osdev.org/VESA_Video_Modes
Revisie-URL: https://wiki.osdev.org/index.php?title=VESA_Video_Modes&oldid=30513

````text
Under legacy BIOS (and CSM-capable UEFI systems in legacy BIOS mode) you can access the video card functions using the standardized VESA/VBE functions.

In UEFI mode, instead, one uses the [[GOP]] (Graphics Output Protocol) to access the video card in a standardized way.

== VESA Functions ==
You'll want to look in the VESA VBE docs for these functions:
All VESA functions return 0x4F in AL if they are supported and use AH as a status flag, with 0x00 being success. This means that you should check that AX is 0x004F after each VESA call to see if it succeeded.

;INT 0x10, AX=0x4F00
: Get Controller Info.  This is the one that returns the array of all supported video modes.
<syntaxhighlight lang="c">
struct VbeInfoBlock {
   char     VbeSignature[4];         // == "VESA"
   uint16_t VbeVersion;              // == 0x0300 for VBE 3.0
   uint16_t OemStringPtr[2];         // isa vbeFarPtr
   uint8_t  Capabilities[4];
   uint16_t VideoModePtr[2];         // isa vbeFarPtr
   uint16_t TotalMemory;             // as # of 64KB blocks
   uint8_t  Reserved[492];
} __attribute__((packed));

VbeInfoBlock *vib = dos_alloc(512);
v86_bios(0x10, {ax:0x4f00, es:SEG(vib), di:OFF(vib)}, &out);
if (out.ax!=0x004f) die("Something wrong with VBE get info");
</syntaxhighlight>

In assembly ([[NASM]]):
<syntaxhighlight lang="nasm">
struc VesaInfoBlock				;	VesaInfoBlock_size = 512 bytes
	.Signature		resb 4		;	must be 'VESA'
	.Version		resw 1
	.OEMNamePtr		resd 1
	.Capabilities		resd 1

	.VideoModesOffset	resw 1
	.VideoModesSegment	resw 1

	.CountOf64KBlocks	resw 1
	.OEMSoftwareRevision	resw 1
	.OEMVendorNamePtr	resd 1
	.OEMProductNamePtr	resd 1
	.OEMProductRevisionPtr	resd 1
	.Reserved		resb 222
	.OEMData		resb 256
endstruc

Main:
	push ds
	pop es
	mov di, VesaInfoBlockBuffer
	call get_vesa_info
	jmp $

;	in:
;		es:di - 512-byte buffer
;	out:
;		cf - set on error
get_vesa_info:
	clc
	mov ax, 0x4f00
	int 0x10
	cmp ax, 0x004f
	jne .failed
	ret
	.failed:
		stc
		ret

ALIGN(4)

	VesaInfoBlockBuffer: istruc VesaInfoBlock
		at VesaInfoBlock.Signature,				db "VESA"
		times 508 db 0
	iend
</syntaxhighlight>

'''INT 0x10, AX=0x4F01, CX=mode, ES:DI=256 byte buffer'''
: Get Mode Info.  Call this for each member of the mode array to find out the details of that mode. The 256 byte buffer will be filled by the mode info block.

<syntaxhighlight lang="c">
struct vbe_mode_info_structure {
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
} __attribute__ ((packed));
</syntaxhighlight>

In assembly [[NASM]]:
<syntaxhighlight lang="nasm">
struc VesaModeInfoBlock				;	VesaModeInfoBlock_size = 256 bytes
	.ModeAttributes		resw 1
	.FirstWindowAttributes	resb 1
	.SecondWindowAttributes	resb 1
	.WindowGranularity	resw 1		;	in KB
	.WindowSize		resw 1		;	in KB
	.FirstWindowSegment	resw 1		;	0 if not supported
	.SecondWindowSegment	resw 1		;	0 if not supported
	.WindowFunctionPtr	resd 1
	.BytesPerScanLine	resw 1

	;	Added in Revision 1.2
	.Width			resw 1		;	in pixels(graphics)/columns(text)
	.Height			resw 1		;	in pixels(graphics)/columns(text)
	.CharWidth		resb 1		;	in pixels
	.CharHeight		resb 1		;	in pixels
	.PlanesCount		resb 1
	.BitsPerPixel		resb 1
	.BanksCount		resb 1
	.MemoryModel		resb 1		;	http://www.ctyme.com/intr/rb-0274.htm#Table82
	.BankSize		resb 1		;	in KB
	.ImagePagesCount	resb 1		;	count - 1
	.Reserved1		resb 1		;	equals 0 in Revision 1.0-2.0, 1 in 3.0

	.RedMaskSize		resb 1
	.RedFieldPosition	resb 1
	.GreenMaskSize		resb 1
	.GreenFieldPosition	resb 1
	.BlueMaskSize		resb 1
	.BlueFieldPosition	resb 1
	.ReservedMaskSize	resb 1
	.ReservedMaskPosition	resb 1
	.DirectColorModeInfo	resb 1

	;	Added in Revision 2.0
	.LFBAddress		resd 1
	.OffscreenMemoryOffset	resd 1
	.OffscreenMemorySize	resw 1		;	in KB
	.Reserved2		resb 206	;	available in Revision 3.0, but useless for now
endstruc

Main:
	; after getting VesaInfoBlock:
	push word [VesaInfoBlockBuffer + VesaInfoBlock.VideoModesSegment]
	pop es
	mov di, VesaModeInfoBlockBuffer
	mov bx, [VesaInfoBlockBuffer + VesaInfoBlock.VideoModesOffset]	;	get video modes list address
	mov cx, [bx]							;	get first video mode number
	cmp cx, 0xffff							;	vesa modes list empty
	je .NoModes
	call get_vesa_mode_info
.NoModes:
	jmp $


;	in:
;		cx - VESA mode number
;		es:di - 256-byte buffer
;	out:
;		cf - set on error
get_vesa_mode_info:
	clc
	mov ax, 0x4f01
	int 0x10
	cmp ax, 0x004f
	jne .failed
	ret
	.failed:
		stc
		ret

ALIGN(4)

VesaModeInfoBlockBuffer:	istruc VesaModeInfoBlock
		times VesaModeInfoBlock_size db 0
	iend
</syntaxhighlight>

'''INT 0x10, AX=0x4F02, BX=mode, ES:DI=CRTCInfoBlock'''
:Set Video Mode.  Call this with the mode number you decide to use.  If you choose a mode that makes use of a linear framebuffer, you should OR the mode number with 0x4000. This sets the "Use LFB" bit in the mode number. Set the bit 11 of BX to instruct the BIOS to use the passed CRTCInfoBlock structure, see the specification for more information.

:[[BIOS|BIOSes]] can switch to protected mode to implement this, and might reset the [[GDT]]. This is observable on [[QEMU]] 2.2.x.

== Will it work with Bochs? ==

For VBE to work in Bochs you need the "VGABIOS-lgpl" BIOS and have a version of Bochs that was compiled with the <tt>--enable-vbe</tt> option... See [[Topic:9655|Vesa Information in Bochs]] thread for more info. Also read about [[Bochs Graphics Adaptor]].

==How to pick the mode I wish?==

VESA stopped assigning codes for video modes long ago -- instead they standardized a much better solution: you can query the video card for what modes it supports, and query it about the attributes of each mode. In your OS, you can  have a function that you call with a desired width, height, and depth, and it returns the video mode number for it (or the closest match). Then, just set that mode

Here's a sample code, assuming you have a VirtualMonitor already ... Basically, you will scan the 'modes list' referenced by the VbeInfoBlock.videomodes[] and then call 'get mode info' for each mode. You can then compare width, height and colordepth of each mode with the desired one.

<syntaxhighlight lang="c">
uint16_t findMode(int x, int y, int d)
{
  struct VbeInfoBlock *ctrl = (VbeInfoBlock *)0x2000;
  struct ModeInfoBlock *inf = (ModeInfoBlock *)0x3000;
  uint16_t *modes;
  int i;
  uint16_t best = 0x13;
  int pixdiff, bestpixdiff = DIFF(320 * 200, x * y);
  int depthdiff, bestdepthdiff = 8 >= d ? 8 - d : (d - 8) * 2;

  strncpy(ctrl->VbeSignature, "VBE2", 4);
  intV86(0x10, "ax,es:di", 0x4F00, 0, ctrl); // Get Controller Info
  if ( (uint16_t)v86.tss.eax != 0x004F ) return best;

  modes = (uint16_t*)REALPTR(ctrl->VideoModePtr);
  for ( i = 0 ; modes[i] != 0xFFFF ; ++i ) {
      intV86(0x10, "ax,cx,es:di", 0x4F01, modes[i], 0, inf); // Get Mode Info

      if ( (uint16_t)v86.tss.eax != 0x004F ) continue;

      // Check if this is a graphics mode with linear frame buffer support
      if ( (inf->attributes & 0x80) != 0x80 ) continue;

      // Check if this is a packed pixel or direct color mode
      if ( inf->memory_model != 4 && inf->memory_model != 6 ) continue;

      // Check if this is exactly the mode we're looking for
      if ( x == inf->XResolution && y == inf->YResolution &&
          d == inf->BitsPerPixel ) return modes[i];

      // Otherwise, compare to the closest match so far, remember if best
      pixdiff = DIFF(inf->Xres * inf->Yres, x * y);
      depthdiff = (inf->bpp >= d)? inf->bpp - d : (d - inf->bpp) * 2;
      if ( bestpixdiff > pixdiff ||
          (bestpixdiff == pixdiff && bestdepthdiff > depthdiff) ) {
        best = modes[i];
        bestpixdiff = pixdiff;
        bestdepthdiff = depthdiff;
      }
  }
  if ( x == 640 && y == 480 && d == 1 ) return 0x11;
  return best;
}
</syntaxhighlight>

== Common Mistakes ==

There's some mistakes that beginners seem to make fairly often when they first start working with VBE.

===VESA Defined Mode Numbers===

For older versions of the VBE specification (VBE 1.0, VBE 1.1 and VBE 1.2) VESA defined some standard mode numbers. For example, mode 0x0113 was 800 * 600 * 15-bpp. In VBE 2.0 these mode numbers were deprecated - VESA decided not to define any more of them, told video card manufacturers they don't need to use the old standard mode numbers, and told programmers to search for the mode they want without relying on any of the old standard mode numbers. Despite this, it's still possible to find obsolete information on the internet suggesting to use these old standard mode numbers.

===Use "Bytes Between Lines"===

People tend to assume that pixel data is contiguous (e.g. line 1, then line 2, then line 3, ...). This isn't always the case, and for a variety of reasons there may be padding between lines (e.g. line 1, padding, then line 2, padding, then line 3, ...). The "Get Mode Information" VBE function returns the number of bytes between lines, and this value should be used. For example, for a 640 * 480 * 16-bpp video mode, the offset of a line is found by "offset = line * bytes_per_line" and not by "offset = line * (640 * 2)". In the same way, you can't use one "rep stosb/w/d/q" or "memset()" to fill the entire display, and you should fill each line separately to avoid writing to any padding.

Note that (for VBE 3.0) there may be 2 "bytes between lines" values returned by the "Get Mode Information" VBE function. The first one is used when the video mode is setup for bank switching and the second one is used when linear frame buffer is being used. For older versions of VBE there is no "bytes between lines for linear frame buffer" value, and the "bytes between lines for bank switched" value is used for both bank switching and linear frame buffer.

===Don't Assume Pixel Formats===

Usually pixel data is in "RGB" format, but this isn't always the case. A pixel can be in "BGR" format or anything else, and may not even use Red, Green and Blue components - it could use "YUV" where there's one luma/brightness component (Y) and two chrominance (UV) components. Always check the "memory_model" field and (for 15-bpp and higher video modes) all of the "component mask" and "field position" fields (e.g. "red_mask_size", "red_field_position", etc) in the structure returned by the "Get Mode Information" VBE function.

===Don't Assume Unused Bits Are Unused===

For some colour depths there's "unused" bits in the pixel data (one "unused" bit per pixel in 15-bpp modes, and 8 "unused" bits per pixel for 32-bpp modes). These unused bits may actually be used by some video cards for a variety of extra features. For one example, I know of a video card where (in 15-bpp modes) if the highest/unused bit is set then it uses the lowest 8 bits of the pixel data as an index into the palette.

===Don't Assume VGA Compatibility===

For 8-bpp video modes people assume that they can use the VGA I/O ports to change the palette. It's much better to use the VBE functions instead, but if you don't then at least test if the video mode uses VGA I/O ports by testing the "VGA compatible mode" flag (bit 5) in "mode_attributes" field returned by the "Get Mode Information" VBE function.

In a similar way, for 4-bpp "planar" video modes some people assume that the VGA I/O ports can be used to switch between planes rather than testing if the "VGA compatible mode" flag is set first.

===Don't Assume The Monitor Supports A Video Mode===

We all like pretty graphics and high resolution video modes. Unfortunately, if VBE says that a video mode is supported by the video card it does not mean that the video mode is also supported by the monitor. VESA has defined 2 video mode timings that are meant to be supported by all monitors (640 * 480 standard VGA timing and 720 * 480 standard VGA timing). For all other video modes you should either use [[EDID]] to find out if the monitor supports the video mode's timing (or not), or provide a way for the user to test the video mode and change it if it doesn't work. This is the approach used by Windows, with a dialog box appearing with the option to accept or revert. No action within 15 seconds reverts. Just an idea ;).

Using EDID for this purpose is complicated. Unless you provide a "CRTC information block" structure when you set the video mode you can't be entirely sure what timing the video card will use; and only some video cards that support VBE 3.0 support the "CRTC information" correctly.

The other alternative is to only ever use video modes that (should) use 640 * 480 and 720 * 480 standard VGA timing. For video modes with lower horizontal resolutions the video card sends the each pixel twice, and for video modes with lower vertical resolutions the video card sends the each row of pixels twice (called "double scanning"); and in both cases the video timing is the same as it would be for the corresponding higher resolution video mode. This gives the following list of "safe" resolutions:
* 720 * 480
* 640 * 480
* 360 * 480 (actually uses "720 * 480" timing)
* 320 * 480 (actually uses "640 * 480" timing)
* 720 * 240 (actually uses "720 * 480" timing)
* 640 * 240 (actually uses "640 * 480" timing)
* 360 * 240 (actually uses "720 * 480" timing)
* 320 * 240 (actually uses "640 * 480" timing)
Colour depth doesn't/shouldn't effect the video timing signals. This means that the best possible safe video mode would be 720 * 480 * 24/32-bpp.

===Don't Read From Video Memory===

Reading from the video memory is slooow! Use [[Double Buffering|double buffering]] instead.

==See Also==
===Articles===
* [[User:Omarrx024/VESA_Tutorial|VESA Tutorial]]
* [[VGA Hardware]] - VESA predecessor
* [[GOP|Graphic Output Protocol]] - VESA successor on [[UEFI]] machines

===Threads===
* [[Topic:9700|VESA, higher modes]] - Initial Thread, reply by Dreamsmith (aka DaidalosGuy)

=== External Links ===
* VBE at [https://en.wikipedia.org/wiki/VESA_BIOS_Extensions Wikipedia]
* [https://web.archive.org/web/20170930204932/http://www.delorie.com/djgpp/doc/ug/graphics/vesa.html Guide: VESA graphics modes]
* [http://www.petesqbsite.com/sections/tutorials/tuts/vbe3.pdf The VESA VBE 3.0 specification]
* [http://www.monstersoft.com/tutorial1/VESA_intro.html Introduction to VESA programming]

[[Category:Video]]
[[Category:FAQ]]
[[de:VESA_BIOS_Extensions]]
````

<a id="w001905"></a>

## W001905 — Partition Table

Bron-ID: W001905
Namespace: Main (0)
Revisie: 30519 | Datum: 2026-04-26T04:19:04Z
Bron: https://wiki.osdev.org/Partition_Table
Revisie-URL: https://wiki.osdev.org/index.php?title=Partition_Table&oldid=30519

````text
The following information about Partition Tables only applies to hard disk drives. Most other storage devices use other partitioning techniques, if they even use partitioning at all.

==MBR==
{{Main|MBR (x86)}}
The Master Boot Record is the traditional way of storing partition information about a hard disk, along with some boot code.
That is, the Partition Table is contained inside the [[MBR (x86)|MBR]], which is stored in the first sector
(cylinder 0, head 0, sector 1 -- or, alternately, [[LBA]] 0) of the hard drive.
(See the [[MBR (x86)|MBR article]] for the overall structure and contents of the MBR.)

Almost all PCs still use an MBR for booting hard disks, and for storing partition information on hard disks.
Traditional MBRs are nearly '''obsolete''' at this time, because the 32 bit design of the LBA fields in the Partition Table begins
to "overflow" when dealing with disks larger than 2Tb.
The successor to the MBR system is [[GPT]] (see below). There were other proposed new standards for
the MBR, but the GPT won the battle.

Information about primary partitions and an extended partition is contained in a 64-byte data structure located in the MBR.
This Partition Table conforms to a
standard layout that is independent of the operating system. Each Partition Table entry is 16 bytes long, making a maximum of four
entries available. Each entry starts at a predetermined offset from the beginning of the sector, as follows:

{| class="wikitable"
|-
! Partition number
! Offset
|-
|Partition 1
|0x01BE (446)
|-
|Partition 2
|0x01CE (462)
|-
|Partition 3
|0x01DE (478)
|-
|Partition 4
|0x01EE (494)
|}

Note: Naming the partition table entries as "1" through "4" is for convenience only.
The partition table entries are not required to be in any kind of order.


Each of the four Partition Table entries contains the following elements, in the following structure:

{| class="wikitable"
! Offset
! Size
! Description
|-
| 0x00
| 1 byte
| Boot indicator bit flag: 0 = no, 0x80 = bootable (or "active")
|-
| 0x01
| 1 byte
| Starting head
|-
| 0x02
| 6 bits
| Starting sector (Bits 6-7 are the upper two bits for the Starting Cylinder field.)
|-
| 0x03
| 10 bits
| Starting Cylinder
|-
| 0x04
| 1 byte
| System ID
|-
| 0x05
| 1 byte
| Ending Head
|-
| 0x06
| 6 bits
| Ending Sector (Bits 6-7 are the upper two bits for the ending cylinder field)
|-
| 0x07
| 10 bits
| Ending Cylinder
|-
| 0x08
| 4 bytes
| Relative Sector (to start of partition -- also equals the partition's starting LBA value)
|-
| 0x0C
| 4 bytes
| Total Sectors in partition
|}

Notes:
* Any one of the partitions may be "active" (ie. bootable).
* At most one partition should be active.
* The Partition Table entries are <b>not</b> aligned on 4 byte boundaries (if the MBR is itself loaded into memory on a 4 byte boundary).
* Therefore, neither are the two uint32_t LBA entry values -- so the LBA values cannot be copied directly into a register.
* All the entry values are encoded little-endian.  Take note of this if fetching the uint32_t LBA entry values two bytes at a time, even on little-endian systems.
* The Cylinder, Head, Sector fields (taken together) are only 3 bytes (24 bits) long.
* Sector values (in the CHS fields) of 0 are illegal.
* CHS fields "max out" on a drive that is approximately 8GB in size -- and are therefore useless on almost all current drives.
* For drives smaller than 8GB, the LBA fields and the CHS fields must "match" when the values are converted into the other format.
* For drives bigger than 8GB, generally the CHS fields are set to Cylinder = 1023, Head = 254 or 255, Sector = 63 -- which is considered an invalid setting.
* If a Partition Table entry is unused, then it should be set to all 0.
* A System ID byte value of 0 is the definitive indicator for an unused entry.
* Any other illegal value (CHS Sector = 0 or Total Sectors = 0) may also indicate an unused entry.

===The System ID byte===
The System ID byte is supposed to indicate what filesystem is contained on the partition (ie. Ext2, ReiserFS, FAT32, NTFS, ...).
There was never any standard created for the System ID byte -- which means that Microsoft went and tried to hog almost all of the possible values.
See [[#External Links|the links below]] for tables of values of the System ID byte, for filesystems that have been lucky enough to acquire their own value by common consensus.

If you create your own custom filesystem, then you can simply pick a System ID value for your filesystem that seems to be unused.
There is also an attempt to standardize the use of System ID value = 0x7f (by the Alt-OS gang), to cover all custom filesystems that follow the standard -- however, their effort seems to be losing steam.

==Extended Partitions==

Extended partitions are a way of adding more than 4 partitions to a partition table.

The partition table may have one and only one entry that has the SystemID 0x5 (or 0xF). This describes an extended partition.

An extended partition is one physical partition that is subpartitioned into "logical" partitions. So, the partition table entry describing it has a StartLBA and NumSectors that describe the space inside which any number of logical partitions may sit.

At the start of an extended partition is an extended partition table. This takes exactly the same form as a normal partition table, apart from most of the fields are unused. Only two of the partition entries are used - the first one describes the desired logical partition, and the second one is a link (much like a linked list) that points at another extended partition table. The size should officially include the logical partition that follows with it. The remaining two entries are not used and should be left zeroed out.

Note that the StartLBA fields in these extended partition table entries are relative to the start of the extended partition itself.

== GPT ==
{{Main|GPT}}
GPT is an updated Partition Table standard, that has been adopted as the recommended partition mechanism under [[UEFI]]. It does not contain the artificial 24 bit or 32 bit limitations of the MBR Partition Tables. It also contains enhancements to the concept of partition tables, in general, and is significantly more complex than the MBR scheme.

==See Also==
[[GPT]]

===External Links===
* [http://www.win.tue.nl/~aeb/partitions/partition_types-1.html System ID byte values ("Partition Types") for known filesystems]
* [http://www.microsoft.com/whdc/device/storage/GPT-on-x64.mspx Microsoft's GPT info]
* [http://www.osdever.net/documents/partitiontypes.php A list of System IDs/Partition Types from osdever]
* [http://www.osdever.net/documents/pdf/partitiontypes.pdf Another list of System IDs/Partition Types from osdever (pdf)]
* [http://en.wikipedia.org/wiki/Extended_partition The wikidpedia article on the Extended Boot Record (AKA Extended Partition) has lots of pictures and many more words]
* GPT at [http://en.wikipedia.org/wiki/GUID_Partition_Table Wikipedia]

[[Category:x86]]
[[Category:Storage]]
````

<a id="w001906"></a>

## W001906 — Modular Kernel

Bron-ID: W001906
Namespace: Main (0)
Revisie: 23607 | Datum: 2019-05-07T17:18:28Z
Bron: https://wiki.osdev.org/Modular_Kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Modular_Kernel&oldid=23607

````text
{{Template:Kernel designs}}

A modular kernel is an attempt to merge the good points of kernel-level drivers and third-party drivers. In a modular kernel, some parts of the system core will be located in independent files called ''modules'' that can be added to the system at run time. Depending on the content of those modules, the goal can vary such as:
* only loading drivers if a device is actually found
* only load a filesystem if it gets actually requested
* only load the code for a specific (scheduling/security/whatever) policy when it should be evaluated
* etc.

The basic goal remains however the same: keep what is loaded at boot-time minimal while still allowing the kernel to perform more complex functions. The basics of modular kernel are very close to what we find in implementation of ''plugins'' in applications or ''dynamic libraries'' in general.

==What are some advantages and disadvantages for a Modular Kernel?==
===Advantages===
* The kernel doesn't have to load everything at boot time; it can be expanded as needed. This can decrease boot time, as some drivers won't be loaded unless the hardware they run is used (NOTE: This boot time decrease can be negligible depending on what drivers are modules, how they're loaded, etc.)
* The core kernel isn't as big
* If you need a new module, you don't have to recompile.

===Disadvantages===
* It may lose stability. If there is a module that does something bad, the kernel can crash, as modules should have full permissions.
* ...and therefore security is compromised. A module can do anything, so one could easily write an evil module to crash things. (Some OSs, like [https://en.wikipedia.org/wiki/Linux Linux], only allow modules to be loaded by the root user.)
* Coding can be more difficult, as the module cannot reference kernel procedures without kernel symbols.

==What does a Modular Kernel look like ?==

There are several components that can be identified in virtually every modular kernel:
;The core
: This is the collection of features in the kernel that are absolutely mandatory regardless of whether you have modules or not.
;The modules loader
: This is a part of the system that will be responsible of preparing a module file so that it can be used as if it was a part of the core itself.
;The kernel symbols table
: This contains additional information about the core and loaded modules that the module loader needs in order to ''link'' a new module to the existing kernel.
;The dependencies tracking
: As soon as you want to ''unload'' some module, you'll have to know whether you can do it or not. Especially, if a module ''X'' has requested symbols from module ''Z'', trying to unload ''Z'' while ''X'' is present in the system is likely to cause havoc.
;Modules
: Every part of the system you might want (or don't want) to have.

== How can such a system boot in first place ?==

Modularization must be done within certain limits if you still want your system to boot. Pushing ''all'' filesystem and device drivers (including the boot device driver) into modules will likely make the boot time more complicated. The following solutions can, however, be used:
* The kernel is provided with an extremely simple [[File Systems|filesystem]] (e.g. SCO's [[BFS]]) driver and that filesystem contains modules to access the rest of system storage (e.g. module for ATA, SCSI, EXT2FS, [[ReiserFS]], FAT, NTFS ...).
* The kernel comes with a built-in native file system driver with other storage modules, and primary configuration files should be stored using that native filesystem. This was the approach followed by Linux, and when some people decided to use [[ReiserFS|reiser]] everywhere, ext2-fs only kernels started having trouble on some machines.
* The bootloader knows it should not only load the ''kernel'' but also a collection of pre-configured modules so the kernel only needs to check and initialize pre-loaded modules to access other modules and primary configuration files. This means that your bootloader acts as an OS of its own, such as [[GRUB]].

In this case, ramdisk drivers and dedicated boot partitions/reserved sectors will be your friends.

==See Also==
===Forum Threads===
*[[Topic:10031|Design of a basic module loader]]
*[[Topic:9921|Calling a function knowing its name]]
*[[Topic:9827|Ideas for IPC]]
*[[Topic:9742|Coff i386 relocations]]
*[[Topic:9713|Device Drivers Interface]]
*[[Topic:9437|Loading Drivers into Kernel]]
*[[Topic:9408|Design & Implementation of Extensible OS]]

[[Category:Kernel]]
````

<a id="w001907"></a>

## W001907 — LILO

Bron-ID: W001907
Namespace: Main (0)
Revisie: 17713 | Datum: 2015-03-03T15:18:58Z
Bron: https://wiki.osdev.org/LILO
Revisie-URL: https://wiki.osdev.org/index.php?title=LILO&oldid=17713

````text
{{Stub}}

LILO (LInux LOader) was probably the most well known boot loader around, due to its popularity as the [https://en.wikipedia.org/wiki/Linux Linux] boot loader. It was designed to allow Linux to multiboot and co-exist with other operating systems, such as DOS and Windows.

It was designed, for the most part, to handle Linux and its initrd's, as well as "chain-loading" other boot loaders. To the knowledge of the authors, no "custom" kernel was yet set up to be booted by LILO.

More recently, [[GRUB]] started to replace LILO in many Linux distributions because of its higher versatility (and support for custom kernels...).

==See Also==
* [http://lilo.go.dyndns.org/ Homepage]

[[Category:Bootloaders]]
````

<a id="w001908"></a>

## W001908 — Link Archiver

Bron-ID: W001908
Namespace: Main (0)
Revisie: 19525 | Datum: 2016-07-22T19:42:53Z
Bron: https://wiki.osdev.org/Link_Archiver
Revisie-URL: https://wiki.osdev.org/index.php?title=Link_Archiver&oldid=19525

````text
{{Stub}}
{{Infobox_Tool
|name=GNU Archiver
|website=http://www.gnu.org/software/binutils/
}}

GNU '''ar''' is a utility that archives a series of files and is part of the [[binutils]] package. It is used to take a group of files, and combine them all into one. This allows one to then compress, send, or most importantly and commonly, link the files into a program.

== Usage ==
The tool itself is about as simple as its purpose. To archive a series of object files: 
<source lang="bash">ar -rcs archive.a obj_fil1.o obj_fil2.o obj_fil3.o obj_fil4.o ... obj_filN.o</source>

Then, to link against them:
<source lang="bash">cc example.c archive.a -o example.bin</source>

'''Important''': If you link other object files besides the library ''archive.a'', take care that the ''archive.a'' is right behind these object files (or have it be the very last file in the list of files to link) - so the linker can resolve the one or other occurrence of a symbol which is covered by some member of the ''archive.a'' library.

== See Also ==

=== Articles ===
* [[binutils]]
* [[LD|ld]]

=== External Links ===
* [https://sourceware.org/binutils/docs/binutils/ar.html#ar GNU 'ar' online manual].

[[Category:Binutils]]
````

<a id="w001909"></a>

## W001909 — Mouse Input

Bron-ID: W001909
Namespace: Main (0)
Revisie: 29232 | Datum: 2024-08-24T14:58:02Z
Bron: https://wiki.osdev.org/Mouse_Input
Revisie-URL: https://wiki.osdev.org/index.php?title=Mouse_Input&oldid=29232

````text
==PC Mouse Interfaces==
Current PCs generally use PS2 mice, or a similar format that emulates a PS2 mouse. Serial mice are a much
older technology that is no longer common.

===USB Mouse===
A USB mouse generally emulates a PS2 mouse, except that it generates IRQs from the USB bus, and not IRQ 12. Good idea is to also select the checkbox "enable mouse capture". "Mouse toggle method" enables you to choose from a list hotkey to switch between mouse capture enabled or disabled.

===Bochs===
If you use bochs, and want your mouse to emulate PS2 mouse, you need to edit keyboard & mouse settings and set mouse type to either "ps2" (this is the mouse without mouse wheel button) or "imps2" (this is the mouse with mouse wheel button).

===PS2 Mouse -- Basic Operation (Microsoft compliant)===
Once a mouse has been initialized (see below), a mouse sends 3 or 4 byte packets to communicate mouse
movement, and mouse button press/release events. These packets show up asynchronously as data on IO port 0x60.
Even though the data shows up on port 0x60 (the keyboard port), it does <i>not</i> trigger an IRQ1. The only
way that you will know that mouse data has arrived is if you handle an appropriate IRQ (generally IRQ12), or
if you occasionally poll bit number 0 (value=1) of port 0x64, to see if data is available on port 0x60. It is
not necessary to handle all of the following things inside the driver, but doing so can make things work faster.

====Keyboard/Aux Data Bit====
Since both keyboard and mouse data is showing up to be read on port 0x60, it is necessary to be able to tell
which is which. To tell if there is any available data on port 0x60 at all, it is necessary
to read a byte from port 0x64. In that byte from port 0x64, bit number 0 (value=1) indicates that a byte is
available to be read on port 0x60. An additional bit, bit number 5 (value=0x20 or 0b00100000), indicates that this next byte
came from the mouse, if the bit is set. If you look at [[RBIL]], it says that this "mouse bit" is MCA specific,
but this is no longer true. All PCs that support PS2 mice use this bit to indicate that the incoming byte was
generated by the auxiliary PS2 input port.

==== PS2 Mouse Subtypes====
There are several types of mice, but they can be separated into two groups, depending on whether they have a scroll wheel.
Mice with scroll wheels can send an additional byte in each mouse packet,
indicating the status of mouse wheels and extra buttons.

Mice with no scroll wheels use 3 byte packets exclusively. Mice with scroll wheels and up to
5 buttons (currently) can send 4 byte packets, if they are initialized properly.

====Mouse Packet Info====
For even greater detail on Microsoft compliant PS2 mouse packets than what is presented next: reference the links below,
or search for the Adam Chapweske article. For mice that are not MS-compliant, see the Linux article.

=====Timing/Generation of Mouse Packets=====
A mouse is usually initialized to generate movement packets at a particular rate. The default rate is 100 packets
per second if the mouse is being moved. A mouse also generates a packet if a button is either pressed
or released. If the mouse is not moving and no buttons are being clicked a mouse will not generate any automatic
packets.

=====Format of First 3 Packet Bytes=====
Even if your mouse is sending 4 byte packets, the first 3 bytes always have the same format. The first byte has
a bunch of bit flags. The second byte is the "delta X" value -- that is, it measures horizontal mouse movement,
with left being negative. The third byte is "delta Y", with down (toward the user) being negative. Typical values
for deltaX and deltaY are one or two for slow movement, and perhaps 20 for very fast movement. Maximum possible
values are +255 to -256 (they are 9-bit quantities, two's complement).
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
! Right Btn 
! Left Btn
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

The top two bits of the first byte (values 0x80 and 0x40) supposedly show Y and X overflows, respectively. They
are not useful. If they are set, you should probably just discard the entire packet.

Bit number 5 of the first byte (value 0x20) indicates that delta Y (the 3rd byte) is a negative number, if it is set.
This means that you should OR 0xFFFFFF00 onto the value of delta Y, as a sign extension (if using 32 bits).

Bit number 4 of the first byte (value 0x10) indicates that delta X (the 2nd byte) is a negative number, if it is set.
This means that you should OR 0xFFFFFF00 onto the value of delta X, as a sign extension (if using 32 bits).

Bit number 3 of the first byte (value 0x8) is supposed to be always set. This helps to maintain and verify packet
alignment. Unfortunately, some older mice (such as 10 year old Microspeed 2 button trackballs) do not set this bit.
[[RBIL]] claims that this bit should be 0, but it is wrong.

The bottom 3 bits of the first byte indicate whether the middle, right, or left mouse buttons are currently being
held down, if the respective bit is set. Middle = bit 2 (value=4), right = bit 1 (value=2), left = bit 0 (value=1).

=====Formats of Optional 4th Packet Byte=====
If the mouse has been initalized so that its mouseID is 3 or 4, it will send a 4th byte in each packet. On all current
mice, the top two bits should be ignored. On some mice, the bits will flip between 0 and 1, based on scroll wheel
movement. If the mouse has a 4th and 5th mouse button, then their state is indicated by bit 4 (value=0x10), and bit
5 (value=0x20), respectively. Note: if the buttons <i>do not</i> exist, then these bits may flip based on scroll
wheel movement! (ie. Be careful that this does not generate spurious "mouse button click" events for buttons that
do not exist.)

The bottom nibble (4 bits) of the 4th byte will be one of the following values:
* 0 -- no scroll wheel movement
* 1 -- vertical scroll up one click
* 0xF -- vertical scroll down one click
* 2 -- horizontal scroll right one click
* 0xE -- horizontal scroll left one click

When only the vertical scrollwheel exists or has been activated, the 4th packet byte simply contains:
<br>
byte 4:
{| border="1" cellspacing="0" cellpadding="5" align="center"
!  Z movement
|}
However, if the mouse has extra buttons and gets initialized properly, then it will look like this:
<br>
byte 4:
{| border="1" cellspacing="0" cellpadding="5" align="center"
! Mostly 0
! Mostly 0
! 5th btn down (pressed)
! 4th btn down (pressed)
! Z3
! Z2
! Z1
! Z0
|}
Where Z0 though Z3 is a 4 bit signed value of the Z movement (scrollwheel), with the values mentioned above (0, 1, 2, 0xE, 0xF)
and with the meanings listed above.

====Non-Linear Movement====
When a user is trying to point at something on a screen, they will quickly move the mouse in the general direction
of the target, and then slow down to accurately point at it. The deltaX/Y values from the mouse packets
can be used directly, but doing that will force the user to move the mouse very far to get the cursor from one area of the
screen to another.

A better answer may be to scale the mouse movement by additional multiples, based on how fast the mouse is moving.
If the mouse is moving slowly, the scale factor can be 1 -- to allow the highest possible sensitivity. When the mouse
is (or has been) moving fast, the scale factor could be 5 or 10, to allow the cursor to move across large distances on
the screen quickly.

There are many ways to do such scaling, obviously. If you are coding the driver in assembler, one suggestion might
be to use the BSR command to generate an approximate log base2 of deltaX plus deltaY, and use that as your scaling factor.

====Doubleclicks====
If you intend to implement a doubleclick, you will need to record timestamps of mousedown events to high accuracy.
The only way to know if a doubleclick has occurred is to know how long ago the previous click of the mouse button
happened. If the delay between the two clicks was less than some threshhold, then it was a doubleclick. The problem
here is that you need to be able to measure very short delays, and there may be extremely long delays between clicks (days, perhaps) --
long enough that your timestamp measurement might overflow. This needs to be handled carefully.

====Timeouts/Mouse Disconnect====
If a mouse is operating normally but is not being moved or clicked, it will not send any packets. If a mouse has been unplugged,
it will also not send any packets. If you want to support "hot plugging" of PS2 mice, then you need to know when a mouse has
become disconnected, because you will need to reinitialize it. If you have not seen any packets from the mouse for awhile,
you can query the mouse to see if it is still alive. A convenient way to do this is to send a special request for a single
mouse packet (mouse command 0xEB). You will get back an ACK (0xFA) from the mouse, and then
a mouse packet (with everything set to 0, probably). Please note that you need to make sure that the 0xFA does not
cause a misalignment of your input mouse packet. Also please note that if the contents of the mouse packet are <i>not</i> 0,
then that means that somehow your mouse packets have become disabled.

In general, it is a good idea to also have timeouts everywhere that the system is waiting for a response from the mouse,
because it may never come.

===PS2 Mouse Commands===
After the PS2 Aux port has been enabled, you can send commands to the mouse. It is recommended to disable automatic packet
streaming mode while "reprogramming" the mouse. You can do this by either sending command 0xF5 to the mouse, or disabling
the "master mouse clock" by setting bit 5 of the Compaq Status byte (see below).

====Waiting to Send Bytes to Port 0x60 and 0x64====
All output to port 0x60 or 0x64 must be preceded by waiting for bit 1 (value=2) of port 0x64 to become clear. Similarly,
bytes cannot be read from port 0x60 until bit 0 (value=1) of port 0x64 is set. See [[PS2 Keyboard]] for further details.

====0xD4 Byte, Command Byte, Data Byte====
Sending a command or data byte to the mouse (to port 0x60) must be preceded by sending a 0xD4 byte to port 0x64 (with appropriate
waits on port 0x64, bit 1, before sending each output byte). Note: this 0xD4 byte does <i>not</i> generate any ACK, from
either the keyboard or mouse.

====Wait for ACK from Mouse====
It is required to wait until the mouse sends back the 0xFA acknowledgement byte after each command or data byte before sending the
next byte (Note: reset commands might not be ACK'ed -- wait for the 0xAA after a reset). A few commands require an additional
data byte, and both bytes will generate an ACK.

====Useful Mouse Command Set====
*Note: remember that the mouse responds to all command bytes and data bytes with an ACK (0xFA).
*Note2: the commands given in the table are sent to port 0x60. If a command needs additional byte (like sampling rate), this byte goes to port 0x60 too.

{| border="1" cellspacing="0" cellpadding="5" align="center"
! Hex value
! Meaning
! Description
|-
! 0xFF
! Reset
! The mouse probably sends ACK (0xFA) plus several more bytes, then resets itself, and always sends 0xAA.
|-
! 0xFE
! Resend
! This command makes the mouse send its most recent packet to the host again.
|-
! 0xF6
! Set Defaults
! Disables streaming, sets the packet rate to 100 per second, and resolution to 4 pixels per mm.
|-
! 0xF5
! Disable Packet Streaming
! The mouse stops sending automatic packets.
|-
! 0xF4
! Enable Packet Streaming
! The mouse starts sending automatic packets when the mouse moves or is clicked.
|-
! 0xF3
! Set Sample Rate
! Requires an additional data byte: automatic packets per second (see below for legal values).
|-
! 0xF2 
! Get MouseID
! The mouse sends sends its current "ID", which may change with mouse initialization.
|-
! 0xEB
! Request Single Packet
! The mouse sends ACK, followed by a complete mouse packet with current data.
|-
! 0xE9
! Status Request
! The mouse sends ACK, then 3 status bytes. See below for the status byte format.
|-
! 0xE8 
! Set Resolution
! Requires an additional data byte: pixels per millimeter resolution (value 0 to 3)

|}

====Additional Useless Mouse Commands====
{| border="1" cellspacing="0" cellpadding="5" align="center"
! Hex value
! Meaning
! Description
|-
! 0xF0
! Set Remote Mode
! The mouse sends ACK (0xFA) and  then reset its movement counters, and enters remote mode
|-
! 0xEE
! Set Wrap Mode
! The mouse sends ACK (0xFA) and  then reset its movement counters, and enters wrap mode
|-
! 0xEC
! Reset Wrap Mode
! The mouse sends ACK, and then enters the last mode, before entering wrap mode, it also resets its movement counters
|-
! 0xEA
! Set Stream Mode
! The mouse sends ACK (0xFA) and  then reset its movement counters, and enters reporting  mode
|-
! 0xE7
! Set Scaling 2:1
! The mouse sends ACK and sets non-linear scaling "2:1"
|-
! 0xE6
! Set Scaling 1:1
! The mouse sends ACK and sets normal linear scaling "1:1"

|}

<br>
The status bytes look like this:
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
!resolution
|}
Byte 3:
{| border="1" cellspacing="0" cellpadding="5" align="center"
!sample rate
|}<br>
Mode: if it is 1, the current mode is remote mode; if 0 then it is stream mode <br>
Enable: if it is 1, then data reporting is enabled; if 0 then data reporting  is disabled<br>
Scaling: if it is 1, scaling 2:1 is enabled; if 0 then scaling 1:1 is enabled.<br>

=====Resolution, Scaling and Sample Rate=====
Definitions:
*Resolution: DeltaX or DeltaY for each millimeter of mouse movement.
*Scaling: Apply a simple non-linear distortion to mouse movement (see Non-Linear Movement, above).
*Sampling Rate: Packets the mouse can send per second.

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
Scaling can either be "1:1" (linear = no scaling) or "2:1" (non-linear). This is the non-linear scaling:

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

Sample Rate can have the following values:  (all values are Decimal, NOT hex)<br>
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

*Note: human eyes don't see movement faster than 30 samples per second, and human fingers cannot click a button
that fast either. A sample rate lower than 30 will cause visibly jerky mouse movement, and may miss mousedown events.
A sample rate significantly higher than 30 will waste precious I/O bus bandwidth. You may test these things for
yourself, but sample rates of 10, 20, 100, or 200 are generally not recommended.

===Initializing a PS2 Mouse===
The PS2 mouse port on a PC is attached to the auxiliary input of the PS2 keyboard controller. That input
might be disabled at bootup, and needs to be enabled. It is usually also desirable to have the mouse generate
IRQ12 interrupts when it sends bytes through the keyboard controller to IO port 0x60. Additionally, it
is necessary to tell the mouse to enable the transmission of packets. Optionally, you may also want to enable additional mouse
features, such as scroll wheels, faster response times, increased resolution, or additional mouse buttons.

====PS/2 Device Unplugging/Hot Plugging====
The person who created the PS/2 device specs did not specify that PS/2 devices can be unplugged and replugged
while the computer remains turned on ("hot plugging"). A long time ago, some other people actually designed motherboards that would
be slightly damaged if PS2 hot plugging occurs. However, mice and keyboards have cords that were made to be tripped over,
and sometimes it is very logical to try moving a mouse from one machine to another, temporarily, without powering
both systems down. So all computers made in the last 15 years should, in fact, support hot plugging of PS2 devices.
When a mouse is plugged into a running system it may send a 0xAA, then a 0x00 byte, and then go into default state (see below).

=====Set Compaq Status/Enable IRQ12=====
On some systems, the PS2 aux port is disabled at boot. Data coming from the aux port will not generate any interrupts.
To know that data has arrived, you need to enable the aux port to generate IRQ12. There is only one way to do that,
which involves getting/modifying the "compaq status" byte. You need to send the command byte 0x20 ("Get Compaq Status Byte")
to the PS2 controller on port 0x64. If you look at [[RBIL]], it says that this command
is Compaq specific, but this is no longer true. This command does <i>not</i> generate a 0xFA ACK byte. The very
next byte returned should be the Status byte. (Note: on some versions of Bochs, you will get a <i>second</i> byte, with
a value of 0xD8, after sending this command, for some reason.) After you get the Status byte, you need to set bit number 1
(value=2, Enable IRQ12), and clear bit number 5 (value=0x20, Disable Mouse Clock). Then send command byte 0x60 ("Set Compaq
Status") to port 0x64, followed by the modified Status byte to port 0x60. This might generate a 0xFA ACK byte from the keyboard.

=====Aux Input Enable Command=====
Send the Enable Auxiliary Device command (0xA8) to port 0x64. This will generate an ACK response from the <i>keyboard</i>,
which you must wait to receive. Please note that using this command is not necessary if setting the Compaq Status byte is
successful -- but it does no harm, either.

====Mouse State at Power-on====
When the mouse is reset, either by applying power or with a reset command (0xFF), it always goes into
the following default state:

* packets disabled
* emulate 3 button mouse (buttons 4, 5, and scroll wheels disabled)
* 3 byte packets
* 4 pixel/mm resolution
* 100 packets per second sample rate

====MouseID Byte====
During initialization, a mouse indicates that it has various features (a scroll wheel, a 4th and 5th mouse
button) by changing its mouseID in response to initialization commands. So you send a set of mouse commands,
and then ask for the mouseID byte with the Get MouseID command (0xF2). If the mouseID changed from its previous value,
then the mouse has changed modes. The mouseID byte is always the next byte sent after
the ACK for the Read MouseID command. At initialization, the mouseID is always 0. Other current legal values are
3 and 4.

====Init/Detection Command Sequences====
If you would like more than just 3 buttons, you will have to use the following sequence(s). 
If the first sequence is accepted: the number of bytes in the mouse packets changes to 4, the scroll wheel on the
mouse becomes activated, and the mouseID changes from 0 to 3.

The first magic sequence goes like this:
*	set sample rate to 200
*	set sample rate to 100
*	set sample rate to 80
*	get the new id to verify acceptance
<br>

<b>After using the above sequence to activate the scroll wheel</b>, the 4th and 5th mouse buttons can be activated
with the following additional magic sequence:
*	set sample rate to 200
*	set sample rate to 200
*	set sample rate to 80
*	get the new id to verify acceptance
<br>
If this second sequence is accepted, the returned mouse ID value changes from 3 to 4.

====Enable Packets====
After the mouse has been initialized to your desired mouseID value, its Samplerate is probably 80 samples per second,
its Resolution is probably 4 pixels/mm, and packets are still disabled. You may want to modify the Samplerate
and Resolution, and then send a 0xF4 command to the mouse to make the mouse automatically generate movement packets.

====Streaming Advantages and Disadvantages====
Instead of enabling automatic streaming packet mode, it is possible to request mouse packets one at a time. Doing this
has some advantages over streaming packet mode. You may see that you need to send or receive at least 4
extra bytes over the I/O bus in order to get each 0xEB command to the mouse, and the I/O bus is very slow. On the other
hand, a typical streaming mode probably sends hundreds more mouse packet bytes over the I/O bus than you need, every second
-- so if there is a disadvantage, it is probably small.

One of the biggest problems with streaming mode is "alignment" -- the packets were never defined to have an obvious boundary.
This means that it is very easy to lose track of which mouse byte is supposed to be the first byte of the next packet.
This problem is completely avoided if you specifically request single packets (instead of using streaming mode) because
every packet begins with an ACK (0xFA), which is easily recognizable.

===PC Serial Mouse===

For info on running a serial mouse on an RS232 port, see [https://web.archive.org/web/20090302000300/https://freedos-32.sourceforge.net/showdoc.php?page=sermouse this document from the freedos documentation].

==Mac Mouse Interface==

{{Stub}}


==See Also==
===Threads===
* [[topic:10247|SANiK's mouse code]]
* [[topic:24277|Mouse driver code in assembly]]
* [[topic:8323|PS2 mouse IRQ]]
* [[topic:6942|PS2 mouse links]]

===External Links===
* [http://os-development.000webhostapp.com/mys.html Tutorial on page os-development]
* [https://www.avrfreaks.net/sites/default/files/PS2%20Keyboard.pdf Adam Chapweske on PS/2 Mouse and Keyboard protocols]
* [http://www.win.tue.nl/~aeb/linux/kbd/scancodes-13.html Linux PS2 mouse article with non-Microsoft mouse info]
* [http://houbysoft.com/download/ps2mouse.html Tutorial with sample C code]
* [http://www.mcamafia.de/pdf/ibm_hitrc07.pdf IBM's Keyboard and Auxiliary Device (mouse) Controller documentation]

[[Category:Human Interface Device]]
[[de:PS/2-Maus]]
````

<a id="w001910"></a>

## W001910 — Symmetric Multiprocessing

Bron-ID: W001910
Namespace: Main (0)
Revisie: 30332 | Datum: 2026-03-17T20:53:43Z
Bron: https://wiki.osdev.org/Symmetric_Multiprocessing
Revisie-URL: https://wiki.osdev.org/index.php?title=Symmetric_Multiprocessing&oldid=30332

````text
{{Stub}}

'''Symmetric Multiprocessing''' (or '''SMP''') is one method of having multiple processors in one computer system. In an SMP system (as opposed to a [[NUMA]] system) all logical cores are able to see the entire memory for the system. Note that SMP and NUMA are not mutually exclusive however; as [[User:Brendan|Brendan]] has pointed out on the forums, Intel's [[IA32_Architecture_Family#Core_i7_Series|Core i7]] implements both SMP and NUMA, as well as hyper-threading.

==Initialisation of an old SMP system==

The startup sequence is different for different CPUs. Intel's system programmer's manual (section 7.5.4) contains the initialization protocol for Intel Xeon processors, and doesn't cover older CPUs. For the generic "all CPU types" algorithm, see Intel's Multi-processor Specification.

For 80486 (with an external 8249DX local APIC), you must use an INIT IPI followed by an "INIT level de-assert" IPI without any SIPI's. This means you can't tell them where to start executing (the vector part of a SIPI) and they always start executing BIOS code. In this case you set the BIOS's CMOS reset value to "warm start with far jump" (i.e. set CMOS location 0x0F to the value 10) so that the BIOS will do a <tt>jmp far ~[0:0x0469]</tt>", and then put the segment & offset of your AP entry point at 0x0469.

The "INIT level de-assert" IPI isn't supported on newer CPUs (Pentium 4 and Intel Xeon), and AFAIK it is ignored completely on these CPUs.

For newer CPUs (P6, Pentium 4) one SIPI is enough, but I'm not sure if older Intel CPUs (Pentium) or CPUs from other manufacturers need a second SIPI or not. It's also possible that the second SIPI is there in case there's a delivery failure for the first SIPI (bus noise, etc).

I normally send the first SIPI and then wait to see if the AP CPU increases a "number of started CPUs" counter. If it doesn't increase this counter within a few milli-seconds, then I send the second SIPI. This is different to Intel's generic algorithm (which has a 200 micro-second delay between SIPIs), but trying to find a time source capable of accurately measuring a 200 micro-second delay during early boot isn't so easy. I've also found that on real hardware, if the delay between SIPIs is too long (and you don't use my method) an AP CPU can run the OS's early AP startup code twice (which in my case would lead to the OS thinking there's twice as many AP CPUs as there are).

You can broadcast these signals across the bus to start every device that is present. However by doing so you might also enable the processors that were disabled on purpose (because they were defective).

==Finding information using MP Table==
You may want to use newer ACPI instead of MP Table. If so, please see the next section.

Some information (which may not be present on newer machines) dedicated for multiprocessing is available. First one must find the MP Floating Pointer Structure. It is aligned on a 16 byte boundary, and contains a signature at the start "_MP_" or 0x5F504D5F. The OS must search in the EBDA, the BIOS ROM space, and last kilobyte of "base memory"; the size of base memory is specified in a 2 byte value at 0x413 in kilobytes, minus 1K.
Here is what the structure looks like:
<syntaxhighlight lang="c">
struct mp_floating_pointer_structure {
    char signature[4];
    uint32_t configuration_table;
    uint8_t length; // In 16 bytes (e.g. 1 = 16 bytes, 2 = 32 bytes)
    uint8_t mp_specification_revision;
    uint8_t checksum; // This value should make all bytes in the table equal 0 when added together
    uint8_t default_configuration; // If this is not zero then configuration_table should be 
                                   // ignored and a default configuration should be loaded instead
    uint32_t features; // If bit 7 is then the IMCR is present and PIC mode is being used, otherwise 
                       // virtual wire mode is; all other bits are reserved
}
</syntaxhighlight>
Here is what the configuration table, pointed to by the floating pointer structure looks like:
<syntaxhighlight lang="c">
struct mp_configuration_table {
    char signature[4]; // "PCMP"
    uint16_t length;
    uint8_t mp_specification_revision;
    uint8_t checksum; // Again, the byte should be all bytes in the table add up to 0
    char oem_id[8];
    char product_id[12];
    uint32_t oem_table;
    uint16_t oem_table_size;
    uint16_t entry_count; // This value represents how many entries are following this table
    uint32_t lapic_address; // This is the memory mapped address of the local APICs 
    uint16_t extended_table_length;
    uint8_t extended_table_checksum;
    uint8_t reserved;
}
</syntaxhighlight>
After the configuration table, there are entry_count entries describing more information about the system, then after that there is an extended table. The entries are either 20 bytes to represent a processor, or 8 bytes for something else. Here are what the processor and IO APIC entries look like.
<syntaxhighlight lang="c"> 
struct entry_processor {
    uint8_t type; // Always 0
    uint8_t local_apic_id;
    uint8_t local_apic_version;
    uint8_t flags; // If bit 0 is clear then the processor must be ignored
                   // If bit 1 is set then the processor is the bootstrap processor
    uint32_t signature;
    uint32_t feature_flags;
    uint64_t reserved;
}
</syntaxhighlight>
Here is an IO APIC entry.
<syntaxhighlight lang="c">
struct entry_io_apic {
    uint8_t type; // Always 2
    uint8_t id;
    uint8_t version;
    uint8_t flags; // If bit 0 is set then the entry should be ignored
    uint32_t address; // The memory mapped address of the IO APIC is memory
}
</syntaxhighlight>
For more information, see http://www.intel.com/design/pentium/datashts/24201606.pdf, chapter 4.

==Finding information using ACPI==
You should be able to find a [[MADT]] table in the [[RSDT]] table or in the [[XSDT]] table. The table has a list of local-APICs, number of which should be the same as the number of cores on your processor. Details of these tables are not listed here, but you can find them easily on this wiki.

==AP startup==
After you've gathered the information, you'll need to disable the PIC and prepare for I/O APIC. You also need to setup BSP's local APIC.
Then, startup the APs using SIPIs.
===Startup Sequence===
The MP specification contains a standard method to start an AP, however it is not recommended to be used, as it contains very precise timings, which, if done incorrectly, can lead to several problems. Brendan offers an alternative method, which should be done for each AP '''individually'''. First send an init IPI and wait 10 milliseconds. Then send a SIPI, and poll for a flag to be set by the AP's trampoline code with a timeout of 1 millisecond. If the timeout was reached, send another SIPI, and poll for the same flag, but this time with a timeout of 1 second. If the AP managed to set the flag, the BSP should set another flag to allow the AP to continue (probably to wait for the scheduler to have a process it needs executing).

However that alternative method is highly overcomplicated. It is much simpler to have it the other way around, just send the two SIPI and make the APs to wait for the BSP, see in the example code below.

===Timing===
The easiest method for the timings is to use the PIT's mode 0. Write 0x30 to IO port 0x43 (select mode 0 for counter 0), then write your count value to 0x40, LSB first (e.g. write 0xA9 then 0x4 for a millisecond). To check if counter has finished, write 0xE2 to IO port 0x43, then read a status byte from port 0x40. If the 7th bit is set, then it has finished.
===Sending IPIs===
IPIs are sent through the BSP's LAPIC. Find the LAPIC base address from the MP tables or ACPI tables, then you can write 32-bit words to base + 0x300 and base + 0x310 to send IPIs. For a init IPI or startup IPI, you must first write the target LAPIC ID into bits 24-27 of base + 0x310. Then, for an init IPI, write 0x00004500 to base + 0x300. For a SIPI, write 0x00004600, ored with the page number at which you want to AP to start executing, to base + 0x300. For more information, see http://wiki.osdev.org/APIC#Local_APIC_registers.

==Example Code==
When you start testing your SMP code on real machines, you'll realize that they do not keep the standard. You must not do Broadcast INIT IPIs nor Broadcast SIPIs. The following example has a decent amount of error checking to be used on real hardware. It needs three variables
* ''numcores'' the number of valid cores
* ''lapic_ids'' an array of Local APIC IDs, numcores element
* ''lapic_ptr'' the pointer to the Local APIC registers
You can get these from the PCMP table above, or see the example code on the [[MADT]] page.

=== BSP Initialization Code ===
<syntaxhighlight lang="c">
volatile uint8_t aprunning = 0;  // count how many APs have started
uint8_t bspid, bspdone = 0;      // BSP id and spinlock flag
// get the BSP's Local APIC ID
__asm__ __volatile__ ("mov $1, %%eax; cpuid; shrl $24, %%ebx;": "=b"(bspid) : : );

// copy the AP trampoline code to a fixed address in low conventional memory (to address 0x0800:0x0000)
memcpy((void*)0x8000, &ap_trampoline, 4096);

// for each Local APIC ID we do...
for(i = 0; i < numcores; i++) {
	// do not start BSP, that's already running this code
	if(lapic_ids[i] == bspid) continue;
	// send INIT IPI
	*((volatile uint32_t*)(lapic_ptr + 0x310)) = (*((volatile uint32_t*)(lapic_ptr + 0x310))) | (i << 24);         // select AP
	*((volatile uint32_t*)(lapic_ptr + 0x300)) = (*((volatile uint32_t*)(lapic_ptr + 0x300)) & 0xfff00000) | 0x00C500;          // trigger INIT IPI
	do { __asm__ __volatile__ ("pause" : : : "memory"); }while(*((volatile uint32_t*)(lapic_ptr + 0x300)) & (1 << 12));         // wait for delivery
	*((volatile uint32_t*)(lapic_ptr + 0x310)) = (*((volatile uint32_t*)(lapic_ptr + 0x310))) | (i << 24);         // select AP
	*((volatile uint32_t*)(lapic_ptr + 0x300)) = (*((volatile uint32_t*)(lapic_ptr + 0x300)) & 0xfff00000) | 0x008500;          // deassert
	do { __asm__ __volatile__ ("pause" : : : "memory"); }while(*((volatile uint32_t*)(lapic_ptr + 0x300)) & (1 << 12));         // wait for delivery
	mdelay(10);                                                                                                                 // wait 10 msec
	// send STARTUP IPI (twice)
	for(j = 0; j < 2; j++) {
		*((volatile uint32_t*)(lapic_ptr + 0x280)) = 0;                                                                     // clear APIC errors
		*((volatile uint32_t*)(lapic_ptr + 0x310)) = (*((volatile uint32_t*)(lapic_ptr + 0x310))) | (i << 24); // select AP
		*((volatile uint32_t*)(lapic_ptr + 0x300)) = (*((volatile uint32_t*)(lapic_ptr + 0x300)) & 0xfff0f800) | 0x000608;  // trigger STARTUP IPI for 0800:0000
		udelay(200);                                                                                                        // wait 200 usec
		do { __asm__ __volatile__ ("pause" : : : "memory"); }while(*((volatile uint32_t*)(lapic_ptr + 0x300)) & (1 << 12)); // wait for delivery
	}
}
// release the AP spinlocks
bspdone = 1;
// now you'll have the number of running APs in 'aprunning'
</syntaxhighlight>

=== AP Initialization Code ===
As the application processors start up in real mode, a little Assembly is needed to enter protected mode. Modify this example to your kernel's needs.
<syntaxhighlight lang="asm">
; this code will be relocated to 0x8000, sets up environment for calling a C function
    .code16
ap_trampoline:
    cli
    cld
    ljmp    $0, $0x8040
    .align 16
_L8010_GDT_table:
    .long 0, 0
    .long 0x0000FFFF, 0x00CF9A00    ; flat code
    .long 0x0000FFFF, 0x008F9200    ; flat data
    .long 0x00000068, 0x00CF8900    ; tss
_L8030_GDT_value:
    .word _L8030_GDT_value - _L8010_GDT_table - 1
    .long 0x8010
    .long 0, 0
    .align 64
_L8040:
    xorw    %ax, %ax
    movw    %ax, %ds
    lgdtl   0x8030
    movl    %cr0, %eax
    orl     $1, %eax
    movl    %eax, %cr0
    ljmp    $8, $0x8060
    .align 32
    .code32
_L8060:
    movw    $16, %ax
    movw    %ax, %ds
    movw    %ax, %ss
    ; get our Local APIC ID
    mov     $1, %eax
    cpuid
    shrl    $24, %ebx
    movl    %ebx, %edi
    ; set up 32k stack, one for each core. It is important that all core must have its own stack
    shll    $15, %ebx
    movl    stack_top, %esp
    subl    %ebx, %esp
    pushl   %edi
    ; spinlock, wait for the BSP to finish
1:  pause
    cmpb    $0, bspdone
    jz      1b
    lock    incb aprunning
    ; jump into C code (should never return)
    ljmp    $8, $ap_startup
</syntaxhighlight>
<syntaxhighlight lang="C">
// this C code can be anywhere you want it, no relocation needed
void ap_startup(int apicid) {
	// do what you want to do on the AP
	while(1);
}
</syntaxhighlight>

==See Also==
===Articles===
*[[Multiprocessing]]
===Threads===
*[[Topic:11564|init-SIPI]]
*[[Topic:28674|full "AP CPU startup" sequence]]

===External Links===
*[https://web.archive.org/web/20170410220205/https://download.intel.com/design/archives/processors/pro/docs/24201606.pdf Intel Multiprocessor Specification]
*[http://www.osdever.net/tutorials/view/multiprocessing-support-for-hobby-oses-explained Multiprocessing Support for Hobby OSes Explained at BonaFide]

[[Category:X86 CPU]]

[[Category:Multiprocessing]]
````

<a id="w001911"></a>

## W001911 — System Commander

Bron-ID: W001911
Namespace: Main (0)
Revisie: 24861 | Datum: 2020-06-21T19:00:10Z
Bron: https://wiki.osdev.org/System_Commander
Revisie-URL: https://wiki.osdev.org/index.php?title=System_Commander&oldid=24861

````text
{{Stub}}

System Commander is a boot menu program. It was originally written by V-Communications, and is now owned by Avanquest Software. It has evolved into a quite complex boot menu that does auto-partitioning and a few other nifty tricks.

It's features:
*It can boot anything from BeOS, FreeBSD, and Windows '98 to the latest GNU/Linux systems.
*It supports heaps of various operating systems and file systems.
*It's packaged with Partition Commander, a complex partition tool.
*It allows you to assign custom graphics to each operating system in the menu.

Unfortunately it doesn't load custom kernels and such, but you can use it as your master boot menu, and install GRUB on a secondary partition and have System Commander boot GRUB!

==See Also==
* You can download it here: - https://system-commander.updatestar.com

[[Category:Bootloaders]]
````

<a id="w001912"></a>

## W001912 — Library Calls

Bron-ID: W001912
Namespace: Main (0)
Revisie: 28159 | Datum: 2023-07-10T14:40:06Z
Bron: https://wiki.osdev.org/Library_Calls
Revisie-URL: https://wiki.osdev.org/index.php?title=Library_Calls&oldid=28159

````text
{{In_Progress}}
{{FirstPerson}}

<source lang="c">
#include <stdio.h>
int answer = 42;
printf("The answer is %d!\n", answer);
</source>

We all know this construct from our userspace programming experiences. We never really thought about how it worked, at least for some time. You ''#include'', you get to use "the system".

For some, when starting their own OS project, there are some surprises. This page is meant to lessen the confusion, or to avoid it outright: How does a library call actually work, and what is different from kernel space?

:Note that the system call procedure described below is pretty close to what Linux actually does. Of course different approaches can be devised.

== Language Basics ==

=== Compiler ===

Let us have a look at the above code fragment, again. You could just as well write:

<source lang="c">
int printf(const char * format, ...);
char * msg = "World";
printf("Hello %s!\n", msg);
</source>

The header ''<stdio.h>'', in this case, does serve no other purpose than to tell the compiler that there is a function ''printf()'' that takes a parameter list consisting of at least a ''const char'' pointer, and returns ''int''. Nothing else is necessary, and mere convenience so you don't have to write your own declarations or ''#include'' each function individually.

=== Linker ===

The '''linker''' is a different story. When the compiler is done compiling your source file into object files, it is up to the linker to '''link''' all object files together into an executable. It also '''resolves symbols'''. For the above code, the object code refers to a function <tt>printf()</tt> which is, however, '''not defined''' in the object code itself.

It is defined in the '''system libraries'''. Luckily the linker knows that these exist, and where to find them. Any symbols not resolved in your own code it tries to resolve using the system libraries. It also links in the '''startup code''', since someone has to set up the environment and call your ''int main()'', right?

=== GCC Cross-Compiler ===

If you did follow the [[GCC Cross-Compiler]] how-to, your cross-compiler does not know about ''<stdio.h>'', and your cross-linker does not know about any system libraries. Which is just as well, since they don't exist.

== At the Bottom ==

Let us assume that, at the bottom of it, you already have some kernel-space function that accepts a ''char *'' as parameter, assumes that this points to a zero-terminated string, and prints it to screen in some way (writing to video mem, calling the BIOS, updating the framebuffer, whatever). We will not bother with other "channels" like writing to file or something.

=== Userspace ''printf()'' ===

Well, the usual userspace process for a library call is already explained above, is it? Well, yes, but only at a high level. What actually '''happens''' when you call <tt>printf()</tt> like that?

Your userland <tt>printf()</tt> does not have a string to print, yet - only a parameter that happens to be an integer, and a manual on how to make a string of it. It then proceeds to construct that very string, "The answer is 42!\n". In some way. Doesn't matter.

'''Then''' comes the funny part. The function that does the actual '''printing''' (see "At the Bottom" above) is in '''kernel space'''. You can't just execute kernel functions like that, only the kernel can.

So let's say your standard library implementation of <tt>printf()</tt>, after assembling the string to be printed, calls a function named <tt>write()</tt>, passing it a pointer to the readily-assembled string and the information that it is to be printed to stdout - like, the integer 1.

<tt>write()</tt> now does something special: It places a special code into one register, the ''char *'' and the integer in some others, and calls an '''interrupt'''.

=== Enter the kernel ===

An interrupt ends userspace processing, and wakes up the kernel - more specifically, the interrupt handler registered by the kernel. Since it was the interrupt reserved for '''system calls''', it knows what to do: The special code tells it that it was <tt>write()</tt> calling the interrupt. That means that there is a ''char *'' in '''this''' register, and an ''int'' in '''that'''... oh, it's a 1, so this goes to stdout (instead of, say, a file).

So the interrupt handler passes the ''char *'' to the printing function we mentioned in "At the Bottom" above, then returns control to the caller.

=== Done ===

The interrupt handler ends, <tt>write()</tt> is back in control, which returns to ''printf()'' which returns to your application. You successfully completed a call to <tt>printf()</tt>.

=== Kernel is Different ===

If you are in kernel space anyway, you don't want that hassle with the registers and the interrupt. After all, you '''can''' call the print function directly as you '''are''' in kernel space. You also never want to write to a file. The <tt>write()</tt> function and the interrupt handler become unnecessary baggage.

'''And''' you probably don't want all the code necessary for e.g. floating point conversions or unicode output linked into your kernel binary, either. The userspace <tt>printf()</tt> is much too heavy for your tastes.

So what do you do? The answer is easy. Just tell the linker that the _system libraries_ are to be found in a different directory than for userspace. (Your ''Makefile'' is already loaded with funny options for compiler and linker, one more wouldn't make a difference. Bad pun warning.)

So you provide a lightweight <tt>printf()</tt> for kernel space. You don't even have to bother to call it <tt>kprintf()</tt> or something as your kernel binary will never be '''linked''' with userspace code. Using some preprocessor ''#ifdef'' magic, you can even use the very same ''<stdio.h>'' header file as for userspace code, reducing redundancy and potential error sources: The preprocessor symbol <tt>__STDC_HOSTED__</tt> (with two leading and trailing underscores that the Wiki stubbornly interprets as boldface markup) is undefined when you set ''-ffreestanding''...

<source lang="c">
#ifdef __STDC_HOSTED__
// Userspace declarations
#else
// Kernelspace declarations
#endif
</source>

[[Category:Standard Libraries]]
[[Category:FAQ]]
````

<a id="w001913"></a>

## W001913 — Non Maskable Interrupt

Bron-ID: W001913
Namespace: Main (0)
Revisie: 29100 | Datum: 2024-06-07T13:39:38Z
Bron: https://wiki.osdev.org/Non_Maskable_Interrupt
Revisie-URL: https://wiki.osdev.org/index.php?title=Non_Maskable_Interrupt&oldid=29100

````text
The '''Non-Maskable Interrupt''' ('''NMI''') is a hardware-driven interrupt much like the PIC interrupts, but the NMI goes either directly to the CPU, or via another controller (e.g., the ISP)---in which case it can be masked.

==About==
NMIs occur for RAM errors and unrecoverable hardware problems. For newer computers these things may be handled using machine check exceptions and/or SMI. For the newest chipsets (at least for Intel) there's also a pile of TCO stuff ("total cost of ownership") that is tied into it all (with a special "TCO IRQ" and connections to SMI/SMM, etc). All of the TCO stuff can be connected to an onboard ethernet controller, and (at least part of it) is intended for remote monitoring of the system. Unfortunately, the chipset documentation doesn't say how BIOSes normally configure the chipset, and the chipsets themselves support several different options in each case. For example, for a RAM error it could be handled by the chipset itself, it could generate an SMI (where the BIOS/SMM handler does "RAM scrubbing" in software), it could generate a "TCO interrupt", etc. If you add it all up it's a huge complex mess (TCO + SMI + SMBus + Northbridge + PCI bus/controller/s + PCI-to-LPC-bridge + god-knows-what) that can be completely different between motherboards (even motherboards with the same chipset).

The short version of this story is that there's only really 2 reasons for an NMI. The first reason is a hardware failure. The second reason is a "watchdog timer", which can be used to detect when the kernel itself locks up (and is sometimes also used for more accurate profiling as it allows EIP to be sampled even when IRQs are disabled).

If a hardware failure caused an NMI then there's no way to figure out which piece of hardware caused the NMI. In this case you may want to inform the user that a hardware error has occurred, and then the kernel should shutdown/reset the machine.

For the watchdog timer, it must be setup by the OS first. This can actually be done even when the chipset itself doesn't have a special watchdog timer for it (e.g. setting the PIT, RTC/CMOS IRQ or a HPET IRQ to "NMI, send to all CPUs" in the I/O APIC). In this case you want the watchdog timer to be fast (i.e. no slow hardware task switching and cache flushing) and you'd also want all CPUs to share the same timer, which means all CPUs would receive the same IRQ at the same time.

As an alternative, you could also use the local APIC's timer or the performance monitoring counter overflow for a "per CPU" watchdog timer.

==Usage==

The NMI is enabled (set high) by the memory module when a memory parity error occurs.

Be careful about disabling the NMI and the PIC for extended periods of time (mind you, watchdog timers typically use NMIs).

On the XT the NMI can be masked by setting bit 7 on I/O port 0xA0. On the AT the NMI can be masked by setting bit 7 on I/O port 0x70. This port is shared with the CMOS RAM index register using bits 0 through 6 of I/O port 0x70. The CMOS RTC expects a read from or write to the data port 0x71 after any write to index port 0x70 or it may go into an undefined state. There may also need to be an [[Inline_Assembly/Examples#I/O_access|I/O delay]] between accessing the index and data registers. The index port 0x70 may be a write-only port and always return 0xFF on read. Hence the bit masking below to preserve bits 0 through 6 of the CMOS index register may not work, nor may it be possible to retrieve the current state of the NMI mask from port 0x70.

<source lang="C">
 void NMI_enable() {
    outb(0x70, inb(0x70) & 0x7F);
    inb(0x71);
 }

 void NMI_disable() {
    outb(0x70, inb(0x70) | 0x80);
    inb(0x71);
 }
</source>

When an NMI occurs you can check the system control port A and B at I/O addresses 0x92 and 0x61 respectively to get an indication of what caused the error:

System Control Port A (0x92) layout:
{| {{wikitable}}
!BIT  
!Description
|-
|0||Alternate hot reset
|-
|1||Alternate gate A20
|-
|2||Reserved
|-
|3||Security Lock
|-
|4*||Watchdog timer status
|-
|5||Reserved
|-
|6||HDD 2 drive activity
|-
|7||HDD 1 drive activity
|}

System Control Port B (0x61)
{| {{wikitable}}
!Bit
!Description
|-
|0||Timer 2 tied to speaker
|-
|1||Speaker data enable
|-
|2||Parity check enable
|-
|3||Channel check enable
|-
|4||Refresh request
|-
|5||Timer 2 output
|-
|6*||Channel check
|-
|7*||Parity check
|}

The important bits are indicated with an '*'.  The Channel Check bit indicates a failure on the bus, probably by a peripheral device such as a modem, sound card, NIC, etc, while the Parity check bit indicates a memory read or write failure.  

[[Category:Interrupts]]

[[de:Non Maskable Interrupt]]
````

<a id="w001914"></a>

## W001914 — Detecting Colour and Monochrome Monitors

Bron-ID: W001914
Namespace: Main (0)
Revisie: 28077 | Datum: 2023-07-09T20:51:24Z
Bron: https://wiki.osdev.org/Detecting_Colour_and_Monochrome_Monitors
Revisie-URL: https://wiki.osdev.org/index.php?title=Detecting_Colour_and_Monochrome_Monitors&oldid=28077

````text
Detecting whether a user has a colour or monochrome video card is a trivial task. The [[BIOS]] [[Memory Map (x86)#BIOS Data Area .28BDA.29|data segment]] has a value in it for this information. Below is a function (in ISO C) to retrieve this:

== The Function ==
<source lang="c">
#include <stdint.h>

enum video_type
{
    VIDEO_TYPE_NONE = 0x00,
    VIDEO_TYPE_COLOUR = 0x20,
    VIDEO_TYPE_MONOCHROME = 0x30,
};

uint16_t detect_bios_area_hardware(void)
{
    const uint16_t* bda_detected_hardware_ptr = (const uint16_t*) 0x410;
    return *bda_detected_hardware_ptr;
}

enum video_type get_bios_area_video_type(void)
{
    return (enum video_type) (detect_bios_area_hardware() & 0x30);
}
</source>

[[Category:Video]]
[[Category:Hardware Detection]]
````

<a id="w001915"></a>

## W001915 — Unix Pipes

Bron-ID: W001915
Namespace: Main (0)
Revisie: 23425 | Datum: 2019-02-27T15:47:23Z
Bron: https://wiki.osdev.org/Unix_Pipes
Revisie-URL: https://wiki.osdev.org/index.php?title=Unix_Pipes&oldid=23425

````text
Pipes, socketpairs and FIFOs are techniques that allow two processes to exchange data through a stream of bytes. Unlike files however, pipes and friends do not consume disk space but instead have a (usually circular) buffer within kernel. If the buffers become overloaded (e.g. if the ''consumer'' is too slow), the ''producer(s)'' will be turned to ''waiting'' state by the system.

Pipes are usually one-way streams with a ''producer'' side and a ''consumer'' side. Note that it is perfectly possible to have multiple producers and/or multiple consumers (though multiple consumers tend to make things hard to use). There is a thread in the forum on this subject; [[Topic:12851|Stream-oriented programming]].

==Usage==
Beside the usual shell-scripting use of pipes (where programs such as <tt>grep</tt> and <tt>less</tt> are linked together so that <tt>grep</tt>'s output become <tt>less</tt>'s input with <tt>grep kernel | less</tt>), several unix programs use such techniques to pass data around pre-build process, one of the most notable example of it being <tt>qmail</tt>. When running GCC, for example, using the <tt>-pipe</tt> flag will use pipes instead of temporary files for intermediate compilation results, thus speeding up compilation.

Pipes can be used by terminal emulators too to connect the teletype device (for example /dev/tty1) with the shell's standard input and output. When a command is executed, the shell uses the inherited file descriptor with the child process' new file descriptor to create a new pipe and connect the child's input and output with the teletype device. For example if you issue a <tt>cat</tt> command in a terminal, that's how the file content from cat's stdout gets to the /dev/tty1, which is read by the tty kernel subsystem to render the appropriate characters on the console.

==Implementation==

There are two commands for creating pipes on a Unix system: pipe() and mkfifo(). Both act in similar ways - the first creates an ''unnamed'' or ''anonymous'' pipe, the second creates a ''named'' pipe. Named pipes appear in the filesystem and exist until explicitly deleted; unnamed pipes do not and only exist while at least one filedescriptor is open on it.

The standard way to create an (unnamed) pipe on Unix, is to call pipe() which "returns" two file descriptors. One is used for writing to the pipe, the other for reading from the pipe.

Internally the pipe is (normally) a circular buffer/queue and two or more semaphores for locking. The size (on Linux) is typically around 4 kilobytes.

Such a pipe exists as long as either end of the pipe is open. It is cleaned after you close() both ends. There are a couple of ways to move file descriptors from a process to another on a typical unix system; the most common and 'correct' way being to pass the file descriptor from a parent to a child over a fork() call.

If one of the file descriptors is closed, and the other is tried to be used (either for read or write), you'll get an errno of EPIPE (Broken pipe).

When you write a pipe command line, it's the shell that calls pipe() and then when it fork()s the processes to do the work, it does some filedescriptor renaming (see dup()) to get the right descriptors at the right places before it finally exec()s the programs. Remember fork()ed childs inherit the parents open file descriptors by default.

Named pipes are similar, except they sit somewhere in the filesystem, and if you open the named pipe repeatedly, you get the same pipe every time. Once created, the named pipe remains in the filesystem until deleted like any other file.

==See Also==
*[http://www.cf.ac.uk/psych/CullingJ/pipes.html Introduction to UNIX Pipes]
*[[wikipedia:Pipeline_(Unix)|The Wikipedia page on Pipes]]

[[Category:IPC]]
````

<a id="w001916"></a>

## W001916 — BootProg

Bron-ID: W001916
Namespace: Main (0)
Revisie: 28477 | Datum: 2023-11-23T08:43:57Z
Bron: https://wiki.osdev.org/BootProg
Revisie-URL: https://wiki.osdev.org/index.php?title=BootProg&oldid=28477

````text
{{Stub}}

'''BootProg''' is a boot loader written by Alexey Frunze. BootProg is able to load [[COM]]/[[MZ]] programs from a FAT12/16/32-formatted media (e.g. floppy, hard disk, USB stick, CD in floppy emulation mode). It is written in NASM and may be an interesting alternative for those who find [[GRUB]] too bloated for their needs, or have a working environment where FAT/EXE is more comfortable than EXT2/ELF.

==See Also==
* [https://github.com/alexfru/BootProg BootProg Project]

[[Category:Bootloaders]]
````

<a id="w001917"></a>

## W001917 — XOSL

Bron-ID: W001917
Namespace: Main (0)
Revisie: 10694 | Datum: 2010-09-06T16:19:29Z
Bron: https://wiki.osdev.org/XOSL
Revisie-URL: https://wiki.osdev.org/index.php?title=XOSL&oldid=10694

````text
{{Stub}}

XOSL, the Extended Operating System Loader, is a graphical boot manager released by Geurt Vos under GPL.  XOSL can dual boot over 30 various operating systems on a single disk.

==See Also==
* [http://www.ranish.com/part/xosl.htm Homepage]
* [http://groups.yahoo.com/group/xosl/ XOSL Yahoo Group]
* [http://www2.arnes.si/~fkomar/xosl.org/ Historical Mirror] 

[[Category:Bootloaders]]
````

<a id="w001918"></a>

## W001918 — Unit Testing

Bron-ID: W001918
Namespace: Main (0)
Revisie: 27984 | Datum: 2023-07-09T17:27:43Z
Bron: https://wiki.osdev.org/Unit_Testing
Revisie-URL: https://wiki.osdev.org/index.php?title=Unit_Testing&oldid=27984

````text
'''Unit testing''' means that, in addition to the implementation code, you also write a "test driver" that puts the implementation code through a set of trials, checking the returns for correctness. Note that this is done during development / testing - the "test driver" is not shipped with the released code.

== The Idea is That... ==
* If you later change things in the implementation code, the test driver tells you if the results remain the same (regression test);
* If you receive a bug report, you do not only fix the implementation code, but also update the test driver so that this bug could never be introduced again without failing unit testing.

Unit testing is a great way to improve the reliability and correctness of your code. However, it requires that the code to-be-tested actually ''can'' be run under control of the test driver. It should be easy to see where this becomes tricky. While it ''should'' be possible to test most kernel functions in a "testbed" test driver, the really "juicy" stuff like interrupt handling, process dispatching or memory management are probably not unit-testable.

== Alternative / Additional Option: Run-Time Self-Tests ==

* Before activating a memory provider, ask it for memory, record the address, free the memory and ask for memory again. Most allocators are expected to return the very same address in this case.

* If your memory manager has some 'free zone merging' feature, allocate as much 'small' zones as you can, then free all the 'odd' zones and then all the 'even' zones (stressing the merge feature). When you're done, try to allocate a single zone as large as the collection of all the small zones you initially get.

* Before starting many threads, launch a thread that is expected to signal a [[Semaphores|semaphore]] and then wait on that semaphore. If your dispatcher (and semaphores) works fine, you can only go on if that other thread has been executed.

* After enabling interrupts (and especially the timer), poll the 'tick counter' and don't go on until it advanced (confirming you that the timer interrupt works fine)

As many of those tests may hang, you're suggested to print what test you're running before you know if the test was successful or not.

[[Category:Testing]]
````

<a id="w001919"></a>

## W001919 — Model Specific Registers

Bron-ID: W001919
Namespace: Main (0)
Revisie: 30860 | Datum: 2026-07-07T17:25:46Z
Bron: https://wiki.osdev.org/Model_Specific_Registers
Revisie-URL: https://wiki.osdev.org/index.php?title=Model_Specific_Registers&oldid=30860

````text
'''Model-Specific Registers''' ('''MSRs''') are special purpose registers provided by x86 processors.  They are commonly used by kernels, firmware and hypervisors to configure processor features.

Despite the name, not all MSRs are strictly model-specific.  Some MSRs are architectural and are supported across many processors while others exist only on particular vendors, families, models, etc.  Software should not assume a MSR exists merely because the processor supports the RDMSR and WRMSR instructions.

== Accessing Model Specific Registers ==

The presence of MSRs on your processor is indicated by [[CPUID]].01h:EDX[bit 5].

<syntaxhighlight lang="c">
#include <stdbool.h>
#include <stdint.h>

#define CPUID_FEATURE_MSR (1u << 5)

struct cpuid_result {
    uint32_t eax;
    uint32_t ebx;
    uint32_t ecx;
    uint32_t edx;
};

static inline struct cpuid_result cpuid(uint32_t leaf, uint32_t subleaf)
{
    struct cpuid_result r;

    asm volatile (
        "cpuid"
        : "=a"(r.eax), "=b"(r.ebx), "=c"(r.ecx), "=d"(r.edx)
        : "a"(leaf), "c"(subleaf)
    );

    return r;
}

bool cpu_has_msr(void)
{
    struct cpuid_result r = cpuid(1, 0);
    return (r.edx & CPUID_FEATURE_MSR) != 0;
}
</syntaxhighlight>

This flag only indicates that the processor supports the MSR access instructions. It does not mean that every MSR index is valid. For model-specific or vendor-specific registers, check the relevant CPUID feature bits and the vendor documentation.

==Reading and Writing MSRs==

MSRs accessed by <code>rdmsr</code> and <code>wrmsr</code> are selected using a 32-bit MSR index in <code>ECX</code>. MSR values are 64 bits wide. When reading, the low 32 bits are returned in <code>EAX</code> and the high 32 bits returned in <code>EDX</code>.  When writing, <code>EAX</code> supplies the low 32 bits and <code>EDX</code> supplies the high 32 bits.

<syntaxhighlight lang="c">
#include <stdint.h>

static inline uint64_t rdmsr(uint32_t msr)
{
    uint32_t lo;
    uint32_t hi;

    asm volatile (
        "rdmsr"
        : "=a"(lo), "=d"(hi)
        : "c"(msr)
    );

    return ((uint64_t)hi << 32) | lo;
}

static inline void wrmsr(uint32_t msr, uint64_t value)
{
    uint32_t lo = (uint32_t)value;
    uint32_t hi = (uint32_t)(value >> 32);

    asm volatile (
        "wrmsr"
        :
        : "c"(msr), "a"(lo), "d"(hi)
        : "memory"
    );
}
</syntaxhighlight>

<code>RDMSR</code> and <code>WRMSR</code> are privileged instructions.  In protected mode and long mode, they are normally usable only by kernel side code.  Attempting to access an unsupported MSR, or writing reserved or invalid bit patterns to an MSR, can raise a general-protection exception.

When modifying an MSR, use a read-modify-write sequence unless the documentation explicitly says that writing the whole register is safe.

<syntaxhighlight lang="c">
uint64_t value = rdmsr(msr);
value |= SOME_ENABLE_BIT;
wrmsr(msr, value); 
</syntaxhighlight>

==Additional x86_64 Registers==

AMD added the [[CPU_Registers_x86-64#IA32_EFER|EFER]] register for controlling specific long mode features. It has since been adopted by Intel.

{| class="wikitable"
|Bit 0
|System Call Extensions (SCE)
|-
|Bits 1-7
|Reserved
|-
|Bit 8
|Long Mode Enable (LME)
|-
|Bit 9
|Reserved
|-
|Bit 10
|Long Mode Active (LMA)
|-
|Bit 11
|No-Execute Enable (NXE)
|-
|Bit 12
|Secure Virtual Machine Enable (SVME)
|-
|Bit 13
|Long Mode Segment Limit Enable (LMSLE)
|-
|Bit 14
|fast FXSAVE/FXSTOR (FFXSR)
|-
|Bit 15
|Translation Cache Extension (TCE)
|-
|Bits 16-63
|Reserved
|}

The by far most interesting is the SCE Bit, as it enables the <code>syscall</code> instruction.

==See Also==
===Articles===
*[[CPUID]]
*[[CPU Registers x86]]
*[[CPU Registers x86-64]]

===External Links===
*http://sandpile.org/x86/msr.htm for documented MSRs
*https://github.com/xen-project/xen/blob/master/xen/arch/x86/include/asm/msr-index.h for MSRs relevant to Xen emulation (including Intel's 0x35)

[[Category:x86 CPU]]
[[Category:CPU Registers]]
[[de:Model-Specific_Register]]
````

<a id="w001921"></a>

## W001921 — Talk:Library Calls

Bron-ID: W001921
Namespace: Talk (1)
Revisie: 3212 | Datum: 2007-03-07T10:40:09Z
Bron: https://wiki.osdev.org/Talk:Library_Calls
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ALibrary_Calls&oldid=3212

````text
This page has been converted, but the content needs to be rearranged.
--[[User:Ineo|Ineo]] 04:40, 7 March 2007 (CST)
````

<a id="w001922"></a>

## W001922 — RBIL

Bron-ID: W001922
Namespace: Main (0)
Revisie: 3260 | Datum: 2007-03-09T06:39:23Z
Bron: https://wiki.osdev.org/RBIL
Revisie-URL: https://wiki.osdev.org/index.php?title=RBIL&oldid=3260
Doorverwijzing: Ralf Brown's Interrupt List

````text
#REDIRECT [[Ralf Brown's Interrupt List]]
````

<a id="w001923"></a>

## W001923 — Ralf Brown's Interrupt List

Bron-ID: W001923
Namespace: Main (0)
Revisie: 28140 | Datum: 2023-07-10T03:01:33Z
Bron: https://wiki.osdev.org/Ralf_Brown%27s_Interrupt_List
Revisie-URL: https://wiki.osdev.org/index.php?title=Ralf_Brown%27s_Interrupt_List&oldid=28140

````text
The Interrupt List is a compendium of zipped textfiles that contain information about all the generic and manufacturer specific [[Real Mode]] [[BIOS|BIOS]] interrupt calls on an x86 PC platform. It also contains generic and manufacturer specific information on [[I/O Ports|I/O port]] calls for particular devices, memory regions, CMOS, and other hardware specs.

It features per-interrupt as well as per-service review of each BIOS service, including notes on which BIOS supports what, etc.

== The Interrupt List ==

The current version number of the list is 61, and it was last updated on July 16, 2000.

The list itself is contained in 6 zipfiles, currently called inter61a.zip through inter61f.zip.

Each of the text files in each of the zipfiles contains as complete a set of specifications as possible on exactly how to use or interface to a particular interrupt command, I/O port, MSR, or other piece of equipment.

Most of the entries are specific to very particular pieces of software or hardware, and are therefore pretty useless if you aren't using, or intending to support, exactly that software or hardware. Sadly, the entries for interrupt and port calls that have been "standardized" are not easy to find.

Some of the most important included files and their contents are:
<pre>
	INTERRUP.A	INT 00 through INT 10/BE  \
	INTERRUP.B	INT 10/BF through INT 15/0F\
	INTERRUP.C	INT 15/10 through INT 15/E7 \
	INTERRUP.D	INT 15/E8 through INT 1A/B0  \
	INTERRUP.E	INT 1A/B1 through INT 1F      \	  total 2520 pages at
	INTERRUP.F	INT 20 through INT 21/43       \  60 lines per page,
	INTERRUP.G	INT 21/44 through INT 21/5E	\ 2684 with INTPRINT -p
	INTERRUP.H	INT 21/5F through INT 21/E2	 >(8754 entries)
	INTERRUP.I	INT 21/E3 through INT 21/F1	/ (4135 tables)
	INTERRUP.J	INT 21/F2 through INT 25       /
	INTERRUP.K	INT 26 through INT 2F/15      /
	INTERRUP.L	INT 2F/16 through INT 2F/79  /
	INTERRUP.M	INT 2F/7A through INT 2F/D9 /
	INTERRUP.N	INT 2F/DA through INT 50   /
	INTERRUP.O	INT 51 through INT 61	  /
	INTERRUP.P	INT 62 through INT 6A	 /
	INTERRUP.Q	INT 6B through INT 91	/
	INTERRUP.R	INT 92 through INT FF  /
	CMOS.LST	a description of the CMOS RAM data bytes
	PORTS.A		a listing of I/O ports 0000h-013Fh
	PORTS.B		a listing of I/O ports 0140h-0AD5h
	PORTS.C		a listing of I/O ports 0AD6h-FFFFh
(from INTERRUP.A)
</pre>

* inter61a.zip is 353K, and contains Interrup.a through Interrup.d
* inter61b.zip is 356K, and contains Interrup.e through Interrup.i
* inter61c.zip is 503K, and contains Interrup.j through Interrup.n
* inter61d.zip is 713K, and contains Interrup.o through Interrup.r, Cmos.lst, and Ports.a, b, and c
* inter61e.zip is 266K, and contains some utility programs
* inter61f.zip is 517K, and contains hypertext programs to view the Interrupt List files through Windows and other Helpfile systems

Each zipfile expands to well over a Meg of files.

== Other information from/about Ralf Brown ==

He is a computer science professor, who also collects information about PCI, and many other tools for programmers. He has some other computer related causes he supports.

== External Links ==

* [http://www.cs.cmu.edu/~ralf/files.html Ralf Brown's copy of the Interrupt List and other apps]
* [http://www.cs.cmu.edu/afs/cs.cmu.edu/user/ralf/pub/WWW/files.html alternate link to the Interrupt List and apps]
* [http://www.cs.cmu.edu/~ralf/ Ralf Brown's home page]
* [http://www-2.cs.cmu.edu/~ralf/ralf-home.html alternate link to his home page]
* [http://www.ctyme.com/intr/int.htm an Interrupt Jump Table]

[[Category:X86]]
[[Category:Interrupts]]
[[Category:Resources]]
````

<a id="w001925"></a>

## W001925 — VGA Text Mode

Bron-ID: W001925
Namespace: Main (0)
Revisie: 30595 | Datum: 2026-05-21T17:49:20Z
Bron: https://wiki.osdev.org/VGA_Text_Mode
Revisie-URL: https://wiki.osdev.org/index.php?title=VGA_Text_Mode&oldid=30595

````text
{{Historical}}

The VGA text mode is a straightforward way of printing text to the screen on IBM PCs, by writing two-byte characters to a standard memory address (0xB8000/0xB0000). It was originally introduced by IBM in 1987 as part of VGA standard for IBM PS/2 computers, and was widespread in PC compatibles through 1990s.

While VGA text has become a baseline for text printing on PCs, it has since been replaced by newer protocols, and is not available when booting with UEFI (and other architectures/on anything other than IBM PCs). As such, when targeting newer hardware (everything released after mid 1990s), and considering that VGA text mode is not available on many modern PCs (UEFI Class 3 machines), printing text by writing it to a [[Drawing In a Linear Framebuffer|Linear Framebuffer]] is the preferred method. The framebuffer can be set up through [[VBE]] or [[GOP]], and is usually done by any competent [[Bootloader]] for you. Many terminal emulator libraries are available for most programming languages that allow printing text to linear framebuffer with ease.

== Basics ==

Assuming that you are in [[protected mode]] and not using the [[BIOS]] to write text to screen, you will have write directly to "video" memory.

This is quite easy. The text screen video memory for color monitors resides at <tt>0xB8000</tt>, and for monochrome monitors it is at address <tt>0xB0000</tt> (see [[Detecting Colour and Monochrome Monitors]] for more information).

Text mode memory takes two bytes for every "character" on screen. One is the character ''codepoint'' byte, the other the ''attribute'' byte. The character displayed by a certain code point depends on the VGA font of the video card, but it should be compatible with ASCII; the top #7 bit being used for region-dependent character sets, aka ''code pages''.

=== Character encoding ===
{| class="wikitable"
|+
! colspan="8" | Attribute
! colspan="8" | Character 
|-
| style="text-align: center; | 15
| colspan="3" style="text-align: left;" | 14&nbsp;&nbsp;&nbsp;<span style="float: right;">12</span>
| colspan="4" style="text-align: left;" | 11&nbsp;&nbsp;&nbsp;<span style="float: right;">8</span>
| colspan="8" style="text-align: left;" | 7&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|-
| Special
| colspan="3" | Background color 
| colspan="4" | Foreground color
| colspan="8" | Character code point
|}

Thus, the text "HeLlo", with a red foreground for the first and last characters, gray for the rest, would be stored in memory as:
{| class="wikitable"
|+
! Address
! Code point (low byte)
! Attribute (high byte)
|-
| <tt>0xb8000</tt>
| <tt>0x48</tt> (<tt>H</tt>)
| <tt>0x4</tt>
|-
| <tt>0xb8002</tt>
| <tt>0x65</tt> (<tt>e</tt>)
| <tt>0x7</tt>
|-
| <tt>0xb8004</tt>
| <tt>0x4c</tt> (<tt>L</tt>)
| <tt>0x7</tt>
|-
| <tt>0xb8006</tt>
| <tt>0x6c</tt> (<tt>l</tt>)
| <tt>0x7</tt>
|-
| <tt>0xb8008</tt>
| <tt>0x6f</tt> (<tt>o</tt>)
| <tt>0x4</tt>
|-
|}


The ''attribute'' byte carries the ''foreground color'' in its lowest 4 bits and the ''background color'' in its highest 3 bits. Those values reference a color from the VGA text mode color palette. The interpretation of bit #7 (the "special" bit) depends on how you (or the [[BIOS]]) configured the hardware (see [[VGA Resources]] for additional info). Usually, it is used either for blink or as the fourth bit of the background color.

For instance, using <tt>0x00</tt> as attribute byte means black-on-black (you'll see nothing). <tt>0x07</tt> is light grey text on a black background (DOS default), <tt>0x1F</tt> is white-on-blue (Win9x's blue-screen-of-death), <tt>0x2a</tt> is green-monochrome.

For color video cards, you have 32 KB of text video memory to use. Since 80x25 mode does not use all 32 KB (80 x 25 x 2 = 4,000 bytes per screen), you have 8 display pages to use.

When you print to any other page than 0, it will ''not'' appear on screen until that page is ''enabled'' or ''copied'' into the page 0 memory space.

== Colors ==
Each character has a attribute byte which is used for specifying the color of the displayed character. This byte is split in a foreground color (4 bits), background (3-4 bits) and the special bit (also can be the fourth byte of the background).

The layout of the byte, using the standard color palette:
<pre>
Bit 76543210
    ||||||||
    |||||^^^-fore color
    ||||^----fore color bright bit
    |^^^-----back color
    ^--------back color bright bit OR enables blinking Text
</pre>

Some foreground + background combination examples:
* <tt>0x01</tt> sets the background to black and the foreground color to blue
* <tt>0x10</tt> sets the background to blue and the foreground color to black
* <tt>0x11</tt> sets both to blue

Usually, the default colors set by the [[BIOS]] are: gray foreground (7) on a black background (0).

=== VGA text mode color palette ===
{| class="wikitable"
|-
! Color number + optional bright bit
! Color name
! RGB value
|-
| 0
| Black
| style="background: #000000; color: white" | 0, 0, 0
|-
| 1
| Blue
| style="background: #0000aa; color: white" | 0, 0, 170
|-
| 2
| Green
| style="background: #00aa00; color: white" | 0, 170, 0
|-
| 3
| Cyan
| style="background: #00aaaa; color: white" | 0, 170, 170
|-
| 4
| Red
| style="background: #aa0000; color: white" | 170, 0, 0
|-
| 5
| Magenta
| style="background: #aa00aa; color: white" | 170, 0, 170
|-
| 6
| Brown
| style="background: #aa5500; color: white" | 170, 85, 0
|-
| 7
| Gray
| style="background: #aaaaaa; color: white" | 170, 170, 170
|-
| 8 <span style="float: right;">(0 + 8)</span>
| Dark Gray
| style="background: #555555; color: white" | 85, 85, 85
|-
| 9 <span style="float: right;">(1 + 8)</span>
| Light Blue
| style="background: #5555ff; color: white" | 85, 85, 255
|-
| 10 <span style="float: right;">(2 + 8)</span>
| Light Green
| style="background: #55ff55; color: black" | 85, 255, 85
|-
| 11 <span style="float: right;">(3 + 8)</span>
| Light Cyan
| style="background: #55ffff; color: black" | 85, 255, 255
|-
| 12 <span style="float: right;">(4 + 8)</span>
| Light Red
| style="background: #ff5555; color: black" | 255, 85, 85
|-
| 13 <span style="float: right;">(5 + 8)</span>
| Light Magenta
| style="background: #ff55ff; color: black" | 255, 85, 255
|-
| 14 <span style="float: right;">(6 + 8)</span>
| Yellow
| style="background: #ffff55; color: black" | 255, 255, 85
|-
| 15 <span style="float: right;">(7 + 8)</span>
| White
| style="background: #ffffff; color: black" | 255, 255, 255
|-
|}

== Printing Strings ==
Here is a bare-bones function to print to the video text buffer. It applies the same <code>attrib</code> to each printed character of the string.

<syntaxhighlight lang="c">
#define VGA_TEXT_BUFFER 0xb8000u

void print_string(const char *s, uint8_t attrib)
{
	volatile uint16_t *vp = (uint16_t *)VGA_TEXT_BUFFER;
	while (*s)
		*vp = (attrib << 8) | *s++;
}
</syntaxhighlight>

Note that this will always print the strings in the same place, as it doesn't keep track of the cursor position. For a more advanced printing function, the cursor position must be stored and incremented appropriately, which will be used to calculate the required address to put the character in the video memory at a specific position.

==Printing Integers==

Just like in any environment, you repeatedly divide the value by the base, the remainder of the division giving you the least significant digit of the value.

For example, since 1234 = 4 + 3* 10 + 2 * 100 + 1* 1000, if you repeatedly divide "1234" by ten and use the remainder of the division, you get the digits:

<pre>
1234 = 123*10 + 4
123 = 12*10 + 3
12 = 1*10 + 2
1 = 1
</pre>

As this algorithm retrieves the digits in the "wrong" order (last-to-first), you have to either work recursively, or invert the sequence of digits afterwards. If you know the numerical value of <tt>number % 10</tt>, you simply have to add this to the character '0' to have the correct character (e.g. '0'+4 == '4')

Here is an example implementation of the itoa() function (which is not standard, but provided by many libraries):

<syntaxhighlight lang="c">
char * itoa( int value, char * str, int base )
{
    char * rc;
    char * ptr;
    char * low;
    // Check for supported base.
    if ( base < 2 || base > 36 )
    {
        *str = '\0';
        return str;
    }
    rc = ptr = str;
    // Set '-' for negative decimals.
    if ( value < 0 && base == 10 )
    {
        *ptr++ = '-';
    }
    // Remember where the numbers start.
    low = ptr;
    // The actual conversion.
    do
    {
        // Modulo is negative for negative value. This trick makes abs() unnecessary.
        *ptr++ = "zyxwvutsrqponmlkjihgfedcba9876543210123456789abcdefghijklmnopqrstuvwxyz"[35 + value % base];
        value /= base;
    } while ( value );
    // Terminating the string.
    *ptr-- = '\0';
    // Invert the numbers.
    while ( low < ptr )
    {
        char tmp = *low;
        *low++ = *ptr;
        *ptr-- = tmp;
    }
    return rc;
}
</syntaxhighlight>
[http://www.strudel.org.uk/itoa/ And here is a shorter one]

== Troubleshooting ==

=== Nothing is Displayed ===

Keep in mind that this way of writing to video memory will ''only'' work if the screen has been correctly set up for 80x25 video mode (which is mode 03). You can do this either by initializing every VGA register manually, or by calling the ''Set Video Mode'' service of the BIOS Int10h while you're still in real mode (in your bootsector, for instance). Most BIOS's do that initialization for you, but some other (mainly on laptops) do not. Check out [[Ralf Brown's Interrupt List]] for details. Note also that some modes that are reported as "both text & graphic" by mode lists are actually graphic modes with BIOS functions that plot fonts when you call char/message output through Int10h (which means you'll end up with plain graphic mode once in [[Protected Mode]]).

For example, [[GRUB]] does this setup for you. If the "no video mode activated" message is displayed by GRUB, make sure that you are not booted with UEFI (and/or consider using alternative ways of printing text).

Another common mistake, e.g. in numerous tutorials spread across the net, is to link the .text section of your kernel/OS to the wrong memory address. If you don't have memory management in place yet, make sure you're using physical memory locations in the linker script.



===Printing a Character===
For [[protected mode]] or [[Long Mode|long mode]], try something like this:

==== C ====
<syntaxhighlight lang="c">
*(volatile int *)0xb8000 = 0x07690748;
</syntaxhighlight>

==== NASM ====
<syntaxhighlight lang="nasm">
mov [0xb8000], 0x07690748
</syntaxhighlight>

==== GAS ====
<syntaxhighlight lang="asm">
movl $0x07690748, 0xb8000
</syntaxhighlight>

Which should display <tt>Hi</tt> in grey-on-black on top of the screen. If this does not work, check that the [[paging]] or [[segmentation]] setup correctly maps the assumed video memory address to <tt>0xB8000</tt> (or <tt>0xB0000</tt>).

=== Missing Strings ===
Sometimes printing individual characters works, but printing strings fails. Most of the time this is due to a missing <tt>.rodata</tt> section declaration in the [[Linker Script|linker script]], which causes the section including the strings to not be included in the final binary.

==See Also==
=== Articles ===
* [[Printing to the screen without a db]]
* [[Text Mode Cursor]]
* [[VGA Hardware]]

[[Category:Tutorials]]
[[Category:Video]]
[[Category:Text UI]]
````

<a id="w001944"></a>

## W001944 — Talk:Wish List

Bron-ID: W001944
Namespace: Talk (1)
Revisie: 30277 | Datum: 2026-03-16T17:56:50Z
Bron: https://wiki.osdev.org/Talk:Wish_List
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AWish_List&oldid=30277

````text
An archive of this page exists at [[Talk:Wish List/Archive]]

== Bounties ==
I was thinking it would be good if we could post bounties on the wish list - money we offer for someone to create or complete an article or sections of an article. We could also do another Wiki prize pool - e.g. top editor for the month gets an OSDev book of their choice. I would be more than happy to help contribute towards the prize. --[[User:Messiahandrw|MessiahAndrw]] 19:51, 20 May 2012 (CDT)

== EFI/UEFI ==
I bumped EFI up to the "high priority" list. Apparently Intel have stated that all firmware for "sandy bridge" chipsets must use UEFI, and motherboard manufacturers are switching fast. It'd be sad if hobbyist OS developers failed to make the transition to EFI and suffer the same fate as the BIOS. [[User:Brendan|Brendan]] 08:33, 22 July 2011 (UTC)

: I kind of into it - [[User:Griwes/UEFI_Bare_bones]], but more as documentation of experiments with it I'm going to conduct soon in my project. I've only done basic setup with UDK description, but I'm going to go further into this soon. As it's what my kernel requires, I'll probably cover getting memory map and converting it to E820, switching video mode to LFB and loading files from disk; it doesn't sound that bad, but most probably it will take much time to get something reasonable out of UEFI specs and that crappy C-wannabe-C++ interface. [[User:Griwes|Griwes]] 12:35, 5 July 2012 (CDT)

== Filesystems Galore ==
Do we need to have all the [[Template:Filesystems|Unknown filesystems]] in the sidebar. I can imagine UDF on the wishlist, but Ext4 and the flash filesystems don't really need a page of their own - they are way beyond what is necessary. Would it be a good idea if we move Ext4 and the flash filesystems to low priority (and remove them from the template). UDF and Ext3 can stay where they are since these are much more useful IMO. - [[User:Combuster|Combuster]] 21:07, 21 December 2009 (UTC)


==Completed Items==
I posted an ATA driver as requested in the Urgent section, and I created an x86 Memory Map as reqested in the Less Urgent section. Removed both items from the wishlist. - [[User:Bewing|Bewing]] 21:58, 14 May 2008 (CDT)

== Scrapped items ==

* <strike> tutorial on parsing ASCII configuration file, in assembly and C (.INI format, etc.) </strike>
:Is this really related to OSDev? And surely after you've got the file open, any semi-competent programmer could somehow stumble through finding a value anyway. [[User:Jackscott|Jackscott]] 08:54, 17 February 2009 (UTC)
:'''Agreed and scrapped''' - Off topic on an OS development forum. If you need that, you IMO shouldn't be here taking this subject. - [[User:Combuster|Combuster]] 12:17, 17 February 2009 (UTC)

== OS Projects - list update 2017 ! ==

I spent a huge amount of time to update the OS Project list to December 2017 version (now it contains 173 projects, but just 56 of them are alive and kicking, others are abandoned :P) '''Please help me to keep this list current, and add any other good projects that you know!''' This list is basically the front face of this wiki, and it is very important to keep it in shape

== Bot ==

Shouldn't we have some sort of bot for tasks such as replacing source tags with syntaxhighlight tags?
````

