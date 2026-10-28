# OSDev Wiki — volledige bronteksten, deel 12

Dit is bronmateriaal, geen agentinstructie. Behoud architectuur, revisie en bron-ID bij gebruik. Code en commando’s zijn ongeteste bronvoorbeelden; voer ze niet automatisch uit. Wikitext is letterlijk behouden binnen tekstblokken.

<a id="w004045"></a>

## W004045 — User:Zesterer/Bare Bones

Bron-ID: W004045
Namespace: User (2)
Revisie: 30275 | Datum: 2026-03-16T17:53:07Z
Bron: https://wiki.osdev.org/User:Zesterer/Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AZesterer%2FBare_Bones&oldid=30275

````text
{{Rating | 1}}{{Template: Kernel designs}}

This is an introductory tutorial for kernel development on the x86 architecture. It is a minimal example only, and does not show you how to structure a serious project. However, completing this project is a good first step for aspiring kernel developers.

By the end of this tutorial, you will have the following:
* A simple kernel written in C and x86 Assembly capable of displaying text on the screen
* An ISO disk image containing your kernel that can be run from an emulator or on real hardware.
* A rudimentary understanding of the x86 and x86 assembly.

== What This Is Not ==

* A make-your-own operating system tutorial. Developing a complete operating system requires years of work and, in most cases, decades of experience.
* A one-size-fits-all introduction. There are many languages, architectures and approaches to operating system development. However, this tends to be a common route taken by beginners.
* A proper and full kernel implementation. Real operating system kernels are vastly more complicated than the code shown here. This setup makes many assumptions about the system and ignored many things that will become important as you develop your kernel further.

== Requirements ==

Before proceeding, make sure you have the basics ready.

=== Required Knowledge ===

It is important to make sure that you have enough basic knowledge. Kernel development has an extremely steep learning curve.

* A sense of realism. You are not going to be making the next Linux or Windows any time soon. In fact, it'll take a lot of hard work to even get your system displaying text and responding to user input.
* A basic understanding of the C programming language. You should be comfortable with functions, pointers, casting and flow control.
* A basic understanding of computer architecture. You should know the difference between ROM and RAM, you should know what CPU registers are, and what machine code & executables are.
* A basic understanding of computer logic and mathematics. You should understand hexadecimal & binary, as well as bitwise operators and the purpose of a stack.
* Some basic command-line skills. You should understand how to run simple commands, navigate your filesystem and manipulate files.


'''If after reading this you feel like you have a few holes in your knowledge, I highly recommend reading [[Beginner Mistakes]] and related [[:Category:OS theory|OS theory]] first.'''

=== Required Tools ===

You will also require some software tools to develop this project.

* A UNIX-like operating system that supports operating system development well such as Linux or a UNIX-like environment (MinGW or Cygwin if you are using Windows).
* A text editor of some sort, preferably with syntax highlighting (it will make your life easier). I recommend the cross-platform [https://atom.io/ Atom editor].
* A copy of QEMU for the i386 architecture. This is only necessary if you want to test your kernel without real hardware.

== Some Background Knowledge ==

First, a little background information.

We will be developing our kernel for the x86 architecture. The x86 is a family of computer architectures first introduced in the 1970s. Most modern PCs are backwards-compatible with x86 however, so we should have no problem running our finished kernel on real hardware.

The x86 is a CISC (Complex Instruction Set Computer) architecture. This means that it has a large instruction set that we can manipulate in order to execute programs.

We will be compiling our kernel with our current operating system, with the intention of it running '''free-standing''' (i.e: independent of any other OS). For this reason, we will be using a '''cross-compiler''' to create our final executable. Cross-compilation is the first stage of writing a new operating system. After a huge amount of work, it is possible to make an operating system 'self-hosting'. This means that it is capable of compiling itself rather than relying on an existing OS. However, that is a long way down the road. For the foreseeable future, you will be cross-compiling your operating system.

We'll be writing the kernel in x86 assembly and the C programming language. When the x86 first loads up our kernel, it won't yet be in a fit state to run C code. This is why we must use assembly to first set up a basic C environment. Once this is done, we can write (most) of the rest of the kernel in C.

The x86 is a complex architecture with various different CPU states and modes. To avoid having to deal with them right now, we'll be using the GRUB bootloader to load our kernel into memory and set up a stable 32-bit 'protected-mode' environment.

To test our kernel, we'll be running it in QEMU. QEMU is an emulator that will allow us to test our kernel without rebooting real hardware to test every change we make.

== Building A Cross-Compiler ==
:''Main article: [[GCC Cross-Compiler]], [[Why do I need a Cross-Compiler?]]

The first thing we need to do is to build ourselves a cross-compiler toolchain to let us compile and link code for the x86. Until we have that, we can't do anything. A standard C compiler won't work: it doesn't understand how to generate the machine code we need for this project.

You can find instructions for building a GCC cross-compiler for the '''i686-elf''' target here: [[GCC Cross-Compiler]]

Don't attempt the rest of the tutorial without getting yourself the correct compiler. I can assure you: it won't work.

== The Code ==

So we know the theory, and we have our tools ready. What now? Code? Well... before putting finger to keyboard, it's a good idea for us to take a step back and think about what we have to work with.

=== Freestanding - What Does This Mean? ===

I've already mentioned that we're running our code ''''freestanding''''. But what does this mean, and how will it affect us? It's important you know. Normally when you write C code in a hosted environment, you have a plethora or interfaces available to you. You can read from files, you can output messages, you can get user input... All with just a few lines of code. Sadly, we don't have that. Those things are provided by an operating system. And right now, we don't have one since we '''ARE''' the operating system.

What we '''DO''' have access to however is a few useful headers GCC automatically provides us with (they give us things like fixed-width integers) and the hardware of the x86. We'd like to output text, so for that we're going to write ourselves a very simple driver that interacts with the x86's VGA buffer and allows us to display text on the screen. In doing this, we'll have to avoid using anything like the C standard library, because it simply isn't available when we're compiling for a freestanding target like we are now.

So without further ado... Let's start programming!

=== The Project Structure ===

Please remember, this is a minimal setup. A more advanced kernel project will have a more complex code structure, as well as an automated build system.

For now, we'll be creating 3 files in our project directory. They are:

<pre>
start.s     - This file will contain our x86 assembly code that starts our kernel and sets up the x86
kernel.c    - This file will contain the majority of our kernel, written in C
linker.ld   - This file will give the compiler information about how it should construct our kernel executable by linking the previous files together
</pre>

=== start.s ===

First, we create '''start.s'''. To best understand the code, I recommend typing it out by hand such that you can understand each part in detail.

<syntaxhighlight lang = "text">
// We declare the 'kernel_main' label as being external to this file.
// That's because it's the name of the main C function in 'kernel.c'.
.extern kernel_main
 
// We declare the 'start' label as global (accessible from outside this file), since the linker will need to know where it is.
// In a bit, we'll actually take a look at the code that defines this label.
.global start
 
// Our bootloader, GRUB, needs to know some basic information about our kernel before it can boot it.
// We give GRUB this information using a standard known as 'Multiboot'.
// To define a valid 'Multiboot header' that will be recognised by GRUB, we need to hard code some
// constants into the executable. The following code calculates those constants.
.set MB_MAGIC, 0x1BADB002          // This is a 'magic' constant that GRUB will use to detect our kernel's location.
.set MB_FLAGS, (1 << 0) | (1 << 1) // This tells GRUB to 1: load modules on page boundaries and 2: provide a memory map (this is useful later in development)
// Finally, we calculate a checksum that includes all the previous values
.set MB_CHECKSUM, (0 - (MB_MAGIC + MB_FLAGS))
 
// We now start the section of the executable that will contain our Multiboot header
.section .multiboot
	.align 4 // Make sure the following data is aligned on a multiple of 4 bytes
	// Use the previously calculated constants in executable code
	.long MB_MAGIC
	.long MB_FLAGS
	// Use the checksum we calculated earlier
	.long MB_CHECKSUM
 
// This section contains data initialised to zeroes when the kernel is loaded
.section .bss
	// Our C code will need a stack to run. Here, we allocate 4096 bytes (or 4 Kilobytes) for our stack.
	// We can expand this later if we want a larger stack. For now, it will be perfectly adequate.
	.align 16
	stack_bottom:
		.skip 4096 // Reserve a 4096-byte (4K) stack
	stack_top:
 
// This section contains our actual assembly code to be run when our kernel loads
.section .text
	// Here is the 'start' label we mentioned before. This is the first code that gets run in our kernel.
	start:
		// First thing's first: we want to set up an environment that's ready to run C code.
		// C is very relaxed in its requirements: All we need to do is to set up the stack.
		// Please note that on x86, the stack grows DOWNWARD. This is why we start at the top.
		mov $stack_top, %esp // Set the stack pointer to the top of the stack
 
		// Now we have a C-worthy (haha!) environment ready to run the rest of our kernel.
		// At this point, we can call our main C function.
		call kernel_main
 
		// If, by some mysterious circumstances, the kernel's C code ever returns, all we want to do is to hang the CPU
		hang:
			cli      // Disable CPU interrupts
			hlt      // Halt the CPU
			jmp hang // If that didn't work, loop around and try again.
</syntaxhighlight>

=== kernel.c ===

'kernel.c' contains our main kernel code. Specifically, it contains code for displaying text on the screen using the VGA textmode buffer.

<syntaxhighlight lang = "c">
// GCC provides these header files automatically
// They give us access to useful things like fixed-width types
#include <stddef.h>
#include <stdint.h>

// First, let's do some basic checks to make sure we are using our x86-elf cross-compiler correctly
#if defined(__linux__)
	#error "This code must be compiled with a cross-compiler"
#elif !defined(__i386__)
	#error "This code must be compiled with an x86-elf compiler"
#endif

// This is the x86's VGA textmode buffer. To display text, we write data to this memory location
volatile uint16_t* vga_buffer = (uint16_t*)0xB8000;
// By default, the VGA textmode buffer has a size of 80x25 characters
const int VGA_COLS = 80;
const int VGA_ROWS = 25;

// We start displaying text in the top-left of the screen (column = 0, row = 0)
int term_col = 0;
int term_row = 0;
uint8_t term_color = 0x0F; // Black background, White foreground

// This function initiates the terminal by clearing it
void term_init()
{
	// Clear the textmode buffer
	for (int col = 0; col < VGA_COLS; col ++)
	{
		for (int row = 0; row < VGA_ROWS; row ++)
		{
			// The VGA textmode buffer has size (VGA_COLS * VGA_ROWS).
			// Given this, we find an index into the buffer for our character
			const size_t index = (VGA_COLS * row) + col;
			// Entries in the VGA buffer take the binary form BBBBFFFFCCCCCCCC, where:
			// - B is the background color
			// - F is the foreground color
			// - C is the ASCII character
			vga_buffer[index] = ((uint16_t)term_color << 8) | ' '; // Set the character to blank (a space character)
		}
	}
}

// This function places a single character onto the screen
void term_putc(char c)
{
	// Remember - we don't want to display ALL characters!
	switch (c)
	{
	case '\n': // Newline characters should return the column to 0, and increment the row
		{
			term_col = 0;
			term_row ++;
			break;
		}
		
	default: // Normal characters just get displayed and then increment the column
		{
			const size_t index = (VGA_COLS * term_row) + term_col; // Like before, calculate the buffer index
			vga_buffer[index] = ((uint16_t)term_color << 8) | c;
			term_col ++;
			break;
		}
	}
	
	// What happens if we get past the last column? We need to reset the column to 0, and increment the row to get to a new line
	if (term_col >= VGA_COLS)
	{
		term_col = 0;
		term_row ++;
	}
	
	// What happens if we get past the last row? We need to reset both column and row to 0 in order to loop back to the top of the screen
	if (term_row >= VGA_ROWS)
	{
		term_col = 0;
		term_row = 0;
	}
}

// This function prints an entire string onto the screen
void term_print(const char* str)
{
	for (size_t i = 0; str[i] != '\0'; i ++) // Keep placing characters until we hit the null-terminating character ('\0')
		term_putc(str[i]);
}



// This is our kernel's main function
void kernel_main()
{
	// We're here! Let's initiate the terminal and display a message to show we got here.
	
	// Initiate terminal
	term_init();

	// Display some messages
	term_print("Hello, World!\n");
	term_print("Welcome to the kernel.\n");
}
</syntaxhighlight>

=== linker.ld ===

This file is a '''link script'''. It is used to define how bits of the final kernel executable will be stitched together. It allows us to specify alignment, address offset, and other such useful properties of parts of the code that we need when our kernel gets linked together into an executable. You don't normally do this when you're writing C code - the compiler will do this for you. But this an operating system kernel, so we need to explicitly define a lot of these things.

<syntaxhighlight lang = "text">
/* The bootloader will start execution at the symbol designated as the entry point. In this case, that's 'start' (defined in start.s) */
ENTRY(start)

/* Tell the linker part of the compiler where the various sections of the kernel will be put in the final kernel executable. */
SECTIONS
{
	/* Begin putting sections at 1 Megabyte (1M), a good place for kernels to be loaded at by the bootloader. */
	/* This is because memory below 1 Megabyte is reserved for other x86-related things, so we can't use it */
	. = 1M;

	/* We align all sections in the executable at multiples of 4 Kilobytes (4K). This will become useful later in development when we add paging */

	/* First put the multiboot header, as it's required to be near the start of the executable otherwise the bootloader won't find it */
	/* The Multiboot header is Read-Only data, so we can put it in a '.rodata' section. */
	.rodata BLOCK(4K) : ALIGN(4K)
	{
		*(.multiboot)
	}

	/* Executable code */
	.text BLOCK(4K) : ALIGN(4K)
	{
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
}

</syntaxhighlight>

== Compiling And Linking ==

Now comes the magic. We'll be compiling the code we've just written into '''object''' files (if you don't know what these are, you can think of them as flat-pack shelves from IKEA. They are blobs of compiled code packaged along with instructions explaining how they should be linked into a larger executable).

Once we've compiled the object files, we'll be using our linker (part of our cross-compiler toolchain) to link the object files together into the final kernel executable!

=== Compiling ===

To compile our code, we'll need to run the following commands:

<pre>
i686-elf-gcc -std=gnu99 -ffreestanding -g -c start.s -o start.o
i686-elf-gcc -std=gnu99 -ffreestanding -g -c kernel.c -o kernel.o
</pre>

This will create two object files named '''start.o''' and '''kernel.o''' ready for linking.

There are several parts to the above command:
* '''-std=gnu99''' tells the compiler to adhere to the C99 GNU standard. This gives us all of the abilities of C99, plus a bunch of useful extra things that the GNU developers added in for us.
* '''-ffreestanding''' tells the compiler to generate free-standing code (i.e: does not rely on an existing operating system to run).
* '''-g''' tells the compiler to add debugging symbols to the compiled code. As the kernel grows, it'll be increasingly useful to have a good way of debugging problems. It's best to start early.
* '''-c''' tells the compiler to generate just object files rather than compiled and linked executables.

=== Linking ===

Finally, to link the objects together into the final executable, we'll run the following command:

<pre>
i686-elf-gcc -ffreestanding -nostdlib -g -T linker.ld start.o kernel.o -o mykernel.elf -lgcc
</pre>

This command also has several unexplained parts:
* '''-nostdlib''' is used to specify that we aren't linking against a C standard library. This should be obvious, since we're running freestanding.
* '''-T <link-script>''' is used to specify our linker script, 'linker.ld'.
* '''-lgcc''' tells the linker to link against '''libgcc''' which the the built-in platform-independent library that gcc provides for us to deal with simple code that GCC can implicitly generate (i.e: moving and manipulating data, low-level maths operations, etc.)

== Running The Kernel With QEMU ==

After all that, the linker command should have produced a file named '''mykernel.elf'''. This is our kernel image. Congratulations! I would tell you to give yourself a pat on the back, but it's wise to wait until we've seen it actually working before doing that.

Do you remember how I said that we'd need to use the GRUB bootloader? We will need to for real hardware. But luckily for us, QEMU has the ability to read Multiboot kernels built-in, so we don't need to go through the hassle of attaching our kernel to GRUB.

To run your kernel with QEMU, you can use the following command (replace '''mykernel.elf''' with the name of your kernel file)

<pre>
qemu-system-i386 -kernel mykernel.elf
</pre>

If everything went to plan, you should see something like the following appear:

[[Image:Qemu-barebones-x86-c.png]]

If this isn't what you see, then I recommend going back through the code and checking you typed everything up correctly, ran the correct commands, and that you have a working cross-compiler. Please only ask for help on the forums or via IRC if you've double-checked your method.

== Running The Kernel With GRUB And Real Hardware ==

To attach your kernel to GRUB and test it on real hardware, you'll have to first have GRUB installed. We'll be using GRUB2, so make sure you have the latest version on your system.

We'll need to start by creating a new directory tree for our GRUB ISO build. I call it 'isoroot', but you can choose whatever name you want. Inside that directory, you'll need to create a directory called 'boot' and a directory below that called 'grub'. Create a GRUB configuration file called '''grub.cfg''' in 'isoroot/boot/grub'. You'll want to put the following in it, and then adjust the contents according to your directory structure and kernel name.

=== grub.cfg ===

<syntaxhighlight lang = "text">
menuentry "My Kernel" {
	multiboot /boot/mykernel.elf
	boot
}
</syntaxhighlight>

Once you have done this, copy your final kernel '''.elf''' executable that we linked previously into the 'isoroot/boot' directory.

Your final file structure should look roughly like this:

<pre>
├── isoroot
│   └── boot
│       ├── grub
│       │   └── grub.cfg
│       └── mykernel.elf
</pre>

This last command will take your GRUB directory, read it, and produce a final ISO at the end. Make sure you replace 'isoroot' with whatever your GRUB directory name is.

<pre>
grub-mkrescue isodir -o mykernel.iso
</pre>

Congratulations! If all went to plan, you've just produced your first ever working x86 kernel ISO disk image. This disk image will happily run on emulators like VMWare, QEMU, VirtualBox and Microsoft VirtualPC. If you want to run it on real hardware, simply flash it to a USB or disk drive like you would a Linux distribution LiveCD. For a good tutorial on how to do this, take a look at [[https://help.ubuntu.com/community/BurningIsoHowto Ubuntu: Burning ISO File Tutorial]].

Note: The x86 VGA textmode device we rely on in this tutorial is dependent on the existence of the BIOS. This ISO can only be booted properly from a UEFI system if it is in a BIOS compatibility mode (often referred to as 'legacy mode').

== What Now? ==

At this point, you may be wondering what else we can do. I'm sure you have grand visions of a working filesystem, a GUI (Graphical User Interface), or even your system running some of your favourite games. But now is not the time for grand visions: You would end up disappointed anyway. The path to writing a complete operating system is a long and confusing one that involves years of dedicated work. Keep your goals realistic, but feel free to play around with the code and test out new ideas.

With that in mind, here are a list of things you can probably achieve within at most a few days with a little work.

=== Add Color Support To Your Terminal ===

Color will improve the look of your kernel a lot. You'll need to find a sensible way of defining available colors in code (The VGA textmode buffer provides support for 16 colors in both the background and foreground of your text) and then changing them in the code. Color support will also be useful for displaying colorful error messages as your kernel grows in size (trust me: you'll spend a lot of time debugging).

=== Add ANSI Support To Your Terminal ===

ANSI (American National Standards Institute) has a standardised system of escape codes that modern computer terminals should understand. These escape codes allow a computer program to manipulate terminal color, text effects, clear the screen and move the text cursor around. You can find more details about ANSI here: [[https://en.wikipedia.org/wiki/ANSI_escape_code ANSI Escape Codes - Wikipedia]].

=== Add Scrolling To Your Terminal ===

Scrolling is a common feature in CLIs (Command-Line Interfaces). When text runs off the bottom of the screen, can scroll the rest of the text upwards to fit a new line below rather than simply loop back to the top of the screen as we do now. To achieve this, you'll need to shift all the characters in the VGA buffer upwards by one row.

=== Create A Build System For Your Kernel ===
:''Main article: [[Meaty Skeleton]]

At the moment, we compile and link our kernel by manually running several commands. As your kernel grows, this will no longer be a realistic way of doing things. You'll need to design and implement a build system that can build your kernel for you. Common kernel build systems use 'make', 'CMake', 'autoconf', 'SCons' or 'Tup'.

=== Call Global Constructors ===
:''Main article: [[Calling Global Constructors]]

In C and particularly C++, the compiler will often generate code that is meant to be run when the program starts. Currently, we don't have any means to run such code. As our kernel grows, we'll need to add support for this.

=== Going Further On x86 ===
:''Main article: [[Going Further on x86]]

There is a whole world of interesting things to explore through operating system development. To improve our kernel further, we will need to better understand the x86 and what hardware we have available.

[[Category:Bare bones tutorials]]
[[Category:C]]
[[Category:C++]]
````

<a id="w004046"></a>

## W004046 — User:Zesterer/BareBones

Bron-ID: W004046
Namespace: User (2)
Revisie: 20610 | Datum: 2017-03-29T03:55:34Z
Bron: https://wiki.osdev.org/User:Zesterer/BareBones
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AZesterer%2FBareBones&oldid=20610
Doorverwijzing: User:Zesterer/Bare Bones

````text
#REDIRECT [[User:Zesterer/Bare Bones]]
````

<a id="w004047"></a>

## W004047 — File:Qemu-barebones-x86-c.png

Bron-ID: W004047
Namespace: File (6)
Revisie: 20613 | Datum: 2017-03-29T05:23:06Z
Bron: https://wiki.osdev.org/File:Qemu-barebones-x86-c.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AQemu-barebones-x86-c.png&oldid=20613

````text
A screenshot of a successfully booted x86 bare-bones kernel.
````

<a id="w004048"></a>

## W004048 — Why function implementations shouldn't be put in header files

Bron-ID: W004048
Namespace: Main (0)
Revisie: 29754 | Datum: 2025-11-11T16:08:06Z
Bron: https://wiki.osdev.org/Why_function_implementations_shouldn%27t_be_put_in_header_files
Revisie-URL: https://wiki.osdev.org/index.php?title=Why_function_implementations_shouldn%27t_be_put_in_header_files&oldid=29754

````text
'''''This document is a side-bar to [http://forum.osdev.org/viewtopic.php?f=13&t=30771&p=264866 a thread in the General Programming forum], and explains some basic ideas of the history and use of include headers in languages which use them. It also covers material relating to [[linkers]] and [[Execution Loader|loaders]], specifically the historical origin of them and their relationship to compilers and [[assemblers]]. The original text is from a [http://forums.devshed.com/programming-42/simple-calculator-514904.html reply post by Schol-R-LEA] on the DevShed C++ forum. The wording is lighthearted and somewhat sarcastic, but it should be clear enough, and the subjects are important Required Information for anyone using C or C++, for OS dev or anything else.'''''

A big part of programming, especially in an object-oriented language like C++, is being able to use the same functions or objects in many different programs, so that you don't have to repeat the same code over and over again. While it is possible (and pretty easy in modern systems) to simply copy the code by hand all over the place, it makes all of your program source files very large and repetitious, and tends to be error prone. Also, if you have to fix something that is copied to hell and gone, you need to fix it everywhere you have it - which is ''really'' error prone.

One solution is to automate copying the source. This is basically what '''#include''' does: it sticks a copy of the included file into the source code at the point where it is included, verbatim. Whatever was in the included file will show up in the program where the '''#include''' directive was. In fact, most of the job of the preprocessor is the replace one piece of text with another one, then hand the result on to the compiler proper.

This is a bit better, but it still has problems. First off, if you include all of the source code, you end up having to compile all of the code in those source files you've included - again and again and again. This is a slow process even today; in the days of yesteryear, it would have made compiling "Hello, World!" an all day event. And since you would be including a lot of code that you don't actually end up using, the program size grows like a cancer in your disk space.

What you really want is a way to copy the program functions, objects, etc. without having to recompile them every time, and pick out only the parts you need. Enter the linkage editor, or linker as it's usually called. This can take different pieces of compiled programs ('object code') and tie them all together into a nice like executable package, adding only the parts you need. If you want to get really fancy, it can even wait until you actually run the program to link the really common parts, meaning that if you have five programs using a shared function, you only need one copy in memory (though you only want to do this with very common things, since indiscriminate dynamic linking can slow programs down a lot). Now you can have a ''library'', a file composed of pre-compiled object code that any program can use. A lot of the things which you probably think of as being part of C++ (e.g., the '''string''' and '''iostream''' classes, and in fact almost all of the the common functions programmers use regularly) are actually part of a standard library, not the core language.

(<strike>Hysterical</strike>Historical note: Linkers actually came before preprocessors, and in fact before compilers or even assemblers (the same is true of interpreters, curiously enough, though they were special-purpose ones for handling things like floating-point math and 'interpreted' a sort of bytecode). The term 'compiler' originally referred to what we now call a linker, because it compiled a list of routines to add to an executable binary image - this was back in the days when men were men and programs were written with toggle switches - because otherwise you'd spend a lifetime punching in a simple addition routine.)

Of course, there are still problems with this, the biggest one being that your source file doesn't know what is defined in the libraries and the other parts of your program until it's linked, meaning that it would have no way of knowing if you really meant to call '''pow(2, "foo")''' until the linker goes bonkers on you trying to find a function named ''''pow'''' that takes an integer and a character string as an argument - rather than that other one which takes two doubles. So we need to declare all of the functions etc. before we can use them, in every file that uses the functions... what a hassle...

But wait, we've still got '''#include'''! We can create a file of nothing but declarations - a ''header file'' - which solves the problem of having to add all of those tedious prototypes without having to repeat ourselves any more than we have to.

There are still some problems, mind you. If you try to get clever and stick some actual program code in a header file, and that header file is included in more than one place in a program, the linker will have a fit trying to figure out which function ''''quux(int, int)'''' the the real McCoy, even if the source code for them is the exact same thing. This means that header files have to be declarations and nothing but (there's a few weird exceptions involving things like macros - which are actually inserted into the source code directly by the preprocessor - and templates - which are a safer and less annoying type of macro, though C++ programmers tend to get sniffy about it when freaky Lisp hackers like me point that out - but that's getting ahead of ourselves). You can even get in trouble if you have the same header included in two places in the same source file, as you'd be declaring the same thing twice, which has led to some funny tricks involving the conditional compilation directives in (yet again) the pre-processor to make sure it can't happen.

There's also the matter of telling the linker itself where to get the different pieces. Most linkers have options for this, but if you have to write out the whole list of files to link together in a large project every time you try to compile it, your fingers will wear away to nubs, and your bound to screw it up about a quarter of the time anyway. So this, too, get automated away: ''makefiles'' (or ''project files'' in a lot of newer systems) are scripts telling the linker where to go and what to do when it gets there. They can also do a lot of the other scut work, like checking to see if a file has been changed (so that it doesn't compile it again) and cleaning up all of the temporary files after it's done.

But all of this amounts to nothing if a) the program is small enough to fit into a single file without making your eyes glaze over reading it, and b) you aren't going to be re-using the code anywhere else. In that case, you may as well just stick it all in one big file and be done with it - unless, of course, the whole point of the exercise is to show you about all the stuff I just rattled off to you.

For further information, consult your pineal gland. Or don't, see if I care fnord.

[[Category:Programming]]
````

<a id="w004049"></a>

## W004049 — User:Narke

Bron-ID: W004049
Namespace: User (2)
Revisie: 25154 | Datum: 2020-09-13T21:13:14Z
Bron: https://wiki.osdev.org/User:Narke
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANarke&oldid=25154

````text
I'm '''Konstantin Tcholokachvili''', versed in osdev.

* [[User:Narke/Languages|Programming languages for osdev]]
* [[User:Narke/Ring0OSPitfalls|Pitfalls of ring0 only OS design]]

* [http://datasheets.chipdb.org/Intel/x86/Intel%20Architecture/24201606.PDF Intel Multiprocessor Specification]
````

<a id="w004050"></a>

## W004050 — User:No92/Random Bits

Bron-ID: W004050
Namespace: User (2)
Revisie: 20860 | Datum: 2017-06-04T23:26:41Z
Bron: https://wiki.osdev.org/User:No92/Random_Bits
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANo92%2FRandom_Bits&oldid=20860

````text
==Warnings==
Random info on how to get tools to run without warnings or annoying features.

===OVMF===
EDK has a very weird build system. After cloning, it requires you to make the Base Tools and set up a target.txt configuration. In order to disable serial output cluttering your terminal you have to comment out the TerminalDxe module:
<source lang="bash">
make -C BaseTools
cp /path/to/target.txt Conf/target.txt
patch OvmfPkg/OvmfPkgX64.fdf < /path/to/OvmfPkgX64.fdf.patch
cd OvmfPkg
./build.sh
</source>
You'll find OVMF.fd at Build/OvmfX64/DEBUG_GCC5/FV/OVMF.fd.

target.txt may look something like this for a x86_64 4-threaded machine with GCC 5+ installed:
<source lang="text">
ACTIVE_PLATFORM       = OvmfPkg/OvmfPkgX64.dsc
TARGET_ARCH           = X64
TOOL_CHAIN_TAG        = GCC5
BUILD_TARGET          = RELEASE
MAX_CONCURRENT_THREAD_NUMBER = 9
</source>

OvmfPkgX64.fdf.patch looks like this for the EDK2 tree at c17956e0eedce299ac253ac40238ce90a5e623e0:
<source lang="diff">
--- OvmfPkg/OvmfPkgX64.fdf	2017-03-26 17:01:47.324258000 +0200
+++ OvmfPkg/OvmfPkgX64.fdf	2017-03-26 17:02:22.321889038 +0200
@@ -239,7 +239,7 @@
 INF  MdeModulePkg/Universal/Console/ConPlatformDxe/ConPlatformDxe.inf
 INF  MdeModulePkg/Universal/Console/ConSplitterDxe/ConSplitterDxe.inf
 INF  MdeModulePkg/Universal/Console/GraphicsConsoleDxe/GraphicsConsoleDxe.inf
-INF  MdeModulePkg/Universal/Console/TerminalDxe/TerminalDxe.inf
+# INF  MdeModulePkg/Universal/Console/TerminalDxe/TerminalDxe.inf
 INF  MdeModulePkg/Universal/DriverHealthManagerDxe/DriverHealthManagerDxe.inf
 INF  MdeModulePkg/Universal/BdsDxe/BdsDxe.inf
 INF  MdeModulePkg/Application/UiApp/UiApp.inf
</source>

===QEMU===
There are still dumbasses out there abusing stderr for debug output (why is this allowed and included in release 2.8.0?), so we have to patch it out.

<source lang="diff">
--- block/vvfat.c	2017-04-11 00:00:05.816645363 +0200
+++ block/vvfat.c	2017-04-11 00:02:40.890137141 +0200
@@ -1162,10 +1162,6 @@
     s->qcow_filename = NULL;
     s->fat2 = NULL;
     s->downcase_short_names = 1;
-
-    fprintf(stderr, "vvfat %s chs %d,%d,%d\n",
-            dirname, cyls, heads, secs);
-
     s->sector_count = cyls * heads * secs - (s->first_sectors_number - 1);

     if (qemu_opt_get_bool(opts, "rw", false)) {
</source>

Compile QEMU as always using configure and make.

Running QEMU properly™ requires some flags to be set:
<source lang="bash">
qemu-system-x86_64 -M accel=kvm:tcg -net none -serial stdio -vga std
</source>

Also, it's quite convenient that QEMU can treat directories in your file system as FAT disks:
<source lang="bash">
qemu-system-x86_64 -drive format=raw,file=fat:rw:hdd
</source>
In this case, <code>hdd</code> is the directory with your operating system's file system.

===YASM===
The latest release 1.3.0 has no flag to turn off the <code>segment register ignored in 64-bit mode</code> warning. Fix this by compiling from source. Nicely enough, it both adds the flag to turn it off and disables the warning by default (why did they bother creating the flag?)

The build system isn't documented (or the documentation is hidden in such a way that I can't find it), but it appears that it's based on cmake which does most of the work for you.
<source lang="bash">
cmake -DCMAKE_INSTALL_PREFIX="$HOME/opt/yasm" -Wno-dev .
make
make install
</source>

==Makefile==
Filtering out flags for specific files is easy. This is usually required for [[Stack Smashing Protector]]:
<source lang="make">
kernel/boot/ssp.c.o: CFLAGS := $(filter-out -fstack-protector-all,$(CFLAGS)) -fno-stack-protector
</source>
This line filters out the <code>-fstack-protector-all</code> flags, replacing it with <code>-fno-stack-protector</code>.

==VirtualBox==
This is nothing but a huge mess, but still something you might want to test your OS on. While endusers will mostly use the GUI, it actually makes a lot of sense to configure it from a bash script.

===Basic VM Management===
You'll be using <code>VBoxManage</code> the entire time.

Creating a VM is simple: you just pass it the name:
<source lang="bash">
VBoxManage createvm --name "MyOS" --register
</source>

Deleting should be just as simple, but there is no way to force deleting in the <code>rm -f</code> way. Therefore, we just suppress the output and run true on failure:
<source lang="bash">
VBoxManage unregistervm "MyOS" --delete 2> /dev/null || true
</source>

Next, you'll want to set some basic properties of the VM.
<source lang="bash">
VBoxManage modifyvm "MyOS" --memory 1024 --cpus 2
</source>For an extensive list of all supported options, look [https://www.virtualbox.org/manual/ch08.html#vboxmanage-modifyvm here].

You need something to boot from; but first, you have to configure your IDE controller:
<source lang="bash">
VBoxManage storagectl "MyOS" --add ide --name "IDE"
</source>

Next, you can attach your images. Attaching an ISO image is done like this:
<source lang="bash">
VBoxManage storageattach "MyOS" --storagectl "IDE" --port 0 --device 0 --medium /path/to/your/image.iso --type dvddrive
</source>

===Host Key Behavior===
The default host key is Right Control, which one might want to change. Setting this requires knowing the VirtualBox-specific keycodes:
{| {{wikitable}}
! Key
! VirtualBox Code
|-
| Left Ctrl || 65507
|-
| Left Alt || 65513
|-
| Right Ctrl || 65508
|-
| Super/Meta || 65515
|-
| Menu || 65383
|}

'''Note''': these are known to change between versions; these were tested with <code>VBoxManage 5.1.22r115126</code>. Also, setting this affects all VMs; remember that when booting up your other VMs.
<source lang="text">
VBoxManage setextradata global GUI/Input/HostKeyCombination 65508
</source>

You can also change keybinds. For instance, you'd do this in order to unset the Close action and map Power Off to Host+X:
<source lang="bash">
VBoxManage setextradata global GUI/Input/MachineShortcuts Close=,PowerOff=X
</source>

===Managing the GUI===
To disable the bottom Status Bar on a per-VM basis, run:
<source lang="bash">
VBoxManage setextradata "MyOS" GUI/StatusBar/Enabled false
</source>

Reordering the indicators can be done as well:
<source lang="bash">
VBoxManage setextradata "MyOS" GUI/StatusBar/IndicatorOrder HardDisks,OpticalDisks,FloppyDisks,Network,USB,SharedFolders,Display,VideoCapture,Features,Mouse,Keyboard
</source>

You can hide entire menus by their label:
<source lang="bash">
VBoxManage setextradata "MyOS" GUI/RestrictedRuntimeMenus View,Input,Devices,Debug,Help
</source>

Or you can hide members of a menu. For example, if you want to hide <code>Detach</code> from the Machine menu, do:
<source lang="bash">
VBoxManage setextradata "MyOS" GUI/RestrictedRuntimeMachineMenuActions Detach
</source>

===Video Modes===
If you use GRUB for video mode setting, you'll see that nothing happens. Apparently, VirtualBox requires you to define custom resolutions.
<source lang="bash">
VBoxManage setextradata "MyOS" CustomVideoMode1 1920x1080x32
</source>

Please note that you can define multiple resolutions; just name them CustomVideoMode2, CustomVideoMode3, and so forth.

===VM Shutdown===
If you want your script to wait for the VM to shut down, we have to loop until our VM is not in the list of running VMs:
<source lang="bash">
until [ -z "$(VBoxManage list runningvms | grep \"MyOS\")" ]
do
	sleep .5
done
</source>

==Multiboot 2==
While the tag structure is very useful, it's not perfect; but that's for another discussion. Sometimes, the spec is just blatantly wrong. According to the spec, the structure of the ELF-Symbols tag (type 9) is this:
<source lang="C">
struct mb2_elf {
	uint32_t	type;
	uint32_t	size;
	uint16_t	num;
	uint16_t	entsize;
	uint16_t	shndx;
	uint16_t	reserved;
	char		shdr[1];
};
</source>

However, when examining the section header table, it appears to be off by 4 bytes. Digging into the latest GRUB2 source reveals that either GRUB2 or the spec is wrong. In reality, the structure provided by GRUB2 looks like this:

<source lang="C">
struct mb2_elf {
	uint32_t	type;
	uint32_t	size;
	uint32_t	num;
	uint32_t	entsize;
	uint32_t	shndx;
	char		shdr[1];
};
</source>

Notice that the reserved member is gone and the 16-bit wide members are actually 32-bit. Sometimes, I do wonder why I'm still using broken software.

==PXE==
Unsurprisingly, this is a pain to set up if you start from zero.

===Setting up proxyDHCP===
I will assume that you have some sort of router that is serving DHCP to your network, which also happens to not support PXE. Unless that router is an old computer rocking pfSense, you'll have no way to configure PXE directly on the router. This is where proxyDHCP comes in: it allows you to keep the DHCP server on your network, while also serving PXE requests. For this, you'll have to know one IP address from the subnet you'll be serving. My local IPs, for instance, are in the format <code>192.168.1.xxx</code>. Find your local IP (usually by running <code>ifconfig</code> or <code>hostname -I</code>) and install dnsmasq, which will be serving the PXE requests. The configuration at <code>/etc/dnsmasq.conf</code> looks like this:
<source lang="text">
port=0
log-dhcp
dhcp-range=192.168.1.0,proxy
pxe-service=x86PC,"PXE boot",boot/grub/i386-pc/core
enable-tftp
tftp-root=/home/osdev/OS/hdd
</source>
'''Note:''' replace <code>192.168.1.0</code> with your PXE server and replace the tftp-root with your own. Do not, however, change <code>boot/grub/i386-pc/core</code>; we will need this later.

Save and restart the dnsmasq service.

===Setting up a working network directory===
cd into the tftp root. Here, we will be running GRUB; please note that running anything lower than GRUB 2.02 has produced numerous issues for me. First, we will create a basic netdir:
<source lang="bash">
mkdir -p boot/grub
grub-mknetdir --net-directory=boot --subdir grub
</source>

===Fixing everything===
... and stop here. This entire technology (or the implementation, to be more specific), is broken beyond belief. If you though this is a smooth ride, you'll be disappointed, because we have to redo 50% of the work GRUB just did for us.

For some reason, GRUB will require you to load every single module you use, even though most of them usually are pre-loaded. Even after 30 years of GNU they can't get their stuff straight, GRUB is looking in the wrong directory for the modules. As a workaround, symlink boot/grub/i386-pc to i386-pc. I'll leave this as an exercise to you.

At this point, we'll have to generate a two-line early config file.
<source lang="text">
set root=(tftp,192.168.1.xxx)
configfile boot/grub/grub.cfg
</source>
Replace <code>192.168.1.xxx</code> with the local IP of the machine running dnsmasq. All this does is set up the correct root and jumping to your normal <code>grub.cfg</code>.

Finally, recreate a proper core image. Run something like this:
<source lang="bash">
grub-mkimage -O i386-pc-pxe --output boot/grub/i386-pc/core.0 -c i386-pc/grub.cfg -p=boot/grub  tftp pxe net configfile
</source>
'''Note:''' make sure you do not drop the <code>.0</code> at the end of the output file and the double space before specifying the modules.

Finally, make sure to load the module for every command you run. For instance, if you use multiboot2, you will have to add
<source lang="text">
insmod multiboot2
</source>
to your normal grub.cfg.

===Score some bonus points ...===
... and combine this with [[wikipedia:Wake-on-LAN|Wake-on-LAN]]. See this as a challenge to write your own script to do this.
````

<a id="w004051"></a>

## W004051 — Binutils

Bron-ID: W004051
Namespace: Main (0)
Revisie: 22633 | Datum: 2018-09-09T23:02:02Z
Bron: https://wiki.osdev.org/Binutils
Revisie-URL: https://wiki.osdev.org/index.php?title=Binutils&oldid=22633

````text
{{In Progress}}
{{Stub}}

GNU Binary Utilities, commonly known as binutils is a suite of tools for creating and working with binary files. Instructions for cross compiling binutils can be found the [[GCC_Cross-Compiler#Binutils|GCC Cross Compiler]] page.

== Binutils Applications ==

Please note that this list is not complete.

{| {{wikitable}}
|-
| as || [[GAS|GNU Assembler]]
|-
| ld || [[LD|GNU Linker]]
|-
| ar || Archiver, used for creating libraries
|-
| objdump || Displays information about binary files
|}

== See Also ==
=== External Links ===
* [https://en.wikipedia.org/wiki/GNU_Binutils GNU Binutils on Wikipedia]

[[Category:Tools]]
````

<a id="w004052"></a>

## W004052 — User:A22347

Bron-ID: W004052
Namespace: User (2)
Revisie: 22526 | Datum: 2018-07-16T22:38:36Z
Bron: https://wiki.osdev.org/User:A22347
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AA22347&oldid=22526

````text
Hi, I'm A22347. I program in x86 Assembly, C, C++ and Python.

I am currently working on my operating system named Banana OS.
````

<a id="w004053"></a>

## W004053 — User:Ajxs

Bron-ID: W004053
Namespace: User (2)
Revisie: 26365 | Datum: 2021-09-01T23:03:21Z
Bron: https://wiki.osdev.org/User:Ajxs
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAjxs&oldid=26365

````text
I'm a professional software engineer currently located in Sydney, Australia.

To get an idea of what I'm working on check out my public [https://github.com/ajxs Github Profile].

Wiki pages of which I'm the principal author/maintainer:

* [[Ada]]
* [[Ada Runtime Library]]
* [[GNAT Cross-Compiler]]
* [[UART]]

Wiki pages I have made significant contributions to:

* [[Ada Bare Bones]]
* [[ELF]]
* [[Linker Scripts]]
* [[QEMU]]
* [[Serial Ports]]
````

<a id="w004055"></a>

## W004055 — CMake

Bron-ID: W004055
Namespace: Main (0)
Revisie: 20767 | Datum: 2017-05-06T12:06:58Z
Bron: https://wiki.osdev.org/CMake
Revisie-URL: https://wiki.osdev.org/index.php?title=CMake&oldid=20767
Doorverwijzing: CMake Build System

````text
#REDIRECT [[CMake Build System]]
````

<a id="w004056"></a>

## W004056 — User:Computerfido

Bron-ID: W004056
Namespace: User (2)
Revisie: 20771 | Datum: 2017-05-07T00:57:03Z
Bron: https://wiki.osdev.org/User:Computerfido
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AComputerfido&oldid=20771

````text
Hi! I don't know what to put on this page...
````

<a id="w004057"></a>

## W004057 — User:Lev

Bron-ID: W004057
Namespace: User (2)
Revisie: 20784 | Datum: 2017-05-08T14:47:19Z
Bron: https://wiki.osdev.org/User:Lev
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALev&oldid=20784

````text
Hi,

my OS is LevOS 7 at [http://github.com/levex/levos7 GitHub].

I spend most of my day on operating systems research, I've previously internet at Apple and at Red Hat.

Visit my blog: [http://kernelstuff.org/ Random Kernel Stuff]

My articles in draft:
[[User:Lev/Drafts/Simple Stack Trace]]
````

<a id="w004058"></a>

## W004058 — User:Lev/Drafts/Simple Stack Trace

Bron-ID: W004058
Namespace: User (2)
Revisie: 20785 | Datum: 2017-05-08T14:47:41Z
Bron: https://wiki.osdev.org/User:Lev/Drafts/Simple_Stack_Trace
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALev%2FDrafts%2FSimple_Stack_Trace&oldid=20785

````text
(To be merged into Stack Trace)
````

<a id="w004059"></a>

## W004059 — Generic Interrrupt Controller versions 3 and 4

Bron-ID: W004059
Namespace: Main (0)
Revisie: 20793 | Datum: 2017-05-11T00:15:34Z
Bron: https://wiki.osdev.org/Generic_Interrrupt_Controller_versions_3_and_4
Revisie-URL: https://wiki.osdev.org/index.php?title=Generic_Interrrupt_Controller_versions_3_and_4&oldid=20793
Doorverwijzing: Generic Interrupt Controller versions 3 and 4

````text
#REDIRECT [[Generic Interrupt Controller versions 3 and 4]]
````

<a id="w004060"></a>

## W004060 — Generic Interrrupt Controller

Bron-ID: W004060
Namespace: Main (0)
Revisie: 20795 | Datum: 2017-05-11T00:16:19Z
Bron: https://wiki.osdev.org/Generic_Interrrupt_Controller
Revisie-URL: https://wiki.osdev.org/index.php?title=Generic_Interrrupt_Controller&oldid=20795
Doorverwijzing: Generic Interrupt Controller

````text
#REDIRECT [[Generic Interrupt Controller]]
````

<a id="w004061"></a>

## W004061 — User:Geri/Dawn

Bron-ID: W004061
Namespace: User (2)
Revisie: 20805 | Datum: 2017-05-14T15:32:11Z
Bron: https://wiki.osdev.org/User:Geri/Dawn
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGeri%2FDawn&oldid=20805

````text
= Dawn =

Dawn operating system is an operating system for the SUBLEQ architecture. The conception of Dawn is to allow hardware developers (even hobbists) to design simple hardware to run it. Dawn is a high-level operating system with graphics user interface, multitasking, C compiler, and various programs (paint, text editor, games, etc). 


= Dawn hardware =

Dawn operating system have public specifications to do a hardware compatible with the operating system (start/help/hardware.txt). Dawn having unified IO to run, so Dawn compatible systems are staying compatible with each other. It requires no CPU instruction set to run, only one 64 bit instruction: SUBTRACT and LESS OR EQUAL (SUBLEQ, B-=A; if(B<=0) EIP=C; else EIP+=24). Dawn requires no DMA or interrupts, it supports SMP. Dawn supports traditional pheripheries like keyboard, mouse, touchscreen, 24 or 32 bit frame buffer, 7.1 sound in and output, camera, disk drives. Creating hardware for Dawn platform, or emulating Dawn is keeped as simply as it possible, hardwares and the OS is polling memory locations to communicate with each other. Dawn operating system is loaded from memory address 0 to maximum 256 mbyte (execution starts from 0). From 256 mbyte to 320 mbyte, the memory area is reserved for the hardware. Memory area above 320 mbyte is available for general usage.
````

<a id="w004063"></a>

## W004063 — USB Human Interface Devices

Bron-ID: W004063
Namespace: Main (0)
Revisie: 28137 | Datum: 2023-07-10T02:54:56Z
Bron: https://wiki.osdev.org/USB_Human_Interface_Devices
Revisie-URL: https://wiki.osdev.org/index.php?title=USB_Human_Interface_Devices&oldid=28137

````text
'''USB Human Interface Devices (HID)''' are devices that, like the name suggests, allow an interface that lets humans interact with the computer. Common examples include USB mice, USB keyboards, USB joysticks, and other such devices. The protocol used by USB HID devices is defined in the USB HID specification. Some chipsets support emulating USB keyboards and mice as standard PS/2 devices, but many chipsets don't. Thus, a USB HID driver is necessary in some PCs that may not have PS/2 ports at all. 

== Protocol ==
USB HID devices are mainly based on two protocols: the report protocol and the boot protocol. A report is a data structure that is sent to the host from the device, or can also be sent from the host to the device. When a device sends a report to the host, it normally contains status change information, such as a keypress, mouse movement, etc. When the host sends a report to the device, it normally contains commands for configuring the device, such as setting LEDs on a keyboard, for example. This protocol of course depends on the [[USB#USB_Device_Framework|standard USB framework]]. USB HID devices communicate using interrupt transfers, as they don't always transfer data, but when they do, they require very fast response from the software as well as the data transferred is normally small. Reports generally have two types, depending on the protocol type. Report protocols are based on the concept of "items," and their structures are defined in report descriptors. The boot protocol is much simpler, and follows standard structures for mice and keyboards. For simplicity's sake, this article will discuss the boot protocol only, for now at least. 

=== Detecting HID devices ===
HID devices have class/sub-class values of both zeroes in their device descriptors, and instead have the class/sub-class values valid in their interface descriptors. Keep in mind that interface descriptors cannot be manually requested, and must be acquired along with the configuration and endpoint descriptors. The class value that identifies a HID device in the interface descriptors is 3. The sub-class value in the interface descriptor can be either 1 to indicate the device supports the boot protocol, or zero to indicate the device only supports the report protocol. The protocol field in the interface descriptor as well determines it is a mouse or a keyboard. Specifically, 1 indicates the HID device is a keyboard, while 2 indicates the HID device is a mouse. 

=== "SetProtocol" request ===
Assuming a USB HID device supports the boot protocol, as explained in the section above, where is has a class value of 3 and a sub-class value of 1, the driver software can select the protocol to use. It uses the "SetProtocol" request to tell the device whether it wants to use the report protocol or the boot protocol. For simplicity's sake, this article will describe the boot protocol only, for now. To send the "SetProtocol" request, software sends a regular SETUP transaction to the device's control endpoint zero. The SETUP packet's request type would contain 0x21, the request code for "SetProtocol" is 0x0B, and the value field of the SETUP packet should contain 0 to indicate boot protocol, or 1 to indicate report protocol. The index and length fields both must be zero, as the index is unused and this request has no data stages. This command is only supported on device that support the boot protocol at all. After this command has been used, all reports sent from the device to the host will be either boot reports or regular reports, depending on the type the software requests. 

=== "GetReport" request ===
The software can request a report from a USB device using the control endpoint and the regular SETUP packet. The SETUP packet's request type would contain 0xA1, the request code for "GetReport" is 1, the SETUP packet's "value" field would contain 0x0100, to request an input packet with ID zero, and the length field would be the length of the data stage the host wishes to receive. The SETUP packet should be sent to the device endpoint 0 (control endpoint.) For keyboards, the data stage is normally 8 bytes, while for mice, the data stage has the first 3 bytes in a standard format, while the rest of it may be used by device-specific features. Receiving reports this way is recommended only to test that the device initialization was completed successfully, or such, and the "GetReport" request should not be used to poll the HID device for changes, as the SETUP and STATUS stages waste too much time. Instead, software should poll the HID using interrupt transfers, using the interrupt IN endpoint. 

=== Interrupt endpoint ===
It is generally recommended that the HID reports to the software using interrupt transfers, and that software should normally avoid the "GetReport" request mentioned above. Driver software should request the [[USB#CONFIGURATION|configuration descriptor]] for the HID device. A HID device must support at least one configuration. Software should then scan the [[USB#INTERFACE|endpoint descriptors]], searching for a descriptor that indicates an "interrupt IN" type, which is an endpoint that sends device data to the host using interrupt transfers. Software should save the 4-bit ID of the endpoint, as well as the 8-bit interval of the endpoint. The interval value encodes time in milliseconds (ms) in which timespace the software should poll for a report packet once. For example, if the interval value is 8, software should request a report from the device every 8 ms. If software requests a report too early, for example, after 6 ms, the device may send the same packet as before, or it may not send anything, and return NAK instead. If software requests a report after the timespan, say for example 9 ms, then the device will send the new packet. Software constantly polls USB HID devices using this described method. This is also a good chance to optimize your USB code, as the polling functions will be run hundreds of times per second. Specifically in Bochs, the interval value of the USB mouse is 10 ms, and so software polls the USB device 100 times per second. 

== USB keyboard ==
USB keyboards communicate with software using reports, just like other HID devices. USB keyboards are detected by having a class code of 3 and a protocol value of 1, in the interface descriptor. I will be describing the boot protocol here, for simplicity's sake, for now at least.

=== Report format ===
This report must be requested by the software using interrupt transfers once every interval milliseconds, and the interval is defined in the interrupt IN descriptor of the USB keyboard. The USB keyboard report may be up to 8 bytes in size, although not all these bytes are used and it's OK to implement a proper implementation using only the first three or four bytes (and this is how I do it.) Just for completion's sake, however, I will describe the full report mechanism of the keyboard. Notice that the report structure defined below applies to the boot protocol only.
{| width="70%" border="1"
|-
|'''Offset'''
|'''Size'''
|'''Description'''
|-
|0
|Byte
|Modifier keys status.
|-
|1
|Byte
|Reserved field.
|-
|2
|Byte
|Keypress #1.
|-
|3
|Byte
|Keypress #2.
|-
|4
|Byte
|Keypress #3.
|-
|5
|Byte
|Keypress #4.
|-
|6
|Byte
|Keypress #5.
|-
|7
|Byte
|Keypress #6.
|}

'''Modifier keys status:''' This byte is a bitfield, where each bit corresponds to a specific modifier key. When a bit is set to 1, the corresponding modifier key is being pressed. Unlike PS/2 keyboards, USB keyboards don't have "scancodes" for modifier keys. The bit structure of this byte is:

{| width="70%" border="1"
|-
|'''Bit'''
|'''Bit Length'''
|'''Description'''
|-
|0
|1
|Left Ctrl.
|-
|1
|1
|Left Shift.
|-
|2
|1
|Left Alt.
|-
|3
|1
|Left GUI (Windows/Super key.)
|-
|4
|1
|Right Ctrl.
|-
|5
|1
|Right Shift.
|-
|6
|1
|Right Alt.
|-
|7
|1
|Right GUI (Windows/Super key.)
|}

When software receives an interrupt and, for example, one of the Shift modifier keys are set to 1, software should use the scancode table for the shift modification to get the key from the scancode.

'''Reserved field:''' This byte is reserved by the USB HID specification, and thus software should ignore it.

'''Keypress fields:''' One keyboard report can indicate up to 6 keypresses. All these values are unsigned 8-bit values (unlike PS/2 scancodes, which are mostly 7-bit) which indicate the key being pressed. A reference on the USB scancode to ASCII character conversion table is in the bottom of the article.

=== Keypress mechanism ===
USB keyboards send interrupts when a key is pressed or released, just like a PS/2 keyboard. However, unlike PS/2 keyboards, USB keyboard does not have the concept of "make" and "break" scancodes. When a user presses a key, the interrupt comes in with a scancode value in one of the keypress fields. When a key is released, the corresponding keypress field is returned zero in the next packet. To illustrate this more clearly and to illustrate why there are more than one keypress scancode field, let's look at the following examples.
Assume the user pressed the "A" key, which is scancode 0x04. The returned interrupt packet would look like this:
 00 00 04 00 00 00 00 00

Notice the modifier keys are zero, because the user isn't pressing any. The reserved field is also zero, as recommended by the USB HID spec. The first keypress field contains 0x04, which corresponds to the "A" key. Now, let's assume the user lets go of the "A" key. The packet sent would look like:
 00 00 00 00 00 00 00 00

Now, let's assume the user pressed the "A" key, and then pressed the "B" key (scancode 0x05) without letting go of the "A" key:
 00 00 04 05 00 00 00 00

Notice how one interrupt packet is capable of transferring two keypresses together. Now let's assume the user presses the "C" key (scancode 0x06) without letting go of either "A" or "B" key:
 00 00 04 05 06 00 00 00

Now what if the user lets go of the "A" key but keeps pressing the "B" and "C" keys? The keyboard will indicate that "A" is no longer being pressed, and "B" and "C" will move towards the beginning of the packet:
 00 00 05 06 00 00 00 00

As is probably obvious now, the USB keyboard returns scancodes in order of which one was pressed first. As such, if the first keypress field is zero, there are no keys that send scancodes being pressed. If it is not zero, software can check the next fields, to see if another key is being pressed as well.

The concept of modifier keys is probably obvious, but just for completion, let's assume the user pressed the left shift key with the "X" key (scancode 0x1B). The interrupt packet sent will contain:
 02 00 1B 00 00 00 00 00
Notice that bit 1 (value 0x02) of the modifier field is set, to indicate that the the left shift key is being pressed.

There is also a "phantom condition" which you can think of as an overflow. A USB keyboard packet can indicate up to 6 keypresses in one transfer, but let's imagine someone dropped their keyboard and more than 6 keys were pressed in one time. The keyboard will enter the phantom condition, in which all reported keys will be of the invalid scancode 0x01. Modifier keys will still be reported however. Imagine that 8 keys (or any random number more than 6) are being pressed, and the right shift key is also being pressed. The packet sent would look like:
 20 00 01 01 01 01 01 01

Notice that the modifier keys are still being indicated, but the actual scancodes all return the phantom condition. There are other special scancodes besides the phantom condition: 0x00 indicates there is no scancode and no key being pressed, 0x01 indicates the phantom condition we just explained, 0x02 indicates the keyboard's self-test failed and 0x03 indicates an undefined error occured. Starting from 0x04 the scancodes are valid and correspond to "real" keys. IMHO, a device driver should ignore the phantom condition if it happens.

=== Auto-repeat ===
One thing that makes USB keyboards a pain is that there is no mechanism for auto-repeat and auto-repeat delays in hardware; this must be implemented entirely in software, unlike PS/2 keyboards. For an example auto-repeat delay of 500 milliseconds and an auto-repeat speed of 10 characters per second, when software becomes aware that a certain key is being pressed without being released constantly, it ignores all the key presses except the first one for 500 milliseconds (or whatever auto-repeat delay you like), and if this times out and the key is still being pressed, the driver will report 10 keypresses every 1 second, or whatever auto-repeat speed you want as well. This would make the user feel natural, just like you would when you press and hold a key; the first key appears on the screen, the computer waits a short delay, and then the keys keep coming up.

=== LED lamps ===
LED lamps are also handled in software, and according to the hardware, NumLock, CapsLock and ScrollLock are normal keys that send normal scancodes. The driver is responsible for manipulating the LED lamps when one of these keys are pressed.

To set the LED lamps, the driver sends a SetReport request to the device using a standard USB Setup Transaction, with a one-byte data stage. The setup packet's request type should contain 0x21, the request code for SetReport is 0x09. The value field of the setup packet contains the report ID in the low byte, which should be zero. The high byte contains the report type, which should be 0x02 to indicate an output report, or a report that is being sent from the software to the hardware. The index field should contain the interface number of the USB keyboard, which is the number present in the interface descriptor which indicated this device was a USB keyboard at all. The data stage should be 1 byte, which is a bitfield. This Setup Transaction should be transferred to control endpoint zero, which would work on all hardware. Other hardware may or may not support the optional interrupt OUT endpoint. If the hardware supports the interrupt OUT endpoint, you can just transfer the 1 byte data stage to the interrupt OUT endpoint, without the extra overhead of the SETUP stage and STATUS stage. If the hardware support the interrupt OUT endpoint, you should avoid the control endpoint when possible, as the interrupt OUT endpoint is faster and can be programmed with interrupt transfers instead of setup transfers. The format of the 1-byte data stage (for SETUP transaction) or 1-byte interrupt OUT transfer is shown below. When a bit is set to 1, the corresponding LED is turned on.

{| width="70%" border="1"
|-
|'''Bit'''
|'''Bit Length'''
|'''Description'''
|-
|0
|1
|Num Lock.
|-
|1
|1
|Caps Lock.
|-
|2
|1
|Scroll Lock.
|-
|3
|1
|Compose.
|-
|4
|1
|Kana.
|-
|5
|3
|Reserved, must be zero.
|}

== USB mouse ==
USB mice, just like any other HID device, communicate with the software using reports, which are sent via interrupt endpoints or can be manually requested with the "GetReport" request. USB mice have a protocol value of 2 in the interface descriptor. 

=== Report format ===
This report must be requested by the host using interrupt transfers once every interval milliseconds. Interval is defined in the interrupt IN descriptor of the USB mouse device. Only the first three bytes of the USB mouse report are defined. The remaining bytes, if existed, may be used for device-specific features. Software may request only three bytes in an interrupt transfer, and this will not cause an error even if the actual packet is larger. The table below defines the report format for USB mice operating using the boot protocol.
{| width="70%" border="1"
|-
|'''Offset'''
|'''Size'''
|'''Description'''
|-
|0
|Byte
|Button status.
|-
|1
|Byte
|X movement.
|-
|2
|Byte
|Y movement.
|}


'''Button status:''' This byte is a bitfield, in which the lowest three bits are standard format. The remaining 5 bits may be used for device-specific purposes.

{| width="70%" border="1"
|-
|'''Bit'''
|'''Bit Length'''
|'''Description'''
|-
|0
|1
|When set to 1, indicates the left mouse button is being clicked.
|-
|1
|1
|When set to 1, indicates the right mouse button is being clicked.
|-
|2
|1
|When set to 1, indicates the middle mouse button is being clicked.
|-
|3
|5
|These bits are reserved for device-specific features.
|}


'''X movement:''' This is a signed 8-bit integer that represents the X movement. Bit 7 (value 0x80) determines the sign of the value. When this value is negative, the mouse is being moved to the left. When this value is positive, the mouse is being moved to the right. Notice that unlike PS/2 mice, the movement values for USB mice are 8-bit signed integers and not 9-bit integers.

'''Y movement:''' This is also a signed 8-bit integer that represents the Y movement. When this value is negative, the mouse is being moved up. When the value is positive, the mouse is being moved down (towards the user.)

== USB Report Protocol ==
This is the complicated one supported by all HID class devices. Mouses, keyboards, joysticks...

The interface descriptor will contain an HID descriptor alongside the endpoint descriptors:
{| width="70%" border="1"
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
|HID Descriptor Type (0x22)
|- valign="top"
|align="center" |2
|bcdHID
|align="center" |2
|align="center" |BCD
|HID Class Specification Release Number in Binary-Coded Decimal (i.e, 1.10 is expressed as 110h).
|- valign="top"
|align="center" |4
|bCountryCode
|align="center" |1
|align="center" |ID
|Country code of localised hardware (0 if irrelevant. Mainly for keyboards).
|- valign="top"
|align="center" |5
|bNumDescriptors
|align="center" |1
|align="center" |Number
|Number of descriptors >= 1
|- valign="top"
|align="center" |3i+6
|bDescriptorType
|align="center" |1
|align="center" |Number
|Type of HID descriptor
|- valign="top"
|align="center" |3i+7
|wDescriptorLength
|align="center" |2
|align="center" |Number
|Length of HID Descriptor
|}
There is always one descriptor, a Report Descriptor, followed by n Optional Descriptors. The types and lengths only are in an array at the end of the overarching HID Descriptor.

'''HID Descriptor Types'''
{| border="1" cellpadding="2"
!Value
!Description
|-
|align="center" |21h
|HID Descriptor
|-
|align="center" |22h
|Report Descriptor
|-
|align="center" |23h
|Physical Descriptor
|}

To get the details of these descriptors, use the standard USB [[USB#GET_DESCRIPTOR|GET_DESCRIPTOR]], but with an interface recipient.
{| align="center" border="1" cellpadding="5"
!bmRequestType
!bRequest
!colspan="2" |wValue
!wIndex
!wLength
|- align="center"
|10000001b
|GET_DESCRIPTOR<br />6
|Descriptor Type
|Descriptor Index
|Interface Number
|Descriptor<br />Length
|}
Descriptor Index is 0 except for Physical Descriptors, where Index 0 will enumerate descriptor sets and their sizes.

===Country Codes===
{| border="1" cellpadding="2"
!Value
!Country
!Value
!Country
!Value
!Country
!Value
!Country
!Value
!Country
|-
|align="center" |00h
|''Not Supported''
|align="center" |08h
|French
|align="center" |10h
|Korean
|align="center" |18h
|Slovakia
|align="center" |20h
|UK
|-
|align="center" |01h
|Arabic
|align="center" |09h
|German
|align="center" |11h
|Latin American
|align="center" |19h
|Spanish
|align="center" |21h
|US
|-
|align="center" |02h
|Belgian
|align="center" |0Ah
|Greek
|align="center" |12h
|Netherlands
|align="center" |1Ah
|Swedish
|align="center" |22h
|Yugoslavia
|-
|align="center" |03h
|Canadian-Bilingual
|align="center" |0Bh
|Hebrew
|align="center" |13h
|Norwegian
|align="center" |1Bh
|Swiss/French
|align="center" |23h
|Turkish-F
|-
|align="center" |04h
|Canadian-French
|align="center" |0Ch
|Hungary
|align="center" |14h
|Persian
|align="center" |1Ch
|Swiss/German
|-
|align="center" |05h
|Czechia
|align="center" |0Dh
|International (ISO)
|align="center" |15h
|Poland
|align="center" |1Dh
|Switzerland
|-
|align="center" |06h
|Danish
|align="center" |0Eh
|Italian
|align="center" |16h
|Portuguese
|align="center" |1Eh
|Taiwan
|-
|align="center" |07h
|Finnish
|align="center" |0Fh
|Japan (Katakana)
|align="center" |17h
|Russia
|align="center" |1Fh
|Turkish-Q
|}

=== Report Descriptor ===
This is not a value table, length and content vary as required.

It's a sequential list of items. Items come in two basic types, short and long.
==== Short Item====
{| width="70%" border="1"
!Offset
!Field
!Size
!Type
!Description
|- valign="top"
|align="center" |0:0
|bSize
|align="center" |2 bits
|align="center" |Enum
|Size of optional data in bytes
|- valign="top"
|align="center" |0:2
|bType
|align="center" |2 bits
|align="center" |Enum
|Type of this descriptor
|- valign="top"
|align="center" |0:4
|bTag
|align="center" |4 bits
|align="center" |Number
|Function of the item
|- valign="top"
|align="center" |1
|Data
|align="center" |bSize
|align="center" |Data
|Item data
|}
Note that as a special case a bSize of 3 corresponds to 4 bytes.

'''bType''':
{| border="1" cellpadding="2"
!Value
!Description
|-
|align="center" |00h
|Main
|-
|align="center" |01h
|Global
|-
|align="center" |02h
|Local
|-
|align="center" |03h
|Reserved
|}
==== Long Item====
{| width="70%" border="1"
!Offset
!Field
!Size
!Type
!Description
|- valign="top"
|align="center" |0:0
|bSize
|align="center" |2 bits
|align="center" |Enum
|Size = 2
|- valign="top"
|align="center" |0:2
|bType
|align="center" |2 bits
|align="center" |Enum
|Type = 3 (Reserved)
|- valign="top"
|align="center" |0:4
|bTag
|align="center" |4 bits
|align="center" |Number
|1111b - Long
|- valign="top"
|align="center" |1
|bDataSize
|align="center" |1
|align="center" |Number
|Byte count of data
|- valign="top"
|align="center" |2
|bLongItemTag
|align="center" |1
|align="center" |Number
|Long item tag
|- valign="top"
|align="center" |3
|Data
|align="center" |bDataSize
|align="center" |Number
|Data
|}

=== Parsing ===
The report descriptor's items are parsed in a sequential manner. The parser is a state machine. A complete report descriptor may look something like this:
<source lang="C">
static const uint8_t hidReportDescriptor [] = 
{
    0x05, 0x01,    // UsagePage(Generic Desktop[1])
    0x09, 0x04,    // UsageId(Joystick[4])
    0xA1, 0x01,    // Collection(Application)
    0x85, 0x01,    //     ReportId(1)
    0x09, 0x01,    //     UsageId(Pointer[1])
    0xA1, 0x00,    //     Collection(Physical)
    0x09, 0x30,    //         UsageId(X[48])
    0x09, 0x31,    //         UsageId(Y[49])
    0x15, 0x80,    //         LogicalMinimum(-128)
    0x25, 0x7F,    //         LogicalMaximum(127)
    0x95, 0x02,    //         ReportCount(2)
    0x75, 0x08,    //         ReportSize(8)
    0x81, 0x02,    //         Input(Data, Variable, Absolute, NoWrap, Linear, PreferredState, NoNullPosition, BitField)
    0x05, 0x09,    //         UsagePage(Button[9])
    0x19, 0x01,    //         UsageIdMin(Button 1[1])
    0x29, 0x03,    //         UsageIdMax(Button 3[3])
    0x15, 0x00,    //         LogicalMinimum(0)
    0x25, 0x01,    //         LogicalMaximum(1)
    0x95, 0x03,    //         ReportCount(3)
    0x75, 0x01,    //         ReportSize(1)
    0x81, 0x02,    //         Input(Data, Variable, Absolute, NoWrap, Linear, PreferredState, NoNullPosition, BitField)
    0xC0,          //     EndCollection()
    0x05, 0x02,    //     UsagePage(Simulation Controls[2])
    0x09, 0xBB,    //     UsageId(Throttle[187])
    0x15, 0x80,    //     LogicalMinimum(-128)
    0x25, 0x7F,    //     LogicalMaximum(127)
    0x95, 0x01,    //     ReportCount(1)
    0x75, 0x08,    //     ReportSize(8)
    0x81, 0x02,    //     Input(Data, Variable, Absolute, NoWrap, Linear, PreferredState, NoNullPosition, BitField)
    0x75, 0x05,    //     ReportSize(5)
    0x81, 0x03,    //     Input(Constant, Variable, Absolute, NoWrap, Linear, PreferredState, NoNullPosition, BitField)
    0xC0,          // EndCollection()
};
</source>
Recall the descriptor types.

'''Main''' - This is added to the logical tree. There are 5 subtypes:
* ''Input'' - 1000 00 nn
* ''Output'' - 1001 00 nn
* ''Feature'' - 1011 00 nn
* ''Collection'' - 1010 00 nn
* ''End Collection'' - 1100 00 nn

'''Global''' - Adjusts the global state machine. New main items will inherit this state. This is useful where multiple axes are similar, for instance.
* ''Usage Page'' - 0000 01 nn
* ''Logical Minimum'' - 0001 01 nn
* ''Logical Maximum'' - 0010 01 nn
* ''Physical Minimum'' - 0011 01 nn
* ''Physical Maximum'' - 0100 01 nn
* ''Unit Exponent'' - 0101 01 nn
* ''Unit'' - 0110 01 nn
* ''Report Size'' - 0111 01 nn
* ''Report ID'' - 1000 01 nn
* ''Report Count'' - 1001 01 nn
* ''Push'' - 1010 01 nn
* ''Pop'' - 1011 01 nn

'''Local''' - Do not carry over to next main item.
* ''Usage'' - 0000 10 nn
* ''Usage Minimum'' - 0001 10 nn
* ''Usage Maximum'' - 0010 10 nn
* ''Designator Index'' - 0011 10 nn
* ''Designator Minimum'' - 0100 10 nn
* ''Designator Maximum'' - 0101 10 nn
* ''String Index'' - 0111 10 nn
* ''String Minimum'' - 1000 10 nn
* ''String Maximum'' - 1001 10 nn
* ''Delimiter'' - 1010 10 nn

Note that locals can apply to more than one control is a single item.

=== Physical Descriptors ===
To quote the HID specification:<br />
''"Note Physical Descriptors are entirely optional. They add complexity and offer very little in return for most devices. However, some devices, particularly those with a large number of identical controls (for example, buttons) will find that Physical Descriptors help different applications assign functionality to these controls in a more consistent manner. Skip the following section if you do not plan on supporting Physical Descriptors."''

== See also ==

=== Articles ===
* [[Universal Serial Bus]]

=== External links ===
* [http://www.usb.org/developers/hidpage/HID1_11.pdf USB HID specification]
* [https://github.com/haiku/haiku/tree/master/src/add-ons/kernel/drivers/input/usb_hid Haiku's USB HID implementation]
* [https://www.win.tue.nl/~aeb/linux/kbd/scancodes-14.html USB scancodes to ASCII characters conversion table]

[[Category:USB]]
[[Category:Human Interface Device]]
````

<a id="w004064"></a>

## W004064 — Intel Advanced Encryption Standard New Instructions

Bron-ID: W004064
Namespace: Main (0)
Revisie: 29083 | Datum: 2024-06-07T13:37:37Z
Bron: https://wiki.osdev.org/Intel_Advanced_Encryption_Standard_New_Instructions
Revisie-URL: https://wiki.osdev.org/index.php?title=Intel_Advanced_Encryption_Standard_New_Instructions&oldid=29083

````text
{{Template:Stub}}

You'll find these instructions on any mid-tier Intel CPU since about 2010 and any AMD CPU since about 2011. Support for these is indicated by checking bit 25 in ecx after calling [[CPUID]] with eax = 0x00000001.

== Overview ==
{| {{wikitable}}
! Instruction
! Description
|-
| <code>AESENC</code> || encrypt a round
|-
| <code>AESENCLAST</code> || encrypt last round
|-
| <code>AESDEC</code> || decrypt a round
|-
| <code>AESDECLAST</code> || decrypt last round
|-
| <code>AESIMC</code> || AES inverse mix columns
|-
| <code>AESKEYGENASSIST</code> || used for key expansion
|}

== See Also ==
=== Articles ===
* [[Wikipedia:AES_instruction_set|AES instruction set]]

=== External Links ===
* [https://software.intel.com/sites/default/files/article/165683/aes-wp-2012-09-22-v01.pdf Intel Paper], including C code samples
* [http://tab.snarc.org/posts/technical/2012-04-12-aes-intrinsics.html A blog post] showing an example implementation using GCC's builtins
````

<a id="w004065"></a>

## W004065 — User:Obiwac/AQUA

Bron-ID: W004065
Namespace: User (2)
Revisie: 20883 | Datum: 2017-06-07T21:56:34Z
Bron: https://wiki.osdev.org/User:Obiwac/AQUA
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AObiwac%2FAQUA&oldid=20883

````text

== AQUA OS ==

AQUA is a 32 bit operating system. I combines a nice, natural and speedy graphical user interface with a whole bunch of APIs for developers. It even comes bundled with its own scripting language, AQUA Script. There are also a lot of available drivers, like a serial driver, for communicating with micro controllers like the Arduino. It also has a 3D engine, a HTML parser, some apps to play around with, and soon, a virtual assistant. It will be streamlined with the user interface, not replacing it, but working together with it. An example of this, is machine learning. If you do some actions routinely, AQUA will learn, and try and automatically do some actions for you. An example of this is, that each day, you open up the drawing application at 7:30. Then, if you do this enough times around about 7:30, the AQUA will then prompt you, at 7:30, each day, asking if you want to open up the draw application.
````

<a id="w004066"></a>

## W004066 — Windows Subsystem for Linux

Bron-ID: W004066
Namespace: Main (0)
Revisie: 30177 | Datum: 2026-03-09T16:45:58Z
Bron: https://wiki.osdev.org/Windows_Subsystem_for_Linux
Revisie-URL: https://wiki.osdev.org/index.php?title=Windows_Subsystem_for_Linux&oldid=30177

````text
'''Windows Subsystem for Linux''' (or '''WSL''') is an official compatibility layer of [[Windows NT|Microsoft Windows]] which allows Windows users to use Linux applications without virtual machines or a third-party program like [[Cygwin]]. WSL includes a Linux-compatible kernel capable of running [[ELF]] executables, along with a Bash shell and standard Linux command-line tools.

Due to the ubiquity of Linux in programming (with OS Development being no exception) and the various programs and libraries which are only designed to run on Linux machines, WSL may prove useful to any Windows-using OS developers.

== Availability ==
Windows Subsystem for Linux is only available on 64-bit Windows 10 machines running the Anniversary Update or later (check by opening System>Settings>About).

Note that the way you use the WSL changes depending on what update you're on. See the [https://docs.microsoft.com/en-us/windows/wsl/about Microsoft Documentation on WSL] for details.

== Installing Windows Subsystem for Linux ==
To enable Windows Subsystem for Linux, follow these steps:
* Put your computer into Developer Mode. to do this:
** Open settings
** Select "System & Security"
** Select the "For Developers" tab
** Select the "Developer Mode" radio button
* Enable the Windows Subsystem for Linux feature via either the GUI or the Command Line
** '''GUI''':
*** Open the "Turn Windows features on or off" utility (try searching for it in Start)
*** Enable "Windows Subsystem for Linux (beta)"
*** Click OK
** '''Command Line''':
*** Open PowerShell
*** Run 
 Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Windows-Subsystem-Linux
** When prompted to reboot, do so.
* Go to the Windows Store and install your distro of choice.
** If installing Ubuntu, you do not want the Ubuntu package. That is Ubuntu 16.04, and is far too outdated to be useful, as it uses GCC 5. Ubuntu 18.04 is now available with the more modern GCC 7, and GCC 8 on apt.

Assuming no errors are given, Windows Subsystem for Linux should now be installed.

== Using Windows Subsystem for Linux ==

* To start your distro of choice, there are two options:
** '''GUI'''
*** Create a start manager shortcut to your distro and click that
** '''Command Line'''
*** Type "wsl" in the command line. This will launch your default distros default shell.
**** Note that you can change the default distro using wslconfig, and the default shell through normal means.
* Run your programs and utilities.
** Note that you can run windows programs from the WSL, including from the windows PATH, but you have to append the file extension to them.
*** Ex: notepad.exe will work.
*** script.py will also work, if script.py is on your windows path and has the appropriate shebang. This will run using the distro Python.
**** python.exe script.py will not work, as thats not a full path. where script.py will reveal the full path, which then works.
**** Interactive windows apps such as this work less than well from the WSL at the time of writing.

At this point, you have access to the wide array of command line utilities for Linux. But maybe you want more, maybe you want GUI apps too.

GUI apps are officially unsupported by the WSL at the moment, but for the most part they work fine if you install an X server such as VcXsrv.

After installing an X server, you simply export DISPLAY=0 and GUI apps will start working usually. Notably, QEMU works quite well in this manner if you don't want to install it on windows for whatever reason. You may also wish to install a terminal emulator to improve the WSL experience. Windows tabbed terminal emulators are.. lacking to say the least, but installing something such as Terminator works quite well.


== See Also ==

=== External Links ===
* [[wikipedia:Windows_Subsystem_for_Linux|Windows Subsystem for Linux]] on Wikipedia
* [https://docs.microsoft.com/en-us/windows/wsl/about Microsoft Documentation on WSL]
* [https://www.reddit.com/r/Windows10/comments/8dnyig/using_a_gui_with_wsl_on_windows_10_ubuntu_wsl/ Using a GUI with WSL on Windows 10 (Ubuntu wsl)] on Reddit
* [https://www.reddit.com/r/bashonubuntuonwindows/comments/6ysgn4/guide_to_xfce4_install_in_wsl_for_advanced_noobs/ Guide to XFCE4 install in WSL for advanced noobs.] on reddit

[[Category:Compatibility Layers]]
[[Category:Windows]]
[[Category:Linux]]
````

<a id="w004067"></a>

## W004067 — Problems Booting From USB Flash

Bron-ID: W004067
Namespace: Main (0)
Revisie: 29079 | Datum: 2024-06-07T13:37:19Z
Bron: https://wiki.osdev.org/Problems_Booting_From_USB_Flash
Revisie-URL: https://wiki.osdev.org/index.php?title=Problems_Booting_From_USB_Flash&oldid=29079

````text

== Overview ==

Unfortunately, there are multiple potential problems when trying to boot from USB flash (and "flash like") devices, mostly caused by historical concerns combined with a lack of adequate standardisation.


== History ==

=== Ancient Computers ===

Originally USB didn't exist and no computer had firmware that was able to boot from any USB device.

When USB was created various manufacturers made (ISA and PCI) USB controller cards, and these cards were plugged into computers that don't support USB. In theory, if a standard describing how "boot from USB" was supposed to work existed these cards could have included device ROMs that hook/replace the BIOS "int 0x13" and made it possible to boot from USB, but there was no standard so none of the USB controller cards could do that.

=== Very Old Computers ===

Eventually firmware vendors realised it'd be nice to be able to boot from USB. Unfortunately there was still no standard describing how "boot from USB" should work, so different firmware vendors made different choices. Some decided that because USB is a removable device (unlike hard drives at the time) it should act like a floppy disk. Some decided that because USB is (potentially) much larger than a floppy disk it should act like a hard disk. Some decided to support both and provide a setting in their BIOS configuration so that half the time it doesn't work until the end user figures out why and finds/changes the BIOS setting.

=== Old Computers ===

It didn't take too long before firmware vendors started inventing schemes to try to auto-detect whether a USB device should behave like floppy or like hard drive. Their intentions were good (trying to avoid end user hassle), but there was no standard describing how "auto-detect how to boot from USB" was supposed to work so their implementations were hideous hacks that were incompatible between firmware vendors.

Some checked for a "valid looking" BIOS Parameter Block (BPB) in the MBR and decided it should behave like a floppy if there was a BPB (and behave like a hard disk if there was no BPB). Some checked for a "valid looking" partition table in the MBR and decided it should behave like a hard disk if there was a partition table (and behave like a floppy disk if there was no partition table). Some checked for a "valid looking" BPB and a "valid looking" partition table and made "unknown" decisions if there was neither or both (possibly including deciding that the device is not bootable when there's no BPB and not partition table).

=== Modern Computers ===

==== El Torito ====

Some USB flash device vendors have decided that (some) USB flash devices should also be able to emulate a USB CD drive (e.g. as part of the "U3 specification", where firmware sees something that looks like a USB hub with a USB flash device and a USB CD device attached). This adds another layer of complications to the previous "auto-detect how to boot from USB" schemes (deciding if the computer should boot from USB flash or from "same USB flash emulating a CD").

==== EFI/UEFI Compatibility ====

When EFI/UEFI was created a new partitioning scheme ([[GPT]]) was added and the way an OS boots (for all devices) was changed. For most cases firmware expects to find a FAT volume (e.g. the EFI System Partition) containing a boot loader file and loads the entire file (not just one sector).

However, a lot of computers support BIOS and UEFI; and some of these computers have another layer of "auto-guesswork hackery" to try to decide if a USB device should be booted using UEFI or BIOS. This mostly involves detecting if the device is partitioned using GPT and/or has something that looks like a (FAT) EFI System Partition and/or has a boot loader within that partition.


== Other Problems ==

=== Firmware Bugs ===

For some unknown reason; some computers will check for a "valid looking" MBR (with an active/boot partition), and then load that boot sector from within that partition and execute it without ever executing any code in the MBR.

=== Microsoft Disk Signatures ===

At some point (starting with Windows NT version 3.5); Microsoft decided they felt like adding a "unique" 32-bit disk signature to the MBR (and keep track of the signatures for every disk that Windows had seen in the registry). They decided to use the 4 bytes at offset 0x01B8 in the MBR for this disk signature, and decided that if the bytes in this location are not "unique" it would be fine for Windows to modify them, even if the device is never used by Windows, even if the device has no partitions that Windows is able to mount, even if it trashes executable MBR code for (older) competing operating systems, even if it destroys TPM checks done by competing operating systems, even though it conflicts with "prior usage" for competing commercial operating systems (PTS-DOS and DR-DOS), and even though (for partitioned devices) the MBR (and any sectors before the first partition) should be considered "owned by a boot manager that is not part of any OS" (and therefore an OS should assume it lacks the authority to tamper with the MBR without explicit consent from a user/admin).

In addition, there is no (known to me) documentation/standard that describes how these "unique" 32-bit signatures are supposed to be generated. This means that when 2 or more operating systems are used on the same computer they can "fight" (disagree on what the signature should be), causing the signature set by one OS to be trashed by another OS. Note that (to avoid problems caused by what I'd call "Microsoft's malicious disregard for other operating systems") Microsoft's disk signature was added to the UEFI specification via. a brief/inadequate description saying that it "may be used by the OS" (but not saying which OS may use it, or how).

== Booting Reliability ==

To increase the chance that an OS on a USB flash device will be booted properly by the BIOS on an unknown computer; the OS has to take into account all of problems described above. This means:

* The MBR must have a partition table with an active partition with the boot loader starting in the active partition (in case the firmware doesn't support "floppy emulation")
* The MBR must have code to "chain-load" the boot loader in the active partition (in case the firmware doesn't fail to execute the MBR's code and decides to treat the device like a hard drive)
* It should have a "DOS 3.31 BPB" (a BIOS Parameter Block including the 4-byte "large total logical sectors" field at offset 0x020) in case the firmware doesn't like the partition table
* The MBR code should be prepared to handle both cases (treated as floppy or treated as hard disk by BIOS)
* The MBR code must not use the 4 bytes at offset 0x01B8 in the MBR for anything
* The device may (optionally) also have El Torito tables to support being treated as a CD (but can't assume that the firmware will use it if the device is USB flash).
* The device may (optionally) also support UEFI such that the device is bootable on both BIOS and UEFI systems (e.g. with a "hybrid GPT" and/or with El Torito tables)

''Note: If these guidelines are followed, the MBR's code will be limited to the 404-byte area starting at offset 0x024 and ending at offset 0x01B7 (excluding the initial "jmp" at offset 0x000).''

[[Category:Booting]]
[[Category:Troubleshooting]]
[[Category:USB]]
````

<a id="w004068"></a>

## W004068 — Inline Functions in C

Bron-ID: W004068
Namespace: Main (0)
Revisie: 21003 | Datum: 2017-07-25T20:42:03Z
Bron: https://wiki.osdev.org/Inline_Functions_in_C
Revisie-URL: https://wiki.osdev.org/index.php?title=Inline_Functions_in_C&oldid=21003

````text
In C you can define inline functions for optimization purposes.
The problem is that the linker will fail (at least with GCC, with an undefined reference error) if no optimization option (e.g. -O2) is given to the compiler.
That is because by default, with no optimization, the compiler doesn't inline functions, and as the compiler doesn't create named references or entry points for their separate code, the linker won't find them.
Here is a possible solution that does not require defining the functions twice, and works for every optimization level. This is done by defining them in the header and declaring them as extern inline in the implementation file.

== Solution 1 ==

'''File: example1.h'''
<source lang="c">
inline long something(long i)
{
  return i + 2;
}
</source>

'''File: example1.c'''
<source lang="c">
#include "example1.h"
extern inline long something(long i);
</source>

This way, the compiler inlines the function if possible, but additionally allocates a name or reference that points to the specific implementation, in case a call is needed when inlining is not possible.

== Solution 2 ==

'''File: example2.h'''
<source lang="c">
static inline long something(long i)
{
  return i + 2;
}
</source>

This is another solution, adding the static keyword.

[[Category:C]]
````

<a id="w004069"></a>

## W004069 — Timer Interrupt Sources

Bron-ID: W004069
Namespace: Main (0)
Revisie: 30009 | Datum: 2026-02-15T17:24:53Z
Bron: https://wiki.osdev.org/Timer_Interrupt_Sources
Revisie-URL: https://wiki.osdev.org/index.php?title=Timer_Interrupt_Sources&oldid=30009

````text
This page gives an overview of various [[:Category:Timers|timers]] and their capabilities, advantages and disadvantages.

== Overview ==
{| class="wikitable"
|-
! 
! Architecture
! Per CPU Core
! Counter
! Calibration Not Required
! Fixed Frequency IRQ
! IRQ on terminal count
|-
| [[RTC]]
| x86 PC
| {{No}}
| {{No}}
| {{Yes}}
| {{Yes}}
| {{No}}
|-
| [[PIT]]
| x86 PC
| {{No}}
| {{No}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
|-
| [[ACPI Timer|ACPI Power Management Timer]]
| x86 PC
| {{No}}
| {{Yes}}
| {{Yes}}
| {{No}}
| {{No}}
|-
| [[HPET]]
| x86 PC
| {{No}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
|-
| [[TSC]]
| x86 PC
| {{Yes}}
| {{Yes}}
| {{No}}
| {{No}}
| {{No}}
|-
| [[APIC timer|Local APIC timer (older)]]
| x86 PC
| {{Yes}}
| {{No}}
| {{No}}
| {{Yes}}
| {{Yes}}
|-
| [[APIC timer|Local APIC timer (newer)]]
| x86 PC
| {{Yes}}
| {{Yes}}
| {{No}}
| {{No}}
| {{Yes}}
|-
| [[ARM Generic Timer|Generic Timer]]
| ARM
| {{Yes}}
| {{Yes}}
| {{Yes}}
| Configurable
| {{Yes}}
|-
| [[ARM Local Timer|Local Timer]]
| ARM
| {{No}}
| {{No}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
|-
| [[BCM System Timer]]
| ARM/BCM
| {{No}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
|}

== Timers (x86) ==

=== RTC ===

Discovery: Standard (assumed to exist)

Calibration: Not necessary

Access speed: Slow ISA IO ports

Counter: Not really (1 Hz if you use RTC's date/time)

Fixed frequency IRQ: Yes, up to 8 KHz (125 us) becoming unreliable at faster frequencies

IRQ on terminal count: No

=== PIT ===

Discovery: Standard (assumed to exist, but may be emulated by HPET)

Calibration: Not necessary

Access speed: Slow ISA IO ports (or slow SMM emulation of ISA ports)

Counter: No

Fixed frequency IRQ: Yes, up to 596.591 KHz (1.676 us)

IRQ on terminal count: Yes, 828 ns precision

=== ACPI Power Management Timer ===

Discovery: ACPI tables (FADT)

Calibration: Not necessary

Access speed: IO ports (may or may not be "faster, non-ISA IO ports")

Counter: Yes, 24-bit or 32-bit, monotonically increasing at 3.579545 MHz

Fixed frequency IRQ: No

IRQ on terminal count: No

=== HPET ===

Discovery: ACPI tables (HPET)

Calibration: Not necessary

Access speed: Fast (typically memory mapped device)

Counter: Yes, 64-bit monotonically increasing at 10 MHz or better

Fixed frequency IRQ: Yes, up to 10 Mhz or better (100 ns or better)

IRQ on terminal count: Yes, 100 ns or better precision

=== TSC ===

Discovery: CPUID

Calibration: Yes

Access speed: Very fast (built into CPU)

Counter: Yes, 64-bit monotonically increasing at CPU's clock speed (which can vary on old CPUs)

Fixed frequency IRQ: No

IRQ on terminal count: No

=== Local APIC timer (older) ===

Discovery: MP specification table, ACPI tables (APIC/MADT)

Calibration: Yes

Access speed: Very fast (built into CPU)

Counter: No

Fixed frequency IRQ: Yes, depends on CPU's bus/link speed, typically 100 Mhz or better (10 ns or better)

IRQ on terminal count: Yes, depends on CPU's bus/link speed, typically 10 ns or better

=== Local APIC timer (newer, with TSC deadline mode) ===

Discovery: MP specification table, ACPI tables (APIC/MADT)

Calibration: Yes

Access speed: Very fast (built into CPU)

Counter: Yes, 64-bit monotonically increasing at CPU's clock speed (which can vary on old CPUs)

Fixed frequency IRQ: No

IRQ on terminal count: Yes, depends on CPU's clock speed, typically 1 ns or better precision

== Timers (ARM) ==

=== Generic Timer ===

Discovery: always present

Calibration: read CNTP_FREQ system register

Access speed: Very fast (built into CPU)

Counter: Yes, CNTP_CVAL and CNTV_CVAL system registers

Fixed frequency IRQ: Configurable (either CPU clock or fixed crystal clock)

IRQ on terminal count: Yes

=== Local Timer ===

Discovery: always present

Calibration: No

Access speed: Moderate (using MMIO)

Counter: No

Fixed frequency IRQ: Yes, 38.4 MHz

IRQ on terminal count: Yes

=== System Timer ===

Discovery: on BCM2835-2837 SoC (Raspberry Pi)

Calibration: No

Access speed: Moderate (using MMIO)

Counter: Yes

Fixed frequency IRQ: Yes

IRQ on terminal count: Yes

== Precision ==
For counters; ignoring "emulated with something else", the options in order of best precision are: TSC, HPET, ACPI power management timer

For fixed frequency IRQ; ignoring "emulated with something else", the options in order of best precision are: local APIC timer, HPET, PIT, RTC

For IRQ on terminal count (e.g. needed for "tickless"); ignoring "emulated with something else", in order of best precision are: local APIC timer in TSC deadline mode, local APIC timer without TSC deadline mode, HPET, PIT

== Emulation ==

Counters can be emulated in software using a fixed frequency IRQ (e.g. doing "tick++;" in the IRQ handler).

Fixed frequency IRQ can be emulated in software using IRQ on terminal count (e.g. just set the new count to the same value each time).

IRQ on terminal count can be emulated in software using a fixed frequency IRQ (e.g. doing "count--; if(count == 0)" in the IRQ handler).

== Notes ==

A "counter" is something you poll when you need to know how much time has passed (e.g. for file system timestamps, measuring how much CPU time each thread consumed, etc).

For local APIC timer, there's one per CPU. This is important for scalability (rather than many CPUs fighting to access the same single timer).

A good OS would detect which timers exist and determine their capabilities; then use this information to select the best timers to use for each different purposes.

== See also ==

=== Threads ===

* https://forum.osdev.org/viewtopic.php?p=277285#p277285: Original Post by Brendan

[[Category:Timers]]
[[Category:Interrupts]]
[[Category:X86]]
[[Category:ARM]]
````

<a id="w004070"></a>

## W004070 — Building libgcc for mcmodel=kernel

Bron-ID: W004070
Namespace: Main (0)
Revisie: 28490 | Datum: 2023-12-05T12:35:18Z
Bron: https://wiki.osdev.org/Building_libgcc_for_mcmodel%3Dkernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Building_libgcc_for_mcmodel%3Dkernel&oldid=28490

````text
{{Rating|2}}
In [[X86-64|x86_64]], if you link (e.g. through a linker script) your kernel directly to the higher half of the virtual memory, you might want to build the kernel with <tt>-mcmodel=kernel</tt> or <tt>-mcmodel=large</tt>.

Also, if you link with <tt>libgcc</tt> and use <tt>crtbegin.o</tt> and <tt>crtend.o</tt> (see [[Calling_Global_Constructors|Calling Global Constructors]]), you might wand to build <tt>libgcc</tt> with <tt>-mcmodel=kernel</tt> or <tt>-mcmodel-large</tt>.

Otherwise, you may get errors when linking, either from <tt>crtstuff.c</tt> or from kernel code, like:

<source lang="text">
.../crtbegin.o: In function `deregister_tm_clones':
crtstuff.c:(.text+0x2): relocation truncated to fit: R_X86_64_32 against symbol `__TMC_END__' defined in .dtors section in ...
crtstuff.c:(.text+0x1d): relocation truncated to fit: R_X86_64_32 against `.tm_clone_table'
.../crtbegin.o: In function `register_tm_clones':
crtstuff.c:(.text+0x31): relocation truncated to fit: R_X86_64_32 against symbol `__TMC_END__' defined in .dtors section in ...
crtstuff.c:(.text+0x5f): relocation truncated to fit: R_X86_64_32 against `.tm_clone_table'
.../crtbegin.o: In function `__do_global_dtors_aux':
crtstuff.c:(.text+0x88): relocation truncated to fit: R_X86_64_32 against symbol `__DTOR_END__' defined in .dtors section in ...
.../crtend.o
crtstuff.c:(.text+0x8e): relocation truncated to fit: R_X86_64_32 against `.dtors'
crtstuff.c:(.text+0xdb): relocation truncated to fit: R_X86_64_32 against `.eh_frame'
.../crtbegin.o: In function `frame_dummy':
crtstuff.c:(.text+0x10c): relocation truncated to fit: R_X86_64_32 against `.bss'
crtstuff.c:(.text+0x111): relocation truncated to fit: R_X86_64_32 against `.eh_frame'
.../crtend.o: In function `__do_global_ctors_aux':
crtstuff.c:(.text+0x13): relocation truncated to fit: R_X86_64_32 against `.ctors'
collect2: error: ld returned 1 exit status
</source>

One solution is to build a specific cross compiler, used only for kernel compilation (different from the one used for the userland), with <tt>libgcc</tt> (where <tt>crtbegin.o</tt> and <tt>crtend.o</tt> are generated, from <tt>crtstuff.c</tt>) compiled, like the kernel, with <tt>-mcmodel=kernel</tt> or <tt>-mcmodel=large</tt>. As far as we can tell, you could even mix the two modes, e.g.: compile the kernel with <tt>-mcmodel=kernel</tt> and <tt>libgcc</tt> with <tt>-mcmodel=large</tt> or the other way around.

In this case we also use <tt>-mno-red-zone</tt> to compile <tt>libgcc</tt> because it may be needed for the kernel (see [[Libgcc_without_red_zone|Libgcc without red zone]]).

Afterwards, you can compile and link your kernel with the generated cross compiler using <tt>-mcmodel=kernel</tt> or <tt>-mcmodel=large</tt>, and <tt>-mno-red-zone</tt>, and link with <tt>crtbegin.o</tt> and <tt>crtend.o</tt>.

These recipes were tested and work at least for GCC 6, 7 and 8.

Starting from the [[GCC_Cross-Compiler|GCC Cross-Compiler]] recipe, and using <tt>TARGET=x86_64-elf</tt>, we can proceed as in the next sections.

== Compiling libgcc with -mcmodel=kernel ==

This may require a workaround. First compile <tt>binutils</tt> as usual, then add the binaries to the <tt>PATH</tt>, and start to compile <tt>gcc</tt>. When compiling <tt>libgcc</tt> with <tt>-mcmodel=kernel</tt>, it will fail. Then patch the <tt>Makefile</tt> to disable <tt>PIC</tt>, repeat and continue:

<source lang='bash'>
# (...)
mkdir build
cd build
../gcc-*/configure --target=$TARGET --disable-nls --enable-languages=c,c++ --without-headers --prefix=$PREFIX
make all-gcc
make all-target-libgcc CFLAGS_FOR_TARGET='-g -O2 -mcmodel=kernel -mno-red-zone' || true
# will fail with: cc1: error: code model kernel does not support PIC mode
sed -i 's/PICFLAG/DISABLED_PICFLAG/g' $TARGET/libgcc/Makefile
make all-target-libgcc CFLAGS_FOR_TARGET='-g -O2 -mcmodel=kernel -mno-red-zone'
make install-gcc
make install-target-libgcc
</source>

== Compiling libgcc with -mcmodel=large ==

First compile <tt>binutils</tt> as usual, then add the binaries to the <tt>PATH</tt>, and start to compile <tt>gcc</tt>. Then compile <tt>libgcc</tt> with <tt>-mcmodel=large</tt>:

<source lang='bash'>
# (...)
mkdir build
cd build
../gcc-*/configure --target=$TARGET --disable-nls --enable-languages=c,c++ --without-headers --prefix=$PREFIX
make all-gcc
make all-target-libgcc CFLAGS_FOR_TARGET='-g -O2 -mcmodel=large -mno-red-zone'
make install-gcc
make install-target-libgcc
</source>

== See Also ==
=== Articles ===
* [[GCC_Cross-Compiler|GCC Cross-Compiler]]
* [[Libgcc|libgcc]]
* [[Libgcc_without_red_zone|libgcc without red zone]]
* [[Calling_Global_Constructors|Calling Global Constructors]]

[[Category:Compilers]]
````

<a id="w004072"></a>

## W004072 — Talk:Real Mode OS Warning

Bron-ID: W004072
Namespace: Talk (1)
Revisie: 23214 | Datum: 2019-01-16T06:59:51Z
Bron: https://wiki.osdev.org/Talk:Real_Mode_OS_Warning
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AReal_Mode_OS_Warning&oldid=23214

````text
Can this be added to the Main Page / Introduction category? -[[User:FallenAvatar]]
````

<a id="w004073"></a>

## W004073 — User:Gravaera/ARM AMBA Advanced Peripheral Bus

Bron-ID: W004073
Namespace: User (2)
Revisie: 21029 | Datum: 2017-08-01T05:38:11Z
Bron: https://wiki.osdev.org/User:Gravaera/ARM_AMBA_Advanced_Peripheral_Bus
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FARM_AMBA_Advanced_Peripheral_Bus&oldid=21029

````text
This page seeks to provide a quick refresher reference on the ARM AMBA (Advanced Microcontroller Bus Architecture's) APB (Advanced Peripheral Bus). This is a low-power bus, meant to host low performance microcontrollers, but be able to also interface with the higher powered AHB-Lite (Advanced High-performance Bus) and the AXI (Advanced eXtensible Interface). APB is optimized for low power consumption and reduced interface complexity.

At the time of writing, this page is current with AMBA APB version 3.

= Logical description =

** The APB transitions lines from HIGH/LOW or LOW/HIGH on the HIGH phase of the clock.
** Each transfer takes at least 2 cycles.
** APB uses an unpipelined logical protocol.

= Physical layer description =

APB has a total of 7 lines.

** PCLK:
** PADDR:
** PWRITE:
** PSEL:
** PENABLE:
** PWDATA:
** PREADY:

APB version 3 adds the PSLVERR line to indicate an error condition during transfer.

** PSLVERR: As of APBv3. It is recommended (not required) that PSLVERR be driven low while it is not being sampled. PSLVERR can be asserted for both read and write transactions. PSLVERR is only sampled on the last cycle of an APB transfer. Failed transactions may or may not have altered the state of the peripheral. This is peripheral specific. PSLVERR does not mean that a write transaction has not updated the peripheral. PSLVERR means that read transactions may return invalid data; in particular, there is no requirement for failed transactions indicated by PSLVERR to return all 0s from a read.

= Protocol description =

APB slaves can extend APB transfers by using the PREADY signal. The slave does this by driving PREADY low during the ACCESS phase when PENABLE is high. As a result of PREADY being held low by the slave, the PADDR, PWRITE, PSEL and PENABLE signals remain unchanged for the additional (extended) cycles during which PREADY is held low. When the transfer is done, the slave drives PREADY high. This transfer extension facility therefore is not for pausing/controlling the master's transfer rate, but for indicating when a transfer is over.

State diagram:

[[File:apbstatediagram.png]]

= Signals =

A write transaction is indicated by PWRITE being held HIGH.
A read transaction is indicated by PWRITE being held LOW.

= Mapping to AHB =

PSLVERR is mapped to the HRESP=ERROR AHB signal state for both read and write transactions on the AHB. This is achieved by directly mapping APB's PSLVERR to AHB's HRESP[0].

= Mapping to AXI =

PSLVERR is mapped to RRESP/BRESP=SLVERR AHB signal state. This is achieved by mapping the PSLVERR to RRESP[1] for reads and to BRESP[1] for writes.
````

<a id="w004074"></a>

## W004074 — File:Apbstatediagram.png

Bron-ID: W004074
Namespace: File (6)
Revisie: 21028 | Datum: 2017-08-01T05:37:19Z
Bron: https://wiki.osdev.org/File:Apbstatediagram.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AApbstatediagram.png&oldid=21028

````text
APB state transitions
````

<a id="w004075"></a>

## W004075 — User talk:Thecool1kevin

Bron-ID: W004075
Namespace: User talk (3)
Revisie: 21035 | Datum: 2017-08-02T22:01:08Z
Bron: https://wiki.osdev.org/User_talk:Thecool1kevin
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AThecool1kevin&oldid=21035

````text
Hi
````

<a id="w004076"></a>

## W004076 — User talk:Thecool1kevin/printf

Bron-ID: W004076
Namespace: User talk (3)
Revisie: 21036 | Datum: 2017-08-02T22:01:25Z
Bron: https://wiki.osdev.org/User_talk:Thecool1kevin/printf
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AThecool1kevin%2Fprintf&oldid=21036

````text
Hello, everyone. This is where you get some free code of questionable quality to get you on your feet in OSDev!
To use the following snippets, you need a memset(), a strlen() and a putc(), to put characters to the screen.
Take what you like, as these are very simple implementations and some are of sub par quality. Free food for all (this is the closest you'll ever get to free code without credit).

Some downsides:
* Negative width is not supported, nor are doubles and floats.
* Kind of hard to maintain tbh.

The upsides:
* Free code to get you started.
* Portable and easy to use (for the most part)

The code provided here should by no means stay in your OS or even be part of your OS for very long. They are just meant to be starting points for
formatted texts. Free code can sometimes bite y'know?

== NOTE ON -lgcc ==
Everything below has to be linked with the -lgcc option! See "How to link with libgcc" in [[Libgcc]].
Basically:
<pre>
i686-elf-gcc -T linker.ld -o myos.kernel -ffreestanding boot.o kernel.o -nostdlib -lgcc
</pre>


== strings.h ==
<source lang="c">
// Implement these yourself. The wiki has plenty of resources!
void* memset (void*, uint32_t, size_t);
size_t strlen(const char* str);
// Implementation provided below
int strtol(char a[]);
</source>

== kprintf.h ==
<source lang="c">
// Implementations of these are going to be provided below
int kprintf(const char* format, ...);
int fprintf(uint8_t file, const char* format, va_list args);
</source>

== int strtol(char[] a) ==
<source lang="c">
int strtol(char a[])
{
    int n = 0;
    for (int c = 0; c < strlen(a); c++)
        n = n * 10 + a[c] - '0';
    return n;
}
</source>

== kprintf.c ==
<source lang="c">
/*
 * Filename: kprintf.c
 * Author:   Kevin Dai
 * Email:   kevindai02@outlook.com
 *
 * Created on 31-Jul-2017 02:10:59 PM
 *
 * @ Last modified by:   Kevin Dai
 * @ Last modified time: 02-Aug-2017 12:52:38 PM
 */

// Have fun maintaining the code!

#include "kprintf.h"
// #include "a bunch of other stuff"
const char __base1[16] = { '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F' };
const char __base2[16] = { '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f' };

// There used to be wrapper methods above hardware implementation details that we do not need to worry about
void kputc(uint8_t file, const char c)
{
    /* TODO: Implement a put character to screen */
}

int kprintstr(uint8_t file, const char* str)
{
    for(uint32_t i = 0; i < strlen(str); i++)
        kputc(file, str[i]);
    return strlen(str);
}

// Our own secret code reusability thing that takes a bijillion arguments because I hate myself
// Look, this is bad code and I keep on sticking new arguments in but its so repeated that I thought why
// not stick it into some giant blob of disgustingness and forget about it?
int _convbase(uint8_t file, long long num, uint8_t base, bool small, bool isSigned, int width, char pad, char foo, bool isOct, char oct)
{
    char ret[23]; memset(ret, 0, 23); // 22 for 64 bit octal, +1 for EOL
    if(num == 0)
        ret[0] = '0';
    else
    {
        if(isSigned)
        {
            long long divres = num < 0 ? -num : num; // Absolute value
            for(int i = 0; divres > 0; i++) // Do base conversion
            {
                if(!small)
                    ret[i] = __base1[divres % base];
                else
                    ret[i] = __base2[divres % base];
                divres /= base;
            }
        }
        else
        {
            unsigned long long divres = (unsigned long long) num;
            for(int i = 0; divres > 0; i++) // Do base conversion
            {
                if(!small)
                    ret[i] = __base1[divres % base];
                else
                    ret[i] = __base2[divres % base];
                divres /= base;
            }
        }
    }

    if(strlen(ret) > 1)
    {
        uint8_t i = 0;
        uint8_t j = strlen(ret) - 1;

        // Reverse the string
        while (i < j)
        {
            char temp = ret[i];
            ret[i] = ret[j];
            ret[j] = temp;
            i++; j--;
        }
    }

    // Deal with the padding
    if((signed) strlen(ret) + foo < width) // Deal with padding
    {
        for(uint32_t i = 0; i < (unsigned)(width - strlen(ret) - foo); i++)
            kputc(file, pad);
        if(isOct)
            kputc(file, oct);
        kprintstr(file, ret);
        return width;
    }
    else
    {
        if(isOct)
            kputc(file, oct);
        return kprintstr(file, ret);
    }
}

/* ==================================== THINGS THAT ARE EXPOSED TO THE REAL WORLD ==================================== */

/**
 * Kernel print format to the normal output stream.
 * @param  format  The formatted text, "%[flags][width][length]specifier" is supported
 * @param  VARARGS Arguments to go with the formatted text
 * @return         Length of text written. Will be negative if error occured.
 */
int kprintf(const char* restrict format, ...)
{
    va_list args;
    va_start(args, format);
    int ret = fprintf(_NORM, format, args);
    va_end(args);

    return ret;
}

/**
 * File print format for the kernel.
 * @param  file    IO stream id: _NORM, _ERR, _INFO
 * @param  format  Formatted text, "%[flags][width][length]specifier" is supported
 * @param  VARARGS Arguments to go with the formatted text
 * @return         Length of text written. Will be negative if error occured.
 */
int fprintf(uint8_t file, const char* restrict format, va_list args)
{
    int written = 0;

    while(*format != '\0')
    {
        if(written > INT_MAX) return -1; //TODO: Errnos
        // Parse the format baby!
        if (*format == '%')
        {
            // Initial values (the format properties)
            bool flags[4] = { false, false, false, false };
            int width = -1;
            char length = 0;
            char specifier = 0;

            char tmp[11]; int _i = 0;
            memset(tmp, 0, 11);

            format++; // Let the parsing begin!
            while (specifier == 0)
            {
                switch (*format) // Parse the formats now
                {
                    // All of our flags. Here, we need to allow multiple flags to be accepted.
                    // The if statements ensure our formats are in the right order
                    case '0': // Handle the corner case with numbers
                        if (tmp[0] != 0 && width == -1 && length == 0 && specifier == 0 && _i < 10) { tmp[_i] = '0'; _i++; }
                        else if (!flags[0] && width == -1 && length == 0 && specifier == 0) flags[0] = true;
                        else return -1;
                        break;
                    case '+': if (!flags[1] && width == -1 && length == 0 && specifier == 0) flags[1] = true; else return -1; break;
                    case ' ': if (!flags[2] && width == -1 && length == 0 && specifier == 0) flags[2] = true; else return -1; break;
                    case '#': if (!flags[3] && width == -1 && length == 0 && specifier == 0) flags[3] = true; else return -1; break;

                    // Most of our widths
                    case '*':
                        if (width == -1 && length == 0 && specifier == 0) width = va_arg(args, int); else return -1;
                        break;
                    case '1': case '2': case '3': case '4': case '5': case '6': case '7': case '8': case '9':
                        if (width == -1 && length == 0 && specifier == 0 && _i < 10) { tmp[_i] = *format; _i++; }
                        else return -1;
                        break;

                    // All of our lengths
                    case 'h': case 'l':
                        if (length == 0 && specifier == 0) length = *format; else return -1;
                        break;

                    // All of our specifiers. Formats end when the specifier is specified :)
                    case 's': case 'x': case 'X': case 'u': case 'o': case 'c': case '%': case 'd': case 'i':
                        if (specifier == 0) specifier = *format; else return -1;
                        break;

                    default: return -1;
                }
                format++; // Move on in life
            }

            // Turn the string we collected into an actual number
            if(width == -1 && tmp[0] == 0) // If no width was specified...
                width = 0;
            else if(width == -1 && tmp[0] != 0) // If a width was specified in the format
            {
                if(strtol(tmp) < INT_MAX)
                    width = strtol(tmp); // Convert it
                else // Width too large
                    return -1;
            }

            // Now that all the information on the format has been collected, we need to print it on screen
            if(specifier == 's')
            {
                const char* str = va_arg(args, const char*);
                if(strlen(str) < (unsigned) width) // Deal with padding
                {
                    for(uint32_t i = 0; i < (unsigned)(width - strlen(str)); i++)
                        kputc(file, ' ');
                    kprintstr(file, str);
                    written += width;
                }
                else
                    written += kprintstr(file, str);
            }
            else if(specifier == 'c')
            {
                const char c = (char) va_arg(args, const int);
                if(1 < (unsigned) width) // Deal with padding
                {
                    for(uint32_t i = 0; i < (unsigned)(width - 1); i++)
                        kputc(file, ' ');
                    kputc(file, c);
                    written += width;
                }
                else
                {
                    kputc(file, c);
                    written++;
                }
            }
            else if(specifier == '%')
            {
                kputc(file, '%');
                written ++;
            }
            else if(specifier == 'x' || specifier == 'X' || specifier == 'u' || specifier == 'o')
            {
                unsigned long long val = 0;
                // Collect the argument based on the length specifier
                if(length == 'h')
                    val = (unsigned long long)((unsigned short int) va_arg(args, uint32_t)); // char promotes to int
                else if(length == 'l')
                    val = va_arg(args, unsigned long long);
                else
                    val = (unsigned long long) va_arg(args, uint32_t);

                // Organize our information etc.
                char pad = ' ';
                if(flags[0]) pad = '0';
                uint8_t base = 10;
                if(specifier == 'x' || specifier == 'X') // Base 16 number (hex)
                    base = 16;
                else if(specifier == 'u') // Base 10 (decimal)
                    base = 10;
                else // Base 8 (octal)
                    base = 8;

                // Deal with prefixes now
                char foo = 0;
                if(specifier == 'x' && flags[3])      { written += kprintstr(file, "0x"); foo += 2; }
                else if(specifier == 'X' && flags[3]) { written += kprintstr(file, "0X"); foo += 2; }
                else if(specifier == 'o' && flags[3]) foo++;

                // Write it!
                written += _convbase(file, val, base, specifier != 'X', false, width, pad, foo, (specifier == 'o' && flags[3]), '0');
            }
            else if(specifier == 'd' || specifier == 'i')
            {
                long long val = 0; // Fetch the values
                if(length == 'h')
                    val = (long long)((short int) va_arg(args, long));
                else if(length == 'l')
                    val = va_arg(args, long long);
                else
                    val = (long long) va_arg(args, int);

                if(val < 0 && flags[1]) val = -val; // Make positive
                // Deal with the signage
                char pad = ' ';
                if(flags[0]) pad = '0';
                char foo = 0;

                if(val < 0) { foo++; written++; } // Negative sign
                if(val < 0 && flags[0]) kprintstr(file, "-");
                if(val >= 0 && flags[2]) { foo++; written += kprintstr(file, " "); } // Space for positive

                // Write it!
                written += _convbase(file, val, 10, false, true, width, pad, foo, (val < 0 && !flags[0]), '-');
            }

        }
        // If we are not parsing a format
        else
        {
            kputc(file, *format);
            format++;
            written++;
        }
    }

    return written;
}
</source>

== Unit testing (ish) ==
<source lang="c">
void __test_kprintf__()
{
    // Test case 1:
    kprintf("[1]: Hello, world!\n");
    kprintf("[2]: %s %x %X %u %o %c %% %d %i \n", "Hello, world!", 0xDEADBEEF, 0xCAFEBABE, 10, 8, 'c', 'd', -10, 0);
    kprintf("[3]: %0#8X %#010x %#4o %0*u\n", 0xDEAD, 0xBEEF, 8, 4, 10);
    long long myLong = -9223372036854775807L;
    kprintf("[4]: %ld 0x%lX 0x%hX %hi\n", myLong, 0xFEFFFFFFBFFFFFFFL, 0xDEADBEEF, myLong);
    kprintf("[5]: %+ 6d %10s %10c\n", -1024, "string", 'c');
    kprintf("[6]: % 6d %10i %#010X\n", 1024, -123456, 0xFEEDFACE);
    kprintf("[7 (bug)]: %X %lX\n", 0xDECAFC0FFEEL, 0xDECAFC0FFEEL);
    kprintf("[7 (fix)]: %X %lX\n", (int) 0xDECAFC0FFEEL, 0xDECAFC0FFEEL);
}
</source>
````

<a id="w004077"></a>

## W004077 — User:Gravaera/ARMv7 Common Memory System Architecture,

Bron-ID: W004077
Namespace: User (2)
Revisie: 21039 | Datum: 2017-08-03T02:16:15Z
Bron: https://wiki.osdev.org/User:Gravaera/ARMv7_Common_Memory_System_Architecture%2C
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FARMv7_Common_Memory_System_Architecture%2C&oldid=21039
Doorverwijzing: ARMv7 Common Memory System Architecture,

````text
#REDIRECT [[ARMv7 Common Memory System Architecture,]]
````

<a id="w004078"></a>

## W004078 — ARMv7 Common Memory System Architecture,

Bron-ID: W004078
Namespace: Main (0)
Revisie: 21041 | Datum: 2017-08-03T02:16:48Z
Bron: https://wiki.osdev.org/ARMv7_Common_Memory_System_Architecture%2C
Revisie-URL: https://wiki.osdev.org/index.php?title=ARMv7_Common_Memory_System_Architecture%2C&oldid=21041
Doorverwijzing: ARMv7 Common Memory System Architecture

````text
#REDIRECT [[ARMv7 Common Memory System Architecture]]
````

<a id="w004079"></a>

## W004079 — User talk:Zesterer/Bare Bones

Bron-ID: W004079
Namespace: User talk (3)
Revisie: 21060 | Datum: 2017-08-09T19:08:18Z
Bron: https://wiki.osdev.org/User_talk:Zesterer/Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AZesterer%2FBare_Bones&oldid=21060

````text
== Editing the Main Page ==

User pages don't belong there, neither does self-promotion.

[[User:No92|no92]] 14:08, 9 August 2017 (CDT)
````

<a id="w004080"></a>

## W004080 — Category:UEFI

Bron-ID: W004080
Namespace: Category (14)
Revisie: 21067 | Datum: 2017-08-10T09:56:00Z
Bron: https://wiki.osdev.org/Category:UEFI
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AUEFI&oldid=21067

````text
The UEFI Category contains pages relating the Unifed Extensible Firmware Interface.

[[Category:Firmware]] [[Category:X86]] [[Category:X86-64]] [[Category:ARM]] [[Category:IA-64]]
````

<a id="w004081"></a>

## W004081 — UEFI NVRAM

Bron-ID: W004081
Namespace: Main (0)
Revisie: 21070 | Datum: 2017-08-10T10:11:08Z
Bron: https://wiki.osdev.org/UEFI_NVRAM
Revisie-URL: https://wiki.osdev.org/index.php?title=UEFI_NVRAM&oldid=21070

````text
== Introduction ==
Both BIOS and UEFI store variables in NVRAM - non-volatile RAM. The location of this is platform-specific - early BIOSes used  the [[CMOS]].
BIOS variables, in CMOS or otherwise, were not standardised. Accessing their location sometimes was possible for the OS, sometimes not. Knowing their meaning meant knowing the BIOS version, potentially to a build level. Clearly, maintaining a catalogue of all BIOSes is untenable.

[[UEFI]] solves this problem. UEFI provides an extensible, standardised interface to access NVRAM variables - infact, it is one of the few runtime services that UEFI provides, indicating that the variables are not otherwise accessible to the OS. This means that at any point during the OS's execution, it can both get and set these variables.

== Interface ==
{{Template:stub}}
Under the UEFI system table, under runtime services, there are 3 variable services provided.
<source lang="C">
typedef
EFI_STATUS
GetVariable (
IN CHAR16* VariableName,
IN EFI_GUID* VendorGuid,
OUT UINT32* Attributes OPTIONAL,
IN OUT UINTN* DataSize,
OUT VOID* Data OPTIONAL
);
typedef
EFI_STATUS
GetNextVariableName (
IN OUT UINTN *VariableNameSize,
IN OUT CHAR16 *VariableName,
IN OUT EFI_GUID *VendorGuid
);
typedef
EFI_STATUS
SetVariable (
IN CHAR16 *VariableName,
IN EFI_GUID *VendorGuid,
IN UINT32 Attributes,
IN UINTN DataSize,
IN VOID *Data
);
</source>
[[Category:UEFI]]
````

<a id="w004082"></a>

## W004082 — User:Bellezzasolo/C++ Exceptions

Bron-ID: W004082
Namespace: User (2)
Revisie: 21104 | Datum: 2017-08-16T20:42:47Z
Bron: https://wiki.osdev.org/User:Bellezzasolo/C%2B%2B_Exceptions
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ABellezzasolo%2FC%2B%2B_Exceptions&oldid=21104

````text
==Introduction==
There is a page in the main namespace about adding G++ exception code to your kernel. This page provides a different approach.
This implementation has its limitations, but is reasonably simple, and is standalone. Note: dirty Visual C++ code.
===RTTI===
This implementation depends on an RTTI implementation. Guess what... I did that too. All classes must be derived from Object to be used in RTTI/Exceptions.
====Object.h====
<source lang="cpp">
KCSTDLIB_API_CLASS(Object)
{
public:
	Object(const TypeId type);		//Call this with TypeIdNoRTTI(this)
	virtual ~Object();
	const TypeId getType(){ return m_type; }
protected:
	const TypeId m_type;
	const bool m_didRegisterException;
};

KCSTDLIB_API_FUNC(void enableFancyStuff());		//Allows new objects to use RTTI and exceptions.
bool isFancyEnabled();

#undef typeid

KCSTDLIB_API_FUNC(const TypeId DECL_CALL typeID(Object* obj));

#define typeid(x) typeID(x)

template <class D, class B> bool ConstructorHelper(char* classname, size_t size, D* derived = (D*)NULL, B* base = (B*)NULL)
{
	TypeId derivedtype = TypeIdNoRTTI(derived);
	TypeId basetype = TypeIdNoRTTI(base);
	RegisterType(classname, derivedtype, size);
	RegisterInheritance(derivedtype, basetype);
	return true;
}

//This forces the use of an init method, which is no bad thing. Create another constructor if you need fancies. You can stick more constant constructors on the end. Of course, just replicate this code if you need.
#define OBJECT_DERVIED_CONSTRUCTOR(classname, baseobj) \
	classname(TypeId type = 0) \
	:baseobj(ConstructorHelper<classname, baseobj>(#classname, sizeof(classname))&&type==0?TypeIdNoRTTI(this):type)
</source>
====Object.cpp====
<source lang="cpp">
#include "Object.h"

static bool isMemorySetup = false;

Object::Object(const TypeId type)
:m_type(type), m_didRegisterException(isMemorySetup)
{
	if (isMemorySetup)
	{
		//Add this class to the exception unwinding list.
	}
}


Object::~Object()
{
	if (m_didRegisterException)
	{
		//Remove this class from the exception unwinding list.
	}
}

KCSTDLIB_API_FUNC(void DECL_CALL enableFancyStuff())
{
	isMemorySetup = true;
	RegisterType("Object", TypeIdNoRTTI((Object*)NULL), sizeof(Object));
}

bool isFancyEnabled()
{
	return isMemorySetup;
}

KCSTDLIB_API_FUNC(const TypeId DECL_CALL typeID(Object* obj))
{
	return obj->getType();
}
</source>
==== RTTI.h====
<source lang="cpp">
#include "hash.h"

typedef hash_t TypeId;

KCSTDLIB_API_FUNC(const char* DECL_CALL LookupName(TypeId type));

KCSTDLIB_API_FUNC(bool DECL_CALL IS_A(TypeId derived, TypeId base));

KCSTDLIB_API_CLASS(type_info) {
public:
	bool operator == (const type_info& rhs)const
	{
		return m_type == rhs.m_type;
	}
	bool operator != (const type_info& rhs) const
	{
		return m_type != rhs.m_type;
	}
	bool before(const type_info& rhs) const
	{
		return m_type < rhs.m_type;
	}
	const char* name(TypeId type) const
	{
		return LookupName(type);
	}
	size_t hash_code() const
	{
		return m_type;
	}
protected:
	TypeId m_type;
};



KCSTDLIB_API_FUNC(void DECL_CALL RegisterInheritance(TypeId derived, TypeId base));
KCSTDLIB_API_FUNC(void DECL_CALL RegisterType(char* classname, TypeId type, size_t size));

template<class T>
TypeId TypeIdNoRTTI(const T*) //this function is instantiated for every different type
{
	auto strlenlocal = [](const char* str)
	{
		int len = 0;
		while (str[len])
			len++;
		return len;
	};
	TypeId type = SuperFastHash(__FUNCDNAME__, strlenlocal(__FUNCDNAME__));
	
	return type;
}

#endif
</source>
====RTTI.cpp====
<source lang="cpp">
typedef struct _CLASS_INHERITANCE_ENTRY {
	void* inherits;		//PCLASS_INHERITANCE_ENTRY
	_CLASS_INHERITANCE_ENTRY* prev;
	_CLASS_INHERITANCE_ENTRY* next;
}CLASS_INHERITANCE_ENTRY, *PCLASS_INHERITANCE_ENTRY;

typedef struct _CLASS_TYPE_ENTRY {
	TypeId classtype;
	char* classname;
	size_t size;
	PCLASS_INHERITANCE_ENTRY inheritances;
	_CLASS_TYPE_ENTRY* prev;
	_CLASS_TYPE_ENTRY* next;
}CLASS_TYPE_ENTRY, *PCLASS_TYPE_ENTRY;

PCLASS_TYPE_ENTRY typestacktop = NULL;

static PCLASS_TYPE_ENTRY AddEntry()
{
	if (!typestacktop)
	{
		typestacktop = new CLASS_TYPE_ENTRY;
		if (!typestacktop)
			return NULL;
		typestacktop->prev = typestacktop->next = NULL;
	}
	else
	{
		PCLASS_TYPE_ENTRY prev = typestacktop;
		typestacktop = typestacktop->next = new CLASS_TYPE_ENTRY;
		if (!typestacktop)
			return NULL;
		typestacktop->prev = prev;
		typestacktop->next = NULL;
	}
	return typestacktop;
}

static PCLASS_INHERITANCE_ENTRY AddIneritsEntry(PCLASS_INHERITANCE_ENTRY& firstptr)
{
	PCLASS_INHERITANCE_ENTRY& cur = firstptr;
	while (cur && cur->next)
	{
		cur = cur->next;
	}
	if (!cur)
	{
		cur = new CLASS_INHERITANCE_ENTRY;
		if (!cur)
			return NULL;
		cur->prev = cur->next = NULL;
	}
	else
	{
		PCLASS_INHERITANCE_ENTRY prev = cur;
		cur = cur->next = new CLASS_INHERITANCE_ENTRY;
		if (!cur)
			return NULL;
		cur->prev = prev;
		cur->next = NULL;
	}
	return cur;
}

static PCLASS_TYPE_ENTRY LookupType(TypeId type)
{
	PCLASS_TYPE_ENTRY typeent = typestacktop;
	while (typeent)
	{
		if (typeent->classtype == type)
			return typeent;
		typeent = typeent->prev;
	}
	return NULL;
}

KCSTDLIB_API_FUNC(const char* DECL_CALL LookupName(TypeId type))
{
	if (!isFancyEnabled())
		return NULL;
	PCLASS_TYPE_ENTRY ent = LookupType(type);
	return ent->classname;
}

static bool IS_A(PCLASS_TYPE_ENTRY derived, TypeId base)
{
	PCLASS_INHERITANCE_ENTRY inherits = derived->inheritances;
	while (inherits)
	{
		PCLASS_TYPE_ENTRY ent = (PCLASS_TYPE_ENTRY)inherits->inherits;
		if (ent->classtype == base || IS_A(ent, base))
			return true;
		inherits = inherits->next;
	}
	return false;
}

KCSTDLIB_API_FUNC(bool DECL_CALL IS_A(TypeId derived, TypeId base))
{
	if (derived == base)
		return true;
	if (!isFancyEnabled())
		return false;
	PCLASS_TYPE_ENTRY derv = LookupType(derived);
	PCLASS_TYPE_ENTRY bas = LookupType(base);
	if (!derv || !bas)
		return false;
	return IS_A(derv, base);
}

KCSTDLIB_API_FUNC(void DECL_CALL RegisterInheritance(TypeId derived, TypeId base))
{
	if (!isFancyEnabled())
		return;
	PCLASS_TYPE_ENTRY ent = LookupType(derived);
	PCLASS_TYPE_ENTRY baseent = LookupType(base);
	PCLASS_INHERITANCE_ENTRY inheritent = AddIneritsEntry(ent->inheritances);
	inheritent->inherits = baseent;
}

KCSTDLIB_API_FUNC(void DECL_CALL RegisterType(char* classname, TypeId type, size_t size))
{
	if (!isFancyEnabled())
		return;
	if (LookupType(type))
		return;
	PCLASS_TYPE_ENTRY ent = AddEntry();
	if (!ent)
		return;
	ent->classname = classname;
	ent->classtype = type;
	ent->size = size;
	ent->inheritances = NULL;
}
</source>
===Exceptions===
OK, here's what you've been waiting for. Put away that boring RTTI stuff.
====exception.h====
<source lang="cpp">

KCSTDLIB_API_FUNC(void* DECL_CALL ChaiOSRegisterTry(const char* file, unsigned int line, jmp_buf buffer));
KCSTDLIB_API_FUNC(void DECL_CALL ChaiOSRegisterCatch(TypeId type, jmp_buf buffer));
KCSTDLIB_API_FUNC(void DECL_CALL RegisterExceptionObject(Object* obj));
KCSTDLIB_API_FUNC(void DECL_CALL CallTryLambda(void* tryblockptr, jmp_buf buffer));
KCSTDLIB_API_FUNC(Object* DECL_CALL ChaiOSReadExceptionThrow(int val));
KCSTDLIB_API_FUNC(void DECL_CALL ChaiOSThrow(TypeId type, Object* obj, wchar_t* filename, unsigned int linenum));
KCSTDLIB_API_FUNC(void DECL_CALL ChaiOSgoAfterState(void* ptr));

#define STR_HELPER(x) #x
#define STR(x) STR_HELPER(x)

#define COMBINE1(X,Y) X##Y  // helper macro
#define COMBINE(X,Y) COMBINE1(X,Y)

//Call this in each function you use a try block. Prepares some variables
#define PREPARE_TRY() \
	void* tryblockptr = NULL; \
	TypeId catchtype = NULL; \
	int lastsetjmp = 0;

//Begins a try block. Prepares a setjmp. Uses line-local variables. When longjmp is called, executes try block.
#define TRY \
	jmp_buf COMBINE(buffer, __LINE__); \
	unsigned int COMBINE(val, __LINE__) = lastsetjmp = setjmp(COMBINE(buffer, __LINE__)); \
	if ((COMBINE(val, __LINE__)) == 0) \
		tryblockptr = ChaiOSRegisterTry(__FILE__, __LINE__, COMBINE(buffer, __LINE__)); \
	if ((COMBINE(val, __LINE__)) != 0)

//Stick this after every try block. It goes to the end of the exception code.
#define END_TRY \
	if (lastsetjmp != 0) \
	ChaiOSgoAfterState(tryblockptr);

//Stick this at the end of your exception handling code.
#define END_EXCEPT() \
	jmp_buf COMBINE(buffer,__LINE__); \
	unsigned int COMBINE(val, __LINE__) = lastsetjmp = setjmp(COMBINE(buffer, __LINE__)); \
	if (COMBINE(val,__LINE__) == 0)\
		CallTryLambda(tryblockptr, COMBINE(buffer,__LINE__));
	
template <class T> TypeId chaiosCatchHelper(T x)		//Takes a pointer.
{
	TypeId type = typeID(x);
	return type;

};
//Typename goes in x. You then access it as (x) e, e.g. Object* e
#define CATCH(x) \
	x* sampler = new x; \
	catchtype = chaiosCatchHelper(sampler); \
	delete sampler; \
	jmp_buf COMBINE(buffer, __LINE__); \
	unsigned int COMBINE(val, __LINE__) = lastsetjmp = setjmp(COMBINE(buffer, __LINE__)); \
	if (COMBINE(val, __LINE__) == 0) \
		ChaiOSRegisterCatch(catchtype, COMBINE(buffer, __LINE__)); \
	if (COMBINE(val, __LINE__) != 0) \
		if(x* e = static_cast<x*>(ChaiOSReadExceptionThrow(COMBINE(val, __LINE__))))

#define CATCH_ALL \
	jmp_buf COMBINE(buffer, __LINE__); \
	unsigned int COMBINE(val, __LINE__) = lastsetjmp = setjmp(COMBINE(buffer, __LINE__)); \
	if (COMBINE(val, __LINE__) == 0) \
		ChaiOSRegisterCatch(0, COMBINE(buffer, __LINE__)); \
	if (COMBINE(val, __LINE__) != 0) \

#define END_CATCH \
	if (lastsetjmp != 0) \
	ChaiOSgoAfterState(tryblockptr);

#define THROW(x) \
	TypeId type = chaiosCatchHelper(x); \
	ChaiOSThrow(type, x, __FILEW__, __LINE__);

class CException : public Object
{
public:
	OBJECT_DERVIED_CONSTRUCTOR(CException, Object)
	{

	}
	virtual ~CException()
	{

	}

	wchar_t* getMessage()
	{
		return m_message;
	}

	void setMessage(wchar_t* msg)
	{
		m_message = msg;
	}
protected:
	wchar_t* m_message;
};
</source>
====exception.cpp====
<source lang="cpp">
enum EntryTypes {
	TRY_BLOCK,
	CATCH_BLOCK,
	DESTRUCTOR
};

typedef struct _EXCEPTION_STACK_BLOCK_ENTRY {
	EntryTypes type;
	union {
		//Try block
		struct {
			jmp_buf state;
			jmp_buf afterstate;
			const char* file;
			unsigned int line;
		}tryblock;
		//Catch block
		struct  {
			TypeId excepttype;
			jmp_buf exceptblock;
		}catchblock;
		//Destructor
		Object* obj;
	};
	_EXCEPTION_STACK_BLOCK_ENTRY* prev;
	_EXCEPTION_STACK_BLOCK_ENTRY* next;
}EXCEPTION_STACK_BLOCK_ENTRY, *PEXCEPTION_STACK_BLOCK_ENTRY;

static PEXCEPTION_STACK_BLOCK_ENTRY stacktop = NULL;

static PEXCEPTION_STACK_BLOCK_ENTRY AddStackEntry()
{
	if (!stacktop)
	{
		stacktop = new EXCEPTION_STACK_BLOCK_ENTRY;
		if (!stacktop)
			return NULL;
		stacktop->prev = stacktop->next = NULL;
	}
	else
	{
		PEXCEPTION_STACK_BLOCK_ENTRY prev = stacktop;
		stacktop = stacktop->next = new EXCEPTION_STACK_BLOCK_ENTRY;
		if (!stacktop)
			return NULL;
		stacktop->prev = prev;
		stacktop->next = NULL;
	}
	return stacktop;
}

//Delete the pointer when you're finished with the object.
static PEXCEPTION_STACK_BLOCK_ENTRY PopStack()
{
	if (!stacktop)
		return NULL;
	PEXCEPTION_STACK_BLOCK_ENTRY value = stacktop;
	if (value->prev)
		value->prev->next = NULL;
	stacktop = value->prev;
	return value;
}

KCSTDLIB_API_FUNC(void* DECL_CALL ChaiOSRegisterTry(const char* file, unsigned int line, jmp_buf buffer))
{
	if (!isFancyEnabled())
		return NULL;
	PEXCEPTION_STACK_BLOCK_ENTRY ent = AddStackEntry();
	if (!ent)
		return NULL;
	ent->type = TRY_BLOCK;
	ent->tryblock.file = file;
	ent->tryblock.line = line;
	memcpy(ent->tryblock.state,buffer, sizeof(jmp_buf));
	return ent;
}

KCSTDLIB_API_FUNC(void DECL_CALL ChaiOSRegisterCatch(TypeId type, jmp_buf exceptblock))
{
	if (!isFancyEnabled())
		return;
	PEXCEPTION_STACK_BLOCK_ENTRY ent = AddStackEntry();
	ent->type = CATCH_BLOCK;
	ent->catchblock.excepttype = type;
	memcpy(ent->catchblock.exceptblock, exceptblock, sizeof(jmp_buf));
}

KCSTDLIB_API_FUNC(void DECL_CALL RegisterExceptionObject(Object* obj))
{
	if (!isFancyEnabled())
		return;
	PEXCEPTION_STACK_BLOCK_ENTRY ent = AddStackEntry();
	ent->type = DESTRUCTOR;
	ent->obj = obj;
}

KCSTDLIB_API_FUNC(void DECL_CALL CallTryLambda(void* tryblockptr, jmp_buf buffer))
{
	if (!isFancyEnabled())
		return;
	PEXCEPTION_STACK_BLOCK_ENTRY tryblk = (PEXCEPTION_STACK_BLOCK_ENTRY)tryblockptr;
	//Save the state for after the try block
	memcpy(tryblk->tryblock.afterstate, buffer, sizeof(jmp_buf));
	//Jump into the try block
	longjmp(tryblk->tryblock.state, 0);
}

KCSTDLIB_API_FUNC(void DECL_CALL ChaiOSgoAfterState(void* ptr))
{
	if (!isFancyEnabled())
		return;
	PEXCEPTION_STACK_BLOCK_ENTRY tryblk = (PEXCEPTION_STACK_BLOCK_ENTRY)ptr;
	//Clean up our stack
	//Save the state we want to return to
	jmp_buf afterstate;
	memcpy(afterstate, tryblk->tryblock.afterstate, sizeof(jmp_buf));
	//Delete stack entries above our pointer
	while (stacktop != tryblk && stacktop)
	{
		PEXCEPTION_STACK_BLOCK_ENTRY todel = stacktop;
		stacktop = todel->prev;
		delete todel;
	}
	//Delete our try block pointer (stacktop)
	stacktop = tryblk->prev;
	delete tryblk;
	stacktop->next = NULL;
	longjmp(afterstate, 0);
}

//TODO: make this nicer than a global variable
Object* except = NULL;

KCSTDLIB_API_FUNC(void DECL_CALL ChaiOSThrow(TypeId type, Object* obj, wchar_t* filename, unsigned int linenum))
{
	if (!isFancyEnabled())
		return;
	//Walk the stack
	PEXCEPTION_STACK_BLOCK_ENTRY ptr = PopStack();
	bool caught = false;
	while (ptr)
	{
		bool breakloop = false;
		switch (ptr->type)
		{
		case DESTRUCTOR:
			delete ptr->obj;
			break;
		case CATCH_BLOCK:
			if (!caught && (IS_A(type, ptr->catchblock.excepttype) || ptr->catchblock.excepttype == 0))
			{
				//Jump into the except block
				except = obj;
				//TODO: nicer than global variable
				longjmp(ptr->catchblock.exceptblock, 1);
				caught = true;
			}
			break;
		case TRY_BLOCK:
			if (caught)
			{
				breakloop = true;
				longjmp(ptr->tryblock.afterstate, 0);
			}
			//printf("Going past try block at %s, line %d\n", ptr->tryblock.file, ptr->tryblock.line);
			break;
		};
		delete ptr;
		if (breakloop == true)
			break;
		ptr = PopStack();
	}
	//TODO: kernel panic
	if (IS_A(typeID(obj), TypeIdNoRTTI((CException*)NULL)))
	{
		getTerminal()->kpanic(((CException*)obj)->getMessage(), 0x999, 0, 0, 0, 0, filename, linenum);
	}
	else
	{
		getTerminal()->kpanic(L"Unhandled Exception!", 0x999);
	}
}

KCSTDLIB_API_FUNC(Object* DECL_CALL ChaiOSReadExceptionThrow(int val))
{
	if (!isFancyEnabled())
		return NULL;
	//TODO: nicer than global variable
	return except;
}
</source>
== Conclusion ==
Did you really expect this code to work out of the box? No. This code is a sample implementation. It's easy enough to make work, but I hope to inspire you to make a better implementation.
````

<a id="w004083"></a>

## W004083 — User:Narke/Languages

Bron-ID: W004083
Namespace: User (2)
Revisie: 30910 | Datum: 2026-08-16T11:34:46Z
Bron: https://wiki.osdev.org/User:Narke/Languages
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANarke%2FLanguages&oldid=30910

````text
A summary of programming languages which can be used to implement kernels.

{| class="wikitable"
|+ Programming languages for OSDev
|-
! Number !! Language           !! Example OS !! Link
|-
| 1       || ADA               || Ironclad   || https://ironclad.cx
|-
| 2       || Assembly language || KolibriOS  || https://kolibrios.org/en/
|-
| 3       || ATS               ||            || 
|-
| 4       || C                 || NetBSD     || https://www.netbsd.org
|-
| 5       || C++               || managarm   || https://github.com/managarm/managarm
|-
| 6       || Crystal           ||            || 
|-
| 7       || colorForth        || colorForth || https://sourceforge.net/projects/colorforth/
|-
| 8       || D                 || PowerNex   || https://github.com/PowerNex/PowerNex
|-
| 9       || Deca              ||            || 
|-
| 10      || Forth             ||            || 
|-
| 11      || FreeBasic         ||            || 
|-
| 12      || FreePascal        || FPOS       || https://wiki.freepascal.org/Operating_Systems_written_in_FPC
|-
| 13      || HolyC             || TempleOS   || https://templeos.org/
|-
| 14      || Lisaac            || IsaacOS    || https://github.com/ontologiae/IsaacOS
|-
| 15      || Hare              || Bunnix     || https://git.sr.ht/~sircmpwn/bunnix
|-
| 16      || Haskell           ||            || 
|-
| 17      || Mesa              || Pilot      || https://en.wikipedia.org/wiki/Pilot_(operating_system)
|-
| 18      || Modula-3          ||            || 
|-
| 19      || Myrddin           ||            || 
|-
| 20      || Nim               || nimkernel  || https://github.com/dom96/nimkernel
|-
| 21      || Oberon 2          ||            || 
|-
| 22      || Orth              || Orth demo  || https://github.com/602p/orth
|-
| 23      || Pascal            ||            || 
|-
| 24      || Rust              || Redox      || https://www.redox-os.org/
|-
| 25      || Terra             ||            ||
|-
| 26      || V                 || Vinix      || https://github.com/vlang/vinix
|-
| 27      || Volt              ||            || 
|-
| 28      || Wacc              ||            || 
|-
| 29      || X                 ||            || 
|-
| 30      || Zig               || zorroOS    || https://github.com/zig-osdev
|-
| 31      || Zimbu             ||            || 
|}
````

<a id="w004084"></a>

## W004084 — Make

Bron-ID: W004084
Namespace: Main (0)
Revisie: 21113 | Datum: 2017-08-21T14:07:59Z
Bron: https://wiki.osdev.org/Make
Revisie-URL: https://wiki.osdev.org/index.php?title=Make&oldid=21113
Doorverwijzing: Makefile

````text
#REDIRECT [[Makefile]]
````

<a id="w004085"></a>

## W004085 — User:Schol-r-lea/The Shoehorn Parable

Bron-ID: W004085
Namespace: User (2)
Revisie: 21134 | Datum: 2017-08-22T21:40:15Z
Bron: https://wiki.osdev.org/User:Schol-r-lea/The_Shoehorn_Parable
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASchol-r-lea%2FThe_Shoehorn_Parable&oldid=21134

````text
'''''N.B.: This is one of a group of forum posts extracted so that the information in them can be re-worked into a less confrontational, more informational set of wiki pages. They are here only temporarily.'''''


Imagine that you own a company which makes making shoe horns. You hear that some people are trying to make knives out of your shoehorns, which just seems weird to you, and so you stop taking orders from the weirdos. You are just getting things started for a new business line in selling televisions, and figure that the new model of shoehorns will last just long enough to pay for that re-tooling, when some guys walk into the factory with a 50-star general following them. One of these guys is one you know who has been making knives out of your shoehorns, while the other one is rolling his eyes and is frantically trying to convince the general that he is making a mistake. The general goes up to you and says, "I need a million of these bayonets you are making by next week".

You might ask him what he is talking about, but he points to a shoehorn, and the knife guy hands you a suitcase full of money with an enthusiastic gleam in his eyes. When you tell him you don't make bayonets, and that those are shoehorns, the general insists that these are perfect for his needs.

OK, you think, we can sell the shoehorns to them, its obvious that something crazy is going on but since I can't seem to talk this general out of it, I might as well get the money.

A few months pass, and you keep getting more orders for 'bayonets'. Meanwhile, the television project isn't going well, and you're thinking you might need to close the factory entirely, or at least focus on cheaper shoehorns again. Then, out of nowhere, government officials from a dozen countries start showing up with duffel bags full of cash, demanding that you sell them bayonets. Would you turn down that sort of money, just at the point when your company is about to tank?

This is basically what happened to Intel. They never wanted to be in the home computer market; they were making embedded systems chips with an eye on the emerging workstation market. The 8086 was something they threw together to squeeze another year or two out of the 8080 line until their 'real' products, the iAPX 432 workstation CPU and the 8051 SBC controller, were ready. They were blindsided by the PC market, but it was roaring just at a time when Intel's big gamble in the 432 was falling apart. That left them with a product line they didn't want, but couldn't afford to leave.

The engineering is not even a factor in the story. What mattered was that IBM laid down the law about home computers, just as companies like Osborne and Proc Tech were dying of terrible management, and everyone still in the game figured that you couldn't lose money by copying what The Empire was doing. Intel could have been selling abaci, but if IBM were using them, then everyone bought their abacus from Intel.

-----
''Korona's (rather apt) reply:'' That is the history, but today Intel's shoehorns are no longer made of wood or plastic but instead of carbon fiber reinforced titanium with lightsabers attached to both sides. You can still use them go get your shoes on (because some of your clients are actually doing this) but their primary function is providing the best bayonet available. Instead of trying to make new "proper" bayonets from scratch it's much cheaper and more reliable to just keep using those shoehorns. And even if somebody did actually try to manufacture proper bayonets there is a high chance that he will implement some design decision that renders the proper bayonets worse than the shoehorns.
-----
''My response:'' OK, that's very true. As much as I hate the architecture, I can't disagree that for what people want out of them - that is to say, performance and lots of it - they are the best. I may be wistful about what might have been, but I still use an i5 laptop myself.
````

<a id="w004086"></a>

## W004086 — User:Schol-r-lea/Hells Forum

Bron-ID: W004086
Namespace: User (2)
Revisie: 21116 | Datum: 2017-08-22T20:58:25Z
Bron: https://wiki.osdev.org/User:Schol-r-lea/Hells_Forum
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASchol-r-lea%2FHells_Forum&oldid=21116

````text
Jean Phillipe, open Hell's Forum!

(BTW... all links below NSFW)

Except here, here are a ''dozen'' [https://www.youtube.com/watch?v=vBqrK9tyvzU Gordon Ramsays], all trying to run things their way, whether they have any standing to or not... and several dozen [https://www.youtube.com/watch?v=62BL-4Htuho Josephs], [https://www.youtube.com/watch?v=4cHfiuNkxUY Rajs], [https://www.youtube.com/watch?v=-MAGfZ_hC6Y Melindas] and [https://www.youtube.com/watch?v=UjE-cBDFphQ Giovannis]... and more than a few [https://www.youtube.com/watch?v=BfD8L1euBuk Crazy] [https://www.youtube.com/watch?v=vF5wYg4qBiw Amy] [https://www.youtube.com/watch?v=J3tDSGsYS3I Bouzaglo]s wandering in from ''[https://www.youtube.com/watch?v=iHLmT-rHW6Q OS Nightmares]'' to make things even crazier.

Or perhaps comparing it to the ''[https://www.youtube.com/watch?v=SXUzbcNM6K0 Dark Souls]'' fanbase would make more sense - except that "Get Good!" doesn't cover it really, though it is part of it all. Get Attitudinous may be closer to the mark.

Thing is, though, with a topic so demanding, and so focused, and so - dare I say it? - esoteric, that's sort of inevitable. It's like mountain climbing - it demands careful attention to detail, rigorous preparation, utmost skill, and - let's be honest here - a real egotistical streak, a belief that you can persevere and overcome. Even if, as I have argued, the skills used in OS dev don't really reflect those needed in other types of programming, it is still very much a test of one's ability to even get the most basic self-booting system going. It is a field which draws in people with strong beliefs and opinions, and no small amount of arrogance. It wouldn't be OS dev otherwise.
````

<a id="w004087"></a>

## W004087 — User:Schol-r-lea/Early CLIs

Bron-ID: W004087
Namespace: User (2)
Revisie: 21137 | Datum: 2017-08-22T21:41:57Z
Bron: https://wiki.osdev.org/User:Schol-r-lea/Early_CLIs
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASchol-r-lea%2FEarly_CLIs&oldid=21137

````text
'''''N.B.: This is one of a group of forum posts extracted so that the information in them can be re-worked into a less confrontational, more informational set of wiki pages. They are here only temporarily.'''''


The similarities they have are very superficial (much like the systems themselves), even in terms of user experience, and are shared by nearly every command interpreter of the time which wasn't just a BASIC or Forth interpreter. You might as well say that Tenex and VMS were the same as ITS - not that I expect anyone here who isn't a history buff to know any of them, but still. They are similar because that was the way nearly every command interpreter worked, going back nearly a decade before Unix existed. The style goes all the way back to systems like CTSS, with some things appearing even earlier.

----

The directory structure was copied whole cloth from Unix (which copied Multics, which copied CTSS going way back to 1963), that's true... in MS-DOS 2.2, more than three years after MS-DOS was introduced (it didn't have directories before then at all, and CP/M never had them, at least not as part of the file system proper). The commands were similar in that they were text... but so were the command systems for ITS, Tenex, VM/CMS, Apple II ProDOS (AppleDOS, and several others like Commodore 64 and the Atari 8-bit systems, didn't have a separate command interpreter, instead dropping you right into the BASIC interpreter instead) and a dozen other OSes which had no connection to speak of to either Unix nor CP/M.

And it ignores a lot of things that are very, very different, such as the fact that MS-DOS works directly with every disk by disk letter, an arrangement typical of microcomputers of the time, whereas disks in minicomputer and mainframe OSes like Unix always mounted the disks as sub-directories of a root directory. Or the fact that Unix required hardware memory protection to run at all after some early version (definitely by v6, which was the version used in the Lions book). Or that it was designed for timesharing, meaning that most users weren't even in the same building as the computer itself (as I said, it was a timesharing system), and you might have up to 32 people trying to use the same computer at the same time.

In contrast, the 8088 had no CPU memory protection (segmentation existed, but it wasn't protection - its purpose was to allow 16-bit addressing on a 20-bit machine) and the PC didn't add any on top of it that MS-DOS could have used. MS-DOS didn't really run at all except when at the command.com prompt - once a program started, it had control of the entire system, and MS-DOS wouldn't even stay completely resident if a program needed the memory it was using, but would have to have the stub left in memory load the rest of itself back in once the program exited.

Things like interrupt handlers were set up by MS-DOS, true, but there was nothing to stop a program from replacing those hooks with it's own - which is in fact how early Terminate-and-Stay-Resident (TSR) utilities like Sidekick worked. No OS running on something the size of a PDP-11 or a VAX would ever be so sloppy (mind you, Unix was and still is pretty sloppy, compared to many of the OSes of the time, but not nearly that sloppy). MS-DOS wasn't a full OS by most standards, more of a program launcher with a built-in library for handling a few of the hardware operations, and had nothing anyone could call a kernel or nucleus (the accepted term for the main part of an OS before Unix mangled all the terminology). It is more than a monitor (a type of system loader used in the 1950s that predated operating systems, and came back into use briefly in the 1970s as microcomputers came along), but only barely.

---

Despite the impression that the home computer revolution might have given (and I know that the former WP countries came late to the party on that, so you probably only knew the C64 at first), systems that used a BASIC interpreter for their main user interface were in the minority even in their heyday. The first such system was at Dartmouth in 1968, and Dartmouth Timeshared BASIC was the original BASIC implementation, which was designed as a way for ordinary students to use the computers, something no one else had managed to accomplish at that point.

This was something that few other system designers in the 1960s even considered a good idea at all; while Multics and TYMSHARE were intended for wider use, they were still focused on business and civil service use, so making things easy for the users wasn't even on the radar. Most other systems of the time assumed that no one who wasn't a trained system operator - not even a programmer, because most batch-processing systems didn't let programmers do anything other than enter code into a line editor, the assumption being that you wrote it out by hand first - would ever even see the console (probably a teletype with a roll of paper, not even a glass terminal) that was running the command interpreter.

BASIC only caught on in the microcomputer world because Paul and Billy-Boy saw a ground-floor opportunity in a system which shipped with 256 bytes of RAM (note the lack of any sort of prefix, and no that's not a joke - absurd, yes, but not a joke), had toggle switches as its sole interface, and had to be mocked up in the magazine article announcing it because the only working prototype was lost in the mail. They chose BASIC because they already knew how to write an interpreter for it, and Gates himself later admitted that it probably wasn't the best choice in retrospect (Ted Nelson claims he'd met Allen a year earlier, when he and Gates were trying to flog BASIC for the PDP-8 or some such before the whole Altair thing came around, and tried to talk him into backing Forth instead, which might have made for an interesting turn of events had it gone that way). Even then, one needed a 4 kilobyte add-on board to run the interpreter, and another add-on board to connect to a teletype, a terminal, or a television, in order to use it, and yet another add-on board to connect to the paper tape reader to read the interpreter in in the first place (in this video, the TTY had the tape reader built into it, but that wasn't universal).

No matter what you think of Gates today, or even about his arrogance at the time (what with the infamous "Open Letter to Hobbyists" debacle), the fact is they were the first ones to have even an assembler for the damn thing, and the fact that they then licensed their BASIC dialect to all and sundry afterwards meant that BASIC was on every microcomputer and small business computer for the next decade. Even the 'Open Letter' spread it, as it spurred the development of the various 'Tiny BASIC' interpreters (and launched Dr. Dobb's Journal, which started as a means of publishing them), and led to companies like Apple, Tandy, and Commodore to bundle the interpreter in the ROMs of the '1.5'-th generation micros such as the Apple II and the Commodore PET.
````

<a id="w004088"></a>

## W004088 — User:Schol-r-lea/OISC Wars Part 1

Bron-ID: W004088
Namespace: User (2)
Revisie: 21135 | Datum: 2017-08-22T21:40:46Z
Bron: https://wiki.osdev.org/User:Schol-r-lea/OISC_Wars_Part_1
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASchol-r-lea%2FOISC_Wars_Part_1&oldid=21135

````text
'''''N.B.: This is one of a group of forum posts extracted so that the information in them can be re-worked into a less confrontational, more informational set of wiki pages. They are here only temporarily.'''''


For example, you state that Unix, CP/M, and MS-DOS are similar, and influenced each other. This anhistorical assertion seems to be based solely in the fact that they were all text-based, but given the time period, that means nothing about how similar they were, because for all intents and purposes, all systems were text-mode at that time.

The only GUIs which were around before 1983 - the year both the Lisa and the Star were unveiled, and the year Project Athena began as a research project on GUI development - were NLS and the Smalltalk User Environment (and no, I don't mean the language, I mean the environment - the PARC UI came first, around 1970, and the language was named after it in 1972 because it was developed for scripting it based on Kay's experiments with a Logo derivative called Flex at MIT). NLS was an experiment in user experience, and ran on a series of different mainframes (the CDC-160A as a bare system, then a CDC-3100 under what I assume was either MASTER or SCOPE, then an SDS-940 under BTS, then finally a PDP-10 under Tenex, each version being a ground-up re-write in assembly and only the last two being graphical to any real degree). There was a graphical console for the CADR machine (the predecessor of the LMI and Symbolics Lisp Machines), but that was an experimental one-shot system with what was at the time unique hardware, and it was only graphical in that it could render graphics. There was the LINC-12, which was similar, though that was at least mass-produced. Similarly, there were graphical programs for some PDP-10 mainframe operating systems such as ITS, WAITS and Tenex, but they were also experimental things that used specialized consoles and needed to be scheduled so that all the timesharing users could be kicked off while the experiments were running; their main method of use remained text-mode.

And finally there were were the early microcomputers, most of which had a graphical mode, but because they mostly used television sets as monitors, the resolutions for the non-text modes weren't suited for text display, hence the need for two or more modes.

That's about it. Graphical displays were a rarity, and graphical user interfaces were still basically experiments, even as Xerox and Apple began trying (unsuccessfully at first) to market them.

So much for text being a similarity. Unix and MS-DOS were nothing alike in most other regards. For example, Unix, which was based in Multics in many ways but eliminated some of the 'vital parts' (hence the punny name - 'eunuchs', get it?), was a multitasking OS from the start, something which was crucial in the entire 'IPC through pipes' shenanigans. So why did they both use similar command line systems? Well... they didn't. MS-DOS borrowed some things from Unix later, but initially they were completely dissimilar. One can argue that MS-DOS copied CP/M, but trust me, CP/M didn't copy Unix - Unix was an obscure oddity when Kildall started work on CP/M in 1974, and the main influence on it was Tenex, not Multics.

If you still really think MS-DOS and Unix were similar, I dare you to run a Bourne shell script from that time period on an MS-DOS machine of any period without a specialized interpreter.

The claim that MIPS is similar to x86 is simply bizarre, and indicates a genuine lack of understanding of CPU architecture. Seriously, how anyone could find anything similar between the mess that is the 8086, and the remarkable (if perhaps excessive) minimalism of the MIPS and DLX designs, even as compared to OISC, is beyond comprehension. Name even one aspect of them that is similar that cannot be found in every actual CPU architecture after 1970, and I will be amazed.

No, OISC doesn't count - it is, believe it or not, actually emulated in microcode in every conceivable hardware implementation, because (as I learned the hard way when entertaining my own flights of fancy in this arena) real hardware needs registers. Any Turing complete computing machine based on the Random Access Machine model that doesn't use registers explicitly will need to have microcode to use them implicitly, period, end of subject. This isn't something you can handwave away - Subleq is not a simple instruction, and there is no way to implement it as one. It is an intrinsically compound operation, one which actually hides very real complexity.

In any case, what does the CPU architecture have to do with the price of chewing gum? NONE OF THIS IS RELEVANT, because all of this is buried under mounds and mounds of software - software like the BIOS, which is only 'hardware' in that it is stored in a permanent or semi-permanent medium (hence 'firmware'). Firmware is still software; it's just software that is embedded into how the hardware runs, and can know details about the hardware that are less discoverable without it. It doesn't do anything that can't be done in software that isn't stored in Flash memory, and due to the historical development of the PC, it isn't a particularly good set of programs.

CPU architecture has next to nothing to do with the complexity of the operating systems and development tools. You seem to think that you can scale some implementation of Subleq as an FPLA or a strung-together pile of TTL tinkertoys into something that can complete with CPUs implemented as 22nm integrated circuits, or else that someone is going to front the $5 billion it would cost for a silicon implementation to be brought to life - and that is the most outrageous part of all of this. Even assuming that were true, how would it make coding in a language like C any different? Even for an OS, most C code is going to be the same no matter the hardware. You simply don't seem to have thought any of this through.
````

<a id="w004089"></a>

## W004089 — User:Schol-r-lea/Early MIPS Workstations

Bron-ID: W004089
Namespace: User (2)
Revisie: 21136 | Datum: 2017-08-22T21:41:33Z
Bron: https://wiki.osdev.org/User:Schol-r-lea/Early_MIPS_Workstations
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASchol-r-lea%2FEarly_MIPS_Workstations&oldid=21136

````text
'''''N.B.: This is one of a group of forum posts extracted so that the information in them can be re-worked into a less confrontational, more informational set of wiki pages. They are here only temporarily.'''''


First off, you do realize that that is a CPU, not a full computer or even a motherboard, correct? Most of the systems that used the TMPR3927 (or any other MIPS implementation of the time) didn't actually use PCI, because they didn't ''have'' any extension buses - they were game consoles with fixed hardware and proprietary connectors for a limited selection of proprietary user controller devices. I don't know if the Sony consoles used the PCI bus model internally, but even if they did, that was only because that was what that particular implementation was designed for. That was Toshiba's decision, and had nothing to do with the MIPS in general.

Second, PCI was used on a ''lot'' of systems at the time, and while it arose out of the PC world, it wasn't tied to the PC specifically. PowerPC Macs used it. Pegasus II PPC motherboards used it. DEC Alpha workstations used it. Even some zSystem minis used it. It was a common interface which had been specifically designed ''not'' to be dependent on the x86. The cards that worked on it? Different story, since most were designed to only work on PCs. 

That having been said, there weren't a lot of MIPS desktops and workstations which used PCI, because '''''there were never a lot of MIPS desktops and workstations'''''. I don't know of a single MIPS desktop-class system at all prior to the Loongson, more's the pity. The [https://en.wikipedia.org/wiki/Advanced_Computing_Environment Advanced Computing Environment project], which was intended to come up with a replacement for the PC (which was already long in the tooth by 1991), was supposed to use MIPS, but the whole thing ended in acrimony and not a single system implementing the idea was ever sold, though a bunch of systems around that time which no one ever heard of took some of the ideas and made a hash of them to everyone's detriment.

The only MIPS workstations of any real note were the Silicon Graphics [https://en.wikipedia.org/wiki/SGI_Indigo Indigo], [https://en.wikipedia.org/wiki/SGI_IRIS_4D Iris], [https://en.wikipedia.org/wiki/SGI_Octane Octane], and [https://en.wikipedia.org/wiki/SGI_Indy Indy] systems, of which only the Octane (and the [https://en.wikipedia.org/wiki/SGI_Origin_200 Origin] rack-mounted server)  used PCI to the best of my knowledge; the earlier ones had proprietary buses which they never licensed (meaning that for the most part, only SGI hardware worked in SGI machines). Oh, and there were later models of the [https://en.wikipedia.org/wiki/Sony_NEWS Sony NEWS worsktations] which used MIPS, too, and those didn't use PCI either; they died out before PCI had really come into the market in significant amounts at all, really. Sony would prefer people forgot their disastrous foray into that market, anyway, and frankly, no one misses them.
````

<a id="w004090"></a>

## W004090 — User:Schol-r-lea/OISC Wars Part 2

Bron-ID: W004090
Namespace: User (2)
Revisie: 21138 | Datum: 2017-08-22T21:42:33Z
Bron: https://wiki.osdev.org/User:Schol-r-lea/OISC_Wars_Part_2
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASchol-r-lea%2FOISC_Wars_Part_2&oldid=21138

````text
'''''N.B.: This is one of a group of forum posts extracted so that the information in them can be re-worked into a less confrontational, more informational set of wiki pages. They are here only temporarily.'''''


@Geri: Getting back on topic a bit, the point has been made - and beaten to death - that regardless of what anyone thinks of the x86, we're stuck with it for the foreseeable future. The point has also been made that despite the apparent simplicity of the Subleq 'instruction set', there is zero chance of a hardware implementation of it with similar - or even adequate - performance coming to light. Similarly, the very 'simplicity' (which, as I also said, is misleading in the extreme) works against your in a software simulation, as the low code density - the ratio of pseudo-machine instructions to native instructions - is overwhelming. It isn't even a case where the 'simple' system is easier to write assembly code or target a compiler for - it is orders of magnitude harder on both of them, if anything.

You counter all of these arguments with talk about 'freedom', but I am still trying to figure out what you mean by that, as whole issue of re-implementation... isn't one. Seriously, it isn't a real world issue at all. It certainly isn't one to the buying public, to whom the whole topic would seem like a waste of breath. Most people don't want to use computers; they want to get things done, and in many cases the best way to get them done in with a computer. Talking about 'computer users' as something separate from what they use them would make no more sense than talking about 'microwave oven users' or 'automobile users', except that there are so many different things that computers can (and even must) be used for, and so many different and often conflicting ways to do them, that the topic is very important - to the developers. Not to the users. The users' main goal is to not have to learn about computers, to have things that they can use without a lot of extra things to learn, which means that they will stick to what they have already learned whenever they can.

You say a lot of things which simply aren't true, or even comprehensible. I think we've hit a case of Poe's Law here, where we can't even tell if you are a fanatic, or just a troll. Either way, no one should be taking you seriously.
````

<a id="w004091"></a>

## W004091 — User:Schol-r-lea/Itanic

Bron-ID: W004091
Namespace: User (2)
Revisie: 21133 | Datum: 2017-08-22T21:39:37Z
Bron: https://wiki.osdev.org/User:Schol-r-lea/Itanic
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASchol-r-lea%2FItanic&oldid=21133

````text
'''''N.B.: This is one of a group of forum posts extracted so that the information in them can be re-worked into a less confrontational, more informational set of wiki pages. They are here only temporarily.'''''


I was assuming that Zaval was making a joke, but apparently not. I don't know enough about the Itanium ISA to really judge, but from what little I saw of it, I wasn't impressed - I had the sense that they were combining a lot of old mistakes with several newer and shinier ones (not ''new'', exactly, since they had already been tried and failed elsewhere, but newer).

I agree with Brendan that a slow migration would be the wisest path for Intel - the only way a new architecture would make sense at this point is if if gave such a dramatic boost in performance that it could emulate the x86 at the equivalent of current speeds (for whatever 'current' happens to be at the time). This worked for Apple in 1989 only because Motorola had done almost nothing to improve the 680x0 series' performance between 1978 and 1988 except increase the clock speed, and the PowerPC chips were already at higher clock speeds than the 68040, meaning that a PowerMac actually could be faster under emulation than a 68040 Mac running native. Trying to do the same with the most heavily optimized chips on the market (even now, the top i7 is faster per core than any of their competitors, though AMD has done a lot of catching up with the Ryzen it still isn't really a match, even if it has much better value for price) is really unlikely to succeed unless a radically new approach arises.  Trust me, Subleq isn't nearly radical enough I mean something like quantum computing or something else that throws out the von Neuman and Harvard models out entirely, and possibly even abandons the RAM machine model in general.

As I say, as much as I hate the x86 personally, I really don't see it dying off yet... for reasons Intel doesn't control.

Who could change it? Microsoft, potentially, though they court an uprising if they try. Like Intel, Microsoft had been planning to move away from the x86 since the early 1990s (see the [https://en.wikipedia.org/wiki/Advanced_Computing_Environment "Advanced Computing Environment"] debacle, and the moves to put Windows on Alpha, MIPS, and partially, PowerPC). Despite the talk of 'secure computing' (never a real priority for Microsoft, simply because it was never enough of a priority for the average user so it bought them nothing), the primary reason for .NET is the same as the primary reason for the Java Virtual Machine - it was meant as a lever with which to pry users out of an existing hardware platform (the VAX and the older 386 Sun workstations, going to the Sun SPARC workstations, in the latter case, and from x86 to... something else, in the former). 

It hasn't worked out either time. In the Java case, Sun themselves got overwhelmed by cheap PCs which gave a significant portion of the power of their workstations for a fraction of the cost, and they didn't have enough money to keep improving the workstations so even the performance edge was lost soon enough. In the Microsoft case, there simply hasn't been a successor for the x86 which was equal in sheer speed over the past fifteen years, never mind superior. They had their bags packed, but had nowhere to move to, because Intel and AMD could - and had to, in order to stay alive - pour ten times as much money into the x86 ISA as anyone else could for any other one, and no one else wanted a replacement so badly that they could afford to foot the bill.

Things may be changing soon, though. Right now, CPU performance in general has been on a plateau for the past ten years - it has improved, but only by very small increments compared to just the previous five. The limiting factors in performance are now elsewhere - disk access (hence the introduction of Optane and the ever rising popularity of SSDs despite their price), graphics (though that had reached a plateau of sorts as well, until the drive to go 4K gave it greater impetus again), main memory size and speed, and... well, malware. 

Seriously, the main thing that hurts Windows (and Mac and Linux, to a much lesser extent due to being lower-priority targets, but still present) is the massive amounts of malicious and intrusive junk that accumulates in every PC starting from the first time it gets on a network. CPU speed can't hope to overcome a tsunami like that. Without constant, intensive maintenance, even the fastest i7 will run slower than an XT after a year of regular Internet use because it will be more clogged with crud than my arteries (and I am a lardball who is pushing fifty). 

This means that MS has to be doing some serious thinking about getting a move on. I think that the ARM port of Windows might be for more than just mobile systems - my guess is that they believe that by performing an Apple-esque trapeze act to a new platform, and get more software running .NET rather than any native model, they can unclog their systems (temporarily) and tighten up the control of the platform at the same time.  I doubt it will work - they would be throwing the hardcore PC gamers and other high-performance audiovisual users under the bus for at least a few years, and there are just too many legacy programs which they would be screwing over for them to drop x86 emulation, ever -  I think it will prove too tempting for them not to try. 

Besides, neither gamers nor artists are significant markets for MS, so no real loss there - 'serious' gamers are a tiny sliver of the market, while most musicians, video editors, and artists are on Macs anyway - though the stink it would raise would hurt their PR. They have already been slowly tightening the noose around pre-XP legacy software, so that, too isn't as big an issue as it would have been in, say, 2003. 

It would be a band-aid over a sucking chest wound, but it would buy them a couple of years as a fix - and while Microsoft isn't as short-term oriented as most other big corpse-rat cultures, they still think 'strategic long-term planning' is 18 months to two years. It may sound crazy to us engineers and hackers, but given that this approach does seem to be working for them, I will leave it to the MBAs to decide if they have the right of it. 

Even if not, that's two more years, and hey, [https://en.wikibooks.org/wiki/Sufism/Nasrudin#Nasreddin_and_the_Sultan.27s_Horse maybe the horse will sing].

But we were talking about Subleq and the affect that overdosing on Hypeocaine has had on Geri's brain, right? Ooh, now with extra [https://en.wikipedia.org/wiki/Drinking_the_Kool-Aid Kool-Aid] (Bitter Almond flavored, of course)!

-----
Oh, two minor notes regarding the Itanic: first, it was never actually intended for desktop systems, or even workstations - it was, and remains, primarily a server design.  They had tentative plans to make workstation versions (HP actually did make one based on it) and eventually migrate it down to the desktop, but that was predicated on it being successful as a server system first. It hasn't been one for the most part, more due to poor marketing, vitriolic corporate alliances that quickly fell apart, and the whole [https://en.wikipedia.org/wiki/Itanium RAMBUS] disaster (the initial Itanium 2 chipset, the [http://www.informit.com/articles/article.aspx?p=482339&seqNum=6 E8870], was intended to work only with RDRAM, as was the 840 chipset for the Pentium III and the 850 and 860 chipsets for the Pentium 4, and when RDRAM [http://www.geek.com/chips/intels-rambus-mistake-564770/ proved to be more expensive, less reliable, and less performant than existing SDRAM],  they had to scramble to fix them) than due to any issues with the ISA or CPUs themselves.

Second, they still make them, hence the 'remains' part. There was even a new production series, Kittson, that started getting releases in February of [2017], [https://en.wikipedia.org/wiki/Itanium according to Wicked-Pedo]. It never took off, but it never actually went away, either, though it probably will soon now that the HP deal has ended.

-----
'''Geri's reply:''' from the 97 tons of comments, i have time to react on one, and thats the topic of itanium (which someone obviously mentioned as joke). if we see try to try to abstract the architectures in some form of coordinate system using they complexity, subleq is probably is far of a side, and the another end, there is possibly the itanium. itanium is basically a parody of itself, it had 10x more transistors to show up the same performance as an equivalent x86. in theory, all of they instructions is designed to be very very efficient, but actually no one was able to write proper compilers, a real workload rarely needs for example 3 multiplications directly after each other (and thats why the SIMD units are not so efficient in high level codes aniway, intel should be aware of that). itanium is actually (while its much more complex and non-free than even the x86) meant not for servers, it meant to replace the whole x86, including on desktops. intel also convienced microsoft, and a lot of corporations to sit up on they fanwagoon, and as we seen, they failed miserabely. the conclusion of the story is that having extremely complex instruction sets with vliw and simd will just not solve the issues of computing, and i really hope that nobody will try to emulate that.


-----
'''My Reply:''' I expect that both the Intel C compiler team and the GCC team would disagree. They both had solid compilers for it before it was actually on the market, and Linux was successfully ported in less than six months. Who ''didn't'' have compilers? Anyone who was waiting to see if it would get picked up by anyone other than HP, which was pretty much everyone else - including Microsoft.

'''IRT SIMD''', it is a ''very'' common thing in certain workloads, including things like spreadsheets and graphics. 

SIMD is meant for two things: high-performance number crunching, and graphics processing. The move to push video processing (back) onto GPUs, which was just getting going at the time ([http://www.jargon.net/jargon/jargonfile/c/cycleofreincarnation.html cycle of reincarnation], yo), made SIMD a lot less useful for desktop and workstation systems, but it was and remains a real boon for HPC - where it is known as 'vector processing' and is absolutely essential for many heavy CPU-bound simulations (OK, ''real'' vec proc requires multiple '''parallel''' operations, but SIMD is at least a move in the right direction for it). Anything involving large vectors or matrices would benefit from it.

On the desktop? It's pretty much just in spreadsheets, and since you would need either a lot of hand-coded assembly, or a specialized compiler and and a language designed specifically to use the special operations, it probably wouldn't be used even for that initially.

'''Further clarification later:''' Before anyone questions my earlier statement about needing a special language for vector processing: yes, basic forms of automatic vectorization are a thing, but not enough of a thing for this, what with the general problem being NP-Hard and all.

Speaking of which, know what else is probably NP-Hard (assuming that it is tractable at all - I don't know if that has been demonstrated or not but I think it has been)? Optimizing the order of instruction in a UTM. Or anything else that has only a single data-dependent flow of control operation, such as, say, an OISC. Mind you, this applies to RAM machines in general, but having only a single instruction does the compiler devs no favors.

While RISC is generally easier to target than CISC, the main reasons are from orthogonality, regularity, and expanded register files. While OISC is regular to an extreme, it is not orthogonal - it can't be, since all operations look and behave exactly the same. And an OISC has no explicit registers.

In fact, I would be really curious to see just how this C compiler of yours works... C is a really, really good fit for a register machine, but a terrible one for an OISC. Not sure what language would be a good fit for one (something in the Constraint or Dataflow paradigms, maybe, or something for declaratively defining finite state automata), but C (and most of the other Algol family languages) isn't it.
````

<a id="w004092"></a>

## W004092 — User:Schol-r-lea/CPU Implementation

Bron-ID: W004092
Namespace: User (2)
Revisie: 22060 | Datum: 2018-01-12T02:23:13Z
Bron: https://wiki.osdev.org/User:Schol-r-lea/CPU_Implementation
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASchol-r-lea%2FCPU_Implementation&oldid=22060

````text
'''''N.B.: This is one of a group of forum posts extracted so that the information in them can be re-worked into a less confrontational, more informational set of wiki pages. They are here only temporarily.'''''

'''''Note also that part of this discussion came from me mentioning to Geri a flight-of-fancy idea I'd had in the late 1990s, which I called '[[Schol-R-LEA/Reduced State Vector Processor |Reduced State Vector Processor]]' (RSVP), based on the idea of a manycore stack machine with a very small set of registers - four to five - and what for the time were huge caches, where the ALUs attached to the cache sections rather than the CPU cores. It was a daydream, possibly suitable for some future research project but certainly not a workable system as it was, and I was using it to contrast how I saw that one the one hand, and how Geri saw SubLEq, on the other.'''''


Here's a rule of thumb: for hardware CPU implementations, simple instructions operating exclusively on explicit registers, and only load, store, and instruction read operations working on memory, is the most efficient design. In a pseudo-machine (such as JVM), the opposite is true - complex operations with no registers (e.g., a stack machine) work best.

There are exceptions to both, but OISCs aren't among them. The main one right now is, of course, x86, where the complex instructions get broken down, re-merged, re-ordered, and who knows what else by a massively complex instruction pipelining and interpretation mechanism. I am still not convinced that a RISC cannot do better, but right now, none of them do, because no one has any reason to make them better.

Realistically, reconciling either of them into a single model for both native and interpreted code is probably impossible, even for the embodiment of the abstract principle of The Reconciliation of Opposites fnord (don't ask, I don't understand it and never wanted to be it - thanks for the psychological booby prize, Eris!). Reconciling either of them with an OISC is even more daunting.

In any case, none of this really matters, because, who the hell is going to spend $5 billion on the silicon wafer development and design work for it? It's the same problem that Geri has, but at least I recognize that it is a problem. He seems to think that a 'simpler' ISA means a simpler and hence less expensive development program, which is absolutely false (in both particulars - it would require a great deal of silicon to implement, and the baked-in costs of new chip development approach or even outweigh the particular design cost in any case - $2 billion is about the opening cost for any new CPU regardless of complexity, because of how ICs are made). Developing silicon for an OISC - any OISC - as a hardware CPU, regardless of performance, would cost as much as the development of a new x86 CPU microarchitecture, if not more - at least Intel and AMD are on familiar ground with that piece of trash.

-----

https://www.youtube.com/watch?v=CUAXCeRjw3c

https://youtu.be/N7ut61pSLwk

https://youtu.be/FkeRMQzD-0Y

https://youtu.be/hgcNM-6wr34

-----
I also took a look at a video on Mill while looking for it, an architecture which has been mentioned here numerous times before but which I never took the time to really look into, and along the way ran across the central design idea of it, the [https://en.wikipedia.org/wiki/Belt_machine belt machine] model. I think if I ever revisit RSVP as a design idea, I would probably need to consider using the conveyor structure rather than the stack I had originally envisioned. If I ever get my life together, and get into grad school on a PhD track (as I should have done 25 years ago), I might want to see if I can get a project for a prototype RSVP put together in some university's prototyping fab. Where I would get the $1 million to $5 million grants needed for that would be another matter.

I am sure several of you will note the x100 difference in what I am saying this would cost, versus my earlier minimum estimate for developing a commercial product version of an OISC. The key words here are 'prototype' and 'commercial product'. 

Building an OISC using a field-programmable device? pretty much the cost of the FPLA or FPGA, the device programmer (the device that writes to the PLD, not the human software developer), and a breadboard, plus the time to program it - maybe $30 US if you already have the device burner. Building a TTL OISC, maybe $50-$100, and don't even consider how it performs (for the record, terribly, by current standards - lightspeed considerations ensure that no TTL system can compete with integrated silicon, even compared to a programmable logic array, because it will take longer for a signal to go from one chip to another than it would take for the PLD to complete the operation). 

Designing an [https://en.wikipedia.org/wiki/Application-specific_integrated_circuit ASIC] and getting it fabricated? Several thousand dollars, to hundreds of thousands depending on how far you take it, and its performance probably won't be significantly better than the FPGA implementation (they are basically the same technology, but a custom ASIC can be better optimized - sometimes). This, by the way, is the technology most chipsets use, or at least used to (I am guessing it has moved on since then, but I would need to check).

Designing an unoptimized prototype or proof of concept in a small-scale silicon foundry of the sort owned by some universities? $500,000, minimum. Yeah, that's quite a jump. Note that while the cost for creating the RISC-I is usually given as around $100,000 IIRC, that was in 1981 - the adjusted cost would be around $850,000, I think, so the real cost has dropped considerably.

Developing a commercially viable, optimized IC layout and implementing it in a fab's development line? There's the $2 billion I quoted earlier.

-----

There is a world of difference between a hobbyist making an OISC implementation with an FPGA or a hand-wrapped breadboard full of discrete ICs, and even the lowest end consumer-grade implementations. It is like comparing a Ford or a Toyota made in a factory to a life-size model made from Legos by some ambitious teens.

A student project on designing a new architecture using a dinky University-owned fab isn't at all the same as a professional product, either. Interestingly enough, for what it can actually accomplish, the University foundry will cost a lot more overall for the results than the professional foundry, it's just that what can be accomplished by a University fab and what a company like Intel or GlobalFoundry can do are not really comparable.

One of the rarely-mentioned aspects of RISC is that the Berkeley RISC I and Stanford MIPS 1 designs were originally developed because simple, no-frills CPUs with no microcode was the most that the fabs owned by UC-Berkeley and Stanford U. were up to producing, as well as being the most which could be covered in a two-semester graduate courseload. It was a real shock to find that these stripped down architectures proved to be as fast or faster than the commercial microprocessors of the time, and even some of the minicomputers (this was back when a TTL design actually could outperform a contemporary microprocessor, sometimes, due to the limits of IC designs, transistor densities, and silicon real estate of the day). The RISC revolution got underway precisely because of how inefficient the mainstream designs of the time were.

This is itself yet another nail in the coffin of Geri's claim that an OISC can be made by a startup on a shoestring and then sold for US$1 apiece in the sort of volumes that a startup can make them in (i.e., production runs in the hundreds rather than millions, even when going fabless and contracting the actual production - which means that the unit costs would have to be astronomical to defray the development cost).

-----
Note that this is after repeated upgrades; [http://www.computerhistory.org/revolution/digital-logic/12/286/1591 a page I found states] the process used in 1981 was 5μm (5000 nm). I have not been able to find the process density used for the MIPS 1 at Stanford, but I expect it was similar. 

To further put this in context, [https://en.wikipedia.org/wiki/Transistor_count#Microprocessors according to Wicked-Pedo], that's the same process density that Intel was using on the 8085, four years earlier. The process Intel used for the original 8086 in 1978 was 3μm, as were the processes used in the 8088 a year later, the Commodore-Mos Tech WDC-65C02 in 1982, and the ARM I in 1985 (!!!). Just to drive home the point about Intel's lead over most of the other companies, the 68000 was using a 3.5μm process in 1979, the same year as the 8088 - and Motorola was Intel's primary competitor. Intel, AMD, and Acorn were using 350nm processes in 1997, meaning that the gap in time for the implementation of higher density processes has widened tremendously, though it may just be because the universities don't need more (after all, they can only cover so much in a few semesters, and foundries are very expensive - even for research, the schools simply can't match the budgets of the corpse-rat developers).

The take-away:

* Right now, Intel, AMD, and nVidia are just about neck-and-neck, though AMD seems to have pulled ahead right now. No one else seems to comes close to them, even the memory, ASIC, and FPGA manufacturers. Even Oracle is using a 20nm process equivalent to that used on Ivy Bridge, though that may be partly due to Oracle not really giving much of a damn about SPARC or anything else they got from Sun except MySQL (they bought the company primarily to kill  off that single 'product', or at least get control over it; the rest is just excess baggage, and their main interest in them is in how much they can get people to cough up for Java certification classes).
* With the exception of the introduction of 32-bit p-mode in 1985, and 64-bit long mode in 2001, most of the improvements in performance for the x86 (and processors in general) since then have depended on the improved process. These come mainly from lower signal delays, but have also been because they could simply do more per square nm of chip real estate (and now cubic nm, with the multilayer processes). Intel and AMD have thrown a mountain of work on using those improvements to speed up the x86, and are now really good at it, but I can't help but wonder what would have happened if that effort had been put into a design such as MIPS, SPARC (which descends from the Berkeley RISC), PowerPC, Alpha, or even the M680x0 or the [https://en.wikipedia.org/wiki/Transputer Transputer] (and yes, INMOS was a big influence on my RSVP idea). The point is, these speed ups wouldn't even have been possible in 1978, even for the design principles which were already known at the time (which was most of them, actually - most have their roots in the CDC mainframes and Cray minisupers), simply because the chips couldn't have held all of it. The rise of RISC was compensating for something that is no longer an issue. 
* A lot of the reason why RISC was so much better at the time was because most of the commercial microprocessor designers were focusing on features rather than performance. The real take-away from RISC was that trying to do too much in hardware rather than software was actually hurting performance, especially as increased use of compiled languages made the assembly language bells and whistles of something like the VAX or the 68000 redundant.
````

<a id="w004093"></a>

## W004093 — User:Schol-r-lea/OISC Wars Part 3

Bron-ID: W004093
Namespace: User (2)
Revisie: 21131 | Datum: 2017-08-22T21:38:57Z
Bron: https://wiki.osdev.org/User:Schol-r-lea/OISC_Wars_Part_3
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASchol-r-lea%2FOISC_Wars_Part_3&oldid=21131

````text
'''''N.B.: This is one of a group of forum posts extracted so that the information in them can be re-worked into a less confrontational, more informational set of wiki pages. They are here only temporarily.'''''


While I don't think that OISCs make good general-purpose CPUs, I ''can'' see one, or something like one, as the unit of a specialized [https://en.wikipedia.org/wiki/Systolic_array systolic array] that could be used as a signal processor or part of a larger CPU - in other words, as nodes in precisely the sort of DSP or GPU you seem to think are such a bad idea. However, that's a really, really sketchy proposition, and not one I see anyone backing as a production device unless a ''lot'' more research and development proves it useful - it isn't really all that different from existing DSPs, so it would need to show some spectacular advantages to make it usable.

One thing I need to impress is that my statements are not about your interest in Subleq based systems itself, but in the claims you are making about them - and the conspiratorial tone you are using when discussing existing systems. I am all for new ideas, but they need a lot of proving and refining before they can be used in a practical setting.

The Subleq instruction, and the proof that it is Turing-complete, [https://en.wikipedia.org/wiki/One_instruction_set_computer dates back to the mid-1960s]; don't you think that if it were a viable system, someone would have at least ''tried'' to use it in a commercial system by now?  I won't say it will ''never'' work - I have no idea what may happen in the future - but '''today''' it just makes no sense given the technology and the state of our understanding of the idea.

(The [https://en.wikipedia.org/wiki/Paillier_cryptosystem one class of OISCs that has been used commercially], in software, is a specialized one for cryptography and cryptanalysis. Interestingly enough, IBM has also sold [https://en.wikipedia.org/wiki/Zero_instruction_set_computer ZISCs] for specialized neural networking projects as well, though apparently they have discontinued them. Even more esoteric [https://en.wikipedia.org/wiki/Category:Models_of_computation computation models] such as [https://en.wikipedia.org/wiki/Petri_net Petri nets] and even [https://en.wikipedia.org/wiki/Laws_of_Form Calculus of Indications] have also been used; yea, do many things come to pass.) 

There is a saying in some engineering disciplines which the WTFers are fond of repeating: any new idea has to be measured not from 0, but from -100 (or lower). This is in part to compensate for the natural enthusiasm that someone working on an idea has, but also to account for the resistance to change which is inherent in most commercial markets. You have to prove not only that the idea is ''better'' than the existing one, but that it is ''dramatically and overwhelmingly better''. You have yet to do this so far. Maybe you will some day, but so far, no.
````

<a id="w004195"></a>

## W004195 — User:Lkurusa

Bron-ID: W004195
Namespace: User (2)
Revisie: 23036 | Datum: 2018-10-13T09:03:04Z
Bron: https://wiki.osdev.org/User:Lkurusa
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALkurusa&oldid=23036

````text
Hello!

I'm Lev, my website is: http://osdev.me/

I'm lkurusa on Freenode, feel free to nudge me!

User namespace pages of interest:
* [[User:Lkurusa/HiFive1_Bare_Bones]]
````

<a id="w004196"></a>

## W004196 — ARMv7 Generic Timers

Bron-ID: W004196
Namespace: Main (0)
Revisie: 28114 | Datum: 2023-07-09T22:49:12Z
Bron: https://wiki.osdev.org/ARMv7_Generic_Timers
Revisie-URL: https://wiki.osdev.org/index.php?title=ARMv7_Generic_Timers&oldid=28114

````text
This page is current with the description of the ARM Generic Timer behaviour specified in the ARMv7 manual. There may have been differences introduced in ARMv8 and AArch64. This page does not take such differences, if they exist, into account.

This page is meant to be a quick reference and refresher on the way the ARMv7 generic timers work.

= Overview =

The ARM Generic Timers (henceforth, "GT") are architecturally specified in ARMv7 as an <tt>OPTIONAL</tt> extension to the ARMv7-a and ARMv7-r streams. The feature is specified to provide a system-wide timestamp-counter (henceforth, "syscounter") reference which operates independent of the CPU clock's frequency, allowing for TSC measurements which are invariant over time, regardless of processor scaling, power state and throttling.

This upcounter can be read in two forms, and these readout-forms are logically distinct from the actual value of the syscounter (though the physcounter will usually reflect the syscounter exactly):
* Physical upcounter readout ("physcounter"). This is read as a 64-bit read using the CNTPCT register.
* Virtual upcounter readout ("virtcounter"). This is the value of the physical timer, minus a 64-bit virtual offset (CNTVOFF). This is read using the CNTVCT register.

The feature also specifies a set of 4 timers per CPU which base their operation on these two upcounters:
* Secure phystimer: based on 
* Non-secure phystimer: based on 
* Hypervisor timer: based on 
* Virtual timer: based on 

= System counters =

The GT's syscounter is specified to present the following behaviour to software:
* At least 56 bits wide. Any read from the syscounter, physcounter or virtcounter is ''zero-extended'' to 64 bits.
* The syscounter increments at a fixed frequency, with architecturally supported ranges being from 1-50 MHz.
* Supports clock multiplication to save power. I.e, the hardware crystal may operate at a lower frequency than the frequency claimed by the platform, and a multiplier may applied to the readout value to present the illusion of operation at a higher frequency. The underlying crystal's frequency may change dynamically at runtime.
* Roll-over time is guaranteed to be at least 40 years.
* Clock drift is not required to be at any level of accuracy, but ARM <tt>RECOMMENDS</tt> that drift be not greater than 10 seconds per 24 hours. The use of clock multiplication must not change the drift behaviour of the sysupcounter.
* The syscounter is architecturally specified to begin counting from 0 at #RESET.
* The syscounter must be implemented in the always-on power domain.

All agents (including devices in the system other than the host processors) reading the syscounter must be presented with a uniform view of the passage of time.

== Registers ==

The control registers for the system counter are not accessible as coprocessor interface registers, and are only accessible through a memory mapped interface. The controls include enabling/disabling the syscounter, setting the syscounter's value, changing the syscounter's frequency and multiplier, and enabling/disabling halt-on-debug, so that a debugger can halt the syscounter when a processor enters halting debug-mode.

The physical and virtual upcounters as well as the timer registers are architecturally configurable.

=== CNTKCTL ===
Enables PL1 software to control access to the counters and timers from PL0 modes.
* Bits PL0PCTEN and PL0VCTEN enable/disable access to <tt>CNTVCT</tt>, <tt>CNTPCT</tt> and <tt>CNTFREQ</tt> from PL0 modes.
* Bit PL0PTEN enables/disables access to <tt>CNTP_CTL</tt>, <tt>CNTP_CVAL</tt> and <tt>CNTP_TVAL</tt> from PL0 modes.
* Bit PL0VTEN enables/disables access to <tt>CNTV_CTL</tt>, <tt>CNTV_CVAL</tt> and <tt>CNTV_TVAL</tt> from PL0 modes.
* Bits EVENTEN, EVENTDIR and EVENTI enable and control the direction and bit-position of the bit that triggers the events generated by the virtual counter (CNTVCT).

=== CNTHCTL ===
Enables PL2 software to control access to the counters and timers from non-secure PL1 and PL0 modes.
* Bit PL1PCTEN enables/disables access to CNTPCT from non-secure PL1 and PL0 modes.
* PL1PCEN enables/disables access to the phyisical timer registers (CNTP_CTL, CNTP_TVAL, CNTP_CVAL) from non-secure PL1 and PL0 mode.

=== CNTFREQ ===
The <tt>CNTFREQ</tt> register is only ''writeable'' by ''secure'' PL1 mode software, and must be initialized to state the frequency of the system upcounter, '''in Hz'''.
<tt>CNTFREQ</tt> is ''readable'' by PL2, secure and non-secure PL1 by default. <tt>CNTKCTL.PL0PCTEN</tt> enables <tt>CNTFREQ</tt> to be read from secure and non-secure PL0.

=== CNTPCT ===

Use ISB to ensure that CNTPCT reads are executed in order.

* Is always accessible from PL2.
* Is always accessible from PL1 modes. However, if hypervisor extensions are supported accesses to CNTPCT will generate a hyp-trap from PL1 unless CNTHCTL.PL1PCTEN is set to 1.
* Is only accessible from PL0 modes if CNTKCTL.PL0CTEN is set to 1. Attempts to access CNTPCT from PL0 when CNTKCTL.PL0CTEN==0 will trigger an #UNDEFINED exception.

CNTPCT is logically meant to be read by PL2 hypervisors when they need to read a timestamp value. It can be read by '''secure''' PL1 software with no hindrances as well.
There is no statement about whether or not it can be accessed from PL3 monitors.

=== CNTVOFF ===

* Accessible as R/W from PL2.
* Accessible from PL3 (Monitor mode) only when SCR.NS==1.

If the virtualization extensions are not supported, accesses to CNTVOFF from '''PL3''' are <tt>UNPREDICTABLE</tt>, and reads of CNTVCT act as if CNTVOFF is zero.
CNTVOFF is logically meant to be written by a PL2 hypervisor when switching between GuestVMs. It is also writeable by PL3 when SCR.NS=1, but this is not very useful to a PL3 monitor.

=== CNTVCT ===

In a processor that doesn't support the virtualization extensions, CNTVCT is identical to CNTPCT.

* Always accessible from '''secure''' PL1.
* Always accessible from '''non-secure''' PL1 and PL2.
* Accessible from PL0 modes when CNTKCTL.PL0VCTEN==1.

CNTVCT is logically meant to be used by both '''secure''' and '''non-secure''' PL1 kernels. It can be read by PL2 hypervisors, but the usefulness of such a read is questionable, when they can instead read CNTPCT.
There is no statement about whether or not it can be accessed from PL3 monitors.

== Virtual counter Event Streams ==

Events can be triggered by the virtcounter through the use of the CNTKCTL register. You would set the bit position whose transition from 0=>1 or 1=>0 will trigger an event, and then enable the event. The design of this feature obviously means that virtcounter events can only be triggered on power-of-2 counter-values.

= Timers =

* The timers can be used as either count-up or count-down timers.
* The timers can operate in either physical or virtual time.
* The timers output signals can be configured as level-sensitive PPI IRQ signals to the GICC.

== Security Extentions not implemented ==
* Physical timer and virtual timer.

The virtual timer should be used by PL1 kernels. The physical timer should ideally be left alone.

== Security extensions implemented, no Hypervisor extensions implemented ==
* '''non-secure''' physical timer.
* '''Secure''' physical timer.
* Virtual timer.

The virtual timer should be used by non-secure PL1 kernels.
The secure physical timer should be used by PL3 monitors and '''secure''' PL1 kernels.
The non-secure physical timer should be ideally left alone, and seems to exist solely for the sake of supporting non-secure PL1 kernels which insist on using the physcounter and physical registers instead of the virtcounter and virtual registers.

== Security extensions and hypervisor extensions implemented ==
* '''Non-secure''' PL1 physical timer.
* '''Secure''' PL1 physical timer.
* '''Non-secure''' PL2 physical timer.
* Virtual timer.

The virtual timer should be used by non-secure PL1 kernels.
The secure PL1 physical timer should be used by '''secure''' PL1 kernels.
The '''non-secure''' PL2 physical timer should be used by PL2 hypervisors.
The '''non-secure''' PL1 physical timer should be left alone, ideally and seems to exist solely for the sake of supporting non-secure PL1 kernels that insist on using the physcounter and physical registers instead of the virtcounter and virtual registers.

== Registers ==



== CNTFREQ ==

[[Category:ARM]]
[[Category:Timers]]
````

<a id="w004197"></a>

## W004197 — User:A22347/Memory Manager

Bron-ID: W004197
Namespace: User (2)
Revisie: 21505 | Datum: 2017-09-10T20:58:14Z
Bron: https://wiki.osdev.org/User:A22347/Memory_Manager
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AA22347%2FMemory_Manager&oldid=21505

````text
Here is the memory manager for my kernel. Feel free to use.

<source lang="c">
#define BLOCK_SIZE 64
#define MEMORY_BASE 0x18000000
#define MEMORY_HIGH 0x80000000 
#define MEMORY (MEMORY_HIGH - MEMORY_BASE)
#define BLOCKS (MEMORY / BLOCK_SIZE)

char blocksUsed[BLOCKS] = { 0 };
int tried = 0;
uint8_t* lastFreeAddress = MEMORY_BASE;
uint8_t* lastMallocAddress = MEMORY_BASE;

void* malloc (size_t size)
{
	tried = 0;
	int stage = 0;
	uint8_t* ptr = lastFreeAddress;
	unsigned int blocksAlloced = 0;
	while (blocksAlloced < size) {
		if (blocksUsed[(((unsigned int) ptr + blocksAlloced) - MEMORY_BASE) / BLOCK_SIZE] == 0) {
			blocksAlloced += BLOCK_SIZE ;
		} else {
			ptr += blocksAlloced + BLOCK_SIZE ;

			if (stage == 0) {
				stage = 1;
				ptr = lastMallocAddress;
			}

			blocksAlloced = 0;
		}
	}
	for (int i = 0; i < blocksAlloced / BLOCK_SIZE ; ++i) {
		if (i == blocksAlloced / BLOCK_SIZE - 1) {
			blocksUsed[((((unsigned int) ptr) - MEMORY_BASE) / BLOCK_SIZE) + i] = 2;		//show that this is the last in the block
		} else {
			blocksUsed[((((unsigned int) ptr) - MEMORY_BASE) / BLOCK_SIZE) + i] = 1;
		}

	}
	lastMallocAddress = ptr + blocksAlloced;
	freeBytesApprox -= blocksAlloced;
	return ptr;
}

void free (void *blk)
{
	unsigned int locator = blk;
	while (blocksUsed[((locator - MEMORY_BASE) / BLOCK_SIZE)]) {
		if (blocksUsed[((locator - MEMORY_BASE) / BLOCK_SIZE)] == 2) {
			blocksUsed[((locator - MEMORY_BASE) / BLOCK_SIZE)] = 0;
			break;
		}
		blocksUsed[((locator - MEMORY_BASE) / BLOCK_SIZE)] = 0;
		locator += BLOCK_SIZE;
	}
	lastFreeAddress = blk;
	freeBytesApprox += locator - (int) blk + BLOCK_SIZE;
}

</source>
````

<a id="w004198"></a>

## W004198 — Talk:Context Switching

Bron-ID: W004198
Namespace: Talk (1)
Revisie: 22558 | Datum: 2018-08-07T04:19:54Z
Bron: https://wiki.osdev.org/Talk:Context_Switching
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AContext_Switching&oldid=22558

````text
This whole page needs to be changed so that "context switch" is replaced with "task switch"; because there are many kinds of contexts and many kinds of context switches (where task switches is only one of them). It's like having a page with the title "Colour" that only talks about blue that says things like "There are two main colours, light blue and dark blue".
````

<a id="w004199"></a>

## W004199 — LLVM

Bron-ID: W004199
Namespace: Main (0)
Revisie: 26382 | Datum: 2021-09-06T12:43:56Z
Bron: https://wiki.osdev.org/LLVM
Revisie-URL: https://wiki.osdev.org/index.php?title=LLVM&oldid=26382

````text
LLVM is a compiler infrastructure for several languages. It consists of clang as the C compiler, the LLVM library (includes [http://blog.llvm.org/2010/04/intro-to-llvm-mc-project.html llvm-mc] as the assembler) and lld as the linker. The LLVM library can be used to create several languages like Rust.

== OS Development with LLVM ==

LLVM recently has been used in development of BSDs.

One good thing about LLVM is that it is cross-compiler by design.

The following command gives the supported architectures

 llc -version

As it is a cross compiler by default, clang can called to compile i686-elf for the simply by

 clang -target i686-elf

To link the object files, simply use <code>ld.lld</code>, as for LLD, architectures doesn't matter.

=== Building LLVM ===
{{Main|LLVM Cross-Compiler}}

[[Category:Tools]]
[[Category:Compilers]]
[[Category:Linkers]]
[[Category:Assemblers]]
````

<a id="w004200"></a>

## W004200 — User:A22347/Printf

Bron-ID: W004200
Namespace: User (2)
Revisie: 25267 | Datum: 2020-10-27T16:33:49Z
Bron: https://wiki.osdev.org/User:A22347/Printf
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AA22347%2FPrintf&oldid=25267

````text
Here is an example printf implementation. It does not support hexadecimal floating point numbers.
It can easily be ported into a kernel or userspace library as it only relies on a small number of standard functions.

<source lang="c">

#include <stdio.h>         //putchar
#include <string.h>        //strcpy, strcat, memcpy, memset
#include <ctype.h>         //isdigit
#include <stdbool.h> 
#include <stdint.h>
#include <stdarg.h>

char* __int_str(intmax_t i, char b[], int base, bool plusSignIfNeeded, bool spaceSignIfNeeded,
                int paddingNo, bool justify, bool zeroPad) {
    
    char digit[32] = {0};
    memset(digit, 0, 32);
    strcpy(digit, "0123456789");
    
    if (base == 16) {
        strcat(digit, "ABCDEF");
    } else if (base == 17) {
        strcat(digit, "abcdef");
        base = 16;
    }
    
    char* p = b;
    if (i < 0) {
        *p++ = '-';
        i *= -1;
    } else if (plusSignIfNeeded) {
        *p++ = '+';
    } else if (!plusSignIfNeeded && spaceSignIfNeeded) {
        *p++ = ' ';
    }
    
    intmax_t shifter = i;
    do {
        ++p;
        shifter = shifter / base;
    } while (shifter);
    
    *p = '\0';
    do {
        *--p = digit[i % base];
        i = i / base;
    } while (i);
    
    int padding = paddingNo - (int)strlen(b);
    if (padding < 0) padding = 0;
    
    if (justify) {
        while (padding--) {
            if (zeroPad) {
                b[strlen(b)] = '0';
            } else {
                b[strlen(b)] = ' ';
            }
        }
        
    } else {
        char a[256] = {0};
        while (padding--) {
            if (zeroPad) {
                a[strlen(a)] = '0';
            } else {
                a[strlen(a)] = ' ';
            }
        }
        strcat(a, b);
        strcpy(b, a);
    }
    
    return b;
}

void displayCharacter(char c, int* a) {
    putchar(c);
    *a += 1;
}

void displayString(char* c, int* a) {
    for (int i = 0; c[i]; ++i) {
        displayCharacter(c[i], a);
    }
}

int vprintf (const char* format, va_list list)
{
    int chars        = 0;
    char intStrBuffer[256] = {0};
    
    for (int i = 0; format[i]; ++i) {
        
        char specifier   = '\0';
        char length      = '\0';
        
        int  lengthSpec  = 0; 
        int  precSpec    = 0;
        bool leftJustify = false;
        bool zeroPad     = false;
        bool spaceNoSign = false;
        bool altForm     = false;
        bool plusSign    = false;
        bool emode       = false;
        int  expo        = 0;
        
        if (format[i] == '%') {
            ++i;
            
            bool extBreak = false;
            while (1) {
            
                switch (format[i]) {
                    case '-':
                        leftJustify = true;
                        ++i;
                        break;
                        
                    case '+':
                        plusSign = true;
                        ++i;
                        break;
                        
                    case '#':
                        altForm = true;
                        ++i;
                        break;
                        
                    case ' ':
                        spaceNoSign = true;
                        ++i;
                        break;
                        
                    case '0':
                        zeroPad = true;
                        ++i;
                        break;
                        
                    default:
                        extBreak = true;
                        break;
                }
                
                if (extBreak) break;
            }
            
            while (isdigit(format[i])) {
                lengthSpec *= 10;
                lengthSpec += format[i] - 48;
                ++i;
            }
            
            if (format[i] == '*') {
                lengthSpec = va_arg(list, int);
                ++i;
            }
            
            if (format[i] == '.') {
                ++i;
                while (isdigit(format[i])) {
                    precSpec *= 10;
                    precSpec += format[i] - 48;
                    ++i;
                }
                
                if (format[i] == '*') {
                    precSpec = va_arg(list, int);
                    ++i;
                }
            } else {
                precSpec = 6;
            }
            
            if (format[i] == 'h' || format[i] == 'l' || format[i] == 'j' ||
                   format[i] == 'z' || format[i] == 't' || format[i] == 'L') {
                length = format[i];
                ++i;
                if (format[i] == 'h') {
                    length = 'H';
                } else if (format[i] == 'l') {
                    length = 'q';
                    ++i;
                }
            }
            specifier = format[i];
            
            memset(intStrBuffer, 0, 256);
            
            int base = 10;
            if (specifier == 'o') {
                base = 8;
                specifier = 'u';
                if (altForm) {
                    displayString("0", &chars);
                }
            }
            if (specifier == 'p') {
                base = 16;
                length = 'z';
                specifier = 'u';
            }
            switch (specifier) {
                case 'X':
                    base = 16;
                case 'x':
                    base = base == 10 ? 17 : base;
                    if (altForm) {
                        displayString("0x", &chars);
                    }
                    
                case 'u':
                {
                    switch (length) {
                        case 0:
                        {
                            unsigned int integer = va_arg(list, unsigned int);
                            __int_str(integer, intStrBuffer, base, plusSign, spaceNoSign, lengthSpec, leftJustify, zeroPad);
                            displayString(intStrBuffer, &chars);
                            break;
                        }
                        case 'H':
                        {
                            unsigned char integer = (unsigned char) va_arg(list, unsigned int);
                            __int_str(integer, intStrBuffer, base, plusSign, spaceNoSign, lengthSpec, leftJustify, zeroPad);
                            displayString(intStrBuffer, &chars);
                            break;
                        }
                        case 'h':
                        {
                            unsigned short int integer = va_arg(list, unsigned int);
                            __int_str(integer, intStrBuffer, base, plusSign, spaceNoSign, lengthSpec, leftJustify, zeroPad);
                            displayString(intStrBuffer, &chars);
                            break;
                        }
                        case 'l':
                        {
                            unsigned long integer = va_arg(list, unsigned long);
                            __int_str(integer, intStrBuffer, base, plusSign, spaceNoSign, lengthSpec, leftJustify, zeroPad);
                            displayString(intStrBuffer, &chars);
                            break;
                        }
                        case 'q':
                        {
                            unsigned long long integer = va_arg(list, unsigned long long);
                            __int_str(integer, intStrBuffer, base, plusSign, spaceNoSign, lengthSpec, leftJustify, zeroPad);
                            displayString(intStrBuffer, &chars);
                            break;
                        }
                        case 'j':
                        {
                            uintmax_t integer = va_arg(list, uintmax_t);
                            __int_str(integer, intStrBuffer, base, plusSign, spaceNoSign, lengthSpec, leftJustify, zeroPad);
                            displayString(intStrBuffer, &chars);
                            break;
                        }
                        case 'z':
                        {
                            size_t integer = va_arg(list, size_t);
                            __int_str(integer, intStrBuffer, base, plusSign, spaceNoSign, lengthSpec, leftJustify, zeroPad);
                            displayString(intStrBuffer, &chars);
                            break;
                        }
                        case 't':
                        {
                            ptrdiff_t integer = va_arg(list, ptrdiff_t);
                            __int_str(integer, intStrBuffer, base, plusSign, spaceNoSign, lengthSpec, leftJustify, zeroPad);
                            displayString(intStrBuffer, &chars);
                            break;
                        }
                        default:
                            break;
                    }
                    break;
                }
                    
                case 'd':
                case 'i':
                {
                    switch (length) {
                    case 0:
                    {
                        int integer = va_arg(list, int);
                        __int_str(integer, intStrBuffer, base, plusSign, spaceNoSign, lengthSpec, leftJustify, zeroPad);
                        displayString(intStrBuffer, &chars);
                        break;
                    }
                    case 'H':
                    {
                        signed char integer = (signed char) va_arg(list, int);
                        __int_str(integer, intStrBuffer, base, plusSign, spaceNoSign, lengthSpec, leftJustify, zeroPad);
                        displayString(intStrBuffer, &chars);
                        break;
                    }
                    case 'h':
                    {
                        short int integer = va_arg(list, int);
                        __int_str(integer, intStrBuffer, base, plusSign, spaceNoSign, lengthSpec, leftJustify, zeroPad);
                        displayString(intStrBuffer, &chars);
                        break;
                    }
                    case 'l':
                    {
                        long integer = va_arg(list, long);
                        __int_str(integer, intStrBuffer, base, plusSign, spaceNoSign, lengthSpec, leftJustify, zeroPad);
                        displayString(intStrBuffer, &chars);
                        break;
                    }
                    case 'q':
                    {
                        long long integer = va_arg(list, long long);
                        __int_str(integer, intStrBuffer, base, plusSign, spaceNoSign, lengthSpec, leftJustify, zeroPad);
                        displayString(intStrBuffer, &chars);
                        break;
                    }
                    case 'j':
                    {
                        intmax_t integer = va_arg(list, intmax_t);
                        __int_str(integer, intStrBuffer, base, plusSign, spaceNoSign, lengthSpec, leftJustify, zeroPad);
                        displayString(intStrBuffer, &chars);
                        break;
                    }
                    case 'z':
                    {
                        size_t integer = va_arg(list, size_t);
                        __int_str(integer, intStrBuffer, base, plusSign, spaceNoSign, lengthSpec, leftJustify, zeroPad);
                        displayString(intStrBuffer, &chars);
                        break;
                    }
                    case 't':
                    {
                        ptrdiff_t integer = va_arg(list, ptrdiff_t);
                        __int_str(integer, intStrBuffer, base, plusSign, spaceNoSign, lengthSpec, leftJustify, zeroPad);
                        displayString(intStrBuffer, &chars);
                        break;
                    }
                    default:
                        break;
                    }
                    break;
                }
                    
                case 'c':
                {
                    if (length == 'l') {
                        displayCharacter(va_arg(list, wint_t), &chars);
                    } else {
                        displayCharacter(va_arg(list, int), &chars);
                    }
                    
                    break;
                }
                    
                case 's':
                {
                    displayString(va_arg(list, char*), &chars);
                    break;
                }
                    
                case 'n':
                {
                    switch (length) {
                        case 'H':
                            *(va_arg(list, signed char*)) = chars;
                            break;
                        case 'h':
                            *(va_arg(list, short int*)) = chars;
                            break;
                            
                        case 0: {
                            int* a = va_arg(list, int*);
                            *a = chars;
                            break;
                        }
                            
                        case 'l':
                            *(va_arg(list, long*)) = chars;
                            break;
                        case 'q':
                            *(va_arg(list, long long*)) = chars;
                            break;
                        case 'j':
                            *(va_arg(list, intmax_t*)) = chars;
                            break;
                        case 'z':
                            *(va_arg(list, size_t*)) = chars;
                            break;
                        case 't':
                            *(va_arg(list, ptrdiff_t*)) = chars;
                            break;
                        default:
                            break;
                    }
                    break;
                }
                   
                case 'e':
                case 'E':
                    emode = true;
                    
                case 'f':
                case 'F':
                case 'g':
                case 'G':
                {
                    double floating = va_arg(list, double);
                    
                    while (emode && floating >= 10) {
                        floating /= 10;
                        ++expo;
                    }
                    
                    int form = lengthSpec - precSpec - expo - (precSpec || altForm ? 1 : 0);
                    if (emode) {
                        form -= 4;      // 'e+00'
                    }
                    if (form < 0) {
                        form = 0;
                    }
                    
                    __int_str(floating, intStrBuffer, base, plusSign, spaceNoSign, form, \
                              leftJustify, zeroPad);
                    
                    displayString(intStrBuffer, &chars);
                    
                    floating -= (int) floating;
                    
                    for (int i = 0; i < precSpec; ++i) {
                        floating *= 10;
                    }
                    intmax_t decPlaces = (intmax_t) (floating + 0.5);
                    
                    if (precSpec) {
                        displayCharacter('.', &chars);
                        __int_str(decPlaces, intStrBuffer, 10, false, false, 0, false, false);
                        intStrBuffer[precSpec] = 0;
                        displayString(intStrBuffer, &chars);
                    } else if (altForm) {
                        displayCharacter('.', &chars);
                    }
                    
                    break;
                }
                    
                    
                case 'a':
                case 'A':
                    //ACK! Hexadecimal floating points...
                    break;
                
                default:
                    break;
            }
            
            if (specifier == 'e') {
                displayString("e+", &chars);
            } else if (specifier == 'E') {
                displayString("E+", &chars);
            }
            
            if (specifier == 'e' || specifier == 'E') {
                __int_str(expo, intStrBuffer, 10, false, false, 2, false, true);
                displayString(intStrBuffer, &chars);
            }
            
        } else {
            displayCharacter(format[i], &chars);
        }
    }
    
    return chars;
}

__attribute__ ((format (printf, 1, 2))) int printf (const char* format, ...)
{
    va_list list;
    va_start (list, format);
    int i = vprintf (format, list);
    va_end (list);
    return i;
    
}
</source>
````

<a id="w004201"></a>

## W004201 — USTAR

Bron-ID: W004201
Namespace: Main (0)
Revisie: 30314 | Datum: 2026-03-17T18:59:46Z
Bron: https://wiki.osdev.org/USTAR
Revisie-URL: https://wiki.osdev.org/index.php?title=USTAR&oldid=30314

````text
{{Filesystems}}

USTAR is the POSIX standard form of TAr, Unix's ''tape archive'' format. It has become a very common general-purpose archive format; tools to create tar archives are available for every OS. It is also very very simple. As USTAR is a POSIX standard (POSIX.1-1988 and POSIX.1-2001), it's well defined and very well documented. You can also find a lot of example code, and the GNU tar utility is Open Source.

As an archive format, USTAR is not strictly a filesystem, but can very easily be used as one in read-only mode. Writing files is allowed under the standard, but only by appending complete updated copies of the files to be changed. This restriction makes USTAR unsuitable for a general-purpose writeable filesystem. However, read-only ''tarfs'' implementations have their place. Notably, the Plan 9 and Inferno OSs developed by Ken Thompson and Dennis Ritchie included it with other archive filesystems, all read-only.

Tar archives may be written direct to floppy disks or partitions. It's designed for media which may be longer than the archive; it's fine with trailing garbage. Its block size, 512 bytes, formerly corresponded with that of most storage media, but this is of little importance unless you read or write one block at a time.

== Format Details ==

Each file and directory has a 512 bytes sector containing meta data (i-node with filename if you like). If the file is not empty, then that meta data sector is followed by data sectors with file contents rounded up to 512 bytes.

{| {{wikitable}} cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
|-
!Offset
!Size
!Description
|-
|0
|100
|File name
|-
|100
|8
|File mode
|-
|108
|8
|Owner's numeric user ID
|-
|116
|8
|Group's numeric user ID
|-
|124
|12
|File size in bytes (octal base)
|-
|136
|12
|Last modification time in numeric Unix time format (octal)
|-
|148
|8
|Checksum for header record
|-
|156
|1
|Type flag
|-
|157
|100
|Name of linked file
|-
|257
|6
|UStar indicator "ustar" then NUL
|-
|263
|2
|UStar version "00"
|-
|265
|32
|Owner user name
|-
|297
|32
|Owner group name
|-
|329
|8
|Device major number
|-
|337
|8
|Device minor number
|-
|345
|155
|Filename prefix
|}

The only trick is, that file size is not stored in binary, rather in an ASCII octal string. For example 1025 is stored as '000000002001'.

The field Type flag tells what kind of file it's about.

{| {{wikitable}} cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
|-
!Type flag
!Meaning
|-
|'0' or (ASCII NUL)
|Normal file
|-
|'1'
|Hard link
|-
|'2'
|Symbolic link
|-
|'3'
|Character device
|-
|'4'
|Block device
|-
|'5'
|Directory
|-
|'6'
|Named pipe (FIFO)
|}

Pretty much that's all you need to know for a basic implementation.

== Example Code ==

We need a helper function to convert ASCII octal number into binary:
<syntaxhighlight lang="c">
int oct2bin(unsigned char *str, int size) {
    int n = 0;
    unsigned char *c = str;
    while (size-- > 0) {
        n *= 8;
        n += *c - '0';
        c++;
    }
    return n;
}
</syntaxhighlight>

Then file lookup is as simple as:
<syntaxhighlight lang="c">
/* returns file size and pointer to file data in out */
int tar_lookup(unsigned char *archive, char *filename, char **out) {
    unsigned char *ptr = archive;

    while (!memcmp(ptr + 257, "ustar", 5)) {
        int filesize = oct2bin(ptr + 0x7c, 11);
        if (!memcmp(ptr, filename, strlen(filename) + 1)) {
            *out = ptr + 512;
            return filesize;
        }
        ptr += (((filesize + 511) / 512) + 1) * 512;
    }
    return 0;
}
</syntaxhighlight>
If you do not load the entire archive into memory, then you will have to load the sector from the disk in the beginning of the loop (before 'int filesize'), and seeking to the next sector (or skipping more) at the end of the loop (instead of 'ptr +=').

== See Also ==
=== External Links ===
* [https://github.com/calccrypto/tar Example implementation] on github with file creation and removal, MIT licensed
* [https://www.ibm.com/support/knowledgecenter/en/ssw_aix_71/com.ibm.aix.files/tar.h.htm Tar format details] at IBM Knowledge Center
* [http://www.gnu.org/software/tar GNU tar] for Linux and UNIX like OSes
* [http://gnuwin32.sourceforge.net/packages/gtar.htm Tar for Windows]
* [https://en.wikipedia.org/wiki/Tar_(computing) Wikipedia on USTAR]
* [https://chromium.googlesource.com/native_client/nacl-newlib/+/refs/heads/master/newlib/libc/include/tar.h newlib tar header file]

[[Category:Filesystems]]
[[Category:File Formats]]
````

<a id="w004203"></a>

## W004203 — SRAT

Bron-ID: W004203
Namespace: Main (0)
Revisie: 30858 | Datum: 2026-07-07T17:21:51Z
Bron: https://wiki.osdev.org/SRAT
Revisie-URL: https://wiki.osdev.org/index.php?title=SRAT&oldid=30858

````text
{{ACPI}}

''This page is about the ACPI SRAT (System/Static Resource Affinity Table)''
It describes how the [[APIC]] works.
==Introduction==
The SRAT table is described by the ACPI Specification 5.0a [http://www.acpi.info/DOWNLOADS/ACPI_5_Errata%20A.pdf] in section 5.2.16 as a table providing information that allows an OSPM (Operating System Power Management [System]) "to associate processors and memory
ranges, including ranges of memory provided by hot-added memory devices, with system localities proximity domains and clock domains." Basically, the SRAT provides information on relationships of resources in the system - namely the processor(s) and memory.
The SRAT table associates each processor and blocks of memory to a plain integer, named ''proximity domains'' in ACPI jargon.

==Header Structure==
The SRAT table follows this structure:

<syntaxhighlight lang="c">
struct SRAT
{
    char signature[4];   // Contains "SRAT"
    uint32_t length;     // Length of entire SRAT including entries
    uint8_t  rev;        // 3
    uint8_t  checksum;   // Entire table must sum to zero
    uint8_t  OEMID[6];
    uint64_t OEMTableID; // For the SRAT it's the manufacturer model ID
    uint32_t OEMRev;     // OEM revision for OEM Table ID
    uint32_t creatorID;  // Vendor ID of the utility used to create the table
    uint32_t creatorRev;
    
    uint8_t reserved[12];
} __attribute__((packed));
</syntaxhighlight>

This header structure (48 bytes) is then followed by a list of Static Resource Allocation Structures. There are 3 types of SRAS:
{| class="wikitable"
|-
! Type
! Name
|-
| 0x0
| Processor Local APIC/SAPIC Affinity Structure
|-
| 0x1
| Memory Affinity Structure
|-
| 0x2
| Processor Local x2APIC Affinity Structure
|-
|}

==Processor Local APIC Affinity Structure==
This structure describes the association of a processor's APIC ID (or SAPIC ID/EID) and the proximity domain (in ACPI jargon) in which it belongs to.

<syntaxhighlight lang="c">
struct SRAT_proc_lapic_struct
{
    uint8_t type;      // 0x0 for this type of structure
    uint8_t length;    // 16
    uint8_t lo_DM;     // Bits [0:7] of the proximity domain
    uint8_t APIC_ID;   // Processor's APIC ID
    uint32_t flags;    // Flags (see below)
    uint8_t SAPIC_EID; // The processor's local SAPIC EID.
    uint8_t hi_DM[3];  // Bits [8:31] of the proximity domain
    uint32_t _CDM;     // The clock domain which the processor belongs to (more jargon)
} __attribute__((packed));
</syntaxhighlight>

Currently, only one flag is defined for the <code>flags</code> field:

{| class="wikitable"
|-
! Field name
! Field bit width
! Bit offset
! Description
|-
| Enabled
| 1
| 0
| If clear, the OS should ignore the contents of the structure. This is to allow system firmware to populate the SRAT with a static number of entries and enable as needed.
|-
| ''Reserved''
| 31
| 1
| Must be zero.
|-
|}

==Memory Affinity Structure==
The Memory Affinity structure contains:
* A range of memory and the unexciting integer (proximity domain) to which it belongs
* Information about whether the range of memory is hot-pluggable

Keep in mind, that there can be several Memory Affinity structures corresponding to single proximity domain due to various memory holes (e.g. [[Memory Map (x86)#:~:text=384 KiB System / Reserved ("Upper Memory")|384 KiB Reserved ("Upper Memory")]] hole may be present even on ([[QEMU]]) [[UEFI]] machines). For example, 1GiB domain may have <tt>(0x0, 0x80000)</tt> and <tt>(0x100000 - 0x40000000)</tt> Memory Affinities.

<syntaxhighlight lang="c">
struct SRAT_mem_struct
{
    uint8_t type;         // 0x1 for this type of structure
    uint8_t length;       // 40
    uint32_t domain;      // The domain to which this memory region belongs to
    uint8_t reserved1[2]; // Reserved
    uint32_t lo_base;     // Low 32 bits of the base address of the memory range
    uint32_t hi_base;     // High 32 bits of the base address of the memory range
    uint32_t lo_length;   // Low 32 bits of the length of the range
    uint32_t hi_length;   // High 32 bits of the length
    uint8_t reserved2[4]; // Reserved
    uint32_t flags;       // Flags (see below)
    uint8_t reserved3[8]; // Reserved
} __attribute__ ((packed));
</syntaxhighlight>

Flags:

{| class="wikitable"
|-
! Field name
! Field bit width
! Bit offset
! Description
|-
| Enabled
| 1
| 0
| If clear, the OS should ignore the contents of the structure. This is to allow system firmware to populate the SRAT with a static number of entries and enable as needed.
|-
| Hot-pluggable*
| 1
| 1
| If both Enabled and Hot-pluggable flags are set, then the system supports hot-adding and hot-removing of this memory region.
|-
| Non-Volatile
| 1
| 2
| If set, this memory region is non-volatile memory
|-
| ''Reserved''
| 29
| 3
| Must be zero.
|-
|}

&#42; The asterisk says nothing

==Processor Local x2APIC Affinity Structure==
The x2APIC structure is basically the [[API]] C structure but for x2APIC.

<syntaxhighlight lang="c">
struct SRAT_proc_lapic2_struct
{
    uint8_t type;         // 0x2 for this type of structure
    uint8_t length;       // 24
    uint8_t reserved1[2]; // Must be zero
    uint32_t domain;      // The proximity domain which the logical processor belongs to
    uint32_t x2APIC_ID;   // Processor's x2APIC ID
    uint32_t flags;
    uint32_t _CDM;        // The clock domain which the processor belongs to (more jargon)
    uint8_t reserved2[4]; // Reserved.
} __attribute__((packed));
</syntaxhighlight>

==Clock Domain==
Refer to the documentation (section 6.2.1).
The clock domain represents the clock connected to a processor of that domain. When a processor attempts to access a clock source that is not in the same clock domain as itself, inaccuracies may occur when compared to other processors. This is the same concept as a proximity domain, where processors accessing memory or I/O resources outside its domain will need to ask a processor inside the domain to perform the memory or I/O action.


[[Category:ACPI]]
````

<a id="w004204"></a>

## W004204 — User:Joshw

Bron-ID: W004204
Namespace: User (2)
Revisie: 21574 | Datum: 2017-09-27T16:28:22Z
Bron: https://wiki.osdev.org/User:Joshw
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJoshw&oldid=21574

````text
https://github.com/joshwyant
````

<a id="w004206"></a>

## W004206 — Sysroot

Bron-ID: W004206
Namespace: Main (0)
Revisie: 21614 | Datum: 2017-10-14T16:27:18Z
Bron: https://wiki.osdev.org/Sysroot
Revisie-URL: https://wiki.osdev.org/index.php?title=Sysroot&oldid=21614
Doorverwijzing: Meaty Skeleton

````text
#REDIRECT [[Meaty_Skeleton#System_Root]]
````

<a id="w004207"></a>

## W004207 — User:Schol-r-lea/Understanding RISC vs CISC

Bron-ID: W004207
Namespace: User (2)
Revisie: 22457 | Datum: 2018-06-12T16:19:54Z
Bron: https://wiki.osdev.org/User:Schol-r-lea/Understanding_RISC_vs_CISC
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASchol-r-lea%2FUnderstanding_RISC_vs_CISC&oldid=22457

````text
'''For more on the background of this topic, see [[Historical Notes on CISC and RISC]].'''

The RISC vs CISC issue is often badly misunderstood. The main reasons RISC is often seen as an improvement over CISC are

*    Elimination of rarely-used instructions (which were originally intended to make assembly programming easier, but add complications to compiler optimization) reduces the design cost, saves silicon real estate (meaning it can use fewer transistors, or use the same number of transistors to add things like caches), and reduces or eliminates the need for microcoding in the instruction decoding and execution.
*    Using only simple instructions (a separate issue from the one above) makes for faster instruction throughput (since everything fits into one cycle without having to make a cycle excessively long), reduces energy consumption, and reduces design and production overhead.
*    Fixed-size instruction formats mean that the instruction decoder always knows exactly how large the instruction it needs to pre-fetch is (at the cost of less efficient memory usage for common instructions, hence the later retrofitting of the 16-bit Thumb instructions on to the ARM).
*    Elimination of multiple addressing modes for data operations means that each instruction always accesses data in the same way, with most instructions only operating on registers (with value offsets - often used for indexing - being handled via constant fields in the instructions themselves).
*    Very large register files, with all or almost all registers being general-purpose, simplifies compiler implementation and optimization. The instruction pointer in a RISC is usually special-purpose, though exceptions exist even for that. Things like the stack and frame pointers are usually set by convention and assembly-language naming rather than hardwired.
*    The use of register-based conventions (or register windows) for handling procedure call returns, and the passing of procedure arguments and return values, allows procedures to only use the in-memory stack as needed, rather than requiring it in any procedure call. This makes optimizing the calls easier for a compiler writer.


In addition, many CISCy designs - including the 8086 - were just poorer designs, period. They often had cripplingly small register files (or even were accumulator-based systems, like the 6502, with only a single general register that was the implicit argument of most instructions), were often filled with oddball exceptions and special cases, and had various kludges in how the memory was handled (not just segmentation; for example, the 'Page Zero' memory and fixed 'Page 1' stack on the 6502). This was by no means universal, however; the VAX and the Motorola 68000, two designs which epitomized CISC, had very cleanly designed ISAs with consistent addressing formats and large (for the time) register sets.

At first, RISCs also had the advantage that even a heavily optimized design could fit onto a single-chip CPU, whereas many optimizations used in CISC systems - such as out-of-order execution and the use of large, multi-level associative memory caches - wouldn't, given the transistor densities of the 1980s and early to mid 1990s. While the other advantages were independent of the CPU implementation, this was an issue with any CPU designed then, and made RISC a very appealing option in that period.

Finally, a single chip design will always have an advantage over a multi-chip or TTL one simply due to the smaller lightspeed delay. However, early on this had to be balanced against the ability of larger systems to parallelize their instruction pipelines. Also, a multi-chip system using dozens or even hundreds of smaller, special-purpose CPUs such as DSPs - or even general-purpose ones, such as in the Connection Machine and the Hypercube - could take this even further, as seen in pretty much every supercomputer in use today, which might combine dozens or even hundreds of general-purpose processors and thousands of GPGPU and DSP units to crunch a single computation.

The single-chip vs. TTL issue ceased to be a factor in the early 1990s as it became possible to put increasingly complex architectures onto a single chip - today, no one except retrocomputing hobbyists build individual CPUs out of multiple chips. However, the smaller footprint of the CPU cores gave RISC designs an early advantage in the move to multiple-core CPUs, though this was not really exploited for non-technical reasons, and by the time multicore ARM and MIPS CPUs were common, the CISC designs had caught up.

However, as Moore's Law ground on, the advantages in silicon real estate melted away, as existing CISC optimizations were mated to ones specific to IC designs. Modern superscalar CISC designs get most of the same advantages as RISCs, but do so by throwing hardware and electricity at the problem (by the use of complex pipelining, instruction re-ordering, register renaming over large hidden register files, various forms of instruction merging and splitting, and especially, caching). Furthermore, by performing the splitting of complex instructions by dynamic binary translation, rather than using the older method of microcoded instructions, it gave the system more flexibility in instruction reordering.

All of these optimizations came at significant cost in terms of both silicon real estate and power consumption, but by the mid-1990s, Moore's Law meant that not only could more transistors be fit onto a chip, but over all power consumption was dropping due to the shorter distances between transistors. By the early 2000s, even RISC systems were adopting some of these methods to improve their performance, most notably Dynamic Out-of-Order Execution (DOoOX), though due to the markets they soon found themselves primarily in (mobile devices and other low-power-consumption uses such as single board computers) they limited this in order to maintain their advantages in lower heat generation and power consumption.

Today, RISC vs CISC is less about raw computing power (though RISC advocates, including myself, often argue that it has the potential for it if the same amount of design effort were applied to it) than over the costs of new chip development and improvements to the designs (Intel pours billions into each new chip generation), retail costs per unit, and the drastic differences in energy consumption (DOoOX in particular is immensely energy-intensive, and accounts for around 90% of the wattage needed by current x86 chips, which is why RISC designs which do apply it do so sparingly).
````

<a id="w004208"></a>

## W004208 — Mach-O

Bron-ID: W004208
Namespace: Main (0)
Revisie: 30179 | Datum: 2026-03-09T18:36:53Z
Bron: https://wiki.osdev.org/Mach-O
Revisie-URL: https://wiki.osdev.org/index.php?title=Mach-O&oldid=30179

````text
{{Stub}}
{{File formats}}

Mach-O is an object, shared library and executable format used in systems based on the Mach kernel, most notably MacOS.

== See Also ==
=== External Links ===
* [https://en.wikipedia.org/wiki/Mach-O Mach-O Format on Wikipedia]
* [https://lowlevelbits.org/parsing-mach-o-files/ Parsing Mach-O files]
* [https://github.com/aidansteele/osx-abi-macho-file-format-reference Backup of the OS X ABI Mach-O File Format Reference]
* [https://github.com/apple-oss-distributions/xnu/blob/main/EXTERNAL_HEADERS/mach-o/loader.h Mach-O data structure definitions in the XNU source]
* [https://blog.xpnsec.com/building-a-mach-o-memory-loader-part-1/ Building a Custom Mach-O Memory Loader for macOS (series)]
* [https://hexiosec.com/blog/macho-files/ So Macho - A look at Apple executable files]

[[Category:Executable Formats]]
````

<a id="w004209"></a>

## W004209 — User:Schizius

Bron-ID: W004209
Namespace: User (2)
Revisie: 21663 | Datum: 2017-10-31T05:52:59Z
Bron: https://wiki.osdev.org/User:Schizius
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASchizius&oldid=21663

````text
Professional firmware engineer, and amateur operating system developer.
````

<a id="w004210"></a>

## W004210 — Garbage Collection

Bron-ID: W004210
Namespace: Main (0)
Revisie: 21680 | Datum: 2017-11-12T19:31:04Z
Bron: https://wiki.osdev.org/Garbage_Collection
Revisie-URL: https://wiki.osdev.org/index.php?title=Garbage_Collection&oldid=21680
Doorverwijzing: Garbage collection

````text
#REDIRECT:[[Garbage collection]]
````

<a id="w004211"></a>

## W004211 — Program Memory Allocation Types

Bron-ID: W004211
Namespace: Main (0)
Revisie: 29514 | Datum: 2025-05-10T13:02:50Z
Bron: https://wiki.osdev.org/Program_Memory_Allocation_Types
Revisie-URL: https://wiki.osdev.org/index.php?title=Program_Memory_Allocation_Types&oldid=29514

````text
'''NB:''' ''The original version of this was based on a [http://forum.osdev.org/viewtopic.php?f=13&t=29829&start=45|forum post] discussing automatic memory management.''

By it's nature, a program's thread of execution require a certain amount of memory for storing both code and data during its operation. While some of this memory usage can be determined statically when a program is compiled or assembled, most programs also have some memory use which cannot be until the program is loaded into memory, and in many cases, there is some whose size can only be determined over the course of the program execution. 

There are four general models of memory usage in most programs, including operating systems: ''read-only memory'', ''static global memory'', ''lexically allocated memory'', and ''dynamically allocated memory''. All programs use the first two to some degree, the overwhelming majority of programs use the third (and most programming languages are designed to apply it as a matter of course), and the fourth is used in a significant proportion of programs, including any program in a language which implements [[Garbage collection]] as a primary language feature.

== Read-Only Memory ==
Read-only memory, as the name implies, is a region of memory in which constant values are stored; the program should never attempt to change the values of addresses in read-only memory. This includes the memory for the program code (often called the ''.text section''), as well as data constants (the ''.rodata section''). In some instances, a compiler may place constants representing program literals (e.g., quoted strings, hard-coded numeric values) in .rodata as well.

Most modern systems incorporate some form of hardware page or segment protection, to prevent read-only memory from be altered. Trying to change something in read-only memory on such systems will result in a memory protection fault of some kind. Management of read-only memory sections is by necessity a function of the operating system for both the system itself and for userspace applications.

== Global Memory ==
Global memory is used for storing variables which are in scope to the entire program and whose size is known when a process is launched (in most cases, it can be determined at . This is a fixed amount of space given to the program when it starts, and cannot be increased or decreased. 

Some global memory may be initialized when the program starts, using values that are part of the executable image, but there may be a separate area of global memory (usually referred to as the ''.bss section'') is simply an amount of memory that the loader clears when the program starts - the executable file just stores the size of the .bss section, to avoid having a large blob of useless empty memory in the file.

==Lexically-allocated memory==
Lexically-allocated memory, also called function local memory, is memory which is automatically allocated in a block when a function begins and deallocated when it returns. It can be seen as an area of 'scratch memory' which the program can use for temporary values whose lifespan is determined by the running time of the function which is using it, and can always be automatically reclaimed for reuse after the function exits.

In many cases, program is given a large, but fixed, block of memory for this purpose, whose size can generally be set when the program is launched; whether the program can request for additional memory for this block, and how, is a matter of OS design. While a process is in principle free to use this memory as it chooses, most operating systems and high-level languages impose standard conventions on the use to ensure the basic purpose - memory automatically allocated on function entry and automatically freed on function exit - is enforced, and many CPU instruction sets have hardware support for these conventions. 

The function's local memory is structured as an ''activation record'' or ''activation frame'', which contains the function's local variables (or references to them), as well as means of accessing the function's arguments and the address which the function should return to when it exits. Most of the time, the size of an activation record is fixed at compile time, but some kinds of local variables (most notably strings) can be dynamically sized for a given function invocation in some languages, so long as it doesn't change once the activation record is created. 

Since functions may call other functions, or even call themselves recursively, in general each call to a function has to be provided with a separate activation record. This ensures that a given function has it's own versions of each of it's variables; this means that recursive calls do not overwrite the caller's variables. The variables are thus managed in a 'Last In, First Out' (LIFO) fashion, a fact which is used by most systems in implementing the activation records, as will be described below in the section on ''stack allocation''.

The primary exception to this is when the last action a function performs is a call to another function, which is known as a ''tail call''. In this instance, the compiler can reduce the short-term memory load by means of ''Tail Call Optimization'', in which the existing activation record is reused for the function it is calling. Many compilers offer TCO as an optimization option, and some languages mandate it either as an default for all tail calls, or as something which can be specified through language pragmas.

=== Stack Allocation ===
Stack allocation is a widely used method for implementing lexically-scoped memory allocation. Because activation records are created and free in a 'LIFO' order, it is possible to push the activation records (or at least a reference to it) onto a stack data structure. This ''program stack'' has been near-ubiquitous for languages designed after the early 1960s, and many ISAs have either hardware support for the program stack, or a register convention that amounts to the same effect.

The general model for a register-based stack is to have a set of registers set aside as pointers that define both the total size of the stack, and it's current state. The mechanisms for sizing the ''.stack section'' vary, but the basic mechanism for manipulating them are more or less universal.

The stack's dynamic state is defined by a ''stack pointer'', a counter which points to the address on the stack where the most recently added element is found. When a new element is ''pushed'' onto the stack, the stack pointer is decremented (for a stack that grows downward - for a variety of reasons, most hardware stacks grow down by default, so the rest of this discussion assume this direction), and the data for the new top of the stack is copied to the location addressed by the stack pointer.

When an element is ''popped off'' of the stack, the stack pointer is incremented by the same amount, which has the effect of freeing the memory used for the previous top element.

To manage an activation record on a stack, a second register, the  ''base pointer'' (also called a ''frame pointer''), which points into the stack data, is used. 

When a function is to be called, the caller pushes the function's arguments onto the stack in an order defined by a ''[[Calling Convention]]'', which defines the order in which arguments and local variables are stored. Depending on the convention, saving the caller's own state (e.g., the registers it is actively using) may be the caller's responsibility, or that of the called function, or be some combination of the two (e.g., certain registers may be 'sacred' and have to be preserved by the called function, while others can be used freely by the function and it is up to the caller to save them before the call). 

The calling convention may also specify that some or all of the arguments be passed  in registers rather than on the stack, to reduce memory use and call overhead, which could affect what the caller needs to preserve for its own use. 

Once the arguments are pushed, the call to the function is made by pushing the address just past the current one (the one where the call is made from) as the ''return address'' for the function, and jumping to the ''entry point'' of the called function. Some ISAs have a special instruction for this purpose, which either pushed the value to the stack directly, or stores it into a return address register; for example, the '''CALL''' instruction on the Intel x86 instruction set. When a return-address register is used instead, the function would be responsible for saving it's own return address in its activation record explicitly, though this does allow a function that does no function calls of it's own to avoid the overhead of pushing the address.

When control is passed to the function, the function has to first save the current base pointer (the base pointer of the calling function). It then copies the current stack pointer to the base pointer, which can then be used as a base for offsets up into the arguments, or down into local variables. Finally, the stack is decremented by the sum total amount of memory needed for all of the function's local variables.

When a the function exits, the function copies it's base pointer back to the stack pointer, then pushes the caller's base to the base pointer, which has the effect of restoring the stack to the point of the function's entry. It then can return to the caller, often through a special instruction such '''RET''' on the x86 (which also removes the return address from the stack).


== Dynamically Allocated Memory ==
''Dynamic memory'', also called ''Heap Memory'', is memory used for data when neither the size nor the lifespan of the variable is known beforehand. This a region of memory that can be allocated to programs in pieces, at run time, without being automatically reclaimed when the function that allocated it ends (some languages have a special means of reclaiming dynamic memory after it is no longer in use called garbage collection, but that's not relevant here). Most operating systems have a system call to allow the program to request that the system allocate enough memory to hold one or more values of a given type; it returns a pointer to the newly allocated memory space. There usually is a corresponding system call which allows the program to return the allocated memory to the system. In between allocation and release, the memory becomes available for use, so long as there is at least one pointer to the memory block; you can pass the address of that memory around from one pointer to another, treating it as if it were the same as a pointer to any other non-constant memory. However, once the system releases that memory, the pointers to the memory are all invalid, and should not be used - preferably, you should null them after releasing the memory they point to. When the process exits, the operating system (should) release the memory used, even if it wasn't deallocated, but as long as the program is running, the memory will still be allocated until freed.

There are just three rules to follow with dynamic memory: don't lose the starting address of the memory block, or you won't be able to deallocate later; always remember to deallocate it after you are done with it; and don't try to access the memory after it has been deallocated. These rules are trickier to follow than they sound, however, as it is easy to lose track of where the memory is and how you are using it at any given time. Failing to release memory when it is finished is a memory leak, while a pointer that points to a freed block (or one that is invalid for some other reason) is a wild pointer.

So, if it is so complicated, why bother with dynamic memory? Because you can make arrays of arbitrary size without knowing how big it has to be beforehand. Also, you can use it to allocate structures with pointers in them, which can then point to other dynamically allocated memory, letting you chain the allocated memory into more complex data structures such as lists and trees. It simply wouldn't be practical to make these structures without dynamic allocation.

This brings us to application memory management. Most of the time, programs do not fetch a new block of memory each and every time they need one; the overhead of system calls is too high, and it potentially can slow the whole system down as the OS manages the allocations. So the usual practice is to allocate larger blocks of memory than is actually needed, and have use that as a kind of scratch space for allocating memory from. The allocations and frees are usually wrapped up in a pair of library functions corresponding to the system calls, and the program only calls the OS again if it needs more. The local allocator has to keep track of the local heap memory, but usually this isn't too complicated.

However, this leaves the programmer to keep track of the actual pointers to the memory, which can be very complicated indeed. For this reason, various approaches for automating memory management have been devised, with [[Garbage collection]] being the most common (though a related form called 'reference counting' is often used as well, and often confused with it). Basically, in garbage collection, the library functions require the program to get all pointer variables from the allocator, and a part of the program - the garbage collector - checks periodically to see if the blocks of allocated memory still have any pointers pointing at them. When a block has no pointers to it, the allocator frees it for the program, without any action on the part of the programmer. Older garbage collectors had to stop the program in order to do this, but some modern multithreaded ones can do so in parallel to the program's other threads.

While GC is most associated with interpreters, it is applicable to compiled code as well - it was originally developed for the original Lisp 1.5 compiler in 1961. It is possible to use a garbage collector as a library and requiring the programmer to follow the GC discipline, but it is far more common for it to be implemented in the language itself. It has a number of other advantages for both security and simplicity, especially in scripting languages - it allows the language translator to abstract away the allocation and deallocation of variable memory entirely - but there are distinct costs to it as well. There are endless variations on garbage collection meant to reduce the overhead of doing the collection, but a certain amount of overhead is inevitable when garbage collection is used.

[[Category:Memory management]]
````

<a id="w004212"></a>

## W004212 — Calling Convention

Bron-ID: W004212
Namespace: Main (0)
Revisie: 21683 | Datum: 2017-11-12T21:20:39Z
Bron: https://wiki.osdev.org/Calling_Convention
Revisie-URL: https://wiki.osdev.org/index.php?title=Calling_Convention&oldid=21683
Doorverwijzing: Calling Conventions

````text
#REDIRECT:[[Calling Conventions]]
````

<a id="w004213"></a>

## W004213 — Executable Format

Bron-ID: W004213
Namespace: Main (0)
Revisie: 21694 | Datum: 2017-11-19T17:55:39Z
Bron: https://wiki.osdev.org/Executable_Format
Revisie-URL: https://wiki.osdev.org/index.php?title=Executable_Format&oldid=21694
Doorverwijzing: Executable Formats

````text
#REDIRECT:[[Executable Formats]]
````

<a id="w004214"></a>

## W004214 — User:Schol-r-lea/GRUB Legacy

Bron-ID: W004214
Namespace: User (2)
Revisie: 21772 | Datum: 2017-11-26T21:02:31Z
Bron: https://wiki.osdev.org/User:Schol-r-lea/GRUB_Legacy
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASchol-r-lea%2FGRUB_Legacy&oldid=21772

````text
GRUB stands for GRand Unified Bootloader and is a [[bootloader]] and the reference implementation of the [[Multiboot|Multiboot Specification]]. GRUB can boot any Multiboot-compliant OS (e.g. Hurd, [http://grub.enbug.org/MultibootSystems among others]), however it can also boot some non-Multiboot-compliant OSes, e.g. Windows, via a chain-loading function, and some others via special support, e.g. Linux.  mbchk can be used to determine if a file is Multiboot-compliant.

This article concentrates on GRUB Legacy. Information regarding modern [[User:Schol-r-lea/GRUB]], a full rewrite, can be found in a separate article.

==Features==
GRUB takes away all the complexities out of trying to boot your OS by doing it for you. GRUB can handle a variety of filesystems: from MS-DOS FAT systems over Linux ext2fs (Second Extended File System) to BSD FFS, and can load kernels in various binary formats, including "generic" [[ELF]], FreeBSD a.out, flat binary, and "generic" executables (any file with a valid Multiboot header).

One major advantage of GRUB (or more precisely, a Multiboot-compliant bootloader) is that the kernel will be entered in a known state, which includes the [[A20 Line]] having been enabled, and [[Protected mode]] having been entered. This takes a lot of the pain out of writing a kernel, rendering GRUB a very useful tool for the amateur, or anyone who wants to spend more time on the intricacies of the kernel rather than worrying about these generic start-up procedures.

Consult the [http://www.gnu.org/software/grub/manual/multiboot/ Multiboot Specification] if you want to know what GRUB expects and what you can expect from it.

==Installation and booting==
This section describes how to install GRUB on your <em>actual computer</em>, if you are using some kind of emulator (e.g. [[Bochs]]) to test your OS in you're probably more interested in the setup described in e.g. [[Bare bones#Booting the kernel|Booting the kernel]] or [http://www.jamesmolloy.co.uk/tutorial_html/1.-Environment%20setup.html Environment setup]. However, you might still be interested in [[#Load your kernel over network (TFTP)|how to load your kernel over TFTP]].

===Installing to floppy===
This tutorial will show you how to install GRUB and your self-made kernel on a floppy disk, to avoid screwing up your hard drive. Installing GRUB as your system bootloader depends heavily on your system setup and is beyond the scope of this tutorial.

GRUB cannot be installed directly to the intended boot device. Once an operating system is up and running, GRUB's probing the BIOS for available drives might yield wrong results. Thus, a four-step process is used (using two floppy disks: the boot disk to-be, and an auxiliary disk).

NOTE: creating your own GRUB bootdisk requires a Unix-like environment. If you're working under Windows (or don't feel like setting up a GRUB disk with Linux), check out our [[Disk Images]] page to get a ready-to-use image.

# Prepare the boot disk;
# create a rudimentary GRUB installation on an auxiliary disk;
# boot from the auxiliary disk, and complete the GRUB installation on the boot disk.

You will find the [ftp://alpha.gnu.org/gnu/grub/ source as well as precompiled binaries] at the [http://www.gnu.org/software/grub GRUB web page]. (Windows users will want the <code>grub-0.9x-i386-pc.tar.gz</code> package.) Download and unpack the archive.

=====Preparing the boot disk=====
Format a floppy disk; this will be your boot disk. Create a directory <code>boot</code> on that disk, and copy the files <code>stage1</code> and <code>stage2</code> from the tarball (or your build directory) to <code>boot</code>. Note that, after the installation, <code>stage2</code> must not be relocated (e.g. by Windows' disk defragmentation), or the bootloader will be unable to find it.

=====Auxiliary GRUB install=====
Remove the boot disk from the floppy drive and insert the auxiliary disk. The files <code>stage1</code> and <code>stage2</code> from the GRUB distribution must be written to this disk, to sector 0 and sector 1+ respectively. In your shell (for Windows users, this means the [[Cygwin]] shell), type:

<source lang="bash">
dd if=<stage1> of=/dev/fd0 bs=512 count=1
dd if=<stage2> of=/dev/fd0 bs=512 seek=1
</source>

Now GRUB will be able to boot stage1 and stage2 from this disk. We cannot use it as our boot disk, though, since writing stage2 to sector 1 destroyed the floppy's format structure (you can't write files to the disk anymore, and any previously written <code>boot</code> directory could no longer be found). That's why we will now set up the other disk to boot GRUB and have a valid structure.

Leave the auxiliary disk in the drive, and boot from that floppy.

=====Completing the install=====
GRUB will boot into its command shell. (If your system does boot from hard drive instead, check your BIOS boot drive settings.) When the <code>grub ></code> prompt appears, remove the auxiliary disk from the drive and replace the boot disk. From the GRUB shell, you can now complete the installation on the boot disk:

<source lang="bash">
install (fd0)/boot/stage1 (fd0) (fd0)/boot/stage2 (fd0)/boot/menu.cfg
</source>

The parameters tell GRUB:

* where to find the stage1 file;
* which device to install GRUB on (the floppy drive) - this writes stage1 to the boot sector, you can delete <code>boot/stage1</code> afterwards;
* where to find the stage2 file (hardcoding the sector number of that file into the boot sector stage1; that's why <code>stage2</code> must not be moved after this procedure);
* where stage2 should look for the configuration file.

Remove the boot disk from the drive, and reboot your operating system so you can edit <code>/boot/menu.cfg</code>.

=====Configuring GRUB=====
When you boot from the just-created boot floppy, GRUB stage1 is read from the floppy's boot sector. This stage1 "knows" where to find stage2 (it was told so during installation), and loads that. The stage2 does lots of interesting stuff (we'll come to that), but most importantly, it "knows" how to read file systems, so it can find the configuration file, <code>boot/menu.cfg</code>.

The configuration file tells GRUB where to look for the actual kernel image(s). You can set it up so it offers you a selection of various different kernels, even allowing you to boot your "real" OS from hard drive after booting GRUB from floppy. (In the end, you will probably want to [[#Installing to hard disk|install both GRUB and your own OS kernel on the hard drive]], but we'll stick with the floppy solution for now.)

Add an entry like this to the configuration file:

<pre>
title   MyOS
root    (fd0)
kernel  /boot/kernel.bin
</pre>

Now all we have to do is to compile a kernel image and save it as <code>/boot/kernel.bin</code>. GRUB will find, load, and start it. For more details, menu options etc., refer to the GRUB manuals.

===Installing to USB===
'''NOTE''': I have tried many tools to install GRUB onto a USB device under windows, however, I have not been able to figure out how. So, to install, either use a Linux distro on your PC of use a Linux Live CD.

Putting GRUB onto a USB device is as easy as following the above example of installing to a standard hard disk. There is one extra step involved, finding out under which mount point your USB device is located. You may even have to mount it yourself if it is not already mounted. To install use the following steps:

If you need to mount your USB device: (Exchange <code>sdb1</code> with your device name.)

<source lang="bash">
mkdir /mnt/sdb1
mount /dev/sdb1 /mnt/sdb1
</source>

Create required directories:

<source lang="bash">
mkdir /mnt/sdb1/boot
mkdir /mnt/sdb1/boot/grub
</source>

Locate the <code>stage1</code>, <code>stage2</code>, and <code>*_stage1_5</code> files:

<source lang="bash">
find / -name stage1
</source>

Copy stage1 and stage2 files, <code>/usr/lib/grub/</code> is an example location, use the location returned by the find command:

<source lang="bash">
cp /usr/lib/grub/stage1 /mnt/sdb1/boot/grub/stage1
cp /usr/lib/grub/stage2 /mnt/sdb1/boot/grub/stage2
</source>

For FAT formatted USB device use:

<source lang="bash">
cp /usr/lib/grub/fat_stage1_5 /mnt/sdb1/boot/grub/fat_stage1_5
</source>

For ext2 formatted USB device use:

<source lang="bash">
cp /usr/lib/grub/e2fs_stage_1_5 /mnt/sdb1/boot/grub/e2fs_stage_1_5
</source>

Then execute the GRUB install application:

<source lang="bash">
sudo grub
</source>

Enter the following commands:

<source lang="bash">
root (hd1,1)
setup (hd1)
quit
</source>
if you don't know where your kernel is, you can use the command "find /boot/grub/e2fs_stage_1_5" to get the parameters which would be used in the command root (sdb1,1) and setup (sdb1).

===Installing to hard disk===
See [http://www.gnu.org/software/grub/manual/html_node/Installation.html Installation] in the official GRUB manual.

===Installing to CD===
See [[Bootable El-Torito CD with GRUB Legacy]].

==Load your kernel over network (TFTP)==
You can also use GRUB's builtin network support to increase your build/test turn around time. You can boot another machine with a kernel directly from your build directory without having to copy it over manually. See also, [http://www.gnu.org/software/grub/manual/html_node/General-usage-of-network-support.html#General-usage-of-network-support How to set up your network] in the official GRUB manual.

=====Getting GRUB to support your card=====
First you need a version of GRUB that supports your network card. Most likely you will have to build one from source, so [ftp://alpha.gnu.org/gnu/grub/ grab it] and extract it to a work directly.

Run

<source lang="bash">
./configure --help
</source>

to see a list of options. In my case, I use Netgear tulip cards, so I execute:

<source lang="bash">
./configure --enable-tulip
make
make install
</source>

You may want to consider using the <code>--prefix</code> option if you don't want to overwrite your existing GRUB install or do not have root access.

NOTE: this was done on a Linux box, I assume using Cygwin under Windows would work as well.

=====Set up a TFTP server=====
You will need a TFTP server running on your build machine and pointing to your build directory. I've been using the [http://support.3com.com/software/utilities_for_windows_32_bit.htm 3CServer utility from 3COM], but there are several out there for most platforms.

Make sure any firewalls you have installed allow incoming connections on the TFTP ports.

=====Install as you would normally=====
On your target machine install the GRUB binary you just built as documented above.

=====Configure GRUB=====
I use the following entry in my <code>menu.lst</code> to boot my kernel.

<pre>
title Network Boot
# Obtain an IP address for this card
dhcp
# I don't have DHCP setup to deliver the TFTP server's IP address,
# so I set it manually this is my build machine
tftpserver 192.168.0.50
# The root device is the network
root (nd)
# My kernel is directly under the TFTP directory
kernel /kernel.exe
# I also load a module for testing out multitasking
module /module.exe
</pre>

And that should be it. If you're not running DHCP you can use <code>ifconfig</code> to manually set addresses, gateways, etc. Your can also use the BOOTP or RARP protocol.

==Other loaders that are Multiboot compliant==
Chris Giese's [http://my.execpc.com/~geezer/temp/mbload.zip MBload] also works with Multiboot-compliant kernels. Check out the [[Topic:8549|related thread]] for more info.

==See Also==
===Wiki Pages===
*[[Grub Error 13]]
*[[Multiboot]]

===Tutorials===
*[[Bare bones]]

===Threads===
*[[Topic:9548|Switching back to real mode from GRUB]]
*[[Topic:9586|Setting boot-time graphic video mode with GRUB]]
*[[Topic:18255|GRUB VBE Patch]]

===External Links===
*[http://tldp.org/HOWTO/Linux+Win9x+Grub-HOWTO/index.html Linux+Win9x+Grub HOWTO]
*[http://www.pixelbeat.org/docs/disk/ GRUB PC Disk details]
*[http://www.gnu.org/software/grub/manual/legacy/grub.html GRUB Legacy manual]
*[https://gna.org/projects/grub4dos/ GRUB for DOS]

[[Category:Bootloaders]]
[[de:Grand Unified Bootloader]]
````

<a id="w004215"></a>

## W004215 — User:Schol-r-lea/GRUB

Bron-ID: W004215
Namespace: User (2)
Revisie: 21771 | Datum: 2017-11-26T21:00:33Z
Bron: https://wiki.osdev.org/User:Schol-r-lea/GRUB
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASchol-r-lea%2FGRUB&oldid=21771

````text
'''GRUB''' (GRand Unified Bootloader) and is the GNU Project's primary bootloader and the reference implementation of the [[Multiboot]] 2 Specification. GRUB can boot any Multiboot-compliant OS (e.g. Hurd, among others), however it can also boot some non-Multiboot-compliant OSes, e.g. Windows, via a chain-loading function, and some others via special support, e.g. Linux. mbchk can be used to determine if a file is Multiboot-compliant. s the GNU project's bootloader. 

The current version 2 series have a more complete feature set than [[User:Schol-r-lea/GRUB Legacy|GRUB 0.97]] (commonly referred to as "GRUB Legacy"), and is now the recommended version. However, GRUB Legacy is somewhat easier to work with, and some pages in the wiki still refer to the earlier version, so the information for it has been retained in a separate page.

== History ==
GRUB was initially developed by Erich Boleyn as part of work on booting the operating system GNU/Hurd, developed by the Free Software Foundation. In 1999, Gordon Matzigkeit and Yoshinori K. Okuji made GRUB an official software package of the GNU Project and opened the development process to the public

GRUB version 2 started its life as the [http://www.nongnu.org/pupa/ PUPA] (note the pun) research project and was rewritten from the ground up. In 2002, it was integrated into the development branch after GRUB 1.97. 

Due to initial stability problems and confusion brought by the move from Multiboot 1 to Multiboot 2, adoption of GRUB 2 by major Linux and FreeBSD distros was slow prior to 2008. GRUB 0.97 remained in wide deployment until 2010, by which time GRUB 2 had improved to the point that it had become the ''de facto'' standard for most non-commercial operating systems. Use of GRUB Legacy lingered with hobby OSes out of ease of use, but by 2013, GRUB 2 was the recommended boot loader for most new OS projects.

== Features ==

GRUB takes away all the complexities out of trying to boot your OS by doing it for you. GRUB can handle a variety of [[File Systems]], including Microsoft [[NTFS]], [[FAT32]], and [[VFAT]], Linux [[Ext2]]fs/[[Ext3]]fs/[[Ext4]]fs, [[ReiserFS]], and BSD FFS. It supports loading kernels in a variety of binary [[Executable Formats]], including [[ELF]][32|64], and Unix [[a.out]], as well as a raw binary image; it should be able to load any kernel image file in those formats which has a valid [[Multiboot]] header. It can also 'chain-load' non-Multiboot systems such as Microsoft Windows by loading their native boot loader as a raw binary image.

One major advantage of GRUB (or more precisely, a Multiboot-compliant bootloader) is that the kernel will be entered in a known state, which on the x86, includes having the system already in [[Protected mode]] when the OS is launched. This takes a lot of the pain out of writing a kernel, rendering GRUB a very useful tool for the amateur, or anyone who wants to spend more time on the intricacies of the kernel rather than worrying about these generic start-up procedures. 

Other features available in GRUB 2 include:
* Basic scripting support
* GUI (bootsplash support, custom colors, custom themes, ...)
* Memory management
* Internationalization support
* Rescue mode

== Multiboot ==

Some versions of GRUB 2 like to put Multiboot modules in relatively high physical memory addresses, in contrast to GRUB-legacy which loaded them into low memory.  Be careful when making your kernel work with GRUB 2 that it is not making any assumptions about where the Multiboot modules will appear.

When your kernel gets control, the machine state is defined as follows: [http://www.gnu.org/software/grub/manual/multiboot/html_node/Machine-state.html Multiboot machine state]. Your code should have minimal dependency on this initial state; for example, define your own GDT instead of relying on the GDT setup by GRUB.

=== Header ===
As the GRUB 2 manual puts it:

{{quote|The primary requirement for GRUB is that it be compliant with the ''Multiboot Specification''.}}

But the Multiboot header as used by older versions of GRUB 2 (field is present in GRUB 1.99 and newer) did ''not'' include the ''header_length'' field that is specified in the [http://nongnu.askapache.com/grub/phcoder/multiboot.pdf Multiboot 1.6 specification].

GRUB 2 also supports the old [https://www.gnu.org/software/grub/manual/multiboot/multiboot.html Multiboot 0.6.96 specification]. It is possible to include both headers.

== Creating GRUB 2 Disk Images ==

These instructions cover installing GRUB to a [[Disk Image]] under various development hosts. 

=== Linux ===


==== BIOS hardware ====

==== UEFI Hardware ====
=====Compiling GRUB=====
Older GRUB versions are riddled with nasty bugs. As this probably includes the version from your package management, you should be compiling GRUB from source. As we're compiling for UEFI, you should pass the appropriate flags to configure. An example invocation might look something like this:
<source lang="text">
../grub-2.02~rc2/configure --prefix="$HOME/opt/grub" --target=x86_64 --with-platform=efi
</source>

After completing the build, GRUB refused to do anything as it was missing a font file. To fix this, run
<source lang="text">
bin/grub-mkfont -o share/grub/unicode.pf2 /usr/share/fonts/truetype/unifont/unifont.ttf
</source>

GRUB might warn you about share/locale/ missing. To solution is to create the missing directory.

=====Building a GRUB UEFI binary (BOOTX64.EFI)=====
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


=== Windows (Using Cygwin or MinGW) ===
Because GRUB is designed as a part of the GNU project, it assumes the use of GCC and the Binutils tool chain. While this does not place an restrictions on the toolchain used by the OS, it does mean that compiling GRUB itself will require a [[GCC Cross-Compiler]] and [[Binutils]] configured as an [[OS Specific Toolchain]]. 

Most of the instructions for building GRUB under Linux apply, but there are some notable difference.

=== MacOS X ===
It might be useful to create an image file for a HDD; The following instructions help you create a HDD with an MBR partition map.
Based on information from <palk> on #osdev, it is slightly more complicated to create a HDD image than you'd expect.

This information might not be applicable to Linux users, who will most probably want to use a loopback device. This is for developers on OS X, which doesn't
have a loopback device and has a finicky image mounter.

1. First, create a blank, raw image with DD, with the required size. Here, I'll make a 80MB image -- 163840 sectors of 512 bytes.
<source lang="bash">
dd if=/dev/zero of=disk.img count=163840 bs=512
</source>

2. Next, calculate the CHS values for the disk and have them at hand. TODO: Explain this.
    Here, it's a 80MB disk, so the CHS values are 78, 32 and 63 respectively.

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

==== Additional useful options ====
Whatever device you are using, you may want to have a PC partition table and create a partition you format in one of the filesystems supported by GRUB. If you do, be sure to add the following option the grub-install arguments:
 --modules="part_msdos"

In general, if GRUB 2 happens to fail to do what you want and you suspect that it needs some missing functionality, just try to add a module name you believe has the functionality you need to the --modules argument. The module files generally are in /boot/grub/i386-pc/.


====Blessing the binary on macOS====
Macs require bootable binaries to be 'blessed' by a utility:
<source lang="text">
bless --verbose --folder=/Volumes/EFI --file=/Volumes/EFI/EFI/BOOT/BOOTX64.EFI --setBoot
</source>


== See Also ==
=== Articles ===

* [[GRUB|GRUB Legacy]]
* [[Bootable El-Torito CD with GRUB Legacy]]
* [[Multiboot]]

=== External Links ===

* [http://www.gnu.org/software/grub/ GRUB 2 Homepage]
* [http://grub.enbug.org/ GRUB 2 Wiki]

[[Category:Bootloaders]]
[[de:Grand_Unified_Bootloader]]
````

<a id="w004216"></a>

## W004216 — User:Schol-r-lea/Upgrading GRUB Legacy to GRUB 2

Bron-ID: W004216
Namespace: User (2)
Revisie: 21711 | Datum: 2017-11-19T21:11:41Z
Bron: https://wiki.osdev.org/User:Schol-r-lea/Upgrading_GRUB_Legacy_to_GRUB_2
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASchol-r-lea%2FUpgrading_GRUB_Legacy_to_GRUB_2&oldid=21711

````text
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
   multiboot /boot/kernel-file   # The multiboot command replaces the kernel command
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
````

<a id="w004217"></a>

## W004217 — User:Schol-r-lea/Brief Tutorial On Data Structures In Assembly Language

Bron-ID: W004217
Namespace: User (2)
Revisie: 22221 | Datum: 2018-03-16T22:44:04Z
Bron: https://wiki.osdev.org/User:Schol-r-lea/Brief_Tutorial_On_Data_Structures_In_Assembly_Language
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASchol-r-lea%2FBrief_Tutorial_On_Data_Structures_In_Assembly_Language&oldid=22221

````text
'''''N.B.: this was originally created as a [http://forums.devshed.com/programming-languages-139/structures-assembly-language-408817.html post on the DevShed forum] in December 2006. I am posting it here for now in order to edit it for a more general use later.'''''

This tutorial gives a general overview of how data structures can be implemented in various assembly languages, with examples for a number of commonly used assemblers, with special attention being given to the Intel and AT&T (GAS) syntaces for x86 protected mode. The examples are all for 32-bit systems, with a matching C `int` type; for other system bit widths, the sizes will need to be adjusted as appropriate.


= General Concepts =
Conceptually at least, a structure in assembly language is like a structure in any other language: a fixed group of heterogeneous variables which have their own names but which are part of a larger composite variable. For example, in C++, a struct might look like

    struct {
       int foo, bar;
       char baz;
    } qux;

The variables inside the braces would all be part of the composite variable `qux`; to access one part of `qux`, say `foo`, you would use `quux.foo` or something similar. These are what are known as the ''fields'' or ''members'' of the structure.

It's much the same in assembly language, except of course that the types of the variables are implicit rather than explicit. Thus, it is up to the programmers using the data structures to ensure that the field sizes are correct, and the instructions operating on them are appropriate to the data they represent.

Most assemblers have support for defining structures explicitly, via directives, as will be explained below. These directives allow the assembly programmer to define how a structure is laid out in memory, and in most case can also define a structure 'type' as a series of memory offsets from the first field of the structure.

It is possible to define structures even if the assembler has no such directives, however. In that case, the structure definition can be created by a set of programming conventions; the programmer would be responsible for deciding what those conventions are. So, if we agree to use labels which start with the name of the structure as a whole, followed by a period, and then the name of the element in question, you would end up with a 'structure' that consists of a series of explicit labels and/or offsets.

Most of this discussion will focus on the convention-based approach, both to demonstrate the principles more explicitly, and to avoid tying the discussion too tightly to the syntaces of any specific assemblers. This will be followed by brief explanations of the structure directives in some of the more popular assemblers, and how they can be used to the same effect, and how they can improve the clarity of the structure definitions.

while it is preferable to use the directives appropriate to the assembler when possible, in general the result will be the same in terms of how the structure gets laid out in memory. 

= Structures by Convention =
In order to manage a structure in the absence of assembler-supported structure directives, we need to first decide how we want to describe our structures. One simple approach, which most assemblers will permit, is to define the fields - or more generally, the offsets for the fields - by using the name of the structure or structure type, a period, and the name of the field. This is similar to the way fields are accessed in many high-level languages, and most assemblers with directive-based structures use this approach or one very similar to it. 


== Layout in Memory of a Unique Structure ==

Defining the fields of a structure becomes simply a matter of defining the location in memory of the structure itself, then defining the locations of the fields.

{|
|+
|-
! MASM syntax
! NASM syntax
! GAS Syntax (any)
|-
|
    qux:              
    ; blind label marking 
    ; the start of the
    ; struct, the same actual 
    ; addr. as qux.foo in
    ; this case.
    
    ; the struct's fields:
    qux.foo: DWORD 1 ; int
    qux.bar: DWORD 1 ; int
    qux.baz: BYTE  1 ; char 
    align 4
|
    qux:              
    ; blind label marking 
    ; the start of the
    ; struct, the same actual 
    ; addr. as qux.foo in
    ; this case.
    
    ; the struct's fields:
    qux.foo: resd 1 # int
    qux.bar: resd 1 # int
    qux.baz: resb 1 # char 
    align 4
|
    qux:              
    # blind label marking 
    # the start of the
    # struct, the same actual 
    # addr. as qux.foo in
    # this case.
    
    # the struct's fields:
    qux.foo: .space 4 # int
    qux.bar: .space 4 # int
    qux.baz: .space 1 # char 
    .align 4
|-
|}


Note that because `baz` was a one byte value, we need to pad three bytes after it to ensure word alignment. The structure itself is 9 bytes long, but the whole memory use is 12 byte when you count the padding. This will be important later. 


=== Accessing a Uniquely-Defined Structure's Fields ===

Since in this model, the fields are just ordinary labels which happen to follow a naming convention, they would be used in the same manner as any other globally defined memory variables.


== Defining a Structure Type ==
Now, you might say that this is a lot of pointless work for a single structure; while you can treat the  composite variable as a single structure, you really have three separate values, with separate labels. In this case, it is true that they could have been just as easily treated as separate data values; however, this is equally true in the C++ example above. However, declaring a single struct in this manner is useful for showing the logical and conceptual connections  between the elements; it shows that they are represent something as a whole.

That having been said, the real power in the idea of data structures is not from declaring a single structure in this manner, but from declaring ''types'' of structures. Again, in C++, you could define a structure type: 

    struct quux {
       int foo, bar;
       char baz;
    };
    
    quux flarp, *frink;

Now, instead of having a single struct named quux, we define a sort of pattern or template (not a template in the C++ sense, but bear with me) for defining structures of the same type. In assembly language, this sort of structure type would be defined not as a series of labels, but as a series of ''offsets'', like so:

{|
|+
|-
! x86, Intel syntax
! GAS Syntax (any)
|-
|
    ; size of quux structs, incl. padding
    quux.sizeof equ 12  
    quux.foo    equ 0   ; 1st element size
    quux.bar    equ 4   ; 2nd element size
    quux.baz    equ 8   ; 3rd element size
|
    # size of quux structs, incl. padding
    quux.sizeof .= 12  
    quux.foo    .= 0   # 1st element size
    quux.bar    .= 4   # 2nd element size
    quux.baz    .= 8   # 3rd element size
|-
|}

You would then define a variable the size of the whole structure type to hold the structure:

{|
|+
|-
! MASM syntax
! NASM syntax
! GAS Syntax (any)
|-
|
    flarp  BYTE quux.sizeof dup(?)
|
    flarp  resb quux.sizeof

|
    flarp  .space quux.sizeof
|-
|}


Note that a pointer to a struct would still be the regular size of a pointer, which for most 32-bit architectures is 4 bytes:

{|
|+
|-
! MASM
! NASM
|-
|
    SECTION .DATA
    frink    dd 0
    
    SECTION .CODE
    ; ... do things...
        lea eax, flarp
        mov frink, eax
|
    section [data]
    frink    dd 0
    
    section [text]
    ; ... do things...
        lea eax, flarp
        mov frink, eax
|-
|}


{|
|+
|-
! GAS, x86
! GAS, ARM
! GAS. MIPS
|-
|
    .section .data
    frink .long 0
    
    .section .text
    # ... do things...
        lea flarp, %eax
        movl %eax, frink
|
    .section .data
    frink .word 0
    
    .section .text
    @ ...do things...
        ldr %r0, =frink
        ldr %r1, =flarp
        str %r0, [%r1]
|
    .section .data
    frink .word 0
    
    .section .text
    # ...do things...
        la %t0, flarp
        la %t1, frink
        sw %t0, 0(%t1) 
|-
|}



There may be variations on this, such as using data pools in ARM assembly or different forms of indirection on the x86, but this is the general model.

=== Using an Instance of a Structure Type === 
At this point, the structure variables that make up flarp are implicitly defined by the offsets rather than defined by explicit labels. To use the struct you would get a pointer to the start of the struct variable, then use that pointer plus the defined offsets to access the structure elements.


{|
|+
|-
! Intel
! GAS, x86 
|-
|
        mov ebx, flarp
        mov eax, [ebx+quux.baz]
|
        movl flarp, %ebx
        movl (%ebx, quux.baz), %eax  
|-
|}

{|
|+
|-
! ARM, GAS syntax
! MIPS, GAS syntax
|-
|
        ldr %r0, 
|
        lw %t0, flarp
        lb %t1, quux.baz(%t0)
|-
|}



== Dynamically Allocated Structures ==

Now we're ready to see the real advantage of structures: defining values at runtime, either on the stack or in the heap. To dynamically allocate a structure of a given type, you just allocate the space need for it, and use the same offset-based approach to accessing the fields as before.

=== Stack Allocation ===

Most languages allocate the local variables for functions on the system stack, allowing them to be created and freed in a simple, consistent manner. To define a structure on the stack, you would do the same as you would a scalar variable, which is to allocate memory for it as part of the function's ''stack frame''.

{|
|+
|-
! ARM, GAS syntax
! MIPS, GAS syntax
|-
|
    
|
        addi $sp, $sp, -20
        sw   $fp, $fp.fp($sp)
        move $fp, $sp
        sw   $ra, fp.ra($fp)
        addi $t0, $fp, -20   # location of the struct on the stack
        la   $t1, frink
        sw   $t0, ($t1)      # frink now points to the structure
    
|-
|}

Note that the 12 bytes has to be in addition to anything else you are saving on the stack, such as the frame pointer, the return address, the s-registers, and so on; the struct (and any other local variables) would come after everything else being saved. This is also why we padded the struct to word alignment rather than halfword alignment - in the MIPS processor, values on the stack are always assigned in word increments (the same applies to heap structures).

Just as in C, you have to be careful with pointers to values on the stack; once the stack is cleared, the variable is gone, and trying to use a pointer to it after that may severely corrupt your stack. This can be a tricky problem, as it will only become apparent if the stack grows up to that point in memory again; it may not show up for several function calls later.

=== Heap Allocation ===

As with stack allocation, heap allocation works in much the same way as it does for scalar values; it simple is a matter of allocating a large enough block of memory to store the structure.

Since heap allocation is dependent on the operating system being used, any examples given would reflect the OS used. For this reason, no example here can apply to a new OS. For this reason, the examples will cover the use for four established systems (Windows, MacOS, FreeBSD, and Linux) in order to contrast any differences in how they need to be handled. There is also [[Chrestomathy: User-Space Memory Allocation|separate page]] where OS developers can explain their own system's memory handling methods, to highlight the different methods which new OS developers might consider using (this page will include examples for the established systems as well).

{|
|+
|-
! MASM
! NASM, Windows
! GAS, x86 Windows
|-
|
    
|
    
|
    
|-
|}

{|
|+
|-
! NASM, x86 MacOS
|-
|
    
|-
|}

{|
|+
|-
! NASM, x86 Linux
! GAS, x86 Linux
|-
|
    
|
    
|-
|}


{|
|+
! GAS, ARM Linux
! GAS, MIPS Linux
|-
|
    
|
        li   $a0, quux.sizeof
        li   $v0, malloc      # malloc == syscall 9
        syscall
        sw   $v0, frink       # save the pointer to the allocated space
|-
|}

As always the case for heap allocation, you have to be careful to retain at least one pointer to the structure as a whole, in order to free it later, and you should not try to access a structure pointer after the structure has been freed.

==== Using Heap Structures for Linked Data Structures ====
With heap variables, you can also create more elaborate data structures using structs and pointers, such as a linked list or tree. For example, let's consider a binary tree of lookup keys and values, in which each node has a 15-character string (16 bytes total to allow for a zero-delimiter for a maximum size key), a 50-byte data buffer (note the unaligned size!), and two 32-bit pointers for left and right branches:

    struct KeyNode {
        char key[16];
        uint8_t data[50];
        KeyNode *left, *right; 
    };


the general data structure would be:


{|
|+
|-
! MASM / NASM
! GAS (any)
|-
|
    KeyNode.sizeof equ 132
    KeyNode.key    equ 0
    KeyNode.data   equ 16
    align 4
    KeyNode.left   equ 68
    KeyNode.right  equ 100
|
    KeyNode.sizeof = 132
    KeyNode.key    = 0
    KeyNode.data   = 16
    .align 4
    KeyNode.left   = 68
    KeyNode.right  = 100
|-
|}

The code to manage them might be:


{|
|+
|-
! MASM
! NASM, Windows
! GAS, x86 Windows
|-
|
    
|
    
|
    
|-
|}

{|
|+
|-
! NASM, x86 MacOS
|-
|
    
|-
|}

{|
|+
|-
! NASM, x86 Linux
! GAS, x86 Linux
|-
|
    
|
    
|-
|}

{|
|+
|-
! ARM, GAS syntax
! MIPS, GAS syntax
|-
|
        
|
    # create a node  
        li   $a0, KeyNode.sizeof
        li   $v0, malloc
        syscall
        move $a0, $v0
        jal  insert
    
    # ...
    
    insert:
    
|-
|}

= Structures Using Assembler Directives = 
Up to now, this discussion has focused on manually defining structures and their fields using the same methods used in defining scalar values, by allotting space based on the size of the structure and using hand-computed offsets for defining and accessing their fields. However, just as it makes sense for the assembler to compute the addresses and offsets for labels and jumps or references to them, there would be a clear advantage for the programmer if the assembler allowed the structures field addresses or offsets be automatically calculated. Most modern assemblers do indeed have some way of doing this, though the specifics often vary greatly. 

Let's take the same C++ structure type as before, and apply the different assemblers' techniques for them.


== Defining and Using a Structure Type ==
    struct quux {
       int foo, bar;
       char baz;
    } flarp, *frink;



{|
|+
|-
! MASM syntax
! NASM syntax
! FASM Syntax
|-
|
    quux STRUCT ALIGN DWORD
        foo dd  ?
        bar dd  ?
        baz db  ?
    quux ENDS
    
    
|
    struc quux:              
        .foo: resd 1
        .bar: resd 1
        .baz: resb 1
        alignb 4
    endstruc
    
    flarp: 
      istruc quux
        at .foo, dd 0
        at .bar, dd 0
        at .baz, db '\00'
      iend

    frink: dd 0 ; null-init'ed ptr

|
    struc quux  foo, bar, baz 
    {              
        .foo dd foo 
        .bar dd bar
        .baz db baz
    }
      
    frink quux 0, 0, '\00'
    flarp dd 0  ; null-init'ed ptr
|-
|}

{|
|+
|-
! GAS Syntax (any)
|-
|
    .struct quux
        foo: .space 4
        bar: .space 4
        baz: .space 1 
    .end_struct
    
|-
|}

== Declaring Structures ==
Now, it is worth recalling that this is just a way of imposing a structured perspective on ''untyped'' memory. Most assemblers do not enforce data types, for scalars or structures, and it is up to the programmer to ensure that they are using the data in the intended manner, regardless of whether it is in a structure or not.



= Forth Assemblers, High Level Assemblers and Macro-Extensible Assemblers =
Most assemblers strive to maintain a close parallel between the assembly mnemonics and data declarations, and the resulting machine code that is actually executed. While most assembly languages do not have a strict one-to-one correspondence between the mnemonics and the opcodes, they do try to keep it as close to them as is feasible; ands while they will automatically calculate things such as label addresses, constant arithmetic in immediate arguments and indexes, and sizes for data, how those computed values are used is still up to the coder. In general, those abstraction mechanisms which assemblers do provide - structs and macros being the main examples - they rarely allow them to change the behavior or syntax of the language.

There are a number of exceptions to this, however. These fall into three main categories:

* '''Forth Assemblers''' - these are assemblers designed to work with Forth interpreters and compilers, and are used for creating executable Forth words which are automatically added to the current Forth dictionary. They essentially are used to extend the language to add support for hardware-dependent low-level operations, and are entirely integrated into the Forth system they are a part of. Generally speaking, these will operate on data structures defined in Forth, rather than having a separate structure mechanism and syntax.
* '''High-Level Assemblers''' - these are an intermediate between a conventional assembler and a high-level language. Most of the examples in this category use standard assembly mnemonics for the majority of the code but providing some higher-level operations such as loops and function returns to simplify the parts which are relatively common and reduce some of the repetitiveness of assembly code. Many of these, though not all, are aimed at making learning assembly language easier for novices. Several of them are actually ''called'' "High Level Assembly" or "HLA", including the best know one for x86 machines, [http://plantation-productions.com/Webster/|Randall Hyde's HLA], which is associated with the book ''The Art of Assembly'' by the same author.
** ''Alternate-Syntax Assemblers'' - This is a subclass of HLAs which eschew conventional mnemonics for a high-level style syntax, while still only allowing operators which directly correspond to the machine code. Some of these are also ''universal assembly languages'', which provide operators corresponding to the machine code operators of a specific machine, or a subset of them common to many different architectures, but do so in a general syntax similar to some high-level language which it is modeled after (usually C) and can usually be ported to and reassembled on other systems with only some relatively minor adjustments (something not normally possible with assembly code). One of the languages using the name "C--" is an assembler of this type. 
* '''Macro-Extensible Assemblers''' - these are assemblers whose macro facilities are capable of extending and altering the syntax of the language itself, sometimes to the point of becoming, in effect, a high-level language implemented as a set of macros rather than a compiler or interpreter. These were a subject of experimentation in the 1960s, during the period when compiler technology was in its infancy - these powerful macro assemblers seemed to offer an alternative to the problem of implementing a full compiler. The best known example of this is the SNOBOL4 'interpreter', which would convert a line or several lines of SNOBOL into an assembly language program through syntax transformations, assemble it, and execute it before returning to its REPL. These have mostly disappeared since the early 1970s, as compiler design improved to the point where writing a compiler was no longer more difficult than writing a macro preprocessor capable of this together with the macros for doing the transformations.
````

<a id="w004218"></a>

## W004218 — User:Schol-r-lea/Brief Tutorial On Data Strctures In Assembly Language

Bron-ID: W004218
Namespace: User (2)
Revisie: 21750 | Datum: 2017-11-25T22:01:17Z
Bron: https://wiki.osdev.org/User:Schol-r-lea/Brief_Tutorial_On_Data_Strctures_In_Assembly_Language
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASchol-r-lea%2FBrief_Tutorial_On_Data_Strctures_In_Assembly_Language&oldid=21750
Doorverwijzing: User:Schol-r-lea/Brief Tutorial On Data Structures In Assembly Language

````text
#REDIRECT [[User:Schol-r-lea/Brief Tutorial On Data Structures In Assembly Language]]
````

<a id="w004219"></a>

## W004219 — Assembly Language

Bron-ID: W004219
Namespace: Main (0)
Revisie: 21765 | Datum: 2017-11-26T16:44:45Z
Bron: https://wiki.osdev.org/Assembly_Language
Revisie-URL: https://wiki.osdev.org/index.php?title=Assembly_Language&oldid=21765
Doorverwijzing: Assembly

````text
#REDIRECT[[Assembly]]
````

<a id="w004220"></a>

## W004220 — File System

Bron-ID: W004220
Namespace: Main (0)
Revisie: 21768 | Datum: 2017-11-26T17:51:44Z
Bron: https://wiki.osdev.org/File_System
Revisie-URL: https://wiki.osdev.org/index.php?title=File_System&oldid=21768
Doorverwijzing: File Systems

````text
#REDIRECT[[File Systems]]
````

<a id="w004221"></a>

## W004221 — ARM Paging

Bron-ID: W004221
Namespace: Main (0)
Revisie: 30870 | Datum: 2026-07-07T17:36:55Z
Bron: https://wiki.osdev.org/ARM_Paging
Revisie-URL: https://wiki.osdev.org/index.php?title=ARM_Paging&oldid=30870

````text
{{In Progress}}
== Introduction ==
ARM CPUs are used in smaller applications than x86 CPUs, although the line is blurring. Due to the number of different ARM architectures, details of which can differ significantly, this page is aimed at ARMv7-A and ARMv8. ARMv7-M does not have the same concept of virtual memory - it does not have an MMU. The author understands that paging on ARMv5 and ARMv6 is similar to ARMv7, but ARMv4 is somewhat different.

== ARMv7-A ==
ARMv7-A supports two different paging modes. These are the short descriptor format and long descriptor format described in B3.5 and B3.6 respectively of the ARMv7 reference manual.
The long descriptor format is an ARM equivalent of the X86 [[PAE]] system. However, even the short descriptor format allows access to a 1TB physical address space, but only with a 16MB granularity. As is described in the ARM:
=== Overview ===
Detecting paging support is done by consulting the "coprocessor" registers. It is represented in ID_MMFR0
<syntaxhighlight lang="asm">
mcr p15, 0, <Rt>, c0, c1, 4
</syntaxhighlight>
The structure of the register is as follows:
{| class="wikitable"
|-
! 31-28
! 27-24
! 23-20
! 19-16
! 15-12
! 11-8
! 7-4
! 3-0
|-
| Innermost Shareability
| FCSE Support
| Auxillary Registers
| TCM Support
| Shareability levels
| Outermost Shareability
| PMSA Support
| VMSA Support
|}
* VMSA support - the gold.
** 0b0000 - not supported (no paging)
** 0b0001 - implementation defined. Weird MMU somewhere
** 0b0010 - VMSAv6, with cache and TLB type registers. ARMv6 paging.
** 0b0011 - VMSAv7, with support for remapping and access flag. ARMv7-A, as described in the following section.
** 0b0100 - VMSAv7 with PXN bit supported.
** 0b0101 - VMSAv7, PXN and long format descriptors. EPAE is supported.
==== Short Format ====
* Up to two levels of address lookup
* 32 bit input addresses
* Output addresses up to 40 bits
* Supports >32 bit Physical Addresses with supersections
* Support for No access, Client and Manager domains
* 32 bit table entries
==== Long Format ====
* Up to three levels of address lookup
* Input addresses of up to 40 bits, when used for stage 2 translations
* Output addresses of up to 40 bits
* 4KB assignment granularity across the entire PA range
* No support for domains, all memory regions are treated as in a Client domain
* 64-bit table entries
* Fixed 4KB table size, unless truncated by the size of the input address space

Note that the Large Physical Address Extension is an optional feature. Furthermore, if an implementation supports LPAE, it also supports the ARM multiprocessing extensions.
The paging mode is controlled with the TTBCR (Translation Table Base Control Register).
=== Control Registers ===
==== TTBCR ====
{| class="wikitable"
|-
! 31
! 30
! 29-28
! 27-26
! 25-24
! 23
! 22
! 21-19
! 18-16
! 15-14
! 13-12
! 11-10
! 9-8
! 7
! 6
! 5
! 4
! 3
! 2-0
|-
| EAE
| IDF
| SH1
| ORGN1
| IRGN1
| EPD1
| A1
| SBZP
| T1SZ
| SBZP
| SH0
| ORGN0
| IRGN0
| EPD0
| SBZP
| PD1
| PD0
| SBZP
| T0SZ
|}

* EAE - Extended Address Enable. SBZP if LPAE is not supported
'''Following fields are SBZP if EAE=0'''
* IDF - Implementation Defined
* SH1 - Shareability attribute for memory associated with translation table walks using TTBR1.
{| class="wikitable"
|-
! 00
! 01
! 10
! 11
|-
| non-shareable
| unpredictable
| outer shareable
| inner shareable
|}
* ORGN1 - Outer cacheability using TTBR1
{| class="wikitable"
|-
! 00
! 01
! 10
! 11
|-
| outer non-cacheable
| outer write-back write-allocate cacheable
| outer write-through cacheable
| outer write-back no write-allocate cacheable
|}
* IRGN1 - Inner cacheablility using TTBR1
{| class="wikitable"
|-
! 00
! 01
! 10
! 11
|-
| inner non-cacheable
| inner write-back write-allocate cacheable
| inner write-through cacheable
| inner write-back no write-allocate cacheable
|}
* EPD1 - Disable Page walks with TTBR1. If 0, table walks are performed. Otherwise, a translation fault is generated.
* A1 - defines whether TTBR0 or TTBR1 defines the ASID, for 0 and 1 respectively. The ASID is the Address Space Identifier.
* SBZP - Should Be Zero or Preserved. This is more commonly called RES0.
* T1SZ - The size of the memory region addressed by TTBR1. 2^(32-T1SZ) is the size.
* SH0 - like SH1, but for TTBR0.
* ORGN0 - ""
* IRGN0 - ""
* EPD0 - ""
'''The following fields only apply when EAE is 0'''
* PD1 - like EPD1.
* PD0 - like EPD0.
'''This field can take different meanings
* T0SZ - like T1SZ. If EAE=0, this is field N.
* N - Indicated the width of the base address in TTBR0. The base address is bits [31:14-N]. If N=0, the format is compatible with ARMv5 and ARMv6. This field also determines whether TTBR0 or TTBR1 is used for the page walk.
===== Accessing the TTBCR =====
To access TTBCR, software reads or writes the CP15 registers with <opc1> set to 0, <CRn> set to c2, <CRm> set to c0,
and <opc2> set to 2. For example:
<syntaxhighlight lang="asm">
MRC p15, 0, <Rt>, c2, c0, 2 ; Read TTBCR into Rt
MCR p15, 0, <Rt>, c2, c0, 2 ; Write RT to TTBCR
</syntaxhighlight>
(ARMv7-A ARM, Section B4.1, page 1728)<br />
Here RT denotes a register of your choice.
==== TTBR0 ====
===== EAE=0 =====
{| class="wikitable"
|-
! 31-x
!(x-1)-7
! 6
! 5
! 4-3
! 2
! 1
! 0
|-
| TTB0A
| SBZP
| IRGN[0]
| NOS
| RGN
| IMP
| S
| C/IRGN[1]
|}
*TTB0A - Bits [31:x] of the TTB0 table base address. Must be 2^x aligned, as determined in the TTBCR.
* IRGN[0] - SBZP if Multiprocessor extensions are not present. Otherwise, bit zero of IRGN
* NOS - Not Outer Shareable. If 1, region is only inner shareable. Ignored when TTBR0.S == 0, SBZP if no distinction between outer or inner shareable.
* RGN - region bits. Outer cacheability attributes, see TTBCR.ORGN0.
* IMP - Implementation Defined
* S - Shareable. 0 - non-shareable, 1 - shareable.
* C - Cacheable. 0 - inner non-cacheable, 1 - inner cacheable. If Multiprocessor extensions are present, this is bit 1 of IRGN
* IRGN - inner region attributes. See TTBCR.IRGN0
===== EAE = 1 =====
{| class="wikitable"
|-
! 63-56
! 55-48
! 47-40
! 39-x
! (x-1)-0
|-
| SBZP
| ASID
| SBZP
| BADDR
| SBZP
|}
* ASID - Address Space Identifier
* BADDR - Bits [39:x] of base address of table. Must be 2^x aligned.
==== TTBR1 ====
See [[#TTBR0]], except x is fixed to 14 when EAE=0.

==== Accessing the TTBRx register ====
To access TTBR0 in an implementation that does not include the Large Physical Address Extension, or bits[31:0]
of TTBR0 in an implementation that includes the Large Physical Address Extension, software reads or writes the
CP15 registers with <opc1> set to 0, <CRn> set to c2, <CRm> set to c0, and <opc2> set to 0. For example:
<syntaxhighlight lang="asm">
MRC p15, 0, <Rt>, c2, c0, 0 ; Read 32-bit TTBR0 into Rt
MCR p15, 0, <Rt>, c2, c0, 0 ; Write Rt to 32-bit TTBR0
</syntaxhighlight>
In an implementation that includes the Large Physical Address Extension, to access all 64 bits of TTBR0, software
performs a 64-bit read or write of the CP15 registers with <CRm> set to c2 and <opc1> set to 0. For example:
<syntaxhighlight lang = "asm">
MRRC p15, 0, <Rt>, <Rt2>, c2 ; Read 64-bit TTBR0 into Rt (low word) and Rt2 (high word)
MCRR p15, 0, <Rt>, <Rt2>, c2 ; Write Rt (low word) and Rt2 (high word) to 64-bit TTBR0
</syntaxhighlight>
In these MRRC and MCRR instructions, Rt holds the least-significant word of TTBR0, and Rt2 holds the most-significant
word.

To access TTBR1 in an implementation that does not include the Large Physical Address Extension, or bits[31:0]
of TTBR1 in an implementation that includes the Large Physical Address Extension, software reads or writes the
CP15 registers with <opc1> set to 0, <CRn> set to c2, <CRm> set to c0, and <opc2> set to 1. For example:
<syntaxhighlight lang="asm">
MRC p15, 0, <Rt>, c2, c0, 1 ; Read 32-bit TTBR1 into Rt
MCR p15, 0, <Rt>, c2, c0, 1 ; Write Rt to 32-bit TTBR1
</syntaxhighlight>
In an implementation that includes the Large Physical Address Extension, to access all 64 bits of TTBR1, software
performs a 64-bit read or write of the CP15 registers with <CRm> set to c2 and <opc1> set to 1. For example:
<syntaxhighlight lang="asm">
MRRC p15, 1, <Rt>, <Rt2>, c2 ; Read 64-bit TTBR1 into Rt (low word) and Rt2 (high word)
MCRR p15, 1, <Rt>, <Rt2>, c2 ; Write Rt (low word) and Rt2 (high word) to 64-bit TTBR1
</syntaxhighlight>
In these MRRC and MCRR instructions, Rt holds the least-significant word of TTBR1, and Rt2 holds the most-significant
word.

=== Page Tables ===
OK, so now we know how to set up the control registers, we need to know the in-memory structures.
ARM is more flexible than x86 in terms of page size. While x86 supports 4KB and 4MB pages, ARM supports 4KB, 64KB and 1MB pages.
It can also support 16MB pages, but this is optional (guaranteed
The first level translation table is 16KB in size when N = 0. Generally, it is 2^(14-N).
==== Short Descriptor: Level 1 ====
Invalid:
{| class="wikitable"
|-
! 31-2
! 1
! 0
|-
| Ignored
| 0
| 0
|}
Page Table:
{| class="wikitable"
|-
! 31-10
! 9
! 8-5
! 4
! 3
! 2
! 1
! 0
|-
| BADDR
| IMP
| Domain
| SBZ
| NS
| PXN
| 0
| 1
|}
* BADDR - Page table base address, bits [31:10]. Must be 1KB aligned.
* IMP - Implementation defined
* Domain - used as a memory protection mechanism. 16 possible domains.
* SBZ - Should be Zero
* NS - Non-secure bit. This is used by the security extensions
* PXN - Privileged Execute Never - SBZ if PXN is not supported

Section:
{| class="wikitable"
|-
! 31-20
! 19
! 18
! 17
! 16
! 15
! 14-12
! 11-10
! 9
! 8-5
! 4
! 3
! 2
! 1
! 0
|-
| BADDR
| NS
| 0
| nG
| S
| AP[2]
| TEX[2:0]
| AP[1:0]
| IMP
| Domain
| XN
| C
| B
| 1
| PXN
|}
* BADDR - Section Base Address [31:20]. Must be 1MB aligned.
* XN - Execute never. Stops execution of page.
* nG - not global. Determines how this is marked in the TLB.

Supersection:
{| class="wikitable"
|-
! 31-24
! 23-20
! 19
! 18
! 17
! 16
! 15
! 14-12
! 11-10
! 9
! 8-5
! 4
! 3
! 2
! 1
! 0
|-
| BADDR
| BADDRM
| NS
| 1
| nG
| S
| AP[2]
| TEX[2:0]
| AP[1:0]
| IMP
| BADDRH
| XN
| C
| B
| 1
| PXN
|}
* BADDR - Supersection Base address [31:24]. 16MB aligned.
* BADDRM - Supersection base address [35:32], if supported.
* BADDRH - Supersection base address [39:36], if supported.

==== Short Descriptor: Level 2 ====
Invalid:
{| class="wikitable"
|-
! 31-2
! 1
! 0
|-
| Ignored
| 0
| 0
|}
Large Page:
{| class="wikitable"
|-
! 31-16
! 15
! 14-12
! 11
! 10
! 9
! 8-6
! 5-4
! 3
! 2
! 1
! 0
|-
| BADDR
| XN
| TEX[2:0]
| nG
| S
| AP[2]
| SBZ
| AP[1:0]
| C
| B
| 0
| 1
|}
* BADDR - Large Page Base Address [31:16]

Small page:
{| class="wikitable"
|-
! 31-12
! 11
! 10
! 9
! 8-6
! 5-4
! 3
! 2
! 1
! 0
|-
| BADDR
| nG
| S
| AP[2]
| TEX[2:0]
| AP[1:0]
| C
| B
| 1
| XN
|}
* BADDR - Small Page Base Address [31:12]

==== Long Descriptor: Level 1/2 ====
Invalid:
{| class="wikitable"
|-
! 63-1
! 0
|-
| Ignored
| 0
|}
Block:
{| class="wikitable"
|-
! 63-52
! 51-40
! 39-n
! (n-1)-12
! 11-2
! 1
! 0
|-
| UBAT
| SBZP
| ADDR
| SBZP
| LBAT
| 0
| 1
|}
* UBAT - Upper Block Attributes
* ADDR - Output Address [39:n]
* LBAT - Lower Block Attributes
* n - 30 for first level, 21 for 2nd level.

Table:
{| class="wikitable"
|-
! 63
! 62-61
! 60
! 59
! 58-52
! 51-40
! 39-12
! 11-2
! 1
! 1
|-
| NST
| APT
| XNT
| PXNT
| Ignored
| SBZP
| ADDR
| Ignored
| 1
| 1
|}
Stage 1 attributes, SBZP at stage 2:
* NST - NSTable. For secure memory accesses, determines type of next level. Otherwise ignored.
* APT - APTable. Access permissions limit for next level lookup.
* XNT - XNTable. XN limit for subsequent lookups.
* PXNT - PXNTable. PXN limit for subsequent levels. SBZ for non-secure PL2 (hypervisor) level 1 translation tables.
Any stage:
* ADDR - Next level table address [39:12].

==== Long Descriptor: Level 3 ====
Invalid:
{| class="wikitable"
|-
! 63-1
! 0
|-
| Ignored
| 0
|}
Reserved:
{| class="wikitable"
|-
! 63-2
! 1
! 0
|-
| SBZP
| 0
| 1
|}
Page
{| class="wikitable"
|-
! 63-52
! 51-40
! 39-12
! 11-2
! 1
! 1
|-
| UPAT
| SBZP
| ADDR
| LPAT
| 1
| 1
|}
* UPAT - Upper Page Attributes
* ADDR - Output Address [39:12]
* LPAT - Lower Page Attributes

===== Stage 1 Attributes =====
Upper:
{| class="wikitable"
|-
! 63-59
! 58-55
! 54
! 53
! 52
|-
| Ignored
| Ignored
| XN
| PXN
| CONT
|}
* CONT - Contiguous. 16 adjacent entries point to contiguous memory regions.
Lower:
{| class="wikitable"
|-
! 11
! 10
! 9-8
! 7-6
! 5
! 4-2
|-
| nG
| AF
| SH[1:0]
| AP[2:1]
| NS
| AttrIndex[2:0]
|}
* AttrIndex - memory attributes index field
===== Stage 2 Attributes =====
Upper:
{| class="wikitable"
|-
! 63-59
! 58-55
! 54
! 53
! 52
|-
| Ignored (system MMU)
| Ignored (software)
| XN
| 0
| CONT
|}
Lower:
{| class="wikitable"
|-
! 11
! 10
! 9-8
! 7-6
! 5-2
|-
| 0
| AF
| SH[1:0]
| HAP[2:1]
| MemAttr[3:0]
|}
* HAP - Stage 2 Access Permissions
* MemAttr - Stage 2 memory attributes

=== Choice Between TTBR0 and TTBR1 ===
==== Short format ====
{| class = "wikitable"
|-
!
!
! colspan="3" | TTBR0 table
|-
! TTBCR.N 
!First address translated with TTBR1
! Size 
! Index range
|-
| 0b000 
| TTBR1 not used 
| 16KB
| VA[31:20]
|-
| 0b001
| 0x80000000
| 8KB
| VA[30:20]
|-
| 0b010
| 0x40000000
| 4KB
| VA[29:20]
|-
| 0b011
| 0x20000000
| 2KB
| VA[28:20]
|-
| 0b100
| 0x10000000
| 1KB
| VA[27:20]
|-
| 0b101
| 0x08000000
| 512 bytes
| VA[26:20]
|-
| 0b110
| 0x04000000
| 256 bytes
| VA[25:20]
|-
| 0b111
| 0x02000000
| 128 bytes
| VA[24:20]
|}
==== Long format ====
{| class="wikitable"
|-
! colspan="2" | TTBCR
! colspan="2" | Input address range
|-
! T0SZ
! T1SZ
! TTBR0
! TTBR1
|-
| 0b000
| 0b000
| All addresses
| Not used
|-
| M
| 0b000
| 0 to 2^(32-M)-1
| 2^(32-M) to maximum address
|-
| 0b000
| N
| 0 to (2^32-2^(32-N)-1)
| (2^32-2^(32-N)) to maximum
|-
| M
| N
| 0 to (2^(32-M)-1)
| 2^32-2^(32-N) to maximum
|}

=== Recursive Table Mapping ===
If you wish to use a recursive mapping trick, such as is done on x86, use the long format. The short format might be doable with much hastle. The long format has been designed with recursive mapping in mind, supports the full physical address space, and does not suffer from mismatching of the page size to table like the short format does. However, the basic structure is similar to PAE, so if you did this naively, 1GB of the address space would be wasted. Instead, use the support for eliminating the first stage by concatenating the 2nd stage. Then point the last few entries to the appropriate points in the table. This reaps all the benefits of the long format, while eliminating bloat of the recursive mapping.

== ARM v8-A ==
ARM v8-A introduces a 64 bit architecture, AArch64.

The first step in enabling paging is reading the architecture memory model feature registers.

You access this via the '''ID_AA64MMFR0_EL1''' register.<syntaxhighlight lang="asm">
MRS <Xt>, ID_AA64MMFR0_EL1
</syntaxhighlight>
{| class="wikitable"
|+ID_AA64MMFR0_EL1
|-
!63-60
!59-56
!55-48
!47-44
!43-40
!39-36
!35-32
!31-28
!27-24
!23-20
!19-16
!15-12
!11-8
!7-4
!3-0
|-
|ECV
|FGT
|RES0
|ExS
|TGran4_2
|TGran64_2
|TGran16_2
|TGran4
|TGran64
|TGran16
|BigEndEL0
|SNSMem
|BigEnd
|ASIDBits
|PARange
|}
{| class="wikitable"
|+4KB translation granularity
!TGran4
!Support
|-
|0000
|Supported
|-
|0001
|Supports 52-bit input/output addresses (FEAT_LPA2)
|-
|1111
|Unsupported
|}
{| class="wikitable"
|+64KB translation granularity
!TGran64
!Support
|-
|0000
|Supported
|-
|1111
|Unsupported
|}
{| class="wikitable"
|+16KB translation granularity
!TGran16
!Support
|-
|0000
|Unsupported
|-
|0001
|Supported
|-
|0010
|Supports 52-bit input/output addresses (FEAT_LPA2)
|}
{| class="wikitable"
|+Mixed Endian Support
!BigEnd
!Support
|-
|0000
|Unsupported
|-
|0001
|Supports mixed endian
|}
BigEndEL0 is whether mixed endian is supported at EL0 (applications).
{| class="wikitable"
|+Address Space ID Length
!ASIDBits
!Number of Bits
|-
|0000
|8
|-
|0010
|16
|}
{| class="wikitable"
|+Physical Address Range
!PARange
!Number of Bits
!Max Memory
|-
|0000
|32
|4GB
|-
|0001
|36
|64GB
|-
|0010
|40
|1TB
|-
|0011
|42
|4TB
|-
|0100
|44
|16TB
|-
|0101
|48
|256TB
|-
|0110
|52
|4PB
|-
|0111
|56
|64PB
|}
'''Note that addresses >52 bits require the VMSAv9-128 translation system.'''

Now you have this information, the '''TCR_EL1''' can be configured, which controls translations for EL1&EL0. This is somewhat similar to the ''TTBCR'' of ARMv7-A.

==== TCR_EL1 ====
<syntaxhighlight lang="asm">
MRS <Xt>, TCR_EL1
MRS TCR_EL1, <Xt>
</syntaxhighlight>'''Upper:'''
{| class="wikitable"
|-
! 63-62
! 61
! 60
! 59
! 58
! 57
! 56
!55
!54
!53
!52
!51
|-
| RES0
| MTX1
| MTX0
| DS
| TCMA1
| TCMA0
| E0PD1
|E0PD0
|NFD1
|NFD0
|TBID1
|TBID0
|}
{| class="wikitable"
|-
!50-46
!45-41
!42
!41
! 40
! 39
! 38
! 37
! 36
! 35
! 34-32
|-
|HWU1
|HWU0
|HPD1
|HPD0
| HD
| HA
| TBI1
| TBI0
| AS
| RES0
| IPS
|}
'''Lower:'''
{| class="wikitable"
|-
! 31-30
! 29-28
! 27-26
! 25-24
! 23
! 22
! 21-16
! 15-14
! 13-12
! 11-10
! 9-8
! 7
! 6
! 5-0
|-
| TG1
| SH1
| ORGN1
| IRGN1
| EPD1
| A1
| T1SZ
| TG0
| SH0
| ORGN0
| IRGN0
| EPD0
| RES0
| T0SZ
|}
'''MTX -''' Extended memory tag checking. If 1, then bits [59:56] of a VA hold a logical address tag (for TTBR1 and TTBR0 respectively).

'''DS -''' When not using VMSAv9-128, and FEAT_LPA2 is implemented. Set for output address bits 51:48 to be non-zero.

'''TCMA''' - Sets accesses as unchecked in the memory tagging extension. (FEAT_MTE2)

'''E0PD1''' - Unprivileged access to any address with TTBR1_EL1 will generate a translation fault.

'''E0PD0''' - Unprivileged access to any address with TTBR0_EL1 will generate a translation fault.

'''NFD''' - Non-fault translation timing disable.

'''TBID''' - FEAT_PAuth, address tagging.

'''HWUx''' - [62:59], allows hardware use of bit [n] when using TTBRx.

'''HPD''' - Hierarchical permission disables.

'''HD''' - Hardware management of Dirty state.

'''HA''' - Hardware access flag update.

'''TBI'''  - Top byte ignored, used for address tagging.

'''AS''' - ASID size. 0 = 8 bit, 1 = 16 bit.

'''IPS -''' Intermediate physical address size. Same as for PARange.

'''TGx''' - Granule size for TTBRx_EL1. 01=16KB, 10=4KB, 11=64KB.

'''SHx'''- Shareability attribute for TTBRx_EL1. 00=non-shareable, 10=Outer shareable, 11 = Inner shareable.

'''ORGNx''' - Outer cacheabilty for memory in walks through TTBRx_EL1.

'''IRGNx''' - Inner cacheabilty for memory in walks through TTBRx_EL1.

'''EPDx''' - Disable translation walks for TTBRx_EL1.

'''A1''' - Selects where ASID is defined. 0 for TTBR0_EL1, 1 for TTBR1_EL1.

'''TxSZ''' - Size of memory region addressed by TTBRx_EL1. The size is 2^(64-TxSZ).

==== TCR2_EL1 ====
Only when FEAT_TCR2 is implemented.
{| class="wikitable"
|-
! 63-22
! 21
! 20
! 19
! 18
! 17
! 16
! 15
!14
! 13-12
! 11
!10
! 9-6
! 5
! 4
! 3
!2
!1
!0
|-
| RES0
| FNGNA1
| FNGNA0
| RES0
| FNG1
| FNG0
| A2
| DisCH1
|DisCH0
| RES0
| HAFT
|PTTWI
| RES0
| D128
| ATE
| POE
|E0POE
|PIE
|PnCH
|}


==External References==
*[http://www.embedded-bits.co.uk/2011/mmucode/ Turning on an ARM MMU]
*[https://github.com/bztsrc/raspi3-tutorial/tree/master/10_virtualmemory Tutorial for 64 bit ARM MMU]

[[Category:ARM]]
[[Category:AArch64]]
[[Category:Paging]]
[[Category:Memory management]]
````

<a id="w004225"></a>

## W004225 — User:Johnwilson

Bron-ID: W004225
Namespace: User (2)
Revisie: 21828 | Datum: 2017-12-05T20:23:13Z
Bron: https://wiki.osdev.org/User:Johnwilson
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnwilson&oldid=21828

````text

== About me ==
I'm John Wilson.  Programming in 80x86 assembly since 1983, first using MASM (V1.00) and later TASM (V2.01 - V5.0).  I've been chipping away at my own OS projects most of that time.  The one that's actually seen the light of day is a minicomputer emulator I've been working on since 1993, which ended up in a "Windows 3.1"-like situation of being 99% of an OS but depending on DOS just for file access and a few other things (and even there of course it set up a 32-bit PM environment and occasionally even enabled SMP, all behind DOS's back).  So I wrote FAT and ISO9660 boot loaders, a FAT filesystem driver, date/time stuff, and a low-memory allocator, yielding an emulator which looks exactly like the DOS version but needs no DOS.  The idea is that it's for users who think "DOS" is a dirty word -- and yes I've also ported the emulator to Windows, Linux, and, for some reason, OS/2, but the drivers for legacy bus adapters work best under DOS since the application can access the hardware directly w/o having everything dribble through an OS-supplied driver/event framework.  And if UEFI really does kill DOS for good in 2020, well it's a good thing I'll have three years to update my code to be able to live entirely without the BIOS (which it uses for PCI device discovery when necessary, and for finding memory, and video mode changes, and INT 13h is still the default hard-drive access method until I really trust my newish ATA/SATA driver, and, um, figure out how to reliably correlate INT 13h drive numbers with ATA channels/units).

Also, years ago I wrote a stand-alone four-user FORTH-79 environment for the PDP-11, which boots from an RL01 or RL02 disk (or DECtape, but that's untested in ~20 years since my DECtape hardware is pining for the fjords) and does time-slicing so everyone gets along happily, as long as they stay out of each other's memory (there's no protection -- the MMU is never turned on since I originally wrote the system on a PDT-11/150 machine that doesn't have one).  As part of that, I wrote a series of boot blocks (for various disk-controller types) which could be written onto a disk with an RT-11 directory structure and could load any one particular .SAV file as a stand-alone program, which was useful for the Fuzzball OS too.

== Development environment ==
DR-DOS V8.0 (bought before it was withdrawn due to GPL violation), my own editor, Borland TASM V5.0 (bought when it was still available, after I was *very* disappointed with MASM V6 since it broke all my code), and Open Watcom WLINK V1.9.  My own MAKE (I needed one which allows wildcards in rules -- Watcom WMAKE seems to but it's unusably slow and the behavior is not documented so I'm not sure it's doing what I mean), and my own MAKEISO and MAKETAR utilities so that I could do those on DOS w/o having to move things to Linux for part of every build.  All my coding is done in assembly language, officially because it's faster, smaller, and more powerful (which is all true), but really because I think in assembly language anyway and HATE C SO MUCH.  I mean it's not as bad as most of the other semicolon languages but it still gets in the way more than it helps.
````

<a id="w004226"></a>

## W004226 — File:Icon floppy.png

Bron-ID: W004226
Namespace: File (6)
Revisie: 21844 | Datum: 2017-12-13T13:27:11Z
Bron: https://wiki.osdev.org/File:Icon_floppy.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AIcon_floppy.png&oldid=21844

````text
A floppy.
````

<a id="w004227"></a>

## W004227 — File:Icon clock.png

Bron-ID: W004227
Namespace: File (6)
Revisie: 21846 | Datum: 2017-12-13T14:01:45Z
Bron: https://wiki.osdev.org/File:Icon_clock.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AIcon_clock.png&oldid=21846

````text
New clock icon.
````

<a id="w004228"></a>

## W004228 — File:Icon phone.png

Bron-ID: W004228
Namespace: File (6)
Revisie: 21849 | Datum: 2017-12-13T14:33:45Z
Bron: https://wiki.osdev.org/File:Icon_phone.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AIcon_phone.png&oldid=21849

````text
New phone icon.
````

<a id="w004229"></a>

## W004229 — File:Icon site.png

Bron-ID: W004229
Namespace: File (6)
Revisie: 21850 | Datum: 2017-12-13T15:22:51Z
Bron: https://wiki.osdev.org/File:Icon_site.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AIcon_site.png&oldid=21850

````text
New site icon.
````

<a id="w004230"></a>

## W004230 — File:Icon license.png

Bron-ID: W004230
Namespace: File (6)
Revisie: 21852 | Datum: 2017-12-13T16:39:34Z
Bron: https://wiki.osdev.org/File:Icon_license.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AIcon_license.png&oldid=21852

````text
Another great new icon, also made by myself ;) Concept of a license icon: human hand which is giving a key...
````

<a id="w004231"></a>

## W004231 — User:Updater3

Bron-ID: W004231
Namespace: User (2)
Revisie: 21865 | Datum: 2017-12-14T09:34:38Z
Bron: https://wiki.osdev.org/User:Updater3
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AUpdater3&oldid=21865

````text
Good day, stranger ;) My primary goal at this wiki is to update the heavily outdated Projects page<br> Also I created 5 nice new icons for OSProject template - the old icons from 2007 year were so ugly :D
````

<a id="w004232"></a>

## W004232 — User talk:Updater3

Bron-ID: W004232
Namespace: User talk (3)
Revisie: 22332 | Datum: 2018-04-15T21:52:16Z
Bron: https://wiki.osdev.org/User_talk:Updater3
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AUpdater3&oldid=22332

````text
Good day, stranger ;) My primary goal at this wiki is to update the heavily outdated Projects page<br> Also I created 5 nice new icons for OSProject template - the old icons from 2007 year were so ugly :D

== Warning: Please stop worsening the project lists ==

Hi. I'm a wiki admin and I reverted your changes to [[Notable_Projects]] and [[Projects]] for the reasons outlined in the edit messages, and now you have restored the changes, starting an edit war. This is an official wiki warning to stop making these wiki pages worse. The notable projects page is meant to contain the larger, sustained projects, that have a good degree of notability, and serves as useful bases for people to learn osdev from; or projects that are very far and very notable (even if they are currently inactive). It's not a top 10 (or top 11) list of projects (although we may want to reduce it to no more than a few dozen). Your top 10 list is biased towards systems that aren't the best examples to learn from, in my personal opinion, though many of them are fine, I don't really see evidence that some of them are notable (and there are more notable ones you omit). Please don't accuse wiki admins of vandalism for trying to undo your changes that are really difficult to undo because you make many edits (I've tried to preserve the edits you made that I agree with). Floppy support really is an irrelevant part of operating systems, a much more interesting thing would be whether it's a microkernel or monolithic, or whether there's an ARM64 port, or so on. I'll revert your recent changes. I don't have time to do surgical reverts of your changes. It seems personal that you removed my Sortix operating system, which should be objectively notable by osdev standards since it's been covered a number of times on various sites. Feel free to make your own top 10 (or 11) list in your personal namespace. If you continue this edit war, I will be forced to revoke your wiki edit privileges. --[[User:Sortie|Sortie]] 16:52, 15 April 2018 (CDT)
````

<a id="w004233"></a>

## W004233 — Abandoned Projects

Bron-ID: W004233
Namespace: Main (0)
Revisie: 28552 | Datum: 2024-01-09T21:37:07Z
Bron: https://wiki.osdev.org/Abandoned_Projects
Revisie-URL: https://wiki.osdev.org/index.php?title=Abandoned_Projects&oldid=28552

````text
__NOTOC__
This page maintains a list of 76 academic, personal, and small non-commercial operating systems - which are considered as have been abandoned by their creators. For information regarding commercial or mainstream operating systems, please visit [[wikipedia:Operating_system|Wikipedia]].

OSes have a high mortality rate: between 2004 and 2006, 112 of 213 OS Projects disappeared from the Internet. And looking at how often these projects are started by using the [[Forum:2|OS Project Announcement forum]], in that same time period 68 projects were announced! The current total is 175 projects, but just 58 of them are active (as of January 1, 2018) and only 99 have been updated at least once since 2015. Please help keep this list current by correcting it if you see any outdated information.

'''Contents:''' [[#A|A]] [[#B|B]] [[#C|C]] [[#D|D]] [[#E|E]] [[#F|F]] [[#G|G]] [[#H|H]] [[#I|I]] [[#J|J]] [[#K|K]] [[#L|L]] [[#M|M]] [[#N|N]] [[#O|O]] [[#P|P]] [[#Q|Q]] [[#R|R]] [[#S|S]] [[#T|T]] [[#U|U]] [[#V|V]] [[#W|W]] [[#X|X]] [[#Y|Y]] [[#Z|Z]] [[#9|9]]

= A =

{{OSProject
|OSName=Adelian
|OSDescription=A desktop OS based on our RTOS (MDOS). Designed from scratch, this modular OS features a GUI, CLI, VFS, sound etc etc. Complete nano-kernel plus many completed modules
|Contact=Gary Partis, info [at] adelian.co.uk
|License=Proprietary :(
|URL=http://www.adelian.co.uk
|Status=Last update: 2010-07-03, but is still being used as a basis for the "Picture Frame" commercial product
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=Aksel
|OSDescription=Aksel is a small kernel which was saved at different stages of the development so that it can be easily used as a learning tool. It can be used with GRUB or on its own for stages where its size is less the 64K as I did not work around to solve the fact that you cannot access more than 1MB+64kB from memory in Real Mode, but I used GRUB instead. The commands implemented so far are: cls, mem, date, time, esp (shows stack pointer), and test (used to test malloc). Of course previous stages do not have all the commands implemented. It only handles lower case and numbers, no key+shiftkey has been implemented yet. Aksel is compiled with DJGCC and uses MTOOLS to build the image for BOCHS. Inside the archives, e.g. ''command interface improved'', there is a floppy available for testing
|Contact=ivandiprima [at] gmail [dot] com
|License=Open source
|URL=http://www.3turtles.co.uk
|Status=Last update: 2011-12-08, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=Amoeba
|OSDescription=Amoeba is a distributed operating system developed by Andrew S. Tanenbaum, the creator of Minix
|Contact=amoeba-support [at] cs.vu.nl
|License=Open source (MIT)
|URL=http://www.cs.vu.nl/pub/amoeba/
|Status=Last update: 1998-12-01, abandoned
|FloppyAvailable=Too fat for a floppy! 26 MB size
}}

{{OSProject
|OSName=Aprom
|OSDescription=Aprom is a weird, modular 32bit OS. Tightly bound design, roughly microkernel. Been in development every now and then over the past decade. A spontaneous fusion of a large algorithms library. Stuff from kernel to big stable programs present, but hardware support is lousy. Continually developed
|Contact=artlav at OSDev forum
|License=Proprietary :(
|URL=http://orbides.1gb.ru/aprom.php
|Status=Last release: 2010-10-23
|FloppyAvailable=Fits on a floppy! :) ("basic version"; full is 12MB)
}}

{{OSProject
|OSName=Arcrascent OS
|OSDescription=Arcrascent OS is a x86/x86-64 operating system in development since June 2013. It is fully UNIX-compatible but not an Unix clone. It includes FAT12/16/32 and EXT2/EXT3 drivers, floppy and (S)ATA drivers, (very) basic networking, sound driver, keyboard/mouse driver. It also has a real-mode emulator, multitasking, process prioritization, very advanced serial console ("SerCon"), an enhanced device file system for intercomponent communication, full-fledged ACPI controller (supports rebooting and shutdown). This operating system has a very unique Superiority management system where a superelevated process can, through the "SerCon" execute commands at RING0 level for programs that require direct hardware access. The goal is to be user-friendly and to provide a comfortable environment for both developer and regular users. An upcoming kernel module is called "CSM" and contains complex algorithms that will improve human machine interaction. The module currently has the ability to interpret English statements and extract new information it can obtain from such statements. The language interpreter will support visual & auditory inputs and even support outputs in multiple forms of communication (e.g. text, visual, sound). When completed, this innovative ''AI-like'' module will support machine "imagination" and inference, making a computer feel less like a machine!
|Contact=Joonyoung Lee, 0fb1d8 [at] gmail [dot] com
|License=Proprietary :(
|URL=https://sites.google.com/site/arcrascent/home
|Status=Last release: 2015-07-29 - Inactive?
}}

= B =

= C =

{{OSProject
|OSName=CakeOS
|OSDescription=CakeOS (Cake) is a 32 bit operating system for x86 designed to be easy to use, with an interface that is both unique and yet instinctive. It is currently under development, with support for tasking, a dynamic heap, a 32bpp vesa/vga driver with mouse support, a basic shell and window manager, and several drivers in development. Cake has many ambitious aims for desktop usage.
|Contact=Luke Mitchell, lukem_95 [at] hotmail [dot] com
|License=Open source (Creative Commons)
|URL=http://code.google.com/p/cakeos
|Status=Last update: 2009-12-16, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=CapROS
|OSDescription=CapROS is a new operating system that merges some very old ideas about capabilities with some newer ideas about performance and resource management. The result is a small, secure, real-time operating system that provides orthogonal persistence. Our homepage: http://www.capros.org/
|Contact=Charles Landau, charlie [at] charlielandau [dot] com
|License=Open source (GNU GPLv2)
|URL=https://sourceforge.net/projects/capros/
|Status=Last update: 2010-02-19, abandoned
|FloppyAvailable=Too fat for a floppy! >40 MB size
}}

{{OSProject
|OSName=Caracal
|OSDescription=Caracal started life as a hobby OS which was going to make it as far as a 'hello world' kernel. Since I got the OS Dev bug it has now grown to a multiboot compatible Protected/Long mode and ARM compatible OS with multitasking support. The ARM port is in its early infancy and is designed to test portability and to run on the Raspberry Pi. Since Caracal has started using GRUB2, CBoot, the bootloader has been abandoned. As with a lot of these hobby OSes, I have a lot less time to spend on Caracal than I would like! My homepage: http://www.ajsoft.co.uk/ . Currently the Caracal Kernel is early alpha. CBoot has been abandoned in favour of using GRUB2
|Contact=AJ at OSDev forum
|License=Proprietary :(
|URL=https://sourceforge.net/projects/caracal32/
|Status=Last update: 2007-09-25, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=Cédille
|OSDescription=Cédille is a microkernel that attempts to be just that - a kernel with barely anything in it so it doesn't screw up. It boots via Grub and is inspired by Mach and Linux.
|Contact=Lionel at OSDev forums
|License=Open source (MIT)
|URL=https://github.com/Lionel07/Cedille
|Status=Last commit: 2015-03-22, abandoned
}}

{{OSProject
|OSName=Clicker
|OSDescription=Exploring new ways in OS design. Clicker32 is a microkernel-based system for x86 hardware. It's based on modular architecture and already supports preemptive multithreading + user-level processes.  Our homepage: https://sourceforge.net/projects/clicker/
|Contact=Pype & the Clicker Development Team, pype_1999.geo [at] yahoo [dot] com
|License=Open source
|URL=https://sourceforge.net/projects/clicker/
|Status=Last release: 2006-01-20, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=Crocos
|OSDescription=Crocos is a small opensource UNIX-like kernel for x86/x86_64 systems (written in C), designed with simplicity in mind, for educational purposes. It is developed in several steps to allow people to understand how a tiny operating system can be built from scratch. The main idea used in early development phases is to run a multiprocesses environment inside one Linux process. This way, it is possible to implement a maximum of features with the comfort of Linux for debugging and tests. Crocos is currently a multitasks system embedded inside one Linux process. It supports read operations on an ext2 file system.
|Contact=Guillaume Duranceau, g_duranceau [at] users [dot] sourceforge [dot] net
|License=Open source (GNU GPLv2)
|URL=http://crocos.sourceforge.net/
|Status=Last update: 2009-10-15, abandoned
|FloppyAvailable=Too fat for a floppy! 10 MB size
}}

= D =

{{OSProject
|OSName=d264b
|OSDescription=A rewrite of Dreckig OS in 64-bit assembly language. Uses the same principles and technology from Dreckig OS including the megalithic kernel architecture. Currently in early stages of development
|Contact=https://twitter.com/wizardofhaas
|License=Open source
|URL=https://github.com/WizardOfHaas/d264b
|Status=Last commit: 2014-12-25, abandoned
|FloppyAvailable=Could fit on a floppy, but no-one has tried :P
}}

{{OSProject
|OSName=Derrick Operating System
|OSDescription=A monolithic 32-bit operating system kernel designed for stability and speed. Derrick is running in protected mode. It's coded in x86 assembly and it's aimed as a single-tasking hybrid monolithic OS.
|Contact=Inflater, curko [at] centrum [dot] sk
|License=Proprietary :(
|URL=http://derrick.xf.cz
|Status=Last release: 2008-12-28, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=DEX Extensible operating system (dex-os)
|OSDescription=dex-os is an educational operating system that supports a plug-in like architecture and aims to achieve zero static inter-module calls
|Contact=Joseph Emmanuel Dayo, j_dayo [at] yahoo [dot] com
|License=Open source
|URL=http://sourceforge.net/projects/dex-os/
|Status=Last release: 2007-04-15, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=DexOS
|OSDescription=DexOS is a 32bit OS coded in 100% ASM by Craig Bamford, based on the idea that it would be cool to have a OS based on a console type OS, but - instead of running on a XBox or PS - it would run on x86 PC.  From the start, as you would expect from a OS based on a game's console OS, optimizing for speed has been of paramount important in the over all design. To this end there's no virtual memory, paging, and only a single process is allowed (though that process can spawn multiple threads). The program runs in ring0, you have direct access to all hardware (including CPU and graphics). Memory allocation is the responsibility of the app -- there's no front-end memory allocation. The entire OS will fit into less than 100k. My homepage: http://dex-os.github.io/
|Contact=Team Dex4u, justdex4u [at] aol [dot] com
|License=Open source (DexOS license)
|URL=https://github.com/dex-os/DexOS
|Status=Last commit: 2015-01-22, abandoned
}}

{{OSProject
|OSName=DreckigOS
|OSDescription=Dreckig OS is an x86, real mode, multi-tasking(kinda) operating system written in assembly language. It uses a megalithic kernel, an experimental kernel architecture. Dreckig also has a GUI and a custom RAM disk file system. I am also developing a programming language for use with Dreckig OS.
|Contact=https://twitter.com/wizardofhaas
|License=Open source
|URL=https://github.com/WizardOfHaas/DreckigOS
|Status=Last commit: 2015-06-20
}}

{{OSProject
|OSName=Drex
|OSDescription=It's a kernel for the IA32 architecture written in C and asm. It's based on the exokernel architecture but forks from it when simplicity can be gained.
|Contact=Gianluca Guida, gianluca [at] drex [dot] tk
|License=Open source (GNU GPLv2)
|URL=http://drex.sourceforge.net
|Status=Last release: 2005-12-07, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=DSD
|OSDescription=DSD is based on Jimix OS, which is an x86(_64) OS based on a microkernel architecture. It is written in C++ and all IPC methods are wrapped over a Remote Method Invocation paradigm
|Contact=https://github.com/dwnoble/dsd
|License=Open source
|URL=https://github.com/dwnoble/dsd
|Status=Last release: 2009-02-21, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=Dynacube Operating System
|OSDescription=A Full fledged open source, x86 based, 32bit Protected mode, multitasking operating system. Check it out.
|Contact=Gandalf, mdshah82 [at] yahoo [dot] com
|License=Open source
|URL=http://www.dynacube.net/
|Status=Last release: 2006-09-19
|FloppyAvailable=Could fit on a floppy, but no-one has tried :P
}}

= E =

= F =

{{OSProject
|OSName=FlachsOS
|OSDescription=FlachsOS is a 32-Bit Realtime Operating System.
|Contact=Daniel Weiler (Xardas) and Michael Wangler (Wacky), michael.wangler [at] web [dot] de
|License=Open source
|URL=https://sourceforge.net/projects/flachsos/
|Status=Last commit: 2004, is abandoned
|FloppyAvailable=Could fit on a floppy, but no-one has tried :P
}}

{{OSProject
|OSName=FreeNOS
|OSDescription=FreeNOS (Free Niek's Operating System) is an experimental microkernel operating system for learning purposes
|Contact=Niek Linnenbank, nieklinnenbank [at] gmail [dot] com
|License=Open source (GNU GPLv3)
|URL=http://www.FreeNOS.org
|Status=Last commit: 2009-08-02, is abandoned
|FloppyAvailable=Could fit on a floppy, but no-one has tried :P
}}

= G =

{{OSProject
|OSName=GeekOS
|OSDescription=Earlier versions of GeekOS have been used as the basis for student projects in operating system courses. This new development version of GeekOS is a rewrite designed to address limitations in the original version. It is not specifically designed for course projects, but because it strives for simplicity, it might be useful for people interested in learning about OS kernel implementation. Currently it builds on x86 and runs under Bochs and Qemu
|Contact=david [dot] hovemeyer [at] gmail [dot] com
|License=Open source (GNU GPLv2)
|URL=https://github.com/NikSteel/GeekOS
|Status=Last commit: 2015-07-22, is abandoned
}}

{{OSProject
|OSName=Gemini
|OSDescription=The goal is to create a production quality micro kernel based on the latest findings in operating system research. The result will be a micro-kernel having the following features: no abstractions, safe those necessary for protection, minimal set of primitives, support for capability based security, support for resource management needed for real-time and multimedia applications
|Contact=https://sourceforge.net/u/guidoj/profile/
|License=Open source (GNU GPLv2)
|URL=http://gemini.sourceforge.net/
|Status=Last release: 2001-02-17, is abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=Genius HorizonOS
|OSDescription=This OS is made by a schoolboy (he was born in 1996) and is a small 32-bit, closed source OS but you still can ask the developer about how did he do that to help you to do the thing that you asking about. It supports FAT12, floppy disks, memory management, keyboard, and it will support graphics soon.
|Contact=Mohamed Elghamry , mohamed_salah1996 [at] yahoo [dot] com , Mezo40 at OSDev forums
|License=Proprietary :(
|URL=http://mohamedsalah96.web.officelive.com/GHOS.aspx
|Status=Last release: 2012-06-17, is abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

= H =

= I =

{{OSProject
|OSName=IanOS
|OSDescription=IanOS is a small OS written for the x86_64 processors. The main goal of this OS is to be a system that is simple enough for beginners to understand yet sufficiently complex to demonstrate principles of OS design. It is not intended, at this stage anyway, to be a production OS.  IanOS is written in assembly language and C. Full source code, documentation, and build instructions are available. There is a blog where design principles and changes are discussed.
|Contact=Ian Jack, ian [at] ijack [dot] org [dot] uk
|License=Open source
|URL=http://www.ijack.org.uk
|Status=Last release: 2012-04-24, is abandoned
|FloppyAvailable=Could fit on a floppy, but no-one has tried :P
}}

{{OSProject
|OSName=IBOX
|OSDescription=IBOX is a "modular" operating system for the x86 desktop. The main goals of this os are: Portability, Stability, Security, and Customizable.  It will be written in assembly language and C. I suggest you take a look for yourself
|Contact=William McCabe, firstjourney_os [at] yahoo [dot] com
|License=Open source (OSL 3.0)
|URL=http://sourceforge.net/projects/ibox
|Status=Last release: 2004-06-06, is abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=Idylla OS
|OSDescription=Idylla OS is small and simple operating system. It will be written in assembly language and C. The main goals: Portability, Stability, Security and Customizable.
|Contact=milyges, milyges [at] gmail [dot] com,  milyges at OSDev forums
|License=Open source (GNU GPLv2)
|URL=https://sourceforge.net/projects/idyllaos/
|Status=Last release: 2008-07-25, is abandoned
|FloppyAvailable=Too fat for a floppy! >40 MB size
}}

{{OSProject
|OSName=ISOS
|OSDescription=ISOS is a very simple multithreaded OS for the Evaluator-7T board from ARM. It is based on JayOS also listed on this page. It features pre-emptive multi-threading, communication between threads, etc. and hardware drivers.
|Contact=Wilhem Meignan, w.m [at] wanadoo [dot] fr
|License=Open source
|URL=http://wilhem.meignan.free.fr/
|Status=Last release: 2003-05-02, is abandoned
|FloppyAvailable=Could fit on a floppy, but no-one has tried :P
}}

= J =
{{OSProject
|OSName=JaysOS
|OSDescription=JaysOS is a "toy OS" that runs on the Gameboy Advance. It provides preemptive threads, semaphores,  condition variables, and message queues.
|Contact=Justin Armstrong, ja [at] badpint [dot] org
|License=Proprietary :(
|URL=http://badpint.org/jaysos/
|Status=Last release: 2002-09-19, abandoned
|FloppyAvailable=Could fit on a floppy, but no-one has tried :P
}}

{{OSProject
|OSName=JNode
|OSDescription=JNode (Java New Operating System Design Effort) is an open source java OS written fully in java (with a very small assembler nano-kernel).
|Contact=Martin Husted Hartvig , hagar [at] jnode [dot] org
|License=Open source
|URL=http://www.jnode.org
|Status=Last release: 2009-01-29, abandoned
|FloppyAvailable=Too fat for a floppy! >230 MB size
}}

{{OSProject
|OSName=JonOS
|OSDescription=JonOS is a 16-bit operating system made entirely in asm, released under the GNU General Public License. It can't do much yet, but it has a cool prompt and a nice clock ;)
|Contact=Wojciech Komorowski aka GigaWolf, gigawolf [at] hotmail [dot] com
|License=Open source (GNU GPLv2)
|URL=http://qrf.servequake.com/JonOS/
|Status=Last release: 2005-04-01, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=JS-OS
|OSDescription=JS-OS is a 32-bit operating system, written mostly in C, that is aiming to become a UNIX-like clone, but with some added/better capabilities. This project was started in late 2012 by a high school student and is meant to be a learning/research tool for both the developer and the user. The code is mostly commented and not too difficult to follow. Using this operating system is meant to be easy and very intuitive. It is currently under development and supports preemptive multitasking, interrupts, an ext-2 clone file system (supporting up to 16GB files), a bunch of file system utilities, a basic shell, a dynamic heap, memory protection, a 32bpp VESA VBE and a VGA driver, a window manager, keyboard and mouse driver, and a sound driver.
|Contact=John Smith, jsmithcontact [at] tormail [dot] org
|License=Open source (GNU GPLv3)
|URL=https://github.com/JSmith-BitFlipper/JS-OS
|Status=Last commit: 2013-12-25, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

= K =

{{OSProject
|OSName=Kiddie OS
|OSDescription=Research operating system of mobile autonomous robots. It is suggested the system will act as the common intelligence for different types of machines including autonomous and humanoid robots able to collaborate with each other. It utilizes an idea and concept of self-learning adaptive operating system. Started in C, later rewritten in C++. Has bootable microkernel, isr's, vm, simple scheduler, driver interface, stub drivers, pci enumerator, tiny jvm and experimental pure java drivers. Supports both x86-32 and ARM architectures. There are plans to add multicore support and native gpgpu computing support to run parts of OS on it.
|Contact=Artur Emagulov, artur.a.e [at] gmail [dot] com
|License=Open source (BSD)
|URL=http://code.google.com/p/kiddie/
|Status=Last commit: 2013-06-13, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=Kosar OS
|OSDescription=Sample x86 OS with farsi support. Non-english homepage.
|Contact=http://kosaros.blog.ir/
|License=Proprietary :(
|URL=http://kosaros.blog.ir/
|Status=Last release: 2015-10-12, abandoned
}}

{{OSProject
|OSName=Kryos
|OSDescription=Kryos is an open source hobby/developer Operating System. Kryos has a basic command line interface (cli), multitasking, drivers, debugging and basic executable loading, not to mention a lot of other features
|Contact=Michael Lazear, onyxkernel [at] gmail [dot] com
|License=Open source (BSD)
|URL=https://code.google.com/archive/p/onyxkernel/
|Status=Last release: 2009-06-11, abandoned
|FloppyAvailable=Could fit on a floppy, but no-one has tried :P
}}

= L =

{{OSProject
|OSName=L4ka
|OSDescription=L4ka project has two extremely fast microkernels: Pistachio and Hazelnut. It is available for x86, ARM, MIPS and other architectures. Our homepage: http://l4ka.org/
|Contact=University of Karlsruhe, Germany, l4ka [at] ira [dot] uka [dot] de
|License=Open source
|URL=https://github.com/l4ka
|Status=Last commit: 2015-09-01, abandoned
}}

{{OSProject
|OSName=Leviathan
|OSDescription=Leviathan is a micro/modular 64 bit kernel that aims to be used by the general public. It includes
support for my rwfs filesystem which can handle 500 million exabytes of storage.
|Contact=Kristian Hart, nekros729 [at] gmail [dot] com
|License=Open source (GNU GPLv3)
|URL=https://code.google.com/archive/p/leviathanv/
|Status=Last commit: 2009-12-12, abandoned
|FloppyAvailable=Could fit on a floppy, but no-one has tried :P
}}

{{OSProject
|OSName=Lily
|OSDescription=Lily is a microkernel based on the I/O automata formal model designed to support the development of reactive programs.  Device drivers and programs are expressed as automata that can be composed at run-time. x86(32-bit)/C++/C
|Contact=Justin Wilson, jrwilson [at] go [dot] wustl [dot] edu
|License=Open source
|URL=http://github.com/jrwilson/Lily
|Status=Last commit: 2012-05-14, abandoned
|FloppyAvailable=Could fit on a floppy, but no-one has tried :P
}}

{{OSProject
|OSName=LUnix
|OSDescription=LNG is an operationg system primarly for the good old Commodore64 home-computer. There also is a native version for the successor Commodore 128. Ports to other 6502/6510 driven 8-Bit Computers are possible but not yet started. LUnix started in 1993 and reached the internet in 1994. In 1997 LUnix 0.1 was rewritten from scratch, the result is LNG. Our homepage: http://lng.sourceforge.net/
|Contact=Daniel Dallman, Dallmann [at] heilbronn [dot] netsurf [dot] de
|License=Open source (GNU GPLv2)
|URL=https://sourceforge.net/projects/lng/
|Status=Last release: 2004-09-10, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

= M =

{{OSProject
|OSName=M3
|OSDescription=M3 is an operating system targeting the IA32 architecture. It is currently under development. The goal for this project is to learn the ins and outs of OS development, and share this knowledge by writing tutorials and well-commented code. My homepage: http://m3os.wordpress.com
|Contact=Peter de Tagyos, peter [at] ptss [dot] net
|License=Open source (GNU GPLv3)
|URL=https://code.google.com/archive/p/m3os
|Status=Last commit: 2009-04-13, abandoned
|FloppyAvailable=Could fit on a floppy, but no-one has tried :P
}}

{{OSProject
|OSName=Mammoth OS
|OSDescription=Mammoth OS is a compact, lightweight kernel designed to have a monolithic core, and a modular kernel extension system. We hope to have a full implementation of a C compiler toolchain, and other languages such as C++ and Assembly by August 2009. Still very basic pre-alpha. Implemented proper I/O functions. Next release: Advanced Output, Memory Manager
|Contact=callum [at] rohedin [dot] co [dot] uk
|License=Open source (GNU GPLv2)
|URL=http://launchpad.net/mammoth-os
|Status=Last release: 2009-02-18, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=ManRiX
|OSDescription=ManRiX OS is open source microkernel based operating system with POSIX compliance.ManRiX OS is written totally from scratch using C and Assembly language.
|Contact=Manish Regmi, regmi_manish [at] gmail [dot] com ; Rajesh Bikram R.C., rajesh.rc [at] gmail [dot] com
|License=Open source (GNU GPLv2)
|URL=http://manrix.sourceforge.net/
|Status=Last commit: 2008-02-06, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=Mattise
|OSDescription=Mattise is a very simple hobby operating system for x86 architectures. It is a monolithic kernel written completely in C and Assembly with paging, ELF loading (modules and executables) and a working Newlib port. It has a rudimentary shell and a working NASM and Binutils port. Basic C-only version of the kernel, without paging or newlib; basic binary file loader
|Contact=Matthew Iselin, pcmattman [at] users.sourceforge.net
|License=Open source (GNU GPLv2)
|URL=http://mattise.sourceforge.net/
|Status=Last release: 2007-04-01, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=MazzanetOS
|OSDescription=MazzanetOS is a DOS/UNIX-like operating system written in assembly language and C++. It currently runs off a floppy disk and includes a graphics demo. It will run (hopefully) on any PC.
|Contact=mazzanet, mazzanet [at] hotmail [dot] com
|License=Open source (GNU GPLv2)
|URL=http://mazzanetos.sourceforge.net
|Status=Last commit: 2003-01-17, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=Melon
|OSDescription=Melon is a C++ operating system designed to be simple and fast and to provide a coherent OO framework for developing applications.
|Contact=Alexis211, alexis211 [at] gmail [dot] com
|License=Open source (Public Domain)
|URL=http://github.com/Alexis211/Melon
|Status=Last commit: 2014-10-28, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=Minirighi
|OSDescription=A didactic Linux-like kernel.
|Contact=Andrea Righi, righiandr [at] users [dot] sourceforge [dot] net
|License=Open source (GNU GPLv2)
|URL=http://minirighi.sf.net
|Status=Last release: 2008-04-04, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=Mojo OS
|OSDescription=Mojo OS aims to be a modern desktop operating system. Ity is currently under active development but far from finished. Read much more on the homepage.
|Contact=Christian Lange, clange [at] softwarewizard [dot] dk
|License=Open source (GNU GPLv3)
|URL=https://github.com/Fedjmike/Mojo
|Status=Last release: 2013-09-13, abandoned
|FloppyAvailable=Could fit on a floppy, but no-one has tried :P
}}

{{OSProject
|OSName=Mona OS
|OSDescription=Mona OS is a small, new and fast microkernel system written in C++ for Intel x86 architecture. It is neither a POSIX or Windows clone.
|Contact=Higepon, higepon [at] users [dot] sourceforge [dot] jp
|License=Open source (MIT)
|URL=http://www.monaos.org/
|Status=Last release: 2011-08-05, abandoned
|FloppyAvailable=Too fat for a floppy! >30 MB size
}}

{{OSProject
|OSName=MSB-OS
|OSDescription=An OS originally designed as a platform for my own A.I. work (linguistics), but soon to be used for education purposes as well. All programming of MSB-OS has been done directly in machine code (without using assembler).
|Contact=osproject [at] magicschoolbook [dot] com , DavidCooper at OSDev forums
|License=Open source (which is a machine code)
|URL=http://www.magicschoolbook.com/computing/os-project
|Status=Last release: 2011-06-27, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=MyXomycota
|OSDescription=MyXomycota is a monolithic system running in protected mode and using paging. It is written in C, newlib (a small stdlibc) is partly ported. It runs from a floppy disk (loaded with BIOS interrupts on startup, hence even USB floppy disk drives are supported). I am sorry but many texts are German. Kernel with programs running in user mode (simple shell, brainfuck compiler, simple editor (cedit), standard programs (ls, cp, rm, cd, echo, file, top, mount) and some ported programs (aargh interpreter, flat assembler 1.67.26, ed))
|Contact=Max Reitz, xanclic [at] googlemail [dot] com
|License=Open source (GNU GPLv2)
|URL=http://sourceforge.net/projects/myxomycota
|Status=Last commit: 2009-06-26, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

= N =

{{OSProject
|OSName=Nano Os
|OSDescription=Nano Os is a Hobby/Educational Operating System Project.
It is a monolithic kernel OS for PC and compatibles with 386 and above. It is written in g++ and assembly (NASM)
|Contact=Ashok Shankar Das, ashok [dot] s [dot] das [at] gmail [dot] com
|License=Open source (GNU GPLv2)
|URL=http://code.google.com/p/nanoos/
|Status=Last commit: 2014-02-21, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=Nanos
|OSDescription=Nanos is a protected mode nanokernel OS for x86 computers. It is written in assembly and assembled with NASM. No further work will be done after the bootable kernel achievement. My homepage: https://silentorbit.com/nanos/
|Contact=Peter Hultqvist, nanos [at] silentorbit [dot] com
|License=Open source (GNU GPLv3)
|URL=https://github.com/nwg/nanos
|Status=Last commit: 2015-02-17, abandoned
}}

{{OSProject
|OSName=NDK
|OSDescription=NDK is neuraldk's ongoing experimentation at creating a new, modern operating system. Currently in its infancy, it's of little use to anyone but the beginning operating systems developer.
|Contact=, carbonBased [at] neuraldk [dot] org
|License=Open source
|URL=http://www.neuraldk.org/product-NDK
|Status=Last release: 2006-03-23, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=Neptune Operating System
|OSDescription=Neptune is a self hosting 32 bit highly modular graphical multitasking operating system written in C and some assembly. Old website - http://www.brokenthorn.com/mos , new website - http://www.brokenthorn.com/mos/site2/
|Contact=http://www.brokenthorn.com/forums/index.php
|License=Proprietary :( (author promises that it will become "shared source" for educational use only)
|URL=http://www.brokenthorn.com/mos/site2/
|Status=Last release: 2009-06-14, but is not abandoned
|FloppyAvailable=Could fit on a floppy, but no-one has tried :P
}}

= O =

{{OSProject
|OSName=ominOS
|OSDescription=ominOS is a small lightweight kernel that aims to be simple to and have easy to understand source code to learn from. Its definitely not doing anything revolutionary, but I love working on it and seeing where it goes. Currently the os has ports of NASM, Binutils, and some graphic libraries. Also has a partially implemented TCP/IP stack. My homepage: http://ominos.sourceforge.net
|Contact=Anthony Lineberry, anthony [dot] lineberry [at] gmail [dot] com
|License=Open source (GNU GPLv2)
|URL=https://sourceforge.net/projects/ominos/
|Status=Last commit: 2008-02-24, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=Onem OS
|OSDescription=Onem OS is an x86-16 assembly kernel. Its set of features is very limited, however it might be interesting for beginners
|Contact=https://github.com/primis
|License=Open source
|URL=https://github.com/primis/onem16
|Status=Last commit: 2012-05-05, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=Orange OS
|OSDescription=The Orange OS Project started with the intent to design a lightweight Operating System. It contains a small kernel designed for those interested in the low-level, practical aspects of building an x86 system. Currently, it is designed to fit on a floppy
|Contact=Berlin Brown, bigbinc [at] hotmail [dot] com
|License=Open source (GNU GPLv2)
|URL=http://orangelin.sourceforge.net/
|Status=Last commit: 2008-03-28, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=OSSO
|OSDescription=A Micro Kernel Operating System written from scratch. My homepage: http://osso.sourceforge.net
|Contact=Alessandro Iurlano, iurlano [at] users [dot] sourceforge [dot] net
|License=Open source (GNU GPLv2)
|URL=https://sourceforge.net/projects/osso/
|Status=Last update: 2009-01-12, abandoned
|FloppyAvailable=Could fit on a floppy, but no-one has tried :P
}}

= P =

{{OSProject
|OSName=Pebble Operating System
|OSDescription=Pebble is a 32-bit operating sytem for IA-32 systems. It can run MS-DOS and DPMI compatible applications and will provide advanced features such as multitasking and paging. It will be a good platform to run old MS-DOS and embedded programs.
|Contact=Raghu Kaippully, kraghu [at] users [dot] sourceforge [dot] net
|License=Open source (GNU GPLv3)
|URL=http://sourceforge.net/projects/pebble-os/
|Status=Last commit: 2012-02-02, abandoned
|FloppyAvailable=Could fit on a floppy, but no-one has tried :P
}}

{{OSProject
|OSName=Plop
|OSDescription=Plop is a tiny OS that serves as a bootloader for other operating systems. Sadly its proprietary, however it is still very useful and fits on a floppy
|Contact=Elmar Hanlhofer
|License=Proprietary :(
|URL=https://www.plop.at/en/bootmanager/download.html
|Status=Last release: 2013-04-15, but is not abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=Pocket OS
|OSDescription=Pocket OS is an operating system in the spirit of DOS.
|Contact=pierrel5 [at] free [dot] fr
|License=Open source
|URL=http://pierrel5.free.fr/PocketOS/eindex.htm
|Status=Last release: 2013-02-19, abandoned
|FloppyAvailable=Too fat for a floppy! >15 MB size
}}

{{OSProject
|OSName=Polix32
|OSDescription=Polix is a 32-bit  operating system for x86,  based on Linux kernel.
|Contact=michael, uzm386 [at] gmail [dot] com
|License=Open source (GNU GPLv2)
|URL=http://uzm386.awardspace.us/polix32.html
|Status=Last release: 2010-04-24, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=POSYS
|OSDescription=POSYS: Programmers Operating SYStem Small, fast, dynamic, secure. (In development) Language: NASM + CC386.  Useful links at the homepage - http://posysos.sourceforge.net/
|Contact=Chris Pop, popcristian [at] users [dot] sourceforge [dot] net
|License=Open source (GNU GPLv2)
|URL=http://www.sourceforge.net/projects/polysos/
|Status=Last release: 2003-01-09, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=Prex
|OSDescription=The Prex project is an open source initiative to provide a portable real-time operating system for embedded systems. My homepage: http://prex.sourceforge.net/
|Contact=Kohsuke Ohtani, kohtani [at] users [dot] sourceforge [dot] net
|License=Open source (BSD)
|URL=http://www.sourceforge.net/projects/prex/
|Status=Last commit: 2009-10-01
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=PuddingOS - PNL/OS
|OSDescription=A small, fast and complete Operating System in 32 bits
|Contact=David Delassus, linkdd62 [at] gmail [dot] com
|License=Open source (BSD)
|URL=http://codingteam.net/project/puddingos
|Status=Last commit: 2008-08-05, abandoned
|FloppyAvailable=Could fit on a floppy, but no-one has tried :P
}}

= Q =

= R =

{{OSProject
|OSName=RhinOS
|OSDescription=RhinOS is a french 32bits x86 micro-kernel like OS. At the time of writing, it has its own bootloader, a physical memory manager (buddy system), a virtual memory manager for the kernel (slab allocator and buddy system), a minimal thread management and IPC via message passing; also has the thread management and IPC
|Contact=Gab, gb.viel [at] gmail [dot] com
|License=Open source (GNU GPLv3)
|URL=http://code.google.com/p/rhinos/
|Status=Last update: 2013-09-27, abandoned
|FloppyAvailable=Could fit on a floppy, but no-one has tried :P
}}

{{OSProject
|OSName=Rhombus
|OSDescription=Rhombus is a microkernel-based operating system written in C for protected mode x86. It currently contains a kernel, drivers, shell utilities, and a C library, all written from scratch. It has a distributed VFS mechanism, a human-readable/writable remote procedure call protocol, and some other neat features. It is roughly UNIX-like, but with some major architectural differences. It is distributed under an OpenBSD-style license. It has been under pretty active development for the last three years, and is nearing a public alpha/beta release.
|Contact=Nick Johnson, nickbjohnson4224 [at] gmail [dot] com
|License=Open source (ISC)
|URL=http://github.com/nickbjohnson4224/rhombus/
|Status=Last update: 2012-02-13, abandoned
|FloppyAvailable=Perhaps is too fat to fit on a floppy, and no-one has tried ;)
}}

{{OSProject
|OSName=RYMOS
|OSDescription=Graphic os under construction maked with NASM and GCC
|Contact=Roberto Rodriguez, prfiredragon [at] gmail [dot] com
|License=Open source (GNU GPLv2)
|URL=https://sourceforge.net/projects/rymos/
|Status=Last update: 2012-02-23, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

= S =

{{OSProject
|OSName=sanos
|OSDescription=A small 32-bit x86 operating system kernel for jbox appliances. A jbox is a JavaOS server appliance running on standard PC hardware. This enables you to run java server applications without the need to install a host operating system. Only a standard Java HotSpot VM and the sanos kernel are needed.
|Contact=Michael Ringgaard, mri [at] jbox [dot] dk
|License=Open source (BSD)
|URL=http://www.jbox.dk
|Status=Last release: 2012-03-08, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=Sartoris Microkernel
|OSDescription=The Sartoris Project aim is to develop a portable microkernel and a set of operating system services that support:   -Efficient implementation of local system calls  - Concurrent execution of several OS 'personalities', ie a UNIX environment and a native microkernel-based interface  - Simple and elegant integration of distributed operating system components. My homepage: http://sartoris.sourceforge.net/
|Contact=Nicodega, nicoba4 [at] hotmail [dot] com
|License=Open source (BSD)
|URL=https://sourceforge.net/projects/sartoris/
|Status=Last release: 2005-09-18, abandoned
|FloppyAvailable=Could fit on a floppy, but no-one has tried :P
}}

{{OSProject
|OSName=ScorchOS
|OSDescription=A 32-bit pmode OS aimed to be light-weight, fast, well-documented and easy to develop.
|Contact=Bob Moss, bobbyjmoss [at] gmail [dot] com
|License=Open source (BSD)
|URL=https://sourceforge.net/projects/scorchos
|Status=Last release: 2012-12-09, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=SharpOS
|OSDescription=SharpOS is the original open source effort to write an operating system in 100% C#, with a strong focus on security, reliability, and maintainability. It has a wiki.
|Contact=https://sourceforge.net/p/sharpos/mailman/sharpos-developers/?viewmonth=201011
|License=Open source
|URL=https://sourceforge.net/projects/sharpos/
|Status=Last commit: 2009-01-26, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=smiddyOS
|OSDescription=smiddyOS is an operating system that is written in FASM asm. It is mainly a learning environment for the author, with pretty loose goals.
|Contact=smiddy on BOS forum, Dex OS forum, smiddy at OSDev forums
|License=Proprietary :(
|URL=http://smiddyOS.asmhackers.net/
|Status=Last release: 2006-11-29, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=Solar_OS
|OSDescription=Simple, easy, fast 32bit GUI OS in ASM
|Contact=bogdanontanu [at] yahoo [dot] com
|License=Proprietary :(
|URL=http://www.oby.ro/os/
|Status=Last release: 2009-07-22, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

{{OSProject
|OSName=SollerOS
|OSDescription=A unix-based 32 bit operating system written in assembly-it has no connection to Solar_OS, the similar names are just a coincidence
|Contact=Jeremy Soller, jackpot51 [at] gmail [dot] com
|License=Open source
|URL=http://solleros.googlecode.com
|Status=Last release: 2012-08-17, abandoned
|FloppyAvailable=Could fit on a floppy, but no-one has tried :P
}}

{{OSProject
|OSName=Sugatha RTOS kernel
|OSDescription=Sugatha is an RTOS kernel with upto 8 priority levels. It is mainly written in C with the CPU specific code in asm. My homepage: http://sugatha.sourceforge.net
|Contact=dushara, nidujay [at] sourceforge [dot] net
|License=Open source (BSD)
|URL=https://sourceforge.net/projects/sugatha/
|Status=Last release: 2007-06-28, abandoned
|FloppyAvailable=Could fit on a floppy, but no-one has tried :P
}}

= T =

{{OSProject
|OSName=TempOS
|OSDescription=TempOS is an educational and multi purpose Operating System that was born as an undergraduate work and has it's your main goal to be not only another Operating System, but a complete toolkit designed to help students on their O.S. courses. My homepage: http://tempos-project.org
|Contact=Renê S. Pinto, rene [at] renesp [dot] com [dot] br
|License=Open source (GNU GPLv2)
|URL=https://sourceforge.net/p/tempos-project/
|Status=Last update: 2012-04-15, abandoned
|FloppyAvailable=Too fat for a floppy! 10 MB size
}}

{{OSProject
|OSName=The Damn Small OS
|OSDescription=An Exo-kernel where the application builds the system it self, providing the most basical functions for a multi-tasking system. All the response is thrown to the application, let it manage, but protecting the resources. An extreme approach of Operating Systems, that should be highly portable. 100% in C.
|Contact=the team, alphakiller_ [at] msn [dot] com
|License=Open source
|URL=http://sourceforge.net/projects/comos
|Status=Last update: 2006-01-14, abandoned
|FloppyAvailable=Too fat for a floppy! 11 MB size
}}

{{OSProject
|OSName=Trion
|OSDescription=A portable nanokernel-based OS. Our homepage: http://trion.sourceforge.net
|Contact=Trion Developers Mailing List, trion-kernel-dev [at] lists [dot] sourceforge [dot] net
|License=Open source (BSD)
|URL=http://www.sourceforge.net/projects/trion/
|Status=Last commit: 2006-04-10, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

= U =

{{OSProject
|OSName=udos: a micro disk operating system
|OSDescription=udos is for PC/AT compatibles with legacy FDD
|Contact=Katsuhiko Gondow, gondow [at] cs.titech.ac.jp
|License=Open source (GNU GPLv2)
|URL=http://www.sde.cs.titech.ac.jp/~gondow/udos/
|Status=Last release: 2009-01-20, abandoned
|FloppyAvailable=Fits on a floppy! :)
}}

= V =

{{OSProject
|OSName=Valix
|OSDescription=
The Valix Operating System is a managed-code operating system. Valix runs no userland binaries: instead, an object-oriented interpreter is built into the kernel.
This offers superior security (the only binary the CPU is directly running is the kernel itself) and faster speeds compared to other interpreters,
since time is not wasted with context switching between kernelmode and usermode. Valix itself is written in GCC C, C++, and FASM Assembly under GPLv3;
source code available at http://gitorious.com/valix . x86 architecture. My homepage: https://sites.google.com/site/valixsys/
|Contact=xvedejas and cfaust at #valix on irc.freenode.net, or - xvedejas [at] gmail [dot] com
|License=Open source (GNU GPLv3)
|URL=https://github.com/xvedejas/Valix/
|Status=Last commit: 2014-08-25, abandoned
|FloppyAvailable=Perhaps is too fat to fit on a floppy, and no-one has tried ;)
}}

= W =

{{OSProject
|OSName=WISDOM
|OSDescription=WISDOM is a 32-bit operating system with a primitive shell implemented. WISDOM is also a research into the field of design of operating system
|Contact=R Karthick and Anita Shelton, rkarthick [at] gmx [dot] net
|License=Open source (GNU GPLv2)
|URL=http://wisdom.sourceforge.net
|Status=Last update: 2003-10-22, abandoned
|FloppyAvailable=Could fit on a floppy, but no-one has tried :P
|CloserInspection={{CloserInspectionValidityNew|The source code contains folder.htt files which are infected with the Redlof virus}}
}}

= X =

{{OSProject
|OSName=XOmB
|OSDescription=General purpose OS built on top of the XOmB exokernel. Aims to do away with the legacy crap of x86 and utilize the features of x86_64 to their benefit. Devices as given to the user with the most minimal of abstractions. The kernel itself is multicore 64 bit only. It supports multicore scheduling, a userspace keyboard driver, a userspace VESA driver (through x86 emulation), and is actively developed by a group of undergraduate and graduate students.
|Contact=wolfwood or untwisted or wilkie or steveklabnik at the OSDev forums
|License=Open source
|URL=https://github.com/xomboverlord/xomb/
|Status=Last update: 2013-05-31, is active
|FloppyAvailable=Perhaps is too fat to fit on a floppy, and no-one has tried ;)
}}

= Y =

= Z =

= 9 =

[[Category:Resources]]
````

<a id="w004234"></a>

## W004234 — File:9front.png

Bron-ID: W004234
Namespace: File (6)
Revisie: 22027 | Datum: 2018-01-04T09:58:03Z
Bron: https://wiki.osdev.org/File:9front.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3A9front.png&oldid=22027

````text
9front screenshot
````

<a id="w004235"></a>

## W004235 — File:FreeDOS.png

Bron-ID: W004235
Namespace: File (6)
Revisie: 22028 | Datum: 2018-01-04T09:58:56Z
Bron: https://wiki.osdev.org/File:FreeDOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AFreeDOS.png&oldid=22028

````text
FreeDOS screenshot
````

<a id="w004236"></a>

## W004236 — File:KolibriOS.png

Bron-ID: W004236
Namespace: File (6)
Revisie: 22029 | Datum: 2018-01-04T09:59:52Z
Bron: https://wiki.osdev.org/File:KolibriOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AKolibriOS.png&oldid=22029

````text
KolibriOS screenshot
````

<a id="w004237"></a>

## W004237 — File:MikeOS.png

Bron-ID: W004237
Namespace: File (6)
Revisie: 22030 | Datum: 2018-01-04T10:00:31Z
Bron: https://wiki.osdev.org/File:MikeOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AMikeOS.png&oldid=22030

````text
MikeOS screenshot
````

<a id="w004238"></a>

## W004238 — File:PrettyOS.png

Bron-ID: W004238
Namespace: File (6)
Revisie: 22031 | Datum: 2018-01-04T10:02:15Z
Bron: https://wiki.osdev.org/File:PrettyOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3APrettyOS.png&oldid=22031

````text
PrettyOS screenshot
````

<a id="w004239"></a>

## W004239 — File:SnowdropOS.png

Bron-ID: W004239
Namespace: File (6)
Revisie: 22032 | Datum: 2018-01-04T10:02:47Z
Bron: https://wiki.osdev.org/File:SnowdropOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ASnowdropOS.png&oldid=22032

````text
SnowdropOS screenshot
````

<a id="w004240"></a>

## W004240 — File:Sortix.png

Bron-ID: W004240
Namespace: File (6)
Revisie: 22033 | Datum: 2018-01-04T10:03:12Z
Bron: https://wiki.osdev.org/File:Sortix.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ASortix.png&oldid=22033

````text
Sortix screenshot
````

<a id="w004241"></a>

## W004241 — File:TatOS.png

Bron-ID: W004241
Namespace: File (6)
Revisie: 22034 | Datum: 2018-01-04T10:03:46Z
Bron: https://wiki.osdev.org/File:TatOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ATatOS.png&oldid=22034

````text
tatOS screenshot
````

<a id="w004242"></a>

## W004242 — File:TempleOS.png

Bron-ID: W004242
Namespace: File (6)
Revisie: 22035 | Datum: 2018-01-04T10:04:22Z
Bron: https://wiki.osdev.org/File:TempleOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ATempleOS.png&oldid=22035

````text
TempleOS screenshot
````

<a id="w004243"></a>

## W004243 — File:VanadiumOS.png

Bron-ID: W004243
Namespace: File (6)
Revisie: 22036 | Datum: 2018-01-04T10:04:45Z
Bron: https://wiki.osdev.org/File:VanadiumOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AVanadiumOS.png&oldid=22036

````text
VanadiumOS screenshot
````

<a id="w004244"></a>

## W004244 — File:Visopsys.png

Bron-ID: W004244
Namespace: File (6)
Revisie: 22037 | Datum: 2018-01-04T10:05:51Z
Bron: https://wiki.osdev.org/File:Visopsys.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AVisopsys.png&oldid=22037

````text
Visopsys screenshot
````

<a id="w004245"></a>

## W004245 — Schol-R-LEA/Reduced State Vector Processor

Bron-ID: W004245
Namespace: Main (0)
Revisie: 22061 | Datum: 2018-01-12T02:23:32Z
Bron: https://wiki.osdev.org/Schol-R-LEA/Reduced_State_Vector_Processor
Revisie-URL: https://wiki.osdev.org/index.php?title=Schol-R-LEA%2FReduced_State_Vector_Processor&oldid=22061

````text
'''The Reduced State Vector Processor (RSVP)'''

''J Osako, 22 September 1998''

The current emphasis in CPU design on fast instruction performance is, in the opinion of this author, misplaced. The primary overhead in current software is in I/O and interrupt latency, not instruction execution; most systems today are bound to a slow turnaround in context switching, and are usually idle while waiting for user events. Further, most of the activity in modern systems is bound to processes that involve large vector and matrix calculation, particularly those needed for audio and video processing. While most such work is usually offloaded to DSPs and other coprocessors, some still must be handled by the CPU, and the CPU ought to be designed to handle such calculations efficiently.

To these ends, I propose a CPU design meant to minimze processor state, maximize parallel processing efficiency and enhance vector calculations. I call this the Reduced State Vector Processor (RSVP) concept.

The basic concept is to eliminate the majority of the registers in the CPU, replacing them with large (>1M) write-through caches. The processor would have separate caches for code, data and stack, and would have at least two seperate caches for each of these functions. The intention is that each process would be allocated a separate cache set, and that in the case of a cache miss, the CPU would automatically trigger a task switch to a process which is in another cache set while the missed cache is updated in background.

All operations would be direct-to-memory, with the ALU operating directly on the caches; in effect, the caches would act as an extremely large register set. This allows the CPU to be reduced to a set of four state registers : Instruction Pointer, Stack Pointer, Frame Pointer and Cache Pointer. Given this minimal state, and the fact that all operations are cached, allows for a single-cycle, single instruction context switch.

As said earlier, the actual calculations are performed directly upon the caches, the ALU has to be able to operate on any section of the cache as needed. Taking this idea further, it can be seen that, logically, the ALU should be bound to the cache it works on, not the CPU, leaving the CPU to perform only the control functions. ALU operations that require more than one cycle should trigger a context switch, just as a a cache miss would, allowing the ALU to operate independently. Further, since it can operate on arbitrary sections of the cache, it should be possible for it to operate on multiple operands, or operands of arbitrary size. This allows for efficient matrix and VLW operations as a natural extension of the ALU design. It should be able, in principle, to operate upon the entire addressable memory; the ALU would call cache refills and continue operating completely independent of the CPU itself.

The final logical step of this design is to add multiple CPUs. Since there are already redundant caches, and the CPU structure is so minimal, it should be possible to have several of these simplified CPUs on a single chip, switching between a large collection of semi-autonomous cache/ALU sets as needed. A reasonable plan, given current chip densities, is a 4 CPU, 8 Cache-set design, which would have a total of 24M of cache memory on-chip. While this is a very large amount of memory, it is well within current densities, and the simplified structure of the processor overall eliminates most of the processor hardware.
````

<a id="w004246"></a>

## W004246 — Reduced State Vector Processor

Bron-ID: W004246
Namespace: Main (0)
Revisie: 22062 | Datum: 2018-01-12T02:23:32Z
Bron: https://wiki.osdev.org/Reduced_State_Vector_Processor
Revisie-URL: https://wiki.osdev.org/index.php?title=Reduced_State_Vector_Processor&oldid=22062
Doorverwijzing: Schol-R-LEA/Reduced State Vector Processor

````text
#REDIRECT [[Schol-R-LEA/Reduced State Vector Processor]]
````

<a id="w004247"></a>

## W004247 — Thread

Bron-ID: W004247
Namespace: Main (0)
Revisie: 30651 | Datum: 2026-05-28T11:09:18Z
Bron: https://wiki.osdev.org/Thread
Revisie-URL: https://wiki.osdev.org/index.php?title=Thread&oldid=30651

````text
== Introduction ==

In operating systems, threads are the basic unit of execution and concurrency of various tasks pending in the system. They are generally internal components of processes and share the same set of resources, i.e. file handles, shared-memory handles, POSIX signals, message-passing buffers and many more. But they can have specific sets of special resources in architectures like Windows NT, where the kernel can destroy ''some resources corresponding to a thread when it terminates.''

Threads can see each other executing in a parallel fashion as they get selected by the scheduler, and then execute for a specific period of time. That means if the threads were designed to repeatedly print a specific number unique to each thread, you would see them being printed in a random fashion. But threads can concurrently execute in real-time on multi-processor systems only, because multiple CPUs are executing the tasks simultaneously as opposed to uni-processor systems where one CPU executes each task one-by-one.

== Advantages ==

All modern operating systems implement threads with deep support. They export kernel calls for configuring how the kernel manages various threads. Threads have many advantages over traditional processes and are used in newer applications:

1. '''Concurrency''' - Threads allow multiple tasks to execute in a parallel fashion and speed up the various work of the program. For example, two worker threads can be parsing two different CSS stylesheets in a browser while loading a page. This increases the output and responsiveness of the program.

2. '''Better Resource Usage''' - When work is divided into separate processes, a huge overhead is incurred as compared to dividing the work between threads that live in the same address space under one process. It also reduces context switches and can provide better cache usage for the program.

3. '''User-level Management of Threads''' - Threads can also be managed at a user-level which eliminates the need for a context switch to the kernel. This greatly improves performance with the the help of [[Multiprocessor Scheduling#Scheduler Activations|Scheduler Activations]] (see Scheduling) and the M:N threading model (which we will talk about later in this article).

4. '''Simplicity''' - Creating a single program which effectively utilizes a thread API, is very simple compared with creating separate work-processes and corresponding programs, using inter-process communication to inter-link them.

== Complexity of Threads ==

Multithreading in programs and the operating system itself can lead to the following undesirable conditions:

1. '''Race Conditions & Synchronization Overhead''' - When many threads are working on the same data structure in memory, then they must somehow serialize their changes with synchronization techniques. This greatly impacts performance when the number of competing threads is huge, especially when concurrency bugs cause deadlocks, livelocks or other race conditions. For example, if threads 'A' and 'B' require both the resources 'R1' and 'R2', and 'A' locks 'R1' first and 'B' locks 'R2' first. Now both are competing for the other resource, both of which are already locked. This results into a complete stoppage of execution and is called a deadlock.

2. '''Reliability''' - Multithreading is indeed complex compared to the single-threaded model. Also, if one thread throws an exception (not C++) which the system cannot handle then the whole process may be terminated.

== Kernel Threads ==
{{main|Kernel Multitasking}}

The task scheduler in the kernel handles kernel-level threads. These threads are held in a data structure by the kernel and are managed by the kernel. They can be blocked for I/O by the kernel and easily destroyed from external processes.

Upon entry into the kernel, the kernel stack for the thread is loaded and the user stack, along with its execution state, is saved. Each thread may have its own stack or share a set of stacks.

=== Multiple Kernel Stacks ===

The main advantage of each thread having its own kernel stack is that system calls can block inside of the kernel and easily resume later from that point. If a page fault or interrupt were to occur during a system call, then it's possible to switch to another context and back, and later complete the system call.
However, a large number of threads could tie up a significant amount of memory, most of which would sit unused at any given time. Further, the constant switching of kernel threads may lead to higher cache misses and thus poorer performance.

=== Single Kernel Stack ===

An alternative is to have one kernel stack per processor and let threads share the stack(s). This greatly reduces the amount of memory that must be allocated. The thread gets control of its processor's kernel stack when entering the kernel and relinquishes it upon a thread switch. Since the stack pointer is reset to the top, the stack is effectively destroyed and created anew. However, this means that threads cannot easily block or be preempted inside of the kernel. For example, a microkernel may need to send a message to a server that handles page faults, but upon the switch to the server thread, the stack of the faulting thread would be wiped out. The kernel must either provide a mechanism to restart interrupted system calls (using continuations, for example) or guarantee that system calls will not block, be preempted, or fault (any such condition would be fatal.)

=== Variable Kernel Stacks ===

A compromise between multiple stacks and a single stack is to use a single stack per processor for all threads, but allocate a new stack when a thread blocks or is preempted inside of the kernel. The blocked thread takes ownership of the old stack, and the other threads share the new one. When the thread is unblocked and completes its operation inside of the kernel, its stack is deallocated. This allows blocking inside of the kernel while minimizing memory usage. The main issue with this approach is that memory for stacks has to be allocated and deallocated every time threads are switched, possibly at a time when the kernel must handle an interrupt in a timely manner.

== User Threads ==

The user-level threads are held by the user-level scheduler and are not seen by the kernel. They can be created and destroyed in userspace and thus are of even lower overhead than kernel threads.

== Threading Models ==

Threads can be implemented in a system using these three methods:

=== '''1:1 Model''' ===

Each  that the process uses in user-space could be scheduled in the kernel. This means that the scheduler will actually hold structures about all threads that the process is using.

==== Advantages ====

When all user-level threads are backed by unique kernel threads, then the program gets the following benefits -

1. All threads can execute concurrently on separate processors. The scheduler can assign each thread to a processor and if possible all of them can really execute in a parallel fashion.

2. Depending on the kernel to manage threads is simple compared to managing user-level threads separately.

3. Pausing for some specific time is easier with kernel threads because the kernel can easily access the APIC timer.

4. The scheduler can be built with a simplistic model without the concept of the user-level scheduler.

==== Disadvantages ====

When all user-level threads are backed by kernel threads, then the their creation, destruction, pausing, waiting, blocking, etc. become expensive operation due to a context switch to the kernel. It also causes a load-imbalance at the process level. A process with a huge amount of threads running could harm the system as it will take up most of the vital time of other important tasks. This problem can be dealt with '''group scheduling.'''

=== '''1:n Model''' ===

In this model, all the user-level threads are directly mapped to one kernel thread. For the kernel, this process will look like a single-threaded program. But in user-mode the kernel-thread will directly execute the user-level scheduler which in turn selects a thread.

==== Advantages ====

This method reduces the overhead of creating, destroying, blocking, etc. user-level threads as all threads can be managed in user-space without any context switch to the kernel. It also allows the process to have a custom scheduler for its tasks and could also help in co-operative scheduling

==== Disadvantages ====

Threads cannot easily be blocked for external resources without stopping other threads. As the kernel can only pause a kernel thread, the blocking of a user-level thread will in turn take down the process's output if it takes too long.

=== '''M:N Model''' ===

This is a theoretically ideal model for threading. There are 'M' number of threads in user-space backed by 'N' number of kernel threads, provided M > N. It was previously implemented on FreeBSD but was deprecated due to its complexity.

==== Advantages ====

This method allows fast user-level thread management with the functionality of kernel-threads. Also, it reduces the resource usage of thread structures in kernel. For example, if there are two CPUs in the system and a process uses 8 threads. It would be more practical to map groups of 4 threads to 2 kernel threads. The priority of these two threads could be increased as the mapping increases, and that will allow a dynamic relationship between user-level and kernel threads. If more than two-kernel threads are used for running threads, then they will still not be able to execute concurrently due to the limitation of two CPUs.

==== Disadvantages ====

The M:N model is quite complicated. It involves co-operation from both the user-level side and the kernel side. In practical tests, this complexity has many times failed for general applications to give substantial benefit over the 1:1 model. This is why it has been deprecated in systems like Solaris and FreeBSD.

== See Also ==
* Warton, Matthew. ''Single Kernel Stack L4''. 2 Nov 2005. https://sjmulder.nl/dl/pdf/unsorted/2005%20-%20Warton%20-%20Single%20Kernel%20Stack%20L4.pdf
* [https://sites.ualberta.ca/dept/chemeng/AIX-43/share/man/info/C/a_doc_lib/aixprggd/genprogc/understanding_threads.htm Understanding Threads]

[[Category:Processes and Threads]]
[[Category:Multiprocessing]]
[[Category:Multitasking]]
````

<a id="w004248"></a>

## W004248 — SMT

Bron-ID: W004248
Namespace: Main (0)
Revisie: 27976 | Datum: 2023-07-09T17:21:37Z
Bron: https://wiki.osdev.org/SMT
Revisie-URL: https://wiki.osdev.org/index.php?title=SMT&oldid=27976

````text
{{Stub}}
== Introduction ==

Simultaneous Multi-threading is a technique in hardware which focuses to improve efficiency of CPUs which support speculative execution. It is designed by allowing multiple execution pipelines in one physical core. It is based on the principle that accessing memory in RAM is eternally very long as compared to execution at the CPU level. Whenever the processor reads/writes data to or from physical memory then it remains idle for a considerable amount of time. That time can be utilized to execute other instructions in alternative execution pipelines.
````

<a id="w004249"></a>

## W004249 — TSC

Bron-ID: W004249
Namespace: Main (0)
Revisie: 29527 | Datum: 2025-05-25T23:31:33Z
Bron: https://wiki.osdev.org/TSC
Revisie-URL: https://wiki.osdev.org/index.php?title=TSC&oldid=29527

````text
{{Stub}}

== Introduction ==

The '''Timestamp Counter''' is a 64-bit internal register which is present in all Intel processors after the Pentium. It stores the number of cycles executed by the CPU after the latest reset. The time-stamp counter can be read by software using the '''RDTSC''' instruction. It was a precise method of getting a high-resolution measure of the passage of time. But on hyper-threading and multi-core systems, user-level software cannot rely on it as the time-stamp counters of all the CPUs in the system may or may not be synchronized. Other than that, the speed of the CPU may change as the OS or BIOS may take power-saving steps or step-up the performance of the CPU using hardware-based mechanisms. But recent processors also allow a ''constant-rate TSC'' which ticks at the nominal frequency of the CPU rather than the current-frequency of the CPU which depends on a turbo state or power-saving state. This feature is used in kernels to record time with high-precision and low-overhead on each CPU.

The RDTSC (and RDTSCP) instruction can be disabled in user mode by setting the ''Time Stamp Disable'' (TSD) bit in [[CR4]].

== Portability ==

The TSC and related instructions are not portable and using them in '''user-level software is not advised due to portability reasons'''. Some CPU vendors which compete with Intel don't have a TSC or don't implement the RDTSC instruction.

== Relation with the APIC Timer ==

The TSC is now used widely for measuring time in modern kernels, like the linux kernel. The APIC timer can be set to the '''TSC-deadline mode''' which uses the processor's internal time-stamp counter to issue an interrupt when the counter exceeds a specific value decided by software. But it is not guaranteed that the APIC timer will use the processor's core crystal frequency (internal clock) or the processor's bus clock. The IA32_TSC_DEADLINE_MSR (model-specific register) is used for setting the deadline value when the interrupt should be issued.

== See also ==

=== Threads ===
* [[Topic:21038|TSC Rate]] - goes into details about measuring time using the TSC

[[Category:x86 CPU]]
[[Category:CPU Registers]]
````

<a id="w004250"></a>

## W004250 — User:Shukantpal

Bron-ID: W004250
Namespace: User (2)
Revisie: 22078 | Datum: 2018-01-14T07:32:06Z
Bron: https://wiki.osdev.org/User:Shukantpal
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AShukantpal&oldid=22078

````text
Shukant Pal is a OSDev'er who is creating his own software infrastructure. He wants to redefine how software internally works and wants to fully open-source the day-to-day software we use. He has started out by building a OS kernel (silcos kernel).
````

<a id="w004251"></a>

## W004251 — User:Shukantpal/Silcos Kernel

Bron-ID: W004251
Namespace: User (2)
Revisie: 22493 | Datum: 2018-07-01T09:39:30Z
Bron: https://wiki.osdev.org/User:Shukantpal/Silcos_Kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AShukantpal%2FSilcos_Kernel&oldid=22493

````text
== Introduction ==

The Silcos kernel is designed to build a portable and reliable software platform above it. It abstracts system services and policies within the kernel in a object-oriented and modular approach. It is built in C++, C, and intel-syntax assembly. It uses infrequent extended inline asm with GCC support. It has become a open-source project which can be further developed by the open-source community. It was first commited on GitHub on October 19, 2017 and is undergoing rapid development since then. It has become a fairly mature kernel waiting for becoming ripe and supporting user-level software. The Silcos kernel project is a part of the larger view of a open-source, modular and secure microkernel-based operating system. It incorporates several system services in the kernel space to reduce the amount of IPC overhead seen in traditional microkernels. But it keeps the drivers strictly outside the kernel, except interrupt-handlers and special cases, and also aims to implement the '''Uniform Driver Interface'''. It follows the key principles and techniques of other kernel-mode software (like LINUX) and aims to form a similar interface for user-level software, allowing them to be seamlessly ported to the Silcos application environment. At the same time, it incorporates a generic interface for most services allowing them to be '''easily extended using loadable kernel modules'''.

=== Supported Architectures ===

The kernel will eventually come to all major hardware architectures. It is currently being built for the Intel IA32 architecture and handles the most recent hardware developments. Right now, it doesn't focus on the older hardware bugs and is keen to run only on recent CPUs after 2010. It wants to be more compatible towards advanced hardware rather than being bloated with older-hardware compatibility software. Eventually it will support the following architectures -

1. IA32 and IA64

2. ARM

3. PowerPC

=== Overview ===

The kernel source is stored online at [GitHub https://github.com/SukantPal/Silcos-Kernel]. The kernel is highly modular in terms of source code and binary interface. The whole kernel is divided into various modules which separates the highly-used code from low-priority code. This improves memory-locality and cache efficiency. All of the boot-time modules are stored in the first 2-MB huge page of the kernel-space (3GB higher-half space). This means only TLB entry is required to map all of the kernel boot-time modules.

=== Initialization ===

Although not listed in the section below, the Initor module in the ''/Initializer'' directory of the workspace, is actually loaded by the bootloader. It is listed as the main kernel in the ''grub.cfg'' configuration file, but it is actually responsible for loading rest of the kernel-modules.

=== Modules ===

The kernel currently divides itself into the following modules -

1. KernelHost

2. HAL

3. ModuleFramework

4. ObjectManager

5. ResourceManager

6. ExecutionManager (formerly the Microkernel which also comprised of KernelHost & HAL but was split into three)

Each module is loaded at runtime by the bootloader and passing into the multiboot structure given to the kernel. The kernel host initializes itself - physical memory allocator, kernel-page allocator, object (slab) allocator, module-loader. After initializing itself, the KernelHost links the other kernel modules and if any, will show a linking error and stop. Otherwise, it will continue and call the initialization function of each module. This is in the form of __init functions in each module. This function can be utilized to setup any objects that have their own specific allocator (in slab allocation techniques).

== KernelHost ==

The KernelHost module is the one which is given control by the bootloader. It uses multiboot and expects protected mode with uniform segments. The first thing is does is load the defaultBootGDT and enable higher-half paging. Then it parses the multiboot table and calls the Main() function. This early-boot code is present in KernelHost/Source/Boot/IA32/InitRuntime.asm which co-operates with paging code. Then the Main() function is given control (after loading the ESP with a kernel boot stack of 4KB size).

KernelHost comprises of these following services -

1. Physical Memory Allocator (see Memory/KFrameManager.h)

2. Kernel Page Allocator (see Memory/KMemoryManager.h)

3. Slab Allocator (see Memory/KObjectManager.h)

4. Module Loader & Linker (see ModuleLoader/*)

=== Memory Allocation Techniques ===

The Silcos kernel uses the same technique for allocating kernel pages and page frames in physical memory. To save space, the kernel developer(s) have separated the algorithm from the actual allocator. This means that the page-frame allocator & kernel-page allocator actually share the code which allocates their memory blocks. This technique saves binary file-size and improves memory-locality. The kernel uses the zone allocation algorithm to take blocks from individual memory zones, which relies on the buddy allocation technique to manage blocks in powers of 2 in each zones.

'''BuddyAllocator''' - Memory/BuddyAllocator.hpp

'''ZoneAllocator''' - Memory/ZoneAllocator.hpp

'''Buddy Allocator''' - The buddy allocator uses a table of block descriptors provided by the actual allocator. These blocks are the smallest unit of allocation from the client allocator. In both cases, the page-frame allocator and kernel-page allocator pass a table of descriptors which hold information on 4KB blocks of page-frames & kernel-pages. The allocator receives allocation in the form of pointer to the first block in the amount of memory allocator. For example, if at 64KB address 16KB is allocated then the block at 64KB will be returned. The allocator must convert the block to its address.

The Silcos kernel uses improvements over the traditional binary buddy allocator. It introduces the concept of super-blocks and lists for each (lowerOrder, upperOrder) pair of buddy blocks. It is explained in greater detail in the GitHub wiki.

'''Zone Allocator''' - The zone allocator divides the memory into various zones which have fixed & aligned boundaries. These boundaries are aligned to the maximum size of the block that can be allocated. For example, the page-frame allocator allows upto 2MB allocations so all zones are aligned by 2MB boundaries. A zone allocator configures itself from a global block descriptor table. In the page-frame allocator, for example, page-frames are represented by MMFRAME descriptors which are located at a fixed address in physical memory (depending on the architecture). The zone allocator will have separate buddy allocators for each memory zone. These buddy allocators will hold pointers to the table of blocks from which they will allocate.

The zone allocator also provides zone preferences. For example, the page-frame allocator provides multiple zones for kernel, code and data allocations. This reduces concurrent access to a zone in multiple-processor systems and reduces the overall synchronization overhead. But if the CODE zone is filled then allocating from DATA zone is okay and vice versa is also okay. Similar is for all three zones - kernel, data, and code. So all of these zones are of the same preference.

Zones of the same preference are kept in the same circular list. But each preference is given a separate list of zones and kept at a specific index in an array. For example, the DRIVER zone in 32-bit system is for 32-bit devices which use maximum 4GB addresses. So if DRIVER zone is fully used, then going to CODE zone above it is wrong essentially. Thus, it is of lower preference. Similar is for the DMA zone where the limit is 16MB. The page-frame allocator uses only three preferences - (DMA), (DRIVER), (CODE, DATA, KERNEL).

The kernel-page allocator has two zones with only one preference - (ZONE_KOBJECT, ZONE_KMODULE) for objects & modules specifically.

=== Object Allocation ===

The kernel uses a slab allocator for getting space for all kernel objects. Interfaces are provided for creating & deleting object allocators at runtime. The allocator has a front-end and a back-end. The back-end allocates kernel-pages as 'slabs' and creates a slab-descriptor at the very end of that page. These slabs are just 4KB blocks of raw memory that are mapped to any arbitrary physical address which contain a descriptor at the end. Now, the maximum no. of objects are carved out of the slab and linked to the descriptor at the end in a stack. Whenever a object is allocated from the slab it is popped out of the stack and whenever a object is freed back to the slab it is pushed to that stack.

The allocator maintains two lists of slabs - partial and full list. The partial list contains those slabs which have some objects in their stacks and can be used for allocation. The full list contain slabs which are saturated and cannot be used for allocating fresh objects. A cache of one empty slab is maintained in the allocator too. This is to reduce slab-allocation overhead on small fluctuations in object requirement.

=== Module Loader ===

The module loader is a essential component of the kernel host. It is the main reason it is called the kernel host - it loads and manages the other modules. The module loader maintains records of other modules and their dynamic-link structs which contain information about their symbols. It unpacks the file-image of the module and forms segments in kernel-memory. This part of the KernelHost makes it bloated and large due to which the former Silcos microkernel was divided into KernelHost, HAL and ExecutionManager.

The module loader uses the Executable and Linkable Format as the basis of its ABI and module infrastructure. Each module is implemented as a shared library (other than the KernelHost itself, as earlier mentioned, it is a PIE).

Recent changes to the kernel have allowed the KernelHost to contain undefined symbols that actually link to other modules. This means memory-allocators can use features of other modules after a sign indicating that modules are linked changes.

== ExecutionManager ==

The kernel abstracts all execution-based services into this module. It controls critical aspects of the system which directly affect its performance & reliability. All these services were packed into a single module that would fit into one 32-KB L1 cache. Scheduling and task management come under its jurisdictions.

=== Task Management ===

The kernel manages tasks in a generic manner. Any task descriptor can refer to a thread-group, scheduler activation, or just a traditional thread. This generalization evolves the techniques of group scheduling and user-level scheduling. A task is more primitive than the thread and may or may not have a user stack (but a kernel stack is required). It is dispatched by a dispatcher function which can continue to execute the thread or jump to another scheduling context (which may be in user-space).

=== Scheduler ===

The kernel generalizes the concept of scheduling by using the concept of scheduling class, which is already present in most modern kernels. Each processor has three scheduler task queues - which are abstracted in the (C++) class ScheduleRoller, which contains mostly pure virtual functions. All processor structs contains three pointers to unique ScheduleRollers which are actually child-classes of the ScheduleRoller at a fixed offset. This allows the size of the scheduler's task queue to changed effectively.

The kernel (will) provide these three scheduling classes -

'''1. RoundRobin'''

'''2. CFS'''

'''3. Soft Real-time Scheduling'''

'''TODO:''' Finish documentation on all other modules
````

<a id="w004252"></a>

## W004252 — Silcos Kernel

Bron-ID: W004252
Namespace: Main (0)
Revisie: 22085 | Datum: 2018-01-14T14:28:35Z
Bron: https://wiki.osdev.org/Silcos_Kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Silcos_Kernel&oldid=22085
Doorverwijzing: User:Shukantpal/Silcos Kernel

````text
#REDIRECT [[User:Shukantpal/Silcos Kernel]]
````

<a id="w004253"></a>

## W004253 — User:Aj/Windows Subsystem for Linux

Bron-ID: W004253
Namespace: User (2)
Revisie: 22090 | Datum: 2018-01-15T12:25:05Z
Bron: https://wiki.osdev.org/User:Aj/Windows_Subsystem_for_Linux
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAj%2FWindows_Subsystem_for_Linux&oldid=22090

````text
On this page, we cover some basic information about Windows Subsystem for Linux (WSL) when used as a cross-compilation environment. Instructions for setting up the initial cross-compiler on WSL can be found under the article [[GCC Cross-Compiler]] and this article assumes that you now have a working GGC compiler for your chosen target architecture in $HOME/opt/cross/bin. The author also uses Ubuntu for WSL, but other options are available through the Windows Store app.

== File System ==
WSL has its own self-contained file system. Although this is theoretically accessible (as a sub-folder in %AppData%), it is '''strongly''' recommended that you do not access WSL files from Windows. This is because files edited from Windows do not  necessarily update inside the WSL shell.

Instead, you should use the automatic mountpoints within WSL to access files on the native Windows filesystem. For those unfamiliar with Linux, these mountpoints are located under "/mnt/[drive]". For example, the root of "C:\" can be found under "/mnt/c". The command "ln -s" (create symbolic link) may be used to make your chosen project location easily accessible in your home directory.

Suppose an IDE is being used in Windows such as Eclipse, Visual Studio etc... it would be possible to use ln as follows:

   ln -s "/mnt/c/Users/[username]/Documents/MyOS" ~/myos

This will create a permanent symlink to your project on your usual Windows filesystem, accessible by typing "cd ~/myos" from your Linux shell. Files will be kept up to date so that your compiler and IDE see the same version of your source files.

== Suggested Packages ==
It is suggested that you keep your chosen WSL distro up to date by typing:
   
   sudo apt-get update
   sudo apt-get upgrade

at the Bash shell. This will update the package lists and automatically update any packages that you have installed via "apt-get install x". In addition, you may find the following packages useful as a part of your toolchain:

   sudo apt-get install x

where x is:

grub2 - Install GRUB2 including tools such as grub-mkrescue, recommended elsewhere on the wiki for booting your OS.
git - Install Git version control.
nasm - Install the NASM assembler.
xorriso - Used by GRUB2 if you are using ISO images. On testing, grub-mkrescue has been observed to silently fail if this is not installed!

While many other packages can be useful for OS development, many of them will already be installed as requirements of the [[GCC Cross-Compiler]] build process.
````

<a id="w004254"></a>

## W004254 — User:Aj/WSL

Bron-ID: W004254
Namespace: User (2)
Revisie: 22088 | Datum: 2018-01-15T12:04:25Z
Bron: https://wiki.osdev.org/User:Aj/WSL
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAj%2FWSL&oldid=22088
Doorverwijzing: User:Aj/Windows Subsystem for Linux

````text
#REDIRECT [[User:Aj/Windows Subsystem for Linux]]
````

<a id="w004255"></a>

## W004255 — UDI Environment

Bron-ID: W004255
Namespace: Main (0)
Revisie: 23112 | Datum: 2018-12-01T02:42:31Z
Bron: https://wiki.osdev.org/UDI_Environment
Revisie-URL: https://wiki.osdev.org/index.php?title=UDI_Environment&oldid=23112

````text

== Introduction ==

The UDI Environment is described, by the definition in the real documentation, as a description and its corresponding implementation of all interfaces provided to UDI-compliant drivers which include the system services, implicit scheduling and synchronization, and inter-module communication infrastructure. The environment is the means by which the Uniform Driver Interface provides a software and hardware independent nature to its drivers. The UDI specs provide a complete environment for the development of compliant driver.

The drivers use the services of the UDI environment by '''service calls'''. These are implemented as functions which are '''exported to all UDI driver instances'''. This makes them common to all UDI drivers. The environment must provide two types of service calls - asynchronous and synchronous.

== Development ==

UDI environments are generally tied to the OS and are more platform-dependent. They can be developed by looking into the UDI specifications. The UDI environment must implement some '''core services''' and '''utility functions'''. For source code compatibility, the interface must be compacted into two header files - '''udi.h''' and '''udi_physio.h''' (see Physical I/O Specifications).

== Interfaces ==

=== Fundamental Types ===

The UDI environment provides utility functions & '''types'''.

The fundamental types given by UDI are -

==== Specific-length Types ====

The Uniform Driver Interface declares specific-length types for use in arithmetic and logical operations. They have fixed lengths that don't change between different platforms.

<source lang="C">
typedef	uint8_t		udi_ubit8_t;
typedef uint16_t	udi_ubit16_t;
typedef uint32_t	udi_ubit32_t;
typedef int8_t		udi_sbit8_t;
typedef int16_t		udi_sbit16_t;
typedef int32_t		udi_sbit32_t;
typedef bool		udi_bool_t;
</source>

The driver must use these types while passing arguments and returning variables. They can also use the fast ints instead of the fixed width ones.

==== Abstract Types ====

These are '''integral types''' whose size is defined by the implementation. By keeping the size of these types abstract, the Uniform Driver Interface is able to maintain source portability between different platforms and also allow drivers to adapt to newer architectures. Specifically, for binary portability, their size is defined by the corresponding ABI specification.

===== Size Type =====

This type is used for referring to the size to any '''buffer''' or '''object'''. It can also store offsets in buffers and memory itself. It is flexible and can '''represent different ranges in different domains''' which means that its size can change between different domains. That is why it is a '''non-transferable''' variable across domain boundaries.

It must have at least 16 bits in size, for following guidelines.

<source lang="C">
typedef <INTEGRAL TYPE> udi_size_t;
</source>

===== Index Type =====

This is '''used by the driver''' to access indices of object stored in an array. It holds values in the range 0 to 255 (is zero-based) for the driver. As indices to environment object are global for the driver, it is transferable across domain boundaries.

I think is type need not have a 1 byte size as the UDI environment '''may encrypt''' the index to hide it from the driver (information presented in this statement is not guaranteed).

<source lang="C">
typedef <INTEGRAL TYPE> udi_index_t;
</source>

'''i) Control Block Index'''

The index type can be used to refer to a control that was registered through '''udi_cb_init_t''' structure. It can be used for allocating control blocks and getting hold of scratch space sizes and configuring other properties of UDI control blocks.

Note that zero is a invalid value currently, and is reserved for future usage.

'''ii) Metalanguage Index'''

The index type can also be used for holding metalanguage indices. It can be used for figuring out the metalanguages used by a driver. These indexes as associated with '''meta''' declarations in Static Driver Properties. These are of importance while the UDI Build System compiles and bundles the drivers.

'''iii) Ops Index'''

It can also be used to point to a channel operations vector by using the '''udi_ops_init_t''' function. It can be used to anchor channels and set default channel types by the driver.

Zero is a reserved value here also, and is booked for future usage.

'''iv) Region Index'''

A region-index is used to refer to a driver-defined region and is used for manipulating attributes of regions created by or on behalf of the driver instance.

The zero value refers to the '''primary region''' of the driver while other values refer to '''secondary regions'''.

==== Opaque Types ====

These types are not for direct access by drivers. They are essentially handles (like in Windows NT, just relating man) which are passed to service calls to refer to actual objects. They are used to protect system data and prevent malicious code from corrupt valuable data. The size of these handles are specified in the ABI bindings, so these handles have the size of long on IA32/64 architectures.

A rather simple implementation will use just the address of the object in the handle. But for better security, determined projects could use a value that is the result of the operation between the ID given to a UDI driver and a index into a handle table. After undoing the effect of that operation, the environment will get the address of the object by accessing a handle table.

Each '''opaque handle type''' will have a corresponding '''null''' value. It is used for referring to null objects that don't really exist. This value need not be zero and thus the '''UDI_HANDLE_IS_NULL macro''' is provided for testing whether the handle carries a null value. null handles are not to be passed to service calls and will be treated as invalid operands. Also, the driver should never expect to receive a null value from a returning service call.

These handles may or may not be transferable depending on the object being referred to.

'''i) Channels & udi_channel_t'''

'''File''' - <source lang="C">#include "udi.h"</source>
'''Declaration''' - <source lang="C">typedef <HANDLE> udi_channel_t;</source>
'''NullHandle''' - <source lang="C">#define UDI_NULL_CHANNEL <NULL HANDLE VALUE></source>

'''udi_channel_t''' is used to refer to UDI Channels (described on another page) and are '''transferrable opaque handles'''. They are used for bi-directional communication between different drivers and with the environment.

'''ii) Buffer Path Routing Handle & udi_buf_path_t'''

'''File''' - <source lang="C">#include "udi.h"</source>
'''Declaration''' - <source lang="C">typedef <HANDLE> udi_buf_path_t;</source>
'''NullHandle''' - <source lang="C">#define UDI_NULL_BUF_PATH <NULL HANDLE VALUE></source>

All UDI buffers are associated with buffer path handles. They indicate destinations for incoming data and are explicitly allocated by the driver. They are '''not transferable between regions'''.

'''iii) Origin Handles & udi_origin_handle_t'''

'''File''' - <source lang="C">#include "udi.h"</source>
'''Declaration''' - <source lang="C">typedef <HANDLE> udi_origin_handle_t;</source>
'''NullHandle''' - <source lang="C">#define UDI_NULL_ORIGIN <NULL HANDLE VALUE></source>

Environments should use '''origin handles''' to hold information about origination of user requests. UDI drivers are responsible for copying the origin handle from received control blocks into control blocks generated on behalf of that control block. This is used by the environment to track quota usage, resource distribution, etc. in the system.

The UDI_NULL_ORIGIN may be set for the origin of any control block instead of copying it from another source, but drivers can't allocate or delete origin handles on their own. They are '''transferable opaque handles'''.

==== Self-contained Opaque Types ====

These are special opaque-like types that can be interpreted by the environment implementation only, but they are stored locally and have value semantics. Normal opaque handles have reference semantics but here the case is different. Assignment will make a copy of the whole object.

These are '''not transferable between regions'''.

'''i) Timestamp Type''' - This is used to refer to a point in time and is relative to a starting point that is implementation defined. The driver cannot make use of it on its own and it is for determining relative differences in time. The implementation should use the '''kernel-timestamp''' maintained by the kernel for scheduling use. It should be able to do so effectively by mapping the time variable in user-space itself.

<source lang="C">typedef <OPAQUE TYPE> udi_timestamp_t;</source>

=== Semi-Opaque Types ===

UDI also levers the advantages of object that have a driver-visible part and implementation-dependent part too. The visible part is a C structure and is kept with a pointer by the driver.

These are allocated by the environment because the driver ain't knowing what the size of the object is.

==== Control Blocks ====

The Uniform Driver Interface provides a _control block_ type to provide a context for asynchronous service calls and channel operations. These are semi-opaque object and are '''transferable'''.

They will be discussed on another page.

==== Buffers ====

UDI also provides a buffer type for storing object and protocol data in dynamic memory on the fly. They need to be logically continuous and any machine or OS differences need to hidden. This should not be hard to implement - all you need is a large & small memory allocator.

We'll have another page on UDI buffer 'management'.

=== Structures with Fixed-binary representation ===

Driver usually don't need to be concern with the binary representation of bits or endian-ness of the number while operating in software-defined data structures. But hardware-defined structures have a '''fixed endian-ness''' which causes architectural dependencies in binary files if not taken care of. That is why UDI also provides functions in the environment for the drivers to store and manipulate hardware-defined types with specified binary representation. Drivers have to follow certain rules while declaring C hardware-defined structs.

=== Commonly Used Derived Types ===

These types are not fundamental and are derived from other UDI types, but are not implementation dependent.

==== UDI Status ====

This type is used for providing a uniform method of reporting status or error within the I/O system.

'''File''' - <source lang="C">#include <udi.h></source>
'''Declaration''' - <source lang="C">typedef udi_ubit32_t udi_status_t;</source>
'''MaskValues & Flags''' -
<source lang="C">
#define UDI_STATUS_CODE_MASK               0x0000FFFF
#define UDI_STAT_META_SPECIFIC             0x00008000
#define UDI_SPECIFIC_STATUS_MASK           0x00007FFF
#define UDI_CORRELATE_OFFSET               0x16
#define UDI_CORRELATE_MASK                 0xFFFF0000
</source>
'''StatusValues''' -
<source lang="C">
#define UDI_OK                             0  // request was successfully completed without any errors
#define UDI_STAT_NOT_SUPPORTED             1  // this operation is not supported or given parameters are not handled by the environment
#define UDI_STAT_NOT_UNDERSTOOD            2  // parameters given exceed a range or don't make sense in this context, used in channel operations
#define UDI_STAT_INVALID_STATE             3  // request understand and supported, but parameters don't make sense here; for channel operations
#define UDI_STAT_MISTAKEN_IDENTITY         4  // request is implemented and understood, but not applicable for the device or object concerned
#define UDI_STAT_ABORTED                   5  // operation was successfully aborted on requesting cancellation
#define UDI_STAT_TIMEOUT                   6  // operation timeout passed due to which it was aborted
#define UDI_STAT_BUSY                      7  // the device or object concerned is being used by another process or is busy
#define UDI_STAT_RESOURCE_UNAVAIL          8  // the system cannot provide anymore resources for the given request
#define UDI_STAT_HW_PROBLEM                9  // system has detected a hardware problem due to which the request could not execute
#define UDI_STAT_NOT_RESPONDING            10 // device is not present or not responding
#define UDI_STAT_DATA_UNDERRUN             11 // data transfer from device is less than required or expected
#define UDI_STAT_DATA_OVERRUN              12 // data transfer from device is more than required or expected
#define UDI_STAT_DATA_ERROR                13 // data transfer error occurred from device
#define UDI_STAT_PARENT_DRV_ERROR          14 // parent device has faced an error due to this request failing
#define UDI_STAT_CANNOT_BIND               15 // driver tried to bind with its parent, but was denied; also used by the parent to indicate denying
#define UDI_STAT_CANNOT_BIND_EXCL          16 // driver cannot bind exclusively to the parent as another child is already bound
#define UDI_STAT_TOO_MANY_PARENTS          17 // this device cannot bind with the parent as the parent already has maximum supported children
#define UDI_STAT_INVALID_PARENT            18 // the parent-binding request cannot execute as meta-language of parent is not supported by driver
#define UDI_STAT_TERMINATED                19 // request ended abruptly due to termination of the region without notice
#define UDI_STAT_ATTR_MISMATCH             20 // the driver has been given a custom attribute that it cannot apply on its device
</source>

UDI Status is a 32-bit type composed of two separate 16-bit values as seen in the two different sets of macros given. They are ''''status code'''' field and a ''''correlation field''''.

When a error must be signaled then the driver must put an proper status code and assign it to the '''udi_status_t''' variable. The correlation field must be 0 for new errors. It is to be set for errors that are derived from other errors or which have meta-language specific semantics.

The STATUS_CODE_MASK is used for filtering out the correlation field and getting out the status field. The UDI_STAT_META_SPECIFIC macro is used for telling that the '''status code used is not of the common status codes given and is meta-language specific'''. Those are not defined here, not.

[[Category:Uniform Driver Interface]]
````

<a id="w004256"></a>

## W004256 — Talk:UDI Environment

Bron-ID: W004256
Namespace: Talk (1)
Revisie: 22111 | Datum: 2018-01-20T13:29:32Z
Bron: https://wiki.osdev.org/Talk:UDI_Environment
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AUDI_Environment&oldid=22111

````text
Pls somebody advice me about how to write this page on UDI Environment. I have already started giving brief descriptions of the various UDI types that are required to be given by the environment. Am I correctly writing this page (UDI Environment)?
----
--[[User:Shukantpal|Shukantpal]] 05:54, 18 January 2018 (CST)

:I don't know enough about UDI, but it's looking good. [[User:Bellezzasolo|Bellezzasolo]] 06:00, 20 January 2018 (CST)

::Okay, I am gonna remain busy for these few weeks so yah. Work is paused here for sometime. Contributions are welcome...

----
--[[User:Shukantpal|Shukantpal]] 07:29, 20 January 2018 (CST)
````

<a id="w004257"></a>

## W004257 — AMD Atombios

Bron-ID: W004257
Namespace: Main (0)
Revisie: 30413 | Datum: 2026-03-25T03:12:18Z
Bron: https://wiki.osdev.org/AMD_Atombios
Revisie-URL: https://wiki.osdev.org/index.php?title=AMD_Atombios&oldid=30413

````text
A pointer to the AMD Atombios may be found in either the BAR0 or expansion ROM fields in the [[PCI#PCI Device Structure|PCI config space]] of most current AMD graphics cards, as well as AMD's integrated graphics processors.

== Commands ==

To use execute the commands in the [[#Command Table|command tables]], one needs to write an [[AMD Atombios Bytecode|Atombios Bytecode]] interpreter. Example Atombios interpreter code can be found in the Linux source tree in drivers/gpu/drm/radeon. Check around in the files atom.c and atom.h. One of the first commands one would need to issue is the ASIC_Init command at index 0, which initializes the ASIC (Application Specific Integrated Circuit). These commands provide an API for the driver to configure the graphics card without necessarily knowing the specific registers and values to write.

== Data ==

The [[#Data Table|data table]] points to other tables which give useful information about the graphics card, like the engine, memory and reference clocks.

== Structures ==

All offsets are from the beginning of the ROM image. To find the offsets of other tables, see this table, with some important fields:

{| class="wikitable"
!Offset
!Size
!Description
|-
|0x00
|Word
|BIOS Magic, should be 0xAA55
|-
|0x30
|10 bytes
|ATI Magic, should be " 761295520"
|-
|0x48
|Word
|[[#Atom ROM table|Atom ROM Table Base]]
|}

=== Common Header ===

Most tables will have a header with the following structure:

{| class="wikitable"
!Offset
!Size
!Description
|-
|0x00
|Word
|Structure Size
|-
|0x02
|Byte
|Table Format Revision, changed when parser is not backward compatible
|-
|0x03
|Byte
|Table Content Revision
|}

=== Atom ROM table ===

{| class="wikitable"
!Offset
!Size
!Description
|-
|0x00
|4 bytes
|Common header
|-
|0x04
|4 bytes
|Atom Magic, should be "ATOM"
|-
|0x08
|Word
|BIOS Runtime Segment Address
|-
|0x0A
|Word
|Protected Mode Info Offset
|-
|0x0C
|Word
|Config Filename Offset
|-
|0x0E
|Word
|CRC Block Offset
|-
|0x10
|Word
|Name String offset, sometimes zero terminated. Maximum 512 bytes.
|-
|0x12
|Word
|Int 10 offset
|-
|0x14
|Word
|PCI Bus Device Init Code
|-
|0x16
|Word
|IO Base Address
|-
|0x18
|Word
|Subsystem Vendor ID
|-
|0x1A
|Word
|Subsystem ID
|-
|0x1C
|Word
|PCI Info Offset
|-
|0x1E
|Word
|Command Table Base
|-
|0x20
|Word
|Data Table Base
|-
|0x22
|Byte
|Extended Function Code
|-
|0x23
|Byte
|Reserved
|}

=== Command Table ===

The offset for this table is found at offset 0x1E of the [[#Atom ROM table|Atom ROM table]]. Many of these tables are only internally between functions. All of the values in this table are offsets from the start of the ROM image, except for the header.

{| class="wikitable"
!Offset
!Size
!Description
|-
|0x00
|4 bytes
|[[#Common Header|Common Header]]
|-
|0x04
|x words
|Array offsets of commands. See [[AMD Atombios Commands]] for a list of commands.
|}

=== Command Format ===

{| class="wikitable"
!Offset
!Size
!Description
|-
|0x00
|Word
|Command Length
|-
|0x02
|Word
|Reserved
|-
|0x04
|Byte
|WS?
|-
|0x05
|Byte
|PS - Parameter Stack
|-
|0x06-0x????
|??
|Command bytecode
|}

=== Data Table ===

The offset for this table is found at offset 0x20 of the [[#Atom ROM table|Atom ROM table]]. All of the values in this table are offsets from the beginning of the ROM.

{| class="wikitable"
!Offset
!Size
!Description
|-
|0x00
|4 bytes
|[[#Common Header|Common Header]]
|-
|0x04
|Word
|UtilityPipeline
|-
|0x06
|Word
|MultimediaCapabilityInfo
|-
|0x08
|Word
|MultimediaConfigInfo
|-
|0x0A
|Word
|StandardVesa_Timing
|-
|0x0C
|Word
|[[#Firmware Info|FirmwareInfo]]
|-
|0x0E
|Word
|PaletteData
|-
|0x10
|Word
|LCD_Info
|-
|0x12
|Word
|DIGTransmitterInfo
|-
|0x14
|Word
|AnalogTV_Info
|-
|0x16
|Word
|SupportedDevicesInfo
|-
|0x18
|Word
|GPIO_I2C_Info
|-
|0x1A
|Word
|VRAM_UsageByFirmware
|-
|0x1C
|Word
|GPIO_Pin_LUT
|-
|0x1E
|Word
|VESA_ToInternalModeLUT
|-
|0x20
|Word
|ComponentVideoInfo
|-
|0x22
|Word
|PowerPlayInfo
|-
|0x24
|Word
|CompassionateData
|-
|0x26
|Word
|SaveRestoreInfo
|-
|0x28
|Word
|PPLL_SS_Info
|-
|0x2A
|Word
|OemInfo
|-
|0x2C
|Word
|XTMDS_Info
|-
|0x2E
|Word
|MclkSS_Info
|-
|0x30
|Word
|Object_header
|-
|0x32
|Word
|IndirectIOAccess
|-
|0x34
|Word
|MC_InitParameter
|-
|0x36
|Word
|ASIC_VDDC_Info
|-
|0x38
|Word
|ASIC_InternalSS_Info
|-
|0x3A
|Word
|TV_VideoMode
|-
|0x3C
|Word
|VRAM_Info
|-
|0x3E
|Word
|MemoryTrainingInfo
|-
|0x40
|Word
|IntegratedSystemInfo
|-
|0x42
|Word
|ASIC_ProfilingInfo
|-
|0x44
|Word
|VoltageObjectInfo
|-
|0x46
|Word
|PowerSourceInfo
|}

=== Firmware Info ===

The offset for this table is found at offset 0x0C of the [[#Data Table|Data Table]]. Clock values are in 10KHz units. This table has many versions, but many of the fields are the same. In versions other than 1.1, only the changed fields will be listed.

==== Version 1.1 ====
{| class="wikitable"
!Offset
!Size
!Description
|-
|0x00
|4 bytes
|[[#Common Header|Common Header]]
|-
|0x04
|Dword
|Firmware Revision
|-
|0x08
|Dword
|Default Engine Clock
|-
|0x0C
|Dword
|Default Memory Clock
|-
|0x10
|Dword
|Driver Target Engine Clock
|-
|0x14
|Dword
|Driver Target Memory Clock
|-
|0x18
|Dword
|Max Engine Clock PLL Output
|-
|0x1C
|Dword
|Max Memory Clock PLL Output
|-
|0x20
|Dword
|Max Pixel Clock PLL Output
|-
|0x24
|Dword
|ASIC Max Engine Clock
|-
|0x28
|Dword
|ASIC Max Memory Clock
|-
|0x2C
|Byte
|ASIC Max Temperature
|-
|0x2D
|15 bytes
|Reserved
|-
|0x3C
|Word
|Min Engine Clock PLL Input
|-
|0x3E
|Word
|Max Engine Clock PLL Input
|-
|0x40
|Word
|Min Engine Clock PLL Output
|-
|0x42
|Word
|Min Memory Clock PLL Input
|-
|0x44
|Word
|Max Memory Clock PLL Input
|-
|0x46
|Word
|Min Memory Clock PLL Output
|-
|0x48
|Word
|Max Pixel Clock
|-
|0x4A
|Word
|Min Pixel Clock PLL Input
|-
|0x4C
|Word
|Max Pixel Clock PLL Input
|-
|0x4E
|Word
|Min Pixel Clock PLL Output
|-
|0x50
|Word
|Firmware Capability
|-
|0x52
|Word
|Reference Clock
|-
|0x54
|Word
|RTS PM4 Starting Location in ROM, in 1kb units
|-
|0x56
|Byte
|RTS PM4 Packets, in 1kb units
|-
|0x57
|Byte
|Design ID
|-
|0x58
|Byte
|Memory Module ID
|-
|0x59
|3 bytes
|Reserved
|}

==== Version 1.2 ====
{| class="wikitable"
!Offset
!Size
!Description
|-
|0x2D
|Byte
|MinAllowedBL_Level
|-
|0x38
|Dword
|MinPixelClockPLL_Output
|-
|0x4E
|Word
|MinPixelClockPLL_Output, lower 16bits of field at 0x38
|}

==== Version 1.3 ====
{| class="wikitable"
!Offset
!Size
!Description
|-
|0x34
|Word
|3D Acceleration Engine Clock
|}

==== Version 1.4 ====
{| class="wikitable"
!Offset
!Size
!Description
|-
|0x2E
|Word
|BootUpVDDCVoltage, in mV
|-
|0x30
|Word
|LcdMinPixelClockPLL_Output, in MHz
|-
|0x32
|Word
|LcdMaxPixelClockPLL_Output, in MHz
|}

==== Version 2.1 ====
{| class="wikitable"
!Offset
!Size
!Description
|-
|0x10
|8 bytes
|Reserved (was DriverTargetEngineClock and DriverTargetMemoryClock)
|-
|0x24
|Dword
|BinaryAlteredInfo (was ASICMaxEngineClock)
|-
|0x28
|Dword
|DefaultDispEngineClkFreq (was ASICMaxTemperature)
|-
|0x2C
|Byte
|Reserved (was ASICMaxTemperature)
|-
|0x34
|Dword
|Reserved (was 3D Acceleration Engine Clock)
|-
|0x52
|Word
|CoreReferenceClock (was ReferenceClock)
|-
|0x54
|Word
|MemoryReferenceClock (was RTS PM4 Location)
|-
|0x56
|Word
|UniphyDPModeExtClkFreq (was RTS PM4 Packets)
|}

== Bytecode ==
{{In Progress}}
=== Instructions ===
{| class="wikitable"
!Pneumonic
!Opcode
!Description
|-
|Invalid
|0x00
|NULL
|-
|MOV
|01-06
|Move
|-
|AND
|07-0C
|Logical And
|-
|OR
|0D-12
|Logical Or
|-
|SHL
|13-18
|Shift Left
|-
|SHR
|19-1E
|Shift Right
|-
|MUL
|1F-24
|Multiply
|-
|DIV
|25-2A
|Divide
|-
|ADD
|2B-30
|Add
|-
|SUB
|31-36
|Subtract
|-
|SETPORT [ATI]
|37
|Set Port (ATI)
|-
|SETPORT [PCI]
|38
|Set Port (PCI)
|-
|SETPORT [SYSIO]
|39
|Set Port (Sysio)
|-
|SETREGBLOCK
|3A
|Set Register Block
|-
|SETFBBASE
|3B
|Set Framebuffer Base
|-
|CMP
|3C-41
|Compare
|-
|SWTICH
|42
|Switch
|-
|JMP
|43-49
|Jump [Condition]
|-
|TEST
|4A-4F
|Test
|-
|Delay [Milli]
|50
|Millisecond Delay
|-
|Delay [Micro]
|51
|Microsecond Delay
|-
|CALLTABLE
|52
|Call Table
|-
|REP
|53
|Repeat
|-
|CLEAR
|54-59
|Clear
|-
|NOP
|5A
|No Operation
|-
|EOT
|5B
|Functionally NOP
|-
|MASK
|5C-61
|Mask
|-
|POSTCARD
|62
|Poscard
|-
|BEEP
|63
|Beep
|-
|SAVEREG
|64
|Save Register
|-
|RESTOREREG
|65
|Restore Register
|-
|SETDATABLOCK
|66
|Set Data Block
|-
|XOR
|67-6C
|Exclusive OR
|-
|SHL
|6D-72
|Shift Left
|-
|SHR
|73-78
|Shift Right
|-
|DEBUG
|79
|Debug
|}
=== Arguments and Conditions ===
From the AMD Linux interpreter:
<syntaxhighlight lang="c">
#define ATOM_ARG_REG		0
#define ATOM_ARG_PS		    1
#define ATOM_ARG_WS		    2
#define ATOM_ARG_ID		    4
#define ATOM_ARG_FB		    3
#define ATOM_ARG_IMM		5
#define ATOM_ARG_PLL		6
#define ATOM_ARG_MC		    7
</syntaxhighlight>
However, when dealing with a block of instructions of length 6, the encoding is as follows:
{| class="wikitable"
!Offset
!Argument
!Description
|-
|00
|ATOM_ARG_REG
|Register Argument
|-
|01
|ATOM_ARG_PS
|
|-
|02
|ATOM_ARG_WS
|
|-
|03
|ATOM_ARG_FB
|Framebuffer?
|-
|04
|ATOM_ARG_PLL
|PLL Register. 8 byte index.
|-
|05
|ATOM_ARG_MC
|MC Register
|}

Conditions:
{| class="wikitable"
!Offset
!Condition
!Description
|-
|00
|ATOM_COND_ALWAYS
|Unconditional
|-
|01
|ATOM_COND_EQUAL
|Equality
|-
|02
|ATOM_COND_BELOW
|Less Than
|-
|03
|ATOM_COND_ABOVE
|Greater Than
|-
|04
|ATOM_COND_BELOWOREQUAL
|<=
|-
|05
|ATOM_COND_ABOVEOREQUAL
|>=
|-
|06
|ATOM_COND_NOTEQUAL
|Not Equal
|}

== See Also ==
=== Articles ===
* [[Atombios Bytecode]]
* [[AMD Atombios Commands]]

=== External Links ===
* https://community.amd.com/thread/115957

[[Category:Video]]
[[Category:Firmware]]
````

<a id="w004258"></a>

## W004258 — User:Valou3433

Bron-ID: W004258
Namespace: User (2)
Revisie: 26672 | Datum: 2021-11-12T08:01:30Z
Bron: https://wiki.osdev.org/User:Valou3433
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AValou3433&oldid=26672

````text
= '''Valou3433''' =


Hello !
I'm Valentin Haudiquet. I was born in December 8 of 2000.

I started developing operating systems with 'VK', my first kernel (for x86_32) in 2017/2018.
You can look at the source code here : [https://github.com/vhaudiquet/vk GITHUB/VK]

I am now developing a more capable kernel, supporting multiple architectures.

I also have a [https://valou3433.fr website], that i use as a blog (in French).
````

<a id="w004259"></a>

## W004259 — User:Iaz3

Bron-ID: W004259
Namespace: User (2)
Revisie: 22152 | Datum: 2018-02-17T22:56:03Z
Bron: https://wiki.osdev.org/User:Iaz3
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AIaz3&oldid=22152

````text
I don't do much.
````

<a id="w004260"></a>

## W004260 — User talk:Glauxosdever

Bron-ID: W004260
Namespace: User talk (3)
Revisie: 22202 | Datum: 2018-03-09T21:14:41Z
Bron: https://wiki.osdev.org/User_talk:Glauxosdever
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AGlauxosdever&oldid=22202

````text

Notable projects meant to list notable projects, and not the projects you personally like. if your soul cant process it, you should not edit the wiki.  (btw you are from the floppy stuff, its pointless in 2018, that panel should list the sizes instead)
--[[User:Geri|Geri]] 14:21, 9 March 2018 (CST)

* I'll admit it wasn't a good move to do that, and that's why I won't revert your edit this time, and I apologise for removing your edits the first time. To be clear, I don't have a problem with Dawn itself (and I think it actually is quite a bit notable), it's mostly how it's marketed by you. You say, without any standing proof, that SUBLEQ is better than conventional CISC or RISC architectures. Granted, SUBLEQ CPUs may be cheaper/more energy efficient/smaller/etc, but it's like comparing a bike to an airplane: a bike is cheaper/more energy efficient/smaller than an airplane, but an airplane is a lot faster than a bike and actually useful for going into trips of hundreds of kilometers/miles. Same for SUBLEQ vs CISC/RISC. A SUBLEQ CPU will need a lot of instructions for something a CISC or RISC machine would do in only a couple of them. And I'm worried about newbies falling into the SUBLEQ trap. I really suggest you stop acting as if SUBLEQ was a panacea, but yes, Dawn is quite notable, whatever it runs on. I'll let moderators handle the situation. (As for sizes, you really wouldn't like that as SUBLEQ code has a '''very''' low density, thus is quite sizable). [[User:Glauxosdever|Glauxosdever]] 14:52, 9 March 2018 (CST)
** yes. bike, airplane, car, zeppelin, roller. i make zeppelin, not jet engine --[[User:Geri|Geri]] 15:14, 9 March 2018 (CST)

* Notability is about external knowledge. This is not an externally known operating system and I cannot find any third-party original reporting on it. Not notable, delete. --[[User:Kazinsal|Kazinsal]] 15:00, 9 March 2018 (CST)    
** for example http://esoteric.codes/post/168290778358/a-programming-language-with-only-one-command-and --[[User:Geri|Geri]] 15:14, 9 March 2018 (CST)
````

<a id="w004261"></a>

## W004261 — Ada Kalinda

Bron-ID: W004261
Namespace: Main (0)
Revisie: 30501 | Datum: 2026-04-20T23:22:47Z
Bron: https://wiki.osdev.org/Ada_Kalinda
Revisie-URL: https://wiki.osdev.org/index.php?title=Ada_Kalinda&oldid=30501

````text
== Brief history and initial objectives ==


This project dates back from the early 1990s.

A few years earlier computer science had seen two marked evolutions : the Macintosh and the Ada language. Both were, each in its own domain, new worlds.

Strangely, those two worlds were quickly disappearing in their initial form:
* The purity of black and white Macintosh was rapidly superseded by more powerful various machines, color screens, new os versions.
* Ada 83 stayed confined to some specialized area of high reliability software and twelve years later was "augmented" with Ada 95. This brought rare useful additions and a bunch of new possibilities which were more prone to degenerate the initial idea and the occasion of an added compiler complexity.

Being a Pascalist with a strong allergy to [[C]], Ada 83 was a real wonder and a compiler performance at the time. The inside Macintosh four volumes documentation were a thorough description of the Mac Plus system.
I then decided to see if rewriting a Mac Plus OS with Ada 83 was possible and brought some clarity and ease of software maintenance.

But at the time, Ada cross compiler were rare and costly, so that a rewriting of the OS began in fact in CodeWarrior Pascal. Several machines were used, as the dual PPC/i86 power pc, but development mainly progressed on a Mac LCIII with CodeWarriow Windows MetroNub to target an x86 pc. The project advanced up to a functional graphical interface and a start of usable file system. But the Pascal code was becoming very heavy to maintain for a single person and an interruption of the development showed that reinsertion in the project was rather difficult and the Pascal code not so easy to comprehend or read after some years.

Time passing, CodeWarrior tools and machines changing, cross development became more difficult and tentative of rewriting with gcc C was done which was quickly abandoned being an horror story in maintenance and reinsertion in source code.

Then Gnat appeared, and being a new Linux user I saw that version 3.15 built on GCC 2.81 could be used with its pragma "No_Run_Time" as a sort of "half cross-compiler" x86 Linux hosted and generating x86 binary executable on a bare x86 pc.

A few trials were positive but development was not using a gdb stub and disquettes transfer for execution tests were cumbersome.
The boot code was written in this configuration with much pain and beeps and all sort of debugging tricks, those who tried know. Later I decided to use the Bochs emulator to test the binary, a vast improvement indeed.

The Gnat 3.15 being an Ada 95 compiler, and the appeal of novelty aiding, I took the old Code Warrior Pascal source an decided to rewrite with new objects programming possibilities. Rather than an improvement, it resembled a catastrophe, it proved that the new Ada was much more prone to spaghetti code than the Ada 83 and that the best option was to adhere as much as possible to Ada 83 programming which imposed more constraints but resulted in better structured and simpler code, so that return and reinsertion in source code was easier.

Finally, after many years of instructive trials, I converted the initial Pascal code in something as close as possible to Ada 83, with only the "regrettable forced use" of child packages. I would have been very glad to be an Ada 83 purist, but child packages really simplify the Macintosh like OS structure of Kalinda. Only with child packages is it possible to write a complete interface to system services and complete this universal application interface layer with a sub layer which uses the same partially hidden data structures and implements the system itself. Pure Ada 83 would force to duplicate source code for data structures introducing potential compatibility problems and complicating the maintenance of the software.

== Present state of the affair : Kalinda 2018 ==

It seems relatively few people are interested in using Ada 83 or (Ada 95 83 styled) in present developments, and OS development in Ada is rather rare.  [https://wiki.osdev.org/Ada_Bare_bones Ada Bare Bones] project is one of the rare ones. Kalinda experiment could be of some use for Ada OSers as it has a peculiar ancient development path.

Kalinda 2018 uses the following development system :

* A recent gcc toolchain with gnat Ada compiler built on Ubuntu for host Linux X86-64 and targeted to i486-unknown-elf (gcc 7.3.0 with binutils 2.30).

* The NASM assembler used for historical reasons for assembling the boot block and the "amorce" which prepares flat memory, sets protected mode and jumps to Ada code.

* Two versions of the Bochs emulator, one compiled with internal debugger the other with gdb stub, executing for cpu: model=pentium.

* Geany editor/project file for launching compile/bind/link/make/run scripts.

The gcc/gnat compiler is configured with --enable-languages=ada --disable-libquadmath --disable-libada --disable-libssp and subsequently stripped to the strict minimum : the gcc / gnat_1, as, ld, gnatbind, gdb. All useless libraries/includes have been removed. All compilations are done without standard adalib/adainclude but with a very small set of custom RTS files.
The aim is to have the minimal gcc toolchain : 7 files ; 196 Mb (not so bad).

The principle of the whole thing is relatively simple :

Every Ada module is compiled in an object .o with suitable restrictions in a gnat.adc pragma configuration file so that Gnat RTS is not used and some rare compiler calls caught by the small custom "RTS".

The project is bound (elabs sequencing and program launch) in an ada_main with -nostdlib -nostdinc --RTS=... and compiled in .o.

The linker takes all .o planned in .ld script, with the boot_block.bin placed in head position, and produces a binary file the size of a 1.44 disquette which can be directly booted as a hard disk by Bochs. This process take the place of a ROM filling.

The boot block has the simple task of copying 9 64Kb segments (576Kb) from the binary file/hard_disg_img in the low pc memory space between 0x10000 and the 0xA0000 video buffer. This memory space contains system services which were in Macintosh ROM.
This means that Kalinda system "ROM" code is limited to 576 Kb (very happy with it, Mac plus ROM was 128 Kb).

The system then uses RAM memory after the 1Mb pc low memory (Mac like memory manager "zoning" begins at 0x10_0000).

== Git'ed source code ==

For curious OSers, Kalinda was CVS hosted since 2002 on SourceForge and Kalinda 2018 code has been Git'ed [https://sourceforge.net/p/sx-ada-kalinda/2018/ci/master/tree/ there]
The pieces of the development system are available in the "Files" section of the project (except Bochs run system)

To avoid unpleasant experience, note that the source code uses 20 spaces tabulations, so tell your favorite editor to set tabs every 20 spaces (in Geany config section "indentation" : indent_width=20).

After experimenting various text formatting, I settled on "horizontal" layout so that modules routines are all visible at first glance (as much as possible). In such a layout, tabs are nearly compulsory.
I do not like gnat formatting which is too vertical and lengthy.

Printing with enscript / Oxygen font forge derived 5 points bold font gives 160 columns of small but legible characters ans most interfaces modules specs take 2 pages only (private part on verso). Also enscript section 2 separates the comments which are after column 160.

(In march 2018, the Macintosh desk and menu bar are drawn, and menus unroll, but file system is not operative because of floppy driver rewriting, so double click on the trash crash!)

[[Category:Operating Systems]]
````

<a id="w004262"></a>

## W004262 — User:Gwoplock/EHCI

Bron-ID: W004262
Namespace: User (2)
Revisie: 22249 | Datum: 2018-04-03T18:54:08Z
Bron: https://wiki.osdev.org/User:Gwoplock/EHCI
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGwoplock%2FEHCI&oldid=22249

````text
EHCI (Enhanced Host Controller Interface) is the ''Single'' method of interfacing with USB 2.0. This makes life a lot easier than with USB 1.0, where 2 incompatible standards both survived. This is because the USB-IF insisted on a single implementation, produced by Intel.

== Interface ==

On a PC you will normally find the EHCI USB controller on the PCI bus - in fact it is the only access method specified in the specification. USB 2.0 supports interfacing with USB 1.0 devices. However, EHCI is NOT expected to support them. Instead, you will find an UHCI or OHCI ''companion controller''. Therefore you must implement [[OHCI]] and [[UHCI]] as well. Note there are NOT companions of both types, it is one type or another.

The EHCI controller will always have a PCI Class ID of 0x0C, a Subclass ID of 0x03, and an Interface value of 0x20.  These values can be used to find the configuration space of the controller, which contains the memory mapped address of the EHCI registers.  The address information can be found in the BAR0 PCI configuration space register.

== Capability Registers ==

The capability registers can be found in memory at the address specified in the PCI configuration space BAR0 register.

{| {{wikitable}}
! Offset (Hex)
! Name
! Description
|-
| 00 || CAPLENGTH || Capability Register Length
|-
| 01 || Reserved ||
|-
| 02 || HCIVERSION || Interface Version Number (BCD)
|-
| 04 || HCSPARAMS || Structural Parameters
|-
| 08 || HCCPARAMS || Capability Parameters
|-
| 0C || HCSP-PORTROUTE || Companion Port Route Description
|-
|}

== Operation Registers ==

The operation registers can be found after the capability register area in memory.  Add the Capability Register Length value above to the BAR0 address to find the operation register base address.

{| {{wikitable}}
! Offset (Hex)
! Name
! Description
|-
| 00 || USBCMD || USB Command
|-
| 04 || USBSTS || USB Status
|-
| 08 || USBINTR || USB Interrupt Enable
|-
| 0C || FRINDEX || USB Frame Index
|-
| 10 || CTRLDSSEGMENT || 4G Segment Selector
|-
| 14 || PERIODICLISTBASE || Frame List Base Address
|-
| 18 || ASYNCLISTADDR || Next Asynchronous List Address
|-
| 40 || CONFIGFLAG || Configured Flag Register
|-
| 44 || PORTSC[1-N_PORTS] || Port Status/Control Register
|-
|}

=== USB Command Register ===

{| {{wikitable}}
! Bits
! Name
! Description
|-
| 31-24 || Reserved ||
|-
| 23-16 || Interrupt Threshold || Number of micro frames to process between interrupts
|-
| 15-12 || Reserved ||
|-
| 11 || Async Schedule Park Mode Enable ||
|-
| 10 || Reserved ||
|-
| 9-8 || Async Schedule Park Mode Count ||
|-
| 7 || Light Host Controller Reset || Resets the controller without affecting connected devices
|-
| 6 || Interrupt On Async Advance Doorbell || Allows software to trigger interrupt
|-
| 5 || Async Schedule Enable ||
|-
| 4 || Periodic Schedule Enable ||
|-
| 3-2 || Programmable Frame List Size ||
|-
| 1 || Host Controller Reset ||
|-
| 0 || Run ||
|-
|}


=== USB Status Register ===

{| {{wikitable}}
! Bits
! Name
! Description
|-
| 31-16 || Reserved ||
|-
| 15 || Async Schedule Status ||
|-
| 14 || Periodic Schedule Status ||
|-
| 13 || Reclamation ||
|-
| 12 || Halted ||
|-
| 11-6 || Reserved ||
|-
| 5 || Doorbell Interrupt ||
|-
| 4 || Host System Error ||
|-
| 3 || Frame List Rollover ||
|-
| 2 || Port Change Detect ||
|-
| 1 || USB Error Interrupt ||
|-
| 0 || USB Transfer Interrupt ||
|-
|}


=== USB Interrupt Enable Register ===

{| {{wikitable}}
! Bits
! Name
! Description
|-
| 31-6 || Reserved ||
|-
| 5 || Async Advance Interrupt Enable ||
|-
| 4 || Host System Error Interrupt Enable ||
|-
| 3 || Frame List Rollover Interrupt Enable ||
|-
| 2 || Port Change Interrupt Enable ||
|-
| 1 || USB Error Interrupt Enable ||
|-
| 0 || USB Transfer Interrupt Enable ||
|-
|}



=== Port Status/Control Register ===

{| {{wikitable}}
! Bits
! Name
! Description
|-
| 31-23 || Reserved ||
|-
| 22 || Wake On Overcurrent Enable ||
|-
| 21 || Wake On Disconnect Enable ||
|-
| 20 || Wake On Connect Enable ||
|-
| 19-16 || Port Test Control ||
|-
| 15-14 || Port Indicator Control || 0 = Off, 1 = Amber, 2 = Green
|-
| 13 || Companion Port Control || 0 = Local, 1 = Companion Host Controller
|-
| 12|| Port Power ||
|-
| 11-10 || Line Status ||
|-
| 9 || Reserved ||
|-
| 8 || Port Reset ||
|-
| 7 || Suspend ||
|-
| 6 || Force Port Resume ||
|-
| 5 || Overcurrent Change ||
|-
| 4 || Overcurrent ||
|-
| 3 || Port Enabled Changed ||
|-
| 2 || Port Enabled ||
|-
| 1 || Connect Changed ||
|-
| 0 || Connected ||
|-
|}

== Asynchronous Queue List ==

The asynchronous queue list is a ring of Queue Head records that point to the next Queue Head in the ring, and the first Transmit Descriptor in the queue.  As long as the Asynchronous Schedule flag is set in the Command register, the EHCI controller will start with the first Queue Head located at the address in the Next Asynchronous List Address register, and process all Queue Heads until it encounters a Queue Head with the Reclaim Head flag set.  The reclaim head flag is not checked on the very first Queue Head record, so you can set it's reclaim head flag, and the controller will process all Queue Heads until it reaches the first Queue Head again.

The Queue Head contains a copy of the current TD that the controller uses to process each Transfer Descriptor.  This area should initially contain the first TD of the queue in the Next TD field of the TD working area.

=== Queue Head ===

{| {{wikitable}}
! Offset (Hex)
! Name
! Description
|-
| 00 || Horizontal Link Pointer ||
|-
| 04 || Endpoint Characteristics ||
|-
| 08 || Endpoint Capabilities ||
|-
| 0C || Current TD Address ||
|-
| 10 || Current TD Working Area || This area contains as copy of the current TD data.
|-
|}

=== Horizontal Link Pointer ===

{| {{wikitable}}
! Bits
! Name
! Description
|-
| 31-5 || Next Queue Head || Address of the next Queue Head in the ring
|-
| 4-3 || Reserved ||
|-
| 2-1 || Next Queue Type || 0 = Isochronous TD, 1 = Queue Head, 2 = Split Transaction Isochronous TD, 3 = Frame Span Traversal Node
|-
| 0 || Terminate || Set if this is the last Queue Head in a Periodic List.  Not used for Asynchronous List.
|-
|}

=== Endpoint Characteristics ===

{| {{wikitable}}
! Bits
! Name
! Description
|-
| 31-28 || NAK Reload||
|-
| 27 || Control Endpoint || Not used for High Speed devices
|-
| 26-16 || Maximum Packet Length|| 
|-
| 15 || Head of Reclamation List || Set if this is the first Queue Head in an Asynchronous List
|-
| 14 || Data Toggle Control || Set if data toggle should use value from TD
|-
| 13-12 || Endpoint Speed || 0 = Full Speed, 1 = Low Speed, 2 = High Speed
|-
| 11-8 || Endpoint Number || 
|-
| 7 || Inactivate || Only used in Periodic List
|-
| 6-0 || Device Address || 
|-
|}

=== Endpoint Capabilities ===

{| {{wikitable}}
! Bits
! Name
! Description
|-
| 31-30 || High Bandwidth Pipe Multiplier || Must be greater than zero
|-
| 29-23 || Port Number || Used for split transactions
|-
| 22-16 || Hub Address || Used for split transactions
|-
| 15-8 || Split Completion Mask || Used for split transactions
|-
| 7-0 || Interrupt Schedule Mask || Used for split transactions
|-
|}





== See Also ==
=== Articles ===
* [[USB]]
* [[OHCI]]
* [[UHCI]]
=== External Links ===
* [http://www.intel.com/technology/usb/download/ehci-r10.pdf EHCI Specification] + [http://download.intel.com/technology/usb/EHCIv1-1_Addendum.pdf Addendum]

[[Category:USB]]
[[de:Enhanced_Host_Controller_Interface]]
````

<a id="w004263"></a>

## W004263 — BMFS

Bron-ID: W004263
Namespace: Main (0)
Revisie: 30218 | Datum: 2026-03-15T23:49:16Z
Bron: https://wiki.osdev.org/BMFS
Revisie-URL: https://wiki.osdev.org/index.php?title=BMFS&oldid=30218

````text
== BMFS ==

BMFS is a simple file system in active development that supports regular files and directories.

=== Disclaimer ===

I am a contributor of BMFS. There's plenty of other file system implementations that could be used in an OS project.

BMFS, while it has existed for some time, is considered a new project. Be wary of bugs.

== A Little History ==

BMFS was a file system initially designed by Ian Seyler for the BareMetal-OS project. Since then, it has been put into a separate project and the development continued there.

In it's initial design, it only supported a root directory with a maximum of 64 files. The idea was to have a simple file system that supported just enough of a file system for a database program to use.

The development continued and the design changed a bit, and now it supports sub-directories, file permissions, user and group IDs, and more.

It also comes with a user space program to construct file systems and examine their contents, and a [[FUSE]] binding to use BMFS on Linux systems.

The project is now being constantly improved, and one of its design goals is to be used in any other operating system.

== Building BMFS ==

While using the instructions from the project is the most reliable way to build the project, this should also help build the project.

Starting by cloning the project using git, and entering the directory.
<syntaxhighlight lang="bash">
git clone https://github.com/ReturnInfinity/BMFS
cd BMFS
</syntaxhighlight>

Then use GNU Make to build the project, specifying that you are cross compiling using the CONFIG variable.
<syntaxhighlight lang="bash">
make CONFIG=cross
</syntaxhighlight>

The cross compiler here defaults to x86_64-none-elf-gcc. If you would like to use a different target, use the CROSS_COMPILE variable like this:
<syntaxhighlight lang="bash">
make CONFIG=cross CROSS_COMPILE=aarch64-none-elf-
</syntaxhighlight>

Once the project is built, you can install it in a location of your choosing by using the PREFIX variable, and the install target.

<syntaxhighlight lang="bash">
make CONFIG=cross install PREFIX=/home/taylor/.local
</syntaxhighlight>

Now, you have the library libbmfs.a installed at /home/taylor/.local/lib and the headers installed at /home/taylor/include

That means you'll have to pass those paths to GCC, by doing something like this:

<syntaxhighlight lang="bash">
gcc -I /home/taylor/.local/include -L /home/taylor/.local/lib
</syntaxhighlight>

You can also install it to /usr/local, to avoid having to do that.

=== Initializing the File System ===

Once the project is built, and your build system has the information to use the library, you can start writing code.

The header bmfs/bmfs.h is there for convenience to include all the headers used in the project.

Start by including that header in your code.

<syntaxhighlight lang="c">
#include <bmfs/bmfs.h>

void open_file(const char *path) {
    /* Do stuff here */
}
</syntaxhighlight>

Once that's done, you'll need to tell BMFS how to read the disk that contains the file system.

You can do that using the BMFSDisk structure.

<syntaxhighlight lang="c">
int my_disk_read(void *disk_data, void *buf, bmfs_uint64 len, bmfs_uint64 *read_len);

int my_disk_seek(void *disk_data, bmfs_uint64_t pos, int whence);

void open_file(const char *path) {

    struct BMFSDisk disk;

    bmfs_disk_init(&disk);
    disk.seek = my_disk_seek;
    disk.read = my_disk_read;
}

/* Implement my_disk_read and my_disk_seek */
</syntaxhighlight>

Now that the disk is initialized, you can initialize the file system structure and read the file system contents.

<syntaxhighlight lang="c">
int open_file(const char *path) {

    /* Initialize disk here */

    struct BMFS bmfs;

    bmfs_init(&bmfs);

    bmfs_set_disk(&bmfs, &disk);

    int err = bmfs_import(&bmfs);
    if (err != 0) {
        kprintf("Failed to import BMFS file system.\n");
        return -1;
    }
}
</syntaxhighlight>
=== Opening a File ===

With the disk and the file system initialized, you can open the file.
<syntaxhighlight lang="c">
int show_file(const char *path) {

    /* Initialize disk here. */

    /* Initialize file system header. */

    struct BMFSFile file;

    bmfs_file_init(&file);

    err = bmfs_open_file(&bmfs, &file, path);
    if (err == BMFS_ENOENT) {
        kprintf("Entry '%s' does not exist.\n", path);
    } else if (err == BMFS_EISDIR) {
        kprintf("Entry '%s' is a directory.\n", path);
    } else if (err != 0) {
        kprintf("Failed to open '%s'.\n", path);
        return -1;
    }

    bmfs_file_set_mode(&file, BMFS_MODE_READ);

    char buf[512];

    while (bmfs_file_eof(&file)) {

        bmfs_uint64 read_count = 0;

        err = bmfs_file_read(&file, buf, 512, &read_count);
        if (err != 0)
            break;

        my_print_function(buf, read_count);
    }
}
</syntaxhighlight>

=== Opening a Directory ===

Opening a directory is similar to opening a file.

You'll have to initialize the file system the same as you did for the file.

<syntaxhighlight lang="c">
int list_dir(const char *path) {

    /* Initialize the file system here. */

    struct BMFSDir dir;

    bmfs_dir_init(&dir);

    int err = bmfs_open_dir(&bmfs, &dir, path);
    if (err == BMFS_ENOTDIR) {
        kprintf("Entry '%s' is not a directory.\n", path);
        return -1;
    } else if (err == BMFS_ENOENT) {
        kprintf("Directory '%s' does not exist.\n", path);
        return -1;
    } else if (err != 0) {
        kprintf("Failed to open directory '%s'.\n", path);
        return -1;
    }
    
    for (;;) {
        const struct BMFSEntry *entry = bmfs_dir_next(&dir);
        if (entry == BMFS_NULL)
            break;
        
        kprintf("Entry: %s\n", entry->Name);
    }
}
</syntaxhighlight>

=== Creating a File System on the Development Machine ===

Sometimes, in creating an operating system, a file system has to be made in the development machine to transfer system files (such as programs or shared libraries).

For that reason, the BMFS utility program was designed. It allows a user to create a file system on a disk image and read and write to the file system.

To build the utility program, just use the standard 'make' and 'make install' commands in the project directory.
<syntaxhighlight lang="bash">
git clone https://github.com/ReturnInfinity/BMFS
cd BMFS
make
sudo make install
</syntaxhighlight>

Create a disk image called 'bmfs.img' using the 'init' command.
<syntaxhighlight lang="bash">
bmfs init
</syntaxhighlight>

To use a different name, use the `--disk` option.
<syntaxhighlight lang="bash">
bmfs --disk my-file-system.img init
</syntaxhighlight>

Using `bmfs.img` is nice, because all the commands default to this name.

The rest of this tutorial assumes you have used the name 'bmfs.img'.

If this is not the case, just specify the name using the '--disk' option for each command.

To create a directory, us the 'mkdir' command.
<syntaxhighlight lang="bash">
bmfs mkdir /drivers
</syntaxhighlight>

To transfer a file from the host system to the image, us the 'cp' command.
<syntaxhighlight lang="bash">
bmfs cp ahci-driver.sys /drivers/ahci.sys
</syntaxhighlight>

To list the contents of a directory, us the 'ls' command.

<syntaxhighlight lang="bash">
bmfs ls /drivers
</syntaxhighlight>

=== Reporting Bugs ===

If you think there is a bug in the file system implementation, report it on the project issue tracker [https://github.com/ReturnInfinity/BMFS/issues].

=== Going Further ===

The library can do even more than that and new features are added continuously.

Visit the Doxygen generated documentation [https://returninfinity.github.io/bmfs-doc/modules.html] for more information.

[[Category:Filesystems]]
````

<a id="w004264"></a>

## W004264 — User talk:Tay10r

Bron-ID: W004264
Namespace: User talk (3)
Revisie: 22258 | Datum: 2018-04-05T00:25:25Z
Bron: https://wiki.osdev.org/User_talk:Tay10r
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ATay10r&oldid=22258
Doorverwijzing: BMFS

````text
#REDIRECT [[BMFS]]
````

<a id="w004265"></a>

## W004265 — Booting Raspberry Pi 3

Bron-ID: W004265
Namespace: Main (0)
Revisie: 28051 | Datum: 2023-07-09T19:08:29Z
Bron: https://wiki.osdev.org/Booting_Raspberry_Pi_3
Revisie-URL: https://wiki.osdev.org/index.php?title=Booting_Raspberry_Pi_3&oldid=28051

````text
{{Tone}}
{{Spelling}}
{{FirstPerson}}
{{Rating|1}}
This is a tutorial on bare-metal OS development on the Raspberry Pi 3 (RPi 3).

'''THIS IS NOT A PRACTICAL IMPLEMENTATION NOR A REFERENCE! It'S SIMPLY SOMETHING TO BE BUILT UP ON!'''

== Preamble ==
This article assumes that you have all of the necessary materials needed to begin developing on your respective hardware. Other assumptions are that you are comfortable with low-level programming using C/C++, GNU Make buildsystem, and that you are comfortable enough about OS development and hardware to not be confused.. Such details are not in the scope of this article and must be done before proceeding to the next steps. 

This article is also intended for beginner users who want a minimal solution for creating/starting an operating system, although some insight can be derived for the advanced audience. 

== Preperations ==
====Cross Compiler====

A bare-metal AArch64 toolchain is available on the AUR for installation. The links are provided here as well as a download link from the publisher for Windows users.

https://www.linaro.org/downloads/

https://aur.archlinux.org/packages/aarch64-elf-gcc-linaro-bin/

https://aur.archlinux.org/packages/aarch64-elf-newlib-linaro-bin/

====Firmware====

Firmware is required for the RPi 3 to boot properly. The firmware needed can be found at these link:

https://github.com/raspberrypi/firmware/blob/master/boot/start.elf

https://github.com/raspberrypi/firmware/blob/master/boot/bootcode.bin

There is also another file required to boot properly. A config.txt file must be supplied to provide configuration details for the device and the OS. Here are the only entries you need:

<source lang="ini">
boot_delay=1
force_turbo=1
enable_uart=1
</source>

More details about booting and configuration can be found at thse links.

https://elinux.org/RPi_Software#Overview

https://raspberrypi.stackexchange.com/a/10595

https://www.raspberrypi.org/documentation/configuration/config-txt/

https://wiki.beyondlogic.org/index.php?title=Understanding_RaspberryPi_Boot_Process (although a bit old, is still useful)

When loading the OS onto an SD card, these file must be included in the boot partition.

== Overview ==

By now you should have your cross compiler set up. The compiler binaries have the same name as they usually would with "aarch64-elf-" prefixing them (e.x. aarch64-elf-gcc). For this example, three files are used:

*linker.ld - Linker script
*start.S - Setup the environment and call the kernel
*kernel.c - Kernel entry and use

This article is written so that segments of the code will be presented with an explaination, with the full code in the Conclusion section.

== Linking the Kernel ==

Linker script allows us to segment, organize, and align our kernel image. More information about linker scripts can be found osdev's [https://wiki.osdev.org/Linker_Scripts Linker Scripts].

'''The start address and alignment are for the RPi3. Use whatever is applicable to you.'''
<source lang="c">
SECTIONS
{
    . = 0x80000;
 
    .text : { 
        KEEP(*(.boot)); 
        *(.text*) 
    } 
 
    .rodata ALIGN(16) : { 
        *(.rodata*) 
    } 
 
    .data ALIGN(16) : { 
        *(.data*) 
    } 
 
    .bss (NOLOAD) : { 
        *(.bss*) 
    } 
}
</source>

== Booting the Kernel ==

For the RPi 3, after the start.elf finishes, the CPU and SRAM have been enabled and control is given to the kernel image. But there are a few things we must do in order to set up a basic C environment. This is known as a bootstrap stage which initializes our OS and programming environment on startup and hands control to the kernel. Keep in mind that this is for a minimalistic development environment on a single core.

<source lang="asm">
.section .boot

.global _start

_start:
        mrs        x4, mpidr_el1
        and        x4, x4, #3
        cbz        x4, _init
0:      wfe
        b            0b
</source>

The mrs instruction loads data from specialty registers into a standard register. This special register in question is called the MPIDR or the Multiprocessor Affinity Register.  Caring only about the first two bits only we use a bitwise and to weed out any core ID's that aren't zero. Then we apply the cbz instruction, which is a shorthand instruction for comparing the x4 register with 0 and branching if they are equal, i.e. core 0.

If the core ID is 0, we branch to another section that will initialize the environment and will hand control over to the kernel. For now, the C function for our kernel will simply by kern_main.

<source lang="asm">
_init:  ldr     x4, =_start
        mov     sp, x4
        b       kern_main
</source>

Note that with the latest firmware, this is not necessary, as only the primary core runs. It is enough to set up stack and branch to kern_main.

<source lang="asm">
// for latest firmware, after 2020-01-01
.section .boot

.global _start

_start: ldr     x4, =_start
        mov     sp, x4
        b       kern_main
</source>

== Writing the Kernel ==

The kernel is as simple as creating a kern_main function for the bootstrap stage to transfer control to. You notice that in start.S we have the program branch into the function. All we need to do is provide one.

<source lang="c">

#include <stdbool.h>

void
kern_main(void)
{
    while (true);
}
</source>

== Conclusion ==

At the time of writing, QEMU does not yet emulate the RPi 3. The only way to test is to hook up the RPi 3 to an HDMI monitor. If start.elf was successfully able to run, then a square with interpolated colors will appear on the screen. That paird with the ACT (green LED) flashing even after the square shows means that it most likely has loaded the kernel and started executing. As this lacks a UART example, it will be difficult to get concrete proof that the kernel has booted.

== See Also ==

=== Articles ===
* [[Raspberry Pi Bare Bones]] - a much more detailed introduction to booting on Raspberry Pi

[[Category:Raspberry Pi]]
[[Category:Booting]]
````

<a id="w004266"></a>

## W004266 — File:TravorOS 20180422.png

Bron-ID: W004266
Namespace: File (6)
Revisie: 22346 | Datum: 2018-04-22T19:32:12Z
Bron: https://wiki.osdev.org/File:TravorOS_20180422.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ATravorOS_20180422.png&oldid=22346

````text
Appearance of TravorOS in April 22, 2018
````

<a id="w004267"></a>

## W004267 — AArch64

Bron-ID: W004267
Namespace: Main (0)
Revisie: 30864 | Datum: 2026-07-07T17:29:26Z
Bron: https://wiki.osdev.org/AArch64
Revisie-URL: https://wiki.osdev.org/index.php?title=AArch64&oldid=30864

````text
{{In_Progress}}
'''AArch64''' is the 64-bit version of the [[ARM]] architecture, as published by ARM Holdings originally in 2011 as part of the '''ARMv8''' architecture. It has gained much headway in recent years, being used in many mobile phones. It also is gaining market share in the server world, due to performance advancements and its power management. It also is starting to gain steam in the personal computer world, as Apple in 2020 started using it for most of its Macs. This chip, the Apple M1 chip, has performance comparable to Intel and AMD processors. 

The primary barrier to OSDev on AArch64 is the lack of standardization among peripherals; even once you've settled on a board to target, you may not have much documentation on how to program it.

The situation is improving, as [[ARM SystemReady]] is a fairly new standard that aims to standardize ARM hardware. It allows for OSes like Windows, Linux, and the BSDs to be ran out of box on compliant hardware. This means that implementing AArch64 support in your hobby OS may be a good thing, as its popularity and usage continue to expand.

[[UEFI]] implementations are also becoming more common for AArch64, an example is the [https://github.com/tianocore/edk2-platforms/tree/master/Platform/RaspberryPi/RPi4 Tiano Core EDK2] port for the Raspberry Pi 4. This has allowed operating systems to utilize standardized UEFI calls to communicate to the hardware in a much more generic matter. The [[Limine]] bootloader exclusively uses UEFI on AArch64 platforms.

== New Features ==
AArch64 is built around a new instruction set, called A64. It was first introduced with ARMv8-A. Features of the base A64 set include:
* 31 64-bit General purpose registers, labeled X0 - X30.
* A dedicated zero register or Stack pointer register (instruction dependent).
* Program counter is no longer directly accessible.
* Instructions which are still 32 bits long, with most being the same as A32, which can accept either 32 or 64 bit arguments, with addresses assumed to be 64 bit.
* Advanced SIMD (NEON), supporting 32 x 128 bit registers (up from 16), fully IEEE 754 compliant.

== Compiler Support ==
As of 2024, Both [[GCC]] and [[LLVM]] have first class aarch64 support available in their stable branches.

== Emulation Support ==
[[QEMU]] Supports a variety of AArch64 based systems, including rudimentary support for the raspberry pi 4 and 5 in the latest releases.

== See Also ==
=== Articles ===
* [[AArch64 Exceptions]]
* [[Booting Raspberry Pi 3]]
* [[QEMU AArch64 Virt Bare Bones]]

=== External Links ===
* [https://en.wikipedia.org/wiki/AArch64 AArch64] on Wikipedia
* [https://developer.arm.com/documentation/ddi0602/2024-06/?lang=en ISA documentation] on Arm Developer
* [https://www.arm.com/technologies/neon Arm Neon] on Arm
* [https://gcc.gnu.org/onlinedocs/gcc/AArch64-Options.html GCC aarch64 options] on GNU
* [https://www.qemu.org/docs/master/system/target-arm.html QEMU Arm System] on QEMU

[[Category:ARM]]
[[Category:AArch64]]
[[Category:Instruction Set Architecture]]
````

<a id="w004268"></a>

## W004268 — Raspberry Pi 3 AArch64

Bron-ID: W004268
Namespace: Main (0)
Revisie: 26300 | Datum: 2021-08-12T22:58:56Z
Bron: https://wiki.osdev.org/Raspberry_Pi_3_AArch64
Revisie-URL: https://wiki.osdev.org/index.php?title=Raspberry_Pi_3_AArch64&oldid=26300
Doorverwijzing: Booting Raspberry Pi 3

````text
#REDIRECT [[Booting Raspberry Pi 3]]
````

<a id="w004269"></a>

## W004269 — Raspberry Pi 3

Bron-ID: W004269
Namespace: Main (0)
Revisie: 22372 | Datum: 2018-04-28T21:17:51Z
Bron: https://wiki.osdev.org/Raspberry_Pi_3
Revisie-URL: https://wiki.osdev.org/index.php?title=Raspberry_Pi_3&oldid=22372
Doorverwijzing: Booting Raspberry Pi 3

````text
#REDIRECT [[Booting Raspberry Pi 3]]
````

<a id="w004270"></a>

## W004270 — File:AquilaOS.png

Bron-ID: W004270
Namespace: File (6)
Revisie: 22380 | Datum: 2018-05-07T18:59:43Z
Bron: https://wiki.osdev.org/File:AquilaOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AAquilaOS.png&oldid=22380

````text
AquilaOS v0.0.1a running Lua 5.3.4
````

<a id="w004271"></a>

## W004271 — User:Manwar/AquilaOS

Bron-ID: W004271
Namespace: User (2)
Revisie: 22382 | Datum: 2018-05-07T19:16:06Z
Bron: https://wiki.osdev.org/User:Manwar/AquilaOS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AManwar%2FAquilaOS&oldid=22382

````text
{{Infobox Project
|name=AquilaOS

|image=AquilaOS.png
|caption=AquilaOS v0.0.1a

|developers=[[User:Manwar|M. Anwar]]

|kerneltype=Monolithic
|platform=x86
|taskmodel=Preemptive multitasking
|language=C, Assembly

|website=[http://aquilaos.com aquilaos.com]
|github=mohamed-anwar/aquila

|license=GPLv3
}}

'''AquilaOS''' is a Unix-like operating system designed to be fully POSIX compliant. It consists of a monolithic kernel and a set of user tools/utilities. AquilaOS is built mainly for learning and hence the code base is simple, however future proofing is highly anticipated in the design and most parts are made as transparent/architecture-agnostic as possible.

== History ==
AquilaOS started in 2016, following another OS that started in 2012. The original OS lacked robust design and adding more features proved to be cumbersome. So I decided to drop all hacks and workarounds and start from scratch with a more robust future-proof design.

== Design and Features ==
AquilaOS is mostly written in C with a few assembly parts when absolutely needed. It contains a kernel and a system. The following features are currently supported:
* Multitasking and Multi-threading using POSIX threads
* Sessions, process groups and job control
* Virtual file system (VFS) with support for initramfs, tmpfs, devfs, devpts, procfs and ext2
* Devices subsystem (KDev) using devices files with major/minor numbers
* Memory management subsystem (with demand paging and copy-on-write)
* Framebuffer based terminal (with wallpaper)
* AqBox: several UNIX utilities in one binary. (similar to BusyBox)
````

<a id="w004274"></a>

## W004274 — Lua

Bron-ID: W004274
Namespace: Main (0)
Revisie: 30247 | Datum: 2026-03-16T15:47:37Z
Bron: https://wiki.osdev.org/Lua
Revisie-URL: https://wiki.osdev.org/index.php?title=Lua&oldid=30247

````text
{{TutorialTone}}
Lua is a lightweight scripting language written in ANSI [[C]], designed to be easy to port and embed. Being smaller than [[Python]], it is also easier to port.

== Porting ==

Download the source code from here: https://www.lua.org/download.html
Then, modify src/Makefile (not the Makefile at the root).

Change the variable CC to the target OS [[GCC Cross-Compiler|cross compiler]], e.g. 

<syntaxhighlight lang="make">
CC = x86_64-elf-gcc -std=gnu99
</syntaxhighlight>

Depending on the target OS level of POSIX-ness, include and library directories maybe have to be changed.

Lua depends on the following C standard library functions. Some of them can be just stubs (like the time-related functions).

<syntaxhighlight lang="c">
int isalnum(int c);
int iscntrl(int c);
int isgraph(int c);
int islower(int c);
int isprint(int c);
int ispunct(int c);
int isupper(int c);
int isxdigit(int c);
__attribute__((noreturn)) void abort();
double acos(double x);
double asin(double x);
double atan2(double y, double x);
double ceil(double x);
void clearerr(FILE *stream);
clock_t clock();
double cos(double x);
double cosh(double x);
double difftime(time_t t1, time_t t2);
void exit(int status);
double exp(double x);
int fclose(FILE *stream);
int feof(FILE *stream);
int ferror(FILE *stream);
int fflush(FILE *stream);
char *fgets(char *s, int size, FILE *stream);
double floor(double x);
double fmod(double x, double y);
FILE *fopen(const char *path, const char *mode);
int fprintf(FILE *stream, const char *format, ...);
int fputs(const char *s, FILE *stream);
size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream);
FILE *freopen(const char *path, const char *mode, FILE *stream);
double frexp(double x, int *exp);
int fseek(FILE *stream, long offset, int whence);
long ftell(FILE *stream);
size_t fwrite(const void *ptr, size_t size, size_t nmemb, FILE *stream);
int getc(FILE *stream);
struct tm *gmtime(const time_t *time);
double ldexp(double x, int exp);
struct tm *localtime(const time_t *time);
double log(double x);
double log10(double x);
double log2(double x);
time_t mktime(struct tm *tm);
double pow(double x, double y);
int remove(const char *path);
int rename(const char *oldPath, const char *newPath);
char *setlocale(int category, const char *locale);
int setvbuf(FILE *stream, char *buf, int mode, size_t size);
void (*signal(int sig, void (*func)(int)))(int);
double sin(double x);
double sinh(double x);
int snprintf(char *str, size_t size, const char *format, ...);
int sprintf(char *str, const char *format, ...);
int printf(const char *format, ...);
void srand(unsigned int seed);
int strcoll(const char *s1, const char *s2);
char *strerror(int errnum);
size_t strftime(char *s, size_t max, const char *format, const struct tm *tm);
char *strpbrk(const char *s, const char *accept);
double strtod(const char *nptr, char **endptr);
int system(const char *command);
double tan(double x);
double tanh(double x);
time_t time(time_t *tloc);
FILE *tmpfile();
char *tmpnam(char *s);
void *memset(void *s, int c, size_t n);
void *memcpy(void *dest, const void *src, size_t n);
size_t strlen(const char *s);
void *malloc(size_t size);
void *calloc(size_t num, size_t size);
void *memmove(void *dest, const void *src, size_t n);
void free(void *ptr);
void *realloc(void *ptr, size_t size);
double fabs(double x);
int abs(int n);
int ifloor(double x);
int iceil(double x);
double sqrt(double x);
char *getenv(const char *name);
int strcmp(const char *s1, const char *s2);
int strcasecmp(const char *s1, const char *s2);
int strncmp(const char *s1, const char *s2, size_t n);
int strncasecmp(const char *s1, const char *s2, size_t n);
int toupper(int c);
int tolower(int c);
char *strncpy(char *dest, const char *src, size_t n);
long int strtol(const char *nptr, char **endptr, int base);
unsigned long int strtoul(const char *nptr, char **endptr, int base);
char *strcat(char *dest, const char *src);
char *strsep(char **stringp, const char *delim);
char *strstr(const char *haystack, const char *needle);
char *strrchr(const char *s, int c);
void qsort(void *base, size_t nmemb, size_t size, int (*compar)(const void *, const void *));
char *strcpy(char *dest, const char *src);
char *stpcpy(char *dest, const char *src);
char *strchr(const char *s, int c);
size_t strnlen(const char *s, size_t maxlen);
size_t strspn(const char *s, const char *accept);
size_t strcspn(const char *s, const char *reject);
int memcmp(const void *s1, const void *s2, size_t n);
void *memchr(const void *s, int c, size_t n);
int isspace(int c);
int isalpha(int c);
int isdigit(int c);
struct lconv *localeconv();
int rand();
int ungetc(int c, FILE *stream);
</syntaxhighlight>

Finally, to build it, from the root folder (not in the src folder), run the following command:

<syntaxhighlight lang="bash">
make generic
</syntaxhighlight>

If all went well, a "lua" executable should appear in the source folder.

== External Links ==
*[https://www.lua.org/ Lua Homepage]
*[https://www.lua.org/docs.html Lua Documentation]

[[Category:Languages]]
````

<a id="w004275"></a>

## W004275 — ATA Command Matrix

Bron-ID: W004275
Namespace: Main (0)
Revisie: 26274 | Datum: 2021-08-07T01:06:17Z
Bron: https://wiki.osdev.org/ATA_Command_Matrix
Revisie-URL: https://wiki.osdev.org/index.php?title=ATA_Command_Matrix&oldid=26274

````text
This information is current up to ACS-5, and includes obsolete commands from prior versions of the standard.

{| {{wikitable}}
|-
! Opcode
! Name
! Transfer
! Parameters
! Standard
|-
| 00
| NOP
| none
| 8-bit
| ATA-1 to present
|-
| 01
|
|
|
|
|-
| 02
|
|
|
|
|-
| 03
| CFA REQUEST EXTENDED ERROR CODE
| none
| 8-bit
| ATA-4 to present
|-
| 04
|
|
|
|
|-
| 05
|
|
|
|
|-
| 06
| DATA SET MANAGEMENT
| DMA
| 16-bit
| ACS-2 to present
|-
| 07
| DATA SET MANAGEMENT XL
| DMA
| 16-bit
| ACS-4 to present
|-
| 08
| DEVICE RESET
| none
| 8-bit
| ATA-3 to ACS-3
|-
| 09
|
|
|
|
|-
| 0A
|
|
|
|
|-
| 0B
| REQUEST SENSE DATA EXT
| none
| 16-bit
| ACS-2 to present
|-
| 0C
|
|
|
|
|-
| 0D
|
|
|
|
|-
| 0E
|
|
|
|
|-
| 0F
|
|
|
|
|-
| 10
| RECALIBRATE
| none
| 8-bit
| IBM PC/AT to ATA-3
|-
| 11
| RECALIBRATE
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 12
| GET PHYSICAL ELEMENT STATUS
| DMA
| 16-bit
| ACS-4 to present
|-
| 12
| RECALIBRATE
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 13
| RECALIBRATE
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 14
| RECALIBRATE
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 15
| RECALIBRATE
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 16
| RECALIBRATE
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 17
| RECALIBRATE
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 18
| RECALIBRATE
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 19
| RECALIBRATE
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 1A
| RECALIBRATE
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 1B
| RECALIBRATE
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 1C
| RECALIBRATE
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 1D
| RECALIBRATE
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 1E
| RECALIBRATE
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 1F
| RECALIBRATE
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 20
| READ SECTOR(S)
| PIO
| 8-bit
| IBM PC/AT to present
|-
| 21
| READ SECTOR(S) (without retry)
| PIO
| 8-bit
| IBM PC/AT to ATA-4
|-
| 22
| READ LONG
| PIO
| 8-bit
| IBM PC/AT to ATA-3
|-
| 23
| READ LONG (without retry)
| PIO
| 8-bit
| IBM PC/AT to ATA-3
|-
| 24
| READ SECTOR(S) EXT
| PIO
| 16-bit
| ATA-6 to present
|-
| 25
| READ DMA EXT
| DMA
| 16-bit
| ATA-6 to present
|-
| 26
| READ DMA QUEUED EXT
| queued DMA
| 16-bit
| ATA-6 to ATA8-ACS
|-
| 27
| READ NATIVE MAX ADDRESS EXT
| none
| 16-bit
| ATA-6 to ACS-2
|-
| 28
|
|
|
|
|-
| 29
| READ MULTIPLE EXT
| PIO
| 16-bit
| ATA-6 to ACS-3
|-
| 2A
| READ STREAM DMA EXT
| DMA
| 16-bit
| ATA-7 to present
|-
| 2B
| READ STREAM EXT
| PIO
| 16-bit
| ATA-7 to present
|-
| 2C
|
|
|
|
|-
| 2D
|
|
|
|
|-
| 2E
|
|
|
|
|-
| 2F
| READ LOG EXT
| PIO
| 16-bit
| ATA-6 to present
|-
| 30
| WRITE SECTOR(S)
| PIO
| 8-bit
| IBM PC/AT to present
|-
| 31
| WRITE SECTOR(S) (without retry)
| PIO
| 8-bit
| IBM PC/AT to ATA-4
|-
| 32
| WRITE LONG
| PIO
| 8-bit
| IBM PC/AT to ATA-3
|-
| 33
| WRITE LONG (without retry)
| PIO
| 8-bit
| IBM PC/AT to ATA-3
|-
| 34
| WRITE SECTOR(S) EXT
| PIO
| 16-bit
| ATA-6 to present
|-
| 35
| WRITE DMA EXT
| DMA
| 16-bit
| ATA-6 to present
|-
| 36
| WRITE DMA QUEUED EXT
| queued DMA
| 16-bit
| ATA-6 to ATA8-ACS
|-
| 37
| SET MAX ADDRESS EXT
| none
| 16-bit
| ATA-6 to ACS-2
|-
| 38
| CFA WRITE SECTORS WITHOUT ERASE
| PIO
| 8-bit
| ATA-4 to present
|-
| 39
| WRITE MULTIPLE EXT
| PIO
| 16-bit
| ATA-6 to ACS-3
|-
| 3A
| WRITE STREAM DMA EXT
| DMA
| 16-bit
| ATA-7 to present
|-
| 3B
| WRITE STREAM EXT
| PIO
| 16-bit
| ATA-7 to present
|-
| 3C
| WRITE VERIFY
| PIO
| 8-bit
| ATA-1 to ATA-3
|-
| 3D
| WRITE DMA FUA EXT
| DMA
| 16-bit
| ATA-7 to present
|-
| 3E
| WRITE DMA QUEUED FUA EXT
| queued DMA
| 16-bit
| ATA-7 to ATA8-ACS
|-
| 3F
| WRITE LOG EXT
| PIO
| 16-bit
| ATA-6 to present
|-
| 40
| READ VERIFY SECTOR(S)
| none
| 8-bit
| IBM PC/AT to present
|-
| 41
| READ VERIFY SECTOR(S) (without retry)
| none
| 8-bit
| IBM PC/AT to ATA-4
|-
| 42
| READ VERIFY SECTOR(S) EXT
| none
| 16-bit
| ATA-6 to present
|-
| 43
|
|
|
|
|-
| 44
| ZERO EXT
| none
| 16-bit
| ACS-4 to present
|-
| 45
| WRITE UNCORRECTABLE EXT
| none
| 16-bit
| ATA8-ACS to present
|-
| 46
|
|
|
|
|-
| 47
| READ LOG DMA EXT
| DMA
| 16-bit
| ATA8-ACS to present
|-
| 48
|
|
|
|
|-
| 49
|
|
|
|
|-
| 4A
| ZAC Management In
| DMA
| 16-bit
| ACS-4 to present
|-
| 4B
|
|
|
|
|-
| 4C
|
|
|
|
|-
| 4D
|
|
|
|
|-
| 4E
|
|
|
|
|-
| 4F
|
|
|
|
|-
| 50
| FORMAT TRACK
| PIO
| 8-bit
| IBM PC/AT to ATA-3
|-
| 51
| CONFIGURE STREAM
| none
| 16-bit
| ATA-7 to present
|-
| 52
|
|
|
|
|-
| 53
|
|
|
|
|-
| 54
|
|
|
|
|-
| 55
|
|
|
|
|-
| 56
|
|
|
|
|-
| 57
| WRITE LOG DMA EXT
| DMA
| 16-bit
| ATA8-ACS to present
|-
| 58
|
|
|
|
|-
| 59
|
|
|
|
|-
| 5A
|
|
|
|
|-
| 5B
| TRUSTED NON-DATA
| none
| 8-bit
| ACS-2 to present
|-
| 5C
| TRUSTED RECEIVE
| PIO
| 8-bit
| ATA8-ACS to present
|-
| 5D
| TRUSTED RECEIVE DMA
| DMA
| 8-bit
| ATA8-ACS to present
|-
| 5E
| TRUSTED SEND
| PIO
| 8-bit
| ATA8-ACS to present
|-
| 5F
| TRUSTED SEND DMA
| DMA
| 8-bit
| ATA8-ACS to present
|-
| 60
| READ FPDMA QUEUED
| queued DMA
| 16-bit
| ATA8-ACS to present
|-
| 61
| WRITE FPDMA QUEUED
| queued DMA
| 16-bit
| ATA8-ACS to present
|-
| 62
|
|
|
|
|-
| 63
| NCQ NON-DATA
| none
| 16-bit
| ACS-3 to present
|-
| 64
| SEND FPDMA QUEUED
| queued DMA
| 16-bit
| ACS-3 to present
|-
| 65
| RECEIVE FPDMA QUEUED
| queued DMA
| 16-bit
| ACS-3 to present
|-
| 66
|
|
|
|
|-
| 67
|
|
|
|
|-
| 68
|
|
|
|
|-
| 69
|
|
|
|
|-
| 6A
|
|
|
|
|-
| 6B
|
|
|
|
|-
| 6C
|
|
|
|
|-
| 6D
|
|
|
|
|-
| 6E
|
|
|
|
|-
| 6F
|
|
|
|
|-
| 70
| SEEK
| none
| 8-bit
| IBM PC/AT to ATA-6
|-
| 71
| SEEK
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 72
| SEEK
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 73
| SEEK
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 74
| SEEK
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 75
| SEEK
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 76
| SEEK
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 77
| SET DATE & TIME EXT
| none
| 16-bit
| ACS-3 to present
|-
| 77
| SEEK
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 78
| ACCESSIBLE MAX ADDRESS CONFIGURATION
| none
| 16-bit
| ACS-3 to present
|-
| 78
| SEEK
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 79
| SEEK
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 7A
| SEEK
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 7B
| SEEK
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 7C
| REMOVE ELEMENT AND TRUNCATE
| none
| 16-bit
| ACS-4 to present
|-
| 7C
| SEEK
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 7D
| RESTORE ELEMENTS AND REBUILD
| none
| 16-bit
| ACS-5 to present
|-
| 7D
| SEEK
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 7E
| REMOVE ELEMENT AND MODIFY ZONES 
| none
| 16-bit
| ACS-5 to present
|-
| 7E
| SEEK
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 7F
| SEEK
| none
| 8-bit
| IBM PC/AT to ATA-2
|-
| 80
| Vendor Specific
|
|
|
|-
| 81
| Vendor Specific
|
|
|
|-
| 82
| Vendor Specific
|
|
|
|-
| 83
| Vendor Specific
|
|
|
|-
| 84
| Vendor Specific
|
|
|
|-
| 85
| Vendor Specific
|
|
|
|-
| 86
| Vendor Specific
|
|
|
|-
| 87
| CFA TRANSLATE SECTOR
| PIO
| 8-bit
| ATA-4 to present
|-
| 88
| Vendor Specific
|
|
|
|-
| 89
| Vendor Specific
|
|
|
|-
| 8A
| Vendor Specific
|
|
|
|-
| 8B
| Vendor Specific
|
|
|
|-
| 8C
| Vendor Specific
|
|
|
|-
| 8D
| Vendor Specific
|
|
|
|-
| 8E
| Vendor Specific
|
|
|
|-
| 8F
| Vendor Specific
|
|
|
|-
| 90
| EXECUTE DEVICE DIAGNOSTIC
| none
| 8-bit
| IBM PC/AT to present
|-
| 91
| INITIALIZE DEVICE PARAMETERS
| none
| 8-bit
| IBM PC/AT to ATA-5
|-
| 92
| DOWNLOAD MICROCODE
| PIO
| 8-bit
| ATA-2 to present
|-
| 93
| DOWNLOAD MICROCODE DMA
| DMA
| 8-bit
| ACS-2 to present
|-
| 94
| STANDBY IMMEDIATE
| none
| 8-bit
| ATA-1 to ATA-3
|-
| 95
| IDLE IMMEDIATE
| none
| 8-bit
| ATA-1 to ATA-3
|-
| 96
| MUTATE EXT
| none
| 16-bit
| ACS-5 to present
|-
| 96
| STANDBY
| none
| 8-bit
| ATA-1 to ATA-3
|-
| 97
| IDLE
| none
| 8-bit
| ATA-1 to ATA-3
|-
| 98
| CHECK POWER MODE
| none
| 8-bit
| ATA-1 to ATA-3
|-
| 99
| SLEEP
| none
| 8-bit
| ATA-1 to ATA-3
|-
| 9A
| Vendor Specific
|
|
|
|-
| 9B
|
|
|
|
|-
| 9C
|
|
|
|
|-
| 9D
|
|
|
|
|-
| 9E
|
|
|
|
|-
| 9F
| ZAC Management Out
| DMA
| 16-bit
| ACS-4 to present
|-
| A0
| PACKET
| packet
| 8-bit
| ATA-3 to ACS-3
|-
| A1
| IDENTIFY PACKET DEVICE
| PIO
| 8-bit
| ATA-3 to ACS-3
|-
| A2
| SERVICE
| varies
| 8-bit
| ATA-3 to ATA8-ACS
|-
| A3
|
|
|
|
|-
| A4
|
|
|
|
|-
| A5
|
|
|
|
|-
| A6
|
|
|
|
|-
| A7
|
|
|
|
|-
| A8
|
|
|
|
|-
| A9
|
|
|
|
|-
| AA
|
|
|
|
|-
| AB
|
|
|
|
|-
| AC
|
|
|
|
|-
| AD
|
|
|
|
|-
| AE
|
|
|
|
|-
| AF
|
|
|
|
|-
| B0
| SMART
| PIO
| 8-bit
| ATA-3 to present
|-
| B1
| Device Configuration Overlay
| PIO
| 8-bit
| ATA-6 to ACS-2
|-
| B2
| SET SECTOR CONFIGURATON EXT
| none
| 16-bit
| ACS-4 to present
|-
| B3
|
|
|
|
|-
| B4
| Sanitize Device
| none
| 16-bit
| ACS-2 to present
|-
| B5
|
|
|
|
|-
| B6
| NV Cache
| DMA
| 16-bit
| ATA8-ACS to ACS-2
|-
| B7
| Reserved for CFA
|
|
|
|-
| B8
| Reserved for CFA
|
|
|
|-
| B9
| Reserved for CFA
|
|
|
|-
| BA
| Reserved for CFA
|
|
|
|-
| BB
| Reserved for CFA
|
|
|
|-
| BC
|
|
|
|
|-
| BD
|
|
|
|
|-
| BE
|
|
|
|
|-
| BF
|
|
|
|
|-
| C0
| CFA ERASE SECTORS
| none
| 8-bit
| ATA-4 to ACS-2
|-
| C1
| Vendor Specific
|
|
|
|-
| C2
| Vendor Specific
|
|
|
|-
| C3
| Vendor Specific
|
|
|
|-
| C4
| READ MULTIPLE
| PIO
| 8-bit
| ATA-1 to ACS-3
|-
| C5
| WRITE MULTIPLE
| PIO
| 8-bit
| ATA-1 to ACS-3
|-
| C6
| SET MULTIPLE MODE
| none
| 8-bit
| ATA-1 to ACS-3
|-
| C7
| READ DMA QUEUED
| queued DMA
| 8-bit
| ATA-4 to ATA8-ACS
|-
| C8
| READ DMA
| DMA
| 8-bit
| ATA-1 to present
|-
| C9
| READ DMA (without retry)
| DMA
| 8-bit
| ATA-1 to ATA-4
|-
| CA
| WRITE DMA
| DMA
| 8-bit
| ATA-1 to present
|-
| CB
| WRITE DMA (without retry)
| DMA
| 8-bit
| ATA-1 to ATA-4
|-
| CC
| WRITE DMA QUEUED
| queued DMA
| 8-bit
| ATA-4 to ATA8-ACS
|-
| CD
| CFA WRITE MULTIPLE WITHOUT ERASE
| PIO
| 8-bit
| ATA-4 to present
|-
| CE
| WRITE MULTIPLE FUA EXT
| PIO
| 16-bit
| ATA-7 to ACS-3
|-
| CF
|
|
|
|
|-
| D0
|
|
|
|
|-
| D1
| CHECK MEDIA CARD TYPE
| none
| 8-bit
| ATA-6 to ACS-2
|-
| D2
|
|
|
|
|-
| D3
|
|
|
|
|-
| D4
|
|
|
|
|-
| D5
|
|
|
|
|-
| D6
|
|
|
|
|-
| D7
|
|
|
|
|-
| D8
|
|
|
|
|-
| D9
|
|
|
|
|-
| DA
| GET MEDIA STATUS
| none
| 8-bit
| ATA-3 to ATA-7
|-
| DB
| ACKNOWLEDGE MEDIA CHANGE
| none
| 8-bit
| ATA-1 to ATA-2
|-
| DC
| BOOT - POST-BOOT
| none
| 8-bit
| ATA-1 to ATA-2
|-
| DD
| BOOT - PRE-BOOT
| none
| 8-bit
| ATA-1 to ATA-2
|-
| DE
| MEDIA LOCK
| none
| 8-bit
| ATA-1 to ATA-7
|-
| DF
| MEDIA UNLOCK 
| none
| 8-bit
| ATA-1 to ATA-7
|-
| E0
| STANDBY IMMEDIATE
| none
| 8-bit
| ATA-1 to present
|-
| E1
| IDLE IMMEDIATE
| none
| 8-bit
| ATA-1 to present
|-
| E2
| STANDBY
| none
| 8-bit
| ATA-1 to present
|-
| E3
| IDLE
| none
| 8-bit
| ATA-1 to present
|-
| E4
| READ BUFFER
| PIO
| 8-bit
| ATA-1 to present
|-
| E5
| CHECK POWER MODE
| none
| 8-bit
| ATA-1 to present
|-
| E6
| SLEEP
| none
| 8-bit
| ATA-1 to present
|-
| E7
| FLUSH CACHE
| none
| 8-bit
| ATA-4 to present
|-
| E8
| WRITE BUFFER
| PIO
| 8-bit
| ATA-1 to present
|-
| E9
| READ BUFFER DMA
| DMA
| 8-bit
| ACS-2 to present
|-
| E9
| WRITE SAME
| 
| 8-bit
| ATA-1 to ATA-2
|-
| EA
| FLUSH CACHE EXT
| none
| 8-bit
| ATA-6 to present
|-
| EB
| WRITE BUFFER DMA
| DMA
| 8-bit
| ACS-2 to present
|-
| EC
| IDENTIFY DEVICE
| PIO
| 8-bit
| ATA-1 to present
|-
| ED
| MEDIA EJECT
| none
| 8-bit
| ATA-1 to ATA-7
|-
| EE
| IDENTIFY DEVICE DMA
| DMA
| 8-bit
| ATA-3
|-
| EF
| SET FEATURES
| none
| 8-bit
| ATA-1 to present
|-
| F0
| Vendor Specific
|
|
|
|-
| F1
| SECURITY SET PASSWORD
| PIO
| 8-bit
| ATA-3 to present
|-
| F2
| SECURITY UNLOCK
| PIO
| 8-bit
| ATA-3 to present
|-
| F3
| SECURITY ERASE PREPARE
| none
| 8-bit
| ATA-3 to present
|-
| F4
| SECURITY ERASE UNIT
| PIO
| 8-bit
| ATA-3 to present
|-
| F5
| SECURITY FREEZE LOCK
| none
| 8-bit
| ATA-3 to present
|-
| F6
| SECURITY DISABLE PASSWORD
| PIO
| 8-bit
| ATA-3 to present
|-
| F7
| Vendor Specific
|
|
|
|-
| F8
| READ NATIVE MAX ADDRESS
| none
| 8-bit
| ATA-4 to ACS-2
|-
| F9
| SET MAX ADDRESS
| none
| 8-bit
| ATA-4 to ACS-2
|-
| FA
| Vendor Specific
|
|
|
|-
| FB
| Vendor Specific
|
|
|
|-
| FC
| Vendor Specific
|
|
|
|-
| FD
| Vendor Specific
|
|
|
|-
| FE
| Vendor Specific
|
|
|
|-
| FF
| Vendor Specific
|
|
|
|}



[[Category:ATA]]
````

<a id="w004276"></a>

## W004276 — QEMU fw cfg

Bron-ID: W004276
Namespace: Main (0)
Revisie: 30295 | Datum: 2026-03-16T20:22:31Z
Bron: https://wiki.osdev.org/QEMU_fw_cfg
Revisie-URL: https://wiki.osdev.org/index.php?title=QEMU_fw_cfg&oldid=30295

````text
[[QEMU]] provides a facility for passing strings and files into the VM. This facility is useful for passing kernel parameters, files, or other resources into a guest.


== QEMU command line ==

To pass strings or files into QEMU to appear in the fw_cfg device, use one or more command line parameters like those shown below:

  -fw_cfg name=opt/com.name.domain.your.example,string=1
  -fw_cfg name=opt/com.name.domain.your.example,file=example_filename.png
  -fw_cfg opt/com.name.domain.your.example,file=example_filename.png

Names that begin with "opt/" are reserved for users. It is strongly recommended to use a RFQDN (Reversed Fully Qualified Domain Name) as a prefix for the resource name. In the example above, it is assumed you are the owner of the domain "your.domain.name.com", and your resource is named "example". Following this advice guarantees that there will never be a conflict, since only one entity can own a given domain name. The name may be up to 55 characters long, including the "opt/" prefix. The "name=" part of the argument may be omitted, as shown in the third example above.

== Detecting QEMU ==

It is a very good idea for your OS to first make sure it is running under QEMU, before attempting to access the fw_cfg device. Accessing I/O ports for devices that don't exist on a real machine may have undefined behavior. To detect the presence of QEMU, you should read CPUID leaf 0x40000000 and check for the hypervisor signature in the ebx, ecx, edx registers. Under QEMU, those registers will contain the signature "TCGTCGTCGTCG" or "KVMKVMKVM\0\0\0". A link to a CPUID hypervisor leaf reference is at the end of the article.

Since the hypervisor leaf can be disabled in QEMU, an alternate way to detect QEMU is to look for ACPI PnP ID "QEMU0002" in the ACPI tables. This is required to detect the fw_cfg MMIO address on non-x86 architectures, and is a reliable way to auto-detect the I/O port base addresses, in case they are changed in the future.

== Accessing the device from your OS ==

QEMU provides three I/O ports on x86, or three MMIO addresses on other architectures (''details for other architectures are omitted in this article for now, see the QEMU documentation file below if you need to use fw_cfg on another architecture'').

<syntaxhighlight lang="c">
  #define FW_CFG_PORT_SEL     0x510
  #define FW_CFG_PORT_DATA    0x511
  #define FW_CFG_PORT_DMA     0x514
</syntaxhighlight>

''FW_CFG_PORT_SEL'' is a 16-bit port at I/O address 0x510. You write a "selector" to this port to control which data will be read when reading ''FW_CFG_PORT_DATA''. More about selectors below.

''FW_CFG_PORT_DATA'' is an 8-bit port at I/O address 0x511. You read one byte at a time of the file from this port, after selecting a file by writing a value to ''FW_CFG_PORT_SEL''.

''FW_CFG_PORT_DMA'' is a 32-bit port at I/O address 0x514. DMA is covered later in this article.

=== Selectors ===

<syntaxhighlight lang="c">
  #define FW_CFG_SIGNATURE    0x0000
  #define FW_CFG_ID           0x0001
  #define FW_CFG_FILE_DIR     0x0019
</syntaxhighlight>

Listed above are a few fixed-purpose selectors for probing for the existence of the device, and for getting the number of files present in the device. Besides those selectors, each file will be assigned a selector. You determine the selector to use for a file you wish to read by reading the directory file ''FW_CFG_FILE_DIR''.

You should ensure that the fw_cfg device is present by outputting ''FW_CFG_SIGNATURE'' to I/O port ''FW_CFG_PORT_SEL'', then read four bytes from ''FW_CFG_PORT_DATA''. If you read "QEMU" from the data port, the fw_cfg device is present.

=== Enumerating Files ===

The first four bytes of the ''FW_CFG_FILE_DIR'' selector will be a 32-bit big-endian number which is the number of files present in the directory.

Following the count, there is a sequence of entries with the following structure:

<syntaxhighlight lang="c">
  struct FWCfgFile {		/* an individual file entry, 64 bytes total */
      uint32_t size;		/* size of referenced fw_cfg item, big-endian */
      uint16_t select;		/* selector key of fw_cfg item, big-endian */
      uint16_t reserved;
      char name[56];		/* fw_cfg item name, NUL-terminated ascii */
  };
</syntaxhighlight>

These can be read one at a time by reading ''sizeof(FWCfgFile)'' bytes into a single variable of type ''FWCfgFile'' and examined one by one until you find a structure with the desired name. There is no requirement to read one entry at a time, you may read multiple entries, or even all entries, in a single operation.

=== Reading files ===

Once you have found the ''FWCfgFile'' structure which has the desired name, read the ''select'' field from the structure. It will contain the big-endian selector for the file. After byte-swapping the selector, output it to ''FW_CFG_PORT_SEL'', then read ''size'' bytes from ''FW_CFG_PORT_DATA'' and write them to the desired location in memory.

An example implementation is provided in the references below.

== Advanced Usage ==

The information provided earlier in this article is sufficient to use fw_cfg. The following sections describe DMA mode, which provides increased performance, the ability to seek to an offset within a file before doing a data access. Early versions of QEMU (before 2.4) allowed you to write changes back into the firmware.

=== DMA ===

Without DMA, you would be writing an I/O port to select a file then repeating I/O to read each byte. DMA replaces that with a more efficient operation, you put a 16 byte descriptor in memory for an operation, then write the physical address of that descriptor to the DMA register. The command descriptors must have the following memory layout:

<syntaxhighlight lang="c">

// fw_cfg DMA commands
typedef enum fw_cfg_ctl_t {
    fw_ctl_error = 1,
    fw_ctl_read = 2,
    fw_ctl_skip = 4,
    fw_ctl_select = 8,
    fw_ctl_write = 16
} fw_cfg_ctl_t;

typedef struct FWCfgDmaAccess {
    uint32_t control;
    uint32_t length;
    uint64_t address;
} FWCfgDmaAccess;
</syntaxhighlight>

==== Commands ====

Commands are specified in the ''control'' field. Some commands use the upper 16 bits of the ''control'' field to hold a file selector. Some or all of the other fields may be unused depending upon which command is being issued, and should be initialized to zeros unless they hold usable information.

NOTE: All three fields must be in big-endian order. You can ensure that the values written are in big-endian "network" order using ''htonl'' and ''htobe64'' byte swapping functions. Those functions will pass the value through unmodified on big-endian machines, and will byte swap the input on little-endian machines.

===== Select =====

The select command selects the file specified in the upper 16 bits of the ''control'' field. Selecting a file also resets its seek position to the beginning of the file.

===== Skip =====

The skip command advances the seek position through the file by the amount specified in the ''length'' field.

===== Read =====

The read command copies ''length'' bytes from the current seek position of the currently selected file into memory at the physical memory address ''address'', and advances the seek position of the file appropriately.

===== Write =====

The write command writes ''length'' bytes to the current seek position of the currently selected file from physical memory address ''address'', and advances the seek position of the file appropriately.

NOTE: Writing is not supported in QEMU starting at version 2.4. Earlier versions allowed writes.

==== Command Errors ====

If any I/O operation goes outside the bounds of the selected file, then the entire operation is cancelled immediately before changing any memory or firmware values, and the ''error'' bit (bit 0) of the command descriptor is updated to be nonzero.

==== Command Completion ====

Currently QEMU will immediately complete every command (whether it succeeds or fails) during the time that your driver is writing to the DMA register. QEMU documentation warns that fw_cfg may become asynchronous some day (meaning, it may not continue to complete commands instantly in the future). Your driver should enter a loop that reads the command field until it becomes zero after issuing a command, just in case. Typically it will check the command field and it will already be zero, and it won't run one iteration of the loop. You must also break out of the loop if bit 0 of the ''control'' field becomes set, which occurs in an error condition.

==== DMA I/O Ports ====

The value written to the DMA port must be in big-endian byte order. The upper 32 bits of the command descriptor physical address must be written to the DMA port if the value is nonzero. Then, the lower 32 bits of the address must be written to (DMA port + 4). For example:

<syntaxhighlight lang="C">
        FWCfgDmaAccess cmd;

        cmd.control = htonl(fw_ctl_read);
        cmd.address = htobe64(buffer_ptr);
        cmd.length = htonl(size);

        // Assumes identity mapping - replace this with
        // virt-to-phys translation if necessary
        uintptr_t cmd_physaddr = uintptr_t(&cmd);

        // Split it into halves and avoid warning for upper bits in 32 bit build
        uint32_t cmd_physaddr_lo = uint32_t(cmd_physaddr & 0xFFFFFFFFU);
        uint32_t cmd_physaddr_hi = sizeof(uintptr_t) > sizeof(uint32_t)
                ? uint32_t(cmd_physaddr >> 32)
                : 0;

        // Skip writing high half if unnecessary.
        if (cmd_physaddr_hi)
            outl(FW_CFG_PORT_DMA, htonl(cmd_physaddr_hi));
        outl(FW_CFG_PORT_DMA + 4, htonl(cmd_physaddr_lo));

        // Hardware loads zero into the entire command address register
        // after executing a command. In particular, subsequent commands
        // can skip writing the upper half of the address if their value
        // is all zeros
</syntaxhighlight>

== References ==
* [https://github.com/qemu/qemu/blob/master/docs/specs/fw_cfg.rst QEMU fw_cfg documentation on GitHub]
* [https://github.com/doug65536/dgos/blob/master/boot/x86/qemu_x86.cc Example implementation]
* [http://www.sandpile.org/x86/cpuid.htm#level_4000_0000h CPUID hypervisor leaf]

[[Category:QEMU]]
````

<a id="w004277"></a>

## W004277 — User:Iaz3/Personal Stuff

Bron-ID: W004277
Namespace: User (2)
Revisie: 30390 | Datum: 2026-03-23T14:00:30Z
Bron: https://wiki.osdev.org/User:Iaz3/Personal_Stuff
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AIaz3%2FPersonal_Stuff&oldid=30390

````text
<!--Dont use these yet, they do mean things like add us to a category when we're not ready.
{{Rating|1}}
-->
{{Template:Kernel designs}}

In this tutorial we will write a simple kernel for [[IA32_Architecture_Family|32-bit x86]] and boot it. This is the first step in creating your own operating system. 

This tutorial attempts to serves as an example of how to create a minimal extensible system, following best practice. This article is heavily based on [[Bare Bones]], but unlike it, this article attempts to serve as an example of how to structure your code and project, rather than throwing it all away in the next step with [[Meaty Skeleton]].

This tutorial uses existing technology to get you started and straight into [[kernel]] development, rather than developing your own [[Languages|programming language]], your own [[compiler]], and your own [[bootloader]]. In this tutorial, we use:
* The [[LD|GNU Linker]] from [[Binutils]] to link our [[Object File|object files]] into the final kernel.
* The [[NASM|NASM Assembler]] to [[Assembly|assemble instructions]] into object files containing machine code.
* The [[GCC|GNU Compiler Collection]] to compile our high level code into assembly.
* The [[C]] programming language (or optionally [[C++]]) to write the high level parts of our [[kernel]].
* The [[GRUB]] bootloader to [[Bootloader|load]] our kernel using the [[Multiboot]] boot protocol that loads us into 32-bit protected mode with [[paging]] disabled.
* The [[ELF]] as the [[Executable Formats|executable format]] that gives us control of where and how the kernel is loaded.

This article assumes you are using a Unix-like operating system such as Linux which supports operating systems development well. Windows users should be able to complete it from a [[MinGW]], [[Cygwin]], or if Windows 10, the Windows Subsystem For Linux environment.(Note: if using WSL, you will want Ubuntu 18.04 from the store, rather than Ubuntu, which defaults to 16.04 and has GCC 5 instead of GCC 7/8)

Succeeding at operating systems development requires becoming an expert, having patience, and reading all the instructions very carefully. You need to read everything in this article before proceeding. If you run into problems, you need to reread the article even more carefully and then do it thrice more for good measure. If you still have issues, our community is experienced and will gladly help at the [http://forum.osdev.org forums] or on [[Chat|IRC]].

<big>'''WAIT! Have you read [[Getting Started]], [[Beginner Mistakes]], and some of the related [[:Category:OS theory|OS theory]]?'''</big>

== Building a Cross-Compiler ==
{{Main|GCC Cross-Compiler}}

The first thing you should do is set up a [[GCC Cross-Compiler]] for '''i686-elf'''. You have not yet modified your compiler to know about the existence of your operating system, so we use a generic target called i686-elf, which provides you with a toolchain targeting the System V ABI. This setting is well tested and understood by the osdev community and will allow you to easily set up a bootable kernel using GRUB and Multiboot. (Note that if you are already using an ELF platform, such as Linux, you may already have a GCC that produces ELF programs. This is not suitable for osdev work, as this compiler will produce programs for Linux, and your operating system is '''not''' Linux, no matter how similar it is. You will certainly run into trouble if you don't use a cross-compiler.)

You will ''not'' be able to correctly compile your operating system without a cross-compiler.

You will ''not'' be able to correctly complete this tutorial with a x86_64-elf cross-compiler, as GRUB is only able to load 32-bit multiboot kernels. If this is your first operating system project, you should do a 32-bit kernel first. If you use a x86_64 compiler instead and somehow bypass the later sanity check, you will end up with a kernel that GRUB doesn't know how to boot.

== Overview ==

By now, you should have set up your [[GCC Cross-Compiler|cross-compiler]] for i686-elf (as described above). This tutorial provides a minimal solution for creating an operating system for x86, and attempts to serve as a starting point to expand your kernel.

To start with, we just need three main files: 

* boot.asm - kernel entry point that sets up the processor environment
* kernel.c - your actual kernel routines
* linker.ld - tells [[LD]] how to link the above files together

=== Project Structure ===

The way you layout your project can be very important. Consider that you may eventually wish to port your operating system to systems other than x86, or just to organize your code.

<div class="toccolours mw-collapsible" style="width:400px; overflow:auto;">
Directory Structure:
<div class="mw-collapsible-content">
<!--TODO: ugly formatting pls fix-->
* Project
** src
*** kernel
**** arch
***** i686
****** boot.asm
****** linker.ld
*** kernel.c
</div>
</div>

=== Build System ===

A build system is outside the scope of this tutorial, so it is assumed you have one set up and are able to make it work.

== Booting the Operating System ==

To start the operating system, an existing piece of software will be needed to load it. This is called the bootloader and in this tutorial we will be using [[GRUB]]. Writing your own bootloader is an advanced subject, but it is commonly done. We'll later configure the bootloader, but the operating system needs to handle when the bootloader passes control to it. The kernel is passed a very minimal environment, in which the stack is not set up yet, virtual memory is not yet enabled, hardware is not initialized, and so on.

The first task we will deal with is how the bootloader starts the kernel. We are lucky because there exists a Multiboot Standard, which describes an easy interface between the bootloader and the operating system kernel. It works by putting a few magic values in some global variables (known as a multiboot header), which is searched for by the bootloader. When it sees these values, it recognizes the kernel as multiboot compatible and it knows how to load us, and it can even forward us important information such as memory maps, but we won't need that yet.

Since there is no stack yet and we need to make sure the global variables are set correctly, we will do this in assembly.

=== Bootstrap Assembly ===

We will now create a file called boot.asm and discuss its contents. In this example, we are using the NASM assembler.

The very most important piece to create is the multiboot header, as it must be very early in the kernel binary, or the bootloader will fail to recognize us. Our linker script linker.ld will ensure it is near the beginning.

<div class="toccolours mw-collapsible mw-collapsed">
Kernel/arch/i686/boot.asm
<div class="mw-collapsible-content">
<syntaxhighlight lang="asm">
; Declare constants for the multiboot header.
MBALIGN  equ  1 << 0            ; align loaded modules on page boundaries
MEMINFO  equ  1 << 1            ; provide memory map
FLAGS    equ  MBALIGN | MEMINFO ; this is the Multiboot 'flag' field
MAGIC    equ  0x1BADB002        ; 'magic number' lets bootloader find the header
CHECKSUM equ -(MAGIC + FLAGS)   ; checksum of above, to prove we are multiboot
 
; Declare a multiboot header that marks the program as a kernel. These are magic
; values that are documented in the multiboot standard. The bootloader will
; search for this signature in the first 8 KiB of the kernel file, aligned at a
; 32-bit boundary. The signature is in its own section so the header can be
; forced to be within the first 8 KiB of the kernel file.
section .multiboot
align 4
	dd MAGIC
	dd FLAGS
	dd CHECKSUM
 
; The multiboot standard does not define the value of the stack pointer register
; (esp) and it is up to the kernel to provide a stack. This allocates room for a
; small stack by creating a symbol at the bottom of it, then allocating 16384
; bytes for it, and finally creating a symbol at the top. The stack grows
; downwards on x86. The stack on x86 must be 16-byte aligned according to the
; System V ABI standard and de-facto extensions. The compiler will assume the
; stack is properly aligned and failure to align the stack will result in
; undefined behavior.
section .bss
align 16
stack_bottom:
resb 16384 ; 16 KiB
stack_top:
 
; The linker script specifies _start as the entry point to the kernel and the
; bootloader will jump to this position once the kernel has been loaded. It
; doesn't make sense to return from this function as the bootloader is gone.
; Declare _start as a function symbol with the given symbol size.
section .text
global _start:function (_start.end - _start)
_start:
	; The bootloader has loaded us into 32-bit protected mode on a x86
	; machine. Interrupts are disabled. Paging is disabled. The processor
	; state is as defined in the multiboot standard. The kernel has full
	; control of the CPU. The kernel can only make use of hardware features
	; and any code it provides as part of itself. There's no printf
	; function, unless the kernel provides its own <stdio.h> header and a
	; printf implementation. There are no security restrictions, no
	; safeguards, no debugging mechanisms, only what the kernel provides
	; itself. It has absolute and complete power over the
	; machine.
 
	; To set up a stack, we set the esp register to point to the top of our
	; stack (as it grows downwards on x86 systems). This is necessarily done
	; in assembly as languages such as C cannot function without a stack.
	mov esp, stack_top
 
	; This is a good place to initialize crucial processor state before the
	; high-level kernel is entered. It's best to minimize the early
	; environment where crucial features are offline. Note that the
	; processor is not fully initialized yet: Features such as floating
	; point instructions and instruction set extensions are not initialized
	; yet. The GDT should be loaded here. Paging should be enabled here.
	; C++ features such as global constructors and exceptions will require
	; runtime support to work as well.
 
	; Enter the high-level kernel. The ABI requires the stack is 16-byte
	; aligned at the time of the call instruction (which afterwards pushes
	; the return pointer of size 4 bytes). The stack was originally 16-byte
	; aligned above and we've since pushed a multiple of 16 bytes to the
	; stack since (pushed 0 bytes so far) and the alignment is thus
	; preserved and the call is well defined.
        ; note, that if you are building on Windows, C functions may have "_" prefix in assembly: _kmain
	extern kmain
	call kmain
 
	; If the system has nothing more to do, put the computer into an
	; infinite loop. To do that:
	; 1) Disable interrupts with cli (clear interrupt enable in eflags).
	;    They are already disabled by the bootloader, so this is not needed.
	;    Mind that you might later enable interrupts and return from
	;    kmain (which is sort of nonsensical to do).
	; 2) Wait for the next interrupt to arrive with hlt (halt instruction).
	;    Since they are disabled, this will lock up the computer.
	; 3) Jump to the hlt instruction if it ever wakes up due to a
	;    non-maskable interrupt occurring or due to system management mode.
	cli
.hang:	hlt
	jmp .hang
.end:
</syntaxhighlight>
</div>
</div>

You can then assemble boot.asm using:
<syntaxhighlight lang="bash">nasm -felf32 boot.asm -o boot.o</syntaxhighlight>

== Implementing the Kernel ==

So far we have written the bootstrap assembly stub that sets up the processor such that high level languages such as C can be used. It is also possible to use other languages such as C++.

=== Freestanding and Hosted Environments ===

If you have done C or C++ programming in user-space, you have used a so-called Hosted Environment. Hosted means that there is a C standard library and other useful runtime features. Alternatively, there is the Freestanding version, which is what we are using here. Freestanding means that there is no C standard library, only what we provide ourselves. However, some header files are actually not part of the C standard library, but rather the compiler. These remain available even in freestanding C source code. In this case we use <stdbool.h> to get the bool datatype, <stddef.h> to get size_t and NULL, and <stdint.h> to get the intx_t and uintx_t datatypes which are invaluable for operating systems development, where you need to make sure that the variable is of an exact size (if we used a short instead of uint16_t and the size of short changed, our VGA driver here would break!). Additionally you can access the <float.h>, <iso646.h>, <limits.h>, and <stdarg.h> headers, as they are also freestanding. GCC actually ships a few more headers, but these are special purpose.

=== Writing a kernel in C ===

The following shows how to create a simple kernel in C. This kernel uses the VGA text mode buffer (located at 0xB8000) as the output device. It sets up a simple driver that remembers the location of the next character in this buffer and provides a primitive for adding a new character. Notably, there is no support for line breaks ('\n') (and writing that character will show some VGA-specific character instead) and no support for scrolling when the screen is filled up. Adding this will be your first task. Please take a few moments to understand the code.

<div class="toccolours mw-collapsible mw-collapsed">
Kernel/kernel.c
<div class="mw-collapsible-content">
<syntaxhighlight lang="c">
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

/* Check if the compiler thinks we are targeting the wrong operating system. */
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

static const size_t VGA_WIDTH = 80;
static const size_t VGA_HEIGHT = 25;

size_t terminal_row;
size_t terminal_column;
uint8_t terminal_color;
uint16_t* terminal_buffer;

void terminal_initialize(void) 
{
	terminal_row = 0;
	terminal_column = 0;
	terminal_color = vga_entry_color(VGA_COLOR_LIGHT_GREY, VGA_COLOR_BLACK);
	terminal_buffer = (uint16_t*) 0xB8000;
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

void kmain(void) 
{
	/* Initialize terminal interface */
	terminal_initialize();

	/* Newline support is left as an exercise. */
	terminal_writestring("Hello, kernel World!\n");
}
</syntaxhighlight>
</div>
</div>

Notice how we wish to use the common C function <tt>strlen</tt>, but this function is part of the C standard library that we don't have available. Instead, we rely on the freestanding header <stddef.h> to provide <tt>size_t</tt> and we simply declare our own implementation of <tt>strlen</tt>. You will have to do this for every function you wish to use (as the freestanding headers only provide macros and data types).

Note that this is a temporary measure to simplify this tutorial, later on you will want to want to [[Creating_a_C_Library|create a libc]] with strlen and any other standard functions you need next to the kernel directory. Your build system should build the libc directory first, and the kernel should depend on it. See [[Meaty_Skeleton#libc_and_libk_Design|Meaty Skeleton]] for details on how to do this correctly.

Compile using:
<syntaxhighlight lang="bash">i686-elf-gcc -c kernel.c -o kernel.o -std=gnu99 -ffreestanding -O2 -Wall -Wextra</syntaxhighlight>

Note that the above code uses a few extensions and hence we build as the GNU version of C99.

=== Writing a kernel in C++ ===

Writing a kernel in [[C++]] is easy. Note that not all features from the language is available. For instance, exception support requires special runtime support and so does memory allocation. To write a kernel in C++, simply adopt the code above: Add an extern "C" declaration to the main method. Notice how the kmain function has to be declared with C linkage, as otherwise the compiler would include type information in the assembly name (name mangling). This complicates calling the function from our above assembly stub and we therefore use C linkage, where the symbol name is the same as the name of the function (with no additional type information). Save the code as kernel.c++ (or what your favorite C++ filename extension is). 

You can compile the file kernel.c++ using:

<syntaxhighlight lang="bash">i686-elf-g++ -c kernel.c++ -o kernel.o -ffreestanding -O2 -Wall -Wextra -fno-exceptions -fno-rtti</syntaxhighlight>

Note that you must have also built a cross C++ compiler for this work.

== Linking the Kernel ==

We can now assemble boot.s and compile kernel.c. This produces two object files that each contain part of the kernel. To create the full and final kernel we will have to link these object files into the final kernel program, usable by the bootloader. When developing user-space programs, your toolchain ships with default scripts for linking such programs. However, these are unsuitable for kernel development and we need to provide our own customized linker script. Save the following in linker.ld:

<div class="toccolours mw-collapsible mw-collapsed">
Kernel/arch/i686/linker.ld
<div class="mw-collapsible-content">
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
	}

	/* The compiler may produce other sections, by default it will put them in
	   a segment with the same name. Simply add stuff here as needed. */
}
</pre>
</div>
</div>

With these components you can now actually build the final kernel. We use the compiler as the linker as it allows it greater control over the link process. Note that if your kernel is written in C++, you should use the C++ compiler instead. 

You can then link your kernel using:

<syntaxhighlight lang="bash">i686-elf-gcc -T linker.ld -o myos.bin -ffreestanding -O2 -nostdlib boot.o kernel.o -lgcc</syntaxhighlight>

Note: Some tutorials suggest linking with i686-elf-ld rather than the compiler, however this prevents the compiler from performing various tasks during linking.

The file myos.bin is now your kernel (all other files are no longer needed). Note that we are linking against [[libgcc]], which implements various runtime routines that your cross-compiler depends on. Leaving it out '''will''' give you problems in the future. If you did not build and install [[libgcc]] as part of your cross-compiler, you should go back now and build a cross-compiler with [[libgcc]]. The compiler depends on this library and will use it regardless of whether you provide it or not.

== Verifying Multiboot ==

If you have [[GRUB]] installed, you can check whether a file has a valid [[Multiboot]] version 1 header, which is the case for our kernel. It's important that the Multiboot header is within the first 8 KiB of the actual program file at 4 byte alignment. This can potentially break later if you make a mistake in the boot assembly, the linker script, or anything else that might go wrong. If the header isn't valid, GRUB will give an error that it can't find a Multiboot header when you try to boot it. This code fragment will help you diagnose such cases:

<syntaxhighlight lang="bash">grub-file --is-x86-multiboot myos.bin</syntaxhighlight>

<tt>grub-file</tt> is quiet but will exit 0 (successfully) if it is a valid multiboot kernel and exit 1 (unsuccessfully) otherwise. You can type <tt>echo $?</tt> in your shell immediately afterwards to see the exit status. You can add this grub-file check to your build scripts as a sanity test to catch the problem at compile time. Multiboot version 2 can be checked with the <tt>--is-x86-multiboot2</tt> option instead. If you invoke the <tt>grub-file</tt> command manually in a shell, it is convenient to wrap it in a conditional to easily see the status. This command should work now:

<syntaxhighlight lang="bash">
if grub-file --is-x86-multiboot myos.bin; then
  echo multiboot confirmed
else
  echo the file is not multiboot
fi
</syntaxhighlight>

== Booting the Kernel ==
In a few moments, you will see your kernel in action.

=== Building a bootable cdrom image ===
You can easily create a bootable CD-ROM image containing the GRUB bootloader and your kernel using the program <tt>grub-mkrescue</tt>. You may need to install the GRUB utility programs and the program <tt>xorriso</tt> (version 0.5.6 or higher). First you should create a file called grub.cfg containing the contents:

<pre>
menuentry "myos" {
	multiboot /boot/myos.bin
}
</pre>

Note that the braces must be placed as shown here. You can now create a bootable image of your operating system by typing these commands:
<syntaxhighlight lang="bash">
mkdir -p isodir/boot/grub
cp myos.bin isodir/boot/myos.bin
cp grub.cfg isodir/boot/grub/grub.cfg
grub-mkrescue -o myos.iso isodir
</syntaxhighlight>

Congratulations! You have now created a file called myos.iso that contains your Hello World operating system. If you don't have the program <tt>grub-mkrescue</tt> installed, now is a good time to install GRUB. It should already be installed on Linux systems. Windows users will likely want to use a Cygwin variant if no native grub-mkrescue program is available.

'''Warning:''' GNU GRUB, the bootloader used by <tt>grub-mkrescue</tt>, is licensed under the GNU General Public License. Your iso file contains copyrighted material under that license and redistributing it in violation of the GPL constitutes copyright infringement. The GPL requires you publish the source code corresponding to the bootloader. You need to get the exact source package corresponding to the GRUB package you have installed from your distribution, at the time <tt>grub-mkrescue</tt> is invoked (as distro packages are occasionally updated). You then need to publish that source code along with your ISO to satisfy the GPL. Alternative, you can build GRUB from source code yourself. Clone the latest GRUB git from savannah (do not use their last release from 2012, it's severely out of date). Run autogen.sh, ./configure and make dist. That makes a GRUB tarball. Extract it somewhere, then build GRUB from it, and install it in a isolated prefix. Add that to your PATH and ensure its <tt>grub-mkrescue</tt> program is used to produce your iso. Then publish the GRUB tarball of your own making along with your OS release. You're not required to publish the source code of your OS at all, only the code for the bootloader that's inside the iso.

=== Testing your operating system (QEMU) ===
Virtual Machines are very useful for development operating systems, as they allow you to quickly test your code and have access to the source code during the execution. Otherwise, you would be in for an endless cycle of reboots that would only annoy you. They start very quickly, especially combined with small operating systems such as ours.

In this tutorial, we will be using QEMU. You can also use other virtual machines if you please. Simply adding the ISO to the CD drive of an empty virtual machine will do the trick.

Install QEMU from your repositories, and then use the following command to start your new operating system.

<syntaxhighlight lang="bash">qemu-system-i386 -cdrom myos.iso</syntaxhighlight>

This should start a new virtual machine containing only your ISO as a cdrom. If all goes well, you will be met with a menu provided by the bootloader. Simply select myos and if all goes well, you should see the happy words "Hello, Kernel World!" followed by some mysterious character.

Additionally, QEMU supports booting multiboot kernels directly without bootable medium:

<syntaxhighlight lang="bash">qemu-system-i386 -kernel myos.bin</syntaxhighlight>

=== Testing your operating system (Real Hardware) ===

The program grub-mkrescue is nice because it makes a bootable ISO that works on both real computers and virtual machines. You can then build an ISO and use it everywhere. To boot your kernel on your local computer you can install myos.bin to your /boot directory and configure your bootloader appropriately.

Or alternatively, you can burn it to an USB stick (erasing all data on it!). To do so, simply find out the name of the USB block device, in my case /dev/sdb but this may vary, and using the wrong block device (your harddisk, gasp!) may be disastrous. If you are using Linux and /dev/sdx is your block name, simply:

<syntaxhighlight lang="bash">sudo dd if=myos.iso of=/dev/sdx && sync</syntaxhighlight>

Your operating system will then be installed on your USB stick. If you configure your BIOS to boot from USB first, you can insert the USB stick and your computer should start your operating system.

Alternatively, the .iso is a normal cdrom image. Simply burn it to a CD or DVD if you feel like wasting one of those on a few kilobytes large kernel.

== Moving Forward ==
Now that you can run your new shiny operating system, congratulations! Of course, depending on how much this interests you, it may just be the beginning. Here's a few things to get going.

=== Adding Support for Newlines to Terminal Driver ===
The current terminal driver does not handle newlines. The VGA text mode font stores another character at the location, since newlines are never meant to be actually rendered: they are logical entities. Rather, in terminal_putchar check if c == '\n' and increment terminal_row and reset terminal_column.

=== Adding Support for the Cursor to Terminal Driver ===
The current driver does not move the cursor from wherever it defaulted to. Moving the cursor is a bit more complicated than displaying text, since it requires using IO ports. See [[Text Mode Cursor]] for details on how to do this.

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

Make your operating system self-hosting and then complete Bare Bones under your own operating system while following all the instructions. This is a five star exercise and you may need a couple of years to solve it.

== Frequently Asked Questions ==

;Why the Multiboot header? Wouldn't a pure [[ELF]] file be loadable by GRUB anyway?
:GRUB is capable of loading a variety of formats. However, in this tutorial we are creating a Multiboot compliant kernel that could be loaded by any other compliant bootloader. To achieve this, the multiboot header is mandatory.

;Will GRUB wipe the BSS section before loading the kernel?
:Yes. For ELF kernels, the .bss section is automatically identified and cleared (despite the Multiboot specification being a bit vague about it). For other formats, if you ask it politely to do so, that is if you use the 'address override' information from the Multiboot header (flag #16) and give a non-zero value to the bss_end_addr field. Note that using "address override" with an ELF kernel will disable the default behavior and do what is described by the "address override" header instead.

;What is the state of registers/memory/etc. when GRUB calls my kernel?
:GRUB is an implementation of the Multiboot specification. Anything not specified there is "undefined behavior", which should ring a bell (not only) with C/C++ programmers... Better check the Machine State section of Multiboot documentation, and assume nothing else.

;I still get <tt><nowiki>Error 13: Invalid or unsupported executable format from GRUB</nowiki></tt>...
:Chances are the Multiboot header is missing from the final executable, or it is not at the right location.
:It may also happen if you use an ELF object file instead of an executable (e.g. you have an ELF file with unresolved symbols or unfixable relocations). Try to link your ELF file to a binary executable to get more accurate error messages.

;I get <tt><nowiki>Boot failed: Could not read from CD-ROM (code 0009)</nowiki></tt> when trying to boot the iso image in QEMU
:If your development system is booted from EFI it may be that you don't have the PC-BIOS version of the grub binaries installed anywhere.  If you install them then grub-mkrescue will by default produce a hybrid ISO that will work in QEMU.  On Ubuntu this can be achieved with: <b>apt-get install grub-pc-bin</b>.

== See Also ==

=== Articles ===

* [[Books]]

=== External Links ===

* [https://www.gnu.org/software/grub/manual/multiboot/multiboot.html Multiboot Specification]
* [http://pubs.opengroup.org/onlinepubs/9699919799// The POSIX standard]

[[:Category:Bare bones tutorials]]
[[:Category:C]]
[[:Category:C++]]
````

<a id="w004278"></a>

## W004278 — Intel Dual Band Wireless

Bron-ID: W004278
Namespace: Main (0)
Revisie: 30414 | Datum: 2026-03-25T12:34:42Z
Bron: https://wiki.osdev.org/Intel_Dual_Band_Wireless
Revisie-URL: https://wiki.osdev.org/index.php?title=Intel_Dual_Band_Wireless&oldid=30414

````text
This page provides basic documentation of the freebsd iwm driver, which supports the following WiFi + Bluetooth chipsets:
* Intel Dual Band Wireless 3160
* Intel Dual Band Wireless 3165
* Intel Dual Band Wireless 3168
* Intel Dual Band Wireless 7260
* Intel Dual Band Wireless 7265
* Intel Dual Band Wireless 8260
* Intel Dual Band Wireless 8265

These notes are meant to provide an overview of the driver, so developers may implement their own WiFi drivers for this chipset. The original source code will need to be used in conjunction with these notes in order to have all the details. The original source code can be found at: https://github.com/freebsd/freebsd/tree/master/sys/dev/iwm

== Registers ==

Lots of work seems to be handled by the firmware
 - 0x000 : CSR_HW_IF_CONFIG_REG	:
    - PREPARE = 0x08000000
    - NIC_READY = 0x00400000

 - 0x008 : CSR_INT:
    - Interrupt Status, same bits as CSR_INT_MASK

 - 0x010 : FH_INT_STS:
    - FH interrupt status

 - 0x00c : CSR_INT_MASK:
    - FW_ALIVE =    0x00000001
    - WAKEUP =      0x00000002
    - RX_CMD_RESP = 0x00000008
    - OVERHEAT =    0x00000020
    - HW_KILL =     0x00000040
    - FW_ERR =      0x02000000
    - TXQ_ADV =     0x04000000
    - FH_RX =       0x80000000

 - 0x020 : CSR_RESET:
    - SW_RESET =    0x00000080

 - 0x024 : CSR_GP_CNTRL:
    - HW_RF_KILL_SW = 0x08000000 

 - 0x028 : 32-bit Hardware Revision Register, used to identify specific version:
    - 7000 Series Chips:
        - 31-16: Reserved
        - 15-4: Device type (see IWM_CSR_HW_REV_TYPE in if_iwmreg.h in freebsd)
        - 3-2: Revision step (A,B,C,D)
        - 1-0: Minor Revision number
        
    - 8000 Series Chips:
        - 31-16: Reserved
        - 15-4: Device type (see IWM_CSR_HW_REV_TYPE in if_iwmreg.h in freebsd)
        - 3-0: Revision step (A,B,C,D,...)

 - 0x088 : CSR_MBOX_SET_REG:
        - OS_ALIVE = 0x20

 - 0x0A8 : CSR_MAC_SHADOW_REG_CTRL

 
== Initialization Steps ==


1. Start by making the NIC prepare for use by setting the PREPARE and NIC_READY bits in CSR_HW_IF_CONFIG_REG, wait for NIC_READY to read as set.

2. Signal that the OS is ready by setting the OS_ALIVE bit in CSR_MBOX_SET_REG

3. Start allocating the various DMA buffers
        - A buffer for firmware transfers, 0x20000 byte region, 16-byte aligned

        - A buffer for the tx scheduler, 1024-byte aligned
        - A 'keep warm' page (4096 bytes) for internal use by the device, 4096-byte aligned
        - An rx ring - 256-byte aligned
        - tx rings - 256 byte aligned

4. Startup the hardware (iwm_start_hw)
    - Reset the device by setting the SW_RESET bit of CSR_RESET and waiting 10us
    - Setup and enable the associated HW_KILL interrupt.

5. Start the firmware (iwm_run_init_mvm_ucode)
    - Stop if RF kill
    - Set iwm_wait_phy_db_entry to be called on notification of both IWM_INIT_COMPLETE_NOTIF and IWM_CALIB_RES_NOTIF_PHY_DB
        - Parses and stores some information regarding the PAPD (?) and TXP (?)
    - Load ucode and wait for the fw to report being alive (WIP, iwm_mvm_load_ucode_wait_alive)
        - Read the firmware (iwm_read_firmware)
            - Load firmware into driver memory
            - Parse firmware contents (WIP)
        - Set iwm_alive_fn to be called on notification of MVM_ALIVE
        - Start the firmware (WIP, iwm_start_fw)
            - Check if NIC_READY bit is set, if not, redo step 1
            - Clear interrupt flags, disable interrupts
            - Ensure rfkill handshake is clear
                - Set RFKILL bit in CSR_UCODE_DRV_GP1_CLR
                - Set CMD_BLOCKED bit in CSR_UCODE_DRV_GP1_CLR
            - Init the NIC (iwm_nic_init)
                - Configure NIC (WIP, iwm_mvm_nic_config)
                    - Get the radio information (currently from the firmware, as we haven't read the nvm yet) and configure the NIC's radios appropriately
                - Setup RX queues (WIP, iwm_nic_rx_init)
                    - Stop RX dma (iwm_pcie_rx_stop)
                    - Reset pointers
                    - Set physical address of RX ring
                    - Set physical address of RX status
                    - Enable RX DMA
                    - Set the RB count
                - Setup TX queues (WIP, iwm_nic_tx_init)
                    - Deactivate TX scheduler (write 0 to peripheral register, SCD_TXFACT)
                    - Set physical address of 'keep warm' page
                    - Set physical addresses of TX rings
                    - Set queues to auto-activate (set bit 18 of peripheral register, SCD_GP_CTRL)
                - Enable MAC shadow registers by writing 0x800fffff to CSR_MAC_SHADOW_REG_CTRL

            - Disable all interrupts besides FH_TX
            - Ensure rfkill handshake is still clear
            - Load the firmware (WIP, iwm_pcie_load_given_ucode, iwm_pcie_load_given_ucode_8000)
                - For 7000 Series
                    - Load the cpu sections (iwm_pcie_load_cpu_sections)
                        - See 'Firmware Transfers' section for details.
                    - If dual CPU
                        - Write the CPU2 header address
                        - Load the cpu sections for cpu2 (WIP, iwm_pcie_load_cpu_sections)
                    - Re-enable interrupts
                    - Release the CPUs from reset (Set CSR_RESET to 0)
                - For 8000 Series
                    - 
        - Wait for MVM_ALIVE notification
        - Setup the tx scheduler (WIP, iwm_trans_pcie_fw_alive)
        - Configure firmware paging if necessary, based on the ucode information (WIP, iwm_save_fw_paging, iwm_send_paging_cmd)
    - Initialize nvm/eeprom (WIP, iwm_nvm_init)
        - Reads and parses EEPROM
    - Initialize Bluetooth config (WIP, iwm_send_bt_init_conf)
        - Sends Bluetooth initialization command
    - Get valid TX antennas (WIP, iwm_mvm_get_valid_tx_ant)
        - Part of the information obtained from the eeprom
    - Send valid TX antennas (WIP, iwm_send_tx_ant_cfg)
        - Sends a command with the previously obtained antenna configuration
    - Send physical config command to initialize ucode, and start callibrations (WIP, iwm_send_phy_cfg_cmd)
        - Sends PHY_CONFIGURATION command
    - Wait for init notification
        - Wait for callibration complete notification

6. Initialize the channel map (WIP, iwm_init_channel_map)
    - Parse nvm data to determine the supported channels and wifi bands supported by the device

7. Attach the radiotap and register to the wifi stack (WIP, iwm_radiotap_attach)



== Scan Steps ==


1. If firmware reports UMAC scan support:
    Setup a scan command with the desired scan channels and send to the device

2. Otherwise, if firmware reports LMAC scan support:
    Setup a scan command with the desired scan channels and send to the device


== Transmit Steps ==

Fill a tx command to send to the firmware, encrypting the frame if necessary


== Firmware Transfers ==

Firmware transfers are done via a separate DMA engine.
Sections are sent to it in chunks of 0x20000 bytes.

Target offsets over 0x40000 and under 0x57fff are treated as extended memory.

Target offset of 0xFFFFCCCC specifies the CPU1 and CPU2 separator section.
Target offset of 0xAAAABBBB specifies the paging separator section.

See iwm_pcie_load_firmware_chunk for more information on how to program the DMA engine.


== TX/TFD Scheduler ==

Each TX scheduler entry takes the following format:

 struct tx_scheduler_entry {
    struct {
        uint16_t tx_cmd_byte_count : 12;
        uint16_t station_idx : 4;
    } tfd_queue[256 + 64];
 }


There are as many tx_scheduler_entry structures as queues (MAX_QUEUES = 31).

Each TFD queue has a max size of 256 entries, freebsd driver adds 64 'BC_DUP' entries too, reason unknown right now.


== TX Ring ==

There are 256 TX rings, each ring has the following structure:

 struct tx_ring_entry {
    uint32_t lo_addr;
    uint16_t hi_addr : 4;
    uint16_t tx_buf_len : 12;
 } PACKED


 lo_addr : lo component of the DMA buffer address
 hi_addr : hi component of the DMA buffer address 
 tx_buf_len : length of the DMA buffer

Additionally, each ring has a command buffer to submit commands to the firmware.

There are two command formats:

 struct tx_cmd {
    struct cmd_hdr hdr;
    uint8_t data[320];
 }


 struct tx_cmd_wide {
    struct cmd_hdr_wide hdr;
    uint8_t data[316];
 }


=== Command Headers ===

 struct cmd_hdr {
    uint8_t opcode;
    uint8_t flags;
    uint8_t index;
    uint8_t queue_id;
 }



 struct cmd_hdr_wide {
    uint8_t opcode;
    uint8_t group_id;
    uint8_t index;
    uint8_t queue_id;
    uint16_t length;
    uint8_t rsv;
    uint8_t ver;
 }



== RX Ring (256-byte aligned) ==

There are up to 4096 RX rings (only power of two sizes supported), represented by an array of 32-bit addresses, each ring entry has a byte aligned 32-bit address pointing to an RX buffer. (RX_BUFFER_SIZE = 4096 or 8192 or 16384)

== RX Status Area (16-byte aligned) ==

This is a single region, that takes the following format:

 struct rx_status {
    uint16_t closed_rb_num;
    uint16_t closed_frame_num;
    uint16_t finished_rb_num;
    uint16_t finished_frame_num;
    uint32_t rsv;
 }


 closed_rb_num [0:11] : the index of the reserve buffer that was last closed
 closed_frame_num [0:11] : the index of the frame that was last closed
 finished_rb_num [0:11] : the index of the current reserve buffer, to which the previous frame was written
 finished_frame_num [0:11] : the index of the frame which was last transferred

== RF Hardware switch kill functionality ==

Read CSR_GP_CNTRL, bit HW_RF_KILL_SW to get the switch status, 
    0 = RF kill is on, turn off radio
    1 = RF kill is off, can turn on radio


== Notification Mechanism ==

Firmware related communications are all done via queues, the device responds by sending response commands via the rx ring. The freebsd driver queues these into a notification queue.

== Firmware Parsing ==
=== uCode Format ===


 #define UCODE_MAGIC 0x0a4c5749
 
 struct ucode_hdr {
    uint32_t zr0;
    uint32_t magic;
    uint8_t name[64];
    uint32_t version;
    uint32_t build;
    uint64_t rsv;
    uint8_t data[0];
 }


 struct ucode_tlv {
    uint32_t type;
    uint32_t length;
    uint8_t data[0];
 }

 enum ucode_tlv_type {
    UCODE_TLV_INVALID = 0,
    UCODE_TLV_INST = 1,
    UCODE_TLV_DATA = 2,
    UCODE_TLV_INIT = 3,
    UCODE_TLV_INIT_DATA = 4,
    UCODE_TLV_BOOT = 5,
    UCODE_TLV_PROBE_MAX_LEN = 6,
    UCODE_TLV_PAN = 7,
    UCODE_TLV_RUNT_EVTLOG_PTR = 8,
    UCODE_TLV_RUNT_EVTLOG_SIZE = 9,
    UCODE_TLV_RUNT_ERRLOG_PTR = 10,
    UCODE_TLV_INIT_EVTLOG_PTR = 11,
    UCODE_TLV_INIT_EVTLOG_SIZE = 12,
    UCODE_TLV_INIT_ERRLOG_PTR = 13,
    UCODE_TLV_ENHANCE_SENS_TBL = 14,
    UCODE_TLV_PHY_CALIBRATION_SIZE = 15,
    UCODE_TLV_WOWLAN_INST = 16,
    UCODE_TLV_WOWLAN_DATA = 17,
    UCODE_TLV_FLAGS = 18,
    UCODE_TLV_SEC_RT = 19,
    UCODE_TLV_SEC_INIT = 20,
    UCODE_TLV_SEC_WOWLAN = 21,
    UCODE_TLV_DEF_CALIB = 22,
    UCODE_TLV_PHY_SKU = 23,
    UCODE_TLV_SECURE_SEC_RT = 24,
    UCODE_TLV_SECURE_SEC_INIT = 25,
    UCODE_TLV_SECURE_SEC_WOWLAN = 26,
    UCODE_TLV_NUM_OF_CPU = 27,
    UCODE_TLV_CSCHEME = 28,
    UCODE_TLV_API_CHANGES_SET = 29,
    UCODE_TLV_ENABLED_CAPABILITIES = 30,
    UCODE_TLV_N_SCAN_CHANNELS = 31,
    UCODE_TLV_PAGING = 32,
    UCODE_TLV_SEC_RT_USNIFFER = 34,
    UCODE_TLV_SDIO_ADMA_ADDR = 35,
    UCODE_TLV_FW_VERSION = 36,
    UCODE_TLV_FW_DBG_DEST = 38,
    UCODE_TLV_FW_DBG_CONF = 39,
    UCODE_TLV_FW_DBG_TRIGGER = 40,
    UCODE_TLV_FW_GSCAN_CAPA = 50,
    UCODE_TLV_FW_MEM_SEG = 51,
 }

Sample parsing code at https://gitlab.com/hgoel0974/iwifi_fw_parser

[[Category:Network Hardware]]
````

<a id="w004279"></a>

## W004279 — User:Frabert

Bron-ID: W004279
Namespace: User (2)
Revisie: 22540 | Datum: 2018-08-03T10:59:43Z
Bron: https://wiki.osdev.org/User:Frabert
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AFrabert&oldid=22540

````text
Precompiled Binutils/GCC/GDB/Make/GRUB toolchains for 64bit Windows (x86_64-w64-mingw32): [https://drive.google.com/open?id=1HQ-Ug9y5PKbfIoTnR6o-Wpd3KTKk7a2q]
````

<a id="w004280"></a>

## W004280 — User:TheCool1Kevin/VSCode Debug

Bron-ID: W004280
Namespace: User (2)
Revisie: 30319 | Datum: 2026-03-17T19:05:04Z
Bron: https://wiki.osdev.org/User:TheCool1Kevin/VSCode_Debug
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATheCool1Kevin%2FVSCode_Debug&oldid=30319

````text
{{Rating|1}}

== Preparing your Project ==

If your project is already set up with VSCode, you may want to skip this section. First, you'll need to install VSCode and configure your project with VSCode. To do that, grab a copy of VSCode from [http://www.example.com the official website] (fortunately, it is cross-platform).
For this tutorial, you will need to have configured and installed the '''C/C++''' extension by Microsoft. To install extensions, follow this guide: [https://code.visualstudio.com/docs/editor/extension-gallery VS Code Extension Marketplace].
Furthermore, you may also want to configure Intellisense for your project by following this guide: [https://code.visualstudio.com/docs/languages/cpp C/C++ for VS Code].
Finally, familiarize yourself with the VSCode configuration files, namely:
*'''<code>.vscode/tasks.json</code>'''
*'''<code>.vscode/launch.json</code>'''
If a file listed above does not exist, you will have to manually create it.

== Setting up launch.json ==

Your empty <code>launch.json</code> should look like:
<syntaxhighlight lang="json">
{
    "version": "0.2.0",
    "configurations": []
}
</syntaxhighlight>


Inside <code>"configurations"</code>, create a new entry:


<syntaxhighlight lang="json">
{
    "name": "Launch with GDB",
    "type": "cppdbg",
    "request": "launch",
    "program": "",
    "cwd": "",
    "args": [],
    "sourceFileMap": {
        "<source-path>": "<target-path>"
    },
    "targetArchitecture": "",
    "MIMode": "gdb",
    "miDebuggerPath": "",
    "miDebuggerArgs": "",
    "customLaunchSetupCommands": [
        {
            "text": "target remote localhost:1234",
            "description": "Connect to QEMU remote debugger"
        }
    ],
    "setupCommands": [
        {
            "description": "Enable pretty-printing for gdb",
            "text": "-enable-pretty-printing",
            "ignoreFailures": true
        }
    ],
    "preLaunchTask": "Launch QEMU"
}
</syntaxhighlight>



And configure it based on your project:

{| {{wikitable}}
|-
! Option name
! Descriptionlinux
|-
| <code>program</code>
| The program binary, it doesn't really matter whlinuxat you set it to. I set it to <code>${workspaceRoot}/bin/myos.bin</code>
|-
| <code>cwd</code>
| The current working directory. It really doesn't matter actually due to some issues with VSCode. I set it to <code>${workspaceRoot}</code>
|-
| <code>sourceFileMap</code>
| This will replace the first instance of <code><source-path></code> with <code><target-path></code>.
I set <code><source-path></code> to <code>/mnt/c/</code> and <code><target-path></code> to <code>/C:/</code>.
This was supposed to fix this error I was getting:
<pre>Unable to open 'exceptions.c':
File not found (file:///mnt/c/Users/USER/PATH/TO/FILE/file.c).</pre>
If you're not having this issue, you should remove this option entirely.
|-
| <code>targetArchitecture</code>
| Self explanatory. I set it to x86
|-
| <code>miDebuggerPath</code>
| The path to your gdb executable. I set it to <code>C:/cygwin64/bin/gdb.exe</code>. You may want to set it to <code>/usr/bin/gdb</code>.
|-
| <code>setupCommands</code>
| Explained below. A bunch of GDB commands to be run in '''sequential order.'''
|-
| <code>preLaunchTask</code>
| A task to be run before GDB is launched.
|-
|}

Under <code>setupCommands</code> add:
<syntaxhighlight lang="json">
{
    "text": "file C:/Users/USER/PATH/TO/BINARY/myos.bin",
    "description": "Load binary."
},
{
    "text": "break isr_handler",
    "description": "Break on exception handler."
}
</syntaxhighlight>

The first command loads the binary file into GDB. Make sure that, regardless of your host OS, you include the full path to your OS binary (issue [https://github.com/Microsoft/vscode-cpptools/issues/815 #815].
The second command is optional. It's to catch exceptions.

== Setting up tasks.json ==

Your empty <code>tasks.json</code> should look like:
<syntaxhighlight lang="json">
{
    "version": "2.0.0",
    "tasks": []
}
</syntaxhighlight>

Under <code>"tasks"</code>, create a new entry:
<syntaxhighlight lang="json">
{
    "label": "Launch QEMU",
    "type": "shell",
    "windows": {
        "command": "start -FilePath 'qemu-system-i386' -ArgumentList '-S -gdb tcp::1234 -boot d -cdrom bin/myos.iso -m 512'"
    }
    "linux": {
        "command": "qemu -S -gdb tcp::1234 -boot d -cdrom bin/myos.iso -m 512 -daemonize"
    }
}
</syntaxhighlight>

The <syntaxhighlight>-daemonize</syntaxhighlight> option for qEMU will start it asynchronously in the background. '''Note that you cannot use stdio output with this.''' You should probably redirect your serial output to a file.

Notice that VSCode for Windows executes Powershell which makes it a pain in the ass to start a process asynchronously.

Also, note that the value under <code>label</code> corresponds to the value under <code>preLaunchTask</code> as we are launching QEMU before GDB starts.

== Debugging ==

VSCode is not set up to debug normally. Press <code>F5</code> (or <code>FN+F5</code>) to debug.
The IDE provides a nice graphical wrapper to GDB and an easy way to start the debugger from within the IDE. All features supported with GDB is supported with VSCode.

Some things you can do from within the debug console:
* <code>-exec</code> executes a GDB command, i.e., <code>-exec info registers</code> executes the command <code>info registers</code>
* You can evaluate expressions with graphical display of the data, for example:
** <code>(struct myStruct *) 0xCAFEBABE</code> will display the memory contents casted to the struct (there's a nice GUI tree view).
** <code>myVariable</code> will print out the value of that vaariable.
** <code>myStruct -> member</code> works too
** <code>*(myPointer + 2)</code> works
** Basically anything short of evaluating functions works.

If you have any issues, let me know (somehow)!

== Screenshots ==

See local variables, stack trace and all that good stuff.

[[File:Vscode_debug.PNG|700px||VSCode Debug]]

<br />
By typing <code>-exec info registers</code> into the debug console, you are able to see all the registers.

[[File:Vscode_dbg1.PNG|600px||VSCode Debug]]

<br />
Exception catching works!

[[File:Vscode_dbg2.PNG|700px||VSCode Debug]]

[[Category:OS Development]]
[[Category:Tutorials]]
[[Category:Tools]]
[[Category:IDEs]]
````

<a id="w004281"></a>

## W004281 — File:Vscode debug.PNG

Bron-ID: W004281
Namespace: File (6)
Revisie: 22543 | Datum: 2018-08-03T22:32:55Z
Bron: https://wiki.osdev.org/File:Vscode_debug.PNG
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AVscode_debug.PNG&oldid=22543

````text
Debugging with VSCode.
````

<a id="w004282"></a>

## W004282 — File:Vscode dbg1.PNG

Bron-ID: W004282
Namespace: File (6)
Revisie: 22544 | Datum: 2018-08-03T22:40:02Z
Bron: https://wiki.osdev.org/File:Vscode_dbg1.PNG
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AVscode_dbg1.PNG&oldid=22544

````text

````

<a id="w004283"></a>

## W004283 — File:Vscode dbg2.PNG

Bron-ID: W004283
Namespace: File (6)
Revisie: 22546 | Datum: 2018-08-03T22:45:52Z
Bron: https://wiki.osdev.org/File:Vscode_dbg2.PNG
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AVscode_dbg2.PNG&oldid=22546

````text

````

<a id="w004284"></a>

## W004284 — User:No92/UEFI Bare Bones

Bron-ID: W004284
Namespace: User (2)
Revisie: 23761 | Datum: 2019-07-17T20:15:04Z
Bron: https://wiki.osdev.org/User:No92/UEFI_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANo92%2FUEFI_Bare_Bones&oldid=23761

````text
{{Rating|2}}

We will be creating a minimal example capable of booting on UEFI, printing "Hello World" and waiting for a keystroke before returning.

==Prerequisites==
* clang and lld (specifically lld-link)
* qemu-system-x86_64
* make, git, wget, …

==Setup==
We will be utilizing Zircon's headers for UEFI, as they offer a clean interface without some insane build system (TianoCore) or ugly hacks (gnu-efi). You can extract them from the [https://github.com/fuchsia-mirror/zircon source] yourself (they are located at <code>system/private/efi</code>, or use [https://github.com/vineyard-os/zircon-uefi this mirror]. Place them in the kernel/include directory.

==Makefile==
Next, set up a Makefile with the following contents:
<source lang="make">
CC		:= clang
LD		:= lld-link-6.0
EMU		:= qemu-system-x86_64

CFLAGS		:= -ffreestanding -flto -fno-stack-protector -fpic -fshort-wchar -Ikernel/include -MMD -mno-red-zone -std=c11 -target x86_64-pc-win32-coff -Wall -Wextra
LDFLAGS		:= -subsystem:efi_application -nodefaultlib -dll
EMUFLAGS	:= -drive if=pflash,format=raw,file=bin/OVMF.fd -drive format=raw,file=fat:rw:bin/hdd -M accel=kvm:tcg -net none -serial stdio

OBJ		:= kernel/kernel.o
KERNEL		:= bin/hdd/efi/boot/bootx64.efi

OVMF_URL	:= https://dl.bintray.com/no92/vineyard-binary/OVMF.fd
OVMF_BIN	:= OVMF.fd
OVMF		:= bin/$(OVMF_BIN)

$(KERNEL): $(OBJ)
	mkdir -p $(dir $@)
	$(LD) $(LDFLAGS) -entry:efi_main $< -out:$@

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

-include $(OBJ:.o=.d)

test: $(KERNEL) $(OVMF)
	$(EMU) $(EMUFLAGS)

$(OVMF):
	mkdir -p bin
	wget $(OVMF_URL) -O $(OVMF) -qq

clean:
	rm -f $(KERNEL)
	rm -f $(OBJ) $(OBJ:.o=.d)

.PHONY: test clean

</source>

===Why do I need this option?===
* -ffreestanding: lets the compiler know that you are not in a hosted environment
* -flto: enable lld to use LTO (link-time optimization)
* -fno-stack-protector: the [[Stack-Smashing Protector]] needs (trivial) runtime support, which isn't provided here
* -fpic: UEFI requires the resulting binary to be position-independent
* -fshort-wchar: UEFI specifies wchar to be 16 bits long
* -mno-red-zone: [[Why_do_I_need_a_Cross_Compiler%3F#-mno-red-zone_.28x86_64_only.29|you should disable this for x86_64 kernel code]]
* -target x86_64-pc-win32-coff: UEFI applications are some variation of Windows' PE binaries

==kernel/kernel.c==
<source lang="c">
#include <efi/boot-services.h>
#include <efi/runtime-services.h>
#include <efi/system-table.h>
#include <efi/types.h>

#include <stdbool.h>

/* I'm too lazy to type this out five times */
#define ERR(x) if(EFI_ERROR((x))) return (x)

efi_status efi_main(efi_handle handle __attribute__((unused)), efi_system_table *st) {
	efi_status status;
	efi_input_key key;

	/* reset the watchdog timer */
	st->BootServices->SetWatchdogTimer(0, 0, 0, NULL);
	ERR(status);

	/* clear the screen */
	status = st->ConOut->ClearScreen(st->ConOut);
	ERR(status);

	/* print 'Hello World' */
	status = st->ConOut->OutputString(st->ConOut, L"Hello World");
	ERR(status);

	/* flush console input buffer */
	status = st->ConIn->Reset(st->ConIn, false);
	ERR(status);

	/* poll for a keystroke */
	while((status = st->ConIn->ReadKeyStroke(st->ConIn, &key)) == EFI_NOT_READY);
	ERR(status);

	return EFI_SUCCESS;
}

</source>

==Building==
Run <source lang="bash" enclose="none">make test</source> to build the kernel and run it in QEMU.

==What's next?==
* getting a memory map
* setting up a display mode with the Graphics Output Protocol
* getting the hell out of UEFI with <code>ExitBootServices</code>

==See Also==
===Articles===
* [[UEFI]]
===External Links===
* https://www.rodsbooks.com/efi-programming/hello.html
````

<a id="w004285"></a>

## W004285 — Brendan's Multi-tasking Tutorial

Bron-ID: W004285
Namespace: Main (0)
Revisie: 30636 | Datum: 2026-05-26T21:59:31Z
Bron: https://wiki.osdev.org/Brendan%27s_Multi-tasking_Tutorial
Revisie-URL: https://wiki.osdev.org/index.php?title=Brendan%27s_Multi-tasking_Tutorial&oldid=30636

````text
==Overview==

This tutorial will describe a way to implement multi-tasking and task switching for a kernel that uses "kernel stack per task". It has been designed to allow the reader to implement a full featured scheduler (while avoiding common pitfalls) in steps that can be tested before moving on to the next step.

Note: I use the word "task" as generic term to mean "the thing that a scheduler schedules". For operating systems that don't support multi-threading, a task might be a process. For operating systems that do support multi-threading a task would be a thread.

==Step 1: Initialization and <code>switch_to_task()</code>==

==== Thread Control Block ====
Start by creating some kind of data structure that will keep track of a task's information. Typically this is called a "process control block" or "thread control block". At a minimum, this structure needs to contain:
* The task's kernel stack top.
* The task's virtual address space (e.g. the value for CR3 on 80x86 systems).
* A "next task" field that can be used for multiple different linked lists of tasks later on.
* A "state" field (also to be used later).
<syntaxhighlight lang="c" line="1">
struct thread_control_block
{
    void* esp;
    void* esp0;
    void* cr3;
    thread_control_block* next;
    uint8_t state;
};
</syntaxhighlight>It might also contain other/optional fields, like:
* the scheduling policy of the task
* the scheduling priority of the task
* the ID of the process the task/thread belongs to
* a "task name" string
* the amount of CPU time the task has consumed so far

Don't forget that you can come back and add fields to this structure whenever you like - you don't need to have everything initially.

==== Initialization ====
Once you've created an initial data structure for keeping track of a task's information; create an <code>initialize_multitasking</code> function. The idea here (initially) is that there's already a task that has been running since boot, and you only need to create the information for it by allocating some memory for the task's information structure that you just created and setting the fields in that structure as appropriate.<syntaxhighlight lang="c" line="1">
void initialize_multitasking();
</syntaxhighlight>

==== Switching Tasks ====
Next; write a low-level function to switch from one task (that is assumed to be running kernel code) to another task (that is also assumed to be running kernel code).

Note that the low-level task switch code should be written in pure assembly. If it's written in inline assembly in the middle of a C function then the compiler may add its own "function prologue" and "function epilogue" code that changes the layout of the stack. This is important because when a new task is being created the kernel needs to put values on the new task's kernel stack to match the values that the <code>switch_to_task</code> expects to pop off of the new task's stack.

For example (NASM syntax, 32-bit 80x86, <code>cdecl</code> calling convention, single-CPU only/globals used for "per CPU" variables, no support for FPU/MMX/SSE/AVX, untested):<syntaxhighlight lang="c">
extern void switch_to_task(thread_control_block* next_thread);
extern thread_control_block* current_task_TCB;
</syntaxhighlight><syntaxhighlight lang="asm" line="1">
;C declaration:
;   void switch_to_task(thread_control_block *next_thread);
;
;WARNING: Caller is expected to disable IRQs before calling, and enable IRQs again after function returns

switch_to_task:

    ;Save previous task's state

    ;Notes:
    ;  For cdecl; EAX, ECX, and EDX are already saved by the caller and don't need to be saved again
    ;  EIP is already saved on the stack by the caller's "CALL" instruction
    ;  The task isn't able to change CR3 so it doesn't need to be saved
    ;  Segment registers are constants (while running kernel code) so they don't need to be saved

    push ebx
    push esi
    push edi
    push ebp

    mov edi,[current_task_TCB]    ;edi = address of the previous task's "thread control block"
    mov [edi+TCB.ESP],esp         ;Save ESP for previous task's kernel stack in the thread's TCB

    ;Load next task's state

    mov esi,[esp+(4+1)*4]         ;esi = address of the next task's "thread control block" (parameter passed on stack)
    mov [current_task_TCB],esi    ;Current task's TCB is the next task TCB

    mov esp,[esi+TCB.ESP]         ;Load ESP for next task's kernel stack from the thread's TCB
    mov eax,[esi+TCB.CR3]         ;eax = address of page directory for next task
    mov ebx,[esi+TCB.ESP0]        ;ebx = address for the top of the next task's kernel stack
    mov [TSS.ESP0],ebx            ;Adjust the ESP0 field in the TSS (used by CPU for for CPL=3 -> CPL=0 privilege level changes)
    mov ecx,cr3                   ;ecx = previous task's virtual address space

    cmp eax,ecx                   ;Does the virtual address space need to be changed?
    je .doneVAS                   ; no, virtual address space is the same, so don't reload it and cause TLB flushes
    mov cr3,eax                   ; yes, load the next task's virtual address space
.doneVAS:

    pop ebp
    pop edi
    pop esi
    pop ebx

    ret                           ;Load next task's EIP from its kernel stack
</syntaxhighlight>

==== Creating New Tasks ====
Next; write a function to create a new kernel task. This is a little bit like the code you already wrote for "initialise_multitasking()" (because it would allocate some memory for the structure used to keep track of the task's information and fill it in); except that you will need to allocate a new kernel stack for the new task and put values on the new kernel stack to match the values that your "switch_to_task(task)" function expects to pop off the stack after switching to the task. Don't forget that the values on the new kernel stack will include a "return EIP" (the address that the "switch_to_task(task)" function will return to), which should come from an input parameter of the "create_kernel_task()" function (e.g. like "create_kernel_task(void * startingEIP)" except probably with a function pointer instead of a void pointer if you can remember the correct syntax for that).

Note that you can have a "task start up function" that is executed when a new task first gets CPU time and does a few initialisation things and then passes control to the task's normal code. In this case the new kernel stack will include a "return EIP" that contains the address of the "task start up function", plus an extra "return EIP" (for when the "task start up function" returns) that contains the address of the task itself (taken from an input parameter of the "create_kernel_task()" function). This is a little slower, but avoids the need for each task's code to do any initialisation itself.

Finally; test everything you've done for this step by creating a second kernel task; where the existing first kernel task has loop that uses "switch_to_task()" to switch to the new second kernel task, and the new second task has a loop that does "switch_to_task" to switch to the existing first task. If everything works well you'll end up with two tasks that constantly switch to each other. If you like you can do more testing (e.g. have lots of tasks where one task switches to the next and the last task switches back to the first). It's also fun to see out how many task switches per second your code is capable of.


==Step 2: "Schedule()" Version 1==

Having tasks that explicitly switch to each other is a little inconvenient, and it'd be nice if the kernel could automatically select a task to switch to next, so the next step is to write a function that does this.

''Note: If you're confident in your abilities you might be tempted to skip this step and go directly to "Step 4: "Schedule()" version 2" below; but this step is so simple that skipping it won't save much time and it's probably better to get a little experience with "extremely simple" before moving on to "very simple".''

The code to select which task to give CPU time to next has a huge effect on the nature and "feel" of the OS; and it's possible for this code to become very complicated for this reason. However, it's better to start simple ("learn to walk before you run") so we're going to start with the simplest (and most awful) code possible. More specifically, we're going to start with a cooperative round robin scheduler; and then (later on) we're going to replace it with something a little more complex (that is a lot less awful).

''Note: Round robin is awful because when the OS is under heavy load and something happens you need to wait for all of those tasks to use the CPU before the OS can respond. For example, if there are 1234 tasks running and each one uses an average of 5 ms of CPU time, and the user presses a key; then it will take over 6 seconds for the task that responds to the key press to get CPU time and the user will not be impressed with how laggy your OS is. Some people try to fix this problem by putting tasks that are unblocked at the start of the list, but that doesn't work well because a task can deliberately "block then unblock" to keep itself at the start of the list and hog all of the CPU time while preventing other tasks from getting any CPU time.''

To implement a round robin scheduler, you mostly only need a circular linked list. For a circular linked list, each item in the list points to the next item in the list, and the last item in the list points back to the first item in the list. In other words, we just need a "next task" field to implement it; and when we created the data structure to keep track of a task's information we already included a field that can be used for this purpose.

Essentially; you just need to set the "next_task" field in the data structure for each task's information in both the code to initialise multi-tasking and the code to create a new task.

Once that's done, you can write a "schedule()" function that selects a task to switch to and then switches to the selected task. You only need single line of code to select the next task, like "selected_task = current_task_TCB->next_task;" and you already have a function to switch to a specific task; and both of these things can be combined. In other words, the "Schedule()" function can contain a single line of code like "switch_to_task(current_task_TCB->next_task);".

As soon as you've finished implementing "schedule()" you can test it using the kernel tasks you were using for testing earlier, just by replacing the calls to "switch_to_task()" with calls to "schedule()". If you had multiple additional kernel tasks with different code for each kernel task (so it could switch to the right next task) you can also delete the duplicates so that you end up with multiple additional kernel tasks executing the same code.

==Step 3: Time Accounting (Optional)==

Sooner or later you'll probably want to keep track of things like the total amount of CPU time each task has consumed (which can be used for things like calculating the average CPU load for each task, average time spent idle, etc).

''Note: A timer is something that either generates an IRQ after a requested amount of time has passed, or generates an IRQ at a specific frequency. A counter is something that keeps a count that can be read whenever you like. A timer may not be a counter (it might not have a "current count" that you can read), and a counter may not be a timer (it might not be able to generate an IRQ).''

To do this you'll need some kind of time counter, like the CPU's time stamp counter, HPET's main counter or ACPI's counter. If the computer is too old to have an actual counter you may need to implement one yourself using a timer (e.g. by configuring a timer to generate an IRQ at a specific frequency and then doing something like "ticks_since_boot++;" in the timer's IRQ handler).

For time accounting, when certain things happen you want to subtract the counter's current value from the value it had last time (to determine how much time elapsed since last time); then add the time elapsed to the amount of time that the current task has consumed and update the time the counter was read last, a little bit like this:

<syntaxhighlight lang="c">
void update_time_used(void) {
    current_count = read_counter();
    elapsed = last_count - current_count;
    last_count = current_count;
    current_task_TCB->time_used += elapsed;
}
</syntaxhighlight>

The first place this needs to be called is during task switches, to update the amount of time that the previously running task consumed before switching to the next task. For extra goodness "update_time_used()" can also be called when someone asks the kernel how much time a task has consumed, so that (e.g.) if a task has been running for 10 ms already then that 10 ms is included in the time a task has consumed.

The other place you might want to update the amount of time a task has consumed is immediately after the CPU changes from user-space code to kernel code and immediately before the CPU changes from kernel code to user-space code; so that you can keep track of "amount of time task spent running kernel code" and "amount of time task spent running user-space code" separately.

''Note that it's a good idea to make the time accounting a little abstract and use a relatively high precision. For example, rather than just doing "current_task_TCB->time_used += elapsed;" you might convert "elapsed" into nanoseconds. That way (regardless of what kind of counter the computer supports and/or if you add support for more modern counters later) the time accounting will always use the same time scale.''

Finally; test to make sure that your scheduler is keeping track of the time each task consumes correctly.


==Step 4: Task States and "Schedule()" Version 2==

For a real OS (regardless of which scheduling algorithm/s the scheduler uses) most tasks switches are caused because the currently running task has to wait for something to happen (data received from a different task or storage devices/file system or network, or time to pass, or for user to press a key or move the mouse, or for a mutex to become available, etc), or happen because something that a task was waiting for happened causing a  task to unblock and preempt the currently running task.

To prepare for this, we want to add some kind of "state" to each task so that it's easy to see what a task is doing (e.g. if it's waiting and what it's waiting for).

For task's that aren't waiting; I find it easier (later, when you add support for blocking and unblocking tasks) to have a "running" state (when the task is currently using the CPU) and a separate "ready to run" state. The idea is that when a task is given CPU time it is removed from the scheduler's list of "ready to run" tasks and its state is changed from "ready to run" to "running"; and when a task stops using CPU time either it's state is changed back to "ready to run" and it is put back on the scheduler's list of "ready to run" tasks, or its state is changed to "waiting for something" and it is not put back on the scheduler's list of "ready to run" tasks.

Note that making this change will involve changing the scheduler's list of tasks from a circular linked list into a "non-circular" linked list; and (because we've been using a singly linked list and have been using the "current_task_TCB" variable as a temporary "start of linked list" variable) you will need to add two new global variables - one for the start of the linked list and one for the end of the linked list. Initially both of these variables will be NULL because there's no tasks on the list, when the first task is added both of the variables will be set to the first task's information (when there is only one task on the list it must be the first and last task), and when more tasks are added they just get appended to the list (like "last_task->next_task = new_task; last_task = new_task;").

After creating the new "first and last task on the scheduler's ready to run list" variables; update the function that initialises multi-tasking so that it sets the initial task's state to "running" (and delete any older code that put the initial task on the scheduler's list of tasks); and then update the code that creates new kernel tasks so that the new task's state is set to "ready to run" just before the new task is added to the scheduler's "ready to run" list.

The next change is to update the low-level "switch_to_task()" function, so that if and only if the previously running task is still in the "running" state the previously running task's state is changed from "running" to "ready to run" and the previously running task is put back on the scheduler's list of "ready to run" tasks; and then set the state of the next task to be given CPU time to "running" (regardless of what state it was in before, and without removing it from any list).

Finally; the "schedule()" function needs to be changed so that it removes the task from the "ready to run" list before doing a task switch, making it something like this:

<syntaxhighlight lang="c">
void schedule(void) {
    if( first_ready_to_run_task != NULL) {
        thread_control_block * task = first_ready_to_run_task;
        first_ready_to_run_task = task->next;
        switch_to_task(task);
    }
}
</syntaxhighlight>

Note that if there's only one task you don't really want to do any task switches (because there's no other tasks to switch to), and in this case the task will be "running" and the list of "ready to run" tasks will be empty. The previous version of "schedule()" was a little less efficient (it would've done a task switch from one task to the same task).

==Step 5: Race Conditions and Locking Version 1==

So far we haven't had to care about things like race conditions and locking because its a cooperative scheduler and there's only one CPU. However, before the scheduler's code can progress much further we're going to need some way to avoid race conditions and ensure everything happens in a correct order.

When there's a single CPU, in theory, this can be done just by disabling IRQs in places that matter. For multiple CPUs this doesn't work (e.g. disabling IRQs on one CPU won't stop a different CPU from modifying the scheduler's list of "ready to run" threads at an unfortunate time) and you need something more robust (e.g. spinlock).

We won't be implementing support for multiple CPUs; but that doesn't mean we can't (and shouldn't) make it a little easier to add support for multiple CPUs much much later. For this reason I'm going to pretend that there's a "big scheduler lock" and implement functions to acquire and release this lock, but then I'm not going to actually have a lock and I'm just going to disable and enable IRQs instead.

Unfortunately there's a minor problem - what if IRQs are already disabled for some other reason? For assembly language this isn't a problem - you can PUSHFD to save the original flags and disable IRQs when you enter a critical section, and then POPFD the original flags when you leave the critical section. For higher level languages you can't just leave stuff on the stack like this because the compiler will get horribly confused and think that local variables are in the wrong places, etc.

To work around that problem, there's two alternatives - have a function that returns the old EFLAGS as an output parameter and disables IRQs, then another function that restores the old EFLAGS from an input parameter, where the caller has to store the old EFLAGS value somewhere itself. The other alternative is to keep a counter that keeps track of how many pieces of code have wanted IRQs disabled, where you'd increment the counter when disabling IRQs at the start of a critical section, and decrement the counter and only enable IRQs if the counter is zero at the end of a critical section.

The first option is probably more efficient; but we don't really care much about getting the highest performance possible at this stage, and because I want to make it look a little bit like a lock (to make it easier to support multiple CPUs later) I'm going to use the second option. It might look like this:

<syntaxhighlight lang="c">
int IRQ_disable_counter = 0;

void lock_scheduler(void) {
#ifndef SMP
    CLI();
    IRQ_disable_counter++;
#endif
}

void unlock_scheduler(void) {
#ifndef SMP
    IRQ_disable_counter--;
    if(IRQ_disable_counter == 0) {
        STI();
    }
#endif
}
</syntaxhighlight>

Once that's done, you need to add a comment to your "schedule()" function and to your "switch_to_task()" function saying that the caller is responsible for making sure the scheduler is locked before calling.

Then you need to comply with the new comment by locking and unlocking the scheduler at the right places. Fortunately you only have a few kernel tasks that call "schedule()" directly; so you only need to add locking/unlocking to them. The end result might look a little like this:

<syntaxhighlight lang="c">
void test_kernel_task(void) {
    // unlock_scheduler();

    for(;;) {
        lock_scheduler();
        schedule();
        unlock_scheduler();
    }
}
</syntaxhighlight>

This can be a little confusing depending on your perspective. From the perspective of one task, it locks the scheduler, does a task switch, and then when it gets CPU time again it unlocks the scheduler; and this is fine (the scheduler is always unlocked after its locked). From the perspective of the CPU, one task locks the scheduler then the scheduler does a task switch and a completely different task unlocks the scheduler; and this is also fine (the scheduler is still unlocked after its locked).

Note that "one task locks the scheduler then the scheduler does a task switch and a completely different task unlocks the scheduler" means that one of the first things that a new task will need to do (when it gets CPU time) is unlock the scheduler's lock. If you decided to use a "task start up function" in Step 1, then the "task start up function" can unlock the schedulers lock before returning to the task's code, and if you decided not to do that then you'll need to have an "unlock_scheduler();" at the start of every kernel task.

Finally, you should test to make sure your multi-tasking still works the same.

==Step 6: Blocking and Unblocking Tasks==

When the currently running task blocks, the kernel will put the task on some other list or structure (e.g. a list of tasks that called "sleep()" and are waiting to be woken up, a list of tasks waiting to acquire a mutex, etc) and change the task's state to "blocked for whatever reason". When a task in unblocked it will be removed from whatever list or structure it was on, and either its state will be changed to "ready to run" and it'll be put back on the scheduler's "ready to run" list, or it's state will be changed to "running" and it won't be put on any list.

Rather than duplicating this logic in all of the places that block and unblock tasks; it's nice for the scheduler to provide functions that can be used for blocking and unblocking tasks.

Blocking a task is incredibly easy (mostly because we prepared for it in the last step by making sure a task in the "running" state is not on the scheduler's "ready to run" list). You only need to set the task's state and then find another task to run, which might look a little like this:

<syntaxhighlight lang="c">
void block_task(int reason) {
    lock_scheduler();
    current_task_TCB->state = reason;
    schedule();
    unlock_scheduler();
}
</syntaxhighlight>

Unblocking a task is a little more involved. The first step is to decide if the task being unblocked should (or shouldn't) preempt the currently running task. For now (because we don't have any fancy task priorities or anything) we could never preempt, or we could only preempt if there was only one running task (because it's likely that if there was only one running task that task has been able to use lots of CPU time anyway). For fun (and because it helps to show how preemption would work) I'm going to go with the latter option.

If the scheduler decides that no preemption happens, it just sets the unblocked task's state to "ready to run" and puts the task on the scheduler's "ready to run" list.

If the scheduler decides that preemption will happen, it just calls "switch_to_task()" to switch immediately to the unblocked task.

The code might look something like this:

<syntaxhighlight lang="c">
void unblock_task(thread_control_block * task) {
    lock_scheduler();
    if(first_ready_to_run_task == NULL) {

        // Only one task was running before, so pre-empt

        switch_to_task(task);
    } else {
        // There's at least one task on the "ready to run" queue already, so don't pre-empt

        last_ready_to_run_task->next = task;
        last_ready_to_run_task = task;
    }
    unlock_scheduler();
}
</syntaxhighlight>

The other thing we'll need is a reason to block a task. For this, we can add the ability for a task to voluntarily pause itself until some other task un-pauses it, which is something that might be useful later. To implement this we only need to define a new task state (PAUSED).

Finally, you can test the blocking and unblocking by having one of the kernel tasks pause itself (e.g. "block_task(PAUSED);"), and then get the next kernel task that will be given CPU time to unpause the task that paused itself. Note that you will need to be a little careful here because you don't want to unpause the task before it pauses itself, but we're still using a cooperative round robin scheduler so it's easy to determine which order tasks will be given CPU time and avoid that problem.


==Step 7: "Sleep()", "Nano_sleep()" and "Nano_sleep_until()"==

Sometimes a task wants to wait for time to pass (e.g. maybe a text editor wants to create a backup of your unsaved work every 15 minutes, so it has a task that sleeps for 15 minutes and saves your work when it wakes up). Sometimes a task wants to wait for a very small amount of time to pass (e.g. maybe a device driver needs to reset its device and then wait for 20 milliseconds). For both of these cases you want to block the task so that it doesn't waste any CPU time, and then have a timer IRQ unblock the task when the requested amount of time has passed.

Of course it's impossible to guarantee that a task will be blocked for exactly the amount of time it requested; and even if it is possible there's no guarantee that the scheduler would give the task CPU time as soon as it's unblocked. The only guarantee that can be made is that the task will not be unblocked too early, and an OS should provide this guarantee (and should warn programmers that their task may be unblocked later than they wanted).

Sometimes a task wants to do something regularly (e.g. maybe it's displaying an animated icon and needs to update the icon every 250 ms). For this, using something like "nano_sleep()" doesn't work properly. If you have a loop that updates the icon then waits for 250 ms; what happens is that it takes a little time to update the icon and the OS doesn't guarantee you'll be blocked for exactly 250 ms (and CPU time used by other tasks will make the delay erratic), so the icon might be updated after 300 ms the first time, then after 600 ms the next time, an so on; and on average it might be updated every 500 ms. There's no way to fix this problem using something like "nano_sleep()" because it uses relative time (e.g. "250 ms from now").

To fix the problem you need a function that uses absolute time instead of relative time (e.g. "sleep until time since boot is >= 123456"). This might look a little like this:

<syntaxhighlight lang="c">
    delay = 250000000;   // 250000000 nanoseconds is 250 milliseconds
    next_update = get_time_since_boot();
    for(;;) {
        do_something();
        next_update += delay;
        nano_sleep_until(next_update);
    }
</syntaxhighlight>

For code like this, the time between calls to "do_something()" still varies, but if its possible to call "do_something" often enough you can guarantee that these variations will average out and that "do_something()" will be called every 250 ms on average. Of course if there's a design flaw and "do_something()" takes too long (e.g. longer than 250 ms), or if there's so many other tasks consuming CPU time that this task can't keep up (which is a problem that can be fixed with task priorities that we don't have yet), then it isn't possible to call "do_something" often enough - if that happens, the "nano_sleep_until(next_update);" will do nothing (no delay at all) and "do_something()" will be called as often as possible, which is still the best behaviour anyone can expect in those conditions.

Fortunately, if you have "nano_sleep_until()" then you can use it to implement "nano_sleep()", like this:

<syntaxhighlight lang="c">
void nano_sleep(uint64_t nanoseconds) {
    nano_sleep_until(get_time_since_boot() + nanoseconds);
}
</syntaxhighlight>

In a similar way, if you have either "nano_sleep_until()" or "nano_sleep()" you can use them to implement "sleep()" - you only need to multiply seconds by 1000000000 to get nanoseconds.

What this means is that "nano_sleep_until()" is the only function that the scheduler really needs to care about.

To implement "nano_sleep_until()"; the general idea is to put the task onto a list of sleeping tasks and block the task; and then have some sort of timer (not a time counter like we used earlier, but a timer that has an IRQ) to check the list of tasks and unblock any tasks that need to be woken up.

The problem here is that the timer's IRQ handler might need to wake up several tasks at the same time, and as soon as it wakes up one task the scheduler might decide to do a task switch; and that task switch will make it impossible for the timer IRQ handler to wake up the other tasks at the right time. We need to fix that problem before implementing "nano_sleep_until()".

Of course this problem will happen on other places too (e.g. multiple tasks waiting for the same data from disk being unblocked at the same time, multiple tasks waiting for a "pthread_cond_broadcast()", etc). We want a generic solution that solves the problem everywhere, not just a hack for "nano_sleep_until()".

What we really want is a way to tell the scheduler to postpone any task switches until we're ready. More specifically, we want a way to tell the scheduler to postpone task switches while the kernel is holding any lock (and do the postponed task switch/es when we aren't holding a lock any more), and we want to do this for all locks including the scheduler's lock.


==Step 8: Locking Version 2==

To tell the scheduler to postpone task switches while we're holding one lock, you just need a "postpone task switches" flag. However, what if locks are nested (e.g. where you acquire two different locks, and then release one, then release the other)? To make it work for nested locks you need a counter to keep track of how many locks the CPU is currently holding.

Essentially; when you acquire a lock you increase the counter, and when you release the lock you decrease the counter; and task switches are postponed whenever the counter is non-zero.

So, how do you postpone task switches?

This is relatively easy - before doing a task switch (in both "schedule()" and "switch_to_task()") the scheduler checks if task switches are being postponed (by checking if the counter is zero or non-zero); and if task switches are being postponed (counter is non-zero) the scheduler sets a "task switch was postponed" flag and doesn't do the task switch at all. Then, when a lock is released you decrement the counter and see if the counter became zero; and if the counter did become zero you check if the scheduler set the "task switch was postponed" flag, and if it was you clear it and ask the scheduler to do the postponed task switch by calling "schedule()" again.

We already have code for locking and unlocking the scheduler; but you can't really use the same lock that the scheduler uses because you don't want the scheduler to disable task switching every time it needs to do a task switch, so we're going to have a different lock for miscellaneous stuff instead. Ideally you'd have many locks, but that's overkill for a tutorial. Of course we still won't be adding support for multiple CPUs and we'll still be enabling/disabling IRQs instead of having an actual lock. 

It might look a little like this:

<syntaxhighlight lang="c">
int postpone_task_switches_counter = 0;
int task_switches_postponed_flag = 0;

void lock_stuff(void) {
#ifndef SMP
    CLI();
    IRQ_disable_counter++;
    postpone_task_switches_counter++;
#endif
}

void unlock_stuff(void) {
#ifndef SMP
    postpone_task_switches_counter--;
    if(postpone_task_switches_counter == 0) {
        if(task_switches_postponed_flag != 0) {
            task_switches_postponed_flag = 0;
            schedule();
        }
    }
    IRQ_disable_counter--;
    if(IRQ_disable_counter == 0) {
        STI();
    }
#endif
}
</syntaxhighlight>

The "schedule()" function only needs a few lines to check "postpone_task_switches_counter" and set "task_switches_postponed_flag", and that might end up a looking like this:

<syntaxhighlight lang="c">
void schedule(void) {
    if(postpone_task_switches_counter != 0) {
        task_switches_postponed_flag = 1;
        return;
    }
    if( first_ready_to_run_task != NULL) {
        thread_control_block * task = first_ready_to_run_task;
        first_ready_to_run_task = task->next;
        switch_to_task(task);
    }
}
</syntaxhighlight>

You'd need to make the same little change at the start of the "switch_to_task()" function, but that's "architecture specific assembly". For 32-bit 80x86 it might just a few instructions inserted at the start of the function (before all other instructions), like:

<syntaxhighlight lang="asm">
    cmp dword [postpone_task_switches_counter],0
    je .continue
    mov dword [task_switches_postponed_flag],1
    ret

.continue:
</syntaxhighlight>

Of course if you wanted to you could have a high level language wrapper that does this before calling the low level assembly function - either way is fine (I don't like wrappers much, but if you care about portability a wrapper would mean there's a little less code to change when porting).

Finally, test the code to make sure it still works!



==Step 9: "Nano_sleep_until()" Again==

Now that we've improved locking, we can acquire a lock (e.g. the scheduler's lock) and wake up lots of sleeping tasks with caring if that will or won't cause task switches (or if a task we unblock preempts the task we're using), and then simply release the lock; so we're ready to implement "nano_sleep_until()".

To start with we're going to need some kind of structure to keep track of sleeping tasks. Good ways to do this are complicated (e.g. "time buckets") and we don't want complicated, so let's use an unsorted linked list (which isn't great because we'll need to check every task on the list, but it will work and it can be replaced with something more complicated later).

The other thing I should explain is that there's two very different ways of using a timer. The easiest way is to use a timer that generates an IRQ at a fixed frequency. A much better way is to determine how long until the timer IRQ is needed next and then configure the timer to generate an IRQ at that time; because this gives you a lot less unnecessary IRQs and also gives you a lot better precision.

I'm going to use the easy way, not just because it's easier but also because we'll be using an unsorted list (which makes it harder to determine how long until a timer IRQ is needed next).

For "nano_sleep_until()" you'd store the time the task is supposed to wake up somewhere, put the task on the unsorted list, then block the task. You'll need to add a "wake up time" field to the data structure you're using to keep track of tasks for this, and you'll need to define a new task state ("SLEEPING"). The code might look like this:

<syntaxhighlight lang="c">
thread_control_block * sleeping_task_list = NULL;

void nano_sleep_until(uint64_t when) {
    lock_stuff();

    // Make sure "when" hasn't already occured

    if(when < get_time_since_boot()) {
        unlock_scheduler();
        return;
    }

    // Set time when task should wake up

    current_task_TCB->sleep_expiry = when;

    // Add task to the start of the unsorted list of sleeping tasks

    current_task_TCB->next = sleeping_task_list;
    sleeping_task_list = current_task_TCB;

    unlock_stuff();

    // Find something else for the CPU to do

    block_task(SLEEPING);
}
</syntaxhighlight>

Note that we're using the same "next" field in the data structure that the kernel uses to keep track of a task's information that the scheduler uses for its list of "ready to run" tasks. A task can't be ready to run and sleeping at the same time, so this is fine.

The code for the timer IRQ handler depends a little on which timer you're using. If you're using the PIT chip on 80x86 PCs, and the PIT chip has been set to a frequency of about 1000 Hz, then the timer IRQ handler might look like this:

<syntaxhighlight lang="c">
uint64_t time_since_boot = 0;
uint64_t time_between_ticks = 1000000;  // 1000 Hz = 1 ms between ticks = 1000000 nanoseconds between ticks

void PIT_IRQ_handler(void) {
    thread_control_block * next_task;
    thread_control_block * this_task;

    lock_stuff();
    time_since_boot += time_between_ticks;

    // Move everything from the sleeping task list into a temporary variable and make the sleeping task list empty

    next_task = sleeping_task_list;
    sleeping_task_list = NULL;

    // For each task, wake it up or put it back on the sleeping task list

    while(next_task != NULL) {
        this_task = next_task;
        next_task = this_task->next;

        if(this_task->sleep_expiry <= time_since_boot) {
            // Task needs to be woken up
            unblock_task(task);
        } else {
            // Task needs to be put back on the sleeping task list
            task->next = sleeping_task_list;
            sleeping_task_list = task;
        }
    }

    // Done, unlock the scheduler (and do any postponed task switches!)

    unlock_stuff();
}
</syntaxhighlight>

Finally, test the code to make sure tasks block and unblock correctly when they call "sleep()" or "nano_sleep()" or "nano_sleep_until()". Please note that currently there's a minor problem with the scheduler - if all tasks are blocked waiting for something (either paused or sleeping) there won't be any tasks that the scheduler can give CPU time to so the scheduler will give CPU time to a task that is supposed to be blocked (which is bad!). For now, just make sure that at least one task is running.


==Step 10: Idle Time==

For a real OS, often there's nothing for a CPU to do because all tasks are blocked waiting for something (e.g. waiting or the user to press a key). There are two very different ways to handle this situation - either you have a special "idle task" that never blocks so that (as far as the scheduler is concerned) there is always a task that can be given CPU time; or you add code to the scheduler to wait until a task is unblocked.

The first way is easier initially. The problem is that modern CPUs have power management involving multiple power saving states, where deeper power saving states save more power but take longer to return to full speed. The normal solution is to progressively move to deeper sleeps states so that if the CPU is only idle for a short time it's only in a light sleep state and can return to full speed quickly, and if the CPU was idle for a long time it'll be in a deep sleep state and save lots of power. What this means is that for modern power management you need to keep track of how long a CPU has been idle, and that's hard to do when you have a simple idle task that doesn't know when the CPU starts being idle or stops being idle.

For this, I'm going to try to cover both options.

===Idle Time With Idle Task===

To implement an idle task the first thing you'll need is an idle task(!). We can already create new kernel tasks, we just need code for the task itself. This code can be a loop that does nothing; but most CPU have an instruction you can use to wait for something (e.g. the "HLT" instruction on 80x87 that waits until an IRQ occurs) that saves a little power, and in that case it's probably a good idea to use that instruction in the idle task's loop.

The code might look like this:

<syntaxhighlight lang="c">
void kernel_idle_task(void) {
    for(;;) {
        HLT();
    }
}
</syntaxhighlight>

The next problem is that currently we're still using an awful cooperative round robin scheduler, which means that if the scheduler gives the idle task any CPU time it might hog all of the CPU time even after other tasks wake up. What we really need is a scheduler that isn't awful - e.g. a scheduler that understands task priorities, so that we can give the idle task the lowest possible priority and let all the other tasks (which will have higher priorities) preempt the idle task.

However; we don't really want to implement a complicated scheduler yet. Instead, we can use a few temporary hacks to simulate an extremely low priority without actually implementing full support for task priorities.

For this, you'll need a variable to keep track of which task is the idle task. Then you'd modify the "unblock_task()" code so that the idle task is always pre-empted, like this:

<syntaxhighlight lang="c">
void unblock_task(thread_control_block * task) {
    lock_scheduler();
    if( (first_ready_to_run_task == NULL) || (current_task_TCB == idle_task) ) {

        // Only one task was running before, or the idle task is currently running, so pre-empt

        switch_to_task(task);

    } else {
        // There's at least one task on the "ready to run" queue already, so don't pre-empt

        last_ready_to_run_task->next = task;
        last_ready_to_run_task = task;
    }
    unlock_scheduler();
}
</syntaxhighlight>

The other thing you'd want to do is modify "schedule()" so that it tries to avoid giving the idle task CPU time when other task's are "ready to run":

<syntaxhighlight lang="c">
void schedule(void) {
    if(postpone_task_switches_counter != 0) {
        task_switches_postponed_flag = 1;
        return;
    }
    if( first_ready_to_run_task != NULL) {
        thread_control_block * task = first_ready_to_run_task;
        first_ready_to_run_task = task->next;
        if(task == idle_task) {
            // Try to find an alternative to prevent the idle task getting CPU time
            if( first_ready_to_run_task != NULL) {
                // Idle task was selected but other task's are "ready to run"
                task = first_ready_to_run_task;
                idle_task->next = task->next;
                first_ready_to_run_task = idle_task;
            } else if( current_task_TCB->state == RUNNING) {
                // No other tasks ready to run, but the currently running task wasn't blocked and can keep running
                return;
            } else {
                // No other options - the idle task is the only task that can be given CPU time
            }
        }
        switch_to_task(task);
    }
}
</syntaxhighlight>

Note that this is a relatively crude hack; but that's fine for now - later we're going to replace the scheduler with something less awful anyway.


===Idle Time Without Idle Task===

In this case we need to add support for "no task running" in every piece of code that currently assumes "current_task_TCB" points to a valid task's information, so that we can set "current_task_TCB" to NULL when the CPU is idle.

The code for blocking a task should be fine (even though it does use "current_task_TCB") because the scheduler will never call this function when there's no task running; and the code for unblocking a task is fine.

The (optional) code for time accounting does need to be fixed though, and it'd be nice to keep track of how much time the CPU spend idle too, so that can be changed to something like this:

<syntaxhighlight lang="c">
uint64_t CPU_idle_time = 0;

void update_time_used(void) {
    current_count = read_counter();
    elapsed = last_count - current_count;
    last_count = current_count;
    if(current_task_TCB == NULL) {
        // CPU is idle
        CPU_idle_time += elapsed;
    } else {
        current_task_TCB->time_used += elapsed;
    }
}
</syntaxhighlight>

That only leaves the "schedule()" function itself, which could end up like this:

<syntaxhighlight lang="c">
void schedule(void) {
    thread_control_block * task;

    if(postpone_task_switches_counter != 0) {
        task_switches_postponed_flag = 1;
        return;
    }

    if( first_ready_to_run_task != NULL) {
        task = first_ready_to_run_task;
        first_ready_to_run_task = task->next;
        switch_to_task(task);
    } else if( current_task_TCB->state == RUNNING) {
        // Do nothing, currently running task can keep running
    } else {

        // Currently running task blocked and there's no other tasks

        task = current_task_TCB;                           // "task" is the task that we're borrowing while idle
        current_task_TCB = NULL;                           // Make sure other code knows we're not actually running the task we borrowed
        uint64_t idle_start_time = get_time_since_boot();  // For future power management support

        // Do nothing while waiting for a task to unblock and become "ready to run".  The only thing that is going to update 
        // first_ready_to_run_task is going to be from a timer IRQ (with a single processor anyway), but interrupts are disabled.
        // The timer must be allowed to fire, but do not allow any task changes to take place.  The task_switches_postponed_flag
        // must remain set to force the timer to return to this loop.


        do {
            STI();          // enable interrupts to allow the timer to fire
            HLT();          // halt and wait for the timer to fire
            CLI();          // disable interrupts again to see if there is something to do
        } while( (first_ready_to_run_task == NULL);

        // Stop borrowing the task

        current_task_TCB = task;                          

        // Switch to the task that unblocked (unless the task that unblocked happens to be the task we borrowed)

        task = first_ready_to_run_task;
        first_ready_to_run_task = task->next;
        if(task != current_task_TCB) {
            switch_to_task(task);
        }
    }
}
</syntaxhighlight>

==Step 11: Time Slices==

Most schedulers have some kind of limit on the amount of time that a task can consume CPU time; which can mean terminating a task (if the kernel assumes that the task locked up) or switching to a different task as part of a time sharing scheme.

Because we've been using an awful cooperative round robin scheduler anyway; the next logical step is to add time slices to convert it into an awful preemptive round robin scheduler.

For this you will need some kind of timer again, where the timer's IRQ handler just calls "schedule()" when a task has used all of the CPU time that the scheduler gave it. However, if you're using a timer set to a fixed frequency typically a relatively high frequency is used (e.g. 1000 Hz) to get better precision from it, and you don't want to do a task switch so often. In that case the easy solution is for the timer's IRQ handler to decrease a "time slice length remaining" variable and only do the task switch if the time has run out.

The other thing that might be worth mentioning is that (for most scheduling algorithms) when there's only one task that can be running there's point having the overhead of "schedule()" because it's only going to decide to let the currently running task keep running; and if you implemented idle time without a special idle task there might be no tasks running. For both of these situations there's no point having a time slice length. For that we can just use a special value (e.g. zero) to signify that there's no time slice length.

For the earlier example (using the PIT on 80x86 at a frequency of 1000 Hz) it might look like this:

<syntaxhighlight lang="c">
uint64_t time_slice_remaining = 0;

uint64_t time_since_boot = 0;
uint64_t time_between_ticks = 1000000;  // 1000 Hz = 1 ms between ticks = 1000000 nanoseconds between ticks

void PIT_IRQ_handler(void) {
    thread_control_block * next_task;
    thread_control_block * this_task;

    lock_stuff();
    time_since_boot += time_between_ticks;

    // Move everything from the sleeping task list into a temporary variable and make the sleeping task list empty

    next_task = sleeping_task_list;
    sleeping_task_list = NULL;

    // For each task, wake it up or put it back on the sleeping task list

    while(next_task != NULL) {
        this_task = next_task;
        next_task = this_task->next;

        if(this_task->sleep_expiry <= time_since_boot) {
            // Task needs to be woken up
            unblock_task(task);
        } else {
            // Task needs to be put back on the sleeping task list
            task->next = sleeping_task_list;
            sleeping_task_list = task;
        }
    }

    // Handle "end of time slice" preemption

    if(time_slice_remaining != 0) {
        // There is a time slice length
        if(time_slice_remaining <= time_between_ticks) {
            schedule();
        } else {
            time_slice_remaining -= time_between_ticks;
        }
    }

    // Done, unlock the scheduler (and do any postponed task switches!)

    unlock_stuff();
}
</syntaxhighlight>

The other change that's needed is that the "time_slice_remaining" variable needs to be set when a task switch happens. This can be done in the low level task switch code (which is "architecture specific assembly").

If you added a wrapper earlier (when updating locking in Step 8), and if you implemented idle time using an idle task, then it might look like this:

<syntaxhighlight lang="c">
#define TIME_SLICE_LENGTH  50000000   // 50000000 nanoseconds is 50 ms

switch_to_task_wrapper(thread_control_block *task) {
    if(postpone_task_switches_counter != 0) {
        task_switches_postponed_flag = 1;
        return;
    }

    if(task == idle_task) {
        time_slice_remaining = 0;
    } else {
        time_slice_remaining = TIME_SLICE_LENGTH;
    }
    switch_to_task(task);
}
</syntaxhighlight>

If you added a wrapper earlier and implemented idle time without an idle task, then it might look like this:

<syntaxhighlight lang="c">
#define TIME_SLICE_LENGTH  50000000   // 50000000 nanoseconds is 50 ms

switch_to_task_wrapper(thread_control_block *task) {
    if(postpone_task_switches_counter != 0) {
        task_switches_postponed_flag = 1;
        return;
    }

    if((current_task_TCB == NULL) {
        // Task unblocked and stopped us being idle, so only one task can be running
        time_slice_remaining = 0;
    } else if( (first_ready_to_run_task == NULL) && (current_task_TCB->state != RUNNING) ) {
        // Currently running task blocked and the task we're switching to is the only task left
        time_slice_remaining = 0;
    } else {
        // More than one task wants the CPU, so set a time slice length
        time_slice_remaining = TIME_SLICE_LENGTH;
    }
    switch_to_task(task);
}
</syntaxhighlight>

If you're not using a wrapper, it shouldn't be too hard to convert the changes above into assembly and add them to your low-level task switch code (for both cases; it's just some comparisons, branches and moves).


==Step 12: Task Termination==

Tasks are terminated for a variety of reasons (e.g. they crashed, called "exit()", received a kill signal, etc); and sooner or later you'll want to provide a low-level function that the kernel can use to terminate a task for any of these reasons.

Terminating a task is a little tricky - you want to do things like free any memory that the task was using for its kernel stack; but you can't do that while the  CPU is currently using the task's kernel stack. To work around that, you cheat - you don't terminate the task immediately and just set the task's state to "TERMINATED" and make the task block. A little later (when you're running some other task) there's no problem with freeing things like the terminated task's kernel stack because you know the CPU isn't using it.

So, how do you tell another task that it needs to clean up after a recently terminated task? Usually I write micro-kernels with message passing, and when a task is terminated I send a message to a "cleaner" to tell it the task was terminated, and then the "cleaner" can do the clean up work. We don't have any message passing (and don't have pipes or IPC of any kind yet) but what we do have is the ability to pause and unpause tasks (from Step 6), and as long as we're a little bit careful that's good enough.

The task termination might look like this:

<syntaxhighlight lang="c">
thread_control_block *terminated_task_list = NULL;

void terminate_task(void) {

    // Note: Can do any harmless stuff here (close files, free memory in user-space, ...) but there's none of that yet

    lock_stuff();

    // Put this task on the terminated task list

    lock_scheduler();
    current_task_TCB->next = terminated_task_list;
    terminated_task_list = current_task_TCB;
    unlock_scheduler();

    // Block this task (note: task switch will be postponed until scheduler lock is released)

    block_task(TERMINATED);

    // Make sure the cleaner task isn't paused

    unblock_task(cleaner_task);

    // Unlock the scheduler's lock

    unlock_stuff();
}
</syntaxhighlight>

Note that we're using the same "next" field in the data structure that the kernel uses to keep track of a task's information that the scheduler uses for its list of "ready to run" tasks (which is used for sleeping tasks too!). A task can't be ready to run and terminated at the same time, and a task can't be sleeping and terminated at the same time, so this is fine.

Of course this won't work without a "cleaner" task. The cleaner task might look a little bit like this:

<syntaxhighlight lang="c">
void cleaner_task(void) {
    thread_control_block *task;

    lock_stuff();

    while(terminated_task_list != NULL) {
        task = terminated_task_list;
        terminated_task_list = task->next;
        cleanup_terminated_task(task);
    }

    block_task(PAUSED);
    unlock_stuff();
}

void cleanup_terminated_task(thread_control_block * task) {
        kfree(task->kernel_stack_top - KERNEL_STACK_SIZE);
        kfree(task);
}
</syntaxhighlight>

==Step 13: Mutexes and Semaphores==

There are a lot of similarities between mutexes and semaphores - the both cause tasks to block when they can't acquire them and unblock a task when they can be acquired later, and they both are used for similar purpose (to restrict the number of tasks accessing a piece of data at the same time). The only real difference is the number of tasks that are allowed to access a piece of data at the same time - for mutexes this number is always one, and for semaphores the number is may be higher than 1.

In fact; if you really wanted to you could implement semaphores and then not bother implementing mutexes (and just use a semaphore with a count of 1 instead). However, this isn't really a great idea because you can optimise a mutex more(because you know it will only allow one task) and semaphores are rarely used (e.g. I have never needed one, ever). Even though it's not a great idea, this is exactly what we're going to do - there's so many similarities that it will avoid code duplication, and we don't really care about performance much anyway!

For multi-CPU systems, a semaphore (or mutex) begins with data structure to keep track of its state, and a spinlock to protect that state. We're not writing code for multi-CPU systems so we're not going to have a spinlock; and we already have (at least conceptually) a "big scheduler lock" we can use to protect the semaphore's data; so we're going to cheat to keep things simple.

The first thing we'll want is the semaphore's structure itself, which mostly just needs a few fields for counts plus a linked list for waiting tasks. For the linked list of waiting tasks I want to make sure tasks are taken from the front and added to the back to ensure that one task can be very unlucky and end up waiting forever while other tasks are lucky and keep acquiring the semaphore, which is slightly more tricky with singly linked lists (you need a "last task" field and a "first task" field). The semaphore's structure can be something like this:

<syntaxhighlight lang="c">
typedef struct {
    int max_count;
    int current_count;
    thread_control_block *first_waiting_task;
    thread_control_block *last_waiting_task;
} SEMAPHORE;
</syntaxhighlight>


Next we'll want some functions to create a semaphore or a mutex, like this:

<syntaxhighlight lang="c">
SEMAPHORE *create_semaphore(int max_count) {
    SEMAPHORE * semaphore;

    semaphore = kmalloc(sizeof(SEMAPHORE));
    if(semaphore != NULL) {
        semaphore->max_count = max_count;
        semaphore->current_count = 0;
        semaphore->first_waiting_task = NULL;
        semaphore->last_waiting_task = NULL;
    }
}

SEMAPHORE *create_mutex(void) {
    return create_semaphore(1);
}
</syntaxhighlight>

With that done, we want some functions to acquire a semaphore or a mutex:

<syntaxhighlight lang="c">
void acquire_semaphore(SEMAPHORE * semaphore) {
    lock_stuff();
    if(semaphore->current_count < semaphore->max_count) {
        // We can acquire now
        semaphore->current_count++;
    } else {
        // We have to wait
        current_task_TCB->next = NULL;
        if(semaphore->first_waiting_task == NULL) {
            semaphore->first_waiting_task = current_task_TCB;
        } else {
            semaphore->last_waiting_task->next = current_task_TCB;
        }
        semaphore->last_waiting_task = current_task_TCB;
        block_task(WAITING_FOR_LOCK);    // This task will be unblocked when it can acquire the semaphore
    }
    unlock_stuff();
}

void acquire_mutex(SEMAPHORE * semaphore) {
    acquire_semaphore();
}
</syntaxhighlight>

Lastly, we need to be able to release the semaphore or mutex:

<syntaxhighlight lang="c">
void release_semaphore(SEMAPHORE * semaphore) {
    lock_stuff();

    if(semaphore->first_waiting_task != NULL) {
        // We need to wake up the first task that was waiting for the semaphore
        // Note: "semaphore->current_count" remains the same (this task leaves and another task enters)

        thread_control_block *task = semaphore->first_waiting_task;
        semaphore->first_waiting_task = task->next;
        unblock_task(task);
    } else {
        // No tasks are waiting
        semaphore->current_count--;
    }
    unlock_stuff();
}

void release_mutex(SEMAPHORE * semaphore) {
    release_semaphore();
}
</syntaxhighlight>


Note that we're using the same "next" field in the data structure that the kernel uses to keep track of a task's information that the scheduler uses for its list of "ready to run" tasks (which is used for sleeping tasks and terminated tasks too!). A task can't be in multiple states at the same time, so this is fine.


==Step 14: Inter-process Communication (IPC)==

There's a lot of different types of IPC (remote procedure calls, signals, pipes, synchronous messages, asynchronous messages, ...) but most of them have a few similarities:

* they aren't just used for communication between different processes (e.g. can be used for communication between different threads in the same process, and for for a monolithic kernel, this can include communication between drivers in kernel space and tasks in user-space).
* they cause tasks to block (e.g. when waiting for data to arrive, and sometimes when waiting to send data) and unblock (when data arrives or data can be sent)
* often (especially for micro-kernels) it's responsible for the majority of task switches

I'm not going to provide examples of any of the different kinds of IPC - I want you to implement whatever you feel like having by yourself to get more experience. Note that there's multiple examples of "tasks blocked waiting on a list" in the previous steps of this tutorial.


==Step 15: "Schedule()" Version 3==

If you've got this far, you should have a fairly respectable scheduler with most of the features a scheduler in a modern OS needs. The main problem at the moment is that it's still an awful round robin scheduler. It's time to replace it with something better.

So, what is "better"?

It's impossible to answer that question without knowing what the OS will be used for and what kinds of tasks it will be running - an OS designed specifically for (e.g.) "hard real time" embedded systems (where the scheduler must provide strict guarantees) is very different to a (e.g.) an OS designed for scientific computer (where you expect a single task will hog all CPU time and the scheduler itself isn't very important).

However, for a general purpose OS that has to handle a wide range of different tasks with different requirements it becomes much easier to define "better" - for this case, "better" is something that is able to handle a wide range of different tasks with different requirements!

For this, you'd need to figure out what the different requirements are. Maybe you want to support "hard real-time" tasks (which is not recommended for beginners in general); and maybe you want to support "soft real-time" tasks (things like video decoders and sound systems that have to process data at a fixed rate to avoid dropping frames or causing audible glitches), and maybe you want to support "background tasks" that never get any CPU time unless there's nothing else the CPU can be used for.

Once you've decided what the different requirements are, you can define "scheduling policies" for each one. For an example; Linux uses four scheduling policies (SCHED_FIFO, SCHED_RR, SCHED_DEADLINE and SCHED_OTHER).

Then you'd need to figure out how the different policies relate to each other. The simplest way to do this is to assign them an order; where tasks using a lower policy only ever get CPU time when there are no tasks in a higher policy that want CPU time (and where "policy 0" might be sort of real-time policy and "policy 3" might be for background/idle tasks). In any case, you'll probably end up with a kind of "meta-scheduler" that decides which policy should be used. Once that's done, you need to choose a scheduling algorithm for each scheduling policy, and decide if each scheduling policy has (or doesn't have) task priorities within the policy, and decide on the rules that determine when one task (that's been created or unblocked) will preempt the currently running task.

For an full example (from a previous micro-kernel of mine), I used:

* Policy 0: Intended for low latency tasks (e.g. device driver IRQ handlers). Uses a "highest priority task that can run does run" scheduling algorithm, and tasks within this policy have a priority (0 to 255). These tasks will preempt all tasks in lower policies and all tasks with lower priority in the same policy.

* Policy 1: Intended for tasks with some latency requirements (e.g. user interfaces, GUIs, etc). Uses a "highest priority task that can run does run" scheduling algorithm, and tasks within this policy have a priority (0 to 255). These tasks will preempt all tasks in lower policies and all tasks with lower priority in the same policy.

* Policy 2: Intended for normal tasks. Uses a "variable frequency, fixed time slice" scheduling algorithm. Tasks within this policy have a priority (0 to 255) that determines how many (fixed length) time slices the task gets. Tasks will preempt all tasks in lower policies but will not preempt tasks in the same policy.

* Policy 3: Intended for background tasks. Uses a "variable time slice" scheduling algorithm, and tasks have a priority (0 to 255) that determines the length of the time slice they get. These tasks never preempt anything.


Once you've decided how your scheduler will work; you should only need to modify your "schedule()" and "unblock_task()" functions to make it happen.


==Adding User-Space Support==

Sooner or later you're probably going to want user-space tasks (e.g. processes and threads). This doesn't have much to do with the scheduler itself - it just means that while a kernel task is running it can switch between kernel and user-space where appropriate. It's important/useful to understand that when the CPU is running user-space code, something (system call, IRQ, exception, ...) will happen to cause the CPU to switch to kernel code, and then after kernel code is already running the kernel's code may decide to do a task switch. Because kernel code is always running before a task switch happens, kernel code will also be running after a task switch happens. In other words, task switches only ever happen between tasks running kernel code and other tasks running kernel code (and never involve a task running user-space code). This also means that the "switch_to_task()" code described in this tutorial (which is very fast because it doesn't save or load much) doesn't need to be changed when you add user-space support (until you start supporting things like FPU, MMX, SSE, AVX).

Typically; the main difficulty of adding user-space support is that you need to create a new virtual address space, have an executable loader (that might start the executable file directly, or might start a "user-space executable loader" that starts the executable file), create a new "first task for process" that includes a user-space stack, etc. I prefer to create a kernel task first and then do all this other work (to set up a process) while using the kernel task, because this works a lot better with task priorities (e.g. if a very high priority task creates a very low priority process, then most of the work needed to set up the low priority process is done by a low priority task, and the high priority task can continue without waiting for all that work).

Note: There are two common ways that processes can be started - spawning a new process from nothing, and "fork()". Forking is horribly inefficient and far more complicated (as you need to clone an existing process, typically including making all of the existing process' pages "copy on write", and then reversing all the work very soon after when the new child process calls "exec()"). For beginners I recommend not having "fork()" and just implementing a "spawn_process()" (and for people who are not beginners I recommend not having "fork()" because it's awful and has a risk of security problems - e.g. malicious code using "fork()" and then having its own implementation of "exec()" designed to inject malware into the new child process).

Once you have the ability to start a new process (including starting an initial task/thread for that process), adding a way to spawn a new task/thread for an existing process should be easy.


==Adding FPU/MMX/SSE/AVX Support (80x86 only)==

Originally (80386, single-CPU) Intel designed protected mode so that an OS can avoid saving/loading FPU state during task switches. The general idea was to keep track of an "FPU owner" and use a flag ("TS" in EFLAGS) to indicate when the currently running task isn't the FPU owner. If the CPU executes an instruction that uses FPU but the current task isn't the FPU owner, then the CPU raises an exception (because "TS" is set), and the exception handler saves the FPU state (belonging to a different task) and loads the FPU state for the currently running task. This can (in some cases) improve performance a lot - for example, if you have 100 tasks running where only one uses FPU, you'd never need to save or load FPU state. Intel continued this original idea when newer extensions (MMX, SSE, AVX) where added (although for AVX the implementation is significantly different).

However; when almost all tasks are using FPU/MMX/SSE/AVX state, it makes performance worse (due to the extra cost of an inevitable exception); and it doesn't work well for multi-CPU (where the currently running task's FPU state may still be in a completely different CPU).

The simplest solution is to always save and load FPU/MMX/SSE/AVX state during all task switches (and that would be a recommended starting point). There are also multiple more complex (and more efficient) ways of doing it; including always saving the previous task's FPU/MMX/SSE/AVX state during task switches if you know it was used but still postponing the work of load the next task's state until the task actually does need it; and including keeping track of "never used, sometimes used, always used" and pre-loading the FPU/MMX/SSE/AVX state when you know a task always uses it (or uses it often enough).

[[Category:Tutorials]]
[[Category:Multitasking]]
````

<a id="w004286"></a>

## W004286 — User:Gravaera/RISC-V Overview

Bron-ID: W004286
Namespace: User (2)
Revisie: 24098 | Datum: 2019-11-11T04:45:20Z
Bron: https://wiki.osdev.org/User:Gravaera/RISC-V_Overview
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FRISC-V_Overview&oldid=24098

````text
Riscv notes:

CSR=Control and Status Reg
WIRI: Reserved Write Ignored, Reads Ignore Values:
	Hardware ignores writes to this field if RW; if RO, illegal exception generated.
WPRI: Reserved Writes Preserve Values, Reads Ignore Values
	Hardware will commit write changes to this field, but software should not change the value
	in this field in the first place: preserve what you read out.
WLRL: Write/Read Only Legal Values: Be careful to only write legal, and shouldn't
	assume that reads after illegal write will return legal values.
WARL: Write Any, Read Legal Values: You can write illegal values and when read back, only
	legal values will be returned.

The Hart enters MMode on #RESET. 
Privilege level: 0=User, 1=Supervisor, 2=RSVD, 3=Machine; Machine mode must always exist.
	Implementations may also implement a "Debug" and "OEM Test" mode. No privlevel value
	is assigned by the spec. Since increasing privilege is encoded as increasingly higher
	numerical values, this is seamlessly doable. Machine mode should be trapped if it tries to
	access the D-CSRs [sec 2.1].

All privileged intrs are prefixed by the SYSTEM major opcode. Privileged instrs are 2 categories:
	(1) RMW ops to CSR regs, (2) All other privileged instrs.

	CSRs:
CSRs at level X are accessible to all levels higher than it (greater than X) as well.
Shadow registers are RW versions of a lower priv reg which can be written to by a higher priv lvl.
	They enable higher priv determine what environment lower priv sees. Multiple higher privlvls
	may use the same shadow reg so shadow regs don't need to be provided to every privlvl for
	any given CSR.

	* Attempting to access a non-existent CSR generates an illegal instr exception.
	* Accessing a CSR from a lower privlevel generates an illegal instr exception.

	* There are 4096 CSRs altogether (12 bit encoding space).
	* 12 bit encoding space broken down into:
		CSR_IDX[11:10]: RW perms. (0x3=RO, 0x0-0x2=RW).
		CSR_IDX[9:8]: Lowest priv level which can access the CSR. Because increasing
			privilege is encoded using increasing value integers, this means that
			any privlevel CSR_IDX[9:8] or higher can access the CSR.



Multiple wordsizes: 32, 64, 128:
* When CPU supports multiple wordsizes and is operating in wordsize lower than greatest
  mode supported, all operations must ignore source reg high bits sign extend result
  regs up to highest wordsize high bits.
* Higher priv modes can query the current wordsize(XLEN) of a lower mode by reading
  xSTATUS.[yXLEN], where X is current privlvl and Y is privlvl to be queried about.

	* See MISA CSR for how to detect MMode wordsize.
	* See MSTATUS CSR for how to control xXLEN of lower priv modes.
	  ^ This means that MMode can support mixed wordsize guests (32, 64, 128).
	* See SSTATUS CSR for how to control UXLEN.
	  ^ This means that SMode can support mixed wordsize binaries (32, 64, 128).
	* In all cases, the OEM is free to HARDWIRE values for xSTATUS.[xXL].

Machine Mode:

MISA CSR: Always readable without fault but returns 0 if not implemented.
	XLEN:XLEN-1: MXL | XLEN-3:26: RSVD. | 25:0: Extensions bits.
		MXL: Sets CPU ISA/reg wordsize: 0=unimplemented; 1=32; 2=64; 3=128
			RW if multiple XLENs supported. The RISC-V spec gives advice on
			how to detect your current MXL using this reg without knowing
			beforehand.
		Extensions: WARL: Each bit from 0-25 represents a letter from A-Z.
			G-bit: Accesses separate namespace of new letters gA-gZ.
				G also means that ADFIM are implicitly supported.
			See riscv-misa-extension-bits.png.
			I: must be set; represents the base ISA.
			X: Non-standard extensions present.
			N: User-level traps.
MVENDOR CSR: Always readable w/o fault but returns 0 if unimplemented or "non-commercial" vendor.
MARCHID CSR: Always readable w/o fault but returns 0 if unimplemented.
	Bit XLEN (MSb): 0=open source vendor; 1=proprietary vendor.
MIMPID CSR: Always readable w/o fault but returns 0 if unimplemented.
MHARTID CSR: Always readable w/o fault.
	* Gives the HART ID. Not the Core ID. Hart IDs need not be contiguous.
	* At least one hard must have an ID of 0. Explicitly stated that this is BSP.
MSTATUS CSR: Virtualization, xIntEn, xPrevIntEn, xPreviousPriv, xXLEN
* Restricted copies of this reg appear in S and U Mode.
* Bits are intended to be toggled with atomic instructions.
	* See Trap Architecture section for how traps and xRET modify the xIE, xPIE and xPP bits.
	* Setting xXLEN sets the XLEN (wordsize) of the lower privlvl in question.
	  ^ For systems whose highest wordsize is 32, MSTATUS.[xXL] does not exist.
	  ^ In all cases, the OEM may HARDWIRE xSTATUS.[xXL] to some value.
	Bits:
	* Bit XLEN-1: SD bit. See FPU and other feature save and restore section.
	* Bit 16:13: FS and XS bits: Also see FPU and other feature save/restore section.
	* Bit 17: MPRV: See "Cute features"
	* Bit 19: MXR: See "Cute features"
	* Bit 20, 21, 22: See "Virtualization" section.
MTVEC CSR: Must always be implemented but may contain hardwired RO value.
	* Is WARL so you can test to see if it is hardwired.

	Vector base alignment:
	* Vector addr can be anywhere as long as it is 4B aligned for direct mode.
	* For Vectored mode, aligmment is 4*XLEN.

	Bits:
	* 1:0: MODE:	0=Direct: all exceptions set PC to this.[BASE]
			1=Vectored: Async exceptions set PC to (this.BASE + 4*CAUSE)
MIDELEG/MEDELEG CSR: By default *ALL* traps route to MMode, but can be delegated to SMode.
* If only MMode exists, or if only M+UMode exist without UMode-traps (N), then M?DELEG should NOT
  EXIST. I.e, accessing will trigger a fault.
* Delegation from MMode is only to SMode. SMode can then choose to delegate to UMode if UMode traps
  are supported (MISA.Extensions[N]).
  ^ That said, if SMode doesn't exist (only M+U), then delegation goes to UMode.

	MEDELEG:
	* Bit positions in MEDELEG correspond to exception cause values as used by MCAUSE.
	* MEDELEG[11] and SEDELEG[11:9] are hardwired to 0; CANNOT be delegated.
	MIDELEG:
	* Bit positions in MIDELEG correspond those of the MIP reg.
MIP CSR: Machine Interrupt Pending
^ Has bits: MTIP/STIP/UTIP; MSIP/SSIP/USIP; MEIP/SEIP/UEIP.
* See Interrupt architecture section for explanation of how these bits work.
MIE CSR: Machine Interrupt Enable:
* Has bits: MTIE/STIE/UTIE; MSIE/SSIE/USIE; MEIE/SEIE/UEIE.
* See interrupt architecture section for explanation of how these bits work.

Supervisor Mode: Detected by MISA.Extensions[S].

UserMode: Detected by MISA.Extensions[U].

FLOATING POINT UNIT AND OTHER FEATURE SAVE/RESTORE:
	* Check FPU usage status when save/restoring using xSTATUS.[FS].
	* Check extension feature usage status when save/restoring using xSTATUS.[XS].

	When a feature is disabled, any attempt to access its regs/state will trigger an
	unspecified exception.

	FS and XS field format: Both are 2 bits.
		0=off: unit/features is off;
		1=initial: feature is on and initialized, state unmodified since initialization;
		2=clean: none dirty some state clean;
		3=dirty: FPU or some features dirty.
	SD bit: This is a superbit which encompasses the state of both FS and XS.


CUTE FEATURES:
	Modified PRiVilege: Enable using xSTATUS.[MPRV]:
		Causes CPU to perform LOADS and STORES in mode X as if they were performed from
		mode xSTATUS.[xPP]. Basically set xPP and then you get to simulate data accesses
		(NOT INSTR FETCH+DECODE) as if executing in mode xPP.

	Make eXecutable Readable: Enable using xSTATUS.[MXR]:
		RISCV CPUs will fault on data load from vmem pages without READ perms set
		by default. This feature makes it easier for a trap and emulate handler to
		emulate by allowing pages marked as EXEC only to also count as if they had READ
		set.
````

<a id="w004287"></a>

## W004287 — User:Gravaera/RISC-V Trap Architecture

Bron-ID: W004287
Namespace: User (2)
Revisie: 22569 | Datum: 2018-08-16T04:49:11Z
Bron: https://wiki.osdev.org/User:Gravaera/RISC-V_Trap_Architecture
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FRISC-V_Trap_Architecture&oldid=22569

````text
Trap architecture:
Trap is the umbrella term for IRQs, exceptions etc. Vertical traps are those taken from lower
to higher. Horizontal traps are those taken from same to same.
By default, all traps at any privlvl are taken directly to MACHINE mode.
* xRET (MRET, SRET, URET) are used to return from mode X respectively.
* RISC-V supports the idea of userlvl traps -- traps taken from highpriv to userlvl.
	* When a trap occurs, CPU stores Previous Privlvl in xPP bit (xSTATUS.[xPP]).
	  ^ You can also read yPP from mode X where Y is a lower privlvl.
	* When a trap occurs from priv Y to X:
	  * xPP <= y; xPIE <= xIE. xIE <= 0.
	* When executing xRET (returning from from X to Y):
	  * xIE <= xPIE; [MODECHANGE]; xPIE = 1 ALWAYS; xPP = U (or M if U unsupp) ALWAYS;
	* OS is expected to manually save xSTATUS.
	* MISA.Extensions[N]: Indicates support for user-level traps.

	Trap sequence:
	On trap taken to mode X:
	* xCause <= exception cause;
	* xEPC <= PC that triggered the trap.
	* Then the standard trap sequence goes with xIE, xPIE, xPP [out of order].
	* If the trap was delegated, the MCAUSE, MEPC, MIE, MPIE, MPP regs are not changed.

* Delegation is based on the prilvl the trap was taken from. So a trap taken from MMode cannot be
  delegated to SMode. It must have been taken FROM UMode or SMode to be delegated BY MMode TO SMode.
  ^ See MEDELEG and MIDELEG CSRs for trap delegation information.

Interrupt architecture:

* RISC-V specifies the following events as "interrupts":
  * Timer Interrupt (xTI?): mip.[MTIP]: cleared by writing to timer compare reg.
    ^ MMode can only trigger a MTimer event by writing to the timer CSRs.
	* MMode can trigger a STimer or UTimer event by writing to mip[STIP] or mip.[UTIP].
	  ^ SMode and UMode must clear the STIP/UTIP bit by making calls to AEE/SEE through ABI/SBI.
  ^ Software Interrupt (mip.[xSI?]):
    ^ These are also the vector for IPIs.
  ^ External Interrupt (mip.[xEI?])
	xSTATUS.[xIE]: 0=IRQs disabled; 1=IRQs enabled for respective mode.
	* While in mode X, IRQs in mode X-1 and lower are always disabled.
	* While in mode X, IRQs in mode X+1 and higher ALWAYS ENabled. Highpriv IRQs always taken.

IRQ PRIORITY: Harts prioritize these three types of interrupts as follows:
	External IRQ > Software Interrupt > Timer Interrupt > Synchronous traps (exceptions?).
	^ It is a bit odd that they prioritize Softint over timer, but that may be a symptom of IPIs
	  being delivered as Software Ints.

An IRQ will be taken if its bit is set in both mie.[CAUSE] and if mip.[CAUSE] gets set.

	IPIs:
	* IPIs are simulated as software interrupts.
	* The source hart writes to the target hart's mip.[MSIP] CSR, and this delivers the IPI.
	  ^ SMode must deliver IPIs by executing calls to the SEE.
	  ^ SMode CLEARS pending IPIs/SoftInts by clearing sip.[SSIP].
	  ^ UMode delivers IPIs (if it ever has to) by calling the AEE.
	* A hart can IPI itself by writing to its own mip.[MSIP].
	* The mechanism for writing to a foreign hart's CSR space is IMPLEMENTATION DEFINED.
	  ^ Might be via MMIO or some other mechanism.

	External IRQs:
	* mip.[MEIP] is set by an external IRQ signal. Cleared by writing to the platform's PIC.
	* mip.[S/U-EIP] work the same way.

	Injecting external IRQs into SMode/UMode:
	* See Virtualization section.

	Non-Maskable IRQ:
	* There is no Interrupt Pending bit in mip for an NMI-EIP because a pending NMI can be
	  recognized implicitly by the very fact that you are executing the NMI handler.
````

<a id="w004288"></a>

## W004288 — User:Gravaera/RISC-V Virtualization

Bron-ID: W004288
Namespace: User (2)
Revisie: 22570 | Datum: 2018-08-16T04:49:56Z
Bron: https://wiki.osdev.org/User:Gravaera/RISC-V_Virtualization
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FRISC-V_Virtualization&oldid=22570

````text
Virtualization:
ENABLE BITS FOR VIRTUALIZATION:
	MMU Management trapping (enable with MSTATUS.TVM):
	* Hardwired to 0 (disabled) if SMode unsupported (MISA.Extensions[S]=0).
	* Traps all accesses to SATP CSR.
	* Traps all executions of SFENCE.VMA instr by SMode.

	WFI trapping (enable with MSTATUS.TW):
	* Hardwired to 0 (disabled) if SMode unsupported.
	* Traps execution of WFI by SMode (not by UMode).
	* After WFI is executed, an IMPLEMENTATION DEFINED countdown is done and if CPU is
	  still halted, trap is taken to MMode.

	SRET instr trapping (enable with MSTATUS.TSR):
	* This is provided to enable simulation of something called "Augmented Virtualization".

	Injecting external IRQs into SMode/UMode from MMode:
	* See mip.[SEIP/UEIP] bit.
	  ^ This bit has unusual behaviour in that reading it in certain ways will return a value
	    OR'd with a value from the chipset's PIC.
	  ^ Writing this bit with a CSRR[SC] RMW instr will always ONLY write the mip.[xEIP] bit tho.
	  ^ mip.[UEIP] is only accessible if misa.Ext[N]==1 (UMode interrupts). 
	* mip.[UEIP] works the same way as mip.[SEIP].
````

<a id="w004289"></a>

## W004289 — User:Gravaera/ARMv8 Overview

Bron-ID: W004289
Namespace: User (2)
Revisie: 22572 | Datum: 2018-08-16T12:38:39Z
Bron: https://wiki.osdev.org/User:Gravaera/ARMv8_Overview
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FARMv8_Overview&oldid=22572

````text
This page seeks to provide a reference which will be a refresher for an individual who has previously already read the ARMv8 manuals. It neither claims nor seeks to be an exhaustive overview of ARMv8.

= ARMv8 abstract machine =
An ARMv8 implementation's abstract virtual machine consists of:
* Execution state:
* ISA mode.
* System Registers.
* Debug Architecture v8.

== Execution state ==
Execution state encompasses the supported register set, the supported ISAs, portions of the exception model, the VMSA state and the programmers' model. There are two broad execution state choices:

ARMv8 supports transitioning between execution states (AArch32 and AArch64) but only by transitioning from one Exception Level to another. This support enables an AArch64 supervisor/hypervisor to support a subordinate software layer running in a different exception level, in a different execution state. ARM calls such transitions between execution states "**Interprocessing**".

AArch64 execution state:
* 31 64-bit GP regs, including X30 which takes on the role of the traditional LR register (link register); a 64-bit PC, SP and ELR (Exception Link Register).
* 32 128-bit SIMD registers for vector and scalar FP.
* Only one ISA: The A64 ISA.
* ARMv8 Exception Model with up to 4 exception levels: EL0-EL3 that form the execution privilege lobster hierarchy.
* VMSAv8 implementation which provides up to 64 bit virtual addressing.
* PSTATE elements which contain PE state, and A64 instructions which manipulate PSTATE.
* Intuitive naming scheme for system registers which a suffix that indicates the lowest EL at which the register is accessible.

AArch32 execution state:
* 12 32-bit GP regs, 32-bit PC, LR and SP. In AArch32 the LR is used both as ELR and the procedure link register.
* Banked register state which is accessible in different PE modes.
* A single ELR_Hyp register for returns from Hyp mode.
* 32 64-bit registers for SIMD vector and scalar FP.
* Support for 2 ISAs (A32 and T32).
* ARMv7-a exception model with enhancements to map it onto the ARMv8 Exception Model (based on Exception Levels).
* VMSAv7 which supports up to 32 bits of virtual addressing.
* PSTATE elements. A32 and T32 ISAs have instructions which can manipulate PSTATE.

== ISA Mode ==

* A64 uses 64bit registers, but a fixed 32-bit instruction encoding.
* A32 uses 32-bit registers and a fixed 32-bit instruction encoding.
* T32 uses a variable length instruction encoding which can either be 16 or 32-bits long.

== Profiles (Sec A1.2) ==
* A-profile supports the A32, T32 and A64 instruction sets.
* R-profile supports only the A32 and T32 ISAs.
* M-profile supports only the a variant of the T32 ISA.

By implication, R and M profile PEs do not support execution in AArch64 mode and M profile PEs only support execution in Thumb mode.


= Debug Architecture =
A1.2.1 "The ARM Debug architecture is fully integrated with the architecture, and does not have a separate version number."
A1.3.4: "All ARMv8 implementations support both models [Self-hosted Debug and External debug]."
I.e, it appears that ARMv8 Debug is meant to be supported by all implementations.
````

<a id="w004290"></a>

## W004290 — Brokenthorn's Known Bugs

Bron-ID: W004290
Namespace: Main (0)
Revisie: 30147 | Datum: 2026-03-05T23:39:42Z
Bron: https://wiki.osdev.org/Brokenthorn%27s_Known_Bugs
Revisie-URL: https://wiki.osdev.org/index.php?title=Brokenthorn%27s_Known_Bugs&oldid=30147

````text
Brokenthorn's tutorials are some of the best OSDev tutorials out there. They teach good coding practices, and go very in depth. But, like all tutorials, there are a few bugs in it. Here are the most prevalent:

==Wrong description of CR0.PE==
In OSDev8, CR0.PE (bit 0) is incorrectly described as setting the processor to 32-bit mode. Setting bit 0 of CR0 does not put the processor in 32-bit mode. Performing a far jump to a 32-bit code segment puts the processor in 32-bit mode. The far jump itself will still be executed in 16-bit mode.

==iret using inline ASM==
In OSDev15, the provided ISR handler uses inline asm to iretd from a C function:
<syntaxhighlight lang="C">
void int_handler_5 () {
 
	_asm add esp, 12
	_asm pushad
 
	// do whatever...
 
	_asm popad
	_asm iretd
}
</syntaxhighlight>

This makes assumptions about the calling conventions used, and isn't a good idea in general.  Better options may be found [[Interrupt_Service_Routines|here]].

==Missing semicolon==
In OSDevPE, there is a missing semicolon in the IMAGE_THUNK_DATA structure

==Issues with paging code==
In OSDev18, the paging code has some issues. It directly accesses physical memory without mapping it. You can use it as a base, but implement recursive paging.

[[Category:FAQ]]
[[Category:Troubleshooting]]
````

<a id="w004291"></a>

## W004291 — HiFive-1 Bare Bones

Bron-ID: W004291
Namespace: Main (0)
Revisie: 30458 | Datum: 2026-04-06T15:40:25Z
Bron: https://wiki.osdev.org/HiFive-1_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=HiFive-1_Bare_Bones&oldid=30458

````text
{{BeginnersWarning}}
{{FirstPerson}}
{{TutorialExplain}}
{{Rating|2}}

If you are looking to get a [[RISC-V]] board, the HiFive1 is a relatively easy (albeit slightly expensive at US$69) way to get a RISC-V board. This Bare Bones tutorial is an attempt at bringing up the HiFive-1 board and getting ready to start some OS development as you would on most other platforms.

== Prerequisites ==

You'll need a few pieces of software:

* [http://www.git-scm.com/ Git] for acquiring the SDK
* build-essential, bison, flex for building the SDK
* minicom (or screen) for checking on the serial of the HiFive-1 board

== Acquiring the SDK ==

First, in order to compile for the HiFive-1 Board, you need to download and build the SDK:

<syntaxhighlight lang="shell-session">
$ git clone --recursive https://github.com/sifive/freedom-e-sdk.git
$ cd freedom-e-sdk
$ make tools
</syntaxhighlight>

The SDK builds a bunch of stuff we do not really need (including a [[C Library|Newlib]] port that lets the developer write basic C programs and run it bare-metal). Unfortunately, I did not yet invest the time to turn off those unneeded features, so for now we will need to wait for a full build which does take some time. Grab a tea or coffee.

== The Bare Bones ==

This Bare Bones tutorial will setup the 16Mhz Crystal for UART timing, then initialize the UART and finally print a nice message to the serial port.

== The kernel source code ==

First, create a file named main.c which will serve as both the entry point from the firmware and the starting point for the kernel you will be writing.

<syntaxhighlight lang="c">
#include <stdint.h>
#include <stddef.h>

/* GPIO */
#define GPIO_CTRL_ADDR 0x10012000UL
#define GPIO_IOF_EN    0x38
#define GPIO_IOF_SEL   0x3C
#define IOF0_UART0_MASK         0x00030000UL

/* UART */
#define UART0_CTRL_ADDR 0x10013000UL
#define UART_REG_TXFIFO         0x00
#define UART_REG_RXFIFO         0x04
#define UART_REG_TXCTRL         0x08
#define UART_REG_RXCTRL         0x0c
#define UART_REG_IE             0x10
#define UART_REG_IP             0x14
#define UART_REG_DIV            0x18
#define UART_TXEN               0x1

/* PRCI */
#define PRCI_CTRL_ADDR 0x10008000UL
#define PRCI_HFROSCCFG (0x0000)
#define PRCI_PLLCFG (0x0008)
#define ROSC_EN(x) (((x) & 0x1) << 30)
#define PLL_REFSEL(x) (((x) & 0x1) << 17)
#define PLL_BYPASS(x) (((x) & 0x1) << 18)
#define PLL_SEL(x) (((x) & 0x1) << 16)


/* This function will read a 32-bit value from an MMIO register */
static inline uint32_t
mmio_read_u32(unsigned long reg, unsigned int offset)
{
	return (*(volatile uint32_t *) ((reg) + (offset)));
}

/* This function will write a byte to an MMIO register */
static inline void
mmio_write_u8(unsigned long reg, unsigned int offset, uint8_t val)
{
	(*(volatile uint32_t *) ((reg) + (offset))) = val;
}

/*This function will write a 32-bit value to an MMIO register */
static inline void
mmio_write_u32(unsigned long reg, unsigned int offset, uint32_t val)
{
	(*(volatile uint32_t *) ((reg) + (offset))) = val;
}

/* Initialize the UART */
static void
uart_init()
{
	/* These two writes enable the UART via the GPIO */
	mmio_write_u32(GPIO_CTRL_ADDR,
		GPIO_IOF_SEL,
		mmio_read_u32(GPIO_CTRL_ADDR, GPIO_IOF_SEL)
			& ~IOF0_UART0_MASK);

	mmio_write_u32(GPIO_CTRL_ADDR,
		GPIO_IOF_EN,
		mmio_read_u32(GPIO_CTRL_ADDR, GPIO_IOF_EN)
			| IOF0_UART0_MASK);

	/*
	 * Assuming a 16Mhz Crystal (which is Y1 on the HiFive1), the divisor
	 * for a 115200 baud rate is 138
	 */
	mmio_write_u32(UART0_CTRL_ADDR, UART_REG_DIV, 138);
	mmio_write_u32(UART0_CTRL_ADDR,
		UART_REG_TXCTRL,
		mmio_read_u32(UART0_CTRL_ADDR, UART_REG_TXCTRL)
			| UART_TXEN);

	/* busy loop until the line is asserted... */
	volatile int i = 0;
	while(i++ < 1000000);
}

/* Transmit a single byte over the UART */
static void
__uart_write(uint8_t byte)
{
	/* wait for the UART to become ready */
	while (mmio_read_u32(UART0_CTRL_ADDR, UART_REG_TXFIFO) & 0x80000000)
		;
	
	/* write to the UART transmit FIFO */
	mmio_write_u8(UART0_CTRL_ADDR, UART_REG_TXFIFO, byte);
}


/* Transmit a buffer of length "len" over the UART */
static void
uart_write(uint8_t *buf, size_t len)
{
	int i;
	for (i = 0; i < len; i ++) {
		__uart_write(buf[i]);
		/* If an LF was written, also write a CR */
		if (buf[i] == '\n') {
			__uart_write('\r');
		}
	}
}

/* People, the simplest ever strlen function */
static size_t
strlen(char *str)
{
    int len = 0;
    int i;
    
    for (i = 0; str[i] != 0; i ++)
        len ++;

    return len;
}

/* Write a null-terminated string to the UART, transmitting it */
static void
uart_write_string(uint8_t *buf) 
{
	uart_write(buf, strlen((char *) buf));
}

/* Initialize the clock source for the UART, in this case the 16MHz crystal */
static void
prci_init(void)
{
	/* Make sure the HFROSC is on */
	mmio_write_u32(PRCI_CTRL_ADDR, PRCI_HFROSCCFG,
			mmio_read_u32(PRCI_CTRL_ADDR, PRCI_HFROSCCFG)
			 | ROSC_EN(1));

	/* Run off 16 MHz Crystal for accuracy */
	mmio_write_u32(PRCI_CTRL_ADDR, PRCI_PLLCFG,
			mmio_read_u32(PRCI_CTRL_ADDR, PRCI_PLLCFG)
			 | (PLL_REFSEL(1) | PLL_BYPASS(1)));
	mmio_write_u32(PRCI_CTRL_ADDR, PRCI_PLLCFG,
			mmio_read_u32(PRCI_CTRL_ADDR, PRCI_PLLCFG)
			 | (PLL_SEL(1)));

	/* Turn off HFROSC to save power */
	mmio_write_u32(PRCI_CTRL_ADDR, PRCI_HFROSCCFG,
			mmio_read_u32(PRCI_CTRL_ADDR, PRCI_HFROSCCFG)
			 & ~(ROSC_EN(1)));
}

/* The entry point */
void
main(void)
{
	prci_init();
	uart_init();

	uart_write_string("Hello, world!\nThis is myOS on the HiFive-1 Board!\n");

	/* For now, just halt */
	for (;;);
}

/* The _actual_ entry point, this is then fixated to 0x20400000 via the linker script */
 __attribute__((section(".init")))
void
_start(void)
{
	main();
}
</syntaxhighlight>

Next, we need to do some linker script magic to make sure that our code is positioned where it should be. It's worth noting that the firmware seems to perform an unconditional jump to 0x204000000 and expects the application to continue from there. Thus, in the above code we place the _start() function into a special section (aptly named, ".init") which we will move to the correct address via the linker script:

<syntaxhighlight lang="c">
OUTPUT_ARCH("riscv")

ENTRY(_start)

MEMORY
{
  flash (rxai!w) : ORIGIN = 0x20400000, LENGTH = 512M
  ram (wxa!ri) : ORIGIN = 0x80000000, LENGTH = 16K
}

PHDRS
{
  flash PT_LOAD;
  ram_init PT_LOAD;
  ram PT_NULL;
}

SECTIONS
{
  __stack_size = DEFINED(__stack_size) ? __stack_size : 2K;

  .init           :
  {
    KEEP (*(SORT_NONE(.init)))
  } >flash AT>flash :flash

  .text           :
  {
    *(.text .text.*)
  } >flash AT>flash :flash

  .fini           :
  {
    KEEP (*(SORT_NONE(.fini)))
  } >flash AT>flash :flash

  .rodata         :
  {
    *(.rdata)
    *(.rodata .rodata.*)
  } >flash AT>flash :flash

  . = ALIGN(4);

  .data          :
  {
    *(.data .data.*)
    . = ALIGN(8);
    PROVIDE( __global_pointer$ = . + 0x800 );
    *(.sdata .sdata.*)
    *(.gnu.linkonce.s.*)
    . = ALIGN(8);
    *(.srodata.cst16)
    *(.srodata.cst8)
    *(.srodata.cst4)
    *(.srodata.cst2)
    *(.srodata .srodata.*)
  } >ram AT>flash :ram_init

  . = ALIGN(4);
  PROVIDE( _edata = . );
  PROVIDE( edata = . );

  PROVIDE( _fbss = . );
  PROVIDE( __bss_start = . );
  .bss            :
  {
    *(.bss .bss.*)
    *(COMMON)
    . = ALIGN(4);
  } >ram AT>ram :ram

  . = ALIGN(8);
  PROVIDE( _end = . );
  PROVIDE( end = . );

  .stack ORIGIN(ram) + LENGTH(ram) - __stack_size :
  {
    PROVIDE( _heap_end = . );
    . = __stack_size;
    PROVIDE( _sp = . );
  } >ram AT>ram :ram
}
</syntaxhighlight>

Now this linker script also provides a few extra symbols that can come handy when developing your OS. In particular, it gives your OS a stack location!

Now that we have the kernel source code and we have a linker script, what remains is building our image. First, however, let's setup a few environment variables:

<syntaxhighlight lang="shell-session">
$ export SDK=/the/location/where/your/freedom/sdk/lies
$ export SDK_PREFIX=$(SDK)/work/build/riscv-gnu-toolchain/riscv64-unknown-elf/prefix/bin
$ export CROSS=$(SDK_PREFIX)/riscv64-unknown-elf-
$ export OPENOCD=$(SDK)/work/build/openocd/prefix/bin/openocd
</syntaxhighlight>

Modify the $SDK variable to where your SDK is cloned to.

Finally, we can now proceed to build our image, simply:

<syntaxhighlight lang="shell-session">
$ ${CROSS}gcc main.c \
  -g \
  -march=rv32imac \
  -mabi=ilp32 \
  -mcmodel=medany \
  -o main.img \
  -T linker.lds \
  -nostartfiles
</syntaxhighlight>

Before you execute that command, you might be wondering what those options are.

* "-g" turns on debug symbols and additional debugging information. This can be useful for your kernel and (I believe) you can also the on-chip debugger to jump to functions, inspect variables, as you would in GDB.
* "-march=rv32imac" specifies the available instructions for the board. The HiFive-1 is a 32-bit RISC-V board, with the IMAC extensions.
** "I" refers to Integer Base Instructions
** "M" refers to Integer Multiplication and Division Instructions
** "A" refers to atomic instructions
** "C" refers to compressed instructions (think Thumb mode on ARM)
* "-mabi=ilp32" specifies the ABI of the platform. "ilp32" means that Integers, Longs, Pointers are 32-bit wide.
* "-mcmodel=medany" tells GCC to compile for a Medium/Anywhere code model.

The rest of the arguments should be familiar from other programming activities.

At this point, you should have a main.img in your directory and that is kernel that we have just compiled!

== Testing on real hardware ==

To test on real hardware, you will need to first plug your HiFive-1 board to your computer of choice (preferably the one with the freshly compiled main.img from the previous steps. :-) ).

Before you can flash the kernel to the board, however, you will need an OpenOCD specification for the said board. You can find one for the HiFive-1 board [https://github.com/sifive/freedom-e-sdk/blob/e582766d596dcaa303634c121638b8f614ffd4c7/bsp/sifive-hifive1/openocd.cfg here]. (I am unable to directly include it, because its license is not CC0-compatible, thus you'll need to grab it yourself).

Once you have this, the process of flashing an image to the board is relatively quick:

<syntaxhighlight lang="shell-session">
$ ${OPENOCD} \
  -f openocd.cfg \
  -c "flash protect 0 64 last off; \
      program main.img verify;\
      resume 0x20400000;\
      exit"
</syntaxhighlight>

In this command, we first turn off the board's flash protection, program the board with the kernel and verify that the hashes match, and resume from the starting point. Finally, we exit the on-chip debugger.

If all went well, you are done and the UART (which you can observe via minicom) will now output the following:

<pre>
Hello, world!
This is myOS on the HiFive-1 Board!
</pre>


[[Category:RISC-V]]
[[Category:Bare_bones_tutorials]]
````

<a id="w004292"></a>

## W004292 — User:Carver

Bron-ID: W004292
Namespace: User (2)
Revisie: 22994 | Datum: 2018-09-27T15:53:47Z
Bron: https://wiki.osdev.org/User:Carver
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACarver&oldid=22994

````text
== Projects ==
=== OSDev ===
* [https://dorper.me/software/xtos XtOS]
* [https://dorper.me/software/doops DOOPS]

=== Websites ===
* [https://dorper.me Dorper]
* [https://yt.dorper.me LiteTube]
* [https://wiki.d0.cx DorpWiki]
````

<a id="w004293"></a>

## W004293 — Template:FirstPerson

Bron-ID: W004293
Namespace: Template (10)
Revisie: 28981 | Datum: 2024-06-06T17:23:25Z
Bron: https://wiki.osdev.org/Template:FirstPerson
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AFirstPerson&oldid=28981

````text
{{NoteBox|This page or section refers to its readers or editors using ''I'', ''my'', ''we'' or ''us''. It should be [{{fullurl:{{FULLPAGENAME}}|action=edit}} edited] to be in an encyclopedic tone.}}

<includeonly>[[Category:Articles Written in First Person]]</includeonly>
````

<a id="w004294"></a>

## W004294 — Template:Who

Bron-ID: W004294
Namespace: Template (10)
Revisie: 22694 | Datum: 2018-09-10T01:29:27Z
Bron: https://wiki.osdev.org/Template:Who
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AWho&oldid=22694

````text
<sup>[''who?'']</sup>
````

<a id="w004295"></a>

## W004295 — Template:Opinion

Bron-ID: W004295
Namespace: Template (10)
Revisie: 22746 | Datum: 2018-09-10T03:20:07Z
Bron: https://wiki.osdev.org/Template:Opinion
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AOpinion&oldid=22746

````text
{{NoteBox|This article is written like an opinion piece. Please [{{fullurl:{{FULLPAGENAME}}|action=edit}} edit] it to be more objective.}}

<includeonly>[[Category:Opinionated]]</includeonly>
````

<a id="w004296"></a>

## W004296 — Template:Bias

Bron-ID: W004296
Namespace: Template (10)
Revisie: 22950 | Datum: 2018-09-25T20:01:30Z
Bron: https://wiki.osdev.org/Template:Bias
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ABias&oldid=22950

````text
<center style="border: 1px solid #cfcfbf; margin-top: 25px; margin-bottom: 25px; background-color: #f0f0ff; text-align: center;">
This article '''may be unbalanced towards certain viewpoints'''. Please [{{fullurl:{{FULLPAGENAME}}|action=edit}} improve the article] by adding information on neglected viewpoints.
</center>

<includeonly>[[Category:Bias]]</includeonly>
````

<a id="w004297"></a>

## W004297 — Template:Which

Bron-ID: W004297
Namespace: Template (10)
Revisie: 22708 | Datum: 2018-09-10T02:07:17Z
Bron: https://wiki.osdev.org/Template:Which
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AWhich&oldid=22708

````text
<sup>[''which?'']</sup>
````

<a id="w004298"></a>

## W004298 — Template:How

Bron-ID: W004298
Namespace: Template (10)
Revisie: 22709 | Datum: 2018-09-10T02:08:24Z
Bron: https://wiki.osdev.org/Template:How
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AHow&oldid=22709

````text
<sup>[''how?'']</sup>
````

<a id="w004300"></a>

## W004300 — Template:EditThis

Bron-ID: W004300
Namespace: Template (10)
Revisie: 22720 | Datum: 2018-09-10T02:32:44Z
Bron: https://wiki.osdev.org/Template:EditThis
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AEditThis&oldid=22720

````text
[{{fullurl:{{FULLPAGENAME}}|action=edit}} {{{1}}}]
````

<a id="w004301"></a>

## W004301 — Template:TutorialTone

Bron-ID: W004301
Namespace: Template (10)
Revisie: 22721 | Datum: 2018-09-10T02:33:53Z
Bron: https://wiki.osdev.org/Template:TutorialTone
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ATutorialTone&oldid=22721

````text
<center style="border: 1px solid #cfcfbf; margin-top: 25px; margin-bottom: 25px; background-color: #f0f0ff; text-align: center;">
This article is written like a tutorial. Please {{EditThis|edit}} it to have more information and documentation instead of example code and step by step instructions.
</center>

<includeonly>[[Category:Tone]]</includeonly>
````

<a id="w004302"></a>

## W004302 — Template:Rude

Bron-ID: W004302
Namespace: Template (10)
Revisie: 22745 | Datum: 2018-09-10T03:18:50Z
Bron: https://wiki.osdev.org/Template:Rude
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ARude&oldid=22745

````text
{{NoteBox|This article insults or is rude to the reader. Please {{EditThis|improve the article}} by fixing the tone to not insult or talk down on the reader.}}

<includeonly>[[Category:Rude]]</includeonly>
````

<a id="w004303"></a>

## W004303 — Template:Warning

Bron-ID: W004303
Namespace: Template (10)
Revisie: 22734 | Datum: 2018-09-10T02:57:45Z
Bron: https://wiki.osdev.org/Template:Warning
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AWarning&oldid=22734

````text
<center>
{|style="border: 1px solid #bfcfcf; padding: .0em .25em .0em; background-color: #fff0f0; text-align: center;"
|
'''WARNING:''' {{{1}}}
|}
</center>
````

<a id="w004304"></a>

## W004304 — Template:What

Bron-ID: W004304
Namespace: Template (10)
Revisie: 22736 | Datum: 2018-09-10T03:02:45Z
Bron: https://wiki.osdev.org/Template:What
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AWhat&oldid=22736

````text
<sup>[''what?'']</sup>
````

<a id="w004305"></a>

## W004305 — Template:NoteBox

Bron-ID: W004305
Namespace: Template (10)
Revisie: 22788 | Datum: 2018-09-10T04:40:16Z
Bron: https://wiki.osdev.org/Template:NoteBox
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ANoteBox&oldid=22788

````text
<center>
{|style="border: 1px solid #cfcfbf; margin-top: 25px; margin-bottom: 25px; background-color: #f0f0ff; text-align: center;"
|
{{{1}}}
|}
</center>
````

<a id="w004306"></a>

## W004306 — Template:BeginnersWarning

Bron-ID: W004306
Namespace: Template (10)
Revisie: 28953 | Datum: 2024-06-06T17:16:16Z
Bron: https://wiki.osdev.org/Template:BeginnersWarning
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ABeginnersWarning&oldid=28953

````text
{{NoteBox|<big><b>WAIT! Have you read [[Getting Started]], [[Beginner Mistakes]], and some of the related [[:Category:OS theory|OS theory]]?</b></big>}}
````

<a id="w004307"></a>

## W004307 — Template:TutorialExplain

Bron-ID: W004307
Namespace: Template (10)
Revisie: 22770 | Datum: 2018-09-10T03:50:43Z
Bron: https://wiki.osdev.org/Template:TutorialExplain
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ATutorialExplain&oldid=22770

````text
{{NoteBox|This tutorial needs to explain what the code does as tutorials are not just copy paste. You can help out by {{EditThis|editing}} this page to include more context to what the code does.}}
````

<a id="w004308"></a>

## W004308 — Category:Articles Written in First Person

Bron-ID: W004308
Namespace: Category (14)
Revisie: 29403 | Datum: 2025-02-17T08:45:29Z
Bron: https://wiki.osdev.org/Category:Articles_Written_in_First_Person
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AArticles_Written_in_First_Person&oldid=29403

````text
Articles that are written like there is only one author of the article. OSDev Wiki is not a personal wiki and therefore users should not use "I" or speak in the 1st person.

[[Category:Wiki administration]]
````

<a id="w004309"></a>

## W004309 — Template:According to who

Bron-ID: W004309
Namespace: Template (10)
Revisie: 22783 | Datum: 2018-09-10T04:24:20Z
Bron: https://wiki.osdev.org/Template:According_to_who
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AAccording_to_who&oldid=22783

````text
<sup>[''according to who?'']</sup>
````

<a id="w004310"></a>

## W004310 — Category:D

Bron-ID: W004310
Namespace: Category (14)
Revisie: 27588 | Datum: 2023-01-14T06:10:59Z
Bron: https://wiki.osdev.org/Category:D
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AD&oldid=27588

````text
The D Programming Language.
[[category:languages]]
````

<a id="w004311"></a>

## W004311 — Template:Why

Bron-ID: W004311
Namespace: Template (10)
Revisie: 22811 | Datum: 2018-09-10T05:27:39Z
Bron: https://wiki.osdev.org/Template:Why
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AWhy&oldid=22811

````text
<sup>[''why?'']</sup>
````

<a id="w004312"></a>

## W004312 — Template:Spelling

Bron-ID: W004312
Namespace: Template (10)
Revisie: 22816 | Datum: 2018-09-10T05:32:00Z
Bron: https://wiki.osdev.org/Template:Spelling
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ASpelling&oldid=22816

````text
{{NoteBox|This article has many spelling and grammar mistakes. You could help out by {{EditThis|fixing them}}.}}
````

<a id="w004314"></a>

## W004314 — What order should I make things in

Bron-ID: W004314
Namespace: Main (0)
Revisie: 22841 | Datum: 2018-09-14T22:53:18Z
Bron: https://wiki.osdev.org/What_order_should_I_make_things_in
Revisie-URL: https://wiki.osdev.org/index.php?title=What_order_should_I_make_things_in&oldid=22841
Doorverwijzing: What Order Should I Make Things In?

````text
#REDIRECT [[What Order Should I Make Things In?]]
````

<a id="w004315"></a>

## W004315 — Talk:What order should I make things in

Bron-ID: W004315
Namespace: Talk (1)
Revisie: 22843 | Datum: 2018-09-14T22:53:18Z
Bron: https://wiki.osdev.org/Talk:What_order_should_I_make_things_in
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AWhat_order_should_I_make_things_in&oldid=22843
Doorverwijzing: Talk:What Order Should I Make Things In?

````text
#REDIRECT [[Talk:What Order Should I Make Things In?]]
````

<a id="w004316"></a>

## W004316 — Template:Sole Editor

Bron-ID: W004316
Namespace: Template (10)
Revisie: 22852 | Datum: 2018-09-14T23:04:57Z
Bron: https://wiki.osdev.org/Template:Sole_Editor
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ASole_Editor&oldid=22852

````text
{{NoteBox|This article was written like there is only one author. This is a wiki, not a personal site. You can help the wiki by {{EditThis|editing}} this article to remove mentions of authors.}}
````

<a id="w004317"></a>

## W004317 — Template:More References

Bron-ID: W004317
Namespace: Template (10)
Revisie: 22854 | Datum: 2018-09-14T23:10:08Z
Bron: https://wiki.osdev.org/Template:More_References
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AMore_References&oldid=22854

````text
{{NoteBox|This page is suffering from a lack of links in the ''See Also'' section. You can help change this by {{EditThis|adding links}}.}}
````

<a id="w004318"></a>

## W004318 — Template:DiscussThis

Bron-ID: W004318
Namespace: Template (10)
Revisie: 22868 | Datum: 2018-09-14T23:24:23Z
Bron: https://wiki.osdev.org/Template:DiscussThis
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ADiscussThis&oldid=22868

````text
[[Talk:{{FULLPAGENAME}}|{{{1}}}]]
````

<a id="w004319"></a>

## W004319 — Template:Says who

Bron-ID: W004319
Namespace: Template (10)
Revisie: 22876 | Datum: 2018-09-14T23:32:22Z
Bron: https://wiki.osdev.org/Template:Says_who
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ASays_who&oldid=22876

````text
<sup>[''says who?'']</sup>
````

<a id="w004320"></a>

## W004320 — Interrupts Tutorial

Bron-ID: W004320
Namespace: Main (0)
Revisie: 29118 | Datum: 2024-06-08T07:23:54Z
Bron: https://wiki.osdev.org/Interrupts_Tutorial
Revisie-URL: https://wiki.osdev.org/index.php?title=Interrupts_Tutorial&oldid=29118

````text
{{Rating|1}}
[[Category:Tutorials]]

Every operating system that needs to work with the hardware (efficiently) must make use of interrupts. For example, you could use the entirety of an AP to poll the mouse, or you could use the mouse IRQs instead and save much more CPU time, and a lot of electrical load. Therefore, every reasonable operating system makes use of interrupts.

==Pre-requisites==
Before you create an IDT, you need to create a [[GDT]], load it properly, and configure the segment registers accordingly.

==[[Interrupt Descriptor Table]]==
===Entries===
In order to make use of interrupts, you need an IDT.

When an interrupt is fired, the CPU uses the vector as an index into the IDT. The CPU reads the entry of the IDT in order to figure out what to do prior to calling the [[ISR]], and what the address of the handler is.

This is the structure of a single (32-bit) IDT entry:
<source lang="c">
typedef struct {
	uint16_t    isr_low;      // The lower 16 bits of the ISR's address
	uint16_t    kernel_cs;    // The GDT segment selector that the CPU will load into CS before calling the ISR
	uint8_t     reserved;     // Set to zero
	uint8_t     attributes;   // Type and attributes; see the IDT page
	uint16_t    isr_high;     // The higher 16 bits of the ISR's address
} __attribute__((packed)) idt_entry_t;
</source>
and this is the structure of a single (64-bit) IDT entry:
<source lang="c">
typedef struct {
	uint16_t    isr_low;      // The lower 16 bits of the ISR's address
	uint16_t    kernel_cs;    // The GDT segment selector that the CPU will load into CS before calling the ISR
	uint8_t	    ist;          // The IST in the TSS that the CPU will load into RSP; set to zero for now
	uint8_t     attributes;   // Type and attributes; see the IDT page
	uint16_t    isr_mid;      // The higher 16 bits of the lower 32 bits of the ISR's address
	uint32_t    isr_high;     // The higher 32 bits of the ISR's address
	uint32_t    reserved;     // Set to zero
} __attribute__((packed)) idt_entry_t;
</source>
===Table===
To create an IDT, simply create a 256-entry array of descriptors:
<source lang="c">
__attribute__((aligned(0x10))) 
static idt_entry_t idt[256]; // Create an array of IDT entries; aligned for performance
</source>
You will also need a special IDTR structure, which looks like:
<source lang="c">
typedef struct {
	uint16_t	limit;
	uint32_t	base;
} __attribute__((packed)) idtr_t;
</source>
for a 32-bit IDT, or like:
<source lang="c">
typedef struct {
	uint16_t	limit;
	uint64_t	base;
} __attribute__((packed)) idtr_t;
</source>
for a 64-bit IDT.

Don't forget to define an IDTR:
<source lang="c">
static idtr_t idtr;
</source>

===ISRs===
In a C source file, define a general exception handler:
<source lang="c">
__attribute__((noreturn))
void exception_handler(void);
void exception_handler() {
    __asm__ volatile ("cli; hlt"); // Completely hangs the computer
}
</source>
This will act as your main exception handler. When you receive a CPU exception, this is the handler you will call.

Now, in an assembly file ('''nasm''' '''assembler''' specifically), define these two macros:
<source lang="asm">
%macro isr_err_stub 1
isr_stub_%+%1:
    call exception_handler
    iret 
%endmacro
; if writing for 64-bit, use iretq instead
%macro isr_no_err_stub 1
isr_stub_%+%1:
    call exception_handler
    iret
%endmacro
</source>
Then, use these macros to define your 32 exception handlers:
<source lang="asm">
extern exception_handler
isr_no_err_stub 0
isr_no_err_stub 1
isr_no_err_stub 2
isr_no_err_stub 3
isr_no_err_stub 4
isr_no_err_stub 5
isr_no_err_stub 6
isr_no_err_stub 7
isr_err_stub    8
isr_no_err_stub 9
isr_err_stub    10
isr_err_stub    11
isr_err_stub    12
isr_err_stub    13
isr_err_stub    14
isr_no_err_stub 15
isr_no_err_stub 16
isr_err_stub    17
isr_no_err_stub 18
isr_no_err_stub 19
isr_no_err_stub 20
isr_no_err_stub 21
isr_no_err_stub 22
isr_no_err_stub 23
isr_no_err_stub 24
isr_no_err_stub 25
isr_no_err_stub 26
isr_no_err_stub 27
isr_no_err_stub 28
isr_no_err_stub 29
isr_err_stub    30
isr_no_err_stub 31
</source>
Finally, in assembly, define a "stub table". (This is used to prevent excessive code reuse, and not related to actual function.)

Using NASM macros:
<source lang="asm">
global isr_stub_table
isr_stub_table:
%assign i 0 
%rep    32 
    dd isr_stub_%+i ; use DQ instead if targeting 64-bit
%assign i i+1 
%endrep
</source>
===Assembling===
Finally, we can assemble the IDT:
We need to
* 1. Assign the IDT entries with the correct values,
* 2. Reload the IDTR register,
* 3. Set the interrupt flag.

To define the entries, it is appropriate to make use of a helper function. That helper function would look like:
<source lang="c">
void idt_set_descriptor(uint8_t vector, void* isr, uint8_t flags);
void idt_set_descriptor(uint8_t vector, void* isr, uint8_t flags) {
    idt_entry_t* descriptor = &idt[vector];

    descriptor->isr_low        = (uint32_t)isr & 0xFFFF;
    descriptor->kernel_cs      = 0x08; // this value can be whatever offset your kernel code selector is in your GDT
    descriptor->attributes     = flags;
    descriptor->isr_high       = (uint32_t)isr >> 16;
    descriptor->reserved       = 0;
}
</source>
for a 32-bit IDT, or like:
<source lang="c">
void idt_set_descriptor(uint8_t vector, void* isr, uint8_t flags);
void idt_set_descriptor(uint8_t vector, void* isr, uint8_t flags) {
    idt_entry_t* descriptor = &idt[vector];

    descriptor->isr_low        = (uint64_t)isr & 0xFFFF;
    descriptor->kernel_cs      = GDT_OFFSET_KERNEL_CODE;
    descriptor->ist            = 0;
    descriptor->attributes     = flags;
    descriptor->isr_mid        = ((uint64_t)isr >> 16) & 0xFFFF;
    descriptor->isr_high       = ((uint64_t)isr >> 32) & 0xFFFFFFFF;
    descriptor->reserved       = 0;
}
</source>
for a 64-bit IDT.

Finally, to set the entries at last, this is what the function would look like:
<source lang="c">
static bool vectors[IDT_MAX_DESCRIPTORS];

extern void* isr_stub_table[];

void idt_init(void);
void idt_init() {
    idtr.base = (uintptr_t)&idt[0];
    idtr.limit = (uint16_t)sizeof(idt_entry_t) * IDT_MAX_DESCRIPTORS - 1;

    for (uint8_t vector = 0; vector < 32; vector++) {
        idt_set_descriptor(vector, isr_stub_table[vector], 0x8E);
        vectors[vector] = true;
    }

    __asm__ volatile ("lidt %0" : : "m"(idtr)); // load the new IDT
    __asm__ volatile ("sti"); // set the interrupt flag
}
</source>
(IDT_MAX_DESCRIPTORS being the number of entries in your IDT - or the last entry's index + 1)

Congratulations! You have successfully defined your IDT, loaded it, and enabled interrupts.

==What to do next==
After completing this tutorial, there is still much left for you to do to fully harness the power of interrupts.

You can:
* Initialize the [[PIC]]
* Make use of PIC [[IRQ]]s
* Understand the [[NMI]]
* Configure the local [[APIC]]
* Write a driver for the [[IOAPIC]]
* Make use of [[Message Signaled Interrupts]]
==See also==
===Threads===
* [https://forum.osdev.org/viewtopic.php?f=1&t=33160&p=285871#p285871 Interrupts don't work in GRUB]
===References===
* [https://github.com/austanss/skylight/blob/trunk/kernel/src/cpu/interrupts/idt.c The code this tutorial is based off of]

[[Category:Interrupts]]
````

<a id="w004321"></a>

## W004321 — Bootloader FAQ

Bron-ID: W004321
Namespace: Main (0)
Revisie: 29620 | Datum: 2025-08-06T22:24:49Z
Bron: https://wiki.osdev.org/Bootloader_FAQ
Revisie-URL: https://wiki.osdev.org/index.php?title=Bootloader_FAQ&oldid=29620

````text
[[Category:FAQ]]{{Template:Bias}}This page intends to answer the most frequent questions related to writing a bootloader.

== GRUB or custom bootloader? ==

This is one of the never-ending arguments of the OSDev community. No one can give you a definitive answer, so it's up to you to weigh the available options and make up your mind.

=== GRUB ===

Arguments in favour of using GRUB as your bootloader are:
* You can avoid almost all legacy things that make booting complicated (e.g. [[Real Mode|real mode]] [[Segmentation|segmentation]], [[A20|A20]], the large list of BIOS interrupts, etc) so you can get to writing the kernel and the userspace faster.
* If you are willing to implement Multiboot, most arguments in favour of writing a custom bootloader are voided anyway.

=== Custom bootloader ===

Apart from the "Not Invented Here" syndrome and the "enjoying doing fun things" argument, there are technical arguments in favour of doing a custom bootloader:
* Better control over the booting process. See also [[#What should my bootloader support?|What should my bootloader support?]] for things a bootloader should do. Hint: GRUB probably doesn't do half of them.
* Less external dependencies, which may be desirable as internal components can be patched at any time whenever the need arises. This also allows to easily add features to your bootloader at your own pace.
* If your OS is not UNIX-like, a custom bootloader may make it easier to integrate it as part of the OS installation process, as GRUB's installer requires some level of "UNIX-ness" to work.
* Less bloat. Let's admit it, GRUB is not only a bootloader, but also a boot manager. A bootloader is (should be) OS-dependent, while a boot manager is (should be) able to chainload any bootloader the user selects. In other words, it's two separate concerns.

== BIOS or UEFI? ==

{{Template:Main|UEFI#Legacy bootloader or UEFI application?}}

== How do I load the kernel (BIOS)? ==

You probably followed a tutorial that shows how to write a "bootloader" that prints "Hello, World!" using the BIOS. Now you naturally ask how to load the kernel, which is indeed the intended purpose of a bootloader.

Unfortunately, loading the kernel isn't quite a simple thing, as seen from [[#What should my bootloader support?|What should my bootloader support?]] Of course, these things won't fit in the 512 bytes you got in your stage-1, so you need to load more sectors before even initialising the environment for the kernel.

Depending on the media, the boot process is differentiated enough to make sense to have different bootloaders for each supported media. Initially, however, it's probably a good idea to start with a single media (e.g. CDs), go up to the point where an initial kernel can be started and then support more media.

=== Hard disks ===

Assuming a [[MBR_(x86)|MBR]]-partitioned hard disk, the BIOS loads and executes the Master Boot Record (MBR), which is responsible for loading and executing the Volume Boot Record (VBR) of the active partition. Normally, the VBR should simply do some initial checks and load stage-2 from either the [[File Systems|filesystem]] or some reserved sectors right after the VBR (not the MBR, see "Footnote 1"). I suggest doing the latter, in order not to bother with filesystems yet. If you, however, decide to implement a filesystem already, be warned that common filesystems don't reserve much space for stage-1. [[FAT|FAT]] is probably the worst in this regard, reserving only one single sector (the VBR) for doing initial checks, parsing the filesystem tables and loading stage-2. [[Ext2|Ext2]] is somewhat better, reserving another sector right after the VBR, but then again Ext2 needs more parsing than FAT. Or, if you want, design your own filesystem (don't do it before designing and implementing the VFS), where the VBR says how many sectors at the start of the partition are reserved so you can put the whole bootloader there.

'''Footnote 1''': The space after the MBR and before the first partition is not supposed to be used by any OS.

'''Footnote 2''': For a [[GPT|GPT]]-partitioned hard disk, the boot process is slightly differentiated, as the Protective Master Boot Record (PMBR) needs to be aware of the existence of the GPT.

=== Floppies ===

When booting from a floppy, the BIOS loads and executes its first sector, which should do some initial checks and load stage-2 from either the filesystem or some reserved sectors right after sector 0.

Note however that floppies have small capacities (usually 1.44 MB) and are slow and generally unreliable. Not to mention that most machines less than a decade old don't support them anyway.

=== CDs ===

For CDs, there is the [[El-Torito|El-Torito specification]]. The CD has an [[ISO 9660|ISO-9660 filesystem]] and the BIOS reads a file from the filesystem and executes the code from it. Compliant BIOSes will load the full file if requested appropriately. However, some older BIOSes may still only load the first sector (2048 bytes), so you might want to split it into two stages; either as two files, or as two parts of the same file where the second part of the file starting at offset 2048 contains a magic number that's checked by the first part.

Note that the CDs you buy in bulk may usually be burned only once. Please look for CD-RWs which, while a bit more expensive than CD-Rs, allow to be rewritten multiple times, making them acceptable for testing your OS.

=== USB drives ===

The last time you installed a Linux distro, chances are you wrote the live CD to the USB drive and booted from it. The problem, however, is that BIOSes treat USB drives as hard disks, so the CD image needs to additionally contain a valid MBR that is then loaded by the BIOS. Now, in contrast to hard drives, the space available between the MBR and the first interesting ISO-9660 sector may be used for stage-2.

=== Network ===

A Network bootloader can load your kernel straight from the machine which compiled it. Many (all?) network chipsets and BIOSes made in the last 15 or so years support the PXE standard for network boot. You will need a tftp (Trivial FTP) server to serve your kernel and stage-2 bootloader, and may need to specially configure a suitable DHCP server.

== What should my bootloader support? ==

Apart from loading the kernel, the bootloader also has to prepare the environment appropriately before handing off control to the kernel. This could include:
* Detecting CPU/BIOS features. You need to make sure that the CPU or the BIOS supports any features that you use (e.g. INT 0x13 extensions).
* [[Detecting Memory (x86)|Getting the memory map]]. Where will you even load the kernel if you don't know what memory areas actually are memory?
* Enabling [[A20|A20]] and [[Protected Mode|protected mode]] with [[Paging|paging]] (or [[Long Mode|long mode]] if running on a [[x86-64|x86_64]]). GRUB doesn't enable paging or long mode, but that doesn't mean other bootloaders shouldn't do it either. Not enabling paging or long mode in the bootloader means that you need to decide a priori in which physical memory area the kernel will reside, which may lead to two potential consequences:
** There might not be actually any memory at that address;
** It makes it a bit harder, if not impossible, and more messy to do a [[Higher Half Kernel|higher half kernel]].

When you are more advanced, you may also want/need to do these things in the bootloader:
* [[VESA Video Modes|Set a suitable video mode]];
* Get the [[ACPI]] info;
* Load the needed kernel modules or the ramdisk;
* Feel free to add anything you deem important.

It's probably a good idea to start from the bare minimum in order to load the kernel, then implement whatever you see the kernel needs. Don't try to match an existing specification (e.g. Multiboot), otherwise it would be probably better to use an existing bootloader (e.g. GRUB).

[[Category:Bootloaders]]
````

<a id="w004322"></a>

## W004322 — User:Janworks

Bron-ID: W004322
Namespace: User (2)
Revisie: 22942 | Datum: 2018-09-25T15:44:54Z
Bron: https://wiki.osdev.org/User:Janworks
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJanworks&oldid=22942

````text
Current draft: [[User:Janworks/Intel Ethernet i219-V]]
````

<a id="w004323"></a>

## W004323 — User:Janworks/Intel Ethernet i219-V

Bron-ID: W004323
Namespace: User (2)
Revisie: 25053 | Datum: 2020-08-12T08:49:20Z
Bron: https://wiki.osdev.org/User:Janworks/Intel_Ethernet_i219-V
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJanworks%2FIntel_Ethernet_i219-V&oldid=25053

````text
''Note: This is a draft version, receiving does not work yet.''

== Overview ==
The i219-V and similar cards are present in many current devices, thus supporting them in a custom OS is desirable. Compared to older ethernet devices like the [[Intel 8254x]], the i219-V separates PHY (actual network hardware) and MAC (controller that communicates with software) into two different devices; for that reason, Intel's documentation of the i219-V does not contain any information on the card's software configuration registers. As the i219-V is designed to work with Intel's SPT PCH (Sunrise Point Platform Controller Hub, sometimes also called ''Intel 100 Series''), developers need to consult the PCH's documentation for writing a working driver. Unfortunately, up to now (September 2018), there seems to be no suitable documentation available, thus one has to deduce the needed information with older documentation and Intel's official Linux e1000e driver source code.

== Programming ==

=== Accessing the Configuration Registers ===
As stated in the PCH documentation, the GbE (Gigabit Ethernet) Controller resides at PCIe Bus 0, Device 31, Function 1. The BAR0 PCIe register then points to the card's configuration memory, which can be accessed through MMIO in the same way as with older cards.
<source lang="c">
// Map BAR0 memory area
pci_bar_info_t bar0Info;
pci_get_bar_info(deviceCfgSpaceHeader, 0, &bar0Info);
deviceBar0Memory = (uint8_t *)mmio_map(bar0Info.baseAddress, bar0Info.size, VM_R | VM_W);
if(!deviceBar0Memory)
    panic("Error: Could not map Intel i219 BAR0 MMIO.");

// Enable PCI bus mastering (else no PCI accesses possible)
uint32_t commandRegister = deviceCfgSpaceHeader->commonHeader.command;
commandRegister |= 0x04;
deviceCfgSpaceHeader->commonHeader.command = commandRegister;</source>

For convenience one can define two helper functions for accessing given register offsets:
<source lang="c">
// Reads the given device register using MMIO.
static uint32_t i219_read(e1000_register_t reg)
{
    return *((uint32_t *)(deviceBar0Memory + reg));
}

// Sets a new value for the given device register using MMIO.
static void i219_write(e1000_register_t reg, uint32_t value)
{
    *((uint32_t *)(deviceBar0Memory + reg)) = value;
}</source>

=== Retrieving the MAC Address ===
A simple test for checking whether the card controller was detected correctly is reading the MAC address from the card's RAR register.
<source lang="c">uint32_t macLow = i219_read(E1000_REG_RAL);
if(macLow != 0x00000000)
{
    // MAC can be read from RAL[0]/RAH[0] MMIO directly
    macAddress[0] = macLow & 0xFF;
    macAddress[1] = (macLow >> 8) & 0xFF;
    macAddress[2] = (macLow >> 16) & 0xFF;
    macAddress[3] = (macLow >> 24) & 0xFF;
    uint32_t macHigh = i219_read(E1000_REG_RAH);
    macAddress[4] = macHigh & 0xFF;
    macAddress[5] = (macHigh >> 8) & 0xFF;
}
else
    panic("Could not read MAC from MMIO");</source>

=== Device Initialization ===
After verifying that the device was properly detected, one can configure it for basic receiving and transmitting of Ethernet packets; the initialization steps are very similar to the e1000 cards. A list of the used registers can be found at the end of this article.

==== Disable Interrupts ====
Before modifying the card's settings one needs to disable its interrupts, to prevent them from firing before everything is set up properly.
<source lang="c">// Disable all interrupts
i219_write(E1000_REG_IMC, 0xFFFFFFFF);

// Clear pending interrupts
i219_read(E1000_REG_ICR);</source>

==== Clear Multicast Table Array ====
This register array controls multicast filtering. Since it might have random contents on startup, it needs to be zeroed at initialization.
<source lang="c">// Clear multicast table array
for(int i = 0; i < 128; ++i)
    i219_write(E1000_REG_MTA + 4 * i, 0x00000000);</source>

==== Transmit Descriptor Ring ====
The packets to be transmitted are copied into a ring buffer in RAM (consisting of a set of transmit descriptors), which is shared with the network card. The each time the driver inserts a packet, it increments the ring buffer's tail pointer; the hardware keeps the head pointer, that points toward the packet after the one that is transmitted next. All packets between the head and tail pointer are owned by the hardware; all other (empty) descriptors can be accessed by the driver.

(Legacy) descriptors look the same as for older network cards:
<source lang="c">// Structure of transmit descriptors.
typedef struct
{
    // Physical address of the transmit descriptor packet buffer.
    uint64_t address;
    
    // Length of the packet part to transmit.
    uint16_t length;
    
    // Check sum offset.
    uint8_t cso;
    
    // Command field.
    uint8_t command;
    
    // Packet transmission status.
    uint8_t status;
    
    // Checksum start field.
    uint8_t css;
    
    // Unused.
    uint16_t special;
} __attribute__((__packed__)) tx_desc_t;</source>

One can now allocate the ring buffer and the packet memory, and pass their physical address for DMA to the network card. The amount of descriptors can be chosen rather freely, but a size of <code>TX_DESC_COUNT=256</code> seems to be usual. The packet buffer size depends on the maximum transmission unit (MTU); if no long packets (jumbo frames) are supported, <code>TX_BUFFER_SIZE=2048</code> is sufficient. Since these memory regions are accessed via DMA, they need to be contiguous in physical memory!
<source lang="c">// Allocate transmit data buffer
uint64_t txBufferMemPhy;
txBufferMem = heap_alloc_contiguous(TX_DESC_COUNT * TX_BUFFER_SIZE, VM_R | VM_W, &txBufferMemPhy);
if(!txBufferMem)
    panic("Could not allocate i219 transmit data buffer.");

// Allocate and initialize transmit descriptor buffer
uint64_t txDescriptorsPhy;
txDescriptors = heap_alloc_contiguous(TX_DESC_COUNT * sizeof(tx_desc_t), VM_R | VM_W, &txDescriptorsPhy);
if(!txDescriptors)
    panic("Could not allocate i219 transmit descriptor buffer.");
for(int i = 0; i < TX_DESC_COUNT; ++i)
{
    // Initialize descriptor
    tx_desc_t *currDesc = &txDescriptors[i];
    currDesc->address = txBufferMemPhy + i * TX_BUFFER_SIZE;
    currDesc->length = 0;
    currDesc->status = 0;
    currDesc->cso = 0;
    currDesc->css = 0;
    currDesc->special = 0;
}

// Pass transmit descriptor buffer
i219_write(E1000_REG_TDBAH, txDescriptorsPhy >> 32);
i219_write(E1000_REG_TDBAL, txDescriptorsPhy & 0xFFFFFFFF);
i219_write(E1000_REG_TDLEN, TX_DESC_COUNT * sizeof(tx_desc_t));
i219_write(E1000_REG_TDH, 0);
i219_write(E1000_REG_TDT, 0);
txTail = 0;</source>

==== Receive Descriptor Ring ====
<source lang="c">TODO</source>

==== Enable transmit/receive ====
Once all queues are created, transmitting and receiving can be started by setting the appropriate flags in the respective control registers:
<source lang="c">// Enable transmitter
uint32_t tctl = i219_read(E1000_REG_TCTL);
tctl |= E1000_TCTL_EN; // EN (Transmitter Enable)
tctl |= E1000_TCTL_PSP; // PSP (Pad Short Packets)
tctl |= E1000_TCTL_RTLC; // RTLC (Re-transmit on Late Collision)
i219_write(E1000_REG_TCTL, tctl);

// Enable receiver
uint32_t rctl = i219_read(E1000_REG_RCTL);
rctl |= E1000_RCTL_EN; // EN (Receiver Enable)
rctl &= ~E1000_RCTL_SBP; // SBP (Store Pad Packets)
rctl |= E1000_RCTL_BAM; // BAM (Broadcast Accept Mode)
rctl &= ~E1000_RCTL_SZ_4096;
rctl |= E1000_RCTL_SZ_2048; // BSIZE = 2048 (Receive Buffer Size)
rctl &= ~E1000_RCTL_BSEX;
rctl |= E1000_RCTL_SECRC; // SECRC (Strip Ethernet CRC)
i219_write(E1000_REG_RCTL, rctl);</source>

==== Enable Interrupts ====
Finally one can enable interrupts again, the card should now be ready to receive and transmit packets.
<source lang="c">i219_write(E1000_REG_IMS, E1000_IMS_RXT0);</source>
This is the most important interrupt; it fires when a new packet was received by the network card. The card also provides mechanisms to delay interrupts, such that one interrupt accounts for receiving multiple packets, instead of firing once for each packet.

=== Sending Packets ===
Sending packets works the same way as for e1000 cards: The driver waits until the next descriptor is empty (i.e., its contents were sent by the hardware), stores the packet data and increments its tail pointer, such that the hardware can access the descriptor.
<source lang="c">// Wait until descriptor at current tail is unused
tx_desc_t *desc = &txDescriptors[txTail];
if(desc->length)
    while(!(desc->status & 0xF)) // Catches "descriptor done" (DD) and various errors
        pause_once();

// Copy packet data
memcpy(&txBufferMem[txTail * TX_BUFFER_SIZE], packet, packetLength);
desc->length = packetLength;
desc->command = 0x8 | 0x2 | 0x1; // RS (Report Status), IFCS (Insert FCS), EOP (End Of Packet)
desc->cso = 0;
desc->status = 0;
desc->css = 0;
desc->special = 0;

// Update tail
++txTail;
if(txTail == TX_DESC_COUNT)
    txTail = 0;
i219_write(E1000_REG_TDT, txTail);</source>

=== Receiving Packets ===
The packets receieved are copied into a ring buffer in RAM (consisting of a set of receive descriptors), which is shared with the network card. The each time the driver processes a packet, it increments the ring buffer's tail pointer; the hardware keeps the head pointer, that points toward the packet after the one that is transmitted next. All packets between the head and tail pointer are owned by the hardware; all other (empty) descriptors can be accessed by the driver.

(Legacy) descriptors look the same as for older network cards:
<source lang="c">// Structure of receive descriptors.
typedef struct {
	volatile uint64_t addr;
	volatile uint16_t length;
	volatile uint16_t checksum;
	volatile uint8_t status;
	volatile uint8_t errors;
	volatile uint16_t special;
} __attribute__((__packed__)) rx_desc_t;</source>

One can now allocate the ring buffer and the packet memory, and pass their physical address for DMA to the network card. The amount of descriptors can be chosen rather freely, but a size of <code>RX_DESC_COUNT=256</code> seems to be usual. The packet buffer size depends on the maximum transmission unit (MTU); if no long packets (jumbo frames) are supported, <code>RX_BUFFER_SIZE=2048</code> is sufficient. Since these memory regions are accessed via DMA, they need to be contiguous in physical memory!
<source lang="c">// Allocate transmit data buffer
uint64_t rxBufferMemPhy;
rxBufferMem = heap_alloc_contiguous(RX_DESC_COUNT * RX_BUFFER_SIZE, VM_R | VM_W, &rxBufferMemPhy);
if(!rxBufferMem)
    panic("Could not allocate i219 receive data buffer.");

// Allocate and initialize transmit descriptor buffer
uint64_t rxDescriptorsPhy;
rxDescriptors = heap_alloc_contiguous(RX_DESC_COUNT * sizeof(rx_desc_t), VM_R | VM_W, &rxDescriptorsPhy);
if(!rxDescriptors)
    panic("Could not allocate i219 receive descriptor buffer.");
for(int i = 0; i < RX_DESC_COUNT; ++i)
{
    // Initialize descriptor
    rx_desc_t *currDesc = &rxDescriptors[i];
    currDesc->address = rxBufferMemPhy + i * RX_BUFFER_SIZE;
    currDesc->length = 0;
    currDesc->status = 0;
    currDesc->cso = 0;
    currDesc->css = 0;
    currDesc->special = 0;
}

// Pass receive descriptor buffer
i219_write(E1000_REG_RDBAH, rxDescriptorsPhy >> 32);
i219_write(E1000_REG_RDBAL, rxDescriptorsPhy & 0xFFFFFFFF);
i219_write(E1000_REG_RDLEN, RX_DESC_COUNT * sizeof(rx_desc_t));
i219_write(E1000_REG_RDH, 0);
i219_write(E1000_REG_RDT, RX_DESC_COUNT);
//Prefetch control might need to be set to zero
i219_write(E1000_REG_RDTR, 0);
rxCur = 0;</source>

And handling a RXT0 interrupt:
<source lang="c">
uint32_t status = i219_read(E1000_REG_ICR);
if (status & E1000_ICR_RXT0)
	{
		//kprintf(u"Packet interrupt: RX %d\n", dinfo->rxCur);
		while (dinfo->maprxdescs[dinfo->rxCur].status != 0)
		{
			uint8_t *buf = raw_offset<uint8_t*>(dinfo->maprxbuf, dinfo->maprxdescs[dinfo->rxCur].addr - dinfo->rx_buf_phy);
			uint16_t len = dinfo->maprxdescs[dinfo->rxCur].length;

			// Here you should inject the received packet into your network stack


			dinfo->maprxdescs[dinfo->rxCur].status = 0;
			auto old_cur = dinfo->rxCur;
			dinfo->rxCur = (dinfo->rxCur + 1) % dinfo->RX_DESC_COUNT;
			i219_write(E1000_REG_RDT, old_cur);
		}
}
</source>

=== Register and Flags List ===
<source lang="c">/*
Intel network controller register and flag definitions.
Taken from Linux e1000e driver.
*/

/* Registers */
typedef enum
{
    E1000_REG_CTRL = 0x00000, /* Device Control - RW */
    E1000_REG_STATUS = 0x00008, /* Device Status - RO */
    E1000_REG_EECD = 0x00010, /* EEPROM/Flash Control - RW */
    E1000_REG_EERD = 0x00014, /* EEPROM Read - RW */
    E1000_REG_CTRL_EXT = 0x00018, /* Extended Device Control - RW */
    E1000_REG_FLA = 0x0001C, /* Flash Access - RW */
    E1000_REG_MDIC = 0x00020, /* MDI Control - RW */
    E1000_REG_SCTL = 0x00024, /* SerDes Control - RW */
    E1000_REG_FCAL = 0x00028, /* Flow Control Address Low - RW */
    E1000_REG_FCAH = 0x0002C, /* Flow Control Address High -RW */
    E1000_REG_FEXT = 0x0002C, /* Future Extended - RW */
    E1000_REG_FEXTNVM = 0x00028, /* Future Extended NVM - RW */
    E1000_REG_FEXTNVM3 = 0x0003C, /* Future Extended NVM 3 - RW */
    E1000_REG_FEXTNVM4 = 0x00024, /* Future Extended NVM 4 - RW */
    E1000_REG_FEXTNVM6 = 0x00010, /* Future Extended NVM 6 - RW */
    E1000_REG_FEXTNVM7 = 0x000E4, /* Future Extended NVM 7 - RW */
    E1000_REG_FEXTNVM9 = 0x5BB4, /* Future Extended NVM 9 - RW */
    E1000_REG_FEXTNVM11 = 0x5BBC, /* Future Extended NVM 11 - RW */
    E1000_REG_PCIEANACFG = 0x00F18, /* PCIE Analog Config */
    E1000_REG_FCT = 0x00030, /* Flow Control Type - RW */
    E1000_REG_VET = 0x00038, /* VLAN Ether Type - RW */
    E1000_REG_ICR = 0x000C0, /* Interrupt Cause Read - R/clr */
    E1000_REG_ITR = 0x000C4, /* Interrupt Throttling Rate - RW */
    E1000_REG_ICS = 0x000C8, /* Interrupt Cause Set - WO */
    E1000_REG_IMS = 0x000D0, /* Interrupt Mask Set - RW */
    E1000_REG_IMC = 0x000D8, /* Interrupt Mask Clear - WO */
    E1000_REG_IAM = 0x000E0, /* Interrupt Acknowledge Auto Mask */
    E1000_REG_IVAR = 0x000E4, /* Interrupt Vector Allocation Register - RW */
    E1000_REG_SVCR = 0x000F0,
    E1000_REG_SVT = 0x000F4,
    E1000_REG_LPIC = 0x000FC, /* Low Power IDLE control */
    E1000_REG_RCTL = 0x00100, /* Rx Control - RW */
    E1000_REG_FCTTV = 0x00170, /* Flow Control Transmit Timer Value - RW */
    E1000_REG_TXCW = 0x00178, /* Tx Configuration Word - RW */
    E1000_REG_RXCW = 0x00180, /* Rx Configuration Word - RO */
    E1000_REG_PBA_ECC = 0x01100, /* PBA ECC Register */
    E1000_REG_TCTL = 0x00400, /* Tx Control - RW */
    E1000_REG_TCTL_EXT = 0x00404, /* Extended Tx Control - RW */
    E1000_REG_TIPG = 0x00410, /* Tx Inter-packet gap -RW */
    E1000_REG_AIT = 0x00458, /* Adaptive Interframe Spacing Throttle - RW */
    E1000_REG_LEDCTL = 0x00E00, /* LED Control - RW */
    E1000_REG_LEDMUX = 0x08130, /* LED MUX Control */
    E1000_REG_EXTCNF_CTRL = 0x00F00, /* Extended Configuration Control */
    E1000_REG_EXTCNF_SIZE = 0x00F08, /* Extended Configuration Size */
    E1000_REG_PHY_CTRL = 0x00F10, /* PHY Control Register in CSR */
    E1000_REG_PBA = 0x01000, /* Packet Buffer Allocation - RW */
    E1000_REG_PBS = 0x01008, /* Packet Buffer Size */
    E1000_REG_PBECCSTS = 0x0100C, /* Packet Buffer ECC Status - RW */
    E1000_REG_IOSFPC = 0x00F28, /* TX corrupted data  */
    E1000_REG_EEMNGCTL = 0x01010, /* MNG EEprom Control */
    E1000_REG_EEWR = 0x0102C, /* EEPROM Write Register - RW */
    E1000_REG_FLOP = 0x0103C, /* FLASH Opcode Register */
    E1000_REG_ERT = 0x02008, /* Early Rx Threshold - RW */
    E1000_REG_FCRTL = 0x02160, /* Flow Control Receive Threshold Low - RW */
    E1000_REG_FCRTH = 0x02168, /* Flow Control Receive Threshold High - RW */
    E1000_REG_PSRCTL = 0x02170, /* Packet Split Receive Control - RW */
    E1000_REG_RDFH = 0x02410, /* Rx Data FIFO Head - RW */
    E1000_REG_RDFT = 0x02418, /* Rx Data FIFO Tail - RW */
    E1000_REG_RDFHS = 0x02420, /* Rx Data FIFO Head Saved - RW */
    E1000_REG_RDFTS = 0x02428, /* Rx Data FIFO Tail Saved - RW */
    E1000_REG_RDFPC = 0x02430, /* Rx Data FIFO Packet Count - RW */
    E1000_REG_RDTR = 0x02820, /* Rx Delay Timer - RW */
    E1000_REG_RADV = 0x0282C, /* Rx Interrupt Absolute Delay Timer - RW */
    
    E1000_REG_RAL = 0x05400, // Receive Address Low
    E1000_REG_RAH = 0x05404, // Receive Address High
    E1000_REG_RDBAL = 0x02800, // RX Descriptor Base Address Low
    E1000_REG_RDBAH = 0x02804, // RX Descriptor Base Address High
    E1000_REG_RDLEN = 0x02808, // RX Descriptor Length
    E1000_REG_RDH = 0x02810, // RX Descriptor Head
    E1000_REG_RDT = 0x02818, // RX Descriptor Tail
    E1000_REG_TDBAL = 0x03800, // TX Descriptor Base Address Low
    E1000_REG_TDBAH = 0x03804, // TX Descriptor Base Address High
    E1000_REG_TDLEN = 0x03808, // TX Descriptor Length
    E1000_REG_TDH = 0x03810, // TX Descriptor Head
    E1000_REG_TDT = 0x03818, // TX Descriptor Tail
    
    E1000_REG_MTA = 0x05200
} e1000_register_t;

/* Device Control */
typedef enum
{
    E1000_CTRL_FD = 0x00000001, /* Full duplex.0=half; 1=full */
    E1000_CTRL_GIO_MASTER_DISABLE = 0x00000004, /* Blocks new Master reqs */
    E1000_CTRL_LRST = 0x00000008, /* Link reset. 0=normal,1=reset */
    E1000_CTRL_ASDE = 0x00000020, /* Auto-speed detect enable */
    E1000_CTRL_SLU = 0x00000040, /* Set link up (Force Link) */
    E1000_CTRL_ILOS = 0x00000080, /* Invert Loss-Of Signal */
    E1000_CTRL_SPD_SEL = 0x00000300, /* Speed Select Mask */
    E1000_CTRL_SPD_10 = 0x00000000, /* Force 10Mb */
    E1000_CTRL_SPD_100 = 0x00000100, /* Force 100Mb */
    E1000_CTRL_SPD_1000 = 0x00000200, /* Force 1Gb */
    E1000_CTRL_FRCSPD = 0x00000800, /* Force Speed */
    E1000_CTRL_FRCDPX = 0x00001000, /* Force Duplex */
    E1000_CTRL_LANPHYPC_OVERRIDE = 0x00010000, /* SW control of LANPHYPC */
    E1000_CTRL_LANPHYPC_VALUE = 0x00020000, /* SW value of LANPHYPC */
    E1000_CTRL_MEHE = 0x00080000, /* Memory Error Handling Enable */
    E1000_CTRL_SWDPIN0 = 0x00040000, /* SWDPIN 0 value */
    E1000_CTRL_SWDPIN1 = 0x00080000, /* SWDPIN 1 value */
    E1000_CTRL_ADVD3WUC = 0x00100000, /* D3 WUC */
    E1000_CTRL_EN_PHY_PWR_MGMT = 0x00200000, /* PHY PM enable */
    E1000_CTRL_SWDPIO0 = 0x00400000, /* SWDPIN 0 Input or output */
    E1000_CTRL_RST = 0x04000000, /* Global reset */
    E1000_CTRL_RFCE = 0x08000000, /* Receive Flow Control enable */
    E1000_CTRL_TFCE = 0x10000000, /* Transmit flow control enable */
    E1000_CTRL_VME = 0x40000000, /* IEEE VLAN mode enable */
    E1000_CTRL_PHY_RST = 0x80000000, /* PHY Reset */
} e1000_ctrl_flags_t;

/* Extended Device Control */
typedef enum
{
    E1000_CTRL_EXT_LPCD = 0x00000004, /* LCD Power Cycle Done */
    E1000_CTRL_EXT_SDP3_DATA = 0x00000080, /* SW Definable Pin 3 data */
    E1000_CTRL_EXT_FORCE_SMBUS = 0x00000800, /* Force SMBus mode */
    E1000_CTRL_EXT_EE_RST = 0x00002000, /* Reinitialize from EEPROM */
    E1000_CTRL_EXT_SPD_BYPS = 0x00008000, /* Speed Select Bypass */
    E1000_CTRL_EXT_RO_DIS = 0x00020000, /* Relaxed Ordering disable */
    E1000_CTRL_EXT_DMA_DYN_CLK_EN = 0x00080000, /* DMA Dynamic Clk Gating */
    E1000_CTRL_EXT_LINK_MODE_MASK = 0x00C00000,
    E1000_CTRL_EXT_LINK_MODE_PCIE_SERDES = 0x00C00000,
    E1000_CTRL_EXT_EIAME = 0x01000000,
    E1000_CTRL_EXT_DRV_LOAD = 0x10000000, /* Drv loaded bit for FW */
    E1000_CTRL_EXT_IAME = 0x08000000, /* Int ACK Auto-mask */
    E1000_CTRL_EXT_PBA_CLR = 0x80000000, /* PBA Clear */
    E1000_CTRL_EXT_LSECCK = 0x00001000,
    E1000_CTRL_EXT_PHYPDEN = 0x00100000,
} e1000_ctrl_ext_flags_t;

/* Device Status */
typedef enum
{
    E1000_STATUS_FD = 0x00000001, /* Duplex 0=half 1=full */
    E1000_STATUS_LU = 0x00000002, /* Link up.0=no,1=link */
    E1000_STATUS_FUNC_MASK = 0x0000000C, /* PCI Function Mask */
    E1000_STATUS_FUNC_SHIFT = 2,
    E1000_STATUS_FUNC_1 = 0x00000004, /* Function 1 */
    E1000_STATUS_TXOFF = 0x00000010, /* transmission paused */
    E1000_STATUS_SPEED_MASK = 0x000000C0,
    E1000_STATUS_SPEED_10 = 0x00000000, /* Speed 10Mb/s */
    E1000_STATUS_SPEED_100 = 0x00000040, /* Speed 100Mb/s */
    E1000_STATUS_SPEED_1000 = 0x00000080, /* Speed 1000Mb/s */
    E1000_STATUS_LAN_INIT_DONE = 0x00000200, /* Lan Init Compltn by NVM */
    E1000_STATUS_PHYRA = 0x00000400, /* PHY Reset Asserted */
    E1000_STATUS_GIO_MASTER_ENABLE = 0x00080000, /* Master request status */
    E1000_STATUS_2P5_SKU = 0x00001000, /* Val of 2.5GBE SKU strap */
    E1000_STATUS_2P5_SKU_OVER = 0x00002000, /* Val of 2.5GBE SKU Over */
} e1000_device_status_flags_t;

/* Receive Control */
typedef enum
{
    E1000_RCTL_EN = 0x00000002, /* enable */
    E1000_RCTL_SBP = 0x00000004, /* store bad packet */
    E1000_RCTL_UPE = 0x00000008, /* unicast promisc enable */
    E1000_RCTL_MPE = 0x00000010, /* multicast promisc enable */
    E1000_RCTL_LPE = 0x00000020, /* long packet enable */
    E1000_RCTL_LBM_NO = 0x00000000, /* no loopback mode */
    E1000_RCTL_LBM_MAC = 0x00000040, /* MAC loopback mode */
    E1000_RCTL_LBM_TCVR = 0x000000C0, /* tcvr loopback mode */
    E1000_RCTL_DTYP_PS = 0x00000400, /* Packet Split descriptor */
    E1000_RCTL_RDMTS_HALF = 0x00000000, /* Rx desc min thresh size */
    E1000_RCTL_RDMTS_HEX = 0x00010000,
    E1000_RCTL_RDMTS1_HEX = E1000_RCTL_RDMTS_HEX,
    E1000_RCTL_MO_SHIFT = 12, /* multicast offset shift */
    E1000_RCTL_MO_3 = 0x00003000, /* multicast offset 15:4 */
    E1000_RCTL_BAM = 0x00008000, /* broadcast enable */
    /* these buffer sizes are valid if E1000_RCTL_BSEX is 0 */
    E1000_RCTL_SZ_2048 = 0x00000000, /* Rx buffer size 2048 */
    E1000_RCTL_SZ_1024 = 0x00010000, /* Rx buffer size 1024 */
    E1000_RCTL_SZ_512 = 0x00020000, /* Rx buffer size 512 */
    E1000_RCTL_SZ_256 = 0x00030000, /* Rx buffer size 256 */
    /* these buffer sizes are valid if E1000_RCTL_BSEX is 1 */
    E1000_RCTL_SZ_16384 = 0x00010000, /* Rx buffer size 16384 */
    E1000_RCTL_SZ_8192 = 0x00020000, /* Rx buffer size 8192 */
    E1000_RCTL_SZ_4096 = 0x00030000, /* Rx buffer size 4096 */
    E1000_RCTL_VFE = 0x00040000, /* vlan filter enable */
    E1000_RCTL_CFIEN = 0x00080000, /* canonical form enable */
    E1000_RCTL_CFI = 0x00100000, /* canonical form indicator */
    E1000_RCTL_DPF = 0x00400000, /* discard pause frames */
    E1000_RCTL_PMCF = 0x00800000, /* pass MAC control frames */
    E1000_RCTL_BSEX = 0x02000000, /* Buffer size extension */
    E1000_RCTL_SECRC = 0x04000000, /* Strip Ethernet CRC */
} e1000_rx_ctrl_flags_t;

/* Transmit Control */
typedef enum
{
    E1000_TCTL_EN = 0x00000002, /* enable Tx */
    E1000_TCTL_PSP = 0x00000008, /* pad short packets */
    E1000_TCTL_CT = 0x00000ff0, /* collision threshold */
    E1000_TCTL_COLD = 0x003ff000, /* collision distance */
    E1000_TCTL_RTLC = 0x01000000, /* Re-transmit on late collision */
    E1000_TCTL_MULR = 0x10000000, /* Multiple request support */
} e1000_tx_ctrl_flags_t;

/* Interrupt Cause Read */
typedef enum
{
    E1000_ICR_TXDW = 0x00000001, /* Transmit desc written back */
    E1000_ICR_LSC = 0x00000004, /* Link Status Change */
    E1000_ICR_RXSEQ = 0x00000008, /* Rx sequence error */
    E1000_ICR_RXDMT0 = 0x00000010, /* Rx desc min. threshold (0) */
    E1000_ICR_RXT0 = 0x00000080, /* Rx timer intr (ring 0) */
    E1000_ICR_ECCER = 0x00400000, /* Uncorrectable ECC Error */
    E1000_ICR_INT_ASSERTED = 0x80000000, /* If this bit asserted, the driver should claim the interrupt */
    E1000_ICR_RXQ0 = 0x00100000, /* Rx Queue 0 Interrupt */
    E1000_ICR_RXQ1 = 0x00200000, /* Rx Queue 1 Interrupt */
    E1000_ICR_TXQ0 = 0x00400000, /* Tx Queue 0 Interrupt */
    E1000_ICR_TXQ1 = 0x00800000, /* Tx Queue 1 Interrupt */
    E1000_ICR_OTHER = 0x01000000, /* Other Interrupts */
} e1000_intr_cause_flags_t;

/* Interrupt Mask Set */
typedef enum
{
    E1000_IMS_TXDW = E1000_ICR_TXDW, /* Tx desc written back */
    E1000_IMS_LSC = E1000_ICR_LSC, /* Link Status Change */
    E1000_IMS_RXSEQ = E1000_ICR_RXSEQ, /* Rx sequence error */
    E1000_IMS_RXDMT0 = E1000_ICR_RXDMT0, /* Rx desc min. threshold */
    E1000_IMS_RXT0 = E1000_ICR_RXT0, /* Rx timer intr */
    E1000_IMS_ECCER = E1000_ICR_ECCER, /* Uncorrectable ECC Error */
    E1000_IMS_RXQ0 = E1000_ICR_RXQ0, /* Rx Queue 0 Interrupt */
    E1000_IMS_RXQ1 = E1000_ICR_RXQ1, /* Rx Queue 1 Interrupt */
    E1000_IMS_TXQ0 = E1000_ICR_TXQ0, /* Tx Queue 0 Interrupt */
    E1000_IMS_TXQ1 = E1000_ICR_TXQ1, /* Tx Queue 1 Interrupt */
    E1000_IMS_OTHER = E1000_ICR_OTHER, /* Other Interrupt */
} e1000_intr_mask_flags_t;</source>
````

<a id="w004324"></a>

## W004324 — Category:Bias

Bron-ID: W004324
Namespace: Category (14)
Revisie: 22952 | Datum: 2018-09-25T20:06:35Z
Bron: https://wiki.osdev.org/Category:Bias
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ABias&oldid=22952

````text
This category lists pages which may biased towards certain viewpoints.
````

<a id="w004325"></a>

## W004325 — User talk:Carver

Bron-ID: W004325
Namespace: User talk (3)
Revisie: 22991 | Datum: 2018-09-27T06:07:36Z
Bron: https://wiki.osdev.org/User_talk:Carver
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ACarver&oldid=22991

````text

````

<a id="w004326"></a>

## W004326 — Talk:Babystep5

Bron-ID: W004326
Namespace: Talk (1)
Revisie: 27485 | Datum: 2022-11-05T23:55:00Z
Bron: https://wiki.osdev.org/Talk:Babystep5
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABabystep5&oldid=27485

````text
== keyhandler not preserving registers ==

A user on IRC was following this tutorial series and noticed that keyhandler doesn't preserve ax, bx and other registers possibly used by printreg16. Can we afford to patch the tutorials individually or is it time to replace this antiquated series by something better and more complete? [[User:Glauxosdever|Glauxosdever]] 06:31, 26 September 2018 (CDT)

:It's pretty strange for a bootloader tutorial to install a custom keyboard handler in the first place. A replacement would be nice, but I'm not sure if the replacement should be a bootloader tutorial or a general real mode development tutorial. --[[User:Octocontrabass|Octocontrabass]] 18:55, 5 November 2022 (CDT)
````

<a id="w004327"></a>

## W004327 — User:Glauxosdever/Bootloader FAQ

Bron-ID: W004327
Namespace: User (2)
Revisie: 22987 | Datum: 2018-09-26T13:18:48Z
Bron: https://wiki.osdev.org/User:Glauxosdever/Bootloader_FAQ
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGlauxosdever%2FBootloader_FAQ&oldid=22987
Doorverwijzing: Bootloader FAQ

````text
#REDIRECT [[Bootloader FAQ]]
````

<a id="w004328"></a>

## W004328 — Talk:SUBLEQ

Bron-ID: W004328
Namespace: Talk (1)
Revisie: 22993 | Datum: 2018-09-27T06:12:40Z
Bron: https://wiki.osdev.org/Talk:SUBLEQ
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ASUBLEQ&oldid=22993

````text
Reverted revision 22971 due to the fact that the links contain more info than the page itself. --[[User:Carver|Carver]]
````

<a id="w004329"></a>

## W004329 — VMware tools

Bron-ID: W004329
Namespace: Main (0)
Revisie: 28102 | Datum: 2023-07-09T22:37:51Z
Bron: https://wiki.osdev.org/VMware_tools
Revisie-URL: https://wiki.osdev.org/index.php?title=VMware_tools&oldid=28102

````text
VMware's various virtualization products implement a backdoor which provides some useful functionality to the guest operating system, similar to the [[VirtualBox Guest Additions]]. In addition to VMware, QEMU also implements some of the functionality of the VMware backdoor, including support for absolute mouse positioning which can be very useful if you lack a USB stack and want to be able to productively use QEMU's integrated VNC server.

Sample code in this article is provided from ToaruOS and is based on research from the VMware SVGA developer kit and open-vm-tools.

== VMware Backdoor ==

To communicate with the host, VMware uses a port-and-register based backdoor. Magic values, commands, and arguments are stored in the eax, ebx, ecx, and edx registers, a "magic" port operation is executed, and then the returned values are stored in these registers. There are three ways to access the backdoor, which uses two different magic port values. Most backdoor commands use the single port "in" version, while longer messages are read and written through the "high bandwidth" "rep out/in" version.

First, let's define a structure to store the register values and also provide some helpful aliases through unions. I am using unnamed unions here, but you may wish to name them depending on your compiler configuration:

<source lang="c">
typedef struct {
	union {
		uint32_t ax;
		uint32_t magic;
	};
	union {
		uint32_t bx;
		size_t size;
	};
	union {
		uint32_t cx;
		uint16_t command;
	};
	union {
		uint32_t dx;
		uint16_t port;
	};
	uint32_t si;
	uint32_t di;
} vmware_cmd;
</source>

The union values explain the purpose of some of the registers in the single port "in" version of the backdoor. DX will contain the port number used for the port read, EAX will store a magic value, EBX may contain a size argument, and CX will indicate the command we are requesting from the VM. You will not we also have values for SI and DI, which are used by the "high bandwidth" version of the backdoor.

Now let's set up some functions to perform the various backdoor calls. GCC's inline assembly makes this very easy, as we can just pass the values from our struct into a short assembly call as the registers we want to set. We'll have three different methods available: one for the single-port backdoor and both an "in" and "out" method for the high bandwidth version. We'll also set the magic values and ports appropriately before calling the assembly instructions:

<source lang="c">
#define VMWARE_MAGIC  0x564D5868
#define VMWARE_PORT   0x5658
#define VMWARE_PORTHB 0x5659

void vmware_send(vmware_cmd * cmd) {
	cmd->magic = VMWARE_MAGIC;
	cmd->port = VMWARE_PORT;
	asm volatile("in %%dx, %0" : "+a"(cmd->ax), "+b"(cmd->bx), "+c"(cmd->cx), "+d"(cmd->dx), "+S"(cmd->si), "+D"(cmd->di));
}

static void vmware_send_hb(vmware_cmd * cmd) {
	cmd->magic = VMWARE_MAGIC;
	cmd->port = VMWARE_PORTHB;
	asm volatile("cld; rep; outsb" : "+a"(cmd->ax), "+b"(cmd->bx), "+c"(cmd->cx), "+d"(cmd->dx), "+S"(cmd->si), "+D"(cmd->di));
}

static void vmware_get_hb(vmware_cmd * cmd) {
	cmd->magic = VMWARE_MAGIC;
	cmd->port = VMWARE_PORTHB;
	asm volatile("cld; rep; insb" : "+a"(cmd->ax), "+b"(cmd->bx), "+c"(cmd->cx), "+d"(cmd->dx), "+S"(cmd->si), "+D"(cmd->di));
}
</source>

== Detecting the VMware Backdoor ==

''Note: Blindly reading from IO ports is probably not the best approach to detecting some special VM hardware.''

In order to ensure that we are communicating with a VMware backdoor implementation, we should query it as follows:

<source lang="c">
int is_vmware_backdoor(void) {
	vmware_cmd cmd;
	cmd.bx = ~VMWARE_MAGIC;
	cmd.command = CMD_GETVERSION;
	vmware_send(&cmd);
	
	if (cmd.bx != VMWARE_MAGIC || cmd.ax == 0xFFFFFFFF) {
		/* Not a backdoor! */
		return 0;
	}
	
	return 1;
}
</source>

== Absolute Mouse Coordinates ==

The backdoor allows us to disable the PS/2 mouse and instead receive absolute mouse coordinates. This means we don't need to "capture" the mouse pointer in the VM. As an extra bonus, QEMU implements this functionality as well.

When the absolute mouse is enabled, the PS/2 mouse status bit indicates not that the PS/2 port should be read, but that the backdoor should be used to read mouse data instead, so you should route PS/2 mouse interrupts to your VMware backdoor handler where appropriate.

We'll provide two functions, one to enable the absolute mouse, and one to disable it.

Our enable function will perform 4 backdoor calls: One to enable the mouse functionality, two to request status information, and one to switch to absolute mode. Let's define some magic values for these commands:

<source lang="c">
#define CMD_ABSPOINTER_DATA    39
#define CMD_ABSPOINTER_STATUS  40
#define CMD_ABSPOINTER_COMMAND 41

#define ABSPOINTER_ENABLE   0x45414552 /* Q E A E */
#define ABSPOINTER_RELATIVE 0xF5
#define ABSPOINTER_ABSOLUTE 0x53424152 /* R A B S */
</source>

Now we can enable the absolute mouse like this:

<source lang="c">
void mouse_absolute(void) {
	vmware_cmd cmd;

	/* Enable */
	cmd.bx = ABSPOINTER_ENABLE;
	cmd.command = CMD_ABSPOINTER_COMMAND;
	vmware_send(&cmd);

	/* Status */
	cmd.bx = 0;
	cmd.command = CMD_ABSPOINTER_STATUS;
	vmware_send(&cmd);

	/* Read data (1) */
	cmd.bx = 1;
	cmd.command = CMD_ABSPOINTER_DATA;
	vmware_send(&cmd);

	/* Enable absolute */
	cmd.bx = ABSPOINTER_ABSOLUTE;
	cmd.command = CMD_ABSPOINTER_COMMAND;
	vmware_send(&cmd);
}
</source>

Disabling the mouse is a bit easier as it is a single backdoor command:

<source lang="c">
void mouse_relative(void) {
	vmware_cmd cmd;
	cmd.bx = ABSPOINTER_RELATIVE;
	cmd.command = CMD_ABSPOINTER_COMMAND;
	vmware_send(&cmd);
}
</source>

With these two functions, we can toggle the absolute mouse pointer on and off, which is useful if users want to play games in our OS that require a relative mouse pointer (like Quake), so we'll want to provide a user-accessible method to toggling the pointer mode.

Now let's handle the actual mouse events. When your PS/2 mouse driver receives a byte from the mouse when the absolute pointer is enabled, it should ignore that byte and call a function in the VMware driver to get the actual mouse data.

<source lang="c">
void vmware_handle_mouse(void) {
	vmware_cmd cmd;
	/* Read the mouse status */
	cmd.bx = 0;
	cmd.command = CMD_ABSPOINTER_STATUS;
	vmware_send(&cmd);
	
	/* Mouse status is in EAX */
	if (cmd.ax == 0xFFFF0000) {
		/* An error has occured, let's turn the device off and back on */
		mouse_off();
		mouse_absolute();
		return;
	}
	
	/* The status command returns a size we need to read, should be at least 4. */
	if ((cmd.ax & 0xFFFF) < 4) return;
	
	/* Read 4 bytes of mouse data */
	cmd.bx = 4;
	cmd.command = CMD_ABSPOINTER_DATA;
	vmware_send(&cmd);
	
	/* Mouse data is now stored in AX, BX, CX, and DX */
	int flags   = (cmd.ax & 0xFFFF0000) >> 16; /* Not important */
	int buttons = (cmd.ax & 0xFFFF); /* 0x10 = Right, 0x20 = Left, 0x08 = Middle */
	int x       = (cmd.bx); /* Both X and Y are scaled from 0 to 0xFFFF */
	int y       = (cmd.cx); /* You should map these somewhere to the actual resolution. */
	int z       = (int8_t)(cmd.dx); /* Z is a single signed byte indicating scroll direction. */
	
	/* TODO: Do something useful here with these values, such as providing them to userspace! */
}
</source>

== Message Channels ==

Advanced functionality of the backdoor employs message channels. We'll be using two different message channels: RPCI channels and TCLO channels. We'll use RPCI channels to send capability strings to VMware, and the TCLO channel to receive events.

Message channels have five basic operations: open, close, receive, send, ack.

<source lang="c">
#define MESSAGE_OPEN  0x00000000
#define MESSAGE_SEND  0x00010000
#define MESSAGE_RECV  0x00030000
#define MESSAGE_ACK   0x00050000 
#define MESSAGE_CLOSE 0x00060000

/* Open a message channel */
int open_msg_channel(uint32_t protocol) {
	vmware_cmd cmd;
	cmd.cx = CMD_MESSAGE | MESSAGE_OPEN;
	cmd.bx = protocol;
	vmware_send(&cmd);
	
	/* AX indicates status; 0x10000 is success */
	if ((cmd.ax & 0x10000) == 0) {
		/* Failed */
		return -1;
	}
	
	/* Channel is returned in upper two bytes of EDX */
	return cmd.dx >> 16;
}

/* Close a message channel */
void close_msg_channel(int channel) {
	vmware_cmd cmd;
	cmd.cx = CMD_MESSAGE | MESSAGE_CLOSE;
	cmd.bx = 0;
	cmd.dx = channel << 16; /* channel passed in high two bytes of EDX */
	vmware_send(&cmd);
}
</source>

To send and receive messages, we first send a command containing our requested operation and its size to the normal backdoor. We then follow this up with either a read or write to the high-bandwidth backdoor. In the receive case, we also send an acknowledgement command afterwards.

<source lang="c">
/* Send data to a message channel */
int send_msg(int channel, char * msg, size_t size) {
	/* Write the send size to the regular backdoor */
	vmware_cmd send = {0};
	send.cx = CMD_MESSAGE | MESSAGE_SEND;
	send.size = size;
	send.dx = channel << 16;
	vmware_send(&send);
	
	if (size == 0) return 0; /* Nothing more to do for empty messages */
	
	if (((send.cx >> 16) & 0x0081) != 0x0081) {
		/* This error indicates the response is not expected
		   to come from the correct source (the high bandwidth
		   backdoor) and we should bail. */
		return -1;
	}
	
	/* Write the data to the high-bandwidth backdoor */
	vmware_cmd data = {0};
	data.bx = 0x00010000; /* High-bandwidth commands are in EBX */
	data.cx = size; /* And ECX stores the size */
	data.dx = channel << 16; /* Channel still in EDX though */
	data.si = (uint32_t)msg; /* Probably needs adjustments for 64-bit guests */
	vmware_send_hb(&data);
	
	/* Confirm status code */
	if (!(cmd.bx & 0x00010000)) {
		/* Failed to send */
		return -1;
	}
	
	return 0; /* Success */
}

/* Receive data from message channel */
int recv_msg(int channel, char * buf, size_t bufsize) {
	int size;
	/* Request the recv data size */
	vmware_cmd recv = {0};
	recv.cx = CMD_MESSAGE | MESSAGE_RECV;
	recv.dx = channel << 16;
	vmware_send(&recv);
	
	size = cmd.bx; /* Data size from bx */
	
	if (size == 0) return 0; /* Nothing more to do for empty message */
	
	if (((cmd.cx >> 16)  & 0x0083) != 0x0083) {
		/* The VM is trying to send us data through the wrong mechanism */
		return -1;
	}

	if (size > bufsize) {
		/* There is insufficient space in our buffer to fit this message */
		return -1;
	}
	
	vmware_cmd data = {0};
	data.bx = 0x00010000; /* Same BX value as sending; just means "high bandwidth data" */
	data.cx = size;
	data.dx = channel << 16;
	data.di = (uint32_t)buf; /* For reads, DI */
	vmware_get_hb(&data);
	
	/* Check status bit */
	if (!(data.bx & 0x00010000)) {
		/* Failed to receive */
		return -1;
	}
	
	vmware_cmd ack = {0};
	ack.cx = CMD_MESSAGE | MESSAGE_ACK;
	ack.bx = 0x0001;
	ack.dx = channel << 16;
	vmware_send(&ack);
	
	return size;
}
</source>

With these primitives, we can implement the RPCI and TCLO channels.

=== RPCI ===

RPCI is used to send strings containing capability information and other data to the VM. These are a simple one-shot messages.

<source lang="c">
#define MSG_PROTO_RPCI 0x49435052 /* R P C I */
int rpci_string(char * request) {
	int channel = open_msg_channel(MSG_PROTO_RPCI);
	if (channel < 0) return channel; /* Return error code */
	
	size_t size = strlen(request) + 1; /* RPCI messages include nul terminator */
	send_msg(channel, request, size); /* Send RPCI message */
	
	/* The VM sends a response that should contain a success code */
	char buf[16]; /* Response buffer */
	int recv_size = recv_msg(channel, buf, sizeof(buf));

	close_msg_channel(channel); /* Close the channel now that we are done with it */
	
	if (recv_size < 0) return recv_size; /* Return error code */
	
	return 0; /* 0 for success */
}
</source>

=== TCLO ===

The TCLO channel is used to receive and acknowledge events. We'll use it later to receive display resolution information. There is only one TCLO channel available, and we'll keep it open through multiple operations.

<source lang="c">
#define MSG_PROTO_TCLO 0x4f4c4354 /* T C L O */
int open_tclo_channel(void) {
	static int tclo = -1;
	if (tclo != -1) {
		/* If the TCLO channel was already open, close it so we can reopen */
		close_msg_channel(tclo);
	}
	tclo = open_msg_channel(MSG_PROTO_TCLO);
	return tclo;
}
</source>

To communicate with the TCLO channel, we first send an empty message indicating we are expecting an event message:

<source lang="c">
int tclo_query(void) {
	int tclo = open_tclo_channel();
	if (tclo < 0) return -1; /* Failed to open channel */

	char buf[256]; /* Should be big enough */

	if (send_msg(tclo, buf, 0) < 0) return -1; /* Failed to send empty message */
	
	while (1) {
		int size = recv_msg(tclo, buf, 256);
		if (i < 0) return -1; /* Receive error */
		
		if (i == 0) {
			/* Empty response means nothing available */
			/* This is a good time to send capability reports. */
			/* This is also a good time to yield! */
			if (send_msg(tclo, buf, 0) < 0) return -1; /* Failed to send blank response */
		} else {
			/* TODO: Check message types */
		}
	}
}
</source>

Messages received from the TCLO channel are human-readable text. We'll use prefix matching to check different commands. I'll assume you have a function called "startswith" which can be implemented with strncmp.

Two important commands are "reset" and "ping". This is where we're going to start to see some very specific responses VMware is expecting. TCLO replies do not include nul bytes. We'll write a quick macro to send replies so we don't have to repeat ourselves:

<source lang="c">
#define TCLO_REPLY(x) send_msg(tclo, x, sizeof(x))
if (startswith(buf, "reset")) {
	if (TCLO_REPLY("OK ATR toolbox") < 0) return -1; /* Failed to send */
} else if (startswith(buf, "ping")) {
	if (TCLO_REPLY("OK ") < 0) return -1; /* Failed to send */
} else {
	if (TCLO_REPLY("ERROR Unknown command") < 0) return -1; /* Failed to send */
}
</source>

== Automatically Setting the Display Resolution ==

There are some esoteric RPCI strings we need to send to indicate we are capable of receiving display resolution data. I have not tested whether some of these may be omitted, but the sample code from VMware sends all of them:

<source lang="c">
if (rpci_string("tools.capability.resolution_set 1") < 0) { return 1; }
if (rpci_string("tools.capability.resolution_server toolbox 1") < 0) { return 1; }
if (rpci_string("tools.capability.display_topology_set 1") < 0) { return 1; }
if (rpci_string("tools.capability.color_depth_set 1") < 0) { return 1; }
if (rpci_string("tools.capability.resolution_min 0 0") < 0) { return 1; }
if (rpci_string("tools.capability.unity 1") < 0) { return 1; }
</source>

As for where to send these, the VMware sample code sends them once on startup and then again when there is nothing in the TCLO queue after receiving a "Capabilities_Register" event from TCLO. I have found only the latter is necessary for things to work.

<source lang="c">
int send_capabilities = 0;
...
/* This is a good time to send capability reports. */
if (send_capabilities) {
	if (rpci_string("tools.capability.resolution_set 1") < 0) { return 1; }
	if (rpci_string("tools.capability.resolution_server toolbox 1") < 0) { return 1; }
	if (rpci_string("tools.capability.display_topology_set 1") < 0) { return 1; }
	if (rpci_string("tools.capability.color_depth_set 1") < 0) { return 1; }
	if (rpci_string("tools.capability.resolution_min 0 0") < 0) { return 1; }
	if (rpci_string("tools.capability.unity 1") < 0) { return 1; }
	send_capabilities = 0;
} else {
	/* This is also a good time to yield! */
}
</source>

<source lang="c">
...
} else if (startswith(buf, "Capabilities_Register")) {
	if (TCLO_REPLY("OK ") < 0) return -1;
	send_capabilities = 1;
} else {
...
</source>

Once we have registered our ability to handle resolution events, we'll also see "Resolution_Set" events from the TCLO channel:

<source lang="c">
...
} else if (startswith(buf, "Resolution_Set")) {
	/* Split the arguments */
	char * x_str = &buf[15]; /* one space after Resolution_Set */
	char * y_str = strstr(x_str, " "); *y_str = '\0'; y_str++;
	
	int x = atoi(x_str);
	int y = atoi(y_str);
	
	set_display_resolution(x,y);

	if (TCLO_REPLY("OK ") < 0) return -1;
	/* Now may also be a good time to yield for an extended period of time
	   as it's unlikely the user will be resizing the guest again for a while */
} else {
...
</source>

And that's it - we have everything we need to set the display resolution to optimally fill the VMware window.

Do note, however, that unlike the Bochs display device, the VMware one uses strides that may not match width*bpp, which is more apparent at arbitrary window sizes.

[[Category:Emulators]]
````

<a id="w004330"></a>

## W004330 — User:Klange/VMware tools

Bron-ID: W004330
Namespace: User (2)
Revisie: 23018 | Datum: 2018-09-29T02:04:33Z
Bron: https://wiki.osdev.org/User:Klange/VMware_tools
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKlange%2FVMware_tools&oldid=23018
Doorverwijzing: VMware tools

````text
#REDIRECT [[VMware tools]]
````

<a id="w004331"></a>

## W004331 — User:Lkurusa/HiFive1 Bare Bones

Bron-ID: W004331
Namespace: User (2)
Revisie: 23039 | Datum: 2018-10-19T11:49:55Z
Bron: https://wiki.osdev.org/User:Lkurusa/HiFive1_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALkurusa%2FHiFive1_Bare_Bones&oldid=23039
Doorverwijzing: HiFive-1 Bare Bones

````text
#REDIRECT [[HiFive-1 Bare Bones]]
````

<a id="w004332"></a>

## W004332 — QEMU RISC-V Virt Bare Bones

Bron-ID: W004332
Namespace: Main (0)
Revisie: 30478 | Datum: 2026-04-12T16:51:34Z
Bron: https://wiki.osdev.org/QEMU_RISC-V_Virt_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=QEMU_RISC-V_Virt_Bare_Bones&oldid=30478

````text
{{ stub }}

RISC-V is a instruction set architecture, fully open source. The ISA has a bunch of extensions, in this tutorial we will assume that imad are available.

At our disposal we will have a generic board we will program for: virt. This board is available for [[QEMU]].

You can obtain a [https://mirrors.edge.kernel.org/pub/tools/crosstool/files/bin/x86_64/10.1.0 RISC-V GCC toolchain] on kernel.org; Designing an operating system for this architecture is not complicated and is pretty straight forward:

== kernel.c ==

The thing we will make here is a simple serial port output/reader.

<syntaxhighlight lang="c">
#include <stdint.h>
#include <stddef.h>
 
unsigned char * uart = (unsigned char *)0x10000000; 
void putchar(char c) {
	*uart = c;
	return;
}
 
void print(const char * str) {
	while(*str != '\0') {
		putchar(*str);
		str++;
	}
	return;
}
 
void kmain(void) {
	print("Hello world!\r\n");
	while(1) {
		// Read input from the UART
		putchar(*uart);
	}
	return;
}
</syntaxhighlight>

This will print a simple "Hello world!" and a newline following, after that it will echo any input given to the serial port.

== entry.S ==
Be wary we are just setting up stack and jumping into kernel. It's the kernel's job to set up IRQs and other system stuff

<syntaxhighlight lang="asm">
.section .init

.option norvc

.type start, @function
.global start
start:
	.cfi_startproc
	
.option push
.option norelax
	la gp, global_pointer
.option pop
	
	/* Reset satp */
	csrw satp, zero
	
	/* Setup stack */
	la sp, stack_top
	
	/* Clear the BSS section */
	la t5, bss_start
	la t6, bss_end
bss_clear:
	sd zero, (t5)
	addi t5, t5, 8
	bltu t5, t6, bss_clear
	
	la t0, kmain
	csrw mepc, t0
	
	/* Jump to kernel! */
	tail kmain
	
	.cfi_endproc

.end

</syntaxhighlight>

Take in account that you need to implement a type of locking if you want to support [[SMP]], and implement a trampoline or a jail for the CPUs. Be wary that we haven't enabled interrupts just yet and any return from <code>kmain</code> will result in invalid opcodes being executed.

== linker.ld ==

We need to specify where our kernel will be loaded at. For this example we will use the start of the RAM as our load address.

<syntaxhighlight lang="c">
ENTRY(start);
 
. = 0x80000000;
 
SECTIONS {
	/* Include entry point at start of binary */
	.text : ALIGN(4K) {
		*(.init);
		*(.text);
	}
	.bss : ALIGN(4K) {
		PROVIDE(bss_start = .);
		*(.bss);
		. += 4096;
		PROVIDE(stack_top = .);
		. += 4096;
		PROVIDE(global_pointer = .);
		PROVIDE(bss_end = .);
	}
	.rodata : ALIGN(4K) {
		*(.rodata);
	}
	.data : ALIGN(4K) {
		*(.data);
	}
}
</syntaxhighlight>

== Final ==

The process is pretty straightforward:
<syntaxhighlight lang="shell-session">
$ riscv64-elf-gcc -Wall -Wextra -c -mcmodel=medany kernel.c -o kernel.o -ffreestanding
$ riscv64-elf-as -c entry.S -o entry.o
$ riscv64-elf-ld -T linker.ld -lgcc -nostdlib kernel.o entry.o -o kernel.elf 
</syntaxhighlight>

Once this is done, the kernel can be run with the following:
<syntaxhighlight lang="shell-session">
$ qemu-system-riscv64 -machine virt -bios none -kernel kernel.elf -serial mon:stdio
</syntaxhighlight>

You will promptly see a terminal with the "Hello world!" and you can "write stuff" to it. A serial terminal is very "boring", but you still have a lot at your disposal, you have a PLIC, a CLIC and a bunch of opportunities to learn PCI internals.

== Going further ==
* Store input somewhere for later processing (i.e: "echo" command)
* Implement SMP support, for now our basic kernel will crash on any multicore system
* [[QEMU]] will give FDT in register a1. Use libfdt to parse the tree to avoid nightmares
* Implement a heap
* Enable Sv39 [[Paging|paging]]
* [[SMP]] support
* Manage [[IRQ|IRQs]]
* You have a PIO and a MMIO window at your disposal, use them wisely, you will also have to face the PCIe ECAM, for your sanity: Do not implement bridges

== See also ==
=== Articles ===
* [[PCI Express]]
* [[HiFive-1 Bare Bones]]
* [[QEMU RISC-V Virt Meaty Skeleton]]

=== External Links ===
* [https://msyksphinz-self.github.io/riscv-isadoc/html/rvi.html RISC-V instruction set]
* [https://osblog.stephenmarz.com/ Small Bare Metal RISCV OS in asm and Rust]
* [https://marz.utk.edu/my-courses/cosc562/riscv/ Page of above with asm code setting up different kernel stack for each hart]
* [https://github.com/cksystemsteaching/selfie Selfie: a minimalist self hosting compiler, riscv64 os, and simulator]
* [https://github.com/cksystemsteaching/selfie/tree/main/theses Selfie page includes 2 well written bare-metal theses]
* [https://github.com/phoenix-rtos Complete open source Microkernel for riscv64 (also arm7, ia32) plus support code]
* [https://github.com/mit-pdos/xv6-riscv XV6: a reimplementation of original Unix for teaching riscv64 OS development]
* [https://github.com/michaelengel/xv6-vf2 Port of XV6 to execute on VisionFive 2 SoC]
* [https://github.com/lupyuen/nuttx-star64 Wonderfully clear and detailed base level port how2: NuttX RTOS to Star64 SoC]

[[Category:Bare bones tutorials|RISC-V bare bones]]
[[Category:RISC-V]]
[[Category:QEMU]]
````

<a id="w004333"></a>

## W004333 — User talk:Glauxosdever/Higher Half x86 Bare Bones

Bron-ID: W004333
Namespace: User talk (3)
Revisie: 23055 | Datum: 2018-10-27T19:37:36Z
Bron: https://wiki.osdev.org/User_talk:Glauxosdever/Higher_Half_x86_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AGlauxosdever%2FHigher_Half_x86_Bare_Bones&oldid=23055

````text
This article is a replacement for [[Higher Half x86 Bare Bones]], which was broken and not exactly compatible with [[Bare Bones]] last time I checked. Please audit both [[Higher Half x86 Bare Bones]] and this article and merge them. [[User:Glauxosdever|Glauxosdever]] 14:36, 27 October 2018 (CDT)
````

<a id="w004336"></a>

## W004336 — User:Glauxosdever/Higher Half x86 Bare Bones

Bron-ID: W004336
Namespace: User (2)
Revisie: 23061 | Datum: 2018-10-27T19:40:25Z
Bron: https://wiki.osdev.org/User:Glauxosdever/Higher_Half_x86_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGlauxosdever%2FHigher_Half_x86_Bare_Bones&oldid=23061
Doorverwijzing: Higher Half x86 Bare Bones

````text
#REDIRECT [[Higher Half x86 Bare Bones]]
````

<a id="w004337"></a>

## W004337 — WSL

Bron-ID: W004337
Namespace: Main (0)
Revisie: 23069 | Datum: 2018-11-04T16:00:03Z
Bron: https://wiki.osdev.org/WSL
Revisie-URL: https://wiki.osdev.org/index.php?title=WSL&oldid=23069
Doorverwijzing: Windows Subsystem for Linux

````text
#REDIRECT[[Windows Subsystem for Linux]]
````

<a id="w004338"></a>

## W004338 — Category:GRUB

Bron-ID: W004338
Namespace: Category (14)
Revisie: 23076 | Datum: 2018-11-10T20:54:03Z
Bron: https://wiki.osdev.org/Category:GRUB
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AGRUB&oldid=23076

````text
GRUB, or the GRand Unified Bootloader, is the GNU bootloader, and default bootloader for most common Linux distributions. There is a [[GRUB Legacy|legacy version]], and the more modern [[GRUB 2]].
[[Category:Bootloaders]]
````

<a id="w004339"></a>

## W004339 — File:Gdt bits fixed.png

Bron-ID: W004339
Namespace: File (6)
Revisie: 23082 | Datum: 2018-11-14T22:08:27Z
Bron: https://wiki.osdev.org/File:Gdt_bits_fixed.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AGdt_bits_fixed.png&oldid=23082

````text
Fixed version of the Gdt_bits.png file (S instead of 1)
````

<a id="w004340"></a>

## W004340 — User:Gravaera/VMSAv8-64 Overview

Bron-ID: W004340
Namespace: User (2)
Revisie: 23095 | Datum: 2018-11-26T05:00:44Z
Bron: https://wiki.osdev.org/User:Gravaera/VMSAv8-64_Overview
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FVMSAv8-64_Overview&oldid=23095

````text
This page seeks to summarize the ARMv8 Virtual Memory System Architecture for the AArch64 execution state. ARMv8 has a slightly different MMU scheme for AArch32 execution state. This article is not meant to be an exhaustive overview, but should serve as a quick refresher for a person who has already read the ARMv8 VMSAv8-64 manual section.

* VMSAv8 currently only supports 56 bits of virtual addresses, making the top 8 bits of input addresses to the MMU "reserved".
* ARMv8 also smooths out the translation scheme, making the page tables resemble other architectures by removing the notion of sections and super-sections, only using small and large pages.
* However, ARMv8 also introduces multiple "translation granules" which enable the use of 16KiB and 64KiB pages and page tables.

= Canonical Addresses =
Canonical addresses are supported for Secure mode (EL3, EL1 and EL0) and Non-secure mode (EL2, EL1 and EL0). They must be explicitly enabled by setting xTCR_ELx.TBIn ("Top byte ignored"). The way they work on ARMv8 is that the TBI bit  determines whether or not the processor will ignore the top 8 bits of input addresses to the MMU. If xTCR_ELx.TBI is not enabled, those bits in the input address will be taken to be valid bits, and will trigger a translation abort. ARM calls this feature "Address Tagging".

If xTCR_ELx.TBI is enabled, those bits in the input address will be ignored by the MMU. Moreover:

== In EL3 ==
Canonical addresses are enabled using <tt>TCR_EL3.TBI</tt>. The specific effect of TBI is not stated for EL3.

== In EL2 ==
Enable using TCR_EL2.TBI. The specific effect of TBI is not stated for EL2.
Verbatim: "If the controlling TBI bit for the address being loaded into the PC is set to 1, then bits[63:56] of the PC are forced to be 0x00."

== In EL1 and EL0 ==
Enable using TCR_EL1.TBI.
Verbatim: "If the controlling TBIn bit for the address being loaded into the PC is set to 1, then bits[63:56] of the PC are forced to be a sign extension of bit[55] of that address."

In other words, it's not possible to use the canonical high address range of a 64-bit vaddrspace on AArch64, but you can link your hypervisor or EL3 monitor binary image to a canonically high vaddr (with the top bits all set to 1) and run it on the processor regardless.

= Translation regime terminology =
This section is a minor note about the way ARM talks about multi-stage translation. Translation generally has 1 or 2 stages. When there is only one stage of translation that stage is called stage1 by ARM. When there are 2 stages of translation, the stage that translates from VA to IPA is called stage1 and the stage that translates from IPA to PA is called stage2. It would have been nice if they always just called the stage that maps from VA/IPA to PA stage2 because it would have made it clearer which stage they were referring to.

Second, the term "flat mapping" is used to refer to a passthrough from one stage to another when a stage is disabled. So for example, if stage2 is disabled on a hypervisor build, then the IPA output by stage1 will be returned unmodified as the PA from the (inexistent) stage2. ARM calls this "flat mapping".

= Address Generation capability =

== Virtual addresses ==
The ARMv8 processor has a maximum VA size of 48 bits, from 0x00000000_00000000 to 0x0000FFFF_FFFFFFFF. The architecture gives two vaddrspace subranges (but only) for EL0 and EL1. Other exception levels have one vaddrspace range setup. EL0 and EL1's vaddrspace ranges are:
* The bottom vaddrspace range goes from 0x0 - 0x00000000_00000000.
* The top vaddrspace range goes from 0xFFFF0000_00000000 - 0xFFFFFFFF_FFFFFFFF.

Each translation regime and stage which uses an MMU and takes a VA as input can be configured to support fewer than 48 bits of vaddrspace.

== Intermediate Physical addresses ==
The ARMv8 processor can generate a maximum of 48 bits of IPA. The output address width from each stage of translation (stage1=IPA, stage2=PA) must be configured in the TCR_ELx.[I]PS register that controls that stage of translation.

== Physical addresses ==
The PA is issued to the mmemory subsystem. Architecturally ARMv8 have a maximum physical address width of 48 bits but the implementation may support fewer than 48 bits. Check the implemented physical address width for your processor implementation using ID_AA64MMFR0_EL1.PARange.

= Translation table format =
All translation tables in VMSAv8-64 execution state use the long descriptor table format from ARMv7-LPAE (Long Physical Address Extension).
VMSAv8-32 continues to use both the short and long descriptor table formats.

= Enabling the MMU =

== Enabling the MMU for EL3 ==
Use SCTLR_EL3.{EE, M} to enable and TCR_EL3 to control. Use TTBR0_EL3 to set/change the top level table.

== Enabling the MMU for EL2 ==
Use SCTLR_EL2.{EE, M} to enable and TCR_EL2 to control. Use TTBR0_EL2 to set/change the top level table.

== Enabling the MMU for stage 2 for EL1 and EL0 guests ==
Use SCTLR_EL2.EE and HCR_EL2.VM to enable. Use VTCR_EL2 to control. Use VTTBR_EL2 to set/change the top level table.

== Enabling the MMU for EL1 kernels ==
Use SCTLR_EL1.{EE, M} to enable. Use TCR_EL1 to control. Use TTBR[0/1]_EL1 to set/change the top level table.
````

<a id="w004341"></a>

## W004341 — User talk:Floppout76

Bron-ID: W004341
Namespace: User talk (3)
Revisie: 23137 | Datum: 2018-12-15T14:28:48Z
Bron: https://wiki.osdev.org/User_talk:Floppout76
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AFloppout76&oldid=23137

````text
== Warning: Please don't focus on irrelevant aspects on the projects page ==

Hi, wiki admin here, were you [[User_talk:Updater3|Updater3]] who I have given a warning about this before? Focusing on just floppies is weird because they're irrelevant, yes, I've seen your bugs talking about motherboard support but that's also not too important. There's a lot more important aspects of operating systems to focus on, like whether they're installable, whether an installation can be upgraded to a newer version, whether there's networking, whether the OS is self-hosting, etc. Likewise the processor architecture and possibly the memory requirements could matter. If the boot methods were important, it would be useful to mark the available ones like netboot, cdrom, floppy, disk image, etc. Although that information should be limited to the really relevant parts and be information dense, so it doesn't distract from the rest of the entry. I also reverted your other changes to the page, which were fine editorial changes. Feel free to make those again. If you'd like to make major changes to the page, please discuss it on the talk page or the forums and get some consensus. --[[User:Sortie|Sortie]] 08:21, 15 December 2018 (CST)
````

<a id="w004342"></a>

## W004342 — File:Haiku.png

Bron-ID: W004342
Namespace: File (6)
Revisie: 23139 | Datum: 2018-12-21T21:33:26Z
Bron: https://wiki.osdev.org/File:Haiku.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AHaiku.png&oldid=23139

````text
Some applications running on Haiku.
````

<a id="w004343"></a>

## W004343 — User:Waddlesplash

Bron-ID: W004343
Namespace: User (2)
Revisie: 23145 | Datum: 2018-12-21T22:09:16Z
Bron: https://wiki.osdev.org/User:Waddlesplash
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AWaddlesplash&oldid=23145

````text
I'm one of the developers of [https://www.haiku-os.org/ Haiku].

You can also find me on [https://github.com/waddlesplash/ GitHub] and [https://twitter.com/waddlesplash Twitter].
````

<a id="w004344"></a>

## W004344 — File:Diyos.png

Bron-ID: W004344
Namespace: File (6)
Revisie: 23148 | Datum: 2018-12-21T23:41:03Z
Bron: https://wiki.osdev.org/File:Diyos.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ADiyos.png&oldid=23148

````text

````

<a id="w004345"></a>

## W004345 — User talk:Electrodeyt

Bron-ID: W004345
Namespace: User talk (3)
Revisie: 23157 | Datum: 2018-12-27T22:50:55Z
Bron: https://wiki.osdev.org/User_talk:Electrodeyt
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AElectrodeyt&oldid=23157

````text
Hi, and thank you for [[Special:Contributions/Electrodeyt|your contributions]] to the OsDev wiki. Your recent talk page comments  were not added to the bottom of the page. New discussion page messages and topics should always be added to the bottom. Your message has been moved. In the future you can use the "Add topic" link in the top right. Thank you. [[User:Bellezzasolo|Bellezzasolo]] 16:50, 27 December 2018 (CST)
````

<a id="w004346"></a>

## W004346 — Tail Recursion and Tail Call Optimization

Bron-ID: W004346
Namespace: Main (0)
Revisie: 30315 | Datum: 2026-03-17T19:01:34Z
Bron: https://wiki.osdev.org/Tail_Recursion_and_Tail_Call_Optimization
Revisie-URL: https://wiki.osdev.org/index.php?title=Tail_Recursion_and_Tail_Call_Optimization&oldid=30315

````text
== What is a tail call? ==
Informally, a tail call is when a function calls another function, then returns immediately. In assembly, it's when a <code>CALL</code> and a <code>RET</code> instruction are found right next to each other. For example, in the following code:

<syntaxhighlight lang="asm">
myFunction:
    CALL foo
    CALL bar
    RET
</syntaxhighlight>

<code>CALL bar</code> is a tail call, but <code>CALL foo</code> is not.

It's important that there is no operation between <code>CALL bar</code> and <code>RET</code>. Consider the following example:

<syntaxhighlight lang="c">
int foo(x){
  return bar(x) * 2;
}
</syntaxhighlight>

Even though this looks like a tail call, the caller has to modify the return value after the <code>CALL</code> operation. As such, it's not actually a tail call. In assembly:

<syntaxhighlight lang="asm">
    CALL bar
    SHL EAX, 1
    RET
</syntaxhighlight>

However the following is a tail call:

<syntaxhighlight lang="c">
int foo(x){
  return bar(x * 2);
}
</syntaxhighlight>

In assembly:

<syntaxhighlight lang="asm">
    SHL EAX, 1
    CALL bar
    RET
</syntaxhighlight>

As you can see, there is no operation between <code>CALL</code> and <code>RET</code> this time.

== Optimizing a tail call ==

A tail call can be optimized by simply replacing <code>CALL</code> and <code>RET</code> with <code>JMP</code>. So looking at this example, one more time:

<syntaxhighlight lang="c">
int foo(x){
  return bar(x * 2);
}
</syntaxhighlight>

In assembly, no tail call optimization:

<syntaxhighlight lang="asm">
    SHL EAX, 1
    CALL bar
    RET
</syntaxhighlight>

In assembly, with tail call optimization:

<syntaxhighlight lang="asm">
    SHL EAX, 1
    JMP bar
</syntaxhighlight>

This has a minor speed up as you no longer have to save and restore the instruction pointer. The callee (in this case, <code>bar</code>) will return all the way back to the caller of this function.

It doesn't have much of an impact on a function like this, but it can have immense impact on recursive calls. A recursive call that takes 1000 recursions will have to save and restore the instruction pointer 1001 times without tail call optimization, but only once with it.

== Epilogues and Prologues ==
In the above examples, it is assumed that functions don't have a prologue or epilogue which is highly impractical. As such a proper implementation of tail call optimization is a lot trickier than shown here. This is only a rudimentary example.

[[Category:Programming]]
[[Category:Compiler Development]]
````

<a id="w004347"></a>

## W004347 — Talk:Pascal Bare Bones

Bron-ID: W004347
Namespace: Talk (1)
Revisie: 23172 | Datum: 2019-01-05T10:09:23Z
Bron: https://wiki.osdev.org/Talk:Pascal_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APascal_Bare_Bones&oldid=23172

````text
There was funny fact about crossbinutils at my system, those files was lied at /usr/bin/bin, so when I've moved them to /usr/bin linker began work! And also there was a bug with kwritedword function.
````

<a id="w004348"></a>

## W004348 — User:Johnburger/OS/Design

Bron-ID: W004348
Namespace: User (2)
Revisie: 23205 | Datum: 2019-01-16T02:43:46Z
Bron: https://wiki.osdev.org/User:Johnburger/OS/Design
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FOS%2FDesign&oldid=23205

````text
An Operating System (OS) is different things to different people. Some think of it as the pretty windowing interface. Others think of it as a sophisticated work tool. Still others can't even distinguish it from the applications that they run.

To an OS developer, however, the OS is the implementation of something that tames the hardware of the computer; controls it and distributes it for the use of the applications that it will deign to allow run.

__TOC__

To me, an Operating System is a hierarchy of layers:

* At the core is the CPU and RAM: the processor and available storage. The architecture defines the way the Operating System can possibly operate - for example, without a Memory Management Unit there can be no Paging. The resources provided here are the fundamental building blocks.
* Next is the peripheral hardware. The OS needs to provide software that controls these - and share their usage among all processes.
* Building on the peripherals are the services that use them: abstractions to handle their complexity and "common up" differences between different versions of the hardware. For example, both a hard drive and USB memory stick provide raw storage, while a File System layers on the concepts of directories and files.
* Finally, applications use these services, and (usually indirectly) the hardware they rely on, to provide features to the end user. The OS doesn't provide many applications - indeed, one could argue it provides none.

A more specific and detailed breakdown of these separate layers is provided below.

== Processor Resources ==

=== Central Processing Unit(s) ===

==== Task Context ====

=== Floating Point Unit(s) ===

=== Address Spaces ===

==== Physical ====

==== Logical ====

==== Input/Output Ports ====

=== Segments ===

==== Global ====

==== Local ====

=== Random Access Memory (RAM) ===

=== Interrupts ===

==== Exceptions ====

==== Interrupts ====

=== Direct Memory Access (DMA) ===

== Devices ==

=== Timer(s) ===

=== Real-Time Clock ===

=== Buses ===

==== Peripheral Component Interconnect (PCI) ====

===== Bridges =====

==== Universal Serial Bus ====

===== Hubs =====

===== Keyboard =====

===== Mouse =====

===== Serial Port =====

===== Storage =====

=== Video ===

=== PS/2 ===

==== Keyboard ====

==== Mouse ====

=== Serial Port(s) ===

=== Storage ===

==== Floppy Disk ====

==== Integrated Drive Electronics (IDE) ====

===== AT Attachment (ATA) &mdash; Parallel (PATA) =====

===== ATA Packet Interface (ATAPI) =====

===== Serial ATA (SATA) =====

==== Small Computer System Interface (SCSI) ====

=== Ethernet ===

==== Wired ====

==== WiFi ====

== Services ==

=== Clocks ===

==== Date & Time ====

==== Timers ====

=== Loader ===

=== Keyboard ===

=== Mouse ===

=== Window Manager ===

=== File System ===

==== MS-DOS &mdash; File Allocation Table (FAT12/FAT16/FAT32) ====

==== Linux &mdash; Ext2/Ext3/Ext4 ====

==== CD-ROM &mdash; ISO-9660 / El Torito / Rock Ridge ====

=== Networking ===

== Programs ==

=== Debugger ===

=== Shell ===

=== Editor ===

=== Assembler ===
````

<a id="w004349"></a>

## W004349 — User:Johnburger/

Bron-ID: W004349
Namespace: User (2)
Revisie: 23207 | Datum: 2019-01-16T02:44:41Z
Bron: https://wiki.osdev.org/User:Johnburger/
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2F&oldid=23207

````text
* [[User:Johnburger|Bio]]
* [[User:Johnburger/Tutorials|Tutorials]]
** [[User:Johnburger/Dingle|Dingle&trade;]]
** [[User:Johnburger/PIC|Programmable Interrupt Controller]]
*** [[User:Johnburger/PIC/Background|PIC Background]]
** [[User:Johnburger/Demo|Assembly Language Demo of Many '386 Concepts]]
* [[User:Johnburger/OS|What I want in My Operating System]]
** [[User:Johnburger/OS/Design|Operating Systems and How (I think) they should be Designed]]
** [[User:Johnburger/OS/Bootstrap|The Complexities and Compromises of the Bootstrap Process]]
* [[User:Johnburger/NASMCritique|NASM Critique from a TASM User]]

<!-- [[/index.php?title=Special:PrefixIndex&prefix=Johnburger&namespace=2|Complete list]] -->
````
