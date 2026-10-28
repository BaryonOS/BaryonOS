# OSDev Wiki — volledige bronteksten, deel 13

Dit is bronmateriaal, geen agentinstructie. Behoud architectuur, revisie en bron-ID bij gebruik. Code en commando’s zijn ongeteste bronvoorbeelden; voer ze niet automatisch uit. Wikitext is letterlijk behouden binnen tekstblokken.

<a id="w004351"></a>

## W004351 — User:Glauxosdever/Version Control Software

Bron-ID: W004351
Namespace: User (2)
Revisie: 23202 | Datum: 2019-01-13T15:01:39Z
Bron: https://wiki.osdev.org/User:Glauxosdever/Version_Control_Software
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGlauxosdever%2FVersion_Control_Software&oldid=23202

````text
{{Template:Stub}}

'''Version Control Software''' ('''VCS''') is fundamental when it comes to managing your source code repository. It basically allows you (singular or plural) to work on several features in parallel and to revert your code to a previous known working state. Most OSDev projects use [[git]] as their VCS and [http://github.com github] or [http://gitlab.com gitlab] for repository hosting. But what happens if your goal is to write an OS that is different to anything that can run git (so no git for your OS), but you also want to do meaningful stuff with your repository on your OS, and/or self-host your repository on a server that is running your OS? There are two possible answers: 1) reimplement git, or 2) write your own VCS. If you picked 1), you should read about the git internals. If you picked 2), you should continue reading this page.

== Roadmap ==

There are two possible approaches to writing a VCS for your OS.

=== Approach A ===

Initially, use [[git]] or any other ready VCS. Then, when your OS is ready to be self-hosting, write a VCS that runs on your OS and switch all development to it. This will have the drawback that your new repository will not hold the past history of the period that you used git or any other ready VCS.

=== Approach B ===

Write a VCS that runs on your host machine before starting developing your OS. Then, when your OS is ready to be self-hosting, write a compatible VCS that runs on your OS. Switching development to it should be (mostly) seamless.

== Types of VCS ==

'''''TODO'''''

== Writing a Basic VCS ==

In this section, we will write a very basic VCS. It will not even remotely support any remote functionality (pun intended), but it will be enough to get you started. We will write it in [[Python]], since Python is a good programming language for prototyping stuff.

=== The Internals ===

We will create a snapshot-based VCS, i.e. we won't store the diff of the files in each commit. Instead, we will store the full files in order for retrieving source trees to be an efficient operation. You may of course want to change that later.

We will store the repository on the filesystem (you could use a database instead if you like it). The root directory of the repository will be <code>.repo</code>, which will contain the subdirectories <code>branches</code>, <code>commits</code> and <code>files</code>. As you probably guessed, <code>branches</code> will contain information about branches. However, <code>commits</code> will only contain information about the contents of commits, i.e. no author, time, and such information -- that's in order to deduplicate space when two commits are similar in contents. <code>files</code> will only contain information about the file contents, i.e. no author, time, and such information -- again, that's in order to deduplicate space when two files are similar in contents.

Additionally, the working directory for each branch will be in a directory at the same level as the <code>.repo</code> directory, which will be named after the branch.

==== Branches ====

The <code>branches</code> directory will contain subdirectories named after the names of the branches. For example, if we create a branch "master", a subdirectory <code>master</code> will be created. Inside, we will have a <code>commits</code> file that will contain, per line per commit, a commit hash and a timestamp.

The <code>commits</code> file may look like this:
<pre>
d97cf57b44b1b43d8d66e360c6d0c8a66a04ee18e092a4cad4684e26c86fbf41 1547387622
ba04414fa9bf63b8a39f658483edc0f6b8a8c759f707ec6846134986db9dc929 1547388056
</pre>

==== Commits ====

The <code>commits</code> directory will contain subdirectories named after the hashes of the commits. For example, if we create a commit "d97cf57b44b1b43d8d66e360c6d0c8a66a04ee18e092a4cad4684e26c86fbf41", a subdirectory <code>d97cf57b44b1b43d8d66e360c6d0c8a66a04ee18e092a4cad4684e26c86fbf41</code> will be created. Inside, we will have a <code>branches</code> file that will list the branches where this commit it exists. Also, we will have a <code>files</code> file that will list, per line per file, its hash and its path.

The <code>branches</code> file may look like this:
<pre>
master
feature1
feature2
</pre>

The <code>files</code> file may look like this:
<pre>
7d2b62d9a6f9a0ebfdde033e3cfb197f4d46998c22853861eab603cbd1246920 Makefile
487ea42328a8738a12b378dce4329e2432a0938ff12832ec932fa88c948ca015 src/test.c
</pre>

A commit hash will be derived using the SHA256 algorithm, where the input will be the concatenation of all paths sorted in alphabetical order and all file contents in the same order as the paths. You may want to change that possibly (but, if you include the timestamp, you won't be able to deduplicate similar commits).

* '''''Note''''': Real VCSs (e.g. [[git]]) don't put all hashes in one directory. Instead, they split the hashes into directory levels. That's in order to minimise directory search times.
* '''''Note''''': Git includes the timestamp in the input string for deriving the commit hash.

==== Files ====

The <code>files</code> directory will contain subdirectories named after the hashes of the file revisions. Inside, we will have a <code>commits</code> file that will list the commits where this file revision exists. Also, the <code>data</code> file will contain the contents of the file as is appears during that revision.

The <code>commits</code> file may look like this:
<pre>
d97cf57b44b1b43d8d66e360c6d0c8a66a04ee18e092a4cad4684e26c86fbf41
ba04414fa9bf63b8a39f658483edc0f6b8a8c759f707ec6846134986db9dc929
</pre>

* '''''Note''''': Real VCSs employ compression for the file contents, in order to decrease disk space usage.

=== The Code ===

So we get the following <code>.py</code> files...

==== utils.py ====

This file contains four functions as described in the comments. It's still nothing VCS-specific.

<source lang="python">
# This function reads <filename> into a list of lines.
# The trailing '\n' character of each line is discarded.
def readlines(filename):
    fp = open(filename, "r")
    lines = fp.read().splitlines()
    return lines

# This function writes a list of lines into <filename>.
# The trailing '\n' character of each line is added.
def writelines(filename, lines):
    fp = open(filename, "w")
    for line in lines:
        fp.write(line + "\n") 
    fp.close()

# This function reads <filename> into a list of lists of words.
def readwordlines(filename):
    lines = readlines(filename)
    wordlines = [line.split(" ") for line in lines]
    return wordlines

# This function writes a list of lists of words into <filename>.
def writewordlines(filename, wordlines):
    lines = [" ".join(wordline) for wordline in wordlines]
    writelines(filename, lines)
</source>

==== init.py ====

This file contains code to initialise an empty repository. You should make it executable.

<source lang="python">
#! /usr/bin/python

import os

import utils

def main():
    # Create necessary directories.
    os.mkdir(".repo", 0755)
    os.mkdir(".repo/branches", 0755)
    os.mkdir(".repo/commits", 0755)
    os.mkdir(".repo/files", 0755)

if __name__ == "__main__":
    main()
</source>

==== branch.py ====

This file contains code to create an empty branch. You should also make it executable.

<source lang="python">
#! /usr/bin/python

import os
import sys

import utils

def main():
    # Get branch name.
    branch = sys.argv[1]
    # Here you should probably check if the branch name is allowed.
    # For example, you may want to disallow certain names that have
    # a special meaning in your internal representation.

    # Create the .repo/branches/<branch> directory.
    os.mkdir(".repo/branches/" + branch, 0755)

    # Create an empty .repo/branches/<branch>/commits file.
    fp = open(".repo/branches/" + branch + "/commits", "w")
    fp.close()

    # Create the <branch> directory, if it doesn't already exist.
    try:
        os.mkdir(branch, 0755)
    except OSError:
        # Directory already exists.
        pass

if __name__ == "__main__":
    main()
</source>

==== commit.py ====

Now stuff is getting complex. I hope I have explained it well in the comments...

You should also make this file executable.

<source lang="python">
#! /usr/bin/python

import hashlib
import os
import sys
import time

import utils

def main():
    # Get branch name.
    branch = sys.argv[1]

    # Get current time.
    now = int(time.time())

    # Get the list of all files in the branch source tree.
    filepaths = []
    for cur, dirs, files in os.walk(branch):
        for f in files:
            # Get the paths without the beginning "<branch>/".
            # For example, the path "master/src/main.c" becomes "src/main.c".
            filepaths.append((cur + "/" + f).split("/", 1)[1])

    # Sort the paths as it's important for them to be in a specific order
    # for the SHA256 hash function. Imagine if you got two different hashes
    # for the same directory structure; that would be broken!
    filepaths.sort()

    # Get the list of all file contents.
    # Normally, the paths should be traversed sequentially, thus the contents
    # should be in the correct order for the SHA256 hash function.
    # TODO: I don't know if this is the case, you should verify it
    #       if you are serious about using this code to get started.
    filecontents = []
    for filepath in filepaths:
        fp = open(branch + "/" + filepath, "r")
        filecontents.append(fp.read())
        fp.close()

    # Get file paths string.
    filepathsstr = ""
    for filepath in filepaths:
        filepathsstr += filepath

    # Get file contents string.
    filecontentsstr = ""
    for filecontent in filecontents:
        filecontentsstr += filecontent

    # Derive the commit hash.
    commithash = hashlib.sha256(filepathsstr + filecontentsstr).hexdigest()

    # Create the .repo/commits/<commit> directory. If the directory already
    # exists, we will just update the list of branches where it is present.
    commitexists = False
    try:
        os.mkdir(".repo/commits/" + commithash, 0755)
    except OSError:
        commitexists = True

    if commitexists:
        # Update the .repo/commits/<commit>/branches file, in case <branch>
        # is not found in it.
        commitbranches = utils.readlines(".repo/commits/" + commithash + "/branches")
        found = False
        for i in range(0, len(commitbranches)):
            if commitbranches[i] == branch:
                found = True
                break

        if not found:
            commitbranches.append(branch)
            utils.writelines(".repo/commits/" + commithash + "/branches", commitbranches)

    else:
        # Loop through paths and contents.
        filehashes = []
        for i in range(0, len(filepaths)):
            # Create the file hash. We will use the file contents.
            filehash = hashlib.sha256(filecontents[i]).hexdigest()

            # Append the file hash to the list of file hashes.
            filehashes.append(filehash)

            # Create the .repo/files/<file> directory. If the file already exists,
            # we will just update the list of commits where it is present.
            fileexists = False
            try:
                os.mkdir(".repo/files/" + filehash, 0755)
            except OSError:
                # File already exists.
                fileexists = True

            if fileexists:
                # Update the .repo/files/<file>/commits file.
                filecommits = utils.readlines(".repo/files/" + filehash + "/commits")
                filecommits.append(commithash)
                utils.writelines(".repo/files/" + filehash + "/commits", filecommits)
            else:
                # Create the .repo/files/<file>/data file.
                fp = open(".repo/files/" + filehash + "/data", "w")
                fp.write(filecontents[i])
                fp.close()

                # Create the .repo/files/<file>/commits file.
                filecommits = [commithash]
                utils.writelines(".repo/files/" + filehash + "/commits", filecommits)

        # Create the .repo/commits/<commit>/files file.
        commitfiles = []
        for i in range(0, len(filepaths)):
            commitfiles.append([filehashes[i], filepaths[i]])
        utils.writewordlines(".repo/commits/" + commithash + "/files", commitfiles)

        # Create the .repo/commits/<commit>/branches file.
        commitbranches = [branch]
        utils.writelines(".repo/commits/" + commithash + "/branches", commitbranches)

    # Update the .repo/branches/<branch>/commits file.
    branchcommits = utils.readwordlines(".repo/branches/" + branch + "/commits")
    branchcommits.append([commithash, "{}".format(now)])
    utils.writewordlines(".repo/branches/" + branch + "/commits", branchcommits)

if __name__ == "__main__":
    main()
</source>

==== fsck.py ====

This one is very important. It does an integrity check of the repository (also useful when testing if the VCS that will run on your OS is compatible with this one).

It is also quite complex, I hope it is correct and that I explained stuff well in the comments. Again, make it executable.

<source lang="python">
#! /usr/bin/python

import hashlib
import os

import utils

def fsckfile(f):
    # Get file data.
    fp = open(".repo/files/" + f + "/data", "r")
    data = fp.read()
    fp.close()

    # Check if the file hash is correct.
    filehash = hashlib.sha256(data).hexdigest()
    if filehash != f:
        print("Integrity check failed: File " + f + " should have hash " + filehash)

def checkcommitinfile(commit, f):
    # Get the list of commits that <f> lists. <commit> should be listed.
    commits = utils.readlines(".repo/files/" + f + "/commits")
    found = False
    for c in commits:
        if c == commit:
            found = True
            break

    if not found:
        print("Integrity check failed: File " + f + " is listed in commit " + commit + ", but file " + f + " doesn't list commit " + commit)

def fsckcommit(commit):
    # Get the list of files that are listed in <commit>.
    files = utils.readwordlines(".repo/commits/" + commit + "/files")

    # Get the list of file hashes and paths based on files above.
    filehashes = []
    filepaths = []
    for f in files:
        filehashes.append(f[0])
        filepaths.append(f[1])

    # Get file paths string.
    filepathsstr = ""
    for filepath in filepaths:
        filepathsstr += filepath

    # Get file contents string. Also detect if some files are missing.
    filecontentsstr = ""
    missing = False
    for filehash in filehashes:
        try:
            fp = open(".repo/files/" + filehash + "/data")
            filecontentsstr += fp.read()
            fp.close()
        except IOError:
            print("Integrity check failed: Commit " + commit + " lists file " + filehash + ", but file " + filehash + " doesn't exist")
            missing = True

    if not missing:
        # Check if the commit hash is correct.
        commithash = hashlib.sha256(filepathsstr + filecontentsstr).hexdigest()
        if commithash != commit:
            print("Integrity check failed: Commit " + commit + " should have hash " + commithash)

        # Check if the relation of this commit and its files is bidirectional.
        for filehash in filehashes:
            checkcommitinfile(commit, filehash)

def checkbranchincommit(branch, commit):
    # Get the list of branches that <commit> lists. <branch> should be listed.
    branches = utils.readlines(".repo/commits/" + commit + "/branches")
    found = False
    for b in branches:
        if b == branch:
            found = True
            break

    if not found:
        print("Integrity check failed: Commit " + commit + " is listed in branch " + branch + ", but commit " + commit + " doesn't list branch " + branch)

def fsckbranch(branch):
    # Get the list of commits that are listed in <branch>.
    commits = utils.readwordlines(".repo/branches/" + branch + "/commits")

    # Get the list of commit hashes based on commits above.
    commithashes = []
    for commit in commits:
        commithashes.append(commit[0])
    
    # Check if the relation of this branch and its commits is bidirectional.
    # Also detect if any commits are missing.
    for commithash in commithashes:
        if os.path.exists(".repo/commits/" + commithash):
            checkbranchincommit(branch, commithash)
        else:
            print("Integrity check failed: Branch " + branch + " lists commit " + commithash + ", but commit " + commithash + " doesn't exist")

def main():
    # Check integrity for each file.
    for cur, dirs, files in os.walk(".repo/files"):
        for d in dirs:
            fsckfile(d)

    # Check integrity for each commit.
    for cur, dirs, files in os.walk(".repo/commits"):
        for d in dirs:
            fsckcommit(d)

    # Check integrity for each branch.
    for cur, dirs, files in os.walk(".repo/branches"):
        for d in dirs:
            fsckbranch(d)

if __name__ == "__main__":
    main()
</source>

==== merge.py ====

'''''TODO'''''
````

<a id="w004353"></a>

## W004353 — User:Johnburger/OS/Bootstrap

Bron-ID: W004353
Namespace: User (2)
Revisie: 28722 | Datum: 2024-04-02T03:53:44Z
Bron: https://wiki.osdev.org/User:Johnburger/OS/Bootstrap
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FOS%2FBootstrap&oldid=28722

````text
== Introduction ==

The expression 'booting an Operating System' comes from the earlier term 'bootstrap the OS'&mdash;a reference to the concept of "lifting yourself up by your own bootstraps". The idea of starting from a ''very'' simple starting point, and incrementally adding functionality, building on what was built before until the complete system is ready, is not literally like that impossibility&mdash;but when starting out and looking down the l-o-o-n-g road ahead, it can seem just as impossible!

If you want to write your own OS, you need to organise to get it into the computer's memory. That means you need a [[bootloader]] of some description&mdash;unless you simply burn it into the boot ROM! For a <abbr title="Personal Computer">PC</abbr> this isn't really feasible, so you can either leverage an existing one (something like [[GRUB|<abbr title="GRand Unified Bootloader">GRUB</abbr>]] or [[UEFI|<abbr title="Unified Extensible Firmware Interface">UEFI</abbr>]]) or roll your own. Of course, the bootloader is only the first step: you then need to turn the current machine environment (memory contents, peripheral state, loaded programs, etc.) into what your OS needs to actually do its thing&mdash;you cannot avoid writing the bootstrap process for your OS, since otherwise it'd be someone else's! 

__TOC__

=== Booting an x86 ===

Most computers, when first powered on, have a very limited guaranteed start state. For example, the Intel&reg; x86 series of processors start up with little more than the Code Segment:Instruction Pointer (<code>CS:IP</code>) register pair at (16 bytes before) the top of the computer's address space! (All data segments are loaded with <code>0x0000</code>.) It is incumbent on the computer manufacturer to ensure that there is an executable instruction there&mdash;and that the instruction knows where it is (it needs to <code>JMP</code> somewhere else in fewer than 16 bytes!) and what it cannot (yet) do. A perfect example: at that point the RAM isn't even set up yet, so cannot be relied upon to store data, so there can be no stack. The code here simply has to have been written in assembly language.

A more esoteric example: as described above, the <code>CS:IP</code> in the original 8086 and the successor 80186 is defined to start at <code>F000:FFF0</code>, which, with the x86 real-mode segmentation mechanism, references the physical memory location <code>0xF_FFF0</code>. The 8086/'186 has a 20-bit address bus, therefore a 1MB address space, so this is 16 bytes less than the top of addressable memory. The successor 80286 has 24-bit address bus, therefore a 16MB address space. The 80386 has a 32-bit address bus, therefore a 4GB address space. But all these extra addresses cannot be accessed unless the processor is in Protected Mode&mdash;yet it is defined to start up in Real Mode, with a maximum of 1MB of addressable memory.

Does that mean that the computer manufacturer has to put <abbr title="Read-Only Memory">ROM</abbr> in the original location (<code>0x000F_FFF0</code>), with <abbr title="Random-Access Memory">RAM</abbr> both below and above it? How inconvenient! Well, yes&mdash;and no. "Yes", because in order to run legacy software that expects the original PC's memory map, the original memory map needs to exist. But "no", because they can add a software-settable electronic switch that can move the addresses between the top of the 'old' address space to the top of the 'new' address space. That way, part of switching into Protected Mode would be to also set the switch to move the [[BIOS|<abbr title="Basic Input/Output System">BIOS</abbr>]] from the 1MB area to the 16MB or 4GB area.

Only&hellip; they don't have to. Intel thought about this, and helped manufacturers using the '286 (and above) who didn't want to worry about legacy with a simple trick. At startup, the <abbr title="Central Processing Unit">CPU</abbr> still initialises the <code>CS:IP</code> as <code>F000:FFF0</code>&mdash;since it's in Real Mode, they had no choice. But they also 'fake' <code>CS</code> to reference the address lines above the original 20 lines as <code>1</code> too. That means that at startup, the actual first instruction executed is at <code>0xFF_FFF0</code> (for '286), <code>0xFFFF_FFF0</code> (for '386, '486 and Pentium&trade;), <code>0xF_FFFF_FFF0</code> (for Pentium Pro&trade;) etc.&mdash;the top of the CPU's address space!

But if those address lines are stuck at <code>1</code>, how can code less than 1MB be accessed? Those 'phantom' address bits are defined to remain <code>1</code> until <code>CS</code> is loaded with a new value, either by a far <code>JMP</code>, far <code>CALL</code>, or interrupt&mdash;upon which they'll be loaded with <code>0</code> if the processor is still in Real Mode. This means that the bootstrap code has to carefully NOT load <code>CS</code> until it has somehow established code in the original 1MB (either by 'flipping' the aforementioned switch, or simply putting code in RAM there to execute)&mdash;or it has entered Protected Mode and can access the whole address space.

=== BIOS Boot ===

While the above is interesting in its own right&mdash;and describes just how limited the CPU's execution environment is at first power-on&mdash;most developers don't need to worry about any of this. In fact, most developers don't have to worry about the bootstrap process at all! They just wait (impatiently) for it to take place, then start working with a fully-booted OS. But for the OS developer, the bootstrap process is part of the problem of getting an OS up and working.

Luckily, much of the hard work to boot a PC has been done by the computer manufacturer. They provide a [[BIOS|<abbr title="Basic Input/Output System">BIOS</abbr>]], to not only get the CPU into a much more well-defined state (albeit still in Real Mode&hellip;), but also to provide services that the nascent OS can leverage.

For example, the original IBM&reg; PC&trade; boots from the floppy disk (if one is fitted). There has been a '''LOT''' of code executed to get each part of the PC ready, and typically the last thing that the BIOS does is attempt to load the very first sector of the floppy into RAM at <code>0x0_7C00</code>. If the load is successful, '''AND''' the 512-byte sector has the signature <code>0xAA55</code> at <code>0x0_7DFE</code> (indicating that the code knows it is a boot block), then the BIOS simply <code>JMP</code>s to (or <code>CALL</code>s) <code>0x0000:7C00</code> (or sometimes, annoyingly, <code>0x07C0:0000</code>&hellip;). And as far as the BIOS is concerned, its job is done. (If that code were to do a <code>RET</code>, the BIOS may very well try the next configured boot mechanism&mdash;but that's only in newer BIOSes.)

That 512-byte sector (510 bytes, given the required 2-byte BIOS signature) has to start the OS boot process. There is no way that the entire OS can fit in 510 bytes, so at best all it can do is go looking for the rest of the OS on the floppy&mdash;using the BIOS' services to do the loading. After all, the boot sector can't hold the driver code for the floppy as well as the boot loader&mdash;not least because the floppy driver code may be proprietary to the computer manufacturer! That is the whole point of the BIOS: to provide standard entry points for different services, that are each implemented with code specific to the peripherals supplied with the PC. The history of the PC has been so long that now most standard peripherals have a standard programmatical interface (all driver code can thus be the same), but when the PC was first designed it was recognised that this was not going to be the case at the beginning.

This historical design is still extant in BIOSes today. Exactly the same thing happens in today's PCs&mdash;except that the boot sector is usually loaded from a hard disk (or perhaps a [[USB|<abbr title="Universal Serial Bus">USB</abbr>]] stick) rather than a floppy (see below). There has been a relatively recent press to modernise this process: the Unified Extensible Firmware Interface (UEFI) is slowly replacing the traditional BIOS in new machines (not least due to Microsoft&reg; requiring it for Windows&trade; 10 compliance), and with it the assumptions of what the starting environment is for a boot loader (for one thing, it's already switched to Protected Mode!). But there are sufficient legacy PCs out there&mdash;and UEFI allows Legacy booting&mdash;that it is still worth the effort to understand and even implement a from-BIOS boot system. And then this can be adapted to suit the UEFI environment: after all, the UEFI environment is further down the desired path to a fully-booted OS, so there are some things the boot loader simply doesn't have to do any more!

== BIOS Environment ==

Assuming that the BIOS has just loaded ''your'' code into RAM from the boot media (either floppy, hard disk or USB stick) and <code>JMP</code>ed to it, the default x86 (legacy) BIOS environment is quite limited&mdash;but far more useful than the original power-on state! You can expect the following:

* The CPU is in Real Mode;
* The [[A20]] Gate is (probably) OFF. See [[#A20_Gate|A20 Gate]] below.
* The screen is (probably) in 80x25 text mode&mdash;even if splash-screen graphics were used during the boot process, it should have reverted to text mode for legacy reasons;
* The <code>CS:IP</code> code register pair are ''either'' <code>0x0000:7C00</code> ''or'' <code>0x07C0:0000</code> (both correlate to physical address <code>0x0_7C00</code>);
* The real-mode Interrupt Vector Table (IVT) is at <code>0x0_0000</code>, with various default CPU exception handlers, BIOS interrupt handlers, and hardware [[IRQ|<abbr title="Interrupt ReQuest">IRQ</abbr>]] handlers set up;
* The <code>SS:SP</code> stack register pair is set up ''somewhere'' in RAM&mdash;but this is often (*gasp!*) <code>0x0_0400</code>&mdash;which is right at the end of the IVT.<br />(And remember, stacks grow '''DOWN'''&mdash;so every <code>PUSH</code>, <code>CALL</code> or <code>INT</code> is corrupting the IVT&hellip;);
* The [[Memory_Map_(x86)#BIOS_Data_Area_(BDA)|BIOS Data Area (BDA)]] is at <code>0X0_0400</code>. This is used by the various BIOS services as storage for their state. For example:
** The keyboard state and buffer, holding which keys have been pressed;
** The current video mode, including the number of (text) rows and columns;
** Information about other hardware, such as COM ports and printer ports;
* The 512 bytes loaded from the boot media are at <code>0x0_7C00</code>&mdash;nicely matching the contents of <code>CS:IP</code>;
* The [[Memory_Map_(x86)#Extended_BIOS_Data_Area_(EBDA)|Extended BIOS Data Area (EBDA)]] is at the top of (real-mode) RAM&mdash;something like <code>0x9_FC00</code>. You can find out where from the BIOS;
* The <code>DL</code> register (probably) holds the BIOS identifier for the boot media (either <code>0x00</code> or <code>0x80</code>);
* The <code>ES:SI</code> data register pair ''may'' hold the location of the hard disk boot partition's information in the Master Boot Record (MBR)&mdash;usually at <code>0x0_07xx</code>. See [[#Hard_Disk_Boot_Environment|Hard Disk Boot Environment]] below.

=== Floppy Boot Environment ===

If the boot sector was loaded from a floppy disk, the above information (except for <code>ES:SI</code>) is complete. The <code>DL</code> register should contain <code>0x00</code>.

=== Hard Disk Boot Environment ===

While it is possible to put your boot sector in the first sector of the hard disk (in which case it's almost as though it was loaded from a floppy, except for <code>DL</code> holding <code>0x80</code>), it is probably better to take advantage of the hard disk's partitioning system. Most hard disks are usually partitioned into a number of regions, each with a dedicated role: maybe a boot or recovery partition, or (for Linux) a swap partition. Of course the biggest partition is usually where the OS is installed&mdash;although there may be an OS boot partition and a user data partition. These all appear as separate "volumes" or "drives" to the OS, even though in reality they're merely regions on the same physical disk.

The most common mechanism to partition a hard drive is the legacy Master Boot Record (MBR) partition table, created by such commands as <code>fdisk</code> or <code>diskpart</code>. The more recent [[GPT|<abbr title="Globally Unique IDentifier">GUID</abbr> Partition Table (GPT)]] mechanism uses a more generic (and complicated) system. The MBR mechanism is going to be assumed for the rest of this article.

==== Master Boot Record (MBR) ====

The MBR is an example of a "chain loader". This is a bootloader that attempts to load another bootloader. That bootloader in turn could load yet another, or actually start to load the OS. The MBR doesn't care&mdash;as long as the next loader meets a couple of criteria, it will <code>JMP</code> into it.

The first thing the MBR does is move itself out of the way, and <code>JMP</code> to the new location. The usual relocation point is <code>0x0_0600</code>, freeing up the previous <code>0x0_7C00</code> location as the load point for the next bootloader. The new bootloader doesn't even have to know that it was loaded second: it may not even care.

The next thing the MBR does is examine its own boot sector. The final two bytes are the boot signature <code>0xAA55</code>, but the previous 64 bytes are configured as a four-entry partition table, with each 16-byte partition entry describing a partition (region with a start sector and size) on the disk (or left as all zeroes). That leaves 512-2-64=446 bytes for code to let the MBR "do its thing". The MBR looks through the table at each entry, looking for one marked as the boot partition. If it doesn't find it, it prints an error message and stops.

If it finds an entry marked as a boot partition, it uses the information in the entry to load the first sector of that region into <code>0x0_7C00</code>. If that is not successful, or the last two bytes of that new sector are not the boot signature <code>0xAA55</code>, it prints an error message and stops.

Otherwise, it simply <code>JMP</code>s to the new boot sector's entry point. That boot sector could take advantage of knowing that the MBR (probably) used <code>ES:SI</code> to search the partition table for the partition entry, in which case the new boot sector now knows where on the disk it starts, and how large its partition is!

To write your bootloader into the beginning of the partition rather than the beginning of the hard disk requires more work: rather than a sector number of <code>1</code>, the installer needs to parse the partition table itself to work out what the start sector should be. But that's the installer's job&mdash;an article in its own right!

=== USB Stick Boot Environment ===

A USB-enabled BIOS can boot off a USB stick by treating it as a pretend hard disk. If the BIOS is implemented correctly, the boot loader doesn't even know that it isn't loading off a hard disk, since the standard BIOS calls will "just work". Depending on how large the USB stick is, it may or may not even have its own MBR&mdash;in all respects, the boot loader can boot from the USB stick as though it was a hard disk (see above). As a development environment target, a USB stick is actually quite useful: it's cheap, easily removable so the main OS can be booted back into, and not so much of a worry that you might "trash" the whole stick&mdash;just use the main OS to reformat it and try again.

=== CD-ROM Boot Environment ===

A [[ISO_9660|<abbr title="Compact Disc - Read Only Memory">CD-ROM</abbr>]] is also viable boot media, as can be seen by the numerous so-called "Live CDs" that are circulating to allow you to try a different OS before actually installing it. You can burn a CD-ROM image onto a CD, or just use the virtual image stored in a <code>.iso</code> file as the starting point for a Virtual Machine (see [[VMware]], [[Microsoft Virtual PC]] and [[Bochs]]).

Because the layout of a CD-ROM is so different from earlier media (for one thing, it's designed to be read by a wide variety of different computers, so its native format is nothing like any others!), it was designed to also support legacy modes. It can be marked to pretend to be a floppy (with tiny storage), a hard disk, or none of the above&mdash;just used natively. The first two modes mean that all the previous boot environments are available&mdash;but as soon as your OS boots, all pretence is gone and the OS will need to continue to access it as a true CD-ROM. That means that the early part of the bootstrap code can be the same as before, but true CD-ROM drivers and [[#File_System|file system]] code would be needed after that.

== Bootstrap Phases ==

Now that your OS bootstrap code is about to start executing&hellip; what should it do first? What '''''must''''' it do, what is easier done earlier rather than later, and what can be postponed until last? It of course depends on what features of the computer the OS will take advantage of, and how the OS will arbitrate its services to the programs that will run under its umbrella. Given an end goal of a completely booted OS, getting ''there'' from ''here'' depends a lot on what you want the OS to do.

I think of the bootstrap process as a series of phases. Assuming a BIOS boot:

* There are things that need to be done before entering Protected Mode. For example, once in Protected Mode the BIOS is effectively unavailable;
* After entering Protected Mode, the next goal may be Paging Mode&mdash;although you can do both of these at the same time, it may be easier to be able to access the whole CPU's address space from Protected Mode before starting Paging;
* After Paging Mode is enabled, there's probably still many things that need to be set up before the system is ready to start executing its first program.

The OS that I'm designing is a full disk-based, multi-processing, multi-threaded, segmented, virtual memory operating system. The OS will be providing all of the services (at a minimum) that the BIOS currently performs&mdash;without access to the BIOS. It thus needs to re-implement them inside the OS environment. The OS boot process will therefore be very long and involved, and each of the above phases needs to be carefully planned and implemented to avoid duplicate effort, and wasted space or time. For example, just looking at one aspect of the OS from top down:

# The booted OS needs to load files from the hard disk as part of its operation;
# The boot process needs to load various modules of the OS, as files, from the hard disk;
# The boot sector needs to load non-BIOS-specific code (including the disk reading code!) from the hard disk just to start the bootstrap process.

Since 3. above is needed before even switching to Protected Mode, the boot sector '''must''' use the BIOS to load more of the bootstrap code. But since 2. above needs to load modules while in Protected Mode, it means that it needs to understand how to both read from the boot media as well as follow the file system to find the modules to load. It either needs its own cut-down version of what the OS will do, sufficient to load modules for booting but then discarded when boot is complete&mdash;or else it uses the OS' actual media-reading code and file system parse code to load the modules. The latter implies that ''that'' code is loaded as part of 3. above.

=== Before Protected Mode ===

The bootloader needs to be generic, but that means that it has to interrogate the computer that it is running on to find out its setup. That could be a single lookup ("Oh! This is a Raspberry Pi 3 Model B+. I need the following modules..."), but on a PC it is ''much'' more complicated! In fact, the only way to know on a PC is to ask the BIOS various questions, or get the BIOS to make various changes in that computer's specific way. And (with only a couple of exceptions) the BIOS is not callable in Protected Mode&mdash;all the questions and settings need to be done before doing the switch, or else the bootloader needs to allow back-and-forth transitions between Real Mode and Protected Mode (complicated!). As highlighted above, the most important of the BIOS-reliant functions is simply loading the rest of the bootloader&mdash;or at least sufficient so that the OS can start to "fend for itself"!

And of course the bootloader needs to set up the system preparatory to the transition to Protected Mode. According to Intel&reg;, the only explicit requirement before the switch is that a minimal Global Descriptor Table (GDT) needs to be set up. In my design, I also want to set up the Interrupt Descriptor Table (IDT) too: I want to be able to diagnose any exceptions that my code may generate, and switching to Protected Mode and only ''then'' setting up the IDT is a lot of Protected Mode code that may go awry before it's ready.

Finally, there is some code that only needs to execute once per boot: it may switch a mode in the computer, or initialise a complicated memory structure. Once run, that code would only sit there taking up space&mdash;except that the last thing that the Real Mode bootloader does is to <code>JMP</code> to the Protected Mode code, which could easily be somewhere completely different. That means that the Real Mode code can be simply abandoned: overwritten later in the course of the OS' normal operation, not realising that it was obliterating the very thing that gave it life! Or, less emotively, I often liken the bootloader to a rocket booster stage: something that is discarded after it has fulfilled its function. This looks like a perfect spot for once-off code!

==== BIOS Functions ====

===== Boot Loader =====

===== Memory Map =====

==== Once-off Code ====

===== A20 Gate =====

===== Video Mode =====

==== CPU System Structures ====

===== Global Descriptor Table (GDT) =====

===== Interrupt Descriptor Table (IDT) =====

===== Local Descriptor Table (LDT) =====

=== Before Paged Mode ===

==== CPU System Structures ====

===== Page Tables =====

=== Before Running First Program ===

By "first program", I really mean "first user program". I like to think that the actual first program any OS should run is its own bootstrap program. By writing that as a standard OS program, it means that it can leverage the services that the OS provides (memory allocation, file services, and even networking), as well as setting up each service as it is being installed. And finally, it can be "unloaded" when it finishes, and not hang around until the OS is shut down.

Of course, that means that the OS' program run-time environment needs to have been designed:
* How do programs call into services like the file system or networking stack?
* How do device drivers provide services to configure what they do?
* How does the OS service requests for resources such as memory?
````

<a id="w004354"></a>

## W004354 — User:Fallenavatar

Bron-ID: W004354
Namespace: User (2)
Revisie: 23212 | Datum: 2019-01-16T06:58:49Z
Bron: https://wiki.osdev.org/User:Fallenavatar
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AFallenavatar&oldid=23212

````text
== WSL ==
All code is free for anyone else to use in this section. USE AT YOUR OWN RISK!

[[User:FallenAvatar/WSL_Notes]] (WIP)

[[User:FallenAvatar/WSL_Bare_Bones]] (WIP)
````

<a id="w004355"></a>

## W004355 — User:FallenAvatar/WSL Notes

Bron-ID: W004355
Namespace: User (2)
Revisie: 23213 | Datum: 2019-01-16T06:59:08Z
Bron: https://wiki.osdev.org/User:FallenAvatar/WSL_Notes
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AFallenAvatar%2FWSL_Notes&oldid=23213

````text

== Random Notes ==

* 100% working on my end for Bare Bone and Meaty Skeleton
* WSL is running Ubuntu 14.04
* WSL does not have an install of GRUB (use grub-pc-bin)
* WSL has a rather out of date version of GCC install by default (4.8.2)
* To build GCC need bison, flex, byacc, as well as the prereqs mentioned on [[GCC_Cross-Compiler#Downloading_the_Source_Code]]
* Need to update pre-built gcc, [http://askubuntu.com/a/581497/353923 Server Fault - How do I use the latest GCC on Ubuntu?] use gcc-6 instead of gcc-4.9
* Recommended list of packages to apt-get before starting
** git
** bison
** flex
** byacc
** build-essential
** gcc-7 (After adding PPA: sudo add-apt-repository ppa:ubuntu-toolchain-r/test)
** libgmp3-dev
** libmpfr-dev
** libmpc-dev
** texinfo
** grub-pc-bin
** xorriso
* Need to come up with steps and test all of the above to make sure it is enough.
* Optional/Helpful packages to apt-get
** genisoimage (provides "isoinfo" which can be helpful for sanity checking iso files)
** nasm (if you prefer Intel syntax over AT&T)
* [[Bare_Bones]] does work with all prereqs mentioned above. Running the kernel (with -kernel) or the iso (with -cdrom) in QEMU works (confirmed by myself +2 on the forums)
* [[Meaty_Skeleton]] works same as the Bare Bones (works for me in qemu, maybe others, though unconfirmed by myself)
* For cleanup purposes, [http://askubuntu.com/questions/17823/how-to-list-all-installed-packages]This is a good way to see all "installed" packages. Adding a "> file.txt" after it will allow you to easily reference and review the list to decide what needs to be removed. And "sudo apt-get autoremove" should be your final command (repeated until nothing shows up if necessary)

<pre>
[19:52] <Kazinsal> should add something about installing a Windows-based X server like Xming
[19:52] <Kazinsal> you can then set up your bash startup scripts to export DISPLAY=0:0
[19:52] <Kazinsal> and then anything X or SDL inside WSL will be forwarded to Xming or whatever
</pre>

== Step by Step ==

# Prerequisites
## A fully up to date Windows 10 x64 install
## [[Required Knowledge]]
# Install WSL
## Open Control Panel -> Program and Features -> Turn Windows features on or off
## Select "Windows Subsystem for Linux (Beta)"
## Hit OK
## Let it install
## Reboot (Yes, you MUST reboot before continuing)
# Installing BASH
## Open an elevated command prompt
## Run "bash" and follow the instructions/prompts provided
# Setup shared folder (Optional)
## Create a folder somewhere in your windows filesystem that you will use for easy access (I created a wsl folder in My Documents)
## "cd ~"
## "ln -s "wsl" "/mnt/c/Users/<WindowsUserName>/Documents/wsl" Replacing paths as you prefer
# Installing and Updating software
## Updating system GCC
### Add PPA for gcc-7: sudo add-apt-repository ppa:ubuntu-toolchain-r/test
### Update apt lists so it can find gcc-7: sudo apt-get update
### Install GCC 7: sudo apt-get install gcc-7
### Point gcc to the new version: sudo update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-7 60 --slave /usr/bin/g++ g++ /usr/bin/g++-7
#### You can see here if you want to have multiple copies of gcc installed and be able to switch between them: https://askubuntu.com/questions/466651/how-do-i-use-the-latest-gcc-on-ubuntu/581497#581497
## Installing/Compiling a cross compiler
### Prereqs
#### sudo apt-get install make bison flex libgmp3-dev libmpfr-dev libmpc-dev texinfo
### Follow [[GCC_Cross-Compiler]]
## Install additional tools
### sudo apt-get install git grub-pc-bin xorriso
### Optional: sudo apt-get install nasm genisoimage
## Clean Up
### sudo apt-get autoremove
#### Repeat til nothing shows up
# Bare Bones
## See [[Bare_Bones]]

== ToDo ==
# Finish Step-by-Step
# Test Step-by-Step + clean-up
# Remove specific version numbers and add instructions to find "current" versions
# Test again
# Seperate instructions and add to appropriate Wiki Pages
## [[GCC_Cross-Compiler]]
## [[Building_GCC]]
## [[Bare_Bones]]
````

<a id="w004356"></a>

## W004356 — User:Eekee

Bron-ID: W004356
Namespace: User (2)
Revisie: 29202 | Datum: 2024-07-30T15:00:04Z
Bron: https://wiki.osdev.org/User:Eekee
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AEekee&oldid=29202

````text
Update: I've been slowly improving at Forth, but now I'm finding [[Plain English Programming]] a huge leap forward. It completely sidesteps my problems reading code, and so I have a much larger set of examples to learn from than with any other practical systems language. It helps that most definitions are very small, a point encouraged by the absence of nested loops or conditions. Forth was also rather good for me, but required other skills which were only coming to me very slowly. Even in the simplest definitions, tracking stack order is still sometimes a challenge. Organizing data in memory is something I'm not as good at as I thought I was. Plain English has high-level types. It has conventional local variables too, but gives them generic names based on their type. I like that. Lisp was my second choice, but Plain English is actually a little closer to the machine. I like that too.

I'm not sure I'll do any OS development in Plain English, but some OS dev concepts are applicable to the plans I have for it. The IDE (which doubles as a sample program) rather resembles an operating system in use, being permanently full-screen with its own way of working, but that's more a point of curiosity than anything technical. 

I still might make a Forth interpreter as a BIOS or UEFI application. Plain English would need a bit more work, being 32-bit only at present.

----

Hello! I'm the offspring of [[James T. Klik]] and [[Lino Commando]]! I'm joking, but I am slowly developing a system with mixed REPL and mouse input. It's not necessarily going to run on real hardware or even Qemu, but I do rather dream of it doing so. Similarly, I like to flirt with [[Alta Lang]] but in practice I'm 'just' developing on Forth, with plans to build an extensive library. My dream is for it to run under operating systems and directly on hardware, a possibility achieved by [https://duckduckgo.com/?q=inferno+os Inferno] and [http://www.ocp.inf.ethz.ch/wiki/OCP/Home Oberon]. Also like Inferno and its parent OS, Plan 9, I plan to make heavy use of IPC. Unlike Inferno and Plan 9, it will not be "just plain text", structured data will be the norm. I'm not entirely sure why I'm targetting hardware, but there's a certain freedom in it and it does rather satisfy a very long-standing ambition.

This system is Kaph. It ''was'' a huge project, a whole panoply of high-level goals, which rather put me off for a while, so I came up with some other projects and listed them below. Now I see I can begin by implementing a Forth, build a partial version of Kaph on top of that, and see what changes and additions need to be made.

== More about me ==

My levels of experience are quite atypical due to my health holding me back from accomplishing much but not always holding me back from observation. I've had decades to absorb concepts and see what does and doesn't work, but little practice. My health is not a fun subject, so let's leave it at that. I'm getting a bit done now, and I'm finding the process of development to be a lot of fun in itself. 

I have had ''some'' practice. For instance, some time ago, I tried to be a [https://wiki.osdev.org/What_order_should_I_make_things_in#Ideas_for_new_Archetypes Real Man], but then I learned to relax. I am definitely not Barry Pi, I'll take a consistent architecture, thank you. The existence of Andy Microbaum is tangential to my universe, much like it is to Plan 9. I don't need to say "I love these archetypes," do I? :)

::One day, Uriel met a man who argued convincingly that Plan 9 is a microkernel.
::On another day, Uriel met a man who argued convincingly that Plan 9 is a macrokernel.
::Uriel was enlightened.

I have dabbled in CPU design a little. I'm obviously tempted to pursue this further, but I seem to have even more to learn about this than OS design, and that's just for a "simple" CPU; no MMU. Trying to design 8-bit CPUs alerted me to some surprising complexities. Still, when I look at Dawn, reviewed below, I start thinking of it again. If you want to design CPUs complete with their own instruction sets, I recommend starting with a larger word size than 8 bits.

Ultimately, I'm sure it would be much more fun to develop for a made-up architecture, even if only in emulation, than for any hardware containing such horrors as ACPI. I'm told USB is very unfriendly to operating system development too (although I'm starting to think it depends on your ideas about how things should be structured). Another possibility is developing for emulated older hardware, which in turn can and in some cases has been reimplemented in FPGA. The more time goes by, the more possibilities open up in hardware and the more lines blur. I should remember to look into RISC-V. This all reminds me, I wonder if the GNU universal peripheral bus is anywhere near ready, or indeed if it's sensibly designed. 

== Kaph Project and Concepts ==

My plans for Kaph include several distinct concepts which are unusual in themselves or unusual at the OS level. 
* An unusual (but not entirely unique) UI design with REPL and modifiable GUI in close association -- Inspired by colorForth and the Sam text editor
* Execute or "plumb" any text, where the "plumber" is a very powerful spin on file association -- Inspired by Oberon via the Acme text editor on Plan 9
* Text-heavy GUI where any text is selectable and much or all text is editable -- To empower the plumber, assist the REPL, increase the power of the GUI (see previous point), and because forcing the user to copy-type is very poor
* A single language embracing domain-specific languages -- Inspired by my difficulties with Plan 9's maze of twisty little languages, none quite alike
* Universal data interchange structures and mechanisms intended to make pipe filtering quicker and easier, and to ease the process of manipulating data with different DSLs
* Modification of in-memory data by transient programs -- Using Acme with Unix-like Plan 9 commands showed me how practical this way of working is. Wanting to use it for images too was a seed idea for Kaph. Some Forth block editors are essentially this; the text's existence in memory is independent of the lifetime of the commands which modify it.
* Unified device interface; perhaps not "everything is a file", but reducing "everything" to a very small number of categories with the help of the universal data interchange structures -- Inspired first by Atari 8-bit home computers, then Unix and Plan 9, but I see a need to avoid oversimplification
* Scalable UI is not quite so uncommon in this decade, but there are still areas where improvement is clearly possible
* Increased availability of undo, especially undoing file changes at the OS level -- Sam & Acme taught me that undo solves the problem of running extremely powerful text processing commands without thinking about them first. I don't think I can make this as system-wide as a certain mainframe OS, but I would at least like it in the filesystem.

=== UI and window system ===

Acme's facility of executing or plumbing any text anywhere while almost all the text is editable adds up to an extremely powerful system. I would like to capture this power without requiring a 3-button mouse. If I can make it work on touchscreens, I'll be very happy indeed.

On the other hand, I don't want to imitate Acme's dynamic window tiling. Nor do I want to allow programs excessive control over their windows. I've been happy with simple, static window tiling, and this is what I want to implement for Kaph. I intend to have a single REPL window serving whichever window is current, but I have yet to work out exactly how it will integrate.

Under the surface, a big part of Acme's power comes from scripting it. The ability to execute anything means anything can be a script, whether global, specific to the editor, a script in a directory (very powerful), or a line or two in a plain-text file. Such scripts can communicate with Acme, either by being run under Acme's Edit command or by using the virtual files Acme serves.

The plumber too is programmable. In Plan 9, it is designed to send messages to ports, but I plan to simply have it execute scripts. The scripts can send messages to programs if desired. This implies many programs will be able to receive relatively arbitrary messages, much like Amiga programs with their AREXX ports. (I just thought, this offers a means of implementing async i/o: start a command which will send a message when the i/o is ready. If the commands and message-passing are as lightweight as I hope, this may be reasonable.)

=== Language ===

[discuss forth; mention lisp; mention making forth safer; start to consider pep]

=== Data interchange structures ===

In Plan 9 and Unix, I got very tired of addressing fields by number or trying to make regexps to match the right part. For example, try picking dates out of ls -l output. It's slightly easier in Plan 9 because the output is fixed; it's not internationalized and doesn't change depending on whether the output is a terminal or pipe, (the latter is sometimes disgusting!) but 'slightly easier' is still far from easy. The fields have names, why can't I address them by name? In Kaph, lists of files, processes, or other things will be represented as data structures with named and typed fields. [to do: more detail]

~~page update WIP~~

old:

A more "meta" thing I learned from Sam Acme and Plan 9 is the power of having a few simple but powerful interfaces with a powerful command language behind them, rather than many discrete applications. Plan 9 isn't actually the best in this regard, having many disparate languages which take a bit of thought to interface together, and the easy way is often the fragile way. This is why Kaph will have the one language, but this is not to say it will exclude DSLs. (DSL = Domain Specific Language, a small language for a specific task.) 

Forth is perhaps ''the'' ideal language for integrating DSLs into itself. In Charles Moore's wording, it was a 'compiler' before it was a language. He came up with this compiler system and used it to implement "problem oriented languages" before developing a full language with it. See [https://colorforth.github.io/POL.htm Programming a Problem-Oriented Language]. 

Lisp is also good for DSLs. Both Forth and Lisp are especially good because they mean you don't have to write a parser for each DSL. You just use the existing parser, only spreading out the syntax of your DSL a little bit. I even prefer this spreading; I find printf and regexps too terse to read, and I hate having DSLs stuffed into strings where normal code formatting practice breaks down.

I have been tempted to use some variety of Lisp instead of Forth, but I'm not keen on its basic principles. I would rather use a language closer to the hardware and find a safe way of working with it.

== Minor Projects ==

=== Various Native Forths ===

Because Forths are relatively easy to make and the results are quite satisfying. Also, it's a form of practice for some aspects of OS development. I may write some up as barebones tutorials. 

Progress and plans:
* I have a direct-threaded real-mode x86 Forth in development. It might be working already, but I felt it important to concentrate on personal development instead.
* I have some embedded ARM hardware; the obvious next target.

'Threaded' here is Forth interpreter design jargon, not a word for one form of multitasking. It's because the execution flow jumps between routines in the manner of a literal thread in a complex weave or seam. I initially intended to immitate the double-indirect threaded Jonesforth, but all that indirection was too much for my brain. Besides, double-indirect threading was invented to save memory and was apparently faster on some hardware, but stores addresses as data. These days, I think I'd rather save the data cache for actual data. Direct-threaded Forths are simpler, and [http://pygmy.utoh.org/pygmyforth.html Pygmy Forth for DOS] shows they are perfectly capable of being fast and tiny even on an 8088.

There are many public domain Forths; plenty of code to use if desired. This of course raises the possibility of just porting one instead of developing from scratch. However, developing Forth programs is greatly improving my programming skills.

=== A8.86 ===

When I was 13 or 14 years old, the ''Atari 8-bit Home Computer Operating System Manual'' showed how many peripherals of very different types and needs could be operated through a unified, consistent interface. It blew my mind! When I first got a MS-DOS reference book some years later, I was repulsed by how ad-hoc and junky the interfaces were by comparison. Naturally, I wanted to make an OS imitating Atari's design, but I didn't have the programming or organizational skills in the 90s. When I got Linux years later I found some clean Unix interfaces, but many obstructively non-uniform interfaces too. I was also astonished to find how primitive and generally rough Unix terminal interfaces are. Atari provided a full-screen text editor for all-purpose input starting in 1978! Plan 9 went some way toward fixing all this, but was still disappointing in some ways as outlined under Kaph above. ''So,'' if I had time, I'd still like to produce an OS imitating Atari's. :) However, most of the features I want have ended up as part of Kaph.

I might still do something along these lines as a sort of Kaph-lite. It would have to have some interesting graphics hackery because I'm imitating Atari. For you Amiga fans, Amiga Inc. was working with Atari before they approached Commodore. I don't know how closely they worked together, but the graphics features some find so amazing in the Amiga are entirely typical of ''Atari's'' way of thinking. ;) They're obvious next steps from the graphics systems of the 8-bit Ataris. I don't blame Amiga Inc for ditching Atari; Atari's boss at the time was a horrible person to work with and Commodore offered them literally 5 times more money.

You know, "interesting graphics hackery" suggests a hardware project... I'm thinking "A8.ARM" ''hardware'' running Kaph. That would be great, if not too much of a challenge! :D But speaking of challenges and hardware, I've had thoughts of designing my own CPU...

== Suggestions ==

=== Remote Shinies ===

If you want to be [[James T. Klik]], may I recommend becoming [[Nick Stacky]] first? Using other operating systems' graphics drivers is great! Of course, you will have to design a networked window system or interface to X. ''Don't'' try to use Wayland over X, basic input breaks down over the network! VNC is also a bit rubbish in the input stakes. X may be tough, but it gets it right. Plan 9 also gets it right, but doesn't have drivers so it's more of an example you may wish to imitate. Plan 9's windowing is much simpler than X, but note that you'll have to understand its filesystem semantics to understand how its window system is exported correctly. In places, it takes a genius to understand Plan 9's simplicity! ;)

== Reviews ==

Note to self: try more OSs! I had plans for this section, but I always seem to be doing something else. Occasionally, the "something else" is even OS coding. >.>

===FreeDOS===

I dip into FreeDOS every now and then, especially to code in 16-bit assembly language, but almost everything is unstable. It's just-about usable on my Thinkpad x61 (2007), was hardly usable on Qemu years ago, and seems even worse on a more recent version of Qemu. Even on the Thinkpad, many editors will crash or develop unpredictable bugs if you touch the thumbstick while pressing a key.

I use software from the 3 released FreeDOS ISOs. The best text editors have severe interrupt/keyboard issues under Qemu, although they're not so bad on real hardware. They're all very different from each other, most having as little relation to each other as Vim has to Notepad. They also tend to have peculiar design choices -- I have a file with notes on each editor which makes me alternately laugh, cry, and yell WHY? ;) I use SetEdit (despite instability) because it saves its session when I quit to run Nasm, and I use very simple batch scripts to run Nasm. I prefer quitting the editor to starting a subshell because the subshells of many editors don't work in FreeDOS, (especially not in Qemu,) and they are often cumbersome to invoke anyway.

There are several GUIs for DOS. 2 or 3 of them appear to be virtually OS projects in the way Windows 1-3 were OSs built on DOS. They're unstable. OpenGEM is stable, but preserves some of the the worst clunkiness of 80s GUI, including clearing the selection when you scroll. (Atari fixed that, but the fix never reached OpenGEM.) 

I stuck with the shell, using COMMAND.COM-compatible FreeCOM until I found it difficult to grep for a regexp containing a single double-quote character. I had to add a double-quote to the file list and put up with the "file not found" error, otherwise FreeCOM would ignore the redirection character on the same line. Now I use 4DOS which has a proper means of quoting arguments.

4DOS also allows setting the default text colours, reducing the problem I have with bright-for-bold displays. However, the problem returns as soon as I pipe text into a pager, or run some unconfigurable programs or run an editor whose configuration doesn't work.

I've had a little fun with assembly-language coding and some Forth and BASIC interpreters, but I don't really enjoy FreeDOS, or any DOS. There are some nice games, but I'd rather game on my tablet. And the one Forth interpreter I want to keep using, I run under DOSBox.

===Nano-reviews===

* '''9front''' I used full-time for about 10 years! In a sense, it was my primary OS. I only used Linux for web browsing, 3D virtual worlds, and occasionally OpenTTD or some other game. Everything else I did on 9front. However, there were some frustrations. I didn't have enough free creative energy to overcome the lack of configurability. Many of the little languages were uncomfortable, and so were the differences between them. If you're comfortable with such languages and with deeply sophisticated C, it's a very special system! So is its parent, Plan 9 from Bell Labs.
* '''KolibriOS''' is a bit of a favourite because it was an early find which is impressively complete and capable, and because it has simple APIs. However, its GUI is locked in the past with tiny fonts I can no longer comfortably read. I have ''endured'' many traditional GUIs since the 1980s so I can say KolibriOS's GUIs are better than average (apart from the size); stable and less clunky than some, but it's not up to the UI standards of the present era.
* '''MenuetOS''' gets a mention because it's KolibriOS's parent. I believe the APIs are also simple even for assembly-language programming, as are Kolibri's. It's certainly capable of rendering larger, smoother text. However, it has very few user programs by comparison, perhaps it because it switched to closed-source almost at the height of open-source enthusiasm.
````

<a id="w004357"></a>

## W004357 — Memory Map (x86

Bron-ID: W004357
Namespace: Main (0)
Revisie: 23244 | Datum: 2019-01-23T11:46:57Z
Bron: https://wiki.osdev.org/Memory_Map_(x86
Revisie-URL: https://wiki.osdev.org/index.php?title=Memory_Map_%28x86&oldid=23244
Doorverwijzing: Memory Map (x86)

````text
#REDIRECT [[Memory Map (x86)]]
````

<a id="w004358"></a>

## W004358 — User:Khorlane

Bron-ID: W004358
Namespace: User (2)
Revisie: 23332 | Datum: 2019-01-31T16:14:28Z
Bron: https://wiki.osdev.org/User:Khorlane
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKhorlane&oldid=23332

````text
= MyOS =
Yep, I don't have good name, yet.
= Description =
Playing around with creating my very own [[Lino Commando]] Hobby OS for [[IA32_Architecture_Family|32-bit x86]] written in assembler using [[NASM]]. The development machine is a Microsoft Surface running Windows 10. The testing machine is [[Bochs]].

Currently, MyOS gets itself into Protected Mode and that about it.

= Credit =
Most of the code comes directly from [http://www.brokenthorn.com/Resources/OSDevIndex.html BrokenThorn Operating System Development Series]

In addition, I've read through many of the OS Dev [[Tutorials]] and some of the FYSOS books [http://www.fysnet.net/osdesign_book_series.htm Operating System Design Book Series] by Benjamin David Lunt.

= Tools Used =
'''Assembler'''

[[NASM]]

'''Disk Emulator'''

[[ImDisk]]

'''Hex Editor'''

[http://en.wikipedia.org/wiki/UltraEdit UltraEdit]
or
[http://mh-nexus.de/en/hxd/ HxD]

'''Virtual Machine'''

[[Bochs]]

= The Code =
== Stage 1 ==
Main task is to load Stage 2.
<source lang="asm">
;**************************************************************************************************
; Stage1.asm
;   A Simple Boot Sector that:
;   1. is exactly 512 bytes long
;   2. has the Magic Word at the end (0xAA55)
;   3. allows us to have a useable floppy
;      where we can put our Stage2/Stage3 code
;      by coding a proper BIOS Parameter Block
;   4. has code to load our Stage2 code
; Broken Thorn Entertainment
; Operating Systems Development Tutorial
; http://www.brokenthorn.com/Resources/OSDevIndex.html
;
; nasm -f bin Stage1.asm -o Stage1.bin -l Stage1.lst
;**************************************************************************************************

[bits 16]                               ; we are in 16 bit real mode
    ORG   0                             ; we will set regisers later
    JMP   Booter                        ; jump to start of bootloader

;--------------------------------------------------------------------------------------------------
; BIOS Parameter Block
;   and yes, this block must start at offset 0x003
;   and yes, these are the required fields
;   and yes, they must be in this order
;   you can change the names (obviously)
; Tutorial 5: Bootloaders 3
;--------------------------------------------------------------------------------------------------

; The BIOS Parameter Block begins 3 bytes from start. We do a far jump, which is 3 bytes in size.
; If you use a short jump, add a "nop" after it to offset the 3rd byte.
; See Wikipedia "Design of the FAT file system" for more info on the BIOS Parameter Block

                                        ; Hex Offset from beginning of Boot Sector
OEM                   DB "MyOs    "     ; 0x003  8 bytes padded with spaces
BytesPerSector        DW 512            ; 0x00B  2 bytes
SectorsPerCluster     DB 1              ; 0x00D  1 byte
ReservedSectors       DW 1              ; 0x00E  2 bytes
NumberOfFATs          DB 2              ; 0x010  1 bytes
RootEntries           DW 224            ; 0x011  2 bytes
TotalSectors          DW 2880           ; 0x013  2 bytes
Media                 DB 0xf0           ; 0x015  1 byte
SectorsPerFAT         DW 9              ; 0x016  2 bytes
SectorsPerTrack       DW 18             ; 0x018  2 bytes DOS 3.31 BPB
HeadsPerCylinder      DW 2              ; 0x01A  2 bytes DOS 3.31 BPB
HiddenSectors         DD 0              ; 0x01C  4 bytes DOS 3.31 BPB
TotalSectorsBig       DD 0              ; 0x020  4 bytes DOS 3.31 BPB
DriveNumber           DB 0              ; 0x024  1 byte  Extended BIOS Parameter Block
Unused                DB 0              ; 0x025  1 byte  Extended BIOS Parameter Block
ExtBootSignature      DB 0x29           ; 0x026  1 byte  Extended BIOS Parameter Block
SerialNumber          DD 0xa0a1a2a3     ; 0x027  4 bytes Extended BIOS Parameter Block
VolumeLabel           DB "MYOS FLOPPY"  ; 0x028 11 bytes Extended BIOS Parameter Block
FileSystem            DB "FAT12   "     ; 0x036  8 bytes Extended BIOS Parameter Block padded with spaces

;--------------------------------------------------------------------------------------------------
; Prints a string
; DS => SI: 0 terminated string
;--------------------------------------------------------------------------------------------------
Print:
    MOV   AH,0Eh                        ; set function code 0E (BIOS INT 10h - Teletype output) 
PrintLoop:
    LODSB                               ; Load byte at address DS:(E)SI into AL
    OR    AL,AL                         ; If AL = 0
    JZ    PrintDone                     ;   then we're done
    INT   10h                           ; Put character on the screen using bios interrupt 10
    JMP   PrintLoop                     ; Repeat until null terminator found
  PrintDone:
    RET                                 ; we are done, so return

;--------------------------------------------------------------------------------------------------
; Convert CHS to LBA
; Given: AX = Cluster to be read
; LBA = (Cluster - 2) * sectors per cluster
; Tutorial 6: Bootloaders 4
;--------------------------------------------------------------------------------------------------
ClusterLBA:
    SUB   AX,0x0002                     ; Adjust cluster to be zero based
    XOR   CX,CX                         ; CX =
    MOV   CL,BYTE [SectorsPerCluster]   ;  SectorsPerCluster
    MUL   CX                            ; AX = AX * CX
    ADD   AX,WORD [DataSector]          ; AX = AX + Data Sector
    RET

;--------------------------------------------------------------------------------------------------
; Convert LBA to CHS
; AX => LBA Address to convert
; absolute sector = (LBA %  sectors per track) + 1
; absolute head   = (LBA /  sectors per track) MOD number of heads
; absolute track  =  LBA / (sectors per track * number of heads)
; Tutorial 6: Bootloaders 4
;--------------------------------------------------------------------------------------------------
LBACHS:
    XOR   DX,DX                         ; DL = Remainder of
    DIV   WORD [SectorsPerTrack]        ;  AX \ SectorsPerTrack
    INC   DL                            ;   Plus 1
    MOV   BYTE [AbsoluteSector],DL      ;    Save DL 
    XOR   DX,DX                         ; DL = Remainder of
    DIV   WORD [HeadsPerCylinder]       ;  AX \ HeadsPerCylinder
    MOV   BYTE [AbsoluteHead],DL        ;   Save DL
    MOV   BYTE [AbsoluteTrack],AL       ; Save AL (what's left after the above dividing)
    RET

;--------------------------------------------------------------------------------------------------
; Reads a series of sectors
; CX    => Number of sectors to read
; AX    => Starting sector
; ES:BX => Buffer to read to
; Tutorial 5: Bootloaders 3
;--------------------------------------------------------------------------------------------------
ReadSector:
    MOV   DI,0x0005                     ; five retries for error
ReadSectorLoop:
    PUSH  AX
    PUSH  BX
    PUSH  CX
    CALL  LBACHS                        ; convert starting sector to CHS
    MOV   AH,0x02                       ; BIOS read sector
    MOV   AL,0x01                       ; read one sector
    MOV   CH,BYTE [AbsoluteTrack]       ; track
    MOV   CL,BYTE [AbsoluteSector]      ; sector
    MOV   DH,BYTE [AbsoluteHead]        ; head
    MOV   DL,BYTE [DriveNumber]         ; drive
    INT   0x13                          ; invoke BIOS
    JNC   ReadSectorOk                  ; test for read error
    XOR   AX,AX                         ; BIOS reset disk
    INT   0x13                          ; invoke BIOS
    DEC   DI                            ; decrement error counter
    POP   CX
    POP   BX
    POP   AX
    JNZ   ReadSectorLoop                ; attempt to read again
    INT   0x18
ReadSectorOk:
    MOV   SI,ProgressMsg
    CALL  Print                         ;
    POP   CX
    POP   BX
    POP   AX
    ADD   BX,WORD [BytesPerSector]      ; queue next buffer
    INC   AX                            ; queue next sector
    LOOP  ReadSector                    ; read next sector
    RET

;--------------------------------------------------------------------------------------------------
; Boot Loader Entry Point
;--------------------------------------------------------------------------------------------------
Booter:
    ;-------------------------------------------------------
    ;- code located at 0000:7C00, adjust segment registers
    ;-------------------------------------------------------
    CLI                                 ; disable interrupts
    MOV   AX,0x07C0                     ; setup
    MOV   DS,AX                         ;  registers
    MOV   ES,AX                         ;   to point
    MOV   FS,AX                         ;    to our
    MOV   GS,AX                         ;     segment

    ;--------------
    ;- create stack
    ;--------------
    MOV   AX,0x0000                     ; set the
    MOV   SS,AX                         ;  stack to
    MOV   SP,0xFFFF                     ;   somewhere safe
    STI                                 ; restore interrupts

    ;-------------------------
    ;- Display loading message
    ;-------------------------
    MOV   SI,LoadingMsg                 ; si points to first byte in message
    CALL  Print                         ; print message

    ;--------------------------
    ; Load root directory table 
    ; Tutorial 6: Bootloaders 4  
    ;--------------------------
    ; compute size of root directory and store in "cx"
    XOR   CX,CX                         ; zero out cx
    XOR   DX,DX                         ; zero out dx
    MOV   AX,0x0020                     ; 32 byte directory entry
    MUL   WORD [RootEntries]            ; total size of directory
    DIV   WORD [BytesPerSector]         ; sectors used by directory
    XCHG  AX,CX                         ; swap ax cx
    ; compute location of root directory and store in "ax"
    MOV   AL,BYTE [NumberOfFATs]        ; number of FATs
    MUL   WORD [SectorsPerFAT]          ; sectors used by FATs
    ADD   AX,WORD [ReservedSectors]     ; adjust for bootsector
    MOV   WORD [DataSector],AX          ; base of root directory
    ADD   WORD [DataSector],CX
    ; read root directory into memory (7C00:0200)
    MOV   BX,0x0200                     ; read root dir
    CALL  ReadSector                    ;  above bootcode

    ;-------------------------------
    ; Find Stage 2 in Root Directory
    ; Tutorial 6: Bootloaders 4
    ;-------------------------------
    MOV   CX,WORD [RootEntries]         ; load loop counter
    MOV   DI,0x0200                     ; locate first root entry
FindFat:
    PUSH  CX                            ; save loop counter on the stack
    MOV   CX,0x000B                     ; eleven character name
    MOV   SI,Stage2Name                 ; Stage2 file name to find
    PUSH  DI
    REP   CMPSB                         ; test for entry match
    POP   DI
    JE    LoadFat                       ; found our file, now load it
    POP   CX                            ; pop our loop counter
    ADD   DI,0x0020                     ; queue next directory entry
    LOOP  FindFat                       ; keep looking
    JMP   FindFatFailed                 ; file not found, this is bad!

    ;--------------------------
    ; Load FAT
    ; Tutorial 6: Bootloaders 4
    ;--------------------------
LoadFat:
    ; save starting cluster of boot image
    MOV   DX,WORD [DI + 0x001A]         ; save file's
    MOV   WORD [Cluster],DX             ;  first cluster

    ; compute size of FAT and store in "cx"
    XOR   AX,AX
    MOV   AL,BYTE [NumberOfFATs]        ; number of FATs
    MUL   WORD [SectorsPerFAT]          ; sectors used by FATs
    MOV   CX,AX

    ; compute location of FAT and store in "ax"
    MOV   AX,WORD [ReservedSectors]     ; adjust for bootsector

    ; read FAT into memory (7C00:0200)
    MOV   BX,0x0200                     ; read FAT
    CALL  ReadSector                    ;  into memory above our bootcode

;--------------------------------------------------------------------------------------------------
; Load Stage 2
; Tutorial 6: Bootloaders 4
;--------------------------------------------------------------------------------------------------
    ; read Stage2 file into memory (0050:0000)
    MOV   AX,0x0050                     ; set segment register
    MOV   ES,AX                         ;  to 50h
    MOV   BX,0x0000                     ; push our starting address (0h)
    PUSH  BX                            ;  onto the stack

LoadStage2:
    MOV   AX,WORD [Cluster]             ; cluster to read
    POP   BX                            ; buffer to read into
    CALL  ClusterLBA                    ; convert cluster to LBA
    XOR   CX,CX                         ; CL =
    MOV   CL,BYTE [SectorsPerCluster]   ;  sectors to read
    CALL  ReadSector                    ; read a sector
    PUSH  BX                            ; push buffer ptr to stack

    ; compute next cluster
    MOV   AX,WORD [Cluster]             ; identify current cluster
    MOV   CX,AX                         ; copy current cluster
    MOV   DX,AX                         ; copy current cluster
    SHR   DX,0x0001                     ; divide by two
    ADD   CX,DX                         ; sum for (3/2)
    MOV   BX,0x0200                     ; location of FAT in memory
    ADD   BX,CX                         ; index into FAT
    MOV   DX,WORD [BX]                  ; read two bytes from FAT, indexed by BX
    TEST  AX,0x0001                     ; test under mask, if cluster number is odd
    JNZ   LoadStage2OddCluster          ;  then process Odd Cluster

LoadStage2EvenCluster:
    AND   DX,0000111111111111b          ; take low  twelve bits DX x'ABCD' -> x'0BCD'
    JMP   LoadStage2CheckDone

LoadStage2OddCluster:
    SHR   DX,0x0004                     ; take high twelve bits DX x'ABCD' -> x'0ABC'

LoadStage2CheckDone:
    MOV   WORD [Cluster],DX             ; store new cluster
    CMP   DX,0x0FF0                     ; If DX is less than EOF (0x0FF0)
    JB    LoadStage2                    ;   then keep going (JB = Jump Below)

;--------------------------------------------------------------------------------------------------
; Jump to Stage 2 code
;--------------------------------------------------------------------------------------------------
    MOV   SI,Stage2Msg                  ; si points to first byte in message
    CALL  Print                         ; print message
    MOV   AH,0X00                       ; wait
    INT   0x16                          ;  for keypress
    MOV   SI,NewLineMsg                 ; print
    CALL  Print                         ;  new line
    PUSH  WORD 0x0050                   ; Jump to our Stage2 code that we put at 0050:0000
    PUSH  WORD 0x0000                   ;   by using a Far Return which pops IP(0h) then CS(50h)
    RETF                                ;   and poof, we're executing our Stage2 code!

;--------------------------------------------------------------------------------------------------
; Failed to find FAT (File Allocation Table)
;--------------------------------------------------------------------------------------------------
FindFatFailed:
    MOV   SI,FailureMsg                 ; print
    CALL  Print                         ;  failure message
    MOV   AH,0x00                       ; wait for
    INT   0x16                          ;  keypress
    INT   0x19                          ; warm boot computer

;--------------------------------------------------------------------------------------------------
; Working Storage
;--------------------------------------------------------------------------------------------------
    AbsoluteHead    DB 0x00
    AbsoluteSector  DB 0x00
    AbsoluteTrack   DB 0x00
    Cluster         DW 0x0000
    DataSector      DW 0x0000
    FailureMsg      DB 0x0D, 0x0A, "MISSING OR CURRUPT STAGE2", 0x0D, 0x0A, 0x00
    LoadingMsg      DB 0x0D, 0x0A, "MyOs v0.1.1 Stage 1", 0x00
    NewLineMsg      DB 0x0D, 0x0A, 0x00
    ProgressMsg     DB ".", 0x00
    Stage2Msg       DB 0x0D, 0x0A, " Hit Enter to Jump to Stage 2 ", 0x00
    Stage2Name      DB "STAGE2  BIN"

;--------------------------------------------------------------------------------------------------
; Make it a Boot Sector! (must be exactly 512 bytes)
;--------------------------------------------------------------------------------------------------
    TIMES 510-($-$$) DB 0               ; make boot sector exactly 512 bytes
    DW 0xAA55                           ; Magic Word that makes this a boot sector
</source>

== Stage 2 ==
Main task is to find and load the Kernel.
<source lang="asm">
;**********************************************************
; Stage2.asm
;   Stage2 Bootloader
;
; Broken Thorn Entertainment
; Operating Systems Development Tutorial
; http://www.brokenthorn.com/Resources/OSDevIndex.html
;
; nasm -f bin Stage2.asm -o Stage2.bin -l Stage2.lst
;**********************************************************

; Remember the memory map-- 500h through 7BFFh is unused above the BIOS data area.
; We are loaded at 500h (50h:0h)
[bits 16]
    ORG   0500h
    JMP   Main                          ; jump to Main

;--------------------------------------------------------------------------------------------------
; Prints a null terminated string
; DS => SI: 0 terminated string
;--------------------------------------------------------------------------------------------------
[bits 16]
PutStr:
    PUSHA                               ; save registers
    MOV   AH,0Eh                        ; Nope-Print the character
PutStr1:
    LODSB                               ; load next byte from string from SI to AL
    OR    AL,AL                         ; Does AL=0?
    JZ    PutStr2                       ; Yep, null terminator found-bail out
    INT   10h                           ; invoke BIOS
    JMP   PutStr1                       ; Repeat until null terminator found
PutStr2:
    POPA                                ; restore registers
    RET                                 ; we are done, so return

;--------------------------------------------------------------------------------------------------
; Install our GDT
; Tutorial 8: Protected Mode
;--------------------------------------------------------------------------------------------------
[bits 16]
InstallGDT:
    CLI                                 ; disable interrupts
    PUSHA                               ; save registers
    LGDT  [GDT2]                        ; load GDT into GDTR
    STI                                 ; enable interrupts
    POPA                                ; restore registers
    RET                                 ; All done!

;--------------------------------------------------------------------------------------------------
; Enable A20 line through output port
;--------------------------------------------------------------------------------------------------
[bits 16]
EnableA20:
    CLI                                 ; disable interrupts
    PUSHA

    CALL  WaitInput                     ; wait for keypress
    MOV   AL,0ADh
    OUT   64h,AL                        ; disable keyboard
    CALL  WaitInput                    

    MOV   AL,0D0h
    OUT   64h,AL                        ; tell controller to read output port
    CALL  WaitOutput                   

    IN    AL,60h
    PUSH  EAX                           ; get output port data and store it
    CALL  WaitInput                    

    MOV   AL,0D1h
    OUT   64h,AL                        ; tell controller to write output port
    CALL  WaitInput                    

    POP   EAX
    OR    AL,2                          ; set bit 1 (enable a20)
    OUT   60h,AL                        ; write out data back to the output port

    CALL  WaitInput                    
    MOV   AL,0AEh                       ; enable keyboard
    OUT   64h,AL

    CALL  WaitInput                     ; wait for keypress
    POPA
    STI                                 ; enable interrupts
    RET

;------------------------------
; Helper routines for EnableA20
;------------------------------
WaitInput:
    IN    AL,64h                        ; wait for
    TEST  AL,2                          ;  input buffer
    JNZ   WaitInput                     ;   to clear
    RET

WaitOutput:
    IN    AL,64h                        ; wait for
    TEST  AL,1                          ;  output buffer
    JZ    WaitOutput                    ;   to clear
    RET

;--------------------------------------------------------------------------------------------------
; Floppy Driver Routines
;--------------------------------------------------------------------------------------------------
;------------------------------------------
; Convert CHS to LBA
; LBA = (cluster - 2) * sectors per cluster
;------------------------------------------
[bits 16]
ClusterLBA:
    SUB   AX,0002h                      ; zero base cluster number
    XOR   CX,CX
    MOV   CL,BYTE [SectorsPerCluster]   ; convert byte to word
    MUL   CX
    ADD   AX,WORD [DataSector]          ; base data sector
    RET

;---------------------------------------------------------------------------
; Convert LBA to CHS
; AX = LBA Address to convert
;
; absolute sector = (logical sector / sectors per track) + 1
; absolute head   = (logical sector / sectors per track) MOD number of heads
; absolute track  = logical sector / (sectors per track * number of heads)
;---------------------------------------------------------------------------
[bits 16]
LBACHS:                                 ;
    XOR   DX,DX                         ; prepare dx:ax for operation
    DIV   WORD [SectorsPerTrack]        ; calculate
    INC   DL                            ; adjust for sector 0
    MOV   BYTE [AbsoluteSector],DL
    XOR   DX,DX                         ; prepare dx:ax for operation
    DIV   WORD [HeadsPerCylinder]       ; calculate
    MOV   BYTE [AbsoluteHead],DL
    MOV   BYTE [AbsoluteTrack],AL
    RET

;-----------------------------------
; Read a series of sectors
; CX     = Number of sectors to read
; AX     = Starting sector
; ES:EBX = Buffer
;-----------------------------------
[bits 16]
ReadSector:
    MOV   DI,0005h                      ; five retries for error
ReadSector1:
    PUSH  AX
    PUSH  BX
    PUSH  CX
    CALL  LBACHS                        ; convert starting sector to CHS
    MOV   AH,02h                        ; BIOS read sector
    MOV   AL,01h                        ; read one sector
    MOV   CH,BYTE [AbsoluteTrack]       ; track
    MOV   CL,BYTE [AbsoluteSector]      ; sector
    MOV   DH,BYTE [AbsoluteHead]        ; head
    MOV   DL,BYTE [DriveNumber]         ; drive
    INT   13h                           ; invoke BIOS
    JNC   ReadSector2                   ; test for read error
    XOR   AX,AX                         ; BIOS reset disk
    int   13h                           ; invoke BIOS
    DEC   DI                            ; decrement error counter
    POP   CX
    POP   BX
    POP   AX
    JNZ   ReadSector1                   ; attempt to read again
    INT   18h
ReadSector2:
    POP   CX
    POP   BX
    POP   AX
    ADD   BX,WORD [BytesPerSector]      ; queue next buffer
    INC   AX                            ; queue next sector
    LOOP  ReadSector                    ; read next sector
    RET

;------------------------------------
; Load Root Directory Table to 07E00h
;------------------------------------
[bits 16]
LoadRootDir:
    PUSHA                               ; store registers
    PUSH  ES
    ; compute size of root directory and store in "CX"
    XOR   CX,CX                         ; clear registers
    XOR   DX,DX
    MOV   AX,32                         ; 32 byte directory entry
    MUL   WORD [RootEntries]            ; total size of directory
    DIV   WORD [BytesPerSector]         ; sectors used by directory
    XCHG  AX,CX                         ; move into AX
    ; compute location of root directory and store in "AX"
    MOV   AL,BYTE [NumberOfFATs]        ; number of FATs
    MUL   WORD [SectorsPerFAT]          ; sectors used by FATs
    ADD   AX,WORD [ReservedSectors]
    MOV   WORD [DataSector],AX          ; base of root directory
    ADD   WORD [DataSector],CX
    ; read root directory into 07E00h
    PUSH  WORD RootSegment
    POP   ES
    MOV   BX,0                          ; copy root dir
    CALL  ReadSector                    ; read in directory table
    POP   ES
    POPA                                ; restore registers and return
    RET

;-----------------------------
; Loads FAT table to 07C00h
; ES:DI = Root Directory Table
;-----------------------------
[bits 16]
LoadFAT:
    PUSHA                               ; store registers
    PUSH  ES
    ; compute size of FAT and store in "CX"
    XOR   AX,AX
    MOV   AL,BYTE [NumberOfFATs]        ; number of FATs
    MUL   WORD [SectorsPerFAT]          ; sectors used by FATs
    MOV   CX,AX
    ; compute location of FAT and store in "AX"
    MOV   AX,WORD [ReservedSectors]
    ; read FAT into memory (Overwrite our bootloader at 07C00h)
    PUSH  WORD FatSegment
    POP   ES
    XOR   BX,BX
    CALL  ReadSector 
    POP   ES
    POPA                                ; restore registers and return
    RET

;----------------------------------------------------------------
; Search for filename in root table
; parm DS:SI = File name
; ret  AX    = File index number in directory table. -1 if error
;----------------------------------------------------------------
[bits 16]
FindFile:
    PUSH  CX                            ; store registers
    PUSH  DX
    PUSH  BX
    MOV   BX,SI                         ; copy filename for later
    ; browse root directory for binary image
    MOV   CX,WORD [RootEntries]         ; load loop counter
    MOV   DI,RootOffset                 ; locate first root entry at 1 MB mark
    CLD                                 ; clear direction flag
FindFile1:
    PUSH  CX
    MOV   CX,11                         ; eleven character name. Image name is in SI
    MOV   SI,BX                         ; image name is in BX
    PUSH  DI
    REP   CMPSB                         ; test for entry match
    POP   DI
    JE    FindFile2
    POP   CX
    ADD   DI,32                         ; queue next directory entry
    LOOP  FindFile1
    ; Not Found
    POP   BX                            ; restore registers and return
    POP   DX
    POP   CX
    MOV   AX,-1                         ; set error code
    RET
FindFile2:
    POP   AX                            ; return value into AX contains entry of file
    POP   BX                            ; restore registers and return
    POP   DX
    POP   CX
    RET

;-----------------------------------------
; Load file
; parm ES:SI  = File to load
; parm EBX:BP = Buffer to load file to
; ret  AX     = -1 on error, 0 on success
; ret  CX     = number of sectors read
;-----------------------------------------
[bits 16]
LoadFile:
    XOR   ECX,ECX                       ; size of file in sectors
    PUSH  ECX
    PUSH  BX                            ; BX => BP points to buffer to write to; store it for later
    PUSH  BP
    CALL  FindFile                      ; find our file. ES:SI contains our filename
    CMP   AX,-1
    JNE   LoadFile1
    ; failed to find file
    POP   BP
    POP   BX
    POP   ECX
    MOV   AX,-1
    RET
LoadFile1:
    SUB   EDI,RootOffset
    SUB   EAX,RootOffset
    ; get starting cluster
    PUSH  WORD RootSegment              ; root segment loc
    POP   ES
    MOV   DX,WORD [ES:DI + 0001Ah]      ; DI points to file entry in root directory table. Refrence the table...
    MOV   WORD [Cluster],DX             ; file's first cluster
    POP   BX                            ; get location to write to so we dont screw up the stack
    POP   ES
    PUSH  BX                            ; store location for later again
    PUSH  ES
    CALL  LoadFAT
LoadFile2:
    ; load the cluster
    MOV   AX,WORD [Cluster]             ; cluster to read
    POP   ES                            ; bx:bp=es:bx
    POP   BX
    CALL  ClusterLBA
    XOR   CX,CX
    MOV   CL,BYTE [SectorsPerCluster]
    CALL  ReadSector 
    POP   ECX
    INC   ECX                           ; add one more sector to counter
    PUSH  ECX
    PUSH  BX
    PUSH  ES
    MOV   AX,FatSegment                 ;start reading from fat
    MOV   ES,AX
    XOR   BX,BX
    ; get next cluster
    MOV   AX,WORD [Cluster]             ; identify current cluster
    MOV   CX,AX                         ; copy current cluster
    MOV   DX,AX
    SHR   DX,0001h                      ; divide by two
    ADD   CX,DX                         ; sum for (3/2)
    MOV   BX,0                          ; location of fat in memory
    ADD   BX,CX
    MOV   DX,WORD [ES:BX]
    TEST  AX,0001h                      ; test for odd or even cluster
    JNZ   LoadFile3
    AND   DX,0000111111111111b          ; Even cluster - take low 12 bits
    JMP   LoadFile4
LoadFile3:
    SHR   DX,0004h                      ; Odd cluster  - take high 12 bits
LoadFile4:
    MOV   WORD [Cluster],DX
    CMP   DX,0FF0h                      ; test for end of file marker
    JB    LoadFile2
    ; We're done
    POP   ES
    POP   BX
    POP   ECX
    XOR   AX,AX
    RET

;--------------------------------------------------------------------------------------------------
; Stage 2 Entry Point
; - Set Data segment registers and stack
; - Install GDT
; - Enable A20
; - Read Stage3 into memory
; - Protected mode (pmode)
;--------------------------------------------------------------------------------------------------
[bits 16]
Main:
    ;----------------------------
    ; Set Data Segement registers
    ;----------------------------
    CLI                                 ; disable interrupts
    XOR   AX,AX                         ; null segments
    MOV   DS,AX
    MOV   ES,AX

    ;-----------------
    ; Set up our Stack
    ;-----------------
    MOV   AX,00h                        ; stack begins at 09000h-0FFFFh
    MOV   SS,AX
    MOV   SP,0FFFFh
    STI                                 ; enable interrupts

    ;----------------
    ; Install our GDT
    ;----------------
    CALL  InstallGDT

    ;-----------
    ; Enable A20
    ;-----------
    CALL  EnableA20

    ;----------------------
    ; Print loading message
    ;----------------------
    MOV   SI,LoadingMsg
    CALL  PutStr

    ;----------------------
    ; Initialize filesystem
    ;----------------------
    CALL  LoadRootDir                   ; Load root directory table

    ;----------------------
    ; Read Stage3 from disk
    ;----------------------
    MOV   EBX,0                         ; BX:BP points to buffer to load to
    MOV   BP,RModeBase
    MOV   SI,Stage3Name                 ; our file to load
    CALL  LoadFile
    MOV   DWORD [Stage3Size],ECX        ; save the size of Stage3
    CMP   AX,0                          ; Test for success
    JE    GoProtected                   ; yep--onto Stage 3!

    ;------------------
    ; This is very bad!
    ;------------------
    MOV   SI,FailureMsg                 ; Nope--print error
    CALL  PutStr                        ;
    MOV   AH,0                          ; wait
    INT   16h                           ;  for keypress
    INT   19h                           ; warm boot computer
    CLI                                 ; If we get here, something really went wrong
    HLT

GoProtected:
    MOV   SI,Stage3Msg
    CALL  PutStr
    MOV   ah,00h                        ; wait
    INT   16h                           ;  for keypress
    ;--------------
    ; Go into pmode
    ;--------------
    CLI                                 ; clear interrupts
    MOV   EAX,CR0                       ; set bit 0 in cr0--enter pmode
    OR    EAX,1
    MOV   CR0,EAX
    JMP   CodeDesc:GoStage3             ; far jump to fix CS. Remember that the code selector is 08h!

  ; Note: Do NOT re-enable interrupts! Doing so will triple fault!
  ; We will fix this in Stage 3.

;--------------------------------------------------------------------------------------------------
; Get to Stage3 - Our Kernel!
; - Set Data Segment Register
; - Set up our Stack
; - Copy Kernel to address 1 MB
; - Jump to our Kernel!!
;--------------------------------------------------------------------------------------------------
[bits 32]
GoStage3:
    ;----------------------------
    ; Set Data Segement registers
    ;----------------------------
    MOV   AX,DataDesc                   ; set data segments to data selector (10h)
    MOV   DS,AX
    MOV   SS,AX
    MOV   ES,AX

    ;-----------------
    ; Set up our Stack
    ;-----------------
    MOV   ESP,90000h                    ; stack begins from 90000h

    ;-------------------
    ; Copy kernel to 1MB
    ;-------------------
    MOV   EAX,DWORD [Stage3Size]
    MOVZX EBX,WORD [BytesPerSector]
    MUL   EBX
    MOV   EBX,4
    DIV   EBX
    CLD
    MOV   ESI,RModeBase
    MOV   EDI,PModeBase
    MOV   ECX,EAX
    REP   MOVSD                         ; copy image to its protected mode address

    ;--------------------
    ; Jump to our Kernel!
    ;--------------------
    JMP   CodeDesc:PModeBase            ; jump to our kernel! Note: This assumes Kernel's entry point is at 1 MB

    ;-------------------
    ; We never get here! 
    ;-------------------
    CLI                                 ; Stop 
    HLT                                 ;  execution

;--------------------------------------------------------------------------------------------------
; Global Descriptor Table (GDT)
; Tutorial 8: Protected Mode
;--------------------------------------------------------------------------------------------------
GDT1:
;----------------
; null descriptor
;----------------
                  DD  0
                  DD  0
NullDesc          EQU 0
;----------------
; code descriptor
;----------------
                  DW  0FFFFh            ; limit low
                  DW  0                 ; base low
                  DB  0                 ; base middle
                  DB  10011010b         ; access
                  DB  11001111b         ; granularity
                  DB  0                 ; base high
CodeDesc          EQU 8h
;----------------
; data descriptor
;----------------
                  DW  0FFFFh            ; limit low
                  DW  0                 ; base low
                  DB  0                 ; base middle
                  DB  10010010b         ; access
                  DB  11001111b         ; granularity
                  DB  0                 ; base high
DataDesc          EQU 10h
;-------------------
; pointer to our GDT
;-------------------
GDT2:
                  DW  GDT2-GDT1-1       ; limit (Size of GDT)
                  DD  GDT1              ; base of GDT

;--------------------------------------------------------------------------------------------------
; Working Storage
;--------------------------------------------------------------------------------------------------
FatSegment        EQU 2C0h
PModeBase         EQU 100000h           ; where the kernel is to be loaded to in protected mode
RModeBase         EQU 3000h             ; where the kernel is to be loaded to in real mode
RootOffset        EQU 2E00h
RootSegment       EQU 2E0h

LoadingMsg        DB  0Dh
                  DB  0Ah
                  DB  "MyOs v0.1.1 Stage 2"
                  DB  00h

Stage3Msg         DB  0Dh
                  DB  0Ah
                  DB  " Hit Enter to Jump to Stage 3"
                  DB  00h

FailureMsg        DB  0Dh
                  DB  0Ah
                  DB  "*** FATAL: MISSING OR CURRUPT STAGE3.BIN. Press Any Key to Reboot"
                  DB  0Dh
                  DB  0Ah
                  DB  0Ah
                  DB  00h


AbsoluteHead      DB  00h
AbsoluteSector    DB  00h
AbsoluteTrack     DB  00h
BytesPerSector    DW  512
Cluster           DW  0000h
DataSector        DW  0000h
DriveNumber       DB  0
HeadsPerCylinder  DW  2
Stage3Name        DB  "STAGE3  BIN"      ; kernel name (Must be 11 bytes)
Stage3Size        DB  0                  ; size of kernel image in bytes
NumberOfFATs      DB  2
ReservedSectors   DW  1
RootEntries       DW  224
SectorsPerCluster DB  1
SectorsPerFAT     DW  9
SectorsPerTrack   DW  18
</source>

== Stage 3 ==
The Kernel

Includes some primitive video code and gets a single scan code from the keyboard.
<source lang="asm">
;**********************************************************
; Stage3.asm
;   A basic 32 bit binary kernel running
;
; Broken Thorn Entertainment
; Operating Systems Development Tutorial
; http://www.brokenthorn.com/Resources/OSDevIndex.html
;
; nasm -f bin Stage3.asm -o Stage3.bin -l Stage3.lst
;**********************************************************

[bits  32]                              ; 32 bit code
    ORG   100000h                       ; Kernel starts at 1 MB
    JMP   Stage3                        ; Jump to entry point

;--------------------------------------------------------------------------------------------------
; Video Routines
;--------------------------------------------------------------------------------------------------

;---------------
;- Color Codes -
;---------------
;  0 0 Black
;  1 1 Blue
;  2 2 Green
;  3 3 Cyan
;  4 4 Red
;  5 5 Purple
;  6 6 Brown
;  7 7 Gray
;  8 8 Dark Gray
;  9 9 Light Blue
; 10 A Light Green
; 11 B Light Cyan
; 12 C Light Red
; 13 D Light Purple
; 14 E Yellow
; 15 F White
; Example 3F
;         ^^
;         ||
;         ||- Foreground F = White
;         |-- Background 3 = Cyan

;------------------------------------------
; Routine to calculate video memory address
;   represented by the given Row,Col
;------------------------------------------
CalcVideoAddr:
    PUSHA                               ; Save registers
    XOR   EAX,EAX                       ; Row calculation
    MOV   AL,[Row]                      ;  row
    DEC   EAX                           ;  minus 1
    MOV   EDX,160                       ;  times
    MUL   EDX                           ;  160
    PUSH  EAX                           ;  save it
    XOR   EAX,EAX                       ; Col calculation
    MOV   AL,[Col]                      ;  col
    MOV   EDX,2                         ;  times
    MUL   EDX                           ;  2
    SUB   EAX,EDX                       ;  minus 2
    POP   EDX                           ; Add col calculation
    ADD   EAX,EDX                       ;  to row calculation
    ADD   EAX,VidMem                    ;  plus VidMem
    MOV   [VidAdr],EAX                  ; Save in VidAdr
    POPA                                ; Restore registers
    RET                                 ; Return to caller

;------------------------------
; Put a character on the screen
; EDI = address in video memory
;------------------------------
PutChar:
    PUSHA                               ; Save registers
    MOV   EDI,[VidAdr]                  ; EDI = Video Address
    MOV   DL,[Char]                     ; DL = character
    MOV   DH,[ColorAttr]                ; DH = attribute
    MOV   WORD [EDI],DX                 ; Move attribute and character to video display
    POPA                                ; Restore registers
    RET                                 ; Return to caller

;---------------------------------
; Print a null terminated string
; EBX = address of string to print
;---------------------------------
PutStr:
    PUSHA                               ; Save registers
    CALL  CalcVideoAddr                 ; Calculate video address
    XOR   ECX,ECX                       ; Clear ECX
    PUSH  EBX                           ; Copy the string address in EBX
    POP   ESI                           ;  to ESI
    MOV   CX,[ESI]                      ; Grab string length using ESI, stuff it into CX
    SUB   CX,2                          ; Subtract out 2 bytes for the length field
    ADD   ESI,2                         ; Bump past the length field to the beginning of string
PutStr1:
    MOV   BL,BYTE [ESI]                 ; Get next character
    CMP   BL,0Ah                        ; NewLine?
    JNE   PutStr2                       ;  No
    MOV   BYTE [Col],1                  ;  Yes, back to col 1
    INC   BYTE [Row]                    ;   and bump row by 1
    CALL  CalcVideoAddr                 ; Calculate video address
    JMP   PutStr3                       ; Continue
PutStr2:
    MOV   [Char],BL                     ; Stash our character
    CALL  PutChar                       ; Print it out
    ADD   DWORD [VidAdr],2              ; Bump Video Address by 2
    INC   BYTE [Col]                    ; Bump column by 1
PutStr3:
    INC   ESI                           ; Bump ESI to next character in our string
    LOOP  PutStr1                       ; Loop (Decrement CX each time until CX is zero)
    CALL  MoveCursor                    ; Update cursor (do this once after displaying the string, more efficient)
    POPA                                ; Restore registers
    RET                                 ; Return to caller

;-----------------------
; Update hardware cursor
;-----------------------
MoveCursor:
    PUSHA                               ; Save registers
    MOV   BH,BYTE [Row]                 ; BH = row
    MOV   BL,BYTE [Col]                 ; BL = col
    DEC   BH                            ; BH-- (Make row zero based)

    XOR   EAX,EAX                       ; Clear EAX
    MOV   ECX,TotCol                    ; ECX = TotCol
    MOV   AL,BH                         ; Row
    MUL   ECX                           ;  * TotCol
    ADD   AL,BL                         ;  + Col
    MOV   EBX,EAX                       ; Save result in EBX (BL,BH in particular)

    XOR   EAX,EAX                       ; Clear EAX
    MOV   DX,03D4h                      ; Set VGA port to  03D4h (Video controller register select)
    MOV   AL,0Fh                        ; Set VGA port-index 0Fh (cursor location low byte)
    OUT   DX,AL                         ; Write to the VGA port
    MOV   DX,03D5h                      ; Set VGA port to  03D5h (Video controller data)
    MOV   AL,BL                         ; Set low byte of calculated cursor position from above
    OUT   DX,AL                         ; Write to the VGA port

    XOR   EAX,EAX                       ; Clear EAX
    MOV   DX,03D4h                      ; Set VGA port to  03D4h (Video controller register select)
    MOV   AL,0Eh                        ; Set VGA port-index 0Fh (cursor location high byte)
    OUT   DX,AL                         ; Write to the VGA port
    MOV   DX,03D5h                      ; Set VGA port to  03D5h (Video controller data)
    MOV   AL,BH                         ; Set high byte of calculated cursor position from above
    OUT   DX,AL                         ; Write to the VGA port

    POPA                                ; Restore registers
    RET                                 ; Return to caller

;-------------
; Clear Screen
;-------------
ClrScr:
    PUSHA                               ; Save registers
    CLD                                 ; Clear DF Flag, REP STOSW increments EDI
    MOV   EDI,VidMem                    ; Set EDI to beginning of Video Memory
    MOV   CX,2000                       ; 2,000 'words' on the screen
    MOV   AH,[ColorAttr]                ; Set color attribute
    MOV   AL,' '                        ; We're going to 'blank' out the screen
    REP   STOSW                         ; Move AX to video memory pointed to by EDI, Repeat CX times, increment EDI each time
    MOV   BYTE [Col],1                  ; Set Col to 1
    MOV   BYTE [Row],1                  ; Set Row to 1
    POPA                                ; Restore registers
    RET                                 ; Return to caller

;-------------------
;Set Color Attribute
;-------------------
SetColorAttr:
    PUSHA                               ; Save registers
    MOV   AL,[ColorBack]                ; Background color (e.g. 3)
    SHL   AL,4                          ;  goes in highest 4 bits of AL
    MOV   BL,[ColorFore]                ; Foreground color in lowest 4 bits of BL (e.g. F)
    OR    EAX,EBX                       ; AL now has the combination of background and foreground (e.g. 3F)
    MOV   [ColorAttr],AL                ; Save result in ColorAttr
    POPA                                ; Restore registers
    RET                                 ; Return to caller

;--------------------------------------------------------------------------------------------------
; Install our IDT
;--------------------------------------------------------------------------------------------------
InstallIDT:
    CLI                                 ; Disable interrupts
    PUSHA                               ; Save registers
    LIDT  [IDT2]                        ; Load IDT into IDTR
    MOV   EDI,IDT1                      ; Set EDI to beginning of IDT
    MOV   CX,2048                       ; 2048 bytes in IDT
    XOR   EAX,EAX                       ; Set all 256 IDT entries to NULL (0h)
    REP   STOSB                         ; Move AL to IDT pointed to by EDI, Repeat CX times, increment EDI each time
    STI                                 ; Enable interrupts
    POPA                                ; Restore registers
    RET                                 ; All done!

;--------------------------------------------------------------------------------------------------
; Keyboard Routines
;--------------------------------------------------------------------------------------------------
ReadKeyboard:
    ;--------------
    ; Read scancode from the keyboard buffer and reset the keyboard
    ;--------------
    IN    AL,060h                       ;Obtain scancode form Keyboart I/O Port
    MOV   CL,AL                         ;Store the scancode in CL for now
    IN    AL,061h                       ;Parse the Keyboard Command Port
    MOV   AH,AL                         ;Store command code in AH for now
    OR    AL,080h                       ;Set AL to disable command code
    OUT   061h,AL                       ;Output disable command to Keyboard Command Port
    MOV   AL,AH                         ;Set AL to the original command code
    OUT   061h,AL                       ;Output enable command to Keyboard Command Port
    MOV   AL,CL                         ;Restore the scancode to AL
    RET

;--------------------------------------------------------------------------------------------------
; Stage3 - Our Kernel!
;--------------------------------------------------------------------------------------------------
Stage3:
    ;--------------
    ; Set registers
    ;--------------
    MOV   AX,10h                        ; Set data
    MOV   DS,AX                         ;  segments to
    MOV   SS,AX                         ;   data selector
    MOV   ES,AX                         ;    (10h)
    MOV   ESP,90000h                    ; Stack begins from 90000h

    CALL  InstallIDT                    ; Install our Interrupt Descriptor Table

    ;-------------------------------
    ; Clear screen and print success
    ;-------------------------------
    MOV   BYTE [ColorBack],Black        ; Background color
    MOV   BYTE [ColorFore],Purple       ; Foreground colr
    CALL  SetColorAttr                  ; Set color
    CALL  ClrScr                        ; Clear screen

    MOV   BYTE [Row],10                 ; Row 10
    MOV   BYTE [Col],1                  ; Col 1
    MOV   EBX,Msg1                      ; Put
    CALL  PutStr                        ;  Msg1
    MOV   EBX,NewLine                   ; Put
    CALL  PutStr                        ;  a New Line
    MOV   EBX,Msg2                      ; Put
    CALL  PutStr                        ;  Msg2

    MOV   EBX,NewLine
    CALL  PutStr
    CALL  ReadKeyboard
    MOV   [Char],AL
    CALL  PutChar

    ;---------------
    ; Stop execution
    ;---------------
    CLI
    HLT

;--------------------------------------------------------------------------------------------------
; Interrupt Descriptor Table (IDT)
;--------------------------------------------------------------------------------------------------
IDT1:
TIMES 2048  DB 0                        ; The IDT is exactly 2048 bytes - 256 entries 8 bytes each
;-------------------
; pointer to our IDT
;-------------------
IDT2:
                  DW  IDT2-IDT1-1       ; limit (Size of IDT)
                  DD  IDT1              ; base of IDT

;--------------------------------------------------------------------------------------------------
; Working Storage
;--------------------------------------------------------------------------------------------------
%macro String 2
%1          DW  %%EndStr-%1
            DB  %2
%%EndStr:
%endmacro
String  Msg1,"------   MyOs v0.1.2   -----"
String  Msg2,"------  32 Bit Kernel  -----"
String  NewLine,0Ah

ColorBack   DB  0                       ; Background color (00h - 0Fh)
ColorFore   DB  0                       ; Foreground color (00h - 0Fh)
ColorAttr   DB  0                       ; Combination of background and foreground color (e.g. 3Fh 3=cyan background,F=white text)
Char        DB  0                       ; ASCII character
Row         DB  0                       ; Row (1-25)
Col         DB  0                       ; Col (1-80)
VidAdr      DD  0                       ; Video Address

;--------------------------------------------------------------------------------------------------
; Equates
;--------------------------------------------------------------------------------------------------
VidMem      EQU 0B8000h                 ; Video Memory (Starting Address)
TotCol      EQU 80                      ; width and height of screen
Black       EQU 00h                     ; Black
Cyan        EQU 03h                     ; Cyan
Purple      EQU 05h                     ; Purple
White       EQU 0Fh                     ; White
</source>
````

<a id="w004359"></a>

## W004359 — User talk:Khorlane

Bron-ID: W004359
Namespace: User talk (3)
Revisie: 23324 | Datum: 2019-01-31T11:00:49Z
Bron: https://wiki.osdev.org/User_talk:Khorlane
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AKhorlane&oldid=23324

````text
== Edits ==

Would you mind squashing multiple minor edits into a large one instead of spamming dozens of edits? There's a preview button right next to the one that says 'Save page' to help you out. On an off note, you should probably use a VCS for hosting your code, not your User page. -[[User:No92|no92]] 05:00, 31 January 2019 (CST)
````

<a id="w004360"></a>

## W004360 — User talk:No92

Bron-ID: W004360
Namespace: User talk (3)
Revisie: 23333 | Datum: 2019-01-31T16:20:32Z
Bron: https://wiki.osdev.org/User_talk:No92
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ANo92&oldid=23333

````text
Sorry about spamming the change log will stop doing that for sure. My intent was to turn this into a useful tutorial, so was using my page a place to get started. My code is on GitHub.
````

<a id="w004361"></a>

## W004361 — File:Qword.png

Bron-ID: W004361
Namespace: File (6)
Revisie: 23351 | Datum: 2019-02-08T20:14:09Z
Bron: https://wiki.osdev.org/File:Qword.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AQword.png&oldid=23351

````text
Screenshot of qword running in QEMU
````

<a id="w004362"></a>

## W004362 — SWAPGS

Bron-ID: W004362
Namespace: Main (0)
Revisie: 27990 | Datum: 2023-07-09T17:34:54Z
Bron: https://wiki.osdev.org/SWAPGS
Revisie-URL: https://wiki.osdev.org/index.php?title=SWAPGS&oldid=27990

````text
== FS and GS ==

In x86-64 mode, only the FS and GS registers participate in a limited form of [[segmentation]]; only their base addresses are used to calculate effective addresses. Due in part to the requirement for per-CPU data, as well as the design of AMD's SYSCALL/SYSRET instructions in long mode, the GS register often holds a base address to a structure containing per-CPU data. (Note: the [[System V ABI]] specifies that FS is used for [[Thread Local Storage]], and unless you configure GCC otherwise (possibly by building an [[OS Specific Toolchain]]), GCC will generate code that uses FS for TLS)

(Each CPU core has its own set of MSRs, so they can each have a unique GSBase)

== GSBase, KernelGSBase, and FSBase MSRs ==

Instead of allowing FS and GS to select a 'long' descriptor in the [[GDT]] (similar to the [[TSS]]), AMD (thankfully) created 3 new [[Model Specific Registers]] to tell the CPU what the base address of FS and GS should be. Consequently, your GDT does not need to have any descriptors for FS/GS, and in fact you can load <code>0</code> into the selectors. There is a caveat though; on loading a null selector, AMD CPUs will preserve the "hidden registers", while Intel processors clear the hidden registers.

FSBase is MSR <code>0xC0000100</code>, GSBase is <code>0xC0000101</code>, and KernelGSBase is <code>0xC0000102</code>.

== Loading a Null Selector into FS or GS ==

As detailed above, Intel and AMD processors differ in their behaviour when loading a null selector into FS and GS. On AMD chips, the effective values of FSBase and GSBase are preserved, as detailed in the AMD manual (revision 3.30, section 4.5.3 [page 73]):

<blockquote>
Segment register-load instructions (MOV to Sreg and POP Sreg) load only a 32-bit base-address value
into the hidden portion of the FS and GS segment registers. The base-address bits above the low 32 bits
are cleared to 0 as a result of a segment-register load. <b>When a null selector is loaded into FS or GS, the
contents of the corresponding hidden descriptor register are not altered.</b>
</blockquote>

This behaviour is not documented in the Intel manuals, and from empirical testing the hidden portions are cleared to 0 when a null selector is loaded. (probably)

== SWAPGS ==

To facilitate using GS to store kernel data (its 'original' intention was to be used in conjunction with SYSCALL/SYSRET), a 'SWAPGS' instruction is present in long mode, which swaps the values in the KernelGSBase and GSBase [[Model Specific Registers]]. Since the processor will use the value in the GSBase MSR as the base address of GS, KernelGSBase allows the kernel to 'save' another base address.

The typical use of SWAPGS is to keep the 'user' GS (which likely has a base address of 0) in the GSBase MSR, and the address of the kernel's per-CPU structure in KernelGSBase. Upon entry to Ring 0 (through a system call, software interrupt, or some other method), the kernel will use SWAPGS so accessing <code>[GS:0]</code> will get the pointer to the kernel data. Upon leaving Ring 0, SWAPGS will be called again, to switch GS back to the 'user' GS. For instance:

<source lang="asm">
interrupt_entry:
    swapgs
    push %r15
    push %r14
    ...
    pop %r14
    pop %r15
    swapgs
    iretq
</source>

Then, in kernel code somewhere:
<source lang="asm">
get_cpu_struct:
    mov %gs:0, %rax
    ret
</source>

(Note: LEA does not work with segments, so I use the snippet above, and have the first field of the structure be a pointer to itself (SysV ABI mandates this of the FS pointer to TLS as well)

== Complications ==

A problem should start to become apparent after studying how SWAPGS behaves -- it's not nestable! For instance, if the CPU is in Ring 0 when it is interrupted, then GSBase will already contain the correct pointer; calling SWAPGS at this point would load the user GS, which could cause crashes, or worse. Thus, it is necessary to determine whether the interrupt context was in Ring 0 or Ring 3, and by extension determine whether or not GS needs to be swapped.

When the CPU is interrupted, it helpfully pushes a couple of things to the stack, notably CS. (see page 252 of the AMD manual volume 2 for a helpful diagram). By looking at the value of CS on the stack, one can determine the CPL of the interrupted context. The code snippet above can be modified as such to swap GS when the interrupted context was in Ring 3, and not to swap when it was in Ring 0:

<source lang="asm">
.macro swapgs_if_necessary
	cmp $0x08, 0x8(%rsp)
	je 1f
	swapgs
1:
.endm

interrupt_entry:
    swapgs_if_necessary
    push %r15
    push %r14
    ...
    pop %r14
    pop %r15
    swapgs_if_necessary
    iretq
</source>

As a sidenote, the handlers for SYSCALL/SYSRET can unconditionally perform a SWAPGS, since they will only be called from Ring 3 (and return to Ring 3, respectively). If the kernel is interrupted (by a timer IRQ, for example) while handling the syscall, the snippet above will correctly choose not to swap GS.

== Complications, Part 2 ==

There is a potential race condition in the code above; if the interrupt calling <code>interrupt_entry</code> is itself interrupted (perhaps by a machine check exception, or NMI) -- after the CPU has pushed the interrupt stack frame, but before SWAPGS is called -- the snippet will incorrectly decide '''not''' to swap GS, since the most-recent value of CS on the stack would be that of Ring 0 code.

In this situation, there is another method to determine whether GS needs to be swapped -- by reading the value of GSBase from the MSR. Note that this is a slower process, and it is advisable to only do this method of checking "in an NMI/MCE/DEBUG/whatever super-atomic entry context". (according to Linux)

Note: for this reason, when implementing the entry points to the kernel from userspace (typically syscall handlers), interrupts should be '''disabled'''. (for example, mask IF in MSR_SF_MASK, or use an interrupt gate instead of a call gate for your interrupt handler). Only '''after''' SWAPGS is called should interrupts be enabled (to preserve allow processes in syscalls to be pre-empted). Similarly, interrupts should be disabled before the final SWAPGS to usermode. The relevant CPU mechanisms should re-enable interrupts upon a SYSRET or IRET.

=== External Links ===
* [https://www.kernel.org/doc/Documentation/x86/entry_64.txt What Linux has to say about SWAPGS]

[[Category:X86 CPU]]
[[Category:CPU Registers]]
````

<a id="w004363"></a>

## W004363 — Talk:Virtio

Bron-ID: W004363
Namespace: Talk (1)
Revisie: 23364 | Datum: 2019-02-12T11:07:23Z
Bron: https://wiki.osdev.org/Talk:Virtio
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AVirtio&oldid=23364

````text
== K3achas' edit ==

The C spec only specifies a minimum range for data types like int, so the size of an int really is compiler- and machine-dependent. uint32_t and similar types guarantee a specific size, are easier to understand (it's just as many bits wide as its name states) and, most importantly, my documentation for virtio uses derived types of these. So please, don't use int, long and byte to specify member lengths. If you disagree, feel free to reply. [[User:No92|no92]] 05:07, 12 February 2019 (CST)
````

<a id="w004364"></a>

## W004364 — Universal Plug-n-Play

Bron-ID: W004364
Namespace: Main (0)
Revisie: 27957 | Datum: 2023-07-09T16:13:51Z
Bron: https://wiki.osdev.org/Universal_Plug-n-Play
Revisie-URL: https://wiki.osdev.org/index.php?title=Universal_Plug-n-Play&oldid=27957

````text
Universal Plug-n-Play (UPnP) is a standard set of network protocols defined to allow network devices to discover and retrieve metadata from other devices on the same network.  It uses various network protocols, such as UDP, HTTP and SOAP messages to allow reliable cross-platform communication.

== Device Discovery ==

UPnP devices can be discovered by listening for multicast UDP messages on port 1900.

These messages will contain HTTP-like requests and headers containing URLs that allow clients to further query the device for more information using HTTP GET requests, which return XML documents defining any services, actions or events supported by the device.

An example of a NOTIFY message sent by a device every few minutes may look like:

 NOTIFY * HTTP/1.1
 HOST: 239.255.255.250:1900
 LOCATION: http://100.100.100.100/Device.xml
 NT: upnp:rootdevice
 NTS: ssdp:alive  

== Device Information ==
Further information about the device can be retrieved by sending an HTTP GET request to the URL specified in the LOCATION header.  The HTTP content that is returned will contain information about the device, and any services provided by the device.  Each service has a unique Service Type that can be used to determine the exact functionality supported by the service, as well as a more generic Service ID that can be used as a fallback service classification for compatibility.

Each service listed in this document will have a link (SCPDURL) to a service definition XML file that provides even more metadata about that specific service, as well as a "control" URL that is used to send SOAP messages to control the service, and an "event subscription" URL that is used to subscribe to event notifications.

An example device definition document is shown below:

<source lang="xml">
<root xmlns="urn:schemas-upnp-org:device-1-0" xmlns:pnpx="http://schemas.microsoft.com/windows/pnpx/2005/11" xmlns:df="http://schemas.microsoft.com/windows/2008/09/devicefoundation">
	<specVersion>
		<major>1</major>
		<minor>0</minor>
	</specVersion>
	<device>
		<deviceType>urn:schemas-upnp-org:device:MediaServer:1</deviceType>
		<UDN>uuid:55076f6e-6b79-4d65-6401-00d0b811d10b</UDN>
		<friendlyName>MediaServer</friendlyName>
		<manufacturer>Manufacturer, Inc.</manufacturer>
		<manufacturerURL>http://www.manufacturer.com</manufacturerURL>
		<modelName>MediaServer 1.0</modelName>
		<modelURL>http://www.manufacturer.com/MediaServer</modelURL>
		<modelDescription>Media Server</modelDescription>
		<modelNumber>Media Server Home</modelNumber>
		<serialNumber>123345-50023</serialNumber>
		<UPC>1009234789893</UPC>
		<presentationURL>http://100.100.100.100/</presentationURL>
		<iconList>
			<icon>
				<mimetype>image/jpeg</mimetype>
				<height>48</height>
				<width>48</width>
				<depth>24</depth>
				<url>/images/icon-48x48.jpg</url>
			</icon>
			<icon>
				<mimetype>image/jpeg</mimetype>
				<height>120</height>
				<width>120</width>
				<depth>24</depth>
				<url>/images/icon-120x120.jpg</url>
			</icon>
		</iconList>
		<serviceList>
			<service>
				<serviceType>urn:schemas-upnp-org:service:ConnectionManager:1</serviceType>
				<serviceId>urn:upnp-org:serviceId:ConnectionManager</serviceId>
				<SCPDURL>/ConnectionManager.xml</SCPDURL>
				<eventSubURL>/ConnectionManager/Event</eventSubURL>
				<controlURL>/ConnectionManager/Control</controlURL>
			</service>
			<service>
				<serviceType>urn:schemas-upnp-org:service:ContentDirectory:1</serviceType>
				<serviceId>urn:upnp-org:serviceId:ContentDirectory</serviceId>
				<SCPDURL>/ContentDirectory.xml</SCPDURL>
				<eventSubURL>/ContentDirectory/Event</eventSubURL>
				<controlURL>/ContentDirectory/Control</controlURL>
			</service>
		</serviceList>
	</device>
</root>
</source>

== Service Information ==
Further information concerning a service can be retrieved by sending an HTTP GET request to the URL specified in the SCPDURL field of the service listed in the device definition XML document.  This URL may be a relative URL from the LOCATION field in the NOTIFY message sent by the device, or it may be a full URL if it starts with "http".

The Action List section lists all of the methods that can be invoked on the service, along with the IN and OUT parameters to each method.  The type of each parameter is listed in the Serivce State Table section.

<source lang="xml">
<scpd xmlns="urn:schemas-upnp-org:service-1-0">
	<specVersion>
		<major>1</major>
		<minor>0</minor>
	</specVersion>
	<actionList>
		<action>
			<name>Browse</name>
			<argumentList>
				<argument>
					<name>ObjectID</name>
					<direction>in</direction>
					<relatedStateVariable>A_ARG_TYPE_ObjectID</relatedStateVariable>
				</argument>
				<argument>
					<name>BrowseFlag</name>
					<direction>in</direction>
					<relatedStateVariable>A_ARG_TYPE_BrowseFlag</relatedStateVariable>
				</argument>
				<argument>
					<name>Filter</name>
					<direction>in</direction>
					<relatedStateVariable>A_ARG_TYPE_Filter</relatedStateVariable>
				</argument>
				<argument>
					<name>StartingIndex</name>
					<direction>in</direction>
					<relatedStateVariable>A_ARG_TYPE_Index</relatedStateVariable>
				</argument>
				<argument>
					<name>RequestedCount</name>
					<direction>in</direction>
					<relatedStateVariable>A_ARG_TYPE_Count</relatedStateVariable>
				</argument>
				<argument>
					<name>SortCriteria</name>
					<direction>in</direction>
					<relatedStateVariable>A_ARG_TYPE_SortCriteria</relatedStateVariable>
				</argument>
				<argument>
					<name>Result</name>
					<direction>out</direction>
					<relatedStateVariable>A_ARG_TYPE_Result</relatedStateVariable>
				</argument>
				<argument>
					<name>NumberReturned</name>
					<direction>out</direction>
					<relatedStateVariable>A_ARG_TYPE_Count</relatedStateVariable>
				</argument>
				<argument>
					<name>TotalMatches</name>
					<direction>out</direction>
					<relatedStateVariable>A_ARG_TYPE_Count</relatedStateVariable>
				</argument>
				<argument>
					<name>UpdateID</name>
					<direction>out</direction>
					<relatedStateVariable>A_ARG_TYPE_UpdateID</relatedStateVariable>
				</argument>
			</argumentList>
		</action>
	</actionList>
	<serviceStateTable>
		<stateVariable sendEvents="no">
			<name>A_ARG_TYPE_SortCriteria</name>
			<dataType>string</dataType>
		</stateVariable>
		<stateVariable sendEvents="no">
			<name>A_ARG_TYPE_UpdateID</name>
			<dataType>ui4</dataType>
		</stateVariable>
		<stateVariable sendEvents="no">
			<name>A_ARG_TYPE_SearchCriteria</name>
			<dataType>string</dataType>
		</stateVariable>
		<stateVariable sendEvents="no">
			<name>A_ARG_TYPE_Filter</name>
			<dataType>string</dataType>
		</stateVariable>
		<stateVariable sendEvents="no">
			<name>A_ARG_TYPE_Result</name>
			<dataType>string</dataType>
		</stateVariable>
		<stateVariable sendEvents="no">
			<name>A_ARG_TYPE_Index</name>
			<dataType>ui4</dataType>
		</stateVariable>
		<stateVariable sendEvents="no">
			<name>A_ARG_TYPE_ObjectID</name>
			<dataType>string</dataType>
		</stateVariable>
		<stateVariable sendEvents="no">
			<name>A_ARG_TYPE_Count</name>
			<dataType>ui4</dataType>
		</stateVariable>
		<stateVariable sendEvents="no">
			<name>A_ARG_TYPE_BrowseFlag</name>
			<dataType>string</dataType>
			<allowedValueList>
				<allowedValue>BrowseMetadata</allowedValue>
				<allowedValue>BrowseDirectChildren</allowedValue>
			</allowedValueList>
		</stateVariable>
	</serviceStateTable>
</scpd>
</source>

== Invoking Actions ==
Actions listed in the service definition XML document can be invoked by sending a HTTP POST request containing a SOAP envelope with the desired Action and any arguments defined in the service definition XML document.  The device will perform the requested action and return a SOAP envelope containing any OUT parameter values defined in the service definition XML document.

An example SOAP request is shown below.  The actual path, action and arguments will vary depending on the action being invoked.

<source lang="xml">
POST /Service/Control HTTP/1.0
HOST: 100.100.100.100:9000
CONTENT-LENGTH: 321
CONTENT-TYPE: text/xml; charset="utf-8"
USER-AGENT: OS/1.0 UPnP/2.0 product/1.0
SOAPACTION: "urn:schemas-upnp-org:service:ContentDirectory:1"

<?xml version="1.0"?>

<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
	<s:Body>
		<u:Browse xmlns:u="urn:schemas-upnp-org:service:ContentDirectory:1">
			<ObjectID>0</ObjectID>
			<BrowseFlag>BrowseDirectChildren</BrowseFlag>
			<Filter>*</Filter>
			<StartingIndex>0</StartingIndex>
			<RequestedCount>0</RequestedCount>
			<SortCriteria></SortCriteria>
		</u:Browse>
	</s:Body>
</s:Envelope>
</source>


== External Links ==
[https://openconnectivity.org/developer/specifications/upnp-resources/upnp UPnP Specifications and Architecture Documentation]

[[Category:Network Protocols]]
[[Category:Standards]]
````

<a id="w004365"></a>

## W004365 — File:AsuroTUIWindows.png

Bron-ID: W004365
Namespace: File (6)
Revisie: 23453 | Datum: 2019-03-13T14:51:29Z
Bron: https://wiki.osdev.org/File:AsuroTUIWindows.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AAsuroTUIWindows.png&oldid=23453

````text
Asuro Window Manager.
````

<a id="w004366"></a>

## W004366 — ARM SMMU versions 1 and 2

Bron-ID: W004366
Namespace: Main (0)
Revisie: 30115 | Datum: 2026-03-03T15:12:55Z
Bron: https://wiki.osdev.org/ARM_SMMU_versions_1_and_2
Revisie-URL: https://wiki.osdev.org/index.php?title=ARM_SMMU_versions_1_and_2&oldid=30115

````text
The ARM SMMU specification outlines an [[IOMMU]] architecture for ARM processors, similar to Intel's VT-d. The SMMU mediates accesses to RAM by devices, and can translate addresses generated by DMA engines so as to correct them as the OS requires.

This page is current with versions 1 and 2 of the SMMU specification. This page does not serve as an exhaustive description of the SMMU. It is meant to serve as a refresher to jog the memory of people who have already read the SMMU specification.

A common 's' in a register name, as in SMMU_sCR0 implies that versions of this register exist for both secure and non-secure mode and that both versions are being talked about in that instance.

The SMMU supports processing transactions from multiple streams at once, and translation of multiple contexts at once.

== Terminology: ==
* Just in general, the SMMU spec refers to DMA engines and MCUs on the chipset as being "upstream" from the SMMU and it refers to RAM as being "downstream" from the SMMU.
* The "Stream mapping table" is a term used to refer to the combined working of the SMMU_SMRn + SMMU_S2CRn registers. Do not confuse the SMMU_SMRn ("Stream ''match''ing registers") with the umbrella term "Stream ''mapp''ing registers". ARM defines Stream mapping as (Section 2.3.2) "''the process of mapping a StreamID to the associated Stream-to-Context register, SMMU_S2CRn''".
* "Contexts" are synonymous with Intel's "domain" terminology used in the VT-d specification; they refer to a set of IO-page tables for translating IO-Addresses into output addresses on the bus/memory controller.
* Transactions: Any write to or read from the SMMU by any agent is an SMMU transaction.
** Client transactions: Client transactions are requests sent through the SMMU which can be potentially translated. These are generally bus messages.
** Configuration transactions: Configuration transactions are accesses to the SMMU register interface to program the SMMU's behaviour (using the SMMU_* registers).
* StreamIDs: These are used to identify the source of a transaction. They identify a device uniquely, or otherwise a set of devices working together deliberately to appear to be the same device.
** A single device may also pretend to be multiple devices by using multiple StreamIDs.
** An example of a StreamID might be a PCI Bus+Slot+Function combination.

== Usage model ==

=== Contrast against Intel VT-d ===

Both SMMU and VT-d support 1st-stage translation. 1st-stage translation can be used:
* To isolate devices and ensure that they don't access memory beyond what they are expected to (i.e, beyond the scatter gather list that has currently been programmed into the device).

VT-d 1st-stage translation can be used:
* To isolate guest VMs and ensure that they don't touch memory outside of what they have been allocated as their guest physical address space (IPA addrspace).

Only SMMU supports 2nd-stage translation. 2nd stage translation can be used:
* When passing a device through to a guest VM, to translate the IPA addresses that the guest will program into the device, into their correct PAs.

Only VT-d supports PASID 2nd-stage translation. PASID translation can be used:
* To enable a device to directly walk the page tables of userspace processes, and translate virtual addresses into physical addresses.
** For example, the virtual address of a graphics texture buffer can be sent to the GFX card directly, and the VT-d DHRU will be able to walk the page tables of the sending process and translate the virtual address on its own.
* PASID translation is then chained into the standard translation flow to isolate the device's address accesses in the standard way that an IOMMU is normally used.

VT-d only supports PCI devices in its current form (at the time of writing) while SMMU supports multiple buses but leaves it up to the OEM implementation to determine how to map a bus technology's device IDs onto the identifier nomenclature ("StreamIDs") employed by the SMMU specification. For example, for the SMMU, an OEM is likely to map the 16-bit PCI Bus+Slot+Function onto the 15/16-bit StreamID space provided by the SMMU.

=== Secure/Non-secure mode split and SSD ===
The SMMU splits its registers for use between secure and non-secure mode; both modes can use the SMMU at the same time and there is register banking for some of the registers to facilitate this. Because both modes can use the SMMU at the same time, the SMMU also needs to know which devices have been partitioned into secure-mode devices and which are left for non-secure mode. In particular, secure mode devices can generate client transactions which access both secure memory and non-secure memory whereas non-secure mode devices can only access non-secure memory.

There is a distinction to be kept in mind about the difference between the SMMU's SSD feature (Security-State Determination) and the marking of a transaction by an upstream bus technology/device as either secure or non-secure. A non-secure device may maliciously mark its transactions as being secure, so the SSD feature which sits within the SMMU is meant to thwart that. When an upstream device is determined to be SSD-non-secure, the SMMU will never generate a downstream transaction which is marked secure. When an upstream device is determined to be SSD-secure, the SMMU may generate a downstream transaction which is either secure or non-secure.

* The SMMU may not have a programmable SSD register space, in which case the SSD determination is done via some [implementation defined] mechanism attached to the SMMU.
* The SMMU SSD register space is present if SMMU_IDR1.SSDTP == 0b01 or 0b11.
* When this field has either of those values, the SSD space is present in the SMMU.
** When the SSD space is present in the SMMU, the SMMU_SSDRn registers are present.
** These SMMU_SSDRn registers are bitfields, 32 bits each. If bit<N> is 1 then device<N> is non-secure, else it is secure.

==== Splitting the Stream Mapping table between secure and non-secure mode ====

The Stream Mapping table is global and shared between Secure and non-secure mode. Secure mode must decide how many Stream Mapping register groups it will expose for non-secure mode's use. ARM recommends that secure mode software leaves at least one Stream mapping register group for use by non-secure software.

Having an SMMU_S2CR which has been allocated to secure be configured to mode point to a context bank which is non-secure is UNPREDICTABLE. Having an SMMU_S2CR which has been allocated to non-secure mode be configured to point to a context bank which is secure will trigger an Unimplemented Context Bank SMMU Fault.

==== Splitting the Context banks between secure and non-secure mode ====

Some number of context banks can be reserved for use by secure mode using SMMU_SCR1.NSNUMCBO. The allocation begins from the top of the available context bank space. The SMMU will intelligently present the correct number of context banks to non-secure mode through non-secure accesses to SMMU_IDR1.NUMCB. '' SMMU_SCR1.NSNUMCBO actually reserves a number of context banks for '''non-secure''' mode. The leftover context banks that come '''after''' those reserved by SMMU_SCR1.NSNUMCBO are what are actually reserved for '''secure''' mode. ''.

=== Translation, bypass and faulting ===

The SMMU will do one of 3 things with any client transaction that is sent to it:
* Translate the transaction using translation tables.
* Allow the transaction to bypass translation, and just modify the attributes of the transaction (read/write, shareability, caching, etc).
* Generate an SMMU fault.

== Setup/Initialization related information ==

=== SSD knowledge ===

The SSD register space, if available, tells the OS which devices are secure vs non-secure. For a secure-mode OS, it allows the OS to set which StreamIDs are secure/non-secure. A Secure-mode OS should set these registers up. A non-secure mode OS may choose to read these registers to determine which devices are under its purvue.

Also, secure mode software may need to prevent access by non-secure software to the global register file of the SMMU. This can be done by setting SMMU_SCR1.GASRAE==1 (Section 2.10.1).

=== StreamIDs ===
The number of supported streamID '''bits''' is determined by reading SMMU_IDR0.NUMSIDB[3:0].
* Ranges from 0-15.
* Can be 0 in the case of an SMMU which is dedicated to handling transactions from a single upstream StreamID.

For SMMUv2:
* Support for 16-bit streamIDs exists if SMMU_IDR0.EXIDS==1 AND SMMU_IDR0.NUMSIDB==15.
* 16-bit Stream IDs are enabled by setting SMMU_sCR0.EXIDENABLE==1.

Furthermore, if the OS intends to use Stream Indexing instead of Stream Matching, then the number of StreamIDs is further constrained by the number of SMMU_S2CRn registers available. The number of SMMU_S2CRn registers is directly determined by the number of SMMU_SMRn registers (Section 2.3.2, "Stream indexing": ''This means the maximum StreamID is determined by the number of implemented SMMU_S2CRns''). See [[#Stream Matching Registers]] for how to determine the number of SMMU_S2CRn registers by inferring it from the number of SMMU_SMRn regs.

The OS should determine the max number of StreamIDs based on the Stream Mapping technique it intends to use, and set up metadata accordingly.

=== Stream Matching Registers ===

SMMUv1 supports ''up to'' 128 SMMU_SMRs.
SMMUv2 may optionally support the Extended Stream Matching Extension which provides  ''up to'' 1024 SMMU_SMRs.

The number of SMMU_SMRn regs determines the number of SMMU_S2CRn regs (Section 2.3.2, "Stream matching": ''the optional Extended Stream Matching Extension, that provides up to 1024 Stream Match Registers and associated Stream-to-Context registers'').

* Support for Stream Matching is indicated via SMMU_IDR0.SMS == 1.
* Support for Extended Stream Matching is indicated via SMMU_IDR0.EXSMRGS == 1.
* Enable Extended Stream Matching using SMMU_sCR2.EXSMRGENABLE==1.
** Setting SMMU_SCR2.EXNSSMRGDISABLE==1 with a secure write to SMMU_SCR2 disables the use of Extended Stream Matching by non-secure mode.
* Support for compressed Indexing (which is a subfeature under Stream Matching) is indicated via a secure read of SMMU_IDR2.COMPINDEXS == 1. A non-secure read of this bit only states whether or not compressed indexing is available for use by non-secure software.
* The number of Stream Mapping Register Groups is determined by reading SMMU_IDR0.NUMSMRG.
* It is possible for an implementation not to support Stream '''Matching''', but still allow software to reference the Stream '''Mapping''' Table (which is the '''combination''' of the SMMU_SMRn and SMMU_S2CRn register sets).
** In such a case, the SMMU_SMRn register group will be RES0.

''ARM found it important to mention that if the SMMU support Extended Stream Matching and the extension is not enabled, the (up to) 128 SMMU_SMRs that are exposed, map onto the final 128 SMMU_SMRs provided by the extension. Since it's apparently important, I'll mention it too.''

=== Context Banks and number of supported Contexts ===

The max number of supported contexts is implied by the max number of SMMU_CBARn registers (Section 2.3.4: "The number of SMMU_CBARn regs matches the number of entries in the translation context bank table, and is IMPLEMENTATION DEFINED. SMMU_IDR1.NUMCB specifies the number of implemented SMMU_CBARn regs."). That said, NUMCB is an 8-bit field, and section 2.6.1 says: "The SMMU architecture provides space for up to 128 translation context banks".

* Multiple SMMU_SMRn regs may cause multiple StreamIDs to map to the same Context when Stream Matching is used.
* Each incoming StreamID maps directly to a specific Context when Stream Indexing is used.

Section 2.6.1: '' On reset no translation context bank table entry initializatoin is required. Typically, the SMMU implementation [sic] initializes the required values before use. ''

=== Support for stage1 and stage2 ===

* Support for stage1 translation is indicated via SMMU_IDR0.S1TS.
* Support for stage2 translation is indicated via SMMU_IDR0.S2TS.
* Support for stage1 followed by stage2 translation is indicated via SMMU_IDR0.NTS.
** If SMMU_IDR0.NTS is set, then both SMMU_IDR0.S1TS and SMMU_IDR0.S2TS must be set.

If both stage1 and stage2 are supported, then all contexts except the first X can support either stage1 or stage2 translation. The first X context banks '''may''' be restricted such that they only support stage2. To detect the number X which can only be used for stage2 translation, see SMMU_IDR1.NUMS2CB. The SMMU_CBARn.TYPE field governs the stage of translation for a context bank which supports both stages.

=== Stream Mapping Table ===

Section 2.3.10: "On reset, no initialization is performed on the Stream mapping table entries.". Ergo the OS must explicitly initialize all the stream mapping reg groups "before the SMMU is enabled" [sic].

=== TLBs ===
Section 2.5.2: "In SMMUv2, All TLBs are disabled from reset. All MMUs are disabled from reset and the contexnts of the TLBs have no effect on address translation."

However, an implementation may require an IMPLEMENTATION DEFINED TLB invalidation routine to be called before enabling TLBs/MMU after #RESET. Such a routine must be doucmented clearly by the device. If TLBs which have not been invalidated since #RESET are used their behaviour is UNPREDICTABLE.

== Client transaction flowchart ==

=== Image ===

[[File:Smmu-transaction-flowchart.png]]

=== Synopsis: ===
Do not allow a single StreamID to map to multiple contexts. Multiple stream IDs can however, map to a single context.

* Client transactions are enabled with the "Client Port Disable" bit: SMMU_sCR0.CLIENTPD==0.
* If SMMU_sCR0.CLIENTPD==1, then bypass is assumed for all incoming transactions.
** Bypass attribute transformation is done based on the values in SMMU_sCR0.
* If SMMU_sCR0.CLIENTPD==0, client transaction processing is enabled, and:
** Stream Mapping table is searched.
*** Stream Mapping table search is done using either Stream Matching, Compressed Stream Indexing or Stream Indexing.
** If Stream Mapping search yields ''no results'' then ''either'':
*** [if SMMU_sCR0.USFCFG==1 -- USFCFG means "Unidentified Stream Fault Config"] An Unidentified Stream SMMU fault is generated.
*** [if SMMU_sCR0.USFCFG==0] The unidentifiable stream ID is handled by bypassing it and applying the attribute transformation specified in SMMU_sCR0.
** If Stream mapping table search yields multiple results:
*** [if SMMU_sCR0.SMCFCFG==1 -- SMCFCFG means "Stream Match Conflict Fault Config"] A Stream Match Conflict SMMU Fault is generated.
*** [if SMMU_sCR0.SMCFCFG==0 ] The conflict is handled via bypass and applying the attribute transformation specified in SMMU_sCR0 (just like the Unidentified Stream case).
** If Stream Mapping table search yields a single result ('This is the desired normal case').
*** The Stream Mapping table lookup will identify one of the 128 SMMU_S2CRn Stream to Context Registers.
*** The S2C register will determine which of the 3 handling mechanisms will be used to deal with the incoming client transaction (translation, bypass or fault).
**** [if SMMU_S2CRn.Type == Fault] An Invalid Context SMMU fault is generated.
**** [if SMMU_S2CRn.Type == bypass] The client transaction is handled via bypass and the attributes specified in SMMU_S2CRn are applied to the transaction.
**** [if SMMU_S2CRn.Type == Translation] The client transaction is handled by translation and:
***** The attributes in SMMU_CBARn (stage 1 Context Bank Attribute Reg) and SMMU_SBA2Rn (stage 2 Context Bank Attribute Reg) are 'optionally' applied to the output transaction which is generated downstream.
***** [if SMMU_CBn_SCTLR.M==1 ''for the particular stage in question''] then translation is done for that stage using the configuration in SMMU_CBARn.

=== Flow for Translation-type scenarios ===

[[File:Smmu-stage1-into-stage2-flowchart.png]]

== Stream Mapping ==

As stated before on this page, Stream Mapping is defined by ARM as "'the process of mapping a StreamID to the associated Stream-to-Context register'".

When a client transaction comes in from upstream, if client transaction processing is enabled (SMMU_sCR0.CLIENTPD==0), the SMMU must decide what SMMU context to use to translate the transaction into the output transaction which will eventually be emitted downstream by the SMMU. The process of determining which Context bank should be used to handle client transactions sent by a particular upstream StreamID is an "associative search" [sic] which uses one of 3 methods: Stream Matching, Compressed Stream Indexing or Stream Indexing.

The Stream Mapping table is global and shared between Secure and non-secure mode. Secure mode must decide how many Stream Mapping register groups it will expose for non-secure mode's use. ARM recommends that secure mode software leaves at least one Stream mapping register group for use by non-secure software. To set the number of Stream mapping regs exposed to non-secure mode, use SMMU_SCR1.NSNUMSMRGO. The SMMU will intelligently deduce the number of SMR regs to tell non-secure mode about, and will report the appropriate number to a non-secure reader through SMMU_IDR0.NUMSMRG.

== Transaction attribute combinations ==

See tables 2-2 and 2-3 for information on how cacheability, shareability, INSTCFG (instruction fetch), etc combine to form the final output attributes of the client transaction.

== TLB Management ==

The SMMU maintains its own TLB of translations to speed up client transactions. The topology of the TLBs is IMPLEMENTATION DEFINED. TLB implementations may cache entries for any level or stage (or combination thereof) of translation. TLB maintenance can be done via either: (1) Broadcast TLB operations or (2) TLB maintenance register accesses.

=== TLB tagging ===
Section 2.5: "Software must treat all TLB entries as being tagged with the input address. For other attributes [sic] TLB enteries are categorized so that the tagging applied to an entry depends on the translation regime. For a particular translation regime, a TLB entry must specify 'appropriate information' [sic], and certain conditions [sic] must be satisfied. For example, a non-secure stage 1 TLB entry must specify an ASID."

[[File:Smmu-tlb-tagging.png]]

=== TLB Prefetching behaviour ===

The SMMU is allowed to prefetch the translation tables pointed to by a context bank if that context bank is pointed to by an entry in the Stream Mapping table. This is permitted even if there is no client transaction which is using/ has used that translation (section 2.5.1).

=== TLB Maintenance ===

==== HYPC contexts ====

Section 2.8.2: ''TLB entries that are allocated as a result of HYPC accesses are Hyp tagged. All HYP tagged TLB entries are considered to belong to the same software entity and have no associated ASID or VMID.''

Ergo the SMMU assumes that there is only one piece of software that is acting as a hypervisor.

==== MONC contexts (Section 2.9.2) ====

SMMUv2 provides the following TLB maintenance operations for maintaining MONC TLB tagged entries:
* SMMU_TLBIALLM: Invalidate all EL3 Monitor entries.
* SMMU_TLBIVAM: Invalidates all EL3 Monitor entries which are associated with a 64-bit VA.
* SMMU_TLBIVALM: Same as TLBIVAM, but invalidates only the last level table entries.

ARM found it necessary to warn that: ''These operations are not required to invalidate non-MONC Secure TLB entries''.

==== E2HC contexts (Section 2.10) ====

E2HC context TLB entries are tagged by the TLB as E2H, but have no security state tagging. They are coupled with an ASID if they are non-global.

TLB entries for E2HC contexts matching combinations of (E2H identification) or (E2H identification+ASID tagging) are required to be invalidated by:
* SMMU_TLBIALLH
* SMMU_TLBIVAH
* SMMU_TLBIVAH64
* SMMU_TLBIVALH64
* SMMU_TLBIALLNSNH

E2H identified TLB entries are not required to be invalidated by:
* Secure TLBI operations.
* SMMU_TLBIVMID.
* SMMU_TLBIVMIDS1

Additionally, the standard SMMU_CBn_TLBI* operations work on E2HC context TLB entries, except that instead of matching based on VMID, they match based on (E2H identification) or (E2H identification + ASID tagging).

In particular '''other than the specific TLB invalidate operations bulleted above, E2HC contexts are not considered to be hypervisor contexts by the SMMU'''.

Operation of the TLB is UNPREDICTABLE if there exist TLB entries allocated in the cache identified as HYPC and E2HC concurrently. Software must invalidate the HYPC/E2HC TLBs when switching between operating the SMMU with E2HC vs HYPC support enabled. Please see Section 2.10, example 2-1 for a full listing of the procedure for enabling E2HC operation.

== Context banks ==

Each context bank is one page large, where "one page" is defined as PAGESIZE which may be either 4KiB or 64KiB. The intention is that a hypervisor can
Accessing an unimplemented context bank's register space will either result in an IMPLEMENTATION DEFINED result of either RAZ/WI or a Configuration Access SMMU Fault.

The SMMU_CBA2Rn.MONC, SMMU_CBARn.HYPC and SMMU_CBA2Rn.VA64 bits work together to define the security context and page table format (long vs short) for translation of a particular context bank.

=== Hypervisor Contexts (HYPC) (section 2.8) ===

Hypervisor mode contexts (HYPC) are the page tables of an OS running in HYP mode and not the 2nd stage page tables for a guest IPA. The SMMU is able to walk HYPC page tables. For that reason a HYPC bank cannot be a stage1 followed by stage2 context. I couldn't think of a good reason for a normal hypervisor to prefer to use HYPC tables when it can just use normal IO page tables. '''I could also be misinterpreting the purpose of this feature''', as it appears to be linked to VHE extensions, and it may pertain to a special page table format used by VHE guests.

Support for HYPC is mututally exclusive with support for E2HC. HYPC is supported:
* For both SMMUv1 and SMMUv2, if VHE is not supported, then HYPC is supported.
* In an SMMUv2 implementation that has VHE, if SMMU_CR0.HYPMODE==0, HYPC is available, else E2HC is available.

VHE is supported by the SMMU if:
* SMMU_IDR2.E2HS==1

To set a context bank to be a HYPC bank, set SMMU_CBARn.HYPC==1.

Caveats:
* Do not set the HYPC bit on '''secure''' context banks -- that's illogical. See section 2.8.3 for details.
* The SMMU_CBn_TCR2.SEP field is ignored for HYPC contexts.

=== Monitor Contexts (MONC) (Section 2.9) ===

I couldn't determine whether MONC contexts are meant for use by devices claimed specifically by Monitor Mode (EL3) or whether this bit is meant to be set by all secure mode software. The specific wording of the manual is:
Section 2.9: ''In SMMUv2, '''for a secure bank''', when the value of the corresponding SMMU_CBA2Rn.MONC bit is 1, the bank is a MONC (Secure Monitor) context bank. MONC Translation context banks are intended to be used by the Secure EL3 translation regime.''

The phrasing seems to support the latter of the two interpretations.

Caveats:
* Do not set both HYPC and MONC on any bank because that is UNPREDICTABLE.
* MONC only pertains to contexts reserved for secure mode use. Non-secure contexts ignore and treat the MONC bit as 0. Set it to 0.
* Sign extension via SMMU_CBn_TCR2.SEP is ignored for MONC banks.

=== VHE EL2 Contexts (E2HC) (Section 2.10) ===

E2HC contexts are meant to be used by hypervisor guests running in pseudo-EL2 (ARM's VHE feature) on top of another hypervisor which is running in real-EL2. The SMMU can only be programmed to use either E2HC or HYPC, but not both at the same time. This is a decision which must be made globally and it affects all software using the SMMU whether secure or nonsecure, whether EL3, EL2 or EL1. In fact, the HYPC and E2HC bits are polymorphically at the same position in the register.

E2HC contexts must be programmed to be of SMMU_CBARn.TYPE of 0b01 (stage1 followed by stage2 bypass). Unlike HYPC however, the E2HC translation regime is split between two TTBR registers. See the TLB management section to understand how the TLB entries for E2HC are handled by the TLB.

== IRQs / SMMU Faults ==

=== Context bank faults ===

The SMMU may raise faults for a context bank when processing a stage of translation for a particular context bank.
* For SMMUv1, the SMMU_CBAR register states which interrupt is raised to signal faults while processing that context.
* For SMMUv2, each context bank has its own IRQ pin.

[[Category:ARM Virtualization]]
````

<a id="w004367"></a>

## W004367 — File:Smmu-stage1-into-stage2-flowchart.png

Bron-ID: W004367
Namespace: File (6)
Revisie: 23468 | Datum: 2019-03-28T04:03:57Z
Bron: https://wiki.osdev.org/File:Smmu-stage1-into-stage2-flowchart.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ASmmu-stage1-into-stage2-flowchart.png&oldid=23468

````text
Outline of SMMU stage1 into stage2 translation
````

<a id="w004368"></a>

## W004368 — File:Smmu-transaction-flowchart.png

Bron-ID: W004368
Namespace: File (6)
Revisie: 23469 | Datum: 2019-03-28T04:04:23Z
Bron: https://wiki.osdev.org/File:Smmu-transaction-flowchart.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ASmmu-transaction-flowchart.png&oldid=23469

````text
General SMMU flowchart
````

<a id="w004369"></a>

## W004369 — BCM System Timer

Bron-ID: W004369
Namespace: Main (0)
Revisie: 28167 | Datum: 2023-07-10T14:52:12Z
Bron: https://wiki.osdev.org/BCM_System_Timer
Revisie-URL: https://wiki.osdev.org/index.php?title=BCM_System_Timer&oldid=28167

````text
{{Stub}}

== Introduction ==

The '''BCM System Timer''' is a very simple, easy to use timer found on BCM2835-BCM2837 SoC (all Raspberry Pis, from Zero to 3B+).
This peripheral provides four 32-bit timer channels and a single 64-bit free running counter. Each channel has an output compare register,
which is compared against the 32 least significant bits of the free running counter values. When the two values match, the system timer 
peripheral generates a signal to indicate a match for the appropriate channel. The match signal is then fed into the interrupt controller.
The interrupt service routine then reads the output compare register and adds the appropriate offset for the next timer tick.  
The free running counter is driven by the timer clock and stopped whenever the processor is stopped in debug mode. 

The periperal's address is MMIO_BASE + 0x3000, which is 0x20003000 for older models, and 0x3F003000 on ARM Quad Cores, RPi2 and upwards.

This timer operates on a fixed 1MHz frequency.

== See Also ==
=== External Links ===
* [https://www.raspberrypi.org/app/uploads/2012/02/BCM2835-ARM-Peripherals.pdf ARM Peripherals] Section 12, Page 172

[[Category:ARM]]
[[Category:Timers]]
````

<a id="w004370"></a>

## W004370 — ARM Local Timer

Bron-ID: W004370
Namespace: Main (0)
Revisie: 28113 | Datum: 2023-07-09T22:48:42Z
Bron: https://wiki.osdev.org/ARM_Local_Timer
Revisie-URL: https://wiki.osdev.org/index.php?title=ARM_Local_Timer&oldid=28113

````text
{{Stub}}

== Introduction ==

The '''ARM Local Timer''' is a single, 28 bit countered periodic timer found on Quad Core ARM CPUs (all new Raspberry Pi models, from RPi2 and upwards).

The periperal's address is 0x40000034.

This timer operates on a fixed 38.4MHz frequency.

== See Also ==
=== External Links ===
* [https://www.raspberrypi.org/documentation/hardware/raspberrypi/bcm2836/QA7_rev3.4.pdf ARM Quad A7 Core] Section 4.11, Page 17

[[Category:ARM]]
[[Category:Timers]]
````

<a id="w004371"></a>

## W004371 — ARM Generic Timer

Bron-ID: W004371
Namespace: Main (0)
Revisie: 23793 | Datum: 2019-08-08T03:53:58Z
Bron: https://wiki.osdev.org/ARM_Generic_Timer
Revisie-URL: https://wiki.osdev.org/index.php?title=ARM_Generic_Timer&oldid=23793
Doorverwijzing: ARMv7 Generic Timers

````text
#REDIRECT [[ARMv7 Generic Timers]]
````

<a id="w004372"></a>

## W004372 — FUSE

Bron-ID: W004372
Namespace: Main (0)
Revisie: 27939 | Datum: 2023-07-09T15:49:54Z
Bron: https://wiki.osdev.org/FUSE
Revisie-URL: https://wiki.osdev.org/index.php?title=FUSE&oldid=27939

````text
Creating a [https://en.wikipedia.org/wiki/Filesystem_in_Userspace Filesystem in Userspace] driver is easy. It consist of three parts:
* kernel side module (don't care for now)
* libfuse, a standard user space library, you'll have to link your application with it (otherwise don't care for now)
* your application, which implements the file system hooks expected by libfuse.


It's worth mentioning that the first part has been ported to almost every mainline operating systems (Linux, MacOSX, BSDs, Minix etc.), and there's also a Windows port. The second part was originally written for C, but has been ported to many programming languages, so you can write the third part in C, Python, NodeJS etc. This page focuses on the original C implementation.

Once you have a FUSE driver for your own file system, then you can mount a file system image in your build environment, create files and directories in it etc. Also because FUSE is well structured, you'll be able to re-use significant parts of your driver when you finally implement a native VFS driver in your kernel.

== FUSE Skeleton ==

In it's simplest form, a FUSE driver looks like this:
<source lang="c">
#define FUSE_USE_VERSION 29
#include <fuse.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>

static struct fuse_operations myfs_ops = {
};

char *devfile = NULL;

int main(int argc, char **argv)
{
  int i;

  // get the device or image filename from arguments
  for (i = 1; i < argc && argv[i][0] == '-'; i++);
  if (i < argc) {
    devfile = realpath(argv[i], NULL);
    memcpy(&argv[i], &argv[i+1], (argc-i) * sizeof(argv[0]));
    argc--;
  }
  // leave the rest to FUSE
  return fuse_main(argc, argv, &myfs_ops, NULL);
}
</source>
Let's examine this source line by line.

The first line defines the FUSE API compatibility level. That's libfuse 2.9 as of writing.

Then we include some C headers, fuse.h among others.

The ''myfs_ops'' structure is very important, this struct holds the hooks that we implement for our file system. All hooks are optional, but the driver is disfunctional without some of them. Right now the list is empty.

The ''devfile'' is the name of the device or image file where the file system resides. Because FUSE was designed in a way to work without a device file too, there's no standard way to get this.

The main() function is the heart of the file system driver. Before we can pass the control to FUSE, we have to save the device file, and remove it from the arguments. The for() loop skips any options on the command line (should not be needed, just a precaution). After that the ''i''th argument will be the device that we convert to an absolute path using realpath(). This also checks if the given path is valid, and returns NULL on non-existent files. The memcpy() and the argc-- removes the device from command line arguments.

Finally, we pass control to the FUSE library with fuse_main().

You should be able to compile this driver with:
<source lang="bash">
gcc -I/usr/include/fuse -lfuse myfs.c -o myfs
</source>

To test and mount I recommend to use the following flags:
<source lang="bash">
./myfs -d -f -s myfsimage.bin somedir
</source>
Here -d means print debug messages. The -s tells FUSE to use single thread, and -f to keep it in the foreground so that if you printf() something in your application you can read it. Then cames the name of the image, "myfsimage.bin" which should be a partition image, and "somedir" which is a standard directory where we want to mount the file system in the image. Pressing [Ctrl]+[C] will umount. If you start FUSE in the background (without -f), then you can unmount with
<source lang="bash">
fusermount -u somedir
</source>

=== Adding File Stats ===

First and most important part of the file system driver is to get the attributes of a file (or directory). This hook is called when the user opens the file, lists a directory or calls stat(). It receives a path and an empty stat buffer which you have to fill in according to your file system. If everything was ok, the function returns 0, otherwise a '''negative''' errno, like "return -ENOENT;".
<source lang="c">
int myfs_getattr(const char *path, struct stat *st)
{
  if (!strcmp(path, "/")) {
    // it's the root directory (just an example, you probably have more directories)
    st->st_mode = S_IFDIR | 0755; // access rights and directory type
    st->st_nlink = 2;             // number of hard links, for directories this is at least 2
    st->st_size = 4096;           // file size
  } else {
    st->st_mode = S_IFREG | 0644; // access rights and regular file type
    st->st_nlink = 2;             // number of hard links
    st->st_size = 4096;           // file size
  }
  // user and group. we use the user's id who is executing the FUSE driver
  st->st_uid = getuid();
  st->st_gid = getgid();
  return 0;
}
</source>
This is a very minimal example, you should open ''devfile'', read and parse your file system structures in it and fill in ''st'' fields accordingly.

Of course, we have to tell FUSE that we have implemented a hook by adding it to myfs_opts:
<source lang="c">
static struct fuse_operations myfs_ops = {
  .getattr = myfs_getattr
};
</source>

=== Add Directory Listings ===

Very similar to the first hook, but this time we got a function as argument:
<source lang="c">
int myfs_readdir(const char *path, void *buffer, fuse_fill_dir_t filler, off_t offset, struct fuse_file_info *fi)
{
  filler(buffer, ".", NULL, 0);       // current directory reference
  filler(buffer, "..", NULL, 0);      // parent directory reference
  filler(buffer, "abc.txt", NULL, 0); // any filename at path in your image
  return 0;
}
</source>
If you also implement the "opendir" hook, then ''fi->fh'' will hold the file handle for the opened directory, and you have to read the entry at ''offset''. Otherwise you have to use the ''path'' argument to get the directory entries in that directory.

Of course, we have to tell FUSE that we have implemented a hook by adding it to myfs_opts:
<source lang="c">
static struct fuse_operations myfs_ops = {
  .getattr = myfs_getattr,
  .readdir = myfs_readdir
};
</source>

=== Adding More Hooks ===

These are the basics. There are hooks like "open", "read", "write", "mkdir", "unlink" etc. which should be implemented. The detailed description of these is out of the scope of this introductionary article, but the available hooks (along with a long a very detailed description in comments) can be found [https://github.com/libfuse/libfuse/blob/master/include/fuse.h#L277 in the fuse.h C header].

== Porting FUSE to your OS ==

This is not that easy, but not particularly hard either. There's a FUSE protocol between the kernel and libfuse which you have to implement in your VFS. The protocol is not really documented, but you can take a look at the source which is well commented and uses /dev/fuse pseudo device file. After that kernel part you'll have to port libfuse which depends on libc, so should be straightforward. If you manage to port both, then you'll get access to almost all file systems out there at once. There are many drivers, for archives like tar, zip, z7, for file systems like iso9660, ext4, HFS+, [[NTFS]] etc. which you can compile and use on your OS just as-is.

== See Also ==
=== External Links ===

* [https://en.wikipedia.org/wiki/Filesystem_in_Userspace Filesystem in Userspace] on wikipedia
* [https://github.com/libfuse/libfuse github page] of libfuse with examples
* [https://engineering.facile.it/blog/eng/write-filesystem-fuse/ Tutorial on FUSE]
* [https://github.com/ReturnInfinity/BMFS/blob/master/utils/bmfs-fuse.c BMFS implementation] from Return Infinity, implements [[BMFS]] with FUSE
* [http://www.secfs.net/winfsp/ FUSE Windows port] (requires Cygwin)

[[Category:Filesystems]]
````

<a id="w004373"></a>

## W004373 — File:Smmu-tlb-tagging.png

Bron-ID: W004373
Namespace: File (6)
Revisie: 23503 | Datum: 2019-04-01T19:00:08Z
Bron: https://wiki.osdev.org/File:Smmu-tlb-tagging.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ASmmu-tlb-tagging.png&oldid=23503

````text

````

<a id="w004374"></a>

## W004374 — Talk:ARM Generic Timer

Bron-ID: W004374
Namespace: Talk (1)
Revisie: 23522 | Datum: 2019-04-01T20:28:23Z
Bron: https://wiki.osdev.org/Talk:ARM_Generic_Timer
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AARM_Generic_Timer&oldid=23522

````text
Hey, I have a page about the ARM Generic Timer already -- please see: https://wiki.osdev.org/User:Gravaera/ARM_Generic_Timers

Feel free to merge this page into mine and then move my article out of my personal namespace into the global namespace; or if you can't edit my page or move it, let me know so I can do it instead later -- I'm not very knowledgeable about how MediaWiki works~

[[User:Gravaera|Gravaera]] 15:27, 1 April 2019 (CDT)
````

<a id="w004375"></a>

## W004375 — Talk:Generic Interrupt Controller

Bron-ID: W004375
Namespace: Talk (1)
Revisie: 23524 | Datum: 2019-04-01T20:38:34Z
Bron: https://wiki.osdev.org/Talk:Generic_Interrupt_Controller
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AGeneric_Interrupt_Controller&oldid=23524

````text
[[User:No92]] I noticed that you reverted an edit I made to this page concerning GIC bypass -- was my edit inaccurate?
````

<a id="w004376"></a>

## W004376 — User:Wukl

Bron-ID: W004376
Namespace: User (2)
Revisie: 23542 | Datum: 2019-04-03T18:59:41Z
Bron: https://wiki.osdev.org/User:Wukl
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AWukl&oldid=23542

````text
Hi, my name is Luc and I have managed to develop an OS for seven years without writing a single line of functioning code. You can follow my progress, if any, here: https://git.wukl.net/brutality
````

<a id="w004377"></a>

## W004377 — File:Itmouse.png

Bron-ID: W004377
Namespace: File (6)
Revisie: 23553 | Datum: 2019-04-04T15:31:14Z
Bron: https://wiki.osdev.org/File:Itmouse.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AItmouse.png&oldid=23553

````text

````

<a id="w004378"></a>

## W004378 — Game port

Bron-ID: W004378
Namespace: Main (0)
Revisie: 28285 | Datum: 2023-08-18T18:04:55Z
Bron: https://wiki.osdev.org/Game_port
Revisie-URL: https://wiki.osdev.org/index.php?title=Game_port&oldid=28285

````text
Game port is an old and obsolete parallel communication port. Used mostly for analog joysticks, but also for another devices (like MIDI devices or paddles). Game port uses an DB-15 port. It has eight analog connections for four buttons and four for the axis.

== History ==

The port was introduced in the IBM Game Control Adapter, an optional add-on board for the original IBM PC. Creative Labs included the game port in their soundcards (SoundBlaster)
to provide an "all-in-one" solution for gamers, because sound cards were mostly used for gaming. Since then it was very popular among gamers.
Joysticks aren't limited to four buttons as you may think. Some joysticks (like Microsoft Sidewinder with nine buttons) used multiplexation techniques to get up to 15 buttons or even 20.
 
However, there's a serious drawback; old analog joysticks (that use the game port) use an potentiometer (variable resistor) with a resistance of usually 100kΩ and a capacitor. The controller board calculates the charge threshold of the capacitor, and then converts it to numbers (axis). The problem is that the joystick resistor starts to heat up with time (10-15 mins.), and the resistance changes, leading to incorrect values. Analog joysticks need constant calibration.


== Programming the game port ==

Game port uses port 201h. It's a read and write port. Nothing much to say, it is very simple.
Any axis read must be preceded by an write, it can be any data, it doesn't matter.
{| {{wikitable}}
|-
! IO Port
! Access Type
! Purpose
|-
| 0x201
| Read/Write
| Joystick status
|}

{| {{wikitable}}
|-
! Bits
! Purpose
|-
| 0
| X joystick axis (A). If bit is set, timing isn't active. Same for all.
|-
| 1
| Y joystick axis (A)
|-
| 2
| X joystick axis (B)
|-
| 3
| Y joystick axis (B)
|-
| 4
| Status of (A) button 1. If bit is not set, button has been pressed. Same for all.
|-
| 5
| Status of (A) button 2
|-
| 6
| Status of (B) button 1
|-
| 7
| Status of (B) button 2
|}

There's also and [[BIOS]] call for joystick; Int 0x15, AH 0x84. But it is poorly supported, and most BIOSes have an buggy implementation.


== Example code ==
Example code, to get button status. Uses polling to detect status. You can also use [[IRQ]] 9-11, they are free for peripherals, but it is a waste for such a simple device.

'''WARNING!''' this code has not been tested!

<source lang="c">
/* Simple joystick port controller. hextakatt 2019, public domain. Improvements are welcome, send me an PM (in the forum) if you have any. */
#include <stdint.h>
#include <stdbool.h>

/* Joystick 200h - 201h. */
#define JOYSTICK_PORT      0x201
#define GARBAGE_DATA       0xAA55
#define MAX_TIME_ATTEMPTS  1000

/* Joystick values: axis and buttons */
enum JoystickValues
{
    JoystickButtonA = 0x10,
    JoystickButtonB = 0x20,
    JoystickButtonC = 0x40,
    JoystickButtonD = 0x80,
    JoystickAxisX   = 0x01,
    JoystickAxisY   = 0x02,
    JoystickDeltaX  = 0x04,
    JoystickDeltaY  = 0x08,
};

/* A nice struct which contains the joystick status */
struct JoystickStatus
{
    int16_t AxisX;
    int16_t AxisY;
    int16_t DeltaX;
    int16_t DeltaY;
    bool ButtonA;
    bool ButtonB;
    bool ButtonC;
    bool ButtonD;
    bool JoystickFlag;
};

struct JoystickStatus *jst;
/* If joystick is present, flag is true. */
jst->JoystickFlag = false;

uint8_t joystick_values[] = {
    0x01, 0x02, 0x04, 0x08, /* Axis */
    0x10, 0x20, 0x40, 0x80, /* Buttons */
}

bool joystick_button(uint8_t buttonnum)
{
    for (int i = 4; i <= 8; ++i) { 
        if (joystick_values[i] == buttonnum) {
 	           return ((inb(JOYSTICK_PORT) & buttonnum) == 0);
        }
    }
    kputs("Invalid value: %x; Out of bounds.\n", buttonnum);
    return 0;
}

/* Dump all joystick values to an structure */
struct JoystickStatus joystick_dump(void)
{
    struct JoystickStatus jst;
    jst.AxisX   = joystick_status(JoystickAxisX);
    jst.AxisY   = joystick_status(JoystickAxisY);
    jst.DeltaX  = joystick_status(JoystickDeltaX);
    jst.DeltaY  = joystick_status(JoystickDeltaY);
    jst.ButtonA = joystick_button(joystick_status(JoystickButtonA));
    jst.ButtonB = joystick_button(joystick_status(JoystickButtonB));
    jst.ButtonC = joystick_button(joystick_status(JoystickButtonC));
    jst.ButtonD = joystick_button(joystick_status(JoystickButtonD));
    return jst;
}

/* 
 * There's no IRQ for joystick port, so we poll instead.
 * However isn't a problem, because very few data is transmitted
 * through game port.
 */
uint16_t joystick_status(uint8_t byte)
{
    int timeout = 0;
    /* Disable interrupts, so they don't affect timing */
    asm volatile("cli");

    uint16_t stat = inb(JOYSTICK_PORT);
    /* Any read needs to be with a write. The byte that we send 
     * can be garbage, it doesn't matters.
     */
    outb(JOYSTICK_PORT, GARBAGE_DATA);

    /* Poll until we get our requested value */
    while (1) {
        if (stat & byte) {
            jst->JoystickFlag = true;
            break;
        }
      	 else if (timeout == MAX_TIME_ATTEMPTS) {
            jst->JoystickFlag = false;
            return 0;
    	}
        ++timeout;
    }
    /* Now we can enable interrupts again */
    asm volatile("sti");
    return (stat & byte);
}
</source>

== See Also ==
'''External Links'''

[https://github.com/torvalds/linux/blob/master/drivers/input/gameport/gameport.c Linux gameport implementation]

[http://web.archive.org/web/20021208052845/http://www.nondot.org/sabre/os/files/HCI/JOYSTICK.TXT Nondot OSRC joystick article]

[https://www.minuszerodegrees.net/oa/OA%20-%20IBM%20Game%20Control%20Adapter.pdf IBM Game Control Adapter manual]

[[Category:Hardware Interfaces]]
````

<a id="w004379"></a>

## W004379 — User:Hextakatt

Bron-ID: W004379
Namespace: User (2)
Revisie: 23556 | Datum: 2019-04-06T01:43:47Z
Bron: https://wiki.osdev.org/User:Hextakatt
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AHextakatt&oldid=23556

````text
Hi, I'm hextakatt. (Eh, the wiki nickname capitalization pisses me off). I'm from Chile and I am working in Gryphus, my operating system. Written mostly in C, and very little assembly.
Nothing much to say, yet. :D
````

<a id="w004380"></a>

## W004380 — User:No92/ACPICA

Bron-ID: W004380
Namespace: User (2)
Revisie: 23579 | Datum: 2019-04-17T14:13:08Z
Bron: https://wiki.osdev.org/User:No92/ACPICA
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANo92%2FACPICA&oldid=23579

````text
The ACPI Component Architecture ACPICA provides an operating system (OS)-independent reference implementation of the Advanced Configuration and Power Interface. It can be adapted to any host OS. The ACPICA code is meant to be directly integrated into the host OS, as a kernel-resident subsystem. Hosting the ACPICA requires no changes to the core ACPICA code. However, it does require a small OS-specific interface layer, which must be written specifically for each host OS.

The complexity of the ACPI specification leads to a lengthy and difficult implementation in OS software. The purpose of the ACPI Component Architecture is to simplify ACPI implementations for operating system vendors by providing major portions of an ACPI implementation in OS-independent ACPI modules that can be easily integrated into any OS.

=Setup=

Obtain ACPICA at [https://www.acpica.org/downloads acpica.org] and extract the archive.

==Formatting==
There are basically two flavors of ACPICA around; they differ in code style only. Should you want your ACPICA function looking like <code>AcpiGetIrqRoutingTable</code>, you are lucky and don't have to do anything. Should you want them looking like <code>acpi_get_irq_routing_table</code>, you will have to convert them to "Linux style". Thankfully, there is a tool supplied to do exactly this. Run this:
<source lang="bash">
make acpisrc
</source>
To convert the source code in a folder, you can now do this:
<source lang="bash">
./generate/unix/bin/acpisrc -ldqy <source directory> <destination directory>
</source>
You might have to fix <code>acdisasm.h:478</code>, changing <code>AH_TABLE</code> to <code>struct ah_table</code> for a successful compilation (you do want this, right?).

==Integrating the code==
This boils down to copying a few files over. Start off by copying the headers from <code>source/include</code>. After that, copy over everything from <code>source/components</code>, except for the subdirectories <code>debugger</code> and <code>disassembler</code>. This means doing this:
<source lang="bash">
cp source/components/{dispatcher,events,executer,hardware,parser,namespace,utilities,tables,resources}/*.c <destination>
</source>

You might have to adjust the header <code>#includes</code>, which can be done with sed or a simple script.

==Writing your own platform header==
This is a header placed in <code>include/platform/acmyos.h</code>. A basic one might look something like this:
<source lang="c">
#pragma once

#include <ctype.h>
#include <stdint.h>
#include <string.h>

#define ACPI_MACHINE_WIDTH          64
#define COMPILER_DEPENDENT_INT64    int64_t
#define COMPILER_DEPENDENT_UINT64   uint64_t

#define ACPI_USE_DO_WHILE_0
#define ACPI_MUTEX_TYPE             ACPI_OSL_MUTEX

#define ACPI_CACHE_T                ACPI_MEMORY_LIST
#define ACPI_USE_LOCAL_CACHE        1

#define ACPI_SINGLE_THREADED
#define ACPI_DEBUG_OUTPUT
#define ACPI_USE_STANDARD_HEADERS
#define ACPI_USE_SYSTEM_CLIBRARY

#include <acpica/platform/acgcc.h>
</source>

Be aware that this assumes that you have a few utility functions ready, mostly from <code>string.h</code>. This is a minimum viable example, keeping the requirements down somewhat. See the documentation about what each line means and how to modify it to suit your needs.

=Writing your OSL=

This is the longest task for implementation. If you have a good kernel to begin with, you probably won't have to write a lot of code though. Again, for details see the documentation.

==Environmental and ACPI Tables==

===acpi_os_initialize===
<source lang="c">
acpi_status acpi_os_initialize(void) {
	return AE_OK;
}
</source>

This is called during ACPICA initialization to do your initialization work, if needed.

===acpi_os_terminate===
<source lang="c">
acpi_status acpi_os_terminate(void) {
	return AE_OK;
}
</source>

This is called during ACPICA termination, should that occur.

===acpi_os_get_root_pointer===
<source lang="c">
acpi_physical_address acpi_os_get_root_pointer(void)
</source>

Just return the RSDP. If your OS doesn't use UEFI, this boils down to:
<source lang="c">
acpi_physical_address ret;
ret = 0;
acpi_find_root_pointer(&ret);
return ret;
</source>

Note: The ACPI specification is highly portable specification, however, it has a static part which is generally non-portable: the location of the Root System Descriptor Pointer. This pointer may be found in many different ways depending on the chipset. On PC-compatible computers (without EFI) it is located in lower memory generally somewhere between 0x80000 and 0x100000. However, even within the PC compatible platform, an EFI-enabled board will export the RSDP to the OS on when it loads it through the EFI system tables. Other boards on server machines which are not PC-compatibles, like embedded and handheld devices which implement ACPI will again, not all be expected to position the RSDP in the same place as any other board. The RSDP is therefore located in a chipset-specific manner; From the time the OS has the RSDP, the rest of ACPI is completely portable. However, the way the RSDP is found is not. This would be the reason that the ACPICA code wouldn't try to provide routines to expressly find the RSDP in a portable manner. If your system uses EFI, locate it in the system tables or use Multiboot2 compliant loader, which provides the RSDP for you.

===acpi_os_predefined_override===
<source lang="c">
acpi_status acpi_os_predefined_override(const struct acpi_predefined_names *PredefinedObject, acpi_string *NewValue) {
	if(!PredefinedObject || !NewValue) {
		return AE_BAD_PARAMETER;
	}

	*NewValue = NULL;
	return AE_OK;
}
</source>

This allows you to override predefined objects in the ACPI namespace. It is called when a new object is found in the ACPI namespace. However you can just put NULL in *NewValue and return.

===acpi_os_table_override===
<source lang="c">
acpi_status acpi_os_table_override(struct acpi_table_header *ExistingTable, struct acpi_table_header **NewTable) {
	*NewTable = 0;
	return AE_OK;
}
</source>

The same of AcpiOsPredefinedOverride but for entire ACPI tables. You can replace them. Just put NULL in *NewTable and return.

===acpi_os_physical_table_override===
<source lang="c">
acpi_status acpi_os_physical_table_override(struct acpi_table_header *ExistingTable, acpi_physical_address *NewAddress, uint32_t *NewTableLength) {
	*NewAddress = 0;
	return AE_OK;
}
</source>

This allows you to override a found ACPI table with a new table via a physical address and a length. Again, this gets the NULL treatment.

==Memory Management==

===acpi_os_map_memory===
<source lang="c">
void *acpi_os_map_memory(acpi_physical_address PhysicalAddress, acpi_size length)
</source>

This is not really easy. ACPICA is asking you to map a physical address in the virtual address space. If you don't use paging, just return PhysicalAddress. You need:
# To round Length up to the size of a page (Length can be 2, 1024 for example)
# Find a range of virtual addresses where map the physical frames.
# Map the physical frames to the virtual addresses choosen.
# Return the virtual address plus the page offset of the physical address. (Eg. If you where asked to map 0x40E you have to return 0xF000040E and not just 0xF0000000)

===acpi_os_unmap_memory===
<source lang="c">
void acpi_os_unmap_memory(void *where, acpi_size length)
</source>

Unmap pages mapped using acpi_os_map_memory. <code>where</code> is the virtual address returned in acpi_os_map_memory and length is equal to the length of the same function. Just remove the virtual address from the page directory and set that virtual address as reusable. Note: for the last two functions you might need a separated heap.

===acpi_os_get_physical_address===
<source lang="c">
acpi_status acpi_os_get_physical_address(void *LogicalAddress, acpi_physical_address *PhysicalAddress)
</source>

Get the physical address of LogicalAddress and put it in *PhysicalAddress. If you do not use paging just put LogicalAddress in *PhysicalAddress.

===acpi_os_allocate===
<source lang="c">
void *acpi_os_allocate(acpi_size size) {
	return malloc(size);
}
</source>

===acpi_os_free===
<source lang="c">
void acpi_os_free(void *Memory) {
	free(Memory);
}
</source>

===acpi_os_readable===
<source lang="c">
uint8_t acpi_os_readable(vy_unused void *Memory, vy_unused acpi_size Length)
</source>

This is never called, just have it in here.

===acpi_os_writable===

<source lang="c">
uint8_t acpi_os_writable(vy_unused void *Memory, vy_unused acpi_size Length)
</source>

This is never called, just have it in here.

==Multithreading and Scheduling Services==

===acpi_os_get_thread_id===
<source lang="c">
acpi_thread_id acpi_os_get_thread_id(void)
</source>

Return a non-zero value identifying the current thread. If you disabled threading as in the example configuration above, just return 1.

===acpi_os_execute===
<source lang="c">
acpi_status acpi_os_execute(vy_unused acpi_execute_type Type, vy_unused acpi_osd_exec_callback Function, vy_unused void *Context)
</source>

Create a new Thread (or process) with entry point at Function using parameter Context. Type is not really useful. When the scheduler chooses this thread it has to put Context on the stack to have something like:

<source lang="c">
Function(context);
</source>

If you disabled threading as in the example configuration above, this will not be called.

===acpi_os_sleep===
<source lang="c">
void acpi_os_sleep(uint64_t Milliseconds) {
	sleep(Milliseconds);
}
</source>

Put the current thread to sleep for n milliseconds. If you disabled threading as in the example configuration above, this is the same as <code>acpi_os_stall</code>, just with different units.

===acpi_os_stall===
<source lang="c">
void acpi_os_stall(vy_unused uint32_t Microseconds)
</source>

Stall the current thread for <code>n</code> microseconds. If you disabled threading as in the example configuration above, this is the same as <code>acpi_os_sleep</code>, just with different units.

===acpi_os_wait_events_complete===
<source lang="c">
void acpi_os_wait_events_complete(void)
</source>

This function blocks until all asynchronous events initiated by <code>acpi_os_execute</code> have completed. If you disabled threading as in the example configuration above, you probably don't care.

==Mutual Exclusion and Synchronization==
If you disabled threading as in the example configuration above, you can simply stub these functions out to do nothing and return <code>AE_OK</code>, if applicable.

<source lang="c">
acpi_status acpi_os_create_mutex(acpi_mutex *mutex);
void acpi_os_delete_mutex(acpi_mutex mutex);
acpi_status acpi_os_acquire_mutex(acpi_mutex mutex, uint16_t timeout);
void acpi_os_release_mutex(acpi_mutex mutex);

acpi_status acpi_os_create_semaphore(uint32_t max_units, uint32_t initial_units, acpi_semaphore *s);
acpi_status acpi_os_delete_semaphore(acpi_semaphore s);
acpi_status acpi_os_wait_semaphore(acpi_semaphore s, uint32_t units, uint16_t timeout);
acpi_status acpi_os_signal_semaphore(acpi_semaphore s, uint32_t units);

acpi_status acpi_os_create_lock(acpi_spinlock *lock);
void acpi_os_delete_lock(acpi_spinlock lock);
acpi_cpu_flags acpi_os_acquire_lock(acpi_spinlock lock);
void acpi_os_release_lock(acpi_spinlock lock, acpi_cpu_flags flags);
</source>

==Interrupt Handling==
==Port Input/Output==
==PCI Configuration Space Access==
==Formatted output==
==System ACPI Table Access==
==Miscellaneous==

===undefined references fixup===
I don't know where this goes, so do this here: ACPICA requires these functions to exist, even though we don't use the debugger.

<source lang="c">
acpi_status AcpiDbSingleStep(void *state, void *op, uint32_t optype) {
	panic("%s unimplemented", __func__);
}
</source>

<source lang="c">
void AcpiDbSignalBreakPoint(void *state) {
	panic("%s unimplemented", __func__);
}
</source>

<source lang="c">
void AcpiDbDumpMethodInfo(acpi_status status, void *state) {
	panic("%s unimplemented", __func__);
}
</source>

<source lang="c">
void AcpiDbDisplayArgumentObject(void *desc, void *state) {
	panic("%s unimplemented", __func__);
}
</source>

<source lang="c">
void AcpiDbDisplayResultObject(void *desc, void *walk) {
	panic("%s unimplemented", __func__);
}
</source>
````

<a id="w004381"></a>

## W004381 — MIPS

Bron-ID: W004381
Namespace: Main (0)
Revisie: 23598 | Datum: 2019-05-03T18:45:19Z
Bron: https://wiki.osdev.org/MIPS
Revisie-URL: https://wiki.osdev.org/index.php?title=MIPS&oldid=23598
Doorverwijzing: Category:MIPS

````text
#REDIRECT [[Category:MIPS]]
````

<a id="w004382"></a>

## W004382 — ARC

Bron-ID: W004382
Namespace: Main (0)
Revisie: 29927 | Datum: 2026-01-22T21:37:38Z
Bron: https://wiki.osdev.org/ARC
Revisie-URL: https://wiki.osdev.org/index.php?title=ARC&oldid=29927

````text
{{Stub}}

ARC is a firmware specification that was developed in the early 90s by a consortium including companies like Microsoft, SCO, and MIPS Technologies. It was loosely used by several [[MIPS]], [[PowerPC]], and [[Alpha]] machines, as well as a particularly unusual SGI [[x86]] machine. Most machines that implement ARC are loosely conformant at best, and more often, they directly violate the spec in some way.

ARC based systems expect a boot payload in ECOFF format, an extended version of [[COFF]].

== See Also ==
=== External Links ===
[ftp://ftp.linux-mips.org/pub/linux/mips/doc/ARC/riscspec.pdf ARC Specification v1.2]

[[Category:Firmware]]
[[Category: DEC Alpha]]
[[Category: MIPS]]
[[Category: PowerPC]]
````

<a id="w004383"></a>

## W004383 — Talk:Bootloader FAQ

Bron-ID: W004383
Namespace: Talk (1)
Revisie: 23602 | Datum: 2019-05-04T09:27:43Z
Bron: https://wiki.osdev.org/Talk:Bootloader_FAQ
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABootloader_FAQ&oldid=23602

````text
== Split page? ==
This page is a whole bunch of topics:
* Bootloader choices -- nicely written
* Firmware choices -- not only lacks any explanation, it omits CoreBoot, OpenFirmware (a possibility on x86 with CoreBoot), [[ARC]], [[System_Initialization_(SGI_Mips)|ARCS]], and devices intended to boot from internal Flash memory or other ROM (And that's just the ones I've heard of! lol)
* A detailed look into loading the kernel with just one of those firmware choices
* Things a bootloader should support
I can't see how to organize these disparate things into one article. Should the page have a section on loading the kernel for each of the firmware choices? No, loading the kernel with UEFI is nicely explained in another page. I propose:
* Move "How do I load the kernel (BIOS)?" into its own page
* Fill in the firmware choices section with things actually useful to making a decision, especially BIOS vs. UEFI but also a little look into the other options
* Retain bootloader choices and things a bootloader should support
* Adding a section on boot media, but rather than try to list every one, lump them into categories, each with overall pros and cons, brief notes on each firmware, and links to the firmware pages for more info.  Categories:
** Internal Flash 
** Attached Media (including hard disks)
** Network
Any thoughts? Yes, I will help with the work when I can, but for such big changes I thought I should ask for opinions first.
[[User:Eekee|Eekee]] 15:53, 3 May 2019 (CDT)
===Re: Split page?===
I would tend to agree; this should also include placing this page more prominently on the Main Page and [[Bare Bones]] and [[Meaty Skeleton]].
[[User:No92|no92]] 04:27, 4 May 2019 (CDT)
````

<a id="w004384"></a>

## W004384 — File:Qword1.png

Bron-ID: W004384
Namespace: File (6)
Revisie: 23613 | Datum: 2019-05-09T23:25:03Z
Bron: https://wiki.osdev.org/File:Qword1.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AQword1.png&oldid=23613

````text
Screenshot of qword running GCC and various programs.
````

<a id="w004385"></a>

## W004385 — User:Gravaera/Xilinx Ultrascale+ MPSoC

Bron-ID: W004385
Namespace: User (2)
Revisie: 23641 | Datum: 2019-05-23T06:36:18Z
Bron: https://wiki.osdev.org/User:Gravaera/Xilinx_Ultrascale%2B_MPSoC
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FXilinx_Ultrascale%2B_MPSoC&oldid=23641

````text
= Boot process =

The SoC has a CSU (Configuration and Security Unit) ROM. This ROM is executed first followed by a PMU Pre-boot ROM (See Ch 37 "Basic Clock Generators"), and then the CSU Boot ROM samples the "mode pins" (asserted by the chipset) which tell it which internal storage device should be used as the boot device for the FSBL (First Stage Boot Loader). The FSBL is packaged with some headers which tell the CSU ROM software about how to load it.

After FSBL begins executing the CSU ROM software goes resident and monitors the running system for "tamper signals" from "various sources [sic] in the system".

Power management for the SoC is done through the PMU (Power Management Unit) which takes requests to power up and down peripherals and also is used to enable and disable clocks and resets.

The Processing System (The traditional general purpose computing platform) and the Programmable Logic (The FPGA embedded into the SoC) go by those names. The PS and PL are connected by means of an AXI Cache-coherent  Interconnect which is called the "AMBA Interconnect".

= Clocks =

There are 5 top level clock sources in the clock tree; these are called the PLL source clocks because they pipe into the 5 top level PLLs. They are:
* PS_REF_CLK (Device pin, normal source)
* ALT_REF_CLK (Its source is chosen from one of two MIO pins)
* VIDEO_REF_CLK (Its source is chosen from one of two MIO pins)
* AUX_REF_CLK (Its source is within the PL Fabric)
* GTR_REF_CLK. Its source is an multiplexer output from one of the four GTR inputs to the SIOU -- these four are:
** 0: PCIe/USB.
** DisplayPort.
** SATA.
** SGMII.

The PS subsystem has two power domains: the LPD (Low Power Domain) and the FPD (Full Power Domain). There are '''two''' top-level PLLs within the LPD and '''three''' top level PLLs within the FPD, for a total of 5 top-level PLLs within the PS subsystem. These 5 PLLs are called:
* RPLL_CLK: RPU (Realtime Processing Unit -- Cortex R .Little cluster of the Big.little arrangement) PLL.
* IOPLL_CLK: IO Peripheral PLL.
* APLL_CLK: APU (Application Processing Unit -- Cortex A Big cluster) PLL.
* VPLL_CLK: Video PLL (GPU and video codec unit).
* DPLL_CLK: DDR PLL.

There are however, 5 divisors within each of the power domains for a total of 10 divisors. The outputs from all 5 top-level PLL are split up and piped into the 5 divisors in both power domains such that both power domains have access to the '''outputs''' from all 5 PLLs, supplied to them as '''inputs''' to their 5 respective divisors. The divisors which reside in power domains that are different from the power domain that their PLL source resides in are called "power domain crossed clocks".

The 5 divisors then pipe into various "clock generators" which are piped into various peripherals and IPs.
````

<a id="w004386"></a>

## W004386 — Xilinx Ultrascale+ MPSoC

Bron-ID: W004386
Namespace: Main (0)
Revisie: 23624 | Datum: 2019-05-17T05:40:40Z
Bron: https://wiki.osdev.org/Xilinx_Ultrascale%2B_MPSoC
Revisie-URL: https://wiki.osdev.org/index.php?title=Xilinx_Ultrascale%2B_MPSoC&oldid=23624
Doorverwijzing: User:Gravaera/Xilinx Ultrascale+ MPSoC

````text
#REDIRECT [[User:Gravaera/Xilinx Ultrascale+ MPSoC]]
````

<a id="w004387"></a>

## W004387 — Talk:PC Speaker

Bron-ID: W004387
Namespace: Talk (1)
Revisie: 23755 | Datum: 2019-07-11T23:42:26Z
Bron: https://wiki.osdev.org/Talk:PC_Speaker
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APC_Speaker&oldid=23755

````text
The PWM "How It Works" section talks about "approximately 60 millionths of a second" for the speaker to switch positions, i.e. 60 microseconds; the rest of the article, however, talks about 60 MILLIseconds.  1/(60us) is about 16.7 kHz, 1/(60ms) is about 16.7 Hz; as the speaker is easily capable of playing beeps in the 100-5000 Hz range at a reasonable amplitude, I'm guessing 60 microseconds is correct, and 60 ms isn't.  Source?
- unknown

I have tested the code in QEMU (-soundhw pcspk) and in real hardware, and seems that even calling the nosound() function doesn't stops the sound. 
In QEMU, after having called such a function, the sound starts to sound choppy.
- hextakatt
````

<a id="w004388"></a>

## W004388 — Scalable Screen Font

Bron-ID: W004388
Namespace: Main (0)
Revisie: 30325 | Datum: 2026-03-17T20:49:30Z
Bron: https://wiki.osdev.org/Scalable_Screen_Font
Revisie-URL: https://wiki.osdev.org/index.php?title=Scalable_Screen_Font&oldid=30325

````text
'''Scalable Screen Font''' is a compressed font format which comes with a small, free, MIT licensed rendering library. It was designed specially for hobby OS developers, therefore it has minimal dependencies. It is also floating-point arithmetic free, so it will work even in kernels that haven't initialized the [[FPU]] or [[SSE]] yet, or simply don't support floating arithmetic at all.

= Supported Font Types =

* Bitmap - you can convert [[PC Screen Font]] with UNICODE table into SSFN directly. X11 Bitmap Distribution Format, Portable Compiled Fonts and Windows FNT/FON files are also supported.
* Pixmap - to have colorful emoji icons, you can convert TGA and PNG files into SSFN fonts.
* Vector - vector based fonts can be converted from PostScript Type1, OpenType and [[TrueType_Fonts|TrueType fonts]].

Fonts are compressed, and in case of vector fonts, with a data-loss compression. With a typical OpenType file, the SSFN font usually takes only the half the file size (or less).

= Library =

The SSFN package comes with a single ANSI C/C++ header file, which has two renderers. The header contains everything, no shared library nor static linking needed. They are compatible with both [[VESA]] and [[GOP]] framebuffers.

=== Simple Renderer ===

Designed specially for OS consoles, has only one function. It can render unscaled bitmap fonts directly to the framebuffer. Has absolutely no dependencies, and compiles to less than two kilobytes of code.

<syntaxhighlight lang="c">
#define SSFN_CONSOLEBITMAP_TRUECOLOR        /* use the special renderer for 32 bit truecolor packed pixels */
#include <ssfn.h>

/* set up context by global variables */
ssfn_src = &_binary_console_sfn_start;      /* the bitmap font to use */
ssfn_dst.ptr = 0xE0000000;                  /* framebuffer address and bytes per line */
ssfn_dst.p = 4096;
ssfn_dst.fg = 0xFFFFFFFF;                   /* colors, white on black */
ssfn_dst.bg = 0;
ssfn_dst.x = 100;                           /* coordinates to draw to */
ssfn_dst.y = 200;

/* render text directly to the screen and then adjust ssfn_dst.x and ssfn_dst.y */
ssfn_putc('H');
ssfn_putc('e');
ssfn_putc('l');
ssfn_putc('l');
ssfn_putc('o');
</syntaxhighlight>

=== Normal Renderer ===

There's another for user space applications. This one can render all three types of fonts, supports gzip compressed fonts, it can scale, anti-alias and kern glyphs. Has minimal libc dependencies (memset, memcmp, realloc, free) and compiles to about 28 kilobytes of code. (Just for completeness, you can compile it in total dependency-free mode if you define SSFN_MAXLINES, but then there'll be no internal glyph cache and you must provide inflated fonts only to ssfn_load().)

<syntaxhighlight lang="c">
#define SSFN_IMPLEMENTATION                         /* use the normal renderer implementation */
#include <ssfn.h>

ssfn_t ctx;                                         /* the renderer context */
ssfn_buf_t buf;                                     /* the destination pixel buffer */

/* you don't need to initialize the library, just make sure the context is zerod out */
memset(&ctx, 0, sizeof(ssfn_t));

/* add one or more fonts to the context. Fonts must be already in memory */
ssfn_load(&ctx, &_binary_times_sfn_start);          /* you can add different styles... */
ssfn_load(&ctx, &_binary_timesbold_sfn_start);
ssfn_load(&ctx, &_binary_timesitalic_sfn_start);
ssfn_load(&ctx, &_binary_emoji_sfn_start);          /* ...or different UNICODE ranges */
ssfn_load(&ctx, &_binary_cjk_sfn_start);

/* select the typeface to use */
ssfn_select(&ctx,
    SSFN_FAMILY_SERIF, NULL,                        /* family */
    SSFN_STYLE_REGULAR | SSFN_STYLE_UNDERLINE,      /* style */
    64                                              /* size */
);

/* describe the destination buffer. Could be a 32 bit linear framebuffer as well */
buf.ptr = sdlsurface->pixels;                       /* address of the buffer */
buf.w = sdlsurface->w;                              /* width */
buf.h = sdlsurface->h;                              /* height */
buf.p = sdlsurface->pitch;                          /* bytes per line */
buf.x = buf.y = 100;                                /* pen position */
buf.fg = 0xFF808080;                                /* foreground color */

/* rasterize the first glyph in an UTF-8 string into a 32 bit packed pixel buffer. Returns how many bytes were consumed from the string */
ssfn_render(&ctx, &buf, "Hello");
ssfn_render(&ctx, &buf, "ello");
ssfn_render(&ctx, &buf, "llo");                     /* assuming there's a ligature for "ll" in the font */
ssfn_render(&ctx, &buf, "o");

/* free resources */
ssfn_free(&ctx);                                    /* free the renderer context's internal buffers */
</syntaxhighlight>
The renderer takes care of the font direction, it uses horizontal or vertical alignment automatically. It cannot determine right-to-left though, for that you'll need
a minimal BiDi state machine too. That has to be implemented in the text renderer (or text shaping library) that's built on top of the low level rasterizer. The [http://www.unicode.org/reports/tr9/ algorithm to properly display bidirectional texts] is specified by UNICODE. But once you have decided that you need to draw a glyph in right-to-left direction, just pass SSFN_STYLE_RTL to ssfn_select().

= Links =
== See Also ==
* [[VGA Fonts]]
* [[PC Screen Font]]

== External Links ==
* [https://forum.osdev.org/viewtopic.php?f=2&t=33719 OSDEV forum] on Scalable Screen Fonts
* https://gitlab.com/bztsrc/scalable-font2 source code repository
* [https://gitlab.com/bztsrc/scalable-font2/blob/master/docs/sfn_format.md .sfn file format] description
* [https://gitlab.com/bztsrc/scalable-font2/blob/master/docs/API.md SSFN API] documentation

[[Category:Graphical UI]]
[[Category:Fonts]]
````

<a id="w004389"></a>

## W004389 — Talk:Historical Notes on CISC and RISC

Bron-ID: W004389
Namespace: Talk (1)
Revisie: 23684 | Datum: 2019-06-11T23:25:35Z
Bron: https://wiki.osdev.org/Talk:Historical_Notes_on_CISC_and_RISC
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AHistorical_Notes_on_CISC_and_RISC&oldid=23684

````text
Anyone interested in RISC vs CISC should give Sun Microsystems libmath a read (it's in the BSD source trees). It's amazing how many thousands of lines of C they wrote, to do things that take a single instruction on Intel X87 CISC FPU. They also used a lot of type punning, so the code Sun wrote in the 90's might stop working with modern compilers. But X87 still works as it always has, and always will.
* (Above by Norepinephrine. Please sign posts on talk pages with 4 '~' characters.)
* There are several objections to this. A FPU is not a "computer" in the sense of RISC or CISC, it's not a central processing unit. Floating point emulation was commonly needed until some time in the 90s. (Later in embedded hardware.) The x87 was never the only FPU around; other CPU ranges had their own. (Until the Pentium, the x87 was slower than other architectures' FPUs, including SPARC's.) Finally, the big reason: ''If I remember right,'' libmath is a standard library which does a whole lot more than floating point emulation. It's also responsible for extended-precision arithmetic, for instance 100-digit numbers; a very hard thing to make high-performance. (I may be mixing it up with libm.) [[User:Eekee|Eekee]] 18:25, 11 June 2019 (CDT)
````

<a id="w004390"></a>

## W004390 — User:Ask

Bron-ID: W004390
Namespace: User (2)
Revisie: 23669 | Datum: 2019-06-09T15:37:01Z
Bron: https://wiki.osdev.org/User:Ask
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAsk&oldid=23669

````text
Hello
````

<a id="w004391"></a>

## W004391 — File:Screenshot-191112e.png

Bron-ID: W004391
Namespace: File (6)
Revisie: 23701 | Datum: 2019-06-21T20:10:55Z
Bron: https://wiki.osdev.org/File:Screenshot-191112e.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AScreenshot-191112e.png&oldid=23701

````text
A screenshot of the Serenity OS on 2019-06-07.
````

<a id="w004392"></a>

## W004392 — Xv6

Bron-ID: W004392
Namespace: Main (0)
Revisie: 29896 | Datum: 2026-01-17T08:16:24Z
Bron: https://wiki.osdev.org/Xv6
Revisie-URL: https://wiki.osdev.org/index.php?title=Xv6&oldid=29896

````text
Originally [http://www.minix.org Minix], just like Xv6, was designed for educational purposes. As years went by, Minix became more complex and more like a real OS, so that at version 3 being educational was not a goal anymore. So much that Tanenbaum officially changed its focus to being fully functional. As a result, its codebase does not help beginners any more. Thankfully we still have Xv6, which remained a simple educational OS.

== History ==

Xv6 is a re-implementation of the classic Dennis Richie's and Ken Thompson's UNIX V6 for MIT Operating System course. UNIX was originally written in K&R C for the PDP-11. Xv6 is a modernized version of that, written in ANSI C for the Intel x86-32, x86-64 and RISC-V processor, keeping the original UNIX philosophy of simplicity.

== Advantages ==

* Very small codebase. Being educational, it focuses on how an OS should work instead of implementing everything. A single programmer can easily overview its entire codebase.
* K.I.S.S. uses clear and simple algorithms only, most sources are no longer than few hundred lines.
* Minimal assembly. Most of the code is written in ANSI C.
* Uses protected mode on x86. It abstracts the x86 architecture away with minimal lines of code, and it does not rely on BIOS.
* Fairly modern. It supports IOAPIC, LAPIC etc. It does not use obsolete devices such as floppies like other tutorials do.
* Fairly sophisticated. It uses paging for virtual address spaces, and it is a multitasking OS with symmetric multiprocessing (SMP) support.
* Fully functional. Unlike most tutorials and bare-bones, Xv6 really implements everything that has to be implemented in a working kernel.
* Easy to understand. Has a great commentary book, that also points out which parts of the kernel should be refactored for a real-life OS and how.

== Disadvantages ==

* Unlike Minix, Xv6 is a [[monolithic kernel]], which means OS layers are not separated.
* Source is not separated either, furthermore kernel space code is mixed with user space code. Thankfully there's a [https://github.com/mit-pdos/xv6-public/blob/master/runoff.list list] of files grouped by which module they belong. Everything not listed there is user space code.

== External Links ==
* [https://pdos.csail.mit.edu/6.828/2018/xv6.html Xv6 homepage]
* [https://pdos.csail.mit.edu/6.828/2018/xv6/book-rev11.pdf Commentary book on Xv6] in PDF, it is brief (less than 100 pages) and an easy reading
* [https://github.com/mit-pdos/xv6-public Source on GitHub]
** [https://github.com/mit-pdos/xv6-riscv RISC-V version]
** [https://xv6-64.gitlab.io/xv6-64/ x86-64 version]
* [https://www.youtube.com/watch?v=fWUJKH0RNFE&list=PLbtzT1TYeoMhTPzyTZboW_j7TPAnjv9XB hhp3 walkthrough on YouTube]

[[Category:Operating Systems]]
````

<a id="w004393"></a>

## W004393 — BOOTBOOT

Bron-ID: W004393
Namespace: Main (0)
Revisie: 27196 | Datum: 2022-06-21T07:40:45Z
Bron: https://wiki.osdev.org/BOOTBOOT
Revisie-URL: https://wiki.osdev.org/index.php?title=BOOTBOOT&oldid=27196

````text
The BOOTBOOT loader has exactly the opposite philosophy than [[GRUB]], it is not a Grand Unified Loader, rather a set of simple and small loaders which provide the same boot environment for your kernel on different platforms.

== History ==

This loader was created to an easy alternative to the complex GRUB by one of the OSDev forum members.

== Supported Platforms ==

BOOTBOOT can load your 64 bit kernel on the following platforms:
* Raspberry Pi (AArch64)
* Legacy BIOS (x86_64 with MBR)
* Legacy BIOS boot with any arbitrary boot manager (x86_64, chainloaded from VBR)
* Legacy BIOS boot with a single OS on a forward-compatible GPT disk (x86_64, booted from PMBR)
* Legacy BIOS for embedded systems (x86_64, in BIOS Expansion ROM)
* UEFI (x86_64)
* UEFI for embedded systems (x86_64, in PCI Option ROM)
* CDROM, El Torito "no emulation" BIOS boot (x86_64, hybrid GPT/ISO9660 image)
* CDROM, El Torito UEFI boot Extension (x86_64)
* ISOLINUX / LILO / BOOTLIN / etc. (x86_64 via the Linux kernel boot protocol)
* COREBOOT: as a primary payload in ROM (x86_64)

For GRUB compatibility, BOOTBOOT is also Multiboot compliant, so you can add it to the grub.cfg in it's own entry. This is useful if want to dual boot multiple OSes.

== Supported Sources ==

Your kernel can be wrapped in an initial ramdisk archive ([[USTAR]], cpio, James Molloy's initrd format etc.), which in turn can be a simple file on a [[FAT|FAT16/32]] partition; or can occupy the whole partition as well using a file system of your choosing ([[SFS]] among others). BOOTBOOT uses GPT partitions, so your boot partition should be either an [[EFI_System_Partition|ESP]], or it should have the bootable flag (attribute 2) set.

If despite the warning you decide to [[Roll Your Own Filesystem]], then BOOTBOOT is definitely for you. You can either write an FS module (one function), or just make sure the kernel is the first executable in the archive / on the partition. BOOTBOOT is capable of loading kernels from unknown file systems as well.

Optionally you can load your OS from ROM (see qemu's -option-rom command line argument, or bochs rc file; or with [https://coreboot.org coreboot] as a BIOS using qemu -bios), and the Raspberry Pi version can also load the ramdisk over serial line. Although it's not implemented as of yet, the BOOTBOOT protocol allows loading the initrd from a TFTP server.

== Your Kernel ==

You can use ELF64 or PE32+ formats for your kernel. You must link it as a [[Higher Half Kernel]], and your entry point should point to [[Long Mode]] code. That's all. Unlike all the other boot protocols (e.g. [[Multiboot]] and [[stivale]]), there's absolutely no need for special sections nor for magic structs, unimplementable in some languages. No real mode nor protected mode trampoline code needed either, which simplifies your build environment significantly. Example linker script provided for both x86_64 and AArch64. There are [https://gitlab.com/bztsrc/bootboot/tree/master/mykernel example kernels] written in [[C]], [[C++]], [[Pascal]], [[Ada]], [[Rust]] and [[Go]].

BOOTBOOT passes a configuration to your kernel in a newline separated, zero-terminated UTF-8 string. Each line contains a ''key=value'' pair, where only two keys, "screen" and "kernel" are reserved. You are free to use any other keys that your kernel wishes to use.

== Console ==

BOOTBOOT does not support VGA text mode, it sets up graphic mode regardless to the platform. You don't have to mess around with [[VESA]] or [[GOP]] any more, you will get the frame buffer ready to use. The example Hello World kernel provided with the BOOTBOOT source has a minimal puts() example, which outputs text on that frame buffer using [[PC Screen Font]] (same that Linux Console uses). You can also use the [[Scalable Screen Font]] library's ''ssfn_putc()''.

The loader also initializes the [[Serial_Ports|serial console]] for you with 115200 baud, 8 data bits, no parity and 1 stop bit. Your kernel can send debug messages from the start.

== How to Install ==

You can install BOOTBOOT yourself following the [[Bootable_Disk|bootable disk]] tutorial and using one of the provided mkboot.c utilities. But the simplest way is to use 
the multiplatform (Windows, MacOSX, Linux) disk image creator tool, [https://gitlab.com/bztsrc/bootboot/tree/master/mkbootimg mkbootimg] which is very similar to [[GRUB|grub-mkrescue]].

For input, you provide the contents of your initrd in a directory, and a JSON configuration describing what disk image you would like. The rest is taken care for you.

== There's more... ==

Besides of the pre-compiled boot loader binaries, BOOTBOOT also provides dependency free tools written in ANSI C to map GPT [[EFI System Partition]] into MBR, install boot sectors or send your ramdisk over serial line from another machine. The aforementioned Hello World kernel can be used as a skeleton for your kernel. Forget the boot process, you can start by implementing your kernel right away.

For all the features and available options, read the PDF documentation. It describes how to install an initial ramdisk with your kernel in it on a disk image, what are the exact memory mappings, which C structure is used to pass information to your kernel, how to tell bootstrap processor from application processors apart, etc.

== Threads ==
*[https://forum.osdev.org/viewtopic.php?f=2&t=33362 BOOTBOOT forum topic]

== External Links ==
*[https://gitlab.com/bztsrc/bootboot Source on Gitlab] the repository
*[https://gitlab.com/bztsrc/bootboot/raw/master/bootboot_spec_1st_ed.pdf BOOTBOOT Specification] and user's manual in PDF format
*[https://gitlab.com/bztsrc/bootboot/tree/master/mkbootimg mkbootimg] simple to use, dependency-free [[Bootable_Disk|bootable disk]] creator
*[https://gitlab.com/bztsrc/bootboot/tree/master/mykernel Example Hello World Kernels] with Makefile and minimal linker script
*Mykernel in [https://gitlab.com/bztsrc/bootboot/tree/master/mykernel/c C]
*Mykernel in [https://gitlab.com/bztsrc/bootboot/tree/master/mykernel/cpp C++]
*Mykernel in [https://gitlab.com/bztsrc/bootboot/tree/master/mykernel/rust Rust]
*Mykernel in [https://gitlab.com/bztsrc/bootboot/tree/master/mykernel/pas Pascal]
*Mykernel in [https://gitlab.com/bztsrc/bootboot/tree/master/mykernel/ada Ada]
*Mykernel in [https://gitlab.com/bztsrc/bootboot/tree/master/mykernel/go Go]

[[Category:Bootloaders]]
[[Category:ARM]]
[[Category:Raspberry Pi]]
[[Category:x86-64]]
[[Category:UEFI]]

[[de:BOOTBOOT]]
````

<a id="w004394"></a>

## W004394 — EFI System Partition

Bron-ID: W004394
Namespace: Main (0)
Revisie: 27953 | Datum: 2023-07-09T16:07:02Z
Bron: https://wiki.osdev.org/EFI_System_Partition
Revisie-URL: https://wiki.osdev.org/index.php?title=EFI_System_Partition&oldid=27953

````text
The EFI System Partition (ESP) is a dedicated partition on [[GPT]]. Usually a small one (32-256MiB), located at the beginning of the disk and its partition record at the beginning of the GPT.

== Identify ==
Unlike normal bootable partitions, which have the attribute flag bit 2 set, ESP is identified by its GUID, which is

 C12A7328-F81F-11D2-BA4B-00A0C93EC93B

== Format ==
In theory ESP can be formatted to any [[FAT]] file system, [[FAT12]]/[[FAT16|16]]/[[FAT32|32]]. In reality most firmware only support FAT32, and some platforms with limited resources may support [[FAT16]] too. [[FAT12]] is very rarely implemented, and it's safe to assume it won't work on a real hardware.

There's one notable exception, Apple machines (like iMac and MacBooks) use a special firmware which only accepts [[HFS+]] file system on the EFI System Partition.

== Access ==
In the EFI Shell, the ESP is the very first file system, denoting as ''FS0:''. [[GRUB]] and [http://elilo.sourceforge.net ELILO] recognize ESP, and they install their files there by default. Other boot loaders, like [http://www.rodsbooks.com/refind/ rEFInd] can only work if ESP exists.
From your application, you can use the [[Loading files under UEFI|EFI_SIMPLE_FILESYSTEM_PROTOCOL]] to access files on the ESP programmatically.

== Important Files on ESP ==
* FS0:\STARTUP.NSH - an EFI Shell script, similar to MS-DOS autoexec.bat
* FS0:\BOOTMGR.EFI - the EFI boot manager
* FS0:\BOOT\BOOTX86.EFI - the default x86_32 boot loader
* FS0:\BOOT\BOOTX64.EFI - the default x86_64 boot loader

To avoid filename clashes, you should not create files in the root directory of ESP. The specification mandates that root directory is for vendor directories, like FS0:\APPLE, FS0:\MICROSOFT etc. Some applications (like GRUB) just creates a one level directory without vendor parent directory. Feel free to create a directory for your own OS, and put your files there.

== See Also ==
=== Articles ===
* [[UEFI]]
* [[Bootable Disk]] - tutorial on how to create GPT

=== External Links ===
* [http://www.intel.com/technology/efi/ Intel EFI specifications et al.]
* [[wikipedia:EFI_System_Partition|Wikipedia: EFI System Partition]]

[[Category:Firmware]]
[[Category:UEFI]]
[[Category:Disks]]
````

<a id="w004395"></a>

## W004395 — Raspberry Pi 4

Bron-ID: W004395
Namespace: Main (0)
Revisie: 30869 | Datum: 2026-07-07T17:36:24Z
Bron: https://wiki.osdev.org/Raspberry_Pi_4
Revisie-URL: https://wiki.osdev.org/index.php?title=Raspberry_Pi_4&oldid=30869

````text

== Raspberry Pi 4 ==

This board is one of the [[Raspberry Pi]] models, released in 2019. It has 1G - 8G RAM, ARMv8 (AArch32 and AArch64) CPU, and VideoCore VI GPU.

== Hardware ==

CPU is a BCM2711, containing 4 Cortex-A72 cores. In Linux they appear as

<pre>
processor	: 0
model name	: ARMv7 Processor rev 3 (v7l)
BogoMIPS	: 108.00
Features	: half thumb fastmult vfp edsp neon vfpv3 tls vfpv4 idiva idivt vfpd32 lpae evtstrm crc32 
CPU implementer	: 0x41
CPU architecture: 7
CPU variant	: 0x0
CPU part	: 0xd08
CPU revision	: 3
</pre>

The graphics is done with a Video Core VI. This is backwards-compatible with Video Core IV from Broadcom.

== Device memory map ==

Taken from Linux /proc/iomem as root

<pre>
00000000-3b3fffff : System RAM
fd500000-fd50930f : pcie@7d500000
fd580000-fd58ffff : genet@7d580000
  fd580e14-fd580e1c : unimac-mdio.-19
fd5d2200-fd5d222b : thermal@7d5d2200
fe007000-fe007aff : dma@7e007000
fe007b00-fe007eff : dma@7e007b00
fe00a000-fe00a023 : watchdog@7e100000
fe00b200-fe00b3ff : dwc_otg
fe00b840-fe00b87b : mailbox@7e00b840
fe00b880-fe00b8bf : mailbox@7e00b880
fe100000-fe100113 : watchdog@7e100000
fe101000-fe102fff : cprman@7e101000
fe104000-fe10400f : rng@7e104000
fe200000-fe2000b3 : gpio@7e200000
fe201000-fe2011ff : serial@7e201000
  fe201000-fe2011ff : serial@7e201000
fe215000-fe215007 : aux@7e215000
fe300000-fe3000ff : mmcnr@7e300000
fe340000-fe3400ff : emmc2@7e340000
fe600000-fe6000ff : firmwarekms@7e600000
fe980000-fe98ffff : dwc_otg
fec00000-fec03fff : hub
fec04000-fec07fff : core0
fec11000-fec1101f : watchdog@7e100000
600000000-603ffffff : pcie@7d500000
  600000000-6000fffff : PCI Bus 0000:01
    600000000-600000fff : 0000:01:00.0
      600000000-600000fff : xhci-hcd
</pre>

== Devices as found by Linux, with addresses and Linux driver name ==
<pre>
fd500000.pcie    brcm-pcie
fd580000.genet    bcmgenet
fd5d2200.thermal    brcmstb_thermal
fe007000.dma    bcm2835-dma
fe007b00.dma    bcm2835-dma
fe00b840.mailbox    bcm2835_vchiq
fe00b880.mailbox    bcm2835-mbox
fe100000.watchdog    bcm2835-pm
fe101000.cprman    bcm2835-clk
fe104000.rng    iproc-rng200
fe200000.gpio    pinctrl-bcm2835
fe200000.gpiomem    gpiomem-bcm2835
fe201000.serial    uart-pl011
fe215000.aux    bcm2835-aux-clk
fe300000.mmcnr    mmc-bcm2835
fe340000.emmc2    sdhci-iproc
fe600000.firmwarekms    vc4_firmware_kms
fe980000.usb    dwc_otg
feb00000.hevc-decoder    argon-mem
feb10000.argon-local-intc    argon-mem
feb20000.h264-decoder    argon-mem
feb30000.vp9-decoder    argon-mem
</pre>

== Still to write into text ==

- Standard XHCI USB controller, in addition to the embedded IP block that was in the Pi 1-3. The USB-A connectors are run by the XHCI block now, while the charging port is on the IP block. Previously they were all on the IP block.

- Network devices have a mac address mapping to https://maclookup.app/macaddress/DCA632 - the raspberry pi foundation's own block. Not attached to pci-e, somehow magically part of SoC.

- WLAN controller is connected to mmc0 using SDIO.

- SD controller is now a SDHCI thing. Should work according to spec (except for [https://github.com/torvalds/linux/blob/master/drivers/mmc/host/sdhci-iproc.c#L188 this], min clock frequency to 200kHz instead of 100kHz).

CPU is similar to the one in the Rpi3, except with the A53 cores replaced in-situ with A72. Same startup logic, same usage. Also limited to max 4 cores per chip by design.

GPIOs are similar, but have more options: https://elinux.org/RPi_BCM2711_GPIOs

== See Also ==
* [[Raspberry Pi Bare Bones]]

[[Category:ARM]]
[[Category:AArch64]]
[[Category:Raspberry Pi]]
````

<a id="w004396"></a>

## W004396 — User:Mrwoggle

Bron-ID: W004396
Namespace: User (2)
Revisie: 24103 | Datum: 2019-11-15T16:10:27Z
Bron: https://wiki.osdev.org/User:Mrwoggle
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMrwoggle&oldid=24103

````text
I'm mr_woggle (mrwoggle on these forums, because the under square is somehow not allowed) If you like developing OSes, please link on: [https://github.com/robinkrens Github]. I think it is a great way to watch other people's progress!
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

<a id="w004397"></a>

## W004397 — QEMU Monitor

Bron-ID: W004397
Namespace: Main (0)
Revisie: 31017 | Datum: 2026-10-12T12:04:38Z
Bron: https://wiki.osdev.org/QEMU_Monitor
Revisie-URL: https://wiki.osdev.org/index.php?title=QEMU_Monitor&oldid=31017

````text
The '''QEMU monitor''' can be used to inspect and modify the VM state. For an overview of available comments, see the [https://qemu.weilnetz.de/doc/qemu-doc.html#pcsys_005fmonitor QEMU Manual]. This monitor is built-in and doesn't require an external debugger like [[GDB]]. Some common usage scenarios on how to use the [[QEMU]] monitor are listed below. 

==Running QEMU Monitor==
Adding the argument <source lang="bash" enclose="none">-monitor [dev]</source> will start the qemu monitor and start a bash-like shell at the device. For example:
<source lang="bash">
qemu -hda disk.img -monitor stdio  #start monitor at current shell
qemu -hda disk.img -monitor telnet:127.0.0.1:[7777] #start monitor at different PC
</source>

==Common usage==
===Registers===
<source lang="bash" enclose="none">info registers</source> will print out the current state of all registers. Besides global regisers (EAX and EIP) it also list information about segments selector and descriptor tables. Below is a snippet of the output:

<source lang="bash">
...
CS =0008 00000000 ffffffff 00cf9a00 DPL=0 CS32 [-R-]
DS =0010 00000000 000fffff 00c09300 DPL=0 DS   [-WA]
...
</source>

* ''CS =0008''
:Describes the segment selector. The code selector is using the second entry of the GDT (DS is using the third entry of the GDT)
* ''00000000 fffffff''
:The base and limit of the CS segment. Here the CS uses all of the available memory. If you have a look at DS segment, you will see that the limit is 0x000fffff
* ''00cf9a00 DPL=0 CS32 [-R-]''
:Flags of the segment. The first field show a summary of all flags combined. DPL, privilege level, can only run code using privilege level 1. CS32 means it is running in 32bit mode. In case you are running real mode, CS16 would show up. The last field is particularly interesting as it shows the access rights, [-R-], read-only 

===Memory===
Similar to GDB, <source lang="bash" enclose="none">x/Nx [addr]</source> will show N words  starting at address [addr]. For example, to see what is loaded in memory after ''0x7C00'' (bootloader):

<source lang="bash">
(qemu) x/12x 0x7C00
00007c00: 0x7d121688 0x898000bd 0x7d13bbec 0xe8000ee8
00007c10: 0x68bb00e1 0x0005e87d 0xeb008ee8 0x8a0eb4fe
00007c20: 0x74003c07 0x01c38307 0xf1eb10cd 0xb45260c3
</source>

To inspect an instruction at a certain memory location use <source lang="bash" enclose="none">i</source> instead. The format is similar: <source lang="bash" enclose="none">x/Ni [addr]</source>. For example, to see the an instruction at a certain place:
<source lang="bash">
(qemu) x/i 0x7C00
0x00007c00:  mov    %dl,(%esi)
</source>

'''Note:''' <source lang="bash" enclose="none">x</source> normally inspects a virtual or linear addresses. In case paging is set up, <source lang="bash" enclose="none">xp</source> can be used to assure to display the physical address. 

===Read I/O ports===
QEMU can also be used to read and write to I/O ports. For example, to check the values of the first PIC:

<source lang="bash">
(qemu) i/i
portb[0x0020] = 0x11
</source>

=== Test exception handlers ===
<source enclose="none">(qemu) nmi</source> is the most common way to do this, causing a non maskable interrupt. This is usually done to test the exception handles features that are used on all exceptions or done to make sure the exception handlers NMI specific features work.

==See Also==
[[GDB]]

==External Links==
* [https://qemu.weilnetz.de/doc/qemu-doc.html QEMU Manual]

[[Category:QEMU]]
````

<a id="w004398"></a>

## W004398 — Talk:Raspberry Pi 4

Bron-ID: W004398
Namespace: Talk (1)
Revisie: 23747 | Datum: 2019-07-04T16:29:18Z
Bron: https://wiki.osdev.org/Talk:Raspberry_Pi_4
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ARaspberry_Pi_4&oldid=23747

````text
Is this article going to be about bare bones development for the Raspberry Pi 4? If so, I think it should be integrated in [[ARM_RaspberryPi_Tutorial_C]]. I think the Linux specific parts should be scraped.
````

<a id="w004399"></a>

## W004399 — Joystick

Bron-ID: W004399
Namespace: Main (0)
Revisie: 23756 | Datum: 2019-07-13T20:35:16Z
Bron: https://wiki.osdev.org/Joystick
Revisie-URL: https://wiki.osdev.org/index.php?title=Joystick&oldid=23756
Doorverwijzing: Game port

````text
#REDIRECT [[Game port]]
````

<a id="w004400"></a>

## W004400 — PCI Local Bus Signals

Bron-ID: W004400
Namespace: Main (0)
Revisie: 27883 | Datum: 2023-07-09T04:28:11Z
Bron: https://wiki.osdev.org/PCI_Local_Bus_Signals
Revisie-URL: https://wiki.osdev.org/index.php?title=PCI_Local_Bus_Signals&oldid=27883

````text
=Prologue=

The PCI local bus, or PCI "Legacy" bus as it is called in common parlance, is a 32 or 64 bit bus capable of speeds from 33MHz to 533MHz, and it supports peripheral voltages of 3.3 andv 5v, though the bus itself specifies a 3.3v signaling environment. PCI peripherals may support 32 or 64 bit addressing transparently. Peripherals must support a minimum pinout of 47 pins for non-masters and 49 pins (+REQ and GNT for arbitration) for masters; but additional pins are specified which add support for features like JTAG, 64-bit addressing and data transfer, SMB I2C Management, IRQ pins and locked bus transactions.

The name "Local Bus" comes from the origins of the PCI bus, which are the IBM-PC ISA bus days, when the growing requirement for a high bandwidth connection to graphics cards caused vendors to experiment with the AGP bus, which was a wire connection that placed the graphics card close to the CPU, as opposed to being positioned behind the north or south bridge controller. From here, the PCI Local Bus's elder wizards (The PCI-SIG - PCI Special Interest Group) have carefully monitored its growth and expanded the specification, and enhanced its feature set. PCI found itself well positioned to then provide the locality of design required to support the bandwidth hungry consumer requirements imposed on new IBM-PC peripherals such as SCISI and Gigabit Ethernet IPs. Its Plug and Play software interface along with the care taken by PCI-SIG enabled it to present a single interface to software, even as the devices being added to its repertoire expanded rapidly.

This Wiki page is current with '''revision 3.0''' of the PCI specification. This wiki page does not take the PCI Express bus into account. This page is not meant to serve as an exhaustive reference for the PCI Local Bus specification, but it is meant to function as a condensed refresher for an individual who has already read the PCI Local Bus specification.

==Signals==

The signals on this page are written up from the perspective of a peripheral card, since this page is describing the the card pinout for PCI peripherals. PCI Signals are active low unless stated otherwise here. PCI signals are latched on the '''rising edge''' of CLK. Some signals are only latched when they '''qualify''' for latching. I.e, when they have a meaning for the current ongoing transaction type.

===Signal types===

{| class="wikitable"
|-
! Type
! Shortname (used throughout the docs)
! Description
|-
| Input
| in
| Input signals are flows from a bridge or other agent to the card
|-
| Output
| out
| Output signals are flows from the card to another agent.
|-
| Tristate
| t/s
| Tristate signals are '''bidirectional''' flows to and from the card, which must be driven only by one agent at a time (obviously). Tristate signals may be in high, low or floating (High-Z/High impedance) state.
|-
| Sustained Tristate
| s/t/s
| Sustained tristate signals are '''active low''' signals which must be driven high for at least one clock cycle after they are driven active. I.e, after an agent drives a s/t/s signal active (low), when it wishes to relinquish control of the pin, it must drive it high for one signal before letting it float.
|-
| Open Drain
| o/d
| Open Drain signals are pulled up into the floating state unless explicitly driven by an agent.
|}

===Signals===

====Core Signals====

Signals with the type "core" are part of the minimum card pinout which is required of all devices. Signals without the "core" keyword are optional card pinouts.

{|class="wikitable"
|-
! Name
! Type
! Relationship to CLK
! Description
|-
| CLK
| in, core
| N/A
| CLK is an input to every PCI device. All signal parameters including timing are defined relative to this signal. Valid clock speeds are those defined by the PCI and PCI-X specifications.
|-
| RST#
| in, core
| async
| Reset brings all '''PCI-specific''' registers, sequencers and signals to a consistent state. The effect of RST# on a device beyond sequencers and PCI configuration regs is implementation defined. Devices which can wake the system have additional RST# requirements. On the assertion of RST#, all pci output signals must be driven to their ''benign state''. REQ# and GNT# must be tristated. The central resource ''may'' drive AD, C/BE and PAR low to prevent floating, but '''not high'''. AD[63:32], C/BE[7:4] and PAR64 have special requirements when not connected (i.e, 64-bit card connected to a 32-bit host). Only devices which are required to boot the system will respond while in RST#, '''with the exception of configuration accesses'''.
|-
| AD[31:0]
| t/s, core
| sync
| Address and Data lines 0-32. After the assertion of FRAME#, the first latch of clock will treat these as address signals. This first latch is called the '''Address phase''', and during this latch, the C/BE[3:0] lines are latched and interpreted as an accompanying 4-bit command value. From the second latch of clock onward within a FRAME# active period, these pins when latched are interpreted as data bits. These subsequent latches are called '''Data phase''' latches, and during this latch, the C/BE[3:0] lines are latched and interpreted as byte-enables which state which of the 32 Data lines contain valid data -- each byte-enable pertains to one group of 8 lines of data.
|-
| C/BE#
| t/s, core
| sync
| C/BE is a multiplexed signal - it can at any given moment either be in use as either a '''Command''' which accompanies the address phase or a '''Byte-Enable''' which accompanies the data phase.
|-
| PAR#
| t/s, core
| sync
| Parity is invalid during the address phase, and becomes valid during the first data phase clock cycle and for all subsequent data phase cycles until FRAME# is deasserted, as long as TRDY# or IRDY# are also asserted.
|-
| FRAME#
| s/t/s, core
| sync
| Cycle Frame is asserted at the start of a transaction and is held active for the duration of it.
|-
| IRDY#
| s/t/s, core
| sync
| Initiator ready is used as a flow control mechanism. When the master is reading, it asserts IRDY# to state that it is ready to receive more data. When the master is writing, it asserts #IRDY to indicate to the slave that there is more data available for it to read.
|-
| TRDY#
| s/t/s, core
| sync
| Target ready is used as a flow control mechanism. When the slave is being written to, if it has not yet asserted TRDY#, the master interprets the current cycle as a WAIT cycle and the master will relatch the same data on the next cycle.
|-
| STOP#
| s/t/s, core
| sync
| This indicates that the slave is requesting that the master stop the current transaction.
|-
| LOCK#
| s/t/s, core
| sync
| This guarantees a logically atomic transaction comprised of multiple transactions each with their own address and data phases. Lock is meant to be used by bridges. Non-bridge devices which assert LOCK# are not conformant with the PCI spec. The PCI-SIG is actively attempting to phase out and eliminate the LOCK# signal.
|-
| IDSEL
| in, core
| sync
| "Initialization Device Select": This is used as a chipselect during configuration read and write transactions.
|-
| DEVSEL#
| s/t/s, core
| sync
| When actively driven, this signal indicates that the asserting device recognizes itself as being the target of the current address phase.
|-
| REQ#
| t/s, core, MASTERS ONLY
| sync
| Request is a signal to the '''central resource''' which forms part of the arbitration protocol. All devices have logically separate REQ# lines to the arbiter.
|-
| GNT#
| t/s, core, MASTERS ONLY
| sync
| Grant is an indicator to the device from the '''central resource''' that is has control over the bus. All devices have logically separate GNT# lines from the arbiter.
|-
| PERR#
| s/t/s
| sync
| Parity error: This is used to signal parity errors in data during any PCI transaction other than a "Special Cycle". 
|-
| SERR#
| o/d
| sync
| System Error: this is for reporting address and data parity errors on the "Special Cycle" command, or any other system error with a "catastrophic result".
|-
| INTA#, INTB#, INTC#, INTD#
| o/d
| '''async'''
| The interrupt lines must be used in order, from A to D. A device with only one IRQ must assert INTA#, and it may not assert INTB#/C#/D#, and so on. A single PCI function (bus/slot/function) may not generate IRQs on more than one line (i.e, only one IRQ per function). A device with multiple IRQs may wire each function to whichever INT# signal it wishes so long as that INT# signal meets the constraints outlined in the first sentence of this cell. PCI requires that the motherboard designer must connect every INT# line that is implemented to the IRQ subsystem on the motherboard. PCI devices must assume that they will be sharing an IRQ line on the motherboard's IRQ controller, and all PCI devices must be ready to play nicely with IRQ sharing, including with other functions within their own device package.
As you can see, PCI interrupt signals are '''active low, level triggered''' (open drain driven state is maintained by the device until the IRQ is acked).
|}

Devices must ignore their GNT# lines while RST# input is asserted. The arbiter must ignore all REQ# lines while RST# is asserted.

====Non-core signals====

These signals are extensions and they are not required of cards.

{|class="wikitable"
|-
! Name
! Type
! Relationship to CLK
! Description
|-
| PRSNT[1:2]#
| in
| Unknown
| The present signals are meant only for add-in cards and are not required for all devices. It indicates when a card has been connected and what its voltage/power requirements are.
|-
| CLKRUN#
| in, o/d, s/t/s
| Unknown
|
|-
| M66EN#
| in
| unknown
| This input to the device states whether the bus is being driven at 33 or 66 MHz.
|-
| PME#
| o/d
| async
| This is used by devices to request a change in its own, or in the system's power state. Devices may not assert this signal without software permitting/enabling them first. Once asserted by the device, the signal is sticky until software clears it.
|-
| 3.3Vaux
| in
| unknown
| this is an additional 3.3V power input for add-in cards which enables them to participate in power management events when software has turned off main power to the device.
|-
| AD[63:32]
| t/s
| sync
| These pins are the top 32 bits of a 64-bit transaction. They work the same as AD[32:0] except that their FRAME# signal is the REQ64# signal instead.
|-
| C/BE[7:4]
| t/s
| sync
| These only act as command signals during the address phase when using the Dual Address generation feature. During the data phase they act as byte enables on the high 32 bits of the latched data for a 64 bit transaction.
|-
| REQ64#
| s/t/s
| sync
| This, like FRAME# is asserted by the currently GNT#-ed bus master and it indicates that a frame is in progress.
|-
| ACK64#
| s/t/s
| sync
| When this signal is driven by the device which has decoded the address of the address phase as its own, it indicates to the master that the device is willing to transfer data using 64-bits. It has the same timing and implicatins as DEVSEL#.
|-
| PAR64
| t/s
| sync
| This is the upper DWORD parity signal which has the same function as the PAR signal, but for the AD[63:32] pins.
|-
| TCK, TDI, TDO, TMS, TRST
| in, in, out, in in
| unknown
| These are JTAG pins for interacting with the card.
|-
| SMBCLK, SMBDAT
| o/d
| unknown
| These are I2C pins for the SMBUS standard.
|}

[[Category:Buses]]
[[Category:PCI]]
[[Category:PCI Local Bus]]
````

<a id="w004401"></a>

## W004401 — User:Gravaera/ARM SMMU versions 1 and 2

Bron-ID: W004401
Namespace: User (2)
Revisie: 23788 | Datum: 2019-08-08T03:49:23Z
Bron: https://wiki.osdev.org/User:Gravaera/ARM_SMMU_versions_1_and_2
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FARM_SMMU_versions_1_and_2&oldid=23788
Doorverwijzing: ARM SMMU versions 1 and 2

````text
#REDIRECT [[ARM SMMU versions 1 and 2]]
````

<a id="w004402"></a>

## W004402 — User:Gravaera/ARM Generic Timers

Bron-ID: W004402
Namespace: User (2)
Revisie: 23791 | Datum: 2019-08-08T03:51:58Z
Bron: https://wiki.osdev.org/User:Gravaera/ARM_Generic_Timers
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FARM_Generic_Timers&oldid=23791
Doorverwijzing: ARMv7 Generic Timers

````text
#REDIRECT [[ARMv7 Generic Timers]]
````

<a id="w004403"></a>

## W004403 — Talk:UEFI App Bare Bones

Bron-ID: W004403
Namespace: Talk (1)
Revisie: 25171 | Datum: 2020-09-17T15:41:39Z
Bron: https://wiki.osdev.org/Talk:UEFI_App_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AUEFI_App_Bare_Bones&oldid=25171

````text

== Hello World ==
Without wanting to overly criticise the content of the article, would 'UEFI Hello World' be a better title for this article? The article, while very relevant to operating system development, has very little in common with the theme of the 'Bare Bones' series of articles contained on this site. A more relevant 'UEFI Bare Bones' article could be written as an addendum to one of the other bare bones articles that more completely demonstrates how to create a UEFI bootloader.
I recently came upon this article while searching for a basic introduction to implementing UEFI compatibility into a hobby OS. While it does provide a very good introduction to the topic, I feel that it's overly general. I understand the pedagogical value in not just spoonfeeding the audience copy+pasteable answers to complex topics like developing bootloader, but the topic of UEFI in the context of osdev is much more specialised and as a result this title is a bit misleading.
--[[User:Ajxs|Ajxs]] 08:31, 11 August 2019 (CDT)

== GNU-EFI and Toolchain ==
I'm not sure why the author of the article has decided not to link against full GNU-EFI. The reason given is "To avoid bloat and unnecessary dependencies on the rest of gnu-efi", this seems really unnecessary. It would seem simpler for the purpose of a 'Hello World' style app to take the path of least resistance and link against GNU-EFI in its complete form rather than editing its headers and selectively copying and including it in the source tree. This seems like a bad practice to me.
Is the author using a cross compiler to avoid assuming that the user is using <tt>x86_64-elf-gcc</tt>? I've been successful using <tt>x86_64-elf-gcc</tt> and using <tt>objcopy</tt> to create the final binary in <tt>efi-app-x86_64</tt> format. This seems like a simpler approach. If there's a good reason not to do this, I would appreciate being told!
--[[User:Ajxs|Ajxs]] 08:31, 11 August 2019 (CDT)

:GNU-EFI performs some questionable stuff to make ELFs run on UEFI, which expects a specific type of a PE (or related format, don't quote me on the exact name). It does this by compiling everything as an ELF and modifying it to appear as a PE, including forging relocations (IIRC). This is just a horrible hack, needless to say, especially given that you could just use a mostly appropriate compiler, as done in the article (on clang, you would just use the x86_64-pc-win32-coff target and the appropriate linker flags). The point of using this specific cross-compiler is to avoid this mess by actually producing an appropriate file. If you want your kernel to be an ELF, just have a UEFI loader that loads the kernel (use could use this to position your kernel at the -2 GiB mark) and passes control to it. However, personally I just steal [[User:No92/UEFI_Bare_Bones|Zircon's UEFI headers and work with them]] to load my kernel. [[User:No92|no92]] 10:35, 11 August 2019 (CDT)

::I'm not sure I understand what you mean here. You ( and the article's author ) refer to GNU-EFI as having/being some kind of a toolchain/build system. GNU-EFI is just a library is it not? If you compile the main EFI executable using <tt>x86_64-elf-gcc</tt> and link against GNU-EFI, the output will be a completely valid ELF file. Using <tt>readelf/objdump</tt> on the output confirms this. GNU-EFI has no idea about the target executable format. My build pipeline involves compiling my bootloader as an ELF static lib that links statically against GNU-EFI, and then copying the resulting <tt>.so</tt> to a PE executable. This seems to work absolutely fine. I'm ready to be corrected, however. Are you referring to using <tt>objcopy --target=efi-app-x86_64</tt> to create a PE executable from the ELF file, or am I misunderstanding something? 
::As for creating an ELF loader to load the kernel from within the bootloader, I had assumed that this was a standard approach. I've seen a few examples online where people have compiled their kernel into the same EFI executable as the bootloader, but I thought this was unusual. --[[User:Ajxs|Ajxs]] 01:27, 12 August 2019 (CDT)

:::[https://github.com/vathpela/gnu-efi/blob/master/README.gnuefi#L153 There's more going on than just linking in a static library] (e.g. forging relocations, providing a linker script that they say apps have to use). The whole point of this monstrosity is to create an UEFI executable, which you could just do with a header-only library and the correct compiler, instead of relying on this massive hack to make ELFs run on UEFI. Adding points of failure in code that is being compiled incorrectly just isn't a good idea to begin with, with the alternative way being simpler (and, this may be a point too, doesn't introduce additional dependencies on make, and therefore, extra steps). [[User:No92|no92]] 09:51, 12 August 2019 (CDT)

::::Thanks for pointing that out! I see your point. That does sound pretty fragile. Without researching more into it I had assumed that objcopy was doing all of the heavy lifting here. Once I'm done with reading and working through all of this I might update the tutorial to point this out, since I don't think it's immediately obvious. Thanks again.
````

<a id="w004404"></a>

## W004404 — Libc

Bron-ID: W004404
Namespace: Main (0)
Revisie: 23811 | Datum: 2019-08-12T08:34:19Z
Bron: https://wiki.osdev.org/Libc
Revisie-URL: https://wiki.osdev.org/index.php?title=Libc&oldid=23811
Doorverwijzing: C Library

````text
#REDIRECT [[C Library]]
````

<a id="w004405"></a>

## W004405 — User:Jflopezfernandez

Bron-ID: W004405
Namespace: User (2)
Revisie: 23822 | Datum: 2019-08-15T22:03:05Z
Bron: https://wiki.osdev.org/User:Jflopezfernandez
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJflopezfernandez&oldid=23822

````text
You can reach me at jflopezfernandez at gmail dot com.
````

<a id="w004406"></a>

## W004406 — User talk:Jflopezfernandez

Bron-ID: W004406
Namespace: User talk (3)
Revisie: 23823 | Datum: 2019-08-15T22:05:08Z
Bron: https://wiki.osdev.org/User_talk:Jflopezfernandez
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AJflopezfernandez&oldid=23823

````text
If you find I've made a mistake in an edit or have given incorrect information, please do let me know; I'm always looking to keep learning.
````

<a id="w004407"></a>

## W004407 — User:Arcun

Bron-ID: W004407
Namespace: User (2)
Revisie: 23828 | Datum: 2019-08-24T23:55:04Z
Bron: https://wiki.osdev.org/User:Arcun
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AArcun&oldid=23828

````text
I'm ARCUN, an esoteric programmer who is trying to learn how to make an OS to use on a CPU that will be implemented on an FPGA. I'm also trying to introduce some unconventional aspects into a new OS so as to create an esoteric OS.
````

<a id="w004408"></a>

## W004408 — Communications

Bron-ID: W004408
Namespace: Main (0)
Revisie: 30158 | Datum: 2026-03-09T08:41:58Z
Bron: https://wiki.osdev.org/Communications
Revisie-URL: https://wiki.osdev.org/index.php?title=Communications&oldid=30158

````text
= Introduction =

There are numerous mechanisms, techniques and protocols for getting data from one place to another, and many have a long history, an established implementation, and industry-standard terms. This page tries to enumerate the ones used for computing, so therefore what an Operating System may need to provide drivers and modules for.

To provide some sort of order, this page first differentiates between the most basic of implementations: how many "wires" are used simultaneously to send data.

* "Parallel" implies that there are sufficient data wires side-by-side to send a single datum (byte, word or whatever) at once, with extra wires for control;
* "Serial" implies that a single datum needs to be split into multiple parts first - usually the individual bits - and then sent over a smaller number of wires (as few as one). There may or may not be control wires as well.

Within the above two categories, the different techniques are listed in approximate complexity order. They start by differentiating themselves from other techniques - which is sometimes only subtle - and then usually link to other Wiki pages dedicated to the topic.

== Common Terms ==

Many terms are used in the Communications arena, and sometimes they're used ambiguously. The following are the usual definitions of these terms:

=== Communications ===
; Link : The connection between two devices. This could be multiple wires in a cable, or even wirelessly over a radio link.
; Channel : A single path on a link, carrying one stream of data. A link may comprise of multiple channels; the usual number is two, one in each direction.

=== Characteristics ===
; Synchronous : '''1.''' Synchronous communications implies that the data is sent in lock-step with a clock, often provided on another wire. The receiver can notice the change of state of the clock, and "sample" the data wire/s during defined transitions (e.g. rising edge, falling edge, or both).<br>'''2.''' Synchronous communications often implies "continuous": there is always data, but that data may be "filler" if there's nothing to communicate.
; Asynchronous : '''1.''' Asynchronous communications implies that each side maintains its own clock, and sends data at a predetermined rate (often called "baud rate").<br>'''2.''' Asynchronous communications often implies "at any time": there is usually "dead space" on the link, and then one side or the other will burst into life, transmit the data, then go idle again.

=== Directions ===
; Simplex : Data can only ever flow over the link in one direction.
; Duplex : Data can flow over the link in both directions.
; Full-Duplex : Data can flow over the link in both directions simultaneously.
; Half-Duplex : Data can flow over the link in either direction, '''''BUT''''' only in one direction at a time.
::A perfect example is the walkie talkie: only one side can transmit at a time, or else neither side will hear the other - hence the tradition of saying "Over!" just before releasing the Press-To-Talk (PTT) button.

= Parallel Communications =

== Bus ==

== Parallel Port ==

== Small Computer System Interface (SCSI) ==

== Peripheral Component Interconnect (PCI) ==

=== PCI Express (PCIe) ===
Although this ''looks'' to the computer as a (nearly) standard PCI bus, strictly speaking this is a Serial interface. At the lowest (electrical) level the data is sent over a single channel instead of a parallel bus, reducing the pinout and allowing for a simpler, thus faster interface. To further increase speed, PCIe supports "lanes", which are actually parallel PCIe data channels with only one set of control wires - back to being parallel!

= Serial Communications =

== General Purpose I/O (GPIO) ==

== 1-Wire Interface ==

The (misnamed) one-wire interface needs a second wire: the ground. But it does use just the one wire for each direction of communications, by requiring both sides to use a transmitter that can be electrically turned off (tri-state or open drain). By each side connecting both their transmitter and receiver to that one wire, then either side can transmit - just not both at the same time (half-duplex).

== Single-Wire Protocol (SWP) ==

The (misnamed) Single-Wire Protocol needs a second wire: the ground. But it does use just the one wire for each direction of communications - simultaneously. In the Master-to-Slave direction, the Master communicates HIGHs and LOWs with differently-timed pulses. The Slave communicates HIGHs and LOWs by applying a load (or not) during those pulses. The Master detects the amount of current drawn by the Slave to determine what state the Slave is transmitting.

== Inter-Integrated Circuit (I&sup2;C) ==
This is often spelled "I2C" for convenience (pronounced "Eye Two See"), or occasionally "IIC", but the correct name comes from the idea that the two "I"s look like they multiply together, hence the pronunciation "Eye Squared See".

The I&sup2;C cable has two signals, plus ground: a Clock signal (SCK) that is always generated by the Master; and a Data signal (SDA) that is generated by whichever device is currently the transmitter - either the Master commanding or requesting something, or the Slave replying.

Note that multiple Slaves can be connected to the same I&sup2;C bus simultaneously. Part of the protocol is that the Master sends the address of the Slave that it is communicating to: only that Slave should respond.

== Serial Peripheral Interface (SPI) ==

The standard SPI cable has four signals, plus ground:
* A Master-driven Clock (SCK) to clock data into and out of the Slaves;
* A Master-driver Slave Select (SS) to select and activate which Slave is being commanded;
* The Master-Out/Slave In line (MOSI) data line; and
* The Master-In/Slave-Out line (MISO) data line.

The Master uses SCK to clock out its commands on MOSI while ''simultaneously'' clocking in the Slave's (previous) response on MISO.

The naming makes it obvious which direction the signal flows - but not so obvious is that there can be multiple Slaves. If there are, they can either be on their own individual SS lines (meaning that the Master needs a dedicated SS line for each of them), or on a single SS line.

To allow the latter (since all Slaves would be activated simultaneously), the MOSI/MISO lines need to be connected as a cascaded ring. The Master sends out enough information for ALL the devices in the correct order, while receiving the responses from the ''last'' command in the same order.

* The first bits sent out by the Master are for the ''last'' device in the chain, and pass through each device before reaching their destination; while the last bits sent are for the first device. Only when SS is dropped is the received data that's still in the Slaves' buffers "actioned" by the Slaves.
* To receive the responses, the Master again raises SS to all the devices, and clocks the Slaves' responses in. Again, the first device's response passes through all the other devices before arriving at the Master as the ''last'' data received.

=== Dual SPI ===

After SPI had been used for a while, it became apparent that the normal usage was for the Master to send out its command while ''ignoring'' the previous response, and then send out a "Do Nothing" command while clocking in the response to the original command. Although this was less efficient of the bandwidth available, it was far easier for programmers to think about - especially if the program needed the results of the previous command before it could decide what the next command should be.

That meant that the dedicated Input and Output lines were less efficient than a different model: keep the two data lines, but make them ''bidirectional'', so that they were ''both'' used to send two bits of the command or two bits of the response. This meant that the same amount of data could be transferred in half the number of clocks.

=== Quad SPI ===

The "Quad" in the name is because Quad SPI devices have four data lines rather than the two of SPI and Dual SPI. It can still be used in original SPI or Dual SPI modes - the last two data lines are ignored - but in full Quad SPI mode, commands and responses can be sent four bits at a time rather than two or one.

== Serial Port ==
Most people who program computers know what a serial port is, since that's an easy way for the program to communicate in real time to the operator. The most common version is that used for the [[Serial Port|COM port]] on the PC: a (now) 9-pin connector that has a Ground, Receive (RX) and Transmit (TX) pin, plus a "baud rate" setting you've got to get right to communicate anything!

But a generic port that can be simply connected to an external device is often much more complex that this, yet still goes by the name "Serial Port".

=== RS-232 / RS-422 / RS-485 ===

These terms define the electrical standard used on the wires for a serial port.

=== Comm Port ===

=== Synchronous Port ===

==== HDLC ====

==== X.25 ====

== Controller Area Network (CAN) ==

This is often used in vehicles, connecting sensors and switches to the Engine Management Computer, Body Control Module and dashboard gauges and lights. All the devices are connected to the same cable, and each broadcast packets of information for any other device to receive, interpret, and act on (change something else, or update a display). Using the one cable reduces the number of dedicated cables required, which saves a lot of weight, but it means that each device needs to be able to turn off its transmitter, and to be intelligent enough to decode the packets of information.

== MilStd-1553 Bus ==

This is often used in aircraft, connecting sensors and switches to the Flight Control Computer and instrument clusters. All the devices are connected to the same cable, and each broadcast packets of information for any other device to receive, interpret, and act on (change something else, or update a display). For redundancy and safety, typically two separate cable runs are connected to each device. Using only two cables reduces the number of dedicated cables required, which saves a lot of weight, but it means that each device needs to be able to turn off its transmitters, and to be intelligent enough to decode the packets of information.

== Universal Serial Bus (USB) ==

== Ethernet ==
This is arguably the most complicated Serial interface, despite it essentially only having two data channels. It's the protocol/s that are communicated on top of this electrical protocol that complicate things: most commonly the whole TCP/IP suite used by the Internet.

=== Internet Protocol (IP) ===

==== IP version 4 (IPv4) ====

==== IP version 6 (IPv6) ====

=== UDP ===

=== TCP ===

=== HTTP, HTTPS, E-mail, ... ===
[[Category:Theory]]
````

<a id="w004409"></a>

## W004409 — User:Ajxs/uart

Bron-ID: W004409
Namespace: User (2)
Revisie: 24080 | Datum: 2019-11-06T03:51:01Z
Bron: https://wiki.osdev.org/User:Ajxs/uart
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAjxs%2Fuart&oldid=24080

````text

````

<a id="w004410"></a>

## W004410 — Talk:Tail Recursion and Tail Call Optimization

Bron-ID: W004410
Namespace: Talk (1)
Revisie: 27026 | Datum: 2022-04-02T23:31:42Z
Bron: https://wiki.osdev.org/Talk:Tail_Recursion_and_Tail_Call_Optimization
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ATail_Recursion_and_Tail_Call_Optimization&oldid=27026

````text
Hey, I guess my question is, if this was incomplete, why not keep it in your own personal namespace? I could be misunderstanding the status and direction of this work, though. [[User:Gravaera|Gravaera]] 23:30, 2 September 2019 (CDT)

== wtf ==
''Why make the page in the first place?'' --[[User:Superleaf1995|Superleaf1995]] 18:31, 2 April 2022 (CDT)
````

<a id="w004411"></a>

## W004411 — User:Ajxs/quotes

Bron-ID: W004411
Namespace: User (2)
Revisie: 23899 | Datum: 2019-09-03T22:48:15Z
Bron: https://wiki.osdev.org/User:Ajxs/quotes
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAjxs%2Fquotes&oldid=23899

````text
A curated list of noteworthy programming quotes.



"When I'm working on a problem, I never think about beauty. I think only how to solve the problem. But when I have finished, if the solution is not beautiful, I know it is wrong." - '''Richard Buckminster Fuller'''
----

"When you are dealing with things that are measured (speed, distance, weight etc.) rounding is OK as the accuracy of the original measurement is probably less than the accuracy of your calculation, and users accept that the final number is still an estimate even if its a good estimate.
When you are dealing with things that are counted (money, people, ICBMs) rounding errors are a disaster. There is always an arithmetically correct answer, and, often the accuracy of this answer is enshrined in regulation, treaties and tax law. Providing a slightly wrong number gets you a free ticket into the Kafkaesque world of auditors, tax collectors and government inspectors; you may never escape with your sanity intact." - '''[https://softwareengineering.stackexchange.com/a/224867/288044 James Anderson]'''
----

"A good scientist is a person with original ideas. A good engineer is a person who makes a design that works with as few original ideas as possible. There are no prima donnas in engineering." - '''Freeman Dyson'''
----

"Any fool can write code that a computer can understand. Good programmers write code that humans can understand." - '''Martin Fowler'''
----

"There are two ways of constructing a software design: One way is to make it so simple that there are obviously no deficiencies, and the other way is to make it so complicated that there are no obvious deficiencies." - '''Tony Hoare'''
----

"By relieving the brain of all unnecessary work, a good notation sets it free to concentrate on more advanced problems, and in effect increases the mental power of the race. Before the introduction of the Arabic notation, multiplication was difficult, and the division even of integers called into play the highest mathematical faculties. Probably nothing in the modern world would have more astonished a Greek mathematician than to learn that...a large proportion of the population of Western Europe could perform the operation of division for the largest numbers. This fact would have seemed to him a sheer impossibility... Our modern power of easy reckoning with decimal fractions is the almost miraculous result of the gradual discovery of a perfect notation." - '''Alfred North Whitehead'''
----

"I do not feel obliged to believe that the same God who endowed us with sense, reason and intellect has intended for us to forgo their use." - '''Galileo'''
----

"For those laughing at COBOL, IMHO, it is like laughing at the Egyptian Pyramids, they are there from 5000 years and they still will be there in next 5000 years, while today's "hello world" housing needing 24 controls to work will be deleted, replaced, forgotten next month." - '''[https://softwareengineering.stackexchange.com/questions/247/is-cobol-still-worth-learning Nihar More]'''
----

"The CIA wants you to think it's nothing more than if/then/else..." - '''Terry A Davis''', on C's switch statement
----

"If GRUB is so hard to use, why do people ITT keep acting like writing a bootloader is a litmus test for programming ability?
Pick one guys, either GRUB is too hard or GRUB users are too dumb to write an OS." - '''[https://forum.osdev.org/viewtopic.php?f=1&t=33769 StudlyCaps]'''
----
````

<a id="w004412"></a>

## W004412 — File:Soso-gears.png

Bron-ID: W004412
Namespace: File (6)
Revisie: 23903 | Datum: 2019-09-07T23:14:49Z
Bron: https://wiki.osdev.org/File:Soso-gears.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ASoso-gears.png&oldid=23903

````text

````

<a id="w004413"></a>

## W004413 — File:Soso-lua.png

Bron-ID: W004413
Namespace: File (6)
Revisie: 23904 | Datum: 2019-09-07T23:16:32Z
Bron: https://wiki.osdev.org/File:Soso-lua.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ASoso-lua.png&oldid=23904

````text

````

<a id="w004414"></a>

## W004414 — User:Superleaf1995

Bron-ID: W004414
Namespace: User (2)
Revisie: 26879 | Datum: 2022-01-23T20:04:47Z
Bron: https://wiki.osdev.org/User:Superleaf1995
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASuperleaf1995&oldid=26879

````text
Hello I'm Superleaf1995, a C and Assembly coder that has build a lot of MS-DOS programs.

Userpages:
* [[User:Superleaf1995/Explaining the architectures|Explaining the architectures]]
* [[User:Superleaf1995/AFPFS|Arbitrary File Position File System]]
* [[User:Superleaf1995/PCI in assembly|Code of PCI article translated into Intel Assembly]]
* [[User:Superleaf1995/lowFS|lowFS specification]]
* [[User:Superleaf1995/Compile GCCMVS under MVS3.8j|Compile GCCMVS under MVS3.8j]]

Send a happy face to COM1:
<source lang="c">
typedef int i;typedef void v;typedef char c;i z(){return inb(0x3F8+5)&32;}w(c a){while(!z()){}outb(0x3F8,a);}u(){w(':');w(')');}
</source>

== 390 Bits ==

stack preparation for thread switching:
<source lang="c">
/* follows MVS calling convention (used by gccmvs) */
void SchedPrepareThreadStack(struct thread* thread)
{
    /* R13 is used as a stack pointer, now we have to setup a few things up */
    thread->context.r15 = (unsigned int)thread->stack + stack_size;

    /* stack+76 should point to stack+180 (because this would be the next frame!) */
    *((uint32_t *)(&((uint8_t *)thread->stack)[76])) = (&((uint8_t *)thread->stack)[180]);
    *((uint32_t *)(&((uint8_t *)thread->stack)[8])) = (&((uint8_t *)thread->stack)[180]);

    /* Set backchain to 0 for stack unwinding */
    *((uint32_t *)(&((uint8_t *)thread->stack)[4])) = NULL;
    *((uint32_t *)(&((uint8_t *)thread->stack)[8])) = NULL;
    return;
}
</source>

stack unwinding:
<source lang="c">
int DbgUnwindStack(cpu_context *ctx_frame)
{
    extern uint32_t __stack[1024]; /* your kernel stack*/
    uint8_t *frame = (uint8_t *)ctx_frame->r13;

    /* Corrupt stack pointer? */
    if((ptrdiff_t)frame < (ptrdiff_t)&__stack) {
        KeDebugPrint("Potentially corrupt stack, (SP=%x)\r\n", (unsigned int)frame);
        frame = (uint8_t *)&__stack;
    }

    /* Pointer to top of the stack */
    while(frame != NULL) {
        /* R14 is stored on frame+12; and holds the address of the return point of the caller */
        uint32_t retaddr = *((uint32_t *)&frame[12]);
        struct DebugSymData *retsym = DbgGetSymbol((void *)retaddr);
        /* And R15 is also stored on frame+16, and contains the branching/base address of the callee */
        uint32_t calladdr = *((uint32_t *)&frame[16]);
        struct DebugSymData *callsym = DbgGetSymbol((void *)calladdr);

        if(retsym != NULL) {
            KeDebugPrint("RET=(%i=%s:%i)\r\n", (unsigned int)retaddr, retsym->name, (int)((ptrdiff_t)retaddr - (ptrdiff_t)retsym->addr));
        }

        if(callsym != NULL) {
            KeDebugPrint("CALL=(%x=%s:%i)\r\n", (unsigned int)calladdr, callsym->name, (int)((ptrdiff_t)calladdr - (ptrdiff_t)callsym->addr));
        }

        /* Get the forward chain */
        KeDebugPrint("%p\r\n", (unsigned int)frame);
        frame = *((uint8_t **)(&frame[8]));
    }
    return 0;
}
</source>
````

<a id="w004415"></a>

## W004415 — User:Superleaf1995/UDOS

Bron-ID: W004415
Namespace: User (2)
Revisie: 27045 | Datum: 2022-04-20T23:23:39Z
Bron: https://wiki.osdev.org/User:Superleaf1995/UDOS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASuperleaf1995%2FUDOS&oldid=27045

````text
{{Infobox Project
|name=UDOS
|image=uDOS.png
|developers=[[User:Superleaf1995|Superleaf1995]]
|kerneltype=Time shared exokernel
|platform=S/370, S/370-XA, S/390, zEC12, zBC12, z114, z196, z10, z9 ([https://en.wikipedia.org/wiki/IBM_Z S390 and z/Arch families])
|taskmodel=4-level round robin
|language=[[Assembly]] ([[GAS|GAS Syntax]]), [[C]]
|license=Unlicense (Public domain)
}}

''UDOS'' is a small OS made for mainframes. It has an integrated console for executing commands and consists of the normal things an OS has: Memory management, VFS, Queued I/O commands, Disk drivers, Filesystem drivers, Text printing & Input processing. All of that but in the mainframe world.

It also supports the equivalent of SYSCALL, with it's own ABI.
````

<a id="w004416"></a>

## W004416 — Detecting Raspberry Pi Board

Bron-ID: W004416
Namespace: Main (0)
Revisie: 30289 | Datum: 2026-03-16T18:59:11Z
Bron: https://wiki.osdev.org/Detecting_Raspberry_Pi_Board
Revisie-URL: https://wiki.osdev.org/index.php?title=Detecting_Raspberry_Pi_Board&oldid=30289

````text
There are several Raspberry Pi boards, all shipped with an [[ARM_Overview|ARM SoC]], but each has different memory addresses.

= Detecting the board =
In order to detect which board you have, you have to read the MIDR_ELx system register. The manufacturer is encoded in bits [31:24], and the model in the so called PartNum in the bits [15:4]. PartNum is implementation defined, meaning each manufacturer are free to assign values as they like.
{| class="wikitable"
|-
! Board
! AArch64
! PartNum
! MMIO base address
|-
| Rpi1 / Rpi Zero
| {{No}}
| 0xB76
| 0x20000000
|-
| Rpi2
| {{No}}
| 0xC07
| 0x3F000000
|-
| Rpi3
| {{Yes}}
| 0xD03
| 0x3F000000
|-
| Rpi4
| {{Yes}}
| 0xD08
| 0xFE000000
|-
|Rpi Zero 2
|{{Yes}}
|0xD03
| 0x3F000000
|}


Kernels compiled for [[AArch32]] mode can run on all boards, and they are loaded into memory at address 0x8000. The Raspberry Pi 3 and upwards also supports [[AArch64]], loaded into 0x80000, and uses a different Assembly to read the same system register. Regardless to mode and load address, the MMIO base address is tied to the board.
<syntaxhighlight lang="c">
#include <stdint.h>

void _start()
{
    uint32_t reg;
    uint32_t *mmio_base;
    char *board;

    /* read the system register */
#if __AARCH64__
    asm volatile ("mrs %x0, midr_el1" : "=r" (reg));
#else
    asm volatile ("mrc p15,0,%0,c0,c0,0" : "=r" (reg));
#endif

    /* get the PartNum, detect board and MMIO base address */
    switch ((reg >> 4) & 0xFFF) {
        case 0xB76: board = "Rpi1"; mmio_base = 0x20000000; break;
        case 0xC07: board = "Rpi2"; mmio_base = 0x3F000000; break;
        case 0xD03: board = "Rpi3"; mmio_base = 0x3F000000; break;
        case 0xD08: board = "Rpi4"; mmio_base = 0xFE000000; break;
        default:    board = "????"; mmio_base = 0x20000000; break;
    }
}
</syntaxhighlight>
Once you know the base address, you can add the peripheral's address to it to get its IO registers. For example, [[BCM System Timer]] can be accessed at base address + 0x3000, and the [[Raspberry_Pi_Bare_Bones|UART0]] is at base address + 0x201000.

=See also=
* [[ARM Overview]]
* [[Raspberry Pi]]
* [[Raspberry Pi Bare Bones]]
* [https://developer.arm.com/docs/ddi0595/latest/aarch64-system-registers/midr_el1 ARM Developer, Main ID Register]

[[Category:ARM]]
[[Category:Raspberry Pi]]
[[Category:Hardware Detection]]
````

<a id="w004417"></a>

## W004417 — User:Mercury0x000d

Bron-ID: W004417
Namespace: User (2)
Revisie: 23935 | Datum: 2019-09-16T18:24:48Z
Bron: https://wiki.osdev.org/User:Mercury0x000d
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMercury0x000d&oldid=23935

````text
Hey there! I'm Mercury, longtime computer enthusiast and creator/primary coder of the Night Kernel. This is my sandbox to flesh out ideas on which I'm working.



{{Filesystems}}
{{In Progress}}

=== MFAT ===
That's right, it's Mercury's FAT file system! It's similar to exFAT, but slimmed down and simpler. In a way, FAT-based file systems are a great choice for solid state drives since fragmentation doesn't add to the overall load time of the file.


==== Features ====
<ul>
<li>Very low overhead</li>
<li>Minimal wasted drive space</li>
<li>Long filename support</li>
</ul>


==== Differences from exFAT ====
<ul>
<li>exFAT stores the case given to a file but internally ignores case when matching filenames, but MFAT honors case exactly as presented, eliminating the need for an upcase table.</li>
<li>MFAT uses the traditional method of determining free sectors by walking the FAT as opposed to using a volume allocation bitmap.</li>
<li>No 0x55AA signature at the end of the boot partition. Since regular FAT drivers <i>should</i> verify this signature prior to attempting to parse the file system structures, this is done to signify that they should not attempt to process an MFAT file system.</li>
<ul>



== Partition Layout ==

{| {{wikitable}}
|+ Overview of the order of structures in an exFAT partition
! Section !! Contents !! Offset (sector number) !! Size (in sectors)
|-
| rowspan = 3 | Main Boot Region || [[#Boot_Sector_Structure|Boot Sector]] || 0 || 1
|-
| [[#Extended_Boot_Sectors_Structure|Extended Boot Sectors]] || 1 || 8
|-
| Reserved sectors || 10 || 1
|-
| rowspan = 2 | FAT Region || [[#FAT_Region_Structure|First FAT]] || FATOffset || FATLength
|-
| Second FAT || FATOffset + FATLength || FATLength * (NumberOfFATs - 1)
|-
| rowspan = 2 | Data Region || Cluster Heap || ClusterheapOffset || ClusterCount * 2<sup>SectorsPerClusterShift</sup>
|-
| Excess Space (to end of partition or disk) || ClusterheapOffset + ClusterCount * 2<sup>SectorsPerClusterShift</sup> || VolumeLength - (ClusterHeapOffset + ClusterCount * 2<sup>SectorsPerClusterShift</sup>)
|-
|}




== Structures ==

=== Boot Sector Structure ===


{| {{wikitable}}
|-
! Offset !! Size !! Name !! Description
|-
| 0x000 || 3 || JumpBoot || Jump instruction to start of boot code. For x86 CPUs, the byte sequence is usually EB 76 90.
|-
| 0x003 || 8 || FileSystemName || The "MFAT&nbsp;&nbsp;&nbsp;&nbsp;" signature.
|-
| 0x00B || 53 || MustBeZero || This range of bytes is set null to help existing FAT drivers from mistakenly parsing an MFAT partition under the assumption that it contains valid FAT16 or FAT32 data.
|-
| 0x048 || 8 || VolumeSize || The total number of sectors in this MFAT partition.
|-
| 0x050 || 4 || FATOffset || Offset in sectors from the start of the partition to the first FAT
|-
| 0x054 || 4 || FATSize || Number of sectors in each FAT.
|-
| 0x058 || 4 || ClusterHeapOffset || Offset in sectors from the start of the partition to the start of the Cluster Heap, where files and folders live.
|-
| 0x05C || 4 || ClusterHeapSize || The total number of clusters in the Cluster Heap.
|-
| 0x060 || 4 || RootDirectoryChain || The starting cluster of the root directory's cluster chain. Usually, but not necessarily, zero.
|-
| 0x064 || 4 || VolumeSerialNumber || A serial number for this partition.
|-
| 0x068 || 1 || Revision || Specifies the version of MFAT specification to which the structures in this partition adhere.
|-
| 0x06A || 4 || VolumeFlags || 
{| {{wikitable}}
|-
! Bit !! Name !! Description
|-
| 0 || VolumeIntegrity || 0 - Partition is not in a good state<br>
1 - Partition is in a good state (the most recent file operation completed and all structures were written back to disk)
|-
| 1 - 31 || n/a || Reserved
|-
|}
|-
| 0x06C || 1 || BytesPerSectorShift || The number of places one would need to shift to translate bytes to sectors or sectors to bytes. Serves as an assist to allow shifting instead of multiplication or division.
|-
| 0x06D || 1 || SectorsPerClusterShift || Similar to the above, except used to calculate sectors per cluster.
|-
| 0x06F || 1 || DriveSelect || Similar to the Drive Number field found at offset 0x24 in previous FAT versions. Usually 0x80 for hard disks.
|-
| 0x070 || 1 || PercentInUse || Percent of clusters in the Cluster Heap which are allocated, rounded down to the nearest whole number.
|-
| 0x071 || 7 || Reserved || Do not use
|-
| 0x078 || 390 || BootCode || Boot code pointed to by the initial jump instruction at the beginning of the structure
|-
|}



=== Extended Boot Sectors Structure ===

{| {{wikitable}}
|-
! Offset !! Size !! Name !! Description
|-
| 0x00 || 2<sup>BytesPerSectorShift</sup> - 4 || ExtendedBootCode || Additional space for boot code.
|-
| 2<sup>BytesPerSectorShift</sup> - 4 || 4 || ExtendedBootSignature || A series of bytes (0x00, 0x00, 0x55, 0xAA) which indicates this is a proper boot sector, similar to previous FAT versions, but now a 32-bit value instead of 16-bit. Should be verified prior to depending on any other fields in the Extended Boot Sector. Note that this signature will appear on <i>each</i> of the Extended Boot Sectors.
|-
|}



=== FAT Region Structure ===
{| {{wikitable}}
|-
! Offset !! Size !! Name !! Description
|-
| 0x00 || 4 || FatEntry[0] || Comprised of a Media Descriptor byte (always 0xF8) followed by three bytes of 0xFF each 
|-
| 0x04 || 4 || FatEntry[1] || Always 0xFFFFFFFF. Do not use, interpret, or change.
|-
| 0x08 || 4 || FatEntry[2] || One of the following:

{| {{wikitable}}
|-
! Value !! Description
|-
| 0x00000000 || Denotes a free cluster available for allocation
|-
| 0x00000002 through ClusterCount - 1 || A pointer to the next cluster in the chain
|-
| 0xFFFFFFF7 || Bad cluster
|-
| 0xFFFFFFFF || End-of-chain markeer
|-
|}
These are identical to the values used in previous FAT versions, with the difference that 0x00000001, 0xFFFFFFF0 - 0xFFFFFFF6, and 0xFFFFFFF8 - 0xFFFFFFFE are no longer ascribed any significance.

|-
| ... || ... || ... || ...
|-
|}

The Second FAT is structured identically to the first, and is only present in transaction-safe exFAT implementations.



== Directory Hexdump Example ==

    00200000  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
    00200010  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
    00200020  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
    00200030  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
    00200040  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
    00200050  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
    00200060  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
    00200070  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
    00200080  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
    00200090  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
    002000A0  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
    002000B0  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
    002000C0  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
    002000D0  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
    002000E0  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
    002000F0  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|

&nbsp;



[[Category:Filesystems]]
````

<a id="w004419"></a>

## W004419 — WebAssembly

Bron-ID: W004419
Namespace: Main (0)
Revisie: 28777 | Datum: 2024-04-30T10:23:13Z
Bron: https://wiki.osdev.org/WebAssembly
Revisie-URL: https://wiki.osdev.org/index.php?title=WebAssembly&oldid=28777

````text
'''WebAssembly''' is a relatively new standard to support architecture independent executables, primarily on Web pages. But it is a much more than that, and the way it has been specified makes it uniquely qualified for a language, OS and machine independent executable format.

== Rationale ==

What's any Web has to do with OS kernels, you may ask. The truth is, the name WebAssembly (or '''WASM''' in short) is quite misleading, as it's a very well specified bytecode format with a minimal interpreting environment, which '''does not require any Web-related technologies at all'''.

A kernel already should have an executable format loader and parser (like an [[ELF]] parser), but that must use native CPU instructions. Adding a small bytecode interpreter to that would allow any hobby OS to execute machine and OS independent binaries, which is a great benefit. It's similar how different Android devices use Java for executing the proprietary APK system.

Unlike Java and Dalvik, interpreting WebAssembly binary is much easier, has no legal concerns, and there are many already written libraries to do so in many languages (most notably [[C]] and [[Rust]]).

== Comparison to Other Bytecode Formats ==

=== Lua ===

[[Lua]] is a very popular scripting language. However it's compiler, interpreter and execution environment is mixed into a single library. That's good for integrating with user space applications, but makes it difficult to embed into kernels (which would only need the bytecode interpreter part). The bytecode format is not really standardized, although seems quite stable.

=== Python ===

As strange as it seems, there's no such thing as Python bytecode standard. Therefore it is pretty impossible to embed a Python interpreter into a kernel. There are interpreter libraries which you could link in, but just as the format, they are constantly changing, updated, and limited to a specific version of Python.

=== Java ===

[[Java_For_Starters|Java]] Has a very well defined bytecode format, but unfortunately there's no simple bytecode interpreter library. Both the official JVM and it's OpenSource counterparts are huge bloated software, not really designed for embedding into kernels. Even if there were a simple interpreter library, Java is licensed by Oracle and use is subject to certain legal terms. To clearify, Android does not use the Java bytecode at all, it's interpreting Dalvik bytecode which is compiled from Java source.

=== WebAssembly ===

Unlike the rest, WebAssembly is not tied to any programming language, instead it's an open specification made by W3C. Therefore it is possible to compile C, C++, Rust, Pascal or even Basic into WASM bytecode. Also the execution environment is separated, and very well defined, therefore there are many implementations you can choose from.

One of the most known C/C++ compiler that produces WASM bytecode is [https://emscripten.org/ EmScripten], which is built on top of the LLVM compiler architecture.
There's a [https://github.com/WebAssembly/spec/tree/master/interpreter reference bytecode interpreter] provided by W3C, which is in OCAML and aimed at simplicity (and not speed).

The [https://www.w3.org/TR/wasm-core-1/ WebAssembly documentation an specification] allows unlimited languages and interpreters, and there are no licensing concerns involved. You are free to use any WASM compiler for any language and implement your own interpreter in your kernel.

== WebAssembly Interpreter Parlance ==

* '''WASM''': a file which contains [https://www.w3.org/TR/wasm-core-1/#binary binary WebAssembly bytecode], also the name of the reference interpreter.
* '''WAT''': WebAssembly Text source format, is a [https://www.w3.org/TR/wasm-core-1/#text plain text representation] of the bytecode (see [https://github.com/WebAssembly/binaryen Binaryen] wasm-at tool).
* '''WABT''': WebAssembly Binary Toolkit, equivalent of binutils. Includes assembler, objdump and other tools (see [https://github.com/WebAssembly/wabt WABT]).
* '''MVP''': Minimal Viable Product, which means the smallest WASM interpreter (nothing Web related required or included). Defined in [https://www.w3.org/TR/wasm-core/ WebAssembly Core Specification]
* '''WASI''': WebAssembly System Interface, this defines an ABI and a set of functions how the bytecode interacts with the OS and non-WASM libraries.
* '''EmScripten''': a WASI module, functions like memory allocation, everything that the EmScripten compiler may generate code for. Very simply speaking it's WebAssembly's current libc.
* '''Web API''': a standardized way to [https://www.w3.org/TR/wasm-web-api/ integrate WebAssembly into webpages], not our concern right now. Note that Web API is ''optional'' for WebAssembly.

== Linking ==

WebAssembly has a very clean module linking interface, the [https://github.com/WebAssembly/WASI WASI]. Also defined for OS-independent, easy [https://hacks.mozilla.org/2019/03/standardizing-wasi-a-webassembly-system-interface/ non-Web integration]. There're official POSIX and SDL modules (more like library wrappers) for WebAssembly, but the number of unofficial, third-party libraries are growing day by day.

Linkage for JavaScript, C/C++, Rust, Python and many other languages are now available and well tested. For kernels, you'll be interested in the [https://github.com/WebAssembly/wasm-c-api C] or Rust linkage mostly. You can find and incomplete and growing [https://github.com/appcypher/awesome-wasm-langs list here].

Right now EmScripten module is used for an OS interface, but this is going to change pretty soon. Big effort is put into WASI to provide a fully featured libc interface for WASM bytecode using musl as reference. Once WASI specs got frozen, all you'll need to implement/include is a WASI interpreter.

== Binary Format ==

A WASM file starts with a four byte magic, "asm\001", where the last character denotes format version. As such, it's pretty easy to identify.

After the header come several sections, one of which contains the bytecode. The binutils equivalent [https://github.com/WebAssembly/wabt WABT] includes a tool called [https://webassembly.github.io/wabt/doc/wasm-objdump.1.html wasm-objdump] to dump these and disassemble bytecode. The details and encodings are specified by the WebAssembly Core Specification, section [https://www.w3.org/TR/wasm-core-1/#binary binary format].

== See Also ==

==== Specifications ====
* [https://www.w3.org/TR/wasm-core-1 WebAssembly Core Specification]
* [https://github.com/WebAssembly/WASI WASI], ABI specification to run WASM outside of Web browsers
* [https://github.com/WebAssembly/WASI/blob/main/docs/WASI-overview.md WASI overview]
* [https://github.com/CommonWA/cwa-spec CommonWA] minimal specification of the standard API for non-Web usermode environments
* [https://en.wikipedia.org/wiki/WebAssembly WebAssembly] on Wikipedia
* [https://developer.mozilla.org/en-US/docs/WebAssembly WebAssembly] on Mozilla Development Network
* [https://www.rust-lang.org/what/wasm Rust and WebAssembly]

==== Compilers ====
* [https://emscripten.org/ EmScripten], a reference C/C++ compiler that produces WASM bytecode
* [https://surma.dev/things/c-to-webassembly/ LLVM] good description on how to use LLVM's default wasm target '''without''' the EmScripten bloat
* [https://github.com/tweag/asterius Asterius], a [[Haskell]] compiler that produces WASM bytecode
* [https://rustwasm.github.io/ wasm-pack] a Rust compiler that produces WASM bytecode
* [https://github.com/i-net-software/JWebAssembly JWebAssembly], a Java bytecode to WebAssembly bytecode converter
* [https://github.com/WebAssembly/binaryen Binaryen], low level tools for WebAssembly, provided by W3C
* [https://github.com/bytecodealliance/wasmtime/tree/master/cranelift Cranelift], a WASM to native code compiler in Rust

==== Interpeters ====
* [https://github.com/WebAssembly/spec/tree/master/interpreter Reference Interpreter] in '''OCaml''', provided by W3C
* [https://github.com/WebAssembly/wasmint libwasmint] Interpreter in '''C++''', a library designed for embedding, provided by W3C
* [https://github.com/WebAssembly/wabt/tree/master/src/interp wasm-interp] Interpreter in '''C++''', part of WABT, more complete than libwasmint
* [https://github.com/bytecodealliance/wasmtime/ wasmtime] yet another wasm interpreter, this time in '''Rust''' with WASI support (this is the reference implementation for WASI)
* [https://github.com/kanaka/wac wac, wax, wace] Interpreter in '''C''', this provides MVP, WASI, EmScripten-compatibility; probably the easiest to embed in a kernel
* [https://github.com/wasm3/wasm3 wasm3] Interpreter in '''C''', aims for high performance and WASI compatibility
* [https://github.com/paritytech/wasmi wasmi], Interpreter in '''Rust'''
* [https://github.com/mohanson/pywasm pywasm] Interpreter in '''Python'''
* [https://github.com/perlin-network/life life] Interpreter in '''Go'''
* [https://github.com/kenny-ngo/wasmjit wasmjit] a Linux kernel module in '''C''' to run wasm bytecode
* [https://github.com/cervus-v/cervus cervus] WebAssembly subsystem for Linux in '''C'''
* [https://github.com/pablotron/pwasm PWASM] easy to integrate WASM execution library (uses DynASM JIT) written in '''C'''

==== Examples ====
* [https://github.com/kwast-os/kwast kwast] a kernel written in Rust which runs wasm bytecode in userspace (under heavy development as of early 2020).
* [https://github.com/ColinEberhardt/wasm-mandelbrot wasm-mandelbrot] example on how to use EmScripten, wasm-toolchain, clang + BinaryEm, etc.
* [https://github.com/mdn/webassembly-examples/ WebAssembly examples] by Mozilla
* [https://developer.mozilla.org/en-US/docs/WebAssembly/Rust_to_wasm Tutorial on Rust to WASM] by Mozilla

[[Category:Instruction Set Architecture]]
[[Category:Standards]]
````

<a id="w004420"></a>

## W004420 — User talk:Ajxs/Ada Runtime Library

Bron-ID: W004420
Namespace: User talk (3)
Revisie: 24040 | Datum: 2019-10-06T03:38:30Z
Bron: https://wiki.osdev.org/User_talk:Ajxs/Ada_Runtime_Library
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AAjxs%2FAda_Runtime_Library&oldid=24040

````text
The GPRBuild package is not available through the FSF, it is specific to GNAT Pro or GNAT GPL via AdaCore's website.
: It looks like GPRBuild is distributed by the FSF. I could be wrong about this since I don't have it infront of me, but I came to that conclusion from the fact that the version available from the Ubuntu repos lists the FSF in the '--version' text. It also looks like it's available from the FSF here: https://directory.fsf.org/wiki/Gprbuild --[[User:Ajxs|Ajxs]] 03:22, 24 September 2019 (CDT)

That page shows 2 links where to get them, 1 to AdaCore and 1 to debian, who get it from AdaCore.--[[User:Lucretia|Lucretia]] 03:31, 24 September 2019 (CDT)
: You're right about it being an AdaCore product. I'm not sure of what difference where you get it has on licensing, if any. I assumed that the FSF had built it from source and relicensed it. I'll also pay more attention to my labeling of AdaCore GNAT PRO vs GNAT GPL too. --[[User:Ajxs|Ajxs]] 03:56, 24 September 2019 (CDT)
:: GPRBuild is only available from AdaCore and it is GPLv3 like all their stuff.--[[User:Lucretia|Lucretia]] 06:14, 24 September 2019 (CDT)

I would link to [https://github.com/AdaCore/gprbuild] as well as to the libre site, it is currently difficult to know which commits work from GH for specific versions of GCC.--[[User:Lucretia|Lucretia]] 03:35, 24 September 2019 (CDT)

I would change the following:

The GNAT compiler offers a facility for configuring the runtime-library used during the compilation process. This is done by either using the --RTS=example command-line switch, or by using the for Runtime ("Ada") use "example"; directive in the project’s configuration file. 

to:

The GNAT compiler offers a facility for specifying a different runtime-library used during the compilation process. This is done by either using the --RTS=example command-line switch, or by using the <pre>for Runtime ("Ada") use "example";</pre> directive in the project’s configuration file. --[[User:Lucretia|Lucretia]] 03:40, 24 September 2019 (CDT)


----

I think "the approach recommended by AdaCore is to create and link against an empty run-time library." needs more elaboratio; 1) where do they say this? 2) What constitutes an empty library?

: Forgive me, I was misquoting AdaCore from [https://docs.adacore.com/gnat_rm-docs/html/gnat_rm/gnat_rm/obsolescent_features.html#pragma-no-run-time this page]. They did not use the term "empty run-time", that was my mistake. I'll amend the article to correct this. --[[User:Ajxs|Ajxs]] 22:38, 5 October 2019 (CDT)

This doesn't read well and is confusing:

"As noted in the source code comments above, the run-time sources are used as a dummy run-time to build the run-time library itself. One method to implement this is to include a pre-requisite step in your makefile to copy the sources to the final build destination adainclude directory, then to specify this build directory as the run-time for the project."

1) It's not really a dummy runtime.
2) The Copying of the runtime sources is really annoying and confusing, you can make symbolic links to them if you must have them in a separate source dir, GNAT usually just dumps specs and bodies in adainclude. But why not just set the Source_Dirs directive in the GPR to runtime/sources?

--[[User:Lucretia|Lucretia]] 03:43, 5 October 2019 (CDT)

: My reason for not taking the same approach that you did in regards to creating the RTS adainclude directory is that I like to conceptualise the resulting 'build' directory as being a portable, final built artifact of the build process. This is just a matter of opinion I guess, one that I have ported over to Ada from my approach in C. I agree that copying things over is a bit clumsy, I just went with the approach that requires the least effort to understand at a glance.

: I'll amend the wording of "dummy run-time" as well. --[[User:Ajxs|Ajxs]] 22:38, 5 October 2019 (CDT)

----

I think to make this a full discussion/explanation of the RTS, I think this should be expanded further. I implemented a small secondary stack in bare bones on github - [https://github.com/Lucretia/bare_bones/blob/master/build/gnat/rts/common/adainclude/s-secsta.ads], this should be added to this discussion as an extension to ZFP but allows the developer to be more flexible with the kind of Ada they write, being able to return indefinite types is really handy. You'l also see from the bare bones source that the secondary stack enables hte use of 'Image attributes as well.

Also, I have memcpy [https://github.com/Lucretia/bare_bones/blob/master/build/gnat/rts/common/adainclude/memory.ads] implemented so you can assign objects to each other, this is also something which should go into the RTS and should be written about here, I don't have it mentioned in the main article, because I implemented that later.

: You're absolutely right. A secondary stack is a very, very valuable addition. A memcpy implementation is a great point too. --[[User:Ajxs|Ajxs]] 22:38, 5 October 2019 (CDT)

: The topic is extremely broad, so you'll have to forgive me falling behind in a few areas. I've tried to tackle the simpler aspects first. To avoid getting myself stuck into going overboard and going into too much detail I'd tried to keep the article a good balance of being practical to the topic of osdev, as well as being theoretically correct and providing a good introduction to the topic in an academically correct way. There's so much more that can be written on the runtime as a whole. None of this is to say that the article is anywhere near fully complete though. There's lots more to add with time, and I really appreciate your help! :) If you can think of other good topics to add feel free to list them and I'll be happy to do some more research. As I think I mention somewhere else, if you'd like me to publish it so you can edit and get the full credit I'm happy to go that route. --[[User:Ajxs|Ajxs]] 22:38, 5 October 2019 (CDT)

I think that this should also go into more detail on the gnat.adc file, like I do in the main bare bones article here. 

: Good idea. Would you like me to do this? If I publish the page in the mainstream you would be able to do the editing yourself and receive the proper credit if you wanted. Otherwise I'm happy to write it up myself. --[[User:Ajxs|Ajxs]] 22:38, 5 October 2019 (CDT)

I would like to clean up my bare bones article and source code, so that:

1) The source is Ada 2012.
2) The main article references this one, so this needs to really implement anything I have in the main article relating to the RTS so I can strip it out.
3) Write a further article covering the PC's board support package, which I've already started in bare bones, and you've already started with the port i/o package.

As part of the BSP article, I would like help to get multiboot.ad[sb] updated, I never did work out how to get the parameters from the command line, which I think is the new way of getting the boot parameters from grub, having this would make that package about as perfect as you can get. Multiboot should stand on it's own and not be under a PC root package, remember the multiboot standard is portable to other platforms.

: I've made a very simple initial implementation of this in my 'cxos' kernel example. You can see how I initialise the runtime and propagate the GRUB magic number and info structure to my multiboot init function here in this file [https://github.com/ajxs/cxos/blob/master/src/runtime/src/x86-multiboot-start.S x86-multiboot-start.S]. This is part of the base system initialisation process, which takes place within the RTS, as I talk about in the article here. You can see the structures used and associated functionality in the [https://github.com/ajxs/cxos/blob/master/src/runtime/src/x86-multiboot.ads x86-multiboot.ads] file and associated package body.<br/>
: Careful with these links, they might change very soon. But they should be good for a few days.<br/>
: If you want me to do any of the editing here I'm happy to. 

: What you said about Multiboot not being PC specific is very valid. It's a great suggestion that I'll take that on board in my own work! --[[User:Ajxs|Ajxs]] 22:38, 5 October 2019 (CDT)

But the Console package can be a child of the PC package.

I'm happy to start cleaning up the bare bones source into separate repos to enable this split.

: Awesome. Feel free to reach out if you need any help from me. --[[User:Ajxs|Ajxs]] 22:38, 5 October 2019 (CDT)

--[[User:Lucretia|Lucretia]] 05:00, 5 October 2019 (CDT)
````

<a id="w004421"></a>

## W004421 — Ada Runtime Library

Bron-ID: W004421
Namespace: Main (0)
Revisie: 24723 | Datum: 2020-05-25T14:11:27Z
Bron: https://wiki.osdev.org/Ada_Runtime_Library
Revisie-URL: https://wiki.osdev.org/index.php?title=Ada_Runtime_Library&oldid=24723

````text
This guide intends to serve as an introduction to the Ada run-time library, and as a guide to creating a basic Ada run-time library suitable for bare-metal development targeting the x86 platform.<br/>
For other architectures, AdaCore provides pre-made run-time libraries and board support packages targeting a variety of ARM/RISC-V boards. These are freely available on [https://github.com/AdaCore/bb-runtimes Github], as well as being packaged with their 'arm-elf' and 'riscv32-elf' compilers.

''Note: This guide is specific to the GNAT Ada compiler, available from either the FSF as part of [[GCC]], or as a stand-alone product from [https://www.adacore.com/download AdaCore]. Additionally it assumes the use of the GPRBuild suite of utilities, which are freely available as part of AdaCore’s GNAT compiler or can be built [https://github.com/AdaCore/gprbuild from source].''

== Background ==

The run-time library for a language is typically the standard library of functions responsible for implementing the interface with the underlying functionality exposed by the system. These are normally statically linked against the executable at compile-time.

The Ada run-time library is responsible for the implementation of the standard library defined in annexes A-H of the [http://ada-auth.org/standards/12rm/html/RM-TOC.html Ada Language Reference Manual]. Not all annexes defined in the standard are required to be implemented for a specific platform.<br/>
The Ada run-time is the equivalent of C’s [[wikipedia:C_standard_library|libc]]. However unlike the C standard the Ada standard does not explicitly define a differentiation between hosted and freestanding environments.

As with other systems programming languages the program being compiled is statically linked against the run-time library to produce the final executable. The compiler will attempt to ensure that only the required sections of the run-time are linked, so as to minimize the final binary size. In the case that only minimal or no standard library features are required, the approach recommended by AdaCore [https://docs.adacore.com/gnat_rm-docs/html/gnat_rm/gnat_rm/obsolescent_features.html#pragma-no-run-time here] is to use a run-time library implementing only the features required.

The GNAT compiler offers a facility for configuring the runtime-library used during the compilation process. This is done by either using the <tt>--RTS=example</tt> command-line switch, or by using the 
<pre>
for Runtime ("Ada") use "example";
</pre>
directive in the project’s configuration file.<br/>
This can be either the name of a run-time library that is visible to the compiler due to being in one of its search directories, or a relative path specifying the location of a run-time library directory.

As mentioned above, the Ada standard does not explicitly define a differentiation between hosted and freestanding environments. So unlike using GCC’s <tt>-ffreestanding</tt> switch to compile C for a bare-metal target, GNAT cannot automatically provide users with a valid run-time library suitable for targeting non-hosted environments. Bare-metal targets will often require fine-grained control to satisfy the limitations of the platform, so the developer will have to provide a run-time purpose built for the freestanding environment. For many targets there are Zero Foot Print' (zfp) run-time libraries available which are suitable for the purpose of operating-system development, but for x86 there are no zfp run-times freely available from the major vendors so the best option for a developer is to implement one themselves.

== Anatomy of a run-time library ==

An Ada run-time library is fundamentally a directory tree containing the run-time sources and compiled library files. The <tt>adainclude</tt> subdirectory contains the run-time sources, which will provide the Ada equivalent of C header files for our run-time. The <tt>adalib</tt> subdirectory contains the compiled library artifacts that programs will be statically linked against. The compiler will require the presence of these two directories within the run-time directory to detect it as valid. The location of these directories can be customised by creating two files named <tt>'ada_source_path'</tt> and <tt>'ada_object_path'</tt> which allow the specification of alternate locations for the source and object directories. For more information on this feature, refer to the [https://gcc.gnu.org/onlinedocs/gcc-8.2.0/gnat_ugn/Search-Paths-and-the-Run-Time-Library-RTL.html GNAT documentation].<br/>
A run-time library may be placed within the GNAT compiler's default search directories which will make it globally visible for the compiler, or can be placed in another location and specified as a relative path within the project's configuration file.<br/>
''Note: GPRBuild requires the presence of a run-time within its search path before it will fully acknowledge the presence of GNAT for a particular target.''

== Creating a run-time library suitable for x86 operating-system development ==

''Note: This section assumes that you are using an x86/x86-64 host compiler and are targeting an x86 system. Creating a run-time for other targets will be different. This section also assumes that you have a suitable [[GNAT_Cross-Compiler|cross-compiler]]. While you may be able to copy run-time source files from your system GNAT installation, a compiler specific to your target system will still be required. A detailed guide to building a GNAT cross-compiler can be found [[GNAT_Cross-Compiler|here]].''

In order to compile Ada code for bare-metal targets using the GNAT compiler, developers will need to configure an appropriate Ada run-time library for their target platform. The Ada standard recommends that all of the packages defined in ‘Annex A: Predefined Language Environment’ be defined for a platform, but for the purposes of bare-metal development this is impractical.<br/>
In practical terms, creating a new run-time for a particular target is accomplished through the creation and compilation of a static library. Control over which library units are included in the final run-time library is as simple as including and omitting the appropriate files from the library’s source tree.<br/>
For the purposes of operating-system development, at a bare-minimum the run-time library will need to implement parts of Annex C: ‘Systems Programming’. The terminology used within the Ada community for such a run-time library suitable for bare-metal is a 'zfp' run-time, short for 'Zero Foot Print'.

The best approach to creating a run-time library for a new platform is to start with components from the standard run-time library of your system's Ada compiler. Many of these components are platform independent and can be safely copied to your new run-time without further refinement.

Shown below is an example directory structure for a run-time library GNAT project:
<pre>
- runtime
    - build             ( The build directory. Contains the final built run-time library. )
        - adainclude    ( Contains the run-time sources to be included in applications. )
        - adalib        ( Contains the compiled run-time static library. )
    - obj               ( The intermediate build directory. Contains build artifacts. )
    - source            ( Contains the run-time library source files. )
</pre>


An example GNAT project configuration file for creating a run-time library can be seen below. The comment annotations will explain the relevant options.<br/>
For more information regarding GNAT project configuration refer to the AdaCore documentation: [https://docs.adacore.com/gprbuild-docs/html/gprbuild_ug/gnat_project_manager.html]

<source lang="ada">
library project Runtime is
   --  Tells Gprbuild to create any missing directories in the build process.
   for Create_Missing_Dirs use "True";
   --  The source files for the run-time library.
   --  The 'build' directory is used in this case so that GNAT can use our
   --  run-time as the selected run-time library during the build process. 
   --  To facilitate this, we copy our source files to the build directory.
   --  This serves two purposes, one is to build the run-time itself, the 
   --  second is to function as the run-time library's spec includes.
   for Source_Dirs use ("build/adainclude");
   --  The directory used for build artifacts.
   for Object_Dir use "obj";

   for Languages use (
     "Ada",
     "Asm_Cpp"
   );

   package Builder is
      for Global_Configuration_Pragmas use "runtime.adc";
      for Switches ("Ada") use (
        "-nostdlib",
        "-nostdinc"
      );
   end Builder;

   --  For a list of all compiler switches refer to: https://gcc.gnu.org/onlinedocs/gcc-9.2.0/gnat_ugn/Alphabetical-List-of-All-Switches.html#Alphabetical-List-of-All-Switches
   package Compiler is
      for Default_Switches ("Ada") use (
        "-O0",
        --  Using these directives instructs the Gnat GCC backend to place
        --  each function and data object in its own section in the resulting
        --  object file. GNAT requires this in order for the linker to 
        --  perform dead code elimination within the runtime library.
        --  For more information refer to:
        --    https://gcc.gnu.org/onlinedocs/gnat_ugn/Compilation-options.html
        "-ffunction-sections", 
        "-fdata-sections",
        "-gnat2012",
        --  This switch enables GNAT's Internal implementation mode. This 
        --  allows us to modify internal packages such as 'Ada' or 'System'
        --  which are needed to build a run-time library. This switch is only
        --  needed when building the run-time and should be disabled for all 
        --  other development.
        "-gnatg",
        --  Instructs GNAT to remove dead code from the runtime library.
        --  Requires the use of the '-ffunction-sections' and
        --  '-fdata-sections' directives as detailed above.
        "-Wl,--gc-sections"
      );
   end Compiler;

   --  The destination directory for the final library artifacts.
   --  We need to select a static library for this purpose to ensure that the 
   --  run-time is linked staticly againt our executable.
   --  For more information on library project options refer to: 
   --   http://docs.adacore.com/live/wave/gprbuild/html/gprbuild_ug/gprbuild_ug/gnat_project_manager.html#library-projects
   for Library_Dir use "build/adalib";
   for Library_Kind use "static";
   for Library_Name use "gnat";

   --  We require a valid run-time library to build our run-time. 
   --  We need to ensure that the run-time we use for this purpose is built 
   --  using the same compiler and targets the same platform. A reliable way
   --  to do this is to use our run-time's sources as a run-time to build
   --  itself.
   for Runtime ("Ada") use "build";

   for Target use "i686-elf";
end Runtime;
</source>

As noted in the source code comments above, in this setup the run-time sources are used as a run-time library during the build process. One method to implement this is to include a pre-requisite step in your makefile to copy the sources to the final build destination <tt>adainclude</tt> directory, then to specify this build directory as the run-time for the project. An example makefile for facilitating this can be seen below. 

''Please note that this makefile may not be fit for all projects and purposes.''

<source lang="make">
RUNTIME_BINARY := ${LIB_DIR}/libgnat.a
RUNTIME_PROJ   := runtime

SRC_DIR        := src
BUILD_DIR      := build
LIB_DIR        := ${BUILD_DIR}/adalib
INCLUDE_DIR    := ${BUILD_DIR}/adainclude

LINKER_SCRIPT  := x86-multiboot.ld
RUNTIME_CONFIG := runtime.xml

.PHONY: clean

all: ${RUNTIME_BINARY}

clean:
	gprclean -P${RUNTIME_PROJ}
	rm -rf ${BUILD_DIR}

# Build the run-time library.
${RUNTIME_BINARY}: ${INCLUDE_DIR}/*.ad[sb] ${INCLUDE_DIR}/*.S \
	${LIB_DIR} ${BUILD_DIR}/${LINKER_SCRIPT} ${BUILD_DIR}/${RUNTIME_CONFIG}
	gprbuild -P${RUNTIME_PROJ}

# Copy all Ada source files to the final include directory.
${INCLUDE_DIR}/*.ad[sb]: ${SRC_DIR}/*.ad[sb] ${COMMON_SRC_DIR}/*.ad[sb] ${INCLUDE_DIR}
	cp -a ${SRC_DIR}/*.ad[sb] ${INCLUDE_DIR}

# Copy any Assemblersource files to the final include directory.
${INCLUDE_DIR}/*.S: ${SRC_DIR}/*.S ${INCLUDE_DIR}
	cp -a ${SRC_DIR}/*.S ${INCLUDE_DIR}

# Copy the linker script to the final run-time directory.
${BUILD_DIR}/${LINKER_SCRIPT}:
	cp -a ${SRC_DIR}/${LINKER_SCRIPT} ${BUILD_DIR}/${LINKER_SCRIPT}

# Copy our run-time config to the final run-time directory.
${BUILD_DIR}/${RUNTIME_CONFIG}:
	cp -a ${SRC_DIR}/${RUNTIME_CONFIG} ${BUILD_DIR}/${RUNTIME_CONFIG}

# Create the necesary directory tree.
${INCLUDE_DIR}:
	mkdir -p ${INCLUDE_DIR}

${LIB_DIR}:
	mkdir -p ${LIB_DIR}
</source>


We can begin creating the source code for a minimal run-time by copying a list of files from our compiler's native run-time directory into the source directory of our run-time GNAT project. If you're using FSF GNAT this is most likely in a similar location to: <tt>/usr/lib/gcc/x86_64-linux-gnu/<version>/adainclude/</tt>. If you're using AdaCore GNAT Pro the directory structure for run-time sources is broken down into different sections, but you should still be able to find the same required files. ''Note: If you intend to use source files obtained from AdaCore's GNAT Pro or associated material in the construction of a run-time library, be sure to read their licensing to understand any restrictions imposed upon the use of their open-source material.'' <br/>
A list of files to copy from your compiler's default hosted run-time to your run-time library project's source directory are found below, together with an explanation of their purpose.

{| {{wikitable}}
|-
! Filename
! Description
|-
| a-unccon.ads
| Package specification for the <tt>Ada.Unchecked_Conversion </tt> package. This package provides a facility for unchecked casting from one type to another. This is generally discouraged in Ada, but can be necessary in bare-metal development.<br/>
You may notice the <tt>pragma Import (Intrinsic, Ada.Unchecked_Conversion);</tt> line here. This tells us that the body of this procedure is an *intrinsic subprogram* within the GNAT compiler and a definition is supplied by GNAT which we can import. For more information regarding intrinsic subprograms refer to AdaCore's documentation here: [https://docs.adacore.com/gnat_rm-docs/html/gnat_rm/gnat_rm/intrinsic_subprograms.html]
|-
| ada.ads
| The spec for the <tt>Ada</tt> package. This is the main parent package for most of the run-time library and is required to be present.
|-
| interfac.ads
| The spec for the <tt>Interfaces</tt> package. This package defines the necessary data types and functions for interfacing with other languages. This is of particular note to bare-metal platforms, since it provides definitions of the <tt>Integer_X</tt> and <tt>Unsigned_X</tt> types. It also provides definitions of the intrinsic shift and rotate functions to work on these types.
|-
| s-maccod.ads
| The spec for the <tt>System.Machine_code</tt> package. This package provides functionality for using inline assembly. This is relevant to operating-system development since it allows us more control over functionality like Port I/O. GNAT uses the same format for inline assembly as the rest of GCC. However it has additional features and syntax.
|-
| s-stoele.ads
| The spec for the <tt>System.Storage_Elements</tt> package. This package provides functionality for using and manipulating memory addresses, which we will require for memory-mapped I/O.
|-
| s-stoele.adb
| The body for the <tt>System.Storage_elements</tt> package. See above. Contains the definitions for the functions contained in the spec.
|-
| s-unstyp.ads
| The spec for the <tt>System.Unsigned_Types</tt> package. Contains definitions for unsigned integer types.
|-
| system.ads
| This is the spec file for the <tt>System</tt> package, which is the parent package for many of the packages in our run-time library. Every Ada program requires the <tt>System</tt> package to be present. This package contains important information about the target system, which is referenced by the compiler during code generation.<br/>
Refer to the 'System Implementation Parameters' section within the file for a list of parameters read by the compiler. For additional information regarding how these parameters are interpreted, refer to the following file within the compiler directory: <tt>gcc-<version>/gcc/ada/targparm.ads</tt>.<br/>
Additional changes are needed to be made to this file to make it suitable for a bare-metal target. These changes are detailed in the section below.
|}

=== System.ads === 

Every Ada program is required to have access to the <tt>System</tt> package. This package provides a specification for the target system hardware to the compiler. Since a run-time is specific to a single target hardware configuration, there will only be one system specification per run-time.<br/>
Detailed below are the necessary changes required to convert the System package specification from a hosted run-time to a freestanding one suitable for kernel development. For more information on individual settings refer to this file within your GNAT installation: <tt>gcc-<version>/gcc/ada/targparm.ads</tt>. A copy of this file can be found online at: [https://www2.adacore.com/gap-static/GNAT_Book/html/frontend/targparm__ads.htm]

{| {{wikitable}}
|-
! Change
! Description
|-
| <tt>Command_Line_Args := False</tt>
| Since our platform is bare-metal, there is no access to command line parameters.
|-
| <tt>Configurable_Run_Time := True</tt>
| Instructs GNAT that this is a user-configured run-time library.
|-
| <tt>Exit_Status_Supported := False</tt>
| As above, our platform does not support exit statuses.
|-
| <tt>Stack_Check_Probes := False</tt>
| Indicates that our run-time does not use GCC's stack checking mechanism.
|-
| <tt>Suppress_Standard_Library := True</tt>
| According to the GNAT docs, If this flag is True, then the standard library is not included by default in the executable.
|-
| <tt>ZCX_By_Default := False</tt>
| Indicates if zero cost exceptions are active by default. Refer to the [https://gcc.gnu.org/onlinedocs/gnat_ugn/Run-Time-Libraries.html documentation] for more information on Zero-cost-exceptions.
|-
| <tt>GCC_ZCX_Support := False</tt>
| Disables GCC's support for zero-cost exceptions.
|}

Also, add the following line in the private part of the package:

<source lang="ada">
private
   Run_Time_Name : constant String := "Bare Bones Run Time";
</source>

According to <tt>targparm.ads</tt>, this directive should be the first thing after the private keyword.

=== Platform-specific support code ===

For the purposes of bare-metal development, it may be advantageous to define platform-specific support packages within the run-time library to avoid cluttering the main kernel package with platform-specific code. This also supports a greater degree of abstraction, and better portability of the kernel itself.<br/>
Any packages contained within the runtime will be accessible to the linked program automatically. Be mindful not to declare any new packages as children of the top level <tt>Ada</tt> or <tt>System</tt> packages, as GNAT will consider any non-standard children to be ‘implementation-specific’: for internal use only and forbid their use outside of the run-time.

It might be advantageous in certain situations to declare platform-specific support packages as children of the <tt>System</tt> package to restrict the direct use of platform-specific code in the linked application.

One example of useful platform support code for x86 would be to implement functionality for [[Port_IO|Port IO]]. An example implementation can be seen below.

'''x86.ads'''
<source lang="ada">
-------------------------------------------------------------------------------
--  X86
--
--  Purpose:
--    This package contains support code for the x86 system.
-------------------------------------------------------------------------------
package x86 is
   pragma Preelaborate (x86);
end x86;
</source>

'''x86-port_io.ads'''
<source lang="ada">
with Interfaces;
with System;

-------------------------------------------------------------------------------
--  X86.PORT_IO
--
--  Purpose:
--    This package contains functionality for port-mapped I/O on the x86
--    platform.
--    Functions are included for inputting and outputting data to port-mapped
--    addresses, useful for interacting with system peripherals.
-------------------------------------------------------------------------------
package x86.Port_IO is
   pragma Preelaborate (x86.Port_IO);

   ----------------------------------------------------------------------------
   --  Inb
   --
   --  Purpose:
   --    This function reads a byte from a particular IO port.
   --  Exceptions:
   --    None.
   ----------------------------------------------------------------------------
   function Inb (
     Port : System.Address
   ) return Interfaces.Unsigned_8
   with Volatile_Function;

   ----------------------------------------------------------------------------
   --  Outb
   --
   --  Purpose:
   --    This function writes a byte to a particular IO port.
   --  Exceptions:
   --    None.
   ----------------------------------------------------------------------------
   procedure Outb (
     Port : System.Address;
     Data : Interfaces.Unsigned_8
   );
end x86.Port_IO;
</source>


'''x86-port_io.adb'''
<source lang="ada">
with System.Machine_Code;

package body x86.Port_IO is
   ----------------------------------------------------------------------------
   --  Inb
   ----------------------------------------------------------------------------
   function Inb (
     Port : System.Address
   ) return Interfaces.Unsigned_8 is
      Data : Interfaces.Unsigned_8;
   begin
      System.Machine_Code.Asm (
        Template => "inb %w1, %0",
        Inputs => (
          System.Address'Asm_Input ("Nd", Port)
        ),
        Outputs => (
          Interfaces.Unsigned_8'Asm_Output ("=a", Data)
        ),
        Volatile => True);

      return Data;
   end Inb;

   ----------------------------------------------------------------------------
   --  Outb
   ----------------------------------------------------------------------------
   procedure Outb (
     Port : System.Address;
     Data : Interfaces.Unsigned_8
   ) is
   begin
      System.Machine_Code.Asm (
        Template => "outb %0, %w1",
        Inputs => (
          Interfaces.Unsigned_8'Asm_Input ("a", Data),
          System.Address'Asm_Input ("Nd", Port)
        ),
        Volatile => True);
   end Outb;
end x86.Port_IO;
</source>

Fundamentally, the role of the runtime library is to implement the features defined in the Ada language standard for the particular target platform. This requires the creation of target-specific packages to interface with platform features such as internal clocks, [[UART|UARTs]] and [[PIT|timers]], among other things. These packages can then be used by language-defined packages in the runtime library to expose this functionality to the linked application.

=== System initialization code ===

It is possible to perform the initialization of the underlying hardware platform within the run-time library.<br/>
AdaCore GNAT’s ARM bareboard target run-times employ this method of system initialization, packaging the run-time library as a combined ‘board support package’.<br/>
These run-times typically include a linker script containing a memory map of the target as well as assembly code for initializing the target platform. These linker scripts are not used in the building of the run-time library itself, but are invoked when a program is linked against the run-time library. This is facilitated via the use of additional configuration files bundled with the run-time library.<br/>
GNAT/Gprbuild is capable of building Asm/C files bundled inside a GNAT project, which allows us to include platform initialization code. XML configuration files can be used to provide additional configuration options to Gprconfig when linking against the run-time library. These XML files are generally referred to in official documentation as gprbuild’s ‘knowledge base’. In addition to its own directory, Gprbuild will search the source directories for the program being compiled, as well as its run-time library directories. To see what directories are searched for source and configuration files, the ‘gprls’ utility can be used. Example usage is <tt>gprls -v -P project.gpr</tt>.

A practical usage of this would be to define a <tt>runtime.xml</tt> file, placed within the final <tt>adainclude</tt> directory of the run-time library. This is used to automatically use a linker script for the target platform when the kernel is linked against the library.

<source lang="xml">
<?xml version="1.0" ?>
<gprconfig>
<configuration>
    <config>
      <![CDATA[
        package Linker is
            for Required_Switches use Linker'Required_Switches & (
              "-T${RUNTIME_DIR(ada)}/x86-multiboot.ld"
            );
        end Linker;
      ]]>
   </config>
  </configuration>
</gprconfig>
</source>

Where <tt>${RUNTIME}</tt> is a predefined variable made available by Gprbuild.

Additional documentation for these features can be found at: [http://docs.adacore.com/live/wave/gprbuild/html/gprbuild_ug/gprbuild_ug/companion_tools.html#configuration]

=== Exceptions ===

Exception handling in Ada is implemented in the run-time library package <tt>Ada.Exceptions</tt>. This is defined in section 11 of the the Ada Language Reference Manual and can be referenced [http://www.adaic.com/resources/add_content/standards/12rm/html/RM-11-4-1.html here]. Comprehensive exception handling functionality is typically disabled for bare-metal targets due to the large amount of underlying functionality required for its implementation, such as stack unwinding, streams, secondary stack, etc. As such it is not normally included in the run-time library for such targets. Local handling exceptions within a subprogram is still possible without the <tt>Ada.Exceptions</tt> package being implemented for a target.

For a bare-bones run-time library used for operating-system development it is recommended to use the <tt>No_Exception_Propagation</tt> restriction pragma. This restriction ensures that an exception is never propagated over a subprogram boundary into an outer scope. The implementation of exception handling over subprogram boundaries typically requires the use of <tt>setjmp/longjmp</tt>, which are unsuitable for the purpose of bare-metal development. With this restriction applied the only case in which an exception may be raised is when the exception handler is contained within the same subprogram. According to [https://docs.adacore.com/gnat_rm-docs/html/gnat_rm/gnat_rm/standard_and_implementation_defined_restrictions.html#no-exception-propagation official documentation], the effect of this is that raising an exception is essentially a <tt>goto</tt> statement.

Any exception not handled explicitly within the same subprogram will be caught by the <tt>Last_Chance_Handler</tt> subprogram.

=== Elaboration ===

Ada refers to the expansion of definitions contained within package files as elaboration. The Ada standard defines the pragma directive <tt>Preelaborate</tt> to specify that a package requires no elaboration at run-time. This is ideal for our purposes since our run-time library will be instantiated and initialized at boot time, prior to having a fully functional system environment. At this point we have limited facilities for initializing run-time library components. Marking our run-time library components as pre-elaborable ensures that they do not require any additional execution at run-time, any violation of this pragma will generate a compile-time error. The legality rules for use of this directive can be found in the Reference Manual: [http://ada-auth.org/standards/12rm/html/RM-C-4.html]

The language standard also defines the ‘pure’ pragma. This pragma specifies that any call to any of the package’s units should be considered to have no side-effects. This is ideal in many circumstances, but a developer would be advised to be wary that this pragma advises the compiler that it may cache subsequent calls to the same package unit. As a result this pragma directive is not suitable for a package using any volatile memory access.

Additional documentation on this subject is available from AdaCore at: 
[https://docs.adacore.com/gnat_ugn-docs/html/gnat_ugn/gnat_ugn/elaboration_order_handling_in_gnat.html]

=== Image attributes ===
Scalar types in Ada support the <tt>Image</tt> attribute. This attribute returns a string representation of a variable. This is of particular significance for operating-system development due to the need to ouput debugging information in string format to the console. The <tt>Image</tt> attribute is defined in [https://www.adaic.org/resources/add_content/standards/05rm/html/RM-K.html Annex K] of the Ada Language Reference Manual. Support for the image attributes of a particular scalar type is implemented in the run-time library.<br/>
For operating-system development, arguably the most important types to be able to print are unsigned integer types. By default, the GNAT compiler expects functionality for imaging unsigned integer types to be defined in the package <tt>s-imguns.ads</tt> and its associated package body. Provided below is an example implementation:

'''s-imguns.ads'''
<source lang="ada">
with System.Unsigned_Types;

-------------------------------------------------------------------------------
--  SYSTEM.IMG_UNS
--
--  Purpose:
--    This package provides an implementation of the Image attribute for
--    unsigned integer types.
-------------------------------------------------------------------------------
package System.Img_Uns is
   pragma Pure;

   ----------------------------------------------------------------------------
   --  Image_Unsigned
   --
   --  purpose:
   --    Computes Unsigned'Image (V) and stores the result in S (1 .. P) \
   --    setting the resulting value of P. The caller guarantees that S is
   --    long enough to hold the result, and that S'First is 1.
   ----------------------------------------------------------------------------
   procedure Image_Unsigned (
     V :        System.Unsigned_Types.Unsigned;
     S : in out String;
     P : out    Natural
   )
   with Inline;

   ----------------------------------------------------------------------------
   --  Set_Image_Unsigned
   --
   --  Purpose:
   --    Stores the image of V in S starting at S (P + 1), P is updated to
   --    point to the last character stored. The value stored is identical
   --    to the value of Unsigned'Image (V) except that no leading space is
   --    stored. The caller guarantees that S is long enough to hold the
   --    result. S need not have a lower bound of 1.
   ----------------------------------------------------------------------------
   procedure Set_Image_Unsigned (
      V :        System.Unsigned_Types.Unsigned;
      S : in out String;
      P : in out Natural
   );

end System.Img_Uns;
</source>

'''s-imguns.adb'''
<source lang="ada">
with System.Unsigned_Types; use System.Unsigned_Types;

package body System.Img_Uns is
   ----------------------------------------------------------------------------
   --  Image_Unsigned
   ----------------------------------------------------------------------------
   procedure Image_Unsigned (
     V :        System.Unsigned_Types.Unsigned;
     S : in out String;
     P : out    Natural
   ) is
      pragma Assert (S'First = 1);
   begin
      S (1) := ' ';
      P := 1;
      Set_Image_Unsigned (V, S, P);
   exception
      when Constraint_Error =>
         return;
   end Image_Unsigned;

   ----------------------------------------------------------------------------
   --  Set_Image_Unsigned
   --
   --  Implementation Notes:
   --    - Refer to: http://www.nihamkin.com/2016/11/25/
   --      writing-linux-modules-in-ada-part-3/
   ----------------------------------------------------------------------------
   procedure Set_Image_Unsigned (
     V :        Unsigned;
     S : in out String;
     P : in out Natural
   ) is
      --  The number of digits in the resulting string.
      Digit_Count : Natural := 0;
   begin
      Get_Digit_Count :
         declare
            V2 : Unsigned := V;
         begin
            while V2 /= 0 loop
               Digit_Count := Digit_Count + 1;
               V2 := V2 / 10;
            end loop;
         exception
            when Constraint_Error =>
               Digit_Count := 0;
         end Get_Digit_Count;

      Write_To_String :
         begin
            if Digit_Count = 0 then
               P := P + 1;
               S (P) := '0';
            else
               for I in reverse 0 .. (Digit_Count - 1) loop
                  P := P + 1;
                  S (P) := Character'Val (48 + (V / 10 ** I) rem 10);
               end loop;
            end if;
         exception
            when Constraint_Error =>
               return;
         end Write_To_String;
   end Set_Image_Unsigned;

end System.Img_Uns;
</source>

User-defined images for a given type can be implemented by overriding the default implementation of the <tt>Put_Image</tt> attribute.

== Initialization ==

The Ada run-time library is initialized by calling the <tt>adainit</tt> procedure. This procedure is responsible for the elaboration of the individual packages within the run-time library.

It is the responsibility of the platform initialization code to call this procedure prior to using any of the subprograms in the run-time library.<br/>
As noted [https://gcc.gnu.org/onlinedocs/gnat_ugn/Binding-with-Non-Ada-Main-Programs.html here] ''"...it is assumed that the basic execution environment must be setup to be appropriate for Ada execution at the point where the first Ada subprogram is called. In particular, if the Ada code will do any floating-point operations, then the FPU must be setup in an appropriate manner. For the case of the x86, for example, full precision mode is required..."''<br/>

The Ada standard provides the following implementation advice: ''"If an implementation supports Export for a given language, then it should also allow the main subprogram to be written in that language. It should support some mechanism for invoking the elaboration of the Ada library units included in the system, and for invoking the finalization of the environment task. On typical systems, the recommended mechanism is to provide two subprograms whose link names are '<tt>adainit</tt>' and '<tt>adafinal</tt>'. Adainit should contain the elaboration code for library units. Adafinal should contain the finalization code. These subprograms should have no effect the second and subsequent time they are called."'' Additional documentation can be found [http://ada-auth.org/standards/12rm/html/RM-B-1.html here]

In the case of a pure Ada kernel, a good place to put this procedure call is within the boot code assembly to ensure that the run-time is properly initialized prior to the jump to Ada code.

== Additional Resources ==

Additional documentation on the subject of creating a new run-time library is available from AdaCore at: 
[https://docs.adacore.com/gnathie_ug-docs/html/gnathie_ug/gnathie_ug/the_gnat_configurable_run_time_facility.html]

Information regarding porting AdaCore's ARM runtimes to a new system can be found here:
[https://blog.adacore.com/porting-the-ada-runtime-to-a-new-arm-board]


[[Category:Ada]]
````

<a id="w004422"></a>

## W004422 — Devicetree

Bron-ID: W004422
Namespace: Main (0)
Revisie: 30825 | Datum: 2026-06-29T19:22:14Z
Bron: https://wiki.osdev.org/Devicetree
Revisie-URL: https://wiki.osdev.org/index.php?title=Devicetree&oldid=30825

````text
{{Stub}}

Device tree is a standard for describing the layout of devices attached to a computer. As opposed to discovering devices by e.g. enumerating [[PCI]] devices as is the case with [[x86]], kernels which use device tree are handed a static device tree blob by the [[bootloader]]. The device tree specification specifies two formats for device trees:

* DTS (Device Tree Source) which is a human-readable plain text specification describing the various devices attached to the system and their layout in memory.
* DTB (Device Tree Blob) which is a flattened device tree binary blob to be parsed by the kernel at boot.

The command line tool <code>dtc</code>, or [https://github.com/dgibson/dtc Device Tree Compiler], can be used to compile a DTS into a DTB.
Device trees are used by Linux and iOS/Darwin to boot on a variety of architectures including [[AArch64]].

== See Also ==
=== External Links ===
* [[Raspberry Pi]] documentation describing the format and usage of DTS: https://www.raspberrypi.com/documentation/computers/configuration.html#device-trees-overlays-and-parameters
* The Device Tree Specification: https://www.devicetree.org/specifications/
* The Device Tree Compiler: https://github.com/dgibson/dtc
* Linux Kernel Documentation about Device Tree: https://www.kernel.org/doc/Documentation/devicetree/

[[Category:Firmware]]
[[Category:ARM]]
[[Category:Raspberry Pi]]
[[Category:Hardware]]
````

<a id="w004423"></a>

## W004423 — Talk:Serial Port

Bron-ID: W004423
Namespace: Talk (1)
Revisie: 24078 | Datum: 2019-11-06T03:38:07Z
Bron: https://wiki.osdev.org/Talk:Serial_Port
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ASerial_Port&oldid=24078

````text
Very interesting article! What are your plans for integrating this with the other serial port articles? --[[User:Ajxs|Ajxs]] 21:38, 5 November 2019 (CST)
````

<a id="w004425"></a>

## W004425 — Talk:MZ

Bron-ID: W004425
Namespace: Talk (1)
Revisie: 24165 | Datum: 2019-12-19T12:32:58Z
Bron: https://wiki.osdev.org/Talk:MZ
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMZ&oldid=24165

````text
== How big is a page? ==

There's a note to say a paragraph is 16 bytes, but nothing to say how big a page is. The size is needed to understand the 2nd and 3rd entries in the file structure. [[User:Eekee|Eekee]] 20:35, 20 November 2019 (CST)

According to http://www.delorie.com/djgpp/doc/exe/, a page appears to be 512 bytes. [[User:A22347|A22347]] 8:36, 22 November 2019 (AEDT)

Thanks! Added. [[User:Eekee|Eekee]] 06:32, 19 December 2019 (CST)
````

<a id="w004426"></a>

## W004426 — File:Klika os.png

Bron-ID: W004426
Namespace: File (6)
Revisie: 24138 | Datum: 2019-12-04T21:44:34Z
Bron: https://wiki.osdev.org/File:Klika_os.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AKlika_os.png&oldid=24138

````text

````

<a id="w004427"></a>

## W004427 — PCI Local Bus Commands

Bron-ID: W004427
Namespace: Main (0)
Revisie: 27901 | Datum: 2023-07-09T04:53:58Z
Bron: https://wiki.osdev.org/PCI_Local_Bus_Commands
Revisie-URL: https://wiki.osdev.org/index.php?title=PCI_Local_Bus_Commands&oldid=27901

````text
= Prologue =

This page seeks to describe the PCI local bus commands -- these commands are sent over the C/BE signal pins from a master to a target.

This page is not meant to be an exhaustive description but rather a quick refresher for the use of someone who has already read the PCI Local Bus specification. This page references signals documented in [[User:Gravaera/PCI Local Bus Signals|this article here]] and recommends it as prior reading.

== PCI C/BE commands ==

''All commands except for the Configuration commands are optional. All devices other than host bridges are _required_ to respond to configuration commands.''

{|class="wikitable"
|-
! C/BE Value
! Name
! Short Description
|-
| 0b0000
| Interrupt Acknowledge
|-
| 0b0001
| Special Cycle
|-
| 0b0010
| I/O Read
|
|-
| 0b0011
| I/O Write
|
|-
| 0b0100
| Reserved
|
|-
| 0b0101
| Reserved
|
|-
| 0b0110
| Memory Read
|
|-
| 0b0111
| Memory Write
|
|-
| 0b1000
| Reserved
|
|-
| 0b1001
| Reserved
|
|-
| 0b1010
| Configuration Read
|
|-
| 0b1011
| Configuration Write
|
|-
| 0b1100
| Memory Read Multiple
|
|-
| 0b1101
| Dual Address Cycle
|
|-
| 0b1110
| Memory Read Line
|
|-
| 0b1111
| Memory Write and Invalidate
|
|}

== General requirements ==

=== Memory commands ===

* ''IF'' a target implements the memory read/write commands, then it must support '''all''' of them, whether explicitly or by emulating the more complex ones internally by mapping them behind the scenes to repetitions of the basic ones.

* Devices between the master and the target (including the master and target) which prefetch data into a buffer or otherwise cache data to present burst behaviour must conform to the PCI specification's constraints on "prefetchable vs non-prefetchable memory". Prefetchable memory is defined as follows:
** When read, it has '''no''' side effects (such as advancing a hardware cursor in a device register, or acknowledging an interrupt).
** Bridge devices between the master and target may merge reads together harmlessly.
** When being read, the target may safely return all bytes in a DWORD irrespective of the Byte Enables.

* For the cache-line read/write commands, the target must read/write to the end of the containing cache line and not beyond, irrespective of whether or not the offset of the read/write was aligned to the beginning of said cache-line.

* If a master is reading/writing from/to a target which does not implement the Cacheline-Size register, the master should '''assume a cache line size of 16 or 32 bytes''' and proceed accordingly.

=== Prefetching ===

* A bridge between a master and target which prefetches is responsible for discarding/invalidating prefetched data which ends up not being consumed by the master. The second "Implementation Note" in section 3.1.2 strongly implies that a PCI master should be able to poll a memory address and not worry about receiving stale data from the bridges between itself and the target.

== Caveats for each command, if they exist ==

=== Memory Read Line and Memory Read Multiple ===

* The master assumes the burden of knowing whether or not the desired address range is prefetchable, which allows any bridge between the master and target to prefetch. Burst read/write commands should not be issued to a non-prefetchable range.

[[Category:PCI]]
[[Category:PCI Local Bus]]
````

<a id="w004428"></a>

## W004428 — User:Gravaera/PCI Local Bus Device Adress Spaces

Bron-ID: W004428
Namespace: User (2)
Revisie: 24135 | Datum: 2019-12-02T22:43:54Z
Bron: https://wiki.osdev.org/User:Gravaera/PCI_Local_Bus_Device_Adress_Spaces
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FPCI_Local_Bus_Device_Adress_Spaces&oldid=24135

````text
[[Category:PCI Local Bus]]

= Prologue =

This page seeks to provide a quick reference for a person who has already read the PCI Local Bus Specification and is not meant to serve as an exhaustive description of the PCI Local Bus.

= Address Configuration for device address space blocks =

PCI devices may export one or more address space blocks or regions. The device must specify for each block, the size of the block and which address space it must be allocated into by the host OS software. The device does not determine where within the requested address space it will end up being allocated to. The address allocated by the host OS is written into the device's BARs (Base address registers).

PCI SIG cautions creators of new devices to prefer to request blocks in the Memory Address space and not the I/O address space since this will increase their portability.
````

<a id="w004429"></a>

## W004429 — Category:PCI Local Bus

Bron-ID: W004429
Namespace: Category (14)
Revisie: 24137 | Datum: 2019-12-02T22:46:03Z
Bron: https://wiki.osdev.org/Category:PCI_Local_Bus
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3APCI_Local_Bus&oldid=24137

````text
[[Category:PCI]] [[Category:Buses]]
````

<a id="w004430"></a>

## W004430 — User:Superleaf1995/AFPFS

Bron-ID: W004430
Namespace: User (2)
Revisie: 24189 | Datum: 2019-12-24T21:59:00Z
Bron: https://wiki.osdev.org/User:Superleaf1995/AFPFS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASuperleaf1995%2FAFPFS&oldid=24189

````text
'''AFPFS''' abritary file positioning file system is a theoric and proven (in my own personal DOS-alike OS) file system. 

== Bootloader size record ==
I was able to manage to make a 62 byte long bootloader with AFPFS. It was very stripped out of functions, but it got to load the kernel properly 

== Bootsector ==
The bootsector has information like bytes per sectors, heads, and label. Then the boot thingy.


== Files ==
The FS views the disk as a array, a big one.

There are no directories, there are only 'groups' in files. Every file arbitraryly uses 0FF8h as  SOF (Start of file), when 0FF8h, means that there is a new file. 0FF9h means that this is the last file in partition and works as an EOF for the entire partition, leaving the empty part untouched until written on, and allowing for multiple partitions.

There are no FATs or whatsoever.

== File entry ==

After the SOF there is file metadata Wich is:

1-byte wide of filename
filename wide as (above byte + 1)

3-bytes wide extension

2-byte indicating group

2-byte indicating member ID in group (same IDs in multiple files means that they are the same file but split up)

And then the file data.

== Conclusion ==

The good of this filesystem is that since there is no file table, the files will always be indexed in disk. Regardless of a corrupt file or sector.

This Filesystem can be simpler than FAT if we remove the groups and tune it a bit.

Still, leave the floppy signature at the end of the boot sector.

Note: Only tested on floppy disk, requires bytecopiers to copy to a floppy image. And have to manually put the file metadata on all files. (ASM simplifies this).

== Notes ==
Obviously, put a jump before boot info.
Put the jump  after or before file metadata, depending on implementation.

== Implementation. ==
A implementation version, truly arbitraryly because there are no IDs or whatsoever, just filenames (see DISK.img in releases)

https://github.com/SuperLeaf1995/LeafDOS/tree/master/SOURCE/KERNEL
````

<a id="w004431"></a>

## W004431 — User talk:Superleaf1995/AFPFS

Bron-ID: W004431
Namespace: User talk (3)
Revisie: 24169 | Datum: 2019-12-19T18:32:42Z
Bron: https://wiki.osdev.org/User_talk:Superleaf1995/AFPFS
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ASuperleaf1995%2FAFPFS&oldid=24169

````text
== Binary files question ==
What happens if a binary file contains 0FF8h or 0FF9h? Is there a way to encapsulate binary data so that it's not scanned for these numbers? (encapsulate = put into a container) [[User:Eekee|Eekee]] 06:22, 19 December 2019 (CST)
* SOF should always be followed by metadata and an ASCII filename, if no such are present, the SOF is ignored. After partition signature 0FF9, there should be a SOF, and to validate it like I mentioned before. --[[User:Superleaf1995|Superleaf1995]] 12:32, 19 December 2019 (CST)

== Metadata ==
* I think the first line should be two lines, "filename length" and "filename". I don't want to edit it because it's a user page. [[User:Eekee|Eekee]] 06:22, 19 December 2019 (CST)
* Does the filename need to be repeated if the member id is the same? Or more generally, what happens if it's different? I'm just looking for trouble, here. ;) [[User:Eekee|Eekee]] 06:22, 19 December 2019 (CST)
* a logical conclusion would be that different filename OR/AND id means a different file. And
That's bad for bootloaders, that's why group Id exists tho, the OS can reserve a special group ID for the kernel (or any important file in general). If it is different, it's a different file. But it's a simple idea , so no specific rules may be followed (it can be a different file or not). I would personally assign different group IDs to files that haves same filename and same I'd as another file. --[[User:Superleaf1995|Superleaf1995]] 12:28, 19 December 2019 (CST)
````

<a id="w004432"></a>

## W004432 — NVMe

Bron-ID: W004432
Namespace: Main (0)
Revisie: 30943 | Datum: 2026-09-07T18:30:41Z
Bron: https://wiki.osdev.org/NVMe
Revisie-URL: https://wiki.osdev.org/index.php?title=NVMe&oldid=30943

````text
Non-Volatile Memory Express is a controller interface for (usually solid-state) storage and used in most modern PCs. Its design is tailored towards modern hardware and software scenarios, allowing parallel and asynchronous enqueueing and completion of commands via queues composed of submission and completion ring buffers. While NVMe is typically used to access drives over [[PCI Express]], newer revisions of the specification also support a NVMe-over-fabrics (NVMe-oF) transport that encapsulates NVMe commands, allowing them to be transmitted to a controller via TCP or Infiniband. The real advantage of this is realised when paired with an enterprise-class network adapter like the [[Intel E810]], which supports RDMA, thus freeing the CPU from this transmission.

It is generally advisable to refer to the specifications for details. This is especially relevant when targeting NVMe drives that support previous revisions of the specification, where some features might not have been supported yet. While newer revisions of the specifications (2.0 and later) have added many new features, they are not really useful for basic operation of the device. For avoiding this added complexity (and the hard-to-follow nature of the spec now being broken up into parts), it is generally recommended to start by working with the 1.3d revision, as this predates the new complexity while offering the features expected of a NVMe driver.

== Overview ==

* NVMe controllers can be found as [[PCI]] devices with class code 1 and subclass code 8.
* Its registers are accessible through BAR 0 (it should be 64-bit memory IO).
* The controller processes commands submitted to it from "submission queues". The driver prepares commands in the queue's circular buffer in memory, and then updates the tail pointer register for the queue. 
* The controller may process commands in any order it likes. 
* When the controller has finished processing a command, it appends an entry to a "completion queue". The completion queue to use is specified when a submission queue is created. The controller sends an interrupt when a completion queue has available commands. The driver processes all new entries in the queue's circular buffer, and then updates the head pointer register for the queue.
* At reset, only one submission queue and one completion queue exists. These are the admin queues. The driver sets their base addresses in the ASQ and ACQ registers. 
* The admin queues can process admin commands, such as creating IO queues (used to submit IO commands, like read/write sectors), and query information about the controller and drives (called "namespaces") connected to it.
* The admin queues always use the Queue Identifier of 0.

== Controller registers ==

<div class="toccolours mw-collapsible mw-collapsed">
'''References'''
<div class="mw-collapsible-content">
* NVMe specification 1.3d, Chapter 3.1 "Register Definition"
* NVMe Base Specification 2.3, Chapter 3.1.4 "Controller Properties": newer revisions abstract away MMIO registers as "Controller Properties", which is useful for using NVMe-over-Fabrics. For now, you only need to worry about the "I/O Controller" column and what it supports.
</div></div>

{| class="wikitable"
|-
!style="white-space:nowrap;" |Offset
!style="white-space:nowrap;" |Name
!style="white-space:nowrap;" |Description
|-
|align="center" |0x00-0x07
|align="center" |CAP
|Controller capabilities.
|-
|align="center" |0x08-0x0B
|align="center" |VS
|Version.
|-
|align="center" |0x0C-0x0F
|align="center" |INTMS
|Interrupt mask set.
|-
|align="center" |0x10-0x13
|align="center" |INTMC
|Interrupt mask clear.
|-
|align="center" |0x14-0x17
|align="center" |CC
|Controller configuration.
|-
|align="center" |0x1C-0x1F
|align="center" |CSTS
|Controller status.
|-
|align="center" |0x24-0x27
|align="center" |AQA
|Admin queue attributes.
|-
|align="center" |0x28-0x2F
|align="center" |ASQ
|Admin submission queue.
|-
|align="center" |0x30-0x37
|align="center" |ACQ
|Admin completion queue.
|-
|align="center" |0x1000+(2Y)*DSTRD
|align="center" |SQxTDBL
|Submission queue X tail doorbell.
|-
|align="center" |0x1000+(2Y+1)*DSTRD
|align="center" |CQxHDBL
|Completion queue X head doorbell.
|}

<code>Y</code> is a Queue Identifier, which is zero for the Admin Queue. <code>DSTRD</code> is the doorbell stride, which is specified in the controller capabilities (<code>CAP</code>) register.

Note that some fields are what the specification refers to as "0's based values". In this case, the numbering is offset by one, so that a value of zero cannot be represented. Setting a value of <code>0</code> encodes the number 1, a value of <code>1</code> encodes the number 2 and so forth.

Also note that the specification lays down some notable restrictions for MMIO:
* The MMIO access to the register space is defined by a 64-bit BAR, i.e. BAR0 contains the lower 32 bits and BAR1 the upper 32 bits of the <code>Memory Register Base Address</code>.
* The MMIO space should be mapped as uncachable, as usual.
* Accesses to the MMIO space should either be using the registers native width, or aligned 32-bit accesses. Anything else is undefined behavior.
* Accesses to more than a single register at a time are not allowed.

== Data structures ==

=== NVMe Queue ===
An NVMe queue is 128 bytes can represent both Admin and IO queues. This isn't standard but is a good way to keep track of your queues (And it's used in the examples below).
{| class="wikitable"
|-
|'''Bits'''
|'''Contents'''
|-
| align="center" |0-63
|Queue Address
|-
|64-127
|Queue Size
|}

=== Submission queue entry ===

A submission queue entry - a command - is 64 bytes, arranged in 16 DWORDs.

{| class="wikitable"
|-
!style="white-space:nowrap;" |DWORD
!style="white-space:nowrap;" |Contents
|-
|align="center" |0
|Command DWORD 0 (see below)
|-
|align="center" |1
|NSID (namespace identifier). If n/a, set to 0.
|-
|align="center" |2-3
|Reserved.
|-
|align="center" |4-5
|Metadata pointer.
|-
|align="center" |6-9
|Data pointer. 2 PRPs (see next section).
|-
|align="center" |10-15
|Command specific.
|}

Format of the Command DWORD 0:

{| class="wikitable"
|-
!style="white-space:nowrap;" |Bits
!style="white-space:nowrap;" |Contents
|-
|align="center" |0-7
|Opcode.
|-
|align="center" |8-9
|Fused operation. 0 indicates normal operation.
|-
|align="center" |10-13
|Reserved.
|-
|align="center" |14-15
|PRP or SGL selection. 0 indicates PRPs.
|-
|align="center" |16-31
|Command identifier. This is put in the completion queue entry.
|}

=== PRP ===

A PRP (physical region page) is a 64-bit physical memory address. It must be DWORD aligned. A list of PRPs is used in a data transfer to specify where data is transferred from/to in memory. A PRP list is subject to the follow rules:

* The size of the region specified by a given PRP is the minimum of: the amount of data that can be transferred without crossing a page boundary; and the amount of data remaining to be transferred.
* Only the first entry in a PRP list can be page misaligned.
* If a PRP list is not long enough to cover the entire transfer, then the last entry chains to a page containing more PRP entries.

=== Completion queue entry ===

A completion queue entry is 16 bytes.

{| class="wikitable"
|-
!style="white-space:nowrap;" |Bits
!style="white-space:nowrap;" |Contents
|-
|align="center" |0-31
|Command specific.
|-
|align="center" |32-63
|Reserved.
|-
|align="center" |64-79
|Submission queue head pointer.
|-
|align="center" |80-95
|Submission queue identifier.
|-
|align="center" |96-111
|Command identifier.
|-
|align="center" |112
|Phase bit. Toggled when entry written.
|-
|align="center" |113-127
|Status field. 0 on success.
|}

Where new entries end in the completion queue can be determined by inspecting the phase bit.

== Commands ==

=== Queue Types ===
As you saw near the beginning of this page, you have access to two types of command queues - Admin queues and IO queues. For common purposes, you will need one Admin completion queue and one Admin submission queue, and you will need at least one IO completion queue and one IO submission queue - though, some developers may wish to include multiple IO queues. Admin queues are used to create IO queues and retrieve information about the controller or namespace as shown below, and IO queues are used to perform IO actions on your NVMe controller (Such as reading or writing sectors). To create your admin queues, you need to allocate memory for their addresses. I like to allocate one page for both. You then need to write the addresses of both queues to the appropriate registers. 

==== Example ====

* ''nvme_queue'' represents an NVMe queue described near the beginning of this page.
* In this example, we are using a queue size of 64. (It's zero-based, so we'd set the size field to 63)

<syntaxhighlight lang="C">
bool create_admin_submission_queue(nvme_queue *sq) {
	sq->address = (uint64_t)malloc(PAGE_SIZE);
	if (sq->address == 0)
		return false;
	sq->size = 63;
	// 0x28 is the Admin Submission queue register
	nvme_write_reg(0x28, sq->address);
	return true;
}

bool create_admin_completion_queue(nvme_queue *cq) {
	cq->address = (uint64_t)malloc(PAGE_SIZE);
	if (cq->address == 0)
		return false;
	cq->size = 63;
	// 0x30 is the Admin Completion queue register
	nvme_write_reg(0x30, cq->address);
	return true;
}
</syntaxhighlight>

=== Sending commands ===
This example shows how you can read and write sectors from an NVMe disk. This assumes you have already used your admin completion queues to create '''two''' IO completion queues. 

* ''QUEUE_SIZE'' is the size of your queues. This value should not be zero-based.
* ''nvme_command_entry'' is a '''submission''' queue entry - this example doesn't handle completion queue entries.
* ''completion_queue_head'' is the IO completion queue head.
* ''submission_queue_tail'' is the IO submission queue tail.
* ''nvme_doorbell_strd'' is the NVMe doorbell stride.

<syntaxhighlight lang="C">
bool nvme_send_command(uint8_t opcode, uint32_t nsid, void *data, uint64_t lba, uint16_t num_blocks, nvme_completion *completion) {
	uint64_t sq_entry_addr = submission_queue.address + (submission_queue_tail * sizeof(nvme_command_entry));
	uint64_t cq_entry_addr = completion_queue.address + (completion_queue_head * sizeof(nvme_completion));
	nvme_command_entry command_entry;
	command_entry.opcode = opcode;
	command_entry.nsid = nsid;
	command_entry.prp1 = (uintptr_t)data;
	command_entry.prp2 = 0;
	command_entry.command_specific[0] = (uint32_t)lba;
	command_entry.command_specific[1] = (uint32_t)((uint64_t)lba >> 32);
	command_entry.command_specific[2] = (uint16_t)(num_blocks - 1);
	memcpy((void *)sq_entry_addr, &command_entry, sizeof(nvme_command_entry));
	submission_queue_tail++;
	nvme_write_reg(0x1000 + 2 * (4 << nvme_doorbell_strd), submission_queue_tail);
	if (submission_queue_tail == QUEUE_SIZE)
		submission_queue_tail = 0;
	// You should wait for completion here
	map_page(cq_entry_addr);
	completion = (nvme_completion *)cq_entry_addr;
	completion_queue_head++;
	nvme_write_reg(0x1000 + 3 * (4 << nvme_doorbell_strd), completion_queue_head);
	if (completion_queue_head == QUEUE_SIZE)
		completion_queue_head = 0;
	return completion->status != 0;
}

bool nvme_read(uint64_t lba, uint32_t sector_count, void *buffer) {
	nvme_completion *completion = NULL;
	if (nvme_send_command(0x02, nsid, buffer, lba, sector_count, completion) != NVME_SUCCESS)
	return true;
	if (completion->status != NVME_SUCCESS)
		return true;
	return false;
}

bool nvme_write(uint64_t lba, uint32_t sector_count, void *buffer) {
	nvme_completion *completion = NULL;
	if (nvme_send_command(0x01, nsid, buffer, lba, sector_count, completion) != NVME_SUCCESS)
	return true;
	if (completion->status != NVME_SUCCESS)
		return true;
	return false;
}
</syntaxhighlight>

=== Admin commands ===

==== Create IO submission queue ====

* Opcode is 0x01.
* The base address of the queue should be put in the DWORDs 6 and 7 of the commands.
* Command DWORD 10 contains the queue identifier in the low word, and the queue size in the high word. The queue size should be given as one less than the actual value.
* Command DWORD 11 contains flags in the low word, and the completion queue identifier in the high word (where completion entries for this submission queue will be posted). Flag (1 << 0) indicates the queue is physically contiguous (recommended; non-contiguous are not supported by all controllers).

==== Create IO completion queue ====

* Opcode is 0x05.
* The base address of the queue should be put in the DWORDs 6 and 7 of the commands.
* Command DWORD 10 contains the queue identifier in the low word, and the queue size in the high word. The queue size should be given as one less than the actual value.
* Command DWORD 11 contains flags in the low word, and the interrupt vector in the high word. Please note that if you are using MSI/MSI-X the interrupt vector should be the MSI vector + 1 (MSI vector 0 is reserved for the admin completion queues) - the NVMe also specification recommends you use MSI-X unless it isn't available, otherwise you can use regular MSI. Flag (1 << 0) indicates the queue is physically contiguous (recommended; non-contiguous are not supported by all controllers), and flag (1 << 1) enables interrupts.

==== Identify ====

* Opcode is 0x06.
* The base address of the output (a single page) should be put in the DWORDs 6 and 7 of the command.
* The low byte of command DWORD 10 indicates what is to be identified: 0 - a namespace, 1 - the controller, 2 - the namespace list.
* If identifying a namespace, set DWORD 1 to the namespace ID.

=== IO commands ===

==== Read ====

* Opcode is 0x02.
* DWORD 1 contains the NSID.
* DWORDs 6-9 contain the PRP list for the data transfer.
* DWORDs 10-11 contain the starting LBA.
* The low word of DWORD 12 contains the number of blocks to transfer. This should be given as one less than the actual value.

==== Write ====

* Opcode is 0x01.
* DWORD 1 contains the NSID.
* DWORDs 6-9 contain the PRP list for the data transfer.
* DWORDs 10-11 contain the starting LBA.
* The low word of DWORD 12 contains the number of blocks to transfer. This should be given as one less than the actual value.

== Checklist ==

=== Initialisation ===

* Find PCI function with class code 0x01 and subclass code 0x08.
* Enable interrupts, bus-mastering DMA, and memory space access in the PCI configuration space for the function.
* Map BAR0.
* Check the controller version is supported.
* Check the capabilities register for support of the NVMe command set.
* Check the capabilities register for support of the host's page size.
* Reset the controller.
* Set the controller configuration, and admin queue base addresses.
* Start the controller.
* Enable interrupts and register a handler.
* Send the identify command to the controller. Check it is an IO controller. Record the maximum transfer size.
* Reset the software progress marker, if implemented.
* Create the first IO completion queue, and the first IO submission queue.
* Identify active namespace IDs, and then identify individual namespaces. Record their block size, capacity and whether they are read-only.

=== Shutdown ===

* Delete IO queues.
* Inform the controller of shutdown.
* Wait until CSTS.SHST updates.

=== Submitting a command ===

* Build PRP lists.
* Wait for space in the submission queue. The controller indicates its internal head pointer in completion queue entries.
* Setup the command.
* Update the queue tail doorbell register.

=== IRQ handler ===

* For each completion queue, read all entries where the phase bit has been toggled.
* Check the status of the commands. 
* Use the submission queue ID and command ID to work out which submitted command corresponds to this completion entry.
* Update the completion queue head doorbell register.

== Troubleshooting ==

QEMU is immensely useful for debugging the basics of a NVMe driver with [https://qemu-project.gitlab.io/qemu/devel/tracing.html its trace events]. It supplies [https://github.com/qemu/qemu/blob/master/hw/nvme/trace-events a number of useful NVMe trace events], which can be enabled with <code>--trace "pci_nvme*"</code> on a QEMU build that has the tracing support compiled in (which is a configure-time feature).

== See Also ==

=== External Links ===

* [https://nvmexpress.org/specifications/ NVMe Specifications]
** [https://nvmexpress.org/nvm-express-specification-archives/#nvm-express-base-specification Older specification revisions]

=== Forum Threads ===

* [https://forum.osdev.org/viewtopic.php?f=8&t=36366 here]

[[Category:Storage]]
[[Category:Standards]]
````

<a id="w004433"></a>

## W004433 — User:Peyty

Bron-ID: W004433
Namespace: User (2)
Revisie: 29446 | Datum: 2025-03-25T14:21:08Z
Bron: https://wiki.osdev.org/User:Peyty
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APeyty&oldid=29446

````text
Hello there!

My name is PeyTy aka Miraculous Ladybugreport and I'm a developer since ehh... 2007 maybe? I also create tools and apps from start to finish (i.e. full circle), so I've decided to challenge myself with creating a complete operating system, privacy tools and programming language. All open-source, of course.

Projects:

https://github.com/GreenteaOS or https://greenteaos.github.io

https://github.com/hexalang or https://hexalang.github.io

My GitHub page is https://github.com/PeyTy
````

<a id="w004434"></a>

## W004434 — File:GreenteaOS-preview.jpg

Bron-ID: W004434
Namespace: File (6)
Revisie: 29167 | Datum: 2024-06-29T14:31:56Z
Bron: https://wiki.osdev.org/File:GreenteaOS-preview.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AGreenteaOS-preview.jpg&oldid=29167

````text
GreenteaOS one of the GUI prototypes screenshot
````

<a id="w004435"></a>

## W004435 — User:Peyty/GreenteaOS

Bron-ID: W004435
Namespace: User (2)
Revisie: 29611 | Datum: 2025-07-31T12:36:40Z
Bron: https://wiki.osdev.org/User:Peyty/GreenteaOS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APeyty%2FGreenteaOS&oldid=29611

````text
{{Infobox Project
|name=GreenteaOS

|image=GreenteaOS-preview.jpg
|caption=User interface prototype

|developers=[[User:Peyty|Oleh Petrenko]]

|kerneltype=engine
|platform=x86-64, ARM64, RISC-V
|taskmodel=Preemptive multitasking
|language=NASM, Hexa

|website=[https://greenteaos.org greenteaos.org]
}}

'''GreenteaOS''' is a 64-bit desktop OS for x86-64 and emerging platforms like ARM64 and RISC-V. Aims to run .exe files natively and become open-source Windows alternative. Follows own non-NT/non-UNIX monolithic kernel design. Currently in the alpha state. The project emphasizes performance, compatibility, and user-centric design for desktop environments.

== History ==
Greentea OS began in 2016 as a fork of ReactOS, later in 2018 reconsidered to create its own kernel from scratch. Competition should improve development speed of both projects, due to more people involved with different points of view.

First notable complex GUI apps, using Win32 API, like [https://www.youtube.com/watch?v=vh-icbJuQgg Solitaire & Minesweeper games and a Calculator (calc.exe), are operational since June/July 2025], therefore moving the project from the Pre-Alpha into the Alpha stage. Real-time '''news and demos''' can be found on the [https://www.youtube.com/@greenteaos.hexalang project's YouTube channel].
[[File:Demo.jpg|alt=Demo from the July 2025 release|thumb|Demo from the July 2025 release]]
Since 2018 project was focused on research, experimentation and laying out the foundation of the system: kernel was rewritten two times from C then C++ to the Hexa programming language. Additional prototypes of various components were created in D, Rust, TypeScript and Pascal. Project aimed to have basic proof-of-concept implementations of the broad features like SMP, WoW64, etc first before focusing on a full-scale OS design.

2025 is a year of starting a full-scale development, logically following proof-of-concept GUI apps compatibility.

Codebase features lots of TODO comments as of June 2025: they are supposed to be right in the context of problems to be solved. Lots of them show language design experimentation and open, clean acknowledgment of any particular feature (be it OS, PL, GUI or anything else) incompleteness compared to other projects that generally stigmatize TODO comments. This is a deliberate choice and will be adjusted from the future feedback.

=== Forking Clarification ===
While technically being a fork originally, Greentea OS was supposed to pivot from ReactOS components as soon as possible from the very beginning. While some people perceive Tofita-based GTOS as a "new Greentea OS", its still the very same project, following continuation of the process of reducing dependencies from the existing projects, like ReactOS being the most noticeable one.

This shift was motivated by the belief that competition among open-source Windows alternatives, such as ReactOS and Wine, would accelerate innovation and attract diverse perspectives from the developer community.

== Design and Features ==
Greentea OS features own kernel '''Tofita''': a portable desktop-oriented monolithic design with UEFI bootloader.

GreenteaFS (name may change) is also considered to be used as a primary file system. With hybrid approach of CoW and non-CoW structure to achieve maximal performance on consumer desktop PCs (in comparison to most other fs's designed for server use). Sources of GreenteaFS are not yet available due to active experimentation. Somewhat influenced by TFS of Redox. Intended to be ''very'' compatible with NTFS in API terms (like streams support and unicode file names).

GUI is, of course, intended to be hardware accelerated in the future, with GPU drivers as parts of the kernel (i.e. non-userspace). Only support for AMD GCN1+ and DX12-capable Intel is planned, with Nvidia being still closed source and controversial topic (seems like this is going change, due to Nvidia' strategy of Linux kernel driver support). Support for the Windows NT GPU drivers depends on the progress of NT API implementation and driver sandboxing.

Safety to be provided with user (personal) space and apps isolation. Think of a portable user folders and apps in wine-prefix manner.

Kernel was rewritten in a custom programming language Hexa (initial prototypes were in C and C++).

=== Tofita Engine ===
As the system’s core, the Tofita Engine handles process management, memory allocation, services, sandboxing and device drivers in a monolithic structure. Its design prioritizes low-latency interactions for desktop applications, such as gaming or productivity software.

Unlike a conventional monolithic kernel, which consolidates core OS functions (e.g., memory management, process scheduling) into a single executable, the Tofita Engine extends this concept by tightly integrating user-facing components, such as the graphical user interface (GUI) and hardware drivers, into a cohesive framework.

== Critical Reception ==
Greentea OS and ReactOS have diverged due to differing design philosophies, but Greentea OS remains committed to fostering a collaborative open-source ecosystem.

Yet, Greentea OS was somewhat controversial in the eyes of ReactOS community: some members distorted the facts surrounding both projects, creating an impression of a conflict or abuse. This is a hard topic, growing like a snowball from mere misunderstanding, and severed connections between both projects in the past. Still, there is no intention from Greentea OS project to spread any negativity toward ReactOS project or its contributors.

Developers believe that having more options, than solely Wine and ReactOS, will attract more attention from IT community as a whole and to the problem of Windows not delivering high quality software (and, you know, all that telemetry stuff and annoying updates). BTW Windows 7 was recently updated to silently break on pre-SSE2 computers.

If you're still in doubt due to some niche narrative about conflicting state between the projects, check the ReactOS forum: you will find relevant topics where core project members from the both sides share the common sentiment about resolving any misunderstandings.

This complex topic has not diminished Greentea OS's respect for ReactOS's contributions.

== Community and Contribution ==
Greentea OS is an open-source project hosted on [https://github.com/GreenteaOS GitHub], [https://gist.github.com/PeyTy/1f56368d3ab6c5eb7daf580faae3095f welcoming contributions from developers worldwide].

The project encourages involvement in areas like kernel development, optimization, compatibility and GUI design.

=== Hexa Programming Language ===
Hexa was specifically designed with a purpose of easing collaboration: way simpler to use than C++ or Rust, yet providing comparable benefits. Hexa tries to follow the core concept of the C programming language here: simpler to use languages scale well for especially large projects like operating systems.

== Licensing and Legal Terms ==
The project strictly follows the GPL license with all respect to the contributors' hard work, eliminating any possible third-party abuse. This also implies the benefits of the GPL license towards accumulation of efforts, instead of fragmentation.

The reverse engineering (when involved) occurs under the relevant laws, to avoid any ambiguity and "religious" understanding of the reverse engineering approaches.

Greentea OS project de-mystifies [https://gist.github.com/PeyTy/8795d675f96bee4bdf54eb41e75e21cd this issue], by openly citing all the relevant laws and rules, on a per-country basis, as you may see in the  [https://github.com/GreenteaOS/Caramel Caramel] repo. Those rules can be shared between projects (for example [https://exectos.eu.org/contributing/clean-room-guidelines/ ExectOS Wiki]), or at least used as a reference to guide contributor judgments, instead of the witch hunting.

== Links ==
You may found sources at [https://github.com/GreenteaOS GitHub]. Notably [https://github.com/GreenteaOS/Tofita Tofita] kernel ones.

Official website: [https://greenteaos.github.io greenteaos.github.io].
````

<a id="w004436"></a>

## W004436 — Talk:QEMU Monitor

Bron-ID: W004436
Namespace: Talk (1)
Revisie: 28057 | Datum: 2023-07-09T20:20:06Z
Bron: https://wiki.osdev.org/Talk:QEMU_Monitor
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AQEMU_Monitor&oldid=28057

````text
I added a link to this page from the main [[QEMU]] page, under the monitor heading. --[[User:Ajxs|Ajxs]] 01:15, 26 December 2019 (CST)
````

<a id="w004437"></a>

## W004437 — File:ISBN 1695978552.jpg

Bron-ID: W004437
Namespace: File (6)
Revisie: 24253 | Datum: 2020-01-03T04:14:13Z
Bron: https://wiki.osdev.org/File:ISBN_1695978552.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_1695978552.jpg&oldid=24253

````text

````

<a id="w004438"></a>

## W004438 — File:ISBN 0130220051.jpg

Bron-ID: W004438
Namespace: File (6)
Revisie: 24255 | Datum: 2020-01-03T04:17:29Z
Bron: https://wiki.osdev.org/File:ISBN_0130220051.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0130220051.jpg&oldid=24255

````text

````

<a id="w004439"></a>

## W004439 — File:ISBN 012383872X.jpg

Bron-ID: W004439
Namespace: File (6)
Revisie: 24256 | Datum: 2020-01-03T04:18:28Z
Bron: https://wiki.osdev.org/File:ISBN_012383872X.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_012383872X.jpg&oldid=24256

````text

````

<a id="w004440"></a>

## W004440 — File:OS-on-kaleid.png

Bron-ID: W004440
Namespace: File (6)
Revisie: 25724 | Datum: 2021-02-25T20:42:33Z
Bron: https://wiki.osdev.org/File:OS-on-kaleid.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AOS-on-kaleid.png&oldid=25724

````text
The OS/K (OS on Kaleid) screenshot
````

<a id="w004441"></a>

## W004441 — File:Qword-reference-screenshot-1.png

Bron-ID: W004441
Namespace: File (6)
Revisie: 24292 | Datum: 2020-01-12T03:30:57Z
Bron: https://wiki.osdev.org/File:Qword-reference-screenshot-1.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AQword-reference-screenshot-1.png&oldid=24292

````text
This is an example of qword OS running inside QEMU with several CLI application outputs.
````

<a id="w004442"></a>

## W004442 — Basic Theory Of Computer Science

Bron-ID: W004442
Namespace: Main (0)
Revisie: 28637 | Datum: 2024-02-21T23:33:33Z
Bron: https://wiki.osdev.org/Basic_Theory_Of_Computer_Science
Revisie-URL: https://wiki.osdev.org/index.php?title=Basic_Theory_Of_Computer_Science&oldid=28637

````text
{{In_Progress}}

This article is intend to explain to new users, who are unfamiliar with computer theory, basic concepts of computing and computer systems organization and structure.

The idea of the creation of this article was inspired by the general disorganization of such kind of theoretic articles and pages at OSDev.org.
And as long as theory is '''very''' important for beginners ''(and as long as they generally never ever read serious [[books]])'', this chaotic structure of theoretic material will cause problems in comprehensive study of OS development. But this article is meant to fix the problem

It's impossible to study all the topics of physics, circuitry and computer science in within one article. Instead, all these subjects will be briefly touched.
To do so in organized order, we must divide system into layers, each having its own rules and aims. To do so, we must '''manage complexity of the system'''

== Managing the complexity of electronic systems ==
[[File:Abstraction_Layers.png|thumb|300px|right|Levels of abstraction]]
The system you are using to read this article ''(let me guess it's a personal computer)'' is one of most complex systems ever created by the mankind. Just imagine hundreds of billions transistors on several separate integrated circuits joint together and operating at gigahertz rates, running hundreds of millions lines of code in a blink of an eye - hundreds of thousands of computer scientists, engineers and programmers united their efforts to make this miracle come true in less than 80 years...

However, I bet you are not experiencing any kind of trouble with using such a "miracle". The main reason of it is that, despite you don't realize it really, you are '''managing complexity of your system'''. It have let you to open this article in few mouse clicks without having to keep in mind details of, for example, the HTML protocol your browser used to obtain this article from the website. 
Instead, you delegate this task to your ''web-browser'' - this is an '''application software'''[[File:Application_Software.png|20px]], which uses abilities of '''operating system'''[[File:Operating_System.png|20px]] to run, which uses definitions and rules of '''architecture''' [[File:Architecture.png|20px]] layers, and so on

Usually developers define 9 layers of abstraction for an average electronic computing system:
#'''Application Software''' [[File:Application_Software.png|20px]]
#:Programs, which use operating system features to solve user problems
#'''Operating System''' [[File:Operating_System.png|20px]]
#:Operates computer hardware, manage system resources and provide API and drivers for application software
#'''Architecture''' [[File:Architecture.png|20px]] (also known as [[Instruction Set Architecture|ISA]])
#:Describes electronic system from programmer's perspective. Defines instruction set, visible registers. Is implemented by microarchitecture
#'''Microarchitecture''' [[File:Microarchitecture.png|20px]]
#:Implements an architecture using logic elements
#'''Logic''' [[File:Logic.png|20px]]
#:Combines digital circuits to create functional blocks, such as adders, multiplexers and so on.
#'''Digital Circuits''' [[File:Digital_Circuits.png|20px]]
#:Uses analog circuits to create models of the digital circuits, which can be used in digital design
#'''Analog Circuits''' [[File:Analog_Circuits.png|20px]]
#:Combines devices to create circuits with desired properties
#'''Devices''' [[File:Devices.png|20px]]
#:Uses physical models to create devices, such as transistors and diodes, usable in circuits
#'''[[#Physics layer|Physics]]''' [[File:Physics.png|20px]]
#:Explores the world and systematizes obtained knowledge to laws and theories which can be relatively <abbr title="Meaning that they don't have to obtain that knowledge by themselves">easily</abbr> studied by other persons
----
As you can see, each abstraction level use models and features of underlying level (e.g. '''logic''' uses '''digital circuits''' as much as '''application software''' uses '''operating system''' features)
Also, each level hides details of implementation from the overlying level (e.g. user do not have to bother about representing their requests in "computer" form as long as '''application software''' provides the human-understandable GUI)

And it means, that each layer accomplishes two tasks:
*'''Provides simplified interfaces for overlying layers'''
*:For example, there are no ''files'' on the hard disk of your PC - only some bytes written to it in the specified order. ''File'' is just an abstract model of storing files on the physical drive, implemented by a ''filesystem'' - one of the parts of your '''operating system'''. So, ''filesystem'' provides a very convenient '''interface''' for application software, by '''implementing a simple file I/O'''
*'''Hides implementation details from overlying layer and manages them'''
*:For example, ''filesystem'' also contains drivers for different types of storage devices, such as hard drives, optical drives and so on. Each one have its own interface to handle. However, you, as a user, don't want to keep these details in mind. So, '''filesystem''' takes care of it, '''hiding such a details''' from user, '''managing''' all that low-level things by itself

''Since this tutorial was written for OS and applications developers, detailed description of three lower levels of hierarchy will be skipped''

=== Digital Circuits ===
[[File:Analog_Digital_Compare.png|thumb|300px|right|Difference between analog and digital signals being interfered. While analog signal turns out to be totally messed up, digital one still remains understandable]]
As you surely know, all the modern computers are implemented using digital logic, but why?

There are few major reasons:
#Digital signals are more ''stable'' and ''reliable'' than analog ones, because they are less prone for distortions, noises, and interferences.
#Digital circuits are ''easy'' and ''cheap'' to design and implement in hardware using different technologies
#Different digital circuits can be ''easily combined'', creating more complex circuits.

These facts are the main reasons of domination of the digital logic over analog today.

So, what helps digital logic to attain such appealing properties?
It operates only two signal states - LOW and HIGH, instead of wide spectrum of voltages as in analog circuits.

Contrary to popular belief - these do not directly correspond to "lack of voltage" and "some voltage".
This is just not how most electronic components (e.g. transistors) function.
In reality, LOW is any "trivial" voltage, and HIGH is any "nontrivial" voltage.

The states of LOW and HIGH may also be titled 0 and 1.

The end goal of a digital circuit is usually to reach a digital end device, that also categorizes voltages as LOW and HIGH.
Data is usually stored in digital registers.
The data is then changed by electronic components in the circuit, and given back to the registers or the end device.
This allows for implementing various models of computation.

=== Logic ===

To properly modify digital data using a digital circuit, it is useful to have a logical framework with a finite amount of well-defined operations that combined may yield a plethora of results.

The standard logical framework used in modern digital circuitry is [[wikipedia:Boolean_algebra|Boolean algebra]].
A Boolean algebra is a space with two possible values: 0 and 1.
It is equipped with a negation, addition and multiplication.

'''Negation''' is an operation that performed on 0 returns 1 and performed on 1 returns 0.

'''Addition''' is an operation defined by the following:
* 0 + 0 = 0
* 0 + 1 = 1
* 1 + 0 = 1
* 1 + 1 = 1
Furthermore, this operation is associative.

'''Multiplication''' is an operation defined by the following:
* 0 * 0 = 0
* 0 * 1 = 0
* 1 * 0 = 0
* 1 * 1 = 1
This operation is also associative.

In computer science, negation is usually called a '''NOT''' gate, addition is called an '''OR''' gate, and multiplication is called an '''AND''' gate.
They can all be built from simple electronic components.

There are more gates that can be created out of those operations. For example, a well-known gate is '''XOR''', defined for any a, b in a Boolean algebra as not(a) * b + a * not(b).
From a practical perspective, XOR (or the eXclusive OR gate) is a way to perform the OR operation without triggering 1 for the 1,1 case.

These gates, often also called logic gates, can be chained together to create more complex mechanisms.
For example, a common mechanism is called a half-adder.
It adds two bits to each other.
* 0 + 0 = 00
* 0 + 1 = 01
* 1 + 0 = 01
* 1 + 1 = 10

It is very easy to implement. The first bit simply works like an AND gate, while the second bit works like a XOR gate.

[[Category:In Progress|{{PAGENAME}}]]
[[Category:Theory|{{PAGENAME}}]]
````

<a id="w004443"></a>

## W004443 — File:Abstraction Layers.png

Bron-ID: W004443
Namespace: File (6)
Revisie: 24335 | Datum: 2020-01-17T20:34:18Z
Bron: https://wiki.osdev.org/File:Abstraction_Layers.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AAbstraction_Layers.png&oldid=24335

````text
Image taken from the "Digital Design and Computer Architecture" Harris&Harris
````

<a id="w004445"></a>

## W004445 — File:Jsdos-1.gif

Bron-ID: W004445
Namespace: File (6)
Revisie: 24339 | Datum: 2020-01-17T23:02:11Z
Bron: https://wiki.osdev.org/File:Jsdos-1.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AJsdos-1.gif&oldid=24339

````text

````

<a id="w004446"></a>

## W004446 — File:Physics.png

Bron-ID: W004446
Namespace: File (6)
Revisie: 24343 | Datum: 2020-01-18T09:46:14Z
Bron: https://wiki.osdev.org/File:Physics.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3APhysics.png&oldid=24343

````text
Image taken from the "Digital Design and Computer Architecture" Harris&Harris
````

<a id="w004447"></a>

## W004447 — File:Devices.png

Bron-ID: W004447
Namespace: File (6)
Revisie: 24344 | Datum: 2020-01-18T09:46:30Z
Bron: https://wiki.osdev.org/File:Devices.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ADevices.png&oldid=24344

````text
Image taken from the "Digital Design and Computer Architecture" Harris&Harris
````

<a id="w004448"></a>

## W004448 — File:Analog Circuits.png

Bron-ID: W004448
Namespace: File (6)
Revisie: 24345 | Datum: 2020-01-18T09:46:51Z
Bron: https://wiki.osdev.org/File:Analog_Circuits.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AAnalog_Circuits.png&oldid=24345

````text
Image taken from the "Digital Design and Computer Architecture" Harris&Harris
````

<a id="w004449"></a>

## W004449 — File:Digital Circuits.png

Bron-ID: W004449
Namespace: File (6)
Revisie: 24346 | Datum: 2020-01-18T09:47:07Z
Bron: https://wiki.osdev.org/File:Digital_Circuits.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ADigital_Circuits.png&oldid=24346

````text
Image taken from the "Digital Design and Computer Architecture" Harris&Harris
````

<a id="w004450"></a>

## W004450 — File:Logic.png

Bron-ID: W004450
Namespace: File (6)
Revisie: 24347 | Datum: 2020-01-18T09:47:23Z
Bron: https://wiki.osdev.org/File:Logic.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ALogic.png&oldid=24347

````text
Image taken from the "Digital Design and Computer Architecture" Harris&Harris
````

<a id="w004451"></a>

## W004451 — File:Microarchitecture.png

Bron-ID: W004451
Namespace: File (6)
Revisie: 24348 | Datum: 2020-01-18T09:47:40Z
Bron: https://wiki.osdev.org/File:Microarchitecture.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AMicroarchitecture.png&oldid=24348

````text
Image taken from the "Digital Design and Computer Architecture" Harris&Harris
````

<a id="w004452"></a>

## W004452 — File:Architecture.png

Bron-ID: W004452
Namespace: File (6)
Revisie: 24349 | Datum: 2020-01-18T09:47:57Z
Bron: https://wiki.osdev.org/File:Architecture.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AArchitecture.png&oldid=24349

````text
Image taken from the "Digital Design and Computer Architecture" Harris&Harris
````

<a id="w004453"></a>

## W004453 — File:Operating System.png

Bron-ID: W004453
Namespace: File (6)
Revisie: 24350 | Datum: 2020-01-18T09:48:13Z
Bron: https://wiki.osdev.org/File:Operating_System.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AOperating_System.png&oldid=24350

````text
Image taken from the "Digital Design and Computer Architecture" Harris&Harris
````

<a id="w004454"></a>

## W004454 — File:Application Software.png

Bron-ID: W004454
Namespace: File (6)
Revisie: 24351 | Datum: 2020-01-18T09:48:28Z
Bron: https://wiki.osdev.org/File:Application_Software.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AApplication_Software.png&oldid=24351

````text
Image taken from the "Digital Design and Computer Architecture" Harris&Harris
````

<a id="w004455"></a>

## W004455 — File:Threshold Formation MOSFET.gif

Bron-ID: W004455
Namespace: File (6)
Revisie: 24362 | Datum: 2020-01-20T11:52:07Z
Bron: https://wiki.osdev.org/File:Threshold_Formation_MOSFET.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AThreshold_Formation_MOSFET.gif&oldid=24362

````text

````

<a id="w004456"></a>

## W004456 — File:ISBN 1717425364.jpg

Bron-ID: W004456
Namespace: File (6)
Revisie: 24371 | Datum: 2020-01-25T20:14:00Z
Bron: https://wiki.osdev.org/File:ISBN_1717425364.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_1717425364.jpg&oldid=24371

````text
USB: The Universal Serial Bus
````

<a id="w004457"></a>

## W004457 — File:Latest gui.png

Bron-ID: W004457
Namespace: File (6)
Revisie: 24374 | Datum: 2020-01-25T20:37:04Z
Bron: https://wiki.osdev.org/File:Latest_gui.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ALatest_gui.png&oldid=24374

````text
GUI Screen Shot
````

<a id="w004458"></a>

## W004458 — Roll Your Own Bootloader

Bron-ID: W004458
Namespace: Main (0)
Revisie: 24379 | Datum: 2020-02-03T01:45:51Z
Bron: https://wiki.osdev.org/Roll_Your_Own_Bootloader
Revisie-URL: https://wiki.osdev.org/index.php?title=Roll_Your_Own_Bootloader&oldid=24379
Doorverwijzing: Rolling Your Own Bootloader

````text
#REDIRECT [[Rolling Your Own Bootloader]]
````

<a id="w004459"></a>

## W004459 — U-Boot

Bron-ID: W004459
Namespace: Main (0)
Revisie: 30207 | Datum: 2026-03-12T15:41:11Z
Bron: https://wiki.osdev.org/U-Boot
Revisie-URL: https://wiki.osdev.org/index.php?title=U-Boot&oldid=30207

````text
{{stub}}

'''Das U-Boot''' (or more simply just '''U-Boot''') is an open source generic bootloader mainly used in small-footprint systems, such as Chromebooks and single-board computers. Implementations exist for [[x86]] PCs and Chromebooks (where it is often the default bootloader) and [[PowerPC]] Amigas, as well as several [[ARM]], [[MIPS]], [[Blackfin]], and [[RISC-V]] SBCs.

== See Also ==
===External Links===
* [[Wikipedia:Das U-Boot|Wikipedia article on U-Boot]]

[[Category:Bootloaders]]
````

<a id="w004460"></a>

## W004460 — User talk:Galaxyx

Bron-ID: W004460
Namespace: User talk (3)
Revisie: 24403 | Datum: 2020-02-24T14:36:18Z
Bron: https://wiki.osdev.org/User_talk:Galaxyx
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AGalaxyx&oldid=24403

````text
hey
````

<a id="w004461"></a>

## W004461 — User:Ajxs/ada bare bones

Bron-ID: W004461
Namespace: User (2)
Revisie: 24487 | Datum: 2020-03-06T03:00:56Z
Bron: https://wiki.osdev.org/User:Ajxs/ada_bare_bones
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAjxs%2Fada_bare_bones&oldid=24487

````text
''The aim of this tutorial is to produce a small Multiboot kernel written in the Ada programming language.''

==Preface==

One of the first things people ask on the Ada IRC channel on Freenode is ''"Can Ada be used for OS development?"'' to which the answer is a resounding '''yes'''.

=== Prerequisites ===

The following section details the prerequisite software necessary for successfully building the kernel.

==== Cross-Compiler ====

{{Main|GNAT Cross-Compiler}}

This tutorial requires the use of a [[GCC_Cross-Compiler|cross-compiler]] with Ada support targeting the <tt>i686-elf</tt> architecture. Theoretically, any Ada compiler capable of producing bare-metal ELF binaries for this architecture is suitable. The recommended compiler is [https://www.gnu.org/software/gnat/ GNAT], which is part of the GNU Compiler Collection maintained by the Free Software Foundation. Historically it was difficult to create cross-compiler builds for GNAT, but recent versions of GCC have made this much simpler. For a tutorial on how to set up a GNAT cross-compiler, refer to [[GNAT Cross-Compiler|this article]].

==== GPRbuild ====

This tutorial will require the use of the [https://docs.adacore.com/gprbuild-docs/html/gprbuild_ug.html GPRBuild] utility to build GNAT project files. GPRbuild is a generic build tool designed for the construction of multi-language systems. It is included with AdaCore's distribution of their proprietary GNAT compiler: [https://www.adacore.com/download] or can be built from source files obtained from AdaCore's [https://github.com/AdaCore/gprbuild github profile]. Older versions of GNAT distributed by the Free Software Foundation were able to build GNAT project files using the <tt>gnatmake</tt> utility, however newer versions do not support all the required features.

The cross-compiler mentioned above will need to be properly integrated with GPRbuild and its associated tools. This can be accomplished by following the guide detailed [[GNAT_Cross-Compiler#GPRbuild_Integration|here]].

== Ada run-time library ==
{{Main|Ada Runtime Library}}

The Ada run-time library is responsible for the implementation of the standard library as defined in the [http://ada-auth.org/standards/12rm/html/RM-TOC.html Ada Language Reference Manual]. Not all features outlined in the reference manual need to be implemented for every platform. Since the kernel will be running in a freestanding environment without the support of an underlying operating system, it will be built with what is known as a ''a zero-footprint run-time system'' (commonly abbreviated as 'ZFP'). A zero-footprint RTS will typically provide only the bare-minimum functionality for the target architecture.

The following steps will detail how to create an initial zero-footprint runtime library suitable for building the kernel.

The first step is to set up the directory structure to hold the runtime library. GNAT expects the runtime library to conform to the a set directory structure. It requires the presence of two directories: <tt>adalib</tt> and <tt>adainclude</tt>. <tt>adainclude</tt> contains the runtime package specifications and source files, which are used analogously to include files in C. <tt>adalib</tt> contains the compiled binary artifacts of the runtime library.

The following code demonstrates setting up a directory structure for the RTS:

<source lang="bash">
mkdir -p bare_bones/src/pc
cd bare_bones
mkdir -p rts/boards/i386/adalib
mkdir -p rts/boards/i386/adainclude
mkdir -p rts/src
mkdir -p obj
</source>

The best way to begin creating a new run-time library is to base it upon the existing source files obtained from the system GNAT installation, modifying them where necessary.

The following code demonstrates copying the required files from the host system's GNAT installation's run-time library into <tt>rts/src</tt> and then creating symbolic links to <tt>rts/boards/${arch}/adainclude</tt>. Where <tt>${arch}</tt> is <tt>i386</tt>, <tt>armv6</tt>, etc. The source location will need to be modified to reflect the location of the system compiler.

<source lang="bash">
system_compiler_dir="/usr/lib/gcc/x86_64-linux-gnu/8"
rts_dir="${PWD}/rts"

for f in "ada.ads" "a-unccon.ads" "a-uncdea.ads" "gnat.ads" "g-souinf.ads" \
"interfac.ads" "s-atacco.adb" "s-atacco.ads" "s-maccod.ads" "s-stoele.adb" \
"s-stoele.ads"
do
	cp "${system_compiler_dir}/adainclude/$f" "${rts_dir}/src/"
	ln -s "${rts_dir}/src/$f" "${rts_dir}/rts/boards/i386/adainclude/$f"
done
</source>

==Files==

===gnat.adc===

This file in the root directory of the build tells GNAT there are some configuration pragmas to apply to the build. These pragmas can also be placed at the start of your custom sytem.ads (see below), but we'll place them here for now.

'''Note:''' ''Do not use '''pragma No_Run_Time''' as it is obsolete and has been for a number of years now!''

What these do is to tell GNAT how much of the RTS we can use in our kernel, which is not a lot really.

<source lang="ada">
pragma Discard_Names;
pragma Restrictions (No_Enumeration_Maps);
pragma Normalize_Scalars;
pragma Restrictions (No_Exception_Propagation);
pragma Restrictions (No_Finalization);
pragma Restrictions (No_Tasking);
pragma Restrictions (No_Protected_Types);
pragma Restrictions (No_Delay);
pragma Restrictions (No_Recursion);
pragma Restrictions (No_Allocators);
pragma Restrictions (No_Dispatch);
pragma Restrictions (No_Implicit_Dynamic_Code);
pragma Restrictions (No_Secondary_Stack);
</source>

The final version of [https://github.com/Lucretia/bare_bones/blob/master/gnat.adc gnat.adc] can be downloaded from GitHub.

By passing the '''-r''' flag to the binder (inside the bare_bones.gpr file), the binder will list further restrictions you can apply to enforce further checks.

<source lang="ada">
   package Binder is
      for Default_Switches ("Ada") use ("-r");
   end Binder;
</source>

Between the GNAT Reference Manual and the Ada 2005 Reference Manual, you can find out what the various pragmas are and what they do.

====Discard_Names====

In Ada, the compiler generates strings for various data types, e.g. enumerations, these strings can then be used in I/O.

<source lang="ada">
type Fruit is (Orange, Banana, Apple);
--  Ada defines the following strings, "Orange", "Banana" and "Apple" in an array.

--  These strings can be accessed using the 'Image attribute, as in
Put (Fruit'Image (Orange));
--  Prints "Orange" to the console.
</source>

This pragma tells the compiler not to generate these tables.

====Normalize_Scalars====

Forces all scalars to be initialised, see the latest [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/Pragma-Normalize_005fScalars.html#Pragma-Normalize_005fScalars GNAT RM:Normalize_Scalars] for more information.

====No_Exception_Propagation====

This forces the compiler to disallow any attempt to raise an exception over a subprogram boundary. All exceptions are caught with the Last_Chance_Handler subprogram. See [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/No_005fException_005fPropagation.html#No_005fException_005fPropagation GNAT RM:No_Exception_Propagation] for more information.

The [http://docs.adacore.com/gnat-hie-docs/html/gnathie_ug_3.html#SEC8 GNAT High Integrity Edition] documentation states the folowing:

<blockquote>
Exception declarations and raise statements are still permitted under this restriction. A raise statement is compiled into a call of __gnat_last_chance_handler.
</blockquote>

I have been unable, thus far, to raise my own exceptions, although I can declare one at library level. On placing a declaration inside console.ads:

<source lang="ada">
   TE : exception;  --  A badly named exception.
</source>

and raising inside bare_bones.adb:

<source lang="ada">
   raise Console.TE;
exception
   when Console.TE =>
      Put ("TE caught", 1, 2);
</source>

and upon compiling, I get the following:

<pre>
bare_bones.adb:17:04: construct not allowed in configurable run-time mode
bare_bones.adb:17:04: file a-except.ads not found
bare_bones.adb:17:04: entity "Ada.Exceptions.Raise_Exception" not available
gnatmake: "/home/laguest/src/mine/bare_bones/src/bare_bones.adb" compilation error
make: *** [disk/boot/bare_bones] Error 4
</pre>

so the compiler is looking for the Ada.Exceptions package which defines the normal language standard exception handling, not the cut down version we are using here. Do the same with one of the language defined exceptions works fine. ''I will file a bug at FSF for this as I believe it to be incorrect''.

I have had it confirmed via a [http://gcc.gnu.org/bugzilla/show_bug.cgi?id=53684 bug I filed] that the programmer is not allowed to declare their own exceptions and catch them. I have experimented by creating a version of the Ada.Exceptions (a-except.ad[sb]) package:

<source lang="ada">
with System;

package Ada.Exceptions is
   pragma Preelaborate (Exceptions);

   type Exception_Id is private;
   pragma Preelaborable_Initialization (Exception_Id);

   Null_Id : constant Exception_Id;

   procedure Raise_Exception
     (E       : Exception_Id;
      Message : String := "");
   pragma No_Return (Raise_Exception);
private
   type Exception_Id is new Natural;

   Null_Id : constant Exception_Id := Exception_Id'First;
end Ada.Exceptions;

with GNAT.Source_Info;
with Last_Chance_Handler;

package body Ada.Exceptions is
   procedure Raise_Exception
     (E       : Exception_Id;
      Message : String := "") is
      pragma Unreferenced (E);
      pragma Unreferenced (Message);
      File             : String := GNAT.Source_Info.File;
      Line             : Positive := GNAT.Source_Info.Line;
      Source_Location  : String := GNAT.Source_Info.Source_Location;
      Enclosing_Entity : String := GNAT.Source_Info.Enclosing_Entity;
      pragma Unreferenced (File, Line, Source_Location, Enclosing_Entity);
   begin
      Last_Chance_Handler (System.Null_Address, 0);

      loop
         null;
      end loop;
   end Raise_Exception;
end Ada.Exceptions;
</source>

This seemed to work until I tried to catch the exception inside bare_bones.adb at which point the compiler threw up a number of warnings, resulting in a compile failure:

<pre>
bare_bones.adb:23:17: warning: pragma Restrictions (No_Exception_Propagation) in effect
bare_bones.adb:23:17: warning: "Constraint_Error" may call Last_Chance_Handler
bare_bones.adb:30:07: warning: pragma Restrictions (No_Exception_Propagation) in effect
bare_bones.adb:30:07: warning: Last_Chance_Handler will be called on exception
bare_bones.adb:33:17: warning: pragma Restrictions (No_Exception_Propagation) in effect
bare_bones.adb:33:17: warning: "Constraint_Error" may call Last_Chance_Handler
bare_bones.adb:38:20: warning: pragma Restrictions (No_Exception_Propagation) in effect
bare_bones.adb:38:20: warning: "Constraint_Error" may call Last_Chance_Handler
bare_bones.adb:44:20: warning: pragma Restrictions (No_Exception_Propagation) in effect
bare_bones.adb:44:20: warning: "Constraint_Error" may call Last_Chance_Handler
bare_bones.adb:50:20: warning: pragma Restrictions (No_Exception_Propagation) in effect
bare_bones.adb:50:20: warning: "Constraint_Error" may call Last_Chance_Handler
bare_bones.adb:56:20: warning: pragma Restrictions (No_Exception_Propagation) in effect
bare_bones.adb:56:20: warning: "Constraint_Error" may call Last_Chance_Handler
bare_bones.adb:61:42: warning: pragma Restrictions (No_Exception_Propagation) in effect
bare_bones.adb:61:42: warning: "Constraint_Error" may call Last_Chance_Handler
bare_bones.adb:86:21: warning: pragma Restrictions (No_Exception_Propagation) in effect
bare_bones.adb:86:21: warning: "Constraint_Error" may call Last_Chance_Handler
bare_bones.adb:86:30: warning: pragma Restrictions (No_Exception_Propagation) in effect
bare_bones.adb:86:30: warning: "Constraint_Error" may call Last_Chance_Handler
bare_bones.adb:86:39: warning: pragma Restrictions (No_Exception_Propagation) in effect
bare_bones.adb:86:39: warning: "Constraint_Error" may call Last_Chance_Handler
bare_bones.adb:89:26: warning: pragma Restrictions (No_Exception_Propagation) in effect
bare_bones.adb:89:26: warning: "Constraint_Error" may call Last_Chance_Handler
bare_bones.adb:97:20: warning: pragma Restrictions (No_Exception_Propagation) in effect
bare_bones.adb:97:20: warning: "Constraint_Error" may call Last_Chance_Handler
bare_bones.adb:104:20: warning: pragma Restrictions (No_Exception_Propagation) in effect
bare_bones.adb:104:20: warning: "Constraint_Error" may call Last_Chance_Handler
bare_bones.adb:110:20: warning: pragma Restrictions (No_Exception_Propagation) in effect
bare_bones.adb:110:20: warning: "Constraint_Error" may call Last_Chance_Handler
bare_bones.adb:124:20: warning: pragma Restrictions (No_Exception_Propagation) in effect
bare_bones.adb:124:20: warning: "Constraint_Error" may call Last_Chance_Handler
bare_bones.adb:130:20: warning: pragma Restrictions (No_Exception_Propagation) in effect
bare_bones.adb:130:20: warning: "Constraint_Error" may call Last_Chance_Handler
bare_bones.adb:136:20: warning: pragma Restrictions (No_Exception_Propagation) in effect
bare_bones.adb:136:20: warning: "Constraint_Error" may call Last_Chance_Handler
bare_bones.adb:142:20: warning: pragma Restrictions (No_Exception_Propagation) in effect
bare_bones.adb:142:20: warning: "Constraint_Error" may call Last_Chance_Handler
bare_bones.adb:148:20: warning: pragma Restrictions (No_Exception_Propagation) in effect
bare_bones.adb:148:20: warning: "Constraint_Error" may call Last_Chance_Handler
bare_bones.adb:151:04: warning: pragma Restrictions (No_Exception_Propagation) in effect
bare_bones.adb:151:04: warning: Last_Chance_Handler will be called on exception
bare_bones.adb:160:04: warning: pragma Restrictions (No_Exception_Propagation) in effect
bare_bones.adb:160:04: warning: this handler can never be entered, and has been removed
bare_bones.adb:160:09: violation of restriction "No_Exception_Propagation" at ../gnat.adc:10
gnatmake: "/home/laguest/src/mine/bare_bones/src/bare_bones.adb" compilation error
make: *** [disk/boot/bare_bones] Error 4
</pre>

So the net effect of this is to not bother with this package and only use the language defined exceptions for bare metal programming.

====No_Exception_Registration====

Ensures no stream operations are performed on types declared in Ada.Exceptions, see [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/No_005fException_005fRegistration.html#No_005fException_005fRegistration GNAT RM:No_Exception_Registration] for more information.

====No_Finalization====

Controlled types cannot be used, see [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/No_005fFinalization.html#No_005fFinalization GNAT RM:No_Finalization] for more information.

====No_Tasking====

Turns off tasking, so you cannot define tasks or protected objects or do anything related to tasking, see [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/No_005fTasking.html#No_005fTasking GNAT RM:No_Tasking] for more information.

====No_Protected_Types====

This is pretty much here for reinforcement of the above restriction. See [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/No_005fProtected_005fTypes.html#No_005fProtected_005fTypes GNAT RM:No_Protected_Types] for more information.

====No_Delay====

You cannot use delay statements or the calendar package, see [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/No_005fDelay.html#No_005fDelay GNAT RM:No_Delay] for more information.

====No_Recursion====

Should be self evident, see [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/No_005fRecursion.html#No_005fRecursion GNAT RM:No_Recursion] for more information.

====No_Allocators====

You cannot use dynamic memory, see [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/No_005fAllocators.html#No_005fAllocators GNAT RM:No_Allocators] for more information.

====No_Dispatch====

You cannot call a subprogram using Ada's object-orientated mechanism, see [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/No_005fDispatch.html#No_005fDispatch GNAT RM:No_Dispatch] for more information.

====No_Implicit_Dynamic_Code====

You cannot use nested subprograms or any other features that generate trampolines on the stack, see [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/No_005fImplicit_005fDynamic_005fCode.html#No_005fImplicit_005fDynamic_005fCode GNAT RM:No_Implicit_Dynamic_Code] for more information.

====No_Secondary_Stack====

Without a secondary stack, you cannot return unconstrained types, such as arbitrary length strings, or variant records, see [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/No_005fSecondary_005fStack.html#No_005fSecondary_005fStack GNAT RM:No_Secondary_Stack] for more information.

What this also means is you cannot use the runtime features 'Image and 'Val on any types, this would be useful for sending debugging info to the console, i.e. means you don't have to write your own code for converting strings to/from numbers.

I believe that it would be a good idea to have a small secondary stack defined in the assembler startup code, but define your own System.Secondary_Stack (s-secsta.ad[sb]) package which provides the correct API. Inside this package in it's executable part, you could then import the secondary stack from the assembly code, this would then be executed on elaboration of the package at the start of the kernel's execution.

===system.ads===

Every Ada program must have access to the System package, this essentially tells the compiler what kind of hardware we are building for, therefore there will be 1 system.ads file per architecture your kernel supports.

Copy a system.ads from GCC that matches the target you are working with, in our case this is  gcc-<version>/gcc/ada/system-linux-x86.ads, name it system.ads and place it into rts/boards/i386/adainclude/ we need to edit this a bit.

We don't need to change anything from the top as all the integer sizes should be correct. Go to the private part of the spec and change the following values:

# Command_Line_Args => False
# Configurable_Run_Time => True
# Exit_Status_Supported => False
# Stack_Check_Probes => False
# Suppress_Standard_Library => True
# ZCX_By_Default => False
# GCC_ZCX_Support => False

For more information on these options, see gcc-<version>/gcc/ada/targparm.ads.

Also, add the following line in the private part of the package:

<source lang="ada">
private

   Run_Time_Name : constant String := "Bare Bones Run Time";
</source>

According to targparm.ads, it must be the first thing after the private keyword. It should also show up in error messages in parentheses, but I've not managed to get it to show up thus far. This is useful as it should show you which RTS you are currently using, just in case you configure your build incorrectly.

===Last chance handler===

When you start to write and compile Ada using this custom environment, the compiler will automatically place calls from the runtime into your final binary (this is what the compiler normally does, but we've restricted it a lot). One of these calls is to '''Last_Chance_Handler''' so create 2 new files and place into ''rts/boards/'''<arch>'''/adainclude'', as follows.

====last_chance_handler.ads====

<source lang="ada">
with System;

procedure Last_Chance_Handler
  (Source_Location : System.Address; Line : Integer);
pragma Export (C, Last_Chance_Handler, "__gnat_last_chance_handler");
</source>

====last_chance_handler.adb====

<source lang="ada">
procedure Last_Chance_Handler
  (Source_Location : System.Address; Line : Integer) is
   pragma Unreferenced (Source_Location, Line);
begin
   --  TODO: Add in code to dump the info to serial/screen which
   --  is obviously board specific.
   loop
      null;
   end loop;
end Last_Chance_Handler;
</source>

As you can see, the meat of the handler is actualy a null loop at the moment, this is something you need to complete for your OS kernel and also, per platform.

===Compiling the runtime===

Create a file called gnat.gpr in the root directory and copy the following into it:

<source lang="ada">
library project gnat is
   type Arch_Name is ("i386", "arm");
   type Board_Name is ("pc", "rpi");

   Arch  : Arch_Name  := "i386";
   Board : Board_Name := external ("Board");

   case Board is
      when "pc" =>
         Arch := "i386";
      when "rpi" =>
         Arch  := "arm";
   end case;

   for Source_Dirs use ("rts/boards/" & Arch & "/adainclude");
   for Object_Dir use "obj"; --"rts/boards/" & Arch & "/adalib";

   package Builder is
      Basic_Switches := ("-gnat2005", "-g", "-x", "-a", "-gnatg",
			 "-gnatec=../gnat.adc");

      case Board is
         when "pc" =>
            for Default_Switches ("Ada") use Basic_Switches &
               ("-m32", "-march=i386");
         when "rpi" =>
            for Default_Switches ("Ada") use Basic_Switches &
               ("-march=armv6zk", "-mfpu=vfp", "-mfloat-abi=hard", "-marm",
                "-mcpu=arm1176jzf-s", "-mtune=arm1176jzf-s");
      end case;
   end Builder;

   package Compiler is
      for Default_Switches ("Ada") use ("-O2", "-ffunction-sections", "-fdata-sections");
   end Compiler;

   for Library_Kind use "static";
   for Library_Name use "gnat-4.6";
   for Library_Dir use "rts/boards/" & Arch & "/adalib";
end gnat;
</source>

Now compile with the following command:

<source lang="bash">
gnatmake -XBoard=pc -Pgnat.gpr
</source>

Inside rts/boards/i386/adainclude/ you should have the RTS sources symbolically linked along with the custom last_chance_hander and system files. Inside rts/boards/i386/adalib/ you should have the libgnat-4.6.a and also *.ali matching the source which are required by GNAT.

'''N.B:''' Please note that the above lib might need it's name changing as some OSes might have built libgnat with a version number different to the one I'm using. Yours might be libgnat-4.4.a and GNAT will expect to find that, so change the name in the GPR file accordingly.

===startup.s===

This is PC specific so place this in the src/pc directory.

====GAS====

<source lang="asm">
.global startup                         # Make the startup entry point symbol visible to the linker

# Set up the Multiboot header (see GRUB docs for details)
.set ALIGN,    1<<0                     # Align loaded modules on page boundaries
.set MEMINFO,  1<<1                     # Provide memory map
.set FLAGS,    ALIGN | MEMINFO          # This is the Multiboot 'flag' field
.set MAGIC,    0x1BADB002               # 'magic number' lets your bootloader find the header
.set CHECKSUM, -(MAGIC + FLAGS)         # Checksum required

header:                                 # Must be in the first 8kb of the kernel file
.align 4, 0x90                          # Pad the location counter (in the current subsection) to a 4-byte
                                        # storage boundary. The way alignment is specified can vary with
                                        # host system architecture.
.long MAGIC
.long FLAGS
.long CHECKSUM

# Reserve initial kernel stack space
.set STACKSIZE, 0x4000                  # 0x4000 being 16k.
.lcomm stack, STACKSIZE                 # Reserve 16k stack on a32-bit boundary
.comm  mbd, 4                           # Declare common symbol mbd, allocate it 4-bytes of
                                        # uninitialized memory.
.comm  magic, 4                         # Declare common symbol magic, allocate it 4-bytes of
                                        # uninitialized memory.

startup:
    movl  $(stack + STACKSIZE), %esp    # Set up the stack

# The following saves the contents of the registers as they will likely be
# overwritten because main is not our actual entry point, Bare_Bones is. We
# will use these 2 symbols inside Bare_Bones.

    movl  %eax, magic                   # EAX indicates to the OS that it was loaded by a Multiboot-compliant boot
                                        # loader
    movl  %ebx, mbd                     # EBX must contain the physical address of the Multiboot information
                                        # structure

    call  main                          # Call main (created by gnatbind)

    cli                                 # Disable interrupts. then intentionally hang the system.
                                        # CLI only affects the interrupt flag for the processor on which it is
                                        # executed.

hang:
    hlt                                 # Because the HLT instruction halts until an interrupt occurs, the
                                        # combination of a CLI followed by a HLT is used to intentionally hang
                                        # the computer if the kernel returns.
                                        # HLT is in a loop just in case a single HLT instruction fails to execute
                                        # for some reason, (such as in the case of an NMI).
    jmp   hang
</source>

===multiboot.ads===

I won't show the source to this package as  I have made it more Ada-like and it's quite large, so I will link to the current versions, [https://github.com/Lucretia/bare_bones/blob/master/src/pc/multiboot.ads multiboot.ads] and [https://github.com/Lucretia/bare_bones/blob/master/src/pc/multiboot.adb multiboot.adb].

===Console===

'''Disclaimer:''' I wrote this package a long time ago and have reformatted it using my current Ada programming style. I have not gone too far into the code, so it may not be the best implementation of an console.

The following 2 files give you access to the VGA console at 80x25 characters. As they are PC specific, they go into the src/pc directory.

====console.ads====

<source lang="ada">
with System;

package Console is
   pragma Preelaborate (Console);

   type Background_Colour is
     (Black,
      Blue,
      Green,
      Cyan,
      Red,
      Magenta,
      Brown,
      Light_Grey);

   for Background_Colour use
     (Black      => 16#0#,
      Blue       => 16#1#,
      Green      => 16#2#,
      Cyan       => 16#3#,
      Red        => 16#4#,
      Magenta    => 16#5#,
      Brown      => 16#6#,
      Light_Grey => 16#7#);

   for Background_Colour'Size use 4;

   type Foreground_Colour is
     (Black,
      Blue,
      Green,
      Cyan,
      Red,
      Magenta,
      Brown,
      Light_Grey,
      Dark_Grey,
      Light_Blue,
      Light_Green,
      Light_Cyan,
      Light_Red,
      Light_Magenta,
      Yellow,
      White);

   for Foreground_Colour use
     (Black         => 16#0#,
      Blue          => 16#1#,
      Green         => 16#2#,
      Cyan          => 16#3#,
      Red           => 16#4#,
      Magenta       => 16#5#,
      Brown         => 16#6#,
      Light_Grey    => 16#7#,
      Dark_Grey     => 16#8#,
      Light_Blue    => 16#9#,
      Light_Green   => 16#A#,
      Light_Cyan    => 16#B#,
      Light_Red     => 16#C#,
      Light_Magenta => 16#D#,
      Yellow        => 16#E#,
      White         => 16#F#);

   for Foreground_Colour'Size use 4;

   type Cell_Colour is
      record
         Foreground : Foreground_Colour;
         Background : Background_Colour;
      end record;

   for Cell_Colour use
      record
         Foreground at 0 range 0 .. 3;
         Background at 0 range 4 .. 7;
      end record;

   for Cell_Colour'Size use 8;

   type Cell is
      record
         Char   : Character;
         Colour : Cell_Colour;
      end record;

   for Cell'Size use 16;

   Screen_Width  : constant Natural := 80;
   Screen_Height : constant Natural := 25;

   subtype Screen_Width_Range  is Natural range 1 .. Screen_Width;
   subtype Screen_Height_Range is Natural range 1 .. Screen_Height;

   type Row    is array (Screen_Width_Range)  of Cell;
   type Screen is array (Screen_Height_Range) of Row;

   Video_Memory : Screen;

   for Video_Memory'Address use System'To_Address (16#000B_8000#);

   pragma Import (Ada, Video_Memory);

   procedure Put
     (Char       : in Character;
      X          : in Screen_Width_Range;
      Y          : in Screen_Height_Range;
      Foreground : in Foreground_Colour := White;
      Background : in Background_Colour := Black);

   procedure Put
     (Str        : in String;
      X          : in Screen_Width_Range;
      Y          : in Screen_Height_Range;
      Foreground : in Foreground_Colour := White;
      Background : in Background_Colour := Black);

   procedure Clear (Background : in Background_Colour := Black);
end Console;
</source>

====console.adb====

<source lang="ada">
package body Console is
   procedure Put
     (Char       : in Character;
      X          : in Screen_Width_Range;
      Y          : in Screen_Height_Range;
      Foreground : in Foreground_Colour := White;
      Background : in Background_Colour := Black) is
   begin
      Video_Memory (Y)(X).Char              := Char;
      Video_Memory (Y)(X).Colour.Foreground := Foreground;
      Video_Memory (Y)(X).Colour.Background := Background;
   end Put;

   procedure Put
      (Str        : in String;
       X          : in Screen_Width_Range;
       Y          : in Screen_Height_Range;
       Foreground : in Foreground_Colour := White;
       Background : in Background_Colour := Black) is
   begin
      for Index in Str'First .. Str'Last loop
         Put (Str (Index),
              X + Screen_Width_Range (Index) - 1,
              Y,
              Foreground,
              Background);
      end loop;
   end Put;

   procedure Clear (Background : in Background_Colour := Black) is
   begin
      for X in Screen_Width_Range'First .. Screen_Width_Range'Last loop
         for Y in Screen_Height_Range'First .. Screen_Height_Range'Last loop
            Put (' ', X, Y, Background => Background);
         end loop;
      end loop;
   end Clear;
end Console;
</source>

===bare_bones.adb===

This is platform independent and therefore goes into the src directory.

<source lang="ada">
with Console; use Console;

procedure Bare_Bones is
begin
   Clear;

   Put ("Hello, bare bones in Ada.",
        Screen_Width_Range'First,
        Screen_Height_Range'First);
end Bare_Bones;
pragma No_Return (Bare_Bones);
</source>

===linker.ld===

This is a PC specific script so goes into the src/pc directory.

<pre>
OUTPUT_FORMAT(elf32-i386)

/* Tell the linker which startup code to use, we do this as there is no way to do this (well not easily) from the GNAT tools. */
STARTUP(startup.o)

ENTRY (startup)

SECTIONS
{
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
</pre>

===makefile===

Place this file in the root directory.

<source lang="make">
ARCH		=	i386
RTS_DIR		=	`pwd`/rts/boards/$(ARCH)

ifeq ($(ARCH),i386)
GNATMAKE	=	gnatmake
AS		=	as
ASFLAGS		=	--32 -march=i386

OBJS		=	obj/startup.o obj/multiboot.o obj/console.o
BOARD		=	pc

.PHONY: obj/multiboot.o obj/console.o

endif

all: bare_bones

bare_bones: $(OBJS) src/bare_bones.adb
	$(GNATMAKE) --RTS=$(RTS_DIR) -XBoard=$(BOARD) -Pbare_bones.gpr

obj/startup.o: src/$(BOARD)/startup.s
	$(AS) $(ASFLAGS) src/$(BOARD)/startup.s -o obj/startup.o

.PHONY: clean

clean:
	-rm obj/* *~ bare_bones
</source>

===bare_bones.gpr===

Place this file in the root directory.

<source lang="ada">
project Bare_Bones is
   type Arch_Name is ("i386", "arm");
   type Board_Name is ("pc", "rpi");

   Arch  : Arch_Name  := "i386";
   Board : Board_Name := external ("Board");

   -- TODO: Add in a case statement that adds an arch dir to source.

   case Board is
      when "pc" =>
         for Source_Dirs use ("src", "src/pc");
      when "rpi" =>
         for Source_Dirs use ("src", "src/rpi");
   end case;

   for Object_Dir use "obj";
   for Exec_Dir use ".";
   for Main use ("bare_bones.adb");

   package Builder is
      Basic_Switches := ("-gnat2005", "-g", "-x", "-a", "-gnatg",
                         "-gnatec=../gnat.adc", "-gnaty-I", "-gnaty+d");

      case Board is
         when "pc" =>
            for Default_Switches ("Ada") use Basic_Switches &
               ("-m32", "-march=i386");
         when "rpi" =>
            for Default_Switches ("Ada") use Basic_Switches &
               ("-march=armv6zk", "-mfpu=vfp", "-mfloat-abi=hard", "-marm",
                "-mcpu=arm1176jzf-s", "-mtune=arm1176jzf-s");
      end case;
   end Builder;

   package Compiler is
      case Board is
         when "pc" =>
            for Default_Switches ("Ada") use
               ("-O0", "-g", "-ggdb", "-ffunction-sections", "-fdata-sections");
         when "rpi" =>
            for Default_Switches ("Ada") use
               ("-O0", "-g", "-ggdb", "-ffunction-sections", "-fdata-sections");

      end case;
   end Compiler;

-- To reduce size of final binary.
   package Linker is
      for Default_Switches ("Ada") use
         ("-Wl,--gc-sections", "-static", "-nostartfiles", "-nodefaultlibs",
          "-T../src/" & Board & "/linker.ld", "-v");
   end Linker;
end Bare_Bones;
</source>

==Targetting GNAT to bare metal==

'''NOTE''' As of GCC 4.9.4 the code to make GNAT build for bare metal targets has been put back in, the relevant patch for 4.9.2 can be found [https://github.com/Lucretia/free-ada/blob/develop/files/gcc-4.9.2/0001-Applied-patch-from-latest-trunk-to-fix-bare-metal-bu.patch here]

Then GNAT is built, it automatically builds:

# The compiler
# The runtime
# The tools (gnatmake, gnatlink, gnatls, etc.).

Unfortunately, the toolchain cannot be built for bare machine targets (i.e. <arch>-elf|coff|pe) straight out of the box like a C compiler can be.

To enable this, a number of files need to be modified. I will show what needs to be done to support arm-elf, ready for the next section on the Raspberry Pi. This work is based on work done previously [http://www.zsk.p.lodz.pl/~morawski/Dyplomy/Praca%20dyplomowa%20p.%20Horna.pdf].

===gcc/ada/adaint.c===

===gcc/ada/adaint.h===

===gcc/ada/gcc-interface/Makefile.in===

===gcc/ada/gsocket.h===

===gcc/ada/mlib-tgt-specific-bare.adb===

Can we get rid of this?

===gcc/ada/s-oscons-tmplt.c===

===gcc/ada/system-bare-armel.ads===

===gnattools/configure[.ac]===

==Raspberry Pi==

===Boot process===

As stated in [http://www.raspberrypi.org/phpBB3/viewtopic.php?f=63&t=6685], the RPi boot proces is as follows:

# Power on starts the stage 1 boot loader which is on the SoC, which loads the stage 2 boot loader (bootcode.bin) into L2 cache (thus turning it on).
# bootcode.bin enables SDRAM and loads the stage 3 boot loader (loader.bin).
# loader.bin loads and executes the VideoCore firmware (start.elf).
# start.elf loads config.txt, cmdline.txt and kernel.img.

The config.txt file can contain aline "kernel=<name>" where you can name the kernel image anything you like.

Ideally for development we would use an emulator or some kind of netbooting facility so we don't have to keep switching the SD Card from the Pi to the PC and vice versa, this would get tedious really fast.

===U-Boot===

Seems you need serial access to the board to do this, so I won't be atempting this yet.

By following the information starting on the [http://kernelnomicon.org/?p=92 FreeBSD] porting page, we can build u-boot for RPi.


<source lang="bash">
git clone git://github.com/gonzoua/u-boot-pi.git

cd u-boot-pi
make rpi_b_config
</source>

==Testing==

Make sure you have built the RTS above before this next stage otherwise you won't be able to compile the kernel.

<source lang="bash">
make qemu
</source>

On the QEMU window, it should clear the screen, the the cursor won't move so it will be in the middle of the screen, in the top-left corner will be the message "Hello, bare bones in Ada."

==Source access==

I have created a Git repository on [https://github.com/Lucretia/bare_bones GitHub] containing the source above so you don't have to do it by hand if you don't want to.

In fact there have ben a lot of changes since I started this project and it is a better idea to grab the source from GitHub as it will be more up to date. I will leave the documents above so you can see how it's evolved and also how it works, maybe a bit clearer that it is now.

==Future==

# Implement a secondary stack so we can use 'Image attributes.
# Implement a 64-bit version (non EFI).
# Implement a (U)EFI version.
````

<a id="w004463"></a>

## W004463 — File:Analog Digital Compare.png

Bron-ID: W004463
Namespace: File (6)
Revisie: 24475 | Datum: 2020-03-04T17:56:58Z
Bron: https://wiki.osdev.org/File:Analog_Digital_Compare.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AAnalog_Digital_Compare.png&oldid=24475

````text

````

<a id="w004464"></a>

## W004464 — GNAT Cross-Compiler

Bron-ID: W004464
Namespace: Main (0)
Revisie: 25437 | Datum: 2020-12-14T00:38:28Z
Bron: https://wiki.osdev.org/GNAT_Cross-Compiler
Revisie-URL: https://wiki.osdev.org/index.php?title=GNAT_Cross-Compiler&oldid=25437

````text
{{rating|1}}

This article focuses on creating a [[GCC_Cross-Compiler|cross-compiler]] suitable for cross-compiling Ada code for operating system development.

[https://www.gnu.org/software/gnat/ GNAT] is a fully featured Ada compiler maintained by the Free Software Foundation, which forms part of the [[GCC]] compiler collection. Additionally, there is a proprietary version of the GNAT compiler maintained by [https://www.adacore.com/ AdaCore]. For the remainder of this tutorial, any reference to GNAT will be referring to the FSF version, which is freely available from the Free Software Foundation.

As of the time of writing, GNAT remains the most fully-featured freely available Ada compiler on the market. So this will be used as the basis of the cross-compiler.

== Prerequisites ==

In order to build a GNAT cross-compiler a local installation of both GCC and GNAT is required. Additionally, since GNAT is a front-end to GCC, all of the pre-requisite dependencies necessary for building a GCC cross-compiler are required.

For best results, building a version of GCC/GNAT which is identical to your system GCC/GNAT is recommended. This minimises the chance of incompatibility with the host compiler resulting in errors during the build process.

=== Installing Build Dependencies ===

The table below lists the various build dependencies for building GNAT from source. These can either be built from source, or installed via the package manager in a *nix distribution. The name of the package in Debian based Linux distributions is provided for convenience.

{| {{wikitable}}
|-
! Dependency
| Source Code
| Apt package (Debian, Ubuntu, Mint, WSL, ...)
|-
|GCC
|N/A
|'''<tt>build-essential</tt>'''
|-
|GNAT
|N/A
|'''<tt>gnat</tt>'''
|-
|Make
|N/A
|'''<tt>build-essential</tt>'''
|-
|[https://www.gnu.org/software/bison/ Bison]
|[https://ftp.gnu.org/gnu/bison/]
|'''<tt>bison</tt>'''
|-
|[https://github.com/westes/flex Flex]
|[https://github.com/westes/flex/releases]
|'''<tt>flex</tt>'''
|-
|[https://gmplib.org/ GMP]
|[https://ftp.gnu.org/gnu/gmp/]
|'''<tt>libgmp3-dev</tt>'''
|-
|MPC
|[https://ftp.gnu.org/gnu/mpc/]
|'''<tt>libmpc-dev</tt>'''
|-
|[https://www.mpfr.org/ MPFR]
|[https://ftp.gnu.org/gnu/mpfr/]
|'''<tt>libmpfr-dev</tt>'''
|-
|[https://www.gnu.org/software/texinfo/ Texinfo]
|[https://ftp.gnu.org/gnu/texinfo/]
|'''<tt>texinfo</tt>'''
|}

== Build ==

The following section of this guide details the build instructions for creating a GNAT cross-compiler. This will take the form of a series of shell commands which perform the individual necessary instructions to complete the build. This form is chosen since it provides easily visualised instructions and can be combined to produce an easily reproducible recipe in the form of a single shell script. A <tt>bash</tt> compatible shell is assumed, however the instructions should be easily adaptable to any other common shell scripting format. The following section assumes that the reader is building a compiler on an <tt>x86_64</tt> system running Linux and targeting the <tt>i686-elf</tt> architecture.

=== Preparation ===

The following sets up the base variables used within our build script. The <tt>BUILD_TARGET</tt> variable specifies the [[Target_Triplet|target triplet]] for our build. This is a specific string format used by GNU tools to specify the target system for a build. The <tt>BUILD_PREFIX</tt> variable provides the installation directory prefix for our cross-compiler installation. <tt>${HOME}/opt/cross/</tt> is a common location for installing cross-compilers. Any sensible location can be used here, however sticking to established conventions is recommended for new developers. The <tt>HOST</tt> variable provides the target triplet that identifies our host system.

<source lang="bash">
#!/usr/bin/env bash

# Recipe for building and installing a GNAT capable GCC cross-compiler from `x86-64-linux-elf`
# to `i686-elf` from source.

# The target triplet for the build.
export BUILD_TARGET="i686-elf"
# The install prefix.
export BUILD_PREFIX="${HOME}/opt/cross/${BUILD_TARGET}"
# The host target triplet.
export HOST="x86_64-pc-linux-gnu"

# Update the PATH variable for this script so it includes the build directory.
# This will add our newly built cross-compiler to the PATH.
# After the initial GCC cross-compiler build this initial cross-compiler 
# will be used to build a version with Ada support.
export PATH="${BUILD_PREFIX}/bin:${PATH}"

# The concurrency to use during the build process.
# Adjust this based upon the capabilities of the host system.
concurrency=8

# The directory where local library dependencies are installed.
# This is used to find installed dependencies such as MPFR, GMP, MPC, etc.
# This is the default location for installations using aptitude.
# Adjust this as necessary for the target system.
local_lib_dir="/usr/local"
</source>

This guide assumes that the source for the build prerequisites is stored in the location <tt>${HOME}/src</tt> with another directory <tt>${HOME}/src/build</tt> used as an intermediate build directory prior to installation. If necessary, these directories can be substituted for others in the scripts below by modifying these variables.

<source lang="bash">
# The directory where the source directories are located.
source_dir="${HOME}/src"
# The directory to use as storage for the intermediate build dirs.
build_dir="${HOME}/src/build"
</source>

The below variables define the version of each dependency to build. These correspond to the source directories contained within the parent source directory listed above. These are defined here to facilitate easily changing the versions of the various dependencies in a single place.

<source lang="bash">
# The versions of each dependency to build.
binutils_version="2.32"
gcc_version="8.3.0"
</source>

=== Binutils ===

The following code details the process for building [https://www.gnu.org/software/binutils/ GNU binutils], which is a required component in the cross-compiler build. Binutils provides utilities such as <tt>as</tt>, <tt>ld</tt>, and <tt>objdump</tt> built for the target architecture.

<source lang="bash">
binutils_dir="binutils-${binutils_version}"

cd "${build_dir}" || exit 1

if [[ ! -d "${build_dir}/${binutils_dir}" ]]; then
	mkdir "${build_dir}/${binutils_dir}" || exit 1
fi

cd "${build_dir}/${binutils_dir}" || exit 1

${source_dir}/${binutils_dir}/configure          \
	--target="${BUILD_TARGET}"               \
	--prefix="${BUILD_PREFIX}"               \
	--host="${HOST}"                         \
	--disable-nls                            \
	--disable-multilib                       \
	--disable-shared                         \
        --with-sysroot || exit 1

# Check the host environment.
make configure-host || exit 1
make -j${concurrency} || exit 1
make -j${concurrency} install || exit 1
</source>

Detailed explanations for the selected configuration options can be seen below:

{| {{wikitable}}
|-
! Option
! Explanation
|-
| --target="${BUILD_TARGET}"
| Sets the [[Target_Triplet|target triplet]] for this cross-compiler build.
|-
| --prefix="${BUILD_PREFIX}"
| Sets the install directory for the final build.
|-
| --host="${HOST}"
| Sets the host architecture for the cross-compiler.
|-
| --disable-nls
| Disables internationalisation, which is not necessary for this build.
|-
| --disable-multilib
| Disables multilib support. Specifies that multiple target variant support, such as bi-endianness support, soft/hard float support, etc, will not be added. Assuming this build is targeting <tt>i686-elf</tt> it is not required.
|-
| --disable-shared
| Disables building of shared libraries. Since this is a cross-compiler build, these are not necessary.
|-
| --with-sysroot
| Enable [[sysroot]] support in the cross-compiler. Using the bare argument here without a directory specified will default to an empty directory. Supplying this argument will allow support for integrating a custom sysroot at a later point.
|}

Additional configuration information can be found by calling the autoconf script with the <tt>--help</tt> argument.

=== GCC ===

In order to successfully create a build of GCC with Ada support, an initial GCC cross-compiler for the target architecture is required. This initial bootstrapping build will be built with only C support. This will be used to bootstrap our final GCC build with GNAT support.

<source lang="bash">
gcc_dir="gcc-${gcc_version}"

cd "${build_dir}" || exit 1

if [[ ! -d "${build_dir}/${gcc_dir}" ]]; then
	mkdir "${build_dir}/${gcc_dir}" || exit 1
fi

cd "${build_dir}/${gcc_dir}" || exit 1

${source_dir}/${gcc_dir}/configure          \
	--target="${BUILD_TARGET}"          \
	--prefix="${BUILD_PREFIX}"          \
	--enable-languages="c"              \
	--disable-multilib                  \
	--disable-shared                    \
	--disable-nls                       \
	--with-gmp=${local_lib_dir}         \
	--with-mpc=${local_lib_dir}         \
	--with-mpfr=${local_lib_dir}        \
	--without-headers || exit 1

make -j${concurrency} all-gcc || exit 1
make -j${concurrency} install-gcc || exit 1
</source>

Detailed explanations for the configuration options specific to this step can be seen below:

{| {{wikitable}}
|-
! Option
! Explanation
|-
| --enable-languages="c"
| In order to bootstrap our GNAT build, only C support is required. This will speed up the cross-compiler build.
|-
| --disable-nls
| Disables internationalisation, which is not necessary for this build.
|-
| --disable-multilib
| Disables multilib support. Specifies that multiple target variant support, such as bi-endianness support, soft/hard float support, etc, will not be added. Assuming this build is targeting <tt>i686-elf</tt> it is not required.
|-
| --disable-shared
| Disables building of shared libraries. Since this is a cross-compiler build, these are not necessary.
|-
| --with-gmp/mpc/mpfr
| Instructs the build where to find these library dependencies.
|-
| --without-headers
| Tells GCC not use any target headers from a libc when building the cross-compiler.
|}

Additional configuration information can be found [https://gcc.gnu.org/install/configure.html here].

=== GNAT ===

The code below shows the recipe for the final GCC build which will add Ada support.

<source lang="bash">
gcc_dir="gcc-${gcc_version}"

cd "${build_dir}" || exit 1

if [[ ! -d "${build_dir}/${gcc_dir}" ]]; then
	mkdir "${build_dir}/${gcc_dir}" || exit 1
fi

cd "${build_dir}/${gcc_dir}" || exit 1

${source_dir}/${gcc_dir}/configure          \
	--target="${BUILD_TARGET}"          \
	--prefix="${BUILD_PREFIX}"          \
	--enable-languages="c,c++,ada"      \
	--disable-libada                    \
	--disable-nls                       \
	--disable-threads                   \
	--disable-multilib                  \
	--disable-shared                    \
	--with-gmp=${local_lib_dir}         \
	--with-mpc=${local_lib_dir}         \
	--with-mpfr=${local_lib_dir}        \
	--without-headers || exit 1

make -j${concurrency} all-gcc || exit 1
make -j${concurrency} all-target-libgcc || exit 1
make -j${concurrency} -C gcc cross-gnattools ada.all.cross || exit 1
make -j${concurrency} install-strip-gcc install-target-libgcc || exit 1
</source>


Detailed explanations for the configuration options specific to this step be seen below:

{| {{wikitable}}
|-
! Option
! Explanation
|-
| --enable-languages="c,c++,ada"
| This is where Ada support is added to the compiler. C++ support is required for the build.
|-
| --disable-libada
| Disables building the target system [[Ada Runtime Library]]. GCC does not understand how to build a suitable run-time library. As required, this will need to be built manually.
|}

== Final Setup ==

In order to use the newly built cross-compiler its binaries must be visible on the system's <tt>PATH</tt>. On a Linux system this can be facilitated by adding the following line to the current user's <tt>~/.bashrc</tt> file, or the equivalent file on the host system:

<source lang="bash">
# Adjust the following to reflect the location of the installed cross-compiler build.
export PATH="${HOME}/opt/cross/i686-elf:$PATH"
</source>

== GPRbuild Integration ==

[https://docs.adacore.com/gprbuild-docs/html/gprbuild_ug.html GPRbuild] is a build tool designed for instrumenting the compilation of GNAT projects. It is written and maintained by AdaCore and is installed alongside their GNAT compiler or can be built from source. Its source code is publicly available on [https://github.com/AdaCore/gprbuild AdaCore's Github profile]. Its function is analogous to that of GNU Make. It is capable of compiling projects featuring a variety of source languages such as Ada, C and Assembly, among others. Modern versions of GNAT do not natively feature the capability to compile GNAT project files, and will require the use of GPRbuild.

GPRbuild will not automatically recognise the existence of the new cross-compiler toolchain. Additional steps are necessary before GPRbuild will recognise our target architecture as being valid. This is accomplished by adding configuration information about the newly built toolchain to GPRbuild's 'knowledge base'  linker configuration file. This file instructs GPRbuild how to link executables and libraries using the various toolchains supported by the host system. If GPRbuild was installed as part of an AdaCore GNAT installation, this configuration file will be located at <tt>${prefix}/share/gprconfig/linker.xml</tt>, where <tt>${prefix}</tt> is the location of the GNAT install directory. The easiest way to add a new target architecture is to duplicate an existing toolchain's target configuration. Inside the linker configuration file, search for an existing configuration such as <tt>leon-elf</tt>, and duplicate each entry for the existing configuration. modifying it to reflect the newly built <tt>i686-elf</tt>, or similar, target.

''Note: GPRbuild uses regular expressions to pattern match the target triplets.''  

A truncated example of the new configuration entries can be seen below:

<source lang="xml">
  <configuration>
    <targets>
       <!-- ... -->
       <target name="^i686-elf$" />
    </targets>
    <config>
   for Library_Support  use "static_only";
   for Library_Builder  use "${GPRCONFIG_PREFIX}libexec/gprbuild/gprlib";
    </config>
  </configuration>

  <!-- ... -->

  <configuration>
    <targets>
      <target name="^i686-elf$" />
    </targets>
    <config>
   for Archive_Builder  use ("i686-elf-ar", "cr");
   for Archive_Builder_Append_Option use ("q");
   for Archive_Indexer  use ("i686-elf-ranlib");
   for Archive_Suffix   use ".a";
    </config>
  </configuration>
</source>

Once each entry has been duplicated and modified to reference the newly built toolchain, paying special attention to preserve any regex special characters, the newly built toolchain should be visible to GPRbuild.

Additional information on GPRbuild configuration, and GPRbuild's toolchain knowledge base, can be found here: [https://docs.adacore.com/gprbuild-docs/html/gprbuild_ug/companion_tools.html]

[[Category:Ada]]
[[Category:Compilers]]
[[Category:Tutorials]]
````

<a id="w004465"></a>

## W004465 — File:Bsod xp.png

Bron-ID: W004465
Namespace: File (6)
Revisie: 24500 | Datum: 2020-03-14T10:51:50Z
Bron: https://wiki.osdev.org/File:Bsod_xp.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ABsod_xp.png&oldid=24500

````text
Windows XP BSOD
````

<a id="w004466"></a>

## W004466 — File:Bsod mac.jpg

Bron-ID: W004466
Namespace: File (6)
Revisie: 24501 | Datum: 2020-03-14T10:53:31Z
Bron: https://wiki.osdev.org/File:Bsod_mac.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ABsod_mac.jpg&oldid=24501

````text
MacOSX BSOD
````

<a id="w004467"></a>

## W004467 — File:Bsod bsd.jpg

Bron-ID: W004467
Namespace: File (6)
Revisie: 24502 | Datum: 2020-03-14T10:55:42Z
Bron: https://wiki.osdev.org/File:Bsod_bsd.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ABsod_bsd.jpg&oldid=24502

````text
FreeBSD BSOD
````

<a id="w004468"></a>

## W004468 — File:Bsod linux.jpg

Bron-ID: W004468
Namespace: File (6)
Revisie: 24503 | Datum: 2020-03-14T10:57:59Z
Bron: https://wiki.osdev.org/File:Bsod_linux.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ABsod_linux.jpg&oldid=24503

````text
Linux BSOD
````

<a id="w004469"></a>

## W004469 — File:Bsod osz.png

Bron-ID: W004469
Namespace: File (6)
Revisie: 24504 | Datum: 2020-03-14T11:08:02Z
Bron: https://wiki.osdev.org/File:Bsod_osz.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ABsod_osz.png&oldid=24504

````text
OS/Z BSOD, and example of built-in debugger
````

<a id="w004470"></a>

## W004470 — Kernel Panic

Bron-ID: W004470
Namespace: Main (0)
Revisie: 29939 | Datum: 2026-01-30T17:29:41Z
Bron: https://wiki.osdev.org/Kernel_Panic
Revisie-URL: https://wiki.osdev.org/index.php?title=Kernel_Panic&oldid=29939

````text
A '''kernel panic''' is an unrecoverable system error.

== Overview ==
[[File:bsod_xp.png|200px|thumb|Windows XP]]
[[File:bsod_mac.jpg|200px|thumb|Mac OS X]]
[[File:bsod_bsd.jpg|200px|thumb|FreeBSD]]
[[File:bsod_linux.jpg|200px|thumb|Linux]]
[[File:bsod_osz.png|200px|thumb|Hobby OS example]]

=== History ===

Windows used to be very, very unstable (almost every pre-NT kernel versions, and early NT versions). When they crashed, they displayed an error message with white text and blue background. This happened so often, that it made to the pop culture, dubbed "Blue Screen of Death", hence the acronym "BSOD".

Despite the name's origin, BSOD is not unique to Windows, all systems have this feature, but they are usually black instead of blue.

=== Purpose ===

The main purpose of a kernel panic screen is to inform the user that an unrecoverable error happened, and they have to reset the computer to continue. Quite often, the crash screen contains low-level information for developers to help debugging.

Such low-level information often includes a textual error message, stack trace and register dumps.

There are at least a few OSes which go a step further, and instead of displaying the debug information provide a built-in debugger which can be used interactively if a system crash happens. For example, Amiga could be debugged remotely using serial connection, and some modern systems start GDB server. In general, systems that have some sort of debugging capability tend to display less information on the crash screen.

For big money corp OSes (running on rack servers and mainframes and alike) it is very common not to display any information, except for a code. The maintainers are supposed to pay insanely huge amounts of money for courses where they get a printed documentation which lists those codes and how to work around each. Luckily, those "big" OSes are well written and very rarely crash.

Atari ST is an example of a minimalistic screen: it displayed 2 to 9 bombs depending on the error. It was a very limited error code system.

=== Funky side ===

Because a crash screen by its nature is a sad to see, developers started to put little things on screen to cheer the users up. One of the earliest examples is Amiga's "Guru Meditation", suggesting that the user will need a computer guru to solve the problem. Later they renamed it to "Grim Reaper". Undeniably funnier than a simple "System Crashed".

The reason why Microsoft picked the blue background is that according to shrinks, blue makes people calm.
[[File:Amiga Guru Meditation.gif|alt=Shows an error code for the Amiga Guru Meditation saying "Software Failure. Press left mouse button to continue"|thumb|196x196px|Guru Meditation]]
For OSes with the console metaphor, funny [[ASCII]] arts are also very popular, specially among hobby OS developers. This is not uncommon to mainstream either, although most just prints developer debug, some BSDs draw ASCII arts too.

== Implementation ==

To implement a kernel panic in your kernel, many write a special function. This function can be dependency-free (if wanted to work no matter what), and might not rely on other parts of your kernel. It is often self-contained, so that it can display the screen regardless which parts of the OS failed. This function is commonly called "kpanic" or "panic", but it can have many names.

On legacy x86 computers, implementing kernel panics is simple, as the kernel can write characters into the video memory at 0xB8000 directly. For a modern PC implementing kernel panics is a little bit more difficult, as the frame buffer has no fixed address, and the kernel has to include a font renderer and its own font too.

For an example, an implementation could load a self-contained <tt>kpanic()</tt> routine to a fixed address below the 1Meg mark. This module would have a predefined header at the beginning, holding a pointer to the initialization routine, as well as information about the current screen mode.  If and when the kernel panics, all it has to do is retrieve this header from this fixed location, grab the initialization address, and jump to it.

A few notes to consider:
 - make sure that this memory allocated for this module does not rely upon any part of the kernel.
 - make sure it is not part of the paging process.  i.e: it is identity mapped as physical memory outside the paging system.
 - save such information as the current video mode's frame buffer, pixel depth, bytes per scan line, etc.
 - with modern hardware, you will have to render font characters to the screen, since Legacy VGA hardware is no longer present.
 - remember to update this video information if you ever change the screen mode.
 - load this module as soon as possible, allowing the kernel to panic during boot if needed.
 - A design like mentioned above may not be needed if willing to assume that the framebuffer and some devices are stable (not always a great idea for things like USB, but otherwise fine for simpler devices)

=== Explicit Invocation ===

You can call this <tt>kpanic</tt> from your kernel when a specific error condition is met. For example, during boot, if you figure there's not enough RAM, or the CPU lacks a feature that your kernel depends on, you failed to load an important system file, etc.

=== Implicit Invocation ===

Then, it is also very common to start with a single exception handler that does nothing else than calling <tt>kpanic</tt>. You set up this in the IDT for all exceptions, and as your kernel development progresses, you replace the handlers one-by-one with their final implementations. This way, exceptions that are not handled properly by your kernel will always trigger a BSOD.

== Examples ==

=== Windows NT ===
On Windows NT, kernel panics are technically called "bug checks". Prior to Windows 8 the bug check screen displayed context info, currently it displays only a bug check code and some graphics. The bug check routine is provided by the kernel, and it uses many HAL and boot video driver functions, to reset the current display device, emulate the bios to set up a VGA mode, other services are provided by the kernel and the debugger modules, to output to the debugger, to freeze the remaining processors and mask off and disable any interrupts on the current processor, so the system would stop running.

=== Mac OS X ===
Note the complete lack of any technical information, not even an error code shown. It is worth noting that Mac OS X also has a console-only kernel panic, which looks like BSD's. (However, Mac OS X saves a memory dump to disk and starts a GDB server to allow a remote debugger to be connected on crash, so there's really no need to print any details on screen.)

=== FreeBSD ===
All about minimal required information, note the stack trace. This amount of information aims for the 'Goldilocks zone' of a kernel panic: it provides enough detail to indicate a critical issue and facilitate initial diagnosis, without overwhelming the user with raw data that typically requires specialized knowledge to interpret.

=== Linux ===
As much information as possible, important details often lost in the noise. Includes register dumps and stack trace, and requires a screen pager.
Totally frightening to average users, no clear instructions for non-experts what to do.

=== Hobby OSes ===
An example of a built-in debugger on a crash screen would be OS/Z an operating system developed by one of the OSDev's forum members. For further examples, the forum topic [https://forum.osdev.org/viewtopic.php?f=1&t=18924 When your OS goes crazy] has many examples of funny ASCII arts.

=== Other ===
[https://www.jwz.org/xscreensaver/ Xscreensaver's] BSOD module displays mock-ups of many operating systems' kernel panics, along with a few other situations. The first half of [https://www.youtube.com/watch?v=YIqbMCfR3r0 this video] shows a few of them.

== See Also ==

* [[wikipedia:Crash (computing)|Wikipedia on Crash]]
* [[wikipedia:Guru Meditation|Wikipedia on Guru Meditation]]

[[Category:OS Development]]
````

<a id="w004471"></a>

## W004471 — Bootable Disk

Bron-ID: W004471
Namespace: Main (0)
Revisie: 30564 | Datum: 2026-05-06T21:26:53Z
Bron: https://wiki.osdev.org/Bootable_Disk
Revisie-URL: https://wiki.osdev.org/index.php?title=Bootable_Disk&oldid=30564

````text
This brief tutorial is about how to create your own bootable disk image. For information about partitioning see [[GPT]] and [[FAT]].
{{Rating|1}}

== BIOS Disk==
Bochs can be a powerful tool to debug, especially early in the operating system load sequence. However, BOCHS can only boot with BIOS and is somewhat legacy in what it emulates. In order to create a basic bootable disk you need to calculate the size based on Cylinders, Heads (max 16) & Sectors (max 63). Then, you need to create the disk and install grub and your files on it.

The following uses a disk layout of 80*16*63, which needs to be explicitly configured in BOCHS.
<syntaxhighlight lang="bash">
$ dd if=/dev/zero of=disk.img bs=512 count=80640 # create an empty null-filled disk. You can use a random device to get a facsimile of a real disk if desired
$ LOOPBACK_DEVICE=$(losetup --find --show disk.img ) # load the disk as a loopback device
$ parted $LOOPBACK_DEVICE mklabel msdos # Create a MBR partition label
$ parted $LOOPBACK_DEVICE mkpart primary fat32 1MiB 100% # Create a partition starting at 1 MiB that spans the rest of the disk with label fat32
$ losetup -d $LOOPBACK_DEVICE # Delete the loopback device
$ LOOPBACK_DEVICE=$(losetup --find --show --partscan disk.img ) # Reload, the partitions will now be loaded and be accessible as well
$ mkfs.vfat -F 32 ${LOOPBACK_DEVICE}p1 # Create a FAT32 partition
$ DISK_NAME="/mnt/disk" 
$ mkdir -p $DISK_NAME # Create a mount point
$ mount -t vfat ${LOOPBACK_DEVICE}p1 $DISK_NAME # Mount the image
$ cd $DISK_NAME
$ mkdir boot
$ grub-install --no-nvram --directory /usr/lib/grub/i386-pc/ --modules="normal part_msdos multiboot2 fat" $LOOPBACK_DEVICE --no-floppy --target=i386-multiboot # Install grub
</syntaxhighlight>

You can now copy your kernel image to /mnt/disk/boot.

<syntaxhighlight lang="bash">
$ umount $DISK_NAME # unmount the disk image
$ losetup -d $LOOPBACK_DEVICE # remove the loopback device entries (similar to ejecting a USB)
</syntaxhighlight>

You can now use a BIOS-booting virtual machine, such as Bochs, to boot from this disk. When the grub shell comes up, use the following to boot (assuming you used multiboot2)
<syntaxhighlight lang="bash">
multiboot2 (hd0,msdos1)/boot/<yourkernelimage>
boot
</syntaxhighlight>

If you are using an EFI device, you may need to get the grub bios files. On debian, these are in the ''grub-pc-bin'' package. '''Do not install this package''', instead:
<syntaxhighlight lang="bash">
mkdir grub-bios
sudo apt download grub-pc-bin # Download the package to the current working directory
dpkg-deb -R grub-pc-bin_<version>.deb grub-bios/ # Unpack the deb file to grub-bios
</syntaxhighlight>
You will have to change the --directory in your grub-install to point to:<syntaxhighlight lang="bash">
$(pwd)/grub-bios/usr/lib/grub/i386-pc
</syntaxhighlight><syntaxhighlight lang="bash">
 
</syntaxhighlight>


==Requirements==
you'll need the following:
*fdisk
* mkfs.vfat
*[[bootloader]]
*your [[Kernels|kernel]] and maybe some other files


==Required Steps==
===Creating an Empty Image===
First of all, let's create an empty disk image file, let's say 128 megabytes in size.
<syntaxhighlight lang="bash">
$ dd if=/dev/zero of=diskimage.dd bs=1048576 count=128
</syntaxhighlight>
=== Creating the Partitioning Table===
As the GPT is more complex thing than MBR partitioning tables, we'll use fdisk, which has an interactive interface (for macOS you will need to use the [https://www.gnu.org/software/fdisk/ gnu fdisk]). Inputs (that you're supposed to type) are after the ":" colons.
<syntaxhighlight lang="bash">
$ fdisk diskimage.dd

Welcome to fdisk (util-linux 2.30.2).
Changes will remain in memory only, until you decide to write them.
Be careful before using the write command.

Device does not contain a recognized partition table.
Created a new DOS disklabel with disk identifier 0xfa00b86e.

Command (m for help): g
Created a new GPT disklabel (GUID: E6B4945A-8308-448B-9ACA-0E656854CF66).

Command (m for help): n p
Partition number (1-128, default 1): 1
First sector (2048-262110, default 2048): 2048
Last sector, +sectors or +size{K,M,G,T,P} (2048-262110, default 262110): +8M

Created a new partition 1 of type 'Linux filesystem' and of size 8 MiB.

Command (m for help): t 1
Selected partition 1
Partition type (type L to list all types): 1
Changed type of partition 'Linux filesystem' to 'EFI System'.

Command (m for help): w
The partition table has been altered.Syncing disks.
$
</syntaxhighlight>
The commands used were:
*g - create a new GPT partitioning table
* n p - new primary partition
*1 2048 +8M - first partition, starting sector, partition size
*t 1 - change the type of the first partition
*1 - type 1 is [[EFI System Partition]] (or ESP in short)
*w - write out changes

===Creating a File System on the First Partition ===
Now this is not at simple as it seems, because we have to create the file system somewhere in the middle of a file. For that, we'll create a [[Loopback_Device|loop device]].
<syntaxhighlight lang="bash">
$ losetup -o $((2048*512)) --sizelimit $((8*1024*1024)) -f diskimage.dd
</syntaxhighlight>
Here the arguments are:
*-o $((2048*512)) - tells losetup that our partition is starting at the 2048th sector (one sector is 512 bytes)
*--sizelimit $((8*1024*1024)) - specifies the limit in 8 megabytes
*-f - tells to find a suitable loopback device
* and the last parameter is our disk image's filename.
To see which device was used for your image, you can do
<syntaxhighlight lang="bash">
$ losetup -a
</syntaxhighlight>
which will list all loopback devices.

Now that we have a device which points inside the image file exactly to the partition, we can create a FAT filesystem on it:
<syntaxhighlight lang="bash">
$ mkfs.vfat -F 16 -n "EFI System" /dev/loop0
</syntaxhighlight>
Here
*-F 16 - tells to create a FAT16
*-n "EFI System" - sets the label for the partition (don't change, some firmware checks for this)
*/dev/loop0 - is the loopback device (change to the one you saw in losetup -a output)

===Adding Files to the Partition===
The next step is to mount our loopback device, so that we can copy files to the partition.
<syntaxhighlight lang="bash">
$ mkdir somedir
$ mount /dev/loop0 somedir
</syntaxhighlight>
For simplicity, let's call our kernel MSDOS.SYS and out configuration file CONFIG.SYS:
<syntaxhighlight lang="bash">
$ cp MSDOS.SYS CONFIG.SYS somedir/
</syntaxhighlight>
Some boot loaders are so called two-stage loaders. This means they have two parts: a boot sector and a 2nd stage file. You must copy that 2nd stage to the partition:
<syntaxhighlight lang="bash">
$ cp 2NDSTAGE.BIN somedir/
</syntaxhighlight>
[[UEFI]] will need a special [[PE]] executable named '''EFI/BOOT/BOOT(arch).EFI''', which you can create with [[GNU-EFI]]:
<syntaxhighlight lang="bash">
$ mkdir somedir/EFI somedir/EFI/BOOT
$ cp BOOTX64.EFI somedir/EFI/BOOT
</syntaxhighlight>

Once you're finished with the copy, it is important to dismount the partition:
<syntaxhighlight lang="bash">
$ umount somedir
$ rmdir somedir
</syntaxhighlight>

===Adding the BIOS Boot Loader===
If you're planning to create a hybrid image with the PMBR (that you can also boot on legacy BIOS machines), then you need to add the legacy BIOS boot loader code:
<syntaxhighlight lang="bash">
$ dd if=boot.bin of=diskimage.dd conv=notrunc bs=446 count=1
$ dd if=boot.bin of=diskimage.dd conv=notrunc bs=1 count=2 skip=510 seek=510
</syntaxhighlight>
The first command copies the code, and the second one the two magic bytes at the end of the sector. This way the bytes 446 - 510 (where the partitioning table resides) won't be overwritten. It is very important preserve the ESP entry required by UEFI.

Here the arguments mean:
*if=boot.bin - the name of the boot loader, should be 512 bytes
*of=diskimage.dd - the disk image
*conv=notrunc - tells dd to update an existing image file instead of creating a new one
*bs=446 - block size (sector size is 512 bytes, but we want to write less)
*skip=510 - skip that many blocks (in second line bs is 1, so skip 510 bytes)
*count=1 - tells dd to update one sector only

==See Also==
*[[Tutorials]]
*[[Bootloader]]
*[[Kernels]]
*[[EFI System Partition]]

[[Category:Booting]]
[[Category:Tutorials]]
````

<a id="w004472"></a>

## W004472 — Internal Kernel Debugger

Bron-ID: W004472
Namespace: Main (0)
Revisie: 29625 | Datum: 2025-08-22T00:03:46Z
Bron: https://wiki.osdev.org/Internal_Kernel_Debugger
Revisie-URL: https://wiki.osdev.org/index.php?title=Internal_Kernel_Debugger&oldid=29625

````text
{{In Progress}}

An Internal Kernel Debugger (which will be referred to as the IKD from now on) is an inbuilt equivalent to GDB or any other debugger but is inbuilt into your kernel, or some other part of your OS.

== Preexisting Tools ==

=== Minidbg ===

[https://gitlab.com/bztsrc/minidbg Minidbg] is a “minimal, easy to integrate, multiplatform ANSI C debugger with serial interface” which works with AARCH64 and X86_64 devices. Minidbg can be easily implemented into a kernel with very little time or effort to actually implement it. It does not have any dependencies other than the assembler you use, and the architecture you target. It it also licensed under a permissive license.

== Making Your Own IKD ==

If you wish to develop your own IKD, you will first need to have some form of input; this could be via a PS/2 keyboard or a serial port. The method does not matter as long as it works. The next thing you need to know is when to trigger the IKD. Possible methods to trigger the IKD could be CPU exceptions, serial interrupts, or anything else.

=== What Does an IKD Do? ===

The IKD can dump memory, dump registers, and read most critical parts of the system such as the GDT, IDT, kernel heap allocation info (whats allocated, and such, its VMM stuff, and others), the PMM, VMM, and so on, it should be able to dump anything and everything, including ACPI tables.
````

<a id="w004474"></a>

## W004474 — Talk:NexArchive

Bron-ID: W004474
Namespace: Talk (1)
Revisie: 24567 | Datum: 2020-04-06T18:25:39Z
Bron: https://wiki.osdev.org/Talk:NexArchive
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ANexArchive&oldid=24567

````text
How i'm supposed to find a file?, What is ''Unique Directory index''?, What exactly is ''Array of next directory entries''?, I'm just curious --[[User:Superleaf1995|Superleaf1995]] 13:25, 6 April 2020 (CDT)
````

<a id="w004475"></a>

## W004475 — User:Superleaf1995/PCI in assembly

Bron-ID: W004475
Namespace: User (2)
Revisie: 24568 | Datum: 2020-04-06T18:52:08Z
Bron: https://wiki.osdev.org/User:Superleaf1995/PCI_in_assembly
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASuperleaf1995%2FPCI_in_assembly&oldid=24568

````text
<source lang="asm">
; AL = BUS
; BL = SLOT
; CL = FUNC
; DL = OFFSET
; Returns AX as TMP
pciConfigReadWord:
	mov byte [.off], dl
	xor edx, edx
	
	mov byte [.tmp], al
	xor eax, eax ; Only leave AL (assume we are using 386+)
	mov al, byte [.tmp]
	mov dword [.lbus], eax
	
	mov byte [.tmp], bl
	xor ebx, ebx
	mov bl, byte [.tmp]
	mov dword [.lslot], ebx
	
	mov byte [.tmp], cl
	xor ecx, ecx
	mov cl, byte [.tmp]
	mov dword [.lfunc], ecx
	
	; Everything set, now calculate address
	xor eax, eax
	xor ebx, ebx
	xor ecx, ecx
	
	mov ecx, [.lfunc]
	mov ebx, [.lslot]
	mov eax, [.lbus]
	shl eax, 16
	shl ebx, 11
	shl ecx, 8
	mov dl, byte [.off]
	and dl, 0FCh
	
	or eax, ebx
	or eax, ecx
	or eax, edx
	mov ebx, 080000000h
	or eax, ebx
	mov dword [.addr], eax
	
	mov eax, dword [.addr]
	out 0CF8h, eax
	
	in eax, 0CFCh
	mov cl, [.off]
	and cl, 2
	shr eax, cl
	
	mov ebx, 8
	mul ebx
	
	and ax, 0FFFFh
	
	ret

.off	db 0
.addr	dd 0
.lbus	dd 0
.lslot	dd 0
.lfunc	dd 0
.tmp	dw 0
</source>

'''WARNING''': Code untested, Ctr+C and Ctr+V at your own risk!
````

<a id="w004476"></a>

## W004476 — User:Xenos/UEFI Bare Bones

Bron-ID: W004476
Namespace: User (2)
Revisie: 26635 | Datum: 2021-10-31T01:52:27Z
Bron: https://wiki.osdev.org/User:Xenos/UEFI_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AXenos%2FUEFI_Bare_Bones&oldid=26635

````text
== Introduction ==

This tutorial shows how to build a simple "Hello world" bare bones EFI application. It is largely based on the [[UEFI App Bare Bones]] article, but instead of using (parts of) gnu-efi and a Windows targeted toolchain, it uses a bare metal targeted [[GCC Cross-Compiler|GCC cross compiler]] and no foreign sources.

== Prerequisites ==

=== Bare metal GCC cross compiler ===

There are two approaches using the GNU (GCC / binutils) toolchain which are most common for building EFI applications:

* gnu-efi uses native tools, provided that the given version of objcopy can create EFI applications.
* Various tutorials use Windows (MinGW, MSYS) targeted toolchains.

This tutorial uses a different approach, which is closer to other, non-UEFI bare bones tutorials on OS development, by using a bare metal ELF targeted [[GCC Cross-Compiler|GCC cross compiler]]. However, a few minor modifications are necessary in order to create EFI applications with the created toolchain.

==== Target system selection ====

Before starting, one needs to decide on a target architecture, which will determine the further compiler options. For this tutorial, a bare metal ELF targeted cross compiler will be used; however, for binutils to be able to create EFI applications in the PE format, an additional PE target is needed. Depending on the chosen target architecture, one may use one of the following:

===== i386 =====

<source lang="bash">export TARGET=i686-elf
export TARGETS=$TARGET,i686-pe</source>

===== x86_64 =====

<source lang="bash">export TARGET=x86_64-elf
export TARGETS=$TARGET,x86_64-pe</source>

==== Build environment ====

<source lang="bash">export PREFIX=/opt/cross/$TARGET
export PATH="$PREFIX/bin:$PATH"</source>

==== Building binutils ====

<source lang="bash">mkdir build-binutils
cd build-binutils
../binutils-x.y.z/configure --target=$TARGET --enable-targets=$TARGETS --prefix="$PREFIX" --with-sysroot --disable-nls
make
make install</source>

==== Building GCC ====

<source lang="bash">mkdir build-gcc
cd build-gcc
../gcc-x.y.z/configure --target=$TARGET --prefix="$PREFIX" --disable-nls --enable-languages=c,c++ --without-headers
make all-gcc
make all-target-libgcc
make install-gcc
make install-target-libgcc</source>

=== MTools ===

The disk image is created using [[MTools]].

== Source files ==

=== main.c ===

This tutorial creates a simple "Hello world" EFI application, which does the following:

# Print "Hello world" to the standard EFI console.
# Flush the input buffer, to remove any possibly pending key strokes.
# Wait for a key to be pressed.
# Exit and return to the environment.

This is done in the following C code:

<source lang="c">#include "efi.h"

EFI_STATUS efi_main(EFI_HANDLE ImageHandle, EFI_SYSTEM_TABLE* SystemTable)
{
	(void)ImageHandle;
	EFI_STATUS Status;
	EFI_INPUT_KEY Key;

	/* Print message. */
	Status = SystemTable->ConOut->OutputString(SystemTable->ConOut, L"Hello World\n\r" MESSAGE);
	if(EFI_ERROR(Status))
		return Status;

	/* Empty the console input buffer to flush out any keystrokes entered before this point. */
	Status = SystemTable->ConIn->Reset(SystemTable->ConIn, false);
	if(EFI_ERROR(Status))
		return Status;

	/* Wait for keypress. */
	while((Status = SystemTable->ConIn->ReadKeyStroke(SystemTable->ConIn, &Key)) == EFI_NOT_READY) ;

	return Status;
}</source>

=== efi.h ===

Most EFI applications make use of a large number of data types, structs, constants, function prototypes etc. which are declared in a common header file or set of header files. For this simple bare bones application, only a subset of these is used, and so a minimal set of declarations may be used. The following file declares only those parts which are necessary for simple console output and input.

<source lang="c">#include <stdint.h>
#include <stdbool.h>
#include <wchar.h>

#ifndef EFI_H
#define EFI_H

typedef void* EFI_PVOID;
typedef void* EFI_HANDLE;

#ifdef __i386__
#define MESSAGE L"i386\r\n"
typedef uint32_t UINTN;
#endif
#ifdef __amd64__
#define MESSAGE L"x86_64\r\n"
typedef uint64_t UINTN;
#endif

typedef UINTN EFI_STATUS;

#define EFIERR(a) (a | ~(((EFI_STATUS)-1) >> 1))
#define EFI_ERROR(a) (a & ~(((EFI_STATUS)-1) >> 1))

#define EFI_NOT_READY EFIERR(6)

typedef struct {
	uint64_t Signature;
	uint32_t Revision;
	uint32_t HeaderSize;
	uint32_t CRC32;
	uint32_t Reserved;
} EFI_TABLE_HEADER;

typedef struct {
	uint32_t MaxMode;
	uint32_t Mode;
	uint32_t Attribute;
	uint32_t CursorColumn;
	uint32_t CursorRow;
	uint8_t  CursorVisible;
} SIMPLE_TEXT_OUTPUT_MODE;

typedef EFI_STATUS (*EFI_TEXT_CLEAR_SCREEN)(void *This);
typedef EFI_STATUS (*EFI_TEXT_ENABLE_CURSOR)(void *This, uint8_t Visible);
typedef EFI_STATUS (*EFI_TEXT_SET_ATTRIBUTE)(void *This, UINTN Attribute);
typedef EFI_STATUS (*EFI_TEXT_STRING)(void *This, const wchar_t *String);

typedef EFI_STATUS (*EFI_TEXT_QUERY_MODE)(
	void  *This,
	UINTN ModeNumber,
	UINTN *Columns,
	UINTN *Rows);

typedef EFI_STATUS (*EFI_TEXT_SET_CURSOR_POSITION)(
	void  *This,
	UINTN Column,
	UINTN Row);

typedef struct {
	EFI_PVOID                    Reset;
	EFI_TEXT_STRING              OutputString;
	EFI_PVOID                    TestString;
	EFI_TEXT_QUERY_MODE          QueryMode;
	EFI_PVOID                    SetMode;
	EFI_TEXT_SET_ATTRIBUTE       SetAttribute;
	EFI_TEXT_CLEAR_SCREEN        ClearScreen;
	EFI_TEXT_SET_CURSOR_POSITION SetCursorPosition;
	EFI_TEXT_ENABLE_CURSOR       EnableCursor;
	SIMPLE_TEXT_OUTPUT_MODE      *Mode;
} EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL;

typedef struct {
	wchar_t ScanCode;
	wchar_t UnicodeChar;
} EFI_INPUT_KEY;

typedef EFI_STATUS (*EFI_INPUT_RESET)(void *This, bool ExtendedVerification);
typedef EFI_STATUS (*EFI_INPUT_READ_KEY)(void *This, EFI_INPUT_KEY *Key);

typedef struct {
	EFI_INPUT_RESET    Reset;
	EFI_INPUT_READ_KEY ReadKeyStroke;
	EFI_PVOID          WaitForKey;
} EFI_SIMPLE_TEXT_INPUT_PROTOCOL;

typedef struct {
	EFI_TABLE_HEADER                Hdr;
	EFI_PVOID                       FirmwareVendor;
	uint32_t                        FirmwareRevision;
	EFI_PVOID                       ConsoleInHandle;
	EFI_SIMPLE_TEXT_INPUT_PROTOCOL  *ConIn;
	EFI_PVOID                       ConsoleOutHandle;
	EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL *ConOut;
	EFI_PVOID                       StandardErrorHandle;
	EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL *StdErr;
	void                            *RuntimeServices;
	void                            *BootServices;
	UINTN                           NumberOfTableEntries;
	void                            *ConfigurationTable;
} EFI_SYSTEM_TABLE;

#endif</source>

=== Linker script ===

To link the final binary and keep the sections which are needed, a linker script is used. The following linker scripts, for different target architectures, do the following:

# Select the output file format (ELF and its specific flavor).
# Select the output architecture.
# Set the entry point to the function efi_main.
# Select the necessary sections and align them on page boundaries.
# Create a fake relocation entry, since EFI loaders check for the presence of a relocation table.
# Discard unnecessary sections.

For linking, one must make sure that the correct target architecture is selected.

==== i386 ====

<source lang="c">OUTPUT_FORMAT("elf32-i386")
OUTPUT_ARCH(i386)
ENTRY(efi_main)
SECTIONS
{
	. = 4096;
	ImageBase = .;
	.hash : { *(.hash) }
	.gnu.hash : { *(.gnu.hash) }
	. = ALIGN(4096);
	.text :
	{
		_text = .;
		*(.text)
		*(.text.*)
		*(.gnu.linkonce.t.*)
		. = ALIGN(16);
	}
	_etext = .;
	_text_size = . - _text;
	. = ALIGN(4096);
	.rdata :
	{
		_data = .;
		*(.got.plt)
		*(.got)
		*(.rodata*)
		*(.srodata)
		*(.gnu.linkonce.r.*)
	}
	. = ALIGN(4096);
	.data :
	{
		*(.data*)
		*(.sdata*)
		*(.gnu.linkonce.d.*)
	}
	. = ALIGN(4096);
	.bss :
	{
		*(.sbss)
		*(.scommon)
		*(.dynbss)
		*(.bss)
		*(.gnu.linkonce.b.*)
		*(COMMON)
	}
	. = ALIGN(4096);
	.dynamic  : { *(.dynamic) }
	. = ALIGN(4096);
	.rel :
	{
		*(.rel.data)
		*(.rel.data.*)
		*(.rel.got)
		*(.rel.stab)
	}
	_edata = .;
	_data_size = . - _etext;
	. = ALIGN(4096);
	.reloc :
	{
		LONG(_data);
		LONG(10);
		SHORT(0);
		*(.reloc)
	}
	. = ALIGN(4096);
	.dynsym   : { *(.dynsym) }
	. = ALIGN(4096);
	.dynstr   : { *(.dynstr) }
	. = ALIGN(4096);
	/DISCARD/ :
	{
		*(.rel.reloc)
		*(.eh_frame)
		*(.note*)
		*(.comment*)
	}
}</source>

==== x86_64 ====

<source lang="c">OUTPUT_FORMAT("elf64-x86-64")
OUTPUT_ARCH(i386:x86-64)
ENTRY(efi_main)
SECTIONS
{
	. = 4096;
	ImageBase = .;
	.hash : { *(.hash) }
	.gnu.hash : { *(.gnu.hash) }
	. = ALIGN(4096);
	.text :
	{
		_text = .;
		*(.text)
		*(.text.*)
		*(.gnu.linkonce.t.*)
		. = ALIGN(16);
	}
	_etext = .;
	_text_size = . - _text;
	. = ALIGN(4096);
	.rdata :
	{
		_data = .;
		*(.got.plt)
		*(.got)
		*(.rodata*)
		*(.srodata)
		*(.gnu.linkonce.r.*)
	}
	. = ALIGN(4096);
	.data :
	{
		*(.data*)
		*(.sdata*)
		*(.gnu.linkonce.d.*)
	}
	. = ALIGN(4096);
	.bss :
	{
		*(.sbss)
		*(.scommon)
		*(.dynbss)
		*(.bss)
		*(.gnu.linkonce.b.*)
		*(COMMON)
	}
	. = ALIGN(4096);
	.dynamic  : { *(.dynamic) }
	. = ALIGN(4096);
	.rel :
	{
		*(.rel.data)
		*(.rel.data.*)
		*(.rel.got)
		*(.rel.stab)
	}
	_edata = .;
	_data_size = . - _etext;
	. = ALIGN(4096);
	.reloc :
	{
		LONG(_data);
		LONG(10);
		SHORT(0);
		*(.reloc)
	}
	. = ALIGN(4096);
	.dynsym   : { *(.dynsym) }
	. = ALIGN(4096);
	.dynstr   : { *(.dynstr) }
	. = ALIGN(4096);
	/DISCARD/ :
	{
		*(.rel.reloc)
		*(.eh_frame)
		*(.note*)
		*(.comment*)
	}
}</source>

== Building ==

The build process consists of three steps. First, the main input (C) file is compiled using GCC. This produces an ELF object file. Note that for compiling, a freestanding environment must be used, since there is no C library used in this tutorial. The following flags are applied:

; -ffreestanding : enables the freestanding environment, where no C library is used.
; -fpic : produces position-independent code, which can be loaded anywhere in memory.
; -fno-stack-protector : disables a [[Stack Smashing Protector|stack protector]].
; -fshort-wchar : sets wide character strings to use 16 bit, since EFI uses 16 bit unicode.
; -Wall -Wextra -Wpedantic : enables a number of helpful warnings.
; -O3 : enables optimization.

In the next step, the object file is linked using the appropriate linker script, which selects the emitted sections and their memory layout. This will create an ELF shared library file. The following flags are used:

; -nostdlib : disables standard library search paths.
; -shared : creates a shared library file.
; -Wl,-T,linker.lds : makes use of the supplied linker script.
; -Wl,-Bsymbolic : binds references to global symbols to their definition within the shared library.
; -Wl,-znocombreloc : do not combine multiple relocation sections.

Finally, objcopy is used to build an EFI application out of the ELF library created in the previous step. The output file name is chosen so that the file is booted automatically in the EFI boot process, and can be found in the following table, depending on the target architecture.

{| class="wikitable"
|-
! architecture
! boot file name
! PE executable machine type
|-
| i386 / IA32
| BOOTIA32.EFI
| 0x14c
|-
| x86_64 / AMD64
| BOOTX64.EFI
| 0x8664
|-
| IA64 / Itanium
| BOOTIA64.EFI
| 0x200
|-
| ARM / AArch32
| BOOTARM.EFI
| 0x1c2
|-
| AArch64
| BOOTAA64.EFI
| 0xaa64
|}

In principle, it is also possible to omit the last step and to link directly to an EFI application, without using the intermediate ELF format. However, for this to work also the libgcc and object files must be present in a suitable ([[COFF]]) format. The GCC compiler used here produces ELF files, and so the intermediate step is used.

Depending on the target architecture, different further adaptations may be necessary.

=== i386 ===

The following additional compiler options are used on the i686 target:

; -mgeneral-regs-only : uses only general purpose registers (since SSE etc. registers need operating system support).

The following commands are used:

<source lang="bash">i686-elf-gcc -ffreestanding -fpic -fno-stack-protector -fshort-wchar -mgeneral-regs-only -Wall -Wextra -Wpedantic -O3 -o main.o -c main.c
i686-elf-gcc -nostdlib -shared -Wl,-T,i386.lds -Wl,-Bsymbolic -Wl,-znocombreloc -o kernel_ia32.elf main.o -lgcc
i686-elf-objcopy -I elf32-i386 -O efi-app-ia32 kernel_ia32.elf BOOTIA32.EFI</source>

=== x86_64 ===

The following additional compiler options are used on the x86_64 target:

; -mno-red-zone : disables the [[Libgcc without red zone|red zone]].
; -mgeneral-regs-only : uses only general purpose registers (since SSE etc. registers need operating system support).
; -mabi=ms : uses the Microsoft ABI for all function calls (which is needed for all calls to and from the EFI API).

The following commands are used:

<source lang="bash">x86_64-elf-gcc -ffreestanding -fpic -fno-stack-protector -fshort-wchar -mno-red-zone -mgeneral-regs-only -mabi=ms -Wall -Wextra -Wpedantic -O3 -o main64.o -c main.c
x86_64-elf-gcc -nostdlib -shared -Wl,-T,x86_64.lds -Wl,-Bsymbolic -Wl,-znocombreloc -o kernel_x64.elf main64.o -lgcc
x86_64-elf-objcopy -I elf64-x86-64 -O efi-app-x86_64 kernel_x64.elf BOOTX64.EFI</source>

== Creating disk images ==

There are different possibilities how to boot an EFI application. One possibility is to create a disk image and to copy the file into the directory /EFI/BOOT. The following commands create a floppy image, format it and copy the file(s) into the correct directory using [[MTools]].

<source lang="bash">dd if=/dev/zero of=fat.img bs=1k count=1440
mformat -i fat.img -f 1440 ::
mmd -i fat.img ::/EFI
mmd -i fat.img ::/EFI/BOOT
mcopy -i fat.img BOOT*.EFI ::/EFI/BOOT</source>

== Running ==

=== QEMU with OVMF ===

One possibility to run EFI applications is to use [[QEMU]] with [https://github.com/tianocore/tianocore.github.io/wiki/OVMF OVMF]. Compiled OVMF images can be downloaded for different target architectures, and one must make sure to choose the correct files. The following examples will directly boot the bare bones kernel, wait for a key press and drop into the BIOS.

==== i386 ====

<source lang="bash">qemu-system-i386 -machine q35 -m 256 -smp 2 -net none \
    -global driver=cfi.pflash01,property=secure,value=on \
    -drive if=pflash,format=raw,unit=0,file=OVMF_CODE-pure-efi.fd,readonly=on \
    -drive if=pflash,format=raw,unit=1,file=OVMF_VARS-pure-efi.fd \
    -drive if=ide,format=raw,file=fat.img</source>

==== x86_64 ====

<source lang="bash">qemu-system-x86_64 -machine q35 -m 256 -smp 2 -net none \
    -global driver=cfi.pflash01,property=secure,value=on \
    -drive if=pflash,format=raw,unit=0,file=OVMF_CODE-pure-efi.fd,readonly=on \
    -drive if=pflash,format=raw,unit=1,file=OVMF_VARS-pure-efi.fd \
    -drive if=ide,format=raw,file=fat.img</source>

=== VirtualBox ===

[[VirtualBox]] can boot in UEFI mode and run the "Hello world" application created in this tutorial in both 32 and 64 bit mode. A virtual machine with the appropriate settings can be created with the following commands.

==== i386 ====

Create the virtual machine:

<source lang="bash">VBoxManage createvm --name UEFI32 --ostype "Other" --register
VBoxManage modifyvm UEFI32 --ioapic on
VBoxManage modifyvm UEFI32 --boot1 floppy
VBoxManage modifyvm UEFI32 --memory 256 --vram 16
VBoxManage modifyvm UEFI32 --firmware efi32
VBoxManage storagectl UEFI32 --name "Floppy" --add floppy
VBoxManage storageattach UEFI32 --storagectl "Floppy" --port 0 --device 0 --type fdd --medium "fat.img"</source>

Run:

<source lang="bash">VBoxManage startvm UEFI32</source>
 
==== x86_64 ====

Create the virtual machine:

<source lang="bash">VBoxManage createvm --name UEFI64 --ostype "Other_64" --register
VBoxManage modifyvm UEFI64 --ioapic on
VBoxManage modifyvm UEFI64 --boot1 floppy
VBoxManage modifyvm UEFI64 --memory 256 --vram 16
VBoxManage modifyvm UEFI64 --firmware efi64
VBoxManage storagectl UEFI64 --name "Floppy" --add floppy
VBoxManage storageattach UEFI64 --storagectl "Floppy" --port 0 --device 0 --type fdd --medium "fat.img"</source>

Run:

<source lang="bash">VBoxManage startvm UEFI64</source>

[[Category:Firmware]]
[[Category:UEFI]]
````

<a id="w004477"></a>

## W004477 — Ada

Bron-ID: W004477
Namespace: Main (0)
Revisie: 28262 | Datum: 2023-08-04T01:53:38Z
Bron: https://wiki.osdev.org/Ada
Revisie-URL: https://wiki.osdev.org/index.php?title=Ada&oldid=28262

````text
Ada is a strongly typed programming language that enjoys widespread use within the embedded systems and safety-critical software industry. It was originally developed under contract to the United States Department of Defense, as part of a project to develop a single unified language meeting the safety and reliability requirements of the department's embedded systems projects. 

The language was first standardized in 1983, and has since undergone multiple revisions. The latest version is Ada 2012. The language itself is defined by the ISO standard [https://www.iso.org/standard/61507.html ISO/IEC 8652], and the Military standard MIL-STD-1815A. In 1991, the US Department of Defense mandated the use of Ada for all software, though exceptions to this rule were often granted. This mandate was effectively removed in 1997, as the DoD began to embrace 'commercial off the shelf' (COTS) technology. Similar requirements existed in other NATO countries for command and control systems, and Ada was the mandated or preferred language for defense-related applications in countries such as Sweden, Germany, and Canada.

== Features ==

Ada has a number of useful features that are of particular interest for low-level programming and operating system development. 

=== Custom types ===

In addition to being a strongly typed language, Ada allows for the definition of new scalar, enumerated and record types. Custom primitive types can also be constrained to a predefined range of values. The example below demonstrates the definition of a new integer type based upon Ada's native <tt>Natural</tt> type, restricted to a predefined range. The use of the <tt>subtype</tt> directive informs the compiler that other variables of the <tt>Natural</tt> type are compatible with the newly defined subtype.

<source lang="ada">
   VGA_COL_COUNT : constant := 80;
   VGA_ROW_COUNT : constant := 24;

   subtype Col is Natural range 0 .. VGA_COL_COUNT - 1;
   subtype Row is Natural range 0 .. VGA_ROW_COUNT - 1;
</source>

The below example illustrates the creation of incompatible custom integer types. While their base type and range constraints are identical, Ada treats both as separate, incompatible types. An assignment of a variable of one type to the value of another is illegal, and will trigger a compile-time error.
<source lang="ada">
   type Integer_1 is range 1 .. 10;
   type Integer_2 is range 1 .. 10;
   A : Integer_1 := 8;
   B : Integer_2 := A; -- illegal!
</source>

The following example demonstrates the creation of a custom enumerated type. It also demonstrates a subtype of an enumerated type with a constrained range of values.

<source lang="ada">
   type Day_Of_Week is (Monday, Tuesday,
     Wednesday, Thursday, Friday, Saturday, Sunday);

   subtype Work_Day is Day_Of_Week range Monday .. Friday;
</source>

A variable with the type of <tt>Work_Day</tt> is restricted to its constrained range. Any attempt to assign a value outside of this range to a variable of this type will raise a <tt>Constraint_Error</tt> exception at runtime.


=== Representation clauses ===

Ada allows for explicitly defining the in-memory representation of scalar and compound types. The following example demonstrates the definition of a record type (equivalent to structures in C), as well as its associated representation in memory.

<source lang="ada">
   ----------------------------------------------------------------------------
   --  The format of the System Table Descriptor pointer used by the processor
   --  to load descriptor tables like the GDT and IDT.
   ----------------------------------------------------------------------------
   type System_Table_Descriptor is
      record
         Size   : Unsigned_16;
         Offset : System.Address;
      end record
   with Size => 48;
   for System_Table_Descriptor use
      record
         Size   at 0 range 0  .. 15;
         Offset at 0 range 16 .. 47;
      end record;
</source>

The <tt>Size</tt> ''aspect specifier'' instructs the compiler that the <tt>System_Table_Descriptor</tt> type must be 48 bits in size. The ''record representation clause'' instructs the compiler as to the required layout of this record type in memory. This example specifies that the <tt>Size</tt> member should occupy bits 0 to 15, and the <tt>Offset</tt> member should occupy bits 16 to 47. This feature is analogous to C's bit-fields.

The following example demonstrates defining the in-memory representation of an enumerated type.

<source lang="ada">
   ----------------------------------------------------------------------------
   --  The privilege level for a particular descriptor.
   --  These correspond to the 'protection ring' that this descriptor is
   --  accessible from.
   ----------------------------------------------------------------------------
   type Descriptor_Privilege_Level is (
      Ring_0,
      Ring_1,
      Ring_2,
      Ring_3
   )
   with Size => 2;
   for Descriptor_Privilege_Level use (
      Ring_0 => 0,
      Ring_1 => 1,
      Ring_2 => 2,
      Ring_3 => 3
   );
</source>

The <tt>Size</tt> ''aspect specifier'' instructs the compiler that the <tt>Descriptor_Privilege_Level</tt> type must be 2 bits in size. The ''representation clause'' instructs the compiler as to required representation of each possible value of the enumerated type in memory. In this example the value of <tt>Ring_0</tt> will be represented by a value of <tt>0x0</tt> in memory, the value of <tt>Ring_1</tt> will be represented by <tt>0x1</tt>, and so on.

== Example ==
The following example, and accompanying comparison with C, demonstrates the configuration of a hypothetical [[UART]] device by interfacing with an 8-bit memory-mapped configuration register. This example has been adapted from a presentation by AdaCore viewable at: https://www.youtube.com/watch?v=qvmDqbuQe-M

<source lang="ada">
with System.Storage_Elements; use System.Storage_Elements;

-------------------------------------------------------------------------------
--  Main
-------------------------------------------------------------------------------
procedure Main is
   ----------------------------------------------------------------------------
   --  Baud rate type.
   ----------------------------------------------------------------------------
   type Baud_Rate_T is (b_9600, b_14400, b_115200);
   for Baud_Rate_T use (
     b_9600   => 0,
     b_14400  => 1,
     b_115200 => 7
   );

   ----------------------------------------------------------------------------
   --  Parity Select Type
   ----------------------------------------------------------------------------
   type Parity_T is (None, Even, Odd);
   for Parity_T use (
     None => 0,
     Even => 1,
     Odd  => 2
   );

   ----------------------------------------------------------------------------
   --  Control Register for a hypothetical UART device.
   ----------------------------------------------------------------------------
   type UART_Control_Register_T is
      record
         Baud_Rate : Baud_Rate_T;
         Parity    : Parity_T;
         Unused    : Boolean := False;
         ISR       : Boolean;
      end record
   with Volatile_Full_Access,
     Size => 8;
   for UART_Control_Register_T use
      record
         Baud_Rate at 0 range 0 .. 2;
         Parity    at 0 range 3 .. 4;
         Unused    at 0 range 5 .. 6;
         ISR       at 0 range 7 .. 7;
      end record;

   ----------------------------------------------------------------------------
   --  The address of the UART control register.
   ----------------------------------------------------------------------------
   UART_CONTROL_REG_ADDR : constant System.Address
     := To_Address(16#8000_0040#);

   ----------------------------------------------------------------------------
   --  The UART control register itself.
   ----------------------------------------------------------------------------
   UART_Control_Register : UART_Control_Register_T
   with Import,
     Convention => Ada,
     Address    => UART_CONTROL_REG_ADDR;

begin
   --  Configure the UART.
   UART_Control_Register.Baud_Rate := b_115200;
   UART_Control_Register.Parity    := Even;
end Main;

</source>

Contrast this with the same functionality implemented in C. Despite being shorter in length, the register cannot be altered without using bitwise operators to manipulate the individual fields. This approach is generally considered more error-prone than using a record in Ada overlaid at the register's memory-mapped address. It is possible to define a <tt>struct</tt> type in C with bit-fields for the individual elements, however the C standard does not guarantee the layout and order of the individual fields (refer to section 6.7.2.1 paragraph 11 of the C1X standard).

<source lang="c">
#include <stdint.h>

#define UART_CNTL_REG_ADDR       0x80000040
#define UART_CNTL_BAUD_MASK      0x07
#define UART_CNTL_BAUD_SHIFT     0
#define UART_CNTL_PARITY_MASK    0x18
#define UART_CNTL_PARITY_SHIFT   3
#define UART_CNTL_IE_MASK        0x80
#define UART_CNTL_BAUD_9600      0
#define UART_CNTL_BAUD_14400     1
#define UART_CNTL_BAUD_115200    7
#define UART_CNTL_PARITY_NONE    0
#define UART_CNTL_PARITY_EVEN    1
#define UART_CNTL_PARITY_ODD     2
#define UART_CNTL_ISR_ENABLE     2
#define UART_CNTL_ISR_DISABLE    2

int main(int argc, char **argv)
{
	/** The UART control register pointer. */
	volatile uint8_t *uart_control_reg = (uint8_t*)UART_CNTL_REG_ADDR;

	// Configure the UART.
	*uart_control_reg &= ~UART_CNTL_BAUD_MASK;
	*uart_control_reg |= UART_CNTL_BAUD_115200;
	*uart_control_reg &= ~UART_CNTL_PARITY_MASK;
	*uart_control_reg |= (UART_CNTL_PARITY_EVEN << UART_CNTL_PARITY_SHIFT);
}

</source>

==See Also==

===Articles===
*[[Ada Bare bones]]
*[[Ada Runtime Library]]
*[[BOOTBOOT]] loader has an example 64 bit higher half kernel in Ada

===Operating System projects using Ada===
*[https://ironclad.nongnu.org Ironclad]
*[https://muen.codelabs.ch/ Muen Separation Kernel]
*[https://marte.unican.es/ MaRTE OS]
*[https://github.com/wookey-project/ewok-kernel EwoK]
*[https://github.com/ajxs/cxos CXOS]

===External Links===
*[https://www.adaic.org/ Ada Resource Association]
*[https://github.com/ohenley/awesome-ada Awesome-Ada: A curated list of awesome resources related to the Ada and SPARK programming language]

[[Category:Ada]]
[[Category:Languages]]
````

<a id="w004478"></a>

## W004478 — Template talk:Infobox Project

Bron-ID: W004478
Namespace: Template talk (11)
Revisie: 24611 | Datum: 2020-04-19T21:58:35Z
Bron: https://wiki.osdev.org/Template_talk:Infobox_Project
Revisie-URL: https://wiki.osdev.org/index.php?title=Template_talk%3AInfobox_Project&oldid=24611

````text
Some people use GitLab, BitBucker or etcetera. --[[User:Superleaf1995|Superleaf1995]] 16:58, 19 April 2020 (CDT)
````

<a id="w004479"></a>

## W004479 — "zig cc" Cross-Compiler

Bron-ID: W004479
Namespace: Main (0)
Revisie: 30526 | Datum: 2026-04-29T19:26:48Z
Bron: https://wiki.osdev.org/%22zig_cc%22_Cross-Compiler
Revisie-URL: https://wiki.osdev.org/index.php?title=%22zig_cc%22_Cross-Compiler&oldid=30526

````text
If you want to write an operating system in C, you'll [[Why_do_I_need_a_Cross_Compiler|need a cross-compiler]]. However, [[GCC_Cross-Compiler|building a GCC cross-compiler]] can be painful. If you just want to write C (not C++), you can use [https://andrewkelley.me/post/zig-cc-powerful-drop-in-replacement-gcc-clang.html <kbd>zig cc</kbd>] - a C cross-compiler that comes with the [https://ziglang.org/ Zig programming language]. There are [https://ziglang.org/download/ pre-compiled binaries] for Windows, Linux, MacOS, and FreeBSD.

{{FirstPerson}}

== Bare-Bones with <kbd>zig cc</kbd> ==

This section describes how to build [[Bare Bones]] using <kbd>zig cc</kbd>. First off, install the following software:

* [https://ziglang.org/download/ Zig] (tested with version 0.6.0)
* GNU binutils (we just care about the [[LD|linker]] - if <kbd>ld --version</kbd> reports "GNU ld", you're good to go)
* [[NASM]]

Start off with the standard [[Bare Bones]] setup, but use the <kbd>boot.asm</kbd> file provided in [[Bare Bones with NASM]] instructions (since Zig does not come with an assembler, just a C compiler). At that point, you can continue with [[Bare Bones]], but instead of using <kbd>i686-elf-gcc</kbd>, use the following command to build <kbd>kernel.c</kbd>:

<syntaxhighlight lang="shell-session">
$ zig build-obj --c-source kernel.c -target i386-freestanding
</syntaxhighlight>

This should create a <kbd>kernel.o</kbd> file, just the same as if you had used gcc.

When you get to the linking step, we won't be able to use <kbd>i686-elf-gcc</kbd>, but luckily, modern versions of [[LD|GNU <kbd>ld</kbd>]] are able to emulate almost any other linker, so we can use the system <kbd>ld</kbd> like so:

<syntaxhighlight lang="shell-session">
$ ld -m elf_i386 -T linker.ld -o myos.bin boot.o kernel.o
</syntaxhighlight>

And at that point, you can carry on with [[Bare Bones]], and you should have an operating system, compiled without having to compile your own cross-compiler!

[[Category:Compilers]]
````

<a id="w004480"></a>

## W004480 — User:Wesleyac

Bron-ID: W004480
Namespace: User (2)
Revisie: 24614 | Datum: 2020-04-22T22:07:26Z
Bron: https://wiki.osdev.org/User:Wesleyac
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AWesleyac&oldid=24614

````text
Hi! I'm Wesley. In the past, I've worked on [https://fuchsia.dev/ Fuchsia], a new open-source, capabilities-based operating system, primarily writing a networking stack in Rust. You can find more info about me and the sorts of things that I do on [https://blog.wesleyac.com/ my blog]. I use [https://www.mypronouns.org/they-them they/them pronouns].
````

<a id="w004481"></a>

## W004481 — Talk:"zig cc" Cross-Compiler

Bron-ID: W004481
Namespace: Talk (1)
Revisie: 24647 | Datum: 2020-05-03T04:45:11Z
Bron: https://wiki.osdev.org/Talk:%22zig_cc%22_Cross-Compiler
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3A%22zig_cc%22_Cross-Compiler&oldid=24647

````text
== Linker ==

Hullo! I suggest changing "a modern gcc toolchain" to "GNU binutils", that being the toolchain for gcc and other compilers. If I remember right, binutils doesn't depend on gcc. It's a [https://www.gnu.org/software/binutils/ separate project,] and a separate package in Linux distros.

Done! Thanks for the note :) -wesleyac
````

<a id="w004482"></a>

## W004482 — User:Wesleyac/"zig cc" Cross-Compiler

Bron-ID: W004482
Namespace: User (2)
Revisie: 24646 | Datum: 2020-05-03T04:45:11Z
Bron: https://wiki.osdev.org/User:Wesleyac/%22zig_cc%22_Cross-Compiler
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AWesleyac%2F%22zig_cc%22_Cross-Compiler&oldid=24646
Doorverwijzing: "zig cc" Cross-Compiler

````text
#REDIRECT [["zig cc" Cross-Compiler]]
````

<a id="w004483"></a>

## W004483 — User talk:Wesleyac/"zig cc" Cross-Compiler

Bron-ID: W004483
Namespace: User talk (3)
Revisie: 24648 | Datum: 2020-05-03T04:45:11Z
Bron: https://wiki.osdev.org/User_talk:Wesleyac/%22zig_cc%22_Cross-Compiler
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AWesleyac%2F%22zig_cc%22_Cross-Compiler&oldid=24648
Doorverwijzing: Talk:"zig cc" Cross-Compiler

````text
#REDIRECT [[Talk:"zig cc" Cross-Compiler]]
````

<a id="w004484"></a>

## W004484 — User:Superleaf1995/Explaining the architectures

Bron-ID: W004484
Namespace: User (2)
Revisie: 28776 | Datum: 2024-04-30T10:13:28Z
Bron: https://wiki.osdev.org/User:Superleaf1995/Explaining_the_architectures
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASuperleaf1995%2FExplaining_the_architectures&oldid=28776

````text
An architecture is a type of CPU. There are 8-bit, 16-bit, 32-bit, 64-bit and even 128-bit architectures, distinguished by RISC or CISC instruction sets. The most prevalent of them all were Intel's x86 architecture.

=== x86 ===
The most used architecture in computers. Devices using it includes:
* Every single commodity PCs
* Macs produced from 2006 to 2020
* The original Xbox
* Xbox One
* PlayStation 4
* First and second generation of ASUS Zenfone Android phones

=== Itanium ===
HP and Intel's failed attempt at making a 64-bit architecture before Intel finally switching to the x86-64 architecture with EM64T. Devices using it includes:
* HP Integrity line of servers
* Fujitsu PRIMEQUEST line of workstation produced from 2005 to 2011
* Some SGI workstation produced from 2001

=== ARM ===
The most popular architecture for embedded devices. Devices using it includes:
* Apple Silicon Macs
* All iOS and most Android phones
* Game Boy Advance
* Wii (Hollywood GPU only)
* Nintendo Switch
* Panasonic/Goldstar 3DO
* Raspberry Pi and other single-board computers
* Palm phones running Palm OS 5 or above and webOS
* TI-Nspire line of graphing calculators
* Acorn Archimedes

=== PowerPC ===
Devices using it includes:
* Macintoshes produced from 1994 to 2006
* Xbox 360
* PlayStation 3
* GameCube
* Wii
* Wii U

=== MIPS ===
Devices using it includes:
* PlayStation 1
* PlayStation 2
* Nintendo 64
* Most old SGI workstations
* Most routers

=== Motorola 68000 series ===
90's favorite architecture. Devices using it includes:
* Sega Genesis
* Commodore Amiga
* Atari Jaguar and ST
* Palm phones running Palm OS 4 or below
* Macintoshes produced from the year 1984 to 1994

=== IBM System/390 ===
Devices using it includes:
* IBM ES/9000 series
* IBM 9672 series

=== MOS 6502 ===
80's favorite architecture. 16-bit devices may use the WDC 65C816 superset of the MOS 6502 architecture. Devices using it includes:
* NES/Famicom
* SNES/Super Famicom
* Apple ][ (including Apple //c and Apple IIgs)
* Commodore PET, VIC-20 and 64
* Atari 2600, 5200, 7600 and Atari 8-bit computers

=== Intel 8080 and Zilog Z80 ===
80's second favorite architecture. Devices using it includes:
* ZX Spectrum
* MSX
* TRS-80
* BBC Micro
* Game Boy and Game Boy Color (using a custom Z80-based CPU)
* Sega Master System
* Sega Genesis (sound processing only)
* Old TI graphing calculators
````

<a id="w004485"></a>

## W004485 — User:Acurioso

Bron-ID: W004485
Namespace: User (2)
Revisie: 24667 | Datum: 2020-05-13T02:48:29Z
Bron: https://wiki.osdev.org/User:Acurioso
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAcurioso&oldid=24667

````text
I'm developing an experimental / hobby operating system based on graph nodes and edges as a primitive data structure.
````

<a id="w004486"></a>

## W004486 — File:SerenityOS-8ea4375.png

Bron-ID: W004486
Namespace: File (6)
Revisie: 24737 | Datum: 2020-06-03T05:50:17Z
Bron: https://wiki.osdev.org/File:SerenityOS-8ea4375.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ASerenityOS-8ea4375.png&oldid=24737

````text

````

<a id="w004487"></a>

## W004487 — PE+

Bron-ID: W004487
Namespace: Main (0)
Revisie: 24747 | Datum: 2020-06-07T12:18:07Z
Bron: https://wiki.osdev.org/PE%2B
Revisie-URL: https://wiki.osdev.org/index.php?title=PE%2B&oldid=24747
Doorverwijzing: PE

````text
#REDIRECT [[PE]]
````

<a id="w004488"></a>

## W004488 — Bran's Known Bugs

Bron-ID: W004488
Namespace: Main (0)
Revisie: 24787 | Datum: 2020-06-18T13:00:27Z
Bron: https://wiki.osdev.org/Bran%27s_Known_Bugs
Revisie-URL: https://wiki.osdev.org/index.php?title=Bran%27s_Known_Bugs&oldid=24787
Doorverwijzing: Bran's Kernel Development Tutorial Known Bugs

````text
#REDIRECT [[Bran's Kernel Development Tutorial Known Bugs]]
````

<a id="w004489"></a>

## W004489 — Talk:Bran's Known Bugs

Bron-ID: W004489
Namespace: Talk (1)
Revisie: 24789 | Datum: 2020-06-18T13:00:27Z
Bron: https://wiki.osdev.org/Talk:Bran%27s_Known_Bugs
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABran%27s_Known_Bugs&oldid=24789
Doorverwijzing: Talk:Bran's Kernel Development Tutorial Known Bugs

````text
#REDIRECT [[Talk:Bran's Kernel Development Tutorial Known Bugs]]
````

<a id="w004490"></a>

## W004490 — Mister Perfect

Bron-ID: W004490
Namespace: Main (0)
Revisie: 24954 | Datum: 2020-06-27T08:32:53Z
Bron: https://wiki.osdev.org/Mister_Perfect
Revisie-URL: https://wiki.osdev.org/index.php?title=Mister_Perfect&oldid=24954

````text
Mister Perfect is always searching for the optimal code. He overthrows his design decisions again and again in his quest to be the most stable, clean, elegant, useful code. Mister is good friends with [[Dae Dreamer]], however Mister has actually written some code.

== Pro's & Con's ==

Pros:
* Mister spends all his time reading, researching, and contemplating. He will be very helpful on the forums, especially on [https://forum.osdev.org/viewforum.php?f=15 OS Design & Theory], as he knows everything related to OS development.

Con:
* When he finally gets around to writing code, he'll read a paper or a forum post advocating a different approach, and he'll start over.
* It's a never-ending story...

== Going further than Mister Perfect ==
Realize that nothing and nobody is perfect. After a thorough designing process settle for the current design.

== Mister Perfect's bookshelf ==
Lots of books about OS design.

== Mister Perfect's opponents position ==
* [[Cowboy Coder]] - Cowboy doesn't believe in plans. Real men jump straight into coding.

== People and OSes related to Mister Perfect ==
* Hurd (GNU's official kernel, or more correctly library-OS)
* [[User:Nexos|Nexos]] My OS will be perfect (or close enough)
* [[User:eekee|eekee]], who spent all his time rewriting his designs until he lowered his standards

[[Category:Developer Archetype]]
````

<a id="w004491"></a>

## W004491 — Andy Microbaum

Bron-ID: W004491
Namespace: Main (0)
Revisie: 29225 | Datum: 2024-08-18T21:05:17Z
Bron: https://wiki.osdev.org/Andy_Microbaum
Revisie-URL: https://wiki.osdev.org/index.php?title=Andy_Microbaum&oldid=29225

````text
Andy abhors monolithic kernels like Linux, Windows or BSD. He rather writes a carefully designed little kernel that does some basic stuff and leaves the rest for user space.

== Pro's & Con's ==
Pro:
* easier to maintain
Con:
* more overhead
* harder to program

== Going further than Andy Microbaum ==
Consider other kernel designs: Layered, modular or even smaller (nanokernel, exokernel)

== Andy Microbaum's bookshelf ==
Research papers on microkernels

== Andy Microbaum's opponents position ==
Monolithic kernel (= all in one big kernel) [[Mono Lizzy]]

See also the famous [https://en.wikipedia.org/wiki/Tanenbaum%E2%80%93Torvalds_debate Tanenbaum–Torvalds debate]

== People and OSes related to Andy Microbaum ==
* Amoeba [https://www.cs.vu.nl/pub/amoeba/] (a libre distributed OS)
* Mach (the fundament of Hurd)
* QNX is a proprietary 32-bit POSIX OS. Its microkernel design was chosen to allow extreme modularity allowing it to run on extremely tiny hardware. Its GUI was also modular and designed similarly to a microkernel. In the 90s, a demo was made, fitting onto a single 1.4MB floppy disk the OS with window system, GUI web browser and other programs, web server, a virtual file device creating dynamic content for the web server, and network drivers.
* [[User:PeterX|PeterX]] I'm strongly against a monolithic kernel, so I fall more or less under the microkernel category
* [[User:eekee|eekee]] -- I'm planning a microkernel with the goal of making it easy to write new services, even trivial ones. The relative ease of maintaining a microkernel also attracted me, and I'm enjoying the challenge of designing interfaces for minimal overhead.

[[Category:Developer Archetype]]
````

<a id="w004492"></a>

## W004492 — User:PeterX

Bron-ID: W004492
Namespace: User (2)
Revisie: 25765 | Datum: 2021-03-18T13:13:05Z
Bron: https://wiki.osdev.org/User:PeterX
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APeterX&oldid=25765

````text

````

<a id="w004493"></a>

## W004493 — Cowboy Coder

Bron-ID: W004493
Namespace: Main (0)
Revisie: 29821 | Datum: 2026-01-02T03:40:59Z
Bron: https://wiki.osdev.org/Cowboy_Coder
Revisie-URL: https://wiki.osdev.org/index.php?title=Cowboy_Coder&oldid=29821

````text
Cowboy Coder doesn't waste time planning what he's going to work on or thinking through problems. Cowboy is ready to jump into writing code. Any problems we can fix when we get to them.

Cowboy follows the [https://productcoalition.com/the-code-and-fix-model-2cabd4c48166 Code and Fix] software development method.


==Pros and Cons==

Pros:
* He has a lot of passion.
* He doesn't waste time getting started.
* He will often follow a tutorial as a starting point but has no idea what to do once he reaches the end.
* He sees himself as a [https://en.wikipedia.org/wiki/Jack_of_all_trades,_master_of_none Jack of all trades].

Cons:
* Once the complexity of the task catches up to him, Cowboy doesn't have a plan, so he quickly looses motivation. As a result, Cowboy has many abandoned and unfinished projects. For example, he'll try to port a couple of [[C Library|C libraries]] to no avail, he'll try to [[Creating a C Library|roll his own]], giving up not soon after, then deciding to restart over writing his kernel in [[Assembly]] or [[D]].
* His motivation easily changes - one moment Cowboy will be working a [[Rolling Your Own Bootloader|bootloader]], then read a blog post about UI design, and drop everything to work on a [[GUI]].
* Trivial problems, when first encountered and not planned for, often prevent Cowboy from [https://en.wiktionary.org/wiki/see_the_forest_for_the_trees seeing the forest for the trees]. For example, he'll focus his time on trying to avoid the next line of code from triple faulting, without caring to understand the bigger picture.
* Cowboy's questions on the forum are super-focused, with no context of the greater problem he's trying to solve, mainly because he doesn't know the greater problem himself.

==Going further than Cowboy Coder==

A vision of what you want your OS to be, and a TODO list to get there.

==Cowboy's Bookshelf==

Nothing. Reading takes time away from starting Cowboy's next project.

==Opponents Position==

Cowboy's opponents are [[Mister Perfect]] who overplans and it's never good enough, and [[Dae Dreamer]] who has never written a line of code.

==People And OSes Related To Cowboy==

* [[User:Jankowalski25|Jankowalski25]] Definitely me. A lot of unfinished projects and test tools, but almost empty list of completed projects. Everything is flying and the question "what now?" always comes back.
* [[User:Vladislav12|Vladislav12]] No plan, no roadmap. Just me, vim, gcc and 50 private unfinished projects on github and sometimes, the docs are just bad. Trying yo change.
* ''Insert your names here''

[[Category:Developer Archetype]]
[[Category:Developer Anti-Pattern]]
````

<a id="w004494"></a>

## W004494 — Dae Dreamer

Bron-ID: W004494
Namespace: Main (0)
Revisie: 29207 | Datum: 2024-08-09T09:35:52Z
Bron: https://wiki.osdev.org/Dae_Dreamer
Revisie-URL: https://wiki.osdev.org/index.php?title=Dae_Dreamer&oldid=29207

````text
Dae Dreamer spends his all of his free time reading about operating systems, compilers, languages, IPC, memory management, UI, device management, etc. Even though he spends hours each day reading about OS development and has fleshed his entire OS in his head, Dae claims he never has time to actually work on it - either due to work commitments, a spouse, other hobbies, or just procrastination. But, he considers himself part of the OS development community. Dae is friends with [[Mister Perfect]].

==Pros and Cons==

Pros:
* He knows everything there is to know about operating systems, and could teach university courses on the topic.
* He lurks on the [https://forum.osdev.org/viewforum.php?f=15 OS Design & Theory] sub-forum, often giving well thought out page long responses. He even occasionally asks a question to solve a problem in his dream OS, which doesn't yet exist.

Cons:
* He's never written a line of code.

==Going further than Dae Dreamer==

Follow a [[Tutorials|tutorial]] and start some coding.

==Dae's Bookshelf==

A bit of everything. One day this knowledge will be useful.

==Opponents Position==

[[Cowboy Coder]] - Dae's arch enemy.

==People And OSes Related To Dae==
* [[User:Egoros7|Egoros7]]: My OS will be perfect and stable as rock! Well, remained to create it :D
* [[User:eekee|eekee]] is Dae Dreamer's younger brother and a close cousin of [[Mister Perfect]]. He has been trying to distance himself from the family for years, but their influence lingers.

[[Category:Developer Archetype]]
[[Category:Developer Anti-Pattern]]
````

<a id="w004495"></a>

## W004495 — Sir Dunning-Kruger

Bron-ID: W004495
Namespace: Main (0)
Revisie: 25719 | Datum: 2021-02-24T19:28:34Z
Bron: https://wiki.osdev.org/Sir_Dunning-Kruger
Revisie-URL: https://wiki.osdev.org/index.php?title=Sir_Dunning-Kruger&oldid=25719

````text
Sir D.K. suffers from incompetence so big that he thinks he is very competent.

== Pro's & Con's ==
Pro:
* feels good
Con:
* just incompetent

== Going further than Sir D.K. ==
Read a lot about OS design and techniques. Then you see that's not easy.

== Sir D.K.'s bookshelf ==
Some science fiction novels

== Sir D.K.'s opponents position ==
Thinking one is too stupid despite actually being able to program.

== People and OSes related to Sir D.K. ==
* People saying "I will emulate Windows, macOS and Linux."

[[Category:Developer Archetype]]
[[Category:Developer Archetype]]
[[Category:Developer Anti-Pattern]]
````

<a id="w004496"></a>

## W004496 — User:Superleaf1995/lowFS

Bron-ID: W004496
Namespace: User (2)
Revisie: 24962 | Datum: 2020-06-29T05:28:09Z
Bron: https://wiki.osdev.org/User:Superleaf1995/lowFS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASuperleaf1995%2FlowFS&oldid=24962

````text
{{stub}}

lowFS stands for '''Low Filesystem''', simple huh?.

Is a table-filesystem designed by me (Superleaf1995) because why not?

In this filesystem each entry points to a portion of a file, this allows loading parts of the file progressively. If any of the tables is damaged, a checksum every 8 entries is made to ensure nothing is wrong. lowFS assumes the disk is a plain array of bytes.

This filesystem also tries to be simple as possible, and be compatible with 16-bit hosts as well as 64-bit ones.

== Bootsector ==
There are three ways to identify a lowFS disk: checking byte 0x02, checking the last 4 bytes of sector 4 or checking the last 4 bytes of the disk. It should contain the characters 'LFFL' (ASCII). Followed by a table:
<source lang="c">
struct lowFS_megatable {
    uint8_t attribute; // Attribute byte
    uint8_t version; // lowFS version
    union {
        struct lowFS_floppy {
            uint8_t heads;
            uint8_t bytes_per_sector;
            uint16_t sectors_per_track;
        };
        struct lowFS_non_floppy {
            uint32_t n_lba;
        };
    }table;
    uint16_t reserved;
}__attribute__((packed));
</source>

Attribute:
{| {{wikitable}}
|-
! Bit
! Purpose
|-
| 0
| If set, the disk is in big endian, if clear, the disk is in little endian
|-
| 1
| If set, disk has bad sectors
|-
| 2-7
| TBD
|}

* If all bits are set, the endianess is little, the disk has no bad sectors (assumed) and there are no checksums

== Entry ==
The entry header is basically like a FAT entry, however, this is a header ''for entries'' not for files.
<source lang="c">
struct lowFS_entry_header {
    uint8_t reserved;      // Should always be 0xCC
    uint8_t entry_type;    // Type of entry
    uint32_t name_len;     // Length of the name (variable, upon the OS how to handle)
    uint64_t nn_entry;     // Next part of this entry (pointer)
    uint64_t previous;     // Previous entry header in the chain (pointer)
    uint64_t next_entry;   // Next entry header in the chain (pointer)
    uint64_t size;         // Size of entry (lowFS_entry struct not included). If set to 0, the host must know the size parsing entry type
};
</source>

* nn_entry should be set to 0 if this is the final part of the entry.
* next_entry should be set to 0 if this is the final part of the chain of entries.

== Entry types ==
Following the entry_t header, follows the contents of the entry. Notice that these varies in size, but always should align to 8-bytes.

If entry_type is set to '''ENTRY_TYPE_FILEMETADATA''' (0xF0):
<source lang="c">
struct lowFS_entry_filemetadata {
    uint64_t filename_len; // length of filename
    uint64_t len; // length of file in bytes
    uint8_t filename[filename_len]; // variable size, kernel should alloc memory for incoming filename
    uint8_t encrypt; // If it's nonzero, the file is encrypted with some TBD encryption system
    uint8_t perm; // use for unix permissions
};
</source>
* Filename should be ended with a NULL character

If entry_type is set to '''ENTRY_TYPE_FILEPART''' (0xF2):
<source lang="c">
struct lowFS_entry_filepart {
    uint64_t clen; // Length of the bytes of the file
    uint8_t content[clen]; // variable size, kernel should alloc memory for incoming file content
    uint16_t checksum; // verifies that the data in the file is correct (First byte of content ANDed by the last one)
};
</source>

If entry type is set to '''ENTRY_TYPE_DIRECTORYMETADATA''' (0xF4):
<source lang="c">
struct lowFS_entry_directorymetadata {
    uint64_t dirname_len; // length of dirname
    uint8_t dirname[dirname_len]; // variable size, kernel should alloc memory for incoming dirname
    uint8_t perm; // use for unix permissions
};
</source>

== Checksum ==
The checksum is in total, 8-bytes.

The checksum is a simple 2-byte value 0x53BC. This byte is placed after 8 entries (directorymetadata,filepart,etc). This word should be used for hosts that confides that the disk is OK, in this case, it should skip the following 6-bytes.

In case the host wants to read the 6-byte value, it should AND the aforementioned 2-byte value with the previous 2 bytes before the checksum bytes. Regardless of it being a entry header or a regular entry or not. The result should match the checksum. If a FILEPART entry is before the checksum, the following 4-bytes are the higher 4-bytes of the size of the file. Otherwise it's the 4-checksum bytes.

After the checksum, the entries continue as normal.

A checksum cannot be after the last entry or entry header in a chain.

<source lang="c">
struct lowFS_checksum {
    uint16_t val; // 0x53BC
    uint16_t bf; // Previous 2 bytes ANDed with 0x53BC
    uint32_t fp; // FILEPART size high bytes
};
</source>
````

<a id="w004497"></a>

## W004497 — User:Superleaf1995/uniFS

Bron-ID: W004497
Namespace: User (2)
Revisie: 24886 | Datum: 2020-06-22T12:26:53Z
Bron: https://wiki.osdev.org/User:Superleaf1995/uniFS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASuperleaf1995%2FuniFS&oldid=24886
Doorverwijzing: User:Superleaf1995/lowFS

````text
#REDIRECT [[User:Superleaf1995/lowFS]]
````

<a id="w004498"></a>

## W004498 — File:Turbo C Preview.png

Bron-ID: W004498
Namespace: File (6)
Revisie: 24889 | Datum: 2020-06-22T13:25:50Z
Bron: https://wiki.osdev.org/File:Turbo_C_Preview.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ATurbo_C_Preview.png&oldid=24889

````text
self-taken. using dosbox.
````

<a id="w004500"></a>

## W004500 — Printing

Bron-ID: W004500
Namespace: Main (0)
Revisie: 28513 | Datum: 2023-12-13T07:22:11Z
Bron: https://wiki.osdev.org/Printing
Revisie-URL: https://wiki.osdev.org/index.php?title=Printing&oldid=28513

````text
{{stub}}

== Overview ==
Printing is the process of taking text, documents or photos from a computer and putting them onto a paper using a device called printer.

== Printer interface ==
To print something, you need to establish a connection from the computer to the printer. This can be done through a cable or wireless using [[Wi-Fi]]. In the old days, printers used a big port called [[parallel port]]. Now most of the printers have a [[USB]] type-B port.

== Printing basics ==
The easiest way to print something is using the [[parallel port]] because you don't need to have anything else than ports set-up for it to work.

== See also ==
[[Parallel port]]

=== External links ===
* [http://www.hp.com/ctg/Manual/bpl13210.pdf HP 5e PCL reference]

[[Category:Printing]]
````

<a id="w004501"></a>

## W004501 — Mono Lizzy

Bron-ID: W004501
Namespace: Main (0)
Revisie: 29884 | Datum: 2026-01-12T17:11:27Z
Bron: https://wiki.osdev.org/Mono_Lizzy
Revisie-URL: https://wiki.osdev.org/index.php?title=Mono_Lizzy&oldid=29884

````text
Lizzy prefers a single big kernel all compiled at once.

== Pro's & Con's ==
Pro:
* Faster because there's no context switches between drivers and no [[IPC]] overhead.
Con:
* Driver bugs may corrupt data in kernel space and/or cause panics. A buggy driver can bring down the entire kernel.
* Beginners may not realize they need to organize a [[monolithic kernel]] just as carefully as a [[microkernel]] if not more so
* All the drivers get loaded into memory at once; loading drivers at run-time requires an extra system
* Security: higher attack surface due to the amount of code running on kernel mode, with full privileges, compared to a [[microkernel]] for example.

== Going further than Lizzy ==
* Add a module system to your kernel so you can load drivers at run-time
* Add good IPC and a system to allow userspace programs to access certain hardware, giving you a hybrid kernel
Note that neither are necessarily good ideas. Monolithic or microkernel is a matter of choice, not rungs on the ladder to progress. A module system may seem more obviously good, but it depends how many drivers you have.

== Lizzy's bookshelf ==
[https://en.wikipedia.org/wiki/Lions%27_Commentary_on_UNIX_6th_Edition,_with_Source_Code Lions' Commentary on UNIX 6th Edition, with Source Code] and other books on traditional Unix, Linux, and BSD kernel design. Unix v6 or xv6 source code, perhaps the source of some BSD.

== Lizzy's opponents position ==
Microkernel and similar (layered, modular).

See also the famous [https://en.wikipedia.org/wiki/Tanenbaum%E2%80%93Torvalds_debate Tanenbaum–Torvalds debate]

== People and OSes related to Lizzy ==
* Linux
* BSD
* Unix v6
* xv6
* Linus Torvalds, the man behind Linux

''Not'' Windows; that's a hybrid.

[[Category:Developer Archetype]]
````

<a id="w004502"></a>

## W004502 — Talk:Mono Lizzy

Bron-ID: W004502
Namespace: Talk (1)
Revisie: 24906 | Datum: 2020-06-22T17:39:40Z
Bron: https://wiki.osdev.org/Talk:Mono_Lizzy
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMono_Lizzy&oldid=24906

````text
Having a big kernel is not bad at all :( --[[User:Superleaf1995|Superleaf1995]] 12:25, 22 June 2020 (CDT)

Indeed, this was written by [[Andy Microbaum]], I think. The 'spaghetti code' statement is just plain wrong for any properly-managed project, no matter how 'monolithic' it is. I think I'll go ahead and fix that. Might leave a small caution about it & similar issues though. [[User:Eekee|Eekee]] 12:39, 22 June 2020 (CDT)
````

<a id="w004503"></a>

## W004503 — Kernel Multitasking

Bron-ID: W004503
Namespace: Main (0)
Revisie: 30655 | Datum: 2026-05-28T14:09:53Z
Bron: https://wiki.osdev.org/Kernel_Multitasking
Revisie-URL: https://wiki.osdev.org/index.php?title=Kernel_Multitasking&oldid=30655

````text
When a task (process, thread, application, program, ...) is using the CPU, some of the task's state is stored in the CPU - things like the contents of registers, the address of the task's current stack, the current instruction pointer, which virtual address space the task is using, etc.

A task switch involves saving any state that the previous task had in the CPU (that the task may have modified) and then loading any state that the next task expects to be in the CPU. Typically a kernel stores at some of this state on the task's (kernel) stack and some of the state in data structure/s in kernel-space (e.g. in a "task control block" structure).

There are two very different ways of implementing multi-tasking, depending on how the kernel stacks are done. However, the mechanisms outlined below are not mutually exclusive, and both may be used at the same time in different situations, to potentially achieve better memory usage and performance.

==Kernel Stack Per Task==

In most kernels, each task has its own kernel stack used when a task moves between user-space (and user stack) and kernel (and kernel stack). When the kernel is entered (as a result of interrupt, exception, system call, etc.), all the registers of userspace are typically pushed onto its corresponding kernel stack. Then, task switches only ever happen after something else (IRQ, exception, system call) has already caused the task to switch to kernel code and kernel stack; which means that task switches can only ever occur between tasks that are running kernel code. Additionally, since the kernel stack holds all the context, it makes it easy to make the kernel be preemptible at any moment, and not just while running in user mode. This is the recommended model for the beginners.

For example (NASM syntax, 32-bit 80x86, "cdecl" calling convention, single-CPU only/globals used for "per CPU" variables, no support for FPU/MMX/SSE/AVX, untested):

<syntaxhighlight lang = "nasm">
;C declaration:
;   void switch_tasks(thread_control_block *next_thread);
;
;WARNING: Caller is expected to disable IRQs before calling, and enable IRQs again after function returns

switch_tasks:

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

    cmp eax,ecx                   ;Does the virtual address space need to being changed?
    je .doneVAS                   ; no, virtual address space is the same, so don't reload it and cause TLB flushes
    mov cr3,eax                   ; yes, load the next task's virtual address space
.doneVAS:

    pop ebp
    pop edi
    pop esi
    pop ebx

    ret                           ;Load next task's EIP from its kernel stack
</syntaxhighlight>

The following is an example of the same function for the System V X86_64 calling convention:
<syntaxhighlight lang = "nasm">
switch_tasks:
    ; Save previous task's state
    push rbp
    push rbx
    push r12
    push r13
    push r14
    push r15

    mov rdi,[current_task_TCB]
    mov [rdi+TCB.ESP],rsp

    ; Adjust TSS, IA32_SYSENTER_ESP and other registers to point to the rights stack,
    ; similarly to the previous example (may also be done in higher level language
    ; by the caller. Also, switch the active page tables, fp/vector data, and so on here.
    ; All of that depends on the kernel layout

    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret
</syntaxhighlight>

When called, the example function above behaves very similarly to sched_yield from POSIX, where it returns to the context of the caller after the other threads are done running and have yielded their execution, and the caller was scheduled to run again.

Note that the low-level task switch code should be written in pure assembly. If it's written in inline assembly in the middle of a C function then the compiler may add its own "function prologue" and "function epilogue" code that changes the layout of the stack. This is important because when a new task is being created the kernel needs to put values on the new task's kernel stack to match the values that the "switch_tasks" expects to pop off of the new task's stack.

Also note that (for kernels primarily written in higher level languages) the low-level task switch code may be the only part of the scheduler that needs to be rewritten when porting the kernel to a different architecture.

This low-level task switch code may be called directly from other places in the kernel (e.g. when a task is being pre-empted and the caller knows exactly which task is pre-empting the current task); but may also be called by higher-level code that selects a task to switch to (e.g. when the currently running task is being terminated and the kernel doesn't know which task to switch to).

==Kernel Stack Per CPU==

For some rare kernels (mostly only a few micro-kernels), there is only one kernel stack per CPU (and each task doesn't have its own kernel stack), which gets (primarely) reused by the tasks/threads, instead of having their own individual stacks. Because of that, the common stacks cannot store any of the thread's state, which makes the task switching (and as a result, the rest of the kernel) radically different from the previous model. In theory, this has the potential to save some memory (the stacks are typically 4-8KB in size on the smaller end, or much larger in the more complex kernels, or some programming languages, such as Rust) and achieve better performance because of the cache locality, especially for microkernels, which execute more threads, and do more context switches. However, this comes with a lot of trade-offs discussed later, and is not recommended to beginners. 

Compared to the stack per thread model, instead of saving the state to the stack, the userspace state typically gets stored to some kind of current per-thread data structure ("thread control block", TCB) immediately after the switch from userspace to kernel; and symmetrically, when returning to the userspace, the state is also restored from the active thread's TCB (which might be different from the entry if the kernel has decided to schedule a different thread).

Then, the kernel typically has some "thread to return to" variable (for each CPU), and any kernel code can modify this variable when it decides a task switch should happen, instead of calling yield/switch_tasks function as explained before. When the kernel wants to block, it cannot just leave its context on the stack and instead has to explicitly save and set its resume context (continuations), or fall back to the previous scheme and allocate a stack for it.

In addition, the preemption of the kernel code also becomes more difficult and has to be designed much more carefully, since it is no longer possible to just preempt the kernel code at any arbitrary place and immediately schedule a different task to run. To combat this, the kernel may limit what is allowed to preempt it, not reschedule while preempted (to not rug pull the preempted context from its thread context), and use explicit preemption points during long operations (which may check if the other task needs to run, save the progress to the continuation point, give up what it's doing, and explicitly run the different task). Another option is making the kernel completely nonpreemptible, which might be good for a carefully designed microkernel, but comes with a big latency issues for monolithic kernels (or usual kernels in general).

For that reasons, "kernel stack per CPU" is not recommended for beginners; partly because it's only beneficial in rare cases (e.g. it'd be silly for a monolithic kernel), and partly because it becomes a lot more complex as soon as you try to overcome the latency problems due to the reasons outlined before. However, it might still be useful as a later advanced optimization in some sort of hybrid system, where the kernel mostly uses stack per thread, but is allowed to give up the stacks of the treads that are not currently running (which subjects them to use of the mechanisms outlined above) to potentially improve the memory consumption and performance.


==See Also==
===Articles===
*[[Context Switching]]
*[[Multitasking Systems]]
*[[Monotasking Systems]]
*[[Scheduling Algorithms]]

===Threads===
*[[Topic:9669|Designing a kernel to be preemptible]]
*[[Topic:812|Making a fully preemptible kernel]]

=== External Links ===
*[[wikipedia:Computer_multitasking| Computer Multitasking]] on Wikipedia.
* https://www.cse.iitd.ac.in/~sbansal/os/bib/draves91continuations.pdf

[[Category:Task Models]]
[[Category:Multitasking]]
````

<a id="w004504"></a>

## W004504 — Talk:Kernel Multitasking

Bron-ID: W004504
Namespace: Talk (1)
Revisie: 24924 | Datum: 2020-06-24T05:48:28Z
Bron: https://wiki.osdev.org/Talk:Kernel_Multitasking
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AKernel_Multitasking&oldid=24924
Doorverwijzing: Talk:Cooperative Multitasking

````text
#REDIRECT [[Talk:Cooperative Multitasking]]
````

<a id="w004505"></a>

## W004505 — File:Qemu gui mkfree1.png

Bron-ID: W004505
Namespace: File (6)
Revisie: 24938 | Datum: 2020-06-25T12:06:12Z
Bron: https://wiki.osdev.org/File:Qemu_gui_mkfree1.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AQemu_gui_mkfree1.png&oldid=24938

````text
Screenshot of the graphical environment developed to run on the mkfreeOS kernel.
````

<a id="w004506"></a>

## W004506 — User:Rhodez/QuasiOS

Bron-ID: W004506
Namespace: User (2)
Revisie: 24978 | Datum: 2020-07-02T08:16:42Z
Bron: https://wiki.osdev.org/User:Rhodez/QuasiOS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ARhodez%2FQuasiOS&oldid=24978

````text
{{Infobox Project
|name=QuasiOS
|developers=[[User:Rhodez|Jørn guldberg]], Patrick Jakobsen and Jakob Kjær-Kammersgaard 
|kerneltype=TBD
|platform=x86
|taskmodel=TBD
|language=[[Assembly]] ([[NASM|NASM]]), [[C]] and OSL
|license=TBD
}}

'''QuasiOS''' is an 64-bit operating system for x86. More info coming.
````

<a id="w004507"></a>

## W004507 — User:Rhodez

Bron-ID: W004507
Namespace: User (2)
Revisie: 24982 | Datum: 2020-07-02T08:49:56Z
Bron: https://wiki.osdev.org/User:Rhodez
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ARhodez&oldid=24982

````text
Hi, I'm Jørn Guldberg from Denmark. 

I'm working on [https://wiki.osdev.org/User:Rhodez/QuasiOS QuasiOS] with two of my friends. 
We also have a more niche project which is called [https://wiki.osdev.org/User:Rhodez/SingOS SingOS].


If you want to catch me send me a mail: jorn@guld-berg.dk
````

<a id="w004508"></a>

## W004508 — User:Rhodez/SingOS

Bron-ID: W004508
Namespace: User (2)
Revisie: 24981 | Datum: 2020-07-02T08:46:05Z
Bron: https://wiki.osdev.org/User:Rhodez/SingOS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ARhodez%2FSingOS&oldid=24981

````text
{{Infobox Project
|name=SingOS
|developers=[[User:Rhodez|Jørn guldberg]], Patrick Jakobsen and Jakob Kjær-Kammersgaard 
|kerneltype=TBD
|platform=x86 16-bit real-mode
|taskmodel=TBD
|language=[[Assembly]] ([[NASM|NASM]])
|license=Open Source
}}

'''SingOS''' is an 16-bit operating system for x86. The Legacy edition is aimed to be compliant with the original IBM PC BIOS. 


== 2 year anniversary ==
When the project turned two years, we ran the system on a computer from 1992 with a Intel 386sx processor.
[[File:SingOS-2y.jpeg|400px|thumb|left|2 year anniversary]]
````

<a id="w004509"></a>

## W004509 — File:SingOS-2y.jpeg

Bron-ID: W004509
Namespace: File (6)
Revisie: 24980 | Datum: 2020-07-02T08:44:17Z
Bron: https://wiki.osdev.org/File:SingOS-2y.jpeg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ASingOS-2y.jpeg&oldid=24980

````text
2 year anniversary
````

<a id="w004511"></a>

## W004511 — User:Andreykapustin

Bron-ID: W004511
Namespace: User (2)
Revisie: 24995 | Datum: 2020-07-08T15:04:42Z
Bron: https://wiki.osdev.org/User:Andreykapustin
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAndreykapustin&oldid=24995

````text
I'm currently working on a new language SPL (System Programming Language) to replace C for OS implementation.
The idea was to take C as a starting point, and add missing features (error handling, strict typing, programming-by-contract, modularity, and so on).

I've been doing it for quite some time, so the SPL specifications are finished, and the SPL compiler (written in SPL itself) can correctly process any SPL code. I'm only now starting to write some OS-related code, at the same time finishing the native IA-32 compiler backend.

Anyone interested can find out more at the project site at [https://sourceforge.net/projects/splcomp/], from which language/library specifications can be downloaded as well as the current stable compiler (alas, Windows only at this time, but it works under WINE/Linux too).
````

<a id="w004512"></a>

## W004512 — GOP

Bron-ID: W004512
Namespace: Main (0)
Revisie: 30351 | Datum: 2026-03-18T19:37:13Z
Bron: https://wiki.osdev.org/GOP
Revisie-URL: https://wiki.osdev.org/index.php?title=GOP&oldid=30351

````text
This is the new standard for UEFI that superseded [[VESA]] (BIOS) and UGA (EFI 1.0).

== Graphics Output Protocol ==
It has basically the same functions as VESA, you can query the modes, set the modes.  It also provides an efficient BitBlitter function, which you can't use from your OS unfortunately. GOP is an EFI Boot Time Service, meaning you can't access it after you call ExitBootServices().  However, the framebuffer provided by GOP persists, so you can continue to use it for graphics output in your OS.

NOTE: UEFI uses its own ABI. You can either configure your build environment to use that globally, or you must use a wrapper function. These examples use the latter for compatibility reasons. Omit uefi_call_wrapper if you have configured your build system for the former. See [[GNU-EFI]] for more information.

=== Detecting GOP ===
As with other UEFI protocols, you have to locate a structure with the function pointers first using the protocol's GUID.
<syntaxhighlight lang="c">
  EFI_GUID gopGuid = EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID;
  EFI_GRAPHICS_OUTPUT_PROTOCOL *gop;

  status = uefi_call_wrapper(BS->LocateProtocol, 3, &gopGuid, NULL, (void**)&gop);
  if(EFI_ERROR(status))
    PrintLn(L"Unable to locate GOP");
</syntaxhighlight>
GOP is the default protocol, so you should be able to locate it on all UEFI firmware. It can probably only fail if you're on an old EFI (pre-UEFI) machine, like an Itanium-based computer or a Mac released before 2009.

If your kernel uses GRUB, you need to insert a module called "all_video" before loading the kernel to add UEFI GOP compatibility. Not doing so will display a message saying "WARNING: no console will be available to OS".
<syntaxhighlight lang="c">
insmod all_video

menuentry "Example OS" {
  multiboot2  /boot/kernel.bin
  boot
}
</syntaxhighlight>

=== Get the Current Mode ===
In order to get the mode code for the current video mode, you must set the mode as well to circumvent some buggy UEFI firmware. Otherwise this is done using the QueryMode function, and then gop->Mode->Mode will contain the code.
<syntaxhighlight lang="c">
  EFI_GRAPHICS_OUTPUT_MODE_INFORMATION *info;
  UINTN SizeOfInfo, numModes, nativeMode;

  status = uefi_call_wrapper(gop->QueryMode, 4, gop, gop->Mode==NULL?0:gop->Mode->Mode, &SizeOfInfo, &info);
  // this is needed to get the current video mode
  if (status == EFI_NOT_STARTED)
    status = uefi_call_wrapper(gop->SetMode, 2, gop, 0);
  if(EFI_ERROR(status)) {
    PrintLn(L"Unable to get native mode");
  } else {
    nativeMode = gop->Mode->Mode;
    numModes = gop->Mode->MaxMode;
  }
</syntaxhighlight>

=== Query Available Video Modes ===
Similarly to VESA, there's no standard mode codes, rather you have a function to query the available modes. Now you know how many modes there are (numModes above), and which one is currently set (nativeMode). You can iterate on the modes and query the information structure for each:
<syntaxhighlight lang="c">
for (i = 0; i < numModes; i++) {
  status = uefi_call_wrapper(gop->QueryMode, 4, gop, i, &SizeOfInfo, &info);
  PrintLn(L"mode %03d width %d height %d format %x%s",
    i,
    info->HorizontalResolution,
    info->VerticalResolution,
    info->PixelFormat,
    i == nativeMode ? "(current)" : ""
  );
}
</syntaxhighlight>

=== Set Video Mode and Get the Framebuffer ===
This is pretty easy. The mode argument is between 0 and numModes.
<syntaxhighlight lang="c">
  status = uefi_call_wrapper(gop->SetMode, 2, gop, mode);
  if(EFI_ERROR(status)) {
    PrintLn(L"Unable to set mode %03d", mode);
  } else {
    // get framebuffer
    PrintLn(L"Framebuffer address %x size %d, width %d height %d pixelsperline %d",
      gop->Mode->FrameBufferBase,
      gop->Mode->FrameBufferSize,
      gop->Mode->Info->HorizontalResolution,
      gop->Mode->Info->VerticalResolution,
      gop->Mode->Info->PixelsPerScanLine
    );
  }
</syntaxhighlight>
To get the same value as scanline in VESA (also commonly called pitch in many graphics libraries), you have to multiply PixelsPerScanLine by the number of bytes per pixel. That can be detected by examining the gop->Mode->Info->PixelFormat field. For example with 32 bit packed pixel formats,
<syntaxhighlight lang="c">
  pitch = 4 * gop->Mode->Info->PixelsPerScanLine;
</syntaxhighlight>

=== Plotting Pixels ===
{{Main|Drawing In Protected Mode}}
Now you can use the returned framebuffer exactly the same way as you would with VESA, there's absolutely no difference. 
To calculate the offset for an (X,Y) coordinate on screen, do pitch*Y+pixelbytes*X. For example for 32 bit true-color (where pixelbytes is 4):
<syntaxhighlight lang="c">
static inline void PlotPixel_32bpp(int x, int y, uint32_t pixel)
{
   *((uint32_t*)(gop->Mode->FrameBufferBase + 4 * gop->Mode->Info->PixelsPerScanLine * y + 4 * x)) = pixel;
}
</syntaxhighlight>
For drawing characters, you can use the same method described in [[VGA Fonts]].

=== Don't Read From Video Memory ===
Reading from the video memory is very slow. Use [[Double Buffering|double buffering]] instead.

== Acceleration ==
EFI GOP provides some rudimentary accelerated 2D Graphics functions. '''These are only available as''' '''BootServices''', so once you call ExitBootServices(), you need to use the Framebuffer directly (easy, but slow), or load your own graphics card driver (fast, but hard).

An alternative approach might be to frankenstein the GOP driver (which typically can be found in a BAR on the graphics adapter, or you can query UEFI with ''LocateProtocol'' and then use IMAGE_LOADED_PROTOCOL to find the image. This would be a massive hack, though, for some fairly rudimentary display services.

Acceleration is provided through one function, '''Block Transfer'''. It has several modes.
<syntaxhighlight lang="c">
typedef
EFI_STATUS
(EFIAPI *EFI_GRAPHICS_OUTPUT_PROTOCOL_BLT) (
  IN  EFI_GRAPHICS_OUTPUT_PROTOCOL            *This,
  IN  EFI_GRAPHICS_OUTPUT_BLT_PIXEL           *BltBuffer,   OPTIONAL
  IN  EFI_GRAPHICS_OUTPUT_BLT_OPERATION       BltOperation,
  IN  UINTN                                   SourceX,
  IN  UINTN                                   SourceY,
  IN  UINTN                                   DestinationX,
  IN  UINTN                                   DestinationY,
  IN  UINTN                                   Width,
  IN  UINTN                                   Height,
  IN  UINTN                                   Delta         OPTIONAL
  );


</syntaxhighlight>

=== Modes ===
{| class="wikitable"
|+
!Name
!Description
|-
|EfiBltVideoFill
|Fills the specified area of the screen with the pixel value in the BltBuffer at (0, 0)
|-
|EfiBltVideoToBltBuffer
|Copies the specified source rectangle from the display to the BltBuffer at (Dx, Dy)
|-
|EfiBltBufferToVideo
|Copies from the BltBuffer to the video display at (Dx, Dy)
|-
|EfiBltVideoToVideo
|Copies from one rectangle of the display to another.
|}
'''Delta''' should be set to the length of a line of the BltBuffer when using it in a copy mode, although if the (X,Y) are zero and delta is zero, the GOP will assume that the Width parameter is the length of a line.

==See Also==
===Articles===
* [[VESA]] - the former video standard

=== External Links ===
* GOP description in the [https://uefi.org/specs/UEFI/2.11/12_Protocols_Console_Support.html#graphics-output-protocol UEFI Specification]
* GOP at [https://en.wikipedia.org/wiki/Graphics_Output_Protocol Wikipedia]
* UEFI Graphic Features at [https://en.wikipedia.org/wiki/Unified_Extensible_Firmware_Interface#Graphics_features Wikipedia]
* [https://tianocore-docs.github.io/edk2-UefiDriverWritersGuide/draft/23_graphics_driver_design_guidelines/232_graphics_output_protocol_implementation EDK2 on implementing GOP]
* [https://www.intel.com/content/dam/doc/guide/uefi-driver-graphics-controller-guide.pdf Intel documentation on GOP]

[[Category:UEFI]]
[[Category:Video]]
````

<a id="w004513"></a>

## W004513 — File:Managarm-demo.png

Bron-ID: W004513
Namespace: File (6)
Revisie: 25040 | Datum: 2020-08-05T18:12:34Z
Bron: https://wiki.osdev.org/File:Managarm-demo.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AManagarm-demo.png&oldid=25040

````text
A demo of Managarm running xclock and glxgears in Weston using XWayland
````

<a id="w004514"></a>

## W004514 — File:Cyjon.png

Bron-ID: W004514
Namespace: File (6)
Revisie: 25055 | Datum: 2020-08-13T06:09:54Z
Bron: https://wiki.osdev.org/File:Cyjon.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ACyjon.png&oldid=25055

````text

````

<a id="w004515"></a>

## W004515 — File:ISBN 198508659X.jpg

Bron-ID: W004515
Namespace: File (6)
Revisie: 25085 | Datum: 2020-08-24T02:57:34Z
Bron: https://wiki.osdev.org/File:ISBN_198508659X.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_198508659X.jpg&oldid=25085

````text

````

<a id="w004516"></a>

## W004516 — Python

Bron-ID: W004516
Namespace: Main (0)
Revisie: 30153 | Datum: 2026-03-06T16:52:02Z
Bron: https://wiki.osdev.org/Python
Revisie-URL: https://wiki.osdev.org/index.php?title=Python&oldid=30153

````text
{{Template:Stub}}

Python is a high-level programming language that is used in many projects.

== See Also ==
* [[Porting Python]]

[[Category:Python]]
[[Category:Languages]]
````

<a id="w004517"></a>

## W004517 — File:1267.png

Bron-ID: W004517
Namespace: File (6)
Revisie: 25091 | Datum: 2020-08-25T13:28:23Z
Bron: https://wiki.osdev.org/File:1267.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3A1267.png&oldid=25091

````text

````

<a id="w004518"></a>

## W004518 — Busy loop

Bron-ID: W004518
Namespace: Main (0)
Revisie: 28366 | Datum: 2023-10-04T11:58:33Z
Bron: https://wiki.osdev.org/Busy_loop
Revisie-URL: https://wiki.osdev.org/index.php?title=Busy_loop&oldid=28366

````text
== Definition ==
A busy loop is a loop that continuously polls for an event. An alternative to this inefficient use of resources, for example, is to ask for an [[interrupt]] or sleep until the event occurs.

== Use Cases ==
Busy loops, while expensive, can be used efficiently in some cases. You can implement a mutex like so, using a busy loop:
<source lang=c>
int lock(mutex *mutex) {
    // Wait for the mutex to become free.
    while(mutex->in_use);
    mutex->in_use = 1;
    return 0;
}
</source>

Busy loops can also be used to poll hardware, although it is inefficient. This is sometimes the only option for some hardware that does not support interrupts (bless its heart).

[[Category:Anti-Pattern]]
````

<a id="w004519"></a>

## W004519 — User:Iguessthislldo

Bron-ID: W004519
Namespace: User (2)
Revisie: 25835 | Datum: 2021-05-05T18:01:56Z
Bron: https://wiki.osdev.org/User:Iguessthislldo
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AIguessthislldo&oldid=25835

````text
OS: [https://github.com/iguessthislldo/georgios Georgios]

WIP Pages:

* [[User:Iguessthislldo/libx86emu]]
* [[User:Iguessthislldo/Zig_Bare_Bones]]
````

<a id="w004520"></a>

## W004520 — User:Ajxs/Ada Bare Bones

Bron-ID: W004520
Namespace: User (2)
Revisie: 25111 | Datum: 2020-09-04T05:13:13Z
Bron: https://wiki.osdev.org/User:Ajxs/Ada_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAjxs%2FAda_Bare_Bones&oldid=25111

````text
''The aim of this tutorial is to produce a small Multiboot kernel written in the Ada programming language.''

==Preface==

One of the first things people ask on the Ada IRC channel on Freenode is ''"Can Ada be used for OS development?"'' to which the answer is a resounding '''yes'''.

=== Prerequisites ===

The following section details the prerequisite software necessary for successfully building the kernel.

==== Cross-Compiler ====

{{Main|GNAT Cross-Compiler}}

This tutorial requires the use of a [[GCC_Cross-Compiler|cross-compiler]] with Ada support targeting the <tt>i686-elf</tt> architecture. Theoretically, any Ada compiler capable of producing bare-metal ELF binaries for this architecture is suitable. The recommended compiler is [https://www.gnu.org/software/gnat/ GNAT], which is part of the GNU Compiler Collection maintained by the Free Software Foundation. Historically it was difficult to create cross-compiler builds for GNAT, but recent versions of GCC have made this much simpler. For a tutorial on how to set up a GNAT cross-compiler, refer to [[GNAT Cross-Compiler|this article]].

==== GPRbuild ====

This tutorial will require the use of the [https://docs.adacore.com/gprbuild-docs/html/gprbuild_ug.html GPRBuild] utility to build GNAT project files. GPRbuild is a generic build tool designed for the construction of multi-language systems. It is included with AdaCore's distribution of their proprietary GNAT compiler: [https://www.adacore.com/download] or can be built from source files obtained from AdaCore's [https://github.com/AdaCore/gprbuild github profile]. Older versions of GNAT distributed by the Free Software Foundation were able to build GNAT project files using the <tt>gnatmake</tt> utility, however newer versions do not support all the required features.

The cross-compiler mentioned above will need to be properly integrated with GPRbuild and its associated tools. This can be accomplished by following the guide detailed [[GNAT_Cross-Compiler#GPRbuild_Integration|here]].

== Ada run-time library ==
{{Main|Ada Runtime Library}}

The Ada run-time library is responsible for the implementation of the standard library as defined in the [http://ada-auth.org/standards/12rm/html/RM-TOC.html Ada Language Reference Manual]. Not all features outlined in the reference manual need to be implemented for every platform. Since the kernel will be running in a freestanding environment without the support of an underlying operating system, it will be built with what is known as a ''a zero-footprint run-time system'' (commonly abbreviated as 'ZFP'). A zero-footprint RTS will typically provide only the bare-minimum functionality for the target architecture.

The following steps will detail how to create an initial zero-footprint runtime library suitable for building the kernel.

The first step is to set up the directory structure to hold the runtime library. GNAT expects the runtime library to conform to the a set directory structure. It requires the presence of two directories: <tt>adalib</tt> and <tt>adainclude</tt>. <tt>adainclude</tt> contains the runtime package specifications and source files, which are used analogously to include files in C. <tt>adalib</tt> contains the compiled binary artifacts of the runtime library.

The following code demonstrates setting up a directory structure for the RTS:

<source lang="bash">
mkdir -p bare_bones/src/pc
cd bare_bones
mkdir -p rts/boards/i386/adalib
mkdir -p rts/boards/i386/adainclude
mkdir -p rts/src
mkdir -p obj
</source>

The best way to begin creating a new run-time library is to base it upon the existing source files obtained from the system GNAT installation, modifying them where necessary.

The following code demonstrates copying the required files from the host system's GNAT installation's run-time library into <tt>rts/src</tt> and then creating symbolic links to <tt>rts/boards/${arch}/adainclude</tt>. Where <tt>${arch}</tt> is <tt>i386</tt>, <tt>armv6</tt>, etc. The source location will need to be modified to reflect the location of the system compiler.

<source lang="bash">
system_compiler_dir="/usr/lib/gcc/x86_64-linux-gnu/8"
rts_dir="${PWD}/rts"

for f in "ada.ads" "a-unccon.ads" "a-uncdea.ads" "gnat.ads" "g-souinf.ads" \
"interfac.ads" "s-atacco.adb" "s-atacco.ads" "s-maccod.ads" "s-stoele.adb" \
"s-stoele.ads"
do
	cp "${system_compiler_dir}/adainclude/$f" "${rts_dir}/src/"
	ln -s "${rts_dir}/src/$f" "${rts_dir}/rts/boards/i386/adainclude/$f"
done
</source>

==Structure==
<overview of the structure of the repo>

==Runtime library==

===gnat.adc===

During compilation the GNAT compiler will search for a file named <tt>gnat.adc</tt> within the current working directory. This file is expected to contain one or more configuration pragmas that will be applied to the current compilation. These pragma directives instruct the compiler what features of the runtime should be restricted. This is of particular importance to kernel development since many of Ada's features are not supported by a freestanding, bare-metal environment. 

These pragma directives can alternatively be placed at the start of the runtime library's <tt>system.ads</tt> file (see below), however convention dictates using the <tt>gnat.adc</tt> file for this purpose.

Additionally, it is possible to apply additional global configuration files by appending the following line to the <tt>Builder</tt> package in a GNAT project file:

<source lang="ada">
   package Builder is
      --  ...
      for Global_Configuration_Pragmas use "kernel.adc";
   end Builder;
</source>

It is also possible to instruct the compiler to apply additional files containing configuration pragmas to the current compilation using the switch <tt>-gnatec=path</tt> on the command line. These configuration pragmas are applied in addition to those found in <tt>gnat.adc</tt>, if it is present.
More information on configuration files can be found in the GNAT documentation: [https://gcc.gnu.org/onlinedocs/gnat_ugn/Handling-of-Configuration-Pragmas.html]

The GNAT Reference Manual and the Ada Reference Manual provide information on the various configuration pragmas.
See below for a list of restriction pragmas useful for a bare bones kernel and runtime:

<source lang="ada">
pragma Discard_Names;
pragma Restrictions (No_Enumeration_Maps);
pragma Normalize_Scalars;
pragma Restrictions (No_Exception_Propagation);
pragma Restrictions (No_Finalization);
pragma Restrictions (No_Tasking);
pragma Restrictions (No_Protected_Types);
pragma Restrictions (No_Delay);
pragma Restrictions (No_Recursion);
pragma Restrictions (No_Allocators);
pragma Restrictions (No_Dispatch);
pragma Restrictions (No_Implicit_Dynamic_Code);
pragma Restrictions (No_Secondary_Stack);
</source>

'''Note:''' ''Do not use <tt>pragma No_Run_Time</tt>. It is obsolete.''

Below is an explanation of these configuration pragmas:

{| {{wikitable}}
! Restriction
! Purpose
|-
| Discard_Names || The Ada compiler automatically generates strings containing the names of enumerated data types, among others. These strings can be used in I/O.

<source lang="ada">
type Fruit is (Orange, Banana, Apple);
--  Ada defines the following strings, "Orange", "Banana" and "Apple" in an array.

--  These strings can be accessed using the 'Image attribute, as in
Put (Fruit'Image (Orange));
--  Prints "Orange" to the console.
</source>

This directive instructs the compiler not to generate these strings.
|-
| Normalize_Scalars || Forces all scalars to be initialised. Refer to [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/Pragma-Normalize_005fScalars.html#Pragma-Normalize_005fScalars GNAT RM:Normalize_Scalars] for more information.
|-
| No_Exception_Propagation || This directive forces the compiler to disallow any attempt to raise an exception over a subprogram boundary. Refer to [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/No_005fException_005fPropagation.html#No_005fException_005fPropagation GNAT RM:No_Exception_Propagation] for more information.

Note: The [http://docs.adacore.com/gnat-hie-docs/html/gnathie_ug_3.html#SEC8 GNAT High Integrity Edition] documentation states the following:

<blockquote>
Exception declarations and raise statements are still permitted under this restriction. A raise statement is compiled into a call of <tt>__gnat_last_chance_handler</tt>.
</blockquote>

All exceptions that are not handled with an explicit exception handler within its subprogram will be caught with the <tt>Last_Chance_Handler</tt> subprogram. This will cause a warning to be issued at compile time.
|-
| No_Exception_Registration || Ensures no stream operations are performed on types declared in Ada.Exceptions. See [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/No_005fException_005fRegistration.html#No_005fException_005fRegistration GNAT RM:No_Exception_Registration] for more information.
|-
| No_Finalization
| Restricts the use of controlled types. Refer to [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/No_005fFinalization.html#No_005fFinalization GNAT RM:No_Finalization] for more information.
|-
| No_Tasking
| This directive restricts all features related to tasking, including the use of protected objects. Refer to [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/No_005fTasking.html#No_005fTasking GNAT RM:No_Tasking] for more information.
|-
| No_Protected_Types
| This reinforces the above restriction. Refer to [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/No_005fProtected_005fTypes.html#No_005fProtected_005fTypes GNAT RM:No_Protected_Types] for more information.
|-
| No_Delay
| Restricts the use of <tt>delay</tt> statements or the calendar package. Refer to [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/No_005fDelay.html#No_005fDelay GNAT RM:No_Delay] for more information.
|-
| No_Recursion
| Restricts the use of recursion. Refer to [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/No_005fRecursion.html#No_005fRecursion GNAT RM:No_Recursion] for more information.
|-
| No_Allocators
| Restricts the use of dynamic memory. This is essential for a bare-metal application, since there is no underlying facility for allocation of dynamic memory. Refer to [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/No_005fAllocators.html#No_005fAllocators GNAT RM:No_Allocators] for more information.
|-
| No_Dispatch
| Disallows calling a subprogram using Ada's object-orientated mechanism. Refer to [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/No_005fDispatch.html#No_005fDispatch GNAT RM:No_Dispatch] for more information.
|-
| No_Implicit_Dynamic_Code
| Disallows nested subprograms or any other features that generate trampolines on the stack. Refer to [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/No_005fImplicit_005fDynamic_005fCode.html#No_005fImplicit_005fDynamic_005fCode GNAT RM:No_Implicit_Dynamic_Code] for more information.
|-
| No_Secondary_Stack
| Ada uses a ''secondary stack'' to return unconstrained types, such as arbitrary length strings or variant records. This directive instructs the compiler that there is no secondary stack present, and to restrict the use of code that requires one. Refer to [http://gcc.gnu.org/onlinedocs/gcc-4.7.0/gnat_rm/No_005fSecondary_005fStack.html#No_005fSecondary_005fStack GNAT RM:No_Secondary_Stack] for more information.
|}

Passing the <tt>-r</tt> flag to the binder instructs it to emit a list of further restrictions that are possible to apply to the project.
<source lang="ada">
   package Binder is
      for Default_Switches ("Ada") use ("-r");
   end Binder;
</source>

===system.ads===

Every Ada program must have access to the System package, this essentially tells the compiler what kind of hardware we are building for, therefore there will be 1 system.ads file per architecture your kernel supports.

Copy a system.ads from GCC that matches the target you are working with, in our case this is  gcc-<version>/gcc/ada/system-linux-x86.ads, name it system.ads and place it into rts/boards/i386/adainclude/ we need to edit this a bit.

We don't need to change anything from the top as all the integer sizes should be correct. Go to the private part of the spec and change the following values:

# Command_Line_Args => False
# Configurable_Run_Time => True
# Exit_Status_Supported => False
# Stack_Check_Probes => False
# Suppress_Standard_Library => True
# ZCX_By_Default => False
# GCC_ZCX_Support => False

For more information on these options, see gcc-<version>/gcc/ada/targparm.ads.

Also, add the following line in the private part of the package:

<source lang="ada">
private

   Run_Time_Name : constant String := "Bare Bones Run Time";
</source>

According to targparm.ads, it must be the first thing after the private keyword. It should also show up in error messages in parentheses, but I've not managed to get it to show up thus far. This is useful as it should show you which RTS you are currently using, just in case you configure your build incorrectly.

===Last chance handler===

The Ada runtime requires the presence of a <tt>Last_Chance_Handler</tt> subprogram. This subprogram is used as a handler for any exceptions that are not explicitly handled within their subprogram. These calls to the <tt>Last_Chance_Handler</tt> procedure in the case of unhandled exceptions will be automatically generated by the compiler.

In order to facilitate this the last chance handler procedure must be defined somewhere within the program. Typically this is defined within the runtime. The last chance handler procedure may have any name, however the compiler will search for a procedure with external linkage with the name <tt>__gnat_last_chance_handler</tt>. 

Create the following files in the <tt>rts/boards/${arch}/adainclude</tt>:

====last_chance_handler.ads====

<source lang="ada">
with System;

procedure Last_Chance_Handler
  (Source_Location : System.Address; Line : Integer);
pragma Export (C, Last_Chance_Handler, "__gnat_last_chance_handler");
</source>

====last_chance_handler.adb====

<source lang="ada">
procedure Last_Chance_Handler
  (Source_Location : System.Address; Line : Integer) is
   pragma Unreferenced (Source_Location, Line);
begin
   --  TODO: Add in board-specific code to dump exception information to serial/screen.
   loop
      null;
   end loop;
end Last_Chance_Handler;
</source>

The contents of the <tt>Last_Chance_Handler</tt> procedure will need to be tailored to the specific platform of the kernel. Typically this procedure will dump information regarding the exception to output media such as a [[Serial_Ports|serial port]].

===Compiling the runtime===

Create a file called <tt>gnat.gpr</tt> in the root directory with the following contents:

<source lang="ada">
library project gnat is
   type Arch_Name is ("i386", "arm");
   type Board_Name is ("pc", "rpi");

   Arch  : Arch_Name  := "i386";
   Board : Board_Name := external ("Board");

   case Board is
      when "pc" =>
         Arch := "i386";
      when "rpi" =>
         Arch  := "arm";
   end case;

   for Source_Dirs use ("rts/boards/" & Arch & "/adainclude");
   for Object_Dir use "obj"; --"rts/boards/" & Arch & "/adalib";

   package Builder is
      Basic_Switches := ("-gnat2005", "-g", "-x", "-a", "-gnatg",
			 "-gnatec=../gnat.adc");

      case Board is
         when "pc" =>
            for Default_Switches ("Ada") use Basic_Switches &
               ("-m32", "-march=i386");
         when "rpi" =>
            for Default_Switches ("Ada") use Basic_Switches &
               ("-march=armv6zk", "-mfpu=vfp", "-mfloat-abi=hard", "-marm",
                "-mcpu=arm1176jzf-s", "-mtune=arm1176jzf-s");
      end case;
   end Builder;

   package Compiler is
      for Default_Switches ("Ada") use ("-O2", "-ffunction-sections", "-fdata-sections");
   end Compiler;

   for Library_Kind use "static";
   for Library_Name use "gnat-4.6";
   for Library_Dir use "rts/boards/" & Arch & "/adalib";
end gnat;
</source>

Now compile with the following command:

<source lang="bash">
gprbuild -XBoard=pc -Pgnat.gpr
</source>

Inside <tt>rts/boards/i386/adainclude/</tt> the RTS sources should be symbolically linked along with the custom <tt>last_chance_hander</tt> and system files. Inside <tt>rts/boards/i386/adalib/</tt> there should be the <tt>libgnat-4.6.a</tt> file as well as <tt>*.ali</tt> matching the source files, which are required by GNAT.

''Note: Please note that the above lib might need it's name changed as some OSes might have built libgnat with a version number different to the one used here. The system libnat version may be <tt>libgnat-4.4.a</tt> and GNAT will expect to find that, so change the name in the GPR file accordingly.''

==Kernel==

===startup.s===

This is the main system bootstrapping code. This version is PC specific, so place this in the <tt>src/pc</tt> directory.

''Note: This source file is written using GAS syntax.''

<source lang="asm">
.global startup                         # Make the startup entry point symbol visible to the linker

# Set up the Multiboot header (see GRUB docs for details)
.set ALIGN,    1<<0                     # Align loaded modules on page boundaries
.set MEMINFO,  1<<1                     # Provide memory map
.set FLAGS,    ALIGN | MEMINFO          # This is the Multiboot 'flag' field
.set MAGIC,    0x1BADB002               # 'magic number' lets your bootloader find the header
.set CHECKSUM, -(MAGIC + FLAGS)         # Checksum required

header:                                 # Must be in the first 8kb of the kernel file
.align 4, 0x90                          # Pad the location counter (in the current subsection) to a 4-byte
                                        # storage boundary. The way alignment is specified can vary with
                                        # host system architecture.
.long MAGIC
.long FLAGS
.long CHECKSUM

# Reserve initial kernel stack space
.set STACKSIZE, 0x4000                  # 0x4000 being 16k.
.lcomm stack, STACKSIZE                 # Reserve 16k stack on a32-bit boundary
.comm  mbd, 4                           # Declare common symbol mbd, allocate it 4-bytes of
                                        # uninitialized memory.
.comm  magic, 4                         # Declare common symbol magic, allocate it 4-bytes of
                                        # uninitialized memory.

startup:
    movl  $(stack + STACKSIZE), %esp    # Set up the stack

# The following saves the contents of the registers as they will likely be
# overwritten because main is not our actual entry point, Bare_Bones is. We
# will use these 2 symbols inside Bare_Bones.

    movl  %eax, magic                   # EAX indicates to the OS that it was loaded by a Multiboot-compliant boot
                                        # loader
    movl  %ebx, mbd                     # EBX must contain the physical address of the Multiboot information
                                        # structure

    call  main                          # Call main (created by gnatbind)

    cli                                 # Disable interrupts. then intentionally hang the system.
                                        # CLI only affects the interrupt flag for the processor on which it is
                                        # executed.

hang:
    hlt                                 # Because the HLT instruction halts until an interrupt occurs, the
                                        # combination of a CLI followed by a HLT is used to intentionally hang
                                        # the computer if the kernel returns.
                                        # HLT is in a loop just in case a single HLT instruction fails to execute
                                        # for some reason, (such as in the case of an NMI).
    jmp   hang
</source>

===multiboot.ads===

I won't show the source to this package as  I have made it more Ada-like and it's quite large, so I will link to the current versions, [https://github.com/Lucretia/bare_bones/blob/master/src/pc/multiboot.ads multiboot.ads] and [https://github.com/Lucretia/bare_bones/blob/master/src/pc/multiboot.adb multiboot.adb].

===Console===

'''Disclaimer:''' I wrote this package a long time ago and have reformatted it using my current Ada programming style. I have not gone too far into the code, so it may not be the best implementation of an console.

The following 2 files give you access to the VGA console at 80x25 characters. As they are PC specific, they go into the src/pc directory.

====console.ads====

<source lang="ada">
with System;

package Console is
   pragma Preelaborate (Console);

   type Background_Colour is
     (Black,
      Blue,
      Green,
      Cyan,
      Red,
      Magenta,
      Brown,
      Light_Grey);

   for Background_Colour use
     (Black      => 16#0#,
      Blue       => 16#1#,
      Green      => 16#2#,
      Cyan       => 16#3#,
      Red        => 16#4#,
      Magenta    => 16#5#,
      Brown      => 16#6#,
      Light_Grey => 16#7#);

   for Background_Colour'Size use 4;

   type Foreground_Colour is
     (Black,
      Blue,
      Green,
      Cyan,
      Red,
      Magenta,
      Brown,
      Light_Grey,
      Dark_Grey,
      Light_Blue,
      Light_Green,
      Light_Cyan,
      Light_Red,
      Light_Magenta,
      Yellow,
      White);

   for Foreground_Colour use
     (Black         => 16#0#,
      Blue          => 16#1#,
      Green         => 16#2#,
      Cyan          => 16#3#,
      Red           => 16#4#,
      Magenta       => 16#5#,
      Brown         => 16#6#,
      Light_Grey    => 16#7#,
      Dark_Grey     => 16#8#,
      Light_Blue    => 16#9#,
      Light_Green   => 16#A#,
      Light_Cyan    => 16#B#,
      Light_Red     => 16#C#,
      Light_Magenta => 16#D#,
      Yellow        => 16#E#,
      White         => 16#F#);

   for Foreground_Colour'Size use 4;

   type Cell_Colour is
      record
         Foreground : Foreground_Colour;
         Background : Background_Colour;
      end record;

   for Cell_Colour use
      record
         Foreground at 0 range 0 .. 3;
         Background at 0 range 4 .. 7;
      end record;

   for Cell_Colour'Size use 8;

   type Cell is
      record
         Char   : Character;
         Colour : Cell_Colour;
      end record;

   for Cell'Size use 16;

   Screen_Width  : constant Natural := 80;
   Screen_Height : constant Natural := 25;

   subtype Screen_Width_Range  is Natural range 1 .. Screen_Width;
   subtype Screen_Height_Range is Natural range 1 .. Screen_Height;

   type Row    is array (Screen_Width_Range)  of Cell;
   type Screen is array (Screen_Height_Range) of Row;

   Video_Memory : Screen;

   for Video_Memory'Address use System'To_Address (16#000B_8000#);

   pragma Import (Ada, Video_Memory);

   procedure Put
     (Char       : in Character;
      X          : in Screen_Width_Range;
      Y          : in Screen_Height_Range;
      Foreground : in Foreground_Colour := White;
      Background : in Background_Colour := Black);

   procedure Put
     (Str        : in String;
      X          : in Screen_Width_Range;
      Y          : in Screen_Height_Range;
      Foreground : in Foreground_Colour := White;
      Background : in Background_Colour := Black);

   procedure Clear (Background : in Background_Colour := Black);
end Console;
</source>

====console.adb====

<source lang="ada">
package body Console is
   procedure Put
     (Char       : in Character;
      X          : in Screen_Width_Range;
      Y          : in Screen_Height_Range;
      Foreground : in Foreground_Colour := White;
      Background : in Background_Colour := Black) is
   begin
      Video_Memory (Y)(X).Char              := Char;
      Video_Memory (Y)(X).Colour.Foreground := Foreground;
      Video_Memory (Y)(X).Colour.Background := Background;
   end Put;

   procedure Put
      (Str        : in String;
       X          : in Screen_Width_Range;
       Y          : in Screen_Height_Range;
       Foreground : in Foreground_Colour := White;
       Background : in Background_Colour := Black) is
   begin
      for Index in Str'First .. Str'Last loop
         Put (Str (Index),
              X + Screen_Width_Range (Index) - 1,
              Y,
              Foreground,
              Background);
      end loop;
   end Put;

   procedure Clear (Background : in Background_Colour := Black) is
   begin
      for X in Screen_Width_Range'First .. Screen_Width_Range'Last loop
         for Y in Screen_Height_Range'First .. Screen_Height_Range'Last loop
            Put (' ', X, Y, Background => Background);
         end loop;
      end loop;
   end Clear;
end Console;
</source>

===bare_bones.adb===

This is platform independent and therefore goes into the src directory.

<source lang="ada">
with Console; use Console;

procedure Bare_Bones is
begin
   Clear;

   Put ("Hello, bare bones in Ada.",
        Screen_Width_Range'First,
        Screen_Height_Range'First);
end Bare_Bones;
pragma No_Return (Bare_Bones);
</source>

===linker.ld===

This is a PC specific script so goes into the src/pc directory.

<pre>
OUTPUT_FORMAT(elf32-i386)

/* Tell the linker which startup code to use, we do this as there is no way to do this (well not easily) from the GNAT tools. */
STARTUP(startup.o)

ENTRY (startup)

SECTIONS
{
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
</pre>

===makefile===

Place this file in the root directory.

<source lang="make">
ARCH		=	i386
RTS_DIR		=	`pwd`/rts/boards/$(ARCH)

ifeq ($(ARCH),i386)
GPRBUILD	=	gprbuild
AS		=	as
ASFLAGS		=	--32 -march=i386

OBJS		=	obj/startup.o obj/multiboot.o obj/console.o
BOARD		=	pc

.PHONY: obj/multiboot.o obj/console.o

endif

all: bare_bones

bare_bones: $(OBJS) src/bare_bones.adb
	$(GPRBUILD) --RTS=$(RTS_DIR) -XBoard=$(BOARD) -Pbare_bones.gpr

obj/startup.o: src/$(BOARD)/startup.s
	$(AS) $(ASFLAGS) src/$(BOARD)/startup.s -o obj/startup.o

.PHONY: clean

clean:
	-rm obj/* *~ bare_bones
</source>

===bare_bones.gpr===

Place this file in the root directory.

<source lang="ada">
project Bare_Bones is
   type Arch_Name is ("i386", "arm");
   type Board_Name is ("pc", "rpi");

   Arch  : Arch_Name  := "i386";
   Board : Board_Name := external ("Board");

   -- TODO: Add in a case statement that adds an arch dir to source.

   case Board is
      when "pc" =>
         for Source_Dirs use ("src", "src/pc");
      when "rpi" =>
         for Source_Dirs use ("src", "src/rpi");
   end case;

   for Object_Dir use "obj";
   for Exec_Dir use ".";
   for Main use ("bare_bones.adb");

   package Builder is
      Basic_Switches := ("-gnat2005", "-g", "-x", "-a", "-gnatg",
                         "-gnatec=../gnat.adc", "-gnaty-I", "-gnaty+d");

      case Board is
         when "pc" =>
            for Default_Switches ("Ada") use Basic_Switches &
               ("-m32", "-march=i386");
         when "rpi" =>
            for Default_Switches ("Ada") use Basic_Switches &
               ("-march=armv6zk", "-mfpu=vfp", "-mfloat-abi=hard", "-marm",
                "-mcpu=arm1176jzf-s", "-mtune=arm1176jzf-s");
      end case;
   end Builder;

   package Compiler is
      case Board is
         when "pc" =>
            for Default_Switches ("Ada") use
               ("-O0", "-g", "-ggdb", "-ffunction-sections", "-fdata-sections");
         when "rpi" =>
            for Default_Switches ("Ada") use
               ("-O0", "-g", "-ggdb", "-ffunction-sections", "-fdata-sections");

      end case;
   end Compiler;

-- To reduce size of final binary.
   package Linker is
      for Default_Switches ("Ada") use
         ("-Wl,--gc-sections", "-static", "-nostartfiles", "-nodefaultlibs",
          "-T../src/" & Board & "/linker.ld", "-v");
   end Linker;
end Bare_Bones;
</source>

==Testing==

Make sure you have built the RTS above before this next stage otherwise you won't be able to compile the kernel.

<source lang="bash">
make qemu
</source>

On the QEMU window, it should clear the screen, the the cursor won't move so it will be in the middle of the screen, in the top-left corner will be the message "Hello, bare bones in Ada."


==Next Steps==

A useful next step for further developing the bare bones kernel outlined in this article is implementing capability for using the <tt>'Image</tt> attributes on scalar types. This facilitates the printing of integers in string form, which is extremely useful for kernel debugging. A simple guide on how to accomplish this is detailed [[Ada_Runtime_Library#Image_attributes|here]].

[[Category:Bare bones tutorials]]
[[Category:Ada]]
````

<a id="w004521"></a>

## W004521 — SCSI

Bron-ID: W004521
Namespace: Main (0)
Revisie: 28359 | Datum: 2023-10-03T09:56:53Z
Bron: https://wiki.osdev.org/SCSI
Revisie-URL: https://wiki.osdev.org/index.php?title=SCSI&oldid=28359

````text
SCSI stands for "Small Computer Systems Interface" though the original name is rarely used in favor of just calling it SCSI (pronounced: scuzzy).

== Introduction ==
SCSI is a collection of standards defining a hardware interface, at the electrical level, an addressing scheme, at the logical level, and the protocol to be used, by software, to communicate with and transfer data between computers and peripheral devices. The term is also used as a prefix for the related hardware attached via the such as "SCSI hard drive" or "SCSI tape drive" or "SCSI controller"  and accessories, as in "SCSI cable" or "SCSI port".

It was, and to an extent still is, found as an interface type for a wide variety of hardware. Such hardware commonly included both hard disks and tape drives (especial internal devices), but was also used for a variety of other peripherals including floppy drives, CD-ROM drives (and other optical media), printers, scanners, cameras, and more. A particularly unusual example would be a SCSI network adapter.

== Standards ==

{| class="wikitable"
|SCSI-1
|the older, and nominally original, version often found on computers from the late 80s through the 90s.
|-
|SCSI-2
|a newer more modern version which encompassed two variants referred to as '''Fast SCSI''' and '''Wide SCSI'''.
|-
|SCSI-3
|rowspan="2"|a collection of standards describing various enhancements to SCSI. It was first introduced, as a term, to describe devices<br/>which transcended the limitations, particularly speed, of earlier SCSI-2 devices.
|}

Various later changes are labeled as 'Ultra' and some specifier such as a version of the max speed.

== Physical Attachment ==

=== Interfaces ===
SCSI comes in two forms, the older parallel and the newer serial. In each case the commands and protocols are essentially similar above the hardware level.

'''Parallel SCSI''', or just SCSI, refers to use of a parallel data bus (multiple bits sent/received simultaneously) to transmit commands and data. It is very common for such buses to use a large number of separate lines for individual bits, often matching the native word length (or a multiple) used in the system the SCSI hardware is installed in. As such there has historically been both 8-bit SCSI and 16-bit SCSI.

'''Serial Attached SCSI''', aka SAS, transmits data over a serial bus and benefits from the nature of serial communications in the form of speed and increased reliability.

=== Cabling ===
A number of different cabling systems have been used, some of which still are, including a manifestation very similar to the PC Parallel Port using a DB-25 (25 pin) connector which was common on Apple Macintosh computers for a long time. Other examples include Serial Attached SCSI (SAS) which passes over a [[SATA]] data cable and iSCSI which is "internet enabled" and can be passed over any physical transport supporting IP (internet protocol).

== See Also ==

=== External Links ===
*[https://en.wikipedia.org/wiki/SCSI Wikipedia page (SCSI)]
*[https://en.wikipedia.org/wiki/Parallel_SCSI Wikipedia page (Parallel SCSI)]
*[https://www.seagate.com/files/staticfiles/support/docs/manual/Interface%20manuals/100293068j.pdf Seagate SCSI Interface Manual]

[[Category:Hardware Interfaces]]
````

<a id="w004522"></a>

## W004522 — User:Silverfox1955

Bron-ID: W004522
Namespace: User (2)
Revisie: 25124 | Datum: 2020-09-05T15:22:23Z
Bron: https://wiki.osdev.org/User:Silverfox1955
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASilverfox1955&oldid=25124

````text
Wiki pages that I have largely contributed to.

* [[Ext4]]
````

<a id="w004523"></a>

## W004523 — Jbd2

Bron-ID: W004523
Namespace: Main (0)
Revisie: 27954 | Datum: 2023-07-09T16:08:12Z
Bron: https://wiki.osdev.org/Jbd2
Revisie-URL: https://wiki.osdev.org/index.php?title=Jbd2&oldid=27954

````text
{{Stub}}

Jbd2 is a journaling system used in [[Ext3]], [[Ext4]], and [[OCFS2]]. This system allows for changes to the filesystem to be reapplied based on the transaction state. See [[Journaling]] for a generic description of filesystem journaling.

[[Category:Filesystems]]
````

<a id="w004524"></a>

## W004524 — User:Unprotectedmode

Bron-ID: W004524
Namespace: User (2)
Revisie: 25137 | Datum: 2020-09-08T01:32:10Z
Bron: https://wiki.osdev.org/User:Unprotectedmode
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AUnprotectedmode&oldid=25137

````text
This is my userpage.

I've done some assembly in both 8080/ALTAIR emulation, and x86. I'd like to write a modern x86-64 OS as well as a retro-aimed DOS-type OS that multitasks in v8086 mode and emulates modern hardware to the older setups (e.g. USB HIDs as keyboard, PS/2 mouse, and joystick port)
````

<a id="w004525"></a>

## W004525 — User:Narke/Ring0OSPitfalls

Bron-ID: W004525
Namespace: User (2)
Revisie: 25149 | Datum: 2020-09-10T22:17:08Z
Bron: https://wiki.osdev.org/User:Narke/Ring0OSPitfalls
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANarke%2FRing0OSPitfalls&oldid=25149

````text
Pitfalls of a ring0 only OS are:
* Software's crash will crash the kernel, think of a pointer's misuse/segfault, memory safe languages aren't concerned by this.
* Other user's RAM can be accessed to dump the memory and see their data (passwords)
* Easy to rootkit by hijacking syscalls
````

<a id="w004526"></a>

## W004526 — UEFI Bare Bones

Bron-ID: W004526
Namespace: Main (0)
Revisie: 25170 | Datum: 2020-09-17T15:41:39Z
Bron: https://wiki.osdev.org/UEFI_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=UEFI_Bare_Bones&oldid=25170
Doorverwijzing: UEFI App Bare Bones

````text
#REDIRECT [[UEFI App Bare Bones]]
````

<a id="w004527"></a>

## W004527 — Talk:UEFI Bare Bones

Bron-ID: W004527
Namespace: Talk (1)
Revisie: 25172 | Datum: 2020-09-17T15:41:39Z
Bron: https://wiki.osdev.org/Talk:UEFI_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AUEFI_Bare_Bones&oldid=25172
Doorverwijzing: Talk:UEFI App Bare Bones

````text
#REDIRECT [[Talk:UEFI App Bare Bones]]
````

<a id="w004528"></a>

## W004528 — User:Nexos/UEFI Bare Bones

Bron-ID: W004528
Namespace: User (2)
Revisie: 25179 | Datum: 2020-09-17T16:00:21Z
Bron: https://wiki.osdev.org/User:Nexos/UEFI_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANexos%2FUEFI_Bare_Bones&oldid=25179

````text
{{BeginnersWarning}}
{{Rating|1}}

Welcome to the long and fun road of OS Development! This page will help you get started by writing a [[UEFI]] [[kernel]] bootable by [[GRUB]] and [[BOOTBOOT]] for x86_64.

Before getting started, it may be a good idea to look at [[Bare Bones]].

Also, look at the required knowledge. Get a copy of the Intel Manuals. If you have trouble, question cane be asked on the forum or #osdev at freenode. You will need a lot of knowledge to succeed. OSDev takes time and patience. You will also need a firm knowledge of [[C]].

== Prerequisites ==
To get started, you should set up a [[GCC Cross Compiler]]. You will also need [[NASM]] as well.

If you want to use Windows, these steps can be completed in [[Cygwin]] or [[MinGW]].

Once you have completed this, you can get started!

== Getting started ==
To start an operating system, a [[bootloader]] is needed. You make your own, but for now we will use GRUB. Also, BOOTBOOT will be shown as well.
````

<a id="w004529"></a>

## W004529 — File:LemonOS.png

Bron-ID: W004529
Namespace: File (6)
Revisie: 25737 | Datum: 2021-03-05T22:20:03Z
Bron: https://wiki.osdev.org/File:LemonOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ALemonOS.png&oldid=25737

````text
Lemon OS
````

<a id="w004530"></a>

## W004530 — Limine

Bron-ID: W004530
Namespace: Main (0)
Revisie: 30493 | Datum: 2026-04-18T12:40:08Z
Bron: https://wiki.osdev.org/Limine
Revisie-URL: https://wiki.osdev.org/index.php?title=Limine&oldid=30493

````text
'''Limine''' is an advanced, portable, multiprotocol bootloader, with support for Linux, the [[Limine Bare Bones|native Limine boot protocol]], and [[Multiboot|multiboot1 and 2]].

== History ==

Limine was created as the reference implementation for the '''stivale''' boot protocols. The protocols were conceived as a response to the shortcomings of [[Multiboot]]. As of Limine 3.x, a new boot protocol, simply called the [[Limine Bare Bones|Limine boot protocol]] was introduced to supersede stivale 1 and 2. As of Limine 4.x, support for the legacy stivale protocols has been dropped.

It originally only supported x86 [[BIOS]] and the stivale1 protocol, but it was later expanded to also support [[UEFI]], the Linux and [[multiboot]] protocols, and other architectures such as aarch64.

== Supported protocols and filesystems ==

As mentioned above, Limine supports the [[Limine Bare Bones|Limine]] boot protocol, alongside Linux's own boot protocol (which means one can boot Linux fully using Limine), multiboot 1 and 2 (allowing it to boot a vast catalogue of hobby OSes and more), and chainloading to allow to indirectly boot unsupported operating systems such as Microsoft Windows.

It supports the [[FAT|FAT12/16/32]] and [[ISO 9660]] (used by optical media and hybrid ISO images) filesystems.

== Supported Architectures ==

Currently Limine supports 32-bit [[X86]] (bootloader only, not protocol), [[X86-64]], [[AArch64|aarch64]] (64 bit [[ARM]]), riscv64 (64 bit [[RISC-V]]), and loongarch64. 32-bit X86 support is targeted for Pentium Pro (i686) class machines as the minimum. 

== How to use Limine with your kernel ==

The [[Limine Bare Bones]] article contains a basic tutorial on how to use Limine and its protocol. Furthermore, a Git repository containing a simple example template for a portable 64-bit kernel loaded using the Limine boot protocol can be found in the external links section.

== See Also ==

=== Articles ===

* [[Limine Bare Bones]]

== External Links ==
*[https://github.com/Limine-Bootloader/limine-c-template Barebones kernel template using Limine in C.]
*[https://github.com/Limine-Bootloader/Limine Limine source code on Codeberg.]
*[https://github.com/Limine-Bootloader/limine-protocol/blob/trunk/PROTOCOL.md Limine boot protocol specification.]

[[Category:Bootloaders]]
````

<a id="w004531"></a>

## W004531 — GNU-EFI

Bron-ID: W004531
Namespace: Main (0)
Revisie: 30353 | Datum: 2026-03-18T19:39:41Z
Bron: https://wiki.osdev.org/GNU-EFI
Revisie-URL: https://wiki.osdev.org/index.php?title=GNU-EFI&oldid=30353

````text
[https://github.com/ncroxon/gnu-efi GNU-EFI] is a very lightweight developing environment to create UEFI applications. [[EDK2]] is a large, complex environment with its own build system. GNU-EFI on the other hand is a set of libraries and headers for compiling UEFI applications with a system's native GCC.

This tutorial will guide you through to create a simple Hello World UEFI application using this library.

== Basic Concept ==

The main concept is to use your host native compiler, then convert the resulting [[ELF]] into UEFI-compatible [[PE]]. This means you can use your native toolchain, and only one additional step is required.

Of course you can also use [[GCC Cross-Compiler]] generating PE directly as described in [[UEFI App Bare Bones]]. Whichever you prefer.

== Requirements ==

Download and compile [https://github.com/ncroxon/gnu-efi GNU-EFI].
<syntaxhighlight lang="bash">
$ git clone https://github.com/ncroxon/gnu-efi.git
$ cd gnu-efi
$ make
</syntaxhighlight>

=== Libraries ===
This "make" should create the following three object files:
* '''crt0-efi-x86_64.o''': A CRT0 (C runtime initialization code) that will call the "efi_main" function.
* '''libgnuefi.a''': A library containing a single function (''_relocate'') that is used by the CRT0.
* '''libefi.a''': A library containing convenience functions like CRC computation, string length calculation, and easy text printing.

Strictly speaking only the first two are needed, but it worth having the third.

=== Headers ===

You'll need some header files. You can use the ones shipped with your Linux distribution (/usr/include/efi), this is the preferable as they are updated to the latest. Then you can use the ones in the [[EDK2]] package, or the minimal headers in [https://github.com/ncroxon/gnu-efi/tree/master/inc GNU-EFI inc]. This latter '''inc''' directory is installed with the git clone command, however it is just a minimal set of headers, only the essentials and not necessarily up-to-date.

=== Linker Script ===

This is pretty simple, we'll just use [https://github.com/ncroxon/gnu-efi/blob/master/gnuefi/elf_x86_64_efi.lds elf_x86_64_efi.lds] that GNU-EFI provides.

== Creating an EFI executable ==

The traditional "Hello, world" UEFI program is shown below, which we will compile using GNU-EFI in this tutorial.

'''main.c''':
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

=== Compilation ===

To compile the main.c above, you'll have to use some environment specific command line options. These are:
<syntaxhighlight lang="bash">
$ gcc -Ignu-efi-dir/inc -fpic -ffreestanding -fno-stack-protector -fno-stack-check -fshort-wchar -mno-red-zone -maccumulate-outgoing-args -c main.c -o main.o
</syntaxhighlight>
* '''-I''': set this to the efi headers directory (see above)
* '''-fpic''': the UEFI PE executables must be relocatable
* '''-ffreestanding''': there's no hosted gcc environment, we don't have libc
* '''-fno-stack-protector -fno-stack-check -mno-red-zone''': stack must be strictly used, no additional canaries or pre-allocated local variable space allowed
* '''-fshort-wchar''': it is very important that UEFI uses 16 bit characters (wide-characters or wchar_t, defined as CHAR16 in efi headers)
* '''-maccumulate-outgoing-args''': function calls must include the number of arguments passed to the functions

Replace 'gnu-efi-dir' with your repository's path.

=== Linking ===

It is very simple, but there is a trick to it. To generate properly relocatable code, we should link it as a shared library.
<syntaxhighlight lang="bash">
$ ld -shared -Bsymbolic -Lgnu-efi-dir/x86_64/lib -Lgnu-efi-dir/x86_64/gnuefi -Tgnu-efi-dir/gnuefi/elf_x86_64_efi.lds gnu-efi-dir/x86_64/gnuefi/crt0-efi-x86_64.o main.o -o main.so -lgnuefi -lefi
</syntaxhighlight>
* '''-shared -Bsymbolic''': tell GNU ld to create so
* '''-L''' and '''-T''': where to find the static GNU-EFI libraries (.a) and the linker script
* '''.o''': it is important to specify crt0 as the first. Should work as the last too, but I had problems
* '''-l''': linking with gnuefi is a must, as that contains the relocation code. Linking with efi is optional, but recommended

Replace 'gnu-efi-dir' with your repository's path.

=== Converting Shared Object to EFI executable ===

The last step is to convert our ELF shared object into an UEFI PE.
<syntaxhighlight lang="bash">
$ objcopy -j .text -j .sdata -j .data -j .rodata -j .dynamic -j .dynsym  -j .rel -j .rela -j .rel.* -j .rela.* -j .reloc --output-target efi-app-x86_64 --subsystem=10 main.so main.efi
</syntaxhighlight>
* '''-j''': these specify which sections to keep during convertion
* '''--output-target efi-app-x86_64''': this tells objcopy to generate a PE32+ format, with architecture code 0x8664
* '''--subsystem=10''': this is the most important. It sets the file's type to UEFI executable in the PE header.

Now you can copy '''main.efi''' to your [[EFI System Partition]], and after boot run it from the EFI Shell. Or you can rename it to '''EFI\BOOT\BOOTX64.EFI''' and it should be executed automatically on boot.

== Calling Any Arbitrary UEFI Function ==

The '''libefi.a''' has wrappers for the most common UEFI functions, but you might need to call something not covered. For completeness, it provides:
<syntaxhighlight lang="c">
uefi_call_wrapper(func, numarg, ...);
</syntaxhighlight>
For example, the "Print" function used in our main.c and which accepts printf compatible arguments, is under the hood nothing else than a call to:
<syntaxhighlight lang="c">
uefi_call_wrapper(ST->ConOut->OutputString, 2, ST->ConOut, buffer);
</syntaxhighlight>
The biggest advantage of 'uefi_call_wrapper' is that doesn't matter what ABI your gcc is using, it will always correctly translate that into UEFI ABI. If, and only if you've used the correct gcc options, then you should be able to make the same call as:
<syntaxhighlight lang="c">
ST->ConOut->OutputString(ST->ConOut, buffer);
</syntaxhighlight>
It worth noting that GNU-EFI uses the ''host'' compiler, therefore you might need additional gcc options to get this working. Using 'uefi_call_wrapper' on the other hand will always work.

== See also ==
=== Articles ===
* [[UEFI]]
* [[UEFI Bare Bones]]
* [[EFI System Partition]]

== External Links ==
* [http://www.uefi.org/specifications UEFI specifications et al.]
* [https://github.com/ncroxon/gnu-efi GNU-EFI source]

[[Category:x86-64]]
[[Category:UEFI]]
````

<a id="w004532"></a>

## W004532 — SD card

Bron-ID: W004532
Namespace: Main (0)
Revisie: 28082 | Datum: 2023-07-09T21:06:13Z
Bron: https://wiki.osdev.org/SD_card
Revisie-URL: https://wiki.osdev.org/index.php?title=SD_card&oldid=28082

````text
SD - Secure Digital is memory card that is used in many devices like mobile phones or cameras.

== Detecting SD card ==
SD card can be founded on PCI device with class 0x08 and subclass 0x05. ProgIF 0x00 means DMA isn't supported, 0x01 means DMA is supported and 0x02 means vendor unique SD device. SD card using memory mapped ports that base address is in BAR0.

== See Also ==
https://openlabpro.com/guide/interfacing-microcontrollers-with-sd-card/

[[Category:Storage]]
````

<a id="w004533"></a>

## W004533 — User talk:Himself65

Bron-ID: W004533
Namespace: User talk (3)
Revisie: 25249 | Datum: 2020-10-12T12:46:22Z
Bron: https://wiki.osdev.org/User_talk:Himself65
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AHimself65&oldid=25249

````text
* [https://www.github.com/Himself65 GitHub]
````

<a id="w004534"></a>

## W004534 — Dizz Tracked

Bron-ID: W004534
Namespace: Main (0)
Revisie: 29743 | Datum: 2025-11-01T23:54:14Z
Bron: https://wiki.osdev.org/Dizz_Tracked
Revisie-URL: https://wiki.osdev.org/index.php?title=Dizz_Tracked&oldid=29743

````text
Dizz gets distracted by unneccessary side projects. Be it a compiler, a linker, a game, a browser etc. 

== Pro's & Con's ==
Pro:
* never feels bored.
Con:
* wastes a lot of time and work.

== Going further than Dizz Tracked ==
Make a good roadmap.

== Dizz Tracked's bookshelf ==
Book about compiler creation, game development, software rendering, [https://craftinginterpreters.com/ "Crafting Interpreters"] etc.

== People and OSes related to Dizz Tracked ==
* [[User:PeterX|PeterX]] That's kind of me. But I'm trying to improve.
* [[User:eekee|eekee]] - I actually need to be this, for now. I've got better at planning only to realise that I ''[[Dae Dreamer|still]]'' need more practice with my chosen languages.
* [[User:Partisani|partisani]]: This page describes me perfectly.
* [[User:Demindiro|Demindiro]]: "compiler" check, "game development" check, "software rendering" check, "interpreters" check ... oh dear. Well, I haven't made a browser or linker. Yet.

[[Category:Developer Archetype]]
````

<a id="w004535"></a>

## W004535 — File:SkiftOS.png

Bron-ID: W004535
Namespace: File (6)
Revisie: 25276 | Datum: 2020-10-29T11:53:27Z
Bron: https://wiki.osdev.org/File:SkiftOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ASkiftOS.png&oldid=25276

````text

````

<a id="w004536"></a>

## W004536 — Linker Script

Bron-ID: W004536
Namespace: Main (0)
Revisie: 25280 | Datum: 2020-10-29T21:06:17Z
Bron: https://wiki.osdev.org/Linker_Script
Revisie-URL: https://wiki.osdev.org/index.php?title=Linker_Script&oldid=25280
Doorverwijzing: Linker Scripts

````text
#REDIRECT [[Linker Scripts]]
````

<a id="w004537"></a>

## W004537 — Loading files under UEFI

Bron-ID: W004537
Namespace: Main (0)
Revisie: 30859 | Datum: 2026-07-07T17:24:59Z
Bron: https://wiki.osdev.org/Loading_files_under_UEFI
Revisie-URL: https://wiki.osdev.org/index.php?title=Loading_files_under_UEFI&oldid=30859

````text
[[UEFI]] is supposed to provide an easy way of loading files from partitions. Sadly it is not so easy, considerably more complicated than [[Reading_sectors_under_UEFI|reading sectors]], but at least it parses the file system for you.

== Volume Handle ==

There's no common "Open File" function in UEFI. Instead each volume has its own. So the first thing you need to do is locate a volume handle. You probably want to use the same file system that your application was loaded from, so you need to use the image handle provided to your '''efi_main'''. Then you need to use the EFI_LOADED_IMAGE_PROTOCOL to figure out the device your application resides on.
<syntaxhighlight lang="c">
EFI_FILE_HANDLE GetVolume(EFI_HANDLE image)
{
  EFI_LOADED_IMAGE *loaded_image = NULL;                  /* image interface */
  EFI_GUID lipGuid = EFI_LOADED_IMAGE_PROTOCOL_GUID;      /* image interface GUID */
  EFI_SIMPLE_FILE_SYSTEM_PROTOCOL *IOVolume;                        /* file system interface */
  EFI_GUID fsGuid = EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_GUID; /* file system interface GUID */
  EFI_FILE_HANDLE Volume;                                 /* the volume's interface */

  /* get the loaded image protocol interface for our "image" */
  uefi_call_wrapper(BS->HandleProtocol, 3, image, &lipGuid, (void **) &loaded_image);
  /* get the volume handle */
  uefi_call_wrapper(BS->HandleProtocol, 3, loaded_image->DeviceHandle, &fsGuid, (VOID*)&IOVolume);
  uefi_call_wrapper(IOVolume->OpenVolume, 2, IOVolume, &Volume);
  return Volume;
}

/**
 * This is your application's main entry point, which receives its image handle
 */
int efi_main(EFI_HANDLE image, EFI_SYSTEM_TABLE *systab)
{
  EFI_FILE_HANDLE Volume = GetVolume(image);

  /* ... */
}

</syntaxhighlight>

Here [[GNU-EFI]]'s libefi.a provides a '''LibOpenRoot''' function, which can be used instead of IOVolume:
<syntaxhighlight lang="c">
/* get the volume handle */
return LibOpenRoot(loaded_image->DeviceHandle);
</syntaxhighlight>

== Read Data from File ==

Now that we have a '''Volume''' instance, it's rather easy to use it. It has the classic Open / Read / Close abstraction.

=== Open ===
<syntaxhighlight lang="c">
CHAR16              *FileName = L"DATA.TXT";
EFI_FILE_HANDLE     FileHandle;

/* open the file */
uefi_call_wrapper(Volume->Open, 5, Volume, &FileHandle, FileName, EFI_FILE_MODE_READ, EFI_FILE_READ_ONLY | EFI_FILE_HIDDEN | EFI_FILE_SYSTEM);
</syntaxhighlight>

Note that the file name passed to '''Volume->Open''' is a UNICODE16 string, meaning all characters are 2 bytes long. To get the L"" string literal correctly, you'll need to pass the "-fshort-wchar" command line option to gcc.

=== Read ===
<syntaxhighlight lang="c">
/* read from the file */
UINT64              ReadSize = FileSize(FileHandle);
UINT8               *Buffer = AllocatePool(ReadSize);

uefi_call_wrapper(FileHandle->Read, 3, FileHandle, &ReadSize, Buffer);
</syntaxhighlight>

=== Close ===
<syntaxhighlight lang="c">
/* close the file */
uefi_call_wrapper(FileHandle->Close, 1, FileHandle);
</syntaxhighlight>

=== Get File Size ===
In order to know how much to read ('''ReadSize''' variable above), you'll need to know the file's size. For that, you should use '''FileHandle->GetInfo'''. The problem is, there's no way of knowing how big buffer the information structure requires, so you'll have to grow the buffer dynamically if GetInfo fails. GNU-EFI provides a pretty handy wrapper for that, [https://sourceforge.net/p/gnu-efi/code/ci/master/tree/lib/hand.c LibFileInfo].

<syntaxhighlight lang="c">
UINT64 FileSize(EFI_FILE_HANDLE FileHandle)
{
  UINT64 ret;
  EFI_FILE_INFO       *FileInfo;         /* file information structure */
  /* get the file's size */
  FileInfo = LibFileInfo(FileHandle);
  ret = FileInfo->FileSize;
  FreePool(FileInfo);
  return ret;
}
</syntaxhighlight>

== See also ==
=== Articles ===
* [[UEFI]]
* [[GNU-EFI]]
* [[POSIX-UEFI]]
* [[UEFI Bare Bones]]
* [[EFI System Partition]]
* [[Reading sectors under UEFI]]

=== External Links ===
* [http://www.uefi.org/specifications UEFI specifications et al.]
* [https://sourceforge.net/p/gnu-efi/code/ci/master/tree/ GNU-EFI source]

[[Category:UEFI]]
````

<a id="w004538"></a>

## W004538 — Advanced Projects

Bron-ID: W004538
Namespace: Main (0)
Revisie: 27855 | Datum: 2023-07-08T17:59:27Z
Bron: https://wiki.osdev.org/Advanced_Projects
Revisie-URL: https://wiki.osdev.org/index.php?title=Advanced_Projects&oldid=27855

````text
__NOTOC__
This is the list of advanced and more successful alternative operating systems, that somewhat grow more than being just a hobby or research operating system. Their success outside of the relatively tiny spheres of research and hobby operating system development means that they have an actual user basis, and a team is working on it (or has regular contributions). They are definitely more than a one-person-project. Forks, clones of proprietary OSes, as well as research projects sponsored by companies and/or universities belong in this list too.

Projects that are still in the hobby development phase but are mature, can be found at [[Notable Projects]].

Everyone is welcome to add their own projects to the regular [[Projects]] list of all hobbyist operating systems.

== Advanced and Successful Alternative Operating Systems ==

{{OSProject
|OSName=9front
|OSDescription=Plan9front (or 9front) is a fork of the Plan 9 from Bell Labs operating system. The project was started to remedy a perceived lack of devoted development resources inside Bell Labs, and has accumulated various fixes and improvements.
|Contact=http://lists.9front.org/
|License=Open source
|URL=http://lists.9front.org/
|Pic=[[File:9front.png|200px]]
|Status=Active
}}

{{OSProject
|OSName=Amiga Research Operating System
|OSDescription=AROS aims to create a free open source AmigaOS like OS and make it better than the original. Our homepage: http://aros.sourceforge.net
|Contact=Ola Jensen, ola [at] aros [dot] org
|License=Open source (MPL-like)
|URL=http://www.aros.org
|Pic=[[File:Aros.jpg|200px]]
|Status=Active
}}

{{OSProject
|OSName=FreeDOS
|OSDescription=Today, FreeDOS is ideal for anyone who wants to bundle a version of DOS without having to pay a royalty for use of DOS. FreeDOS will also work on old hardware and embedded systems. FreeDOS is also an invaluable resource for people who would like to develop their own operating system. While there are many free operating systems out there, no other free DOS-compatible operating system exists.
|Contact=The mailing lists at http://www.freedos.org/lists/
|License=Open source (GNU GPLv2)
|URL=http://www.freedos.org
|Pic=[[File:FreeDOS.png|200px]]
|Status=Active
}}

{{OSProject
|OSName=Kolibri OS
|OSDescription=Kolibri OS was a fork of the 32-bit version of Menuet OS but has changed much along the way. Despite fitting on a standard 1.44 MB floppy, this wonderful OS contains: the complete GUI desktop, a lot of drivers and great software (such as web browser and music player), system programs and games! The Kernel and most applications, libraries and drivers are written in FASM, but some are in C-- (https://en.wikipedia.org/wiki/C-- ; link to their C-- compiler: http://c--sphinx.narod.ru/). At this OS you can write the ASM code and execute it after assembling. Source code is open - http://websvn.kolibrios.org/listing.php?repname=Kolibri+OS - and the contributions are welcome!
|Contact=The Kolibri OS team
|License=Open source (GNU GPLv2)
|URL=http://www.kolibrios.org
|Pic=[[File:KolibriOS.png|200px]]
|Status=Active
}}

{{OSProject
|OSName=Haiku
|OSDescription=Fully featured open source operating system inspired by the commercial Be Operating System. Has a preemptive, modular kernel, nearly complete POSIX compatibility, a nice (non-X11-based) GUI, and a wide variety of ported and native applications (including a WebKit based browser). Nearly the entire operating system is written in C++98 (including the kernel), albeit with little usage of exceptions.
|Contact=haiku-development [at] freelists.org
|License=Open source (MIT)
|URL=https://www.haiku-os.org/
|Pic=[[File:Haiku.png|200px]]
|Status=Active
}}

{{OSProject
|OSName=HelenOS
|OSDescription=Preemptive microkernel multiserver design, SMP support, lightweight IPC, thread-local storage and user-space managed fibrils. Developed at ETH University Zürich. Our homepage: http://www.helenos.org/
|Contact=helenos-devel [at] lists [dot] modry [dot] cz
|License=Open source (BSD/GPL)
|URL=https://github.com/HelenOS/helenos
|Pic=[[File:HelenOS.png|200px]]
|Status=Active
}}

{{OSProject
|OSName=LK (Little Kernel)
|Contact=Travis Geiselbrecht (travisg at gmail.com)
|OSDescription=An open source embedded multiprocessor kernel for ARM, x86, x86-64. Other platforms are work-in-progress and are in various stages of development with varying activity.
|License=Open source (MIT-Style)
|URL=https://github.com/littlekernel/lk
|Status=Active
}}

{{OSProject
|OSName=Minix
|Contact=A. Tannenbaum and others
|OSDescription=MINIX 3 is a free, open-source, operating system designed to be highly reliable, flexible, and secure. It is based on a tiny microkernel running in kernel mode with the rest of the operating system running as a number of isolated, protected, processes in user mode. It runs on x86 and ARM CPUs, is compatible with NetBSD, and runs thousands of NetBSD packages. Developed at VU University, Amsterdam
|License=Open source (GPT)
|URL=http://minix3.org/
|Pic=[[File:Minix.jpg|200px]]
|Status=Active
}}

{{OSProject
|OSName=ReactOS
|OSDescription=A GPL project to clone WinNT written from scratch. It runs: Firefox, OpenOffice, Quake III Arena and much more. A lot of work is still need to be done. Looking for developers
|Contact=a team of developers, ros-general [at] reactos [dot] com
|License=Open source (GNU GPLv2)
|URL=http://www.reactos.com
|Pic=[[File:ReactOS.png|200px]]
|Status=Active
}}

{{OSProject
|OSName=SerenityOS
|OSDescription=A graphical Unix-like operating system for desktop computers. It aims to marry the aesthetic of late-1990s productivity software and the power-user accessibility of late-2000s *nix.
|Contact=Andreas Kling and others
|License=Open source (BSD 2-Clause License)
|URL=https://serenityos.org
|Pic=[[File:SerenityOS.png|200px]]
|Status=Active
}}

[[Category: Resources]]
````

<a id="w004539"></a>

## W004539 — OPAL

Bron-ID: W004539
Namespace: Main (0)
Revisie: 30246 | Datum: 2026-03-16T15:46:40Z
Bron: https://wiki.osdev.org/OPAL
Revisie-URL: https://wiki.osdev.org/index.php?title=OPAL&oldid=30246

````text
'''OPAL''' (OpenPower Abstraction Layer) is an ongoing project created by IBM in an effort of standardizing the boot environment and low-level services available across POWER systems, it was introduced with POWER8.

These low-level services are meant to be used both in boot and runtime, they consist of a series of calls that can be realized for managing different hardware-related and basic Input/Output tasks, all from getting device trees to flashing devices to a console with read/write capabilities.

== Calling the functions ==

To access an OPAL routine, the system is required to be in [[Endianness|big endian]] mode.

Each OPAL routine has a number associated to it, this number is to be loaded in R0, from there only a trampoline has to be calculated using the OPAL base and entrypoint, passed to the loaded payload in R8 and R9 respectively.

The OPAL routines use the [https://refspecs.linuxfoundation.org/ELF/ppc64/PPC-elf64abi.html SysV ABI].

A really simple example of a trampoline routine and a hello world using it would be:

<syntaxhighlight lang="asm">
.section .text
.align 2
.globl opal_call
opal_call:
	/* OPAL calls must be in big-endian mode */
	/* On call, r0, r3 ... should already be set to the desired values*/
	/* Assume OPAL base is in r13 and the entrypoint in r14 */

	/* Save current MSR in non-volatile r15 */
	mfmsr	%r15
	/* Save return address in non-volatile r16 */
	mflr	%r16

	/* Turn off the LE bit in MSR (ordinarily also do DR, IR and EE) */
	li	%r11, 0x01
	andc	%r11, %r15, %r11

	/* Get address of trampoline */
	bl	.+4
	mflr	%r12 /* 0 */
	addi	%r12, %r12, 28 /* 32 bytes between the mflr and the return */
	mtlr	%r12
	
	/* 12 */
	/* We need a return trampoline to return to little endian mode */
	/* Put the trampoline address in LR, OPAL entry in SRR0, MSR in SRR1 */
	mr %r2, %r13
	mtsrr0 	%r14
	mtsrr1	%r11

	/* 24 */
	/* Do it */
	rfid

	/* 28 */
	/* Trampoline here returns us to LE and restores MSR and LR */
	GO_LITTLE_ENDIAN
	mtmsr	%r15
	mtlr	%r16
	blr

.align 2
.globl main
main:
	li	%r0, 1 /* OPAL_CONSOLE_WRITE */
	li	%r3, 0 /* terminal 0 */
	addi	%r4, %r8, len - here /* ptr to length of string */
	addi	%r5, %r8, str - here /* ptr to string start */
	bl opal_call

.section .data
len:
	/* This value is read by OPAL so it must be big-endian in both modes */
	.long 0x00
	/* byte-universal form of BE .long (strend - str) */
	.byte 0
	.byte 0
	.byte 0
	.byte (strend - str)

str:
	.string "Hello World!\n"
strend:
</syntaxhighlight>

== Function listing ==

A complete listing can be found [https://github.com/open-power/skiboot/tree/master/doc/opal-api here], but this is a list of the most useful functions.

* OPAL_CEC_POWER_DOWN - Powers down the system.
* OPAL_CONSOLE_WRITE, OPAL_CONSOLE_READ, OPAL_CONSOLE_FLUSH, OPAL_CONSOLE_WRITE_BUFFER_SPACE - Console management, from printing to reading.
* OPAL_GET_DEVICE_TREE - Fetching a flattened device tree (FDT) for kernel use.
* OPAL_START_CPU and OPAL_RETURN_CPU - Secondary CPU management.

== See Also ==

=== External Links ===
* [https://github.com/streaksu/microski A complete example of a Hello World OS using OPAL]
* [https://refspecs.linuxfoundation.org/ELF/ppc64/PPC-elf64abi.html PowerPC64 SysV API]
* [https://github.com/open-power/skiboot/tree/master/doc/opal-api 1987 Skiboot's OPAL API documentation]

[[Category:PowerPC]]
[[Category:Firmware]]
[[Category:Booting]]
[[de:OpenPower Abstraction Layer]]
````

<a id="w004540"></a>

## W004540 — User:Kb

Bron-ID: W004540
Namespace: User (2)
Revisie: 27535 | Datum: 2022-12-19T10:56:36Z
Bron: https://wiki.osdev.org/User:Kb
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKb&oldid=27535

````text
Kai Burghardt
* on [https://en.wikipedia.org/wiki/User:Kai_Burghardt Wikipedia]
* on [https://rosettacode.org/wiki/User:Root Rosetta code]
* on [https://wiki.freepascal.org/User:Kai_Burghardt FreePascal Wiki]
* on [https://stackoverflow.com/users/9636977 stack overflow]
* on [https://orcid.org/0000-0003-2117-3974 ORCID ID]
````

<a id="w004541"></a>

## W004541 — Plain English Programming

Bron-ID: W004541
Namespace: Main (0)
Revisie: 28680 | Datum: 2024-03-01T18:56:49Z
Bron: https://wiki.osdev.org/Plain_English_Programming
Revisie-URL: https://wiki.osdev.org/index.php?title=Plain_English_Programming&oldid=28680

````text
This page relates to Plain English Programming as developed by the Osmosian Order of Plain English Programmers. Notes primarily pertain to the CAL-4700 standalone IDE and compiler. There is at least one other version, Folds/english on Github, 'an authorized "dynamic fork"'. (The major differences are in UI, with Folds/english offering the standard Win32 UI elements.) Language differences, if any, should be noted below.

[[Español Llano]] is a Spanish-language equivalent.

The language is very much like pseudocode. It allows a small degree of arbitrary word choice, but the code understood by the compiler must precisely describe the program's operation. It's strongly typed with a tiny runtime; less than 2KB. It natively targets only 32-bit Windows, but is so simple that retargetting is likely no harder than any other aspect of OS development.

Far from "Hello, world," the first thing a user learns to do with CAL-4700 is recompile CAL itself. Then, the user is walked through making an app which takes a string, downloads an image from Google Image Search, and renders it in the style of famous painter Monet. It's a rather broad introduction to the language's features.

Note that a sense of humor will be an advantage when reading CAL's instructions. 

== Language ==

The language and its documentation are almost entirely free of jargon and symbols other than a subset of English punctuation. Only the period, comma, colon, and semicolon are commonly used. The minus sign functions as a unary minus for negating variables.

The Order's blog makes a case for the language being no more verbose than C or C++. This works if you're a good typist, finding whole words no more trouble to type than symbols. (Having a lot of practice in typing English, I (eekee) find it much less trouble to type than other languages. I find it hard to omit the spaces in both CamelCase and underscore-joined words, and I dislike symbols as I find using the shift key to be uncomfortable. CAL plain English only requires me to use 4 shifted symbols; colon double-quotes and the two parentheses.)

"Not" and the suffix "n't" are recognized and understood as you'd expect.

Some words are ignored. Development of the language started with the realization that human infants ignore a lot of words when they're starting to learn to understand language.

Specifying parameters is more verbose than in human English. Example: a number and another number and a third number and a fourth number. It could be shortened, but typically, you set up a structure and pass that to a routine. For instance, you set up a box and then "Draw the box with the black color." Routines with more than 2 parameters are rare in CAL-4700.

Wording tends to differ from English in some other ways too. For example, "Draw the box with the black color", rather than "Draw the box in black." FIXME: check if "in/into/to" is interpreted differently from "with". If not, the latter example is possible.

Definition order doesn't matter. (The Order likes to sort definitions by name and rely on incremental search to find them.)

=== Types ===

It has a very strict type system, surprisingly strict for a language intended to be plain English. "A buffer is a string", the compiler is told, and you can "Read the file into a buffer", but you can't "Read the file into a string." (The instructions are wrong on this point. eekee intends to raise the issue with the Order.) This example could be worked around with a record containing "a buffer and a string at the buffer," similar to the example of a box below.

'''Numbers''' are 32-bit signed integers. Unsigned numbers are not well-supported, but appear in ratios.

Floating-point support is not included for philosophical reasons. (Something to do with the philosopher Kronecker.)

'''Ratios''' are pairs of numbers, dividend/divisor.

'''Pointers''' with arithmetic permitted. 

'''Strings''' are referenced with start and end pointers. Derived types include substrings and buffers. Riders are a more complex type which works on these. Memory allocation is fully automatic.

'''Records''' do the work of structs and unions. For example:

<pre>
A box has
  a left coord, a top coord, a right coord, a bottom coord,
  a left-top spot at the left, and a right-bottom spot at the right.
</pre>

Note the union of a 4-number structure with 2 "spots" -- pairs of numbers. Not shown: Each spot has an x and a y.

'''Things''' are records with implicit 'previous' and 'next' pointers for use in doubly-linked lists. These lists may be iterated over with very little code. You write, "get a <type> from the list", and later "Loop."

Many more types are built from these, especially for the GUI.

There is no syntax for genera-purpose '''arrays''' as the language designers felt this would not be plain English. This did not prevent them implementing strings as character arrays using pointers and pointer math. The many string routines are good examples, showing how simple this is, though they only cover single-byte pointer math. (Remember the size of numbers and pointers could change if you re-target the compiler.) Memory allocation is built on the routines "To assign a pointer given a byte count", "To reassign a pointer given a byte count", and "To unassign a pointer".

=== Routines vs. Functions ===

Like C functions, routines may take parameters and return values, but unlike C functions, they may not be part of an expression. You call one routine per line.

Functions may be part of expressions. Function calls may even look like references to structure elements. In that regard, they seem similar to the methods of a pure OO language but the instructions recommends using them sparingly.

=== Loops and Conditionals ===

Nesting of loops or conditionals is not supported. Conditionals may appear within loops, but may not be nested within each other. This was chosen for clarity.

In place of nested loops, testing the remainder from a division is commonly used. Alternatively, the inner loop may be put into another definition.

Conditionals generally call a routine for the true case. Multiple routines may be called, but good style is to fit the conditional onto one line, with just one or two calls. There is no "else" for the false case. Use the construct in the next paragraph.

Switches and "else" are handled with "if" and "exit":

<pre>
If <decider 1>, <call 1>; exit.
If <decider 2>, <call 2>; exit.
If <decider 3>, <call 3>; exit.
<default case code>.
</pre>

A ''decider'' is a routine which returns a boolean. These are built in a similar way to switches, but the return syntax is unique:

<pre>
If <other decider 1>, say yes.
If <other decider 2>, say yes.
Say no.
</pre>

== Cross-Compiling with CAL ==

The compiler, text editor, "finder", and a document editor are all integrated into a single binary. In its original form, compilation is triggered by the Run menu entry in the text editor (Ctrl-R). All the files without an extension in the current directory are combined and compiled. The resultant Windows executable is then run. To compile OS code, CAL must be modified. CAL's subsystems are, on the whole, simple and easy to modify, so this will likely be no harder than many other elements of OS development. However, there are quite a number of different areas to change.

The compiler relies only on its own internals and files in the single source directory, there is no include search path, so there is very little to go wrong. Of course, you may wish to add path search for your own project as the present arrangement makes it hard to maintain a consistent version of the library. Folds/english may have useful code for this; it searches subfolders if a certain folder name is present.

=== Memory allocation ===

You will need some sort of allocator to use many of the language's features. The routines are, "To assign a pointer given a byte count", "To reassign a pointer given a byte count", and "To unassign a pointer". These call Win32 functions which need to be replaced. Additionally, initialization calls Win32 function GetProcessHeap, assigning the returned value to the global variable, "heap pointer".

=== The assembly language issue ===

CAL doesn't include an assembler, nor does it work with an external assembler. Rather, hex strings of machine code are entered in-place, together with comments showing the assembly language. It looks like this:

<pre>
To add a number to a pointer;
To add a number to another number:
Intel $8B8508000000. \ mov eax,[ebp+8] \ the number
Intel $8B00. \ mov eax,[eax]
Intel $8B9D0C000000. \ mov ebx,[ebp+12] \ the other number
Intel $0103. \ add [ebx],eax
</pre>

The good news is that there are only 447 "Intel" lines totalling 1841 bytes of machine code. They're all in the standard library, called ''the noodle''.

The authors of CAL hand-assemble this code, but there are alternatives. You could write a script to assemble a file with raw output format and hexdump the result. Write each snippet to the file and run the script. Or, you could write an inline assembler. There's little need to cover the entire instruction set of your CPU, CAL-4700 only uses 36 different instructions with a small number of addressing modes. Using all the instructions is for full-complexity compiler projects; the sort which aren't likely to leave you with any time for your OS.

=== Binary format ===

To target x86-32-ELF, such as for a bootloader, modify ''the compiler'' to output the required format. 

=== ABI ===

To target ABIs which pass arguments in registers, the minimum necessary change is to alter only the Call statement. Internal calls will still use the stack for argument passing.

Alternatively, you could modify the compiler so all calls pass arguments in registers. In CAL-4700, the compiler only emits 2 instructions on its own, to push and pop eax. Some work may be needed around those.

=== CPU architecture ===

First, decide if you're changing the ABI. Calling convention is baked into machine code in the noodle.

If you're keeping the ABI, the compiler may only need 4 lines changed; 2 each to push and pop a register. You may also want to check over other binary code emitted by the compiler for endianness and arch-dependent ABI details.

Changing "intel" to another name can be done with a simple search-and-replace on the compiler, replacing 12 instances on 11 lines.

The remaining work is to rewrite the 447 machine-code instructions in the noodle.

== Graphics ==

CAL-4700 includes a basic GUI and, almost separately, turtle graphics. How easy would it be to use them in your OS?

The GUI depends on quite a number of Windows calls, from font rendering to roundy boxes. Obviously, these would need to be replaced or redesigned.

The turtle graphics are much simpler, needing only a line drawing routine.

A 96-character font is implemented in the turtle graphics. It looks better at some PPI values than others, but may be useful for debug messages or as a very retro stylistic choice. (To find the code, start with the routine "to write a string" in the noodle. The routine "to draw a string" uses Windows font routines.)

=== Coordinate system ===

The coordinate system works entirely with integers, but is not tied to pixels. The basic unit is the "twip" which is 1/20th of a printer's point. Inches and other units are converted to twips, which in turn are multiplied by a number derived from the PPI (pixels per inch) setting to produce pixel coordinates. Rotation may be specified in degrees or fractions of a circle.

== Porting CAL ==

Porting CAL itself to your OS might seem like a quick way to get a file manager and text editor, but its drawing code relies on some relatively complex operations. At minimum, you'd have to provide routines to draw fonts and lines clipped to boxes, and convert the "roundy boxes" to square. It doesn't help that CAL's native vector font has entirely different interface routines from TrueType fonts.

=== Porting The Noodle ===

An odd feature of the compiler makes this relatively easy, initially. The compiler ignores routines which aren't called. Thus, the noodle's many Win32 calls can be replaced piecemeal. For example, to postpone dealing with sockets, comment out the lines to initialize and finalize winsock in the start up and shut down routines. On the other hand, this feature makes cleaning up harder.

=== CAL looks like an OS... ===

...with its own UI design and standards, but it's not. :) GUI code is merged, with several routines in ''the desktop'' file having code specific to the individual apps. This works well for CAL as a program, but it's an impossible framework for an OS unless you want the most inconvenient kind of [[Megalithic Kernel]].

== License ==

CAL-4700 is not under an open-source license. Each source file begins with a single-line copyright giving only years, "the osmosian order" and a version number. This unfortunately includes all the code which might be called the standard library, and the GUI code which you may (or may not) wish to use. It would be best to contact the Osmosian Order, tell them what you want to do; the programs and systems you want to create and the licensing you would like to release it under, and ask how their copyright will affect it.

Operating systems are amongst the things the Osmosian Order would like to see written in Plain English. This is stated in the [http://osmosian.com/manifesto.pdf Osmosian Manifesto. (PDF)] The manifesto also gives an email address for contact.

== Links ==

* [https://osmosianplainenglishprogramming.blog/ The Osmosian Order's blog] — Introduces the language, presents examples and answers many questions.
* Download - search the blog for "http" to get the latest link.
* [http://osmosian.com/ osmosian.com] has very little content, just a semi-humorous slideshow and a link to the manifesto. (This server also hosts downloads.)
* [https://github.com/Folds/english Folds/english on Github] — An older version with an open-source licence. Its GUI is more deeply integrated with Windows.
* [http://osmosian.com/instructions.pdf Instructions (PDF)] for a deeper look at the language. It's a little out of date. The up to date instructions are included in the CAL download.
* [http://osmosian.com/manifesto.pdf The Osmosian Manifesto (PDF)] explains what the Order would like to see, (including operating systems,) and gives a contact email.

== People and OSs using PEP ==

* [[User:eekee|eekee]] says, "I'm very happy to find a language which circumvents my problems with jargon and symbols. I can just read CAL's code without getting confused. I'm also happy with its simplicity and extensibility."

[[Category:Languages]]
````

<a id="w004542"></a>

## W004542 — User:Boringcactus

Bron-ID: W004542
Namespace: User (2)
Revisie: 25375 | Datum: 2020-11-22T05:05:58Z
Bron: https://wiki.osdev.org/User:Boringcactus
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ABoringcactus&oldid=25375

````text
hi!

https://www.boringcactus.com

i'm sketching out in my mind an OS called Hope, loosely inspired by Oberon and written in [https://crowbar-lang.org the Crowbar programming language I'm also working on].

resources i'm finding helpful in that process:

* [[Raspberry Pi Bare Bones]]
* [https://jsandler18.github.io/ Building an Operating System for the Raspberry Pi]

aspiring to be [[James T. Klik]] and [[Alta Lang]] with a crumb of [[Andy Microbaum]], but mostly just [[Dizz Tracked]].
````

<a id="w004543"></a>

## W004543 — File:Cyjon 0.1331.png

Bron-ID: W004543
Namespace: File (6)
Revisie: 25378 | Datum: 2020-11-23T20:16:59Z
Bron: https://wiki.osdev.org/File:Cyjon_0.1331.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ACyjon_0.1331.png&oldid=25378

````text

````

<a id="w004544"></a>

## W004544 — BSOD

Bron-ID: W004544
Namespace: Main (0)
Revisie: 25389 | Datum: 2020-11-28T19:57:56Z
Bron: https://wiki.osdev.org/BSOD
Revisie-URL: https://wiki.osdev.org/index.php?title=BSOD&oldid=25389
Doorverwijzing: Kernel Panic

````text
#REDIRECT [[Kernel Panic]]
````

<a id="w004545"></a>

## W004545 — Using UEFI Runtime Services in your Kernel

Bron-ID: W004545
Namespace: Main (0)
Revisie: 25396 | Datum: 2020-12-01T00:08:34Z
Bron: https://wiki.osdev.org/Using_UEFI_Runtime_Services_in_your_Kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Using_UEFI_Runtime_Services_in_your_Kernel&oldid=25396

````text
[[UEFI]] Runtime Services are functions provided by the firmware that can be used in any CPU mode, and at any time, even after you exit UEFI Boot Services. Runtime Services can contain useful functions for an operating system, such as resetting/shutting down the system, getting the current time, setting the current time, etc. One may wish to use these functions outside of the boot process. The process is relatively trivial.


== Obtaining the Runtime Services Pointer ==

=== Bootloader ===

In order to obtain the Runtime Services pointer it is highly recommended to use a modified, minimal, UEFI-exclusive bootloader.
Some bootloaders that will not work:

* GRUB
* coreboot
* NexBoot
* Any BIOS-based bootloader


=== Obtaining ===

In order to obtain the Runtime Services pointer you will need an info structure that you pass to your kernel. If you do not have one it is imperative that you create one.

You will need to simply add a member to the structure.
<source lang="c">
typedef struct s_boot_info 
{
    // A pointer to the EFI runtime services structure
    // that contains numerous useful function pointers
    EFI_RUNTIME_SERVICES *RT;
} boot_info;
</source>
In order to pass the pointer to the kernel, you need to define the EFI_RUNTIME_SERVICES structure yourself.

You need to define the Runtime Services header and structure.
<source lang="c">
typedef struct s_efi_table_header {
	uint64_t                     signature;
	uint32_t                     rev;
	uint32_t                     size;
	uint32_t                     crc;
	uint32_t                     reserved;
} Efi_Table_Header;
</source>
<source lang="c">
// A structure replicating that of the EFI_RUNTIME_SERVICES structure in UEFI
typedef struct s_efi_runtime_service_handle {
	Efi_Table_Header header;
	Efi_Get_Time                    GetTime;
	Efi_Set_Time                    SetTime;
	Efi_Get_Wakeup_Time             GetWakeupTime;
	Efi_Set_Wakeup_Time             SetWakeupTime;
	Efi_Set_Virtual_Address_Map     SetVirtualAddressMap;
	Efi_Convert_Pointer             ConvertPointer;
	Efi_Get_Variable                GetVariable;
	Efi_Get_Next_Variable_Name      GetNextVariableName;
	Efi_Set_Variable                SetVariable;
	Efi_Get_Next_High_Mono_Count    GetNextHighMonotonicCount;
	Efi_Reset_System                ResetSystem;
	Efi_Update_Capsule              UpdateCapsule;
	Efi_Query_Capsule_Capabilities  QueryCapsuleCapabilities;
	Efi_Query_Variable_Info         QueryVariableInfo;
} Efi_Runtime_Services;
</source>
You will also need to declare the methods as well as dependent structures and enums yourself.
<source lang="c">
typedef enum {
	EfiResetCold,
	EfiResetWarm,
	EfiResetShutdown
} Efi_Reset_Type;

typedef
uint64_t (EfiApi *Efi_Reset_System) 
(
    Efi_Reset_Type  	        	ResetType,
    uint64_t		              	ResetStatus,
    uint64_t				DataSize,
    uint16_t                   		*ResetData
);
</source>

To find the correct declarations/structures, refer to the UEFI headers, specifically GNU-EFI's ''efiapi.h''.

'''EFIAPI'''

This definition is not a type, it is a macro that specifies that a function must use the UEFI calling convention.
<source lang="c">
#define EfiApi __attribute__(ms_abi)
</source>

== Using the Runtime Services ==
Now that you have a pointer, you can now start calling UEFI functions.
<source lang="c">
void kmain(boot_info *info)
{
    // Shuts down the system
    info->RT->ResetSystem(EfiResetShutdown, 0, 0, nullptr);
}
</source>


[[Category:UEFI|{{PAGENAME}}]]
````

<a id="w004546"></a>

## W004546 — Talk:Using UEFI Runtime Services in your Kernel

Bron-ID: W004546
Namespace: Talk (1)
Revisie: 25398 | Datum: 2020-12-01T04:11:32Z
Bron: https://wiki.osdev.org/Talk:Using_UEFI_Runtime_Services_in_your_Kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AUsing_UEFI_Runtime_Services_in_your_Kernel&oldid=25398

````text
Decided this would be pretty useful information for some people. I also feel like one of the only people who've done it.

[[User:Rizxt|Rizxt]] 22:11, 30 November 2020 (CST)
````

<a id="w004547"></a>

## W004547 — Talk:Scalable Screen Font

Bron-ID: W004547
Namespace: Talk (1)
Revisie: 25403 | Datum: 2020-12-03T21:07:45Z
Bron: https://wiki.osdev.org/Talk:Scalable_Screen_Font
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AScalable_Screen_Font&oldid=25403

````text
I am going to be linking this page into the new page I am writing about porting legacy-BIOS OSes to UEFI.

Cheers,

rizxt
````

<a id="w004548"></a>

## W004548 — Porting to UEFI

Bron-ID: W004548
Namespace: Main (0)
Revisie: 29105 | Datum: 2024-06-07T13:40:00Z
Bron: https://wiki.osdev.org/Porting_to_UEFI
Revisie-URL: https://wiki.osdev.org/index.php?title=Porting_to_UEFI&oldid=29105

````text
In recent years, UEFI has taken over firmware duties in PC-compatibles. Most modern machines use UEFI firmware. A goal for OS developers may be to have their OS boot using UEFI natively (as opposed to legacy BIOS or CSM). If you followed the legacy Bare Bones tutorial, and you want your OS to work on real hardware, then you may wish to know this information.

== Changes ==
Most UEFI implementations do implement compatibility with legacy BIOS in the form of CSM (compatibility support module), so you may not need to make your OS UEFI compatible. However Intel already announced the '''removal of CSM by 2020''', so relying on it is not future-proof. Just note that not all machines may support CSM and therefore it may be worth it to make your OS UEFI capable.

However, one breaking change that will definitely affect your OS is that UEFI does not support traditional CGA text mode. You are required to use graphics mode, and there is no text mode emulation in place.

Here are a few mandatory changes for legacy-BIOS OSes:

* You will need to write your own (new) terminal driver that uses bitmap fonts to draw text onto the screen.
* You will need to request a graphical framebuffer from your bootloader.

== Porting ==
=== Boot Process ===
==== GRUB ====
For [[GRUB]], you will need to specify that you wish to request a linear framebuffer in your multiboot header.

Worth noting that one may be wanting to use multiboot2 instead of multiboot as the latter does not provide the kernel with critical information such as the RSDP address.

==== BOOTBOOT ====
For [[BOOTBOOT]], no changes are required to boot in UEFI.

==== Limine ====
The [[Limine Bare Bones | Limine boot protocol]] allows a kernel to run mostly unmodified on both UEFI and BIOS.

==== Custom Bootloaders ====
For custom bootloaders, you are on your own, obviously.

One starting point would be to figure out a standardised boot protocol to use to load your kernel from both a BIOS version of your loader and a UEFI one.

One could also decide to ditch BIOS compatibility entirely and have the kernel rely on UEFI facilities directly, although for the sake of portability this is discouraged.

=== Terminal ===
For your terminal driver, you can roll your own, or use a library.

Your options include:
* Using the [[Scalable Screen Font]] library
* Creating your own driver using [[PC Screen Font]]s
* Creating your own driver using 8x8 fonts, stored as 64-bit unsigned integers (see section "Displaying a character" on [[VGA Fonts]])

For a few examples of graphical terminal drivers, refer to the links section at the bottom.

=== What about 32-bit OSes? ===
Most UEFI firmware is designed for x86-64, so therefore if your kernel is designed for x86 or i386, you will need to either

* Port your OS to x86-64, or
* Create a trampoline at your kernel entry which drops you back down to protected mode from long mode

== Links ==

=== Multiboot example ===
* [https://github.com/Slapparoo/MultibootBasicGraphics/blob/master/src/loader.s Multiboot header example] in assembly

=== Terminal drivers ===
* [https://github.com/Slapparoo/MultibootBasicGraphics/blob/master/src/vga.cpp MultibootBasicGraphics]
* BOOTBOOT's example [https://gitlab.com/bztsrc/bootboot/-/blob/master/mykernel/kernel.c Hello World kernel] (bare minimum [[PC Screen Font]] driver)

[[Category:x86]]
[[Category:UEFI]]
````

<a id="w004549"></a>

## W004549 — User:Atie

Bron-ID: W004549
Namespace: User (2)
Revisie: 27297 | Datum: 2022-08-15T11:10:04Z
Bron: https://wiki.osdev.org/User:Atie
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAtie&oldid=27297

````text
Hello
````

<a id="w004550"></a>

## W004550 — Reading sectors under UEFI

Bron-ID: W004550
Namespace: Main (0)
Revisie: 25441 | Datum: 2020-12-15T16:38:07Z
Bron: https://wiki.osdev.org/Reading_sectors_under_UEFI
Revisie-URL: https://wiki.osdev.org/index.php?title=Reading_sectors_under_UEFI&oldid=25441

````text
Note that you don't necessarily need to access raw block devices, because UEFI provides [[Loading_files_under_UEFI|files abstraction]] with EFI_SIMPLE_FILE_SYSTEM_PROTOCOL (typically limited to [[FAT]]-only partitions).

== Iterating on Disks ==

If you already have a handle for a block device, then your job is easy. However more often you don't, which means first you must query the list of block devices in the system. Because you don't know in advance how many devices there are, you must locate the BLOCK_IO_PROTOCOL twice.
<source lang="c">
EFI_GUID bioGuid = BLOCK_IO_PROTOCOL;
EFI_BLOCK_IO *bio;
EFI_HANDLE *handles = NULL;
UINTN handle_size = 0;

/* first, locate the protocol with empty buffer */
status = uefi_call_wrapper(BS->LocateHandle, 5, ByProtocol, &bioGuid, NULL, &handle_size, handles);
if (status != EFI_BUFFER_TOO_SMALL || handle_size == 0) {
    /* AUCH, error we don't have BLOCK_IO */
}

/* get a buffer for the handles now that we know the buffer size */
status = uefi_call_wrapper(BS->AllocatePages, 4, 0, 2, (handle_size + PAGESIZE-1) / PAGESIZE, (EFI_PHYSICAL_ADDRESS*)&handles);
if (EFI_ERROR(status) || handles == NULL) {
    /* AUCH, error memory allocation failed */
}

/* second round, locate the protocol with big enough buffer */
status = uefi_call_wrapper(BS->LocateHandle, 5, ByProtocol, &bioGuid, NULL, &handle_size, handles);
</source>
Now "handles" is an array that has a handle for each and every block device in the system.

We should iterate on that array, and use ith handle to get the block IO interface for a device.
<source lang="c">
for (i = 0; i < handle_size / sizeof(EFI_HANDLE); i++) {
    status = uefi_call_wrapper(BS->HandleProtocol, 3, handles[i], &bioGuid, (void **) &bio);
    /* if unsuccessful, skip and go over to the next device */
    if (EFI_ERROR(status) || bio == NULL || bio->Media->BlockSize==0)
        continue;
    /* TODO: do something with that device. */
}
</source>

== Loading Raw Sectors ==

The "bio" interface inside the iteration above refers to one particular device. You can use that interface to load sectors from that device quite easily:
<source lang="c">
UINT8 mbr[512];

status = uefi_call_wrapper(bio->ReadBlocks, 5, bio, bio->Media->MediaId, 1, 512, &mbr);
if (EFI_ERROR(status)) {
    /* AUCH, error reading sector */
}
</source>

== See also ==
=== Articles ===
* [[UEFI]]
* [[UEFI Bare Bones]]
* [[EFI System Partition]]
* [[Loading files under UEFI]]

== External Links ==
* [http://www.uefi.org/specifications UEFI specifications et al.]
* [https://sourceforge.net/p/gnu-efi/code/ci/master/tree/ GNU-EFI source]

[[Category:x86-64]]
[[Category:UEFI]]
````

<a id="w004552"></a>

## W004552 — Pit

Bron-ID: W004552
Namespace: Main (0)
Revisie: 25488 | Datum: 2020-12-27T16:43:23Z
Bron: https://wiki.osdev.org/Pit
Revisie-URL: https://wiki.osdev.org/index.php?title=Pit&oldid=25488
Doorverwijzing: Programmable Interval Timer

````text
#REDIRECT [[Programmable Interval Timer]]
````

<a id="w004553"></a>

## W004553 — User:Superleaf1995/LeafOS

Bron-ID: W004553
Namespace: User (2)
Revisie: 25490 | Datum: 2020-12-27T23:12:36Z
Bron: https://wiki.osdev.org/User:Superleaf1995/LeafOS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASuperleaf1995%2FLeafOS&oldid=25490
Doorverwijzing: User:Superleaf1995/uDOS

````text
#REDIRECT [[User:Superleaf1995/uDOS]]
````

<a id="w004554"></a>

## W004554 — Stivale Bare Bones

Bron-ID: W004554
Namespace: Main (0)
Revisie: 27249 | Datum: 2022-07-19T14:27:54Z
Bron: https://wiki.osdev.org/Stivale_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Stivale_Bare_Bones&oldid=27249
Doorverwijzing: Limine Bare Bones

````text
#redirect [[Limine Bare Bones]]
````

<a id="w004555"></a>

## W004555 — Stivale

Bron-ID: W004555
Namespace: Main (0)
Revisie: 27250 | Datum: 2022-07-19T14:29:54Z
Bron: https://wiki.osdev.org/Stivale
Revisie-URL: https://wiki.osdev.org/index.php?title=Stivale&oldid=27250
Doorverwijzing: Limine Bare Bones

````text
#redirect [[Limine Bare Bones]]
````

<a id="w004556"></a>

## W004556 — Go

Bron-ID: W004556
Namespace: Main (0)
Revisie: 29986 | Datum: 2026-02-12T17:10:33Z
Bron: https://wiki.osdev.org/Go
Revisie-URL: https://wiki.osdev.org/index.php?title=Go&oldid=29986

````text
{{stub}}
'''Go''' (also called '''Golang''') is a language developed at Google. It was never designed to be a bare metal language, but with some tricks you can compile a kernel with it.

== Operating System Development ==

Go is a garbage-collected language with a comparable amount of runtime to C and C++, and the language needs some callbacks in order to work. Most notably Go lacks important features for kernel development such as converting an integer into a pointer (which is trivial in C; a small Assembly wrapper would be required for Go) and unions. Expect the need for creating wrappers just like that to workaround the language and make suitable for bare metal. Some of the projects below simply use [[C]] for the parts which cannot be written in Go natively (with the [[GRUB|Multiboot header]] being one of them).

Details were discussed at GolangUK2017 talk "Bare Metal Gophers: Can you write a kernel in Go?", see [https://speakerdeck.com/achilleasa/bare-metal-gophers-can-you-write-an-os-kernel-in-go slides here].

At the 36th Chaos Communication Congress in 2019 a full framework for bare metal Go development was released, see [https://www.youtube.com/watch?v=4QircrJjEtQ talk] and [https://github.com/abarisani/abarisani.github.io/tree/master/research/tamago slides], the TamaGo framework currently supports several ARM unikernels and also support AMD64 KVM bare metal execution.

== Compilers ==
Several compilers for Go exist:
* [https://go.googlesource.com/go Go] ([https://github.com/golang/go GitHub mirror]), the official main implementation. Architecture support: [[AMD64]], [[x86]], [[ARM]], [[AArch64|ARM64]], [[PowerPC|ppc64]], [[PowerPC|ppc64le]], [[MIPS|mips64]], [[MIPS|mips64le]], [[MIPS]], [[MIPS|mipsle]], [[RISC-V|riscv64]], [[LoongArch|loong64]], and [[s390x]].

* [https://gcc.gnu.org/onlinedocs/gccgo/ gccgo], a [[GCC]] frontend for Go
* [https://tinygo.org/ TinyGo], implementation targeted at embedded systems. Uses [[LLVM]] as the codegen backend.

== Projects ==

* [https://github.com/achilleasa/bare-metal-gophers Bare Metal Gophers] - code repository for the talk (does not compile with latest Go compiler)
* [https://github.com/usbarmory/tamago TamaGo] - a complete bare metal Go for AMD64/ARM/RISC-V processors
* [https://github.com/icexin/eggos EggOS] - a fairly complete environment to run Go applications on bare metal (some parts written in C)
* [https://github.com/tgascoigne/goose Goose] - the bare minimum environment and a protected mode "Hello World" kernel in Go
* [https://gitlab.com/bztsrc/bootboot/tree/master/mykernel/go mykernel] a simplistic, long mode "Hello World" kernel in Go

== See Also ==

* [[Go Bare Bones]]
* [[BOOTBOOT]] loader has an example 64 bit higher half kernel in Go

== External Links ==
* [https://go.dev/ Go homepage]
* [https://speakerdeck.com/achilleasa/bare-metal-gophers-can-you-write-an-os-kernel-in-go Bare Metal Gophers: Can you write a kernel in Go?]
* [https://www.reddit.com/r/golang/comments/j9u10g/a_go_unikernel_running_on_x86_bare_metal/ A Go unikernel running on x86 bare metal (EggOS)]

[[Category:Languages]]
````

<a id="w004557"></a>

## W004557 — File:Tilck screenshot.png

Bron-ID: W004557
Namespace: File (6)
Revisie: 25568 | Datum: 2021-01-19T17:17:25Z
Bron: https://wiki.osdev.org/File:Tilck_screenshot.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ATilck_screenshot.png&oldid=25568

````text
A screenshot of Tilck: https://github.com/vvaltchev/tilck
````

<a id="w004558"></a>

## W004558 — POSIX-UEFI

Bron-ID: W004558
Namespace: Main (0)
Revisie: 30907 | Datum: 2026-08-09T08:28:35Z
Bron: https://wiki.osdev.org/POSIX-UEFI
Revisie-URL: https://wiki.osdev.org/index.php?title=POSIX-UEFI&oldid=30907

````text
[https://gitlab.com/bztsrc/posix-uefi POSIX-UEFI] is a very lightweight developing environment to create UEFI applications, similar to [[GNU-EFI]]. The [[EDK2]] is a large environment with its own build system. POSIX-UEFI on the other hand is just a single, statically linked library (about 32k) and headers for compiling UEFI applications under Linux (and other POSIX compatible systems).

This tutorial will guide you through to create a simple Hello World UEFI application using this library.

== Basic Concept ==

The concept is, EDK2's API (which gnu-efi tries to comply with) has a reasonably unfamiliar API. POSIX-UEFI aims to wrap that over with a more conventional POSIX API.

== Requirements ==

Download [https://gitlab.com/bztsrc/posix-uefi POSIX-UEFI] and copy the '''uefi''' directory into your source tree (about a dozen files, ca. 132K).
<source lang="bash">
$ git clone https://gitlab.com/bztsrc/posix-uefi.git
$ cd (your project)
$ ln -s ../posix-uefi/uefi
</source>

=== Libraries ===
Normally this is completely hidden from the developer, not needed to know. When you first time run ''USE_GCC=1 make'', it will create the following files in the "uefi/" directory:
* '''crt0_x86_64.o''': A CRT0 (C runtime initialization code) that will call your "main" function (note: no efi_main!!!).
* '''libuefi.a''': A small static library which provides the POSIX compatibility layer.
This is only when you use GCC. CLang links directly with the object files, because it is configured to generate PE files in the first place, so it can't link with an ELF static library.

=== Headers ===

POSIX-UEFI provides everything in a single header file. It has defines and typedefs for almost everything you'll ever need (Simple File System Protocol, Console In/Out Protocols, Graphics Output Protocol, Serio IO Protocol, Block IO Protocol etc. etc. etc.), but it not complete, and never intended to be. The goal is to have all the defines needed for it's libc. Since it provides a POSIX layer, all the usual UEFI defines are renamed according to ANSI C standards, like EFI_MEMORY_DESCRIPTOR -> efi_memory_descriptor_t, UINTN -> uintn_t etc.

If you need some more header files, you can use the EDK2 or gnu-efi ones under /usr/include/efi. POSIX-UEFI will add those include paths as well, and its header file was carefully written so that it can work with and without those headers, no naming conflicts.

=== Linker Script ===

Don't care. POSIX-UEFI has your back covered and chooses the correct script for you.

=== Code Examples ===

The repository contains [https://gitlab.com/bztsrc/posix-uefi/-/tree/master/examples examples] for all the functions that a boot loader might need:
* The source for the Hello World example we are talking about
* Querying and displaying command line arguments
* Listing directories and loading files (lot simpler to use "fopen" than [[Loading_files_under_UEFI|using UEFI to load files]])
* Reading and writing sectors (again, lot simpler than [[Reading_sectors_under_UEFI|using UEFI to read sectors]])
* Using the serial port with fprintf to print debug messages
* Detecting and listing [[Detecting_Memory_(x86)|memory map]]
* Querying and setting video modes with [[GOP]]
* Printing UTF-8 strings with bitmap fonts, similar to [[VGA Fonts]] but proportional
* Printing UTF-8 strings with vector based fonts using [[Scalable Screen Font]] library
* Loading and blitting PNG images to the screen (without libpng using GOP->Blt)
* Loading and parsing [[ELF]] files
* Exit Boot Services and pass parameters to an ELF "kernel"
You can use these examples as a base for your [[Rolling_Your_Own_Bootloader|bootloader]]. If this isn't enough, then I'd suggest to take a look at the [[BOOTBOOT]] loader.

== Creating an EFI executable ==

The traditional "Hello, world" UEFI program is shown below, which we will compile using POSIX-UEFI in this tutorial.

'''main.c''':
<source lang="c">
#include <uefi.h>

int main (int argc, char **argv)
{
  printf("Hello, world!\n");
  return 0;
}
</source>

As you can see, almost exactly the same as the standard POSIX version, except for the include. There's one notable difference, the UEFI environment uses wide characters, so you get the command line wchar_t, and all the string literals must be prefixed like this: L"", but POSIX-UEFI does this conversion transparently for you. If you comment out USE_UTF8 in uefi.h, then you'll get wchar_t everywhere, and your string will be passed as-is to the UEFI interfaces.

=== Compiling and Linking ===

Create a minimalistic Makefile for your project:

'''Makefile''':
<source lang="make">
TARGET = helloworld.efi

include uefi/Makefile
</source>

That's all. Now you can compile your project for UEFI by running ''make''. By default compiles using LLVM toolchain, for GNU gcc, add "USE_GCC=1" before the include. For a full list of available Makefile variables, see the documentation.

The point being, just don't care about the details. POSIX-UEFI takes care of everything, from figuring out whether you can use the host compiler or you need a cross one, or which flags has to be used for gcc and for CLang, etc. That's the point of POSIX-UEFI, to make your life easier.

As a result, you'll get "helloworld.efi" in your project's directory.

== Calling Any Arbitrary UEFI Function ==

Unlike GNU-EFI, which has many many library functions to cover up the EFI API, POSIX-UEFI is really lightweight and only provides two additional functions, no more. It provides the same globals like GNU-EFI (BS, RT, ST). Furthermore it gives you 'exit_bs' to leave the UEFI realm, and 'getchar_ifany' for non-blocking key press reads. Everything else can be accessed just like in EDK2, no need for "uefi_call_wrapper":
<source lang="c">
ST->ConOut->OutputString(ST->ConOut, buffer);
</source>
On the other hand, POSIX-UEFI provides standard libc API (hence its name). For example, opening a file goes just like under Linux, you can use fopen. Printing with printf, fprintf; using streams like stdin, stdout, stderr; allocating using malloc / free works the same way as expected on a POSIX compliant system.

== See also ==
=== Articles ===
* [[UEFI]]
* [[UEFI Bare Bones]]
* [[GNU-EFI]]
* [[EFI System Partition]]
* [[Bootable Disk]]
* [[Rolling Your Own Bootloader]]

== External Links ==
* [http://www.opengroup.org/austin/papers/posix_faq.html POSIX faq] 
* [http://www.uefi.org/specifications UEFI specifications et al.]
* [https://gitlab.com/bztsrc/posix-uefi POSIX-UEFI source]

[[Category:x86-64]]
[[Category:UEFI]]
````

<a id="w004559"></a>

## W004559 — Talk:PowerPC Overview

Bron-ID: W004559
Namespace: Talk (1)
Revisie: 25593 | Datum: 2021-02-01T01:27:00Z
Bron: https://wiki.osdev.org/Talk:PowerPC_Overview
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APowerPC_Overview&oldid=25593

````text
The PowerPC page has not been updated in over 20 years.  The links and some other information is very much out of date.
* Motorolla spun off its processors to a separate company, Freescale, that was then purchased by NXP.
* Apple no longer sells PPC based equipment that I know of
* There are 32 and 64 bit versions of the PowerPC, and several core variations currently out there
I'm new to the OSDev.org community. Is it a problem if I update some of the links and other information about the PowerPC?
A lot of the OS work I do these days is primarily on the PowerPC (E500v2 (32 bit, single processor, embedded) and E6500 (64 bit, multi-thread multi core with hypervisor support and IO MMU), so I currently have a lot of information on hand.
````

<a id="w004560"></a>

## W004560 — User:Gravaera/PCI Local Bus Signals

Bron-ID: W004560
Namespace: User (2)
Revisie: 25613 | Datum: 2021-02-06T02:22:56Z
Bron: https://wiki.osdev.org/User:Gravaera/PCI_Local_Bus_Signals
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FPCI_Local_Bus_Signals&oldid=25613
Doorverwijzing: PCI Local Bus Signals

````text
#REDIRECT [[PCI Local Bus Signals]]
````

<a id="w004561"></a>

## W004561 — User:Gravaera/PCI Local Bus Commands

Bron-ID: W004561
Namespace: User (2)
Revisie: 25615 | Datum: 2021-02-06T02:24:28Z
Bron: https://wiki.osdev.org/User:Gravaera/PCI_Local_Bus_Commands
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FPCI_Local_Bus_Commands&oldid=25615
Doorverwijzing: PCI Local Bus Commands

````text
#REDIRECT [[PCI Local Bus Commands]]
````

<a id="w004562"></a>

## W004562 — OVMF

Bron-ID: W004562
Namespace: Main (0)
Revisie: 30143 | Datum: 2026-03-05T23:16:12Z
Bron: https://wiki.osdev.org/OVMF
Revisie-URL: https://wiki.osdev.org/index.php?title=OVMF&oldid=30143

````text
{{Stub}}

'''OVMF''' (Open Virtual Machine Firmware) is an [[UEFI]] implementation to be used in virtual machines or emulators, such as [[QEMU]]. It is free and open source software. It is part of Tianocore's [[EDK2|EDK II]] project.

Many Linux distributions let you install OVMF with their default installer.

For example on Debian/Ubuntu and derivatives:

  sudo apt install ovmf

Or on Arch/Manjaro and derivatives:

  sudo pacman -S ovmf

== See Also ==
=== External links ===
* https://github.com/tianocore/tianocore.github.io/wiki/OVMF

[[Category:UEFI]]
[[Category:Firmware]]
````

<a id="w004563"></a>

## W004563 — User:Andrewaprice

Bron-ID: W004563
Namespace: User (2)
Revisie: 25670 | Datum: 2021-02-13T02:50:49Z
Bron: https://wiki.osdev.org/User:Andrewaprice
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAndrewaprice&oldid=25670

````text
=Perception=
Perception is the name of my OS project.

https://github.com/AndrewAPrice/Perception

== Wiki Things ==
* [[MessiahAndrw/Designing A Language]]
* [[User:MessiahAndrw/LLVM OS Specific Toolchain]]
````

<a id="w004565"></a>

## W004565 — User:Foliagecanine

Bron-ID: W004565
Namespace: User (2)
Revisie: 25682 | Datum: 2021-02-14T22:59:45Z
Bron: https://wiki.osdev.org/User:Foliagecanine
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AFoliagecanine&oldid=25682

````text
= foliagecanine =
Hello, my username is foliagecanine.

I've been experimenting with operating systems for several years (approx. since 2016).

== Pages ==
* [[User:foliagecanine/Paging in Long Mode]]
````

<a id="w004566"></a>

## W004566 — User:Foliagecanine/Paging in Long Mode

Bron-ID: W004566
Namespace: User (2)
Revisie: 25810 | Datum: 2021-04-15T17:47:06Z
Bron: https://wiki.osdev.org/User:Foliagecanine/Paging_in_Long_Mode
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AFoliagecanine%2FPaging_in_Long_Mode&oldid=25810

````text
Paging in [[Long Mode]] is very similar to paging in protected mode. It is based off of [[Physical Address Extension]].

== Paging Structures ==

=== Page-map Level 4 Tables and Entries ===
PML4 Tables, or Page-map Level 4 Tables, are the topmost paging structure. It is an array of 512 PML4Es (Page-map Level 4 Entries).

Each PML4E points to a PDPT (Page Directory Pointer Table). PML4Es are 64 bits in length and use the following format:

{| {{wikitable}}
! Bit(s)
! Name
! Description
|-
| 0 || P || Page Present
|-
| 1 || R/W || Read/write (0=read only)
|-
| 2 || U/S || User/supervisor (0=supervisor only)
|-
| 3 || PWT || Page write-through
|-
| 4 || PCD || Page cache disable
|-
| 5 || A || Accessed
|-
| 6 || IGN || Ignore
|-
| 8:7 || MBZ || Must Be Zero
|-
| 11:9 || AVL || Available to Software
|-
| 51:12 || Page Directory Pointer Base Address || First 36 bits of the pointer to a page containing a Page Directory Pointer Table
|-
| 62:52 || Available || Available
|-
| 63 || NX || No-execute/disable execution
|-
|}

PML4 Tables must be 4KiB aligned due to the fact that the addresses use 36 bits, but can address up to 48 bits.

=== Page Directory Pointer Tables and Entries ===
PDPTs, or Page Directory Pointer Tables, are arrays of 512 Page Directory Pointer Entries. PDPTs are the level below PML4Ts but above PDs.

Each PDPTE is 64 bits in length and use the following format:
{| {{wikitable}}
! Bit(s)
! Name
! Description
|-
| 0 || P || Page Present
|-
| 1 || R/W || Read/write (0=read only)
|-
| 2 || U/S || User/supervisor (0=supervisor only)
|-
| 3 || PWT || Page write-through
|-
| 4 || PCD || Page cache disable
|-
| 5 || A || Accessed
|-
| 6 || IGN || Ignore
|-
| 7 || PS || Size (0 = <1GiB, 1 = 1GiB pages)
|-
| 8 || IGN || Ignore
|-
| 11:9 || AVL || Available to Software
|-
| 51:12 || Page Directory Base Address || First 36 bits of the pointer to a page containing a Page Directory.

Will point to a 1GiB page if PS=1 and is supported.
|-
| 62:52 || Available || Available
|-
| 63 || NX || No-execute/disable execution
|-
|}

The S bit is dependent on whether 1GiB pages are supported. For more information on 1GiB pages, see the AMD64 Architecture Programmer's Manual section 5.3.5.

PDPTs must be 4KiB aligned.

=== Page Directories and Page Directory Entries ===

PDs, or Page Directories, are arrays of 512 Page Directory Entries. PDs are the level below PDPTs but above PTs.

Each Page Directory Entry is 64 bits long and uses the following format:

{| {{wikitable}}
! Bit(s)
! Name
! Description
|-
| 0 || P || Page Present
|-
| 1 || R/W || Read/write (0=read only)
|-
| 2 || U/S || User/supervisor (0=supervisor only)
|-
| 3 || PWT || Page write-through
|-
| 4 || PCD || Page cache disable
|-
| 5 || A || Accessed
|-
| 6 || IGN || Ignore
|-
| 7 || PS || Size (0 = <2MiB, 1 = 2MiB pages)
|-
| 8 || IGN || Ignore
|-
| 11:9 || AVL || Available to Software
|-
| 51:12 || Page Table Base Address || First 36 bits of the pointer to a page containing Page Table

Will point to a 2MiB page if PS=1.
|-
| 62:52 || Available || Available
|-
| 63 || NX || No-execute/disable execution
|-
|}

PDs must be 4KiB aligned.

=== Page Tables and Entries ===

The lowest level of tables are Page Tables. They are arrays of 512 PTEs (Page Table Entries). Page Table Entries point to 4KiB pages.

Each Page Table Entry is 64 bits long and uses the following format:
{| {{wikitable}}
! Bit(s)
! Name
! Description
|-
| 0 || P || Page Present
|-
| 1 || R/W || Read/write (0=read only)
|-
| 2 || U/S || User/supervisor (0=supervisor only)
|-
| 3 || PWT || Page write-through
|-
| 4 || PCD || Page cache disable
|-
| 5 || A || Accessed
|-
| 6 || D || Dirty
|-
| 7 || PAT || (Ignore)
|-
| 8 || G || Global Translation
|-
| 11:9 || AVL || Available to Software
|-
| 51:12 || Physical Page Base Address || First 36 bits of the pointer to a physical page of memory

Will point to a 2MiB page if PS=1.
|-
| 62:52 || Available || Available
|-
| 63 || NX || No-execute/disable execution
|-
|}

=== CR3 ===

Bits 51:12 of the CR3 register will point to a PML4T.

== Determining Physical Addresses ==

[[File:LongModePagingVirtualAddressExample.png|right|x320px]]

To determine a physical address from a virtual address, the following is used:

# The PML4T index is determined by bits 47:39 of the virtual address.
# If the P bit is set, The PML4E will point to a PDPT.
# The PDPT index is determined by bits 38:30 of the virtual address.
# If the P and S bits are set (and 1GiB pages are supported), the PDPE will point to a 1GiB page. Otherwise, if the P bit is set, it will point to a PD.
# The PD index is determined by bits 29:21 of the virtual address.
# If the P and S bits are set, the PDE will point to a 2MiB page. Otherwise, if the P bit is set, it will point to a PT.
# The PT index is determined by biys 20:12 of the virtual address.
# If the P bit is set, the PTE will point to a 4KiB page.
# Any remaining bits are ORed with the physical address to determine the final virtual address

* If at any step the P bit is not set, it will produce a #PF (see [[Exceptions#Page_Fault]]).
* If at any step the R/W bit is not set and the operation is a write, it will produce a #PF.
* If at any step the U/S bit is not set and the proccessor is not in Ring 0, it will produce a #PF.
* If at any step reserved (MBZ) bits are set to 1, it will produce a #PF.
* If at any step the NX bit is set and the operation is an instruction fetch, it will produce a #PF.

== See Also ==

=== Articles ===
* [[Paging]]
* [[Setting Up Paging With PAE]]
````

<a id="w004567"></a>

## W004567 — Terminals

Bron-ID: W004567
Namespace: Main (0)
Revisie: 29384 | Datum: 2025-02-08T12:30:48Z
Bron: https://wiki.osdev.org/Terminals
Revisie-URL: https://wiki.osdev.org/index.php?title=Terminals&oldid=29384

````text
This page is about the very essentials of how to implement video terminal functionality. Terminals are connected to the OS over [[Serial_Ports|serial port]], but could use network too with the [[telnet]] and [[ssh]] protocols.

==History==
[[File:VT220_Irssi.jpg|200px|thumb|A real terminal]]
[[File:PuTTY_Ubuntu.png|200px|thumb|PuTTY with serial port option]]
[[File:Minicom_cs_xterm.png|200px|thumb|Minicom running in xterm]]

Originally terminals were line printers. The user typed in a command, pressed enter, and the computer executed that command and printed the results line by line. This is where [[Command_Line|command line interface]] originates. It understood the basic [https://en.wikipedia.org/wiki/ASCII#Control_characters ASCII control characters], like ASCII 8 (backspace), ASCII 10 (line feed), ASCII 13 (carriage  return) etc. but nothing else.

Although this input and output method works for simple programs, it is very complicated to implement a text editor for example. Many editors kept the line terminal heritage, like Vi, but to provide a more comfortable interface, the printer was replaced with a screen. With it come the need to position the cursor on the screen, hence video terminal and video terminal sequences was born.

There were many more or less successful terminals (most notably the VT52), but finally Digital Equipment Corporation's [[VT100]] got widespread and its codes got standardized. The VT100 codes (also called ANSI escape codes) are the most widely emulated sequences, ever.

==Communication==

When your OS is communicating over the serial like, it is very likely that on the other end there's a terminal. Either a real one, or a terminal emulator, like PuTTY or minicom. It is the terminal's duty to handle the [[Keyboard|keyboard]], the key modifier keys (like Shift, Control, Alt), and send terminal key sequences. It is also the terminal's duty to interpret the received terminal codes (sometimes also called escape sequences) and visualize them properly. These sequences always start with the ASCII 27 (escape) character, "\033" in C literals, and terminated by a latin uppercase or lowercase letter or tilde ("~"). In between there might be numeric parameters (0-9) separated by comma ("," and ";") but such arguments are always prefixed by "[". This two bytes start sequence "\033[" also called Control Sequence Introducer, or CSI.

===Receiving Keys===

This section is about the bytes your OS might receive from a terminal. It is important to know that terminals are different, some are configurable, so they might send different codes. For a comprehensive list of all possible key codes, consult the [https://raw.githubusercontent.com/marcv81/termcap/master/termcap.src termcap database].

{| {{wikitable}}
! Code
! Name
! Meaning
|-
| ASCII 3 || Break (^C) || the Break signal
|-
| ASCII 7 || Bell (^G) || makes a beep
|-
| ASCII 8 || BS (^H) || Backspace, moves the cursor left and delete, some terminals send ASCII 127 instead
|-
| ASCII 9 || Tab (^I) || Tabulator, moves to cursor to the next multiple of 8 coloumn on the right
|-
| ASCII 10 || Line Feed (^J) || move the cursor down (on modern machines line feed and carriage return is combined and called newline)
|-
| ASCII 13 || Carriage Return (^M) || Return or Enter key, move the cursor to the left most coloumn
|-
| ASCII 27 || Escape (^[) || start the command sequences
|-
| ASCII 32 - 126 || keys || "normal" keys
|-
| ASCII 127 or \033[3~ || Delete || delete without moving the cursor, some terminals send ASCII 127 instead of ASCII 8, but \033[3~ is standard
|-
| \033[A || Up || up arrow key
|-
| \033[B || Down || down arrow key
|-
| \033[C || Right || right arrow key
|-
| \033[D || Left || left arrow key
|-
| \033[5~ or \033[S || PgUp || the Page Up key
|-
| \033[6~ or \033[T || PgDn || the Page Down key
|-
| \033[7~ or \033[H || Home || the Home key
|-
| \033[8~  || End || the End key
|-
| \033[11~ to \033[21~ || F1 - F10 || function keys
|-
| \033[''row'';''col''R || Status report || answer to the \033[6n code
|}
Note that cursor keys are standard, but there might be differences with the Home, End, PgUp, PgDn and Fn keys depending on the terminal. Key modifiers (like Shift, Caps Lock, Control, Alt, Compose etc.) are always interpreted by the terminal, and have no codes.

===Sending Sequences===

These are the bytes that your OS might send to a terminal. Terminals and terminal emulators are different, they usually understand different codes. However there's a subset of codes that all guaranteed to understand, and those are the VT100 codes. Modern terminal emulators usually support more, the [https://vt100.net/docs/vt220-rm/ VT220] subset is the smallest common denominator. Everything above that requires exact terminal type detection and differentiated codes.

{| {{wikitable}}
! Code
! Name
! Meaning
|-
| \033[''n''A || CUU || move the cursor up ''n'' rows
|-
| \033[''n''B || CUD || move the cursor down ''n'' rows
|-
| \033[''n''C || CUF || move the cursor right ''n'' coloumns
|-
| \033[''n''D || CUB || move the cursor left ''n'' coloumns
|-
| \033[''n''E || CNL || move the cursor to the beginning of line ''n'' down
|-
| \033[''n''F || CPL || move the cursor to the beginning of line ''n'' up
|-
| \033[''n''G || CHA || move the cursor to coloumn ''n''
|-
| \033[H || CUP || move the cursor to the top left corner of screen
|-
| \033[''row'';''col''H || CUP || move the cursor
|-
| \033[''row'';''col''f || HVP || move the cursor
|-
| \033[J or \033[0J || ED || clear the screen from cursor to the end
|-
| \033[1J || ED || clear the screen from beginning to the cursor
|-
| \033[2J || ED || clear the entire screen
|-
| \033[K or \033[0K || EL || clear the line from cursor to the end
|-
| \033[1K || EL || clear the line from beginning to the cursor
|-
| \033[2K || EL || clear the entire line
|-
| \033[S || SU || scroll up one page
|-
| \033[T || SD || scroll down one page
|-
| \033[m || SGR || reset color attributes
|-
| \033[1m || SGR || select bold or intensive color
|-
| \033[''30-37''m || SGR || select foreground color
|-
| \033[''40-47''m || SGR || select background color

|}
The Select Graphic Rendition (or SGR) codes are often concatenated into one command, for example to select bright white on blue background \033[1;37;0;44m. Some terminals understands color codes 90 - 97 (same as 1;30 - 1;37) and 100 - 107 (same as 1;40 - 1;47).

For the foreground and background color codes, these are as follows:

{| class="wikitable"
|-
! FG
! BG
! Name
! RGB (VGA colors)
|-
| 30 || 40 || Black || style="background: #000; color: white" | 0,0,0
|-
| 31 || 41 || Red || style="background: #AA0000; color: white" | 170,0,0
|-
| 32 || 42 || Green || style="background: #00AA00; color: white" | 0,170,0
|-
| 33 || 43 || Brown || style="background: #AA5500; color: white" | 170,85,0
|-
| 34 || 44 || Blue || style="background: #0000AA; color: white" | 0,0,170
|-
| 35 || 45 || Magenta || style="background: #AA00AA; color: white" | 170,0,170
|-
| 36 || 46 || Cyan || style="background: #00AAAA; color: white" | 0,170,170
|-
| 37 || 47 || White || style="background: #AAAAAA; color: black" | 170,170,170
|-
| 1;30 || 1;40 || Bright Black (Gray) || style="background: #555555; color: white" | 85,85,85
|-
| 1;31 || 1;41 || Bright Red || style="background: #FF5555; color: white" | 255,85,85
|-
| 1;32 || 1;42 || Bright Green || style="background: #55FF55; color: black" | 85,255,85
|-
| 1;33 || 1;43 || Bright Yellow || style="background: #FFFF55; color: black" | 255,255,85
|-
| 1;34 || 1;44 || Bright Blue || style="background: #5555FF; color: white" | 85,85,255
|-
| 1;35 || 1;45 || Bright Magenta || style="background: #FF55FF; color: black" | 255,85,255
|-
| 1;36 || 1;46 || Bright Cyan || style="background: #55FFFF; color: black" | 85,255,255
|-
| 1;37 || 1;47 || Bright White || style="background: #FFFFFF; color: black" | 255,255,255
|}

===Detecting Terminal===

First and most, you should detect the size of the screen. This is done by moving the cursor over the screen, and then query the position.

\033[999;999H\033[6n\033[H = your OS to the terminal

\033[''row'';''col''R = the terminal's answer, screen size

To detect the exact terminal type to determine if you can use advanced codes (like \033[38;2;''r'';''g'';''b''m RGB foreground color), well, not possible, as there's no commonly supported identify command. Each terminal has a special command with a specific answer that you can use. Or simply leave it to the user, many OS use the TERM environment variable to select a code set from the termcap database.

==Implementing A Terminal==

Once your OS is sufficiently advanced, you'll probably want to implement a terminal emulator on your OS too so that you can receive connections. For that, you must interpret these sequences, display them on screen accordingly, and convert the keyboard scancodes into ASCII sequences. Under DOS, this exactly what ANSI.SYS does. For Linux, take a look at minicom's source code.

==See Also==
*[[Keyboard]]
*[[VGA Fonts]]
*[[Serial ports]]

===External Links===
*[https://raw.githubusercontent.com/marcv81/termcap/master/termcap.src termcap database]
*[https://vt100.net/docs/vt220-rm/ VT220 Programmer's Manual]
*[https://vt100.net https://vt100.net] many useful docs and specs, no 1. source for video terminals
*[https://www.chiark.greenend.org.uk/~sgtatham/putty/ PuTTY]
*[https://en.wikipedia.org/wiki/Minicom Minicom]
*[https://github.com/V01D-NULL/limine-terminal-port limine terminal port]
*[https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797 ANSI Escape Sequences] (close to) exhaustive list.

[[Category:Human Interface Device]]
[[Category:Network Hardware]]
[[Category:Common Devices]]
````

<a id="w004568"></a>

## W004568 — VT100

Bron-ID: W004568
Namespace: Main (0)
Revisie: 25701 | Datum: 2021-02-19T11:56:26Z
Bron: https://wiki.osdev.org/VT100
Revisie-URL: https://wiki.osdev.org/index.php?title=VT100&oldid=25701
Doorverwijzing: Terminals

````text
#REDIRECT [[Terminals]]
````

<a id="w004569"></a>

## W004569 — File:VT220 Irssi.jpg

Bron-ID: W004569
Namespace: File (6)
Revisie: 25703 | Datum: 2021-02-19T12:09:28Z
Bron: https://wiki.osdev.org/File:VT220_Irssi.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AVT220_Irssi.jpg&oldid=25703

````text

````

<a id="w004570"></a>

## W004570 — File:PuTTY Ubuntu.png

Bron-ID: W004570
Namespace: File (6)
Revisie: 25704 | Datum: 2021-02-19T12:10:16Z
Bron: https://wiki.osdev.org/File:PuTTY_Ubuntu.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3APuTTY_Ubuntu.png&oldid=25704

````text

````

<a id="w004571"></a>

## W004571 — File:Minicom cs xterm.png

Bron-ID: W004571
Namespace: File (6)
Revisie: 25705 | Datum: 2021-02-19T12:11:05Z
Bron: https://wiki.osdev.org/File:Minicom_cs_xterm.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AMinicom_cs_xterm.png&oldid=25705

````text

````

<a id="w004572"></a>

## W004572 — File:UDOS.png

Bron-ID: W004572
Namespace: File (6)
Revisie: 27043 | Datum: 2022-04-20T23:20:10Z
Bron: https://wiki.osdev.org/File:UDOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AUDOS.png&oldid=27043

````text
udos screenshot
````

<a id="w004573"></a>

## W004573 — File:BleskOS-VGA-edition-Alpha-2-computer.jpg

Bron-ID: W004573
Namespace: File (6)
Revisie: 25729 | Datum: 2021-03-04T16:14:43Z
Bron: https://wiki.osdev.org/File:BleskOS-VGA-edition-Alpha-2-computer.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ABleskOS-VGA-edition-Alpha-2-computer.jpg&oldid=25729

````text

````

<a id="w004574"></a>

## W004574 — General Protection Fault

Bron-ID: W004574
Namespace: Main (0)
Revisie: 25746 | Datum: 2021-03-11T23:15:59Z
Bron: https://wiki.osdev.org/General_Protection_Fault
Revisie-URL: https://wiki.osdev.org/index.php?title=General_Protection_Fault&oldid=25746
Doorverwijzing: Exceptions

````text
#REDIRECT [[Exceptions#General_Protection_Fault]]
````

<a id="w004575"></a>

## W004575 — Loading Icons

Bron-ID: W004575
Namespace: Main (0)
Revisie: 28586 | Datum: 2024-01-31T08:23:18Z
Bron: https://wiki.osdev.org/Loading_Icons
Revisie-URL: https://wiki.osdev.org/index.php?title=Loading_Icons&oldid=28586

````text
Once you have graphical mode, and you have learned how to [[Drawing In Protected Mode|display pixels]] and [[PC Screen Font|display fonts]], it is time to learn how to display icons.

==Graphics Modes==
{{Main|Getting VBE Mode Info}}
{{Main|GOP}}
If you haven't set up graphics mode, do that first. You can only display icons on pixel based frame buffers.

==Decoding Images==
To put an icon on screen, all you need to do is iterate on it's pixels, in height lines and width times, copying each pixel one-by-one to the screen. The difficulty is in how to get that pixel array in the first place, because images aren't stored like that on disk.

===Windows BMP===
Simply put, do NOT use. You'll run into strange problems like some editors save OS/2 bitmap, not Windows bitmap (slightly different, but enough to give you a headache), and that the image is upside down, channel order is not what you expect etc. Unless you're developing on Windows (where the tools only create Windows BMP) using this format is just asking for trouble.

===Targa===
A much better choice would be Targa format (.tga). It is a very very simple format, used by many game engines (like Quake) too. Essentially you can save uncompressed bitmap images with a 18 bytes header:
<source lang="c">
typedef struct {
  unsigned char magic1;             // must be zero
  unsigned char colormap:           // must be zero
  unsigned char encoding;           // must be 2
  unsigned short cmaporig, cmaplen; // must be zero
  unsigned char cmapent;            // must be zero
  unsigned short x;                 // must be zero
  unsigned short y;                 // image's height
  unsigned short h;                 // image's height
  unsigned short w;                 // image's width
  unsigned char bpp;                // must be 32
  unsigned char pixeltype;          // must be 40
} __attribute__((packed)) tga_header_t;
</source>This is followed by the data, each pixel on 4 bytes: blue, green, red, alpha channels in order. This simple. Many image editors support this format (like The GIMP), or you can use [https://imagemagick.org/index.php ImageMagick] CLI tool to convert any image format into .tga with a simple command (available in all Linux distributions).
<source lang="bash">
$ convert image.png image.tga
</source>When saving with The GIMP, first convert the image to RGB (Image / Mode / RGB). Then add Alpha channel (Layer / Transparency / Add Alpha Channel). If these options are inactive, that means your image is already RGB or has alpha. Finally choose Export As... and enter a filename ending with ".tga". In the popup, uncheck "RLE compression" and select origin "Top". 

If you want to support all TGA options, including palette images and RLE compression, then that's still pretty simple. Just use the following code snippet:
<source lang="c">
/**
 * Parse TGA format into pixels. Returns NULL or error, otherwise the returned data looks like
 *   ret[0] = width of the image
 *   ret[1] = height of the image
 *   ret[2..] = 32 bit ARGB pixels (blue channel in the least significant byte, alpha channel in the most)
 */
unsigned int *tga_parse(unsigned char *ptr, int size)
{
    unsigned int *data;
    int i, j, k, x, y, w = (ptr[13] << 8) + ptr[12], h = (ptr[15] << 8) + ptr[14], o = (ptr[11] << 8) + ptr[10];
    int m = ((ptr[1]? (ptr[7]>>3)*ptr[5] : 0) + 18);

    if(w<1 || h<1) return NULL;

    data = (unsigned int*)malloc((w*h+2)*sizeof(unsigned int));
    if(!data) return NULL;

    switch(ptr[2]) {
        case 1:
            if(ptr[6]!=0 || ptr[4]!=0 || ptr[3]!=0 || (ptr[7]!=24 && ptr[7]!=32)) { free(data); return NULL; }
            for(y=i=0; y<h; y++) {
                k = ((!o?h-y-1:y)*w);
                for(x=0; x<w; x++) {
                    j = ptr[m + k++]*(ptr[7]>>3) + 18;
                    data[2 + i++] = ((ptr[7]==32?ptr[j+3]:0xFF) << 24) | (ptr[j+2] << 16) | (ptr[j+1] << 8) | ptr[j];
                }
            }
            break;
        case 2:
            if(ptr[5]!=0 || ptr[6]!=0 || ptr[1]!=0 || (ptr[16]!=24 && ptr[16]!=32)) { free(data); return NULL; }
            for(y=i=0; y<h; y++) {
                j = ((!o?h-y-1:y)*w*(ptr[16]>>3));
                for(x=0; x<w; x++) {
                    data[2 + i++] = ((ptr[16]==32?ptr[j+3]:0xFF) << 24) | (ptr[j+2] << 16) | (ptr[j+1] << 8) | ptr[j];
                    j += ptr[16]>>3;
                }
            }
            break;
        case 9:
            if(ptr[6]!=0 || ptr[4]!=0 || ptr[3]!=0 || (ptr[7]!=24 && ptr[7]!=32)) { free(data); return NULL; }
            y = i = 0;
            for(x=0; x<w*h && m<size;) {
                k = ptr[m++];
                if(k > 127) {
                    k -= 127; x += k;
                    j = ptr[m++]*(ptr[7]>>3) + 18;
                    while(k--) {
                        if(!(i%w)) { i=((!o?h-y-1:y)*w); y++; }
                        data[2 + i++] = ((ptr[7]==32?ptr[j+3]:0xFF) << 24) | (ptr[j+2] << 16) | (ptr[j+1] << 8) | ptr[j];
                    }
                } else {
                    k++; x += k;
                    while(k--) {
                        j = ptr[m++]*(ptr[7]>>3) + 18;
                        if(!(i%w)) { i=((!o?h-y-1:y)*w); y++; }
                        data[2 + i++] = ((ptr[7]==32?ptr[j+3]:0xFF) << 24) | (ptr[j+2] << 16) | (ptr[j+1] << 8) | ptr[j];
                    }
                }
            }
            break;
        case 10:
            if(ptr[5]!=0 || ptr[6]!=0 || ptr[1]!=0 || (ptr[16]!=24 && ptr[16]!=32)) { free(data); return NULL; }
            y = i = 0;
            for(x=0; x<w*h && m<size;) {
                k = ptr[m++];
                if(k > 127) {
                    k -= 127; x += k;
                    while(k--) {
                        if(!(i%w)) { i=((!o?h-y-1:y)*w); y++; }
                        data[2 + i++] = ((ptr[16]==32?ptr[m+3]:0xFF) << 24) | (ptr[m+2] << 16) | (ptr[m+1] << 8) | ptr[m];
                    }
                    m += ptr[16]>>3;
                } else {
                    k++; x += k;
                    while(k--) {
                        if(!(i%w)) { i=((!o?h-y-1:y)*w); y++; }
                        data[2 + i++] = ((ptr[16]==32?ptr[m+3]:0xFF) << 24) | (ptr[m+2] << 16) | (ptr[m+1] << 8) | ptr[m];
                        m += ptr[16]>>3;
                    }
                }
            }
            break;
        default:
            free(data); return NULL;
    }
    data[0] = w;
    data[1] = h;
    return data;
}
</source>This will decode the most common TGA variants according to the Targa specification. It will return pixels in a format that's most common for frame buffers.

===PNG, JPEG and other "professional" formats===
Using these is not a simple thing to do. Definitely more complex than those ca. 80 SLoC above. You could port libpng, libjpeg, etc. but that's not an easy task either. They usually have their dependencies on their own, including setjmp and longjmp, and it is not trivial to use libpng's API for example.

Instead, use [https://github.com/nothings/stb/blob/master/stb_image.h stb_image.h]. This is a single header file (no library linking needed, just libc), which is easy to use, for example the code to load PNG images looks like this:
<source lang="c">
#define STBI_IMPLEMENTATION
#define STBI_ONLYPNG
#include <stb_image.h>

/**
 * Parse PNG format into pixels. Returns NULL or error, otherwise the returned data looks like
 *   ret[0] = width of the image
 *   ret[1] = height of the image
 *   ret[2..] = 32 bit ARGB pixels (blue channel in the least significant byte, alpha channel in the most)
 */
unsigned int *png_parse(unsigned char *ptr, int size)
    int i, w, h, f;
    unsigned char *img, *p;
    stbi__context s;
    stbi__result_info ri;
    s.read_from_callbacks = 0;
    s.img_buffer = s.img_buffer_original = ptr;
    s.img_buffer_end = s.img_buffer_original_end = ptr + size;
    ri.bits_per_channel = 8;
    img = p = (unsigned char*)stbi__png_load(&s, (int*)&w, (int*)&h, (int*)&f, 1, &ri);
    data = (unsigned int*)malloc((w*h+2)*sizeof(unsigned int));
    if(!data) { free(img); return NULL; }
    // convert the returned image into frame buffer format
    for(i = 0; i < w * h; i++, p += f)
        switch(f) {
            case 1: data[2 + i] = 0xFF000000 | (p[0] << 16) | (p[0] << 8) | p[0]; break;
            case 2: data[2 + i] = (p[1] << 24) | (p[0] << 16) | (p[0] << 8) | p[0]; break;
            case 3: data[2 + i] = 0xFF000000 | (p[0] << 16) | (p[1] << 8) | p[2]; break;
            case 4: data[2 + i] = (p[3] << 24) | (p[0] << 16) | (p[1] << 8) | p[2]; break;
        }
    free(img);
    data[0] = w;
    data[1] = h;
    return data;
}
</source>One downside is, stb_image.h doesn't return pixels in the same format as the typical frame buffer expects. You must convert those into 32 bit ARGB pixels.

==See Also==
*[[Drawing In Protected Mode]]
*[[Double Buffering]]

===External Links===
*[https://www.gamers.org/dEngine/quake3/TGA.txt TGA specification]
*[https://web.archive.org/web/20070923112451/http://nothings.org/stb_image.h stb_image.h] header only implementation for loading PNG, JPEG, etc. image formats

[[Category:Graphical UI]]
[[Category:Tutorials]]
````

<a id="w004577"></a>

## W004577 — File:Soso-v0.3-gears.png

Bron-ID: W004577
Namespace: File (6)
Revisie: 25787 | Datum: 2021-04-03T14:07:17Z
Bron: https://wiki.osdev.org/File:Soso-v0.3-gears.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ASoso-v0.3-gears.png&oldid=25787

````text

````

<a id="w004578"></a>

## W004578 — Talk:ATA PIO Mode

Bron-ID: W004578
Namespace: Talk (1)
Revisie: 25792 | Datum: 2021-04-09T03:45:21Z
Bron: https://wiki.osdev.org/Talk:ATA_PIO_Mode
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AATA_PIO_Mode&oldid=25792

````text
What are the standard IO ports for ATA? In Bochs they offer four ATA controllers but only the ports 0x1fx are included here. --[[User:Mattf|Matt F]] 22:45, 8 April 2021 (CDT)
````

<a id="w004579"></a>

## W004579 — File:LongModePagingVirtualAddressExample.png

Bron-ID: W004579
Namespace: File (6)
Revisie: 25809 | Datum: 2021-04-15T17:39:17Z
Bron: https://wiki.osdev.org/File:LongModePagingVirtualAddressExample.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ALongModePagingVirtualAddressExample.png&oldid=25809

````text
An example of how virtual addresses are translated into physical addresses.
````

<a id="w004580"></a>

## W004580 — User talk:Mishatheosmaker

Bron-ID: W004580
Namespace: User talk (3)
Revisie: 25825 | Datum: 2021-04-30T17:25:06Z
Bron: https://wiki.osdev.org/User_talk:Mishatheosmaker
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AMishatheosmaker&oldid=25825

````text
My personal talk page. It took me a few years to figure out how to make one.
````

<a id="w004581"></a>

## W004581 — User:Mishatheosmaker

Bron-ID: W004581
Namespace: User (2)
Revisie: 26268 | Datum: 2021-08-06T10:47:10Z
Bron: https://wiki.osdev.org/User:Mishatheosmaker
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMishatheosmaker&oldid=26268

````text
Hi, my name is Misha. I am an operating system developer. I prefer to write drivers for hardware such as a USB driver.
````

<a id="w004582"></a>

## W004582 — User:Qoob

Bron-ID: W004582
Namespace: User (2)
Revisie: 25832 | Datum: 2021-05-05T00:02:48Z
Bron: https://wiki.osdev.org/User:Qoob
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AQoob&oldid=25832

````text
qoob is a retard
````

<a id="w004583"></a>

## W004583 — User:Iguessthislldo/libx86emu

Bron-ID: W004583
Namespace: User (2)
Revisie: 25845 | Datum: 2021-05-06T19:53:19Z
Bron: https://wiki.osdev.org/User:Iguessthislldo/libx86emu
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AIguessthislldo%2Flibx86emu&oldid=25845

````text
{{Rating|3}}

'''libx86emu''' is a C library for emulating a [[real mode]] or a limited [[protected mode]] [[x86]] CPU. Like [[Virtual 8086 Mode]], it can be used to call [[BIOS]] functions, such as the ones for using [[VBE]], without dropping back into real mode. Unlike virtual 8086 mode though, it can be used in protected mode and [[x86-64|long mode]]. It might also be preferred because it doesn't require as much low level integration with the OS as the virtual 8086 mode does. It was created by the [[wikipedia:SciTech Software|SciTech Software Inc.]] in the 1990's. It was integrated into the [[wikipedia:XFree86|XFree86 server]], which was later forked to create the [[wikipedia:X.org server|X.org server]]. In addition to [https://cgit.freedesktop.org/xorg/xserver/tree/hw/xfree86/x86emu the fork that still exists in X.org], there is [https://github.com/wfeldt/libx86emu a separate fork on GitHub] that is maintained as part of OpenSUSE Linux.

While it's possible to use the libx86emu found in X.org, this article will assume the OpenSUSE fork is being used, specifically version 3.1. It will also assume the library is going to be integrated into the kernel and the source code of the library isn't going to be modified.

== Dependencies ==

The following C headers and definitions are required to build:

* <code>stdio.h</code>
** Can include the <code>stddef.h</code> freestanding header to get <code>size_t</code>.
** <code>int vsnprintf(char* buffer, size_t bufsz, const char* format, va_list vlist</code>
*** This can be a stub implementation, but is needed for logging to be fully functional.

* <code>stdlib.h</code>
** Can include the <code>stddef.h</code> freestanding header to get <code>NULL</code> and <code>size_t</code>.
** <code>void* malloc(size_t size)</code>
** <code>void* calloc(size_t num, size_t size)</code>
** <code>void free(void* ptr)</code>

* <code>string.h</code>
** <code>char* strcat(char* dest, const char* src);</code>
*** This can be a stub implementation, but is needed for logging to be fully functional.

* <code>sys</code>
** <code>io.h</code>
*** These could be stubs depending on how emulation is setup to perform port I/O.
*** <code>unsigned char inb(unsigned short int port)</code>
*** <code>unsigned short int inw(unsigned short int port)</code>
*** <code>unsigned int inl(unsigned short int port)</code>
*** <code>void outb(unsigned char value, unsigned short int port)</code>
*** <code>void outw(unsigned short int value, unsigned short int port)</code>
*** <code>void outl(unsigned int value, unsigned short int port)</code>

* <code>time.h</code>
** <code>time_t</code>
** <code>time_t time(time_t* arg)</code>
*** This can be a stub implementation, only needed to use the <code>X86EMU_RUN_TIMEOUT</code> flag to limit the time a emulation runs.

== Building ==

All the C files in the root of the source tree are part of the library and should be built together with the headers outlined above. One thing to note is that the undefined behavior sanitizer (UBsan), which is the <code>-fsanitize=undefined</code> option in Clang and GCC, should be not be used with libx86emu because some of the emulated instructions might set it off during execution.

=== Building with the Zig Build System ===

If using the Zig build system to build the library, it always passes <code>-fsanitize=undefined</code> for C code in a debug mode, so it will need to be disabled. It can be disabled by passing <code>-fsanitize-blacklist=BLACKLIST_FILE</code> to the C compile options, where <code>BLACKLIST_FILE</code> is a path to a file relative to the Zig build file. This file should contain something like:

  [undefined]
  src:*/libx86emu/*

This assumes the C files are in a directory called <code>libx86emu</code>.

== Example Usage ==

'''TODO'''

== Also See ==

=== Forum Posts ===
* [https://forum.osdev.org/viewtopic.php?f=1&t=31388 Question about libx86emu]

=== External Links ===
* [https://github.com/wfeldt/libx86emu libx86emu on GitHub]
* [https://cgit.freedesktop.org/xorg/xserver/tree/hw/xfree86/x86emu libx86emu in X.org server]
````

<a id="w004585"></a>

## W004585 — PSF

Bron-ID: W004585
Namespace: Main (0)
Revisie: 25853 | Datum: 2021-05-10T20:23:26Z
Bron: https://wiki.osdev.org/PSF
Revisie-URL: https://wiki.osdev.org/index.php?title=PSF&oldid=25853
Doorverwijzing: PC Screen Font

````text
#REDIRECT [[PC_Screen_Font]]
````

<a id="w004586"></a>

## W004586 — File:LF OS-20210519.png

Bron-ID: W004586
Namespace: File (6)
Revisie: 25871 | Datum: 2021-05-19T13:37:24Z
Bron: https://wiki.osdev.org/File:LF_OS-20210519.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ALF_OS-20210519.png&oldid=25871

````text
Screenshot of LF OS kernel initialization in qemu, after that it would jump to userspace - weren't there a <code>while(1);</code> just before enabling interrupts - this is currently the best thing to screenshot.
````

<a id="w004587"></a>

## W004587 — File:FoxOS.png

Bron-ID: W004587
Namespace: File (6)
Revisie: 25873 | Datum: 2021-05-19T18:33:09Z
Bron: https://wiki.osdev.org/File:FoxOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AFoxOS.png&oldid=25873

````text
The FoxOS logo
````

<a id="w004588"></a>

## W004588 — User:Imdabigboss

Bron-ID: W004588
Namespace: User (2)
Revisie: 25875 | Datum: 2021-05-19T18:39:14Z
Bron: https://wiki.osdev.org/User:Imdabigboss
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AImdabigboss&oldid=25875

````text
Hello! :)

I'm a hobbyist OS developer, I'm trying to learn new things and explore deeper into computers.

My current projects:
* [[User:ImDaBigBoss/FoxOS|FoxOS]], a collaborative open source project.
````

<a id="w004589"></a>

## W004589 — User:ImDaBigBoss/FoxOS

Bron-ID: W004589
Namespace: User (2)
Revisie: 25878 | Datum: 2021-05-19T18:43:02Z
Bron: https://wiki.osdev.org/User:ImDaBigBoss/FoxOS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AImDaBigBoss%2FFoxOS&oldid=25878

````text
Our goal is to develop an operation system that focuses on the terminal, performence and reliability. We hope to provide you with a functionnal, performant and stable OS.

And we do that together, '''everyone''' can help!

Here is the link to our [https://github.com/TheUltimateFoxOS/FoxOS github repository] if you want to help! Along with the [https://theultimatefoxos.github.io/ project's website]

[[File:FoxOS.png|200px]]
````

<a id="w004590"></a>

## W004590 — File:Mu-screenshot.png

Bron-ID: W004590
Namespace: File (6)
Revisie: 25887 | Datum: 2021-05-21T18:28:09Z
Bron: https://wiki.osdev.org/File:Mu-screenshot.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AMu-screenshot.png&oldid=25887

````text
A screenshot of the Mu computer showing a Lisp-based prototyping environment with two columns of function definitions on the left and a REPL on the right. The REPL has just rendered a circle in a little window.
````

<a id="w004591"></a>

## W004591 — User:Nullplan/Random Number Generator

Bron-ID: W004591
Namespace: User (2)
Revisie: 26174 | Datum: 2021-07-03T09:47:18Z
Bron: https://wiki.osdev.org/User:Nullplan/Random_Number_Generator
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANullplan%2FRandom_Number_Generator&oldid=26174

````text
A '''random number generator''' is a function that returns an erratic sequence of numbers. The goal is to simulate random events, such as a dice roll, for various purposes. Ideally, each time the function is called, each possible output is equally likely to be returned, but this does not always happen. Of interest in this community are particularly in-kernel RNGs.

=Entropy=
Computers are deterministic devices. If the program is the same and all inputs are the same, the same result will be calculated every time. How, then, can a computer generate random numbers? The computer cannot, but the physical world around it can. Many physical events are random to some degree, or more technically, have some degree of entropy. A sound recording will contain some level of noise even in the best shielded recording rooms on the planet. Humans type on keyboards with an inconsistent timing, and type quasi-random text like the article you are currently reading. Even other computers can provide entropy: While most network traffic is entirely machine generated, the exact relative timing of network packets in the end derives from the moment the power button was pressed, another human action. Therefore the timing and nature of input events can provide us with some entropy.

The state of a random number generator can be seen as an '''entropy buffer'''. Calculations based on the entropy buffer will have as many different outputs as the entropy buffer has different states. To deal with the combinatoric explosion, entropy is typically measured in bits. If one bit of entropy has been gathered, the entropy buffer can only be in one of two states. An entropy buffer is said to be full if as much entropy has been gathered as the buffer is large. At that point, any state of the buffer is possible, and therefore any calculation based on the buffer will have any possible result.

==Sources of entropy==
Generally the timing and nature of all input device events can be sources of entropy. As mentioned already, this includes keyboards, mice, and gamepads, but also network cards and hard disks. The reason for the latter is the behavior of air molecules on the surface of the platter, which cause random delays to hard disk commands. However, there is also dedicated hardware to generate entropy, such as avalanche diodes. And x86 contains a dedicated instruction to read a random number from an unspecified source.

===Timing===
When using timing as entropy source, the timestamp read should be as precise as possible. The TSC works well for this. Gauging the entropy gained from that operation requires knowledge of the timing window for the event to occur in and the tick rate of the TSC. For example, if a TSC has a tick rate of 3 GHz and an event has a 10ms window to occur, then the TSC read can have any one of 30 million values, which means the entropy gained from this is ca. 24.8 bits. Were the TSC slower, only 1 GHz, then the entropy would only be ca. 23.2 bits.

===x86 randomness instructions===
There is an RDRAND instruction in the x86 instruction set. Support for it is indicated by ECX bit 30 in CPUID function 1, subfunction 0. So support might be tested like this:
<source lang="asm">
  xor eax, eax
  xor ecx, ecx
  inc eax
  cpuid
  bt ecx, 30 ; carry flag now indicates support for RDRAND</source>
RDRAND reads a random number into a 16-, 32-, or 64-bit register. It might not be successful, however, and sets the carry flag to indicate success. Since no guarantees exist how quickly a new random number might become available, it is necessary to cap the number of retries, such as like this:
<source lang="asm">
  mov ecx, 100
.retry:
  rdrand eax
  jc .success
  loop .retry
.failure:
  ; handle failed attempt
.success:
  ; handle successful read
</source>A good use of this instruction might be to periodically or otherwise regularly read a random number and add to the entropy pool.

There is also an RDSEED instruction. Support is indicated by EBX bit 18 from CPUID function 7, subfunction 0. It has the same interface as RDRAND, but internally uses a different random number generator, that is supposed to make exhaustion less likely. Still, RDSEED can fail just like RDRAND. See the Intel documentation for the precise differences and assurances made.

==Adversarial entropy==
If an adversary can somehow observe the state of the entropy pool and contribute their own entropy into the pool, then it is possible that they would provide entropy in such a way as to force the entropy pool into a lower entropy state. A simple example would be an entropy source periodically checking if a certain bit in the pool is set, and then providing entropy until it is clear. This way, for most of the time, that given bit is clear, and the number of possible states the entropy buffer can be in is halved. A more sophisticated attack is described on DJB's blog: https://blog.cr.yp.to/20140205-entropy.html

=Low-entropy PRNGs=
For non-cryptographic applications, it is generally enough for a random number generator to simply produce a different sequence every time, and have it be reasonably erratic. Cryptographic security (see next section) is not required. For these, it is generally not necessary to gather entropy continuously from various sources; it is enough to seed them once from a value that is different each time the program is run (e.g. system time). The advantage of these algorithms is that they are fast and simple, and do not require kernel support. Therefore they are often implemented in language runtime libraries. The drawback is, of course, that they are not cryptographically secure. Another disadvantage is that their state is not shared across the entire system. Therefore it is possible for two different processes to accidentally happen upon the same sequence and generate the exact same sequence in lock-step.
==Linear-feedback shift register==
One of the earliest methods for calculating erratic sequences, a linear-feedback shift register consists of a shift register with a feedback (a line leading from the shift-out bit to the input bit) and some linear function being applied to it. In Fibonacci configuration, it is the input bit that has a linear function applied to it, while in Galois configuration, the output bit is fed directly back into the input, but triggers some XOR gates in line with the shift registers. The output of the function is a single bit, so in order to generate more, multiple calls are required. Simple example:
<source lang="c">static uint32_t seed = 0xcafebabe; /* anything nonzero will do */
void set_seed(uint32_t s) {
  if (!s)
    seed = 0xcafebabe;
  else
    seed = s;
}

int fibonacci_lfsr(void)
{
  /* taps in positions 32, 31, 28 */
  uint32_t bitIn = (seed ^ (seed >> 1) ^ (seed >> 4)) & 1;
  int bitOut = seed & 1;
  seed = (seed >> 1) | (bitIn << 31);
  return bitOut;
}
int galois_lfsr(void)
{
  int bitOut = seed & 1;
  seed = (seed >> 1) ^ (seed & 1? 0xc8000000ul : 0);
  return bitOut;
}
</source>

==Linear congruential generator==
A very popular choice, the linear congruential generator has been the mainstay of C runtime libraries everywhere. For example, at the time of this writing, musl, klibc, glibc, pdclib, and uclibc all use an LCG to implement the standard rand() function by default. In fact, several use the exact same one, for example given by pdclib:
<source lang="c">int rand( void )
{
    _PDCLIB_seed = _PDCLIB_seed * 1103515245 + 12345;
    return ( int )( _PDCLIB_seed / 65536 ) % 32768;
}</source>Mathematically, the seed is multiplied with a constant, then another constant is added (this would be the linear part), and the whole thing is evaluated in some remainder class ring (that would be the congruential part). The modulus of that ring is generally a power of two, and generally one more than the maximum value the type of the seed can hold. This is because that ring is implicit.

Notice how both constants mentioned are odd numbers. This is required for maximum period length. However, since both are odd, a peculiar thing is happening: If the seed on input is odd, on output it will be even, and if it is even, on output it will be odd. Therefore its lowest bit is not random at all, it will always simply toggle. This is one of the well-known properties of LCGs, that the low bits are significantly less random than the high bits. It is therefore understandable that most libraries will modify the output to only return the high bits of the state.
==Wichmann-Hill==
In 1982, Brian Wichmann and David Hill proposed to combine three linear congruential generators, then normalizing and summing the results to get a number uniformly distributed between 0 and 1. A common instance is:
<source lang="c">static uint16_t seed[3]; /* seed with numbers between 1 and 30000 */
float wichmann_hill(void) {
  seed[0] = (171 * seed[0]) % 30269;
  seed[1] = (172 * seed[1]) % 30307;
  seed[2] = (170 * seed[2]) % 30323;
  return fmod(seed[0] / 30269.0 + seed[1] / 30307.0 + seed[2] / 30323.0, 1.0);
}</source>This version is known to have a period of just shy of of seven trillion (the least common multiple of 30268, 30306, and 30322).
==Mersenne Twister==
The Mersenne Twister uses a generalized feedback shift register and a huge amount of state to provide a pseudo-random function with an utterly giant period. The name comes from the fact that that period happens to be a Mersenne prime. Implementation according to Wikipedia:
<source lang="c">#include <stddef.h>
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

=Cryptographic security=
In order to understand the following section, we first should review what cryptographic security means with respect to random number generators. Cryptographers distinguish two types of security: Information-theoretic security and computational security. The former of these is the stronger claim. If a cryptosystem is information-theoretically secure, an attacker cannot say anything about an encrypted message without owning the key. Such cryptosystems are rare. Since the attacker must not be able to say '''anything''', everything including the length must be hidden. How can one even hide the length of a message? The only choices are to not send the message at all, which defeats the purpose of a cryptosystem, or to send an infinitely long message. So once you start sending, you can never stop. If you ever stop sending, the attacker has at least an upper bound on the length of the information. In practice, you can probably stop when you die, since then it will no longer matter, but such a system still is not an idea for the future. And the length can matter a lot. Imagine a secret service looking for a video file. And they intercept a 64 byte encrypted message. Is that message the video they are looking for?

Because information-theoretic security is impractical, and usually too strong a claim for the needs of the users, most cryptosystems only deliver computational security. Certain data about the message is revealed to everyone: The length to some granularity, and often the metadata (when was it sent? Who sends it to whom?). And the remaining data can in theory be recovered, but it is computationally infeasible to do so. For an RNG, cryptographic security means that no attacker being able to observe the output of the function before or after the victim called it shall be able to say anything about the data the victim got from their calls. This requirement can be fulfilled by real-world randomness generators, but also by cryptographically secure pseudo-RNGs that were seeded with entropic data.

=High-entropy PRNGs=
'''Notice:''' The code in this section is illustrative. Actual implementations should not use it.

Imagine, if you will, a fixed-size entropy buffer. It is initialized to some value. Now some entropic data comes along. How can this entropy be captured in the entropy buffer without destroying the entropy already in there, especially without knowing how entropic the data really is? A simple idea would be to append the new data to the entropy buffer, but that would lead to an endlessly growing entropy buffer. But one of the properties of cryptographically secure hash functions is compression, right? So we could just hash the concatenation of the entropy buffer and the new data. Or, somewhat more formally:
<source lang="c">static unsigned char entropy_pool[SHA_1_HASH_SIZE];

void mix_in_entropy(const void *data, size_t len)
{
  SHA1_context ctx;
  sha1_init(&ctx);
  sha1_add_data(&ctx, entropy_pool, sizeof entropy_pool);
  sha1_add_data(&ctx, data, len);
  sha1_finalize(&ctx, entropy_pool);
}</source>
This fulfills the goal of having a fixed size buffer. The hashing process also distributes the entropy over the entire buffer, which is a desirable trait. In practice, cryptographic security is not required at this stage, and e.g. Linux uses a non-secure hash function for this purpose.

Now we have a constant sized buffer, with contents growing more and more unpredictable as entropy is added. But how do we actually extract random numbers from it? If we just returned the entropy buffer, we would be giving away the crown jewels. The entropy buffer must be kept secret, because knowing it can massively narrow down the possible random numbers any future consumer is getting. Which compromises cryptographic security.

However, we can hash the entropy pool. The hash of the entropy pool is as random as the entropy pool itself and allows no information about the contents of the pool itself. That is a good idea, but has multiple serious flaws: If the output becomes larger than the size of the used hash function, it will start to repeat. Also, if there are multiple calls in close succession, they will all get the same "random" data. Both of these issues can be remedied by mixing the output back into the entropy pool:

<source lang="c">void get_entropy(void *data, size_t len)
{
  SHA1_context ctx;
  unsigned char buf[SHA_1_HASH_SIZE];
  while (len)
  {
    sha1_init(&ctx);
    sha1_add_data(&ctx, entropy_pool, sizeof entropy_pool);
    sha1_finalize(&ctx, buf);
    mix_in_entropy(buf, sizeof buf);
    size_t cpylen = min(len, sizeof buf);
    memcpy(data, buf, cpylen);
    data = (char*)data + cpylen;
    len -= cpylen;
  }
}</source>

It is at this point that the distinction between pseudo-RNG and "real" RNG really breaks down. Here we have a pseudo-RNG being constantly reseeded with real-world entropic data. It returns a nearly endless sequence of random data and will only repeat if more bits can be read than the entropy buffer has available states before more external entropy is mixed in. It can even be seeded from a "real" RNG. So what is it?

Of course this implementation is not perfect. It lacks any kind of synchronization, for one thing. Using it in a multi-threaded environment as is would introduce data races, which are undefined behavior. And it is not a simple topic, since you would want mix_in_entropy() to be safe for interrupt handlers in most cases, so a simple big mutex will not necessarily do the trick. Also, the implementer of these lines is not a cryptographer, so I am certain I made some rookie mistakes. Cryptography can be counter-intuitive at times.


=Further Reading=
* [https://www.2uo.de/myths-about-urandom/] - Further information about the two randomness devices in Linux, and general cryptography advice.
* [https://github.com/ryco117/Fortuna] - An implementation of Fortuna, a CSPRNG.
````

<a id="w004592"></a>

## W004592 — User talk:Neon

Bron-ID: W004592
Namespace: User talk (3)
Revisie: 25914 | Datum: 2021-05-24T01:27:07Z
Bron: https://wiki.osdev.org/User_talk:Neon
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ANeon&oldid=25914

````text

````

<a id="w004593"></a>

## W004593 — User:Neon

Bron-ID: W004593
Namespace: User (2)
Revisie: 26087 | Datum: 2021-06-05T04:38:54Z
Bron: https://wiki.osdev.org/User:Neon
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANeon&oldid=26087

````text
= Neptune SDK =

The Neptune SDK (NSDK) consists of the Neptune Build Environment (NBE) and tool chain. This includes NASM, NCC, NLINK, NLIB, NDBG, NMAKE, and NBUILD. The entire system is to be built with NSDK.

''This is an ongoing project.''

= Neptune Boot =

The loader for Neptune is configurable, extendable; supporting BIOS and EFI firmware and runs in 32 bit protected mode with a purposed plan to provide a 64 bit build as well even if just for EFI. It implements the Multiboot 1 specification.

== Memory Management ==

Memory management is done in two layers. This is to separate the firmware specific code from the core program.

'''Layer 1: Memory Driver Objects'''

There is a Memory Driver Object for EFI and BIOS. For EFI builds, we request free pages from the firmware. For BIOS builds, we reserve an area of space and use a bitmap allocator to manage free frames of memory.

'''Layer 2: Zone allocation'''

Sometimes it is necessary for the BIOS firmware to be able to access allocated memory. For this reason, we use a Zone Allocation scheme: We allocate from a Zone (such as Low Memory or Standard Memory). Standard Memory is > 1MB. A Free List is used to allocate and free blocks of memory.

== Driver Objects and Device Objects ==

The Neptune Boot Library (NBL) defines the core framework for Driver Objects and Device Objects. There are some standard Driver Objects for file systems, keyboard mapping, and a few others; and firmware-specific Driver Objects such as for disk, memory, video, and others. Firmware Driver Objects are defined in nbl_bios and nbl_efi so they can be easily linked.

Driver Objects implement a set of standard functions. Driver objects operate and create Device Objects. NBOOT either calls a driver service directly through a function pointer or through the Device Object the driver is attached to. There is a special case for partitioning support: Partitions are implemented as Disk Driver Objects that create Disk Device Objects for each partition. When NBOOT reads from a device like "disk(1)part(2)" it calls the Disk Driver Object associated with the Device Object "disk(1)part(2)" which passes the command to an attached Device Object to handle for "disk(1)". This special case is why we support Device Objects having an attached secondary Device Object as well. Almost everything is done through Device Objects for abstraction: it allows NBOOT to support virtually any disk and file system type by just adding the supported Driver Object to the NBL.

== BIOS Build: How do we call the BIOS? ==

The Driver Objects in nbl_bios need to drop the system to real mode to call the BIOS. We implemented a general I/O Services function -- io_services -- that is used by the BIOS Driver Objects. This is done by dropping the system to 16 bit protected mode then 16 bit real mode. We pass a Register Set to our I/O Services function that provides the Input to the BIOS call. io_services is modeled after the dos.h int86 function. Self modifying code is used to implement the general INT n instruction needed. In summary, the algorithm is the following:

# Save IDTR and ESP
# Load real mode IVTR into IDTR
# Enter 16 bit protected mode
# Enter 16 bit real mode
# Call BIOS
# Restore previous IDTR
# Enter 32 bit protected mode

== BIOS Build: How do we boot? ==

In BIOS builds, we have the boot sector codes ("Stage 1"), NSTART ("Stage 1.5") and NBOOT ("Stage 2"). NSTART is only required for BIOS builds. NSTART.SYS is prepended to NBOOT.EXE to produce the file "NBOOT". When Stage 1 completes, NSTART locates and copies NBOOT to its expected load address, and executes it in 32 bit protected mode. NSTART operates primarily in 32 bit protected mode itself. It is expected that "Stage 1" will load the entire NBOOT image into memory so NSTART does not need to load anything.

= Neptune Executive =

== Memory Management ==

A lot of careful design has been put into the mechanics of the memory management involved in the kernel. As of this writing, it is the largest part of the kernel. We wanted to make sure everything was covered and be extendable. We hope the information presented here can provide some ideas to others that are working on their own memory management designs and putting everything together.

* '''Terms'''
** '''PFN''' - Page Frame Number
** '''VAD''' - Virtual Address Descriptor
** '''LPB''' - Loader Parameter Block
** '''PTE''' - Page Table Entry
*** '''Standard PTE''' - Active x86 PTE mapped to a PFN
*** '''System PTE''' - Used for page swapping / demand loading
*** '''Free PTE''' - Used for allocating kernel PTE's

=== VAD Tree: How do we allocate from Low Memory? ===

Low Memory is treated special in order to accommodate legacy hardware devices and firmware service requirements. These requirements can require specific physical address alignment criteria, addressing limitations (like must be <640k) and must be in contiguous pages. We use the same technique that is to be used with User Space with the use of Virtual Address Descriptors (VAD's) to manage regions of the address space.

Low Memory regions are mapped either through the System Pool (Kernel Space) or per-process VAD Tree (User Space).

=== PFN Database: How do we allocate from Standard Memory? ===

When the system starts up, either paging is not enabled (NBOOT 32 bit) or paging is enabled and all RAM is accessible (NBOOT 64 bit). In both cases, we have access to all of the physical address space. We manage physical frames by creating a Page Frame Number (PFN) free stack within the physical frames themselves. This is what we call the "PFN Database". At any time we can allocate and free PFN's using this Database.

A Header is stored in all '''free allocatable frames''' in physical memory: a global pointer stores the top of the stack. This is done early during the startup process. An "allocatable" frame is a block of memory that is currently not in use by the Kernel, Modules, or other structures. The Kernel may free some of this memory for use. The PFN Database is used to allocate from "Standard Memory" above 1MB physical.

The most important services are:

* MmScanSystemMemoryMap
* MmFreeFrame
* MmAllocateFrame
* MmGetFreeFrame

MmScanSystemMemoryMap scans the memory map and builds the PFN Database within the free frames. The memory map is obtained from the Loader Parameter Block (LPB) provided from NBOOT. MmGetFreeFrame returns the PFN on the top of the stack, which is the next available PFN. MmAllocFrame allocates the next available PFN. MmFreeFrame returns the PFN back to the free stack.

=== PFN Database: How do we allocate with paging? ===

When paging is enabled, we can only update physical frames through a virtual address. This requires careful interaction between the mapping (allocating a Page) and the PFN Database. The process is to get what would be the next PFN without actually allocating it yet. Then we map this PFN to some free Page. And then we call the PFN Database to allocate using this virtual address. That is, the process is, in this order:

* Free PFN = MmGetFreeFrame
* Map PFN to a virtual address. Do NOT write to this address yet
* Call MmAllocateFrame (address)

In other words, we map a page to the next PFN so that we can access the physical frame and write to it using the mapped page. This setup is hidden behind MmAllocPage.

=== Kernel Pool: Where can we safely get free kernel pages from? ===

Alright, so we can allocate free PFN's to use. So where do we map it to? This is where the System Pool comes in. The basic idea is that we need to reserve an area of Kernel Space and manage what regions of the area are in use or free to map. Since we need a way to describe pages, we introduce a new PTE type for a "Free PTE". We select a Page Table to act as the System Pool, initially all PTE's are "Free PTE" type. Free PTE's have a special format to store a link between other Free PTE's effectively creating a PTE linked list. Like with the PFN Database, the implementation uses a single global to store the top of the PTE Free Stack.

When a Free PTE is selected to be allocated, it is turned into a Standard PTE mapped to a PFN. When a Standard PTE is to be free'd, it is turned into a Free PTE and reattached to the stack. There are 1024 Free PTE's creating a 4MB System Pool.

Free PTE's have the following format:
<source lang="c">
typedef struct _MMPTELIST {
	uint32_t valid: 2;
	uint32_t oneEntry : 1;
	uint32_t filler0 : 8;
	uint32_t nextEntry : 20;
	uint32_t prototype : 1;
}MMPTELIST;
</source>

=== Kernel Heap: How do we allocate and free kernel resources? ===

The Kernel Heap allocates free pages from the System Pool and maps them with free frames from the PFN Database. The Kernel Heap is a Cached based Slab allocator; this allows us to avoid unnecessary reallocations. All kernel objects have a dedicated Cache that which allocations are taken place on. There is a set of special Caches, in powers of 2, used for general allocations as well. There is also a Cache of Caches that Cache objects are allocated from. If resources run low, the Kernel calls the allocator to actually release Free'd resources which will free the page from the System Pool (if it is empty) and the frame from the PFN Database.

=== Recursive paging: How do we update paging structures? ===

A well known problem is that all paging structures use PFN's and the Page Directory Base Register (PDBR) uses the physical address of the root paging structure. However when paging is enabled, this becomes an issue: we need to be able to dynamically update, create, remove paging structures with paging enabled. What we need to know is what virtual address the paging structures are located at, and store that somewhere. Yet again we run into the problem of needing memory to manage memory.

A common solution to this is "recursive paging". Because this is a microkernel architecture, we want to reserve as much of the address space to User Space. So we go with the highest possible page table and recursively define it so the last page table has the PFN to itself. This reserves the last 4MB of the address space to the paging structures themselves: without needing to allocate memory. The recursive structure gives us the following addresses:

* MM_PAGE_DIRECTORY_BASE 0xfffff000
* MM_PAGE_DIRECTORY_END 0xffffffff
* MM_PAGE_TABLE_BASE 0xffc00000
* MM_PAGE_TABLE_END 0xffffffff
* MM_KERNEL_PAGE_TABLE_BASE 0xfff00000
* MM_KERNEL_PAGE_TABLE_END 0xfff00fff

In order to map something into the address space of a process K, the kernel must switch to the respective address space and then update or write to the paging structures starting at MM_PAGE_TABLE_BASE.

=== VAD Tree: How do we manage User Space? ===

User Space is a little complicated as we don't know what will be where: it can contain different segments, user space stack, shared libraries, copy-on-write segments, file mappings, and others. To know where things are, we use a Virtual Address Descriptor (VAD) tree. This is part of each Process Control Block (PCB) for managing the respective Process User Address Space.

=== How is page swapping / demand paging implemented? ===

When the valid bit of a PTE is set to 0, all other bits are ignored by the processor. This allows us to store whatever we want in the remaining bits. In fact, we already do this with the System Pool. We define what we call a "System PTE" that stores information about how to find the page in Swap Space. If a page fault occurs, we check if the PTE is a System PTE and, if it is, it will tell us how to locate the page so we can bring it back in. When the Kernel is told to load a Process, it will allocate the PTE's for the process as System PTE's only without loading any part of the process into memory. When it executes the process and a page fault occurs, the kernel will load the real page into memory on demand.

The System PTE type for page swapping is the following.
<source lang="c">
typedef struct _MMPTE_SYSTEM {
	uint32_t valid: 1;
	uint32_t pageFileLow: 4;
	uint32_t reserved: 7;
	uint32_t pageFileHi: 20;
}MMPTE_SYSTEM;
</source>

=== What about Copy on Write? ===

The Standard PTE has 3 bits next to the PFN field that are free for use by the operating system. One of the bits is set to mark for Copy on Write pages. Copy on Write pages are read only; on a write attempt a page fault will trigger. If the Copy on Write bit is set, the Kernel is free to allocate a new PFN for the page, copy the original page, and map the new PFN as a now Writable page.

=== Initializing: How do we start at 1MB to a higher half kernel? ===

Setting up paging is a bit complicated: the kernel is loaded at 1MB after all. Additionally, we are still using the stack setup by NBOOT in low memory. So setup needs to be done very carefully in order to gather what is needed to initialize core memory systems, identity map ourselves to two locations at once, enable paging, relocate the kernel to 3GB and continue executing from there, continue memory management setup, create stack space for the kernel, and finally remove Identity Space.

# MmScanMemoryMap
# MmBuidPfnDatabase
# MmBuildKernelSpace
# HalEnablePaging
# ExRelocateSelf
# Jump to KERNEL_VIRTUAL
# MmBuildSystemPTEPool
# MmInitializeGlobalCacheList
# MmBuildKernelStack
# MmRemoveIdentitySpace
````

<a id="w004594"></a>

## W004594 — User:Klange

Bron-ID: W004594
Namespace: User (2)
Revisie: 25955 | Datum: 2021-05-27T01:26:54Z
Bron: https://wiki.osdev.org/User:Klange
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKlange&oldid=25955

````text
Hi. I make [[User:Klange/ToaruOS|ToaruOS]] and [https://github.com/kuroko-lang/kuroko Kuroko].
````

<a id="w004595"></a>

## W004595 — Open Programmable Interrupt Controller

Bron-ID: W004595
Namespace: Main (0)
Revisie: 30159 | Datum: 2026-03-09T08:43:48Z
Bron: https://wiki.osdev.org/Open_Programmable_Interrupt_Controller
Revisie-URL: https://wiki.osdev.org/index.php?title=Open_Programmable_Interrupt_Controller&oldid=30159

````text
The '''Open Programmable Interrupt Controller''' '''(OPIC)''' is an interrupt controller with 8-16 inputs, much similar to the PIC, that provides a method for PCI devices to trigger interrupts.

Limited information on the OPIC can be found on the indexed Internet.

== History ==
The OPIC, also known as the OpenPIC, was an SMP controller, similar to the modern APIC, supporting up to 32 processors. It also has 8-16 interrupt input lines allowing for PCI devices to trigger interrupts to the CPU.

It was designed jointly by AMD/Cyrix. AMD released an OpenPIC specification on their website about 20 years ago but it has since been taken down and cannot be found on web archives.

It remains to be an elusive controller, and the only reference to the documentation in source code is in the linux/include/linux/openpic.h header. A snippet of that header cannot be shared due to licensing issues.

Wayback Machine archives of this specification redirect to a 404 Not Found page on AMD's website.

Many different computer encyclopedias/dictionaries offer this same definition:
 (OPEN Programmable Interrupt Controller) An SMP chip architecture endorsed by AMD and Cyrix Corporation that provides symmetric multiprocessing (SMP) for x86 and PowerPC systems. It can support up to 32 processors. See APIC.

A patent for "Method for PCI IO using PCI device memory mapping in a logically partitioned system" namedrops the OPIC:
 The Intel Open Programmable Interrupt Controller is an example of an interrupt controller that is in wide use in PC, Workstation, and PC server class computers and that commonly provides 8 or 16 such interrupt inputs for receiving interrupts from PCI devices.

This definition contradicts the other definitions that do not associate the OPIC with Intel, but rather AMD/Cyrix.

== Information ==
Official documentation on the OPIC has been completely purged from the internet. An email to ''openpic@amd.com'' for further information returned a delivery error.

The author of this page is currently ongoing further investigation into the OPIC specifications.

== Implementations ==
The OPIC was supposed to be implemented in some IBM systems, however when the OPIC was scrapped IBM went with the [[MPIC]].

The Linux kernel has implemented a driver for the OPIC, and is most likely your most solid source of information on the OPIC.
[[Category:Interrupts]]
````

<a id="w004596"></a>

## W004596 — OPIC

Bron-ID: W004596
Namespace: Main (0)
Revisie: 25991 | Datum: 2021-05-27T21:58:50Z
Bron: https://wiki.osdev.org/OPIC
Revisie-URL: https://wiki.osdev.org/index.php?title=OPIC&oldid=25991
Doorverwijzing: Open Programmable Interrupt Controller

````text
#REDIRECT [[Open_Programmable_Interrupt_Controller]]
````

<a id="w004597"></a>

## W004597 — Drawing In a Linear Framebuffer

Bron-ID: W004597
Namespace: Main (0)
Revisie: 30599 | Datum: 2026-05-21T22:46:51Z
Bron: https://wiki.osdev.org/Drawing_In_a_Linear_Framebuffer
Revisie-URL: https://wiki.osdev.org/index.php?title=Drawing_In_a_Linear_Framebuffer&oldid=30599

````text
Linear framebuffer is a region in memory, which represents the image on screen as an array of pixels. By writing to it, you can draw on the display(s), and as a result display text, as well as images and graphic interfaces (windows, menus, icons, fancy cursors and buttons, etc.). Framebuffer is almost universally available on modern x86 hardware, as well as on the most other architectures (ARM, RISC-V and so on). When booting the computer, the framebuffer can be set up through the firmware (BIOS or UEFI on x86), and is usually done by most competent bootloaders before handing over the control. On x86, Framebuffer effectively replaced the [[VGA Text Mode]] (which might still be considered when making a BIOS bootloader or targeting obsolete hardware, e.g. making a DOS clone). Once the operating system is booted, the video modes may also be switched by a [[GPU]] driver.

==Graphics Modes==
{{Main|Getting VBE Mode Info}}
{{Main|GOP}}
VGA and VBE modes can be selected using BIOS interrupt 0x10 (while in real mode). [http://www.ctyme.com/intr/int-10.htm INT 0x10], [[VESA Video Modes]] and [http://www.vesa.org/ VESA] are resources for, VGA and VBE/VESA.

VGA is limited to 16-color 640x480, meanwhile VBE (BIOS systems) and [[GOP]] ([[UEFI]] machines excluding some very early ones) can go up to the monitor and video card's maximum supported resolution.

==Switching==

The cleanest way to set up your video mode is to go through the video BIOS. It can be performed through the regular <tt>Int 0x10</tt> interface, or through the (optional) [[Protected mode]] interface offered by VBE3. As you can guess, <tt>Int 0x10</tt> requires a 16-bit environment, so you can only use it in [[Real Mode]] or [[Virtual 8086 Mode]]

Practically, the options are (in ascending order of difficulty):
*Set up the mode you want at early stage (in the bootloader) before entering protected mode.
**Develop on an [[UEFI]] system, and get a [[GOP]] framebuffer at boot-time.
**Let your [[Bootloader]] do the switch for you.
*If using an emulator, write a driver for the [[Bochs VBE Extensions|Bochs Graphics Adapter]] (supported by many open-source emulators, not just Bochs).
*Switch back to [[Real Mode]] or [[Unreal Mode]] for setting the proper video mode (Napalm at [http://www.rohitab.com/discuss/topic/35103-switch-between-real-mode-and-protected-mode/ rohitab.com] has a neat little function for reference.)
*Write a [[VGA Hardware|VGA driver]] that can do low-resolution modes on practically all hardware
*Use the PMID from VBE3, if present
*Set up a [[Virtual 8086 Mode|V8086]] [[Virtual_Monitor|monitor]] that will execute the mode-switching code
*Run a software code translation tool to produce pmode code out of bios rmode code. ([http://www.osdev.org/phpBB2/viewtopic.php?t=10321 SANiK is on the catch])
*You write a driver for your specific [[:Category:Video|graphics card]]

==Locating Video Memory==

For standard VGA video modes the video memory will be at address <tt>0xA0000</tt> for EGA/VGA video modes and <tt>0xB8000</tt> for CGA and text modes. To find out which one look at the following table:

{| class="wikitable"
! Mode
! Description
|-
| 00
| text 40*25 16 color (mono)
|-
| 01
| text 40*25 16 color
|-
| 02
| text 80*25 16 color (mono)
|-
| 03
| text 80*25 16 color
|-
| 04
| CGA 320*200 4 color
|-
| 05
| CGA 320*200 4 color (m)
|-
| 06
| CGA 640*200 2 color
|-
| 07
| MDA monochrome text 80*25
|-
| 08
| PCjr
|-
| 09
| PCjr
|-
| 0A
| PCjr
|-
| 0B
| ''reserved''
|-
| 0C
| ''reserved''
|-
| 0D
| EGA 320*200 16 color
|-
| 0E
| EGA 640*200 16 color
|-
| 0F
| EGA 640*350 mono
|-
| 10
| EGA 640*350 16 color
|-
| 11
| VGA 640*480 mono
|-
| 12
| VGA 640*480 16 color
|-
| 13
| VGA 320*200 256 color
|-
|}

For VESA modes, the framebuffer address is stored in the [[Getting VBE Mode Info|mode info block]]. This is the '''physical''' address of the linear framebuffer (it is not a 16-bit far pointer but a 32-bit linear pointer) : if you use paging, you have to map it somewhere to use it. The length of the framebuffer in bytes is pitch * height.

For GOP, the framebuffer address is in the EFI_GRAPHICS_PROTOCOL struct, gop->Mode->FrameBufferBase. GOP '''does not''' support teletype character modes, only graphics pixel oriented modes.

== Plotting Pixels ==

=== Location ===
If you wanted to plot a red pixel in the middle of your screen. The first thing you have to know is where the middle of the screen is. In 320x200x8 (mode 13), this will be at 100x320+160 = 32160. In general, your screen can be described by:
* width: how many pixels you have on a horizontal line
* height: how many horizontal lines of pixels are present
* pitch: how many ''bytes'' of VRAM you should skip to go one pixel down
* depth: how many ''bits'' of color you have
* "pixelwidth": how many bytes of VRAM you should skip to go one pixel right.

"pitch" and "width" may seem redundant at first sight but they aren't. It's not rare once you go to higher (and exotic) resolutions to have e.g. 8K bytes per line while your screen is actually 1500 pixels wide (32-bits per pixel). The good news is that it allows smooth horizontal scrolling (which is mainly useful for 2D games :P )

Pitch and pixel width are usually announced by VESA mode info. Once you know them, you can calculate the place where you plot your pixel as:
<syntaxhighlight lang="c">unsigned char *pixel = vram + y*pitch + x*pixelwidth;</syntaxhighlight>

=== Color ===
The second thing to know is what value you should write for "red". This depends on your screen setup, again. In EGA mode, you have a fixed palette featuring dark-red (color 4) and light-red (color 12). Yet, EGA requires you to plot each bit of that on different pixel plane, so refer to EGA programming tutorials if you '''really''' want such modes supported.
In conventional 320x200x8 VGA mode, you have the same colours 4 and 12 as in EGA so you would plot your red pixel with

 *pixel = 4;

Yet, in VGA, the palette is reprogrammable (as you can learn in FreeVGA documents), so virtually any value between 0..255 could be 'red' if you program the palette so :P

So a full "putpixel" function for VGA would be
<syntaxhighlight lang="c">
/* example for 320x200 VGA */
void putpixel(int pos_x, int pos_y, unsigned char VGA_COLOR)
{
    unsigned char* location = (unsigned char*)0xA0000 + 320 * pos_y + pos_x;
    *location = VGA_COLOR;
}
</syntaxhighlight>

Finally, in VESA and GOP modes, you usually have truecolor or hicolor, and in both of them, you have to give independent red, green and blue values for each pixel. modeinfo will (again) instruct you of how the RGB components are organized in the pixel bits. E.g. you will have <tt>0bRRRRRGGGGGBBBBB</tt> for 15-bits mode, meaning that #ff0000 red is there <tt>0x7800</tt>, and #808080 grey is <tt>0x4210</tt> (pickup pencil, draw the bits and see by yourself)
<syntaxhighlight lang="c">
static void putpixel(unsigned char* screen, int x,int y, int color) {
    unsigned where = x*pixelwidth + y*pitch;
    screen[where] = color & 255;              // BLUE
    screen[where + 1] = (color >> 8) & 255;   // GREEN
    screen[where + 2] = (color >> 16) & 255;  // RED
}
</syntaxhighlight>

=== Optimizations ===

It can be tempting from here to write other drawing functions from calls to putpixel... Don't. Drawing a filled rectangle means you access successive pixels and then advance by "pitch - rect_width" to fill the next line. If you do a "for(y=100;y<200;y++) for(x=100;x<200;x++) putpixel (screen,x,y,RED);" loop, you'll recompute 'where' about 10,000 times, as well as make an excessive amount of function calls. Even if the compiler has optimized y*pitch into adds and shifts instead of multiplication, it's silly to waste CPU time while you could do
<syntaxhighlight lang="c">
static void fillrect(unsigned char *vram, unsigned char r, unsigned char g, unsigned   char b, unsigned char w, unsigned char h) {
    unsigned char *where = vram;
    int i, j;

    for (i = 0; i < w; i++) {
        for (j = 0; j < h; j++) {
            //putpixel(vram, 64 + j, 64 + i, (r << 16) + (g << 8) + b);
            where[j*pixelwidth] = r;
            where[j*pixelwidth + 1] = g;
            where[j*pixelwidth + 2] = b;
        }
        where+=pitch;
    }
}
</syntaxhighlight>
That should be enough to get you started coding (or googling for) a decent video library.

== Drawing Text ==
{{Main|VGA Fonts}}

Once in graphic mode, you no longer have the BIOS or the hardware to draw fonts for you. The basic idea is to have font data for each character and use it to plot (or not to plot) pixels. There are plenty of ways to store those fonts depending on whether they have multiple colors or not, alpha channel or not etc. What you will basically have, however, is:
<syntaxhighlight lang="c">
// holding what you need for every character of the set
font_char* font_data[CHARS];
 
// rendering one of the character, given its font_data
void draw_char(screen, where, font_char*);

void draw_string(screen, where, char* input) {
    while(*input) {
        draw_char(screen,where,font_data[input]);
        where += char_width;
        input++;
    }
}
 
void draw_char(screen, where, font_char*) {
    for (l = 0; l < 8; l++) {
        for (i = 8; i > 0; i--) {
            j++;
            if ((font_char[l] & (1 << i))) {
                c = c1;
                put_pixel(j, h, c);
            }
        }
        h++;
        j = x;
    }
}
</syntaxhighlight>

== Drawing Icons ==
{{Main|Loading Icons}}

For a GUI, you'll probably want to display icons. It's as simple as
<syntaxhighlight lang="c">
void draw_icon(x, y, w, h, pixels) {
    for (l = j = 0; l < h; l++) {
        for (i = 0; i < w; i++, j++) {
            put_pixel(x + i, y + l, pixels[j]);
        }
    }
}
</syntaxhighlight>
The difficulty is, icons aren't stored as width, height and pixels array on disk. First you have to [[Loading Icons|decode an image file]] to get that information.

==See Also==
=== Articles ===
*[[How_do_I_set_a_graphics_mode|Extra Notes]]
*[[Double Buffering]]
*[[3D Renderer Basics]]
*[[PC Screen Font]]
*[[Scalable Screen Font]]
*[[Loading Icons]]

=== External Links ===
*[http://bos.asmhackers.net/forum/viewtopic.php?id=65 Covers basically the same, and is ASM-oriented.]

[[Category:Video]]
[[Category:Graphical UI]]
[[Category:Tutorials]]
````

<a id="w004598"></a>

## W004598 — ACPI Timer

Bron-ID: W004598
Namespace: Main (0)
Revisie: 30392 | Datum: 2026-03-24T00:36:08Z
Bron: https://wiki.osdev.org/ACPI_Timer
Revisie-URL: https://wiki.osdev.org/index.php?title=ACPI_Timer&oldid=30392

````text
The ACPI Power Management Timer is a very simple timer which runs at 3.579545 MHz and generates an SCI when the counter has overflown. It is quite limiting (you cannot set custom rates, for example). This generally means that using other timers to generate interrupts, like [[APIC_timer|APIC Timer]] and [[HPET]], is usually a better idea. However, the ACPI PM Timer is still a very nice and straightforward time source, which is very easy to support and can be used to calibrate other timers (LAPIC/TSC) and/or as a general monotonic time source, without necessarily having to make use of its interrupt.

== Detecting the ACPI Power Management Timer ==
The computer must obviously support [[ACPI]] first. Then, you need to look into the [[RSDP]] to get the [[RSDT]] or [[XSDT]], to find the [[FADT]] table. Once you got it, you need to check if the PMTimerLength is 4. If it isn't, then you must assume the timer is not available. Then, if the ACPI revision is 2 or greater, and the X_PMTimerBlock Generic Address Structure is available (check if the address is not 0), use it. Otherwise, use PMTimerBlock as an IO port.

== 24-bit and 32-bit mode ==
The ACPI Power Management Timer supports 24 and 32 bit counters. To check if the counter is 32 bit, simply check if bit 8 of FADT Flags if set.

== Reading the counter value ==
If using X_PMTimerBlock:
* Check in which address space the address is:
** If it's MMIO: map the address into virtual memory and read the counter as an unsigned 32 bit integer from it
** If it's IO: read the counter as unsigned 32 bit integer from IO
Else if using PMTimerBlock:
* Read the counter as an unsigned 32 bit integer from IO

== Counter value ==
{| {{wikitable}}
! Bits
! Description
|-
| 0:23 || The 24 bits of the counter.
|-
| 24:31 || The upper 8 bits of the counter, if it's a 32 bit one. If the counter is 24 bit, then they're 0.
|}

== IRQs ==
The ACPI Power Management Timer generates a SCI when the counter has overflown. To know more about SCIs, see [[FADT#SCI_Interrupt]].

== Handling overflows ==
The ACPI PM timer overflows after very short time, which is about 5 seconds with a 24 bit counter, and about 20 minutes with 32 bits, which means that you must explicitly handle it if you are using it as a time source.

One of the strategies to handling overflows is recording the last time read from the timer into a variable, and having a software counter to count overflows (with atomics, if your kernel supports multiprocessing). When you detect that the ACPI timer's counter has gone back in time, you increment the overflow counter, keeping the monotonic time. To not miss the potential overflows while the timer has not been read for a long time, the kernel may set up a periodic task, which makes sure the timer is always read before that happens (for example, once a second). You may then choose to use interrupts from a different timer (for example, LAPIC one-shot or TSC deadline) to generate the event, or potentially use the timer itself.

== Comparison to other x86 timers ==
In its current state, x86 PCs have many different timers and time sources (PIT, RTC, ACPI PM Timer, HPET, LAPIC, TSC, paravirtualization timers, etc.), all of which might or might not be present in any given PC. While the ACPI timer is simple, it is still important for mid 2000s to new hardware.

Comparing it to PIT, the ACPI PM timer in combination with some other timer is often a better choice, considering that the former has a lot of quirks (has been seen be broken in different on both old and latest generation hardware), is more difficult to discover (requires executing AML to do it properly, or probing it on its standard ports, which has its own set of problems), and is arguably more difficult to support.

Comparing PM Timer to HPET, the latter should offer better precision, and more functionality. However, there are a lot of systems that allow disabling HPET in BIOS, or just don't have it in the first place (especially laptops), which means that you can't rely on it always being present.

Finally, TSC is a good alternative to ACPI PM timer as a monotonic time source. If invariant TSC is present, it is probably more convenient to use it instead, with the ACPI PM, HPET or other timer being used as a calibration reference during boot (if the CPUID leaf 0x15 TSC frequency is not available).

This means, that in kernels targeting physical hardware made during and after late 90s, it is a good idea to support several different timers/counters. Namely, ACPI PM, HPET and APIC/TSC is a solid set of timers to support to cover most of the relevant hardware.

== See Also ==

=== External Links ===

* https://uefi.org/specs/ACPI/6.5_A/04_ACPI_Hardware_Specification.html#power-management-timer

[[Category:ACPI]]
[[Category:Timers]]
````

<a id="w004599"></a>

## W004599 — Message Signaled Interrupts

Bron-ID: W004599
Namespace: Main (0)
Revisie: 26004 | Datum: 2021-05-29T15:19:58Z
Bron: https://wiki.osdev.org/Message_Signaled_Interrupts
Revisie-URL: https://wiki.osdev.org/index.php?title=Message_Signaled_Interrupts&oldid=26004
Doorverwijzing: PCI

````text
#REDIRECT[[PCI#Message_Signaled_Interrupts]]
````

<a id="w004600"></a>

## W004600 — BGRT

Bron-ID: W004600
Namespace: Main (0)
Revisie: 29081 | Datum: 2024-06-07T13:37:35Z
Bron: https://wiki.osdev.org/BGRT
Revisie-URL: https://wiki.osdev.org/index.php?title=BGRT&oldid=29081

````text
{{ACPI}}

The Boot Graphics Record Table (BGRT) is an optional [[ACPI]] table which contains a pointer to the image that has been drawn during boot.

== Table structure ==
{| {{wikitable}}
! Offset (hex)
! Length
! Description
|-
| 00 || 4 || Signature 'BGRT'
|-
| 04 || 4 || Length of the table (in bytes, including header)
|-
| 08 || 1 || Revision
|-
| 09 || 1 || Checksum
|-
| 0a || 6 || OEM ID
|-
| 10 || 8 || OEM Table ID
|-
| 18 || 4 || OEM Revision
|-
| 1c || 4 || Creator ID
|-
| 20 || 4 || Creator revision
|-
| 24 || 2 || Version ID (must be 1)
|-
| 26 || 1 || Status
|-
| 27 || 1 || Image type
|-
| 28 || 8 || Image address
|-
| 30 || 4 || Image X offset
|-
| 34 || 4 || Image Y offset
|}

== Fields ==
=== Status ===
Indicates the image's status.
{| {{wikitable}}
! Bits
! Description
|-
| 0 || Displayed
|-
| 1:2 || Orientation offset
|-
| 3:7 || Reserved (must be 0)
|}
Orientation offset:
* 0b00: no offset
* 0b01: 90º
* 0b10: 180º
* 0b11: 270º

=== Image type ===
0: Image is a bitmap

1-255: Reserved

=== Image address ===
64-bit physical address to an in-memory copy of the displayed image. It can be a 24 bit bitmap (0xRRGGBB) or a 32 bit bitmap (0xrrRRGGBB), where lowercase R means reserved.

=== Image offsets ===
Describes the offsets to the (X, Y) of the screen for the top left image corner.

[[Category:ACPI]]
````

<a id="w004601"></a>

## W004601 — Talk:BGRT

Bron-ID: W004601
Namespace: Talk (1)
Revisie: 26029 | Datum: 2021-05-31T12:26:50Z
Bron: https://wiki.osdev.org/Talk:BGRT
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABGRT&oldid=26029

````text
= Practicality =
What is the actual practical use for the BGRT? It seems to me like pretty much a waste of RAM. --[[User:Rizxt|Rizxt]] 11:56, 30 May 2021 (CDT)

Answer: so operating systems can use the original manufacturer's wallpaper for its own reasons. Anyways, this is an ACPI thing, it is defined on the specification. -- [[User:Atie|Atie]]
````

<a id="w004602"></a>

## W004602 — Interrupts tutorial

Bron-ID: W004602
Namespace: Main (0)
Revisie: 26024 | Datum: 2021-05-30T20:50:23Z
Bron: https://wiki.osdev.org/Interrupts_tutorial
Revisie-URL: https://wiki.osdev.org/index.php?title=Interrupts_tutorial&oldid=26024
Doorverwijzing: Interrupts Tutorial

````text
#REDIRECT [[Interrupts Tutorial]]
````

<a id="w004603"></a>

## W004603 — Talk:VFS

Bron-ID: W004603
Namespace: Talk (1)
Revisie: 26027 | Datum: 2021-05-31T00:32:41Z
Bron: https://wiki.osdev.org/Talk:VFS
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AVFS&oldid=26027

````text
=== Reading / Writing ===
Section 3.2 states that the VFS should translate positions into sectors and whatnot, but that seems to be the job of the native filesystem driver, no? I thought the point of a VFS was to redirect file read/writes to the appropriate filesystem driver, and the filesystem driver does all the hard work of translating positions to sectors and whatnot. Am I wrong in this assumption? --[[User:Rizxt|Rizxt]] 19:32, 30 May 2021 (CDT)
````

<a id="w004604"></a>

## W004604 — Talk:Busy loop

Bron-ID: W004604
Namespace: Talk (1)
Revisie: 26030 | Datum: 2021-05-31T12:35:13Z
Bron: https://wiki.osdev.org/Talk:Busy_loop
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABusy_loop&oldid=26030

````text
Doesn't the sample code introduce a race condition between the while loop and the update? If two processes are attempting to lock the same mutex, and a switch occurs just after the first has seen mutex->in_use become false, but before it has set it to 1, then the second could also see it as false, set it to 1, and continue followed by the first doing the same thing once it's active again.
<source lang=c>
int lock(mutex *mutex) {
    // Wait for the mutex to become free.
    while(mutex->in_use);
    // Added: what if a task switch happens here?
    mutex->in_use = 1;
    return 0;
}</source>

Answer: indeed that is a problem, specially in multicore. Atomic operations MUST be used here. There are GCC bultins that can do that for you without using assembly, like for example, __sync_bool_compare_swap, if I remember clear the name. --[[User:Atie|Atie]]
````

<a id="w004605"></a>

## W004605 — User talk:Rizxt

Bron-ID: W004605
Namespace: User talk (3)
Revisie: 26036 | Datum: 2021-05-31T22:01:04Z
Bron: https://wiki.osdev.org/User_talk:Rizxt
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ARizxt&oldid=26036

````text
Please stop deleting articles. We are not Wikipedia, we do not have a policy on notability, and it is terribly difficult to recover deleted articles. One of the articles you deleted was about building a D cross-compiler, but you misidentified it as a typo of "GCC". --[[User:Klange|klange]] 16:51, 31 May 2021 (CDT)

I see. My apologies. --[[User:Rizxt|Rizxt]] 16:53, 31 May 2021 (CDT)

I have a backup of the page and I will work on restoring it. --[[User:Rizxt|Rizxt]] 17:01, 31 May 2021 (CDT)
````

<a id="w004606"></a>

## W004606 — GDC Cross-Compiler

Bron-ID: W004606
Namespace: Main (0)
Revisie: 28036 | Datum: 2023-07-09T18:46:46Z
Bron: https://wiki.osdev.org/GDC_Cross-Compiler
Revisie-URL: https://wiki.osdev.org/index.php?title=GDC_Cross-Compiler&oldid=28036

````text
{{In_Progress}}
If you reached here, you should have enough knowledge about cross-compilers, how they work. and so on. If you don't, check out [[GCC Cross-Compiler]].

== Requirements ==
We assume you have a host system with a working [[GCC]] installation. If you are not using a bash shell, you might have to modify some of the command lines below. If you have just installed the basic [[Cygwin]] package, you have to run the setup.exe again and install the following packages:

* [[GCC]]
* Make
* Flex
* Bison
You will also have to install the following (using your system's package management):

[https://gmplib.org GNU GMP] (libgmp-devel on Cygwin, libgmp3-dev on apt-based systems, dev-libs/gmp on Gentoo)

[https://www.mpfr.org GNU MPFR] (libmpfr-devel on Cygwin, libmpfr-dev on apt-based systems, dev-libs/mpfr on Gentoo)

[https://multiprecision.org MPC] (libmpc-devel on Cygwin, libmpc-dev on apt-based systems, dev-libs/mpc on Gentoo)

== Step 1 - Bootstrap ==
We will build a toolset running on your host that can turn source code into object files for your target system.

We need the binutils and the [[GCC]] packages from [http://ftp.gnu.org/gnu/]. Make sure you downloaded exactly GCC ver 4.9.x (or 3.x, if you like old stuffs). Download them to /usr/src (or whereever you think appropriate), and unpack them. Also, we need gdc package from [https://github.com/D-Programming-GDC/GDC/tree/gdc-4.9]. Current version is 0.24.

=== Preparation ===
   export PREFIX=/usr/local/cross
   export TARGET=i586-elf
   cd /usr/src
   mkdir build build/binutils build/gcc

=== binutils ===
   cd /usr/src/build/binutils
   /usr/src/binutils-x.xx/configure --target=$TARGET --prefix=$PREFIX --disable-nls
   make all
   make install

=== gcc ===
This needs some work out. First, unpack gcc, and we got /usr/src/gcc-x.x.x Go to /usr/src/gcc-x.x.x/gcc, and unpack gdc.This will create a subdirectory named "d".

   cd /usr/src/gcc-x.x.x
   ./gcc/d/setup-gcc.sh

Then go to /usr/src/gcc-x.x.x/gcc/d, open d-lang.cc and look at line 188. You'll see this

  const char* cygwin_d_os_versym = D_OS_VERSYM;

Change D_OS_VERSYM to whatever you want, since it is a string, like "MyOS". But do not set it to NULL, or your compiler will segfault. Forum Post by Wilkie

Now, you can build [[GCC]]:

   cd /usr/src/build-gcc
   export PATH=$PATH:$PREFIX/bin
   ../gcc-x.x.x/configure --target=$TARGET --prefix=$PREFIX --disable-nls \
       --enable-languages=c,d,c++ --without-headers
   make all-gcc
   make install-gcc

== Step 2 ==
Before you can build a kernel, you must have a kind of D Runtime. At least you must have a correct object.d file.

TO BE CONTINUED...

[[Category:Compilers]]
[[Category:D]]
````

<a id="w004607"></a>

## W004607 — LAI

Bron-ID: W004607
Namespace: Main (0)
Revisie: 31024 | Datum: 2026-10-15T12:25:08Z
Bron: https://wiki.osdev.org/LAI
Revisie-URL: https://wiki.osdev.org/index.php?title=LAI&oldid=31024

````text
{{ACPI}}

'''LAI''', or Lightweight AML Interpreter, is a lightweight alternative to the bulky [[ACPICA]]. It is much, much smaller, and requires less functions in the OS interface layer be implemented. As well, not all of the functions need to be implemented (although it is optimal to do so) because LAI simply panics when a function is needed and undefined, rather than requiring them at link time. This means that you can use the simpler functions of LAI if you aren't ready to implement an entire OS interface layer just to power off the machine.

As an aside, LAI is permissively MIT-licensed, quite like ACPICA, which allows you to choose between GPLv2, a BSD License, and a non-OSI "Intel" License.

Official documentation can be [https://github.com/managarm/lai/wiki/ found on GitHub].

== Uses ==
LAI's core is just an [[AML]] interpreter which currently works on most hardware with sane AML. However, it includes drivers and helpers for:
* Embedded Controller 
* Legacy PCI IRQ pin routing
* Power management (Setting sleep states S0-S5, reset) 
* Discovering the [[RSDP]] in BIOS systems
* Enabling ACPI mode 

LAI is no longer maintained by its developers. It is recommended by the LAI developers for you to use [[uACPI]] instead of LAI. LAI will likely stay compatible with future AML revisions assuming that the ACPI spec does not change much in AML (AML is rarely changed).

NOTE: If one needs something as simple as LAI, it would likely be easier to update LAI to support the newer ACPI specs compared to porting over uACPI or ACPICA which both have a far larger set of "host" functions.

== Porting ==
In order to call into the kernel, LAI has a few standardized "host" functions that the kernel needs to implement, most of these are weakly linked, and are optional. However that does not mean that only the few required host functions are needed, in practice, all of the Required and Integral functions are needed for smooth operation. You can find LAI's full host documentation [https://github.com/managarm/lai/wiki/Host-API-Documentation here]
=== Required ===
These functions are 100% required for the proper functioning of the interpreter, it will not link without these functions

==== laihost_log ====
<syntaxhighlight lang="c">
/* Logs a message. level can either be LAI_DEBUG_LOG for debugging info,
   or LAI_WARN_LOG for warnings */
void laihost_log(int level, const char *msg);
</syntaxhighlight>
==== laihost_panic ====
<syntaxhighlight lang="c">
/* Reports a fatal error, and halts. */
__attribute__((noreturn)) void laihost_panic(const char *msg);
</syntaxhighlight>
==== laihost_malloc ====
<syntaxhighlight lang="c">
void *laihost_malloc(size_t);
</syntaxhighlight>
==== laihost_realloc ====
<syntaxhighlight lang="c">
void *laihost_realloc(void *, size_t);
</syntaxhighlight>
==== laihost_free ====
<syntaxhighlight lang="c">
void *laihost_free(void *);
</syntaxhighlight>
=== Integral ===
These functions, while not required to link with LAI, are practically integral to its operation

==== laihost_map ====
<syntaxhighlight lang="c">
/* Maps count bytes from the given physical address and returns
   a virtual address that can be used to access the memory. */
void *laihost_map(size_t address, size_t count);
</syntaxhighlight>
==== laihost_unmap ====
<syntaxhighlight lang="c">
/* Unmaps count bytes from the given virtual address.
   LAI only calls this on memory that was previously mapped by laihost_map(). */
void laihost_unmap(void *pointer, size_t count);
</syntaxhighlight>
==== laihost_scan ====
<syntaxhighlight lang="c">
/* Returns the (virtual) address of the n-th table that has the given signature,
   or NULL when no such table was found. */
void *laihost_scan(char *sig, size_t index);
</syntaxhighlight>
==== laihost_out ====
<syntaxhighlight lang="c">
/* Write a byte/word/dword to the given I/O port. */
void laihost_outb(uint16_t port, uint8_t val);
void laihost_outw(uint16_t port, uint16_t val);
void laihost_outd(uint16_t port, uint32_t val);
</syntaxhighlight>
==== laihost_in ====
<syntaxhighlight lang="c">
/* Read a byte/word/dword from the given I/O port. */
uint8_t laihost_inb(uint16_t port);
uint16_t laihost_inw(uint16_t port);
uint32_t laihost_ind(uint16_t port);
</syntaxhighlight>
==== laihost_pci_read ====
<syntaxhighlight lang="c">
/* Read a byte/word/dword from the given device's PCI configuration space
   at the given offset. */
uint8_t laihost_pci_readb(uint16_t seg, uint8_t bus, uint8_t slot, uint8_t fun, uint16_t offset);
uint16_t laihost_pci_readw(uint16_t seg, uint8_t bus, uint8_t slot, uint8_t fun, uint16_t offset);
uint32_t laihost_pci_readd(uint16_t seg, uint8_t bus, uint8_t slot, uint8_t fun, uint16_t offset);
</syntaxhighlight>
==== laihost_pci_write ====
<syntaxhighlight lang="c">
/* Write a byte/word/dword to the given device's PCI configuration space
   at the given offset. */
void laihost_pci_writeb(uint16_t seg, uint8_t bus, uint8_t slot, uint8_t fun, uint16_t offset, uint8_t val);
void laihost_pci_writew(uint16_t seg, uint8_t bus, uint8_t slot, uint8_t fun, uint16_t offset, uint16_t val);
void laihost_pci_writed(uint16_t seg, uint8_t bus, uint8_t slot, uint8_t fun, uint16_t offset, uint32_t val);
</syntaxhighlight>
==== laihost_sleep ====
<syntaxhighlight lang="c">
/* Sleeps for the given amount of milliseconds. Can be stubbed on emulators */
void laihost_sleep(uint64_t ms);
</syntaxhighlight>

=== Optional ===
These functions are optional and LAI will not use them unless you try to use certain features. For example the sync functions will only be called when a lock is contended, this should not happen if you do not call LAI from 2 threads at once. (However it has been observed on at least one machine with buggy AML)
==== laihost_sync_wait ====
<syntaxhighlight lang="c">
/* Blocks the current thread.
   The host ensures that calls to laihost_sync_wait() and laihost_sync_wake()
   on the same lai_sync_state are totally ordered.
   Before blocking, sync->val is compared against val. If these two values are non-equal, the function returns immediately.
   Returns a non-zero value if the timeout was exhausted and zero otherwise. */
int laihost_sync_wait(struct lai_sync_state *sync, unsigned int val, int64_t timeout);
</syntaxhighlight>
==== laihost_sync_wake ====
<syntaxhighlight lang="c">
/* Unblocks a blocked thread.
   The host ensures that calls to laihost_sync_wait() and laihost_sync_wake()
   on the same lai_sync_state are totally ordered. */
void laihost_sync_wake(struct lai_sync_state *sync);
</syntaxhighlight>

=== Unnecessary ===
==== laihost_handle_amldebug ====
<syntaxhighlight lang="c">
/* Reports information about a variable.
   This function does not need to be implemented, lai has a default implementation that will print all the info needed for debugging */
void laihost_handle_amldebug(lai_variable_t *var);
</syntaxhighlight>

== Initialization ==
Before you start using LAI's functions, you need to tell it the ACPI's revision and to let it enumerate the AML namespace.
<syntaxhighlight lang="c">
lai_set_acpi_revision(rsdp->revision);
lai_create_namespace();
</syntaxhighlight>

== API ==
LAI's full API documentation can be found [https://github.com/managarm/lai/wiki on GitHub], however here are some simple examples for using some helpers
=== ACPI Mode ===
This is how you enter full ACPI mode in LAI, do not forget to install an SCI handler and handle ACPI events.
<syntaxhighlight lang="c">
// From <lai/helpers/sci.h>
lai_enable_acpi(0); // if you use legacy 8259 PIC
lai_enable_acpi(1); // if you use the IOAPIC
</syntaxhighlight>

=== Shutdown ===
This is how you shutdown in LAI:
<syntaxhighlight lang="c">
lai_enter_sleep(5); // From <lai/helpers/pm.h>, requires you have enumerated the namespace and are in ACPI mode
</syntaxhighlight>
=== Reboot ===
This is how you reboot in LAI:
<syntaxhighlight lang="c">
lai_acpi_reset(); // From <lai/helpers/pm.h>
</syntaxhighlight>

[[Category:ACPI]]
````

<a id="w004609"></a>

## W004609 — Talk:LAI

Bron-ID: W004609
Namespace: Talk (1)
Revisie: 26052 | Datum: 2021-06-01T14:37:11Z
Bron: https://wiki.osdev.org/Talk:LAI
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ALAI&oldid=26052

````text
== Misinformation ==
== Misinformation ==
Rizxt, I would highly appreciate you to not create articles about topics you're not familiar with. LAI (not Lai) is an AML interpreter which only provides some helpers like for example shutdown. Also, initializing LAI is NOT only setting up the ACPI revision. You need to create a namespace, using lai_create_namespace().
-- [[User:Atie|Atie]]

The documentation needs to be updated. I never heard anything about creating a namespace, and the documentation on the Host API functions doesn't label which are weak, and which are link-time required. --[[User:Rizxt|Rizxt]] 09:37, 1 June 2021 (CDT)
````

<a id="w004610"></a>

## W004610 — Lai

Bron-ID: W004610
Namespace: Main (0)
Revisie: 26044 | Datum: 2021-06-01T06:03:41Z
Bron: https://wiki.osdev.org/Lai
Revisie-URL: https://wiki.osdev.org/index.php?title=Lai&oldid=26044
Doorverwijzing: LAI

````text
#REDIRECT [[LAI]]
````

<a id="w004611"></a>

## W004611 — Talk:Lai

Bron-ID: W004611
Namespace: Talk (1)
Revisie: 26046 | Datum: 2021-06-01T06:03:41Z
Bron: https://wiki.osdev.org/Talk:Lai
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ALai&oldid=26046
Doorverwijzing: Talk:LAI

````text
#REDIRECT [[Talk:LAI]]
````

<a id="w004612"></a>

## W004612 — User:Seds

Bron-ID: W004612
Namespace: User (2)
Revisie: 26056 | Datum: 2021-06-02T00:15:34Z
Bron: https://wiki.osdev.org/User:Seds
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASeds&oldid=26056

````text
{{Stub}}
<div class="center" style="width: auto; margin-left: auto; margin-right: auto;">Warning: Many parts of RISC-V are not yet finally. Things might and will change! Look at the official specification for the most up-to-date information</div>
----
RISC-V is not a single ISA, rather a meta-ISA. It defines basics and boundaries for a family of implementations.
The specification is published under a Creative Common License and actively developed on [[#External Links|github]].
The problems, which are addressed by developement of RISC-V are the legal problems with developing real (hardware) processors, be it
for educational, hobbyist or economical purposes of most other architectures and the huge historical baggage of some processor
families (x86...).
An implementation consists of one of the [[#Base ISA|Base ISAs]] and zero or more [[#Extensions|Extensions]].

== Architecture ==
The RISC-V ISA has fixed-length 32-bit instructions aligned on their natural boundaries, but is designed to encode variable-length instructions.<br/>
The base ISA operates on a little-endian memory system, but non-standard extensions may add support for big-endian or bi-endian.

=== Hardware Threads ===
The RISC-V ISA specifies hardware threads, called ''harts''. A ''hart'' is defined by its own instruction fetch unit.
A processor may contain multiple ''harts'', at least one.
Each hart has an ID associated with it.

=== Exceptions, Traps and Interrupts ===
In RISC-V the term ''Exception'' refers to an unusual condition at run-time associated with an instruction in the current hardware thread.<br/>
A ''Trap'' is a synchronous transfer of control to a trap handler and is caused by an exceptional condition within a RISC-V thread. The trap handlers usually execute in a more privileged environment.<br/>
An external event that occurs asynchronously to the current thread will cause an ''Interrupt''. When an interrupt occurs, some instruction is selected to experience a trap.

=== Privileges ===
The spec defines 4 priviledge modes, of which a valid combination has to be implemented. The modes are:

{| class="wikitable"
|Debug 
| Mode for ''complete'' control, for debuggers
|-
| Machine
| Mode with nearly full control, no debug registers (think firmware), not optional
|-
| Hypervisor
| VM-Hypervisor
|-
| Supervisor
| OS-level
|-
| User
| Application-level, lowest privilege
|}

==== Possible Combinations ====
Valid combinations of privilege levels are:
* One level: Machine mode only for embedded systems
* Two levels: Machine and User mode, small systems
* Three levels: Machine, Supervisor and User mode, ''Complex'' systems able to run Unix-like operating systems


== Base ISA ==

The base ISA specifies RV32I and RV64I, 32 and 64-bit respectively, most of what is said about RV32I also applies to RV64I. Additionally, there is also RV32E, a reduced version of RV32I for embedded systems and RV128I, a 128-bit version, which is mostly a placeholder so far.

==== RV32I ====
RV32I offers 31 general-purpose registers (x1-x31) which hold integer values, the x0 register is hardwired to zero, all registers are 32 bits wide. It specifies a number of logical and arithmetic operations (and, or, xor, shift left and right, addition and subtraction), all of which are available with a source register or an immediate.

==== RV32E ====
RV32E reduces the number of general-purpose registers to 15 (x1-x15), and x0 is still hardwired to constant zero.

==== RV64I ====
RV64I is very similar to its 32-bit counterpart, but offers 64-bit wide registers and can read the CSRs in one operation instead of requiring the programmer to read the upper and lower half separately. Additionaly, there are some instructions to work with words (32 bit) instead of double-words.

==== RV128I ====
Like RV64I but 128bit register length.

== Register set ==

RISC-V base ISA consists of 32 general purpose registers of size X bits, given X could be 32, 64 and 128-bits, depending on your base ISA. Beware that some variants may reduce the set of general purpose registers. 

{| class="wikitable" 
|- style="font-weight:bold; text-align:center;"
! Register
! Calling convention
! Description
|-
| x0
| zero
| Hardwired to hold 0
|-
| x1
| ra
| Return address
|-
| x2
| sp
| Stack pointer
|-
| x3<br />
| gp
| Global pointer
|-
| x4
| tp
| Thread pointer
|-
| x5-x7
| t0-t2
| Temporary registers
|-
| x8
| s0/fp
| Saved register and/or frame pointer
|-
| x9
| s1
| Saved register
|-
| x10-x11
| a0-a1
| Function arguments and/or return values
|-
| x12-x17
| a2-a7
| Function arguments
|-
| x18-x27
| s2-s11
| Saved registers
|-
| x28-x31
| t3-t6
| Temporary registers
|-
| pc
| pc<br />
| Program counter
|}

== Extensions ==

An extensions can be one of the officialy defined ones or a vendor-specific one (there is opcode space explicitely reserved for this) and can define additional elements, including opcodes and registers.

The following Standard Extensions are noteworthy here:
==== Zicsr ====
This extensions defines some kind of secondary address space for '''Control and Status Register''', which are used for controlling
thins like interrupts, privilege level, hart-local timers and such.
These registers can be accessed via the ''CSR instructions''.

==== RVM (Multiply-Divide Instruction Extension) ====
The RVM Extension adds instructions for multiplying, dividing and computing the remainder of a division.

==== RVA (Atomic Instruction Extension) ====
The RVA Extension adds instruction to work atomically with memory, including reserved load and conditional store.

==== RVF & RVD & RVQ (Floating Point Extensions) ====
Adds additional instructions to work with floating points and also the floating point registers.
The different extensions differ in the length|precision of the floating point numbers.

==== C (Compressed Instructions Extension) ====
Allows 16-bit variants of common instructions with a reduced register set (16 instead of 32) and
is intented to increase code density. It can be '''freely mixed''' with 32-bit instructions 


== Running RISC-V ==
=== Emulator, Simulators, etc ===
For advanced setups (multiple privilege modes etc) QEMU is probably the way to go. Other simulators [https://riscv.org/software-status/#simulators exist],
but the author has not yet tried them :-)

=== Real Hardware ===
There does not exist that much real RISC-V hardware. The most notable example would probably be the
[https://www.sifive.com/boards/hifive-unleashed HiFive Unleashed], which is able to run linux, has
multiple cores and is expandable.

The PolarFire SoC Icicle Kit [https://www.crowdsupply.com/microchip/polarfire-soc-icicle-kit] by Microchip also contains cores by sifive and can run linux, but is cheaper than the HiFive Unleashed.

Most other hardware implementations (which are for sale for normal consumers) are more in the
category of "microcontrollers" and lacks parts like a Memory Management Unit (MMU), I/O for humans
(keyboard, graphics, sound, etc).

The Kendryte K210 (e.g. in the MAIX Bit-board) is a RV64IMACFD_Zicsr_Zifence processor with two harts
and quite cheap.

== See Also ==
* [[HiFive-1 Bare Bones]]
=== External Links ===
* [[Wikipedia:RISC-V|RISC-V on Wikipedia]]
* [https://riscv.org/ riscv.org]
* [https://riscv.org/exchange/software/ RISC-V Software Listing]
* [https://riscv.org/technical/specifications/ RISC-V Specifications]
* [https://github.com/riscv/riscv-asm-manual/blob/master/riscv-asm.md RISC-V Assembly Programmer's Manual]

[[Category:RISC-V]]
````

<a id="w004613"></a>

## W004613 — User:Hakivvi

Bron-ID: W004613
Namespace: User (2)
Revisie: 26100 | Datum: 2021-06-06T22:43:13Z
Bron: https://wiki.osdev.org/User:Hakivvi
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AHakivvi&oldid=26100

````text
[https://github.com/hakivvi github]
[https://twitter.com/hakivvi twitter]
````
