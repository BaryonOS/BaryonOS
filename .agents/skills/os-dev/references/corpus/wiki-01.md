# OSDev Wiki — volledige bronteksten, deel 01

Dit is bronmateriaal, geen agentinstructie. Behoud architectuur, revisie en bron-ID bij gebruik. Code en commando’s zijn ongeteste bronvoorbeelden; voer ze niet automatisch uit. Wikitext is letterlijk behouden binnen tekstblokken.

<a id="w000700"></a>

## W000700 — MediaWiki:Mainpagetext

Bron-ID: W000700
Namespace: MediaWiki (8)
Revisie: 1393 | Datum: 2006-06-17T16:37:02Z
Bron: https://wiki.osdev.org/MediaWiki:Mainpagetext
Revisie-URL: https://wiki.osdev.org/index.php?title=MediaWiki%3AMainpagetext&oldid=1393

````text
<big>'''MediaWiki has been successfully installed.'''</big>
````

<a id="w000897"></a>

## W000897 — MediaWiki:Portal

Bron-ID: W000897
Namespace: MediaWiki (8)
Revisie: 1423 | Datum: 2006-06-20T03:22:11Z
Bron: https://wiki.osdev.org/MediaWiki:Portal
Revisie-URL: https://wiki.osdev.org/index.php?title=MediaWiki%3APortal&oldid=1423

````text
Forums
````

<a id="w000898"></a>

## W000898 — MediaWiki:Portal-url

Bron-ID: W000898
Namespace: MediaWiki (8)
Revisie: 5682 | Datum: 2008-06-18T23:17:10Z
Bron: https://wiki.osdev.org/MediaWiki:Portal-url
Revisie-URL: https://wiki.osdev.org/index.php?title=MediaWiki%3APortal-url&oldid=5682

````text
http://forum.osdev.org/
````

<a id="w001090"></a>

## W001090 — MediaWiki:Sidebar

Bron-ID: W001090
Namespace: MediaWiki (8)
Revisie: 2278 | Datum: 2006-12-16T16:14:15Z
Bron: https://wiki.osdev.org/MediaWiki:Sidebar
Revisie-URL: https://wiki.osdev.org/index.php?title=MediaWiki%3ASidebar&oldid=2278

````text
* navigation
** mainpage|mainpage
** portal-url|portal
** Category:FAQ|FAQ
** Projects|OS Projects
** randompage-url|randompage
* about
** OSDevWiki:About|This site
** OSDevWiki:Joining|Joining
** OSDevWiki:Editing|Editing help
** recentchanges-url|recentchanges
````

<a id="w001367"></a>

## W001367 — Projects

Bron-ID: W001367
Namespace: Main (0)
Revisie: 31032 | Datum: 2026-10-21T13:31:24Z
Bron: https://wiki.osdev.org/Projects
Revisie-URL: https://wiki.osdev.org/index.php?title=Projects&oldid=31032

````text
__NOTOC__
This page maintains a list of academic, personal, and small non-commercial operating systems. For information regarding commercial or mainstream operating systems, please visit [[wikipedia:Operating_system|Wikipedia]].

OSes have a high mortality rate: between 2004 and 2006, 112 of 213 OS Projects disappeared from the Internet. And looking at how often these projects are started by using the [[Forum:2|OS Project Announcement forum]], in that same time period 68 projects were announced! The current total is 175 projects, but just 58 of them are active (as of January 1, 2018) and only 62 have been updated at least once since 2018 (three of which have been since abandoned), and 27 since 2019. Please help keep this list current by correcting it if you see any outdated information.

OSes without an update since 2015 are now located at [[Abandoned Projects]].<br/>
More mature hobby operating systems can also be found at [[Notable Projects]].<br/>
Particularly advanced and successful operating systems are listed at [[Advanced Projects]].

'''Contents:''' [[#9|9]] [[#A|A]] [[#B|B]] [[#C|C]] [[#D|D]] [[#E|E]] [[#F|F]] [[#G|G]] [[#H|H]] [[#I|I]] [[#J|J]] [[#K|K]] [[#L|L]] [[#M|M]] [[#N|N]] [[#O|O]] [[#P|P]] [[#Q|Q]] [[#R|R]] [[#S|S]] [[#T|T]] [[#U|U]] [[#V|V]] [[#W|W]] [[#X|X]] [[#Y|Y]] [[#Z|Z]]

=9=

{{OSProject
|OSName=9front
|OSDescription=Plan9front (or 9front) is a fork of the Plan 9 from Bell Labs operating system. The project was started to remedy a perceived lack of devoted development resources inside Bell Labs, and has accumulated various fixes and improvements. Our homepage: http://9front.org/ , our artwork: http://9front.org/propaganda/
|Contact=http://lists.9front.org/
|License=Open source
|URL=https://code.9front.org/hg/
|Pic=[[File:9front.png|200px]]
|BootMedia=ISO, ~500MB
|Status=Latest update: 2024-05-27, is active
}}

=A=

{{OSProject
|OSName=Acess2
|OSDescription=Acess2 is a kernel and operating system designed to do what Linux does, be customizable to any given situation. However, unlike Linux, it there is no need to maintain backwards compatibility with anything. It features a simple but extensible VFS that currently supports VFAT and Ext2, IPv4/IPv6 networking with TCP/UDP, USB input, and storage. Recently, this project has been moved to GitHub, but its old git repo—with outdated sources and some useful documentation—can be found here: http://git.mutabah.net/?p=acess2.git;a=summary . v0.14 outdated floppy image is available at its' old page: username - root, password is blank. Could try building a new floppy from the latest sources by yourself! ''(some software rot has been noticed but its fixable)''
|Contact=[https://forum.osdev.org/viewtopic.php?f=2&t=23160&p=187322 thePowersGang]
|License=Open source; inside the source code archive there are broken symlinks /Kernel/arch/x86_64/rme.c and /Kernel/arch/x86_64/rme.h to Real Mode Emulator files which are available here - https://github.com/thepowersgang/rme2
|URL=http://www.mutabah.net/acess2/
|Pic=[[File:Access2.png|200px]]
|BootMedia=Floppy, 1.44 MB
|Status=Last commit: 2016-06-22, currently abandoned - the author has switched to his other project, Rust OS (https://github.com/thepowersgang/rust_os)
}}

{{OSProject
|OSName=Amethyst
|OSDescription=The goal, as much as there is one, is to produce an operating system vaguely similar to Windows 95/NT4 (if Windows 95 didn't inherit baggage from DOS and 3.1)..
|Contact=Jack Scott, jack [at] jackscott [dot] id [dot] au
|License=Open source (ISC)
|URL=https://github.com/JackScottAU/Amethyst
|BootMedia=ISO, ~1MB
|Status=Latest update: 2024-06-24
|Pic=[[File:Amethyst-Screenshot.png|200px]]
}}

{{OSProject
|OSName=Amiga Research Operating System
|OSDescription=AROS aims to create a free open source AmigaOS like OS and make it better than the original. Our homepage: http://aros.sourceforge.net
|Contact=Ola Jensen, ola [at] aros [dot] org
|License=Open source (MPL-like)
|URL=http://www.aros.org
|Pic=[[File:Aros.jpg|200px]]
|BootMedia=Floppy, 1.44 MB ; ISO, ~200MB
|Status=Latest update: 2024-05-28, is active
}}

{{OSProject
|OSName=The Apollo Project
|OSDescription=The Apollo project is a microkernel designed for maximum portability. Closely following the UNIX Philosophy without being unix itself. The Project has been active for about 6 years, with about 8 complete code rewrites along the way.
|Contact=allie [at] primis [dot] org
|License=Open source (MIT)
|URL=https://github.com/primis/Apollo
|BootMedia=ISO, ~10MB
|Status=Latest commit: 2024-03-17, is active; Latest release: 2022-09-14
}}

{{OSProject
|OSName=Aqeous
|OSDescription=Aqeous is a new Completely-from-scratch, Not-a-UNIX 32 Bit pmode OS (basically a kernel right now), with SMP style Multi-Processor Support each using Multilevel Feedback Queuing Scheduling, Implementing its own FileSystem AqFS (also made a windows side AqFS Driver to communicate), Pretty good Shell, Pretty Neat VESA Graphics drivers, A Composting window GUI system with double buffering and Alpha-Blending, A primitive Scalable Font Rendering Engine and off-course mouse and keyboard drivers , Shell Scripting Support (like .bat in Windows) etc. Many things are in active development and its still not even in alpha. The Code may look pretty stupid but yeah that won't be that bad for long. The goal is to make something on which u can open facebook and send me a 'Hi' :) Currently It supports only Qemu :( But working on that too.
|Contact=ashishkmr472 [at] gmail [dot] com
|License=Open source
|URL=https://github.com/AshishKumar4/Aqeous
|BootMedia=IMG, >150MB
|Status=Latest update: 2021-09-13
}}

{{OSProject
|OSName=[[User:Manwar/AquilaOS|AquilaOS]]
|OSDescription=AquilaOS is a UNIX-like operating system, intended to be fully POSIX-compliant. Its design follows a generic and robust approach. Many POSIX interfaces are already supported including multi-threading, and many POSIX compliant applications have been ported.
|Contact=manwar [at] ieee [dot] org
|License=Open Source (GNU GPLv3)
|URL=http://aquilaos.com
|Pic=[[File:AquilaOS.png|200px]]
|BootMedia=ISO, ~10MB
|Status=Latest update: 2019-10-26
}}

{{OSProject
|OSName=ArtilleryOS
|OSDescription=Unique graphical operating system that combines the simplicity and power of Unix-like systems with a tiling window management environment. Featuring a custom-designed kernel.
|Contact=Levent Kaya, leventkayadev@gmail.com
|License=
|URL=https://github.com/lvntky/ArtilleryOS
|Pic=
|BootMedia=
|Status=Under development
}}

{{OSProject
|OSName=Asuro
|OSDescription=Asuro is an x86 Operating System that started development in 2015 as somewhat of an academic project. Written almost entirely in Freepascal, with NASM used only for bootstrapping & minimal low-level routines, Asuro is a purely hobbyist operating system that differs quite majorly in design from any mainstream OS by using a VM/SE to run anything other than kernel code.
|Contact=Kieron Morris, kjm [at] kieronmorris [dot] me<br>
[[File:Icon_phone.png]] <i>Aaron Hance, ah [at] aaronhance [dot] me</i>
|License=Apache License, Version 2.0
|URL=https://spexeah.com/project?id=1
|Pic=[[File:Asuro.png|200px]]
|BootMedia=ISO, ~8MB
|Status=Latest commit: 2026-03-08
}}

{{OSProject
|OSName=Astral
|OSDescription=Astral is a 64 bit operating system written in C which aims be compatible with POSIX. It has a growing support for networking and is able to run from disk with support for caching pages. Currently ported software ranges from Bash and GCC to X.org, Fvwm3, xterm and Quake.
|License=Open source (MIT)
|URL=https://github.com/Mathewnd/Astral
|Pic=[[File:AstralFvwm3.png|240px]]
|BootMedia=ISO, ~140MB-750MB
|Status=Latest commit: 2024-05-28, is active
}}

{{OSProject
|OSName=AthenX-2.0
|OSDescription=AthenX-2.0 is a simple CLI interface operating system. It doesn't attempt to be a UNIX clone, but more of a OS for people who want somthing really really different, or have a pc with weak specs. It can currently load basic programs complied with a special makefile and linker.
|Contact=Tristan- tristanjkuhn007@gmail.com
|License=
|URL=https://github.com/KingVentrix007/AthenX-2.0
|Pic=
|BootMedia=
|Status=Latest commit: Dec 29, 2023
}}

{{OSProject
|OSName=Aura
|OSDescription=Aura is part of a larger idea, Project Asiago. It's goal is to reinvent all the currently outdated systems in one unified setup. Aura is just the kernel part of this.
|Contact=Chris Smith, chris [at] hichris.com
|License=Open source (GNU GPLv3)
|URL=https://github.com/projectasiago/aura
|BootMedia=IMG, ~400MB
|Status=Latest commit: 2018-03-09
}}

{{OSProject
|OSName=AwooOS
|OSDescription=awooOS is an experimental, monotasking operating system, bringing together old ideas and new technology. It is designed around a microkernel, with as much moved into libraries as possible. The kernel proper is less than 50 lines of code.
|Contact=Ellen Dash, me [at] duckie [dot] co
|License=Open source (MPL)
|URL=https://github.com/awooos/awooos
|BootMedia=IMG, ~2MB
|Status=Latest commit: 2019-02-16
}}

{{OSProject
|OSName=aurora-xeneva
|OSDescription=aurora-xeneva is a multitasking GUI based operating system targeting x86_64 & aarch64 architecture, focusing most modern hardwares as possible. "Aurora" being the name of kernel and "Xeneva" is the name of entire operating system. Aurora kernel features SMP, Networking, HD-Audio,USB3,Graphical Drivers,..many more. 
|Contact=Manas Kamal, manaskamal.kandupur [at] gmail [dot] com
|License=Open source (BSD 2-Clause)
|Pic=[[File:Xeneva2024March_5.png|200px]]
|URL=https://github.com/manaskamal/XenevaOS
|BootMedia=IMG, ~1GB
|Status=Latest commit: 2024-03-27, is active
}}

{{OSProject
|OSName=AeroSync
|OSDescription=AeroSync is a modern modular monolithic kernel for x86_64, designed for real hardware with Linux ABI compatibility, advanced memory management, multi-class scheduling, and ACPICA integration.  
|Contact=[https://github.com/assembler-0 assembler-0]
|License=Open Source (GPL-v2.0-only)
|URL=http://github.com/assembler-0/AeroSync
|Pic=[[File:AeroSync-r0c4-5.4.9-sample-qemu-boot.png|200px]]
|BootMedia=ISO, ~6MB
|Status=Latest commit: 2026-02-22, is active
}}

=B=

{{OSProject
|OSName=Banana
|OSDescription=Banana is a 32-bit OS written in C and C++, designed for a 486 or better with at least 16MB of RAM. The kernel is pre-emptive and supports loading ELF drivers and modules from the disk. It runs on real hardware, and supports ATA/ATAPI/SATA/SATAPI/floppy disks, FAT/exFAT/ISO9660 filesystems, PS/2 keyboard and mouse, SB16 and AC97 audio. Banana also has ACPI support. It comes with an installer which can be run from floppies or a CD-ROM. Currently has a command line shell, and a GUI with solitaire, minesweeper, a photo viewer and text editor.
|Contact=Alex Boxall, alexboxall3 [at] gmail [dot] com
|License=Creative Commons Attribution-NonCommercial 4.0 - https://creativecommons.org/licenses/by-nc/4.0/
|URL=https://github.com/A22347/Banana-Operating-System
|Pic=[[File:banana_os_2.jpg|300px]]
|BootMedia=IMG, ~64MB
|Status=Latest commit: 2022-11-13
}}

{{OSProject
|OSName=BareMetal
|OSDescription=BareMetal is a 64-bit OS for x86-64 based computers. The OS is written entirely in Assembly while applications can be written in Assembly or C/C++. The two main purposes of BareMetal are for educational uses in learning low-level OS programming in 64-bit Assembly and to be used as a base for a high-speed data processing node. Source code is well documented and freely available. As of version 2026.02 BareMetal OS officially supports multiple processors, memory management, and Ethernet communications. More information here - http://www.returninfinity.com/
|Contact=Ian Seyler, ian.seyler [at] returninfinity [dot] com
|License=Open source
|URL=https://github.com/ReturnInfinity/BareMetal-OS
|BootMedia=IMG, >128MB
|Status=Latest commit: 2026-01-31
}}

{{OSProject
|OSName=BCOS
|OSDescription=BCOS is a practical distributed operating system, initially aimed at 80x86/PC compatible computers. In general BCOS is meant to (eventually) make a group of computers connected by a network (a cluster of computers) behave like a single computer with multiple users. 
|Contact=Brendan Trotter, btrotter [at] gmail [dot] com
|License=Proprietary :(
|URL=http://bcos.hopto.org/
|BootMedia=Unknown
|Status=Latest update: 2019-06-15
}}

{{OSProject
|OSName=Beelzebub
|OSDescription=Beelzebub (The Lord of Flies) features a hybrid kernel, accompanied by libraries and apps written mainly in C++, serving as a platform for development and experimentation of kernel/OS/runtime features. It currently targets AMD64, with plans to support IA-32 and ARM architectures later. It aims to become a clean, modern, and efficient operating system for servers and workstations.
|Contact=contact [at] vercas [dot] com
|License=Open source (BSD-like)
|URL=https://github.com/vercas/Beelzebub
|Pic=[[File:Beelz_3.png|45px]]
|BootMedia=Floppy, 1.44 MB (will be available soon)
|Status=Latest commit: 2021-06-06
}}

{{OSProject
|OSName=BeeOS
|OSDescription=A simple "Unix-like" x86 kernel trying to be POSIX compliant. Currently, the project does not have any long term mighty ambition, it just want to be a good learning tool for operating systems and low level programming topics. A future "on-the-field" usage, especially when ARM port will be ready, is not excluded.
|Contact=davxy [at] datawok [dot] net
|License=Open source (GNU LGPLv3)
|URL=https://github.com/davxy/beeos
|BootMedia=Floppy, 1.44 MB (will be available soon)
|Status=Latest commit: 2023-05-29, is active
}}

{{OSProject
|OSName=BleskOS
|OSDescription=BleskOS is 32-bit operating system written from scratch in C. One of main features is alternative graphic user interface that do not use windows, as classic operating systems do, but switches between programs on full screen. BleskOS is designed for older computers. Goal of BleskOS is to provide system, that could be installed on older computers to make them usable again. BleskOS have wide range of drivers such as VBE, AC97, Intel HD Audio, PS/2 keyboard, PS/2 mouse, PATA(PI), SATA(PI), USB controllers UHCI/OHCI/EHCI, FAT filesystems, ethernet cards from Realtek, Intel, AMD and more. BleskOS also supports widely used file formats like JPG, PNG, GIF, MP3, TXT, HTML and more. Every version is tested on multiple real computers.
|Contact=support@bleskos.com
|License=Open source (MIT)
|URL=https://github.com/VendelinSlezak/BleskOS
|Pic=[[File:Bleskos_2023.png|200px]]
|BootMedia=Floppy, 1.44 MB ; ISO, ~2MB
|Status=Latest commit: 2024-05-25, is active
}}

{{OSProject
|OSName=BlueKernel
|OSDescription=The BlueKernel is a simple microkernel design intended for students. It has pre-emptive multitasking with threads, message-passing and virtual memory. It is currently targeted for the IA-32 processors, but the design is strictly separated into a hardware-dependent mechanisms layer, with an independent policy layer on top of that. It has its own basic boot loader that loads the kernel and a disk driver. It also has a basic text-mode display driver and keyboard driver. As with most microkernels, the drivers run in user space and help to demonstrate message passing and interrupts. When the system starts it loads a server that allows the user to load their own processes. There are very simple example user programs to demonstrate threads, message-passing and interrupts. The website is written like a textbook to explain how it all works, focusing on the implementation rather than theory.
|Contact=http://bluekernel.com.au/#intro,Download%20+%20Contact
|License=Creative Commons, http://creativecommons.org/licenses/by-nc-nd/4.0/
|URL=http://www.bluekernel.com.au
|BootMedia=Floppy, 1.44 MB (at ./makefile change 20160 to 2880)
|Status=Latest update: 2018-12-15
}}

{{OSProject
|OSName=BoneOS
|OSDescription=BoneOS is an Operating System for everyone built by everyone. The goal is to make an Operating System for 'Everyone'. Ranging from Programmers to Normal Users. This operating system currently supports x86 and x64. Later will move forward to other architectures and platforms such as MIPS, PowerPC, ARM, etc...
|Contact=Bone OS Team (https://github.com/Bone-Project/BoneOS)
|License=Open source (GNU GPLv3)
|URL=https://github.com/Bone-Project/BoneOS
|BootMedia=IMG, ~10MB
|Status=Latest commit: 2023-11-20, is active
}}


{{OSProject
|OSName=Brutal
|OSDescription=An operating system inspired by brutalist design that combines the ideals of UNIX from the 1970s with modern technology and engineering. Brutal is a microkernel written in modern C that targets x86-64 and RISCV (WIP). Brutal has: its own UEFI bootloader, a WIP C compiler, an IDL, a GUI, and much more...
If you want a 'complete' list of work in progress or implemented features you can take a look at the roadmap: https://github.com/brutal-org/brutal/blob/main/meta/roadmap.md
|Contact=Brutal discord server: (https://discord.com/invite/gamGsfg)
|License=Open source (MIT) source code available at: https://github.com/brutal-org/brutal
|URL=https://brutal.smnx.sh/
|Pic=[[File:BrutalDemo.png|200px]]
|BootMedia=Floppy, 1.44 MB (will be available soon)
|Status=Latest commit: 2023-08-21, is active
}}

{{OSProject
|OSName=BoxOS
|OSDescription=A simple, Unix-like operating system written in C and GAS assembly. Under heavy development as it cannot switch to Ring-3 as of writing. Major goals include porting many Linux applications and becoming an alternative Unix-like OS to Linux and the BSDs.
|Contact=mailing list: box-kernel@gymli.org, personal: dholman@gymli.org
|License=Open Source (GPLv2)
|URL=https://git.gymli.org/dholman/box.git
|Status=Last tag: 2024-05-29, is active
|BootMedia=IMG, expected to be between 1GB and 2.5GB with a complete userspace
}}

{{OSProject
|OSName=BoredOS
|OSDescription=BoredOS is a UNIX-like operating system written from scratch mostly in C as a hobby project. The name BoredOS comes from creating and working on the project out of boredom.. although now an obsession. It runs purely on x86-64 and has decent POSIX compat.
|Contact=chris@boredos.dev
|License=GNU GPLv3
|URL=https://boredos.dev
|BootMedia=ISO, ~100MB
|Status=Active
|Pic=}}

=C=

{{OSProject
|OSName=CoolPotOS
|OSDescription=CoolPotOS - A monolithic kernel and operating system running under x86
Hello everyone, this is a simple operating system as well as my first project on this field. It can run on i386 or x86_64 machines.
|Contact=3330887365@qq.com
|License=Open source (MIT)
|URL=https://github.com/plos-clan/CoolPotOS
|Pic=[[File:CoolPotOS.png|240px]]
|BootMedia=ISO, >5MB
|Status=Active
}}

{{OSProject
|OSName=ChaiOS
|OSDescription=Modular multi-platform OS. Even Kernel C library dynamically linked (using very basic duplicates). Written in Visual C++ and NASM.
|Contact=solocle: chaios.project [at] gmail [dot] com, personal: bellezzasolo [at] gmail [dot] com
|License=Open source (MIT)
|URL=https://github.com/ChaiSoft/ChaiOS
|Pic=[[Image:ChaiOS sine.png|160px]]
|BootMedia=IMG, >20MB
|Status=Last commit: 2023-04-27
}}

{{OSProject
|OSName=Cloudium OS
|OSDescription=This is being written in pure ASM with cloud computing in mind. Exploring the new exokernel designs and basin design decisions only on speed and cloud. Our homepage: https://sourceforge.net/projects/cloudium-os/
|Contact=0xjarno [at] gmail [dot] com
|License=Proprietary :(
|URL=https://sourceforge.net/projects/cloudium-os/
|BootMedia=IMG, 2KB
|Status=Latest update: 2017-09-15
}}

{{OSProject
|OSName=Cthulhu OS
|OSDescription=Cthulhu OS is actively developed x86-64 based microkernel which uses C11 and custom libc. Currently supports simple user space code. Long term goals include standard desktop operating system. Cthulhu OS supports creating processes from initramfs (called rlyeh) and memory + syscall management. Sadly it has a horrible toolchain and is very difficult (if not impossible) to compile in the up-to-date development environment, unless the author of this project fixes it
|Contact=Peter Vanusanik, admin [at] en-circle [dot] com
|License=Open source (MIT)
|URL=http://enerccio.github.io/Cthulhu-OS/
|BootMedia=Floppy, 1.44 MB (download unavailable)
|Status=Last commit: 2016-01-31
}}

{{OSProject
|OSName=CatK
|OSDescription=CatK is a kernel taking inspiration from FreeBSD and similar Unix-like systems written almost completely in C.
|Contact=No Contact Information
|License=Open Source (GNU GPLv3)
|URL=https://github.com/Rodmatronic/CatK/tree/un-shitified
|BootMedia=IMG, ~2MB (download unavailable)
|Status=Last commit: 2024-08-01
}}

{{OSProject
|OSName=CrusaderOS
|OSDescription=Crusader OS is an DIY hobby operating system made for x86 using GRUB-legacy as bootloader and DIY kernel with some basic ext-like FS. I've made a website for this project so feel free to check out: https://daja201.github.io/crusader-os-v02.html. Also right now it can be booted on real HW but things like keyboard or graphics may not work properly.
|License=Open Source (no licence)
|URL=https://github.com/Daja201/Crusader-OS-v02
|BootMedia=ISO ~600kB
|Status=Last commit: 2026-04-03
|Pic=[[File:Screenshot of COS.png|160px]]
|Website wiki=https://daja201.github.io/crusader-os-v02.html}}
=D=

{{OSProject
|OSName=Dawn
|OSDescription=Dawn OS is a high-level operating system for the [https://en.wikipedia.org/wiki/One_instruction_set_computer#Subtract_and_branch_if_less_than_or_equal_to_zero SUBLEQ] instruction set. It supports SMP, 64 bit memory addressing, multitasking, it has a built-in C compiler, virtual keyboard, and its' own P2P network standard with automatic forwarding beetween clients, and contains various built-in tools like paint, text editor, sound player, chess, piano, amoeba
|Contact=http://users.atw.hu/gerigeri/DawnOS/download.html
|License=Proprietary :(
|URL=http://users.atw.hu/gerigeri/DawnOS/index.html
|Pic=[[Image:Dawn.png|240px]]
|BootMedia=IMG, ~640MB
|Status=Latest release: 2020-07-06
}}

{{OSProject
|OSName=Dennix
|OSDescription=Dennix is a unix-like operating system for x86 and x86_64 written in C and C++. It consists of a monolithic kernel, a standard C library, userspace applications and a graphical user interface. It also comes with several ports of third-party software.
|Contact=https://github.com/dennis95/dennix
|License=Open source (ISC)
|URL=https://github.com/dennis95/dennix
|BootMedia=ISO, ~64MB
|Status=Latest commit: 2023-08-12, is active
}}

{{OSProject
|OSName=DiyOS
|OSDescription=A learn-by-doing OS with a basic GUI.
|Contact=Leonardo Monteiro (leonardoms1911 AT gmail.com)
|License=GPL-3.0
|URL=https://github.com/leonardoms/DiyOS
|Pic=[[Image:Diyos.png|240px]]
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2022-01-29
}}

{{OSProject
|OSName=DragonWare
|OSDescription=DragonWare is a microkernel-based, full operating system for PC hardware. It provides a lightweight bootloader, microkernel and a complete userland and drivers. The purpose of DragonWare is to explore a new, different userland API and provide a complete operating system instead of a bare kernel.
|Contact=Aggelos Tselios (aggelostselios777 AT gmail.com)
|License=Open source (GNU GPLv3)
|URL=https://github.com/tseli0s/DragonWare
|Pic=[[Image:DragonWare-dcp.png|240px]]
|BootMedia=ISO, 828kb
|Status=Latest commit: 2026-10-19, is active
}}

{{OSProject
|OSName=DreamOs
|OSDescription=Home-made operating system, developed in C and asm for x86 architectures.
|Contact=Ivan Gualandri, contact info at the bottom of Github page
|License=Open source (GNU GPLv3)
|URL=https://github.com/inuyasha82/DreamOs
|BootMedia: ISO, 10MB ; Floppy, 1.44 MB (will be available soon)
|Status=Latest commit: 2024-01-09, is active
}}

{{OSProject
|OSName=DreamOs64
|OSDescription=Home-made 64 bit operating system, developed in C and asm from scratch
|Contact=Ivan Gualandri, contact info at the bottom of Github page
|License=Open source (GNU GPLv3)
|URL=https://github.com/dreamos82/DreamOs64
|BootMedia: ISO, 10MB ; Floppy, 1.44 MB (will be available soon)
|Status=Latest commit: 2024-05-26, is active
}}

{{OSProject
|OSName=duckOS
|OSDescription=An x86 monolithic kernel and operating system written in modern C++. Comes with a completely in-house kernel, window manager & compositor, graphical applications, and command-line utilities. It also has ports of binutils and gcc.
|Contact=byteduck, contact info on GitHub
|License=Open source (GNU GPLv3)
|URL=https://github.com/byteduck/duckOS
|Pic=[[Image:DuckOS.png|240px]]
|BootMedia=IMG, ~120MB
|Status=Latest commit: 2024-04-20, is active
}}

{{OSProject
|OSName=Dux
|OSDescription=An x86 operating system with module loading capabilities, started in 2008
|Contact=Ellen Dash, me [at] duckie [dot] co
|License=Open source (MIT)
|URL=https://github.com/duckinator/dux
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2017-12-05
}}

=E=

{{OSProject
|OSName=Escape
|OSDescription=Escape is a 32-Bit microkernel operating system for X86 that supports multitasking and multithreading. It's implemented in ANSI C, C++ and a bit assembler and most parts of it are UNIX-like. The goal is to experiment with it and learn as much as possible about operating systems, hardware and so on.
|Contact=Nils Asmussen, nils [at] script-solution [dot] de
|License=Open source (GNU GPLv2)
|URL=https://github.com/Nils-TUD/Escape
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2023-07-24
}}

{{OSProject
|OSName=Einherjar
|OSDescription=Forth computing environment for PowerPC Macs.
|Contact=Konstantin Tcholokachvili, konstantin [dot] tcholokachvili [at] protonmail [dot] com
|License=Open source (MIT)
|URL=https://github.com/narke/Einherjar
|BootMedia=ISO, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2020-03-16
}}

{{OSProject
|OSName=ExectOS
|OSDescription=ExectOS is a modern, EFI-enabled, general purpose operating system written from scratch and implementing the XT architecture. It runs on x86 and x86_64 architectures and provides NT drivers compatibility layer. Currently, it is in the pre-alpha state.
|Contact=Rafal Kupiec, belliash [at] codingworkshop [dot] eu [dot] org
|License=Open source (GNU GPLv3)
|URL=https://exectos.eu.org/
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest update: 2025-07-19, is active
|Pic=[[Image:ExectOS.png|240px]]}}

=F=

{{OSProject
|OSName=Fiwix
|OSDescription=Fiwix is an '''operating system kernel''' written from scratch, based on the UNIX architecture and fully focused on being POSIX compatible. It is designed and developed mainly as a hobby OS and, since it serves also for educational purposes, the kernel code is kept as simple as possible for the benefit of students and OS enthusiasts. It is small in size (less than 50K lines of code), runs on the i386 hardware platform and is compatible with a good base of existing GNU applications. A special floppy image with RAMdisk support is available.
|Contact=Jordi Sanfeliu, jordi [at] fibranet [dot] cat
|License=Open source (MIT)
|URL=https://www.fiwix.org/
|BootMedia=Floppy, 1.44 MB ; IMG, ~230MB; ISO, ~570MB
|Status=Latest release: 2023-11-15, is active
}}

{{OSProject
|OSName=Fling OS
|OSDescription=Fling OS is an educational operating system with full technical documentation and aiming to have full reference and tutorial articles on every aspect of its code. As of September 2015, they had a C# kernel with ELF driver and USB 2.0 support for x86, a C# AOT compiler, and a cross-platform (x86/MIPS) compiler verification kernel. They launched over 30 articles and a series of 10 Getting Started tutorial videos on 17th September 2015 having been sponsored by Imagination Technologies over the previous summer. Stable core kernel including USB 2.0 and ELF drivers. Also first release of their AOT C# to x86/MIPS compiler and compiler verification kernel. Industry sponsored and collaborating with University Of Bristol. Our homepage: http://www.flingos.co.uk
|Contact=Edward Nutting , osdev [at] flingos.co.uk
|License=Open source (GNU GPLv2)
|URL=https://github.com/FlingOS/FlingOS
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2017-10-30
}}

{{OSProject
|OSName=[[User:ImDaBigBoss/FoxOS|FoxOS]]
|OSDescription=FoxOS's goal is to develop an operation system that focuses on the terminal, performence and reliability. And hope to provide users with a functionnal, performant and stable OS.
|License=Open source (MIT)
|URL=https://theultimatefoxos.github.io/
|Status=Active
|BootMedia=IMG, 60-100MB
|Pic=[[File:FoxOS.png|200px]]
}}

{{OSProject
|OSName=Foton
|OSDescription=A simple, clean code, multi-tasking kernel written in C language for 64-bit processors from the AMD64 family.

Foton (workname) is a hobby operating system based mostly on NOTHING. Lazy development begin at first day of year 2013 and Assembly language was choosen as first one. 
|Contact=CorruptedByCPU at Discord [https://discord.com/invite/cRhs9yy2YE]
|License=Private
|URL=https://github.com/CorruptedByCPU/Foton
|Pic=[[File:Foton_v0.638.png|200px|thumb|Desktop]]
|BootMedia=ISO, 2,3 MiB https://blackdev.org/files/foton.iso
|Status=Latest commit: 2025-07-04, is active
}}

{{OSProject
|OSName=FreeDOS
|OSDescription=Today, FreeDOS is ideal for anyone who wants to bundle a version of DOS without having to pay a royalty for use of DOS. FreeDOS will also work on old hardware and embedded systems. FreeDOS is also an invaluable resource for people who would like to develop their own operating system. While there are many free operating systems out there, no other free DOS-compatible operating system exists. Sources are here: http://www.ibiblio.org/pub/micro/pc-stuff/freedos/files/repositories/1.3/
|Contact=Jim Hall, the mailing lists at http://www.freedos.org/lists/
|License=Open source (GNU GPLv2)
|URL=http://www.freedos.org
|Pic=[[File:FreeDOS.png|200px]]
|BootMedia=Floppy, 1.44 MB ; IMG, 32MB
|Status=Latest commit: 2024-05-29, is active
}}

{{OSProject
|OSName=FROST
|OSDescription=FROST is an OS written in FreeBASIC based on the FROST-Microkernel. Basic kernel functionality available, IPC, VFS and SMP in development
|Contact=https://github.com/thrimbor/frost
|License=Open source
|URL=https://github.com/thrimbor/frost
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2018-04-12
}}

{{OSProject
|OSName=Fudge
|OSDescription=Fudge is a concurrent-focused operating system. Fudge is different from most other operating systems in that it adheres strongly to the actor model. The kernel and every task in the system communicates by posting messages to each other's mailboxes. The way this is implemented makes concurrency feel like a natural part of your programs especially in comparison to the asynchronous programming models used in file-centric operating systems.
|Contact=Jens Nyberg, jens [dot] nyberg [at] gmail [dot] com
|License=Open source (MIT)
|URL=http://github.com/jezze/fudge/
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2026-01-03, is active
}}

{{OSProject
|OSName=Fusion
|OSDescription=Fusion is a modern x86-64 kernel implemented in Nim. It's intentionally not Unix-like. Its core features are: single address space and message passing through channels for task communication. Planned features include capability-based security and state machine oriented channels. It has a companion website that walks through the process of creating the kernel from scratch: https://0xc0ffee.netlify.app/osdev
|Contact=Khaled Hammouda https://github.com/khaledh
|License=MIT
|URL=https://github.com/khaledh/fusion
|BootMedia=QEMU Virtual FAT
|Status=Latest commit: 2025-06-03
}}

{{OSProject
|OSName=FuzzyOS
|OSDescription=32-bit x86 multiprocessing OS capable of running console or graphical user applications. The bootloader and kernel are written in NASM/C, the STL and user applications are written C/C++, while the tests are written in bash and python. It's built as a hobby project for educational purposes. 
|Contact=https://github.com/scopeInfinity/FuzzyOS
|License=Open source (Apache 2.0)
|URL=https://github.com/scopeInfinity/FuzzyOS
|Pic=[[File:FuzzyOS.gif]]
|BootMedia=Floppy, 1.44 MB (will be available soon) ; IMG, 1 MB
|Status=Latest commit: 2021-10-25
}}

{{OSProject
|OSName=FYSOS
|OSDescription=Intel/AMD 32-bit and 64-bit operating system, including most modern hardware specifically the USB and related hardware.
|Contact=Ben Lunt, fys [at] fysnet [dot] net
|License=Proprietary :(
|URL=http://www.fysnet.net/fysos.htm
|Pic=[[File:latest_gui.png|200px]]
|BootMedia=Floppy, 1.44 MB ; IMG, 9 MB
|Status=Latest update: 2022-04-18, is active
}}

= G=

{{OSProject
|OSName=[[User:max/Ghost Kernel|Ghost OS]]
|OSDescription=A homemade operating system with a microkernel for the IA32 (x86) platform. The project is written in C++ and Assembly. Features: multiprocessor- & multitasking support, kernel API library, custom C library, ELF support, IPC (messages, signals, shared memory, pipes), VFS, window server & GUI with homemade toolkit, PS/2 keyboard & mouse driver, VESA video driver and more...
|Contact=Max Schlüssel, lokoxe [at] gmail [dot] com
|License=Open source (GNU GPLv3)
|URL=http://ghostkernel.org/
|Pic=[[Image:ghost-ui-preview.png|160px]]
|BootMedia=ISO, ~80MB
|Status=Latest release: 2025-02-02, is active
}}

{{OSProject
|OSName=[[User:Goldside543/goldspace | Goldspace]]
|OSDescription=A 32-bit free and open source monolithic kernel for the x86 architecture, written in C and assembly. It has multitasking, around a dozen system calls, ELF support, a PS/2 keyboard driver, and more. Also included is Gash, an in-kernel shell for testing features.
|Contact=goldside000 [at] outlook [dot] com
|License=Open source (GNU GPLv2)
|URL=https://github.com/Goldside543/goldspace
|Pic=
|BootMedia=ISO, ~30MB
|Status=Latest commit: 2025-06-04, is active
}}

{{OSProject
|OSName=Gubernatrix
|OSDescription=A 64-bit single-address space, containerizing operating system.
|Contact=himanshu.goel2797 [at] gmail [dot] com
|License=Open source (MIT)
|URL=https://gitlab.com/hgoel0974/gubernatrix
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2019-10-15
}}

{{OSProject
|OSName=[[User:Peyty/GreenteaOS|GreenteaOS]]
|OSDescription=Greentea OS is a 64-bit desktop OS for x86-64 and also potentially promising platforms like ARM64 and RISC-V. Aims to run .exe files natively and NT drivers under sandbox compatibility layer. Follows own non-NT/non-UNIX modular kernel design called Tofita Engine. Developed completely in a custom programming language Hexa tuned for the purposes of the OS. Currently in the alpha state.
|Contact=unstubborn [at] proton [dot] me
|License=Open source (GNU LGPLv3)
|URL=https://github.com/GreenteaOS
|Pic=[[Image:GreenteaOS-preview.jpg|160px]]
|BootMedia=ISO, ~50MB
|Status=Latest commit: 2025-06-18, is active
}}

=H=

{{OSProject
|OSName=Haiku
|OSDescription=Fully featured open source operating system inspired by the commercial Be Operating System. Has a preemptive, modular kernel, nearly complete POSIX compatibility, a nice (non-X11-based) GUI, and a wide variety of ported and native applications (including a WebKit based browser). Nearly the entire operating system is written in C++98 (including the kernel), albeit with little usage of exceptions.
|Contact=haiku-development [at] freelists.org
|License=Open source (MIT)
|URL=https://www.haiku-os.org/
|Pic=[[File:Haiku.png|200px]]
|BootMedia=ISO, 1.4GB
|Status=Latest commit: 2024-05-29, is active
}}

{{OSProject
|OSName=HelenOS
|OSDescription=Preemptive microkernel multiserver design, SMP support, lightweight IPC, thread-local storage and user-space managed fibrils. Our homepage: http://www.helenos.org/
|Contact=helenos-devel [at] lists [dot] modry [dot] cz
|License=Open source (BSD/GPL)
|URL=https://github.com/HelenOS/helenos
|Pic=[[File:HelenOS.png|200px]]
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Last commit: 2024-05-21, is active
}}

{{OSProject
|OSName=HeliX
|OSDescription=HeliX is a german open-source OS. It has a nice shell and already supports FAT12, multitasking, keyboards and mice and very, very much more... HeliX is still in development and it doesn't exist a long time , so there are many features to come! [at] this point of time there are no downloads at the webpage, because there are still some heavy bugs (; -please be patient; they'll come!
|Contact=Illusoft, Ufo [at] sandrakoenig [dot] de
|License=Open source (BSD 2-clause)
|URL=https://github.com/Helix-OS/helix-os
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Last commit: 2016-01-15, is abandoned
}}

{{OSProject
|OSName=[[User:Hhubsinfo/hhuOS|hhuOS]]
|OSDescription=hhuOS is a small operating system for the x86-architecture, build for teaching purposes. The main goal of this project is to show, how different aspects of operating systems theory can be implemented and linked together. The system is not aimed to be a fully featured operating system for daily use. It is implemented in C++ with small parts of assembly where needed.
|Contact=ruhland [at] hhu [dot] de
|License=Open source (GPLv3)
|URL=https://github.com/hhuOS/hhuOS
|Pic=[[Image:HhuOS-logo.png|160px]]
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2024-05-27, is active
}}

{{OSProject
|OSName=Hoho
|OSDescription=Hoho is an x86 open-source OS supporting multitasking with ELF executables, virtual memory, FAT and virtual filesystem, ATA drives and Floppy disks. Soon it will be self hosted with GCC working on top of it.
|Contact=David, kingbabasula [at] gmail [dot] com
|License=Open source (Apache 2.0)
|URL=https://github.com/davidepianca98/hoho-os
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Last commit: 2016-11-12, likely abandoned
}}

{{OSProject
|OSName=Hydrogen
|OSDescription=Hydrogen is an operating system targeting embedded systems (servers, thin clients, SBCs) running on x86. Its written in C and assembly (NASM). There is no support for VGA, its serial-only. My homepage: https://github.com/moondeck/hydrogen/commits/master/
|Contact=moondeck, olgierd [at] novakovsky [dot] eu
|License=Open source (Apache 2.0)
|URL=https://github.com/moondeck/hydrogen/commits/master/
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2017-06-01
}}

{{OSProject
|OSName=HypnoticOS
|OSDescription=64-bit Hobby Operating System
|Contact=https://github.com/hypnoticos/hypnoticos
|License=GNU GPLv3
|URL=https://github.com/hypnoticos/hypnoticos
|Pic=[[Image:hypnoticos-screenshot.png|160px]]
|BootMedia=ISO, 16 MB
|Status=Last commit: 2024-07-13
}}

=I=

{{OSProject
|OSName=Ironclad
|OSDescription=Ironclad is a mostly formally verified hard real-time kernel written in SPARK and Ada. It is made to be 100% free software, free in the sense that it respects the user's freedom. It supports several architectures and features an advanced security model with features like Mandatory Access Control (MAC). The screenshot features Gloire, a distribution of Ironclad.
|Contact=streaksu@protonmail.com
|License=GNU GPLv3
|URL=https://ironclad.cx
|Pic=[[File:Gloire.png|240px]]
|BootMedia=IMG, 1 GB (given for a Gloire)
|Status=Latest commit: 2024-05-28, is active
}}

{{OSProject
|OSName=Iridium
|OSDescription=Iridium is a small monolithic kernel designed to be a minimal OS to be compatible with as much x86_64 hardware as possible. Any x86_64 compatible CPU should be able to boot Iridium. Iridium comes with a CSPRNG. The Iridium kernel comes in two branches, dev, and main, main is stable and gaurenteed to function, while dev is a hot mess and includes constant rewrites.
|Contact=security@stackunderflow.org
|License=GNU GPLv3
|URL=https://stackunderflow.org
|Pic=
|BootMedia=IMG file, size configurable.
|Status=Its honestly easier to tell you gaps in development at this point.
}}

=J=

{{OSProject
|OSName=JaeOS
|OSDescription=Just Another Embedded OS. My homepage: http://jaeos.com/
|Contact=http://jaeos.com/contact-us.html
|License=Open source (MIT)
|URL=https://github.com/azsoter/jaeos-devel
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2020-11-05
}}

{{OSProject
|OSName=JSD/OS
|OSDescription=JSD/OS which either stands for "Jake S. Del Mastro Operating System" or "Jake'S Disk Operating System" is a tiny Pseudo Single-Tasking operating system for 32bit x86, developed on and off since 2014. The goal is to build an OS with a very small memory footprint, a fast file system and minimal overhead to allow user requests to be processed as quickly as possible, ideal for interactive applications, using a unique scheduler known as CuFS (Conciously unFair Scheduling). The system can run applications packaged in the ELF object format and has full support for memory protection via virtual address spaces. The system includes a shell which works with many DOS & Unix commands.
|Contact=https://jakedelmastro.com/contact/
|License=Open source (BSD 3-Clause)
|URL=https://github.com/pgrAm/JSD-OS
|Pic=[[File:Jsdos-1.gif|240px]]
|BootMedia=Floppy, 1.44 MB (will be available soon) ; IMG, 2 MB
|Status=Latest commit: 2022-07-23
}}

{{OSProject
|OSName=JSLK
|OSDescription=JSLK is a 32-bit hobby kernel designed to run on the x86 architecture (although portability has been taken into consideration). It was started in August 2017 with the objective of learning how a computer operating system works and to improve the coding skills of the main developer. The kernel is mostly written in C with some bits of assembly and has partial compatibility with C++ (a runtime is provided). Some of its features are: extensive HAL API, System V ABI compatibility, memory management, interrupt handling, VGA driver, timers and delays, some synchronization primitives, a growing home-made C library, virtual memory, a VFS, a keyboard and RTC driver and more. Website: https://sofferjacob.github.io/jslk.html
|Contact=Jacobo Soffer, sofferjacob [at] gmail [dot] com
|License=Open source (GNU GPLv3)
|URL=https://github.com/sofferjacob/jslk
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Last commit: 2018-12-21
}}

=K=

{{OSProject
|OSName=KLIKA-OS
|OSDescription=Simple x86_64 multi tasking OS with GUI. Features preemptive multitasking, memory management, userspace, FAT 12/14/16, GUI with VESA, etc. Written in C and assembly. 
|Contact=Zaharije Pasalic, zaharije [at] klika [dot] ba
|License=Open source
|URL=https://github.com/klikaba/klika-os
|Pic=[[File:Klika_os.png|200px]]
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2020-02-22
}}


{{OSProject
|OSName=KnightOS
|OSDescription=Open-source operating system for Texas Instruments calculators. Features preemptive multitasking, memory management, etc. Written in z80 assembly. Mature SDK, usable kernel, usable userspace, no math. Our homepage: http://knightos.org
|Contact=Drew DeVault, sir [at] cmpwn [dot] com
|License=Open source (Beerware)
|URL=https://github.com/KnightOS/KnightOS
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2021-05-02
}}

{{OSProject
|OSName=Kolibri OS
|OSDescription=Kolibri OS was a fork of the 32-bit version of Menuet OS but has changed much along the way. Despite fitting on a standard 1.44 MB floppy, this wonderful OS contains: the complete GUI desktop, a lot of drivers and great software (such as web browser and music player), system programs and games! The Kernel and most applications, libraries and drivers are written in FASM, but some are in C-- (https://en.wikipedia.org/wiki/C-- ; link to their C-- compiler: http://c--sphinx.narod.ru/). At this OS you can write the ASM code and execute it after assembling. Source code is open - http://websvn.kolibrios.org/listing.php?repname=Kolibri+OS - and the contributions are welcome!
|Contact=The Kolibri OS team
|License=Open source (GNU GPLv2)
|URL=http://www.kolibrios.org/en/
|Pic=[[File:KolibriOS.png|240px]]
|BootMedia=Floppy, 1.44 MB
|Status=Latest commit: 2024-05-29, is active
}}

{{OSProject
|OSName=Kora OS
|OSDescription=Kora is an operating system under it's way. The OS is still a young pup develop by an enthusiast engineer as a learning project.
|Contact=Fabien Bavent (fabien.bavent@gmail.com)
|License=Open source (GNU AGPL)
|URL=https://github.com/AxFab/kora-kernel
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2023-05-11, is active
}}

{{OSProject
|OSName=Korus OS
|OSDescription=Korus OS is an experimental operating system designed for learning, research, and low-level system development.
|Contact=Leandro Mesquita (ciano@ymail.com)
|License=This project is licensed under the MIT License.
|URL=https://github.com/xlmp/KorusOS
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2026-04-14, is active
}}


{{OSProject
|OSName=KOS
|OSDescription=KOS is a Hobby Operating System. It is meant to be minimalist and easy to understand.
|Contact=Karthik Kumar Viswanathan (karthikkumar@gmail.com)
|License=Open source (Standard Disclaimer)
|URL=https://github.com/guilt/KOS
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2019-07-29
}}

=L=

{{OSProject
|OSName=Lambda OS
|OSDescription=Lambda OS is a hobby operating system developed by Peter Farley. Lambda OS is designed to work on any i386-compatible processor, but its' design allows for easier addition of new architectures without any major modification of the main kernel code
|Contact=Peter Farley, far.peter1 [at] gmail [dot] com
|License=Open source (MIT)
|URL=https://github.com/farlepet/lambda-os
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2022-08-30, is active
}}

{{OSProject
|OSName=LaylaOS
|OSDescription=LaylaOS is a hobby operating system with a monolithic kernel that is written in C. It has many features including networking, ATA/ATAPI, AHCI, Intel HDA, virtual consoles, vdso, IPC, demand paging, and ACPI (ACPICA). Ported software includes SDL2, Qt5, musl and more
|Contact=Mohammed Isam, mohammed_isam1984 [at] yahoo [dot] com
|License=Open source (GPL2)
|URL=https://github.com/moisam/laylaos
|Pic=[[File:LaylaOS.png|240px]]
|BootMedia=IMG, 3.0 GB
|Status=Latest commit: 2024-09-08, is active
}}

{{OSProject
|OSName=Lemon OS
|OSDescription=Lemon OS is a UNIX-like 64-bit operating system written in C++. It features various software ports such as DOOM, networking (TCP/UDP), and various drivers including AHCI and NVMe.
|License=Open source (BSD 2-clause)
|URL=https://github.com/LemonOSProject/LemonOS
|Pic=[[File:LemonOS.png|240px]]
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2023-11-04, is active
}}

{{OSProject
|OSName=Libra
|OSDescription=Libra is a small hobby kernel, trying to power La Carrera.
|License=Open source (GPL-v3)
|URL=https://github.com/Sysdev32/libra
|BootMedia=IMG, 17.75 MB
|Status=Latest commit: 2026-17-06, is active
}}
{{OSProject
|OSName=LF OS
|OSDescription=Microkernel based operating system for amd64 with high ambitions. Work primarily happens on a [https://praios.lf-net.org/littlefox/lf-os_amd64 gitlab instance] (complete with CI/CD system), but the master branch is also mirrored to github for convenience. Check out the repo if you want to know more, changes are happening too fast to update everywhere x)
|Contact=[https://littlefox.narf.press/lf-os_amd64/md_CONTRIBUTING.html CONTRIBUTING.md]
|License=Open source (MIT)
|URL=https://github.com/LittleFox94/lf-os_amd64
|Pic=[[File:LF_OS-20210519.png|200px|thumb|LF OS kernel initialization in QEMU]]
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2024-05-29, is active
}}

=M=

{{OSProject
|OSName=MonkeOS
|OSDescription=MonkeOS is a monolithic operating system written in Rust for the x86_64 architecture, designed for fast iteration and practical usability. It features a graphical desktop environment, a native command ecosystem, and support for running real-world software such as Lua and TinyCC through a custom libc/POSIX-like compatibility layer.
|Contact=David Dull
|License=Open source (see repository)
|URL=https://codeberg.org/CoopsCorp/MonkeOS
|BootMedia=ISO (~150MB)
|Status=Active (latest commit: 2025-06-21)
}}

{{OSProject
|OSName=Maestro
|OSDescription=Maestro is a kernel and operating system written in Rust, aiming to be a lightweight version of Linux, removing legacy while staying usable in most usecases. It runs on the x86_64 and x86 (32 bit) architectures and is able to run bash, GNU coreutils & binutils, neofetch and gcc/g++, using musl.
|Contact=Maestro Discord server: https://discord.gg/4JMBN3YPAk
|License=Open source (AGPLv3). Source code: https://github.com/maestro-os/maestro
|URL=https://blog.lenot.re
|Pic=[[File:Maestro.png|200px]]
|BootMedia=ISO, 48MB
|Status=Latest commit: 2025-06-21, is active
}}

{{OSProject
|OSName=Managarm
|OSDescription=Managarm is a 64-bit OS for x86-64. It’s written in C++ with a custom libc and a GNU like userland on top. Managarm’s main purpose is aiming for Linux compatibility while being completely asynchronous in terms of I/O. The OS is capable of running Weston and kmscon while effort is made into porting (a subset of) Xorg. Furthermore, Managarm supports many modern hardware devices, including USB 3 and has nearly full ACPI support.
|Contact=The official Managarm Discord server https://discord.gg/7WB6Ur3
|License=Open source (MIT)
|URL=http://www.managarm.org
|Pic=[[File:Managarm-demo.png|200px]]
|BootMedia=IMG, ~1GB
|Status=Latest commit: 2024-05-29, is active
}}

{{OSProject
|OSName=MaslOS 2
|OSDescription=MaslOS 2 is a goofy 64-bit OS written in C++. It has a custom stdlib, a desktop and a custom GUI-Framework. It's the follow up to the original MaslOS. The main difference being an actual scheduler and support for ELF loading/execution. It is very WIP but quite a few things work already and it boots on real hardware. Mostly done just for fun. (Not POSIX compliant or anything)
|Contact=You can contact me using Discord @marceldarcel
|License=Open-Source (AGPL-3.0)
|URL=https://github.com/marceldobehere/MaslOS-2
|Pic=[[File:MaslOS2-demo.png|230px]]
|BootMedia=ISO, 64 MB
|Status=Latest commit: 2024-05-08, is active
}}

{{OSProject
|OSName=MentOS
|OSDescription=MentOS (Mentoring Operating System) is an open source educational operating system. The goal of MentOS is to provide a project environment that is realistic enough to show how a real Operating System work, yet simple enough that students can understand and modify it in significant ways. There are so many operating systems, why did we write MentOS? It is true, there are a lot of education operating system, BUT how many of them follow the guideline defined by Linux? MentOS aims to have the same Linux's data structures and algorithms. It has a well-documented source code, and you can compile it on your laptop in a few seconds!
|Contact=The MentOS development team
|License=MIT
|URL=https://mentos-team.github.io/
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2024-04-05, is active
}}

{{OSProject
|OSName=MenuetOS
|OSDescription=MenuetOS is a fully 32 bit assembly written graphical operating system. Menuet supports 32 bit x86 assembly programming as a faster and smaller system footprint.  Menuet has no unix roots and the basic system is meant to be a clean asm based structure. Menuet isn't based in any particular operating system, since the idea has been to remove the extra layers between different parts of software, which complicate programming and create bugs.  Menuet's application structure is not specifically reserved for asm programming since the header can be produced with practically any other language. However, the overall application programming design is intended for easy 32 bit asm programming. The GUI is extremely easy to handle with assembly language
|Contact=The MenuetOS development team
|License=Proprietary :( 64 bit version ; Limited open source 32 bit version
|URL=http://www.menuetos.net/
|BootMedia=Floppy, 1.44 MB
|Status=Latest release of open source 32-bit version: 2015-02-20, Latest release of proprietary 64-bit version: 2019-02-02
}}

{{OSProject
|OSName=Merlon
|OSDescription=Merlon is an operating system based upon my previous kernel, ATOS. It is designed to be easy to understand, and fairly lightweight (for example, it only requires 3MB of RAM to run on an x86). Currently only implemented for x86, but it should be easy to port to other platforms. Has a preemptive kernel which allows dynamic loading of kernel modules, and supports page replacements and a dynamically linked kernel-to-usermode API. Current working on implementing more of the C POSIX library.
|Contact=Alex Boxall, alexboxall3 [at] gmail [dot] com
|License=BSD 3 Clause
|URL=https://github.com/alexdboxall/Merlon/
|Pic=[[File:Merlon_os.jpg|200px]]
|BootMedia=IMG, ~64MB
|Status=Latest commit: 2024-06-01, is active
}}

{{OSProject
|OSName=Mettā
|OSDescription=Mettā aims to be your mithril compass, that is, a device powerful in determining what is worth and what is not worth doing, in determining when it is the right time for doing so and also on doing the things it can do, without taking up your time.
|Contact=Berkus, berkus [at] exquance [dot] com
|License=Open source (Boost)
|URL=https://github.com/berkus/metta
|Pic=[[Image:Metta_bootup.png|200px]]
|BootMedia=ISO, ~10MB
|Status=Latest commit: 2020-02-17
}}

{{OSProject
|OSName=MicroBe OS
|OSDescription=Hobby OS aimed to learn how things are done. Written in x86 assembler. With huge time gaps developed since 1997 but so far no release available to public
|Contact=Vladimír Šiman, online [at] microbe [dot] cz
|License=Proprietary :(
|URL=http://www.microbe.cz/
|BootMedia=IMG, ~300KB
|Status=Last commit: 2017-03-04
}}

{{OSProject
|OSName=MikeOS
|OSDescription=MikeOS is an operating system for x86 PCs, written in assembly language. It is a learning tool to show how simple 16-bit, real-mode OSes work, with well-commented code and extensive documentation. It has a BASIC interpreter with 46 instructions, supports over 60 syscalls, could manage a serial terminal connection and output the sound through PC speaker. There is also a file manager, text editor, image viewer and some games
|Contact=Mike Saunders, okachi [at] gmail [dot] com
|License=Open source
|URL=http://mikeos.sourceforge.net/
|Pic=[[File:MikeOS.png|240px]]
|BootMedia=Floppy, 1.44 MB
|Status=Latest release: 2022-04-09
}}

{{OSProject
|OSName=MichalOS
|OSDescription=MichalOS is a new x86 hobby OS project that is based on MikeOS: it has been created during the time when it seemed that MikeOS is abandoned, and has more features/programs compared to the original project. Also, these two projects seem to have a different focus: MikeOS appears better suited for education (i.e. has more documentation) while MichalOS is more about running/creating the cool stuff: has more games, a music player with nice "retro computing"-style music, etc.
|Contact=Michal Prochazka, https://www.prochazkaml.eu/
|License=Open source
|URL=https://github.com/prochazkaml/MichalOS
|BootMedia=Floppy, 1.44 MB
|Status=Latest commit: 2023-07-29, is active
}}

{{OSProject
|OSName=MINTIA
|OSDescription=MINTIA is a late 1980s inspired multitasking and paging operating system, written entirely in a custom programming language. It runs on two hobby computer architectures, XR/station and fox32, and and sports sophisticated capabilities such as page swapping, fully shared file mapping, and fundamentally asynchronous I/O. It targets a 4MB RAM minimum, and has a custom API with 120 syscalls.
|Contact=Will, hyenasky@proton.me
|License=Custom source-available license
|URL=https://github.com/xrarch/mintia/
|Pic=[[File:Mintia.png|240px]]
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2024-04-30
}}

{{OSProject
|OSName=mkfreeOS
|OSDescription=The kernel is developed in C++. It is designed to run on Intel x86 architecture. Supported functionalities: Multitasking, Multithreading, FAT32 and BFS file systems (proper for mkfree), Virtual memory manager, Controllers, Hierarchical protection domains (Kernel runs in ring 0 and user applications in ring 3). Multitasking: Each task is executed every 1ms intervals. For now all tasks have the same priority. There is a main thread of execution and it can have these states: 1- suspended, 2- waiting and 3- executing. Multiple threads: Each task can have several threads of execution and can have several states: 1- suspended, 2- waiting and 3- executing. GUI: Simple graphical interface.
|Contact=Ramón Mayedo, Ramón [at] gmail [dot] com
|License=Open source
|URL=https://github.com/ramonmayedo/mkfreeOS
|Pic=[[Image:qemu_gui mkfree1.png|200px]]
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2021-03-28
}}

{{OSProject
|OSName=Modularus Kernel
|OSDescription=Modularus is a research operating system kernel implementing a fully dynamic, hyper-modular architecture in which all subsystems can be replaced at runtime. Kernel components communicate through a unified VFS-based interface extending the “everything is a file” design philosophy. The architecture does not conform strictly to monolithic, microkernel, or exokernel models, instead forming a self-bootstrapped, structurally reconfigurable system.
|Contact=https://github.com/VOXIDEVOSTRO
|License=MIT License
|URL=https://github.com/VOXIDEVOSTRO/Modularus
|BootMedia=ISO, IMG
|Status=Active development
}}

{{OSProject
|OSName=Mu
|OSDescription=Minimal software stack bootstrapped from 32-bit x86 machine code without any C. Supports [[VBE|SVGA]] with a single font ([http://unifoundry.com/unifont GNU Unifont]), a [[PS/2_Keyboard]], [[ATA_PIO_Mode#28_bit_PIO|ATA disks using 28-bit PIO mode]]. Implemented in a memory-safe statement-oriented language, and includes a Lisp-based prototyping environment.
|Contact=[http://akkartik.name/contact Kartik Agaram]
|License=Open Source (GNU GPL v2)
|URL=https://github.com/akkartik/mu
|Pic=[[File:Mu-screenshot.png|200px]]
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2023-12-19, is active
}}

{{OSProject
|OSName=MVS/380
|OSDescription=IBM's old IBM mainframe operating system (MVS) was public domain. With a freely available IBM hardware emulator available, this operating system was dusted off and being given a new life with some radical architecture changes. There's a similar VM/380 available too, for another IBM mainframe OS. Version 1.0 released and a group of people driving it forward on a daily basis instead of being a one-man project. Our homepage: http://mvs380.sourceforge.net
|Contact=Paul Edwards, fight [dot] subjugation [at] gmail [dot] com
|License=Open source (Public Domain)
|URL=https://sourceforge.net/projects/mvs380/
|BootMedia=IMG, ~10 MB
|Status=Latest update: 2024-05-24, is active
}}

{{OSProject
|OSName=MysticOS
|OSDescription=MysticOS is an exokernel based OS. Its main goal is to be able to adapt to the users wishes and abilities at run time, providing the perfect environment for anybody. Current features include FreeBasic support and hardware accelerated graphics (2D and 3D). The OS is currently pre-alpha - there are no official releases yet. However, nightly builds, source code and documentation are available
|Contact=Marcel Sondaar, marcel_willem [at] hotmail [dot] com
|License=Open source (GNU LGPL)
|URL=http://www.d-rift.nl/combuster/mos3/
|BootMedia=Floppy, 1.44 MB
|Status=Latest commit: 2017-09-11
}}

=N=

{{OSProject
|OSName=Nano
|OSDescription=Small and simple operating system. There are 8086 real-mode (Nano16) and 386 protected-mode (Nano32) versions.
|Contact=vialamo at OSDev forums
|License=Open source
|URL=https://github.com/NANO-DEV/
|BootMedia=Floppy, 1.44 MB
|Status=Latest commit: 16-bit: 2020-01-13, 32-bit: 2020-01-26
}}

{{OSProject
|OSName=NetDOS/32
|OSDescription=32bit rewrite of my previous crappy operating system, NetDOS aka NetDOS/16. Will support multitasking and run NetDOS/16 programs.
|Contact=clementttttttttt at OSDev forums
|License=dowhateveryouwantwithitidontcare
|URL=https://github.com/clementtttttttt/NetDOS-32
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2021-06-27
}}

{{OSProject
|OSName=NexNix
|OSDescription=NexNix is designed to be a powerful, safe, and modern operating system. It aims to take Unix and greatly improve on it, by making the filesystem more structured, being a true microkernel, implementing modern security measures, and being more GUI-centric. It still aims to be compatible with the vast majority of Unix programs. Currently, it only supports i386 and x86_64, with slowly developing ports to aarch64 and RISC-V 64.
|License=Apache 2.0
|URL=https://github.com/nexos-dev/nexnix
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2024-02-27, is active
}}

{{OSProject
|OSName=Night Kernel
|OSDescription=The Night kernel is a 32-bit drop-in replacement for the original 16-bit kernel of the FreeDOS operating system. It uses linear memory addressing and operates in protected mode on the Intel x86 architecture. The typical user will retain compatibility with their DOS applications and gain protected mode abilities such as task switching between applications, protected memory and increased overall performance in a DOS environment. The community forum has moved from google groups to https://nightkernel.proboards.com/
|Contact=mercury0x0d-at-protonmail.com
|License=Open source (GPLv3)
|URL=https://github.com/mercury0x000d/NightKernel
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2024-02-13, is active
}}

{{OSProject
|OSName=nightingale
|OSDescription=Nightingale is a small operating system with a vaguely UNIX-like userland. It supports multiple processes, loadable kernel modules, networking, and has a (fairly) full featured shell with pipes and file redirection. It has no video support, and only communicates with the outside world via the serial ports and network card.
|Contact=nightingale-at-tylerphilbrick.com
|License=Open source (GPLv3)
|URL=https://github.com/tyler569/nightingale
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2024-05-16, is active
}}

{{OSProject
|OSName=NOS
|OSDescription=NOS is a microkernel OS project written in C++, making use of modern C++ standards (from C++11 onwards). It targets multiple architectures, is meant to be portable and provides a structured, object-oriented API. It is mostly inspired by the L4KA series of microkernels. Currently it is the basic kernel boot and thread creation / switching on x86 32-bit and 64-bit platforms
|Contact=Manuel Hohmann, XenOS at OSDev forums
|License=None
|URL=https://github.com/xenos1984/NOS
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2024-04-23, is active
}}

{{OSProject
|OSName=NovumOS
|OSDescription=A modern, high-performance 32-bit kernel written primarily in Zig. Key features include Symmetric Multi-Processing (SMP) with up to 16 cores and a work-stealing scheduler, advanced memory management with PSE (4MB Huge Pages) and demand paging, and a fully functional FAT32 driver with LFN support. It features a custom embedded scripting language (Nova) and a Ring 3 user-space environment with an SDK.
|Contact=MinecAnton209 on GitHub
|License=Apache 2.0
|URL=https://github.com/MinecAnton209/NovumOS
|BootMedia=ISO / Raw Image
|Status=Active, latest release v0.23-beta.3 (Feb 2026)
}}

{{OSProject
|OSName=Neuro-SamaOS
|OSDescription=A 64-bit UEFI hobby OS that boots on real hardware. Features: xHCI, HDAudio, PS/2, AMP, PCI, APIC, IOAPIC, AVX256, SSE. Includes a self-healing IDT with a built-in CISC instruction parser that can skip faulty machine code by manipulating RIP. Written entirely from scratch as a tribute to TempleOS, not based on Linux or Windows.
|Contact=Andy T. Davis andrewtdavis409@gmail.com
|License=Open source (MIT)
|URL=https://neurosamaos.dpdns.org/
|BootMedia=ISO / Efi
|Status=Active (Pre-Alfa 2026.09.02) (Next version maybe is Alfa 2027.7.12)
|Pic=[[File:NeuroSamaOSImg.png|240px]]}}

=O=

{{OSProject
|OSName=OpenXSKernel
|OSDescription=OpenXSKernel is a kernel development project for 64bits operating systems, based on the MIT open source license. It was presented by an operating system development project "XJ380" by XINGJI Studios(XINGJI Interactive Software). It will continue to be updated with the project "XJ380".
|Contact=guoqiyu1115@outlook.com
|License=Open source (MIT)
|URL=https://www.xingjisoft.com/open-source
|Status=Active
}}

{{OSProject
|OSName=Oberon System
|OSDescription=The Oberon System is an academic operating system and an integrated software environment developed by Niklaus Wirth and Jürg Gutknecht since 1980s at [http://www.oberon.ethz.ch/ ETH in Zürich]. Currently it is a single-user, multi-core, multi-tasking system that runs on bare hardware or on top of a host operating system (currently Microsoft Windows or Linux). The developers aim at producing a reliable, real-time operating system suitable for embedded systems and for industrial and in particular medical applications. Earlier it was called "Aos" (Active Object System), a nomenclature that is still in use. It is written in the Active Oberon programming language, which evolved from Oberon, a programming language in the Pascal/Modula tradition. The graphical user interface is referred to as “Bluebottle”. See also: [http://en.wikipedia.org/wiki/Oberon_(operating_system) Oberon Operating System on Wikipedia] and [http://ignorethecode.net/blog/2009/04/22/oberon/ Lukas Mathis' Blog: Ignore the Code]. Note that some stuff on these pages appear outdated, but this may be more a problem of keeping the Web-pages as current as the System. More current and in depth documentation is included in the systems, although you (sorrily) have to master their (partially) unconventional user-interface before you can access the documentation. Many dead links in [http://www-old.oberon.ethz.ch/native Native Oberon] reference site can be reanimated by replacing www with www-old. It has been revitalized in fall 2013 by Niklaus Wirth implementing a RISC processor in FPGA. See [http://projectoberon.com Project Oberon]
|Contact=Forums - http://www.ocp.inf.ethz.ch/forum/ ; mailing list - https://lists.inf.ethz.ch/pipermail/oberon
|License=Open source (BSD-like)
|URL=https://www.projectoberon.net/
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2022-12-29, is active. More info at [http://sourceforge.net/projects/nativeoberon/ SourceForge Native Oberon], [http://www.ethoberon.ethz.ch/ Old ETH Oberon Home Page], [http://www.ssw.uni-linz.ac.at/Oberon.html Linz Oberon V4], [http://sourceforge.net/projects/oberon/ SourceForge Oberon V4], and [http://projectoberon.com Project Oberon])
}}
{{OSProject
|OSName=OS
|OSDescription=OS for x86 with the goal to provide a complex system basics implemented with a maximum delay of 1 year (virtual memory, file systems, emulation, GUI, multitasking, clean-up of general information, drivers from other OSes, USB, SATA), and general study written in NASM, C and miscellaneous language snippets gathered from the usual PC demo code repositories, books, and study of professional programs. It aims at easily building a custom OS-like DOS application manually, simple or with all the available protection and system features, to study the implementation of the different system-level tasks. It contains simple 386 malloc/free functionality for paging, simple FAT32 LBA support for displaying files and folders with a simple fopen/readdir/Read_Disk/Read_Disk_DWORD for clusters, and several functions for handling VGA, PS/2, PIT timer, ATA-ATAPI. It boots from DOS and can return to it with the exittodos command, even under the newest laptops. Contains documentation in Spanish and English. '''Decompress the TAR to the root directory. c:\start.bat launches it'''. Has miniprograms that can be invoked as commands with arguments, and has fail-proof ATA-ATAPI detection (only primary master enabled by now). '''In 2021 it will add a basic emulator for being able to call things like video modes natively derived from the BIOS and to access the memory map from the kernel.'''
|Contact=~ at OSDev forums
|License=Open source (Public Domain or no license - to be treated as a simple replay view of writing random code as a game)
|URL=http://sourceforge.net/projects/lowest-kernel/files/
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest update: 2021-01-05
}}

{{OSProject
|OSName=OS/K (OS on Kaleid)
|OSDescription=A fully free as in freedom operating system concepted from scratch, with the only goal to be a hobby OS to learn and practice. It is under regular development, since the contributors are students.
|Contact=os-k-team@os-k.eu
|License=Free Software (GNU GPL version 3)
|URL=https://forge.chapril.org/os-k-team/os-k/src/branch/master/kaleid
|Pic=[[File:OS-on-kaleid.png|200px]]
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2021-02-22
}}

{{OSProject
|OSName=OS/Z
|OSDescription=OS/Z is an open-source, multiplatform, micro-kernel hobby OS written in C and assembly. It implements some interesting, non-standard concepts, and it's deliberately not fully POSIX compliant (although the API is POSIXish as much as possible). Its goal is to handle big amounts of data (in magnitude of yotta) in an user friendly, efficient way. Has it's own built-in debugger, memory allocator, unique VFS implementation and graphical interface protocol which is somewhere between X and Wayland. Currently supports x86_64 and AArch64 architectures and is under heavy development.
|Contact=bzt at OSDev forums
|License=Open source (Creative Commons)
|URL=https://bztsrc.gitlab.io/osz/
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest update: 2020-11-02
}}

{{OSProject
|OSName=opuntiaOS
|OSDescription=opuntiaOS - an operating system targeting x86 and ARMv7. It features a kernel with great features like SMP and Ext2, custom runtime libraries for C/C++/ObjC and libraries for UI.
|Contact=nimelehin@gmail.com
|License=BSD-2-Clause License
|URL=https://github.com/opuntiaOS-Project/opuntiaOS
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2023-04-06, is active
}}

{{OSProject
|OSName=Oxide
|OSDescription= Oxide or "02" is an OS for the 65xx family of processors and other mpu/mcu in embedded systems. Low run levels support headless or autonomous operation while higher runlevels are for human interfaces such as games, browsers, applications and media viewers. Next release is going to include the following features: X/Y coordinates, addition and subtraction, counter, select/reset. The project is going to be released at wayfarertechnologies.net (site is down, the author is seeking a new host)
|Contact=wayfarer.technologies@gmail.com
|License=<undecided open source license; OS is for an open hardware platform>
|URL=http://forum.6502.org/viewtopic.php?f=2&t=7542
|BootMedia=No releases yet; floppy - unlikely, ROM images are more likely
|Status=Under development since ~2023
}}

=P=

{{OSProject
|OSName=PatchworkOS or Patchwork
|OSDescription=Patchwork is a 64 bit monolithic hobbyist OS built from scratch in C for the x86_64 architecture, it is intended as an easy-to-modify toy-like Unix-inspired OS (not Unix-like) it takes many ideas from Unix while simplifying them and removing some of the fat. Made entirely for fun.
|Contact=Kai Norberg, KN.KaiNorberg@gmail.com
|License=
|URL=https://github.com/KaiNorberg/PatchworkOS
|Pic=
|BootMedia=
|Status=Under Development
}}

{{OSProject
|OSName=PDOS - Public Domain Operating System
|OSDescription=One of very few operating systems that have an explicit "released to the public domain" notice. What that means is that if you spend time on this, and you later see a commercial use for it, there is absolutely no restriction on selling/modifying etc, any more than you would dig up Shakespeare and ask him if it's OK to use "Hamlet". It is designed to look like MSDOS, and the 8086 version can currently execute some MSDOS executables unchanged, the 80386 version can currently execute some Win32 executables (including gccwin - a modified GCC 3.2.3 for Windows msvcrt.dll) unchanged, and the S/3X0 version can currently execute some MVS executables unchanged (and on real IBM mainframe z/Arch hardware executes S/370 in effective AMODE 32 giving access to 4 GiB of memory), and the x64 UEFI version can execute some Win64 executables unchanged, including a completely public domain completely-self-hosting environment. It's written in C90, with some assembler. The public domain C compiler included (only) in the x64 version has a small amount of (included) C99 dependency. There is also a native ARM32 version.
|Contact=Paul Edwards, mutazilah@gmail.com
|License=Open source (Public Domain)
|URL=http://pdos.org
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2024-05-30, is active
}}

{{OSProject
|OSName=Pedigree
|OSDescription=Monolithic OS with several backends supported - x86, x64, MIPS32, ARM and PowerPC. Kernel written in C++ with the obvious bits of ASM. Offers a reasonable amount of POSIX support and a tiling GUI and can run Apache, DOSBox, and various other common programs. Planned to also offer a native API alongside POSIX for Pedigree-specific applications
|Contact=JamesM, bluecode, [http://ideasandcode.blogspot.com/ pcmattman], IRC freenode.net#pedigree
|License=Open source (ISC)
|URL=https://github.com/miselin/pedigree
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2023-05-29, is active
}}

{{OSProject
|OSName=Perception
|OSDescription=An x86-64 OS built around a microkernel. Custom build system, IPC IDL, UI toolkit. Not POSIXish. Userland is built in modern C++.
|Contact=[https://forum.osdev.org/ucp.php?i=pm&mode=compose&u=1592 AndrewAPrice]
|License=Open source (Apache 2)
|URL=https://github.com/AndrewAPrice/Perception
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2024-03-15, is active
}}

{{OSProject
|OSName=PicoBSD
|OSDescription=The stripped down version of FreeBSD for hobbyists and embedded systems developers. Its' primary difference is a tiny size, hence the "PicoBSD" name. Unlike OpenBSD ''(which provides the installation floppy with a limited set of features)'', PicoBSD is the only modern BSD system which could fit on a floppy  - although, as the time passes and the BSD kernel grows in size - it becomes more and more difficult to fit. The archived floppies are available at https://sourceforge.net/projects/archiveos/files/p/picobsd/
|Contact=freebsd-embedded mailing list, https://lists.freebsd.org/mailman/listinfo/freebsd-embedded
|License=Open source (BSD)
|URL=https://people.freebsd.org/~picobsd/old/picobsd.html
|BootMedia=Floppy, 1.44 MB
|Status=Last update: 2017-11-18
}}

{{OSProject
|OSName=Plan 42
|OSDescription=A small public-domain OS, based on the 'nanos' nanokernel.
|Contact=Andy Elvey
|License=Open source (Public Domain)
|URL=https://github.com/mooseman/plan_42
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Last commit: 2016-01-08, abandoned
}}

{{OSProject
|OSName=Plant OS
|OSDescription=A simple OS used to study made by several students
|Contact=Yan.Huang24@student.xjtlu.edu.cn
|License=Open source (MIT)
|URL=https://github.com/plos-clan/Plant-OS
|BootMedia=
|Status=Active
}}

{{OSProject
|OSName=PlusOS (Plus Operating System)
|OSDescription=Plus operating system is hobby project. That's built to run as auxiliary OS with Major OS like Windows and Linux etc..bootable with grub2 and own boot loader.focused on eBook reading software(PDF,EPUB,MOBI,DJVU,CHM...).with programs like calculator, notepad,source code editor, hex viewer, text viewer,image viewer and many games.VESA 32bpp GUI from startup.read-only plus maybe writing support for standard file systems FAT/NTFS/EXT/CDFS.codec for archives(ZIP,RAR,7z,GZ ...).codec for image formats(JPG,BMP,GIF,PNG...).video player or at least thumbnails extract(AVI,MKV,MP4,3GP...). mountable as filesystem from ISO,ISZ file.generic drivers VBE,USB,HD Audio,Keyboard, mouse.bootable on qemu and of course on real computer systems.at least all drivers for one computer (my) this is primary goal.
In summary goal is building a stable and usefull OS.
A Project By Muhammad Arshad Latti.
|Contact=arshadlatti@gmail.com
|URL=https://sourceforge.net/projects/plusos/
|BootMedia=ISO, ~4 MB
|Status=Last commit: 2020-09-24
}}

{{OSProject
|OSName=PrettyOS
|OSDescription=OS with a simple kernel created for educational purposes. A great care is put on keeping the code as readable as possible. PrettyOS is written in C and some i386 assembly (own bootloader). PrettyOS offers network, FAT12/16/32, uhci, ohci, ehci, xhci. My homepage: http://prettyos.de
|Contact=Dr. Erhard Henkes, http://prettyos.de
|License=Open source (BSD)
|URL=https://sourceforge.net/p/prettyos/code/HEAD/tree/
|Pic=[[File:PrettyOS.png|240px]]
|BootMedia=Floppy, 1.44 MB
|Status=Latest commit: 2017-06-17
}}

{{OSProject
|OSName=Project_DragonFly_VOS
|OSDescription=An Operating System development project that started back in 2013. This project's main objective is to create a minimal runtime environment that can be expanded by any willing programmer or power-user. The details of this project are open to public viewing. For more information about the intended design/architecture of this '''''incomplete''''' project, please click here: [https://tophatproductions115.tonidoid.com/app/websharepro/share/TXPVOSSystemArchitecture1964239483/] This project will move at a slow pace, due to prior obligations on the part of the developer. For license details and limitations please see this document: [https://drive.google.com/open?id=1o6xuWftTi33NpwSWdYGTcxHB6lxrOy-ILoOgxnhh9ME] Please note that code comments in the OS Development repo follow a semi-specific format, for ease of reading and refactoring. Links to the public repo can be found here: [https://drive.google.com/open?id=0B98YR-Jm_dNAYy1mM0g1LWtzVHM] The TXP-Network websites have been down for some time now, due to the loss of the last web-host. The current link will not work. 
|Contact=TopHatProductions115 [at] mail [dot] txp-network [dot] tk
|License=Open source (TXP-Network)
|URL=http:// txp-network [dot] tk/
|Status=Projected Release Year=TBA, Third rewrite and Research Phase, still in development. 
}}

{{OSProject
|OSName=Pure64
|OSDescription=Pure64 is a second stage bootloader for 64-bit PC's with compatible Intel or AMD processors. The loader gets the computer into a full 64-bit state with no legacy compatibility layers. Pure64 also enables and configures all available Cores/CPUs in the computer. An information table is stored in memory after Pure64 is finished that stores important details about the computer. Our old website (http://www.returninfinity.com/pure64.html) is dead
|Contact=Ian Seyler, https://github.com/IanSeyler , iseyler [at] returninfinity [dot] com
|License=Open source
|URL=https://github.com/ReturnInfinity/Pure64
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2024-05-15, is active
}}

=Q=

{{OSProject
|OSName=Quark Operating System
|OSDescription=Quark is an Object Oriented 32 bit operating system. It aims to be a complete multitasking kernel.
|Contact=Ananth Shrinivas, compstud2001 [at] yahoo [dot] com
|License=Open source
|URL=https://github.com/quark-os/quark-os
|BootMedia=ISO, ??? MB (expected to be of a "floppy size")
|Status=Last commit: 2017-09-19
}}

{{OSProject
|OSName=[[User:Rhodez/QuasiOS|QuasiOS]]
|OSDescription=QuasiOS is a x86 64-bit operating system which will be build from the ground up, currently in assembly and C, but will migrate to our own OSL (Operating System Language), when the compiler is ready. It have three main focus goals. A password capability-based system with cryptographic filesystem. A modularized kernel where the modules is hot swappable, hence updates can be made live. User friendly configuration and interaction.
Some of the work are going to be our master theses for the next year, and we expect rapid development during our theses (2020-2021).     
|Contact=contact [at] quasios [dot] com
|License=Open source (License will be updated at some point)
|URL=https://git.imada.sdu.dk/Sandsized/QuasiOS-64-bit
|BootMedia=Unknown
|Status=Last commit: 2020-03-10, can't find repository
}}

{{OSProject
|OSName=Quinn
|OSDescription=Quinn OS is a 32-bit x86 compatible protected mode hobby operating system with multitasking. There is a graphical user interface, FAT file system driver and basic TCP/IP stack.
|Contact=Andrew Pamment, apamment [at] yandex [dot] com
|License=Open source
|URL=https://github.com/apamment
|BootMedia=Unknown
|Status=Latest commit: 2017-10-08, can't find repository
}}

{{OSProject
|OSName=qword
|OSDescription=Kernel and distro written in C and x86 assembly targeted at x86_64. Our philosophy is "keep it simple and make it work", which seems to have been working so far.
|Contact=OS Development discord server (not associated with osdev.org), where most of the devs are: https://discord.gg/RnCtsqD
|License=Open source
|URL=https://github.com/qword-os/qword
|BootMedia=IMG, ??? MB (expected to be of a "floppy size")
|Status=Latest commit: 2020-11-19, repository archived
|Pic=[[Image:Qword1.png|240px]]
}}

=R=

{{OSProject
|OSName=RaidouOS
|OSDescription=x86_64 multitasking graphical operating system with support for PS/2, a round robin scheduler, and double-buffering for VESA 1280x1024 24-bit color mode. It has a custom font and simple terminal, as well as a basic desktop with window management. It can produce basic audio output through the PC speaker.
|Contact=aptrock327 on discord
|License=Open source (GNU GPLv2)
|URL=https://github.com/AptRock327/RaidouOS
|Status=Latest commit: 2024-02-09, is active
|Pic=[[Image:Raidou.png|240px]]
}}

{{OSProject
|OSName=RDOS
|OSDescription=32-bit x86 based OS written mostly in assembly. Provides protection with segmentation and paging. Has drivers for FAT-based file systems, TCP/IP, USB, sound, LFB based VESA support with a GUI API. The user-level API is based on C++ classes. OpenWatcom is used for building both applications and device-drivers. Stable, has some 1000 commercial installations
|Contact=Leif Ekblad, leif [at] rdos [dot] net
|License=Open source (GNU GPLv2) / Proprietary for commercial usage
|URL=http://www.rdos.net/svn/trunk
|Status=Latest commit: 2023-02, is active
}}

{{OSProject
|OSName=ReactOS
|OSDescription=A GPL project to clone WinNT written from scratch. It runs: Firefox, OpenOffice, Quake III Arena and much more. A lot of work is still need to be done. Looking for developers
|Contact=a team of developers, ros-general [at] reactos [dot] com
|License=Open source (GNU GPLv2)
|URL=http://www.reactos.com
|Status=Latest release: 2019-02-24, is active
}}

{{OSProject
|OSName=Reaver OS
|OSDescription=Open source project striving to provide a µkernel and a set of necessary services. Written in modern C++, currently targeting AMD64 SMP systems. Not POSIXish. Timer framework, IPI framework, thread switching are finished. There's also a basic, dumb scheduler without wait queues. Work progresses on enabling userspace and basic syscalls
|Contact=Michał "Griwes" Dominiak, griwes [at] griwes [dot] info
|License=Open source
|URL=https://github.com/griwes/ReaverOS
|Status=Last update: 2017-01-25
}}

{{OSProject
|OSName=Red OS
|OSDescription=32-bit microkernel designed for code readability. 
|Contact=nick [at] primis [dot] org
|License=Open source (BSD 2-clause)
|URL=https://github.com/primis/redos
|Status=Last update: 2016-04-28, abandoned
}}

{{OSProject
|OSName=Redox
|OSDescription=Redox is a microkernel-based operating system, a complete, fully-functioning, general-purpose operating system with a focus on safety, freedom, reliability, correctness, and pragmatism; wherever possible, the system components are written in Rust and run in user-space.
|Contact=https://matrix.to/#/#redox:matrix.org (Matrix Space).
|License=Open source (MIT)
|URL=https://redox-os.org
|Status=Latest update: Jun 14 2025
}}

{{OSProject
|OSName=Retro Rocket
|OSDescription=An x86_64 OS aiming to be like the microcomputer operating systems of the 80s but with modern enhancements. With SMP, Multitasking, Networking (TCP/IP), and graphics, C kernel and drivers with a userland completely written in its own BASIC dialect. Identity mapped and unrestricted, PEEK and POKE what you like, you break it, you keep it. Installable to hard disk, or bootable from CD/DVD. 
|Contact=info [at] retrorocket.dev
|License=Open source (Apache 2.0)
|URL=https://github.com/brainboxdotcc/retro-rocket/
|Status=Latest update: 2025-08-24, is active
}}

{{OSProject|OSName=racaOS|OSDescription=A framekernel-based operating system written in rust. It contains a framework called Zodiac. Zodiac offers basic functions like a microkernel, and the other parts of the kernel are written in safe rust, so that racaOS can be not only as safe as a microkernel-based OS, but also as  fast as a microkernel-based OS.|URL=https://github.com/zzjrabbit/racaOS|License=Open source (Apache)}}

{{OSProject
|OSName=RISC OS
|OSDescription=Designed in Cambridge, England by Acorn. First released in 1987, its origins can be traced back to the original team that developed the [http://www.arm.com/ ARM] microprocessor. Niche community using emulation, legacy systems and newer ARM hardware such as the [http://beagleboard.org BeagleBoard]. "[http://www.riscosopen.org/viewer/view/ Shared source]" fork ([http://www.castle-technology.co.uk/ Castle Technology Ltd]'s RO&nbsp;5) available for free non-commercial use, proprietary fork ([http://riscos.com RISCOS&nbsp;Ltd]'s RO&nbsp;4&nbsp;&amp;&nbsp;6) closed source. Stable, semi-defunct commercial use. Development currently focused on ARMv7 hardware
|Contact=RISC OS Open, https://www.riscosopen.org/content/contact
|License=Open source
|URL=http://riscosopen.org
|Status=Latest commit: 2019-02-25, is active
}}

{{OSProject
|OSName=rxv64
|OSDescription=rxv64 is a rewrite of xv6, targeting multicore 64-bit x86\_64 machines. It is written in Rust and while still a pedagogical system it supports PCIe, AHCI, the APIC, fast system calls, SYSCALLQ/SYSRETQ.
|Contact=Dan Cross, crossd [at] gmail [dot] com
|License=Open source (MIT)
|URL=https://github.com/dancrossnyc/rxv64
|Status=Latest commit: 2023-05-06, is active
}}

=S=

{{OSProject
|OSName=SeaOS
|OSDescription=SeaOS is a hybrid kernel with loadable modules that supports ATA, AHCI, EXT2, ELF, and many other fancy acronyms. It has basic networking support, initial VT-x support, and is self-hosting with a fairly complete unix-like userland. Designed for simplicity. My homepage: http://dbittman.github.io/seaos
|Contact=Daniel Bittman, danielbittman1 [at] gmail [dot] com
|License=Open source (GNU GPLv2)
|URL=https://github.com/dbittman/seakernel
|Status=Last commit: 2016-05-13, abandoned
}}

{{OSProject
|OSName=SerenityOS
|OSDescription=SerenityOS is a graphical Unix-like OS written in C++. It combines a 1990's style GUI with a modern CLI. Everything is from scratch, including a web browser with JavaScript and HTTPS support. A growing number of 3rd party packages are available as optional ports, including GCC, bash, vim, Python, SDL2, etc. You can watch videos of the system being developed on YouTube: https://www.youtube.com/c/AndreasKling.
|Contact=Andreas Kling, kling [at] serenityos [dot] org
|License=Open source (BSD 2-clause)
|URL=https://github.com/SerenityOS/serenity
|Pic=[[File:SerenityOS-8ea4375.png|200px]]
|Status=Last commit: Recently
}}

{{OSProject
|OSName=SerpaeOS
|OSDescription=SerpaeOS is a free, open-sourced x86 OS. It aims to provide a good resource for OSDevers to observe how certain features are implemented. Reach out to join the development team!
|License=GNU GPLv2
|Contact=Jaihson Kresak, serpaeos.devers [at] gmail [dot] com
|URL=https://serpaeos.sourceforge.io
|Status=Active
}}

{{OSProject
|OSName=Silcos
|OSDescription=Silcos is a operating system that aims to provide a secure & robust environment for applications. It will sandbox all external executable files and also have advanced security in matter of object-management. It is in the development phase with the kernel being developed. The silcos kernel works in a modules which are dynamically loaded at runtime and divides itself into various modules like KernelHost, ModuleFramework, ObjectManager, and ExecutionManager, etc. It requires open-source support and will appreciate any!

It is currently being developed in C++ and has a very object-oriented approach. Full documentation is provided in the code itself. Other than that, good GitHub wikis document it very well.
|Contact=Shukant Pal, sukantk3.4 [at] gmail [dot] com
|License=GNU GPLv3 (open-source)
|URL=https://github.com/SukantPal/Silcos-Kernel
|Status=Last commit: 2018-12-22, under rapid development, developers required
}}

{{OSProject
|OSName=SimpleOS,
|OSDescription=SimpleOS is a very simple Operating System coded mainly in C with a bit of Assembly. 
|Contact=Harvey Xing, xingharvey [at] gmail [dot] com
|License=Open source (MIT)
|URL=https://github.com/xing1357/SimpleOS/
|Status=Latest update: 2021-06-04, is active
}}

{{OSProject
|OSName=[https://wiki.osdev.org/User:Rhodez/SingOS SingOS]
|OSDescription=SingOS (Single Task Operating System) is a concept/pilot operating system that aims to do only one task at a time and provide all resources for that task. It is only in 16-bit real mode, with a single feature in 32-bit protected mode. The special thing is the version which is strictly using original IBM PC BIOS calls, and should be compliant with IBM PC 5150. [https://git.imada.sdu.dk/Sandsized/SingOS_Legacy_BIOS https://git.imada.sdu.dk/Sandsized/SingOS_Legacy_BIOS] This might be the only of the versions which would be active, since we have started to develop QuasiOS (which will be a full functional 64-bit OS). 
At the university we have a historical collection of computers, why we have fun to make a operating system which runs on those machines. It's mostly fun, and is not well developed yet.  

|Contact=contact [at] singos [dot] dk
|License=Open source
|URL=https://git.imada.sdu.dk/Sandsized/SingOS
|Status=Last commit: 2019-09-20, Only updated occasional
}}

{{OSProject
|OSName=[https://www.sivelkiria.org/ SivelkiriaOS]
|OSDescription=Sivelkiria is a new operating system at early stage of development. It aims to provide a platform which unites all software solutions into a harmonic whole, thus taking user experience to the next level. In Sivelkiria, compatibility of all programs is guaranteed by design. Using any device becomes easy and comfortable. The software designed for Sivelkiria OS can be used in any context. 

|Contact=contact hello@sivelkiria.org
|License=Open source (BSD 2-clause "Simplified" License)
|URL=https://git.sivelkiria.org/
|Status=Active
}}

{{OSProject
|OSName=skiftOS
|OSDescription=skiftOS is a hobby operating system built for learning and fun targeting the x86 platform. It features a kernel named hjert, a graphical user interface with a compositing window manager, and familiar UNIX utilities.
|Contact=Nicolas Van Bossuyt, nicolas.van.bossuyt [at] skiftos [dot] org
|License=Open source (MIT)
|URL=https://github.com/skiftOS/skift
|Pic=[[File:skiftOS.png|200px]]
|Status=Last commit: Recently
}}

{{OSProject
|OSName=Snowdrop OS
|OSDescription=Snowdrop OS is a small-scale 16-bit real mode operating system for the IBM PC architecture. Snowdrop was developed from scratch, using only assembly language. Snowdrop boots from a FAT12 filesystem (floppy disk) and comes with a shell, aSMtris (a Tetris clone), and a few other example programs that could greatly simplify the development -  for example, see http://sebastianmihai.com/snowdrop/gamedev.php
|Contact=email address on website below
|License=Open source (Public Domain)
|URL=http://sebastianmihai.com/snowdrop/
|Status=Latest update: 2017-07-21, is active
|Pic=[[File:SnowdropOS.png|240px]]
}}

{{OSProject
|OSName=SO3
|OSDescription=Smart Object Oriented Operating system is a compact, lightweight, full featured and extensible operating system particularly well-suited for embedded systems in general.
|Contact=https://gitlab.com/smartobject/so3
|License=Open Source (GPL2)
|LastReleaseYear=2021
|LastReleaseMonth=3
|LastReleaseDay=05
|LastReleaseDesription=2021.4
|URL=https://gitlab.com/smartobject/so3
}}

{{OSProject
|OSName=Sortix
|OSDescription=Sortix is a small self-hosting operating-system aiming to be a clean and modern POSIX implementation. It is a hobbyist operating system written from scratch with its own base system, including kernel and standard library, as well as ports of third party software. It has a straightforward installer and can be developed under itself. Releases come with the source code in /src, ready for tinkering.
|Contact=https://sortix.org/
|License=Open source (ISC)
|LastReleaseYear=2019
|LastReleaseMonth=1
|LastReleaseDay=03
|LastReleaseDescription=1.0
|URL=https://sortix.org/
|Pic=[[File:Sortix.png|200px]]
}}

{{OSProject
|OSName=Soso
|OSDescription=Soso is a Unix-like 32bit operating system. Its features are multitasking with processes and threads, paging, Virtual File System, FAT32, system calls, basic Musl libc port, userspace ELF files, framebuffer (/dev/fb0), mmap, PS/2 mouse, Unix local sockets, and shared memory.
|License=Open source (BSD 2-clause)
|URL=https://github.com/ozkl/soso
|Pic=[[File:Soso-v0.3-gears.png|200px]]
|Status=Last commit: 2021-03-15, in active development
}}

{{OSProject
|OSName=SymbOS
|OSDescription=SymbOS is a multitasking graphical OS for 8-bit computers using the Z80 CPU. It aims to bring a Windows 95-like experience to hardware from the 1980s, including binary software compatibility across a range of originally incompatible computers (Amstrad CPC/PCW/NC, MSX, Enterprise 128, and more). Both original hardware (printers, sound) and modern hobbyist expansions (FAT mass storage, networking) are supported through device drivers. A growing library of userspace applications have been written using Z80 assembly and two native development environments, the Quigs IDE and a self-hosting native C compiler.
|Contact=Jörn Mika, http://www.prodatron.net/
|License=Mixed open/closed source
|URL=http://symbos.org/
|Status=Active
|Pic=[[File:symbos.gif|200px]]
}}

=T=

{{OSProject
|OSName=TermuOS
|OSDescription=TermuOS is an open-source x86_64 operating system focused on a Linux-inspired terminal environment with multitasking, memory management, a virtual file system, networking support, and a custom shell. The project is written primarily in C and x86_64 assembly and is designed for learning operating system development and low-level programming. Our site: termuos.netlify.app
|Contact=Ronnie Harrod, termukernel@gmail.com
|License=Open source (MIT)
|URL=https://github.com/TermuOS-Project/TermuOS
|BootMedia=termuos.iso 6.1MB
|Status=Lastest commit: 2026-09-23
}}

{{OSProject
|OSName=tachyon
|OSDescription=tachyon is another Hobby OS, longing to support x86_64 only. Currently, it boots on qemu, bochs, virtualbox and real hardware. it has not much to see, really, but a working physical and virtual memory management (still improving), kernel and user threads and some other hardware interfacing... Basic goal was to create everything from scratch with a clean code base. It uses a plugin mechanism to load different supported kernel components.
|Contact=mduft on GitHub
|License=Open source (MIT)
|URL=https://github.com/mduft/tachyon3
|Status=Latest update: 2023-02-06
}}

{{OSProject
|OSName=tatOS
|OSDescription=A 32bit x86 OS written in assembly featuring a protected mode driver for USB flash drive and mouse. Source package includes tedit editor and ttasm assembler. Supports UHCI, EHCI, PS2 keyboard and 800x600x8bpp graphics - has a basic GUI
|Contact=Tom Timmermann
|License=Open source
|URL=https://github.com/tatimmer/tatOS
|Status=Latest commit: 2016-07-23, may be abandoned
|Pic=[[File:TatOS.png|200px]]
}}

{{OSProject
|OSName=[[wikipedia:TempleOS|TempleOS]]
|OSDescription=TempleOS is a 64-bit, lightweight OS with multitasking and multicore support. It is ring-0-only and works in a single address space. The entire OS and all of its software was created over 15 years by Terry A. Davis alone. Mr. Davis also developed a programming language (Holy C) together with a compiler for it during this time. TempleOS supports the FAT32 and RedSea filesystems (the latter created by Mr. Davis), and also file compression. A networking stack is missing, partially for ideological reasons, but there are forks available with added functionality.
|Contact=Terry A Davis, http://www.templeos.org
|License=Open source (Public Domain)
|URL=http://templeos.org/
|Status=([http://www.thedalleschronicle.com/news/2018/sep/07/man-killed-train-had-tech-following/ Author passed away])
|Pic=[[File:TempleOS.png|240px]]
}}

{{OSProject
|OSName=Tilck
|OSDescription=Tilck ('''Ti'''ny '''L'''inux-'''C'''ompatible '''K'''ernel) is a x86 monolithic preemptable kernel designed to be partially compatible with <code>linux-i686</code> at binary level. For the moment, it's mostly an educational project: the perfect playground for playing in kernel mode while retaining the ability to compare how the very same usermode bits run on the Linux kernel as well. In the long term, the project targets the '''embedded''' world and '''ARM''', including MMU-less systems. Currently, the project has full '''ACPI''' support and can run software like '''BusyBox''', '''VIM''', '''fbDOOM''', '''Lua''', '''TinyCC''', without any customizations. Part of the project are also an interactive '''bootloader''' (both legacy '''BIOS''' and '''UEFI''' boot) and a ''solid'' '''test infrastructure''', with extras like '''kernel coverage'''.
  
|Contact=vladislav [dot] valtchev [at] gmail [dot] com
|License=Open source (BSD 2-clause)
|URL=https://github.com/vvaltchev/tilck
|Status=Latest commit: 2021-08-10, is active
|Pic=[[Image:Tilck_screenshot.png|200px]]
}}

{{OSProject
|OSName=TinyOS
|OSDescription=Event-driven operating system targeted for wireless sensor network embedded systems. Has a megalithic kernel architecture
|Contact=https://github.com/tinyos/tinyos-main
|License=Open source (BSD)
|URL=https://github.com/tinyos/tinyos-main
|Status=Latest commit: 2018-12-13, is active
}}

{{OSProject
|OSName=[[User:Klange/ToaruOS|ToaruOS (とあるOS)]]
|OSDescription=A complete educational operating system for x86-64 PCs, with a bootloader, kernel, libc, window compositor, graphical applications, and Unix-like utilities.
|Contact=#toaruos on libera.chat
|License=Open source (NCSA/University of Illinois license)
|URL=http://github.com/klange/toaruos
|Status=Latest commit: 2023-01-05, is active
|Pic=[[Image:Toaruos_screenshot.png|200px]]
}}

{{OSProject
|OSName=TravorOS
|OSDescription=A research-purpose Operating System that discovers features on Intel x86 processors which is inspired by Linux. Welcome to contribute!
|Contact=Travor Liu  <travor_lzh [at] outlook [dot] com>
|License=Open source (GNU GPLv3)
|URL=https://github.com/TravorLZH/TravorOS
|Status=Latest commit: 2018-08-07, abandoned
|Pic=[[Image:TravorOS_20180422.png|200px]]
}}

{{OSProject
|OSName=Tupai
|OSDescription=Tupai is a monolithic unix-inspired operating system kernel that runs on i386-compatible machines.
|Contact=Joshua Barretto, joshua.s.barretto [at] gmail [dot] com
|License=Open source (GNU GPLv3)
|URL=https://gitlab.com/zesterer/tupai/commits/dev
|Status=Latest commit: 2017-08-03
|Pic=[[Image:Tupai-0-1-0.png|200px]]
}}

{{OSProject
|OSName=týndur
|OSDescription=The microkernel-based community OS of the German OS development community Lowlevel. See the [http://forum.osdev.org/viewtopic.php?f=2&t=21355 tyndur 0.2.2 announcement] in the forum
|Contact=tyndur-devel mailing list, tyndur-devel [at] tyndur [dot] org ; Kevin at OSDev forums
|License=Open source
|URL=https://git.tyndur.org/lowlevel/tyndur/commits/master
|Status=Latest update: 2018-01-06
}}

{{OSProject
|OSName=Tysos
|OSDescription=The type-safe operating system is a 64-bit multitasking microkernel written in C#. Ahead-of-time compiler is mostly complete, work has switched to implementing the kernel
|Contact=John Cronin, jncronin [at] tysos [dot] org
|License=Open source
|URL=https://github.com/jncronin/tysos
|Status=Last update: 2021-02-01
}}

=U=

{{OSProject
|OSName=Uinxed-Kernel
|OSDescription=Uinxed is a Unix-like operating system kernel developed from scratch, focusing on modern computer architecture and advanced system design concepts. The project aims to build an efficient, stable, and scalable operating system kernel while maintaining code clarity and maintainability.
|Contact=2609948707@qq.com
|License=Open source (Apache 2.0)
|URL=https://github.com/ViudiraTech/Uinxed-Kernel
|Status=Active
|Pic=[[File:UX20260722-234105.png|200px]]
}}

{{OSProject
|OSName=Unet Operating System
|OSDescription=Unet is a Unix-like operating system that offers the best compatibility for various platforms. Whether you need a server, a desktop, or an embedded system, Unet can meet your needs. Unet also provides advanced networking, security, and storage features, and supports most popular embedded storage and networking devices.
|Contact=lithicsoft@gmail.com or https://discord.gg/7VxhnqeSUf
|License=Proprietary
|URL=
|Status=Active
}}

{{OSProject
|OSName=[[User:Superleaf1995/UDOS|UDOS]]
|OSDescription=A mainframe OS - yes it's compatible with z/Arch albeit lacks SYSG support.
|Contact=Superleaf1995
|License=Open source (Public Domain - Unlicense)
|URL=https://github.com/udos-project/KinnowOS
|Status=Latest commit: 2022-12-18
|Pic=[[File:uDOS.png|200px]]
}}

{{OSProject
|OSName=[[User:MrOlsen7905/uBixOS|uBixOS]]
|OSDescription=A hobby/research x86 (i386) operating system written from scratch in C/C++ since 2002. Monolithic kernel with an O(1) SMP scheduler, paged copy-on-write VM, a VFS with its own native copy-on-write filesystem (UbixFS), lwIP networking, a FreeBSD-derived POSIX userland (musl-linked, tcsh), and a composited shared-memory window system (views).
  Boots via GRUB2/multiboot on bare metal or QEMU.
|Contact=MrOlsen7905
|License=Open source (BSD 3-Clause)
|URL=https://github.com/cwolsen7905/UbixOS
|Status=Active — v2.3.0-BETA, latest commit: 2026-06-08
|Pic=[[File:uBixOS.png|200px]]
}}

{{OSProject
|OSName=Ultibo core
|OSDescription=A unikernel environment based on the Free Pascal compiler and Lazarus IDE, initially targeting single board computers such as Raspberry Pi and also supporting QEMU the design is intended to be portable to other platforms. The modular architecture allows applications to pick and choose what features to use in a project and the compiler produces a bootable kernel image which includes all of the required RTL components. Comes with a comprehensive list of features including pre-emptive threading, multicore support, IPv4 networking, FAT/NTFS/CDFS file systems, USB support, SD/MMC support, drivers for common peripherals such as GPIO, I2C, SPI, PWM, and DMA, C library support, hardware accelerated OpenGL ES and OpenVG graphics and much more. Packaged in an installer download for Windows or as an install script for Linux customized versions of both Free Pascal and Lazarus IDE are included along with full source and a large collection of examples. Our homepage: https://ultibo.org/
|Contact=info [at] ultibo [dot] org
|License=Open source (GNU LGPL 2.1 with static linking exemption)
|URL=https://github.com/ultibohub/
|Status=Latest commit: 2021-05-14, is active
}}

{{OSProject
|OSName=Upanix
|OSDescription=Upanix – an x86 based 32 bit operating system designed and developed in C++. It uses x86 architectural features for memory management (paging, segmentation) and process management (TSS, Call Gates, Interrupt Gates). Aim is to build a minimal complete OS which supports USB storage, ELF binaries/dlls, GCC/g++ compiler and build tools, Internet (wifi) and a basic UI
|Contact=Prajwala Prabhakar, srinivasa_prajwal [at] yahoo [dot] co [dot] in
|License=Open source (GNU GPLv3)
|URL=https://github.com/prajwal83/upanix
|Status=Latest commit: 2018-06-11
}}

=V=

{{OSProject
|OSName=Vanadium OS
|OSDescription=Vanadium OS is a multi-user Unix-like OS. i386, amd64. Fully protected mode. Memory, file and hardware protection. root and unprivileged users. GUI without X, modular kernel, and has a live floppy image with a single user mode! My homepage: http://www.durlej.net/contact ; project pages: http://www.durlej.net/v/ , http://www.durlej.net/nameless , https://github.com/p-durlej/newsys . One of the great features is that you can write C code and instantly compile it to execute
|Contact=Piotr Durlej, http://www.durlej.net/contact
|License=Open source (BSD 2-clause)
|URL=https://github.com/p-durlej/newsys
|Status=Abandoned
|Pic=[[File:VanadiumOS.png|240px]]
}}

{{OSProject
|OSName=Vinix
|OSDescription=Vinix is a 64-bit UNIX-like OS written in the [https://vlang.io/ V programming language], that aims to be source compatible with most Linux/*nix programs. It aims to be approachable and be runnable on real hardware. It can run a vast array of Linux userland software including bash, GCC, and X.org, amongst others.
|License=Open source (GNU GPLv2)
|URL=https://github.com/vlang/vinix
|Status=Active
|Pic=[[File:Vinix.png|240px]]
}}

{{OSProject
|OSName=Visopsys
|OSDescription=Visopsys (VISual OPerating SYStem) is an alternative operating system for PC-compatible computers, written “from scratch”, and developed primarily by a single hobbyist programmer since 1997. Its primary "useful feature" is a reasonably functional partition management program - the ‘Disk Manager’, which can create, format, delete, resize, defragment, copy, and move partitions, and modify their attributes. It supports both DOS/MBR and UEFI partition tables.  It can also copy hard disks, and has a simple and friendly graphical interface.  A basic version can fit on a bootable floppy disk, or you can use the entire system from a ‘live’ CD/DVD
|Contact=Andy McLaughlin, andy [at] visopsys [dot] org
|License=Open source (GNU GPLv2)
|URL=http://visopsys.org/
|Status=Last commit: 2023-09-21, is active
|Pic=[[File:Visopsys.png|240px]]
}}

{{OSProject
|OSName=VSTa
|OSDescription=VSTa is a copylefted system, originally written by Andrew Valencia, which uses ideas from several research operating systems in its implementation. It attempts to be POSIXish except where POSIX gets in the way, and runs on a number of different PC configurations. VSTa is also designed to take advantage of SMP right out of the box. My homepage: http://www.vsta.org
|Contact=Andrew Valencia, https://github.com/vandys
|License=Open source (GNU GPLv2)
|URL=http://sources.vsta.org:7100/vsta/index
|Status=Last commit: 2015-01-12, abandoned
}}

= W=

{{OSProject
|OSName=WingOS
|OSDescription=WingOS is an basic open source 64bit Operating System written from scratch with c++ and a little bit of assembly. WingOS has SMP support.
|License=open source (MIT)
|URL=https://github.com/Supercip971/WingOS_x64/
|Status=Latest commit: 2020-12-24, is active
}}

=X=

{{OSProject
|OSName=XJ380 OS
|OSDescription=XJ380 is a 64-bits graphical operating system. It was launched in 2022 by XINGJI Studios(XINGJI Interactive Software). It will continue to be developed. And on August 1, 2026, they open-sourced the system's kernel.
|Contact=guoqiyu1115@outlook.com
|License=Open Source (Without GUI) (Apache 2.0)
|URL=https://github.com/xingji-studio/OpenXJ380
|Status=Active
}}

{{OSProject
|OSName=XEOS
|OSDescription=XEOS is an experimental 32/64 bits Operating System for x86 platforms, written from scratch in Assembly and C. It includes a C99 Standard Library, and aims at POSIX/SUS2 compatibility.
* Status: http://www.xs-labs.com/en/projects/xeos/status/
* Roadmap: http://www.xs-labs.com/en/projects/xeos/roadmap/
* GitHub: http://github.com/macmade/XEOS
* API reference: http://doc.xs-labs.com/XEOS
|Contact=Jean-David Gadina - XS-Labs - www.xs-labs.com
|License=Open source
|URL=http://www.xs-labs.com/en/projects/xeos/
|Status=Latest commit: 2017-01-11
}}

{{OSProject
|OSName=xOS
|OSDescription=xOS is a hobbyist operating system project written mostly from scratch for the PC, entirely in FASM-style assembly. The principal goal of xOS is to be compact and lightweight, yet be suitable for use in the 21st century, in terms of speed, features and hardware support. As such, xOS supports IDE and SATA hard disks, partial support for USB, networking, a rudimentary web browser, and a compositing graphical user interface. The kernel and drivers have been written entirely in assembly, while applications can be written in C.
|Contact=omarx024 [at] gmail [dot] com
|License=Open source (MIT)
|URL=http://omarrx024.github.io/ (link is dead)
|Status=Latest commit: 2017-07-09, possibly abandoned
}}

{{OSProject
|OSName=xv6
|OSDescription=xv6 is a re-implementation of Dennis Ritchie's and Ken Thompson's Unix
Version 6 (v6).  xv6 loosely follows the structure and style of v6,
but is implemented for a modern RISC-V multiprocessor using ANSI C.
* Open source of companion text: https://github.com/mit-pdos/xv6-riscv-book
* Port of xv6 to VisionFive 2 SoC: https://github.com/michaelengel/xv6-vf2
|URL=https://github.com/mit-pdos/xv6-riscv
|License=Open source (MIT)
|Status:Used in MIT OS courses (current)
}}

{{OSProject
|OSName=xv6-64
|OSDescription=xv6-64 is a port of MIT xv6 for the x86-64
|URL=https://xv6-64.gitlab.io/xv6-64/
|License=Open source (MIT)
|Status:Used in some OS courses (current)
|Contact=pda+xv6@unistra.fr}}

{{OSProject
|OSName=XtOS
|OSDescription=XtOS is an operating system with a kernel written in C and a userspace written in C source code which is interpreted on runtime. The C interpreter is based on TinyC and implemented inside the kernel along with the window manager. Everything runs in Ring 0 and C applications are able to access most of the functions that the kernel is able to.
|Contact=ch@dorper.me
|License=Open source
|URL=https://git.dorper.me/projects/XT (link is dead)
|Status=Latest commit: 2018-08-24
}}

=Y=

{{OSProject
|OSName=YaxOS
|OSDescription=YaxOS is a very primitive OS running in real mode.
|Contact=markboldyrev+yaxos@gmail.com
|License=GPLv3
|URL=https://gitlab.com/SopaXorzTaker/yaxos
|Status=Latest commit: 2018-12-26, is active
}}

=Z=

{{OSProject
|OSName=Zeal 8-bit OS
|OSDescription=Conceived as a hardware abstraction layer for Z80-based computers, Zeal 8-bit OS is a single-tasking operating system with a small, concise Unix-like API. It is ROM-able, modular and configurable. Header files are included for assembly language and C. Videos for Zeal 8-bit OS and Computer: https://www.youtube.com/@Zeal8bit Web emulator for Zeal 8-bit Computer: https://zeal8bit.github.io/Zeal-WebEmulator/
|Contact=contact [at] zeal8bit [dot] com
|License=Open source (Apache 2.0)
|URL=https://github.com/Zeal8bit/Zeal-8-bit-OS
|Status=Latest update: 2023-04-05, is active
}}

{{OSProject
|OSName=ZeldaOS
|OSDescription=Written in C from scratch, ZeldaOS is an UNIX-like kernel which is aiming for POSX.1 compliance and running on any x86 and x86_64 processors.  Also, the kernel is designed to support more and more networking features.
|Contact=jiezheng@vmware.com
|License=Proprietary
|URL=https://github.com/chillancezen/ZeldaOS
|Status=Latest update: 2019-05-26, is active
}}

{{OSProject
|OSName=ZeldaOS.x86_64
|OSDescription=Written in C from scratch, ZeldaOS.x86_64 is a 64_bit kernel which supports x86_64 features, it's also a bare metal hypervisor with Intel VT-x technology.
|Contact=jiezheng@vmware.com
|License=Proprietary
|URL=https://github.com/chillancezen/ZeldaOS.x86_64
|Status=Latest update: 2019-06-25, is active
}}

{{OSProject
|Contact=diaviekone13@gmail.com
|License=Open Source (Apache 2.0 and CC BY 4.0)
|URL=https://zxfoundation-devlog.vercel.app
|Status=Latest update: 2026-05-31 (Active)
|OSName=ZXFoundation™
|BootMedia=3390 (E)CKD Disk Image (~2M CCKD, ~600M raw)
|OSDescription=ZXFoundation™ is a DGP (Domain/Gate/Portal) kernel that exclusively targets z/Architecture. A completely different design, no POSIX-compliance planned, isolated architecture, deeply integrated with z/Architecture specifics. Then project serves primarily as a PoC, and is intended for research and educational purposes. It aims to be a performant kernel that offers high stability for mainframes. And moreover a (working) proof that z/Architecture is a facinating world, not just tied to big corps and IBM themselves. Written completely in modern C++23 modules. It also implements its own IPL loader, ZXFoundationLoader™-a highly capable loader with many assistance and can be used in a kernel barebones if you are interested in the architecture.
}}

{{OSProject
|OSName=ZeSystem
|Contact=ZeSystemInc
|URL=https://github.com/ZeSystem-Inc/Z.eSystem-Plus-3SU
|License=MPL 2.0
|OSDescription=Z.eSystem is an operating system built from scratch using C++, featuring a custom file system and no external dependencies.
|BootMedia=raw disk image
|Status=Lastest update: 11-07-2026 (Active)
}}
[[Category:Resources]]
````

<a id="w001368"></a>

## W001368 — Resources

Bron-ID: W001368
Namespace: Main (0)
Revisie: 28287 | Datum: 2023-08-20T09:46:01Z
Bron: https://wiki.osdev.org/Resources
Revisie-URL: https://wiki.osdev.org/index.php?title=Resources&oldid=28287

````text
This page maintains a list of resources related to operating system development. Often, links become outdated or sites go down. If a link does not work, try to see if it was backed up on archive.org.

== Operating System Development ==
* [http://www.osdever.net/ Bonafide OS Development]
* [https://web.archive.org/web/20130119090155/http://www.nondot.org/sabre/os/articles The Operating System Resource Center]
* [http://www.acm.uiuc.edu/sigops/roll_your_own/ How to Write an Operating System]
* [https://www.devimperium.com/tutorials/start_your_small_operating_system_in_assembly Start your small Operating System in Assembly]
* [http://neuraldk.org/writing.php?FilterCat=Os+Development&FilterDev=All NeuralDK OSDev Tuts]
* [http://www.brokenthorn.com/Resources/OSDevIndex.html BrokenThorn OS Development Tutorials]
* [https://web.archive.org/web/20130119090155/http://www.nondot.org/sabre/os/articles/MiscellaneousSoftware/ OS FAQ, overall design, VM design, threads..]
* [https://web.archive.org/web/20130119090155/http://www.nondot.org/sabre/os/articles/MemoryManagement/ Memory Management]
* [https://web.archive.org/web/20130119090155/http://www.nondot.org/sabre/os/articles/ProtectedMode/ Protected Mode]
* [http://www.dtweed.com/circuitcellar/ Circuit Cellar Ink, an embedded systems development magazine]
* [http://createyourownos.blogspot.com/ Create Your Own OS (by Pritam Zope)]
* [https://github.com/dreamos82/Osdev-Notes Osdev Notes - A set of notes on how to write an operating system]

== Hardware Specifications ==
* [http://perso.wanadoo.fr/pierrelib/index.html Pierre's Library]
* [http://www.mcamafia.de/pdf/pdfref.htm List of PS/2 documents]
* [http://datasheets.chipdb.org/ ChipDB datasheet archive]
* [https://web.archive.org/web/20160313204854/https://freemicronix.org/docs/ FreeMicroNix Documentation Collection]

=== Processors ===
* [https://software.intel.com/en-us/articles/intel-sdm/ Intel Manuals]
* [https://web.archive.org/web/20130119090155/http://www.nondot.org/sabre/os/articles/ProcessorArchitecture/ Processor Architecture]
* [http://developer.amd.com/resources/developer-guides-manuals/ x86-64(AMD64)]

=== BIOS ===
* [https://web.archive.org/web/20100102024125/http://phoenix.com/NR/rdonlyres/56E38DE2-3E6F-4743-835F-B4A53726ABED/0/specsbbs101.pdf BIOS Boot Specification]
* [https://web.archive.org/web/20180202045759/http://download.intel.com/support/motherboards/desktop/sb/pnpbiosspecificationv10a.pdf Plug and Play BIOS Specification]

=== Motherboard Devices ===
* [https://web.archive.org/web/20130119090155/http://www.nondot.org/sabre/os/articles/MiscellaneousDevices/ PIC, PIT, DMA, RTC, ..]

=== Human Input Devices ===
* [http://www.ti.com/lit/ds/symlink/pc16550d.pdf Serial Mouse]
* [https://web.archive.org/web/20130119090155/http://www.nondot.org/sabre/os/articles/HumanInterfaceDevices/ Human Interface Devices]
* [[Topic:8323|PS/2 mice]]
* [[Topic:10247|PS2 mouse driver]]

=== Video ===
* [http://www.vesa.org official VESA & VBE website]

=== Storage ===
* [http://www.t13.org official ATA working group]
* [http://www.ata-atapi.com/ another ATA/ATAPI website, includes reference driver]
* [https://web.archive.org/web/20091027131605/http://geocities.com/SiliconValley/2072/atapi.htm an introduction to ATA/ATAPI commands]. It contains a couple of ASM examples to read sectors and identify drives.
* [https://web.archive.org/web/20130119090155/http://www.nondot.org/sabre/os/articles/DiskandDiscDrives/ Disk and Disc Drives]
* [[Topic:8339|collected links about Ultra DMA, on ATA drives and how PCI configuration comes.]]

=== Audio ===
* [https://web.archive.org/web/20130119090155/http://www.nondot.org/sabre/os/articles/SoundDevices/ Sound Devices]

=== Network and Communication ports ===
* [https://web.archive.org/web/20130119090155/http://www.nondot.org/sabre/os/articles/CommunicationDevices/ Serial and Parallel port, SPP, EPP, ECP]
* [https://web.archive.org/web/20130119090155/http://www.nondot.org/sabre/os/articles/NetworkingDevices/ Networking Devices]
* [http://pdf1.alldatasheet.com/datasheet-pdf/view/84677/ETC/RTL8139/+5_5J8UCRC/1SuMCAY.+/datasheet.pdf RTL8139 family datasheet]

=== Buses ===
* [https://web.archive.org/web/20130119090155/http://www.nondot.org/sabre/os/articles/InterconnectBuses/ Interconnect Buses]
* [http://www.usb.org/developers/docs official USB developers docs]z
* [https://retired.beyondlogic.org/usb/usbdevdrvs.htm more USB info]
* [http://www.o3one.org/hwdocs/usb/hcir1_0a.pdf USB OHCI]
* [https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/ehci-specification-for-usb.pdf USB EHCI]
* [http://download.microsoft.com/download/1/6/1/161ba512-40e2-4cc9-843a-923143f3456c/ohci_11.pdf OHCI for FireWire] 
* [http://www.o3one.org/hwdocs/usb/hcir1_0a.pdf OHCI mirror]

== Software Specifications ==
* [http://www.delorie.com/djgpp/doc/rbinter/ix/ Ralf Brown's Interrupt List]
* [https://web.archive.org/web/20130119090155/http://www.nondot.org/sabre/os/articles/TheBootProcess/ The Boot Process]
* [https://web.archive.org/web/20130119090155/http://www.nondot.org/sabre/os/articles/Partitions/ Partitions]
* [https://web.archive.org/web/20130119090155/http://www.nondot.org/sabre/os/articles/FileSystems/ File Systems]
* [https://web.archive.org/web/20130119090155/http://www.nondot.org/sabre/os/articles/ExecutableFileFormats/ Executable File Formats]
* [https://web.archive.org/web/20130119090155/http://www.nondot.org/sabre/os/articles/PlugandPlaySpecs/ Plug'n'Play Specs]
* [https://web.archive.org/web/20130119090155/http://www.nondot.org/sabre/os/articles/DeviceDriverInterfaces/ Device Driver Interfaces]
* [http://home.teleport.com/~brainy/fatgen102.pdf FAT12-32]
* [http://www.amazon.com/Art-Software-Security-Assessment-Vulnerabilities/dp/0321444426 The Art of Software Security Assessment]
* [http://www.amazon.com/Hacking-Art-Exploitation-Jon-Erickson/dp/1593271441/ref=pd_bbs_sr_1?ie=UTF8&s=books&qid=1210492590&sr=1-1 Art of Exploitation]
* [http://www.amazon.com/Reversing-Secrets-Engineering-Eldad-Eilam/dp/0764574817/ref=pd_bbs_sr_1?ie=UTF8&s=books&qid=1210492638&sr=1-1 Secrets of Reverse Engineering]
* [http://www.amazon.com/Rootkits-Subverting-Addison-Wesley-Software-Security/dp/0321294319/ref=pd_bbs_3?ie=UTF8&s=books&qid=1210492638&sr=1-3 Subverting the Windows Kernel]

== Other Specifications == 
* [http://alexfru.narod.ru/emiscdocs.html stockpile of technical documents (partly in Russian)]
* [https://web.archive.org/web/20130119090155/http://www.nondot.org/sabre/os/articles/OtherHardware/ Legos, ATX, NetPC's..]

== Source Code ==
* [http://files.osdev.org/mirrors/geezer/osd/snippets.htm OSD Code Snippet Index]
* [http://www.singlix.org/trdos/specs.html stockpile of documents at singlix.org]
* [http://gee.cs.oswego.edu/pub/misc/malloc.c a reference implementation for malloc()]

== University Courses ==
* [https://web.archive.org/web/20090507122053/http://www.scs.cs.nyu.edu/aos/ Advanced Operating Systems (NYU)]
* Operating System Engineering (MIT) [http://pdos.csail.mit.edu/6.828/2004/ (2004)] [http://ocw.mit.edu/OcwWeb/Electrical-Engineering-and-Computer-Science/6-828Fall-2006/CourseHome/index.htm (2006)]
* [http://pages.cs.wisc.edu/~solomon/cs537.html Introduction to Operating Systems (WISC)]

[[Category:Resources]]
````

<a id="w001369"></a>

## W001369 — OSDev Wiki:Sandbox

Bron-ID: W001369
Namespace: OSDev Wiki (4)
Revisie: 28523 | Datum: 2023-12-15T19:25:03Z
Bron: https://wiki.osdev.org/OSDev_Wiki:Sandbox
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDev_Wiki%3ASandbox&oldid=28523

````text
====Interwiki Link====

Testing new interwiki mapping

*[[wikipedia:List_of_operating_systems]]
*[[wikibooks:C Programming]]

====Lists====
<br />
* '''OS Name''' - Title or short description. 
** Architecture: x86
** Languages: Assembly, C
** Source code: Yes,GPL
** URL: [http://www.osdev.org http://www.osdev.org]
<br />
* '''OS Name''' - Title or short description. 
** Contact: Chase (chase AT osdev.org)
** URL: [http://www.osdev.org http://www.osdev.org]

#Numbered List
##Item
##Item
#Another Item
##SubItem
##Are there ones that do Roman, Number and Letter?

====Link Types====

* Forum Link: [[forum:8]]
* Forum topic link: [[topic:677]]
* Wiki link: [[Main_Page]]
* Wiki link with custom label: [[Main_Page|OSDev Home]]
* Wiki link with anchor: [[Tool_Comparison#x86_Assemblers]]
* Wiki link with anchor and custom label: [[Tool_Comparison#x86_Assemblers|Assembler Comparisons]]
* External Link: [http://www.osdev.org OSDev]

====Subpages====
[[OSDev_Wiki:Sandbox/Subpage|Sub-page]]

[[OSDev_Wiki:Sandbox/Subpage/Subsubpage|Sub-sub-page]]

====Code in a Box====

 #include <kmem.h>
 
 extern void* kmalloc(int size)
 {
      return getFreeMemPtr(size);
 }

====Better Code in a Box====

<pre>
#include <kmem.h>
 
extern void* kmalloc(int size)
{
     return getFreeMemPtr(size);
}
</pre>

====Best Code in a Box====

<source lang="c">
#include <kmem.h>
 
extern void* kmalloc(int size)
{
     return getFreeMemPtr(size);
}
</source>

====Table 1====

{| {{wikitable}}
|-
! Column 1
! Column 2
|-
| Item
| Item
|-
| Item 2
| Item 2
|-
|}

====Advanced Table====

{| {{wikitable}}
|-
! 
! Column 1
! Column 2
! Column 3
|-
| Item
| {{No}}
| {{Yes}}
| {{No}}
|-
| Item 2
| {{Yes}}
| {{Yes}}
| {{Yes}}
|}

====Complex Table====

{| {{wikitable}}
|-
! rowspan=2 |
! colspan=2 | Column 1
! rowspan=2 | Column 2
|-
! Foo1
! Foo2
|-
| Item 2
| {{Yes}}
| {{Yes}}
| {{Yes}}
|}
````

<a id="w001372"></a>

## W001372 — Talk:List of OS resources

Bron-ID: W001372
Namespace: Talk (1)
Revisie: 1375 | Datum: 2006-06-17T15:01:54Z
Bron: https://wiki.osdev.org/Talk:List_of_OS_resources
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AList_of_OS_resources&oldid=1375

````text
Should we keep with the names given by the authors of the pages? That's what I did; maybe we should write desciptions?
````

<a id="w001374"></a>

## W001374 — Talk:Categorized Main Page/archive

Bron-ID: W001374
Namespace: Talk (1)
Revisie: 10230 | Datum: 2010-05-04T16:27:27Z
Bron: https://wiki.osdev.org/Talk:Categorized_Main_Page/archive
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ACategorized_Main_Page%2Farchive&oldid=10230

````text
:''This is an archive of past discussions. Please do not edit the contents of this page.''

----

== Article Focus ==

Should there be articles about specific devices (that seems like repeating wikipedia) or should the articles be focused on
how the devices are related to designing an OS. Instead of having an article about what some device is, have it be about
writing drivers for it, how the device operates and what the OS needs to do to communicate with it. <br />
-[[User:Telexicon|Telexicon]] 18:04, 2 December 2006 (CST)


You mean pages like [[wikipedia:Sound_blaster]]? If you use the Sound Blaster as an example, we probably shouldn't a single Sound Blaster page, instead we should have a category page that provides a brief intro description to the sound blaster family, links the the wikipedia articles, and has links to our pages that cover more programming specific info by major card type.<br />
-[[User:Chase|Chase]] 18:29, 2 December 2006 (CST)

I agree. It would be useful to have a small introduction for each device and then list articles and resources for it. Then in another section of the article list our pages that go into details about interaction with the device and writing code for it. For writing code do you think things should be in C when they can be? -[[User:Telexicon|Telexicon]] 00:06, 3 December 2006 (CST)

Pseudocode should suffice for the real programmers (apart from it being not prejudiced about one language), but i guess some people are better off with some sample code which they can directly import. Personally, i'd prefer pseudocode unless either C or ASM is more obvious. Supplying both would probably be best - see the coma bug part [[CPU Bugs|here]] - [[User:Combuster|Combuster]] 09:50, 3 December 2006 (CST)

== Layout ==

In the hardware section on the main page I have noticed that some of the links go to categories and some of them go to articles. Would you prefer each of those be articles with links to more articles all of which are in the same category? Or would you rather have each of those links be categories with a small description and listing each of the sub-articles in that category? -[[User:Telexicon|Telexicon]] 00:06, 3 December 2006 (CST)

As an example each of the links on the main page could go to a main article for each category. That main article and all its sub-articles would all be included in that category (unless the sub-article led to a new sub-topic in which case it would become a sub-category). If a person were to click on the category page there would be a small paragraph on what the general idea of the category is and then a link using the <nowiki>{{main|Main Article}}</nowiki> template. It might be laid out like this:

*Main Page
**Category X Main Page (in Category X)
***Sub-Article X<sub>1</sub> (in Category X)
***Sub-Article X<sub>2</sub> (in Category X)
***Sub-Category Y Main Page (in Category Y)
****Sub-Article Y (in Category Y)
***Sub-Category Z Main Page (in Category Z)
****Sub-Article Z (in Category Z)

with the categories laid out like

*Broad Category (Hardware, OS Theory, Tools)
**Category X (uses <nowiki>{{main|Category X Main Page}}</nowiki> - it is a sub-category of Broad Category.
***Category Y (uses <nowiki>{{main|Category Y Main Page}}</nowiki> - it is sub-category of Category X

If any style for this were to be used it would be nice to have it be consistent across all the pages so this wiki's layout isn't confusing to readers. [[User:Telexicon|Telexicon]] 04:42, 3 December 2006 (CST)

how about not categorizing categories and instead provide a header with subcategories/parent category manually. More work, but should solve your issue. ''Oops mediawiki already does that O.o'' - [[User:Combuster|Combuster]] 09:50, 3 December 2006 (CST)

== Related Content ==

At the bottom of many articles is a list of related reading, forum topics and external links. How do you think these should be laid out? Is there a consistent that is preferable for many users? Would it be something where every related item set gets its own heading? :

Should it be where all of them are subheadings of Related or See Also

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

As with the last item I think it would be best if it was consistent. Also, for all these items could they possibly be compiled into a style guide if any final decisions are made on how the articles should be laid out? [[User:Telexicon|Telexicon]] 04:59, 3 December 2006 (CST)

Or just a links section as in [[CPUID|here]]? - [[User:Combuster|Combuster]] 09:50, 3 December 2006 (CST)

== Tools ==

Right now the Tools section looks like this:
=== Tools ===
* [[Tool Comparison]]
* [[NASM]]
* [[GAS|GAS (GNU AS)]]
* [[GCC]]
* [[GCC Derivatives|DJGPP/Cygwin]]
* [[Borland]]
* [[Visual Studio]]
* [[:Category:Emulators|Emulators]]

Do you think it would work better to have it like this? :

=== Tools ===
* [[Tool Comparison]]
* [[Assemblers]]
* [[Compilers]]
* [[Linkers]]
* [[Editors]]
* [[IDEs]]
* [[Bootloaders]]
* [[:Category:Emulators|Emulators]]

That way each page could host as many tools under each category and not fill up the main Tools menu.


* '''Strong Support''' I'd actually been thinking the same thing. Also, if all the links were to categories we wouldn't need to maintain those pages. -[[User:Jhawthorn|Jhawthorn]] 11:58, 5 December 2006 (CST)


Good idea. The current is one is like chaos and the old wiki. Also I'd go for the categories as well:
* [[Tool Comparison]]
* [[:Category:Assemblers|Assemblers]]
* [[:Category:Compilers|Compilers]]
* [[:Category:Linkers|Linkers]]
* [[:Category:Editors|Editors]]
* [[:Category:IDEs|IDEs]]
* [[:Category:Bootloaders|Bootloaders]]
* [[:Category:Emulators|Emulators]]
- [[User:Combuster|Combuster]] 16:37, 5 December 2006 (CST)



I went ahead and changed it -[[User:Jhawthorn|Jhawthorn]] 19:38, 5 December 2006 (CST)

== File systems ==

Enough for todays work. Question: should the filesystem link point to the [[File Systems|theory page]] or the [[:Category:Filesystems|category page]], or should both be present on the main page? 
- [[User:Combuster|Combuster]] 09:03, 6 December 2006 (CST)

== Main Page test ==

Just throwing around ideas: [[User:Jhawthorn/Main_Page]]

I removed the [edit] links and added "more" links at the end to each Section linking to it's category page.

-[[User:Jhawthorn|Jhawthorn]] 01:44, 7 December 2006 (CST)


I've removed the [edit] links from the real main page. If there is opposition to this change please revert it.

-[[User:Jhawthorn|Jhawthorn]] 18:40, 8 December 2006 (CST)

==Instructions==
I have some pages in [[:Category:X86_Instructions]] as a test. I think they look pretty cool. They could use some more information, opcodes, clocks on more recent processors, etc. Any Opinions? --[[User:Jhawthorn|Jhawthorn]]


I Was reading the recent changes and the first thought that occurred to me was '''OMFG'''. I really consider this overkill. Documenting all instructions is
# an very exhaustive task and 
# we have the intel/amd manuals for that. (and once downloaded we dont need an internet connection for it)
# IMO beyond the scope of this wiki
# good cause for ambiguity when we whant to do it for more than one processor.
Also, clock cycles and the like are pretty much indeterminate on 6th generation processors and later. And giving each opcode an separate page hardly works to the advantage of any comparison. Talking about differences between processors is a more useful task as people can not generally find those without looking at more than one datasheet. - [[User:Combuster|Combuster]] 12:08, 13 December 2006 (CST)

:I'll remove clocks. I only used it because I had the information on hand. I was only planning on having instructions that pertained strongly to osdev, or are referred to by other pages (i feel that all the current instructions fit in this category). Is this still disagreeable Combuster? <small>''Please sign your comments, [[User:Jhawthorn|Jhawthorn]], by using four tildes (~). Thanks :-) --[[User:Walling|Walling]]''</small>

:: I wrote the [[INT]] article. I tried to write a little more than just a single line. I think some instructions can be nice to write an article about, while others are not. What is an article then? I think there should at least be some wikilinks to other articles. Hopefully they all turn blue in the future as well. Then the reader can browse to other relevant articles. If one wants to know anything about enabling interrupts he might search for [[STI]]. He should be able to read about what it does and why and linked to further information. He might want to know about IRQs or CPU exceptions and stuff like that. An external reference could be added too, fx. the Intel Reference. That is one of the ideas with a wiki, I think. What do you say? --[[User:Walling|Walling]] 18:34, 13 December 2006 (CST)


I'd merge INT, STI, CLI, and LIDT into an [[Interrupts]] page. Speaking of which, I think I should create that page sometime - [[User:Combuster|Combuster]] 02:36, 14 December 2006 (CST)

: Okay, maybe you're right. --[[User:Walling|Walling]] 03:33, 14 December 2006 (CST)

== Main Page design ==

[http://www.osdev.org/phpBB2/viewtopic.php?t=12428 Discussion Thread] --[[User:Walling|Walling]] 03:19, 14 December 2006 (CST)

== Tutorial link ==

I think a link to the [[:Category:Tutorials|tutorial category]] should be added to the first box (below the FAQ link). Comments? - [[User:Combuster|Combuster]] 15:59, 9 January 2007 (CST)
*Absolutely -[[User:Jhawthorn|Jhawthorn]] 19:03, 9 January 2007 (CST)

== os dev image ==

do you thing we should change welcome to osdev.org to a image? --[[User:Stinkfly|Stinkfly]] 04:10, 24 August 2007 (CDT)

I personally don't think so. Any particular reason why it should be? All I see is a tonne of images not loading for some people, added bandwidth (quite a few forum members are still on dialup), etc. Of course, I welcome the argument against me. [[User:Yayyak|Yayyak]] 06:39, 24 August 2007 (CDT)

:Agreed. There is as far as I can see no reason that the text box is not suitable, and the overhead of having an image is larger and less supported. - [[User:Combuster|Combuster]] 04:38, 25 August 2007 (CDT)
:: Just thought! I could make a mediawiki theme for osdev.org with not many images --[[User:Stinkfly|Stinkfly]] 13:22, 21 September 2007 (CDT)
:::If you read the previous replies, you should have noticed a preference for an ''absence'' of images. - [[User:Combuster|Combuster]] 04:03, 24 September 2007 (CDT)

== Combine the header templates into one? ==

I had the thought of combining the three templates for the header into one (or use one that just includes the other three) so it's easier to edit and add the header templates. Just shaves a few steps off the time needed to play with the templates. --[[User:Troy martin|Troy Martin]] 02:49, 25 August 2009 (UTC)

: Sounds like a good idea to me. Maybe it would also be a good idea to put a link to the [[Chat|IRC Channel chat page]] on the main page, to make people notice it. I didn't even know we had one until I looked at the orphaned page list! --[[User:Creature|Creature]] 16:33, 28 August 2009 (GMT + 1)

== languages ==
Should Haskell, D, C--, and C# be listed on the main page? IMO the list of languages is getting long. They're not the languages used for kernel, let alone systems, programming besides by a small minority. --[[User:Messiahandrw|MessiahAndrw]] 03:59, 23 December 2009 (UTC)

:I just put them there for the sake of being complete (as there weren't too many anyhow), but I agree it's a bit crowded and long. We could stick to only putting the main languages on the main page (ASM, C, C++). --[[User:Creature|Creature]] 10:31, 23 December 2009 (UTC)

:'''Agreed''', C-- is in disuse, C# is not supported, and Haskell is definately not supported by the community. The language tab is already complete as it was - There's a [[:Category:Languages|more]] link for the daring ones (or idiots) in case they do want something else. Also, I'd consider it unwise to promote such "esoteric languages" at the front page - it would only lead to more clueless people who just happen to know that language without the needed fundament. I see no advantages in keeping this week's change. - [[User:Combuster|Combuster]] 17:56, 23 December 2009 (UTC)
````

<a id="w001375"></a>

## W001375 — OSDev Wiki:Site support

Bron-ID: W001375
Namespace: OSDev Wiki (4)
Revisie: 18575 | Datum: 2015-11-18T21:10:34Z
Bron: https://wiki.osdev.org/OSDev_Wiki:Site_support
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDev_Wiki%3ASite_support&oldid=18575

````text
Monetary donations are not needed at this time. The hardware, colocation, and bandwidth costs are covered by myself, [http://forum.osdev.org/memberlist.php?mode=viewprofile&u=2 Chase]. If OSDev ever needs money to continue operation or purchase new hardware a notice will be posted to the [http://forum.osdev.org/ OSDev forums]. What we need is people dedicated to updating and maintaining this wiki and answering questions on the forums. If you really want to donate money to something please consider a donation to the [http://www.cancer.org American Cancer Society].

-Chase

[[Category:Wiki administration|{{PAGENAME}}]]
````

<a id="w001376"></a>

## W001376 — Talk:Projects

Bron-ID: W001376
Namespace: Talk (1)
Revisie: 21932 | Datum: 2017-12-17T15:50:31Z
Bron: https://wiki.osdev.org/Talk:Projects
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AProjects&oldid=21932

````text
== Mirroring ==
If you desire you may mirror this page. In your mirrored page you must state that you are a mirror and provide the original URL of the Project page. For a good example see: http://www.osdcom.info/content/view/28/38/




This gives a 404...? --[[User:Phillid|Phillid]] 02:08, 17 November 2013 (CST)

== Format == 
'''All OSes should be listed in alphabetical order.'''

The format of data on this page is in the form of:
* '''OS Name''' - Title or short description. 
** Contact: Chase (chase AT osdev.org)
** URL: [http://www.osdev.org http://www.osdev.org]

== TODO ==
*Add Amateur OSes from [http://en.wikipedia.org/wiki/Osdev http://en.wikipedia.org/wiki/Osdev]
*Add [http://xos.freezee.org/ http://xos.freezee.org/] Added but now the site is down, wait and see if it comes back up
*Add [http://comos.sourceforge.net/ http://comos.sourceforge.net/] ? If it gets fixed/contact the author(waiting for response)

== Dating ==

User (rejected wiki license) asked for projects to be dated - [[User:Combuster|Combuster]] 12:25, 3 October 2011 (UTC)

== Entry Template ==

I wrote two templates for the entries into the list. One is the major and the second is a sub template. The first works like this:
<tt>
  <nowiki>{{</nowiki>OSProjectEntry
  |OSName='''<The project name>'''
  |OSDescription='''<A description of the project>'''
  |Contact='''<A contact address, name, or internet message location>'''
  |License='''<License of this project>'''
  |URL='''<The URL for this project>'''
  |LastReleaseDate='''<A date of the last release known>'''
  |LastReleaseDescription='''<A description of the last release known>'''
  |FloppyAvailable='''<The availability of floppy with your OS.<br>This info is valuable for the people who still use floppies, maybe as the <br>"virtual floppies" which - for example - could be added to coreboot open source BIOS>'''
</tt>''The line (below) uses the optional second template and can be included to provide a closer inspection message for questionable entries.''<tt>
  |CloserInspection=<nowiki>{{</nowiki>CloserInspectionValidity|'''<Reason>'''<nowiki><br \></nowiki>''''<Your name>'''}}
</tt>''The last part regardless if you include the (above) single line or not is to add the two closing curly brackets.''<tt>
  <nowiki>}}</nowiki>
</tt>
If done correctly your entry will appear just like the others. You can also edit that page to see how each entry was made if you are still unsure. 
--[[User:Kmcguire|Kevin]] 17:45, 27 May 2007 (CDT)


Should their be a thing in the template that tells what license the OS is under? cause a few on the Wiki are even closed source. --[[User:Earlz|Earlz]] 13:50, 22 April 2009 (UTC)


Now it is. For example.
<tt>
  |License=CC0
</tt>
This works for the OSProject template too. --[[User:Glauxosdever|Glauxosdever]] 08:02, 4 April 2016 (CDT)

== Dead Websites ==

If a website is dead, please check the https://archive.org WayBack Machine to see if an older version of it was archived. --[[User:Davidaylaian|David Aylaian]] 09:26, 17 December 2017 (CST)

== About dead websites ==

Sadly there is a big problem - even when the website is a part of archive,
usually the downloads with OS source code are not archived :(
````

<a id="w001379"></a>

## W001379 — MBR (x86)

Bron-ID: W001379
Namespace: Main (0)
Revisie: 28022 | Datum: 2023-07-09T18:08:27Z
Bron: https://wiki.osdev.org/MBR_(x86)
Revisie-URL: https://wiki.osdev.org/index.php?title=MBR_%28x86%29&oldid=28022

````text
A '''Master Boot Record''' ('''MBR''') is the bootsector of a hard disk. That is, it is what the BIOS loads and runs, when it boots a hard disk. The MBR is the very first sector of the hard disk; it contains an MBR Bootstrap program (described below),
and a [[Partition Table]]. Devices that emulate a hard disk during system initialization must also contain an MBR, because they
must also have Partition Tables -- even if they are not bootable.

The BIOS will only boot an MBR from a device if that device is in the "boot sequence" stored in [[CMOS]], and if the MBR on the device
is formatted correctly. On the other hand, if a device is not in the boot sequence (but has a "drive number"), it is still possible
for a [[Real Mode]] program (such as another MBR or bootloader) to load and boot that device's MBR directly.

==MBR Bootstrap==

An MBR is loaded by the [[BIOS]] at physical address 0x7c00, with DL set to the "drive number" that the MBR was loaded from.
The BIOS then jumps to the very beginning of the loaded MBR (0x7c00), because that part of the MBR contains the "bootstrap" executable code.

Typical MBR bootstrap code will do the following:

* relocate itself away from the 0x7c00 physical address (using a memory copy, and usually a far jump)
* determine which partition (or hard disk) to boot from (either by looking for the active partition, or by presenting the user with a selection of installed operating systems to choose from)
** if the user selected an "inactive" partition, then set the selected partition entry to "active", and clear the "active" bits of other partition entries
** use [[ATA in x86 RealMode (BIOS)|BIOS INT 13h]] commands to rewrite the MBR if the partition table entries were modified
* use [[ATA in x86 RealMode (BIOS)|BIOS INT 13h]] commands to load the [[Volume Boot Record]] (VBR, the "bootsector" of the bootloader) from the beginning of the selected partition to physical address 0x7c00
* set DS:SI pointing to the selected partition table entry
* jump to 0x7c00 (with CS set to 0, and DL set to the "drive number")

Note: it is intended for the value of DL, and the DS:SI pointer to be passed all the way into the kernel, for the kernel's use.
This also means that the relocated MBR should not be overwritten during the boot process -- because the DS:SI pointer is aimed
at a partition table entry inside that MBR, and needs to remain valid.

==MBR Format==

Notes:

* See the [[Partition Table]] article for the format of each partition table entry field.
* It is important to remember that the partition table entries are <b>not</b> aligned on 32-bit boundaries.
* Naming the partition table entries as "First" through "Fourth" is for convenience only. The partition table entries are <b>not</b> required to be in any kind of order.
* <b>Any</b> one of the partitions may be "active".
* There is supposed to be one active partition table entry, at most.
* Windows seems to verify and require that the partition it boots from be marked "active".
* Most other operating systems don't seem to care about the "active" bit in the partition table entry.

{| {{wikitable}}
! Offset
! Size (bytes)
! Description
|-
| 0x000
| 440<sup>1</sup>
| MBR <b>Bootstrap</b> (flat binary executable code)
|-
| 0x1B8
| 4
| Optional "Unique Disk ID / Signature"<sup>2</sup>
|-
| 0x1BC
| 2
| Optional, reserved 0x0000<sup>3</sup>
|-
| 0x1BE
| 16
| First partition table entry
|-
| 0x1CE
| 16
| Second partition table entry
|-
| 0x1DE
| 16
| Third partition table entry
|-
| 0x1EE
| 16
| Fourth partition table entry
|-
| 0x1FE
| 2
| (0x55, 0xAA) "Valid bootsector" signature bytes
|}

<sup>1</sup> This can be extended to 446 bytes if you omit the next 2 optional fields: Disk ID and reserved.

<sup>2</sup> The 4 byte "Unique Disk ID" is used by recent Linux and Windows systems to identify the drive.
"Unique" in this case means that the IDs of all the drives attached to a particular system
are distinct.

<sup>3</sup> The 2 byte reserved is usually 0x0000. 0x5A5A means read-only according to https://neosmart.net/wiki/mbr-boot-process/.

==Partition table entry format==
{| {{wikitable}}
!Offset<sup>1</sup>
!Size (bytes)
!Description
|-
|0x00
|1
|Drive attributes (bit 7 set = active or bootable)
|-
|0x01
|3
|CHS Address of partition start
|-
|0x04
|1
|Partition type
|-
|0x05
|3
|CHS address of last partition sector
|-
|0x08
|4
|LBA of partition start
|-
|0x0C
|4
|Number of sectors in partition
|}

<sup>1</sup> Relative to the start of the partition entry.

==Traditional MBR==

The DOS FDISK program was the first to ever use an MBR, so that MBR became the de-facto standard.
It also automatically became the standard for the minimum level of functionality of an MBR. It was never
changed after it was first introduced.

The whole point of the FDISK program is to manipulate the MBRs of the hard disks attached to a system.
When FDISK partitions a blank disk, it writes an MBR to sector 0 of the drive. When FDISK adds a new partition
to a disk, it adds an entry into the Partition Table in the MBR. When FDISK makes a partition "active",
it sets the "flag" byte in the Partition Table entry to 0x80.

The MBR that FDISK uses is coded to:

* relocate itself to <tt>0x0000:0x0600</tt>
* examine the byte at offset 0x1be, 0x1ce, 0x1de, and 0x1ee to determine the active partition
* load only the first sector of the active partition (which is expected to contain a DOS bootsector) to <tt>0x0000:0x7c00</tt> (hence the previous relocation)
* set SI
* jump to 0x7c00 -- transferring control to the DOS bootsector.

<!--Commented out because rant
It is extremely unfortunate and stupid that this MBR loads only one sector of the booted partition. There is no
additional complication to load more than one sector in the code; but doing so makes writing bootloaders <b>much</b> simpler
if, perhaps, 8 sectors are loaded instead. So it may be wise to always replace this old MBR with a custom MBR that loads
more than one sector, for your OS. DOS/Windows will still boot correctly if more than one sector is loaded.
-->

==Dual Booting==

A typical system may have several hard disks on it, and each hard disk can have 4 standard partitions (without
going into the extra complication of Extended Partitions). Each of those partitions could theoretically hold
its own distinct bootable OS, and filesystem.

However, the standard x86 boot sequence will only ever boot the MBR from the "C:" disk
(the first disk found during the disk detection phase). And the standard MBR will only allow a single active
partition on that disk, and will only ever boot that one partition. This is really completely inadequate.

It is not possible to modify the BIOS/CMOS boot sequence -- but you <i>can</i> change the MBR. So, one solution is to replace the standard MBR
with a "custom, Dual Booting" MBR. (For a good example, see John Fine's SMBMBR in the [[#External Links|External Links]], below.)

A simple dual booting MBR will allow the user to select any partition on the current drive, to boot. A more
complicated dual booting MBR will also allow the user to select other hard disks, or even to specifically
choose other partitions on other hard disks, to boot.

For one MBR to load and run a different MBR off a different drive is called "chain loading". If all the MBRs
are dual booting, then the user can cycle through all the disks, and choose to boot the "correct" partition from
the "correct" drive.

A dual booting MBR is a huge improvement over the standard MBR. The only problem is that an MBR bootstrap is limited
to a little over 400 bytes of code. Such a tiny program is insufficient to create a "nice" user-friendly dual
booting system that has commercial appeal. It <b>is</b> enough space to hack up an interface that is good enough
for the person who wrote it.

One way around this size limitation is to note that a bootloader can contain a <b>lot</b> more code than an MBR. So a custom
MBR can try to load a "preferred" bootloader (if it can find one on some partition, somewhere) -- and that preferred
bootloader can have a very pretty, user-friendly interface that allows the user to select any partition
off any drive, to boot.

It is also nice, if there is only one partition on only one drive (ie. there is no choice for a user to make), if the
MBR will boot that one partition automatically -- without bothering the user with prompts.

==Generic MBRs==

As said above, the MBR that is written to the disk by the DOS FDISK program is "the most generic" one. But every disk partitioning
app must write some sort of MBR to the disk, and every generic bootloader may well need a custom MBR.

Fortunately, there was a standard for them to conform to (the FDISK MBR), and they all did conform to it. Some of them may be
Dual Booting, or have other features -- but they will all end up loading your bootloader at the standard address (0x7c00),
with the DL register set to the boot "drive number", and DS:SI pointing at the correct partition table entry of the correct
MBR.

==Building a Custom MBR Bootstrap program==

When the BIOS transfers control to the MBR bootstrap code, the system is in Real mode. The MBR will probably also run entirely
in Real mode, so it is important to understand [[Real Mode#Real Mode Memory Addressing|Real mode addressing]].

Self-relocation is one of the [[C#Things C can't do|things C can't do]], and most modern C compilers can't create code that is
compatible with Real mode, anyway. So an MBR must be written in [[Assembly]].

It is necessary to build an MBR that is exactly 512 bytes long. How this is done depends on your assembler and linker.
The last 2 bytes must be the special boot pattern (0x55 followed by 0xAA), and as said above, the bootstrap portion of the
MBR must be at most 446 bytes long. You must also put at least one partition table entry in the Partition Table
portion of the MBR -- otherwise some BIOSes will refuse to boot from the disk. This includes most UEFI firmware in BIOS-compatibility
mode.

===Initial Environment===

When the BIOS loads and runs an MBR, it is loaded into memory at physical address 0x7c00. This is ''usually'' <tt>0x0000:0x7c00</tt>
(CS = 0, offset address 0x7c00). However, some BIOSes load
to <tt>0x7c0:0x0000</tt> (CS = 0x07c0, offset address 0) -- which resolves to the same physical address, but can cause problems.
A good practice is to enforce CS:IP with a far jump near the beginning of your bootstrap code.
The MBR will probably need to immediately relocate itself anyway, and that is a good time to enforce CS:IP.

The BIOS passes very little useful information directly to the MBR. In fact, the only important number is the value in DL -- the "drive
number". It needs to be passed to all later calls to BIOS function INT 13h, so that byte in DL probably needs to be saved carefully.

The values in all the other registers, and in most of memory, are undefined.

===Immediate Priorities===

It is important to immediately set up a [[stack]], and also to set the rest of the CPU's segment registers
(DS, ES, FS, GS) properly. Setting up a stack involves pointing the SS:SP pair of registers at some memory that is not being used for anything
else (and the address must be even). The other segment registers should usually be set to 0.

See the [[Memory Map (x86)]] article to understand what memory is available during boot (low memory from 0x500 to 0x7ffff, generally).

In general, you do not want to fragment your available memory, or the memory of each 64K "page", if possible. The MBR needs a stack,
and a place to relocate itself to. The MBR will be loading a bootloader at 0x7c00, so it is reasonably convenient for the MBR to
relocate itself either to somewhere around the 0x500 to 0x600 address range, or to 0x7a00 (ie. just below 0x7c00). The stack can then
point to 0x7c00 (if the MBR is not at 0x7a00), or the stack can be just below the relocated MBR.

==Storing an MBR to the disk==

To write an MBR to the first sector of a disk, you must use special disk I/O tools, because the MBR (by definition) is not inside any disk partition.
The MBR only exists on the "raw device". There are quite a few "disk editing" tools available; some are listed in
[[:Category:Disk Image Utilities|Disk Image Utilities]].

==x86 Examples==
Assuming one knows how to read sectors from disk and has setup a function allowing them to do so, and assuming the inputs to said function are:
* EBX - 32-bit LBA Address
* CX - Sector Count
* ES:DI - Buffer
* BYTE [bootDrive] - Drive Number<br />
An extremely simple MBR complying to the standard could look like:
<syntaxhighlight lang="asm">
[bits 16]
[org 0x0600]

start:
  cli                         ; We do not want to be interrupted
  xor ax, ax                  ; 0 AX
  mov ds, ax                  ; Set Data Segment to 0
  mov es, ax                  ; Set Extra Segment to 0
  mov ss, ax                  ; Set Stack Segment to 0
  mov sp, ax                  ; Set Stack Pointer to 0
  .CopyLower:
    mov cx, 0x0100            ; 256 WORDs in MBR
    mov si, 0x7C00            ; Current MBR Address
    mov di, 0x0600            ; New MBR Address
    rep movsw                 ; Copy MBR
  jmp 0:LowStart              ; Jump to new Address

LowStart:
  sti                         ; Start interrupts
  mov BYTE [bootDrive], dl    ; Save BootDrive
  .CheckPartitions:           ; Check Partition Table For Bootable Partition
    mov bx, PT1               ; Base = Partition Table Entry 1
    mov cx, 4                 ; There are 4 Partition Table Entries
    .CKPTloop:
      mov al, BYTE [bx]       ; Get Boot indicator bit flag
      test al, 0x80           ; Check For Active Bit
      jnz .CKPTFound          ; We Found an Active Partition
      add bx, 0x10            ; Partition Table Entry is 16 Bytes
      dec cx                  ; Decrement Counter
      jnz .CKPTloop           ; Loop
    jmp ERROR                 ; ERROR!
    .CKPTFound:
      mov WORD [PToff], bx    ; Save Offset
      add bx, 8               ; Increment Base to LBA Address
  .ReadVBR:
    mov EBX, DWORD [bx]       ; Start LBA of Active Partition
    mov di, 0x7C00            ; We Are Loading VBR to 0x07C0:0x0000
    mov cx, 1                 ; Only one sector
    call ReadSectors          ; Read Sector

  .jumpToVBR:
    cmp WORD [0x7DFE], 0xAA55 ; Check Boot Signature
    jne ERROR                 ; Error if not Boot Signature
    mov si, WORD [PToff]      ; Set DS:SI to Partition Table Entry
    mov dl, BYTE [bootDrive]  ; Set DL to Drive Number
    jmp 0x7C00                ; Jump To VBR

times (218 - ($-$$)) nop      ; Pad for disk time stamp

DiskTimeStamp times 8 db 0    ; Disk Time Stamp

bootDrive db 0                ; Our Drive Number Variable
PToff dw 0                    ; Our Partition Table Entry Offset

times (0x1b4 - ($-$$)) nop    ; Pad For MBR Partition Table

UID times 10 db 0             ; Unique Disk ID
PT1 times 16 db 0             ; First Partition Entry
PT2 times 16 db 0             ; Second Partition Entry
PT3 times 16 db 0             ; Third Partition Entry
PT4 times 16 db 0             ; Fourth Partition Entry

dw 0xAA55                     ; Boot Signature
</syntaxhighlight>
Of course this is an extremely simplified MBR, and is only able to load the [first] active partition.

==Comments==

==See Also==

===Articles===

* [[System Initialization (x86)]]

===External Links===
* [http://www.osdever.net/downloads/bootsectors/smbmbr03.zip Fine's MBR code on osdever]
* [http://www.nondot.org/sabre/os/articles/TheBootProcess/ OSRC] -- for more boot sector info
* [http://www.ranish.com/part/ Ranish Partition Manager] -- Disk partitioning software

[[Category:X86]]
[[Category:Booting]]
[[Category:Bootloaders]]

[[de:Master Boot Record]]
````

<a id="w001380"></a>

## W001380 — User:ComputerPsi

Bron-ID: W001380
Namespace: User (2)
Revisie: 1475 | Datum: 2006-07-16T01:58:51Z
Bron: https://wiki.osdev.org/User:ComputerPsi
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AComputerPsi&oldid=1475

````text
== About ComputerPsi ==
I am an assembly operating system programmer. I try to program simplest, fastest, and most efficent code for anything I do. Even in protected mode, I strive for optimal performance - it takes me longer to write code and I have to have a full understanding of whatever I am doing, but it's worth it. My code is still far from breaking the megabyte barrier even though I am already doing everything in protected mode.
````

<a id="w001382"></a>

## W001382 — User:SpooK

Bron-ID: W001382
Namespace: User (2)
Revisie: 1481 | Datum: 2006-07-17T04:53:51Z
Bron: https://wiki.osdev.org/User:SpooK
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASpooK&oldid=1481

````text
== About SpooK ==
At the time of this writing, I have no formal training in operating system design or computer programming in general. My computer skills are mostly amateur (not a main source of income) and I am not a mathematics guru, hence why I love computers, since they do the math for me.


What I do posses, is 16+ years knowledge of computer systems including hardware, software, programming languages such as BASIC, C and Assembly. I am familiar with HTML/Javascript and some scripting languages such as PERL and PHP. I have over 5 years of experience with specific x86 operating system design and programming. I have extensive experience with DOS, 16/32/64 bit Windows, and POSIX compliant operating systems such as the various distributions of Linux.


Other interests include website design, challenging computer games, football, electronics, and my greatest interest... my wife.


My personal OS Project, is [http://www.dynatos.org/ DynatOS].
````

<a id="w001384"></a>

## W001384 — Category:X86

Bron-ID: W001384
Namespace: Category (14)
Revisie: 12645 | Datum: 2012-02-18T11:36:15Z
Bron: https://wiki.osdev.org/Category:X86
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AX86&oldid=12645

````text
The x86 category includes information covering Intels 8086,186,286,386,486,Pentium, and newer cpus that initialize in an 8086 compatibility mode. The term x86 is typically limited to 16 and 32-bit cpus. Cpus that have 64-bit extensions to x86 compatible cpus are commonly known as x86-64. Computer systems are considered to be x86 architecture systems if they are opcode compatible with Intel x86 cpus even when employing cpus by manufacturers such as Cyrix and AMD. X86 now has the X87 coprocessor (FPU) intergrated

[[Category:Platforms]]
````

<a id="w001387"></a>

## W001387 — Help:Starting a new page

Bron-ID: W001387
Namespace: Help (12)
Revisie: 6617 | Datum: 2008-11-04T22:23:37Z
Bron: https://wiki.osdev.org/Help:Starting_a_new_page
Revisie-URL: https://wiki.osdev.org/index.php?title=Help%3AStarting_a_new_page&oldid=6617
Doorverwijzing: OSDev Wiki:Editing

````text
#REDIRECT [[OSDev Wiki:Editing]]
````

<a id="w001388"></a>

## W001388 — Main Page

Bron-ID: W001388
Namespace: Main (0)
Revisie: 29074 | Datum: 2024-06-07T13:34:23Z
Bron: https://wiki.osdev.org/Main_Page
Revisie-URL: https://wiki.osdev.org/index.php?title=Main_Page&oldid=29074
Doorverwijzing: Expanded Main Page

````text
#REDIRECT [[Expanded Main Page]]
````

<a id="w001389"></a>

## W001389 — Template:OS Development

Bron-ID: W001389
Namespace: Template (10)
Revisie: 10173 | Datum: 2010-04-28T20:28:52Z
Bron: https://wiki.osdev.org/Template:OS_Development
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AOS_Development&oldid=10173

````text
* [[Introduction|What is it? Why?]]
* [[History]]
* [[Getting Started]]
* [[Troubleshooting]]
* [[:Category:FAQ|FAQ]]
* [[Tutorials]]
<noinclude>[[Category:Main Page Templates]]</noinclude>
````

<a id="w001390"></a>

## W001390 — Template:Welcome

Bron-ID: W001390
Namespace: Template (10)
Revisie: 29146 | Datum: 2024-06-24T04:49:22Z
Bron: https://wiki.osdev.org/Template:Welcome
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AWelcome&oldid=29146

````text
<div style="margin: 0 0 8px 0; padding: 1px; border: 1px solid #CCCCCC;">
{| style="width: 100%; margin: 0; padding: 0; border: 0; background-color: #FCFCFC; color: #000000; border-collapse: collapse;"
| align="center" style="vertical-align: top;" |
<div class="plainlinks" style="text-align: center; padding: 0.7em 0;">
<div style="font-size: 160%;">Welcome to [http://www.osdev.org/ OSDev.org]</div>
<div style="font-size: 90%; margin-top: 0.4em; line-height: 130%;">Our [https://wiki.osdev.org wiki] and [https://forum.osdev.org forums] provide 200k+ posts and [[Special:Statistics|{{NUMBEROFARTICLES}}]] articles about the creation of operating systems.</div>
<div style="font-size: 90%; margin-top: 0.4em; line-height: 130%;">Become a [https://forum.osdev.org/viewtopic.php?t=677 wiki author]! See the [[Wish List]] for suggestions on where to start.</div>
</div>
|}
</div>
````

<a id="w001391"></a>

## W001391 — User:Chase

Bron-ID: W001391
Namespace: User (2)
Revisie: 29132 | Datum: 2024-06-22T06:57:00Z
Bron: https://wiki.osdev.org/User:Chase
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AChase&oldid=29132

````text

This is an edit from notchase

My Sandbox: [[User:Chase/temp]]
````

<a id="w001392"></a>

## W001392 — Template:Languages

Bron-ID: W001392
Namespace: Template (10)
Revisie: 10172 | Datum: 2010-04-28T20:28:39Z
Bron: https://wiki.osdev.org/Template:Languages
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ALanguages&oldid=10172

````text
* [[:Category:Languages|Which Language]]
* [[Assembly]]
* [[C]]
* [[C++]]
<noinclude>[[Category:Main Page Templates]]</noinclude>
````

<a id="w001393"></a>

## W001393 — Template:Tools

Bron-ID: W001393
Namespace: Template (10)
Revisie: 10176 | Datum: 2010-04-28T20:29:36Z
Bron: https://wiki.osdev.org/Template:Tools
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ATools&oldid=10176

````text
* [[Tool Comparison]]
* [[:Category:Assemblers|Assemblers]]
* [[:Category:Compilers|Compilers]]
* [[:Category:Linkers|Linkers]]
* [[:Category:IDEs|IDEs]]
* [[:Category:Bootloaders|Bootloaders]]
* [[:Category:Emulators|Emulators]]
* [[:Category:Disk Image Utilities|Disk Image Utilities]]
<noinclude>[[Category:Main Page Templates]]</noinclude>
````

<a id="w001394"></a>

## W001394 — Template:Resources

Bron-ID: W001394
Namespace: Template (10)
Revisie: 19157 | Datum: 2016-05-08T21:14:44Z
Bron: https://wiki.osdev.org/Template:Resources
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AResources&oldid=19157

````text
* [http://forum.osdev.org/ Forums]
* [[Books]]
* [[Resources]]
* [[Academic Papers]]
* [[Projects|Project List]]
* [[OSDev (foreign language)|Non-English OSDev]]
<noinclude>[[Category:Main Page Templates]]</noinclude>
````

<a id="w001395"></a>

## W001395 — Template:OS theory

Bron-ID: W001395
Namespace: Template (10)
Revisie: 10174 | Datum: 2010-04-28T20:29:01Z
Bron: https://wiki.osdev.org/Template:OS_theory
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AOS_theory&oldid=10174

````text
* [[Kernels]]
* [[Memory Management]]
* [[Processes and Threads]]
* [[:Category:IPC|IPC]]
* [[Device Driver Interfaces]]
* [[File Systems]]
* [[:Category:Networking|Networking]]
* [[Text UI]]
* [[:Category:Graphical UI|Graphical UI]]
* [[Security]]
* [[Licensing]]
<noinclude>[[Category:Main Page Templates]]</noinclude>
````

<a id="w001397"></a>

## W001397 — Template:Hardware

Bron-ID: W001397
Namespace: Template (10)
Revisie: 10171 | Datum: 2010-04-28T20:28:30Z
Bron: https://wiki.osdev.org/Template:Hardware
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AHardware&oldid=10171

````text
*[[:Category:Buses|Buses]]
*[[:Category:Common Devices|Common Devices]]
*[[:Category:Human_Interface_Device|Input Devices]]
*[[:Category:Network Hardware|Network Hardware]]
*[[:Category:Platforms|Platforms]]
**[[:Category:X86|X86]]
***[[:Category:X86 CPU|X86 CPU]]
*[[:Category:Sound|Sound]]
*[[:Category:Storage|Storage]]
*[[:Category:Video|Video]]
<noinclude>[[Category:Main Page Templates]]</noinclude>
````

<a id="w001398"></a>

## W001398 — File:Os-projects.png

Bron-ID: W001398
Namespace: File (6)
Revisie: 1559 | Datum: 2006-11-26T19:21:01Z
Bron: https://wiki.osdev.org/File:Os-projects.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AOs-projects.png&oldid=1559

````text
[[Category:Banner Exchange]]
````

<a id="w001399"></a>

## W001399 — Category:Banner Exchange

Bron-ID: W001399
Namespace: Category (14)
Revisie: 2340 | Datum: 2006-12-16T20:58:12Z
Bron: https://wiki.osdev.org/Category:Banner_Exchange
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ABanner_Exchange&oldid=2340

````text
A page for all the banner and buttons graphics that can be used on any site to promote operating system development and the OSes created by their developers. If possible download to your website and host them. If you can provide a link on your site to http://www.osdev.org or [[Projects]] to help promote independent operating systems. To add a new logo just go to [[Special:Upload]] and add <nowiki>[[Category:Banner Exchange]]</nowiki> as part of the description.

[[Category:Resources]]
````

<a id="w001400"></a>

## W001400 — Template:Main

Bron-ID: W001400
Namespace: Template (10)
Revisie: 28974 | Datum: 2024-06-06T17:21:20Z
Bron: https://wiki.osdev.org/Template:Main
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AMain&oldid=28974

````text
:''Main article:'' [[{{{1}}}]]
````

<a id="w001401"></a>

## W001401 — Kernels

Bron-ID: W001401
Namespace: Main (0)
Revisie: 8875 | Datum: 2009-08-26T04:27:20Z
Bron: https://wiki.osdev.org/Kernels
Revisie-URL: https://wiki.osdev.org/index.php?title=Kernels&oldid=8875
Doorverwijzing: Kernel

````text
#REDIRECT [[Kernel]]
````

<a id="w001402"></a>

## W001402 — Template:Wikitable

Bron-ID: W001402
Namespace: Template (10)
Revisie: 28972 | Datum: 2024-06-06T17:21:14Z
Bron: https://wiki.osdev.org/Template:Wikitable
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AWikitable&oldid=28972

````text
border="2" cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
````

<a id="w001403"></a>

## W001403 — Template:Yes

Bron-ID: W001403
Namespace: Template (10)
Revisie: 28957 | Datum: 2024-06-06T17:17:39Z
Bron: https://wiki.osdev.org/Template:Yes
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AYes&oldid=28957

````text
style="background: #ddffdd" | {{{1|Yes}}}
````

<a id="w001404"></a>

## W001404 — Template:No

Bron-ID: W001404
Namespace: Template (10)
Revisie: 16083 | Datum: 2014-04-12T01:44:43Z
Bron: https://wiki.osdev.org/Template:No
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ANo&oldid=16083

````text
style="background: #ffdddd" | {{{1|No}}}
````

<a id="w001405"></a>

## W001405 — Tool Comparison

Bron-ID: W001405
Namespace: Main (0)
Revisie: 24610 | Datum: 2020-04-19T17:20:13Z
Bron: https://wiki.osdev.org/Tool_Comparison
Revisie-URL: https://wiki.osdev.org/index.php?title=Tool_Comparison&oldid=24610

````text
This is a comparison of various operating system development tools.

==x86 Assemblers==
{| {{wikitable}}
|-
!
! colspan="2" | Syntax
! colspan="4" | Output Format
! colspan="3" | Instruction Set
|-
!
! [[Assembly#AT&T Syntax|AT&T]]
! [[Assembly#Intel Syntax|Intel]]
! [[ELF]]
! [[a.out]]
! [[COFF]]
! flat
! 16 bit
! 32 bit
! 64 bit
|-
! [[GAS]]
| {{Yes}}
| {{Yes}}{{ref|1}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}{{ref|2}}
| {{Yes}}
| {{Yes}}{{ref|3}}
|-
! [[NASM]]
| {{No}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}{{ref|4}}
|-
! [[YASM]]
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{No}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
|-
! [[FASM]]
| {{No}}
| {{Yes}}
| {{Yes}}
| {{No}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
|-
! [[JWASM]]
| {{No}}
| {{Yes}}
| {{Yes}}
| {{No}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
|-
! [[MASM]]{{ref|5}}
| {{No}}
| {{Yes}}
| {{No}}
| {{No}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}{{ref|6}}
|-
! [[Sol_asm]]
| {{No}}
| {{Yes}}{{ref|7}}
| {{Yes}}
| {{No}}
| n/a
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
|-
! [[AS86]]
| {{No}}
| {{Yes}}
| {{No}}
| {{No}}
| {{No}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{No}}
|-
! [[LLVM|llvm-mc]]
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{No}}
| {{No}}
| {{Yes}}
| {{Yes}}
|}

* {{note|1}} using the <code>.intel_syntax</code> directive. Several opcodes will still use the AT&T name.
* {{note|2}} While GAS normally writes only "pure" 32-bit i386 code, it has [http://sourceware.org/binutils/docs/as/i386_002d16bit.html limited support] for writing code to run in real mode or in 16-bit protected mode code segments.
* {{note|3}} You have to choose the instruction set at compile time (of the assembler). GAS can not assemble for more than one operating mode at a time, i.e. you would need one GAS for 32bit and one GAS for 64bit if you'd want to mix those two.
* {{note|4}} 64-bit support is only available in NASM 2.x.
* {{note|5}} 64-bit support is only available in a separate executable, named ml64.
* {{note|6}} Sol_Asm uses its own syntax, which looks like the Intel syntax.

==Compilers==
{| {{wikitable}}
|-
!
! Extensions
! colspan=8 | Output format
|-
!
! Inline Assembly
! [[ELF]]
! [[a.out]]
! flat
! [[PE]]
! COFF
! 16-bit code segments
! 32-bit code segments
! 64-bit code segments
|-
! [[GCC]]
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| ?
| {{Yes}}
| {{Yes}}
|-
! [[LLVM|Clang]]
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{No}}
| {{Yes}}
| {{Yes}}
| {{No}}
| {{Yes}}
| {{Yes}}
|-
! msvc
| {{Yes}}
| {{No}}
| {{No}}
| {{No}}{{Ref|4}}
| {{Yes}}
| {{Yes}}
|  Version 4.X
| {{Yes}}
| {{Yes}}
|-
! icl
| {{Yes}}
| {{No}}
| {{No}}
| {{No}}
| {{Yes}}
| {{Yes}}
| {{No}}
| {{Yes}}
| {{Yes}}
|-
! [[Watcom]]
| {{Yes}}
| {{No}}
| {{No}}
| {{No}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
|-
! [[Smaller C]]
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{No}}
| {{Yes}} (incl. huge & unreal mode(l)s){{Ref|5}}
| {{Yes}}
| {{No}}
|}

* {{Note|4}} Visual Studio can emit <code>.COM</code> files.
* {{Note|5}} Smaller C [https://github.com/alexfru/SmallerC/pull/25#issuecomment-516694067| wasn't really designed to properly support 16-bit and 64-bit CPUs].

==Linkers==
{| {{wikitable}}
|-
! rowspan=2 | Name
! colspan=5 | Input
! colspan=5 | Output
! rowspan=2 | Flat Binary Images
! rowspan=2 | Set Code Address
! rowspan=2 | Library Support
! rowspan=2 | Supports Debug Info
! rowspan=2 | 32bit Code
|-
! OMF/ OBJ
! Win32/ COFF
! ELF
! A.OUT
! Others
! Win32/ PE
! Win32/ DLL
! ELF
! A.OUT
! Others
|-
! [[JLoc]]
| {{Yes}}
|
|
|
|
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| DOS/COM
| {{Yes}}
| {{Yes}}
| {{No}}
| OMF/OBJ
| {{Yes}}
|-
! [[ALink]]
| {{Yes}}
| {{Yes}}
| {{No}}
| {{No}}
| DJGPP/COFF
| {{Yes}}
| {{Yes}}
| {{No}}
| {{No}}
| DOS/COM DOS/EXE
| {{No}}
| {{Yes}}
| {{Yes}}
| {{No}}
| {{Yes}}
|-
! [[LD]]
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| DJGPP/COFF OMF/COFF
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| various
| {{Yes}}
| {{Yes}}
| {{Yes}}
| ECOFF DWARF STABS
| {{Yes}}
|-
! [[TLink]]
| {{Yes}}
| {{Yes}}
| {{No}}
| {{No}}
|
| {{Yes}}
| {{Yes}}
| {{No}}
| {{No}}
| DOS/EXE DOS/COM
| {{No}}
| {{Yes}}
| {{Yes}}
| OMF/OBJ
| {{Yes}}
|-
! [[Link]]
| {{Yes}}
| {{Yes}}
| {{No}}
| {{No}}
|
| {{Yes}}
| {{Yes}}
| {{No}}
| {{No}}
| DOS/EXE DOS/COM LE
| {{No}}
| {{Yes}}
| {{Yes}}
| CODEVIEW DWARF
| {{Yes}}
|-
! [[VAL]]
| {{Yes}}
|
|
|
|
| {{No}}
| {{No}}
| {{No}}
| {{No}}
| DOS/EXE
| {{No}}
| {{No}}
| {{No}}
| {{Yes}}
| {{No}}
|-
! [[WLink]]
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{No}}
|
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{No}}
| DOS/EXE DOS/COM LE Pharlap Novell_NLM Causeway QNX
| {{Yes}}
| {{Yes}}
| {{Yes}}
| WATCOM CODEVIEW DWARF
| {{Yes}}
|-
! [[smlrl]]
| {{No}}
| {{No}}
| {{Yes}}
| {{No}}
|
| {{Yes}}
| {{No}}
| {{Yes}}
| {{Yes}}
| DOS/COM DOS/EXE MACH-O
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{No}}
| {{Yes}}
|-
! [[LLVM|LLD]]
| 
| {{Yes}}
| {{Yes}}
| 
|
| 
| 
| 
| 
| 
| 
| 
| {{Yes}}
| 
| {{Yes}}
|}

[[Category:Tools]]
````

<a id="w001406"></a>

## W001406 — Template:Ref

Bron-ID: W001406
Namespace: Template (10)
Revisie: 1570 | Datum: 2006-11-27T08:40:06Z
Bron: https://wiki.osdev.org/Template:Ref
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ARef&oldid=1570

````text
<small><sup id="ref_{{{1}}}" class="plainlinks">[{{fullurl:{{FULLPAGENAME}}}}#endnote_{{{1|}}}]</sup></small>
````

<a id="w001407"></a>

## W001407 — Template:Note

Bron-ID: W001407
Namespace: Template (10)
Revisie: 8449 | Datum: 2009-07-09T19:31:23Z
Bron: https://wiki.osdev.org/Template:Note
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ANote&oldid=8449

````text
<includeonly><cite id="endnote_{{{1}}}" style="font-style: normal;">[[#ref_{{{1}}}|<span style="position:relative; bottom:0.25em; font-size:0.8em;">↑</span>]]</cite>&nbsp;</includeonly><noinclude>This is a template used for notes, it's used together with [[:Template:Ref|ref]].

== Example ==
=== Input ===
<pre>This is foobar.{{ref|foobar}} Another note.{{ref|another_note}}

* {{note|foobar}} Foobar is a word
* {{note|another_note}} Yes, another note</pre>

=== Output ===

This is foobar.{{ref|foobar}} Another note.{{ref|another_note}}

* {{note|foobar}} Foobar is a word
* {{note|another_note}} Yes, another note
</noinclude>
````

<a id="w001408"></a>

## W001408 — Template:In Progress

Bron-ID: W001408
Namespace: Template (10)
Revisie: 22717 | Datum: 2018-09-10T02:23:01Z
Bron: https://wiki.osdev.org/Template:In_Progress
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AIn_Progress&oldid=22717

````text
<center>
{|style="border: 1px solid #cfcfbf; margin-top: 25px; margin-bottom: 25px; background-color: #f0f0ff; text-align: center;"
|
[[image:Under_Construction.png|This page is under construction!]]
This page or section is a work in progress and may thus be incomplete. Its content may be changed in the near future.
|
|}
</center>

<includeonly>[[Category:In Progress]]</includeonly>
````

<a id="w001409"></a>

## W001409 — History

Bron-ID: W001409
Namespace: Main (0)
Revisie: 30565 | Datum: 2026-05-06T21:21:54Z
Bron: https://wiki.osdev.org/History
Revisie-URL: https://wiki.osdev.org/index.php?title=History&oldid=30565

````text
This page covers the history of Operating System Development (OSDev) as it applies to amateur, educational and smaller operating systems. Other pages cover different aspects of the history of computer operating systems. A general history of mainstream OSes can be found on [[Wikipedia:History_of_operating_systems|Wikipedia]]. A more detailed timeline and hierarchy of UNIX and similar OSes can be found at [http://www.levenez.com/unix/ the UNIX history page].

== Introduction ==
Informal operating system development has been a growing hobby over the last few years, and has become an enjoyable pass-time. Many of the developers that frequent [http://www.osdev.org osdev.org] are studying college (university) level computer science, but there are many also that have no formal background in computer science, and are self-taught.

Several factors have brought about the upswing in popularity of developing operating systems. The first is the availability of open documentation, and the access to this provided by the Internet. The second factor is the availability of cheap hardware that can be used personally for testing the systems on (since a computer must be restarted to test an operating system, this makes it hard to test on a system with multiple users).

Another possible factor is the recent popularity of GNU/Linux systems, and the [http://www.kernel.org Linux kernel] in particular. The Linux kernel was not the first kernel to open its sources to the world, but it is (and has been for the last decade at least) the most popular. Since many programmers are aware of Linux's open state, they may be more inclined to delve deeper into the kernel than they would be with a closed kernel. Today, Linux has become a mainstream kernel, with many companies investing in its development. It has become a good example of how a hobbyist kernel can become something big, given enough man-hours and the right chances.

== About the word 'OSDev' ==
The use of OSDev as an abbreviation for Operating System Development seems to have started well before OSDev-related newsgroup traffic started in 1996. Prior to its use with hobbyist development, OSDev also stood for other things; mainly On Screen Device. OSDev is now a common enough term that the [[wikipedia:Osdev|Wikipedia article]] on OSdev is about operating system development.

== Timeline of the OSDev Community ==
This is a timeline includes historical information related to websites and other communities devoted to operating system development.

* '''1995''': The [http://web.archive.org/web/20110514135936/http://www.nondot.org/sabre/os/articles Operating System Resource Center] is launched. It is no longer online.

* '''Early 1996''': Informal operating system development that didn't directly connect to an academic project started to take off.

* '''Sept/Oct 1996''': Malte Kroeger created a mailing list(os-list@aros.net) that grew to 30+ people in size.

* '''Sept/Oct 1996''': The os-list mailing list discussions were moved over to the comp.os.misc newsgroup.

* '''Sept 10 1996''': Michael Hobbs posts a message on comp.os.misc that stated ''I'm beginning to believe that there exists at least 1 OS for each person who reads this group.'' This pattern holds true 10 years later.

* '''Sept 12 1996''': It is suggested on comp.os.misc that a dedicated os development newsgroup be created. Some of the suggested names include: comp.os.design, comp.os.homebrew.develop, comp.os.development, and alt.hobby.osdev. Two names are agreed on; comp.os.development and comp.os.assembly. Eventually, the names would change to use alt instead of comp. [http://groups.google.com/group/comp.lang.asm.x86/browse_thread/thread/3bac948d6839ce04/92a8429f1d67e831?q=osdev#92a8429f1d67e831 original discussion]

* '''1997''': A web based message board was created at www.effect.net.au/os-dev/wwwboard/wwwboard.htm to host osdev discussions.

* '''1997''': One of the first OS project lists is created that contains OSes built by single developers. There were 20 projects and Linux was listed as it was still considered a small OS. [http://web.archive.org/web/19970327091052/www.effect.net.au/os-dev/oslist.html Internet Archive mirror]

* '''Sept 11 1997''': One year after being suggested the newsgroups alt.os.development and alt.os.assembly are created. [[NASM]] is recommended as the assembler of choice. [http://web.archive.org/web/19971009075251/http://www.net1.demon.co.uk/alt.os/ Archive of the original announcement and group descriptions]

* '''Dec 3 1998''': Stu George's OSFAQ is [http://groups.google.com/group/alt.os.development/browse_frm/thread/a511029156767157/43f0449f98fe5301?lnk=gst&rnum=107#43f0449f98fe5301 announced], hosted on a dialup account.

* '''Nov 14 1999''': Mega-tokyo.com is [http://groups.google.com/group/alt.os.development/browse_frm/thread/a6fa18b8d02f721d/c98172c2134ec840?lnk=gst&rnum=422#c98172c2134ec840 opened], with Stu George's OSFAQ hosted there.

* '''2000''': Forums are added to Mega-tokyo.com. For the next few years, these forums would grow to become the biggest OSDev forums. They would remain the biggest until the merger (this issue is currently under some debate in the [[topic:15219|forum]]. Certainly, there were more posts per day at mega-tokyo.com).

* '''Aug 2000''': The [http://www.osdev.org http://www.osdev.org] website is created as a place to contain os development information.

* '''Oct 18 2006''': The two largest os development forums, Mega-Tokyo.com and OSDev.org, are merged in to one single forum creating the single largest community of operating system developers working on different OSes. The original OS development newsgroups are eclipsed by several orders of magnitude in traffic.

== Famous Quotes / Usenet Posts==
* '''Aug 25 1991''': Linus Torvalds announces the Linux kernel for the first time on comp.os.minix. [https://web.archive.org/web/20100815075326/http://www.linux.org:80/info/linus.html]
* '''Oct 6 1991''': Linus Torvalds announces v0.02 of the Linux kernel. [http://groups.google.com/group/comp.os.minix/browse_thread/thread/e3df794a2bce97da]
* '''Jan 30 1992''': The famous Microkernel vs. Monolithic kernel debate. [http://groups.google.com/group/comp.os.minix/browse_frm/thread/c25870d7a41696d2/f447530d082cd95d#f447530d082cd95d]
* Bill Gates: ''The best way to prepare [to be a programmer] is to write programs, and to study great programs that other people have written. In my case, I went to the garbage cans at the Computer Science Center and fished out listings of their operating system.''
* Brian Kernighan: ''First make it run, then make it run fast.''
* Dennis Ritchie: ''UNIX is basically a simple operating system, but you have to be a genius to understand the simplicity.''

[[Category:OS Development]]
[[Category:History]]
````

<a id="w001410"></a>

## W001410 — Monolithic Kernel

Bron-ID: W001410
Namespace: Main (0)
Revisie: 30583 | Datum: 2026-05-19T15:50:22Z
Bron: https://wiki.osdev.org/Monolithic_Kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Monolithic_Kernel&oldid=30583

````text
{{Template:Kernel designs}}

==Design==
{|align="right"
|[[Image:Monolithic.png|frame|right|Basic overview of a monolithic kernel.]]
|}

A monolithic kernel includes all (or at least, most) of its services in the kernel proper.

This reduces the amount of context switches and messaging involved, making the concept faster than a [[Microkernel]]. On the downside, the amount of code running in kernel space makes the kernel more prone to fatal bugs.

The word "monolithic" by itself means a single piece (mono) that is of or like stone (lithic), however when applied to kernels the exact meaning is more general. Most people{{who}} consider that a kernel where device drivers and services run as part of the kernel is a monolithic kernel, regardless of whether parts are dynamically loaded "kernel modules" or if everything is a true single unchangeable binary. For this reason, a distinction can be drawn between "monolithic" and "pure monolithic".

Modern versions of Linux are a well-known example of a monolithic kernel - while drivers are shipped as dynamically loaded "kernel modules" they are still loaded into and running in kernel space. Monolithic kernels are common for the 80x86/PC architecture.

Examples of "pure monolithic" kernels are rare for the 80x86/PC architecture (but more common in embedded systems). This is because of the wide variety of devices, hardware and CPU features that may be present within a modern PC - a pure monolithic kernel would need to be far too large or compiled specifically for the computer before use.

In general most OS's aren't "pure monolithic" or "pure micro-kernel", but fall somewhere between these extremes in order to make use of the advantages of both methods.

==Examples==
*Linux
*MSDOS, including Windows 9x (95, 98, Me)
*Windows NT <ref>https://www.realworldtech.com/forum/?threadid=65915&curpostid=65936</ref>
*[[Mac OS X]] ([[XNU]])
*Haiku <ref>https://discuss.haiku-os.org/t/does-haiku-use-a-micro-kernel/6625/9</ref>
*BeOS <ref>https://lwn.net/Articles/495162/</ref>
*ReactOS
*DragonFly BSD <ref>https://www.dragonflybsd.org/docs/handbook/ConfigureKernel/</ref>
*Plan 9 <ref>https://en.wikipedia.org/wiki/Plan_9_from_Bell_Labs</ref>
*Most BSDs
*[[Xv6]]

[[Category:Kernel]]
````

<a id="w001411"></a>

## W001411 — Microkernel

Bron-ID: W001411
Namespace: Main (0)
Revisie: 29930 | Datum: 2026-01-25T01:36:59Z
Bron: https://wiki.osdev.org/Microkernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Microkernel&oldid=29930

````text
{{Template:Kernel designs}}
== Design ==
[[Image:Microkernel.png|frame|right|Basic overview of a microkernel]]
A Microkernel tries to run most services - like networking, filesystem, etc. - as daemons / servers in user space. All that's left to do for the kernel are basic services, like physical memory allocation (the actual memory '''manager''' may be implemented in userspace), scheduling, and messaging (Inter Process Communication).

In theory, this concept makes the kernel more responsive (since much functionality resides in preemptible user-space threads and processes, removing the need for context-switching into the kernel proper), and improves the stability of the kernel by reducing the amount of code running in kernel space. There are also additional benefits for OS' that support multi-CPU computers (much simpler re-entrancy protection and better suitability for asynchronious functionality) and distributed OS' (code can use services without knowing if the service provider is running on the same computer or not). A drawback is the amount of messaging and Context Switching involved, which makes microkernels conceptually slower than monolithic kernels. (This isn't to say that a smartly designed microkernel could not beat a foolishly designed monolithic one.)

In practice things can be quite different. For example, if the filesystem crashed, the kernel would continue running, but the user would still lose some data - unless provisions were made to restart the filesystem server / daemon, and a data recovery system exists. Microkernels can be more stable, but it requires additional design work; it does not come free with the architecture. Likewise, the additional design work that has to be done to get a microkernel design right could also be spent on making a monolithic kernel preemptable.

AmigaOS, for example, was a microkernel - and an unusual one: Since the original AmigaOS had no memory protection, its messaging was as quick as it could get (passing a pointer to memory), making the AmigaOS kernel one of the fastest ever devised. On the other hand, that lack of memory protection also meant that the microkernel architecture gave no added stability (later versions did implement MMU support, but at the same speed cost that affects other microkernel systems).

One of the side-effects of using a micro-kernel design (or a modular Monolithic Kernel), is the changes needed for booting the OS. If the file system is handled by a user-space process loaded by the kernel, the kernel itself contains no code to handle file systems or storage device drivers to load the file system process in the first place. One method of resolving this is for the boot loader to load a "RAM disk image", containing the kernel and several support files (device drivers, etc).

It's also possible for an OS design to borrow concepts from both monolithic kernels and micro-kernels in order to use the benefits of either method where appropriate.

== Examples ==
* Mach
* QNX
* [[L4]]
* AmigaOS
* [http://minix3.org Minix]

== See Also ==

=== Forum Threads ===
* [[Topic:10099|microkernels]]
* [[Topic:10234|discussing microkernel vs modular macrokernel]]

==== Recommended Reading ====
* [https://raw.githubusercontent.com/openmach/openmach.github.io/master/docs/mach/midwaytr.ps The Increasing Irrelevance of IPC performance in Microkernel-based Operating Systems] by Brian N. Bershad
* [http://www.google.com/search?q=The+Persistent+Relevance+of+IPC+Performance+for+Microkernel-Based+Operating+Systems The Persistent Relevance of IPC performance in Microkernel-based Operating Systems] by Wilson C. Hsieh, M. Frans Kaashoek, and William E. Weihl
* [https://os.itec.kit.edu/downloads/publ_1996_liedtke_ukernels-must-be-small.pdf µ-Kernels Must And Can Be Small] by Jochen Liedtke
* [https://web.archive.org/web/20190719155407/http://linuxfinances.info/info/microkernel.html Microkernel-based OS Efforts] by Christopher Browne
* [http://web.archive.org/web/20140803112320/http://i30www.ira.uka.de/~neider/edu/mkc/mkc.html Microkernel Construction Notes] by Raphael Neider

[[Category:Kernel]]
````

<a id="w001413"></a>

## W001413 — File:Microkernel.png

Bron-ID: W001413
Namespace: File (6)
Revisie: 1579 | Datum: 2006-11-27T23:27:28Z
Bron: https://wiki.osdev.org/File:Microkernel.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AMicrokernel.png&oldid=1579

````text
Basic concept of microkernel.
````

<a id="w001414"></a>

## W001414 — File:Monolithic.png

Bron-ID: W001414
Namespace: File (6)
Revisie: 1581 | Datum: 2006-11-27T23:33:25Z
Bron: https://wiki.osdev.org/File:Monolithic.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AMonolithic.png&oldid=1581

````text
Basic overview of a monolithic kernel.
````

<a id="w001415"></a>

## W001415 — Processes and Threads

Bron-ID: W001415
Namespace: Main (0)
Revisie: 26612 | Datum: 2021-10-27T21:12:29Z
Bron: https://wiki.osdev.org/Processes_and_Threads
Revisie-URL: https://wiki.osdev.org/index.php?title=Processes_and_Threads&oldid=26612

````text
==Processes/Tasks==
'''Processes''' are running programs, including code, data, heap, and [[stack]]. In most implementations (but not always), each process has its own virtual address space (i.e. its own logical address-to-physical memory mapping) and its own set of system resources (files, environment variable, etc.). It is fairly common to for each process to have several threads (see below). This way there is a process to maintain the address space and several threads to control the process' execution.

==Threads==
{{Main|Thread}}
A '''thread''' is a control flow in an executable image. Threads can be "user level" (i.e., the process handles multiple threads within itself) or "kernel level" (i.e., the OS scheduler handles multiple threads within a single process). Two threads from the same process naturally share the same code and global data but are given different [[stack]]s so that they do not interfere with each other's local variable and may have their own call chain.

==Task Models==
*[[Monotasking Systems]]
*[[Multitasking Systems]]
*[[Real-Time Systems]]
*[[Continuation Systems]]

==Scheduling==
* [[Context Switching]]
* [[Scheduling Algorithms]]
* [[Multiprocessor Scheduling]]
* [[Blocking Process]]

==See Also==
*[[Kernel Multitasking]]
*[[Cooperative Multitasking]] tutorial
*[[:Category:IPC|IPC]]
*[[:Category:Synchronization|Synchronization]]

[[Category:Processes and Threads]]
````

<a id="w001417"></a>

## W001417 — Template:Quotation

Bron-ID: W001417
Namespace: Template (10)
Revisie: 28987 | Datum: 2024-06-06T17:24:22Z
Bron: https://wiki.osdev.org/Template:Quotation
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AQuotation&oldid=28987

````text
{| class="toccolours" align="center" WIDTH="75%" style="padding: 20px 30px 10px 10px;"
|- 
| 
<div class="scroll">{{{1}}}</div>
<p style="text-align: right;">
{{if | {{{2|}}} | —{{{2}}}}}
</p>
|}
````

<a id="w001418"></a>

## W001418 — Context Switching

Bron-ID: W001418
Namespace: Main (0)
Revisie: 28474 | Datum: 2023-11-21T04:00:11Z
Bron: https://wiki.osdev.org/Context_Switching
Revisie-URL: https://wiki.osdev.org/index.php?title=Context_Switching&oldid=28474

````text
In your average, memory-protected environment, a "context" is a virtual address space, the executable contained in it, its data etc.

A "context switch" occurs for a variety of reasons - because a kernel function has been called, the application has been preempted, or because it had yielded its time slice.

A context switch involves storing the old state and retrieving the new state. The actual information stored and retrieved may include EIP, the general registers, the segment registers, CR3 (and the paging structures), FPU/MMX registers, SSE registers and other things. Because a context switch can involve changing a large amount data it can be the one most costly operation in an operating system.

There are many ways of performing a context switch. The x86 CPU provides a way of doing it completely in hardware, but for performance and portability reasons most modern OS's do context switches in software.

==Software Context Switching==

Software context switching can be used on all CPUs, and can be used to save and reload only the state that needs to be changed. The basic idea is to provide a function that saves the current [[Stack|stack pointer]] (ESP) and reloads a new stack pointer (SS:ESP). When the function is called EIP would be stored on the old stack and a new EIP would be popped off the new stack when the function returns. Of course the operating system usually needs to change much more than just the stack and EIP.

Note how preemption occurs in an interrupt handler. If your OS saves register state for interrupt handlers, then if you stick a setjmp / longjmp in your scheduler, you can jump into the interrupt handler of the IRQ that preempted the process you're switching to. Then just return.

Eflags, the general registers and any data segment registers should also be pushed on the old stack and popped off the new stack. If the paging structures need to be changed, CR3 will also need to be reloaded.

The FPU/MMX and SSE state could be saved and reloaded, but the CPU can also be tricked into generating an exception the first time that an FPU/MMX or SSE instruction is used by copying the hardware context switch mechanism (setting the TS flag in CR0).

===Details===

When the CPU changes to a higher privilege level (CPL 0 being the highest) it will load new values for SS and ESP from the [[Task State Segment]] (TSS). '''If the operating system uses multiple privilege levels it must create and load a TSS'''. An interrupt generated while the processor is in ring 3 will switch the stack to the resulting permission level stack entry in the TSS.  During a software context switch the values for SS0:ESP0 (and possibly SS1:ESP1 or SS2:ESP2) will need to be set in the TSS.  If the processor is operating in [[Long Mode]], the stack selectors are no longer present and the RSP0-2 fields are used to provide the destination stack address.

If a context switch also entails a change in IO port permissions, a different TSS may be loaded for each [[wikipedia:Process_(computing)|Process]].  When running virtual 8086 tasks, the IO permission map in the TSS isn't checked to provide I/O port protection.  IO protection can be implemented by setting the IO Permission Level to 0.  This will generate a [[Exceptions#General_Protection_Fault|General Protection Fault]] when a process in ring 3 attempts to write to or read from an IO port.  The GP fault handler can then check permissions and carry out the port IO on behalf of the user-mode code.

===Other Possibilities===

During a context switch the operating system can do additional work that isn't strictly part of the context switch. One common thing is calculating the amount of time the last thread/task/process used so that software (and the end user) can determine where all the CPU time is going. Another possibility would be dynamically changing thread/task/process priorities.

===Performance Considerations===

Translating a virtual address to a physical address is expensive.  The processor must access the pages table structures, which usually have 3-4 levels.  Thus, a single memory access actually requires 4-5 memory accesses.

To mitigate this issue, most modern processors cache virtual-to-physical translations in a translation lookaside buffer ([[TLB]]).  The TLB is part of the MMU and is (mostly) transparent to the system developer and users.

When virtual memory is updated -- for instance, when one process's address space is replaced with another's during a software context switch -- the TLB suddenly contains "stale" translations that are no longer valid.  These translations must be flushed for correct behavior.  Writing to CR3 will flush the TLB.  However, by writing to CR3, you also eliminate all translations for the kernel, in addition to the last user process.  This is less than ideal, as the next few operations must wait for the slow virtual-to-physical translations.

Recent Intel and AMD processors sport a tagged TLB, which allow you to tag a given translation with a certain address space configuration.  In this scheme TLB entries never get "stale", and thus there is no need to flush the TLB.

==Hardware Context Switching==

Some CPU's have a special mechanism to perform context switches in hardware. The following information gives details on 80x86 CPU's only.

The hardware context switching mechanism (called Hardware Task Switching in the CPU manuals) can be used to change all of the CPU's state except for the FPU/MMX and SSE state. To use the hardware mechanism you need to tell the CPU where to save the existing CPU state, and where to load the new CPU state. The CPU state is always stored in a special data structure called a TSS (Task State Segment).

To trigger a context switch and tell the CPU where to load its new state from, the far version of CALL and JMP instructions are used. The offset given is ignored, and the segment is used to refer to a "TSS Descriptor" in the GDT. The TSS descriptor is used to specify the base address and limit of the TSS to be used to load the new CPU state from.

The CPU has a register called the "TR" (or Task Register) which tells which TSS will receive the old CPU state. When the TR register is loaded with an "LTR" instruction the CPU looks at the GDT entry (specified with LTR) and loads the visible part of TR with the GDT entry, and the hidden part with the base and limit of the GDT entry. When the CPU state is saved the hidden part of TR is used.

===A step further with Hardware Switches ...===

In addition to the CALL and JMP instructions, a context switch can be triggered by a using a Task-Gate Descriptor. Unlike TSS Descriptors, task-gate descriptors can be in the GDT, LDT or IDT. Normally, task-gate descriptors are used in the IDT, so that an exception (or IRQ) can cause a context switch, which is the only way of handling a double fault exception with complete reliability.

The design of the basic hardware mechanism is limited by the number of usable entries in the GDT because TSS descriptors can be in the GDT only (theoretical limit is 8190 tasks). However, it is possible to avoid this restriction by dynamically changing TSS descriptor/s, by setting the TSS descriptor's base before each context switch. Care must be taken when using this approach when task-gate descriptors in the IDT are also used (the TSS descriptors referred to by each task-gate descriptor would have to be constant). Also context switches can't be initiated with a CALL instruction, because the CPU saves the GDT entry to use for the return in the TSS's "backlink" field.

If the FPU/MMX and SSE state also needs to be changed during a context switch there are a few options. The data could be explicitly saved by any code that causes a context switch, or the CPU can generate an exception the first time an FPU/MMX or SSE instruction is used. With the second option, the exception handlers would save the old FPU/MMX/SSE state and reload the new state. This option may prevent this data from being changed when it's not necessary (for e.g. when no tasks or only one task is using them), but fails to work correctly in a multiprocessor environment without additional synchronization which may be more expensive than using the first option.

===Performance Considerations===

Because the hardware mechanism saves almost all of the CPU state it can be slower than is necessary. For example, when the CPU loads new segment registers it does all of the access and permission checks that are involved. As most modern operating systems don't use segmentation, loading the segment registers during context switches may be not be required, so for performance reasons these operating systems tend not to use the hardware context switching mechanism. Due to it not being used as much CPU manufacturers don't optimize CPUs for this method anymore (AFAIK). In addition the new 64 bit CPU's do not support hardware context switches when in 64 bit/long mode.

[[Category:Processes and Threads]]
[[Category:Multitasking]]
````

<a id="w001419"></a>

## W001419 — QEMU

Bron-ID: W001419
Namespace: Main (0)
Revisie: 30887 | Datum: 2026-07-13T04:17:34Z
Bron: https://wiki.osdev.org/QEMU
Revisie-URL: https://wiki.osdev.org/index.php?title=QEMU&oldid=30887

````text
{{Emulators}}

QEMU is a free and open-source emulator that performs hardware virtualization. It is widely available for variety of host operating-systems and requires minimal configuration for use in operating-system development. It is capable of emulating a wide variety of systems including [[ARM]], [[x86]] and [[RISC-V]], among others. For a more comprehensive list of targets refer to the [https://www.qemu.org/docs/master/ official documentation].


==Features==
* Supports multiple hosts, multiple targets.
* Two operating modes: full system emulation (of interest for operating-system development) and Linux user process emulation (of interests to people who want to emulate applications).
* Uses 'just in time' code compilation technique (allowing reuse of previous interpretation). This makes it faster than the popular alternative emulator [[Bochs]].
* Provides native [[GDB]] support. QEMU can be attached to [[GDB]]/[[DDD]] to debug the guest operating-system. Refer to the [[QEMU#GDB-Stub|GDB-Stub]] section below for more information.
* Supports [[VBE]] 2.0.

===VBE Support===

[[VBE]] support be checked by typing '<tt>vbeprobe</tt>' on the [[GRUB]] command-line. The test returns:

{| class="wikitable"
! Mode
! Format
! Resolution
|-
|0x101
|Packed pixel
|640x480x8
|-
|0x110
|Direct Color
|640x480x15
|-
|0x111
|Direct Color
|640x480x16
|-
|0x112
|Direct Color 	
|640x480x24
|-
|0x103
|Packed pixel 	
|800x600x8
|-
|0x113
|Direct Color 	
|800x600x15
|-
|0x114
|Direct Color 	
|800x600x16
|-
|0x115
|Direct Color 	
|800x600x24
|-
|0x105
|Packed pixel 	
|1024x768x8
|-
|0x116
|Direct Color 	
|1024x768x15
|-
|0x117
|Direct Color 	
|1024x768x16
|-
|0x118
|Direct Color
|1024x768x24
|-
|0x107
|Packed pixel 	
|1024x768x8
|-
|0x119
|Direct Color 	
|1024x768x15
|-
|0x11A
|Direct Color 	
|1024x768x16
|}

==Supported Architectures==
* [[:Category:X86|x86]]
* [[X86-64|x86_64]]
* [[:Category:ARM|ARM]]
* [[AArch64|ARM64]]
* [[Alpha]]
* LoongArch
* [[LatticeMico32]]
* [[M68k_Overview|Motorola 68000]]
* [[MicroBlaze]]
* [[MIPS]]
* [[MIPS64]]
* [[Moxie]]
* [[PA-RISC]]
* [[PowerPC]]
* [[PowerPC64]]
* [[RISC-V]]
* [[IBM System/390]]
* [[SuperH]]
* [[:Category:Sparc|SPARC]]
* [[SPARC64]]
* [[TriCore]]
* [[Unicore]]
* [[Xtensa]]

==Supported Devices==
==== Non-Display Hardware ====
* [[Ne2000]] network card
* [[Intel_Ethernet_i217|e1000]] network cards
* [[RTL8139]] network card
* [[Intel 8254x]] network cards
* [[AMD_PCNET|AMD PCnet]] network cards
* [[PC Speaker]]
* [[Sound Blaster 16]] sound cards
* [[AC97]]
* [[Intel High Definition Audio]]
* [[Virtio]] devices
* PCI support (With BIOS32).
* [[QEMU_fw_cfg]] Allows you to pass info to the guest

==== Display Adapters ====
Note it is possible to have more than display adapter attached to the virtual machine. However, only one can have VGA compatibility. For example: <code>-device VGA -device ati-vga,model=rage128pro</code> won't work, but <code>-device bochs-display -device ati-vga,model=rage128pro</code> will work.

{| class="wikitable"
|+
!QEMU Device Name
!Emulates Real Hardware
!VGA Compatibility
!Notes
!See
|-
|vga
|No
|Yes
|QEMU's standard video device. Emulates both the standard 1987-era VGA hardware, as well as the Bochs Display Adapter (which is an easy way to get a framebuffer for your OS if you're not using BIOS/VBE/UEFI). Look for the PCI device with ID 1234:1111 and set registers via PCI MMIO, then access the framebuffer pointed to in the PCI BARs. Alternatively use the VGA port I/O registers, BIOS ROM, and memory area if you're only interested in the VGA compatible hardware and ignore the Bochs extensions.
|[[Bochs VBE Extensions]] / [[VGA Hardware]]
|-
|isa-vga
|No
|Yes
|This is the same device as 'vga', but without support for PCI. Access to Bochs extension registers is via port I/O, with the framebuffer hardcoded at 0xE0000000.
|[[Bochs VBE Extensions]] / [[VGA Hardware]]
|-
|bochs-display
|No
|No
|This is the Bochs display hardware that the 'vga' and 'isa-vga' devices have, but without the VGA hardware backwards compatibility. This hardware will only work with UEFI firmware, as BIOS firmware requires VGA compatibility.
|[[Bochs VBE Extensions]]
|-
|secondary-vga
|No
|No
|''As far as I can tell this is the same device as 'bochs-display' with a different name, but more research is required. - [[User:Jackscott]]''
|[[Bochs VBE Extensions]]
|-
|ramfb
|No
|No
|A very simple graphics device that just has a framebuffer in memory to write to.
|[[Ramfb]]
|-
|vmware-svga
|No
|Yes
|VMWare's virtual video hardware, with similar intentions to the Bochs Display Adapter. Unless you're targetting VMWare as a hypervisor platform, this device is largely deprecated.
|[[VMWare SVGA-II]]
|-
|qxl-vga
|No
|Yes
|Another virtual-only video card a bit more modern (just) than the Bochs and VMWare devices. It has basic 2D acceleration and multi-monitor support.
|
|-
|qxl
|No
|No
|The same device as the 'qxl-vga' device, but without VGA backwards compatibility. Used mainly for adding secondary video cards to Windows 9x guests.
|
|-
|cirrus-vga
|Yes
|Yes
|Emulates the Cirrus Logic CLGD 5446 graphics card on the PCI bus.
|[http://www.bitsavers.org/components/cirrusLogic/graphics/Alpine_VGA_Family_CL-GD543X_4X_Technical_Reference_Manual_4ed_Feb95.pdf Technical Reference Manual]
|-
|isa-cirrus-vga
|Yes
|Yes
|An ISA version of the Cirrus Logic CLGD 5446 graphics card.
|[http://www.bitsavers.org/components/cirrusLogic/graphics/Alpine_VGA_Family_CL-GD543X_4X_Technical_Reference_Manual_4ed_Feb95.pdf Technical Reference Manual]
|-
|ati-vga
|Yes
|Yes
|Emulates either an ATI Rage 128 Pro ('model=rage128p') or an ATI Radeon RV100 ('model=rv100'). This device is emulated on the AGP bus.
| [[ATI Rage 128]]
|}

===== VirtIO Display Adapters =====

QEMU also supports [[Virtio GPU]] devices, with many permutations based on whether the emulated device has support for 3D operations (the <code>-gl</code> options), support for legacy VGA compatibility (the <code>-vga</code> options), and whether the device is exposed on the PCI bus (the <code>-pci</code> options).

The Virtio GPU supports double buffering, multiple display outputs, and a hardware cursor.

{| class="wikitable"
|+
!QEMU Device Name
!3D Support?
!VGA Compatibility?
!PCI Device?
!Notes
|-
|virtio-gpu-device
|No
|No
|No
|
|-
|virtio-gpu-pci
|No
|No
|Yes
|
|-
|virtio-vga-device
|No
|Yes
|No
|
|-
|virtio-vga-pci
|No
|Yes
|Yes
|
|-
|virtio-gpu-gl-device
|Yes
|No
|No
|
|-
|virtio-gpu-gl-pci
|Yes
|No
|Yes
|This is the recommended device for modern UEFI guest operating systems (such as Linux or Windows 8+).
|-
|virtio-vga-gl-device
|Yes
|Yes
|No
|
|-
|virtio-vga-gl-pci
|Yes
|Yes
|Yes
|
|}

==Usage==

QEMU does not require the use of a configuration script like Bochs. An example usage of QEMU can be seen below:

<syntaxhighlight lang="bash">
qemu-system-i386                                 \
  -accel tcg,thread=single                       \
  -cpu core2duo                                  \
  -m 128                                         \
  -no-reboot                                     \
  -drive format=raw,media=cdrom,file=myos.iso    \
  -serial stdio                                  \
  -smp 1                                         \
  -usb                                           \
  -vga std
</syntaxhighlight>

When running QEMU on a non-UNIX system, it may be necessary to use the <tt>-L</tt> command-line option to instruct QEMU where to find a BIOS image.

The <tt>-m 128</tt> argument instructs QEMU to create the guest system with 128MB of RAM. The <tt>-drive format=raw,media=cdrom,file=myos.iso</tt> argument instructs QEMU to create a drive in our guest system. In this case the created drive is a CDROM drive, with <tt>myos.iso</tt> as its loaded media. If more than one drive in is required in the guest system, it is possible to use the <tt>-boot</tt> option to define the order in which the system will check for bootable media.

The <tt>-serial stdio</tt> argument used above instructs QEMU to redirect the serial input and output to the host system's <tt>stdio</tt> stream. This is particularly useful for debugging purposes. In the documentation, the option to redirect this to <tt>stdio</tt> is described as Unix only. Presumably, one should be able to achieve a similar effect with <tt>-serial file:CON</tt> on Windows.

In order to help track down the source of a triple fault, the <tt>-d int</tt> option can be used to show additional debugging information on interrupts that occur. Additionally, the <tt>-no-shutdown</tt> and <tt>-no-reboot</tt> options can be used. These instruct the virtual machine to halt on error instead of rebooting or shutting down, allowing for the operator to inspect the state of the machine after a triple-fault.

Additional useful command-line options are listed within the [[QEMU#Useful_QEMU_command-line_options|Useful QEMU command-line options]] section.

The keyboard shortcuts <tt>CTRL-ALT-{1,2,3}</tt> can be used inside the emulator to swap in/out of the emulation screen, the QEMU monitor and a serial console. Additional debugging commands can be issued from the system console, such as changing disk images and performing memory dumps, among others.

===The debug console===

Instead of the serial port, as outlined above, one can also use the <tt>debugcon</tt> device. By running QEMU with <tt>-debugcon stdio</tt>, you can print a message to your host system's terminal by just writing each byte to port 0xe9. This is the same port as the one used by [[Bochs#I/O_debugger_macros|the equivalent feature in Bochs]].

Unlike the serial port, the debug console is one-way – you can only output. On the flip side, implementing support is much easier, and outputting the messages isn't artificially throttled by a baudrate.

===The QEMU monitor===

{{Main|QEMU Monitor}}


QEMU features its own internal 'monitor' console for debugging the guest operating-system. Through various commands, the monitor allows you to inspect the running guest OS, change removable media and USB devices, take screenshots and audio grabs, and control various aspects of the virtual machine. The monitor can be accessed by the key combination <tt>CTRL-ALT-2</tt> within QEMU.

It is possible to redirect QEMU's monitor output to stdio using the following command-line option when invoking QEMU:
<syntaxhighlight lang="bash">
-monitor stdio
</syntaxhighlight>


Some useful commands:
; xp : eXamine Physical memory.  Much like GDB's <tt>x</tt> command, but with no address translation.
; cpu n : switch to CPU n.  Note that GDB's threads are numbered from 1, but QEMU's CPUs are numbered from 0.
; info registers : dump register state
; info tlb : Show virtual memory translation state.
; info mem : Show the page table mappings in a compact form.
; help : List all commands -- keep in mind that there may be more commands available than those mentioned in the [https://www.qemu.org/docs/master/system/monitor.html QEMU documentation].


A full list of the capabilities of QEMU's monitor is available in the [https://www.qemu.org/docs/master/system/monitor.html official documentation] as well as [https://en.wikibooks.org/wiki/QEMU/Monitor here].

==Useful QEMU command-line options==

''Shown below are a list of command-line options for QEMU which are of special significance to Operating-System development. For a full list of options refer to the [https://qemu.weilnetz.de/doc/qemu-doc.html official QEMU documentation].''

{| class="wikitable"
|-
! Option
! Description
|-
| <tt>-no-reboot</tt>
| Prevents QEMU from rebooting in the event of a [[Triple Fault]].
|-
| <tt>-no-shutdown</tt>
| Don’t exit QEMU on guest shutdown, but instead only stop the emulation.
|-
| <tt>-d</tt>
| Enables the printing of extra debugging information. Arguments for this option include <tt>cpu_reset, int, guest_errors</tt> ( among others ). This can be extremely useful when setting up an [[IDT]] to see interrupt execution in real-time.
|-
| <tt>-gdb</tt> or <tt>-s</tt>
| Launches QEMU in GDB Stub mode. This causes QEMU to accept incoming connections from a [[GDB]] client. See below for more information or refer to [https://wiki.qemu.org/Documentation/Debugging official documentation].
|-
| <tt>-S</tt>
| Causes the guest CPU not to start execution on startup. This is very useful for debugging as it launches the guest in a paused state. The user must use the <tt>continue</tt> command in the console ( or GDB ) to initiate execution on the guest system.
|}

==GDB-Stub==

Starting QEMU with the <tt>-gdb</tt> or <tt>-s <dev></tt> command-line options will instruct QEMU to listen for an incoming [[GDB]] connection. By default QEMU will listen for a connection over HTTP on <tt>localhost:1234</tt>, but the option will accept an argument for a different connection. It is useful to use this switch together with the <tt>-S</tt> option, which will cause QEMU to start in a paused state. This will allow additional time to connect a GDB client, to start the simulation a <tt>continue</tt> command will need to be issued within GDB or in the QEMU console.

For convenience, it is also possible to create a file containing commands for GDB to automatically execute. GDB will read and execute the contents of a file with the name <tt>.gdbinit</tt> in the current working directory. Alternatively, a different file may be specified by the use the <tt>-command=file</tt> command-line argument. An example file can be seen below:

<pre>
file <my-kernel-binary>
target remote localhost:1234

# Inspect page tables
x /8wg &page_tables_start
</pre>

This will automatically load the kernel binary's symbol file into the debugger and then open the remote connection to QEMU.<br/>
Ensure that the kernel was compiled with debugging symbols included. This can be accomplished via the use of the GCC option <tt>-g</tt>. If you find that the debugger can't find local variables, try using the <tt>-fno-omit-frame-pointer</tt> option during compilation, or disable optimizations.

When running an SMP kernel, the <tt>info threads</tt> and <tt>thread</tt> commands can be used to provide useful information. It is also possible to use the QEMU monitor and its commands using the <tt>monitor</tt> command in GDB. For a list of available commands and their description, use the <tt>monitor help</tt> command.

==Tracing==

Tracing in QEMU allows you to view logs for many different peripherals that use the system.
You can enable tracing with the argument <tt>--trace "prefix*"</tt>. The prefix depends on the peripheral you're trying to debug.

For example, the AHCI peripheral (<tt>hw/ide/ahci.c</tt>) uses functions such as <tt>ahci_mem_read_ahci</tt>.
You can enable tracing for this peripheral with the command line argument <tt>--trace "ahci_*"</tt>

<pre>
ahci_mem_read_32_host ahci(0x5a1b5d4a6370): mem read [reg:CAP] @ 0x0: 0xc0141f05
ahci_mem_read_32 ahci(0x5a1b5d4a6370): mem read @ 0x0: 0xc0141f05
ahci_mem_read ahci(0x5a1b5d4a6370): read4 @ 0x0: 0x00000000c0141f05
</pre>

==Getting detailed logs==

It is possible to enable the output of additional debugging information to stdout by uncommenting certain preprocessor directives within the QEMU source files.

Some files within the QEMU source code have commented lines of the form:
<syntaxhighlight lang="c">
// #define DEBUG_*
</syntaxhighlight>

Recompiling QEMU from source with this preprocessor variable defined will cause additional debugging information from the applicable file to be printed to stdout.

==See Also==

===Articles===
*[[QEMU and GDB in long mode]]
*[[QEMU_fw_cfg]] Pass strings and files into the VM from the QEMU command line

===External Links===
*[http://www.qemu.org/ QEMU homepage]
*[http://qemu-forum.ipi.fi/ QEMU forum]
*[https://en.wikibooks.org/wiki/QEMU QEMU Wikibook]
*[https://wiki.archlinux.org/index.php/QEMU#Creating_a_hard_disk_image QEMU at ArchWiki]
* [https://www.qemu.org/docs/master/specs/index.html System Emulation Guest Hardware Specifications — QEMU documentation] List of hardware emulated by QEMU
* [https://www.kraxel.org/blog/2019/09/display-devices-in-qemu/ VGA and other display devices in QEMU] Overview of emulated display devices
* [https://www.reddit.com/r/UTMapp/comments/1fl293h/quick&#x20;overview&#x20;of&#x20;every&#x20;qemu&#x20;display&#x20;adapter&#x20;in/ Quick overview of every QEMU display adapter in UTM (for x86-based guests) : r/UTMapp]

[[Category:Emulators]]
[[Category:QEMU]]
[[de:QEMU]]
````

<a id="w001421"></a>

## W001421 — OSDev Wiki:Editing

Bron-ID: W001421
Namespace: OSDev Wiki (4)
Revisie: 28451 | Datum: 2023-11-07T12:38:30Z
Bron: https://wiki.osdev.org/OSDev_Wiki:Editing
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDev_Wiki%3AEditing&oldid=28451

````text
== Getting edit permissions ==

:''Full article: [[OSDev Wiki:Joining|Joining]]''
To prevent spam, this wiki does not allow anonymous edits. If you would like to edit this wiki, please sign up for a forum account and join the wiki group.

== Trying out features ==

You can use the [[OSDev Wiki:Sandbox|Sandbox]] to experiment with MediaWiki syntax. You can also make pages underneath your username, just click on your name on the top menu. You can read more about the editing features: [[wikipedia:Help:Wikitext examples|Wikipedia Editing Help]].

== Starting a new page ==

New pages can be started by following a link to a non-existent page which leads to an edit page. Users are encouraged to start new articles while paying close attention to the overall organization of the OSDev Wiki.

When creating a new article, you should bear in mind that:
* The title that you choose is important, as this will be the title of your new article.
* Un-doing mistakes is rarely easier than avoiding them, so it is always a good idea to do a little homework first.
* If you just want to experiment, use the [[OSDev Wiki:Sandbox|sandbox]] first.

== Editing features ==

=== Interwiki links ===

You can create interwiki links to Wikipedia: <nowiki>[[wikipedia:Article name|Link title]]</nowiki>. You can create links to forums: <nowiki>[[forum:8|OSDev Wiki Forum]]</nowiki>, or to a forum topic: <nowiki>[[topic:17323|Topic on OSDev logos]]</nowiki>.

=== Signing discussions ===

When writing in the Discussion Tab on the wiki you can sign it with your name and a timestamp using four tildes (~), like this: <nowiki>-- ~~~~</nowiki>

== More information ==

* [[OSDev Wiki:Manual Of Style|Manual Of Style]]
* [[forum:8|OSDev Wiki Forum]] - Forum for the users and maintainers of this Wiki.

[[Category:Wiki administration|{{PAGENAME}}]]
````

<a id="w001422"></a>

## W001422 — OSDev (foreign language)

Bron-ID: W001422
Namespace: Main (0)
Revisie: 28595 | Datum: 2024-02-01T10:45:40Z
Bron: https://wiki.osdev.org/OSDev_(foreign_language)
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDev_%28foreign_language%29&oldid=28595

````text
==French==
* http://a.michelizza.free.fr/pmwiki.php?n=TutoOS.TutoOS
* http://sos.enix.org/fr/SOSDownload

==German==
* http://www.lowlevel.eu/ (Forum: [http://forum.lowlevel.eu/]; IRC: irc.euirc.net, #lost)
* http://www.henkessoft.de/ (Forum: [http://www.c-plusplus.de/forum/f62])

==Japanese==
* http://wiki.osdev.info/

==Polish==
* http://osdev.pl/ - Site about Open Source, doesn't seem to be really about OSDev.

==Portuguese==
* http://www.dei.isep.ipp.pt/~orlando/so2/

==Russian==
* http://osrc.info
[[Category:Resources]]
````

<a id="w001425"></a>

## W001425 — Assembly

Bron-ID: W001425
Namespace: Main (0)
Revisie: 26136 | Datum: 2021-06-13T16:02:07Z
Bron: https://wiki.osdev.org/Assembly
Revisie-URL: https://wiki.osdev.org/index.php?title=Assembly&oldid=26136

````text
Down to the very core of computers, every signal sent to any part of the machine is a series of electrical pulses. If the electrical pulse is high, for example 5 volts, that means a binary 1 digit is sent. If the electrical pulse is low, for example 0 volts, that means a binary 0 digit is sent. Put 4 binary digits together and you get a nibble. Put 8 together, you get a byte. Put 16 together, you get a word. Computers understand commands in such segments, but humans would rather not. So, they developed a better way of viewing such information and that was with hexadecimal. First of all, it didn't take 16 characters to show one command and in general, it became more convenient to remember commands like:
 90 - No Operation
 CC - Break
 E9 - 16 bit jump
 CD - Interrupt
 etc.
But soon, it was understood that such codes needed to be user-friendly, so they invented assembly code. With assembly code, you can represent operations with special codes that are more human-readable. For example, JMP jumped, INT interrupted, NOP did no operation, etc.

An assembler is the program that reads an assembly file (usually with the extension .asm) and converts it to a binary executable. This is very much akin to a compiler generating a executable program. However, assemblers are considered separately from compilers because compilers hide the low-level details of your system, while assemblers expose you to these details.

The source code written for any assembler is defined to be "Assembly language." But that doesn't mean much, since each assembler may require different source code for exactly the same program on an exactly identical platform.

== Assembly Syntax ==
In assembly, every instruction (or operation/operator/opcode) is followed by a comma-separated list of parameters (or operands). These operands can be either immediate values (i.e. numbers such as 1, 2, [[Hexadecimal Notation|0x]]16, 0101[[Binary Notation|b]]), or registers, or a memory location. For example,
<source lang="asm">
mov eax, 123
</source>
The instruction is mov, the operands are eax and 123. Here eax is a register and 123 is an immediate value.

There are two main syntaxes for assembly: the Intel syntax, and the AT&T syntax.

=== Intel Syntax ===
Generally speaking, the first operand of an operation is the destination operand, and the other operand (or operands) is the source operand. For example:
<source lang="asm">
mov eax, 123
</source>
The mov instruction takes a value from the source operand, and places it in the destination operand, so the value 123 would be placed into the register eax. The original Intel syntax is that of the Intel ASM386 assembler (later licensed out to RadiSys but which eventually died off). Since then, many assemblers invented many variations of it. Also, it is used in most online assembly tutorials because the majority of those tutorials were written when [[TASM]] (which uses the Intel syntax) was the dominant assembler.

=== AT&T Syntax ===
AT&T syntax is the reverse of Intel syntax. The data operated on moves from left to right. Here is the same statement as above in AT&T syntax.
<source lang="asm">
mov $123, %eax
</source>
There are also some other differences with operands:

* Literal values such as 123 are prefixed with '$' (see example above).
* Memory locations have no prefix: ''mov 123, %eax'' moves the value stored at 123 to eax.
* Registers are prefixed with '%'
* When using a register as a pointer, it is put in parenthesis: ''mov (%eax), %ebx'' moves the value stored at the pointer stored in eax to ebx.
* When using a statement with memory locations (with no register present), a suffix is needed after the instruction. For example: movl $123, 123. The 'l' (stands for long) and is needed to tell the assembler that the values are 32 bits.
* Pointers can be offset by constants by prefixing them: 4(%eax) points to 4+%eax.
* Pointers can be offset by another register by writing them as: (%eax,%ebx) which points to %eax+%ebx
* Finally you can combine it all and throw in a scale if you want: 4(%eax,%ebx,2) would be 4+%eax+%ebx*2

== See Also ==

=== Articles ===
* [[:Category:Assemblers|Assemblers]]
* [[Inline Assembly]]
* [[Opcode syntax]] for more detail on AT&T syntax differences
* [[Learning 80x86 Assembly]]: list of freely available online resources to help with learning 80x86 assembly language programming

=== External Links ===
* [http://www.gnu.org/software/binutils/ GNU Binutils] - A collection of free softwares including the GNU assembler (gas)
* [http://sourceware.org/binutils/docs/as gas manual] - Official manual for the GNU assembler (gas)
* [http://www.nasm.us/ NASM] - Official site of Netwide Assembler:NASM
* [[wikibooks:X86_Assembly|x86_Assembly]] - A wikibook on writing assembly for x86 based PCs
* [http://www.plantation-productions.com/Webster/www.artofasm.com/index.html The Art of Assembly Language Programming], online book, 16-bit DOS edition. (Later editions focus on a high-level assembler dialect created by the author.)
* [http://www.drpaulcarter.com/pcasm/ PC ASM], online book, 32 bit protected mode assembly on x86
* [http://ref.x86asm.net x86 Opcode and Instruction Reference], a database of the instructions and opcodes as free viewable, searchable, and downloadable XML files. They also sell a hardcopy version.

[[Category:Assembly]]
[[Category:Languages]]
[[de:Assembler]]
````

<a id="w001429"></a>

## W001429 — 3c90x

Bron-ID: W001429
Namespace: Main (0)
Revisie: 22698 | Datum: 2018-09-10T01:35:54Z
Bron: https://wiki.osdev.org/3c90x
Revisie-URL: https://wiki.osdev.org/index.php?title=3c90x&oldid=22698

````text
{{stub}}
==3c90x NIC==
This page is about the 3c90x/3c90xB/3c90xC family of ethernet adaptors manufactured by 3com. A number of 3com cards share the programming interface of this family; this includes the 3c556 10/100LAN + 56k Modem MiniPCI adapter (also known as 3CN3AC1556B and 3Com® Megahertz® 10 Mbps LAN CF+ Card) and 3c555 10/100LAN MiniPCI adapters, which support the 3c90xC interface.

===The Card===
* 3c90x (FastEtherLink XL) is a successor to 3c59x (which are successor to 3c5x9). A new feature introduced in this series is full bus master operation mode with seperate download and upload buffer chains.
* 3c90x support PIO for compatibility; 3c90xB and 3c90xC do not.
* the adapter can be setup to wake the system upon reception of packets with a programmable filter.
* VLAN support.
* TCP/IP checksum checking support.

There are three register mechanisms which enable the host to access and configure the NIC: PCI configuration registers, host registers (IO/memory space) and the NIC's EEPROM. Issuing a command to the NIC is achieved by writing to the write-only command register (a host register).
The read-only interrupt status register informs about the interrupt source, the current register bank visible in the window and whether a command is being executed. The 3x90xC provides also the IntStatusAuto register, reading of which returns the value of the interrupt status register and causes side effects which are supposed to improve interrupt handling efficiency.
Bytes 0-15 of the host registers form a switchable window, which means the meanings of particular bytes/words/dwords in the window depends on the currently selected register bank.
The docs are well written and easy to follow.

===Programming considerations===
* For 3c555, 3c556, 3c575 and 3c656 cards, the EEPROM command for "Read" has value 0x230 (and not 0x80 like for 3c90x cards).

===Resources===
* [http://www.koders.com/c/fidF032F73835297E3B6018361AB99075FE9753E884.aspx Etherboot 3c90x driver]
* [http://www.koders.com/c/fid9EA43C2ED859EDDB9FBA96E106FCDFB55F467A9A.aspx Linux 3c59x driver, driver operation explained at the top of the file]
* [http://www.koders.com/c/fid98F17E9728B0400DDA72C7AEC829DBBA27A753F9.aspx FreeBSD 3x90x driver]
* [http://www.3com.com/products/en_US/printsafe.jsp?sku=3CN3AC1556B 3com product details]
* [https://people.freebsd.org/~wpaul/3Com/3c90xb.pdf 3c90x and 3x90xB NICs technical reference]
* https://people.freebsd.org/~wpaul/3Com/README

===See Also===
* [[Networking]]

[[Category:Network Hardware]]
````

<a id="w001431"></a>

## W001431 — OSDev Wiki talk:Pages To Convert

Bron-ID: W001431
Namespace: OSDev Wiki talk (5)
Revisie: 7266 | Datum: 2009-02-18T10:32:24Z
Bron: https://wiki.osdev.org/OSDev_Wiki_talk:Pages_To_Convert
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDev_Wiki_talk%3APages_To_Convert&oldid=7266

````text
== Rendundant pages ==
I'm thinking that [[mtfaq:HardwareComm|HardwareComm]] is somewhat reduntant here...
[[User:TheQuux|TheQuux]] 04:10, 29 November 2006 (CST)

Agreed. Posted a comment on [[Category_Talk:Hardware]] about it - [[User:Combuster|Combuster]] 07:56, 20 January 2007 (CST)

== Missing Pages ==
Did somebody create the bochs page without marking it here, or is it still WIP? - [[User:Combuster|Combuster]] 16:02, 11 December 2006 (CST)

Sorry Combuster. I was moving the emulators over and then ran out of time. Fixed. Also there are some other pages that have been moved and weren't marked. I'll try to have a look through them.

Somebody partially ported the algorithms page, decided to post it here as well for completeness sake - [[User:Combuster|Combuster]] 16:48, 17 December 2006 (CST)

==Tutorials==
There are some pages which seem to be in a "tutorial"-like format (Baby steps, bare bones, etc) which don't cleanly fit into any existing sections. What should we do about them? -[[User:Jhawthorn|Jhawthorn]] 01:39, 31 December 2006 (CST)


The Tutorials are helpful for new users, This is supposed to be a Wiki full of useful information about OSDev (<i>From a developers experience..</i>) right?. I don't think just posting the information exactly out of the specifications is the right idea. --[[User:Brynet-Inc|Brynet-Inc]] 01:01, 2 January 2007 (CST)


The dilemma here is that tutorials, FAQs and the like don't fit in the original design of the Wiki. (and to be honest, It made the MT FAQ hard to navigate) To cope we could 'namespace' them, like suffixing 'Tutorial' and putting them together into a separate category (tree). That way people can easily see when it involves an informational or an instructional document.
Possible Wikilinks: [[C Kernel Tutorial]] - [[Bootloader Tutorial]] - [[:Category:Tutorials|Tutorials]]. I have to agree with Brynet-Inc that dropping them would be a Bad Thing. ''(@BI: copying from specs isnt really related to tutorials. If you do find such a page plz put a note on that talkpage to start a discussion about rewriting it)'' - [[User:Combuster|Combuster]] 06:41, 2 January 2007 (CST)

:What would you think about something like <code><nowiki>[[Tutorial:C Kernel|]]</nowiki></code> ([[Tutorial:C Kernel|C Kernel]]) ?-[[User:Jhawthorn|Jhawthorn]] 14:28, 2 January 2007 (CST)
::Same principle. I see you went ahead already :) - [[User:Combuster|Combuster]] 01:06, 3 January 2007 (CST)

== Recently Converted Table ==
What about splitting the table in '''for review''' and '''work in progress''' sections, and remove the pages that are on it for a few months now? - [[User:Combuster|Combuster]] 07:56, 20 January 2007 (CST)

== Deletion? ==

I'm pretty sure this can be deleted now. If nobody objects in the next week or so, I'll go ahead and do it. - [[User:Jackscott|Jackscott]]

: I agree. It's outdated. - [[User:Walling|Walling]] 13:53, 16 February 2009 (UTC)

: Seconded as well. - [[User:Combuster|Combuster]] 22:53, 16 February 2009 (UTC)

: Thirdeded. Ed. --[[User:Troy martin|Troy Martin]] 02:08, 18 February 2009 (UTC)

Deleted - [[User:Combuster|Combuster]] 10:32, 18 February 2009 (UTC)
````

<a id="w001432"></a>

## W001432 — GRUB Legacy

Bron-ID: W001432
Namespace: Main (0)
Revisie: 23075 | Datum: 2018-11-10T20:53:38Z
Bron: https://wiki.osdev.org/GRUB_Legacy
Revisie-URL: https://wiki.osdev.org/index.php?title=GRUB_Legacy&oldid=23075

````text
GRUB stands for GRand Unified Bootloader and is a [[bootloader]] and the reference implementation of the [[Multiboot|Multiboot Specification]]. GRUB can boot any Multiboot-compliant OS (e.g. Hurd, [http://grub.enbug.org/MultibootSystems among others]), however it can also boot some non-Multiboot-compliant OSes, e.g. Windows, via a chain-loading function, and some others via special support, e.g. Linux.  mbchk can be used to determine if a file is Multiboot-compliant.

This article concentrates on GRUB Legacy. Information regarding modern [[GRUB]], a full rewrite, can be found in a separate article.

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
{{FirstPerson}}
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
[[Category:GRUB]]
[[de:Grand Unified Bootloader]]
````

<a id="w001433"></a>

## W001433 — A20 Line

Bron-ID: W001433
Namespace: Main (0)
Revisie: 30909 | Datum: 2026-08-14T21:48:42Z
Bron: https://wiki.osdev.org/A20_Line
Revisie-URL: https://wiki.osdev.org/index.php?title=A20_Line&oldid=30909

````text
The A20 Address Line is the physical representation of the 21st bit (number 20, counting from 0) of any memory access. When the IBM-AT (Intel 286) was introduced, it was able to access up to sixteen megabytes of memory (instead of the 1 MByte of the 8086). But to remain compatible with the 8086, a quirk in the 8086 architecture (memory wraparound) had to be duplicated in the AT. To achieve this, the A20 line on the address bus was disabled by default.

The wraparound was caused by the fact the 8086 could only access 1 megabyte of memory, but because of the segmented memory model it could effectively address up to 1 megabyte and 64 kilobytes (minus 16 bytes).  Because there are 20 address lines on the 8086 (A0 through A19), any address above the 1 megabyte mark wraps around to zero. For some reason a few short-sighted programmers decided to write programs that actually used this wraparound (rather than directly addressing the memory at its normal location at the bottom of memory).  Therefore in order to support these 8086-era programs on the new processors, this wraparound had to be emulated on the IBM AT and its compatibles; this was originally achieved by way of a latch that by default set the A20 line to zero. Later the 486 added the logic into the processor and introduced the A20M pin to control it.

For an operating system developer (or [[Bootloader]] developer) this means the A20 line has to be enabled so that all memory can be accessed. This started off as a simple hack but as simpler methods were added to do it, it became harder to program code that would definitely enable it and even harder to program code that would definitely disable it.

==Keyboard Controller==
The traditional method for A20 line enabling is to directly probe the keyboard controller. The reason for this is that Intel's [["8042" PS/2 Controller|8042 keyboard controller]] had a spare pin which they decided to route the A20 line through. This seems foolish now given their unrelated nature, but at the time computers weren't quite so standardized. Keyboard controllers are usually derivatives of the [http://www.diakom.ru/el/elfirms/datashts/Smsc/42w11.pdf 8042] chip. By programming that chip accurately, you can either enable or disable bit #20 on the address bus.

When your PC boots, the A20 gate is generally disabled, but some BIOSes (and emulators, like QEMU) do enable it for you, as do some high-memory managers (HIMEM.SYS) or bootloaders ([[GRUB]]).

==Testing the A20 line==
Before enabling the A20 with any of the methods described below it is better to test whether the A20 address line was already enabled by the [[BIOS]]. This can be achieved by comparing, at boot time in real mode, the bootsector identifier (0xAA55) located at address 0000:7DFE with the value 1 MiB higher which is at address FFFF:7E0E. When the two values are different it means that the A20 is already enabled otherwise if the values are identical it must be ruled out that this is not by mere chance. Therefore the bootsector identifier needs to be changed, for instance by rotating it left by 8 bits, and again compared to the 16 bits word at FFFF:7E0E. When they are still the same then the A20 address line is disabled otherwise it is enabled.

The following code performs a check (not like described above -- more directly).
<syntaxhighlight lang="nasm">
; The following code is public domain licensed

bits 16

; Function: check_a20
;
; Purpose: to check the status of the a20 line in a completely self-contained state-preserving way.
;          The function can be modified as necessary by removing push's at the beginning and their
;          respective pop's at the end if complete self-containment is not required.
;
; Returns: 0 in ax if the a20 line is disabled (memory wraps around)
;          1 in ax if the a20 line is enabled (memory does not wrap around)

check_a20:
    pushf
    push ds
    push es
    push di
    push si

    cli

    xor ax, ax ; ax = 0
    mov es, ax

    not ax ; ax = 0xFFFF
    mov ds, ax

    mov di, 0x0500
    mov si, 0x0510

    mov al, byte [es:di]
    push ax

    mov al, byte [ds:si]
    push ax

    mov byte [es:di], 0x00
    mov byte [ds:si], 0xFF

    cmp byte [es:di], 0xFF

    pop ax
    mov byte [ds:si], al

    pop ax
    mov byte [es:di], al

    mov ax, 0
    je check_a20__exit

    mov ax, 1

check_a20__exit:
    pop si
    pop di
    pop es
    pop ds
    popf

    ret
</syntaxhighlight>

'''Note:''' The above code may seem confusing to you, if so, below is the simplified code.
<syntaxhighlight lang="nasm">
;	out:
;		ax - state (0 - disabled, 1 - enabled)
get_a20_state:
	pushf
	push si
	push di
	push ds
	push es
	cli

	mov ax, 0x0000					;	0x0000:0x0500(0x00000500) -> ds:si
	mov ds, ax
	mov si, 0x0500

	not ax						    ;	0xffff:0x0510(0x00100500) -> es:di
	mov es, ax
	mov di, 0x0510

	mov al, [ds:si]					;	save old values
	mov byte [.BufferBelowMB], al
	mov al, [es:di]
	mov byte [.BufferOverMB], al

	mov ah, 1
	mov byte [ds:si], 0
	mov byte [es:di], 1
	mov al, [ds:si]
	cmp al, [es:di]					;	check byte at address 0x0500 != byte at address 0x100500
	jne .exit
	dec ah
.exit:
	mov al, [.BufferBelowMB]
	mov [ds:si], al
	mov al, [.BufferOverMB]
	mov [es:di], al
	shr ax, 8					    ;	move result from ah to al register and clear ah
	pop es
	pop ds
	pop di
	pop si
	popf
	ret
	
	.BufferBelowMB:	db 0
	.BufferOverMB	db 0
</syntaxhighlight>

===Testing The A20 Line From Protected Mode===

When in Protected Mode it's easier to test A20 because you can access A20's set memory addresses using any odd megabyte address and compare it to it's even megabyte neighbor.

<syntaxhighlight lang="nasm">
bits 32

; Check A20 line
; Returns to caller if A20 gate is cleared.
; Continues to A20_on if A20 line is set.
; Written by Elad Ashkcenazi 

is_A20_on?:   

pushad
mov edi,0x112345  ;odd megabyte address.
mov esi,0x012345  ;even megabyte address.
mov [esi],esi     ;making sure that both addresses contain diffrent values.
mov [edi],edi     ;(if A20 line is cleared the two pointers would point to the address 0x012345 that would contain 0x112345 (edi)) 
cmpsd             ;compare addresses to see if the're equivalent.
popad
jne A20_on        ;if not equivalent , A20 line is set.
ret               ;if equivalent , the A20 line is cleared.

A20_on:
; *your code from here*
</syntaxhighlight>

==Enabling==
There are several sources that enable A20, commonly each of the inputs are or'ed together to form the A20 enable signal. This means that using one method (if supported by the chipset) is enough to enable A20. If you want to disable A20, you might have to disable all present sources. Always make sure that the A20 has the requested state by testing the line as described above.

===Keyboard Controller===
For the original method to enable the A20 line, some hardware IO using the Keyboard Controller chip (8042 chip) is necessary.

'''NOTE:''' Waiting on the keyboard controller after the last command is important. Not doing so may cause an unwanted keyboard interrupt later on.
<syntaxhighlight lang="c">
void init_A20(void)
{
   uint8_t   a;

   disable_ints();

   kyb_wait_until_done();
   kyb_send_command(0xAD);         // disable keyboard

   kyb_wait_until_done();
   kyb_send_command(0xD0);         // Read from input

   kyb_wait_until_done();
   a=kyb_get_data();

   kyb_wait_until_done();
   kyb_send_command(0xD1);         // Write to output

   kyb_wait_until_done();
   kyb_send_data(a|2);

   kyb_wait_until_done();
   kyb_send_command(0xAE);         // enable keyboard

   kyb_wait_until_done();
   enable_ints();
}
</syntaxhighlight>

or in [[assembly]]

<syntaxhighlight lang="nasm">
;;
;; NASM 32bit assembler
;;

bits 32
section .text

enable_A20:
        cli                     ; disable interrupts

        call    a20wait
        mov     al,0xAD
        out     0x64,al         ; disable keyboard

        call    a20wait
        mov     al,0xD0
        out     0x64,al         ; read controller output port

        call    a20wait2
        in      al,0x60         ; save response byte
        push    eax

        call    a20wait
        mov     al,0xD1
        out     0x64,al         ; write next byte into controller output port

        call    a20wait
        pop     eax
        or      al,2            ; set controller output bit for A20 on
        out     0x60,al         ; activate A20

        call    a20wait
        mov     al,0xAE
        out     0x64,al         ; reactivate keyboard

        call    a20wait
        sti                     ; reactivate interrupts
        ret

a20wait:                        ; wait until input buffer is clear
        in      al,0x64
        test    al,2
        jnz     a20wait
        ret


a20wait2:                       ; wait until response byte has arrived
        in      al,0x64
        test    al,1
        jz      a20wait2
        ret
</syntaxhighlight>

===Fast A20 Gate===
On most newer computers starting with the IBM PS/2, the chipset has a FAST A20 option that can quickly enable the A20 line. To enable A20 this way, there is no need for delay loops or polling, just 3 simple instructions.

<syntaxhighlight lang="nasm">
in al, 0x92
or al, 2
out 0x92, al
</syntaxhighlight>

As mentioned at [http://www.win.tue.nl/~aeb/linux/kbd/A20.html the see also site], it would be best to do the write only when necessary, and to make sure bit 0 is 0, as it is used for fast reset. An example follows:

<syntaxhighlight lang="nasm">
in al, 0x92
test al, 2
jnz after
or al, 2
and al, 0xFE
out 0x92, al
after:
</syntaxhighlight>

However, the Fast A20 method is not supported everywhere and there is no reliable way to tell if it will have some effect or not on a given system. Even worse, on some systems, it may actually do something else like blanking the screen, so it should be used only after the [[BIOS]] has reported that FAST A20 is available. Code for systems lacking FAST A20 support is also needed, so relying only on this method is discouraged. Also, on some chipsets you might have to enable Fast A20 support in the BIOS configuration screen.

=== INT 0x15 ===
Since the 1990s, most [[BIOS]]es provide a function in interrupt 0x15 to quickly enable the A20 gate:

<syntaxhighlight lang="nasm">
bits 16

mov     ax, 0x2403              ; Query A20 gate support
int     0x15
jc      a20_ns                  ; INT 0x15 is not supported
test    ah, ah
jnz     a20_ns                  ; INT 0x15 is not supported

mov     ax, 0x2402              ; Get A20 gate status
int     0x15
jc      a20_failed              ; Couldn't get status
test    ah, ah
jnz     a20_failed              ; Couldn't get status
test    al, al
jnz     a20_activated           ; AL = 1, A20 gate is already activated

mov     ax, 0x2401              ; Activate A20 gate
int     0x15
jc      a20_failed              ; Couldn't activate the gate
test    ah, ah
jnz     a20_failed              ; Couldn't activate the gate

a20_activated:                  ; Go on
</syntaxhighlight>

=== Access of 0xee ===
On some systems reading ioport 0xee enables A20, and writing it disables A20. (Or, sometimes, this action only occurs when ioport 0xee is enabled.) And similar things hold for ioport 0xef and reset (a write causes a reset).
The i386SL/i486SL documents say

The following ports are visible only when enabled,
Any writes to these ports cause the action named.
Name of Register     Address   Default Value  Where placed    Size
FAST CPU RESET         EFh          N/A         82360SL         8
FAST A20 GATE          EEh          N/A         82360SL         8  

Enable A20:
<syntaxhighlight lang="nasm">
in al,0xee
</syntaxhighlight>

Disable A20:
<syntaxhighlight lang="nasm">
out 0xee,al
</syntaxhighlight>

'''NOTE''' that it doesn't matter what AL contains when writing and AL is undefined while reading (to / from port 0xee)

===Recommended Method===

Because there are several different methods that may or may not be supported, and because some of them cause problems on some computers; the recommended method is to try all of them until one works in the "order of least risk". Essentially:
*Test if A20 is already enabled - if it is you don't need to do anything at all
*Try the BIOS function. Ignore the returned status.
*Test if A20 is enabled (to see if the BIOS function actually worked or not)
*Try the keyboard controller method.
*Test if A20 is enabled in a loop with a time-out (as the keyboard controller method may work slowly)
*Try the Fast A20 method last
* Test if A20 is enabled in a loop with a time-out (as the fast A20 method may work slowly)
* If none of the above worked, give up

===Final code example===
<syntaxhighlight lang="nasm">
;	out:
;		ax - state (0 - disabled, 1 - enabled)
get_a20_state:
	pushf
	push si
	push di
	push ds
	push es
	cli

	mov ax, 0x0000					;	0x0000:0x0500(0x00000500) -> ds:si
	mov ds, ax
	mov si, 0x0500

	not ax							;	0xffff:0x0510(0x00100500) -> es:di
	mov es, ax
	mov di, 0x0510

	mov al, [ds:si]					;	save old values
	mov byte [.BufferBelowMB], al
	mov al, [es:di]
	mov byte [.BufferOverMB], al

	mov ah, 1						;	check byte [0x00100500] == byte [0x0500]
	mov byte [ds:si], 0
	mov byte [es:di], 1
	mov al, [ds:si]
	cmp al, [es:di]
	jne .exit
	dec ah
.exit:
	mov al, [.BufferBelowMB]
	mov [ds:si], al
	mov al, [.BufferOverMB]
	mov [es:di], al
	shr ax, 8
	pop es
	pop ds
	pop di
	pop si
	popf
	ret
	
	.BufferBelowMB:	db 0
	.BufferOverMB	db 0

;	out:
;		ax - a20 support bits (bit #0 - supported on keyboard controller; bit #1 - supported with bit #1 of port 0x92)
;		cf - set on error
query_a20_support:
	push bx
	clc

	mov ax, 0x2403
	int 0x15
	jc .error

	test ah, ah
	jnz .error

	mov ax, bx
	pop bx
	ret
.error:
	stc
	pop bx
	ret

enable_a20_keyboard_controller:
	cli

	call .wait_io1
	mov al, 0xad
	out 0x64, al
	
	call .wait_io1
	mov al, 0xd0
	out 0x64, al
	
	call .wait_io2
	in al, 0x60
	push eax
	
	call .wait_io1
	mov al, 0xd1
	out 0x64, al
	
	call .wait_io1
	pop eax
	or al, 2
	out 0x60, al
	
	call .wait_io1
	mov al, 0xae
	out 0x64, al

    call .wait_io1
	sti
	ret
.wait_io1:
	in al, 0x64
	test al, 2
	jnz .wait_io1
	ret
.wait_io2:
	in al, 0x64
	test al, 1
	jz .wait_io2
	ret

;	out:
;		cf - set on error
enable_a20:
	clc									;	clear cf
	pusha
	mov bh, 0							;	clear bh

	call get_a20_state
	jc .fast_gate

	test ax, ax
	jnz .done

	call query_a20_support
	mov bl, al
	test bl, 1							;	enable A20 using keyboard controller
	jnz .keyboard_controller

	test bl, 2							;	enable A20 using fast A20 gate
	jnz .fast_gate
.bios_int:
	mov ax, 0x2401
	int 0x15
	jc .fast_gate
	test ah, ah
	jnz .failed
	call get_a20_state
	test ax, ax
	jnz .done
.fast_gate:
	in al, 0x92
	test al, 2
	jnz .done

	or al, 2
	and al, 0xfe
	out 0x92, al

	call get_a20_state
	test ax, ax
	jnz .done

	test bh, bh							;	test if there was an attempt using the keyboard controller
	jnz .failed
.keyboard_controller:
	call enable_a20_keyboard_controller
	call get_a20_state
	test ax, ax
	jnz .done

	mov bh, 1							;	flag enable attempt with keyboard controller

	test bl, 2
	jnz .fast_gate
	jmp .failed
.failed:
	stc
.done:
	popa
	ret
</syntaxhighlight>

==See Also==
===External Links===
*http://www.win.tue.nl/~aeb/linux/kbd/A20.html

[[Category:X86]]
[[Category:Memory management]]
[[de:A20-Gate]]
````

<a id="w001436"></a>

## W001436 — Talk:Resources

Bron-ID: W001436
Namespace: Talk (1)
Revisie: 30944 | Datum: 2026-09-08T12:52:27Z
Bron: https://wiki.osdev.org/Talk:Resources
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AResources&oldid=30944

````text
:''An archive of this talk page exists at [[Talk:Resources/Archive]]''
----

==Invalid/Missing Links==
Following links and their sub links  appears to be broken.
* [http://perso.wanadoo.fr/pierrelib/index.html Pierre's Library]
* [http://www.nondot.org/sabre/os/articles The Operating System Resource Center]
* [http://www.int6.org int6.org - OS Development Resources Collection]
* [http://devnulled.ath.cx/~oslib The OsLib]
* [http://k101.f2g.net/ Kernel 101]

== Merging link pages ==
As discussed in [[Talk:Quick Links]], there are many pages with links to useful sites, and it would be useful to merge them into one page, here. Many of those are part of Category:Pages For Review.

The pages that are proposed for the merge are:
* [[Resources]]
* [[Quick Links]]
* [[Kernels With Source]]
* [[OSRC]]
* [[Specifications]]
* [[Links]]

A proposed layout of sections (each with the appropriate links):
* Operating System General
* Operating System Development
* Hardware Specifications
** Processors
** Motherboard Devices
** Human Input Devices
** Video
** Audio
** Network
** ...
* Software Specifications
* Other Specifications
* Sample Kernels

All comments are welcome (especially nods of approval :) 

- [[User:Combuster|Combuster]] 12:51, 10 July 2007 (CDT)

:No objections here. --Alboin



I completed merging the pages listed above. All links in specifications appeared to be duplicates of quick links, I located [[Miscellaneous Code]] which might be viable for a merge as well.
Pruned dead links:
* [http://www.cs.ucla.edu/~~kohler/class/aos-f04/ Advanced OS Development], includes 5 labs showing pmode, virtual memory, loading and running programs, etc. (UCLA)
* [http://www.kernelguys.org Kernelguys.org] Kernel related resources collection
* [http://www.clipx.net/ng/bios/ interrupts and POST variables list, an alternative to ralf brown's interrupt list]
* [http://www.hut.fi/~then/mytexts/mouse.html MS mouse protocol] 
* [http://www.hut.fi/~then/mytexts/mouse.html how mice work in general]
* [http://my.execpc.com/~geezer/temp/serial.c serial mouse demo program]
* [http://warmaster.ath.cx/bkerndev/ bkerndev]
* [[Topic:8753|quick overview of what realtime Operating Systems are (really)]] (not really dead, but close enough)
Fixups or confirmations that these are unrecoverable might be welcome.
I invite people for a double-check and final comments before removing the original pages.

- [[User:Combuster|Combuster]] 12:25, 18 September 2007 (CDT)
:They all are dead for me too. As for the page, *Hands over cookie*, nice work. --Alboin

Deleted Quick links, kernels with source, osrc, specification and links. I have [[Talk:Resources/Merge Archive|archived]] their talkpages for future reference. I will update the main page's templates shortly. Thereafter, I think this is case closed. - [[User:Combuster|Combuster]] 16:18, 26 September 2007 (CDT)

== Mouse info ==
Did anyone happen to save copies of the mouse info? 3 out of the 5 links are dead. [[User:Bewing|Bewing]] 00:33, 11 September 2007 (CDT)
:For the record, the links were pruned during the merge:
:* [http://www.hut.fi/~then/mytexts/mouse.html how mice work in general]
:* [http://my.execpc.com/~geezer/temp/serial.c serial mouse demo program]
:As for backups, I don't have them. Anybody who does is still welcome - [[User:Combuster|Combuster]] 16:18, 26 September 2007 (CDT)
````

<a id="w001437"></a>

## W001437 — Talk:Tool Comparison

Bron-ID: W001437
Namespace: Talk (1)
Revisie: 14085 | Datum: 2012-11-17T16:17:24Z
Bron: https://wiki.osdev.org/Talk:Tool_Comparison
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ATool_Comparison&oldid=14085

````text
== Should AT&T syntax/Intel syntax/elf/a.out/flat link to independent articles? ==
I was wondering if each file format should link to their own articles? --''[[User:Brynet-Inc|Brynet-Inc]]''

I placed AT&T syntax and Intel syntax as sections of the article on assembly. Someone who is technologically savvy could link those sections in there. I think the file formats should link to their own articles, under maybe a Executable File Format category, under a more general File Format category? Just a suggestion. --[[User:FMota|FMota]] 16:52, 29 November 2006 (CST)

== 16/32/64 bits? ==
To add or not to add a support 16/32/64-bit column? May be useful as nasm does not support it, as well as things like gas that chokes on 32 bit instructions when compiled for a 64 bit x86... 
Please remove if unwanted --''[[User:Combuster|Combuster]]''

I've filled in the Yasm and Gas fields, Nice idea. --''[[User:Brynet-Inc|Brynet-Inc]]''

==Other tools==
I like where this article's gone. Let's add more tools. Also, I need to fix my <nowiki>{{ref}}</nowiki> and <nowiki>{{note}}</nowiki> templates. --[[User:Jhawthorn|Jhawthorn]] 00:23, 1 December 2006 (CST)

The table comparing compilers has some minor issues.

I'm trying out this interesting tool called HLA, for High Level Assembler.  Check it out at [http://webster.cs.ucr.edu/AsmTools/HLA/index.html]  Should it be under assemblers or compilers.  I'm thinking compilers because it relies on a separate assembler, yet it claims to be an assembler with a really advanced macro feature. --[[User:Mr.Confuzed|Mr.Confuzed]] 20:24, 3 December 2006 (CST)

It remains an assembler at the basic level, and therefore should be qualified as such - [[User:Combuster|Combuster]] 03:05, 4 December 2006 (CST)

==Visual Studio==
Visual Studio does support inline assembly (it isn't gas or nasm however)

It also can output .com files. [[User:Telexicon|Telexicon]] 06:19, 2 December 2006 (CST)

Added the info to the page, please check it -[[User:Combuster|Combuster]] 07:11, 2 December 2006 (CST)

== Compiler separation ==

Wouldn't it be useful to split compilers by language (possible consequence: rename visual studio to Visual C++)

-[[User:Combuster|Combuster]] 06:52, 2 December 2006 (CST)

== Watcom compiler ==

AFAIK, WASM and the OpenWatcom compiler doesn't support x86-64, so there is some incorrect information there.
--[[User:Rdos|Rdos]] 10:17, 17 November 2012 (CST)
````

<a id="w001438"></a>

## W001438 — ACPI

Bron-ID: W001438
Namespace: Main (0)
Revisie: 30223 | Datum: 2026-03-16T14:50:17Z
Bron: https://wiki.osdev.org/ACPI
Revisie-URL: https://wiki.osdev.org/index.php?title=ACPI&oldid=30223

````text
{{ACPI}}

ACPI (Advanced Configuration and Power Interface) is a [[:Category:Power management|power management]] and configuration standard for the PC, developed by Intel, Microsoft and Toshiba. ACPI allows the operating system to discover the hardware configuration of the computer and control the amount of power each device is given (allowing it to put certain devices on standby or power-off for example). It is also used to control and/or check thermal zones (temperature sensors, fan speeds, etc), battery levels, PCI IRQ routing, CPUs, NUMA domains and many other things.

== Implementing ACPI ==

Information about ACPI is stored in the [[BIOS]] memory (for those systems that support ACPI of course).

There are 2 main parts to ACPI. The first part is the tables used by the OS for configuration during boot (these include things like how many CPUs, APIC details, NUMA memory ranges, etc). The second part is the run time ACPI environment, which consists of [[AML]] code (a platform independent OOP language that comes from the BIOS and devices) and the ACPI SMM (System Management Mode) code.

To begin using ACPI, the operating system must look for the RSDP (Root System Description Pointer). This is covered in [[RSDP]] because it is too verbose to put here.

If the RSDP is found and the verification is valid, it contains a pointer to the [[RSDT]] (Root System Description Table) and for newer versions of ACPI (ACPI 2.0 and later) there is an additional XSDT (eXtended System Description Table). Both the RSDT and the XSDT contain pointers to other tables. The only real difference between the RSDT and the XSDT is that the XSDT contains 64 bit pointer instead of 32 bit pointers.

For the run time part of ACPI the main table to detect is the FADT (Fixed ACPI Description Table) as this contains information needed to enable ACPI.

You have two possibilities of using the ACPI. You can write your own ACPI table reader and AML interpreter. Or you can integrate [[ACPICA]] in your OS.

== Switching to ACPI Mode ==

On some PCs, this is already done for you if...

* the SMI command field in the FADT is 0
* the ACPI enable and ACPI disable fields in the FADT are both 0
* bit 0 (value 1) of the PM1a control block I/O port is set

Otherwise, write the value of the ACPI Enable field into the register number pointed to by the smi command field, like so:

<syntaxhighlight lang="c">
outb(fadt->smi_command,fadt->acpi_enable);
</syntaxhighlight>
Linux waits 3 seconds for the hardware to change modes.
Then poll the PM1a control block until bit 0 (value 1) sets. When this bit is set, it means power management events are generating SCIs and not SMIs, which means your OS has to handle the events and the System Management BIOS isn't doing anything for you. The SCI is an IRQ that the FADT tells you about.
<syntaxhighlight lang="c">
while (inw(fadt->pm1a_control_block) & 1 == 0);
</syntaxhighlight>

== See Also ==
=== Articles ===
* [[Shutdown]]
* [[ACPICA]]
* [[UACPI]]
* [[RSDP]]
* [[RSDT]]

=== Forum Links ===
* [[Topic:16990|ACPI Shutdown code with good explanation in C]]

=== External Links ===
* [https://uefi.org/sites/default/files/resources/ACPI_6_3_final_Jan30.pdf ACPI Specification]

[[Category:ACPI]]
[[de:Advanced Configuration and Power Interface]]
````

<a id="w001440"></a>

## W001440 — RSDP

Bron-ID: W001440
Namespace: Main (0)
Revisie: 30901 | Datum: 2026-07-24T07:08:46Z
Bron: https://wiki.osdev.org/RSDP
Revisie-URL: https://wiki.osdev.org/index.php?title=RSDP&oldid=30901

````text
The '''RSDP''' (Root System Description Pointer) is a data structure in the [[ACPI]] specification.{{ACPI}}

== Structure ==
The structure varies depending on the revision. Later revisions are backward compatible with earlier revisions.<syntaxhighlight lang="c">
// structure for revision 0 (version 1.0)
struct RSDP_t {
 char Signature[8];
 uint8_t Checksum;
 char OEMID[6];
 uint8_t Revision;
 uint32_t RsdtAddress;
} __attribute__ ((packed));

// structure for revision 2 (version 2.0+)
struct XSDP_t {
 char Signature[8];
 uint8_t Checksum;
 char OEMID[6];
 uint8_t Revision;
 uint32_t RsdtAddress;      // deprecated since version 2.0

 uint32_t Length;
 uint64_t XsdtAddress;
 uint8_t ExtendedChecksum;
 uint8_t reserved[3];
} __attribute__ ((packed));
</syntaxhighlight>

=== Fields ===

==== Signature ====
An 8 byte magic number used for locating the RSDP, containing <code>RSD PTR</code>.

==== Checksum ====
A byte used to verify the first 20 bytes of the RSDP.

==== OEMID ====
An OEM-supplied string that identified the OEM.

==== Revision ====
The RSDP revision, used for determining which fields are available.

==== RSDT Address ====
A 32-bit <u>physical</u> address pointing to the [[RSDT]].

==== Length ====
The size of the XSDP (RSDP size if fixed, XSDP size might change in later revisions).

==== XSDT Address ====
A 64-bit <u>physical</u> address pointing to the [[XSDT]]. If the revision is at least 2, the XSDT should be used regardless of architecture, as the RSDT was deprecated.

==== Extended Checksum ====
A checksum used for the entire table.

==Locating==
On IA-PC systems, the RSDP is either located within the first 1 KiB of the [[EBDA]] (Extended BIOS Data Area; a 2 byte address to the start of it is located at <code>0x40E</code>), or in the memory region from <code>0x000E0000</code> to <code>0x000FFFFF</code>. To find the table, the operating system has to find the <code>RSD PTR</code>  signature (notice the last space character) in one of the two areas. The signature always starts on a 16 byte boundary.

On [[UEFI]] systems, the RSDP is located in the EFI Configuration Table.

When using a [[Multiboot#Multiboot 2|Multiboot 2]] compliant loader, a copy of the RSDP is contained in the ''ACPI new RSDP'' or ''ACPI old RSDP'' tag respectively.

==Validating==

Once the RSDP has been located, the checksum for the first 20 bytes should be checked. If valid, the revision should be checked, and if it's at least 2, a checksum should be calculated using the Length field.

To verify the checksum, all bytes checked need to add (as a byte, with overflow) up to 0.

== Usage ==
The RSDP points to either the [[RSDT]] or [[XSDT]], which needs to be parsed in order to do anything with the ACPI.

== See Also ==

=== External Links ===

* [https://uefi.org/sites/default/files/resources/ACPI_Spec_6_5_Aug29.pdf ACPI Specification (version 6.5)]

[[Category:ACPI]]
````

<a id="w001441"></a>

## W001441 — APM

Bron-ID: W001441
Namespace: Main (0)
Revisie: 30229 | Datum: 2026-03-16T14:59:20Z
Bron: https://wiki.osdev.org/APM
Revisie-URL: https://wiki.osdev.org/index.php?title=APM&oldid=30229

````text
{{Historical}}

'''APM''' (Advanced Power Management) is a [[:Category:Power management|Power Management]] standard, developed by Intel and Microsoft, which allows the operating system to control the amount of power sent to devices. [[ACPI]] replaces APM as the main power management system for operating systems, mainly because [[SMP]] and APM do not mix well.

== APM Initialization ==
The APM standard defines three interfaces through which the APM BIOS and the Operating System can communicate. They are the Real Mode Interface, the 16bit Protected Mode Interface, and the 32bit Protected Mode Interface. Only one of the interfaces may be in use at any given time. Before any of the interfaces can be used, the operating system must check if APM is actually supported (many very new computers don't support APM anymore) and if it is, connect to one of its interfaces. It is a good idea to send a disconnect command prior to connecting to an interface. 

=== Installation Check ===
This function checks if APM is even supported. It must be performed in '''Real Mode''' (or vm86 mode).
<syntaxhighlight lang="asm">
;perform an installation check
mov ah,53h            ;this is an APM command
mov al,00h            ;installation check command
xor bx,bx             ;device id (0 = APM BIOS)
int 15h               ;call the BIOS function through interrupt 15h
jc APM_error          ;if the carry flag is set there was an error
                      ;the function was successful
                      ;AX = APM version number
                          ;AH = Major revision number (in BCD format)
                          ;AL = Minor revision number (also BCD format)
                      ;BX = ASCII characters "P" (in BH) and "M" (in BL)
                      ;CX = APM flags (see the official documentation for more details)
</syntaxhighlight>

=== Connecting to an interface ===
This function connects to an APM interface based on the value of <code>interface_number</code> which should be one of the following:
<pre>
01h = Real Mode Interface
02h = 16bit Protected Mode Interface
03h = 32bit Protected Mode Interface
</pre>
It must be performed in '''Real Mode''' (or vm86 mode).
<syntaxhighlight lang="asm">
;connect to an APM interface
mov ah,53h               ;this is an APM command
mov al,[interface_number];see above description
xor bx,bx                ;device id (0 = APM BIOS)
int 15h                  ;call the BIOS function through interrupt 15h
jc APM_error             ;if the carry flag is set there was an error
                         ;otherwise, the function was successful
                         ;The return values are different for each interface.
                         ;The Real Mode Interface returns nothing.

                         ;16-Bit Protected Mode Interface
                         ;AX = Code Segment
                         ;BX = Entry Point (Offset)
                         ;CX = Data Segment
                         ;SI = Code Segment Length
                         ;DI = Data Segment Length

                         ;32-Bit Protected Mode Interface
                         ;AX = 32-Bit Code Segment
                         ;EBX = 32-Bit Entry Point (Offset)
                         ;CX = 16-Bit Code Segment
                         ;DX = Data Segment
                         ;ESI 0:15 = 32-Bit Code Segment Length
                         ;ESI 16:31 = 16-Bit Code Segment Length
                         ;DI = Data Segment Length
</syntaxhighlight>

The APM driver or OS is responsible for setting up the proper environment for the 16-bit and 32-bit protected mode interface calls.  This includes setting up the GDT (or LDT) with a 16-bit code segment and a 16-bit data segment for the 16-Bit protected mode interface, or setting up a 32-bit code segment '''and''' a 16-bit code segment, along with a data segment before using the 32-bit protected mode interface.

Note: The 16-bit and 32-bit code segments returned from the function above are the real-mode segments containing the protected mode code.  This means that the value returned must first be converted from the real-mode segment number to a linear address.  This can be done by shifting the value to the left by 8 bits.

Once these GDT (or LDT) entries have been created, they can be used to call the 16-bit or 32-bit protected mode APM functions, using the value returned above in the BX register as the offset for a far-call into the proper code segment.

=== Disconnecting from an Interface ===
This function will disconnect from whatever interface is connected when it is called. If no interface is connected or an error occurs it will set the carry flag upon return. It may be performed in '''any''' APM interface mode.
<syntaxhighlight lang="asm">
;disconnect from any APM interface
mov ah,53h               ;this is an APM command
mov al,04h               ;interface disconnect command
xor bx,bx                ;device id (0 = APM BIOS)
int 15h                  ;call the BIOS function through interrupt 15h
jc .disconnect_error            ;if the carry flag is set see what the fuss is about. 
jmp .no_error

.disconnect_error:       ;the error code is in ah.
cmp ah,03h               ;if the error code is anything but 03h there was an error.
jne APM_error            ;the error code 03h means that no interface was connected in the first place.
                         
.no_error:
                         ;the function was successful
                         ;Nothing is returned.
</syntaxhighlight>

=== Setting APM Driver Version ===
After an APM interface is connected, APM '''always''' runs as if it was version 1.0, to support legacy code. Unfortunately in that mode it isn't possible to do some things as setting the power state for all devices (as needed e.g. to shut the system down). So, if the APM installation check reports a version higher than 1.0, it might be useful to tell the APM that you support 1.1 or 1.2 version using INT 0x15 / AX=0x530E

<syntaxhighlight lang="asm">
mov ah,53h               ;this is an APM command
mov al,0eh               ;set driver supported version command
mov bx,0000h             ;device ID of system BIOS
mov ch,01h               ;APM driver major version
mov cl,01h               ;APM driver minor version (can be 01h or 02h if the latter one is supported)
int 15h
jc .version_error
;at this point AX holds the APM version that is connected, AH=major version AL=minor version
;so an additional check might be implemented
jmp .no_error
.version_error:
;ah can hold: 03h if the interface wasn't connected, 09h if the device ID wasn't recognised (BX nonzero), 0Bh if APM v1.1 still isn't engaged
.no_error:
;continue
</syntaxhighlight>

== Controlling Devices ==
After the operating system has connected to an APM interface it can begin to control the power state of various devices in the computer. Before a given device can be controlled, however, power management for that device must be enabled. Only then can the actual power state of the device be set.

=== Enabling Power Management ===
This function will enable power management for all devices. This is, by far, faster than enabling power management for each device separately.
<syntaxhighlight lang="asm">
;Enable power management for all devices
mov ah,53h              ;this is an APM command
mov al,08h              ;Change the state of power management...
mov bx,0001h            ;...on all devices to...
mov cx,0001h            ;...power management on.
int 15h                 ;call the BIOS function through interrupt 15h
jc APM_error            ;if the carry flag is set there was an error	
</syntaxhighlight>
To enable power management for a specific device, find the device number for the device in the APM documentation, and put the device number in bx instead of 0001h.

=== Setting Power States ===
This function will set the power state of every device to the value of <code>power_state</code> which may be one of the following.
<pre>
01h = Standby
02h = Suspend
03h = Off

20h...7Fh = OEM-defined power states 
</pre>
To set the power state for a specific device, find the device number for the device in the APM documentation, and put the device number in bx instead of 0001h.
<syntaxhighlight lang="asm">
;Set the power state for all devices
mov ah,53h              ;this is an APM command
mov al,07h              ;Set the power state...
mov bx,0001h            ;...on all devices to...
mov cx,[power_state]    ;see above
int 15h                 ;call the BIOS function through interrupt 15h
jc APM_error            ;if the carry flag is set there was an error	
</syntaxhighlight>

== APM Reliability ==
Because APM is used by calling BIOS functions, it is much simpler to implement in an operating system. These BIOS functions, however, may be buggy or broken on some BIOSes. This is one of the main reasons why [[ACPI]] has succeeded. 

* [http://lxr.linux.no/source/arch/x86/kernel/apm_32.c#L2055 A list of defective BIOSes]

== See Also ==
=== Articles ===
* [[Shutdown]]
* [[ATA/ATAPI Power Management]]
* [[ACPI]], the standard superseding APM

=== External Links ===
* [http://download.microsoft.com/download/1/6/1/161ba512-40e2-4cc9-843a-923143f3456c/APMV12.rtf Official document by Intel and Microsoft describing the APM standard version 1.2]

[[Category:In Progress]]
[[Category:Power management]]
[[de:Advanced Power Management]]
````

<a id="w001443"></a>

## W001443 — VGA Resources

Bron-ID: W001443
Namespace: Main (0)
Revisie: 25369 | Datum: 2020-11-19T20:15:07Z
Bron: https://wiki.osdev.org/VGA_Resources
Revisie-URL: https://wiki.osdev.org/index.php?title=VGA_Resources&oldid=25369

````text
[[Category:VGA]]

== VGA Resources ==
* [[:Category:VGA|This wiki]]
* [http://www.google.com/search?hl=en&ie=UTF-8&q=VGA+registers+programming&btnG=Google+Search Google Search "VGA registers programming"],
* [http://files.osdev.org/mirrors/geezer/osd/graphics/index.htm OSD: Graphics] From Chris Giese, containing many code snippets.
* [http://web.archive.org/web/20140218012818/http://atschool.eduweb.co.uk/camdean/pupils/amac/vga.htm Tutorial on VGA graphics] including basis of font and line rendering, palette setting, unchained mode, etc.
* [https://github.com/FlingOS/FlingOS/tree/develop/Kernel/Libraries/Kernel.VGA FlingOS VGA Driver] has a good (low-performance but easy to read, protected-mode) VGA driver supporting text and graphical modes

=== [http://www.osdever.net/FreeVGA/vga/vga.htm Free VGA] ===

* [http://www.osdever.net/FreeVGA/vga/graphreg.htm Graphics Registers] -- control the way the CPU accesses video RAM.
* [http://www.osdever.net/FreeVGA/vga/seqreg.htm Sequencer Registers] -- control how video data is sent to the DAC.
* [http://www.osdever.net/FreeVGA/vga/attrreg.htm Attribute Controller Registers] -- selects the 16 color and 64 color palettes used for EGA/CGA compatibility.
* [http://www.osdever.net/FreeVGA/vga/crtcreg.htm CRT Controller Registers] -- control how the video is output to the display.
* [http://www.osdever.net/FreeVGA/vga/colorreg.htm Color Registers] -- selects the 256 color palette from the maximum possible colors.
* [http://www.osdever.net/FreeVGA/vga/extreg.htm External Registers] -- miscellaneous registers used to control video operation.

== SVGA resources ==

* The VESA home page: http://www.vesa.org
* Emulating VESA calls from protected mode: [[Virtual 8086 Mode]], or use a library like [https://github.com/wfeldt/libx86emu libx86emu]. Xfree86 used [https://cgit.freedesktop.org/xorg/xserver/tree/hw/xfree86/x86emu this code] to access VESA functions on Linux.

==== Selected coding tricks ====

* [http://files.osdev.org/mirrors/geezer/software/90x60.asm 90x60 text mode] from Chris Giese <geezer@execpc.com>
* [http://files.osdev.org/mirrors/geezer/osd/graphics/modes.c Changing text/graphic mode and fonts] -- this also has code for switching video modes without bios
````

<a id="w001446"></a>

## W001446 — Detecting CPU Speed

Bron-ID: W001446
Namespace: Main (0)
Revisie: 30288 | Datum: 2026-03-16T18:58:39Z
Bron: https://wiki.osdev.org/Detecting_CPU_Speed
Revisie-URL: https://wiki.osdev.org/index.php?title=Detecting_CPU_Speed&oldid=30288

````text
==What is CPU Speed==

"CPU speed" has several different definitions:
# How quickly the processor can execute code (e.g. instructions per second)
# How fast the processor's clock is running (e.g. cycles per second)

How quickly a CPU can execute code is important for determining the CPU's performance. How fast a CPU's clock is running is only useful for specific cases (e.g. calibrating the CPU's TSC to use for measuring time).

There are also several different measurements for these different "CPU speeds":
# Best case
# Nominal case
# Average case
# Current case
# Worst case

For example, look at a modern Intel Core i7 CPU (with turbo-boost, power management and hyper-threading). The best case instructions per second would occur when:
# There is no throttling/power saving at all.
# Only one logical CPU is running (turbo-boost activated and hyper-threading not being used)
# Simple instructions with no dependencies in a loop that fits in the CPU's "loop buffer" are being executed
# There are no branch mispredictions
# There are no accesses to memory (no data being transferred to/from caches or RAM)

The worst case instructions per second would be the exact opposite and may be several orders of magnitude worse (e.g. a best case of 4 billion instructions per second and a worst case of 100 million instructions per second). The nominal instructions per second (commonly referred to as "nominal cycles per second") is an estimation of the normal average instructions per second a developer would expect. All of these things are fixed values - a specific CPU always has the same best case, worst case and nominal case, and these values don't change depending on CPU load, which instructions are/were executed, etc.

The current instructions per second is the instructions per second at a specific instant in time and must be somewhere between the best and worst cases. It can't be measured exactly, but can be estimated by finding the average instructions per second for a very short period of time. The average case is something that has to be measured. Both the current instructions per second and the average instructions per second depend heavily on the code that was running. For example, the average instructions per second for a series of NOP instructions may be much higher that the average instructions per second for a series of DIV instructions.


==General Method==

In order to tell what the CPU speed is, we need two things:
# Being able to tell that a given (precise) amount of time has elapsed.
# Being able to know how many 'clock cycles' a portion of code took.

Once these two sub-problems are solved, one can easily tell the CPU speed
using the following :

<pre>
prepare_a_timer(X milliseconds ahead);
while (timer has not fired) {
   inc iterations_counter;
}
cpuspeed_mhz = (iteration_counter * clock_cycles_per_iteration)/1000;
</pre>

Note that except for very special cases, using a busy-loop (even calibrated) to introduce delays is a bad idea and that it should be kept for very small delays
(nano or micro seconds) that must be complied with when programming hardware only.

Also note that PC emulators (like BOCHS, for instance) are rarely realtime, resulting in the clock appearing to run faster
than expected.

=== Waiting for a given amount of time ===

There are two circuits in a PC to deal with time: the [[PIT]] (Programmable Interval Timer - Intel 8253 and 8254 are common PITs) and the RTC (Real Time Clock). The PIT is probably the better of the two for this task.

The PIT has two operating modes that can be useful for telling the CPU speed:
# the ''periodic interrupt'' mode (0x36), in which a signal is emitted to the interrupt controller at a fixed frequency. This is especially interesting on PIT channel 0 which is bound to IRQ0 on a PC.
# the ''one shot'' mode (0x34), in which the PIT will decrease a counter at its top speed (1.19318 MHz) until the counter reaches zero.

Whether or not an IRQ is fired by channel0 in 0x34 mode should be checked.

Note that theoretically, ''one shot'' mode could be used with a ''polling'' approach, reading the current count on the channel's data port, but I/O bus cycles have unpredictable latency and it is up to the programmer to make sure the timestamp counter is not affected by this approach.

=== Knowing how many cycles your loop takes ===

This step depends on the CPU. On 286, 386 and 486, each instruction takes
a well-known and deterministic amount of clock cycles to execute. This allows
the programmer to tell exactly how many cycles a loop iteration took by looking
up the timing of each instruction and then summing them up.

Since the multi-pipelined architecture of the Pentium; however, such numbers are
no longer communicated (for a major part because the same instruction could have
variable timings depending on its surrounding, which makes the timing almost useless).

It is possible to create code which is exceptionally pipeline hostile such as:

<syntaxhighlight lang="asm">
xor eax,edx
xor edx,eax
xor eax,edx
xor edx,eax
...
</syntaxhighlight>

A simple xor instruction takes one cycle, which guarantees that the processor cannot pipeline this code as the current instructions operands depend on the results from the last calculation. One can check that, for a small count (tested from 16 to 64), RDTSC will show the instruction count is almost exactly (sometimes off by one) the cycles count. Unfortunately, when making the chain longer, code cache misses will occur, ruining the whole process.

E.g. [http://www.sylvain-ulg.be.tf/resources/speed.c looping on a chain of 1550 XORs] may require a hundred of iterations before it stabilizes around 1575 clock cycles on a AMDx86-64 and will take an exceedingly long time to stabilize on a Pentium3.

Despite this inaccuracy, it gives relatively good results across the whole processor generation given a reasonably accurate timer. If very accurate measurements are needed the next method should prove more useful.

A Pentium developer has a much better tool to tell timings: the ''Time Stamp Counter'': an internal counter that can be read using RDTSC special instruction

[https://www.ccsl.carleton.ca/~jamuir/rdtscpm1.pdf rdtscpm1.pdf] explains how that feature can be used for performance monitoring and should provide the necessary information on how to access the TSC on a Pentium.

===RDTSC Instruction Access===

The presence of the Time Stamp Counter (and thus the availability of RDTSC instruction) can be detected through the [CPUID] instruction. Calling CPUID with eax=1, will put feature flags in edx. TSC is bit four of that field.

Note that prior to using the CPUID instruction, make sure the processor supports it by testing the 'ID' bit in eflags. (This is 0x200000 and is modifiable only when CPUID instruction is supported. For systems that doesn't support CPUID, writing a '1' at that place will have no effect.)

In the case of a processor that does not support CPUID, you'll have to use more eflags-based tests to tell if a 486, 386, etc. is running and then pick up one of the 'calibrated loops' for that architecture (8086 through 80486 may have variable instruction timings).


=== Working Example Code===

A Real Mode Intel-copyrighted example is present in the above-mentioned application note.
Another code snippet submitted by user DennisCGC that will give the total measured frequency of a Pentium processor follows.

Some notes:
*  ''irq0_count'' is a variable, which increases each time when the timer interrupt is called.
*  In this code it's assumed that the [PIT] is programmed to 100 Hz (The formula to calculate a desired Hz is provided in the snippet.)
*  The code assumes that the command CPUID is supported.

<syntaxhighlight lang = "asm">
;__get_speed__:
   ;first do a cpuid command, with eax=1
   mov  eax,1
   cpuid
   test edx,byte 0x10      ; test bit #4. Do we have TSC ?
   jz   detect_end         ; no ?, go to detect_end
   ;wait until the timer interrupt has been called.
   mov  ebx, ~[irq0_count]

;__wait_irq0__:

   cmp  ebx, ~[irq0_count]
   jz   wait_irq0
   rdtsc                   ; read time stamp counter
   mov  ~[tscLoDword], eax
   mov  ~[tscHiDword], edx
   add  ebx, 2             ; Set time delay value ticks.
   ; remember: so far ebx = ~[irq0]-1, so the next tick is
   ; two steps ahead of the current ebx ;)

;__wait_for_elapsed_ticks__:

   cmp  ebx, ~[irq0_count] ; Have we hit the delay?
   jnz  wait_for_elapsed_ticks
   rdtsc
   sub eax, ~[tscLoDword]  ; Calculate TSC
   sbb edx, ~[tscHiDword]
   ; f(total_ticks_per_Second) =  (1 / total_ticks_per_Second) * 1,000,000
   ; This adjusts for MHz.
   ; so for this: f(100) = (1/100) * 1,000,000 = 10000
   mov ebx, 10000
   div ebx
   ; ax contains measured speed in MHz
   mov ~[mhz], ax
</syntaxhighlight>

See the Intel manual (see links) for more information.
:- bugs report are welcome. IM to [http://www.mega-tokyo.com/forum/index.php?action=viewprofile;user=DennisCGc DennisCGC]

=== Without Interrupts ===

{{Tone}}

I'd be tempted to say 'yes', though I haven't gave it a test nor heard of it elsewhere so far. Here is the trick:

<syntaxhighlight lang="C">
disable()     // disable interrupts (if still not done)
outb(0x43,0x34);   // set PIT channel 0 to single-shot mode
outb(0x40,0);
outb(0x40,0);      // program the counter will be 0x10000 - n after n ticks
long stsc=CPU::readTimeStamp();
for (int i=0x1000;i>0;i--);
long etsc=CPU::readTimeStamp();
outb(0x43,0x04);   // read PIT counter command ??
byte lo=inb(0x40);
byte hi=inb(0x40);
</syntaxhighlight>

Now, we know that
* ticks=(0x10000 - (hi*256+lo)) periods of 1/1193180 seconds have elapsed at least and no more than ticks+1.
* etsc-stsc clock cycles have elapsed during the same time.

Thus (etsc-stsc)*1193180 / ticks should be your CPU speed in Hz ...

As far as i can say, 0x1000 iterations lead to 10 PIT ticks on a 1GHz CPU and a bit less than 0x8000 ticks on the same CPU running BOCHS. This certainly means that on very high speed systems, the discovered speed may not be accurate at all, or worse, less than 1 tick could occur ...

This technique is currently under evaluation in [the forum|Forum:5849]

:- hope you like my technique /PypeClicker

== Asking the SMBios for CPU speed ==
{{Main|SMBIOS}}

The [[SMBIOS]] (System Management BIOS) Specification addresses how motherboard and system vendors present management information about their products in a standard format by extending the BIOS interface on Intel architecture systems. The information is intended to allow generic instrumentation to deliver this information to management applications that use DMI, CIM or direct access, eliminating the need for error prone operations like probing system hardware for presence detection.

Do note that SMBIOS was never intended for initialization purposes. It was intended to provide information for asset management systems to quickly determine what computers contained what hardware. Unfortunately this means that it might be quite unreliable, especially on cheap/home systems. So ultimately it may not be the best way to determine CPU speed.

===SMBios Processor Information===

A Processor information (type 4) structure describes features of the CPU as detected by the SMBios. The exact structure is depicted in section 3.3.5 (p 39) of the [http://www.dmtf.org/standards/documents/SMBIOS/DSP0134.pdf standard]. Within that information you will find the processor type, family, manufacturer etc. and also:
* the External Clock (bus) frequency, which is a word at offset 0x12,
* the Maximum CPU speed in MHz, which is a word at offset 0x14 (e.g. 0xe9 is a 233MHz processor),
* the Current CPU speed in MHz, (word at offset 0x16).

===Getting the SMBIOS Structure===

SMBios provide a _Get SMBIOS Information_ function that tells you how many structures exists. You can then use _Get SMBIOS Structure_ function to read processor information.

As an alternative, you can locate the _SMBIOS Entry Point_ and then traverse manually the SMBIOS structure table, looking for type 4.

All this is depicted in 'Accessing SMBIOS Information' structure of the standard (p 11).

==Links==

===Related threads in the forum===
*Forum:5849
*Forum:767
*Forum:922
*Forum:8949 featuring info on bogomips, how linux does it and durand's code.
* https://forum.osdev.org/viewtopic.php?f=1&t=32808


===Other resources===
* http://cs.usfca.edu/~cruse/cs630s04/lesson23.ppt‎, a crash course on PIT, and how to use it to compute CPU speed.
* http://www.sandpile.org/post/msgs/20004561.htm
* http://www.midnightbeach.com/jon/pubs/rdtsc.htm

*ftp://download.intel.com/support/processors/procid/
:especially section 12: "Operating Frequency" on page 29 of [ftp://download.intel.com/support/processors/procid/24161815.pdf 24161815.pdf]

<!--Searching for SMBIOS should give you info on that too, it contains entries about the CPU, including current speed.-->
====Regarding SMBIOS====
*http://www.dmtf.org/standards/smbios
*http://www.dmtf.org/standards/documents/SMBIOS/DSP0134.pdf
*http://www.pcpitstop.com/faq/smbios.asp

[[Category:X86 CPU]]
[[Category:Hardware Detection]]
````

<a id="w001447"></a>

## W001447 — X86-64

Bron-ID: W001447
Namespace: Main (0)
Revisie: 30486 | Datum: 2026-04-15T15:10:11Z
Bron: https://wiki.osdev.org/X86-64
Revisie-URL: https://wiki.osdev.org/index.php?title=X86-64&oldid=30486

````text
{{DISPLAYTITLE:x86-64}}
This article discusses '''x86-64''' CPUs (AMD64 and Intel's equivalent [[EM64T]] implementation). [[IA-64]] (Itanium) is '''really''' a different beast and not addressed here.

== Features ==

===Long Mode===

Long mode extends general registers to 64 bits (RAX, RBX, RIP, RSP, RFLAGS, etc), and adds eight additional integer registers (R8, R9, ..., R15) plus eight more SSE registers (XMM8 to XMM15) to the CPU. Linear addresses are extended to 64 bit (however, a given CPU may implement less than this) and the physical address space is extended to 52 bits (a given CPU may implement less than this). In essence long mode adds another mode to the CPU.

Long mode does not support hardware task switching or virtual 8086 tasks. In long mode the current CS determines if the code currently running is 64 bit code (true long mode) or 32 bit code (compatibility mode), or even 16-bit protected mode code (still in compatibility mode). Using paging has become mandatory, and segmentation has been stripped down for performance reasons.

The first 64 bit CPUs from both Intel and AMD support 40 bit physical addresses and 48 bit linear addresses.

=== Segmentation in Long Mode ===

Segmentation in long mode functions with a flat model with the exception of two registers: FS and GS.  Setting the base address for these two segment registers is possible via two specific [[MSR|Model Specific Register (MSR)]]s, FS.base (C000_0100h) and GS.base (C000_0101h).  

Additionally there is a long mode specific instruction called [[SWAPGS]], which swaps the contents of GS.base and another MSR called KernelGSBase (C000_0102h).  This instruction is particularly useful for preserving kernel information for a specific logical processor core across context switches.  '''Note: This is an exchange operation'''.

=== Further information ===

''This feature overview is incomplete. Please see the [[http://en.wikipedia.org/wiki/X86-64 Wikipedia article on x86-64]] for more information.

==Setting up==

===How do I detect if the CPU is 64 bits ?===

After calling CPUID with EAX=0x80000001, all AMD64 compliant processors have the longmode-capable-bit turned on in the extended feature flags (bit 29) in EDX. There are also other bits required by long mode; you can check them out in the CPUID docs in the [http://support.amd.com/us/Processor_TechDocs/24594.pdf AMD general purpose instruction reference] (Link dead, the original author probably meant "AMD64 Architecture Programmer’s Manual Volume 3: General Purpose and System Instructions", found here: http://developer.amd.com/resources/developer-guides-manuals/)

===How do I enable Long Mode ?===

The steps for enabling long mode are:
* Disable paging
* Set the PAE enable bit in CR4
* Load CR3 with the physical address of the PML4 (Level 4 Page Map)
* Enable long mode by setting the LME flag (bit 8) in MSR 0xC0000080 (aka [[CPU_Registers_x86-64#IA32_EFER|EFER]])
* Enable paging

''Reference: [https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3a-part-1-manual.pdf Intel 64 and IA-32 Architectures Software Developer's Manual], Section 9.8.5''

Now the CPU will be in compatibility mode, and instructions are still 32-bit.  To enter long mode, the D/B bit (bit 22, 2nd 32-bit value) of the GDT code segment must be clear (as it would be for a 16-bit code segment), and the L bit (bit 21, 2nd 32-bit value) of the GDT code segment must be set.  Once that is done, the CPU is in 64-bit long mode.

=== Are there restrictions on 32-bit code running in Legacy Mode ?===

x86-64 processors can operate in a legacy mode, they still start in real mode and 16 and 32 bit protected mode is still available (along with the associated Virtual 8086 mode). This means an x86 operating system, even DOS, will still run just fine. The only difference is that physical addresses can be up to 52 bits (or as many bits as implemented by the CPU) when PAE is used.

However, [[Virtual 8086 Mode]] does not exist in long/compatibility mode.

If you are running on a multi-processor system, you could send one processor a STARTUP IPI to a real mode memory address (see Intel MultiProcessor specification for more details) that loads a real mode program. The main problem with this approach is that it relies on multiple processors being present in the system.

===Entering Long Mode directly===

Protected mode must be entered before activating long mode. A minimal protected-mode environment must be established to allow long-mode initialization to take place. This environment must include the following:
* A protected-mode IDT for vectoring interrupts and exceptions to the appropriate handlers while in protected mode.
* The protected-mode interrupt and exception handlers referenced by the IDT.
* Gate descriptors for each handler must be loaded in the IDT.
:--''AMD64 docs, volume 2, section 14.4 (Enabling Protected Mode), 24593 Rev. 3.10 February 2005''

That being said, [[Topic:11093|we have a thread]] where Brendan shows how to enable 64-bit long mode with no 32-bit IDT and no 32-bit segments - be assured, however, that any paging-related exception that occurs in long mode before you enable 64-bit IDT will cause the processor to reset due to a triple fault.

There is also [[Entering Long Mode Directly|an example]] of this implemented in a [[bootloader]].

=== Notifying the BIOS ===

In order for the firmware built into the system to optimize itself for running in Long Mode, AMD recommends that the OS notify the BIOS about the intended target environment that the OS will be running in: 32-bit mode, 64-bit mode, or a mixture of both modes.  This can be done by calling the BIOS interrupt 15h from Real Mode with AX set to 0xEC00, and BL set to 1 for 32-bit Protected Mode, 2 for 64-bit Long Mode, or 3 if both modes will be used.

== 64 bit Environment Models ==

There are three 64 bit programming models you need to consider: LP64, ILP64, LLP64. Each model has its own pitfalls. The I/L/P stand for Int, Long, Pointer, respectively; the 64 is the number of bits in each.

LP64 means Longs (and Long Longs) and Pointers are 64 bits wide, Ints are 32 bits wide.
LLP64 means Long Longs and Pointers are 64 bits wide, Longs and Ints are 32 bit wide.
ILP64 means Ints, Longs (and Long Longs) and Pointers are 64 bit wide.

Most *nixes use the LP64 model, Windows uses the LLP64 convention. ILP64 is used very rarely.

=== Data Types ===
This table lists the breakdown of sizes in the various programming models.

{| class="wikitable"
|-
!Datatype
!LP64
!ILP64
!LLP64
!ILP32
!LP32
|-
!char 
|      8 
|       8 
|       8 
|       8 
|      8
|-
!short 
|     16 
|      16 
|      16 
|      16 
|     16
|-
!_int   
|     32 
|      -- 
|      32 
|      -- 
|     --
|-
!int     
|     32 
|      64 
|      32 
|      32 
|     16
|-
!long 
|     64 
|      64 
|      32 
|      32 
|     32
|-
!long long
|     -- 
|      -- 
|      64 
|      -- 
|     --
|-
!pointer
|     64 
|      64 
|      64 
|      32 
|     32
|}

=== Models used by 64bit OSs ===
The following table lists what some current 64bit OS have as a programming model.

{| class="wikitable"
! OS               
! Mode
|-
| Windows XP X64       
| LLP64
|-
| Linux                
|  LP64
|-
| FreeBSD/OpenBSD      
|  LP64
|-
| Solaris              
|  LP64
|-
| DEC OSF/1 Alpha      
|  LP64
|-
| SGI Irix             
|  LP64
|-
| HP UX 11             
|  LP64
|}

=== Text Segment Types ===

Another thing that you must keep in mind, that although the address space (and with it all the pointers) are 64 bit wide, the generated code in the text segment is most likely not. That's because by default gcc compiles to the "mov" instruction which has only 32 bit immediate. This means 64 bit programs are limited to 2G, just as 32 bit mode programs.

If you have ever seen an error message like this:
<pre>
relocation truncated to fit: R_X86_64_32 against symbol
</pre>
then your code hit this barrier. For Assembly, you must use the "movabs" instruction instead of "mov", and for gcc you need to select a different text segment model with the "-mcmodel" argument.

{| class="wikitable"
! Flag               
! Text Segment Addressing
|-
| -mcmodel=small
| The program and its symbols must be linked in the lower 2 GB of the address space (this is the default model)
|-
| -mcmodel=large
| This model makes no assumptions about addresses and sizes of sections.
|-
| -mcmodel=medium
| The program is linked in the lower 2 GB of the address space. Small symbols are also placed there. Symbols with sizes larger than -mlarge-data-threshold are put into large data or bss sections and can be located above 2GB.
|-
| -mcmodel=kernel
| The kernel runs in the negative 2 GB of the address space. This model has to be used for Linux kernel code.
|}
It worth noting that code models are different for architectures, as they are tied with the instruction encoding. For example, AArch64 has a "-mcmodel=tiny" too, which allows 1M addressing, unknown to x86_64. And for AArch64 "-mcmodel=small" has a 4G limitation, not 2G as for the x86_64.

== See Also ==
=== Articles ===
* [[EM64T|Intel EM64T]]
* [[Creating a 64-bit kernel]]
* [[BOOTBOOT|BOOTBOOT bootloader]]
* [[Limine|Limine bootloader]]
* [[X86-64 Instruction Encoding]]
* [[Setting_Up_Long_Mode|Setting up long mode]]

=== Wikipedia ===
* [[Wikipedia:AMD64|AMD64]]
* [[Wikipedia:64-bit|64-bit]]
=== External Links ===
* [https://web.archive.org/web/20090419192844/http://www.amd.com/us-en/assets/content_type/DownloadableAssets/dwamd_AMD64_Porting_FAQ.pdf Porting to AMD64: FAQ]
* [https://web.archive.org/web/20190515211610/http://www.amdboard.com/hammerspecial.html AMD64 Information]
* [https://web.archive.org/web/20160707054920/http://www.x86-64.org/documentation.html x86-64 ABI and assembly guide]
* [https://downloads.openwatcom.org/ftp/devel/docs/elf-64-gen.pdf ELF-64 Object File Format]
* [https://stackoverflow.com/questions/1753602/what-are-the-names-of-the-new-x86-64-processors-registers What are the names of the new X86_64 processors registers? on Stack Overflow]

[[Category:X86 CPU]]
[[Category:X86-64]]
[[Category:Operating Modes]]

[[de:Long Mode]]
````

<a id="w001448"></a>

## W001448 — GCC Cross-Compiler

Bron-ID: W001448
Namespace: Main (0)
Revisie: 30955 | Datum: 2026-09-17T11:09:07Z
Bron: https://wiki.osdev.org/GCC_Cross-Compiler
Revisie-URL: https://wiki.osdev.org/index.php?title=GCC_Cross-Compiler&oldid=30955

````text
{{FirstPerson}}
{{rating|1}}
This tutorial focuses on creating a GCC cross-compiler for your own operating system. This compiler that we build here will have a generic target (i686-elf) that allows you to leave the current operating system behind, meaning that no headers or libraries of the host operating system will be used. Without using a cross-compiler for operating system development, a lot of unexpected things can happen because the compiler assumes that the code is running on the host operating system.

== Introduction ==

Generally speaking, a cross-compiler is a compiler that runs on platform A (the '''host'''), but generates executables for platform B (the '''target'''). These two platforms may (but do not need to) differ in CPU, operating system, and/or [[:Category:Executable Formats|executable format]]. In our case, the host platform is your current operating system and the target platform is the operating system you are about to make. It is important to realize that these two platforms are not the same; the operating system you are developing is always going to be different from the operating system you currently use.

=== Why cross-compilers are necessary ===
{{Main|Why do I need a Cross Compiler?}}

The compiler must know the correct target platform (CPU, operating system). The compiler that comes with the host system does not know by default that it is compiling something else entirely, unless a lot of problematic options are passed to it, which will create a lot of problems in the future. The solution is to build a cross-compiler.

=== Which compiler version to choose ===
{{Main|Building GCC}}

The newest [[GCC]] is recommended as it is the latest and greatest release. In principle, you could have problems if the distance is too great between your system compiler and your new cross-compiler. A new major GCC release series is released yearly (e.g. 15.x) with occasional patch releases (e.g. 15.2), and modern releases shouldn't have any big problems building other modern releases. If you have trouble, you can try the same version as your system GCC, or try to [[Building GCC|build the newest GCC as your system compiler]].

This command prints the current compiler version:

<syntaxhighlight lang="bash">gcc --version</syntaxhighlight>

macOS users should install the Command Line Tools from Apple's developer website and use Clang to cross-compile GCC.

=== Which binutils version to choose ===
{{Main|Cross-Compiler Successful Builds}}
The latest and greatest [[Binutils]] release is recommended. Note, however, that not all combinations of GCC and Binutils work. If you run into trouble, use a Binutils that was released at roughly the same time as your desired compiler version. It doesn't matter what Binutils version you have installed on your current operating system. This command prints the binutils version:
<syntaxhighlight lang="bash">ld --version</syntaxhighlight>

=== Deciding on the target platform ===
{{Main|Target Triplet}}

You should already know this. If you are following the [[Bare Bones]] tutorial, you wish to build a cross-compiler for <tt>i686-elf</tt>.

=== Note on arm-none-eabi-gcc ===
'''The following is unfounded criticism as of gcc 14.2.0:'''

There is the prebuilt package gcc-arm-none-eabi on apt-get for Debian/Ubuntu, but you shouldn't use it because it neither contains a libgcc.a nor freestanding C header files like stdint.h.<br>
Instead you should build it yourself with <tt>arm-none-eabi</tt> being the $TARGET.

== Preparing for the build ==
<!-- Note how this section ([[Preparing GCC Build]]) is shared by [[Building GCC]] and [[GCC Cross-Compiler]]. -->
{{:Preparing GCC Build}}

== The Build ==

We build a toolset running on your host that can turn source code into object files for your target system.

You need to decide where to install your new compiler. It is dangerous and a very bad idea to install it into system directories. You also need to decide whether the new compiler should be installed globally or just for you. If you want to install it just for you (recommended), installing into <tt>$HOME/opt/cross</tt> is normally a good idea. If you want to install it globally, installing it into <tt>/usr/local/cross</tt> is normally a good idea.

Please note that we build everything out of the source directory tree, as is considered good practice. Some packages only support building outside, some only inside and some both (but may not offer extensive checking with make). Building GCC inside the source directory tree fails miserably, at least for older versions.

=== Preparation ===

<syntaxhighlight lang="bash">
export PREFIX="$HOME/opt/cross"
export TARGET=i686-elf
export PATH="$PREFIX/bin:$PATH"
</syntaxhighlight>

We add the installation prefix to the <tt>PATH</tt> of the current shell session. This ensures that the compiler build is able to detect our new binutils once we have built them.

The prefix will configure the build process so that all the files of your cross-compiler environment end up in $HOME/opt/cross. You can change that prefix to whatever you like (e.g., /opt/cross or $HOME/cross would be options). If you have administrator access and wish to make the cross-compiler toolchain available to all users, you can install it into the /usr/local prefix - or perhaps a /usr/local/cross prefix if you are willing to change the system configuration such that this directory is in the search paths for all users. Technically, you could even install directly to /usr, so that your cross-compiler would reside alongside your system compiler, but that is not recommended for several reasons (like risking to overwrite your system compiler if you get TARGET wrong, or getting into conflict with your system's package management).

=== Binutils ===
<!--
This confuses some people a bit too much and they should just get it through package management and follow the above instructions.
# If you wish to build these packages as part of binutils:
mv isl-x.y.z binutils-x.y.z/isl
mv cloog-x.y.z binutils-x.y.z/cloog
# But reconsider: You should just get the development packages from your OS.
-->
<syntaxhighlight lang="bash">
cd $HOME/src

mkdir build-binutils
cd build-binutils
../binutils-x.y.z/configure --target=$TARGET --prefix="$PREFIX" --with-sysroot --disable-nls --disable-werror --enable-default-execstack=no
make
make install
</syntaxhighlight>

This compiles the binutils (assembler, disassembler, and various other useful stuff), runnable on your system but handling code in the format specified by $TARGET.

'''--disable-nls''' tells binutils not to include native language support. This is basically optional, but reduces dependencies and compile time. It will also result in English-language diagnostics, which the people on the [http://forum.osdev.org/ Forum] understand when you ask your questions. ;-)

'''--with-sysroot''' tells binutils to enable sysroot support in the cross-compiler by pointing it to a default empty directory. By default, the linker refuses to use sysroots for no good technical reason, while gcc is able to handle both cases at runtime. This will be useful later on.

'''--enable-default-execstack=no''' tells binutils that the stack is never executable on your platform. Older systems (such as Linux) have an executable stack by default, and every single object file must contain a <tt>.note.GNU-stack</tt> ELF note to disable the executable stack. Any object file without such a note will taint the executable and reduce security, possibly allowing an attacker to put malicious code directly on the stack via a stack overflow vulnerability. This option disables all of this behavior, and the stack is never executable under any circumstances, as new operating systems should be secure from the start and never have an executable stack. If you're building a new system compiler, rather than a cross-compiler, you may wish to omit this option.

=== GDB ===

It may be worth noting that if you wish to use '''GDB''', and you are running on a different computer architecture than your OS (most common case is developing for ARM on x86_64 or x86_64 on ARM), you need to cross-compile GDB separately. While technically a part of Binutils, it resides in a separate repository.

The protocol for building GDB to target a different architecture is very similar to that of regular Binutils:

<syntaxhighlight lang="bash">
../gdb.x.y.z/configure --target=$TARGET --prefix="$PREFIX" --disable-werror
make all-gdb
make install-gdb
</syntaxhighlight>

The <tt>'''--disable-nls'''</tt> and <tt>'''--with-sysroot'''</tt> options don't seem to have any effect.

=== GCC ===
:''See also the [http://gcc.gnu.org/install/configure.html official instructions for configuring gcc].''

Now, you can build [[GCC]].

It will take a while to build your cross-compiler. On a multi-core machine, speed up the build by parallelizing it, e.g. <tt>make -j 8 all-gcc</tt>, if 8 is the number of jobs to run simultaneously.

If you are building a cross compiler for x86-64, you may want to consider building libgcc without the "red zone": [[Libgcc_without_red_zone]]
<!--
This confuses some people a bit too much and they should just get it through package management and follow the above instructions.
# If you wish to build these packages as part of gcc:
mv libiconv-x.y.z gcc-x.y.z/libiconv # Mac OS X users
mv gmp-x.y.z gcc-x.y.z/gmp
mv mpfr-x.y.z gcc-x.y.z/mpfr
mv mpc-x.y.z gcc-x.y.z/mpc
mv isl-x.y.z gcc-x.y.z/isl
mv cloog-x.y.z gcc-x.y.z/cloog
# But reconsider: You should just get the development packages from your OS.
-->

<syntaxhighlight lang="bash">
cd $HOME/src

# The $PREFIX/bin dir _must_ be in the PATH. We did that above.
which -- $TARGET-as || echo $TARGET-as is not in the PATH

mkdir build-gcc
cd build-gcc
../gcc-x.y.z/configure --target=$TARGET --prefix="$PREFIX" --disable-nls --enable-languages=c,c++ --without-headers --enable-initfini-array --disable-hosted-libstdcxx
make all-gcc
make all-target-libgcc
make all-target-libstdc++-v3
make install-gcc
make install-target-libgcc
make install-target-libstdc++-v3
</syntaxhighlight>

We build [[libgcc]], a low-level support library that the compiler expects available at compile time. Linking against [[libgcc]] provides integer, floating point, decimal, stack unwinding (useful for exception handling) and other support functions. Note how we are ''not'' simply running <tt>make && make install</tt> as that would build way too much, not all components of gcc are ready to target your unfinished operating system.

'''--disable-nls''' is the same as for binutils above.

'''--without-headers''' tells [[GCC]] not to rely on any C library (standard or runtime) being present for the target.

'''--enable-languages''' tells [[GCC]] not to compile all the other language frontends it supports, but only C (and optionally C++).

'''--enable-initfini-array''' enables the modern <tt>.init_array</tt> and <tt>.fini_array</tt> ABIs for global constructors.

'''--disable-hosted-libstdcxx''' presumably makes a version of libstdc++ work without a libc (whoever added this option and libstdc++ above: please clarify what it does exactly, and what the limitations are). If you are not using C++, you can remove this option and the libstdc++ make targets.

== Using the new Compiler ==

Now you have a "naked" cross-compiler. It does not have access to a C library or C runtime yet, so you cannot use most of the standard includes or create runnable binaries. But it is quite sufficient to compile the kernel you will be making shortly. Your toolset resides in $HOME/opt/cross (or what you set <tt>$PREFIX</tt> to). For example, you have a GCC executable installed as <tt>$HOME/opt/cross/bin/$TARGET-gcc</tt>, which creates programs for your TARGET. 

You can now run your new compiler by invoking something like:

<syntaxhighlight lang="bash">$HOME/opt/cross/bin/$TARGET-gcc --version</syntaxhighlight>

Note how this compiler is ''not'' able to compile normal C programs. The cross-compiler will spit errors whenever you want to #include any of the standard headers (except for a select few that actually are platform-independent, and generated by the compiler itself). This is quite correct - you don't have a standard library for the target system yet!

The C standard defines two different kinds of executing environments - "freestanding" and "hosted". While the definition might be rather fuzzy for the average application programmer, it is pretty clear-cut when you're doing OS development: A kernel is "freestanding", everything you do in user space is "hosted". A "freestanding" environment needs to provide only a subset of the C library: <tt>float.h</tt>, <tt>iso646.h</tt>, <tt>limits.h</tt>, <tt>stdalign.h</tt>, <tt>stdarg.h</tt>, <tt>stdbool.h</tt>, <tt>stddef.h</tt>, <tt>stdint.h</tt> and <tt>stdnoreturn.h</tt> (as of C11). All of these consist of typedef s and #define s "only", so you can implement them without a single .c file in sight.

Note that to have these compiler-provided includes work properly, you need to build your kernel with the <tt>-ffreestanding</tt> flag. Otherwise, they might attempt including your standard library's copy of the headers, which isn't gonna work if you don't have a standard library.

To use your new compiler simply by invoking <tt>$TARGET-gcc</tt>, add <tt>$HOME/opt/cross/bin</tt> to your <tt>$PATH</tt> by typing:

<syntaxhighlight lang="bash">export PATH="$HOME/opt/cross/bin:$PATH"</syntaxhighlight>

This command will add your new compiler to your PATH for this shell session. If you wish to use it permanently, add the PATH command to your <tt>~/.profile</tt> configuration shell script or similar. Consult your shell documentation for more information.

You can now move on to complete the [[Bare Bones]] tutorial variant that lead you here and complete it using your new cross-compiler. If you built a new GCC version as your system compiler and used it to build the cross-compiler, you can now safely uninstall it unless you wish to continue using it.

== Troubleshooting ==

In general, '''verify''' that you read the instructions carefully and typed the commands precisely. Don't skip instructions. You will have to set your PATH variable again if you use a new shell instance, if you don't make it permanent by adding it to your shell profile. If a compilation seems to have gotten really messed up, type <tt>make distclean</tt>, and then start the make process over again. Ensure your un-archiver doesn't change newline characters.

==== ld: cannot find -lgcc ====

You specified that you want to link the GCC low-level runtime library into your executable through the <tt>-lgcc</tt>' switch, but forgot to build and properly install the library.<br>
If you got no warnings or errors while installing libgcc and you still have problems you can copy the library in your project and link it with <tt>-L. -lgcc</tt><br>
The libgcc is at $PREFIX/lib/gcc/$TARGET/<gcc-version>/libgcc.a .

==== Binutils 2.9 ====

What's alphabetically on the top or bottom is not necessarily the latest version. After 2.9 comes 2.10, 2.11, 2.12 and then there are more releases that are all newer and progressively more likely to build or support your choice of GCC version.

==== Building GCC: the directory that should contain system headers does not exist ====

You might encounter this error when building <tt>mingw32</tt> targets, for example <tt>x86_64-w64-mingw32</tt>. The offending directory that can't be found is <tt>$SYSROOT/mingw/include</tt>. If you look in your sysroot, you will, of course, realise that no such folder exists.

The solution is simply to create the empty folders:
<syntaxhighlight lang="bash">
mkdir -p $SYSROOT/mingw/include
mkdir -p $SYSROOT/mingw/lib
</syntaxhighlight>

This will allow the build to proceed. The reason this happens is that the <tt>mingw32</tt> (and mingw itself) configures <tt>INCLUDE_PATH</tt> and <tt>LIBRARY_PATH</tt> to be, as can be guessed, <tt>/mingw/include</tt> and <tt>/mingw/lib</tt>, instead of the defaults <tt>/usr/include</tt> and <tt>/usr/lib</tt>. Why the build fails even though nothing is required in those folders, and why it doesn't just make them, is beyond me.

=== GCC libsanitizer failing to build ===
Sometimes GCC can't build libsanitizer, if that happens append <tt>--disable-libsanitizer</tt> to the configure command.<br>
This only applies for building a hosted compiler.

=== Building libgcc: pthreads.h not found ===
In some cases, libgcc will attempt to add support for threads which is not available for the target system. Performing the configure step with "--disable-threads" will stop this error from occurring for now.

== More advanced ==
Using this simple cross compiler will be sufficient for quite some time, but at some point you will want the compiler to automatically include your own system headers and libraries.

Building an [[OS Specific Toolchain|OS-specific toolchain]] for your own operating system is the next step in the future, and should be done around the time you add a user-space and a [[libc]].

== See Also ==
=== Articles ===
*[[Cross-Compiler Successful Builds]] - combinations of GCC and Binutils which have been shown to work with this tutorial by OSDev.org members.
*[[Target Triplet]] - on target triplets and their use
*[[OS Specific Toolchain]] - going a step further and adding your own target.
*[[LLVM Cross-Compiler]] - some compilers make things much easier.
*[[Canadian Cross]] - making things yet more complicated.

=== External Links ===
*http://kegel.com/crosstool has a popular example of a script that automatically downloads, patches, and builds binutils, gcc, and glibc for known platforms.
*http://gcc.gnu.org/onlinedocs/gccint/Libgcc.html - Summary of the support functions you get when you link with libgcc.
*http://forums.gentoo.org/viewtopic.php?t=66125 - Compiling Windows applications under Linux
*http://www.libsdl.org/extras/win32/cross/README.txt - dito
*https://github.com/travisg/toolchains - Another script for building simple cross compilers
*https://www.youtube.com/watch?v=aESwsmnA7Ec - A walkthrough of how to build a cross-compiler using Cygwin on Windows.
*https://github.com/Cheapskate01/Cross-Compiler-Build-Script - A dead-simple script that Binutils and Gcc for you.

[[Category:Compilers]]
[[Category:Tutorials]]
[[de:Cross-Compiler]]

=== Prebuilt Toolchains ===
{{warning|Downloading random executable off random google drives and websites is dangerous. It's your responsibility to identify the sources and determine whether they are trustworthy.}}

These were built by people in the OSdev community for their own building needs and shared at will, without guaranteeing any support or that it will even work on your setup. YMMV.

'''Latests versions for Linux (many arch)'''

* [https://www.kernel.org/pub/tools/crosstool/ kernel.org various hosts/targets]

'''For Linux i686 host'''
* [https://drive.google.com/open?id=1zcFAmxi7mtOwhMaKE36IsjLRB0uEv17p aarch64-elf 9.3.0 target]
* [https://drive.google.com/open?id=1uEFrOJPxy13vxWCy-5IbxCJAs5TRdtTE arm-eabi 9.3.0 target]
* [https://drive.google.com/open?id=13Kg6Xd8acUnwUoZQBTOjwAQQeGOeYzVz i386-elf 9.3.0 target]
* [https://drive.google.com/open?id=1F5RsfIEfcpRYAqu5UuGTKgkMa3eBsXnP i486-elf 9.3.0 target]
* [https://drive.google.com/open?id=1PdEFqMEJf_Vuf0drO1m8bjlrsudAmI5k i586-elf 9.3.0 target]
* [https://drive.google.com/open?id=1g9jzEIn8CB6ZiVrc0uxbZprepcX4gYex i686-elf 9.3.0 target]
* [https://drive.google.com/open?id=1xIeNJwD0Do-REFxzCLOkMIVYI3HGJUTX mips-elf 9.3.0 target]
* [https://drive.google.com/open?id=10UuOf9LW4y9WEJcTWrMZW0GAlOWh9KG7 mips64-elf 9.3.0 target]
* [https://drive.google.com/open?id=1oW6UWr-OY22EgtyuXovORUtHeVLkL-Tw m64k-elf 9.3.0 target]
* [https://drive.google.com/open?id=1H3Cbq4D_kjROB-7Otiisa-mac_9FYUM- powerpc-elf 9.3.0 target]
* [https://drive.google.com/open?id=1kFGNHWhcBD9cf8X2y-BW49Su04lMa1ev sh-elf 9.3.0 target]
* [https://drive.google.com/open?id=1XzQDKTK35EX8b380bPRbuhX1GPKrtI77 sparc-elf 9.3.0 target]
* [https://drive.google.com/open?id=1pU8bS0McTyRUTHb1b_c8ZVa2Ka_XeeEp x86_64-elf 9.3.0 target]
* [https://drive.google.com/open?id=1cqk9RzY3QXQuaS-YdIWtCECKL81pcsv- xtensa-elf 9.3.0 target]

'''For Linux x86_64 host'''

* [https://drive.google.com/file/d/0Bw6lG3Ej2746STJaM2dNbC05elE/view?usp=sharing i386-elf & i686-elf 7.1.0 target uploaded by TheAlmostGenius]

* [https://newos.org/toolchains/i386-elf-7.5.0-Linux-x86_64.tar.xz i386-elf 7.5.0 target]
* [https://newos.org/toolchains/x86_64-elf-7.5.0-Linux-x86_64.tar.xz x86_64-elf 7.5.0 target]
* [https://newos.org/toolchains/aarch64-elf-7.5.0-Linux-x86_64.tar.xz aarch64-elf 7.5.0 target]
* [https://newos.org/toolchains/arm-eabi-7.5.0-Linux-x86_64.tar.xz arm-eabi 7.5.0 target]
* [https://newos.org/toolchains/m68k-elf-7.5.0-Linux-x86_64.tar.xz m68k-elf 7.5.0 target]
* [https://newos.org/toolchains/microblaze-elf-7.5.0-Linux-x86_64.tar.xz microblaze-elf 7.5.0 target]
* [https://newos.org/toolchains/mips-elf-7.5.0-Linux-x86_64.tar.xz mips-elf 7.5.0 target]
* [https://newos.org/toolchains/nios2-elf-7.5.0-Linux-x86_64.tar.xz nios2-elf 7.5.0 target]
* [https://newos.org/toolchains/powerpc-elf-7.5.0-Linux-x86_64.tar.xz powerpc-elf 7.5.0 target]
* [https://newos.org/toolchains/riscv32-elf-7.5.0-Linux-x86_64.tar.xz riscv32-elf 7.5.0 target]
* [https://newos.org/toolchains/riscv64-elf-7.5.0-Linux-x86_64.tar.xz riscv64-elf 7.5.0 target]
* [https://newos.org/toolchains/sh-elf-7.5.0-Linux-x86_64.tar.xz sh-elf 7.5.0 target]
* [https://newos.org/toolchains/sparc-elf-7.5.0-Linux-x86_64.tar.xz sparc-elf 7.5.0 target]

The packages from phillid.tk below have been shrunk to about 10 MiB for each pair of packages (GCC &amp; Binutils).
Please note that this has been achieved by enabling only the C front-end for GCC.
If you're going to write your OS in any language but C or Assembly, these packages aren't for you.
These are actually Pacman packages, but untarring them to / and rm-ing /.MTREE and other clutter dotfiles contained in the package will work the same.

'''For Windows host'''
* [https://drive.google.com/file/d/0B85K_c7mx3QjUnZuaFRPWlBIcXM/edit?usp=sharing i686-elf 4.8.2 target]
* [https://mega.co.nz/#F!bBxA3SKJ!TDL4i1NjaZKd4YMo9p2U7g x86_64-elf 5.1.0 target]
* [https://github.com/lordmilko/i686-elf-tools i686-/x86_64-elf 15.2.0 target + GDB (Windows/Linux/WSL)]

'''For Windows Subsystem for Linux (Beta) host'''
* [http://www.bin-os.com/i686-elf-6.1.0.tar.gz i686-elf 6.1.0 target] (extracts to a directory called "cross", don't forget to install 'make' - I would recommend "apt-get install build-essential" to also add additional useful tools)

'''For Windows Subsytem for Linux host'''
* [https://drive.google.com/file/d/1zn37YmyVrtsPOfe88jZRZ_-_3_jzpXBP/view?usp=sharing i686-elf 13.1.0 target] ('''Important: extracts to a directory called "cross-tk"''')

'''For macOS host'''

x86_64-elf [https://formulae.brew.sh/formula/x86_64-elf-binutils binutils] and [https://formulae.brew.sh/formula/x86_64-elf-gcc gcc] (canonical target name x86_64-pc-elf) can be installed from [https://brew.sh homebrew]:
<pre>
$ brew install x86_64-elf-gcc
</pre>
i686-elf toolchain is also [https://formulae.brew.sh/formula/i686-elf-gcc available] in homebrew.

'''ARM prebuilt toolchains for multiple host platforms'''

ARM provides it's own prebuilt toolchain based upon GNU utilities for development targeting ARM systems.

* [https://developer.arm.com/open-source/gnu-toolchain/gnu-rm/downloads GNU ARM Embedded Toolchain]

'''Docker image'''
* [https://hub.docker.com/r/joshwyant/gcc-cross/ i686-elf 8.1.0 target]
````

<a id="w001449"></a>

## W001449 — Talk:X86-64

Bron-ID: W001449
Namespace: Talk (1)
Revisie: 1686 | Datum: 2006-11-30T20:19:58Z
Bron: https://wiki.osdev.org/Talk:X86-64
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AX86-64&oldid=1686

````text
== Going immediately to 64 bit ==
Seems the topic referenced has got a new number after the move from MT, somebody know which thread that was and fix that? 
'' nvm fixed it ''

- [[User:Combuster|Combuster]] 13:24, 30 November 2006 (CST)
````

<a id="w001450"></a>

## W001450 — CPUID

Bron-ID: W001450
Namespace: Main (0)
Revisie: 30267 | Datum: 2026-03-16T17:44:54Z
Bron: https://wiki.osdev.org/CPUID
Revisie-URL: https://wiki.osdev.org/index.php?title=CPUID&oldid=30267

````text
The CPUID instruction can be used to retrieve various information about your CPU, such as its vendor string and model number, the size of internal caches and (more interesting), the list of CPU features supported.

== How to use CPUID ==
=== Checking CPUID availability ===
Prior to using the CPUID instruction, you should also make sure the processor supports it by testing the 'ID' bit (0x200000) in eflags. This bit is modifiable only when the CPUID instruction is supported. For systems that don't support CPUID, changing the 'ID' bit will have no effect.

'''Note:''' Implementing this routine in for example C can lead to issues, because the compiler may change EFLAGS at any time.

This assembly routine checks if CPUID is supported:

<syntaxhighlight lang="asm">
    pushfd                               ;Save EFLAGS
    pushfd                               ;Store EFLAGS
    xor dword [esp],0x00200000           ;Invert the ID bit in stored EFLAGS
    popfd                                ;Load stored EFLAGS (with ID bit inverted)
    pushfd                               ;Store EFLAGS again (ID bit may or may not be inverted)
    pop eax                              ;eax = modified EFLAGS (ID bit may or may not be inverted)
    xor eax,[esp]                        ;eax = whichever bits were changed
    popfd                                ;Restore original EFLAGS
    and eax,0x00200000                   ;eax = zero if ID bit can't be changed, else non-zero
    ret
</syntaxhighlight>

Note 1: There are some old CPUs where CPUID is supported but the ID bit in EFLAGS is not (NexGen). There are also CPUs that support CPUID if and only if it has to be enabled first (Cyrix M1).

Note 2: You can simply attempt to execute the CPUID instruction and see if you get an invalid opcode exception. This avoids problems with CPUs that do support CPUID but don't support the ID bit in EFLAGS; and is likely to be faster for CPUs that do support CPUID (and slower for CPUs that don't).

=== Basic usage ===
The idea of the CPUID instruction is that you can call it with different values in EAX, and it will return different information about the processor. For example, if we want the Vendor ID String (see below), we should code something like that:

<syntaxhighlight lang="asm">
mov eax, 0x0
cpuid
</syntaxhighlight>

There are differences between AMD and Intel. According to the Intel CPUID application note, we should first check the Vendor ID String for "GenuineIntel" before taking out information, such as the Processor Signature, Processor Feature Flags, etc.

=== CPU Vendor ID String ===

When called with EAX = 0, CPUID returns the vendor ID string in EBX, EDX and ECX. Writing these to memory in this order results in a 12-character string. These can be tested against known Vendor ID strings:

<syntaxhighlight lang="c">
// Vendor strings from CPUs.
#define CPUID_VENDOR_AMD           "AuthenticAMD"
#define CPUID_VENDOR_AMD_OLD       "AMDisbetter!" // Early engineering samples of AMD K5 processor
#define CPUID_VENDOR_INTEL         "GenuineIntel"
#define CPUID_VENDOR_VIA           "VIA VIA VIA "
#define CPUID_VENDOR_TRANSMETA     "GenuineTMx86"
#define CPUID_VENDOR_TRANSMETA_OLD "TransmetaCPU"
#define CPUID_VENDOR_CYRIX         "CyrixInstead"
#define CPUID_VENDOR_CENTAUR       "CentaurHauls"
#define CPUID_VENDOR_NEXGEN        "NexGenDriven"
#define CPUID_VENDOR_UMC           "UMC UMC UMC "
#define CPUID_VENDOR_SIS           "SiS SiS SiS "
#define CPUID_VENDOR_NSC           "Geode by NSC"
#define CPUID_VENDOR_RISE          "RiseRiseRise"
#define CPUID_VENDOR_VORTEX        "Vortex86 SoC"
#define CPUID_VENDOR_AO486         "MiSTer AO486"
#define CPUID_VENDOR_AO486_OLD     "GenuineAO486"
#define CPUID_VENDOR_ZHAOXIN       "  Shanghai  "
#define CPUID_VENDOR_HYGON         "HygonGenuine"
#define CPUID_VENDOR_ELBRUS        "E2K MACHINE "
 
// Vendor strings from hypervisors.
#define CPUID_VENDOR_QEMU          "TCGTCGTCGTCG"
#define CPUID_VENDOR_KVM           " KVMKVMKVM  "
#define CPUID_VENDOR_VMWARE        "VMwareVMware"
#define CPUID_VENDOR_VIRTUALBOX    "VBoxVBoxVBox"
#define CPUID_VENDOR_XEN           "XenVMMXenVMM"
#define CPUID_VENDOR_HYPERV        "Microsoft Hv"
#define CPUID_VENDOR_PARALLELS     " prl hyperv "
#define CPUID_VENDOR_PARALLELS_ALT " lrpepyh vr " // Sometimes Parallels incorrectly encodes "prl hyperv" as "lrpepyh vr" due to an endianness mismatch.
#define CPUID_VENDOR_BHYVE         "bhyve bhyve "
#define CPUID_VENDOR_QNX           " QNXQVMBSQG "
</syntaxhighlight>
 
You already know that the Vendor ID String is returned in EBX, ECX, EDX. Let us take an Intel processor. It should return "GenuineIntel". Look at the following text to see how the string is placed in the registers:

<pre>
      MSB         LSB
EBX = 'u' 'n' 'e' 'G'
EDX = 'I' 'e' 'n' 'i'
ECX = 'l' 'e' 't' 'n' 
 
 Where, MSB stands for "Most Significant Byte" and LSB is the "Least Significant Byte".
</pre>

Also, EAX is set to the maximum EAX value supported for CPUID calls, as not all queries are supported on all processors.

=== CPU Features ===

When called with EAX = 1 (CPUID_GETFEATURES), CPUID returns a bit field in EDX containing the following values.
Note that different brands of CPUs may have given different meanings to these. 
Recent processors also use ECX and EBX for features (which form a different set), with which you should be very careful as some old CPUs return bogus information in this register.

<syntaxhighlight lang="c">
enum {
    CPUID_FEAT_EBX_BRAND_INDEX        = 0xFF << 0,
    CPUID_FEAT_EBX_CLFLUSH_LINE_SIZE  = 0xFF << 8,
    CPUID_FEAT_EBX_APIC_ID_SPACE      = 0xFF << 16,
    CPUID_FEAT_EBX_INITIAL_APIC_ID    = 0xFF << 24,
    
    CPUID_FEAT_ECX_SSE3               = 1 << 0,
    CPUID_FEAT_ECX_PCLMUL             = 1 << 1,
    CPUID_FEAT_ECX_DTES64             = 1 << 2,
    CPUID_FEAT_ECX_MONITOR            = 1 << 3,
    CPUID_FEAT_ECX_DS_CPL             = 1 << 4,
    CPUID_FEAT_ECX_VMX                = 1 << 5,
    CPUID_FEAT_ECX_SMX                = 1 << 6,
    CPUID_FEAT_ECX_EST                = 1 << 7,
    CPUID_FEAT_ECX_TM2                = 1 << 8,
    CPUID_FEAT_ECX_SSSE3              = 1 << 9,
    CPUID_FEAT_ECX_CID                = 1 << 10,
    CPUID_FEAT_ECX_SDBG               = 1 << 11,
    CPUID_FEAT_ECX_FMA                = 1 << 12,
    CPUID_FEAT_ECX_CX16               = 1 << 13,
    CPUID_FEAT_ECX_XTPR               = 1 << 14,
    CPUID_FEAT_ECX_PDCM               = 1 << 15,
    CPUID_FEAT_ECX_PCID               = 1 << 17,
    CPUID_FEAT_ECX_DCA                = 1 << 18,
    CPUID_FEAT_ECX_SSE4_1             = 1 << 19,
    CPUID_FEAT_ECX_SSE4_2             = 1 << 20,
    CPUID_FEAT_ECX_X2APIC             = 1 << 21,
    CPUID_FEAT_ECX_MOVBE              = 1 << 22,
    CPUID_FEAT_ECX_POPCNT             = 1 << 23,
    CPUID_FEAT_ECX_TSC                = 1 << 24,
    CPUID_FEAT_ECX_AES                = 1 << 25,
    CPUID_FEAT_ECX_XSAVE              = 1 << 26,
    CPUID_FEAT_ECX_OSXSAVE            = 1 << 27,
    CPUID_FEAT_ECX_AVX                = 1 << 28,
    CPUID_FEAT_ECX_F16C               = 1 << 29,
    CPUID_FEAT_ECX_RDRAND             = 1 << 30,
    CPUID_FEAT_ECX_HYPERVISOR         = 1 << 31,

    CPUID_FEAT_EDX_FPU                = 1 << 0,
    CPUID_FEAT_EDX_VME                = 1 << 1,
    CPUID_FEAT_EDX_DE                 = 1 << 2,
    CPUID_FEAT_EDX_PSE                = 1 << 3,
    CPUID_FEAT_EDX_TSC                = 1 << 4,
    CPUID_FEAT_EDX_MSR                = 1 << 5,
    CPUID_FEAT_EDX_PAE                = 1 << 6,
    CPUID_FEAT_EDX_MCE                = 1 << 7,
    CPUID_FEAT_EDX_CX8                = 1 << 8,
    CPUID_FEAT_EDX_APIC               = 1 << 9,
    CPUID_FEAT_EDX_SEP                = 1 << 11,
    CPUID_FEAT_EDX_MTRR               = 1 << 12,
    CPUID_FEAT_EDX_PGE                = 1 << 13,
    CPUID_FEAT_EDX_MCA                = 1 << 14,
    CPUID_FEAT_EDX_CMOV               = 1 << 15,
    CPUID_FEAT_EDX_PAT                = 1 << 16,
    CPUID_FEAT_EDX_PSE36              = 1 << 17,
    CPUID_FEAT_EDX_PSN                = 1 << 18,
    CPUID_FEAT_EDX_CLFLUSH            = 1 << 19,
    CPUID_FEAT_EDX_DS                 = 1 << 21,
    CPUID_FEAT_EDX_ACPI               = 1 << 22,
    CPUID_FEAT_EDX_MMX                = 1 << 23,
    CPUID_FEAT_EDX_FXSR               = 1 << 24,
    CPUID_FEAT_EDX_SSE                = 1 << 25,
    CPUID_FEAT_EDX_SSE2               = 1 << 26,
    CPUID_FEAT_EDX_SS                 = 1 << 27,
    CPUID_FEAT_EDX_HTT                = 1 << 28,
    CPUID_FEAT_EDX_TM                 = 1 << 29,
    CPUID_FEAT_EDX_IA64               = 1 << 30,
    CPUID_FEAT_EDX_PBE                = 1 << 31
};
</syntaxhighlight>

== Using CPUID from GCC ==

Alternatively, one can use the __get_cpuid function that comes with GCC. To use this function, include <cpuid.h>.

<syntaxhighlight lang="c">
#include <cpuid.h>

/* Example: Get CPU's model number */
static int get_model(void)
{
    int ebx, unused;
    __cpuid(0, unused, ebx, unused, unused);
    return ebx;
}

/* Example: Check for builtin local APIC. */
static int check_apic(void)
{
    unsigned int eax, unused, edx;
    __get_cpuid(1, &eax, &unused, &unused, &edx);
    return edx & CPUID_FEAT_EDX_APIC;
}

</syntaxhighlight>

{{Disputed}}

CPUID can be invoked with various request codes (in eax) and will return values in general registers (much as a built-in service interrupt). The following code is made Public Domain out of Clicker's x86/cpu.h

<syntaxhighlight lang="c">
/* DEPRECATED: You should use the <cpuid.h> header that comes with GCC instead. */

enum cpuid_requests {
  CPUID_GETVENDORSTRING,
  CPUID_GETFEATURES,
  CPUID_GETTLB,
  CPUID_GETSERIAL,
  
  CPUID_INTELEXTENDED=0x80000000,
  CPUID_INTELFEATURES,
  CPUID_INTELBRANDSTRING,
  CPUID_INTELBRANDSTRINGMORE,
  CPUID_INTELBRANDSTRINGEND,
};
    
/** issue a single request to CPUID. Fits 'intel features', for instance
 *  note that even if only "eax" and "edx" are of interest, other registers
 *  will be modified by the operation, so we need to tell the compiler about it.
 */
static inline void cpuid(int code, uint32_t *a, uint32_t *d) {
  asm volatile("cpuid":"=a"(*a),"=d"(*d):"a"(code):"ecx","ebx");
}
    
/** issue a complete request, storing general registers output as a string
 */
static inline int cpuid_string(int code, uint32_t where[4]) {
  asm volatile("cpuid":"=a"(*where),"=b"(*(where+1)),
               "=c"(*(where+2)),"=d"(*(where+3)):"a"(code));
  return (int)where[0];
}
</syntaxhighlight>

== See Also ==

=== Articles ===
* [[VMX]] Intel's Virtual-Machine Extensions
* [[Detecting CPU Topology (80x86)]] Detecting CPU Topology on 80x86

=== Threads ===
* [[Topic:11998|CPUID For OS Developers]], an implementation written by Brynet-Inc.
* [[Topic:12146|Forum topic about issues with CPUID]]

===External Links===
* [https://sandpile.org/x86/cpuid.htm sandpile.org]: the world's leading source for for technical x86 processor information
* [https://x86-cpuid.org x86-cpuid.org]: for complete CPUID listings ([https://gitlab.com/x86-cpuid.org/x86-cpuid-db GitLab repository], [https://lore.kernel.org/lkml/ZpkckA2SHa1r3Bor@lx-t490 Project announcement].)
* Intel's [http://bochs.sourceforge.net/techspec/24161821.pdf CPUID Specification]
* AMD's [http://support.amd.com/TechDocs/25481.pdf CPUID Specification]

[[Category:X86 CPU]]
[[Category:Hardware Detection]]
[[de:CPUID]]
````

<a id="w001452"></a>

## W001452 — CPU Bugs

Bron-ID: W001452
Namespace: Main (0)
Revisie: 30264 | Datum: 2026-03-16T17:43:15Z
Bron: https://wiki.osdev.org/CPU_Bugs
Revisie-URL: https://wiki.osdev.org/index.php?title=CPU_Bugs&oldid=30264

````text
Computers are made by humans, and thus inherently prone to errors. This page describes known bugs for various models and brands.

== Affecting almost all modern architectures ==

=== Spectre ===

Spectre is an exploit that affects most modern CPUs made after 1995 that implement out-of-order execution (x86, x86_64, ARM, AMD, and potentially more) and allows the reading of physical memory by userland code. There is no sensible software fix for this issue. For more details, see https://spectreattack.com/spectre.pdf.

== x86 misfeatures ==

=== ESP is not cleared ===

The x86 IRET will not clear upper bits of the stack register (32:16) when returning to 16-bit mode. As the result, the kernel high 16bit of ESP may be leaked to the userspace. Same is true for 64-bit kernel to 16-bit userspace transition.

==== Mitigations ====
One mitigation would be to simply not allow 16-bit code in user mode. Linux, however, does the following to work around the issue: In 32-bit mode, it reserves a segment for fixing up the stack pointer. If it detects a return to userspace with a 16-bit stack, it sets the base address of that segment such that the leaked bits can be set in ESP without moving the actual stack pointer, and then loads that segment as stack segment. Therefore the leaked bits will equal the original bits, so no harm is done.

In 64-bit mode, this is not possible for lack of segmentation support. So instead Linux adopts another method, and will always leak bits chosen randomly at boot. During boot, Linux allocates 48 bytes for each CPU to serve as ESPFIX stack (actually, as long as there is space on the same page left, all CPUs will use the same page, just different offsets on it). These 48 bytes are mapped into kernel space, such that the leaked bits of ESP are entirely random. If a return to a 16-bit stack is detected, the return frame is copied into the ESPFIX stack, before switching the stack over to that address and performing the return. Since that may fail, the ESPFIX stack is mapped read-only (for the copy, a writable mapping of the same page is used). So if the IRET succeeds, nothing is ever written to the ESPFIX stack and everything works out. If something goes wrong, however, since the CPU is already running in Ring 0, it would not switch stacks back. It would try to write an interrupt frame onto the stack, but since the stack is read-only, that will fail. Therefore, a double-fault is caused, and the double-fault handler (running with an IST stack) changes the information such that it looks like a general protection fault occurred on the IRET instruction. And then the GPF handler just does its normal thing.

This way, the leaked bits will not equal the bits userspace had at the start of the interrupt, but they will be random for each boot. Since the ESPFIX stack is so small, a large number of CPUs will have the same leaked ESP bits, so no important data is leaked.

=== NULL selector load may not clear MSR_GS_BASE ===

Intel CPUs do not specify what happens with MSR_GS_BASE if NULL selector is loaded. The Intel CPUs seem to load it with zero, AMD CPUs preserve the previous values (now documented in the AMD64 Architecture Programmer's Manual Volume 2: System Programming). This detail needs to be taken into account for the context switches, if kernel tries to optimize the slow MSR operations.

==== Mitigations ====
Don't load a NULL selector into GS thinking it changes the GS.base register. Always use the MSR operations, unless running on a CPU with the WRFSGSBASE feature, in which case, the GS.base address can be set with the WRGSBASE instruction.

=== FXSAVE/FNSAVE ===

The Intel and AMD differ in what context is saved/restored. AMD CPUs do not save/restore certain parts (FIP/FOP) only when exception is pending (see CVE-2006-1056)

=== SYSRET ===

The Intel CPUs do not handle properly the non-canonical return address.  If a non-canonical address is present in RCX when executing SYSRET, a General Protection Fault will be taken in CPL0 with CPL3 registers. (see CVE-2006-0744)

==== Mitigations ====
The only possible way to execute a SYSCALL instruction such that it jumps to kernel space with a non-canonical address is to put that instruction at address 0x00007ffffffffffe. This is invalid since there is nowhere for the CPU to return to (the return address is invalid). To prevent this from occurring, it is possible to disallow the mapping of an executable page at address 0x00007ffffffff000. Alternatively, the problem can be handled directly: If the return address has any of the top 16 bits set, return to userspace with the IRET instruction instead. Alternatively, construct the necessary stack frame, and jump directly to your GPF handler.

=== SS selector ===

On AMD CPU, SS selector may become unusable when in-kernel interrupt arrives (sets SS to NULL) and thread is switched and returned to userspace via SYSRET. The numerical SS value is correct however the descriptor cache is wrong. This affects only the 32-bit compatibility mode usage of SS.

==== Mitigation ====
Don't switch threads after an in-kernel interrupt (that is: Only switch threads from the outermost stack layer, when returning to userspace or handling a system call). Alternatively, if you did switch threads after an in-kernel interrupt, always return via IRET. You need to jump to the slower IRET return path after a system call in some cases anyway, so you may as well make this one of the cases.

=== PUSH selector ===

On Intel CPUs, when running in 32-bit protected mode, the push will only modify the low 16bit of stack and write there the selector. The high 16 bits remains unmodified. AMD CPUs do not do this. It may have some security impact, that some of stack is not initialized.

Note: This applies to every time a selector is pushed to the stack. So both a PUSH instruction and an implicit push following an interrupt of some sort are affected.

==== Mitigation ====
When reading a selector value off the stack, always mask out the bits you are interested in. Very often, the important information is just the RPL of the selector (which already contains the information, whether the selector is kernel or user space), or the TI bit. The actual table index is rarely needed. Doing this also hardens all parts of the kernel that read selectors against modifications of the GDT later on.

=== Nesting of NMI interrupt ===

If CPU is executing the NMI interrupt handler, CPU guarantees to keep NMI masked until the IRET is executed. However if for some reason NMI triggers some other exception, which executes IRET then the NMI may trigger again, possibly overwriting
its own stack as on AMD64 it runs with IST stack (fun starts if SMI is triggering IRET for some reason).

== Intel ==

=== Transactional Synchronization eXtensions (TSX) Bug ===

In August 2014, Intel announced that a bug exists in the TSX implementation on Haswell, Haswell-E, Haswell-EP and early Broadwell CPUs, which resulted in disabling the TSX feature on affected CPUs via a microcode update. The bug was fixed in F-0 steppings of the vPro-enabled Core M-5Y70 Broadwell CPU in November 2014.

=== Extended Page Table (EPT) Bug ===

A MOV to CR3 when EPT is enabled may lead to an unexpected page fault or an incorrect page translation.

Affected processors:
*Intel Xeon E5-#### v2, where #### is a 4-digit number, optionally followed by a letter.
*Intel Xeon E7-#### v2, where #### is a 4-digit number.
*Intel Xeon E3-12## v2, where ## is a 2-digit number, optionally followed by a letter.

=== F00F Bug ===

Affects: Intel i586 series (Pentium 1, Pentium MMX, Pentium Overdrive, Pentium MMX Overdrive)

This bug is caused by executing LOCK CMPXCHG8B eax (F0 0F C7 C8) By containing two opcode errors, an unallowed lock and a non-memory target, together with trying to cache the results, it confuses the cpu to enter a deadlock state, locking up the entire computer involved.

To fix this bug, the [[IDT]] entry containing the invalid opcode should be marked as uncacheable or writethrough to eliminate one necessary factor, or by marking the same page as not-writable which further confuses the processor, this time into the pagefault handler instead of into a deadlock. If paging is to be left disabled, the only workaround is to disable the cpu's caches, which is far from efficient. Further discussion of various solutions is presented [http://web.archive.org/web/20070927195343/http://www.x86.org/errata/dec97/f00fbug.htm here].

We can check, if the processor is Pentium through the [[CPUID]] instruction. Calling it with EAX=1 will return the CPU signature in EAX. We can extract the Family Number from the CPU signature and compare it with 5, because the Pentium belongs to Family 5.

=== FDIV bug ===

The Pentium FDIV bug is a bug in the Intel P5 Pentium floating point unit (FPU). Because of the bug, the processor can return incorrect decimal results, an issue troublesome for the precise calculations needed in fields like math and science. Discovered by Professor Thomas R. Nicely at Lynchburg College, Intel attributed the error to missing entries in the lookup table used by the floating-point division circuitry.

This problem occurs only on some models of the original Pentium processor. Any Pentium family processor with a clock speed of at least 120 MHz is new enough not to have this bug.

=== Buggy HLT ===

Some of the first 100 MHz Intel DX chips had a buggy HLT state, prompting the developers of Linux to implement a "no-hlt" option for use when running on those chips, but this was fixed in later chips.

=== Core-microarchitecture Bugs ===

[http://web.archive.org/web/20190804131349if_/https://www.geek.com/images/geeknews/2006Jan/core_duo_errata__2006_01_21__full.gif See a list of known bugs as of 2006]

=== 'Meltdown' Page Table Bug ===

Modern (1995 and upwards) Intel x86 chips contain a bug in the out-of-order execution hardware that allows unprivileged userland software to gain access to kernel memory when the kernel is mapped into the userland address space. To avoid vulnerability, it is recommended that the kernel and userland page tables remain separate (i.e: PTI, Page Table Isolation). For more details, visit [https://spectreattack.com/ this page].

== AMD ==

=== DragonFly BSD Heavy Load Crash ===

AMD has confirmed that some of its processors contain a bug that could cause program errors under certain specific conditions. The bug was initially discovered by Matt Dillon, a DragonFly BSD developer.

Consecutive back-to-back pops and (near) return instructions can create a condition where the processor incorrectly updates the stack pointer. The specific manifestations in DragonFly were random segmentation faults under heavy load.

A program exception has been identified in previous generations of the AMD Opteron processor that occurs in certain environments that leverage a very specific GCC compiler build. A workaround has been identified for the small segment of customers this could potentially impact. Also, this marginal erratum impacts the previous four generations of AMD Opteron processors which include the AMD Opteron 2300, 8300 ("Barcelona" and "Shanghai",) 2400, 8400 ("Istanbul",) and 4100, 6100 ("Lisbon" and "Magny-Cours") series processors.

=== Ryzen Bug ===

AMD has confirmed that some of its processors contain a bug that could cause program errors under certain specific conditions when executing code near the canonical address boundary. Insert a guard page (unmapped 4K page, or larger page) before canonical address boundary.

=== CPUID Bugs ===
For older K5 CPUs, the feature flags returned by "CPUID 0x00000001" in EDX are dodgy - bit 9 is used to indicate support for PGE (and not used to indicate support for the local APIC). 

== Cyrix ==

=== Coma Bug ===

Affects: Cyrix 6x86 series

This bug is caused when several implicitly locked instructions are pipelined into an infinite loop. In effect when an instruction completes, the following locked instruction is executed directly afterward, maintaining bus lock and inhibiting interrupts. In an infinite loop, this will lock all interrupts on the processor, rendering it useless.

To fix this bug, one must write to the cyrix registers and set the NO-LOCK bit in CCR1, which disables all but the most essential bus locks. The downside of this is that read-modify-write atomicity is no longer guaranteed on multiprocessor systems. Source code that should prevent this condition: (untested)

<syntaxhighlight lang="asm">
MOV AL, 0xC1   ; 0xC1 refers to CCR1
OUT 0x22, AL   ; Select Register
IN 0x23, AL    ; Load Contents
OR AL, 0x10    ; Set No-Lock bit
MOV AH, AL     ;
MOV AL, 0xC1   ; 0xC1 refers to CCR1
OUT 0x22, AL   ; Select register
MOV AL, AH     ; Load new contents
OUT 0x23, AL   ; Write new CCR1 with No-Lock set
</syntaxhighlight>
[[Category:X86 CPU]]
````

<a id="w001453"></a>

## W001453 — Cygwin

Bron-ID: W001453
Namespace: Main (0)
Revisie: 30176 | Datum: 2026-03-09T16:45:35Z
Bron: https://wiki.osdev.org/Cygwin
Revisie-URL: https://wiki.osdev.org/index.php?title=Cygwin&oldid=30176

````text
"Cygwin" is two things. For one, it is a DLL implementing most of the POSIX API on top of Windows, so to ease porting GNU/Linux tools to Windows.

But in a wider sense it is also a setup.exe for downloading and installing a wide collection of such ported tools on your Windows machine, including a bash shell, GCC toolchain, Apache, PostgreSQL, and many other valuables. As such, it enables OS developers working with Windows to use the same toolset as OS developers under Linux.

== Using Cygwin ==
The Cygwin installer offers to add an icon to your start menu and / or desktop. By clicking on that, you get a console which actually is a bash shell. /cygdrive/<letter>/ under Cygwin is equivalent to <letter>:\ under Windows.

By default the Cygwin installer does not install GCC. Just restart the installer and select GCC (and, if you want to do C++, G++) from the "devel" group. Also download binutils as many applications require this. (And whatever other tools you want; recommended minimum is make, rcs, cvs, and openssh.)

After the installer finished, you just pick up the GCC manual from gcc.gnu.org (or do 'info gcc' in the Cygwin shell) and be a happy camper.

== Cygwin Known Problems ==
Some applications may not compile with Cygwin with such errors as
"default file extension - cannot create executables" or "cannot find crt2.o" or "cannot find -lmingw".

The reason for these messages is that it appears some utilities installed ''after'' the "core" Cygwin files, mess up Cygwins pointers to it's default libraries. 
The solution is to start Cygwin Setup and select the following four packages to re-install:
       libs/mingw-runtime
       devel/gcc-mingw-core
             gcc-mingw-gpp++
             mingw-runtime
Click on 'Keep' next to each package to change the option to 'Re-install'.
Then click next, next etc.. and the libraries will be re-installed. 

Re-installation can be sped up by selecting "Install from Local Directory" rather than "Install from Internet" when Cygwin setup starts. This will use packages already on your computer, installing from the Internet each time will guarantee any updates available are installed.

There is a small chance that Cygwin may 'break' itself after updates, but after following this procedure you should be back in business.

== Cygwin beyond OSDev ==
If you go beyond OS development, there are several caveats regarding Cygwin that make its approach of "POSIX on Windows" less than perfect. Two more refer to [[Licensing Issues]] when linking Windows applications using Cygwin:

* linking to the Cygwin DLL (POSIX API) makes your code fall under the GPL if distributed, and being dependent on the Cygwin DLL being present in a system;
* linking to the standard msvcrt.dll (Win32 API) leaves the licensing decision in your hands and makes your code independent of the Cygwin DLL. Alas, no POSIX here.

If you intend to use the Cygwin gcc for building software, you should check out the permutations of the "win32" and "no-cygwin" options.

== See Also ==

=== Articles ===
* [[Cygwin Issues]]

=== External Links ===
* [https://www.cygwin.com Cygwin Environment Official Site]

[[Category:Compatibility Layers]]
[[Category:Linux]]
[[Category:Windows]]
````

<a id="w001454"></a>

## W001454 — Visual Studio

Bron-ID: W001454
Namespace: Main (0)
Revisie: 25386 | Datum: 2020-11-28T04:21:26Z
Bron: https://wiki.osdev.org/Visual_Studio
Revisie-URL: https://wiki.osdev.org/index.php?title=Visual_Studio&oldid=25386

````text
{{FirstPerson}}

Visual Studio can be used as an integrated development environment for writing assembly, C and C++ with fully featured F5 debugging. Code can be compiled and debugged using either a [[GCC_Cross-Compiler]] toolchain you as you would normally use, or the Microsoft Visual C++ compiler. Due to the fact the standard Visual C++ compiler is only capable of emitting [[OMF]] object files, with the linker only producing [[PE Binaries]], if you pursue this avenue you will have to either use a bootloader which understands these formats or convert them to, for example, [[ELF]] or flat binary.

Due to the fact the only known way of doing native i686-elf operating system development in Visual Studio relies on third-party non-free software, the majority of this article explores how the native capabilities of Visual Studio can be used for doing operating systems development.

If you are interested in attempting to use the Visual C++ toolchain for operating systems development, [http://ksrenevasan.blogspot.com/2005/10/writing-multiboot-pe-kernels-using.html Kaushik Srenevasan's blog] serves as a good starting point to see how you can "[Write] multiboot PE kernels using Visual C++". This is supplemented with [http://ksrenevasan.blogspot.com/2005/10/writing-multiboot-pe-kernels-using_03.html part 2] which explains some of the things not covered in the first part.

MinGW32's objcopy should do the job (parameters go something like this: -x -g -X -S -Obinary kernel.bin). But you don't have to strip any information from PE file, the other option is to set section alignment in memory to 0x200 which is generally equal to the section alignment on disk and do some math:
<source lang="asm">
mov eax, [es:0x3c]          ; PE header pointer in MZ header
mov ecx, [es:eax+0x28]      ; AddressOfEntryPoint in PE header
</source>

where es is loaded with base address of your [[PE Binaries|PE]] file in memory, ecx will be loaded with the entry point relative to base address of PE file.  Of course, if this is protected mode with a flat 4GB address space, use a register instead of using ES.

'''Note:'''  The options and procedures described here for Visual C++ operating system development are for VS.NET 2003.  Similar procedures should work on earlier versions.  Consult the MSDN or post to the forum if you have any questions.

== Native i686-elf Development ==

====Pre-requisites====
In order to utilize Visual Studio as a development environment for native i686-elf operating system development, you must first install the following pre-requisites

* Visual Studio 2015+
* [https://visualgdb.com/ VisualGDB] (see below)
* [https://www.nasm.us/pub/nasm/releasebuilds/2.11.08/win32/nasm-2.11.08-installer.exe NASM 2.11.08]
* A [[GCC_Cross-Compiler|GCC Cross-Compiler toolchain]] that contains GDB. This process has been tested with [https://github.com/lordmilko/i686-elf-tools i686-elf-tools] (GCC 7.1.0 / Binutils 2.28 / GDB 8.0)
* [https://qemu.weilnetz.de/w32/2017/qemu-w32-setup-20170113.exe QEMU 2.8.50.0]
* An assembly syntax highlighter, such as [https://github.com/Trass3r/AsmHighlighter AsmHighlighter] (though you might have to compile it yourself)

It is '''strongly recommended''' you attempt to use the recommend versions of these items before attempting to use newer versions. In particular, issues have been observed when using NASM and QEMU versions newer than the ones listed above (see below).

Install QEMU to '''C:\Program Files (x86)\qemu''', NASM to '''C:\Program Files (x86)\nasm''' and extract i686-elf-tools to '''C:\Program Files (x86)\i686-elf-tools''' (such that the path to your GCC is '''C:\Program Files (x86)\i686-elf-tools\bin\i686-elf-gcc.exe''')

Unfortunately, in order to do operating system development in Visual Studio you will require the third-party add-in '''VisualGDB'''. Unlike all of the other software on this list, VisualGDB is not free. You can download a 30-day trial of VisualGDB from their website to demonstrate the principles outlined in this article and determine how you want to proceed.

The first time you launch Visual Studio after installing VisualGDB you may be prompted to configure the '''VISUALGDB_DIR''' environment variable (which you should do) as well as whether you want to utilize their Clang-based IntelliSense engine (you can disable of the options on this prompt). If you are interested in purchasing VisualGDB, it appears you will need the '''Custom''' VisualGDB edition.

This process has been tested with VisualGDB 5.1 in Visual Studio 2015. A sample project demonstrating a basic boot sector and kernel that you can compile, debug and step through can be found [https://github.com/lordmilko/VSKernelDev here].

====Configuration====

The following outlines how you can configure building and debugging your operating system in Visual Studio from scratch. Rather than configuring these one by one, you can also transplant (and rename based on your project name) the *.vgdbsettings files from the [https://github.com/lordmilko/VSKernelDev sample project] into your solution and you should be good to go.

* In Visual Studio create a new VisualGDB solution using the '''Custom Project Wizard'''. If prompted to configure the custom project's settings, click '''Finish''' to leave these as their defaults (we will configure them one by one below)
* Right click your project in Solution Explorer and select '''VisualGDB Project Properties'''

=====Build Settings=====

* On the ''Build Settings'' tab, next to ''Build command:'' click '''Customize''' and fill in the following details
: '''Command''': $(VISUALGDB_DIR)\make.exe
: '''Arguments''': all
: '''Working directory''': $(SourceDir)

* Next to ''Clean Command:'' click '''Customize''' and fill in the following details
: '''Command''': $(VISUALGDB_DIR)\make.exe
: '''Arguments''': clean
: '''Working directory''': $(SourceDir)

* In the ''Main binary:'' field enter the '''$(BuildDir)\<image>''' where '''<image>''' is the name of the file your '''Makefile''' generates that contains your entire operating system. e.g. '''$(BuildDir)\os-image'''
: The configuration steps outlined in this tutorial assume that all of your output files will be emitted in the most "convenient" place possible (the same place as your source files, the root of your project, etc). If you are interested in having your output files be emitted to a single directory, you should investigate this after you have the basic configuration working.
   
* Untick the ''Try detecting common Makefile types and updating source lists in them'' option. To begin with we would like to control everything ourselves; you can explore re-enabling options like this once we have the basic configuration working

=====Debug Settings=====

* On the ''Debug settings'' tab, deselect '''Break-in to GDB using Ctrl-Break events instead of Ctrl-C (required under Cygwin)'''

* Next to ''Use a custom GDB executable:'' click '''Customize''' and fill in the following details
: '''GDB debugger executable''': C:\Program Files (x86)\i686-elf-tools\bin\i686-elf-gdb.exe
   
* Next to ''GDB launch command:'' click '''Customize''' and fill in the following details
: '''Arguments''': --interpreter mi --readnow
: '''Working directory''': $(ProjectDir)
   
* Tick the '''Use a gdbserver:''' option then click '''Customize''' and fill in the following details
: '''Command''': C:\Program Files (x86)\qemu\qemu-system-i386.exe
: '''Arguments''': -S -gdb tcp::1234,ipv4 -soundhw all -drive file=$(TargetFileName),if=floppy
: '''Working directory''': $(ProjectDir)
   
* In the ''Target selection command:'' field enter '''-target-select remote :1234'''

* Change the '''Debugging start mode:''' to '''Use "continue" command'''

=====IntelliSense Settings=====

* On the '''IntelliSense Settings''' tab, under ''Clang IntelliSense'' set the ''IntelliSense engine:'' to '''Use native Visual Studio IntelliSense engine'''

=====GDB Settings=====

* On the '''GDB settings''' tab, untick '''Support 'step into new instance' through breakpoint in: main'''
: We don't need a breakpoint in any kind of main function, but if you want one at a later point you can re-enable this and change the function name to the real entrypoint of your kernel.

=====GDB Startup Commands=====

* On the '''GDB startup commands''' tab, under ''The following GDB commands will be run AFTER selecting a target:'' enter the following

<source lang="c">
 symbol-file kernel.elf
 add-symbol-file boot_sect.elf 0x7c00
 directory $(RemoteSourceDir)/src 
</source>
: These will load the symbols for your kernel and bootsector respectively, allowing you to debug through the sourcecode in your debugger.
   
: If you have all of your source under a '''src/''' subdirectory, it appears that NASM may cause all *.asm files but the main one to be resolved using relative, instead of absolute paths, thus resulting in GDB being unable to find them when attempting to set breakpoints. Specifying '''directory $(RemoteSourceDir)/src''' adds the '''src/''' folder as an additional search location for GDB to use when resolving breakpoint locations. If you're not using a '''src/''' folder, then you likely don't need this line

If you've successfully completed these steps you should be all ready to start developing your operating system using Visual Studio

====Important Considerations====

* When you modify an *.asm and recompile, it doesn't seem to automatically detect such files are modified; as such you may need to force rebuild instead, or investigate how to get your make system detect changes to assembly files
* When stepping through assembly code, until you switch to 32-bit protected mode you may have issues with your source files not always lining up with where the debugger is currently at. This seems to be an unavoidable consequence of trying to debug 16-bit code; once you start debugging 32-bit assembly or C/C++ however it all seems to be fine
* When you terminate QEMU, GDB will detect the process was terminated but won't actually end the debug session (as such you'll need to hit the Stop button yourself)
* If you try and create a '''Makefile''' in Solution Explorer with no file extension Visual Studio will probably add a '''.cpp''' to the end of it again and move it under the ''Source files'' filter. You will probably want to remove the extension again and maybe move it under the project root, outside the ''Source files'' filter
* Remember that in C++ projects in Visual Studio, folders in Solution Explorer simply represent "filters" rather than actual folders; as such as you start organizing your files remember to place your new files under the actual folder they belong in, along with organizing them under the correct filter
* QEMU versions newer than the stipualted version may experience errors when the recommended version of GDB 8.0 attempts to attach to them. If you wish to use a newer QEMU version, you will need to investigate whether modifying your QEMU command line arguments or [https://github.com/lordmilko/i686-elf-tools compiling a newer GDB version] resolves this issue
* NASM versions newer than the stipulated version may not generate symbol files properly, resulting in breakpoints in '''%include''''d assembly files always hitting the last line of the file. If you wish to use a newer NASM version, you will need to investigate whether modifying your NASM command line arguments or [https://github.com/lordmilko/i686-elf-tools compiling a newer GDB version] resolves this issue

== Visual C++ vs Visual Studio ==
Visual C++ refers only to the Microsoft C++ IDE and compiler, where as Visual Studio refers to the entire Microsoft family of compilers and IDEs as a whole. In later version, such as the Visual Studio .NET series, all languages share the same IDE program, but in Visual C++ .NET, compatibility for all other languages are removed completely, except through configuring the IDE to manually use another compiler/assembler.

While the Microsoft compilers are free, Visual Studio is not. However, a free edition of Visual Studio is available, known as [http://www.visualstudio.com/en-US/products/visual-studio-express-vs Visual Studio 2013 Express] is available for download from Microsoft's website, but must be activated (freely via e-mail) to continue use after 30-days.

The Microsoft compiler that is provided with the express edition of Visual C++ 2013 can build either native 32-bit or 64-bit PE files, or .NET assemblies.

Of course, you could disable the compiler and add a custom build event which runs a shell script and invokes Cygwin. If you are using Visual Studio (not Express), it is possible to use the Visual Studio SDK to create a "makefile project" that allows you to use a custom build script (such as invoking the [[Cygwin]] tool chain) to compile your code, build your image, and launch the emulator (so you can just press F5 for the whole thing to build and the emulator to start). Compiling Bochs with debugging enabled may allow you to use the Visual Studio debugger (including line by line execution) except this hasn't yet been tested.

== Creating the Project: ==

For the kernel and any drivers, create a Win32 Project and select DLL, empty project.  Choose DLL if you want to have a kernel that can export functions using the standard Win32 method.  It is relatively simple to use this to export functions for use by device drivers...

== Custom C++ Runtime ==

Since you can't use standard C/C++ runtime in your kernel, you'll need to write some of it's functionality yourself. The following article will help to [[Visual_C++_Runtime|write your custom Visual C++ runtime]]

== Some basic definitions: ==

<source lang="c">#define EXTERN		extern "C"
#define EXPORT		EXTERN __declspec(dllexport)    // exported from DLL
#define IMPORT		EXTERN __declspec(dllimport)    // imported from DLL
#define NAKED		__declspec(naked)		// no prolog or epilog code added
#define NORETURN	__declspec(noreturn)

// Some examples
EXTERN void SomeFunction(int this, int that);
EXPORT int AnotherFunction(unsigned __int64 bigParam);

// In a .cpp file
EXPORT NAKED int AnotherFunction(unsigned __int64)
{
  __asm
  {
    mov eax, dword ptr [esp+4]
    xor eax, dword ptr [esp+8]
    ret
  }
}
</source>

I use these to create functions that end up with reasonably undecorated names like <code>_SomeFunction@8</code> instead of <code>?@SomeFunction@YAKK000I@@Z</code> (as a __cdecl normal function would be named...)  The macros also allow easy import and export from a DLL.

== Compiler Options ==

Here is the meat of this article.  These are the compiler options (right-click project, select properties) that I use for my OS.

==== General ====
* Output Directory: .
: Add a post-build step to copy only the ''real'' output file to the bin directory. Otherwise VS puts .lib and some linker files there as well.
* Intermediate Directory: .

==== C/C++ :: General ====
* Additional Include Directories: <set as needed>
* Debug Information Format: Disabled
: At the stage my OS is in, I have no use for PDB files.  I am in the process of writing a debugger, though, so in the future this could change.
* Warning Level: Level 4 (/W4)
* Detect 64-bit Portability Issues: No
: This relies on a special __w64 token in various typedefs.  I do not know how to use this with my OS.  Just be careful:  int and long are still 32-bit if compiling for a x64 target (using VS 2005)

==== C/C++ :: Optimization ====
* Optimization: Minimize Size (/O1)
: This is really up to you.  For me, space is more important than speed for now, but this can easily be changed.  If you are implementing source-level debugging, you might want to disable all optimizations.
* Global Optimizations: Yes (/Og)
: Again, enable only if not using a source-level debugger
* Favor Size Or Speed: Favor Small Code (/Os)
: Set as needed, only if /Og enabled
* Optimize for Processor: Pentium Pro, II, III (/G6)
: Set as needed

==== C/C++ :: Preprocessor ====
* Ignore Standard Include Path: Yes (/X)

==== C/C++ :: Code Generation ====
* Enable String Pooling: Yes (/GF)
: Places string literals in a read-only data section.  This doesn't mean much for OS code, but enable this ONLY if you do not modify string literals in-place, as this would change it in all instances.
* Enable Minimal Rebuild: No
: This option attempts to analyse header files and only rebuilds sources if what it uses has changed. Can speed up building, but also frequently makes mistakes leading to runtime errors. Also enabling this seems to add 0xCC pad bytes to the EXE, which causes bloating.
* Enable C++ Exceptions: No
: Unless you have an exceptional (pun intended) configuration, these require runtime support and are generally not needed anyways.
* Basic Runtime Checks: Default
: Enabling any runtime checks requires special support code.
* Struct Member Alignment: 1 Byte (/Zp1)
: This is really up to you, but most structs that I have need to be aligned this way. If you choose default (8 byte), you can use #pragma pack(push, 1) and #pragma pack(pop) to adjust the packing.  Consult MSDN for more info.
* Buffer Security Check: No
: Again, this requires runtime support code

==== C/C++ (misc. options) ====
* Language
* Force Conformance in For Loop Scope: Yes (/Zc:forScope)
: A Good Idea.  Makes the ''i'' in ''for (int i = 0; ...)'' local to the loop.
* Output Files
* Assembler Output: Assembly, Machine Code, and Source (/FAcs)
: Outputs the assembly listing of the code to files in a given directory.  This is nice for assembly-level debugging, as it has the source code lines nearby.
* ASM List Location: <directory>\
: Make sure there is a terminating \, otherwise VS will try to put everything in one file.
* Advanced
* Calling Convention: __stdcall (/Gz)
: Again, up to you, but I find the lack of name decoration handy for debugging.  Functions declared <tt>extern "C" void <nowiki>[[Do Something]]</nowiki>(int p1, int p2)</tt> show up as <tt><nowiki>_[[Do Something]]@8</nowiki></tt> rather than <tt><nowiki>?@[[Do Something]]@YAXZSASD</nowiki></tt> or similar.
* Command Line
* /Oy-
: Disables frame pointer (EBP) omission, included with optimization for size.  This is handy to get [[Stack#Stack trace|stack backtraces]] in case of a crash.

==== Linker ====
* General
* Output File: <set as needed>
* Enable Incremental Linking: No (/INCREMENTAL:NO)
: Reduces the bloat of the generated EXE or DLL.  Linking seems fast enough, anyways.
* Additional Library Directories: <set as needed>
* Input
* Ignore All Default Libraries: Yes (/NODEFAULTLIB)
: Ignores the default libc.lib, libcmt.lib, etc.
* Debugging
* Generate Debug Info: No
: Until I create a better debugger for my OS, I have no use for this.  Set as needed.
* Generate Map File: Yes (/MAP)
: Generates a map file (function name and address) and actually ''sorts'' by ascending address, unlike GCC.
* Map File Name: <set as needed>
* Optimization
* References: Eliminate Unreferenced Data
* Enable COMDAT Folding: Remove Redundant COMDATs
* Advanced
* Entry Point: <set as needed>
: The linker will complain if it is not __stdcall with 12 bytes of arguments (3 32-bit params), but is only a warning.
* Fixed Base Address: Generate a relocation section
: Images will be relocated later.  See below.
* Command Line
* /DRIVER
* /ALIGN: 512
: Together, they set the [[Section Alignment]] and [[File Alignment]] to 512 bytes.  My boot loader is not sophisticated enough to handle these being different.  The downside is that restrictions (read-only, etc) on sections are meaningless, as they require page-granularity for hopefully obvious reasons.

== Bootloader Stuff ==

As part of the build process, I use a tool I wrote to rebase all the PE files.  The
Microsoft rebase utility (and imagehlp api) only works on a 64K granularity, but I
want one with page granularity.

My bootsector is aware of the FAT file system and loads the rest of the bootloader off of
the floppy disk.  This bootloader reads a configuration file, loads the kernel to 0xC0000000,
and then loads each driver on consecutive page-aligned addresses after the kernel.  It passes
an array of these addresses (and filenames) to the kernel, which can then link the images
to itself and call their entry points.

Another option is to use a separate linker such as [[WLink]] with a linker script such as the one found on the [[Watcom]] page.

=== Multiboot ===
To be booted by GRUB, you can make your kernel multiboot. THis involves the embedding of a multiboot header in the first 8K of the image.
This can be done as follows:
<source lang="cpp">
//Entry point goes here

//The good ol' multiboot header
#pragma pack(push,1)
struct MULTIBOOT_HEADER {
    uint32_t magic;
    uint32_t flags;
    uint32_t checksum;
    uint32_t header_addr;
    uint32_t load_addr;
    uint32_t load_end_addr;
    uint32_t bss_end_addr;
    uint32_t entry_addr;   
};
#pragma pack(pop)

#pragma code_seg(".a")
__declspec(align(4)) MULTIBOOT_HEADER header {
    0x1BADB002,
    0x10003,
    -(0x1BADB002+0x10003),
    (uint32_t)&header - BASE_ADDR + LOADBASE,
    LOAD_BASE,
    0,
    0,
    (uint32_t)&entry - BASE_ADDR + LOADBASE
};
</source>

== The Rebase Utility ==

Attached to [[Topic:10632|this thread]] is the source code for a page-granular rebase utility.  It changes the base address of any PE file with a relocation section to the nearest page-aligned address and then removes the relocation section.  It compiles under Visual Studio .NET 2003 with default settings successfully, but should work fine on any Microsoft compiler.

To use it, create a text file that contains the relative paths and names of the PE files you wish to rebase, such as
<pre>
system\kernel.sys
driver\fat.sys
driver\fd.sys
driver\kbdmouse.sys
driver\vga.sys
</pre>
and call the utility from a batch file or the console as
<pre>
mvrebase 0xC0000000 rebasefiles.txt
</pre>
and, in the example given above, it will base kernel.sys at 0xC0000000 and will base subsequent files at the next page-aligned address.

== Express 64 bit compilers ==
You can gain 64 bit compilers by installing the Windows SDK VC++ compilers (please note for VC10 SP1 you need to install the update)

=== Quirks ===
In the 64 bit compilers you '''cannot:'''
*Create naked functions
*Do inline assembly - but it does come with [[MASM]], though you need to separate your C/C++ and assembly into separate source files.
*Do non-fastcall calling

This is why if you intend to do 64 bit development in MSVC++ you should have an external assembly layer (seperate versions for 32 bit and 64 bit), or use somewhat more limited intrinsics, and for asm if you want to avoid name decoration you need to declare them in a header file like this:
<source lang="cpp">
#ifdef __cplusplus //if this is C++
extern "C" { //declare as C functions
#endif
 disable(); //a useful example. disables interrupts (duh!)
 //your functions go here
#ifdef __cplusplus
} //and if it was C++ we need to close the brackets
#endif
</source>
And in your asm layer:
<source lang="asm">
BITS 32 ;32 bit version
@disable@0:
cli
ret 
;fastcall name decoration (@0 to be replaced by size of args (bytes)
;Number of bytes is always prefixed by @
BITS 64 ;64 bit version
disable:
cli
ret  ;No decoration at all
</source>

== Intrinsics ==
For intrinsics, #include <intrin.h>. This IS suitable for a kernel, but don't ignore standard headers. An example follows:
<source lang="cpp">
//main body of intrinsics
#include <intrin.h>
//I/O operations
#include <conio.h>

void someFunc()
{
    __enable();  //STI
    char c = __inbyte(0x60); //IN
    unsigned short w = __inpw(0x1F0);
    unsigned int d = __inp(0xCFC);
    __disable();  //CLI
    __halt();     //HLT
}
</source>

From more intrinsics, see [http://msdn.microsoft.com/en-us/library/26td21ds.aspx Compiler Intrinsics]
Note that the very helpful __setReg, which is reference in some areas of MSDN, is no longer available.

== See Also ==
* [[C++]]
[[Category:IDEs]]
[[Category:Compilers]]
[[Category:Visual C++]]

== Forum Posts ==
* [http://forum.osdev.org/viewtopic.php?f=1&t=21260 Booting non-ELF kernel with GRUB tutorial]
* [http://forum.osdev.org/viewtopic.php?f=2&t=21859 MSVC Grub - A new way]
````

<a id="w001455"></a>

## W001455 — Hexadecimal Notation

Bron-ID: W001455
Namespace: Main (0)
Revisie: 29869 | Datum: 2026-01-12T02:51:39Z
Bron: https://wiki.osdev.org/Hexadecimal_Notation
Revisie-URL: https://wiki.osdev.org/index.php?title=Hexadecimal_Notation&oldid=29869

````text
'''Hexadecimal notation''' is just another base for representing numbers in, like [[Binary Notation|binary]].

In hexadecimal, there are 16 digits: 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, A, B, C, D, E, and F. 0 through 9 preserve their normal meaning, and A through F in hexadecimal is 10 through 15 in decimal notation. As any base, we can calculate the value of a given number, say <code>1CA</code>, by writing the whole number out extensively. <code>1CA</code> in hexadecimal is the same as:
 A*10^0 + C*10^1 + 1*10^2
In decimal, this means
 10*16^0 + 12*16^1 + 1*16^2
Let's find out what this number is:
 10 + 12*16 + 16*16 = 10 + 192 + 256 = 202 + 256 = 458
So <code>1CA</code> in hexadecimal is the same as <code>458</code> in decimal notation.

Hexadecimal notation is useful because it allows you to use powers of two easily without having to write everything out in binary. Converting from hexadecimal to binary is easy: every hexadecimal number is 4 bits, so each hex digit corresponds to a sequence of 4 bits.
{| {{wikitable}}
|-
! Hex
! Binary
! Base 10
|-
! 0
! 0000
! 0
|-
! 1
! 0001
! 1
|-
! 2
! 0010
! 2
|-
! 3
! 0011
! 3
|-
! 4
! 0100
! 4
|-
! 5
! 0101
! 5
|-
! 6
! 0110
! 6
|-
! 7
! 0111
! 7
|-
! 8
! 1000
! 8
|-
! 9
! 1001
! 9
|-
! A
! 1010
! 10
|-
! B
! 1011
! 11
|-
! C
! 1100
! 12
|-
! D
! 1101
! 13
|-
! E
! 1110
! 14
|-
! F
! 1111
! 15
|}

To convert a long number, just replace the hexadecimal digits with their corresponding sequence of bits. For example,
 1CA (hex) = 0001 1100 1010 (bin) = 111001010 (bin)
Two hexadecimal digits correspond exactly to one byte, unlike the strange 255 limit in decimal, the hexadecimal byte limit is FF.

== Notation in programming languages ==
In programming languages and most [[assemblers]], a hexadecimal number is usually prefixed with <code>0x</code> to make the compiler aware that you are using hexadecimal. For example:
  0x123 0x0 0xABCDEF 0xCD
The other alternate notation known as the ''Intel hex notation'', which suffixes hexadecimal numbers with a <code>h</code> letter, matching the <code>b</code> suffix for binary. This notation is supported by most [[x86]] assemblers.

[[Category:Theory]]
````

<a id="w001456"></a>

## W001456 — User:Telexicon

Bron-ID: W001456
Namespace: User (2)
Revisie: 1804 | Datum: 2006-12-04T11:28:38Z
Bron: https://wiki.osdev.org/User:Telexicon
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATelexicon&oldid=1804

````text
===Languages===

* C/C++
* x86 asm (intel)


I dont know what to put here... mabye ill work in the sandbox later...
````

<a id="w001457"></a>

## W001457 — Storage

Bron-ID: W001457
Namespace: Main (0)
Revisie: 30961 | Datum: 2026-09-22T13:12:22Z
Bron: https://wiki.osdev.org/Storage
Revisie-URL: https://wiki.osdev.org/index.php?title=Storage&oldid=30961

````text
==Interfaces==
*[[:Category:ATA|ATA]] and [[ATAPI]] (Also collectively referred to as IDE)
*[[SATA]]
*[[SCSI]]
*[[Fiberchannel]]
*[[Infiniband]]

==Devices==
*[[Floppy Disk Controller|Floppy Drives]]
*[[IDE|Hard Disks]]
*[[CD-ROM|CD-ROM/RW Drives]]
*[[DVD|DVD-ROM/RW Drives]]
*[[USB Flash Memory]]

==Storage Device Drivers==
*[[Floppy Disk Controller|Floppy Disk Driver]]
*[[DMA]]

[[Category:Storage]]
````

<a id="w001458"></a>

## W001458 — Floppy Disk Controller

Bron-ID: W001458
Namespace: Main (0)
Revisie: 28697 | Datum: 2024-03-12T03:43:00Z
Bron: https://wiki.osdev.org/Floppy_Disk_Controller
Revisie-URL: https://wiki.osdev.org/index.php?title=Floppy_Disk_Controller&oldid=28697

````text
== Overview and Documentation ==
The Floppy Disk Controller (FDC) is a (legacy) device that controls '''internal''' 3.5/5.25 inch floppy disk drive devices on desktop x86 systems.
There are a range of chips that have been produced for this function which include: 8272A, 82078, 82077SL & 82077AA. The 82077AA is the most
advanced, and has been produced since 1991. For more recent systems, a model of that chip has been embedded in the motherboard chipset.
(So pay close attention to that datasheet, below.)

=== Accessing Floppies in Real Mode ===
For bootloaders or OSes that run with the CPU remaining in [[Real Mode]], use [[BIOS#BIOS functions|BIOS Function]]
INT13h AH=2 (read) or AH=3 (write) to access floppy drives. You need to know the "drive number" (typically 0 or 1), and put that value in DL.
More detailed info can be found in the [[ATA in x86 RealMode (BIOS)]] article, because accessing a floppy is identical to accessing a hard disk
(using CHS) in Real Mode. The rest of this article deals with creating Protected Mode drivers for the floppy subsystem.

Note: the Extended BIOS Int13h functions do not work with floppies.

Note2: the BIOS IRQ0 handler remembers a timeout for turning the motor off, from the last BIOS floppy access. The last access may have been
attempting to load your bootloader. So, in the distant future, if the BIOS ever receives 36 more IRQ0 ticks (if you ever return to Real Mode)
it may turn off your floppy motors for you, once.

=== Accessing USB Floppy Drives ===
All [[USB]] devices, including USB floppy drives, are accessed indirectly (using SCSI-style commands encoded in USB datapackets) over the USB bus.
USB floppy drives do not use any of the IO ports or FDC commands described in this article.

=== The Floppy Subsystem is Ugly ===
Many of the devices that an OS controls in an x86 system have had functional patches added to them over the years. This makes them all
rather unpleasant when coding drivers. The floppy subsystem is probably the worst. As the functionality evolved, some
of the bit definitions were actually '''reversed''' in meaning, not merely made obsolete. Several commands require duplicating the
same information in two different locations -- and if the duplicated info doesn't match, the command fails. There were never definitive
flags added to identify which "mode" the controller is operating in. Also, the floppy
subsystem is the primary remaining one that accesses the obsolete and terrible ISA DMA system for its data transfers.

=== How Many Controllers? One! Usually. ===
Each ribbon cable for floppy drives can support 2 drives. One floppy controller chip can control 2 floppy cables, for a total
of 4 drives. It is possible for a system to have more than one controller chip, but you will not find many existing systems with
more than one. If a system has more than one floppy controller, the second controller will be found at a base IO port address of 0x370.

Some systems also have a third floppy controller, but these are exceedingly rare. Most of these will have originally been disk
duplication systems. The third floppy controller can be found at a base IO port address of 0x360.

=== How Many Drives? ===
It is probably wisest to always get the drive count and types from [[CMOS]], register 0x10.

=== The Actual Drive ===
In the olden days, there used to be 5.25 inch low-density, high-density, and single-sided drives. There used to be media
for each of these drive types. There also used to be 3.5 inch low density media. None of this really exists anymore. The only
actual hardware you will typically run into now is 3.5 inch, 1.44MB drives.

A 1.44MB floppy disk is usually formatted with 80 cylinders, 2 sides, and 18 sectors per track. The drive records the two sides
with two "heads" that are bolted together. They cannot seek independently. The "track" on one side of a disk is always exactly
opposite the track on the other side of the disk. (There is a misconception about this in many floppy driver code examples.)
You only need to seek one head to find a particular cylinder, so that both heads may read/write that cylinder.

=== CHS ===
Floppy drives use CHS addressing exclusively. There are always 2 heads (sides), but the driver (and controller) must also know how many cylinders and
sectors per track the media expects. Typically, as said above, it is 80 cylinders and 18 sectors per track. In usual CHS fashion,
the cylinders and heads are counted starting with a 0 base, but the sector numbers start counting from 1. That is, legal cylinder
numbers are typically 0 to 79 inclusive, heads are 0 or 1, and sector numbers are 1 to 18 inclusive. Asking for sector number 0
is always highly illegal and this is a major source of errors in prototype driver code.

It is, however, much  more logical to address things in LBA (Logical Block Addressing), as the first sector is at 0 (like an array).
Conversion between the two is rather simple. The equations are as follows:

''CYL = LBA / (HPC * SPT)''

''HEAD = (LBA % (HPC * SPT)) / SPT''

''SECT = (LBA % (HPC * SPT)) % SPT + 1''

''LBA = ( ( CYL * HPC + HEAD ) * SPT ) + SECT - 1''

This can be described in C with the following code:

<source lang="c">
void lba_2_chs(uint32_t lba, uint16_t* cyl, uint16_t* head, uint16_t* sector)
{
    *cyl    = lba / (2 * FLOPPY_144_SECTORS_PER_TRACK);
    *head   = ((lba % (2 * FLOPPY_144_SECTORS_PER_TRACK)) / FLOPPY_144_SECTORS_PER_TRACK);
    *sector = ((lba % (2 * FLOPPY_144_SECTORS_PER_TRACK)) % FLOPPY_144_SECTORS_PER_TRACK + 1);
}
</source>

You would then send this data to the floppy controller.

=== DMA Data Transfers ===
The floppy typically uses ISA DMA (which is '''not''' the same thing as PCI BusMastering DMA) to do data transfers. The floppy
is hardwired to DMA channel 2. The only other way of doing data transfers is called "PIO Mode" (see below).

To do DMA data transfers:
Set up DMA channel 2 (as in the [[DMA]] article) -- by setting the total transfer bytecount - 1, the target buffer physical address,
and the transfer direction.

init/reset the controller if needed (see below), select the drive if needed (see below),
set up the floppy controller for DMA using the "specify" command,
seek to the correct cylinder, issue a sense interrupt command, then issue the standard read/write commands. The data transfer happens invisibly.
The controller will send an IRQ6 when the transfer is complete. Then read the "result" bytes to see if there were any errors.
See below for more detail.

=== PIO Data Transfers ===
PIO data transfers can either be done using polling or interrupts. Using PIO mode can transfer data 10 percent faster than a DMA
transfer, but there is an immense cost in CPU cycle requirements. However, if your OS or application is single-tasking, then there
is nothing else that the CPU cycles can be used for anyway, so you may as well use PIO mode.

In general, the controller has a 16 byte buffer, and wants to send an IRQ6 whenever the buffer count reaches a "threshold" value that
is set by your driver. If you are using PIO mode floppy transfers in a multitasking environment (bad idea), then the IRQ6 events should
be used to fill or drain the floppy controller's buffer via some system buffer, in the interrupt handler code.

If you are using PIO mode in a singletasking environment then the IRQ6s just waste CPU cycles, and you should be using polling instead.
You can usually shut the IRQs off by setting bit 3 (value = 8) of the Digital Output Register (see below). If you want, you can even
toggle that bit on and off for specific commands, so that you receive some types of interrupts but not others.

To do PIO data transfers: init/reset the controller if needed (see below), select the drive if needed (see below),
seek to the correct cylinder, issue a sense interrupt command, then issue the standard read/write commands.
After you send the command, either poll the RQM bit in the Main Status Register to determine when the controller wants to have data moved
in/out of the FIFO buffer -- or wait for an IRQ6 to do the same thing. When the transfer is complete, read the "result" bytes to see
if there were any errors. See below for more detail.

==== Bochs Can't Handle Polling PIO ====
If you try to turn off the IRQs in Bochs (to use pure polling PIO mode), Bochs will panic. It may also not be able to handle
non-DMA data transfers properly, even if IRQs are turned on. It is currently mainly designed to model the floppy using DMA.

=== There are 3 "Modes" ===
There were several generations of floppy controller chips on several generations of 80286 (and prior) computers, before
the 82077AA chip existed. The 82077AA was built to emulate all of them, by setting various pins on the chip to 5 volts.
The three modes are: PC-AT mode, PS/2 mode, and Model 30 mode. The most likely mode you will see on any hardware that
still runs is Model 30 mode. You may find some pre-1996 Pentium machines using PS/2 mode. Sadly, most emulator programs run in PS/2 mode!
In the documentation, you can ignore PC-AT mode. Or you can try to handle all three, by only using registers and commands that are identical
in all 3 modes.

=== Most Commands Run "Silently" ===
There are only a few commands/conditions that produce interrupts: a reset (in polling mode only), Seek, Recalibrate, or one of the
read/write/verify/format commands. Several commands do not produce any result bytes, either. If you want to verify that a silent
command actually worked, just about the only thing you can do is use the Dumpreg command to check the current state of the
controller.

=== Timing Issues ===
On real hardware, there are definite timing issues. Seek delays and motor delays are just what any programmer would expect. It
would be nice if the drive would send an IRQ when the motor was up to speed, but it does not. Two things that you may not
expect are that quite new hardware probably still needs artificial delays between outputting "command/parameter" bytes, and that you
probably also need artificial delays between inputting "result" bytes. There is a bit in the MSR to test in order to know when the
next byte can be sent/retrieved. It is not a good idea to simply hardcode specific delays between output/input bytes.
Looping 20 times, and testing the value of the RQM bit in the MSR each time, should always be a sufficient "timeout".

However, using IO Port reads to generate delays (or polling MSR) leads to poor performance in a multitasking environment, and
you may want to put the driver to sleep for the shortest possible period (microsleep), instead.

=== Reliability ===
In real hardware, floppy drives are extremely unreliable. On emulators, it doesn't matter -- but for code that is intended for
real hardware, make sure to retry every command at least twice more after any error.

== Registers ==
The floppy controller is programmed through 9 registers, which can be accessed through IO ports 0x3F0 through 0x3F7 (excluding 0x3F6). As usual on the
PC architecture, some of those IO ports access different controller registers depending on whether you read from or write to them.
Note that code snippets and datasheets name these registers based on their trigrams (e.g. SRA, MSR, DIR, CCR, etc.).
Use standard "outb" and "inb" commands to access the registers.

The basic set of floppy registers can be found in the following enumeration:

<source lang="c">
enum FloppyRegisters
{
   STATUS_REGISTER_A                = 0x3F0, // read-only
   STATUS_REGISTER_B                = 0x3F1, // read-only
   DIGITAL_OUTPUT_REGISTER          = 0x3F2,
   TAPE_DRIVE_REGISTER              = 0x3F3,
   MAIN_STATUS_REGISTER             = 0x3F4, // read-only
   DATARATE_SELECT_REGISTER         = 0x3F4, // write-only
   DATA_FIFO                        = 0x3F5,
   DIGITAL_INPUT_REGISTER           = 0x3F7, // read-only
   CONFIGURATION_CONTROL_REGISTER   = 0x3F7  // write-only
};
</source>

All commands, parameter information, result codes, and disk data transfers go through the FIFO port.
MSR contains the "busy" bitflags, that must be checked before reading/writing each byte through the FIFO.
DOR controls the floppy drive motors, floppy drive "selection", and resets.
The other registers contain very little information, and are typically accessed very little, if at all.

Note: IO port 0x3F6 is the ATA (hard disk) Alternate Status register, and is not used by any floppy controller.

Note2: some people prefer to give the registers values based on their offset from the base address, and then add the
FDC's base address (0x3F0 or 0x370). So STATUS_REGISTER_A would have value 0, STATUS_REGISTER_B value 1, etc., and to
access STATUS_REGISTER_B on FDC 1, you would use FDC1_BASE_ADDRESS + STATUS_REGISTER_B = 0x370 + 1 = 0x371.

As said above, the most common controller chip has 3 modes, and many bitflags in the registers are different (or opposite!) depending on
the mode. However, all of the important registers and bitflags remain the same between modes.
They are the following:

=== FIFO ===
FIFO: The FIFO register may not have a 16byte buffer in all modes, but this is a minor difference that does not really affect its operation.

=== Tape Drive Register ===

TDR: The Tape Drive Register is a R/W register which is identical in all modes. According to the documentation for the Intel 82077AA, the two least significant bits form a number between 1 and 3 (0 is an invalid value, drive 0 may not be selected) that assigns tape support to one particular drive out of four total drives. It is probably irrelevant for modern uses, is not supported by any emulator and is probably not worth implementing support for unless you have all the real hardware to test this functionality on.

=== DOR bitflag definitions ===

{| {{wikitable}}
|-
! Mnemonic
! bit number
! value
! meaning/usage
|-
| MOTD
| 7
| 0x80
| Set to turn drive 3's motor ON
|-
| MOTC
| 6
| 0x40
| Set to turn drive 2's motor ON
|-
| MOTB
| 5
| 0x20
| Set to turn drive 1's motor ON
|-
| MOTA
| 4
| 0x10
| Set to turn drive 0's motor ON
|-
| IRQ
| 3
| 8
| Set to enable IRQs and DMA
|-
| RESET
| 2
| 4
| Clear = enter reset mode, Set = normal operation
|-
| DSEL1 and 0
| 0, 1
| 3
| "Select" drive number for next access
|}

Note: the IRQ/DMA enable bit (bit 3, value = 8) cannot be cleared in "PS/2 mode", so for PIO transfers you should make sure to have a stubbed IRQ6 handler
in place, just in case the IRQs happen anyway. The bit '''must''' be set for DMA to function.

Note2: if you want to execute a command that accesses a disk (see the command list below), then that respective disk must have its
motor spinning (and up to speed), '''and''' its "select" bits must be set in the DOR, '''first'''.

Note3: toggling DOR reset state requires a 4 microsecond delay. It may be smarter to use DSR reset mode, because
the hardware "untoggles" reset mode automatically after the proper delay.

=== MSR bitflag definitions ===

{| {{wikitable}}
|-
! Mnemonic
! bit number
! value
! meaning/usage
|-
| RQM
| 7
| 0x80
| Set if it's OK (or mandatory) to exchange bytes with the FIFO IO port
|-
| DIO
| 6
| 0x40
| Set if FIFO IO port expects an IN opcode
|-
| NDMA
| 5
| 0x20
| Set in Execution phase of PIO mode read/write commands only.
|-
| CB
| 4
| 0x10
| Command Busy: set when command byte received, cleared at end of Result phase
|-
| ACTD
| 3
| 8
| Drive 3 is seeking
|-
| ACTC
| 2
| 4
| Drive 2 is seeking
|-
| ACTB
| 1
| 2
| Drive 1 is seeking
|-
| ACTA
| 0
| 1
| Drive 0 is seeking
|}

The two important bits are RQM and DIO. NDMA and BUSY are also useful in polling PIO mode.
Most important is RQM, which is set when it is OK (or necessary!) to read/write data from/to the FIFO port.
NDMA signals the end of the "execution phase" of a command, and the beginning of "result phase".
DIO and BUSY should be checked to verify proper command termination (the end of "result phase" and beginning of "command phase").

=== CCR and DSR ===
The bottom 2 bits of DSR match CCR, and setting one of them sets the other. The upper 6 bits on both DSR and CCR default to 0, and
can always be set to zero safely. So, even though they have different bit definitions, you always set them with identical values, and
one or the other of DSR and CCR can be ignored in any modern system.
Alternately, you can always set both of them, for maximum compatibility with ancient chipsets.

The bottom 2 bits specify the data transfer rate to/from the drive. You want both bits set to zero for a 1.44MB or 1.2MB floppy drive.
So generally, you want to set CCR to zero just once, after bootup (because the BIOS may not have done it, unless it booted a floppy disk).

Note: a reset procedure does not affect this register. However, if you have drives of different types on the bus which
use different datarates, then you need to switch the datarate when you select the other drive. It also seems to be possible
to modify this register while the FDC is in "reset state".

Note2: some tutorials seem to claim that changing/setting the datarate causes an IRQ6. This is false.

Datarates used for setting either DSR or CCR:

<pre>
Datarate   value   Drive Type
1Mbps        3       2.88M
500Kbps      0       1.44M, 1.2M
</pre>

Note: There is also a 300Kbps (value = 1), and a 250Kbps setting (value = 2) but they are only for utterly obsolete drives/media.

=== DIR register, Disk Change bit ===
This bit (bit 7, value = 0x80) is fairly useful. It gets set if the floppy door was opened/closed. Sadly, almost all the emulator
programs set and clear this bit completely inappropriately (especially after a reset).
Do not trust your handling of this bit until you have tested the functionality on real hardware.

Note: The datasheet is very confusing about the value of the bit, because Model 30 mode shows the bit as being inverted. But
in Model 30 mode, the '''signal''' is also inverted, so it comes out the same. "False" always means the bit is cleared, and
"true" always means the bit is set.

Note2: You must turn on the drive motor bit before you access the DIR register for a selected drive (you do not have to wait
for the motor to spin up, though). It may also be necessary to read the register five times (discard the first 4 values) when
'''changing''' the selected drive -- because "selecting" sometimes takes a little time.

Basically, you want to keep a flag for whether there is media in each drive. If Disk Change is set and there was media,
the OS should get a signal that the previous media was ejected.

Once the Disk Change bit signals "true" (and you have processed that "event"), you need to try to clear the bit. The main way to
clear the bit is with a '''successful''' Seek/Recalibrate to a '''new''' cylinder on the media. (A reset does not work. If the
controller thinks the heads are already on the correct cylinder, it will eat a Seek command without clearing the Disk
Change bit. If the heads are already on cylinder 0, a Recalibrate is also a no-op.) If the seek '''fails''', you can be fairly
certain that there is no media in the drive anymore. It is important to note that
this means you should check the value of Disk Change just prior to every Seek command that you do, because otherwise you
will lose any Disk Change information. This is also true for implied seeks, and relative seeks.

Apparently a small number of floppy drives also support one additional way to clear the bit -- something that
Linux calls a "twaddle". Simply toggle the drive motor bit on, off, and then on again. When your driver tries to clear the
Disk Change bit the first time, it can try a twaddle, and see if it works, and keep a flag if it does.


== Programming Details ==
Overall, the controller needs to be initialized and reset once (see below for the steps involved). Then drives can be
accessed. To access drives:
# Turn on the drive's motor and select the drive, using an "outb" command to the DOR IO port.
# Wait for awhile for the motor to get up to speed, using some waiting scheme.
# Issue your command byte plus some parameter bytes (the "command phase") to the FIFO IO port.
# Exchange data with the drive / "seek" the drive heads (the "execution phase"), on the FIFO IO port.
# Get an IRQ6 at the end of the execution phase, but '''only if the command HAS an execution phase'''.
# Read any "result bytes" produced by the command (the "result phase"), on the FIFO IO port.
# The commands "Recalibrate", "Seek", and "Seek Relative" do not have a result phase, and require an additional "Sense Interrupt" command to be sent.

And then you are ready for the next command. See below for more detail on how to issue a command.

=== Motor Delays ===
Note: the Linux floppy driver sourcecode has a comment that claims that turning on the MOTC or MOTD bits in the DOR
can completely lock up some systems. This claim seems unlikely?

When you turn a floppy drive motor on, it takes quite a few milliseconds to "spin up" -- to reach the (stabilized) speed
needed for data transfer. The controller has electronics to handle a large variation in rotation speed, but it has its limits.
If you start reading/writing immediately after the motor is turned on, "the PLL will fail to lock on to the data signal" and you will get an error.

After you are done reading (or writing), you should typically wait an additional two seconds to turn the motor off. (It may also be smart
to seek the heads back to cylinder 0 just before turning the motor off.) You could leave the motor on longer,
but it might give the user the idea that the floppy is still transferring data and that it's taking an awfully long time.
The reason to leave the motor on is that your driver may not know if there is a queue of sector reads or writes
that are going to be executed next. If there are going to be more drive accesses immediately, they won't need to wait
for the motor to spin up.

The suggested delays when turning the motor on are:

* 300 milliseconds (for a 3.5" floppy).
* 500 milliseconds (for a 5.25" floppy).

These values should be more than enough for any floppy drive to spin up correctly. 50ms should be sufficient, in fact.

Another functional method is not delaying or waiting at all, but just enter a loop and keep retrying any command until it works.
A 3.5 inch disk rotates once every 200ms, so each retry is effectively a delay.

=== Gap Lengths ===
There are two different gap lengths that are controlled by software for specifying the amount of blank space between sectors.
The gap lengths are used by the floppy hardware to help find the "start of sector" markers, and to avoid problems caused by speed variations
in different floppy drives (for e.g. writing a sector on a slower drive would cause the sector to take up more physical space on the disk,
potentially overwriting the next sector). The first gap length (typically called "GPL1") is used when reading or writing data, and sets the
length of the gap between sectors that the floppy should expect (but doesn't change this gap). The second (typically called "GPL2")
is the gap length for the "format track" command, which specifies the amount of space between sectors to use. The actual gap lengths depend on many factors,
but GPL1 is always a bit less than GPL2 so that the floppy hardware starts expecting the next sector near the end of the blank space.

Standard values for these gap lengths can be found in the floppy controller datasheets. However, it is possible to squeeze more sectors
on each track by reducing the gap length. For example, by using a gap length of 0x1C when formatting (and 0x0C when reading/writing)
instead of 0x54 (and 0x1B when reading/writing) it's possible to format a normal 1440 KB floppy disk with 21 sectors per track to create
a 1680Kb floppy disk. These disks may be unreliable on very old computers (where accuracy and speed variation may be worse),
but were considered reliable enough for Microsoft to distribute a version of Windows on 1680Kb floppies (Windows 95 on a set of 12 floppies).
It is also possible to format 3 extra cylinders on each disk, for a total of 83.

=== Hardware Control of Bad Cylinders ===
The FDC subsystem has a built-in method for handling unreliable media. However, it is probably not a good idea to use it.
It involves finding a bad sector on the media, and then marking the entire track or cylinder as being bad, during the formatting process.

If you try to do this, then you cannot simply seek to a cylinder. All of the cylinders get "remapped" with new "TrackID"s. Whenever
you seek to a cylinder, then you need to use the ReadID command to verify that the cylinder you seeked to contains the data that you
actually want. So this eliminates any possibility of using implied seeks, and adds an extra step to most read/write operations.

=== Procedures ===

==== Reinitialization ====

The BIOS probably leaves the controller in its default state. "Drive polling mode" on, FIFO off, threshold = 1, implied seek off,
lock off. This is a lousy state for the controller to be in, and your OS will need to fix it. The BIOS probably also does not have
any better guess as to the proper values for the "Specify" command than your OS does (the values are specific to the particular
drive). There is certainly no reason why you should trust the BIOS to have done any of it '''correctly.'''

So, when your OS is initializing:
# Send a Version command to the controller.
# Verify that the result byte is 0x90 -- if it is not, it might be a good idea to abort and not support the floppy subsystem. Almost all of the code based on this article will work, even on the oldest chipsets -- but there are a few commands that will not.
# If you don't want to bother having to send another Configure command after every Reset procedure, then:
## Send a better Configure command to the controller. A suggestion would be: drive polling mode off, FIFO on, threshold = 8, implied seek on, precompensation 0.
## send a Lock command.
# Do a Controller Reset procedure.
# Send a Recalibrate command to each of the drives.

==== Controller Reset ====

The second most common failure mode for any floppy command is for the floppy controller to lock up forever. This
condition is detected with a timeout, and needs to be fixed with a Reset. You also need one Reset during initialization. Hopefully, it is
the only one you will ever need to do.

# Either use:
## Bit 2 (value = 4) in the DOR: Save the current/"original" value of the DOR, write a 0 to the DOR, wait 4 microseconds, then write the original value (bit 2 is always set) back to the DOR.
## '''or''' Bit 7 (value = 0x80) in the DSR: Precalculate a good value for the DSR (generally 0), and OR it with 0x80. Write that value to the DSR.
# Wait for the resulting IRQ6 (unless you have IRQs turned off in the DOR)
# If (and only if) [[#Configure|drive polling mode]] is turned on, send 4 Sense Interrupt commands (required).
# If your OS/driver never sent a Lock command, then you probably need to send a new Configure command (the fifo settings were lost in the reset).
# Do a Drive Select procedure for the next drive to be accessed.

Note: A reset clears all the Specify information, so the next Select procedure must send a new Specify command (use some sort of flag to tell the
driver to do this).

Note2: Emulators will often set the Disk Change flag to "true" after a reset, '''but this does not happen on real hardware''' -- it is
a shared bug in all the emulators that do it. Another shared bug is that most emulators do not fire an IRQ6 if disk polling mode is off.

Note3: A reset does not change the [[#Configure|drive polling mode or implied seek]] settings.

==== Drive Selection ====

Each floppy drive on the system may be a different type. When switching between accessing different types of drives, you need
to fix the Specify and Datarate settings in the controller. The controller does '''not''' remember the settings on a per-drive
basis. The only per-drive number that it remembers is the current cylinder.

# Send the correct Datarate setting to CCR. Usually this is a 0 (1.44MB floppy drive).
# If the newly selected drive is a different type than the previously selected drive (or changing from PIO to DMA mode), send a new Specify command.
# Set the "drive select" bits (and the other bitflags) in DOR properly, including possibly turning on the Motor bit for the drive (if it will be accessed soon).

==== Detecting Media ====

The user can swap media out of a floppy drive at any moment. If your driver sends a command to the drive, and the
command fails -- this may be the reason why.

# Turn the drive motor bit on.
# Read DIR. If the "Disk Change" bitflag is set to "true", then the floppy drive door was opened, so the OS needs to test if a new disk is in the drive.

==== Waiting ====

Waiting for the drive can be done in many ways, and it is an OS-specific design decision. You can poll the value of
the PIT, waiting for it to count down to a certain value. You can poll a memory location that contains "the current time"
in some format, waiting for it to reach a certain value. Your OS can implement a realtime callback, where a particular function
in the driver will be called at a particular time. Or you can implement some form of multitasking "blocking", where the driver
process is put in a "sleep" state, and is not assigned any timeslices for a certain length of time.

==== The Proper Way to issue a command ====

# Read MSR (port 0x3F4).
# Verify that RQM = 1 and DIO = 0 ((Value & 0xc0) == 0x80) -- if not, reset the controller and start all over.
# Send your chosen command byte to the FIFO port (port 0x3F5).
# In a loop: loop on reading MSR until RQM = 1. Verify DIO = 0, then send the next parameter byte for the command to the FIFO port.
# Either Execution or Result Phase begins when all parameter bytes have been sent, depending on whether you are in PIO mode, and the command has an Execution phase. If using DMA, or the command does not perform read/write/head movement operations, skip to the Result Phase.
# (In PIO Mode Execution phase) read MSR, verify NDMA = 1 ((Value & 0x20) == 0x20) -- if it's not set, the command has no Execution phase, so skip to Result phase.
# begin a loop:
# Either poll MSR until RQM = 1, or wait for an IRQ6, using some waiting method.
# In an inner loop: transfer a byte in or out of the FIFO port via a system buffer, then read MSR. Repeat while RQM = 1 and NDMA = 1 ((Value & 0xa0) == 0xa0).
# if NDMA = 1, loop back to the beginning of the outer loop, unless your data buffer ran out (detect underflow/overflow).
# Result Phase begins. If the command does not have a Result phase, it silently exits to waiting for the next command.
# If using DMA on a read/write command, wait for a terminal IRQ6.
## Loop on reading MSR until RQM = 1, verify that DIO = 1.
## In a loop: read the next result byte from the FIFO, loop on reading MSR until RQM = 1, verify CMD BSY = 1 and DIO = 1 ((Value & 0x50) == 0x50).
# After reading all the expected result bytes: check them for error conditions, verify that RQM = 1, CMD BSY = 0, and DIO = 0. '''If not''' retry the entire command again, several times, starting from step 2!

Note: implementing a failure timeout for each loop and the IRQ is pretty much required -- it is the only way to detect many command errors.

=== Commands ===
Each command is a single byte with a value less than 32, which is written to the DATA_FIFO port. There are three "option bits"
that can be OR'ed onto some command bytes, typically called MF, MT, and SK. Each command must be followed by a specific set of
"parameter bytes", and returns a specific set of "result bytes". See the discussion of each command for its list of parameter bytes,
and result bytes.

A command byte may only be sent to the FIFO port if the RQM bit is 1 and the DIO bit is 0, in the MSR. If these
bits are not correct, then the previous command encountered a fatal error, and you must issue a [[#Controller Reset|reset]].

The following is an enumeration of the values of the command bytes. The ones that you actually will '''use''' are marked
with a * and a comment.

<source lang="c">
enum FloppyCommands
{
   READ_TRACK =                 2,	// generates IRQ6
   SPECIFY =                    3,      // * set drive parameters
   SENSE_DRIVE_STATUS =         4,
   WRITE_DATA =                 5,      // * write to the disk
   READ_DATA =                  6,      // * read from the disk
   RECALIBRATE =                7,      // * seek to cylinder 0
   SENSE_INTERRUPT =            8,      // * ack IRQ6, get status of last command
   WRITE_DELETED_DATA =         9,
   READ_ID =                    10,	// generates IRQ6
   READ_DELETED_DATA =          12,
   FORMAT_TRACK =               13,     // *
   DUMPREG =                    14,
   SEEK =                       15,     // * seek both heads to cylinder X
   VERSION =                    16,	// * used during initialization, once
   SCAN_EQUAL =                 17,
   PERPENDICULAR_MODE =         18,	// * used during initialization, once, maybe
   CONFIGURE =                  19,     // * set controller parameters
   LOCK =                       20,     // * protect controller params from a reset
   VERIFY =                     22,
   SCAN_LOW_OR_EQUAL =          25,
   SCAN_HIGH_OR_EQUAL =         29
};
</source>

==== "Deleted sectors" ====
Deleted sectors is a legacy feature, dating back to the 1970s, when IBM data entry terminals stored a single database record in each (128 byte) floppy sector, and a record could be marked as deleted by writing a different address mark before the sector. This feature was almost never used with IBM PC and compatibles – exceptions were occasional abuse by copy protection schemes, and (possibly) hiding bad sectors. Few (if any) emulators emulate this functionality, and many late model FDCs didn't implement it.

The "WRITE_DELETED_DATA" command can be used to create a deleted sector, and the "READ_DELETED_DATA" command can be used to read one back.

==== Option bits ====
OR these bits onto the above read/write/format/verify commands.

===== Bit MT =====
Value = 0x80. Multitrack mode. The controller will switch automatically from Head 0 to Head 1
at the end of the track. This allows you to read/write twice as much data with a single command.
===== Bit MF =====
Value = 0x40. "MFM" magnetic encoding mode. Always set it for read/write/format/verify operations.

A zero value represents the old "single density" FM format, which (on IBM PCs) was only used by 8-inch floppies (which were extremely rare). Few people ever used FM format, except for data interchange with non-PC systems (such as CP/M or minicomputers). Later model FDCs (e.g. Intel 82078) dropped support for it.

===== Bit SK =====
Value = 0x20. Skip mode. This skips "deleted sectors" (see explanation in section above).

Ignore this bit and leave it cleared, unless you have a really good reason not to.

==== Status Registers ====
There are 3 registers that hold information about the last error encountered. The st0 register information
is passed back with the result bytes of most commands. The st1 and st2 information is returned in the result
bytes of read/write commands. They can also be retrieved with a Dumpreg command.

===== st0 =====
The top 2 bits (value = 0xC0) are set after a reset procedure, with polling on. Either bit being set at any other
time is an error indication.
Bit 5 (value = 0x20) is set after every Recalibrate, Seek, or an implied seek. The other bits are not useful.

===== st1 =====
The st1 register provides more detail about errors during read/write operations.

Bit 7 (value = 0x80) is set if the floppy had too few sectors on it to complete a read/write. This is often
caused by '''not subtracting 1''' when setting the DMA byte count.
Bit 4 (value = 0x10) is set if your driver is too slow to get bytes in or out of the FIFO port in time.
Bit 1 (value = 2) is set if the media is write protected.
The rest of the bits are for various types of data errors; indicating bad media, or a bad drive.

===== st2 =====
The st2 register provides more (useless) detail about errors during read/write operations.

The bits all indicate various types of data errors for either bad media, or a bad drive.


==== Configure ====
This command initializes controller-specific values: the data buffer "threshold" value, implied seek enable, FIFO disable, polling enable.
(And "Write Precompensation".) A good setting is: implied seek on, FIFO on, drive polling mode off, threshold = 8, precompensation 0.

If you enable implied seeks, then you don't have to send Seek commands (or Sense Interrupt commands for the Seek commands).

If you leave the FIFO disabled, then it cannot buffer data -- you will get an IRQ6 or DMA request for every single byte (which needs
to be serviced quickly, because the very next byte will automatically cause an overflow/underflow and error out your
transfer!).

Drive polling mode is just an annoyance that is there for backwards software compatibility. It makes you need Sense Interrupts
after every reset. Always turn it off, if you send a Configure command.

A big "threshold" such as 15 will wait 15 bytes between interrupts. So there won't be many interrupts, but you only
have one byte left in the FIFO before it overflows/underflows and kills the r/w operation. thresh_val = threshold - 1.

Write precompensation is a technical thing having to do with drive head magnetics. A precomp_val of 0 tells the controller/drive
to use the manufacturer default value. Use that unless you have a very good reason for setting another value.

* Configure command = 0x13
* First parameter byte = 0
* Second parameter byte = (implied seek ENable << 6) | (fifo '''DIS'''able << 5) | (drive polling mode '''DIS'''able << 4) | thresh_val (= threshold - 1)
* Third parameter byte = precomp_val = 0
* No result bytes.
* No interrupt.

==== Version ====
Returns one byte. If the value is 0x90, the floppy controller is a 82077AA.

* Version command = 0x10
* No parameter bytes.
* No interrupt.
* First result byte = 0x90

==== Lock ====
Under default circumstances, every Controller Reset will disable the fifo, and set the fifo threshold to 1 (thresh_val = 0).
If you change these settings with the Configure command and don't want to have to fix them after every Controller Reset,
then you can send a Lock command with the lock bit turned on. You can "unset" the lock, by sending another Lock command
with the lock bit turned off. Use the MT option bit as the lock bit.

* Lock command = 0x94
* '''or''' Unlock command = 0x14
* No parameter bytes.
* No interrupt.
* First result byte = lock bit << 4

==== Specify ====
This command puts information in the controller about the next disk drive to be accessed.

'''Important Note:''' The controller only has one set of registers for this information. It does not '''store''' the information
between multiple drives. So if you are switching control between two different types of drives (with different specify values)
then you need to send a new Specify command every single time you select the other drive.

If your driver will be using DMA to transfer data, set the NDMA bit to 0. If using PIO mode instead, set it to 1.

* Specify command = 0x3
* First parameter byte = SRT_value << 4 | HUT_value
* Second parameter byte = HLT_value << 1 | NDMA
* No result bytes.
* No interrupt.

===== SRT, HLT and HUT =====
These parameters sent with the Specify command to the controller are meant to optimize drive performance, and head lifetime.

The values are specific to the exact model, condition, and age of floppy drive installed on the system. The values sent by the driver to the
controller were always meant
to be '''adaptive'''. That is, your driver is theoretically supposed to keep statistics of how often Seek commands fail with the current
setting of SRT. If they always work, and your driver wants to optimize performance, then it can send a new Specify command, with the SRT
value reduced by 1. Then begin keeping new statistics. Similarly for HLT regarding Read/Write operations. As drives age and collect
dirt, the driver would automatically compensate by seeing higher statistical error rates, and increase the values of SRT and HLT.

Keeping statistics in that way only works when the drive in question is used often. Now that internal floppy drives are nearly obsolete,
it is worthless. So the current recommendation is just to use very safe values, and forget about performance.

If you look up spec sheets for individual floppy drives, they usually show a worst-case "track to track seek time" = SRT, but not the other two.

* SRT = "Step Rate Time" = time the controller should wait for the head assembly to move between successive cylinders. A reasonable amount of time
to allow for this is 3ms for modern 3.5 inch floppy drives. A very safe amount would be 6 to 8ms. To calculate the value for the SRT setting
from the given time, use "SRT_value = 16 - (milliseconds * data_rate / 500000)". For a 1.44 MB floppy and 8ms delay this gives "SRT_value =
16 - (8 * 500000 / 500000)" or a parameter value of 8.

* HLT = "Head Load Time" = time the controller should wait between activating a head and actually performing a read/write.
A reasonable value for this is around 10ms. A very safe amount would be 30ms. To calculate the value for the HLT setting
from the given time, use "HLT_value = milliseconds * data_rate / 1000000".
For a 1.44 MB floppy and a 10ms delay this gives "HLT_value = 10 * 500000 / 1000000" or 5.

* HUT = "Head Unload Time" = time the controller should wait before deactivating the head. To calculate the value for the HUT setting
from a given time, use "HUT_value = milliseconds * data_rate / 8000000".
For a 1.44 MB floppy and a 240 mS delay this gives "HUT_value = 24 * 500000 / 8000000" or 15. However, it seems likely that the smartest
thing to do is just to set the value to 0 (which is the maximum in any mode).

==== Sense Interrupt ====
This command's main function is to return any error code from a Seek or Recalibrate command to your driver. It also clears an internal
bitflag in the controller. It is required in three circumstances that produce interrupts.

# After doing a Controller Reset procedure with [[#Configure|drive polling mode]] turned on.
# After the completion of a Seek command (or Relative Seek).
# After the completion of a Recalibrate command.

These are the only times when you should send a Sense Interrupt. You should still send them even if you have IRQs turned off in
the DOR and you are using PIO polling instead. If you send Sense Interrupt commands at other times: the command will complete, return
a 0x80, and then lock up the controller until you do a Reset.

* Sense Interrupt command = 0x8
* No parameter bytes.
* No interrupt.
* First result byte = [[#st0|st0]]
* Second result byte = controller's idea of the current cylinder

Note: if you try to read the result bytes without waiting for RQM to set, then you are likely to always get an incorrect result value
of 0. This is also likely to get your driver out of sync with the FDC for input/output. The correct value of st0 after a reset should
be 0xC0 | drive number (drive number = 0 to 3). After a Recalibrate/Seek it should be 0x20 | drive number.

==== Recalibrate ====
Note: There is an error in the FDC datasheet regarding this command. Some statements say the command will try a maximum of 80 head
assembly steps. In other places it says 79 steps. The value 79 is correct.

The motor needs to be on, and the drive needs to be selected. For this particular command, you do not have to wait for the command to
complete before selecting a different drive, and sending another Recalibrate command to it (but the Step Rates have to match, for this
to work).

It is possible for a normal 1.44M floppy to be formatted with 83 cylinders. So, theoretically, it may take two (or more) Recalibrates
to move the head back to cylinder 0. It is a good idea to test bit 5 (value = 0x20) in [[#st0|st0]] after the Sense Interrupt, and
retry the Recalibrate command if that bit is clear.

* Recalibrate command = 0x7
* First parameter byte = drive number = 0 to 3.
* No result bytes.
* The interrupt may take up to 3 seconds to arrive, so use a long timeout.

It is possible to poll the "disk active" bits in the MSR to find out when the head movement is finished.
A Sense Interrupt command is required after this command completes, to clear it from being BUSY. (Multiple Sense Interrupts,
if you ran multiple simultaneous Recalibrates.)

==== Seek ====
The motor needs to be on, and the drive needs to be selected.
For this particular command, you do not have to wait for the command to complete before selecting a different drive, and sending another
Seek command to it. Maximum cylinder number is 255; if the disk has more, you must use the Relative Seek command, instead. There is
really no reason to ever use head 1 when seeking.

* Seek command = 0xf
* First parameter byte = (head number << 2) | drive number (the drive number must match the currently selected drive!)
* Second parameter byte = requested cylinder number
* No result bytes.
* The interrupt may take up to 3 seconds to arrive, so use a long timeout.

It is possible to poll the "disk active" bits in the MSR to find out when the head movement is finished.

A Sense Interrupt command is required after this command completes, to clear it from being BUSY. (Multiple Sense Interrupts,
if you ran multiple Seeks.)

Note: the controller tries to remember what cylinder each drive's heads are currently on. If you try to seek to that same
cylinder, then the controller will silently ignore the command (and return a "success" value). One of the things this means
is that you can get a "success" return value on a seek '''even if there is no media in the drive''', if you happen to seek
to the wrong cylinder number.

===== Relative seek =====
The normal Seek command allows you to select an absolute cylinder number from 0 to 255. It is also possible to use a "relative"
seek command in all situations, and especially for drives that have more than 255 cylinders (there are none, currently).

To use a relative seek, set the MT bit to 1. To seek to higher cylinder numbers set the MFM bit -- clear MFM to seek
backwards to lower cylinder numbers. If you seek past cylinder 255, there are a lot of extra complications.
Otherwise, the command behaves identically to regular Seek.

==== Read/Write ====
Note: Remember that this is in CHS format, so the sector number starts at 1.

* Read command = MT bit | MFM bit | 0x6
* or Write command = MT bit | MFM bit | 0x5
* First parameter byte = (head number << 2) | drive number (the drive number must match the currently selected drive!)
* Second parameter byte = cylinder number
* Third parameter byte = head number (yes, this is a repeat of the above value)
* Fourth parameter byte = starting sector number
* Fifth parameter byte = 2 (all floppy drives use 512bytes per sector)
* Sixth parameter byte = EOT (end of track, the last sector number on the track)
* Seventh parameter byte = 0x1b (GAP1 default size)
* Eighth parameter byte = 0xff (all floppy drives use 512bytes per sector)

* First result byte = [[#st0|st0]] status register
* Second result byte = [[#st1|st1]] status register
* Third result byte = [[#st2|st2]] status register
* Fourth result byte = cylinder number
* Fifth result byte = ending head number
* Sixth result byte = ending sector number
* Seventh result byte = 2

Note: if you try to read the result bytes without waiting for RQM to set, then you are likely to always get an incorrect result
value of 0. This is also likely to get your driver out of sync with the FDC for input/output.

Note2: You'll see that there's no sector count parameter. Instead, the FDC figures out when to stop by the DMA signaling to the FDC
that it's done, or in PIO mode, by the FDC experiencing a FIFO overrun or underrun.

Note3: Floppy media and electronics are well known for being unreliable. Any read or write command that fails should be retried
at least twice, unless it was a write and failed on "write protect".

=== Perpendicular Mode and 2.88M floppies ===
If you are using an emulator and you need a floppy disk image that is bigger than 1440Kb, there is a 2880Kb image available.
In order to access it in Pmode, you need to modify your driver to handle Perpendicular Mode. Basically, it is an extra configuration
command where you enable any of the four drives for perpendicular mode.

Note: If the parameter byte is 0 (except for the "perpendicular enable" bits), then a reset will not affect the settings.

* Perpendicular Mode command = 0x12
* First parameter byte = (Drive 3 enable << 5) | (Drive 2 enable << 4) | (Drive 1 enable << 3) | (Drive 0 enable << 2)
* No result bytes.
* No interrupt.

You also need to set CCR/DSR for the 1M datarate (value = 3) to access a 2.88M drive.

=== Additional Programming Notes ===
If you are doing a transfer between 2 floppy drives (so that both motors are on), and you are toggling "selection" between the two,
there may be a short delay required.

== Code Examples ==

=== A common coding error example ===
The following code intentionally contains a common bug that causes an infinite loop (waiting for IRQ6) on most emulators.

<source lang="c">
volatile byte ReceivedIRQ = false;

// This function gets called when an IRQ6 is generated.
void FloppyHandler()
{
   ReceivedIRQ = true;
}

// Waits for an IRQ to be issued.
void WaitForIRQ()
{
   ReceivedIRQ = false;
   while(!ReceivedIRQ) ;
}

// buggy example Controller Reset function
void ResetFloppy()
{
    DisableController();
    EnableController();

    WaitForIRQ();
}
</source>

Sure this code ''looks'' OK, but some emulators or floppy drives might manage to be faster than your code. What if you've just returned from
''EnableController()'' and the floppy already issued the IRQ6? Then ''ReceivedIRQ'' will be set to true, your driver will enter ''WaitForIRQ()'',
set it to false again and then infinitely loop, waiting for an IRQ that has already been received. It's usually better to do something like:

<source lang="c">
volatile byte ReceivedIRQ = false;

// This function gets called when an IRQ6 is generated.
void FloppyHandler()
{
   ReceivedIRQ = true;
}

// pretty good Controller Reset function (it should do more checking of MSR)
void ResetFloppy()
{
    ReceivedIRQ = false; 	// This will prevent the FDC from being faster than us!

    // Enter, then exit reset mode.
    outb(Controller.DOR,0x00);
    outb(Controller.DOR,0x0C);

    while(!ReceivedIRQ) ;	// Wait for the IRQ handler to run

    // sense interrupt -- 4 of them typically required after a reset
    for (i = 4 ; i > 0 ; --i);
    {
       send_command(SENSE_INTERRUPT);
       read_data_byte();
       read_data_byte();
    }

    outb(controller.CCR,0x00);	// 500Kbps -- for 1.44M floppy

    // configure the drive
    send_command(SPECIFY);
    outb(Controller.FIFO, steprate_headunload);
    outb(Controller.FIFO, headload_ndma);
}
</source>

== Related Links ==
=== Articles ===
* [[DMA]]

=== Reference Documents ===
* http://www.osdever.net/documents/82077AA_FloppyControllerDatasheet.pdf?the_id=41
* http://bos.asmhackers.net/docs/floppy/
* http://bos.asmhackers.net/docs/floppy/docs/floppy_tutorial.txt
* [http://www.intel.com/design/archives/periphrl/docs/29046803.htm Intel 82078 CHMOS SINGLE-CHIP FLOPPY DISK CONTROLLER datasheet (useless)]
* http://www.brokenthorn.com/Resources/OSDev20.html

=== Forum Posts ===
* [[topic:13538|TUTORIAL, with DMA, by Mystran (highly recommended, but has a few tiny errors)]]
* [[topic:310|Floppy in pmode]]
* [[topic:12279|Floppy Disk Driver]]
* [[topic:8405|Floppy programming tutorial (floppy_tutorial.txt) companion thread]]
* [[topic:11181|Multitask Floppy Driver]]
* [[topic:17274|PIO mode information]]

=== Implementations ===
* [https://github.com/FDOS/kernel/blob/master/drivers/floppy.asm FreeDOS] (C, GPL)
* [http://bos.asmhackers.net/docs/floppy/snippet_9/fdc.c GazOS] (C, GPL)
* [http://bos.asmhackers.net/docs/floppy/snippet_5/FLOPPY.ASM RDOS] (Assembly, GPL)
* [https://github.com/torvalds/linux/blob/master/drivers/block/floppy.c Linux] (C, GPL)

[[de:Floppy Disk Controller]]
[[Category:Storage]]
[[Category:Common Devices]]
````

<a id="w001460"></a>

## W001460 — Memory Allocation

Bron-ID: W001460
Namespace: Main (0)
Revisie: 30361 | Datum: 2026-03-19T23:08:33Z
Bron: https://wiki.osdev.org/Memory_Allocation
Revisie-URL: https://wiki.osdev.org/index.php?title=Memory_Allocation&oldid=30361

````text
:''This page is about the allocation of memory from memory that is already available to the process (like malloc() and new()). For the allocation of page frames see [[Page Frame Allocation]].''

One of the most basic functions of a [[Kernels|kernel]] is the [[Memory Management|memory management]], i.e. the allocating and freeing of memory.

At square one, the kernel is the only process in the system. But it is not alone: [[BIOS]] data structures, memory-mapped hardware registers etc. populate the address space. Among the first things a kernel must do is to start bookkeeping about which areas of physical memory are available for use and which are to be considered "occupied".

The free space will subsequently be used for kernel data structures, application binaries, their heap and [[stack]] etc. - the kernel needs a function that marks a memory area as reserved, and makes that memory available to the process requiring it. In the C Standard Library, this is handled by malloc() and free(); in C++ by new() and delete().

==The Big Picture==

An operating system has many [[Memory_Management|memory management]] layers. Each layer is built on top of the previous layer. To put you in perspective:
* RAM is allocated in the Physical Memory Manager, which is typically a [[Page_Frame_Allocation|page frame allocator]] (another popular choices are [https://en.wikipedia.org/wiki/Slab_allocation SLAB] and buddy allocator).
* Pages are allocated into address space (typically via sbrk() or mmap() system calls) by the Virtual Memory Manager.
* Then the available address space is managed by a user-space allocator, typically called malloc(), this is the topic at hand.

Kernel heap management is very similar, has the same abstractions, but uses different functions:
* at the bottom level, uses the same PMM.
* memory is usually mapped into kernel address space by a special function, and there's no need for system call to raise privilege level.
* allocating kernel [[Heap|heap]] is done via kmalloc(), which is very similar to malloc(), but not defined in libc rather in a kernel library.

As you can see, the VMM differs in only one thing: which [[Paging|paging]] tables are used to map the pages allocated by the PMM (user-space vs. kernel-space). Some protection bits, like supervisor page also differ, otherwise these steps are identical.

Top level memory management is also very similar, the only difference is, which callback is used in the VMM when the manager runs out of free virtual memory. This implies that you can use any existing memory allocator as kmalloc() too, if you replace that callback in them.

So memory allocation goes through these steps:
 1. do we have enough free virtual memory?
 2. if so, use some structure to record memory allocation status (depends on the allocator)
 3. if not, then ask the VMM to enlarge available address space (sbrk/mmap/mapkernelheap etc.)
 4. VMM in turn calls the PMM to allocate RAM
 5. the newly allocated RAM is recorded in the appropriate paging tables by the VMM
 6. go to step 2.

Steps 3 - 5 are slow (mainly because of the required privilege level change), so memory allocators tend to minimize these calls, and reuse the memory in their already mapped address space as much as possible.

==A very very simple Memory Manager==

The easiest you can do is the WaterMark allocator. Just keep track of how far you've allocated and forget about the notion of "freeing".

            before ...
                                                           <-N-->
    +----+-----+--+--------//--+             +----+-----+--+----+---//--+
    |##A#|##B##|C#|  free      |             |##A#|##B##|C#|##D#|free   |
    +----+-----+--+---------//-+             +----+-----+--+----+---//--+
                  ^freebase    ^freetop                    ^d   ^fbase  ^ftop

When allocating N bytes for D, simply check that freetop-freebase>N and increment freebase by N. Period.
A very simple Memory Manager

Now, if you need to free things, one of the easiest solution is to put at the start of the freed zone a descriptor that allows you to insert it in a list of free zones. Keeping that list sorted by address helps you identifying contiguous free zones and allows you to merge them in larger free zones.

       first free                        Structure of a   +--.--.--.--+    +---> +-- ...
         \/                              free zone        |F  R  E  E |    |     | FREE
    +----+-----+--+----+---+-----//--+                    |nextfreeptr|----+
    |##A#|free |C#|free|#E#|   free  |               <----|prevfreeptr|
    +----+-|---+--+-|--+---+-----//--+                    | zone size |
           +-next>--+                                     +///////////+

<!--
What data structure and algorithms can be used with such lists has been discussed in [http://www.mega-tokyo.com/forum/index.php?board=1;action=display;threadid=6486 Memmaker's] started thread (as well as in many others)
-->

==Fixed size allocation==
Allocating and deallocating fixed sized areas of memory is extremely simple. You can basically treat all free memory as a linked list of nodes. To allocate memory, you remove the front node from the linked list. To free memory, you return the front node to the linked list. This provides a constant allocation/releasing time and there is no fragmentation.

In the real world, programs like to allocate different sized chunks of memory so it's unlikely you can solely rely on this method.

However, it is theoretically possible for a microkernel to be designed so that all memory structures are exactly the same size (the Process struct, Thread struct, Message struct, etc). This would be very fast and efficient.

On this note, most Lisp implementations have a single 'box-and-pointer' base data type. A box-and-pointer type is a pair of values, usually pointer/pointer or atom/pointer (atom means a numeric value). On a 32-bit system, this structure is 8 bytes big. All other data structures (lists, trees, objects, etc) are built on top of this type. As a consequence memory allocation on a Lisp system is very fast.

==Tips to go further==

* Sometimes, and especially when you're working with objects, you have to allocate many objects that always have a certain size. It is wise to create pools of pre-divided large blocks for such objects.
* It's way easier to keep the size of allocated objects in a header hidden from the requester, so that a call to <tt>free</tt> doesn't require the object's size. Normally this hidden header is kept just before the block returned by <tt>malloc</tt>.
* It's way easier to design a memory allocator in a host OS than in your kernel. Also, if you implement the full malloc interface (<tt>malloc</tt>, <tt>calloc</tt>, <tt>realloc</tt> and <tt>free</tt> is enough on Linux) a good sanity test is to compile your <tt>malloc</tt> into a shared library, then compile something (like your whole host OS tree) with your malloc using LD_PRELOAD.
* Magic words like "F R E E" and "U S E D" will make your life easier when debugging. TimRobinson even allows 32 bits to store the address of the requester so that you can see "okay, this is a N-bytes block that was requested by <tt>MySillyFunction()</tt>, line 3405" ...

==Memory & Microkernels==

In a [[Microkernel|microkernel]] environment, there comes up a question: where the hell shall I put the memory management? In sense of heap management: give the kernel a dedicated allocator and a dedicated memory area to use - you might need two of them: one for the messages, and one for all the other stuff.

The other task of memory management: Process address space management, keeping track of used pages (yes, lets talk about paging, it is nice, it is neat, it causes a warm fuzzy feeling beneath the toes) and assigning memory to processes as needed, you can split it up. To be more precise, you have to split this task up - or keep every aspect of memory management in kernel land to make it easy. A recommended method for managing process address space: handle it in a per-process-manner. The actual process needs memory: allocate the memory and hand it out to the process. So you can keep the page allocating routines easy and straight forward. For this task of allocating/deallocating memory, you should take into consideration, that the task performing such actions should be able to slip into address spaces at needs (it loads the required page directory and does what it has to do - slimy little weasel thou' it is.) Take those things into good consideration and put quite an amount of brainwork into them. It's worth doing good engineering here.

==Porting an existing Memory Allocator==
It's not always desirable or practical to write your own memory allocator. Writing an efficient memory allocator can be an entire project in itself and fortunately it's extremely easy to port an existing memory allocator to your OS (to run in either kernel or user space). The advantages of using an existing memory allocator are; porting one is much faster than writing your own especially when you want to focus on other areas of your OS, it is likely to be well-tested so you do not have to debug the memory allocator, it takes a minimal amount of work to port it, and finally, someone else has done the hard work to make it fast, scalable, stable, etc.

Porting a memory allocator is fairly simple to do. Most of them are no more than one source and/or header file. The functionality you must hook is for allocating and freeing pages to your program, so the memory allocator has memory to work with.

There are a pair of hooks that some memory allocators use which allow the allocator to request memory from the kernel in terms of 'pages'. These memory allocators will have a constant stored somewhere in the source on how many large a page is (e.g. '#define PAGE_SIZE 4096' for 4KB pages) so the allocator knows how many pages to request at a time. With these allocators you must hook/implement:

*void *alloc_page(size_t pages) - Allocates 'pages' consecutive pages in virtual memory and returns a pointer to the beginning of the group.
*void free_page(void *start, size_t pages) - Frees 'pages' consecutive pages in virtual memory from 'start' back to the kernel.

In addition, some memory allocators will require you to hook locking functionality to ensure critical sections of the memory allocator aren't executed simultaneously by multiple threads. Typically the functions will appear like;

*void wait_and_lock_mutex() - Locks a mutex before entering a critical section. The simplest 'lock' solution is to disable interrupts which may be suitable as a starting point. For best performance, it is recommended to implement a spin lock.
*void unlock_mutex() - Unlocks the mutex after leaving a critical section. You either enable interrupts again or reset the spin lock. This allows any waiting threads to enter the critical section.

===Choosing a Memory Allocator===
There are many memory allocators to choose from. There is no perfect memory allocator because there are many different goals different allocators try to achieve. Usually these are conflicting goals, so different allocators have different trade-offs.

Some allocators are;

*..fast. They can perform the most allocations and releases per second. 'Fast' is context sensitive, and an allocator that is fast when in some circumstances (allocating lots of large chunks) may not be in other circumstances (allocating and releasing lots of small chunks).

*..space efficient. While other allocators may be aligning memory to page boundaries or have huge internal structures taking up megabytes, these allocators make sure all memory is packed into it's tightest fitting area and no byte is left to waste. This is especially important if you do not have much RAM.

*..stable. The other allocators may be fast, but these allocators are designed to run for a very long time. They focus on minimising memory fragmentation, which is important for a server that runs for months on end.

*..scalable. There are other allocators that are faster when allocating from a single thread but other threads must lock and wait for their turn. Scalable allocators can handle allocations from hundreds of threads on the latest quad-core CPUs concurrently with no significant performance penalty and minimal locking.

*..real-time. There may be a fast allocator that on 'average' takes 75 cycles to allocate a chunk, but has the occasional worse case situation where it'll take 350 cycles. A real-time allocator might guarantee to return a memory pointer in less than 200 cycles. This is desirable in media decoding and real-time systems.

There are a lot of allocators to choose from, so this is far from being a comprehensive list;
*[https://github.com/blanham/liballoc/ liballoc] - Excellent allocator that was originally a part of the Spoon Operating System and designed to be plugged into hobby OS's. 
*[http://gee.cs.oswego.edu/ dlmalloc] - Doug Lea's Memory Allocator. A good all purpose memory allocator that is widely used and ported.
*[http://goog-perftools.sourceforge.net/doc/tcmalloc.html TCMalloc] Thread-Caching Malloc. An experimental scalable allocator.
*[http://www.nedprod.com/programs/portable/nedmalloc/ nedmalloc] A very fast and very scalable allocator. These two properties have made it somewhat popular in multi-threaded video games as an alternative to the default provided allocator.
*[http://www.malloc.de/en/ ptmalloc] A widely used memory allocator included with glibc that scales reasonably while being space efficient.
*[https://github.com/jemalloc/jemalloc jemalloc] A general purpose malloc(3) implementation that emphasizes fragmentation avoidance and scalable concurrency support, first used in FreeBSD
*[https://github.com/emeryberger/Hoard Hoard] is a drop-in replacement for malloc that can dramatically improve application performance, especially for multithreaded programs running on multiprocessors and multicore CPUs.
*[https://github.com/fysnet/FYSOS/tree/master/bucket Bucket] is a simple drop-in replacement for malloc for beginners.  Most importantly, it has detailed documentation of what happens under the hood.  Not just source code comments.

==See Also==
===Tutorials===
*[[Writing a memory manager]]

===External Links===
*[http://www.osdever.net/tutorials/view/memory-management-1 Memory Management 1] - Part one of a two part series on memory management by Tim Robinson
*[http://www.osdever.net/tutorials/view/memory-management-2 Memory Management 2] - Part two of a two part series on memory management by Tim Robinson
*[http://www.cs.ucsb.edu/~grze/papers/Keyword/MEMORY-MANAGEMENT.html Publications about 'Memory Management'] - A list of some nice articles
*[http://rtportal.upv.es/rtmalloc/ TLSF: Memory Allocator for Real-Time] General purpose dynamic memory allocator specifically designed to meet real-time requirements
[[Category:Memory management]]
````

<a id="w001461"></a>

## W001461 — Page Frame Allocation

Bron-ID: W001461
Namespace: Main (0)
Revisie: 30500 | Datum: 2026-04-20T16:42:28Z
Bron: https://wiki.osdev.org/Page_Frame_Allocation
Revisie-URL: https://wiki.osdev.org/index.php?title=Page_Frame_Allocation&oldid=30500

````text
==Physical Memory Allocators==

These are the algorithms that will provide you with a new page frame when you need it. The client of this algorithm is usually indifferent to which frame is returned, and especially, a request for n frames doesn't need to return contiguous frames (unless you are allocating memory for DMA operations like network packet buffers).

N will be the size of the memory in pages in the following text.
===Bitmap===

A large array of N/8 bytes is used as a large bit map of the memory usage (that is, bit #i in byte #n define the status of page #n*8+i). Setting the state of a given page is fast (O(1)), allocating a page may take time (O(N)).

* an uint32_t comparison can test up to 32 bits at once and thus speed up allocations
* keeping a pointer to the last allocated bit may improve the performance of the next search (keeping information about the fact all the previous bytes were searched unsuccessfully)

===Stack/List of pages===

The address of each available physical frame is stored in a [[stack]]-like dynamic structure. Allocating a page is fast (O(1)), freeing a page too but checking the state of a page is not practical, unless additional metadata is sorted by physical address.

===Sized Portion Scheme===

You split each area of, say 16kb into (for example) chunks of 1 8kb, and 2 4kb's. Then you hand out each chunk. By doing this you can find closer fits to exact sizes. That means less waste. So say that you have an area of 32kb

[[Image:Sized Portion Scheme.png]]

You can even have 1, 2, even 3 or 4 (or more!) types of layouts for each portion. This way you have even more sizes to choose from.

===Buddy Allocation System===
This is the physical memory allocator of Linux kernel. Note that linux has several buddies depending on whether the memory is suitable for ISA DMA, or is coming from 'high physical memory' or just 'normal'. Each buddy contains k bitmaps, each indicating the availability of 2^i-sized and 2^i aligned blocks of free pages. Usually, linux uses from 4K to 512K blocks.

                  0   4   8   12  16  20  24  28  32  36
                  ###.#....#........#...###...########.... real memory pattern
 
    buddy[0]--->  ###.#.xx.#xxxxxxxx#.xx###.xx########xxxx 5  free 4K , 5-byte bitmap
    buddy[1]--->  # # # . # . x x . # . # # . # # # # x x  5  free 8K , 20-bits map
    buddy[2]--->  #   #   #   .   #   #   #   #   #   .    2  free 16K, 10-bits map
    buddy[3]--->  #       #       #       #       #        0  free 32K, 5-bits map

A buddy for N pages is about twice the size of a bitmap for the same area, but it allows a faster location of collections of pages. The figure above shows a 4-buddy with free pages/blocks denoted as . and used pages/blocks denoted as #. When a block contains at least one used sub-block, it is itself marked as used. Sub-blocks that are part of a larger block are also marked as used (x in the figure). Say we want to allocate a 12-K region on this buddy, we'll look up the bitmap of free 16K blocks (which says we have one such starting at page #12 and another starting at page #36). buddy[2]->bit[4] is then set to 'used'. Now we only want 3 pages out of the 4 we got, so the remaining page is returned to the appropriated buddy bitmap (e.g. the single pages map). The resulting buddy is

                  0   4   8   12  16  20  24  28  32  36
                  ###.#....#..###...#...###...########.... real memory pattern
 
    buddy[0]--->  ###.#.xx.#xx###.xx#.xx###.xx########xxxx 6  free 4K , 5-byte bitmap
    buddy[1]--->  # # # . # . # # . # . # # . # # # # x x  5  free 8K , 20-bits map
    buddy[2]--->  #   #   #   #   #   #   #   #   #   .    1  free 16K, 10-bits map
    buddy[3]--->  #       #       #       #       #        0  free 32K, 5-bits map

Note that initially, only the largest regions are available, so if buddy[0] is apparently empty, we need to check buddy[1], then buddy[2] etc. for a free block to be split.

===Hybrid scheme===

Allocators may be chained so that (for instance) a [[stack]] only covers the last operations and that the 'bottom' of the stack is committed to a bitmap (for compact storage). If the stack lacks pages, it can scan the bitmap to find some (possibly in a background job).

===Hybrid scheme #2===

Instead of keeping track of just bits representing pages, or just page numbers on a [[stack]], use a big array of structs to represent the memory. In these page frame structs, store a single link to a next page (pointer-sized) and a 8-16 bit information block indicating its status. Also include the virtual page pointer and the TCB to which the page number belongs. Keep pointers to each type of page, to both the start and the end of their lists. This way, you can easily display information about their content, the amount of pages for each type available, mix types, allow dynamic cleaning threads, do copy-on-write fairly easily and keep clear & concise overviews of the pages. It functions as a reverse page mapping table that lists types of pages too.

For an example implementation see AtlantisOS 0.0.2 or higher.

==Virtual Addresses Allocator==
===Flat List===

One straightforward way to manage big areas of addresses space is a linked-list (as depicted below). Each "free" region is associated with a descriptor giving its size and its base address. When some space needs to be allocated, the list is scanned for a region being large enough with a "first fit" or "best fit" (or whatever) algorithm. This was e.g. the way memory was managed by MS-DOS. When memory is allocated, the suitable entry is either removed (the whole region is allocated) or resized (only a portion of the region is allocated).

Note that with flat linked-lists, both "is memory at address XXX free" or "where can i get a block of size YYY" questions may require a complete traversal of the list to get answered. If virtual memory gets fragmented and the list gets longer, that may become an issue. "Is memory at address XXX free?" is mainly used to merge two free zone into a new (bigger) one when a block is released, and it is easier to deal with if the list is kept ordered by growing addresses.

[[Image:Flat_list.png]]

===Tree-based approach===

Since it is frequent that the list is searched for a given address or a given size, it may be interesting to use more efficient data structures. One of them that still keeps the ability of traversing the whole list is the AVL Tree. Each "node" in the AVL tree will describe a memory region and has pointer to the subtree of lower nodes and to the subtree of higher nodes.

[[Image:Tree based.png|none]]

While insertion/deletion in such a balanced tree requires more complex operations than linked list manipulation, searching the tree is usually achieved with O(log2(N)) rather than O(N) for linked lists -- that is, if you have 1000 entries, it requires 1000 iterations to scan the list against 10 iterations to find a given interval in the tree.

Linux has used AVL trees for virtual addresses management for quite a while. Note however that it uses it for regions (like what you find in /proc/xxxx/maps), not for a malloc-like interface.

==See Also==
===Articles===
* [[Writing A Page Frame Allocator]]
* [[Memory Allocation]]
* [[Memory management]]
* [[Writing a memory manager]] - a tutorial

===Threads===
* [[Topic:11320|Allocating memory for an allocator without an allocator]]
* [[Topic:9450|A bitmap based allocation technique]]
* [[Topic:8489|Ways to keep track of allocated RAM]]
* [[Topic:8463|Questions about Memory Allocation]]
* [[Topic:8387|Memory Management]]
* [[Topic:8325|Memory Management to the X'th]]
* [[Topic:9036|MM Questions]]
* [[Topic:8741|(about)Tim Robinson Memory Management Tutorial #1]]
* [[Topic:9781|Managing used/free pages]]
* [[Topic:10279|Malloc, etc. (tute by curufir)]]
* [[Topic:10747|Physical MM (by Freanan)]]
* [[Post:195116|Concepts and key points on alternative memory management schemes]]

===External Links===
*mystran's [http://replay.web.archive.org/20081206102136/http://www.cs.hut.fi/~tvoipio/memtutor.html Basic VMM for Dummies (cached)]
* [[wikipedia:Page replacement algorithm|Page replacement algorithm]] on Wikipedia

[[Category:Common_Algorithms]]
[[Category:Memory management]]
[[Category:Physical Memory]]
[[de:Physische Speicherverwaltung]]
````

<a id="w001462"></a>

## W001462 — Talk:Storage

Bron-ID: W001462
Namespace: Talk (1)
Revisie: 5057 | Datum: 2008-01-22T07:49:35Z
Bron: https://wiki.osdev.org/Talk:Storage
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AStorage&oldid=5057

````text
== IDE or ATA ==
IDE is the wrong term, should be ATA. However, everybody uses it. -[[User:Combuster|Combuster]] 07:16, 2 December 2006 (CST)

== USB ==

Shouldn't we also have USB listed here as a storage interface? Since we have Floppy Controller listed twice, too? [[User:Bewing|Bewing]] 19:29, 21 January 2008 (CST)

Not really. USB is just a specification of sending data across. You practically tunnel something else (like scsi commmands) over an usb cable. ([http://www.usb.org/developers/devclass_docs/usb_msc_overview_1.2.pdf doc]). IMHO if you add it, put it under the driver header. - [[User:Combuster|Combuster]] 01:49, 22 January 2008 (CST)
````

<a id="w001463"></a>

## W001463 — Talk:Kernels

Bron-ID: W001463
Namespace: Talk (1)
Revisie: 2666 | Datum: 2007-01-03T09:21:14Z
Bron: https://wiki.osdev.org/Talk:Kernels
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AKernels&oldid=2666

````text
== Kernel classification ==
I think this kernel issue needs some thought: [[topic:12260|This thread]] has turned into a discussion whether modular kernels are the same as monolithic. (some say they are different) IMO you have three axis in os design: 
* user space drivers / kernel mode
* single binary / separate modules
* minimal abstraction / full abstraction
where monolithic = kernelmode + single binary, modular kernel = kernelmode + modules, microkernel = usermode + modules and exokernel = anything with minimal abstraction.

Can we reach consensus about this or should this to be moved to the forum for open debate?

In the meantime i'll add the exokernel to complete the list according to the "old" model, as the notion doesnt depend on the micro/macro/modular/monolith issue.

-[[User:Combuster|Combuster]] 08:15, 2 December 2006 (CST)

==Category==
Should this page be a category? Seems like it would fit better with the rest of the wiki. -[[User:Jhawthorn|Jhawthorn]] 01:18, 3 January 2007 (CST)

I think leaving it, or renaming it to [[Kernel Architecture]] (or comparison) would be more useful, because kernels have several aspects, and its easier on the user (read: n00b) to have a dedicated block where he can quickly look up the basic differences without having to read each individual page. - [[User:Combuster|Combuster]] 03:21, 3 January 2007 (CST)
````

<a id="w001464"></a>

## W001464 — Talk:Floppy Disk Controller

Bron-ID: W001464
Namespace: Talk (1)
Revisie: 4095 | Datum: 2007-06-17T11:32:14Z
Bron: https://wiki.osdev.org/Talk:Floppy_Disk_Controller
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AFloppy_Disk_Controller&oldid=4095

````text
== Title ==
Should this page be renamed to [[Floppy Drive Controller]]? IMO it would be more appropriate as it discusses the FDC rather than floppies.

-[[User:Combuster|Combuster]] 08:32, 2 December 2006 (CST)


I think that would work. What if on the [[Storage]] page there was another section about programming each of these devices.
So this would be under the storage driver section and there could be additional pages about each device.
-[[User:Telexicon|Telexicon]] 17:54, 2 December 2006 (CST)
````

<a id="w001465"></a>

## W001465 — Exokernel

Bron-ID: W001465
Namespace: Main (0)
Revisie: 29667 | Datum: 2025-09-19T21:30:09Z
Bron: https://wiki.osdev.org/Exokernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Exokernel&oldid=29667

````text
{{Template:Kernel designs}}

Exokernels are an attempt to separate security from abstraction, making non-overrideable parts of the operating system do next to nothing but securely multiplex the hardware. The goal is to avoid forcing any particular abstraction upon applications, instead allowing them to use or implement whatever abstractions are best suited to their task without having to layer them on top of other abstractions which may impose limits or unnecessary overhead. This is done by moving abstractions into untrusted user-space libraries called "library operating systems" (libOSes), which are linked to applications and call the operating system on their behalf.

Exokernels are often compared to (or conflated with) hypervisors, and the implementations of both are usually quite similar, especially when hardware virtualization is used; most exokernels can in fact serve as a hypervisor for other OSes, if one chooses to. However, the use cases are radically different - the purpose of a hypervisor is ''providing'' an abstraction over the hardware, whereas the purpose of an exokernel is to make such an abstraction unnecessary.  Furthermore, in most current uses, the hypervisor itself is running as or in an instance of a general-purpose OS. While the move to 'containerized' clients for hypervisors can be seen as a hybrid of the two ideas (at least on a conceptual level), in practice they are not the same.

== Exokernel Concept ==

[[Image:exokernel.png|frame|right]]

The concept of an exokernel is orthogonal to that of micro- vs. monolithic kernels. It is not important whether the secure multiplexing is performed in privileged kernel code or usermode servers, just that there are no forced abstractions.

=== Example ===

Take the typical abstraction of a file. Files, as the application sees them, do not exist on disk. On the disk, there are disk sectors. The operating system abstracts the reality of the disk to create the more convenient illusion of files and a filesystem. Usually, security is also provided at this level. ACLs, Unix-style permissions, etc. are applied to files. Security is combined with the abstraction.

On exokernels, security is provided on the unabstracted hardware level, in this example, to disk sectors. LibOSes provide any desired abstractions on top of this interface. Non-overrideable security is put in the exokernel, and the overrideable abstraction is implemented in a libOS. Security is separated from abstraction.

== Advantages ==

The upside of this is that user-space applications are allowed to implement their own, optimized memory management (by directly accessing e.g. memory tables), file access (by "raw" disk access), etc. This can, for special applications, result in significant performance increases. Engler et al. benchmarked a web server, Cheetah, running on an exokernel being up to eight times faster than the competition. Cheetah, among other things, uses knowledge of HTTP to combine IO requests, knowledge of HTML to appropriately colocate resources on disk, avoids copying data by sending resources to clients directly from the file cache, and caches pregenerated network packets. See [http://pdos.csail.mit.edu/papers/exo-sosp97/exo-sosp97.html Application Performance and Flexibility on Exokernel Systems].

In addition, by being aware of resource availability, revocation and allocation, it is hoped that applications can make more efficient and intelligent use of hardware resources. For example, a server would know not to make its cache in memory larger than the amount of memory it actually has. In traditional systems, the operating system would fake however much the server had allocated with virtual memory, and the server would carry on, ignorant of any page faults, swapping, and paging.

It is also hoped that exokernels will, in a similar way to microkernels, ease development and testing of new operating system ideas. New scheduling techniques, memory management methods, filesystems, etc. can be developed and tested in libOSes, which are more quickly and easily modified than current operating systems. In [http://www.cs.biu.ac.il/~wiseman/2os/microkernels/exokernel.pdf The Exokernel Operating System Architecture], Engler recalls the story of an undergraduate who was able to develop and test a new page table structure on an exokernel in a week while the designers had only been able to simulate it.

== Disadvantages ==

Exokernel technology is still not thoroughly researched, so surprises might lurk everywhere. Moreover, the added flexibility for user-space programs also means reduced consistency. While it would be theoretically possible to provide libOSes that enable e.g. MacOS, Windows, and Linux applications to run simultaneously on the same system, that would also mean different look & feels for each of them. In addition, different libOSes may have varying levels of compatibility and interoperability with each other.

It can be difficult to design exokernel interfaces. The designer must develop adequate and appropriate interfaces to low level hardware and delicately balance power and minimalism vs. enough protection. Engler et al. remark that their exokernel interfaces went through many revisions.

The ease of creation and mixing of libOSes could lead to code messes that would be a nightmare for maintenance coders and system administrators. Maintenance coders would have to deal with not only the application code, but any overriden abstractions or new implementations.

Information that could otherwise be useful to a kernel can be lost if the related abstractions are implemented in libOSes rather than the kernel.

== Exokernel Derivates ==

While Monolithic Kernel and Microkernel are rather well-defined terms, the advocates of exokernel-like technology have coined many different terms - nanokernel, picokernel, cache kernel, virtualizing kernel etc. Most of these are relatively minor variations of each other.

=== Nanokernel/Picokernel ===

Nanokernels and picokernels are usually small kernels considered by their creators to be even smaller than [[Microkernel|microkernels]]. Examples include: [http://home.gna.org/adeos/ Adeos], [http://www.cis.upenn.edu/~KeyKOS/NanoKernel/NanoKernel.html KeyKOS], and [http://lseos.sourceforge.net/ LSE/OS]. Another very famous example is the symbian EKA2 kernel. This nanokernel implements drivers inside the kernel making it not fully a microkernel.

=== Cache Kernel ===

The [http://www-dsg.stanford.edu/papers/cachekernel/main.html Stanford cache kernel] caches kernel objects, like address spaces and threads, and allows usermode “application kernels” to manage them, loading and unloading them as needed. Application kernels manage their threads' page faults, exceptions, etc., and the cache kernel allows several of these application kernels to coexist in a single system.

=== Virtualizing Kernel ===

Virtualizing kernels are usually designed to allow multiple operating systems to run on a single computer, by allowing free execution of unprivileged instructions and trapping and simulating privileged instructions. Adeos, while it calls itself a nanokernel, is similar in concept to virtualizing kernels. Unlike exokernels, virtualizing kernels attempt to be as transparent as possible, to avoid requiring many modifications, if any, to the hosted operating systems.

== See Also ==

=== Papers ===

[https://web.archive.org/web/20210421070535/https://u.cs.biu.ac.il/~wiseman/2os/microkernels/exokernel.pdf The Exokernel Operating System Architecture]

[[Category:Kernel]]
[[Category:OS theory]]
````

<a id="w001467"></a>

## W001467 — ISA DMA

Bron-ID: W001467
Namespace: Main (0)
Revisie: 30347 | Datum: 2026-03-17T21:08:14Z
Bron: https://wiki.osdev.org/ISA_DMA
Revisie-URL: https://wiki.osdev.org/index.php?title=ISA_DMA&oldid=30347

````text
;The main points about ISA DMA are:
* ISA DMA is not the same thing as PCI Busmastering DMA;
* ISA DMA channels 1, 2 and 3 are available for 8 bit transfers to ISA peripherals;
* ISA DMA channels 5, 6 and 7 are available for 16 bit transfers to ISA peripherals;
* Transfers must not cross physical 64 KB boundaries and must never be bigger than 64 KB;
* Transfers must be physically contiguous, and can only target the lowest 16 MB of physical memory;
* ISA DMA is slow - theoretically 4.77 MB/second, but more like 400 KB/second due to ISA bus protocols;
* ISA DMA frees up CPU resources, but adds an extremely heavy load to the memory bus;
* Very few devices currently use ISA DMA -- only internal floppies, some embedded sound chips, some parallel ports, and some serial ports.

Notes:
* Sound Blaster and Sound Blaster PRO only support 8 bit DMA;
* [[Sound Blaster 16]]+ supports both;
* [[Floppy Disk Controller|Floppy disk controllers]] only support 8 bit DMA and are hardwired to use DMA Channel 2.


== There Is More Than One Kind of DMA on a PC ==
Modern PCI controllers always have their own 'Busmastering DMA', which is far better than ISA DMA. Even USB floppy drives send their DMA
data using PCI Busmastering, through the PCI USB controller.
PCI Busmasters can access memory with 32 bit addressing. Newer PCI cards are starting to support 64 bit addressing (although at the moment most
don't). Typically PCI cards use 'scatter-gather' bus mastering, where one page is used as a directory of data pages. This almost completely overcomes
the "physical memory only" limitation of all forms of DMA.


== ISA DMA Background ==
'''ISA DMA''' (''Industry Standard Architecture Direct Memory Access''), like ISA itself, is an [https://en.wikipedia.org/wiki/Appendix_(anatomy) appendix] for modern PCs. It is used by the
internal floppy disk controller, ISA sound cards, ISA network cards, and parallel ports (if they support ECP mode). Whilst interrupt, keyboard
and timer interface circuits have obvious and relevant uses, the ISA DMA controller and its programming interface are still well and truly stuck in the
1970s where they were first designed.

The idea behind DMA is that you can set up a 'channel' with an address pointing into memory and the length of the data to be transferred. Once set up, the
CPU can tell the peripheral owning the channel to do whatever it is supposed to do (e.g. read a sector). Then the CPU can go do something else.
When the memory bus isn't being used by the CPU, the DMA chip takes over and transfers data between the peripheral and memory without involving the CPU. When
the transfer is complete (e.g. an entire sector has been sent to the floppy drive) the DMA chip then signals that it is finished. The DMA chip can even
signal if it has run out of data, allowing the system to locate the next block of data to transfer on the same DMA transaction.
DMA can improve the speed of a system quite a bit and was borrowed by Intel (who designed the DMA controller chip) from the old 1960s mainframes which had
DMA channels for all devices (CPUs weren't all on a single chip and very slow back then).

Of course all good ideas can have downsides and while Intel can't really be blamed for what is about to be described, IBM certainly can.

In the beginning there was a PC, but the PC was slow. IBM looked down from the heavens and said "Slap on a DMA controller -- that should speed it up." IBM's
heart was in the right place; its collective brains were elsewhere as the DMA controller never met the needs of the system. The PC/AT standard contains 2
Intel 8237A DMA chips, connected as Master/Slave. The second chip is Master, and its first line (Channel 4) is used by the first chip, which is Slave. (This
is unlike the interrupt controller, where the first chip is Master.) The 8237A was designed for the old 8080 8-bit processor and this is probably the main
reason for so many DMA problems. The 8088 and 8086 processors chosen by IBM for its PC were too advanced for the DMA controller.

Previously it was mentioned that a DMA controller is able to signal completion and even ask for more information. Unfortunately this would make expansion
slots too big, so IBM left all of the connections to the DMA chips off. The only time you know when a transfer is complete is for a peripheral to signal an
interrupt. This implies that all peripherals using an ISA DMA channel are limited to no more than 64 KB transfers for fear of upsetting the DMA controller.

Even with the PC/AT, IBM began bypassing the ISA DMA used in the PC/XT and used [[ATA PIO Mode]] for the hard disk. This
was because of the 64 KB limitations outlined above and the fact that the 286 processor could perform 16 bit transactions at 6 MHz. Even the ISA bus could
run at a speed of up to 12 MHz, far faster than the 4.77 MHz the DMA controller was running at.

Expansion card designers were also upset with DMA's lack of capabilities, noticeably 'Hard-Card' hard disk expansion card manufacturers who depended on the
speed of data transfer.

To get around the limitations of the 'on board' DMA controller, expansion card manufacturers began to put their own DMA controllers on their expansion
cards. They functioned exactly the same way as the 'on board' DMA, 'stealing memory bus cycles' when the processor wasn't looking and thus improving the
performance of the system as a whole. These "ISA Bus Masters" are still usually limited to the lower 16 MiB of memory, but do not have the 4.77 MHz issue.
This trend continued through the creation of the PCI bus, which eventually entirely replaced the ISA bus in PCs.

== Technical Details ==
Each 8237 DMA chip has 4 DMA channels. DMA0-DMA3 on the first chip and DMA4-DMA7 on the second. The first DMA controller is wired up for 8 bit transfers,
while the second is wired up for 16 bit transfers. On some tutorials or other wiki articles, you will sometimes see the '''second''' DMA chip (channels 4 to 7) labeled as the "'''master'''" controller, and the first (channels 0 to 3) called the "slave". This is highly confusing, and these terms will not be used again, here.

DMA Channel 0 is unavailable as it was used for a short time for DRAM memory refresh, and remains reserved because of this (even though modern computers
don't use it). DMA channel 4 cannot be used for peripherals because it is used for cascading the other DMA controller.

The internal address registers of the DMA controller are only 16 bits. In order to extend this, IBM added one ''external'' "page register" byte per channel,
allowing access to 16 MB of memory (24 bits total). If a DMA transfer crosses a 64 KB boundary, the internal address register wraps around to zero,
and the external page register is '''not''' incremented. The DMA controller will happily continue the transfer with whatever data it finds at
the new address.

ISA-based DMA controllers are specified to run at 4.77 MHz. '''No exceptions'''. If the "front-side" (memory) bus of a system runs at 133 MHz, it will be
artificially slowed down to 4.77 MHz when transferring each ISA DMA byte/word. This includes EISA and PS/2 32-bit
controllers, even though these controllers have an extra page register (which allows a 4 GiB addressing space)
and the ability to do 32 bit transfers. (These DMA controllers exist only on EISA and MCA systems, which are now obsolete and are not further
described here.)


== Programming Details ==

=== 16 bit issues ===
The 16 bit channels (5, 6, and 7) have a special addressing scheme to handle the way they increment addresses. The internal registers increment by 1,
but the memory address needs to be incremented by 2 between each access. The solution is that the "start address" that the CPU stores in the DMA controller
needs to be shifted right by 1 bit. On each memory access, this internal 16 bit address value is incremented by one, then that value is taken and shifted
one bit to the left (clearing the lower bit) before being used as an address. The external "page register" addressing byte is then appended in the normal
way. It is important to note that the upper bit of the internal address is '''lost''' when it is shifted left -- it is not ORed into the page register byte.
This prevents DMA transfers bigger than 64 KB from working even though they should be technically possible for 16 bit channels, because the address register
will effectively wrap around to zero without incrementing the page register byte.

=== Physical Memory vs. Paging ===
Paged memory mapping is exclusively controlled by the CPU. The whole point of DMA is to bypass the CPU. Therefore, no DMA can ever access any virtual
memory addresses. All DMA is always done on physical memory addresses only. ISA DMA has a 16 MB physical address limit.

Since DMA runs independently of the CPU, it is important that an OS allocates a block of contiguous physical memory for the DMA transfer in
a way that prevents that memory from being used for any other purpose, or swapped out, until the DMA transfer is complete.

Note: VM86 mode does not use ''physical'' addressing. The memory addresses are ''fake''. In VM86 mode the OS must emulate any DMA transactions on
behalf of an application.

=== Buffer Size ===
A typical 1.44 MB floppy disk can easily transfer 36 sectors of data in a single transfer. This is only 18 KB. The biggest internal floppy with
worst-case formatting may be able to transfer 84 sectors at once. This is still only 42 KB. A Soundblaster card may run best with a 64 KB buffer. It
is never necessary to try to double-buffer ISA DMA transfers, because they are so slow anyway. There are at most 6 usable DMA channels, and it is not
necessary to allocate a full 64 KB to each of them. Putting all of this together, any OS should be easily able to allocate all the ISA DMA physical
memory that it needs from a 256 KB pool, or even only half of that.

=== The Flip-Flop ===
Many devices on a PC (e.g. ATA disk drives) use 8 bit IO Ports to receive 16 bit values. This is done using a flip-flop. The device expects the low
byte first. As soon as it receives a byte, the flip-flop changes state and then the device expects the high byte. When the high byte is received,
the flip-flop changes state again, and the device expects a new low byte. Usually, each 16 bit 'register' will have its own flip-flop, but the ISA
DMA controller has a problem regarding this.

On a 8237 chip, there is '''only one''' flip-flop. And there are eight of the 16 bit registers. And there can be up to three device drivers all competing
for the use of that one flip-flop simultaneously.

This creates two serious problems. One is "contention issues". The other is that it is difficult to be sure what state the flip-flop is currently in.
The standard solution for dealing with the flip-flop state issue is to reset the flip-flop to "low byte" state every single time you want to use it, just
so you can be certain it is in the proper state before sending bytes. There are only two solutions to "contention": either use a [[lock]], or allow only
one ISA DMA driver, so that contention is impossible.

=== Masking DRQ ===
Setting up a DMA transfer always requires setting up both "ends" of the transfer. That is, whichever peripheral owns the DMA channel needs to be told
to transfer a block of data via DMA. And the DMA controller needs to be told the memory address, the transfer length, perhaps a transfer "mode", and
a transfer direction (read or write). So, one of these two things always needs to be done ''first'' -- and it is usually the peripheral that has a long
latency time before being ready to transfer the first byte. If you set up the peripheral first, however, its first DMA request signal (often called
DRQ) may arrive while you are in the middle of setting up the DMA controller.

The answer is to mask DRQ for a particular channel while you are initializing the DMA controller. There are three ways of temporarily disabling channels
described below.

=== Transfer Length ===
The value that gets stored into each Count Register is always the transfer length (either bytes or words) '''minus 1'''. If you forget to subtract
the one, you will get an error on your transfer.

=== Interrupt on Completion ===
As implemented in a PC, the DMA controller can not send interrupts. Hopefully, whichever peripheral "owns" each DMA channel will send the CPU an
interrupt when a transfer completes. However, some peripherals may '''not''' send an interrupt if a transfer fails with an error. As always, timeouts
are important.

=== The Registers ===
The master and slave DMA controllers are very similar, so (to save space) both of them have been combined into the following table. Please try
not to let this confuse you.
Note: for Address and Count Registers on channels 5 to 7, see [[#16 bit issues|16 bit issues]] above.

Each 8237A has 18 registers, addressed via the I/O Port bus:

{| {{wikitable}}
|-
!Channels 0-3
!Channels 4-7
|
|
|
|-
!IO Port
!IO Port
!Size
!Read or Write
!Function
|-
|0x00
|0xC0
|Word
|W
|Start Address Register channel 0/4 (unusable)
|-
|0x01
|0xC2
|Word
|W
|Count Register channel 0/4 (unusable)
|-
|0x02
|0xC4
|Word
|W
|Start Address Register channel 1/5
|-
|0x03
|0xC6
|Word
|W
|Count Register channel 1/5
|-
|0x04
|0xC8
|Word
|W
|Start Address Register channel 2/6
|-
|0x05
|0xCA
|Word
|W
|Count Register channel 2/6
|-
|0x06
|0xCC
|Word
|W
|Start Address Register channel 3/7
|-
|0x07
|0xCE
|Word
|W
|Count Register channel 3/7
|-
|0x08
|0xD0
|Byte
|R
|Status Register
|-
|0x08
|0xD0
|Byte
|W
|Command Register
|-
|0x09
|0xD2
|Byte
|W
|Request Register
|-
|0x0A
|0xD4
|Byte
|W
|Single Channel Mask Register
|-
|0x0B
|0xD6
|Byte
|W
|Mode Register
|-
|0x0C
|0xD8
|Byte
|W
|Flip-Flop Reset Register
|-
|0x0D
|0xDA
|Byte
|R
|Intermediate Register
|-
|0x0D
|0xDA
|Byte
|W
|Master Reset Register
|-
|0x0E
|0xDC
|Byte
|W
|Mask Reset Register
|-
|0x0F
|0xDE
|Byte
|RW
|MultiChannel Mask Register (reading is undocumented, but it works!)
|-
|}

Each Channel also has an external R/W Page Address Register that contains the upper 8 bits of the 24 bit transfer memory address:

{| {{wikitable}}
|-
|0x87
|Channel 0 Page Address Register (unusable)
|-
|0x83
|Channel 1 Page Address Register
|-
|0x81
|Channel 2 Page Address Register
|-
|0x82
|Channel 3 Page Address Register
|-
|0x8F
|Channel 4 Page Address Register (unusable)
|-
|0x8B
|Channel 5 Page Address Register
|-
|0x89
|Channel 6 Page Address Register
|-
|0x8A
|Channel 7 Page Address Register
|-
|}

==== Useful Registers ====

;Single Channel Mask Registers 0x0A and 0xD4 (Write)
{| {{wikitable}}
|-
|Bit 7
|Bit 6
|Bit 5
|Bit 4
|Bit 3
|Bit 2
|Bit 1
|Bit 0
|-
|
|
|
|
|
|MASK_ON
|SEL 1
|SEL 0
|}


These registers are used to mask (or unmask) DRQ for a single channel only, on either the master or slave DMA chip.
That is, if you do not want to figure out the mask states of all the other channels, you can mask/unmask DRQ for one channel at a time.
Use the SEL 0 and 1 bits to select the channel, and the MASK_ON bit to set or clear masking for it.
Note that masking DMA channel 4 will mask 7, 6, 5 and 4 due to cascading.


;MultiChannel Mask Registers 0x0F and 0xDE (Read and Write)
{| {{wikitable}}
|-
|Bit 7
|Bit 6
|Bit 5
|Bit 4
|Bit 3
|Bit 2
|Bit 1
|Bit 0
|-
|
|
|
|
|MASK3
|MASK2
|MASK1
|MASK0
|}

Setting the appropriate bits to 0 or 1 allows you to unmask or mask (respectively) DRQ for those channels. Using this register means that your driver needs
to know the desired mask states of ''all'' the channels at that moment. There are several ways to do this, but one is simply to read this register, first.
Note that masking DMA channel 4 will mask 7, 6, 5 and 4 due to cascading.


;DMA Mode Registers 0x0B and 0xD6 (Write)
{| {{wikitable}}
|-
|Bit 7
|Bit 6
|Bit 5
|Bit 4
|Bit 3
|Bit 2
|Bit 1
|Bit 0
|-
|MOD1
|MOD0
|DOWN
|AUTO
|TRA1
|TRA0
|SEL1
|SEL0
|}

Setting this register is a little tricky as it depends highly on the peripheral you are programming the DMA controller for. However, the
driver for the peripheral is the entity that needs to set this register, and it should know what mode the peripheral needs.

* '''SEL0''' and '''SEL1''' select the channel you want to change;
* '''TRA0''' and '''TRA1''' selects the transfer type;
** 0b00 runs a self test of the controller;
** 0b01 Peripheral is writing to memory;
** 0b10 Peripheral is reading from memory;
** 0b11 invalid.
* '''AUTO''': When this bit is set, after a transfer has completed the channel resets itself to the address and count values you programmed into it. This is great for floppy transfers. Read in a track - the values set themselves up for reading again immediately. For writing you'd only need to alter the transfer mode - not the addresses. Some expansion cards do not support auto-init DMA such as Sound Blaster 1.x. These devices will crash if used with auto-init DMA. Sound Blaster 2.0 and later do support auto-init DMA.
* '''DOWN''': Reverses the memory order of the data, when set. Memory is accessed from high addresses down to low addresses (the address is decremented between each transfer).
* '''MOD0''' and '''MOD1''': This is where some problems can arise based on the peripheral the DMA controller is attached to. The DMA controller has several modes:
** 0b00 = Transfer on Demand;
** 0b01 = Single DMA Transfer;
** 0b10 = Block DMA Transfer;
** 0b11 = Cascade Mode (use to cascade another DMA controller).

Single transfer mode is good for peripherals than cannot cache a lot of data at once.
Non-82077AA Floppy controllers, Sound Blaster, and Sound Blaster Pro should use Single Transfer DMA Mode.

Block transfer mode is good for peripherals that can buffer entire blocks of information. An example of this is a hard disk controller board.

Demand transfer mode is good for peripherals that start and stop intermittently such as a tape drive. The drive can read a whole load of information for as
long as it can and the suspend the transfer to move to another section of the tape. Newer floppy controllers also work well with demand transfer because
they have FIFO buffers to store information being read and written (but you need to set up the FIFO properly). The peripheral controls the flow, and as
the information flow is uninterrupted, performance can be gained. CPUs in these later computers generally have caches and can continue working
uninterrupted during a demand DMA transfer. Older computers will slow down as their CPUs wait for the memory bus to become available.

;Flip-Flop Reset Registers 0x0C and 0xD8 (Write)
;Master Reset Registers 0x0D and 0xDA (Write)
;Mask Reset Registers 0x0E and 0xDC (Write)

Send any value to the Reset registers to activate them.
Master Reset sets Flip-Flop low, clears Status, and sets all Mask bits ON.
Mask Reset sets all Mask bits OFF.

The following statement from the previous wiki article needs to be verified on real hardware, because it is likely to be wrong: "The Reset Flip-Flop command
must be sent ''before any'' 16 bit transaction. The flip-flop ''does not reset'' after the DMA controller has received the second byte."


==== The Other Registers ====

;Status Registers 0x08 and 0xD0	(Read)
{| {{wikitable}}
|-
|Bit 7
|Bit 6
|Bit 5
|Bit 4
|Bit 3
|Bit 2
|Bit 1
|Bit 0
|-
|REQ3
|REQ2
|REQ1
|REQ0
|TC3
|TC2
|TC1
|TC0
|}

* REQ3-0: When set: DMA Request Pending.
* TC3-0: When set: Transfer Complete.
* Reading this register will clear the TC bits.

This register isn't very important in light of the fact that the 8237 can't send an IRQ to tell you that it has finished. Usually there is no need to poll
this register as the peripheral (at the other end of the DMA) will send an interrupt when a transaction has completed.


;Command Registers 0x08 and 0xD0 (Write)
{| {{wikitable}}
|-
|Bit 7
|Bit 6
|Bit 5
|Bit 4
|Bit 3
|Bit 2
|Bit 1
|Bit 0
|-
|DACKP
|DRQP
|EXTW
|PRIO
|COMP
|COND
|ADHE
|MMT
|}

This register really shows how incompatible the 8237 is with the PC hardware.
* Let's start with EXTW and COMP. These increase the speed of DMA transfer by 25% be removing one of the clock cycles. Does it work? No.
* PRIO. When zeroed, this allows DMA priorities to be rotated allowing freedom and liberty for all peripherals that share the data bus. Does it work? No.
* MMT and ADHE. Did you know that the IBM PC could do memory to memory transfers since 1981? That's right, hardware sprites, hardware frame buffering from one location to another. Does it work? No.
* COND. Hooray the only bit in the control register that does something useful. Setting this bit disables the DMA controller. This is one way to set up multiple DMA channels without masking each and every channel.

;Request Registers 0x09 and 0xD2 (Write)
Used for memory to memory transfers and setting up priority rotation -- absolutely useless.

;"Intermediate" Registers 0x0D and 0xDA (Read)
Never implemented on PCs. Useless.

== Examples ==

=== Floppy Disk DMA Initialization ===

You need only implement 1 to 3 tiny routines to perform a DMA transfer. This example is the Floppy Drive controller (probably the most common followed by SoundBlaster).

Note: the following code is not optimal, because there is an OUT to the same IO port twice (in two places). This causes an extra delay on the IO Port
bus. Real code should separate the two "out 0x4" and "out 0x5" calls with an "out" to some other port.

<syntaxhighlight lang="asm">
initialize_floppy_DMA:
; set DMA channel 2 to transfer data from 0x1000 - 0x33ff in memory
; paging must map this _physical_ memory elsewhere and _pin_ it from paging to disk!
; set the counter to 0x23ff, the length of a track on a 1.44 MiB floppy - 1 (assuming 512 byte sectors)
; transfer length = counter + 1
    out 0x0a, 0x06      ; mask DMA channel 2 and 0 (assuming 0 is already masked)
    out 0x0c, 0xFF      ; reset the master flip-flop
    out 0x04, 0         ; address to 0 (low byte)
    out 0x04, 0x10      ; address to 0x10 (high byte)
    out 0x0c, 0xFF      ; reset the master flip-flop (again!!!)
    out 0x05, 0xFF      ; count to 0x23ff (low byte)
    out 0x05, 0x23      ; count to 0x23ff (high byte),
    out 0x81, 0         ; external page register to 0 for total address of 00 10 00
    out 0x0a, 0x02      ; unmask DMA channel 2
    ret
</syntaxhighlight>

Once you have set up your start address and transfer length you do not need to touch it again, if you are using autoinit. Once reading or writing
is selected, you don't need to change that, either. To ''change'' selecting reading or writing you use the mode register.

<syntaxhighlight lang="asm">
prepare_for_floppy_DMA_write:
    out 0x0a, 0x06      ; mask DMA channel 2 and 0 (assuming 0 is already masked)
    out 0x0b, 0x5A      ; 01011010
                        ; single transfer, address increment, autoinit, write, channel2)
    out 0x0a, 0x02      ; unmask DMA channel 2
    ret

prepare_for_floppy_DMA_read:
    out 0x0a, 0x06      ; mask DMA channel 2 and 0 (assuming 0 is already masked)
    out 0x0b, 0x56      ; 01010110
                        ; single transfer, address increment, autoinit, read, channel2)
    out 0x0a, 0x02      ; unmask DMA channel 2
    ret
</syntaxhighlight>

Some hardware, as well as VirtualPC do not support autoinit. You may want to set the Mode registers to 0x4A and 0x46 in the above routines, instead.

The above routines use single transfer mode for compatibility, but during the initialization of your floppy driver if you detect an "advanced" floppy
controller (using the Version command), "demand transfer" should be used to reduce overhead.

== References ==

=== Articles ===
* [[Floppy Disk Controller]]
* [[Sound Blaster 16]]

=== External Links ===
* [http://www.intel-assembler.it/PORTALE/4/231466_8237A_DMA.pdf Intel 8237A datasheet]
*[http://bos.asmhackers.net/docs/dma/docs/ http://bos.asmhackers.net/docs/dma/docs/]

[[Category:Storage]]
[[de:DMA]]
````

<a id="w001469"></a>

## W001469 — Memory management

Bron-ID: W001469
Namespace: Main (0)
Revisie: 29805 | Datum: 2025-12-26T12:33:42Z
Bron: https://wiki.osdev.org/Memory_management
Revisie-URL: https://wiki.osdev.org/index.php?title=Memory_management&oldid=29805

````text
'''Memory management''' is a critical part of any operating system [[Kernels|kernel]]. Providing a quick way for programs to allocate and free memory on a regular basis is a major responsibility of the kernel. There are many implementations for [[Page_Frame_Allocation|allocating physical memory]] including bitmaps,  buddy allocation and using tree structures or queues/stacks.

''For an overview of memory allocation models, and methods of allocating memory, see [[Program Memory Allocation Types]]. If you are looking for heap type memory management, which is the allocation of smaller chunks of memory not on large boundaries then see the [[Heap|Heap]] page. A heap is commonly implemented (in the popular way of thinking) not only in the kernel, but also in applications - in the form of a standard library. For a discussion of automatic memory management methods, see [[Garbage Collection]].''

==Address Spaces==
Many platforms, including x86, use a memory management unit ([[MMU]]) to handle translation between the virtual and physical address spaces. Some architectures have the MMU built-in, while others have a separate chip. Having multiple address spaces allows each task to have its own memory space to work in. In modern systems this is a major part of memory protection. Keeping processes' memory spaces separate allows them to run without causing problems in another process's memory space.

===Physical Address Space===
The physical address space is the direct memory address used to access a real location in RAM. The addresses used in this space are the bit patterns used to identify a memory location on the address bus.

In this memory model, every executable or library must either use PIC (position-independent code), or come with relocation tables so jump and branch targets can be adjusted by the loader.

The AmigaOS used this memory model, in absence of a [[MMU]] in early 680x0 CPUs. It is most efficient, but it does not allow for protecting processes from each other, thus it is considered obsolete in today's desktop operating systems. It is also prone to memory fragmentation; certain embedded systems still use it, however.

===Virtual Address Space===
The advent of [[MMU]]s (Memory Management Units) allows virtual addresses to be used. A virtual address can be mapped to any physical address. It is possible to provide each executable with its own address space, so that memory always starts at 0x0000 0000. This relieves the executable loader of some relocation work, and solves the memory fragmentation problem - you no longer need physically continuous blocks of memory. And since the kernel is in control of the virtual-to-physical mapping, processes cannot access each other's memory unless allowed to do so by the kernel.

==Memory Translation Systems==
The x86 platform is unique in modern computer systems in that it has two methods for handling the mapping between virtual and physical addresses. The two methods, paging and segmentation, each use a very different system to manage memory mapping.

===Segmentation===
{{Main|Segmentation}}
Segmentation is not commonly available in mainstream systems except for the x86. In protected mode this method involves separating each area of memory for a process into units. This is handled by the [[CPU Registers x86#Segment Registers|segment registers]]: '''CS, DS, SS, ES, FS, GS''' (CodeSegment, DataSegment, StackSegment, the rest are ExtraSegments).

===Paging===
{{Main|Paging}}
Having an individual virtual-to-physical mapping for each address is of course ineffective. The traditional approach to virtual memory is to split up the available physical memory into chunks (pages), and to map virtual to physical addresses ''page-wise''. This task is largely handled by the [[MMU]], so the performance impact is low, and generally accepted as an appropriate price to pay for memory protection.

[[Image:Virtual memory.png|frame|none|paging illustrated: two process with different views of the same physical memory]]

==Virtual Memory==

The next step is, instead of reporting an "out of memory" once the physical memory runs out, is to take pages that are not actually accessed currently, and write them to hard disk (a swap file or a partition) - freeing up the physical memory page. This is referred to as "paging out" memory.

This requires additional bookkeeping and scheduling, introduces a severe performance hit when a process accesses a page that's currently swapped out and must be swapped in again from hard drive, and requires some smart design to run efficiently at all. Do it wrong, and this one part of your OS can ''severely'' impact your performance.

On the other hand, your "virtual address space" grows to whatever your CPU and hard drive can handle. In concept, CPU caches and RAM simply become cache layers on top of your hard drive, which represents your "real" memory limitation.

Page swapping systems relies on the assumption that, at a given time, a process does not need all of its memory to work properly, but only a subset of it (like, if you're copying a book, you certainly don't need the whole book and a full set of blank pages: the current chapter and a bunch of blank page can be enough if someone can bring you new blank pages and archive the pages you've just written when you come short on blank pages, or bring you the next chapter when you're almost done with the current one). This is known as the ''working set'' abstraction. In order to run correctly, a process requires at least its working set of physical pages: if less pages are provided to the process, there's a high risk of ''thrashing'', which means the process will be constantly requiring pages to be swapped in -- which forces other pages from this process's working set to be swapped out while they should have remained present.

<blockquote>
   ''Note: there are alternatives to page-swapping like segments-swapping and process-swapping. In those cases, the swap is rather user-software controlled, which puts more stress on the application developer and leads to longer swapping burst as the logical things to be swapped are bigger than 4K pages.''

   ''Other note: mainstream desktop OSes have a speculative algorithm that tries to reduce the 'page miss' frequency by loading *more* than what is required, and hoping that these extra pages will be useful. As programs tend to have *localized* access and that disks can read a track of N sectors faster than N independent sector, speculative swap-in may pay.''
</blockquote>

==See Also==

===Articles===
*[[Detecting Memory (x86)]]
*[[Garbage collection]]
*[[Memory Allocation]]
*[[Page Frame Allocation]]
*[[Writing a memory manager]] - a tutorial
*[[Brendan's Memory Management Guide]]

===Threads===
*[[topic:12307|Paging Mechanisms]]
*[[topic:12022|Paging Concepts (by Brendan)]]
*[[topic:22661|Paging Explained (by Creature)]]

===External Links===
*[http://www.amd.com/us-en/Processors/DevelopWithAMD/0,,30_2252_739_7044,00.html AMD Systems Programming Documentation] Chapters 3 & 4 of Volume 2
*[http://www.intel.com/products/processor/manuals/ Intel Systems Programming Documentation] Chapters 3 & 4 of Volume 3A
*[http://linux-mm.org/ LinuxMM] - A wiki documenting memory management projects and development
*[http://www.osdever.net/tutorials/index#Memory-Management Memory Management Articles] - Bona Fide OS Development Tutorials on Memory Management
*[[wikipedia:Memory management | Memory management]] on Wikipedia.
*[http://duartes.org/gustavo/blog/post/motherboard-chipsets-memory-map Jun 2008: Motherboard Chipsets and the Memory Map] by Gustavo Duarte
*[http://duartes.org/gustavo/blog/post/anatomy-of-a-program-in-memory Jan 2009: Anatomy of a Program in Memory] by Gustavo Duarte

[[Category:Memory management]]
[[Category:Physical Memory]]
[[Category:Virtual Memory]]
[[de:Speicherverwaltung]]
````

<a id="w001471"></a>

## W001471 — Category:Storage

Bron-ID: W001471
Namespace: Category (14)
Revisie: 1765 | Datum: 2006-12-03T08:31:01Z
Bron: https://wiki.osdev.org/Category:Storage
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AStorage&oldid=1765

````text
This category includes articles about storage device interfaces, the storage devices themselves and articles concerning driver development and implementation.
{{main|Storage}}

[[Category:Hardware]]
````

<a id="w001472"></a>

## W001472 — Category:Kernel

Bron-ID: W001472
Namespace: Category (14)
Revisie: 2332 | Datum: 2006-12-16T20:47:23Z
Bron: https://wiki.osdev.org/Category:Kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AKernel&oldid=2332

````text
This category lists information regarding kernel design.
{{main|Kernels}}

[[Category:OS theory]]
````

<a id="w001473"></a>

## W001473 — Monotasking Systems

Bron-ID: W001473
Namespace: Main (0)
Revisie: 11215 | Datum: 2011-03-06T09:09:27Z
Bron: https://wiki.osdev.org/Monotasking_Systems
Revisie-URL: https://wiki.osdev.org/index.php?title=Monotasking_Systems&oldid=11215

````text
{{Stub}}

Monotasking systems, also referred to as single-tasking systems, are operating systems in which only one thread of execution is run at a given time. When an application is executed, it takes control of the whole computer, save for a 'resident' part of the OS which handles system calls and reloads the rest of the system when the application exits. Generally speaking, such systems have little or no protection against malicious software.

There are two forms of monotasking systems. The first form is where only one program can be running at once (DexOS is a perfect example), with the second form allowing multiple programs to be running simultaneously but only one program can be in focus at a time as a limitation of the user interface (early Unix systems, or imagine Linux with only one TTY device).

The canonical example of a monotasking operating system is MS-DOS, which falls in between the two forms. While not supporting multitasking directly, MS-DOS allows programs to terminate execution but does not mark the memory they exist in as usable (known as [http://en.wikipedia.org/wiki/Terminate_and_Stay_Resident Terminate and Stay Resident]). Programs which TSR can be communicated with after they quit by devices and other programs.

While they are easier to design and write, monotasking operating systems are extremely limited. As a result, nearly all modern general-purpose OS designs are for [[Multitasking Systems]].

[[Category:Task Models]]
````

<a id="w001474"></a>

## W001474 — Multitasking Systems

Bron-ID: W001474
Namespace: Main (0)
Revisie: 30762 | Datum: 2026-06-16T09:33:34Z
Bron: https://wiki.osdev.org/Multitasking_Systems
Revisie-URL: https://wiki.osdev.org/index.php?title=Multitasking_Systems&oldid=30762

````text
Multitasking Systems are operating systems (or even system extensions) which share available processor time between multiple tasks automatically.

==Types of Multitasking Systems==
There are many ways multitasking can be achieved.

===Cooperative Multitasking===

For [[Cooperative_Multitasking|cooperative multitasking]], a task uses the CPU until it voluntarily gives up the CPU (e.g. yields or exits). Examples of cooperative multitasking systems are pre-X MacOS, or Windows 3.x.

In some single language cooperative multitasking systems, such as Oberon and ruby, the compiler/interpreter automatically ensures that the code will periodically yield control; it allows such program to run multiple threads on operating systems such as DOS.

===Preemptive Multitasking===

[[Preemptive_Multitasking|Preemptive multitasking]] system is an extension of cooperative multitasking, where some task switches are becoming involuntary, and are done for one or more reasons (including when the task consumed the time it was given and/or when a higher priority task needed the CPU). It is important to emphasize here that the involuntary preemption is usually a last resort for the kernel, and in a well-designed system, most of the task switches are/should still be voluntary (as in a purely cooperatively multitasked system). 

Examples include almost all modern operating systems - e.g. Linux, *BSD, post-3.x Windows, BeOS, AmigaOS.

You can further subdivide these systems into those who can preempt tasks, and those who can preempt ''the kernel itself''. Linux (pre-2.6 kernel) is an example of the former, while e.g. AmigaOS is an example for the latter. This is a major concern for multimedia applications or any "soft" [Real-Time Systems] because a non-preemptive kernel introduces latencies that can ruin such "near real-time" performance.

===Real-Time Multitasking===

In a Real-Time Operating System (RTOS), the scheduler works in a specific kind of [[Preemptive_Multitasking|preemptive multitasking]] system. It is designed for applications requiring responses within a fixed time window and tight time constraints. These systems are crucial in fields where delay can result in failure of the application.

However, a Real-Time OS can also be slow. The main goal of a RTOS is to complete tasks within the scheduled timeframe. This means that it isn't any slower or any faster. A RTOS guarantees not speed, but reliability. It is crucial for applications that cannot tolerate delays, such as avionics and aerospace systems, embedded medical devices or automotive controls.

[[Category:Task Models]]
[[Category:Multitasking]]

==See Also==
===Articles===
*[[Context Switching]]
*[[Monotasking Systems]]
*[[Scheduling Algorithms]]
*[[Kernel Multitasking]]

=== External Links ===
*[[wikipedia:Computer_multitasking| Computer Multitasking]] on Wikipedia.

[[Category:Task Models]]
[[Category:Multitasking]]
````

<a id="w001475"></a>

## W001475 — Talk:Processes and Threads

Bron-ID: W001475
Namespace: Talk (1)
Revisie: 1786 | Datum: 2006-12-03T15:37:15Z
Bron: https://wiki.osdev.org/Talk:Processes_and_Threads
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AProcesses_and_Threads&oldid=1786

````text
==Category==
What category should this be in?
Should this and IPC be merged under the category Concurrent Computer? (That is how it is done in wikipedia) - [[User:Telexicon|Telexicon]] 03:20, 3 December 2006 (CST)

I think its a direct sub to OS Theory. Even DOS can run several processes without it being concurrent - it just suspends the previous task. I think linking from IPC and so would be more appropriate (added to os theory for now)

- [[User:Combuster|Combuster]] 09:37, 3 December 2006 (CST)
````

<a id="w001476"></a>

## W001476 — Real-Time Systems

Bron-ID: W001476
Namespace: Main (0)
Revisie: 22704 | Datum: 2018-09-10T01:53:31Z
Bron: https://wiki.osdev.org/Real-Time_Systems
Revisie-URL: https://wiki.osdev.org/index.php?title=Real-Time_Systems&oldid=22704

````text
{{Opinion}}
Real-Time Operating Systems are systems in which certain processes or operations have guaranteed minimum and/or maximum response times. That is to say, the system ensures that it will complete operation x after time '''t''' but before time '''t<sub>2</sub>''', whatever '''t''' and '''t<sub>2</sub>''' are, without fail, even at the expense of other lower priority operations.

Speed, in and of itself, is not critical; the primary goal is predictability. A response time less than '''t''' may be just as bad as, or worse than, one greater than '''t<sub>2</sub>'''.

'Soft' real-time systems (e.g., laser-printer microcontrollers) promise best-effort reliability; 'hard' real-time systems (e.g., 'fly-by-wire' avionics in an aircraft) need to have zero-failure-tolerance reliability.

Real time operating systems are, as a general rule, only used in time-dependent embedded applications; general-purpose systems rarely if ever need to meet real-time constraints. When the do occur, real-time considerations may rule out the use of some common techniques, such as virtual memory, which may make the system's behavior less deterministic. Best-effort real-time functionality can however be useful in general purposes systems for supporting the needs of applications like digital audio workstations which demand both reliability (live recording) and low latency (real-time synthesis/processing), often at the same time.

One of the best-known Real Time OS for the x86 platform is QNX. Each system call of QNX is documented with a 'worst case completion time'.

It should be noted that "being Real Time" not necessarily means that an OS is very good at playing MPEGs, or using hardware efficiently - this is a common misunderstanding. To the contrary, for a system to provide hard real-time services implies that it can use only a limited percentage of the system's resources, including (or even especially) CPU time, during most if not all of the system's running time, as it must ensure that those resources are available for the real-time operations at need. It also fundamentally changes how software can be built for the system. For example, Rate Monotonic Scheduling - a hard real-time scheduling algorithm - can guarantee time restraints only up to 70% CPU load. Beyond that, the system has "to hit the red button" because it can no longer guarantee anything.

This implies that applications have to state their run-time requirements beforehand - how often they must be called in a second, which maximum response time is acceptable etc. All this information must be provided by the application programmer. In some cases the information is provided in implicit form, for example by arranging processes to have a certain order of priorities which allows them to meet their goals.

Bottom line, hard real time is for industrial, medical, or military systems. On your average desktop, it is misplaced.

== See Also ==
=== Articles ===
* [[Scheduling Algorithms#Real-Time Scheduling Algorithms|Real-Time Scheduling Algorithms]]
=== External Links ===
* [[wikipedia:Real_time_operating_system|Wikipedia - Real Time Operating System]]
* [http://doc.cat-v.org/plan_9/real_time/ Papers describing the real time implementation in Plan 9 from Bell Labs]

[[Category:Task Models]]
````

<a id="w001477"></a>

## W001477 — Scheduling Algorithms

Bron-ID: W001477
Namespace: Main (0)
Revisie: 28155 | Datum: 2023-07-10T14:36:22Z
Bron: https://wiki.osdev.org/Scheduling_Algorithms
Revisie-URL: https://wiki.osdev.org/index.php?title=Scheduling_Algorithms&oldid=28155

````text
A scheduling algorithm is the algorithm which dictates how much CPU time is allocated to [[Processes and Threads]]. The goal of any scheduling algorithm is to fulfill a number of criteria:
* no task must be starved of resources - all tasks must get their chance at CPU time;
* if using priorities, a low-priority task must not hold up a high-priority task;
* the scheduler must scale well with a growing number of tasks, ideally being O(1). This has been done, for example, in the Linux kernel.

==Interactive Scheduling Algorithms==
===Round Robin===
Round Robin is the simplest algorithm for a preemptive scheduler. Only a single queue of processes is used. When the system timer fires, the next process in the queue is switched to, and the preempted process is put back into the queue.

Each process is assigned a time slice or "quantum". This quantum dictates the number of system timer ticks the process may run for before being preempted. For example, if the timer runs at 100Hz, and a process' quantum is 10 ticks, it may run for 100 milliseconds (10/100 of a second). To achieve this, the running process is given a variable that starts at its quantum, and is then decremented each tick until it reaches zero. The process may also relinquish its quantum by doing a blocking system call (i.e. I/O), like in other preemptive algorithms.

In the Round Robin algorithm, each process is given an equal quantum; the big question is how to choose the time quantum.

Here are some considerations:
The smaller the quantum, the larger the proportion of the time used in context switches.<br>
Interactive processes should do I/O before being preempted, so that unnecessary preemptions are avoided.<br>
The larger the quantum, the "laggier" the user experience - quanta above 100ms should be avoided.<br> 

A frequently chosen compromise for the quantum is between 20ms and 50ms.

Advantages of Round Robin include its simplicity and strict "first come, first served" nature. Disadvantages include the absence of a priority system: lots of low privilege processes may starve one high privilege one.

===Priority-Based Round Robin===
Priority scheduling is similar to Round Robin, but allows a hierarchy of processes. Multiple process queues are used, one for each priority. As long as there are processes in a higher priority queue, they are run first. For example, if you have 2 queues, "high" and "low", in this state:

"high": X<br>
"low": xterm, vim, firefox

The first process to run would be X, then if it blocked (for I/O, probably), the state would be:

"high":<br>
"low": xterm, vim, firefox

The next process that would run would be xterm. If process "kswapd" is added to "high", it would then get the next quantum:

"high": kswapd<br>
"low": vim, firefox, xterm

There are usually between four and sixteen queues used in a priority scheduler.

Advantages of this algorithm are simplicity and reasonable support for priorities. The disadvantage (or possible advantage) is that privileged processes may completely starve unprivileged ones. This is less of a problem than it appears, because processes (especially daemons, which are usually privileged) are usually blocked for I/O. 

Let's have a look on the round robin scheduler with three processes in the queue: A B C:

<pre>
A(time 0) B(time 10) C(time 10)  A's time slice is zero: let's do round robin scheduling:
B(time 10) C(time 10) A(time 10) ... one clock tick occurs ... the next one ...
B(time 8) C(time 10) A(time 10)  ... several clock ticks occur ... b's time slice is worn out ...
C(time 10) A(time 10) B(time 10) ... ten clock ticks later ...
A(time 10) B(time 10) C(time 10) ... now A has its share of CPU time again.
</pre>

====SVR2 Unix Implementation====
Classical UNIX systems scheduled ''equal-priority'' processes in a round-robin manner, each running for a fixed time quantum. If a higher priority process becomes runnable, it will preempt the current process (if it's ''not'' running in kernel mode, since classical UNIX kernels were non-preemptive) even if the process did not finish its time quantum. This way, high priority processes can possibly starve low-priority ones. To avoid this, a new factor in calculating a process priority was introduced: the 'usage' factor.

This factor allows the kernel to vary processes priorities dynamically. When a process is not running, the kernel periodically increases its priority. When a process receives some CPU time, the kernel reduces its priority. This scheme will potentially prevent the starvation of any process, since eventually the priority of any waiting process will rise high enough to be scheduled.

All user-space priorities are lower than the lowest system priority. The usage factor of a user-process is calculated by the amount of compute time to real-time consumed by the process. A process that has used a lot of compute time in the last real-time unit is assigned a low user priority. Because interactive processes are characterized by low ratios of compute to real time, interactive response is maintained without any special arrangements.

If there are no processes eligible for execution, the CPU idles till the next interrupt, which will happen ''at most'' after one clock tick. After handling the interrupt, the kernel again attempts to schedule a process to run.

=====References=====
Ken Thompson, "UNIX Implementation", 2.3 - Synchronization and Scheduling, Bell Laboratories

Maurice J. Bach, "The Design of the UNIX Operating System", Chapter 8 - Process Scheduling and Time, Prentice Hall

=== Shortest Process Next ===
Version of SRTN (Shortest Remaining Time Next) for interactive systems. The Problem here is that we can't say what the user's next command will be. This algorithm needs prediction :)

=== Lottery Scheduling ===
Lottery Scheduling is a simple algorithm that statistically guarantees a variable fraction of processor time to each runnable process. The concept is much like a lottery. At each scheduling decision, each runnable process is given a number of "lottery tickets". Then a random number is generated, corresponding to a specific ticket. The process with that ticket gets the quantum.

Although there is no absolute guarantee that processes will be treated equally, the frequency of scheduling events in a preemptive multitasking system means that it comes very close to doing so. The number of tickets given to a process divided by the total number of tickets is the statistical fraction of the quanta given to that process. For example, if these processes are given this number of tickets:

foo - 5<br>
bar - 7<br>
bash - 4

The fractions of processor time given to each should be:

foo - 5/16 - 31%<br>
bar - 7/16 - 44%<br>
bash - 4/16 - 25%

As you can see, it is trivial to create a fine grained priority system: just give higher priority processes more tickets.

Advantages of Lottery Scheduling include fine grained priorities and statistical fairness. Disadvantages include the need for a reliable random number generator and non-absolute guarantees, especially on systems with large quanta.

You need to implement a [[Random Number Generator]] to use this algorithm.

==Batch Scheduling Algorithms==


=== First Come First Served ===

This scheduling method is used on Batch-Systems, it is NON-PREEMPTIVE.
It implements just one queue which holds the tasks in order they come in.

The order the tasks arrive is very important for the Turnaround-Time:

Task1(24) Task2(6) Task3(6)<br>
avg. Turnaround-Time = (24 + 30 + 36) / 3 = 30 time units  (this assumes all tasks arrive at time 0)

Task1(6) Task2(6) Task3(24)<br>
avg. Turnaround-Time = (6 +12 +36) / 3 = 18 time units (this assumes all tasks arrive at time 0)

Strengths:

-Simple<br>
-Fair

Problems:

-Convoy Effect<br>
-Order of task arrival is very important for average Turnaround time

=== Shortest Job First (SJF) ===

Is also NON-PREEMPTIVE. It selects the shortest Job/Process which is available in the run queue.<br>
This scheduling algorithm assumes that run times are known in advance.

Strengths:

-nearly optimal (Turnaround Time)

Problems:

-Only optimal if all jobs/process are available simultaneously<br> 
-Usually run times are not known ...

===Shortest Remaining Time Next===

Preemptive version of SJF (Shortest Job First).
Schedulre picks the job/process which has the lowest remaining time to run.

Strengths:

-probably optimal (Tournaround Time)

Problems:

-again run times must be known

===Highest Response Ratio Next===
{{stub}}

==Real-Time Scheduling Algorithms==

Real-Time Scheduling Algorithms are a special class of algorithms of which it is required that they can guarantee a process will be done before its deadline. The only way these algorithms can work is if they at least know when the deadline for a process is, and how much the process takes of the system. Only if the system is not overloaded (subjective term) can the threads be guaranteed to finish before their deadline.

Each task has to be scheduled X<sub>t</sub> times a second, or every Y<sub>t</sub> milliseconds (Y<sub>t</sub> = 1000 / X<sub>t</sub>). Each run of that task takes at most Z<sub>t</sub> milliseconds. This task then creates a load of L<sub>t</sub> = Z<sub>t</sub> / Y<sub>t</sub>.

The system as a whole has a load L, which is the sum of all task-loads: L = <big>E</big> L<sub>t</sub>. If the system load exceeds 0.7 (in some rare cases it can be slightly larger, but we don't count them) the system is unschedulable using Rate Monotonic Scheduling. If this system load exceeds 1.0 it is unschedulable for any real-time system. Note that for normal systems any load is possible, including the ones that are extremely large. They will make the system very unusable though.

===Rate Monotonic Scheduling===
Rate Monotonic Scheduling is a way to schedule Real-Time threads in such a way, that can be guaranteed that none of the threads will ever exceed their deadline.

The load of the system may vary, but there is a utilisation-based test that, if satisfied, guarantees that the system will always be schedulable. As an example the utilisation limit for a system with one process is 100% (as there is no need for preemption). The utilisation limit for a system with 3 processes is approximately 69%.

The utilisation based test is ''sufficient'' but not ''necessary''. If a process set passes the utilisation based test, it will be schedulable. However, process sets can be constructed that fail the utilisation test but are in fact (trivially) schedulable.

The RMS scheduling works by assigning each task a priority based on its interval. The task with the shortest interval gets the highest priority and the task with the largest interval gets the lowest priority (still real-time though). The tasks are then run similar to a prioritized preempting [#Round-Robin]. This means, any task that can run runs, and if a task runs but a task with a higher priority is available, the higher one runs instead.

If your system is based on a [[#Round-Robin|Round-Robin]] scheduler, this is the easiest way to do Real-Time scheduling.

===Earliest Deadline First===

Each task in an EDF scheduler is assigned a _deadline_ (e.g. a moment in the future at which the task _must_ be completed). Every time a task is inserted in the system or completed, the scheduler looks for the task which has the closest deadline and selects it for execution. In order to ensure that the scheduler is still able to meet each deadline, a '''monitor'' must evaluate if each new task doesn't overload the system and deny execution if it will do so.

In order to implement EDF-based system, one will have to know both the _deadline_ of the task (which could optionally be computed as "no more than X ms in the future") and the expected time needed to perform the task (required by the monitor). QoS network routers usually implement variants of EDF scheduling.

Again, there is a utilisation based test for EDF systems. The limit is simpler however - it is always 100%, no matter how many processes are in the set. This makes dynamic analysis of schedulability easier. Not only that, but the EDF utilisation test is both ''sufficient'' and ''necessary'', so can be relied on to provide an accurate indication of schedulability.

For more information, see "Real time systems and programming languages" by Burns & Wellings.

==See Also==
===Articles===
*[[Real-Time Systems]]

===Threads===
*[[Topic:8575|Real-Time Scheduling]]

===External Links===
*[https://web.archive.org/web/20070818053107/http://josh.trancesoftware.com/linux/ Linux 2.6 scheduler, under FDL]
*[[Wikipedia:Scheduling (computing)]]

[[Category:Processes and Threads]]
[[Category:Common Algorithms]]
[[Category:Multitasking]]
[[Category:Scheduling]]
````

<a id="w001478"></a>

## W001478 — 8259 PIC

Bron-ID: W001478
Namespace: Main (0)
Revisie: 30849 | Datum: 2026-07-04T18:16:11Z
Bron: https://wiki.osdev.org/8259_PIC
Revisie-URL: https://wiki.osdev.org/index.php?title=8259_PIC&oldid=30849

````text
{{Historical}}

The 8259 ''Programmable Interrupt Controller'' (PIC) is one of the most important chips making up the [[x86]] architecture. Without it, the x86 architecture would not be an interrupt driven architecture. The function of the 8259A is to manage hardware interrupts and send them to the appropriate system [[Interrupts|interrupt]]. This allows the system to respond to devices needs without loss of time (from polling the device, for instance).

It is important to note that [[APIC]] has replaced the 8259 PIC in more modern systems, especially those with multiple cores/processors.

==What does the 8259 PIC do?==
The 8259 PIC controls the CPU's interrupt mechanism, by accepting several interrupt requests and feeding them to the processor in order. For instance, when a keyboard registers a keyhit, it sends a pulse along its interrupt line ([[IRQ]] 1) to the PIC chip, which then translates the IRQ into a system interrupt, and sends a message to interrupt the CPU from whatever it is doing. Part of the kernel's job is to either handle these IRQs and perform the necessary procedures (poll the keyboard for the scancode) or alert a userspace program to the interrupt (send a message to the keyboard driver).

Without a PIC, you would have to poll all the devices in the system to see if they want to do anything (signal an event), but with a PIC, your system can run along nicely until such time that a device wants to signal an event, which means you don't waste time going to the devices, you let the devices come to you when they are ready.

===The IBM PC 8259 PIC Architecture===
In the beginning (IBM PC and XT), only a single 8259 PIC chip was used, which provided 8 IRQs to the system. These were traditionally mapped by the BIOS to interrupts 8 to 15 (0x08 to 0x0F). It is unlikely that any of these single-PIC machines will be encountered these days.

===The IBM PC/AT 8259 PIC Architecture===
The IBM PC/AT extended the PC architecture by adding a second 8259 PIC chip. This was possible due to the 8259A's ability to cascade interrupts, that is, have them flow through one chip and into another. This gives a total of 15 interrupts. Why 15 and not 16? That's because when you cascade chips, the PIC needs to use one of the interrupt lines to signal the other chip.

Thus, in an AT, IRQ line 2 is used to signal the second chip. Because of this, IRQ 2 is not available for use by hardware devices, which got wired to IRQ 9 on the slave PIC instead. The real mode BIOS used to set up an interrupt handler for IRQ 9 that redirects to the IRQ 2 handler. This way DOS drivers who used IRQ 2 continued to work. This two-chip architecture is still used and available in modern systems, and hasn't changed (except for the advent of the above-mentioned APIC architecture).

==How does the 8259 PIC chip work?== 
Each of the two 8259 PICs in modern systems have 8 inputs. When any of the inputs is raised, the PIC sets a bit internally telling one of the inputs needs servicing. It then checks whether that channel is masked or not, and whether there's an interrupt already pending.
If the channel is unmasked and there's no interrupt pending, the PIC will raise the interrupt line. On the slave, this feeds IRQ 2 to the master, and the master is connected to the processor interrupt line.

When the processor accepts the interrupt, the master checks which of the two PICs is responsible for answering, then either supplies the interrupt number to the processor, or asks the slave to do so. The PIC that answers looks up the "vector offset" variable stored internally and adds the input line to form the requested interrupt number. After that the processor will look up the interrupt address and act accordingly (see [[Interrupts]] for more details).

==Programming with the 8259 PIC==
Each chip (master and slave) has a command port and a data port (given in the table below). When no command is issued, the data port allows us to access the interrupt mask of the 8259 PIC.
{| {{wikitable}}
|-
! Chip - Purpose
! [[I/O ports|I/O port]]
|-
| Master PIC - Command
| 0x0020
|-
| Master PIC - Data
| 0x0021
|-
| Slave PIC - Command 	
| 0x00A0
|-
| Slave PIC - Data 
| 0x00A1
|}
* Each PIC vector offset must be divisible by 8, as the 8259A uses the lower 3 bits for the interrupt number of a particular interrupt (0..7).
* The only way to change the vector offsets used by the 8259 PIC is to re-initialize it, which explains why the code is "so long" and plenty of things that have apparently no reasons to be here.
* If you plan to return to real mode from protected mode (for any purpose), you really must restore the PIC to its former configuration.

===Real Mode===
{| {{wikitable}}
|-
! Chip
! Interrupt numbers (IRQ)
! Vector offset
! Interrupt Numbers
|-
| Master PIC
| 0 to 7
| 0x08
| 0x08 to 0x0F
|-
| Slave PIC
| 8 to 15
| 0x70
| 0x70 to 0x77
|}
These default BIOS values suit real mode programming quite well; they do not conflict with any CPU exceptions like they do in protected mode.

===Protected Mode===
In protected mode, the IRQs 0 to 7 conflict with the CPU exception which are reserved by Intel up until 0x1F. (It was an IBM design mistake.) Consequently it is difficult to tell the difference between an IRQ or an software error. It is thus recommended to change the PIC's offsets (also known as remapping the PIC) so that IRQs use non-reserved vectors. A common choice is to move them to the beginning of the available range (IRQs 0..0xF -> INT 0x20..0x2F). For that, we need to set the master PIC's offset to 0x20 and the slave's to 0x28. For code examples, see below.

==Code Examples==

===Common Definitions===
This is just a set of definitions common to the rest of this section. For the outb(), inb() and io_wait() functions, see [[Inline_Assembly/Examples#I/O_access|this page]].
<syntaxhighlight lang="C">
#define PIC1		0x20		/* IO base address for master PIC */
#define PIC2		0xA0		/* IO base address for slave PIC */
#define PIC1_COMMAND	PIC1
#define PIC1_DATA	(PIC1+1)
#define PIC2_COMMAND	PIC2
#define PIC2_DATA	(PIC2+1)
</syntaxhighlight>

===End of Interrupt===
Perhaps the most common command issued to the PIC chips is the ''end of interrupt'' (EOI) command (code 0x20). This is issued to the PIC chips at the end of an IRQ-based interrupt routine. If the IRQ came from the Master PIC, it is sufficient to issue this command only to the Master PIC; however if the IRQ came from the Slave PIC, it is necessary to issue the command to both PIC chips.
<syntaxhighlight lang="C">
#define PIC_EOI		0x20		/* End-of-interrupt command code */

void PIC_sendEOI(uint8_t irq)
{
	if(irq >= 8)
		outb(PIC2_COMMAND,PIC_EOI);
	
	outb(PIC1_COMMAND,PIC_EOI);
}
</syntaxhighlight>

===Initialisation===
When you enter protected mode (or even before hand, if you're not using [[GRUB]]) the first command you will need to give the two PICs is the initialise command (code 0x11). This command makes the PIC wait for 3 extra "initialisation words" on the data port. These bytes give the PIC:
* Its vector offset. (ICW2)
* Tell it how it is wired to master/slaves. (ICW3)
* Gives additional information about the environment. (ICW4)

<syntaxhighlight lang="c">
/* reinitialize the PIC controllers, giving them specified vector offsets
   rather than 8h and 70h, as configured by default */

#define ICW1_ICW4	0x01		/* Indicates that ICW4 will be present */
#define ICW1_SINGLE	0x02		/* Single (cascade) mode */
#define ICW1_INTERVAL4	0x04		/* Call address interval 4 (8) */
#define ICW1_LEVEL	0x08		/* Level triggered (edge) mode */
#define ICW1_INIT	0x10		/* Initialization - required! */

#define ICW4_8086	0x01		/* 8086/88 (MCS-80/85) mode */
#define ICW4_AUTO	0x02		/* Auto (normal) EOI */
#define ICW4_BUF_SLAVE	0x08		/* Buffered mode/slave */
#define ICW4_BUF_MASTER	0x0C		/* Buffered mode/master */
#define ICW4_SFNM	0x10		/* Special fully nested (not) */

#define CASCADE_IRQ 2

/*
arguments:
	offset1 - vector offset for master PIC
		vectors on the master become offset1..offset1+7
	offset2 - same for slave PIC: offset2..offset2+7
*/
void PIC_remap(int offset1, int offset2)
{
	outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);  // starts the initialization sequence (in cascade mode)
	io_wait();
	outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);
	io_wait();
	outb(PIC1_DATA, offset1);                 // ICW2: Master PIC vector offset
	io_wait();
	outb(PIC2_DATA, offset2);                 // ICW2: Slave PIC vector offset
	io_wait();
	outb(PIC1_DATA, 1 << CASCADE_IRQ);        // ICW3: tell Master PIC that there is a slave PIC at IRQ2
	io_wait();
	outb(PIC2_DATA, CASCADE_IRQ);             // ICW3: tell Slave PIC its cascade identity
	io_wait();
	
	outb(PIC1_DATA, ICW4_8086);               // ICW4: have the PICs use 8086 mode (and not 8080 mode)
	io_wait();
	outb(PIC2_DATA, ICW4_8086);
	io_wait();

	// Unmask both PICs.
	outb(PIC1_DATA, 0);
	outb(PIC2_DATA, 0);
}
</syntaxhighlight>

''Note the presence of io_wait() calls, on older machines its necessary to give the PIC some time to react to commands as they might not be processed quickly.''

==Disabling==
If you are going to use the processor local APIC and the IOAPIC, you must first disable the PIC. This is done by masking every single interrupt.
<syntaxhighlight lang="C">
void PIC_disable(void) {
    outb(PIC1_DATA, 0xff);
    outb(PIC2_DATA, 0xff);
}
</syntaxhighlight>

==Masking==
The PIC has an internal register called the IMR, or the Interrupt Mask Register. It is 8 bits wide. This register is a bitmap of the request lines going into the PIC. When a bit is set, the PIC ignores the request and continues normal operation. Note that setting the mask on a higher request line will not affect a lower line. Masking IRQ2 will cause the Slave PIC to stop raising IRQs.

Here is an example of how to mask an IRQ:
<syntaxhighlight lang="C">
void IRQ_set_mask(uint8_t IRQline) {
    uint16_t port;
    uint8_t value;

    if(IRQline < 8) {
        port = PIC1_DATA;
    } else {
        port = PIC2_DATA;
        IRQline -= 8;
    }
    value = inb(port) | (1 << IRQline);
    outb(port, value);        
}

void IRQ_clear_mask(uint8_t IRQline) {
    uint16_t port;
    uint8_t value;

    if(IRQline < 8) {
        port = PIC1_DATA;
    } else {
        port = PIC2_DATA;
        IRQline -= 8;
    }
    value = inb(port) & ~(1 << IRQline);
    outb(port, value);        
}
</syntaxhighlight>

==ISR and IRR==
The PIC chip has two interrupt status registers: the In-Service Register (ISR) and the Interrupt Request Register (IRR).  The ISR tells us which interrupts are being serviced, meaning IRQs sent to the CPU.  The IRR tells us which interrupts have been raised.  Based on the interrupt mask (IMR), the PIC will send interrupts from the IRR to the CPU, at which point they are marked in the ISR.

The ISR and IRR can be read via the OCW3 command word.  This is a command sent to one of the command ports (0x20 or 0xa0) with bit 3 set.  To read the ISR or IRR, write the appropriate command to the command port, and then read the command port (not the data port).  To read the IRR, write 0x0a.  To read the ISR, write 0x0b.  

The ISR and IRR are each 8 bits.  Here is an example of how to read 16 bits worth of ISR and IRR data from two cascaded PICs:
<syntaxhighlight lang="C">
#define PIC_READ_IRR                0x0a    /* OCW3 irq ready next CMD read */
#define PIC_READ_ISR                0x0b    /* OCW3 irq service next CMD read */

/* Helper func */
static uint16_t __pic_get_irq_reg(int ocw3)
{
    /* OCW3 to PIC CMD to get the register values.  PIC2 is chained, and
     * represents IRQs 8-15.  PIC1 is IRQs 0-7, with 2 being the chain */
    outb(PIC1_COMMAND, ocw3);
    outb(PIC2_COMMAND, ocw3);
    return (inb(PIC2_COMMAND) << 8) | inb(PIC1_COMMAND);
}

/* Returns the combined value of the cascaded PICs irq request register */
uint16_t pic_get_irr(void)
{
    return __pic_get_irq_reg(PIC_READ_IRR);
}

/* Returns the combined value of the cascaded PICs in-service register */
uint16_t pic_get_isr(void)
{
    return __pic_get_irq_reg(PIC_READ_ISR);
}
</syntaxhighlight>

Note that these functions will show bit 2 (0x0004) as on whenever any of the PIC2 bits are set, due to the chained nature of the PICs.  Also note that it is not necessary to reset the OCW3 command every time you want to read.  Once you set it for either the IRR or the ISR, future reads of the CMD port will return the appropriate register.  The chip remembers what OCW3 setting you used.  (Disclaimer: I have not tested this last part, but that's what the spec says.)

==Spurious IRQs==

When an IRQ occurs, the PIC chip tells the CPU (via. the PIC's INTR line) that there's an interrupt, and the CPU acknowledges this and waits for the PIC to send the interrupt vector. This creates a race condition: if the IRQ disappears after the PIC has told the CPU there's an interrupt but before the PIC has sent the interrupt vector to the CPU, then the CPU will be waiting for the PIC to tell it which interrupt vector but the PIC won't have a valid interrupt vector to tell the CPU.

To get around this, the PIC tells the CPU a fake interrupt number. This is a spurious IRQ. The fake interrupt number is the lowest priority interrupt number for the corresponding PIC chip (IRQ 7 for the master PIC, and IRQ 15 for the slave PIC).

There are several reasons for the interrupt to disappear. In my experience the most common reason is software sending an EOI at the wrong time. Other reasons include noise on IRQ lines (or the INTR line).

===Handling Spurious IRQs===

For a spurious IRQ, there is no real IRQ and the PIC chip's ISR (In Service Register) flag for the corresponding IRQ will not be set. This means that the interrupt handler must not send an EOI back to the PIC to reset the ISR flag.

The correct way to handle an IRQ 7 is to first check the master PIC chip's ISR to see if the IRQ is a spurious IRQ or a real IRQ. If it is a real IRQ then it is treated the same as any other real IRQ. If it is a spurious IRQ then you ignore it (and do not send the EOI).

The correct way to handle an IRQ 15 is similar, but a little trickier due to the interaction between the slave PIC and the master PIC. First check the slave PIC chip's ISR to see if the IRQ is a spurious IRQ or a real IRQ. If it is a real IRQ then it is treated the same as any other real IRQ. If it's a spurious IRQ then don't send the EOI to the slave PIC; however you will still need to send the EOI to the master PIC because the master PIC itself won't know that it was a spurious IRQ from the slave.

Also note that some kernels (e.g. Linux) keep track of the number of spurious IRQs that have occurred (e.g. by incrementing a counter when a spurious IRQ occurs). This can be useful for detecting problems in software (e.g. sending EOIs at the wrong time) and detecting problems in hardware (e.g. line noise).


==See Also==

===Articles=== 
*[[APIC]]
*[[IOAPIC]]

===Threads===

===External Links=== 
*[http://pdos.csail.mit.edu/6.828/2005/readings/hardware/8259A.pdf Intel Datasheet]
*http://www.brokenthorn.com/Resources/OSDevPic.html
*http://helppc.netcore2k.net/hardware/8259

[[Category:Interrupts]]
[[Category:X86]]
[[de:Programmable Interrupt Controller]]
````

<a id="w001479"></a>

## W001479 — NASM

Bron-ID: W001479
Namespace: Main (0)
Revisie: 30122 | Datum: 2026-03-03T19:43:57Z
Bron: https://wiki.osdev.org/NASM
Revisie-URL: https://wiki.osdev.org/index.php?title=NASM&oldid=30122

````text
The '''Netwide Assembler (NASM)''' is a popular assembler for the x86/x64 architecture. NASM is open source software and is freely available from [http://www.nasm.us their official website].

NASM is available for many operating systems, including (but not limited to) Windows, GNU/Linux, BSD and Solaris. Because it is Open Source, NASM can be made to run on pretty much any operating system, and is a common choice for a first port to a hobby OS.

== Comparison to [[GAS]] ==
Those preferring it over gas claim its easy syntax and powerful macro capabilities are a plus. On the other hand, other users claim gas is better integrated into the GNU Compiler Collection ([[GCC]]) and has more cross-platform capabilities. Similar to editor and browser discussions, this is likely to remain an unresolved debate.

==Tips==
* Use <tt>-w+orphan-labels</tt>. Often, labels without colons were not really intended to be labels, and are actually misspellings or typos. This flag will cause NASM to warn you if there are labels without colons. On newer versions, this warning is enabled by default.
* Explicitly define your sections. While NASM defaults to <tt>.text</tt>, it doesn't put labels declared as globals in <tt>.text</tt> correctly, unless the label is preceded by a section directive at some point.
* Use the <tt>-g</tt> flag to generate debug information. For [[ELF]] files it uses [[DWARF]] by default. Other debugging formats can be selected with the <tt>-F</tt> flag.

==See also==
*[[User:Johnburger/NASMCritique|Critique of NASM from a TASM user]]

===Articles===
*[[Tool Comparison]]
*[[GAS]]
*[[Assembly]]

===External Links===
* [https://www.nasm.us NASM home page]
* [https://www.nasm.us/docs.html NASM manual]
* [https://pacman128.github.io/pcasm/ PC Assembly Book] Introduction to x86 assembly using NASM
* [http://www.duntemann.com/assembly.html Assembly Language Step-by-Step: Programming with Linux], a book about learning Assembly language using NASM
* The [https://sourceforge.net/projects/nasmx/ NASMX project] (formerly NASM32) is a collection of macros, include files and examples that are meant to accelerate NASM programming; including helpers to begin developing 32-bit and 64-bit Windows, Linux and XBox applications.


[[Category:Assemblers]]
[[de:Netwide Assembler]]
````

<a id="w001480"></a>

## W001480 — GAS

Bron-ID: W001480
Namespace: Main (0)
Revisie: 20294 | Datum: 2017-01-26T21:05:28Z
Bron: https://wiki.osdev.org/GAS
Revisie-URL: https://wiki.osdev.org/index.php?title=GAS&oldid=20294

````text
{{Stub}}
{{Infobox_Tool
|name=GNU Assembler
|website=http://www.gnu.org/software/binutils/
}}
The '''GNU Assembler''' ('''GAS''', executable named 'as') is part of the [[binutils]] package. [[GCC]] generates [[Assembly|assembly]] source code and automatically invokes GAS which assembles the code into machine code in an [[Object Files|object file]]. GCC allows developers to forward assembly directly to the assembler using inline assembly feature.

GAS is not ''one'' assembler, but a collection of assemblers - one for each of the platforms supported by GCC. They are very similar in their available options, their macro language, and so on. GAS provides a very solid and well-supported assembler that is fully integrated with the other GNU tools (implicit <tt>make</tt> rules, inline assembly from C/C++ sources, the ability to be called as <tt>gcc -o myfile.o myfile.s</tt>, the same assembly syntax used by all the tools).

== Usage ==

=== AT&T Syntax ===

The AT&T assembly syntax is traditional for Unix-like operating systems on the [[x86]] and [[x86-64]] platforms. The syntax is reminiscent of the original assembler format used in the original Unix operating system. The operand order for instructions is source and then destination. The assembly language is unambiguous as registers must be prefixed with a <tt>%</tt> character and constants must be prefixed with the <tt>$</tt> character.

=== Intel Syntax Support ===

Up until v2.10 of binutils, GAS supported only the AT&T syntax for [[x86]] and [[x86-64]], which differs significantly from the Intel syntax used by virtually every other assembler. Today, GAS supports both syntax sets (<tt>.intel_syntax</tt> and the default <tt>.att_syntax</tt>), and even allows disabling the otherwise mandatory operand prefixes '%' or '$' (<tt>..._syntax noprefix</tt>). There ''are'' some pitfalls - several FP opcodes suffer from a reverse operand ordering that is bound to stay in there for compatibility reasons, <tt>.intel_syntax</tt> generates less optimized opcodes on occasion (try <tt>mov</tt>'ing to %si...).

It is generally discouraged to use the support for Intel Syntax because it can subtly and surprisingly different than the real Intel Syntax found in other assemblers. [[Tool Comparison|A different assembler]] should be considered if Intel Syntax is desired.

== See Also ==

=== Articles ===
* [[Assembly]]
* [[Tool Comparison]]
* [[Wikipedia:GNU Assembler|GNU Assembler]] - Wikipedia
* [[Wikibooks:X86 Assembly/GAS Syntax|GAS Syntax]] - Wikibook

=== External Links ===
* [https://sourceware.org/binutils/docs/as/ GNU as official manual]

[[Category:Assemblers]]
[[Category:Binutils]]
````

<a id="w001481"></a>

## W001481 — GCC

Bron-ID: W001481
Namespace: Main (0)
Revisie: 27491 | Datum: 2022-11-18T21:23:12Z
Bron: https://wiki.osdev.org/GCC
Revisie-URL: https://wiki.osdev.org/index.php?title=GCC&oldid=27491

````text
The GNU Compiler Collection (GCC) is a toolkit for compiling and assembling programs for a wide variety of platforms and languages. It is one of the core utilities in the Free Software Foundation's GNU Project.

While it is most closely associated with C and C++, it is in fact a suite of several compilers and their libraries. The newest version of GCC as of 2022-11-18 is version 12.2.0 which includes support for ANSI and K&R C, Objective-C, C++, Fortran, Go, D and Ada.

GCC is strongly supported on POSIX compliant platforms but has also been ported to other operating systems such as Windows. GCC also includes support for targeting many different CPU architectures including [[X86]], [[X86-64]], [[IA-64]], [[Alpha]], [[Sparc Overview|SPARC]], [[MIPS Overview|MIPS]], [[PowerPC Overview | PowerPC]], and [[ARM Overview|ARM]].

== OS Development with GCC ==
Although other compilers can be used for OS development, OS developers are encouraged to use one of GCC derivatives.

You need a [[GCC_Cross-Compiler|cross-compiler]] to compile your operating system, as it will otherwise assume you are building programs for your current operating system, when it really is doing something else.

=== Building GCC ===
{{Main|Building GCC}}

It is worth getting the newest release of GCC and installing it as your system compiler and occasionally upgrading it. It will also help bootstrap a proper cross-compiler.

=== Creating a GCC Cross-Compiler ===
{{Main|GCC Cross-Compiler}}

It is easy and takes a few moments to build a cross-compiler that targets your operating system. It may take a while to build it on slower computers, but you only need to do it once, and you save all the time you would otherwise spend on "fixing" the completely imaginary problems you would encounter otherwise. Later on, when you start building a user-space for your operating system, it is worth creating an [[OS_Specific_Toolchain|OS Specific Toolchain]] for absolute control of the compiler and to easy compiling user-space programs.

=== OS-Specific Toolchain ===
{{Main|OS Specific Toolchain}}

Once you begin adding a user-space, it's worth setting up a toolchain that understands your operating system and how to produce programs for it. This is the first task when you begin porting GCC.

=== Hosted GCC Cross-Compiler ===
{{Main|Hosted GCC Cross-Compiler}}

This tutorial finishes your OS Specific Toolchain by building it with user-space support.

=== Porting GCC ===
{{Main|Porting GCC to your OS}}

You will most certainly want to port GCC if you wish to go self-hosting. It's a bunch of work, but it's not that terribly bad when your operating system becomes ready.

== GCC on other systems ==
GCC comes installed by default on virtually every Unix system. It has also been ported to Windows and other operating systems:

* [[MinGW]] is a port of the GNU environment for Windows.
* [[Cygwin]] is the choice of most forum members under Windows, hence it is likely to be easier to get help.
* [[DJGPP]] is an old port of GCC to DOS and can be used under Windows. Using DJGPP is discouraged.

== See Also ==

=== Articles ===
* [[LCC]]
* [[Tool Comparison]]
* [[Inline Assembly]]
* [[libgcc]]

=== Threads ===

=== External Links ===
* [https://www.fsf.org/ Free Software Foundation]
* [[Wikipedia:GNU Compiler Collection|GNU Compiler Collection]] on Wikipedia
* [https://gcc.gnu.org/ GCC Homepage]

[[Category:Compilers]]
[[de:GNU_Compiler_Collection]]
````

<a id="w001484"></a>

## W001484 — BIOS

Bron-ID: W001484
Namespace: Main (0)
Revisie: 30975 | Datum: 2026-09-27T17:55:31Z
Bron: https://wiki.osdev.org/BIOS
Revisie-URL: https://wiki.osdev.org/index.php?title=BIOS&oldid=30975

````text
'''BIOS''' (Basic Input/Output System) is a system firmware, which is pre-installed on the computer's motherboard, that performs the hardware initialization during the booting process, and loads the [[Bootloader]], offering generalized low-level services to the PC system programmers.
During the DOS era, the operating systems used BIOS services as the standard interface to interact with keyboard, display, disks, and other I/O devices. The more modern operating systems typically only use BIOS to load the kernel, which then uses its own device drivers, instead of the ones of the BIOS.

These BIOS services are often named "BIOS functions".
In [[Real Mode]], they can be easily accessed through [[software interrupts]], using [[Assembly|assembly]] language.

The original BIOS firmware was proprietary to the IBM PC, and had to be reverse-engineered by the other manufacturers, becoming a de facto standard, and was then standardized around mid-1990s. As the result, many BIOSes (especially in the earlier hardware, and even the older versions of firmware used in x86 emulators and VMs) contain quirks, requiring the bootloaders to have many workarounds to be bootable on all the systems. This, among with the clunky BIOS interface and other problems, is the reason for the [[Real Mode OS Warning]] and the usual recommendation to start with an existing [[Bootloader]] (which doesn't in any way stop the OS developer with writing their own later on) when starting out with the OSDev.

The BIOS has now been replaced with [[UEFI]], which is universally available on x86 starting with late 2000s/early 2010s hardware (since it is required by Microsoft to have Windows 8 and newer pre-installed). Compared to BIOS, UEFI provides more functionality/has fewer limitations, has a more convenient interface, offers better security, and provides faster start-up times. On some earlier (class 1-2) UEFI machines, the BIOS operating systems might still be loaded through CSM (Compatibility Support Module). However, a lot of newer (UEFI class 3) hardware (notably, produced in the 2020s) has started to drop the compatibility support, and only allow booting natively with UEFI.

==BIOS functions==

To access a BIOS function, you generally set the AH [[CPU Registers x86|CPU register]] (or AX, or EAX for some functions) to a particular value, and then do
an INT opcode. The value in AH (or AX, or EAX), combined with the particular interrupt number selected requests a specific
BIOS function. (Other CPU registers hold any "arguments" to the function, and often the return values from the function, also.)

It is simplest to create a listing of BIOS functions by specifying the interrupt number, and the value
of AH (or AX, or EAX) that selects the function. It is also easiest to refer to particular BIOS functions this way in discussions. For example,
INT 0x13, AH=0 is a BIOS function that resets hard disks or floppy disks.

Note: the INT and AH values are always listed in hexadecimal notation. Accidentally using a decimal value in an INT opcode
is a very common source of errors when using BIOS functions.

To an extent, the BIOS functions are organized by interrupt number:
* INT 0x10 = video display functions (including [[VESA|VESA/VBE]])
* INT 0x13 = mass storage (disk, floppy, CDROM) access
* INT 0x14 = [[Serial Ports|serial port]] functions
* INT 0x15 = memory size, miscellaneous system functions
* INT 0x16 = keyboard functions
* INT 0x17 = [[Parallel port|parallel port]] functions
* INT 0x1A = [[RTC]], [[PCI]], ...

The exhaustive list of BIOS functions is available from [[RBIL]].

Unfortunately, the PC industry has never been good about maintaining standards. So each PC manufacturer and each BIOS
manufacturer randomly made up new BIOS functions. It is also possible to "hook" any of these interrupts, and insert extra
functions that mimic BIOS functions. Early PC hardware and software manufacturers did this often. So there ended up being literally
thousands of BIOS functions (or mimics). The RBIL list is enormous, and is mostly filled with functions that only
work when combined with some completely obsolete computer, BIOS, or piece of hardware or software.

===Common functions===

Unfortunately, the [[RBIL]] does not clearly indicate which BIOS functions are "generic" (in some sense). That is, the ones that are always available,
and that everyone uses. Partially this is because the "standard" BIOS functions grew over time, so if you go back far enough in time you can
usually find a computer that does not support almost any specific BIOS function.
But there is definitely a set that is commonly used in most current OSes.

====INT 0x10====
{| class="wikitable"
|-
! Function
! Description
|-
| AH = 0x0
| Set video mode
|-
| AH = 0x1
| Set up the cursor
|-
| AH = 0x2
| Set cursor position
|-
| AH = 0x3
| Get cursor position
|-
| AH = 0xE
| Display character
|-
| AH = 0xF
| Get current video page and mode
|-
| AH = 0x12
| Detect EGA/[[VGA]]
|-
| AH = 0x13
| Display string
|-
| AX = 0x1110
| Set 8x8 [[VGA_Fonts|font]]
|-
| AX = 0x1130
| Get 8x8 [[VGA_Fonts|font]]
|-
| AX = 0x1200
| Alternate print screen
|-
| AX = 0x1201
| Turn off cursor emulation
|-
| AX = 0x4F00
| VESA get video information
|-
| AX = 0x4F01
| VESA get mode information call
|-
| AX = 0x4F02
| Select VESA video modes
|-
| AX = 0x4F0A
| VESA 2.0 protected mode interface
|}

====INT 0x13====
{| class="wikitable"
|-
! Function
! Description
|-
| AH = 0x0
| Reset floppy/hard disk
|-
| AH = 0x2
| Read floppy/hard disk in CHS mode
|-
| AH = 0x3
| Write floppy/hard disk in CHS mode
|-
| AH = 0x15
| Detect second disk
|-
| AH = 0x41
| Test presence of the [[Disk access using the BIOS (INT 13h)#LBA_in_Extended_Mode|BIOS Enhanced Disk Drive Services (EDD)]]
|-
| AH = 0x42
| Read to disk in [[LBA]] mode
|-
| AH = 0x43
| Write from disk in LBA mode
|}
(see [[ATA_in_x86_RealMode_(BIOS)|ATA using BIOS]] for more detail on these BIOS function calls)

====INT 0x14====
{| class="wikitable"
|-
! Function
! Description
|-
| AH = 0x0
| Initialize serial port
|-
| AH = 0x1
| Write byte to serial port
|-
| AH = 0x2
| Read byte from serial port
|-
| AH = 0x3
| Get serial port status
|}

====INT 0x15====
{| class="wikitable"
|-
! Function
! Description
|-
| AH = 0x86
| Delay for a microsecond interval
|-
| AH = 0x87
| Copy data to extended memory
|-
| AH = 0xC0
| Detect MCA bus
|-
| AX = 0x2400
| Disable [[A20 Line|A20 gate]]
|-
| AX = 0x2401
| Enable A20 gate
|-
| AX = 0x2402
| Get A20 gate status
|-
| AX = 0x0530
| Detect [[APM]] BIOS
|-
| AX = 0x5300
| APM detect
|-
| AX = 0x5303
| APM connect using 32 bit
|-
| AX = 0x5304
| APM disconnect
|-
| EAX = 0xE820
| Get complete memory map
|-
| AX = 0xE801
| Get contiguous memory size
|-
| AX = 0xE881
| Get contiguous memory size
|-
| AH = 0x88
| Get contiguous memory size
|-
| AH = 0x89
| Switch to [[protected mode]]
|}

(see [[Detecting Memory (x86)]] for more detail on the memory size related calls)

====INT 0x16====
{| class="wikitable"
|-
! Function
! Description
|-
| AH = 0x0
| Read keyboard scancode and character (blocking)
|-
| AH = 0x1
| Read keyboard scancode and character (non-blocking)
|-
| AH = 0x2
| Get modifier keys status
|-
| AH = 0x3
| Set keyboard repeat rate
|}

====INT 0x1A====
{| class="wikitable"
|-
! Function
! Description
|-
| AX = 0xB120
| Find PCI device
|-
| AX = 0xB103
| Find PCI class code
|-
| AX = 0xB106
| PCI bus operations
|-
| AX = 0xB108
| PCI read configuration byte
|-
| AX = 0xB109
| PCI read configuration word
|-
| AX = 0xB108
| PCI read configuration dword
|}

====Others====
* INT 0x11 -- Hardware detection
* INT 0x12 -- Get low memory size
* INT 0x17 -- [[Parallel port]] ("printer") related functions
* INT 0x18 -- Return to BIOS (this usually makes the BIOS attempt to boot from the next device)

===Assembly notes===

Each BIOS function (as described in RBIL) has a specific set of "result" registers. Beyond those listed registers,
the BIOS functions are supposed to perfectly preserve all the other register values. Early versions of [[Bochs]] (below 2.3)
had a small problem with this. The lower halves of all the 32bit extended registers (ie. EBX, ECX) were preserved properly, but
the upper words of some of the registers got trashed.

The BIOS functions themselves should never crash. On any error they will:
* almost always set the carry flag (test with JC),
* sometimes return "AH = 0x86 (unsupported function)",
* sometimes return "AH = 0x80 (invalid command)"
* or (for seriously buggy BIOSes) return with nothing changed.

Try to always test these error returns, because in many circumstances the BIOS functions might appear to be returning valid
(but very wrong) data -- rather than an error code.

==BIOS in Protected Mode==

Unfortunately, in [[Protected mode]], almost all BIOS functions become unavailable, and trying to call them nonetheless will result in exceptions or
unreliable responses (because of the different way '''segment''' values are handled). Some newer services however (such as [[SMBIOS]], [[PCI]], [[PnP]], or [[VBE]])
offer an interface that is compatible with 32-bit Protected Mode, known as [[BIOS32]].

If you must use Real Mode BIOS functions after the CPU has been switched into Protected Mode, then see [[Virtual 8086 Mode]],
or perhaps exit Protected Mode, and momentarily return to [[Real Mode]]. Both methods have caveats, therefore any calls to the BIOS should be done before any physical device is programmed by your code:

* BIOS calls may use interrupts, which means that you need to forward IRQs or map the PIC back to its original configuration.
* BIOS calls may access devices that you have already configured - notably the [[PIT]] and [[PIC]]
* BIOS calls can enter protected mode on their own to access MMIO registers, which is beyond the limits of virtual 8086 mode.
* In real mode, you have no way of managing interrupts and your drivers may get stuck for interrupts being lost.
* In real mode, you have no control over time, performance and security guarantees.

The only device that's mostly exempt from these problems is the Video BIOS, which is not generally bundled with your motherboard and therefore can't rely on BIOS services either. Most current OSes - commercial and hobbyist alike - use a v8086 monitor or emulator to support graphics devices without a native driver so many BIOSes have been tested against such a set-up.

==BIOS in Long Mode==

Just like in [[Protected Mode]], BIOS functions are unavailable in [[Long Mode]] too. Unfortunately there's no [[Virtual 8086 Mode]] to come to the rescue. It is necessary to momentarily switch to [[Real Mode]], or to emulate a CPU and interpret opcodes by software. All necessary information for the latter approach can be found in Intel and AMD documentation.

==Additional Information from the BIOS==

Most of the useful information you get from the BIOS will come from calling [[BIOS routines]], however you cannot use them in [[Protected Mode]] nor [[Long Mode]] without switching momentarily in [[Real Mode]].

Some of the BIOS detection/state result is stored in the [[Memory Map (x86)#BIOS Data Area .28BDA.29|BIOS Data Area]].

Additional information is kept in the [[CMOS]] chip, such as the [[Boot Sequence|Boot order]].

== Compatibility on Modern Machines ==
BIOS has been replaced by [[UEFI]] on modern [[x86]] machines starting from the 2010s, whose UEFI firmware kept supporting the "legacy" BIOS functions through an emulation layer, CSM.

However, Intel announced the removal of the Compatibility Support Module (CSM) by 2020 to completely phase out legacy [[Real Mode]] (also known as 16-bit) [[BIOS]] code and force a transition to a far more standardized [[UEFI]] implementation. Though many AMD systems still support it, and in fact you can bring back CSM via a UEFI application called [https://github.com/CSMWrap/CSMWrap CSMWrap].

== See Also==

===Articles ===
*[[BIOS32]]
*[[DMI]]
*[[Real Mode OS Warning]]

===Threads===
*[[Topic:23125|ASM example code about how to call BIOS function from Long Mode (by switching into RM and back again)]]

===External Links===
*[[wikipedia:BIOS | BIOS on Wikipedia]]
*[https://www.ctyme.com/intr/int.htm Ralph Brown's interrupt list online]
*[https://archive.org/details/bitsavers_ibmpcps2PSTechnicalReferenceApr87_5816663 1987 IBM PS and PS/2 BIOS Interface Technical Reference]

[[Category:X86]]
[[Category:BIOS]]
[[Category:Firmware]]
[[de:Basic Input Output System]]
````

<a id="w001485"></a>

## W001485 — PCI

Bron-ID: W001485
Namespace: Main (0)
Revisie: 30693 | Datum: 2026-06-05T06:45:16Z
Bron: https://wiki.osdev.org/PCI
Revisie-URL: https://wiki.osdev.org/index.php?title=PCI&oldid=30693

````text
== The PCI Bus ==

The PCI ([https://en.wikipedia.org/wiki/Conventional_PCI Peripheral Component Interconnect]) bus was defined to establish a high performance and low cost local bus that would remain through several generations of products. By combining a transparent upgrade path from 132 MB/s (32-bit at 33 MHz) to 528 MB/s (64-bit at 66 MHz) and both 5 volt and 3.3 volt signalling environments, the PCI bus meets the needs of both low end desktop systems and high-end LAN servers. The PCI bus component and add-in card interface is processor independent, enabling an efficient transition to future processors, as well as use with multiple processor architectures. The disadvantage of the PCI bus is the limited number of electrical loads it can drive. A single PCI bus can drive a maximum of 10 loads. (Remember when counting the number of loads on the bus, a connector counts as one load and the PCI device counts as another, and sometimes two.)

== Configuration Space ==

The PCI specification provides for totally software driven initialization and configuration of each device (or target) on the PCI Bus via a separate Configuration Address Space. All PCI devices, except host bus bridges, are required to provide 256 bytes of configuration registers for this purpose.

Configuration read/write cycles are used to access the Configuration Space of each target device. A target is selected during a configuration access when its IDSEL signal is asserted. The IDSEL acts as the classic "chip select" signal. During the address phase of the configuration cycle, the processor can address one of 64 32-bit registers within the configuration space by placing the required register number on address lines 2 through 7 (AD<nowiki>[7..2]</nowiki>) and the byte enable lines.

PCI devices are inherently little-endian, meaning all multiple byte fields have the least significant values at the lower addresses. This requires a big-endian processor, such as a Power PC, to perform the proper byte-swapping of data read from or written to the PCI device, including any accesses to the Configuration Address Space.

Systems must provide a mechanism that allows access to the PCI configuration space, as most CPUs do not have any such mechanism. This task is usually performed by the Host to PCI Bridge (Host Bridge). Two distinct mechanisms are defined to allow the software to generate the required configuration accesses. Configuration mechanism #1 is the preferred method, while mechanism #2 is provided for backwards compatibility. Only configuration mechanism #1 will be described here, as it is the only access mechanism that will be used in the future.

=== Configuration Space Access Mechanism #1 ===

Two 32-bit I/O locations are used, the first location (<code>0xCF8</code>) is named CONFIG_ADDRESS, and the second (<code>0xCFC</code>) is called CONFIG_DATA. CONFIG_ADDRESS specifies the configuration address that is required to be accesses, while accesses to CONFIG_DATA will actually generate the configuration access and will transfer the data to or from the CONFIG_DATA register.

The CONFIG_ADDRESS is a 32-bit register with the format shown in following figure. Bit 31 is an enable flag for determining when accesses to CONFIG_DATA should be translated to configuration cycles. Bits 23 through 16 allow the configuration software to choose a specific PCI bus in the system. Bits 15 through 11 select the specific device on the PCI Bus. Bits 10 through 8 choose a specific function in a device (if the device supports multiple functions). 

The least significant byte selects the offset into the 256-byte configuration space available through this method. Since all reads and writes must be both 32-bits and aligned to work on all implementations, the two lowest bits of CONFIG_ADDRESS must always be zero, with the remaining six bits allowing you to choose each of the 64 32-bit words. If you don't need all 32 bits, you'll have to perform the unaligned access in software by aligning the address, followed by masking and shifting the answer.

{| {{wikitable}}
|-
! Bit 31
! Bits 30-24
! Bits 23-16
! Bits 15-11
! Bits 10-8
! Bits 7-0
|-
| Enable Bit
| Reserved
| Bus Number
| Device Number
| Function Number
| Register Offset<sup>1</sup>
|}
<sup>1</sup> Register Offset has to point to consecutive DWORDs, ie. bits 1:0 are always 0b00 (they are still part of the Register Offset).

The following code segment illustrates the use of configuration mechanism #1 to read 16-bit fields from configuration space. Note that this segment, the outl(port, value) and inl(port) functions refer to the OUTL and INL Pentium assembly language instructions. 

<syntaxhighlight lang="c">
uint16_t pciConfigReadWord(uint8_t bus, uint8_t slot, uint8_t func, uint8_t offset) {
    uint32_t address;
    uint32_t lbus  = (uint32_t)bus;
    uint32_t lslot = (uint32_t)slot;
    uint32_t lfunc = (uint32_t)func;
    uint16_t tmp = 0;
  
    // Create configuration address as per Figure 1
    address = (uint32_t)((lbus << 16) | (lslot << 11) |
              (lfunc << 8) | (offset & 0xFC) | ((uint32_t)0x80000000));
  
    // Write out the address
    outl(0xCF8, address);
    // Read in the data
    // (offset & 2) * 8) = 0 will choose the first word of the 32-bit register
    tmp = (uint16_t)((inl(0xCFC) >> ((offset & 2) * 8)) & 0xFFFF);
    return tmp;
}
</syntaxhighlight>

When a configuration access attempts to select a device that does not exist, the host bridge will complete the access without error, dropping all data on writes and returning all ones on reads. The following code segment illustrates the read of a non-existent device.

<syntaxhighlight lang="c">
uint16_t pciCheckVendor(uint8_t bus, uint8_t slot) {
    uint16_t vendor, device;
    /* Try and read the first configuration register. Since there are no
     * vendors that == 0xFFFF, it must be a non-existent device. */
    if ((vendor = pciConfigReadWord(bus, slot, 0, 0)) != 0xFFFF) {
       device = pciConfigReadWord(bus, slot, 0, 2);
       . . .
    } return (vendor);
}
</syntaxhighlight>

=== Configuration Space Access Mechanism #2 ===

This configuration space access mechanism was deprecated in PCI version 2.0. This means it's only likely to exist on hardware from around 1992 (when PCI 1.0 was introduced) to 1993 (when PCI 2.0 was introduced), which limits it to 80486 and early Pentium motherboards.

For access mechanism #2, the IO port at <code>0xCF8</code> is an 8-bit port and is used to enable/disable the access mechanism and set the function number. It has the following format:

{| {{wikitable}}
|-
! Bits 7-4
! Bits 3-1
! Bit 0
|-
| Key (0 = access mechanism disabled, non-zero = access mechanism enabled)
| Function number
| Special cycle enabled if set
|}

The IO port at <code>0xCFA</code> (the "Forwarding Register") is also an 8-bit port, and is used to set the bus number for subsequent PCI configuration space accesses.

Once the access mechanism has been enabled; accesses to IO ports <code>0xC000</code> to <code>0xCFFF</code> are used to access PCI configuration space. The IO port number has the following format:

{| {{wikitable}}
|-
! Bits 15-12
! Bits 11-8
! Bits 7-2
! Bits 1-0
|-
| Must be 1100b
| Device number
| Register index
| Must be zero
|}

Note that this limits the system to 16 devices per PCI bus.

=== Memory Mapped PCI Configuration Space Access ===

PCI Express introduced a new way to access PCI configuration space, where it's simply memory mapped and no IO ports are used. This access mechanism is described in [[PCI Express]].

Note that systems that do provide the memory mapped access mechanism are also required to support PCI access mechanism #1 for backwards compatibility.

=== Detecting Configuration Space Access Mechanism/s ===

In general there are 4 cases:
* Computer doesn't support PCI (either the computer is too old, or your OS is being run at some time in the future after PCI has been superseded)
* Computer supports mechanism #2
* Computer supports mechanism #1 but doesn't support the memory mapped access mechanism
* Computer supports both mechanism #1 and the memory mapped access mechanism

For BIOS systems, <code>int 0x1A, AX=0xB101</code> will tell you if the system uses mechanism #1 or mechanism #2. If this function doesn't exist you can't be sure if the computer supports PCI or not. If it says mechanism #1 is supported you won't know if the memory mapped access mechanism is also supported or not.

For UEFI systems, it's extremely safe to assume that mechanism #2 is not supported; and you can test to see if the computer supports PCI or not by checking to see if the "PCI bus support" protocol exists. If PCI is supported, there's no easy way to determine if (e.g.) the computer supports mechanism #1 or not.

For both BIOS and UEFI systems, you can check the ACPI tables to determine if the memory mapped access mechanism is supported.

This leaves a few cases uncovered (e.g. where you don't know if whether mechanism #1 or #2 are supported despite trying all of the above). For these cases the only option left is manual probing. This means 2 specific tests - whether mechanism #1 is supported, and if not whether mechanism #2 is supported. Please note that manual probing has risks; in that if there is no PCI (e.g. the system only has ISA) the IO port accesses might cause undefined behaviour (especially on systems where the ISA bus ignores highest 6 bits of the IO port address, where accessing IO port <code>0xCF8</code> is the same as accessing IO port <code>0xF8</code>).

=== PCI Device Structure ===

The PCI Specification defines the organization of the 256-byte Configuration Space registers and imposes a specific template for the space. Figures 2 & 3 show the layout of the 256-byte Configuration space. All PCI compliant devices must support the Vendor ID, Device ID, Command and Status, Revision ID, Class Code and Header Type fields. Implementation of the other registers is optional, depending upon the devices functionality.

==== Common Header Fields ====

The following field descriptions are common to all Header Types:

{| {{wikitable}}
|-
! Register !! Offset !! Bits 31-24 !! Bits 23-16 !! Bits 15-8 !! Bits 7-0
|-
| 0x0
| 0x0
| colspan="2" | Device ID
| colspan="2" | Vendor ID
|-
| 0x1
| 0x4
| colspan="2" | Status
| colspan="2" | Command
|-
| 0x2
| 0x8
| Class code || Subclass || Prog IF || Revision ID
|-
| 0x3
| 0xC
| BIST
| Header type
| Latency Timer
| Cache Line Size
|-
| ...
|}

* ''Device ID:'' Identifies the particular device. Where valid IDs are allocated by the vendor.
* ''Vendor ID:'' Identifies the manufacturer of the device. Where valid IDs are allocated by PCI-SIG (the list is [https://pcisig.com/membership/member-companies here]) to ensure uniqueness and <code>0xFFFF</code> is an invalid value that will be returned on read accesses to Configuration Space registers of non-existent devices.
* ''[[#Status_Register|Status]]:'' A register used to record status information for PCI bus related events.
* ''[[#Command_Register|Command]]:'' Provides control over a device's ability to generate and respond to PCI cycles. Where the only functionality guaranteed to be supported by all devices is, when a 0 is written to this register, the device is disconnected from the PCI bus for all accesses except Configuration Space access.
* ''Class Code:'' A read-only register that specifies the type of function the device performs.
* ''Subclass:'' A read-only register that specifies the specific function the device performs.
* ''Prog IF(Programming Interface Byte):'' A read-only register that specifies a register-level programming interface the device has, if it has any at all.
* ''Revision ID:'' Specifies a revision identifier for a particular device. Where valid IDs are allocated by the vendor.
* ''BIST:'' Represents that status and allows control of a devices BIST (built-in self test).
* ''Header Type:'' Identifies the layout of the rest of the header beginning at byte <code>0x10</code> of the header. If bit 7 of this register is set, the device has multiple functions; otherwise, it is a single function device. Types:
** <code>0x0</code>: [[#Header_Type_0x0|a general device]]
** <code>0x1</code>: [[#Header Type_0x1_(PCI-to-PCI_bridge)|a PCI-to-PCI bridge]]
** <code>0x2</code>: [[#Header_Type_0x2_(PCI-to-CardBus_bridge)|a PCI-to-CardBus bridge]]. 
* ''Latency Timer:'' Specifies the latency timer in units of PCI bus clocks.
* ''Cache Line Size:'' Specifies the system cache line size in 32-bit units. A device can limit the number of cacheline sizes it can support, if a unsupported value is written to this field, the device will behave as if a value of 0 was written.

Remember that the PCI devices follow little ENDIAN ordering. The lower addresses contain the least significant portions of the field. Software to manipulate this structure must take particular care that the endian-ordering follows the PCI devices, not the CPUs.

===== Command Register =====

Here is the layout of the Command register:

{| {{wikitable}}
|-
! Bits 11-15 !! Bit 10 !! Bit 9 !! Bit 8 !! Bit 7 !! Bit 6 !! Bit 5 !! Bit 4 !! Bit 3 !! Bit 2 !! Bit 1 !! Bit 0
|-
| Reserved
| Interrupt Disable
| Fast Back-to-Back Enable
| SERR# Enable
| Reserved
| Parity Error Response
| VGA Palette Snoop
| Memory Write and Invalidate Enable
| Special Cycles
| Bus Master
| Memory Space
| I/O Space
|-
|
| RW
| RO
| RW
| RO
| RW
| RO
| RO
| RO
| RW
| RW
| RW
|}

* ''Interrupt Disable'' - If set to 1 the assertion of the devices INTx# signal is disabled; otherwise, assertion of the signal is enabled.
* ''Fast Back-Back Enable'' - If set to 1 indicates a device is allowed to generate fast back-to-back transactions; otherwise, fast back-to-back transactions are only allowed to the same agent.
* ''SERR# Enable'' - If set to 1 the SERR# driver is enabled; otherwise, the driver is disabled.
* ''Bit 7'' - As of revision 3.0 of the PCI local bus specification this bit is hardwired to 0. In earlier versions of the specification this bit was used by devices and may have been hardwired to 0, 1, or implemented as a read/write bit.
* ''Parity Error Response'' - If set to 1 the device will take its normal action when a parity error is detected; otherwise, when an error is detected, the device will set bit 15 of the Status register (Detected Parity Error Status Bit), but will not assert the PERR# (Parity Error) pin and will continue operation as normal.
* ''VGA Palette Snoop'' - If set to 1 the device does not respond to palette register writes and will snoop the data; otherwise, the device will trate palette write accesses like all other accesses.
* ''Memory Write and Invalidate Enable'' - If set to 1 the device can generate the Memory Write and Invalidate command; otherwise, the Memory Write command must be used.
* ''Special Cycles'' - If set to 1 the device can monitor Special Cycle operations; otherwise, the device will ignore them.
* ''Bus Master'' - If set to 1 the device can behave as a bus master; otherwise, the device can not generate PCI accesses.
* ''Memory Space'' - If set to 1 the device can respond to Memory Space accesses; otherwise, the device's response is disabled.
* ''I/O Space'' - If set to 1 the device can respond to I/O Space accesses; otherwise, the device's response is disabled.

If the kernel configures the BARs of the devices, the kernel also have to enable bits 0 and 1 for it to activate.

===== Status Register =====

Here is the layout of the Status register:

{| {{wikitable}}
|-
! Bit 15 !! Bit 14 !! Bit 13 !! Bit 12 !! Bit 11 !! Bits 9-10 !! Bit 8 !! Bit 7 !! Bit 6 !! Bit 5 !! Bit 4 !! Bit 3 !! Bits 0-2
|-
| Detected Parity Error
| Signaled System Error
| Received Master Abort
| Received Target Abort
| Signaled Target Abort
| DEVSEL Timing
| Master Data Parity Error
| Fast Back-to-Back Capable
| Reserved
| 66 MHz Capable
| Capabilities List
| Interrupt Status
| Reserved
|-
| RW1C
| RW1C
| RW1C
| RW1C
| RW1C
| RO
| RW1C
| RO
| RO
| RO
| RO
| RO
|
|}

* ''Detected Parity Error'' - This bit will be set to 1 whenever the device detects a parity error, even if parity error handling is disabled.
* ''Signalled System Error'' - This bit will be set to 1 whenever the device asserts SERR#.
* ''Received Master Abort'' - This bit will be set to 1, by a master device, whenever its transaction (except for Special Cycle transactions) is terminated with Master-Abort.
* ''Received Target Abort'' - This bit will be set to 1, by a master device, whenever its transaction is terminated with Target-Abort.
* ''Signalled Target Abort'' - This bit will be set to 1 whenever a target device terminates a transaction with Target-Abort.
* ''DEVSEL Timing'' - Read only bits that represent the slowest time that a device will assert DEVSEL# for any bus command except Configuration Space read and writes. Where a value of <code>0x0</code> represents fast timing, a value of <code>0x1</code> represents medium timing, and a value of <code>0x2</code> represents slow timing.
* ''Master Data Parity Error'' - This bit is only set when the following conditions are met. The bus agent asserted PERR# on a read or observed an assertion of PERR# on a write, the agent setting the bit acted as the bus master for the operation in which the error occurred, and bit 6 of the Command register (Parity Error Response bit) is set to 1.
* ''Fast Back-to-Back Capable'' - If set to 1 the device can accept fast back-to-back transactions that are not from the same agent; otherwise, transactions can only be accepted from the same agent.
* ''Bit 6'' - As of revision 3.0 of the PCI Local Bus specification this bit is reserved. In revision 2.1 of the specification this bit was used to indicate whether or not a device supported User Definable Features.
* ''66 MHz Capable'' - If set to 1 the device is capable of running at 66 MHz; otherwise, the device runs at 33 MHz.
* ''Capabilities List'' - If set to 1 the device implements the pointer for a New Capabilities Linked list at offset <code>0x34</code>; otherwise, the linked list is not available.
* ''Interrupt Status'' - Represents the state of the device's INTx# signal. If set to 1 and bit 10 of the Command register (Interrupt Disable bit) is set to 0 the signal will be asserted; otherwise, the signal will be ignored.

==== Header Type 0x0 ====

This table is applicable if the Header Type is <code>0x0</code>. (Figure 2)
{| {{wikitable}}
|-
! Register !! Offset !! Bits 31-24 !! Bits 23-16 !! Bits 15-8 !! Bits 7-0
|-
| 0x0
| 0x0
| colspan="2" | Device ID
| colspan="2" | Vendor ID
|-
| 0x1
| 0x4
| colspan="2" | Status
| colspan="2" | Command
|-
| 0x2
| 0x8
| Class code || Subclass || Prog IF || Revision ID
|-
| 0x3
| 0xC
| BIST
| Header type
| Latency Timer
| Cache Line Size
|-
| 0x4
| 0x10
| colspan="4" | Base address #0 (BAR0)
|-
| 0x5
| 0x14
| colspan="4" | Base address #1 (BAR1)
|-
| 0x6
| 0x18
| colspan="4" | Base address #2 (BAR2)
|-
| 0x7
| 0x1C
| colspan="4" | Base address #3 (BAR3)
|-
| 0x8
| 0x20
| colspan="4" | Base address #4 (BAR4)
|-
| 0x9
| 0x24
| colspan="4" | Base address #5 (BAR5)
|-
| 0xA
| 0x28
| colspan="4" | Cardbus CIS Pointer
|-
| 0xB
| 0x2C
| colspan="2" | Subsystem ID
| colspan="2" | Subsystem Vendor ID
|-
| 0xC
| 0x30
| colspan="4" | Expansion ROM base address
|-
| 0xD
| 0x34
| colspan="3" | Reserved
| colspace="1" | Capabilities Pointer
|-
| 0xE
| 0x38
| colspan="4" | Reserved
|-
| 0xF
| 0x3C
| Max latency || Min Grant || Interrupt PIN || Interrupt Line 
|}

The following field descriptions apply if the Header Type is <code>0x0</code>:

* ''CardBus CIS Pointer:'' Points to the Card Information Structure and is used by devices that share silicon between CardBus and PCI.

* ''Interrupt Line:'' Specifies which input of the system interrupt controllers the device's interrupt pin is connected to and is implemented by any device that makes use of an interrupt pin. For the x86 architecture this register corresponds to the PIC IRQ numbers 0-15 (and not I/O APIC IRQ numbers) and a value of <code>0xFF</code> defines no connection.

* ''Interrupt Pin:'' Specifies which interrupt pin the device uses. Where a value of <code>0x1</code> is INTA#, <code>0x2</code> is INTB#, <code>0x3</code> is INTC#, <code>0x4</code> is INTD#, and <code>0x0</code> means the device does not use an interrupt pin.

* ''Max Latency:'' A read-only register that specifies how often the device needs access to the PCI bus (in 1/4 microsecond units).

* ''Min Grant:'' A read-only register that specifies the burst period length, in 1/4 microsecond units, that the device needs (assuming a 33 MHz clock rate).

* ''Capabilities Pointer:'' Points (i.e. an offset into this function's configuration space) to a linked list of new capabilities implemented by the device. Used if bit 4 of the status register (Capabilities List bit) is set to 1. The bottom two bits are reserved and should be masked before the Pointer is used to access the Configuration Space.

==== Header Type 0x1 (PCI-to-PCI bridge) ====

This table is applicable if the Header Type is <code>0x1</code> (PCI-to-PCI bridge) (Figure 3)

{| {{wikitable}}
|-
! Register !! Offset !! Bits 31-24 !! Bits 23-16 !! Bits 15-8 !! Bits 7-0
|-
| 0x0
| 0x0
| colspan="2" | Device ID
| colspan="2" | Vendor ID
|-
| 0x1
| 0x4
| colspan="2" | Status
| colspan="2" | Command
|-
| 0x2
| 0x8
| Class code || Subclass || Prog IF || Revision ID
|-
| 0x3
| 0xC
| BIST
| Header type
| Latency Timer
| Cache Line Size
|-
| 0x4
| 0x10
| colspan="4" | Base address #0 (BAR0)
|-
| 0x5
| 0x14
| colspan="4" | Base address #1 (BAR1)
|-
| 0x6
| 0x18
| Secondary Latency Timer
| Subordinate Bus Number
| Secondary Bus Number
| Primary Bus Number
|-
| 0x7
| 0x1C
| colspan="2" | Secondary Status
| I/O Limit
| I/O Base
|-
| 0x8
| 0x20
| colspan="2" | Memory Limit
| colspan="2" | Memory Base
|-
| 0x9
| 0x24
| colspan="2" | Prefetchable Memory Limit
| colspan="2" | Prefetchable Memory Base
|-
| 0xA
| 0x28
| colspan="4" | Prefetchable Base Upper 32 Bits
|-
| 0xB
| 0x2C
| colspan="4" | Prefetchable Limit Upper 32 Bits
|-
| 0xC
| 0x30
| colspan="2" | I/O Limit Upper 16 Bits
| colspan="2" | I/O Base Upper 16 Bits
|-
| 0xD
| 0x34
| colspan="3" | Reserved
| Capability Pointer
|-
| 0xE
| 0x38
| colspan="4" | Expansion ROM base address
|-
| 0xF
| 0x3C
| colspan="2" | Bridge Control || Interrupt PIN || Interrupt Line 
|}

===== Header Type Register =====

Here is the layout of the Header Type register:

{| {{wikitable}}
|-
! Bit 7 !! Bits 6-0
|-
| MF
|| Header Type     
|}

* ''MF'' - If MF = 1 Then this device has multiple functions.
* ''Header Type'' - <code>0x0</code> Standard Header - <code>0x1</code> PCI-to-PCI Bridge - <code>0x2</code> CardBus Bridge

===== BIST Register =====

Here is the layout of the BIST register:

{| {{wikitable}}
|-
! Bit 7 !! Bit 6 !! Bits 4-5 !! Bits 0-3
|-
| BIST Capable || Start BIST || Reserved || Completion Code
|}

* ''BIST Capable'' - Will return 1 the device supports BIST.
* ''Start BIST'' - When set to 1 the BIST is invoked. This bit is reset when BIST completes. If BIST does not complete after 2 seconds the device should be failed by system software.
* ''Completion Code'' - Will return 0, after BIST execution, if the test completed successfully.

==== Header Type 0x2 (PCI-to-CardBus bridge) ====

This table is applicable if the Header Type is <code>0x2</code> (PCI-to-CardBus bridge)

{| {{wikitable}}
|-
! Register !! Offset !! Bits 31-24 !! Bits 23-16 !! Bits 15-8 !! Bits 7-0
|-
| 0x0
| 0x0
| colspan="2" | Device ID
| colspan="2" | Vendor ID
|-
| 0x1
| 0x4
| colspan="2" | Status
| colspan="2" | Command
|-
| 0x2
| 0x8
| Class code || Subclass || Prog IF || Revision ID
|-
| 0x3
| 0xC
| BIST
| Header type
| Latency Timer
| Cache Line Size
|-
| 0x4
| 0x10
| colspan="4" | CardBus Socket/ExCa base address
|-
| 0x5
| 0x14
| colspan="2" | Secondary status
| Reserved
| Offset of capabilities list
|-
| 0x6
| 0x18
| CardBus latency timer
| Subordinate bus number
| CardBus bus number
| PCI bus number
|-
| 0x7
| 0x1C
| colspan="4" | Memory Base Address 0
|-
| 0x8
| 0x20
| colspan="4" | Memory Limit 0
|-
| 0x9
| 0x24
| colspan="4" | Memory Base Address 1
|-
| 0xA
| 0x28
| colspan="4" | Memory Limit 1
|-
| 0xB
| 0x2C
| colspan="4" | I/O Base Address 0
|-
| 0xC
| 0x30
| colspan="4" | I/O Limit 0
|-
| 0xD
| 0x34
| colspan="4" | I/O Base Address 1
|-
| 0xE
| 0x38
| colspan="4" | I/O Limit 1
|-
| 0xF
| 0x3C
| colspan="2" | Bridge Control
| Interrupt PIN
| Interrupt Line
|-
| 0x10
| 0x40
| colspan="2" | Subsystem Vendor ID
| colspan="2" | Subsystem Device ID
|-
| 0x11
| 0x44
| colspan="4" | 16-bit PC Card legacy mode base address
|}

=== Base Address Registers ===

Base Address Registers (or BARs) can be used to hold memory addresses used by the device, or offsets for port addresses. Typically, memory address BARs need to be located in physical ram while I/O space BARs can reside at any memory address (even beyond physical memory). To distinguish between them, you can check the value of the lowest bit. The following tables describe the two types of BARs:

{|{{wikitable}}
|+Memory Space BAR Layout 
|-
! Bits 31-4
! Bit 3
! Bits 2-1
! Bit 0
|-
| 16-Byte Aligned Base Address
| Prefetchable
| Type
| Always 0
|}

{|{{wikitable}}
|+I/O Space BAR Layout
|-
! Bits 31-2
! Bit 1
! Bit 0
|-
| 4-Byte Aligned Base Address
| Reserved
| Always 1
|}

The Type field of the Memory Space BAR Layout specifies the size of the base register and where in memory it can be mapped. If it has a value of <code>0x0</code> then the base register is 32-bits wide and can be mapped anywhere in the 32-bit Memory Space. A value of <code>0x2</code> means the base register is 64-bits wide and can be mapped anywhere in the 64-bit Memory Space (A 64-bit base address register consumes 2 of the base address registers available). A value of <code>0x1</code> is reserved as of revision 3.0 of the PCI Local Bus Specification. In earlier versions it was used to support memory space below 1MB (16-bit wide base register that can be mapped anywhere in the 16-bit Memory Space).

When a base address register is marked as Prefetchable, it means that the region does not have read side effects (reading from that memory range doesn't change any state), and it is allowed for the CPU to cache loads from that memory region and read it in bursts (typically cache line sized). Hardware is also allowed to merge repeated stores to the same address into one store of the latest value. If you are using paging and want maximum performance, you should map prefetchable MMIO regions as WT (write-through) instead of UC (uncacheable). On x86, frame buffers are the exception, they should be almost always be mapped WC (write-combining).

==== Address and size of the BAR ====

When you want to retrieve the actual base address of a BAR, be sure to mask the lower bits. For 16-bit Memory Space BARs, you calculate <code>(BAR[x] & 0xFFF0)</code>. For 32-bit Memory Space BARs, you calculate <code>(BAR[x] & 0xFFFFFFF0)</code>. For 64-bit Memory Space BARs, you calculate <code>((BAR[x] & 0xFFFFFFF0) + ((BAR[x + 1] & 0xFFFFFFFF) << 32))</code> For I/O Space BARs, you calculate <code>(BAR[x] & 0xFFFFFFFC)</code>.

Before attempting to read the information about the BAR, make sure to disable both I/O and memory decode in the command byte. You can restore the original value after completing the BAR info read. This is needed as some devices are known to decode the write of all ones to the register as an (unintended) access.

To determine the amount of address space needed by a PCI device, you must save the original value of the BAR, write a value of all 1's to the register, then read it back. The amount of memory can then be determined by masking the information bits, performing a bitwise NOT ('~' in C), and incrementing the value by 1. The original value of the BAR should then be restored. The BAR register is naturally aligned and as such you can only modify the bits that are set. For example, if a device utilizes 16 MB it will have BAR0 filled with 0xFF000000 (0x1000000 after decoding) and you can only modify the upper 8-bits. [https://web.archive.org/web/20150101180929/https://www.pcisig.com/reflector/msg05233.html]

=== Class Codes ===

The Class Code, Subclass, and Prog IF registers are used to identify the device's type, the device's function, and the device's register-level programming interface, respectively.

The following table details most of the known device types and functions:

{| {{wikitable}}
|-
! Class Code !! Subclass !! Prog IF
|-
| rowspan="2" | 0x0 - Unclassified
| 0x0 - Non-VGA-Compatible Unclassified Device || --
|- 
| 0x1 - VGA-Compatible Unclassified Device || --
|-
| rowspan="22" | 0x1 - Mass Storage Controller
| 0x0 - SCSI Bus Controller || --
|-
| rowspan="8" | 0x1 - IDE Controller
| 0x0 - ISA Compatibility mode-only controller
|-
| 0x5 - PCI native mode-only controller
|-
| 0xA - ISA Compatibility mode controller, supports both channels switched to PCI native mode
|-
| 0xF - PCI native mode controller, supports both channels switched to ISA compatibility mode
|-
| 0x80 - ISA Compatibility mode-only controller, supports bus mastering
|-
| 0x85 - PCI native mode-only controller, supports bus mastering
|-
| 0x8A - ISA Compatibility mode controller, supports both channels switched to PCI native mode, supports bus mastering
|-
| 0x8F - PCI native mode controller, supports both channels switched to ISA compatibility mode, supports bus mastering
|-
| 0x2 - Floppy Disk Controller || --
|-
| 0x3 - IPI Bus Controller || --
|-
| 0x4 - RAID Controller || --
|-
| rowspan="2" | 0x5 - ATA Controller
| 0x20 - Single DMA
|-
| 0x30 - Chained DMA
|-
| rowspan="3" | 0x6 - Serial ATA Controller
| 0x0 - Vendor Specific Interface
|-
| 0x1 - AHCI 1.0
|-
| 0x2 - Serial Storage Bus
|-
| rowspan="2" | 0x7 - Serial Attached SCSI Controller
| 0x0 - SAS
|-
| 0x1 - Serial Storage Bus
|-
| rowspan="2" | 0x8 - Non-Volatile Memory Controller
| 0x1 - NVMHCI
|-
| 0x2 - NVM Express
|-
| 0x80 - Other || --
|-
| rowspan="10" | 0x2 - Network Controller
| 0x0 - Ethernet Controller || --
|-
| 0x1 - Token Ring Controller || --
|-
| 0x2 - FDDI Controller || --
|-
| 0x3 - ATM Controller || --
|-
| 0x4 - ISDN Controller || --
|-
| 0x5 - WorldFip Controller || --
|-
| 0x6 - PICMG 2.14 Multi Computing Controller || --
|-
| 0x7 - Infiniband Controller || --
|-
| 0x8 - Fabric Controller || --
|-
| 0x80 - Other || --
|-
| rowspan="5" | 0x3 - Display Controller
| rowspan="2" | 0x0 - VGA Compatible Controller
| 0x0 - VGA Controller
|-
| 0x1 - 8514-Compatible Controller
|-
| 0x1 - XGA Controller || --
|-
| 0x2 - 3D Controller (Not VGA-Compatible) || --
|-
| 0x80 - Other || --
|-
| rowspan="5" | 0x4 - Multimedia Controller
| 0x0 - Multimedia Video Controller || --
|-
| 0x1 - Multimedia Audio Controller || --
|-
| 0x2 - Computer Telephony Device || --
|-
| 0x3 - Audio Device || --
|-
| 0x80 - Other || --
|-
| rowspan="3" | 0x5 - Memory Controller
| 0x0 - RAM Controller || --
|-
| 0x1 - Flash Controller || --
|-
| 0x80 - Other || --
|-
| rowspan="15" | 0x6 - Bridge
| 0x0 - Host Bridge || --
|-
| 0x1 - ISA Bridge || --
|-
| 0x2 - EISA Bridge || --
|-
| 0x3 - MCA Bridge || --
|-
| rowspan="2" | 0x4 - PCI-to-PCI Bridge
| 0x0 - Normal Decode
|-
| 0x1 - Subtractive Decode
|-
| 0x5 - PCMCIA Bridge || --
|-
| 0x6 - NuBus Bridge || --
|-
| 0x7 - CardBus Bridge || --
|-
| rowspan="2" | 0x8 - RACEway Bridge
| 0x0 - Transparent Mode
|-
| 0x1 - Endpoint Mode
|-
| rowspan="2" | 0x9 - PCI-to-PCI Bridge
| 0x40 - Semi-Transparent, Primary bus towards host CPU
|-
| 0x80 - Semi-Transparent, Secondary bus towards host CPU
|-
| 0x0A - InfiniBand-to-PCI Host Bridge || --
|-
| 0x80 - Other || --
|-
| rowspan="21" | 0x7 - Simple Communication Controller
| rowspan="7" | 0x0 - Serial Controller
| 0x0 - 8250-Compatible (Generic XT)
|-
| 0x1 - 16450-Compatible
|-
| 0x2 - 16550-Compatible
|-
| 0x3 - 16650-Compatible
|-
| 0x4 - 16750-Compatible
|-
| 0x5 - 16850-Compatible
|-
| 0x6 - 16950-Compatible
|-
| rowspan="5" | 0x1 - Parallel Controller
| 0x0 - Standard Parallel Port
|-
| 0x1 - Bi-Directional Parallel Port
|-
| 0x2 - ECP 1.X Compliant Parallel Port
|-
| 0x3 - IEEE 1284 Controller
|-
| 0xFE - IEEE 1284 Target Device
|-
| 0x2 - Multiport Serial Controller || --
|-
| rowspan="5" | 0x3 - Modem
| 0x0 - Generic Modem
|-
| 0x1 - Hayes 16450-Compatible Interface
|-
| 0x2 - Hayes 16550-Compatible Interface
|-
| 0x3 - Hayes 16650-Compatible Interface
|-
| 0x4 - Hayes 16750-Compatible Interface
|-
| 0x4 - IEEE 488.1/2 (GPIB) Controller || --
|-
| 0x5 - Smart Card Controller || --
|-
| 0x80 - Other || --
|-
| rowspan="18" | 0x8 - Base System Peripheral
| rowspan="5" | 0x0 - PIC
| 0x0 - Generic 8259-Compatible
|-
| 0x1 - ISA-Compatible
|-
| 0x2 - EISA-Compatible
|-
| 0x10 - I/O APIC Interrupt Controller
|-
| 0x20 - I/O(x) APIC Interrupt Controller
|-
| rowspan="3" | 0x01 - DMA Controller
| 0x00 - Generic 8237-Compatible
|-
| 0x01 - ISA-Compatible
|-
| 0x02 - EISA-Compatible
|-
| rowspan="4" | 0x02 - Timer
| 0x00 - Generic 8254-Compatible
|- 
| 0x01 - ISA-Compatible
|-
| 0x02 - EISA-Compatible
|-
| 0x03 - HPET
|-
| rowspan="2" | 0x3 - RTC Controller
| 0x0 - Generic RTC
|-
| 0x1 - ISA-Compatible
|-
| 0x4 - PCI Hot-Plug Controller || --
|-
| 0x5 - SD Host controller || --
|-
| 0x6 - IOMMU || --
|-
| 0x80 - Other || --
|-
| rowspan="7" | 0x9 - Input Device Controller
| 0x0 - Keyboard Controller || --
|-
| 0x1 - Digitizer Pen || --
|-
| 0x2 - Mouse Controller || --
|-
| 0x3 - Scanner Controller || --
|-
| rowspan="2" | 0x4 - Gameport Controller
| 0x0 - Generic
|-
| 0x10 - Extended
|-
| 0x80 - Other || --
|-
| rowspan="2" | 0xA - Docking Station
| 0x0 - Generic || --
|-
| 0x80 - Other || --
|-
| rowspan="9" | 0xB - Processor
| 0x0 - 386 || --
|-
| 0x1 - 486 || --
|-
| 0x2 - Pentium || --
|-
| 0x3 - Pentium Pro || --
|-
| 0x10 - Alpha || --
|-
| 0x20 - PowerPC || --
|-
| 0x30 - MIPS || --
|-
| 0x40 - Co-Processor || --
|-
| 0x80 - Other || --
|-
| rowspan="19" | 0xC - Serial Bus Controller
| rowspan="2" | 0x0 - FireWire (IEEE 1394) Controller
| 0x0 - Generic
|-
| 0x10 - OHCI
|-
| 0x1 - ACCESS Bus Controller || --
|-
| 0x2 - SSA || --
|-
| rowspan="6" | 0x3 - USB Controller
| 0x0 - UHCI Controller
|-
| 0x10 - OHCI Controller
|-
| 0x20 - EHCI (USB2) Controller
|-
| 0x30 - XHCI (USB3) Controller
|-
| 0x80 - Unspecified
|-
| 0xFE - USB Device (Not a host controller)
|-
| 0x4 - Fibre Channel || --
|-
| 0x5 - SMBus Controller || --
|-
| 0x6 - InfiniBand Controller || --
|-
| rowspan="3" | 0x7 - IPMI Interface
| 0x0 - SMIC
|-
| 0x1 - Keyboard Controller Style
|-
| 0x2 - Block Transfer
|-
| 0x8 - SERCOS Interface (IEC 61491) || --
|-
| 0x9 - CANbus Controller || --
|-
| 0x80 - Other || --
|-
| rowspan="8" | 0xD - Wireless Controller
| 0x0 - iRDA Compatible Controller || --
|-
| 0x1 - Consumer IR Controller || --
|-
| 0x10 - RF Controller || --
|-
| 0x11 - Bluetooth Controller || --
|-
| 0x12 - Broadband Controller || --
|-
| 0x20 - Ethernet Controller (802.1a) || --
|-
| 0x21 - Ethernet Controller (802.1b) || --
|-
| 0x80 - Other || --
|-
| 0xE - Intelligent Controller || 0x0 - I20 || --
|-
| rowspan="4" | 0xF - Satellite Communication Controller
| 0x1 - Satellite TV Controller || --
|-
| 0x2 - Satellite Audio Controller || --
|-
| 0x3 - Satellite Voice Controller || --
|-
| 0x4 - Satellite Data Controller || --
|-
| rowspan="3" | 0x10 - Encryption Controller
| 0x0 - Network and Computing Encrpytion/Decryption || --
|-
| 0x10 - Entertainment Encryption/Decryption || --
|-
| 0x80 - Other || --
|-
| rowspan="5" | 0x11 - Signal Processing Controller
| 0x0 - DPIO Modules || --
|-
| 0x1 - Performance Counters || --
|-
| 0x10 - Communication Synchronizer || --
|-
| 0x20 - Signal Processing Management || --
|-
| 0x80 - Other || --
|-
| 0x12 - Processing Accelerator || -- || --
|-
| 0x13 - Non-Essential Instrumentation || -- || --
|-
| 0x14 - 0x3F (Reserved) || -- || --
|-
| 0x40 - Co-Processor || -- || --
|-
| 0x41 - 0xFE (Reserved) || -- || --
|-
| 0xFF - Unassigned Class (Vendor specific) || -- || --
|-
|}

== Enumerating PCI Buses ==

There are 3 ways to enumerate devices on PCI buses. The first way is "brute force", checking every device on every PCI bus (regardless of whether the PCI bus exists or not). The second way avoids a lot of work by figuring out valid bus numbers while it scans, and is a little more complex as it involves recursion. For both of these methods you rely on something (firmware) to have configured PCI buses properly (setting up PCI to PCI bridges to forward request from one bus to another). The third method is like the second method, except that you configure PCI bridges while you're doing it.

For all 3 methods, you need to be able to check if a specific device on a specific bus is present and if it is multi-function or not. Pseudo-code might look like this:

<syntaxhighlight lang="c">
 void checkDevice(uint8_t bus, uint8_t device) {
     uint8_t function = 0;

     vendorID = getVendorID(bus, device, function);
     if (vendorID == 0xFFFF) return; // Device doesn't exist
     checkFunction(bus, device, function);
     headerType = getHeaderType(bus, device, function);
     if( (headerType & 0x80) != 0) {
         // It's a multi-function device, so check remaining functions
         for (function = 1; function < 8; function++) {
             if (getVendorID(bus, device, function) != 0xFFFF) {
                 checkFunction(bus, device, function);
             }
         }
     }
 }

 void checkFunction(uint8_t bus, uint8_t device, uint8_t function) {
 }
</syntaxhighlight>

Please note that if you don't check bit 7 of the header type and scan all functions, then some single-function devices will report details for "function 0" for every function.

=== "Brute Force" Scan ===

For the brute force method, the remaining code is relatively simple. Pseudo-code might look like this:

<syntaxhighlight lang="c">
 void checkAllBuses(void) {
     uint16_t bus;
     uint8_t device;

     for (bus = 0; bus < 256; bus++) {
         for (device = 0; device < 32; device++) {
             checkDevice(bus, device);
         }
     }
 }
</syntaxhighlight>

For this method, there are 32 devices per bus and 256 buses, so you call "checkDevice()" 8192 times.

=== Recursive Scan ===

The first step for the recursive scan is to implement a function that scans one bus. Pseudo-code might look like this:

<syntaxhighlight lang="c">
 void checkBus(uint8_t bus) {
     uint8_t device;

     for (device = 0; device < 32; device++) {
         checkDevice(bus, device);
     }
 }
</syntaxhighlight>

The next step is to add code in "checkFunction()" that detects if the function is a PCI to PCI bridge. If the device is a PCI to PCI bridge then you want to extract the "secondary bus number" from the bridge's configuration space and call "checkBus()" with the number of the bus on the other side of the bridge.

Pseudo-code might look like this:

<syntaxhighlight lang="c">
 void checkFunction(uint8_t bus, uint8_t device, uint8_t function) {
     uint8_t baseClass;
     uint8_t subClass;
     uint8_t secondaryBus;

     baseClass = getBaseClass(bus, device, function);
     subClass = getSubClass(bus, device, function);
     if ((baseClass == 0x6) && (subClass == 0x4)) {
         secondaryBus = getSecondaryBus(bus, device, function);
         checkBus(secondaryBus);
     }
 }
</syntaxhighlight>

The final step is to handle systems with multiple PCI host controllers correctly. Start by checking if the device at bus 0, device 0 is a multi-function device. If it's not a multi-function device, then there is only one PCI host controller and bus 0, device 0, function 0 will be the PCI host controller responsible for bus 0. If it's a multi-function device, then bus 0, device 0, function 0 will be the PCI host controller responsible for bus 0; bus 0, device 0, function 1 will be the PCI host controller responsible for bus 1, etc (up to the number of functions supported).

Pseudo-code might look like this:

<syntaxhighlight lang="c">
 void checkAllBuses(void) {
     uint8_t function;
     uint8_t bus;

     headerType = getHeaderType(0, 0, 0);
     if ((headerType & 0x80) == 0) {
         // Single PCI host controller
         checkBus(0);
     } else {
         // Multiple PCI host controllers
         for (function = 0; function < 8; function++) {
             if (getVendorID(0, 0, function) == 0xFFFF) break;
             bus = function;
             checkBus(bus);
         }
     }
 }
</syntaxhighlight>

=== Recursive Scan With Bus Configuration ===

This is similar to the recursive scan above; except that you set the "secondary bus" field in PCI to PCI bridges (using something like <code>setSecondaryBus(bus, device, function, nextBusNumber++);</code> instead of the <code>getSecondaryBus();</code>). However; if you are configuring PCI buses you are also responsible for configuring the memory areas/BARs in PCI functions, and ensuring that PCI bridges forward requests from their primary bus to their secondary buses.

Writing code to support this without a deep understanding of PCI specifications is not recommended; and if you have a deep understanding of PCI specifications you have no need for pseudo code. For this reason there will be no example code for this method here.

=== Configuring PCI-to-PCI bridges ===
To configure this the kernel has to forget about BIOS for a moment, first scan the root PCI device, (check if it is multi-function to scan multiple buses). Root bus is always 0.

Secondary and subordinate bus acts as a range start-end of what buses the PCI-to-PCI bridge will manage.

Then, after this step it's up to implementation: Scan each device, then if a bridge is found, allocate a bus number to it (note: PCI-to-PCI bridges can have multiple bridges within them). Scan that bus and find more devices, once you find more bridges add 1 to the subordinate bus for each bridge found, because PCI-to-PCI bridges can manage multiple bridges.

And this is just the beginning: After allocating bus numbers, you need to allocate MMIO, it would be trivial if it wasn't for the fact that PCI has 3 areas the kernel manages: IO, Prefetch and Memory.

A bridge can manage multiple buses, but that means it spans all the memory of these buses, if device 1 is behind bridge 2, which is behind bridge 1, then bridge 2 will contain the memory area of device 1 + any other device's areas, supposing IO is 4M, Memory is 16M and Prefetch is 5MB (supposing there are 3 devices in bridge's 2 bus), bridge 2 would contain those, take in reference table for Header type <code>0x1</code>. However, bridge 1 will contain the areas of bridge 2 + any other devices in bridge's 1 bus.

Once all memory areas are allocated, the devices can be used. Note that PCI-to-PCI bridges also have BAR's.

If the kernel does not configure a PCI-to-PCI bridge, the BIOS will probably do, however on environments without BIOS, this method is mandatory otherwise devices behind that bridge won't show up.

== IRQ Handling ==
If you're using the old [[PIC]], your life is really easy. You have the ''Interrupt Line'' field of the header, which is read/write and it says which interrupt will the PCI device fire when it needs attention. However, it is not actually used by the device and is purely informational (PCI-LB 2.2, 6.2.4. Interrupt Line).

If you plan to use the [[I/O APIC]], things aren't so easy. Basically the PCI bus specifies that there are 4 interrupt pins. They are labeled INTA#, INTB#, INTC#, and INTD#. You find out what pin a device is using by reading the ''Interrupt Pin'' field of the header. So far, so good. 

The only problem is that the PCI pins correspond to an arbitrary I/O APIC pin. It's up to the programmer to find the mapping. How is that done? You must parse the [[MP]] Tables or the [[ACPI]] tables. The MP tables are easy, only they aren't supported on newer hardware. The ACPI tables, however, involve parsing AML, which is not an easy task. If one wants to take a shortcut, you can use [[ACPICA]]. 

Once you've found the I/O APIC pin, all you do is map that to an IRQ using the I/O APIC redirection table. See the [[I/O APIC]] article for more information on this.

Alternatively, you could just use MSI or MSI-X, and skip complicated ACPI.

== Message Signaled Interrupts ==
Message Signaled Interrupts, or MSI, is an alternate interrupt mechanism in which a PCI device can offload the job of triggering an interrupt to a different MMIO device, such as the [[APIC]] on x86 platforms or the [[Generic_Interrupt_Controller_versions_3_and_4|GICv3/4]] on ARM platforms. It was introduced with PCI 2.2 and is mandatory in PCIe, so you can be sure that they're usable on modern hardware.

There are two versions of MSI: the original MSI, which can support up to 32 interrupts, and MSI-X, which can support up to 2048 interrupts. Support for either protocol is device-specific and can be detected via the capabilities list in the PCI device's configuration space. A device might support both MSI and MSI-X, but only one of the two can be enabled. Both MSI and MSI-X supersede legacy pin-based PCI interrupts.

MSI is configured by setting a message address and message data, with masking settings over the data for each interrupt you want to trigger from the device. Quintessentially, the message address and data are exactly that: an arbitrary physical address that the PCI device will write an arbitrary 32-bit value to. These can technically be anything (you can debug this part of the system by using an address in the framebuffer, for example), but in order for it to actually trigger an interrupt, you need to set the address to a MMIO hardware register on a device that will eventually trigger a traditional IRQ on the processor, such as the APIC or the GIC. The message address and data are therefore ultimately platform-specific.

MSI-X is configured similarly to MSI, but more flexibly, as it allows a table of up to 2048 interrupts each with their own message address/data pair.

=== Enabling MSI ===
First, check that the device has a pointer to the capabilities list (status register bit 4 set to 1).
Then, traverse the capabilities list. The low 8 bits of a capability register are the ID - <code>0x05</code> for MSI. The next 8 bits are the offset (in [[#Configuration Space|PCI Configuration Space]]) of the next capability.

The MSI capability is as follows:
{| {{wikitable}}
|-
! Register !! Offset !! Bits 31-24 !! Bits 23-16 !! Bits 15-8 !! Bits 7-0
|-
| Cap + 0x0
| Cap + 0x0
| colspan="2" | Message Control
| Next pointer || Capability ID = 05
|-
| Cap + 0x1
| Cap + 0x4
| colspan="4" | Message Address [Low]
|-
| Cap + 0x2
| Cap + 0x8
| colspan="4" | [Message Address High]
|-
| Cap + 0x2/0x3
| Cap + 0x8/0xC
| colspan="2" | Reserved
| colspan="2" | Message Data
|-
| Cap + 0x4
| Cap + 0x10
| colspan="4" | [Mask]
|-
| Cap + 0x5
| Cap + 0x14
| colspan="4" | [Pending]
|}

Here is the layout of the message control register:

{| {{wikitable}}
|-
! Bits 15-9 !! Bit 8 !! Bit 7 !! Bits 6-4 !! Bits 3-1 !! Bit 0
|-
| Reserved
| Per-vector masking
| 64-bit
| Multiple Message Enable
| Multiple Message Capable
| Enable
|}

'''Message address register format (x86_64)'''

The message address register is used to vector interrupts to a target processor, and its format is architecture-specific. For x86_64, the format is as follows:

{| {{wikitable}}
|+
!Bits 31-20
!Bits 19-12
!Bits 11-4
!Bit 3
!Bit 2
!Bits 1-0
|-
|0x0FEE
|Destination ID
|Reserved
|Redirection hint (RH)
|Destination mode (DM)
|XX
|}
Consult Intel vol. 3, section 10.11.1, for more details.

'''Message data register format (x86_64)'''

The message data register is also architecture-specific, and is used to vector interrupts to an interrupt vector on the destination processor, and configure the delivery mode, edge/level triggered configuration, etc. For x86_64, the format is:

{| {{wikitable}}
|+
!Bits 63-16
!Bit 15
!Bit 14
!Bits 13-11
!Bits 10-8
!Bits 7-0
|-
|Reserved
|Trigger mode
|Level
|Reserved
|Delivery mode
|Vector
|}
Consult Intel vol. 3, section 10.11.2.

'''Multiple messages:'''
{| {{wikitable}}
|-
! MME / MMI !! Interrupts
|-
| 000 || 1
|-
| 001 || 2
|-
| 010 || 4
|-
| 011 || 8
|-
| 100 || 16
|-
| 101 || 32
|}

In MME, specifies the number of low bits of Message Data that may be modified by the device.

Therefore, the interrupt vector block allocated must be aligned accordingly.

'''Interrupt masking'''

If capable, you can mask individual messages by setting the corresponding bit (1 << n), in the mask register.

If a message is pending, then the corresponding bit in the pending register is set.

Note that the PCI specification doesn't specify the location of these registers if the message address is 32-bit. This is because a function that supports masking is required to implement 64-bit addressing!

=== Enabling MSI-X ===
Like for MSI, you have to find the MSI-X capability, but the ID for MSI-X is '''0x11'''

The structure is as follows:
{| {{wikitable}}
|-
! Register !! Offset !! Bits 31-24 !! Bits 23-16 !! Bits 15-8 !! Bits 7-3 !! Bits 2-0
|-
| Cap + 0x0
| Cap + 0x0
| colspan="2" | Message Control
| Next Pointer
| colspan="2" | Capability ID = 11
|-
| Cap + 0x1
| Cap + 0x4
| colspan="4" | Table Offset
| colspan="1" | BIR
|-
| Cap + 0x2
| Cap + 0x8
| colspan="4" | Pending Bit Offset
| colspan="1" | Pending Bit BIR
|}

Unlike MSI, MSI-X supports 2048 interrupts. This is achieved by maintaining a table of interrupts in the PCI device's address space. The wording of the PCI 3.0 specification indicates that this ''must'' be via a Memory BAR.

'''BIR''' specifies which BAR is used for the Message Table. This may be a 64-bit BAR, and is zero-indexed (so BIR=0, BAR0, offset <code>0x10</code> into the header).

'''Table Offset''' is an offset into that BAR where the Message Table lives. Note that it is 8-byte aligned - so simply mask BIR.

The format of Message Control is as follows:
{| {{wikitable}}
|-
! Bit 15 !! Bit 14 !! Bits 13-11 !! Bits 10-0
|-
| Enable
| Function Mask
| Reserved
| Table Size
|}

'''Table Size''' is N - 1 encoded, and is the number of entries in the MSI-X table. This field is Read-Only.

Now you have all the information you need to find the MSI-X table:
{| {{wikitable}}
|-
! Bits 127-96 !! Bits 95-64 !! Bits 63-32 !! Bits 31-0
|-
| Vector Control (0)
| Message Data (0)
| Message Address High (0)
| Message Address Low (0)
|-
| Vector Control (1)
| Message Data (1)
| Message Address High (1)
| Message Address Low (1)
|-
| ...
| ...
| ...
| ...
|-
| Vector Control (N - 1)
| Message Data (N - 1)
| Message Address High (N - 1)
| Message Address Low (N - 1)
|}

'''Vector Control''' is as follows:
{| {{wikitable}}
|-
! Bits 31-1 !! Bit 0
|-
| Reserved
| Masked
|}

Note that '''Message Address''' is is 4-byte aligned, so, again, mask the low bits. The interrupt is masked if '''Masked''' is set to 1.

Message Address and Data are as they were for MSI - quintessentially any arbitrary address and data you wish, but ultimately platform-specific as far as actually generating interrupts. However, unlike with MSI, you can specify independent vectors for all the interrupts, only limited by having the same upper 32 bits in the message address.

== Multi-function Devices ==
Multi-function devices behave in the same manner as normal PCI devices. The easiest way to detect a multi-function device is bit 7 of the header type field. If it is set (value = <code>0x80</code>), the device is multi-function -- else it is not. Make sure you mask this bit when you determine header type. To detect the number of functions you need to scan the PCI configuration space for every function - unused functions have vendor <code>0xFFFF</code>. Device IDs and Class codes vary between functions. Functions are not necessarily in order - you can have function <code>0x0</code>, <code>0x1</code> and <code>0x7</code> in use.

== Disclaimer ==

This text originates from "Pentium on VME", unknown author, md5sum d292807a3c56881c6faba7a1ecfd4c79. The original document is apparently no longer present on the Web ...

Closest match: [https://web.archive.org/web/20071009221818/http://www.quicklogic.com/images/appnote10.pdf]

== References ==
* PCI Local Bus Specification, revision 3.0, PCI Special Interest Group, August 12, 2002
== See Also ==
=== Articles ===
* [[PCI Express]]
=== External Links ===
* [https://devicehunt.com/about Very useful page about VendorID and DeviceID]
* [https://cds.cern.ch/record/551427/files/cer-2308933.pdf Very detailed PCI-to-PCI architecture specification]
* [http://xillybus.com/tutorials/pci-express-tlp-pcie-primer-tutorial-guide-1 How PCI Express devices talk]
* [https://docs.oracle.com/cd/E19120-01/open.solaris/819-3196/hwovr-22/index.html PCI Local Bus (Solaris documentation)]
* [https://tldp.org/LDP/tlk/dd/pci.html PCI in the Linux kernel]
* [https://pci-ids.ucw.cz More up to date PCI vendor and device numbers]
* [https://msdn.microsoft.com/en-us/library/ms903537.aspx Structure describes that PCI configuration space for PCI devices]

[[Category:Buses]]
[[Category:PCI]]
[[Category:Standards]]
[[de:Peripheral_Component_Interconnect]]
````

<a id="w001486"></a>

## W001486 — File:Exokernel.png

Bron-ID: W001486
Namespace: File (6)
Revisie: 1818 | Datum: 2006-12-05T02:48:40Z
Bron: https://wiki.osdev.org/File:Exokernel.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AExokernel.png&oldid=1818

````text

````

<a id="w001487"></a>

## W001487 — File:Virtual memory.png

Bron-ID: W001487
Namespace: File (6)
Revisie: 1819 | Datum: 2006-12-05T02:48:58Z
Bron: https://wiki.osdev.org/File:Virtual_memory.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AVirtual_memory.png&oldid=1819

````text

````

<a id="w001488"></a>

## W001488 — Segmentation

Bron-ID: W001488
Namespace: Main (0)
Revisie: 31007 | Datum: 2026-10-09T18:17:42Z
Bron: https://wiki.osdev.org/Segmentation
Revisie-URL: https://wiki.osdev.org/index.php?title=Segmentation&oldid=31007

````text
{{Historical}}

Segmentation is the first memory protection scheme that was supported by the [[x86]] architecture. As the name suggests, it involves splitting the real memory into independent, discriminated ''segments'' or ''sections'', each for a different purpose like code, data, stack and so on. With segmentation, memory addressing no longer consists of flat addresses - instead, the programmer addresses segment start points and specifies the desired offset within the segment.

Segmentation as a technique for memory protection is considered obsolete by both CPU manufacturers and the broader programming community. As such, it is no longer supported in long mode. However, a 64 bit [[GDT]] is needed to enter [[Long Mode|long mode]], and segments are still necessary to transition between long mode and [[Compatibility Mode|compatibility mode]]. Instead of the outdated segmentation model, the modern recommendation for serious operating system projects is to employ a flat memory model and [[paging]]. For more information, consult [[x86-64]].

==Real mode==

In [[Real Mode]] you use a logical address in the form A:B to address memory. This is translated into a physical address using the equation:

 Physical address = (A * 0x10) + B

The registers in pure real-mode are limited to 16 bits for addressing. 16 bits can represent any integer between 0 and 64k. This means that if we set A to be a fixed value and allow B to change, we can address a 64k area of memory. This 64k area is called a segment.

 A = A 64k segment B = Offset within the segment

The base address of a segment is the (A * 0x10) portion of the equation I showed. It should be obvious that segments can overlap.

Eg, the segment 0x1000 has a base address of 0x10000. This segment occupies the physical address range 0x10000 -> 0x1FFFF, However the segment 0x1010 has a base address of 0x10100. This segment occupies the physical address range 0x10100 -> 0x200FF

As you can see, we could use either segment to reach physical addresses between 0x10100 and 0x1FFFF since the segments overlap.

The x86 line of computers have 6 segment registers (CS, DS, ES, FS, GS, SS). They are totally independent of one another.

{| class="wikitable"
|-
! CS
| Code Segment
|-
! DS
| Data Segment
|-
! SS
| [[Stack]] Segment
|-
! ES
| Extra Segment
|-
! FS
| rowspan=2 | General Purpose Segments
|-
! GS
|}

DS, ES, FS, GS, SS are used to form addresses when you want to read/write to memory. They don't always have to be explicitly encoded, because some processor operations assume that certain segment registers will be used.

E.g.

<code>mov [si], ax</code> will write the word contained in ax to the address <code>ds:si</code>

<code>mov es:[di]</code>, AX will write the word contained in ax to the address <code>es:di</code>

<code>cmpsb</code> will compare the byte at <code>ds:si</code> to the byte at <code>es:di</code>, set the zero flag if they are equal and decrement/increment SI and DI according to the state of the direction flag.

As you can see, often the segment register being used is not contained in the instruction, but there is one being used. '''Every time''' an address is formed on an x86 processor there will be a segment register involved.


===Operations that affect segment registers===

Beside CS, segment registers may be loaded with a general register (<code>mov ds, ax</code>) or with the top-of-stack (<code>pop ds</code>).

CS is the only Segment Register that cannot be directly altered. The only time (I'm sure I'm missing one) CS is altered is when the code switches execution into another segment. The only commands that can do this are:

====Far Jump====
Here the new value for CS is encoded in the jump instruction. e.g.: <code>jmp 0x10:0x100</code> says to load CS with segment 0x10 and IP with <tt>0x100</tt>. <code>cs:ip</code> is the logical address of the instruction to be executed.
====Far Call====
This is exactly the same as a far jump, but the current values of CS/IP are pushed onto the [[stack]] before executing at the new position.
====INT====
The processor reads the new value of CS/IP from the Interrupt Vector Table and then executes what is effectively a far call after pushing EFLAGS onto the [[stack]].
====Far Return====
Here the processor pops the return segment/offset from the [[stack]] into CS/IP and switches execution to that address.
====IRET====
This is exactly the same as a far return apart from the processor popping EFLAGS off the [[stack]] in addition to CS/IP.

Apart from these cases no instruction alters the value of CS.

==Protected Mode==
:''Read more about [[Global Descriptor Table]]''

In [[Protected mode]] you use a logical address in the form A:B to address memory. As in [[Real Mode]], A is the segment part and B is the offset within that segment. The registers in protected mode are limited to 32 bits. 32 bits can represent any integer between 0 and 4 GiB.

Because B can be any value between 0 and 4GiB our segments now have a maximum size of 4 GiB (Same reasoning as in real-mode).

Now for the difference.

In protected mode A is not an absolute value for the segment. In protected mode A is a selector. A selector represents an offset into a system table called the [[Global Descriptor Table]] (GDT). The GDT contains a list of descriptors. Each of these descriptors contains information that describes the characteristics of a segment.

Each segment descriptor contains the following information:

* The base address of the segment
* The default operation size in the segment (16-bit/32-bit)
* The privilege level of the descriptor (Ring 0 -> Ring 3)
* The granularity (Segment limit is in byte/4kb units)
* The segment limit (The maximum legal offset within the segment)
* The segment presence (Is it present or not)
* The descriptor type (0 = system; 1 = code/data)
* The segment type (Code/Data/Read/Write/Accessed/Conforming/Non-Conforming/Expand-Up/[[Expand_Down|Expand-Down]])

For the purposes of this explanation I'm only interested in 3 things. The base address, the limit and the descriptor type.

If the descriptor type is clear (System type) then the descriptor isn't actually describing a segment, it's describing either one of the special gate mechanisms, where to find an LDT, or a TSS. These have nothing to do with general addressing, so I'll assume a descriptor type of 1 (code/data type) and leave you to read the Intel manuals for the rest.

The segment is described by its base address and limit. Remember in real-mode where the segment was a 64k area in memory? The only difference here is that the size of the segment isn't fixed. The base address supplied by the descriptor is the start of the segment, the limit is the maximum offset the processor will allow before producing an exception.

So the range of physical addresses in our protected mode segment is:

  Segment Base -> Segment Base + Segment Limit

Given a logical address A:B (Remember that A is a selector) we can determine the physical address it translates to using:

  Physical address = Segment Base (Found from the descriptor GDT[A]) + B

All the other rules from real-mode still apply.

===Notes===
* Segments can overlap
* CS,DS,ES,FS,GS,SS are independent of each other
* CS cannot be changed directly

In protected mode CS can also be changed via the TSS or a gate.

==Notes Regarding C==
*Most C compilers assume a flat-memory model.
*In this model all the segments cover the full address space (usually 0->4Gb on x86). In essence this means that we completely ignore the A part of our A:B logical address. The reason for this is that most processors don't actually have segmentation (and it's much easier for the compiler to optimize).
*This leaves you with 2 descriptors per privilege level (usually Ring 0 and Ring 3), one for code and one for data, which both describe precisely the same segment. The only difference being that the code descriptor is loaded into CS, and the data descriptor is used by all the other segment registers. The reason you need both a code and data descriptor is that the processor will not allow you to load CS with a data descriptor (This is to help with security when using a segmented memory model, and although useless in the flat-memory model it is still required because you can't turn off segmentation).
*In general if you want to use the segmentation mechanism, by having the different segment registers represent segments with different base addresses, you won't be able to use a modern C compiler, and may very well be restricted to just Assembly.
*So, if you're going to use C, do what the rest of the C world does, which is set up a flat-memory model, use paging, and ignore the fact that segmentation even exists.

== Notes Regarding Pascal (FPC) ==

Above may apply in theory to Free Pascal, however, in reality is ignored, if the compiler at all pays any attention to same.
The twin segments for CODE and DATA are used, and as specified above,therefore, required. Size limits, however, are respected (does NOT have to be 4GB in length).

The 'A in A:B' is what allows 48 and 64 bit pointer references, not only with Pascal's NewFrontier unit, but Free Pascal as well(Word:Longint Pointer reference).

==See Also==
=== Articles ===
* [[Segment Limits#Segmentation|Segment Limits]]
* [[Paging]]

=== Threads ===

===External Links===
*[https://thestarman.pcministry.com/asm/debug/Segments.html Removing the Mystery from SEGMENT : OFFSET Addressing]
*[http://duartes.org/gustavo/blog/post/memory-translation-and-segmentation Aug 2008: Memory Translation and Segmentation] by Gustavo Duarte

[[Category:X86]]
[[Category:Memory management]]
[[Category:Memory Segmentation]]
````

<a id="w001489"></a>

## W001489 — Category:Assemblers

Bron-ID: W001489
Namespace: Category (14)
Revisie: 3106 | Datum: 2007-02-28T06:47:36Z
Bron: https://wiki.osdev.org/Category:Assemblers
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AAssemblers&oldid=3106

````text
[[Category:Tools]]
````

<a id="w001491"></a>

## W001491 — Category:Compilers

Bron-ID: W001491
Namespace: Category (14)
Revisie: 16376 | Datum: 2014-05-15T02:48:35Z
Bron: https://wiki.osdev.org/Category:Compilers
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ACompilers&oldid=16376

````text
Here is a wide range of [[Compiler|compilers]] that can be used for Operating System Development.
[[Category:Tools]]
````

<a id="w001492"></a>

## W001492 — Category:Emulators

Bron-ID: W001492
Namespace: Category (14)
Revisie: 1839 | Datum: 2006-12-06T01:33:53Z
Bron: https://wiki.osdev.org/Category:Emulators
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AEmulators&oldid=1839

````text
[[Category:Tools]]
````

<a id="w001493"></a>

## W001493 — Category:Tools

Bron-ID: W001493
Namespace: Category (14)
Revisie: 2395 | Datum: 2006-12-17T00:24:51Z
Bron: https://wiki.osdev.org/Category:Tools
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ATools&oldid=2395

````text
This category lists various tools for OS development.

[[Category:Main Page Topics]]
````

<a id="w001495"></a>

## W001495 — Linkers

Bron-ID: W001495
Namespace: Main (0)
Revisie: 25052 | Datum: 2020-08-10T05:21:16Z
Bron: https://wiki.osdev.org/Linkers
Revisie-URL: https://wiki.osdev.org/index.php?title=Linkers&oldid=25052

````text
A '''Linkage Editor''' or '''linker''' is a programming tool which combines one or more partial [[Object Files]] and [[Link Archiver|libraries]] into a (more) complete executable object file. It often has to modify the executable code from these files, most often to resolve the names of external symbols referenced in one file into addresses (or address markers) matching the code in other files. In many cases, the linker is also able to resolve the actual addresses of most or even all of the labels within the executable code, though in many instances the final resolution is left to the [[Execution Loader]].

'''Historical Note:''' the original name for a linker was ''compiler'', because the original linkers were intended to compile a list of punch cards which the card reader would select before running a program. Modern compilers got their name because they grew out of this function of connecting disparate subroutines into a working program.

==Object Files==

It's typical good-practice to split a large project in several code units that can be compiled/assembled independently. Each such group of source files will be translated into a single object file (typically named yourstuff.obj or yourstuff.o). Object files are thus intermediate binaries that will be used to create your executable and they contain:

* raw datas (binary numbers and ascii strings, for instance)
* machine code
* instructions on how raw datas and machine code are stored in the file
* instructions on what items are still missing from that object file, what it offers (symbol table) and how to fix the raw data/code if a missing item is found (relocations).

Other debugging info such as mapping between code offsets and filename/lineno, or what tool has been used to produce the file and when it was produced may also be available.

==Sections==

Code and data are grouped in sections that share similar properties. For instance, all your code should not be modified and should be executable, so it is placed in the .text section that has r-x flags (read, don't write, exec). Arrays of numbers, on the other side, may be modified, but they certainly shouldn't be jumped to, so they'll rather go to the .data section with rw- flags.

At the stage of intermediate object files, you usually don't know where the section will end up in memory, so only references that remains in the same section (e.g. calling a local procedure) can be pre-compute. All the rest (e.g. indicating the address of a string in the print_hello() function, calling a function from another object file, etc.) is left to the linker.

==The linking process==

The ultimate goal of a linker is to collect bits from all the sections of all the "input" object file to produce an executable file. That usually means that all the .text sections will be merged into a single .text output section, for instance. Moreover, the linker computes the definitive size and (usually) location of each sections according to some rules.

It will also walk the "missing items" list and check other object file's symbol table to make sure every dependency can be resolve. Be it a single printf symbol that cannot be found anywhere, the linker aborts here and throw you an error message. Every time a missing symbol is found, the relocation list for that symbol in the "importing" object file is used to patch the binary (e.g. write the definitive address of the symbol everywhere you referenced it).

As a result, you now have an executable image that has no "dangling references", that has well-known address for every bit of data and every opcode, which you'll write into an executable file ([[PE|.EXE]] or another [[ELF]] file, most likely).

==Linker Scripts==
{{Main|Linker Scripts}}

In many linkers, the user is allowed to alter the default rules, to force a given section to end at a given address, or to add padding here and there. Some can even modify which "output section" will be used for every "input section" manually!

== External Links ==
* [http://www.amazon.com/Computer-Systems-Programmers-Randal-Bryant/dp/013034074X Computer Systems: A Programmer's Perspective.] Chapter 7:Linking,which give several examples to understand process of linking 
* [http://linker.iecc.com/  Linkers and Loaders,by John R. Levine],a book discussing operational principle of linkers and loaders

== See Also ==
* [[Execution Loader]]
* [[Object Files]]
* [[Executable Formats]]
* [[Link Archiver]]
* [[Why function implementations shouldn't be put In header files]]


[[Category:Linkers]]
````

<a id="w001496"></a>

## W001496 — LD

Bron-ID: W001496
Namespace: Main (0)
Revisie: 27591 | Datum: 2023-01-15T00:03:15Z
Bron: https://wiki.osdev.org/LD
Revisie-URL: https://wiki.osdev.org/index.php?title=LD&oldid=27591

````text
{{Stub}}
{{Infobox_Tool
|name=GNU Linker
|website=http://www.gnu.org/software/binutils/
}}

The '''GNU Linker''' (executable named 'ld') is part of the [[binutils]] package. It is the standard linker under Linux and other Unix-like operating systems. It supports most input and output formats in existence (through the use of libbfd), and it is reportedly the most flexible of the listed linkers.

<!--
TODO: This paragraph is somewhat incoherent.
This description is based on the ld sources, that can compile to support all formats. Please be aware that the ld that comes with DJGPP is very limited in its output formats (DJGPP/COFF, A.Out, flat binary), and also the ld that is part of Linux etc. - you have to ./configure and recompile ld to your requirements.
-->
== Features ==
* Supports most known input formats (ELF, DJGPP/COFF, Win32/COFF, a.out, etc)
* Supports most known output formats (ELF, Win32/PE, COFF, a.out etc).
* Supports creation of shared libraries.
* Provides full control over the generated output file, through the flexible [[Linker Scripts|Linker Script]] language.
* Can create flat binary files.
* Can specify code/data addresses, with separate load and execution addresses (vital for a higher half kernel).
* Supports DWARF, ECOFF and STABS debugging information.
* Free software under the GPL version 3.

== Usage ==
ld does not create shared libraries by itself. You have to give the proper flags to the compiler so that it can generate position-independent code (note that this is talking about ELF shared libraries only).

ld is able to link while keeping relocations in your final object (called the 'incremental link'), and it also can leave unresolved symbols if instructed to do so.

<!--
TODO: This doesn't seem relevant for this article.
On the PE side, you have to use a huge hack to get ld to generate relocatable DLLs. This involves running ld three (!) times:

<pre>
ld -s --base-file $(BASE).base --dll -o $(TARGET) $(OBJS) $(LIBS) $(LDFLAGS)
dlltool --as=as --dllname $(TARGET) $(DTFLAGS) --base-file $(BASE).base --output-exp $(EXP_TEMP)
ld -s --base-file $(BASE).base --dll -o $(TARGET) $(OBJS) $(LIBS) $(LDFLAGS)
dlltool --as=as --dllname $(TARGET) $(DTFLAGS) --base-file $(BASE).base --output-exp $(EXP_TEMP)
ld --dll -o $(TARGET) $(EXP) $(OBJS) $(LIBS) $(LDFLAGS)
</pre>

However, this is not, in general, a problem for OS development.
-->
You might also want to check out [[ar]], a static-link library companion for ld.

<!--
=== Using ld to build a modular OS kernel ===
TODO: This should go into its own tutorial
One can use ld to build a modular OS kernel, i.e. a kernel which exports key functions and/or data structures as dynamically linkable items, for use by other independently generated executable objects ("modules").

To set this up, you need to do the following:
* Pass the "-shared" flag to ld when linking the kernel image, to generate tables for exported symbols
* Pass the "-Bsymbolic" flag, to ensure that kernel symbols get resolved (instead of being left for resolution at runtime)
* Add the "-z defs" flags to make sure no symbols are left undefined
* Include the ".hash", ".dynsym" and ".dynstr" sections in your link script (in this order)
* Include the ".got.plt" section in your link script, even though it is not used (linker error if omitted)
* Use 'PROVIDE_HIDDEN( symbol )' in your linker script to avoid exposing <symbol> to external modules
* Add code to the OS kernel for resolving ELF symbols against the kernel export tables

Optionally, one can use the "--hash-style=gnu" flag to generate better optimized hash table structures (section ".gnu.hash") - see [https://blogs.oracle.com/ali/entry/gnu_hash_elf_sections GNU Hash ELF sections]
-->
== What about the LLVM Linker (lld) ==
LLD is meant to be a drop-in replacement for LD (from the GNU binutils). You should decide which linker you want to support at the start of your project because switching linkers later could take a lot of effort and head scratches. In general, however, it should be perfectly fine to use. While it is faster, it generally has fewer options.

== See Also ==
=== Articles ===
* [[Object Files]]
* [[Tool Comparison]]
* [[Linker Scripts]]
* [[Wikipedia:GNU linker|GNU linker]] - Wikipedia

=== External Links ===
* [https://sourceware.org/binutils/docs/ld/ GNU ld official manual]

[[Category:Binutils]]
[[Category:Linkers]]
````

<a id="w001497"></a>

## W001497 — Category:Linkers

Bron-ID: W001497
Namespace: Category (14)
Revisie: 1849 | Datum: 2006-12-06T03:13:14Z
Bron: https://wiki.osdev.org/Category:Linkers
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ALinkers&oldid=1849

````text
[[Category:Tools]]
````

<a id="w001498"></a>

## W001498 — TLink

Bron-ID: W001498
Namespace: Main (0)
Revisie: 22856 | Datum: 2018-09-14T23:11:42Z
Bron: https://wiki.osdev.org/TLink
Revisie-URL: https://wiki.osdev.org/index.php?title=TLink&oldid=22856

````text
{{Stub}}
TLink is the linker that comes with Borland C and their assembler TASM. The difference between TLink and TLink32 is that TLink32 creates ONLY Win32/PE (and DLL) files where as TLink creates Dos and Win16 files.

*Pros
**Supports Win32/COFF and OMF/OBJ
**Has support for libraries
**Supports creating Win32/PE files and DLL's
**Supports OMF/OBJ (and DWARF?) debugging information
*Cons
**Lacking in both input and output fileformats
**Can't create flat binary images
**Can't create shared libraries

== See Also ==
=== External Links ===
* [ftp://ftp.kis.p.lodz.pl/pub/laboratoria/T.Koszmider/Bp/tasm/doc/tlink.txt TLINK.TXT]

[[Category:Linkers]]
````

<a id="w001499"></a>

## W001499 — JLoc

Bron-ID: W001499
Namespace: Main (0)
Revisie: 30984 | Datum: 2026-09-28T14:17:30Z
Bron: https://wiki.osdev.org/JLoc
Revisie-URL: https://wiki.osdev.org/index.php?title=JLoc&oldid=30984

````text
{{Stub}}
JLoc was written by John Fine.  It is available from [https://files.osdev.org/mirrors/osdever/downloads/other/jloc07.zip here].

:"This program is a linker/locator for use when you need more control over placement of sections within the image, than a standard linker provides."

JLoc can be tricky to use for first timers. Carefull reading of the documentation is required. Its not a get-up-and-go linker like all the other linkers. JLoc requires you to write a custome script for each "executable" image you want to link. JLoc gives you flexibility over HOW the object files are linked, regarding loaded addresses and so-forth.

*Pros
**Supports OMF/OBJ, DJGPP/COFF
**Supports 32bit code
**Supports flat binary file generation
**Handles library files (OMF/OBJ)
**OMF/OBJ debugging information
**Free
*Cons
**Does not support any output file formats other than flat binary and DOS/COM
**Confusing to use for beginners
**Can't create shared libraries
**Is a 16 bit executable, so it can't run on 64 bit windows without a VM


[[Category:Linkers]]
````

<a id="w001500"></a>

## W001500 — ALink

Bron-ID: W001500
Namespace: Main (0)
Revisie: 31016 | Datum: 2026-10-12T11:57:25Z
Bron: https://wiki.osdev.org/ALink
Revisie-URL: https://wiki.osdev.org/index.php?title=ALink&oldid=31016

````text
{{stub}}

'''ALink''' ('''Anthony's Linker''') is a linker written as a companion to [[NASM]]. Its aim to provide a free linker that supports NASM's output formats and supports some of the more popular executable formats. Its last release occurred in 2000.

== About ==
=== Advantages ===
* Free
* Supports input formats OMF/OBJ, Win32/COFF objects
* Supports output as DOS/COM, DOS/EXE, Win32/PE
* Can build Win32/PE DLL files
* Can set the code address
* Supports .LIB (OBJ) library files

=== Disadvantages ===
* Doesn't support flat binary images
* Strips debugging information
* Can't create [[Shared library|shared libraries]]

== See Also ==
=== External ===
* [http://alink.sourceforge.net ALink's homepage]
* [https://github.com/minxomat/alink Updated ALink source code compatible with GCC 4.9]

[[Category:Linkers]]
````

<a id="w001501"></a>

## W001501 — Link

Bron-ID: W001501
Namespace: Main (0)
Revisie: 22691 | Datum: 2018-09-10T01:16:14Z
Bron: https://wiki.osdev.org/Link
Revisie-URL: https://wiki.osdev.org/index.php?title=Link&oldid=22691

````text
{{Stub}}
{{FirstPerson}}
Link and NLink are Microsoft's linkers. There are really three fundamentally different linkers.

Versions 1.x through 3.x can only produce DOS output. Technically, the really early ones can only produce .com files, and IIRC, starting with 2.x they can also produce .exe files.

Starting with version 4.x, MS Link can also produce NE format output files. These accept OMF/OBJ format with some MS extensions.

Starting with version 6.x, MS link can produce only PE format output files.

These accept either COFF/PE or 32-bit OMF/OBJ format object files and produce (only) PE files as output. They support a few things missing from the Borland linker such as the ability to specify the entry point. Debug formats supported include COFF and CodeView.

Here I should note that I'm listing only the relatively mainstream versions of MS link. They've also produced versions that produce various other formats of output, mostly for OS/2 and/or Windows 3.x VxDs (e.g. LE and LX format).

[[Category:Linkers]]
````

<a id="w001502"></a>

## W001502 — VAL

Bron-ID: W001502
Namespace: Main (0)
Revisie: 28562 | Datum: 2024-01-17T00:50:51Z
Bron: https://wiki.osdev.org/VAL
Revisie-URL: https://wiki.osdev.org/index.php?title=VAL&oldid=28562

````text
{{Stub}}
{{Tone}}
VALink is a linker that has been floating around for ages. It is showing its age quite well. There exists an Enhnaced VALink that needs a review!

*Pros
** Its free
** Supports makeing DOS/EXE files.
*Cons
** Can ONLY handle 16bit code.

== See also ==

=== Articles ===

* [[LD]]

=== External Links ===

* [https://ftp.sunet.se/mirror/archive/ftp.sunet.se/pub/simtelnet/msdos/pgmutl/val_link.zip Download link for VAL]
* [https://news.ycombinator.com/item?id=30413188 Old hacker news comment about it]

[[Category:Linkers]]
````

<a id="w001503"></a>

## W001503 — WLink

Bron-ID: W001503
Namespace: Main (0)
Revisie: 12870 | Datum: 2012-03-01T15:13:06Z
Bron: https://wiki.osdev.org/WLink
Revisie-URL: https://wiki.osdev.org/index.php?title=WLink&oldid=12870

````text
WLink is the linker that comes with [[Watcom]] C/C++. Supports a wide range of options and is flexible to a degree. The only problem is that Watcom just uses the virtual address entries of the ELF format, which can be specified at link time with the offset=0x???????? directive.

*Pros
** Supports range of input formats, OMF/OBJ, Win32/COFF, ELF
** Supports library files
** Supports many output file formats, from EXE, MZ, COM, LE or Win32/PE, to Novell NLM files, Pharlap files, QNX, OS2, etc.
** Handles setting code offset address
** Supports Watcom, Codeview and Dward debugging information
** Supports flat binary files
** Custom segment location
[[Category:Linkers]]

== See Also ==

* [[Watcom]]
````

<a id="w001504"></a>

## W001504 — Category:Bootloaders

Bron-ID: W001504
Namespace: Category (14)
Revisie: 2433 | Datum: 2006-12-18T11:21:34Z
Bron: https://wiki.osdev.org/Category:Bootloaders
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ABootloaders&oldid=2433

````text
A bootloader is a program written to load a more complex kernel.
{{main|Bootloader}}

[[Category:Tools]]
````

<a id="w001505"></a>

## W001505 — File Systems

Bron-ID: W001505
Namespace: Main (0)
Revisie: 29795 | Datum: 2025-12-19T21:49:56Z
Bron: https://wiki.osdev.org/File_Systems
Revisie-URL: https://wiki.osdev.org/index.php?title=File_Systems&oldid=29795

````text
{{Tone}}

{{Filesystems}}

File systems are the operating system's method of ordering data on persistent storage devices like disks. They provide an abstracted interface to access data on these devices in such a way that it can be read or modified efficiently. Which file system is convenient depends on the target application of the operating system. For example, Windows uses the FAT32 or NTFS file system. If a disk has a large capacity, FAT32 is inconvenient, because the FAT system was designed considering the smaller disks available at that time. At the same time, an NTFS file system is not convenient on a tiny disk, because it was designed to work with large volumes of data - there would be excessive overhead when using devices such as a 1.44 MB floppy disk.

For details on specific filesystems, browse [[:Category:Filesystems|this list]] of filesystems.

== File System Theory ==

{{In Progress}}

A filesystem provides a generalized structure over persistent storage, allowing the low-level structure of the devices (e.g., disk, tape, flash memory storage) to be abstracted away. Generally speaking, the goal of a filesystem is allowing logical groups of data to be organized into ''files'', which can be manipulated as a unit. In order to do this, the filesystem must provide some sort of index of the locations of files in the actual secondary storage. The fundamental operations of any filesystem are:

* Tracking the available storage space
* Tracking which block or blocks of data belong to which files
* Creating new files
* Reading data from existing files into memory
* Updating the data in the files
* Deleting existing files

(Perceptive readers will note that the last four operations - Create, Read, Update, and Delete, or CRUD - are also applicable to many other data structures, and are fundamental to databases as well as filesystems.)

Additionally, there are other features which go along with a practical filesystem:

* Assigning human-readable names to files, and renaming files after creation 
* Allowing files to be divided among non-contiguous blocks in storage, and tracking the parts of files even when they are ''fragmented'' across the medium
* Providing some form of hierarchical structure, allowing the files to be divided into ''directories'' or ''folders''
* Buffering reading and writing to reduce the number of actual operation on the physical medium
* Caching frequently accessed files or parts of files to speed up access
* Allowing files to be marked as "read-only" to prevent unintentional corruption of critical data
* Providing a mechanism for preventing unauthorized access to a user's files

Additional features may be found on some filesystems as well, such as automatic encryption, or journalling of read/write activity.

=== Indexing Methods ===
There are several methods of indexing the contents of files, with the most commonly used being ''i-nodes'' and ''File Allocation Tables''.

==== inodes ====
inodes (information nodes) are a crucial design element in most Unix file systems: Each file is made of data blocks (the sectors that contains your raw data bits), index blocks (containing pointers to data blocks so that you know which sector is the nth in the sequence), and one inode block.

The inode is the root of the index blocks, and can also be the sole index block if the file is small enough. Moreover, as Unix file systems support hard links (the same file may appear several times in the directory tree), inodes are a natural place to store Metadata such as file size, owner, creation/access/modification times, locks, etc.

==== FAT ====
The File Allocation Table ([[FAT]]) is the primary indexing mechanism for MS-DOS and its descendants. There are several variants on FAT, but the general design is to have a table (actually a pair of tables, one serving as a backup for the first in case it is corrupted) which holds a list of blocks of a given size, which map to the whole capacity of the disk.

== Workings of File Systems ==

There are several common approaches to storing disk information. However, in comparison to [[Memory Management|memory management]], there are some key differences in managing disk media:
* Data can only be written in fixed size chunks (commonly called "blocks" or "sectors").
* For HDDs, access times are different for different physical locations on the disk. Seeking usually incurs a high cost.
* For SSDs, the duration of I/O operations depends on the number of disk requests only (i.e. reading 10 sectors in one access is faster than reading 5 sectors twice); any latency differences are mostly invisible to software.
*Data throughput is very small compared to RAM.
*Data commonly has to be maintained.
Hence some file systems have specialized structures, algorithms, or combinations thereof to improve speed ratings.

===Allocation Table===

The allocation table is comparable to the bitmap approach. However instead of just having a field free or occupied, it may contain other information. Advantage is that the use of this structure is simple, Disadvantage is that this approach is relatively slow, and a separate set of data is needed to define the which sections are used, and in which order.
[[FAT]] for example, combines a linked list and an allocation table in the same structure. 

=== Separate file and system areas ===

Some filesystems keep metadata and actual contents in separate areas on the disk. This makes specific sorts of data easy to find, as well as allowing for a special "free" area. Downside is that you have to keep track of the boundary or boundaries between these areas, as the usage can differ and the disk has to adapt for that (big files: more data, less metadata; small files: less data, more metadata).
This method is used prominently in [[SFS]].

=== Other methods ===
{{In Progress}}

==Network File Systems==

All these file systems are a way to create a large, distributed storage system from a collection of "back end" systems. That means you cannot (for instance) format a disk in "NFS" but you instead mount a "virtual" NFS partition that will reflect what's on another machine. Note that a new generation of file systems is under heavy research, basing on latest P2P, cryptography and error correction techniques (such as the Ocean Store Project or Archival Intermemory.)

For details on various network file systems [[:Category:Network Filesystems|look here]]

== File systems for OSDevers ==
While you could pick <insert favorite filesystem here> as your OS main filesystem, you might want to consider all options. Commonly, you'll want to have a filesystem that is operational very quickly so that you can concentrate on the rest before implementing a "real filesystem"

==="Beginners" filesystems===
There are only five filesystems that are both relatively easy to implement and worth to consider. There is no general recommendation as the choice depends largely on style and OS design. Instead you can read the comparison and make your own educated decision.

'''[[USTAR]]'''
*<code>+</code> Of these beginner "filesystems", this is the simplest by far to implement
*<code>+</code> Incredibly simple: a sector with metadata followed by data sectors
*<code>+</code> Widely used: utilities to create tar images are available for every mainstream OS and many minor ones
*<code>+</code> Supports special files (like devices and symlinks)
*<code>+</code> Supports Unix permissions
*<code>-</code> Not a filesystem in the common understanding of the term
*<code>-</code> Generally read-only, was never designed for in-place modifications
*<code>-</code> No support for fragmentation
*<code>-</code> No standard partition type for it (not that you should even consider using USTAR as a disk partition format)
*<code>-</code> Not actually the format used for ramdisks by things like Linux - that's CPIO

'''RAMFS/TMPFS'''
*<code>+</code> High flexibility of implementation
*<code>+</code> Fast
*<code>+</code> Will allow you to test out your VFS API without having to rely on filesystem specifics
*<code>+</code> *Highly* recommended as a starter filesystem to avoid morphing your VFS interface around a specific filesystem
*<code>+</code> Ideal to unpack a [[USTAR]] or [[CPIO]] [[initrd]] image into
*<code>-</code> Changes are, obviously, not persistent, and only in memory, to be wiped after a reboot

'''[[FAT]]'''
* <code>+</code> Can be read and written by virtually all OSes
*<code>+</code> The "standard" for floppies
*<code>+</code> Relatively easy to implement
*<code>-</code> Part of it involving long filenames and compatibility is [http://en.swpat.org/wiki/Microsoft_FAT_patents patented by Microsoft]
*<code>-</code> Large overhead
*<code>-</code> No support for large (>4 GB) files
*<code>-</code> No support for Unix permissions

'''[[Ext2]]''' 
*<code>+</code> Supports large files (with an extension)
*<code>+</code> Supports Unix permissions
*<code>+</code> Can be put on floppies
*<code>+</code> Can be read and written from Linux
*<code>-</code> Can not natively be read and written from Windows (but [http://www.fs-driver.org/ drivers] are available)
*<code>-</code> Very large overhead
*<code>-</code> Of these beginner filesystems, this is the most complex

'''[[BMFS]]''' 
*<code>+</code> Supports large files
*<code>+</code> Implementation Available as static library
*<code>+</code> Comes with utility program for creating disk images on Linux and Windows
*<code>+</code> Comes with FUSE bindings, allowing it to be mounted on Linux systems
* <code>+</code> Contains source code documentation
*<code>-</code> Does not support fragmentation
* <code>-</code> Less control over the source code

'''[[ISO 9660]]'''
The defined standard for CDs. If you boot from CD then this is the way to go. If not, avoid making it your first filesystem.

===Rolling your own===
There are many kinds of filesystems, from the well-known to the more obscure. Designing a unique one is a great challenge for any hobbyist OS developer. Many start by recreating [[FAT]]-like filesystems because it tends to be the first idea that comes to mind when thinking about how to organize files. While this approach is simple and familiar, there is real value in pushing beyond it. Rather than settling for another variation of what's already been done, this is an opportunity to explore new ideas, solve problems in original ways, and create something that truly fits the goals of your OS. Investigate all the possibilities before deciding to roll your own, try to think beyond the common perceptions of how operating systems store files.

If despite of this warning you decide to create your own file system, then you should start that by implementing a [[FUSE]] driver for it. This gives the advantage that you can mount your file system image as any other storage device, and you can lists its contents, create new files and directories etc. with standard tools. FUSE is available for Linux, macOS and Windows as well.

====Guidelines if you do decide to roll your own====
*Consider carefully what it will be used for.
*Use a program to figure out the layout (e.g. a spreadsheet). The basic areas needed are:
** Bootsector.  This is essential for booting on some systems such as BIOS-x86 and Atari ST, unnecessary for others such as UEFI and OpenFirmware. Even if you don't intend to boot on systems which require it, reserving the first sector will allow your OS to be ported to them at a later time. Note that reserving space for a MBR-like partition table is needed to allow the filesystem to work in "logical partitions".
** Partition metadata. This could fit into the first sector with the boot code, or be a separate group of sectors at a specific location. (FAT puts it in the first sector, calling it the FAT parameter block. ext* use a separate location, calling it the superblock.) At a minimum, this should contain the filesystem size, location of the file table, and a version number.  Leave plenty of reserved space for features you don't think of. If you put it in the first sector, don't forget to leave space for a jmp instruction, the boot code, and a partition table!
**File table. Don't think of this as just a simple table containing a list of files and their locations.  One idea is, instead of storing files, the system would store file parts, and the file table would list the parts in each file.  This would be useful for saving space if many files on the disk are the same or similar (for example, license agreements).
**Data area. Files will be stored here.
*Create a program to read and write disk images with your filesystem. Parts of this will be portable into the filesystem driver.
*It is strongly recommended to create a [[FUSE]] driver for your file system.
*Implement the common API of the file system and integrate it into the OS, for example as a kernel module.

===Expert filesystems===
Once you have a beginner's file system under your belt you might want support for more advanced ones. Here are some:
*[[NTFS]] - (Windows) New Technologies File System. It's hard to find documentation. Try [http://www.opensource.apple.com/source/ntfs/ Apple NTFS (open source)].
*Btrfs - B-tree file system. It's a Linux file system with features such as copy-on-write and transparent compression.

==See Also==
*[[I use a Custom Filesystem - What Bootloader Solution is right for me%3F]]

===External links=== 
*[https://blog.koehntopp.info/2023/05/05/50-years-in-filesystems-1974.html 50 years in filesystems] - an approachable account of the history of file systems. Includes, among other information, some real-world examples of heuristics one can use to avoid fragmentation.

[[Category:OS theory]]
[[Category:Filesystems]]
````

<a id="w001506"></a>

## W001506 — FAT

Bron-ID: W001506
Namespace: Main (0)
Revisie: 30279 | Datum: 2026-03-16T17:58:45Z
Bron: https://wiki.osdev.org/FAT
Revisie-URL: https://wiki.osdev.org/index.php?title=FAT&oldid=30279

````text
{{Filesystems}}
The '''File Allocation Table''' ('''FAT''') was the native file system of MS-DOS. FAT was originally introduced by Marc McDonald in Stand-alone Disk BASIC with 8-bit FAT entries and 16 byte directory entries. The better known FAT12 variant, with 12-bit FAT entries and 32 byte directory entries, was introduced with DOS. FAT is a very simple file system -- nothing more than a singly-linked list of clusters in a gigantic table. A FAT file system uses very little memory (unless the OS caches the whole allocation table in memory) and is one of, if not the, most basic file system in use today.

== Overview ==
There are several different versions of the FAT file system. Each version was designed for a different size of storage media. 
=== FAT 12 ===
FAT 12 was designed for floppy disks and can manage a maximum size of 16 megabytes because it uses 12 bits to address the clusters.

=== FAT 16 ===
FAT 16 was designed for early hard disks and could handle a maximum size of 64K clusters * the cluster size. The larger the hard disk, the larger the cluster size would be, which leads to large amounts of "slack space" on the disk.

=== FAT 32 ===
FAT 32 was introduced to us by Windows95-B and Windows98. FAT32 solved some of FAT's problems. No more 64K max clusters! Although FAT32 uses 32 bits per FAT entry, only the bottom 28 bits are actually used to address clusters on the disk (top 4 bits are reserved). With 28 bits per FAT entry, the filesystem can address a maximum of about 270 million clusters in a partition. This enables very large hard disks to still maintain reasonably small cluster sizes and thus reduce slack space between files.

=== ExFAT ===
{{Main|ExFAT}}
ExFAT is the filesystem used on SDXC cards, created by Microsoft. It is FAT32 with actually 32 bits per FAT entry, with the ability to indicate a file is fully consecutive on disk (allowing you to skip reading the FAT), some more advanced features and a fully redesigned file entry system. Since it's so similar to FAT32, please merge any bits of info from the exFAT article into this one.

Microsoft has published the official specification at https://docs.microsoft.com/en-us/windows/win32/fileio/exfat-specification .

=== VFAT ===
VFAT is an extension to the FAT file system that has the ability to use long filenames (up to 255 characters). First introduced by Windows 95, it uses a "kludge" whereby long filenames are marked with a "volume label" attribute and filenames are subsequently stored in 11 byte chunks in sequential directory entries. (This is a bit of an oversimplification, but close enough).

== Implementation Details ==
The FAT file system views the storage media as a flat array of clusters. If the physical media does not address its data as a flat list of sectors (really old hard disks and floppy disks) then the cluster numbers will need to be translated before being sent to the disk. The storage media is organized into three basic areas.

* The boot record
* The File Allocation Table (FAT)
* The directory and data area

=== Boot Record ===
The boot record occupies one sector, and is always placed in logical sector number zero of the "partition". If the media is not divided into partitions, then this is the beginning of the media. This is the easiest sector on the partition for the computer to locate when it is loaded. If the storage media is partitioned (such as a hard disk), then the beginning of the actual media contains an [[MBR (x86)]] or other form of partition information. In this case each partition's first sector holds a [[Volume Boot Record]].

==== BPB (BIOS Parameter Block) ====

The boot record contains both code and data, mixed together. The data that isn't code is known as the BPB.

{| {{wikitable}}
|- 
! Offset (decimal)
! Offset (hex)
! Size (in bytes)
! Meaning
|- 
| 0
| 0x00
| 3
|	The first three bytes EB 3C 90 disassemble to JMP SHORT 3C NOP. (The 3C value may be different.) The reason for this is to jump over the disk format information (the BPB and EBPB). Since the first sector of the disk is loaded into ram at location 0x0000:0x7c00 and executed, without this jump, the processor would attempt to execute data that isn't code. Even for non-bootable volumes, code matching this pattern (or using the E9 jump opcode) is required to be present by both Windows and OS X. To fulfil this requirement, an infinite loop can be placed here with the bytes EB FE 90.
|- 
| 3
| 0x03
| 8
|	OEM identifier. The first 8 Bytes (3 - 10) is the version of DOS being used. The next eight Bytes 29 3A 63 7E 2D 49 48 and 43 read out the name of the version. The official FAT Specification from Microsoft says that this field is really meaningless and is ignored by MS FAT Drivers, however it does recommend the value "MSWIN4.1" as some 3rd party drivers supposedly check it and expect it to have that value. Older versions of dos also report MSDOS5.1, linux-formatted floppy will likely to carry "mkdosfs" here, and FreeDOS formatted disks have been observed to have "FRDOS5.1" here. If the string is less than 8 bytes, it is padded with spaces.
|- 
| 11
| 0x0B
| 2
|	The number of Bytes per sector (remember, all numbers are in the little-endian format).
|- 
| 13
| 0x0D
| 1
|	Number of sectors per cluster.
|- 
| 14
| 0x0E
| 2
|	Number of reserved sectors. The boot record sectors are included in this value.
|- 
| 16
| 0x10
| 1
|	Number of File Allocation Tables (FAT's) on the storage media. Often this value is 2.
|- 
| 17
| 0x11
| 2
|	Number of root directory entries (must be set so that the root directory occupies entire sectors).
|- 
| 19
| 0x13
| 2
|	The total sectors in the logical volume. If this value is 0, it means there are more than 65535 sectors in the volume, and the actual count is stored in the Large Sector Count entry at 0x20.
|- 
| 21
| 0x15
| 1
|	This Byte indicates the [https://en.wikipedia.org/wiki/Design_of_the_FAT_file_system#BPB20_OFS_0Ah media descriptor type].
|- 
| 22
| 0x16
| 2
|	Number of sectors per FAT. FAT12/FAT16 only.
|- 
| 24
| 0x18
| 2
|	Number of sectors per track.
|- 
| 26
| 0x1A
| 2
|	Number of heads or sides on the storage media.
|- 
| 28
| 0x1C
| 4
|	Number of hidden sectors. (i.e. the LBA of the beginning of the partition.)
|- 
| 32
| 0x20
| 4
|	Large sector count. This field is set if there are more than 65535 sectors in the volume, resulting in a value which does not fit in the ''Number of Sectors'' entry at 0x13.
|}

Note: the "geometry" of the media (sectors per track, heads, and perhaps the number of bytes in a sector) is not necessarily known correctly by the program that originally formats the media. Also, if the media is moved (from the computer that formatted it) to another machine with a different BIOS -- then the new BIOS may specify a different geometry for the same media. So it is generally a very bad idea to trust the "SPT" or "heads" numbers. Get them from the BIOS instead, if possible.

Note2: many of the values in the BPB are not correctly "aligned". That is, word-sized values are not stored on word ("even" address) boundaries. On some architectures, accessing misaligned words may cause the code to crash. Making a copy of the BPB (somewhere else in memory and shifted up one byte) may solve the problem.

==== Extended Boot Record ====
The extended boot record information comes right after the BPB. The data at the beginning is known as the EBPB. It contains different information depending on whether this partition is a FAT 12, FAT 16, or FAT 32 filesystem. Immediately following the EBPB is the actual boot code, then the standard 0xAA55 boot signature, to fill out the 512-byte boot sector. Offsets shows are from the start of the standard boot record.

===== FAT 12 and FAT 16 =====
{| {{wikitable}}
|-
! Offset (decimal)
! Offset (hexadecimal)
! Length (in bytes)
! Meaning
|-
| 36
| 0x024
| 1
|Drive number. The value here should be identical to the value returned by BIOS interrupt 0x13, or passed in the DL register; i.e. 0x00 for a floppy disk and 0x80 for hard disks. This number is useless because the media is likely to be moved to another machine and inserted in a drive with a different drive number.
|- 
| 37
| 0x025
| 1
|	Flags in Windows NT. Reserved otherwise.
|- 
| 38
| 0x026
| 1
|Signature (must be 0x28 or 0x29).
|- 
| 39
| 0x027
| 4
|VolumeID 'Serial' number. Used for tracking volumes between computers. You can ignore this if you want.
|- 
| 43
| 0x02B
| 11
|Volume label string. This field is padded with spaces.
|-
| 54
| 0x036
| 8
|System identifier string. This field is a string representation of the FAT file system type. It is padded with spaces. The spec says never to trust the contents of this string for any use.
|-
| 62
| 0x03E
| 448
|Boot code.
|-
| 510
| 0x1FE
| 2
|Bootable partition signature 0xAA55.
|}

===== FAT 32 =====
{| {{wikitable}}
|-
! Offset (decimal)
! Offset (hexadecimal)
! Length (in bytes)
! Meaning
|-
| 36
| 0x024
| 4
|Sectors per FAT. The size of the FAT in sectors.
|- 
| 40
| 0x028
| 2
|	Flags.
|- 
| 42
| 0x02A
| 2
|FAT version number. The high byte is the major version and the low byte is the minor version. FAT drivers should respect this field.
|- 
| 44
| 0x02C
| 4
|The cluster number of the root directory. Often this field is set to 2.
|-
| 48
| 0x030
| 2
|The sector number of the FSInfo structure.
|-
| 50
| 0x032
| 2
|The sector number of the backup boot sector.
|-
| 52
| 0x034
| 12
|Reserved. When the volume is formated these bytes should be zero.
|-
| 64
| 0x040
| 1
|Drive number. The values here are identical to the values returned by the BIOS interrupt 0x13. 0x00 for a floppy disk and 0x80 for hard disks.
|- 
| 65
| 0x041
| 1
|Flags in Windows NT. Reserved otherwise.
|- 
| 66
| 0x042
| 1
|Signature (must be 0x28 or 0x29).
|- 
| 67
| 0x043
| 4
|Volume ID 'Serial' number. Used for tracking volumes between computers. You can ignore this if you want.
|- 
| 71
| 0x047
| 11
|Volume label string. This field is padded with spaces.
|-
| 82
| 0x052
| 8
|System identifier string. Always "FAT32&nbsp;&nbsp;&nbsp;". The spec says never to trust the contents of this string for any use.
|-
| 90
| 0x05A
| 420
|Boot code.
|-
| 510
| 0x1FE
| 2
|Bootable partition signature 0xAA55.
|}


=== FSInfo Structure (FAT32 only) ===
{| {{wikitable}}
|-
! Offset (decimal)
! Offset (hexadecimal)
! Length (in bytes)
! Meaning
|-
| 0
| 0x0
| 4
|Lead signature (must be 0x41615252 to indicate a valid FSInfo structure)
|-
| 4
| 0x4
| 480
|Reserved, these bytes should never be used
|-
| 484
| 0x1E4
| 4
|Another signature (must be 0x61417272)
|-
| 488
| 0x1E8
| 4
|Contains the last known free cluster count on the volume. If the value is 0xFFFFFFFF, then the free count is unknown and must be computed. However, this value might be incorrect and should at least be range checked (<= volume cluster count)
|-
| 492
| 0x1EC
| 4
|Indicates the cluster number at which the filesystem driver should start looking for available clusters. If the value is 0xFFFFFFFF, then there is no hint and the driver should start searching at 2. Typically this value is set to the last allocated cluster number. As the previous field, this value should be range checked.
|-
| 496
| 0x1F0
| 12
|Reserved
|-
| 508
| 0x1FC
| 4
|Trail signature (0xAA550000)

|}

==== exFat boot record ====
For exFAT the whole boot record was recreated from scratch instead of extending the existing FAT12/16/32 boot records even further. You can recognize exFAT by noticing that in the FAT12/16/32 boot record, the "bytes per sector" is zero.

{| {{wikitable}}
|- 
! Offset (decimal)
! Offset (hex)
! Size (in bytes)
! Meaning
|- 
| 0
| 0x00
| 3
|	The first three bytes EB 3C 90 disassemble to JMP SHORT 3C NOP. (The 3C value may be different.) The reason for this is to jump over the disk format information (the BPB and EBPB). Since the first sector of the disk is loaded into ram at location 0x0000:0x7c00 and executed, without this jump, the processor would attempt to execute data that isn't code. Even for non-bootable volumes, code matching this pattern (or using the E9 jump opcode) is required to be present by both Windows and OS X. To fulfil this requirement, an infinite loop can be placed here with the bytes EB FE 90.
|- 
| 3
| 0x03
| 8
|	OEM identifier. This contains the string "EXFAT   ". Not to be used for filesystem determination, but it's a nice hint.
|- 
| 11
| 0x0B
| 53
|	Set to zero. This makes sure any FAT driver will not be able to load it.
|- 
| 64
| 0x40
| 8
|	Partition offset. No idea why the partition itself would have this, but it's here. Might be wrong. Probably best to just ignore.
|- 
| 72
| 0x48
| 8
|	Volume length.
|- 
| 80
| 0x50
| 4
|	FAT offset (in sectors) from start of partition.
|- 
| 84
| 0x54
| 4
|	FAT length (in sectors).
|- 
| 88
| 0x58
| 4
|	Cluster heap offset (in sectors).
|- 
| 92
| 0x5C
| 4
|	Cluster count
|- 
| 96
| 0x60
| 4
|	Root directory cluster. Typically 4 (but just read this value).
|- 
| 100
| 0x64
| 4
|	Serial number of partition.
|- 
| 104
| 0x68
| 2
|	Filesystem revision
|- 
| 106
| 0x6A
| 2
|	Flags
|- 
| 108
| 0x6C
| 1
|	Sector shift
|- 
| 109
| 0x6D
| 1
|	Cluster shift
|- 
| 110
| 0x6E
| 1
|	Number of FATs
|- 
| 111
| 0x6F
| 1
|	Drive select
|- 
| 112
| 0x70
| 1
|	Percentage in use
|- 
| 113
| 0x71
| 7
|	Reserved (set to 0).
|}

To read the filesystem, find out how big a 'sector' and a 'cluster' are. A sector is (1 << sectorshift) bytes, a cluster is (1 << (sectorshift + clustershift)) bytes. Then, find the start of the FAT and the start of the cluster heap (note that the first cluster is *still* cluster 2).

<syntaxhighlight lang="C">    
// This allows you to zero-index clusters:
uint64_t clusterArray = clusterheapoffset * sectorsize - 2 * clustersize;
uint64_t fatOffset = fatoffset * sectorsize;
uint64_t usablespace = clustercount * clustersize;
</syntaxhighlight>

Note that all values in the BPB are now naturally aligned and that this code is *significantly* simpler than FAT32's BPB reading.

=== File Allocation Table ===
The File Allocation Table (FAT) is a table stored on the storage media that indicates the status and location of all data clusters that are on the disk. It can be considered the "table of contents" of a disk. The cluster may be available for use, it may be reserved by the operating system, it may be unavailable due to a bad sector on the disk, or it may be in use by a file. The clusters of a file need not be right next to each other on the disk. In fact it is likely that they are scattered widely throughout the disk. The FAT allows the operating system to follow the "chain" of clusters in a file. 

==== FAT 12 ====
FAT 12 uses 12 bits to address the clusters on the disk. Each 12 bit entry in the FAT points to the next cluster of a file on the disk. Given a valid cluster number, here is how you extract the value of the next cluster in the cluster chain:
<syntaxhighlight lang="C">
unsigned char FAT_table[sector_size * 2]; // needs two in case we straddle a sector
unsigned int fat_offset = active_cluster + (active_cluster / 2);// multiply by 1.5
unsigned int fat_sector = first_fat_sector + (fat_offset / sector_size);
unsigned int ent_offset = fat_offset % sector_size;

//at this point you need to read two sectors from disk starting at "fat_sector" into "FAT_table".

unsigned short table_value = *(unsigned short*)&FAT_table[ent_offset];

table_value = (active_cluster & 1) ? table_value >> 4 : table_value & 0xfff;

//the variable "table_value" now has the information you need about the next cluster in the chain.
</syntaxhighlight>
If "table_value" is greater than or equal to (>=) 0xFF8 then there are no more clusters in the chain. This means that the whole file has been read. If "table_value" equals (==) 0xFF7 then this cluster has been marked as "bad". "Bad" clusters are prone to errors and should be avoided. If "table_value" is not one of the above cases then it is the cluster number of the next cluster in the file.

The entries under index 0 and 1 are reserved. Index 0 is used as a value in other entries signifying that the given cluster is free, with the corresponding first entry in the table holding the value of the BPB_Media field in its low 8 bits and 0xf in its top 4 bits. For example, if BPB_Media is 0xF8, then the zeroth entry should hold the value 0xFF8. The second entry (index 1) is unused but must hold the value 0xFFF.

FAT12 uses an entry size that is not evenly divisible by 8 bits. This has some consequences.

First is storage in the table. Consider successive entries with values 0x123 and 0x456. In the bytes of the table, they'll be stored 0x23 0x61 0x45. Note that if you do little-endian 16-bit loads, you get 0x6123 at offset 0 and 0x4561 at offset 1, letting you recover the original two entry values with the shifts, masks, and offsets seen in the above code block.

The second is that, as seen above with the offsets used being 0 and 1, those word bytes might not be 16-bit aligned. That usually just means the x86 takes a slower path to load the word if you do e.g. <tt>*(unsigned short *)bytes</tt>, but if you're use something like UBSan to avoid undefined behavior, those UB-catching routines can be triggered (usually resulting in a panic) if you don't load the two bytes separately and stick them together yourself.

The third consequence is that the word bytes might not be *sector* aligned. Which means if your code loads a single sector of the table, it needs a special case where it loads two if the entry straddles the sector-size boundary. Or you can just load two sectors every time as seen above.

==== FAT 16 ====
FAT 16 uses 16 bits to address the clusters on the disk. Because of this, it is much easier to extract the values out of a 16 bit File Allocation Table. Here is how it is done:
<syntaxhighlight lang="C">
unsigned char FAT_table[sector_size];
unsigned int fat_offset = active_cluster * 2;
unsigned int fat_sector = first_fat_sector + (fat_offset / sector_size);
unsigned int ent_offset = fat_offset % sector_size;

//at this point you need to read from sector "fat_sector" on the disk into "FAT_table".

unsigned short table_value = *(unsigned short*)&FAT_table[ent_offset];

//the variable "table_value" now has the information you need about the next cluster in the chain.
</syntaxhighlight>
If "table_value" is greater than or equal to (>=) 0xFFF8 then there are no more clusters in the chain. This means that the whole file has been read. If "table_value" equals (==) 0xFFF7 then this cluster has been marked as "bad". "Bad" clusters are prone to errors and should be avoided. If "table_value" is not one of the above cases then it is the cluster number of the next cluster in the file.

The entries under index 0 and 1 are reserved. The zeroth entry is reserved because index 0 is used as value of other entries signifying that the given cluster is free. Zeroth entry has to hold value of the BPB_Media field from in the low 8 bits, and the rest of the bits have to be set to zero. For example, if BPB_Media is 0xF8, then the zeroth entry should hold the value 0xFFF8. The first entry is reserved for the future and must hold the value 0xFFFF.

==== FAT 32 and exFAT ====
FAT 32 uses 28 bits to address the clusters on the disk. The highest 4 bits are reserved. This means that they should be ignored when read and unchanged when written. exFAT uses the full 32 bit to encode sector numbers. Similar to the same operation on a 16 bit FAT:
<syntaxhighlight lang="C">
unsigned char FAT_table[sector_size];
unsigned int fat_offset = active_cluster * 4;
unsigned int fat_sector = first_fat_sector + (fat_offset / sector_size);
unsigned int ent_offset = fat_offset % sector_size;

//at this point you need to read from sector "fat_sector" on the disk into "FAT_table".

//remember to ignore the high 4 bits.
unsigned int table_value = *(unsigned int*)&FAT_table[ent_offset];
if (fat32) table_value &= 0x0FFFFFFF;

//the variable "table_value" now has the information you need about the next cluster in the chain.
</syntaxhighlight>
If "table_value" is greater than or equal to (>=) 0x0FFFFFF8 (or 0xFFFFFFF8 for exFAT) then there are no more clusters in the chain. This means that the whole file has been read. If "table_value" equals (==) 0x0FFFFFF7 (or 0xFFFFFFF7 for exFAT) then this cluster has been marked as "bad". "Bad" clusters are prone to errors and should be avoided. If "table_value" is not one of the above cases then it is the cluster number of the next cluster in the file.

The entries under index 0 and 1 are reserved. The zeroth entry is reserved because index 0 is used as value of other entries signifying that the given cluster is free. Zeroth entry has to hold value of the BPB_Media field from in the low 8 bits, and the rest of the bits have to be set to zero. For example, if BPB_Media is 0xF8, then the zeroth entry should hold the value 0xFFFFFFF8. The first entry is reserved for the future and must to hold the value 0xFFFFFFFF.

Note that on exFAT, some files are not written out into the FAT. In the case that a file is fully contiguous, exFAT allows the operating system to encode this information and not update the FAT for this file. Unlike FAT32 therefore, the FAT table is not used for allocation status of a cluster; instead there is an allocation bitmap to handle that. See below under directory entries for that.

=== Directories on FAT12/16/32 ===
A directory entry simply stores the information needed to know where a file's data or a folder's children are stored on the disk. It also holds information such as the entry's name, size, and creation time. There are two types of directories in a FAT file system. Standard 8.3 directory entries, which appear on all FAT file systems, and Long File Name directory entries which are optionally present to allow for longer file names.

==== Standard 8.3 format ====
{| {{Wikitable}}
|-
! Offset (in bytes)
! Length (in bytes)
! Meaning
|-
| 0
| 11
| 8.3 file name. The first 8 characters are the name and the last 3 are the extension.
|-
| 11
| 1
| Attributes of the file. The possible attributes are: <pre>READ_ONLY=0x01 HIDDEN=0x02 SYSTEM=0x04 VOLUME_ID=0x08 DIRECTORY=0x10 ARCHIVE=0x20 LFN=READ_ONLY|HIDDEN|SYSTEM|VOLUME_ID </pre> (LFN means that this entry is a [[#Long_File_Names|long file name entry]])
|-
| 12
| 1
| Reserved for use by Windows NT. (Windows NT Repurposed this field to capitalize shorter filenames without using LFN entries. If bit 3 is set the base name is lower case. If bit 4 is set, the extension is lower case)
|-
| 13
| 1
| Creation time in hundredths of a second, although the official FAT Specification from Microsoft says it is tenths of a second. Range 0-199 inclusive. Based on simple tests, Ubuntu16.10 stores either 0 or 100 while Windows7 stores 0-199 in this field.
|-
| 14
| 2
| The time that the file was created. Multiply Seconds by 2.

{| {{Wikitable}}
|-
| Hour 
| 5 bits
|-
| Minutes 
| 6 bits
|-
| Seconds 
| 5 bits
|}

|-
| 16
| 2
| The date on which the file was created.

{| {{Wikitable}}
|- 
| Year
| 7 bits
|-
| Month 
| 4 bits
|-
| Day 
| 5 bits
|}

|-
| 18
| 2
| Last accessed date. Same format as the creation date.
|-
| 20
| 2
| The high 16 bits of this entry's first cluster number. For FAT 12 and FAT 16 this is always zero.
|-
| 22
| 2
| Last modification time. Same format as the creation time.
|-
| 24
| 2
| Last modification date. Same format as the creation date.
|-
| 26
| 2
| The low 16 bits of this entry's first cluster number. Use this number to find the first cluster for this entry.
|-
| 28
| 4
| The size of the file in bytes.
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
| Checksum generated of the short file name when the file was created. The short filename can change without changing the long filename in cases where the partition is mounted on a system which does not support long filenames.
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

=== Directories on exFAT ===
exFAT redesigned these directory entries from the ground up. 

{| {{Wikitable}}
|-
! Offset (in bytes)
! Length (in bytes)
! Meaning
|-
| 0
| 1
| Entry type
|-
| 1
| 31
| Rest of entry.
|}

The base for every entry is that they are all still 32 bytes, and they all start with the type in the first byte. The types I've encountered that are relevant for reading files from disk:

==== File entry ====
{| {{Wikitable}}
|-
! Offset (in bytes)
! Length (in bytes)
! Meaning
|-
| 0
| 1
| Entry type = 0x85
|-
| 1
| 1
| Count of secondary entries.
|-
| 2
| 2
| Checksum of entry set
|-
| 4
| 2
| File attributes
|-
| 6
| 2
| Reserved
|-
| 8
| 4
| Creation date and time
|-
| 12
| 4
| Modification date and time
|-
| 16
| 4
| Access date and time
|-
| 20
| 1
| Creation time in hundredths of a second (0-199) to be added to the FAT style date/time for more accuracy. See FAT12 entry for format of date/time.
|-
| 21
| 1
| Modification time in hundredths of a second (0-199).
|-
| 22
| 1
| UTC offset for creation time
|-
| 23
| 1
| UTC offset for modification time
|-
| 24
| 1
| UTC offset for access time
|-
| 25
| 7
| Reserved.
|}

==== Stream "extension" entry ====
It's called an extension, but it's 100% required to exist directly after the "file" entry.

{| {{Wikitable}}
|-
! Offset (in bytes)
! Length (in bytes)
! Meaning
|-
| 0
| 1
| Entry type = 0xC0
|-
| 1
| 1
| Secondary flags
|-
| 2
| 1
| Reserved
|-
| 3
| 1
| Name length
|-
| 4
| 2
| Name hash
|-
| 6
| 2
| Reserved
|-
| 8
| 8
| Valid data length. When writing large files, exFAT allocates the whole file first, and then incrementally updates this as data is written. Not sure what you're supposed to do with this, if it's not dataLength yell at the user?
|-
| 16
| 4
| Reserved
|-
| 20
| 4
| First cluster.
|-
| 24
| 8
| Data length.
|}

==== File name entry ====
{| {{Wikitable}}
|-
! Offset (in bytes)
! Length (in bytes)
! Meaning
|-
| 0
| 1
| Entry type = 0xC1
|-
| 1
| 1
| flags
|-
| 2
| 30
| File name characters (15 UTF16 code units).
|}

To actually use these, they typically come in the order:

- File entry
- Stream extension entry
- File name entry
- (Additional file name entries)

The file entry has the file metadata info, the stream extension tells you how it's stored and the file name entries tell you what it's called. There is no 8.3 name any more.

When reading the file, the second bit in the stream extension secondary flags indicates if it's stored as extent, or if you need to use the FAT table. If it is set, the file is contiguous and the FAT is not up to date, if it is clear, the FAT is accurate and needs to be used (but could still say it's contiguous).

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
| Checksum generated of the short file name when the file was created. The short filename can change without changing the long filename in cases where the partition is mounted on a system which does not support long filenames.
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


== Programming Guide ==
This section is intended to give you information about common functions that are preformed on a FAT file system.

=== Reading the Boot Sector ===
The Boot Sector is always placed at logical sector number zero. You can either read the boot sector into an array and access it's members that way or you can read it into a structure and access it through the structure. Either way, the values in the boot sector need to be readily available in order to do much of anything with a FAT file system.

Here is an example of some boot sector structures in C. 
<syntaxhighlight lang="c">
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
	unsigned short		bytes_per_sector;
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
</syntaxhighlight>
Important pieces of information that can be extracted from the boot sector include:

'''Total sectors in volume (including VBR):'''
<syntaxhighlight lang="C">
total_sectors = (fat_boot->total_sectors_16 == 0)? fat_boot->total_sectors_32 : fat_boot->total_sectors_16;
</syntaxhighlight>

'''FAT size in sectors:'''
<syntaxhighlight lang="C">
fat_size = (fat_boot->table_size_16 == 0)? fat_boot_ext_32->table_size_16 : fat_boot->table_size_16;
</syntaxhighlight>

'''The size of the root directory (unless you have FAT32, in which case the size will be 0):'''
<syntaxhighlight lang="C">
root_dir_sectors = ((fat_boot->root_entry_count * 32) + (fat_boot->bytes_per_sector - 1)) / fat_boot->bytes_per_sector;
</syntaxhighlight>
This calculation will round up. 32 is the size of a FAT directory in bytes.


'''The first data sector (that is, the first sector in which directories and files may be stored):'''
<syntaxhighlight lang="C">
first_data_sector = fat_boot->reserved_sector_count + (fat_boot->table_count * fat_size) + root_dir_sectors;
</syntaxhighlight>


'''The first sector in the File Allocation Table:'''
<syntaxhighlight lang="C">
first_fat_sector = fat_boot->reserved_sector_count;
</syntaxhighlight>


'''The total number of data sectors:'''
<syntaxhighlight lang="C">
data_sectors = total_sectors - (fat_boot->reserved_sector_count + (fat_boot->table_count * fat_size) + root_dir_sectors);
</syntaxhighlight>


'''The total number of clusters:'''
<syntaxhighlight lang="C">
total_clusters = data_sectors / fat_boot->sectors_per_cluster;
</syntaxhighlight>
This rounds down.

'''The FAT type of this file system:'''
<syntaxhighlight lang="C">
if (sectorsize == 0) 
{
   fat_type = ExFAT;
}
else if(total_clusters < 4085) 
{
   fat_type = FAT12;
} 
else if(total_clusters < 65525) 
{
   fat_type = FAT16;
} 
else
{
   fat_type = FAT32;
}
</syntaxhighlight>

=== Reading Directories ===
The first step in reading directories is finding and reading the root directory. On a FAT 12 or FAT 16 volumes the root directory is at a fixed position immediately after the File Allocation Tables:
<syntaxhighlight lang="C">
first_root_dir_sector = first_data_sector - root_dir_sectors;
</syntaxhighlight>

In FAT32 and exFAT, root directory appears in data area on given cluster and can be a cluster chain. In exFAT it cannot be encoded as extent and will always be present in the FAT.
<syntaxhighlight lang="C">
root_cluster_32 = extBS_32->root_cluster;
</syntaxhighlight>

For each given cluster number we can calculate the first sector of it (relative to the partition's offset):
<syntaxhighlight lang="C">
first_sector_of_cluster = ((cluster - 2) * fat_boot->sectors_per_cluster) + first_data_sector;
</syntaxhighlight>

After the correct cluster has been loaded into memory, the next step is to read and parse all of the entries in it. Each entry is 32 bytes long. For each 32 byte entry this is the flow of execution:
# If the first byte of the entry is equal to 0 then there are no more files/directories in this directory. FirstByte==0, finish. FirstByte!=0, goto 2.
# If the first byte of the entry is equal to 0xE5 then the entry is unused. FirstByte==0xE5, goto 8, FirstByte!=0xE5, goto 3.
# Is this entry a long file name entry? If the 11'th byte of the entry equals 0x0F, then it is a long file name entry. Otherwise, it is not. 11thByte==0x0F, goto 4. 11thByte!=0x0F, goto 5.
# Read the portion of the long filename into a temporary buffer. Goto 8.
# Parse the data for this entry using the table from further up on this page. It would be a good idea to save the data for later. Possibly in a virtual file system structure. goto 6
# Is there a long file name in the temporary buffer? Yes, goto 7. No, goto 8
# Apply the long file name to the entry that you just read and clear the temporary buffer. goto 8
# Increment pointers and/or counters and check the next entry. (goto number 1) 

This process should be repeated until all of the entries have been read from the cluster. You should then check to see if there is another cluster following this one in the cluster chain or if this is the last cluster in the chain. See [[#Following_Cluster_Chains|section below]] and [[#File_Allocation_Table|FAT]] section for more information. You should do the above process for each cluster in the chain, following it until there are no more clusters left in the chain. Then you can check if any of the entries that you just read are directories. If the are they should each be read in the same way starting with their first cluster number which is stored in the entry.

=== Following Cluster Chains ===
There are two basic steps to following cluster chains. The first step is to find out if there is another "link" (cluster) following the current one in the chain. The second step is to actually use the value read from the FAT to read the next sector. Here is the basic idea:
# Extract the value from the FAT for the _current_ cluster. (Use the previous section on the File Allocation Table for details on how exactly to extract the value.) goto number 2
# Is this cluster marked as the last cluster in the chain? (again, see the above section for more details) Yes, goto number 4. No, goto number 3
# Read the cluster represented by the extracted value and return for more directory parsing.
# The end of the cluster chain has been found. Our work here is finished. :)

=== Reading extents ===
On exFAT, files can have a bit set in their flags that indicate it is stored as extent-based. This means that the whole file is contiguous, and that the file size plus the first cluster indicate where the (whole) file is. The FAT entries will contain garbage and are not to be trusted.

To read this, do the same calculation as above, except you may in every step assume that the next cluster is the numerically next cluster and that enough sectors have been allocated for the file size.

== Creating a fresh FAT filesystem ==
Typically during development you want to create a disk image with a FAT filesystem. There are two common approaches for this, either by using a utility that works directly on images, or by using a [[Loopback Device]] and using the OS' own driver to work on the image. A less common alternative is to have an actual disk in your drive.

The most buildscript-friendly tool is [[MTools]] - which can do all operations directly on a disk image using the -i argument and supplies every DOS command related to files in this fashion, only prefixed with an <tt>m</tt>. It can also use a configuration file to access drives in their DOS fashion, allowing you to use for instance A: and C: as actual drives. The tool can be built out of the box for Windows and is included in many a linux package manager.

Linux-only developers can, often with a bit of sudo and permission magic, automate the [[Loopback Device]] in combination with <tt>mkdosfs</tt> or <tt>mkfs.vfat</tt> as well as partition editing. This method is less portable as the commands often can't be reused outside of Linux. Several developers also make the error of passing -F to mkdosfs in an attempt to choose a FAT size, which often has the effect of creating a corrupt filesystem since the result doesn't follow [[#Reading_the_Boot_Sector|the official rule for FAT sizes anymore]].

Windows users can make use of [[Virtual Floppy Drive|VFD]] for loopback devices. It comes with a GUI, but at the cost of not being properly automatable in a script.

==See Also==

=== Threads ===
* from raw bits to directory listing (code posted) in the [[Topic:11247|forum]]
* Public Domain FAT32 code in the [[Topic:13993|forum]]
* FAT12/FAT16 bootsector code in the [[Topic:21155|forum]]

=== External Links ===
* [http://www.osdever.net/downloads/docs/fatgen103.zip FAT32 File System Specification] - from Cottontail OS Development Library
* [http://download.microsoft.com/download/1/6/1/161ba512-40e2-4cc9-843a-923143f3456c/fatgen103.doc FAT32 File System Specification] - from Microsoft (documentation, but in tutorial style with many code examples)
* [http://board.flatassembler.net/topic.php?t=12680 About an error in the above specification]
* http://scottie.20m.com/fat.htm
* http://www.maverick-os.dk/FileSystemFormats/FAT12_FileSystem.html
* http://www.pjrc.com/tech/8051/ide/fat32.html
* [http://web.archive.org/web/20170112194555/http://www.viralpatel.net/taj/tutorial/fat.php Intro into Sectors and Addressing]
* http://elm-chan.org/fsw/ff/00index_e.html - simple (V)FAT12/16/32 read/write library with good documentation
* http://gitorious.org/unix-stuff/fat-util - Utility to read, remove and extract files on FAT12, 16 and 32
* http://www.larwe.com/zws/products/dosfs/index.html - fat12/16/32 compatible fs driver
* http://www.isdaman.com/alsos/protocols/fats/nowhere/FAT.HTM

[[Category:Filesystems]]
[[de:FAT]]
````

<a id="w001507"></a>

## W001507 — HPFS

Bron-ID: W001507
Namespace: Main (0)
Revisie: 30688 | Datum: 2026-06-03T20:31:41Z
Bron: https://wiki.osdev.org/HPFS
Revisie-URL: https://wiki.osdev.org/index.php?title=HPFS&oldid=30688

````text
{{Historical}}
{{Stub}}
{{Filesystems}}

The HPFS was designed by IBM/Microsoft for IBMs new windowing system, OS/2. It was designed to be fast, remove all the shortcomings of FAT, support long filenames, small cluster sizes, remove fragmentation as much as possible and support more attributes.

== About HPFS (High Performace Filesystem) ==

HPFS is the precursor to NTFS and is, in a nutshell, NTFS minus the security features embeded into NTFS. Instead of storing cluster chains in a single linked list format, HPFS stores its information in sorted B-Tree's. This makes searching for files very fast.

Instead of keeping the directory tables and other descriptors at the start of the disk, HPFS bands them at regular intervals throughout the disk and in the middle of the disk, with the theory being that the heads only have to move half as much in any direction.

== Links ==

More information about HPFS can be found at: http://web.archive.org/web/20120204050411/http://www.nondot.org/sabre/os/files/FileSystems/HPFS/

[[Category:Filesystems]]
````

<a id="w001508"></a>

## W001508 — NTFS

Bron-ID: W001508
Namespace: Main (0)
Revisie: 30013 | Datum: 2026-02-15T17:38:49Z
Bron: https://wiki.osdev.org/NTFS
Revisie-URL: https://wiki.osdev.org/index.php?title=NTFS&oldid=30013

````text
{{Filesystems}}

'''NTFS''' (New Technology File System) is [[Windows NT]]'s native file system. Based on [[HPFS]], it adds support for journaling, encryption, sparse files, transparent compression and security features such as access control lists. Filenames are encoded in UTF-16, case-insensitive, with a filename length limit of 255 code points excluding <code>/\:*"?<>|</code> and null (<code>\0</code>) which are reserved.

== About ==
NTFS doesn't only add security features to [[HPFS]]. In NTFS, there is a lot more built-in redundancy. For example: in [[HPFS]], wiping out a sector in the wrong place can render the entire volume inaccessible. Support for multiple hard-links to a file (before NTFS, the only easy access was through the POSIX subsystem, but Windows 2000 (NT 5) added this to Win32 as well) was also added.

NTFS supports an arbitrary number of file forks (much like Mac OS, except Mac OS always has exactly 2 forks for each file).

[[HPFS]] decrees that a cluster is always 512 bytes long and a cluster is always one sector. For the sake of performance and compatibility with some (especially Japanese) machines, NTFS allows sectors of different sizes. It also supports clusters of more than one sector, which can be beneficial on performance.

In short, NTFS' most significant changes:
* Better and more security.
* Multiple hard-links to one file.
* An arbitrary number of forks.
* Variable cluster and sectors sizes (usually resulting in better performance).

== Implementation ==
Due to its proprietary nature and lack of documentation, NTFS is a difficult file system to implement.

The NTFS-3G project (GPLv2-licensed) provides a [[FUSE]]-based read/write implementation of the filesystem for Linux, BSD, MacOS and other operating systems.
Linux also support NTFS natively through the NTFS3 read/write driver since version 5.15.

== Structure ==

The NTFS format is built around "file" tables that allow both pre-defined and custom attributes to be stored and read by the operating system.

The NTFS boot sector is similar to other file systems, like FAT.

{| {{wikitable}}
! Field
! Type
|-
|JMP || int8_t[3]
|-
|OEM System || char[8]
|-
|Bytes Per Sector || uint16_t
|-
|Sectors Per Cluster || int8_t
|-
|Reserved Sector Count || uint16_t
|-
|Table Count || int8_t
|-
|Root Entry Count || uint16_t
|-
|Sector Count || uint16_t
|-
|Media Type || int8_t
|-
|Sectors Per Table || uint16_t
|-
|Sectors Per Track || uint16_t
|-
|Heads || uint16_t
|-
|Hidden Sector Count || uint32_t
|-
|Sector Count (32-bit) || uint32_t
|-
|Reserved || uint32_t
|-
|Sector Count (64-bit) || uint64_t
|}

This is followed immediately by a NTFS specific header.

{| {{wikitable}}
!Field
!Type
|-
|Master File Table Cluster || uint64_t
|-
|Master File Table Mirror Cluster || uint64_t
|-
|Clusters Per Record || int8_t
|-
|Reserved || int8_t[3]
|-
|Clusters Per Index Buffer || int8_t
|-
|Reserved || int8_t[3]
|-
|Serial Number || uint64_t
|-
|Checksum || uint32_t
|}

Using the "Master File Table Cluster" and "Sectors Per Cluster" values, you can find the Master File Table.  This table contains entries for every object in the file system, including files, folders, and the tables themselves.  The size of each record in the Master File Table can be calculated using the "Clusters Per Record" and "Sectors Per Cluster" fields from the boot sector.

Each record starts with the same header structure.

{| {{wikitable}}
! Field
! Type
|-
|Record Type || char[4]
|-
|Update Sequence Offset || uint16_t
|-
|Update Sequence Length || uint16_t
|-
|Log File Sequence Number || uint64_t
|-
|Record Sequence Number || uint16_t
|-
|Hard Link Count || uint16_t
|-
|Attributes Offset || uint16_t
|-
|Flags || uint16_t
|-
|Bytes In Use || uint32_t
|-
|Bytes Allocated || uint32_t
|-
|Parent Record Number || uint64_t
|-
|Next Attribute Index || uint32_t
|-
|Reserved || uint32_t
|-
|Record Number || uint64_t
|}

The remainder of the file record contains additional tables and data for this record.  The "Attributes Offset" field contains the byte offset (from the start of the record) of the beginning of the attribute list for this record.

Attributes have a variable length, but always start with the same sequence.

{| {{wikitable}}
! Field
! Type
|-
| Attribute Type || uint32_t
|}

If the "Attribute Type" field contains the value 0xffffffff, this marks the end of the attribute list.  Otherwise, the attribute sequence continues with the length of the attribute "record".

{| {{wikitable}}
! Field
! Type
|-
| Attribute Length || uint32_t
|}

This length value defines the total length of the attribute record, including the "Attribute Type" and "Attribute Length" fields.

== See Also ==
=== External Links ===
* [https://web.archive.org/web/20210922203602/https://flatcap.org/linux-ntfs/ntfs/index.html Guide to NTFS]
* [https://dubeyko.com/development/FileSystems/NTFS/ntfsdoc.pdf NTFS Documentation], reverse-engineered, originally for the Linux-NTFS project.
* [https://www.writeblocked.org/resources/NTFS_CHEAT_SHEETS.pdf Cheat sheet]
* [http://www.linux-ntfs.org/ The Linux NTFS project]
* [http://www.ntfs-3g.org/ NTFS-3G Read/Write Drivers For Linux/FreeBSD/BeOS]
* [http://git.kernel.org/?p=linux/kernel/git/torvalds/linux-2.6.git;a=tree;f=fs/ntfs;hb=HEAD Linux fs/ntfs tree]
* [http://www.opensource.apple.com/source/ntfs/ Apple Open Source NTFS site]
* [https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc#attribute_chains libfsntfs library documentation]
[[Category:Filesystems]]
[[de:NTFS]]
````

<a id="w001509"></a>

## W001509 — Ext2

Bron-ID: W001509
Namespace: Main (0)
Revisie: 29719 | Datum: 2025-10-19T14:43:19Z
Bron: https://wiki.osdev.org/Ext2
Revisie-URL: https://wiki.osdev.org/index.php?title=Ext2&oldid=29719

````text
{{Filesystems}}
The '''Second Extended Filesystem''' ('''ext2fs''') is a rewrite of the original ''Extended Filesystem'' and as such, is also based around the concept of "inodes."  Ext2 served as the de facto filesystem of Linux for nearly a decade from the early 1990s to the early 2000s when it was superseded by the journaling file systems [[Ext3|ext3]] and [[ReiserFS]]. It has native support for UNIX ownership / access rights, symbolic- and hard-links, and other properties that are common among UNIX-like operating systems. Organizationally, it divides disk space up into groups called "block groups." Having these groups results in distribution of data across the disk which helps to minimize head movement as well as the impact of fragmentation. Further, some (if not all) groups are required to contain backups of important data that can be used to rebuild the file system in the event of disaster.

''Note: Most of the information here is based off of work done by Dave Poirier on the ext2-doc project (see the [[#External Links|links section]]) which is graciously released under the [http://www.fsf.org/licenses/fdl.html GNU Free Documentation License].  Be sure to buy him a beer the next time you see him.''

== Basic Concepts ==
'''Important Note: All values are little-endian unless otherwise specified'''

=== What is a Block? ===
The Ext2 file system divides up disk space into logical blocks of contiguous space. The size of blocks need not be the same size as the sector size of the disk the file system resides on.  The size of blocks can be determined by reading the field starting at byte 24 in the [[#Superblock|Superblock]].

=== What is a Block Group? ===
Blocks, along with inodes, are divided up into "block groups." These are nothing more than contiguous groups of blocks.

Each block group reserves a few of its blocks for special purposes such as:
* A bitmap of free/allocated blocks within the group
* A bitmap of allocated inodes within the group
* A table of inode structures that belong to the group
* Depending upon the revision of Ext2 used, some or all block groups may also contain a backup copy of the [[#Superblock|Superblock]] and the [[#Block_Group_Descriptor_Table|Block Group Descriptor Table]].

=== What is an Inode? ===
An inode is a structure on the disk that represents a file, directory, symbolic link, etc. Inodes do not contain the data of the file / directory / etc. that they represent. Instead, they link to the blocks that actually contain the data.  This lets the inodes themselves have a well-defined size which lets them be placed in easily indexed arrays.  Each block group has an array of inodes it is responsible for, and conversely every inode within a file system belongs to one of such tables (and one of such block groups).

== Superblock ==
The first step in implementing an Ext2 driver is to find, extract, and parse the superblock.  The Superblock contains all information about the layout of the file system and possibly contains other important information like what optional features were used to create the file system.  Once you have finished with the Superblock, the next step is to look at the [[#Block_Group_Descriptor_Table|Block Group Descriptor Table]]

=== Locating the Superblock ===
The Superblock is always located at byte 1024 from the beginning of the volume and is exactly 1024 bytes in length. For example, if the disk uses 512 byte sectors, the Superblock will begin at LBA 2 and will occupy all of sector 2 and 3.

=== Determining the Number of Block Groups ===
From the Superblock, extract the size of each block, the total number of inodes, the total number of blocks, the number of blocks per block group, and the number of inodes in each block group. From this information we can infer the number of block groups there are by:
* Rounding up the total number of blocks divided by the number of blocks per block group
* Rounding up the total number of inodes divided by the number of inodes per block group
* Both (and check them against each other)

=== Base Superblock Fields ===
These fields are present in all versions of Ext2
{| {{wikitable}}
! Starting
Byte
! Ending
Byte
! Size
in Bytes
! Field Description
|-
| 0 || 3 || 4 || Total number of inodes in file system
|-
| 4 || 7 || 4 || Total number of blocks in file system
|-
| 8 || 11 || 4 || Number of blocks reserved for superuser (see offset 80)
|-
| 12 || 15 || 4 || Total number of unallocated blocks
|-
| 16 || 19 || 4 || Total number of unallocated inodes
|-
| 20 || 23 || 4 || Block number of the block containing the superblock (also the starting block number, NOT always zero.)
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
| 56 || 57 || 2 || Ext2 signature (0xef53), used to help confirm the presence of Ext2 on a volume
|-
| 58 || 59 || 2 || File system state ([[#File_System_States|see below]])
|-
| 60 || 61 || 2 || What to do when an error is detected ([[#Error_Handling_Methods|see below]])
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
==== File System States ====
{| {{Wikitable}}
! Value
! State Description
|-
| 1 || File system is clean
|-
| 2 || File system has errors
|}
==== Error Handling Methods ====
{| {{Wikitable}}
! Value
! Action to Take
|-
| 1 || Ignore the error (continue on)
|-
| 2 || Remount file system as read-only
|-
| 3 || Kernel panic
|}
==== Creator Operating System IDs ====
{| {{Wikitable}}
! Value
! Operating System
|-
| 0 || [http://kernel.org/ Linux]
|-
| 1 || [http://www.gnu.org/software/hurd/hurd.html GNU HURD]
|-
| 2 || MASIX (an operating system developed by Rémy Card, one of the developers of ext2)
|-
| 3 || [http://www.freebsd.org/ FreeBSD]
|-
| 4 || Other "Lites" (BSD4.4-Lite derivatives such as [http://www.netbsd.org/ NetBSD], [http://www.openbsd.org/ OpenBSD], [http://www.opensource.apple.com/source/xnu/ XNU/Darwin], etc.)
|}

=== Extended Superblock Fields ===
These fields are only present if Major version (specified in the base superblock fields), is greater than or equal to 1.
{| {{Wikitable}}
! Starting
Byte
! Ending
Byte
! Size
in Bytes
! Field Description
|-
| 84 || 87 || 4 || First non-reserved inode in file system. (In versions < 1.0, this is fixed as 11)
|-
| 88 || 89 || 2 || Size of each inode structure in bytes. (In versions < 1.0, this is fixed as 128)
|-
| 90 || 91 || 2 || Block group that this superblock is part of (if backup copy)
|-
| 92 || 95 || 4 || Optional features present (features that are not required to read or write, but usually result in a performance increase. [[#Optional_Feature_Flags|see below]])
|-
| 96 || 99 || 4 || Required features present (features that are required to be supported to read or write. [[#Required_Feature_Flags|see below]])
|-
| 100 || 103 || 4 || Features that if not supported, the volume must be mounted read-only [[#Read-Only_Feature_Flags|see below]])
|-
| 104 || 119 || 16 || File system ID (what is output by blkid)
|-
| 120 || 135 || 16 || Volume name (C-style string: characters terminated by a 0 byte)
|-
| 136 || 199 || 64 || Path volume was last mounted to (C-style string: characters terminated by a 0 byte)
|-
| 200 || 203 || 4 || Compression algorithms used (see Required features above)
|-
| 204 || 204 || 1 || Number of blocks to preallocate for files
|-
| 205 || 205 || 1 || Number of blocks to preallocate for directories
|-
| 206 || 207 || 2 || (Unused)
|-
| 208 || 223 || 16 || Journal ID (same style as the File system ID above)
|-
| 224 || 227 || 4 || Journal inode
|-
| 228 || 231 || 4 || Journal device
|-
| 232 || 235 || 4 || Head of orphan inode list
|-
| 236 || 1023 || X || (Unused)
|}
==== Optional Feature Flags ====
These are optional features for an implementation to support, but offer performance or reliability gains to implementations that do support them.
{| {{Wikitable}}
|-
! Flag Value
! Description
|-
| 0x0001 || Preallocate some number of (contiguous?) blocks (see byte 205 in the superblock) to a directory when creating a new one (to reduce fragmentation?)
|-
| 0x0002 || AFS server inodes exist
|-
| 0x0004 || File system has a journal (Ext3)
|-
| 0x0008 || Inodes have extended attributes
|-
| 0x0010 || File system can resize itself for larger partitions
|-
| 0x0020 || Directories use hash index
|}

==== Required Feature Flags ====
These features if present on a file system are required to be supported by an implementation in order to correctly read from or write to the file system.
{| {{Wikitable}}
|-
! Flag Value
! Description
|-
| 0x0001 || Compression is used
|-
| 0x0002 || Directory entries contain a type field
|-
| 0x0004 || File system needs to replay its journal
|-
| 0x0008 || File system uses a journal device
|}

==== Read-Only Feature Flags ====
These features, if present on a file system, are required in order for an implementation to write to the file system, but are not required to read from the file system.
{| {{Wikitable}}
|-
! Flag Value
! Description
|-
| 0x0001 || Sparse superblocks and group descriptor tables
|-
| 0x0002 || File system uses a 64-bit file size
|-
| 0x0004 || Directory contents are stored in the form of a [http://en.wikipedia.org/wiki/Binary_tree Binary Tree]
|}

== Block Group Descriptor Table ==
The Block Group Descriptor Table contains a descriptor for each block group within the file system. The number of block groups within the file system, and correspondingly, the number of entries in the Block Group Descriptor Table, is described [[#Determining_the_Number_of_Block_Groups|above]].  Each descriptor contains information regarding where important data structures for that group are located.

=== Locating the Block Group Descriptor Table ===
The table is located in the block immediately following the Superblock. So if the block size (determined from a field in the superblock) is 1024 bytes per block, the Block Group Descriptor Table will begin at block 2.  For any other block size, it will begin at block 1.  Remember that blocks are numbered starting at 0, and that block numbers don't usually correspond to physical block addresses.

=== Block Group Descriptor ===
A Block Group Descriptor contains information regarding where important data structures for that block group are located.
{| {{Wikitable}}
! Starting
Byte
! Ending
Byte
! Size
in Bytes
! Field Description
|-
| 0 || 3 || 4 || Block address of block usage bitmap
|-
| 4 || 7 || 4 || Block address of inode usage bitmap
|-
| 8 || 11 || 4 || Starting block address of inode table
|-
| 12 || 13 || 2 || Number of unallocated blocks in group
|-
| 14 || 15 || 2 || Number of unallocated inodes in group
|-
| 16 || 17 || 2 || Number of directories in group
|-
| 18 || 31 || X || (Unused)
|}

== Inodes ==
Like blocks, each inode has a numerical address. It is extremely important to note that unlike block addresses, '''inode addresses start at 1'''.

With Ext2 versions prior to Major version 1, inodes 1 to 10 are reserved and should be in an allocated state. Starting with version 1, the first non-reserved inode is indicated via a field in the Superblock.  Of the reserved inodes, number 2 subjectively has the most significance as it is used for the root directory.

Inodes have a fixed size of either 128 for version 0 Ext2 file systems, or as dictated by the field in the Superblock for version 1 file systems.  All inodes reside in inode tables that belong to block groups.  Therefore, looking up an inode is simply a matter of determining which block group it belongs to and indexing that block group's inode table.

=== Determining which Block Group contains an Inode ===
From an inode address (remember that they start at 1), we can determine which group the inode is in, by using the formula:

   block group = (inode – 1) / INODES_PER_GROUP

where INODES_PER_GROUP is a field in the Superblock

=== Finding an inode inside of a Block Group ===
Once we know which group an inode resides in, we can look up the actual inode by first retrieving that block group's inode table's starting address (see [[#Block_Group_Descriptor|Block Group Descriptor]] above).  The index of our inode in this block group's inode table can be determined by using the formula:

   index = (inode – 1) % INODES_PER_GROUP

where % denotes the [http://en.wikipedia.org/wiki/Modulo_operation Modulo operation] and INODES_PER_GROUP is a field in the Superblock (the same field which was used to determine which block group the inode belongs to).

Next, we have to determine which block contains our inode.  This is achieved from:

   containing block = (index * INODE_SIZE) / BLOCK_SIZE

where INODE_SIZE is either fixed at 128 if VERSION < 1 or defined by a field in the Superblock if VERSION >= 1.0, and BLOCK_SIZE is defined by a field in the Superblock.

Finally, mask and shift as necessary to extract only the inode data from the containing block.

=== Reading the contents of an inode ===
Each inode contains 12 direct pointers, one singly indirect pointer, one doubly indirect block pointer, and one triply indirect pointer.  The direct space "overflows" into the singly indirect space, which overflows into the doubly indirect space, which overflows into the triply indirect space.

'''Direct Block Pointers''': There are 12 direct block pointers.  If valid, the value is non-zero.  Each pointer is the block address of a block containing data for this inode.

'''Singly Indirect Block Pointer''': If a file needs more than 12 blocks, a separate block is allocated to store the block addresses of the remaining data blocks needed to store its contents. This separate block is called an indirect block because it adds an extra step (a level of indirection) between an inode and its data. The block addresses stored in the block are all 32-bit, and the capacity of stored addresses in this block is a function of the block size. The address of this indirect block is stored in the inode in the "Singly Indirect Block Pointer" field.

'''Doubly Indirect Block Pointer''': If a file has more blocks than can fit in the 12 direct pointers and the indirect block, a double indirect block is used. A double indirect block is an extension of the indirect block described above only now we have two intermediate blocks between the inode and data blocks. The inode structure has a "Doubly Indirect Block Pointer" field that points to this block if necessary.

'''Triply Indirect Block Pointer''': Lastly, if a file needs still more space, it can use a triple indirect block. Again, this is an extension of the double indirect block.  So, a triple indirect block contains addresses of double indirect blocks, which contain addresses of single indirect blocks, which contain address of data blocks. The inode structure has a "Triply Indirect Block Pointer" field that points to this block if present.

[http://en.wikipedia.org/wiki/File:Ext2-inode.gif This image from Wikipedia] illustrates what is described above pretty well.

=== Inode Data Structure ===

{| {{Wikitable}}
! Starting
Byte
! Ending
Byte
! Size
in Bytes
! Field Description
|-
| 0 || 1 || 2 || Type and Permissions ([[#Inode_Type_and_Permissions|see below]])
|-
| 2 || 3 || 2 || User ID
|-
| 4 || 7 || 4 || Lower 32 bits of size in bytes
|-
| 8 || 11 || 4 || Last Access Time (in [http://en.wikipedia.org/wiki/Unix_time POSIX time])
|-
| 12 || 15 || 4 || Creation Time (in [http://en.wikipedia.org/wiki/Unix_time POSIX time])
|-
| 16 || 19 || 4 || Last Modification time (in [http://en.wikipedia.org/wiki/Unix_time POSIX time])
|-
| 20 || 23 || 4 || Deletion time (in [http://en.wikipedia.org/wiki/Unix_time POSIX time])
|-
| 24 || 25 || 2 || Group ID
|-
| 26 || 27 || 2 || Count of hard links (directory entries) to this inode. When this reaches 0, the data blocks are marked as unallocated.
|-
| 28 || 31 || 4 || Count of disk sectors (not Ext2 blocks) in use by this inode, not counting the actual inode structure nor directory entries linking to the inode.
|-
| 32 || 35 || 4 || Flags ([[#Inode_Flags|see below]])
|-
| 36 || 39 || 4 || [[#OS_Specific_Value_1|Operating System Specific value #1]]
|-
| 40 || 43 || 4 || Direct Block Pointer 0
|-
| 44 || 47 || 4 || Direct Block Pointer 1
|-
| 48 || 51 || 4 || Direct Block Pointer 2
|-
| 52 || 55 || 4 || Direct Block Pointer 3
|-
| 56 || 59 || 4 || Direct Block Pointer 4
|-
| 60 || 63 || 4 || Direct Block Pointer 5
|-
| 64 || 67 || 4 || Direct Block Pointer 6
|-
| 68 || 71 || 4 || Direct Block Pointer 7
|-
| 72 || 75 || 4 || Direct Block Pointer 8
|-
| 76 || 79 || 4 || Direct Block Pointer 9
|-
| 80 || 83 || 4 || Direct Block Pointer 10
|-
| 84 || 87 || 4 || Direct Block Pointer 11
|-
| 88 || 91 || 4 || Singly Indirect Block Pointer (Points to a block that is a list of block pointers to data)
|-
| 92 || 95 || 4 || Doubly Indirect Block Pointer (Points to a block that is a list of block pointers to Singly Indirect Blocks)
|-
| 96 || 99 || 4 || Triply Indirect Block Pointer (Points to a block that is a list of block pointers to Doubly Indirect Blocks)
|-
| 100 || 103 || 4 || Generation number (Primarily used for NFS)
|-
| 104 || 107 || 4 || In Ext2 version 0, this field is reserved. In version >= 1, Extended attribute block (File ACL). 
|-
| 108 || 111 || 4 || In Ext2 version 0, this field is reserved. In version >= 1, Upper 32 bits of file size (if feature bit set) if it's a file, Directory ACL if it's a directory
|-
| 112 || 115 || 4 || Block address of fragment
|-
| 116 || 127 || 12 || [[#OS_Specific_Value_2|Operating System Specific Value #2]]
|}

==== Inode Type and Permissions ====
{| {{Wikitable}}
|+The type indicator occupies the top hex digit (bits 15 to 12) of this 16-bit field
! Type value
in hex
! Type Description
|-
| 0x1000 || FIFO
|-
| 0x2000 || Character device
|-
| 0x4000 || Directory
|-
| 0x6000 || Block device
|-
| 0x8000 || Regular file
|-
| 0xA000 || Symbolic link
|-
| 0xC000 || Unix socket
|}
{| {{Wikitable}}
|+Permissions occupy the bottom 12 bits of this 16-bit field
! Permission
value in hex
! Permission
value in octal
! Permission Description
|-
| 0x001 || 00001 || [http://en.wikipedia.org/wiki/Filesystem_permissions#Traditional_Unix_permissions Other—execute permission]
|-
| 0x002 || 00002 || [http://en.wikipedia.org/wiki/Filesystem_permissions#Traditional_Unix_permissions Other—write permission]
|-
| 0x004 || 00004 || [http://en.wikipedia.org/wiki/Filesystem_permissions#Traditional_Unix_permissions Other—read permission]
|-
| 0x008 || 00010 || [http://en.wikipedia.org/wiki/Filesystem_permissions#Traditional_Unix_permissions Group—execute permission]
|-
| 0x010 || 00020 || [http://en.wikipedia.org/wiki/Filesystem_permissions#Traditional_Unix_permissions Group—write permission]
|-
| 0x020 || 00040 || [http://en.wikipedia.org/wiki/Filesystem_permissions#Traditional_Unix_permissions Group—read permission]
|-
| 0x040 || 00100 || [http://en.wikipedia.org/wiki/Filesystem_permissions#Traditional_Unix_permissions User—execute permission]
|-
| 0x080 || 00200 || [http://en.wikipedia.org/wiki/Filesystem_permissions#Traditional_Unix_permissions User—write permission]
|-
| 0x100 || 00400 || [http://en.wikipedia.org/wiki/Filesystem_permissions#Traditional_Unix_permissions User—read permission]
|-
| 0x200 || 01000 || [http://en.wikipedia.org/wiki/Sticky_bit Sticky Bit]
|-
| 0x400 || 02000 || Set group ID
|-
| 0x800 || 04000 || Set user ID
|}

==== Inode Flags ====
{| {{Wikitable}}
|-
! Flag Value
! Description
|-
| 0x00000001 || Secure deletion (not used)
|-
| 0x00000002 || Keep a copy of data when deleted (not used)
|-
| 0x00000004 || File compression (not used)
|-
| 0x00000008 || Synchronous updates—new data is written immediately to disk
|-
| 0x00000010 || Immutable file (content cannot be changed)
|-
| 0x00000020 || Append only
|-
| 0x00000040 || File is not included in 'dump' command
|-
| 0x00000080 || Last accessed time should not updated
|-
| ... || (Reserved)
|-
| 0x00010000 || Hash indexed directory
|-
| 0x00020000 || AFS directory
|-
| 0x00040000 || Journal file data
|}
==== OS Specific Value 1 ====
{| {{Wikitable}}
! Operating
System
! How they use this field
|-
| Linux || (reserved)
|-
| HURD || "translator"?
|-
| MASIX || (reserved)
|}
==== OS Specific Value 2 ====
{| {{Wikitable}}
! Operating
System
! How they use this field
|-
| Linux ||
{| {{Wikitable}}
! Starting
Byte
! Ending
Byte
! Size
in Bytes
! Field Description
|-
| 116 || 116 || 1 || Fragment number
|-
| 117 || 117 || 1 || Fragment size
|-
| 118 || 119 || 2 || (reserved)
|-
| 120 || 121 || 2 || High 16 bits of 32-bit User ID
|-
| 122 || 123 || 2 || High 16 bits of 32-bit Group ID
|-
| 124 || 127 || 4 || (reserved)
|}
|-
| HURD ||
{| {{Wikitable}}
! Starting
Byte
! Ending
Byte
! Size
in Bytes
! Field Description
|-
| 116 || 116 || 1 || Fragment number
|-
| 117 || 117 || 1 || Fragment size
|-
| 118 || 119 || 2 || High 16 bits of 32-bit "Type and Permissions" field
|-
| 120 || 121 || 2 || High 16 bits of 32-bit User ID
|-
| 122 || 123 || 2 || High 16 bits of 32-bit Group ID
|-
| 124 || 127 || 4 || User ID of author (if == 0xFFFFFFFF, the normal User ID will be used)
|}
|-
| MASIX ||
{| {{Wikitable}}
! Starting
Byte
! Ending
Byte
! Size
in Bytes
! Field Description
|-
| 116 || 116 || 1 || Fragment number
|-
| 117 || 117 || 1 || Fragment size
|-
| 118 || 127 || X || (reserved)
|}
|}

=== Directories ===
Directories are inodes which contain some number of "entries" as their contents.  These entries are nothing more than a name/inode pair.  For instance the inode corresponding to the root directory might have an entry with the name of "etc" and an inode value of 50. A directory inode stores these entries in a linked-list fashion in its contents blocks.

The root directory is Inode 2.

The total size of a directory entry may be longer then the length of the name would imply (The name may not span to the end of the record), and records have to be aligned to 4-byte boundaries. Directory entries are also not allowed to span multiple blocks on the file-system, so there may be empty space in-between directory entries. Empty space is however not allowed in-between directory entries, so any possible empty space will be used as part of the preceding record by increasing its record length to include the empty space. Empty space may also be equivalently marked by a separate directory entry with an inode number of zero, indicating that directory entry should be skipped.

=== Directory Entry ===

{| {{Wikitable}}
! Starting
Byte
! Ending
Byte
! Size
in Bytes
! Field Description
|-
| 0 || 3 || 4 || Inode
|-
| 4 || 5 || 2 || Total size of this entry (Including all subfields)
|-
| 6 || 6 || 1 || Name Length least-significant 8 bits
|-
| 7 || 7 || 1 || [[#Directory_Entry_Type_Indicators|Type indicator]] (only if the feature bit for "directory entries have file type byte" is set, else this is the most-significant 8 bits of the Name Length)
|-
| 8 || 8+N-1 || N || Name characters
|}


==== Directory Entry Type Indicators ====

{| {{Wikitable}}
|-
! Value
! Type Description
|-
| 0 || Unknown type
|-
| 1 || Regular file
|-
| 2 || Directory
|-
| 3 || Character device
|-
| 4 || Block device 
|-
| 5 || FIFO	 
|-
| 6 || Socket
|-
| 7 || Symbolic link (soft link)
|}

== Quick Summaries ==
=== How To Read An Inode ===
# Read the Superblock to find the size of each block, the number of blocks per group, number Inodes per group, and the starting block of the first group (Block Group Descriptor Table).
# Determine which block group the inode belongs to.
# Read the Block Group Descriptor corresponding to the Block Group which contains the inode to be looked up.
# From the Block Group Descriptor, extract the location of the block group's inode table.
# Determine the index of the inode in the inode table.
# Index the inode table (taking into account non-standard inode size).

Directory entry information and file contents are located within the data blocks that the Inode points to.

=== How To Read the Root Directory ===
The root directory's inode is defined to always be 2. Read/parse the contents of inode 2.

==See Also==
===External Links===
* [http://www.nongnu.org/ext2-doc/ ext2-doc project: Second Extended File System] - implementation-oriented documentation, describes internal structure in human language.
* [http://web.mit.edu/tytso/www/linux/ext2intro.html Design and Implementation of the Second Extended Filesystem] (overview)
* [http://ext2.sourceforge.net/2005-ols/paper-html/ State of the Art: Where we are with the Ext3 filesystem] - Paper by Mingming Cao, Theodore Y. Ts'o, Badari Pulavarty, and Suparna Bhattacharya describing extended features for ext2

[[Category:Filesystems]]
[[de:Ext2]]
````

<a id="w001510"></a>

## W001510 — Talk:Ext2

Bron-ID: W001510
Namespace: Talk (1)
Revisie: 13918 | Datum: 2012-08-19T09:51:57Z
Bron: https://wiki.osdev.org/Talk:Ext2
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AExt2&oldid=13918

````text
== Page contents ==
* should we keep ext2 / ext3 together?
* should this page be called ext2 / e2fs / ext2fs ?
- [[User:Combuster|Combuster]] 07:48, 6 December 2006 (CST)


My personal preference would be for the ext3 page to list only the differences between the two. But I am not opposed to them being in the same article.

-[[User:Jhawthorn|Jhawthorn]] 00:58, 7 December 2006 (CST)

== License conflict ==

This page is technically licensed via GNU Documentation license, which is incompatible with the current cc0 license that we are using on the rest of the wiki. I suggest flagging a review. --[[User:Primis|Primis]] 02:40, 19 July 2012 (CDT)

: There's a whole world of difference between ideas and expressions. Someone has taken the effort here to write up a new explanation of the subject, and of course the author will have had to reference other sources - most of them likely being "all rights reserved" and therefore [[Licensing|worse than the GPL]]. If you can't copy other people's ideas you would never be able to write a thing. Instead, '''copy'''right is about duplicating other people's work, not about providing the same information in a '''different''' way. have you found any piece of text that's been (mostly) '''copied''' from another source? I've gone through the reference in question and couldn't find anything obvious. - [[User:Combuster|Combuster]] 12:24, 19 July 2012 (CDT)

== byte 206 and 207 reserve gdt size. ==

Bytes 206 and 207 of superblock are used to store reserve gdt size in ext3 to allow online growing and shrinking file system.
````

<a id="w001511"></a>

## W001511 — ReiserFS

Bron-ID: W001511
Namespace: Main (0)
Revisie: 11686 | Datum: 2011-07-02T18:13:32Z
Bron: https://wiki.osdev.org/ReiserFS
Revisie-URL: https://wiki.osdev.org/index.php?title=ReiserFS&oldid=11686

````text
{{Filesystems}}

== About ReiserFS ==
ReiserFS is a [[journaling]] file system that is free if your OS is free, and comes at a licensing cost if your OS is not free. It has excellent performance on large directories and small files, using "dancing trees" instead of B-trees, and does meta-data journaling to improve file-system stability across system crashes.

Unlike "classic" filesystems, Reiser allows you to have files that occupy less than one sector on the disk (i.e. it can store several tiny files or tails of files on the same sector) through its tree organization.

As of version 3.6, ReiserFS supports:
* A maximum of 2<sup>32</sup> - 3 files in total
* A maximum of 2<sup>32</sup> - 4 files in a single directory (In practice this value is limited by the hash function. The r5 hash allows about 1 200 000 file names without collisions.)
* A theoretical maximum file size of 1 EiB (2<sup>60</sup>) but the page cache limits this to 16 TiB on architectures with 32 bit integers.
* A maximum of 2<sup>32</sup> number links to a file
* and A maximum filesystem size of 16 TiB (2<sup>32</sup>) using 4KiB blocks

== Links ==
* http://www.namesys.com - the ReiserFS homepage 
* http://p-nand-q.com/download/rfstool/reiserfs_docs.html - Documentation regarding the format of ReiserFS
* http://en.wikipedia.org/wiki/ReiserFS
[[Category:Filesystems]]
````

<a id="w001512"></a>

## W001512 — BeFS

Bron-ID: W001512
Namespace: Main (0)
Revisie: 29835 | Datum: 2026-01-07T06:35:01Z
Bron: https://wiki.osdev.org/BeFS
Revisie-URL: https://wiki.osdev.org/index.php?title=BeFS&oldid=29835

````text
{{Filesystems}}
'''Be File System (BeFS)''' is the native filesystem of '''BeOS''' and '''Haiku'''. It was designed in the mid-1990s to support the operating system’s core goals: responsiveness, multimedia performance, and pervasive metadata. Unlike many contemporaneous filesystems, BeFS treats files not merely as byte streams, but as structured objects with indexed attributes, enabling database-like queries directly at the filesystem level.

==Basic Details==
{| class="wikitable"
|+
|Block Sizes
|1024, 2048, 4096, or 8192 bytes
|-
|Directory Structures
|B+ Trees
|-
|Max Volume Size
|2 Exabytes
|-
|Max File Size
|~260GB
|-
|Max Filename Length
|255 Bytes
|-
|Filesystem Encoding
|UTF-8
|}

==On-Disk Structure==

===Superblock===
 typedef struct disk_super_block
 {
 char name[B_OS_NAME_LENGTH];
 int32 magic1;
 int32 fs_byte_order;
 uint32 block_size;
 uint32 block_shift;
 off_t num_blocks;
 off_t used_blocks;
 int32 inode_size;
 int32 magic2;
 int32 blocks_per_ag;
 int32 ag_shift;
 int32 num_ags;
 int32 flags;
 block_run log_blocks;
 off_t log_start;
 off_t log_end;
 int32 magic3;
 inode_addr root_dir;
 inode_addr indices;
 int32 pad[8];
 } disk_super_block;

*<code>name</code> The human-readable name (volume label) of the BeFS filesystem, limited to 32 characters.
*<code>magic1</code> First magic number (0x42465331, ASCII "BFS1") used to identify a valid BeFS superblock.
*<code>fs_byte_order</code> Indicates the byte order of the filesystem.
*<code>block_size</code> Size of a filesystem block in bytes (typically a power of 2, such as 1024, 2048, etc.).
*<code>block_shift</code> Log base 2 of the block size, used for quick block-to-byte conversions.
*<code>num_blocks</code> Total number of blocks in the filesystem volume.
*<code>used_blocks</code> Number of blocks currently allocated (used) in the volume.
*<code>inode_size</code> Size of each inode structure on disk (typically 2048 bytes in modern implementations).
*<code>magic2</code> Second magic number (0xdd121031) for additional superblock validation.
*<code>blocks_per_ag</code> Number of blocks per allocation group.
*<code>ag_shift</code> Log base 2 of blocks per allocation group, for efficient calculations.
*<code>num_ags</code> Total number of allocation groups in the filesystem.
*<code>flags</code> Filesystem flags, including clean/dirty state (e.g., 'BFS_CLEAN' for clean, 'BFS_DIRTY' for dirty).
*<code>log_blocks</code> Block run describing the location and extent of the journal log blocks.
*<code>log_start</code> Starting block number of the journal log.
*<code>log_end</code> Ending block number of the journal log.
*<code>magic3</code> Third magic number (0x15b6830e) for further superblock integrity checking.
*<code>root_dir</code> Address (as an inode_addr/block_run) of the root directory inode.
*<code>indices</code> Address (as an inode_addr/block_run) of the indices root inode, used for attribute indexing.
*<code>pad[8]</code> Reserved padding bytes to align the structure.
===Block Runs===
 typedef struct block_run
 {
 int32 allocation_group;
 uint16 start;
 uint16 len;
 } block_run;
BeFS uses block runs to read and allocate disk blocks. 

<code>allocation_group</code> stores allocation group number. 

<code>start</code> Indicates the start of the block run within that allocation group. 

<code>len</code> Length of the block run.

This limits the size of an allocation group to 65,536 bytes, which in turn limits the max block size to 8192 bytes.  

===Inode Structure===
 typedef struct bfs_inode
 {
 int32 magic1;
 inode_addr inode_num;
 int32 uid;
 int32 gid;
 int32 mode;
 int32 flags;
 bigtime_t create_time;
 bigtime_t last_modified_time;
 inode_addr parent;
 inode_addr attributes;
 uint32 type;
 int32 inode_size;
 binode_etc *etc;
 data_stream data;
 int32 pad[4];
 int32 small_data[1];
 } bfs_inode;

*<code>magic1</code> Magic number (0x64358428) used to identify a valid BeFS inode structure.
*<code>inode_num</code> The unique address (block_run/inode_addr) identifying this inode within the filesystem.
*<code>uid</code> User ID of the inode's owner.
*<code>gid</code> Group ID of the inode's owner.
*<code>mode</code> File mode and permissions (similar to Unix stat mode bits).
*<code>flags</code> Inode flags controlling various behaviors or states.
*<code>create_time</code> Timestamp (in microseconds since epoch) of when the inode was created.
*<code>last_modified_time</code> Timestamp (in microseconds since epoch) of the last data modification.
*<code>parent</code> Address (block_run/inode_addr) of the parent directory inode.
*<code>attributes</code> Address (block_run/inode_addr) of the directory containing this inode's extended attributes.
*<code>type</code> Type field, used for special file types (e.g., mail, symbolic links).
*<code>inode_size</code> Total size of the on-disk inode structure (including variable-length sections).
*<code>etc</code> Pointer to additional inode-specific data (e.g., for symbolic links or special nodes).
*<code>data</code> The data_stream structure describing the location and extents of the file's primary data blocks.
*<code>pad[4]</code> Reserved padding bytes for alignment.
*<code>small_data[1]</code> Flexible array starting the variable-length small_data section, used for short file names, symbolic link targets, and other small inline attributes.

<ref>https://www.nobius.org/dbg/practical-file-system-design.pdf</ref>

== OS Support ==
Currently, BeFS is used by BeOS, Magnussoft ZETA (BeOS derivative), as well as Haiku. It has an experimental driver in the Linux kernel since 2.4.x era.<ref>https://tech-pubs.net/reputable-archive/befs-linux.html</ref>

Because of the well-documented nature it was also used by [https://en.wikipedia.org/wiki/SkyOS SkyOS] and served as inspiration for [https://syllable.metaproject.frl/ AtheOS Filesystem]. 

==Performance Characteristics==
BeFS was designed with multithreading and low latency in mind. The filesystem is reentrant and supports fine-grained locking, allowing multiple threads to access different parts of the filesystem concurrently. 

Sequential I/O performance is competitive with other filesystems of its era, but BeFS particularly excels at metadata-heavy operations, such as directory scans and attribute queries. However, write performance can degrade under heavy fragmentation, as BeFS does not aggressively optimize data placement beyond basic allocation heuristics.

==See Also==
[[Category:Filesystems]]
<references />
````

<a id="w001513"></a>

## W001513 — Template:Stub

Bron-ID: W001513
Namespace: Template (10)
Revisie: 29006 | Datum: 2024-06-06T17:30:33Z
Bron: https://wiki.osdev.org/Template:Stub
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AStub&oldid=29006

````text
<center>
{|style="border: 1px solid #cfcfbf; margin-top: 25px; margin-bottom: 25px; background-color: #f0f0ff; text-align: center;"
|
[[image:Stub.png|This article is a stub!]]
This page or section is a [[:Category:Stubs|stub]]. You can help the wiki by ''accurately'' [{{fullurl:{{FULLPAGENAME}}|action=edit}} contributing] to it.
|
|}
</center>

<includeonly>[[Category:Stubs]]</includeonly>
````

<a id="w001514"></a>

## W001514 — FFS (Amiga)

Bron-ID: W001514
Namespace: Main (0)
Revisie: 29569 | Datum: 2025-06-22T13:32:16Z
Bron: https://wiki.osdev.org/FFS_(Amiga)
Revisie-URL: https://wiki.osdev.org/index.php?title=FFS_%28Amiga%29&oldid=29569

````text
{{Filesystems}}

The Amiga Fast File System, to put it bluntly, is not - or rather, it's fast only when compared to the OFS, the Original File System of AmigaOS 1.x.

== About Amiga FFS ==

There are many bright design ideas making the AmigaOS a very special thing, but the file system was not exactly part of it. It is prone to invalidation, holds redundant data, and its directory structure is comparatively slow to traverse. It also lacks any concept of multi-user environments.

Perhaps the only good thing with the Amiga FFS was the concept of the Rigid Disk Block (RDB) - a special area at the beginning of a disk, holding not only the partitioning information. It was also possible to store a file system there - a module that would tell a different AmigaOS machine how to read a partition if it was not formatted in FFS format but something else.

For those interested in its internals should try to find a copy of "The Amiga Guru Book" by Ralph Babel, which holds a complete reference of its rather complex block structure. (It also has a complete reference of the DOS library, as well as interesting information on various internals of the Amiga architecture. It is long out of print, but perhaps you can still find copies on eBay.) The old FAQ also held some info in the internals, which are preserved in the AmigaFFS Document.

== Technical Details ==

=== Root Block ===

The root of the tree is the root block, which is at a fixed place on the disk. The root is like any other directory, except that it has no parent, and it's secondary type is different. AmigaDOS stores the name of the disk volume in the name field of the root block.

Each filing system block contains a checksum, where the sum (ignoring overflow) of all the words in the block is zero.

          +---------------+
        0 |  T. SHORT     | Type
          |---------------|
        1 |       0       | header key (always 0)
          |---------------|
        2 |         0     | Highest seq number (always 0)
          |---------------|
        3 |   HT SIZE     | Hashtable size (=blocksize -56)
          |---------------|
        4 |       0       |
          |---------------|
        5 |   CHECKSUM    |
          |---------------|
        6 |     hash      |
          |     table     |
          /               /
          \               \
  SIZE-51 |               |
          |---------------|
  SIZE-50 |  BMFLAG       | TRUE if bitmap on disk is valid
          |---------------|
  SIZE-49 |   bitmap      | Used to indicate the blocks
  SIZE-24 |    pages      | containing the bitmap
          |---------------|
  SIZE-23 |    DAYS       | Volume last altered date and time
          |---------------|
  SIZE-22 |    MINS       |
          |---------------|
  SIZE-21 |    TICKS      |
          |---------------|
  SIZE-20 |     DISK      | Volume name as a BCPL string
          |     NAME      | of <= 30 characters
          |---------------|
  SIZE-7  |   CREATEDAYS  | Volume creation date and time
          |---------------|
  SIZE-6  |   CREATEMINS  |
          |---------------|
  SIZE-5  |  CREATETICKS  |
          |---------------|
  SIZE-4  |       0       | Next entry on this hash chain
          |---------------| (always 0)
  SIZE-3  |       0       | Parent directory (always 0)
          |---------------|
  SIZE-2  |       0       | Extension (always 0)
          |---------------|
  SIZE-1  |    ST.ROOT    | Secondary type indicates root block
          +---------------+

=== User Directory Blocks ===

          +---------------+
        0 |   T.SHORT     | Type
          |---------------|
        1 |   OWN KEY     | Header Key (pointer to self)
          |---------------|
        2 |       0       | Highest Seq Number (always 0)
          |---------------|
        3 |       0       |
          |---------------|
        4 |       0       |
          |---------------|
        5 |  CHECKSUM     |
          |---------------|
        6 |               |
          |    hash table |
          /               /
          \               \
  SIZE-51 |               |
          |---------------|
  SIZE-50 |    Spare      |
          |---------------|
  SIZE-48 |    PROTECT    |  Protection bits
          |---------------|
  SIZE-47 |       0       | Unused (always 0)
          |---------------|
  SIZE-46 |               |
          |   COMMENT     | Stored as  BCPL string
  SIZE-24 |               |
          |---------------|
  SIZE-23 |     DAYS      | Creation date and time
          |---------------|
  SIZE-22 |     MINS      |
          |---------------|
  SIZE-21 |    TICKS      |
          |---------------|
  SIZE-20 | DIRECTORY NAME| Stored as a BCPL string <=30 chars
          |---------------|
  SIZE-4  | HASHCHAIN     | Next entry with same hash value
          |---------------|
  SIZE-3  |    PARENT     | back pointer to parent directory
          |---------------|
  SIZE-2  |      0        | Extension (always 0)
          |---------------|
  SIZE-1  |  ST.USERDIR   | secondary type
          +---------------+

User directory blocks have type T.SHORT and secondary type ST.USERDIRECTORY. The six information words at the start of the block also indicate the block's own key (this is, the block number) as a consistency check and the size of the hash table. The 50 information words at the end of the block contain the date and time of creation, the name of the directory, a pointer to the next file or directory on the hash chain, and a pointer to the directory above.

To find a file or sub-directory, you must first apply a hash function to its name. This hash function yields an offset in the hash table, which is the key of the first block on a chain linking those with the same hash value (or 0, if there are none). AmigaDOS reads the block with this key and compares the name of the block with the required name. If the names do not match, it reads the next block on the chain, and so on.
1.1.3 File Header Block

           +------------+
        0  |   T.SHORT  | Type
           |------------|
        1  |   OWN KEY  | Header Key
           |------------|
        2  | HIGHEST SEQ| Total number of data blocks in file
           |------------|
        3  |  DATA SIZE | Number of data block slots used
           |------------|
        4  | FIRST DATA | First data block
           |------------|
        5  |  CHECKSUM  |
           |------------|
        6  |            |
           /            /
           \            \
           | DATA BLK 3 |
           | DATA BLK 2 | List of data block keys
  SIZE-51  | DATA BLK 1 |
           |------------|
  SIZE-50  |  Spare     |
           |------------|
  SIZE-49  |   PROTECT  | Protection bits
           |------------|
  SIZE-48  |  BYTESIZE  | Total size of file in bytes
           |------------|
  SIZE-46  |            |
           |  COMMENT   | Comment as a BCPL string
  SIZE-24  |            |
           |------------|
  SIZE-23  |    DAYS    | Creation date and time
           |------------|
  SIZE-22  |    MINS    |
           |------------|
  SIZE-21  |    TICKS   |
           |------------|
  SIZE-20  | FILE NAME  | Stored as BCPL string <= 30 chars
           |------------|
  SIZE-4   |  HASHCHAIN | Next entry with same hash value
           |------------|
  SIZE-3   |   PARENT   | Back pointer to the parent directory
           |------------|
  SIZE-2   |  EXTENSION | Zero pointer to the first extension
           |------------| block
  SIZE-1   |  ST. FILE  | Secondary type
           +------------+

Each terminal file starts with a file header block, which has type T.SHORT and secondary type ST.FILE. The start and end of the block contain name, time, and redundancy information similar to that in a directory block. The body of the file consists of Data blocks with sequence numbers from 1 upwards. AmigaDOS stores the addresses of these blocks in consecutive words downwards from offset size-51 in the block. In general, AmigaDOS does not use all the space for this list and the last data block is not full.

=== File List Block ===

If there are more blocks in the file than can be specified in the block list, then the EXTENSION field is non-zero and points to another disk block which contains a further data block list. The following figure explains the structure of the file list block.

           +-------------+
        0  |   T. LIST   | Type
           |-------------|
        1  |   OWN KEY   | Header Key
           |-------------|
        2  | BLOCK COUNT | =number of data blocks in block list
           |-------------|
        3  | DATA SIZE   | Same as above
           |-------------|
        4  | FIRST DATA  | First Data Block
           |-------------|
        5  |  CHECKSUM   |
           |-------------|
        6  |             |
           /             /
           \             \
           | BLOCK N+3   |
           | BLOCK N+2   | Extended list of data block keys
  SIZE-51  | BLOCK N+1   |
           |-------------|
  SIZE-50  |      info   | (unused)
           |-------------|
  SIZE-4   |     0       | Next in hash list (always 0)
           |-------------|
  SIZE-3   |   PARENT    | File header block of this file
           |-------------|
  SIZE-2   | EXTENTSION  | Next extension block
           |-------------|
  SIZE-1   |   ST. FILE  | Secondary type
           +-------------+

There are as many file extension blocks as required to list the data blocks that make up the file. The layout of the block is very similar to that of a file header block, except that the type is different and the date and filename fields are not used.

=== Data Block (OFS) ===

           +-------------+
        0  |   T. DATA   | type
           |-------------|
        1  |   HEADER    | header key
           |-------------|
        2  |   SEQ NUM   | Sequence number
           |-------------|
        3  |  DATA SIZE  |
           |-------------|
        4  |  NEXT DATA  | next data block
           |-------------|
        5  |  CHECKSUM   |
           |-------------|
        6  |             |
           |             |
           |             |
           |             |
           |    DATA     |
           |             |
           |             |
           |             |
           +-------------+

Data blocks contain only six words of filing system information. These six words refer to the following:

* type (T.DATA)
* pointer to the file header block
* sequence number of the data block
* number of words of data
* pointer to the next data block
* checksum

Normally, all data blocks except the last are full (that is, they have a blocksize = blocksize-6). The last data block has a forward pointer of 0.

=== Data Block (FFS) ===

FFS data blocks are different from OFS data blocks in that there are no metadata in the Former. The whole block is data. The lack of a checksum makes the file system more vulnerable to detect data corruption and there is no way to follow a linked list to get to the next data block in the sequence. The file header block will be the starting point to follow the list of blocks and extension blocks till 'bytesize' amount of bytes has been read, which should be in the last number of blocks indicated (HIGHEST SEQ). These lists of blocks are read in backwards order.

[[Category:Filesystems]]
[[Category:Amiga]]
````

<a id="w001515"></a>

## W001515 — UFS

Bron-ID: W001515
Namespace: Main (0)
Revisie: 30372 | Datum: 2026-03-22T23:21:00Z
Bron: https://wiki.osdev.org/UFS
Revisie-URL: https://wiki.osdev.org/index.php?title=UFS&oldid=30372

````text
{{Filesystems}}
{{Stub}}
{{Distinguish|text=[[FFS (Amiga)|Amiga FFS]]}}

UFS (UNIX File System) also known as FFS (Fast File System) is not a single filesystem but a family of filesystems used by BSD derivatives, such as FreeBSD, NetBSD, OpenBSD and others. Introduced in 4.2BSD, it is a direct descendant of the original UNIX V7 filesystem.

== About FFS / UFS (BSD) ==
What is usually called a "partition" is called a "slice" in *BSD, which is in turn subdivided into "partitions" - a naming pattern that leads to some confusion, and to rather cryptic device names (ad0s1c for the third partition on the second slice on the primary master ATAPI hard drive...).

== See Also ==
=== External Links ===
* [https://man.freebsd.org/cgi/man.cgi?query=ffs&sektion=4 ffs(4)] - FreeBSD manpage on FFS.
* [https://man.netbsd.org/fs.5 fs(5)] - NetBSD manpage describing the superblock structure of its FFS implementation.
* [https://man.openbsd.org/fs.5 fs(5)] - OpenBSD FFS superblock description.
* [https://flylib.com/books/en/2.48.1/ File System Forensic Analysis] - The "UFS1 and UFS2 Data Structures" Chapter

[[Category:Filesystems]]
````

<a id="w001516"></a>

## W001516 — XFS

Bron-ID: W001516
Namespace: Main (0)
Revisie: 29834 | Datum: 2026-01-07T05:54:31Z
Bron: https://wiki.osdev.org/XFS
Revisie-URL: https://wiki.osdev.org/index.php?title=XFS&oldid=29834

````text
{{Filesystems}}
'''XFS''' is a high-performance journaling file system created to replace EFS on IRIX. It has since been ported to [[Linux]] and received additional development there.

== Basic Specifications==
XFS is a 64-bit file system. It supports a maximum file system size of 8 EiB, though many operating systems have implementations that limit this. IRIX is known to support up to 8 TiB volumes.<ref>https://tech-pubs.net/wiki/index.php/XFS</ref>
{| class="wikitable"
|+
|Directory Structure
|B+ Trees
|-
| File Contents 
|B+ Trees 
|-
| Max Volume Size
|8 Terabytes (IRIX), 8 Exabytes (GNU/Linux)
|-
|Max File Size
|4 Terabytes
|-
|Max Filename Length
|255 Bytes
|}

== On-disk Structure ==

=== On disk Header Info ===
 struct xfs_ondisk_hdr {
 __be32 magic; /* magic number */
 __be32 crc; /* CRC, not logged */
 uuid_t uuid; /* filesystem identifier */
 __be64 owner; /* parent object */
 __be64 blkno; /* location on disk */
 __be64 lsn; /* last modification in log, not logged */
 };

* <code>magic</code>:  This is the unique ID telling the OS that the filesystem is an XFS filesystem. The magic humber for XFS is <code>58 46 53 42</code> which in ASCII is "XFSB" standing for XFS Block.
* <code>crc</code>:  This is a checksum for the header to ensure data integrity. It’s not logged, meaning it doesn’t get saved in the XFS transaction log (journal).
* <code>uuid</code>:  This is the filesystem identifier. It’s a unique 128-bit identifier (UUID) for the XFS filesystem, used to distinguish between different XFS filesystems.
* <code>owner</code>:  This represents the parent object of the filesystem (could be a device or logical unit), and it’s typically used in a system where objects can be nested. It helps identify who owns the filesystem.
* <code>blkno</code>:  This is the block number or the location of the XFS filesystem's header on the disk. It specifies where this specific structure is physically located.

* <code>lsn</code>:  The log sequence number (LSN) represents the last modification timestamp in the log. It indicates when the filesystem was last modified but is not logged itself, meaning it’s not part of the transaction log.

=== Allocation Group B+ Tree Blocks ===
 struct xfs_btree_sblock {
 __be32 bb_magic;
 __be16 bb_level;
 __be16 bb_numrecs;
 __be32 bb_leftsib;
 __be32 bb_rightsib;
 /* version 5 extras start here */
 __be64 bb_blkno;
 __be64 bb_lsn;
 uuid_t bb_uuid;
 __be32 bb_owner;
 __le32 bb_crc;
 };
Short blocks use 32-bit pointers for the filesystem blocks.

* <code>bb_magic</code>: Specifies the magic number for the per-allocation group B+tree block.
* <code>bb_level</code>:The level of the tree in which this block is found. If this value is 0, this is a leaf block and contains records; otherwise, it is a node block and contains keys and pointers. Level values increase towards the root. 
* <code>bb_numrecs</code> Number of records in this block.
* <code>bb_leftsib</code> allocation group block number of the left sibling of this B+tree node.
* <code>bb_rightsib</code> AG block number of the right sibling of this B+tree node. 
* <code>bb_blkno</code> FS block number of this B+tree block.  
* <code>bb_lsn</code> Log sequence number of the last write to this block. 
* <code>bb_uuid</code> The UUID of this block, which must match either sb_uuid or sb_meta_uuid depending on which features are set. 
* <code>bb_owner</code> The AG number that this B+tree block ought to be in. 
* <code>bb_crc</code> Checksum of the B+tree block.

<ref>''XFS Algorithms & Data Structures''. 2006. 3rd ed., Silicon Graphics Corparation, Apr. 2025, pp. 7–8, ftp.ntu.edu.tw/linux/utils/fs/xfs/docs/xfs_filesystem_structure.pdf. Accessed 31 Dec. 2025.</ref>
 struct xfs_btree_lblock {
 __be32 bb_magic;
 __be16 bb_level;
 __be16 bb_numrecs;
 __be64 bb_leftsib;
 __be64 bb_rightsib;
 /* version 5 additions start here */
 __be64 bb_blkno;
 __be64 bb_lsn;
 uuid_t bb_uuid;
 __be64 bb_owner;
 __le32 bb_crc;
 __be32 bb_pad;
 };
Long format B+trees are given 64-bit filesystem block numbers instead of 32-bit allocation group block numbers.

* <code>bb_magic</code>  Specifies the magic number for the B+tree block.
* <code>bb_level</code>  The level of the tree in which this block is found. If this value is 0, this is a leaf block and contains records; otherwise, it is a node block and contains keys and pointers.
* <code>bb_numrecs</code>  Number of records in this block.
* <code>bb_leftsib</code>  Filesystem block number of the left sibling of this B+tree node.
* <code>bb_rightsib</code>  Filesystem block number of the right sibling of this B+tree node.
* <code>bb_blkno</code>  Filesystem block number of this B+tree block.
* <code>bb_lsn</code>  Log sequence number of the last write to this block.
* <code>bb_uuid</code>  The UUID of this block, which must match either <code>sb_uuid</code> or <code>sb_meta_uuid</code> depending on which features are set.
* <code>bb_owner</code>  The allocation group number that this B+tree block is located in.
* <code>bb_crc</code>  Checksum of the B+tree block.
* <code>bb_pad</code>  Pads the structure to 64 bytes.

=== Block Headers ===
 struct xfs_da3_blkinfo {
 /* these values are shared with xfs_da_blkinfo */
 __be32 forw;
 __be32 back;
 __be16 magic;
 __be16 pad;
 /* These values are unique to da3 blkinfo */
 __be32 crc;
 __be64 blkno;
 __be64 lsn;
 uuid_t uuid;
 __be64 owner;
 };

* <code>forw</code> Logical block offset of the previous B+tree block at this level.
* <code>back</code> Logical block offset of the next B+tree block at this level.
* <code>magic</code> Magic number for this directory/attribute block.
* <code>pad</code> Padding to maintain alignment.
* <code>crc</code> Checksum of the directory/attribute block.
* <code>blkno</code> Block number of this directory/attribute block.
* <code>lsn</code> Log sequence number of the last write to this block.
* <code>uuid</code> The UUID of this block, which must match either sb_uuid or sb_meta_uuid depending on which features are set.
* <code>owner</code> The inode number that this directory/attribute block belongs to.

== xfsprogs ==
xfsprogs is the open source set of xfs utilities used on GNU/Linux to perform filesystem tasks. It includes:

* mkfs.xfs - used to make new xfs filesystems
* fsck.xfs - used to ensure filesystem consistency
* xfs_freeze - suspends access to filesystem to allow dumping or growing the filesystem. 
* xfs_info - prints geometry info about xfs. 
* xfs_repair - repairs corrupted xfs volumes
* xfs_scrub - used to rebuild filesystem metadata. 

Other programs are included as well. These are the notable ones.<ref>https://archive.kernel.org/oldwiki/xfs.wiki.kernel.org/</ref>

== OS Support ==
Linux kernel has supported XFS since around 2001, in the 2.4.x branch. 

IRIX began using XFS starting with major version 6.2. Special 5.3 version was offered with XFS as well. 

== Shrinking XFS Volumes ==
As of 2025 neither IRIX nor Linux support shrinking XFS volumes for several reasons:<ref>https://web.archive.org/web/20230910042608/https://marc.info/?l=linux-xfs&m=118091640624488&</ref>
* Filesystem has no way to mark allocation groups as unavailable (to prevent use during shrink operation)
* Inodes need to be removed from unavailable allocation groups. 
* Somehow move data out of unavailable groups easily. 
* xfsprogs lacks ability to move journal structures safely

== Further Reading ==
*[https://ftp.ntu.edu.tw/linux/utils/fs/xfs/docs/xfs_filesystem_structure.pdf XFS Algorithms and Data Structures] - Contains useful specifications for the XFS file system, but is from 2006 and is likely outdated.

== References ==
<references />

[[Category:Filesystems]]
````

<a id="w001517"></a>

## W001517 — BFS

Bron-ID: W001517
Namespace: Main (0)
Revisie: 30004 | Datum: 2026-02-14T16:40:08Z
Bron: https://wiki.osdev.org/BFS
Revisie-URL: https://wiki.osdev.org/index.php?title=BFS&oldid=30004

````text
{{Filesystems}}
{{Stub}}

'''BFS''' (''UnixWare Boot File System'') is a SCO specification for a KISS [[File Systems|filesystem]] meant to be used in a [[Bootloader|bootloader]] stage. It only offers one directory (the root) and due to the way information about blocks are stored, only one file may be opened for writing at a time.

== See Also ==
=== External Links ===
* [https://martin.hinner.info/fs/bfs/bfs-structure.html The BFS filesystem structure]

[[Category:Filesystems]]
````

<a id="w001518"></a>

## W001518 — Category:Filesystems

Bron-ID: W001518
Namespace: Category (14)
Revisie: 2343 | Datum: 2006-12-16T20:59:43Z
Bron: https://wiki.osdev.org/Category:Filesystems
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AFilesystems&oldid=2343

````text
This is a list of documented filesystems. For underlying theory visit [[File Systems]]

[[Category:OS theory]]
````

<a id="w001519"></a>

## W001519 — NFS

Bron-ID: W001519
Namespace: Main (0)
Revisie: 29013 | Datum: 2024-06-06T17:31:45Z
Bron: https://wiki.osdev.org/NFS
Revisie-URL: https://wiki.osdev.org/index.php?title=NFS&oldid=29013

````text
{{Filesystems}}
{{Stub}}
'''NFS''' was invented by Sun Microsystems. It became widespread largely because it's quite easy to implement. In return for its simplicity, it tends to give relatively poor performance and a nearly complete lack of safety. These are both largely due to its connectionless nature. When you request data from a file, the server sends you the requested information, but does NOT keep track of which clients have which files open. To keep you from seeing (terribly) out-of-date information from a file, the data you read has an "expiration date". If you refer to the data from more than, say, a minute, it will expire and your client will request the data from the server again, whether it's changed or not. If you write data to the file, you have no way of knowing whether somebody else has updated the information between your reading and writing your data, so you may overwrite things they've written with older data. To ensure at least a little bit of safety, the server is supposed to actually commit data you write to disk before it returns to you.

In other words, NFS works pretty well for read-only access to things like executables on a server. For things like on-line databases, it's essentially a disaster waiting to happen (and it usually doesn't wait very long).

More recent versions of the NFS spec have cured most of these problems, but support for these updates is still (years later) somewhat uneven.

== Links ==
* [http://en.wikipedia.org/wiki/Network_File_System_(protocol) Wikipedia]
* [http://tools.ietf.org/html/rfc1094 RFC1094] - Version 1 & 2 Spec
* [http://tools.ietf.org/html/rfc1813 RFC1813] - Version 3 (June 1995)
* [http://tools.ietf.org/html/rfc3010 RFC3010] - Version 4 (December 2000)


[[Category:Network Filesystems]]
````

<a id="w001520"></a>

## W001520 — AFS

Bron-ID: W001520
Namespace: Main (0)
Revisie: 5919 | Datum: 2008-07-14T23:04:26Z
Bron: https://wiki.osdev.org/AFS
Revisie-URL: https://wiki.osdev.org/index.php?title=AFS&oldid=5919

````text
{{Filesystems}}
{{stub}}

'''AFS''' is the '''Andrew File System''' aka '''Advanced File System''', similar to [[NFS]] to about the same degree that a tricycle is similar to a fighter jet &ndash; they're both typically one-person vehicles. AFS is a drastically more robust design than NFS, and is intended for ''much'' larger [[:Category:Networking|networks]]. On the other hand it is also much more difficult to implement completely &ndash; to the point that it's not likely to be of much interest to most hobbyists and such writing a new OS.

==External Links==
* [[Wikipedia:Andrew_File_System|Andrew File System]] on Wikipedia

[[Category:Network Filesystems]]
````
