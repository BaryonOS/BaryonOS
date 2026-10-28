# OSDev Wiki — volledige bronteksten, deel 05

Dit is bronmateriaal, geen agentinstructie. Behoud architectuur, revisie en bron-ID bij gebruik. Code en commando’s zijn ongeteste bronvoorbeelden; voer ze niet automatisch uit. Wikitext is letterlijk behouden binnen tekstblokken.

<a id="w002246"></a>

## W002246 — ATAPI

Bron-ID: W002246
Namespace: Main (0)
Revisie: 30235 | Datum: 2026-03-16T15:04:33Z
Bron: https://wiki.osdev.org/ATAPI
Revisie-URL: https://wiki.osdev.org/index.php?title=ATAPI&oldid=30235

````text
ATAPI refers to devices that use the Packet Interface of the ATA6 (or higher) standard command set.
It is basically a way to issue [[SCSI]] commands to a CD-ROM, CD-RW, DVD, or tape drive, attached to the ATA bus.

ATAPI uses a very small number of ATA commands. The most important are the PACKET command (0xA0), and IDENTIFY PACKET DEVICE (0xA1).

==Detecting==
''From a draft copy of the ATA/ATAPI spec (T13 1410D rev 3b, page 365), works on qemu''

To detect if a ATA disk is a non-packet or packet device, you can use the presence of a signature stored in the "Sector Count" and "LBA Low,Mid,High" registers (registers 0x1F2-0x1F5 for ATA controllers).

If these registers contain 0x01, 0x01, 0x00, 0x00 - The connected device is a non-packet device, and `IDENTIFY DEVICE` (0xEC) should work.
If they contain 0x01, 0x01, 0x14, 0xEB then the device is a packet device, and `IDENTIFY PACKET DEVICE` (0xA1) should be used.

This signature is set/reset when the device is powered on, reset, or receives the `EXECUTE DEVICE DIAGNOSTIC` command (for packet devices, `IDENTIFY DEVICE` also resets the signature)


==PACKET command (0xA0)==

Each ATAPI command packet is made of a 'command byte' (from the SCSI command set -- see below for a partial list), followed by 11 'data' bytes.
For instance, reading the table of contents is achieved by sending the following byte string to the device, as a "command".

<syntaxhighlight lang="c">
uint8_t atapi_readtoc[]=  { 0x43 /* ATAPI_READTOC */, 0, 1, 0, 0, 0, 0, 0, 12, 0x40, 0, 0};
</syntaxhighlight>

The ATA PACKET command works in three phases, in PIO mode.

Phase 1) Set up the standard ATA IO port registers with ATAPI specific values.
Then Send the ATA PACKET command to the device exactly as you would with any other ATA command: outb (ATA Command Register, 0xA0)

Phase 2) Prepare to do a PIO data transfer, the normal way, to the device.
When the device is ready (BSY clear, DRQ set) you send the ATAPI command string (like the one above) as a 6 word PIO transfer to the device.

Phase 3) Wait for an IRQ. When it arrives, you must read the LBA Mid and LBA High IO port registers. They tell you the packet byte count that the ATAPI drive will send to you, or must receive from you. In a loop, you transmit that number of bytes, then wait for the next IRQ.

In DMA mode, only the first two phases happen. The device handles the details of phase 3 itself, with the PCI drive controller.


==IDENTIFY PACKET DEVICE command (0xA1)==

This command is a "normal" ATA PIO mode command, used during initialization. It is an exact mirror of the ATA IDENTIFY command, except that it only returns information about ATAPI devices. Use it in exactly the same way as you use IDENTIFY, for the meanings of the data returned, you can refer to page 107 of [https://node1.123dok.com/dt01pdf/123dok_us/001/139/1139315.pdf.pdf?X-Amz-Content-Sha256=UNSIGNED-PAYLOAD&X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=7PKKQ3DUV8RG19BL%2F20221119%2F%2Fs3%2Faws4_request&X-Amz-Date=20221119T211954Z&X-Amz-SignedHeaders=host&X-Amz-Expires=600&X-Amz-Signature=c4154afd7a42db51d05db14370b8e885c3354c14cb66986bffcfd1a9a0869ee2 this pdf] ([https://web.archive.org/web/20221119212108/https://node1.123dok.com/dt01pdf/123dok_us/001/139/1139315.pdf.pdf?X-Amz-Content-Sha256=UNSIGNED-PAYLOAD&X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=7PKKQ3DUV8RG19BL%2F20221119%2F%2Fs3%2Faws4_request&X-Amz-Date=20221119T211954Z&X-Amz-SignedHeaders=host&X-Amz-Expires=600&X-Amz-Signature=c4154afd7a42db51d05db14370b8e885c3354c14cb66986bffcfd1a9a0869ee2 archive.org]).

==Disc Content==
Optical media and drives are governed by the MMC part of SCSI specs.
They are structured in sessions and tracks.
The readable entities are called Logical Track. They are contiguous 
strings of blocks. Some media types can bear several sessions with 
several tracks each, others bear only one session with one track.

For more informations about assessing, reading and writing 
optical media, see the section about
[[Optical_Drive#Readable_Disc_Content|Readable Disc Content]]
of article [[Optical Drive]].

==x86 Directions==

Important note: on the Primary bus, the standard set of ATA IO ports is 0x1F0 through 0x1F7. In much or all of the ATAPI documentation, you will see this set of IO ports called the "Task File". The term seems very confusing.

First, you need to have a buffer. If it is going to be a DMA buffer, it needs to follow the PRD rules (see the ATA/ATAPI using DMA article). If it is going to be a PIO buffer, then you need to know the size of the buffer. Call this size "maxByteCount". It must be an unsigned word, and 0 is illegal for PIO mode. A value of zero is <b>mandatory</b> for DMA mode, no matter what size the PRD buffers are.

Assume that the command is in words Command1 through Command6. Device is the Primary slave. Select the target device by setting the master/slave bit in the Device Select Register. There are no other bits needed.

<syntaxhighlight lang="c">
outb (0x1F6, slavebit<<4);
</syntaxhighlight>

If the command is going to use DMA, set the Features Register to 1, otherwise 0 for PIO.

<syntaxhighlight lang="c">
outb (0x1F1, isDMA);
</syntaxhighlight>

The Sectorcount Register and LBA Low Register are unused currently. Send maxByteCount in LBA Mid and LBA High Registers.

<syntaxhighlight lang="c">
outb (0x1F4, (maxByteCount & 0xff));
outb (0x1F5, (maxByteCount >> 8));
</syntaxhighlight>

Send the ATAPI PACKET command to the Command Register

<syntaxhighlight lang="c">
outb (0x1F7, 0xA0);
</syntaxhighlight>

Wait for an IRQ, or poll for BSY to clear and DRQ to set.

Then send the ATAPI command as 6 words, to the data port.

<syntaxhighlight lang="c">
outw (0x1F0, Command1);
outw (0x1F0, Command2);
outw (0x1F0, Command3);
outw (0x1F0, Command4);
outw (0x1F0, Command5);
outw (0x1F0, Command6);
</syntaxhighlight>

Then wait for another IRQ. You cannot poll.


If this was a DMA command (isDMA == 1), then you are done. When the IRQ arrives, the transfer is complete.

If it was a PIO command, when the IRQ arrives, read the LBA Mid and LBA High Registers. This is vital. You told the drive the <i>maximum</i> amount of data to transfer at one time. Now the drive has to tell you the <b>actual</b> transfer size.

Once you have the transfer size (bytecount = LBA High << 8 | LBA Mid), do the PIO transfer.

<syntaxhighlight lang="c">
wordcount = bytecount/2;
</syntaxhighlight>

loop on inw(0x1F0) or outw(0x1f0) wordcount times.

If the transfer is complete, BSY and DRQ will clear. Otherwise, wait for the next IRQ, and read or write the same number of words again.

Notes: there is a possible future change planned to increase the length of ATAPI command strings to 8 words. Check the two bottom bits of ATAPI Identify word 0 to verify 6 or 8 word command size.

Once again, if you use polling to check BSY, DRQ, and ERR after sending the PACKET command, then you should probably ignore the ERR bit for the first four loops. (ATAPI calls this the "CHECK" bit, instead of ERR, but it means the same thing.)

==Complete Command Set==
{|border="1"
|-
!  SCSI Command Name
!  Command Byte (OpCode)
|-
|  TEST UNIT READY
|  0x00
|-
|  REQUEST SENSE
|  0x03
|-
|  FORMAT UNIT
|  0x04
|-
|  INQUIRY
|  0x12
|-
|  START STOP UNIT (Eject device)
|  0x1B
|-
|  PREVENT ALLOW MEDIUM REMOVAL
|  0x1E
|-
|  READ FORMAT CAPACITIES
|  0x23
|-
|  READ CAPACITY
|  0x25
|-
|  READ (10)
|  0x28
|-
|  WRITE (10)
|  0x2A
|-
|  SEEK (10)
|  0x2B
|-
|  WRITE AND VERIFY (10)
|  0x2E
|-
|  VERIFY (10)
|  0x2F
|-
|  SYNCHRONIZE CACHE
|  0x35
|-
|  WRITE BUFFER
|  0x3B
|-
|  READ BUFFER
|  0x3C
|-
|  READ TOC/PMA/ATIP
|  0x43
|-
|  GET CONFIGURATION
|  0x46
|-
|  GET EVENT STATUS NOTIFICATION
|  0x4A
|-
|  READ DISC INFORMATION
|  0x51
|-
|  READ TRACK INFORMATION
|  0x52
|-
|  RESERVE TRACK
|  0x53
|-
|  SEND OPC INFORMATION
|  0x54
|-
|  MODE SELECT (10)
|  0x55
|-
|  REPAIR TRACK
|  0x58
|-
|  MODE SENSE (10)
|  0x5A
|-
|  CLOSE TRACK SESSION
|  0x5B
|-
|  READ BUFFER CAPACITY
|  0x5C
|-
|  SEND CUE SHEET
|  0x5D
|-
|  REPORT LUNS
|  0xA0
|-
|  BLANK
|  0xA1
|-
|  SECURITY PROTOCOL IN
|  0xA2
|-
|  SEND KEY
|  0xA3
|-
|  REPORT KEY
|  0xA4
|-
|  LOAD/UNLOAD MEDIUM
|  0xA6
|-
|  SET READ AHEAD
|  0xA7
|-
|  READ (12)
|  0xA8
|-
|  WRITE (12)
|  0xAA
|-
|  READ MEDIA SERIAL NUMBER / SERVICE ACTION IN (12)
|  0xAB / 0x01
|-
|  GET PERFORMANCE
|  0xAC
|-
|  READ DISC STRUCTURE
|  0xAD
|-
|  SECURITY PROTOCOL OUT
|  0xB5
|-
|  SET STREAMING
|  0xB6
|-
|  READ CD MSF
|  0xB9
|-
|  SET CD SPEED
|  0xBB
|-
|  MECHANISM STATUS
|  0xBD
|-
|  READ CD
|  0xBE
|-
|  SEND DISC STRUCTURE
|  0xBF
|}
==x86 Examples==
Here is an example adapted from a working driver implementation.
<syntaxhighlight lang="c">
// Handy register number defines
#define DATA 0
#define ERROR_R 1
#define SECTOR_COUNT 2
#define LBA_LOW 3
#define LBA_MID 4
#define LBA_HIGH 5
#define DRIVE_SELECT 6
#define COMMAND_REGISTER 7

// Control register defines
#define CONTROL 0x206

#define ALTERNATE_STATUS 0

static __inline void insw(uint16_t __port, void *__buf, unsigned long __n) {
	__asm__ __volatile__("cld; rep; insw"
			: "+D"(__buf), "+c"(__n)
			: "d"(__port));
}

static __inline__ void outsw(uint16_t __port, const void *__buf, unsigned long __n) {
	__asm__ __volatile__("cld; rep; outsw"
			: "+S"(__buf), "+c"(__n)
			: "d"(__port));
}

// This code is to wait 400 ns
static void ata_io_wait(const uint8_t p) {
	inb(p + CONTROL + ALTERNATE_STATUS);
	inb(p + CONTROL + ALTERNATE_STATUS);
	inb(p + CONTROL + ALTERNATE_STATUS);
	inb(p + CONTROL + ALTERNATE_STATUS);
}

// Reads sectors starting from lba to buffer
int read_cdrom(uint16_t port, bool slave, uint32_t lba, uint32_t sectors, uint16_t *buffer) {
        // The command
	volatile uint8_t read_cmd[12] = {0xA8, 0,
	                                 (lba >> 0x18) & 0xFF, (lba >> 0x10) & 0xFF, (lba >> 0x08) & 0xFF,
	                                 (lba >> 0x00) & 0xFF,
	                                 (sectors >> 0x18) & 0xFF, (sectors >> 0x10) & 0xFF, (sectors >> 0x08) & 0xFF,
	                                 (sectors >> 0x00) & 0xFF,
	                                 0, 0};

	outb(port + DRIVE_SELECT, 0xA0 & (slave << 4)); // Drive select
	ata_io_wait(port);
	outb(port + ERROR_R, 0x00); 
	outb(port + LBA_MID, 2048 & 0xFF);
	outb(port + LBA_HIGH, 2048 >> 8);
	outb(port + COMMAND_REGISTER, 0xA0); // Packet command
	ata_io_wait(port); // I think we might need this delay, not sure, so keep this
 
        // Wait for status
	while (1) {
		uint8_t status = inb(port + COMMAND_REGISTER);
		if ((status & 0x01) == 1)
			return 1;
		if (!(status & 0x80) && (status & 0x08))
			break;
		ata_io_wait(port);
	}

        // Send command
	outsw(port + DATA, (uint16_t *) read_cmd, 6);

        // Read words
	for (uint32_t i = 0; i < sectors; i++) {
                // Wait until ready
		while (1) {
			uint8_t status = inb(port + COMMAND_REGISTER);
			if (status & 0x01)
				return 1;
			if (!(status & 0x80) && (status & 0x08))
				break;
		}

		int size = inb(port + LBA_HIGH) << 8
		           | inb(port + LBA_MID); // Get the size of transfer

		insw(port + DATA, (uint16_t *) ((uint8_t *) buffer + i * 0x800), size / 2); // Read it
	}

	return 0;
}
</syntaxhighlight>

==Detecting a Medium's Capacity==
A medium is any media inserted in the ATAPI Drive, like a CD or a DVD. By using the 'SCSI Read Capacity' command, you can read the last LBA of the medium, then you calculate the medium's capacity using this relationship:

Capacity = (Last LBA + 1) * Block Size;

Last LBA and Block Size are returned after processing the command. Almost all CDs and DVDs use blocks with size of 2KB each.

Processing this command goes in the following algorithm:
* Selecting the Drive [Master/Slave].
* Waiting 400ns for select to complete.
* Setting FEATURES Register to 0 [PIO Mode].
* Setting LBA1 and LBA2 Registers to 0x0008 [Number of Bytes will be returned].
* Sending Packet Command, then Polling.
* Sending the ATAPI Packet, then polling.  ATAPI packet must be 6 words long (12 bytes).
* If there isn't an error, reading 4 Words [8 bytes] from the DATA Register.

The ATAPI packet should be in this formulation:


{| border=1
|-
! bit→<br>↓byte
! width=50 | 7
! width=50 | 6
! width=50 | 5
! width=50 | 4
! width=50 | 3
! width=50 | 2
! width=50 | 1
! width=50 | 0
|-
! 0
| colspan=8 align="center" | Operation code = 25h
|-
! 1
| colspan=3 align="center" | '''LUN'''
| colspan=4 align="center" | Reserved
| align="center" | '''RelAdr'''
|-
! 2
| rowspan=4 colspan=8 align="center" | '''LBA''' (MSB) <br /><br /><br /><br /> '''LBA''' (LSB)
|-
! 3
|-
! 4
|-
! 5
|-
! 6
| colspan=8 align="center" | Reserved
|-
! 7
| colspan=8 align="center" | Reserved
|-
! 8
| colspan=7 align="center" | Reserved
| align="center" | '''PMI'''
|-
! 9
| colspan=8 align="center" | Control
|-
! 10
| colspan=8 align="center" | Reserved*
|-
! 11
| colspan=8 align="center" | Reserved*
|}

Note: The last two reserved fields are ATAPI specific.  They are not part of the SCSI command packet version.

The special control fields in the CDB have the following meaning:

*'''RelAdr''' - indicates that the logical block address (LBA) value is relative (only used with [[SCSI Linked Commands|linked commands]]).
*'''PMI''' - partial medium indicator:
**0 - return value for the last LBA  
**1 - return value for the last LBA after which a substantial delay in data transfer will be encountered (e.g., the current track or cylinder)

The target will return capacity data structured as follows:

{|  border=1
|-
! bit→<br>↓byte
! width=50 | 7
! width=50 | 6
! width=50 | 5
! width=50 | 4
! width=50 | 3
! width=50 | 2
! width=50 | 1
! width=50 | 0
|-
! 0–3
| colspan=8 align="center" | Returned LBA
|-
! 4–7
| colspan=8 align="center" | Block length in bytes
|}

If there is an error after polling, there may be no medium inserted, so this command may be also used to detect whether there is a medium or not, and if there is a medium, its capacity is read.

==Operating Theory==

Quoting [http://suif.stanford.edu/~csapuntz/ide.html The Guide to ATA/ATAPI documentation (stanford.edu)]
<blockquote>
''Hosts control ATAPI devices using SCSI command packets. The SCSI command packets are transported over the ATA interface, instead of the parallel SCSI bus. This cool hack is described in ATA/ATAPI-6.
The set of SCSI command packets applicable to all SCSI devices is described in the [ftp://ftp.t10.org/t10/drafts/spc/spc-r11a.pdf SCSI-3 Primary Commands PDF]. Again, ATAPI devices don't implement all of these, so it's best to consult the associated ATAPI spec for a device.
CD-ROM command packets were originally described in [http://suif.stanford.edu/~csapuntz/specs/INF-8020.PDF INF-8020 PDF]. This document contains many inaccuracies in its description of the ATA bus interface, so please double check any statements against ATA/ATAPI-6. (Unfortunately, some of those inaccuracies were implemented!)''
</blockquote>

==Comments==

==See Also==
===Articles===
* [[ATA/ATAPI Power Management]]
* [[ATA/ATAPI using DMA]]
* [[Optical Drive]]

===External Links===
* http://www.t10.org/ -- T10, the group that creates the SCSI (and therefore ATAPI) command set.
* http://www.t10.org/ftp/x3t9.2/document.87/87-106r0.txt -- Direct link to 1987 documentation of SCSI commands related to CD-ROMS. Old but useful.
* http://www.ata-atapi.com -- Public Domain C driver sourcecode, including SATA, Busmatering DMA, ATAPI -- not perfect, but good.
* [https://web.archive.org/web/20130731111228/https://suif.stanford.edu/~csapuntz/ide.html The Guide to ATA/ATAPI documentation]
* [https://web.archive.org/web/20090203133714/http://suif.stanford.edu/~csapuntz/blackmagic.html Old document about ATA/ATAPI errata]
* [http://www.t13.org/Documents/UploadedDocuments/project/d0948r4c-ATA-2.pdf ATA-2 draft] ATA-1 was HDD-only, ATA-2 enables CD-ROM support via ATAPI spec
* [https://archive.org/details/RedBookAudioRecordingCompactDiscDigitalAudioSystemIEC60908SecondEdition199902ISBN2831846382  ISO 60908:1999 CD-DA] aka Red Book
* [https://www.ecma-international.org/publications/files/ECMA-ST/Ecma-130.pdf ECMA-130 CD-ROM] aka Yellow Book or ISO/IEC 10149:1995
* [https://web.archive.org/web/20041013175705/http://suif.stanford.edu/~csapuntz/specs/INF-8020.PDF Draft of the original ATAPI CD-ROM spec (PDF)] Has partial list of CD-ROM ATAPI commands. <b>MANY</b> errors. If something in this document looks wrong to you, it <b>IS</b> wrong. It is still very informative.
* http://www.osta.org/specs/pdf/udf201.pdf -- UDF filesystem format PDF
* http://www.osdever.net/downloads/docs/iso9660.zip -- ISO 9660 filesystem format
* http://bmrc.berkeley.edu/people/chaffee/jolspec.html -- Joliet filesystem specification
* http://www.osdever.net/downloads/docs/susp112.zip -- Rock Ridge Filesystem Sharing Protocol (POSIX)
* http://www.osdever.net/downloads/docs/rrip112.zip -- Rock Ridge Interchange specification (POSIX)
* https://ata.wiki.kernel.org/index.php/Developer_Resources -- Overview of ATA, SATA, ATAPI and related specifications

[[Category:ATA]]
[[Category:Storage]]
````

<a id="w002247"></a>

## W002247 — Talk:ATA

Bron-ID: W002247
Namespace: Talk (1)
Revisie: 5274 | Datum: 2008-03-25T08:51:15Z
Bron: https://wiki.osdev.org/Talk:ATA
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AATA&oldid=5274

````text
I have created a separate ATA category with many articles. There are many errors in this article. I took all the good stuff out of it, and merged those pieces into the separate articles in the ATA category, where they belonged. So, I am nominating this article to be deleted immediately.
[[User:Bewing|Bewing]] 02:33, 23 March 2008 (CDT)
:Normally, I would check all pages to see if all the information has been moved. However, that would take me a lot of time and I don't have the expertise in this area to determine which parts are correct (and therefore need to be preserved), and which aren't. If you or someone else can confirm all ''correct'' pieces of information are present in the new pages, then you can safely delete this page. - [[User:Combuster|Combuster]] 03:51, 25 March 2008 (CDT)
````

<a id="w002248"></a>

## W002248 — PC Speaker

Bron-ID: W002248
Namespace: Main (0)
Revisie: 30842 | Datum: 2026-07-03T00:11:15Z
Bron: https://wiki.osdev.org/PC_Speaker
Revisie-URL: https://wiki.osdev.org/index.php?title=PC_Speaker&oldid=30842

````text
The PC Speaker is the most primitive sound device available on PC compatible systems. It is characterized by the distinctive "beeps" and "squeaks" that it can be made to produce, and is therefore sometimes referred to as the "PC Beeper" or the "PC Squeaker".

== The Raw Hardware ==

The speaker itself has two possible positions, "in" and "out". This position can be set through bit 1 of port 0x61 on the [[Keyboard Controller]]. If this bit is set (=1), the speaker will move to the "out" position, if it is cleared (=0) then the speaker will move to the "in" position. Moving in and out repeatedly produces audible tones if the speed of repetition (the frequency) is within the range the speaker can reproduce and the human ear can hear. Also, a single movement in or out makes a click sound because it's so fast. Thus, a frequency which is too low to be heard as a tone may be heard as a rattle or buzz. (In fact, any frequency produced by this system also produces higher frequencies; look up "square wave harmonics" if you're interested.)

== Modes of Operation ==

=== Through the Programmable Interval Timer (PIT) ===

The PC Speaker can be connected directly to the output of timer number 2 on the [[Programmable Interval Timer]] by setting bit 0 of port 0x61 (=1). In this mode, when the timer goes "high" (=1) the speaker will move to the "out" position. Likewise, when the timer   goes "low" (=0) the speaker will move to the "in" position. By changing the frequency at which timer 2 "ticks", the PC Speaker can be made to output sound of the same frequency.
This mode is very popular because it is easy to program and because it is asynchronous from the rest of the computer's operation, meaning that it takes very little CPU time.
It should also be noted that this is the "official" way to program the PC Speaker and, if a sound card is present, should be the only way that the PC Speaker is programmed. 

=== Pulse Width Modulation ===
In the absence of a real sound card, the PC Speaker can be used to output low quality digital sound. As stated earlier, the speaker itself has only two possible positions, "in" and "out". More positions are needed though in order to play digital sound. Typically, 256 positions (8 bits) are considered adequate to play comprehensible audio. One popular method used by many early PC games to overcome this limitation is called pulse-width modulation. This technique uses the physical properties of the speaker to allow it to output the (relatively) complex sounds that exist in 8-bit audio.

==== How It Works ====
The PC Speaker takes approximately 60 millionths of a second to change positions. This means that if the position of the speaker is changed from "in" to "out" and then changed back in less than 60 microseconds, the speaker did not have enough time to fully reach the "out" position. By precisely adjusting the amount of time that the speaker is left "out", the speaker's position can be set to anywhere between "in" and "out", allowing the speaker to form more complex sounds.

==== Setting It Up ====
You should first have the PIT interrupt you each time the speaker's position needs to be changed. To play standard audio, the CPU needs to be interrupted 44100 times every second. The easiest way to do this is to (re)program timer zero (0) to interrupt you at 44100Hz.

You also need a way to actually change the speaker's position. This could be done simply through bit 1 of port 0x61, but this option is too slow to be practical. Instead, it is best to (re)program timer 2 to properly set the position of the speaker for you while you wait for the next interrupt (or do other, more useful things with any spare CPU time).

==== Playing the Audio ====
When the PIT interrupts you, you need to program PIT timer 2 so that it puts the PC Speaker in the "out" position for a fraction of 60 microseconds. For 8-bit audio, the number of microseconds can be calculated like this:

<syntaxhighlight lang="C">
microseconds = (sample * 60) / 255;
</syntaxhighlight>

== Sample Code ==
This will work on real hardware and in QEMU if it started with <code>-audiodev pa,id=speaker -machine pcspk-audiodev=speaker</code>. The code also changes the PIT timer 2 frequency, so you will have to reset that when you're done "beep"ing :)
<syntaxhighlight lang="c">
// Play sound using built-in speaker
static void play_sound(uint32_t nFrequence) {
	uint32_t Div;
	uint8_t tmp;
 
    //Set the PIT to the desired frequency
	Div = 1193180 / nFrequence;
	outb(0x43, 0xb6);
	outb(0x42, (uint8_t) (Div) );
	outb(0x42, (uint8_t) (Div >> 8));

       //And play the sound using the PC speaker
	tmp = inb(0x61);
 	if (tmp != (tmp | 3)) {
		outb(0x61, tmp | 3);
	}
}
 
// Make it shut up
static void nosound() {
	uint8_t tmp = inb(0x61) & 0xFC;
	outb(0x61, tmp);
}
 
// Make a beep
void beep() {
	 play_sound(1000);
	 timer_wait(10);
	 nosound();
     //set_PIT_2(old_frequency);
}
</syntaxhighlight>

== See Also ==

=== External links ===

* [http://en.wikipedia.org/wiki/Pulse-width_modulation Wikipedia on pulse-width modulation]

=== Forum Threads ===

* [http://forum.osdev.org/viewtopic.php?f=13&t=17293 Sample PC speaker code]

[[Category:Sound]]
````

<a id="w002250"></a>

## W002250 — Talk:Parallel port

Bron-ID: W002250
Namespace: Talk (1)
Revisie: 5288 | Datum: 2008-03-26T09:37:28Z
Bron: https://wiki.osdev.org/Talk:Parallel_port
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AParallel_port&oldid=5288

````text
== Pins 'n' connectors ==
''I believe that it is serial ports that come in 9 and 25 pin varieties. I'm pretty sure Parallel is only 36 and 25. Can anybody confirm this? [[User:Yayyak|Yayyak]] 18:16, 23 March 2008 (CDT)''
: The parallel port has 8 data bits, and at least one V+ and V-. That's already more than the 9 pins you try to have on that connector. Google finds 36-pin parallel ports, I use both 9 and 25-pin serial ports. (Actually, I use UTP cables to connect serial ports over longer distances via converters, which have 8 pins/wires) - [[User:Combuster|Combuster]] 04:37, 26 March 2008 (CDT)
````

<a id="w002251"></a>

## W002251 — Real mode

Bron-ID: W002251
Namespace: Main (0)
Revisie: 5290 | Datum: 2008-03-26T11:35:54Z
Bron: https://wiki.osdev.org/Real_mode
Revisie-URL: https://wiki.osdev.org/index.php?title=Real_mode&oldid=5290
Doorverwijzing: Real Mode

````text
#REDIRECT [[Real Mode]]
````

<a id="w002252"></a>

## W002252 — Protected mode

Bron-ID: W002252
Namespace: Main (0)
Revisie: 5292 | Datum: 2008-03-26T11:37:12Z
Bron: https://wiki.osdev.org/Protected_mode
Revisie-URL: https://wiki.osdev.org/index.php?title=Protected_mode&oldid=5292
Doorverwijzing: Protected Mode

````text
#REDIRECT [[Protected Mode]]
````

<a id="w002253"></a>

## W002253 — Talk:Protected mode

Bron-ID: W002253
Namespace: Talk (1)
Revisie: 5294 | Datum: 2008-03-26T11:37:12Z
Bron: https://wiki.osdev.org/Talk:Protected_mode
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AProtected_mode&oldid=5294
Doorverwijzing: Talk:Protected Mode

````text
#REDIRECT [[Talk:Protected Mode]]
````

<a id="w002254"></a>

## W002254 — Talk:ATAPI

Bron-ID: W002254
Namespace: Talk (1)
Revisie: 15313 | Datum: 2013-10-19T12:37:24Z
Bron: https://wiki.osdev.org/Talk:ATAPI
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AATAPI&oldid=15313

````text
== Shouldn't most of the second half be in an SCSI page ? ==

I am the developer of libburn, which operates CD, DVD,
and BD drives via SCSI commands (SPC, SBC, MMC).

I wonder why no http://wiki.osdev.org/SCSI or ./MMC exists.
It could consolidate the info about the commands to be used
on optical drives attached via "real SCSI", PATA, SATA, USB, ...

OS developers who want to enable porting of CD/DVD/BD burn 
programs need to offer an SCSI passthrough interface.
This interface has to transmit SCSI command descriptor blocks 
and payload data to the drive. Then it has to return payload 
data and/or indicate that there emerged sense data (usually 
error conditions). An example is Linux ioctl SG_IO.

The SCSI gestures of writing to optical media are described in
http://www.libburnia-project.org/browser/libburn/trunk/doc/cookbook.txt
Interesting for those OS developers who want to integrate 
optical burning into the OS itself.

-----------

BTW: The last four of the external links should rather go to
http://wiki.osdev.org/ISO_9660

[[User:Scdbackup|Scdbackup]] 15:12, 15 October 2013 (CDT)

: I have created a new article [[Optical Drive]] which gives an MMC
: oriented overview of handling such drives.
:
: There would still be missing an article between [[ATAPI]] and [[Optical Drive]]
: which takes most of the SCSI related stuff from ATAPI, and from Optical Drive
: the sections before [[Optical_Drive#Readable_Disc_Content]].
: But i myself have only experience with optical drives. The SCSI article
: would also have to refer to tapes and hard disks.
: [[User:Scdbackup|Scdbackup]] 07:35, 19 October 2013 (CDT)

== CD-DA is not related to Joliet ==

The page says:

  An AudioCD (CD-DA) stores data in a Joliet filesystem.

This is plain wrong. The only filesystem-like structure on CD-DA
is the two-level hierarchy of sessions and tracks. CD-DA blocks
cannot be read by the usual SCSI command READ, but rather by
SCSI command READ CD. They bear 2352 bytes of audio data each.

Joliet is a Microsoft extension to ISO 9660 filesystems.
(One may of course store audio files in a filesystem. But
that is not CD Digital Audio.)

---------

Messed up too:

  A data CDROM stores data in an ISO 9660 filesystem.
  A DVD may store data in a UDF or RockRidge filesystem.

Rock Ridge is the POSIX-enabling extension of ISO 9660 filesystems.
Data CD, DVD, and BD (Blu-ray) can store any kind of data in blocks
of 2048 bytes each. ISO 9660 is widely readable. Joliet and Rock Ridge
enhance it for MS-Windows and Unix.

UDF (based on ECMA-167) is intended as successor of ISO 9660 (ECMA-119).
But except by the specs for video entertainment on DVD and Blu-ray,
there is not much sincere use of it.

[[User:Scdbackup|Scdbackup]] 15:31, 15 October 2013 (CDT)

: The problematic statements are now replaced by a short overview of
: the session-track structure and a link to [[Optical Drive]]
: [[User:Scdbackup|Scdbackup]] 07:37, 19 October 2013 (CDT)
````

<a id="w002255"></a>

## W002255 — C Sharp

Bron-ID: W002255
Namespace: Main (0)
Revisie: 29217 | Datum: 2024-08-13T19:02:19Z
Bron: https://wiki.osdev.org/C_Sharp
Revisie-URL: https://wiki.osdev.org/index.php?title=C_Sharp&oldid=29217

````text
<p style="margin-left:2em; font-size:smaller;">Please note that the correct title of this article is C#, however because of technical restrictions it's called "C Sharp".</p>

C# is an object-oriented programming language developed by the .NET Foundation and standardized by the ECMA and ISO. Its syntax is similar to [[C++]], but with significant differences in functionality. Typically, source files written in C# compile into an intermediate bytecode language called [[CIL]] (also referred to as ''IL'') which was then just-in-time compiled into machine code by the .NET runtime. As of .NET 7, traditional ahead-of-time compilation is also available. A standard library (also called a ''system module'' throughout .NET's tooling) provides well-known base classes, such as <code>System.Object</code>, and is automatically referenced by regular C# projects. For kernels written in C#, though, a custom standard library usually needs to be provided. 
== Why write a kernel in C#? ==

The primary benefits of writing a kernel in C# is the type and memory safety the language provides. If a region of code is not explicitly marked as <code>unsafe</code>, it can be safely assumed that it will not perform any invalid memory accesses. This idea can be extended to trivially formally verify a region of IL bytecode as being memory safe, simply by excluding certain instructions that are known to be unsafe. As a verified region of code is certain to be well-behaved, a kernel may run it without any kind of address space switching, nor protection ring changes. Given the fact that two safe (or '''managed''') processes can run in the same address space, there exists potential for an extremely efficient implementation of a microkernel, as an IPC call would be equivalent to a plain function invocation. A similar concept has been explored in the [https://www4.cs.fau.de/Projects/JX/publications/jx-usenix.pdf JX Operating System] with Java - however, it's important to make the distinction that Java manages dependencies in a fundamentally different way than C#, and is known to be more restrictive.

The disadvantages of writing a C# kernel are the complexity needed for a custom standard library to properly function. In addition to regular functions one may find in the standard libraries of Rust or C, C# also requires the system module to provide a garbage collection implementation, as well as many internal data types that the Roslyn C# compiler (<code>csc</code>) uses for newer syntax features. Depending on the IL-to-native code compiler used, intrinsic features (which are often undocumented) also need to be implemented.

Writing a C# kernel also requires a capable GC. A good choice for a garbage collector living in kernel-mode would be an on-the-fly GC, which only needs to stop one thread at a time. This can happen when the thread is idle (e.g. waiting for an I/O operation to complete), meaning no noticeable pause is made to the thread's execution. The kernel's code should also avoid unnecessary allocations that may stress the GC; for example, instead of allocating a temporary buffer with <code>var example = new int[16]</code>, a better solution would be to use a small stack allocation, e.g. <code>Span<int> example = stackalloc int[16]</code>. Another viable option is to use custom pool/allocation functions. As of C# 8.0, you may use the disposable ref structs feature to safely and automatically free temporary buffers that are retrieved with such methods, similar to C++'s RAII. Both stack-allocated memory and pool/custom allocation functionality bypass the GC.
== Compiling your kernel to machine code ==

There are two approaches one could take to converting C# code to machine code:

# Use a compiler which directly converts C# (or a superset of it) to machine code. This is the approach taken by Microsoft's research OS, [http://research.microsoft.com/en-us/projects/singularity/ Singularity], which defines the [http://en.wikipedia.org/wiki/Sing_Sharp Sing# language] for easier interaction with the underlying hardware. The Sing# code is converted to machine code by their compiler, [http://en.wikipedia.org/wiki/Bartok_%28compiler%29 Bartok].
# Create a compiler which converts CIL to machine code. This leverages the C# to CIL compilers already present in .NET or the Mono project. This is the approach taken by many open source C# kernels, including [[squeak:projects/sharpos/|SharpOS]], [http://www.gocosmos.org/index.en.aspx Cosmos], [http://www.mosa-project.org/ the MOSA project] and [https://github.com/jncronin/tysos tysos].

Both of the above approaches could be performed ahead-of-time (AOT) or just-in-time (JIT). The AOT approach is simpler to begin with - you have your compiler running on your development system which is used to produce executable files (e.g. [[ELF]]) which can then be loaded directly by a standard boot loader.  A JIT design would require a not insubstantial amount of code that is loaded before your kernel which then converts your kernel into machine code before running it. The problem here is that the JIT compiler would likely need a lot of services which only your kernel could compile. The best combination is probably to AOT compile your kernel and JIT compiler and have them loaded, and then all other processes can be JIT compiled.

An AOT compiler is built-in to .NET >=7.0. In order to use it, you can:

* Use <code>ilc</code> - the IL compiler, internally used for Native AOT (NAOT) compilation, provided by .NET >7.0. ILC may be invoked manually by installing the .NET tool with <code>dotnet tool install -g ilc</code>. The compiler accepts managed CLR assemblies, in the form of DLL files. You can get such files by compiling your C# source code with <code>csc</code>, which can be installed the same way as ILC. 
* Use Native AOT deployment provided by the .NET build system. Note that this will require large amounts of tweaking in order to customize the default Linux/Windows build rules. For an example, the [https://github.com/MichalStrehovsky/zerosharp ZeroSharp: C# for systems programming] demonstration may be of interest. The repository only uses features that are built-in to .NET.

Alternatively, for a more streamlined experience, the [https://github.com/bflattened/bflat/ bflat] toolchain provides a simple way to use ILC, alongside a simple, systems-programming-oriented "zerolib" standard library. It automatically interfaces with ILC. Do note that bflat is ''not'' integrated with .NET - it also uses its a separate version of the Roslyn C# compiler.

== The runtime ==

Several C# commands (or their equivalent in CIL) require a functioning runtime (in .NET, called the CLR - the Common Language Runtime). For example, the expected result of the <code>newobj</code> CIL command is to create an object on a garbage managed heap. This would require a <code>malloc</code> function and then some method to perform garbage collection on the heap. With clever coding (mainly using static or stack-defined objects), it is possible to have your kernel not use the newobj command until you have initialized your memory allocator.

You may also need to provide an encapsulation of the run-time type information for a class and some way to implement the <code>System.Object.GetType()</code> and <code>typeof()</code> functions to return a class semantically equivalent to <code>System.Type</code>. This is not necessarily required when using Native AOT (ILC) compilation.
== The standard library ==

The standard library is a basic component of the .NET runtime and provides definitions of all the standard types (e.g. <code>System.Int32</code> for a 32-bit signed integer, <code>System.String</code> for a string, <code>System.Collections.Generic.List<T></code> for a list of objects of type <code>T</code>). To write any sort of meaningful code in your kernel, you are probably going to need to provide implementations for most of these.

While using the BCL (Base Class Library - .NET's standard library) is theoretically possible, it is generally not suited for any kind of kernel development - it assumes that the running program is in user-mode, and that most system facilities, such as file I/O, are already configured. Projects such as Cosmos handle this by selectively patching certain methods of the BCL which use code intrinsic to select operating system - called ''plugging''.

Writing a custom core library more suitable for kernel-mode programming is a better solution - the amount of classes that need to be implemented depends on the toolset used. For reference, you can use [https://github.com/bflattened/bflat/tree/master/src/zerolib bflat's minimal runtime library].

== See Also ==
=== Articles ===
* [[CIL]]
* [[C Sharp Bare Bones]]

=== External Links ===
* [http://en.wikipedia.org/wiki/C_Sharp_%28programming_language%29 C#] on Wikipedia
* [https://flattened.net/ bflat - C# as you know it but with Go-inspired tooling]

[[Category:Languages]]
[[Category:CSharp]]
````

<a id="w002256"></a>

## W002256 — Talk:C Sharp

Bron-ID: W002256
Namespace: Talk (1)
Revisie: 14144 | Datum: 2012-12-11T22:13:17Z
Bron: https://wiki.osdev.org/Talk:C_Sharp
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AC_Sharp&oldid=14144

````text
== Too short an answer? ==
I think the remark that you (''only'') need a interpreter is slightly misleading. Yes you can write an OS in C#, No you can't cleanly do it in '''pure C#'''. You need assembly to access processor stubs, and possibly a higher language (c++) to run the CLI interpreter. And memory management for the object-oriented part. And probably several other runtime thingies I can't foresee because I don't know the implementation details of the language. I therefore think that a remark about the feasability is at its place. Also, could someone with in-depth knowledge of the ABI give some ideas what the runtime part actually involves. - [[User:Combuster|Combuster]] 14:59, 31 March 2008 (CDT)

== Cosmos ==

I really think this deserves more than a simple link. Cosmos is not just some OS project, it's OS '''kit'''. I think this is probably where we should be pointing people to when they ask "can I write my own OS in C#". It appears to make things super easy, even for unexperienced OS devers --[[User:Earlz|Earlz]] 22:02, 10 December 2012 (CST)

: I'm slightly biased here so I will try to keep my arguments as neutral as possible.  The mention of Cosmos (and other C# OSes) in the article is in a section that highlights the compilation strategy employed by them (i.e. relying on .NET csc/mono mcs to compile to CIL and then AOT compiling the kernel to machine code) as opposed to the Singularity approach (a variant of C# directly compiled to machine code) rather than to provide a list of C# OSes per se.  I'm sure we could keep a link to C# OSes at the bottom of the article explaining the differences between them if you like, or possibly put it in a separate article etc.
: Secondly, I have some reservations about making Cosmos the first point of call for beginners wanting to write a C# OS.  IMHO the first point of call should be the [[Languages]] article which essentially says 'you can't or its very difficult'.  The Cosmos project provides an excellent kernel itself which you have to instantiate and you can then call functions from the standard .NET library like Console.WriteLine etc (as shown in the screenshot on their site).  In this sense, pointing people to it would be akin to replacing our [[Barebones]] or [[Babystep]] tutorials with a link that simply points to a C kernel with integrated C library and then saying build upon this - it it no way teaches people how to write an OS (by which I mean the low level hardware interface code and memory management + scheduling + IPC etc).
: However, this is a wiki and a democracy therefore please don't be afraid to make any changes you want (with the proviso people may change your edits).  - [[User:Jnc100|John]] 16:13, 11 December 2012 (CST)
````

<a id="w002257"></a>

## W002257 — Creating a 64-bit kernel

Bron-ID: W002257
Namespace: Main (0)
Revisie: 30270 | Datum: 2026-03-16T17:47:34Z
Bron: https://wiki.osdev.org/Creating_a_64-bit_kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Creating_a_64-bit_kernel&oldid=30270

````text
{{Rating|2}}{{Template:Kernel designs}}
{{In_Progress}}
{{Disputed|Talk:Creating_a_64-bit_kernel#Disputed}}

== Prerequisites ==

Make sure that you have the following done before proceeding:

* Have [[GCC_Cross-Compiler|built a cross-compiler]] for the x86_64-elf target.
* Read up on long mode and how to [[X86-64|initialize/use it]] (if you intend on not using a 64-bit aware bootloader or rolling your own).
* Decide now on how to load your kernel - your own bootloader, [[GRUB]] (with separate loader executable), GRUB2 (elf64 + 32-bit bootstrap code), or a 64-bit capable bootloader such as [[Limine]] or [[BOOTBOOT]].

== The Main Kernel ==
The kernel should run in a uniform environment. Let's make this simple for now...

=== kernel.c ===
<syntaxhighlight lang="c">
void kernel_main(void)
{
    /* What goes here is up to you */
}
</syntaxhighlight>

== Compiling ==
Compile each source file like any piece of C code, just remember to use the cross-compiler and the proper options.
Linking will be done later...

<syntaxhighlight lang="bash">
x86_64-elf-gcc -ffreestanding -mcmodel=large -mno-red-zone -mno-mmx -mno-sse -mno-sse2 -c foo.c -o foo.o
</syntaxhighlight>

The -mcmodel=large argument enables us to run the kernel at any 64-bit virtual memory address we want. In fact, using the 'large' code model is discouraged due to its inefficiency, but it can be fine as a start. Check the [[System V ABI|SysV AMD64 ABI]] document for extra details.

You will need to instruct GCC not to use the the AMD64 ABI 128-byte 'red zone', which resides below the stack pointer, or your kernel will be ''interrupt unsafe''. Check this [http://forum.osdev.org/viewtopic.php?t=21720 thread] on the forums for extra context.

We disable [[SSE]] floating point ops. They need special %cr0 and %cr4 setup that we're not ready for. Otherwise, several #UD and #NM exceptions will be triggered.

== Linking ==
The kernel will be linked as an x86_64 executable, to run at a virtual higher-half address. We use a linker script:

=== link.ld ===
 ENTRY(_start)
 SECTIONS
 {
     . = KERNEL_VMA;
 
     .text : AT(ADDR(.text) - KERNEL_VMA)
     {
         _code = .;
         *(.text)
         *(.rodata*)
         . = ALIGN(4096);
     }
 
    .data : AT(ADDR(.data) - KERNEL_VMA)
    {
         _data = .;
         *(.data)
         . = ALIGN(4096);
    }
 
    .eh_frame : AT(ADDR(.eh_frame) - KERNEL_VMA)
    {
        _ehframe = .;
        *(.eh_frame)
         . = ALIGN(4096);
    }
 
    .bss : AT(ADDR(.bss) - KERNEL_VMA)
    {
        _bss = .;
        *(.bss)
 
        /*
         * You usually need to include generated COMMON symbols
         * under kernel BSS section or use gcc's -fno-common
         */
 
         *(COMMON)
        . = ALIGN(4096);
    }
 
    _end = .;
 
    /DISCARD/ :
    {
         *(.comment)
    }
 }

Feel free to edit this linker script to suit your needs. Set ENTRY(...) to your entry function, and KERNEL_VMA to your base virtual address.

You can link the kernel like this:

<syntaxhighlight lang="bash">x86_64-elf-gcc -ffreestanding <other options> -T <linker script> <all object files> -o <kernel executable> -nostdlib -lgcc</syntaxhighlight>

'''Note''': Obviously there is no bootstrap assembly yet, which is the hard part of starting out, and you can't link without it.

== Loading ==
Before you can actually use your kernel, you need to deal with the hard job of loading it. Here are your four options:

=== With your own boot loader ===
This method is the simplest (since you write all the code), though it requires the most work.

I won't give any code, but the basic outline is:
* Set up a stable environment
* Do Protected Mode readying stuff (GDT, IDT, A20 gate, etc.)
* Enter Protected Mode (or skip this step and [[Entering Long Mode Directly|enter long mode directly]])
* Parse kernel [[ELF]] headers (if kernel is separate from executable)
* Set up Long Mode readying stuff (PAE, PML4, etc.) - Remember to set up the higher-half addressing!
* Enter Long Mode by far jump to the kernel entry point in (virtual) memory

=== With a 64 bit aware loader ===
Open Source boot loaders written for long mode kernels already exist, you don't have to reinvent the wheel. Unlike GRUB (which does not support switching to long mode), bootloaders such as [[Limine]] or [[BOOTBOOT]] can load your 64-bit kernel directly by doing all the things listed in the previous section (and more). It saves you the struggle to write and properly link bootstrap code or to implement your own boot loader entirely from scratch. Therefore using a 64-bit aware bootloader is a nice and easy, reasonably bullet-proof, choice for beginners.

=== With legacy GRUB ===
'''Note''': The advise in this section is bit questionable in its current form.
See [[Creating_a_64-bit_kernel_using_a_separate_loader|Creating a 64-bit kernel using a separate loader]]

This requires the use of [[GRUB]] or another multiboot1-compliant loader. This may be the most error free of the four, but creating a multiboot-compatible kernel properly has its own set of pitfalls.

A quick rundown:
* Set up a stable environment
* Read the multiboot information struct to see where GRUB loaded your kernel (look at the module section)
* Parse kernel [[ELF]] headers
* Set up Long Mode readying stuff (PAE, PML4, etc.) - Remember to set up the higher-half addressing!
* Enter Long Mode by far jump to the kernel entry point

Note that this code has to be stored in a elf32 format and must contain the multiboot1-header.

Also remember to set the text section to start at 0x100000 (-Ttext 0x100000) when linking your loader.

Set up GRUB to boot your loader as a kernel in its own right, and your actual kernel as a module. Something like this in menu.lst:
 title My Kernel
 kernel --type=multiboot <loader executable>
 module <kernel executable>

=== With a 32-bit bootstrap in your kernel ===

This requires the use of any ELF64-compatible loader that loads into protected-mode (GRUB2, or patched GRUB Legacy). This may be the simplest in the long run, but is more difficult to set up.
Note that GRUB2, which implements [http://download.savannah.gnu.org/releases-noredirect/grub/phcoder/multiboot.pdf Multiboot 2], does not support switching into long mode.

First, create an assembly file like the following, which will set up virtual addressing and long mode:
==== bootstrap.S ====
 .section .text
 .code32
 
 multiboot_header:
     (only needed if you're using multiboot)
 
 bootstrap:
     (32-bit to 64-bit code goes here)
     (jump to 64-bit code)

Then, add the following to your original linker file:
==== link.ld ====
 ...
 ENTRY(bootstrap)
 ...
 SECTIONS
 {
     . = KERNEL_LMA;
 
     .bootstrap :
     {
         <path of bootstrap object> (.text)
     }
 
     . += KERNEL_VMA;
 
     .text : AT(ADDR(.text) - KERNEL_VMA)
     {
         _code = .;
         *(EXCLUDE_FILE(*<path of bootstrap object>) .text)
         *(.rodata*)
         . = ALIGN(4096);
     }
 ...
The above edits allow the linker to link the bootstrap code with physical addressing, as virtual addressing is set up by the bootstrap.
Note that in this case, KERNEL_VMA will be equivalent to 0x0, meaning that text would have a virtual address at KERNEL_LMA + KERNEL_VMA instead of just at KERNEL_VMA.
Change '+=' to '=' and your bootstrap code if you do not want this behaviour.

Compile and link as usual, just remember to compile the bootstrap code as well!

Set up GRUB2 to boot your kernel (depends on your bootloader) with grub.cfg:
 menuentry "My Kernel" {
     multiboot <kernel executable>
 }

==== With Visual C++ ====
The technique for creating a 64 bit kernel with a 32 bit bootstrap is similar to GCC. You need to create an assembly bootstrap with nasm (masm may work, but the author uses nasm).
Note that this stub '''must''' be assembled to a ''64'' bit object file (-f win64). Your stub then has a BITS 32 directive.
Note that, although nasm will not complain about this, Microsoft link will. It complains about address relocations, due to the memory model settings (/LARGEADDRESSAWARE, which is required for /DRIVER).
As such, you need a method of generating the correct 32 bit code, while fooling link into generating a 64 bit relocation. Here is a macro for you:
<syntaxhighlight lang='asm'>
;Encode 32 bit moves without the ADDR32 issue
%macro mov_abs32 2
%if %1==eax
db 0xB8
%elif %1==ebx
db 0xBB
%elif %1==ecx
db 0xB9
%elif %1==edx
db 0xBA
%elif %1==edi
db 0xBF
%elif %1==esi
db 0xBE
%elif %1==ebp
db 0xBD
%elif %1==esp
db 0xBC
%else
%error "Unknown register"
%endif
dq %2+(0x90909090 << 32)
%endmacro
;This translates as mov %1, %2 NOP NOP NOP NOP
;Example usage:
;mov_abs32 eax, (gdtr-KADDR_OFFSET)
;lgdt [eax]
</syntaxhighlight>

== Possible Problems ==
You may experience some problems. Fix them '''immediately''' or risk spending a lot of time debugging later...

=== My kernel is way too big! ===
Try each of the following, in order:
* Try linking your kernel with the option "-z max-page-size=0x1000" to force the linker to use 4kb pages.
* Make sure you're compiling with the -nostdlib option (equivalent to passing the both -nodefaultlibs and -nostartfiles options).
* You can try changing the OUTPUT_FORMAT to elf64-little.
* Try cross-compiling the '''latest''' version of binutils and gcc.

== Kernel Virtual Memory ==
(This section is based on notes by Travis Geiselbrecht (geist) at the osdev IRC channel)

Long mode provides essentially an infinite amount of address space. An interesting design decision is how to map and use the kernel address space. Linux approaches the problem by permanently mapping the -2GB virtual region 0xffffffff80000000 -> 0xffffffffffffffff to physical address 0x0 upwards. Kernel data structures, which are usually allocated by kmalloc() and the slab allocator, reside above the 0xffffffff80000000 virtual base and are allocate from the physical 0 -> 2GB zone. This necessitates the ability of 'zoning' the page allocator, asking the page allocator to returning a page frame from a specific region, and only from that region. If a physical address above 2GB needs to accessed, the kernel temporarily map it to its space in a temporary mapping space below the virtual addresses base. The Linux approach provides the advantage of not having to modify the page tables much which means less TLB shootdowns on an SMP system.

Another approach is to treat the kernel address space as any other address space and dynamically map its regions. This provides the advantage of simplifying the page allocator by avoiding the need of physical memory 'zones': all physical RAM is available for any part of the kernel. An example of this approach is mapping the kernel to 0xfffffff800000000 as usual. Below that virtual address you put a large mapping for the entire physical address space, and use the virtual 0xfffffff800000000 -> 0xffffffffffffffff region above kernel memory area as a temporary mappings space.

==See Also==
===Articles===
* [[X86-64]]
===Forum Threads===
* [http://forum.osdev.org/viewtopic.php?f=8&t=16779 Creating a 64-bit Kernel Tutorial] about this article
* [http://forum.osdev.org/viewtopic.php?p=170634 Linker-script writers beware: COMMON Symbols] on the obscure 'COMMON' symbols and their effect on BSS
* [http://forum.osdev.org/viewtopic.php?t=21720 Long-mode Kernels and the AMD64 ABI 'Red Zone'] on the 'red zone' and its major effect on interrupt handling
* [http://forum.osdev.org/viewtopic.php?f=1&p=136701 Leaving long mode] to protected mode
* [http://forum.osdev.org/viewtopic.php?f=1&t=17213 Switching from long mode to compatibility mode]

[[Category:Tutorials|Creating a 64-bit kernel]]
[[Category:X86-64|Creating a 64-bit kernel]]
[[Category:Kernel]]
````

<a id="w002258"></a>

## W002258 — VFS

Bron-ID: W002258
Namespace: Main (0)
Revisie: 29334 | Datum: 2024-12-30T14:15:40Z
Bron: https://wiki.osdev.org/VFS
Revisie-URL: https://wiki.osdev.org/index.php?title=VFS&oldid=29334

````text
{{Filesystems}}

<!-- {{In Progress}} -->
==Introduction==
[[image:vfs_diagram.png|frame|VFS in relation to kernel]]
A '''Virtual File System''' (VFS) is not an on-disk file system, or a network file system. As such, it's neither a data structure (like ReiserFS, NTFS or FAT), nor a network protocol (like NFS). Actually, it's just an abstraction that many operating systems provide to applications, so don't let the name scare you.

Virtual file systems are used to separate the high-level interface to the file system from the low level interfaces that different implementations (FAT, ext3, etc) may require, thus providing transparent access to storage devices from applications. This allows for greater flexibility, specially if one wants to support several file systems.

The VFS sits between the higher level user space operations over the file system, and the file system drivers. Having a VFS interface implies having the idea of mount points. A mount point is a path inside the virtual file system tree that represents an in-use file system. This file system may be on a local device, in memory or stored on a networked device.

===Disambiguation of the underlying semantics of a VFS===

A VFS, in concrete terms then, provides a uniform access path and subsystem for a group of file systems of the same type. To date, there are three common types of file system: [[Hierarchical VFS Theory|Hierarchical]] (the most common), tag-based and Database file systems.

== VFS Models ==
=== Indexed (*-DOS/NT)===
The VFS model used in *-DOS and Windows NT assigns a letter from the alphabet to each accessible file system on the machine. This type of VFS is the most simple to implement but is restricted to 26 mounted file systems and can get more and more complex as features are added, so you might want to use numbers instead.
When a file is requested the VFS checks what drive the file is on and then passes the request on to the relevant driver.

=== Mount Point List ===
A more complex model is that of a mount point list. This system maintains a list of mounted file systems and where they are mounted. When a file is requested the list is scanned to determine what file system the file is on. The rest of the path is then passed on to the file system driver to fetch the file. This design is a quite versatile one but suffers from speed problems when large amounts of mount points are used.

=== Node Graph (Unix)===
A VFS model that can be very efficient is the Node Graph. This model maintains a graph of file system nodes that can represent a file, folder, mount point or other type of file. A node graph can be faster to traverse than a list but suffers from complexity problems and, if a large amounts of nodes are needed, can take up large amounts of memory.

Each node in a node graph has the name, permissions and inode stored within a structure along with pointers to file IO functions like Read, Write, Read Dir and Find Dir.

=== Compromise ===
These models represent the basics for a VFS to be designed on, they have their problems however. Scanning through a list of mount points and then passing on to the file system the remainder of the path is usable for a simple OS but requires large amounts of repeated code as each driver must be able to parse a path reliably. A node graph on the other hand, requires a node for each file and directory on the system to present in memory at the same time, otherwise features like mount points would have to be constantly refreshed.

A compromise between these two systems would be to have a list of mounted file systems and use that to determine what mount point a file lies on and then use nodes that do not necessarily have to permanently reside in memory to store file information and methods.

== File Abstraction ==
One of the most important part of an OS is to create the file abstraction, that is, hide the underlying file system structure and sector layout from the application. This is usually implemented by '''open''', '''read''', '''write''' and '''close''' system calls (stands for both POSIX and Windows).

=== Opening a file ===
This function is responsible for finding or creating a context (VFS inode or vnode) for the given filename. This usually means to iterate on the aforementioned VFS nodes in memory, and if file is not found, consult the mount list (see above), the file system driver and the storage driver to get the required metadata (in POSIX terminology, fstat struct). The open system call returns an identifier that connects the opened file with that context. This identifier can be globally unique (like FCB), or local to the process (an integer, typical in UNIX-like systems). The pointed context should have information like
* storage device where the opened file resides
* structure describing which sectors belong to this file on the storage (or a simple inode reference)
* file size
* file position (seek offset)
* file access mode (read / write)
It is advisable to create another abstraction layer here and divide the context into two. One structure describing all the information from the FS point of view (vnode), and another structure for the application's point of view (open file struct, only referencing the vnode). The advantage here is, that you can keep a counter of number of open streams in the vnode, which you'll need later. Remember, more process can open the same file for reading, therefore you cannot put file position in the vnode directly. Whether you allocate this open file list in the process' address space or in the VFS, is up to you. But vnodes must be stored in a central place in VFS, otherwise you won't be able to avoid race-conditions and implement proper file locking.

=== Reading or writing ===
Applications know nothing, all they can see is a linear, contigous data stream. This means that the heavy duty of translation falls to the VFS layer. This is a very complex task, and notoriously hard to get it right for the first time. A typical '''read''' should do the following steps:

1. Using the file position and the file size information in the context, it must determine how many sectors needed (depends on FS logical sector size). It's possible that file position is greater or equal of the file size, therefore the answer is zero. In that case VFS must provide a way to indicate EOF condition. If file position and size is greater than file size, then the size argument must be lowered. Note that file position may not be on sector boundary, and even a read of 2 bytes can spread across multiple sectors.

2. Using the file storage information in the context, VFS must create a list of sector addresses for the device. This can be done in a uniform (FS independent) way, although file systems tends to use very different allocation algorithms (FAT clusters, inode indirect tree, extents etc), therefore most VFS implementation delegates this task to the file system driver. Don't forget that file systems use logical sectors.

3. The list of logical sectors must be converted to a list of physical sectors. This usually means to add the starting sector of the partition the file system is on, but could also mean more or less items in the list if logical sector size and physical sector size differ. In short, the VFS (knowing the characteristics of the underlying storage device) must convert LSN list into LBA list.

4. Now that the list of physical sectors is known, the VFS can use readsector calls in the appropriate disk driver. The storage device is stored in the context (providing the device parameter for the call), and the kernel's device tree should tell which driver to use.

5. Once the sectors are in memory, the VFS needs to copy their contents into the reader's buffer. If file position is not multiple of the sector size, then the first sector will start with extra data VFS must skip. If file position plus read size is not multiple of sector size, then the last sector has extra data that VFS must discard.

6. The VFS must do housekeeping in the context. This typically means adjusting the file position and things alike.

7. The VFS may now notify the reader (in POSIX that means to wake up the blocked process).

Write has similar steps, but there the VFS must allocate logical sectors (or clusters, stripes, etc. whatever the file system is using for allocation) by calling the file system driver. With writes, the VFS should read the first and the last sector at a minimum, because they may have extra data, see step 5 above.

=== Closing a file ===
Finally, the close function is responsible for saving any modifications to the file into the storage (in case of writing) and updating the file system metadata. This usually means a call into the file system driver. Finally, free the memory allocated for the open file list entry. If you have a separate vnode + open file list (you should), then the VFS must decrease the vnode's open streams counter, and if that's zero, also free the vnode.

== See Also ==

*[[Symbolic link]]
*[[Hierarchical VFS Theory]]

=== External Links ===
* 2001 Linux VFS article [http://www.freeos.com/articles/3838/ part 1] and [http://www.freeos.com/articles/3851/ part 2] on [http://www.freeos.com/ FreeOS]
* 1996 Linux VFS [http://tldp.org/LDP/khg/HyperNews/get/fs/vfstour.html article] in the [http://tldp.org/LDP/khg/HyperNews/get/khg.html Linux Kernel Hackers' Guide]
* [http://doc.cat-v.org/plan_9/4th_edition/papers/names The Use of Name Spaces in Plan 9] - Describes how Plan 9 takes advantage of both private namespaces and an VFS that matches [http://9p.cat-v.org the 9P distributed file system protocol].
* 1986 "Vnodes: An Architecture for Multiple File System Types in Sun UNIX" (Kleiman) [https://www.cs.fsu.edu/~awang/courses/cop5611_s2024/vnode.pdf] describes a common approach to VFS

[[Category:Filesystems]]
[[Category:OS theory]]
````

<a id="w002259"></a>

## W002259 — Filesystem

Bron-ID: W002259
Namespace: Main (0)
Revisie: 5345 | Datum: 2008-04-12T04:25:55Z
Bron: https://wiki.osdev.org/Filesystem
Revisie-URL: https://wiki.osdev.org/index.php?title=Filesystem&oldid=5345
Doorverwijzing: File Systems

````text
#REDIRECT [[File_Systems]]
````

<a id="w002260"></a>

## W002260 — User:Thepowersgang

Bron-ID: W002260
Namespace: User (2)
Revisie: 10206 | Datum: 2010-05-03T01:49:32Z
Bron: https://wiki.osdev.org/User:Thepowersgang
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AThepowersgang&oldid=10206

````text
Website: http://www.mutabah.net/ (Self hosted, but reliable)

I am a pure C and Intel Assembler programmer. I am creating an operating system called AcessOS that I hope will be able to incorporate the principle of Wine in running both Microsoft Windows and Linux applications natively (sadly, this is just a pipe dream at the moment). The interface of AcessOS is designed to be a simplification of the Unix system using a Virtual File System Layer with user friendly names and transparent device access.

== Acess ==
Acess is my operating system designed with simplicity for the developer in mind. It makes heavy use of the VFS layer, forcing all drivers to expose themselves as files/folders under the DevFS mount (typically /Devices)
Acess has evolved over the years from a tutorial clone to a tangled mess of code that works in two editions, and then to the fully reworked version, Acess 2.
A more detailed description of Acess 2, along with source download and the API documentation is available on my website (see the top of the page).
Essentially, Acess 2 is designed more along the lines of the Linux kernel in that it can use the same source tree for many different architectures, and can be customized at compile/run time for many different jobs.

== RME ==
The Real Mode Emulator (RME) is one of my side projects. It provides the features of VM8086 mode via emulation, allowing long mode OSs to call BIOS routines, or emulate DOS/other 16-bit code.

== EmulOS ==
EmulOS is a project that grew out of the Realmode emulator code, designed similarly to Qemu, but at the kernel level instead of userland. EmulOS v1 managed to get to a semi-complete stage, but did not use memory management (so could run into issues with addresses > 16MiB) and lacked multi-CPU support.
EmulOS 2 attempts to combat this by using a custom semi-microkernel that does task switching and provides an interface to emulated memory. It also is available on my website (Projects->EmulOS 2)
EmulOS 3 is currently in planning and will end up building upon the Acess 2 kernel, allowing it to use the filesystem and hardware drivers available to Acess.
````

<a id="w002262"></a>

## W002262 — Security

Bron-ID: W002262
Namespace: Main (0)
Revisie: 30747 | Datum: 2026-06-13T20:30:30Z
Bron: https://wiki.osdev.org/Security
Revisie-URL: https://wiki.osdev.org/index.php?title=Security&oldid=30747

````text
Security in an OS is a very important issue. It not only includes security from outside threats, such as viruses and hacking, but internal security as well. For example, the OS is responsible for making sure that processes don't access memory outside of their own address space. If a program does so, it must be shutdown to protect other 'well behaved' programs. Operating system security can be divided into two basic categories, high-level and low-level security.

To see articles under this category, browse [[:Category:Security|this list]].

== Low-level Protection Mechanisms==
There are several different low-level protection mechanisms at the disposal of the operating system programmer. The first mechanism, called "CPU Rings" or more simply "rings", controls which CPU instructions are allowed to be executed. The second and third protection mechanisms are related to memory access. They are called "Paging" and "Segmentation" respectively. They control which areas of memory are allowed to be accessed and/or how those areas of memory are allowed to be accessed.

=== Rings ===
Rings offer a protection layer for programs. They allow certain levels of resource access to processes. This is good, because it keeps bad programs from messing things up. There are, however, several downsides: The more CPU rings you use, the more the OS is tied to the architecture. You can, however, have several architectures each with it's own ring switching code. Another issue with this is that you OS must have a [[TSS]] set up and several other features, making ring switching much more difficult than just running all programs in kernel mode. There are a total of 4 rings in most common architectures. However, many architectures have only two rings (e.g. x86_64), corresponding to ring 0 and 3 in this description.

==== Ring 0 ====
This is kernel mode or supervisor mode. This level has the least protection, and the most access to resources. When starting up, the OS runs in this mode unless it switches out. Interrupt handlers run in this mode.

==== Rings 1 and 2 ====
These rings are mostly used for device drivers. They offer more protection, but not as much as ring 3.

==== Ring 3 ====
This is the ring that most OS's use for applications. This ring is also called Userland, or Userspace. It has the most protection and the least resource access.

Most OS's use only Ring 0 and 3. This is because rings 1 and 2 are unneeded, as device drivers can run in either ring.

Sometimes applications need access to resources that their ring won't allow. If they try to access them, a General Protection Fault (int 13) will be triggered, and the application shutdown. The application must interface with the kernel somehow, and mostly this is done with [[System Calls]].

==== I/O Privilege Level ====
Another important aspect of the ring permission system on the x86 architecture is the I/O Privilege Level (IOPL). It determines which rings have unrestricted access to I/O ports. It is a two bit number set in the EFLAGS register. If the CPL is higher than the IOPL ''or'' mode is Virtual 8086 Mode, I/O port access uses the I/O permission bitmap (IOPB) located in the TSS. Otherwise, if CPL is less or equal to IOPL, access is unrestricted.

It can be used on a process-by-process basis to grant certain servers access to some or all I/O ports.

===Paging===
See [[Paging]]

===Segmentation===
See [[Segmentation]]

==High-level protection Mechanisms==
High level security in an operating system can be accomplished in many different ways. One way would be through file permissions in a [[VFS]]. Files in *nix have a 'permission' value in their inode entry. This controls which users can read, write, execute or delete a file. These are mostly controlled by the [[File Systems]].

==Programming Language Specific Security (Assembly/C/C++)==
Many operating systems around the OSDev community trust user input completely. While it is understood that most users are fighting to get a driver working/stable, it should also be understood that production level code should not contain fundamental security problems. For example, many system call implementations take values off the stack from users and store them into registers and then freely dereference them or use them as lengths, offsets, and so on. Looking deeper into the following subtopics, readers should be able to decipher why this poses a problem and how they can be used to gain elevated privileges inside an otherwise 'secure system'.

Also note that ELF parsers and network stack implementations can pose a large threat. Many OSDev projects trust integers and offsets loaded from ELF files and freely execute and store data at these addresses. Similarly, network protocol parsers are riddled with security-related bugs (mostly integer over-and underflows) that are often used to cause real security problems due to heap-based ramifications. Take a look at the following piece of code:

<source lang="c">
int size = int_from_user();
char *buf = malloc(size + 1);
memcpy(buf, data_from_user, size);
</source>

As you can see, if the user sets size to ((unsigned) -1), the allocation will allocate 0 bytes, which on most usermode heaps will allocate space for the headers and no data, causing a full blown heap overflow controllable by the user.

===Stack-based Overflows===
Knowing the x86 [[Stack]], you should already be able to deduce why this can pose a security threat. It is (relatively) very easy to make a stack overflow, so special care has to be taken while implementing one in your OS.

===Heap-based Overflows===
Heap-based overflows are fundamentally different from stack-based ones and are specific to each allocator. You can help to detect them inside your OS by setting "poison values" for each member of the control structures on free and allocation and checking them before operations. Linux uses this approach (see LIST_POISON1 and LIST_POISON2 [http://lxr.linux.no/linux/include/linux/poison.h#L10]).

===Integer-based Overflows===
Integer Overflows do not cause a direct impact such as stack-based or heap-based overflows do. Instead they lead to these types of bugs (usually heap-based) later on in the code. For instance, let's say we have an application parsing data from a protocol or file format:

<source lang="c">
int size = read_int_from_network();
char *buffer = malloc(size + 1);            // leave room for NULL.
memcpy(buffer, data_from_network(), size);  // copies 2^32-1 bytes into buffer.
</source>

As you can see, there is no verification on the size read from the user and this can cause large problems later in the code. Examples in OSDev where this arises include ELF parsers (think segment/program/entry point offset into the file), network parsers, read/write/lseek implementations, and so on. If your code takes input from the user you *MUST* verify that it fits within the range of the data you are going to process it with. Also you should check for integer-based problems on each math operation done using user-supplied data.

===Prevention===
While obviously these cannot be directly applied to hobby OSes, the ideas can certainly be used and ported to them. The best set of documents for securing systems can be found at
[http://pax.grsecurity.net/docs/ PaX Design and Implementation]

Pax provides almost all the operating system security you need. Reading the documents pertaining to its implementations should allow Hobby OS developers to use most of these ideas in their systems. Most current Windows/BSD (kernel) protections are based on these schemes. Userland Protections can be attributed to [http://en.wikipedia.org/wiki/Address_space_layout_randomization ASLR], Heap Cookies (along with general heap hardening), Stack Canaries, and NX.

Using these mechanisms does not prevent corruption of in memory data, but instead makes using the corruption for elevated privledges/advantage very difficult.

== See Also ==
=== Articles ===
* [[GCC Stack Smashing Protector]]
* [[Resources#Software_Specifications|The Art of Software Security Assessment]]

=== Threads ===

=== External Links ===
* [http://duartes.org/gustavo/blog/post/cpu-rings-privilege-and-protection Aug 2008: CPU Rings, Privilege, and Protection] by Gustavo Duarte
* [http://doc.bughunter.net/buffer-overflow/#heap Public Documents on heap-based overflows]

[[Category:Security]]
````

<a id="w002263"></a>

## W002263 — File:Vfs diagram.png

Bron-ID: W002263
Namespace: File (6)
Revisie: 5471 | Datum: 2008-05-18T10:42:52Z
Bron: https://wiki.osdev.org/File:Vfs_diagram.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AVfs_diagram.png&oldid=5471

````text
Modified Ascii Art diagram of VFS layout in relation to the filesystem drivers, hardware and usermode applications.

Created by thePowersGang
````

<a id="w002264"></a>

## W002264 — Setting Up Paging

Bron-ID: W002264
Namespace: Main (0)
Revisie: 27621 | Datum: 2023-02-06T17:38:59Z
Bron: https://wiki.osdev.org/Setting_Up_Paging
Revisie-URL: https://wiki.osdev.org/index.php?title=Setting_Up_Paging&oldid=27621

````text
{{Rating|1}}
{{In_Progress}}
This is a guide to setting up paging. It will teach you the basic concepts behind paging and how it can help you with your OS. This example will concentrate on Legacy Non-PSE Non-PAE paging (See also [[Setting Up Paging With PAE]]). 

Paging is a term that refers to the management of the computer's ''virtual memory''. If you have not yet created a ''physical memory manager'', please read and follow [[Page Frame Allocation]] before continuing with this article.

==Basic Paging==
Paging allows you to have more than one virtual address space mapped into the physical address space. The [[MMU]] uses what is called a Page Directory to map virtual addresses to physical addresses.


'''Page Directory''' - A table in memory which the MMU uses to find the page tables.


Each index in the Page Directory is a pointer to a Page table.


'''Page Table''' - A table in memory that describes how the MMU should translate a certain range of addresses.


Each index in a Page Table contains the physical memory address to which a certain page should be mapped.

===Creating a Blank Page Directory===
The first step is to create a blank page directory. The page directory is blank because we have not yet created any page tables where the entries in the page directory can point.

Note that all of your paging structures need to be at page-aligned addresses (i.e. being a multiple of 4096). If you have already written a page frame allocator then you can use it to allocate the first free page after your kernel for the page directory. If you have not created a proper page allocator, simply finding the first free page-aligned address after the kernel will be fine, but you should write the page frame allocator as soon as possible. Another temporary solution (used in this tutorial) is to simply declare global objects with __attribute__((align(4096))). Note that this is a GCC extension. It allows you to declare data aligned with some mark, such as 4KiB here. We can use this because we are only using one page directory and one page table. Please note that on the real world, dynamic allocation is too basic to be missing, and paging structures are constantly being added, deleted, and modified. For now, just use static objects;

<source lang="c">
uint32_t page_directory[1024] __attribute__((aligned(4096)));
</source>

Now that we have a page directory, we need to blank it. The page directory should have exactly 1024 entries. We will set each entry to not present so that if the MMU looks for that page table, it will see that it is not there (...yet. We will add the first page table in a moment).

<source lang="c">
//set each entry to not present
int i;
for(i = 0; i < 1024; i++)
{
    // This sets the following flags to the pages:
    //   Supervisor: Only kernel-mode can access them
    //   Write Enabled: It can be both read from and written to
    //   Not Present: The page table is not present
    page_directory[i] = 0x00000002;
}
</source>

A page is "not present" when it's not (intended to be) used. If the MMU finds one, it will Page Fault. Non-present pages are useful for techniques such as Lazy Loading. It's also used when a page has been swapped to disk, so the Page Fault is not interpreted as an error by the OS. To the OS, it means someone needs a page it swapped to disk, so it is restored. A page fault over a page that was never swapped is a error by which the OS has a reason to kill the process.

===Creating Your First Page Table===
The second step is to create a basic page table. In this example we choose to fill up the whole first page table with addresses for the MMU. Because each page is 4 kilobytes large, and because each page table has exactly 1024 entries, filling up the whole table causes us to map 4 megabytes of memory. Also, the page directory is 1024 entries long, so everything can map up to 4GiB, the full 32-bit address space. Remembered the non-present page trick? Without it, we would use 16MiB per each paging structure. A single page directory needs 4KiB, but it can map some tables as non-present, effectively removing their space needs.

Now, its time to create a new page table.
<source lang="c">
uint32_t first_page_table[1024] __attribute__((aligned(4096)));
</source>

We now need to fill each index in the table with an address to which the MMU will map that page. Index 0 (zero) holds the address from where the first page will be mapped. Likewise, index 1 (one) holds the address for the second page and index 1023 holds the address of the 1024th page. That's for the first table. So, to get the page at which a certain index is mapped is as simple as (PageDirIndexOfTable * 1024) + PageTabIndexOfPage. If you multiply that by 4, you'll get the address (in KiB) at which the page will be loaded. For example, page index 123 in table index 456 will be mapped to (456 * 1024) + 123 = 467067. 467067 * 4 = 1868268 KiB = 1824.48046875 MiB = 1.781719207763671875 GiB. It's easy, right?

<source lang="c">
// holds the physical address where we want to start mapping these pages to.
// in this case, we want to map these pages to the very beginning of memory.
unsigned int i;

//we will fill all 1024 entries in the table, mapping 4 megabytes
for(i = 0; i < 1024; i++)
{
    // As the address is page aligned, it will always leave 12 bits zeroed.
    // Those bits are used by the attributes ;)
    first_page_table[i] = (i * 0x1000) | 3; // attributes: supervisor level, read/write, present.
}
</source>

===Put the Page Table in the Page Directory===
The third step is to put the newly created page table into our blank page directory. We do this by setting the first entry in the page directory to the address of our page table.

<source lang="c">
// attributes: supervisor level, read/write, present
page_directory[0] = ((unsigned int)first_page_table) | 3;
</source>

===Enable Paging===
The final step is to actually enable paging. First we tell the processor where to find our page directory by putting it's address into the CR3 register. Because C code cannot directly access the computer's registers, we will need to use assembly code to access CR3. The following assembly is written for GAS. If you use a different assembler then you will need to translate between this assembly format and the format supported by your assembler.

<source lang="asm">
.text
.globl loadPageDirectory
loadPageDirectory:
push %ebp
mov %esp, %ebp
mov 8(%esp), %eax
mov %eax, %cr3
mov %ebp, %esp
pop %ebp
ret
</source>

This small assembly function takes one parameter: the address of the page directory. It then loads the address onto the CR3 register, where the MMU will find it. But wait! Paging is not still enabled. That's what we will do next. We must set the 32th bit in the CR0 register, the paging bit. This operation also requires assembly code. Once done, paging will be enabled.
<source lang="asm">
.text
.globl enablePaging
enablePaging:
push %ebp
mov %esp, %ebp
mov %cr0, %eax
or $0x80000000, %eax
mov %eax, %cr0
mov %ebp, %esp
pop %ebp
ret
</source>

Now let's call the functions!
<source lang="c">
// This should go outside any function..
extern void loadPageDirectory(unsigned int*);
extern void enablePaging();
// And this inside a function
loadPageDirectory(page_directory);
enablePaging();
</source>

Paging should now be enabled. Try printing something to screen like "Hello, paging world!". If all goes well, congratulations! You've just learned the basics of paging. But there are lots of other things to do with it. You won't be able to do almost all of them for now. Just remember that you have a little friend in the CR3 register that will help you one day.

==More Advanced Paging Example==
Add sections on how to dynamically get and free pages...

[[Category:X86 CPU]]
[[Category:Tutorials]]
````

<a id="w002265"></a>

## W002265 — Talk:Setting Up Paging

Bron-ID: W002265
Namespace: Talk (1)
Revisie: 19640 | Datum: 2016-08-01T06:50:43Z
Bron: https://wiki.osdev.org/Talk:Setting_Up_Paging
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ASetting_Up_Paging&oldid=19640

````text
I added a short tutorial on basic paging. Please correct any wrong information or comment on style etc...
[[User:Stevo14|Stevo14]] 04:24, 29 April 2008 (CDT)


Added some notes about page-aligned addresses. You might also want to clarify that it's a good idea to use a page allocator for getting page tables/directories to make sure that they don't get overwritten.

--[[User:Pcmattman|pcmattman]] 18:43, 30 April 2008 (CDT)

I changed the code so that it uses a page aligned address for the page directory and page table. Maybe the more advanced paging section could go through how to set up a proper page allocator. There is also the article on [[Page Frame Allocation]] which could be referenced.
 
[[User:Stevo14|Stevo14]] 00:40, 1 May 2008 (CDT)

It might be an idea to add a section to the end about how to deliberately trigger a page fault to test whether or not paging is correctly enabled.
Maybe something simple like:

<source lang="asm">
.global _test_page_fault
.type _test_page_fault, @function
_test_page_fault:
	andb $0xFE, (_page_tables)
	movb $0, 0
	ret
</source>
--[[User:Ajxs|Ajxs]] 20:54, 29 July 2016 (CDT)

:The problem being with such code is that it makes a lot of unwritten assumptions - it assumes the first page table is stored in the binary, it doesn't work with 4M pages, and it doesn't work if the page has already been cached in the TLB. There is also a tendency that people just grab the code without reading properly and misdiagnose things as a result. I'd recommend some prose to cover the subject instead of premade code. - [[User:Combuster|Combuster]] 01:50, 1 August 2016 (CDT)
````

<a id="w002266"></a>

## W002266 — Talk:ELF

Bron-ID: W002266
Namespace: Talk (1)
Revisie: 22134 | Datum: 2018-02-06T19:03:08Z
Bron: https://wiki.osdev.org/Talk:ELF
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AELF&oldid=22134

````text
Added section on relocation, if anyone wants to proofread or fix any errors I've made, go ahead.
--[[User:Pcmattman|pcmattman]] 18:52, 30 April 2008 (CDT)

Added some theory to contribute to two items on the High Priority section of the OSDev Wish List (just a start)
There may be more to add... --[[User:Chrissacchi|Chrissacchi]] 22:58, 21 June 2016 (CDT)

I don't understand why is the section "=== Using the PIC with programs ===" in the ELF page. This is more about scheduling and has nothing to do on the particular case of the ELF executable format. I think it should be deleted or moved elsewhere, as it does not really contains usefull data / contains data that is unrelated to the subject. --[[User:Valou3433|Valou3433]] 20:04, 6 Feb 2018
````

<a id="w002267"></a>

## W002267 — User:Randomibis

Bron-ID: W002267
Namespace: User (2)
Revisie: 5412 | Datum: 2008-05-02T02:01:00Z
Bron: https://wiki.osdev.org/User:Randomibis
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ARandomibis&oldid=5412

````text
fixin yr typos
````

<a id="w002268"></a>

## W002268 — Talk:What Order Should I Make Things In?

Bron-ID: W002268
Namespace: Talk (1)
Revisie: 22842 | Datum: 2018-09-14T22:53:18Z
Bron: https://wiki.osdev.org/Talk:What_Order_Should_I_Make_Things_In%3F
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AWhat_Order_Should_I_Make_Things_In%3F&oldid=22842

````text
== NICk vs Nick ==
Re: "NICk Stacky: Typo", with NIC meaning network interface controller that was definitely not a '''typing''' error. The real question should be whether we want the keep pun in its original (obvious) state or not, or if dear Solar happened to just miss that. - [[User:Combuster|Combuster]] 17:30, 21 December 2011 (UTC)
:I admit I missed the pun. Apparently, it wasn't that obvious to begin with, as all other references to Nick Stacky in the Wiki use the default capitalization... but I'd be fine either way. -- [[User:Solar|Solar]] 20:24, 21 December 2011 (UTC)

== New Archetypes ==

I would like to propose two other archetypes you occasionally run across: Hal Virtuell (exokernel/virtualizer focus) and Parsa Lang (systems based on new languages, and/or p-machine or high-level language interpreter kernel based design). Both have real-world examples (MIT Exokernel System, VM/370 and Xen; Smalltalk-80, FIG Forth, and JavaOS), and both are addressed in existing pages of the Wiki. Are these worth mentioning? -- [[User:Schol-R-LEA|Schol-R-LEA]] 01:58, 2 May 2008 (CDT)


You have a point. There do exist archetypes that write their OS in language XYZ for no other reason than proving it is possible - Yours truly included :). At one point I even considered getting Haskell support working next to the BASIC support I already had. The only real problem is, you can subdivide these people in two categories: the ones that could potentially be making an OS in HTML (which do not belong here at all), and the people who actually know what they're up to (which I expect to consider the article a funny bit of self-psychology, rather than being truly useful). IMHO "Alto Lango" would be a better name tho...

I wouldn't really want to devote an archetype to exokernels though. Either they virtualize the whole mess and come to something that's just running one other OS rather than really being one, or they're the exokernel type and take it just as a design decision to put everything into userspace, after which you can still apply the other existing archetypes.

The conclusion is really that I think that your proposed archetypes are really just a subset of possible design decisions. If you coerce the language archetype into getting ''several'' programming languages working, it will however perfectly do as a good archetype as being an extreme variant of a development path.

- [[User:Combuster|Combuster]] 17:29, 3 May 2008 (CDT)

----

Here is a new archetype suggestion: '''Prof. Provost'''. His OS will be coded in [https://en.wikipedia.org/wiki/Spec_Sharp#Sing_Sharp Sing#] if he ever makes a prototype, or maybe he'll use the [https://en.wikipedia.org/wiki/B-Method B Method]. Over several years of laborious academic research, he formalized the semantics of the memory manager and proved in [https://en.wikipedia.org/wiki/Coq Coq] that his task scheduler is fair, leading to the publication of a dozen academic articles full of inscrutable mathematical symbols. Prof. Provost is interested in proving that the system is correct in all possible ways, but the implementation is left as an exercise for the reader.

Reaching Prof. Provost's level: The languages Prof. Provost spends the most time using are [https://en.wikipedia.org/wiki/LaTeX LaTeX] to write memoirs, articles and reports, and [https://en.wikipedia.org/wiki/Coq Coq]. Follow his trail, you will likely prusue a Ph.D in computer science (if you have the guts for it), and/or work in the research department of a large software company. You learn about ''abstract interpretation'', ''proof assistants'', ''formal methods'' and ''software verification''. 

His bookshelf contains mostly academic papers and conference proceedings.

People and OSes related to Prof. Provost:
* [http://ssrg.nicta.com.au/projects/TS/l4.verified/ L4.verified] (seL4 a.k.a Secure L4)
* [http://research.microsoft.com/en-us/projects/singularity/ Singularity] by Microsoft ([https://fr.wikipedia.org/wiki/Singularity wikipedia article]) is an operating system written in [https://en.wikipedia.org/wiki/Spec_Sharp#Sing_Sharp Sing#], a language that extends C# with contracts and invariants (in addition to low-level features).
* [http://compcert.inria.fr/ CompCert], a verified C compiler

[[User:Js|Js]] 12:12, 18 May 2016 (CDT)

== So Where's The Archetype for the Unix Impersonator? ==

You know the guy, he want's to make a unix better than unix was! After all, if Linux can pull it off so can he! <small><span class="autosigned">Preceding [[Wikipedia:Signatures|unsigned]] comment added by [[User:Primis]] ([[User talk:Primis|talk]] • [[Special:Contributions/Primis|contribs]]) 22:23, 30 September 2013</span></small>
:Oh wait. That's kinda me. --[[User:Sortie|Sortie]] 05:17, 1 October 2013 (CDT)

: Writing an unix clone is IMO not a style ''extreme'' unlike the existing ones. There's little to say about it not being "normal" except that it feels uninspired - especially for all the ones merely running down the "write your own unix clone" tutorial. - [[User:Combuster|Combuster]] 05:28, 1 October 2013 (CDT)
````

<a id="w002269"></a>

## W002269 — User:Schol-R-LEA

Bron-ID: W002269
Namespace: User (2)
Revisie: 5414 | Datum: 2008-05-02T06:59:19Z
Bron: https://wiki.osdev.org/User:Schol-R-LEA
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASchol-R-LEA&oldid=5414

````text
I have.... returned.
````

<a id="w002270"></a>

## W002270 — User:Tyra

Bron-ID: W002270
Namespace: User (2)
Revisie: 5415 | Datum: 2008-05-03T13:15:23Z
Bron: https://wiki.osdev.org/User:Tyra
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATyra&oldid=5415

````text
This (pointing to a grey box below the desk) is ''my'' computer, and I want to control every single bit of it ''myself''.  

So I decided to make my own OS. Last night, [[GRUB|Grub]] booted my first "kernel", and my kernel wrote an 'A' in adress 0xB8000. I'm so excited :-) Big thanks to all who wrote the tutorials and tools that helped me to get so far.
````

<a id="w002271"></a>

## W002271 — User talk:Tyra

Bron-ID: W002271
Namespace: User talk (3)
Revisie: 5416 | Datum: 2008-05-03T13:16:37Z
Bron: https://wiki.osdev.org/User_talk:Tyra
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ATyra&oldid=5416

````text
When you hear the beep, leave a message...


== **beep** ==
````

<a id="w002272"></a>

## W002272 — Gujin

Bron-ID: W002272
Namespace: Main (0)
Revisie: 30357 | Datum: 2026-03-18T19:43:22Z
Bron: https://wiki.osdev.org/Gujin
Revisie-URL: https://wiki.osdev.org/index.php?title=Gujin&oldid=30357

````text
{{Disputed}}
{{Warning|Gujin appears to be abandonware and hasn't been updated since 2013.}}

Gujin is a GPLed [[bootloader]] for the PC.

== What can Gujin do? ==
It can read FAT12, FAT16, FAT32, ext2, ext3, ext4 (with constant inode size) and ISO 9660 filesystems.

It has a graphical user interface with mouse support, and can be installed on any media: floppy, hard/USB disk partitions, hard/USB disk MBR, CD/DVD-ROM, DVD-RAM (i.e. FAT with 2048 bytes/sectors). It can also use a serial port as input/ouput instead of the screen and keyboard.

Gujin can chain-load other bootloader, load Linux kernel, has an unfinished loader of Multiboot specification, but more importantly for our current interest it can load standard ELF files (more exactly GZIP compressed ELF files).

There is two major ELF variant on the PC: ELF32 (with 32 bits load address, sizes and entry point) and ELF64 (with 64 bits fields), Gujin loads any of them and switch the processor in protected mode to jump to the ELF entry point.

Gujin will not try to set-up memory paging at all, that is the job of the kernel to select which kind of paging it wants - so even the 64 bits ELF files will have to handle the transition from 32 bits to 64 bits themselves (because you need paging to go to 64 bits mode).

Same for interrupts, Gujin switches to protected mode but does not re-enable interrupts, that is the job of the kernel being booted to set-up the Interrupt Decriptor Table and handle each interrupts, because BIOS will not help the kernel any more.

Gujin can also relocate the ELF file if it contains relocation information, see option [http://sourceware.org/binutils/docs-2.19/ld/Options.html#index-g_t_002d_002demit_002drelocs-79 --emit-relocs] of the "ld" linker of the "Binutils" toolchain.

Because most applications will want to collect BIOS informations before the switch to protected mode, Gujin can call in real-mode a function of the ELF file, and if this function returns zero - continue the loading process - else display an error message (for instance: "trying to execute a 64 bits application on a processor without this feature!").

Moreover, in most cases the kernel can decide to return to the Gujin bootloader (if it did not erased it from memory), without forcing a reboot.

The Gujin bootloader is built using standard Linux tools, it does not need Linux to run (only a not-too-buggy BIOS) - but we have to assume you are running Linux to use the GNU toolchain: GCC, Binutils, GZIP... to produce the ELF file for the kernel.
Also, Gujin installer needs either Linux 32 bits or Linux 64 bits to run and install the Gujin bootloader on a device.

Because Windows uses another executable format, you cannot install Cygwin/MinGW on windows and use the compilation toolchain directly, in this case you would have to generate a cross compiler toolchain to produce ELF files, and that is out of scope for this description (but not that difficult).

----

{{FirstPerson}}

So enough text description, let's try it!

The floppies being out-dated, let's say we want to use a USB stick as our test media. In some cases, we may want to use a SD card, basically the process is the same.

We first need to check that the target PC will be able to boot that USB stick or SD card, when Gujin is installed on it.

To have increased chances of success, we will ask the Gujin installer to reformat completely this USB disk or SD card, so first backup any interresting file you have in some other place.

Then, download this [http://www.mirrorservice.org/sites/download.sourceforge.net/pub/sourceforge/g/project/gu/gujin/install/2.8.2/install-2.8.2.tar.gz/install/gujin?extract=true gujin executable] if you are using a 32 bits Linux, and that [http://www.mirrorservice.org/sites/download.sourceforge.net/pub/sourceforge/g/project/gu/gujin/install/2.8.2/install-2.8.2.tar.gz/install/gujin64?extract=true gujin64 executable] if you are using a 64 bits Linux:

If you do not trust anybody, download the source file [http://sourceforge.net/projects/gujin/files/gujin/2.8.2/gujin-2.8.2.tar.gz/download gujin-*.tar.gz], extract it in a directory and type "make" - that will produce an executable named "gujin" (even on a 64 bits Linux).

Remember to visit [http://gujin.org http://gujin.org] to check if there is a newer version of Gujin, and to increase Gujin author counters and keep him happy.

Then, we reformat that dedicated USB stick as a bootable FAT filesystem, erasing all its content: first go in "root" by typing "su" or "sudo" (distribution dependant), then get the device name of you USB stick (let's say it is /dev/sdg) (sometimes /dev/mmcblk0 for SD cards), and type:
<syntaxhighlight lang="bash">
./gujin /dev/sdg --disk=BIOS:0x00,auto
</syntaxhighlight>
Depending on the size of the USB stick, that will have created either a FAT16 or a FAT32 (or even a FAT12) filesystem, but that point is not really important.

You then unplug this device, and replug it: most distribution will automatically mount the filesystem and display a window of its content: only a single file which is the bootloader itself.

To check that this filesystem is correctly created, you can type:
<syntaxhighlight lang="bash">
/sbin/fsck.vfat /dev/sdg
</syntaxhighlight>

With the Gujin installer parameters we used, that would have created a "superfloppy" format on our USB stick, that is currently the format most PC will be able to understand and boot from.

That does not mean your own PC will 100% boot it, due to BIOS bugs - so you need to test now that this USB stick is bootable by your PC or not: umount the USB stick, plug it in the test PC and power it on, see if Gujin is started (you will notice easily).

If it is not started try to check:
  - that the boot order in the BIOS is set to boot USB devices first
  - try the different USB devices if your BIOS has switchable items.
  - try to tell the Gujin installer to use the Extended BIOS instead by typing (--disk=EBIOS:0x00,auto is the default):
  ./gujin /dev/sdg
  - try to tell the Gujin installer to generate a real disk and not a superfloppy by:
  ./gujin --mbr /dev/sdg --disk=BIOS:0x00,auto
  - try the two previous options together:
  ./gujin --mbr /dev/sdg --disk=EBIOS:0x00,auto
  - try to use another (smaller) USB stick, some BIOS will only accept to boot from a FAT16 superfloppy
By now you should know a lot more about your BIOS, and have a bootable USB stick.

----

{{Warning|This section does not use a [[Why_do_I_need_a_Cross_Compiler%3F|cross-compiler]] and will likely not work or cause subtle major problems later.}}

Then, we want to generate this ELF kernel - let's try to generate a "hello world":
Create a file with that content:

<syntaxhighlight lang="c">
const char msg1[] = "Hello protected-mode text world! please reboot ..."; 
#define STACKSIZE 64 * 1024 
static unsigned stack[STACKSIZE / 4] __attribute__ ((aligned(32))); 

void _start (void) 
{ 
	/* We are flat non-paged memory and interrupt disabled */ 
	asm (" mov %0,%%esp " : : "i" (&stack[STACKSIZE / 4])); 
	volatile unsigned short *video_array = (volatile unsigned short *)0xB8000; 
	unsigned cpt1; 

	video_array += 10 * 80; /* few empty lines */ 
	/* We want blue background color and lightgray foreground, so 0x1700: */ 
	for (cpt1 = 0; cpt1 < sizeof(msg1) - 1; cpt1++) 
		video_array[cpt1] = 0x1700 + msg1[cpt1]; 

	while (1) 
		continue; 
}
</syntaxhighlight>

Then compile it like (you may need to add "-fno-stack-protector" too, distribution dependant):
<syntaxhighlight lang="bash">
gcc -m32 -Wall -O2 -s -static -nostartfiles -nodefaultlibs -Wl,-Ttext=0x110000 hello.c -o hello.elf
</syntaxhighlight> 
and compress it by:
<syntaxhighlight lang="bash">
gzip -9 hello.elf -c > hello.kgz
</syntaxhighlight>

You just need to copy that hello.kgz file into the USB stick, and reboot with this USB stick, you will get a menu with "hello.kgz" displayed.

If you click on this filename, you will have "Hello protected-mode text world! please reboot ..." displayed (Because this hello-world do not manage graphic modes, you have to "force start kernel in text mode" in Gujin configuration).

If you want a bit more complex "hello world" applications, you should download Gujin [http://sourceforge.net/projects/gujin/files/install/2.8.2/install-2.8.2.tar.gz/download install*.tar.gz] pack .

you will find there few KGZ files, like previous example but with some addresses displayed to show the exact position of the application in memory.

You will see [http://www.mirrorservice.org/sites/download.sourceforge.net/pub/sourceforge/g/project/gu/gujin/install/2.8.2/install-2.8.2.tar.gz/install/hello_32bits.kgz?extract=true hello_32bits.kgz] and [http://www.mirrorservice.org/sites/download.sourceforge.net/pub/sourceforge/g/project/gu/gujin/install/2.8.2/install-2.8.2.tar.gz/install/hello_32bits_r.kgz?extract=true hello_32bits_r.kgz] to show the effect of relocation, and [http://www.mirrorservice.org/sites/download.sourceforge.net/pub/sourceforge/g/project/gu/gujin/install/2.8.2/install-2.8.2.tar.gz/install/hello_64bits.kgz?extract=true hello_64bits.kgz] and [http://www.mirrorservice.org/sites/download.sourceforge.net/pub/sourceforge/g/project/gu/gujin/install/2.8.2/install-2.8.2.tar.gz/install/hello_64bits_r.kgz?extract=true hello_64bits_r.kgz] to show 64 bits applications.

To get the source code, just download the file [http://sourceforge.net/projects/gujin/files/gujin/2.8.2/gujin-2.8.2.tar.gz/download gujin*.tar.gz], and extract the corresponding c files.

To see the command used to compile them, just search the string "hello_32bits.kgz:" (with semicolon) in the file [http://www.mirrorservice.org/sites/download.sourceforge.net/pub/sourceforge/g/project/gu/gujin/gujin/2.8.2/gujin-2.8.2.tar.gz/gujin/Makefile?extract=true Makefile].

If you just want a real mode "hello world", you can do that too with an ELF file where the load address is null for the code segment.

There is a limit of 64 Kbytes for real mode ELF file.

Just create a file which [http://www.mirrorservice.org/sites/download.sourceforge.net/pub/sourceforge/g/project/gu/gujin/gujin/2.8.2/gujin-2.8.2.tar.gz/gujin/hello_bios.c?extract=true contains]:

<syntaxhighlight lang="c">
/* hello_bios.c - public domain - no support whatsoever */

/* Constant entry block, max 64 KB code+data+stack, do not touch: */
asm (
"	.code16gcc				\n"
"	.section .init,\"ax\",@progbits		\n"
"	.global _start				\n"
"_start:					\n"
//  "int $3 # if debugging with borland td.exe, started by tiny.exe	\n"
"	pushw	%ds				\n"
"	pushw	%es				\n"
"	pushw	%fs				\n"
"	pushw	%gs				\n"
"	pushfl					\n"
"	pushal					\n"
"	movw	%ss,%cs:1f+1			\n"
"	movw	%sp,%cs:2f+2			\n"
"	movw	%cs,%ax				\n"
"	movw	%ax,%ds				\n"
"	movw	%ax,%es				\n"
"	movw	%ax,%fs				\n"
"	movw	%ax,%gs				\n"
"	movw	%ax,%ss				\n"
"	movl	$0,%esp				\n"
"	cld					\n"
"	calll	main				\n"
"	movl	%eax,%cs:3f+2			\n"
"1:	movw	$0,%ax				\n"
"	movw	%ax,%ss				\n"
"2:	movl	$0,%esp				\n"
"	popal					\n"
"	popfl					\n"
"	popw	%gs				\n"
"	popw	%fs				\n"
"	popw	%es				\n"
"	popw	%ds				\n"
"3:	movl	$0,%eax				\n"
"	lretw					\n"
"	.previous				\n"
);

/* For information about asm() construct, see GCC manual:
[http://gcc.gnu.org/onlinedocs/gcc-4.5.0/gcc/Extended-Asm.html http://gcc.gnu.org/onlinedocs/gcc-4.5.0/gcc/Extended-Asm.html]
For information about BIOS services, see
[http://stanislavs.org/helppc/ http://stanislavs.org/helppc/]
*/
enum standard_color {
black,		blue,		green,		cyan,
red,		magenta,	brown,		lightgray,
darkgray,	lightblue,	lightgreen,	lightcyan,
lightred,	lightmagenta,	yellow,		white
};

/* This assumes 16 colors modes, text or graphic - i.e. not 256 colors: */
static inline void
VGA_writestring (const char *string, unsigned char row, unsigned char col)
{
unsigned short page_attribute = 16 * blue + lightgray, strlen_string;
const char *endstring = string;

/* We cannot rely on libc to provide strlen(): */
while (*endstring)
	endstring++;
strlen_string = endstring - string;

asm (
"	xchgl	%%ebp,%4		\n"
"	int	$0x10 # writestring	\n"
"	xchgl	%%ebp,%4		\n"
	: : "a" (0x1301), "b" (page_attribute),
	"c" (strlen_string), "d" (((unsigned short)row << 8) | col),
	"r" (string)	/* in fact %es:%bp, but "B" (string) doesn't work */
	);
}

static inline void BIOS_wait (unsigned nb_microsecond)
{
unsigned short status;

/* Use volatile when there is at least one result, but even if the result
	is not used, the code has to be inserted (status is the result): */
asm volatile ("	int	$0x15	"
	: "=a" (status)
	:  "a" (0x8600), "d" (nb_microsecond & 0xFFFF), "c" (nb_microsecond >> 16)
	);
}

int main (void)
{
VGA_writestring ("Hello BIOS world! Please wait...\r\n", 0, 0);
BIOS_wait (3 * 1000 * 1000); /* 3 seconds before returning to Gujin*/
//	return 0x80000000; /* to return to Gujin menu */
return 0xA0000000; /* to return to Gujin menu, do not display "press a key to continue" */
}
</syntaxhighlight>

And then compile it by:
<syntaxhighlight lang="bash">
gcc -m32 -Wall -Os -s -static -nostartfiles -nodefaultlibs -Wa,-R -Wl,--section-start,.init=0 hello_bios.c -o hello_bios.elf
</syntaxhighlight>
and compress it:
<syntaxhighlight lang="bash">
gzip -9 hello_bios.elf -c > hello_bios.kgz
</syntaxhighlight>
you can copy this hello_bios.kgz to the USB stick and run it, I hope the different functions of the source file are obvious.

The Gujin bootloader is willing to let the real mode and protected mode kernel to access the bootloader internal data (like what is the current video mode, how to display strings...) as long as the kernel is licensed under the GPL license - i.e. when the GZIP kernel contains a GZIP comment describing its license. That is illustrated by [http://www.mirrorservice.org/sites/download.sourceforge.net/pub/sourceforge/g/project/gu/gujin/install/2.8.2/install-2.8.2.tar.gz/install/cleandisk.kgz?extract=true cleandisk.kgz] compiled form this [http://www.mirrorservice.org/sites/download.sourceforge.net/pub/sourceforge/g/project/gu/gujin/gujin/2.8.2/gujin-2.8.2.tar.gz/gujin/cleandisk.c?extract=true cleandisk.c] source code and linked with this [http://www.mirrorservice.org/sites/download.sourceforge.net/pub/sourceforge/g/project/gu/gujin/gujin/2.8.2/gujin-2.8.2.tar.gz/gujin/cleandisk.ld?extract=true linker file].

The main point about the GPL license is that it is not possible to draw a line in between Gujin possible bug and a closed source application: it is no allowed to modify Gujin data while in real mode - but there isn't any enforcement by hardware.

Obviously, Gujin will enable you to run an ELF file with a real mode part AND a protected mode part, just have a look at the example [http://www.mirrorservice.org/sites/download.sourceforge.net/pub/sourceforge/g/project/gu/gujin/install/2.8.2/install-2.8.2.tar.gz/install/hello_gpl.kgz?extract=true hello_gpl.kgz] and its source code [http://www.mirrorservice.org/sites/download.sourceforge.net/pub/sourceforge/g/project/gu/gujin/gujin/2.8.2/gujin-2.8.2.tar.gz/gujin/hello_gpl.c?extract=true hello_gpl.c].

== See Also ==

=== Articles ===
* [[Bootloader]]

=== Threads ===

=== External Links ===

* [http://gujin.sourceforge.net Gujin] at sourceforge.net
 
[[Category:Bootloaders]]
````

<a id="w002273"></a>

## W002273 — User talk:Randomibis

Bron-ID: W002273
Namespace: User talk (3)
Revisie: 5424 | Datum: 2008-05-06T03:47:54Z
Bron: https://wiki.osdev.org/User_talk:Randomibis
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ARandomibis&oldid=5424

````text
would you mind using the "minor edit" checkbox for typefixes? Thanks. - [[User:Combuster|Combuster]] 17:09, 3 May 2008 (CDT)

okay, yep, sorry --[[User:Randomibis|Randomibis]] 22:47, 5 May 2008 (CDT)
````

<a id="w002278"></a>

## W002278 — Memory Map (x86)

Bron-ID: W002278
Namespace: Main (0)
Revisie: 30132 | Datum: 2026-03-04T23:07:33Z
Bron: https://wiki.osdev.org/Memory_Map_(x86)
Revisie-URL: https://wiki.osdev.org/index.php?title=Memory_Map_%28x86%29&oldid=30132

````text
This article describes the contents of the computer's physical memory at the moment that the BIOS jumps to your bootloader code.

== Real mode address space (< 1 MiB)==

When a typical x86 PC boots it will be in [[Real Mode]], with an active [[BIOS]]. During the time the CPU remains in Real Mode, IRQ0 (the clock)
will fire repeatedly, and the hardware that is used to boot the PC (floppy, hard disk, CD, Network card, USB) will also generate IRQs.
This means that during the PC boot process, the Real Mode IVT (see below) must be carefully preserved, because it is being used.

When the [[IVT]] is activated by an [[IRQ]], it will call a BIOS routine to handle the IRQ.
Bootloaders will also access [[BIOS#BIOS functions|BIOS functions]]. This means that the two memory workspaces that the BIOS uses (the BDA and the EBDA) must also
be carefully preserved during boot. Also, every time the BIOS handles an IRQ0 (18 times a second), several bytes in the BDA get overwritten by the BIOS
-- so do not attempt to store anything there while IRQs are active in Real Mode.

After all the BIOS functions have been called, and your kernel is loaded into memory somewhere, the bootloader or kernel may exit Real Mode
forever (often by going into 32bit [[Protected Mode]]). If the kernel never uses Real Mode again, then the first 0x500 bytes of memory in the
PC may be reused and overwritten. (However, it is very common to temporarily return to Real Mode in order to change the [[How_do_I_set_a_graphics_mode|Video Display Mode]].)

When the CPU is in Protected Mode, [[System Management Mode]] (SMM) is still invisibly active, and cannot be shut off. SMM also seems to
use the EBDA. So the EBDA memory area should <b>never</b> be overwritten.

Note: the EBDA is a variable-sized memory area (on different BIOSes). If it exists, it is always immediately below 0xA0000 in memory.
It is absolutely guaranteed to be at most 128 KiB in size. Older computers typically uses 1 KiB from 0x9FC00 - 0x9FFFF, modern firmware can be found using significantly more.
You can determine the size of the EBDA by using BIOS function [[Detecting Memory (x86)#Detecting Low Memory|INT 12h]], or by examining the word at 0x413 in the BDA (see below).
Both of those methods will tell you how much conventional memory is usable before the EBDA.

It should also be noted that your bootloader code is loaded and running in memory at physical addresses 0x7C00 through 0x7DFF. So that memory
area is likely to also be unusable until execution has been transferred to a second stage bootloader, or to your kernel.

===Overview===

{| {{wikitable}}
|-
! start
! end
! size
! description
! colspan=2 | type
|-
! colspan=6 | Real mode address space (the first MiB)
|-
| 0x00000000
| 0x000003FF
| 1 KiB
| Real Mode IVT (Interrupt Vector Table)
| rowspan=2 | unusable in real mode
| rowspan=6 | 640 KiB RAM ("Low memory")
|-
| 0x00000400
| 0x000004FF
| 256 bytes
| BDA (BIOS data area)
|-
| 0x00000500
| 0x00007BFF
| 29.75 KiB
| Conventional memory
| rowspan=3 | usable memory
|-
| 0x00007C00
| 0x00007DFF
| 512 bytes
| Your OS BootSector
|-
| 0x00007E00
| 0x0007FFFF
| 480.5 KiB
| Conventional memory
|-
| 0x00080000
| 0x0009FFFF
| 128 KiB
| EBDA (Extended BIOS Data Area)
| partially used by the EBDA
|-
| 0x000A0000
| 0x000BFFFF
| 128 KiB
| Video display memory
| hardware mapped
| rowspan=4 | 384 KiB System / Reserved ("Upper Memory")
|-
| 0x000C0000
| 0x000C7FFF
| 32 KiB (typically)
| Video BIOS
| rowspan=3 | ROM and hardware mapped / Shadow RAM
|-
| 0x000C8000
| 0x000EFFFF
| 160 KiB (typically)
| BIOS Expansions
|-
| 0x000F0000
| 0x000FFFFF
| 64 KiB
| Motherboard BIOS
|}

===BIOS Data Area (BDA)===

The BDA is only partially standardized and mostly relevant for real mode BIOS operations.
The following is a partial list. See the External Links references below for more detail.

{| {{wikitable}}
|-
! Address
! Size
! Description
|-
| 0x0400
| 8
| IO ports for COM1-COM4 serial (each address is 1 word, zero if none)
|-
| 0x0408
| 6
| IO ports for LPT1-LPT3 parallel (each address is 1 word, zero if none)
|-
| 0x040E
| 2
| EBDA base address >> 4 (<b>usually!</b>)
|-
| 0x0410
| 2
| Packed bit flags for detected hardware
|-
| 0x0413
| 2
| Number of kilobytes before EBDA / unusable memory
|- 
| 0x0417
| 2
| Keyboard state flags
|-
| 0x041E
| 32
| Keyboard buffer
|-
| 0x0449
| 1
| Display Mode
|-
| 0x044A
| 2
| Number of columns in text mode
|-
| 0x0463
| 2
| Base IO port for video (see [[VGA Hardware]])
|-
| 0x046C
| 2
| Number of IRQ0 timer ticks since boot
|-
| 0x0475
| 1
| Number of hard disk drives detected
|-
| 0x0480
| 2
| Keyboard buffer start
|-
| 0x0482
| 2
| Keyboard buffer end
|-
| 0x0497
| 1
| Last keyboard LED/Shift key state
|}

===Extended BIOS Data Area (EBDA)===

You may see "maps" of the EBDA if you search the web. However, those maps are for the original IBM BIOS EBDA. Current BIOS might use a different map. The EBDA area is not standardized.

==Extended Memory (> 1 MiB)==

The region of RAM above 1 MiB is not standardized, well-defined, or contiguous. There are likely to be regions of it that
contain memory mapped hardware, that nothing but a device driver should ever access. There are likely to be regions of it
that contain ACPI tables which your initialization code will probably want to read, and that then can be overwritten and reused.
Some ACPI areas cannot be "reclaimed" this way. Some of the computer's RAM may extend above 4 GiB.

Use the BIOS function [[Detecting Memory (x86)#BIOS Function: INT 0x15, EAX = 0xE820|INT 15h, EAX=0xE820]] to get a reliable map of Extended Memory.


{| {{wikitable}}
|-
! start
! end
! size
! region/exception
! description
|-
! colspan=5 | High Memory
|-
| 0x00100000
| 0x00EFFFFF
| 0x00E00000 (14 MiB)
| RAM -- free for use (if it exists)
| Extended memory <sup>1, 2</sup>
|-
| 0x00F00000
| 0x00FFFFFF
| 0x00100000 (1 MiB)
| Possible memory mapped hardware
| ISA Memory Hole 15-16MB <sup>3</sup>
|-
| 0x01000000
| ????????
| ???????? (whatever exists)
| RAM -- free for use
| More Extended memory <sup>1</sup>
|-
| 0xC0000000 (sometimes, depends on motherboard and devices)
| 0xFFFFFFFF
| 0x40000000 (1 GiB)
| various (typically reserved for memory mapped devices)
| Memory mapped PCI devices, PnP NVRAM?, IO APIC/s, local APIC/s, BIOS, ...
|-
| 0x0000000100000000 (possible memory above 4 GiB)
| ????????????????
| ???????????????? (whatever exists)
| RAM -- free for use (PAE/64bit)
| More Extended memory <sup>1</sup>
|-
| ????????????????
| ????????????????
| ????????????????
| Possible memory mapped hardware
| Potentially usable for memory mapped PCI devices in modern hardware (but typically not, due to backward compatibility)
|}

<sup>1</sup>: Different computers have different amounts of RAM, therefore the amount of extended memory you might find will vary and may be anything from "none" (e.g. an old 80386 system) to "lots".

<sup>2</sup>: Free for use except that your bootloader (ie. GRUB) may have loaded your "modules" here, and you don't want to overwrite those.

<sup>3</sup>: The "ISA Memory Hole" (from 0x00F00000 to 0x00FFFFFF) was used for memory mapped ISA devices (e.g. video cards). Modern computers have no need for this hole, but some chipsets still support it (as an optional feature) and some motherboards may still allow it to be enabled with BIOS options, so it may exist in a modern computers with no ISA devices.


==See Also==
* [[Detecting Memory (x86)]]

===External Links===
* https://web.archive.org/web/20120130052813/http://www.nondot.org/sabre/os/files/Booting/BIOS_SEG.txt -- detailed BIOS Data Area map
* https://web.archive.org/web/20221127214951/https://www.bioscentral.com/misc/bda.htm -- another detailed BIOS Data Area map
* [http://files.osdev.org/mirrors/geezer/osd/ram/index.htm#layout Geezer's memory layout description]
* http://stanislavs.org/helppc/bios_data_area.html

[[Category:X86]]
[[Category:Physical Memory]]
````

<a id="w002279"></a>

## W002279 — Detecting Memory (x86)

Bron-ID: W002279
Namespace: Main (0)
Revisie: 30850 | Datum: 2026-07-05T02:38:23Z
Bron: https://wiki.osdev.org/Detecting_Memory_(x86)
Revisie-URL: https://wiki.osdev.org/index.php?title=Detecting_Memory_%28x86%29&oldid=30850

````text
{{FirstPerson}}
{{Bias}}

Detecting usable physical memory is one of the earliest tasks an operating system kernel performs. On the x86 architecture, the kernel must not assume that all physical addresses correspond to usable RAM. Parts of the physical address space may be reserved for firmware, memory-mapped devices, ACPI tables, SMM, ROMs, framebuffers, PCI configuration space, APICs or defective memory.

The preferred method of detecting usable memory is by using a memory map supplied by the firmware or bootloader.  On BIOS-booted x86 systems, this usually means using <code>INT 0x15, EAX=0xE820</code>. On UEFI systems, this means using <code>BootServices->GetMemoryMap</code>.  Bootloaders such as GRUB, Limine, BOOTBOOT and Multiboot-compatible loaders can also pass a memory map to the kernel.

Note that Manual memory probing should be treated as a last resort.  It is highly unreliable on modern hardware, cannot identify all reserved regions and may accidentally access memory-mapped devices.  A correct physical memory manager should only allocate from regions explicitly reported as usable.  It will also reserve the memory occupied by the kernel image, boot modules, bootloader data, firmware tables, page tables, etc.

See [[Memory Map (x86)]] for a generic layout of the x86 physical address space.

== General considerations ==

The firmware is responsible for discovering installed RAM and configuring the memory controller before the operating system begins executing. This process is chipset-specific and is normally performed while the firmware is still running from ROM or firmware-controlled memory. Once the operating system is running, it usually cannot repeat this initialization process safely or portably.

The physical address space may also contain holes. The region from <code>0xA0000</code> to <code>0xFFFFF</code> is traditionally reserved for video memory, option ROMs, BIOS data, and other platform-specific uses. Some of this memory may still be used after boot by firmware interfaces, SMM, ACPI, or device mappings. Reclaiming portions of this range requires platform-specific knowledge and is not generally useful for a portable kernel.

On systems with several gigabytes of RAM, there is often a memory hole below 4 GiB for memory-mapped hardware. RAM that would otherwise overlap this hole may be remapped above 4 GiB by the firmware or chipset. A 32-bit kernel may need [[PAE]] to access this memory, while a 64-bit kernel can access it in [[Long Mode]] after setting up suitable paging structures.

==Detecting Low Memory==

"Low memory" is the available RAM below 1MB, and usually below 640KB. There are two [[Memory Map (x86)#BIOS functions|BIOS functions]] to get the size of it.

INT 0x12: The INT 0x12 call will return AX = total number of KB.
The AX value measures from 0, up to the bottom of the [[EBDA]] (of course, you probably shouldn't use the first 0x500 bytes of the space either --
i.e. the [[IVT]] or [[BDA]]).

Usage:
<syntaxhighlight lang="nasm">
; Clear carry flag
clc
; Switch to the BIOS (= request low memory size)
int 0x12
; The carry flag is set if it failed
jc .error

; AX = amount of continuous memory in KB starting from 0.
</syntaxhighlight>

Note: this function is supposed to be always present, and may not modify the carry flag. If a BIOS doesn't support it, the carry flag will be set, indicating error.

Alternately, you can just use INT 0x15, EAX = 0xE820 (see below).

==Detecting Upper Memory==

===BIOS Function: INT 0x15, EAX = 0xE820===

See also: [http://www.uruk.org/orig-grub/mem64mb.html]

By far the best way to detect the memory of a PC is by using the INT 0x15, EAX = 0xE820 command.
This function is available on all PCs built since 2002, and on most existing PCs before then. It is the only BIOS function that
can detect memory areas above 4G. It is meant to be the ultimate memory detection BIOS function.

In reality, this function returns an unsorted list that may contain unused entries and (in rare/dodgy cases) may return overlapping areas.
Each list entry is stored in memory at ES:DI, and DI is <b>not</b> incremented for you. The format of an entry is 2 uint64_t's and a uint32_t in the 20 byte version,
plus one additional uint32_t in the 24 byte ACPI 3.0 version.
It is probably best to always store the list entries as 24 byte quantities -- to preserve uint64_t alignments, if nothing else. (Make sure to set
that last uint32_t to 1 before each call, to make your map compatible with ACPI).

* First uint64_t = Base address
* Second uint64_t = Length of "region" (if this value is 0, ignore the entry)
* Next uint32_t = Region "type"
** Type 1: Usable (normal) RAM
** Type 2: Reserved - unusable
** Type 3: ACPI reclaimable memory
** Type 4: ACPI NVS memory
** Type 5: Area containing bad memory
* Next uint32_t = ACPI 3.0 Extended Attributes bitfield (if 24 bytes are returned, instead of 20)
** Bit 0 of the Extended Attributes indicates if the entire entry should be ignored (if the bit is clear). This is going to be a huge compatibility problem because most current OSs won't read this bit and won't ignore the entry.
** Bit 1 of the Extended Attributes indicates if the entry is non-volatile (if the bit is set) or not. The standard states that "Memory reported as non-volatile may require characterization to determine its suitability for use as conventional RAM."
** The remaining 30 bits of the Extended Attributes are currently undefined.

Basic Usage:<br>
For the first call to the function, point ES:DI at the destination buffer for the list. Clear EBX. Set EDX to the magic number 0x534D4150. Set EAX to
0xE820 (note that the upper 16-bits of EAX should be set to 0). Set ECX to 24. Do an INT 0x15.

If the first call to the function is successful, EAX will be set to 0x534D4150, and the Carry flag will be clear.
EBX will be set to some non-zero value, which must be preserved for the next call to the function. CL will contain the
number of bytes actually stored at ES:DI (probably 20).

For the subsequent calls to the function: increment DI by your list entry size, reset EAX to 0xE820, and ECX to 24.
When you reach the end of the list, EBX may reset to 0. If you call the function again with EBX = 0, the list will start
over. If EBX does not reset to 0, the function will return with Carry set when you try to access the entry after the last
valid entry.

(See the [[#Getting an E820 Memory Map|code examples]] below for a detailed ASM example, implementing the algorithm.)

Notes:

* After getting the list, it may be desirable to: sort the list, combine adjacent ranges of the same type, change any overlapping areas to the most restrictive type, and change any unrecognised "type" values to type 2.
* Type 3 "ACPI reclaimable" memory regions may be used like (and combined with) normal "available RAM" areas as long as you're finished using the ACPI tables that are stored there (i.e. it can be "reclaimed").
* Types 2, 4, 5 (reserved, ACPI non-volatile, bad) mark areas that should be avoided when you are allocating physical memory.
* Treat unlisted regions as Type 2 -- reserved.
* Your code must be able to handle areas that don't start or end on any sort of "page boundary".

Typical Output by a call to INT 15h, EAX=E820 in Bochs:

{| class="wikitable"
! Base Address
! Length
! Type
|-
| <tt>0x0000000000000000</tt> || <tt>0x000000000009FC00</tt> || Free Memory (1)
|-
| <tt>0x000000000009FC00</tt> || <tt>0x0000000000000400</tt> || Reserved Memory (2)
|-
| <tt>0x00000000000E8000</tt> || <tt>0x0000000000018000</tt> || Reserved Memory (2)
|-
| <tt>0x0000000000100000</tt> || <tt>0x0000000001F00000</tt> || Free Memory (1)
|-
| <tt>0x00000000FFFC0000</tt> || <tt>0x0000000000040000</tt> || Reserved Memory (2)
|}

===Other Methods===

====PnP====

It is possible to get a fairly good memory map using [[Plug-and-Play|Plug and Play]] (PnP) calls. {Need description and code.}

====SMBIOS====

SMBIOS is designed to allow "administrators" to assess hardware upgrade options or maintain a catalogue of what hardware a company current has in use (ie. it provides information for use by humans, rather than for use by software). It may not give reliable results on many computers -- see:
[http://www.pcpitstop.com/faq/smbios.asp].

SMBIOS will try to tell you the number of memory sticks installed and their size in MB. SMBIOS can be called from protected mode.
However, some manufacturers don't make their systems fully compliant. e.g. HP Itanium goes by the DIG64 specification so their SMBIOS doesn't return all the required device types.

Detecting memory with these functions completely ignores the concept of memory holes / memory mapped devices / reserved areas.

====BIOS Function: INT 0x15, AX = 0xE881====

This function doesn't work in real mode. Instead, it's supposed to be called from 32-bit protected mode. It returns the same information as function E801, but uses extended registers (EAX/EBX/ECX/EDX).

Should be available on everything made after 1994.

Since this is meant to be used in protected mode, the INT instruction cannot be used to call it. The correct calling procedure is unknown, but it might be the same as for protected mode EISA BIOS functions. Further information is probably buried in a Compaq technical reference manual somewhere.

====BIOS Function: INT 0x15, AX = 0xE801====

This function has been around since about 1994, so all systems from after then up to now should have this function.
It is built to handle the 15M memory hole, but stops at the next hole / memory mapped device / reserved area above that.
That is, it is only designed to handle contiguous memory above 16M.

Typical Output: <br>
AX = CX = extended memory between 1M and 16M, in K (max 3C00h = 15MB)

BX = DX = extended memory above 16M, in 64K blocks

There are some BIOSes that always return with AX = BX = 0. Use the CX/DX pair in that case. Some other BIOSes will return CX = DX = 0.
Linux initializes the CX/DX pair to 0 before the INT opcode, and then uses CX/DX, unless they are still 0 (when it will use AX/BX). In any case, it is best to do a sanity check
on the values in the registers that you use before you trust the results. (GRUB just trusts AX/BX -- this is not good.)

Usage:
<syntaxhighlight lang="nasm">
	xor cx, cx
	xor dx, dx
	mov ax, 0xe801
	int 0X15			; request upper memory size
	jc short .err
	cmp ah, 0x86		; unsupported function
	je short .err
	cmp ah, 0x80		; invalid command
	je short .err
	jcxz .useax			; was the CX result invalid?

	mov ax, cx
	mov bx, dx
    ; ...
.useax:
	; AX = number of contiguous Kb, 1M to 16M
	; BX = contiguous 64Kb pages above 16M
</syntaxhighlight>

====BIOS Function: INT 0x15, AX = 0xDA88====

This function returns the number of contiguous KiB of usable RAM starting at 0x00100000 in CL:BX in KiB. This is very similar to "INT 0x15, AX=0x8A" - if this function says there's 14 MiB of RAM at 0x00100000 then you can't assume there isn't more RAM at 0x01000000, so you'd probe for any extra memory starting at 0x01000000. 

If this function isn't supported it'll return "carry = set".

====BIOS Function: INT 0x15, AH = 0x88====

Note: This function may limit itself to reporting 15M (for legacy reasons) even if BIOS detects more
memory than that. It may also report up to 64M. It only reports contiguous (usable) RAM. In some BIOSes, it may not clear the CF flag on success.

Usage:
<syntaxhighlight lang="nasm">
	clc				; CF bug workaround
	mov ah, 0x88
	int 0x15		; request upper memory size
	jc short .err
	test ax, ax		; size = 0 is an error
	je short .err
	; AX = number of contiguous KB above 1M
    ; ...
.err:
	; ...
</syntaxhighlight>

====BIOS Function: INT 0x15, AH = 0x8A====

This function returns the extended memory size in DX:AX in KiB, or to be more specific, it returns the number of contiguous KiB of usable RAM starting at 0x00100000. This is also where it starts getting tricky...

If the ISA memory hole is present (which is a 1 MiB hole from 0x00F00000 to 0x00FFFFFF used by ISA devices for memory mapped I/O - e.g. an ISA video card's linear frame buffer) then this function might not report all usable RAM. For example, it might report RAM from 0x00100000 to 0x00F00000 and wouldn't be able to report any RAM above 0x01000000 (if present).

Basically, if this function says there's 14 MiB of RAM at 0x00100000 then you can't assume there isn't more RAM at 0x01000000. In this case, it's likely that none of the other methods will be able to tell you more, so you'd probe for any extra memory starting at 0x01000000.

If this function isn't supported it'll return "carry = set".

At least one Award BIOS has a bug where it pushes BX without a corresponding pop, resulting in a return to IP:BX with the flags still on the stack. The aforementioned BIOS also supports E820, so the simplest workaround is to avoid this function when E820 works.

====BIOS Function: INT 0x15, AH = 0xC7====

Although not widely supported, this function defined by IBM provides a nice memory map (although not as nice as 0xE820). DS:SI points to the following memory map table:

{| class="wikitable"
! Size
! Offset
! Description
|-
| 2 ||<tt>0x0</tt> || Number of significant bytes of returned data (excluding this <code>uint16_t</code>)
|- 
| 4 ||<tt>0x2</tt> || Amount of local memory between 1-16MB, in 1KB blocks
|-
| 4 ||<tt>0x6</tt> ||  Amount of local memory between 16MB and 4GB, in 1KB blocks
|-
| 4 ||<tt>0xA</tt> ||  Amount of system memory between 1-16MB, in 1KB blocks
|-
| 4 ||<tt>0xE</tt> ||  Amount of system memory between 16MB and 4GB, in 1KB blocks
|-
| 4 ||<tt>0x12</tt> ||  Amount of cacheable memory between 1-16MB, in 1KB blocks
|-
| 4 ||<tt>0x16</tt> ||  Amount of cacheable memory between 16MB and 4GB, in 1KB blocks
|-
| 4 ||<tt>0x1A</tt> ||  Number of 1KB blocks before start of nonsystem memory between 1-16MB
|-
| 4 ||<tt>0x1E</tt> ||  Number of 1KB blocks before start of nonsystem memory between 16MB and 4GB
|-
| 2 ||<tt>0x22</tt> ||  Starting segment of largest block of free memory in <tt>0xC000</tt> and <tt>0xD000</tt> segments
|-
| 2 ||<tt>0x24</tt> ||  Amount of free memory in the block defined by offset <tt>22h</tt>
|-
|}

The minimum number which can be returned by the first <code>uint16_t</code> is 66 bytes. Here's how the memory types are defined:

* Local Memory on the system board or memory that is not accessible from the channel. It can be system or nonsystem memory.
* Channel Memory on adapters. It can be system or nonsystem memory.
* System Memory that is managed and allocated by the primary operating system. This memory is cached if the cache is enabled.
* Nonsystem Memory that is not managed or allocated by the primary operating system. This memory includes memory-mapped I/O devices; memory that is on an adapter and can be directly modified by the adapter; and memory that can be relocated within its address space, such as bank-switched and expanded-memory-specifications (EMS) memory. This memory is not cached.

====CMOS====

The CMOS memory size information may ignore the standard memory hole at 15M. If you use the CMOS size, you may want to simply
assume that this memory hole exists. Of course, it also has no information about any other reserved regions.

Usage:
<syntaxhighlight lang="c">
unsigned short get_cmos_memory_size(void)
{
    unsigned short total;
    unsigned char lowmem, highmem;

    outportb(0x70, 0x30);
    lowmem = inportb(0x71);
    outportb(0x70, 0x31);
    highmem = inportb(0x71);

    total = lowmem | highmem << 8;
    return total;
}
</syntaxhighlight>

====E820h====

There are a few other BIOS functions that claim to give you memory information. However, they are so widely unsupported that it is impossible to even find machines to test the code on. <b>All</b> current machines support E820 (above). If some user should happen to dig up such a dinosaur of a machine that its BIOS does not support any standard memory detection function -- they will not complain that your modern OS fails to support that machine. Just give an error message.

==== Manual Probing ====
{{Warning|This could possibly damage your computer.}}
Use BIOS to get a memory map, or use [[GRUB]] (which calls BIOS for you). Memory probing can have results that are '''unpredictable by nature''' because memory probing is unsupported by vendors.

===== Theoretical introduction =====
Direct memory probing is only useful for very old systems with buggy and/or non-updateable BIOSes, or maybe a system with modified hardware that does not match the firmware anymore. So if you don't intend to support this sort of computers, you don't need memory probing. Period. Even if you need, you may decide that it's safer to run will less memory than is available.

Anyway, don't think that this will save you from the effort of understanding how to call the complicated BIOS APIs. Before launching a probe, you will always need to detect '''if''' the computer where your OS is running really needs it, and '''which''' memory areas really need to be probed (because for other memory areas you should always consider the information provided by the BIOS as authoritative). You also need to take into account the appropriate memory holes and/or memory mapped devices, which may vary from system to system.

When perfectly implemented, directly probing memory may allow you to detect extended memory on systems where the BIOS fails to provide the appropriate support. However, the algorithm will always need to take into account potential holes in system memory or previously detected memory mapped devices, such as frame buffering SVGA cards, etc. Maybe you will want to probe just a specific range of memory that is known to be otherwise undetectable on a specific computer model.

However, the BIOS is part of the computer, and may know things (see [[#Practical obstacles to memory probing]]) you ignore about your memory, motherboard and PCI devices. Probing memory-mapped PCI devices may have '''unpredictable results'''. The results are unpredictable by nature because memory probing is unsupported by vendors. The most likely result is crashing you computer, but you can even '''damage your system permanently''', such as clearing firmware chips or setting unsafe device operation parameters. Just remember the [https://en.wikipedia.org/wiki/CIH_(computer_virus) Chernobyl] computer virus.

Note: You will never get an error from trying to read/write memory that does not exist -- this is important to understand: you will not get valid results, but you won't get an error, either.

===== Practical obstacles to memory probing =====
There follows a list of technical difficulties involved in memory probing that may help to implement such an algorithm if you end up having to do it:
* There can be important data structures left in RAM by the BIOS (e.g. the ACPI tables) that you'd trash. Those structures may be anywhere and the only way to know their addresses is to "ask" the BIOS.
* There can be a memory mapped device from 15 MB to 16 MB (typically "VESA local bus" video cards, or older ISA cards that aren't limited to just video).
* There can also be a(n) (extremely rare) "memory hole" at 0x00080000 for some sort of compatibility with ancient cards.
* On modern systems there can also be faulty RAM that INT 0x15, eax=0xE820 says not to use, that can be anywhere (except in the first 1 MB, probably).
* There can also be large arbitrary memory holes. (e.g. a NUMA system with RAM up to 0x1FFFFFFF, a hole from 0x20000000 to 0x3FFFFFFF, then more RAM from 0x40000000 to 0x5FFFFFFF.)
* It's possible for physical addresses to be truncated in various ways. For example, older chipsets often have less address lines than supported by the processor and typically on motherboard using such chipsets, the extra address lines are simply not connected. For example, Intel desktop chipsets prior to the 955X only have 32 address lines, even though they are usually used with a processor that supports at least PAE. The extra address lines (A32-A35) are simply not connected on the motherboard, and if the processor attempts to access memory using physical addresses exceeding 32-bit, the physical address is truncated by virtue of the extra address lines not being connected on the motherboard.
* There are memory mapped devices (PCI video cards, HPET, PCI express configuration space, APICs, etc) with addresses that must be avoided.
* There are also (typically older) motherboards where you can write a value to "nothing" and read the same value back due to bus capacitance; there are motherboards where you write a value to cache and read the same value back from cache even though no RAM is at that address.
* There are (older, mostly 80386) motherboards that remap the RAM underneath the option ROMs and BIOS to the end of RAM. (e.g. with 4 MB of RAM installed you get RAM from 0x00000000 to 0x000A0000 and more RAM from 0x00100000 to 0x00460000, which causes problems if you test each MB of RAM because you get the wrong answer -- either under-counting RAM up to 0x00400000, or over-counting RAM up to 0x00500000).
* If you write the code properly (ie. to avoid as many of the problems as you can), then it is <i>insanely</i> slow.
* Lastly, testing for RAM (if it actually works) will only tell you where RAM is - it doesn't give you a complete map of the physical address space. You won't know where you can safely put memory mapped PCI devices because you won't know which areas are reserved for chipset things (e.g. SMM, ROMs), etc.

In contrast to this, using the BIOS functions isn't too hard, is much more reliable, gives complete information, and is extremely fast in comparison.

==Memory Map Via GRUB==

[[GRUB]], or any bootloader implementing [http://www.gnu.org/software/grub/manual/multiboot/multiboot.html The Multiboot Specification] provides a convenient way of detecting the amount of RAM your machine has.  Rather than re-invent the wheel, you can ride on the hard work that others have done by utilizing the multiboot_info structure.  When GRUB runs, it loads this structure into memory and leaves the address of this structure in the EBX register.  You may also view this structure at the GRUB command-line with the GRUB command <tt>displaymem</tt> and GRUB 2 command <tt>lsmmap</tt>.

The methods it uses are:
* Try BIOS Int 0x15, eax = 0xE820
* If that didn't work, try BIOS Int 0x15, ax = 0xE801 and BIOS Int 0x12
* If that didn't work, try BIOS Int 0x15, ah = 0x88 and BIOS Int 0x12

GRUB will also be able to give you a memory map if you are booting it off of UEFI, through the same interface.

However, it does not take into account any bugs that are known to effect some BIOSs (see entries in [[RBIL]]). It does not check "E801" and/or "88" returning with carry set.

To utilize the information that GRUB passes to you, first include the file [https://www.gnu.org/software/grub/manual/multiboot/html_node/multiboot_002eh.html multiboot.h] (or [https://cgit.git.savannah.gnu.org/cgit/grub.git/tree/doc/multiboot2.h?h=multiboot2 multiboot2.h] if you are using v2) in your kernel's main file.  Then, make sure that when you load your _main function from your assembly loader, you push EAX and EBX onto the stack. Make sure to do this before calling any other initialization functions, such as [[Calling_Global_Constructors|global constructor initialization]], or the initialization function(s) will probably clobber the registers. Then, define your start function as such:

 _main (multiboot_info_t* mbd, unsigned int magic) {...}

The key for memory detection lies in the multiboot_info struct. To determine the contiguous memory size, you may simply check <tt>mbd->flags</tt> to verify bit 0 is set, and then you can safely refer to <tt>mbd->mem_lower</tt> for conventional memory (e.g. physical addresses ranging between 0 and 640KB) and <tt>mbd->mem_upper</tt> for high memory (e.g. from 1MB). Both are given in kibibytes, i.e. blocks of 1024 bytes each.

To get a complete memory map, check bit 6 of <tt>mbd->flags</tt> and use <tt>mbd->mmap_addr</tt> to access the BIOS-provided memory map. Quoting [http://www.gnu.org/software/grub/manual/multiboot/html_node/Boot-information-format.html#Boot%20information%20format specifications],

{{Quotation
| If bit 6 in the flags uint16_t is set, then the mmap_* fields are valid, and indicate the address and length of a buffer containing a memory map of the machine provided by the BIOS. mmap_addr is the address, and mmap_length is the total size of the buffer. The buffer consists of one or more of the following size/structure pairs (size is really used for skipping to the next pair):''
}}


Taking this into account, our example code would look like the following. Note that if you prefer a version that does not require the multiboot.h header downloaded from the link above, there is a version listed in the code examples section of this article.

<syntaxhighlight lang="c">


#include "multiboot.h"
void _main(multiboot_info_t* mbd, uint32_t magic)
{
    /* Make sure the magic number matches for memory mapping*/
    if(magic != MULTIBOOT_BOOTLOADER_MAGIC) {
        panic("invalid magic number!");
    }

    /* Check bit 6 to see if we have a valid memory map */
    if(!(mbd->flags >> 6 & 0x1)) {
        panic("invalid memory map given by GRUB bootloader");
    }

    /* Loop through the memory map and display the values */
    int i;
    for(i = 0; i < mbd->mmap_length; 
        i += sizeof(multiboot_memory_map_t)) 
    {
        multiboot_memory_map_t* mmmt = 
            (multiboot_memory_map_t*) (mbd->mmap_addr + i);

        printf("Start Addr: %x | Length: %x | Size: %x | Type: %d\n",
            mmmt->addr, mmmt->len, mmmt->size, mmmt->type);

        if(mmmt->type == MULTIBOOT_MEMORY_AVAILABLE) {
            /* 
             * Do something with this memory block!
             * BE WARNED that some of memory shown as availiable is actually 
             * actively being used by the kernel! You'll need to take that
             * into account before writing to memory!
             */
        }
    }
}
</syntaxhighlight>


If your kernel is using Multiboot version 2, the structures work a bit differently. You have to find the mmap tag in the Multiboot info structure, which in turn will give you a memory map table:<syntaxhighlight lang="c">
#include "multiboot2.h"

/* Multiboot 2, unlike the original specification, uses tags
*  to organize its contents. Each tag has an associated type,
*  represented by a 32-bit integer. This function finds a tag
*  given its type. */
struct multiboot_tag *multiboot_find_tag(void *mbd, uint32_t type) {
    /* The multiboot info structure begins with a 32-bit integer
    *  indicating the total size of the structure. This is then
    *  followed by a 32-bit reserved region and then by the tags. */
	struct multiboot_tag *tag = (void *)mbd + 8;
	while ((void *)tag < (void *)mbd + *((uint32_t *)mbd)) {
		if (tag->type == type) {
			return tag;
		}
		tag = (void *)tag + tag->size;
		/* Tags are always aligned on 8-byte boundaries. */
		if ((uintptr_t)tag % 8 > 0) {
			tag = (void *)tag + 8 - ((uintptr_t)tag % 8);
		}
	}
	return 0;
}


void _main(void *mbd, uint32_t magic)
{
    if (magic != MULTIBOOT_BOOTLOADER_MAGIC) {
        panic("invalid magic number!");
    }
    
    struct multiboot_tag_mmap *tag_mmap =
	    (struct multiboot_tag_mmap *)multiboot_find_tag(
		multiboot, MULTIBOOT_TAG_TYPE_MMAP);

	if (!tag_mmap) {
		panic("No memory map tag found!\n");
	}

	struct multiboot_mmap_entry *entry =
	    (struct multiboot_mmap_entry *)(uintptr_t)tag_mmap->entries;
	while ((void *)entry < (void *)tag_mmap + tag_mmap->size) {
		kprintf("Start Addr: %x | Length: %x | Type: %i.\n",
		    entry->addr, entry->len, entry->type);
		entry = (void *)entry + tag_mmap->entry_size;
	}

}
</syntaxhighlight>'''WARNING:''' If you downloaded the multiboot header from gnu.org (linked above) you probably got a version which defines the base address and length fields as one 64-bit unsigned integer each, rather than two 32-bit unsigned integers each. [https://forum.osdev.org/viewtopic.php?t=30318 This may cause gcc to pack the structure incorrectly] which can lead to nonsensical values when you try to read it.
* The linked forum post blames GCC for not properly packing the multiboot struct, however, the real error was the printf implementation/usage. When using the type uint64_t, you must specify %lx (instead of %x) so that printf will read all 64-bits as one argument rather than reading the high-32 as one argument and the low-32 as the next argument.

Alternatively, you can modify the multiboot header, specifically the multiboot_mmap_entry struct, to the following to get the correct values:

<syntaxhighlight lang="c">
struct multiboot_mmap_entry
{
  multiboot_uint32_t size;
  multiboot_uint32_t addr_low;
  multiboot_uint32_t addr_high;
  multiboot_uint32_t len_low;
  multiboot_uint32_t len_high;
#define MULTIBOOT_MEMORY_AVAILABLE              1
#define MULTIBOOT_MEMORY_RESERVED               2
#define MULTIBOOT_MEMORY_ACPI_RECLAIMABLE       3
#define MULTIBOOT_MEMORY_NVS                    4
#define MULTIBOOT_MEMORY_BADRAM                 5
  multiboot_uint32_t type;
} __attribute__((packed));
typedef struct multiboot_mmap_entry multiboot_memory_map_t;
</syntaxhighlight>

Each multiboot mmap entry is stored as the following:

{| class="wikitable"
! Offset
! Field
|-
| 0
| size
|-
| 4
| base_addr_low
|-
| 8
| base_addr_high
|-
| 12
| length_low
|-
| 16
| length_high
|-
| 20
| type
|-
|}


* "size" is the size of the associated structure in bytes, which can be greater than the minimum of 20 bytes. base_addr_low is the lower 32 bits of the starting address, and base_addr_high is the upper 32 bits, for a total of a 64-bit starting address. length_low is the lower 32 bits of the size of the memory region in bytes, and length_high is the upper 32 bits, for a total of a 64-bit length. type is the variety of address range represented, where a value of 1 indicates available RAM, and all other values currently indicated a reserved area.
* GRUB simply uses INT 15h, EAX=E820 to get the detailed memory map, and does not verify the "sanity" of the map. It also will not sort the entries, retrieve any available ACPI 3.0 extended uint32_t (with the "ignore this entry" bit), or clean up the table in any other way.
* One of the problems you have to deal with is that GRUB can theoretically place its Multiboot information, and all the tables it references (elf sections, mmap and modules) anywhere in memory, according to the Multiboot specification. In reality, in current GRUB legacy, they are allocated as parts of the GRUB program itself, below 1MB, but that is not guaranteed to remain the same. For that reason, you should try to protect these tables before you start using a certain bit of memory. (You might scan the tables to make sure their addresses are all below 1M.)
* Another problem is that the "type" field is defined as "1 = usable RAM" and "anything else is unusable". Despite what the multi-boot specification says, lots of people assume that the type field is taken directly from INT 15h, EAX=E820 (and in older versions of GRUB it is). However GRUB 2 supports booting from UEFI/EFI (and other sources) and code that assumes the type field is taken directly from INT 15h, EAX=E820 will become broken. This means that (until a new multi-boot specification is released) you shouldn't make assumptions about the type, and can't do things like reclaiming the "ACPI reclaimable" areas or supporting S4/hibernate states (as an OS needs to save/restore areas marked as "ACPI NVS" to do that). Fortunately a new version of the multi-boot specification should be released soon which hopefully fixes this problem (but unfortunately, you won't be able to tell if your OS was started from "GRUB-legacy" or "GRUB 2", unless it adopts the new multi-boot header and becomes incompatible with GRUB-legacy).

==Memory Detection in Emulators==

When you tell an emulator how much memory you want emulated, the concept is a little "fuzzy" because of the emulated missing bits of RAM
below 1M. If you tell an emulator to emulate 32M, does that mean your address space definitely goes from 0 to 32M -1, with missing bits?
Not necessarily. The emulator might assume that you mean 32M of <i>contiguous</i> memory above 1M, so it might end at 33M -1.
Or it might assume that you mean 32M of total <i>usable</i> RAM, going from 0 to 32M + 384K -1. So don't be surprised if you see a "detected memory size"
that does not exactly match your expectations.

== What about on UEFI? ==
On UEFI, you have 'BootServices->GetMemoryMap'. This function is similar to E820 and is the only solution on new UEFI machines. Basically, to use, first you call it once to get the size of the memory map. Then you allocate a buffer of that size, and then call again to get the map itself. Watch out, by allocating memory you could increase the size of the memory map. Considering that a new allocation can split a free memory area into two, you should add space for 2 additional memory descriptors. It returns an array of EFI_MEMORY_DESCRIPTORs. They have the following format (taken from GNU EFI):
<syntaxhighlight lang="c">
typedef struct {
    UINT32                          Type;           // EFI_MEMORY_TYPE, Field size is 32 bits followed by 32 bit pad
    UINT32                          Pad;
    EFI_PHYSICAL_ADDRESS            PhysicalStart;  // Field size is 64 bits
    EFI_VIRTUAL_ADDRESS             VirtualStart;   // Field size is 64 bits
    UINT64                          NumberOfPages;  // Field size is 64 bits
    UINT64                          Attribute;      // Field size is 64 bits
} EFI_MEMORY_DESCRIPTOR;
</syntaxhighlight>
To traverse them, you can use the NEXT_MEMORY_DESCRIPTOR macro.

Memory types are different to the E820 codes. For converting, see [https://github.com/tianocore/edk2/blob/70d5086c3274b1a5b099d642d546581070374e6e/OvmfPkg/Csm/LegacyBiosDxe/LegacyBootSupport.c#L1601 CSM E820 compatibility].
<syntaxhighlight lang="c">
typedef enum {
  EfiReservedMemoryType,
  EfiLoaderCode,
  EfiLoaderData,
  EfiBootServicesCode,
  EfiBootServicesData,
  EfiRuntimeServicesCode,
  EfiRuntimeServicesData,
  EfiConventionalMemory,
  EfiUnusableMemory,
  EfiACPIReclaimMemory,
  EfiACPIMemoryNVS,
  EfiMemoryMappedIO,
  EfiMemoryMappedIOPortSpace,
  EfiPalCode,
  EfiPersistentMemory,
  EfiMaxMemoryType
} EFI_MEMORY_TYPE;
</syntaxhighlight>

==Code Examples==

===Getting a GRUB Memory Map===

Declare the appropriate structure, get the pointer to the first instance, grab whatever address and length information you want, and finally skip to the next memory map instance by adding size+sizeof(mmap->size) to the pointer, because mmap->size does not take itself into account and because GRUB treating base_addr_low as offset 0 in the structure.  You must also use mmap_length to make sure you don't overshoot the entire buffer.

<syntaxhighlight lang="c">
typedef struct multiboot_memory_map {
	unsigned int size;
	unsigned int base_addr_low,base_addr_high;
// You can also use: unsigned long long int base_addr; if supported.
	unsigned int length_low,length_high;
// You can also use: unsigned long long int length; if supported.
	unsigned int type;
} multiboot_memory_map_t;

// this is really an entry, not the entire map.
typedef multiboot_memory_map_t mmap_entry_t;

int main(multiboot_info* mbt, unsigned int magic) {
	...
	mmap_entry_t* entry = mbt->mmap_addr;
	while(entry < mbt->mmap_addr + mbt->mmap_length) {
		// do something with the entry
		entry = (mmap_entry_t*) ((unsigned int) entry + entry->size + sizeof(entry->size));
	}
	...
}
</syntaxhighlight>

===Getting an E820 Memory Map===

<syntaxhighlight lang="nasm">
; use the INT 0x15, eax= 0xE820 BIOS function to get a memory map
; note: initially di is 0, be sure to set it to a value so that the BIOS code will not be overwritten. 
;       The consequence of overwriting the BIOS code will lead to problems like getting stuck in `int 0x15`
; inputs: es:di -> destination buffer for 24 byte entries
; outputs: bp = entry count, trashes all registers except esi
mmap_ent equ 0x8000             ; the number of entries will be stored at 0x8000
do_e820:
        mov di, 0x8004          ; Set di to 0x8004. Otherwise this code will get stuck in `int 0x15` after some entries are fetched 
	xor ebx, ebx		; ebx must be 0 to start
	xor bp, bp		; keep an entry count in bp
	mov edx, 0x0534D4150	; Place "SMAP" into edx
	mov eax, 0xe820
	mov [es:di + 20], dword 1	; force a valid ACPI 3.X entry
	mov ecx, 24		; ask for 24 bytes
	int 0x15
	jc short .failed	; carry set on first call means "unsupported function"
	mov edx, 0x0534D4150	; Some BIOSes apparently trash this register?
	cmp eax, edx		; on success, eax must have been reset to "SMAP"
	jne short .failed
	test ebx, ebx		; ebx = 0 implies list is only 1 entry long (worthless)
	je short .failed
	jmp short .jmpin
.e820lp:
	mov eax, 0xe820		; eax, ecx get trashed on every int 0x15 call
	mov [es:di + 20], dword 1	; force a valid ACPI 3.X entry
	mov ecx, 24		; ask for 24 bytes again
	int 0x15
	jc short .e820f		; carry set means "end of list already reached"
	mov edx, 0x0534D4150	; repair potentially trashed register
.jmpin:
	jcxz .skipent		; skip any 0 length entries
	cmp cl, 20		; got a 24 byte ACPI 3.X response?
	jbe short .notext
	test byte [es:di + 20], 1	; if so: is the "ignore this data" bit clear?
	je short .skipent
.notext:
	mov ecx, [es:di + 8]	; get lower uint32_t of memory region length
	or ecx, [es:di + 12]	; "or" it with upper uint32_t to test for zero
	jz .skipent		; if length uint64_t is 0, skip entry
	inc bp			; got a good entry: ++count, move to next storage spot
	add di, 24
.skipent:
	test ebx, ebx		; if ebx resets to 0, list is complete
	jne short .e820lp
.e820f:
	mov [es:mmap_ent], bp	; store the entry count
	clc			; there is "jc" on end of list to this point, so the carry must be cleared
	ret
.failed:
	stc			; "function unsupported" error exit
	ret
</syntaxhighlight>

Sample in C (assuming we are in a bootloader environment, real mode, DS and CS = 0000):
<syntaxhighlight lang="c">
// running in real mode may require:
__asm__(".code16gcc\n");

// SMAP entry structure
#include <stdint.h>
typedef struct SMAP_entry {

	uint32_t BaseL; // base address uint64_t
	uint32_t BaseH;
	uint32_t LengthL; // length uint64_t
	uint32_t LengthH;
	uint32_t Type; // entry Type
	uint32_t ACPI; // extended

}__attribute__((packed)) SMAP_entry_t;

// load memory map to buffer - note: regparm(3) avoids stack issues with gcc in real mode
int __attribute__((noinline)) __attribute__((regparm(3))) detectMemory(SMAP_entry_t* buffer, int maxentries)
{
	uint32_t contID = 0;
	int entries = 0, signature, bytes;
	do 
	{
		__asm__ __volatile__ ("int  $0x15" 
				: "=a"(signature), "=c"(bytes), "=b"(contID)
				: "a"(0xE820), "b"(contID), "c"(24), "d"(0x534D4150), "D"(buffer));
		if (signature != 0x534D4150) 
			return -1; // error
		if (bytes > 20 && (buffer->ACPI & 0x0001) == 0)
		{
			// ignore this entry
		}
		else {
			buffer++;
			entries++;
		}
	} 
	while (contID != 0 && entries < maxentries);
	return entries;
}

// in your main routine - memory map is stored in 0000:1000 - 0000:2FFF for example
[...] {
	[...]
	SMAP_entry_t* smap = (SMAP_entry_t*) 0x1000;
	const int smap_size = 0x2000;

	int entry_count = detectMemory(smap, smap_size / sizeof(SMAP_entry_t));

	if (entry_count == -1) {
		// error - halt system and/or show error message
		[...]
	} else {
		// process memory map
		[...]
	}
}
</syntaxhighlight>

===Getting a UEFI Memory Map===
<syntaxhighlight lang="c">
EFI_STATUS                  Status;
EFI_MEMORY_DESCRIPTOR       *EfiMemoryMap;
UINTN                       EfiMemoryMapSize;
UINTN                       EfiMapKey;
UINTN                       EfiDescriptorSize;
UINT32                      EfiDescriptorVersion;

//
// Get the EFI memory map.
//
EfiMemoryMapSize  = 0;
EfiMemoryMap      = NULL;
Status = gBS->GetMemoryMap (
                &EfiMemoryMapSize,
                EfiMemoryMap,
                &EfiMapKey,
                &EfiDescriptorSize,
                &EfiDescriptorVersion
                );
ASSERT (Status == EFI_BUFFER_TOO_SMALL);

//
// Use size returned for the AllocatePool. We allocate 2 additional
// descriptors because AllocatePool might add new descriptors.
//
EfiMemoryMapSize += 2 * EfiDescriptorSize;
EfiMemoryMap = (EFI_MEMORY_DESCRIPTOR *) AllocatePool (EfiMemoryMapSize);
ASSERT (EfiMemoryMap != NULL);
Status = gBS->GetMemoryMap (
                &EfiMemoryMapSize,
                EfiMemoryMap,
                &EfiMapKey,
                &EfiDescriptorSize,
                &EfiDescriptorVersion
                );
if (EFI_ERROR (Status)) {
  FreePool (EfiMemoryMap);
}

//
// Get descriptors
//
EFI_MEMORY_DESCRIPTOR       *EfiEntry = EfiMemoryMap;
do {
  // ... do something with EfiEntry ...
  EfiEntry    = NEXT_MEMORY_DESCRIPTOR (EfiEntry, EfiDescriptorSize);
} while((UINT8*)EfiEntry < (UINT8*)EfiMemoryMap + EfiMemoryMapSize); 
</syntaxhighlight>

===Manual Probing in C===

Notes:
* the interrupt disable and the cache invalidation keep memory consistent.
* the assembly language manual probing code that follows this example is better

<syntaxhighlight lang="c">
 /*
  * void count_memory (void)
  *
  * probes memory above 1mb
  *
  * last mod : 05sep98 - stuart george
  *            08dec98 - ""     ""
  *            21feb99 - removed dummy calls
  *
  */
void count_memory(void)
{
	register ULONG *mem;
	ULONG	mem_count, a;
	USHORT	memkb;
	UCHAR	irq1, irq2;
	ULONG	cr0;

	/* save IRQ's */
	irq1=inb(0x21);
	irq2=inb(0xA1);

	/* kill all irq's */
	outb(0x21, 0xFF);
	outb(0xA1, 0xFF);

	mem_count=0;
	memkb=0;

	// store a copy of CR0
	__asm__ __volatile("movl %%cr0, %%eax":"=a"(cr0))::"eax");

	// invalidate the cache
	// write-back and invalidate the cache
	__asm__ __volatile__ ("wbinvd");

	// plug cr0 with just PE/CD/NW
	// cache disable(486+), no-writeback(486+), 32bit mode(386+)
	__asm__ __volatile__("movl %%eax, %%cr0", ::
		"a" (cr0 | 0x00000001 | 0x40000000 | 0x20000000) : "eax");

	do {
		memkb++;
		mem_count += 1024*1024;
		mem= (ULONG*) mem_count;

		a= *mem;
		*mem= 0x55AA55AA;

          // the empty asm calls tell gcc not to rely on what's in its registers
          // as saved variables (this avoids GCC optimisations)
		asm("":::"memory");
		if (*mem!=0x55AA55AA) mem_count=0;
		else {
			*mem=0xAA55AA55;
			asm("":::"memory");
			if(*mem!=0xAA55AA55)
			mem_count=0;
		}

		asm("":::"memory");
		*mem=a;

	} while (memkb<4096 && mem_count!=0);

	__asm__ __volatile__("movl %%eax, %%cr0", :: "a" (cr0) : "eax");

	mem_end = memkb<<20;
	mem = (ULONG*) 0x413;
	bse_end= (*mem & 0xFFFF) <<6;

	outb(0x21, irq1);
	outb(0xA1, irq2);
}
</syntaxhighlight>

===Manual Probing in ASM===

This is the "least unsafe" algorithm for memory probing. It is "non-destructive" of the contents of memory, and is just generally better than the above C code.

Notes:

* <b>NEVER</b> use manual probing unless you absolutely <b>MUST</b>. This implies that the manual probing code is only used for dodgy old computers, and that the code for manual probing needs to be designed for dodgy old computers (assumptions that are "good" assumptions for modern computers, like "it's unlikely that an ISA video card is present", don't apply). 
* minimize the amount of manual probing you do. For example, if the BIOS supports "Int 0x12" (they all do) then use it to avoid probing RAM below 1 MB. If "Int 0x15, AH=0x88" says there's 0xFFFF KB at 0x00100000 and you think there's more (because a 16-bit value can't tell you there's more if there is) then probe from the end of known RAM (not from 0x00100000). 
* don't assume that writes to "non-RAM" won't be cached (flush the cache with WBINVD or CLFLUSH after testing to make sure you're testing the physical address and not the cache). 
* don't assume that writes to "non-RAM" won't be retained due to bus capacitance (use a dummy write at a different address to avoid this, so you read back the dummy value and not the test value if there's no RAM at the address). 
* don't write a set value to an address and read it back to test for RAM (for e.g. "mov [address],0x12345678; mov [dummy],0x0; wbinvd; cmp [address],0x12345678") because you might be unlucky and find a ROM that contains the same value you're using. Instead try to modify what's already there. 
* test the last bytes of each block, not the first bytes of each block, and make sure that the size of each block is less than 16 KB. This is because some older motherboards relocate the RAM underneath the ROM area to the top of memory (e.g. a computer with 2 MB of RAM might have 128 KB of ROM from 0x000E0000 0x000FFFFF and RAM from 0x00100000 to 0x0020FFFF. 
* don't make any assumptions about the "top of memory". Just because the last byte of RAM is at 0x0020FFFF doesn't mean that there's 2176 KB of RAM installed, and just because there's 2 MB of RAM installed doesn't mean that the last byte of RAM will be at 0x001FFFFF. 
* it's better to assume that memory holes are present (and risk skipping some RAM) than to assume that memory holes aren't present (and risk crashing). This means assuming that the area from 0x00F00000 to 0x00FFFFFF can't be used and not probing this area at all (it's possible that some sort of ISA device is in this area, and that any write to this area can cause problems).

<syntaxhighlight lang="nasm">
;Probe to see if there's RAM at a certain address 
;
; note: "dummy" -> a known good memory address that has nothing important in it
; 
;Input 
; edx   Maximum number of bytes to test 
; esi   Starting address 
; 
;Output 
; ecx   Number of bytes of RAM found 
; esi   Address of RAM 

probeRAM: 
    push eax
    push ebx
    push edx
    push ebp 
    mov ebp,esi             ;ebp = starting address
    add esi,0x00000FFF      ;round esi up to block boundary
    and esi, ~0x00000FFF    ;truncate to block boundary
    push esi                ;Save corrected starting address for later
    mov eax, esi            ;eax = corrected starting address
    sub eax, ebp            ;eax = bytes to skip from original starting address, due to rounding
    xor ecx,ecx             ;ecx = number of bytes of RAM found so far (none)
    sub edx,eax             ;edx = number of bytes left to test
    jc .done                ;  all done if nothing left after rounding
    or esi,0x00000FFC       ;esi = address of last uint32_t in first block
    shr edx,12              ;edx = number of blocks to test (rounded down)
    je .done                ; Is there anything left after rounding?

.testAddress:
    mov eax,[esi]           ;eax = original value
    mov ebx,eax             ;ebx = original value
    not eax                 ;eax = reversed value
    mov [esi],eax           ;Modify value at address
    mov [dummy],ebx         ;Do dummy write (that's guaranteed to be a different value)
    wbinvd                  ;Flush the cache
    mov ebp,[esi]           ;ebp = new value
    mov [esi],ebx           ;Restore the original value (even if it's not RAM, in case it's a memory mapped device or something)
    cmp ebp,eax             ;Was the value changed?
    jne .done               ; no, definitely not RAM -- exit to avoid damage
                            ; yes, assume we've found some RAM

    add ecx,0x00001000      ;ecx = new number of bytes of RAM found
    add esi,0x00001000      ;esi = new address to test
    dec edx                 ;edx = new number of blocks remaining
    jne .testAddress        ;more blocks remaining?
                            ;If not, we're done

.done:
    pop esi                 ;esi = corrected starting address (rounded up)
    pop ebp
    pop edx
    pop ebx
    pop eax
    ret 
</syntaxhighlight>

Further Notes:
* Depending on how it's used some of the initial code could be skipped (e.g. if you know the starting address is always aligned on a 4 KB boundary). 
* the WBINVD instruction seriously affects performance because it invalidates all data in all caches (except the TLB). It would be better to use CLFLUSH so you only invalidate the cache line that needs to be invalidated, but CLFLUSH isn't supported on older CPUs (and older computers is what this code is for). For older computers It shouldn't be too slow because the speed difference between cache and RAM wasn't so much and there's usually only a small amount of RAM (e.g. 64 MB or less). Modern computers have a lot more RAM to test and rely on caches a lot more. For example, an 80486 with 32 MB of RAM might take 1 second, but a Pentium 4 with 2 GB of RAM might take 30 seconds or more. 
* Increasing the block size (e.g. testing every 16 KB instead of testing every 4 KB) will improve performance (and increase risk). 16 KB blocks is probably safe, and larger blocks sizes are not safe. Very large block sizes (e.g. testing every 1 MB) will probably work on modern computers (but you shouldn't need to probe at all on modern computers), and anything larger than 1 MB is guaranteed to give wrong results regularly.
* WBINVD isn't supported on 80386 and older computers. This means that for 80386 and older you can't flush the cache, but this shouldn't matter (for 80386 and older memory ran at the same speed as the CPU so there was no need for a cache). You will need to flush cache on later CPUs though. Having one routine that uses WBINVD and another routine that doesn't use WBINVD is probably better than doing an "if (CPU_is_80486_or_newer) { WBINVD }" in the middle of the loop.

==See Also==

===Threads===
* [http://www.osdev.org/phpBB2/viewtopic.php?t=11391 Grub's multiboot memory map], featuring real examples of GRUB/BIOS reported memory map.
* [https://forum.osdev.org/viewtopic.php?t=30318 QEMU Multiboot Invalid Memory Map], a potential pitfall when detecting memory with GRUB

[[Category:X86]]
[[Category:Physical Memory]]
[[Category:Hardware Detection]]
````

<a id="w002281"></a>

## W002281 — Talk:Detecting Memory (x86)

Bron-ID: W002281
Namespace: Talk (1)
Revisie: 18832 | Datum: 2016-01-09T22:02:47Z
Bron: https://wiki.osdev.org/Talk:Detecting_Memory_(x86)
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ADetecting_Memory_%28x86%29&oldid=18832

````text
== Deleting old code ==

Now that Brendan's manual probing code is here, I'd like to delete the old C manual probing code -- even though "information will be lost". <br>
[[User:Bewing|Bewing]] 01:26, 22 May 2008 (CDT)

'''I'd rather not''' for just that reason :) - Basically you'll be denying everybody that expects to find a sample in C that code. It's not too bad to have a version in both languages really. - [[User:Combuster|Combuster]] 06:11, 22 May 2008 (CDT)

== Updated the low memory snippet ==

The old low memory detection snippet using INT 0x12; AH = 0x00 assumed that AH would be set to 0x80 or 0x86 when something went wrong, I don't think this can be true as RBIL doesn't state this and as AX itself should contain the size. If somebody doesn't like my assembly style, then he/she's free to change it. If someone doesn't agree with me on the 0x80/0x86 part, then could he/she please explain why.

--[[User:Stephanvanschaik|Stephanvanschaik]] 20:32, 19 February 2010 (UTC)

== e820 ==

I'm having a bit of a hard time understanding the workings of e820, and I've had problems finding an answer elsewhere on the internet which assumes you already understand this bit. It's pretty clear that e820 is a real-mode only call; however, you need to set register EDX with the appropriate magic number. Whereas all other registers can have their lower 16 bits set fine, the magic number is a 32-bit value, and EDX is not addressable with 16-bit code. How does one run real mode with 32-bit code and addressing? [[User:Surgo|Surgo]] 19:52, 20 January 2011 (UTC)
:You are making the wrong assumption here: you can access all 32-bit GPRs, all 80-bits FPRs, and even the 64 and 128-bit vector registers with 16-bit code in real mode, EDX being no exception. You are even required to use a 32-bit register to enter protected mode as CR0 is 32 bits as well. The only thing real mode forbids is using segment offsets over 64KB for backwards compatibility and even that can be [[Unreal Mode|disabled]]. Your assembler will take care of the rest and insert the correct opcodes for you. - [[User:Combuster|Combuster]] 22:55, 20 January 2011 (UTC)
::Wow, that was a pretty boneheaded mistake. I think misreading some assembler output caused my misconception. Thank you! [[User:Surgo|Surgo]] 23:05, 20 January 2011 (UTC)

== INT 0x12 ==

The [http://www.ctyme.com/intr/rb-0598.htm RBIL] doesn't say this interrupt returns error, and it's impossible, since this function was there since the very first IBM PC computer, so there was no need to set/reset the carry flag. It's not possible for this function not to be supported. In fact, I had trouble with this and it didn't work for me until I removed the error checking.
[[User:Unsigned|Unsigned]] 19:15, 22 August 2015 (CDT)

: RBIL does say that different functions are invoked for certain values for AX, so you broke the code there. It also states that it might return the value directly from the BDA, which isn't reserved memory and might have changed. A listing in RBIL does not preclude presence in all emulators either. Maybe the more correct way for your code is to clear carry ahead of time (BIOSes are known to be lacking in that), or defer to E820 anyway when int 0x12 fails. - [[User:Combuster|Combuster]] 02:56, 23 August 2015 (CDT)

== Memory via UEFI ==

Uefi also supports getting the memory map via FI_BOOT_SERVICES->GetMemoryMap. Would be nice to have an example and explanation.
````

<a id="w002282"></a>

## W002282 — User:KrnlHckr

Bron-ID: W002282
Namespace: User (2)
Revisie: 5504 | Datum: 2008-05-30T14:54:37Z
Bron: https://wiki.osdev.org/User:KrnlHckr
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKrnlHckr&oldid=5504

````text
Hello!  My name is Sean and I go by KrnlHckr on the OSDev.org board and wiki.  My blog can be found at [http://krnlhckr.blogspot.com blogspot].

== Early Days ==
=== TRS-80 Model III ===
Like so many people in my age bracket, my experience with computers came at an early age when computers in the home were an exceedingly rare thing.  I was 10 when my Dad brought home a TRS-80 Model III from the local Radio Shack.  He and I spent countless hours on that machine, entering code from Byte magazine, taking it apart and examining its insides and upgrading from 16k to 32k of RAM, and basically having a grand time with the new contraption.  While Dad spent his time, when we weren't under the hood, using applications and other tools, I was pouring over the BASIC manual learning how to write programs.

My first coding attempt was some improvements to the TRS-80's "Cannon" program.  It was a silly little graphics program that had a cannon on a castle wall that shot a cannonball every 5 seconds or so.  My modifications allowed the cannoneer to adjust the "power" of the shot which allowed the ball to land closer or farther than the pre-programmed constant distance.  It wasn't anything scientific and surely violated established laws of physics, but who cared!  After I cut my teeth on BASIC I switched over to Z-80 assembly.  Dad had brought home the editor/assembler cassettes and I took to it like a duck to water.  I did all sorts of wonderfully silly things like video manipulation, bouncing characters of the "walls" of the display, and memory dumping (where I learned where the ROM OS was for the TRS-80).

=== C-64 ===
I graduated to the god-like Commodore 64 a few years later and my 6502 manual took a massive beating as I learned that chip.  I loved disk editors and sector nibblers and took great pains to steal as many C-64 games as I (and my friends) could lay our hands on.  At age 12 or so money was not easy to come by and there was no way I was going to be able to pay $25 for a video game.  All my software was pirated and I didn't pay it any mind because ethics of hacking didn't extend to dollars and sense.  I used my C-64 with a cassette drive since I couldn't afford the 1541 floppy drive.  When I was finally able to buy one, though, it was GAME ON!

I wrote sprite routines, did collision detection, wrote a (really nasty) hex editor for disks, and generally had myself glued to this thing 24x7.

=== Tandy 1000EX ===
The day the Tandy 1000EX showed up at our house was a memorable moment.  Although I had my C-64 (we sold the TRS-80 tragically) I still spent a good deal of time on this new machine.  I realized that GW-BASIC was similar to the other BASICs I already knew, but I would also get to learn Pascal and LOGO.  This box would eventually become mine when I went off to college.

== A Career ==
=== PC Support ===
Because of my exposure to PC hardware, I didn't have any trouble getting a job in college.  I joined the PC support team on campus and ran around fixing broken machines, programming jobs, and doing some telecom work.  Biggest time wasters were MUDding and IRC.  I began learning C and assembly on the 8088.

=== UNIX and Linux ===
I got my first exposure to UNIX in 1991 while in high school.  My AP Pascal course was done remotely and our assignments came to us over a UNIX hosted BBS system.  When I got to college, however, the plethora of UNIX systems was mind boggling and I had (in addition to my mainframe account) accounts on four UNIX systems.  AIX, SunOS, Digital UNIX (before they porked with Tru64), and some weird thing called Xenix.  My UNIX reference was a huge SVR4 textbook and it allowed me to make sense of all the fun things that UNIX did that DOS and Windows just didn't bring to the table.

When I got my first Linux book in 1992, I was in the Army and bought a 486 machine just for it.  I played around with it and that is what brought me into my UNIX sysadmin job that I have now.

== Who Cares? ==
Nobody but me perhaps, but this is the OS Dev community and we all love to talk about "the old days" even if the only person listening (or reading) is ourself.  I've never liked applications programming.  I have always prefered being in the trenches close to the hardware.  I like the power that the OS, the kernel in particular, give you.  As an OS developer you literally hold the stability of the system in your hands.  As an applications freak, your poorly written code might crash the app.  As a kernel junkie, your poorly written code will likely crash the system and bring down everything and everyone who uses it.

Rock on, then!

=== Screenshot ===
One thing we love about this community is how much we all adore our screenshots.  Below is my gratuitous contribution, in all its lack-luster glory.
[[Image:Test.png | Screenshot]]
````

<a id="w002283"></a>

## W002283 — File:Test.png

Bron-ID: W002283
Namespace: File (6)
Revisie: 5500 | Datum: 2008-05-29T21:21:14Z
Bron: https://wiki.osdev.org/File:Test.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ATest.png&oldid=5500

````text

````

<a id="w002285"></a>

## W002285 — User:Stevo14

Bron-ID: W002285
Namespace: User (2)
Revisie: 6268 | Datum: 2008-09-16T20:51:09Z
Bron: https://wiki.osdev.org/User:Stevo14
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AStevo14&oldid=6268

````text
This is the user page of Stevo14, otherwise known in the real world as Stephen Dunn.

== Operating System ==
My hobby OS, called "Fluidium", can be found [http://code.google.com/p/fluidium/ here].

== Contributions ==
My contributions to this wiki are listed [http://www.osdev.org/wiki/Special:Contributions/Stevo14 here].

== Scratch Space ==
...empty...
````

<a id="w002288"></a>

## W002288 — Template:Main2/Reference Information

Bron-ID: W002288
Namespace: Template (10)
Revisie: 26349 | Datum: 2021-08-20T00:38:47Z
Bron: https://wiki.osdev.org/Template:Main2/Reference_Information
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AMain2%2FReference_Information&oldid=26349

````text
'''Executable File Types'''
* '''64 bit'''
** [[ELF]]
** [[PE]]
** [[COFF]]
* '''32 bit'''
** [[ELF]]
** [[PE]]
** [[COFF]]
* '''16 bit'''
** [[COM]]
** [[MZ]]
** [[NE]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Executable Formats|more...]]</small></div>

<big>'''Filesystems'''</big>
* [[FAT|FAT Details]]
* [[Ext2|Ext2 Details]]
* [[SFS|SFS Details]]
* [[NTFS|NTFS Details]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Filesystems|more...]]</small></div>
<noinclude>[[Category:Main Page Templates]]</noinclude>
````

<a id="w002289"></a>

## W002289 — Template:Main2/Hardware

Bron-ID: W002289
Namespace: Template (10)
Revisie: 30852 | Datum: 2026-07-06T21:43:13Z
Bron: https://wiki.osdev.org/Template:Main2/Hardware
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AMain2%2FHardware&oldid=30852

````text
* The [[:Category:Hardware|Hardware]] Category and [[Resources#Hardware_Specifications|linked resources]]

<big>
'''CPU''' <br/>
</big>

'''General'''<br />
* [[Register Machine Model of Computation]]
* [[Memory Architecture]]
* [[Instruction Set Architecture]]
** [[Historical Notes on CISC and RISC]]
* [[Execution Architecture]]
** [[Multiprocessing Architecture]]

'''X86''' <br/>
<small>
'''Modes'''
</small>
*[[Real Mode]]
*[[Protected Mode]]
* [[Virtual 8086 Mode]]
* [[X86-64|x86-64]] <!--Architecture Family-->
<small>
'''Other'''
</small>
* [[IA32 Architecture Family]]
* [[Model Specific Registers]] (MSRs)
* [[Detecting CPU Speed]]
* [[Inline Assembly/Examples]]
* [[CPU Bugs]]
* [[SSE]]
* [[Where Can I Find Information About Ports]]
* CPU Registers ([[CPU_Registers_x86|x86]], [[CPU_Registers_x86-64|x86-64]])
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:X86 CPU|more...]]</small></div>

'''ESA/390'''
* [[S390 Barebones|Kernel Barebones]]
* [[Program Status Word]]
* [[PSA Mapping|Processing Storage Area Memory Mapping]]
* [[ZDSFS|z/OS DataSet FileSystem]]
* [[Command SubSystem|Batch I/O Channel SubSystem]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:S390|more...]]</small></div>

'''ARM'''
* [[ARM Overview]]
* [[Raspberry Pi Bare Bones]]
* [[AArch64 Exceptions]]

<big>
'''Memory'''
</big>
* [[Detecting Memory (x86)]]
* [[A20 Line]] or "Why can't I access all my memory?"


<big>
'''IRQs and Exceptions, PIC, NMI, APIC, OPIC'''
</big>
* [[I Can't Get Interrupts Working]]
* The [[Interrupt_Descriptor_Table | Interrupt Descriptor Table]] and [[IDT_problems | IDT problems]]
* [[Interrupt Service Routines]] (ISRs)
* [[Interrupts]]
* [[Non Maskable Interrupt]] (NMI)
* [[APIC]]
* [[PIC]]
* [[OPIC]]

<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Interrupts|more...]]</small></div>

<big>
'''Clocks, Timers and Counters'''
</big>
* [[HPET]]
* [[APIC timer]] (per-processor timer)
* [[Programmable Interval Timer]] (PIT)
* The [[CMOS]] clock, and its [[RTC]] chip
* [[Detecting CPU Speed]]


<big>
'''Video'''
</big>
* [[Accelerated Graphic Cards]]
** [[Native Intel graphics]]
* [[Detecting Colour and Monochrome Monitors]]?
* [[Printing To Screen]] in Protected Mode
* [[Text Mode Cursor]]
* [[Drawing In a Linear Framebuffer]]
* [[Kernel Stdio Theory]]
* [[VGA Resources]] and [[VGA Hardware]] documentation
* Drawing characters in graphics mode using [[VGA Fonts]]
* [[Video Signals And Timing]]
* How to program Bochs' and QEMU's [[Bochs VBE Extensions | BGA]] graphics device
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Video|more...]]</small></div>

<big>
'''Audio'''
</big>
* [[Sound]]
* [[PC Speaker]]
* [[Intel High Definition Audio]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Sound|more...]]</small></div>

<big>
'''Storage devices'''
</big>
* [[:Category:ATA|ATA]] (harddisks)
* [[ATAPI]] (CD-ROM)
* [[AHCI]] ([[SATA]] native)
* [[ISA DMA | DMA]]
* [[NVMe]]
* [[Floppy Disk Controller]] (FDC)
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Storage|more...]]</small></div>

<big>
'''Input/Output'''
</big>
* [[PS/2 Keyboard]]
* [[Mouse Input | Mouse]]
* [[Serial Ports]] (primarily RS-232)


<big>
'''Plug and Play'''
</big>
* [[Plug-and-Play]] (PnP)
* [[PNP Calls In Protected Mode|PnP Calls In Protected Mode]]


<big>
'''PCI, USB and things like this ...'''
</big>
* [[PCI]] 
* [[PCI Express]]
* [[Universal Serial Bus | USB]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Buses|more...]]</small></div>

<big>
'''UEFI'''
</big>
*[[UEFI]]
*Systems with [[broken UEFI implementations]]


<big>
'''ACPI'''
</big>
*[[ACPI]]
*[[RSDP]]
*[[AML]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:ACPI|more...]]</small></div>

<big>
'''Network cards and communicating devices'''
</big>
* Ethernet and other wired networking:
** [[3c90x]]
** [[Intel 8254x]]
** [[Ne2000]]
** [[RTL8139]]
** [[RTL8169]]
** [[Intel Ethernet i217]]
** [[AMD PCNET|AMD PCnet]]
* Wireless Networking:
** [[802.11]] (WiFi standard)
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Network Hardware|more...]]</small></div>

<big>
'''Non-x86 Development'''
</big>
* [[:Category:PowerPC|PowerPC]]
* [[:Category:MIPS|MIPS]]
* [[:Category:Sparc|Sparc]]
* [[:Category:ARM|ARM]]
* [[:Category:RISC-V|RISC-V]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Platforms|more...]]</small></div>

<noinclude>[[Category:Main Page Templates]]</noinclude>
````

<a id="w002290"></a>

## W002290 — Template:Main2/Resources

Bron-ID: W002290
Namespace: Template (10)
Revisie: 19163 | Datum: 2016-05-08T23:32:47Z
Bron: https://wiki.osdev.org/Template:Main2/Resources
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AMain2%2FResources&oldid=19163

````text
* [[Books]] on OS Development
* [[Academic Papers]] on theory and design
* [[Assembly]]
* [[Disk_Images#Images_with_preinstalled_GRUB | Disk Images with preinstalled GRUB]]
* [[Deprecated]], yet relevant, bits of specifications
* [[Chip Numbers, Acronyms and Things]]
* [[Resources]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Resources|more...]]</small></div>
<noinclude>[[Category:Main Page Templates]]</noinclude>
````

<a id="w002291"></a>

## W002291 — Template:Main2/Introduction

Bron-ID: W002291
Namespace: Template (10)
Revisie: 27394 | Datum: 2022-09-11T06:21:11Z
Bron: https://wiki.osdev.org/Template:Main2/Introduction
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AMain2%2FIntroduction&oldid=27394

````text
'''Basic Information'''
* [[Introduction]]
* [[Required Knowledge]]
* [[Beginner Mistakes]]
* [[Getting Started]]
* [[How To Ask Questions]]
* [[GCC Cross-Compiler]]
* [[What Order Should I Make Things In?]]
* [[Licensing]]
* [[Tutorials]]
* '''[[:Category:FAQ|Frequently Asked Questions]]'''
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:OS Development|more...]]</small></div>

'''Environment'''
* [[How kernel, compiler, and C library work together]]
* Using other [[Languages]] than C
* [[UEFI]] and legacy [[BIOS]] (PC)
* [[Inline Assembly]] (including [[Inline Assembly/Examples | examples]])
* [[Object Files]] and relocation


'''Bare Bones / Baby Steps'''
* [[Bare Bones]] (C and C++)
* [[Ada Bare Bones]]
* [[C Sharp Bare Bones]]
* [[D Bare Bones]]
* [[FreeBasic Bare Bones]]
* [[Go Bare Bones]]
* [[Pascal Bare Bones]]
* [[Limine Bare Bones]]
* [[:Category:Babystep | The BabyStep Tutorial]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Bare bones tutorials|more...]]</small></div>

'''Example OS Organization'''
* [[Meaty Skeleton]]


'''Booting and Setup'''
* [[Boot Sequence]]
* [[Diskless Booting]]
* The [[Global_Descriptor_Table | Global Descriptor Table]] and a [[GDT Tutorial]]
* [[Interrupts]]
* [[PS2 Keyboard | Getting Keyboard Input]]


'''Testing and Debugging'''
* [[Troubleshooting]]
* [[Testing]]
* [[Unit Testing]]
* [[How Do I Use A Debugger With My OS]]
* [[Kernel Debugging]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Troubleshooting|more...]]</small></div>

'''Development'''
* [[Going Further on x86]]
* [[Creating an Operating System]]
* [[Code Management]]


'''Compiler Development'''
* [[Compiler|What's a compiler?]]
* [[Making a Compiler | Making a Compiler (Theory)]]
* [[Code Generation and Optimization Techniques]]
** [[Implementing Conditional Statements And Loops]]
** [[Tail Recursion and Tail Call Optimization]]

<noinclude>[[Category:Main Page Templates]]</noinclude>
````

<a id="w002292"></a>

## W002292 — Template:Main2/Tools

Bron-ID: W002292
Namespace: Template (10)
Revisie: 27455 | Datum: 2022-10-05T01:04:11Z
Bron: https://wiki.osdev.org/Template:Main2/Tools
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AMain2%2FTools&oldid=27455

````text
'''Bootloaders'''
* [[Bootloader]] theory
* [[GRUB]]
* [[Limine]]
* [[LILO]]
* [[XOSL]]
* [[System Commander]]
* [[Boot Magic]]
* [[Rolling Your Own Bootloader]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Bootloaders|more...]]</small></div>

'''Compilers'''
* [[GCC]]
** [[Cygwin]]
** [[MinGW]]
** [[Dev-C++]]
** [[DJGPP]]
** '''Building a [[GCC Cross-Compiler]]'''
** [[Cross-Compiler Successful Builds]]
** [[Porting GCC to your OS]]
* [[Calling Conventions]]
** [[System V ABI]]
* [[Watcom]] C/C++
* [[Visual Studio]]
** [[Visual C++ Runtime]]
* [[TCC]]
* [[Smaller C]]
* [[Other Compilers]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Compilers|more...]]</small></div>

'''Assemblers'''
* [[GAS]] (GNU as, part of binutils)
* [[FASM]]
* [[NASM]]
* [[YASM]]
* On the difference between AT&T and Intel [[Opcode syntax]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Assemblers|more...]]</small></div>

'''Linkers'''
* [[LD]] (GNU ld, part of binutils)
* [[Link Archiver]] (GNU ar, part of binutils)
* [[Mold]] (Modern Linker)
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Linkers|more...]]</small></div>

'''Emulators'''
* [[Bochs]]
* [[QEMU]]
* [[Microsoft Virtual PC|Virtual PC]]
* [[VMware]]
* [[VirtualBox]]
* [[SimICS]]
* [[Lingemu]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Emulators|more...]]</small></div>

'''Working with Disk Images'''
* [[Disk Images]] in general, and ...
** [[Loopback Device]] for Linux, *BSD et al. 
** [[hdiutil]] for MacOS X
** [[Windows Tools]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Disk_Image_Utilities|more...]]</small></div>
<noinclude>[[Category:Main Page Templates]]</noinclude>
````

<a id="w002293"></a>

## W002293 — Template:Main2/Design Considerations

Bron-ID: W002293
Namespace: Template (10)
Revisie: 29988 | Datum: 2026-02-13T17:23:57Z
Bron: https://wiki.osdev.org/Template:Main2/Design_Considerations
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AMain2%2FDesign_Considerations&oldid=29988

````text
'''Kernel Models'''
* [[Monolithic Kernel]]
* [[Microkernel]]
* [[Exokernel]] and related models
* [[Modular Kernel]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[Kernels|more...]]</small></div>

'''Task Models'''
* [[Monotasking Systems]]
* [[Multitasking Systems]]
* [[Real-Time Systems]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Task Models|more...]]</small></div>

'''Memory and Resource Management'''
* [[Memory management]] - Overview and Introduction
* [[Segmentation]]
* [[Paging]]
* [[Memory Allocation]]
* [[Page Frame Allocation]]
* [[Memory Management Unit]], or in-depth description of recursive mappings
* [[File Management]]
* [[File Systems]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Memory management|more...]]</small></div>

'''System Calls'''
* [[SYSENTER|x86 System Calls]]
* [[ARM_System_Calls|ARM System Calls]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:System Calls|more...]]</small></div>

'''Scheduling'''
* [[Processes and Threads]]
* [[Context Switching]]
* [[Scheduling Algorithms]]
* [[Multiprocessor Scheduling]]
* [[Blocking Process]] - processes sleeping and waiting
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Processes_and_Threads|more...]]</small></div>

'''Process Sychronization'''
* [[Synchronization Primitives]] - Semaphores, mutexes, spinlocks et al.
* [[Signals]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Synchronization|more...]]</small></div>

'''Inter-Process Communication'''
* [[Message Passing]]
* [[Shared Memory]]
* [[Remote Procedure Call]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:IPC|more...]]</small></div>

<noinclude>[[Category:Main Page Templates]]</noinclude>
````

<a id="w002294"></a>

## W002294 — Template:NocatBox

Bron-ID: W002294
Namespace: Template (10)
Revisie: 5688 | Datum: 2008-06-21T18:12:39Z
Bron: https://wiki.osdev.org/Template:NocatBox
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ANocatBox&oldid=5688

````text
<div style="margin: 0 0 10px 0; padding: 0 1em 0.7em 1em; background-color: #F5FAFF; color: #000000; border: 1px solid #CEDFF2; padding:0.6em 0.8em;">

<h2 style="margin:0;background-color:#CEDFF2;font-size:120%;font-weight:bold;border:1px solid #A3B0BF;text-align:left;color:#000;padding:0.2em 0.4em;">{{{subject}}}</h2>
{{{{{page}}}}}</div>
````

<a id="w002296"></a>

## W002296 — Category:OS Development

Bron-ID: W002296
Namespace: Category (14)
Revisie: 5576 | Datum: 2008-06-04T19:28:12Z
Bron: https://wiki.osdev.org/Category:OS_Development
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AOS_Development&oldid=5576

````text
Articles regarding the process of developing an operating system.

[[Category:Main Page Topics]]
````

<a id="w002297"></a>

## W002297 — Template:OS development

Bron-ID: W002297
Namespace: Template (10)
Revisie: 5590 | Datum: 2008-06-04T19:33:05Z
Bron: https://wiki.osdev.org/Template:OS_development
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AOS_development&oldid=5590
Doorverwijzing: Template:OS Development

````text
#REDIRECT [[Template:OS Development]]
````

<a id="w002298"></a>

## W002298 — Monolithic kernel

Bron-ID: W002298
Namespace: Main (0)
Revisie: 5593 | Datum: 2008-06-04T19:36:06Z
Bron: https://wiki.osdev.org/Monolithic_kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Monolithic_kernel&oldid=5593
Doorverwijzing: Monolithic Kernel

````text
#REDIRECT [[Monolithic Kernel]]
````

<a id="w002299"></a>

## W002299 — Semaphores

Bron-ID: W002299
Namespace: Main (0)
Revisie: 5603 | Datum: 2008-06-04T19:53:54Z
Bron: https://wiki.osdev.org/Semaphores
Revisie-URL: https://wiki.osdev.org/index.php?title=Semaphores&oldid=5603
Doorverwijzing: Synchronization Primitives

````text
#REDIRECT [[Synchronization Primitives]]
````

<a id="w002300"></a>

## W002300 — Talk:Semaphores

Bron-ID: W002300
Namespace: Talk (1)
Revisie: 6224 | Datum: 2008-08-20T16:12:14Z
Bron: https://wiki.osdev.org/Talk:Semaphores
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ASemaphores&oldid=6224
Doorverwijzing: Talk:Synchronization Primitives

````text
#REDIRECT [[Talk:Synchronization Primitives]]
````

<a id="w002302"></a>

## W002302 — Talk:Synchronisation Primitives

Bron-ID: W002302
Namespace: Talk (1)
Revisie: 5602 | Datum: 2008-06-04T19:53:14Z
Bron: https://wiki.osdev.org/Talk:Synchronisation_Primitives
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ASynchronisation_Primitives&oldid=5602
Doorverwijzing: Talk:Synchronization Primitives

````text
#REDIRECT [[Talk:Synchronization Primitives]]
````

<a id="w002303"></a>

## W002303 — Template:Stylenav

Bron-ID: W002303
Namespace: Template (10)
Revisie: 10138 | Datum: 2010-04-26T19:42:17Z
Bron: https://wiki.osdev.org/Template:Stylenav
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AStylenav&oldid=10138

````text
<div style="margin: 0 0 10px 0; padding: 0 1em 0.7em 1em; background-color: #F5FAFF; color: #000000; border: 1px solid #CEDFF2; padding:0.2em 0.2em; text-align: center;">
'''Display: [[Categorized Main Page|Short view]] - [[Expanded Main Page|Expanded view]]'''
</div>
````

<a id="w002304"></a>

## W002304 — Talk:MBR (x86)

Bron-ID: W002304
Namespace: Talk (1)
Revisie: 5903 | Datum: 2008-07-11T23:24:10Z
Bron: https://wiki.osdev.org/Talk:MBR_(x86)
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMBR_%28x86%29&oldid=5903

````text
This page is all over the place. I vote for a rewrite. What should be kept and what destroyed? [[User:Yayyak|Yayyak]] 05:15, 5 June 2008 (CDT)

I agree. Major rewrite. According to Wikipedia, the master boot record has two functions, the partition table and the bootstrapping code. The article should be organized around those functions IMHO.
[[User:Stevo14|Stevo14]] 04:58, 6 June 2008 (CDT)


OK -- I took my shot at rewriting it. I'll post a code example into it, soon. Other than that, if anyone else wants to add or change something now, go for it.<br>[[User:Bewing|Bewing]] 23:24, 11 July 2008 (UTC)
````

<a id="w002305"></a>

## W002305 — File:Old Sound Card.jpg

Bron-ID: W002305
Namespace: File (6)
Revisie: 5632 | Datum: 2008-06-07T11:58:19Z
Bron: https://wiki.osdev.org/File:Old_Sound_Card.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AOld_Sound_Card.jpg&oldid=5632

````text
http://www.flickr.com/photos/clearfo/368289222

Creative Commons License

[[Category:Sound]]
````

<a id="w002316"></a>

## W002316 — File:Really Simple Audio Architecture.png

Bron-ID: W002316
Namespace: File (6)
Revisie: 5702 | Datum: 2008-06-27T08:57:47Z
Bron: https://wiki.osdev.org/File:Really_Simple_Audio_Architecture.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AReally_Simple_Audio_Architecture.png&oldid=5702

````text
[[Category:Sound]]
````

<a id="w002317"></a>

## W002317 — KBC

Bron-ID: W002317
Namespace: Main (0)
Revisie: 17682 | Datum: 2015-03-02T16:19:02Z
Bron: https://wiki.osdev.org/KBC
Revisie-URL: https://wiki.osdev.org/index.php?title=KBC&oldid=17682
Doorverwijzing: PS/2 Keyboard

````text
#REDIRECT [[PS/2 Keyboard#The Keyboard Controller]]
````

<a id="w002318"></a>

## W002318 — File:At-kbc.png

Bron-ID: W002318
Namespace: File (6)
Revisie: 5707 | Datum: 2008-06-28T15:05:35Z
Bron: https://wiki.osdev.org/File:At-kbc.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AAt-kbc.png&oldid=5707

````text
Overview of the AT Keyboard-Controller
````

<a id="w002319"></a>

## W002319 — File:Ps2-kbc.png

Bron-ID: W002319
Namespace: File (6)
Revisie: 5706 | Datum: 2008-06-28T15:01:40Z
Bron: https://wiki.osdev.org/File:Ps2-kbc.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3APs2-kbc.png&oldid=5706

````text
Overview of the PS/2 Keyboard-Controller
````

<a id="w002321"></a>

## W002321 — VGA

Bron-ID: W002321
Namespace: Main (0)
Revisie: 6820 | Datum: 2008-12-09T00:27:24Z
Bron: https://wiki.osdev.org/VGA
Revisie-URL: https://wiki.osdev.org/index.php?title=VGA&oldid=6820
Doorverwijzing: Category:VGA

````text
#REDIRECT [[:Category:VGA]]
````

<a id="w002322"></a>

## W002322 — Entering Long Mode Directly

Bron-ID: W002322
Namespace: Main (0)
Revisie: 30302 | Datum: 2026-03-16T20:32:46Z
Bron: https://wiki.osdev.org/Entering_Long_Mode_Directly
Revisie-URL: https://wiki.osdev.org/index.php?title=Entering_Long_Mode_Directly&oldid=30302

````text
For some reason, you might want to switch to 64 bit mode in your bootloader without entering [[protected mode]]. The following article practically described how to do so, mainly demonstrating it by example assembly code. Additionally, some may find it easier to switch directly to long mode on additional CPUs as they are initialized for [[Symmetric Multiprocessing]].

== Switching to Long Mode ==

The following can be used to switch directly to Long Mode. The code is well documented through comments, and nearly explains almost everything it does.

<syntaxhighlight lang="asm">
%define PAGE_PRESENT    (1 << 0)
%define PAGE_WRITE      (1 << 1)

%define CODE_SEG     0x0008
%define DATA_SEG     0x0010

ALIGN 4
IDT:
    .Length       dw 0
    .Base         dd 0

; Function to switch directly to long mode from real mode.
; Identity maps the first 2MiB.
; Uses Intel syntax.

; es:edi    Should point to a valid page-aligned 16KiB buffer, for the PML4, PDPT, PD and a PT.
; ss:esp    Should point to memory that can be used as a small (1 uint32_t) stack

SwitchToLongMode:
    ; Zero out the 16KiB buffer.
    ; Since we are doing a rep stosd, count should be bytes/4.   
    push di                           ; REP STOSD alters DI.
    mov ecx, 0x1000
    xor eax, eax
    cld
    rep stosd
    pop di                            ; Get DI back.


    ; Build the Page Map Level 4.
    ; es:di points to the Page Map Level 4 table.
    lea eax, [es:di + 0x1000]         ; Put the address of the Page Directory Pointer Table in to EAX.
    or eax, PAGE_PRESENT | PAGE_WRITE ; Or EAX with the flags - present flag, writable flag.
    mov [es:di], eax                  ; Store the value of EAX as the first PML4E.


    ; Build the Page Directory Pointer Table.
    lea eax, [es:di + 0x2000]         ; Put the address of the Page Directory in to EAX.
    or eax, PAGE_PRESENT | PAGE_WRITE ; Or EAX with the flags - present flag, writable flag.
    mov [es:di + 0x1000], eax         ; Store the value of EAX as the first PDPTE.

    
    ; Build the Page Directory.
    lea eax, [es:di + 0x3000]         ; Put the address of the Page Table in to EAX.
    or eax, PAGE_PRESENT | PAGE_WRITE ; Or EAX with the flags - present flag, writeable flag.
    mov [es:di + 0x2000], eax         ; Store to value of EAX as the first PDE.
    
    
    push di                           ; Save DI for the time being.
    lea di, [di + 0x3000]             ; Point DI to the page table.
    mov eax, PAGE_PRESENT | PAGE_WRITE    ; Move the flags into EAX - and point it to 0x0000.


    ; Build the Page Table.
.LoopPageTable:
    mov [es:di], eax
    add eax, 0x1000
    add di, 8
    cmp eax, 0x200000                 ; If we did all 2MiB, end.
    jb .LoopPageTable

    pop di                            ; Restore DI.
    
    ; Disable IRQs
    mov al, 0xFF                      ; Out 0xFF to 0xA1 and 0x21 to disable all IRQs.
    out 0xA1, al
    out 0x21, al
    
    nop
    nop

    lidt [IDT]                        ; Load a zero length IDT so that any NMI causes a triple fault.

    ; Enter long mode.
    mov eax, 10100000b                ; Set the PAE and PGE bit.
    mov cr4, eax
      
    mov edx, edi                      ; Point CR3 at the PML4.
    mov cr3, edx
      
    mov ecx, 0xC0000080               ; Read from the EFER MSR. 
    rdmsr    

    or eax, 0x00000100                ; Set the LME bit.
    wrmsr
      
    mov ebx, cr0                      ; Activate long mode -
    or ebx,0x80000001                 ; - by enabling paging and protection simultaneously.
    mov cr0, ebx                    

    cli
    lgdt [GDT.Pointer]                ; Load GDT.Pointer defined below.
      
    jmp CODE_SEG:LongMode             ; Load CS with 64 bit segment and flush the instruction cache
      
      
    ; Global Descriptor Table
GDT:
.Null:
    dq 0x0000000000000000             ; Null Descriptor - should be present.

.Code:
    dq 0x00209A0000000000             ; 64-bit code descriptor (exec/read).
    dq 0x0000920000000000             ; 64-bit data descriptor (read/write).
      
ALIGN 4
    dw 0                              ; Padding to make the "address of the GDT" field aligned on a 4-byte boundary

.Pointer:
    dw $ - GDT - 1                    ; 16-bit Size (Limit) of GDT.
    dd GDT                            ; 32-bit Base Address of GDT. (CPU will zero extend to 64-bit)


[BITS 64]      
LongMode:
    mov ax, DATA_SEG
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    ; Blank out the screen to a blue color.
    mov edi, 0xB8000
    mov rcx, 500                      ; Since we are clearing uint64_t over here, we put the count as Count/4.
    mov rax, 0x1F201F201F201F20       ; Set the value to set the screen to: Blue background, white foreground, blank spaces.
    rep stosq                         ; Clear the entire screen. 
 
    ; Display "Hello World!"
    mov edi, 0x00b8000              
    
    mov rax, 0x1F6C1F6C1F651F48    
    mov [edi],rax
    
    mov rax, 0x1F6F1F571F201F6F
    mov [edi + 8], rax

    mov rax, 0x1F211F641F6C1F72
    mov [edi + 16], rax
    
    jmp Main.Long                     ; You should replace this jump to wherever you want to jump to.
</syntaxhighlight>

=== Notes about the above code ===
# The above code assumes that the caller has checked whether the CPU supports x86_64 or not.
# As noted in code comments, only 2MB is mapped. The code can be easily modified to map the kernel to higher half.
# The code doesn't include BPB for floppies, or any such thing. This also can be easily added.

== Example bootsector ==

The following example bootsector has been provided for completeness. It checks whether the CPU supports x86_64 or not. It then calls SwitchToLongMode to switch to long mode.

<syntaxhighlight lang="asm">
%define FREE_SPACE 0x9000

ORG 0x7C00
BITS 16

; Main entry point where BIOS leaves us.

Main:
    jmp 0x0000:.FlushCS               ; Some BIOS' may load us at 0x0000:0x7C00 while other may load us at 0x07C0:0x0000.
                                      ; Do a far jump to fix this issue, and reload CS to 0x0000.

.FlushCS:   
    xor ax, ax

    ; Set up segment registers.
    mov ss, ax
    ; Set up stack so that it starts below Main.
    mov sp, Main
    
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    cld

    call CheckCPU                     ; Check whether we support Long Mode or not.
    jc .NoLongMode

    ; Point edi to a free space bracket.
    mov edi, FREE_SPACE
    ; Switch to Long Mode.
    jmp SwitchToLongMode


BITS 64
.Long:
    hlt
    jmp .Long


BITS 16

.NoLongMode:
    mov si, NoLongMode
    call Print

.Die:
    hlt
    jmp .Die


%include "LongModeDirectly.asm"
BITS 16


NoLongMode db "ERROR: CPU does not support long mode.", 0x0A, 0x0D, 0


; Checks whether CPU supports long mode or not.

; Returns with carry set if CPU doesn't support long mode.

CheckCPU:
    ; Check whether CPUID is supported or not.
    pushfd                            ; Get flags in EAX register.
    
    pop eax
    mov ecx, eax  
    xor eax, 0x200000 
    push eax 
    popfd

    pushfd 
    pop eax
    xor eax, ecx
    shr eax, 21 
    and eax, 1                        ; Check whether bit 21 is set or not. If EAX now contains 0, CPUID isn't supported.
    push ecx
    popfd 

    test eax, eax
    jz .NoLongMode
    
    mov eax, 0x80000000   
    cpuid                 
    
    cmp eax, 0x80000001               ; Check whether extended function 0x80000001 is available are not.
    jb .NoLongMode                    ; If not, long mode not supported.

    mov eax, 0x80000001  
    cpuid                 
    test edx, 1 << 29                 ; Test if the LM-bit, is set or not.
    jz .NoLongMode                    ; If not Long mode not supported.

    ret

.NoLongMode:
    stc
    ret


; Prints out a message using the BIOS.

; es:si    Address of ASCIIZ string to print.

Print:
    pushad
.PrintLoop:
    lodsb                             ; Load the value at [@es:@si] in @al.
    test al, al                       ; If AL is the terminator character, stop printing.
    je .PrintDone                  	
    mov ah, 0x0E	
    int 0x10
    jmp .PrintLoop                    ; Loop till the null character not found.
	
.PrintDone:
    popad                             ; Pop all general purpose registers to save them.
    ret


; Pad out file.
times 510 - ($-$$) db 0
dw 0xAA55
</syntaxhighlight>

The above code can be compiled (and tested using QEMU) by the following:

<syntaxhighlight lang="bash">
    nasm -fbin Main.asm -o LongModeDirectly         # The main file's name should be Main.asm
    qemu-system-x86_64 -hda LongModeDirectly        # The secondary file's name should be LongModeDirectly.asm and should be in the same directory
</syntaxhighlight>

=== Notes about the above bootsector ===

* The above bootsector has been written just for completeness.
* The above bootsector can't be used as an actual bootsector as it doesn't:
# Enable [[A20]]
# Detect memory
# Load required files from disk using BIOS

==See Also==

===Forum Threads===

* [http://www.osdev.org/phpBB2/viewtopic.php?t=11093 Related forum thread]


[[Category:X86-64]]
````

<a id="w002323"></a>

## W002323 — Enter Long Mode Directly

Bron-ID: W002323
Namespace: Main (0)
Revisie: 5724 | Datum: 2008-07-01T01:24:19Z
Bron: https://wiki.osdev.org/Enter_Long_Mode_Directly
Revisie-URL: https://wiki.osdev.org/index.php?title=Enter_Long_Mode_Directly&oldid=5724
Doorverwijzing: Entering Long Mode Directly

````text
#REDIRECT [[Entering Long Mode Directly]]
````

<a id="w002324"></a>

## W002324 — Barebones

Bron-ID: W002324
Namespace: Main (0)
Revisie: 8226 | Datum: 2009-06-13T10:10:07Z
Bron: https://wiki.osdev.org/Barebones
Revisie-URL: https://wiki.osdev.org/index.php?title=Barebones&oldid=8226
Doorverwijzing: Bare Bones

````text
#REDIRECT [[Bare Bones]]
````

<a id="w002325"></a>

## W002325 — Template:SmallNavBox

Bron-ID: W002325
Namespace: Template (10)
Revisie: 29007 | Datum: 2024-06-06T17:30:42Z
Bron: https://wiki.osdev.org/Template:SmallNavBox
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ASmallNavBox&oldid=29007

````text
<table style="font-size:95%; line-height:1.5em; padding:0.25em; float:right; margin: 0 0 8px 15px; clear: right; border:1px solid #aaaaaa; background:#eee; text-align:center;{{{1|}}};"><!--

--><tr><th style="background:#ffce7b; padding:0.3em; font-size:1.1em;">{{{Name}}}</th></tr><!--

-->{{If|{{{List0|}}}|<tr><th>{{{Group0}}}</th></tr>|}}<!--
-->{{If|{{{List0|}}}|<tr><td>{{{List0}}}</td></tr>|}}<!--

--><tr><th>{{{Group1}}}</th></tr><!--
--><tr><td>{{{List1}}}</td></tr><!--

-->{{If|{{{List2|}}}|<tr><th>{{{Group2}}}</th></tr>|}}<!--
-->{{If|{{{List2|}}}|<tr><td>{{{List2}}}</td></tr>|}}<!--

-->{{If|{{{List3|}}}|<tr><th>{{{Group3}}}</th></tr>|}}<!--
-->{{If|{{{List3|}}}|<tr><td>{{{List3}}}</td></tr>|}}<!--

-->{{If|{{{List4|}}}|<tr><th>{{{Group4}}}</th></tr>|}}<!--
-->{{If|{{{List4|}}}|<tr><td>{{{List4}}}</td></tr>|}}<!--

-->{{If|{{{List5|}}}|<tr><th>{{{Group5}}}</th></tr>|}}<!--
-->{{If|{{{List5|}}}|<tr><td>{{{List5}}}</td></tr>|}}<!--

-->{{If|{{{List6|}}}|<tr><th>{{{Group6}}}</th></tr>|}}<!--
-->{{If|{{{List6|}}}|<tr><td>{{{List6}}}</td></tr>|}}<!--

-->{{If|{{{List7|}}}|<tr><th>{{{Group7}}}</th></tr>|}}<!--
-->{{If|{{{List7|}}}|<tr><td>{{{List7}}}</td></tr>|}}<!--

-->{{If|{{{List8|}}}|<tr><th>{{{Group8}}}</th></tr>|}}<!--
-->{{If|{{{List8|}}}|<tr><td>{{{List8}}}</td></tr>|}}<!--

-->{{If|{{{List9|}}}|<tr><th>{{{Group9}}}</th></tr>|}}<!--
-->{{If|{{{List9|}}}|<tr><td>{{{List9}}}</td></tr>|}}<!--

-->{{If|{{{List10|}}}|<tr><th>{{{Group10}}}</th></tr>|}}<!--
-->{{If|{{{List10|}}}|<tr><td>{{{List10}}}</td></tr>|}}<!--
--></table>
````

<a id="w002328"></a>

## W002328 — User talk:Virtlink

Bron-ID: W002328
Namespace: User talk (3)
Revisie: 5751 | Datum: 2008-07-02T16:15:54Z
Bron: https://wiki.osdev.org/User_talk:Virtlink
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AVirtlink&oldid=5751

````text
== Copyright ==
Please be careful when copying information to here. Copying large portions from other sites is illegal unless explicitly allowed. (In the current - wikipedia's - case, we can not satisfy the requirements that are needed for copying to be allowed) - [[User:Combuster|Combuster]] 16:04, 2 July 2008 (UTC)

::Okay, I'll just derive something using Wikipedia only as a general guideline.[[User:Virtlink|Virtlink]] 16:15, 2 July 2008 (UTC)
````

<a id="w002329"></a>

## W002329 — Template:Filesystems

Bron-ID: W002329
Namespace: Template (10)
Revisie: 28985 | Datum: 2024-06-06T17:23:37Z
Bron: https://wiki.osdev.org/Template:Filesystems
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AFilesystems&oldid=28985

````text
{{SmallNavBox
| width:200px;
|Name=[[File Systems|Filesystems]]
|Group0=Virtual Filesystems
|List0=<div>
[[VFS]]<br />
</div>
|Group1=Disk Filesystems
|List1=<div>
[[FAT]] [[FAT12|12]]/[[FAT16|16]]/[[FAT32|32]], [[VFAT]], [[ExFAT]]<br />
[[Ext2|Ext 2]]/[[Ext3|3]]/[[Ext4|4]]<br />
[[LEAN]]<br />
[[HPFS]]<br />
[[NTFS]]<br />
[[HFS]]<br />
[[HFS+]]<br />
[[MFS]]<br />
[[ReiserFS]]<br />
[[FFS (Amiga)]]<br />
[[FFS (BSD)]]/[[UFS]]<br />
[[BeFS]]<br />
[[BFS]]<br />
[[XFS]]<br />
[[SFS]]<br />
[[ZDSFS]]<br />
[[ZFS]]<br />
[[USTAR]]<br />
</div>
|Group2=CD/DVD Filesystems
|List2=<div>
[[ISO 9660]]<br />
[[Joliet]]<br />
[[UDF]]<br />
</div>

|Group3=[[:Category:Network Filesystems|Network Filesystems]]
|List3=<div>
[[NFS]]<br />
[[RFS]]<br />
[[AFS]]<br />
</div>

|Group4=Flash Filesystems
|List4=<div>
[[JFFS2]]<br />
[[YAFFS]]<br />
</div>
}}
````

<a id="w002330"></a>

## W002330 — FAT32

Bron-ID: W002330
Namespace: Main (0)
Revisie: 28334 | Datum: 2023-09-28T15:08:19Z
Bron: https://wiki.osdev.org/FAT32
Revisie-URL: https://wiki.osdev.org/index.php?title=FAT32&oldid=28334
Doorverwijzing: FAT

````text
#REDIRECT [[FAT#FAT_32]]
````

<a id="w002331"></a>

## W002331 — FAT12

Bron-ID: W002331
Namespace: Main (0)
Revisie: 28332 | Datum: 2023-09-28T15:07:39Z
Bron: https://wiki.osdev.org/FAT12
Revisie-URL: https://wiki.osdev.org/index.php?title=FAT12&oldid=28332
Doorverwijzing: FAT

````text
#REDIRECT [[FAT#FAT_12]]
````

<a id="w002332"></a>

## W002332 — ARM Overview

Bron-ID: W002332
Namespace: Main (0)
Revisie: 30546 | Datum: 2026-05-02T18:59:54Z
Bron: https://wiki.osdev.org/ARM_Overview
Revisie-URL: https://wiki.osdev.org/index.php?title=ARM_Overview&oldid=30546

````text
ARM is a family of instruction set architectures based on RISC architecture developed by a single company - ARM Holdings. 

Because ARM is a family of architectures and not a single architecture, it can be found in large scale of electronic devices (from simple small embedded systems with ARM MCUs, through smartphones, tablets and MP3 players to low-power servers).
When you look at a device with ARM processing unit (term processing unit is more accurate because ARM can be found in microcontrollers as well as in microprocessors), there are two things that matter: architecture and core. 

So far (Q3 2014) there have been 8 ARM architectures released or announced (where some of them have their extended versions), 7 of them being 32-bit, and the last one being 64-bit, but user-space compatibile with 32-bit instruction set (therefore making it possible to run 32-bit user processes, yet not 32-bit operating systems without virtualization). Very broadly said, with every new architecture version there have been added some new features to all cores (with exceptions) which had been already tried in some cores of a previous architecture. Not all features of previous architecture must be again available in the next one, and not all new versions of technologies added in previous architectures must be compatibile with the old ones. The simplest reason for this one could think of is that designing processors isn't like writing software: when a program decides whether to process old file format or a new one, it makes its decision and then exectues only one code, but  backward compatibility of (for example instruction) formats may mean more transistors and more transistors usually (always?) result in more heat. 

You must know you can't buy an ARM processor just like you would buy Intel abc or AMD xyz. ARM Holdings is a company that designs architecture, writes an '''Architecture Reference Manual''', then designs a core and writes a '''Technical Reference Manual'''. Finally, it sells designed core to a chip-making-company and releases manuals to public. From there, a chip-making-company designs a processing unit - a microcontroller (MCU), a System On a Chip (SoC), an FPGA, whatever - and it may manufacture the silicon itself or order n-thousands of chips from a semiconductor manufacturer. 
All of processors you'll see in tablets or smartphones are SoCs which act as sort of motherboard with a processor. They contain logic for driving peripherals (ethernet, USB, SD/MMC cards, SPI, I2C, audio), they may contain GPU for graphics coprocessing or FPGA for custom logic.

Many uses means either few multifunctional (and theferefore complex and power-consuming) devices, or many simple devices suited just for that kind of operation. ARM processors as RISC devices chose simplicity over complexity, and therefore they are way too many cores with different instructions used. To have just ''one assembler to rule them all'', ARM defined Unified Assembly Language which can be translated for '''any''' of ARM cores.

ARM cores are divided in lastest versions to three main lines:
* Cortex-M cores, used for really small devices, usually with on-chip memory and simpler operations
* Cortex-R cores, used for real-time devices
* Cortex-A cores, used for applications in multifunctional devices like smartphones, TVs or maybe computers.
Apple machines use custom ARM cores, and so do some Nvidia boards.

==Overview==
ARM processors operate in various modes. User, Fast Interrupt, Interrupt, Supervisor, Abort, and System. Each mode has its own stack and environment that it lives in.

While the ARM manuals distinguish between these modes, the demarcation is made not in their associated privilege levels, but their purposes. There are essentially two privilege levels in ARM: the manuals refer to them as either 'Privileged modes' or 'Non-privileged modes'.

Of the modes listed above, 'User' is the only non-privileged mode, while the others are collectively the privileged modes for the architecture.

Why are there so many modes? And what's the difference between the 'Interrupt' mode and the 'Fast Interrupt' mode? Will I have to write a special sort of abstraction which splits my kernel's interrupt handling away from the rest of its operation? No.

These modes are simply the state the processor is left in when a particular type of exception occurs, or a particular type of IRQ.

As copied from the ARM7TDI datasheet (3.6 Operating Modes):
{| class="wikitable"
! Mode 
! Description
|-
| User (usr) || Normal ARM execution state
|-
| FIQ (fiq) || Designed to support data transfer or channel process
|-
| IRQ (irq) || Used for general purpose interrupt handling
|-
| Supervisor (svc) || Protected mode for the operating system
|-
| Abort mode (abt) || Entered after data or instruction prefetch abort
|-
| System (sys) || A privileged user mode for the operating system
|-
| Undefined (und) || Entered when an undefined instruction is executed
|}

==Exceptions, IRQs and Software Interrupts on ARMv4 and up==

The ARMv4+ architectures define seven vectors for the architecture, contrasted with the x86's grand total of 256 vectors. Also, note well that the kernel does not have the choice of which interrupt vector to store the syscall entry to: ARM statically provides a 'SWI' instruction for software interrupts, and no 'INT N' instruction, so syscalls are statically fixed to a particular vector.

Now we get to go into the idea of the various privileged modes of the processor: When the SWI instruction is executed, the ARM processor vectors into the kernel provided vector table, and jumps to the hardware fixed vector which is set aside for the SWI instruction. The kernel is expected to install the syscall strapping code on this vector, since there is no other logical choice.

However, instead of, like x86, reading which privilege level or privilege settings to apply for a particular vector (I'm referring to the x86's GDT selector which is present in each IDT entry), the idea of modes are employed, such that for an SWI instruction, the ARM CPU will automatically enter the 'Supervisor' mode.

Please remember from now that the 'Supervisor' mode is the standard mode which the kernel is expected to operate from. 'System' mode is not switched to on any public vector; It is like, based on the dodgy way the manual refers to it, and the fact that none of the defined interrupts actually switch to 'System' mode, a #SMI (System Management Interrupt, which switched to System Management Mode) on the PC.

From 'System' mode, the kernel would process the SWI, then return to userspace.

Abort mode switched to by the processor on encountering the equivalent of a Page fault on x86. The processor switches to privileged mode, but the specific mode 'Abort', one of those listed above. Most ARM kernels just take note of the fact that the processor was in Abort mode on kernel enter, then switch to 'Supervisor' mode and service the exception.

Interrupt Mode and Fast Interrupt Mode are almost the same, except that FIQ mode is given its own set of registers (the registers are actually switched from those you refer to, although they still have the same names (r8, for example is still, in your assembly, referred to as r8; however, in FIQ mode, you must be aware that if you had information in r8 in usermode, you should not expect to find that same data under the register named r8 in FIQ mode), so that it can execute IRQ handlers without having to save context too much.

Essentially, like x86, you would have a piece of hardware which could be configured by the kernel. The kernel, knowing that it has only 7 vectors, and two of them are designated for IRQs only, would give out the IRQ number of one of the 2 IRQ vectors (IRQ or FIQ vectors). For a device configured to signal an IRQ on the IRQ vector, the processor, on receiving an interrupt signal on the IRQ vector will enter privileged mode, in the IRQ mode state.

For an interrupt received on the FIQ vector, the processor would enter privileged mode, but in the FIQ mode state.

The 'Undefined' Mode is switched to on the encounter by the CPU of an undefined exception. However, based on the tone used in the ARMv4 manual, and the fact that they blatantly imply this, the exception mode was really meant for the kernel to emulate instructions for the usermode process, and then return.

=== Note regarding the 'Spectre' exploit ===

ARM have recently added a new CPU instruction to their specification that mitigates cache speculation side-channel exploits. You can read more about this design patch and recommended action to take regarding Spectre (https://developer.arm.com/-/media/Files/pdf/Cache_Speculation_Side-channels.pdf?revision=8b5a5f33-c686-4b00-8186-187dd2910355 here).

{| class="wikitable"
! Page
! Description
|-
| http://www.embedded.com/design/prototyping-and-development/4006695/How-to-use-ARM-s-data-abort-exception || Good article on the data and instruction ABORT exceptions
|}

==Registers==
ARM processors have a somewhat large number of registers. The ARM7, for example, has 37 registers, 31 of those being 32-bit general registers, and 6 of those being status registers. Some are only usable by certain modes. 

Unlike the x86, important operating registers are clearly visible through general use registers. For example, r15 is 'pc', or the 'program counter', and r13 is the 'stack pointer', or 'sp'.

Along with the general purpose registers, there is also the CPSR register, or, the 'Current Program Status Register'. This registers keeps track of the current operating mode, whether interrupts are enabled or not, etc. The operating system can read and write to this register using the MSR\MRS instructions. ([http://www.arm.com/support/faqdev/1472.html See Here]) There are several other system registers which can be used to [[Detecting_Raspberry_Pi_Board|detect the ARM board]] for example.

{| class="wikitable"
|-
! Mode
! R0
! R1
! R2
! R3
! R4
! R5
! R6
! R7
! R8
! R9
! R10
! R11
! R12
! R13 (SP)
! R14 (LR)
! R15 (PC)
|-
! User32/System 
| R0 || R1 || R2 || R3 || R4 || R5 || R6 || R7 || R8 || R9 || R10 || R11 || R12 || R13 || R14 || R15
|-
! FIQ32
| R0 || R1 || R2 || R3 || R4 || R5 || R6 || R7 || R8FIQ || R9FIQ || R10FIQ || R11FIQ || R12FIQ || R13FIQ || R14FIQ || R15FIQ
|-
! Supervisor32
| R0 || R1 || R2 || R3 || R4 || R5 || R6 || R7 || R8 || R9 || R10 || R11 || R12 || R13SVC || R14SVC || R15SVC
|-
! Abort32 
| R0 || R1 || R2 || R3 || R4 || R5 || R6 || R7 || R8 || R9 || R10 || R11 || R12 || R13ABT || R14ABT || R15ABT
|-
! IRQ32
| R0 || R1 || R2 || R3 || R4 || R5 || R6 || R7 || R8 || R9 || R10 || R11 || R12 || R13IRQ || R14IRQ || R15IRQ
|-
! Undefined32
| R0 || R1 || R2 || R3 || R4 || R5 || R6 || R7 || R8 || R9 || R10 || R11 || R12 || R13UNDEF || R14UNDEF || R15UNDEF
|}

Each mode shares some registers with other modes. Normally registers are specified in ARM instructions they use 4 bits. Which can represent 16 registers. As you can see there are exactly 16 registers which you can reference using instructions in each mode. The mnemonic names are specified
across the top header as R0 through R15. An alias to R13, R14, and R15 is
specified in parenthesis which is SP, LR, and PC. So using the mnemonic R15
is the same as using the mnemonic PC, but keep in mind that this is only relevant to the assembler/compiler. The ARM processor only understands the value given using 4 bits in most instructions. The exactly value represented
using these 4 bits for each mnemonic can be gained by simply removing the prefixed letter R. So that R5 is represented by 4 bits as 0101, R6 as 0110, and R7 as 0111. The denotations that postfix ''IRQ'', ''UNDEF'', ''ABT'', ''SVC'', and ''FIQ'' are simply for display, and are relevant only to this table, and aren't considered valid by any assembler/compiler. For example ''R13ABT'', ''R13IRQ'', ''R13UNDEF'', ''R13SVC'', and ''R13FIQ'' all simply show that
internally the processor maps R13 (SP) to a different address in it's register file.

=== Calling Convention Cheat Sheets ===

Here is a quick overview of common calling conventions. Note that the calling conventions are usually more complex than represented here (for instance, how is a large struct returned? How about a struct that fits in two registers? How about va_list's?). Look up the specifications if you want to be certain. It may be useful to write a test function and use gcc -S to see how the compiler generates code, which may give a hint of how the calling convention specification should be interpreted.

{| {{wikitable}}
! Return Value
! Parameter Registers
! Additional Parameters
! Stack Alignment
! Scratch Registers
! Preserved Registers
! Return address
|-
| R0, R1 || R0, R1, R2, R3 || stack (R13) || 8 byte<sup>[[#Note1|1]]</sup> || R0, R1, R2, R3, R12 || R4, R5, R6, R7, R8, R9, R10, R11, R13 || R14
|}

<small id="Note1">Note 1: Stack is 8 byte aligned at all times outside of prologue/epilogue of non-leaf function. Leaf functions can have 4 byte aligned stacks. 8 byte alignment is required for ldrd/strd to function on the stack, which mostly only comes into play when using varargs with 64-bit integers. Care must be taken in interrupts and exceptions to align the stack to 8 byte before calling C code.</small>

For details look at the [http://infocenter.arm.com/help/topic/com.arm.doc.ihi0042e/IHI0042E_aapcs.pdf EABI specs].

==Instructions==
{| class="wikitable"
! Overview
|-
| Instruction length are constant fixed size. 32-bits A32 mode. 16-bits Thumb32 mode.
|-
| Strays from the RISC design by incorporating some multiple cycle instructions.
|-
| Employs inline conditional fields in instructions to reduce performance loss from pipeline flushes.
|-
| May produce larger code size due to RISC design compared to CISC designs, but supports Thumb32 mode which uses a 16-bit length instructions (more limited instruction set) to do 32-bit operations which produces smaller code footprint with reduced performance.
|-
| Most instruction execute in few clock cycles or single cycles making development of real-time software easier.
|- 
| A32 and Thumb16 modes can be switched on the fly (worthy to note!)
|-
|}

Loading of large immediate values into registers can be interestingly different from x86/x64. A immediate value being a value that is literally encoded into the instruction. For example the x86/x64 compatible processors support loading a 32-bit immediate (also called a constant) into an arbitrary register. The ARM's A32 and Thumb32 instruction sets do not. Instead one must use a series instructions to gain the same effect or store the value in memory outside of the instruction stream. In essence one may consider the storage of the value outside of the instruction stream the same as storage inside but the does exist a difference in many ways and this is something to keep in mind about the ARM. But, any compiler or assembler can and may take care of this problem for you. If you are compiling C/C++ code then these problems will be transparent to you for example, but as I said this is good information to know and understand especially with system software development.

Example, of machine code produced by GCC to load a register with a 32-bit value. As you can note the immediate value is technically outside of the instruction stream. On the x86/x64 the complete
value would have been encoded into the instruction. Also, take note how each instruction is constant in length. The instruction doing the work is LDR using the PC register and a immediate offset in order to place the value 0x12345678 into the register R3.
<pre>
00000000 <main>:
   0:	e1a0c00d 	mov	ip, sp
   4:	e92dd800 	push	{fp, ip, lr, pc}
   8:	e24cb004 	sub	fp, ip, #4
   c:	e59f300c 	ldr	r3, [pc, #12]	; 20 <main+0x20>
  10:	e1a00003 	mov	r0, r3
  14:	e24bd00c 	sub	sp, fp, #12
  18:	e89d6800 	ldm	sp, {fp, sp, lr}
  1c:	e12fff1e 	bx	lr
  20:	12345678 	.word	0x12345678
</pre>

An example of loading the register by storing the value in the instruction stream so to speak.
<pre>              
mov	r8, #0x78
add	r8, r8, #0x56 << 8
add	r8, r8, #0x34 << 16
add	r8, r8, #0x12 << 24
</pre>


Almost all instructions support conditional execution which is directly encoded into the instruction. On a x86/x64 architecture when a series of instructions need to be conditionally executed you will likely find a branching instruction which creates a completely separate code path for the processor to follow is the branch happens. The ARM supports branching, but also almost all individual instructions support conditional execution so that instead of causing the processor to potentially flush and refill the pipeline because a branch occurred instead it will load instructions into the pipeline which have a special conditional field. There are a maximum of 16 specified conditional codes with one reserved condition making 15 usable conditions. So instead of making a branch only to execute a few instructions those few instructions can remain in the execution path and will not be executed or will be executed simply using the conditional codes.

Here are the conditional codes for example:
{| class="wikitable"
|-
! Mnemonic
! Binary
! Description
|-
| EQ || 0000 || Z flag set (equal)
|-
| NE || 0001 || Z flag clear (not equal)
|-
| HS || 0010 || C flag set (unsigned higher or same)
|-
| LO || 0011 || C flag clear (unsigned lower)
|-
| MI || 0100 || N flag set (negative)
|-
| PL || 0101 || N flag clear (non-negative)
|-
| VS || 0110 || V flag set (overflow)
|-
| VC || 0111 || V flag clear (no overflow)
|-
| HI || 1000 || C flag set and Z flag clear
|-
| LS || 1001 || C flag clear or Z flag set
|-
| GE || 1010 || N flag set and V set or N clear and V clear
|-
| LT || 1011 || N set and V clear or N clear and V set
|-
| GT || 1100 || Z clear with (either N or V set), or N clear and V set
|-
| LE || 1101 || Z set or (N set and V clear), or N clear and V set
|-
| AL || 1110 || always (no condition really)
|-
| NV || 1111 || reserved condition
|}

==Memory==
Many ARM processors come equipped with [[MMU]]s, full memory protection schemes for their 4GB address space, and [[TLB]]. 

===== Paging =====
The paging scheme used by ARM processors uses a 2 level page table. The first level page table has 16kB size and each entry covers a region of 1MB (sections) and can be mapped directly or point to a second level page table. The second level page tables have 1kB size and each entry covers a region of 4KB (pages). The TLB also supports 16MB sections and 64kB pages. For those the right bits in the first or second level page table entries have to be set and the entry repeated 16 times. A hardware page table walk for any of the repeated entry then adds a TLB entry for the larger size. On ARMv4 and ARMv5 pages might also be divided into what the manual calls 'subpages', of, for a 4KB page, 1KB subpages, and for a 64KB page, 16KB subpages. (I assume you realize the fact that the subpages in both cases are 1/4 the size of the main page size). This is deprecated on ARMv6 and an alternate page table format can be configured that does not have subpages but has a extra access permission bits and provisions for physical address extentions (for more than 4GB physical memory) instead.

The ARM processor supports 2 independent page tables. The first page table can be abreviated from the full 16kB down to 128 byte in powers of 2 and is used for the lower part of the virtual address space. The second page table is always 16KB in size and is used for addresses beyond the end of the first page table. If the first page table already covers all of the virtual addres space, is already full size (16kB), then the second page table is never used and can be left unset. 

The Architecture Reference Manuals propose to use the abreviated first page table for processes individual address space and having the second level page table static for the kernel. For small processes (using 2GB or less) this results in a smaller page tables and therefore less overhead per process.

Note: The ARMv6 Architecture Reference Manuals state clearly, multiple times, that subpages are deprecated and should not be used. Subpages have been removed in the ARMv7 Architecture Reference Manuals and are no longer available. So for maximum portability, you may decide to stick to 4KB and 64KB page sizes.

Or you can have two sets of abstractions for your ARM Memory Manager port: One MM for ARMv4 and v5, and one for ARMv6 and up. It's all up to you.

===== Memory Detection =====
Memory detection is much different if you are coming from a background in the x86/x64 architecture. The ARM cores are used in many embedded applications and therefore the system board which the core resides on does not need to be overly complicated in order to be compatible with others boards. This is because almost any production board with an ARM core on it was likely custom designed just for that purpose. It is quite possible to use some generic board, but for lots of embedded applications there may not even be an operating system. 

Therefore memory detection mechanisms may be non-existent and instead your operating system may opt for a value to be encoded into it at compile for a specific system (board) which is known to have a certain amount of memory, or you may have a specific driver that your operating system loads just for that board (or is compiled in) where the driver can be queried by your kernel for the amount of memory installed. There are lots of ways and these were just two ideas, but the important part is to understand that unlike the x86/x64 compatible systems you will likely find absolutely no mechanisms to ''properly'' poll the amount of memory.

It may however be possible to probe memory and recover using processor exceptions. This still may not provide information about if a region of memory is FLASH, memory-mapped I/O, RAM, or ROM depending on how the system board was designed as I do suspect it could be quite possible for some ROM to be external to the core and allow writes to silently fail, and this coupled with the possibility of a region of memory to need a special unlock sequence in order to write to it will render your memory auto-detection code into a potential corner-case. 

But, if you ''think'' you can do it -- then by all means try because you might figure out a way. This section is just to give you a good general feeling of what you may be getting into and I do not mean to keep any good ideas you may have from becoming something valuable.

==Exceptions==
''Note: For some reason, the ARM people use the terms 'interrupt' and 'exception' as if they were the same.''

For exceptions, ARM uses a table similar to the IVT of the real mode x86. The table consists of a number of 32-bit entries. Each entry is an instruction (ARM instructions are 4 bytes in length.) that jumps to the appropriate handler.

Take note of that, and understand the design impact it imposes: On x86, the hardware vector table holds the addresses of handler routines. On ARM, the hardware vector table holds actual instructions. These instructions must fit into 4 bytes. This is actually not a big deal since all ARM instructions (assuming ARM mode and not Thumb, or Jazelle) are actually 4 bytes anyway. The general idea is to emulate the behaviour of something like the x86 and simply place a jump instruction into the actual vector table, so that upon indexing into the table, the ARM processor is made to act as if it jumped to an address contained in the jump instruction. From there, consistency is obtained, and portability is eased.

Also used are various devices to ''vector'' interrupts. Two such are the Generic Interrupt Controller and the Vectored Interrupt Controller.

==Coding Gotchas==
===== Heap Pointers Needs To Be Aligned =====
I can not state at this time how many processors support unaligned memory access natively, but from what I know unaligned memory access is more expensive than aligned. And, a good many structures which have fields greater than one byte in size a lot of times are allocated on the heap. So from this you can see how big of a performance hit you could take. Not to mention the bug that would be introduced if running under a processor which does not even handle unaligned memory access gracefully, and by that I meant at the very least raising an exception of some sort so the code can crash and show the problem.
===== Missing Division Functions (__aeabi_uidivmod, __aeabi_idiv)/No Division Support =====
This is caused by using GCC and not linking with ''libgcc''. You ''NEED TO'' link with ''libgcc'' when using GCC. For information about why you should link and information about ''libgcc'', read [[Libgcc]] and [[GCC_Cross-Compiler]].

You must also make sure you use the ''libgcc'' that has been compiled for your target machine/architecture/platform. See [[Libgcc]] for more information. You can produce the correct library for GCC by reading [[GCC_Cross-Compiler]].

====== This section is just if you want a more in depth explanation and discussion. ======
This issue is actually complicated because different CPUs may not support hardware division, support it only through the floating point operations, in thumb mode, in native mode, or a combination. Also, in certain situations you may find it faster or more compact to use different methods because you may be interested in code size or performance. The ''libgcc'' will handle all of these situations and provide the needed symbols, however here are some various sources of information that can give you a more in-depth understanding:
<pre>
The following links talk about different methods in handling this problem: 
    http://forum.osdev.org/viewtopic.php?f=1&t=23857&p=212244
    http://stackoverflow.com/questions/8348030/how-does-one-do-integer-signed-or-unsigned-division-on-arm
Also, some extra information that maybe useful:
    http://www.linkedin.com/groups/ARM-cores-hardware-division-85447.S.242517259
A discussion about this section, and also at the end an example of libgcc's version of the divide function:
    http://forum.osdev.org/viewtopic.php?f=8&t=27767
The source for libgcc and source of the division emulation function:
    https://github.com/mirrors/gcc/blob/master/libgcc/udivmodsi4.c
    https://github.com/mirrors/gcc/blob/master/libgcc/
</pre>

===== Unaligned Memory Access And Byte Order =====
{| border="1px"
| style="background-color: #eeeeee;" | Various newer ARM cores supposedly support unaligned memory access, but a specific bit or two have to be set and unset in the program control registers. I have not checked if QEMU's emulation of the ARMv7 supports this. So keep this in mind.
|}

Also, it is recommended to have a look at this article on [http://en.wikipedia.org/wiki/Endianess#Atomic_element_size_16-bit Endianness] which will help clarify and potentially gives a second source of information.

Let us imagine we have a board with 8 bytes of RAM, as depicted below:

{| class="wikitable"
! Value
| A || B || C || D || E || F || G || H
|-
! Address
| 0 || 1 || 2 || 3 || 4 || 5 || 6 || 7
|}

A ''little endian'' machine reads the least significant byte (LSB) first. So, if we made a 32-bit (word sized) read at address ''0'' on the memory above, that would yield "DCBA". On a ''big endian'' machine, which reads the most significant byte (MSB) first, you would get "ABCD". It all depends on how you visualize memory addresses - whether they flow to the right (big endian), or towards the left (little endian).

Some CPU platforms even support either mode. For example, the ARM7TDMI-S has a pin BIGEND to switch from little to big endian. (Although I could not find the QEMU option to enable this feature.)

ARM cores have two other differences from x86/x64:

* They do not handle unaligned memory accesses in any defined manner. It is specified on ''4-32 ARM7TDMI-S Data Sheet'' that if bit ''0'' for the memory address on a half-word access is ''high'' then the results are '''undefined'''.
* Accessing half-words in little endian mode will swap the half-words in the word.

An example for the latter:

<syntaxhighlight lang="c">
uint32        *a;
uint16        *b;

a = (uint32*)0;
b = (uint16*)0;

a[0] = 0x12345678;
printf("%#04x --> %#04x\n", b[0], b[1]);
</syntaxhighlight>

This gives ''0x5678 --> 0x1234'', while in big endian mode you would get ''0x1234 --> 0x5678''.

{| class="wikitable"
! colspan="8" | Little Endian Word Size (32-bit)
|-
| ED || CB || A9 || 87 || 78 || 9A || BC || DE
|- 
! colspan="4" | Offset 
! colspan="2" | Register(LittleEndian)
! colspan="2" | Register(BigEndian)
|-
! colspan="4" | 00 
| colspan="2" | 0x87A9CBED
| colspan="2" |	0xEDCBA987
|-
! colspan="4" | 01 
| colspan="4" | UNALIGNED - UNDEFINED
|- 
! colspan="4" | 10 
| colspan="4" | UNALIGNED - UNDEFINED
|-
! colspan="4" | 11 
| colspan="4" | UNALIGNED - UNDEFINED
|}

{| class="wikitable"
! colspan="8" | Little Endian Half-Word Size (16-bit)
|-
| ED || CB || A9 || 87 || 78 || 9A || BC || DE
|- 
! colspan="4" | Offset 
! colspan="2" | Register(LittleEndian) 
! colspan="2" | Register(BigEndian)
|-
! colspan="4" | 00 
| colspan="2" | 0xA987
| colspan="2" | 0xEDCB
|-
! colspan="4" | 01 
| colspan="4" | UNALIGNED - UNDEFINED
|- 
! colspan="4" | 10 
| colspan="2" | 0xEDCB 
| colspan="2" | 0xA987
|-
! colspan="4" | 11 
| colspan="4" | UNALIGNED - UNDEFINED
|}

{| class="wikitable"
! colspan="8" | Byte Size (8-bit)
|-
| ED || CB || A9 || 87 || 78 || 9A || BC || DE
|- 
! colspan="4" | Offset 
! colspan="2" | Register(LittleEndian) 
! colspan="2" | Register(BigEndian) 
|-
! colspan="4" | 00
| colspan="2" | 0xED 
| colspan="2" | 0xED
|-
! colspan="4" | 01 
| colspan="2" | 0xCB 
| colspan="2" | 0xCB
|- 
! colspan="4" | 10 
| colspan="2" | 0xA9 
| colspan="2" | 0xA9
|-
! colspan="4" | 11 
| colspan="2" | 0x87 
| colspan="2" | 0x87
|}

''From reading the data sheet it appears that if operating in big endian mode the half-word access would be reversed to be more natural as you would expect on the x86/x64 architecture. But, since I can not actually test it at the moment I am hoping I got it right.''

''The word access with offset of ''10b (0x2)'' may be defined, but I am not sure because it does not really state. However, it may employ some of the mechanisms for loading half-words. (Need someone to come through and correct this if it is wrong)''

The reason memory access has to be aligned is because unaligned access requires additional access cycles, due to the way memory modules work (see DDR datasheet below), and would increase the complexity of load / store operations (and the processor core).

You could simulate an unaligned memory access on the ARM7TDMI-S, but you would have to make separate loads from two memory locations. A compiler could probably emit code to do this automatically, but the checks whether an access is unaligned or not would slow down ''all'' memory accesses. Some code is provided in the ''ARM7TDMI-S Data Sheet'' on page ''4-35'' for such "checked" memory access,  when you do not know if the address will be aligned or non-aligned.

* http://lists.gnu.org/archive/html/qemu-devel/2004-12/msg00206.html - patch for QEMU to support big-endian ARM
* http://download.micron.com/pdf/datasheets/dram/ddr/256MBDDRx4x8x16.pdf Datasheet for DDR memory

===== Branch Instruction In Vector Table =====
Keep in mind when working with the ARMv4 and compatible cores that in the vector table each entry is 32-bits -- and that the value in each slot is ''not'' the address of where the CPU should jump too.

Instead it is an instruction. This can be a little confusing when you first start out because it seems like it makes sense for it to be an address, but instead it is an actual 32-bit instruction. It can be any instruction, but common usage is a ''LDR'' (data load) instruction which loads an immediate into PC from with-in 4K bytes of the instruction, or a ''B'' (branch) which uses a 26-bit signed offset.

Also, if you use a ''B'' instruction. Remember, its relative to the instruction pointer. So for each ''index'' in the table subtract ''index*4'' from it. Then subtract an extra ''8'' from that so you have ''index*4+8'' subtracted. This ''8'' is from the prefetch (pipeline having been filled). 

Failure to realize this can lead to some service routines that work long enough for you to think they are working then bug out days down the road and leave you looking through a lot of added code.
<syntaxhighlight lang="c">
#define ARM4_XRQ_RESET   0x00
#define ARM4_XRQ_UNDEF   0x01
#define ARM4_XRQ_SWINT   0x02
#define ARM4_XRQ_ABRTP   0x03
#define ARM4_XRQ_ABRTD   0x04
#define ARM4_XRQ_RESV1   0x05
#define ARM4_XRQ_IRQ     0x06
#define ARM4_XRQ_FIQ     0x07

/*
    Will install a branch instruction for the 
    interrupt vector for the ARM platform.
*/
void arm4_xrqinstall(uint32 ndx, void *addr)
{
    uint32      *v;
    
    v = (uint32*)0x0;
    v[ndx] = 0xEA000000 | (((uintptr)addr - 8 - (4 * ndx)) >> 2);
}
</syntaxhighlight>

''It can be a wise idea to populate all entries of the table. If the table is populated with zeros each vector will hold the instruction "andeq r0, r0, r0" which does nothing. Meaning if the CPU jumps to an unpopulated vector it will effectively execute a NOP and move to the next vector which can cause a confusing bug in your code. At least point any unused vectors to a dummy function that will notify you an unhandled exception has occured!''

===== Specifying CPSR using AS (Binutils/GCC) Syntax =====
You have to use ''cpsr'' not ''%%cpsr'' or any other form.

<syntaxhighlight lang="c">
uint32 arm4_cpsrget()
{
    uint32      r;
    
    asm("mrs %[ps], cpsr" : [ps]"=r" (r));
    return r;
}

void arm4_cpsrset(uint32 r)
{
    asm("msr cpsr, %[ps]" : : [ps]"r" (r));
}

/* Bit 7 and 6 have to be logic-low/unset/zero/cleared to enable the interrupt type. */
void arm4_xrqenable_fiq()
{
    arm4_cpsrset(arm4_cpsrget() & ~(1 << 6));
}

void arm4_xrqenable_irq()
{
    arm4_cpsrset(arm4_cpsrget() & ~(1 << 7));
}
</syntaxhighlight>

===== GCC (Chars Not Signed) =====
In some cases GCC when targeting ARM architecture may not handle signed values as you may expected.

Consider the case:
<syntaxhighlight lang="c">
void foo(char *a)
{
    *a = -1;
}

void bar()
{
    char      z;
    short     x;

    x = 0;
    foo(&z);

    x = x + z;
}
</syntaxhighlight>

The expected value for ''x'' would be ''-1'' or ''0xFFFF'' in a twos complement system, but instead you may 
end up surprised and spend quite a many hours when you get ''255'' or ''0xFF'' as the result. And, here is
the explanation.
<pre>
  4c:	ebfffffe 	bl	0 <foo>
  50:	e55b300f 	ldrb	r3, [fp, #-15]
  54:	e1a02003 	mov	r2, r3
  58:	e15b30be 	ldrh	r3, [fp, #-14]
  5c:	e0823003 	add	r3, r2, r3
</pre>
The x86/x64 target architecture will cause GCC to treat ''char'' as ''signed char'' [expected behavior by most], but when targeting the
ARM you may be surprised to find that ''char'' is treated as ''unsigned char''. You must specify ''signed''
before ''char'', and then the compiler will generate code to correct perform the addition.

This link explains better. Also, in case you decide to skip the reading. Essentially, code which assumes ''char'' should be
''signed char'' could be considered essentially incorrect and buggy which was stated by one poster. So, good idea to keep
a check on this for portability.
: [http://comments.gmane.org/gmane.linux.ports.arm.kernel/3930 Mailing List Post]

==Emulators==
*[http://bellard.org/qemu/qemu-doc.html#SEC63 QEMU]
*[http://softgun.sourceforge.net/ Softgun]
*[http://skyeye.sourceforge.net/index.shtml Skyeye]

== Targeting Multiple ARM Based Devices ==
For information about targeting multiple ARM based devices see, [[ARM_TargetingMultipleDevices|here]].

==Tutorials And Starting Points ==
{| class="wikitable"
|-
! Page
! Brief Description
|-
| [[ARM_Beagleboard|BeagleBoard]]
| Tutorial on bare-metal [OS] development on the Texas Instruments BeagleBoard. Written specifically for the BeagleBoard-xM Rev C.
|-
| [[ARM_Integrator-CP_Bare_Bones|Integrator Barebones]]
|  This is a tutorial. Read Getting Started and Beginner Mistakes. Should have reasonable knowledge of ARM assembly, but may be able to get by on less.
|-
| [[ARM_Integrator-CP_PL110_Dirty|Integrator-CP QEMU PL110 16-Bit Color Example]]
| Quick, dirty, and to the point on getting the QEMU PL110 16-Bit Color frame buffer working. Will likely not work on the real hardware, but will be very close. This is just to get someone started. Formed as a tutorial, but very short one.
|-
| [[PL050_PS/2_Controller|PL050 PS/2 Controller]]
| Information about interfacing a PS/2 device specifically the mouse, and starts with using the PL050. See links at bottom if PL050 is not of interest.
|-
| [[ARM_Integrator-CP_IRQTimerAndPIC|IRQ, Timer, And PIC]]
| This demonstrates using exceptions (specifically the IRQ exception), Timer, And PIC. It also provides a decent base for hacking with the ARM and/or QEMU.
|-
| [[ARM_Integrator-CP_ITPTMME_Main|ELK Pages (Thin ARM)]]
| The experimental learning kernel pages aimed to take someone gradually through the process of building a functional kernel using possibly (in later part of series) experimental designs and implementations that differ from the standard and conventional design in certain areas.
|-
| [[ARM_RaspberryPi]]
| Description and details on the commonly used Raspberry Pi boards
|-
| [[User:Pancakes/ARM_QEMU_REALVIEW-PB-A|QEMU realview-pb-a board]]
| This page gives you some information about programming for the realview-pb-a board under QEMU. Actual hardware ''could''be different. It also contains a link to the datasheet (which might be hard to find).
|}

== See Also ==
=== External Links ===
* [http://infocenter.arm.com/help/index.jsp ARM Infocenter]
* [http://www.arm.com/documentation/Software_Development_Tools/ More ARM Documentation]
* [http://www.coranac.com/tonc/text/asm.htm Whirlwind Tour of ARM Assembly]
* [http://re-eject.gbadev.org/files/GasARMRef.pdf GAS ARM Reference]
* [http://re-eject.gbadev.org/files/armref.pdf ARM Instruction Reference]
* [http://www.arm.com/miscPDFs/9658.pdf ARM Assembly Language Programming]
* [http://infocenter.arm.com/help/topic/com.arm.doc.dui0159b/DUI0159B_integratorcp_1_0_ug.pdf Integrator/CP Reference Manual]
* [http://www.amazon.com/ARM-System-Developers-Guide-Architecture/dp/1558608745 Amazon: ARM System Developer's Guide: Designing and Optimizing System Software]
* [http://www.embedded-bits.co.uk/2011/mmucode/ Turning on an ARM MMU and Living to tell the tale: The code]
* [https://developer.arm.com/community/arm-community-blogs/b/architectures-and-processors-blog/posts/using-the-stack-in-aarch32-and-aarch64 Using the Stack in AArch32 and AArch64]

[[Category:ARM]]
[[Category:Instruction Set Architecture]]
[[de:ARM]]
````

<a id="w002333"></a>

## W002333 — Category:ARM

Bron-ID: W002333
Namespace: Category (14)
Revisie: 5781 | Datum: 2008-07-02T20:17:04Z
Bron: https://wiki.osdev.org/Category:ARM
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AARM&oldid=5781

````text
[[Category:Platforms]]
````

<a id="w002334"></a>

## W002334 — Category:X86-64

Bron-ID: W002334
Namespace: Category (14)
Revisie: 5782 | Datum: 2008-07-02T20:19:13Z
Bron: https://wiki.osdev.org/Category:X86-64
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AX86-64&oldid=5782

````text
[[Category:X86]]
[[Category:Platforms]]
````

<a id="w002335"></a>

## W002335 — CIL

Bron-ID: W002335
Namespace: Main (0)
Revisie: 24894 | Datum: 2020-06-22T13:59:50Z
Bron: https://wiki.osdev.org/CIL
Revisie-URL: https://wiki.osdev.org/index.php?title=CIL&oldid=24894

````text
{{stub}}

The '''Common Intermediate Language''' ('''CIL''', formerly '''MSIL''') is a byte code intermediate language used in the [[.NET Framework]]. Code which compiles to CLI byte code is considered 'managed code' and can be created by any [[CLI]] compatible compiler (such as the [[C Sharp|C# Compiler]], and executed by the [[Virtual Execution System]] (VES). It is CPU and platform independent.

Most of the time, developers will not be programming directly in CIL but use a higher level language such as [[C Sharp|C#]], [[C++/CLI]], [[Visual Basic .NET]] and [[J#]]. However, CIL has its own assembly language which can be assembled by ILAsm (<code>ilasm.exe</code>). To display the byte code in assembly language, the CIL disassembler ILDAsm (<code>ildasm.exe</code>) is used. However, some specialized tools such as .NET Reflector can represent the byte code as any of the supported higher level languages, such as C#, including the classes, methods and other metadata.

==Safe and unsafe code==
CIL byte code can either be considered safe or unsafe. By default, all C# is safe, which means that the execution system knows where all the data is, all the time. Therefore it is not possible to use pointers in safe code. Thanks to this property, the most common security and stability problems are eliminated, such as buffer overflows and dangling pointers. A [[Garbage collection|garbage collector]] is employed to use the memory as efficient as possible.

===Unsafe code===
Although pointers are not possible in safe code, CIL supports unsafe code which does have support for pointers. Unsafe code is contained in blocks of code which is marked 'unsafe', and in C# it uses the same syntax as C and C++. Objects can be ''pinned'' so that the [[Garbage collection|garbage collector]] doesn't move the data around and pointers remain valid. Using unsafe code is perfectly legal, although discouraged in normal application programming, and may be used to write kernel-level methods.

==Executing CIL Code==
CIL code is executed by a [[Virtual Execution System]] much like Java byte code. Usually this is in the form of a [[Just In Time compiler]], which translates the byte code into a native machine code, usually utilizing optimizations specific to that processor (for example SSE if available). [http://www.mono-project.com Mono] is an open-source .NET project, which consists of an CIL JIT'er (and interpreter for unsupported architectures) as well as a C#->CLI compiler. The JIT'er and can be [http://www.mono-project.com/Embedding_Mono embedded] in to any program or stand alone. The basic JIT'er and compiler is written in C and only requires the C Standard Library to run.

==Metadata==
The CIL byte code is only half of an .NET assembly. The other half is metadata, which is used by the VES to find which method needs to be called. This makes it possible to inherit or extend classes and override methods while the correct overridden method will be called at run-time without recompiling the calling assembly.
````

<a id="w002336"></a>

## W002336 — Template:File formats

Bron-ID: W002336
Namespace: Template (10)
Revisie: 28181 | Datum: 2023-07-10T16:53:03Z
Bron: https://wiki.osdev.org/Template:File_formats
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AFile_formats&oldid=28181

````text
{{SmallNavBox
| width:200px;
|Name=[[Executable Formats]]

|Group1=Microsoft
|List1=
<div>
'''16 bit:'''<br />
[[COM]]<br />
[[MZ]]<br />
[[NE]]<br />
'''Mixed (16/32 bit):'''<br />
[[LE]]<br />
'''32/64 bit:'''<br />
[[PE]]<br />
[[COFF]]<br />
</div>

|Group2=*nix
|List2=
<div>
[[A.out|a.out]]<br />
[[ELF]]<br />
</div>

|Group3=Apple
|List3=
<div>
[[Mach-O]]<br />
</div>
}}
````

<a id="w002337"></a>

## W002337 — FAT16

Bron-ID: W002337
Namespace: Main (0)
Revisie: 28333 | Datum: 2023-09-28T15:08:09Z
Bron: https://wiki.osdev.org/FAT16
Revisie-URL: https://wiki.osdev.org/index.php?title=FAT16&oldid=28333
Doorverwijzing: FAT

````text
#REDIRECT [[FAT#FAT_16]]
````

<a id="w002338"></a>

## W002338 — Category talk:X86-64

Bron-ID: W002338
Namespace: Category talk (15)
Revisie: 5823 | Datum: 2008-07-05T22:24:18Z
Bron: https://wiki.osdev.org/Category_talk:X86-64
Revisie-URL: https://wiki.osdev.org/index.php?title=Category_talk%3AX86-64&oldid=5823

````text
==Is This Section Needed?==
Instead of having a Category:X86-64 and a Category:X86, wouldn't a section like Category:Intel fit better with things like Category:Sparc and Category:MIPS? Then, Category:X86 and Category:X86-64 could be subcategories with their respective articles, and commons simply in Category:Intel. I only recommend this because atm we have articles like [[x86-64]] in Category:x86, but not in Category:x86-64. --[[User:Alboin|Alboin]] 22:24, 5 July 2008 (UTC)
````

<a id="w002339"></a>

## W002339 — Template talk:SmallNavBox

Bron-ID: W002339
Namespace: Template talk (11)
Revisie: 5824 | Datum: 2008-07-06T18:34:53Z
Bron: https://wiki.osdev.org/Template_talk:SmallNavBox
Revisie-URL: https://wiki.osdev.org/index.php?title=Template_talk%3ASmallNavBox&oldid=5824

````text
This template is used to categorize anything ranging from file systems to executable formats. But I can't get the group headers (Group1, Group2) to get visible. Anyone who can help me fix it? [[User:Virtlink|Virtlink]] 18:34, 6 July 2008 (UTC)
````

<a id="w002340"></a>

## W002340 — User:Omega

Bron-ID: W002340
Namespace: User (2)
Revisie: 5853 | Datum: 2008-07-08T09:17:11Z
Bron: https://wiki.osdev.org/User:Omega
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AOmega&oldid=5853

````text
<DIV STYLE='TEXT-ALIGN:JUSTIFY;'>
===Introduction===
----
Welcome. So you think you are ready to start programming your floppy driver. Well, before we begin let's take a quick glance at the following few paragraphs to make sure that this is the type of driver that you actually can and want to support in your Operating System.

===Prerequisites===
<ul>
<li>01. Have you enabled Protected Mode?</li>
<li>02. Have you installed your new IDT?</li>
<li>03. Have you installed your new GDT?</li>
<li>04. Have you installed your new ISR?</li>
<li>05. Have you installed your new IRQ?</li>
<li>06. Have you installed your PIT yet?</li>
</ul>

===Mandatory Tasks===
<ul>
<li>01. Create C Header FDC.H</li>
<li>02. Create C Header FAT12.H</li>
<li>03. Create C File FDC.C</li>
<li>04. Create C File FAT12.C</li>
<li>04. Create Custom OUTB Function</li>
<li>04. Create Custom INB Function</li>
<li>04. Create Custom PRINTF Function</li>
</ul>

===Quick Summary===
This project will produce by the end of this guide one 32 bit Protected Mode Floppy Driver which utilizes the DMA for   disc IO functionality. This code assumes two main things: 01) You want to support a 32 bit Pmode Floppy Disk Driver. 02) You are compiling your C source using GCC or DJGPP.

===Final Warning===
The Floppy Drive is well on its way out and at some point soon no one will have one. Writing a Floppy Driver is quite obsolete to the rest of the world, so doing so must be done for your pure enjoyment. You should not design your OS to rely upon a floppy disk. However, the FAT12 and FAT32 is a lot a like, so it will be super easy to port this to FAT32. <FONT style='color:RED;font-weight:bold;'>This program is distributed freely in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. PLEASE USE THIS CODE AT YOUR OWN RISK!!</FONT>

----

----

===HEADERS::FDC.H===
------
Here we begin by defining our FAT12 Boot sector Structure and Helper Functions.
<pre>
#ifndef FDC_H
#define FDC_H

struct floppyDisk
{
    short bytesPerSector;
    char sectorsPerCluster;
    short reservedSectors;
    char numOfFats;
    short maxRootEntries;
    short totalSectors;
    short sectorsPerFat;
    short sectorsPerTrack;
    short numberOfHeads;
    char bootSig;
    int volumeId;
    char volumeLabel[12];
};

void fdc_reset();
void get_floppy_type();
void init_floppy();
void floppy_ident();
void start_motor();
void stop_motor();
void wait_irq6();
void reset_floppy_controller();
void floppy_read_sectors(unsigned char *buf, int pos, int count);
void floppy_write_sector(unsigned char *buf, int pos, int read);
void fdc_handler();

#endif /* FDC_H */
</pre>

===HEADERS::FAT12.H===
-----
Here we define our FAT12/32 Structures and Helper Function.
<pre>
#ifndef FAT12_H
#define FAT12_H

void get_bootsector();

typedef unsigned char	uint8_t;	/* or #include <stdint.h> */
typedef unsigned short	uint16_t;	/* Note: multi-byte values are little-endian */
typedef unsigned long	uint32_t;   /* 32 bytes total */

struct fat_bootsector			/* Warning: this struct must be packed */
{
	uint8_t		jump[3];               /* 16-bit JMP to boot code, or 8-bit JMP + NOP */
	uint8_t		oem_id[8];             /* e.g. 'MSWIN4.0' */
	uint16_t	bytes_per_sector;	/* usu. =512 */
	uint8_t		sectors_per_cluster;
	uint16_t	num_boot_sectors;	/* usu. =1 */
	uint8_t		num_fats;              /* usu. =2 */
	uint16_t	num_root_dir_ents;
	uint16_t	total_sectors;         /* 16-bit; 0 if num sectors > 65535 */
	uint8_t 	media_ID_byte;         /* usu. =0F0h */
	uint16_t	sectors_per_fat;
	uint16_t	sectors_per_track;
	uint16_t	heads;
	uint32_t	hidden_sectors;	/* =LBA partition start */
	uint32_t	total_sectors_large;   /* 32-bit; 0 if num sectors < 65536 */
	uint8_t		boot_code[474];
	uint8_t		magic[2];              /* 55h, 0AAh */
};                              /* 512 bytes total */

struct fat_dirent               	/* Warning: this struct must be packed */
{
	uint8_t  name[8];               /* ALL-CAPS, pad right with spaces */
	uint8_t  ext[3];                /* ALL-CAPS, pad right with spaces */
	uint8_t  attrib;                /* attribute byte */
	uint8_t  reserved;              /* =0 */
	uint8_t  ctime_ms;              /* file creation time, 10ms units */
	uint16_t ctime;              	/* file creation time, in DOS format */
	uint16_t cdate;              	/* file creation date, in DOS format */
	uint16_t adate;              	/* DOS date of last file access */
	uint16_t st_clust_msw;       	/* high 16 bits of starting cluster (FAT32) */
	uint16_t mtime;              	/* DOS time of last file modification */
	uint16_t mdate;              	/* DOS date of last file modification */
	uint16_t st_clust;           	/* starting cluster */
	uint32_t file_size;          	/* in bytes */
};

struct fat_direntry
{
   unsigned char name[11];
   unsigned char attr;
   char _reserved[10];
   unsigned short time;
   unsigned short date;
   unsigned short cluster;
   unsigned size;
} fat_direntry_t;

struct attrib                   /* Warning: this struct must be packed */
{
	int read_only : 1;      /* b0 */
	int hidden : 1;
	int system : 1;
	int volume_label : 1;
	int directory : 1;
	int archive : 1;
	int reserved : 2;       /* b6, b7 */
};                              /* 1 byte total */

struct dos_time                 /* Warning: this struct must be packed */
{
	unsigned two_secs : 5;  /* low 5 bits: 2-second increments */
	unsigned minutes : 6;   /* middle 6 bits: minutes */
	unsigned hours : 5;     /* high 5 bits: hours (0-23) */
};                              /* 2 bytes total */

struct dos_date                 /* Warning: this struct must be packed */
{
	unsigned date : 5;      /* low 5 bits: date (1-31) */
	unsigned month : 4;     /* middle 4 bits: month (1-12) */
	unsigned year : 7;      /* high 7 bits: year - 1980 */
};

struct FLOPPY_GEOMETRY
{
	int sectors;
	int heads;
	int track;
};

typedef struct fdisk
{
    char *name;
    char floppy_type;
    short base;
    int sr0;
    int motor_on;
    int track;
    int locked;
    struct FLOPPY_GEOMETRY * geometry;
} fdisk_t;

#endif /* FAT12_H */
</pre>

You might notice at some point that none of the FAT32 structures are being used. This is because we have not yet completed that task. For now, I am going to leave this one up to you as the task of doing so at this point is relatively straightforward.

===CFILE::FDC.C===
-----
Here is our brand new Floppy Drive 32 bit Pmode Driver!
<pre>
#include <fdc.h>

#define DISK_PARAMETER_ADDRESS 0x000FEFC7
#define error 1

int first;
int second;
int current_track = -1;
short base_used;
volatile int irq6_state = 0;
int motor_on = 0;
int sr0 = 0;

char *floppy_types[] =
{
    "None.",
    "360kB 5.25: Unsupported.",
    "1.2MB 5.25: Unsupported.",
    "720KB 3.5: Unsupported.",
    "1.44MB 3.5.",
    "2.88MB 3.5: Unsupported."
};

// put in header later
#define LOW_BYTE(x)         (x & 0x00FF)
#define HI_BYTE(x)          ((x & 0xFF00) >> 8)

// the folowing is a simple look up table for channel and its ports
unsigned char maskport[8] = { 0x0A, 0x0A, 0x0A, 0x0A, 0xD4, 0xD4, 0xD4, 0xD4 };
unsigned char modeport[8] = { 0x0B, 0x0B, 0x0B, 0x0B, 0xD6, 0xD6, 0xD6, 0xD6 };
unsigned char clearport[8] = { 0x0C, 0x0C, 0x0C, 0x0C, 0xD8, 0xD8, 0xD8, 0xD8 };
unsigned char pageport[8] = { 0x87, 0x83, 0x81, 0x82, 0x8F, 0x8B, 0x89, 0x8A };
unsigned char addrport[8] = { 0x00, 0x02, 0x04, 0x06, 0xC0, 0xC4, 0xC8, 0xCC };
unsigned char countport[8] = { 0x01, 0x03, 0x05, 0x07, 0xC2, 0xC6, 0xCA, 0xCE };

// dma xfer for floppy
void set_dma(unsigned char *buff, short size, int read, int channel)
{
    char page = 0;
    char mode = 0;
    short offset = 0;
    if(read)
		mode = 0x48 + (char)channel;
	else
		mode = 0x44 + (char)channel;
	page = (char)((int)buff >> 16);
	offset = (short)((int)buff & 0xFFFF);
	size--;
	cli();
	outb(maskport[channel], (0x04 | channel)); // mask channel
	outb(clearport[channel], 0x00);                // stop trans
	outb(modeport[channel], mode);                 // type of trans
	outb(addrport[channel], LOW_BYTE(offset));     // send address
    outb(addrport[channel], HI_BYTE(offset));      // send address
    outb(pageport[channel], page);                 // send page
    outb(countport[channel], LOW_BYTE(size));      // send size
    outb(countport[channel], HI_BYTE(size));       // send size
    outb(maskport[channel], channel);              // unmask channel
    sti();
}

void get_floppy_type()
{
    unsigned char data;
    outb(0x70, 0x10);
    data = inb(0x71);
    first = data >> 4;
    second = data & 0xF;
}

void fdc_handler()
{
    irq6_state = 1;

}

void wait_irq6()
{
    while(irq6_state == 0);
    irq6_state = 0;
}

void start_motor()
{
    outb((base_used + 0x2), 0x1C);
    //delay(1);
    motor_on = 1;
}

void stop_motor()
{
    outb((base_used + 0x2), 0x00);
    //delay(1);
    motor_on = 0;
}

/* from intel manual */
void sendbyte(char byte)
{
    volatile int msr;
    int tmo;
    for(tmo = 0; tmo < 128; tmo++)
    {
        msr = inb((base_used + 0x04));
        if ((msr & 0xC0) == 0x80)
        {
            outb((base_used + 0x05), byte);
            return;
        }
        inb(0x80);
    }
}

/* from intel manual */
int getbyte()
{
    volatile int msr;
    int tmo;
    for (tmo = 0; tmo < 128; tmo++)
    {
        msr = inb((base_used + 0x04));
        if ((msr & 0xd0) == 0xd0)
	        return inb((base_used + 0x05));
        inb(0x80);
    }
    return -1;
}

void waitfdc()
{

    wait_irq6();
		//printf(0x0F,1,1,"Now Recalibrate(1a)");
    sendbyte(0x08);
		//printf(0x0F,1,1,"Now Recalibrate(1b)");
    sr0 = getbyte();
		//printf(0x0F,1,1,"Now Recalibrate(1c)");
    (void)getbyte();
		//printf(0x0F,1,1,"Now Recalibrate(1d)");
}

int fdc_seek(char track)
{
    if(current_track == track)
        return 0;
    sendbyte(0x0F);
    sendbyte(0x00);
    sendbyte(track);
    //delay(1);
    waitfdc();
    if(sr0 != 0x20)
    {
        printf(0x0F,1,1,"'nTrack seek error");
        return error;
    }
    current_track = track;
    return 0;
}

void fdc_recalibrate()
{
    start_motor();
    sendbyte(0x07);
    sendbyte(0x00);
    waitfdc();
    stop_motor();
}

void fdc_reset()
{
   outb((base_used + 0x02), 0);    // stop everything
   motor_on = 0;
   outb((base_used + 0x04), 0);     // data rate (500K/s)
   outb((base_used + 0x02), 0x0C);  // restart ints
   //delay(1);
		//printf(0x0F,1,1,"Now Recalibrate(1)");
   waitfdc();
//   delay(500);
		//printf(0x0F,1,1,"Now Recalibrate(2)");
   sendbyte(0x03);                     // timing
   sendbyte(0xDF);                     // timing
   sendbyte(0x02);                     // timing
   //delay(1);
	//printf(0x0F,1,1,"Now Recalibrate()");
   while(fdc_seek(1) == error);        // set track
   fdc_recalibrate();
}

void init_floppy()
{
    base_used = 0x3F0;
    get_floppy_type();

}

void floppy_ident()
{
    printf(0x0F,1,1,"'tF0: %s",floppy_types[first]);
    printf(0x0F,1,2,"'tF1: %s",floppy_types[second]);
}

void floppy_read_sectors(unsigned char *buf, int pos, int count)
{
    int sec, cyl, head, max_pos = pos + count;
    start_motor();
    for(; pos < max_pos; pos++)
    {
        sec = (pos % 18) + 1;
        cyl = pos / 36;
        head = (pos / 18) % 2;
        outb((base_used + 0x7), 0);
        while(fdc_seek((char)cyl) == error);
        set_dma(0x00000000, 512, 0, 2);
        //delay(1);
//        if(inb(base_used + 0x4) != 0x80)
//        {
//            puts("The DMA bit wasn't set'n");
////            stop_motor();
//            return;
//        }
        sendbyte(0xE6);
        sendbyte((unsigned char)head << 2);
        sendbyte((unsigned char)cyl);
        sendbyte((unsigned char)head);
        sendbyte((unsigned char)sec);
        sendbyte(0x02);
        sendbyte(0x12);
        sendbyte(0x1B);
        sendbyte(0xFF);
        wait_irq6();
        (void)getbyte();
        (void)getbyte();
        (void)getbyte();
        (void)getbyte();
        (void)getbyte();
        (void)getbyte();
        (void)getbyte();
        memcpy((unsigned int *)buf, (unsigned int *)0x00000000, 128);
        buf += 0x200;
    }
    stop_motor();
}

void floppy_write_sector(unsigned char *buf, int pos, int read)
{
    int sec, cyl, head, stat = error;
    sec = (pos % 18) + 1;
    cyl = (pos / 18) / 2;
    head = cyl % 2;
    start_motor();
    outb((base_used + 0x7), 0);
    while(stat == error)
        stat = fdc_seek((char)cyl);
    set_dma(buf, 512, 1, 2);
    if(inb(base_used + 0x4) != 0x80)
    {
        printf(0x0F,1,1,"'nThe DMA bit wasn't set");
        return;
    }
    sendbyte(0xC5);
    sendbyte((unsigned char)head << 2);
    sendbyte((unsigned char)cyl);
    sendbyte((unsigned char)head);
    sendbyte((unsigned char)sec);
    sendbyte(0x02);
    sendbyte(0x12);
    sendbyte(0x1B);
    sendbyte(0xFF);
    wait_irq6();
    (void)getbyte();
    (void)getbyte();
    (void)getbyte();
    (void)getbyte();
    (void)getbyte();
    (void)getbyte();
    (void)getbyte();
    stop_motor();
}

void fdc_install(void)
{
	outb(0x20, 0x20);
	irq_install_handler(6,fdc_handler);
}
</pre>

===CFILE::FAT12.C===
------
Here is our gateway to FDD interaction. We can use this portion to determine specifics about the drive in order to swap out different drivers for different FAT types or we could also load an EXE/BIN file right off our disk and execute it! For now, we only print the values to the screen.
<pre>
#include <fdc.h>

struct floppyDisk floppy1;  //defined in FDC.H

void get_bootsector()  //defined in FAT12.H
{
    char bootsector[512];
    floppy_read_sectors(bootsector, 0, 1);
    floppy1.bytesPerSector = (short)bootsector[11];
    floppy1.bytesPerSector |= (short)bootsector[12] << 8;
    floppy1.sectorsPerCluster = bootsector[13];
    floppy1.reservedSectors = (short)bootsector[14];
    floppy1.reservedSectors |= (short)bootsector[15] << 8;
    floppy1.numOfFats = bootsector[16];
    floppy1.maxRootEntries = (short)bootsector[17];
    floppy1.maxRootEntries |= (short)bootsector[18] << 8;
    floppy1.totalSectors = (short)bootsector[19];
    floppy1.totalSectors |= (short)bootsector[20] << 8;
    floppy1.sectorsPerFat = (short)bootsector[22];
    floppy1.sectorsPerFat |= (short)bootsector[23] << 8;
    floppy1.sectorsPerTrack = (short)bootsector[24];
    floppy1.sectorsPerTrack |= (short)bootsector[25] << 8;
    floppy1.numberOfHeads = (short)bootsector[26];
    floppy1.numberOfHeads |= (short)bootsector[27] << 8;
    floppy1.bootSig = bootsector[38];
    floppy1.volumeId = (int)bootsector[39];
    floppy1.volumeId |= (int)bootsector[40] << 8;
    floppy1.volumeId |= (int)bootsector[41] << 16;
    floppy1.volumeId |= (int)bootsector[42] << 24;
    floppy1.volumeLabel[0] = bootsector[43];
    floppy1.volumeLabel[1] = bootsector[44];
    floppy1.volumeLabel[2] = bootsector[45];
    floppy1.volumeLabel[3] = bootsector[46];
    floppy1.volumeLabel[4] = bootsector[47];
    floppy1.volumeLabel[5] = bootsector[48];
    floppy1.volumeLabel[6] = bootsector[49];
    floppy1.volumeLabel[7] = bootsector[50];
    floppy1.volumeLabel[8] = bootsector[51];
    floppy1.volumeLabel[9] = bootsector[52];
    floppy1.volumeLabel[10] = bootsector[53];
    floppy1.volumeLabel[11] = '\0';

    //print this stuff for now
    printf(0x0F,1,1,"FDD Info");
    printf(0x0F,1,2,"Bytes per Sector: %d",floppy1.bytesPerSector);
    printf(0x0F,1,3,"Sectors per Cluster: %d",floppy1.sectorsPerCluster);
    printf(0x0F,1,4,"Reserved Sectors: %d",floppy1.reservedSectors);
    printf(0x0F,1,5,"Num of Fats: %d",floppy1.numOfFats);
    printf(0x0F,1,6,"Max Root Entries: %d",floppy1.maxRootEntries);
    printf(0x0F,1,7,"Total Sectors: %d",floppy1.totalSectors);
    printf(0x0F,1,8,"Sectors per FAT: %d",floppy1.sectorsPerFat);
    printf(0x0F,1,9,"Sectors per Track: %d",floppy1.sectorsPerTrack);
    printf(0x0F,1,10,"Number of Heads: %d",floppy1.numberOfHeads);
    printf(0x0F,1,11,"Boot Signature: %d",floppy1.bootSig);
    printf(0x0F,1,12,"Volume ID: %d",floppy1.volumeId);
    printf(0x0F,1,13,"Volume Label: %s",floppy1.volumeLabel);
}

void readRoot() 
{
// develop this to find the Root Directory
}
</pre>

===Kernel Initialization Code===
------
Alright, as long as you have successfully created all of these files and have fulfilled the prerequisites, then you are officially ready to get the ball rolling in your kernel. Place this code below in your main kernel function, for example.
<pre>
#include <fdc.h>
#include <fat12.h>

int main() 
{
  sti();                          //Turn on interrupts
  pit();                          //Start your timer (not included)
  fdc_install();                  //Install IRQ Handler
  init_floppy();                  //Initialize Floppy
  fdc_reset();                    //Reset Motor/Recalibrate
  //floppy_ident();               //Could be used to load drivers for different formats, I comment it out for now
  char mbuff[512];
  floppy_read_sectors(mbuff,0,1); //read the floppy boot sector (512 bytes)
  get_bootsector();               //fill our structure with the boot sector values
  for(;;);                        //idle
  return 0;
}
</pre>

===PIT Wait Function===
------
This code is located in my PIT.C source file which is not included here. You will need this code to perform your wait functions.
<pre>
void timer_wait(int ticks)
{
    int eticks;
    eticks = timer_ticks + ticks;

	while (timer_ticks < eticks);
	//schedule();
}
</pre>
The schedule() function simply handles tasks like a  good driver should, but this isn't a good driver and I have that commented out for now; it will be good soon.

===Quick Run Down===
------
Real quick this code works like this:
<ul>
<li>01. We just got back from our loader and interrupts are turned on.</li>
<li>02. The timer (PIT) is enabled by mapping it to the IRQ 0 ISR.</li>
<li>03. The floppy driver is enabled by mapping it to the IRQ 6 ISR.</li>
<li>04. We then reset the motor and recalibrate the drive.</li>
<li>05. Then we issue a floppy_read_sectors command to read only the first 512 bytes (bootsector) of floppy disk.</li>
<li>06. As that is being done we are also filling our struct and printing the contents to screen.</li>
<li>07. Done and Halt.</li>
</ul>

===In Summation===
------
At this point you should be well on your way to making this Floppy Driver happen for your OS. The next item on your list could be building a Virtual File System, Paging/Heap, Multi-Tasking, Scheduling/Tasking, FAT32 or NTFS support, maybe take some time off to work on your console interface, terminal, etc or just take a break from learning for a while because you have certainly come a long way if you are here now. Whatever you choose, make sure that you are taking the time to truly understand what you are learning and are truly capable of putting that knowledge to practical use before you move on to your next project. Most of all, have fun, good luck, and happy coding.
</DIV>
````

<a id="w002341"></a>

## W002341 — Networking

Bron-ID: W002341
Namespace: Main (0)
Revisie: 7596 | Datum: 2009-04-06T22:56:07Z
Bron: https://wiki.osdev.org/Networking
Revisie-URL: https://wiki.osdev.org/index.php?title=Networking&oldid=7596
Doorverwijzing: Network Stack

````text
#REDIRECT [[Network Stack]]
````

<a id="w002343"></a>

## W002343 — Talk:Opcode syntax

Bron-ID: W002343
Namespace: Talk (1)
Revisie: 22123 | Datum: 2018-02-01T01:04:38Z
Bron: https://wiki.osdev.org/Talk:Opcode_syntax
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AOpcode_syntax&oldid=22123

````text
This page mainly discusses the differences between AT&T and Intel assembler syntax (not opcodes, which would be called mnemonics), and should be named to reflect this. [[User:Virtlink|Virtlink]] 20:45, 13 July 2008 (UTC)

== Citation or correction needed == 

{{quote|Case Sensitivity: MOVL is not the same as movl.}}
I cannot find any documentation supporting this claim including the references listed at the end of the page. --[[User:Qyriad|Qyriad]] ([[User talk:Qyriad|talk]]) 01:02, 1 February 2018 (UTC)
````

<a id="w002345"></a>

## W002345 — User:Virtlink

Bron-ID: W002345
Namespace: User (2)
Revisie: 13854 | Datum: 2012-07-25T19:24:17Z
Bron: https://wiki.osdev.org/User:Virtlink
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AVirtlink&oldid=13854

````text
This is my Wiki page. If you have anything to say, please do so on my [[User_talk:Virtlink|talk page]].

== Personal pages ==
* [[User:Virtlink/My dream CPU|My dream CPU]]

== OSDev Environment ==
If someone where to build an OS development environment (a la Visual Studio), it's features would be:
* '''Modularity'''<br />Add support for new languages, architectures, (initrd) editors, etc... through modules.
* '''Syntax highlighting'''
* '''Autocompletion'''<br />For various assembler languages, C, C++.
* '''Integrated emulator and debugger'''
* '''Project management'''
* '''Auto makefile generator'''<br />As part of the project.
* '''Step-by-step debugging'''

{{Infobox Instruction|Instruction=MOV|Definition=Some definition|Usage=MOV dest, src|UsageSyntax=Intel|Usage2=MOVL %src, %dest|Usage2Syntax=AT&T}}
````

<a id="w002346"></a>

## W002346 — GUI

Bron-ID: W002346
Namespace: Main (0)
Revisie: 26818 | Datum: 2021-12-30T10:55:02Z
Bron: https://wiki.osdev.org/GUI
Revisie-URL: https://wiki.osdev.org/index.php?title=GUI&oldid=26818

````text
A '''Graphical User Interface''' or '''GUI''' uses graphics in combination with text to display output. GUIs usually require a mouse to be able to take input and to be used to their fullest extent.

== Requirements ==
Coding a GUI takes lots of time, knowledge, and patience. The OS has to have a graphics driver and a mouse driver at the very least, so it can check for mouse clicks on areas like buttons on the screen.
You need to know how to get into a graphics mode and plot pixels, lines, and shapes on the screen for starters. You also need to know about the signals that [[Mouse Input|PS/2 mice]] send when they are moved and buttons are pressed and held down, and how to implement a driver for handling all that stuff.

== Techniques ==

There are many ways to write a GUI, the following technique is a simple, quick and dirty way to create a generic GUI for your OS.  The tutorial assumes the kernel is running in protected mode and the video resolution has been set using VBE in a linear frame buffer and multi-tasking is already enabled in your kernel.  The kernel should probe the BIOS for the VBE Mode Info Block (see [[Getting VBE Mode Info]]), which returns information about the established video resolution mode.

The Screen module creates a Back Buffer for off screen writing and exposes methods used by other modules to write to the buffer.  
<source lang="c">
 void InitScreen();
 void PutPixel(int X, int Y, int RGB);
 void PutRect(int X, int Y, int Width, int Height, int RGB);
 void PutLine(int X1, int Y1, int X2, int Y2, int RGB);
</source>

Somewhere in real mode, the VESA BIOS Extentions were called to set the resolution (int 0x10 function 0x4f02) and the mode information was retrieved from VBE (int 0x10 function 0x4f01).  The mode info is stored in a VbeModeInfo struct as outlined in the VBE documentation and [[Getting VBE Mode Info| here]].  This tutorial assumes a flat/linear frame buffer is selected (bit 14 of BX register is set when calling function 0x4f01).  This will map the video memory to the processors addess space making it accessible by the kernel (if the segment desciptors are setup right) This way no video page swapping is needed, just one giant memcpy.

During initialization, a Back Buffer is allocated large enough to hold all the pixels and a thread is kicked off to refresh the screen.  The refresh proc just copies the data from the back buffer to the video memory in a continuous loop:
<source lang="c">
 void InitScreen(){
     ScreenBufferSize = VbeModeInfoBlock.XResolution * VbeModeInfoBlock.YResolution 
                                                     * (VbeModeInfoBlock.BitsPerPixel/8);
     ScreenBackBuffer = alloc(ScreenBufferSize);
     CreateThread(ScreenRefreshProc, NULL);
 }

 void ScreenRefreshProc(){
    for (;;){
        PaintDesktop(); //paints the desktop and all windows
        PaintCursor(); //finally paints the cursor so it's on top
 #if WAIT_FOR_VERTICAL_RETRACE
        while ((inportb(0x3DA) & 0x08));
        while (!(inportb(0x3DA) & 0x08));
 #endif
        memcpy(VbeModeInfoBlock.VideoBaseAddr, ScreenZBuffer, ScreenBufferSize);
  }
 }
</source>

Writing to RAM is generally faster than writing to video memory and many areas of the back buffer will be repeatidly over written during a single screen refresh operation, making a back buffer neccessary.

Now a function to plot pixels to the back buffer.  This will be used by other methods instead of writing directly to video memory:
<source lang="c">
 void PutPixel(int x,int y, int color){
    //do not write memory outside the screen buffer, check parameters against the VBE mode info
    if (x<0 || x>VbeModeInfoBlock.XResolution || y<0 || y>VbeModeInfoBlock.YResolution) return;
    if (x) x = (x*(VbeModeInfoBlock.BitsPerPixel>>3));
    if (y) y = (y*VbeModeInfoBlock.BytesPerScanLine);
    register char * cTemp;
    cTemp = &g_ScreenBuffer[x+y];
    cTemp[0] = color & 0xff;
    cTemp[1] = (color>>8) & 0xff;
    cTemp[2] = (color>>16) & 0xff;
 }
</source>

PutLine() and PutRect() a should check the boundaries of the X and Y parameters to ensure no pixels are plotted outside the allocated memory area of the back buffer.  This prevents memory corruption and the pixels wont be visible on the display anyway.  I'll leave the PutRect and PutLine implementation up to you or may come back and fill it in later.

In the RefreshScreenProc() method you'll notice the PaintDesktop() call.  This GUI method works by painting every object from the bottom up.  First the desktop background is painted then each window is painted on top of it (or one another depending on the z-order of the windows and their positions):
<source lang="c">
 void PaintDesktop(){
    //fill the background of the desktop
    PutRect(0,0,Screen.Width, Screen.Height, 0xc0c0c0c0);
    //now tell every child window to paint itself:
    for (int i=0 ; i<ChildWindows.size() ; i++){
        ChildWindows[i].Paint();
    }
 }
</source>
Each child window has it's own Paint() method as well and subsequently calls each of it's child window's Paint() method:
<source lang="cpp">
 void Window::Paint(){
     //paint a navy blue window
     PutRect(this->Left, this->Top, this->Width, this->Height, rgbNavy);
     //put a small red square in the top right hand corner of the window
     PutRect(this->Left + this->Width - 5, this->Top, 5, 5, rgbRed);
     //put the title bar text at the top of the window
     Put8x8String(this->Left+1, this->Top+1, this->Title, rgbBlack);
     for (int i=0 ; i<ChildWindows.size() ; i++){
        ChildWindows[i].Paint();
    }
 }
</source>
After the Paint() method of all the windows have been called, the RefreshScreenProc calls PaintCursor():
<source lang="c">
 void PaintCursor(){
    //just make a white box at the cursor position:
    PutRect(Mouse.X, Mouse.Y, 5, 5, rgbWhite);
 }
</source>

===TODO===

Theres a lot that can be done to improve the quality and performance.  Using this bottom up approach means that each area of the screen could potentially be overwritten several times if multiple windows are stacked on top of one another.  There is a lot of overhead re-rendering the same windows for every refresh itteration.  While this technique has a low memory requirement, performance can be greatly improved by pre-rendering windows in a seperate buffer and copying the contents to the screen buffer.  Each Window would have it's own screen buffer where rendering is performed only when neccessary (creation, resize, etc).  A similar technique is used [http://www.osdever.net/tutorials/view/gui-development here].  In [http://www.osdever.net/tutorials/view/gui-development this]example each window has it's own canvas and the contents are copied to the screen buffer during refresh operations and rendering is performed only once.  The downside to this technique is the additional memory requirements.

Some suggestions and things to watch out for: 

*All drawing should be performed on the back buffer. Drawing to the video memory will get overwritten by the ScreenRefreshProc.
*You will need some way to keep track of child windows and to setup their z-order (display order), so top level windows get drawn last, bottom level windows get drawn first.
*Make more calls in the window's Draw() method to make better looking windows. Add a border and some 3D effects.
*Pixel plotting differs depending on resolution and color depth. You'll want fast pixel plotting methods for each resolution and color depth.
*See [[Drawing In Protected Mode]] for writing strings using fixed width fonts.
*See [[Simple Scalable Fonts]] for a simple scalable font technique.
*See [[Loading Icons]] on how to handle image files to display icons.
*Use Bresenham's line drawing algorithms for best performance and visual appeal.
*PutRect and PutLine should have their own pixel plotting (calling PutPixel on a large rect will slow rendering WAY down).
*There are several ways to handle mouse and keyboard input. None are terribly difficult. Perhaps the easiest is for each Window to capture each mouse and keyboard event and determine whether or not to process them. Another way is for the mouse and keyboard modules to determine which windows to send messages or events to. These generally are implementation specific requirements.
*Use an optimized version of memcpy. Otherwise, on emulators like Bochs, screen updates will be painfully slow. (See: [[topic:18119|Optimized memory functions]])

== The new Alternative: UEFI ==

Instead of using VBE or real mode BIOS calls, you can use the (U)EFI methods like [[GOP]], provided that you make your OS run on (U)EFI and not on old clunky BIOS.

== See Also ==
=== Articles ===
* [[Text UI]]
* [[Getting VBE Mode Info]]
* [[Drawing In Protected Mode]]

[[de:Graphical User Interface]]
[[Category:Video]]
[[Category:Graphical UI]]

=== Threads ===
* [[Topic:27684|Alpha Blending]]

=== External links ===

* [http://www.osdever.net/tutorials/view/gui-development GUI Development]
* [https://jmarlin.github.io/wsbe/ Windowing Systems by Example]
````

<a id="w002347"></a>

## W002347 — Graphical User Interface

Bron-ID: W002347
Namespace: Main (0)
Revisie: 5930 | Datum: 2008-07-14T23:28:41Z
Bron: https://wiki.osdev.org/Graphical_User_Interface
Revisie-URL: https://wiki.osdev.org/index.php?title=Graphical_User_Interface&oldid=5930
Doorverwijzing: GUI

````text
#REDIRECT [[GUI]]
````

<a id="w002349"></a>

## W002349 — RTL8169

Bron-ID: W002349
Namespace: Main (0)
Revisie: 30398 | Datum: 2026-03-24T21:48:49Z
Bron: https://wiki.osdev.org/RTL8169
Revisie-URL: https://wiki.osdev.org/index.php?title=RTL8169&oldid=30398

````text
'''Programming Guide for the RTL8169(S)-32/64 Network Interface Chipsets''' 

The RTL8169 is Realtek's next generation of high-performance network cards.  This particular chipset is designed to operate at 10/100/1000 Mbps speeds.  

== Identification ==
The basic interface for the RTL8169 series is common in several Realtek network cards. This applies to the following noncomprehensive list of PCI Vendor and Device IDs from FreeBSD's re driver:
* 10ec:8161
* 10ec:8168
* 10ec:8169
* 1259:c107
* 1737:1032
* 16ec:0116

== Basic Startup == 

=== Get the MAC Address ===

First things first, we need to get the NIC's physical MAC address. Normally we should go through the EEPROM to access the MAC address, but in this case we will take a shortcut and obtain the address from the MAC0-6 registers.  The MAC registers start at offset 0x00 and should be accessed in 8-bit segments. 

ex:
<syntaxhighlight lang="c">
 char i;
 for (i = 0; i < 6; i++)
 {
    mac_address[i] = inportb(ioaddr + i); /*ioaddr is the base address obtainable from the PCI device configuration space.*/
 }
</syntaxhighlight>

=== Reset the Chip ===

It is necessary to reset the RTL8169 to put the registers in a known default state, as well as begin powering up the chip.  We need to send the reset command to Chip Command register at offset 0x37.  The reset bit is at offset 0x10, therefore we need to do an 'outportb' using those offsets to tell the chip to begin the resetting process.  Once we have sent that command, it will take some time for the chip to complete the operation, and because it is altering registers we should stop all further writes/reads to registers until it has completed.  The reset bit in the Command Register will self-clear once the operation is finished, that's what we should check for before proceeding setting up the chip. 

ex: 
<syntaxhighlight lang="c">
 outportb(ioaddr + 0x37, 0x10); /*set the Reset bit (0x10) to the Command Register (0x37)*/
 while(inportb(ioaddr + 0x37) & 0x10)
   ;/*setting a timeout could be useful if the card is problematic*/
</syntaxhighlight>

== Setting up the Rx Descriptors ==
 
One of the new features introduced with this new-gen chip is that it is completely descriptor-based.  The chip supports up to 3 unique descriptor sets with 1024 descriptors available per set; 1024 for receiving packets, 1024 for normal transmission, and 1024 for high-priority transmission packets.  It is not required to use all of the descriptors per set, or even use all three sets.  Each descriptor is 16-bytes wide and contains a 64-bit buffer address (to store data for transmission or to tell the reception process where to dump the data) and Command uint32_t.

Rx Descriptors are used to tell the NIC where to put packets once received.  Because this is an asynchronous operation, we must setup the descriptors before we enable the reception of packets so that the NIC knows where to put things.  A major function of descriptors is to tell the NIC which descriptors are owned by the OS (host) and which ones are owned by the NIC.  If the descriptor is owned by the NIC, the NIC can then read it and use it for the next packet it receives, if not, it either skips over it or sends a RDU (Rx Descriptor Unavailable) interrupt and halts all further Rx operations.  Because there can be multiple descriptors used, there is also a EOR (End of Rx descriptor Ring) bit used when the NIC has reached the end of the descriptors and needs to loop back to the beginning of the descriptors.  There is one more big 'gotcha' with all of this, the beginning of the descriptor arrays must be aligned on a 256-byte boundary. 

====Descriptor format====
{| {{wikitable}}
! Offset
! Name
! Description
|-
| dword 0 || Flags || 31 bit=OWN (if set, card own this descriptor) 30 bit=EOR (last descriptor in list)
|-
| dword 1 || VLAN  || Unused
|-
| dword 2 || Pointer to buffer low 32 bits  ||
|-
| dword 3 || Pointer to buffer high 32 bits  ||
|-
|}

====Code example====
<syntaxhighlight lang="c">
 struct Descriptor {
     uint32_t command;  /* command/status uint32_t */
     uint32_t vlan;     /* currently unused */
     uint32_t low_buf;  /* low 32-bits of physical buffer address */
     uint32_t high_buf; /* high 32-bits of physical buffer address */
 };
  
 struct Descriptor *Rx_Descriptors = (struct Descriptor *)0x100000; /* 1MB Base Address of Rx Descriptors */
  
 void setup_rx_descriptors()
 {
     /* rx_buffer_len is the size (in bytes) that is reserved for incoming packets */
     unsigned int OWN = 0x80000000, EOR = 0x40000000; /* Bit offsets */
     int i;
     for(i = 0; i < num_of_rx_descriptors; i++) /* num_of_rx_descriptors can be up to 1024 */
     {
         if(i == (num_of_rx_descriptors - 1)) /* Last descriptor? if so, set the EOR bit */
           Rx_Descriptors[i].command = (OWN | EOR | (rx_buffer_len & 0x3FFF));
         else
           Rx_Descriptors[i].command = (OWN | (rx_buffer_len & 0x3FFF));
         /* VLAN adjustments are not part of this guide at the moment - leave as zeros for normal operation */
         Rx_Descriptors[i].low_buf = (unsigned int)&packet_buffer_address; /* This is where the packet data will go */
         /* If you are programming for a 64-bit OS, put the high memory location in the 'high_buf' descriptor area */
     }
 }
</syntaxhighlight>

== Configuring RxConfig and TxConfig ==
=== (Un)locking Registers ===
In order to use some critical registers, we must 'unlock' them to alter their states and then 'lock' them after we are done to prevent accidental alterations to those registers during operation.  The 9346CR register is at offset 0x50 and is used as a key to those critical registers.  In order to 'unlock' them, we must set the card in "Config register write enable" mode.  This is done by setting both the EEM (operating mode) bits.  Once that has been done, it is then ok to finish setting up the card.  Once configuration is done and the OS begins proceeding as normal, the EEM bits should be set low to set the card in "normal" mode and lock the config registers. 

=== TxConfig ===
The Transmit Configuration register is at offset 0x40 and contains the configuration data for things such as DMA thresholds and the loopback status.  There are only two bits that we should be primarily concerned with, the MXDMA and the IFG bits.  The MXDMA bits are used for setting the DMA transfer threshold for transmit bursts.  I usually set this to unlimited burst data (one full packet).  This can be set to lower if you have proper handling through the host OS.  The IFG (Inter-Frame Gap) is the amount of time that elapses between each packet operation.  This can be set to the standard (96/960/9600ns) or you may increase the time (for reasons unknown to me) by increments of 8ns. 

eg:
<syntaxhighlight lang="c">
 outportl(ioaddr + 0x40, 0x03000700); /* IFG: normal, MXDMA: unlimited */
</syntaxhighlight>

=== RxConfig ===
The Receive Configuration register is at offset 0x44 and contains the configuration data for things such as DMA thresholds, FIFO thresholds, and packet reception rules.  The RXFTH bits are used for setting the Rx Fifo Threshold.  This threshold determines how much data is required in the buffers to begin telling the host OS that there is data ready.  I normally set this to unlimited data as it means that the NIC will tell the OS that data is ready as soon as a full packet is in the on-card FIFO buffer.  The MXDMA is based off the same explanation given for the Tx MXDMA. I set the MXDMA to unlimited.  The AB/AM/APM/AAP bits are used for setting packet filters.  I usually set this to a promiscuous mode with all of the previous bits set to allow the card to receive and process any packets that physically reach the NIC. 

eg:
<syntaxhighlight lang="c">
 outportl(ioaddr + 0x44, 0x0000E70F) /* RXFTH: unlimited, MXDMA: unlimited, AAP: set (promisc. mode set) */
</syntaxhighlight>
== Max Packet Sizes ==
=== Max Transmit Packet Size ===
The MTPS register is located at offset 0xEC and is used for setting the maximum packet size that can be transmitted from the RTL8169.  The maximum value that we can provide here is 0x3B.  That value is enough to cover 'jumbo' packets and still allow for a stable FIFO operation (no underruns).  This value can be set lower if need be, but for normal usage the maximum value is fine.  

eg:
<syntaxhighlight lang="c">
 outportb(ioaddr + 0xEC, 0x3B); /* Maximum tx packet size */
</syntaxhighlight>
=== Receive Packet Maximum Size ===
The RMS register is located at offset 0xDA and is used for setting the maximum packet size that can be received into the NIC.  If set to zero, the NIC will accept no packets (must be set prior to normal operation!).  The maximum size that can be set is 0x3FFF (16k - 1), but if a packet larger than 8k is received error bits are set and a lengthier process must be started in order to determine if the packet is actually good or not.  I keep this value to 0x1FFF (8k - 1) as it is rare to receive such large packets and it also ensures that error packets are actually error packets and not false positives.  

eg:
<syntaxhighlight lang="c">
 outportw(ioaddr + 0xDA, 0x1FFF); /* Maximum rx packet size */
</syntaxhighlight>
== Full Reset Example ==
This is a barebones example of how to reset the RTL8169 without anything like auto-negotiation (explained later) or (G)MII interactions.  

eg: 
<syntaxhighlight lang="c">
 struct Descriptor
 {
     uint32_t command;  /* command/status uint32_t */
     uint32_t vlan;     /* currently unused */
     uint32_t low_buf;  /* low 32-bits of physical buffer address */
     uint32_t high_buf; /* high 32-bits of physical buffer address */
 };

 /** 
  * Note that this assumes 16*1024=16KB (4 pages) of physical memory at 1MB and 2MB is identity mapped to 
  * the same linear address range
  */
 struct Descriptor *Rx_Descriptors = (struct Descriptor *)0x100000; /* 1MB Base Address of Rx Descriptors */
 struct Descriptor *Tx_Descriptors = (struct Descriptor *)0x200000; /* 2MB Base Address of Tx Descriptors */
    
 unsigned long num_of_rx_descriptors = 1024, num_of_tx_descriptors = 1024;
  
 void setup_rx_descriptors()
 {
     /* rx_buffer_len is the size (in bytes) that is reserved for incoming packets */
     unsigned int OWN = 0x80000000, EOR = 0x40000000; /* bit offsets */
     int i;
     for(i = 0; i < num_of_rx_descriptors; i++) /* num_of_rx_descriptors can be up to 1024 */
     {
         if(i == (num_of_rx_descriptors - 1)) /* Last descriptor? if so, set the EOR bit */
           Rx_Descriptors[i].command = (OWN | EOR | (rx_buffer_len & 0x3FFF));
         else
           Rx_Descriptors[i].command = (OWN | (rx_buffer_len & 0x3FFF));

         /** packet_buffer_address is the *physical* address for the buffer */
         Rx_Descriptors[i].low_buf = (unsigned int)&packet_buffer_address;
         Rx_Descriptors[i].high_buf = 0;
         /* If you are programming for a 64-bit OS, put the high memory location in the 'high_buf' descriptor area */
     }
 }
   
 void reset()
 {
     unsigned int i;
     unsigned char mac_address[6];
     
     outportb(ioaddr + 0x37, 0x10); /* Send the Reset bit to the Command register */
     while(inportb(ioaddr + 0x37) & 0x10){} /* Wait for the chip to finish resetting */
     
     for (i = 0; i < 6; i++)
       mac_address[i] = inportb(ioaddr + i);
    
     setup_rx_descriptors();
     outportb(ioaddr + 0x50, 0xC0); /* Unlock config registers */
     outportl(ioaddr + 0x44, 0x0000E70F); /* RxConfig = RXFTH: unlimited, MXDMA: unlimited, AAP: set (promisc. mode set) */
     outportb(ioaddr + 0x37, 0x04); /* Enable Tx in the Command register, required before setting TxConfig */
     outportl(ioaddr + 0x40, 0x03000700); /* TxConfig = IFG: normal, MXDMA: unlimited */
     outportw(ioaddr + 0xDA, 0x1FFF); /* Max rx packet size */
     outportb(ioaddr + 0xEC, 0x3B); /* max tx packet size */
     
     /* offset 0x20 == Transmit Descriptor Start Address Register 
        offset 0xE4 == Receive Descriptor Start Address Register 
  
        Again, these are *physical* addresses. This code assumes physical==linear, this is
        typically not the case in real world kernels
     */
     outportl(ioaddr + 0x20, (unsigned long)&Tx_Descriptors[0]; /* Tell the NIC where the first Tx descriptor is. NOTE: If writing a 64-bit address, split it into two 32-bit writes.*/
     outportl(ioaddr + 0xE4, (unsigned long)&Rx_Descriptors[0]; /* Tell the NIC where the first Rx descriptor is. NOTE: If writing a 64-bit address, split it into two 32-bit writes.*/
     
     outportb(ioaddr + 0x37, 0x0C); /* Enable Rx/Tx in the Command register */
     outportb(ioaddr + 0x50, 0x00); /* Lock config registers */
 }
</syntaxhighlight>

== See Also ==
=== Articles ===
* [[RTL8139]] -  Related preceding chipset

=== External Links ===
* [http://realtek.info/pdf/rtl8169s.pdf Datasheet for the RTL8169S and RTL8110S chipsets].
* [http://www.bitsavers.org/components/realtek/RTL8139/RTL8110S_8169S_DataSheet_1.3_200309.pdf Programming Datasheet for RTL8169]

[[Category:Network Hardware]]
[[Category:Standards]]
````

<a id="w002350"></a>

## W002350 — X86-64 Instruction Encoding

Bron-ID: W002350
Namespace: Main (0)
Revisie: 30896 | Datum: 2026-07-17T12:50:16Z
Bron: https://wiki.osdev.org/X86-64_Instruction_Encoding
Revisie-URL: https://wiki.osdev.org/index.php?title=X86-64_Instruction_Encoding&oldid=30896

````text
This article describes how x86 and x86-64 instructions are encoded.

== General Overview ==
An x86-64 instruction may be at most 15 bytes in length. It consists of the following components in the given order, where the prefixes are at the least-significant (lowest) address in memory:
* [[#Legacy Prefixes|Legacy prefixes]] (1-4 bytes, optional)
* [[#Opcode|Opcode with prefixes]] (1-4 bytes, required)
* [[#ModR.2FM|ModR/M]] (1 byte, if required)
* [[#SIB|SIB]] (1 byte, if required)
* [[#Displacement|Displacement]] (1, 2, 4 or 8 bytes, if required)
* [[#Immediate|Immediate]] (1, 2, 4 or 8 bytes, if required)

== Registers ==
The registers are encoded using the 4-bit values in the X.Reg column of the following table. ''X.Reg'' is in binary.
<div style="font-size: smaller">
{| {{wikitable|background: white;}}
!style="background-color: #f9f9f9"| X.Reg
!style="background-color: #f9f9f9"| 8-bit GP
!style="background-color: #f9f9f9"| 16-bit GP
!style="background-color: #f9f9f9"| 32-bit GP
!style="background-color: #f9f9f9"| 64-bit GP
!style="background-color: #f9f9f9"| 80-bit x87
!style="background-color: #f9f9f9"| 64-bit MMX
!style="background-color: #f9f9f9"| 128-bit XMM
!style="background-color: #f9f9f9"| 256-bit YMM
!style="background-color: #f9f9f9"| 16-bit Segment
!style="background-color: #f9f9f9"| 32-bit Control
!style="background-color: #f9f9f9"| 32-bit Debug
|-
!style="background-color: #f9f9f9"| 0.000 (0)
|AL||AX||EAX||RAX||ST0||MMX0||XMM0||YMM0||ES||CR0||DR0
|-
!style="background-color: #f9f9f9"| 0.001 (1)
|CL||CX||ECX||RCX||ST1||MMX1||XMM1||YMM1||CS||CR1||DR1
|-
!style="background-color: #f9f9f9"| 0.010 (2)
|DL||DX||EDX||RDX||ST2||MMX2||XMM2||YMM2||SS||CR2||DR2
|-
!style="background-color: #f9f9f9"| 0.011 (3)
|BL||BX||EBX||RBX||ST3||MMX3||XMM3||YMM3||DS||CR3||DR3
|-
!style="background-color: #f9f9f9"| 0.100 (4)
|AH, SPL<span style="vertical-align: super">[[#Table1Note1|1]]</span>||SP||ESP||RSP||ST4||MMX4||XMM4||YMM4||FS||CR4||DR4
|-
!style="background-color: #f9f9f9"| 0.101 (5)
|CH, BPL<span style="vertical-align: super">[[#Table1Note1|1]]</span>||BP||EBP||RBP||ST5||MMX5||XMM5||YMM5||GS||CR5||DR5
|-
!style="background-color: #f9f9f9"| 0.110 (6)
|DH, SIL<span style="vertical-align: super">[[#Table1Note1|1]]</span>||SI||ESI||RSI||ST6||MMX6||XMM6||YMM6||-||CR6||DR6
|-
!style="background-color: #f9f9f9"| 0.111 (7)
|BH, DIL<span style="vertical-align: super">[[#Table1Note1|1]]</span>||DI||EDI||RDI||ST7||MMX7||XMM7||YMM7||-||CR7||DR7
|-
!style="background-color: #f9f9f9"| 1.000 (8)
|R8L||R8W||R8D||R8||-||MMX0||XMM8||YMM8||ES||CR8||DR8
|-
!style="background-color: #f9f9f9"| 1.001 (9)
|R9L||R9W||R9D||R9||-||MMX1||XMM9||YMM9||CS||CR9||DR9
|-
!style="background-color: #f9f9f9"| 1.010 (10)
|R10L||R10W||R10D||R10||-||MMX2||XMM10||YMM10||SS||CR10||DR10
|-
!style="background-color: #f9f9f9"| 1.011 (11)
|R11L||R11W||R11D||R11||-||MMX3||XMM11||YMM11||DS||CR11||DR11
|-
!style="background-color: #f9f9f9"| 1.100 (12)
|R12L||R12W||R12D||R12||-||MMX4||XMM12||YMM12||FS||CR12||DR12
|-
!style="background-color: #f9f9f9"| 1.101 (13)
|R13L||R13W||R13D||R13||-||MMX5||XMM13||YMM13||GS||CR13||DR13
|-
!style="background-color: #f9f9f9"| 1.110 (14)
|R14L||R14W||R14D||R14||-||MMX6||XMM14||YMM14||-||CR14||DR14
|-
!style="background-color: #f9f9f9"| 1.111 (15)
|R15L||R15W||R15D||R15||-||MMX7||XMM15||YMM15||-||CR15||DR15
|}
</div>
<small id="Table1Note1">1: When any REX prefix is used, SPL, BPL, SIL and DIL are used. Otherwise, without any REX prefix AH, CH, DH and BH are used.</small>

== Legacy Prefixes ==
Each instruction can have up to four prefixes. Sometimes a prefix is required for the instruction while it loses its original meaning (i.e. a 'mandatory prefix'). The following prefixes can be used, the order does not matter:
* Prefix group 1
** 0xF0: LOCK prefix
** 0xF2: REPNE/REPNZ prefix
** 0xF3: REP or REPE/REPZ prefix
* Prefix group 2
** 0x2E: CS segment override
** 0x36: SS segment override
** 0x3E: DS segment override
** 0x26: ES segment override
** 0x64: FS segment override
** 0x65: GS segment override
** 0x2E: Branch not taken
** 0x3E: Branch taken
* Prefix group 3
** 0x66: Operand-size override prefix
* Prefix group 4
** 0x67: Address-size override prefix

When there are two or more prefixes from a single group, the behavior is undefined. Some processors ignore the subsequent prefixes from the same group, or use only the last prefix specified for any group.

=== LOCK prefix ===
With the LOCK prefix, certain read-modify-write instructions are executed atomically. The LOCK prefix can only be used with the following instructions or an [[Exceptions#Invalid Opcode|Invalid Opcode Exception]] occurs: ADC, ADD, AND, BTC, BTR, BTS, CMPXCHG, CMPXCHG8B, CMPXCHG16B, DEC, INC, NEG, NOT, OR, SBB, SUB, XADD, XCHG and XOR.

=== REPNE/REPNZ, REP and REPE/REPZ prefixes ===
The repeat prefixes cause string handling instructions to be repeated.

The '''REP''' prefix will repeat the associated instruction up to CX times, decreasing CX with every repetition. It can be used with the INS, LODS, MOVS, OUTS and STOS instructions.

'''REPE''' and '''REPZ''' are synonyms and repeat the instruction until CX reaches 0 or when ZF is set to 0. It can be used with the CMPS, CMPSB, CMPSD, CMPSW, SCAS, SCASB, SCASD and SCASW instructions.

'''REPNE''' and '''REPNZ''' also are synonyms and repeat the instruction until CX reaches 0 or when ZF is set to 1. It can be used with the CMPS, CMPSB, CMPSD, CMPSW, SCAS, SCASB, SCASD and SCASW instructions

=== CS, SS, DS, ES, FS and GS segment override prefixes ===
Segment overrides are used with instructions that reference non-stack memory. The default segment is implied by the instruction, and using a specific override forces the use of the specified segment for memory operands.

In 64-bit the CS, SS, DS and ES segment overrides are ignored.

=== Branch taken/not taken prefixes ===
Branch hints may be used to lessen the impact of branch misprediction somewhat. The 'branch taken' hint is a strong hint, while the 'branch not taken' hint is a weak hint. The branch hints are only supported by Intel since the Pentium 4. Whether using them on AMD architectures has any (positive or negative) effect at all is not known.

=== Operand-size and address-size override prefix ===
The default operand-size and address-size can be overridden using these prefix. See the following table:

{| {{wikitable|background-color: white}}
!style="background-color: #f9f9f9"| &nbsp;
!style="background-color: #f9f9f9"| CS.d
!style="background-color: #f9f9f9"| REX.W
!style="background-color: #f9f9f9"| Prefix (0x66 if operand, 0x67 if address)
!style="background-color: #f9f9f9"| Operand size
!style="background-color: #f9f9f9"| Address size
|-
!style="background-color: #f9f9f9" rowspan="2"| Real mode /<br />Virtual 8086 mode
| N/A||N/A||{{No}}||16-bit||16-bit
|-
| N/A||N/A||{{Yes}}||32-bit||32-bit
|-
! style="background-color: #f9f9f9" rowspan="4" | Protected mode /<br />Long compatibility mode
| {{No|0}}||N/A||{{No}}||16-bit||16-bit
|-
| {{No|0}}||N/A||{{Yes}}||32-bit||32-bit
|-
| {{Yes|1}}||N/A||{{No}}||32-bit||32-bit
|-
| {{Yes|1}}||N/A||{{Yes}}||16-bit||16-bit
|-
! style="background-color: #f9f9f9" rowspan="4" | Long 64-bit mode
| Ignored||{{No|0}}||{{No}}||32-bit||64-bit
|-
| Ignored||{{No|0}}||{{Yes}}||16-bit||32-bit
|-
| Ignored||{{Yes|1}}||{{No}}||64-bit<span style="vertical-align: super">[[#Table2Note1|1]]</span>||64-bit
|-
| Ignored||{{Yes|1}}||{{Yes}}||64-bit||32-bit
|}
<small id="Table2Note1">1: Certain instructions default to (or are fixed at) 64-bit operands and do not need the REX prefix for this, see [[#Usage|this table]].</small>

==== NASM ====
NASM determines the operand size by looking at the ''MODRM.reg'' or (for a register) ''MODRM.rm'' fields. When they are both 32-bit, the operand size becomes 32-bit. Same for 16-bit and 64-bit. When they differ, an error occurs at compile time.
The address size is determined by looking at (for a memory operand) the ''MODRM.rm'' field, or the ''SIB.base'', ''SIB.index'' and displacement, in that order. So when ''SIB.base'' uses a 16-bit register (such as AX), the address size becomes 16-bit. Using a 32-bit displacement will result in the displacement being truncated.

== Opcode ==
The x86-64 instruction set defines many opcodes and many ways to encode them, depending on several factors.

=== Legacy opcodes ===
Legacy (and x87) opcodes consist of, in this order:
* mandatory prefix;
* REX prefix;
* opcode.

==== Mandatory prefix ====
Certain instructions (most notably the SIMD instructions) require a mandatory prefix (0x66, 0xF2 or 0xF3), which looks like a normal modifier prefix. When a mandatory prefix is required, it is put with the modifier prefixes before the REX prefix (if any).

==== REX prefix ====
The REX prefix is only available in [[Long Mode|long mode]].

===== Usage =====
A REX prefix must be encoded when:
* using 64-bit operand size and the instruction does not default to 64-bit operand size; or
* using one of the extended registers (R8 to R15, XMM8 to XMM15, YMM8 to YMM15, CR8 to CR15 and DR8 to DR15); or
* using one of the uniform byte registers SPL, BPL, SIL or DIL.

A REX prefix must not be encoded when:
* using one of the high byte registers AH, CH, BH or DH.

In all other cases, the REX prefix is ignored. The use of multiple REX prefixes is undefined, although processors seem to use only the last REX prefix.

Instructions that default to 64-bit operand size in long mode are:
{| {{wikitable}}
|-
| CALL (near)||ENTER||Jcc
|-
| JrCXZ||JMP (near)||LEAVE
|-
| LGDT||LIDT||LLDT
|-
| LOOP||LOOPcc||LTR
|-
| MOV CR(n)||MOV DR(n)||POP reg/mem
|-
| POP reg||POP FS||POP GS
|-
| POPFQ||PUSH imm8||PUSH imm32
|-
| PUSH reg/mem||PUSH reg||PUSH FS
|-
| PUSH GS||PUSHFQ||RET (near)
|}

===== Encoding =====
The layout is as follows:
<pre>
  7                           0
+---+---+---+---+---+---+---+---+
| 0   1   0   0 | W | R | X | B |
+---+---+---+---+---+---+---+---+
</pre>
{| {{wikitable|background-color:white;}}
! style="background-color:#f9f9f9;" | Field
! style="background-color:#f9f9f9;" | Length
! style="background-color:#f9f9f9;" | Description
|-
!style="background-color:#f9f9f9;"| 0100
|4 bits||Fixed bit pattern
|-
!style="background-color:#f9f9f9;"| W
|1 bit||When 1, a 64-bit operand size is used. Otherwise, when 0, the default operand size is used (which is 32-bit for most but not all instructions, see [[#Operand-size and address-size override prefix|this table]]).
|-
!style="background-color:#f9f9f9;"| R
|1 bit||This 1-bit value is an extension to the ''MODRM.reg'' field. See [[#Registers|Registers]].
|-
!style="background-color:#f9f9f9;"| X
|1 bit||This 1-bit value is an extension to the ''SIB.index'' field. See [[#64-bit addressing|64-bit addressing]].
|-
!style="background-color:#f9f9f9;"| B
|1 bit||This 1-bit value is an extension to the ''MODRM.rm'' field or the ''SIB.base'' field. See [[#64-bit addressing|64-bit addressing]]. It also extends the register operand for instructions with an opcode ending in ''+rb'', ''+rw'', ''+rd'', ''+ro''.<ref>Intel-64 and IA-32 architectures software manual, volume 2, chapter 3, section 1.1.1. At https://www.intel.com/content/www/us/en/content-details/825760/intel-64-and-ia-32-architectures-software-developer-s-manual-combined-volumes-2a-2b-2c-and-2d-instruction-set-reference-a-z.html#page=104</ref>
|}

Because the first four bits always equal 4, the existence of the REX prefix wipes out opcodes 0x40-0x4F, which were previously individual increment and decrement instructions for all eight registers. The Intel 64 and IA-32 Architectures
Software Developer’s Manual volume 2 states "The single-byte-opcode forms of the INC/DEC instructions are not available in 64-bit mode. INC/DEC functionality is still available using ModR/M forms of the same instructions (opcodes FF/0 and FF/1)."

==== Opcode ====
The opcode can be 1, 2 or 3 bytes in length. Depending on the opcode escape sequence, a different opcode map is selected. Possible opcode sequences are:
* <op>
* 0x0F <op>
* 0x0F 0x38 <op>
* 0x0F 0x3A <op>

Note that opcodes can specify that the REG field in the ModR/M byte is fixed at a particular value.

=== VEX/XOP opcodes ===
A VEX/XOP prefix must be encoded when:
* the instruction has only its VEX/XOP opcode and no legacy opcode; or
* 256-bit YMM registers are used; or
* more than three operands are used (e.g. ''nondestructive-source operations''); or
* when using 128-bit XMM destination registers, bits 128-255 of the corresponding YMM register must be cleared.

A VEX/XOP prefix must not be encoded when:
* when using 128-bit XMM destination registers, bits 128-255 of the corresponding YMM register must not be changed.


There are many VEX and XOP instructions, all of which can be encoded using the three byte VEX/XOP escape prefix. The VEX and XOP escape prefixes use fields with the following semantics:

{| {{wikitable|background-color:white;}}
!style="background-color:#f9f9f9;"| Field
!style="background-color:#f9f9f9;"| Length
!style="background-color:#f9f9f9;"| Description
|-
!style="background-color:#f9f9f9; text-align: left; vertical-align: top;"| VEX/XOP prefix
|style="vertical-align: top;"|8 bits||Prefix.
{| {{wikitable}}
! Prefix
! Opcode map and encoding
|-
| 0xC4||Three-byte VEX
|-
| 0xC5||Two-byte VEX
|-
| 0x8F||Three-byte XOP
|}
|-
!style="background-color:#f9f9f9; text-align: left; vertical-align: top;"| ~R
|style="vertical-align: top;"|1 bit||This 1-bit value is an 'inverted' extension to the ''MODRM.reg'' field. The inverse of REX.R. See [[#Registers|Registers]].
|-
!style="background-color:#f9f9f9; text-align: left; vertical-align: top;"| ~X
|style="vertical-align: top;"|1 bit||This 1-bit value is an 'inverted' extension to the ''SIB.index'' field. The inverse of REX.X. See [[#64-bit addressing|64-bit addressing]].
|-
!style="background-color:#f9f9f9; text-align: left; vertical-align: top;"| ~B
|style="vertical-align: top;"|1 bit||This 1-bit value is an 'inverted' extension to the ''MODRM.rm'' field or the ''SIB.base'' field. The inverse of REX.B. See [[#64-bit addressing|64-bit addressing]].
|-
!style="background-color:#f9f9f9; text-align: left; vertical-align: top;"| map_select
|style="vertical-align: top;"|5 bits||Specifies the opcode map to use.
|-
!style="background-color:#f9f9f9; text-align: left; vertical-align: top;"| W/E
|style="vertical-align: top;"|1 bit||For integer instructions: when 1, a 64-bit operand size is used; otherwise, when 0, the default operand size is used (equivalent with REX.W). For non-integer instructions, this bit is a general opcode extension bit.
|-
!style="background-color:#f9f9f9; text-align: left; vertical-align: top;"| ~vvvv
|style="vertical-align: top;"|4 bits||An additional operand for the instruction. The value of the XMM or YMM register (see [[#Registers|Registers]]) is 'inverted'.
|-
!style="background-color:#f9f9f9; text-align: left; vertical-align: top;"| L
|style="vertical-align: top;"|1 bit||When 0, a 128-bit vector lengh is used. Otherwise, when 1, a 256-bit vector length is used.
|-
!style="background-color:#f9f9f9; text-align: left; vertical-align: top;"| pp
|style="vertical-align: top;"|2 bits||Specifies an implied mandatory prefix for the opcode.
{| {{wikitable}}
! Value (binary)
! Implied mandatory prefix
|-
| 00||none
|-
| 01||0x66
|-
| 10||0xF3
|-
| 11||0xF2
|}
|}

==== Three byte VEX escape prefix ====
The layout is as follows, starting with a byte with value 0xC4:
<pre>
  7                           0       7                           0     7                           0
+---+---+---+---+---+---+---+---+   +---+---+---+---+---+---+---+---+   +---+---+---+---+---+---+---+---+
| 1   1   0   0   0   1   0   0 |   |~R |~X |~B |     map_select    |   |W/E|     ~vvvv     | L |   pp  |
+---+---+---+---+---+---+---+---+   +---+---+---+---+---+---+---+---+   +---+---+---+---+---+---+---+---+
</pre>
A VEX instruction whose values for certain fields are VEX.~X == 1, VEX.~B == 1, VEX.W/E == 0 and map_select == b00001 may be encoded using the [[#Two_byte_VEX_escape_prefix|two byte VEX escape prefix]].

==== Three byte XOP escape prefix ====
The layout is the same as the [[#Three_byte_VEX_escape_prefix|three-byte VEX escape prefix]], but with initial byte value 0x8F:
<pre>
  7                           0       7                           0     7                           0
+---+---+---+---+---+---+---+---+   +---+---+---+---+---+---+---+---+   +---+---+---+---+---+---+---+---+
| 1   0   0   0   1   1   1   1 |   |~R |~X |~B |     map_select    |   |W/E|     ~vvvv     | L |   pp  |
+---+---+---+---+---+---+---+---+   +---+---+---+---+---+---+---+---+   +---+---+---+---+---+---+---+---+
</pre>

The AMD64 Architecture Programmer’s Manual Volume 6 states that the map_select field must be equal to or greater than 8, to differentiate the XOP prefix from the POP instruction that formerly used opcode 0x8F.

==== Two byte VEX escape prefix ====
A VEX instruction whose values for certain fields are VEX.~X == 1, VEX.~B == 1, VEX.W/E == 0 and map_select == b00001 may be encoded using the two byte VEX escape prefix. The layout is as follows:
<pre>
  7                           0       7                           0
+---+---+---+---+---+---+---+---+   +---+---+---+---+---+---+---+---+
| 1   1   0   0   0   1   0   1 |   |~R |     ~vvvv     | L |   pp  |
+---+---+---+---+---+---+---+---+   +---+---+---+---+---+---+---+---+
</pre>

=== 3DNow! opcodes ===
3DNow! opcodes consist of, in this order:
* fixed opcode;
* (ModR/M, SIB, displacement);
* immediate opcode byte.

==== Fixed opcode ====
All 3DNow! opcodes have a fixed two-byte sequence equal to 0x0F 0x0F in the opcode position of the instruction.

==== Immediate opcode byte ====
3DNow! instructions encode the actual opcode as an 8-bit immediate value trailing the instruction (thus after the ModR/M, SIB and displacement).

== ModR/M and SIB bytes ==
The ModR/M and SIB bytes are used to encode up to two operands of an instruction, each of which is a direct register or effective memory address.

=== ModR/M ===
The ModR/M byte encodes a register or an opcode extension, and a register or a memory address. It has the following fields:
<pre>
  7                           0
+---+---+---+---+---+---+---+---+
|  mod  |    reg    |     rm    |
+---+---+---+---+---+---+---+---+
</pre>
{| {{wikitable|background-color: white}}
!style="background-color:#f9f9f9;"| Field
!style="background-color:#f9f9f9;"| Length
!style="background-color:#f9f9f9;"| Description
|-
!style="background-color:#f9f9f9; text-align: left; vertical-align: top;"| MODRM.mod
|style="vertical-align: top;"|2 bits||In general, when this field is b11, then register-direct addressing mode is used; otherwise register-indirect addressing mode is used.
|-
!style="background-color:#f9f9f9; text-align: left; vertical-align: top;"| MODRM.reg
|style="vertical-align: top;"|3 bits||This field can have one of two values:
* A 3-bit opcode extension, which is used by some instructions but has no further meaning other than distinguishing the instruction from other instructions.
* A 3-bit register reference, which can be used as the source or the destination of an instruction (depending on the instruction). The referenced register depends on the [[#Operand-size and address-size override prefix|operand-size]] of the instruction and the instruction itself. See [[#Registers|Registers]] for the values to use for each of the registers. The REX.R, VEX.~R or XOP.~R field can extend this field with 1 most-significant bit to 4 bits total.
|-
!style="background-color:#f9f9f9; text-align: left; vertical-align: top;"| MODRM.rm
|style="vertical-align: top;"|3 bits||Specifies a direct or indirect register operand, optionally with a displacement. The REX.B, VEX.~B or XOP.~B field can extend this field with 1 most-significant bit to 4 bits total.
|}

==== 16-bit addressing ====
These are the meanings of the ''Mod'' (vertically) and ''REX/VEX/XOP.B'' and ''R/M'' bits (horizontally) for [[#Operand-size and address-size override prefix|16-bit addressing]]. ''B.R/M'' and ''Mod'' are in binary. The SIB-byte is not used in 16-bit addressing. In ''Long processing mode'' there is no way to specify 16-bit addresses.
<div style="font-size: 70%; text-align: center">
{| {{wikitable}}
! 16-bit!! colspan="16" style="text-align: left;" | B.R/M
|-
! Mod
! x.000<br />AX, R8W
! x.001<br />CX, R9W
! x.010<br />DX, R10W
! x.011<br />BX, R11W
! x.100<br />SP, R12W
! x.101<br />BP, R13W
! x.110<br />SI, R14W
! x.111<br />DI, R15W
|-
! 00
| style="background-color: white" |[BX + SI]
| style="background-color: white" |[BX + DI]
| style="background-color: white" |[BP + SI]
| style="background-color: white" |[BP + DI]
| style="background-color: white" |[SI]
| style="background-color: white" |[DI]
| style="background-color: white" |[disp16]
| style="background-color: white" |[BX]
|-
! 01
| style="background-color: white" |[BX + SI + disp8]
| style="background-color: white" |[BX + DI + disp8]
| style="background-color: white" |[BP + SI + disp8]
| style="background-color: white" |[BP + DI + disp8]
| style="background-color: white" |[SI + disp8]
| style="background-color: white" |[DI + disp8]
| style="background-color: white" |[BP + disp8]
| style="background-color: white" |[BX + disp8]
|-
! 10
| style="background-color: white" |[BX + SI + disp16]
| style="background-color: white" |[BX + DI + disp16]
| style="background-color: white" |[BP + SI + disp16]
| style="background-color: white" |[BP + DI + disp16]
| style="background-color: white" |[SI + disp16]
| style="background-color: white" |[DI + disp16]
| style="background-color: white" |[BP + disp16]
| style="background-color: white" |[BX + disp16]
|-
! 11
| style="background-color: white" colspan="8"|r/m
|}
</div>

==== 32/64-bit addressing ====
These are the meanings of the ''Mod'' (vertically) and ''REX/VEX/XOP.B'' and ''R/M'' bits (horizontally) for [[#Operand-size and address-size override prefix|32 and 64-bit addressing]]. ''B.R/M'' and ''Mod'' are in binary.
<div style="font-size: 70%; text-align: center">
{| {{wikitable}}
! 32/64-bit!! colspan="16" style="text-align: left;" | B.R/M
|-
! Mod
! 0.000<br />AX
! 0.001<br />CX
! 0.010<br />DX
! 0.011<br />BX
! 0.100<br />SP
! 0.101<br />BP
! 0.110<br />SI
! 0.111<br />DI
! 1.000<br />R8
! 1.001<br />R9
! 1.010<br />R10
! 1.011<br />R11
! 1.100<br />R12
! 1.101<br />R13
! 1.110<br />R14
! 1.111<br />R15
|-
! 00
| style="background-color: white" colspan="4"|[r/m]
| style="background-color: white" |&#91;[[#SIB|SIB]]]
| style="background-color: white" |&#91;[[#RIP.2FEIP-relative_addressing|RIP/EIP]]<span style="vertical-align: super">[[#Table8Note1|1]],[[#Table8Note2|2]]</span> + disp32]
| style="background-color: white" colspan="6"|[r/m]
| style="background-color: white" |&#91;[[#SIB|SIB]]]
| style="background-color: white" |&#91;[[#RIP.2FEIP-relative_addressing|RIP/EIP]]<span style="vertical-align: super">[[#Table8Note1|1]],[[#Table8Note2|2]]</span> + disp32]
| style="background-color: white" colspan="2"|[r/m]
|-
! 01
| style="background-color: white" colspan="4"|[r/m + disp8]
| style="background-color: white" |&#91;[[#SIB|SIB]] + disp8]
| style="background-color: white" colspan="7"|[r/m + disp8]
| style="background-color: white" |&#91;[[#SIB|SIB]] + disp8]
| style="background-color: white" colspan="3"|[r/m + disp8]
|-
! 10
| style="background-color: white" colspan="4"|[r/m + disp32]
| style="background-color: white" |&#91;[[#SIB|SIB]] + disp32]
| style="background-color: white" colspan="7"|[r/m + disp32]
| style="background-color: white" |&#91;[[#SIB|SIB]] + disp32]
| style="background-color: white" colspan="3"|[r/m + disp32]
|-
! 11
| style="background-color: white" colspan="16"|r/m
|}
</div>
<small id="Table8Note1">1: In protected/compatibility mode, this is just ''disp32'', but in long mode this is ''[RIP]+disp32'' (for 64-bit addresses) or ''[EIP]+disp32'' (for 32-bit addresses, i.e. with address-size override prefix, [https://web.archive.org/web/20150810173516/http://objectmix.com/asm-x86-asm-370/69055-effect-address-size-prefix-rip-relative-addressing.html see here]).</small><br />
<small id="Table8Note2">2: In long mode, to encode ''disp32'' as in protected/compatibility mode, use the SIB byte.</small>

===== RIP/EIP-relative addressing =====
Addressing in x86-64 can be relative to the current instruction pointer value. This is indicated with the ''RIP'' (64-bit) and ''EIP'' (32-bit) instruction pointer registers, which are not otherwise exposed to the program and may not exist physically. RIP-relative addressing allows object files to be location independent.

=== SIB ===
The SIB byte has the following fields:
<pre>
  7                           0
+---+---+---+---+---+---+---+---+
| scale |   index   |    base   |
+---+---+---+---+---+---+---+---+
</pre>
{| {{wikitable|background-color: white}}
!style="background-color:#f9f9f9;"| Field
!style="background-color:#f9f9f9;"| Length
!style="background-color:#f9f9f9;"| Description
|-
!style="background-color:#f9f9f9; text-align: left; vertical-align: top;"|SIB.scale
|style="vertical-align: top;"|2 bits||This field indicates the scaling factor of SIB.index, where ''s'' (as used in the tables) equals 2<span style="vertical-align: super;">SIB.scale</span>.
{| {{wikitable}}
! SIB.scale
! factor ''s''
|-
| b00||1
|-
| b01||2
|-
| b10||4
|-
| b11||8
|}
|-
!style="background-color:#f9f9f9; text-align: left; vertical-align: top;"|SIB.index
|style="vertical-align: top;"|3 bits||The index register to use. See [[#Registers|Registers]] for the values to use for each of the registers. The REX.X, VEX.~X or XOP.~X field can extend this field with 1 most-significant bit to 4 bits total.
|-
!style="background-color:#f9f9f9; text-align: left; vertical-align: top;"|SIB.base
|style="vertical-align: top;"|3 bits||The base register to use. See [[#Registers|Registers]] for the values to use for each of the registers. The REX.B, VEX.~B or XOP.~B field can extend this field with 1 most-significant bit to 4 bits total.
|}

==== 32/64-bit addressing ====
The meaning of the SIB byte while using 32 or 64-bit addressing is as follows. The ModR/M byte's ''Mod'' field and the SIB byte's ''index'' field are used vertically, the SIB byte's ''base'' field and REX/VEX/XOP.B bit horizontally. The ''s'' is the [[#SIB|scaling factor]]. ''B.Base'', ''X.Index'' and ''Mod'' are in binary.
<div style="font-size: 70%; text-align: center">
{| {{wikitable}}
! colspan="2" | !! colspan="16" style="text-align: left;" | B.Base
|-
! Mod
! style="text-align: left;" | X.Index
! 0.000<br />AX
! 0.001<br />CX
! 0.010<br />DX
! 0.011<br />BX
! 0.100<br />SP
! 0.101<span style="vertical-align: super">[[#Table10Note1|1]]</span><br />BP
! 0.110<br />SI
! 0.111<br />DI
! 1.000<br />R8
! 1.001<br />R9
! 1.010<br />R10
! 1.011<br />R11
! 1.100<br />R12
! 1.101<span style="vertical-align: super">[[#Table10Note1|1]]</span><br />R13
! 1.110<br />R14
! 1.111<br />R15
|-
! rowspan="16" | 00 !! style="text-align: left;" | 0.000 AX
| style="background-color: white" rowspan="4" colspan="5"|[base + (index * s)]
| style="background-color: white" rowspan="4"|[(index * s) + disp32]
| style="background-color: white" rowspan="4" colspan="7" |[base + (index * s)]
| style="background-color: white" rowspan="4"|[(index * s) + disp32]
| style="background-color: white" rowspan="4" colspan="2"|[base + (index * s)]
|-
! style="text-align: left;" | 0.001 CX
|-
! style="text-align: left;" | 0.010 DX
|-
! style="text-align: left;" | 0.011 BX
|-
! style="text-align: left;" | 0.100<span style="vertical-align: super">[[#Table10Note2|2]]</span> SP
| style="background-color: white" colspan="5"|[base]
| style="background-color: white"| [disp32]
| style="background-color: white" colspan="7"|[base]
| style="background-color: white"| [disp32]
| style="background-color: white" colspan="2"|[base]
|-
! style="text-align: left;" | 0.101 BP
| style="background-color: white" rowspan="11" colspan="5"|[base + (index * s)]
| style="background-color: white" rowspan="11"|[(index * s) + disp32]
| style="background-color: white" rowspan="11" colspan="7"|[base + (index * s)]
| style="background-color: white" rowspan="11"|[(index * s) + disp32]
| style="background-color: white" rowspan="11" colspan="2"|[base + (index * s)]
|-
! style="text-align: left;" | 0.110 SI
|-
! style="text-align: left;" | 0.111 DI
|-
! style="text-align: left;" | 1.000 R8
|-
! style="text-align: left;" | 1.001 R9
|-
! style="text-align: left;" | 1.010 R10
|-
! style="text-align: left;" | 1.011 R11
|-
! style="text-align: left;" | 1.100 R12
|-
! style="text-align: left;" | 1.101 R13
|-
! style="text-align: left;" | 1.110 R14
|-
! style="text-align: left;" | 1.111 R15
|-
| colspan="18" style="background-color: white; border-top: solid 2px black; border-bottom: solid 2px black; border-left: none; border-right: none;" |
|-
! colspan="2" | !! colspan="16" style="text-align: left;" | B.Base
|-
! Mod
! style="text-align: left;" | X.Index
! 0.000<br />AX
! 0.001<br />CX
! 0.010<br />DX
! 0.011<br />BX
! 0.100<br />SP
! 0.101<br />BP
! 0.110<br />SI
! 0.111<br />DI
! 1.000<br />R8
! 1.001<br />R9
! 1.010<br />R10
! 1.011<br />R11
! 1.100<br />R12
! 1.101<br />R13
! 1.110<br />R14
! 1.111<br />R15
|-
! rowspan="16" | 01 !! style="text-align: left;" | 0.000 AX
| style="background-color: white" rowspan="4" colspan="16"|[base + (index * s) + disp8]
|-
! style="text-align: left;" | 0.001 CX
|-
! style="text-align: left;" | 0.010 DX
|-
! style="text-align: left;" | 0.011 BX
|-
! style="text-align: left;" | 0.100<span style="vertical-align: super">[[#Table10Note2|2]]</span> SP
| style="background-color: white" colspan="16"|[base + disp8]
|-
! style="text-align: left;" | 0.101 BP
| style="background-color: white" rowspan="11" colspan="16"|[base + (index * s) + disp8]
|-
! style="text-align: left;" | 0.110 SI
|-
! style="text-align: left;" | 0.111 DI
|-
! style="text-align: left;" | 1.000 R8
|-
! style="text-align: left;" | 1.001 R9
|-
! style="text-align: left;" | 1.010 R10
|-
! style="text-align: left;" | 1.011 R11
|-
! style="text-align: left;" | 1.100 R12
|-
! style="text-align: left;" | 1.101 R13
|-
! style="text-align: left;" | 1.110 R14
|-
! style="text-align: left;" | 1.111 R15
|-
| colspan="18" style="background-color: white; border-top: solid 2px black; border-bottom: solid 2px black; border-left: none; border-right: none;" |
|-
! colspan="2" | !! colspan="16" style="text-align: left;" | B.Base
|-
! Mod
! style="text-align: left;" | X.Index
! 0.000<br />AX
! 0.001<br />CX
! 0.010<br />DX
! 0.011<br />BX
! 0.100<br />SP
! 0.101<br />BP
! 0.110<br />SI
! 0.111<br />DI
! 1.000<br />R8
! 1.001<br />R9
! 1.010<br />R10
! 1.011<br />R11
! 1.100<br />R12
! 1.101<br />R13
! 1.110<br />R14
! 1.111<br />R15
|-
! rowspan="16" | 10 !! style="text-align: left;" | 0.000 AX
| style="background-color: white" rowspan="4" colspan="16"|[base + (index * s) + disp32]
|-
! style="text-align: left;" | 0.001 CX
|-
! style="text-align: left;" | 0.010 DX
|-
! style="text-align: left;" | 0.011 BX
|-
! style="text-align: left;" | 0.100<span style="vertical-align: super">[[#Table10Note2|2]]</span> SP
| style="background-color: white" colspan="16"|[base + disp32]
|-
! style="text-align: left;" | 0.101 BP
| style="background-color: white" rowspan="11" colspan="16"|[base + (index * s) + disp32]
|-
! style="text-align: left;" | 0.110 SI
|-
! style="text-align: left;" | 0.111 DI
|-
! style="text-align: left;" | 1.000 R8
|-
! style="text-align: left;" | 1.001 R9
|-
! style="text-align: left;" | 1.010 R10
|-
! style="text-align: left;" | 1.011 R11
|-
! style="text-align: left;" | 1.100 R12
|-
! style="text-align: left;" | 1.101 R13
|-
! style="text-align: left;" | 1.110 R14
|-
! style="text-align: left;" | 1.111 R15
|}
</div>
<small id="Table10Note1">1: No base register is encoded.</small><br />
<small id="Table10Note2">2: No index register is encoded.</small>

== Displacement ==
A displacement value is a 1, 2, 4, or 8 byte offset added to the calculated address. When an 8 byte displacement is used, no immediate operand is encoded.

The displacement value, if any, follows the ModR/M and SIB bytes discussed above. When the ModR/M or SIB tables state that a ''disp'' value is required, or without a ModR/M byte the use of ''moffset'' (AMD) or ''moffs'' (Intel) in the mnemonic syntax of the instruction, then the displacement bytes are required.

== Immediate ==
Some instructions require an immediate value. The instruction (and the operand-size column in the above table) determine the length of the immediate value. The ''imm8'' mnemonic (or 8-bit [[#Operand-size and address-size override prefix|operand-size]]) means a one byte immediate value, ''imm16'' (or 16-bit operand-size) means a two byte immediate value, ''imm32'' (or 32-bit operand-size) a four byte value and ''imm64'' (or 64-bit operand-size) an eight byte value. When an 8 byte immediate value is encoded, no displacement can be encoded.

== See Also ==
=== External References ===
* AMD64 Architecture Programmer's Manual [https://www.amd.com/system/files/TechDocs/24594.pdf Volume 3: General-Purpose and System Instructions]
* [http://www.intel.com/products/processor/manuals/ Intel 64 and IA-32 Architectures Software Developer's Manuals]

[[Category:X86 CPU]]
[[Category:X86-64]]
````

<a id="w002351"></a>

## W002351 — Talk:CIL

Bron-ID: W002351
Namespace: Talk (1)
Revisie: 5971 | Datum: 2008-07-16T23:37:37Z
Bron: https://wiki.osdev.org/Talk:CIL
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ACIL&oldid=5971

````text
=== Unmanaged/Managed ===
The terms "managed" and "unmanaged" are incorrectly used to safe and unsafe code, respectively. Unmanaged code, in the world of .Net, refers to code that is compiled directly in to machine code. I'm fixing the mistake now.
--[[User:Messiahandrw|Messiahandrw]] 23:37, 16 July 2008 (UTC)
````

<a id="w002352"></a>

## W002352 — C preprocessor

Bron-ID: W002352
Namespace: Main (0)
Revisie: 30418 | Datum: 2026-03-25T17:26:50Z
Bron: https://wiki.osdev.org/C_preprocessor
Revisie-URL: https://wiki.osdev.org/index.php?title=C_preprocessor&oldid=30418

````text

The C preprocessor is the first step in the process of translating C and C++ source code into a binary. Generally, the process walked through is preprocessing, compiling and finally linking. In trivial environments, the preprocessor is used only for '''#include'''ing header files, and providing "header guards" to avoid multiple inclusions. However, the preprocessor can do much more, and can be very useful - not only for C and C++ sources, but for your Assembly sources as well. Use it with care, since it can also obfuscate your source code and introduce bugs that may be very difficult to debug.

== General ==
The preprocessor handles ''preprocessor directives'', which are lines that begin with '' '#' ''. Really old compiler versions demanded that the '' '#' '' be placed in column 1, modern versions of C and C++ allow preprocessor directives to begin in any column, as long as the first non-whitespace character of the line is '' '#' ''.

Lines with preprocessor directives can be "continued" by placing a backslash ('\') as the last character of the line.

== Includes ==
The most familiar use of the preprocessor is to include header files (containing function declarations, definition of constants etc.):

<syntaxhighlight lang="c">
#include <stdio.h>
#include "myheader.h"
</syntaxhighlight>

The effect is that the contents of the given header file are pasted into the source file. The ''technical'' difference between <> and "" is that the compiler is allowed to satisfy <> includes internally, i.e. without actually accessing any on-disk files of that name. None of the prominent compilers do this, to the knowledge of the author, but it has become common practice to use <> for system headers and "" for your own header files.

Header files are searched for in a list of preconfigured directories (the ''include path''). This list of include directories can be prepended to by the user (e.g. by using the "-I <directory>" option of [[GCC]]).

The ''#include'' statement can be used in other contexts, too: As a replacement for assembler-specific include directives, for example.

Another possible use is "templating" a piece of code that keeps recurring in more than one source file but could not be put into a seperate function. This way, you could still reduce redundancy by keeping the shared code in a single file and merely ''#include''ing it where needed. This, however, is a pretty ugly construct and should be avoided if possible.

== Preprocessor Macros, pt. 1 ==
The preprocessor can ''define'' tokens. It is good custom to write these tokens in ALL CAPS. (See pt. 2 as for why.)

<syntaxhighlight lang="c">
#define MYTOKEN
</syntaxhighlight>

Most compilers also allow the definition of preprocessor tokens on the command line, e.g. the "-D MYTOKEN" option for [[GCC]].

== Conditional Compilation ==
The preprocessor can ''conditionally'' select which parts of source code to compile, depending on whether a given token is defined or not (see above).

<syntaxhighlight lang="c">
#define MYTOKEN

#ifdef MYTOKEN
/* This source will be compiled */
#endif

#ifndef MYTOKEN
/* This source will be removed */
#else
/* This source will be compiled */
#endif
</syntaxhighlight>

Note that such ''#if'' / ''#ifdef'' / ''#ifndef'' - ''#endif'' sections can be nested.

== Header Guards ==
Non-trivial projects face the problem that a header file includes other header files in turn. Let's say both ''abc.h'' and ''def.h'' both include ''xyz.h''. Should you ''#include'' both ''abc.h'' and ''def.h'' in your source, you will likely end up with warnings and errors about redefinitions etc.

The solution are ''header guards'', a combination of conditional compilation and token definition:

<syntaxhighlight lang="c">
/* abc.h */

#ifndef ABC_H_
#define ABC_H_

/* declaractions here */

#endif
</syntaxhighlight>

== Preprocessor Macros, pt. 2 ==
Preprocessor tokens can also be assigned a ''value''.

The preprocessor will replace any occurrence of a defined token in the source code with the value the token has been defined to. ''This is also true for tokens that have been defined to nothing'' (as in pt. 1 above). This is the reason why preprocessor tokens are customarily written in ALL CAPS - to avoid accidential clashes with identifiers used in the source code itself.

The ''#if'' statement can be used to base conditional compilation on token values. Note that the preprocessor can only work with compile-time constants. Compiler-evaluated code like `sizeof()` cannot be used in preprocessor directives. On the upside, the preprocessor can natively handle non-numerical values.

<syntaxhighlight lang="c">
#define MYTOKEN foo
#define OTHERTOKEN 42

#if MYTOKEN == foo
/* This code will be compiled */
#elif MYTOKEN == bar
/* This code won't */
#endif

#if OTHERTOKEN > 40
/* Will be compiled. */
#endif

#if OTHERTOKEN != 42
/* Won't be compiled. */
#endif
</syntaxhighlight>

The ''#if'' directive also allows for a simple construct to disable a region of code without having to worry about nested ''/* ... */'' style comments:

<syntaxhighlight lang="c">
#if 0
/* disabled code */
#endif
</syntaxhighlight>

Such code can easily be re-enabled temporarily with no more effort than replacing the "0" with a "1". Source comments as to why you disabled code this way are in order.

== #undef ==
Using the ''#undef'' directive, a preprocessor token can be undefined. This is useful for trickier setups where you might want to redefine a token to a different value: Redefinitions generate a warning message, undefinitions of undefined tokens don't.

This should not be constructed as an advice to ''always'' use ''#undef'' before a ''#define''. Those warnings might actually be pointing to a real problem in your logic. Use ''#undef'' with care.

== Predefined Tokens ==
The preprocessor provides a couple of tokens which are automatically defined to the appropriate values - something very useful when constructing error messages or tracing messages. Note that some obsolete compilers might balk at ''__func__'' and not all tokens may be supported or implemented by all compilers.

{| class="wikitable"
! Preprocessor Token
! Explanation
|-
| __FILE__ || Holds the name of the current source file being compiled (as a string).
|-
| __LINE__ || Holds the current line being compiled (as an integer).
|-
| __DATE__ || Holds the date when the compilation process began (a string with the format "Mmm dd yyyy").
|-
| __TIME__ || Same as the previous, but the time (a string with the format "hh:mm:ss").
|-
| __cplusplus || When defined, the value indicates that C++ compilation is active. When the compiler is (fully) compliant to the standards, the value should be >= 199711L.
|-
| __STDC__ || When defined, the value indicates that the compiler is (fully) compliant with the ANSI C standard.
|-
| __func__ || Holds the name of the function it is used within (as a string).
|}

Different compilers may define extra preprocessor tokens. Visual C++ for example may define _MSC_VER __cplusplus_cli. See the link section below for more information.

== assert() ==
Assertions are used to catch situations which should never happen, even under error circumstances. If the condition given in the parantheses does not evaluate to "true", a diagnosis is printed which contains source file name, line number, and (since C99) name of the current function; the program then calls abort().

<syntaxhighlight lang="c">
#include <assert.h>

assert( sizeof(struct free_memory_block) == 8 );
assert( 1 != 2 );
assert( gdt_ptr != null );
</syntaxhighlight>

For production code, assertions may be turned off by defining NDEBUG:

<syntaxhighlight lang="bash">
gcc -DNDEBUG ...
</syntaxhighlight>

Note that <assert.h> does not have (or need) a header guard, i.e. can be included multiple times in a source file, and that whether NDEBUG is defined or not is evaluated anew ''at every inclusion of <assert.h>''. You can thus enable / disable assertions at a very fine-grained level if necessary:

<syntaxhighlight lang="c">
#include <assert.h>

    /* assert() at this point only fails-on-false if NDEBUG is not defined */
    assert( isChecksumCorrect() );

#ifdef NDEBUG
/* Hard-enabling of assert() even if NDEBUG is defined */
#define NDEBUG_WAS_SET
#undef NDEBUG
#include <assert.h>
#endif

    /* assert() in this block of code should fail-on-false even in production */
    assert( isChecksumCorrect() );

#ifdef NDEBUG_WAS_SET
/* Restoring NDEBUG if it was enabled originally */
#define NDEBUG
#include <assert.h>
#endif
</syntaxhighlight>

== See also ==
=== Articles ===
* [[C]]
* [[Why function implementations shouldn't be put In header files]]

=== External Links ===
* [http://gcc.gnu.org/onlinedocs/cpp/Macro-Pitfalls.html#Macro-Pitfalls Macro Pitfalls] - A number of counter-intuitive consequences of macros and macro expanding design.
* [http://gcc.gnu.org/onlinedocs/cpp/ The GNU C preprocessor manual]
* [http://msdn.microsoft.com/en-us/library/b0084kay(VS.80).aspx VC++ preprocessor information]

[[Category:C]]
[[Category:C++]]
[[Category:Tutorials]]
````

<a id="w002355"></a>

## W002355 — Device Management

Bron-ID: W002355
Namespace: Main (0)
Revisie: 28406 | Datum: 2023-10-18T07:43:32Z
Bron: https://wiki.osdev.org/Device_Management
Revisie-URL: https://wiki.osdev.org/index.php?title=Device_Management&oldid=28406

````text
All modern operating systems have a subsystem called the device manager. The device manager is responsible for detecting and managing devices, performing power management, and exposing devices to userspace. Since the device manager is a crucial part of any operating system, it's important to make sure it's well designed.

==Device Drivers==
Device drivers allow user applications to communicate with a system's devices. They provide a high-level abstraction of the hardware to user applications while handling the low-level device-specific I/O and interrupts. Device drivers can be implemented as loadable kernel modules (for a [[Monolithic Kernel]]) or user-mode servers (for [[Microkernel|Microkernels]]).

==Device Detection==
The main role of the device manager is detecting devices on the system. Usually, devices are organized in a tree structure, with devices enumerating their children. Device detection should begin with a "root bus driver". On x86 systems, the root bus driver would use [[ACPI]]. The root bus driver sits at the root of the device tree. It detects the buses present on the system as well as devices directly connected to the motherboard. Each bus is then recursively enumerated, with its children continuing to enumerate their children until the bottom of the device tree is reached.

Each device that is detected should contain a list of resources for the device to use. Examples of resources are I/O, memory, IRQs, DMA channels, and configuration space. Devices are assigned resources by their parent devices. Devices should just use the resources they're given, which provides support for having the same device driver work on different machines where the resource assignments may be different, but the programming interface is otherwise the same.

Drivers are loaded for each device that's found. When a device is detected, the device manager finds the device's driver. If not loaded already, the device manager loads the driver. It then calls the driver to initialize that device.

How the device manager matches a device to a device driver is an important choice. The way devices are identified is very bus specific. On [[PCI]], a device is identified through a combination of its vendor and device IDs. [[USB]] has the same scheme as PCI, using a vendor and product ID. [[ACPI]] uses PNP IDs to identify devices in the ACPI namespace. With this information, it's possible to build a database using matching IDs to drivers. This information is best stored in a separate file.

==Inter-Process Communication (IPC)==
The device manager needs to implement some form of IPC between it and device drivers. IPC will be used by the device manager to send I/O requests to device drivers, and by device drivers to respond to these requests. It is usually implemented with messages that contain data about the request, such as the I/O function code, buffer pointer, device offset, and buffer length. To respond to these I/O requests, every device driver needs dispatch functions used to handle each I/O function code. Each device needs a queue of these IPC messages for it to handle. On Windows NT, this IPC is done with I/O Request Packets.

==Asynchronous I/O==
There are two main types of I/O: synchronous I/O and asynchronous I/O. Synchronous I/O sends an I/O request and then puts the current thread to sleep until the I/O completes. Asynchronous I/O just sends the I/O request and then returns. I/O completion is reported asynchronously using a callback. Asynchronous I/O improves the efficiency of the system by allowing allowing for the program execution to continue while I/O is performed. It also allows for multiple I/O requests to be started and then handled in the order they complete, not the order they execute. However, this comes at the cost of making programming more complex than using synchronous I/O.

Internally, an operating system should use asynchronous I/O for all of its I/O requests. I/O requests are sent to drivers, and then the function that sent them immediately returns. Eventually, the I/O request will be handled. Once it completes, it returns through the driver stack and finally notifies the application of I/O completion. It can do this using callbacks, signals, or completion queues.

Synchronous I/O can simply be implemented as a special case of asychronous I/O. Just like with asynchronous I/O, an I/O request is sent to the driver, but instead of returning, the thread goes to sleep. Once the I/O completion event is queued, the thread will wake up and execute the callback before returning.

==Power Management==
The device manager also performs power management. Power management is a feature of hardware that allows for the power consumption of the system and devices to be controlled. Each device managed by the device manager should provide functions to set their power state. For power management support, all systems require a power management driver that controls the system power. On x86, this is done through [[ACPI]]. Each device also needs to support power management.

The device manager needs to respond to power management events. Power management events can come from two sources: the user or the system. User-generated power management events are created by user mode applications. They are system-wide events for shutting down, rebooting, hibernating, or putting the system to sleep. When the device manager receives a system-wide power management event, it sets the power state of the system.

System-generated power management events are events that come from the system hardware. Examples of system-generated power management events are plugging/unplugging an AC adapter or closing/opening the lid of a laptop. The device manager takes the appropriate action in response to the event.

==Userspace Exposure==
Once the kernel interfaces for device drivers are complete, one also needs to figure out how to expose devices to userspace. Most Unix-based operating systems expose devices through the filesystem tree. When devices are placed in the filesystem tree, there is a directory (usually /dev) containing special files that represent devices. The advantage of placing devices in the filesystem tree is that devices can be treated as files, meaning they can be read from or written to. Windows NT does not expose devices through the filesystem tree. Instead, there is an internal namespace of objects, through which devices can be found and accessed similarly to files.

No matter how devices are exposed, the functions that are provided for devices must be decided on as well. Both Unix-based operating systems and Windows NT treat devices like files, meaning their functions are open(), close(), read(), and write(). However, it was soon realized that this API would not be adequate for device functions that don't fit into these functions, like setting the graphics mode of a video card. For this purpose, a new syscall called ioctl() was developed, that allows a device to have special functions. However, this is by no means the only way to call device functions.

==Existing Driver Interfaces==
An operating system doesn't need to implement its own driver interface. A few driver interfaces have already been programmed with the intent of being integrated into operating systems. These driver interfaces can be implemented instead of a native driver interface, on top of a native driver interface, or along with a native driver interface.

===Uniform Driver Interface===
{{Main|Uniform Driver Interface}}

Project UDI is a driver interface intended to be binary portable or source portable when running on different CPU architectures. It is not very widespread (however, neither are EDI or CDI); for example, due to philosophical concerns, Linux did not embrace UDI. However, several members of the community are striving to popularize UDI again since it would be of a huge benefit to hobby operating systems. You are strongly encouraged to participate by implenting a UDI environment and writing drivers.

===Extensible Driver Interface===
{{Main|Extensible Driver Interface}}

EDI is a driver interface intended to be source code portable and fairly simple in implementation (also, limited in functionality), so that hobby small hobby OSes may share driver code base.

===Common Driver Interface===
{{Main|Common Driver Interface}}

The Common Driver Interface is a German project made by the people behind Tyndur, a community project of LowLevel, the German equivalent of the OSDev wiki. The interface offers less functionality than UDI but is a lot easier to implement. The creators have also implemented the interface in their own OS and they have made a nice amount of drivers.

==See Also==

===External Links===

*[http://www.osdever.net/tutorials/view/device-management Tim Robinson's Device Management Tutorial]

[[Category:Kernel]]
[[Category:Drivers]]
[[Category:Driver Interfaces]]
````

<a id="w002356"></a>

## W002356 — Talk:Semaphore

Bron-ID: W002356
Namespace: Talk (1)
Revisie: 20318 | Datum: 2017-02-03T11:19:17Z
Bron: https://wiki.osdev.org/Talk:Semaphore
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ASemaphore&oldid=20318

````text
counting semaphores: ", the process waits until the variable is non-zero, and then atomically decrements it before returning." shouldn't it be "the variable is positive" ? [[User:Adek336|Adek336]] 01:43, 20 July 2008 (UTC)

Ten years and no-one noticed this code is wrong.  I'll update the example when I get an opportunity to work out the specifics.  There needs to be another Semaphore around the reading and updating of "last_read".  Example: last_read is 7, messages is 3.  Two threads can enter get() at the same time, neither block on the wait() as there are pending messages.  Thread 1 reads queue[7] and is interrupted.  Thread 2 then reads queue[7] and increments last_read to 8 and continues.  Thread 1 then increments last_read to 9 and continues.  queue[7] was read twice and queue[8] was skipped completely.  [[User:Kenny|Kenny]] 05:19, 3 February 2017 (CST)
````

<a id="w002357"></a>

## W002357 — Category:Security

Bron-ID: W002357
Namespace: Category (14)
Revisie: 6072 | Datum: 2008-07-28T18:40:17Z
Bron: https://wiki.osdev.org/Category:Security
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ASecurity&oldid=6072

````text
Security-related pages are categorized here.

[[Category:OS theory]]
````

<a id="w002358"></a>

## W002358 — IRQ

Bron-ID: W002358
Namespace: Main (0)
Revisie: 6076 | Datum: 2008-07-28T19:38:25Z
Bron: https://wiki.osdev.org/IRQ
Revisie-URL: https://wiki.osdev.org/index.php?title=IRQ&oldid=6076
Doorverwijzing: Interrupts

````text
#REDIRECT [[Interrupts]]
````

<a id="w002359"></a>

## W002359 — Template talk:File formats

Bron-ID: W002359
Namespace: Template talk (11)
Revisie: 12910 | Datum: 2012-03-05T19:35:36Z
Bron: https://wiki.osdev.org/Template_talk:File_formats
Revisie-URL: https://wiki.osdev.org/index.php?title=Template_talk%3AFile_formats&oldid=12910

````text
I notice you categorize each format as either "Microsoft/*nix/Other", there is nothing explicitly tying any of those formats to a particular OS, the Sony PSP for example uses ELF. --[[User:Brynet-inc|Brynet-inc]] 23:52, 28 July 2008 (UTC)
: I agree. The [[Wikipedia:Executable|Wikipedia]] doesn't group the executable formats in any way. We could remove the groups. [[User:Virtlink|Virtlink]] 10:15, 29 July 2008 (UTC)
````

<a id="w002360"></a>

## W002360 — File:Rtl8139.JPG

Bron-ID: W002360
Namespace: File (6)
Revisie: 6102 | Datum: 2008-08-01T11:20:52Z
Bron: https://wiki.osdev.org/File:Rtl8139.JPG
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ARtl8139.JPG&oldid=6102

````text
Realtek RTL8139 NIC (PCI) - Image by Biotechnik
````

<a id="w002361"></a>

## W002361 — Talk:Licensing

Bron-ID: W002361
Namespace: Talk (1)
Revisie: 6246 | Datum: 2008-08-27T06:08:30Z
Bron: https://wiki.osdev.org/Talk:Licensing
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ALicensing&oldid=6246

````text
I think there is a three-clause BSD license now popular without the advertising clause. [[User:Adek336|Adek336]] 21:16, 1 August 2008 (UTC)

In addition to there being a three-clause BSD license as mentioned by Adek336, there is also a 2 line BSD-like license called the ISC license. The text of that license can be found [http://opensource.org/licenses/isc-license.txt here]. IMO, this is the closest you can get to Public Domain in countries that don't honor the Public Domain concept. --[[User:Quok|quok]] 14:40, 26 August 2008 (UTC)

:You could scrap the part "provided that this copyright clause...", which could get into the way if you want to use a modified version of the code under a different license. But nice find nevertheless.

:I admit I had forgotten about this page... -- [[User:Solar|Solar]] 15:03, 26 August 2008 (UTC)

:: I'm not entirely certain if you can scrap the "provided that this copyright clause..." part from the license, since the text makes reference to "the author" and without the copyright clause you'd have no way of knowing who the author is. Perhaps if you just included an "authored by" line? I'm not sure how keeping the copyright clause could get into the way of using that code in a modified form, even in a proprietary project. The ISC license doesn't allow you to relicense the original work (which you'd get with real public domain), but you're free to put any modifications to the code under a different license, like the GPL. As pointed out in this article, you can take BSD and ISC licensed code out of a (L)GPL project and not be affected by the (L)GPL. Provided of course that you only took the original BSD/ISC licensed code and none of the extensions. Also, the ISC license does not require the source be made available. I also believe the "and/or distribute" part makes ISC licensed code safe to use in a proprietary project where redistribution is not allowed. Of course, IANAL. --[[User:Quok|quok]] 16:49, 26 August 2008 (UTC)

Quok's post [http://forum.osdev.org/viewtopic.php?p=135228#p135228 here] should IMHO also be included on this page. It's about using GCC header files when not compiling with GCC. [[User:Virtlink|Virtlink]] 18:58, 26 August 2008 (UTC)

:Perhaps the entire thread should be linked to. All the posts about the GCC header files and the GPL exception should be summarized and included in this page though, not just my post. I'd do it myself but this page really seems to be Solar's baby even if he did forget about it. :) --[[User:Quok|quok]] 06:08, 27 August 2008 (UTC)
````

<a id="w002362"></a>

## W002362 — Ethernet devices

Bron-ID: W002362
Namespace: Main (0)
Revisie: 26434 | Datum: 2021-09-30T20:20:39Z
Bron: https://wiki.osdev.org/Ethernet_devices
Revisie-URL: https://wiki.osdev.org/index.php?title=Ethernet_devices&oldid=26434

````text
== Network Cards ==
There are many different network cards. Each one has specific documentation.

=== 3C90x ===
3com 90x, 90xB, 90xC are PCI ethernet adapters which provide full bus-master operation, download/upload packet chains. All of them allow auto-negotiation. 90xB and 90xC have power management capabilities. Many 3com cards share the programming interface with these models, an example is the [[3c556]] 10/100 LAN + 56k Modem MiniPCI adapter, 3c555 10/100 LAN MiniPCI.

=== DP83900/DP83901SNIC/DP83902SNIC/DP83905AT-LANTIC. ===
The [[DP8390|DP83900]], [[DP8390|DP82901/SNIC]], [[DP8390|DP82902/SNIC]], and [[DP8390|DP82905 AT/LANTIC]] are network cards that have the same general functionality and only minor differences in there system level operation.

=== NE2000 ===
The [[Ne2000]] network card chipset was a reference design that was never meant to go into mainstream production, but after Novell used it as cheap hardware to go with their software it became popular.
It is a good first network card to program because it follows a simple design (making it helpful for learning), they're probably dirt-cheap, and it is supported by most PC [[emulators]]. Both [[Bochs]] and [[QEMU]] provide ISA and PCI implementations.

[[Ne2000]] is not technically a card, it is a standard that several implementers follow. The best available description of the initial standard is located [http://www.national.com/pf/DP/DP8390D.html DP8390D/NS32490D NIC Network Interface Controller] and was published by National Semiconductor.

=== RTL8139 ===
The [[RTL8139]] Network Chip is used on many old Ethernet Network Devices. It supports 10 and 100 MBit. 

=== RTL8169 ===
The [[RTL8169]] is Realtek's next generation of high-performance network cards. This particular chipset is designed to operate at 10/100/1000 Mbps speeds.

=== Broadcom BCM57xx card family ===
There are many ethernet cards from Broadcom, but, fortunately, they are very similar. Specifications are available on Broadcom page [https://docs.broadcom.com/doc/1211168564335 BCM57785] [https://docs.broadcom.com/doc/5718-PG108-R BCM5718]

[[Category:Network Hardware]]
````

<a id="w002363"></a>

## W002363 — Where Can I Find Information About Ports

Bron-ID: W002363
Namespace: Main (0)
Revisie: 26316 | Datum: 2021-08-12T23:52:57Z
Bron: https://wiki.osdev.org/Where_Can_I_Find_Information_About_Ports
Revisie-URL: https://wiki.osdev.org/index.php?title=Where_Can_I_Find_Information_About_Ports&oldid=26316

````text
{{Tone}}
{{FirstPerson}}
== Question: Where Can I Find Information About Ports? ==
<div style="font-size:8pt">
: I have been trying to code a kernel (starting from Brandon's tutorial at osdever). I was lucky to find the ports to display data on screen, to read from a sector, to read the time ([[CMOS]]) and to reboot. Where can I find more information? I want to get into graphics mode, so that I can display images.

: The long version: So, I started with a [[Tutorials|tutorial]], writing hello world was easy. Earlier I had worked on [[Assembly|assembly]] and wrote a simple text-drawing program using [[BIOS]] [[Interrupts|interrupts]]. But, this time I faced the [[kernels|kernel]] stuff, and it used [[ports]] to display data ("Hello World"). I have yet to read the [[Documentation|documents]] properly to appreciate everything. But, anyways I went on searching and I was able to replace "Hello World" with some data on the disk, which required the knowledge of [[LBA]]. I had stumbled on cylinders etc ... the earlier way of reading sector which did not work for me. After knowing the [[LBA]] ports, and going through [[Ext2]] [[Documentation]], I was able to read a small file on the disk. Later, going through other people's code, I came across the code to access [[CMOS]] time and how to reboot.

: But, now when I want to add [[Shutdown|shutdown]], [[How_do_I_set_a_graphics_mode|graphics]] mode, [[networking|network]] and other things, its getting a little painful. I feel the need of a guide.

: Kindly help!
: Thank You.</div>



Hi,

In ''general'', all I/O ports are either part of a legacy/[[ISA]] device ([[Serial_ports|serial ports]], [[Parallel_port|parallel ports]], [[PS2_Keyboard|PS/2 keyboard]], [[Floppy_Disk_Controller|floppy]], [[CMOS]]) and/or part of the chipset, or part of a [[PCI]] device. Regardless of what it is you need the documentation for the specific thing (for e.g. for "XYZ ethernet card" you'd need the documentation for "XYZ ethernet card", and the documentation for "ABC ethernet card" will be useless).

===== Legacy Hardware And Backwards Compatibility =====
In some cases there's a legacy hardware interface for backward compatibility - for e.g. most video cards support the old [[VGA]] I/O ports (until you put them into "native mode" so you can actually use most of it's features). These legacy interfaces are probably worth supporting until you've got proper device drivers. In this case you need the documentation for the legacy device (rather than the documentation for the correct/actual device). For e.g. for a "top of the line" ATI video card you'd want IBM's original [[VGA]] documentation (or equivalent information).

===== Legacy Software Interfaces =====
Also, for some devices there's a legacy software interface. This is mostly limited to video cards though (the [[VGA_Resources|VESA/VBE]] interface) because the rest is too crappy{{how}} (no sane person uses the BIOS for [[Serial_ports|serial ports]], [[Parallel_port|parallel ports]], [[PS2_Keyboard|PS/2 keyboard]], [[Floppy_Disk_Controller|floppy]], or [[ATA_PIO_Mode|hard drives]]{{why}} even though it's possible in theory). Like the "legacy hardware interface", this is mostly just a short-term solution (basic functionality with no extra features and poor performance{{how}}).

===== Standard Interfaces =====
Lastly, for some device types there's a standard interface. This includes [[USB]] controllers (AFAIK there's only 4 different standards for this - [[UHCI]], [[EHCI]], [[XHCI]] and [[OHCI]]) and [[IDE]]/[[ATAPI]] hard disk controllers (but not [[SCSI]] controllers), and not much else. These standard interfaces are actually good interface (not legacy interfaces).

===== Supporting Devices Is Difficult =====
Mostly, to support all devices properly (without using legacy interfaces) you need to read through thousands of pieces of documentation and write thousands of different device drivers (and no, there are not thousands of guides, one for each device, or any other "hand holding" - you need to find, read and understand the manufacturer's documentation).

===== Keep It Minimal =====
Fortunately (IMHO), for a good OS design you don't actually need to write many device drivers{{according_to_who}} (a few common drivers to get things started perhaps). You only really need to design, implement and document suitable device driver interface/s, so that other programmers can easily write the device drivers later. For example, you might write one device driver for one [[ethernet]] card, and (hopefully, one day) twenty more people might use your [[documentation]] (and your first device driver, as a reference) to implement fifty more device [[drivers]] for fifty more [[ethernet]] cards.

Basically what I'm saying is that (IMHO) sane OS developers don't actually write an OS. Instead they write [[Bootloader]]s, [[kernels]], and [[documentation]]. Only after the [[kernels|kernel]] and [[documentation]] are entirely complete do they worry about device [[drivers]], [[File_Systems|file systems]], [[applications]], etc (except for some common/special case stuff that they need to test the kernel and the kernel's interfaces).

== Question: Shutdown, Graphics, Network, and Others ==
<div style="font-size:8pt">
: But, now when I want to add shutdown, graphics mode, network and other things, its getting a little painful.
</div>

===== Shutdown (APM or ACPI) =====
For shutdown there's 2 different "legacy software interfaces" ([[APM]] and [[ACPI]]). [[APM]] is old but simple, while [[ACPI]] is new but a huge over-complicated pain in the neck. To do it without these legacy software interfaces you'd need to see the documentation for each motherboard/chipset.

===== Video (BIOS, VESA/VBE, VGA Interface, or Native) =====
For video there's 2 legacy software interfaces (the original [[BIOS]] Int 0x10 interface, and the [[VGA_Resources|VESA/VBE]] extensions), plus one legacy hardware interface (the "[[VGA_Resources|VGA standard]]"). To do it properly you need programming information for each video card (which can be very difficult to obtain in some cases). Even if you do have good information a modern video a device driver is complex (especially if you start looking at 3D special effects and implementing a JIT compiler for shader language - e.g. similar features and performance to DirectX 10), and it could easily take over a year to implement a good device driver for one video card.

===== Network Cards =====
For [[:Category:Network Hardware|network cards]] there is no legacy software interface and no legacy hardware interface - to do it properly you need the manufacturer's documentation (or equivalent documentation).

===== Source =====
The information contained here was roughly compiled from a post on the forums here, http://forum.osdev.org/viewtopic.php?f=15&t=17704.

[[Category:FAQ]]
````

<a id="w002364"></a>

## W002364 — C--

Bron-ID: W002364
Namespace: Main (0)
Revisie: 31021 | Datum: 2026-10-14T12:13:26Z
Bron: https://wiki.osdev.org/C--
Revisie-URL: https://wiki.osdev.org/index.php?title=C--&oldid=31021

````text
This page is about [https://c--sphinx.narod.ru/indexe.htm Sphinx Full C--], not about C--, the portable assembly language.

Sphinx Full C-- language is in between the [[C|C language]] and x86-assembly. It is a very neat computer language and the Sphinx Full C-- compiler allows to produce (and mix) 16 bit and 32 bit code for real and protected modes. 

It was originally developed by Peter Cellik in Canada and then has been extended for several years by Michael Sheker in Russia. 

==Pros==
* compiles very small executables;
* good for writing real mode x86 operating systems;
* good for writing TSR programs for DOS;
* supports many executable formats: COM, EXE, SYS (DOS drivers), ROM-BIOS, Windows PE-Coff, Windows DLL, 32bit DOS flat or DPMI, MenuetOS;
* good for writing x86-related parts of an operating system in a structured and easily readable and easier debuggable way related to structured languages like C and Pascal;
* very good (because of the structured form) for writing small toy programs which are close to the hardware;
* very good for writing bootsectors in;
* the code can be either low level (assembly-like), medium level (C like) or very easily mixed;
* has almost as good control over all the small details of the output binaries as assembly languages provide, but is much more maintainable thanks to structuring; people develop who develop OSes in pure assembly do not need the high-level structures and portability of C and C++, so this means that for many developers, the lack of these high level structures and portability in C-- may not be an issue;
* relatively easy to port from and to C-with-assembly.

==Cons==
* not portable; it is very closely tied to the x86 assembly;
* is developed by few developers, the community is small; has less users than gcc so may have more bugs;
* in order to be low-level, many limitations are in place: for example, array subscripts (like in x= myArr['''subscript''']) are limited to simple expressions (like myArr[10], myArr[ESI]; but myArr[myArr[1]] is not allowed);
* it can not output 64bit code.

Early versions of [[User:Adek336|panaLiX]] used Sphinx Full C--. The Panalix website is [http://panalix.sourceforge.net/ here] but there is no Sphinx C-- code currently available.

==Examples==
     byte x = 0b01001;
     word y;
     dword z;
     #define THIS_A_DEFINITION 123

===Take over interrupt===
 dword old0x21 = {};
 interrupt int0x21()
 {
     $PUSH DS
     $PUSH ES
     $PUSHA
     $PUSH FS
     //
     DS = CS;
     FS = 0xb800;
     FSBYTE[1682] = '!';
     //
     $POP FS
     $POPA
     $POP ES
     $POP DS
 }
 ...
 int registerInterrupt0x21()
 {
     FS = 0;
     old0x21 = FSDWORD[0x21 * 4];
     FSWORD[0x21 * 4] = #int0x21;
     FSWORD[0x21 * 4 + 2] = CS;
     Log(3, "0x21 registered", 0);
 }

     ESBYTE[0x1b8003]= 'z';
     //
     $ CLI
     kb_cmd(0xad);
     kb_cmd(0xd0);
     kb_read();
     tepx=AL;
     kb_cmd(0xd1);
     $ MOV AL, tepx
     $ OR AL, 2
     $ MOV tepx, AL
     kb_write(tepx);
     kb_cmd(0xae);
     $ STI
     //
     ESBYTE[0x1b8005]= 127;

===read, write===
 void PRINTF()
 {
     BX = 0x0007;
     $ PUSH SI
     SI = AX;
     AH = 0xE;
  UP:
     $ LODSB
     $ CMP AL,0
     $ JZ DOWN
     $ INT 0x10
     $ JMP SHORT UP
  DOWN:
     $ POP SI
 }
 void readStr()
 int tr;
 byte b1; word ln;
 {
  tr = 0; ln = 0;
  do
   {
    tr = 0;
    b1 = wait_key();
    if (b1 == 13) tr = 1;
    if (scanCd == 14)
     {
      tr = 2;
      if (ln > 0)
       {
        putch(b1); putch(' '); putch(b1);
        ln--;
       }
     }
    if (tr == 0) if (ln < 255) if (b1 >= 32)
     {
      putch(b1);
      BX = ln;
      st1[BX] = b1;
      ln++;
     };
   } while (tr <> 1);
  BX = ln;
  st1[BX] = 0;
 }

 : void kbrSpeed(byte sped, dly)
   {
    AH = 3;
    AL = 5;
    BL = sped; // 0x00 - 0x1f
    BH = dly;  // 0 - 3
    $ INT 0x16
   }

===Disk IO===
 byte readDisk(byte drive, head, sector, track, howmany; word segm, offs)
 int err;
 {
  err = 0;
  lab1:
  $ MOV AH, 0
  $ MOV DL, drive
  $ INT 0x13
  $ MOV AX, segm
  $ MOV ES, AX
  $ MOV BX, offs
  $ MOV DL, drive
  $ MOV DH, head
  $ MOV CL, sector
  $ MOV CH, track
  $ MOV AL, howmany
  $ MOV AH, 2
  $ INT 0x13
  AL = 0;
  if (CARRYFLAG)
   {
    err++;
    if (err < maxDiskErrors) $jmp lab1;
    return(AH);
   }
  return(0);
 }
 byte readSector(byte drive; word sector; byte howmany;word segm, offs)
 word a, b, c, d;
 {
  a = sector - 1;
  c = a % 36;
  d = c % 18;
  c = c - d;
  c = c / 18;
  b = a % 18 + 1;
  d = a % 36;
  a = a - d;
  a = a / 36;
  AL = readDisk(drive, c, b, a, howmany, segm,offs);
 }
 byte readCluster(byte drive; word clnum; byte howmany; word segm, offs)
 {
  AL = readSector(drive, clnum+32, howmany, segm, offs);
 }
 int fileCluster(word tstart)
 byte fbuf[512];
 byte fname[11];
 int n1, n2, n3, n4, w5;
 byte b1;
 byte f[11];
 {
  $ PUSH AX
  $ PUSH BX
  $ PUSH CX
  $ PUSH SI
  lastSize = 0;
  w5 = 0;
  SI = tstart;
  n1 = 11;
  loop (n1)
   {
    $ LODSB
    CX = #fname +11 - n1;
    BX = CX;
    DSBYTE[BX] = AL;
   }
  $ POP SI
  n3 = -1;
  n1 = 20;
  do
   {
    n2 = 0;
    if (w5 == 0) w5 = - readSector(0, n1, 1, DS, #fbuf);
    if (w5 <> 0) n1 = 35;
    if (w5 == 0)
    do
     {
      n3 = 0;
      n4 = 0;
      do
       {
        AX = 32 * n2 + #fbuf + n3;
        BX = AX;
        b1 = DSBYTE[BX];
        AX = #fname + n3;
        BX = AX;
        if (DSBYTE[BX] <> b1) n4 = 1;
        n3++;
       } while (n3 <= 10);
      AX = n2 * 32 + #fbuf + 28;
      BX = AX;
      if (n4 == 0) lastSize = DSDWORD[BX];
      AX = n2 * 32 + #fbuf + 26;
      BX = AX;
      if (n4 == 0) w5=DSWORD[BX];
      n2++;
     } while (n2 <=15);
    n1++;
   } while (n1 <= 33);
  $ POP CX
  $ POP BX
  $ POP AX
  AX = w5;
 }

===Enable Protected Mode===
  ...
  ?code32 false
  ...
  DSWORD[#gdtr] = #gdt_end - #gdt0;
  DSWORD[#gdtr] = DSWORD[#gdtr] - 1;
  DSDWORD[#gdtr + 2] = START + #gdt0;
 //
  DSWORD[#idtr] = #idt_end - #idt;
  DSWORD[#idtr] = DSWORD[#idtr] - 1;
  DSDWORD[#idtr + 2] = START + #idt;
 //lgdt bx
  BX = #gdtr;
  $ db 0x3e, 0x0f, 0x01, 0x17
 //lidt bx
  BX = #idtr;
  $ db 0x3e, 0x0f, 0x01, 0x1f
 //
  $ MOV EAX, CR0
  $ OR AL, 1
  $ MOV CR0, EAX
 //
 jump_pm32:
  $ db 0xEA
  $ dw 0
  $ dw 0x10
  }
 ?code32 TRUE
 void _1()
 {
 do_pm32:
 //
  $ MOV AX, 0x18
  $ MOV DS, AX
  $ MOV SS, AX
 //
  $ MOV AX, 0x08
  $ MOV ES, AX
 //
  @setup_pic(0x20, 0x28);
  ...

These snippets may seem a bit spaghetti code, but not because of some language defficiency but because I didn't look very far for them snippets.

==References==
* [http://c--sphinx.narod.ru/indexe.htm The compilers website]
* [http://www.dev0.de/cmm/indexe.htm Mirror of the compilers website]
* [https://github.com/jossk/c--sphinx Github site with all the files plus source for Sphinx C--]

[[Category:Languages]]
````

<a id="w002365"></a>

## W002365 — 3c556

Bron-ID: W002365
Namespace: Main (0)
Revisie: 6151 | Datum: 2008-08-04T20:55:22Z
Bron: https://wiki.osdev.org/3c556
Revisie-URL: https://wiki.osdev.org/index.php?title=3c556&oldid=6151
Doorverwijzing: 3c90x

````text
#REDIRECT [[3c90x]]
````

<a id="w002366"></a>

## W002366 — Talk:Hdiutil

Bron-ID: W002366
Namespace: Talk (1)
Revisie: 6200 | Datum: 2008-08-15T23:18:03Z
Bron: https://wiki.osdev.org/Talk:Hdiutil
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AHdiutil&oldid=6200

````text
Should the "Creating Images" section be moved over to [[Loopback_Device]]? it seems fitting.--[[User:Brynet-inc|Brynet-inc]] 23:18, 15 August 2008 (UTC)
````

<a id="w002367"></a>

## W002367 — Loading a Process

Bron-ID: W002367
Namespace: Main (0)
Revisie: 28561 | Datum: 2024-01-16T16:00:45Z
Bron: https://wiki.osdev.org/Loading_a_Process
Revisie-URL: https://wiki.osdev.org/index.php?title=Loading_a_Process&oldid=28561

````text
To load and start a process can be done in a number of ways.

== Loader Function ==
In [http://forum.osdev.org/viewtopic.php?p=112592#p112592 this post] Brendan describes a way where the created process executes a loader function which loads the executable into it's address space. This loader function is only active when the process gets it's timeslice. This way, the process spawning a new process doesn't get blocked.

* Create a new page directory (mapped into kernel space);
* Copy kernel page tables into the new page directory;
* Setup scheduler data (thread priority, etc...) and set the initial EIP to a "start process" kernel function;
* (Optionally) wait or block until the new thread sets some sort of status to return;
* Return to caller.

After this I've got a new thread/process running in kernel space. When the scheduler gives it a time slice, the kernel sets up "user space" (loads an executable file, etc), and then "returns" to CPL=3. All of this happens from within the new address space though, so there's no hassles with paging, etc.

For error handling, for my OS everything is asynchronous - the spawning thread doesn't block waiting for status from the spawned thread. Just before the kernel "returns" to the new process/thread it sends a status/error message to the first thread. This means the "spawnThread()" function might return "OK" and you might be sent an error (e.g. "failed to load executable file") afterwards.

For synchronous OSs, the "spawnProcess()" function would block until full status can be returned. In theory this is easier for applications programmers to use but worse for performance. For example, a user interface might lock up for several seconds because the thread decided to spawn a process (e.g. where the executable file for the new process happens to be a large file on a slow file system). To avoid this problem application programmers might want to spawn a new thread that spawns the new process; but this makes it harder to use than an asynchronous function, has higher overhead, and most application programmers won't do it (it's either too much hassle, or they simply don't realise the problem is there).

== Process Manager ==
A Process Manager is used to load the executable in AndrewAPrice's [http://forum.osdev.org/viewtopic.php?p=112605#p112605 post].

The process manager runs in it's own thread. Usually this thread is sleeping, but if a job gets added to the process manager's queue ("Spawn this process" - that's all it's used for right now), the process manager's thread is sleeping. The process that sent the request usually sleeps until the process manager has completed it's job - but it's possible in a multi threaded application to do other stuff while the process is loading.

When my scheduler switches to one of these special kernel threads (the process manager, the vfs manager, etc) it actually switches to the memory context of the process it is working in. That way, really large requests (e.g. load 20MB from a file) is actually processing in a separate kernel thread with the data being loaded into the process's address space, allowing the process and all other processes to continue doing what ever it wants to do.

This does have a (fairly insignificant) performance penalty. For example, to load a program into memory the system has to do something along the following lines:

* Parent program requests program to be loaded.
* - JUMP TO KERNEL LAND -
* Kernel adds request to process manager's queue. Wake process manager.
* - JUMP TO USER LAND -
* Parent program requests to go to sleep.
* - JUMP INTO KERNEL LAND -
* Remove program from 'awake' list.
* - WAIT FOR CONTEXT SWITCH INTO PROCESS MANAGER -
* Set up new process's memory. Request file from VFS manager. Wake VFS manager. Send process manager to sleep.

begin of loop:
* - WAIT FOR CONTEXT SWITCH INTO VFS MANAGER -
* Request part of the file from the disk driver. Wake disk driver. Send VFS to sleep. (Requests are broken up into 128kb chunks, so multiple read/writes can occur at the same time).
* - WAIT FOR CONTEXT SWITCH INTO DISK DRIVER -
* Load part of file from disk. Wake VFS.
if not fully loaded go back to beginning of loop (VFS will only request parts of the file, e.g. the ELF header first, then the program headers, then the data, etc).

* Wake process manager.
* - WAIT FOR CONTEXT SWITCH INTO PROCESS MANAGER -
* Add new process to scheduling algorithm. Wake parent process. If nothing else to queue, send process manager to sleep.

As you can see, a lot of work is involved. These special threads aren't really part of the kernel and they don't exist in their own address , they sort of just go up to other processes and say "what's up, can I hang with you?"

== Lazy Loading ==
When your OS supports swapping a page in from any file (instead of only the swap file), then you may create an address space where the pages are marked as not present and mapped to the executable file on disk. This way, when the new process starts executing, your memory manager will swap the required pages in from the executable automatically. It only loads those pages that are required at that point, thus it requires less memory at startup.

== Admission Scheduler ==
Some operating systems use a long-term scheduler or admission scheduler which controls when a specific process is started. It is mainly used for real-time operating systems to control the system's load. The idea is similar to the [[Loading a Process#Process Manager|process manager]] approach. If you ever intent to support real-time scheduling, this is the way to go. Even when your OS is first just a simple desktop OS, because then you can code the admission scheduler to automatically load any executable in it's queue.

== References ==
=== Articles ===
* [[Context Switching]]
* [[Processes and Threads]]

=== Threads ===
* [http://forum.osdev.org/viewtopic.php?f=1&t=15622 Loading processes in (new) virtual address space]

=== External Links ===
* [[Wikipedia:Scheduling (computing)#Long-term_Scheduler|Admission Scheduler]]

[[Category:Loaders]]
[[Category:Processes and Threads]]
````

<a id="w002368"></a>

## W002368 — Page directory

Bron-ID: W002368
Namespace: Main (0)
Revisie: 6212 | Datum: 2008-08-20T14:51:19Z
Bron: https://wiki.osdev.org/Page_directory
Revisie-URL: https://wiki.osdev.org/index.php?title=Page_directory&oldid=6212
Doorverwijzing: Paging

````text
#REDIRECT [[Paging#Page Directory]]
````

<a id="w002369"></a>

## W002369 — Page table

Bron-ID: W002369
Namespace: Main (0)
Revisie: 6213 | Datum: 2008-08-20T14:51:46Z
Bron: https://wiki.osdev.org/Page_table
Revisie-URL: https://wiki.osdev.org/index.php?title=Page_table&oldid=6213
Doorverwijzing: Paging

````text
#REDIRECT [[Paging#Page Table]]
````

<a id="w002370"></a>

## W002370 — ISR

Bron-ID: W002370
Namespace: Main (0)
Revisie: 6217 | Datum: 2008-08-20T15:34:36Z
Bron: https://wiki.osdev.org/ISR
Revisie-URL: https://wiki.osdev.org/index.php?title=ISR&oldid=6217
Doorverwijzing: Interrupt Service Routines

````text
#REDIRECT [[Interrupt Service Routines]]
````

<a id="w002372"></a>

## W002372 — Talk:Bran's Kernel Development Tutorial Known Bugs

Bron-ID: W002372
Namespace: Talk (1)
Revisie: 24788 | Datum: 2020-06-18T13:00:27Z
Bron: https://wiki.osdev.org/Talk:Bran%27s_Kernel_Development_Tutorial_Known_Bugs
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABran%27s_Kernel_Development_Tutorial_Known_Bugs&oldid=24788

````text
I was going to change where the .rodata section is, but figured I'd ask for opinions first. Currently we're telling newbies to add the .rodata section to .text, but IMO that's a horrible place to put it. It should be put in the .data section (whether before or after .data doesn't matter), as static strings ARE data, and by placing them in .data they wouldn't be marked as executable and that's always a good thing. --[[User:Quok|quok]] 16:32, 27 August 2008 (UTC)
:I agree and disagree: I agree it shouldn't go in .text, as I do not want to cause some bug in my code to start executing my static strings. .rodata is read-only, like .text, but it's not executable and should therefore be marked as such. Placing .rodata in .text predates the NX bit, since without it there's not much difference. But I disagree it should go in .data, since .data is writeable and .rodata should not be, by definition. Therefore I think .rodata should go in it's own section, *and* in its own segment so that the segment where the .text section is in can be marked read-only, execute, the segment of .rodata can be marked read-only, no execute, and the .data segment can be marked read-write, no execute. [[User:Jal|Jal]] 10:44, 24 November 2009 (UTC)

== strange placements ==

I'm at lost why the linked script uses these strange AT placements, combined with a . = ALIGN(), instead of just an ALIGN() in place of the AT. So something like this (ignoring the .rodata problems above):

<code>
SECTIONS
{
  .text phys : AT(phys)
  {
    code = .;
    *(.text)
    *(.rodata)
  }
  .data : ALIGN(4096)
  {
    data = .;
    *(.data)
  }
  .bss : ALIGN(4096)
  {
    bss = .;
    *(.bss)
  }
  end = ALIGN(4096);
}</code>

That makes it much more readable, imho. EDIT: does anyone now why it's formatted so weird?

[[User:Jal|Jal]] 10:49, 24 November 2009 (UTC)
````

<a id="w002373"></a>

## W002373 — User:Quok/Silicon Image Datasheets

Bron-ID: W002373
Namespace: User (2)
Revisie: 6252 | Datum: 2008-08-27T19:19:32Z
Bron: https://wiki.osdev.org/User:Quok/Silicon_Image_Datasheets
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AQuok%2FSilicon_Image_Datasheets&oldid=6252

````text
Here is a list of all Silicon Image chips for which there are known publicly available datasheets. Each chip number should be followed by a brief description of the chipset's functionality and a link to the datasheet. These datasheets may not be redistributable, so we only link to them here.


==IDE/PATA Controllers==
* SiI0680A - PCI to 2 Port IDE/PATA
** PCI to 2 Port IDE/PATA
** Manufacturer's Product Page: http://www.sci-worx.com/products/product.aspx?id=31
** Data Sheet URL: http://www.sci-worx.com/docs/SiI-DS-0069-C.pdf
<br />
==SATA Controllers==
* SiI3112
** PCI to 2 Port SATA-150
** Manufacturer's Product Page: http://www.sci-worx.com/products/product.aspx?id=63
** Data Sheet URL: http://gkernel.sourceforge.net/specs/sii/3112A_SiI-DS-0095-B2.pdf.bz2
** Distributed with permission: http://www.osdir.com/ml/linux.ide/2004-10/msg00215.html
<br />
* SiI3114
** PCI to 4 Port SATA150
** Manufacturer's Product Page: http://www.sci-worx.com/products/product.aspx?id=28
** Data Sheet URL: http://www.sci-worx.com/docs/SiI-DS-0103-D.pdf
<br />
* SiI3124
** PCI/PCI-X to 4 Port SATA300
** Manufacturer's Product Page: http://www.sci-worx.com/products/product.aspx?id=27
** Data Sheet URL: http://www.sci-worx.com/docs/SiI-DS-0160-C.pdf
<br />
* SiI3132
** PCI Express (1x) to 2 Port SATA300
** Manufacturer's Product Page: http://www.sci-worx.com/products/product.aspx?id=32
** Data Sheet URL: http://www.sci-worx.com/docs/SiI-DS-0138-D.pdf
<br />
* SiI3512
** PCI to 2 PORT SATA-150
** Manufacturers Product Page: http://www.sci-worx.com/products/product.aspx?id=29
** Data Sheet URL: http://www.sci-worx.com/docs/SiI-DS-0102-D.pdf
<br />
* SiI3531
** PCI Express (1x) to 1 Port SATA300
** Manufacturer's Product Page: http://www.sci-worx.com/products/product.aspx?id=80
** Data Sheet URL: http://www.sci-worx.com/docs/SiI-DS-0208-C.pdf
<br />

==SATA Port Multipliers==
* SiI3726
** 5 Drive SATA300
** Manufacturer's Product Page: http://www.sci-worx.com/products/product.aspx?id=26
** Data Sheet URL: http://www.sci-worx.com/docs/SiI-DS-0121-C1.pdf
<br />
````

<a id="w002374"></a>

## W002374 — User:Combuster/CDRom BS

Bron-ID: W002374
Namespace: User (2)
Revisie: 6263 | Datum: 2008-09-15T18:00:56Z
Bron: https://wiki.osdev.org/User:Combuster/CDRom_BS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACombuster%2FCDRom_BS&oldid=6263

````text
My collection of random CDrom knowledge (or maybe BS). Maybe make a full low-level CDrom info doc later. Currently for storing background stuff for my work for a Hitachi-SH4 platform, and the tools seem to be lacking *nix support.

And '''I want that (***) Red Book.''' ECMA-119 and ECMA-130 don't cover *everything* and sometimes they are extremely cryptic if not incomplete.

== Disk Layout ==
Each disk consists of a series of frames, each is 33 bytes in length
= 24 bytes of data + 8 error correction + 1 subchannel byte (one bit for each channel)
On disk, the data is encoded by 8-to-14 modulation, with three interleaving bits so that each byte occupies 17 bits on the disk's surface. This is to make sure the CD reader can stay in sync with the data. A 27-bit number is added to differentiate between frames.

98 frames are grouped into one sector, making each sector have 2352 bytes of data (and 12¼ byte for each subchannel)

Audio CDs (following CD-DA convention) use the full 2352 bytes
Data CDs (Mode 1 CD-ROM) use only 2048 bytes for actual data, while the remainder is again used for error correction. This additional layer is meant to be able to cover scratches, which normally render several consecutive frames unreadable, but not enough to destroy most of a sector's contents, so that the data can be recovered. For audio, the extra error correction is not used since bad frames can be concealed by interpolation of data.
Data CDs that are less sensitive to data loss (Video CD's for example) or want more space (like PS games) do not store the error correction, but only the header. The Mode 2 CD-ROM therefore has 2336 information bytes per sector.

In total, there are two error correction mechanisms on the sector level, and one one the frame level.

=== Ascii Art ===
<pre>
Track:
------+--------+--------+--------+--------+--------+--------+---
  ... | sector | sector | sector | sector | sector | sector |
------+--------+--------+--------+--------+--------+--------+---
                    ____/        \_____
             ______/                   \_______
Sector:_____/                                  \______
  ___/                                                \___
  | 12 | 4 |                         | 4 | 8 | 172 | 104 |    field size
--+----+---+-------------------------+---+---+-----+-----+--
  |SYNC|HDR|       data x 2048       |EDC| - |ECC-P|ECC-Q|    Mode-1 data
--+----+---+-------------------------+---+---+-----+-----+--
  |SYNC|HDR|             data x 2336                     |    Mode-2 data
--+----+---+---------------------------------------------+--
  |                     data x 2352                      |    CD-DA (Audio)
--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+---+--
                ____/  \_____
           ____/             \_____
Frame:____/                        \____
  --+--------------------+----------+---+----
    |     data x 24      |   CIRC   |sub|
  --+--------------------+----------+---+----
    |        14          |     8    | 1 |
</pre>
Somebody with access to the red book please confirm a frame's ordering

=== Synchronisation field ===
On data discs, this block gives an unique pattern which a CD drive can synchronise to. According to the standard, the rest of the sector is scrambled because of this.

=== Header ===
This field holds the sector number of the current sector, and informs the reader if it deals with a mode-1 or mode-2 disc.

{| {{Wikitable}}

! Byte
! Contents
|- 
| 1
| Minute
|-
| 2
| Seconds
|-
| 3
| Fractions
|-
| 4
| Format Byte
* 0x00 empty
* 0x01 Mode-1 data
* 0x02 Mode-2 data
|}

The timing fields are ordered this way because of the Audio CD (which came before the Data CD). They are stored in binary coded decimal - i.e. the seconds go 0x00 0x01 0x02 0x03 0x04 0x05 0x06 0x07 0x08 0x09 0x10 0x11 etc. The numbers 0xA - 0xF are skipped.

Fractions are numbered from 0x00 to 0x74 (75 fractions in a second)
Seconds are numbered from 0x00 to 0x59 
Minutes are numbered from 0x00 upwards to whatever the CD permits. (There's mention of numbers starting at A0 in the spec)

Note that the actual data seems to start from 00:02:00, leaving 150 sectors for something else. (Red Book, anyone)

=== Error Detection Code ===
The error detection mechanism on Mode-1 data CD's. Basically it is a textbook CRC with a polynomial of 1.8001801B, least significant bit first. The standard is rather cryptic about this number.

The 8 bits following the CRC are empty, probably to detect error conditions in the CRC area.

=== ECC ===
todo. Looks like a pair of CIRC codes.

=== Cross-Interleaved Reed-Solomon Code ===
The error correction mechanism used at the frame level. Sufficient to correct minor read errors. Wikipedia has more info on it: http://en.wikipedia.org/wiki/Cross-interleaved_Reed-Solomon_coding



== Image formats ==
=== ISO image ===
Stores the important stuff. Looks basically like a concatenation of the 2048 byte data parts of a raw CD. Starts from sector 150 onward.

=== CDI image ===
according to cdi2iso, comes in 4 flavours. (raw, normal, PQ, CD+G).
normal images are just the same as ISO images, but they have 150 sectors more at the start. (seems to store TOC stuff) The other formats start with a header reading 00 ff ff ff ff ff ff ff ff ff ff 00, and contain other parts beside the data content.

=== BIN/CUE images ===
A hex dump revealed bin files to have raw sector contents, that means it includes all the headers and error codes of each sector and is the 2352 bytes per sector. For data sectors, its contents starts with sector 00:02:00. The TOC is stored in a separate .CUE file, which is plain text. Example:
 FILE "cd_contents.bin" BINARY
 TRACK 01 MODE1/2352
 INDEX 01 00:00:00
````

<a id="w002375"></a>

## W002375 — User:Inflater

Bron-ID: W002375
Namespace: User (2)
Revisie: 6290 | Datum: 2008-09-24T18:22:20Z
Bron: https://wiki.osdev.org/User:Inflater
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AInflater&oldid=6290

````text
I'm a teenager from Slovakia. I am more than ten years experienced into computers, I know assembly language, pascal and a bit BASIC. 
I got into OSdev by a pure coincidence and I was just curious how things work to the lowest level.

I'm developing my OS too, its called Derrick. Its home page is here - http://derrick.xf.cz.

So anyways, what else should be here? lol.
````

<a id="w002376"></a>

## W002376 — User:Troy martin

Bron-ID: W002376
Namespace: User (2)
Revisie: 9061 | Datum: 2009-10-06T02:23:46Z
Bron: https://wiki.osdev.org/User:Troy_martin
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATroy_martin&oldid=9061

````text
Hey. I'm Troy Martin. I'm a 14-year-old programmer from Vancouver, BC, Canada. I'm no longer a member of the forums but I hope to still work on the wiki.

==My user subpages==

* [[User:Troy_martin/Emulators]] -- NOTE: This page could possibly grow big and take a bit of time to load.

==My larger contributions here==

* [[Printing to the screen without a db]] -- a [[NASM]] macro that acts like printf() or puts() with a string literal.
* [[Turbo C]] -- a wonderful little 16-bit C compiler from Borland.
* [[Real mode assembly bare bones]] series -- oh noes! The noobs shall start flooding in now! Noooooooooooo....!
* Adding <tt><nowiki><source></nowiki></tt> tags to quite a few articles.
* The planned customizations for [[User:Troy martin/Monaco|Monaco]].

==Titanium Bonfire Project==

The Titanium Bonfire Project (http://tinyurl.com/tbproject) is a collection of open-source subprojects, the two current (well, more of the only two) subprojects being TBOS and TBOS32 (which is unreleased as of March 14 2009.)

==TBOS==

[[User:Troy martin/TBOS|TBOS]] is my OS project written in 16-bit assembly. It assembles in NASM and NASM only, but could probably be ported to other assemblers with macro functions quite easily.

* Current version is 1.0.1, working on 1.1.0

==TBOS32==

TBOS32 is an unreleased 32-bit assembly operating system. There's a simple keyboard driver in place using polling, yuck, and a few 16-bit TBOS calls converted to protected mode.

* Not yet released, working on it...
````

<a id="w002377"></a>

## W002377 — Talk:C++ Bare Bones

Bron-ID: W002377
Namespace: Talk (1)
Revisie: 12096 | Datum: 2011-10-13T08:31:00Z
Bron: https://wiki.osdev.org/Talk:C%2B%2B_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AC%2B%2B_Bare_Bones&oldid=12096

````text
== loader() and startup ==
The tuorial seems broken - some essential startcode is missing and the C version of loader() does less than the corresponding ASM versions. Besides, you'd probably want to delay initialisation of static classes to the point where you have a working memory manager.
- [[User:Combuster|Combuster]] 06:43, 3 October 2008 (UTC)

== Code in .data? ==

I've been looking at this after a long time, and suddenly one thing strikes me as odd: Why do we place ctors and dtors (which are executable code) in the .data section? Shouldn't this be in the .text section? -- [[User:Solar|Solar]] 06:01, 26 October 2010 (UTC)

:ctors/dtors are data, an array of function pointers to be precise.
 for(unsigned long *constructor(&start_ctors); constructor < &end_ctors; ++constructor) 
    ((void (*) (void)) (*constructor)) ();
:This means that with the least privileges it ought to be only readable and not necessarily executable. The .rodata section would technically be more appropriate though - [[User:Combuster|Combuster]] 14:48, 26 October 2010 (UTC)

::Ungh... /facepalm... unbelievable how rusty I've gotten. Of course they're only pointers into .text... moved them into .rodata. -- [[User:Solar|Solar]] 10:31, 29 October 2010 (UTC)

== borken? ==

I just had a quick look at the article again, and I found two things that "don't seem right":

* Shouldn't the C++ version of "loader" be linked extern "C"?
* Why does the article mention stuff like "if your linker requires adding leading underscore"? Don't we stubbornly recommend the cross-compiler approach to avoid ''just extactly such'' fluff in the articles?

-- [[User:Solar|Solar]] 08:42, 12 October 2011 (UTC)

:Using loader as a C function is especially wrong, since it is later used as the entry point (with ebx and eax holding important information and esp being undefined, the same goes for the assembly equivalents), and declaring that as C linkage does not fix that. I would just split the function into call_static_constructors() and call_static_destructors() and avoid the problem altogether. That is, after fixing the "<tt>Note also that you should call the destructors in reverse order!</tt>" bug that's contradicting the entire rest of the article.
:(And while we're at the tutorial bashing tour, s/main/kmain/ IMO)

:In general the tutorial is not self-sufficient while it might better be that way (something I commented on three years ago). And the code looks hacky as an unfortunate consequence.
: - [[User:Combuster|Combuster]] 17:34, 12 October 2011 (UTC)

::Gosh... this article is pretty screwed up, is it? I thought we had better than that... -- [[User:Solar|Solar]] 07:30, 13 October 2011 (UTC)

In a complete overhaul, I "synced" this article with [[Bare Bones]]. It ''should'' be self-sufficient now, but I don't have the environment right now to actually test the source.

# Somebody ''please'' check this "destructors in reverse order".
# I feel the calling of constructors and destructors should be done in kmain(). It would make the Multiboot magic number and info structure available to them, and remove the duplicated ASM code from the examples. I might do this edit myself, but not right now.

I hope this improves the article somewhat. -- [[User:Solar|Solar]] 08:31, 13 October 2011 (UTC)
````

<a id="w002378"></a>

## W002378 — User:Troy martin/Emulators

Bron-ID: W002378
Namespace: User (2)
Revisie: 6946 | Datum: 2008-12-26T23:48:29Z
Bron: https://wiki.osdev.org/User:Troy_martin/Emulators
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATroy_martin%2FEmulators&oldid=6946

````text
==Written in C==
===8-bit Outline===

 #include <stdio.h>
 #include <time.h>
 #define LDX 0x01
 #define LDY 0x02
 #define IOX 0x10
 #define HLT 0xFF
 
 unsigned int ip=0, x=0, y=0, acc=0;
 unsigned char memory[256] = {LDX,0x01,LDY,0x06,IOX,HLT,'M','y',' ','e','m','u','l','a','t','o','r',' ','w','o','r','k','s','!','\0'};
 
 int slp(unsigned int mseconds) /* sleep: can be used for setting processor limits */
 {
   clock_t goal = mseconds + clock();
   while (goal > clock());
   return 0;
 }
 
 void ldx(unsigned int xdata)
 {
   x=xdata;
 }
 
 void ldy(unsigned int ydata)
 {
   y=ydata;
 }
 
 void iox()
 {
   switch(x)
   {
     case 0x01:
     printf("%s",memory+y);
     break;
   }
 }
 
 int main(void)
 {
   int CPUIsRunning = 1;
   float memkb=sizeof(memory);
   printf("My Emulator 1.0\n%d bytes (%f KB) of memory.\n\n",sizeof(memory),(memkb/1024));
   while(CPUIsRunning==1)
   {
     //slp(100);
     switch(memory[ip])
     {
       case 0x01:
         ldx(memory[ip+1]);
         ip+=2;
         break;
       case 0x02:
         ldy(memory[ip+1]);
         ip+=2;
         break;
       case 0x10:
         iox();
         ip+=1;
         break;
       case 0xFF:
         printf("\n\nEmulator halted by opcode 0xFF (instruction HLT).\n");
         system("pause");
         CPUIsRunning=0;
         break;
       default:
         printf("\n\nEmulator halted by illegal opcode.\n");
         system("pause");
         CPUIsRunning=0;
         break;
     }
   }
   return 0;
 }

===16-bit===

I'll post code or a link to pastebin'd code when it's more complete.
````

<a id="w002379"></a>

## W002379 — Printing to the screen without a db

Bron-ID: W002379
Namespace: Main (0)
Revisie: 30420 | Datum: 2026-03-25T18:48:17Z
Bron: https://wiki.osdev.org/Printing_to_the_screen_without_a_db
Revisie-URL: https://wiki.osdev.org/index.php?title=Printing_to_the_screen_without_a_db&oldid=30420

````text
[[NASM]] has the ability to create useful macros that have a calling convention similar to [[BASIC]] statements. Macros can be used for many purposes, and here one is used to emulate BASIC's <code>PRINT</code> statement.

Note: this macro is designed to be used in [[real mode]] only, since it uses [[BIOS]] functions, but it can be easily be adapted for other printing methods.

<syntaxhighlight lang="nasm">
; Null terminated string in SI.
print_string:
     ; Save registers
     push ax
     push bx
     push bp        ; Some BIOSes may clobber this register too.
     mov ah, 0Eh    ; INT 10h teletype function.
 
 .loop:
     lodsb          ; Get byte from string.
     test al, al    ; Null terminator reached?
     jz .done       ; Yes, end printing.
 
     int 10h        ; No, print the character.
     jmp .loop      ; Loop!
 
 .done:
     ; Restore registers
     pop bp
     pop bx
     pop ax
     ret
 
 
 %macro print 1+
     section .data    ; At the end of the binary.
 %%string:
     db %1,0
     section .text    ; Back to where we were.
 
     mov si,%%string
     call print_string    ; Print it out using the print_string function.
 %endmacro
</syntaxhighlight>

Now <code>print</code> can be used as a regular function or instruction (Note: using this in a bootloader may not work, it messes with the data section).

<syntaxhighlight lang="nasm">
 print 'Printing without a db in NASM!',0Dh,0Ah    ; Print out a little message!
</syntaxhighlight>

A little known feature of NASM is the usage of the <code>`</code> (back quote) character to contain a string that can use [[C]]-style escape codes like <code>\n</code>.

==See Also==
=== Articles ===
* [[Printing To Screen]]

[[Category:Tutorials]]
[[Category:Video]]
[[Category:Text UI]]
````

<a id="w002382"></a>

## W002382 — Talk:Tutorials

Bron-ID: W002382
Namespace: Talk (1)
Revisie: 16770 | Datum: 2014-09-20T03:25:07Z
Bron: https://wiki.osdev.org/Talk:Tutorials
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ATutorials&oldid=16770

````text
== Ratings ==
Since we got quite a few tutorials by now (and more in the making), how about rating and ordering them based on how far into development the OS in question is? That'd allow the visitor to easily pick up on the subject most useful at the time - [[User:Combuster|Combuster]] 21:34, 19 October 2008 (UTC)
: If you're talking about how advanced a tutorial is, I created a simple box to put at the top of a tutorial, which could indicate this:
{{Rating|4|}}
:[[User:Virtlink|Virtlink]] 11:45, 22 October 2008 (UTC)
::I created a template for the tutorials. I will also apply them, but when anyone disagrees on any rating I give a tutorial, please do not hesitate to change it. [[User:Virtlink|Virtlink]] 22:32, 31 October 2008 (UTC)
:I like the icons :) I was originally thinking about having those ratings here where people have a good overview on what or what not to do next, so I updated the article. It doubles the administration however but IMO its worth that. - [[User:Combuster|Combuster]] 22:22, 2 November 2008 (UTC)

==Classification==

We have quite a bunch of barebones tutorials. Until recently, the main difference was the language of choice but now we're starting to see such tutorials written for other platforms. I suggest that we either create categories like "x86 Barebones Tutorials" or use an expanding list (I ''think'' the wiki supports that but I'm no expert). --[[User:Love4boobies|Love4boobies]] 16:50, 22 March 2011 (UTC)

== Maintenance problem ==

It appears that the rating of articles are listed in the article itself and on this page. This means that it is possible for articles to get out of sync (indeed, I am guilty of that, I just noticed this article today). This page indeed seems redundant given that we have automatically generated category pages that list tutorials of various rankings. Perhaps we should deleted this article or create the means to automatically generate it. --[[User:Sortie|Sortie]] 13:09, 4 November 2013 (CST)
````

<a id="w002383"></a>

## W002383 — User talk:Patrickv

Bron-ID: W002383
Namespace: User talk (3)
Revisie: 6380 | Datum: 2008-10-20T09:04:38Z
Bron: https://wiki.osdev.org/User_talk:Patrickv
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3APatrickv&oldid=6380

````text
== Plugging your "tutorial" ==
I've now removed a link to your tutorial twice, mainly because you're promoting it as "cool" and "needed" while it is unfinished and even now won't teach the reader anything meaningful. Besides if we gave page such a treatment we'd get loads of unrelated spam everywhere on the wiki.

Your tutorial isn't orphaned yet - it's still accessible from the [[:Category:Tutorials|tutorial category]], and more appropriately, from the [[:Category:In Progress|WIP category]]. But IMSNHO right now your story needs to be fleshed out quite a bit more before being worth being actively plugged elsewhere. If it turns out to be good you won't even have to do that yourself :)
- [[User:Combuster|Combuster]] 09:04, 20 October 2008 (UTC)
````

<a id="w002385"></a>

## W002385 — Talk:Modular Kernel

Bron-ID: W002385
Namespace: Talk (1)
Revisie: 6391 | Datum: 2008-10-21T13:41:46Z
Bron: https://wiki.osdev.org/Talk:Modular_Kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AModular_Kernel&oldid=6391

````text
== Disadvantages ==
I don't agree with the following statements under 'Disadvantages':
*  It may loose stability. If there is a module that does something bad, the kernel can crash, as modules should have full permissions.
* ...and therefore security is compromised. A module can do anything, so one could easily write an evil module to crash things (Some OS's only allow modules to be loaded by the root user).
I read that e.g. Minix has it's modules running in user mode, so the first statement is a matter of opinion. Second, a monolithic kernel always has it's 'modules' running in kernel space, so this is not a disadvantage of a modular kernel. It may actually be an advantage that its possible to run the modules in user space, therefore having better protection against 'bad' modules. [[User:Virtlink|Virtlink]] 13:25, 21 October 2008 (UTC)

:Note that a modular kernel having its modules running in userspace would be called a [[Microkernel]]. -- [[User:Solar|Solar]] 13:41, 21 October 2008 (UTC)
````

<a id="w002386"></a>

## W002386 — OSDev Wiki:Joining

Bron-ID: W002386
Namespace: OSDev Wiki (4)
Revisie: 6587 | Datum: 2008-11-04T15:33:44Z
Bron: https://wiki.osdev.org/OSDev_Wiki:Joining
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDev_Wiki%3AJoining&oldid=6587

````text
To be able to edit the OSDev wiki, you need to join the Wiki group from the forum. Start by [http://forum.osdev.org/ucp.php?mode=register creating a forum account], if you don't already have one. Please see [http://forum.osdev.org/viewtopic.php?f=8&t=677 the Wiki Access FAQ] for more information, or ask your question on the [http://forum.osdev.org/viewforum.php?f=8 OSDev Wiki forum].

[[Category:Wiki administration|{{PAGENAME}}]]
````

<a id="w002387"></a>

## W002387 — Nanokernel

Bron-ID: W002387
Namespace: Main (0)
Revisie: 6395 | Datum: 2008-10-22T10:43:52Z
Bron: https://wiki.osdev.org/Nanokernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Nanokernel&oldid=6395
Doorverwijzing: Exokernel

````text
#REDIRECT [[Exokernel#Nanokernel.2FPicokernel]]
````

<a id="w002388"></a>

## W002388 — Template:Kernel designs

Bron-ID: W002388
Namespace: Template (10)
Revisie: 30814 | Datum: 2026-06-23T19:26:21Z
Bron: https://wiki.osdev.org/Template:Kernel_designs
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AKernel_designs&oldid=30814

````text
{{SmallNavBox
| width:200px;
|Name=[[Kernels|Kernel Designs]]
|Group0=Models
|List0=<div>
[[Monolithic Kernel]]<br />
[[Microkernel]]<br />
[[Hybrid Kernel]]<br />
[[Exokernel]]<br />
[[Nanokernel|Nano/Picokernel]]<br />
[[Exokernel#Cache Kernel|Cache Kernel]]<br />
[[Exokernel#Virtualizing Kernel|Virtualizing Kernel]]<br />
[[Megalithic Kernel]]<br />
[[Multikernel]]<br />
</div>
|Group1=Other Concepts
|List1=<div>
[[Modular Kernel]]<br />
[[Higher Half Kernel]]<br />
[[Creating a 64-bit kernel|64-bit Kernel]]<br />
[[Pre-kernel]]<br />
[[Rump kernel]]<br />
</div>
}}
````

<a id="w002390"></a>

## W002390 — DMA

Bron-ID: W002390
Namespace: Main (0)
Revisie: 6414 | Datum: 2008-10-26T00:25:50Z
Bron: https://wiki.osdev.org/DMA
Revisie-URL: https://wiki.osdev.org/index.php?title=DMA&oldid=6414
Doorverwijzing: ISA DMA

````text
#REDIRECT [[ISA DMA]]
````

<a id="w002391"></a>

## W002391 — Talk:DMA

Bron-ID: W002391
Namespace: Talk (1)
Revisie: 6416 | Datum: 2008-10-26T00:25:50Z
Bron: https://wiki.osdev.org/Talk:DMA
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ADMA&oldid=6416
Doorverwijzing: Talk:ISA DMA

````text
#REDIRECT [[Talk:ISA DMA]]
````

<a id="w002392"></a>

## W002392 — Talk:Help My Kernel Crashed

Bron-ID: W002392
Namespace: Talk (1)
Revisie: 6972 | Datum: 2008-12-31T16:52:15Z
Bron: https://wiki.osdev.org/Talk:Help_My_Kernel_Crashed
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AHelp_My_Kernel_Crashed&oldid=6972

````text
This page should be merged with [[Help My Kernel Crashed]]. [[User:Virtlink|Virtlink]] 00:29, 26 October 2008 (UTC)
:Euhm, merging a page with itself? Are you sure it isn't a redirect? Or am I missing something? - [[User:Combuster|Combuster]] 10:59, 27 October 2008 (UTC)
::Well, I meant merging with the [[Troubleshooting]] page, but because they are so alike, I got mixed up. :P [[User:Virtlink|Virtlink]] 11:45, 27 October 2008 (UTC)
:::They are alike -- too alike. '''Definite merge.''' And BTW, when I coded my exception handlers, I set it up to dump registers (like probably everyone does.) Should we add a section on it, especially about the pushing of EIP, CS, and EFLAGS when interrupts fire (in that order) so you can trace the exact location of the crashing code. --[[User:Troy martin|Troy Martin]] 04:37, 9 December 2008 (UTC)
:::: Merged to [[Troubleshooting]], as that page is referenced from the Main Page, and has the most sensible title. [[User:Virtlink|Virtlink]] 16:52, 31 December 2008 (UTC)
````

<a id="w002393"></a>

## W002393 — File:Linear video memory layout 4bpp.png

Bron-ID: W002393
Namespace: File (6)
Revisie: 6421 | Datum: 2008-10-26T06:00:20Z
Bron: https://wiki.osdev.org/File:Linear_video_memory_layout_4bpp.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ALinear_video_memory_layout_4bpp.png&oldid=6421

````text
An overview of the memory layout of linear video memory using 4 bits-per-pixel. This applies to [[Bochs Graphics Adaptor]].
````

<a id="w002394"></a>

## W002394 — File:Linear video memory layout 8bpp.png

Bron-ID: W002394
Namespace: File (6)
Revisie: 6422 | Datum: 2008-10-26T06:00:40Z
Bron: https://wiki.osdev.org/File:Linear_video_memory_layout_8bpp.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ALinear_video_memory_layout_8bpp.png&oldid=6422

````text
An overview of the memory layout of linear video memory using 8 bits-per-pixel. This applies to [[Bochs Graphics Adaptor]].
````

<a id="w002395"></a>

## W002395 — File:Linear video memory layout 15bpp.png

Bron-ID: W002395
Namespace: File (6)
Revisie: 6423 | Datum: 2008-10-26T06:02:24Z
Bron: https://wiki.osdev.org/File:Linear_video_memory_layout_15bpp.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ALinear_video_memory_layout_15bpp.png&oldid=6423

````text
An overview of the memory layout of linear video memory using 15 bits-per-pixel. This applies to [[Bochs Graphics Adaptor]].
````

<a id="w002396"></a>

## W002396 — File:Linear video memory layout 16bpp.png

Bron-ID: W002396
Namespace: File (6)
Revisie: 6424 | Datum: 2008-10-26T06:02:43Z
Bron: https://wiki.osdev.org/File:Linear_video_memory_layout_16bpp.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ALinear_video_memory_layout_16bpp.png&oldid=6424

````text
An overview of the memory layout of linear video memory using 16 bits-per-pixel. This applies to [[Bochs Graphics Adaptor]].
````

<a id="w002397"></a>

## W002397 — File:Linear video memory layout 24bpp.png

Bron-ID: W002397
Namespace: File (6)
Revisie: 6425 | Datum: 2008-10-26T06:02:58Z
Bron: https://wiki.osdev.org/File:Linear_video_memory_layout_24bpp.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ALinear_video_memory_layout_24bpp.png&oldid=6425

````text
An overview of the memory layout of linear video memory using 24 bits-per-pixel. This applies to [[Bochs Graphics Adaptor]].
````

<a id="w002398"></a>

## W002398 — File:Linear video memory layout 32bpp.png

Bron-ID: W002398
Namespace: File (6)
Revisie: 6426 | Datum: 2008-10-26T06:03:17Z
Bron: https://wiki.osdev.org/File:Linear_video_memory_layout_32bpp.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ALinear_video_memory_layout_32bpp.png&oldid=6426

````text
An overview of the memory layout of linear video memory using 32 bits-per-pixel. This applies to [[Bochs Graphics Adaptor]].
````

<a id="w002399"></a>

## W002399 — Bochs Graphics Adaptor

Bron-ID: W002399
Namespace: Main (0)
Revisie: 9615 | Datum: 2010-01-11T14:54:00Z
Bron: https://wiki.osdev.org/Bochs_Graphics_Adaptor
Revisie-URL: https://wiki.osdev.org/index.php?title=Bochs_Graphics_Adaptor&oldid=9615
Doorverwijzing: Bochs VBE Extensions

````text
#REDIRECT [[Bochs VBE Extensions]]
````

<a id="w002400"></a>

## W002400 — Exceptions

Bron-ID: W002400
Namespace: Main (0)
Revisie: 29749 | Datum: 2025-11-08T00:40:46Z
Bron: https://wiki.osdev.org/Exceptions
Revisie-URL: https://wiki.osdev.org/index.php?title=Exceptions&oldid=29749

````text
{| align="right"
  | __TOC__
  |}

'''Exceptions''', as described in this article, are a type of [[interrupt]] generated by the CPU when an 'error' occurs. Some exceptions are not really errors in most cases, such as [[#Page Fault|page faults]].

Exceptions are classified as:
* '''Faults''': These can be corrected and the program may continue as if nothing happened.
* '''Traps''': Traps are reported immediately after the execution of the trapping instruction.
* '''Aborts''': Some severe unrecoverable error.

Some exceptions will push a 32-bit "error code" on to the top of the stack, which provides additional information about the error.  This value must be pulled from the stack before returning control back to the currently running program (i.e. before calling IRET). In [[Long Mode]], the error code is padded with zeros to form a 64-bit push, so that it can be popped like any other value.

{| {{wikitable}}
! Name
! Vector nr.
! Type
! Mnemonic
! Error code?
|-
! [[#Division Error|Division Error]]
| 0 (0x0)
| Fault
| #DE
| No
|-
! [[#Debug|Debug]]
| 1 (0x1)
| Fault/Trap
| #DB
| No
|-
! [[Non Maskable Interrupt|Non-maskable Interrupt]]
| 2 (0x2)
| Interrupt
| -
| No
|-
! [[#Breakpoint|Breakpoint]]
| 3 (0x3)
| Trap
| #BP
| No
|-
! [[#Overflow|Overflow]]
| 4 (0x4)
| Trap
| #OF
| No
|-
! [[#Bound Range Exceeded|Bound Range Exceeded]]
| 5 (0x5)
| Fault
| #BR
| No
|-
! [[#Invalid Opcode|Invalid Opcode]]
| 6 (0x6)
| Fault
| #UD
| No
|-
! [[#Device Not Available|Device Not Available]]
| 7 (0x7)
| Fault
| #NM
| No
|-
! [[#Double Fault|Double Fault]]
| 8 (0x8)
| Abort
| #DF
| Yes (Zero)
|-
! <strike>[[#Coprocessor Segment Overrun|Coprocessor Segment Overrun]]</strike>
| 9 (0x9)
| Fault
| -
| No
|-
! [[#Invalid TSS|Invalid TSS]]
| 10 (0xA)
| Fault
| #TS
| Yes
|-
! [[#Segment Not Present|Segment Not Present]]
| 11 (0xB)
| Fault
| #NP
| Yes
|-
! [[#Stack-Segment Fault|Stack-Segment Fault]]
| 12 (0xC)
| Fault
| #SS
| Yes
|-
! [[#General Protection Fault|General Protection Fault]]
| 13 (0xD)
| Fault
| #GP
| Yes
|-
! [[#Page Fault|Page Fault]]
| 14 (0xE)
| Fault
| #PF
| Yes
|-
! Reserved
| 15 (0xF)
| -
| -
| No
|-
! [[#x87 Floating-Point Exception|x87 Floating-Point Exception]]
| 16 (0x10)
| Fault
| #MF
| No
|-
! [[#Alignment Check|Alignment Check]]
| 17 (0x11)
| Fault
| #AC
| Yes
|-
! [[#Machine Check|Machine Check]]
| 18 (0x12)
| Abort
| #MC
| No
|-
! [[#SIMD Floating-Point Exception|SIMD Floating-Point Exception]]
| 19 (0x13)
| Fault
| #XM/#XF
| No
|-
! [[#Virtualization Exception| Virtualization Exception]]
| 20 (0x14)
| Fault
| #VE
| No
|-
! [[#Control Protection Exception|Control Protection Exception]]
| 21 (0x15)
| Fault
| #CP
| Yes
|-
! Reserved
| 22-27  (0x16-0x1B)
| -
| -
| No
|-
! [[#Hypervisor Injection Exception|Hypervisor Injection Exception]]
| 28 (0x1C)
| Fault
| #HV
| No
|-
! [[#VMM Communication Exception|VMM Communication Exception]]
| 29 (0x1D)
| Fault
| #VC
| Yes
|-
! [[#Security Exception|Security Exception]]
| 30 (0x1E)
| Fault
| #SX
| Yes
|-
! Reserved
| 31 (0x1F)
| -
| -
| No
|-
! [[#Triple Fault|Triple Fault]]
| -
| -
| -
| No
|-
! <strike>[[#FPU Error Interrupt|FPU Error Interrupt]]</strike>
| IRQ 13
| Interrupt
| #FERR
| No
|}

== Exceptions ==

=== Faults ===

==== Division Error ====
The '''Division Error''' occurs when dividing any number by 0 using the DIV or IDIV instruction, or when the division result is too large to be represented in the destination. Since a faulting DIV or IDIV instruction is very easy to insert anywhere in the code, many OS developers use this exception to test whether their exception handling code works.

The saved instruction pointer points to the DIV or IDIV instruction which caused the exception.

==== Bound Range Exceeded ====
This exception can occur when the BOUND instruction is executed. The BOUND instruction compares an array index with the lower and upper bounds of an array. When the index is out of bounds, the Bound Range Exceeded exception occurs.

The saved instruction pointer points to the BOUND instruction which caused the exception.

==== Invalid Opcode ====
The Invalid Opcode exception occurs when the processor tries to execute an invalid or undefined opcode, or an instruction with invalid prefixes. It also occurs in other cases, such as:
* The instruction length exceeds 15 bytes, but this only occurs with redundant prefixes.
* The instruction tries to access a non-existent control register (for example, <code>mov cr6, eax</code>).
* The UD instruction is executed.

The saved instruction pointer points to the instruction which caused the exception.

==== Device Not Available ====
The Device Not Available exception occurs when an FPU instruction is attempted but there is no FPU.  This is not likely, as modern processors have built-in FPUs. However, there are flags in the CR0 register that disable the FPU/MMX/SSE instructions, causing this exception when they are attempted. This feature is useful because the operating system can detect when a user program uses the FPU or XMM registers and then save/restore them appropriately when multitasking.

The saved instruction pointer points to the instruction that caused the exception.

==== Invalid TSS ====
An Invalid TSS exception occurs when an invalid segment selector is referenced as part of a task switch, or as a result of a control transfer through a gate descriptor, which results in an invalid stack-segment reference using an SS selector in the TSS.

When the exception occurred before loading the segment selectors from the TSS, the saved instruction pointer points to the instruction which caused the exception. Otherwise, and this is more common, it points to the first instruction in the new task.

'''Error code:''' The Invalid TSS exception sets an error code, which is a [[#Selector Error Code|selector index]].

==== Segment Not Present ====
The Segment Not Present exception occurs when trying to load a segment or gate which has its `Present` bit set to 0.
However when loading a stack-segment selector which references a descriptor which is not present, a [[#Stack-Segment Fault|Stack-Segment Fault]] occurs.

If the exception happens during a hardware task switch, the segment values should not be relied upon by the handler. That is, the handler should check them before trying to resume the new task. There are three ways to do this, according to the Intel documentation.

The saved instruction pointer points to the instruction which caused the exception.

'''Error code:''' The Segment Not Present exception sets an error code, which is the [[#Selector Error Code|segment selector index]] of the segment descriptor which caused the exception.

==== Stack-Segment Fault ====
The Stack-Segment Fault occurs when:
* Loading a stack-segment referencing a segment descriptor which is not present.
* Any PUSH or POP instruction or any instruction using ESP or EBP as a base register is executed, while the stack address is not in canonical form.
* When the stack-limit check fails.

If the exception happens during a hardware task switch, the segment values should not be relied upon by the handler. That is, the handler should check them before trying to resume the new task. There are three ways to do this, according to the Intel documentation.

The saved instruction pointer points to the instruction which caused the exception, unless the fault occurred because of loading a non-present stack segment during a hardware task switch, in which case it points to the next instruction of the new task.

'''Error code:''' The Stack-Segment Fault sets an error code, which is the stack [[#Selector Error Code|segment selector index]] when a non-present segment descriptor was referenced or a limit check failed during a hardware task switch. Otherwise (for present segments and already in use), the error code is 0.

==== General Protection Fault ====
A General Protection Fault may occur for various reasons. The most common are:
* Segment error (privilege, type, limit, read/write rights).
* Executing a privileged instruction while CPL != 0.
* Writing a 1 in a reserved register field or writing invalid value combinations (e.g. CR0 with PE=0 and PG=1).
* Referencing or accessing a null-descriptor.
* Accessing a memory address with bits 48-63 not matching bit 47 (e.g. 0x_0000_8000_0000_0000 instead of 0x_ffff_8000_0000_0000) in 64 bit mode.
* Executing an instruction that requires memory operands to be aligned (e.g. movaps) without the proper alignment.

The saved instruction pointer points to the instruction which caused the exception.

'''Error code:''' The General Protection Fault sets an error code, which is the [[#Selector Error Code|segment selector index]] when the exception is segment related. Otherwise, 0.

==== Page Fault ====
A Page Fault occurs when:
* A [[Paging|page directory or table]] entry is not present in physical memory.
* Attempting to load the instruction TLB with a translation for a non-executable page.
* A protection check (privileges, read/write) failed.
* A reserved bit in the page directory or table entries is set to 1.

The saved instruction pointer points to the instruction which caused the exception.

===== Error code =====
The Page Fault sets an error code:

<pre>
 31              15                             4               0
+---+--  --+---+-----+---+--  --+---+----+----+---+---+---+---+---+
|   Reserved   | SGX |   Reserved   | SS | PK | I | R | U | W | P |
+---+--  --+---+-----+---+--  --+---+----+----+---+---+---+---+---+
</pre>

{| {{wikitable}}
!
! Length
! Name
! Description
|-valign="top"
!align="left"| P
| 1 bit
| Present
| When set, the page fault was caused by a page-protection violation. When not set, it was caused by a non-present page.
|-valign="top"
!align="left"| W
| 1 bit
| Write
| When set, the page fault was caused by a write access. When not set, it was caused by a read access.
|-valign="top"
!align="left"| U
| 1 bit
| User
| When set, the page fault was caused while CPL = 3. This does not necessarily mean that the page fault was a privilege violation.
|-valign="top"
!align="left"| R
| 1 bit
| Reserved write
| When set, one or more page directory entries contain reserved bits which are set to 1. This only applies when the PSE or PAE flags in CR4 are set to 1.
Note that this includes invalid physical addresses (e.g. 1<<40 on a processor with only 39 physical bits).
|-valign="top"
!align="left"| I
| 1 bit
| Instruction Fetch
| When set, the page fault was caused by an instruction fetch. This only applies when the No-Execute bit is supported and enabled. 
|-valign="top"
!align="left"| PK
| 1 bit
| Protection key
| When set, the page fault was caused by a protection-key violation. The PKRU register (for user-mode accesses) or PKRS MSR (for supervisor-mode accesses) specifies the protection key rights.
|-valign="top"
!align="left"| SS
| 1 bit
| Shadow stack
| When set, the page fault was caused by a shadow stack access.
|-valign="top"
!align="left | SGX
| 1 bit
| Software Guard Extensions
| When set, the fault was due to an [https://en.wikipedia.org/wiki/Software_Guard_Extensions SGX violation]. The fault is unrelated to ordinary paging.
|}

In addition, it sets the value of the CR2 register to the virtual address which caused the Page Fault.

==== x87 Floating-Point Exception ====
The x87 Floating-Point Exception occurs when the FWAIT or WAIT instruction, or any waiting floating-point instruction is executed, and the following conditions are true:
* [[CR0|CR0.NE]] is 1;
* an unmasked x87 floating point exception is pending (i.e. the exception bit in the x87 floating point status-word register is set to 1).

The saved instruction pointer points to the instruction which is about to be executed when the exception occurred. The x87 instruction pointer register contains the address of the last instruction which caused the exception.

'''Error Code:''' The exception does not push an error code. However, exception information is available in the x87 status word register.

==== Alignment Check ====
An Alignment Check exception occurs when alignment checking is enabled and an unaligned memory data reference is performed. Alignment checking is only performed in CPL 3.

Alignment checking is disabled by default. To enable it, set the [[CR0|CR0.AM]] and [[RFLAGS|RFLAGS.AC]] bits both to 1.

The saved instruction pointer points to the instruction which caused the exception.

==== SIMD Floating-Point Exception ====
The SIMD Floating-Point Exception occurs when an unmasked 128-bit media floating-point exception occurs and the [[CR4|CR4.OSXMMEXCPT]] bit is set to 1.  If the OSXMMEXCPT flag is not set, then SIMD floating-point exceptions will cause an Undefined Opcode exception instead of this.

The saved instruction pointer points to the instruction which caused the exception.

'''Error Code:''' The exception does not push an error code. However, exception information is available in the MXCSR register.

=== Traps ===

==== Debug ====
The Debug exception occurs on the following conditions:
* Instruction fetch breakpoint (Fault)
* General detect condition (Fault)
* Data read or write breakpoint (Trap)
* I/O read or write breakpoint (Trap)
* Single-step (Trap)
* Task-switch (Trap)

When the exception is a fault, the saved instruction pointer points to the instruction which caused the exception. When the exception is a trap, the saved instruction pointer points to the instruction after the instruction which caused the exception.

'''Error code:''' The Debug exception does not set an error code. However, exception information is provided in the debug registers ([[CPU_Registers_x86#Debug_Registers]]).

==== Breakpoint ====
A Breakpoint exception occurs at the execution of the INT3 instruction. Some debug software replace an instruction by the INT3 instruction. When the breakpoint is trapped, it replaces the INT3 instruction with the original instruction, and decrements the instruction pointer by one.

The saved instruction pointer points to the byte after the INT3 instruction.

==== Overflow ====
An Overflow exception is raised when the INTO instruction is executed while the overflow bit in RFLAGS is set to 1.

The saved instruction pointer points to the instruction after the INTO instruction.

=== Aborts ===
==== Double Fault ====
A Double Fault occurs when an exception is unhandled or when an exception occurs while the CPU is trying to call an exception handler.  Normally, two exception at the same time are handled one after another, but in some cases that is not possible.  For example, if a page fault occurs, but the exception handler is located in a not-present page, two page faults would occur and neither can be handled.  A double fault would occur.

A double fault will always generate an error code with a value of zero.

The saved instruction pointer is undefined.  A double fault cannot be recovered.  The faulting process must be terminated.

In several starting hobby OSes, a double fault is also quite often a [[I_Cant_Get_Interrupts_Working|misdiagnosed IRQ0]] in the cases where the [[PIC]] hasn't been reprogrammed yet.

==== Machine Check ====
The Machine Check exception is model specific and processor implementations are not required to support it. It uses [[Model Specific Registers|model-specific registers]] to provide error information. It is disabled by default. To enable it, set the [[CR4|CR4.MCE]] bit to 1.

Machine check exceptions occur when the processor detects internal errors, such as bad memory, bus errors, cache errors, etc.

The value of the saved instruction pointer depends on the implementation and the exception.

==== Triple Fault ====
{{main|Triple Fault}}
The Triple Fault is not really an exception, because it does not have an associated vector number. Nonetheless, a triple fault occurs when an exception is generated when attempt to call the double fault exception handler. It results in the processor resetting. See the main article for more information about possible causes and how to avoid them.

== Selector Error Code ==
<pre>
 31         16   15         3   2   1   0
+---+--  --+---+---+--  --+---+---+---+---+
|   Reserved   |    Index     |  Tbl  | E |
+---+--  --+---+---+--  --+---+---+---+---+
</pre>

{| {{wikitable}}
!
! Length
! Name
! Description
|-valign="top"
!align="left"| E
| 1 bit
| External
| When set, the exception originated externally to the processor.
|-valign="top"
!align="left"| Tbl
| 2 bits
| IDT/GDT/LDT table
| This is one of the following values:
{| {{wikitable}}
! Value
! Description
|-
! 0b00
| The Selector Index references a descriptor in the GDT.
|-
! 0b01
| The Selector Index references a descriptor in the IDT.
|-
! 0b10
| The Selector Index references a descriptor in the LDT.
|-
! 0b11
| The Selector Index references a descriptor in the IDT.
|}

|-valign="top"
!align="left"| Index
| 13 bits
| Selector Index
| The index in the GDT, IDT or LDT.
|}

=== Legacy ===
The following exceptions happen on outdated technology, but are no longer used or should be avoided. They apply mostly to the intel 386 and earlier, and might include CPUs from other manufacturers around the same time.

====FPU Error Interrupt====
In the old days, the floating point unit was a dedicated chip that could be attached to the processor. It lacked direct wiring of FPU errors to the processor, so instead it used [[PIC|IRQ 13]], allowing the CPU to deal with errors at its own leasure. When the 486 was developed and multiprocessor support was added, the [[FPU]] was embedded on die and a global interrupt for FPUs became undesirable, instead getting an option for direct error handling. By default, this method is not enabled at boot for backwards compatibility, but an OS should update the settings accordingly.

====Coprocessor Segment Overrun====
When the FPU was still external to the processor, it had separate segment checking in protected mode. Since the 486 this is handled by a [[#General Protection Fault|GPF]] instead like it already did with non-FPU memory accesses.

== See Also ==

=== External Links ===
* [http://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-manual-325462.pdf Intel® 64 and IA-32 Architectures Software Developer's Manual], Volume 3 (System Programming Guide), Chapter 6 (Interrupt and exception handling)

[[Category:X86]]
[[Category:Interrupts]]
[[de:Exception]]
````

<a id="w002401"></a>

## W002401 — Fat imgen

Bron-ID: W002401
Namespace: Main (0)
Revisie: 22632 | Datum: 2018-09-09T23:00:35Z
Bron: https://wiki.osdev.org/Fat_imgen
Revisie-URL: https://wiki.osdev.org/index.php?title=Fat_imgen&oldid=22632

````text
{{DISPLAYTITLE:{{lcfirst:{{PAGENAMEE}}}}}}
'''fat_imgen''' is an utility that is used to create and modify [[FAT12]] floppy images with. It is licensed under the Fair License.

== Binaries ==

Windows and Linux users can download the latest stable binaries from [http://sourceforge.net/projects/fatimgen/files/ SourceForge].

== Source code ==

The source code can be checked out from [http://subversion.assembla.com/svn/slipstream/fat_imgen/ http://subversion.assembla.com/svn/slipstream/fat_imgen/].
It shows many aspects of working with [[FAT12]] images.

== See Also ==
=== Articles ===
* [[FAT]]

=== External Links ===
* [http://www.sourceforge.net/projects/fatimgen SourceForge Project Page]
* [http://www.assembla.com/code/slipstream/subversion/nodes/fat_imgen The project's source code]

[[Category:Disk Image Utilities]]
````

<a id="w002403"></a>

## W002403 — Interrupt

Bron-ID: W002403
Namespace: Main (0)
Revisie: 6465 | Datum: 2008-10-28T11:12:51Z
Bron: https://wiki.osdev.org/Interrupt
Revisie-URL: https://wiki.osdev.org/index.php?title=Interrupt&oldid=6465
Doorverwijzing: Interrupts

````text
#REDIRECT [[Interrupts]]
````

<a id="w002404"></a>

## W002404 — File:Info.png

Bron-ID: W002404
Namespace: File (6)
Revisie: 6480 | Datum: 2008-10-29T17:45:43Z
Bron: https://wiki.osdev.org/File:Info.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AInfo.png&oldid=6480

````text

````

<a id="w002405"></a>

## W002405 — User talk:Alfaomega08

Bron-ID: W002405
Namespace: User talk (3)
Revisie: 12556 | Datum: 2012-02-06T13:23:35Z
Bron: https://wiki.osdev.org/User_talk:Alfaomega08
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AAlfaomega08&oldid=12556

````text
Hello Alfaomega08, welcome to the wiki!
Your uploaded [[:Image:Info.png|info icon]] looks similar (although not exactly the same) as [http://en.wikipedia.org/wiki/Image:Imbox_notice.png this icon] on the Wikipedia website. Is it your own work? [[User:Virtlink|Virtlink]] 13:29, 30 October 2008 (UTC)

Actually, your image is the same as [http://en.wikipedia.org/w/index.php?title=Image%3AInfo_non-talk.png this wikipedia image]. And [http://wiki.osdev.org/index.php?title=Template%3AIn_Progress&diff=6481&oldid=4470 your template] is clearly based on [http://it.wikipedia.org/wiki/Template:Avviso this template] on the Italian wikipedia. Copying from other sites is generally not allowed, and this includes copying from Wikipedia. Your changes to the template might have to be undone, and your image deleted. [[User:Virtlink|Virtlink]] 13:48, 30 October 2008 (UTC)

:Copyrighted template removed. The icon is however placed in the public domain and as such may be freely posted here. - [[User:Combuster|Combuster]] 14:04, 30 October 2008 (UTC)

Thankyou for helping me
--[[User:Bellezzasolo|Bellezzasolo]] 07:23, 6 February 2012 (CST)
````

<a id="w002406"></a>

## W002406 — File:Difficulty 0.png

Bron-ID: W002406
Namespace: File (6)
Revisie: 27823 | Datum: 2023-06-24T18:19:42Z
Bron: https://wiki.osdev.org/File:Difficulty_0.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ADifficulty_0.png&oldid=27823

````text
For tutorials which are not yet rated.
````

<a id="w002407"></a>

## W002407 — File:Difficulty 1.png

Bron-ID: W002407
Namespace: File (6)
Revisie: 27822 | Datum: 2023-06-24T18:19:32Z
Bron: https://wiki.osdev.org/File:Difficulty_1.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ADifficulty_1.png&oldid=27822

````text
For easy tutorials.
````

<a id="w002408"></a>

## W002408 — File:Difficulty 2.png

Bron-ID: W002408
Namespace: File (6)
Revisie: 27824 | Datum: 2023-06-24T18:19:50Z
Bron: https://wiki.osdev.org/File:Difficulty_2.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ADifficulty_2.png&oldid=27824

````text
For medium difficulty tutorials.
````

<a id="w002409"></a>

## W002409 — File:Difficulty 3.png

Bron-ID: W002409
Namespace: File (6)
Revisie: 27825 | Datum: 2023-06-24T18:19:56Z
Bron: https://wiki.osdev.org/File:Difficulty_3.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ADifficulty_3.png&oldid=27825

````text
For advanced tutorials.
````

<a id="w002410"></a>

## W002410 — File:Difficulty 4.png

Bron-ID: W002410
Namespace: File (6)
Revisie: 27826 | Datum: 2023-06-24T18:20:02Z
Bron: https://wiki.osdev.org/File:Difficulty_4.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ADifficulty_4.png&oldid=27826

````text
For master level tutorials.
````

<a id="w002411"></a>

## W002411 — Template:Rating

Bron-ID: W002411
Namespace: Template (10)
Revisie: 28968 | Datum: 2024-06-06T17:18:57Z
Bron: https://wiki.osdev.org/Template:Rating
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ARating&oldid=28968

````text
{{{2|[[Category:Level {{{1|{{{level|0}}}}}} Tutorials]]}}}<table style="font-size:95%; line-height:1.5em; padding:0.25em; float:right; margin: 0 0 8px 15px; clear:right; border:1px solid #aaaaaa; background:#eee; text-align:center;{{{2|}}};"><!--

--><tr><th>Difficulty level</th></tr><tr><td>[[Image:Difficulty {{{1|{{{level|0}}}}}}.png]]<br />{{If|{{Eq|{{{1|{{{level|0}}}}}}|0}}|Not rated|{{If|{{Eq|{{{1|{{{level|0}}}}}}|1}}|Beginner|{{If|{{Eq|{{{1|{{{level|0}}}}}}|2}}|Medium|{{If|{{Eq|{{{1|{{{level|0}}}}}}|3}}|Advanced|Master}}}}}}}}</td></tr><!--

--></table><noinclude>
Usage:
<pre>
{{Rating|3}}
{{Rating|3|}}
</pre>
Use the following levels:
* 0 or not specified: Not rated
* 1: Beginner
* 2: Medium
* 3: Advanced
* 4: Master
Add an empty second parameter to prevent the article from being added to the category.
</noinclude>
````

<a id="w002412"></a>

## W002412 — User:01000101

Bron-ID: W002412
Namespace: User (2)
Revisie: 14294 | Datum: 2013-01-14T02:25:44Z
Bron: https://wiki.osdev.org/User:01000101
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3A01000101&oldid=14294

````text
''' 01000101's wiki page '''

== Introduction ==
I got into OS development quite a while ago in search of a good challenge, and that is exactly what I got.  My first OS was nothing more than a re-implementation of some tutorial code that said "hello world!" on the screen and fit into the boot sector of a floppy disk.  It took a long time for me to acquire more knowledge on the subject, as along the way I picked up some invaluable resources (such as the Intel manuals, and other books) that kept me interested in development.  Later I went on to develop network card drivers.  The RTL8139/RTL8169 chip sets were the first I developed for, and I later went on to both add to the RTL8139 wiki entry and to create the RTL8169 entry.  The latest network card I have written a home brew driver for is the Intel PRO/1000 (82563EB), and it is by far the most feature rich card I've ever owned. If you want to find out more info about me, check out my OS website http://www.dins-os.com .

I also recently started a wiki article on my own optimized library (OptLib).  This can be found here [[User:01000101/optlib/]]

I have some code in the VirtualBox trunk for their E1000 (Intel NIC) driver and probably some optimized memory functions in Pedigree.  I also have posted an example driver for the Intel 825xx-series network chipset and host the C file on my business website (http://www.randomaccessit.com/osdev/i825xx.c)

== D.i.N.S. ==
D.i.N.S. (Drop-in Network Security) is the name of my operating system.  It is designed to act as a specialized deep-packet analyzer for educational and corporate networks.  As this is my operating system, I think it would be appropriate to list some of the implemented features just to give an idea on the overall completeness of the OS. 

* Pure monolithic design.
* Written in Assembly and C.
* Designed and optimized for 64-bit Intel Xeon processors.
* Utilizes 64-bit Long mode.
* Paging; 1:1 map.
* LBA28 HDD support.
* SMP optimization.
* Arbitrary-precision integer support.
* ARP/RARP/ICMPv4+6/DHCP/DNS support.
* Minimal HTTP server.
* Headless design.

On July 15, 2008, D.i.N.S. LLC was formed.  I started getting quite a bit of support from the business world and the education world as well.  About a month later I ended up getting my first investment into the company to buy equipment (computer/network parts).  Currently, I am working towards a much larger investment that will set the project on a year-long developing spree in hopes of getting prototypes out and into public (yet monitored) networks. 

=== DLFS ===
D.i.N.S. Logging File System is a custom designed file system for use in the D.i.N.S. OS.  It is designed to log networking events (such as p2p/torrent detection) by date.  It doesn't use the traditional super-block approach.  Instead, it uses a design closer to the MP tables or SMBIOS structures.  

Basically, a step-by-step of read a log from the FS would go like:
* read the floating structure, it can be in blocks 1 (not the boot sector) through 64, and test the signature "DLFS" in the first 4 bytes.
* calculate and compare the 1-byte checksum.
* either examine the (mostly reserved) extended structure right after the FPS block, or check out the block pointed to in the "start year" field.
* from within the "Start year" structure, you can either examine the next year structure pointed to in the "Next chain" field, or you can jump to a block defined in the "entry" field which is used to store months 0-11.  
* the process can then be repeated (with the exception of the "next chain" field) to end up at the log of any specified day. 

The logs themselves are of variable size (but will occupy 512-bytes as a minimum).  The fragments of logs that exceed the size of one sector are linked together by a "next chain" field in the log structure.  Also, there is a variable amount of logs that can be stored per day.  

Currently only the year block addresses are cached in memory for quick retrieval.  The cache is only updated when a year is created.  Deletion and re-allocation of unused blocks is not currently implemented, but it's in development.

The current state of the FS is in beta and has read/write capabilities.
````

<a id="w002413"></a>

## W002413 — Template:Ratingicon

Bron-ID: W002413
Namespace: Template (10)
Revisie: 6557 | Datum: 2008-11-03T16:24:30Z
Bron: https://wiki.osdev.org/Template:Ratingicon
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ARatingicon&oldid=6557

````text
[[Image:Difficulty {{{1|{{{level|0}}}}}}.png]]<noinclude>


Usage:
<pre>
{{Ratingicon|3}}
</pre>
Use the following levels:
* 0 or not specified: Not rated
* 1: Beginner
* 2: Medium
* 3: Advanced
* 4: Master
</noinclude>
````

<a id="w002414"></a>

## W002414 — Genisoimage

Bron-ID: W002414
Namespace: Main (0)
Revisie: 6544 | Datum: 2008-11-03T14:04:21Z
Bron: https://wiki.osdev.org/Genisoimage
Revisie-URL: https://wiki.osdev.org/index.php?title=Genisoimage&oldid=6544
Doorverwijzing: Mkisofs

````text
#REDIRECT [[mkisofs]]
````

<a id="w002415"></a>

## W002415 — Bootable El-Torito CD with GRUB Legacy

Bron-ID: W002415
Namespace: Main (0)
Revisie: 30412 | Datum: 2026-03-24T22:44:12Z
Bron: https://wiki.osdev.org/Bootable_El-Torito_CD_with_GRUB_Legacy
Revisie-URL: https://wiki.osdev.org/index.php?title=Bootable_El-Torito_CD_with_GRUB_Legacy&oldid=30412

````text
{{Rating|1}}
This tutorial guides you through making a bootable CD .iso image with GRUB Legacy. We are going to create an [[El-Torito]] "no-emulation" bootable CD, which is different from a [[Bootable CD|bootable CD emulating a floppy disc]].

== Requirements ==

You will need the following:
* [[mkisofs]], which is in fact superseeded by [[genisoimage]].
* A ([[Multiboot]] compliant) kernel that [[GRUB Legacy]] can boot.
* The El-Torito GRUB Legacy stage2 file, called <tt>stage2_eltorito</tt>.

=== Ubuntu/Debian ===

In Ubuntu or Debian you can install the required software like this:

<syntaxhighlight lang="bash">
sudo apt-get install genisoimage grub
</syntaxhighlight>

We need a place to store the files on the CD image:

<syntaxhighlight lang="bash">
cd                            # Go to your home directory.
mkdir -p isofiles/boot/grub
</syntaxhighlight>

You now created an <tt>isofiles</tt> directory in your home folder and a <tt>boot/grub</tt> sub-folder inside that. We need the stage2 file. It is installed from the grub package. Just copy it:

<syntaxhighlight lang="bash">
cp /usr/lib/grub/i386-pc/stage2_eltorito  ~/isofiles/boot/grub/
</syntaxhighlight>

=== Windows ===

Look at [[mkisofs#Under windows|this article about how to install and use mkisofs]]. We need a place to store the files on the CD image. Create a folder <tt>isofiles</tt>, which contains the sub-folder <tt>boot\grub</tt> needed by GRUB. We need the stage2 file. You can get it by [http://www.geocities.com/imulgrew/grub_files.tar.gz downloading a package] and unpack it using your favourite unpacking application. Find the file <tt>stage2_eltorito</tt> and copy it to <tt>isofiles\boot\grub</tt>.

== Install a kernel ==

Get a kernel of your own choosing and copy it to wherever you like inside the <tt>isofiles</tt> folder. Preferably it should be placed in the <tt>boot</tt> sub-folder. Now create a <tt>menu.lst</tt> file. This file controls what menu entries GRUB should provide and how the kernels are booted. It must be placed in the GRUB folder <tt>boot/grub</tt> and contain something like this:

<!-- Not really Bash, it's Grub scripting, but lang="bash" makes nice syntax highlighting. -->
<syntaxhighlight lang="bash">
default 0
#timeout 30

#title Boot from hard disk
#chainloader (hd0)+1

title My kernel
kernel /boot/kernel-file    # Edit it to the filename of your kernel.
</syntaxhighlight>

In GRUB there's a concept called root. It's the disk drive or partition where you access the files, like the kernel image and modules. When booting from CD using <tt>stage2_eltorito</tt> you don't need to set the root as it is already set to "<tt>(cd)</tt>".

== Creating the <tt>.iso</tt> image ==

In the following I use the command <tt>genisoimage</tt>, but you can change it to <tt>mkisofs</tt> if that is what its called on your system. Open a command prompt/terminal and go to where the <tt>isofiles</tt> folder is located. It is your home directory on Ubuntu. Issue the command:

<syntaxhighlight lang="bash">
genisoimage -R -b boot/grub/stage2_eltorito -no-emul-boot -boot-load-size 4
            -boot-info-table -o bootable.iso isofiles
</syntaxhighlight>

Now you have a file called <tt>bootable.iso</tt>. Test it using your favourite emulator or burn it to a CD and test on a real computer. I will just explain the command line arguments we used for genisoimage:

{| class="wikitable"
|-
| -R
| Use the Rock Ridge protocol, which enables lower-case filenames on the CD. This is needed by GRUB.
|-
| -b ''file''
| The file to boot (the filename is in the created [[ISO 9660]] file system).
|-
| -no-emul-boot
| Enables no emulation El-Torito boot.
|-
| -boot-load-size 4
| Specifies the number of 512-bytes sectors to load. Four 512-byte sectors (2048 bytes) is one CD sector and is the number supported by most BIOS.
|-
| -boot-info-table
| Patches the boot file to contain info about the CD image. It's needed by GRUB.
|-
| -o bootable.iso
| The filename of the resulting .iso image.
|-
| isofiles
| Other arguments are the files and folders that should be included on the CD. In this case it's only the content of ''isofiles''.
|}

== Advanced stuff ==

=== Give it a label ===

You want to label your CD image, so you can later recognize it when loading your CD. Just pass a '''-V''' command line argument to genisoimage followed by the name you want.

=== Make it be quiet ===

Some day you probably want to create an .iso image directly in your [[Makefile]]. If you don't like the output that genisoimage creates, just pass the command line argument '''-quiet'''.

Sometimes you get a warning about the input character set used. If that is a problem just pass the argument '''-input-charset ascii''' (or utf8 if that is what you use on your filesystem).

=== Make the image from different files and folders ===

When you are building your kernel the parts on the .iso image could be located at different paths. Imagine your project files are like this:

<!-- Fancy Unicode/ascii art tree. Created in Ubuntu command line: "tree -A -F --dirsfirst" -->
<pre>
project/
├── build/
│   ├── modules/
│   │   ├── iso9660.mod
│   │   └── kbdrv.mod
│   └── my-kernel.elf
├── src/
│   ├── grub/
│   │   └── menu.lst
│   └── ...
├── thirdparty/
│   └── grub/
│       └── stage2_eltorito
└── Makefile
</pre>

You don't want to copy those files around in order to create the .iso image. Just use the '''-graft-points''' argument, like this:

<syntaxhighlight lang="bash">
genisoimage -graft-points # ...other arguments here...
            boot/my-kernel.elf=build/my-kernel.elf
            boot/grub/menu.lst=src/grub/menu.lst
            boot/grub/stage2_eltorito=thirdparty/grub/stage2_eltorito
            modules/=build/modules/
</syntaxhighlight>

You can use both files and folders together with the -graft-points argument.

== Troubleshooting ==

If you don't find a solution to your problem below, please ask in [http://forum.osdev.org/ the forums].

=== mkisofs says Uh oh, I cant find the boot image ===

The output of mkisofs looks like this:

<pre>
mkisofs: Uh oh, I cant find the boot image 'isofiles/boot/grub/stage2_eltorito' !
</pre>

This problem arises because mkisofs/genisoimage looks for its boot image as a subdirectory of the filesystem on the CD; make sure that the path you specify starts with 'boot/' rather than the name of your ISO directory ('isofiles/' in this example).

Source: [[topic:20102|this thread]]

=== I get a permission denied error in Linux ===

The output of genisoimage looks like this:

<pre>
Size of boot image is 4 sectors -> No emulation
genisoimage: Permission denied. Error opening boot image file 'isofiles/boot/grub/stage2_eltorito' for update.
</pre>

The problem is that the -boot-info-table command line argument patches the boot file to include information about the CD, but the boot file is not writable by genisoimage. GRUB needs the boot file to be patched in order to access the CD properly. The solution is to make it writeable, like this:

<syntaxhighlight lang="bash">
chown myself isofiles/boot/grub/stage2_eltorito    # Make sure you are the owner of the file. Replace "myself"
                                                   # with your username. Maybe you need to use sudo.
chmod u+w isofiles/boot/grub/stage2_eltorito       # Make the file writable for the owner of the file (you).
</syntaxhighlight>

== See Also ==
=== Articles ===
* [[mkisofs]]
* [[Bootable CD]] - using floppy disk emulation
* [[GRUB]] - for info on using GRUB 2 to make a bootable CD with modern GRUB

=== Forum ===
* [[topic:19050|Thread about making your own .iso image from scratch without tools]]

=== External Links ===
* [http://www.geocities.com/imulgrew/grub_files.tar.gz Package containing the GRUB stages] (including <tt>stage2_eltorito</tt>)
* [ftp://alpha.gnu.org/gnu/grub/grub-0.97.tar.gz Source for GRUB Legacy] to build it yourself

[[Category:Tutorials]]
[[Category:Bootloaders]]
[[Category:GRUB]]
````

<a id="w002417"></a>

## W002417 — OSDevWiki:About

Bron-ID: W002417
Namespace: Main (0)
Revisie: 6569 | Datum: 2008-11-04T15:12:46Z
Bron: https://wiki.osdev.org/OSDevWiki:About
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDevWiki%3AAbout&oldid=6569
Doorverwijzing: OSDev Wiki:About

````text
#REDIRECT [[OSDev Wiki:About]]
````

<a id="w002418"></a>

## W002418 — OSDevWiki:Editing

Bron-ID: W002418
Namespace: Main (0)
Revisie: 6570 | Datum: 2008-11-04T15:14:59Z
Bron: https://wiki.osdev.org/OSDevWiki:Editing
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDevWiki%3AEditing&oldid=6570
Doorverwijzing: OSDev Wiki:Editing

````text
#REDIRECT [[OSDev Wiki:Editing]]
````

<a id="w002419"></a>

## W002419 — OSDevWiki:Joining

Bron-ID: W002419
Namespace: Main (0)
Revisie: 6574 | Datum: 2008-11-04T15:21:34Z
Bron: https://wiki.osdev.org/OSDevWiki:Joining
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDevWiki%3AJoining&oldid=6574
Doorverwijzing: OSDev Wiki:Joining

````text
#REDIRECT [[OSDev Wiki:Joining]]
````

<a id="w002420"></a>

## W002420 — OSDev Wiki talk:Manual Of Style

Bron-ID: W002420
Namespace: OSDev Wiki talk (5)
Revisie: 6615 | Datum: 2008-11-04T21:56:40Z
Bron: https://wiki.osdev.org/OSDev_Wiki_talk:Manual_Of_Style
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDev_Wiki_talk%3AManual_Of_Style&oldid=6615

````text
== Merging ==

IMHO this page should be merged with and moved to [[OSDev Wiki:Manual Of Style]]. What do you think? - [[User:Walling|Walling]] 15:44, 4 November 2008 (UTC)
: Technically this page should be reverted to its redirect state. I must have missed it when the author replaced a redirect to the officially approved manual of style with their own version. - [[User:Combuster|Combuster]] 16:31, 4 November 2008 (UTC)
:: Yes, that sounds like a good idea. Although I like the (slightly more complete) example using tables. I'm not sure I get the thing about [[Manual Of Style#Notes|"Notes"]], and it took a while before I figured out that [[Manual Of Style#See Also|"See Also"]] is an example, and not actually a See Also section. I see you have a draft for a new Manual Of Style. I like that. What about we remove all links to this page and link to the one in the OSDev Wiki-namespace and delete/redirect this page. The old redirect is broken anyway, because it was to OSDevWiki-namespace (without space).. confusing. :) - [[User:Walling|Walling]] 16:55, 4 November 2008 (UTC)
::: Go ahead and merge. I replaced the page back then because the redirect didn't work. I didn't know about the page in the OSDev Wiki namespace. Actually, I didn't know about any of those pages. They are not easy to find. About the notes thing, I used it in the table on [[X86 Instruction Encoding#Operand-size and address-size override prefix|this]] page. [[User:Virtlink|Virtlink]] 17:10, 4 November 2008 (UTC)
:: Okay. That's done now. I merged them and set up a redirect. I think the reason the pages on OSDev Wiki namespace is hard to find is because of a lot of broken links. I've tried to fix that. - [[User:Walling|Walling]] 19:28, 4 November 2008 (UTC)

Moved the talkpage here as well. Removed the boldface rule since that doesn't make much sense here where a lot of pages aren't constituting a definition, but often enough, a concept, compact description, question or tutorial. The new elaborations are fine but the other new rules should have been discussed ''prior'' to editing. It kindof feels like introducing wikipedia concepts and skipping the "make sure edits reflect consensus" part they put on top (maybe it was a bad idea to put it at the bottom here) - [[User:Combuster|Combuster]] 20:22, 4 November 2008 (UTC)
: Well, maybe we should move some of it to [[OSDev Wiki:Editing]] to reflect that it's not rules you need to follow, but rather help in editing articles? - [[User:Walling|Walling]] 21:56, 4 November 2008 (UTC)
````

<a id="w002422"></a>

## W002422 — Network devices:Cards

Bron-ID: W002422
Namespace: Main (0)
Revisie: 6596 | Datum: 2008-11-04T16:17:14Z
Bron: https://wiki.osdev.org/Network_devices:Cards
Revisie-URL: https://wiki.osdev.org/index.php?title=Network_devices%3ACards&oldid=6596
Doorverwijzing: Ethernet devices

````text
#REDIRECT [[Ethernet devices]]
````

<a id="w002423"></a>

## W002423 — Talk:Manual Of Style

Bron-ID: W002423
Namespace: Talk (1)
Revisie: 6607 | Datum: 2008-11-04T20:12:58Z
Bron: https://wiki.osdev.org/Talk:Manual_Of_Style
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AManual_Of_Style&oldid=6607
Doorverwijzing: OSDev Wiki talk:Manual Of Style

````text
#REDIRECT [[OSDev Wiki talk:Manual Of Style]]
````

<a id="w002424"></a>

## W002424 — Ar

Bron-ID: W002424
Namespace: Main (0)
Revisie: 6610 | Datum: 2008-11-04T20:36:26Z
Bron: https://wiki.osdev.org/Ar
Revisie-URL: https://wiki.osdev.org/index.php?title=Ar&oldid=6610
Doorverwijzing: Link Archiver

````text
#REDIRECT [[Link_Archiver]]
````

<a id="w002427"></a>

## W002427 — FPU

Bron-ID: W002427
Namespace: Main (0)
Revisie: 29122 | Datum: 2024-06-08T14:58:12Z
Bron: https://wiki.osdev.org/FPU
Revisie-URL: https://wiki.osdev.org/index.php?title=FPU&oldid=29122

````text
{{Floats}}
The x86 FPU was originally an optional addition to the processor that was able to perform floating point math in hardware, but has since been integrated into the CPU proper and has collected over the years the majority of math-heavy instructions. The modern FPU has become a legacy term for what is actually the vector processing units, which just happens to include the original floating point operations.

== x86 FPU Legacy ==

Originally, the FPU was a dedicated coprocessor chip placed on top of the actual processor. Since it was performing calculations asynchronously from the core logic, its results would have been available after the main processor has executed several other instructions. Since errors would also become available asynchronously, the original PC had the error line of the FPU wired to the [[PIC|interrupt controller]]. When the 486 added multiprocessor support, it became impossible to detect which of the FPUs has raised an exception, after which they integrated the FPU on-die and added an option to signal a regular exception rather than an interrupt. To provide backwards compatibility, the 486 was given a pin to replace the original FPU error line, which would be routed to the PIC and then back into the CPU's IRQ line to simulate the original setup with a dedicated coprocessor. This has the unfortunate consequence that by default, floating point exceptions will not operate as recommended by the manual.

== FPU configuration ==

Due to the many forms of FPUs and vector units, some logic is required to get them in the expected state.

== Detecting an FPU ==
On x86 processors up to the 386, FPUs were external and strictly optional. They allowed the use of different floating-point units, including those which did not strictly correspond to the processor's generation. For example, the 386 was capable of operating with both a 287 (the FPU corresponding to the 286), and the 387 (the contemporary FPU). The 486 line of microprocessors was bifurcated into the 486DX, which included an on-chip floating-point unit, and the 486SX, which did not. The external 487 coprocessor was essentially a modified 486DX that disabled the installed CPU. All x86 CPUs from the Pentium onward have an integrated FPU present (excluding the [https://en.wikipedia.org/wiki/NexGen NexGen 5x86]).

There are two ways to detect an FPU:
# Check the FPU bit in CPUID
# Check the EM bit in CR0, if it is set then the FPU is not meant to be used.
# Check the ET bit in CR0, if it is clear, then the CPU did not detect an 80387 on boot
# Probe for an FPU
The correct order is a bit doubtful. The current official manuals state that attempts to use the FPU when one is not present will lock up the CPU. There are however many sources that contain probing code to various degrees of complexity, with the common consensus that fwait or actual calculations are not to be performed. Similarly, the EM and ET bits can be modified by code and might not have the right values. Different wirings on actual hardware may also cause 386s to not detect an FPU as a 80386, causing the ET bit to have the wrong value on boot.

The common way of testing the presence of an FPU is to have it write it's status somewhere and then check if it actually did.
<source lang="asm">
MOV EDX, CR0                            ; Start probe, get CR0
AND EDX, (-1) - (CR0_TS + CR0_EM)       ; clear TS and EM to force fpu access
MOV CR0, EDX                            ; store control word
FNINIT                                  ; load defaults to FPU
FNSTSW [.testword]                      ; store status word
CMP word [.testword], 0                 ; compare the written status with the expected FPU state
JNE .nofpu                              ; jump if the FPU hasn't written anything (i.e. it's not there)
JMP .hasfpu

.testword: DW 0x55AA                    ; store garbage to be able to detect a change
</source>

To distinguish a 287 and a 387 FPU, you can try if it can see the difference between +infinity and -infinity.

== FPU control ==
If an FPU is found to be present, you should set up the control registers accordingly. If an FPU is not present, you should also set up the registers accordingly.

'''CR0.EM''' (bit 2; counting starts at bit 0 making this the third bit)
:If the EM bit is set, all FPU and vector operations will cause a #UD so they can be '''EM'''ulated in software. Should be off to be actually able to use the FPU
'''CR0.ET''' (bit 4)
:This bit is used on the 386 to tell it how to communicate with the coprocessor, which is 0 for an 287, and 1 for a 387 or later. This bit is hardwired to 1 on 486+
'''CR0.NE''' (bit 5)
:When set, enables '''N'''ative '''E'''xception handling which will use the FPU exceptions. When cleared, an exception is sent via the interrupt controller. Should be on for 486+, but not on 386s because they lack that bit.
'''CR0.TS''' (bit 3)
:Task switched. The FPU state is designed to be lazily switched to save read and write cycles. If set, all meaningful operations will cause a #NM exception so that the OS can backup the FPU state. This bit is automatically set on a hardware task switch, and can be cleared with the CLTS opcode. Software task switching may want to manually set this bit on a reschedule if they want to lazily store FPU state.
'''CR0.MP''' (bit 1)
:This does little else other than saying if an FWAIT opcode is exempted from responding to the TS bit. Since FWAIT will force serialisation of exceptions, it should normally be set to the inverse of the EM bit, so that FWAIT will actually cause a FPU state update when FPU instructions are asynchronous, and not when they are emulated.

'''CR4.OSFXSR''' (bit 9)
:Enables 128-bit SSE support. When clear, most SSE instructions will cause an invalid opcode, and FXSAVE and FXRSTOR will only include the legacy FPU state. When set, SSE is allowed and the XMM and MXCSR registers are accessible, which also means that your OS should maintain those additional registers. Trying to set this bit on a CPU without SSE will cause an exception, so you should check for SSE (or long mode) support first.
'''CR4.OSXMMEXCPT''' (bit 10)
:Enables the #XF exception. When clear, SSE will work until an exception is generated, after which all SSE instructions will fail with an invalid opcode. When set, the exception handler is called instead and the problem may be diagnosed and reported. Again, you can't set this bit without ensuring SSE support is present
'''CR4.OSXSAVE''' (bit 18)
:Enables the XSAVE extension, which is able to save SSE state as well as other next-generation register states. Again, check CPUID before setting: Long mode support is not sufficient in this case.

== Vector unit ==
MMX, 3DNow and the rare EMMI reuse the old FPU registers as vector units, aliasing them into 64 bit data registers. This means that they can be used safely without modifications of the FPU handling. SSE however adds a whole new set of registers, and therefore is disabled by default. To allow SSE instructions, CR4.OSFXSR should be set. Be careful though since writing it on a processor without SSE support causes an exception. When SSE is enabled, FXSAVE and FXRSTOR should be used to store the entire FPU and vector register file.
It is good practice to enable the other SSE bit (CR4.OSXMMEXCPT) as well so that SSE exceptions are routed to the #XF handler, instead of your vector unit automatically disabling itself when an exception occurs.
The state of the art includes AVX, which adds

== Long Mode ==
Long mode demands that SSE and SSE2 are available, and compilers are free to use the SSE registers instead of the old FPU registers for floating point operations. This means that your kernel will need to have SSE enabled before using any floating point operations, whereas 32-bit mode might just happen to work without touching CR0/CR4. Also, long mode doubles the registers for SSE, giving you 16 XMM registers rather than the 8 available in 32-bit mode, which implies that more data is in need of saving.


== FPU state ==
When the FPU is configured, the only thing left to do is to initialize its registers to their proper states. FNINIT will reset the user-visible part of the FPU stack. This will set precision to 64-bit and rounding to nearest, which should be correct for most operations. It will also mask all exceptions from causing an interrupt. You can change the control by issuing an FLDCW. To diagnose broken code, you usually want to enable exceptions for invalid operands and stack overflows (bit 0). Bit 2 allows you to catch divisions by zero as well. Some examples:
<source lang="asm">; FLDCW requires a 16-bit memory operand, immediates do not work
FLDCW [value_37F]   ; writes 0x37f into the control word: the value written by F(N)INIT
FLDCW [value_37E]   ; writes 0x37e, the default with invalid operand exceptions enabled
FLDCW [value_37A]   ; writes 0x37a, both division by zero and invalid operands cause exceptions.</source>


Using the MMX aliases for the FPU registers will cause those registers to be invalidated for floating point use. The EMMS instruction will reset the registers to non-vector use. The x86 calling convention assumes that the stack is usable for either floating point or vector use, so you will need to call EMMS before calling or returning to regular compiler-generated code. Both MMX instructions and EMMS preserves the control word you set with FLDCW so you don't need to adjust it manually afterwards.

SSE operates mostly independent of the FPU registers. It has a separate MXCSR register which deals with control and exceptions, which should be written separately.

== Programming the FPU ==
The x87 FPU contains 8 floating point registers.  Each floating point register holds an 80-bit extended double value (1-bit sign, 15-bit exponent, and 64-bit fractional value), and each register has a matching 2-bit "tag" value in the Tag Register that acts as that registers flags.  The Tag Register contains information about whether each register is empty or not, whether its value is accurate or not, and whether its value is a special value, like "infinity" or "Nan" (Not a number).

The 8 floating point registers are organized in a "stack" configuration, and most FPU instructions operate on the current "top" of the stack, (which is register 0 by default).  The current "top" register index is stored in the FPU Status Register, and is updated automatically by the FPU when a PUSH (FLD) or POP (FST) instruction is executed.  When all 8 stack registers are full (i.e. all 8 tag registers are not marked as "empty"), and a PUSH instruction is executed, a FPU stack-overflow exception will occur.  If the stack-overflow exception interrupt has been enabled, the main CPU will also receive an interrupt.

Because the x87 FPU is a separate processor (and has its own clock), it can execute FPU instructions in parallel, at the same time as the CPU is executing its own instructions.  Applications that use the x87 FPU must first execute one or more FPU commands, then at some later point, it must instruct the main CPU to wait for the FPU to finish processing (FWAIT) in order to ensure that the FPU has finished executing those instructions.  Most of the x87 FPU instructions have a "wait" version and a "no wait" version, so that the programmer can specify at which points in the application that the two processors need to be synchronized.  After the FWAIT CPU instruction is executed (or a "wait" instruction is executed), any calculations performed by the FPU (or any exceptions that have been detected by the FPU) can be addressed by the application at that point.

Sending data to, and pulling data from the 8 FPU registers, ST(0) through ST(7), must be performed using system memory.  It is not possible to directly copy values from a CPU register to an FPU register.  The FPU can copy data from/to system memory in the following formats: 16-Bit Integer, 32-Bit Integer, 32-Bit Float (single), 64-Bit Float (double), and 80-bit Float (extended double).  The FPU also supports reading and writing a 80-bit Binary Coded Decimal (BCD) format, which contains a single "sign" bit, 7 reserved bits, and 18 four-bit hexadecimal "characters".

When reading values from system memory, the extended double format is copied directly into the FPU register, while the other formats are converted to the 80-Bit extended double format before being stored in the FPU register.  When writing values to system memory, the 80-bit value is copied directly when storing the extended double format, and is converted to the appropriate structure for the other formats.  This conversion includes rounding the value based on the current rounding settings in the FPU Control Register.

== Rent-a-coder ==
These functions can be used with GCC (or TCC) to perform some FPU operations without resorting to dedicated assembly:
<source lang="c">void fpu_load_control_word(const uint16_t control)
{
    asm volatile("fldcw %0;"::"m"(control)); 
}</source>

==See Also==
===External Links===
* [http://www.website.masmforum.com/tutorials/fptute/ Simply FPU], a practical guide covering the FPU basics in a userland perspective
* [http://www.ragestorm.net/downloads/387intel.txt Intel 80387 Programmer's Reference Manual], complete with example code
* [http://developer.amd.com/documentation/guides/pages/default.aspx#manuals AMD Programmer's Manuals], has FPU instruction reference conveniently ordered by processor component.
* [http://www.intel.com/products/processor/manuals/ Intel 64-bit Manuals], the Intel version of the manuals. More complete, but also more bloated.

[[Category:X86]]
[[de:FPU_(x86)]]
````

<a id="w002428"></a>

## W002428 — Talk:Detecting Memory

Bron-ID: W002428
Namespace: Talk (1)
Revisie: 6635 | Datum: 2008-11-09T10:56:30Z
Bron: https://wiki.osdev.org/Talk:Detecting_Memory
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ADetecting_Memory&oldid=6635

````text
== Merge ==
Pcmattman, this information should IMHO be merged with [[Detecting Memory (x86)]] (or vice versa). [[User:Virtlink|Virtlink]] 10:56, 9 November 2008 (UTC)
````

<a id="w002429"></a>

## W002429 — Optimizing

Bron-ID: W002429
Namespace: Main (0)
Revisie: 25236 | Datum: 2020-10-04T20:52:05Z
Bron: https://wiki.osdev.org/Optimizing
Revisie-URL: https://wiki.osdev.org/index.php?title=Optimizing&oldid=25236

````text
'''Optimization''' is the practise of taking functioning code and modifying it in order to make it execute faster. Optimization is already discussed very thoroughly in other places for various userland problem domains, and below are a few areas that are important for operating system developers to consider.

Note that to fully understand the ideas on this page, it is important to have fairly good grasp of assembly (x86 examples will be used), and a thorough knowledge of the x86 architecture is a must. But you've already read the Intel CPU manuals, haven't you?

==When and What to Optimize==
Please take the following list with some common sense: your kernel might be different.

====Things to Optimize====
* '''Deeply-nested loops.''' If possible, try to eliminate some of the nesting of the loop. Any instruction taken out of a loop will have a ''much'' more significant speed impact than an instruction taken out of a general routine. If you run though a loop one million times, and there is just one "unnecessary line of code", then there is a significant portion of a second wasted, especially on slower computers. 
* '''Code that is run often.'''
* '''Graphics code.'''
* '''API code.'''

====Things not to bother Optimizing====
* '''Boot code.''' Code that only runs once (and doesn't take very long, such as loading an [[IDT]]), is not worth optimizing.

====Things not to Optimize====
* '''Untested or nonworking code.''' If you try to prematurely optimize, you will almost certainly make any problems you might have worse. 'First make it run, then make it run fast.'
* '''Code that needs to retain clarity.''' It is generally fairly hard to keep optimized code looking as neat and understandable as code that is designed to be easy to understand. Comments can help (and should be used anyway, especially to mark why optimizations were made).
* '''Crash recovery code.''' This will depend on your personal stance. It may be a good idea to err on the side of caution and concentrate on stability of the recovery code, not speed.

==Compiler Optimization==

Modern compilers (such as [[GCC]]) provide inbuilt optimization logic, which is available to you without having to write (or remove) a single line of code. Enabling these compiler optimizations might seem to break previously functioning code; it is important to remember that this is because of bugs in your code that have been there all along, that are just exposed by the optimizations. One such example is when the [[Volatile (keyword)|volatile]] keyword is not used where necessary.

Another side-effect of optimization is that source-level debugging might not always produce the expected results, as variables or whole lines of code are optimized away.

===GCC===

To use compiler optimizations in GCC, simply add the -O(x) switch to the command line when compiling, where (x) is one of the following:

{| {{wikitable}}
! Switch
! Use
|-
| 0 || Disable optimization. (This is the default.)
|-
| 1 || Try to reduce code size and execution time, without performing any optimizations that take a great deal of compilation time.
|-
| 2 || Perform nearly all supported optimizations that do not involve a space-speed tradeoff.
|-
| 3 || Perform nearly all supported optimizations ''including'' those involving a space-speed tradeoff.
|-
| fast || Optimize for speed. This includes all optimizations from -O3, as well as aggressive optimizations that may violate strict compliance with language standards.  (For example, use a fused multiply-add instruction for floating point numbers instead of multiplying and adding separately.)
|-
| s || Optimize for size. This includes all optimizations from -O2 that do not typically increase code size, plus additional size-reducing optimizations. 
|-
| g || Turn on optimizations that do not interfere with debugging (see the [https://gcc.gnu.org/onlinedocs/gcc/Optimize-Options.html GCC docs]). Available since 4.8.
|}

Numerous sources suggest the existence of further optimization levels, like "-O4", "-O6" or "-O99"; but only -O0 through -O3 and -Os (and -Og since 4.8) are ''documented'' and can be relied on.

For more information, read the GCC manual; it lists the exact optimizations done at each level, plus some additional options not enabled at any of the above optimization levels.

===Clang===

Clang supports all of the same optimization flags as GCC, in addition to <code>-Oz</code> which is similar to <code>-Os</code> but reduces code size even more.

'''ToDo: Describe Link-time optimizations.'''

==Inline Functions==
The '''inline''' keyword is a way of requesting that the compiler insert the code instead of calling it. This means that if you create an inline function and then call it from another function, the compiler will try to replace your call with the function body (similar to a preprocessor define, but with none of its disadvantages). 

Inline functions save the clock cycles involved in actually calling a function (argument and return code pushes, call and return jump). Compared to preprocessor defines, they add type safety, and avoid side effects that make macros a hazard. Inlining can also be triggered on and off using compiler options so you can measure the size / speed tradeoff. On the downside, a normal function called from five different locations will only reside in memory once. If you inline that function, it will reside in memory five times. Your code becomes faster, but it also becomes bigger. It is important to note that the inline keyword is merely a request to the compiler, which the compiler can choose to ignore if it deems the size / speed tradeoff to be detrimental.

It is best to only use the '''inline''' keyword on small functions, where it is known to work best and to provide the largest speed boost. An example of small function is an accessor or a mutator inside a C++ class, or a function that simply sets the value. The inline keyword is simply specified before the function type, just like the '''static''' keyword:

<pre>
inline int max_of_three(int x, int y, int z)
{
    if ( x > y && x > z )
        return x;
    return ( y > z ) ? y : z;
}
</pre>

==The Cache==
{{Main|CPU Caches}}
The CPU's cache contains a high-bandwidth store of currently used data. On a modern CPU, it usually takes 2 cycles to do a read or write from level 1 (L1) cache (compared with 1 cycle for a register), 5 cycles for level 2 (L2) cache, and perhaps around 30 cycle for main memory. As you can see, reading from memory is very expensive compared to reading from a register or cache. Unfortunately, this trade-off for speed comes at a size cost: L1 cache is only around 8KiB per processor core, with L2 cache sitting around 4MiB to 12MiB per Core 2 chip. For more information, see [[Wikipedia:Memory_hierarchy|this]] Wikipedia entry.

Most processors try to keep the most recently used data in the two caches (though the method used can differ, check the manuals). For this reason, it pays to keep in mind what data is likely to be in the caches at any given time. If you, for example (and this is a very simplistic example), set your multitasking code to switch processes too fast, you'll waste a higher percentage of time going through cache 'flushes', refilling the cache with new data.

==CPU Pipelines==
{{Stub}}

==See Also==
===Articles===
* [[User:01000101/optlib/|01000101's Optimized Library]]

===Threads===
* [[topic:18119|01000101 asks about optimizing memcpy() and memset()]]
* [[topic:18063|A discussion on the best way to do multi-processor scheduling]]

===External Links===
* [http://www.eventhelix.com/realtimemantra/basics/optimizingcandcppcode.htm Handy tips on optimizing C/C++ code.]
* [https://gcc.gnu.org/onlinedocs/gcc/Optimize-Options.html GCC Optimization Documentation.]
[[de:Codeoptimierung]]
````

<a id="w002430"></a>

## W002430 — Talk:Optimizing

Bron-ID: W002430
Namespace: Talk (1)
Revisie: 7058 | Datum: 2009-01-12T16:34:21Z
Bron: https://wiki.osdev.org/Talk:Optimizing
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AOptimizing&oldid=7058

````text
== Where to optimise ==
Two things: 
* I'm totally missing the bit on the algorithmic complexity thing compared to make the processor do as much at possible in a clock - If you can improve the complexity of an algorithm to something better the improvements are vastly more significant than when you optimise for occupying every processing unit. 
* There's currently a bit on the execution architecture of the Pentium 1 - which is for current standards pretty much in-order architecture. Modern processors have several (IIRC, netburst had 5) execution ports, which gets assigned an instruction (or uop) and start working on them. The result is that P1 optimisation isn't necessarily good optimisation for later architectures. It might be a better idea to explain things more general (AGI stalls, Partial register stalls, register renaming, dependency breaking/delaying, loop blocking) as techniques to get more out of the average case - breaking dependencies will help on all superscalar architectures, and the current UV-optimisation example is just one special case for that, but not necessarily best for later processors.
- [[User:Combuster|Combuster]] 21:47, 18 November 2008 (UTC)

:: Also on the high level there's simple tricks as function inlining (can be good for small functions), which takes away the overhead of the call. It should indeed be done for (inline) assembly functions, but might benefit other areas too like a linked list implementation. And what about compilers like [http://clang.llvm.org/ clang] that can perform Link Time Optimizations? GCC doesn't necessarily set the standard of optimizations. - [[User:Walling|Walling]] 13:33, 19 November 2008 (UTC)

== Compiler Optimisations ==
:''"One thing to be aware of when using compiler optimizations is that when overdone (or even done at all) they can cause previously functioning code to become buggy, or not work at all."''

I have a problem with that. If enabling a compiler optimization results in your code malfunctioning, your code was already broken beforehand, you just didn't realize it. ''volatile'' is not a patch around buggy compilers, it's an error when you ''don't'' set it where necessary. -- [[User:Solar|Solar]] 10:36, 18 November 2008 (UTC)

:That's a good point. When I was thinking up that sentence though, I was thinking of bugs in the compiler (not many, but they do exist). But your scenario is much more likely. [[User:Jackscott|Jackscott]] 10:42, 18 November 2008 (UTC)
::Fixed. [[User:Jackscott|Jackscott]] 07:28, 20 November 2008 (UTC)

:Usually the ''optimizing'' code path of the compiler is even better tested than the non-optimizing one, because it's more often used... ;-) [[User:Solar|Solar]] 16:34, 12 January 2009 (UTC)
````

<a id="w002431"></a>

## W002431 — Talk:Application-Security-Books

Bron-ID: W002431
Namespace: Talk (1)
Revisie: 6688 | Datum: 2008-11-18T21:51:19Z
Bron: https://wiki.osdev.org/Talk:Application-Security-Books
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AApplication-Security-Books&oldid=6688

````text
IMHO, this should be added to the [[Books]] page. [[User:Jackscott|Jackscott]] 10:47, 18 November 2008 (UTC)
:I don't personally know anything about these books, but IMO its best to keep all books together on one page, possibly with headers to group them. - [[User:Combuster|Combuster]] 21:51, 18 November 2008 (UTC)
````

<a id="w002432"></a>

## W002432 — User:Yayyak

Bron-ID: W002432
Namespace: User (2)
Revisie: 6693 | Datum: 2008-11-20T07:30:24Z
Bron: https://wiki.osdev.org/User:Yayyak
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AYayyak&oldid=6693
Doorverwijzing: User:Jackscott

````text
#REDIRECT [[User:Jackscott]]
````

<a id="w002433"></a>

## W002433 — FAQ

Bron-ID: W002433
Namespace: Main (0)
Revisie: 6702 | Datum: 2008-11-21T11:37:20Z
Bron: https://wiki.osdev.org/FAQ
Revisie-URL: https://wiki.osdev.org/index.php?title=FAQ&oldid=6702
Doorverwijzing: Category:FAQ

````text
#REDIRECT [[:Category:FAQ]]
````

<a id="w002434"></a>

## W002434 — IOAPIC

Bron-ID: W002434
Namespace: Main (0)
Revisie: 30694 | Datum: 2026-06-07T16:19:01Z
Bron: https://wiki.osdev.org/IOAPIC
Revisie-URL: https://wiki.osdev.org/index.php?title=IOAPIC&oldid=30694

````text
{{DISPLAYTITLE:I/O APIC}}

The Intel I/O Advanced Programmable Interrupt Controller is used to distribute external interrupts in a more advanced manner than that of the standard [[8259 PIC]]. With the I/O APIC, interrupts can be distributed to physical or logical (clusters of) processors and can be prioritized. Each I/O APIC typically handles 24 external interrupts.

== Detecting I/O APIC ==
In order to detect the existence of an I/O APIC (or multiple ones), the Intel Multi-Processor or [[RSDP|ACPI]] tables (specifically, the [[MADT]]) must be parsed. In the MP tables, configuration tables with the entry identification of 0x01 are for I/O APICs. Parsing will tell how many (if any) I/O APICs exist, what are their APIC ID, base MMIO address and first IRQ (or GSI - Global System Interrupt). For more information on parsing the MP tables, see the External MP Tables Links section below. So you can have, say, 2 I/O APICs, the first handling IRQs 0 - 23 and the second 24 - 47.

== Programming the I/O APIC ==
Each I/O APIC has a set of 2 or 3 (depending on version) 32-bit registers and up to many 64-bit registers (one per IRQ). The 64-bit registers have to be accessed as two 32-bit reads/writes. All registers are memory indexed. It means that you actually have only two 32-bit registers in memory, called IOREGSEL and IOREGWIN. You put the register index in IOREGSEL, and then you can read/write in IOREGWIN. The first three registers contain general information about this I/O APIC, while the remaining registers contain the specific configuration for each IRQ.

=== IOAPICID ===
This register has index 0 (you write 0 to IOREGSEL and then read from IOREGWIN). It's a Read-Only register with almost all bits reserved. The only interesting field is in bits 24 - 27: the APIC ID for this device (each peripheral which is interfaced with the APIC Bus needs an APIC ID, not only CPUs). You shall find this ID in ACPI/MP Tables as well.

=== IOAPICVER ===
This register (index 1) contains the I/O APIC Version in bits 0 - 7, and the '''Max Redirection Entry''' which is "how many IRQs can this I/O APIC handle - 1". It is encoded in bits 16 - 23.

=== IOAPICARB ===
This register (index 2) contains in bits 24 - 27 the APIC Arbitration ID. '''TODO'''

=== IOREDTBL ===
Following that there are two 32-bit registers for each IRQ. The first IRQ has indexes 0x10 and 0x11, the second 0x12 and 0x13, the third 0x14 and 0x15, and so on. So the ''Redirection Entry'' register for IRQ ''n'' is 0x10 + n * 2 (+ 1). In the first of the two registers you access to the LOW uint32_t / bits 31:0, and the second for the high uint32_t / 63:32. Each redirection entry is made of the following fields:
{| class="wikitable"
|-
! Field
! Bits
! Description
|-
| style="width: 100px;" | Vector 
| style="width: 50px;" | 0 - 7
| The Interrupt vector that will be raised on the specified CPU(s).
|-
| Delivery Mode
| 8 - 10
| How the interrupt will be sent to the CPU(s). It can be 000 (Fixed), 001 (Lowest Priority), 010 (SMI), 100 (NMI), 101 (INIT) and 111 (ExtINT). Most of the cases you want Fixed mode, or Lowest Priority if you don't want to suspend a high priority task on some important Processor/Core/Thread.
|-
| Destination Mode
| 11
| Specify how the Destination field shall be interpreted. 0: Physical Destination, 1: Logical Destination
|-
| Delivery Status
| 12
| If 0, the IRQ is just relaxed and waiting for something to happen (or it has fired and already processed by Local APIC(s)). If 1, it means that the IRQ has been sent to the Local APICs but it's still waiting to be delivered.
|-
| Pin Polarity
| 13
| 0: Active high, 1: Active low. For ISA IRQs assume Active High unless otherwise specified in Interrupt Source Override descriptors of the MADT or in the MP Tables.
|-
| Remote IRR
| 14
| '''TODO'''
|-
| Trigger Mode
| 15
| 0: Edge, 1: Level. For ISA IRQs assume Edge unless otherwise specified in Interrupt Source Override descriptors of the MADT or in the MP Tables.
|-
| Mask
| 16
| Just like in the old PIC, you can temporary disable this IRQ by setting this bit, and reenable it by clearing the bit.
|-
| Destination
| 56 - 63
| This field is interpreted according to the Destination Format bit. If Physical destination is choosen, then this field is limited to bits 56 - 59 (only 16 CPUs addressable). You put here the APIC ID of the CPU that you want to receive the interrupt. '''TODO''': Logical destination format...
|}

=== IOREGSEL and IOWIN ===
The register IOREGSEL is an MMIO register select register that is used to access all the other I/O APIC registers. The IOWIN register is the 'data' register. Once the IOREGSEL register has been set, the IOWIN register can be used to write or read the register in the IOREGSEL. The actual position in memory of the two registers is specified in the ACPI MADT Table and/or in the MP table. The IOREGSEL is at the address specified, and IOREGWIN is at the same address + 0x10.

<syntaxhighlight lang="cpp">

#define IOAPICID          0x00
#define IOAPICVER         0x01
#define IOAPICARB         0x02
#define IOAPICREDTBL(n)   (0x10 + 2 * n) // lower-32bits (add +1 for upper 32-bits)

void write_ioapic_register(const uintptr_t apic_base, const uint8_t offset, const uint32_t val) 
{
    /* tell IOREGSEL where we want to write to */
    *(volatile uint32_t*)(apic_base) = offset;
    /* write the value to IOWIN */
    *(volatile uint32_t*)(apic_base + 0x10) = val; 
}
 
uint32_t read_ioapic_register(const uintptr_t apic_base, const uint8_t offset)
{
    /* tell IOREGSEL where we want to read from */
    *(volatile uint32_t*)(apic_base) = offset;
    /* return the data from IOWIN */
    return *(volatile uint32_t*)(apic_base + 0x10);
}

/* @class IOAPIC
 *
 * A sample driver code which control an IOAPIC. It handles one IOAPIC and exposes
 * some functions. It is totally representational, .i.e you should add locking support
 * link all IOAPIC classes in a data structure and much more. Here we are just showing
 * what & how your'e gonna handle this in C++.
 *
 * You could also note that IOAPIC registers "may" cross a page boundary. So maybe you
 * may need to map the physical-base to a double-page (means allocate twice the amount
 * of memory from vmm).
 */
class IOAPIC
{
public:
        enum DeliveryMode
        {
                EDGE  = 0,
                LEVEL = 1,
        };

        enum DestinationMode
        {
                PHYSICAL = 0,
                LOGICAL  = 1
        };

        union RedirectionEntry
        {
                struct
                {
                        uint64_t vector       : 8;
                        uint64_t delvMode     : 3;
                        uint64_t destMode     : 1;
                        uint64_t delvStatus   : 1;
                        uint64_t pinPolarity  : 1;
                        uint64_t remoteIRR    : 1;
                        uint64_t triggerMode  : 1;
                        uint64_t mask         : 1;
                        uint64_t reserved     : 39;
                        uint64_t destination  : 8;
                 };
                 struct
                 {
                        uint32_t lowerDword;
                        uint32_t upperDword;
                 };
        };

        unsigned char id(){ return (apicId); }
        unsigned char ver(){ return (apicVer); }
        unsigned char redirectionEntries(){ return (redirEntryCnt); }
        unsigned long globalInterruptBase(){ return (globalIntrBase); }

        IOAPIC::IOAPIC(unsigned long physRegs, unsigned long apicId, unsigned long gsib)
        {
                this->virtAddr = KernelMemory::allocatePage(PAGE_SIZE);

                /* map virtAddr to physical-regs. Note that your paging code may not support 
                   automatically aligning physRegs to page-boundaries. Be sure to check! */
                EnsureMapping(virtAddr, physRegs, PagePresent | PageReadWrite | PageCacheDisable);
                
               virtAddr += physRegs % PAGE_SIZE;

               apicId = (read(IOAPICID) >> 24) & 0xF0;
               apicVer = read(IOAPICVER);// cast to uint8_t (unsigned char) hides upper bits

               //< max. redir entry is given IOAPICVER[16:24]
               redirEntryCnt = (read(IOAPICVER) >> 16) + 1;// cast to uint8_t occuring ok!
               globalIntrBase = gsib;
        }

        /*
         * Bit of assignment here - implement this on your own. Use the lowerDword & upperDword
         * fields of RedirectionEntry using
         *                                 ent.lowerDword = read(entNo);
         *                                 ent.upperDword = read(entNo);
         *                                 return (ent);
         *
         * Be sure to check that entNo < redirectionEntries()
         *
         * @param entNo - entry no. for which redir-entry is required
         * @return entry associated with entry no.
         */
        RedirectionEntry getRedirEntry(unsigned char entNo);

        /*
         * Bit of assignment here - implement this on your own. Use the lowerDword & upperDword
         * fields of RedirectionEntry using
         *                               write(entNo, ent->lowerDword);
         *                               write(entNo, ent->upperDword);
         *
         * Be sure to check that entNo < redirectionEntries()
         *
         * @param entNo - entry no. for which redir-entry is required
         * @param entry - ptr to entry to write
         */
        void writeRedirEntry(unsigned char entNo, RedirectionEntry *entry);

private:
        /*
         * This field contains the physical-base address for the IOAPIC
         * can be found using an IOAPIC-entry in the ACPI 2.0 MADT.
         */
        unsigned long physRegs;

        /*
         * Holds the base address of the registers in virtual memory. This
         * address is non-cacheable (see paging).
         */
        unsigned long virtAddr;

        /*
         * Software has complete control over the apic-id. Also, hardware
         * won't automatically change its apic-id so we could cache it here.
         */
        unsigned char apicId;

        /*
         * Hardware-version of the apic, mainly for display purpose. ToDo: specify
         * more purposes.
         */
        unsigned char apicVer;

        /*
         * Although entries for current IOAPIC is 24, it may change. To retain
         * compatibility make sure you use this.
         */
        unsigned char redirEntryCnt;

        /*
         * The first IRQ which this IOAPIC handles. This is only found in the
         * IOAPIC entry of the ACPI 2.0 MADT. It isn't found in the IOAPIC
         * registers.
         */
        unsigned long globalIntrBase;

        /*
         * Reads the data present in the register at offset regOff.
         *
         * @param regOff - the register's offset which is being read
         * @return the data present in the register associated with that offset
         */
        uint32_t read(unsigned char regOff)
        {
                *(uint32_t volatile*) virtAddr = regOff;
                return *(uint32_t volatile*)(virtAddr + 0x10);
        }

        /*
         * Writes the data into the register associated. 
         *
         * @param regOff - the register's offset which is being written
         * @param data - dword to write to the register
         */
        void write(unsigned char regOff, uint32_t data)
        {
                *(uint32_t volatile*) virtAddr = regOff;
                *(uint32_t volatile*)(virtAddr + 0x10) = data;
        }
};
</syntaxhighlight>
'apic_base' is the memory base address for a selected IOAPIC, these can be found by enumerating them from the MP or ACPI Tables.


== IO APIC Inputs ==

How other hardware (devices, etc) use IO APIC inputs is completely arbitrary - the motherboard/chipset designer can hard-wire anything they like to any IO APIC input. For the motherboard designer's convenience, most but not all legacy IRQs are often (but not always) connected "1:1" to IO APIC inputs (e.g. IO APIC input #1 may be the same as PIC chip input #1) as this makes firmware a little easier (e.g. no need for "interrupt redirection entries" in ACPI's MADT/APIC table), but this is not a requirement of any standard and not something that useful operating system software can rely on.

To correctly determine what how IO APIC inputs are used (and how they must be configured - as active high or active low, and as edge triggered or level triggered) operating system software must either:

1) Use APIC's MADT/APIC table to determine how legacy IRQs are mapped to IO APIC inputs; then use ACPI's AML (with a suitable interpreter) to determine how PCI devices are connected to IO APIC inputs

2) Use Intel's "MultiProcessor Specification" tables to determine how both legacy IRQs and PCI IRQs are mapped to IO APIC inputs. Note that Intel's "MultiProcessor Specification" is deprecated (an operating system should use ACPI where possible, and fall back to MultiProcessor Specification tables if ACPI doesn't exist or can't be used)

3) Provide (many) motherboard specific drivers, where each driver is able to use motherboard specific information to determine how IO inputs are used

4) Use an excessively "clever" auto-detection scheme (with a high risk of misconfiguration and race conditions). These schemes typically begin with whatever information can be obtained easily (e.g. determining legacy IRQs from ACPI's MADT/APIC table), assuming everything else can be configured as "level triggered active low" (to suit PCI), and then asking device drivers to repeatedly forcing their device to generate an IRQ (while ruling out IO APIC inputs that weren't triggered within a certain period of time after the IRQ was caused) until the specific IO APIC that the device uses can be determined.

== External Links ==
=== MP Tables ===
* [https://web.archive.org/web/20121002210153/http://download.intel.com/design/archives/processors/pro/docs/24201606.pdf Intel MultiProcessor Specification]

=== I/O APIC ===
* [http://web.archive.org/web/20161130153145/http://download.intel.com/design/chipsets/datashts/29056601.pdf Intel 82093AA I/O APIC]
* [http://web.archive.org/web/20140713224739/http://download.intel.com/design/processor/manuals/253668.pdf Intel SDM 3A (see chapter 9.9)]

[[de:I/O Advanced Programmable Interrupt Controller]]
[[Category:Interrupts]]
[[Category:Multiprocessing]]
````

<a id="w002435"></a>

## W002435 — Setting Up Paging With PAE

Bron-ID: W002435
Namespace: Main (0)
Revisie: 28222 | Datum: 2023-07-10T20:25:37Z
Bron: https://wiki.osdev.org/Setting_Up_Paging_With_PAE
Revisie-URL: https://wiki.osdev.org/index.php?title=Setting_Up_Paging_With_PAE&oldid=28222

````text
{{Rating|1}}
{{In_Progress}}
This is a guide to setting up paging with [[PAE]] enabled. You should read [[Setting Up Paging]] first.

===Differences between PAE-Paging and Legacy-Paging===
*PAE allows you to access more physical memory, which is usually 64GiB (in fact, this is implementation specific).
*A new data structure is added, the so called 'Page-Directory-Pointer-Table'
*An entry is now 8-byte-wide (Legacy: 4-byte), so the number of entries is halved to 512 (Legacy: 1024)
*If the CPU supports it you can use the NoExecute-bit

===Setting Up The Data Structures===
As mentioned above the 'Page-Directory-Pointer-Table' is added, which contains 4 Page-Directory-Entries
<source lang="c">
uint64_t page_dir_ptr_tab[4] __attribute__((aligned(0x20))); // must be aligned to (at least)0x20, ...
    // ... turning out that you can put more of them into one page, saving memory
</source>
Keep in mind that the size of the CR3 register remains at 4byte, meaning that a PDPT must be located below 4GiB in physical memory.

Now we need our Page-Directory. For the sake of easiness, we'll use PSE. (2 MIB pages mapped in page directory)
<source lang="c">
// 512 entries
uint64_t page_dir[512] __attribute__((aligned(0x1000)));  // must be aligned to page boundary
</source>

===Making it run===
Ok, now we have our structures. Let's map the first 2 MIB.
<source lang="c">
page_dir_ptr_tab[0] = (uint64_t)&page_dir | 1; // set the page directory into the PDPT and mark it present
page_dir[0] = 0b10000011; //Address=0, 2MIB, RW and present
</source>

Pages are mapped. Now we have to set the PAE-bit in CR4 and load the PDPT into CR3
<source lang="c">
asm volatile ("movl %%cr4, %%eax; bts $5, %%eax; movl %%eax, %%cr4" ::: "eax"); // set bit5 in CR4 to enable PAE		 
asm volatile ("movl %0, %%cr3" :: "r" (&page_dir_ptr_tab)); // load PDPT into CR3
</source>

Finally, we'll enable paging.
Simply done:
<source lang="c">
asm volatile ("movl %%cr0, %%eax; orl $0x80000000, %%eax; movl %%eax, %%cr0;" ::: "eax");
</source>

PAE paging should now be enabled.

==FAQ==
Q: Why is it causing triple fault/page fault?

A: You're kernel is probably not inside the first 2 MiB of the memory. You'll need to identity map your kernel.

Q: How do I map pages?

A: Firstly, align your desired virtual memory address to 1 GiB, and divide it by 1 GiB to obtain the PDPT index. Then, find the index of the page directory by aligning your desired virtual address to 2 MiB and divide it by 2 MiB. Finally, lookup the page directory entry by the index obtained in the previous step, and set the entry's address to your desired physical address that's aligned to 2 MiB.

[[Category:Paging]]
[[Category:Virtual Memory]]
[[Category:X86 CPU]]
[[Category:Tutorials]]
````

<a id="w002436"></a>

## W002436 — JWasm

Bron-ID: W002436
Namespace: Main (0)
Revisie: 21651 | Datum: 2017-10-25T22:25:31Z
Bron: https://wiki.osdev.org/JWasm
Revisie-URL: https://wiki.osdev.org/index.php?title=JWasm&oldid=21651

````text
'''JWasm Macro Assembler''' is an x86 assembler that targets 16, 32 and 64 bit platforms. JWasm is designed as a MASM-compatible assembler and is available under the Sybase Open Watcom Public License. It produces binaries for  the DOS, Windows, Linux, OS/2 and FreeBSD operating systems. JWasm is an almost complete rewrite of the earlier Watcom assembler Wasm. JWasm is written in portable C and has been successfully tested with the Open Watcom development environment, the Microsoft Visual Studio family of development tools, the GNU (GCC) compiler and others. It is currently being upgraded by Japheth.

==History==
JWasm is an upgrade of the earlier Open Watcom assembler Wasm. JWasm has been extensively rewritten to modernize, extend capacity and add additional platform support to it. Among its design targets is a very high level of MASM compatibility. Its initial release is dated 05/20/2008 as v1.7. The current version as of 1/19/2010 is v2.02, adding 64 bit capabilities. It is actively being updated to support the latest operating systems.

==Usage==
JWasm conforms to the Microsoft Macro Assembler notation and uses the standard [[MASM]] documentation and later as a technical reference.

===Abbreviated Notation===
This notation is a fully specified format which occurs in the following form:

<source lang="asm">
    mov eax, DWORD PTR [edi]
</source>

Over time, the parsers in assemblers have improved to the stage where if the assembler can recognize the size of the data then the SIZE specifier may be omitted as such.

<source lang="asm">
    mov eax, [edi]
</source>

This allows for clearer code that is easier to read. However, there are some contexts where the assembler cannot independently determine the data size; For example, if the source operand is a memory operand.  In this situation the historical data SIZE specifiers must be used. The following is an example of this situation.

<source lang="asm">
    movzx eax, [esi]            ; generates an error - data SIZE cannot be determined by the assembler
    movzx eax, BYTE PTR [esi]   ; zero extend a BYTE into the 32 bit EAX register
</source>

===OFFSET Operator===
JWasm's syntax makes a distinction between fixed and transient addressing using the '''OFFSET''' operator. Data written in either the initialised or uninitialised data sections is a known ADDRESS at assembly time, as are code labels, all of which are referenced by the '''OFFSET''' operator. Transient addressing is performed with the normal Intel mnemonics for reading the stack within a procedure.

For a corresponding data entry in the initialised data section,

<source lang="asm">
    textitem db "This is a text item",0
</source>

This data entry can be addressed in the following manner.

<source lang="asm">
    mov eax, OFFSET textitem
</source>

===Transient Stack Addressing===
Operating systems provide memory for the area of memory referred to as the stack.  Under x86 hardware, the stack is the main method of transferring arguments to procedures. Arguments are normally placed on the stack by the PUSH mnemonic in the following form. This example assumes the '''STDCALL''' calling convention and 32 bit data size.

<source lang="asm">
    push arg3
    push arg2
    push arg1
    call FunctionName
</source>

The CALL mnemonic pushed the return address onto the stack then branches to the address of the named procedure. If the procedure has a stack frame where the stack pointer register '''ESP''' is stored in the base pointer register '''EBP''' the first argument for the procedure occurs at address [ebp+8]. While this form of mnemonic notation can be written by experienced assembler programmers, the assembler provides a naming method to remove an un-necessary level of abstraction from writing code of this type.

The programmer can use the '''name''' of the argument in the place of the direct [EBP+displacement] notation to make the code more readable with no loss of performance. When the programmer needs to use the ADDRESS of a transient stack variable (normally referred to as a '''LOCAL''' variable) they have a number of methods. In a prototyped function call they can use the '''ADDR''' operator to obtain the address of a '''LOCAL''' variable. Alternatively they can use the direct Intel mnemonic '''LEA''' to load the effective address of the variable into a register:

<source lang="asm">
    lea eax, named_local_variable
</source>

===Square Brackets===
JWasm, like MASM, uses named variables to represent both fixed and transient addresses. Square brackets are used around expressions to denote that the contents are a memory operand. Programmers coming from a different background where square brackets are used as general ADDRESS indicators have at time had problems with this notation difference but JWasm tolerates the use of square brackets around named variables by simply ignoring them.

There is some flexibility in how square brackets can be used in historical Intel notation compatible assemblers.

<source lang="asm">
    mov eax, [ecx+edx]
    mov eax, [ecx][edx]
</source>

Both notations are correct here and in the second example the extra pair of square brackets function as an ADDITION operator.

===Limited Type Checking===
JWasm supports a pseudo high level notation for creating procedures that perform argument size and count checking. It is part of a system using the '''PROC ENDP PROTO''' and '''INVOKE''' operators. The '''PROTO''' operator is used to define a function prototype that has a matching '''PROC''' that is terminated with the '''ENDP''' operator. The prototyped procedure can then be called with the '''INVOKE''' operator which is protected by the limited size and argument count checking. There is additional notation at a more advanced level for turning off the automatically generated stack frame for the procedure where stack overhead in the procedure call may have an effect with very small procedures. JWasm is also capable of being written completely free of the pseudo high level notation using only bare Intel mnemonics.

Using an example prototype from the 32 bit Windows API function set,

<source lang="asm">
    SendMessage PROTO STDCALL :DWORD,:DWORD,:DWORD,:DWORD
    SendMessage equ <SendMessageA>
</source>

The code to call this function using the '''INVOKE''' notation is as follows.

<source lang="asm">
    invoke SendMessage,hWin,WM_COMMAND,wParam,lParam
</source>

Which is translated exactly to,

<source lang="asm">
    push lParam
    push wParam
    push WM_COMMAND
    push hWin
    call SendMessage
</source>

The advantage of the '''INVOKE''' method is that it tests the size of the data types and the argument count and generates an assembly time error if the arguments do not match the prototype.

===Pseudo High Level Emulation===
JWasm conforms to the historical MASM notation in terms of emulating high level control and loop structures.<br />
It supports the '''.IF''' block structure,

<source lang="asm">
  .if
    ; ...
  .elseif
    ; ...
  .else
    ; ...
  .endif
</source>

It also supports the '''.WHILE''' loop structure,

<source lang="asm">
  .while eax > 0
    sub eax, 1
  .endw
</source>

And the '''.REPEAT''' loop structure.

<source lang="asm">
  .repeat
    sub eax, 1
  .until eax < 1
</source>

The high level emulation also supports C runtime comparison operators that work according to the same rules as Intel mnemonic comparisons. For the .IF block notation the distinction between SIGNED and UNSIGNED data is handles with a minor data type notation variation where the storage size '''DWORD''' which is by default UNSIGNED can also be specified as '''SDWORD''' for SIGNED comparison. This data type distinction is only appropriate for the pseudo high level notation as it is unused at the mnemonic level of code where the distinction is determined by the range of conditional evaluation techniques available in the Intel mnemonics.

The combined pseudo high level emulation allows JWasm to more easily interface with the later current operating systems that use a C style application programming interface. Generally the pseudo high level interface is used for non-speed critical code where clarity and readability are the most important factors, speed critical code is usually written in directly in mnemonics.

==Pre-processor==
The pre-processor in JWasm emulates the capacity in the Microsoft Macro Assembler and for most practical purposes it is near enough to identical. It is an old design dating back to about 1990 When Microsoft introduced the MASM 6.00 series of assemblers that is known to experienced users as quirky and complicated to use for advanced macro designs. Notwithstanding its archaic format it is a reasonably powerful pre-processor with loop techniques, conditional testing, text manipulation commands and the normal text substitution methods associated with arguments passed to the pre-processor.

At its simplest, a macro in JWasm is constructed as follows:

<source lang="asm">
    ItemName MACRO argument1, argument2, argument3:VARARG
      mov argument1, argument2
      mov argument3, argument1
    ENDM
</source>

This macro is called as follows,

<source lang="asm">
    ItemName eax, ecx, edx
</source>

It is expanded by the pre-processor to,

<source lang="asm">
    mov eax, ecx
    mov edx, eax
</source>

==Licence==
JWasm is licenced under the '''Sybase Open Watcom Public License''' and is available for use in environments and projects that are excluded by the Microsoft EULA for MASM. JWasm has no restrictions in writing Open Source software or writing software for non-Microsoft operating systems.
*[http://www.japheth.de/JWasm/License.html JWasm License]

== External Links, Reference And Footnotes ==
*[http://www.japheth.de/JWasm.html JWasm Home (broken link)]
*[https://github.com/JWasm/JWasm JWasm on Github]
*[http://sourceforge.net/projects/jwasm/ JWasm project page on SourceForge]
*[http://www.masm32.com/board/index.php The MASM Forum]
*[http://www.intel.com/products/processor/manuals/ Intel manuals]

[[Category:Assemblers]]
````

<a id="w002437"></a>

## W002437 — User:Tantrikwizard

Bron-ID: W002437
Namespace: User (2)
Revisie: 6727 | Datum: 2008-11-24T20:16:58Z
Bron: https://wiki.osdev.org/User:Tantrikwizard
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATantrikwizard&oldid=6727

````text
Hello everyone.

I'm [[http://www.davidmott.com David Mott]].  I began programming at 9 on a Commador 64 and have spent the better part of my life recieving trace radation from a CRT.  My hair has fallen out but my eye sight is 20/20.  With the advent of LCDs my gums have stopped bleeding.

I've been a professional programmer for nearly 20 years, mostly contract work.  I've been employed by a fantastic email and messaging company since 2006 and hope to retire from it.  I'm an electronics and robotics hobbiest and work on my OS in my spare time.  I'm now about a quarter the way through my new kernel.  The last kernel I wrote was nearly fully functional, it just ended up being a bunch of unmanagable assembler spagetti code so it got scraped.  The latest kernel is mostly in C/C++.  Open Watcom is the compiler of choice.  JWASM adds a nice familiar macro syntax to the mix as well.

I'll try to contribute to the WIKI as I have time.  Most everything is well laid out and already covered but there's a few places I plan to contribute, perhaps a tutorial or two somewhere.
````

<a id="w002439"></a>

## W002439 — Simple Scalable Fonts

Bron-ID: W002439
Namespace: Main (0)
Revisie: 23980 | Datum: 2019-09-25T11:40:24Z
Bron: https://wiki.osdev.org/Simple_Scalable_Fonts
Revisie-URL: https://wiki.osdev.org/index.php?title=Simple_Scalable_Fonts&oldid=23980

````text
So you got a multi-tasking kernel up and running, have created a simple [[GUI]] and want to add some sexy scalable fonts.  This short tutorial is intended to give you some ideas on how you might want to implement scalable fonts, [[Drawing In Protected Mode]] has an example of printing fixed width fonts.  This technique uses instruction based drawing to achieve scalable fonts.

The first thing you'll need is some utility functions:
*PutPixel to plot a pixel
*PutLine to draw a straight line
*PutRect to draw a square
*PutCurve to draw a curve

Assume we want to print a large "A scaled font demo" at 100x100 on the screen using a 25 point font in black:

 PrintScaledString(100,100,25,25,"A scaled font demo", rbgBlack);

The PrintScaledString function looks at each character and calls a separate method to render each character.  For simplicity's sake I'll only focus on the capital letter 'A'.

<source lang="c">
void PrintScaledString(int x, int y, int charWidth, int charHeight, char * str, int RGB)
 {
    int charX=x, charY=y;
    while (*str)
    {
       switch (*str)
       {
          case '\n':
              charX=x;
              charY=charY+charHeight;
              break;
          case 'A':
              DrawA(charX, charY, charWidth, charHeight, RGB);
              break;
          case 'B':
              DrawB(charX, charY, charWidth, charHeight, RGB);
          ... call more draw methods here for each character
       }
       if (*str != '\n') charX+=charWidth;
       str++;
    }
 }
</source>

You have several options for drawing scalable fonts, but the better looking fonts are more difficult to code.  It's not very difficult to code very ugly fonts but if you want quality you'll need some skills.  The letter 'A' is very simple:

<source lang="c">
void DrawA(int x, int y, int charWidth, int charHeight, int RGB
 {
    //first draw a line along the left side:
    PutLine(x, y, x, y+charHeight,RGB);
    //draw a line across the top:
    PutLine(x,y,x+charWidth,y,RGB);
    //draw a line along the right side:
    PutLine(x+charWidth,y,x+charWidth,y+charHeight,RGB);
    //finally close in the box half way up
    PutLine(x,y+(charHeight/2),x+charWidth,y+(charHeight/2),RGB);
 }
</source>

The capital 'A' is very simple to render, more complex characters will require more code and calculations.  You will probably want to research [http://www.gamedev.net/reference/articles/article735.asp collision detection], [http://www.gamedev.net/reference/programming/features/curvessurfaces/ Bezier curves] plus [http://www.gamedev.net/reference/articles/article767.asp Bresenham's line and circle algorithms]

Of course hardwiring typeface into a renderer is a very bad idea. You should store the glyphs in data array, and use the combination of the appropriate PutLine(), PutBezier() etc. calls. There are OpenSource libraries to exactly do that, so that you don't have to reinvent the wheel. See [[Scalable Screen Font]] which comes with a single ANSI C header implementation and has almost no dependencies.

Alternatively, if you have a functioning [[C Library]] for your OS then it may be desirable to port [http://freetype.sourceforge.net/index2.html FreeType 2]. FreeType 2 supports many different font formats and has it's own fast, high quality font rendering engine that directly draws into bitmaps in memory. FreeType 2 is used as the basis for QT, Atheos, Cairo, and many other GUI projects.

[[Category:Graphical UI]]
````

<a id="w002440"></a>

## W002440 — Simple True Type Fonts

Bron-ID: W002440
Namespace: Main (0)
Revisie: 6760 | Datum: 2008-11-27T00:16:08Z
Bron: https://wiki.osdev.org/Simple_True_Type_Fonts
Revisie-URL: https://wiki.osdev.org/index.php?title=Simple_True_Type_Fonts&oldid=6760
Doorverwijzing: Simple Scalable Fonts

````text
#REDIRECT [[Simple Scalable Fonts]]
````

<a id="w002441"></a>

## W002441 — Talk:Simple Scalable Fonts

Bron-ID: W002441
Namespace: Talk (1)
Revisie: 6762 | Datum: 2008-11-27T00:25:10Z
Bron: https://wiki.osdev.org/Talk:Simple_Scalable_Fonts
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ASimple_Scalable_Fonts&oldid=6762

````text
Suggestion for the next edit that adds a lot more code: set up a way to use pre-set sizes if you want to. But that's a little beside the point of scalability! --[[User:Troy martin|Troy Martin]] 00:25, 27 November 2008 (UTC)
````

<a id="w002442"></a>

## W002442 — File:Paging Structure.gif

Bron-ID: W002442
Namespace: File (6)
Revisie: 6765 | Datum: 2008-11-27T02:26:35Z
Bron: https://wiki.osdev.org/File:Paging_Structure.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3APaging_Structure.gif&oldid=6765

````text
Demonstrates layout and addressing for paging
````

<a id="w002443"></a>

## W002443 — Turbo C

Bron-ID: W002443
Namespace: Main (0)
Revisie: 29111 | Datum: 2024-06-07T13:41:29Z
Bron: https://wiki.osdev.org/Turbo_C
Revisie-URL: https://wiki.osdev.org/index.php?title=Turbo_C&oldid=29111

````text
{{stub}}

{{Infobox Tool
|name=Turbo C
|image=Turbo_C_Preview.png
|operating_system=DOS
}}

Turbo C (aka TC) is [http://www.borland.com/ Borland's] 16-bit C compiler. In OS development, it is only useful for developing real mode operating systems. TC normally compiles to a DOS .exe executable, but the generated code can be turned into a flat binary by [[exe2bin]] if you compile with the -mt (model tiny) argument to the compiler, which makes the result position-independent.

== Make bug ==
If you're using Turbo C. There is a chance that the make program that comes in TC hangs indefinitely. To fix this it's necessary to create a makefile that does absolutely nothing. This only happens on Turbo C 2.01.

== Inline Assembly ==

Inline assembly works fine in Turbo C, but there are a few rules:
* You must compile from the command line
* You must own [[TASM]] or pass -Exxx to the compiler
* Only intel syntax is accepted

-Exxx specifies an alternate assembler to use, eg. -Enasm.exe if you wanted to assemble with [[NASM]].

[[Category:C]]
[[Category:Compilers]]
[[Category:IDEs]]
````

<a id="w002444"></a>

## W002444 — File:PS2 connector.png

Bron-ID: W002444
Namespace: File (6)
Revisie: 6791 | Datum: 2008-11-29T00:32:37Z
Bron: https://wiki.osdev.org/File:PS2_connector.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3APS2_connector.png&oldid=6791

````text
A simple labelled drawing of a female PS/2 connector.

This image is from Wikipedia, modified to include labels and saved as a .png. The original, released into the public domain, can be found as an SVG here: http://en.wikipedia.org/wiki/Image:MiniDIN-6_Connector_Pinout.svg
````

<a id="w002445"></a>

## W002445 — X86

Bron-ID: W002445
Namespace: Main (0)
Revisie: 6776 | Datum: 2008-11-28T04:45:41Z
Bron: https://wiki.osdev.org/X86
Revisie-URL: https://wiki.osdev.org/index.php?title=X86&oldid=6776
Doorverwijzing: Category:X86

````text
#redirect [[Category:X86]]
````

<a id="w002446"></a>

## W002446 — File talk:PS2 connector.png

Bron-ID: W002446
Namespace: File talk (7)
Revisie: 6784 | Datum: 2008-11-28T22:49:28Z
Bron: https://wiki.osdev.org/File_talk:PS2_connector.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File_talk%3APS2_connector.png&oldid=6784

````text
What is the original source and license of this file? [[User:Jackscott|Jackscott]] 07:34, 28 November 2008 (UTC)
:Done. Added to file page. [[User:Troy martin|Troy Martin]] 22:49, 28 November 2008 (UTC)
````

<a id="w002447"></a>

## W002447 — Real mode assembly bare bones

Bron-ID: W002447
Namespace: Main (0)
Revisie: 7074 | Datum: 2009-01-17T00:21:45Z
Bron: https://wiki.osdev.org/Real_mode_assembly_bare_bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Real_mode_assembly_bare_bones&oldid=7074
Doorverwijzing: Real mode assembly I

````text
#REDIRECT [[Real mode assembly I]]
````

<a id="w002448"></a>

## W002448 — User:Troy martin/Real mode assembly bare bones

Bron-ID: W002448
Namespace: User (2)
Revisie: 20686 | Datum: 2017-04-10T17:42:15Z
Bron: https://wiki.osdev.org/User:Troy_martin/Real_mode_assembly_bare_bones
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATroy_martin%2FReal_mode_assembly_bare_bones&oldid=20686
Doorverwijzing: Real mode assembly I

````text
#REDIRECT [[Real mode assembly I]]
````

<a id="w002449"></a>

## W002449 — Alta Lang

Bron-ID: W002449
Namespace: Main (0)
Revisie: 30200 | Datum: 2026-03-11T23:59:48Z
Bron: https://wiki.osdev.org/Alta_Lang
Revisie-URL: https://wiki.osdev.org/index.php?title=Alta_Lang&oldid=30200

````text
Why would anyone want to write something as complex as an operating system in something as clunky and old as C? Alta wants something different for her system, a new language for a more elegant operating system. She spends at least as much time on language design as on system design, in the hopes that if she can design the language to be clean enough and powerful enough, the system will simply fall into place. Or maybe you want to start with [[What order should I make things in|something else]].

== Elegant Notation for Elegant Design ==
There is no question that C is [[Languages| not the only choice]] for operating system programming. Many kernels have been written in assembly language, and languages as diverse as PL/1, [[Ada]], Common Lisp, Forth, and Smalltalk have all been used successfully for OS projects. While it takes a great deal of work, implementing in a different language can pay dividends in the long run when you are designing the underlying abstractions of your system - or so the idea goes, at any rate.

To design a new language for the purpose of operating systems design is in some ways the logical extension of this, especially if you intend to tightly couple the language and the environment. Both Smalltalk-80 and most Forth systems were designed in this manner, and while they are not major players today, they were significant in their time, and still influence the way people program now. If you intend a really radical design, it is only natural to think that having an equally radical language to implement it in makes sense.

== Alta's Bookshelf ==
In addition to books on OS theory such as ''[http://www.amazon.com/Modern-Operating-Systems-2nd-GOAL/dp/0130313580 Modern Operating Systems]'' by Tanenbaum, much of Alta's shelf space is dedicated to books on compiler theory and language design, such as the Dragon book (''[http://en.wikipedia.org/wiki/Compilers:_Principles,_Techniques,_and_Tools Compilers: Principles, Techniques, and Tools]'' by Aho, Lam, Sethi and Ullman) and ''[http://www.amazon.com/Advanced-Programming-Language-Design-Raphael/dp/0805311912 Advanced Programming Language Design]'' by Finkel. There will also be books on the intricacies of her preferred programming paradigm(s), such as ''Type Theory and Functional Programming'', ''Object-Oriented Programming: A Unified Foundation'', or ''The Art of the Meta-Object Protocol''.

== Alta's Critics ==
Designing and implementing a compiler suitable for systems work is nearly as difficult as designing an operating system. Designing a new language is, if anything, even more difficult. By taking on all three tasks at once, you are creating an insurmountable hurdle. Furthermore, you'll be cutting off most of the help others could give you, both in design and debugging, leaving you working without a net - where most OS developers spend their time hunting bugs in their OS, you'll be trying to determine if the bug is in the OS, in the language translator, or a result of some flaw in the language itself.

== People and OSes related to Alta Lang ==
* Kether by [[User:Schol-r-lea|Schol-R-LEA]] (to be written in [https://github.com/Schol-R-LEA/Assiah Assiah] and [http://www.thelema-language.org Thelema])
* [[User:Walling|Bjarke Walling]]
* PHPOS by [[User:Thepowersgang|thePowersGang]]
* Lyric by [[User:Ferrarius|Ferrarius]]
* [https://github.com/the-little-language-designer/cicada cicada] by [[User:xyh|xyh]]
* [[User:No92|no92]]
* [[User:Kjam|kjam]]
* Lisp is just too tempting! --[[User:Alyssa|Alyssa]]
* BCOS by [[User:Brendan|Brendan Trotter]], though it also overlaps with [[Eleanore Semaphore]]
* [[User:Joshw|Josh Wyant]]
* [[User:eekee|eekee]] intended to implement Forth for [[User:Eekee#Kaph|Kaph]], but has recently become interested in [https://osmosianplainenglishprogramming.blog/ Plain English Programming.]
* [[User:PeterX|PeterX]] only half (one half liking Scheme and perhaps Python, the other half using Assembler and C)
* [[User:andreykapustin|Andrey Kapustin]]
* Oberon (which is both an operating system and a programming language)
* [[User:Brain|Brain]] - Retro Rocket and Retro Rocket BASIC, 17 years on and counting, and im slowly creating the entire userland in my language...
* '' insert your names here ''

[[Category:Developer Archetype]]
[[Category:Languages]]
````

<a id="w002450"></a>

## W002450 — User:Combuster/Mach64 Hardware

Bron-ID: W002450
Namespace: User (2)
Revisie: 7214 | Datum: 2009-02-12T23:22:19Z
Bron: https://wiki.osdev.org/User:Combuster/Mach64_Hardware
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACombuster%2FMach64_Hardware&oldid=7214

````text
Below you'll find the notes on my reverse engineering tours on the mach64 hardware. Don't expect it to be complete.

The Mach64 is an old, yet widely spread graphics chip that supports high resolution modes, 2D hardware acceleration, and on later versions, accelerated 3D operations. It can be found in many older computers (Pentium 1 till Pentium 3 era) as popular onboard graphics chips, although there are plugin cards and laptop versions available. 

== Introduction to this document ==
Since documentation is scarce and implementations rare, I write most of this based on personal experience. Expect that the information will still be incomplete, but at least slightly more helpful than the code found on the web.

I have four variants, I generally test on three of them since the fourth is bios-disabled for a different graphics card.

I have:
Rage Pro Turbo
Rage IIC
Rage XL
264VT (the disabled one)

All Mach64s labeled 'Rage' are generations of the 264GT
I do not have some older chips that do belong to this family:
88800 chips
264CT chips

Most notably, I only have PCI based devices and therefore can't tell much about the older ISA ones. However, the only other doc around (i.e. VGADOC) is mainly concerned about the ISA variants, although still necessary for all the standard register locations.

== Guided Tour ==
Hop on the bus and get to know the Mach64 family.

=== Detection ===
There are several methods of detecting a Mach64. The most straightforward one is to have a look at the [[PCI]] devices, and see if there's a good chip present. Should you find one, then you should really check out its BARs - it normally has three of them. The first is the offset of the linear framebuffer, the second is the offset of the port I/O range, while the third is a memory-mapped range. Although normally present, people believe that it has sometimes been missing in action.

'''todo: list PCI ids and corresponding information'''

The second method (especially interesting for ISA devices and other ATI devices) works by poking around video BIOS. ATI always put a header on top of their video bios that, among others, tell what kind of device it is. Even now you can still use it to detect a newer Radeon without doing potentially dangerous probes.

What we know from here is where we can find the framebuffer. If you want to play around, you can try writing here as it works in VGA modes as well, minus the nuisance called planes as everything's available as 256k of memory and then some. (the planes are interleaved. every doubleword matches one address in planar memory: byte 0 plane 0, byte 0 plane 1, byte 0 plane 2, byte 0 plane 3, byte 1 plane 0, byte 1 plane 1, ...)

=== Enabling the chip ===
Lets have a chat with the device. If you paid attention you should have gotten the second BAR from the PCI space. If you paid even more attention then you'd know that ISA devices are accessed differently.

At least for PCI you add the register number times four (= register offset) to the base address you have and you can do 32-bit reads and writes off there.

What's the case is that the Mach64 is pretending that it is a [[VGA Hardware|VGA]] card. This is nice for every beginner which just wants to see it work, but not when we want to get more out of it.

Let us start by disabling the VGA frontend and have it show its true nature.

<pre>
uint32_t oldvalue;
oldvalue = inportd(port_base + CRTC_GEN_CNTL);
oldvalue |= CRTC_EXT_DISP_EN;
outportd(port_base + CRTC_GEN_CNTL, oldvalue);
</pre>

Notice how the screen blinks and presents you with some nice colors.

What you can do here is write to the framebuffer and draw some nice graphical images across the screen.

=== All shapes and colours ===

You may have figured that you miss some information about the screen to write to it properly. Of course, we haven't set the resolution yet. 
Or actually, we might have. When the Mach64 emulates the VGA, the registers are mirrored. So when we put the 'VGA' Mach64 into text mode, we set the resolution to an equivalent of 720x400. So when we enabled the Mach64, it will still display 720x400, still 256 colors since that's what the VGA can do, but only graphics mode since that too, is a thing of the VGA.

'''Todo: tell about timing'''

When we know what resolution we want to set, we can tell the Mach64 about it. It however doesn't like to be told how to do things when it is busy doing differently, so it must be turned off to make sure its calm and willing.

So the steps are:
# Disable CRTC output
# Write the CRTC registers
# Enable the CRTC
# Enjoy life.


=== The wristwatch and the work schedule ===

Hey look! The mach64 is intelligent, and it will buffer large bits of memory into a local buffer before displaying it. Since it can do burst transactions that way it occupies the bus for less time than it might have done otherwise.

The elderly have automated this task and will automatically fetch a new set when they are running short of data. The younger rage chips however have this nice DSP which tell when to load image data from memory. If it forgets to load data, it will stutter and repeat whatever it still remembers, which is generally not quite what you want.

'''todo: this is where the horror starts'''

== Code snippets ==

=== Rage IIC register image ===
The undocumented stuff returns. You need to set a clock (undocumented) besides switching to the accelerator CRTC (documented) or you'll be left with a garbled display.

<pre>
    crtcgenctl = &HB010200

    ' kill CRTC
    outportd(portbase + mach64_regs.CRTC_GEN_CNTL, crtcgenctl and (not Mach64_crtc_gen_cntl.CRTC_EN) )

    ' write registers
    outportd(portbase + &H14, &H14000000)
    outportd(portbase + &H20, &H460756)
    outportd(portbase + &H24, &H900714)
    outportd(portbase + &H90, &Had0003)

    ' enable CRTC
    outportd(portbase + mach64_regs.CRTC_GEN_CNTL, crtcgenctl)
</pre>
this isn't much of a documentation but I haven't got much more than this for now.


== Various mach64 chips and their features ==

'''todo: determine if fifo size is bits or bytes, figure the difference in 3d engines'''

{| {{Wikitable}}
! Chip Name
! Identifier
! pixel clock
! mem clock
! chip clock
! std. pll/dac
! DSP
! 3D
! other (see linux sources)
|-
| ATI888GX00 (Mach64 GX) || ? || 135 MHz || 50 MHz 
|| 50 MHz || {{No}} || {{No}} || {{No}} || ATI_CHIP_88800GX
|-
| ATI888CX00 (Mach64 CX) || ? || 135 MHz || 50 MHz 
|| 50 MHz || {{No}} || {{No}} || {{No}} || ATI_CHIP_88800CX
|-
| ATI264CT (Mach64 CT) || ? || 135 MHz || 60 MHz 
|| 60 MHz || {{Yes}} || {{No}} || {{No}} || ATI_CHIP_264CT
|-
| ATI264ET (Mach64 ET) || ? || 135 MHz || 60 MHz 
|| 60 MHz || {{Yes}} || {{No}} || {{No}} || ATI_CHIP_264ET
|-
| ATI264LT (Mach64 LT) || ? || 135 MHz || 63 MHz 
|| 63 MHz || {{Yes}} || {{Yes|24 bit}} || {{No}} || ATI_CHIP_264LT

|-
| ATI264VT (A3) (Mach64 VT) || ? Rev 0.0 || 170 MHz || 67 MHz 
|| 67 MHz || {{Yes}} || {{No}} || {{No}} || ATI_CHIP_264VT
|-
| ATI264VT2 (A4) (Mach64 VT) || ? Rev 0.1 || 200 MHz || 67 MHz 
|| 67 MHz || {{Yes}} || {{No}} || {{No}} || ATI_CHIP_264VT + M64F_MAGIC_POSTDIV
|-
| ATI264VT (Mach64 VT)|| ? Rev 0.x || 170 MHz || 67 MHz 
|| 67 MHz || {{Yes}} || {{No}} || {{No}} || ATI_CHIP_264VT
|-
| ATI264VT3 (B1) (Mach64 VT) || ? Rev 1.x || 200 MHz || 67 MHz 
|| 67 MHz || {{Yes}} || {{Yes|24 bit}} || {{No}} || ATI_CHIP_264VT3
|-
| ATI264VT3 (B2) (Mach64 VT) || ? Rev 2.x || 200 MHz || 67 MHz 
|| 67 MHz || {{Yes}} || {{Yes|24 bit}} || {{No}} || ATI_CHIP_264VTB

|-
| 3D RAGE (Mach64 GT)|| ? Rev x.0|| 135 MHz || 63 MHz 
|| 63 MHz || {{Yes}} || {{No}} || {{No}} || ATI_CHIP_264GT
|-
| 3D RAGE II (Mach64 GT) || ? Rev x.1|| 170 MHz || 63 MHz 
|| 63 MHz || {{Yes}} || {{Yes|24 bit}} || {{No}} || ATI_CHIP_264GTB
|-
| 3D RAGE II+ (Mach64 GT) || ? Rev x.2|| 200 MHz || 63 MHz 
|| 63 MHz || {{Yes}} || {{Yes|24 bit}} || {{No}} || ATI_CHIP_264GTB

|-
| ATI264VT3 (Mach64 VU)|| ? || 200 MHz || 67 MHz 
|| 67 MHz || {{Yes}} || {{Yes|24 bit}} || {{No}} || ATI_CHIP_264VT3
|-
| 3D RAGE II+ (Mach64 GU)|| ? || 200 MHz || 67 MHz 
|| 67 MHz || {{Yes}} || {{Yes|24 bit}} || {{No}} || ATI_CHIP_264GTB
|-
| 3D RAGE LT (Mach64 LG)|| ? || 230 MHz || 63 MHz 
|| 63 MHz || {{Yes}} || {{Yes|24 bit}} || {{No}} || ATI_CHIP_264LTG + M64F_LT_LCD_REGS + M64F_G3_PB_1024x768
|-
| ATI264VT4 (Mach64 VV)|| ? || 230 MHz || 83 MHz 
|| 83 MHz || {{Yes}} || {{Yes|24 bit}} || {{No}} || ATI_CHIP_264VT4
|-
| 3D RAGE IIC (Mach64 GV, PCI)|| ? || 230 MHz || 83 MHz 
|| 83 MHz || {{Yes}} || {{Yes|24 bit}} || {{No}} || ATI_CHIP_264GT2C
|-
| 3D RAGE IIC (Mach64 GW, AGP)|| ? || 230 MHz || 83 MHz 
|| 83 MHz || {{Yes}} || {{Yes|24 bit}} || {{No}} || ATI_CHIP_264GT2C
|-
| 3D RAGE IIC (Mach64 GY, PCI)|| ? || 230 MHz || 83 MHz 
|| 83 MHz || {{Yes}} || {{Yes|24 bit}} || {{No}} || ATI_CHIP_264GT2C 
|-
| 3D RAGE IIC (Mach64 GZ, AGP)|| ? || 230 MHz || 83 MHz 
|| 83 MHz || {{Yes}} || {{Yes|24 bit}} || {{No}} || ATI_CHIP_264GT2C
|-
| 3D RAGE PRO (Mach64 GB, BGA, AGP)|| ? || 230 MHz || 100 MHz 
|| 100 MHz || {{Yes}} || {{Yes|32 bit}} || {{Yes}} || ATI_CHIP_264GTPRO
|-
| 3D RAGE PRO (Mach64 GD, BGA, AGP 1x)|| ? 
|| 230 MHz || 100 MHz || 100 MHz || {{Yes}} || {{Yes|32 bit}} || {{Yes}} || ATI_CHIP_264GTPRO
|-
| 3D RAGE PRO (Mach64 GI, BGA, PCI)|| ? 
|| 230 MHz || 100 MHz || 100 MHz || {{Yes}} || {{Yes|32 bit}} || {{Yes}} || ATI_CHIP_264GTPRO + M64F_MAGIC_VRAM_SIZE
|-
| 3D RAGE PRO (Mach64 GP, PQFP, PCI)|| ? 
|| 230 MHz || 100 MHz || 100 MHz || {{Yes}} || {{Yes|32 bit}} || {{Yes}} || ATI_CHIP_264GTPRO
|-
| 3D RAGE PRO (Mach64 GQ, PQFP, PCI, limited 3D)|| ? 
|| 230 MHz || 100 MHz || 100 MHz || {{Yes}} || {{Yes|32 bit}} || {{Yes}} || ATI_CHIP_264GTPRO
|-
| 3D RAGE LT PRO (Mach64 LB, AGP)|| ? 
|| 236 MHz || 75 MHz || 100 MHz || {{Yes}} || {{Yes|32 bit}} || {{Yes}} || ATI_CHIP_264LTPRO
|-
| 3D RAGE LT PRO (Mach64 LD, AGP)|| ? 
|| 230 MHz || 100 MHz || 100 MHz || {{Yes}} || {{Yes|32 bit}} || {{Yes}} || ATI_CHIP_264LTPRO
|-
| 3D RAGE LT PRO (Mach64 LI, PCI)|| ? 
|| 230 MHz || 100 MHz || 100 MHz || {{Yes}} || {{Yes|32 bit}} || {{Yes}} || ATI_CHIP_264LTPRO + M64F_G3_PB_1_1 + M64F_G3_PB_1024x768
|-
| 3D RAGE LT PRO (Mach64 LP, PCI)|| ? 
|| 230 MHz || 100 MHz || 100 MHz || {{Yes}} || {{Yes|32 bit}} || {{Yes}} || ATI_CHIP_264LTPRO
|-
| 3D RAGE LT PRO (Mach64 LQ, PCI)|| ? 
|| 230 MHz || 100 MHz || 100 MHz || {{Yes}} || {{Yes|32 bit}} || {{Yes}} || ATI_CHIP_264LTPRO
|-
| 3D RAGE XL (Mach64 GM, AGP 2x)|| ? 
|| 230 MHz || 83 MHz || 63 MHz || {{Yes}} || {{Yes|32 bit}} || {{Yes}} || ATI_CHIP_264XL
|-
| 3D RAGE XC (Mach64 GN, AGP 2x)|| ? 
|| 230 MHz || 83 MHz || 63 MHz || {{Yes}} || {{Yes|32 bit}} || {{Yes}} || ATI_CHIP_264XL
|-
| 3D RAGE XL (Mach64 GO, PCI-66)|| ? 
|| 230 MHz || 83 MHz || 63 MHz || {{Yes}} || {{Yes|32 bit}} || {{Yes}} || ATI_CHIP_264XL
|-
| 3D RAGE XC (Mach64 GL, PCI-66)|| ? 
|| 230 MHz || 83 MHz || 63 MHz || {{Yes}} || {{Yes|32 bit}} || {{Yes}} || ATI_CHIP_264XL
|-
| 3D RAGE XL (Mach64 GR, PCI-33)|| ? 
|| 230 MHz || 83 MHz || 63 MHz || {{Yes}} || {{Yes|32 bit}} || {{Yes}} || ATI_CHIP_264XL + M64F_SDRAM_MAGIC_PLL
|-
| 3D RAGE XC (Mach64 GS, PCI-33)|| ? 
|| 230 MHz || 83 MHz || 63 MHz || {{Yes}} || {{Yes|32 bit}} || {{Yes}} || ATI_CHIP_264XL
|-
| 3D RAGE Mobility P/M (Mach64 LM, AGP 2x)|| ? 
|| 230 MHz || 83 MHz || 125 MHz || {{Yes}} || {{Yes|32 bit}} || {{Yes}} || ATI_CHIP_MOBILITY
|-
| 3D RAGE Mobility L (Mach64 LN, AGP 2x)|| ? 
|| 230 MHz || 83 MHz || 125 MHz || {{Yes}} || {{Yes|32 bit}} || {{Yes}} || ATI_CHIP_MOBILITY
|-
| 3D RAGE Mobility P/M (Mach64 LR, PCI)|| ? 
|| 230 MHz || 83 MHz || 125 MHz || {{Yes}} || {{Yes|32 bit}} || {{Yes}} || ATI_CHIP_MOBILITY
|-
| 3D RAGE Mobility L (Mach64 LS, PCI)|| ? 
|| 230 MHz || 83 MHz || 125 MHz || {{Yes}} || {{Yes|32 bit}} || {{Yes}} || ATI_CHIP_MOBILITY
|}
````

<a id="w002451"></a>

## W002451 — Real mode assembly II

Bron-ID: W002451
Namespace: Main (0)
Revisie: 27867 | Datum: 2023-07-09T04:00:30Z
Bron: https://wiki.osdev.org/Real_mode_assembly_II
Revisie-URL: https://wiki.osdev.org/index.php?title=Real_mode_assembly_II&oldid=27867

````text
{{FirstPerson}}
{{Rating|1}}
{{Warning|Please see the [[Real Mode OS Warning]] page before continuing.}}

''In this second installment of the [[real mode assembly bare bones]] series we will cover hex functions and NASM directives and their usefulness.''

==Hexadecimal to ASCII==

This is where things get fun. We're going to convert a value in al to two characters and print them out. The following code snippet from http://www.df.lth.se/~john_e/gems/gem003a.html shows how to convert the low nybble of al to an ASCII character.

<source lang="asm">
 cmp al,10
 sbb al,69h
 das
</source>

Simple, but doesn't do that much. After all, it only converts the low nybble and can do weird things occasionally. So we need to shift (and rotate for the low nybble). Here's how I do it:

<source lang="asm">
 shr al,4        ; Shift right four places to put upper nybble in lower spot
 cmp al,10
 sbb al,69h
 das
 
 mov ah,0Eh   ; Print out upper nybble
 int 10h
</source>

Sure, it works, but it's only printing out the upper nybble and the low nybble disappears! Oh noes! So we have to store a copy of al in a variable first:

<source lang="asm">
 mov [.temp],al
</source>

And restore it after to do the second:

<source lang="asm">
 mov al,[.temp]
</source>

Now we rotate and shift to do the conversion for the low nybble:

<source lang="asm">
 ror al,4
 shr al,4
 cmp al,10
 sbb al,69h
 das
 
 mov ah,0Eh
 int 10h
</source>

===The code===

Now put that all together:

<source lang="asm">
 print_hex_byte: 
   mov [.temp],al
   shr al,4
   cmp al,10
   sbb al,69h
   das
 
   mov ah,0Eh
   int 10h
 
   mov al,[.temp]
   ror al,4
   shr al,4
   cmp al,10
   sbb al,69h
   das
 
   mov ah,0Eh
   int 10h
 
   ret
 
 .temp db 0
</source>

And that function is now callable! Using what you've learned, you can also write '''your own''' call to print all of ax and not just al. Try it, it's not that hard!

==NASM: Lord of Open-Source Assemblers==

[[NASM]] has some great macro abilities, including automating tasks such as writing your db statements for you. Simple, multi-line macros go something like this:

<source lang="asm">
 %macro name operands
    ; code code code
 %endmacro
</source>

You can use %1, %2, etc. to represent the contents of arguments passed to your macro. You can even emulate BASIC statements with macros, like discussed [[printing to the screen without a db|here]].

Even simpler macros are the single-line macros, or %define statements. They create assemble-time statements to be inserted, but they are more like constants than functions.

<source lang="asm">
 %define D_SIGNATURE 0xCAFEF00D
 %define D_INFO 0x29A3B83D
 
 push D_SIGNATURE
 push D_INFO
 jmp 0x2000:0
</source>

Now, that's just example code, it could mean anything, but it depends on what's at 0x2000:0, of course. But the principle is the same, %define is used to create constants and the like.

NASM's macro features are discussed in a lot more detail in the manual: http://www.nasm.us/doc/nasmdoc4.html

==Continuing on==

This tutorial was more of a theory lesson and less of a water pipe spitting out gushes of source code. Future tutorials will have more application, but less copy-and-paste code.

Have fun developing your OS!

[[Category:Assembly]]
[[Category:Real mode assembly]]
 <- [[Real mode assembly I]]  |  [[Real mode assembly III]] ->
````

<a id="w002452"></a>

## W002452 — Talk:Stack

Bron-ID: W002452
Namespace: Talk (1)
Revisie: 14546 | Datum: 2013-03-28T12:12:07Z
Bron: https://wiki.osdev.org/Talk:Stack
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AStack&oldid=14546

````text
The page says using ebp for a stack frame is part of the cdecl calling convention. That's not true. Ebp is not "special" in cdecl. It should be treated as any other non-volatile register (ebx, esi, edi, etc...).

== EBP pointer is incorrect ==

I'm pretty new to assembly so bare with me: I think all of the references to EBP are incorrect on this page. On x86, the push instruction steps the value of ESP and ''then'' stores the value, so that ESP always points to the last value pushed onto the stack, not the next free position.

Therefore in the prologue of a routine, when you <code>PUSH EBP</code> and then <code>MOV EBP, ESP</code>, the resulting value of EBP should point to the saved EBP value (the last thing that was pushed), where as all the diagrams on this page show it pointing to the next position after the saved EBP. This also means that all the offsets used to access local variables and especially parameters are incorrect.

I'm happy to update the page, but since I want to make sure I'm not missing something before I do. Can anyone comment on this before I edit?

:I agree that diagram needs to be updated. I also find it a bit ambiguous for it to have both a "First caller" and a "Routine 1" so perhaps you will use better notation for that, while you're at it. It would be particularly nice to have an actual figure, like some of the other articles do---but only if you have the time. --[[User:Love4boobies|Love4boobies]] 07:12, 28 March 2013 (CDT)
````

<a id="w002453"></a>

## W002453 — User:Troy martin/Code

Bron-ID: W002453
Namespace: User (2)
Revisie: 6850 | Datum: 2008-12-14T23:39:59Z
Bron: https://wiki.osdev.org/User:Troy_martin/Code
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATroy_martin%2FCode&oldid=6850
Doorverwijzing: User:Troy martin/Emulators

````text
#REDIRECT [[User:Troy martin/Emulators]]
````

<a id="w002454"></a>

## W002454 — User:Erikbomb

Bron-ID: W002454
Namespace: User (2)
Revisie: 22444 | Datum: 2018-06-09T16:05:53Z
Bron: https://wiki.osdev.org/User:Erikbomb
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AErikbomb&oldid=22444

````text
<!----
Create the table
----->
{| cellpadding="4" width="100%"
|valign="top" width="50%|
<!----
Left Side Of table
----->
<!----
About me section
----->
````

<a id="w002455"></a>

## W002455 — User talk:Erikbomb

Bron-ID: W002455
Namespace: User talk (3)
Revisie: 22445 | Datum: 2018-06-09T16:06:48Z
Bron: https://wiki.osdev.org/User_talk:Erikbomb
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AErikbomb&oldid=22445

````text
Have something to say about Erikbomb? Please write it here!

== Comments ==
````

<a id="w002469"></a>

## W002469 — File:Header grey.jpg

Bron-ID: W002469
Namespace: File (6)
Revisie: 6884 | Datum: 2008-12-15T04:56:58Z
Bron: https://wiki.osdev.org/File:Header_grey.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AHeader_grey.jpg&oldid=6884

````text

````

<a id="w002471"></a>

## W002471 — Triple-fault

Bron-ID: W002471
Namespace: Main (0)
Revisie: 6921 | Datum: 2008-12-19T10:45:48Z
Bron: https://wiki.osdev.org/Triple-fault
Revisie-URL: https://wiki.osdev.org/index.php?title=Triple-fault&oldid=6921
Doorverwijzing: Triple Fault

````text
#REDIRECT [[Triple Fault]]
````

<a id="w002472"></a>

## W002472 — Talk:Triple Fault

Bron-ID: W002472
Namespace: Talk (1)
Revisie: 9627 | Datum: 2010-01-15T22:36:06Z
Bron: https://wiki.osdev.org/Talk:Triple_Fault
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ATriple_Fault&oldid=9627

````text
== New page on the same subject ==
I pulled this out if a recently new-made triplefault page (which is now a redirect):

"
:When the exception handler of a double fault causes yet another exception (or your [[Interrupt Descriptor Table|IDT]] is messed up, or ...) the CPU resets - this is called a triple fault.
:Not so nice to debug on hardware - if I remember correctly [[Bochs]] dumps some debugging output in case of a triple fault.
:Oh, and it's the most sexy way to drop out of [[protected mode]].
"

Of its own this isn't quite useful for the reader, but if elaborated a bit further it might help improve this article. Any takers? - [[User:Combuster|Combuster]] 10:51, 19 December 2008 (UTC)
````

<a id="w002473"></a>

## W002473 — Root System Description Pointer

Bron-ID: W002473
Namespace: Main (0)
Revisie: 6934 | Datum: 2008-12-22T15:56:23Z
Bron: https://wiki.osdev.org/Root_System_Description_Pointer
Revisie-URL: https://wiki.osdev.org/index.php?title=Root_System_Description_Pointer&oldid=6934
Doorverwijzing: RSDP

````text
#REDIRECT [[RSDP]]
````

<a id="w002474"></a>

## W002474 — Template talk:In Progress

Bron-ID: W002474
Namespace: Template talk (11)
Revisie: 6939 | Datum: 2008-12-24T01:49:52Z
Bron: https://wiki.osdev.org/Template_talk:In_Progress
Revisie-URL: https://wiki.osdev.org/index.php?title=Template_talk%3AIn_Progress&oldid=6939

````text
== "Page or section" ==

I think that "or section" should be added after "This page" so that it fits in and days what it really means other than saying that the entire page is a WIP over and over and over again. --[[User:Troy martin|Troy Martin]] 01:49, 24 December 2008 (UTC)
````

<a id="w002475"></a>

## W002475 — OSDev Wiki talk:License

Bron-ID: W002475
Namespace: OSDev Wiki talk (5)
Revisie: 6947 | Datum: 2008-12-26T23:51:41Z
Bron: https://wiki.osdev.org/OSDev_Wiki_talk:License
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDev_Wiki_talk%3ALicense&oldid=6947

````text
The "See the polls" link goes nowhere in particular. Anyone know what thread its supposed to link to? --[[User:Oros|OrOS]] 03:06, 26 December 2008 (UTC)
:I'd say that http://forum.osdev.org/viewtopic.php?f=8&t=13841 and http://forum.osdev.org/viewtopic.php?f=8&t=13840 would be the topics. --[[User:Troy martin|Troy Martin]] 23:51, 26 December 2008 (UTC)
````

<a id="w002476"></a>

## W002476 — El-Torito

Bron-ID: W002476
Namespace: Main (0)
Revisie: 27153 | Datum: 2022-06-03T12:46:54Z
Bron: https://wiki.osdev.org/El-Torito
Revisie-URL: https://wiki.osdev.org/index.php?title=El-Torito&oldid=27153

````text
'''El-Torito''' is a standard for creating bootable optical media
like CD-ROMs, DVD, or BD. It is an add-on to the [[ISO 9660]]
filesystem.

El-Torito describes different ways of booting from an optical medium.
The starting point can either be an emulated floppy drive, an
emulated hard disk, or a plain block address in the ISO filesystem.

Oldfashioned ways of boot preparations are described in
the articles about [[Bootable CD|bootable CD emulated as a floppy drive]]
and about [[Bootable El-Torito CD with GRUB Legacy|
bootable CD using no emulation (via GRUB legacy)]].

More modern ways are no-emulation setups as made by
[http://www.gnu.org/software/grub/manual/html_node/Making-a-GRUB-bootable-CD_002dROM.html GRUB2 script grub-mkrescue]
or as described by the
[http://www.syslinux.org/wiki/index.php/ISOLINUX ISOLINUX wiki]
of the
[http://www.syslinux.org/wiki/index.php/The_Syslinux_Project Syslinux project].

== Document Scope ==
The intention of this document is to explain what you need to know
from the El-Torito standard to create a bootable CD with your own
boot image. Details about the options for ISO 9660 image production
can be found on the [[Mkisofs]] page.

== El Torito Structure ==
El Torito booting begins by the Boot Record of the ISO 9660
filesystem at block address 0x11. See also article [[ISO 9660]].
This Boot Record points to a Boot Catalog which is stored
in one or more blocks inside the ISO 9660 filesystem.

The content of Boot Record and Boot Catalog is created during
filesystem production by the ISO 9660 producing software.

The Boot Catalog lists the available boot images which may be
prepared for multiple system architectures, called "platforms".
These images are marked either as emulated floppies, or as
emulated hard disks, or as no-emulation images.
In any case they are the first stage in the boot process
where custom code from the ISO filesystem can be executed.

While the emulated boot images are to be interpreted
by the firmware as is supposed for floppies and hard disks,
the no-emulation boot images are on their own:

PC-[[BIOS]] reads from the Boot Catalog the number of
blocks to load, loads them (usually to segment 07c0) and then
executes them as code. As with a normal floppy or hard disk,
the DL register contains the BIOS drive number.

[[EFI]] interprets the boot image as [[FAT]] filesystem and
looks up a standardized file path for further processing. The
file name depends on the processor type.
E.g. /EFI/BOOT/BOOTIA32.EFI for 32 bit x86 and /EFI/BOOT/BOOTX64.EFI
for 64 bit x86.

The original El Torito specification mentions platforms
"80x86", "PowerPC", and "Mac".
Boot setups based on GRUB2 and ISOLINUX use 0x00 = "80x86" for PC-BIOS,
and platform id 0xef for EFI which is defined by the UEFI specification.

=== Creating the Structure in the ISO filesystem ===
The following text assumes that the ISO 9660 producing software is
compatible to [[Mkisofs]]. E.g. mkisofs, genisoimage, or xorrisofs.

[[Mkisofs]] expects that the boot images are submitted as
data files like any other file in the emerging ISO 9660
filesystem. Normally they are part of the directory trees
which get copied into the filesystem.

In most cases the Boot Catalog is represented in the filesystem
as data file, too. Its content is nevertheless composed by
[[Mkisofs]].

The boot images of ISOLINUX and GRUB2 expect to contain
some information about the ISO filesystem and their own
location and length. This information is called Boot Info Table
and gets inserted by [[Mkisofs]], if desired.
Boot Info Table is not specified by El Torito, but is rather
a convention introduced by boot loader developers.

=== Example of ISO Filesystem Production Run for BIOS ===
Put all necessary directories and files underneath directory
./prepared_for_iso . It will be copied as root directory of
the ISO filesystem. Especially make the boot image for PC-BIOS
available at path ./prepared_for_iso/boot/loader.sys ,
so that it will appear in the ISO filesystem as /boot/loader.sys .

Choose an ISO 9660 producing program and eventually its
mkisofs emulation command:
<Source lang=bash>
prog="mkisofs"
</Source>
or
<Source lang=bash>
prog="genisoimage"
</Source>
or
<Source lang=bash>
prog="xorriso -as mkisofs"
</Source>

Produce ISO 9660 filesystem image ./bootable.iso :
<Source lang=bash>
$prog -R -J -c boot/bootcat \
      -b boot/loader.sys -no-emul-boot -boot-load-size 4 \
      -o ./bootable.iso ./prepared_for_iso
</Source>

'''-c boot/bootcat''' - Makes the Boot Catalog accessible as
data file.

'''-b boot/loader.sys -no-emul-boot''' - Causes /boot/loader.sys
to be listed in the emerging Boot Catalog as no-emulation
boot image for PC-BIOS.

'''-boot-load-size 4''' - Specifies the number of 512-bytes sectors
to load. Four 512-byte sectors (2048 bytes) is one CD sector
and is the number supported by most BIOS.

If your boot image needs updating of a Boot Info Table,
then add option '''-boot-info-table''' after -boot-load-size 4.

After this production run, the emerged data file
./bootable.iso can be burned onto optical media.

'''Warning''': Defaults of '''xorriso -as mkisofs''' emulation might deviate from those of '''mkisofs'''.
E.g. not specifying '''-boot-load-size''' assumes a default of four sectors instead of the entire boot file.

=== Hybrid Setup for BIOS and EFI from CD/DVD and USB stick ===
The EL Torito Boot Catalog can offer in the same ISO filesystem
alternative boot images for PC-[[BIOS]] and for [[EFI]].

But El Torito is interpreted by the firmware only if presented on
an optical medium: CD, DVD, BD. For booting PC-BIOS and EFI from
USB stick or other hard-disk-like devices, there is need for an
[[MBR]] and, if desired, for a [[GPT]].

The machine code of the MBR of SYSLINUX can be adjusted to jump with
its execution onto the BIOS El Torito boot image of ISOLINUX. This is known
as "isohybrid".
The MBR and BIOS El Torito boot image written by grub-mkrescue have the same
capability.
The MBR partition table does not matter for the first stage of BIOS booting.

UEFI regulates the relation of MBR partition table and GPT. So for booting
via EFI firmware, the partition tables do matter. The MBR machine code is to
be ignored by EFI, unless in Legacy BIOS mode.

The most modern UEFI layout prescribes an MBR with a single partition which
starts at Logical Block Address 1 and covers the whole image size.
It shall have MBR partition type 0xee.
In this case there is a GUID Partition Table (GPT) which in one of
its partitions marks the EFI System Partition with the FAT filesystem.
Neither MBR partition nor GPT partitions can lead to an address where 
the ISO filesystem can be mounted, unless the ISO offers a second
set of volume descriptors and file tree at a higher block address.
Script grub-mkrescue produces this layout for booting via EFI.

The UEFI legacy partition layout (not to be confused with Legacy BIOS mode)
prescribes that the EFI System Partition shall be marked by an MBR
partition of type 0xef. Other MBR partitions are permissible. E.g.
one which starts at Logical Block Address 0 to make the ISO mountable.
It is not allowed, nevertheless, to let the ISO partition enclose the
EFI System Partition.

Several popular Linux distributions offer a layout that does not comply
to either of the UEFI alternatives. The MBR marks the whole ISO by a
partition of type 0x00. Another MBR partition of type 0xef marks a
data file inside the ISO filesystem with the image of the EFI System 
Partition FAT filesystem. Nevertheless there is a GPT which also marks
the EFI System Partition image file. This GPT is to be ignored by any
UEFI compliant firmware. The nesting of the MBR partitions is made
acceptable by giving the outer MBR partition the type 0x00, which UEFI
specifies to be ignored.

Debian used for its installation image debian-7.1.0-amd64-netinst.iso
the following options of xorriso's mkisofs emulation:

<Source lang=bash>
xorriso -as mkisofs \
  ... \
  -c isolinux/boot.cat \
  -b isolinux/isolinux.bin \
  -no-emul-boot -boot-load-size 4 -boot-info-table \
  -isohybrid-mbr ./syslinux/usr/lib/syslinux/isohdpfx.bin \
  -eltorito-alt-boot \
  -e boot/grub/efi.img \
  -no-emul-boot -isohybrid-gpt-basdat \
  ... \
  ./boot1 \
  ...
</Source>

Option -eltorito-alt-boot separates the settings
for the BIOS boot image (-b) from those for the EFI
boot image (-e).

The files isolinux/isolinux.bin and boot/grub/efi.img
get into the ISO image from source directory ./boot1.

File ./syslinux/usr/lib/syslinux/isohdpfx.bin is copied
directly from the SYSLINUX installation into the System
Area of the ISO image. It contains the isohybrid MBR template.
Option -isohybrid-gpt-basdat announces the FAT image
boot/grub/efi.img as GPT partition.

See [http://www.gnu.org/software/xorriso/man_1_xorrisofs.html man 1 xorrisofs]
for details about the options which are in part not
supported by mkisofs from cdrtools.

See [https://dev.lovelyhq.com/libburnia/libisofs/raw/master/doc/boot_sectors.txt libisofs/doc/boot_sectors.txt], section "SYSLINUX Isohybrid MBR", for informtion on writing a custom isohybrid mbr.

== A BareBones Boot Image with Boot Information Table ==
A Boot Information Table may be put into a boot image. It starts at offset 8 and is 56 bytes long. If you are familiar with Bios Parameter Blocks, you can think of the Boot Information Table as doing a similar job.

<Source lang=asm>
[ORG 0x7C00]
[BITS 16]
start:
  jmp	0x0000:boot
  times 8-($-$$) db 0

  ;	Boot Information Table
  bi_PrimaryVolumeDescriptor  resd  1    ; LBA of the Primary Volume Descriptor
  bi_BootFileLocation         resd  1    ; LBA of the Boot File
  bi_BootFileLength           resd  1    ; Length of the boot file in bytes
  bi_Checksum                 resd  1    ; 32 bit checksum
  bi_Reserved                 resb  40   ; Reserved 'for future standardization'
	
boot:
  ;	Boot code here - set segment registers etc...
  jmp $
</Source>

If you are already familiar with boot sectors for other media, this should look quite familiar already. As usual, the first thing that the code does is jump to a known segment:offset location, where it will presumably set data and stack segment registers. I have used the times macro to provide padding for the table. This has two distinct advantages: firstly, if you decide to add anything else before the table, it will continue to reside at offset 8. Secondly, if you place more than 8 bytes worth of code before the table, you will get an assembler error. The Boot Information Table may need some explanation:

'''bi_PrimaryVolumeDescriptor''' contains the LBA address of the Primary Volume Descriptor, which contains information such as the address and length of the root directory. In practice, this value will normally be 0x10, because [[ISO 9660]] reserves sectors 0x00-0x0F as a 'system area'. And yes - no more CHS calculations, for El-Torito no emulation mode on a modern PC, it's normally a safe bet that int 0x13 extensions are available!

'''bi_BootFileLocation''' is another LBA address containing the location of the boot file on the CD. The size of the boot file can be found (in bytes) in the '''bi_BootFileLength''' field.

'''bi_Checksum''' is a 32 bit checksum of all the 32-bit values in the boot file, starting at offset 64 (just after the boot information table).

=== Important note! ===

There are some BIOS bugs that might prevent No Emulation booting from working properly on certain machines.  A brief list of them is available [http://forum.osdev.org/viewtopic.php?f=1&t=18911&p=146444#p146433 here]

=== What Next? ===
The rest of the CD boot process relies on [[ISO 9660]] rather than the El-Torito standards. After you have done the 'normal' boot sector jobs, such as setting segment registers to known values and saving the contents of DL (which contains the BIOS boot drive number), you can start reading the file system. This must happen by code in the 2k that you have available in your boot sector. The Boot Info Table tells the Logical Block Address (LBA, in units of 2k) and size of the file from where the first 2k came. So you can load more machine code from it without already having an ISO 9660 filesystem driver.

Generally, you will want to load such a driver, so that you can find and load more files of your next stage loader. You have an option here - you can either use the path tables to find a file, or perform a full directory walk.

==See Also==
===Articles===
* [[Genisoimage]]
* [[ISO 9660]]
===External Links===
* [[wikipedia:El Torito (CD-ROM standard)|El Torito (CD-ROM standard)]] on Wikipedia
* [https://web.archive.org/web/20180112220141/https://download.intel.com/support/motherboards/desktop/sb/specscdrom.pdf "El Torito" Bootable CD-ROM Format Specification, Version 1.0 at the Wayback Machine (archived Jan 12, 2018)]
* [http://littlesvr.ca/isomaster/eltoritosuppl.php Boot Information Table format explained]
* [https://dev.lovelyhq.com/libburnia/libisofs/raw/master/doc/boot_sectors.txt Byte level description of boot entry points in ISO 9660 filesystems]
* [http://www.uefi.org/sites/default/files/resources/2_4_Errata_B.pdf UEFI specifications, version 2.4 Errata B]
* [https://gitlab.com/bztsrc/bootboot/-/blob/master/mkbootimg/gpt.c#L218 mkbootimg] dependency-free C source, generates a BIOS/UEFI compatible ISO 9660 Boot Catalog after the GPT

[[Category:Bootloaders]]
````

<a id="w002477"></a>

## W002477 — Double Fault

Bron-ID: W002477
Namespace: Main (0)
Revisie: 6969 | Datum: 2008-12-31T16:22:06Z
Bron: https://wiki.osdev.org/Double_Fault
Revisie-URL: https://wiki.osdev.org/index.php?title=Double_Fault&oldid=6969
Doorverwijzing: Exceptions

````text
#REDIRECT [[Exceptions#Double Fault]]
````

<a id="w002478"></a>

## W002478 — Shell

Bron-ID: W002478
Namespace: Main (0)
Revisie: 19413 | Datum: 2016-07-09T21:56:09Z
Bron: https://wiki.osdev.org/Shell
Revisie-URL: https://wiki.osdev.org/index.php?title=Shell&oldid=19413
Doorverwijzing: Introduction

````text
#REDIRECT [[Introduction#What is a shell?]]
````

<a id="w002480"></a>

## W002480 — Talk:Far Call Trick

Bron-ID: W002480
Namespace: Talk (1)
Revisie: 6999 | Datum: 2009-01-06T02:43:24Z
Bron: https://wiki.osdev.org/Talk:Far_Call_Trick
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AFar_Call_Trick&oldid=6999

````text
I'm not sure why you start by making a far jump to a known location. Why not use this shorter code (not tested, but should AFAIK work - I did something similar in pmode once):

 mov ax, 0              ; New IP
 mov bx, 0x2000         ; New CS
 
 push bx                ; Push new CS
 push ax                ; Push new IP
 
 retf                   ; Sneaky far jump

As a wiki article it would be nice to explain how the trick works. You should also mention the code is for real mode.

--[[User:Walling|Walling]] 12:05, 5 January 2009 (UTC)


Or what about
 push cs
 push word return_address
 push word [segment_no]
 push word 0
 retf

Or (most logical one)
 mov word [segment_no-2], 0
 call far [segment_no-2]

@Walling, your code works like a far jump
- [[User:Combuster|Combuster]] 16:17, 5 January 2009 (UTC)


Well that works too, thanks for the suggestions! --[[User:Troy martin|Troy Martin]] 02:43, 6 January 2009 (UTC)
````

<a id="w002484"></a>

## W002484 — Category talk:X86 Instructions

Bron-ID: W002484
Namespace: Category talk (15)
Revisie: 7024 | Datum: 2009-01-08T16:31:58Z
Bron: https://wiki.osdev.org/Category_talk:X86_Instructions
Revisie-URL: https://wiki.osdev.org/index.php?title=Category_talk%3AX86_Instructions&oldid=7024

````text
Hmmm... what exactly is it that we see growing here? Is this intended as a quick reference for oft-used instructions, or is there some idea of copying the Intel Manual volume 2? Basically I'd consider the x86 instructions to be an excellent case of RTM as far as an OSDev Wiki is concerned, and Intel does a good job documenting their instruction set already... [[User:Solar|Solar]] 14:04, 8 January 2009 (UTC)
:Agreed. This was btw discussed before (at [[Talk:Main_Page/archive#Instructions]]) and I still think that we should merge the majority into a single article and get rid of the category. - [[User:Combuster|Combuster]] 16:31, 8 January 2009 (UTC)
````

<a id="w002485"></a>

## W002485 — Real mode assembly III

Bron-ID: W002485
Namespace: Main (0)
Revisie: 27979 | Datum: 2023-07-09T17:24:16Z
Bron: https://wiki.osdev.org/Real_mode_assembly_III
Revisie-URL: https://wiki.osdev.org/index.php?title=Real_mode_assembly_III&oldid=27979

````text
{{Rating|1}}

In installment number three of the real mode assembly bare bones series, we're covering a bit of stuff on timing, it's uses, and doing a quick skim of INT 15h.

==Timing: Fun in a Few Interrupts==

<source lang="asm">
 xor ax,ax
 int 1Ah
</source>

That little sprig of code grabs the timer ticks since midnight in CX:DX. It's a great seed for generating random numbers. Think what would happen if you threw in some things like this:

<source lang="asm">
 ror dx,2
 shr cx,3
 xchg dl,dh
</source>

You could use them in conjunction with some other instructions (think multiplying, adding, subtracting, etc.) to really create a random number.

'''Quick rundown of ticks:''' There are about 18.2 timer ticks per second, and every tick, interrupt 08h is fired. int 08h calls int 1Ch, which you can use for your own personal use. I'll describe using the [[IVT]] to program software interrupts you can call for uses like system calls and such. To time approximately one second, you can use hlt 18 times. Tip: use loop to make the resulting code smaller and more readable.

==INT 15h==

===AH=86h - Microsecond Timing===

INT 15h does some nifty stuff. Originally meant for accessing the cassette tape system on the original PC and more obscure PCjr, most if not all systems since the mid '80s don't have tape drives. So instead, INT 15h has commonly been used for miscellaneous services. AH=86h is microsecond timing, taking CX:DX as an interval of microseconds to wait for until an iret instruction fires and you get back to whatever you were doing. Note that there is a wait resolution of about 977 microseconds, so it's not extremely accurate, but it's real useful!

===AH=0xC2 - PS/2 Mouse===

Almost any BIOS since the PS/2 uses this int (at least AH=0xC2) for the PS/2 mouse. The full range of calls on INT 15h are available at your nearest [[RBIL]], but the mouse calls are discussed briefly here.

There are two extremely simple calls: AX=0xC200, which enables or disables the mouse (BH=01h for enabled, 00h for disabled), and AX=0xC201, which simply resets the mouse regardless of what's in BX.

You'll have to [http://www.ctyme.com/intr/int-15.htm look for stuff] on these controls to really learn how to use them though.

[[Category:Assembly]]
[[Category:Real mode assembly]]
 <- [[Real mode assembly II]]  |  [[Real mode assembly IV]] ->
````

<a id="w002486"></a>

## W002486 — Category:Real mode assembly

Bron-ID: W002486
Namespace: Category (14)
Revisie: 8525 | Datum: 2009-07-12T15:44:26Z
Bron: https://wiki.osdev.org/Category:Real_mode_assembly
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AReal_mode_assembly&oldid=8525

````text
The content of this category is part of the real mode assembly bare bones series.

[[Category:Bare bones tutorials]]
[[Category:Assembly]]
````

<a id="w002487"></a>

## W002487 — Real mode assembly IV

Bron-ID: W002487
Namespace: Main (0)
Revisie: 27882 | Datum: 2023-07-09T04:26:33Z
Bron: https://wiki.osdev.org/Real_mode_assembly_IV
Revisie-URL: https://wiki.osdev.org/index.php?title=Real_mode_assembly_IV&oldid=27882

````text
{{Rating|2}}

''In this fourth chapter of the [[Real mode assembly bare bones]] series, we're going to program the [[Interrupt Vector Table]] to create a system call interrupt for our use.''

==The IVT, in all it's Glory==

The IVT is an array of 1024 bytes, arranged in 16-bit words. The address of the interrupt handler is stored with the offset word first, segment second. Thing's couldn't be easier.

==Adding a BIOS INT Style Handler==

My style of handler goes like this:

<source lang="asm">
 inthandler:
   cmp ah,0
   je .ahzero
 
   cmp ah,1
   je .ahone
 
   cmp ah,2
   je .ahtwo
 
   ; ......
 
   mov si,msgBadAH
   call print_string
   cli
   hlt
 
 .ahzero:
   ; Do whatever needed here
   iret
</source>

But that's just an outline, you'd need to do it up however you need to use your calls. The FLAGS register is restored in an iret so you need to set it on the stack again depending on how it's set before the iret is done!

==Adding the Handler to the IVT==

This is fairly simple.

# First, null out ES.
# Set AL=interrupt number, and BL=4h.
# Multiply AL by BL, and then put the result (AX) in BX.
# Move the word that is the address of the start of your interrupt handler into [es:bx].
# Add 2 to BX.
# Move your handler's segment into [es:bx].
# Restore your original ES and you're done!

[[Category:Assembly]]
[[Category:Real mode assembly]]
 <- [[Real mode assembly III]]
````

<a id="w002489"></a>

## W002489 — Double Buffering

Bron-ID: W002489
Namespace: Main (0)
Revisie: 28579 | Datum: 2024-01-26T16:31:33Z
Bron: https://wiki.osdev.org/Double_Buffering
Revisie-URL: https://wiki.osdev.org/index.php?title=Double_Buffering&oldid=28579

````text
{{Rating|1}}
Double-buffering is a term that is well known throughout the graphics industry. Most applications (primarily games) seldom work without it as it has many advantages over single-buffering (but also has a few disadvantages).

== What Is It? ==
==== The Buffers ====
Double-buffering possibly sounds harder than it actually is. In theory, you have a buffer A and a buffer B, which are usually respectively called the front-and back-buffer. On single-buffered displays, you draw straight to video memory, so suppose video memory is located at address <code>0xB8000</code> (like console video memory is), you just start modifying values starting from this address (as you probably know) to show characters on the screen. If you would do this with double-buffering (which isn't a very good idea with console video memory, by the way), <code>0xB8000</code> would represent buffer A (the front-buffer) whilst buffer B is created and allocated by your favourite <code>malloc</code> function (or <code>new[]</code> operator).

==== Where To Write To ====
Instead of writing directly to console video memory (at <code>0xB8000</code>), you write everything to the address where Buffer B is located. The process of drawing characters is exactly the same as before, except you're now writing to another memory address.
When you're done drawing, you copy the contents of the back buffer (buffer B) to the front buffer (buffer A). This process is commonly called 'swapping the buffers', though swapping isn't interpreted as switching the contents around, but more of in 'switching the order of the buffers'. By 'swapping the buffers', the back buffer is now seen by the user and carries the same pixel values as the front buffer. The back buffer can then be freely modified again until ready to be then swapped again.

==== What Actually Is Seen ====
When using double-buffering, the user is looking at Buffer A while you're drawing to Buffer B. When using single-buffering, the user is looking at buffer A at the same time you're modifying that buffer. So that means the user doesn't see any pixels being modified when they are looking at the screen. They will notice the changes as soon as you swap the buffers.

==== Important Note ====
I used console video memory as an example here, as practically every OS developer knows how to use this, but it is still not recommended to use double-buffering on a console display, as it is practically useless and wastes memory (unless you're using some kind of game loop for your OS). Double-buffering can be very useful when you're building your own GUI though. On console displays, you usually also don't need double-buffering since there are no problems with artifacts.

==== Example ====
For example, without double buffering consider a simple game rendering like the following:

<source lang="c">
void run()
{
    while(running)
    {
        /* update input */

        /* update game logic */

        clear_screen();
        draw_background();
        draw_levels();
        draw_picksup();
        draw_characters();
    }
}
</source>

Now imagine if each of those functions drew directly to the frame buffer. The video card and monitor may update just after you have drawn the background, or it may update just after you have cleared the screen. More likely, you're going to end up with a combination of effects with resulting in your screen flickering and seeing through objects.

Now imagine if each of these functions drew to a secondary buffer the size of the frame buffer. Each time you modify a pixel it won't be writing directly to the video card. Instead, after <code>draw_characters()</code> you would call:

<source lang="c">
memcpy(framebuffer, backbuffer, width * height * bytesperpixel);
</source>

so the final image in its entirety is sent to the frame buffer, not just parts of it.

== Advantages ==
Now why would you want to use double-buffering over single buffering in e.g. your GUI, games, or video displays? Well, the main advantages of double-buffering are:

* The user does not see every pixel modification (so if the same pixel gets modified 5 times, the user will only see the last one). This helps preventing 'flickering' (or screen flashes).
* Double-buffering usually takes care of artifacts you or the user might be experiencing.
* Writing to video memory is only performed once, on the buffer swap, instead of repeatedly for every pixel of which some might be overlapped later and might be invisible.
* Some use cases (e.g. scrolling text, writing transparent pixels, etc) require reading previously written pixels. With double buffering, you use the front buffer instead of the back buffer. Reading from the video memory is much slower than reading from main memory.

== Disadvantages ==
Even though double-buffering is useful, it also has negative characteristics. Each is listed with an explanation and a possible solution.

=== Memory requirement ===
You need to create a second buffer with the same size of the video memory (and for large displays such as 1280x1024x32 this can be an expensive cost).

This is more of an issue on embedded and older systems with limited memory. In the example of 1280x1024x32, this would mean your application or video driver would need to allocate an extra 5 megabytes, which isn't an expensive once off cost in a modern system that has between several hundred megabytes to several gigabytes of memory.

Also, virtually every modern desktop computer has a dedicated graphics accelerator card which has its own inbuilt memory. While older graphics cards may only have 8MB, many newer high-end models can have close to several gigabytes. This is more than sufficient to store the back buffer in, as well as being optimized for this purpose. An OS should allow the video driver to allocate and manage the double buffer since each graphics vendor will usually have their own implementation.

Note that the buffer doesn't always have to have the same size as the video memory, there are also other ways to do double-buffering, for one you could also have some sort of fast run-length encoded compression system. Then again, that would have an extra performance cost if not done efficiently.

You don't have to double buffer the entire screen. For example, if only part of the screen is constantly updating (the output of a media program or a video game) then only that section of the screen needs to be double buffered and the rest of the screen can be drawn directly to the front buffer. This method saves both memory and performance.

=== Speed ===
You need to copy the back buffer to the front buffer every time the buffer needs to be swapped (in games with a frame-rate of 30, 30 times each second). So better make sure your <code>memcpy</code> and other memory functions are optimized!

However, some graphics cards have the ability to specify the address in memory where the buffer used for rendering is stored. If you're redrawing the entire scene each frame (which is often the case when rendering video or 3D), you simply need to swap two pointers (the pointer to the buffer you're drawing to, and the pointer to the buffer being drawn on screen).

=== Tearing ===
Tearing ([https://web.archive.org/web/20070321063820/http://www.nhancer.com/help/images/VSync.jpg example image]) is an artifact when the buffer is being updated while the graphics card and monitor are midway through updating the screen. The side effect is that parts of multiple frames end up being sent to the monitor at the same time. Tearing is most noticeable at animation at extremely high frame rates (100fps+) with low refresh rates (60Hz) and less noticeable vice versa, though it happens whenever the refresh rate does not match the frame rate.

Tearing can usually be ignored in general applications (office suites, web browsers, terminals) since the entire screen isn't constantly being updated. However, it still occurs, for example dragging a window around fast in a window manager that does not account for tearing ([https://web.archive.org/web/20120102123417/http://yfrog.com/j1nouveauwindowmovep example]).

==== Vertical Synchronization ====
[http://en.wikipedia.org/wiki/Vertical_synchronization Vertical synchronization], more commonly known as v-sync, is when the frame rate is synchronized to match the vertical refresh rate of the screen. This means the double buffer is copied to screen's buffer in a small period between frames known as the [http://en.wikipedia.org/wiki/Vertical_blanking_interval vertical blanking interval]. By updating the screen's buffer between frames, you ensure that only full frames are shown at a time.

To aid in synchronizing, virtually all video cards since the first home computers have the capability to enable an interrupt at the beginning of each vertical blanking interval (known as VBLANK).

An example of vertical synchronization is as follows:

<source lang="c">
/* handle VBLANK, called by the interrupt handler */
void vga_handleVBlank()
{
    if(vga_framerendered)
    {
        memcpy(vga_screen, vga_backbuffer, vga_width * vga_depth * vga_bytesperpixel);
        vga_framerendered = false;
    }
}

/* called when you finish drawing to the double buffer */
void vga_sync()
{
    vga_framerendered = 1;

    /* stick in a loop until the interrupt is fired */
    while(vga_framerendered);
}
</source>

There is no reason you HAVE to synchronize, another method could be you place a spinlock on the buffer, if you fail to acquire the lock simply skip that frame until you do. This may work fine in a video game where you redraw the frame over and over again so if you miss drawing something to the screen it will be redrawn on the following frame, but in a modern multitasking GUI environment, this is unacceptable.

In a GUI environment synchronization becomes somewhat more complex. For example see below.

Unrelated, there is also a horizontal blanking interval, which is rarely utilized. The HBLANK interrupt fires when the display has finished drawing a line. This was used by some early systems to load in a new colour palettes per-line, enabling a wider range of colours on screen at once than the hardware was designed for. HBLANK synchronization is considered depreciated on modern hardware as displays operating over digital connectors such as DVI can receive the entire frame at once, rather than line by line with a blanking interval in-between. Modern video cards may emulate HBLANK with DVI displays to a varying degree, mostly to remain compatible with legacy VGA software, however don't assume it'll be available or reliable in every video mode.

==== Triple buffering ====
[http://en.wikipedia.org/wiki/Triple_buffering Triple buffering] is when three or more buffers are being used, at the cost of more memory to store these buffers in.

In a real-time application (such as a game) utilizing triple buffering there are two back buffers and one front buffer. The system renders one frame to one back buffer, the next frame the other back buffer, the next frame to the original back buffer, etc, constantly interchanging between the two. When the VBLANK interrupt fires, the last frame fully rendered is copied to the front buffer (done efficiently, this can be a case of testing one value and swapping two pointers inside the interrupt handler without any locking).

In a system already utilizing double buffering, triple buffering is simple to implement. In most cases it can be added to the video driver without the underlying system requiring any modifications.

If losing a few MBs of memory is not an issue then triple buffering provides several advantages. First off, there is no need to synchronize frames, so the drawing algorithm can run as often and as fast as it can. Secondly, some real time applications <em>must</em> perform a consistent number of update cycles each second, in which case synchronizing with the refresh rate (which will cause the program to slow down to the speed of the monitor) or skipping frames (the time it takes to execute a single update will not be consistent; some will render frames, some won't) is not an option.

==== Something unique? ====
In GUI applications, where multiple programs are rendering to multiple front and back buffers each at their own speed you need to be somewhat creative.

For example, a method that will work is: each window could consist of a double buffer (or a single buffer that is updated on a redraw/paint event), as well as a double buffer for the entire screen. That way each program can draw on its back buffer and send it its own front buffer when it has finished drawing (either swapping pointers or <code>memcpy</code>) as often as it wants (wrapping a lock around the back buffer), then when the GUI has detected the back buffer has changed, it copies the window's front buffer (locking it temporarily) into the screen's back buffer, then when the VBLANK fires the screen's back buffer is copied to the front buffer if the screen's back buffer has changed.

Ultimately, if memory was not an issue then the best system would be to triple buffer each window and the entire desktop, allowing every window, the desktop manager, and the video card to update at it's own speed. In some GUIs each window is allowed to draw directly to the desktop's back buffer. For example in early versions of Windows programs drew directly to the desktop's back buffer, this meant that all windows overlapping that window also had to be redrawn. The disadvantage of this is that if an application doesn't respond to a redraw request straight away then you can get a visual artefact ([http://i.msdn.microsoft.com/dynimg/IC405533.png example]).

Each window manager usually has its own method. But like mentioned above, tearing is a relatively minor issue in most general GUI programs that can often be overlooked, since you will be dealing with text and images that won't be updating. Therefore, if the front buffer is in the middle of outputting to the monitor while copying the next frame to it, you might notice a slight flicker that lasts for all but 1 monitor refresh (which is no less than 1/60th of a second on most monitors).

== Examples ==
Articles with all theory are boring, so here are some examples of how to use double-buffering.
==== Buffer creation ====

<source lang="c">
/* Supposing the display is 800x600 with 32 bpp (meaning 32/8 = 4 bytes per pixel). */
uint8_t *BackBuffer = ((uint8_t *) (malloc(800 * 600 * 4)));
</source>

This implementation absorbs the same amount of memory the real video memory does, in this case, 800 * 600 * 4 = 1920000 bytes = about 1,83 MB. While using single-buffering needs 1,83 MB of RAM with our video resolution (only the display itself), double-buffering would require 2 * 1,83 MB = about 3,66 MB. The higher the resolution, the more memory is required. There are of course implementations that can use up less than that with special techniques, but for some OS developers, high resolutions, and especially with double-buffering, are expensive features.

If 3,66MB does not seem like much then if you imagine a video playing program that implements triple buffering running on a modern 1920x1080@32bpp LCD display, then the total amount of video memory to store the 3 buffers would be 24,47MB. Fortunately, as display resolution increased so does memory, so using 25MB of memory on a modern computer to play a video may be acceptable.

==== Double buffering ====
<source lang="c">
uint8_t * VidMem;
uint8_t * BackBuffer;

unsigned short ScrW, ScrH;
unsigned char Bpp, PixelStride;
int Pitch;

/*
 * Initializes video, creates a back buffer, changes video modes.
 * Remember that you need some kind of memory allocation!
 */
void InitVideo(unsigned short ScreenWidth, unsigned short ScreenHeight, unsigned char BitsPerPixel)
{
        /* Convert bits per pixel into bytes per pixel. Take care of 15-bit modes as well */
        PixelStride = (BitsPerPixel | 7) >> 3;
        /* The pitch is the amount of bytes between the start of each row. This isn't always bytes * width. */
        /* This should work for the basic 16 and 32 bpp modes (but not 24) */
        Pitch = ScrenWidth * PixelStride;

	/* Warning: 0xEEEEEEE servces as an example, you should fill in the address of your video memory here. */
	VidMem = ((byte *) 0xEEEEEEE);
	BackBuffer = ((byte *) (malloc(ScreenHeight * Pitch)));

	ScrW = ScreenWidth;
	ScrH = ScreenHeight;
	Bpp = BitsPerPixel;

	/* Switch resolutions if needed... */
	/* Do some more stuff... */
}

/*
 * Draws a pixel onto the backbuffer.
 */
void SetPixel(unsigned short X, unsigned short Y, unsigned Colour)
{	
        int offset = X * PixelStride + Y * Pitch;

        /* Put a pixel onto the back buffer here. */
	/* Remember to write to the BACK buffer instead of the FRONT buffer (the front buffer represents your video memory). */
        /* Take care of writing exactly PixelStride bytes as well */
}

/*
 * Swaps the back and front buffer.
 * Most commonly done by simply copying the back buffer to the front buffer.
 */
void SwapBuffers()
{
	/* Copy the contents of the back buffer to the front buffer. */
	memcpy(VidMem, BackBuffer, ScrH * Pitch);
}

/*
 * An example of how to use these functions.
 */
void ProgramLoop()
{
	while(Program_Is_Running)
	{
		/* Handle events, update window coordinates and other things, ... */

		/*
		 * You should probably implement some sort of FillScreen or FillRect function,
		 * or something that clears the back buffer before drawing. Setting the entire
		 * back buffer to 0 (black) will suffice for a basic system.
		 */
		memset(Backbuffer, 0, ScrH * Pitch);

		/* Draw everything: a GUI, windows, other things. This example puts 2 white pixels on the screen. */
		SetPixel(50, 50, 0xFFFFFF);
		SetPixel(25, 25, 0xFFFFFF);

		/* When done drawing, swap the buffers and let the user see what you've drawn. */
		SwapBuffers();
	}
}
</source>

I believe this example is mostly clear. As there are too many different ways of changing video modes or putting pixels, I'm going to let you fill that in yourself. If you want more information, you can go to the [[GUI]] or the [[Drawing In Protected Mode]] pages.

Note also that the above code might be very 'irregular'. As you know, there is no limit on the drawing, so it is possibly that the first draw takes 30 milliseconds, while the second draw takes 40 milliseconds or the first few seconds, you have a frame-rate of 100 frames per second whilst the next couple of seconds, you have a frame-rate of only 20 frames per second. Because of this, drawing times are usually capped. In the most common GUIs (such as the one in Windows), this is not really of any importance, since only parts of the screen are redrawn if they are 'invalidated' (usually referred to as 'Invalidated rectangles'), but in games, the screen is being constantly redrawn, usually up to 60 times per second. Today's APIs usually clamp this value to the screen refresh rate, this is why they usually have values such as '60' or '75'. Normally, a frame-rate of 25 to 30 suffices to have a 'smooth' display.

I've used some sort of loop here (it looks like a game loop), but remember that most GUIs on most OSes do not work with game loops, they usually work with some sort of invalidation system. Meaning a control is redrawn only if it is invalidated, which happens on occasions such as moving or resizing the control.

[[Category:Video]]
[[de:Double_Buffering]]
````

<a id="w002490"></a>

## W002490 — User:Creature

Bron-ID: W002490
Namespace: User (2)
Revisie: 11343 | Datum: 2011-03-29T08:33:46Z
Bron: https://wiki.osdev.org/User:Creature
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACreature&oldid=11343

````text
There is not really much to tell about me, really. I started programming when I was about 14 years old. I first started out with C++, then stopped and started Visual Basic because I had some good documentation. But after a week, I felt Visual Basic didn't give me quite the thrill C++ did, so I dropped Visual Basic and started out with C++ again. Quite crazy, huh?

== Projects ==
I've been doing a lot of projects of which very few have been released out on the internet.

==== Online Projects ====
* [http://jediknight3.filefront.com/file/Dedicated;87120 Dedicated++] - A utility tool for servers in Jedi Knight 2 (even I play games).

==== Projects Which Never Saw The Bright Internet Light ====
* Black Serpent - SDL/OpenGL Snake Clone.
* UIX - User Interface Extension - a GUI library for SDL.
* Box Guy 1 and 2 - A Block Dude SDL Game and its successor.
* Asteroid Chaos - An Asteroid OpenGL clone.
* AudioTech - A simple audio console player that plays loads of formats.
* AutoRestore - An application running in the background that notifies of closing applications and restarts them if requested.
* Back In BlackJack - A .NET BlackJack clone.
* Simple TaskMan - A (very) simple .NET task mananger.
* Boot Tuner - Utility for viewing and modifying Windows startup entries.
* Colour 2 CSS - Utility tool that converts coloured text to inline CSS.

==== Operating System ====
Like almost everyone on these forums and wiki, I also have an OS I create in my spare time. It is called Dynamix. There's not really much to say about it, the only reason I started the project is purely for auto-didactic purposes. I tend to experiment with things every now and then, but apart from that my OS can't really do anything shiny.

=== User Pages ===
{{Special:Prefixindex/User:Creature/}}
````

<a id="w002493"></a>

## W002493 — Real mode assembly I

Bron-ID: W002493
Namespace: Main (0)
Revisie: 27894 | Datum: 2023-07-09T04:44:18Z
Bron: https://wiki.osdev.org/Real_mode_assembly_I
Revisie-URL: https://wiki.osdev.org/index.php?title=Real_mode_assembly_I&oldid=27894

````text
{{Tone}}
{{FirstPerson}}
{{Rating|1}}
{{Warning|Please see the [[Real Mode OS Warning]] page before continuing.}}

''In this tutorial we will assemble a small 16-bit [[assembly|assembly language]] kernel with [[NASM]] and boot it.''

== Overview ==

You're probably going to sigh and dismiss yet ''another'' tutorial on writing operating systems in [[x86]] assembly language, especially since this one uses real mode. But there's a catch to this one; it actually does more than printing "Hello World" to the screen and halting.

For this, you'll need:
* the latest version of NASM (2.05.01 as of November 28<sup>th</sup>, 2008)
* PARTCOPY on Windows or dd on Linux
* an emulator like [[QEMU]], [[Bochs]], or [[Microsoft Virtual PC]]

== So what's it going to look like? ==
Well, there will be a single source file, the kernel. What about a [[bootloader]]? This is such a small kernel, we're not going to use a filesystem at all, we're just going to put the kernel into the first few sectors of the disk image!

The system will have a string printing call (of course), keyboard input, and a strcmp call similar to that of C, all packaged into less than a sector.

=== But I want a GUI and sound effects and all the Windows games to work on my OS... ===
[[Beginner Mistakes]]

== So where's the code? ==
Here you go, go wild.

<source lang="asm">
 org 0x7C00   ; add 0x7C00 to label addresses
 bits 16      ; tell the assembler we want 16 bit code

   mov ax, 0  ; set up segments
   mov ds, ax
   mov es, ax
   mov ss, ax     ; setup stack
   mov sp, 0x7C00 ; stack grows downwards from 0x7C00
 
   mov si, welcome
   call print_string
 
 mainloop:
   mov si, prompt
   call print_string
 
   mov di, buffer
   call get_string
 
   mov si, buffer
   cmp byte [si], 0  ; blank line?
   je mainloop       ; yes, ignore it
 
   mov si, buffer
   mov di, cmd_hi  ; "hi" command
   call strcmp
   jc .helloworld
 
   mov si, buffer
   mov di, cmd_help  ; "help" command
   call strcmp
   jc .help
 
   mov si,badcommand
   call print_string 
   jmp mainloop  
 
 .helloworld:
   mov si, msg_helloworld
   call print_string
 
   jmp mainloop
 
 .help:
   mov si, msg_help
   call print_string
 
   jmp mainloop
 
 welcome db 'Welcome to My OS!', 0x0D, 0x0A, 0
 msg_helloworld db 'Hello OSDev World!', 0x0D, 0x0A, 0
 badcommand db 'Bad command entered.', 0x0D, 0x0A, 0
 prompt db '>', 0
 cmd_hi db 'hi', 0
 cmd_help db 'help', 0
 msg_help db 'My OS: Commands: hi, help', 0x0D, 0x0A, 0
 buffer times 64 db 0
 
 ; ================
 ; calls start here
 ; ================
 
 print_string:
   lodsb        ; grab a byte from SI
 
   or al, al  ; logical or AL by itself
   jz .done   ; if the result is zero, get out
 
   mov ah, 0x0E
   int 0x10      ; otherwise, print out the character!
 
   jmp print_string
 
 .done:
   ret
 
 get_string:
   xor cl, cl
 
 .loop:
   mov ah, 0
   int 0x16   ; wait for keypress
 
   cmp al, 0x08    ; backspace pressed?
   je .backspace   ; yes, handle it
 
   cmp al, 0x0D  ; enter pressed?
   je .done      ; yes, we're done
 
   cmp cl, 0x3F  ; 63 chars inputted?
   je .loop      ; yes, only let in backspace and enter
 
   mov ah, 0x0E
   int 0x10      ; print out character
 
   stosb  ; put character in buffer
   inc cl
   jmp .loop
 
 .backspace:
   cmp cl, 0	; beginning of string?
   je .loop	; yes, ignore the key
 
   dec di
   mov byte [di], 0	; delete character
   dec cl		; decrement counter as well
 
   mov ah, 0x0E
   mov al, 0x08
   int 10h		; backspace on the screen
 
   mov al, ' '
   int 10h		; blank character out
 
   mov al, 0x08
   int 10h		; backspace again
 
   jmp .loop	; go to the main loop
 
 .done:
   mov al, 0	; null terminator
   stosb
 
   mov ah, 0x0E
   mov al, 0x0D
   int 0x10
   mov al, 0x0A
   int 0x10		; newline
 
   ret
 
 strcmp:
 .loop:
   mov al, [si]   ; grab a byte from SI
   mov bl, [di]   ; grab a byte from DI
   cmp al, bl     ; are they equal?
   jne .notequal  ; nope, we're done.
 
   cmp al, 0  ; are both bytes (they were equal before) null?
   je .done   ; yes, we're done.
 
   inc di     ; increment DI
   inc si     ; increment SI
   jmp .loop  ; loop!
 
 .notequal:
   clc  ; not equal, clear the carry flag
   ret
 
 .done: 	
   stc  ; equal, set the carry flag
   ret
 
   times 510-($-$$) db 0
   dw 0AA55h ; some BIOSes require this signature
</source>

To assemble on Windows:

<source lang="dos">
nasm kernel.asm -f bin -o kernel.bin
partcopy kernel.bin 0 200 -f0
</source>

Or on Linux:
<source lang="bash">
nasm kernel.asm -f bin -o kernel.bin
dd if=kernel.bin of=/dev/sda
</source>

Those commands assemble your kernel binary and write them to the first disk (sda might be your system disk, so use sdb, sdc etc. according to your configuration. USB-sticks also appear as one of the "sd" devices). Go ahead and test out your operating system now!

== What next? ==
Why, that's up to you of course! You could add more commands, expand your OS to another sector and learn to use the BIOS sector load functions, add a stack and improve the calls, etc.

Have fun with your OS, however you decide to write it!

'''EDIT on December 12 2008: I've written a second part to this tutorial at [[Real mode assembly II]]. This and future parts will have less code to copy and paste and more theory!'''

[[Category:Assembly]]
[[Category:Real mode assembly]]
 <- none  |  [[Real mode assembly II]] ->
````

<a id="w002494"></a>

## W002494 — Spinlock

Bron-ID: W002494
Namespace: Main (0)
Revisie: 27583 | Datum: 2023-01-13T03:08:34Z
Bron: https://wiki.osdev.org/Spinlock
Revisie-URL: https://wiki.osdev.org/index.php?title=Spinlock&oldid=27583

````text
==Overview==

Like all forms of reentrancy locks, spinlocks are used to ensure orderly access to a resource (e.g. data structure, piece of hardware, etc) so that software running in one context can't get an inconsistent view of that resources because software running in another context is in the middle of modifying the resource. For example, imagine a structure that contains a person's first name and last name that currently contains the data "Fred" and "Smith". If one CPU is changing this data to "Jane" and "Doe", then it might change the first name to "Jane" and then change the last name to "Doe", and a different CPU (or thread, or IRQ handler) might access this structure at the wrong time and read "Jane" and "Smith". To prevent this problem you could use a lock, such that only one context can have the lock at any time and only the context that has the lock is allowed to access the resource. For this example, if one CPU is changing this data to "Jane" and "Doe" it would acquire the lock, then change the data, then release the lock; and if something else attempts to access the structure it has to wait until it can acquire the lock before accessing the data.

A spinlock is a type of reentrancy lock, where the CPU repeatedly attempts to acquire the lock until it succeeds (or, the CPU "spins" until it succeeds). A lock is "uncontended" if the lock can be acquired on the first attempt, and no spinning is necessary. If a lock is "contended" (many attempts to acquire the lock are necessary) a spinlock can waste CPU time. The amount of CPU time wasted due to contention/spinning may be more or less than the overhead of other forms of locks (for example, there can be cases where wasting a little bit of CPU time spinning is better than wasting more CPU time on the overhead of switching tasks).



==Lock Construction==

===Basic Lock===

There are a few [[Atomic operation|Atomic operations]] on the x86 processor that set and compare memory or registers, that can be used as the basis of a spinlock. I'll focus on the BTS and BTR instructions for this discussion (but other instructions like CMPXCHG, ADD/SUB, INC/DEC, etc can also be used).

The basic lock looks something like this (but typically implemented as macros):

<source lang="asm">
acquireLock:
.retry:
    lock bts [lock],0
    jc .retry
    ret


releaseLock:
    lock btr [lock],0
    ret
</source>


===Improved Lock===

The basic lock has a few performance problems. The LOCK prefix can grant one CPU exclusive use of the bus, and therefore prevent other CPUs from accessing the bus for a (very short) period of time. This isn't a problem if the LOCK prefix isn't used often, but (when there's contention) the basic lock shown above uses it continuously and can therefore seriously effect other CPU's ability to use the bus and slow them down. To avoid this, it's better to avoid the LOCK unless it's necessary. This can be done by testing if the lock is free before attempting to acquire the lock:

<source lang="asm">
acquireLock:
    lock bts [lock],0        ;Attempt to acquire the lock (in case lock is uncontended)
    jc .spin_wait            ;Spin if locked ( organize code such that conditional jumps are typically not taken ) 
    ret                      ;Lock obtained

.spin_wait:
    test dword [lock],1      ;Is the lock free?
    jnz .spin_wait           ;no, wait
    jmp acquireLock          ;maybe, retry ( could also repeat bts, jc, ret instructions here instead of jmp )
</source>

Or even:
<source lang="asm">
.spin_wait:
    test dword [lock],1      ;Is the lock free?
    jnz .spin_wait           ;no, wait

acquireLock:
    lock bts [lock],0        ;Attempt to acquire the lock (in case lock is uncontended)
    jc .spin_wait            ;Spin if locked ( organize code such that conditional jumps are typically not taken ) 
    ret                      ;Lock obtained
</source>

In addition, the LOCK prefix can be avoided when the spinlock is released. The CPU guarantees that writes to aligned uint32_t are atomic, and therefore the code to release the lock could be changed to:

<source lang="asm">
releaseLock:
    mov dword [lock],0
    ret
</source>

There is also an issue on CPUs that support hyper-threading. In this case, the real CPU's resources (pipelines, etc) are shared by 2 (or more) logical CPUs, and if one of these logical CPUs is spinning it can waste resources that could have been used by the other logical CPU. To reduce this problem Intel introduced the PAUSE instruction, which is meant to be used in tight polling loops (like spinlocks). The opcode for the PAUSE instruction was specially chosen so that it behaves like a NOP on older CPUs (it's literally a "REP NOP" on older CPUs, where the "REP" prefix is ignored). To improve performance on CPUs that support hyper-threading, the spinlock would be modified like this:

<source lang="asm">
acquireLock:
    lock bts [lock],0        ;Attempt to acquire the lock (in case lock is uncontended)
    jc .spin_with_pause
    ret

.spin_with_pause:
    pause                    ; Tell CPU we're spinning
    test dword [lock],1      ; Is the lock free?
    jnz .spin_with_pause     ; no, wait
    jmp acquireLock          ; retry

releaseLock:
    mov dword [lock],0
    ret
</source>

It is also possible to check the state of a spinlock without attempting to acquire it. This can be done without any LOCK (the CPU guarantees that reads from aligned uint32_t are atomic too):

<source lang="asm">
    cmp dword [lock],0
    je .free
    jne .locked
</source>

===Lock Location===

Modern CPUs tend to operate on cache lines; and for most modern 80x86 CPUs a cache line is a 64-byte area. Each cache line has a state (Modified, Exclusive, etc), and different CPUs request cache lines from each other (and from the memory controller). If a cache line contains a lock that is under contention, then that cache line may remain in one CPU's cache while that CPU is repeatedly testing the lock (and therefore won't cause any bus traffic), until some other CPU modifies that cache line. If another CPU modifies something in that cache line, then that cache line's data would be transferred to the other CPU and modified, then transferred back to the first CPU again (which does cost some bus traffic). To reduce unnecessary bus traffic (caused by other CPUs modifying other data that happens to be in the cache line that contains the lock), Intel recommends using an entire cache line for each reentrancy lock.

In addition, locks should be correctly aligned on a suitable boundary (e.g. a uint32_t boundary). The worst case is for a lock to be split across page boundaries and cache line boundaries; but in general there may also be penalties for accessing misaligned uint16_t/uint32_t/uint64_t's.


===Lock Debugging===

For large and complex software that has to deal with concurrency (e.g. kernels) some types of mistakes involving the incorrect use of reentrancy locks can be hard to avoid. Worse, bugs involving reentrancy locks often depend on exact timing, and can be intermittent bugs that are difficult to detect and correct. These problems include things like "deadlock" (e.g. where whoever acquired the lock attempts to acquire it a second time), releasing a lock that wasn't acquired, failing to release a lock quickly enough (and causing excessive lock contention), etc.

It's easy to add an extra check in the code to release a lock, to detect if the lock was actually acquired first. For other checks extra information is needed.

A spinlock itself only really needs a single bit, but typically in practice a full uint32_t or an entire cache line is used instead. The extra space can be used to store additional information to help detect mistakes. For a simple example, if a uint32_t is used then one bit can be the lock itself, and the remaining 31 bits could be used to store something to identify who has acquired the lock (e.g. a CPU number or thread ID). That way, the code to acquire a lock could be modified to check if the same CPU or same thread is trying to acquire a lock a second time (and detect some types of deadlock), and the code to release a lock could be modified to detect if the lock is being released by the same CPU or thread that acquired it.

In a similar way, a second uint32_t could be used as a counter to keep track of how many attempts to acquire the lock have failed (e.g. due to lock contention), and these counters can be used for finding performance bottlenecks and scalability problems. A reentrancy lock could also be "fully instrumented" to track other information, like the number of times it has been acquired and the total number of cycles it has been acquired for (to determine the average number of cycles the lock remains acquired).

Of course it's possible to use conditional compilation to enable/disable these extra checks. For example you might have a "production" version of your kernel with none of these extra checks enabled (for better performance); and a "testing" version of your kernel with all of these checks enabled.

==Spinlocks in C==
Here is an example implementation of spinlocks in C using standard atomic operations.
<source lang="c">
#include <stdatomic.h>

atomic_flag example_lock_variable = ATOMIC_FLAG_INIT;

void acquire( atomic_flag * lock )
{
    while( atomic_flag_test_and_set_explicit( lock, memory_order_acquire ) )
    {
        /* use whatever is appropriate for your target arch here */
        __builtin_ia32_pause();
    }
}

void release( atomic_flag * lock )
{
    atomic_flag_clear_explicit( lock, memory_order_release );
}
</source>
Declare locks using the atomic_flag type and the ATOMIC_FLAG_INIT initializer, use acquire() to acquire the lock, and use release() to release it. The acquire() function will spin until the lock is acquired before continuing. These functions can be rewritten as macros to force inlining.

==See Also==
===Articles===
*[[Synchronization Primitives]]
*[[Atomic operation]]
*[[Semaphore]]
*[[Mutual Exclusion]]
*[[Deadlock]]
===Threads===
*[[Topic:28481|Brendan explaining basic Spinlocks]]

[[Category:IPC]]
[[Category:Synchronization]]
===External Links===
*[http://www.reddit.com/r/osdev/comments/32zv69/spinlocks_and_contention_some_benchmarks/ Spinlock algorithm benchmarks]
*[http://locklessinc.com/articles/locks/ Spinlocks and Read-Write locks] - Shows basic code for some kinds of spinlocks and read-write locks
````

<a id="w002495"></a>

## W002495 — Disassemble

Bron-ID: W002495
Namespace: Main (0)
Revisie: 21457 | Datum: 2017-08-30T18:00:25Z
Bron: https://wiki.osdev.org/Disassemble
Revisie-URL: https://wiki.osdev.org/index.php?title=Disassemble&oldid=21457
Doorverwijzing: X86-64 Instruction Encoding

````text
#REDIRECT [[X86-64 Instruction Encoding]]
````

<a id="w002496"></a>

## W002496 — Talk:Disassemble

Bron-ID: W002496
Namespace: Talk (1)
Revisie: 7612 | Datum: 2009-04-07T15:00:00Z
Bron: https://wiki.osdev.org/Talk:Disassemble
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ADisassemble&oldid=7612

````text
== Compacted x86 Opcode List? ==

So is this like a copied over and (sorta) wikified x86 opcode format and instruction listing? Or is it 43 kilobytes of x86 ramblings? --[[User:Troy martin|Troy Martin]] 00:30, 22 January 2009 (UTC)


I have several problems with this:
* Performing ''your own'' disassembly has little to do with OS development in practice
* Everything mentioned in this article can be found within the intel manuals (2A and 2B)
* The same manuals tend to disagree about the opcode placement - all binary arithmetic opcodes come in six, several come in pairs (byte and w/d/q), and some comparable opcodes are positioned above each other. Mathematical operators are all over the place. There's just way more noise than the article suggests...
* Its a copy of a complete article written by someone else, which has serious copyright implications. Do you have permission?
- [[User:Combuster|Combuster]] 09:04, 22 January 2009 (UTC)

And there already is an article on [[X86 Instruction Encoding]], (which I did't write specifically for disassembly, but more for programs like IL2CPU, Bartok implementations and SharpOS AOT). [[User:Virtlink|Virtlink]] 13:27, 22 January 2009 (UTC)
----

Every definition and explaination of the instruction encoding that I've ever seen (except for this one) describes the format in hex, and therefore over complicates the otherwise simple octal nature of the encoding.
 ''Performing ''your own'' disassembly has little to do with OS development in practice''
I disagree, there are several places where disassembly can be useful in a kernel.  It's useful for displaying crash dumps, emulating protected instructions in user code or gathering detailed information about page faults.
 ''Everything mentioned in this article can be found within the intel manuals (2A and 2B)''
No it cant, read the article.  This is an entirely different approach to understanding the instruction encoding.  This post describes the instruction encoding with octal instead of hex making the task of encoding or decoding the instructions much easier than hex descriptions.
 ''Its a copy of a complete article written by someone else, which has serious copyright implications. Do you have permission?''
Its a news group post as outlined at the beginning of the document.  There was no copyright so no permission to distribute it is required.  I left the original author, forum, topic and post date on the page for reference.
 ''there already is an article on [[X86 Instruction Encoding]]''
This page is a different approach which simplifies things.  [[X86 Instruction Encoding]] defines things in hex which, in my opinion, is the wrong way to present it.  Nearly the entire instruction set is defined here using the more simple octal approach.

[[User:Tantrikwizard|Tantrikwizard]] 18:29, 22 January 2009 (UTC)



Yet another soul caught unaware by the [http://en.wikipedia.org/wiki/Berne_Convention_for_the_Protection_of_Literary_and_Artistic_Works Convention of Berne] - If there's no message, its "all rights reserved". I therefore must delete this unless you can provide a legal reason not to.

Rules aside, there's '''nothing''' in this article that I hadn't figured by looking at (mainly the opcode map) the intel manuals. Yes the i386 has 8 registers which makes octal a reasonable approach, but only a limited set of unary operators come in eights. basic binary maths come in groups of six, and shifts/divs/multiplies are odd exceptions. BCD stuff is all over the place, as are the prefixes. There are just too many exceptions to claim that everything's grouped in sets of eight - everything's grouped in natural alignment: eight if there's an opcode for each register, two for each operation to distinguish byte and larger sizes, four if a memory operand can be used in either place, and then the holes are filled with whatever random stuff is left. In other words: ''The x86 is a mess'' - [[User:Combuster|Combuster]] 01:00, 23 January 2009 (UTC)




I think that we should try to contact the original author of this because it would be a waste to have to remove this excellent description of the x86 that takes it from a hex encoded instruction set with heaps of exceptions to an octal with only a few.
And, Combuster; Yes the x86 is a mess, but it evolved that way from a clean start. Thinking of instructions this way means that it is easier to decode ''most'' opcodes, and the rest can be decoded in a separate code block.

Addendum: I've emailed the author of the original asking for permission. I asked him to inform us of his decision.

- [[User:Thepowersgang|Thepowersgang]] 03:14, 23 January 2009 (UTC)
----
The octal approach is nice, but the article seems to focus mainly on convincing us that octal is better, with lots of examples and the like.
When we don't get permission, it might be an idea to rewrite it to your own wording. In addition to changing the mono spaced text into tables, create headers. Clarify the hexadecimal, octal, decimal numbers, since just looking at a number currenlty doesn't tell me which number system is used.
And when we do get permission, do we have the permission to modify it from the original? (Which will surely happen since this is a wiki.) [[User:Virtlink|Virtlink]] 10:37, 23 January 2009 (UTC)

----
 I've emailed the author of the original asking for permission.
Thanks for that.  I remember this post coming across the alt.lang.x86 about 10 years ago and lost it.  Subsequently I wrote an assembler then re-discovered this post.  After re-writing the binary generator portion using this octal approach the resulting code was about 1/2 the size than using the intel hex descriptions (I'm not so sharp as combuster to recognize the octal nature of the instruction encoding).

 When we don't get permission, it might be an idea to rewrite it to your own wording. 
Yea, its not put together well enough to be a beginners article or cross reference, its more of an informative post.  It should be redone for osdev.org IMO.  I can assist here and there if this effort is undertaken.
[[User:Tantrikwizard|Tantrikwizard]] 05:53, 24 January 2009 (UTC)

----
On the note of emailing, I sent the email to the listed address but it doesn't seem to be valid anymore (GMail failed to send it) so we should probably just re-write it ourselves.

- [[User:Thepowersgang|Thepowersgang]] 14:28, 26 January 2009 (UTC)

== Silly ==

This article is completely silly. "The 80x86 is an Octal Machine" says it all. Of course, the 80x86 is binary, and as such, it shouldn't surprise us to find that groups of instructions that use 8 different parameter combinations to have a count of 0-7 of certain adjacent three bits. The fact that this may be either 0-7 or 8-e in hex is also not a big surprise. 'Octal' says something about the [i]representation[/i] of a number, to call the instruction set 'octal' is just plain silly. [[User:Jal|Jal]] 15:20, 12 February 2009 (UTC)

:Which is a good reason this article was the victim of a content wipe. As well as the fact that it has a freaking horrible page name for a wiki. "x86 Opcode Breakdown" would be better. --[[User:Troy martin|Troy Martin]] 23:36, 13 February 2009 (UTC)

== Deletion? ==

Since nobody has made any move on rewriting this article, and several members (and I agree) think the article is "completely silly" anyway, should it be deleted? [[User:Jackscott|JackScott]] 10:26, 7 April 2009 (UTC)
:Emptied and redirected to [[X86 Instruction Encoding]]. Functionally equivalent to deletion. - [[User:Combuster|Combuster]] 14:59, 7 April 2009 (UTC)
````

<a id="w002497"></a>

## W002497 — Bare bones

Bron-ID: W002497
Namespace: Main (0)
Revisie: 7134 | Datum: 2009-01-27T23:50:32Z
Bron: https://wiki.osdev.org/Bare_bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Bare_bones&oldid=7134
Doorverwijzing: Bare Bones

````text
#REDIRECT [[Bare Bones]]
````

<a id="w002498"></a>

## W002498 — Talk:Bare bones

Bron-ID: W002498
Namespace: Talk (1)
Revisie: 7136 | Datum: 2009-01-27T23:50:32Z
Bron: https://wiki.osdev.org/Talk:Bare_bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABare_bones&oldid=7136
Doorverwijzing: Talk:Bare Bones

````text
#REDIRECT [[Talk:Bare Bones]]
````

<a id="w002499"></a>

## W002499 — Memory barriers

Bron-ID: W002499
Namespace: Main (0)
Revisie: 22592 | Datum: 2018-08-22T04:05:59Z
Bron: https://wiki.osdev.org/Memory_barriers
Revisie-URL: https://wiki.osdev.org/index.php?title=Memory_barriers&oldid=22592

````text
Memory barriers ensure ordering of load/store commands.

Usually CPUs and compilers reorder these instructions for more efficient execution. Lock-free algorithms often require that perceived execution order be strict, that's when memory barriers or memory fences come in handy.

More detailed description in following articles:

Memory Ordering in Modern Microprocesors, [http://www.linuxjournal.com/article/8211 Part 1] and [http://www.linuxjournal.com/article/8212 Part 2]

Ridiculous Fish's [http://ridiculousfish.com/blog/posts/barrier.html controversial article] about using and abusing memory barriers. (and [http://www.thinkingparallel.com/2007/02/19/please-dont-rely-on-memory-barriers-for-synchronization/ some bashing of this article] just for a different POV)

[http://www.gelato.unsw.edu.au/lxr/source/Documentation/memory-barriers.txt Documentation with explanations on what memory barriers are used in linux kernel and why.]

{{stub}}

[[Category:Synchronization]]
````

<a id="w002502"></a>

## W002502 — User:Earlz

Bron-ID: W002502
Namespace: User (2)
Revisie: 9458 | Datum: 2009-12-20T14:27:40Z
Bron: https://wiki.osdev.org/User:Earlz
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AEarlz&oldid=9458

````text
====Who am I?====
I go by many aliases on the interwebs... The top among them being hckr83 and earlz. My real name is Jordan Earls. 
I have no formal programming training. I am working at Fostech Solutions however, where I am a lead programmer. I have been programming since I was 13, starting with Dark Basic Pro. I am now 18. So, I have been programming a while... 
My spectrum of language knowledge includes

* C (my favorite)
* PHP (second favorite)
* Assembly (x86, intel syntax)
* C++ (everything but heavy templates.)
* Java (only used cause it's what the AP test requires)
* Hex (my EPBP project doesn't have an assembler yet :D )
* Ruby (I'm still learning, but it's an awesome language, my next Project: System Ruby for systems programming)

====My Projects====
This is a list of the projects I've worked on so far(or am still working on) that amounted to anything(in chronological order):

* Mplayer 3.1a. My first project was mplayer 1.0, and it was made in about 2004. This version, 3.1a, was made around 2005. It had a half decent GUI and could play single songs and repeat them and such. It had no playlist support. [http://earlz.biz.tm/old_site/archives/mplayer31.zip mplayer31.zip]

* X-Explorer/X-CMD 0.5. This never really got anywhere. It was my first experience at making a command line and creating an interpreter. The interpreter even had a port of mplayer in it.. it was a horrible language though. [http://earlz.biz.tm/old_site/archives/xcmd05.zip xcmd05.zip]

* JuleOS. A 16bit OS made in Turbo C. It has horrible coding cause I was learning C while I made it. It's at [http://earlz.biz.tm/old_site/archives/JuleOS16.zip JuleOS16.zip]

* JouleOS. A 32bit OS made in gcc. It wasn't a very good OS but it did make it to multitasking and I think it read from a floppy disk once.. [http://earlz.biz.tm/old_site/last_jouleos.tar.gz last_jouleos.tar.gz]

* Open86. An attempt at a PC emulator.. It never got much further than emulating the basic opcodes and being able to display the ascii character set on the video screen. [http://sourceforge.net/projects/open86 open86 @ SF.net] This was then taken over by [[User:Alboin|alboin]] but then it died.. 

* Robot AI. This was a little attempt at instead of trying to create an AI human-like, just eliminate all the ones that aren't. It didn't get very far.. [http://code.google.com/p/robot-ai/ Robot AI @ google-code] 

* SecOS. This was a little 512 byte bootloader OS. It got pretty far for so few bytes, it had a basic command line and it had a very basic filesystem and could read files and execute files. [http://earlz.biz.tm/old_site/secos/SecOS_v1.0_all.zip SecOS_v1.0_all.zip] Good luck with this one, I think there was like 5 bytes of space left

* Niche. A little CMS thing that was suppose to let you develop your own plugins so that you could edit your site with ease and with basically no installation. It got to beta, but really it was a lost cause. [http://sourceforge.net/projects/niche/ niche @ SF.net]

*x86Lib. This actually got half-complete. It is a library for emulating an 8086 CPU. It emulates every opcode, and possibly will be used in some academic software later. [http://sourceforge.net/projects/x86lib x86lib @ SF.net]

*EPBP. This is a little bytecode project I had going on that was meaning to take down Java (because I hate it) but never got too far.. did get a few simple programs going in it though, and a basic module system [http://code.google.com/p/epbp/ EPBP @ Google-Code]

*AlloyOS. This little OS got pretty far, got into ring 0 and had a few very simple devices working.. svn://earlz.biz.tm/alloyos (no website is up for it anymore) 

and.. I think that's it for right now.. not counting the CERTSS project that I'm involved with at work(I can't say much about it anyway though)..

====My Pages====

[[Using the RTC]]

[[OpenBSD's bootloader]] --WIP

[[Getting to Ring 3]] --WIP

[[Linker Scripts]] --have I started yet?


====The TSS====
The TSS can be used for multitasking, though it is recommended to use software multitasking for these reasons:
* Software task switching is faster(usually)
* When you port your OS to a different CPU, it won't have the TSS, so you'll have to implement software task switching anyway
* x86 64bit mode does not allow you to use the TSS for task switching.
Since we will be using the software multitasking approach, the TSS will contain a lot of junk we don't need. Here is the structure of the TSS
<pre>
// A struct describing a Task State Segment.


typedef struct tss_entry_struct tss_entry_t;
</pre>
As you can see.. a lot of wasted crap you don't need. But, intel demands it be used, so... 
Basically what we want to do to setup this TSS structure is give it an initial esp0 stack and setup the segments to point to our kernel segments, and really that's it.. so we can do something like this:
<pre>
/**Ok, this is going to be hackish, but we will salvage the gdt_entry_bits struct to form our TSS descriptor
So some of these names of the fields will actually be different.. maybe I'll fix this later..**/
tss_entry_t tss_entry;

void write_tss(gdt_entry_bits *g)
{
   // Firstly, let's compute the base and limit of our entry into the GDT.
   uint32_t base = (uint32_t) &tss_entry;
   uint32_t limit = base + sizeof(tss_entry);

   // Now, add our TSS descriptor's address to the GDT.
   g->base_low=base&0xFFFFFF; //isolate bottom 24 bits
   g->accessed=1; //This indicates it's a TSS and not a LDT. This is a changed meaning
   g->read_write=0; //This indicates if the TSS is busy or not. 0 for not busy
   g->conforming_expand_down=0; //always 0 for TSS
   g->code=1; //For TSS this is 1 for 32bit usage, or 0 for 16bit.
   g->always_1=0; //indicate it is a TSS
   g->DPL=3; //same meaning
   g->present=1; //same meaning
   g->limit_high=(limit&0xF0000)>>16; //isolate top nibble
   g->available=0;
   g->always_0=0; //same thing
   g->big=0; //should leave zero according to manuals. No effect
   g->gran=0; //so that our computed GDT limit is in bytes, not pages
   g->base_high=(base&0xFF000000)>>24; //isolate top byte.

   // Ensure the TSS is initially zero'd.
   memset(&tss_entry, 0, sizeof(tss_entry));

   //note that CS is loaded from the IDT entry and should be the regular kernel code segment
}
</pre>

Now, I know you may spend a while looking at that atrocious code..but I do believe it works. Oh, and here is our flush_tss function: (in yasm/nasm syntax)
<pre>
GLOBAL tss_flush   ; Allows our C code to call tss_flush().
tss_flush:

</pre>

Ok, so now we are just about ready to do some ring 3 fun stuff!!

====Entering Ring 3====
Ok, the x86 is really a tricky CPU. The only way to get to ring 3 is to fool the processor into thinking it was already in ring 3 to start with. We effectively do this using an iret. I'll give you a simple example on how to execute something as ring 3:(yasm/nasm syntax)
<pre>
GLOBAL _jump_usermode ;you may need to remove this _ to work right.. 
EXTERN _test_user_function
_jump_usermode:
     mov ax,0x23
     mov ds,ax
     mov es,ax 
     mov fs,ax 
     mov gs,ax ;we don't need to worry about SS. it's handled by iret                
     mov eax,esp
     push 0x23 ;user data segment with bottom 2 bits set for ring 3
     push eax ;push our current stack just for the heck of it
     pushf
     push 0x1B; ;user data segment with bottom 2 bits set for ring 3
     push _test_user_function ;may need to remove the _ for this to work right 
     iret ;is there really a different way to make this?
;end
</pre>
Now then, this will call the C function test_user_function and it will be operating in user mode! There is no easy way of getting back to ring 0(excluding IRQs) except for by setting up a task switching system, which you really should have in place to properly appreciate ring 3 in the first place.. But if you would like to test things out in user mode, just have the test_user_function execute a cli or other privileged instruction and you'll be pleased by a GPF. I won't give you source examples on implementing this into your task switching system, as these vary a lot by operating system.
````

<a id="w002503"></a>

## W002503 — RTC

Bron-ID: W002503
Namespace: Main (0)
Revisie: 30895 | Datum: 2026-07-15T08:21:49Z
Bron: https://wiki.osdev.org/RTC
Revisie-URL: https://wiki.osdev.org/index.php?title=RTC&oldid=30895

````text
==Introduction==

The '''Real Time Clock''' (RTC) is the chip that keeps your computer's clock up-to-date, even when it's powered down. It provides a low-precision (seconds) clock information that can be used to set the system wall clock time at the startup. Within the chip is also the 64 bytes of [[CMOS]] RAM.

Typically, an OS will use the [[APIC]] or [[PIT]] for timing purposes and will depend on the RTC only to sync its clock at boot time. However, the RTC is capable of being used for timing purposes as well, however the fastest reliable rate it can be used for is 8 kHz.

If you simply want information about reading the date/time from the RTC, then please see the [[CMOS#Getting Current Date and Time from RTC|CMOS]] article. The rest of this article covers the use of RTC interrupts.

==Capabilities==
The RTC is capable of multiple frequencies. The base frequency is pre-programmed at 32.768 kHz. It is possible to change
this value, but this is the only base frequency that will keep proper time. For this reason, it is strongly recommended
that you NOT change the base frequency. The chip also has a "divider" register that will generate other frequencies from
the base frequency. The output (interrupt) divider frequency is by default set so that there is an interrupt rate of 1024 Hz.
If you need an interrupt frequency other than 1024 Hz, the RTC can theoretically generate 15 interrupt rates between 2 Hz
and 32768 Hz. On most machines however, the RTC interrupt rate can not go higher than 8 kHz.

==Programming the RTC==
RTC interrupts are disabled by default. Enabling RTC interrupts will periodically generate an interrupt on IRQ 8.

===Avoiding NMI and Other Interrupts While Programming===
When programming the RTC, it is '''extremely important''' that the [[NMI]] (non-maskable-interrupt) and other interrupts are disabled.
This is because if an interrupt happens, the RTC may be left in an "undefined" (non functional) state.
This would usually not be too big a deal, except for two things. The RTC is never initialized by BIOS, and it is backed
up with a battery. So even a cold reboot may not be enough to get the RTC out of an undefined state!
See the [[NMI]] page for more information about disabling and enabling it, and the effects of it.

===Setting the Registers===
The 2 IO ports used for the RTC and [[CMOS]] are 0x70 and 0x71. Port 0x70 is used to specify an index or "register number",
and to disable NMI. Port 0x71 is used to read or write from/to that byte of CMOS configuration space. Only three bytes of
CMOS RAM are used to control the RTC periodic interrupt function. They are called RTC Status Register A, B, and C. They are
at offset 0xA, 0xB, and 0xC in the CMOS RAM. To write 0x20 to Status Register A you would do this:

<syntaxhighlight lang="C">
disable_ints();		// important that no interrupts happen (perform a CLI)
outportb(0x70, 0x8A);	// select Status Register A, and disable NMI (by setting the 0x80 bit)
outportb(0x71, 0x20);	// write to CMOS/RTC RAM
enable_ints();		// (perform an STI) and reenable NMI if you wish
</syntaxhighlight>

Other bytes of the CMOS RAM are used by the RTC for other functions, or by the [[BIOS]] and other such services.

===IRQ Danger===
Since the [[IRQ]] number is 8, it has a lower priority in the [[PIC]] than the IRQs with lower numbers. While those other interrupts are being
handled (until your OS sends an [[PIC#End_of_Interrupt|EOI]] and STI), your OS will not receive any clock ticks. Any IRQ handlers that depend on clock
ticks may fail for that reason, because an IRQ of higher number will not preempt an IRQ of lower number.

===Turning on IRQ 8===
To turn on the periodic interrupt, all you have to do is:

<syntaxhighlight lang="C">
disable_ints();			// disable interrupts
outportb(0x70, 0x8B);		// select register B, and disable NMI
char prev=inportb(0x71);	// read the current value of register B
outportb(0x70, 0x8B);		// set the index again (a read will reset the index to register D)
outportb(0x71, prev | 0x40);	// write the previous value ORed with 0x40. This turns on bit 6 of register B
enable_ints();
</syntaxhighlight>

This will turn on the IRQ with the default 1024 Hz rate. Be sure that you install the IRQ handler before you enable the RTC
IRQ. The interrupt will happen almost immediately.

===Changing Interrupt Rate===
Changing the output divider changes the interrupt rate, ''without'' interfering with the RTC's ability to keep proper time.
The lower 4 bits of register A is the divider value. The default is 0110b, or 6. The setting must be a value from 3 to 15, or a value of 0 (which disables the interrupt). To calculate a new frequency:

<syntaxhighlight lang="C">
frequency =  32768 >> (rate-1);
</syntaxhighlight>

"Rate" is the divider setting. If you select a rate of 1 or 2, the RTC will have problems and "roll over" so that it
generates interrupts every 0.81 ms and 3.91 ms, instead of the expected period of 61.0 us or 30.5 us. So, the fastest
rate you can select is 3. This will generate interrupts at 8 kHz or every 122 us. To change the rate:

<syntaxhighlight lang="C">
rate &= 0x0F;			// rate must be above 2 and not over 15
disable_ints();
outportb(0x70, 0x8A);		// set index to register A, disable NMI
char prev=inportb(0x71);	// get initial value of register A
outportb(0x70, 0x8A);		// reset index to A
outportb(0x71, (prev & 0xF0) | rate); //write only our rate to A. Note, rate is the bottom 4 bits.
enable_ints();
</syntaxhighlight>

===Interrupts and Register C===
It is important to know that upon a IRQ 8, Status Register C will contain a bitmask telling which interrupt happened.
The RTC is capable of producing a periodic interrupt (what this article describes), an update ended interrupt, and an alarm interrupt. If you are only using the RTC as a simple timer this is not important.
'''What is important is that if register C is not read after an IRQ 8, then the interrupt will not happen again.''' So, even if you don't care about what type of interrupt
it is, just attach this code to the bottom of your IRQ handler to be sure you get another interrupt. If you're using Bochs, it is also recommended to read Status Register C after initialising the RTC, to make sure any RTC interrupts that were pending before/while the RTC was initialised are acknowledged (not doing this has lead to the RTC timer not sending any interrupts after rebooting in at least one case, observed on Bochs 2.6.6, [http://forum.osdev.org/viewtopic.php?f=1&t=30091 see this thread]).

<syntaxhighlight lang="C">
outportb(0x70, 0x0C);	// select register C
inportb(0x71);		// just throw away contents
</syntaxhighlight> 

==Problems?==
Some RTC timer code may not work on some real machines. I have only found about one out of 5 computers that does not work right. The observed problem is a timer tick happened about once
every second. I'm not sure why this is, and am trying to find a solution. The machine I observed
this on was a 2005 or so Dell (which also has issues USB booting...)


==See Also==
*[[CMOS]]
===External Links===
* [http://www.compuphase.com/int70.txt Periodic Interrupts with the Real Time Clock]
* [http://www.ousob.com/ng/interrupts_and_ports/ng9116b.php CMOS Ram Data Register]
* [https://web.archive.org/web/20150514074601/http://www.nondot.org/sabre/os/files/MiscHW/CMOSTimer.html Using the 1024 HZ Timer Interrupt]
* [https://web.archive.org/web/20150514082645/http://www.nondot.org/sabre/os/files/MiscHW/RealtimeClockFAQ.txt Real Time Clock / CMOS Setup Reference]
* [http://bos.asmhackers.net/docs/timer/docs/cmos.pdf Accessing the CMOS Chip]

[[Category:Common Devices]]
[[Category:Interrupts]]
[[Category:Time]]
[[Category:Timers]]
````

<a id="w002504"></a>

## W002504 — Using the RTC

Bron-ID: W002504
Namespace: Main (0)
Revisie: 7218 | Datum: 2009-02-13T08:17:08Z
Bron: https://wiki.osdev.org/Using_the_RTC
Revisie-URL: https://wiki.osdev.org/index.php?title=Using_the_RTC&oldid=7218
Doorverwijzing: RTC

````text
#REDIRECT [[RTC]]
````

<a id="w002505"></a>

## W002505 — How do I set a VGA mode without the BIOS

Bron-ID: W002505
Namespace: Main (0)
Revisie: 7236 | Datum: 2009-02-14T15:15:17Z
Bron: https://wiki.osdev.org/How_do_I_set_a_VGA_mode_without_the_BIOS
Revisie-URL: https://wiki.osdev.org/index.php?title=How_do_I_set_a_VGA_mode_without_the_BIOS&oldid=7236
Doorverwijzing: VGA Resources

````text
#REDIRECT [[VGA Resources]]
[[Category:FAQ]]
````

<a id="w002506"></a>

## W002506 — Talk:Wish List/Archive

Bron-ID: W002506
Namespace: Talk (1)
Revisie: 16356 | Datum: 2014-05-09T12:46:01Z
Bron: https://wiki.osdev.org/Talk:Wish_List/Archive
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AWish_List%2FArchive&oldid=16356

````text
A collection of archived [[Talk:Wish List|discussions]] for the [[Wish List]]

= 2012 =

== AML ==

What is meant by "AML reference"?  I expanded the AML article a bit a while back but it is unclear exactly how much detail the requestor wants on this.  Certainly we will not provide a reference of every AML command as they are freely available in the linked ACPI specification and doing so would be akin to documenting every x86/arm/mips etc machine code instruction when there are already references available elsewhere.  Unless a more specific request is made I suggest we remove this request.  [[User:Jnc100|John]] 06:07, 5 July 2012 (CDT)

:Probably best to ask [[User Talk:Bellezzasolo]] directly. -- [[User:Solar|Solar]] 08:03, 5 July 2012 (CDT)




= 2008 =

==Memory Management Wanted==
How are we doing the MM page allocations and such? Currently, we've [[Page_Frame_Allocation]]. However, if more info is put into this article for each algorithm and such, it would grow too large and broad. Moreover, what about specific methods? (eg. Buddy system.) Any ideas on how this should be structured?--[[User:Alboin|Alboin]] 19:52, 12 May 2008 (CDT)

I'd go for a stub-and-read-more approach. Pretty much like [[What order should I make things in]]. You can get the templates from there too - [[User:Combuster|Combuster]] 04:32, 13 May 2008 (CDT)




= 2007 =

==Theory Vs Code==
Articles should be more focused on theory rather than code. Code is welcomed, and is useful for backing up an idea, but too much code leads to people copying and pasting and then give up when it comes to having to think for themselves. Theory is welcome more since it goes into detail about how and why, and is much easier to implement the idea into your OS rather than looking at a bunch of code and try to figure out what the heck they're trying to do. Also, it helps people who are not using Assembly/C/C++. - [[User:MessiahAndrw|MessiahAndrw]] 

Agreed. - [[User:Combuster|Combuster]] 06:33, 14 December 2007 (CST)

== Pages For Review ==
The wish list is a useful place to store ideas and should IMO be placed on the main page where now the PFR message lives. Until then, I'm looking for a check that the PFR tag here can be removed - [[User:Combuster|Combuster]] 16:49, 27 August 2007 (CDT)
:Looks fine here. --Alboin
::Placed on main page. - [[User:Yayyak|Yayyak]] 03:39, 20 November 2007 (CST)
== Section: Under Construction ==
IMO Pages Under Construction should be removed (and optionally put in a category, we have templates for that) - [[User:Combuster|Combuster]] 11:13, 8 March 2007 (CST)
:Went ahead for getting no comments - The WIP list was outdated anyway. If you feel offended revert and post comments here. - [[User:Combuster|Combuster]] 03:52, 26 August 2007 (CDT)

==Sections in wanted pages==
I've put this section into little subsections, depending on how much we need them. While I don't think anybody will argue with that as an idea (somebody will prove me wrong there :P), perhaps my categorisation of things will be argued on. Go for it. I placed them depending on how easy they are to write, and how much we lack in that area. [[User:Yayyak|Yayyak]] 20:26, 15 September 2007 (CDT)
:I moved the HPET topic to the lowest-priority category since it isn't required knowledge for almost all system in current use. Other than that, the ordering looks decent. - [[User:Combuster|Combuster]] 17:18, 16 September 2007 (CDT)

==C++ Runtime Support==
This is listed under the Urgent list, but seems to be covered in the newlib part of [[OS Specific Toolchain]]. Should it be removed from the urgent list, or do people still want a separate page? -[[User:AJ|AJ]] 10:39, 20 November 2007 (CST)
:My vote is that we can remove it. Also, there is this: 'More on C++: what support is needed to fully use C++ in a standalone environment? Why? Example code?'. Can this be removed from the non-urgent section, or does this need a proper article on it? - [[User:Yayyak|Yayyak]] 15:43, 20 November 2007 (CST)
::I've got rid of the one in 'Urgent' at the moment - anyone who doesn't like this can always re-instate it! I have left the 'fully supporting c++ in a standalone environment', though. My thought behind this is that although we have articles about porting newlib and libsupc++, we don't currently have anything showing ''how'' things like exceptions and rtti are actually implemented. -[[User:AJ|AJ]] 10:36, 22 November 2007 (CST)
````

<a id="w002507"></a>

## W002507 — User:Gh975223

Bron-ID: W002507
Namespace: User (2)
Revisie: 9711 | Datum: 2010-02-05T13:45:40Z
Bron: https://wiki.osdev.org/User:Gh975223
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGh975223&oldid=9711

````text
Hi I am Graeme Brett Houston BSc. 

I currently work for a large Scottish Organisation, as a Junior System Operator. 

My hobbies excluding collecting DVDs & collecting alternative computer platforms, such as SGI's, Sun Sparc's, Acorn RiscPc, etc, and investigating all the system that go with them.

I am currently writing a small simple Operating System called Aiolios, with the intention that it will be multiplatform.
The Name Aiolios is used after the Greek God of the Winds, signifying what the intended end goals are: 

1. Fast, agile, sleek, Art Nouveau of the computing world


I am well versed in C and reasonably in C++, with tastes of Java & Javascript/VBScript. I would say that i am proficient in VBA under MS Excel and a taste of Assembler (only inline and were necessary) 


My web site is located at: http://www.btinternet.com/~graemebrett.houston/ 

I enjoy traveling around the world on holiday, places I have visited: 

Maderia (Funchal) - March 2001

Turkey (Fethiye, Telos, Hierapolis) - August 2002 

Cyprus (Paphos) - Novemeber 2003
 
Jordan (Amman, Petra) - Novemeber 2003 

Italy (Rome) - July 2004 

Thailand (Bangkok, Cha Am, Hua Hin) - September 2004 

Czech Republic (Pargue) - Novemeber 2004 

France (Paris) - March 2005 

Peru (Miraflora, Lima, Cusco, Yutay) - Novemeber 2005 

China (Beijing) - Novemeber 2005 

Poland (Krakow, returning via Prague, Dresden, Frankfort, Hahn, London thanks to Ryan Air not flying due to a bit of fog for providing alternative routes) - Novemeber 2006 

Cambodia (Siem Reap, Angkor Wat) June 2008

Kenya (Mombasa, Gede) May 2008

Thailand (Bangkok, Cha Am, Hue Hin)June, July 2008

The Gambia (Kotu) March 2009

Senegal March 2009

Egypt (Cairo, Luxor, Aswan, Abu Simbel, Hurghada, Dehab) October 2009

Jordan (Aqaba, Petra, Wadi Rum) October 2009

I have also worked in the Netherlands (Tilburg) on various contracts between 1999 and 2000, so that adds Tilburg & Breda.
````

<a id="w002508"></a>

## W002508 — D

Bron-ID: W002508
Namespace: Main (0)
Revisie: 30986 | Datum: 2026-09-28T14:34:58Z
Bron: https://wiki.osdev.org/D
Revisie-URL: https://wiki.osdev.org/index.php?title=D&oldid=30986

````text
{{Stub}}

D is a programming language, originally created by Walter Bright (from Digital Mars). Originally, the idea was to fix up some parts of C++, but in the meantime, some parts have completely changed. Even though the syntax of the D programming language is similar to that of C++, it is not a variant.

The D language is a language suitable for OS development, as some people on the forums have already stated. It might, however, require some extra changes. There is also a tutorial available on [[GDC Cross-Compiler|how to build a D cross compiler]] (using GDC).

== Limitations ==
D is designed to be a systems language and is perfectly capable of running directly on the hardware with nothing more than an stub in assembly to jump to your main function. However, a lot of the language's features besides the most basic are tightly integrated into the standard library. Attempting to do string operations, slice arrays, hash arrays, use smart pointers, type checking on types not known at compile time, and a myriad of other features will lead to a number of missing symbols at link time that aren't well documented at all. Attempting to use reference the [http://www.digitalmars.com/d/2.0/phobos/phobos.html Phobos] source and port the required functions will soon make you realize how interdependent functions are one another and before you know it you've ported the majority of Phobos in to your kernel (which by the way, references the C standard library a lot).

So in reflection, unless you want to port a D standard library into your kernel from the beginning, the only advantage of D over C or C++ for kernel programming is the advance templating and conditional compile features.

Of course, you could think about writing a 'stub' kernel in D, and then linking it with existing C libraries.

==See Also==
===External Links===
* [[GDC Cross-Compiler|How to build a D cross-compiler]]
* [[D bare bones|D kernel bare bones]]
* [http://forum.osdev.org/viewtopic.php?f=15&t=18914 OS in D language (forums)]

[[Category:Languages]]
[[Category:D]]
[[de:D]]
````

<a id="w002509"></a>

## W002509 — Template:Maybe

Bron-ID: W002509
Namespace: Template (10)
Revisie: 10198 | Datum: 2010-05-01T14:29:10Z
Bron: https://wiki.osdev.org/Template:Maybe
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AMaybe&oldid=10198

````text
style="background: #ffff99" | {{{1|Maybe}}}
````

<a id="w002510"></a>

## W002510 — Talk:IA32 Architecture Family

Bron-ID: W002510
Namespace: Talk (1)
Revisie: 12035 | Datum: 2011-09-22T08:29:44Z
Bron: https://wiki.osdev.org/Talk:IA32_Architecture_Family
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AIA32_Architecture_Family&oldid=12035

````text
I'm seriously impressed, when I saw all the edits in the recent changes spot I assumed "spambot" or "n00b", but great information! --[[User:Troy martin|Troy Martin]] 15:38, 3 March 2009 (UTC)
: That's what you get when two ''different'' people are editing the same article - it won't merge multiple edits. Good work, still :) - [[User:Combuster|Combuster]] 16:52, 3 March 2009 (UTC)
::It's called "Extreme Wiki Editing" :P [[User:Jackscott|JackScott]] 23:55, 3 March 2009 (UTC)
I would like to rename this article to "x86 Architecture Family" or something like that to avoid confusion as it also contains information about non-Intel CPUs. However, I don't know how this will affect the wiki start page template thingy.--[[User:Love4boobies|Love4boobies]] 05:49, 10 March 2011 (UTC)

== Marketing... ==

Currently these tables list marketing names, but marketing names are very confusing because the same CPU with the same features (but possibly, different clock speeds and different cache sizes) are typically marketed under several different names (e.g. Celeron vs. Pentium 4 vs. Xeon); and sometimes entirely different CPUs use the same name (Xeon vs. Xeon vs. Xeon).

Also, there are at least 2 cases where a CPU changes companies. The first example is National Semiconductor's Geode, which was taken over by AMD. In this case "NS Geode GX" was followed by "AMD Geode GX1", which was followed by "AMD Geode GX2". Also note that the "AMD Geode LX" is actually an Athlon and has nothing to do with the "NS Geode GX" (except for the northbridge, etc). The second example is the "Centaur" series, which went from WinChip/IDT to VIA.

I'm thinking it might reduce the confusion if these tables listed microarchitectures (e.g. Netburst, and not Celeron/Pentium 4/Xeon), possibly including code names (e.g. "Netburst Willamette", then "Netburst Northwood", then "Netburst Prescott").

Just a thought....

-Brendan

== AMD hyping... ==

Good information overall. I couldn't help but notice, though, that the AMD section is more "excited" about the subject than the Intel section, which isn't very good for the article - because the "success, on-par, cheaper" stuff takes away the space that ''should'' be used for technical information. I'm sorry but it looks like being written by a fanboy, and I wonder if someone with knowledge on technical details could boil it down a bit. I also noticed a reference to "past year" in the AMD section... which is a bit silly since it's probably several years old. ;-) Someone in a mind of fixing this? -- [[User:Solar|Solar]] 13:45, 19 September 2011 (UTC)

: I'm quite partial to AMD myself, but the tone is certainly different, I don't know what to do with it so I'll let someone else decide. --[[User:Brynet-inc|Brynet-inc]] 20:58, 19 September 2011 (UTC)

Thought about it some more, and decided to cut it down to the technical level myself. I doubt ''anybody'' will use ''this'' page as some kind of "buyer's guide". The discussion if an Athlon is / was better than a PIII or not is pretty pointless today. And even if you write about the Phenom vs. the i7, that information will go stale in a year or so. -- [[User:Solar|Solar]] 08:05, 20 September 2011 (UTC)

: I made some other tweaks, hope you don't mind. --[[User:Brynet-inc|Brynet-inc]] 00:20, 22 September 2011 (UTC)

:: Not at all. It feels like we did the article a ton of good. [[User:Solar|Solar]] 08:29, 22 September 2011 (UTC)
````

<a id="w002511"></a>

## W002511 — How to develop on Mac OS X

Bron-ID: W002511
Namespace: Main (0)
Revisie: 7353 | Datum: 2009-03-06T23:36:58Z
Bron: https://wiki.osdev.org/How_to_develop_on_Mac_OS_X
Revisie-URL: https://wiki.osdev.org/index.php?title=How_to_develop_on_Mac_OS_X&oldid=7353
Doorverwijzing: GCC Cross-Compiler

````text
#REDIRECT [[GCC Cross-Compiler]]

[[Category:FAQ]]
````

<a id="w002512"></a>

## W002512 — User:Nathanjphillips

Bron-ID: W002512
Namespace: User (2)
Revisie: 7368 | Datum: 2009-03-14T11:58:58Z
Bron: https://wiki.osdev.org/User:Nathanjphillips
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANathanjphillips&oldid=7368

````text
Author of PenPot operating system. A simple GUI OS written in an OO and partly functional style.

My first article is on doing a [[Stack Trace]].
````

<a id="w002513"></a>

## W002513 — Stack Trace

Bron-ID: W002513
Namespace: Main (0)
Revisie: 27933 | Datum: 2023-07-09T15:04:22Z
Bron: https://wiki.osdev.org/Stack_Trace
Revisie-URL: https://wiki.osdev.org/index.php?title=Stack_Trace&oldid=27933

````text
A stack trace is debugging output, normally sent to a log file or a debug window that shows the hierarchy of callers that called the current function. A stack trace is generated by analysing the stack to find each stack frame. The addresses of the functions called can be retrieved from each stack frame and the names of the functions displayed.

To implement a stack trace you have to know the structure of the stack frames, which is shown in the article [[Stack]] for X86 CDECL.

=== Walking the stack ===
Often a stack trace is written in assembly as it involves finding the current value of the EBP register. 
To write a stack trace routine in a higher-level language you will need to find EBP. 
This can be done by using a small assembly function or inline assembly, or by using  __builtin_frame_address(0) if you use the GCC compiler. 
On some platform (e.g. x86), the compiler does not necessary save the EBP on the stack. For example, for gcc, use the -fno-omit-frame-pointer to make sure that the EBP is saved. Note that omission of the frame pointer merely causes functions to be missed from the backtrace.

The following C++ code shows how (given the existence of a Trace function) this can be used to walk up the stack:

<source lang="cpp">
/* Assume, as is often the case, that EBP is the first thing pushed. If not, we are in trouble. */
struct stackframe {
  struct stackframe* ebp;
  uint32_t eip;
};
void Debug::TraceStackTrace(unsigned int MaxFrames)
{
    struct stackframe *stk;
    asm ("movl %%ebp,%0" : "=r"(stk) ::);
    Trace("Stack trace:\n");
    for(unsigned int frame = 0; stk && frame < MaxFrames; ++frame)
    {
        // Unwind to previous stack frame
        Trace("  0x{0:16}     \n", stk->eip);
        stk = stk->ebp;
    }
}
</source>

Note that the above code and GDB backtracing require a NULL %ebp, to know when to stop. Otherwise the traces will run off into garbage. To account for this, set up a NULL stack frame before you jump to your C entry point:

<source lang="asm">
mov $stack_end, %esp ; Initialize %esp
...
xor %ebp, %ebp       ; Set %ebp to NULL
call kmain           ; According to calling convention, kmain will save %ebp (=NULL) to the stack
</source>

With this, stack tracers will see the NULL %ebp as the end of the trace.

=== Assembly Implementation ===

This assembly implementation for x86 uses the same algorithm as above and similarly relies on a NULL base pointer to be placed near the top of the stack. Rather than print the contents of the stack, however, it builds an array of addresses which can then be resolved into symbol names.

<source lang="asm">
; Walks backwards through the call stack and builds a list of return addresses.
; Args:
;  * Array of 32-bit addresses.
;  * Maximum number of elements in array.
; Return value: The number of addresses stored in the array.
; Calling convention: cdecl
[global walk_stack]
walk_stack:
    ; Create stack frame & save caller's EDI and EBX.
    push ebp
    mov  ebp,       esp
    sub  esp,       8
    mov  [ebp - 4], edi
    mov  [ebp - 8], ebx
    ; Set up local registers.
    xor  eax,       eax         ; EAX = return value (number of stack frames found).
    mov  ebx,       [esp +  8]  ; EBX = old EBP.
    mov  edi,       [esp + 16]  ; Destination array pointer in EDI.
    mov  ecx,       [esp + 20]  ; Maximum array size in ECX.
.walk:
    ; Walk backwards through EBP linked list, storing return addresses in EDI array.
    test ebx,       ebx
    jz   .done
    mov  edx,       [ebx +  4]  ; EDX = previous stack frame's IP.
    mov  ebx,       [ebx +  0]  ; EBX = previous stack frame's BP.
    mov  [edi],     edx         ; Copy IP.
    add  edi,       4
    inc  eax
    loop .walk
.done:
    ; Restore caller's EDI and EBX, leave stack frame & return EAX.
    mov  edi,       [ebp - 4]
    mov  ebx,       [ebp - 8]
    leave
    ret
</source>

=== Resolving Function Names ===
The next step in producing meaningful output from a stack trace is to find the names of the functions containing the addresses found during the stack walk.

When looking up the name of a function you have to find the biggest address smaller than the value you are looking for. This is because the return address saved by the call is the address of the jsr instruction, which will be offset within the function that is making the call.

To get the information you need to lookup function names you will need to either include debugging symbols in your kernel or load the map file created by your linker into the kernel's memory space. The map file shows the addresses of each of your functions. While you could include the entire map file, it is often quite large and inefficiently stored. Not only this but often functions are not listed in the order that they appear in the object file and the format is not amenable to tracing through to find a specific function.

One possible solution is to pre-process your map file to produce a smaller, more useful format for it. You could do this in a way that allows either binary or linear searching for a particular address. See NobleTech's Web site[http://www.nobletech.co.uk/Products/PenPot/Design/Kernel/Debug/FnNameLookup.aspx] for C# code showing a way of reading the map file produced by GNU ld and outputting a binary file that allows more efficient linear searching for symbols. A binary Win32 console application to do the pre-processing is also available for free from that site. C++ code that can be used in your kernel to look up function names in the pre-processed file format is also shown.

[[Category:Debugging]]
[[Category:X86]]
````

<a id="w002514"></a>

## W002514 — Talk:Stack Trace

Bron-ID: W002514
Namespace: Talk (1)
Revisie: 27647 | Datum: 2023-03-01T23:46:55Z
Bron: https://wiki.osdev.org/Talk:Stack_Trace
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AStack_Trace&oldid=27647

````text
Nice article. I think it should say something about the compiler need to output stack frames for all functions, else walking the stack will fail in some way. The assumption is that EBP (the stack frame pointer) is always saved on the stack for a call. An extended stack walking routine could take DWARF or other debugging information into account to determine the last stack frame. - [[User:Walling|Walling]] 19:25, 15 March 2009 (UTC)


I didn't find this code to work, so I've added an implementation in assembly which gets the right values for me. - [[User:Synon|Synon]] 12:08, 7 January 2018 (CST)


The link at the bottom just leads to a 404 page.  Is there a new source for it? - [[User:Techflash|Techflash]] 16:45, 1 Mar 2023 (GMT-7)
````

<a id="w002515"></a>

## W002515 — Virtual Floppy Disk

Bron-ID: W002515
Namespace: Main (0)
Revisie: 7396 | Datum: 2009-03-21T21:17:30Z
Bron: https://wiki.osdev.org/Virtual_Floppy_Disk
Revisie-URL: https://wiki.osdev.org/index.php?title=Virtual_Floppy_Disk&oldid=7396
Doorverwijzing: Virtual Floppy Drive

````text
#REDIRECT [[Virtual Floppy Drive]]
````

<a id="w002516"></a>

## W002516 — Talk:Interrupt Vector Table

Bron-ID: W002516
Namespace: Talk (1)
Revisie: 7404 | Datum: 2009-03-23T23:21:12Z
Bron: https://wiki.osdev.org/Talk:Interrupt_Vector_Table
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AInterrupt_Vector_Table&oldid=7404

````text
:''Although it is also called the IDT''
By who?

:''to avoid confusions related with the protected mode Interrupt Descriptor Table.''
I see no way to confuse an array of 256 seg:off pairs with a complex 8*unknown quantity -byte descriptor table.

So what's with this, is it completely by opinion? --[[User:Troy martin|Troy Martin]] 03:50, 22 March 2009 (UTC)

:I agree. I think it creates confusion more than anything. I've never heard of anybody calling it the IDT either. [[User:Jackscott|JackScott]] 03:51, 22 March 2009 (UTC)
::'''Disagree'''. I think the reasoning behind it is not unimportant - IDTR points to either a 32-bit IDT, a 64-bit IDT, or an IVT, depending on operating mode. In that respect, "By who", would be the IDTR; to avoid confusion is that using the IDT affects the IVT by virtue of that IDTR. I do agree that the original text could hardly have been ''more'' confusing. - [[User:Combuster|Combuster]] 23:21, 23 March 2009 (UTC)
````

<a id="w002517"></a>

## W002517 — User:Troy Martin

Bron-ID: W002517
Namespace: User (2)
Revisie: 7400 | Datum: 2009-03-22T03:58:57Z
Bron: https://wiki.osdev.org/User:Troy_Martin
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATroy_Martin&oldid=7400
Doorverwijzing: User:Troy martin

````text
#redirect [[User:Troy martin]]
````

<a id="w002518"></a>

## W002518 — Talk:Visual Studio

Bron-ID: W002518
Namespace: Talk (1)
Revisie: 17865 | Datum: 2015-04-07T13:39:08Z
Bron: https://wiki.osdev.org/Talk:Visual_Studio
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AVisual_Studio&oldid=17865

````text
There is a difference between the Visual Studio IDE and the Visual C++ compiler.  This page is labeled 'Visual Studio' but jumps directly into the limitations of the Visual C++ compiler which is a different product.  The VS IDE will work with (AFAIK) every Windows or MS DOS based compiler.  The IDE ships as part of VC++ and as part of other MS Visual compiler lines (C#, VB.net, J++, etc) in addition to the the express edition of compilers that can be downloaded for free.
[[User:Tantrikwizard|Tantrikwizard]]

:Not quite - the article discusses how to use the combination of the IDE and the compiler. I.e. visual studio as a whole. Also, what exactly are you aiming at here? Saying something's bad doesn't help when you don't have an alternative. - [[User:Combuster|Combuster]] 10:44, 26 March 2009 (UTC)

Maybe this information is important to report.
Visual Studio Express Edition has limited functionality compared to the others editions, especially in extension functions
The good new is that since few month Microsoft provide the Visual Studio 2013 Community Edition. This version is equivalent to the professional edition and is for free.
--[[User:Gedd|Gedd]] 06:16, 26 February 2015 (CST)

Microsoft says that the Express edition will disapear with 2015 version--[[User:Gedd|Gedd]] 08:38, 7 April 2015 (CDT)
````

<a id="w002519"></a>

## W002519 — User:01000101/optlib/

Bron-ID: W002519
Namespace: User (2)
Revisie: 28647 | Datum: 2024-02-29T17:37:33Z
Bron: https://wiki.osdev.org/User:01000101/optlib/
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3A01000101%2Foptlib%2F&oldid=28647

````text
== Intro ==
Welcome!  This is the information and source for the OptLib (Optimized Library).  I took great care in making the source as portable (to different OS's) as possible, this code should also compile (using GCC) as-is into 32-bit or 64-bit objects.  There is no support for non-x86(_64) architectures (yet).  This code does not follow any particular standards (as you will see added functions that aren't present in the "standard C libraries"), but I try not to make it "incompatible" with existing setups.  

This article is orginized into code blocks.  Each header file has a set of C functions.  Then those C functions are broken into different sub-versions (SSE/non-SSE).  Each header can have either functions (full functions) or wrappers (SSE instruction with a C interface).

There are a few "glue" components that need to be present for these to work.  "size_t", "bool", "true/false", "uintx_t", and "asm (volatile)" are the core ones.  I'd usually set those up as:

 typedef unsigned char  uint8_t;
 typedef unsigned short int  uint16_t;
 typedef unsigned int  uint32_t;
 typedef unsigned long  size_t;
 typedef uint8_t bool;
 #define true 1
 #define false 0
 #define asm  __asm__
 #define volatile  __volatile__

You will also need to enable the [[FPU]] and unmask a few extra bits ([[SSE]]).  There will be a few C wrappers for a few x86 FPU/SSE instructions as they are powerful on their own and it would be more of a waste to try and emulate their functionality. 

The "result" area in some functions give my recorded timings using the RDTSC instruction with a serializing CPUID instrucion before it to make sure it's as accurate as possible.  Also, these timings are all recorded in one run (both SSE and non-SSE) so the results of each test are able to be compared (as opposed by different runs).  Interrupts are disabled at the time of the tests to remove the possibility of timing a full interrupt handler being processed. 

== License ==
This code is released into the Public Domain or into whatever state of pure freedom you can think of.  With that said, I take no responsibility for damages done or suitability of this code for any purpose.  I really only ask that you give back to the article if you gained something from it to begin with.

== string.h ==
string.h is part of the standard C library. It contains the definition of several functions to manipulate C strings and arrays. See more here: [http://www.cplusplus.com/reference/clibrary/cstring/]
=== memclr() ===
memclr() zeros "m_count" bytes from "m_start".
==== SSE2 version ====
This is an SSE2 version due to the MOVDQA (Aligned Double-Quadword Move).  In the memclr_sse2 function, we aligned the source memory address (if needed) and copy 64-byte chunks per iteration, then cleanup the left-over bytes.   

 static const void * memclr_sse2(const void * const m_start, const size_t m_count)
 {
     // "i" is our counter of how many bytes we've cleared
     size_t i;
 
     // find out if "m_start" is aligned on a SSE_XMM_SIZE boundary
     if((size_t)m_start & (SSE_XMM_SIZE - 1))
     {
         i = 0;
 
         // we need to clear byte-by-byte until "m_start" is aligned on an SSE_XMM_SIZE boundary
         // ... and lets make sure we don't copy 'too' many bytes (i < m_count)
         while(((size_t)m_start + i) & (SSE_XMM_SIZE - 1) && i < m_count)
         {
             asm("stosb;" :: "D"((size_t)m_start + i), "a"(0));
             i++;
         }
     }
     else
     {
         // if "m_start" was aligned, set our count to 0
         i = 0;
     }
  
     asm volatile("pxor %%xmm0,%%xmm0"::); // zero out XMM0
     // clear 64-byte chunks of memory (4 16-byte operations)
     for(; i + 64 <= m_count; i += 64)
     {
         asm volatile(" movdqa %%xmm0, 0(%0);	"    // move 16 bytes from XMM0 to %0 + 0
                      " movdqa %%xmm0, 16(%0);	"
                      " movdqa %%xmm0, 32(%0);	"
                      " movdqa %%xmm0, 48(%0);	"
                      :: "r"((size_t)m_start + i));
     }
  
     // copy the remaining bytes (if any)
     asm(" rep stosb; " :: "a"((size_t)(0)), "D"(((size_t)m_start) + i), "c"(m_count - i));
 
     // "i" will contain the total amount of bytes that were actually transfered
     i += m_count - i;
 
     // we return "m_start" + the amount of bytes that were transfered
     return (void *)(((size_t)m_start) + i);
 }

==== non-SSE (standard) version ====
memclr_std() is the non-SSE version of memclr().  We clear all the D-Words (4-bytes) at a time, and then clear the remaining bytes (<4 bytes) using the STOSx (Store String) instruction combined with the REP prefix. 

 static const void * memclr_std(const void * const m_start, const size_t m_count)
 {
     asm("rep stosl;"::"a"(0),"D"((size_t)m_start),"c"(m_count / 4));
     asm("rep stosb;"::"a"(0),"D"(((size_t)m_start) + ((m_count / 4) * 4)),"c"(m_count - ((m_count / 4) * 4)));
 
     return (void *)((size_t)m_start + m_count);
 }

==== function stub ====
This is the "memclr" that everyone will call to clear memory.  This function calls other sub-functions if various features are available on the CPU.  In this case, if the CPU can use SSE2, we use the SSE2 memclr and if not, we use the standard non-SSE memclr.  This way, older CPU's can still use this function, but newer CPUs with SSE support get to use a more optimized version.

 const void * memclr(const void * const mem, const size_t count)
 {
     // return if there is no count
     if(!count){ return mem; }
     // if the CPU is SSE2 capable, we favor the SSE2 version of this function first 
     else if(cpuid_features.SSE2)
     {
         return memclr_sse2(mem, count);
     }
     // if the CPU didn't support SSE2, then we use the generic standard function
     return memclr_std(mem, count);
 }

==== results ====
10 sets of 100 16-byte aligned 4KB clears [memclr(buf, 4096)]:
{| border="2"
|Software
|non-SSE
|SSE2
|-
|BOCHS
|1055.00
|736.00
|-
|QEMU
|12953.13
|6362.38
|-
|VirtualBox OSE
|43785.66
|19031.96
|}

10 sets of 100 16-byte unaligned 4KB clears [memclr(buf+1, 4096)]:
{| border="2"
|Software
|non-SSE
|SSE2
|-
|BOCHS
|1055.00
|881.00
|-
|QEMU
|36143.71
|11549.49
|-
|VirtualBox OSE
|64734.35
|23246.49
|}

=== strlen() ===
==== SSE4.2 version ====
Experimental: UnTested!
 static size_t strlen_sse4_2(const char * const str)
 {
    size_t index;
 
    asm(" mov $-16, %0;                      "
        " pxor %%xmm0, %%xmm0;               "
        ".strlen_4_2_start:                  "
        " add $16, %0;                       "
        " pcmpistri $0x08, (%0,%1), %%xmm0;  "
        " jnz .strlen_4_2_start;             "
        " add %2, %0;                        "
        :"=a"(index):"d"((size_t)str),"c"((size_t)str));
  
     return index;
 }

==== non-SSE (standard) version ====

 static size_t strlen_std(const char * const str)
 {
     size_t tmp;
 
     tmp = (size_t)str;
 		
     asm(".strlen_start:"
     "    cmpb $0, 0(%1);"
     "	 jz .strlen_done;"
     "    cmpb $0, 1(%1);"
     "	 jz .strlen_1;"
     "    cmpb $0, 2(%1);"
     "	 jz .strlen_2;"
     "    cmpb $0, 3(%1);"
     "	 jz .strlen_3;"
     "    cmpb $0, 4(%1);"
     "	 jz .strlen_4;"
     "    cmpb $0, 5(%1);"
     "	 jz .strlen_5;"
     "    cmpb $0, 6(%1);"
     "	 jz .strlen_6;"
     "    cmpb $0, 7(%1);"
     "	 jz .strlen_7;"
     // if no 0s were found, add 8 and start again 
     "	 add $8, %1;"
     "	 jmp .strlen_start;"
     // here are all the expanded functions that get called once a 0 is found 
     ".strlen_1:"
     "	 inc %1;"
     "	 jmp .strlen_done;"
     ".strlen_2:"
     "	 add $2, %1;"
     "	 jmp .strlen_done;"
     ".strlen_3:"
     "	 add $3, %1;"
     "	 jmp .strlen_done;"
     ".strlen_4:"
     "	 add $4, %1;"
     "	 jmp .strlen_done;"
     ".strlen_5:"
     "	 add $5, %1;"
     "	 jmp .strlen_done;"
     ".strlen_6:"
     "	 add $6, %1;"
     "	 jmp .strlen_done;"
     ".strlen_7:"
     "	 add $7, %1;"
     // output the current value, and exit. 
     ".strlen_done:"
     "	mov %1, %0;"
     :"=r"(tmp):"0"(tmp));
 
     return tmp - (size_t)str;
 }

==== function stub ====

 size_t strlen(const char * const str)
 {
     // check for SSE 4.2
     if(cpuid_features.SSE4_2)
     {
         return strlen_sse4_2(str);
     }
 
     return strlen_std(str);
 }

== math.h ==
=== wrappers ===
==== PADDB/W/D/Q ====
PADDB/W/D/Q is an SSE2 instruction that ADDs two packed integers.  If PADDB is used, each vector is 16 packed bytes that is added together using a single SIMD instruction.  These functions return the result in the DST array (the first one provided).  Also, PADDx does not handle addition overflows, it just wraps around. 

 // add two 16 byte packed integers together
 void paddb(uint8_t DST[16], const uint8_t v2[16])
 {
     // we need an aligned buffer, just in case
     uint8_t SRC[16] __attribute__((aligned(16)));
 
     // if SRC is not aligned, we need to copy those values into our temp buffer
     if((size_t)SRC & (SSE_XMM_SIZE - 1))
     { memcpy(SRC, v2, 16); }
 
     // if SRC is not aligned, PADDB will GPF
     asm(" movdqu (%0), %%xmm1;   "   // move DST into XMM1
         " paddb (%1), %%xmm1;    "   // add DST+SRC into XMM1
         " movdqu %%xmm1, (%0);   "   // store XMM1 into DST
         :: "r"((size_t)DST), "r"((size_t)SRC));
 }

 // add two 8 word packed integers together
 void paddw(uint16_t DST[8], const uint16_t v2[8])
 {
     // we need an aligned buffer, just in case
     uint16_t SRC[8] __attribute__((aligned(16)));
 
     // if SRC is not aligned, we need to copy those values into our temp buffer
     if((size_t)SRC & (SSE_XMM_SIZE - 1))
     { memcpy(SRC, v2, 16); }
 
     // if SRC is not aligned, PADDB will GPF
     asm(" movdqu (%0), %%xmm1;   "   // move DST into XMM1
         " paddw (%1), %%xmm1;    "   // add DST+SRC into XMM1
         " movdqu %%xmm1, (%0);   "   // store XMM1 into DST
         :: "r"((size_t)DST), "r"((size_t)SRC));
 }

 // add two 4 double-word packed integers together
 void paddd(uint32_t DST[4], const uint32_t v2[4])
 {
     // we need an aligned buffer, just in case
     uint32_t SRC[4] __attribute__((aligned(16)));
 
     // if SRC is not aligned, we need to copy those values into our temp buffer
     if((size_t)SRC & (SSE_XMM_SIZE - 1))
     { memcpy(SRC, v2, 16); }
 
     // if SRC is not aligned, PADDB will GPF
     asm(" movdqu (%0), %%xmm1;   "   // move DST into XMM1
         " paddd (%1), %%xmm1;    "   // add DST+SRC into XMM1
         " movdqu %%xmm1, (%0);   "   // store XMM1 into DST
         :: "r"((size_t)DST), "r"((size_t)SRC));
 }

 // add two 2 quad-word packed integers together
 void paddq(uint64_t DST[2], const uint64_t v2[2])
 {
     // we need an aligned buffer, just in case
     uint64_t SRC[2] __attribute__((aligned(16)));
 
     // if SRC is not aligned, we need to copy those values into our temp buffer
     if((size_t)SRC & (SSE_XMM_SIZE - 1))
     { memcpy(SRC, v2, 16); }
 
     // if SRC is not aligned, PADDB will GPF
     asm(" movdqu (%0), %%xmm1;   "   // move DST into XMM1
         " paddq (%1), %%xmm1;    "   // add DST+SRC into XMM1
         " movdqu %%xmm1, (%0);   "   // store XMM1 into DST
         :: "r"((size_t)DST), "r"((size_t)SRC));
 }
````

<a id="w002520"></a>

## W002520 — Fpu

Bron-ID: W002520
Namespace: Main (0)
Revisie: 7447 | Datum: 2009-03-28T22:21:02Z
Bron: https://wiki.osdev.org/Fpu
Revisie-URL: https://wiki.osdev.org/index.php?title=Fpu&oldid=7447
Doorverwijzing: FPU

````text
#REDIRECT [[FPU]]
````

<a id="w002521"></a>

## W002521 — SSE

Bron-ID: W002521
Namespace: Main (0)
Revisie: 30331 | Datum: 2026-03-17T20:52:56Z
Bron: https://wiki.osdev.org/SSE
Revisie-URL: https://wiki.osdev.org/index.php?title=SSE&oldid=30331

````text
{{Floats}}
''' Streaming SIMD Extensions (SSE) ''' is a SIMD extension to the [[x86]] architecture. Introduced by Intel in 1999, it is the successor of the earlier [[MMX]] extension.

== Streaming SIMD Extensions (SSE) ==
=== Introduction ===
SSE was introduced in the Pentium III and offered an additional 70 instructions to the Intel Instruction Set.  SSE instructions can help give an increase in data thouroughput due to Single Instruction, Multiple Data (SIMD) instructions.  These instructions can execute a common expression on multiple data in parallel.  

There are 8 (16 in 64-bit mode) XMM registers (XMM0-7(15)) that come with SSE, and they are 128-bit registers.  Certain SSE instructions (movntdqa, movdqa, movdqu, etc...) can load 16 bytes from memory or store 16 bytes to memory in a single operation.  Also, SSE introduces a few non-temporal hint instructions (movntdqa and movntdq) that allow one-shot memory locations to be stored in non-temporal memory so those location references to do not pollute the small on-chip caches.  

Since this change added new registers, it is disabled by default as the typical operating system of that time was not yet able to save those registers on a task switch. To support SSE, you will need to implement separate code paths for saving and restoring SSE state (as those instructions will cause an exception on processors that do not support it), and handlers for the new exceptions. After that, you can tell the CPU to enable SSE use in userland tasks.

=== Checking for SSE ===
to check for SSE CPUID.01h:EDX.SSE[bit 25] needs to be set
<syntaxhighlight lang="asm">
mov eax, 0x1
cpuid
test edx, 1<<25
jz .noSSE
;SSE is available
</syntaxhighlight>

=== Adding support ===
In order to allow SSE instructions to be executed without generating a #UD, we need to alter the CR0 and CR4 registers. 
 clear the CR0.EM bit (bit 2) [ CR0 &= ~(1 << 2) ]
 set the CR0.MP bit (bit 1) [ CR0 |= (1 << 1) ]
 set the CR4.OSFXSR bit (bit 9) [ CR4 |= (1 << 9) ]
 set the CR4.OSXMMEXCPT bit (bit 10) [ CR4 |= (1 << 10) ]

Here is an asm example:
<syntaxhighlight lang="asm">
;now enable SSE and the like
mov eax, cr0
and ax, 0xFFFB		;clear coprocessor emulation CR0.EM
or ax, 0x2			;set coprocessor monitoring  CR0.MP
mov cr0, eax
mov eax, cr4
or ax, 3 << 9		;set CR4.OSFXSR and CR4.OSXMMEXCPT at the same time
mov cr4, eax
ret
</syntaxhighlight>

=== FXSAVE and FXRSTOR ===
FXSAVE and FXRSTOR are used to save and load the complete SSE, x87 FPU, and MMX states from memory.  The host needs to allocate 512 bytes for the storage and use that memory pointer as an operand to either FXSAVE or FXRSTOR.  Before using either of those instructions, make sure to check the CPUID features for the FXSR bit.  Also, like most SSE instructions, the memory operand needs to be 16-byte aligned or a #GP exception will occur.  Remember to execute FXSAVE *before* any MXCSR modifications happen, or else it the register will most likely get overwritten or set to 0 based on the unknown state of the MXCSR_MASK.  

Example usage:
<syntaxhighlight lang="c">
char fxsave_region[512] __attribute__((aligned(16)));
asm volatile(" fxsave %0 "::"m"(fxsave_region));
</syntaxhighlight>
or in asm:
<syntaxhighlight lang="asm">
segment .code
SaveFloats:
fxsave [SavedFloats]
segment .data
align 16
SavedFloats: TIMES 512 db 0
</syntaxhighlight>
Pitfalls: only one level of saving supported.

=== MXCSR and its helpers LDMXCSR and STMXCSR ===
The MXCSR register holds all of the masking and flag information for use with SSE floating-point operations.  Just like the x87 FPU control word, if you would like to mask certain exceptions from occuring or would like to specify rounding types, MXCSR will need to be modified.  Bits 16-31 are reserved and will cause a #GP exception if set.  LDMXCSR and STMXCSR load and write the MXCSR register respectively.  They both require a 32-bit memory operand.  SSE support needs to already be set up before using either of these instructions (CR4.OSFXSR = 1, CR0.EM = 0, and CR0.TS = 0).  If bits 7-12 are set, all SSE floating-point exceptions are masked.  Bits 0-5 are exception status flags that are set if the corresponding exception has occured.  Bits 13-14 are the RC (Rounding Control) bits.  RC:0 = to nearest, RC:1 = down, RC:2 = up, RC:3 = truncate. 
 
== Updates to SSE ==
Later processors have added more instructions for different work to be performed on the vector registers. Supporting them with SSE support in place doesn't require any effort on the part of the OS (except for AVX, see below). The actual user of the instructions should however check if those instructions actually exist.
=== CPUID bits ===
=====SSE2=====
<small>'''Streaming SIMD Extensions 2 (SSE2)'''</small><br />
The bit for SSE2 can be found on CPUID page 1, in EDX bit 26.
=====SSE3=====
<small>'''Streaming SIMD Extensions 3 (SSE3)'''</small><br />
The bit for SSE3 can be found on CPUID page 1, in ECX bit 0.
=====SSSE3=====
<small>'''Supplemental Streaming SIMD Extensions 3 (SSSE3)'''</small><br />
The bit for SSSE3 can be found on CPUID page 1, in ECX bit 9.

=====SSE4=====
<small>'''Streaming SIMD Extensions 4 (SSE4)'''</small><br />
The bit for SSE4.1 can be found on CPUID page 1, in ECX bit 19

The bit for SSE4.2 can be found on CPUID page 1, in ECX bit 20

The bit for SSE4A can be found on CPUID page 1, in ECX bit 6

=====SSE5=====
<small>'''Streaming SIMD Extensions 5 (SSE5)'''</small><br />
SSE5 was planned as one unit, but split into several:
======XOP======
The bit for XOP can be found on CPUID page 1, in ECX bit 11
======FMA4======
The bit for FMA4 can be found on CPUID page 1, in ECX bit 16
======CVT16======
The bit for CVT16 can be found on CPUID page 1, in ECX bit 29
======AVX======
The bit for AVX can be found on CPUID page 1, in ECX bit 28
======XSAVE======
The bit for XSAVE (needed to manage extended processor states) can be found on CPUID page 1, in ECX bit 26
======AVX2======
The bit for [[AVX2]] can be found on CPUID page 7, 0, in EBX bit 5

=====AVX-512=====
The bits for AVX-512 are in CPUID page 0x0D, 0x0, EAX bits 5-7

AVX512 implements separate features that can also be detected in CPUID page 7, 0. Basic support is detected by checking the AVX512F Bit (AVX-512 Foundation) in CPUID page 7, 0 EBX Bit 16, you can also check various AVX512 Features through the same CPUID Function, the bits are listed [[https://en.wikipedia.org/wiki/CPUID#EAX=7,_ECX=0:_Extended_Features here]]
===X86_64===
When the [[X86-64]] architecture was introduced, AMD demanded a minimum level of SSE support to simplify OS code. Any system capable of long mode should support at least SSE and SSE2, which means that the kernel does not need to care about the old FPU save code.
X86-64 adds 8 SSE registers (xmm8 - xmm15) to the mix. However, you can only access these in 64 bit mode.

'''Advanced Vector Extensions''' is a '''SIMD''' (Single Instruction, Multiple Data) instruction set introduced by Intel in 2011.

=== AVX ===

AVX needs to be enabled by the kernel before being used. Forgetting to do this will raise an #UD on the first AVX call.
Both SSE and OSXSAVE must be enabled before allowing. Failing to do so will also produce an #UD.

AVX is enabled by setting bit 2 of the XCR0 register. Bit 1 of XCR0 must also be set (indicating SSE support).

Here is an example of assembly code enabling AVX after SSE has been enabled (you should check AVX and XSAVE are supported first, see above):

<syntaxhighlight lang="asm">
enable_avx:
    push rax
    push rcx
    push rdx

    xor rcx, rcx
    xgetbv ;Load XCR0 register
    or eax, 7 ;Set AVX, SSE, X87 bits
    xsetbv ;Save back to XCR0

    pop rdx
    pop rcx
    pop rax
    ret
</syntaxhighlight>

To enable AVX-512, set the OPMASK (bit 5), ZMM_Hi256 (bit 6), Hi16_ZMM (bit 7) of XCR0. You must ensure that these bits are valid first (see above).

==See Also==
* [[MMX]] The preceding SIMD instruction extension, made obsolete by SSE
* The [[User:01000101/optlib/|optimisation library]] of 01000101, containing example code

=== References ===
* The [[Wikipedia:EN:Streaming_SIMD_Extensions|Wikipedia article]] on SSE 
* The [[Wikipedia:EN:AVX-512|Wikipedia article]] on AVX-512
[[Category:X86]]
````

<a id="w002523"></a>

## W002523 — Optimization

Bron-ID: W002523
Namespace: Main (0)
Revisie: 7472 | Datum: 2009-03-31T05:22:16Z
Bron: https://wiki.osdev.org/Optimization
Revisie-URL: https://wiki.osdev.org/index.php?title=Optimization&oldid=7472
Doorverwijzing: Optimizing

````text
#REDIRECT [[Optimizing]]
````

<a id="w002524"></a>

## W002524 — User talk:01000101/optlib/

Bron-ID: W002524
Namespace: User talk (3)
Revisie: 10596 | Datum: 2010-08-08T02:47:02Z
Bron: https://wiki.osdev.org/User_talk:01000101/optlib/
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3A01000101%2Foptlib%2F&oldid=10596

````text
== Results ==
Under the "results" section, what exactly do those numbers mean? --[[User:Messiahandrw|MessiahAndrw]] 02:47, 8 August 2010 (UTC)

Nice stuff... how about adding memset() (should be a mere variation of memclr())? Having profiling information on how these functions perform compared to "straight C" implementations or e.g. newlib / glibc functions would be awesome. -- [[User:Solar|Solar]] 09:56, 1 April 2009 (UTC)

== Suggestion for the memclr_sse2 function ==

You wrote

    size_t i;

    // find out if "m_start" is aligned on a SSE_XMM_SIZE boundary
    if((size_t)m_start & (SSE_XMM_SIZE - 1))
    {
        i = 0;
        [...]
    }
    else
    {
        // if "m_start" was aligned, set our count to 0
        i = 0;
    }

It would be cleaner like this:
 size_t i = 0;
 if((size_t)m_start & (SSE_XMM_SIZE - 1))
 {
        [...]
 }
````

<a id="w002525"></a>

## W002525 — User:Pcmattman/Category:Networking

Bron-ID: W002525
Namespace: User (2)
Revisie: 7488 | Datum: 2009-04-02T02:06:28Z
Bron: https://wiki.osdev.org/User:Pcmattman/Category:Networking
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APcmattman%2FCategory%3ANetworking&oldid=7488

````text
This category contains information about networking.
````

<a id="w002526"></a>

## W002526 — User:Pcmattman/Category:NetworkingHardware

Bron-ID: W002526
Namespace: User (2)
Revisie: 7489 | Datum: 2009-04-02T02:07:48Z
Bron: https://wiki.osdev.org/User:Pcmattman/Category:NetworkingHardware
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APcmattman%2FCategory%3ANetworkingHardware&oldid=7489

````text
This category contains information about specific networking hardware, specifically NICs.
````

<a id="w002528"></a>

## W002528 — D Bare Bones

Bron-ID: W002528
Namespace: Main (0)
Revisie: 30605 | Datum: 2026-05-22T14:00:48Z
Bron: https://wiki.osdev.org/D_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=D_Bare_Bones&oldid=30605

````text
{{BadPractice}}
{{Disputed|Talk:D Bare Bones}}
{{TutorialExplain}}
{{BeginnersWarning}}
{{Rating|1}}{{Template:Kernel designs}}

In this Tutorial we will write a kernel in the [[D]] language and boot it.

==Preface==

The following tutorial assumes basic knowledge of a compiler, linker and assembler toolchain. 
It also of course assumes prior knowledge of the D programming language.

==Overview==

In this tutorial we will create a simple D kernel that prints 'D' on to the screen. The basic setup will consist of three files:
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

       call kmain

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
 
extern(C) void kmain(uint magic, uint addr) {
 
	ubyte* vidmem = cast(ubyte*)0xFFFF_8000_000B_8000; //Video memory address
 
	for (int i = 0; i < 80*25*2; i++) { //Loops through the screen and clears it
			volatileStore(vidmem + i, 0);
	}
 
	volatileStore(vidmem, 'D' & 0xFF); //Prints the letter D
	volatileStore(vidmem + 1, 0x07); //Sets the colour for D to be light grey (0x07)
 
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

Note the "-fno-druntime" argument above. This is how gdc spells the -betterC flag from other D compilers, and the "Better C" page of the D language reference explains how this limits the language. If you want to drop that you'll have to add the D runtime to your kernel.

Hopefully this has gotten you started on writing your operating system in the D programming language.

== Further reading ==

[[D_Bare_Bones_II|Creating a minimal output to the console]]

[[D_barebone_with_ldc2|D BareBone with 64 bit and ldc2]]

[[Category:Bare bones tutorials|D bare bones]]
[[Category:D]]
[[Category:Inadvisable]]
````

<a id="w002530"></a>

## W002530 — D Bare bones

Bron-ID: W002530
Namespace: Main (0)
Revisie: 7519 | Datum: 2009-04-03T10:20:59Z
Bron: https://wiki.osdev.org/D_Bare_bones
Revisie-URL: https://wiki.osdev.org/index.php?title=D_Bare_bones&oldid=7519
Doorverwijzing: D Bare Bones

````text
#REDIRECT [[D Bare Bones]]
````

<a id="w002531"></a>

## W002531 — D bare bones

Bron-ID: W002531
Namespace: Main (0)
Revisie: 7518 | Datum: 2009-04-03T10:20:29Z
Bron: https://wiki.osdev.org/D_bare_bones
Revisie-URL: https://wiki.osdev.org/index.php?title=D_bare_bones&oldid=7518
Doorverwijzing: D Bare Bones

````text
#REDIRECT [[D Bare Bones]]
````

<a id="w002533"></a>

## W002533 — Video Signals And Timing

Bron-ID: W002533
Namespace: Main (0)
Revisie: 18071 | Datum: 2015-05-14T06:26:30Z
Bron: https://wiki.osdev.org/Video_Signals_And_Timing
Revisie-URL: https://wiki.osdev.org/index.php?title=Video_Signals_And_Timing&oldid=18071

````text
{{Video warning}}

To make the video card and monitor independent of each other, there is a standard in communication. This page describes the technical parts of that link with the necessary information to program a video card and have it render properly on the attached screen. 

== Display Signal ==
There are 15 pins in a standard VGA Cable. When your video card sends its video data to the monitor, it uses 5 data channels:
* Analog Red
* Analog Green
* Analog Blue
* Horizontal Sync
* Vertical Sync
The screen is built up scanline by scanline, by sending the appropriate RGB values over the connection. However, only having a stream of colours will not tell you which part of the stream belongs to the top-left-pixel on the screen. To solve this they added two more signals: the horizontal sync which pulses when a horizontal line is done, and the vertical sync, which pulses when all rows have been completed. Thus, each frame corresponds to a single vertical synchronisation pulse. Since each frame has the same amount of pixels, the time between consecutive pulses is a constant. The monitor times the frequency of the pulses, then based on that it knows at what time the color data should be sent to what position on the screen.

The system is however not that simple. The first monitors were CRTs, which used magnetic fields to project electron beams onto a phosphoric layer, making them visible for a short period of time. The magnetic fields of a CRT had some inertia - they couldn't be set from a random location to another random location in the time available for one pixel. Hence the video signal has to have some gaps to cope with the time a monitor needs to alter the magnetic fields and point the electron beam back to the other side of the screen. In the meantime there could be no color signal, or you might have gotten stripes on the screen.

CRTs have improved a lot since then, and are now being superseded by the highly intelligent LCD display. The standard for signaling hasn't changed since. Since the original standard was made for CRTs, the rest of the document will implicitly assume an (old) CRT. LCD displays basically decode a CRT signal and then restructure it to fit their own screen grid under analog signaling methods. Similarly, in many other references you will see that the display unit of a video card is referred to as the "CRTC" or "CRT Controller"

== Display Composition ==
All frames of a video signal have a specific layout, and video cards have a semi-standard way of thinking about these signals. Basically, you will have to provide the video card with enough information to be able to derive all sizes present in the following diagram. [[VGA Hardware]] explains how you can give these sizes to a VGA compatible. For other hardware, you should check your card's documentation on how these values are stored within them.

[[Image:VGA crtc.gif]]

Both horizontally and vertically, there are 6 states, and hence, 6 sizes. Each size is in pixels. Most of the emitted data is part of the active display, the area of x resolution * y resolution pixels. Each of the cuts are where:
* active display goes into overscan
* where overscan goes into blanking
* where blanking gets the synchronization line changed
* where the sync is restored to its original value
* where the blanking goes back into overscan
* where the overscan goes into active display, where the scanline or frame is completed and the next one is started.

Video cards usually gives you the following registers to program (both horizontally and vertically)
* Resolution (pixel size of active display)
* Total (total number of 'pixels' in a single run)
* Blanking start and end (or start and size) - marks the location of the blanking. The are that is neither blanking nor active display becomes the overscan area.
* Sync start and end (or start and size) - marks the location of the synchronization pulse

== Frequencies ==

When you increase the resolution, you will need to send more pixels to the display. If you would keep sending pixels at the same rate, the time to transmit one frame will go up, and consequently the amount of frames in a certain timespan will go down. Since a CRT displayed pixel only gives light for a short time before running out of energy, it needs to be repeatedly refreshed. If this is done fast enough (at about 60Hz, 60 times a second) the screen appears almost constant to the human eye. This improves further when the refresh rate goes up to a point where it doesn't matter to the human eye. However when it drops too much, the screen starts appearing flashing, causing headaches to the user. Hence, we need to keep the frequency at at least 60Hz for user's sanity, and below some other rate dictated by the monitor's capabilities. To make a full frame of pixels fit within one sixtieth of a second, we will have to adjust the speed at which these pixels are transmitted. This speed is called the pixel clock, or dot clock. For example, a VGA's dot clock is either 25MHz or 28MHz, corresponding to 25 million pixels per second or 28 million pixels per second, the latter one being only just enough to display a resolution of 720x480 at 60Hz (recall that the active display is only a part of the frame). Most higher resolution video can therefore use a wide range of dot clocks, well above 25MHz, with the current range allowing enough bandwith to easily exceed 1600x1200 at a 100Hz. 

While the resolution is limited by the video card, In most non-VGA scenario's, it is the monitor that can not handle the speed of the signal. The monitor has a allowed vertical frequency (the amount of frames per second, usually listed in Hz), and horizontal frequency (listed in KHz). Some CRTs are fixed frequency, only allowing certain frequencies to be used both horizontally and vertically. Old VGA displays are infamous for burning out when you feed them a signal that doesn't exactly match these rates. While modern CRTs are mostly protected from bad signaling, you must know that you can break hardware in this fashion, and that you need to be careful.

== General Timing Formula ==

In order to cope with all the CRT antiquities, VESA has produced a set of equations that allows you to compute the various display settings you need, given the desired resolution. There are three separate sets of formulas: One to compute all settings from resolution and desired refresh rate, one taking resolution and horizontal frequency, and one taking resolution and dot clock. 

Normally before setting a mode you would do the following:
* Compute the horizontal refresh rate and pixel clock from resolution and frequency
* Check the video card for the nearest pixel clock it can give you
* Compute all settings given the resolution and pixel clock.
* Check if the horizontal and vertical frequencies are within the monitor's allowed range.

If the frequencies happen to be out of range, you will have to adjust the refresh rate and or resolution. Since the refresh rate can go down a bit without giving issues, that's usually the first choice to fix. Hence:
* Clamp the horizontal frequency to its limit
* Compute pixel clock and vertical frequency from resolution and the new horizontal frequency
* Select the appropriate pixel clock the video card has (making sure you are rounding away from the limit, so round down if you use the maximum horizontal frequency)
* Compute the horizontal and vertical frequency given the new dot clock
* Check if the frequencies are within range.

The horizontal frequency should now be correct, you can decide whether or not to allow the new mode because the vertical rate has dropped below an acceptable rate, and you can inform the user that the mode is not supported.

The formula's provided by VESA use various scales of constants. Brendan's working on changing the scales to units. Right now, the scale space is still unknown.

=== Common parts of the GTF formulas ===

<tt>[[#H_PIXELS_RND|H_PIXELS_RND]] = ( ''ROUND'' ( [[#H_PIXELS|H_PIXELS]] / [[#CELL_GRAN_RND|CELL_GRAN_RND]] ) ) * [[#CELL_GRAN_RND|CELL_GRAN_RND]]

'''if''' ( [[#INTERLACE_REQUIRED|INTERLACE_REQUIRED]] == '''true''' ) <br />
{<br />
:[[#V_LINES_RND|V_LINES_RND]] = ''ROUND'' ( [[#V_LINES|V_LINES]] / 2 )<br />
:[[#V_FIELD_RATE_REQUIRED|V_FIELD_RATE_REQUIRED]] = [[#REFRESH_RATE_REQUIRED|REFRESH_RATE_REQUIRED]] * 2<br />
:[[#INTERLACE|INTERLACE]] = 0.5<br />
} '''else''' {<br />
:[[#V_LINES_RND|V_LINES_RND]] = ''ROUND'' ( [[#V_LINES|V_LINES]]) )<br />
:[[#V_FIELD_RATE_REQUIRED|V_FIELD_RATE_REQUIRED]] = [[#REFRESH_RATE_REQUIRED|REFRESH_RATE_REQUIRED]]<br />
:[[#INTERLACE|INTERLACE]] = 0<br />
}<br />

'''if''' ( [[#MARGINS_REQUIRED]] == '''true''' ) {<br />
:[[#TOP_MARGIN_LINES|TOP_MARGIN_LINES]] = ''ROUND'' ( [[#MARGIN_PERCENT|MARGIN_PERCENT]] / 100 * [[#V_LINES_RND|V_LINES_RND]] )<br />
:[[#BOTTOM_MARGIN_LINES|BOTTOM_MARGIN_LINES]] = ''ROUND'' ( [[#MARGIN_PERCENT|MARGIN_PERCENT]] / 100 * [[#V_LINES_RND|V_LINES_RND]] )<br />
:[[#LEFT_MARGIN_PIXELS|LEFT_MARGIN_PIXELS]] = ( ''ROUND'' ( [[#H_PIXELS_RND|H_PIXELS_RND]] * [[#MARGIN_PERCENT|MARGIN_PERCENT]] / 100 / [[#CELL_GRAN_RND|CELL_GRAN_RND]] ) ) * [[#CELL_GRAN_RND|CELL_GRAN_RND]]<br />
:[[#RIGHT_MARGIN_PIXELS|RIGHT_MARGIN_PIXELS]] = ( ''ROUND'' ( [[#H_PIXELS_RND|H_PIXELS_RND]] * [[#MARGIN_PERCENT|MARGIN_PERCENT]] / 100 / [[#CELL_GRAN_RND|CELL_GRAN_RND]] ) ) * [[#CELL_GRAN_RND|CELL_GRAN_RND]]<br />
} '''else''' {<br />
:[[#TOP_MARGIN_LINES|TOP_MARGIN_LINES]] = 0<br />
:[[#BOTTOM_MARGIN_LINES|BOTTOM_MARGIN_LINES]] = 0<br />
:[[#LEFT_MARGIN_PIXELS|LEFT_MARGIN_PIXELS]] = 0<br />
:[[#RIGHT_MARGIN_PIXELS|RIGHT_MARGIN_PIXELS]] = 0<br />
}

'''/* use one of the GTF equations here */'''

[[#V_FRONT_PORCH|V_FRONT_PORCH]] = [[#MIN_PORCH_RND|MIN_PORCH_RND]] + [[#INTERLACE|INTERLACE]]<br />
[[#H_SYNC|H_SYNC]] = ( ''ROUND'' ( [[#H_SYNC_PERCENT|H_SYNC_PERCENT]] / 100 * [[#H_TOTAL|H_TOTAL]] / [[#CELL_GRAN_RND|CELL_GRAN_RND]]  ) ) * [[#CELL_GRAN_RND|CELL_GRAN_RND]]<br />
[[#H_FRONT_PORCH|H_FRONT_PORCH]] = [[#H_TOTAL|H_TOTAL]] / 2 - [[#H_SYNC|H_SYNC]]<br />
[[#H_BACK_PORCH|H_BACK_PORCH]] = [[#H_FRONT_PORCH|H_FRONT_PORCH]] + [[#H_SYNC|H_SYNC]] <br />
</tt>

=== GTF Using resolution and refresh rate ===

<tt>
[[#H_PERIOD_ESTIMATE|H_PERIOD_ESTIMATE]] = ( 1 / [[#V_FIELD_RATE_REQUIRED|V_FIELD_RATE_REQUIRED]] - [[#MIN_V_SYNC_AND_BACK_PORCH|MIN_V_SYNC_AND_BACK_PORCH]] / 1000000 ) 
:/ ( [[#V_LINES_RND|V_LINES_RND]] + 2 * [[#TOP_MARGIN_LINES|TOP_MARGIN_LINES]] + [[#MIN_PORCH_RND|MIN_PORCH_RND]] + [[#INTERLACE|INTERLACE]] ) * 1000000


[[#V_SYNC_AND_BACK_PORCH|V_SYNC_AND_BACK_PORCH]] = ''ROUND'' ( [[#MIN_V_SYNC_AND_BACK_PORCH|MIN_V_SYNC_AND_BACK_PORCH]] / [[#H_PERIOD_ESTIMATE|H_PERIOD_ESTIMATE]] )

[[#V_BACK_PORCH|V_BACK_PORCH]] = [[#V_SYNC_AND_BACK_PORCH|V_SYNC_AND_BACK_PORCH]] - [[#V_SYNC_RND|V_SYNC_RND]]

[[#TOTAL_V_LINES|TOTAL_V_LINES]] = [[#V_LINES_RND|V_LINES_RND]] + 
:[[#TOP_MARGIN_LINES|TOP_MARGIN_LINES]] + [[#BOTTOM_MARGIN_LINES|BOTTOM_MARGIN_LINES]] + 
:[[#V_SYNC_AND_BACK_PORCH|V_SYNC_AND_BACK_PORCH]] + [[#INTERLACE|INTERLACE]] + [[#MIN_PORCH_RND|MIN_PORCH_RND]]


[[#V_FIELD_RATE_ESTIMATE|V_FIELD_RATE_ESTIMATE]] = 1000000 / [[#H_PERIOD_ESTIMATE|H_PERIOD_ESTIMATE]] / [[#TOTAL_V_LINES|TOTAL_V_LINES]]

[[#H_PERIOD|H_PERIOD]] = [[#H_PERIOD_ESTIMATE|H_PERIOD_ESTIMATE]] * [[#V_FIELD_RATE_ESTIMATE|V_FIELD_RATE_ESTIMATE]] / [[#V_FIELD_RATE_REQUIRED|V_FIELD_RATE_REQUIRED]]

[[#V_FIELD_RATE|V_FIELD_RATE]] = 1000000 / [[#H_PERIOD|H_PERIOD]] / [[#TOTAL_V_LINES|TOTAL_V_LINES]]

'''if''' ( [[#INTERLACE_REQUIRED|INTERLACE_REQUIRED]] == '''true''' ) {<br />
:[[#V_FRAME_RATE|V_FRAME_RATE]] = [[#V_FIELD_RATE|V_FIELD_RATE]] / 2<br />
} '''else''' {<br />
:[[#V_FRAME_RATE|V_FRAME_RATE]] = [[#V_FIELD_RATE|V_FIELD_RATE]]<br />
}<br />

[[#TOTAL_ACTIVE_PIXELS|TOTAL_ACTIVE_PIXELS]] = [[#H_PIXELS_RND|H_PIXELS_RND]] + [[#LEFT_MARGIN_PIXELS|LEFT_MARGIN_PIXELS]] + [[#RIGHT_MARGIN_PIXELS|RIGHT_MARGIN_PIXELS]]

[[#IDEAL_DUTY_CYCLE|IDEAL_DUTY_CYCLE]] = [[#C_PRIME|C_PRIME]] - [[#M_PRIME|M_PRIME]] * [[#H_PERIOD|H_PERIOD]] / 1000

[[#H_BLANK_PIXELS|H_BLANK_PIXELS]] = ( ''ROUND'' ( 
::( [[#TOTAL_ACTIVE_PIXELS|TOTAL_ACTIVE_PIXELS]] * [[#IDEAL_DUTY_CYCLE|IDEAL_DUTY_CYCLE]] / ( 100 - [[#IDEAL_DUTY_CYCLE|IDEAL_DUTY_CYCLE]] ) / ( 2 * [[#CELL_GRAN_RND|CELL_GRAN_RND]] ) ) 
:) ) * 2 * [[#CELL_GRAN_RND|CELL_GRAN_RND]]

[[#H_TOTAL|H_TOTAL]] = [[#TOTAL_ACTIVE_PIXELS|TOTAL_ACTIVE_PIXELS]] + [[#H_BLANK_PIXELS|H_BLANK_PIXELS]]

[[#PIXEL_FREQ|PIXEL_FREQ]] = [[#H_TOTAL|H_TOTAL]] / [[#H_PERIOD|H_PERIOD]] * 1000000

[[#H_FREQ|H_FREQ]] = 1 / [[#H_PERIOD|H_PERIOD]]
</tt>

=== GTF Using resolution and pixel clock ===
<tt>
[[#PIXEL_FREQ|PIXEL_FREQ]] = [[#PIXEL_FREQ_REQUIRED|PIXEL_FREQ_REQUIRED]]<br />
[[#TOTAL_ACTIVE_PIXELS|TOTAL_ACTIVE_PIXELS]] = [[#H_PIXELS_RND|H_PIXELS_RND]] + [[#RIGHT_MARGIN_PIXELS|RIGHT_MARGIN_PIXELS]] + [[#LEFT_MARGIN_PIXELS|LEFT_MARGIN_PIXELS]]<br />

[[#IDEAL_H_PERIOD|IDEAL_H_PERIOD]] = ( ( [[#C_PRIME|C_PRIME]] - 100 ) + ( ''SQRT'' ( ( ( 100 - [[#C_PRIME|C_PRIME]] ) ^ 2 ) + 
:( 0.4 * [[#M_PRIME|M_PRIME]] * ( [[#TOTAL_ACTIVE_PIXELS|TOTAL_ACTIVE_PIXELS]] + [[#RIGHT_MARGIN_PIXELS|RIGHT_MARGIN_PIXELS]] + [[#LEFT_MARGIN_PIXELS|LEFT_MARGIN_PIXELS]] ) / [[#PIXEL_FREQ|PIXEL_FREQ]] / 1000000 ) ) )
:) ) / 2 / [[#M_PRIME|M_PRIME]] * 1000<br />

[[#IDEAL_DUTY_CYCLE|IDEAL_DUTY_CYCLE]] = [[#C_PRIME|C_PRIME]] - ( [[#M_PRIME]] * [[#IDEAL_H_PERIOD|IDEAL_H_PERIOD]] / 1000 )<br />

[[#H_BLANK_PIXELS|H_BLANK_PIXELS]] = ( ''ROUND'' ( [[#TOTAL_ACTIVE_PIXELS|TOTAL_ACTIVE_PIXELS]] * [[#IDEAL_DUTY_CYCLE|IDEAL_DUTY_CYCLE]] / ( 100 - [[#IDEAL_DUTY_CYCLE|IDEAL_DUTY_CYCLE]] ) / ( 2 * [[#CELL_GRAN_RND|CELL_GRAN_RND]] ) ) ) * 2 * [[#CELL_GRAN_RND|CELL_GRAN_RND]]<br />

[[#H_TOTAL|H_TOTAL]] = [[#TOTAL_ACTIVE_PIXELS|TOTAL_ACTIVE_PIXELS]] + [[#H_BLANK_PIXELS|H_BLANK_PIXELS]]<br />
[[#H_FREQ|H_FREQ]] = [[#PIXEL_FREQ|PIXEL_FREQ]] / [[#H_TOTAL|H_TOTAL]]<br />
[[#H_PERIOD|H_PERIOD]] = 1 / [[#H_FREQ|H_FREQ]]<br />
[[#V_SYNC_AND_BACK_PORCH|V_SYNC_AND_BACK_PORCH]] = ''ROUND'' ( [[#MIN_V_SYNC_AND_BACK_PORCH|MIN_V_SYNC_AND_BACK_PORCH]] * [[#H_FREQ|H_FREQ]] / 1000000 )<br />
[[#V_BACK_PORCH|V_BACK_PORCH]] = [[#V_SYNC_AND_BACK_PORCH|V_SYNC_AND_BACK_PORCH]] - [[#V_SYNC_RND|V_SYNC_RND]]<br />
[[#TOTAL_V_LINES|TOTAL_V_LINES]] = [[#V_LINES_RND|V_LINES_RND]] + [[#TOP_MARGIN_LINES|TOP_MARGIN_LINES]] + [[#BOTTOM_MARGIN_LINES|BOTTOM_MARGIN_LINES]] + [[#INTERLACE|INTERLACE]] + [[#V_SYNC_AND_BACK_PORCH|V_SYNC_AND_BACK_PORCH]] + [[#MIN_PORCH_RND|MIN_PORCH_RND]]<br />
[[#V_FIELD_RATE|V_FIELD_RATE]] = [[#H_FREQ|H_FREQ]] / [[#TOTAL_V_LINES|TOTAL_V_LINES]]<br />

'''if''' ( [[#INTERLACE_REQUIRED|INTERLACE_REQUIRED]] == '''true''') 
{
:[[#V_FRAME_RATE|V_FRAME_RATE]] = [[#V_FIELD_RATE|V_FIELD_RATE]] / 2
} '''else''' {
:[[#V_FRAME_RATE|V_FRAME_RATE]] = [[#V_FIELD_RATE|V_FIELD_RATE]]
}</tt>

=== GTF Using resolution and horizontal frequency ===
<tt>

[[#H_FREQ|H_FREQ]] = [[#H_FREQ_REQUIRED|H_FREQ_REQUIRED]]<br />
[[#V_SYNC_AND_BACK_PORCH|V_SYNC_AND_BACK_PORCH]] = ''ROUND'' ( [[#MIN_V_SYNC_AND_BACK_PORCH|MIN_V_SYNC_AND_BACK_PORCH]] * [[#H_FREQ|H_FREQ]] / 1000000 )<br />
[[#V_BACK_PORCH|V_BACK_PORCH]] = [[#V_SYNC_AND_BACK_PORCH|V_SYNC_AND_BACK_PORCH]] - [[#V_SYNC_RND|V_SYNC_RND]]<br />
[[#TOTAL_V_LINES|TOTAL_V_LINES]] = [[#V_LINES_RND|V_LINES_RND]] + [[#TOP_MARGIN_LINES|TOP_MARGIN_LINES]] + [[#BOTTOM_MARGIN_LINES|BOTTOM_MARGIN_LINES]] + [[#INTERLACE|INTERLACE]] + [[#V_SYNC_AND_BACK_PORCH|V_SYNC_AND_BACK_PORCH]] + [[#MIN_PORCH_RND|MIN_PORCH_RND]]<br />
[[#V_FIELD_RATE|V_FIELD_RATE]] = [[#H_FREQ|H_FREQ]] / [[#TOTAL_V_LINES|TOTAL_V_LINES]]<br />

'''if''' ( [[#INTERLACE_REQUIRED|INTERLACE_REQUIRED]] == '''true''') <br />
{
:[[#V_FRAME_RATE|V_FRAME_RATE]] = [[#V_FIELD_RATE|V_FIELD_RATE]] / 2
} '''else''' {
:[[#V_FRAME_RATE|V_FRAME_RATE]] = [[#V_FIELD_RATE|V_FIELD_RATE]]
}

[[#TOTAL_ACTIVE_PIXELS|TOTAL_ACTIVE_PIXELS]] = [[#H_PIXELS_RND|H_PIXELS_RND]] + [[#RIGHT_MARGIN_PIXELS|RIGHT_MARGIN_PIXELS]] + [[#LEFT_MARGIN_PIXELS|LEFT_MARGIN_PIXELS]]<br />
[[#IDEAL_DUTY_CYCLE|IDEAL_DUTY_CYCLE]] = [[#C_PRIME|C_PRIME]] - ( [[#M_PRIME|M_PRIME]] / [[#H_FREQ|H_FREQ]] )<br />
[[#H_BLANK_PIXELS|H_BLANK_PIXELS]] = ( ''ROUND'' ( [[#TOTAL_ACTIVE_PIXELS|TOTAL_ACTIVE_PIXELS]] * [[#IDEAL_DUTY_CYCLE|IDEAL_DUTY_CYCLE]] / ( 100 - [[#IDEAL_DUTY_CYCLE|IDEAL_DUTY_CYCLE]] ) / ( 2 * [[#CELL_GRAN_RND|CELL_GRAN_RND]] ) ) ) * 2 * [[#CELL_GRAN_RND|CELL_GRAN_RND]]<br />
[[#H_TOTAL|H_TOTAL]] = [[#TOTAL_ACTIVE_PIXELS|TOTAL_ACTIVE_PIXELS]] + [[#H_BLANK_PIXELS|H_BLANK_PIXELS]]<br />
[[#H_PERIOD|H_PERIOD]] = 1 / [[#H_FREQ|H_FREQ]]<br />
[[#PIXEL_FREQ|PIXEL_FREQ]] = [[#H_TOTAL|H_TOTAL]] * [[#H_FREQ|H_FREQ]]<br />
</tt>

=== Variable reference ===
These are the variables used in the equations above:

==== Hardware properties ====
These are constant for a given monitor. They describe the capabilities and limitations of a CRT. 

{{Anchor|C_PRIME}}
''' C_PRIME '''
:Defaults to 40, the actual value can be computed from EDID data.

{{Anchor|M_PRIME}}
''' M_PRIME '''
:Defaults to 600, the actual value can be computed from EDID data.

{{Anchor|CELL_GRAN_RND}}
'''CELL_GRAN_RND'''
:Cell granularity - the amount of pixels the timings should be aligned to. For example, a VGA uses horizontal timings in multiples of the character clock, each character being 8 (graphics and some text modes) or 9 (most text modes) pixels wide. For a VGA graphics mode, you'll want to supply 8 here because of that. Depending on your specific hardware, the common values are either 8 or 1.

{{Anchor|MIN_PORCH_RND}}
'''MIN_PORCH_RND'''
:Usually 1. Probably used to force at least one unit of blanking around the sync period in order to not confuse graphics hardware that can not handle an absence of blanking around the synchronisation pulse.

{{Anchor|MIN_V_SYNC_AND_BACK_PORCH}}
'''MIN_V_SYNC_AND_BACK_PORCH'''
:The time in microseconds the monitor needs to detect the vertical synchronisation signal and subsequently retrace to the top-left corner of the screen. Defaults to 550.

{{Anchor|V_SYNC_RND}}
'''V_SYNC_RND'''
:The amount of scanlines with the vertical synchronisation pulse active. '''Todo''' Seems to depend on something VESA doesn't tell you. Equals 3, although two scanlines are used in legacy VGA mode 12/13.

{{Anchor|H_SYNC_PERCENT}}
'''H_SYNC_PERCENT'''
: The percentage of a scanline that should have the synchronisation pulse active. Equals 8%

==== Video mode inputs ===
These form the request of the user/programmer.

{{Anchor|MARGINS_REQUIRED}}
'''MARGINS_REQUIRED'''
:The margins are used to add a visible black surrounding to the displayed picture. I assume small margins are intended to make sure the entire picture is still visible when the picture isn't perfectly centered on the screen.

:In theory it would be possible to use margins to simulate a small screen on a larger screen, or to simulate different aspect ratios perfectly. For example, if you were setting up a 640 * 480 video mode on a 16:10 display you could make the left and right margin 64 pixels each (with no top or bottom margins), so that it looks exactly the same as it would on a 4:3 display, rather than being stretched to fit. The opposite is also possible (e.g. displaying 16:10 video data on a 4:3 display with "letterboxing"). Unfortunately the GTF calculations don't actually support different sized left/right and top/bottom margins.

{{Anchor|MARGIN_PERCENT}}
'''MARGIN_PERCENT'''
:'''(Fixme: Percentage!)''' Amount of margins you want to have around your screen. VESA suggests a default of 1.8 to allow for screens that do not perfectly fit the image to the center.

{{Anchor|INTERLACE_REQUIRED}}
'''INTERLACE_REQUIRED'''
:There are two kinds of video modes, interlaced and progressive modes. Progressive modes display the full frame in one go, while interlaced modes display each other scanline in each run, taking two frames to render the complete picture. Normally, you don't want this, and several video cards don't support it, so you will normally want to supply '''false''' here.

{{Anchor|V_LINES}}
'''V_LINES '''
:Vertical resolution of the display you want. For a 640x480 resolution, supply 480.

{{Anchor|H_PIXELS}}
'''H_PIXELS '''
:Horizontal resolution of the display you want. For a 640x480 resolution, supply 640. Note that video cards will usually want this to be a multiple of eight because of [[#CELL_GRAN_RND|cell granularity]].

{{Anchor|PIXEL_FREQ_REQUIRED}}
'''PIXEL_FREQ_REQUIRED'''
:The speed of the dot clock (or pixel clock), in pixels per second. This is usually the same as the dot clock's frequency in Hz, but in some cases there's a factor two difference (2x the frequency when the clock is being doubled, half the frequency when pixels are emitted every other cycle) The dot clock is usually limited by the video card, as higher clocks are needed for higher framerates and resolutions. 

{{Anchor|REFRESH_RATE_REQUIRED}}
'''REFRESH_RATE_REQUIRED '''
:The amount of fields per second in Hz. This is also the number of frames per second for non-interlaced modes, or twice the number of frames per second for interlaced modes.

{{Anchor|H_FREQ_REQUIRED}}
'''H_FREQ_REQUIRED'''
:The desired horizontal frequency in Hz. In the most common use, you'll want this to be the maximum horizontal frequency of the monitor, to find the best refresh rate possible that doesn't violate monitor limitations. 

==== Outputs ====
The final results of the GTF formula. These correspond to the parameters of the VGA CRTC model.

{{Anchor|PIXEL_FREQ}}
'''PIXEL_FREQ'''
:The dot clock required (check units?)

{{Anchor|H_TOTAL}}
'''H_TOTAL'''
:Amount of pixels in a scanline

{{Anchor|LEFT_MARGIN_PIXELS}}
'''LEFT_MARGIN_PIXELS '''
:Amount of margin at the left

{{Anchor|RIGHT_MARGIN_PIXELS}}
'''RIGHT_MARGIN_PIXELS '''
:Amount of margin at the right

{{Anchor|H_SYNC}}
'''H_TOTAL'''
:The duration of horizontal sync in pixels

{{Anchor|H_BACK_PORCH}}
'''H_BACK_PORCH'''
:The amount of pixels between the horizontal sync pulse and the start of the next scanline

{{Anchor|H_FRONT_PORCH}}
'''H_BACK_PORCH'''
:The amount of pixels between the end of active display and the start of the horizontal sync pulse

{{Anchor|TOTAL_V_LINES}}
'''TOTAL_V_LINES'''
:The amount of scanlines in a frame

{{Anchor|TOP_MARGIN_LINES}}
'''TOP_MARGIN_LINES '''
:Amount of margin at the top. 

{{Anchor|BOTTOM_MARGIN_LINES}}
'''BOTTOM_MARGIN_LINES '''
:Amount of margin at the bottom

{{Anchor|V_BACK_PORCH}}
'''V_BACK_PORCH'''
:The amount of scanlines between the Vertical Sync pulse and the start of the next field

{{Anchor|V_FRONT_PORCH}}
'''V_FRONT_PORCH'''
:The amount of scanlines between the end of the field and the start of vertical retrace

The remaining three components needed to generate a complete signal are inputs. For reference, these are: [[#H_PIXELS|H_PIXELS]], [[#V_LINES|V_LINES]] which form the resolution, and [[#V_SYNC_RND|V_SYNC_RND]], which is a constant for a given monitor.

==== Intermediates ====
Other results that are either byproducts, but may be useful for error and compatibility checking.

{{Anchor|V_LINES_RND}}
'''V_LINES_RND '''
:Amount of active display scanlines per frame. Equals [[#V_LINES|the Y resolution]] unless interlaced modes are used.

{{Anchor|H_PIXELS_RND}}
'''H_PIXELS_RND '''
:Actual horizontal resolution used. Basically contains [[#H_PIXELS|the X resolution]] rounded to the nearest allowed boundary. 

{{Anchor|V_FIELD_RATE_REQUIRED}}
'''V_FIELD_RATE_REQUIRED '''
:Rate of complete images being sent to the monitor per second. Unless interlaced mode is used, equals the refresh rate.

{{Anchor|H_FREQ}}
'''H_FREQ'''
:The horizontal frequency in Hz

{{Anchor|H_PERIOD}}
'''H_PERIOD'''
:The time in seconds spent on a scanline

{{Anchor|H_PERIOD_ESTIMATE}}
'''H_PERIOD'''
:The initial estimate of time in seconds spent on a scanline

{{Anchor|INTERLACE}}
'''INTERLACE'''
:Interlaced modes are indicated by skipping half a scanline, so that the horizontal sync shifts half a period. Equals 0 or 0.5 depending on whether the interlace signal is included.

{{Anchor|V_SYNC_AND_BACK_PORCH}}
'''V_SYNC_AND_BACK_PORCH'''
:The number of scanlines spent on VSync and bottom blanking

{{Anchor|V_FIELD_RATE_ESTIMATE}}
'''V_FIELD_RATE_ESTIMATE'''
:The initial estimate of fields per second (check unit?)

{{Anchor|V_FIELD_RATE}}
'''V_FIELD_RATE'''
:The number of fields per second (check unit?)

{{Anchor|V_FRAME_RATE}}
'''V_FRAME_RATE'''
:The number of frames per second (check unit?)

{{Anchor|TOTAL_ACTIVE_PIXELS}}
'''TOTAL_ACTIVE_PIXELS'''
:The amount of horizontally visible pixels, which equals the active display plus overscan

{{Anchor|IDEAL_DUTY_CYCLE}}
'''IDEAL_DUTY_CYCLE'''
:Percentage. One of the steps to compute the amount of horizontal blanking.

{{Anchor|IDEAL_H_PERIOD}}
'''IDEAL_H_PERIOD'''
:Optimal time to spend on a scanline. This can usually not be achieved since a scanline is a integer number of pixels.

{{Anchor|H_BLANK_PIXELS}}
'''H_BLANK_PIXELS'''
:The total amount of blank pixels horizontally (including horizontal sync)

=== Brendan's Sidenotes ===
Quoted:

I think part of the problem is that the original GTF formulas from VESA are copied from their spreadsheet, so that (for a silly example) instead of doing something like:

 frequency = 10000000
 period = 1 / frequency seconds

They'll do:

 frequency_in_MHz = 10
 period_in_ms = 1 / (frequency_in_MHz * 1000000) * 1000

And then simplify it to:

 frequency_in_MHz = 10
 period_in_ms = 1 / frequency_in_MHz / 1000

And then they'll remove any indication of what they've done:

 frequency = 10
 period = 1 / frequency / 1000

So that in the end, the formulas from VESA end up being a confusing mess where you're never too sure if there's a hidden scaling factor or not. Another simple example (that isn't made up) is "MARGINS_PERCENT", which is a value that would range from 0 to 50 that's typically divided by 100; and isn't a value that ranges from 0 to 0.5 that doesn't need scaling.

I plan on going through the formulas again, and making sure that all variables are either in pixels, lines, seconds or hertz (with no implied/hidden milliseconds, microseconds, kilohertz, megahertz or "percent" scaling factors).

I've also noticed that some of the calculations are the same in each formula. Rather than having 3 independent/larger pieces of code, it can be split into 3 smaller pieces of code that all use the same subroutines for common things. For example, you'd be able to replace about 10 lines from each formula with "call do_precalculations" to avoid code duplication (triplication?).

Also, I only posted "primary formulas". There's also a "secondary formula" where the results from the first set of calculations are used to find additional parameters; and there's another calculation (called the "blanking formula") for finding the values for "C_PRIME" and "M_PRIME" from some variables called C, M, J and K that's used for "secondary timings". I want to do the same for these formulas as I'm doing for the primary calculations. Then I need to go through all of these parameters and figure out which ones are actually useful (both for setting a video mode using VBE, and for setting a video mode in a "bare metal" video driver), and then work backwards removing redundant calculations (i.e. calculations that are only used to create information that you don't need anyway). After I've done this, I can combine it with the "common subroutines" idea, so that the entire "secondary formula" ends up being part of the "do_postcalculations()" routine, and so that the entire blanking formula becomes part of the "do_precalculations()" routine.

Then I want to find the valid ranges for all variables used in all calculations. This is mostly because of how I'm planning to implement it. For example, if I knew that "H_FREQ" ranges from 50000 to 1000000 then I'd be able to use "20:12" fixed point for this variable to improve the accuracy I get from a 32-bit integer.

Finally, all of these calculations can be simplified a lot if you assume that the "default GTF values" are being used. I want to provide simplified versions of the formulas, because most of the time you can use the default GTF values and can skip a lot of work.

== See Also ==

=== Forum ===
* [[Topic:19119|Getting a list of usable video modes]] - with emphasis on ''usable'' 

=== External Links ===
* [http://www.cs.unc.edu/Research/stc/FAQs/Video/GTF_V1R1.xls http://www.cs.unc.edu/Research/stc/FAQs/Video/GTF_V1R1.xls] is one of the GTF spreadsheets you can find on the web.
* [http://en.wikipedia.org/wiki/VGA_connector VGA connector] on wikipedia
* [http://en.wikipedia.org/wiki/EDID EDID] on wikipedia

[[Category:Video]]
````

<a id="w002535"></a>

## W002535 — Category:NetworkProtocols

Bron-ID: W002535
Namespace: Category (14)
Revisie: 7592 | Datum: 2009-04-06T22:21:15Z
Bron: https://wiki.osdev.org/Category:NetworkProtocols
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ANetworkProtocols&oldid=7592
Doorverwijzing: Category:Network Protocols

````text
#REDIRECT [[:Category:Network Protocols]]
````

<a id="w002536"></a>

## W002536 — Category:Networking

Bron-ID: W002536
Namespace: Category (14)
Revisie: 7549 | Datum: 2009-04-04T03:45:38Z
Bron: https://wiki.osdev.org/Category:Networking
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ANetworking&oldid=7549

````text
This category is a catch-all for all network-based content. You can find information about hardware, protocols, and other useful concepts here.

[[Category:OS theory]]
````

<a id="w002537"></a>

## W002537 — Category talk:Network devices

Bron-ID: W002537
Namespace: Category talk (15)
Revisie: 7593 | Datum: 2009-04-06T22:50:03Z
Bron: https://wiki.osdev.org/Category_talk:Network_devices
Revisie-URL: https://wiki.osdev.org/index.php?title=Category_talk%3ANetwork_devices&oldid=7593

````text
This page could be deleted - anyone? --[[User:Pcmattman|pcmattman]] 03:38, 4 April 2009 (UTC)
:'''Agreed.''' --[[User:Troy martin|Troy Martin]] 16:35, 4 April 2009 (UTC)
:'''Later''' - fix [http://wiki.osdev.org/Special:Whatlinkshere/Category:Network_devices the links] first - [[User:Combuster|Combuster]] 22:50, 6 April 2009 (UTC)
````

<a id="w002538"></a>

## W002538 — Network Stack

Bron-ID: W002538
Namespace: Main (0)
Revisie: 23577 | Datum: 2019-04-16T19:16:04Z
Bron: https://wiki.osdev.org/Network_Stack
Revisie-URL: https://wiki.osdev.org/index.php?title=Network_Stack&oldid=23577

````text
{{In_Progress}}
This article is about writing a TCP/IP stack, ie. a subsystem which uses a link layer (eg. ethernet card) to process packets of such protocols as IP, [[Address Resolution Protocol|ARP]], TCP, UDP.

== Scanning the PCI devices ==

The first thing to do is to scan the PCI devices installed on the machine so you can detect an Ethernet card by looking at a specific vendor ID and device ID. See the [[PCI]] page for more details.

== Writing a driver for your NIC ==

Once you have located the Ethernet card(s), you will need to implement a driver for it to be able to send and receive data. If you are using an emulator, a good card to write a driver for is the Intel E1000 as it is available on a variety of emulators such as VirtualBox - and has a thorough coverage on osdev.org (see [[Intel Ethernet i217]]). If you have trouble implementing the E1000 driver, you can start with the [[RTL8139]], an older ethernet card that is much simpler.

The first thing to get out of the Ethernet card is the machine's MAC address. This 6-bytes address is needed to exchange data on the local network.

The easiest test you can do is to send an ARP broadcast on the network. You can use [https://www.wireshark.org/ Wireshark] both to capture an example of a valid ARP request and to verify your own request has been received by the target host. As far as receiving data, your network card should capture data sent across the local network, even if it is not addressed to your machine.

== Networking protocols ==

Once you can send and receive data through your NIC and have your machine's MAC address, you will have to implement (at least partially) several networking protocols that coexist on top of each other:

* Ethernet: this is the basic protocol that sends data to another machine on your local network using your MAC address. This is the building block for all the rest as you need to send data to the router if you want to communicate with the outside world.
** [[ARP]] (Address Resolution Protocol): allows to translate an IPv4 address into a MAC address
** IP (Internet Protocol): this sits on top of Ethernet and is required to send data on the Internet given an IP address. The mostly common version is IPv4 which uses a 32-bit IP address, but IPv6 (which is using 128-bit IP addresses) is gaining some traction. Note that IP provides a "best effort" to send a packet, but does not guarantee it will successfully reach its destination, nor that the packets will be received in the order they were sent
*** [[ICMP]] (Internet Control Message Protocol): used by tools such as ping or traceroute
*** UDP (User Datagram Protocol): a connectionless transmission protocol that adds the notion of source and target ports to IP. Application services can subscribe to one or more port(s) to be notified if a UDP message is sent to that port
**** DHCP (Dynamic Host Configuration Protocol): allows to request the machine network configuration information such as its IP address, the IP address of the local router, the DNS, etc.
**** DNS (Domain Name System): get the IP address for a given domain name
*** TCP (Transmission Control Protocol): like UDP, it adds the notion of source and destination port. TCP is however more complex as it creates its own session mechanism and makes sure that the application using it will receive the packets in order, resending packets if need be. 
**** [[SSL/TLS]] (optional): if you want to use a secure connection
***** HTTP (HyperText Transfer Protocol): defines a request and response mechanism to transfer web pages, images and other resources.
***** Telnet: a protocol to remotely access a machine using a command line shell.

A tool of choice to help you will be [https://www.wireshark.org/ Wireshark], a free network sniffer and analyzer. It is a great tool to understand how the various networking protocols are encoded as it explains in great details what each byte of a packet corresponds to. Note that on Windows, Wireshark does not capture the loopback traffic (i.e. traffic made from localhost to localhost), so may not capture network traffic between an emulator and the host machine. You can however use Rawcap to capture the networking traffic into a file and use Wireshark to examine it.

== The network stack ==

Networking protocols are organized as a stack where each layer calls the next layer. A packet sent across the network will be composed of several headers, one for each layer involved.

Consider the example of a DHCP request. This is one of the protocols you might want to implement early on as it allows your machine to find its IP address, get the local router IP address, the DNS IP address - the basic information to be able to properly communicate across the network.

One way to implement this is as follows:

* The Operating System decides to send a DHCP request, so calls the DHCP layer
** The DHCP layer asks the UDP layer to create a packet whose target is IP address 255,255,255,255 (broadcast to the whole local network), port 53, and whose payload size is 300 bytes (the length may vary)
*** The UDP layer asks the IP layer to create a packet of type UDP to IP address 255,255,255,255, of size 308 bytes
**** The IP layer asks the Ethernet layer to create a packet of type IPv4 of length 328 bytes whose target is IP address 255,255,255,255
***** The Ethernet layer creates a packet of size 342 bytes, and writes in the first 14 bytes the Ethernet header, including the source address (the machine's MAC address), the destination MAC address FF:FF:FF:FF:FF:FF (translated from the IP address 255,255,255,255) and sends it back to the IP layer
**** The IP layer writes the IP header in the 20 bytes after the Ethernet header and sends it to the UDP layer
*** The UDP layer writes its header in the 8 bytes after the IP header and sends it to the DHCP layer
** The DHCP layer writes its request in the 300 bytes left and sends it back to the UDP layer
*** The UDP layer completes its header by writing its checksum (which encompasses the DHCP message) and sends it to the IP layer
**** The IP layer sends it to the Ethernet layer
***** The Ethernet layer sends the packet to the Ethernet card, that sends the message across the network

The packet actually sent across the network will look like:

{| {{wikitable}}
|-
| Ethernet header (14 bytes)
|-
| IPv4 header (20 bytes)
|-
| UDP header (8 bytes)
|-
| The DHCP request (300 bytes payload)
|}

The DHCP response will have the same format as the request, and should be processed as follows:
* The Ethernet card driver will verify that the target MAC is the current machine's, and if so sends the packet to the Ethernet layer
* The Ethernet layer will look at the Ethernet header, check the service type (which should be IP) and will send the packet (stripped of its Ethernet header) to the IP layer
* The IP layer will check the IP header, verify the checksum and, because its type is UDP will forward the packet (without its IP header) to the UDP layer.
* The UDP layer will check the UDP header, verify the checksum, and based on the destination port will send the payload to right service - in this example the DHCP layer (once again stripping the UDP header)
* The DHCP layer will read the DHCP message verify that the message type is Response (i.e. It's a response from the router) and will retrieve its IP address, the router's IP address and other networking configuration information.

Note that networking protocols are by definition asynchronously i.e. you send a request on the network and you need to wait for its response. In particular, you have no way of predicting when will a response arrive, if at all. And because an incoming packet is handled by an interrupt handler, it could interrupt your code at any time.

== Little and big endian ==

By convention, any message encoded on the Internet is using big endian (the most significant byte goes first). This is something to always keep in mind for people developing on Intel and AMD processors as x86 processors encode numbers using little endian. As a result, you will have to often convert numbers. Here are two functions to convert the endian for 16 and 32-bit integers:

    uint16_t switch_endian16(uint16_t nb) {
        return (nb>>8) | (nb<<8);
    }
    
    uint_t switch_endian32(uint_t nb) {
        return ((nb>>24)&0xff)      |
               ((nb<<8)&0xff0000)   |
               ((nb>>8)&0xff00)     |
               ((nb<<24)&0xff000000);
    }

== Checksums ==

Several networking protocols use a checksum to verify that the message was not accidentally altered during the transport. Without a valid checksum, the packet is likely to be ignored. The checksum is a 16-bit number computed as follows:
* Split the message to checksum into 16-bit chunks
* Add those chunks
* If the message has an odd number of bytes, the last byte should be counted as the higher byte (e.g. if the last byte is 0x42 then add 0x4200)
* If the sum does not fit in a 16-bit number (i.e. is greater than 0xFFFF), strip the top 16 bits and add them to the low 16 bits. Repeat the last step until you have a 16-bit sum
* Return the binary inverse of that sum

The IP checksum only covers its own header. The UDP and TCP checksums are a bit more complex as they include the UDP/TCP header, the payload (i.e. anything after the UDP/TCP header) as well as a "pseudo header" composed of the source and target IP addresses, the IP type (0x11 for UDP, 0x06 for TCP) and the UDP/TCP message length (starting with the UDP/TCP header).

If properly computing the checksum can be tricky, Wireshark can help you. For this, make sure that it is verifying the checksum (an option not enabled by default) by going to Edit / Preferences / Protocols, select the desired protocol (e.g. UDP, TCP, IPv4) and make sure that "Validate the checksum if possible" is checked. This way, Wireshark will tell you if the checksum is valid, and if not, what its value should be.

== ARP ==

The ARP protocol will be one of the first protocols you will need to implement. Without it, you will not be able to communicate on your local network, let alone on the Internet. Fortunately this is a simple protocol which only requires to implement a few functions:
* Sending requests and processing replies: your OS will need to perform a request to convert an IP address into a MAC address, something which is required to even communicate with your local router. This implies not only sending a request packet but also processing the reply when it comes so your OS can update its ARP table
* Receiving requests and sending replies: your OS will also need to honor the requests sent its way (e.g. when someone asks what is its MAC address). In particular, the local router will send an ARP requests to your machine on a regular basis. Failure to respond will have the router consider your machine is down, and won't forward any more traffic to it

== TCP ==

TCP is one of the most complex networking protocols.

First of all, it creates a virtual connection between the client and the server. To achieve this, a TCP header contains multiple flags that will be used by both sides to communicate about the status of that connection: SYN (synchronize), ACK (acknowledge), PSH (Push), FIN (finish) and others.

On top of that, TCP is trying to alleviate the fact that IP does not guarantee that packets will be received in the order they were sent, let alone received at all. This is why it keeps track of the amount of data actually sent, requires each side to regularly acknowledge the data they have received, and will have packets resent if need be. For this, a TCP header contains a sequence number and an acknowledgement number.

In the course of a TCP connection, both sides send each other some data, split across multiple packets. One way to measure where the communication stands is to send the position (in number of bytes) in that communication. The sequence number in a TCP packet is the position the current packet is at. Likewise, the acknowledgement number indicates the position (still in bytes) where one party expects the other party to send.

When either side receives a TCP packet with a sequence number S, an acknowledgement number A and a payload of size N, its next packet it sends should have the sequence number A (i.e. it's sending the data the other party expects) and the acknowledgement number S + N (or S+1 if N is null).

=== Establishing a connection ===

A TCP connection is established with the following 3-way handshake:
* The client sends a SYN request to the server (i.e. a message with the SYN flag set)
* The server responds with an SYN+ACK request (the standard also allows it to send an ACK and SYN separately, though that rarely happens).
* The client sends an ACK response.

The sequence number used in the SYN packets is the initial sequence number; all further packets shall use sequence numbers that are increments of the initial sequency number.  The sequence number can be reset by sending a new SYN packet with a new sequence number.

The initial SYN and SYN+ACK packets "may" also contain data to be sent to the application, but this is rarely used.  The TCP specification states that this data shall not be delivered to the application until the connection is established (i.e. after the final ACK response packet is received).

=== Transmitting data ===

To send data, either side can send a PSH, ACK message, with the actual data after the TCP header. The other party will need to send an ACK message to acknowledge it has received the packet. If not, the sender will send again the packet. This is where multiple TCP/IP implementations differ - some may wait more or less long before sending an ACK.

=== Closing the connection ===

The termination of the connection :
* The side that wants to close the connection sends a packet with a FIN flag
* The other side sends a FIN, ACK message
* The first side sends an ACK message

Like for the packets used to establish a connection, those packets do not contain any payload - just a TCP header.

=== An example ===

Let's look at an example of a TCP communication for an HTTP GET request:

{| {{wikitable}}
|-
! Source -> Destination
! Destination -> Source
! Comment
|-
| Flag: SYN
seq_nb=0, ack_nb=0
|
| Beginning of the TCP handshake. It is sending byte #0 (a packet without a payload will be considered to have at least one byte communication) and hasn't received any data yet from the server
|-
| 
| Flags: SYN, ACK
seq_nb=0, ack_nb=1
|
|-
| Flag: ACK
seq_nb=1, ack_nb=1
| 
| The TCP handshake is completed, the communication can start
|-
| Flags: PSH, ACK
seq_nb=1, ack_nb=1, len=77
| 
| This is the HTTP GET request sent by the client. This is the first packet with an actual payload
|-
| 
| Flag: ACK
seq_nb=1, ack_nb=78
| The server acknowledges the HTTP request: it has successfully read up to byte #77, so expects the next communication to start at byte #78
|-
| 
| Flags: PSH, ACK
seq_nb=1, ack_nb=78, len=1009
| This is the body of the HTML
|-
| Flag: ACK
seq_nb=78, ack_nb=1010
| 
| The client acknowledges the message sent by the server: it is sending byte #78, and has receveid up to byte #1009 so expects the next communication to start at byte #1010
|-
| Flags: FIN, ACK
seq_nb=78, ack_nb=1010
| 
| The client terminates the TCP connection
|-
| 
| Flags: FIN, ACK
seq_nb=1010, ack_nb=79
| 
|-
| Flag: ACK
seq_nb=79, ack_nb=1011
| 
| The end of the TCP communication
|}

== What to focus on ==
The shape of the stack will vary on design decisions. These may include
* whether or not a packet is passed between processing layers in one buffer or is copied to a new buffer when passing a layer boundary;
* whether in and outbound frames are communicated with the link layer with the use of a dedicated thread, are fully contained in an interrupt handler or in a loop in a single-threaded environment;
* whether frames (eg. ethernet frames) are processed immediately or queued;
* whether you want TCP support or just UDP or maybe only IP support; TCP is the most complex part of the stack, in the lwip implementation half of the code is specific to TCP.

As an example, a stack might
* have the NIC's API provide three functions: setting up the NIC, poll for a frame and send a frame;
* communicate in and outbound frames to the NIC in a one thread;
* demultiplex inbound frames from a reception queue in another thread.

== General considerations ==
* When writing a stack over an ethernet, you may want to provide support for the ARP protocol and resolve functions.
* For the sake of modularity, the station's IP would be better stored in an nic_info struct rather than as a global variable.
* You may want to use Wireshark or another packet sniffer to inspect the communication and netcat which would dump debugging data sent from your OS once you have UDP or TCP support. Also, arping is useful when debugging arp code. You may code a trigger which for example reboots your system upon receipt of an ARP who-has for a chosen IP.
* You may use a dedicated ethernet card on one computer connected with a crossed over cable to another computer (which runs your operating system) and use static IP. Other options include testing under bochs or qemu after implementing drivers for the network devices they provide.

== See Also ==
=== Articles ===
* [[Diskless Booting]]
* [[:Category:Network_Hardware|Network Hardware]]

=== Threads ===

=== External Links ===
* [http://focus.ti.com/lit/an/slaa137a/slaa137a.pdf Details on a implementation of a embedded web server with overview of the hardware, TCP/IP suite, TCP/IP stack and its API.]
* [http://www.faqs.org/rfcs/rfc793.html RFC 793 - Transmission Control Protocol]
* [http://www.amazon.com/TCP-Illustrated-Protocols-Addison-Wesley-Professional/dp/0201633469/ref=pd_bbs_sr_1?ie=UTF8&s=books&qid=1229662804&sr=8-1 TCP/IP Illustrated - A must have book for any type of networking, great reference book]

A number of tcp/ip stacks come with a documentation of their implementations; it makes a good read.
* [http://www.cs.northwestern.edu/~pdinda/minet/ Minet: A User-level TCP/IP Stack]
* [http://www.sics.se/~adam/uip/index.php/Main_Page uIP tiny stack for 8-bit embedded microcontrollers]
* [https://lwip.fandom.com/wiki/LwIP_Wiki lwIP light-weight implementation]

[[Category:Networking]]
````

<a id="w002539"></a>

## W002539 — Address Resolution Protocol

Bron-ID: W002539
Namespace: Main (0)
Revisie: 11170 | Datum: 2011-02-05T23:24:22Z
Bron: https://wiki.osdev.org/Address_Resolution_Protocol
Revisie-URL: https://wiki.osdev.org/index.php?title=Address_Resolution_Protocol&oldid=11170

````text
Address Resolution Protocol, commonly referred to as "ARP", is a protocol that permits hosts to gain the ethernet address (MAC) of a remote host when they only have an [[IPv4]] address. Technically, ARP supports other address formats due to a variable size data format, however it is typically used with [[Ethernet]] and IP addresses.

It should be noted that similar functionality is provided in IPv6 by a different protocol.

== Background ==

ARP is defined in [http://tools.ietf.org/html/rfc826 RFC826]. It runs over the link layer.

== Packet Format ==

'''** Note **'''

All integer literals should be assumed to be in little-endian notation. When sending packets you will need to convert 16-bit and 32-bit integers to big-endian.

==== C-style Packet Format ====

<pre>
struct arp
{
    u16 htype; // Hardware type
    u16 ptype; // Protocol type
    u8  hlen; // Hardware address length (Ethernet = 6)
    u8  plen; // Protocol address length (IPv4 = 4)
    u16 opcode; // ARP Operation Code
    u8  srchw[hlen]; // Source hardware address - hlen bytes (see above)
    u8  srcpr[plen]; // Source protocol address - plen bytes (see above). If IPv4 can just be a "u32" type.
    u8  dsthw[hlen]; // Destination hardware address - hlen bytes (see above)
    u8  dstpr[plen]; // Destination protocol address - plen bytes (see above). If IPv4 can just be a "u32" type.
};
</pre>

==== Description ====

''' Hardware Type '''

The type of the hardware layer over which the ARP packet is being sent. Ethernet is 0x1, and you shouldn't need any other value unless you're working with PPP or something.

''' Protocol Type '''

The type of the protocol address that the ARP request uses. IP is 0x0800. Again, you shouldn't need another value here unless you have unusual requirements (in which case you'll hopefully know what you're doing anyway).

''' Hardware Length '''

The length of the hardware address for the type of hardware layer. MAC addresses are 6 bytes in length.

''' Protocol Length '''

The length of the protocol address for the protocol layer. IPv4 addresses are 4 bytes in length.

''' Operation Code '''

The operation to perform.

''' Source/Dest Hardware Address '''

Hardware addresses of the source and destination hosts. For an ARP request most implementations zero the destination MAC address.

''' Source/Dest Protocol Address '''

Protocol addresses of the source and destination hosts.

=== Operations ===

=== ARP Request ===

An ARP request allows a host to find the MAC address for a host whose IP is known. It's operation code is 0x0001.

=== ARP Reply ===

An ARP reply is the response to an ARP request. It's operation code is 0x0002.

[[Category:Network Protocols]]
[[de:Address Resolution Protocol]]
````

<a id="w002540"></a>

## W002540 — Talk:AGP

Bron-ID: W002540
Namespace: Talk (1)
Revisie: 7597 | Datum: 2009-04-06T22:59:31Z
Bron: https://wiki.osdev.org/Talk:AGP
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AAGP&oldid=7597

````text
There is only mention is this article about AGP and graphics cards. Does anybody know if this same procedue is used for speeding up 66MHz PCI cards, or whether they are already fast. In addition, what about PCI Express cards that aren't graphics cards (4x speed RAID cards and so on). [[User:Jackscott|JackScott]] 23:51, 4 April 2009 (UTC)

I don't know the details of PCIE cards, but they are a completely different standard. AGP programming won't help you there. - [[User:Combuster|Combuster]] 22:59, 6 April 2009 (UTC)
````

<a id="w002542"></a>

## W002542 — Category:Network Protocols

Bron-ID: W002542
Namespace: Category (14)
Revisie: 7591 | Datum: 2009-04-06T22:20:41Z
Bron: https://wiki.osdev.org/Category:Network_Protocols
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ANetwork_Protocols&oldid=7591

````text
This category contains pages that explain various network protocols.

[[Category:Networking]]
````

<a id="w002543"></a>

## W002543 — Multi Processors

Bron-ID: W002543
Namespace: Main (0)
Revisie: 7600 | Datum: 2009-04-07T10:15:00Z
Bron: https://wiki.osdev.org/Multi_Processors
Revisie-URL: https://wiki.osdev.org/index.php?title=Multi_Processors&oldid=7600
Doorverwijzing: Symmetric Multiprocessing

````text
#REDIRECT [[Symmetric Multiprocessing]]
````

<a id="w002544"></a>

## W002544 — Talk:Multi Processors

Bron-ID: W002544
Namespace: Talk (1)
Revisie: 7602 | Datum: 2009-04-07T10:15:01Z
Bron: https://wiki.osdev.org/Talk:Multi_Processors
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMulti_Processors&oldid=7602
Doorverwijzing: Talk:Symmetric Multiprocessing

````text
#REDIRECT [[Talk:Symmetric Multiprocessing]]
````

<a id="w002545"></a>

## W002545 — SMP

Bron-ID: W002545
Namespace: Main (0)
Revisie: 7603 | Datum: 2009-04-07T10:17:52Z
Bron: https://wiki.osdev.org/SMP
Revisie-URL: https://wiki.osdev.org/index.php?title=SMP&oldid=7603
Doorverwijzing: Symmetric Multiprocessing

````text
#REDIRECT [[Symmetric_Multiprocessing]]
````

<a id="w002546"></a>

## W002546 — Category:Network Hardware

Bron-ID: W002546
Namespace: Category (14)
Revisie: 27489 | Datum: 2022-11-15T21:25:54Z
Bron: https://wiki.osdev.org/Category:Network_Hardware
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ANetwork_Hardware&oldid=27489

````text
This category lists devices that allow communication between computers.
===Helpful Resources===

These are links or documents that can help a lot with networking hardware.

* [http://www.cpt.univ-mrs.fr/info/html/Ethernet-HOWTO-5.html Overview of NICs] [http://web.archive.org/web/20061208012137/http://www.cpt.univ-mrs.fr/info/html/Ethernet-HOWTO-5.html via internet archive]

[[Category:Hardware]]
[[Category:Networking]]
````

<a id="w002547"></a>

## W002547 — Porting Python

Bron-ID: W002547
Namespace: Main (0)
Revisie: 30384 | Datum: 2026-03-23T13:51:48Z
Bron: https://wiki.osdev.org/Porting_Python
Revisie-URL: https://wiki.osdev.org/index.php?title=Porting_Python&oldid=30384

````text
{{Rating|4}}
{{In_Progress}}
{{FirstPerson}}

:''This article is a bit out of date, poll [[User:Sortie]] or check [https://pub.sortix.org/sortix/release/nightly/patches/python.patch his patch to Python 3.4].

Python is a popular scripting language that, if ported, can allow you to do a lot of powerful tasks using simple scripts. I gained a lot from [http://www.ailis.de/~k/archives/19-ARM-cross-compiling-howto.html this] and [http://whatschrisdoing.com/blog/2006/10/06/howto-cross-compile-python-25/ this]. Amazing what a simple Google search can do!

The problem with porting Python is that the Python build process actually wants to use the built Python to build the rest of the modules that come with Python (eg, C modules). The build system also does not build build tools with the --build compiler, which means the build can fail quite early if you aren't prepared.

== Prerequisites ==

Porting Python is '''not''' for the faint of heart. You should know a lot about configure scripts and the options of your toolchain.

You will also need a cross toolchain that supports C++ (See [[OS_Specific_Toolchain|OS Specific Toolchain]]) and has a C++ standard library.

== Freebies ==

I have the patches I use for Python 2.6, Python 2.7 and Python 3.10 here:
* [http://www.pedigree-project.org/projects/pedigree-apps/repository/revisions/master/show/packages/python26/patches Python 2.6]
* [http://www.pedigree-project.org/projects/pedigree-apps/repository/revisions/master/show/packages/python27/patches Python 2.7]
* [https://github.com/moisam/laylaos/tree/main/ports/python Python 3.10]

In the directory above the patches directory each link links to, you can find the scripts I use to automate the Python build.

== The Process (for Python3) ==

Download the Python 3.10 source code. You need to patch the '''configure''' script in a way similar to what is done in the patch linked above. You also need to modify '''config.sub''' so it recognizes your OS.

Now you can run '''configure''':
<syntaxhighlight lang="bash">
./configure \
    --host=i686-myos --target=i686-myos \
    --build=i686-pc-linux-gnu \
    --disable-ipv6 --enable-shared \
    --with-system-expat --with-system-ffi \
    --prefix=/usr --with-pymalloc=no
</syntaxhighlight>

Substituting '''i686-myos''' with the architecture and name of your OS. If you are not building on a GNU/Linux system, you also need to change the argument to the '''--build''' option accordingly.

I do not have a working '''utimensat()''' function in my C library, so I run '''sed''' to comment it out in the generated '''pyconfig.h''' file:
<syntaxhighlight lang="bash">
sed -i "s~#define HAVE_UTIMENSAT 1~//#define HAVE_UTIMENSAT 1~" ./pyconfig.h
</syntaxhighlight>

Now you are ready to build and install:
<syntaxhighlight lang="bash">
make && make install DESTDIR=<install_dir>
</syntaxhighlight>

This will install the '''python''' binary in the specified installation directory. Many 3rd party programs expect to find a '''python3''' binary, which you can easily make by creating a symbolic link:
<syntaxhighlight lang="bash">
ln -s python3 <install_dir>/usr/bin/python
</syntaxhighlight>

And you're done!

== The Process (for Python2) ==

Download the Python 2.5 source code. You need to patch it using [http://whatschrisdoing.com/~lambacck/Python2.5_xcompile.patch this patch] in order to make the configure process work properly for the cross-compile.

Basically, this patched configure script now uses two variables: HOSTPYTHON and HOSTPGEN in the later stages of the build. See, Python's build process builds Python itself, then uses the built Python binaries to build the modules. In the case of a cross-compiler, this won't work, so we need to point Python to a working Python binary.

Run:
<syntaxhighlight lang="bash">
./configure && make python Parser/pgen
</syntaxhighlight>
in the Python-2.5 directory.

This will build a version of Python using your host compiler.

Now we need to make that the host Python and get ready for the real cross compile:
<syntaxhighlight lang="bash">
mv python hostpython
mv Parser/pgen Parser/hostpgen
make distclean
</syntaxhighlight>

This is where things get complicated. We now have to configure for the new target. Before you do this, export CC, CXX, AR and RANLIB for your target. In my case, I'll use "i686-myos":
<syntaxhighlight lang="bash">
./configure --host=i686-myos --prefix=/usr/python-cross
</syntaxhighlight>

If things go wrong, you need to make sure your libc and libstdc++ provides the necessary functionality.

At this stage, Python is configured to use your cross toolchain. Now we build Python:
<syntaxhighlight lang="bash">
make python Parser/pgen
</syntaxhighlight>

Right, that wasn't so hard, was it? So now you have the Python binary ready to run on your OS. However, it's missing libraries and modules. Let's build them now:
<syntaxhighlight lang="bash">
make HOSTPYTHON=./hostpython HOSTPGEN=./Parser/hostpgen BLDSHARED="i686-myos-gcc -shared" CROSS_COMPILE=yes
</syntaxhighlight>

You will need to edit "setup.py" to modify the default include and libraries directories to those which your compiler uses. See below ("setup.py") for the changes you need to make.

Basically, the Makefile now calls "$HOSTPYTHON ./setup.py" to continue the build. That's why we need to set HOSTPYTHON now. Another important thing to mention is BLDSHARED. If your cross-toolchain can't build shared objects, you need to use one that can. If you're on Linux and you're using ELF in your OS, just set BLDSHARED="gcc -shared". If not, you're out of luck until your cross-toolchain can build shared objects.

Your modules will now build. Keep track of error messages if you can, there will be modules that won't work until you add more functionality to your libc.

To install, run:
<syntaxhighlight lang="bash">
make install prefix=<install_dir> HOSTPYTHON=./hostpython HOSTPGEN=./Parser/hostpgen BLDSHARED="i686-myos-gcc -shared" CROSS_COMPILE=yes
</syntaxhighlight>

And you're done!

== setup.py ==

This is where things can get really complicated. setup.py is what Python uses to build the modules for your Python port - however it doesn't work so well for cross-compiling. I'll assume you've already applied the patch specified above, so let's get to it!

''' NOTE: Make sure you watch the surrounding indentation when making these changes. Python knowledge would really help in this case. A lot of these changes I'm putting up here without indentation! '''

The first change we make is at the line "disabled_module_list":
<syntaxhighlight lang="python">disabled_module_list = ['_ctypes', 'syslogmodule', 'nismodule']</syntaxhighlight>

This is pretty straightforward. If you're going to put anything here, make it '_ctypes'.

Find "def detect_modules(self)". See at the top how it has those two directories? Let's change that...
<syntaxhighlight lang="python">
#       add_dir_to_list(self.compiler.library_dirs, '/usr/local/lib')
#       add_dir_to_list(self.compiler.include_dirs, '/usr/local/include')
        add_dir_to_list(self.compiler.library_dirs, '/usr/pedigree-cross/lib')
        add_dir_to_list(self.compiler.library_dirs, '/usr/pedigree-cross/i686-pedigree/lib')
        add_dir_to_list(self.compiler.include_dirs, '/usr/pedigree-cross/include')
        add_dir_to_list(self.compiler.library_dirs, '/usr/pedigree-cross/i686-pedigree/include')
</syntaxhighlight>

Of course, you'll update these to the correct paths for your cross-toolchain. These will become not only the search paths for libraries but also the command line for the GCC invocation ("-I" and "-L" arguments).

Further down there's a block setting "lib_dirs" to a bunch of folders such as "/lib64", "/usr/lib64", "/lib", and "/usr/lib". Beneath it is a line that sets up the include directories including "/usr/include". Change this to:
<syntaxhighlight lang="python">
lib_dirs = self.compiler.library_dirs
inc_dirs = self.compiler.include_dirs
</syntaxhighlight>

And if we keep on heading down just a couple of lines there's a "platform = self.get_platform()". The platform is used to make some assumptions further down, so we should set it to something like our OS name:
<syntaxhighlight lang="python">
platform = "pedigree"
</syntaxhighlight>

Halfway there! Find the line "if platform == 'darwin':" and add above it something like this:
<syntaxhighlight lang="python">
        if platform == 'pedigree':
          lib_dirs += ['/usr/pedigree-cross/lib']
          inc_dirs += ['/usr/pedigree-cross/i686-pedigree/include/ncurses']
</syntaxhighlight>

This just adds some directories to the list that we need for the platform. I will admit this is slightly redundant for this example (apart from the ncurses headers) - feel free to skip this if you've already covered it above with the "add_dir_to_list" lines. If you have any extra directories that you want the compile to use you can add them here (eg, unusual libraries locations or include files).

If you have an OpenSSL port (and you can compile against it in your cross-toolchain), skip past this change.

If not, find the lines starting with "ssl_incs = find_file...." and change to:
<syntaxhighlight lang="python">
        ssl_incs = None
        #find_file('openssl/ssl.h', inc_dirs,
        #                     search_for_ssl_incs_in
        #                     )
</syntaxhighlight>

This just removes the SSL header from the search path, so you don't get a bunch of undefined references.

Now we're pretty much done. Keep going down until you find the Berkeley DB stuff (starts with "db_inc_paths", comment out all of the paths, keep the []. Comment out all four of the for() loops (they should be completely commented out). comment out the "for dn in inc_dirs" for block, and finally change the "db_inc_paths" variable as below:
<syntaxhighlight lang="python">
        db_inc_paths = [] # std_variants + db_inc_paths
</syntaxhighlight>

The last step is to comment out the sqlite include paths (search for sqlite_inc_paths, again, keep the []).

Save, exit your editor, and get compiling!

== Caveats ==

You will need dlopen, dlsym and dlclose to be able to do things like "import socket" which indirectly imports "_socket" (<prefix>/lib/python2.5/lib-dynload/_socket.so). By this stage you should already have shared object support so this should be simple!

== What if my host toolchain uses a different shared object format? ==

Let's say you're on Cygwin, and your toolchain targets ELF. You're a little bit stuck because you can't build any modules, because you can't define BLDSHARED to "gcc -shared" (you'd get a DLL), and your ELF toolchain may not support shared objects. There is, of course, a workaround for this. Your BLDSHARED will look a bit like this:
<pre>
BLDSHARED="i686-myos-gcc -nostdlib -shared -Wl,-shared"
</pre>

That should build a shared object that does not rely on any other libraries.

== Works On ==

I've done this on:
*  Python 2.2
*  Python 2.5
*  Python 2.5.4 (the patch does '''not''' work - you'll need to read through the patch file and make the changes manually.)
*  Python 2.6.2
*  Python 2.6.6
*  Python 2.7.3

== Final Word ==

Porting Python is really not that complex if you just work around some minor complexities that the build process uses. Good luck!

[[Category:Porting]]
[[Category:Python]]
[[Category:Tutorials]]
````

<a id="w002548"></a>

## W002548 — Template:Anchor

Bron-ID: W002548
Namespace: Template (10)
Revisie: 12731 | Datum: 2012-02-19T20:31:03Z
Bron: https://wiki.osdev.org/Template:Anchor
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AAnchor&oldid=12731

````text
<span id="{{{ 1|}}}"></span>
````

<a id="w002549"></a>

## W002549 — OpenBSD's bootloader

Bron-ID: W002549
Namespace: Main (0)
Revisie: 28000 | Datum: 2023-07-09T17:43:41Z
Bron: https://wiki.osdev.org/OpenBSD%27s_bootloader
Revisie-URL: https://wiki.osdev.org/index.php?title=OpenBSD%27s_bootloader&oldid=28000

````text
{{In Progress}}

The OpenBSD [[bootloader]] provides many features. It provides a decent way to obtain boot arguments; will load ELF kernel files; and will probe somethings for you. It probes memory and passes a fairly easy to follow memory map(that can be changed by the user manually at boot). It passes disk and com-port info, along with a few other things.

The only bad part about using the OpenBSD bootloader really is that it is has not been ported to any FS but FFS and FFS2. There are both i386 and amd64 version of the bootloader. I will be discussing the i386 version, though most of this should be shared with amd64.

[[Category:Bootloaders]]
````

<a id="w002550"></a>

## W002550 — Talk:Double Buffering

Bron-ID: W002550
Namespace: Talk (1)
Revisie: 28547 | Datum: 2024-01-04T21:52:42Z
Bron: https://wiki.osdev.org/Talk:Double_Buffering
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ADouble_Buffering&oldid=28547

````text
Double buffering is not difficult. The theory isn't difficult, the code isn't difficult, and it's amazingly easy to use. I fail to see why this has three blocks.
[[User:Jackscott|JackScott]] 23:25, 10 April 2009 (UTC)

:I originally put that rating on it not really because of the difficulty, but more because of the fact that it's not something programmers would implement when they're only just starting out. Anyhow, you can change the difficulty rating to whatever you like, I just put it there because it had to have something. -- [[User:Creature|Creature]] 11:39, 11 April 2009 (UTC + 1)

A thing in the next to last paragraph in the page, the stuff about 'Invalidated rectangles' got me thinking a little... Isn't that practically the same as keeping track of the maximum and minimum pixel coordinates (x,y) you have modified? Which in turn would mean that you have the coordinates for the rectangle which needs to be updated, and don't have to repaint the rest of the screen? Sounds like something that many might find useful and which wouldn't be too hard to code, so my question is, shall I sit down and write a C code snippet for it? Or maybe someone already has an appropriate snippet?
--[[User:Teodor väänänen|Teodor väänänen]] 16:47, 12 July 2009 (UTC)

== C/C++ examples ==

Is there really a need for two almost identical pieces of code just to show off reinterpret_cast and the new-operator? If you're writing an OS in C++, you should be familiar with these two anyway.
--[[User:Darkinsanity|Darkinsanity]] 19:17, 8 November 2014 (CST)

: Besides being redundant, it also makes the glaring error of using bits per pixel instead of bytes per pixel. I'll clean it up - [[User:Combuster|Combuster]] 04:37, 9 November 2014 (CST)

== Requesting an example for reading/modifying video memory using double buffering ==

The text mentions double buffering can be used for text scrolling, as an example. I am having trouble understanding how. If the front buffer is in video memory (slow) and the back buffer is in main memory (fast), and you need to read what is currently being displayed, how is that done without reading from video memory?
````

<a id="w002554"></a>

## W002554 — User:Imate900

Bron-ID: W002554
Namespace: User (2)
Revisie: 8584 | Datum: 2009-07-18T05:37:52Z
Bron: https://wiki.osdev.org/User:Imate900
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AImate900&oldid=8584

````text
Hello. I am imate900. I built up craps of Assembler from looking at everything: from Linux source code to x86test.<br>
sanity - pi * sanity + code / sanity = 908.07865645764501234578902893

I program [[User:Imate900/SauOS|SauOS]] (rather cryptic name, eh?) and [[User:Imate900/ROSS|ROSS]]. Once SauOS' new kernel is mature I'm going to start a Windows 95 compat OS called Freedom95.

== Bare Bones I made ==
* [[User:Imate900/32-bit assembler bare bones]]
* [http://asteinborn78.ej.am/codeblocks-ostemplate.zip Code::Blocks OS Template (goes with Bare Bones article)]
== Others ==
* [[User:Imate900/GDC_Cross-Compiler-Part2|A way of part2 in the GDC Cross-Compiler article]]
* [[User:Imate900/Memory_Functions|Memory functions (using lodsb and stosb)]]
* [[User:Imate900/String_Functions|String functions (length, copying and comparing so far)]]
````

<a id="w002555"></a>

## W002555 — User:Imate900/SauOS

Bron-ID: W002555
Namespace: User (2)
Revisie: 8543 | Datum: 2009-07-14T20:19:48Z
Bron: https://wiki.osdev.org/User:Imate900/SauOS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AImate900%2FSauOS&oldid=8543

````text
SauOS is a 32-bit UNIX-like OS booting from GRUB in C.

Currently It Is Adopting Code from ScorchOS (With Permission of the Author) and Having a Total Rewrite.
````

<a id="w002557"></a>

## W002557 — User:Troy martin/TBOS

Bron-ID: W002557
Namespace: User (2)
Revisie: 7845 | Datum: 2009-05-05T15:18:10Z
Bron: https://wiki.osdev.org/User:Troy_martin/TBOS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATroy_martin%2FTBOS&oldid=7845

````text
The '''Titanium Bonfire Operating System''' (TBOS) is a 16-bit, x86 operating system written in NASM-style assembly language. The project is open source, under the BSD license. The project's website is at http://www.tbos.co.nr/, with the Titanium Bonfire Project at http://www.tinyurl.com/tbproject.

Todo lists and such can always be found on the Titanium Bonfire Project website (you may have to search through the blog posts though!)
* Current version is 1.0.1, working on 1.1.0
* The next major version after 1.1.0 will be 1.2.0

TBOS is a purely monolithic and monotasking kernel. Plans for MS-DOS and possibly MikeOS program support have been made and the code is in the works.

==Known major bugs==
* fs_fat12_write will gladly overwrite (or duplicate) a file
* get_string can cause buffer overflows

==hash==

hash (Hardly Amazing SHell) is the built-in shell for TBOS. It's probably the biggest part of the kernel binary since it has 75% of the strings in it. hash is influenced by bash and borrows some ideas from bash and other GNU software, like long options. People who are native DOS users probably won't like working with hash because it's just plain different than the DOS command interpreter.

* Current version is 0.3.x, working on 0.4.0
````

<a id="w002558"></a>

## W002558 — User:Karekare0

Bron-ID: W002558
Namespace: User (2)
Revisie: 7690 | Datum: 2009-04-17T02:37:46Z
Bron: https://wiki.osdev.org/User:Karekare0
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKarekare0&oldid=7690

````text
I changed my username to [[User:Zenith]] a while back. Don't worry, I still frequent the forums/wiki.
````

<a id="w002559"></a>

## W002559 — User:Zenith

Bron-ID: W002559
Namespace: User (2)
Revisie: 7691 | Datum: 2009-04-17T02:38:06Z
Bron: https://wiki.osdev.org/User:Zenith
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AZenith&oldid=7691

````text
Hey! I'm an occasional visitor of OSDev.org, and I do a bit of work on the wiki as well.

My OS is called Synergy, and it has a dedicated website at [http://synergy.ifeelrandom.com synergy.ifeelrandom.com].

I was formerly known as Karekare0 on the forum/wiki.

== Articles I've Written ==
[[Creating a 64-bit kernel]]
````

<a id="w002560"></a>

## W002560 — Talk:Memory Map (x86)

Bron-ID: W002560
Namespace: Talk (1)
Revisie: 29314 | Datum: 2024-11-30T23:37:22Z
Bron: https://wiki.osdev.org/Talk:Memory_Map_(x86)
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMemory_Map_%28x86%29&oldid=29314

````text
Is it actually safe to assume that 0x100000-0xC0000000(other than the 15-16mb gap) is all free for use? shouldn't this be made sure of by a proper BIOS function or manual probing?--[[User:Earlz|Earlz]] 03:24, 17 April 2009 (UTC)

:You have just been found guilty of not reading the [[How_Do_I_Determine_The_Amount_Of_RAM|FAQ]] - [[User:Combuster|Combuster]] 17:05, 20 April 2009 (UTC)


----

Why does it talk about GRUB and bootloader modules if it's supposed to talk about the memory right after BIOS jumped to the bootloader code, it's confusing and misdirecting.
There should be a section maybe that talks about people using GRUB and add warnings abot places they shouldn't touch, but the main article is supposed to be for people creating their own bootloader.

== "There was a typo in the memory table under 1mb. Instead of 0x00007FFF it was 0x0007FFFF" Revert ==

[[User:Bl4ze4447|Bl4ze4447]], I approved the change but immediately reverted it as it's not spam but also it's incorrect -- The memory map in real mode goes from 0x00000 to 0xFFFFF (20 bits), your change of 0x7FFF is only 32k into memory. Please don't make edits without sourcing correct information (and checking your math) - [[User:Primis|Primis]] 23:36, 30 November 2024
````

<a id="w002562"></a>

## W002562 — User:Imate900/ROSS

Bron-ID: W002562
Namespace: User (2)
Revisie: 7707 | Datum: 2009-04-19T13:05:47Z
Bron: https://wiki.osdev.org/User:Imate900/ROSS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AImate900%2FROSS&oldid=7707

````text
ROSS is a 16-bit OS.
== Source ==
<pre>
  mov ax, 0x07C0  ; set up segments
  mov ds, ax
  mov es, ax

  mov si, welcome
  call print_string

loop:
  mov si, prompt
  call print_string

  mov di, buffer
  call get_string

  mov si, buffer
  cmp byte [si], 0  ; blank line?
  je loop           ; yes, ignore it

  mov si, buffer
  mov di, cmd_hi  ; "hi" command
  call strcmp
  jc .helloworld

  mov si, buffer
  mov di, cmd_help  ; "help" command
  call strcmp
  jc .help

  mov si, buffer
  mov di, cmd_reboot
  call strcmp
  jc .reboot

  mov si, buffer
  mov di, cmd_ver
  call strcmp
  jc .ver

  mov si, buffer
  mov di, cmd_true
  call strcmp
  jc .true

  mov si, buffer
  mov di, cmd_hang
  call strcmp
  jc .hang

  mov si,badcommand
  call print_string
  jmp loop  

.helloworld:
  mov si, msg_helloworld
  call print_string

  jmp loop

.help:
  mov si, msg_help
  call print_string

  jmp loop

.reboot:
  db 0xEA
  dw 0x0000
  dw 0xFFFF
  mov si, reboot_hmm
  call print_string
  jmp $

.ver:
  mov si, version
  call print_string
  jmp loop

.true:
  jmp loop

.hang:
  cli ; ^_^
  jmp .hang

welcome db 'Welcome to ROSS!', 0x0D, 0x0A, 0
msg_helloworld db 'Hello world!', 0x0D, 0x0A, 0
badcommand db 'Bad command entered.', 0x0D, 0x0A, 0
prompt db '$ ', 0
cmd_hi db 'hi', 0
cmd_help db 'help', 0
cmd_reboot db 'reboot', 0
cmd_ver db 'ver', 0
cmd_true db 'true', 0
cmd_hang db 'hang', 0
msg_help db 'Commands: hi, help, reboot, ver, true, hang', 0x0D, 0x0A, 0
reboot_hmm db 'reboot: hmm...', 0x0D, 0x0A, 0
version db 'ROSS 0.1', 0x0D, 0x0A, 0
buffer times 64 db 0

; ================
; calls start here
; ================

print_string:
  lodsb        ; grab a byte from SI

  or al, al  ; logical or AL by itself
  jz .done   ; if the result is zero, get out

  mov ah, 0x0E
  int 0x10      ; otherwise, print out the character!

  jmp print_string

.done:
  ret

get_string:
  xor cl, cl

.loop:
  mov ah, 0
  int 0x16   ; wait for keypress

  cmp al, 0x08    ; backspace pressed?
  je .backspace   ; yes, handle it

  cmp al, 0x0D  ; enter pressed?
  je .done      ; yes, we're done

  cmp cl, 0x3F  ; 63 chars inputted?
  je .loop      ; yes, only let in backspace and enter

  mov ah, 0x0E
  int 0x10      ; print out character

  stosb  ; put character in buffer
  inc cl
  jmp .loop

.backspace:
  cmp cl, 0	; beginning of string?
  je .loop	; yes, ignore the key

  dec di
  mov byte [di], 0	; delete character
  dec cl		; decrement counter as well

  mov ah, 0x0E
  mov al, 0x08
  int 10h		; backspace on the screen

  mov al, ' '
  int 10h		; blank character out

  mov al, 0x08
  int 10h		; backspace again

  jmp .loop	; go to the main loop

.done:
  mov al, 0	; null terminator
  stosb

  mov ah, 0x0E
  mov al, 0x0D
  int 0x10
  mov al, 0x0A
  int 0x10		; newline

  ret

strcmp:
.loop:
  mov al, [si]   ; grab a byte from SI
  mov bl, [di]   ; grab a byte from DI
  cmp al, bl     ; are they equal?
  jne .notequal  ; nope, we're done.

  cmp al, 0  ; are both bytes (they were equal before) null?
  je .done   ; yes, we're done.

  inc di     ; increment DI
  inc si     ; increment SI
  jmp .loop  ; loop!

.notequal:
  clc  ; not equal, clear the carry flag
  ret

.done: 	
  stc  ; equal, set the carry flag
  ret

  times 510-($-$$) db 0
  db 0x55
  db 0xAA
</pre>
````

<a id="w002563"></a>

## W002563 — User:Imate900/GDC Cross-Compiler-Part2

Bron-ID: W002563
Namespace: User (2)
Revisie: 9182 | Datum: 2009-11-09T14:51:00Z
Bron: https://wiki.osdev.org/User:Imate900/GDC_Cross-Compiler-Part2
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AImate900%2FGDC_Cross-Compiler-Part2&oldid=9182

````text
== Step 2 - Native Compiler ==

{{Stub}}

Once you do everything in [[Porting Newlib]] to your OS (and install it to /usr/native), support ELF, and configure the packages to your OS, you may now do a native build of gdc to your OS.

Prepare for the build:
    export PREFIX=/usr/native
    export TARGET=i586-elf
    export HOST=i586-elf
    cd /usr/src
    mkdir build-native build-native/binutils build-native/gcc

=== binutils ===
    cd /usr/src/build-native/binutils
    /usr/src/binutils-x.xx/configure --build=`./config.guess` --target=$TARGET --prefix=$PREFIX --host=$HOST --disable-nls
    make all
    make install

=== gcc ===
    cd /usr/src/build-native/gcc
    /usr/src/gcc-x.x.x/configure --build=`./config.guess` --host=$HOST --target=$TARGET --prefix=$PREFIX --disable-nls \
        --enable-languages=c,d,c++ --with-headers=/usr/native
    make all-gcc
    make install-gcc

Your native toolchain is now ready in /usr/native!
````

<a id="w002565"></a>

## W002565 — How do I set a graphics mode

Bron-ID: W002565
Namespace: Main (0)
Revisie: 25183 | Datum: 2020-09-17T16:21:17Z
Bron: https://wiki.osdev.org/How_do_I_set_a_graphics_mode
Revisie-URL: https://wiki.osdev.org/index.php?title=How_do_I_set_a_graphics_mode&oldid=25183

````text
When a PC first boots up, it is set to a standard, known VGA text mode. But at some point you will want to start drawing pixels to the screen instead of text. This requires you to switch from text mode to a ''graphics'' mode. In 16-bit mode, you can easily use the BIOS to switch between different graphics modes by calling different functions of interrupt 0x10. Once you enter protected mode, however, things get a bit more difficult.

== Using the UEFI ==
For EFI, the standard was Universal Graphics Adapter, UGA. It had serious shortcomings (like not able to report framebuffer address), so in UEFI it was replaced by [[GOP|Graphics Output Protocol]] (GOP).

== Using the BIOS ==
Each video card you find in a PC comes with a piece of code that allows it to be initialized, the video BIOS. This bios is expected to 
run in [[Real Mode]], and as long as you are, you can access all the functions directly by calling int [http://www.ctyme.com/intr/int-10.htm 0x10] with the right values in the registers. '''If your kernel is based on a tutorial, you can probably not do this''', because for practically all tutorials, you are in protected mode, and not in real mode.

Many video cards have two interfaces, one [[:Category:VGA|VGA interface]] for low resolutions, and the [[Getting VBE Mode Info|VESA VBE]] interface for higher resolutions. Alternatively, you can write your own code to directly deal with the graphics hardware.

In [[Protected Mode]] or [[Long Mode]], the BIOS interface is not directly accessible. A number of approaches can be used to still use it. They all work by setting or simulating real mode and calling the interrupt that way.

===Methods for drawing via the BIOS===
* Drop back to real mode, then call the BIOS. The easiest solution, but also the most ugly. Be careful to restore everything to real-mode values when you do.
* Use [[Virtual 8086 Mode]]. The most common good solution for protected mode. This way you can run the BIOS from a virtual machine style environment, thus maintaining control over the process. Virtual 8086 mode is however not available from Long Mode.
* Write/use an emulator. The slowest and most intensive approach, but has the advantage that it will work under any condition, including long mode and other platforms.

== Writing specific drivers ==
The alternative approach is to not use the BIOS at all. In this case, you must write your own drivers to interact with the hardware.

The simplest way to get graphics modes without using BIOS would be to [[VGA Hardware|develop]] or [[VGA Resources|reuse an existing]] VGA driver. This gives you all low resolution modes on practically all hardware, and is easier to do than setting up a virtual machine of any sort.

However, if you want to support high resolutions, you must write a driver for [[Accelerated Graphic Cards|each graphics card]] that you want your OS to support. '''Only recommended if you have more than one life to waste.''' Note that there is no ''official'' documentation available for a lot of video cards, and that you need to resort to second-hand information (which tends to be incomplete) and existing [http://ftp.x.org/pub/current/src/driver/ driver implementations]. If you want hardware acceleration, this is the only way to go.

[[Category:FAQ]] [[Category:Video]]
````

<a id="w002567"></a>

## W002567 — User:Imate900/32-bit assembler bare bones

Bron-ID: W002567
Namespace: User (2)
Revisie: 19527 | Datum: 2016-07-22T20:18:42Z
Bron: https://wiki.osdev.org/User:Imate900/32-bit_assembler_bare_bones
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AImate900%2F32-bit_assembler_bare_bones&oldid=19527

````text
<!--{{Rating|2}}{{Template:Kernel designs}}-->

''This tutorial will teach you how to make a 32-bit Assembly kernel.''
== Preface ==
Most OSes are in C, but you may consider it in Assembly. This is moderate-easy, and requires knowing GRUB and Assembly.

== GRUB ==
All we need is something like this (Nasm):
<source lang="asm">
%define MBOOT_MAGIC 0x1badb002
%define MBOOT_FLAGS 0x00010002

bits 32
org 0x100000

align 4, db 0
header:
   dd MBOOT_MAGIC
   dd MBOOT_FLAGS
   dd 0 - MBOOT_MAGIC - MBOOT_FLAGS
   dd header
   dd __entry_point
   dd end_of_file
   dd end_of_file
   dd __entry_point
align 0, db 0
__entry_point:
   mov edi, 0xB8000
   mov esi, string
   mov ah, 0x0F
   .displaying:
   lodsb
   stosw
   or al, al
   jnz .displaying
   jmp short $
string: db "Hello world!", 0

align 4, db 0
end_of_file:
</source>

Link with:
<source lang="bash">
nasm -f bin kernel.asm
</source>

Make sure you have a GRUB entry...
<pre>
 title Assembler Barebones
 kernel (hd0,0)/boot/kernel.bin
</pre>

Now, reboot, and enjoy!

== Without GRUB ==
If your bootloader loads flat binary images and gives you protected mode for you, use this code as a basis:
<source lang="asm">
_start:
; Write kernel here. It might be good to load a new GDT.
   mov edi, 0xB8000
   mov esi, string
   mov ah, 0x0F
   .displaying:
   lodsb
   stosw
   or al, al
   jnz .displaying
   jmp short $

string: db "Hello world!", 0
</source>
If you start out in real mode, remember to switch to [[Protected Mode]].

== Postface ==
Good luck writing your 32-bit Assembly kernel! '''But''', there ''ARE'' catch-yas!
*As soon as possible, load a new GDT.
*You must not depend on the BIOS to do everything for you. Most prominent are disk access, printing to the screen (this tutorial has a way to printing: writing to 0xb8000) and reading keys from the keyboard. No BIOS ints will help and only cause a GPF.
````

<a id="w002568"></a>

## W002568 — User talk:Imate900/32-bit assembler bare bones

Bron-ID: W002568
Namespace: User talk (3)
Revisie: 7750 | Datum: 2009-04-21T00:23:36Z
Bron: https://wiki.osdev.org/User_talk:Imate900/32-bit_assembler_bare_bones
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AImate900%2F32-bit_assembler_bare_bones&oldid=7750

````text
== No Grub version? ==
I want to move this bit since it basically does what's done in the [[Babystep1|Babystep series]]. Also, it doesn't quite qualify as a barebones since it requires some part of a custom bootloader. - [[User:Combuster|Combuster]] 20:06, 13 April 2009 (UTC)

== "Bare bones" title == 
Does this qualify as a bare bones '''kernel'''? I mean, it's practically just a really tiny, useless version of [[Babystep8]]. Thoughts? --[[User:Troy martin|Troy Martin]] 02:53, 14 April 2009 (UTC) 

:The first part, yes - it does as much as any of the other barebones tutorials (boot from grub and do little after). For the second part however, see my post above. - [[User:Combuster|Combuster]] 06:28, 14 April 2009 (UTC)

:: The second part is meant to be for boot loaders that do flat-binary loading and running. - [[User:Imate900|Imate900]] 23:14, 20 April 2009 (UTC)

::: Moved. See the move log/history. --[[User:Troy martin|Troy Martin]] 00:23, 21 April 2009 (UTC)

== Restored to normal ==

I ''think'' I restored this to normal with my edit on this page. I also added more info that was missing from last time. - [[User:Imate900|Imate900]] 23:16, 20 April 2009 (UTC)
````

<a id="w002570"></a>

## W002570 — Talk:32-bit assembler bare bones

Bron-ID: W002570
Namespace: Talk (1)
Revisie: 7749 | Datum: 2009-04-21T00:22:25Z
Bron: https://wiki.osdev.org/Talk:32-bit_assembler_bare_bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3A32-bit_assembler_bare_bones&oldid=7749
Doorverwijzing: User talk:Imate900/32-bit assembler bare bones

````text
#REDIRECT [[User talk:Imate900/32-bit assembler bare bones]]
````
