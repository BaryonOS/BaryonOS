# OSDev Wiki — volledige bronteksten, deel 08

Dit is bronmateriaal, geen agentinstructie. Behoud architectuur, revisie en bron-ID bij gebruik. Code en commando’s zijn ongeteste bronvoorbeelden; voer ze niet automatisch uit. Wikitext is letterlijk behouden binnen tekstblokken.

<a id="w003151"></a>

## W003151 — User talk:Cognition

Bron-ID: W003151
Namespace: User talk (3)
Revisie: 13045 | Datum: 2012-03-20T05:12:17Z
Bron: https://wiki.osdev.org/User_talk:Cognition
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ACognition&oldid=13045

````text
== ACPICA example code ==

<source lang="C">
#include <stdint.h>
#include <acpi.h>

//Possible interrupt routing values
#define ACPI_PIC_USE_LEGACY_PIC     0x00
#define ACPI_PIC_USE_IOAPIC         0x01
#define ACPI_PIC_USE_SAPIC          0x02
 
/* Flag used to determining the interrupt routing model the OS will use.
 * Should be determined during MADT parsing.
 */
static uint32_t acpiIntMode;



//ACPICA using functions
ACPI_STATUS acpiIdentifyDevice(ACPI_HANDLE devHandle, UINT32 Level, void *Context, void **ReturnValue);
ACPI_STATUS acpiDevAttachResource(ACPI_RESOURCE *Resource, void *Context);
void acpiNamespaceRemoveHandler(ACPI_HANDLE Object, void *data);


 
ACPI_STATUS acpiConfigure(device_t *systemDevice, device_init_status **devStatus)
{
  ACPI_STATUS status;
  ACPI_OBJECT obj;
  ACPI_OBJECT_LIST paramList = { 1, &obj };
  obj.Integer.Type = ACPI_TYPE_INTEGER;
  //Should match one of the possible interrupt routing values
  obj.Integer.Value = acpiIntMode;
 
  /*  Try to execute the \_PIC method to let bios know the interrupt routing scheme we plan to use
   *  Might also affect the resource descriptors returned by some _CRS methods
   */
  status = AcpiEvaluateObject(NULL, "\\_PIC", &paramList, NULL);
  if(status == AE_NOT_FOUND )
  {
     kprintf("Warning: Unable to execute \\_PIC method.\n");
  }
  else if(status != AE_OK)
  {
     kprintf("Error executing \\_PIC method Code: %u\n", status);
     return status;
  }
 
  /* Calls a 'Device' object walking function, each time a device is found it will make a call to the callback function
   * provided in the second parameter.  The first parameter can be used by ACPICA to limit the walk to search only for devices
   * with a certain HID value (ascii, null terminated string).  The third and fourth values are implementation specific values 
   * passed along by the OS to provide a single argument and a pointer to provide more specific return information
   */
  return AcpiGetDevices(NULL, &acpiIdentifyDevice, systemDevice, (void **)devStatus);
}
 
ACPI_STATUS acpiIdentifyDevice(ACPI_HANDLE devHandle, UINT32 Level, void *Context, void **ReturnValue)
{
  ACPI_STATUS status;
  ACPI_HANDLE parentHandle;
  ACPI_DEVICE_INFO *devInfo;
  device_t *devObj, *parentDevice;
  hardwareIdList idList = { .count = 0, .root = NULL };
 
  device_init_status **devInitStatus = (device_init_status **)ReturnValue;
  //Initialize the parent device to the system/chipset driver
  parentDevice = Context;
 
  //Attempt to retrieve extra information about the device
  status = AcpiGetObjectInfo(devHandle, &devInfo);
  if(status != AE_OK)
  {
    kprintf("Error retrieving device information - code: %u\n", status);
    return status;
  }
 
  //Make sure the HardwareId field is valid, and add it to the list of strings
  if(devInfo->Valid & ACPI_VALID_HID)
    addHardwareId(&idList, devInfo->HardwareId.String);
  //Do the same thing with any CIDs
  if(devInfo->Valid & ACPI_VALID_CID)
  {
    ACPI_DEVICE_ID *devId = devInfo->CompatibleIdList.Ids;
    for(uint32_t idCount = 0; idCount < devInfo->CompatibleIdList.Count; idCount++)
    {
      addHardwareId(&idList, devId->String);
      devId++;
    }
  }
 
  //Attempt to retrieve a specific parent device pointer, if one exists
  status = AcpiGetParent(devHandle, &parentHandle);
  if(status != AE_NULL_ENTRY)
  {
     ACPI_OBJECT_TYPE objType;
     AcpiGetType(parentHandle, &objType);
     if(objType == ACPI_TYPE_DEVICE)
     {
       device_t *pDev;
       status = AcpiGetData(parentHandle, &acpiNamespaceRemoveHandler, (void **)&pDev);
       /* Use ACPICA's data functions to retrieve an OS specific device pointer for the parent, if one exists.
        * Otherwise assume it's the generic 'System/Chipset' device
        */
       if( status == AE_OK )
         deviceRegisterParent(devObj, pDev);
     }
  }
 
  //OS call to determine the proper driver to load for a given device
  devObj = createDeviceByHardwareId(&idList, devHandle, parentDevice);
 
  //Attempt to attach the current resources the device has to it
  status = AcpiWalkResources(devHandle, "_CRS", &acpiDevAttachResource, devObj);
  if(status != AE_OK)
  {
    ACPI_STATUS nameStatus;
    const char *msgString = "Error retrieving resources for device: ";
    char devPath[80];
    ACPI_BUFFER nameBuffer = { .Length = 80, devPath };
 
    //This basically allows you to get the full ACPI pathname, assuming it can fit into 80 bytes with the trailing null
    nameStatus = AcpiGetName(devHandle, ACPI_FULL_PATHNAME, &nameBuffer);
    if(nameStatus == AE_BUFFER_OVERFLOW)
      kprintf("%s%s\n", msgString, "OVERFLOW");
    else
      kprintf("%s%s\n", msgString, devPath);
 
    return status;
  }
 
  //The address field is bus specific, the device driver should be able to interpret it
  if(devInfo->Valid & ACPI_VALID_ADR)
   devObj->interpretAddressKey(devInfo->Address);
 
  // Ideally you'd query and attach device power state information if exists at this point too, but this is meant to be simple for now
 
  /* Attach the device pointer used by the OS to the namespace object, so we can determine parent devices during enumeration.
   * Also attaches a handler that is called when an object is removed from the namespace (perhaps do to an Unload call), so that
   * properly cleanup and device shutdown can be dealt with
   */
   
  devObj->init(devInitStatus);
  return AcpiAttachData(devHandle, &acpiNamespaceRemoveHandler, devObj);
}
 
//Resource walk callback, called for each resource returned by a resource method (_CRS or _PRS)
ACPI_STATUS acpiDevAttachResource(ACPI_RESOURCE *Resource, void *Context)
{
  device_t *dev = (device_t *)Context;
  //Convert ACPI_RESOURCE typed resources into some OS native resource structure, check the ACPICA Reference manual for more info
  return AE_OK;
}
 
//Deals with a device being removed from the ACPI namespace
void acpiNamespaceRemoveHandler(ACPI_HANDLE Object, void *Data)
{
  device_t *dev = (device_t *)Data;
  registerDeviceRemoval(dev);
}
</source>
````

<a id="w003152"></a>

## W003152 — Talk:ARM Overview

Bron-ID: W003152
Namespace: Talk (1)
Revisie: 13075 | Datum: 2012-03-24T15:26:06Z
Bron: https://wiki.osdev.org/Talk:ARM_Overview
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AARM_Overview&oldid=13075

````text
May have some trouble describing ARM versus the X86/X64 with out throwing a bad light on either one. Even though I feel it is accepted industry wide that the 80386 compatible processors are extremely complex especially the instruction encoding. I still am not sure how much of the industry accepts that RISC is a better overall architecture even though I have read numerous books which do specify RISC is being better and that Intel would be selling a RISC if they could start over, LOL. But... You see what I mean. 

So if you can help then by all means edit it.
: -- [[User:Pancakes|Pancakes]]

:RISC is the better ''theory''. But Intel has burned unbelievable amounts of money in R&D and fab technology, so they still have the edge performance-wise. Many of the things originally introduced for and by RISC have been hammered into submission and added to the x86/x64 chips. As such, I'd just drop the subject. You don't have to make a case, or even a detailed comparison of technologies. Just point out how ARM is different, and keep it at that. IMHO. -- [[User:Solar|Solar]] 05:09, 20 March 2012 (CDT)

:: Thanks, I think it looks much better now. -- [[User:Pancakes|Pancakes]] 20:17GMT, 22 March 2012
````

<a id="w003154"></a>

## W003154 — Talk:PE

Bron-ID: W003154
Namespace: Talk (1)
Revisie: 13058 | Datum: 2012-03-22T09:40:22Z
Bron: https://wiki.osdev.org/Talk:PE
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APE&oldid=13058

````text
Why should anyone want to "keep ASM an C separate"? And aren't PE "sections" just a different name for "segments"? -- [[User:Solar|Solar]] 04:40, 22 March 2012 (CDT)
````

<a id="w003155"></a>

## W003155 — User talk:Pancakes

Bron-ID: W003155
Namespace: User talk (3)
Revisie: 13065 | Datum: 2012-03-22T20:40:52Z
Bron: https://wiki.osdev.org/User_talk:Pancakes
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3APancakes&oldid=13065

````text
Please can you categorise your Integrator CP article. I would suggest Category:ARM. I can't do it because of content filtering.
--[[User:Bellezzasolo|Bellezzasolo]] 08:44, 22 March 2012 (CDT)
: Anytime Bellezzasolo. I got it there now. -- [[User:Pancakes]]
````

<a id="w003156"></a>

## W003156 — User talk:Sparks88

Bron-ID: W003156
Namespace: User talk (3)
Revisie: 13069 | Datum: 2012-03-24T03:20:03Z
Bron: https://wiki.osdev.org/User_talk:Sparks88
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ASparks88&oldid=13069

````text
Oh, man. This was a good link. [http://balau82.wordpress.com/2010/02/28/hello-world-for-bare-metal-arm-using-qemu/|Hello World Bare Metal QEMU ARM]. I am glad you published it I found it interesting. -- [[User:Pancakes|Pancakes]] 22:20, 23 March 2012 (CDT)
````

<a id="w003157"></a>

## W003157 — PS/2 Mouse

Bron-ID: W003157
Namespace: Main (0)
Revisie: 30841 | Datum: 2026-07-02T19:10:32Z
Bron: https://wiki.osdev.org/PS/2_Mouse
Revisie-URL: https://wiki.osdev.org/index.php?title=PS%2F2_Mouse&oldid=30841

````text
Also be sure to read [[Mouse_Input|Mouse Input]].

== Overview ==

The PS/2 Mouse is a device that talks to a PS/2 controller using [[PS/2|serial communication]]. Ideally, each different type of PS/2 controller driver should provide some sort of standard/simple "send byte/receive byte" interface, and the PS/2 Mouse driver would use this interface without caring about lower level details (like what type of PS/2 controller the device is plugged into).


== Mouse Device Over PS/2 ==

Here is the table of command a generic PS/2 compatible mouse understands:

{| class="wikitable"
! colspan="3" | Standard PS/2 Mouse Commands
|-
! Byte !! Data byte !! Description
|-
| 0xE6 || None || Set Scaling 1:1
|-
| 0xE7 || None || Set Scaling 2:1
|-
| 0xE8
|
   {| class="wikitable"
    |-
    ! Byte
    ! Resolution
    |-
    | 00
    | 1 count/mm
    |-
    | 01
    | 2 count/mm
    |-
    | 02
    | 4 count/mm
    |-
    | 03
    | 8 count/mm
    |}
| Set Resolution
|-
| 0xE9 || None || Status Request
|-
| 0xEA || None || Set Stream Mode
|-
| 0xEB || None || Read Data
|-
| 0xEC || None || Reset Wrap Mode
|-
| 0xEE || None || Set Wrap Mode
|-
| 0xF0 || None || Set Remote Mode
|-
| 0xF2
| None
| Get Device ID. See [["8042" PS/2 Controller#Detecting PS/2 Device Types|Detecting PS/2 Device Types]] for the response bytes.
|-
| 0xF3
| Sample rate
| Set Sample Rate, valid values are 10, 20, 40, 60, 80, 100, and 200.
|-
| 0xF4 || None || Enable Data Reporting
|-
| 0xF5 || None || Disable Data Reporting
|-
| 0xF6 || None || Set Defaults
|-
| 0xFE || None || Resend
|-
| 0xFF || None || Reset ('''Note: After the result of the power-on test is sent, the mouse sends its ID (0x00)''')
|-
|}

The most common command reply is ''0xFA'' from the master (mouse), which means acknowledge. You may then get a variable number of bytes afterwards depending on the command. You may also receive other command replies which may state that the master (mouse) has encountered an error decoding your command. For a more detailed list check out some of the links above or look through the Linux source tree. 

First, you have to enable the mouse on the PS/2 bus. This requires sending one byte which is clocked over the PS/2 interface. You will then get a response regarding the result. By sending ''0xF4'' (Enable Data Reporting) the mouse should reply back with a ''0xFA'' which means acknowledgement. Then afterwards as the mouse pointer is moved it will send back the generic packet format like below. Unless you enable an enhanced mode for the mouse (non-standard) this is what you will get when ever the mouse is moved.

{| class="wikitable"
! colspan="9" | Generic PS/2 Mouse Packet Bits
|-
! BYTE!!7!!6!!5!!4!!3!!2!!1!!0 
|- align="center"
| 0 || yo || xo || ys || xs || 1 || bm || br || bl
|- align="center"
| 1 || colspan="8" | X-Axis Movement Value
|- align="center"
| 2 || colspan="8" | Y-Axis Movement Value
|} 
{| class="wikitable"
! Code !! Description 
|-
| yo || Y-Axis Overflow
|-
| xo || X-Axis Overflow
|-
| ys || Y-Axis Sign Bit (9-Bit Y-Axis Relative Offset)
|-
| xs || X-Axis Sign Bit (9-Bit X-Axis Relative Offset)
|-
| 1 || Always One
|-
| bm || Button Middle (Normally Off = 0)
|-
| br || Button Right (Normally Off = 0)
|-
| bl || Button Left (Normally Off = 0)
|}

Each X and Y axis value is relative. The mouse device does not track its location in absolute coordinates. This should also be apparent by the 9-bit values. Instead, it sends back saying I moved this far to the left, to the right, down, or up. To keep track of a mouse position you need to accumulate these relative offsets into a absolute position offset in your code:
<pre>
mouse_x = mouse_x + mouse_packet_rel_x
mouse_y = mouse_y + mouse_packet_rel_y
</pre>
''Being these 9-bit values are signed the above pseudo would work.''

Also, if you simply read the X- or Y-Axis Movement Value fields you will get an 8-bit unsigned value. Which, if used as unsigned will yield incorrect behavior. If you convert it into a signed 8-bit value you will get behavior that is similar to correct, but strange artifacts will appear when the mouse is moved fast. The correct way to produce a 9-bit or greater signed value is as follows:
<syntaxhighlight lang="c">
state = first_byte
d = second_byte
rel_x = d - ((state << 4) & 0x100)
d = third_byte
rel_y = d - ((state << 3) & 0x100)
</syntaxhighlight>
The pseudo code above will cause ''((state << 4) & 0x100)'' to equal ''0x100'' only if the signed bit (9'th bit stored in the first byte) is set. If the 9'th bit is set then the value is deemed negative, but the value in ''second_byte'' is not stored in one or two's complement form. It is instead stored as a positive 8-bit value. So, if ''second_byte'' is say a ''2'' then it will become ''2 minus 0'' since the negative (9'th bit) is off. But, if it is on then it will become ''2 minus 0x100'' which will produce the twos complement, or ''-2''. It will also cause the register to be correctly sign extended no matter its size.

=== Set Sample Rate Example ===

To set the sample rate for example, which is a command with a data byte, one would need to do:
<syntaxhighlight lang="c">
outb(0xD4, 0x64);                    // tell the controller to address the mouse
outb(0xF3, 0x60);                    // write the mouse command code to the controller's data port
while(!(inb(0x64) & 1) asm("pause"); // wait until we can read
ack = inb(0x60);                     // read back acknowledge. This should be 0xFA
outb(0xD4, 0x64);                    // tell the controller to address the mouse
outb(100, 0x60);                     // write the parameter to the controller's data port
while(!(inb(0x64) & 1) asm("pause"); // wait until we can read
ack = inb(0x60);                     // read back acknowledge. This should be 0xFA
</syntaxhighlight>

== Mouse Extensions ==

Here, an example of mouse that supports extensions. To maintain backwards compatibility
you should have to activate these features through the PS/2 bus. Various mouse devices
use different ways. Linux mouse drivers for example sometimes handle multiple different
devices which all share the same standard packet format above, or at least support the
compatibility mode described above.

=== Z-axis ===

To enable the Intellimouse Z-axis extension, you have to set some magic into the sample rate:
<syntaxhighlight lang="c">
set_mouse_rate(200);  // see the example above
set_mouse_rate(100);
set_mouse_rate(80);
mouseid = identify(); // see Get Device ID, 0xF2
</syntaxhighlight>

After that the mouse should not return Mouse ID ''0'', but ''3'', and will send 4 bytes data packages as follows:
{| class="wikitable"
! colspan="9" | Intellimouse #1 PS/2 Mouse Packet Bits
|-
! BYTE!!7!!6!!5!!4!!3!!2!!1!!0 
|- align="center"
| 0 || yo || xo || ys || xs || 1 || bm || br || bl
|- align="center"
| 1 || colspan="8" | X-Axis Movement Value
|- align="center"
| 2 || colspan="8" | Y-Axis Movement Value
|- align="center"
| 3 || colspan="8" | Z-Axis Movement Value
|} 
The Z-axis Movement Value is in "2's complement" format. Valid values are -8 to +7. Other bytes are identical to the PS/2 packet.

=== 5 buttons ===

To enable the 4th and 5th buttons, first you have to try to enable Z-axis, and you can only follow with this if the identification returned ''3''.
<syntaxhighlight lang="c">
if(mouseid == 3) {
  set_mouse_rate(200);
  set_mouse_rate(200);
  set_mouse_rate(80);
  mouseid = identify();
}
</syntaxhighlight>
If this was successful, the identify command should now return Mouse ID ''4'', and the 4 bytes packets will look like this:
{| class="wikitable"
! colspan="9" | Intellimouse #2 PS/2 Mouse Packet Bits
|-
! BYTE!!7!!6!!5!!4!!3!!2!!1!!0 
|- align="center"
| 0 || yo || xo || ys || xs || 1 || bm || br || bl
|- align="center"
| 1 || colspan="8" | X-Axis Movement Value
|- align="center"
| 2 || colspan="8" | Y-Axis Movement Value
|- align="center"
| 3 || 0 || 0 || b5 || b4 || colspan="4" | Z-Axis Movement Value
|} 
Here the Z-Axis Movement Value is stored only in the low 4 bits and the last byte is not sign extended beyond 4 bits. Bits 4 and 5 represents the pressed status of buttons 4 and 5 in that order, same as with bm, br and bl.

=== Emulation ===

Normally bochs does only emulate generic PS/2 mouse. To make bochs to handle Z-axis, you should set the proper mouse type in your bochrc file:
<pre>
mouse: type=imps2, enabled=1
</pre>
If you did the sample rate magic right, then you should see this on the bochs console as soon as you issue the last set sample rate command:
<pre>
0000000000i[KBD   ] wheel mouse mode enabled
</pre>
Qemu on the other hand understands not only the Z-axis mode, but the 5 buttons mode too, and returns Mouse ID ''4'' if you did everything right.

==See Also==
===Articles===
*[[PS/2]]
*[["8042" PS/2 Controller]]
*[[PS/2 Keyboard]]
===External Links===
*[http://www.computer-engineering.org/ps2mouse/ www.Computer-Engineering.org/ps2mouse]
*[http://users.utcluj.ro/~baruch/sie/labor/PS2/PS-2_Mouse_Interface.htm users.utcluj.ro/~baruch/sie/labor/PS2/PS-2_Mouse_Interface.htm]

=== Implementations ===
* [http://lxr.linux.no/#linux+v3.5.4/drivers/input/mouse/psmouse-base.c Linux] (C,GPL)

[[Category:Human Interface Device]]
[[Category:Common Devices]]
[[Category:Hardware Interfaces]]
````

<a id="w003158"></a>

## W003158 — Talk:PS/2 Mouse

Bron-ID: W003158
Namespace: Talk (1)
Revisie: 13121 | Datum: 2012-03-25T05:08:37Z
Bron: https://wiki.osdev.org/Talk:PS/2_Mouse
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APS%2F2_Mouse&oldid=13121

````text
I find another page which essentially goes over what I have here. I did not notice it until after I wrote this one, but once I took a look it seems like the article I am referring to may be biased towards X86/X64 which is GOOD, but I thought I did a nice job for someone to understand no matter their platform of choice. But, this other article is really nice and detailed. Not really sure how to link them together really well?
: The page I am talking about can be found here -> [[Mouse_Input|Mouse Input]].
: --[[User:Pancakes|Pancakes]] 15:05, 24 March 2012 (CDT)

We might need really to enter into talking about the mouse from a platform neutral stance like I kind of did okay on this page. Then branch out into a more detailed explanation of the mouse ([[Mouse_Input]], like here)) and the keyboard ([[PS2_Keyboard]]). Then, also provide more detailed pages for say the I8042 or PL050. But, the current organization of the front page sort of comes from an X86/X64 architecture point of view -- which is OK --  I am neutral in opinion over that.

An alternative idea may be to link this page primarily from the ARM category, which then links to the [[Mouse_Input]] and [[PS2_Keyboard]], and this maintains the wiki holding an X86/X64 face on the front page, which is what most readers are going to be after I am sure?
: --[[User:Pancakes|Pancakes]] 15:22, 24 March 2012 (CDT)


There are several types of "serial communication", including RS-232, FireWire and "PS2". For each type of serial communication we should have:
* A page describing the communication itself (e.g. electrical signals, baud rate, flow control, etc)
* A page for each type of device (e.g. PS/2 mouse, PS/2 keyboard, PS/2 barcode scanner, etc)
* A page for each type of controller (e.g. "8042 or equivelent" on 80x86, PL050 on ARM Integrator/CP board, etc)
The pages for the communication and the pages for each of the devices should always be platform neutral. The pages for each controller probably can't/shouldn't be platform neutral. --[[User:Brendan|Brendan]] 19:35, 24 March 2012 (CDT)

: Yeah, thats what we need.
:: --[[User:Pancakes|Pancakes]] 20:04, 24 March 2012 (CDT)

:::Heh - I broke stuff! This should be a (platform neutral) "PS/2 Mouse" page, and shouldn't have any information about "PS/2 Controller" or other types of keyboard (e.g. we should have a "USB Mouse" or "USB HID" page). We probably also need a generic "User Input" page too. --[[User:Brendan|Brendan]] 23:06, 24 March 2012 (CDT)


:::LOL, I missed it too, but still. It is looking even better. Thats some nice editing there, because I could not have done better myself. -- [[User:Pancakes|Pancakes]] 00:08, 25 March 2012 (CDT)
````

<a id="w003159"></a>

## W003159 — PS/2

Bron-ID: W003159
Namespace: Main (0)
Revisie: 28619 | Datum: 2024-02-10T07:14:23Z
Bron: https://wiki.osdev.org/PS/2
Revisie-URL: https://wiki.osdev.org/index.php?title=PS%2F2&oldid=28619

````text
{{Stub}}

PS/2 is a type of serial communication, typically used for user input devices (keyboard, mouse, bar code scanner, etc). It involves a controller (e.g. "8042 chip" on PC compatible systems), the mechanical and electrical details of the communication itself, and a device.

== History ==

Originally (1981) IBM used a different type of serial communication for XT keyboards. The older interface was not bi-directional - the device could talk to the controller, but the controller couldn't talk to the device.

Later (1984) IBM replaced the older serial interface with a newer bi-directional interface for AT keyboards. This allowed the host to send commands to the device.

Later still (1987) IBM extended the "AT" standard to create the "PS/2" standard. The PS/2 standard is electrically identical to the older AT standard (it only uses a different plug/connector).

Eventually the PS/2 was superseded by USB, however it is still fairly common (and for compatibility, some system have emulation layers that make USB devices appear as PS/2 devices).

== Protocol ==

A diagram of the PS2 connector is below.
{|
|[[Image:PS2_connector.png|frame|right]]
|}

The serial communication typically runs at a baud rate of 10 KHz to 16.7 KHz. Each byte sent by a device to a controller is transmitted as a packet containing 11 bits. The "device to host" packet has the following bits:
* 1 start bit (always 0)
* 8 data bits (least significant bit first)
* 1 parity bit (odd parity)
* 1 stop bit (always 1)

Each byte sent by a controller to a device is transmitted as a packet containing 12 bits. The "host to device" packet has the following bits:
* 1 start bit (always 0)
* 8 data bits (least significant bit first)
* 1 parity bit (odd parity)
* 1 stop bit (always 1)
* 1 acknowledgement bit

== Driver Model ==

There are various types of PS/2 port controllers; including "bit banging" general purpose IO, special support built into the system's chipset and add-on PCI cards. There are also various types of PS/2 devices (keyboard, mouse, touch-pad, bar code scanner, magnetic card/stripe reader, fingerprint reader, etc). 

Obviously you don't want to duplicate the code for every type of PS/2 device within each different PS/2 controller driver. You want to separate them, so that any driver for a PS/2 device can talk to any PS/2 controller driver (and so that the PS/2 device's code can be ported to different architectures without any changes). This is relatively simple to do - PS/2 controller drivers only need to be able to send and receive bytes (there's no baud rate control or anything) and a simple "byte stream" approach suffices.

== See Also ==

=== Articles ===
*[["8042" PS/2 Controller]]
*[[PL050 PS/2 Controller|PL050 PS/2 Controller (ARM)]]
*[[PS/2 Keyboard]]
*[[PS/2 Mouse]]

=== External Links ===
*[http://www.versalent.biz/ps2facts.htm PS2 Facts and Info from Versalent]
*[https://web.archive.org/web/20180302005138/https://computer-engineering.org/ps2protocol/ https://computer-engineering.org/ps2protocol/]

[[Category:Common Devices]]
[[Category:Hardware Interfaces]]
````

<a id="w003160"></a>

## W003160 — I8042 PS/2 Controller

Bron-ID: W003160
Namespace: Main (0)
Revisie: 30586 | Datum: 2026-05-20T07:46:42Z
Bron: https://wiki.osdev.org/I8042_PS/2_Controller
Revisie-URL: https://wiki.osdev.org/index.php?title=I8042_PS%2F2_Controller&oldid=30586

````text
The PS/2 Controller (often called the “Keyboard controller”) is located on the mainboard. In the early days the controller was a single chip ('''Intel 8042'''). As of today it is part of the [[Advanced Integrated Peripheral]].

The name is misleading because the controller does more than controlling communication with PS/2 devices.

{|
|[[Image:ps2-kbc.png|frame|right|Overview of the PS/2-Controller]]
|}


== History ==

A multi-purpose PPI (Intel 8048, Programmable peripheral interface; also used to control other functions, like sound and parity error) controlled the original uni-directional, single channel IBM PC and PC-XT keyboard interface.
The XT controller is 100% obsolete and won't be discussed further in this page.


With the PC-AT, IBM introduced new keyboards (with a new bi-directional protocol) and a new keyboard controller (Intel 8042). The old PPI was not part of the motherboard any more. 

{|
|[[Image:at-kbc.png|frame|right|Overview of the AT-Controller]] 
|}

The 8042 was a powerful micro-controller. To reduce costs, some of the general purpose input/output capabilities of the AT controller was used to control various functions unrelated to the keyboard, including:
* System Reset
* The [[A20 Line|A20-Gate]]


With the introduction of the PS/2 series, the main change to the keyboard controller subsystem was its expansion to control both a keyboard and a mouse. Previously PC and compatible mice were connected to different physical interfaces, including [[Serial Ports]]. The AT keyboard controller and its clones were not capable of interfacing the new PS/2 mouse. Eventually (around the late 80486 and early Pentium time frame) PS/2 style mice became popular, and PC compatible controllers have supported dual channels from then on (nominally one keyboard and one mouse).

For the keyboard functions proper, the PS2 and AT controllers are very similar. The adjunction of a second channel (for the mouse) has forced however to redefine a few status and control bits.

=== Translation ===

The original IBM-PC keyboards (using the old XT interface) used scan code set 1. The new AT keyboards generated different scan codes, or scan code set 2. This change would have created compatibility problems for software that was expecting different scan codes from the keyboard. To avoid the compatibility problem, the keyboard controller supports a translation mode. If translation is enabled the controller will translate scan code set 2 into scan code set 1.

Whenever this translation is enabled (and by default, it is) there is no way to reverse it in software. For example, if you receive the byte 0xB5 from the controller, then you can't know if the original data (sent to the controller by the device) was the byte 0xB5; or if it was the two bytes 0xF0, 0x33; or if it was the two bytes 0xF0, 0xB3.

For software to actually use scan code set 2 (or the even newer, rarely used, scan code set 3), or to allow different devices to be used in the keyboard port, you need to disable this translation to avoid having the data from the device mangled.

== Discovery==

While the presence of I8042 controller can sometimes be assumed (namely, during the BIOS boot), it is not present on all modern PCs (one of the best known examples is the Apple hardware), which might create problems if the IO ports are written to blindly.

The most robust way to discover the I8042 controller on x86 (used by all major operating systems) is with the [[AML]] interpreter. After initializing the ACPI namespace, the one or two (keyboard and mouse/aux device) PnP devices associated to the controller can be found in it (the exact process depends on the operating system design). After discovering the two devices (for example, PNP0300 for keyboard and PNP0F03 for mouse), the "_CRS" method should be used to get the IO ports and the interrupts associated with the controller (each PnP device will have 1 interrupt). 

Since AML is used for discovery of many devices on x86 (and other architectures that use ACPI), the convenient way to initialize the drivers could be letting them register themselves as PnP drivers, and letting the ACPI subsystem of the kernel discover the devices and call a callback, as, for example, has been done by Linux [https://github.com/torvalds/linux/blob/028ef9c96e96197026887c0f092424679298aae8/drivers/input/serio/i8042-acpipnpio.h#L1561]. 

==USB Legacy Support==
By modern standards you will find many PCs bundled with [[Universal Serial Bus|USB]] input devices. Some PCs may not even have PS/2 connectors at all. To remain compatible with old software, the mainboard emulates USB Keyboards and Mice as PS/2 devices. This is called USB Legacy Support.

Because the implementation differ by manufacturer and mainboard there are flaws and sometimes even bugs: 
*Some emulation layers also handle the communication with the real PS/2 connectors regardless of any connected USB device. So maybe not all capabilities of the PS/2 connectors and devices can be used. For example extended mouse modes needed to use the scroll wheel won't work or the keyboard only works on the first PS/2 connector and the mouse only on the second connector.
* The [[SMM BIOS]] that's providing the PS/2 USB Legacy Support may not support extended memory techniques or [[Long Mode]] and may cause system crashes.

This USB Legacy Support should be disabled by the OS as soon as the OS initialises the USB Controller, and this should be done before the OS attempts to initialise the real PS/2 controller. Otherwise the OS would only initialise the emulated PS/2 controller and there's a large risk of problems caused by deficiencies in the firmware's emulation.

==Buffer Naming Perspective==

The PS/2 controller has two (one byte) buffers for data — one buffer for data received from devices that is waiting to be read by your OS, and one for data written by your OS that is waiting to be sent to a PS/2 device. Most data sheets for PS/2 controllers are written from the perspective of the PS/2 device and not from the perspective of software running on the host. Because of this, the names given to these buffers are the opposite of what you expect: the output buffer contains a device's output data (data waiting to be read by software), and the input buffer contains a device's input (data that was sent by software).
==PS/2 Controller IO Ports==

The PS/2 Controller itself uses 2 IO ports (usually, IO ports 0x60 and 0x64). Like many IO ports, reads and writes may access different internal registers.

Historical note: The PC-XT PPI had used port 0x61 to reset the keyboard interrupt request signal  (among other unrelated functions). Port 0x61 has no keyboard related functions on AT and PS/2 compatibles.

{| {{wikitable}} cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
|-
!IO Port
!Access Type
!Purpose
|-
|0x60
|Read/Write
|Data Port
|-
|0x64
|Read
|Status Register
|-
|0x64
|Write
|Command Register
|}


===Data Port===

The Data Port (typically IO Port 0x60) is used for reading data that was received from a PS/2 device or from the PS/2 controller itself and writing data to a PS/2 device or to the PS/2 controller itself.


===Status Register===

The Status Register contains various flags that show the state of the PS/2 controller. The meanings for each bit are:

{| {{wikitable}} cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
|-
! Bit
!Meaning
|-
|0
|Output buffer status (0 = empty, 1 = full)
(must be set before attempting to read data from IO port 0x60)
|-
|1
|Input buffer status (0 = empty, 1 = full)
(must be clear before attempting to write data to IO port 0x60 or IO port 0x64)
|-
|2
|System Flag
Meant to be cleared on reset and set by firmware (via. PS/2 Controller Configuration Byte) if the system passes self tests (POST)
|-
|3
|Command/data (0 = data written to input buffer is data for PS/2 device, 1 = data written to input buffer is data for PS/2 controller command)
|-
|4
|Unknown (chipset specific)
May be "keyboard lock" (more likely unused on modern systems)
|-
|5
|Unknown (chipset specific)
May be "receive time-out" or "second PS/2 port output buffer full"
|-
|6
|Time-out error (0 = no error, 1 = time-out error)
|-
|7
|Parity error (0 = no error, 1 = parity error)
|-
|}


===Command Register===

The Command Port (typically IO Port 0x64) is used for sending commands to the PS/2 Controller (not to PS/2 devices).

== PS/2 Controller Commands==

The PS/2 Controller accepts commands and performs them. These commands should not be confused with bytes sent to a PS/2 device (e.g. keyboard, mouse).

To send a command to the controller, write the command byte to IO port 0x64. If the command is 2 bytes long, then the next byte needs to be written to IO Port 0x60 after making sure that the controller is ready for it (by making sure bit 1 of the Status Register is clear). If there is a response byte, then the response byte needs to be read from IO Port 0x60 after making sure it has arrived (by making sure bit 0 of the Status Register is set).


{| {{wikitable}} cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
|-
! Command Byte
!Meaning
!Response Byte
|-
|0x20
|Read "byte 0" from internal RAM
|Controller Configuration Byte (see below) 
|-
|0x21 to 0x3F
|Read "byte N" from internal RAM (where 'N' is the command byte & 0x1F)
|Unknown (only the first byte of internal RAM has a standard purpose)
|-
|0x60 
|Write next byte to "byte 0" of internal RAM (Controller Configuration Byte, see below)
|None
|-
|0x61 to 0x7F
|Write next byte to "byte N" of internal RAM (where 'N' is the command byte & 0x1F)
|None
|-
|0xA7 
|Disable second PS/2 port (only if 2 PS/2 ports supported) 
|None
|-
|0xA8
|Enable second PS/2 port (only if 2 PS/2 ports supported)
|None
|-
|0xA9
|Test second PS/2 port (only if 2 PS/2 ports supported) 
|0x00 test passed
0x01 clock line stuck low
0x02 clock line stuck high
0x03 data line stuck low
0x04 data line stuck high
|-
|0xAA
|Test PS/2 Controller
|0x55 test passed
0xFC test failed
|-
| 0xAB
| Test first PS/2 port
|0x00 test passed
0x01 clock line stuck low
0x02 clock line stuck high
0x03 data line stuck low
0x04 data line stuck high
|-
|0xAC
|Diagnostic dump (read all bytes of internal RAM)
|Unknown
|-
|0xAD
|Disable first PS/2 port
|None
|-
| 0xAE
|Enable first PS/2 port
|None
|-
|0xC0
| Read controller input port
|Unknown (none of these bits have a standard/defined purpose)
|-
|0xC1
|Copy bits 0 to 3 of input port to status bits 4 to 7
|None
|-
|0xC2
| Copy bits 4 to 7 of input port to status bits 4 to 7
| None
|-
|0xD0
|Read Controller Output Port
|Controller Output Port (see below)
|-
|0xD1 
|Write next byte to Controller Output Port (see below)
Note: Check if output buffer is empty first
| None
|-
| 0xD2
|Write next byte to first PS/2 port output buffer (only if 2 PS/2 ports supported)
(makes it look like the byte written was received from the first PS/2 port)
|None
|-
|0xD3
|Write next byte to second PS/2 port output buffer (only if 2 PS/2 ports supported)
(makes it look like the byte written was received from the second PS/2 port) 
|None
|-
|0xD4
|Write next byte to second PS/2 port input buffer (only if 2 PS/2 ports supported)
(sends next byte to the second PS/2 port)
|None
|-
|0xF0 to 0xFF
|Pulse output line low for 6 ms. Bits 0 to 3 are used as a mask (0 = pulse line, 1 = don't pulse line) and correspond to 4 different output lines.
Note: Bit 0 corresponds to the "reset" line. The other output lines don't have a standard/defined purpose.
|None
|}

Note: Command bytes not listed in the table above should be treated as either chipset specific or unknown and shouldn't be issued. Commands bytes that are marked as "only if 2 PS/2 ports supported" should also be treated as either chipset specific or unknown if the controller only supports one PS/2 port.


===PS/2 Controller Configuration Byte===

Commands 0x20 and 0x60 let you read and write the PS/2 Controller Configuration Byte. This configuration byte has the following format:

{| {{wikitable}} cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
|-
!Bit
! Meaning
|-
|0
|First PS/2 port interrupt (1 = enabled, 0 = disabled)
|-
|1
|Second PS/2 port interrupt (1 = enabled, 0 = disabled, only if 2 PS/2 ports supported)
|-
|2
|System Flag (1 = system passed POST, 0 = your OS shouldn't be running)
|-
|3
|Should be zero
|-
|4
|First PS/2 port clock (1 = disabled, 0 = enabled)
|-
|5
|Second PS/2 port clock (1 = disabled, 0 = enabled, only if 2 PS/2 ports supported)
|-
|6
|First PS/2 port translation (1 = enabled, 0 = disabled)
|-
|7
|Must be zero
|-
|}

Note: Bits that are marked in the table above as "only if 2 PS/2 ports supported" should be treated as either chipset specific or unknown if the controller only supports one PS/2 port.


===PS/2 Controller Output Port===

Commands 0xD0 and 0xD1 let you read and write the PS/2 Controller Output Port. This output port has the following format:

{| {{wikitable}} cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
|-
!Bit
!Meaning
|-
|0
|System reset (output)
'''WARNING''' always set to '1'. You need to pulse the reset line (e.g. using command 0xFE), and setting this bit to '0' can lock the computer up ("reset forever").
|-
|1
|A20 gate (output)
|-
| 2
|Second PS/2 port clock (output, only if 2 PS/2 ports supported)
|-
|3
|Second PS/2 port data (output, only if 2 PS/2 ports supported)
|-
|4
| Output buffer full with byte from first PS/2 port (connected to IRQ1)
|-
| 5
|Output buffer full with byte from second PS/2 port (connected to IRQ12, only if 2 PS/2 ports supported)
|-
|6
|First PS/2 port clock (output)
|-
|7
|First PS/2 port data (output)
|}

Note: Bits that are marked in the table above as "only if 2 PS/2 ports supported" should be treated as either chipset specific or unknown if the controller only supports one PS/2 port.

==Initialising the PS/2 Controller==

Some people assume the PS/2 controller exists and was configured correctly by firmware. This approach can work, but isn't very robust and doesn't correctly support "less simple" scenarios. Examples of why this approach may not work well include:
* Something (e.g. a Boot Manager) left the PS/2 Controller in a dodgy state
*The PS/2 Controller has hardware faults and your OS did no testing
*There's a USB keyboard and a PS/2 mouse, and the BIOS didn't bother initialising the PS/2 controller because it was using USB Legacy Support and not using the mouse
*You want to reliably send data to the second PS/2 device on older hardware and have to know the second PS/2 port exists (see the warning for "Sending Bytes To The Second PS/2 Port" below).

The following steps are for comprehensive PS/2 Controller initialisation. It may be excessive for your purposes, and a more limited version of it may be more suitable. However, it's easy enough to (selectively) remove steps from the following description.

====Step 1: Initialise USB Controllers ====

This has nothing to do with the PS/2 Controller or PS/2 Devices, however if the system is using (typically limited/dodgy) USB Legacy Support it will interfere with PS/2 Controller initialisation. Therefore you need to initialise USB controllers and disable USB Legacy Support beforehand.

====Step 2: Determine if the PS/2 Controller Exists ====
Before you touch the PS/2 controller at all, you should determine if it exists. On some systems (e.g. 80x86 Apple machines) it doesn't exist and any attempt to touch it can cause a system crash. The correct way to do this is is with [[ACPI]] and [[AML]]. While there is a bit 1 (value = 2, the "8042" flag) in the "IA PC Boot Architecture Flags" field at offset 109 in the Fixed ACPI Description Table (FADT), it is unfortunately not reliable. If you find the ACPI PnP devices associated with the controller, or if the system is old enough that it does not support ACPI, you can be certain that the system has it.

====Step 3: Disable Devices====
Devices connected to the PS/2 port(s) must be disabled so that they won't send data at the wrong time and mess up your initialisation; start by sending a command 0xAD and command 0xA7 to the PS/2 controller. If the controller is a single channel device, it will ignore the command 0xA7.

====Step 4: Flush The Output Buffer====
Sometimes (e.g. due to interrupt controller initialisation causing a lost IRQ), data can get stuck in the PS/2 controller's output buffer. To guard against this, now that the devices are disabled and can't send more data to the output buffer, it is a good idea to flush the controller's output buffer. You can do this by reading from port 0x60 and discarding the data.

====Step 5: Set the Controller Configuration Byte====

This is as simple as reading in the Controller Configuration Byte (command 0x20), changing some bits and then writing the new value back (command 0x60). You want to disable IRQs and translation for port 1 by clearing bits 0 and 6. You should also make sure the clock signal is enabled by clearing bit 4.

====Step 6: Perform Controller Self Test====

To test the PS/2 controller, send command 0xAA to it, then wait for it to respond and check if it replied with 0x55 (any value other than 0x55 indicates a self-test fail). This can reset the PS/2 controller on some hardware (tested on a 2016 laptop). At the very least, the Controller Configuration Byte should be restored for compatibility with such hardware. You can either determine the correct value yourself based on the above table or restore the value read before issuing 0xAA.

====Step 7: Determine If There Are 2 Channels====

To determine if the controller is a dual channel one, send a command 0xA8 to enable the second PS/2 port and read the Controller Configuration Byte (command 0x20). Bit 5 of the Controller Configuration Byte should be clear - if it's set then it can't be a dual channel PS/2 controller, because the second PS/2 port should be enabled. If it is a dual channel device, send a command 0xA7 to disable the second PS/2 port again and clear bits 1 and 5 of the Controller Configuration Byte to disable IRQs and enable the clock for port 2 (You need not worry about disabling translation, because it is never supported by the second port).

====Step 8: Perform Interface Tests====

This step tests the PS/2 ports. Use command 0xAB to test the first PS/2 port, then check the result. Then (if it's a dual channel controller) use command 0xA9 to test the second PS/2 port, then check the result.

At this stage, check to see how many PS/2 ports are left. If there aren't any that work you can just give up (display errors and terminate the PS/2 Controller driver). ''Note: If one of the PS/2 ports on a dual PS/2 controller fails, then you can still keep using/supporting the other PS/2 port.''

====Step 9: Enable Devices====

Enable any PS/2 port that exists and works using command 0xAE (for the first port) and command 0xA8 (for the second port). If you're using IRQs (recommended), also enable interrupts for any (usable) PS/2 ports in the Controller Configuration Byte (set bit 0 for the first PS/2 port, and/or bit 1 for the second PS/2 port, then set it with command 0x60).

====Step 10: Reset Devices====

Once controller initialisation is done, all PS/2 devices (if any) should be reset by the driver. This can be done by sending byte 0xFF (reset command, supported by all PS/2 devices) to port 1 (and port 2 for dual channel controllers, see "Sending Bytes to Device/s") and waiting for a response. If the response is 0xFA, 0xAA (Note: the order in which devices send these two seems to be ambiguous) followed by the device PS/2 ID (see "Detecting PS/2 Device Types" for a list of IDs), then the device reset was successful. If the response is 0xFC then the self test failed and the device should be ignored. If theres no response at all then the port is not populated.

==Detecting PS/2 Device Types==

All PS/2 devices should support the Identify command and the Disable Scanning command (which are commands for the device, and not commands for the PS/2 Controller). The device should respond to the Identify command by sending a sequence of none, one or two identification bytes. However, if you just send the Identify command you can't prevent the response from the Identify command from being mixed up with keyboard/mouse data. To fix this problem, you need to send the Disable Scanning command first. Disabling scanning means that the device ignores the user (e.g. keyboards ignore key presses, mice ignore mouse movement and button presses, etc) and won't send data to mess your device identification code up.

The full sequence is:
*Send the Disable Scanning command 0xF5 to the device
*Wait for the device to send ACK back (0xFA)
*Send the Identify command 0xF2 to the device
* Wait for the device to send ACK back (0xFA)
*Wait for the device to send up to 2 bytes of reply, with a time-out to determine when it's finished (e.g. in case it only sends 1 byte)
*Send the Enable Scanning command 0xF4 to the device

A partial list of responses includes:

{| {{wikitable}} cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
|-
! Bytes (Normal)
!Bytes (Translated*)
!Device Type
|-
| None
|None
|Ancient AT keyboard
|-
|0x00
|n/a
|Standard PS/2 mouse
|-
|0x03
|n/a
|Mouse with scroll wheel
|-
|0x04
|n/a
|5-button mouse
|-
|0xAB, 0x83
| 0xAB, 0x41
| rowspan="2" |MF2 keybaord
|-
|0xAB, 0xC1
|0xAB, 0xC1
|-
|0xAB, 0x84
|0xAB, 0x54
|IBM ThinkPads, Spacesaver keyboards, many other "short" keyboards
|-
| rowspan="2" |0xAB, 0x85
| rowspan="2" |0xAB, 0x85
|NCD N-97 keyboard
|-
|122-Key Host Connect(ed) Keyboard
|-
|0xAB, 0x86
|0xAB, 0x86
| 122-key keyboards
|-
|0xAB, 0x90
|0xAB, 0x90
| Japanese "G" keyboards
|-
|0xAB, 0x91
|0xAB, 0x91
|Japanese "P" keyboards
|-
|0xAB, 0x92
|0xAB, 0x92
| Japanese "A" keyboards
|-
|0xAC, 0xA1
| 0xAC, 0xA1
|NCD Sun layout keyboard
|}

<nowiki>*</nowiki> Translation is not possible for the second PS/2 port

Notes:
*If anyone sees any other responses please add to the list above!
*Translated IDs are provided in the table above for completeness, but it's arguably better to disable translation in the controller prior to pulling a device ID to avoid potential confusion.

Once your PS/2 Controller driver knows what types of PS/2 devices are present, it can start suitable device drivers for those devices.

== Hot Plug PS/2 Devices==

'''WARNING:''' PS/2 was never intentionally designed to support hot-plug. Usually it is fine as most PS/2 controllers have reasonably robust IO lines, however some PS/2 controllers (mostly those in old chipsets) might get damaged.

Despite the warning, most OSs (Windows, Linux, etc) do support hot-plug PS/2. It is also relied on by old mechanical switch KVMs (which allow multiple computers to share the same PS/2 devices by effectively disconnecting the device from one computer and connecting it to the next).

When a PS/2 device is removed the PS/2 controller won't know. To work around this, when no data has been received from the device for some length of time (e.g. 2 seconds), an OS can periodically test for the presence of the device by sending an "echo" command to the device and checking for a reply. If the device doesn't respond, then assume the device has been unplugged.

When a PS/2 device is first powered up (e.g. when it is plugged in to a PS/2 port), the device should perform its Basic Assurance Test and then attempt to send a "BAT completion code". This means that software (e.g. an OS) can automatically detect when a PS/2 device has been inserted. ''Note: If a device is removed and then another device (or the same device) is plugged in quickly enough, the software may not have had time to detect the removal.''

When software detects that a device was plugged in it can determine the type of device (see above). If the device was the same type as before software can re-configure it so that the device is in the same state as it was before removal. This means that (for example) someone using an old mechanical switch KVMs doesn't lose state (things like keyboard LEDs, typematic rate, etc) when switching between computers. If the device is not the same as before or there was no previously connected device, then software may need to start a new device driver (and terminate the old device driver, if any).


==Sending Bytes To Device/s==

Unfortunately, the PS/2 Controller does not support interrupt driven transmission (e.g. you can't have a queue of bytes waiting to be sent and then send each byte from inside a "transmitter empty" IRQ handler). Fortunately, little data needs to be sent to typical PS/2 devices and polling suffices.

=== First PS/2 Port ===

To send data to the first PS/2 Port:
*Set up some timer or counter to use as a time-out
* Poll bit 1 of the Status Register ("Input buffer empty/full") until it becomes clear, or until your time-out expires
*If the time-out expired, return an error
* Otherwise, write the data to the Data Port (typically IO port 0x60)

===Second PS/2 Port===

Sending data to the second PS/2 port is a little more complicated, as you need to send a command to the PS/2 controller to tell it that you want to talk to the second PS/2 port instead of the first PS/2 port. To send data to the second PS/2 Port:
*Write the command 0xD4 to Command register (IO port 0x64)
*Set up some timer or counter to use as a time-out
*Poll bit 1 of the Status Register ("Input buffer empty/full") until it becomes clear, or until your time-out expires
*If the time-out expired, return an error
*Otherwise, write the data to the Data Port (IO port 0x60)

'''WARNING:''' If the PS/2 controller is an (older) single channel controller, the command 0xD4 will be ignored, and therefore the byte will be sent to the first PS/2 device. This means that (if you support older hardware) to reliably send data to the second device you have to know that the PS/2 Controller actually has a second PS/2 port.


==Receiving Bytes From Device/s ==

There are 2 ways to receive bytes from device/s: polling and using IRQ.

=== Polling===

To poll, wait until bit 0 of the Status Register becomes set, then read the received byte of data from Data port (typically IO Port 0x60).

There are 2 major problems with polling. The first problem is that (like all polling) it wastes a lot of CPU time for nothing. The second problem is that if the PS/2 controller supports two PS/2 devices, there's no way to reliably determine which device sent the byte you received, unless one of them is disabled and unable to send data.

Note: if the PS/2 controller uses bit 5 of the Status Register as a "second PS/2 port output buffer full" flag, you'd still have problems trying to determine which device sent a byte of data you've received without race conditions. For example, there may be data from the second PS/2 device waiting for you when you check the flag, but before you read from IO Port 0x60 data from the first PS/2 device might arrive and you might read data from the first PS/2 device when you think you're reading data from the second PS/2 device. There's also no easy way to know if the PS/2 controller uses bit 5 of the Status Register as a "second PS/2 port output buffer full" flag.

===Interrupts ===
The interrupt used by keyboard is typically ISA interrupt 1 and the interrupt used by mouse/aux device is typically ISA interrupt 12, however it is best to discover them from ACPI when possible.

Using interrupts is easy. When keyboard interrupt (typically IRQ1) occurs you just read from the Data port (there is no need to check bit 0 in the Status Register first), send the EOI to the interrupt controller and return from the interrupt handler. You know that the data came from the first PS/2 device because you received an interrupt from the device associated with keyboard.

When aux device interrupt (typically IRQ12) occurs you read from Data port (there is no need to check bit 0 in the Status Register first), send the EOI to the interrupt controller/s and return from the interrupt handler. You know that the data came from the second PS/2 device because you received an interrupt associated with that device.

Unfortunately, there is one problem to worry about. If you send a command to the PS/2 controller that involves a response, the PS/2 controller may generate IRQ1, IRQ12, or no IRQ (depending on the firmware) when it puts the "response byte" into the buffer. In all three cases, you can't tell if the byte came from a PS/2 device or the PS/2 controller. In the no IRQ case, you additionally will need to poll for the byte. Fortunately, you should never need to send a command to the PS/2 controller itself after initialisation (and you can disable IRQs and both PS/2 devices where necessary during initialisation).

==CPU Reset==
To trigger a CPU Reset, regardless of what state the CPU is in, write the value 0xFE to the Output port.
<syntaxhighlight lang="asm">
 ;Wait for a empty Input Buffer
 wait1:
 in   al, 0x64
 test al, 00000010b
 jne  wait1
 
 ;Send 0xFE to the keyboard controller.
 mov  al, 0xFE
 out  0x64, al
</syntaxhighlight>

==See Also==
*[[PS/2]]
*[[PL050 PS/2 Controller |PL050 PS/2 Controller (ARM)]]
*[[PS/2 Keyboard]]
*[[PS/2 Mouse]]
===Threads===
*[[Post:69151|PS/2 controller initialisation]]
===External Links===
*[http://web.archive.org/web/20210417040153/http://www.diakom.ru/el/elfirms/datashts/Smsc/42w11.pdf]

[[Category:X86]]
[[Category:Common Devices]]
[[de:Keyboard_Controller]]
````

<a id="w003161"></a>

## W003161 — PS2 Keyboard

Bron-ID: W003161
Namespace: Main (0)
Revisie: 17654 | Datum: 2015-03-01T15:09:38Z
Bron: https://wiki.osdev.org/PS2_Keyboard
Revisie-URL: https://wiki.osdev.org/index.php?title=PS2_Keyboard&oldid=17654
Doorverwijzing: PS/2 Keyboard

````text
#REDIRECT [[PS/2 Keyboard]]
````

<a id="w003162"></a>

## W003162 — Talk:PS2 Keyboard

Bron-ID: W003162
Namespace: Talk (1)
Revisie: 13096 | Datum: 2012-03-25T03:40:42Z
Bron: https://wiki.osdev.org/Talk:PS2_Keyboard
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APS2_Keyboard&oldid=13096
Doorverwijzing: Talk:PS/2 Keyboard

````text
#REDIRECT [[Talk:PS/2 Keyboard]]
````

<a id="w003163"></a>

## W003163 — PS2 Mouse

Bron-ID: W003163
Namespace: Main (0)
Revisie: 13098 | Datum: 2012-03-25T03:41:35Z
Bron: https://wiki.osdev.org/PS2_Mouse
Revisie-URL: https://wiki.osdev.org/index.php?title=PS2_Mouse&oldid=13098
Doorverwijzing: PS/2 Mouse

````text
#REDIRECT [[PS/2 Mouse]]
````

<a id="w003164"></a>

## W003164 — Talk:PS2 Mouse

Bron-ID: W003164
Namespace: Talk (1)
Revisie: 13100 | Datum: 2012-03-25T03:41:35Z
Bron: https://wiki.osdev.org/Talk:PS2_Mouse
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APS2_Mouse&oldid=13100
Doorverwijzing: Talk:PS/2 Mouse

````text
#REDIRECT [[Talk:PS/2 Mouse]]
````

<a id="w003165"></a>

## W003165 — PL050 PS/2 Controller

Bron-ID: W003165
Namespace: Main (0)
Revisie: 30871 | Datum: 2026-07-07T17:41:21Z
Bron: https://wiki.osdev.org/PL050_PS/2_Controller
Revisie-URL: https://wiki.osdev.org/index.php?title=PL050_PS%2F2_Controller&oldid=30871

````text
{{Stub}}
PL050 provides a keyboard or mouse interface(KMI) that is IBM PS/2 or AT-compatible and is licensed by ARM. 
<br>There are several registers to communicate with the controller, some of them are described in details further.

{| class="wikitable"
! Address
! Type
! Description
|-
| Base + 0x00
| rw
| Control register.
|-
| Base + 0x04
| r
| Status register.
|-
| Base + 0x08
| rw
| Received data (read)/ Data to be transmitted (write).
|-
| Base + 0x0c
| rw
| Clock divisor register.
|-
| Base + 0x10
| r
| Interrupt status register.
|}

The table above could be represented as a C struct:
<syntaxhighlight lang="c">
// PL050(KMI) bases under Integrator/CP compatible board.
#define KMI_KB_BASE	            0x18000000	// keyboard
#define KMI_MS_BASE	            0x19000000	// mouse

typedef struct _KMI_MMIO {
	uint32		cr;		// control register (rw)
	uint32		stat;		// status register (r)
	uint32		data;		// data register (rw)
	uint32		clk;		// clock divisor register (rw)
	uint32		ir;		// interrupt control register (r)
} KMI_MMIO;

// Note: set a valid pointer before initialising it.
KMI_MMIO volatile *mmio;
</syntaxhighlight>

''The above uses memory mapped input/output (MMIO), but other architectures may use I/O ports instead or a combination of I/O ports and MMIO for example the X84/64. So understanding your platform is very important to understand how to proceed in talking to the devices.''

So first, configure the PL050 by enabling it. Let's take a look at control register. It contains five different bit fields:
{| class="wikitable"
! Bits
! Name
! Description
|-
| 0
| FKMIC
| The force KMI clock LOW bit field is used to force the PrimeCell KMI clock pad LOW regardless of the state of the KMI FSM.
|-
| 1
| FKMID
| The force KMI data LOW bit field is used to force the PrimeCell KMI data pad LOW regardless of the state of the KMI finite state machine (FSM).
|-
| 2
| KmiEn
| The enable PrimeCell KMI bit field is used to enable the KMI.
|-
| 3
| KMITXINTREn
| Enable transmitter interrupt. This bit field is used to enable the PrimeCell KMI transmitter interrupt.
|-
| 4
| KMIRXINTREn
| Enable receiver interrupt. This bit field is used to enable the PrimeCell KMI receiver interrupt.
|-
| 5
| KMITYPE
| 0 = PS2/AT mode (default), 1 = No line control bit mode.
|-
| 6-7
| -
| Reserved. Read unpredictable.
|}

Setting the ''KmiEn'' will enable KMI. After that it's needed to setup a receiver interrupt, all it's needed is to set ''KMIRXINTREn'' and configure system's interrupt controller to call the callback which processes the input data.

<syntaxhighlight lang="c">
void kmi_init(device_t* dev)
{
    mmio->cr = (1 << 2) | (1 << 4);
}
</syntaxhighlight>

The core function to implement is a one to send commands to PS/2 devices, like keyboard. For that the ''data register'' is used. To send a command, set a command id to data register and wait for an answer. The informations which helps to understand when is located in a ''status register''. Data register does not contain any fields, just used to forward data, while it's interesting to take a detailed look at the status register:
{| class="wikitable"
! Bits
! Name
! Description
|-
| 0
| KMID
| This bit reflects the status of the KMIDATAIN line after synchronizing.
|-
| 1
| KMIC
| This bit reflects the status of the KMICLKIN line after synchronizing and sampling.
|-
| 2
| RXPARITY
| This bit reflects the parity bit for the last received data byte (odd parity).
|-
| 3
| RXBUSY
| This bit indicates that the PrimeCell KMI is currently receiving data. 0 = idle, 1 = receiving data.
|-
| 4
| RXFULL
| This bit indicates that the receiver register is full and ready to be read. 0 = receive register empty, 1 = receive register full, ready to be read.
|-
| 5
| TXBUSY
| This bit indicates that the PrimeCell KMI is currently sending data. 0 = idle, 1 = currently sending data.
|-
| 6
| TXEMPTY
| This bit indicates that the transmit register is empty and ready to transmit. 0 = transmit register full, 1 = transmit register empty, ready to be written.
|-
| 7
| -
| Reserved. Read unpredictable.
|}
<br>
<syntaxhighlight lang="c">
void kmi_send(uint8 cmd)
{
    kmi->data = cmd;
    while ((registers->stat) & (1 << 5)); // wait while data is sending (TXBUSY)
    ASSERT(registers->data == 0xfa);
}
</syntaxhighlight>

These functions are base ones to communicate with PS/2 devices. Using this functions you can enable mouse by writing ''0xF4'', more information about commands could be found at  [[PS/2 Keyboard]] and [[PS/2 Mouse]]
<syntaxhighlight lang="c">
int kmi_setup_mouse()
{
    kmi_send(0xF4);
}
</syntaxhighlight>

Note that, due to the complexity of the Raspberry Pi's USB interface and QEMU's versatilepb being the go-to Pi emulator before raspi2 was developed, some older Pi programs will use the PL050 to easily enable debugging in QEMU. No Pi device so far has PS/2 compatibility hardware.

==See Also==
=== Articles ===
*[[PS/2]]
*[["8042" PS/2 Controller]]
*[[PS/2 Keyboard]]
*[[PS/2 Mouse]]

===External Links===
*[https://developer.arm.com/documentation/ddi0143/c/preface?lang=en ARM PL050 Technical Reference Manual]

[[Category:ARM]]
````

<a id="w003166"></a>

## W003166 — User:Sortie

Bron-ID: W003166
Namespace: User (2)
Revisie: 22222 | Datum: 2018-03-17T14:21:48Z
Bron: https://wiki.osdev.org/User:Sortie
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASortie&oldid=22222

````text
Hi, I'm [http://maxsi.org sortie].

I'm an admin here on the osdev wiki. I'm a moderator on the [https://forums.osdev.org osdev forums]. I'm an operator on the #osdev IRC chat on freenode.org.

I develop the [https://sortix.org Sortix operating system].
<!--

Hi, I'm Sortie. Guess what, I made an operating system! I'd list all the nice features it supports, but somehow the only interesting part for normal people is that you can play Pong.

Edit: A couple years passed, you can now play Quake.

Edit: Another year. I am now self-hosting. I demand I'm eligible to never hear of Windows again!

Edit: Another two years. My OS is installable and self-hosting and the latest release was built under itself. I may have heard of Windows again.

-->

== Warranty ==

Please realize my recommendations aren't applicable in your case this if:

* You are making a microkernel or any other design where my personal experience isn't relevant.
* Your design goals are incompatible with my recommendations.
* You insist on osdeving in ways I cannot approve of.
* You are not trying to make a good operating system.

Let me know if I am wasting my time trying to give inapplicable advise.

== Discouragements ==

* Don't ask questions on #osdev, wait 30 seconds, and then ragequit. I'm upstairs getting my coffee and I can't prepare it that fast.
* Don't follow tutorials that can't be edited by the community.
* Don't follow tutorials that haven't been peer reviewed. The scientific method is important at the University of Osdev.
* Don't follow tutorials made by the inexperienced.
* Don't follow tutorials without obviously consulting #osdev whether the tutorials are bad and whether there are errata.
* Don't blindly follow tutorials: Some tutorials give very bad advise. Some are buggy. The best ones have bugs only experts notice and you won't until things explode.
* Don't write a bootloader unless you must. Use an existing one.
* Don't write an operating system in assembly.
* Don't do crazy real-mode stuff.
* Don't do FAT lest you give your VFS brain damage.
* Don't make it hard to turn your half-assed work into something proper lest you spend lots of time fixing it.
* Don't assume your small project never goes anywhere.
* Don't use \ as a path separator.
* Don't use flat binaries: They are too simple and contain no information where they are loaded. Use ELF instead.
* Don't listen to your own advise. He is a newbie and might not know it. You'll hate his code deeply a few years down the road.
* Don't try to replace C or Unix without truly knowing what's wrong with it and what's good about it.
* Don't use newlib. Use musl.
* Don't do black magic. You're trying to be cool and failing at it. If this is the proper way to do it, declare this the normal approach and think nothing special of it; otherwise do it the proper way instead.
* Don't use code you developed while sleeping. You were unconscious. Do you code well when you are really tired?
* Don't leave proving the code is correct as an exercise for the reader.
* Don't follow operating systems development tutorials on Youtube: They are not community edited and tends to be made by the inexperienced.
* Don't proclaim me a master of osdev and quote my holy words in #osdev.
* Don't write tutorials if you're inexperienced.

== Recommendations ==

* Understand what your goals truly are and what you want from osdev.
* Take osdev seriously and do things well.
* Don't play the role of an operating systems developer, ''be'' one.
* With every incompatibility come a price that must be paid.
* Corollary: Pick your battles.
* Read instructions carefully.
* Take correctness seriously.
* Correct code is secure.
* Know your programming language well.
* Know your ABI well.
* Know your toolchain well.
* Be truly afraid of undefined behavior.
* Use a modern, good Unix for osdev.
* Use UTF-8 (no BOM, Unix line terminators).
* Your operating system should be itself, not fight what it is.
* What other operating systems do: Embrace it, or fight it. Don't half-adopt it.
* Use a [[GCC_Cross-Compiler|cross-compiler]].
* Follow [[Bare Bones]].
* Follow [[Meaty Skeleton]].
* Initialize at compile, link or load time, not run-time.
* Enable paging as soon as possible.
* Have a pre-emptive kernel early on.
* Implement [[Ext2]] as your first filesystem.
* Make a [[OS Specific Toolchain]] when you add a user-space.
* Quote me in your forum signature.
* Be an expert.

== Naively Closed Source ==
{{Main|User:Sortie/Naively Closed Source}}
Just release your hobbyist OS source code already.

== Wiki Contributions ==
It would seem customary to have such a list on your user page, so here goes:

* [[Bare Bones]] (Rewrite)
* [[Building GCC]]
* [[Calling Global Constructors]]
* [[Creating a C Library]]
* [[GCC Cross-Compiler]] (Rewrite)
* [[Why do I need a Cross Compiler?]] (Rewrite)
* [[Cross-Porting_Software]]
* [[Meaty Skeleton]]
* [[OS Specific Toolchain]] (Rewrite)
* [[Hosted GCC Cross-Compiler]]
* [[Stan Dard]]
* [[Cross-Porting Software]]
* [[System V ABI]]
* [[James Molloy%27s Tutorial Known Bugs]]
* [[Host Triplet]]
* [[libgcc]]
* [[Stack Smashing Protector]] (Rewrite)
* [[C Library]] (Rewrite)
* [[Thread Local Storage]]
* [[Hard Build System]]
* (and various other stuff)
````

<a id="w003167"></a>

## W003167 — User:Pancakes/Sandbox

Bron-ID: W003167
Namespace: User (2)
Revisie: 13195 | Datum: 2012-03-30T05:20:53Z
Bron: https://wiki.osdev.org/User:Pancakes/Sandbox
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APancakes%2FSandbox&oldid=13195

````text
===== Unaligned Memory Access And Byte Order =====
{| border="1px"
| style="background-color: #eeeeee;" | The accuracy of this section has not been generally verified, and could be inaccurate.
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

<pre>
uint32        *a;
uint16        *b;

a = (uint32*)0;
b = (uint16*)0;

a[0] = 0x12345678;
printf("%#04x --> %#04x\n", b[0], b[1]);
</pre>

This gives ''0x5678 --> 0x1234'', while in big endian mode you would get ''0x1234 --> 0x5678''.

{| class="wikitable"
! colspan="8" | Little Endian Word Size (32Bit)
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
! colspan="8" | Little Endian Half-Word Size (16Bit)
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
! colspan="8" | Byte Size (8Bit)
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

''From reading the data sheet it appears that if operating in big endian mode the half-word access would be reversed to be more natural as you would expect on the X86/64 architecture. But, since I can not actually test it at the moment I am hoping I got it right.''

''The word access with offset of ''10b (0x2)'' may be defined, but I am not sure because it does not really state. However, it may employ some of the mechanisms for loading half-words. (Need someone to come through and correct this if it is wrong)''

The reason memory access has to be aligned is because unaligned access requires additional access cycles, due tue the way memory modules work (see DDR datasheet below), and would increase the complexity of load / store operations (and the processor core).

You could simulate an unaligned memory access on the ARM7TDMI-S, but you would have to make separate loads from two memory locations. A compiler could probably emit code to do this automatically, but the checks whether an access is unaligned or not would slow down ''all'' memory accesses. Some code is provided in the ''ARM7TDMI-S Data Sheet'' on page ''4-35'' for such "checked" memory access,  when you do not know if the address will be aligned or non-aligned.

* http://lists.gnu.org/archive/html/qemu-devel/2004-12/msg00206.html - patch for QEMU to support big-endian ARM
* http://download.micron.com/pdf/datasheets/dram/ddr/256MBDDRx4x8x16.pdf Datasheet for DDR memory
````

<a id="w003169"></a>

## W003169 — User talk:Pancakes/Sandbox

Bron-ID: W003169
Namespace: User talk (3)
Revisie: 13169 | Datum: 2012-03-27T16:56:11Z
Bron: https://wiki.osdev.org/User_talk:Pancakes/Sandbox
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3APancakes%2FSandbox&oldid=13169

````text
I went over the article language-wise. (Mostly shortening of sentences. Removed documentation of undocumented behaviour. Adjusted "unpredictable" to "undefined".) I did '''not''' double-check if your statements on big vs. little endian are true. You're probably better off referring to http://en.wikipedia.org/wiki/Endianess, or simply linking that article. -- [[User:Solar|Solar]] 04:01, 27 March 2012 (CDT)

: Oh man thats a gigantic help. Many thanks indeed. Yes, I am going to let it sit here and keep working just to be sure. I turned around last night and realized I might have the 8-bit table wrong, LOL. Yeah, I will link it to that Endianess article and compare it very well. But, thanks. For helping me with it. Its tricky to think about it with out making a mistake.
: -- [[User:Pancakes|Pancakes]] 11:56, 27 March 2012 (CDT)
````

<a id="w003170"></a>

## W003170 — Talk:Memory management

Bron-ID: W003170
Namespace: Talk (1)
Revisie: 13172 | Datum: 2012-03-28T04:17:26Z
Bron: https://wiki.osdev.org/Talk:Memory_management
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMemory_management&oldid=13172

````text
I think we should have something on [http://nob.cs.ucdavis.edu/classes/ecs150-2008-02/handouts/memory/mm-segpag.html segmented paging and paged segmentation]. Maybe a brief mention under "Memory Translation Systems" and the details in a separate article to avoid clutter and distraction. --[[User:Andreasbwagner|Andreasbwagner]] 23:17, 27 March 2012 (CDT)
````

<a id="w003171"></a>

## W003171 — User:AlfaOmega08/Cygwin toolchain

Bron-ID: W003171
Namespace: User (2)
Revisie: 13194 | Datum: 2012-03-29T19:29:32Z
Bron: https://wiki.osdev.org/User:AlfaOmega08/Cygwin_toolchain
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAlfaOmega08%2FCygwin_toolchain&oldid=13194

````text
Here, I'm going to explain how to create a toolchain on cygwin to be used to create and run x86-64-elf kernels on Windows. I'm mainly using this page to not forget the steps I've done.

== Needed packages ==
Open cygwin's setup.exe and install:
#gcc
#gcc-g++
#flex
#bison
#nasm
#make
#automake
#autoconf
#libiconv
#libmpfr-dev
#libmpc-dev
#libgmp3-dev

Something more might be needed. I may expand the list.

== The cross compiler ==
First of all you need a compiler. See [[GCC Cross Compiler]]. Or download gcc-4.7.0.tar.bz2 and binutils-2.22.tar.bz2 and use the following commands:
 tar xvf gcc*
 tar xvf binutils*
 mkdir build-{binutils,gcc}

First install binutils:
 cd build-binutils
 ../binutils*/configure --target=x86_64-elf --prefix=/usr/cross64 --disable-nls
 make all -j <# of cores on your proc * 2>
 make install
 cd ..

Then gcc:
 export PATH=$PATH:/usr/cross64/bin
 cd build-gcc
 ../gcc*/configure --target=x86_64-elf --prefix=/usr/cross64 --enable-languages=c,c++ --disable-nls
 make all-gcc -j <# of cores * 2>
 make install-gcc
 make all-target-libgcc -j <# * 2>
 make install-target-libgcc

== GRUB2 ==

The "stable" version of GRUB2 is bugged. It can't boot elf64 properly. So download the latest grub release [http://ftp.gnu.org/gnu/grub/grub-1.99.tar.gz here]. Extract the package and edit /grub-core/loader/multiboot-elfxx.c, by adding 
 # define Elf_Shdr	Elf32_Shdr
at line 25 and:
 # define Elf_Shdr	Elf64_Shdr
at line 32

Now open cygwin and run:
 cd grub*
 ./configure

The gcc version that comes with cygwin is very old and misses some flags of GRUB2 setup so open ./Makefile and ./grub-core/Makefile and remove all references to: "-Werror" and "-Wmissing-field-initializers" and don't touch anything else. Now you can:
 make all -j <# * 2>
 make install

== xorriso ==
The tool I use to make iso image with GRUB2 (grub-mkrescue) needs xorriso which isn't shipped with cygwin. Download the latest release [http://scdbackup.sourceforge.net/xorriso-1.2.0.tar.gz here] and then:
 tar xvf xorriso*
 cd xorriso*
 ./configure
 make all -j <# * 2>
 make install

This needs libiconv (or libiconv-dev?)

Now you should be able to compile & link a 64 bit kernel using /usr/cross64/bin/x86_64-elf-{gcc,g++,ld,objdump,whatever} and the version of nasm which comes packed with cygwin, and to create a (working) boot iso using:
 grub-mkrescue --output=image.iso iso

== GDB ==
So you wan't to use GDB to debug your OS? That's a PITA. It took me a lot to find the right combination of commands to create a decent GDB on cygwin. So download the latest GDB snapshot and compile it like this:
 ./configure --host=i686-pc-cygwin --target=x86_64-unknown-linux-gnu
 make all -j <you know what>
 make install

It seems that x86_64-unknown-linux-gnu is the most similar to x86_64-elf that will accept to work with gdb.

Have a good time OSDeving.
````

<a id="w003172"></a>

## W003172 — User:Bellezzasolo/Memory Management

Bron-ID: W003172
Namespace: User (2)
Revisie: 13891 | Datum: 2012-08-04T21:04:26Z
Bron: https://wiki.osdev.org/User:Bellezzasolo/Memory_Management
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ABellezzasolo%2FMemory_Management&oldid=13891

````text
'''This is my take on memory management'''
=Memory Management=
==Introduction==
Memory mangaement is an essential part of any OS. Without it an OS would be non-functional. There are sevaral different problems and solutions, some of which are listed here.
==Problems and Solutions==
===Physical Memory Management===
====Problem====
How do you divide up physical memory? How do you mantain this memory (in memory)?
====My Solution====
For the division, 4K blocks (or whatever page size) seems logical. For the second problem you have the bootloader set up enough page size for your bitmap (or whatever you use to keep track of memory, say a page stack). I will use two levels of bitmaps (or on X64, 3) with the first level managing the actual memory (4096B blocks per bit), second level managing this for access speed, 1 bit = 128B. The third level will use the same quotient, wheras the second level will be 1 bit to 32B on X64 to keep table level 3 to 128B.

===Virtual Memory Management===
====Problem====
How do you deal with pages? swapping?
====My Solution====
There is no special secret. I have separated code for X86 and X86-64 (#ifdef). My memory manager will swap pages to disk once I have an HDD driver.

===malloc() and free()===
====Problem====
How do you keep track of free/used blocks? How do you keep track of the size of each block (size in malloc)? 
====My Solution====
For the division of each individual page, I use a block size along the line of 16 or 32 bytes (#define) as malloc() is generally for large blocks. Each set of pages (so as not to get in the way of applications or data access) has a header that follows this structure:
{| {{wikitable}}
|-
!Offset (B)
!Size (B)
!Name & Description
|-
|0
|4
|Length - Number of pages accounted for
|-
|4
|(N.Pages*Page Size)/4
|Bitmap
|}
Each page or set of pages has a bitmap that follows this structure (one set of bits represents a block's size (16B typical):<br />
...|FT|FR|...<br />
Where:
*FT is follow through (if not set it is end of malloc()'ed block). For free() purposes.
*FR is free (if set it is used). For malloc() to see wether available.
````

<a id="w003173"></a>

## W003173 — User:Combuster/Recovered MBR

Bron-ID: W003173
Namespace: User (2)
Revisie: 13225 | Datum: 2012-04-02T17:07:05Z
Bron: https://wiki.osdev.org/User:Combuster/Recovered_MBR
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACombuster%2FRecovered_MBR&oldid=13225

````text
Copy of the MBR stuff for archival purposes and backup of the original after the author ragequit on it. Once the world has calmed down, I ought to reconstitute the better parts and improve the theoretical wiki parts on bootloaders.

Stuff looks safe for MBR use. All other purposes are a lot of extended truths from the original author. Discussion included for  reference; '''use sourcecode with caution'''.

== Goal? ==

I'm not sure I understand the purpose of this page, as it isn't ''really'' an article---it just seems to be the source code for a simplistic boot loader; while it may be useful to the community, I'm not sure source code listings are the best use of a wiki. Perhaps it would make more sense to provide a link to this boot loader somewhere? --[[User:Love4boobies|Love4boobies]] 04:19, 30 March 2012 (CDT)

:That would probably be for the best, overall. Look at it this way: If you provide source, you should also provide maintenance and support. That's easy to do on a private project page, but somewhat difficult with a Wiki article. Plus, most of the use cases of this bootloader could be just as well covered by writing a GRUB extension, unless I'm seriously mistaken. -- [[User:Solar|Solar]] 04:29, 30 March 2012 (CDT)

== Corrections Explained ==

I made some corrections, Turdus "un-corrected" them (possibly because he couldn't have known why they were made), and then I rolled back his "un-corrections" and wrote this to explain the reasons for the original corrections.

From the old preface: "''If you're happy with fat32, extfs and pe, elf or even aout, they are supported by multi-boot, so leave now and use [[GRUB]].''". This is wrong because the multiboot specification says nothing about any file system or any executable file format. It could say "''If you're happy with fat32, extfs and pe, elf or even aout, they are supported by GRUB''" but then it'd still be a little wrong because GRUB legacy (one implementation of multiboot) has no support for PE or aout (unlike ELF). To avoid confusion it's just simpler to not say so much, especially as it's unnecessary (as this page isn't about multiboot or GRUB anyway).

From the old chainloading section: "''For drive optimization, the first ''partition'' usually started on the ''second'' cylinder, leaving the remainder of the first track (a 63-sector gap) between the MBR and the first VBR.''". For modern/large drives there are 63 sectors per track (due to old BIOS limits). However, for old/small drives there can be less than 63 sectors per track. For example, a 512 MiB USB flash drive could be presented as 32 sectors per track (with 128 heads and 256 cylinders). Assuming there's always 63 sectors per track is a bad idea. ''Note: I didn't realise this before, but if there are 63 sectors per track (the maximum) then the gap would be 62 sectors and not 63 sectors (as one of the 63 sectors is consumed by the MBR and therefore isn't part of the gap).''

From the GPT section: "''Intel engineers and others decided to create a new partitioning scheme in the gap after the MBR, called [[GPT]]. This however made non IBM PC compatible boot managers (occupying the same sectors) incompatible.''". For GPT there's 2 partition tables for redundancy (one near the start of the disk an one near the end), and the entire partitioning scheme (including the second partition table) is not in the gap. I had also removed the "''(occupying the same sectors)''" part. Mostly (at the time) I was tempted to replace the majority of the history section with links to the relevant wiki pages, and compromised (by reducing unnecessary detail) instead.

--[[User:Brendan|Brendan]] 15:37, 20 March 2012 (CDT)


One correction:
 This is wrong because the multiboot specification says nothing about 
 any file system or any executable file format.
Multiboot requires reading ELF headers for load addresses, and provides the not-so-aptly named "a.out" kludge (while nobody uses a.out) for any other format to provide similar information. I plan on doing a thorough read of the article later and fix things where necessary -- [[User:Combuster|Combuster]] 01:05, 21 March 2012 (CDT)


I made some corrections, Turdus "un-corrected" them #2. I therefore added the disputed tag as a warning to all.

Point is, the bootloader is in essence designed with non-portability in mind. The claim is that it can be used as a VBR, but it fails that claim for any filesystem that embeds information on sector 0 - which include FAT*, HPFS, NTFS, SFS and XFS and only excludes Ext2/3 for as far as I can check. It also promotes using hardcoded continuous blocks for booting which fails when the second stage is a file that might be touched by the filesystem as well as being a good way of causing the typical starter bugs by not loading enough sectors.

I also explicitly added the floppy exclusion because it is considered a VBR by the world (apaprently minus Turdus as per the forums), and because many BIOSes don't support LBA for those devices making any attempt to use this code a shortcut for screwing up that way.

And that the device must support LBA should be obvious, especially to the writer. -- [[User:Combuster|Combuster]] 18:14, 28 March 2012 (CDT)

Corrections:

It is not designed for FAT, HPFS, NTFS,... it would be obvious if you kept "If you're happy with fat32, extfs and pe, elf or even aout, they are supported by GRUB". Maybe pe and aout should have been left out, but the rest applies.

:''"Point is, the bootloader is in essence designed with non-portability in mind."''

How do you know that? It was not designed for existsing filesystems, yes, but it's portable. Tested scenarios:

* MBR (UBRL), VBR (LILO) -> Works,
* MBR (UBRL), VBR (MSDOS FAT32) -> Works,
* MBR (LILO), VBR (UBRL) -> Works.

:''"It also promotes using hardcoded continuous blocks for booting which fails when the second stage is a file"''

You are wrong (as usual), and you really should learn to READ. I wrote in section "2nd stage":

:"Even better, store it in a defragmented, continuous file on your filesystem"

Do you know what "defragmented, continous" means, don't you?

:''"I therefore added the disputed tag as a warning to all."''

Show me the details of your testing before you do that.

:''"I had also removed the "''(occupying the same sectors)''" part." and "with links to the relevant wiki pages"''

You should really read those wiki pages and specifications: http://en.wikipedia.org/wiki/GUID_Partition_Table

It's very clear that first GPT starts right after the MBR: "Partition table header (LBA 1)" Don't see any option on LBA 1 here. Also, it reads: "Partition entries starting LBA ('''always 2''' in primary copy)"

Doesn't matter if first partition starts at 63 or 32 or whatever, the point is, you cannot use the gap after the MBR because it's for the GPT now. Rephrase the sentences if you like. And on modern disks it's not 63 (that's the past), in the last 5 years all disks we bought had partitions megabyte aligned.

:''"I made some corrections, Turdus "un-corrected" them (possibly because he couldn't have known why they were made)"''

Because they do not mean the same, or you've removed important information. For example:

:''"If you are happy with multiboot"''

How would a reader know if it's multiboot he/she is looking for? I wrote some examples on purpose.

Now it reads: "If you're happy with ELF, FAT or ext (fileformat and filesystems supported by version of multi-boot)"

Hope it's good for you. -- [[User:Turdus|Turdus]] 12:15, 29 March 2012 (CDT)

----

The Wiki format does not cater well for such point-by-point replies. If you have an argument, take it to the forum. And '''please''' keep it civil. Calling each other names serves no purpose. What might be marginal on the forum is ''definitely'' out of place on a Wiki discussion page.

Advertising own code has ''always'' been met with criticism here (as I can [[Makefile | attest]]); take it with style and address it instead of trying to shoot it down. Perhaps it would be better to keep the code and prose in a separate place (your project website), and merely provide a link to it in the relevant context; it's not really tutorial style, it ''does'' contain statements that are disputed. -- [[User:Solar|Solar]] 09:04, 29 March 2012 (CDT)

----

Wiki format rules understood.

About the rewrite: it's much more readble indeed, and it reflects what I originally meant, so thank you very much. I wrote only a few words explaining why BIOS Boot partition is useful.

I did not want to advertise my code on the wiki. I wanted to show how to write a 2nd stage that capable of booting via different methods, and for that I needed a working 1st stage. Never wanted to say it's the only way to do it, or anything, it's nothing more than a working example. This is now made clear in ''Preface'', thanks.

About dispute: the usability of the code as VBR was questioned by someone who didn't even bothered to test it, or even clearly reasoning why. His argument was invalid (does not support floppies), since floppies does not have VBRs. But I accept your opinion it's disputed, so my question is, how to make this clear? All I can say is test it, and see it working (I spent endless nights to test it in different scenarios, so I know it's okay. I wouldn't have posted it here if I wasn't 100% sure).
--[[User:Turdus|Turdus]] 0926, 30 March 2012 (CST)

:Not really my bowl of fish here (and I don't feel like testing it ATM), but what about your code requiring LBA addressing? Does that work on floppies, too? Combuster says it doesn't. -- [[User:Solar|Solar]] 03:15, 30 March 2012 (CDT)





== History ==

=== Boot Sector ===

When the system powers up, it determines the boot device, loads the first sector from that device (usually the first sector on the device), and executes it. On early systems, like [[Floppy disk | floppy]]-based ones, the boot sector then proceeded to load the OS kernel directly.

=== Chainloading ===

With the advent of hard disks, drive partitioning became the rule. The ''partition table'' also resided in the boot sector, and the available space for the boot code was reduced to 440 bytes.

The code in the boot sector (now called ''Master Boot Record'' -- MBR) would parse the partition table, select one of the partitions to boot from (usually through an "active" flag), load the first sector ''of that partition'' (the ''Volume Boot Record'' -- VBR), and execute it. The VBR code would then proceed to load the OS kernel.

Since the VBR code expected to be executed from the exact same memory address as the MBR -- as if it were just loaded by the BIOS -- the MBR code had to relocate itself to a different memory location before loading the VBR code to its previous location. This was called ''chainloading''.

[[LILO]] was an example of this approach.

=== Boot Managers ===

If you wanted a sophisticated selection menu (a "boot manager"), the 440 bytes size limit of the MBR was crippling.

Hard drive addressing at that time followed the Cylinder-Head-Sector (CHS) scheme. The MBR resided in the first sector, first head, first cylinder. For access optimization, the rest of the first cylinder was usually left unassigned, and the first ''partition'' started on the ''second'' cylinder, leaving a gap of unassigned sectors. Many boot managers took advantage of this and stored additional code in this gap. That was convenient, but risky - sometimes other software used those sectors too, potentially destroying the boot manager.

=== Two-Stage Bootloader ===

One solution to that problem was to split up the boot manager into two "stages": A first stage (residing in the MBR) loads the second stage from a partition's file system. The second stage then provides traditional "boot manager" services (a pretty selection menu, decent error handling etc.).

[[GRUB]] is an example of this approach.

=== Multiboot ===

[[GRUB]] is capable of traditional chainloading of VBR's as well as booting Linux kernels directly. But the GRUB developers additionally specified the [http://www.gnu.org/software/grub/manual/multiboot/multiboot.html Multiboot] standard: A Multiboot-compliant kernel could be booted with [[protected mode]] already set up, kernel modules loaded to memory, a memory map provided, and several other niceties. The idea was to provide a standard where any Multiboot-loader could load any Multiboot-kernel.

=== ROM expansion ===

Another way of booting was invented for diskless systems. In this case bootloader code resides in ROM, most commonly on a network card. Instead of booting from disk, it loads the OS over the network.

It is possible to hack such a ROM, and make the BIOS load your bootloader code, which in turn loads your OS. Details can be found in the [http://en.wikipedia.org/wiki/BIOS#BIOS_boot_specification BIOS boot specification].

=== GPT ===

As disks became larger, the limits of the MBR partitioning scheme became an issue. Together with a new boot mechanism ([[EFI]]), a new partitioning scheme called [[GPT]] was devised, using the sectors in the gap after the MBR for its data.

Old MBR-based tools - like the aforementioned boot managers - could accidentially overwrite the GPT data structures. As a signal that a given device is using GPT for its partitioning, its MBR partition table marks the whole device as being one single partition of the new type 0xEE. Within the scope of GPT, the so-marked MBR is called ''Protective Master Boot Record''.

=== BIOS Boot Partition ===

Some people find EFI overcomplicated, and wanted the ability to use a GPT-partitioned device but still boot via the old BIOS / two-stage bootloader mechanism. They introduced a special GPT partition type, the [http://en.wikipedia.org/wiki/BIOS_Boot_partition BIOS Boot Partition], which does not hold any filesystem, but is a place to store the second stage of the bootmanager.

== The goal ==
The goal is to load a code and provide consistent environment for it in different scenarios, such as:
* MBR(UBRL) -> 2nd stage
* MBR -> VBR(UBRL) -> 2nd stage
* PMBR -> BBP -> 2nd stage
* GRUB(chainload) -> VBR -> 2nd stage
* GRUB(fs, as kernel) -> 2nd stage
* VBR(partitionless disk) -> 2nd stage
* BIOS -> expansion ROM -> 2nd stage

The boot code is responsible for finding a dedicated filesystem (marked by an active flag for example), locate standard file(s) on it (the kernel and optionally an initrd or modules), load them into memory and pass control over. To achieve this, first you have to get your boot code loaded.

I'll show you how to write a code that can be loaded by standard BIOS boot method, via Multiboot and from ROM as well.

If you use one of the standard filesystems (ext, fat etc.) I advise to read no further, simply use [[GRUB]] or other MultiBoot compliant boot loader. You can take advantage of a loader of your own only if you want to go down through the rabbit hole, and use your own file system or executable format.

== Finally, the code ==
=== 1st stage ===
This is a minimal code (440 bytes at most) that can be placed in a [[MBR|Master Boot Record]] or GPT's Protective MBR. It's loaded by the BIOS or a boot record via chainload.

The 1st stage code shown here is:
* 100% compatible with the original IBM PC master boot record code
* Same binary can be installed as MBR, PMBR or VBR as well
* Forward GPT compatible on non-EFI machines
* Loads a 2nd stage loader from any partition
* Works with harddisks and USB keys (pendrives). Floppies not supported.
* Requires LBA support, no CHS (could be a problem for BIOSes older than 10 years).
* Tested on several virtual and real machines.

stage1.asm:
<source lang="asm">
;*********************************************************************
;*                                                                   *
;*     OS/3D - written by Zoltan Baldaszti (aka Turdus) in 2008      *
;*        Compilation: fasm -d EFISIZE=1 stage1.asm ubrl.bin         *
;*                                                                   *
;*  Universal Boot Record Loader.                                    *
;*                                                                   *
;*  memory occupied: 0500-800                                        *
;*                                                                   *
;*********************************************************************

;get default EFISIZE, PARTSIZE
;EFISIZE=EFI System partition size in kilobytes
;PARTSIZE=size of your filesystem's partition in kilobytes
include "config.inc"
;--------------------------Macros----------------------------
;the writestr mnemonic. Writes a message on screen.
macro writestr msg
{
if ~ msg eq si
	push	si
	mov	si, msg
end if
	call	writestrfunc
if ~ msg eq si
	pop	si
end if
}

;the die mnemonic. Writes a reason and reboots.
macro die msg
{
if ~ msg eq si
	mov	si, msg
end if
	jmp	diefunc
}

;2nd stage header format (64 bytes):
;offset length  desc
; 0      2      expansion ROM magic (AA55h)
; 2      1      size in blocks (40h)
; 3      1      magic E9h
; 4      2      real mode entry point (relative)
; 6      2      checksum
; 7      1      revision, must be zero
; 8     18      architecture (default "x86_bios")
;26      2      pnp ptr, must be zero
;28      4      flags, must be zero
;32     32      Multiboot header (for GRUB compatibility, must contain a
;               protected mode entry point)
;any format can follow.

ldr.id		equ		800h		;position of 2nd stage loader
ldr.firstbyte	equ		840h		;first non-header byte
ldr.executor	equ		803h		;ptr to init code
ldr.loader	equ		500h		;loader and partition type
ldr.drive	equ		501h		;bios drive of loader
ldr.begin	equ		506h		;lba of loader
ldr.bootdiskid	equ		50Eh		;boot disk unique winnt id
ldr.bootbegin	equ		522h		;lba of boot partition
ldr.bootsize	equ		52Ah		;size of boot partition
lba_packet	equ		532h
virtual at lba_packet
lbapacket.size:	dw		?
lbapacket.count:dw		?
lbapacket.addr0:dw		?
lbapacket.addr1:dw		?
lbapacket.sect0:dw		?
lbapacket.sect1:dw		?
lbapacket.sect2:dw		?
lbapacket.sect3:dw		?
end virtual

;*********************************************************************
;*                             code                                  *
;*********************************************************************

;-----------------BIOS called ENTRY POINT--------------------
		ORG		0600h
		USE16
universal_boot_record:
		cli
		jmp		short .skipid
.system:	db		"UBRL"		;Universal Boot Record Loader
.version:	db		0		;version 0.0.1beta

.skipid:	;NOTE: not sure about 32 bit processor yet, so we have to
		;use only 16 bit operands
		;relocate our code to offset 600h
		xor		ax, ax
		mov		ss, ax
		mov		sp, 600h
		push		ax
		pop		es
		push		cs
		pop		ds
		;find our position in memory.
		call		.getaddress
.getaddress:	pop		bx
		sub		bx, .getaddress-universal_boot_record
		mov		si, bx
		cld
		mov		di, sp
		;clear data area 500h-600h
		sub		di, 100h
		mov		cx, 80h
		repnz		stosw
		;and copy ourselves to 600h
		mov		cx, 100h
		repnz		movsw
		jmp		0:.start

.start:		;get boot drive code - original MBR and BIOS passes it in dl,
		;so all bootmanagers should do.

		;save drive code
		mov		byte [ldr.drive], dl
		;initialize lba packet
		mov		byte [lbapacket.size], 16
		mov		byte [lbapacket.count], 58
		mov		byte [lbapacket.addr0+1], 08h
		push		dx
		writestr		.system
		pop		dx
		;check for lba presistance - floppy not supported any more
		;we use pendrive as removable media for a long time
		cmp		dl, byte 80h
		jl		.nolba
.notfloppy:	mov		ah, byte 41h
		mov		bx, word 55AAh
		int		13h
		jc		.nolba
		cmp		bx, word 0AA55h
		jne		.nolba
		test		cl, byte 1
		jnz		.lbaok
.nolba:		die		lbanotf
.lbaok:		;try to load stage2 - it's a continous area on disk
		;started at given sector with maximum size of 7400h bytes
		;doesn't matter if we can load it or not,
		;we still want to search for partitions
		mov		si, stage2_addr
		mov		di, lbapacket.sect0
		push		di
		movsw
		movsw
		movsw
		movsw
		call		loadsectorfunc

		;we have to load the MBR beacuse we may be loaded
		;as a volume boot record
		mov		byte [lbapacket.addr0+1], 07Ch
		pop		di
		xor		ax, ax
		stosw
		stosw
		stosw
		stosw
		;actually we load more than 2 sectors, but it's fast and
		;does not bother anybody. The code is smaller this way,
		;and nevertheless pre-caching fasten up overall boot time.
		call		loadsectorfunc
		jc		.nombr
		mov		bp, 07DFEh
		cmp		word [bp], bx
		je		.mbrok
.nombr:		die		mbrnotf
.mbrok:		inc		byte [bp]	;mess up id
		;copy disk id from MBR
		;NOTE: si now points to 7DB8h
		mov		di, diskid
		movsw
		movsw

		;searching for active partition (skip over 2 more zero bytes)
		inc		si
		inc		si
		;check for EFI partitioning scheme (so we will not depend on
		;special partition record EEh and such)
		;NOTE: bp+2 is 7E00h
		cmp		word [bp+2], 'EF'
		jne		.nextpartition
		cmp		word [bp+4], 'I '
		jne		.nextpartition
		;generate fake MBR partition entry for it
		xor		dx, dx
		xor		cx, cx
		inc		cx
		jmp		near .partitionok

.nextpartition:	cmp		si, bp
		jb		.ok
		;indicate no boot partition found
		mov		byte [ldr.loader], 16
		jmp		near .chk
.ok:		mov		ax, word [si+12]
		mov		word [ldr.bootsize], ax
		mov		al, byte [si]
		mov		cx, word [si+8]
		mov		dx, word [si+10]
		add		si, word 16
		cmp		al, byte 80h		;check for active partition
		jb		.nextpartition
		;this is a dirty hack. It allows to boot from a different disk.
		;this should not be used, but we must keep compatibility with
		;the M$ boot record shit. Anyway, if you use 80h, we won't
		;change the device code that the BIOS gave us.
		je		.partitionok
		mov		byte [ldr.drive], al
		;save the partition info
.partitionok:	mov		word [ldr.bootbegin], cx
		mov		word [ldr.bootbegin+2], dx
		;load partition's (or GPT table's) first 58 sector
		mov		word [lbapacket.sect0], cx
		mov		word [lbapacket.sect1], dx
		call		loadsectorfunc
		;do we have a 2nd stage loader?
.chk:		cmp		word [ldr.id], bx
		jne		.nostage2
		cmp		bzte [ldr.id+3], 0E9h
		jne		.nostage2
		;invoke stage2 real mode code
		writestr		okay
		jmp		ldr.executor

.nostage2:	;if no 2nd stage loader, continue with standard boot mechanism
		;check if it's a valid boot record
		cmp		word [bp], bx
		jne		.noos
		;do not load ourself again, that would be a loop forever
		cmp		word [bp+2+.system-universal_boot_record], 'UB'
		je		.noos
		writestr		okay
		;dl must contain bootdrive
		mov		dl, byte [ldr.drive]
		jmp		07C00h
.noos:		die		osnotfound

;*********************************************************************
;*                          functions                                *
;*********************************************************************

;loads an LBA sector
loadmbr:
loadsectorfunc:
		push		bx
		push		si
		mov		ah, byte 42h
		mov		dl, byte [ldr.drive]
		mov		si, lba_packet
		int		13h
		pop		si
		pop		bx
		ret

;ds:si zero terminated string to write
writestrfunc:
		lodsb
		or		al, al
		jz		.end
		mov		ah, byte 0Eh
		mov		bx, word 11
		int		10h
		jmp		writestrfunc
.end:		ret

;writes the reason, waits for a key and reboots.
diefunc:
		writestr		panic
		call 		writestrfunc
		mov		si, found
		call		writestrfunc
		xor		ax, ax
		int		16h
		mov		al, 0FEh
		out		64h, al
;		hlt
		jmp		far 0FFFFh:0		;invoke BIOS POST

;*********************************************************************
;*                          data area                                *
;*********************************************************************

panic:		db		"-PANIC: no ",0
lbanotf:	db		"LBA support",0
mbrnotf:	db		"PMBR",0
osnotfound:	db		"operating system",0
found:		db		" found",0
okay:		db		".",10,13,0
		db		01B0h-($-$$) dup 0

;right before the partition table some data
stage2_addr:	dd		0FFFFFFFFh,0	;1B0h 2nd stage loader address
diskid:		dd		012345678h	;1B8h WinNT expects it here
		dw		0

		;fake partition tables
		;EFI Partition
		db		0h		;bootable
		db		0FEh,0FFh,0FFh	;CHS not used
		db		0EEh		;fs type
		db		0FEh,0FFh,0FFh	;CHS not used
		dd		1		;start LBA
		dd		0ffffffffh	;end LBA
		;OS/3D root, modify to your needs
		db		80h		;bootable
		db		0FEh,0FFh,0FFh	;CHS not used
		db		03dh		;fs type
		db		0FEh,0FFh,0FFh	;CHS not used
		dd		(EFISIZE*2)+8	;start LBA
		dd		0ffffffffh	;end LBA

		db              01FEh-($-$$) dup 0
		db		55h,0AAh

;*********************************************************************
;*                   end of Universal Boot Record                    *
;*********************************************************************

;*********************************************************************
;*                      GUID Partition Table                         *
;*********************************************************************
;NOTE: the image creator must calculate correct checksums.
gpt_header:
		;gpt header
.signature:	db		"EFI PART"
.revision:	dw		0,1
.size:		dd		92
.crc:		dd		0DEADCC32h	;not used, but should be valid
.reserved:	dd		0
.primarylba:	dd		1,0
.backuplba:	dd		1,0		;not used, but should be valid
.firstusable:	dd		64,0
.lastusable:	dd		0FFFFFFFFh,0	;not used, but should be valid
.diskuuid:	dd		1,2,3,4
.partitionlba:	dd		2,0
.numentries:	dd		2
.sizeentry:	dd		128
.partitioncrc:	dd		0DEADCC32h	;not used, but should be valid
		db		0200h-($-gpt_header) dup 0

gpt_partitions:
.entry0_type:	db		028h,073h,02Ah,0C1h,01Fh,0F8h,0D2h,011h
		db		0BAh,04Bh,0,0A0h,0C9h,03Eh,0C9h,03Bh
.entry0_uuid:	db		1,2,3,4,1,2,3,4,1,2,3,4,1,2,3,4
.entry0_start:	dd		8,0
.entry0_ends:	dd		(EFISIZE*2)+8,0
.entry0_attrib:	dd		0,0
.entry0_name:	db		'E',0,'F',0,'I',0,' ',0,'s',0,'y',0,'s',0
		db		't',0,'e',0,'m',0,' ',0,'p',0,'a',0,'r',0
		db		't',0,'i',0,'t',0,'i',0,'o',0,'n'
		db		128-($-.entry0_type) dup 0

;our test OS/3D system partition
;my stage2 code will look for BOOTDISK first, but if none found, it will use the
;first partition labeled ROOTDISK.
;modify this entry to your needs.
.entry1_type:	db		'FS3D',0,1,0,0,'ROOTDISK'
.entry1_uuid:	db		1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16
.entry1_start:	dd		(EFISIZE*2)+8,0
.entry1_ends:	dd		(EFISIZE*2)+(PARTSIZE*2)+8,0
.entry1_attrib:	dd		0,0
.entry1_name:	db		'B',0,'e',0,'n',0,'d',0,'e'
		db		128-($-.entry1_type) dup 0

;-----------make it logical sector (4k) long----------
		db		4096-($-universal_boot_record) dup 0
</source>

=== 2nd stage ===
The rest of the code can be right after the 1st stage, although it's discouraged due to compatibility issues (see section GPT). It's better to put it on a separate mbr or gpt partition (to protect it from overwriting by accident). Even better, store it in a defragmented, continuous file on your filesystem (note that being continous is essential since 1st stage cannot parse your fs). Do not forget to record the starting LBA address in 1st stage before using. This gives you the ability to use different 2nd stage code each time you reboot.

This is just a skeleton to aid you. It will not parse your filesystem, or set up the environment for your kernel. You have to do these on your own. But everything else is done so you can focus on these important matters.

stage2.asm:
<source lang="asm">
;*********************************************************************
;*                                                                   *
;*     OS/3D - written by Zoltan Baldaszti (aka Turdus) in 2008      *
;*              Compilation: fasm stage2.asm stage2.bin              *
;*                                                                   *
;*  Example 2nd stage loader for UBRL, compatible with GRUB and      *
;*  BIOS boot specification 1.0.1 (expansion ROM) too.               *
;*                                                                   *
;*  memory occupied: 800-7C00                                        *
;*                                                                   *
;*********************************************************************

;-----------header section-------------
;UBR header
                USE16
                ORG                     800h
loader:         db                      55h,0AAh                ;ROM magic
                db                      (loader_end-loader)/512 ;size in 512 blocks
.executor:      jmp                     near realmode_start     ;entry point
.checksum:      dw                      0                       ;checksum
@@:             db                      "Your OS name here"
                db                      18-($-@b) dup 0
.pnpptr:        dw                      0
.flags:         dd                      0
;GRUB header
MB_MAGIC	equ			01BADB002h
MB_FLAGS	equ			01000h
		align			8
.mb_header:	dd			MB_MAGIC		;magic
		dd			MB_FLAGS		;flags
		dd			-(MB_MAGIC+MB_FLAGS)	;checksum (0-magic-flags)
		dd			.mb_header		;our location (GRUB should load us here)
		dd			0800h			;the same... load start
		dd			07C00h			;load end
		dd			0h			;no bss
		dd			multiboot_start		;entry point

;-----------realmode-protmode stub-------------
;MEMORY LAYOUT ON EXECUTION:
;500h		loader and partition type
;501h		bios drive code of loader
;506h		starting lba of this stage on disk
;50Eh		boot disk unique winnt id
;522h		starting lba of MBR boot partition/GPT table
;52Ah		size of MBR boot partition/GPT table
;800h		LDRF header
;7C00h		MBR boot partition/GPT table preloaded (first 16 sectors)
;		appropriate for chainloading

realmode_start: cld
                cli
                mov                     sp, 800h
                ;relocate ourself from ROM to RAM if necessary
                call                    .getaddress
.getaddress:    pop                     si
                mov                     ax, cs
                or                      ax, ax
                jnz                     .reloc
                cmp                     si, .getaddress
                je                      .noreloc
.reloc:         mov                     ds, ax
                xor                     ax, ax
                mov                     es, ax
                mov                     di, loader
                sub                     si, .getaddress-loader
                mov                     cx, ((loader_last+8)-loader)/2
                repnz                   movsw
                xor                     ax, ax
                mov                     ds, ax
                jmp                     0:.relocated
.relocated:	;you'll need to locate boot partition
		;and load it to 7C00h, because neither
		;BIOS nor UBRL have done this for you
		;in this case (loaded from ROM)
.noreloc:
		;do any real mode initialization here
		;(e820, A20 gate etc.)

		lgdt			[GDT_value]
		mov			eax, cr0		;enable protected mode
		or			al, 1
		mov			cr0, eax
		jmp			16:protmode_start

		;global descriptor table
		align			8
GDT_table:	dd			0, 0			;null descriptor
		dd			0000FFFFh,008F9200h	;flat ds
		dd			0000FFFFh,00CF9A00h	;32-bit ring0 cs
GDT_value:	dw			$-GDT_table
		dd			GDT_table

;----------------Multiboot stub-----------------
;MEMORY LAYOUT ON EXECUTION: see Multiboot spec
		USE32
multiboot_start:
		cld
		cli
		;you'll need to locate boot partition
		;and load it to 7C00h, because GRUB
		;hasn't done this for you
		lgdt			[GDT_value]
		cmp			eax, 2BADB002h
		je			protmode_start
		;no GRUB environment available?
		;something really nasty happened, restart computer
		mov			al, 0FEh
		out			64h, al
		hlt

protmode_start:	mov			ax, 8
		mov			ds, ax
		mov			es, ax
		mov			fs, ax
		mov			gs, ax
		mov			ss, ax
		mov			esp, 800h

		;do any prot mode initialization here
		;(irq remap, paging etc.)

;-----------longmode stub if you want-------------
		;enable long mode

;*********************************************************************
;*                        your loader code                           *
;*********************************************************************
		;put your own kernel loader code here.
		;parse your executable format here.
		;call your kernel's main here.

;-----------padding to be multiple of 512----------
                db                      (511-($-loader+511) mod 512) dup 0
loader_end:
;-----------BIOS checksum------------
chksum = 0
repeat $-loader
                load b byte from (loader+%-1)
                chksum = (chksum + b) mod 100h
end repeat
                store byte (100h-chksum) at (loader.checksum)

;-----------bound check-------------
		;fasm will generate an error if your code
		;is bigger than it should be
		db			07400h-($-loader) dup ?
</source>

== Testing ==
=== UBRL in MBR ===
Quite straightforward, create a disk image with UBRL in MBR, and record stage2.bin's position at 1B0h.
=== UBRL in VBR ===
Create a disk image with your favourite MBR, and put UBRL in the 1st sector of a partition. You still have to record stage2.bin's position at 1B0h in MBR.
=== GRUB Multiboot ===
Specify stage2.bin as "kernel" in menu.lst.
=== BIOS ROM ===
Use bochs. Configure optional ROM, specify stage2.bin as ROM's name, and 0xC0000+(vga_bios_size rounded to 2k) as address (as of writing, bochs vga bios is 40960 bytes long, so the address will be 0xCA000). Also modify boot options to boot from "network". Ready to go!

[[Category:Bootloaders]]
````

<a id="w003174"></a>

## W003174 — User:Lionel

Bron-ID: W003174
Namespace: User (2)
Revisie: 15508 | Datum: 2014-01-04T05:24:52Z
Bron: https://wiki.osdev.org/User:Lionel
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALionel&oldid=15508

````text
<source lang="bash">
Lionels-MacBook-Pro:~ lionel$ printbio
-bash: printbio: command not found
</source>

== Wiki Pages ==
{{Special:Prefixindex/User:Lionel/}}
== Projects ==
Beryllium [https://github.com/Lionel07/Beryllium]

elementOS (DEPRECIATED)[https://github.com/Lionel07/elementOS]

Chronos (DEPRECIATED)[https://github.com/Lionel07/Chronos-Legacy]
````

<a id="w003175"></a>

## W003175 — User:Hakonmagnus

Bron-ID: W003175
Namespace: User (2)
Revisie: 17859 | Datum: 2015-04-06T10:29:17Z
Bron: https://wiki.osdev.org/User:Hakonmagnus
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AHakonmagnus&oldid=17859

````text
Hello, my name is Hákon Magnús, and i am a programmer from Iceland.

I have a couple years of programming experience, and i am working on a small software company. I'm currently working on an operating system for the PC. I've been interested in OS development for a while now, and i have a programming style of "reinventing the wheel" because i love starting from ground-up and seeing everything build up :)

== FireOS ==
FireOS is an operating system for the x86 platform PC which i am currently developing. It has its own file system and format and is from ground up.
````

<a id="w003176"></a>

## W003176 — SimNow

Bron-ID: W003176
Namespace: Main (0)
Revisie: 14207 | Datum: 2013-01-07T09:29:55Z
Bron: https://wiki.osdev.org/SimNow
Revisie-URL: https://wiki.osdev.org/index.php?title=SimNow&oldid=14207

````text
'''SimNow''' is an emulator/debugger from AMD originally targeted at their 64-bit processors, but also applicable to Intel processors. It is a very useful tool for debugging at the assembler level; particularly helpful when debugging boot loaders or the early stages of OS code. You can single-step through code and watch memory and registers changing. Although usable as an emulator, SimNow is a little slow for that purpose.

SimNow is available for free for both Windows and Linux. See http://developer.amd.com/tools/cpu-development/simnow-simulator/ .

[[Category:Emulators]]
````

<a id="w003177"></a>

## W003177 — AMD PCNET

Bron-ID: W003177
Namespace: Main (0)
Revisie: 28252 | Datum: 2023-07-18T00:32:29Z
Bron: https://wiki.osdev.org/AMD_PCNET
Revisie-URL: https://wiki.osdev.org/index.php?title=AMD_PCNET&oldid=28252

````text
The '''AMD PCNET''' family of network interface chips are supported by most popular virtual machines and emulators, including QEMU, VMware and VirtualBox. While not as simple as the [[RTL8139]] it is easier to test with an emulator, as the RTL8139 is only supported in QEMU, and getting QEMU's full network support running is sometimes difficult. This article will focus on the Am79C970A a.k.a. the AMD PCnet-PCI II in VirtualBox.

== Overview ==
The PCnet-PCI II is a PCI network adapter.  It has built-in support for CRC checks and can automatically pad short packets to the minimum Ethernet length.

It supports PCI bus mastering and can operate in both 32-bit mode and a legacy 16-bit compatibility mode (this mode is from now on referred to as software style, or SWSTYLE).  Access to the card's registers are through an index/data register system in either IO port space or memory mapped IO.  Given that the MMIO access is sometimes absent on emulators or certain systems, this article will focus on the IO port access.  A final distinction is made between the actual accesses to the index/data registers, which can either be 16-bit or 32-bit.  The 32-bit mode is referred to as DWIO in the specifications (as it implies the DWIO bit is set in a particular register).  Note that any combination of DWIO and SWSTYLE can be selected.

== Initialization and Register Access ==

=== PCI Configuration ===

In the PCI configuration space, the card has vendor ID 0x1022 and device ID 0x2000.  A separate similar device (PCnet-PCI III and clones) has device ID 0x2001 and is programmed similarly.

The first task of the driver should be to enable the IO ports and bus mastering ability of the device in PCI configuration space.  This is done by setting bits 0 and 2 of the control register, e.g. <source lang="c">uint32_t conf = pciConfigReadDWord(bus, slot, func, offset);
conf &= 0xffff0000; // preserve status register, clear config register
conf |= 0x5;        // set bits 0 and 2
pciConfigWriteDWord(bus, slot, func, offset, conf);</source>

You will then want to read the IO base address of the first BAR from configuration space.  We will assume this has the value io_base.

=== Card register access ===

As stated above, the card supports a index/data system of accessing its internal registers.  This means that the index of the register you wish to access is first written to an index port, followed by either writing a new value to or reading the old value from a data register.  To make things slightly more complex, however, the card splits its internal registers into two groups - Control and Status Registers (CSR) and Bus Control Registers (BCR).  Both share a common index port (called Register Address Port - RAP), but use separate data ports: the Register Data Port (RDP) for CSRs and the BCR Data Port (BDP) for BCRs.  During normal initialization and use of the cards, the CSRs are used exclusively.  A further important register exists in the IO space called the reset register.

A further complication exists in that the offsets of the RAP, Reset register and BDP (but not RDP) relative to io_base vary depending on the current value of DWIO:
{| {{wikitable}}
! colspan="2" | DWIO = 0 (16-bit access)
! colspan="2" | DWIO = 1 (32-bit access)
! Register
|-
! Offset
! Length
! Offset
! Length
!
|-
| 0 || 16 || 0 || 16 || First 16 bytes of EPROM (the first 6 bytes are MAC address)
|-
| 0x10 || 2 || 0x10 || 4 || RDP - data register for CSRs
|-
| 0x12 || 2 || 0x14 || 4 || RAP - index register for both CSR and BCR access
|-
| 0x14 || 2 || 0x18 || 4 || Reset register
|-
| 0x16 || 2 || 0x1c || 4 || BDP - data register for BCRs
|}

In addition, the card requires different length data accesses to the registers depending on the setting of DWIO: if DWIO=0, then offsets 0x0 through 0xf are read as single bytes, all others read/written as 16-bit words; if DWIO=1, then all registers (including 0x0 through 0xf) are read/written as 32-bit double words (and with accesses aligned on 32-bit boundaries).  We can write functions to access the registers:<source lang="c">void writeRAP32(uint32_t val)
{
    outd(io_base + 0x14, val);
}

void writeRAP16(uint16_t val)
{
    outw(io_base + 0x12, val);
}

uint32_t readCSR32(uint32_t csr_no)
{
    writeRAP32(csr_no);
    return ind(io_base + 0x10);
}

uint16_t readCSR16(uint16_t csr_no)
{
    writeRAP32(csr_no);
    return inw(io_base + 0x10);
}

void writeCSR32(uint32_t csr_no, uint32_t val)
{
    writeRAP32(csr_no);
    outd(io_base + 0x10, val);
}

void writeCSR16(uint16_t csr_no, uint16_t val)
{
    writeRAP16(csr_no);
    outw(io_base + 0x10, val);
}</source>
and similar functions for BCRs.

Unfortunately it is difficult to determine the current state of DWIO (and therefore know which state the card is in when the driver initializes) as the only way of reporting it is to read BCR18 bit 7, which in turn requires knowledge of the BDP, which requires knowledge of DWIO etc.  Fortunately, following a reset (either hard or soft), the card is in a known state with DWIO=0 (16-bit access).  Normally, therefore, when your driver takes control of the card, it would expect to assume it is in 16-bit mode.  However, it may be the case that firmware or a boot-loader has already initialized the card into 32-bit mode, which you didn't know about.  You should, therefore, reset the card when your driver takes control.  This is accomplished by a read of the reset register:<source lang="c">ind(io_base + 0x18);
inw(io_base + 0x14);</source>Note this snippet reads first from the 32-bit reset register: if the card is in 32-bit mode this will trigger a reset, if in 16-bit mode it will simply read garbage without affecting the card.  It then reads from the 16-bit reset register: if the card was initially in 32-bit mode, it has since been reset and will now be reset again, otherwise it will reset for the first time.

You should now wait 1 microsecond for the reset to complete (using your OSs timing functions).

Then, if desired, you can program the card into 32-bit mode (the rest of this article assumes this, but you can easily substitute read/writeCSR32 with read/writeCSR16 if you like).  To do this, we simply need to perform a 32 bit write of 0 to the RDP.  After reset, RAP points to CSR0, so we are effectively writing 0 to CSR0.  This will not cause any harm as we completely reprogram CSR0 later anyway.<source lang="c">outd(io_base + 0x10, 0);</source>

=== Interrupt handling ===

The next section will enable some interrupts on the card.  We will flesh out the interrupt handler later, but you should install the interrupt handler here as otherwise you will get crashes due to unhandled interrupts.  You need to parse [[ACPI]] tables etc. to determine the proper interrupt routing for your device.

=== SWSTYLE ===

We now need to set the value of SWSTYLE to 2.  After reset, it defaults to 0 representing 16-bit legacy compatibility mode.  We want the card to be able to access all of the first 4 GiB of (physical) memory for its buffers, so need to set it to 32-bit mode.<source lang="c">uint32_t csr58 = readCSR32(58);
csr58 &= 0xFF00;  // SWSTYLE is 8 bits (7:0)
csr58 |= 2;
writeCSR32(58, csr58);</source>

=== ASEL ===

The card has both 10/100baseT and coaxial outputs.  It has functionality to automatically select whichever is attached which is normally enabled by default.  This snippet simply ensures this functionality is enabled by setting the ASEL bit in BCR2 just in case firmware has altered this for some reason.<source lang="c">uint32_t bcr2 = readBCR32(2);
bcr2 |= 0x2;
writeBCR32(2, bcr2);</source>

=== Ring buffers ===

The card uses two ring buffers to store packets: one for packets received and one for packets to be transmitted.  The actual ring buffers themselves are regions of physical memory containing a set number of descriptor entries (DEs) which are fixed 16 bytes in length (for SWSTYLE=2).  Each of these then contains a pointer to the actual physical address of the memory used for the packet.

For example, if you wish to define 32 receive buffers and 8 transmit buffers (similar to what the Linux driver does), then you would need to allocate 32 * 16 bytes for the receive DEs, 8 * 16 bytes for the transmit DEs, 32 * packet length (1544 is used in Linux, but we will use 1520 as it is a multiple of 16) for the actual receive buffers and 8 * packet length for the actual transmit buffers.

The DEs contain a number of important bits for sending/receiving packets, e.g. destination MAC address, error bits etc. but they also contain an important bit called the ownership bit (bit 7 of byte 7).  If this is cleared, it means the driver 'owns' that particular ring buffer entry.  If it is set, it means the card owns it (and the driver should not touch the entire entry).  The way this works is that the only party (driver or card) that can read/write the entry is the one that owns it, and particularly only the owning party can flip ownership back to the other party.  At initialization, you would want the card to 'own' all the receive buffers (OWN = 1) (so it can write new packets into them that it receives, then flip ownership to the driver), and the driver to 'own' all the transmit buffers (OWN = 0) (so it can write packets to be transmitted, then flip ownership to the card).  For the Transmit Ring, remember to not set the OWNer bit until you have a buffer to transmit.

You should also have a variable that stores the current 'pointer' into each buffer (i.e. what is the next one the driver expects to read/write).  The card maintains separate pointers internally.  You also need a simple way of incrementing the pointer (and wrapping back to the start if necessary).

Thus to initialize the ring buffers you'd want something like:<source lang="c">int rx_buffer_ptr = 0;
int tx_buffer_ptr = 0;                 // pointers to transmit/receive buffers

int rx_buffer_count = 32;              // total number of receive buffers
int tx_buffer_count = 8;               // total number of transmit buffers

const int buffer_size = 1520;          // length of each packet buffer

const int de_size = 16;                // length of descriptor entry

uint8_t *rdes;                         // pointer to ring buffer of receive DEs
uint8_t *tdes;                         // pointer to ring buffer of transmit DEs

uint32_t rx_buffers;                   // physical address of actual receive buffers (< 4 GiB)
uint32_t tx_buffers;                   // physical address of actual transmit buffers (< 4 GiB)

// does the driver own the particular buffer?
int driverOwns(uint8_t *des, int idx)
{
    return (des[de_size * idx + 7] & 0x80) == 0;
}

// get the next transmit buffer index
int nextTxIdx(int cur_tx_idx)
{
    int ret = cur_tx_idx + 1;
    if(ret == tx_buffer_count)
        ret = 0;
    return ret;
}

// get the next receive buffer index
int nextRxIdx(int cur_rx_idx)
{
    int ret = cur_rx_idx + 1;
    if(ret == rx_buffer_count)
        ret = 0;
    return ret;
}

// initialize a DE
void initDE(uint8_t *des, int idx, int is_tx)
{
    memset(&des[idx * de_size], 0, de_size);
    
    // first 4 bytes are the physical address of the actual buffer
    uint32_t buf_addr = rx_buffers;
    if(is_tx)
        buf_addr = tx_buffers;
    *(uint32_t *)&des[idx * de_size] = buf_addr + idx * buffer_size;

    // next 2 bytes are 0xf000 OR'd with the first 12 bits of the 2s complement of the length
    uint16_t bcnt = (uint16_t)(-buffer_size);
    bcnt &= 0x0fff;
    bcnt |= 0xf000;
    *(uint16_t *)&des[idx * de_size + 4] = bcnt;

    // finally, set ownership bit - transmit buffers are owned by us, receive buffers by the card
    if(!is_tx)
        des[idx * de_size + 7] = 0x80;
}</source>

=== Card registers setup ===

Finally, once all our ring buffers are set up, we need to give their addresses to the card.  There are two ways of setting up the card registers: we can either program them all directly, or set up a special initialization structure and then pass that to the card.  In this article we will use the latter.

You will need to allocate a 28 byte region of physical memory, aligned on a 32-bit boundary.  The members are:

{| {{wikitable}}
! Offset (bytes)
! Byte 3
! Byte 2
! Byte 1
! Byte 0
|-
| 0 || TLEN(7:4)* || RLEN(7:4)* || colspan="2" | MODE
|-
| 4 || MAC[3] || MAC[2] || MAC [1] || MAC[0] 
|-
| 8 || colspan="2" | Reserved (0) || MAC[5] || MAC[4]
|-
| 12 || LADR[3] || LADR[2] || LADR[1] || LADR[0]
|-
| 16 || LADR[7] || LADR[6] || LADR[5] || LADR[4]
|-
| 20 || colspan="4" | Physical address of first receive descriptor entry
|-
| 24 || colspan="4" | Physical address of first transmit descriptor entry
|}

Note that TLEN and RLEN are the log2 of the number of transmit and receive descriptor entries respectively and are in the high order nibble of the byte while the lower nibble of the byte is reserved.  For example, if you have 8 transmit descriptor entries, TLEN would be 3 (as 2^3 = 8). The maximum value of TLEN and RLEN is 9 (i.e. 512 buffers).  However, after initialization, you may change this to a maximum of 65535 byte writing to the CSR76 and CSR78 registers.

MODE provides various functions to control how the card works with regards to sending and receiving packets, and running loop-back tests.  You probably want to set it to zero (enable transmit and receive functionality, receive broadcast packets and those sent this physical address, disable promiscuous mode).  See the spec description of CSR15 for further details.

You also need to specify the physical address (MAC address) you want the card to use.  If you want to keep the current one, you will need to first read it from the EPROM of the card (it is exposed as the first 6 bytes of the IO space that the registers are in).

LADR is the logical address filter you want the card to use when deciding to accept Ethernet packets with logical addressing.  If you do not wish to use logical addressing (the default), then set these bytes to zero.

To actually set up the card registers, we provide it with the address of our initialization structure by writing the low 16-bits of its address to CSR1 and the high 16-bits to CSR2.

You can also set up other registers at this point, e.g. CSR3 (only interesting bits shown):
{| {{wikitable}}
! Bit number
! Functionality
|-
| 10 || Receive interrupt mask - if set then incoming packets won't trigger an interrupt
|-
| 9 || Transmit interrupt mask - if set then an interrupt won't be triggered when a packet has completed sending.  Depending on your design this may be preferable.
|-
| 8 || Interrupt done mask - if set then you won't get an interrupt when the card has finished initializing.  You probably want this as it is far easier to poll for this situation (which only occurs once anyway).
|-
| 2 || Big-endian enable - you will want to ensure this is cleared to zero
|}

And you may want to set bit 11 of CSR4 which automatically pads Ethernet packets which are too short to be at least 64 bytes.

Once all the control registers are set up, you set bit 0 of CSR0, and then wait for initialization to be done.  You can do this by either waiting for an interrupt (if you didn't disable the initialization done interrupt in CSR3) or by polling until CSR0 bit 8 is set.  Note that if you want to wait for an interrupt you will also need to set bit 6 of CSR0 or interrupts won't be generated (you will need to enable this anyway to get notification of received packets, so it makes sense to set it at the same time as the initialization bit).

Once initialization has completed, you can finally start the card.  This is accomplished by clearing both the INIT bit (bit 0) and STOP bit (bit 2) in CSR0 and setting the STRT bit (bit 1) at the same time.

== Sending packets ==

Sending packets involves simply writing the packet details to the next available transmit buffer, then flipping the ownership for the particular ring buffer entry to the card.  The card regularly scans all the transmit buffers looking for one it hasn't sent, and then will transmit those it finds.

For example:<source lang="c">int sendPacket(void *packet, size_t len, uint8_t *dest)
{
    // the next available descriptor entry index is in tx_buffer_ptr
    if(!driverOwns(tdes, tx_buffer_ptr))
    {
        // we don't own the next buffer, this implies all the transmit
        //  buffers are full and the card hasn't sent them yet.
        // A fully functional driver would therefore add the packet to
        //  a queue somewhere, and wait for the transmit done interrupt
        //  then try again.  We simply fail and return.  You can set
        //  bit 3 of CSR0 here to encourage the card to send all buffers.
        return 0;
    }

    // copy the packet data to the transmit buffer.  An alternative would
    //  be to update the appropriate transmit DE to point to 'packet', but
    //  then you would need to ensure that packet is not invalidated before
    //  the card has a chance to send the data.
    memcpy((void *)(tx_buffers + tx_buffer_ptr * buffer_size), packet, len);

    // set the STP bit in the descriptor entry (signals this is the first
    //  frame in a split packet - we only support single frames)
    tdes[tx_buffer_ptr * de_size + 7] |= 0x2;

    // similarly, set the ENP bit to state this is also the end of a packet
    tdes[tx_buffer_ptr * de_size + 7] |= 0x1;

    // set the BCNT member to be 0xf000 OR'd with the first 12 bits of the
    //  two's complement of the length of the packet
    uint16_t bcnt = (uint16_t)(-len);
    bcnt &= 0xfff;
    bcnt |= 0xf000;
    *(uint16_t *)&tdes[tx_buffer_ptr * de_size + 4] = bcnt;

    // finally, flip the ownership bit back to the card
    tdes[tx_buffer_ptr * de_size + 7] |= 0x80;

    // update the next transmit pointer
    tx_buffer_ptr = nextTxIdx(tx_buffer_ptr);
}</source>

== Handling interrupts and receiving packets ==

Receiving packets is normally done in your interrupt handler - the card will signal an interrupt whenever it receives a packet and has written it to the receive buffer.

Note that interrupts can come from many sources (other than new packets).  If a new packet has been signaled then CSR0 bit 10 will be set.  There are other bits in CSR0 than can be set (depending on how you set up interrupt masks in CSR3) and additionally other bits in CSR4 that can signal interrupts (although these are usually masked out on reset).  After you have properly handled an interrupt, you will need to write a 1 back to the appropriate bit in CSR0 or CSR4 before sending EOI to you interrupt controller (or the interrupt will continue to be signaled).  Bitwise OR CSR0 with 0x7F00 and bitwise OR CSR4 with 0x022A will reset all interrupts.  Remember to preserve bit 6 in CSR0 and bit 11 in CSR4.

Once a receive packet interrupt has been received, you need to loop through the receive descriptor entries (starting at rx_buffer_ptr) handling each packet until you find an entry which the driver doesn't own, then stop.  e.g. <source lang="c">void handleReceiveInterrupt()
{
    while(driverOwns(rdes, rx_buffer_ptr))
    {
        // packet length is given by bytes 8 and 9 of the descriptor
        //  (no need to negate it unlike BCNT above)
        uint16_t plen = *(uint16_t *)&rdes[rx_buffer_ptr * de_size + 8];

        // the packet itself is written somewhere in the receive buffer
        void *pbuf = (void *)(rx_buffers + rx_buffer_ptr * buffer_size);

        // do something with the packet (i.e. hand to the next layer in the
        //  network stack).  You probably don't want to do any extensive
        //  processing here (as this is within an interrupt handler) - just
        //  copy the data somewhere to a queue and continue so that the
        //  system is interrupted for as little time as possible
        handlePacket(pbuf, plen);

        // hand the buffer back to the card
        rdes[rx_buffer_ptr * de_size + 7] = 0x80;

        // increment rx_buffer_ptr;
        rx_buffer_ptr = nextRxIdx(rx_buffer_ptr);
    }

    // set interrupt as handled
    writeCSR32(0, readCSR32(0) | 0x0400);

    // don't forget to send EOI
}</source>

== Important Registers ==

=== CSR0: "The Controller Status Register" ===

The CSR0 is a register that contains flags indicating the cause of an interrupt. Writing a 1 to a flag's bit position clears the corresponding flag, allowing drivers to clear the interrupt flags by simple reading and writing back the read value to the register.

{| {{wikitable}}
|-
! Bit !! Name !! Description
|-
| 31-16 || RES || Reserved. Written as ZEROs and read as undefined.
|-
| 15 || ERR || Error flag that is set when any of the error flags (BABL, CERR, MISS, MERR and ERR) are true.
|-
| 14 || BABL || Babble flag indicating a transmitter timeout error.
|-
| 13 || CERR || Collision Error flag indicating a failure in collision detection.
|-
| 12 || MISS || Missed Frame flag indicating the loss of an incoming receive frame.
|-
| 11 || MERR || Memory Error flag indicating a timeout waiting for the bus.
|-
| 10 || RINT || Receive Interrupt flag indicating the completion of a receive frame.
|-
| 9 || TINT || Transmit Interrupt flag indicating the completion of a transmit frame.
|-
| 8 || IDON || Initialization Done flag indicating the completion of the initialization sequence.
|-
| 7 || INTR || Interrupt Flag indicating the occurrence of one or more interrupt-causing conditions.
|-
| 6 || IENA || Interrupt Enable flag enabling INTA to be active if the Interrupt Flag is set.
|-
| 5 || RXON || Receive On flag indicating that the receive function is enabled.
|-
| 4 || TXON || Transmit On flag indicating that the transmit function is enabled.
|-
| 3 || TDMD || Transmit Demand flag causing buffer management unit access without waiting for the poll-time counter.
|-
| 2 || STOP || Stop flag disabling the chip from all DMA and network activity.
|-
| 1 || STRT || Start flag enabling the PCnet-PCI II controller to send and receive frames and perform buffer management operations.
|-
| 0 || INIT || Initialization flag enabling the PCnet-PCI II controller to begin the initialization procedure.
|}

=== CSR1 and CSR2: "Initialization Block Addresses" ===

As mentioned in [[#Card registers setup|Card registers setup]] these registers are used to provide the physical address of the initialization struct.
{| class="wikitable"
|+ CSR1
|-
! Bit !! Name !! Description
|-
| 31–16 || RES || Reserved.
|-
| 15–0 || IADR[15:0] || Lower 16 bits of the initialization block address. Read/Write accessible when STOP or SPND bit is set.
|}

{| class="wikitable"
|+ CSR2
|-
! Bit !! Name !! Description
|-
| 31–16 || RES || Reserved.
|-
| 15–0 || IADR[15:0] || Upper 16 bits of the initialization block address. Read/Write accessible when STOP or SPND bit is set.
|}

=== CSR3: "Interrupt Masks and Deferral Control" ===

This register is used to control the various masks of the interrupts the card generates. 

{| class="wikitable"
|-
! Bit !! Name !! Description
|-
| 15 || RES || Reserved. Read and written as ZERO.
|-
| 14 || BABLM || Babble Mask. If BABLM is set, the BABL bit will be masked and unable to set the INTR bit.
|-
| 13 || RES || Reserved. Read and written as ZERO.
|-
| 12 || MISSM || Missed Frame Mask. If MISSM is set, the MISS bit will be masked and unable to set the INTR bit.
|-
| 11 || MERRM || Memory Error Mask. If MERRM is set, the MERR bit will be masked and unable to set the INTR bit.
|-
| 10 || RINTM || Receive Interrupt Mask. If RINTM is set, the RINT bit will be masked and unable to set the INTR bit.
|-
| 9 || TINTM || Transmit Interrupt Mask. If TINTM is set, the TINT bit will be masked and unable to set the INTR bit.
|-
| 8 || IDONM || Initialization Done Mask. If IDONM is set, the IDON bit will be masked and unable to set the INTR bit.
|-
| 7 || RES || Reserved. Read and written as ZERO.
|-
| 6 || DXSUFLO || Disable Transmit Stop on Underflow error.
|-
| 5 || LAPPEN || Look Ahead Packet Processing Enable.
|-
| 4 || DXMT2PD || Disable Transmit Two Part Deferral.
|-
| 3 || EMBA || Enable Modified Back-off Algorithm.
|-
| 2 || BSWP || Byte Swap (Endianness control, 1 for big-endian, 0 for little endian).
|-
| 1 || RES || Reserved. The default value of this bit is a ZERO.
|-
| 0 || RES || Reserved. The default value of this bit is a ZERO.
|}

=== CSR58: "Software Style" ===

As mentioned in [[#SWSTYLE|SWSTYLE]] allows the driver to set various behaviors of the card. In the above example we set bit 2 and clear the other bits 0-7 to set SWSTYLE=2, ie. use 32-bit structures. 

{| class="wikitable"
|-
! Bit !! Name !! Description
|-
| 31–16 || RES || Reserved. Written as ZEROs and read as undefined.
|-
| 15–11 || RES || Reserved. Written as ZEROs and read as undefined.
|-
| 10 || APERREN || Advanced Parity Error Handling Enable. When set, indicates that the BPE bits (RMD1 and TMD1, bit 23) are used to indicate parity errors in data transfers to the receive and transmit buffers.
|-
| 9 || CSRPCNET || CSR PCnet-ISA Configuration. When set, indicates that the PCnet-PCI II controller register bits of CSR4 and CSR3 will map directly to the CSR4 and CSR3 bits of the PCnet-ISA (Am79C960) device.
|-
| 8 || SSIZE32 || 32-Bit Software Size. When set, indicates that the controller should use 32-bit structures for the initialization block and the transmit and receive descriptor entries, as set above.
|-
| 7–0 || SWSTYLE || Software Style Register. Determines the style of register and memory resources used by the controller.
|}


==See Also==
===External Links===
* [https://pdf1.alldatasheet.com/datasheet-pdf/view/89876/AMD/AM79C970.html Specification]

[[Category:Network Hardware]]
[[de:AMD_PCnet]]
````

<a id="w003178"></a>

## W003178 — User:Pinged

Bron-ID: W003178
Namespace: User (2)
Revisie: 13275 | Datum: 2012-04-29T01:34:36Z
Bron: https://wiki.osdev.org/User:Pinged
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APinged&oldid=13275

````text
Just because I know it is cool to know when people use your source. I used your serial code
from [http://wiki.osdev.org/ARM_Integrator-CP_Bare_Bones Integrator-CP Bare Bones]!
````

<a id="w003179"></a>

## W003179 — User:Pancakes/LazyDriverInterface

Bron-ID: W003179
Namespace: User (2)
Revisie: 13308 | Datum: 2012-05-01T14:37:56Z
Bron: https://wiki.osdev.org/User:Pancakes/LazyDriverInterface
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APancakes%2FLazyDriverInterface&oldid=13308

````text
== Purpose ==
Among the many designs for a loadable format and interface, specifically, for drivers you have many choices. When I developed this design I was looking to do it with as simple of a design as possible.
With out doing it this way you end up having to write a lot of support code for ELF and loading sections into memory and then doing relocations with lots of arcane looking documentation. Do not get me wrong if you can load your driver using an ELF32 format you have just as powerful if not more powerful interface than this ''lazy'' design. The strong points of the ''lazy'' design is less debugging and smaller kernel size. The less debugging does not mean you wont run into some bugs implementing even this design, but it does mean less debugging in the case of loading an ELF32 image. It also helps in having a simpler kernel as you will see the support code kernel size is trivial. Your doing nothing more than adding the loaded base address to some offsets in the simplest form.

Also, it should work across all main stream architectures including ARM and IA32/64. The only architectures that it may not be able to work with are ones that separate code and data. But, you will likely find such architectures only in very small processors which need not any form of an operating system because they might only have 32kb of actual memory, or less.

== Requirements ==
{| class="wikitable"
! colspan="2" | Requirements
|-
! GCC
| You need ''GCC'', or a compiler which has equivalent features.
|-
! PIC
| You can generate ''position independant code''. For some targets ''PIC'' may be generated by default or in certain circumstances. For other targets the default may be to generate hard code address references in which you need to force it to generate position independent code.
|-
! Image Base 
| You need a compiler that will let you specify your image base or the base for the ''.text'' section. This is because we will force the compiler to produce an absolute address when we directly reference the function symbol for our interface table, yet still produce ''position independent code''
|-
! Fixup
| Your going to have to do just a tad of fixup in your kernel. You can make a single helper function to do it which will use the ''.term'' field. The helper function would take a generic void* type pointer to the driver's first exported function, and also take an argument of the driver image (or .text) section base (base as in first byte)
|}

<html>
<style>
HLT {
  background-color: #ffe899;
  font-style: italic;
}
</style>
</html>

<html>
<HLT>
<strike>
The biggest problem with this type of a interface is you will not have access to .rodata, or at least during the time of this writing I was unable to get .rodata to be relative accessed. But, the good side is the interface is very simple to implement and will produce super small driver images. So this design might be nice for when you really want a small memory footprint. Because, it can decrease the amount of implementation in the kernel.
</strike>
</HLT>
<HLT>
<strike>
Through arcane back magic from the depths of compiler hell I have arisen a way to access your static
initialized data. The cost was great and my mental abilities were pushed to the limit. So I will have to rewrite the entire page so you can understand how to do it.
</strike>
See towards end for how to get compile time initialized data such as strings and other things.
</HLT>
</html>

So, you got a nice kernel. But, you have decided you want to make your drivers and such for the kernel more modular. In this case your likely wanting to create a separate file then have the kernel load from that. The problem is you might be kind of lazy and you really do not want to write and debug all that code just to patch up the object file format that is holding your driver. It could be an ELF format or a PE32 format. Never the less your like me and you like to be lazy. Well, there is a way to be lazy, and it is not a bad idea. I would not say it is a good one either, but thats only because it has not been proven to be a good way yet. So read forth and encounter a trick from the depths of compiler hell.

== Definition ==
There are two parts. The driver/module interface, and the kernel implementation. The driver interface can look something like this:
<pre>
#define DRV_BOARD_TYPE 		1
struct _DRV_BOARD;
typedef int (*pfDRV_BOARD_INIT)(struct _DRV_BOARD *drv);
typedef void* (*pfDRV_BOARD_GETMEMPAGE)(struct _DRV_BOARD *drv);
typedef void (*pfDRV_BOARD_PUTMEMPAGE)(struct _DRV_BOARD *drv, void *page);
typedef uintptr (*pfDRV_BOARD_GETPAGESIZE)(struct _DRV_BOARD *drv);
typedef int (*pfDRV_BOARD_CANMAP)(struct _DRV_BOARD *drv);
typedef struct _DRV_BOARD {
	struct _DRV_BOARD		*next;
	uint32				sig[2];
	uint32				type;
	DRV_KINTERFACE			ki;
	pfDRV_BOARD_INIT		init;
	pfDRV_BOARD_GETMEMPAGE		getMemPage;
	pfDRV_BOARD_PUTMEMPAGE		putMemPage;
	pfDRV_BOARD_GETPAGESIZE		getPageSize;
	pfDRV_BOARD_CANMAP	        canMap;	
	uintptr				term;
} DRV_BOARD;
</pre>
As you notice we have our five exported functions in which the kernel needs to have access too. Also, take note of the ''term'' field. It helps out automated kernel function to patch up the five ''exported functions''. Also, we have a ''64-bit signature field'' and a ''next''. The ''next'' field is not mandatory and I only use it internally in my kernel to link all the drivers of a single type together. The ''ki'' field is important as it essentially gets updated with the functions the kernel exports for the driver to use.

''The above is shared in a header between both the kernel and the driver source trees.''

== Implementation ==
Let us look at a implementation of this structure.
<pre>
#include "idriver.h"

void* getMemPage(DRV_BOARD *drv);
void putMemPage(DRV_BOARD *drv, void *page);
uintptr getPageSize(DRV_BOARD *drv);
int canMap(DRV_BOARD *drv);
int init(DRV_BOARD *drv);

DRV_BOARD __attribute__((section(".text"))) ModuleHeader = {
	.next =			0,
	.type =			DRV_BOARD_TYPE,
	.sig[0] = 		0x12345678,
	.sig[1] = 		0x87654321,
	.ki = {
		.ksys =			0,
		.kalloc =		0,
		.kfree = 		0,
		.kprintf =		0
	},
	.init =			init,
	.getMemPage =	putMemPage,
	.putMemPage =	putMemPage,
	.getPageSize =	getPageSize,
	.canMap = 		canMap,
	.term =			0
};
</pre>

{| class="wikitable"
! colspan="2" | DRV_BOARD 
|-
| getMemPage || Add base of .text section added (if loaded flat binary image at 0x3000 then use 0x3000
|-
| putMemPage || Add base of ....
|-
| canMap || Add base of....
|-
| term || Always zero (or could use negate 0). Know when we reach end of array of functions.
|}

You should notice I zeroed out the interface the kernel will provide. The fields in ''ki'' will be set by the kernel implementation. While, when the kernel wants to call a function of our interface it will call the fields ''init'', ''getMemPage'', ''putMemPage'', and ''canMap''. But, the kernel will be unable to call those fields as they are. The reason is we have set the ''base address'' for the ''.text'' section at ''0x0''. Meaning, the first byte of the ''.text'' section is expected to start at address '0x0'' in memory. How do you say I do this? Well, I use a linker script but there are likely otherways to do it. So before the kernel can call the interface the driver provides it must iterate over and add a ''base offset'' of where the ''.text'' section starts in memory.

''.getMemPage = .getMemPage + actualOffsetOfTextOnceLoaded''

As you can see ''.getMemPage'' might have a value of ''0x7c'' but the ''.text'' section has actually been loaded to say ''0x12930''. So we have to add ''0x12930'' to ''0x7c'' to get an address the kernel can call for a function.

Also I have told GCC to specifically place this data structure into the ''.text'' section. This is not required and it should work fine no matter where it is inside the image. ''As I will show later there is a case where it needs to be before something (for accessing pre-initialized compile time data)''.

Now, you have seen the definition and implementation, but I need to give you an idea on the kernel implementation. Essentially, the kernel is going to have to search for the magic ''64-bit signature field'' or fields I could say. Once found it is going to need to jump over the fields: ''next'', ''sig[0]'', ''sig[1]'', and all of ''ki''. This will allow it to rest on the first pointer. Then for each pointer (''.getMemPAge'', ''.putMemPage'', ''init'', and ''canMap'') it will need to add the base of the text section to the pointer value. Remember, we might have a value like ''0x7c'' but we loaded where the first byte of the text section is at ''0x12930''. 

You may wonder how do we easily find the first byte of the ''.text'' section. Well, the method I use is to ''objcopy -O binary -j .text <input>'' the linked result of the driver. It drops all other sections. So the first byte of the file is the first byte of the ''.text'' section. Try using that command and experiment and you will see it working and understand it better.

== Kernel Code Example ==
Now, when we get to accessing pre-initialized data I will have to show you a little more. Also, let me show an example implementation of the kernel's search routine and ''fixup'' function.
<pre>
void processDriver(KSYS *ksys, KModule *mod)
{
	int				x;
	int				y;
	uint32			*p;
	int				length;
	uint32			type;
	void			**pp;
	DRV_UNK			*unk;	

	p = (uint32*)mod->data;
	length = mod->length >> 2;
	
	unk = 0;
	
	kprintf("loadDriver.lookingForModuleSignature data:%x length:%x\n", p, length);
	for (x = 0; x < length; ++x)
	{
		
		// fixup driver data
		if ((p[x+0] == 0x11223344) && (p[x+1] == 0x44332211))
		{
			kprintf("found driver data\n");
			for (y = 0; p[x+y] != 0xffffffff; ++y)
				p[x+y] = p[x+y] + (uintptr)p;
			kprintf("set data at %x\n", (uint32)&p[x+0]); 
			unk->ki.data = (void*)&p[x+0];
		}
		
		// fixup driver interface
		if ((p[x+0] == 0x12345678) && (p[x+1] == 0x87654321))
		{
			//kprintf("detecting drv type\n");
			type = p[x+2];
			switch(type)
			{
				case DRV_BOARD_TYPE:
					pp = (void**)&ksys->board;
					break;
				case DRV_FS_TYPE:
					pp = (void**)&ksys->fs;
					break;
				case DRV_INTMAN_TYPE:
					pp = (void**)&ksys->intman;
					break;
				case DRV_PIXEL_TYPE:
					pp = (void**)&ksys->pixel;
					break;
				case DRV_BLOCK_TYPE:
					pp = (void**)&ksys->block;
					break;
				case DRV_SERIAL_TYPE:
					pp = (void**)&ksys->serial;
					break;
			}
		    kprintf("found and processed driver header with type %x\n", type);
			/* go ahead and set the kernel interface */
			unk = (DRV_UNK*)&p[x - 1];
			kprintf("unk->ki.kalloc:%x\n", unk->ki.kalloc);
			unk->ki.ksys = ksys;
			unk->ki.kalloc = kheap_alloc;
			unk->ki.kfree = kheap_free;
			unk->ki.kprintf = kprintf;
			/* fixup addresses with image base */
			fixupModule(unk, (uintptr)p);
			/* just chain them together */
			unk->next = *pp;
			*pp = (void*)unk;
		}
	}
	kprintf("loadDriver.canNotFindSig\n");
	return;
}
</pre>
As you can see I am searching for two different signatures. The ''0x11223344'' I have not talked about yet because it had to do with accessing loading pre-initialized data from the ''.data'' section. But, if you look near the bottom you can see it searching for ''0x12345678''. Once found it links it into the link list for each driver type. Then it fills out the ''ki'' (''kernel interface''), and then calls a special function called ''fixupModule''. The ''fixupModule'' function takes the header and the base of this module. Remember me talking about adding the base to the exported function addresses. Well, thats why I am handing it in a integer form as ''(uintptr)p''. The ''fixupModue'' function is very simple really but just so everybody has a better chance to understand I have included it ''below''.

<pre>
#define DRV_UNK_TYPE		1000
typedef struct _DRV_UNK {
	void			*next;
	uint32			sig[2];
	uint32			type;
	DRV_KINTERFACE	ki;
} DRV_UNK;

void fixupModule(DRV_UNK *unk, uintptr base)
{
	uintptr		*a;
	int			x;
	uintptr		*b;

	a = (uintptr*)((uintptr)unk + sizeof(DRV_UNK));

	kprintf("fixupModule..\n");
	x = 0;
	while (a[x] != 0) 
	{
		if (x > 10)
			break;
		kprintf("fixup:%x:%x:%x\n", a[x], base, a[x] + base);
		a[x] = a[x] + base;
		++x;
	}
}
</pre>

Now, DRV_UNK is simply a generic form of the driver structure. It allows me to jump past all the fields and straight into the export table of the driver/module. I have also included it ''above'' the function so you can better understand.

To access pre-initialized data you need to include your ''.data'' section with your output from ''objcopy'' or put them in the same section using the linker script before hand. I think the linker script can do it all, but I have not had enough luck myself. Never the less now your driver file is a flat binary with a ''.text'' and ''.data'' section. What happens is as you can tell from my kernel code above you look for another signature which is what I call a data table. Let me show the data table. This table can be custom to each driver as it is handed back to the module/driver using the kernel interface.
<pre>
char 	msg1[] = "hello world %x\n";

typedef struct _DATA {
	uint32		sig[2];
	char		*msg1;
	uint32		term;
} DATA;

DATA data = {
	.sig[0] =				0x11223344,
	.sig[1] = 				0x44332211,
	.msg1 = 				&msg1[0],
	.term = 				0xffffffff
};
</pre>
Now, your driver should not be using a lot of data. Therefore this table should not be too hard to manage and if it you might want to use a full ELF format with relocation. But, never the less using this method you can hand the data back in a ''kernel interface'' structure just like you use the ''export table'' for the driver. Here is the definition of my ''ki'' table.
<pre>
typedef struct  _DRV_KINTERFACE {
	struct _KSYS		*ksys;
	pfDRV_KI_ALLOC		kalloc;
	pfDRV_KI_FREE		kfree;
	pfDRV_KI_KPRINTF	kprintf;
	void				*data;
} DRV_KINTERFACE;
</pre>

The ''data'' field is where I hand it back to the driver. If you notice all exported functions of the driver take at least one argument which allows them to access the ''ki'' field and thus the ''data'' field. An, example of a driver using the ''data'' field is below:
<pre>
int init(DRV_BOARD *drv)
{	
	DATA		*d;
	
	d = (DATA*)drv->ki.data;
	
	drv->ki.kprintf(d->msg1, d);
	return 0x30;
}
</pre>

== Graphical Depiction Of Object File Transformation ==


{| class="wikitable"
! Object (Before Linking)
|-
| ELF HEADER
|-
| SECTION HEADERS
|-
| style="background-color: #ff9999" | .TEXT (signature 0x1234567887654321)
|-
| .RODATA
|-
| style="background-color: #ff9999" | .DATA (signature 0x1122334444332211)
|-
| .COMMENT
|}

Above is what we get once we compile and produce an object file. Below is the ''linker script'' that I used to combine the ''.data'' and ''.text'' into a single ''.text'' section. Notice how I based the ''.text'' section to ''0x0''. So all references are based on the first byte of the ''.text'' section being at ''0x0''.
<pre>
ENTRY(init)
phys = 0x0;
SECTIONS
{ 
  .text phys : AT(phys) {
    code = .;
    *(.text)
    *(.data)
    . = ALIGN(4096);
  }
  end = .;
}
</pre>
{| class="wikitable"
! Object (After Linking With Linker Script)
|-
| ELF HEADER
|-
| SECTION HEADERS
|-
| style="background-color: #ff9999" | .TEXT (signature 0x1234567887654321) (signature 0x1122334444332211)
|}

With ''.text'' based at 0x0 and ''.data'' based after the ''.text''. Now, we still have the ''ELF'' headers and section headers at the front of the file. It is still in ''ELF'' format, which is not what we want. So using ''objcopy -O binary -j .text <object>'' we can create the flat binary like below:

{| class="wikitable"
! Driver Format
|-
| style="background-color: #99ff99" | .TEXT (signature 0x1234567887654321) (signature 0x1122334444332211)
|}

It now has no headers and no defined entry point which is only partially true. Our structure can
define a ''init'' function which can be called. Both the driver export function table is included and denoted as signature ''0x1234567887654321'' and our static data as ''0x1122334444332211''. We only
need now to get it into memory with the kernel and then do the fix-ups like is detailed in this page.
````

<a id="w003180"></a>

## W003180 — PearPC

Bron-ID: W003180
Namespace: Main (0)
Revisie: 28048 | Datum: 2023-07-09T19:03:32Z
Bron: https://wiki.osdev.org/PearPC
Revisie-URL: https://wiki.osdev.org/index.php?title=PearPC&oldid=28048

````text
{{Emulators}}
{{stub}}

PearPC is an open source [[PowerPC]] emulator released under the GPL. It can run on a Windows, Linux, or FreeBSD host and is capable of running many PowerPC operating systems, including Mac OS X, Darwin, and Linux.

PearPC includes a just-in-time (JIT) compiler for x86 hosts, resulting in execution being extremely fast.

==Hardware==
* G5 (64-bit PowerPC) CPU emulation
* No sound emulation (however there is a fork called ppc-snd)
* 3Com 3C905C or Realtek 8139 ethernet emulation

==External Links==
* [http://pearpc.sourceforge.net/ PearPC official website]
* [http://en.wikipedia.org/wiki/PearPC PearPC Wikipedia Article]

[[Category:Emulators]]
[[Category:PowerPC]]
````

<a id="w003181"></a>

## W003181 — YAFFS

Bron-ID: W003181
Namespace: Main (0)
Revisie: 29106 | Datum: 2024-06-07T13:40:03Z
Bron: https://wiki.osdev.org/YAFFS
Revisie-URL: https://wiki.osdev.org/index.php?title=YAFFS&oldid=29106

````text
{{Filesystems}}
YAFFS (Yet Another Flash File System) is a file system designed for flash media. There are two versions of YAFFS - YAFFS1 and YAFFS2.

Flash media has the benefit that seek time is virtually non-existent - so data can be as fragmented and scattered as much as it likes without affecting performance. This makes it unsuited for non-flash media that requires seeking. It is purposely designed to not try to reclaim space until the disk is near full - as flash memory has a limited number of re-writes so it becomes advantageous to stretch data out across the disk as much as possible.

YAFFS has the benefit of requiring no formatting. A device simply erased to 0 can be mounted and used as a valid YAFFS file system.

YAFFS writes whole pages (also called chunks) to flash media at a time.

YAAFS implementations are recommended to maintain a tree structure in memory of the physical locations of each chunks for fast look up.

Mounting requires scanning every page in the entire media to construct the internal tree structure. As flash memory has no seek time, this is fast.

==YAFFS2==
YAFFS2 is an extension of YAFFS1. It features the following differences:

* Support for media with large pages. Internal structures in YAFFS2 are not fixed to assume 512 byte sizing.

* Support for write once media. A block sequence number is added to each written page. Older pages can be logically overwritten without physically overwriting them.

==External links==
* [http://www.yaffs.net/ YAFFS official website]
** [http://www.yaffs.net/yaffs-2-specification-and-development-notes YAFFS 2 Specification and Development Notes]
* [http://www.linuxdevices.com/articles/AT9680239525.html Introducing YAFFS, the first NAND-specific flash file system]
* [http://en.wikipedia.org/wiki/YAFFS YAFFS Wikipedia Article]

[[Category:Filesystems]]
````

<a id="w003182"></a>

## W003182 — User:Pancakes/LazyDiskLoading

Bron-ID: W003182
Namespace: User (2)
Revisie: 13314 | Datum: 2012-05-02T06:41:45Z
Bron: https://wiki.osdev.org/User:Pancakes/LazyDiskLoading
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APancakes%2FLazyDiskLoading&oldid=13314

````text
== Purpose ==
There are a lot of file system and disk configurations out there. But, wouldn't it be the lazy way if you could load your kernel using your boot loader from almost any file system that does not use encryption? Imagine, how effortlessly you could write your boot loader for the X86/X86_64 platforms. Just simply copy the kernel onto whatever file system resides on that disk and magically through arcane black magic tricks have it loaded into memory by a simple one-stage boot sector program?

''Well through my magic I will show you how to do it.''

== Technical Details ==
Yes, I am writing this before I actually do it. It is more fun to work under pressure!

Oh HELL I gave up. It seemed like a good idea before I started.
````

<a id="w003183"></a>

## W003183 — User talk:Pancakes/LazyDiskLoading

Bron-ID: W003183
Namespace: User talk (3)
Revisie: 13322 | Datum: 2012-05-03T18:39:37Z
Bron: https://wiki.osdev.org/User_talk:Pancakes/LazyDiskLoading
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3APancakes%2FLazyDiskLoading&oldid=13322

````text
I thing GRUB exists for a reason... :) [[User:Alfaomega08|Alfaomega08]]
: I know, LOL.
````

<a id="w003184"></a>

## W003184 — AML

Bron-ID: W003184
Namespace: Main (0)
Revisie: 30224 | Datum: 2026-03-16T14:52:15Z
Bron: https://wiki.osdev.org/AML
Revisie-URL: https://wiki.osdev.org/index.php?title=AML&oldid=30224

````text
{{stub}}
{{ACPI}}

'''ACPI Machine Language''' (AML) is the platform independent code that ACPI utilizes. A knowledge of it is required to even shutdown the computer. It is found in the [[DSDT]] and [[SSDT]] tables, which are in turn found by parsing the [[RSDT]] or [[XSDT]].

AML code is byte code which is parsed from the beginning of each table when that table is read.  It contains definitions of devices and objects within the ACPI namespace.  By parsing the code, paying attention to all appropriate control-flow statements, an AML interpreter can build up a database of all devices within a system and the properties and functions they support (in reference to configuration and power management).

The specification is available from the [https://uefi.org/specifications UEFI website].  In addition, Intel provides a reference implementation in its [[ACPICA]] software.

==ASL and AML==

ASL is ACPI source language.  It is a more human-readable form of the byte code that is AML.  This difference is similar to that between assembly code and the actual binary machine code.  The Intel ASL assembler (iasl) is freely available on many Linux distributions and can convert in either direction between these formats.

==Sample ASL code==

The following is a very simple example of ASL code for a DSDT.

test.asl:
<syntaxhighlight lang="c">
DefinitionBlock ("test.aml", "DSDT", 1, "OEMID ", "TABLEID  ", 0x00000000)
{
    Scope (_SB)
    {
        Device (PCI0)
        {
            Name (_HID, EisaId ("PNP0A03"))
        }
    }
}
</syntaxhighlight>

This can then be compiled to AML by running 'iasl test.asl' to generate test.aml.

Here is an actual device off an HP pavilion g6 ([[RTC]]). This is embedded in a much larger DefinitionBlock:
<syntaxhighlight lang="c">
// ...
Device (RTC)
{
    Name (_HID, EisaId ("PNP0B00"))  // _HID: Hardware ID
    Name (_CRS, ResourceTemplate ()  // _CRS: Current Resource Settings
    {
        IO (Decode16,
            0x0070,             // Range Minimum
            0x0070,             // Range Maximum
            0x01,               // Alignment
            0x08,               // Length
            )
        IRQNoFlags ()
            {8}
    })
    OperationRegion (CMS0, SystemCMOS, Zero, 0x40)
    Field (CMS0, ByteAcc, NoLock, Preserve)
    {
        RTSE,   8, 
        Offset (0x02), 
        RTMN,   8, 
        Offset (0x04), 
        RTHR,   8, 
        Offset (0x06), 
        RTDY,   8, 
        RTDE,   8
    }
}
// ...
</syntaxhighlight>
Notice that the region is in the [[CMOS]].

Following on from the RTC, here is a [[PS/2 Keyboard]] from the same computer:
<syntaxhighlight lang="c">
// ...
Device (PS2K)
{
    Name (_HID, EisaId ("PNP0303"))  // _HID: Hardware ID
    Method (_STA, 0, NotSerialized)  // _STA: Status
    {
        Return (0x0F)
    }

    Name (_CRS, ResourceTemplate ()  // _CRS: Current Resource Settings
    {
        IO (Decode16,
            0x0060,             // Range Minimum
            0x0060,             // Range Maximum
            0x01,               // Alignment
            0x01,               // Length
            )
        IO (Decode16,
            0x0064,             // Range Minimum
            0x0064,             // Range Maximum
            0x01,               // Alignment
            0x01,               // Length
            )
        IRQ (Edge, ActiveHigh, Exclusive, )
            {1}
    })
    Name (_PRS, ResourceTemplate ()  // _PRS: Possible Resource Settings
    {
        StartDependentFn (0x00, 0x00)
        {
            FixedIO (
                0x0060,             // Address
                0x01,               // Length
                )
            FixedIO (
                0x0064,             // Address
                0x01,               // Length
                )
            IRQNoFlags ()
                {1}
        }
        EndDependentFn ()
    })
    Name (_PRW, Package (0x02)  // _PRW: Power Resources for Wake
    {
        0x18, 
        0x03
    })
    Method (_PSW, 1, NotSerialized)  // _PSW: Power State Wake
    {
        Store (Arg0, KBWK)
    }
}
// ...
</syntaxhighlight>


the above 2 samples were gathered via a utility I wrote to snatch DSDT data from the Registry, disassembled by iASL.--[[User:Bellezzasolo|Bellezzasolo]] 14:58, 20 January 2013 (CST)

This defines one table (the [[DSDT]]), and requests that the output AML be placed in a file called test.aml.  The "OEMID " is a 6 character string defining the name of the OEM who made the firmware of the system, the "TABLEID  " is an 8 character string defining the name of the table - it is normally OEM specific.  The last entry in the <code>DefinitionBlock</code> line is the OEM revision id.  It defines one namespace within the root namespace which is called _SB.  Note that all device/scope/object names are 4 characters long, therefore this is sometimes represented as "_SB ".
_SB is a special name in the [[ACPI]] namespace called "System Bus", which is the main scope under which all devices and bus objects are found.  See the ACPI specification table 5-67 for a list of predefined names.

Within the <code>_SB</code> scope we define a device called <code>PCI0</code>, which has one object within it called <code>_HID</code>.  <code>_HID</code> is again a predefined name that refers to a device's Plug and Play hardware ID, in this case the built-in macro <code>EisaId</code> is used to generate the value 0x030ad041 from "PNP0A03", which is the [[PnP]] ID of a [[PCI]] root bus.  A reasonably complete list of PnP IDs is available in the ''External Links'' section below.

== AML Opcodes ==

This table comes from the ACPI specification, where it is provided mainly for use during debugging an AML parser. For example, if your parser fails and the next byte (that it couldn't parse) is "0x72" then you could refer to this table to see that it is an "Add" operation. Don't let this table confuse you into thinking that AML consists a linear flow of simple instructions to be decoded one at a time. Most AML consists of nested, recursively defined structures and lists.

{| {{wikitable}}
! Value (Hex)
! Name
|-
| 0x00 || ZeroOp
|-
| 0x01 || OneOp
|-
| 0x06 || AliasOp
|-
| 0x08 || NameOp
|-
| 0x0A || BytePrefix
|-
| 0x0B || WordPrefix
|-
| 0x0C || DWordPrefix
|-
| 0x0D || StringPrefix
|-
| 0x0E || QWordPrefix
|-
| 0x10 || ScopeOp
|-
| 0x11 || BufferOp
|-
| 0x12 || PackageOp
|-
| 0x13 || VarPackageOp
|-
| 0x14 || MethodOp
|-
| 0x15 || ExternalOp
|-
| 0x2E (‘.’) || DualNamePrefix
|-
| 0x2F (‘/’) || MultiNamePrefix
|-
| 0x30-0x39 ('0'-'9') || DigitChar
|-
| 0x41-0x5A (‘A’-‘Z’) || NameChar
|-
| 0x5B (‘[’) || ExtOpPrefix
|-
| 0x5B 0x01 || MutexOp
|-
| 0x5B 0x02 || EventOp
|-
| 0x5B 0x12 || CondRefOfOp
|-
| 0x5B 0x13 || CreateFieldOp
|-
| 0x5B 0x1F || LoadTableOp
|-
| 0x5B 0x20 || LoadOp
|-
| 0x5B 0x21 || StallOp
|-
| 0x5B 0x22 || SleepOp
|-
| 0x5B 0x23 || AcquireOp
|-
| 0x5B 0x24 || SignalOp
|-
| 0x5B 0x25 || WaitOp
|-
| 0x5B 0x26 || ResetOp
|-
| 0x5B 0x27 || ReleaseOp
|-
| 0x5B 0x28 || FromBCDOp
|-
| 0x5B 0x29 || ToBCD
|-
| 0x5B 0x2A || UnloadOp
|-
| 0x5B 0x30 || RevisionOp
|-
| 0x5B 0x31 || DebugOp
|-
| 0x5B 0x32 || FatalOp
|-
| 0x5B 0x33 || TimerOp
|-
| 0x5B 0x80 || OpRegionOp
|-
| 0x5B 0x81 || FieldOp
|-
| 0x5B 0x82 || DeviceOpList
|-
| 0x5B 0x83 || ProcessorOp
|-
| 0x5B 0x84 || PowerResOp
|-
| 0x5B 0x85 || ThermalZoneOpList
|-
| 0x5B 0x86 || IndexFieldOp
|-
| 0x5B 0x87 || BankFieldOp
|-
| 0x5B 0x88 || DataRegionOp
|-
| 0x5C (‘\’) || RootChar
|-
| 0x5E (‘^’) || ParentPrefixChar
|-
| 0x5F(‘_’) || NameChar
|-
| 0x60 (‘`’) || Local0Op
|-
| 0x61 (‘a’) || Local1Op
|-
| 0x62 (‘b’) || Local2Op
|-
| 0x63 (‘c’) || Local3Op
|-
| 0x64 (‘d’) || Local4Op
|-
| 0x65 (‘e’) || Local5Op
|-
| 0x66 (‘f’) || Local6Op
|-
| 0x67 (‘g’) || Local7Op
|-
| 0x68 (‘h’) || Arg0Op
|-
| 0x69 (‘i’) || Arg1Op
|-
| 0x6A (‘j’) || Arg2Op
|-
| 0x6B (‘k’) || Arg3Op
|-
| 0x6C (‘l’) || Arg4Op
|-
| 0x6D (‘m’) || Arg5Op
|-
| 0x6E (‘n’) || Arg6Op
|-
| 0x70 || StoreOp
|-
| 0x71 || RefOfOp
|-
| 0x72 || AddOp
|-
| 0x73 || ConcatOp
|-
| 0x74 || SubtractOp
|-
| 0x75 || IncrementOp
|-
| 0x76 || DecrementOp
|-
| 0x77 || MultiplyOp
|-
| 0x78 || DivideOp
|-
| 0x79 || ShiftLeftOp
|-
| 0x7A || ShiftRightOp
|-
| 0x7B || AndOp
|-
| 0x7C || NandOp
|-
| 0x7D || OrOp
|-
| 0x7E || NorOp
|-
| 0x7F || XorOp
|-
| 0x80 || NotOp
|-
| 0x81 || FindSetLeftBitOp
|-
| 0x82 || FindSetRightBitOp
|-
| 0x83 || DerefOfOp
|-
| 0x84 || ConcatResOp
|-
| 0x85 || ModOp
|-
| 0x86 || NotifyOp
|-
| 0x87 || SizeOfOp
|-
| 0x88 || IndexOp
|-
| 0x89 || MatchOp
|-
| 0x8A || CreateDWordFieldOp
|-
| 0x8B || CreateWordFieldOp
|-
| 0x8C || CreateByteFieldOp
|-
| 0x8D || CreateBitFieldOp
|-
| 0x8E || TypeOp
|-
| 0x8F || CreateQWordFieldOp
|-
| 0x90 || LandOp
|-
| 0x91 || LorOp
|-
| 0x92 || LnotOp
|-
| 0x92 0x93 || LNotEqualOp
|-
| 0x92 0x94 || LLessEqualOp
|-
| 0x92 0x95 || LGreaterEqualOp
|-
| 0x93 || LEqualOp
|-
| 0x94 || LGreaterOp
|-
| 0x95 || LLessOp
|-
| 0x96 || ToBufferOp
|-
| 0x97 || ToDecimalStringOp
|-
| 0x98 || ToHexStringOp
|-
| 0x99 || ToIntegerOp
|-
| 0x9C || ToStringOp
|-
| 0x9D || CopyObjectOp
|-
| 0x9E || MidOp
|-
| 0x9F || ContinueOp
|-
| 0xA0 || IfOp
|-
| 0xA1 || ElseOp
|-
| 0xA2 || WhileOp
|-
| 0xA3 || NoopOp
|-
| 0xA4 || ReturnOp
|-
| 0xA5 || BreakOp
|-
| 0xCC || BreakPointOp
|-
| 0xFF || OnesOp
|-
|}

== See Also ==
=== External Links ===
* [https://web.archive.org/web/20160911035725/http://tuxmobil.org/pnp_ids.html PnP Device IDs list]
* [https://cdrdv2-public.intel.com/772722/asl-tutorial-v20190625.pdf ACPI Source Language (ASL) Tutorial]

[[Category:ACPI]]
````

<a id="w003185"></a>

## W003185 — User:Kmcguire/JavaClassLoadAndExecute

Bron-ID: W003185
Namespace: User (2)
Revisie: 13363 | Datum: 2012-05-14T01:04:35Z
Bron: https://wiki.osdev.org/User:Kmcguire/JavaClassLoadAndExecute
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKmcguire%2FJavaClassLoadAndExecute&oldid=13363

````text
This page is about reading and interpreting Java bytecode. The code examples below are old and are by no means complete. However, they may serve some purpose as guidance. I have been working on a VM and have come a long ways from the code below, but also my code has become more complex and for someone who is interested in writting a VM for their OS the code below may prove to be much more useful than my full project.

You can find the project here. It is still simple by my effort but it is much larger. I hope to finish it to a point where it can be useful as a base for usage in my own toy OS. Or, useful in yours. I have made effort so far as to not reply on but the bare minimal of the standard library. Also, one day I will change out my ''malloc'' usage for a more in house solution.  

https://code.google.com/p/rhino-java-virtual-machine/

Here are some resources which will allow you to implement a full virtual machine:
{| class="wikitable"
| http://murrayc.com/learning/java/java_classfileformat.shtml
|-
| http://en.wikipedia.org/wiki/Java_bytecode_instruction_listings
|- 
| http://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.3
|}

Like I said the code below is sort of a guide so you can kind of see what is important
to implement. That will help keep you going by being able to run your test files while
you build it. For instance most of the ''constant pool'' types are not going to be used 
but by looking at just the specification it is hard to tell.

Also, most instructions are not used. So you can pretty much work through it as you go
by adding in some catch when it finds an unknown opcode.

''Below is an example of loading multiple class files and interpreting some basic instructions. Also demonstrated is maintaining type information on the stack and in local variables. No garbage collection is needed for the Python code. In the C example code the infrastructure for garbage collection is there, but no code written to actually perform the collection.
== C Example ==
<source lang="c">
#include <stdio.h>
#include <malloc.h>
#include <string.h>

typedef unsigned long long int  uint64;
typedef signed long long int    int64;
typedef unsigned int            uint32;
typedef unsigned short          uint16;
typedef unsigned char           uint8;
typedef signed int              int32;
typedef signed short            int16;
typedef signed char             int8;

#define JVM_SUCCESS                      1
#define JVM_ERROR_METHODNOTFOUND        -1
#define JVM_ERROR_OUTOFMEMORY           -2
#define JVM_ERROR_UNKNOWNOPCODE         -3
#define JVM_ERROR_CLASSNOTFOUND         -4

#define JVM_STACK_ISOBJECTREF   0x00000001
#define JVM_STACK_ISBYTE        0x00000002
#define JVM_STACK_ISCHAR        0x00000003
#define JVM_STACK_ISDOUBLE      0x00000004
#define JVM_STACK_ISFLOAT       0x00000005
#define JVM_STACK_ISINT         0x00000006
#define JVM_STACK_ISLONG        0x00000007
#define JVM_STACK_ISSHORT       0x00000008
#define JVM_STACK_ISBOOL        0x00000009
#define JVM_STACK_ISARRAY       0x00000010

typedef struct _JVMStack {
  uint64                *data;
  uint32                *flags;
  uint32                pos;
  uint32                max;
} JVMStack;

typedef struct _JVMLocal {
  uint64                data;
  uint32                flags;
} JVMLocal;

void jvm_StackInit(JVMStack *stack, uint32 max) {
  stack->max = max;
  stack->pos = 0;
  stack->data = (uint64*)malloc(sizeof(uint64) * max);
  stack->flags = (uint32*)malloc(sizeof(uint32) * max);
}

int jvm_StackMore(JVMStack *stack) {
  return stack->pos;
}

void jvm_StackPush(JVMStack *stack, uint64 value, uint32 flags) {
  stack->flags[stack->pos] = flags;
  stack->data[stack->pos++] = value;
}

void jvm_StackPop(JVMStack *stack, JVMLocal *local) {
  local->flags = stack->flags[--stack->pos];
  local->data = stack->data[stack->pos];
}

typedef struct _JVMConstPoolItem {
  uint8			type;
} JVMConstPoolItem;

typedef struct _JVMConstPoolMethodRef {
  JVMConstPoolItem	hdr;
  uint32		nameIndex;		// name index
  uint32		descIndex;		// descriptor index
} JVMConstPoolMethodRef;

typedef struct _JVMConstPoolClassInfo {
  JVMConstPoolItem	hdr;
  uint32		nameIndex;
} JVMConstPoolClassInfo;

typedef struct _JVMConstPoolUtf8 {
  JVMConstPoolItem	hdr;
  uint16		size;
  uint8			*string;
} JVMConstPoolUtf8;

typedef struct _JVMConstPoolNameAndType {
  JVMConstPoolItem	hdr;
  uint32		nameIndex;
  uint32		descIndex;
} JVMConstPoolNameAndType;

typedef struct _JVMConstPoolFieldRef {
  JVMConstPoolItem	hdr;
  uint32		classIndex;
  uint32		nameAndTypeIndex;
} JVMConstPoolFieldRef;

typedef struct _JVMClassField {
  uint16		accessFlags;
  uint16		nameIndex;
  uint16		descIndex;
  uint16		attrCount;
} JVMClassField;

typedef struct _JVMAttribute {
  uint16		nameIndex;
  uint32		length;
  uint8			*info;
} JVMAttribute;

typedef struct _JVMMethod {
  uint16		accessFlags;
  uint16		nameIndex;
  uint16		descIndex;
  uint16		attrCount;
  JVMAttribute		*attrs;
} JVMMethod;

typedef struct _JVMClass {
  uint16		poolCnt;
  JVMConstPoolItem	**pool;
  uint16		accessFlags;
  uint16		thisClass;
  uint16		superClass;
  uint16		ifaceCnt;
  uint16		*interfaces;
  uint16		fieldCnt;
  JVMClassField		*fields;
  uint16		methodCnt;
  JVMMethod		*methods;
  uint16		attrCnt;
  JVMAttribute		*attrs;
} JVMClass;

typedef struct _JVMBundleClass {
  struct _JVMBundleClass	*next;
  JVMClass			*jclass;
  const char                    *nameSpace;
} JVMBundleClass;

typedef struct _JVMBundle {
  JVMBundleClass		*first;
} JVMBundle;

typedef struct _JVMMemoryStream {
  uint8			*data;
  uint32		pos;
  uint32		size;
} JVMMemoryStream;

typedef struct _JVMObject {
  struct _JVMObject             *next;
  JVMClass                      *class;
  uint64                        *fields;
  struct _JVMObject             *refs;
  int32                         stackCnt;
  
} JVMObject;

typedef struct _JVM {
  JVMObject             *objects;
} JVM;


// java stores all integers in big-endian
#define LENDIAN
#ifdef LENDIAN
#define noths(x) ((x) >> 8 | ((x) & 0xff) << 8)
#define nothl(x) ((x) >> 24 | ((x) & 0xff0000) >> 8 | ((x) & 0xff00) << 8 | (x) << 24)
#endif
#ifdef BENDIAN
#define noths(x) x
#define nothl(x) x
#endif

void msWrap(JVMMemoryStream *m, void *buf, uint32 size) {
  m->pos = 0;
  m->data = buf;
  m->size = size;
}

uint32 msRead32(JVMMemoryStream *m) {
  uint32		v;
  v = ((uint32*)&m->data[m->pos])[0];
  m->pos += 4;
  return nothl(v);
}
uint16 msRead16(JVMMemoryStream *m) {
  uint16		v;
  v = ((uint16*)&m->data[m->pos])[0];
  m->pos += 2;
  return noths(v);
}

uint8 msRead8(JVMMemoryStream *m) {
  uint8			v;
  v = ((uint8*)&m->data[m->pos])[0];
  m->pos += 1;
  return v;
}

uint8* msRead(JVMMemoryStream *m, uint32 sz, uint8 *buf) {
  uint32	x;
  uint32	p;
  
  p = m->pos;
  for (x = 0; x < sz; ++x)
    buf[x] = m->data[x + p];
  
  m->pos += sz;
  return buf;
}

#define TAG_METHODREF			10
#define TAG_CLASSINFO			7
#define TAG_NAMEANDTYPE			12
#define TAG_UTF8			1
#define TAG_FIELDREF			9

JVMClass* jvm_LoadClass(JVMMemoryStream *m) {
  uint16			vmin;
  uint16			vmaj;
  uint32			magic;
  uint16			cpoolcnt;
  uint8				tag;
  JVMConstPoolItem		**pool;
  JVMConstPoolItem		*pi;
  int				x, y;
  JVMConstPoolMethodRef		*pimr;
  JVMConstPoolClassInfo		*pici;
  JVMConstPoolUtf8		*piu8;
  JVMConstPoolNameAndType	*pint;
  JVMConstPoolFieldRef		*pifr;
  JVMClass			*class;
  
  magic = msRead32(m);
  vmin = msRead16(m);
  vmaj = msRead16(m);
  
  class = (JVMClass*)malloc(sizeof(JVMClass));
  /*
    ==================================
    LOAD CONSTANT POOL TABLE
    ==================================
  */
  cpoolcnt = msRead16(m); 
  pool = (JVMConstPoolItem**)malloc(sizeof(JVMConstPoolItem*) * cpoolcnt);
  class->poolCnt = cpoolcnt;
  class->pool = pool;
  fprintf(stderr, "cpoolcnt:%u", cpoolcnt);
  for (x = 0; x < cpoolcnt - 1; ++x) {
    tag = msRead8(m);
    switch (tag) {
      case TAG_METHODREF:
	pimr = (JVMConstPoolMethodRef*)malloc(sizeof(JVMConstPoolMethodRef));
	pool[x] = (JVMConstPoolItem*)pimr;
	pimr->nameIndex = msRead16(m);
	pimr->descIndex = msRead16(m);
	pimr->hdr.type = TAG_METHODREF;
	fprintf(stderr, "nameIndex:%u descIndex:%u\n", pimr->nameIndex, pimr->descIndex);
	break;
      case TAG_CLASSINFO:
	pici = (JVMConstPoolClassInfo*)malloc(sizeof(JVMConstPoolClassInfo));
	pool[x] = (JVMConstPoolItem*)pici;
	pici->nameIndex = msRead16(m);
	pici->hdr.type = TAG_CLASSINFO;
	break;
      case TAG_UTF8:
	piu8 = (JVMConstPoolUtf8*)malloc(sizeof(JVMConstPoolUtf8));
	pool[x] = (JVMConstPoolItem*)piu8;
	piu8->size = msRead16(m);
	piu8->string = (uint8*)malloc(piu8->size + 1);
	msRead(m, piu8->size, piu8->string);
	piu8->string[piu8->size] = 0;
	piu8->hdr.type = TAG_UTF8;
	fprintf(stderr, "TAG_UTF8: size:%u string:%s\n", piu8->size, piu8->string);
	break;
      case TAG_NAMEANDTYPE:
	pint = (JVMConstPoolNameAndType*)malloc(sizeof(JVMConstPoolNameAndType));
	pool[x] = (JVMConstPoolItem*)pint;
	pint->nameIndex = msRead16(m);
	pint->descIndex = msRead16(m);
	pint->hdr.type = TAG_NAMEANDTYPE;
	fprintf(stderr, "TAG_NAMEANDTYPE: nameIndex:%u descIndex:%u\n", pint->nameIndex,
		pint->descIndex);
	break;
      case TAG_FIELDREF:
	pifr = (JVMConstPoolFieldRef*)malloc(sizeof(JVMConstPoolFieldRef));
	pool[x] = (JVMConstPoolItem*)pifr;
	pifr->classIndex = msRead16(m);
	pifr->nameAndTypeIndex = msRead16(m);
	pifr->hdr.type = TAG_FIELDREF;
	fprintf(stderr, "classIndex:%u nameAndTypeIndex:%u\n", pifr->classIndex, 
		pifr->nameAndTypeIndex);
	break;
      default:
	fprintf(stderr, "unknown tag %u in constant pool\n\n", tag);
	exit(-1);
    }
  }
  /*
    ====================
    LOAD SMALL PARAMETERS
    ====================
  */
  class->accessFlags = msRead16(m);
  class->thisClass = msRead16(m);
  class->superClass = msRead16(m);
  /*
    =====================
    LOAD INTERFACES
    =====================
  */
  class->ifaceCnt = msRead16(m);
  printf("class->ifaceCnt:%u\n", class->ifaceCnt);
  class->interfaces = (uint16*)malloc(class->ifaceCnt);
  for (x = 0; x < class->ifaceCnt; ++x)
    class->interfaces[x] = msRead16(m);
  /*
    ======================
    LOAD FIELDS
    ======================
  */
  class->fieldCnt = msRead16(m);
  class->fields = (JVMClassField*)malloc(sizeof(JVMClassField) * class->fieldCnt);
  for (x = 0; x < class->fieldCnt; ++x) {
    class->fields[x].accessFlags = msRead16(m);
    class->fields[x].nameIndex = msRead16(m);
    class->fields[x].descIndex = msRead16(m);
    class->fields[x].attrCount = msRead16(m);
    fprintf(stderr, "accessFlags:%u nameIndex:%u descIndex:%u attrCount:%u\n",
	    class->fields[x].accessFlags,
	    class->fields[x].nameIndex,
	    class->fields[x].descIndex,
	    class->fields[x].attrCount
    );
  }
  /*
    =======================
    LOAD METHODS
    =======================
  */
  class->methodCnt = msRead16(m);
  class->methods = (JVMMethod*)malloc(sizeof(JVMMethod) * class->methodCnt);
  for (x = 0; x < class->methodCnt; ++x) {
    class->methods[x].accessFlags = msRead16(m);
    class->methods[x].nameIndex = msRead16(m);
    class->methods[x].descIndex = msRead16(m);
    class->methods[x].attrCount = msRead16(m);
    class->methods[x].attrs = (JVMAttribute*)malloc(sizeof(JVMAttribute) * 
      class->methods[x].attrCount);
    for (y = 0; y < class->methods[x].attrCount; ++y) {
      class->methods[x].attrs[y].nameIndex = msRead16(m);
      class->methods[x].attrs[y].length = msRead32(m);
      class->methods[x].attrs[y].info = (uint8*)malloc(
	      class->methods[x].attrs[y].length);
      fprintf(stderr, "name:%s\n", 
	      ((JVMConstPoolUtf8*)class->pool[class->methods[x].attrs[y].nameIndex - 1])->string
	      );
      fprintf(stderr, "attrlen:%u\n", class->methods[x].attrs[y].length);
      msRead(m, class->methods[x].attrs[y].length, class->methods[x].attrs[y].info);
    }
  }
  /*
    ======================
    LOAD ATTRIBUTES
    ======================
  */
  class->attrCnt = msRead16(m);
  class->attrs = (JVMAttribute*)malloc(sizeof(JVMAttribute) * class->attrCnt);
  for (x = 0; x < class->attrCnt; ++x) {
    class->attrs[x].nameIndex = msRead16(m);
    class->attrs[x].length = msRead32(m);
    class->attrs[x].info = (uint8*)malloc(class->attrs[x].length);
    msRead(m, class->attrs[x].length, class->attrs[x].info);
  }
  
  return class;
}

JVMClass* jvm_FindClassInBundle(JVMBundle *bundle, const char *className) {
  JVMBundleClass	        *jbclass;
  JVMConstPoolUtf8              *u;
  uint32                        a;
  JVMConstPoolClassInfo         *b;
  JVMConstPoolUtf8              *c;
  
  for (jbclass = bundle->first; jbclass != 0; jbclass = jbclass->next) {
    
    a = jbclass->jclass->thisClass;
    b = (JVMConstPoolClassInfo*)jbclass->jclass->pool[a - 1];
    c = (JVMConstPoolUtf8*)jbclass->jclass->pool[b->nameIndex - 1];
    printf("looking for class [%s]=?[%s]\n", className, c->string);
    if (strcmp(c->string, className) == 0)
      return jbclass->jclass;
  }
  /*
    This is where you want to do code to search externally to the
    bundle, and if needed load the missing class into memory.
  */
  return 0;
}

JVMMethod* jvm_FindMethodInClass(JVMClass *jclass, const char *methodName, const char *methodType) {
  JVMConstPoolUtf8      *a;
  JVMConstPoolUtf8      *b;
  int                   x;
  
  for (x = 0; x < jclass->methodCnt; ++x) {
    /// get method name
    a = (JVMConstPoolUtf8*)jclass->pool[jclass->methods[x].nameIndex - 1];
    /// get method type
    b = (JVMConstPoolUtf8*)jclass->pool[jclass->methods[x].descIndex - 1];
    fprintf(stderr, "findmeth:%s%s\n", a->string, b->string);
    if (strcmp(a->string, methodName) == 0)
      if (strcmp(b->string, methodType) == 0)
        return &jclass->methods[x];
  }
  fprintf(stderr, "could not find method %s of type %s\n", methodName, methodType);
  return 0;
}

int jvm_IsMethodReturnTypeVoid(const char *typestr) {
  int           x;

  /// go all the way to the return type part
  for (x = 0; typestr[x] != ')'; ++x);
  ++x;
  /// anything other than 'V' is non-void
  if (typestr[x] == 'V')
    return 1;
  return 0;
}

int jvm_GetMethodTypeArgumentCount(const char *typestr) {
  /*
      B byte
      C char
      D double
      F float
      I int
      J long
      L classname;
      S short
      Z boolean
  */
  int           x;
  int           c;
  
  /// chop off first parathesis
  typestr++;
  c = 0;
  /// read each type
  for (x = 0; typestr[x] != ')'; ++x) {
    switch (typestr[x]) {
      case 'L':
        c++;
        /// run until we find semi-colon
        for (++x; typestr[x] != ';' && x < 20; ++x);
        break;
      default:
        c++;
        break;
    }
  }

  return c;
}

static int g_dbg_ec = 0;

int jvm_ExecuteObjectMethod(JVM *jvm, JVMBundle *bundle, JVMClass *jclass, JVMObject *jobject,
                         const char *methodName, const char *methodType,
                         JVMLocal *_locals, uint8 localCnt, JVMLocal *_result) {
  JVMMethod                     *method;
  JVMConstPoolUtf8              *a;
  int                           x, y;
  JVMLocal                      *locals;
  uint32                        codesz;
  uint8                         *code;
  uint8                         opcode;
  JVMStack                      stack;
  JVMLocal                      result;

  JVMClass                      *_jclass;
  JVMObject                     *_jobject;
  JVMMethod                     *_method;
  JVMConstPoolMethodRef         *b;
  JVMConstPoolClassInfo         *c;
  JVMConstPoolNameAndType       *d;
  int                           argcnt;
  uint8                         *mclass;
  uint8                         *mmethod;
  uint8                         *mtype;

  g_dbg_ec++;
  if (g_dbg_ec == 3) {
    printf("g_dbg_ec=%u\n", g_dbg_ec);
    exit(-9);
  }
  
  jvm_StackInit(&stack, 1024);
  
  printf("executing %s\n", methodName);
  
  /// 255 should be maximum local addressable
  locals = (JVMLocal*)malloc(sizeof(JVMLocal) * 256);
  /// copy provided arguments into locals
  for (x = 0; x < localCnt; ++x) {
    locals[x].data = _locals[x].data;
    locals[x].flags = _locals[x].flags;
  }
  /// load object into local_0
  locals[0].data = (uint64)jobject;
  locals[0].flags = JVM_STACK_ISOBJECTREF;
  /// the stack references the objects
  jobject->stackCnt++;
  /// find method specifiee
  method = jvm_FindMethodInClass(jclass, methodName, methodType);
  if (!method) {
    fprintf(stderr, "JVM_ERROR_METHODNOTFOUND");
    return JVM_ERROR_METHODNOTFOUND;
  }
  /// find code attribute
  for (x = 0; x < method->attrCount; ++x) {
    a = (JVMConstPoolUtf8*)jclass->pool[method->attrs[x].nameIndex - 1];
    if (strcmp(a->string, "Code") == 0) {
      code = method->attrs[x].info;
      codesz = method->attrs[x].length;
      break;
    }
  }

  printf("execute code\n");
  /// execute code
  for (x = 0; x < codesz;) {
    opcode = code[x];
    printf("opcode(%u/%u):%x\n", x, codesz, opcode);
    switch (opcode) {
      /// nop: no operation
      case 0:
        x += 2;
        break;
      /// aload
      case 0x19:
        y = code[x+1];
        jvm_StackPush(&stack, locals[y].data, locals[y].flags);
        if (locals[x].flags == JVM_STACK_ISOBJECTREF)
          ((JVMObject*)locals[0].data)->stackCnt++;
        x += 2;
        break;
      /// aload_0: load a reference onto the stack from local variable 0
      case 0x2a:
        jvm_StackPush(&stack, locals[0].data, locals[0].flags);
        if (locals[0].flags == JVM_STACK_ISOBJECTREF)
          ((JVMObject*)locals[0].data)->stackCnt++;
        x += 1;
        break;
      /// aload_1
      case 0x2b:
        jvm_StackPush(&stack, locals[1].data, locals[1].flags);
        if (locals[1].flags == JVM_STACK_ISOBJECTREF)
          ((JVMObject*)locals[1].data)->stackCnt++;
        x += 1;
        break;
      /// aload_2
      case 0x2c:
        jvm_StackPush(&stack, locals[2].data, locals[2].flags);
        if (locals[2].flags == JVM_STACK_ISOBJECTREF)
          ((JVMObject*)locals[2].data)->stackCnt++;
        x += 1;
        break;
      /// aload_3
      case 0x2d:
        jvm_StackPush(&stack, locals[3].data, locals[3].flags);
        if (locals[3].flags == JVM_STACK_ISOBJECTREF)
          ((JVMObject*)locals[3].data)->stackCnt++;
        x += 1;
        break;
      /// invokevirtual
      case 0xb6:
      /// invokespecial
      case 0xb7:
         /*
            (1) verify objref is indeed an object reference
                use the type info on the stack
            (2) verify objref is a reference to the described object
            (3) verify the number of arguments are correct
         */
         y = code[x+1] << 8 | code[x+2];
         
         b = (JVMConstPoolMethodRef*)jclass->pool[y - 1];
         c = (JVMConstPoolClassInfo*)jclass->pool[b->nameIndex - 1];
         a = (JVMConstPoolUtf8*)jclass->pool[c->nameIndex - 1];
         // a->string is className of class we are calling method on
         mclass = a->string;

         /// if java/lang/Object just pretend we did
         if (strcmp(mclass, "java/lang/Object") == 0) {
          printf("caught java/lang/Object call and skipped it\n");
          x +=3 ;
          break;
         }
         
         d = (JVMConstPoolNameAndType*)jclass->pool[b->descIndex - 1];
         a = (JVMConstPoolUtf8*)jclass->pool[d->nameIndex - 1];
         // a->string is the method of the class
         _jclass = jvm_FindClassInBundle(bundle, mclass);
         mmethod = a->string;

         a = (JVMConstPoolUtf8*)jclass->pool[d->descIndex - 1];
         // a->string is the type description of the method
         mtype = a->string;

         _method = jvm_FindMethodInClass(_jclass, mmethod, mtype);
         
         argcnt = jvm_GetMethodTypeArgumentCount(a->string);
         
         printf("invokespecial: %s:%s[%u] in %s\n", mmethod, mtype, argcnt, mclass);

         /// pop locals from stack into local variable array
         _locals = (JVMLocal*)malloc(sizeof(JVMLocal) * (argcnt + 2));
         _locals[0].data = (uint64)jobject;
         _locals[0].flags = JVM_STACK_ISOBJECTREF;
         for (y = 0; y < argcnt; ++y) {
           jvm_StackPop(&stack, &result); 
           _locals[y + 1].data = result.data;
           _locals[y + 1].flags = result.flags;
         }
         /// pop object reference from stack
         jvm_StackPop(&stack, &result);
         if (!(result.flags & JVM_STACK_ISOBJECTREF)) {
           fprintf(stderr, "object from stack is not object reference!");
           exit(-8);
         }
         
         /// stays the same since we poped it then placed it into locals
         //((JVMObject*)result.data)->stackCnt

         _locals[0].data = result.data;
         _locals[1].flags = result.flags;

         printf("########################");
         jvm_ExecuteObjectMethod(jvm, bundle, _jclass, jobject, mmethod, mtype, locals, argcnt + 1, &result);
         free(_locals);

         printf("@@@@@@@@@@@%s\n", mtype);

         /// need to know if it was a void return or other
         if (!jvm_IsMethodReturnTypeVoid(mtype)) {
          /// push result onto stack
          printf("return type not void!\n");
          jvm_StackPush(&stack, result.data, result.flags);
         } else {
           printf("return type void..\n");
         }
         
         x += 3;
         break;
      /// return: void from method
      case 0xb1:
         /// need to go through and decrement reference count of objects on stack and local
         while (jvm_StackMore(&stack)) {
           jvm_StackPop(&stack, &result);
           if (result.flags & JVM_STACK_ISOBJECTREF) {
             fprintf(stderr, "function return void decrement stack object ref %u\n", ((JVMObject*)result.data)->stackCnt);
             ((JVMObject*)result.data)->stackCnt--;
           }
         }
         /// also do local variables the same, decrement obj references
         for (y = 0; y < 256; ++y) {
           if (locals[y].flags & JVM_STACK_ISOBJECTREF) {
              fprintf(stderr, "function return void decrement local object ref %u\n", ((JVMObject*)result.data)->stackCnt);
              ((JVMObject*)result.data)->stackCnt--;
           }
         }
         return JVM_SUCCESS;
      default:
        fprintf(stderr, "unknown opcode %x\n", opcode);
        return JVM_ERROR_UNKNOWNOPCODE;
    }
  }
  
  return JVM_SUCCESS;
}

int jvm_CreateObject(JVM *jvm, JVMBundle *bundle, const char *className, JVMObject **out) {
  JVMClass                      *jclass;
  JVMObject                     *jobject;
  JVMLocal                      result;
  JVMLocal                      locals[1];
  
  *out = 0;
  /// find class and create instance
  printf("create object with class %s\n", className);
  jclass = jvm_FindClassInBundle(bundle, className);
  if (!jclass)
    return JVM_ERROR_CLASSNOTFOUND;
  jobject = (JVMObject*)malloc(sizeof(JVMObject));
  *out = jobject;
  if (!jobject)
    return JVM_ERROR_OUTOFMEMORY;
  memset(*out, 0, sizeof(JVMObject));
  jobject->class = jclass;
  /// link us into global object chain
  jobject->next = jvm->objects;
  jvm->objects = jobject;
  /// execute init method
  locals[0].data = (uint64)jobject;
  locals[0].flags = JVM_STACK_ISOBJECTREF;
  /// call default constructor (no arguments)
  return jvm_ExecuteObjectMethod(jvm, bundle, jclass, jobject, "<init>", "()V", &locals[0], 1, &result);
}

uint8* jvm_ReadWholeFile(const char *path, uint32 *size) {
  uint8         *buf;
  FILE          *fp;
  
  fp = fopen(path, "rb");
  fseek(fp, 0, 2);
  *size = ftell(fp);
  fseek(fp, 0, 0);
  buf = (uint8*)malloc(*size);
  fread(buf, 1, *size, fp);
  fclose(fp);
  return buf;
}

void jvm_AddClassToBundle(JVMBundle *jbundle, JVMClass *jclass) {
  JVMBundleClass                *jbclass;

  jbclass = (JVMBundleClass*)malloc(sizeof(JVMBundleClass));
  jbclass->jclass = jclass;

  jbclass->next = jbundle->first;
  jbundle->first = jbclass;

  return;
}

int main(int argc, char *argv[])
{
  uint8			*buf;
  JVMMemoryStream	m;
  JVMClass		*jclass;
  JVMBundle		jbundle;
  JVMBundleClass	*jbclass;
  JVM                   jvm;
  JVMObject             *jobject;
  uint32                size;
  int                   result;

  buf = jvm_ReadWholeFile("Apple.class", &size);
  msWrap(&m, buf, size);
  jclass = jvm_LoadClass(&m);
  jvm_AddClassToBundle(&jbundle, jclass);
  
  buf = jvm_ReadWholeFile("Test.class", &size);
  msWrap(&m, buf, size);
  jclass = jvm_LoadClass(&m);
  jvm_AddClassToBundle(&jbundle, jclass);

  jvm.objects = 0;

  /// create initial object
  result = jvm_CreateObject(&jvm, &jbundle, "Test", &jobject);
  
  return 1;
}
</source>
== Python Example ==
<source lang="python">
#!/usr/bin/python3.1
import os
import sys
import struct
import pprint

def funpack(fd, fmt):
	sz = struct.calcsize(fmt)
	data = fd.read(sz)
	return struct.unpack_from(fmt, data)

def javaReadClass(fd):
	hdr = {}
	magic, vermin, vermaj = funpack(fd, '>IHH')
	print('magic:%x vermin:%x vermaj:%x' % (magic, vermin, vermaj))
	constPoolCnt = funpack(fd, '>H')[0]
	# =========== const pool =========
	constPool = {}
	hdr['vermin'] = vermin
	hdr['vermaj'] = vermaj
	hdr['constPool'] = constPool
	x = 0
	while x < constPoolCnt - 1:
		x = x + 1
		tag = fd.read(1)
		# method ref
		if tag[0] == 10:
			e = {}
			e['name_index'], e['descriptor_index'] = funpack(fd, '>HH')
			e['type'] = 10
			constPool[x] = e
			continue
		# classinfo
		if tag[0] == 7:
			e = {}
			e['name_index'] = funpack(fd, '>H')[0]
			e['type'] = 7
			constPool[x] = e
			continue
		# utf8
		if tag[0] == 1:
			sz = funpack(fd, '>H')[0]
			data = fd.read(sz)
			e = {}
			e['type'] = 1
			e['value'] = data 
			constPool[x] = e
			continue
		# nameandtype
		if tag[0] == 12:
			e = {}
			e['name_index'], e['descriptor_index'] = funpack(fd, '>HH')
			e['type'] = 12
			constPool[x] = e
			continue
		# fieldref
		if tag[0] == 9:
			e = {}
			e['classIndex'], e['nameAndTypeIndex'] = funpack(fd, '>HH')
			e['type'] = 9
			constPool[x] = e
			continue
		raise Exception('unknown tag %s' % tag[0])
	# -------------------------
	classAccessFlags, thisClass, superClass, ifaceCnt = funpack(fd, '>HHHH')
	print('accessFlags:%x' % classAccessFlags)
	cinfo = constPool[thisClass]
	hdr['name'] = constPool[cinfo['name_index']]['value']
	cinfo = constPool[superClass]
	hdr['super'] = constPool[cinfo['name_index']]['value']
	# ========= interfaces =========
	ifaces = {}
	hdr['ifaces'] = ifaces
	x = 0
	while x < ifaceCnt:
		iface = funpack(fd, '>H')
		cinfo = constPool['classinfo'][iface]
		ifaces[x] = cinfo
		x = x + 1
	# -------------------------------
	# ========= fields ==============
	fieldsCnt = funpack(fd, '>H')[0]
	fields = {}
	hdr['fields'] = fields
	x = 0
	while x < fieldsCnt:
		field = {}
		fields[x + 1] = field
		field['accessFlags'], field['name_index'], \
			field['descriptor_index'], field['attrCnt'] = \
				funpack(fd, '>HHHH')
		field['name'] = constPool[field['name_index']]['value']
		x = x + 1
	# -------------------------------
	# ========= methods ========
	methCnt = funpack(fd, '>H')[0]
	methods = {}
	hdr['methods'] = methods
	x = 0
	while x < methCnt:
		method = {}
		methods[x] = method
		method['access_flags'], method['name_index'], \
			method['descriptor_index'], method['attrCnt'] = \
				funpack(fd, '>HHHH')
		attrCnt = method['attrCnt']
		attrs = []
		method['attrs'] = attrs
		y = 0
		# ========== attributes ==========
		while y < attrCnt:
			attr = {}
			attrs.append(attr)
			attr['name_index'], attr['length'] = funpack(fd, '>HI')
			attr['name'] = constPool[attr['name_index']]['value']
			attr['info'] = fd.read(attr['length'])
			y = y + 1
		# --------------------------------
		x = x + 1
	# -------------------------------
	# ========== attributes =========
	attrCnt = funpack(fd, '>H')[0]
	attrs = []
	x = 0
	while x < attrCnt:
		attr = {}
		attrs.append(attr)
		attr['name_index'], attr['length'] = funpack(fd, '>HI')
		attr['name'] = constPool[attr['name_index']]['value']
		attr['info'] = fd.read(attr['length'])
		x = x + 1
	hdr['attrs'] = attrs
	# ======== resolve some data =======
	__methods = {}
	for k in hdr['methods']:
		meth = hdr['methods'][k]
		meth['name'] = hdr['constPool'][meth['name_index']]['value']
		meth['descriptor'] = hdr['constPool'][meth['descriptor_index']]['value']
		__methods[meth['name']] = meth
	hdr['methods'] = __methods
	return hdr

def javaGetClassMethodAndCode(jclass, methName):
	for mk in jclass['methods']:
		method = jclass['methods'][mk]
		if mk == methName:
			# look for code attribute
			for attr in method['attrs']:
				if attr['name'] == b'Code':
					# found code attribute
					return (method, attr['info'])
			
	raise Exception('not found %s::%s' % (jclass['name'], methName))
	return False

def mkhexstr(data):
	h = []
	for d in data:
		h.append('%02x ' % d)
	return ''.join(h)

TYPE_UNK = 0
TYPE_OBJ = 1
TYPE_INT = 2
TYPE_SHORT = 3
TYPE_LONG = 4
TYPE_NULL = 5
TYPE_FLOAT = 6
TYPE_DOUBLE = 7

dbgmap = {
	TYPE_UNK:		'UNKNOWN',
	TYPE_OBJ:		'OBJECT',
	TYPE_INT:		'INT',
	TYPE_SHORT:		'SHORT',
	TYPE_LONG:		'LONG',
	TYPE_NULL:		'NULL',
	TYPE_FLOAT:		'FLOAT',
	TYPE_DOUBLE:	'DOUBLE',
}

class Obj:
	fields = None
	methods = None
	jclass = None

'''
	Variant structure with easier printing support for debugging.
'''
class Var:
	btype = 0
	value = None
	def __repr__(self):
		if self.btype == TYPE_OBJ:
			return '<OBJECT fields=%s>' % self.value.fields
		else:
			return '<%s:%s>' % (dbgmap[self.btype], self.value)

'''
	The stack contains variant like objects. We store
	basic type information on each stack item. 
'''
def var_mk_unk():
	var = Var()
	var.btype = TYPE_UNK
	return var
def var_mk_obj(obj):
	var = Var()
	var.btype = TYPE_OBJ;
	var.value = obj
	return var
def var_mk_float(f):
	var = Var()
	var.btype = TYPE_FLOAT
	var.value = f
	return var
def var_mk_null():
	var = Var()
	var.btype = TYPE_NULL;
	return var
def var_mk_long(l):
	var = Var()
	var.btype = TYPE_LONG
	var.value = l
	return var
def var_mk_int(i):
	var = Var()
	var.btype = TYPE_INT
	var.value = i
	return var

'''
	(1) create object structure
	(2) find specified class in bundle via textual classname
	(3) create fields in object
	(4) create methods in object (**needed??**)
	(5) push object reference to local_0
	(6) execute <init> method of objet
	(7) add object to system instanced objects list
	(8) return the java object
'''
def javaObjectInstance(jsys, jclassname):
	jobj = Obj()
	# look through bundle and find jclassname
	jclass = jsys.bundle[jclassname]
	
	# initialize fields 
	x = 0
	fields = {}
	for field in jclass['fields']:
		fields[x] = var_mk_unk()
	methods = {}
	for method in jclass['methods']:
		methods[method] = jclass['methods'][method]
	jobj.methods = methods
	jobj.fields = fields
	jobj.jclass = jclass
	jobj.name = jclassname
	# need to execute initialization method
	# <init>
	local = {}
	local[0] = var_mk_obj(jobj)
	javaExecuteClassMethod(jsys, jobj, jclass, b'<init>', local)
	
	# make system ref obj created
	jsys.objects.append(jobj)
	return jobj 

'''
	(1) get the code for the method in the javaClass
	(2) execute each opcode
'''
def javaExecuteClassMethod(jsys, jobj, jclass, methName, local):
	method, code = javaGetClassMethodAndCode(jclass, methName)
	print(method)
	
	print('\033[31mjavaExecuteClassMethod(class:%s method:%s\033[37m' % (jobj.name, methName))
	print(mkhexstr(code))
	# execution stack for method
	stack = []

	x = 0
	csz = len(code)
	while x < csz:
		opcode = code[x]
		print('===%02x===' % opcode)
		print(stack)
		# NOP
		if opcode == 0x00:
			x = x + 2
			continue
		# astore
		if opcode == 0x3a:
			ndx = code[x+1]
			local[ndx] = stack.pop(-1)
			x = x + 1
			continue
		# astore_0
		if opcode == 0x4b:
			local[0] = stack.pop(-1)
			x = x + 1
			continue
		# astore_1
		if opcode == 0x4c:
			local[1] = stack.pop(-1)
			x = x + 1
			continue
		# astore_2
		if opcode == 0x4d:
			local[2] = stack.pop(-1)
			x = x + 1
			continue
		# astore_3
		if opcode == 0x4e:
			local[3] = stack.pop(-1)
			x = x + 1
			continue
		# iload
		if opcode == 0x15:
			ndx = code[x+1]
			stack.append(local[ndx])
			x = x + 2
			continue
		# iload_0
		if opcode == 0x1a:
			stack.append(local[0])
			x = x + 1
			continue
		# iload_1
		if opcode == 0x1b:
			stack.append(local[1])
			x = x + 1
			continue
		# iload_2
		if opcode == 0x1c:
			stack.append(local[2])
			x = x + 1
			continue
		# iload_3
		if opcode == 0x1d:
			stack.append(local[3])
			x = x + 1
			continue
		# lstore
		if opcode == 0x37:
			ndx = code[x+1]
			local[ndx] = stack.pop(-1)
			x = x + 2
			continue
		# lstore_0
		if opcode == 0x3f:
			local[0] = stack.pop(-1)
			x = x + 1
			continue
		# lstore_1
		if opcode == 0x40:
			local[1] = stack.pop(-1)
			x = x + 1
			continue
		# lstore_2
		if opcode == 0x41:
			local[2] = stack.pop(-1)
			x = x + 1
			continue
		# lstore_3
		if opcode == 0x42:
			local[3] = stack.pop(-1)
			x = x + 1
			continue
		# aconst_null: push null onto the stack
		if opcode == 0x01:
			stack.append(var_mk_null())
			x = x + 1
			continue
		# lconst_0: push long(0) onto the stack
		if opcode == 0x09:
			stack.append(var_mk_long(0))
			x = x + 1
			continue
		# iconst_5: push long(5) onto the stack
		if opcode == 0x08:
			stack.append(var_mk_long(5))
			x = x + 1
			continue
		# putstatic: set static field to value in the class
		if opcode == 0xb3:
			iby1 = code[x+1]
			iby2 = code[x+2]
			frefndx = iby1 << 8 + iby2
			print('0xb3', frefndx)
			raise Exception('not implemented')
			x = x + 3
			continue
		# iconst_m1
		if opcode == 0x02:
			stack.append(var_mk_int(-1))
			x = x + 1
			continue
		# iconst_0
		if opcode == 0x03:
			stack.append(var_mk_int(0))
			x = x + 1
			continue
		# iconst_1
		if opcode == 0x04:
			stack.append(var_mk_int(1))
			x = x + 1
			continue
		# iconst_2
		if opcode == 0x05:
			stack.append(var_mk_int(2))
			x = x + 1
			continue
		# iconst_3
		if opcode == 0x06:
			stack.append(var_mk_int(3))
			x = x + 1
			continue
		# iconst_4
		if opcode == 0x07:
			stack.append(var_mk_int(4))
			x = x + 1
			continue
		# iconst_5
		if opcode == 0x08:
			stack.append(var_mk_int(5))
			x = x + 1
		# fconst_0
		if opcode == 0x0b:
			stack.append(var_mk_float(0.0))
			x = x + 1
			continue
		# fconst_1
		if opcode == 0x0c:
			stack.append(var_mk_float(1.0))
			x = x + 1
			continue
		# fconst_2
		if opcode == 0x0d:
			stack.append(var_mk_float(2.0))
			x = x + 1
			continue
		# aload_0: load a ref onto the stack from locvar 0
		if opcode == 0x2a:
			stack.append(local[0])
			x = x + 1
			continue
		# aload_0: load a ref onto the stack from locvar 0
		if opcode == 0x2b:
			stack.append(local[1])
			x = x + 1
			continue
		# aload_0: load a ref onto the stack from locvar 0
		if opcode == 0x2c:
			stack.append(local[2])
			x = x + 1
			continue
		# aload_0: load a ref onto the stack from locvar 0
		if opcode == 0x2d:
			stack.append(local[3])
			x = x + 1
			continue
		# putfield: set field to value in object
		if opcode == 0xb5:
			ndx = code[x+1] << 8 | code[x+2]
			#print('stack:%s' % stack)
			value = stack.pop(-1)
			objref = stack.pop(-1)
			print('objref:%s' % objref)
			if objref.btype != TYPE_OBJ:
				print('attempt to set field on non-object')
				return 0
			objref.value.fields[ndx] = value
			print('putfield:%x\n' % ndx)
			x = x + 3
			continue
		# return void from method
		if opcode == 0xb1:
			return var_mk_null()
		# getfield
		if opcode == 0xb4:
			ndx = code[x+1] << 8 | code[x+2]
			objref = stack.pop(-1)
			value = objref.value.fields[ndx]
			stack.append(value)
			x = x + 3
			continue
		# iadd
		if opcode == 0x60:
			value1 = stack.pop(-1)
			value2 = stack.pop(-1)
			value3 = Var()
			value3.btype = TYPE_INT
			value3.value = value1.value + value2.value
			stack.append(value3)
			x = x + 1
			continue
		# return integer from method
		if opcode == 0xac:
			value = stack.pop(-1)
			return value
		# bipush: push a byte onto the stack as
		#         an integer value
		if opcode == 0x10:
			byte = code[x+1]
			var = Var()
			var.btype = TYPE_INT
			var.value = int(byte)
			stack.append(var)
			x = x + 2
			continue
		# idiv: divide two integers
		if opcode == 0x6c:
			value1 = stack.pop(-1)
			value2 = stack.pop(-1)
			value3 = Var()
			value3.btype = TYPE_INT;
			value3.value = int(value2.value / value1.value)
			stack.append(value3)
			x = x + 1
			continue
		# isub
		if opcode == 0x64:
			value1 = stack.pop(-1)
			value2 = stack.pop(-1)
			value3 = Var()
			value3.btype = TYPE_INT;
			value3.value = int(value2.value - value1.value)
			stack.append(value3)
			x = x + 1
			continue
		# imul: multiply two integers
		if opcode == 0x68:
			value1 = stack.pop(-1)
			value2 = stack.pop(-1)
			value3 = Var()
			value3.btype = TYPE_INT;
			value3.value = int(value2.value * value1.value)
			stack.append(value3)
			x = x + 1
			continue
		# invokespecial
		if opcode == 0xb7:
			ndx = code[x+1] << 8 | code[x+2]
			# nameandtype const
			constPool = jclass['constPool']
			m = constPool[ndx]
			name = constPool[m['name_index']]
			desc = constPool[m['descriptor_index']]
			print('..name:%s desc:%s' % (name, desc))
			_name = constPool[name['name_index']]['value']
			name = constPool[desc['name_index']]['value']
			desc = constPool[desc['descriptor_index']]['value']
			
			# just ignore built-in stuff for now until
			# i can try to get it actually created when
			# get a chance
			if _name == b'java/lang/Object':
				print('\033[32mattempted call for %s:%s' % (name, desc))
				x = x + 3
				continue
			
			print('_name:%s name:%s descriptor:%s' % (_name, name, desc))
			
			# handles invokation with arguments
			local = {}
			y = 0
			while stack[-1].btype != TYPE_OBJ:
				local[y] = stack.pop(-1)
				y = y + 1
			objref = stack.pop(-1)
			# reverse arguments
			sz = len(local)
			nlocal = {}
			for k in local:
				nlocal[sz - k] = local[k]
			nlocal[0] = objref

			local = {}
			ret = javaExecuteClassMethod(jsys, objref.value, objref.value.jclass, name, nlocal)
			x = x + 3
			continue
		# dup: duplicate the value on top stack
		if opcode == 0x59:
			stack.append(stack[-1])
			x = x + 1
			continue
		# new
		if opcode == 0xbb:
			ndx = code[x+1] << 8 | code[x+2]
			constPool = jclass['constPool']
			m = constPool[ndx]
			m = constPool[m['name_index']]['value']
			# find apple in our bundle or try loading
			# it from disk in the relative directory
			if m not in jsys.bundle:
				raise Exception('Not In Bundle! [not-implemented]')
			# new java object
			njobj = javaObjectInstance(jsys, m)
			stack.append(var_mk_obj(njobj))
			x = x + 3
			continue
		if opcode == 0xb6:
			#print(stack)
			ndx = code[x+1] << 8 | code[x+2]
			# need to get virtual method and find out
			# how many arguments it needs
			
			# pop from stack until we reach an object type
			local = {}
			y = 0
			while stack[-1].btype != TYPE_OBJ:
				local[y] = stack.pop(-1)
				y = y + 1
			objref = stack.pop(-1)
			# reverse arguments
			sz = len(local)
			nlocal = {}
			for k in local:
				nlocal[sz - k] = local[k]
			nlocal[0] = objref
			
			#print('local:%s' % local)
			#print('nlocal:%s' % nlocal)
			#print('objref:%s' % objref)
			
			constPool = jclass['constPool']
			m = constPool[ndx]
			print('m:%s' % m)
			'''
				If I follow desc out I get the name of the
				method we are calling and its type info.
				
				But, if you follow name you end up with
				the name of the current class.
			'''
			name = constPool[m['name_index']]
			desc = constPool[m['descriptor_index']]
			print('..name:%s desc:%s' % (name, desc))
			_name = constPool[name['name_index']]['value']
			name = constPool[desc['name_index']]['value']
			desc = constPool[desc['descriptor_index']]['value']
			
			print(_name, name, desc)
			
			# objref is the actual instance of an object
			# name identifies the virtual method
			# desc describes the return type and arguments
			ret = javaExecuteClassMethod(jsys, objref.value, objref.value.jclass, name, nlocal)
			print('~~~ ret from call ~~~')
			print('@@', ret)
			stack.append(ret)
			x = x + 3
			continue
			
			
		print('stack:%s' % stack)
		raise Exception('opcode not understood %x' % opcode)

class JavaSystem:
	bundle = {}
	objects = []

jsys = JavaSystem()

fd = open('Apple.class', 'rb')
jclass = javaReadClass(fd)
jsys.bundle[jclass['name']] = jclass
fd.close()

fd = open('Test.class', 'rb')
jclass = javaReadClass(fd)
jsys.bundle[jclass['name']] = jclass
fd.close()

jobj = javaObjectInstance(jsys, b'Test')

local = {}
local[0] = var_mk_obj(jobj)
ret = javaExecuteClassMethod(jsys, jobj, jclass, b'main', local)
print('------ return -------')
pprint.pprint(ret)
</source>
````

<a id="w003186"></a>

## W003186 — I/O APIC

Bron-ID: W003186
Namespace: Main (0)
Revisie: 13361 | Datum: 2012-05-13T21:30:15Z
Bron: https://wiki.osdev.org/I/O_APIC
Revisie-URL: https://wiki.osdev.org/index.php?title=I%2FO_APIC&oldid=13361
Doorverwijzing: IOAPIC

````text
#REDIRECT [[IOAPIC]]
````

<a id="w003187"></a>

## W003187 — CMake Build System

Bron-ID: W003187
Namespace: Main (0)
Revisie: 28241 | Datum: 2023-07-10T23:28:57Z
Bron: https://wiki.osdev.org/CMake_Build_System
Revisie-URL: https://wiki.osdev.org/index.php?title=CMake_Build_System&oldid=28241

````text
{{Rating|1}}

CMake is a cross-platform, multi-environment build system which can remove some of the headache from building your operating system. The aim of this page is to help the reader set up his or her build environment to use CMake.

== About CMake ==

CMake is makefile generator that allows you to provide a description of your build environment, therefore affording the creation of toolchain-specific build files. It is an open-source project that is largely maintained by KitWare, Inc.

=== Who uses CMake? ===

Because CMake was developed by KitWare, it is closely associated with their software. However, momentum for CMake has steadily been increasing, and some fairly high-profile projects have switched over to it, including:

* The K Desktop Environment
* Second Life
* Apache QPid
* ReactOS

== Getting Started ==

=== Design Considerations ===

There are a variety of options at your disposal when you design your CMake project. It is possible to have CMake cater to some of your personal build preferences, and as a result, you should probably ask yourself a few questions:

* Will my build be performed in-source or out-of-source?
* How granular is the project? Will there be a bunch of small components I plan to link together?
* What prerequisites am I considering for my build?
* Will I be generating any code from templates?

The first question is largely one of preference. An ''in-source'' build means that the build output will be placed in the same directory as the project. This might be the root directory of the project or (assuming a heirarchical organization) subdirectories associated with the unit being compiled. However, the generated files mingling with sources can be an annoyance, especially when using version-control systems (CMake does not have a <code>cmake clean</code> command). Some people prefer to send the build output to a separate folder, perhaps named "build", or even multiple directories for debug and release builds. This sort of design is called an ''out-of-source'' build, and can make building for multiple targets easier and more flexible for all but the simplest projects.

Project directory structure is also worth strong consideration. In a quick academic exercise, a flat single-directory layout may be desirable. However, if you plan to go beyond this stage, it may be a good idea to consider leveraging the filesystem to your advantage. Consider the following layout, for instance:

<pre>
src
|
+----+ kernel
|    |
|    +----+ i386
|    |    |
|    |    +---- pc
|    |
|    +----+ arm
|         |
|         +---- rasberrypi
|         +---- beagle
|
+----+ libc
     |
     +---- ...
</pre>

The above might be the natural way of organizing your source code and we can leverage a directory structure like this to our advantage. Consider the following advantages for this directory tree:

* Sub-projects are neatly isolated from one another. Kernel and libc are definitely related, but we might prefer to think of them as separate projects.
* ISA-dependent code can be isolated from common code or other ISA-dependent code.
* Platform-dependent code (which may add further restrictions to ISA-dependent code) is separated from other platforms sharing the same ISA.

The chances are that if you're building an operating system, you won't have many library dependencies. Most of your dependencies will be related to the toolchain required to build the code. For instance, you will probably want a C compiler, and almost certainly an assembler to go with it. You may write your own tools to simplify the process of creating your operating system, and in that case, you may want to ensure support for other programming environments like Python or Perl. Fortunately, this part can be fairly forgiving: adding dependencies such as these is not terribly difficult, and if you change your mind, the change is easy to implement.

=== A Simple CMakeLists.txt ===

One of the nice things about CMake is that it affords you several programming concepts that you are already familiar with. As one would expect, CMake lets you perform build configuration through variables, which are typically defined in user-provided files named CMakeLists.txt. The resulting values of these variables, after cmake processing, can conveniently be found in a file called CMakeCache.txt. CMakeCache.txt is generated when you first run CMake on a CMakeLists.txt file and can be tweaked to provide various special build options for your script. However, the preferred method is to use the <code>SET()</code> macro (and other macros) in a CMakeLists.txt file, which defines a variable name with its first argument and the variable's value using the rest of the argument(s).

Of course, variables aren't really enough, so CMake provides two kinds of procedures: macros and functions. Functions and macros are very subtly different: functions create their own environment so the variables defined in a function are limited to the scope of the callee and below by default. Macros, on the other hand, will place any variables defined within the macro in the parent's environment. The bulk of CMake is implemented using functions and macros; this permits code reuse that is far superior to the targets provided by make. Note that some of these builtin commands will automatically SET variables on our behalf. This is particularly useful when we want to deal with dependency resolution.

A useable CMakeLists.txt can have as little as two lines as code. One of the most important families of macros include <code>ADD_EXECUTABLE()</code> or <code>ADD_LIBRARY()</code>.
Each of these macros take several parameters: the target name as the first parameter and the source file(s) on which the target depends on as the remaining parameter(s).

The <code>CMAKE_MINIMUM_REQUIRED()</code> function, which takes the version of CMake necessary to parse the CMakeList.txt file is also required, because that is how CMake can tell whether or not it meets the requirements to parse your script.

However, such a script has a key disadvantage: every time we add a new source file, we need to edit CMakeLists.txt to add it. Fortunately, CMake comes with a built in file manipulation interface. In this case, we could use the GLOB operator for the <code>FILE()</code> command, which lets us specify a file globbing pattern to collect all of our source files or file paths into one variable. The addition of this extra line of code can make CMake much more useful.

If you have dependencies you need to resolve, CMake can handle that too. The <code>FIND_PACKAGE()</code> command can handle a variety of different types of dependencies, including libraries (like Boost). <code>FIND_PACKAGE()</code> will also attempt to locate a script by the name of <code>Find''<1st param>''.cmake</code> and handle it appropriately. In instances where <code>FIND_PACKAGE()</code> is unable to resolve the package, it sets a special variable called <code>''<1st param>''_NOTFOUND</code>, which you might use to detect optional dependencies. However, if a dependency is absolutely required, then you can simply supply <code>REQUIRED</code> as the second parameter. Failing to find the package will cause CMake to bail out.

You can emit messages back to the console using the <code>MESSAGE()</code> command. Note that this is only run at CMake time. You can use this command for debugging your CMake project. Alternately, if you supply <code>STATUS</code> as the first parameter to this command, it will print out a specialized status message for you.

<source lang="cmake">
# So CMake can tell whether or not it can process this file
CMAKE_MINIMUM_REQUIRED(VERSION 2.8.0)

# Require Perl (for whatever reason)
FIND_PACKAGE(PERL REQUIRED)

MESSAGE(STATUS "Hi!")

# Grab all of the C files in src; store in C_SRCS
FILE(GLOB C_SRCS src/*.c)

# Note how we obtain this variable's value!
ADD_EXECUTABLE(foo ${C_SRCS})
</source>

This is enough for a small project to generate an executable. Creating the associated Makefile and starting the build is simple:

<pre>
$ cd project/
$ mkdir build
$ cd build/
$ cmake ../
$ make
</pre>

Note that if you chance CMakeLists.txt, you will need to run CMake again. In cases like these, CMake may need to update CMakeCache.txt, for instance. Particularly in the case where you use file globbing find your source files, it is imperative that you do this when you add or delete source files; otherwise, CMake will not pick up the changes, and havoc will break loose.

=== Sub-Project Isolation ===

The previous section discussed a sort of "Hello World" implementation of CMakeLists.txt. Unfortunately, operating system development is rarely in the same class as "Hello World", and the chances are that you have advanced beyond intro computer science. If you're reading this, you more than likely have an idea of how you want to structure your project, and that probably means breaking it into manageable pieces. If you followed the advice from [[#Design Considerations|Design Considerations]], then it is very likely that you have thought about this a great deal. The real question is how to put this into practice.

A common approach involves generating custom build scripts for each sub-project you create. Thus, we have a separate CMakeLists.txt for each sub-project we create, and we link them together in the CMakeLists.txt in the project root. If we adhere to our filesystem layout from above, for instance, we'd have 3 CMakeLists.txt files: one in /, one in /src/kernel, and one in /src/libc. We can link them together with the following:

<source lang="cmake">
ADD_SUBDIRECTORY(src/kernel)
ADD_SUBDIRECTORY(src/libc)
</source>

Using the <code>ADD_SUBDIRECTORY()</code> command is analagous to recursively calling make, and in many cases, this is precisely what happens. Other generators may interpret this command differently: for instance, the MSVC generator might decide to turn these directories into multiple projects within a solution. In either case, the child CMakeLists.txt inherits the environment of the parent so variables are propagated. You can use this to your advantage by doing dependency resolution and setting up critical shared variables in the root CMakeLists.txt file.

Alternately, you can leverage the <code>INCLUDE()</code> command to directly insert CMake code into your CMakeLists.txt file at its point of invocation, which can be useful for important small snippets of code into your project. Note that there are some subtle differences between <code>INCLUDE()</code> and <code>ADD_SUBDIRECTORY()</code>:

* You can use <code>INCLUDE()</code> to include any file as CMake code. <code>ADD_SUBDIRECTORY()</code> expects a CMakeLists.txt file in the directory you point it at.
* <code>INCLUDE()</code> operates from the current working directory. <code>ADD_SUBDIRECTORY()</code> will change the current working directory to the supplied path before evaluating.

== Applying CMake to your Operating System ==

=== Building Assembly Code ===

Unless you intend to use somebody else's kernel and write your operating system completely from portable code, it is very likely that you will need an assembler. For the kinds of projects that CMake was designed for, this rarely comes up; as an operating system designer, such support is probably critical to your project. Fortunately, some work has been done to address this issue and in most cases you can get away with not only detecting an assembler, but even specifying the syntax it uses. By using the <code>ENABLE_LANGUAGE()</code> command it is possible to turn on support for assembly:

<source lang="cmake">
# We want an AT&T Syntax Assembler
ENABLE_LANGUAGE(ASM-ATT)

ADD_EXECUTABLE(foo bar.s baz.s)
</source>

=== Setting Appropriate Build Options ===

Depending on your project the stock compiler options may be insufficient for your needs. You may need to supply switches to your toolchain that affect linking or object assembly. CMake provides a number of ways of doing this:

* For C programs, you can use <code>CMAKE_C_FLAGS</code> in the same way you would use <code>$CFLAGS</code> in the context of make. <code>ADD_DEFINITIONS()</code> can also be used, but it is probably inadvisable to do so since a C flag variable exists by default.
* For other languages, (including assembly) use <code>CMAKE_''<lang>''_COMPILE_OBJECT</code>. For instance, if ASM-ATT is enabled, one would modify <code>CMAKE_ASM-ATT_COMPILE_OJBECT</code>.
* Link-time options can be set using <code>SET_TARGET_FLAGS(''target'' PROPERTIES LINK_FLAGS "''flags''")</code>.

=== Build Profile Detection ===

Remember how we made the claim that the directory structure could be leveraged in order to help make our lives easier? You might have gotten some idea from the directory structure of how this might be accomplished, at least on a conceptual level. Consider the example provided by ''src/kernel''. Here, we have a well-defined hierarchy which allows us to narrow down on varying scopes of the actual kernel implementation. We can split our code into three directories, such that:

* <code>/src/kernel/</code> contains the code for the kernel. (platform-indepdendent code might go into this directory).
* <code>/src/kernel/isa/</code> contains code specifically for the instruction set architecture ''isa'' (e.g., i386).
* <code>/src/kernel/isa/platform/</code> contains code for the ''platform'' which is implemented with ''isa''.

Let's consider the ARM branch of our code. While the i386 may see limited usage outside of IBM-compatible PCs, the ARM conversely is found in a number of environments. Many of those environments have their own quirks, such as how the memory bus is physically mapped. Will the kernel binary for a BeagleBone run on a Raspberry Pi? It is possible, but unlikely. Even if it did, what if these two platforms are fundamentally incompatible at some level? We need to take that into consideration. What we need is a macro that does all of the dirty work for us.

We can take two approaches to this problem: we can either make the assumption platforms (and ISAs for that matter) share some things in common, or make the assumption that they don't. In the former approach, we might assume that the following is true of platforms:

* Each platform provides a memory layout via a linker script.
* Each platform provides a CMake file describing its own build flags.
* Each platform provides a number of C or Assembly sources.

And likewise, we might say that each ISA provides:

* A CMake file describing its own build flags.
* A number of C or Assembly sources.

So far, so good. We can approach this by writing a function in order to handle loading the right profile. When you write a function, you delimit the function body between the <code>FUNCTION()</code> and <code>ENDFUNCTION()</code> commands. The first parameter to <code>FUNCTION()</code> shall be the name of the function you are defining, and those remaining are formal parameters to the function. 

However, there is a slight problem: we need a way to report our findings to the caller. Remember that variables defined in functions go out of scope as soon as the function ends. We could use a macro here at the risk of polluting the namespace but it would be much better if we could export variables to the parent scope. Fortunately, the <code>SET()</code> command accepts <code>PARENT_SCOPE</code> as an optional parameter. When this is used, it means that the variable being set should become part of the parent's environment. Let's write a small build profile function now:

<source lang="cmake">
FUNCTION(LOAD_PROFILE ISA PLATFORM)
  # Obtain sources for the ISA
  FILE(GLOB ISA_SRCS "${ISA}/*.c" "${ISA}/*.s")
  FILE(GLOB PLATFORM_SRCS "${ISA}/${PLATFORM}/*.c" "${ISA}/${PLATFORM}/*.s")
  
  # Load flags associated with ISA and Profile
  INCLUDE("${ISA}/flags.cmake")
  INCLUDE("${ISA}/${PLATFORM}/flags.cmake")

  # Now export our output variables
  SET(ISA_SRCS ${ISA_SRCS} PARENT_SCOPE)
  SET(PLATFORM_SRCS ${PLATFORM_SRCS} PARENT_SCOPE)
  SET(PLATFORM_LAYOUT "${ISA}/${PLATFORM}/layout.ld" PARENT_SCOPE)

  # And specific flags
  SET(ISA_C_FLAGS ${ISA_C_FLAGS} PARENT_SCOPE)
  SET(ISA_ASM_FLAGS ${ISA_ASM_FLAGS} PARENT_SCOPE)
  # ...
ENDFUNCTION(LOAD_PROFILE)
</source>

Now, all we have to do is call <code>LOAD_PROFILE()</code> with the provided parameters, and we should be able to set up our build environment in a sane manner:

<source lang="cmake">
FILE(GLOB GENERIC_SRCS "*.c")

# We could also use CMakeCache variables here!
LOAD_PROFILE("arm" "raspberrypi")


# Now set up our environment
ADD_EXECUTABLE(kernel ${PLATFORM_SRCS} ${ISA_SRCS} ${GENERIC_SRCS})

SET(CMAKE_ASM-ATT_COMPILE_OBJECT 
  "<CMAKE_ASM-ATT_COMPILER> ${ISA_ASM_FLAGS} ${PLATFORM_ASM_FLAGS} -o <OBJECT> <SOURCE>")
SET(CMAKE_C_FLAGS "${ISA_C_FLAGS} ${PLATFORM_C_FLAGS}")
SET_TARGET_PROPERTIES(kernel PROPERTIES LINK_FLAGS 
  "-T ${PLATFORM_LAYOUT} -N ${ISA_LINKER_FLAGS} ${PLATFORM_LINKER_FLAGS}")
</source>

Here, we make a reasonable attempt to control the build order, but the truth is, we don't really know exactly what that order should be; it might
be dependent on the platform. For instance, for i386/pc, we might want a multiboot header, which must come within the first 8K of the kernel image. In that case, we must somehow control the ordering. We could have a place <code>FIRST_SRCS()</code> variable present in the platform flags, then use the following loop to extract it from the list:

<source lang="cmake">
FOREACH(I ${FIRST_SRCS})
  # Assume path is relative to src/kernel
  LIST(APPEND TMP_FIRST_SRCS "${CMAKE_CURRENT_LIST_DIR}/${I}"
ENDFOREACH(I)

# Now remove any trace of these files from the other lists
LIST(REMOVE_ITEM ISA_SRCS ${TMP_FIRST_SRCS})
LIST(REMOVE_ITEM PLATFORM_SRCS ${TMP_FIRST_SRCS})

# During exports:
SET(FIRST_SRCS ${TMP_FIRST_SRCS})
</source>

Now, all we have to do is put <code>${FIRST_SRCS}</code> at the head of the list, and we can control the order in which our code is linked.

== See Also ==

=== Articles ===
* [[Makefile]] - One potential target for CMake. The tried and true method of build management.

=== External Links ===
* [http://www.cmake.org CMake Offical Page] - Contains useful links, including how to download and documentation.
* [http://www.cmake.org/Wiki/CMake_Useful_Variables CMake Useful Variables] - Gives names and descriptions of common and useful variables used in CMake.
* [http://www.cmake.org/cmake/help/v3.2 CMake Official Documentation] - Official documentation all about CMake.
* [http://www.cmake.org/Wiki/CMake/Examples CMake Examples] - Examples showing how to perform many common CMake procedures.

[[Category:Build Tools]]
[[Category:Tools]]
````

<a id="w003189"></a>

## W003189 — User talk:MessiahAndrw/Perception Bytecode

Bron-ID: W003189
Namespace: User talk (3)
Revisie: 13438 | Datum: 2012-05-24T06:23:05Z
Bron: https://wiki.osdev.org/User_talk:MessiahAndrw/Perception_Bytecode
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AMessiahAndrw%2FPerception_Bytecode&oldid=13438

````text
: I like it, but I wish you also had your meta-data stuff written up. 
: Such as the format of the file the byte code would be stored in, and
: more explanation about the ''push [type]'', ''push [field]'', and ''push [global]''. -- [[User:Pancakes|Pancakes]] 22:44, 21 May 2012 (CDT)
:: To clarify about the opcodes. Like an example of pushing a field, or example of calling a function.
:: [[User:Pancakes|Pancakes]] 23:43, 21 May 2012 (CDT)
:::This is coming. I'm still cleaning up my own documentation and I'll progressively publish it on here. --[[User:Messiahandrw|MessiahAndrw]] 17:14, 22 May 2012 (CDT)

== Use of user namespace ==

The user namespace of a Wiki is meant to be used for stuff that should eventually be merged into the Wiki proper. IMHO, it should ''not'' be used for the user's projects. For one, it clutters the [[Special:RecentChanges | RecentChanges]] with entries essentially unrelated to the OSDev Wiki's subject. That is not to say that your project is no good or anything; actually I haven't even looked at it from a quality perspective. You might want to consider hosting your project on SourceForge or a similar site; they usually provide free Wikis too. -- [[User:Solar|Solar]] 01:23, 24 May 2012 (CDT)
````

<a id="w003192"></a>

## W003192 — C Library

Bron-ID: W003192
Namespace: Main (0)
Revisie: 30151 | Datum: 2026-03-06T12:08:15Z
Bron: https://wiki.osdev.org/C_Library
Revisie-URL: https://wiki.osdev.org/index.php?title=C_Library&oldid=30151

````text
The C standard library provides string manipulation (string.h), basic I/O (stdio.h), memory allocation (stdlib.h), and other basic functionality to [[C]] programs. The interface is described in the C standard, with further additions described in [[POSIX]] as well as vendor extensions. On Unix platforms, the library is named <tt>libc</tt> and is linked automatically into every executable.

You need a C standard library implementation with the necessary features to run C programs on your operating system. [[C++]] programs can usually use the C standard library as well and the [[C++]] implementation is normally built on top of libc. It is possible to use the C standard interface in a kernel if the library implementation supports this.

== Freestanding and Hosted ==

There are two flavors of the C compilation environment: Hosted, where the standard library is available; and freestanding, where only a few headers are usable that contains only defines and types. The hosted environment is meant for user-space programming while freestanding is meant for kernel programming. The hosted environment is default, but you can switch to the freestanding by passing <tt>-ffreestanding</tt> to your compiler.

The <tt>__STDC_HOSTED__</tt> macro expands to <tt>1</tt> on hosted implementations, or <tt>0</tt> on freestanding ones. The freestanding headers are: <tt>&lt;float.h&gt;</tt>, <tt>&lt;iso646.h&gt;</tt>, <tt>&lt;limits.h&gt;</tt>, <tt>&lt;stdalign.h&gt;</tt>, <tt>&lt;stdarg.h&gt;</tt>, <tt>&lt;stdbool.h&gt;</tt>, <tt>&lt;stddef.h&gt;</tt>, <tt>&lt;stdint.h&gt;</tt>, and <tt>&lt;stdnoreturn.h&gt;</tt>. You should be familiar with these headers as they contain useful declarations you shouldn't do yourself. GCC also comes with additional freestanding headers for [[CPUID]], [[SSE]] and such.

== Implementations ==

There are several open-source C library packages available, and using one may be a viable solution for you. All of them will require some degree of modification to suit your needs.

There is a [http://www.etalabs.net/compare_libcs.html comparison table] of some of these.

=== MyOS libc ===
{{Main|Creating a C Library}}

One of the more obvious solutions is to roll your own. You can get seamless integration with your kernel and the rest of the operating system, without portability layers. You also get the ability to progress with function implementations in step with the hardware functionality you add to your kernel. You can effectively write the standard and append custom interfaces to be available to all apps. It is also the one and only option if you happen to be one of the purists out there. It also serves as quite the learning experience.

On the other hand, writing your own C library takes a lot of time. Using an existing version lets you spend more time working on your own kernel. In addition, there are a great many caveats hidden in the C language specification, and it's exceedingly easy to write a non-compliant implementation that will cause unexpected issues later on when porting third-party software to your OS. Even if your C library is totally compliant, you might still run into issues trying to port third-party software with dependencies on glibc's quirks.

Here are some existing C libraries for you to pursue:

=== [https://www.gnu.org/software/libc/ Glibc] ===
* GPL license
* Should be absolutely complete (even has all the bloat)
* Supports almost every architecture
* Generates large programs
* Is not written with anything other than Linux in mind, making it a hard port.

=== [http://musl.libc.org/ Musl] ===
* MIT license
* No kernel portability layer, uses the Linux system calls directly. You can add your own layer between musl and the kernel to translate Linux system calls into native system calls, which is the method used by [http://midipix.org midipix].
* A full set of math and printf functions
* Support for about 1200 functions
* Many system calls needs to be implemented as it assumes you are a full Linux

=== [https://sourceware.org/newlib/ Newlib] ===
* The license is unrestricted (not GPL or LGPL), but each file likely has a different copyright notice.
* Requires threading, so is more appropriate for a runtime library
* About 400 functions supported

=== [https://rootdirectory.de/doku.php?id=pdclib:start PDCLib] ===
* Creative Commons Zero license (public domain)
* Under active development, and not at full working release 1.0 yet
* Good for linking into kernels
* Support for about 120 functions, currently
* 10 (plus one optional) required syscalls need to be implemented
* No ASM -- should be fully portable

=== [https://www.uclibc.org/ uClibc] ===
* LGPL license

=== [https://www.fefe.de/dietlibc/ Diet Libc] ===
* GPL 2 license
* Optimized for small size
* Many features missing

=== [https://android.googlesource.com/platform/bionic/ Google's Bionic] ===
* BSD license
* No support for locales
* No libthread_db or libm implementation
* Its own smallish implementation of pthreads based on Linux futexes
* Support for x86, ARM and ARM thumb CPU instruction sets and kernel interfaces

=== [https://sortix.org/source/sortix/tree/master/libc Sortix Libc] ===
* ISC license.
* Implements large parts of the C and POSIX standards.
* Subset can be built as the kernel standard library libk.
* Supports over [https://gitlab.com/sortix/sortix/wikis/Ports 70 pieces of third party software].
* The source code is well organized and fairly straightforward.
* Static linking only at this time.
* Part of [https://sortix.org Sortix] and assumes the Sortix system call ABI, which makes it cleaner.
* Some modification is required to support other system call ABIs depending on how similar the ABI is.

=== [http://libc11.org/ Libc11] ===
* Public domain
* Written for the new C11 standards.
* Under development since November 2014. Currently still severely lacking in functionality.

=== [https://github.com/managarm/mlibc mlibc] ===
* MIT license
* Supports C11, POSIX and various Linux and glibc extensions, has locale support
* Ported to a number of architectures (x86_64, aarch64, riscv64, x86, m68k, loongarch64)
* Highly modular (can turn off extensions if not wanted), allows defining your own libc and syscall ABI or reusing an existing one
* Extremely easy to port to your OS, as it only requires a small list of sysdeps for basic operation, while most sysdeps are optional
* Supports a large variety of ports (bash, gcc, wayland, xorg and many more; currently over [https://github.com/managarm/bootstrap-managarm/tree/master/bootstrap.d 400 packages ported on managarm])
* Supports dynamic and static linking

=== [http://pdos.org PDPCLIB] ===
* Public domain
* Under active development
* C90 only, no extensions whatsoever
* 16-bit, 32-bit and 64-bit
* Supports Win32 (x86 and ARM), Win64 (x64) MSDOS, AmigaOS, OS/2, Linux (x86, x64, ARM32, ARM64), MacOS, MVS, CMS, VSE, PDOS/386, z/PDOS, PDOS-generic, UEFI
* Other platforms can probably be added easily, including new platforms
* Not to be confused with PDCLIB (predates and inspired PDCLIB)

== Standards ==

Especially if you want to roll your own C lib, you may want to buy the ISO/IEC 9899 specification to work from.
It is not free. Expect a PDF to cost somewhere around $250 (US) or 250 Swiss Francs, depending on currency conversions.

On the other hand, the INCITS republishes these standards for a lot less: INCITS/ISO/IEC 9899-2011 can be purchased for about $60 (US) from the ANSI web store or from TechStreet.

The older standards (C89/C90, C99) are not commercially available anymore. To find the current standard, go to one of the
following sites and search for document "ISO/IEC 9899".

By the way, you can use latest draft of standard, it's publicly available for free and exactly the same as approved version. This is a path chosen by most open-source software, especially GCC and GLIBC relies on drafts published by ISO/IEC. Latest version of C2011 draft has the name ISO/IEC 9899 N1570 and can be downloaded [http://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf here (PDF)]

=== Specification Stores ===
* [http://webstore.ansi.org From ANSI]
* [http://infostore.saiglobal.com/store From SAI Global]
* [http://www.iso.org/iso/home/store From ISO Store]

=== Specifications ===
* [http://webstore.ansi.org/RecordDetail.aspx?sku=INCITS%2FISO%2FIEC+9899-2012 INCITS/ISO/IEC 9899-2011 at ANSI's eStandards Store]
* [http://www.techstreet.com/products/1852924 INCITS/ISO/IEC 9899-2011 at Thomson Reuters TechStreet]

== Literature ==
* [https://www.amazon.com/dp/0131315099/ The Standard C Library by P.J. Plauger]

[[Category:C]]
[[Category:Standard Libraries]]
````

<a id="w003193"></a>

## W003193 — Talk:Programmable Interval Timer

Bron-ID: W003193
Namespace: Talk (1)
Revisie: 13472 | Datum: 2012-05-30T09:54:52Z
Bron: https://wiki.osdev.org/Talk:Programmable_Interval_Timer
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AProgrammable_Interval_Timer&oldid=13472

````text
== QEMU Bugs? ==
I just removed the following interjection from the page:

 The PIT isn't emulated properly on QEMU/VirtualBox. 
 Use real machines or Bochs to test it (if you delay 
 function works well on QEMU this doesn't mean it will 
 work on a real machine). There is also a piece of bad 
 PIT code in books, so look out!!!

This isn't helpful at all - it doesn't say what specific emulated behaviour is a trait of QEMU, what the code being used is, and what the related fix would have to be. If an issue, it might have better been filed as a bug at QEMU itself, or it might be a problem specific to the poster. In all, we already know that QEMU is lacking in many more corners than the typical differences in physical hardware, but the combination of the PIT and QEMU has never been mentioned on the forums so there's no reason at all to mention this here without knowing the details. - [[User:Combuster|Combuster]] 04:54, 30 May 2012 (CDT)
````

<a id="w003194"></a>

## W003194 — Initrd

Bron-ID: W003194
Namespace: Main (0)
Revisie: 28032 | Datum: 2023-07-09T18:43:01Z
Bron: https://wiki.osdev.org/Initrd
Revisie-URL: https://wiki.osdev.org/index.php?title=Initrd&oldid=28032

````text
{{Filesystems}}

The term '''initrd''' stands for "initial ramdisk". Basically, it is simply a file loaded into memory which the OS uses as a filesystem containing stuff necessary for it to boot.

== Uses of an initrd ==
Usually, initial ramdisks are used by [[Modular Kernel]]s and [[Microkernel]]s, which commonly face a chicken-or-egg problem at boot time: the kernel needs to load its modules from disk, but these modules include the driver that normally allows the kernel to access that disk (although this applies to any other media). An initrd is an elegant solution to this problem: the boot disk driver is stored within the initrd, which is loaded along with the kernel. Then, when the kernel boots, it retrieves the initrd in memory, loads the boot disk driver from it, and uses the freshly loaded driver to load the rest of its modules. This is the solution used by Linux (not the early versions), as well as many hobby OSes.

== Setting up an initrd ==
An initrd is simply a file containing some sort of minimalistic filesystem (if it can be called so). Such a filesystem totally depends on the operating system and its version, but is usually some archive format (compressed or not) such as [[tar]], which has the advantage of being very simple to generate using the ''tar'' command, a simplistic filesystem such as [[SFS]], or just a custom format. The good news is that this custom filesystem can be very simple: in fact, it doesn't need the complications of usual filesystems, such as addition, modification or deletion of files, and the parsing logic can be very simple.

=== Setup in a multiboot-compliant kernel ===
If you are using a [[Multiboot]]-compliant kernel, then most of the job is already performed by your bootloader (e.g. [[GRUB]]). All you have to do is tell the bootloader to load the initrd along with your kernel as a [[Multiboot]] module (for GRUB, this setting is found in <tt>grub.cfg</tt>), and use the [[Multiboot]] structure to find where it has been loaded.

A [[GRUB]] script would look similar to this:
<pre>
menuentry "myos" {
	multiboot /boot/myos.bin
	module  /boot/myos.initrd
}
</pre>

=== Setup in custom-booted kernels ===
If you are using a custom bootloader, then you will have to load the initrd yourself. You can either load it at a fixed address and hardcode it into the kernel, or determine that address dynamically, and pass it to the kernel.

As an alternative, you can use the ''cat'' command to concatenate your kernel image and initrd image, and modify the linker script to setup a constant as the ending address of your kernel, which will match the loading address of your initrd, or just write a simple program to convert your initrd image into a C header containing a byte array containing its bytes, and include that into your kernel.

== Alternatives ==
If you can't (or just don't want to) use an initrd, you can also use a reserved boot partition (which is much the same thing, except you'll need the disk driver before), or just build certain modules directly into the kernel, which might be a bad idea because changing filesystem or disk will require the kernel to be recompiled with different modules, unless you prefer to hard-build several disk drivers into your kernel, which will increase boot time.

The article on [[Modular Kernel]]s shortly describes some other alternative solutions.

== See Also ==

=== Articles ===
* [[tar]] archive format
* [[SFS]] — simple filesystem implementation
* [[BOOTBOOT]] loader that can load tar, cpio, [http://jamesmolloy.co.uk/tutorial_html/8.-The%20VFS%20and%20the%20initrd.html James Molloy]'s and SFS initrds

=== External Links ===
* [http://en.wikipedia.org/wiki/Cpio cpio] archive format
* [http://en.wikipedia.org/wiki/SquashFS SquashFS] — filesystem designed for small disks
* [http://en.wikipedia.org/wiki/Cramfs CramFS] — filesystem designed for compressed ram disks

[[Category:Filesystems]]
[[Category:Booting]]
````

<a id="w003195"></a>

## W003195 — Talk:Tar

Bron-ID: W003195
Namespace: Talk (1)
Revisie: 23338 | Datum: 2019-02-05T13:00:53Z
Bron: https://wiki.osdev.org/Talk:Tar
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ATar&oldid=23338

````text
How is this article related to OS development? --[[User:Love4boobies|Love4boobies]] 16:01, 31 May 2012 (CDT)

:I assume the original author is using it as a ramdisk format... It's probably close to the best choice of format if you want to use a tool that's part of any default Linux install (or if you have any archiver that's more sane than windows explorer). - [[User:Combuster|Combuster]] 17:44, 31 May 2012 (CDT)

::Using "What links here" from the sidebar toolbox is helpful in cases like this. This article is indeed linked from [[Initrd]]; topical, and helpful IMHO. -- [[User:Solar|Solar]] 04:06, 1 June 2012 (CDT)

There is potential cause for confusion here. This page does not detail the format of Unix TAR archives, as readers may intuitively expect. While similar, this page seems, from my perspective, to detail the design of such archive formats. Perhaps rename this article, and create a new one for the Unix TAR standard (currently USTAR). -- [[User:Cwood|CWood]] 20:18, 27 December 2013 (GMT)
* It looks like a decent tutorial for reading a tar format initrd, except obviously conforming to an older tar standard with the 100 character path limit.  I intend to investigate, to add such notes as are necessary, and move it to the tutorials category.  (How on Earth did it get into OS theory?)  [[User:Eekee|Eekee]] 13:55, 28 January 2019 (CST)
** Done. This page '''does''' document the format of Unix TAR archives up to a point, including ustar. It just documents a common subset of the format. [[User:Eekee|Eekee]] 07:00, 5 February 2019 (CST)
````

<a id="w003198"></a>

## W003198 — Memory mapped registers in C/C++

Bron-ID: W003198
Namespace: Main (0)
Revisie: 30402 | Datum: 2026-03-24T22:05:22Z
Bron: https://wiki.osdev.org/Memory_mapped_registers_in_C/C%2B%2B
Revisie-URL: https://wiki.osdev.org/index.php?title=Memory_mapped_registers_in_C%2FC%2B%2B&oldid=30402

````text
==Prelude:==

When accessing memory mapped registers in any language, as long as the compiler being used is an optimizing compiler, it is liable to make assumptions about the values contained in registers, and to possibly re-order reads and writes based on the assumptions it has made to produce code that is more efficient in the use of the CPU's pipeline, and to better enable write buffering and cache use.

However, memory mapped registers are a case where the order of writes to memory is important; along with this, even if the compiler respects the order of the reads and writes from/to a location, it may still make assumptions about the values in a "variable" since it does not know that the variable is actually a memory mapped device's register. The same applies to data retrieved from reads from a port-mapped IO device's registers.

See the following:

==Serialization and memory barriers==

Serialization or memory barriering, also known as memory fencing, etc., refers to the explicit directive to a compiler or CPU that an instruction stream must be executed in the order specified.

When a memory barrier is issued to a compiler, the compiler re-orders all reads and writes with respect to the location where the barrier is issued, such that no writes or reads which appear in source before the barrier will be emitted after it.

When a memory barrier is issued to a CPU, the CPU stops, flushes the pipeline, and waits for all reads and writes in the pipeline to complete before proceeding with any instructions beyond the serializing instruction.

===Compiler memory barriers===

Explicit memory barrier directives to the compiler are compiler specific. Volatile may work as a memory barrier, but it in many cases is an overoptimization. Where possible, avoid the use of volatile as a memory barrier, and prefer to use an explicit memory barrier directive to the compiler.

<syntaxhighlight lang="c">
// Explicit memory barrier for use with GCC.
asm volatile ("": : :"memory");
</syntaxhighlight>

===CPU memory barriers===

CPUs generally either provide explicit memory fencing (serializing) instructions, or ensure that certain behaviours detected in the code stream will force a memory fence operation to take place.

On x86 family CPUs, the issuing of a '''serializing''' instruction will force a memory barrier operation in hardware. An example of a serializing instruction is the CPUID instruction. An example of an operation that forces serialization is a MOV instruction to any control register.

==The C/C++ volatile keyword==

The volatile keyword was, amongst others, meant for use with MMIO-registers, and is used to indicate that a variable may change outside the scope of the current execution stream, or function, without the compiler's knowledge. An example of such a variable might be an actual hardware register, or a shared lock variable, or a shared synchronization memory location which is written to by multiple CPUs, or polled for changes.

Volatile generally denotes that a change may occur without the compiler being able to see the code that causes the change. This indicates that the compiler should not assume anything about the value in such a variable, naturally, and should always write it back to memory immediately, and not hold it in a register, for example.

Note well that this does not mean that the CPU will be forced to write the variable back to memory. It means that the compiler will emit code which will tell the CPU to do so. The CPU may or may not respect this directive pending the [[Caching options]] of the memory in question. For example, if a volatile variable is stored in a location which, to the CPU is "write-back" memory, the CPU may buffer the write and keep it in the cache, and not write it back to physical RAM until nanoseconds or milliseconds later, thus ruining any order preservation, introducing timing errors in the writes to the hardware device.

==Memory caching options==

There are three common memory cache configurations:

* Write-Back caching: This is where the CPU is given the right to lazily write changes back from its cache to memory. It is allowed to do so on the basis of its caching policy.

* Write-through caching: This is where a CPU may use its cache to speed up reads from a cache line which has not been modified. On write however, the CPU must write all changes back to physical store immediately. There is a performance penalty for using write-through caching, relative to the use of write-back caching.

* Uncached memory: This is memory which the CPU has been configured not to use any caching on, whether caching of writes, or caching of reads from variables which have not been written to in the current CPU's scope of knowledge, or that of its cache coherency domain. The performance penalty for using uncached memory is higher than that of using write-through caching.

Hardware registers should be accessed with either Write-through caching or should be set to be uncached.

TODO: Give an example of a set of memory mapped registers using indirect addressing to show how to use all of the above together to ensure proper visibility of accesses to devices from C/C++ code.

==Useful References==
https://www.kernel.org/doc/Documentation/process/volatile-considered-harmful.rst "Volatile Considered harmful"

[[Category:C]]
[[Category:C++]]
````

<a id="w003199"></a>

## W003199 — Talk:Memory mapped registers in C/C++

Bron-ID: W003199
Namespace: Talk (1)
Revisie: 13531 | Datum: 2012-06-12T15:22:01Z
Bron: https://wiki.osdev.org/Talk:Memory_mapped_registers_in_C/C%2B%2B
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMemory_mapped_registers_in_C%2FC%2B%2B&oldid=13531

````text
Article couldn't meaningfully be placed wholesale into [[Volatile]] or [[Memory barriers]], given that it discusses how to use more than these two techniques to achieve a specific effect. I decided to give it its own article. [[User:Gravaera|Gravaera]] 10:22, 12 June 2012 (CDT)
````

<a id="w003200"></a>

## W003200 — File:ISBN 0136006639.jpg

Bron-ID: W003200
Namespace: File (6)
Revisie: 13540 | Datum: 2012-06-13T02:49:43Z
Bron: https://wiki.osdev.org/File:ISBN_0136006639.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0136006639.jpg&oldid=13540

````text
MOS3/e
````

<a id="w003202"></a>

## W003202 — File:ISBN 9780123838728.jpg

Bron-ID: W003202
Namespace: File (6)
Revisie: 13544 | Datum: 2012-06-13T03:54:34Z
Bron: https://wiki.osdev.org/File:ISBN_9780123838728.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_9780123838728.jpg&oldid=13544

````text
Computer Architecture: A Quantitative Approach, 5th ed.
````

<a id="w003203"></a>

## W003203 — File:ISBN 0321486811.jpg

Bron-ID: W003203
Namespace: File (6)
Revisie: 13546 | Datum: 2012-06-13T04:16:21Z
Bron: https://wiki.osdev.org/File:ISBN_0321486811.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0321486811.jpg&oldid=13546

````text
Dragon book
````

<a id="w003204"></a>

## W003204 — File:ISBN 0735648735.jpg

Bron-ID: W003204
Namespace: File (6)
Revisie: 13548 | Datum: 2012-06-13T04:34:08Z
Bron: https://wiki.osdev.org/File:ISBN_0735648735.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0735648735.jpg&oldid=13548

````text
Windows Internals, Part 1
````

<a id="w003205"></a>

## W003205 — File:ISBN 0735665877.jpg

Bron-ID: W003205
Namespace: File (6)
Revisie: 13549 | Datum: 2012-06-13T04:34:44Z
Bron: https://wiki.osdev.org/File:ISBN_0735665877.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0735665877.jpg&oldid=13549

````text
Windows Internals, Part 2
````

<a id="w003206"></a>

## W003206 — Talk:GDT Tutorial

Bron-ID: W003206
Namespace: Talk (1)
Revisie: 29756 | Datum: 2025-11-12T07:29:01Z
Bron: https://wiki.osdev.org/Talk:GDT_Tutorial
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AGDT_Tutorial&oldid=29756

````text
=== Bit shifting error in second code sample ===

<code lang="C">
  // Create the high 32 bit segment
  descriptor  =  limit       & 0x000F0000;
  descriptor |= (flag <<  8) & 0x00F0FF00;
  descriptor |= (base >> 16) & 0xFF0000FF;
</code>

At the last line, the upper 8 bits will always be 0. What is the proper shifting?
: Thanks.  Fixed on the page.  [[User:Jnc100|John]] 14:07, 13 June 2012 (CDT)
````

<a id="w003207"></a>

## W003207 — File:ISBN 1558604979.jpg

Bron-ID: W003207
Namespace: File (6)
Revisie: 13555 | Datum: 2012-06-13T06:25:48Z
Bron: https://wiki.osdev.org/File:ISBN_1558604979.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_1558604979.jpg&oldid=13555

````text
Practical FS Design
````

<a id="w003208"></a>

## W003208 — File:ISBN 0131103628.jpg

Bron-ID: W003208
Namespace: File (6)
Revisie: 13556 | Datum: 2012-06-13T06:28:34Z
Bron: https://wiki.osdev.org/File:ISBN_0131103628.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0131103628.jpg&oldid=13556

````text
K&R
````

<a id="w003209"></a>

## W003209 — File:ISBN 0321751043.jpg

Bron-ID: W003209
Namespace: File (6)
Revisie: 13559 | Datum: 2012-06-13T06:31:57Z
Bron: https://wiki.osdev.org/File:ISBN_0321751043.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0321751043.jpg&oldid=13559

````text
TAOCP
````

<a id="w003210"></a>

## W003210 — File:ISBN 0201853922.jpg

Bron-ID: W003210
Namespace: File (6)
Revisie: 13558 | Datum: 2012-06-13T06:31:19Z
Bron: https://wiki.osdev.org/File:ISBN_0201853922.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0201853922.jpg&oldid=13558

````text
TAOCP fasc1
````

<a id="w003211"></a>

## W003211 — File:ISBN 0132126958.jpg

Bron-ID: W003211
Namespace: File (6)
Revisie: 13561 | Datum: 2012-06-13T06:40:18Z
Bron: https://wiki.osdev.org/File:ISBN_0132126958.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0132126958.jpg&oldid=13561

````text
Computer Networks, 5th ed.
````

<a id="w003212"></a>

## W003212 — File:ISBN 0596006101.jpg

Bron-ID: W003212
Namespace: File (6)
Revisie: 13562 | Datum: 2012-06-13T06:42:18Z
Bron: https://wiki.osdev.org/File:ISBN_0596006101.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0596006101.jpg&oldid=13562

````text
GNU Make
````

<a id="w003213"></a>

## W003213 — File:ISBN 013089592X.jpg

Bron-ID: W003213
Namespace: File (6)
Revisie: 13563 | Datum: 2012-06-13T06:47:48Z
Bron: https://wiki.osdev.org/File:ISBN_013089592X.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_013089592X.jpg&oldid=13563

````text
C:ARM5
````

<a id="w003214"></a>

## W003214 — File:ISBN 0321336313.jpg

Bron-ID: W003214
Namespace: File (6)
Revisie: 13564 | Datum: 2012-06-13T06:49:59Z
Bron: https://wiki.osdev.org/File:ISBN_0321336313.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0321336313.jpg&oldid=13564

````text
TCP/IP Illustrated, vol. 1
````

<a id="w003215"></a>

## W003215 — File:ISBN 020163354X.jpg

Bron-ID: W003215
Namespace: File (6)
Revisie: 13565 | Datum: 2012-06-13T06:50:54Z
Bron: https://wiki.osdev.org/File:ISBN_020163354X.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_020163354X.jpg&oldid=13565

````text
TCP/IP Illustrated, vol. 2
````

<a id="w003216"></a>

## W003216 — File:ISBN 0201634953.jpg

Bron-ID: W003216
Namespace: File (6)
Revisie: 13566 | Datum: 2012-06-13T06:51:20Z
Bron: https://wiki.osdev.org/File:ISBN_0201634953.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0201634953.jpg&oldid=13566

````text
TCP/IP Illustrated, vol. 3
````

<a id="w003217"></a>

## W003217 — File:ISBN 0321563212.jpg

Bron-ID: W003217
Namespace: File (6)
Revisie: 13568 | Datum: 2012-06-13T07:10:17Z
Bron: https://wiki.osdev.org/File:ISBN_0321563212.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0321563212.jpg&oldid=13568

````text
CERT C
````

<a id="w003218"></a>

## W003218 — User talk:G4b

Bron-ID: W003218
Namespace: User talk (3)
Revisie: 13635 | Datum: 2012-06-13T15:34:46Z
Bron: https://wiki.osdev.org/User_talk:G4b
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AG4b&oldid=13635

````text
Use the [[Projects]] page for projects. --[[User:Love4boobies|Love4boobies]] 02:35, 13 June 2012 (CDT)
````

<a id="w003219"></a>

## W003219 — File:ISBN 0136430163.jpg

Bron-ID: W003219
Namespace: File (6)
Revisie: 13574 | Datum: 2012-06-13T09:49:52Z
Bron: https://wiki.osdev.org/File:ISBN_0136430163.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0136430163.jpg&oldid=13574

````text
The Open Book (OSI)
````

<a id="w003220"></a>

## W003220 — File:ISBN 0136832105.jpg

Bron-ID: W003220
Namespace: File (6)
Revisie: 13575 | Datum: 2012-06-13T09:50:58Z
Bron: https://wiki.osdev.org/File:ISBN_0136832105.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0136832105.jpg&oldid=13575

````text
The Little Black Book (OSI)
````

<a id="w003221"></a>

## W003221 — File:ISBN 0139319573.jpg

Bron-ID: W003221
Namespace: File (6)
Revisie: 13576 | Datum: 2012-06-13T09:52:29Z
Bron: https://wiki.osdev.org/File:ISBN_0139319573.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0139319573.jpg&oldid=13576

````text
UNIX Curses Explained
````

<a id="w003222"></a>

## W003222 — File:ISBN 0131411551.jpg

Bron-ID: W003222
Namespace: File (6)
Revisie: 13577 | Datum: 2012-06-13T09:54:33Z
Bron: https://wiki.osdev.org/File:ISBN_0131411551.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0131411551.jpg&oldid=13577

````text
UNIX Network Programming, vol. 1
````

<a id="w003223"></a>

## W003223 — File:ISBN 0130810819.jpg

Bron-ID: W003223
Namespace: File (6)
Revisie: 13578 | Datum: 2012-06-13T09:56:26Z
Bron: https://wiki.osdev.org/File:ISBN_0130810819.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0130810819.jpg&oldid=13578

````text
UNIX Network Programming, vol. 2
````

<a id="w003224"></a>

## W003224 — File:ISBN 013937681X.jpg

Bron-ID: W003224
Namespace: File (6)
Revisie: 13579 | Datum: 2012-06-13T09:59:01Z
Bron: https://wiki.osdev.org/File:ISBN_013937681X.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_013937681X.jpg&oldid=13579

````text
The Unix Programming Environment
````

<a id="w003225"></a>

## W003225 — File:ISBN 059652983X.jpg

Bron-ID: W003225
Namespace: File (6)
Revisie: 13580 | Datum: 2012-06-13T10:24:26Z
Bron: https://wiki.osdev.org/File:ISBN_059652983X.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_059652983X.jpg&oldid=13580

````text
Learning the vi and Vim Editors
````

<a id="w003226"></a>

## W003226 — File:ISBN 0735619670.jpg

Bron-ID: W003226
Namespace: File (6)
Revisie: 13581 | Datum: 2012-06-13T10:34:34Z
Bron: https://wiki.osdev.org/File:ISBN_0735619670.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0735619670.jpg&oldid=13581

````text
Code Complete, 2nd ed.
````

<a id="w003227"></a>

## W003227 — File:ISBN 0201835959.jpg

Bron-ID: W003227
Namespace: File (6)
Revisie: 13582 | Datum: 2012-06-13T10:41:59Z
Bron: https://wiki.osdev.org/File:ISBN_0201835959.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0201835959.jpg&oldid=13582

````text
The Mythical Man-Month
````

<a id="w003228"></a>

## W003228 — File:ISBN 0201550210.jpg

Bron-ID: W003228
Namespace: File (6)
Revisie: 13583 | Datum: 2012-06-13T11:42:49Z
Bron: https://wiki.osdev.org/File:ISBN_0201550210.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0201550210.jpg&oldid=13583

````text
Inside AppleTalk, 2nd ed.
````

<a id="w003229"></a>

## W003229 — User:Lucretia

Bron-ID: W003229
Namespace: User (2)
Revisie: 13590 | Datum: 2012-06-13T12:09:34Z
Bron: https://wiki.osdev.org/User:Lucretia
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALucretia&oldid=13590

````text

````

<a id="w003231"></a>

## W003231 — Ada Bare Bones

Bron-ID: W003231
Namespace: Main (0)
Revisie: 30645 | Datum: 2026-05-27T15:21:22Z
Bron: https://wiki.osdev.org/Ada_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Ada_Bare_Bones&oldid=30645

````text
{{BadPractice}}

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

<syntaxhighlight lang="shell-session">
$ mkdir -p bare_bones/src/pc
$ cd bare_bones
$ mkdir -p rts/boards/i386/adalib
$ mkdir -p rts/boards/i386/adainclude
$ mkdir -p rts/src
$ mkdir -p obj
</syntaxhighlight>

The best way to begin creating a new run-time library is to base it upon the existing source files obtained from the system GNAT installation, modifying them where necessary.

The following code demonstrates copying the required files from the host system's GNAT installation's run-time library into <tt>rts/src</tt> and then creating symbolic links to <tt>rts/boards/${arch}/adainclude</tt>. Where <tt>${arch}</tt> is <tt>i386</tt>, <tt>armv6</tt>, etc. The source location will need to be modified to reflect the location of the system compiler.

<syntaxhighlight lang="bash">
system_compiler_dir="/usr/lib/gcc/x86_64-linux-gnu/8"
rts_dir="${PWD}/rts"

for f in "ada.ads" "a-unccon.ads" "a-uncdea.ads" "gnat.ads" "g-souinf.ads" \
"interfac.ads" "s-atacco.adb" "s-atacco.ads" "s-maccod.ads" "s-stoele.adb" \
"s-stoele.ads"
do
	cp "${system_compiler_dir}/adainclude/$f" "${rts_dir}/src/"
	ln -s "${rts_dir}/src/$f" "${rts_dir}/rts/boards/i386/adainclude/$f"
done
</syntaxhighlight>

==Files==

===gnat.adc===

During compilation the GNAT compiler will search for a file named <tt>gnat.adc</tt> within the current working directory. This file is expected to contain one or more configuration pragmas that will be applied to the current compilation. These pragma directives instruct the compiler what features of the runtime should be restricted. This is of particular importance to kernel development since many of Ada's features are not supported by a freestanding, bare-metal environment. 

These pragma directives can alternatively be placed at the start of the runtime library's <tt>system.ads</tt> file (see below), however convention dictates using the <tt>gnat.adc</tt> file for this purpose.

Additionally, it is possible to apply additional global configuration files by appending the following line to the <tt>Builder</tt> package in a GNAT project file:

<syntaxhighlight lang="ada">
   package Builder is
      --  ...
      for Global_Configuration_Pragmas use "kernel.adc";
   end Builder;
</syntaxhighlight>

It is also possible to instruct the compiler to apply additional files containing configuration pragmas to the current compilation using the switch <tt>-gnatec=path</tt> on the command line. These configuration pragmas are applied in addition to those found in <tt>gnat.adc</tt>, if it is present.
More information on configuration files can be found in the GNAT documentation: [https://gcc.gnu.org/onlinedocs/gnat_ugn/Handling-of-Configuration-Pragmas.html]

The GNAT Reference Manual and the Ada Reference Manual provide information on the various configuration pragmas.
See below for a list of restriction pragmas useful for a bare bones kernel and runtime:

<syntaxhighlight lang="ada">
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
</syntaxhighlight>

'''Note:''' ''Do not use <tt>pragma No_Run_Time</tt>. It is obsolete.''

Below is an explanation of these configuration pragmas:

{| class="wikitable"
! Restriction
! Purpose
|-
| Discard_Names || The Ada compiler automatically generates strings containing the names of enumerated data types, among others. These strings can be used in I/O.

<syntaxhighlight lang="ada">
type Fruit is (Orange, Banana, Apple);
--  Ada defines the following strings, "Orange", "Banana" and "Apple" in an array.

--  These strings can be accessed using the 'Image attribute, as in
Put (Fruit'Image (Orange));
--  Prints "Orange" to the console.
</syntaxhighlight>

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
<syntaxhighlight lang="ada">
   package Binder is
      for Default_Switches ("Ada") use ("-r");
   end Binder;
</syntaxhighlight>

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

<syntaxhighlight lang="ada">
private

   Run_Time_Name : constant String := "Bare Bones Run Time";
</syntaxhighlight>

According to targparm.ads, it must be the first thing after the private keyword. It should also show up in error messages in parentheses, but I've not managed to get it to show up thus far. This is useful as it should show you which RTS you are currently using, just in case you configure your build incorrectly.

===Last chance handler===

The Ada runtime requires the presence of a <tt>Last_Chance_Handler</tt> subprogram. This subprogram is used as a handler for any exceptions that are not explicitly handled within their subprogram. These calls to the <tt>Last_Chance_Handler</tt> procedure in the case of unhandled exceptions will be automatically generated by the compiler.

In order to facilitate this the last chance handler procedure must be defined somewhere within the program. Typically this is defined within the runtime. The last chance handler procedure may have any name, however the compiler will search for a procedure with external linkage with the name <tt>__gnat_last_chance_handler</tt>. 

Create the following files in the <tt>rts/boards/${arch}/adainclude</tt>:

====last_chance_handler.ads====

<syntaxhighlight lang="ada">
with System;

procedure Last_Chance_Handler
  (Source_Location : System.Address; Line : Integer);
pragma Export (C, Last_Chance_Handler, "__gnat_last_chance_handler");
</syntaxhighlight>

====last_chance_handler.adb====

<syntaxhighlight lang="ada">
procedure Last_Chance_Handler
  (Source_Location : System.Address; Line : Integer) is
   pragma Unreferenced (Source_Location, Line);
begin
   --  TODO: Add in board-specific code to dump exception information to serial/screen.
   loop
      null;
   end loop;
end Last_Chance_Handler;
</syntaxhighlight>

The contents of the <tt>Last_Chance_Handler</tt> procedure will need to be tailored to the specific platform of the kernel. Typically this procedure will dump information regarding the exception to output media such as a [[Serial_Ports|serial port]].

===Compiling the runtime===

Create a file called <tt>gnat.gpr</tt> in the root directory with the following contents:

<syntaxhighlight lang="ada">
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
</syntaxhighlight>

Now compile with the following command:

<syntaxhighlight lang="bash">
gprbuild -XBoard=pc -Pgnat.gpr
</syntaxhighlight>

Inside <tt>rts/boards/i386/adainclude/</tt> the RTS sources should be symbolically linked along with the custom <tt>last_chance_hander</tt> and system files. Inside <tt>rts/boards/i386/adalib/</tt> there should be the <tt>libgnat-4.6.a</tt> file as well as <tt>*.ali</tt> matching the source files, which are required by GNAT.

''Note: Please note that the above lib might need it's name changed as some OSes might have built libgnat with a version number different to the one used here. The system libnat version may be <tt>libgnat-4.4.a</tt> and GNAT will expect to find that, so change the name in the GPR file accordingly.''

===startup.s===

This is the main system bootstrapping code. This version is PC specific, so place this in the <tt>src/pc</tt> directory.

''Note: This source file is written using GAS syntax.''

<syntaxhighlight lang="asm">
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
</syntaxhighlight>

===multiboot.ads===

I won't show the source to this package as  I have made it more Ada-like and it's quite large, so I will link to the current versions, [https://github.com/Lucretia/bare_bones/blob/master/src/pc/multiboot.ads multiboot.ads] and [https://github.com/Lucretia/bare_bones/blob/master/src/pc/multiboot.adb multiboot.adb].

===Console===

'''Disclaimer:''' I wrote this package a long time ago and have reformatted it using my current Ada programming style. I have not gone too far into the code, so it may not be the best implementation of an console.

The following 2 files give you access to the VGA console at 80x25 characters. As they are PC specific, they go into the src/pc directory.

====console.ads====

<syntaxhighlight lang="ada">
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
</syntaxhighlight>

====console.adb====

<syntaxhighlight lang="ada">
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
</syntaxhighlight>

===bare_bones.adb===

This is platform independent and therefore goes into the src directory.

<syntaxhighlight lang="ada">
with Console; use Console;

procedure Bare_Bones is
begin
   Clear;

   Put ("Hello, bare bones in Ada.",
        Screen_Width_Range'First,
        Screen_Height_Range'First);
end Bare_Bones;
pragma No_Return (Bare_Bones);
</syntaxhighlight>

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

<syntaxhighlight lang="make">
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
</syntaxhighlight>

===bare_bones.gpr===

Place this file in the root directory.

<syntaxhighlight lang="ada">
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
</syntaxhighlight>

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


<syntaxhighlight lang="shell-session">
$ git clone git://github.com/gonzoua/u-boot-pi.git
$ cd u-boot-pi
$ make rpi_b_config
</syntaxhighlight>

==Testing==

Make sure you have built the RTS above before this next stage otherwise you won't be able to compile the kernel.

<syntaxhighlight lang="shell-session">
$ make qemu
</syntaxhighlight>

On the QEMU window, it should clear the screen, the the cursor won't move so it will be in the middle of the screen, in the top-left corner will be the message "Hello, bare bones in Ada."

==Source access==

I have created a Git repository on [https://github.com/Lucretia/bare_bones GitHub] containing the source above so you don't have to do it by hand if you don't want to.

In fact there have ben a lot of changes since I started this project and it is a better idea to grab the source from GitHub as it will be more up to date. I will leave the documents above so you can see how it's evolved and also how it works, maybe a bit clearer that it is now.

==Next Steps==

A useful next step for further developing the bare bones kernel outlined in this article is implementing capability for using the <tt>'Image</tt> attributes on scalar types. This facilitates the printing of integers in string form, which is extremely useful for kernel debugging. A simple guide on how to accomplish this is detailed [[Ada_Runtime_Library#Image_attributes|here]].

[[Category:Bare bones tutorials]]
[[Category:Ada]]
[[Category:Inadvisable]]
````

<a id="w003232"></a>

## W003232 — File:ISBN 0716782669.jpg

Bron-ID: W003232
Namespace: File (6)
Revisie: 13610 | Datum: 2012-06-13T13:22:04Z
Bron: https://wiki.osdev.org/File:ISBN_0716782669.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0716782669.jpg&oldid=13610

````text
The Language of Machines
````

<a id="w003233"></a>

## W003233 — File:ISBN 0764574817.jpg

Bron-ID: W003233
Namespace: File (6)
Revisie: 13627 | Datum: 2012-06-13T14:09:26Z
Bron: https://wiki.osdev.org/File:ISBN_0764574817.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0764574817.jpg&oldid=13627

````text
Reversing: Secrets of Reverse Engineering
````

<a id="w003234"></a>

## W003234 — File:ISBN 0470068523.jpg

Bron-ID: W003234
Namespace: File (6)
Revisie: 13628 | Datum: 2012-06-13T14:10:35Z
Bron: https://wiki.osdev.org/File:ISBN_0470068523.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0470068523.jpg&oldid=13628

````text
Security Engineering
````

<a id="w003235"></a>

## W003235 — File:ISBN 0321304543.jpg

Bron-ID: W003235
Namespace: File (6)
Revisie: 13629 | Datum: 2012-06-13T14:11:58Z
Bron: https://wiki.osdev.org/File:ISBN_0321304543.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0321304543.jpg&oldid=13629

````text

````

<a id="w003236"></a>

## W003236 — User talk:Combuster

Bron-ID: W003236
Namespace: User talk (3)
Revisie: 13636 | Datum: 2012-06-13T15:44:21Z
Bron: https://wiki.osdev.org/User_talk:Combuster
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ACombuster&oldid=13636

````text
It wasn't directed at a specific user---I already told him on IRC. But there have been others so I thought I'd leave a note. --[[User:Love4boobies|Love4boobies]] 10:44, 13 June 2012 (CDT)
````

<a id="w003238"></a>

## W003238 — File:ISBN 0132856204.jpg

Bron-ID: W003238
Namespace: File (6)
Revisie: 13656 | Datum: 2012-06-13T20:23:00Z
Bron: https://wiki.osdev.org/File:ISBN_0132856204.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0132856204.jpg&oldid=13656

````text
Computer Netowrking: A Top-Down Approach, 6th ed.
````

<a id="w003240"></a>

## W003240 — C Sharp Bare Bones

Bron-ID: W003240
Namespace: Main (0)
Revisie: 30659 | Datum: 2026-05-28T15:36:53Z
Bron: https://wiki.osdev.org/C_Sharp_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=C_Sharp_Bare_Bones&oldid=30659

````text
{{BadPractice}}
{{BeginnersWarning}}
{{Rating|2}}

This tutorial is intended to help you write a simple 'Hello World' OS in [[C Sharp | C#]] which you can then compile to machine code for the IA32 architecture and boot via [[GRUB]].  There are a number of tools for compiling C# to [[CIL]], including Microsoft's csc (distributed with [http://www.microsoft.com/net Microsoft.NET]) and mcs/gmcs/dmcs (distributed with [http://www.mono-project.com Mono]).  In addition there are a number of tools for compiling [[CIL]] to native machine code in an ahead-of-time manner, including Microsoft's ngen, mono (with the --aot option), [http://gocosmos.org Cosmos'] IL2CPU, mosacl from the [http://www.mosa-project.org/projects/mosa MOSA project] and tysila from the [http://www.tysos.org/redmine/projects/tysos/ tysos] project.  Given the author's familiarity with tysos, that is what we will focus on here.

==Prerequisites==

You will need a binutils which can target the elf_i386 emulation, mono (for the gmcs compiler) or csc from .net, grub and its xorriso dependency (for generating iso images), [[NASM]]/[[YASM]]/something similar for the assembly stub and of course tysila.  For debian-based systems try sudo apt-get install nasm xorriso qemu mono-devel.

For tysila, you can download pre-compiled binaries from http://www.tysos.org/files/tysila/tysila-latest.tar.bz2 (http://web.archive.org/web/20170317154703/http://www.tysos.org/files/tysila/tysila-latest.tar.bz2) and extract them to somewhere in your path.  Please note these binaries will not work on 64-bit Windows due to a bug in the current Microsoft CLR (see [http://connect.microsoft.com/VisualStudio/feedback/details/635365/runtimehelpers-initializearray-fails-on-64b-framework here]) so instead you must build it from source.

==Building tysila==

This is only required if you have not downloaded the pre-compiled binaries above.  Use subversion to get the latest sources 'svn co http://www.tysos.org/svn/trunk tysos' (not in Wayback Machine), or download the latest tar ball from http://www.tysos.org/files/src/tysos-latest.tar.bz2 (http://web.archive.org/web/*/http://www.tysos.org/files/src/tysos-latest.tar.bz2).  Tysos is a project developing a full OS kernel and drivers in C#, however we only want the compiler from it therefore we only want to compile part of the build tree.  Enter the tysos directory and run
<syntaxhighlight lang="shell-session">
$ cd tybuild && make && cd ..
$ cd mono/corlib && make mscorlib.dll && cd ../..
$ cd tysila2 && make && cd ..
</syntaxhighlight>

You will need to put the mono/corlib/mscorlib.dll, tysila2/bin/Release/tysila2.exe, tysila2/bin/Release/libsupcs.dll, tysila2/bin/Release/libtysila.dll, tysila2/bin/Release/tydbfile.dll, tysila2/bin/Release/tydisasm.dll and tybuild/bin/Release/tybuild.exe files somewhere in your path.

You may need to enable binfmt_misc support for mono (if trying this on linux).  How to do this is outside the scope of this document but the answer is easily obtained via Google.

==Directory layout==

We will create a directory to build our OS and ISO file in.  Something like mkdir -p barebones/iso/boot/grub should suffice.  Enter the barebones directory and start creating some files.

===loader.asm===

This is the assembly stub which will contain a Multiboot header.

<syntaxhighlight lang="nasm">
global sthrow

extern kmain

MODULEALIGN       equ     1<<0
MEMINFO           equ     1<<1
FLAGS             equ     MODULEALIGN | MEMINFO
MAGIC             equ     0x1BADB002
CHECKSUM          equ     -(MAGIC + FLAGS)

section .text

align 4
dd MAGIC
dd FLAGS
dd CHECKSUM

sthrow:
    hlt
    jmp sthrow
</syntaxhighlight>

===kernel.cs===

This is the actual simple kernel - it just prints a message to the screen.

<syntaxhighlight lang="csharp">
namespace BareBones
{
    class Program
    {
        static int pos = 0;

        unsafe static void Main()
        {
            // Clear the screen
            for(int i = 0; i < 80 * 25 * 2; i++)
                *(byte *)(0xb8000 + i) = 0;

            // Say hi
            Print("Hello World!");
        }

        static void Print(string s)
        {
            foreach(char c in s)
                Print(c);
        }

        unsafe static void Print(char c)
        {
            *(byte *)(0xb8000 + pos) = (byte)c;
            *(byte *)(0xb8000 + pos + 1) = 0x0f;
            pos += 2;
        }
    }
}
</syntaxhighlight>

===linker.ld===

The linker script

<syntaxhighlight lang="c">
ENTRY (_start)

SECTIONS
{
    . = 0x00100000;

    .text ALIGN(0x1000) :
    {
        *(.text)
    }

    .rodata ALIGN(0x1000) :
    {
        *(.rodata*)
    }

    .data ALIGN(0x1000) :
    {
        *(.data)
    }

    .bss ALIGN(0x1000) :
    {
        *(COMMON)
        *(.bss)
    }
}
</syntaxhighlight>

===iso/boot/grub/grub.cfg===
This is a short file to tell grub where to find our kernel
<syntaxhighlight lang="c">
multiboot /kernel.bin
boot
</syntaxhighlight>

==Building it all==
The following commands should build your new C# kernel.  First, assemble the multiboot stub:
<syntaxhighlight lang="shell-session">
$ nasm -felf -o loader.o loader.asm
</syntaxhighlight>

To compile the .cs file to a .exe you have a choice of three options (depending on your architecture):
<syntaxhighlight lang="shell-session">
$ gmcs /target:exe /out:kernel.exe /unsafe kernel.cs
$ csc /target:exe /out:kernel.exe /unsafe kernel.cs
$ tybuild.exe /unsafe kernel.cs
</syntaxhighlight>

To compile kernel.exe to machine code we use tysila:
<syntaxhighlight lang="shell-session">
$ tysila2.exe --arch i586-elf-tysos -fno-rtti -o kernel.o kernel.exe
</syntaxhighlight>

Here, the <tt>-fno-rtti</tt> switch disables run-time type information, support for which would greatly enlarge the size of your kernel and require you to provide a great number of run time functions to support this.

To link:
<syntaxhighlight lang="shell-session">
$ ld -m elf_i386 -T linker.ld -o iso/kernel.bin loader.o kernel.o
</syntaxhighlight>

Then we make a bootable iso image with:
<syntaxhighlight lang="shell-session">
$ grub-mkrescue -o barebones.iso iso
</syntaxhighlight>

And run it on [[QEMU]] with:
<syntaxhighlight lang="shell-session">
$ qemu-system-i386 -cdrom barebones.iso
</syntaxhighlight>

[[Category:Bare bones tutorials]]
[[Category:CSharp]]
[[Category:Inadvisable]]
````

<a id="w003241"></a>

## W003241 — Talk:C Sharp Bare Bones

Bron-ID: W003241
Namespace: Talk (1)
Revisie: 29216 | Datum: 2024-08-12T09:42:23Z
Bron: https://wiki.osdev.org/Talk:C_Sharp_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AC_Sharp_Bare_Bones&oldid=29216

````text
I admit this article is some quite blatent self-promotion but we were missing an article on this topic and I have no knowledge of other tool chains for this.  If anyone wishes to change this to some more-mainstream toolchain (e.g. mono) then please be my guest.  [[User:Jnc100|John]] 12:41, 15 June 2012 (CDT)

== Replacement with Native AOT compilation ==

Hiya! As I'm new to editing this wiki, I figured I should asked whether it's okay for me to replace the contents of this article with a more modern approach, using .NET's Native AOT compilation feature. It would practically describe the theory of operation behind https://github.com/MichalStrehovsky/zerosharp, and a tutorial on how to build such a project from the ground-up, but with some tweaks to run the produced binaries with a bootloader like Limine.

I imagine it would also be possible to create a .NET template that would do the steps I would outline in such an article out-right, but I feel like that's out of the scope of the wiki and an article describing what's actually going on would be nice.

--[[User:Ascerton|ascpixi]] ([[User talk:Ascerton|talk]]) 09:42, 12 August 2024 (UTC)
````

<a id="w003242"></a>

## W003242 — File:ISBN 032157351X.jpg

Bron-ID: W003242
Namespace: File (6)
Revisie: 13685 | Datum: 2012-06-16T05:46:15Z
Bron: https://wiki.osdev.org/File:ISBN_032157351X.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_032157351X.jpg&oldid=13685

````text
Algorithms, 4th ed.
````

<a id="w003243"></a>

## W003243 — File:ISBN 0262033844.jpg

Bron-ID: W003243
Namespace: File (6)
Revisie: 13686 | Datum: 2012-06-16T05:51:30Z
Bron: https://wiki.osdev.org/File:ISBN_0262033844.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0262033844.jpg&oldid=13686

````text
Introduction to Algorithms, 3rd ed.
````

<a id="w003244"></a>

## W003244 — File:ISBN 113318779X.jpg

Bron-ID: W003244
Namespace: File (6)
Revisie: 13688 | Datum: 2012-06-16T07:40:47Z
Bron: https://wiki.osdev.org/File:ISBN_113318779X.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_113318779X.jpg&oldid=13688

````text
Introduction to the Theory of Computation, 3rd ed.
````

<a id="w003245"></a>

## W003245 — Talk:Ada Bare Bones

Bron-ID: W003245
Namespace: Talk (1)
Revisie: 26307 | Datum: 2021-08-12T23:08:22Z
Bron: https://wiki.osdev.org/Talk:Ada_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AAda_Bare_Bones&oldid=26307

````text
First: Good work.

The "Bare Bones" series is intended to get people from the moment the power switch is flipped to their language of choice, perhaps with a "hello world" written to screen as a "this really works" message, and pointing out the major potholes on the road to come.

This here would either need some serious trimming ''down'' to be made into a ''bare'' bones, or ''extended'' into a "kernel in ADA tutorial". Compare the [[FreeBasic Bare Bones]], which does a good job at keeping it "bare", and the Pascal tutorial, which could also stand some trimming-down. -- [[User:Solar|Solar]] 04:58, 18 June 2012 (CDT)

== Targetting GNAT to bare metal ==
Should the 'Targetting GNAT to bare metal' section be removed? It's no longer relevant with the newer GCC versions. I've been able to get a consistently working cross-compiler build on Linux, and I'd be willing to do some work on a separate Ada cross-compiler article if that would help here. --[[User:Ajxs|Ajxs]] 22:09, 5 November 2019 (CST)
````

<a id="w003246"></a>

## W003246 — User:1ottsco

Bron-ID: W003246
Namespace: User (2)
Revisie: 13702 | Datum: 2012-06-18T20:13:37Z
Bron: https://wiki.osdev.org/User:1ottsco
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3A1ottsco&oldid=13702

````text
Hello. I am 1ottsco. I am (like everyone else on this website) writing an OS.

You can visit my website at [http://www.ottsco.net], but right now there isn't anything on it.
````

<a id="w003247"></a>

## W003247 — User talk:Love4boobies/Static Linking

Bron-ID: W003247
Namespace: User talk (3)
Revisie: 13723 | Datum: 2012-06-25T14:34:25Z
Bron: https://wiki.osdev.org/User_talk:Love4boobies/Static_Linking
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ALove4boobies%2FStatic_Linking&oldid=13723

````text
If I may: The whole thing is somewhat lacking contrast. What are you comparing to what? Whether or not you're doing whole-program compilation or separate translation units has little to do with whether you're using a static linker or something else. It seems you're mixing compiler issues, calling convention issues, header file / build system / dependency issues, and linker issues any which way. Presented like this, you're bound to get reflex reactions of the unfriendly kind. -- [[User:Solar|Solar]] 04:47, 18 June 2012 (CDT)

:Thanks for the feedback. I hope you don't mind me moving the discussion here. The comparison is between the technique used by current toolchains and the one I am proposing. The point is that there is an intimate relationship between translation units, object files, and build systems; these concepts were thought of together and serve a common purpose. However, just because the goal is good, doesn't mean the solution is: negative results can be seen in both the resulting machine code and compilation times---due to redundant recompilations. I'll try to shape up the article so that it explains the problem better. --[[User:Love4boobies|Love4boobies]] 09:16, 19 June 2012 (CDT)
````

<a id="w003248"></a>

## W003248 — User:Lionel/What kind of kernel should I make?

Bron-ID: W003248
Namespace: User (2)
Revisie: 16768 | Datum: 2014-09-20T03:22:45Z
Bron: https://wiki.osdev.org/User:Lionel/What_kind_of_kernel_should_I_make%3F
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALionel%2FWhat_kind_of_kernel_should_I_make%3F&oldid=16768

````text
{{In Progress}}
This is a personal choice. Some kernels may have advantages that others don't, but they also have disadvantages. You can start out with an already made kernel, and develop from there. Or you can create your own completely different kind of kernel that is 100% yours. It's your choice. There is no wrong way to develop a kernel, unless, of course it causes too many problems. But if you want to know what common parts make a kernel "sane" then keep on reading.

== Common Parts ==
A kernel needs to have some basic parts, even if they are stubs that call something else. Feel free to add anything that you think most kernels need.

===Interrupt handling ===
{{main|Interrupts}}
This is a must, because ''any'' kernel  needs to handle requests and notifications for the hardware. If the CPU doesn't support some type of signaling system, your best bet is polling.

===  Input/Output (I/O) ===
{{main|I/O Ports}}
You need I/O to communicate, how else do you control the HDD, tell the user what your doing or get input from the user?

Surprisingly, on some systems, you can communicate via memory spaces. For example: 0xB8000 is a memory space where you write characters while in 80x25 text mode..

== Common Types of Kernels ==
'''Note''': These are sorted by difficulty.

=== Megalithic ===
{{Rating|1|}}
{{main|Megalithic Kernel}}
A Megalithic kernel is probably the easiest because everything is in the kernel binary, which means that there is no task switching. There is also no user mode. This is probably the kind of kernel you should make if you are developing on a embedded system.

=== Monolithic ===
{{Rating|1|}}
{{main|Monolithic Kernel}}
Similar to a megalithic kernel; a Monolithic kernel is basically a big unchanging binary, and if there is any change, the kernel needs to be replaced again. It however has a user space. This design is fast, but can be unstable if not written correctly, as any bug can break the kernel easily. It would work well on a embedded system, but modern Monolithic Kernels, such as Linux, run on PC's as well. This is the second easiest, and is the one most people start out on, as components can be changed to create different types. Linux is a prime example of this, but most kernels will fall into this category.

=== Micro ===
{{Rating|2|}}
{{main|Microkernel}}
A Microkernel tries to export all functionality into user-space. It is generally very stable; if a server or daemon crashes, it can be restarted by the kernel or init/watchdog. The main thing you want to be fast here is the [[Inter-Process Communication]], as processes will need to be switched between quickly. This isn't the fastest, but probably is best when stability is required. This type of kernel is recommended for servers.

=== Hybrid ===
{{Rating|2|}}
{{main|Hybrid Kernel}}

Its two best friends, Monolithic and Micro have rubbed off on it and it acts like both. It is alone sometimes, but is happy to share with user-space. This system works quite well, as you don't have to change between modes for any privileged instruction, while code that doesn't need kernel privileges won't bring it down. However, discerning needs to be done, making it more difficult. Windows NT (XP+) and XNU(OSX) use this design.

=== Alien ===
{{Rating|4|}}
{{main|@*%*(@&%(@#}}

This is the essentially the "default" case in a switch function. An alien kernel is no different than any other one. It's just a funny name. An alien kernel is a kernel that has no "mold" to fit in. Something completely different.
````

<a id="w003249"></a>

## W003249 — User talk:Lionel/What kind of kernel should I make?

Bron-ID: W003249
Namespace: User talk (3)
Revisie: 13744 | Datum: 2012-06-28T07:00:30Z
Bron: https://wiki.osdev.org/User_talk:Lionel/What_kind_of_kernel_should_I_make%3F
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ALionel%2FWhat_kind_of_kernel_should_I_make%3F&oldid=13744

````text
I have a problem with the notion of "monolithic == unstable". For one, stability relies very much on the quality of code. The monolithic architecture is ''statistically'' more prone to kernel crashes than a microkernel ''if'' both have the same code quality. Two, while a microkernel might not experience ''kernel'' crashes as often, it takes quite some skill to have your system recover from, say, a crash of the filesystem server. If your system is rendered inoperable, it doesn't matter much if it was the kernel or a user-space process to blame... Hence, and to avoid rants like this very one, I would attempt to tender my statements a bit, not taking sides. -- [[User:Solar|Solar]] 04:14, 25 June 2012 (CDT)

== Making it a normal page. ==

Should I make this a normal page?

:Needs a bit more polish. For example, in the midst of the microkernel section, you talk about "archetypes" being created. Being a regular here, I have an idea what you're aiming at, but read your article with the eyes of a newbie, and you'll see where it could be confusing. Personally, I also prefer a more sombre tone in a tutorial, but that's just me. -- [[User:Solar|Solar]] 08:02, 27 June 2012 (CDT)

:Yeah, I guess, but not ''too'' sombre. --[[User:Lionel|Lionel]] 15:03, 27 June 2012 (CDT)

::Call it occupational hazard. I've been documenting my day's work for business departments for years. They don't even ''get'' the average joke. -- [[User:Solar|Solar]] 02:00, 28 June 2012 (CDT)
````

<a id="w003251"></a>

## W003251 — User talk:Love4boobies/Linkers

Bron-ID: W003251
Namespace: User talk (3)
Revisie: 13724 | Datum: 2012-06-25T14:34:25Z
Bron: https://wiki.osdev.org/User_talk:Love4boobies/Linkers
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ALove4boobies%2FLinkers&oldid=13724
Doorverwijzing: User talk:Love4boobies/Static Linking

````text
#REDIRECT [[User talk:Love4boobies/Static Linking]]
````

<a id="w003252"></a>

## W003252 — Talk:Partition Table

Bron-ID: W003252
Namespace: Talk (1)
Revisie: 27340 | Datum: 2022-09-03T18:21:45Z
Bron: https://wiki.osdev.org/Talk:Partition_Table
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APartition_Table&oldid=27340

````text
Information on this page is poor quality.

I don't think we should mention about an "unofficial" 48-bit format at all, because noone uses it. Unless someone *does* use it, in which case citation needed.

The stuff about extended partitions is incorrect. I'm going to try and change it a bit to reflect reality.

[[User:Jamesm|Jamesm]] 12:55, 28 June 2012 (CDT)

:I'd like the 48-bit format to be deleted or moved to its own page. It took me a couple of reads to realise I needed to just ignore it. [[User:Eekee|Eekee]] 23:31, 1 September 2022 (CDT)

::DONE [[User:Eekee|Eekee]] 13:21, 3 September 2022 (CDT)

== Merge with MBR? ==

This page is basically useless aside from the partition format, which really should be in MBR page, since that'll be where most people expect to find it.
Anyone else agree?
--[[User:Nakst|Nakst]] 08:03, 17 January 2016 (CST)
:I agree. "Partition Table" is too generic a name for a page, MBR is where the info belongs. This page should perhaps be a disambiguation page referencing MBR, GPT, and perhaps others such as the Win2k format. [[User:Eekee|Eekee]] 23:31, 1 September 2022 (CDT)
::Update: I find [[MBR]] too cluttered. It would be good to move its dual-boot material to a new page before merging this. [[User:Eekee|Eekee]] 01:49, 2 September 2022 (CDT)
:::On further perusal, [[MBR (x86)]] is a guide to the MBR portion of bootloader creation. It's not a partition table page at all, except for one section. [https://forum.osdev.org/viewtopic.php?f=8&t=56463&p=342206#p342206 See new forum discussion.] [[User:Eekee|Eekee]] 13:21, 3 September 2022 (CDT)
````

<a id="w003254"></a>

## W003254 — Calling C++ Constructors From C++

Bron-ID: W003254
Namespace: Main (0)
Revisie: 14730 | Datum: 2013-05-24T11:58:55Z
Bron: https://wiki.osdev.org/Calling_C%2B%2B_Constructors_From_C%2B%2B
Revisie-URL: https://wiki.osdev.org/index.php?title=Calling_C%2B%2B_Constructors_From_C%2B%2B&oldid=14730
Doorverwijzing: Calling Global Constructors

````text
#REDIRECT [[Calling Global Constructors]]
````

<a id="w003255"></a>

## W003255 — Talk:Calling C++ Constructors From C++

Bron-ID: W003255
Namespace: Talk (1)
Revisie: 14168 | Datum: 2012-12-19T16:11:39Z
Bron: https://wiki.osdev.org/Talk:Calling_C%2B%2B_Constructors_From_C%2B%2B
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ACalling_C%2B%2B_Constructors_From_C%2B%2B&oldid=14168

````text
Isn't "porting code from assembly to C++" a bit below the expected level of this wiki? I mean, everybody who enters this site ''should'' already have knowledge how to do it - and most of people will either 1) avoid global objects or just 2) use the assembly code from Bare Bones... -- [[User:Griwes|Griwes]] 11:21, 2 July 2012 (CDT)

== Calling constructors and destructors (incorrectly) ==

Hmm, [[Creating a C Library|having added support for this]] to my libc, it would look like this article is doing it <em>wrong</em>. It should use the crtbegin.o and crtend.o object files and use the the linker script to insert kernel-specific crti.o and crtn.o files in the right places, and then call _init. I am yet to do this my kernel, so I am not sure of the specifics. This tutorial seems to mess with GCC internals.
````

<a id="w003256"></a>

## W003256 — User:Griwes

Bron-ID: W003256
Namespace: User (2)
Revisie: 19634 | Datum: 2016-07-27T14:04:39Z
Bron: https://wiki.osdev.org/User:Griwes
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGriwes&oldid=19634

````text
My name is Michał Dominiak, commonly known as "Griwes". I'm a programming and computer science enthusiast from Poland, and I hold a Bachelor of Engineering in the field of Computer Science from Wrocław University of Technology. You can find my thesis, which was about implementing a HDMI controller in VHDL, [http://griwes.info/beng-thesis.pdf here] (in Polish).

For a while now I've worked at Nokia Networks in Wrocław; I also frequent C++ conferences in the US (I've attended C++Now in Aspen, CO three times now, twice as a speaker, and I've been a speaker at CppCon 2015, and soon also at 2016); I've also been spotted at the C++ standardization committee meeting in Oulu, Finland in 2016.

My OS project is [https://github.com/griwes/reaveros ReaverOS], or Rose.

==My pages==

* [[HPET]]
````

<a id="w003257"></a>

## W003257 — File:ISBN 0132916525.jpg

Bron-ID: W003257
Namespace: File (6)
Revisie: 13774 | Datum: 2012-07-05T19:27:37Z
Bron: https://wiki.osdev.org/File:ISBN_0132916525.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0132916525.jpg&oldid=13774

````text
Structured Computer Organization
````

<a id="w003258"></a>

## W003258 — File:PEFigure1.jpg

Bron-ID: W003258
Namespace: File (6)
Revisie: 13792 | Datum: 2012-07-07T10:07:48Z
Bron: https://wiki.osdev.org/File:PEFigure1.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3APEFigure1.jpg&oldid=13792

````text
PE file format
````

<a id="w003259"></a>

## W003259 — MADT

Bron-ID: W003259
Namespace: Main (0)
Revisie: 29938 | Datum: 2026-01-28T14:45:54Z
Bron: https://wiki.osdev.org/MADT
Revisie-URL: https://wiki.osdev.org/index.php?title=MADT&oldid=29938

````text
{{ACPI}}

The '''Multiple APIC Description Table''' ('''MADT''') is an [[ACPI]] table which provides informations necessary for operation on systems with [[APIC]], [[SAPIC]], [[GIC]], or [[LPIC]] implementations. As of June 17th, 2023, its latest definition is provided in the [[ACPI]] Specification Version 6.5, section [https://uefi.org/specs/ACPI/6.5/05_ACPI_Software_Programming_Model.html#multiple-apic-description-table-madt 5.2.12. ''Multiple APIC Description Table (MADT)''].

==Introduction==
The MADT describes all of the interrupt controllers in the system.  It can be used to enumerate the processors currently available.

You can look at the length field in the MADT's header to determine when you have read all the entries.

==Table Structure==
The MADT starts with the standard ACPI table header. The signature is 'APIC'.

{| {{wikitable}}
! Offset (hex)
! Length
! Description
|-
| 00 || 4 || Signature 'APIC'
|-
| 04 || 4 || Length
|-
| 08 || 1 || Revision
|-
| 09 || 1 || Checksum
|-
| 0a || 6 || OEMID
|-
| 10 || 8 || OEM Table ID
|-
| 18 || 4 || OEM Revision
|-
| 1c || 4 || Creator ID
|-
| 20 || 4 || Creator Revision
|}

After the standard header, the following fields are located.

{| {{wikitable}}
! Offset (hex)
! Length
! Description
|-
| 24 || 4 || Local APIC Address
|-
| 28 || 4 || Flags (1 = Dual 8259 Legacy PICs Installed)
|}

If bit 0 in the flags field is set then you need to mask all the 8259 PIC's interrupts, but you should probably do this anyway.

After the Flags field, starting at offset 0x2C, the rest of the MADT table contains a sequence of variable length records which enumerate the interrupt devices on this machine.  Each record begins with the following header fields.

{| {{wikitable}}
! Offset (hex)
! Length
! Description
|-
| 0 || 1 || Entry Type
|-
| 1 || 1 || Record Length
|}

Based on the Entry Type field value, the rest of the record layout can be determined.

== Entry Type 0: Processor Local APIC ==

This type represents a single logical processor and its local interrupt controller.

{| {{wikitable}}
! Offset (hex)
! Length
! Description
|-
| 2 || 1 || ACPI Processor ID
|-
| 3 || 1 || APIC ID
|-
| 4 || 4 || Flags (bit 0 = Processor Enabled) (bit 1 = Online Capable)
|}

If flags bit 0 is set the CPU is able to be enabled, if it is not set you need to check bit 1. If that one is set you can still enable it, if it is not the CPU can not be enabled and the OS should not try.

== Entry Type 1: I/O APIC ==

This type represents a I/O APIC. The global system interrupt base is the first interrupt number that this I/O APIC handles. You can see how many interrupts it handles using the register by getting the number of redirection entries from register 0x01, as described in [[APIC#IO_APIC_Registers|IO APIC Registers]].

{| {{wikitable}}
! Offset (hex)
! Length
! Description
|-
| 2 || 1 || I/O APIC's ID
|-
| 3 || 1 || Reserved (0)
|-
| 4 || 4 || I/O APIC Address
|-
| 8 || 4 || Global System Interrupt Base
|}

== Entry Type 2: I/O APIC Interrupt Source Override ==

This entry type contains the data for an Interrupt Source Override.
This explains how IRQ sources are mapped to global system interrupts.
For example, IRQ source for the timer is 0, and the global system interrupt will usually be 2. So you could look for the I/O APIC with the base below 2 and within its redirection entries, then make the redirection entry for (2 - base) to be the timer interrupt.

{| {{wikitable}}
! Offset (hex)
! Length
! Description
|-
| 2 || 1 || Bus Source
|-
| 3 || 1 || IRQ Source
|-
| 4 || 4 || Global System Interrupt
|-
| 8 || 2 || Flags (see below)
|}

== Entry type 3: I/O APIC Non-maskable interrupt source ==

Specifies which I/O APIC interrupt inputs should be enabled as non-maskable.

{| {{wikitable}}
! Offset (hex)
! Length
! Description
|-
| 2 || 1 || NMI Source
|-
| 3 || 1 || Reserved
|-
| 4 || 2 || Flags (see below)
|-
| 6 || 4 || Global System Interrupt
|}

== Entry Type 4: Local APIC Non-maskable interrupts ==

Configure these with the LINT0 and LINT1 entries in the Local vector table of the relevant processor(')s(') local APIC.

{| {{wikitable}}
! Offset (hex)
! Length
! Description
|-
| 2 || 1 || ACPI Processor ID (0xFF means all processors)
|-
| 3 || 2 || Flags (see below)
|-
| 5 || 1 || LINT# (0 or 1)
|}

== Entry Type 5: Local APIC Address Override ==

Provides 64 bit systems with an override of the physical address of the Local APIC. There can only be one of these defined in the MADT. If this structure is defined, the 64-bit Local APIC address stored within it should be used instead of the 32-bit Local APIC address stored in the MADT header.

{| {{wikitable}}
! Offset (hex)
! Length
! Description
|-
| 2 || 2 || Reserved
|-
| 4 || 8 || 64-bit physical address of Local APIC
|}

== Entry Type 9: Processor Local x2APIC ==

Represents a physical processor and its Local x2APIC. Identical to Local APIC; used only when that struct would not be able to hold the required values.

{| {{wikitable}}
! Offset (hex)
! Length
! Description
|-
| 2 || 2 || Reserved
|-
| 4 || 4 || Processor's local x2APIC ID
|-
| 8 || 4 || Flags (same as the Local APIC flags)
|-
| C || 4 || ACPI ID
|}

== Flags ==
Entries of type 2, 3 and 4 have a flags field which is useful for setting up the I/O APIC redirection entry or the local vector lable entry.  Its format is as follows:
{| class="wikitable"
|+
!Offset
!Length
!Description
|-
|0
|2
|Determines the polarity <br>
0b00 - No override, use default settings for bus<br>
0b01 - Active high override<br>
0b10 - Reserved<br>
0b11 - Active low override<br>
|-
|2
|2
|Determines the trigger mode <br>
0b00 - No override, use default settings for bus <br>
0b01 - Edge triggered <br>
0b10 - Reserved <br>
0b11 - Level triggered <br>
|-
|4
|12
|Reserved
|}
The default for the ISA bus is edge-triggered active high.

Here is a diagram of the interrupt types:

[[File:Edge_vs_level.png]]

==See also==
*[[ACPI]]
*[[RSDT]]

[[Category:ACPI]]
````

<a id="w003260"></a>

## W003260 — Talk:Reboot

Bron-ID: W003260
Namespace: Talk (1)
Revisie: 13805 | Datum: 2012-07-15T18:14:34Z
Bron: https://wiki.osdev.org/Talk:Reboot
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AReboot&oldid=13805

````text
== Short code to do a 8042 reset  ==
While I understand the idea that this code should be taken as an example on how to implement, and not a copypasta, I have no idea what the frz() function is, or why it's relevant --[[User:Primis|Primis]] 13:11, 15 July 2012 (CDT)
*(Looked it over in comparison, its an implementation of halt, not very clear though...) --[[User:Primis|Primis]] 13:14, 15 July 2012 (CDT)
````

<a id="w003261"></a>

## W003261 — Printing to Screen

Bron-ID: W003261
Namespace: Main (0)
Revisie: 13825 | Datum: 2012-07-17T22:03:51Z
Bron: https://wiki.osdev.org/Printing_to_Screen
Revisie-URL: https://wiki.osdev.org/index.php?title=Printing_to_Screen&oldid=13825
Doorverwijzing: Printing To Screen

````text
#REDIRECT [[Printing To Screen]]
````

<a id="w003262"></a>

## W003262 — Talk:Printing to Screen

Bron-ID: W003262
Namespace: Talk (1)
Revisie: 13827 | Datum: 2012-07-17T22:03:51Z
Bron: https://wiki.osdev.org/Talk:Printing_to_Screen
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APrinting_to_Screen&oldid=13827
Doorverwijzing: Talk:Printing To Screen

````text
#REDIRECT [[Talk:Printing To Screen]]
````

<a id="w003263"></a>

## W003263 — User:Virtlink/My dream CPU

Bron-ID: W003263
Namespace: User (2)
Revisie: 13856 | Datum: 2012-07-26T12:31:25Z
Bron: https://wiki.osdev.org/User:Virtlink/My_dream_CPU
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AVirtlink%2FMy_dream_CPU&oldid=13856

````text
My ideal CPU would be more high-level than most. The advantage of having a high-level CPU is that it can have more knowledge about the software than most, and use this information for run-time optimizations. For example, having a single possible calling convention and providing just one ''call'' and one ''ret'' instruction allows the CPU to do anything to make the call happen, without caring about ''how'' it is done. Today the difference between non-load/store (CISC) and load/store (RISC) processors has gotten a lot smaller. Transistors became smaller and cheaper, instructions are heavily pipelined and parallelized, and non-load/store processors are again viable.

=== Memory and registers ===
My CPU would have 64-bit virtual and physical addresses, but does not require the availability of the whole virtual and/or physical address space. For virtual addresses, less than 64-bits may be actually used ([http://en.wikipedia.org/wiki/X86-64#Canonical_form_addresses canonical form addresses]) resulting in a higher half and a lower half of the virtual memory.

The CPU has 32 64-bit general purpose (integer) registers and 32 256-bit SIMD/floating-point registers.

The CPU also tracks the free pages, and it can do that really fast.

=== Stack ===
Stacks are 64-bit. In my ideal CPU they are managed solely by the CPU, and the CPU also manages the stack frames. There are instructions to allocate stacks, destroy them, allocate a chunk of stack memory, calling procedures, returning from procedures and of course pushing and popping values.

The CPU is also responsible for moving the arguments on the stack to the callee, and moving the results back to the caller. The CPU can do this extremely fast, might even used hidden caches or registers for this.

The CPU managed stack opens up all kinds of new possibilities. For example, my CPU would also manage exception handling: no more magic numbers or error codes, but a pointer (which presumably points to some exception object, but the CPU does not care) that is propagated up the call chain. As the CPU knows the exact stack frames, it can unwind the stack and run the exception filters, handlers and finalization code it encounters on its way. If the stack ever runs out, the CPU runs out of memory or there is a division by zero, the CPU will throw the appropriate exception (instead of interrupting).

Also, my CPU would manage the context switches and threads. This can follow naturally as all stacks are managed by the CPU, and CPU just manages the call stack for each thread. The CPU can optimize the saving of the state as it desires.

=== Instruction set ===
The instruction set is a non-load/store variable-length instruction set, again with the argument that higher-level instructions may give the CPU more freedom to optimize. Each instruction consists of a 2-byte opcode, an 1-byte addressing mode specifier, and up to 4 operands.

The first opcode byte selects the opcode map to use, there are 128 available for general use and extensions (such as SIMD extensions) and 128 for vendor-specific extensions. Each opcode map contains up to 255 instructions (encoded in the second opcode byte). This gives a theoretical maximum of 32767 possible general instructions and 32767 vendor-specific instructions.

More at [[User:Virtlink/My dream CPU/ISA|ISA]]

=== CPU functions ===
Instructions tend to be small, quick things that are executed in the blink of an eye. However, many of the basic stuff in for example the C library is required on all computers, and people try to some up with more and more sophisticated methods to squeeze that tiny extra bit of power from ''memcpy'' and ''memset''. In my CPU, there are special CPU functions that perform such operations. These are usually not finished within a single CPU cycle, but the CPU has the ability to do it as optimal as possible. When the CPU vendor wants to speed 99% of the programs up, all it has to do is spend some more transistors in the ''memcpy'' area. On the other hand, an embedded version might include a 1 MiB on-die ROM chip with machine code representations of the functions.
````

<a id="w003264"></a>

## W003264 — User:Virtlink/My dream CPU/ISA

Bron-ID: W003264
Namespace: User (2)
Revisie: 13870 | Datum: 2012-07-27T13:11:41Z
Bron: https://wiki.osdev.org/User:Virtlink/My_dream_CPU/ISA
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AVirtlink%2FMy_dream_CPU%2FISA&oldid=13870

````text
== Registers ==
The CPU has the following registers:
<div style="font-size: smaller; float: left; margin: 10px">
{| {{wikitable}}
! Value
! 64-bit GP
! 256-bit SIMD
|-
| 0||R0||V0
|-
| 1||R1||V1
|-
| 2||R2||V2
|-
| 3||R3||V3
|-
| 4||R4||V4
|-
| 5||R5||V5
|-
| 6||R6||V6
|-
| 7||R7||V7
|-
| 8||R8||V8
|-
| 9||R9||V9
|-
| 10||R10||V10
|-
| 11||R11||V11
|-
| 12||R12||V12
|-
| 13||R13||V13
|-
| 14||R14||V14
|-
| 15||R15||V15
|}
</div>
<div style="font-size: smaller; float: left; margin: 10px">
{| {{wikitable}}
! Value
! 64-bit GP
! 256-bit SIMD
|-
| 16||R16||V16
|-
| 17||R17||V17
|-
| 18||R18||V18
|-
| 19||R19||V19
|-
| 20||R20||V20
|-
| 21||R21||V21
|-
| 22||R22||V22
|-
| 23||R23||V23
|-
| 24||R24||V24
|-
| 25||R25||V25
|-
| 26||R26||V26
|-
| 27||R27||V27
|-
| 28||R28||V28
|-
| 29||R29||V29
|-
| 30||R30||V30
|-
| 31||PC||V31
|}
</div>
<div style="clear: both">&nbsp;</div>

== Instruction encoding ==

Every instruction consists of an opcode, an operand specifier and a number of operands. The source operands (if any) come before the destination operands (if any), in the order specified in the opcode specification. An opcode represents an operation, and the operand size and operands of the operation are encoded explicitly in the instruction. This allows tools to determine the instruction boundaries without knowing the instruction. The CPU can determine the length of the instruction from the opcode and operand specifier alone.

=== Opcode ===
Opcodes may be encoded in various sizes. Compilers are encouraged to choose the smallest opcode size that fits the opcode, but this is not required.

==== 7-bit opcode ====
<pre>
  7                           0
+---+---+---+---+---+---+---+---+
| 0 |          OPCODE           |
+---+---+---+---+---+---+---+---+
</pre>
This 7-bit opcode has all explicit operands set to none, and uses the default operand size, and assumes the values are unsigned. It is used for instructions that have no (explicit) operands and no need to set the operand size to anything other than 64-bit.

An example of a single byte instruction is the NOP instruction, with opcode 0.

==== 11, 18 and 25-bit opcodes ====
These opcodes are more advanced. The CPU can decode the operand size from the opcode and expects an operand specifier to follow the opcode. 
Note that the opcode encoding allows for future CPUs to support opcodes values larger than 25-bit.
<pre>
  15  14                                            3   2       0
+---+---+---+---+---+---+---+---+ +---+---+---+---+---+---+---+---+
| 1 | 0 |                       OPCODE                | US|   OS  |
+---+---+---+---+---+---+---+---+ +---+---+---+---+---+---+---+---+
</pre>
<pre>
  23  22  21                                                                          3   2       0
+---+---+---+---+---+---+---+---+ +---+---+---+---+---+---+---+---+ +---+---+---+---+---+---+---+---+
| 1 | 1 | 0 |                                   OPCODE                                  | US|   OS  |
+---+---+---+---+---+---+---+---+ +---+---+---+---+---+---+---+---+ +---+---+---+---+---+---+---+---+
</pre>
<pre>
  31  30  29  28                                                                                     
+---+---+---+---+---+---+---+---+ +---+---+---+---+---+---+---+---+ +---+---+---+---+---+---+---+---+
| 1 | 1 | 1 | 0 |                                                OPCODE                             ..
+---+---+---+---+---+---+---+---+ +---+---+---+---+---+---+---+---+ +---+---+---+---+---+---+---+---+
                   3   2       0
 +---+---+---+---+---+---+---+---+
..                   | US|   OS  |
 +---+---+---+---+---+---+---+---+
</pre>
{| {{wikitable}}
! Field
! Size
! Description
|-
| US||1-bit||Specifies the interpretation of the values:<br />0: Unsigned<br />1: Signed
|-
| OS||2-bit||Specifies the operand size (<tt>(1 << (6 - OS))</tt>-bit):<br />0: 64-bit<br />1: 32-bit<br />2: 16-bit<br />3: 8-bit
|}

==== Operand specifier ====
The operand specifier can specify from one up to six operands. The encoding may seem arbitrary, but is chosen in such way that the operands are encoded as efficiently as possible. Note that the operand specifier encoding allows for future CPUs to support instructions with more than six operands.
<pre>
  7   6   5                   0
+---+---+---+---+---+---+---+---+
| 1 | 1 | 0 |        OP1        |
+---+---+---+---+---+---+---+---+
</pre>
<pre>
  15                  10                    5                   0
+---+---+---+---+---+---+---+---+ +---+---+---+---+---+---+---+---+
| 0 |        OP1        |          OP2        |         OP3       |
+---+---+---+---+---+---+---+---+ +---+---+---+---+---+---+---+---+
</pre>
<pre>
  23  22  21  20                    15                  10                    5                   0
+---+---+---+---+---+---+---+---+ +---+---+---+---+---+---+---+---+ +---+---+---+---+---+---+---+---+
| 1 | 1 | 1 | 0 |        OP1          |           OP2     |         OP3         |        OP4        |
+---+---+---+---+---+---+---+---+ +---+---+---+---+---+---+---+---+ +---+---+---+---+---+---+---+---+
</pre>
<pre>
  31  30                  25                    20                    15                  10
+---+---+---+---+---+---+---+---+ +---+---+---+---+---+---+---+---+ +---+---+---+---+---+---+---+---+
| 1 | 0 |        OP1        |           OP2       |         OP3         |        OP4        |  OP5  ..
+---+---+---+---+---+---+---+---+ +---+---+---+---+---+---+---+---+ +---+---+---+---+---+---+---+---+
           5                   0
 +---+---+---+---+---+---+---+---+
..   OP5     |         OP6       |
 +---+---+---+---+---+---+---+---+
</pre>
Each operand specification has the following layout:
<pre>
  4               0
+---+---+---+---+---+
| M | I | B |   S   |
+---+---+---+---+---+
</pre>
{| {{wikitable}}
! Field
! Size
! Description
|-
| M||1-bit||0: Direct<br />1: Indirect
|-
| I||1-bit||0: No index<br />1: Index register * scale
|-
| B||1-bit||0: No base<br />1: Base register
|-
| S||2-bit||0: No immediate/offset/displacement<br />1: 16-bit immediate/offset/displacement<br />2: 32-bit immediate/offset/displacement<br />3: 64-bit immediate/offset/displacement
|}
Some examples:
{| {{wikitable}}
! To encode
! M
! I
! B
! S
|-
| ''nothing'' or ''literal'' 0||0||0||0||0
|-
| reg||0||0||1||0
|-
| [reg]||1||0||1||0
|-
| [offset32]||1||0||0||2
|-
| [base+index*scale+disp16]||1||1||1||1
|-
| imm64||0||0||0||3
|-
| reg+imm32||0||0||1||2
|-
| base+index*scale+imm16 (like <tt>LEA</tt>)||0||1||1||1
|-
| [0] (''null reference'')||1||0||0||0
|}
Note however that not all possible operand types are allowed for all operands. For example, destination operands must either be indirect (<tt>M=1</tt>) or be only a register (<tt>B=1, I=0, S=0</tt>). When an operand is not specified or none (<tt>M=I=B=S=0</tt>) then the behavior depends on the instruction. Some instructions missing a destination operand use a source operand also as destination, others treat the operand as the literal value 0, offset 0 or as register R0. Instructions that have not enough or incorrect operands throw an exception.

==== Operand encoding ====
===== Immediates, displacements and offsets =====
Immediate, displacement and offset values are encoded in little-endian order with the number of bits specified in the operand specifier.

When the immediate value is less than the operand size, it is sign-extended or zero-extended depending on the instruction. When the immediate value is greater than the operand size, excess bits are trimmed off.

When the displacement value is less than the address size (64-bit), it is sign-extended.

When the offset value is less than the address size (64-bit), it is zero-extended.

===== Register, base register =====
A register operand or base register (both indirect and direct) is encoded as:
<pre>
  7   6   5                   0
+---+---+---+---+---+---+---+---+
| 0 | 0 | 0 |        REG        |
+---+---+---+---+---+---+---+---+
</pre>
This encoding may seem wasteful, but instruction size is not such a big issue with modern CPUs. This encoding of a register is consistent, simple and future CPUs may extend the syntax to support more than 32 registers.

The <tt>PC</tt> (program counter) register has value 31 and may be used everywhere a register is expected. However, its use is not allowed as a destination operand, and in some addressing modes the use of <tt>PC</tt> is meaningless (e.g. using <tt>PC</tt> as an index register with a scale other than 1).


===== Index * scale =====
An index register with a scaling factor is encoded as:
<pre>
+---+---+---+---+---+---+---+---+ +---+---+---+---+---+---+---+---+
| 0 | 0 | 0 |       INDEX       | | 0 | 0 | 0 | 0 | 0 |   SCALE   |
+---+---+---+---+---+---+---+---+ +---+---+---+---+---+---+---+---+
</pre>
The scaling factor is not encoded with the index register to keep register encoding consistent and to allow for future register extensions. The scaling factor is a power of two (<tt>1 << SCALE</tt>).
{| {{wikitable}}
! Value <tt>SCALE</tt>
! Scaling factor
|-
| b000||1
|-
| b001||2
|-
| b010||4
|-
| b011||8
|-
| b100||16
|-
| b101||32
|-
| b110||64
|-
| b111||128
|}

=== Instruction sizes ===
{| {{wikitable}}
! !! colspan="3" | 0-7 bit operand !! colspan="3" | 8-11 bit operand !! colspan="3" | 12-18 bit operand !! colspan="3" | 19-25 bit operand 
|-
! !! Min !! Max !! Ineff. !! Min !! Max !! Ineff. !! Min !! Max !! Ineff. !! Min !! Max !! Ineff.
|-
! 0 operands
|1||1||6||3||3||6||4||4||7||5||5||8
|-
! 1 operand
|4||14||17||4||14||17||5||15||18||6||16||19
|-
! 2 operands
|6||26||28||6||26||28||7||27||29||8||28||30
|-
! 3 operands
|7||37||39||7||37||39||8||38||40||9||39||41
|-
! 4 operands
|9||47||48||9||47||48||10||48||49||11||49||50
|-
! 5 operands
|11||61||61||11||61||61||12||62||62||13||63||63
|-
! 6 operands
|12||72||72||12||72||72||13||73||73||14||74||74
|}
<small>Min: minimum size with smallest operands<br />
Max: maximum size with largest operands<br />
Ineff: maximum size with largest operands and unnecessarily inefficient encoding</small>
````

<a id="w003265"></a>

## W003265 — File:ISBN 0073523402.jpg

Bron-ID: W003265
Namespace: File (6)
Revisie: 13858 | Datum: 2012-07-26T13:06:55Z
Bron: https://wiki.osdev.org/File:ISBN_0073523402.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0073523402.jpg&oldid=13858

````text

````

<a id="w003266"></a>

## W003266 — File:ISBN 0138297711.jpg

Bron-ID: W003266
Namespace: File (6)
Revisie: 13859 | Datum: 2012-07-26T13:07:32Z
Bron: https://wiki.osdev.org/File:ISBN_0138297711.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0138297711.jpg&oldid=13859

````text

````

<a id="w003267"></a>

## W003267 — File:ISBN 1558604960.jpg

Bron-ID: W003267
Namespace: File (6)
Revisie: 13860 | Datum: 2012-07-26T13:07:52Z
Bron: https://wiki.osdev.org/File:ISBN_1558604960.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_1558604960.jpg&oldid=13860

````text

````

<a id="w003268"></a>

## W003268 — User:Tudoros1

Bron-ID: W003268
Namespace: User (2)
Revisie: 14007 | Datum: 2012-09-22T08:33:12Z
Bron: https://wiki.osdev.org/User:Tudoros1
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATudoros1&oldid=14007

````text
== My development environment ==

* Assembler: Flat Assembler 1.69.31
* Compiler: (Untraditional)Visual Studio 2010
* Operating system(for developing on): Windows 7 Professional x64


== My Operating System "Auckland" ==

My OS has the following files

* Kernel mode
** Includes my KDKHal (KDKHal.dll)(my hardware abstraction layer, and subsystem, CRT considered needing a full OS, everything will be built on top of this).
** Auckland.dll (My Micro kernel and core services are in the kernel proper) Contains:
*** The Micro kernel(Everything but the KDKHal is built on top of this)
**** Basic time manager(a timer built off the KDKHal Time services and a variable).
**** Advanced memory manager(contains virtual and physical memory allocation).
**** Context switcher(The scheduler).
**** The Common namespace(Stores all devices, files, processes, threads, basically anything, like a namespace of structs and the Unix File system)
*** The Advanced Process and Thread manager(Built on top of the scheduler)
*** The IPC manager:
**** Shared memory(Built on top of the memory manager)
**** Shared-copied memory(Built on top of the memory manager and shared memory)
**** Pipes(Built on top of shared-copied memory)
**** Messages(Built on top of pipes)
*** The module loader(loads Executables so you don't have to, and allows new file formats. Internally supports MZ, NE, LE, PE, PE+, ELF, NOP and TCATEXE)
*** System component manager(Very Important, uses the Module loader, which in turn uses this component, sort of a loop, so Exe Format modules can only be in one of the internal formats)
*** IO Manager(for file, device and driver management)
*** The session manager
*** The Runtime(Implements the CRT)
*** The Syscall Interface(contains services itself for defining Kernel implemented User visible services and the Restrictions Manager for security(The Common Namespace does most of this, but this does high level restrictions like not allowing certain functions to be called
** System components(Seamless extension(if you have someone using it))
*** Drivers
*** Executable module loaders
*** aklcomss.dll(For implementing the console and other things) 
*** Compatibility engines(used in 64 bit)
* User mode
** Akldll.dll(nothing to see, except its ability to switch to other processes for User component support, contains the native API)
** User services(someone should put stuff like GUI, Graphics, Network, Sound and other things in here, basically system components in user mode)
** AklCRT.dll(the C library and the main API(yep, no OS-Specific interface like Win32))
** The applications(Every OS has one)
*** Their DLLs

For more info on Auckland, see [[User:Tudoros1/OSes/Auckland|Auckland]] on my user pages
````

<a id="w003269"></a>

## W003269 — File:ISBN 0136108040.jpg

Bron-ID: W003269
Namespace: File (6)
Revisie: 13877 | Datum: 2012-07-28T05:04:16Z
Bron: https://wiki.osdev.org/File:ISBN_0136108040.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0136108040.jpg&oldid=13877

````text
Computer Systems: A Programmer's Perspective, 2nd ed.
````

<a id="w003270"></a>

## W003270 — File:ISBN 013230998X.jpg

Bron-ID: W003270
Namespace: File (6)
Revisie: 13878 | Datum: 2012-07-28T05:05:31Z
Bron: https://wiki.osdev.org/File:ISBN_013230998X.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_013230998X.jpg&oldid=13878

````text
Operating Systems: Internals and Design Principles, 7th ed.
````

<a id="w003271"></a>

## W003271 — File:ISBN 1588530000.jpg

Bron-ID: W003271
Namespace: File (6)
Revisie: 13879 | Datum: 2012-07-28T05:06:41Z
Bron: https://wiki.osdev.org/File:ISBN_1588530000.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_1588530000.jpg&oldid=13879

````text
MMURTL V1.0
````

<a id="w003273"></a>

## W003273 — File:ISBN 0130525642.jpg

Bron-ID: W003273
Namespace: File (6)
Revisie: 13882 | Datum: 2012-07-28T05:20:36Z
Bron: https://wiki.osdev.org/File:ISBN_0130525642.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0130525642.jpg&oldid=13882

````text
Assemblers and Loaders
````

<a id="w003274"></a>

## W003274 — File:ISBN 071678291X.jpg

Bron-ID: W003274
Namespace: File (6)
Revisie: 13884 | Datum: 2012-07-28T13:14:53Z
Bron: https://wiki.osdev.org/File:ISBN_071678291X.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_071678291X.jpg&oldid=13884

````text
Instructor's Manual for Language of Machines
````

<a id="w003275"></a>

## W003275 — File:ISBN 0471117099.jpg

Bron-ID: W003275
Namespace: File (6)
Revisie: 13885 | Datum: 2012-07-28T13:18:05Z
Bron: https://wiki.osdev.org/File:ISBN_0471117099.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0471117099.jpg&oldid=13885

````text
Applied Crypto
````

<a id="w003276"></a>

## W003276 — User:Shikhin

Bron-ID: W003276
Namespace: User (2)
Revisie: 14006 | Datum: 2012-09-22T07:05:14Z
Bron: https://wiki.osdev.org/User:Shikhin
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AShikhin&oldid=14006

````text
I'm Shikhin Sethi. 

My project is outlined at my [http://shikhin.in website], and the source is available at the [http://github.com/Shikhin/Draumr GitHub] page.

== Contributions ==

I have contributed to [[Entering Long Mode Directly]] and [[PXE]] heavily. Other than that, my contributions are sparse.
````

<a id="w003281"></a>

## W003281 — Talk:C Library

Bron-ID: W003281
Namespace: Talk (1)
Revisie: 13984 | Datum: 2012-09-13T19:32:42Z
Bron: https://wiki.osdev.org/Talk:C_Library
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AC_Library&oldid=13984

````text
=types=
It sounds to me like the "types" section is totally backwards.
[[User:Bewing|Bewing]] 20:00, 31 August 2012 (CDT)

:I changed the name of the section to the more appropriate "Versions," and slightly improved it. Do you have anything more specific in mind? --[[User:Love4boobies|Love4boobies]] 09:16, 2 September 2012 (CDT)

:The new wording with the extra information is better. However, first, the way osdev uses these libs, the words "hosted" and "freestanding" do not have useful meanings -- they add confusion. The words "kernel" and "userland runtime library" have much clearer meanings, and I think these words need to be added. Second, isn't the "hosted" library the small, incomplete, subset one that is linked into the kernel? The article text has it the other way around. If I have it wrong, that just shows how confusing the terms are. ;) --[[User:Bewing|Bewing]] 15:24, 4 September 2012 (CDT)

::Damn, I knew I couldn't resist doing the occasional edit... so: The "freestanding" part of the library are all those headers that add defines and typedefs, ''but no code''. As such, they require neither a runtime nor a linker. A "hosted" library is the whole of it, i.e. including those headers that require accompaining object code. This has ''nothing'' to do with kernel / userland. I would expect virtually every kernel to use (a subset of) the hosted lib. -- [[User:Solar|Solar]] 23:57, 4 September 2012 (CDT)

::Thanks very much for the clarification. I think we should stick the paragraph you just wrote into the  article. I'd say the whole concept of a "freestanding" clib is pretty useless in the context of osdev, then. --[[User:Bewing|Bewing]] 02:42, 5 September 2012 (CDT)

:::I was the one who wrote that paragraph. Freestanding implementations are not irrelevant to the world of OS development; they were introduced to the standard for this very purpose (and for embedded programming). The idea is that they don't need to communicate with the underlying environment and can thus always be used for functionality such as variadic functions and macros that indicate arithmetic type ranges; on the other hand, hosted implementations require a lot of rewriting and/or porting---this stuff cannot be used out of the box. E.g., the recommandation generally given to GCC users on this website is that they build a cross-compiler; at that point, they can only use the freestanding library.

:::However, what Solar says is true---while most of the hosted library has no place in a kernel, some of it makes sense (most notably, string handling). There is an important distinction, though. The main purpose of the standard library is to allow for things to be done portably. As I've mentioned in the paragraph above, the hosted stuff is not portable and since it has to be part of the OS, the advantage of portability, that I've just mentioned, goes away. It thus follows that it doesn't matter whether the OS developer implements standard functions or rolls his/her own incompatible ones that have similar goals. On the other hand, the freestanding stuff should always be used because it's always portable and thus increases the portability of the OS as a whole.

:::--[[User:Love4boobies|Love4boobies]] 11:46, 5 September 2012 (CDT)

:::I think I disagree about the portability issue. In the end any OS is going to need a userland runtime C library -- as non-portable as that may be. Once a developer has gone to the work of creating that, they do not lose any portability by adding some of it into their kernel, too. Most developers that I know of add a C lib to their kernel in order to get printf functionality, and that's it. It's silly of them, because the variadic interface is a giant bug-filled security hole in your kernel, so you really don't want it there. A kernel does very little string handling. A kernel also does very little math, and therefore arithmetic type ranges are not useful, either. But if you think that freestanding C libs are something that developers should care about and use, then you should add another paragraph to the article explaining why, I think. Because it certainly isn't intuitively obvious to me. --[[User:Bewing|Bewing]] 16:32, 5 September 2012 (CDT)

:::I didn't mean that they lose portability because it's in the kernel. I meant that the freestanding library offers portability for free and should thus be used, whereas the hosted library's portability is the developer's responsability (regardless of where it is implemented) and is thus unimportant whether it is used or whether something else is used instead. Trying to come up with a new freestanding library would increase maintainability, whereas coming up with a new hosted library wouldn't. Other clarifications:

:::* I don't see any reason to have I/O (such as <tt>printf</tt> or <tt>scanf</tt>) in the kernel.
:::* You don't need to be math-intensive to need to know the type ranges (e.g., think of <tt>SIZE_MAX</tt>).
:::* By string handling, I was mostly thinking of <tt>memset</tt>, <tt>memcpy</tt> and such, which are declared in <tt>&lt;string.h&gt;</tt>.

:::--[[User:Love4boobies|Love4boobies]] 23:57, 5 September 2012 (CDT)

I think talking about portability is missing the point. The "freestanding" part of the library is something that is ''solely'' depending on the implementation of the compiler. GCC, for example, provides its own versions of the freestanding C library headers. -- [[User:Solar|Solar]] 04:38, 12 September 2012 (CDT)

:It's not missing the point at all. It offers information about the implementation which is used to compile the source for the purpose of portability. In the context of kernel development, code that uses the hosted features is not (as) portable because it requires either porting or implementing said hosted features. (Not sure what GCC has to do with anything.) --[[User:Love4boobies|Love4boobies]] 14:31, 13 September 2012 (CDT)

=Is Combuster Being Overly Pedantic Again=
The whole point of MIT and BSD licenses is that some MIT and Berkeley lawyers worked to create copyright licenses that were as close as humanly legally possible to "public domain" without actually using the words "public domain". You can use, modify, and sell the software to your heart's content, with the only restriction being to maintain the license. I say this is "basically public domain". Combuster says no, this does not meet the precise definition of public domain. Does anyone else wish to comment? [[User:Bewing|Bewing]] 20:48, 11 September 2012 (CDT)

:Then I hope you haven't stubbornly violating license agreements by omitting BSDs demand for '''attribution''', even though in commercial contexts it's sadly standard practice. Yes BSD-style licenses are useful for many things, but contrary to public domain and the later CC0, WTFPL and such you are required to do legal bookkeeping, and from every snippet you grab from the web follows the consequence of needing an extra half a page in your LICENSES document.
:Also, the direct consequence of your assumption is that BSD and MIT with all its paid lawyers failed epically at writing the appropriate license texts because the CC0 does things much better than that. Hence, I can't accept the factual accuracy of your statement. - [[User:Combuster|Combuster]] 07:58, 12 September 2012 (CDT)
````

<a id="w003282"></a>

## W003282 — ARP

Bron-ID: W003282
Namespace: Main (0)
Revisie: 13976 | Datum: 2012-09-11T22:20:58Z
Bron: https://wiki.osdev.org/ARP
Revisie-URL: https://wiki.osdev.org/index.php?title=ARP&oldid=13976
Doorverwijzing: Address Resolution Protocol

````text
#REDIRECT [[Address_Resolution_Protocol]]
````

<a id="w003283"></a>

## W003283 — Using Linker Script Values

Bron-ID: W003283
Namespace: Main (0)
Revisie: 30210 | Datum: 2026-03-12T19:58:09Z
Bron: https://wiki.osdev.org/Using_Linker_Script_Values
Revisie-URL: https://wiki.osdev.org/index.php?title=Using_Linker_Script_Values&oldid=30210

````text

Q. Whenever I use a value defined in my linker script, I get garbage.

A. You're not taking the address of the symbol. Define it in [[C]] as <code>extern char symbolname[];</code> instead of <code>extern int symbolname;</code>

== Long Answer ==
A common problem is getting garbage data when trying to use a value defined in a linker script. This is usually because they're dereferencing the symbol. A symbol defined in a linker script (e.g. <code>_ebss = .;</code>) is only a symbol, not a variable. If you access the symbol using <code>extern uint32_t _ebss;</code> and then try to use <code>_ebss</code> the code will try to read a 32-bit integer from the address indicated by <code>_ebss</code>.

The solution to this is to take the address of <code>_ebss</code> either by using it as <code>&amp;_ebss</code> or by defining it as an unsized array (<code>extern char _ebss[];</code>) and casting to an integer. (The array notation prevents accidental reads from <code>_ebss</code> as arrays must be explicitly dereferenced)

[[Category:Linkers]]
[[Category:FAQ]]
[[Category:Troubleshooting]]
````

<a id="w003285"></a>

## W003285 — User:Tudoros1/OSes/Auckland

Bron-ID: W003285
Namespace: User (2)
Revisie: 14014 | Datum: 2012-09-23T09:38:21Z
Bron: https://wiki.osdev.org/User:Tudoros1/OSes/Auckland
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATudoros1%2FOSes%2FAuckland&oldid=14014

````text
This page is dedicated to Auckland, my OS. Here are the main components

== Kernel mode components ==
This area contains the kernel mode components in Auckland. Overall, in the OS proper, two kernel files, auckland.dll and KDKHal.dll, are all you need.

=== Auckland.dll ===
This is the kernel. It's name is a bit confusing. A .dll extension is used for plugin module like things. But the kernel isn't a plugin, it is an application in that it is able to be started. Then again, Windows 1.0 names its important parts kernel.exe, user.exe and gdi.exe despite the fact they are dlls.

==== Micro kernel ====
This part is what makes it a hybrid kernel is that the design is like a micro kernel in concept, but some servers are in kernel mode. It is a hybrid kernel because there are components that can go in user space. But some may call it a monolithic kernel because nothing in a fresh install is a user component.

It contains:

===== Time services =====
This is the area that contains the time related functions. It only contains 9 exported functions:

<source lang="c">
EXPORT int AklGetTime(POINTER time);
EXPORT int AklSetTime(POINTER &time);
EXPORT int AklGetDifference(POINTER diff);
EXPORT int AklSetDifference(POINTER &diff);
EXPORT int AklGetInterval(POINTER interval);
EXPORT int AklAddTimeIntervalCallback(POINTER callback, PDWORD id);
EXPORT int AklGetTimeIntervalCallback(POINTER &callback, PDWORD id);
EXPORT int AklRemoveTimeIntervalCallback(PDWORD id);
</source>

===== Scheduler =====
This is the area that is responsible for managing thread scheduling. It uses a AKL_SCHDL_X86_THREAD structure along with 2 variables. It has no exports:

<source lang="c">
typedef struct _AKL_SCHDL_X86_THREAD
{
  NWORD greg[AKL_REGNUM];
  X87_REG freg[8];
  OCTUPLE_DWORD yreg[AKL_REGNUM];
  NWORD dreg[AKL_REGNUM];
  NWORD flags;
  NWORD eip;
  WORD sreg[6];
  NWORD cr3;
  NWORD filedisc;
  NWORD exattr;
  PAKL_SCHDL_X86_THREAD next;
  PAKL_SCHDL_X86_THREAD prev;
} AKL_SCHDL_X86_THREAD, *PAKL_SCHDL_X86_THREAD;

LINKER(0) PAKL_SCHDL_X86_THREAD data;
LINKER(0) AKL_MK_MUTEX smutex
</source>

===== Memory manager =====
This part handles memory management. It has allocation functions, reservation functions, state get functions and state change functions. It also has a mutex and special finding functions.

==== The Common namespace ====
This is the most important part of the OS. I can't stress it's importance. It contains all files, registry entries, pipes, windows, shell objects, you name it. Like a UNIX file system, but more complete.
Note: This could be part of the micro kernel, because the page fault handler is implemented here.

===== Types =====
Every object has a type, and every type has an object. The types are:

*Standard Object
**IO Object
***File
***Device
***Driver
***Pipe
**Section
**Directory
**Type

==== The Function Layer ====
This area takes up most of the kernel proper

===== Process and thread manager =====
While the Micro kernel is responsible for actual scheduling, this part is for high level stuff like sleeping, terminating, creating and starting. Loading an Executable is always a big struggle in Auckland. First you call AklNewProcess to create a process, AklNewThread to create a thread, AklLoadModule to load the executable and AklStartThread to start it. The function AklTerminateProcess is just a series of AklTerminateThread calls, an AklFlagThreadForTermination call, an AklFlagProcessForTermination call and a HLT instruction

===== IPC =====
The IPC mechanisms are layered for code reuse.

====== Shared Memory ======
In shared memory, Auckland changes the Paging tables of 1 or both processes to point to the same area. Like this:

Thread1->CR3->PDE->PTE-> Memory <-PTE<-PDE<-CR3<-Thread2(Note, it's thread because one thread structure in a process describes the CR3 as well.

It can also make that area Read-only in 1 or both processes

====== Shared-Copied Memory ======
Shared-Copied Memory is implemented on top of Shared Memory, but the areas are also marked Read-Only and when a page fault occurs at that address, it is copied and marked Writable

====== Pipes ======
Pipes are implemented on top of Shared-Copied Memory. In addition, it will free data already read, so if the sender had caused a copy and the receiver has read it, it gets freed.

====== Messages ======
Messages are packets passed through a pipe associated with the message queue. These messages are used to implement Asynchronous IO. Win32 window messages might be implemented using message queues.

===== Module Loader =====
This part of the kernel loads executable files into memory to save you the trouble. It supports MZ, NE, LE, PE, PE+, ELF, NOP and TCATEXE natively while other formats can be supported by adding System Components. Inherently, These components must be in one of the native formats. Using it, different formats can mix. 

===== System Components =====
System Components are the heart and soul of extensibility in Auckland. So far these are the components that exist:

*AklComSS.exe (Auckland Common Subsystem) Contains the common DOS devices and CON, a symbolic link to the programs console
*AklAOA64.exe (Auckland Auckland On Auckland 64bit compatibility file) Handles 32bit syscalls and 32bit System Component compatibility.
*Drivers
*Executable format loaders (For the module loader).

===== IO Manager =====
Self descriptive name, it contains the file management functions, functions for common IO devices and PNP support and drivers.

The compiled drivers are:
*The PCI driver
*The UCHI driver
*The OCHI driver
*The ECHI driver
*The USB driver
*The PC Card driver

===== Session Manager =====
Ok now this is one important thing. A session is essentially a virtual Common Namespace. It contains Special ISC(Inter-Session Communication) functions for when you need to touch another session. Two sessions are made during start up:

*The special Component Session. Regular Auckland processes can't touch them easily.
*The regular Session. This is where the first user programs get loaded to.

More than one session can exist and be pre-emptive with all processes. Its just a virtual Common Namespace and a boundary. There is a kernel mode session space for session space programs.

===== RT (Runtime) =====
This is the CRT library. Not much to see here.

===== Syscall interface =====
This is the hole from User Mode to Kernel Mode. The Syscall interface is an int 2Ch instruction. The ax register contains the service requested. On entry it does a cli instruction, pushes the edx register, sets edx to the user stack, subtracts 16 from edx, copies 16 bytes from esp+4(not the return address) to edx and does a sti instruction.

The Sysenter instruction might be used instead. The ax register contains the service, the edx register the user stack, cx is our cs value (important in distinguishing between 32 bit and 64 bit in 64 bit) and bx is the ss. The flags are still in the flags register.

In either case, for future compatibly, the high word of eax is 0.

At its heart it dispatches calls by the bits 12-15 of the ax register. The current values are:
*0 = Main services.

The main services dispatches calls even further than that, using bits 4-11. They are:
*Time and memory services (The micro kernel)
*File IO (The IO manager)
*Power (AklComSS.exe)
*Security (Syscall interface)
*IPC (IPC)
*Registry (IO manager)
*Module Loader (Module loader)
*Process (Process and thread manager)
*Threads (Process and thread manager)

In addition, the main services will perform security restrictions. While the main security check is in the function itself, this part prevents the call itself.

=== KDKHal.dll ===

The KDKHal.dll file is the OSes Hardware abstraction layer. There are 6 varieties in 32 bit mode:
*ACPI varieties
**KDKHala.dll for ACPI PIC
**KDKHalam.dll for ACPI APIC
*Legacy varieties
**KDKHal.dll for Legacy PIC
**KDKHalm.dll for Legacy APIC

It contains 4 services:
*Interrupt Controlling services.
*Time services.
*IO services.
*Basic File services (enough to get you booted, but not for Auckland)

==== Interrupts ====
The Hal contains lots of services, from setting up an IDT entry, to calling another processor.

==== Time services ====
This area helps the micro kernel time services work. It really a few calls for getting the interval and setting up your handler.

==== IO services ====
This contains C versions of out dx, eax and in eax, dx. This is very helpful for implementing your services.

==== Basic File services ====
Like read file and write file calls. Nothing more nothing less.

=== AklComSS.exe ===
This is the subsystem of Auckland. Anything the kernel realize on that can change, like the console drawing area, is in here. There are 3 parts:

*Console services (exposed by the CON filename)
*DOS names (like COM1, COM2, LPT1, LPT2, etc)
*Power services (Defers to Power.dll)

Note the extension .exe is used for programs. Another naming disaster.

== User Mode ==
This part is User mode. This is where programs, User components and the API lives. There are no built in User Components but the API is. The API is the CRT library so any C program that uses the CRT can work on Auckland.

=== User Components ===
This is the System Components of User mode. There currently are no User Components, but they will come.

=== Programs ===
There are 2 programs that exists, aklinit.exe and aklshell.exe for startup. Another shell program, skytower.exe, and an archive program, museum.exe exist for use. Other programs are:

*shutdown.exe to shutdown (Not to be confused with shutdown.exe on Windows.
*write.exe for editing
*meow.exe for 100 meows to be printed

=== API ===
The API is in two pieces:

*AklCRT.dll, the CRT
*Akldll.dll, contains basic stubs and User Component interaction techniques

== Other things ==
Here are things that I can't bother categorising

Help on the [[User:Tudoros1/FileSystems/NTFS|NTFS]] file system thats so woefully undocumented
````

<a id="w003286"></a>

## W003286 — User:Tudoros1/FileSystems/NTFS

Bron-ID: W003286
Namespace: User (2)
Revisie: 14146 | Datum: 2012-12-12T08:31:11Z
Bron: https://wiki.osdev.org/User:Tudoros1/FileSystems/NTFS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATudoros1%2FFileSystems%2FNTFS&oldid=14146

````text
NTFS is a complex file system and there are hundreds of stuff undocumented. However, some things should stay undocumented due to mind boggling and life wasting. This will contain the needed info to work with simple NTFS volumes.

NOTE: Some info here is guesswork.
NOTE: This info is incomplete

== The Boot sector ==
Note: This corresponds with the $Boot file, as I will show you later. The fields are:

<source lang="c">
struct NTFS_BOOT
{
  BYTE unused1[3];  //This is a jmp instruction followed by a nop.
  BYTE name[8];   //Contains the string "NTFS    "
  WORD sectorsize;   //Must be 512
  BYTE clustersize;   //Max cluster size 256KB
  BYTE unused2[7];   //FAT specific stuff
  BYTE media;   //Must be 0xF8(fixed disk)
  WORD unused3[7];
  DWORD signature;   //Must be 0x00800080
  DOUBLE_DWORD length;   //Doesn't include the backup sector;
  DOUBLE_DWORD mft;   //Cluster of the $MFT
  DOUBLE_DWORD mftmirr;   //Cluster of the $MFTMirr
  BYTE filereclength;   //If negative, length = 2^(-(filereclength))
  BYTE unused4[3];
  BYTE indexreclength;
  BYTE unused4[3];
  DOUBLE_DWORD serialnum;
}
</source>
== File records ==
Ok, now here is where the fun is, assuming you have parsed correctly you will be here.
The Header looks like this:

<source lang="c">
struct NTFS_FILE_2K
{
  NTFS_RECORD header;    //Common Record Header, id is 'FILE'
  DOUBLE_DWORD unused;   //The $LogFile sequence
  WORD sequence;   //Incremented on every delete, 0 if no checking.
  WORD hardlinkcount;   //Decremented by every hardlink removal and Incremented by every new hardlink
  WORD attroff;   //First Attribute offset
  WORD flags;
  DWORD realsize;
  DWORD allocsize;
  DOUBLE_DWORD base;   //non 0 for extensions
  WORD nextattr;   //The next attribute id to be assigned
}

struct NTFS_RECORD
{
  DWORD id;   //A fourCC code
  WORD upsize;   //Words in the update sequence array
  WORD upoff;   //Offset to Update array
}
</source>

Ok, lets explain what they mean:
;Flags:There are two known flags and 2 unknown flags. The known flags are:
*0x1 = In use.
*0x2 = Directory.

;Real/Allocated size:The real size is the offset after the $End attribute padded to a QWORD boundary. The allocated size is the FILE record length.

;Update sequence array:(Note this applies not only to FILE records, but also INDX, RSTR and RCRD records that we will look at later) Because each record is more than 1 sector, there has to be a way of knowing if something failed halfway. While I assume single sector transfers are atomic, multi sector ones are not. The array is located at the offset specified in the upoff value. This array begins with 1 WORD, the update sequence. It is a cyclic structure that increments with every transfer. It is followed by a list of WORDs, the amount of which is indicated by the upsize value - 1. When writing a record to disk, the update sequence needs to be incremented, the last 2 bytes of every sector copied into the correct place in the list of WORDs (the offset in the list of words is sector_number / 2) and the update sequence is copied into the last 2 bytes of every sector. When reading a record from the disk, the last 2 bytes of every sector is checked. If any aren't equal to the update sequence the record is considered corrupted and you somehow restore it. If they all are, all the values in the list of WORDs is copied to the last 2 bytes of the corresponding sectors in the in memory copy and then you parse as indicated by this guide. Perhaps a code sample will help:
<source lang="c">
BOOL CheckUpdateSequence(NTFS_RECORD *record)
{
  DWORD off = 2;
  WORD sequence;
  if (record->upsize < 3)   //3 because there is a minimum of 2 sectors and there is 1 extra word, the update sequence, 2+1=3
  {
    return FALSE;
  }
  sequence = *((record->upoff) + record);
  for (WORD i = 1; i<(record->upsize); i++, off+2)
  {
    WORD *theoff = ((i * 512) - 2) + record;
    if (*theoff != sequence)
    {
      return FALSE;
    }
    *theoff = *((record->upoff) + off + record);   //Why can't we combine 2 passes into 1
  }
  return TRUE;
}

BOOL UpdateUpdateSequence(NTFS_RECORD *record)
{
  DWORD off = 2;
  WORD sequence;
  if (record->upsize < 3)   //3 because there is a minimum of 2 sectors and there is 1 extra word, the update sequence, 2+1=3
  {
    return FALSE;
  }
  sequence = (*((record->upoff) + record)) + 1;
  for (WORD i = 1; i<(record->upsize); i++, off+2)
  {
    WORD *theoff = ((i * 512) - 2) + record;
    *((record->upoff) + off + record) = *theoff;
    *theoff = sequence;
  }
  *((record->upoff) + record) = sequence;
  return TRUE;
</source>

== Attributes ==
NTFS file records consist of attributes. These are streams of metadata. The Filesystem is very flexible in this sense. Their header is in this format:
<source lang="c">
struct NTFS_ATTR
{
  DWORD type;
  DWORD length;   //Length of the attribute on the record;
  BYTE flags;   //Bit 1 means non-resident
  BYTE namelength;   //0 if unnamed
  WORD nameoffset;   //0 if unnamed
  WORD flags;   //Only bit 14(Encrypted) is meaningful, but I'm not going to disclose encryption(some things are better left undocumented)
  WORD attrid;
}
</source>

Resident attributes have this format:
<source lang="c">
struct NTFS_ATTR_RES
{
  NTFS_ATTR header;
  DWORD length;   //a little redundant
  WORD offset;   //also redundant
  WORD flags;   //Bit 1 means indexed
}
</source>

And non-resident ones have this format:
<source lang="c">
struct NTFS_ATTR_NONRES
{
  NTFS_ATTR header;
  DOUBLE_DWORD lowvcn;   //Used for splitting the
  DOUBLE_DWORD highvcn;   //stream across many attributes
  WORD offset;   //a little redundant
  WORD compunitsize;   //0 for uncompressed streams
  DWORD padding;
  DOUBLE_DWORD allocsize;   //Real size rounded to cluster boundary
  DOUBLE_DWORD realsize;
  DOUBLE_DWORD initsize;   //Normally the allocated size
}
</source>

=== Data runs ===

Every non resident attribute is stored somewhere on the disk. The locations are stored in the record and are called data runs. Every run has 1 byte called a size byte. This byte specifies the size of the VCN and LCN values in the run, in bytes. Bits 0-3 specifies the length of the VCN and bits 4-7 specify the length of the LCN. If both are 0, then this is the last run.
The VCN length can't be 0 unless the LCN length is 0 too. The LCN length, however, can be 0 for special reasons. The VCN and LCN follow the size byte and is interpreted as a signed quantity to add to the current values. If the LCN doesn't exist for a compression unit, the unit isn't stored. If the LCN doesn't exist for part of a compression unit, the unit is compressed.

Maybe an example is needed. Here it is: <br />
0x21 0x10 0x89 0x34 0x11 0x07 0x24 0x01 0x19 0x00

The first byte is 0x21, meaning the VCN is 1 byte and the LCN is 2 bytes. The VCN is 0x10(second byte) and the LCN is 0x3489 (third and fourth bytes). When we add 0x10 to the running VCN (currently 0) we get 0x10 and when we add 0x3289 to the running LCN(also 0) we get 0x3489. <br />
The second runs size byte is 0x11, so both VCN and LCN are 2 bytes. The VCN of the second run is 0x07, which when added to the running VCN(0x10) we get 0x17. Same with the running LCN(0x3489), when we add 0x24 we get 0x34AD. <br />
The third runs size byte is 0x01, meaning the VCN is 1 byte long and there is no LCN. The VCN is 0x19, which when added to the VCN equals 0x30. This run crosses one compression unit, so we split it into 2 runs. The first run has a VCN of 0x09 and the second a VCN of 0x10. The first of the 2, along with the second run, is one compression unit, since it contains a stored area and an unstored area, it is compressed. The second of the two is 1 compression unit, so this is a sparse area, and we return all zeros if someone wants the data here. <br />
The forth runs size byte is 0x00, so there are no more runs
The final outcome is:
*Unit 1
**LCN = 0x3489
*Unit 2 (compressed)
**Compressed Length = 0x7 (we still save that part)
**LCN = 0x34AD
*Unit 3 (non existent)

=== Compression ===

=== Standard attributes ===
There are 15 standard attribute types as of NTFS 5 that came with Windows 2000. Some are found in every file ($STANDARD_INFO), others in a certain type of file ($INDEX_ROOT, $DATA), some in files with encryption ($LOGGED_UTILITY_STREAM) and some only in one file ($VOLUME_NAME). In a typical file these attributes are present:
*$STANDARD_INFO
*$FILE_NAME
*$DATA
*$END

and in a typical directory these attributes are present:
*$STANDARD_INFO
*$FILE_NAME
*$INDEX_ROOT
*$INDEX_ALLOCATION
*$INDEX_BITMAP
*$END

==== $STANDARD_INFO (0x00000010) ====
This is the attribute in every file. It contains all the info FAT would typically store in a directory entry. It's format is:
<source lang="c">
struct NTFS_ATTR_STANDARD_INFO
{
  NT_TIME time[4];   //The times are in this order: Creation, Last Written, Record Changed, Last Read 
  DWORD dos;   //A superset of the dos permissions
  DWORD maxver;   //Typically 0
  DWORD version;   //Typically 0
  DWORD class;   //Never used, perhaps it will be implemented in future verions
  DWORD owner;   //The index into the $O and $Q indexes in the file $Quota
  DWORD secure;   //An index into the $SII index in the file &Secure
  DOUBLE_DWORD quotacharge;   //How much to charge the owners quota
  DOUBLE_DWORD unused;   //The last update sequence in the file $UsnJrnl
}
</source>

;DOS Permissions: This is a bit mask with the following bits:
*(0x0001)Read Only(DOS)
*(0x0002)Hidden(DOS)
*(0x0004)System(DOS)
*(0x0020)Archive(DOS)
*(0x0040)Device(DOS)(?)
*(0x0080)Normal(Windows)
*(0x0100)Temp(Windows)
*(0x0200)Sparse(Windows)
*(0x0400)Reparse(Windows)
*(0x0800)Compressed(Windows)
*(0x1000)Offline(Windows)
*(0x2000)Not content indexed(Windows)(?)
*(0x4000)Encrypted(Windows)(?)


==== $ATTRIBUTE_LIST (0x00000020) ====
This attribute is used for listing all attributes in a files extension records. It is rare to see this. It is used when there are too many attributes for the record. It is ordered by type, name and VCN. The format is:

struct NTFS_ATTR_ATTRIBUTE_LIST_ENTRY
{
  DWORD type;   //The attribute type
  WORD length;   //The length of the entry
  BYTE namelength;   
  BYTE nameoffset;
  DOUBLE_DWORD vcn;   //The lowest vcn exposed by this extent
  DOUBLE_DWORD file;   //The file containing it
  WORD id;   //The attribute id if vcn isn't 0
}
</source>

An example on ordering might be needed:
Lets say I have 2 $DATA attributes and 1 $STANDARD_INFO attribute in an extension file. The $DATA attributes are names Hi and Bye. The ordering is as follows:
*Type: $STANDARD_INFORMATION VCN: 0 Name: "" Id: 1
*Type: $DATA VCN: 0 Name: "Bye" Id: 2
*Type: $DATA VCN: 0 Name: "Hi" Id: 3
*Type: $DATA VCN: 200 Name: "Hi" Id: 0

==== $FILE_NAME (0x00000030) ====
This attribute contains the file name. It also contains other stuff. The format is as follows:

<source lang="c">
struct NTFS_ATTR_FILE_NAME
{
  DOUBLE_DWORD parent;   //The parent directory
  NT_TIME time[4];   //The times are in this order: Creation, Last Written, Record Changed, Last Read 
  DOUBLE_DWORD allocsize;   //Real size rounded     to cluster boundary
  DOUBLE_DWORD realsize;   //size of file
  DWORD dos;   //A superset of the dos permissions
  DWORD unused;
  BYTE namelength;   //Typically 0
  BYTE namespace;   //The file namespace
}
</source>

;DOS Permissions: This is a bit mask with the following bits:
*(0x00000001)Read Only(DOS)
*(0x00000002)Hidden(DOS)
*(0x00000004)System(DOS)
*(0x00000020)Archive(DOS)
*(0x00000040)Device(DOS)(?)
*(0x00000080)Normal(Windows)
*(0x00000100)Temp(Windows)
*(0x00000200)Sparse(Windows)
*(0x00000400)Reparse(Windows)
*(0x00000800)Compressed(Windows)
*(0x00001000)Offline(Windows)
*(0x00002000)Not content indexed(Windows)(?)
*(0x00004000)Encrypted(Windows)(?)
*(0x10000000)Directory(Windows)
*(0x20000000)Index view(Windows)(?)

;File namespace: Every file has a name space. There are 3 namespaces:
*POSIX(0): all characters except NULL and / are allowed.
*Win32(1): a subset of POSIX, also doesn't allow ", *, :, <, >, ?, \ and | and can't end with a . or a space.
*DOS(2): a subset of Win32, also doesn't allow +,',', ;, and = and names must be in 8.3 format.
*Win32 and DOS(3): This is just the DOS namespace, but it tells the compiler to not look for a Win32 name because the Win32 name is the DOS name.

==== $OBJECT_ID (0x00000040) ====
This is a W2K attribute. It contains the record GUID. It is four GUIDs in the order:

*Object ID
*Birth Volume ID(Volume it was created)
*Birth Object ID(First Object ID)
*Domain ID (Unused)

Note: Birth Volume, Birth Object and Domain may not be present.

==== $SECURITY_DESCRIPTOR (0x00000050) ====
This attribute will not be present except for special occasions. Normally and index into the $Secure file is all you need. The format is:

 <source lang="c">
struct NTFS_ATTR_SECURITY_DESCRIPTOR
{
  BYTE revision;   //1 for now
  BYTE padding;
  WORD flags;
  DWORD usersid;   //The offset to the SID of the Owner
  DWORD groupsid;   //The offset to the SID of the Group
  DWORD dacl;   //The offset to the DACL
  DWORD sacl;   //The offset to the SACL
}

struct NTFS_ATTR_SECURITY_DESCRIPTOR_ACL
{
  BYTE revision;
  BYTE padding;
  WORD size;
  WORD acecount;   //Amount of ACEs
  WORD padding;
}

struct NTFS_ATTR_SECURITY_DESCRIPTOR_ACE
{
  BYTE type;
  BYTE aceflags;
  WORD size;
  DWORD access;
}
</source>

;Flags:These flags are defined:
*Bit 0(Owner defaulted) The owner or group is
*Bit 1(Group defaulted) provided by a defaulting mechanism
*Bit 2(DACL present)
*Bit 3(DACL defaulted)
*Bit 4(SACL present)
*Bit 5(SACL defaulted)
*Bit 8(DACL Auto Inherit Req)(?)
*Bit 9(SACL Auto Inherit Req)(?)
*Bit 10(DACL Auto Inherited)(?)
*Bit 11(SACL Auto Inherited)(?)
*Bit 12(DACL Protected)(?)
*Bit 13(SACL Protected)(?)
*Bit 14(RM Control Valid)(?)
*Bit 15(Self relative), All offsets are relative to the offset of the header as opposed to pointers. The form used for on disk files.

;Type: The type of the ACE. These types are defined:
*0x00(Access allowed)
*0x01(Access denied)
*0x02(Audit)(?)

;ACE flags: The flags in an ACE, of the following bits:
*Bit 0 (Inherit ACE)
*Bit 1 (Container Inherit ACE)
*Bit 2 (Don't propagate Inherit ACE)
*Bit 3 (Inherit only ACE)
*Bit 6 (Audit on success)
*Bit 7 (Audit on failure)

;Access: These are a bitmask of the following bits:
*Bit 0 (Read file/directory) Read or list a file
*Bit 1 (Write file/directory) Write a file or add a file
*Bit 2 (Append file/directory) Append a file or add a subdirectory
*Bit 3 (Read EA) Read the EA
*Bit 4 (Write EA) Write the EA
*Bit 5 (Execute file) Execute a file
*Bit 6 (Delete child) Delete a child
*Bit 7 (Read attributes) Read DOS attributes
*Bit 8 (Write attributes) Write DOS attributes
*Bit 16 (Delete)
*Bit 17 (Read Control)
*Bit 18 (Write DAC)
*Bit 19 (Write Owner)
*Bit 20 (Synchronise)
*Bit 24 (Access_System_Security)
*Bit 25 (Maximum allowed)
*Bit 28 (Generic All)
*Bit 29 (Generic Execute)
*Bit 30 (Generic Write)
*Bit 31 (Generic Read)

==== $VOLUME_NAME (0x00000060) ====
Note: only found in the $Volume file.

It is just a Unicode string without a terminating NULL.

==== $VOLUME_INFO (0x00000070) ====
Note: only found in the $Volume file.

The format of this attribute is:
<source lang="c">
struct NTFS_ATTR_VOLUME_INFO
{
  DOUBLE_DWORD unused;
  WORD version;
  WORD flags;
}
</source>

;Flags: Here are the flags in the flags value:
*Bit 0 (Dirty) The chkdsk /F command will be performed on reboot
*Bit 1 (Resize Logfile)(?)
*Bit 2 (Upgrade on mount)(?)
*Bit 3 (Mounted on NT4)(?)
*Bit 4 (Delete USN underway)(?)
*Bit 5 (Repair Object Ids)(?)
*Bit 15 (Modified by chkdsk)(?)

;Version: The version value is major version first(Lower offset). Here are the valid version values:
*1.0 = NT 3.1
*1.1 = NT 3.5
*1.2 = NT 4.0
*3.0 = 2K
*3.1 = XP, Vista, 7, 8

==== $DATA (0x00000080) ====
This attribute is simple, its the file data. It is paired with the $INDEX_ROOT, $INDEX_ALLOCATION and $BITMAP attributes for directories.

==== $INDEX_ROOT (0x00000090) ====
This attribute is the root of the B+Tree index. In directories it is named $I30 and is an index of files. The format is:

<source lang="c">
struct NTFS_ATTR_INDEX_ROOT
{
  DWORD type;   //0x00000030 for the $I30 index, 0 for everything else, its the indexed attribute
  DWORD collation;   //The sorting method
  DWORD sizeofalloc;   //How large an index record is(typically 4096 bytes, same as the entry in the BPB(except its in bytes, not clusters)
  DWORD clusofalloc;   //How many clusters in a record(typically 1, same as the entry in the BPB
  DWORD firstindex;   //Offset from this value to the first index entry
  DWORD totalsize;    //The size of the header from the offset of firstindex on
  DWORD allocsize;    //Unsure why this exists, probably for adding more entries with less overhead. Usually the same as totalsize
  DWORD large;   //1 if the $INDEX_ALLOCATION and $BITMAP attributes with the same name exist
}
</source>

;Colation: Here are the Collation rules that exist in NTFS:
*0x00: Binary (A variable length value with the MSB first, smallest to largest).
*0x01: Filename (Filename attributes sorted alphabetically, in accordance to the $Upcase file, so no two files have the same name even in different case).
*0x02: Unicode String (Uppercase letters come first).
*0x10: DWORD (A 32 bit unsigned little-endian value, smallest to largest)
*0x11: Security Identifier (Similar sorting to binary, except the first 2 bytes don't count)
*0x12: Security Hash (2 DWORDs, first DWORD is the Hash and its the most significant. The second one is a Security ID(like the one in $STANDARD_INFO) and is the least significant one).
*0x13: ULONGs (UUIDs, smallest to largest).

The Index entries are in this format:

<source lang="c">
struct NTFS_INDEX_ENTRY
{
  union form    //Invalid if last entry
  {
    DOUBLE_DWORD mftref;   //The file record number, this form is only used if index type(specified at the root) is 0x00000030.
    struct w2k    //This form is used with all new indexes in NTFS 5
    {
      WORD dataoffset;    //Offset to the data in the record
      WORD datasize;    //Length of data
      DWORD padding;
    }
    WORD length;    //Length of the entry
    WORD keylength;    //Length of the key
    DWORD flags;    //Bit 1 is the sub node flag, Bit 2 is the last entry flag
  }
}
</source>

;Key:The key immediately follows this header. Its type is specified in the collation rule of the index. The length is specified in the keylength value. Filenames are a type of key.

;Data: Some data is located inside the entry. It follows the key and contains the needed info for this index entry. In the $I30 index, the data that FAT stores in a directory entry is stored in the key with the filename, because back then there was no data (Only form.mftref was used). Now you know why there is so much info in the $FILENAME attribute. NOTE: When the sub-node flag is set, the last 8 bytes is the VCN of the INDX record in the corresponding $INDEX_ALLOCATION attribute(not sure whether it is included in the size of the entry data.

==== $INDEX_ALLOCATION (0x000000A0) ====
This Attribute is a list of index records. The size of them is declared in the corresponding $INDEX_ROOT attribute. The structure is as follows:

<source lang="c">
struct NTFS_FILE_2K
{
  NTFS_RECORD header;    //Common Record Header, id is 'INDX'
  DOUBLE_DWORD unused;   //The $LogFile sequence
  DOUBLE_DWORD vcn;   //The VCN of the record
  DWORD entries;   //offset to the Index Entries relative to this values offset
  DWORD entriessize;   //Size of all the Index Entries plus the entries value
  DWORD allocsize;    //Size of Index Record - offset of the entries value
  DWORD node;    //Set to 1 if this isn't a leaf node(i.e. Has children)
}
</source>

;VCN:I only see one purpose of this value, validating, by comparing this value with the real vcn of the record to see if they match

==== $BITMAP (0x000000B0) ====
This is what the name implies.

This attribute is used with indexes, but it is also used with the $MFT

== Standard files ==
In NTFS, everything is a file. Even important filesystem structures. Here is a list of files:

*Index 0: $MFT
*Index 0: $MFTMirr
*Index 0: $LogFile
*Index 0: $Volume
*Index 0: $AttrDef
*Index 0: .
*Index 0: $Bitmap
*Index 0: $Boot
*Index 0: $BadClus
*Index 0: $Secure
*Index 0: $UpCase
*Index 0: $Extend
*Indices 12-15: Reserved for more standard files
*Indices 16-23: Reserved for extensions of $MFT

Others exist in random locations, like $ObjId, $Quota, $Reparse and $UsnJrnl.
````

<a id="w003287"></a>

## W003287 — Talk:I8042 PS/2 Controller

Bron-ID: W003287
Namespace: Talk (1)
Revisie: 29479 | Datum: 2025-04-19T23:56:34Z
Bron: https://wiki.osdev.org/Talk:I8042_PS/2_Controller
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AI8042_PS%2F2_Controller&oldid=29479

````text
There was considerable confusion on this page regarding PC-XT,AT and PS2 keyboard & controller systems. I've started to make it straight. My English is what it is, feel free to reformulate.

Edited sections 1 and 1.1. Note : I didn't touch the diagrams (just relabelled the 2d one from XT-controller to the correct AT-controller). However IMO there shoudln't be arrows pointing from the KBC to the speaker gate @port 61 - speaker gate control was simply /never/ done by the 80x42 part. This is another piece of confusion going back to the PPI in PC and XT computers.

-- Czernobyl.

You must not do big deletions of article sections unless you move the text into another article, or verify that the information was duplicated somewhere else. If you don't want the IO Port 0x61 information here, you need to recover that text and put it someplace else. [[User:Bewing|Bewing]] 14:48, 27 September 2012 (CDT)



== Edit Request: The function of "0xC2" Command byte ==
Under the section "PS/2 Controller Commands":

The command byte: "0xC2"

Please change "Copy bits 4 to 6 of input port to status bits 4 to 7"

to "Copy bits 4 to 7 of input port to status bits 4 to 7".

--[[User:Iocoder|Iocoder]] 14:39, 29 January 2013 (CST)

== Initialisation Sequence ==

This initialisation sequence described here does not work on my hardware, though it works in QEMU and VBox. First channel Self-Test returns 0x50 and this is neither documented. Success would be 0x00 and errors 0x01, 0x02, 0x03, 0x04 are documented.

[[User:Glauxosdever|Glauxosdever]] 13:30, 16 February 2016 (CST)
````

<a id="w003288"></a>

## W003288 — Talk:Descriptor Cache

Bron-ID: W003288
Namespace: Talk (1)
Revisie: 14095 | Datum: 2012-11-25T20:29:33Z
Bron: https://wiki.osdev.org/Talk:Descriptor_Cache
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ADescriptor_Cache&oldid=14095

````text
== Cleanup ==

The terminology on this page needs a cleanup.

Inside the CPU, a segment register has a visible part (the "value" of it) and a hidden part (base, limit, attributes). Both the visible and hidden part is loaded during a segment register load (either from the GDT/LDT in protected mode, or from default/computed value in real mode). For protected/long mode, there is no reason for the GDT/LDT descriptor to exist except during segment register loads. For real mode the descriptor never existed.

A cache is something that keeps a copy of something that exists elsewhere; where modifying the actual thing causes the cache to be invalidated/updated to reflect those changes. The hidden part of segment registers are not a cache because they do not necessarily hold a copy of information that exists elsewhere and will not be invalidated or updated when the original information changes.

Essentially what I'm saying is that the "descriptor caches" are not caches (and are much more like registers), and therefore should not be called "descriptor caches".

- [[User:Brendan|Brendan]] ''(please remember to sign)''


: Perhaps we should just merge this page into [[Segmentation]] or something similar instead?  The only thing which links here is [[Unreal mode]] which already explains the way the segment registers are used in unreal mode anyway (although we would also have to update that as its using similar terminology).  [[User:Jnc100|John]] 15:51, 16 November 2012 (CST)


: I'll bounce back the terminology use: caches are about storing copies in places that are faster to access. They are not at all required to be coherent, the TLB isn't after all, and having caching enabled on MMIO/device ram makes that part of it just as incoherent as well. Specifically, the terminology in question is actually used in the official manual. From Intel 3A §3.4.3:
:: '' Every segment register has a “visible” part and a “hidden” part. (The hidden part is sometimes referred to as a'' '''“descriptor cache”''' ''or a “shadow register.”)''
: I also found some diagrams around the manual about the mechanics of segmentation that display a lookup in the GDT in cases where the hidden part is actually used. Not being aware of the cache-like mechanism has yielded several discussions on the forums where people claim a change in the GDT would take immediate effect - with interesting consequences when the situation is ridiculed by asking those people what would happen after one sets CR0.PE. At any rate, the page exists to deal with that specific misunderstanding among newbies and oldtimers alike, and most references here are coming from forum replies. (As a consquence, keep those links working if you do decide to change something)
: - [[User:Combuster|Combuster]] 14:26, 25 November 2012 (CST)
````

<a id="w003290"></a>

## W003290 — MSR

Bron-ID: W003290
Namespace: Main (0)
Revisie: 14088 | Datum: 2012-11-21T20:17:22Z
Bron: https://wiki.osdev.org/MSR
Revisie-URL: https://wiki.osdev.org/index.php?title=MSR&oldid=14088
Doorverwijzing: Model Specific Registers

````text
#REDIRECT [[Model Specific Registers]]
````

<a id="w003291"></a>

## W003291 — User:Henriavelabarbe

Bron-ID: W003291
Namespace: User (2)
Revisie: 14112 | Datum: 2012-11-27T19:27:14Z
Bron: https://wiki.osdev.org/User:Henriavelabarbe
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AHenriavelabarbe&oldid=14112

````text
[[Henriavelabarbe:D barebone with ldc2]]
````

<a id="w003292"></a>

## W003292 — D Bare Bones With LDC2

Bron-ID: W003292
Namespace: Main (0)
Revisie: 26640 | Datum: 2021-11-02T00:27:19Z
Bron: https://wiki.osdev.org/D_Bare_Bones_With_LDC2
Revisie-URL: https://wiki.osdev.org/index.php?title=D_Bare_Bones_With_LDC2&oldid=26640

````text
{{In_Progress}}
{{TutorialExplain}}
{{BeginnersWarning}}
{{Rating|2}}
==Requirements==
You'll need LDC2, Yasm and ld to compile the kernel.

GRUB, mkisofs and QEMU could be used to test it.

==References==
Most sources and information come from osdev wiki and Xomb wiki.

http://wiki.xomb.org/index.php?title=XOmB_Bare_Bones
==Instructions to build the kernel==
<source lang="bash">
yasm -o boot.o boot.s -felf64
yasm -o load.o load.s -felf64
ldc2 -c -nodefaultlib -code-model=large -of=kmain.o kmain.d
ld -nostdlib -nodefaultlibs -b elf64-x86-64 -T linker.ld -o kernel.bin boot.o load.o kmain.o
</source>
Build iso and test using QEMU
<source lang="bash">
mkisofs -R -b boot/grub/stage2_eltorito -no-emul-boot -boot-load-size 16 \
 -boot-info-table -input-charset UTF-8 -o xomb.iso ./iso
qemu-system-x86_64 -cdrom xomb.iso
</source>

==boot.s==

<source lang="asm">
; boot.s

; entry is from bootloader

section .text
bits 32

%include "defines.mac"

; externs given by the linker script
extern _edata
extern _end

; extern to the load.s
extern start64
extern stack

; define the starting point for this module
global start
global _start
start:
_start:

	; Stash values for multiboot we won't touch until 64 bit mode
	mov esi, ebx
	mov edi, eax

	jmp start32

	; the multiboot header needs to be aligned at
	; a 32 bit boundary
	align 4

	multiboot_header:

	dd MULTIBOOT_HEADER_MAGIC
	dd MULTIBOOT_HEADER_FLAGS
	dd -(MULTIBOOT_HEADER_MAGIC + MULTIBOOT_HEADER_FLAGS)
	dd multiboot_header
	dd _start
	dd (_edata-KERNEL_VMA_BASE)
	dd (_end-KERNEL_VMA_BASE)
	dd _start

; the 32 bit entry
global start32
start32:

	; disable interrupts
	cli

	; enable 64-bit page translation table entries
	; by setting CR4.PAE = 1.
	;
	; Paging is not enabled until long mode.
	mov eax, cr4
	bts eax, 5
	mov cr4, eax

	; Create long mode page table and init CR3 to
	; point to the base of the PML4 page table
	mov eax, pml4_base
	mov cr3, eax

	; Enable Long mode and SYSCALL / SYSRET instructions
	mov ecx, 0xC0000080
	rdmsr
	bts eax, 8
	bts eax, 0
	wrmsr

	; Load the 32 bit GDT
	lgdt [pGDT32]

	; Load the 32 bit IDT
	; lidt [pIDT32]

	; establish a stack for 32 bit code
	mov esp, (stack-KERNEL_VMA_BASE) + STACK_SIZE

	; enable paging to activate long mode
	mov eax, cr0
	bts eax, 31
	mov cr0, eax

	jmp CS_KERNEL:(start64-KERNEL_VMA_BASE)

bits 64
code64Jump:
	jmp (start64-KERNEL_VMA_BASE)




; Data Structures Follow
bits 32

; 32 bit gdt

align 4096

pGDT32:
	dw GDT_END - GDT_TABLE - 1
	dq GDT_TABLE - KERNEL_VMA_BASE

GDT_TABLE:

	dq 0x0000000000000000	; Null Descriptor
	dq 0x00cf9a000000ffff	; CS_KERNEL32
	dq 0x00af9a000000ffff,0	; CS_KERNEL
	dq 0x00af93000000ffff,0	; DS_KERNEL
	dq 0x00affa000000ffff,0	; CS_USER
	dq 0x00aff3000000ffff,0	; DS_USER
	dq 0,0					;
	dq 0,0					;
	dq 0,0					;
	dq 0,0					;

	dq 0,0,0				; Three TLS descriptors
	dq 0x0000f40000000000	;

GDT_END:




; Temporary page tables

; These assume linking to 0xFFFF800000000000
align 4096
pml4_base:
	dq (pml3_base + 0x7)
	times 255 dq 0
	dq (pml3_base + 0x7)
	times 255 dq 0

align 4096
pml3_base:
	dq (pml2_base + 0x7)
	times 511 dq 0

align 4096
pml2_base:
	%assign i 0
	%rep 25
	dq (pml1_base + i + 0x7)
	%assign i i+4096
	%endrep

	times (512-25) dq 0

align 4096
; 15 tables are described here
; this maps 40 MB from address 0x0
; to an identity mapping
pml1_base:
	%assign i 0
	%rep 512*25
	dq (i << 12) | 0x087
	%assign i i+1
	%endrep
</source>

==load.s==

<source lang="asm">
; load.s

; entry is from boot.s

bits 64

; Everywhere you see some weird addition logic
; This is to fit the addresses into 32 bit sizes
; Note, they will sign extend!

section .text

; include useful definitions
%include "defines.mac"

; extern to kmain.d
extern kmain

global start64

start64:

	; Initialize the 64 bit stack pointer.
	mov rsp, ((stack - KERNEL_VMA_BASE) + STACK_SIZE)

	; Set up the stack for the return.
	push CS_KERNEL

	; RAX - the address to return to
	mov rax, KERNEL_VMA_BASE >> 32
	shl rax, 32
	or rax, long_entry - (KERNEL_VMA_BASE & 0xffffffff00000000)
	push rax

	; Go into canonical higher half
	; It uses a trick to update the program counter
	;   across a 64 bit address space
	ret

long_entry:

	; From here on out, we are running instructions
	; within the higher half (0xffffffff80000000 ... )

	; We can safely upmap the lower half, we do not
	; need an identity mapping of this region

	; set up a 64 bit virtual stack
	mov rax, KERNEL_VMA_BASE >> 32
	shl rax, 32
	or rax, stack - (KERNEL_VMA_BASE & 0xffffffff00000000)
	mov rsp, rax

	; set cpu flags
	push 0
	lss eax, [rsp]
	popf

	; set the input/output permission level to 3
	; it will allow all access

	pushf
	pop rax
	or rax, 0x3000
	push rax
	popf

	; update the multiboot struct to point to a
	; virtual address
	add rsi, (KERNEL_VMA_BASE & 0xffffffff)

	; push the parameters (just in case)
	push rsi
	push rdi

	; call kmain
	call kmain



	; we should not get here

haltloop:

	hlt
	jmp haltloop
	nop
	nop
	nop



; stack space
global stack
align 4096

stack:
	%rep STACK_SIZE
	dd 0
	%endrep

</source>

==defines.mac==
<source lang="asm">
; multiboot definitions
%define MULTIBOOT_HEADER_MAGIC	0x1BADB002
%define MULTIBOOT_HEADER_FLAGS	0x00010003

; where is the kernel?
%define KERNEL_VMA_BASE			0xFFFF800000000000
%define KERNEL_LMA_BASE			0x100000

; the gdt entry to use for the kernel
%define CS_KERNEL				0x10
%define CS_KERNEL32				0x08

; other definitions

%define STACK_SIZE				0x4000


</source>

==kmain.d==

<source lang="d">
module kmaina;
extern(C) void kmain(int bootLoaderID, void *data) {
        int ypos = 0; //Starting points of the cursor
	int xpos = 0;
	const uint COLUMNS = 80; //Screensize
	const uint LINES = 24;
 
	ubyte* vidmem = cast(ubyte*)0xFFFF8000000B8000; //Video memory address
 
	for (int i = 0; i < COLUMNS * LINES * 2; i++) { //Loops through the screen and clears it
			*(vidmem + i) = 0;
	}
 
	*(vidmem + (xpos + ypos * COLUMNS) * 2) = 'D' & 0xFF; //Prints the letter D
	*(vidmem + (xpos + ypos * COLUMNS) * 2 + 1) = 0x07; //Sets the colour for D to be light grey (0x07)
 
	for (;;) { //Loop forever. You can add your kernel logic here
	}
}
extern(C) void* _Dmodule_ref;
</source>

[[Category:Bare bones tutorials|D bare bones]]
[[Category:D]]
````

<a id="w003293"></a>

## W003293 — Henriavelabarbe:D barebone with ldc2

Bron-ID: W003293
Namespace: Main (0)
Revisie: 22661 | Datum: 2018-09-10T00:02:43Z
Bron: https://wiki.osdev.org/Henriavelabarbe:D_barebone_with_ldc2
Revisie-URL: https://wiki.osdev.org/index.php?title=Henriavelabarbe%3AD_barebone_with_ldc2&oldid=22661

````text
==D Barebone using ldc2==
{{In_Progress}}
===Requirements===
You'll need ldc2, yasm and ld to compile the kernel.

Grub, mkisofs and qemu could be used to test it.
===References===
Most sources and information come from osdev wiki and Xomb wiki.

http://wiki.xomb.org/index.php?title=XOmB_Bare_Bones
===Instructions to build the kernel===
<pre>
yasm -o boot.o boot.s -felf64
yasm -o load.o load.s -felf64
ldc2 -c -nodefaultlib -code-model=large -of=kmain.o kmain.d
ld -nostdlib -nodefaultlibs -b elf64-x86-64 -T linker.ld -o kernel.bin boot.o load.o kmain.o
</pre>
Build iso and test using qemu
<pre>
mkisofs -R -b boot/grub/stage2_eltorito -no-emul-boot -boot-load-size 16 \
 -boot-info-table -input-charset UTF-8 -o xomb.iso ./iso
qemu-system-x86_64 -cdrom xomb.iso
</pre>

===boot.s===

<source lang="asm">
; boot.s

; entry is from bootloader

section .text
bits 32

%include "defines.mac"

; externs given by the linker script
extern _edata
extern _end

; extern to the load.s
extern start64
extern stack

; define the starting point for this module
global start
global _start
start:
_start:

	; Stash values for multiboot we won't touch until 64 bit mode
	mov esi, ebx
	mov edi, eax

	jmp start32

	; the multiboot header needs to be aligned at
	; a 32 bit boundary
	align 4

	multiboot_header:

	dd MULTIBOOT_HEADER_MAGIC
	dd MULTIBOOT_HEADER_FLAGS
	dd -(MULTIBOOT_HEADER_MAGIC + MULTIBOOT_HEADER_FLAGS)
	dd multiboot_header
	dd _start
	dd (_edata-KERNEL_VMA_BASE)
	dd (_end-KERNEL_VMA_BASE)
	dd _start

; the 32 bit entry
global start32
start32:

	; disable interrupts
	cli

	; enable 64-bit page translation table entries
	; by setting CR4.PAE = 1.
	;
	; Paging is not enabled until long mode.
	mov eax, cr4
	bts eax, 5
	mov cr4, eax

	; Create long mode page table and init CR3 to
	; point to the base of the PML4 page table
	mov eax, pml4_base
	mov cr3, eax

	; Enable Long mode and SYSCALL / SYSRET instructions
	mov ecx, 0xC0000080
	rdmsr
	bts eax, 8
	bts eax, 0
	wrmsr

	; Load the 32 bit GDT
	lgdt [pGDT32]

	; Load the 32 bit IDT
	; lidt [pIDT32]

	; establish a stack for 32 bit code
	mov esp, (stack-KERNEL_VMA_BASE) + STACK_SIZE

	; enable paging to activate long mode
	mov eax, cr0
	bts eax, 31
	mov cr0, eax

	jmp CS_KERNEL:(start64-KERNEL_VMA_BASE)

bits 64
code64Jump:
	jmp (start64-KERNEL_VMA_BASE)




; Data Structures Follow
bits 32

; 32 bit gdt

align 4096

pGDT32:
	dw GDT_END - GDT_TABLE - 1
	dq GDT_TABLE - KERNEL_VMA_BASE

GDT_TABLE:

	dq 0x0000000000000000	; Null Descriptor
	dq 0x00cf9a000000ffff	; CS_KERNEL32
	dq 0x00af9a000000ffff,0	; CS_KERNEL
	dq 0x00af93000000ffff,0	; DS_KERNEL
	dq 0x00affa000000ffff,0	; CS_USER
	dq 0x00aff3000000ffff,0	; DS_USER
	dq 0,0					;
	dq 0,0					;
	dq 0,0					;
	dq 0,0					;

	dq 0,0,0				; Three TLS descriptors
	dq 0x0000f40000000000	;

GDT_END:




; Temporary page tables

; These assume linking to 0xFFFF800000000000
align 4096
pml4_base:
	dq (pml3_base + 0x7)
	times 255 dq 0
	dq (pml3_base + 0x7)
	times 255 dq 0

align 4096
pml3_base:
	dq (pml2_base + 0x7)
	times 511 dq 0

align 4096
pml2_base:
	%assign i 0
	%rep 25
	dq (pml1_base + i + 0x7)
	%assign i i+4096
	%endrep

	times (512-25) dq 0

align 4096
; 15 tables are described here
; this maps 40 MB from address 0x0
; to an identity mapping
pml1_base:
	%assign i 0
	%rep 512*25
	dq (i << 12) | 0x087
	%assign i i+1
	%endrep
</source>

===load.s===

<source lang="asm">
; load.s

; entry is from boot.s

bits 64

; Everywhere you see some weird addition logic
; This is to fit the addresses into 32 bit sizes
; Note, they will sign extend!

section .text

; include useful definitions
%include "defines.mac"

; extern to kmain.d
extern kmain

global start64

start64:

	; Initialize the 64 bit stack pointer.
	mov rsp, ((stack - KERNEL_VMA_BASE) + STACK_SIZE)

	; Set up the stack for the return.
	push CS_KERNEL

	; RAX - the address to return to
	mov rax, KERNEL_VMA_BASE >> 32
	shl rax, 32
	or rax, long_entry - (KERNEL_VMA_BASE & 0xffffffff00000000)
	push rax

	; Go into canonical higher half
	; It uses a trick to update the program counter
	;   across a 64 bit address space
	ret

long_entry:

	; From here on out, we are running instructions
	; within the higher half (0xffffffff80000000 ... )

	; We can safely upmap the lower half, we do not
	; need an identity mapping of this region

	; set up a 64 bit virtual stack
	mov rax, KERNEL_VMA_BASE >> 32
	shl rax, 32
	or rax, stack - (KERNEL_VMA_BASE & 0xffffffff00000000)
	mov rsp, rax

	; set cpu flags
	push 0
	lss eax, [rsp]
	popf

	; set the input/output permission level to 3
	; it will allow all access

	pushf
	pop rax
	or rax, 0x3000
	push rax
	popf

	; update the multiboot struct to point to a
	; virtual address
	add rsi, (KERNEL_VMA_BASE & 0xffffffff)

	; push the parameters (just in case)
	push rsi
	push rdi

	; call kmain
	call kmain



	; we should not get here

haltloop:

	hlt
	jmp haltloop
	nop
	nop
	nop



; stack space
global stack
align 4096

stack:
	%rep STACK_SIZE
	dd 0
	%endrep

</source>

===defines.mac===
<source lang="asm">
; multiboot definitions
%define MULTIBOOT_HEADER_MAGIC	0x1BADB002
%define MULTIBOOT_HEADER_FLAGS	0x00010003

; where is the kernel?
%define KERNEL_VMA_BASE			0xFFFF800000000000
%define KERNEL_LMA_BASE			0x100000

; the gdt entry to use for the kernel
%define CS_KERNEL				0x10
%define CS_KERNEL32				0x08

; other definitions

%define STACK_SIZE				0x4000


</source>

===kmain.d===

<source lang="d">
module kmaina;
extern(C) void kmain(int bootLoaderID, void *data) {
        int ypos = 0; //Starting points of the cursor
	int xpos = 0;
	const uint COLUMNS = 80; //Screensize
	const uint LINES = 24;
 
	ubyte* vidmem = cast(ubyte*)0xFFFF8000000B8000; //Video memory address
 
	for (int i = 0; i < COLUMNS * LINES * 2; i++) { //Loops through the screen and clears it
			*(vidmem + i) = 0;
	}
 
	*(vidmem + (xpos + ypos * COLUMNS) * 2) = 'D' & 0xFF; //Prints the letter D
	*(vidmem + (xpos + ypos * COLUMNS) * 2 + 1) = 0x07; //Sets the colour for D to be light grey (0x07)
 
	for (;;) { //Loop forever. You can add your kernel logic here
	}
}
extern(C) void* _Dmodule_ref;
</source>
````

<a id="w003294"></a>

## W003294 — File:ISBN 0321637739.jpg

Bron-ID: W003294
Namespace: File (6)
Revisie: 14116 | Datum: 2012-11-29T12:19:58Z
Bron: https://wiki.osdev.org/File:ISBN_0321637739.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0321637739.jpg&oldid=14116

````text
APUE, 3rd ed.
````

<a id="w003295"></a>

## W003295 — File:ISBN 0201633612.jpg

Bron-ID: W003295
Namespace: File (6)
Revisie: 14117 | Datum: 2012-11-29T12:33:39Z
Bron: https://wiki.osdev.org/File:ISBN_0201633612.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0201633612.jpg&oldid=14117

````text
Design Patterns
````

<a id="w003296"></a>

## W003296 — File:ISBN 1118063333.jpg

Bron-ID: W003296
Namespace: File (6)
Revisie: 14119 | Datum: 2012-11-30T16:40:17Z
Bron: https://wiki.osdev.org/File:ISBN_1118063333.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_1118063333.jpg&oldid=14119

````text
OS Concepts, 9th ed.
````

<a id="w003297"></a>

## W003297 — Floppy

Bron-ID: W003297
Namespace: Main (0)
Revisie: 14123 | Datum: 2012-12-02T20:42:31Z
Bron: https://wiki.osdev.org/Floppy
Revisie-URL: https://wiki.osdev.org/index.php?title=Floppy&oldid=14123
Doorverwijzing: Floppy Disk Controller

````text
#REDIRECT [[Floppy Disk Controller]]
````

<a id="w003298"></a>

## W003298 — User:Ranok

Bron-ID: W003298
Namespace: User (2)
Revisie: 14131 | Datum: 2012-12-08T01:10:12Z
Bron: https://wiki.osdev.org/User:Ranok
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ARanok&oldid=14131

````text
Hacker, biking, hiker, skier

Likes: 
* Playing in VMX root mode
* Exploring x86
* Writing kernel code (C & ASM)

Dislikes:
* Staying up late
* Black coffee

Come find me: ranok in #osdev on Freenode
````

<a id="w003299"></a>

## W003299 — Creating a C Library

Bron-ID: W003299
Namespace: Main (0)
Revisie: 30272 | Datum: 2026-03-16T17:49:42Z
Bron: https://wiki.osdev.org/Creating_a_C_Library
Revisie-URL: https://wiki.osdev.org/index.php?title=Creating_a_C_Library&oldid=30272

````text
{{Rating|3}}

This tutorial will discuss implementing your own Standard C Library (libc). While implementing a minimal subset for kernel use is easy, it is considerably more work to implement sufficient functionality to port third party programs. You may wish to save yourself the effort and port an [[C_Library|existing C library]]. Before going ahead, you may wish to create a [[OS_Specific_Toolchain|OS Specific Toolchain]], which will allow us more control of the compiler. Note that this tutorial is somewhat ELF-specific, although the relevant details can be adapted to other platforms. This tutorial also contains information on how to adapt your GCC cross-toolchain to your operating system.

== Building ==

Now that you have an OS-specific freestanding toolchain that can build your kernel, there is a problem of bootstrapping. To build an OS-specific toolchain that supports a hosted environment, you need the headers of your standard C library installed. To build your libc you need a compiler that supports a hosted environment (building a user-space libc as in a freestanding environment is a logical mistake). The solution is to create a make target that installs your C library and kernel headers into your sysroot's include directory without needing a compiler. Then you can simply build your cross-compiler with --with-sysroot="$SYSROOT" (and not giving the --without-headers option), and you should get a cross-compiler that offers a freestanding environmment for your kernel, and a hosted environment for your user-space. Note that [[libgcc]] may depend on a few things from your C library: if you get errors during building libgcc, then simply add the declarations to your header files. You will get undeclared symbols if GCC deems it appropriate to call a libgcc function that needs a libc symbol, so it is a good idea to implement what libgcc needs early on. Note that [[libgcc]] is <em>not</em> optional, and gcc <em>will</em> emit calls to it if it thinks it is a good idea.

Then you can simply build your C library source files using:

<syntaxhighlight lang="bash">
x86_64-myos-gcc -c strfoo.c -o strfoo.o
x86_64-myos-as x86_64/crt0.s -o x86_64/crt0.o
x86_64-myos-ar rcs libc.a strfoo.o x86_64/crt0.o
</syntaxhighlight>

Note that unlike the kernel, you don't need to add extra special options that disable standard include directories and libraries. After all, this is the C library meant for user-space. You don't need to add -I options to the compiler if you already have installed your libc and kernel headers into your sysroot. Otherwise, simply add the appropriate -I options to the above commands. It may be useful to use the same libc for both user-space and the kernel, in that case don't make the mistake of thinking that user-space-libc and kernel-libcs are the same thing. First of all many user-space things don't make sense (or even work) in the kernel. Secondly, a kernel-usable libc must be built differently than the user-space libc because it is <em>freestanding</em>. A kernel libc must have all the special options that a kernel binary is passed. For instance, don't forget to add -mno-red-zone on x86_64 to both your kernel and libc, or interrupts may corrupt the stack.

== Program Initialization ==
:''See also: [[Calling Global Constructors]]''

The first and most important thing to implement in a C library is the _start function, to which control is passed from your program loader. Its task is to initialize and run the process. Normally this is done by initializing the C library (if needed), then calling the global constructors, and finally calling exit(main(argc, argv)). You can change the name of the default program entry point by adding ENTRY=_my_start_name in your OS-specific binutils emulparams script (binutils/ld/emulparams). You can change which start files are used by modifying gcc/gcc/config/myos.h in your OS-specific GCC. The macros STARTFILE_SPEC and ENDFILE_SPEC define the object files to use in the GCC spec language. See gcc/gcc/config/gnu-user.h for examples on how to use this. If you decide to use the conventional (GNU-like) names and semantics for these initialization files, the following information applies:

=== crt0.o ===
The crt0.o object file will contain the _start function that initializes the process and calls exit(main(argc, argv)). This file is normally written in assembly, though depending on how your program loader invokes _start, you may be able to write it in C. Note that the SysV ABIs have some opinions on how _start is invoked and the stack contents. Adhering to this standard may help porting third party software to your OS.

Below is a simple implementation of crt0.s for x86_64. It assumes that the program loader has put *argv and *envp on the stack, and that %rdi contains argc, %rsi contains argv, %rdx contains envc, and %rcx contains envp
<syntaxhighlight lang="asm">
.section .text

.global _start
_start:
	# Set up end of the stack frame linked list.
	movq $0, %rbp
	pushq %rbp # rip=0
	pushq %rbp # rbp=0
	movq %rsp, %rbp

	# We need those in a moment when we call main.
	pushq %rsi
	pushq %rdi

	# Prepare signals, memory allocation, stdio and such.
	call initialize_standard_library

	# Run the global constructors.
	call _init

	# Restore argc and argv.
	popq %rdi
	popq %rsi

	# Run main
	call main

	# Terminate the process with the exit code.
	movl %eax, %edi
	call exit
.size _start, . - _start
</syntaxhighlight>

This implementation is careful to set up the end of the stack frame linked list. If you compile your files without optimization or you use -fno-omit-frame-pointer, then each function adds itself to this linked list. This is very useful if you wish to add calltracing support. In that case, you'll need to know when you have reached the end, which is why we add an explicit zero in the above code..

Next the _start function calls initialize_standard_library. Note how the program loader register usage nicely fits the x86_64 SysV calling convention, and how the initialize_standard_library(int argc, char* argv[], int envc, char* envp[]) function accepts the same arguments as _start. We save the argc and argv variables, because we'll need them in a moment when we call main. How initialize_standard_library is implemented is up to you, but normally it sets environ to envp, sets up stdin, stdout, and stderr, the heap, and whatever needs to be done.

Programming languages such as C++ offer a feature called global constructors. This allows code to be executed before the main function is called, but after the core standard library has been initialized. Note how GCC allows C programs to use the same feature using the function attribute ((constructor)). Some third party software, such as binutils, silently relies on global constructor functions to be called and otherwise malfunction, even if the program compiled seemingly correctly. To call these global constructors, we call the traditionally-named _init function, which is a piece of magic we'll set up in a moment.

Finally we restore the %rdi and %rsi registers and call main, and use its return value as argument to exit. The _exit function must then call the _fini function, which calls the global destructor functions (as opposed to _init that calls the constructors).

=== crtbegin.o, crtend.o, crti.o, and crtn.o ===

These special object files are provided by your OS-specific compiler. The compiler is told to make these files in gcc/gcc/config.gcc by extra_parts="crtbegin.o crtend.o". GCC internally maintains tables of global constructor/destructor functions. However, these tables are set up in a manner such that you cannot directly link to them yourself. Rather, the files crtbegin.o and crtend.o contains the instructions that uses these tables and calls them as requested. The bad news, however, is that crtbegin.o and crtend.o offer no symbols we can call. Instead, GCC expects us provide these symbols ourselves using some linker tricks. The STARTFILE_SPEC and ENDFILE_SPEC macros tell the compiler which files must be linked in before anything else, and which files must be linked in after anything else. The idea is that files are linked in this order: crt0.o, crti.o, crtbegin.o, your-program-foo.o, your-program-bar.o, crtend.o, crtn.o.

The crtbegin.o and crtend.o files contain the necessary instructions that call global constructors in the .init section, and the instructions that call the global destructors in the .fini section. GCC expects us to put the header of the _init function in crti.o's .init section and the footer of the _init function in crtn.o.'s .init section. The same applies to the _fini function and the .fini section. Using that, we can now implement the _init and _fini functions in crti.o and crtn.o.

Hence an crti.s implementation will simply be (x86_64):
<syntaxhighlight lang="asm">
.section .init
.global _init
_init:
   push %rbp
   movq %rsp, %rbp
   /* gcc will nicely put the contents of crtbegin.o's .init section here. */

.section .fini
.global _fini
_fini:
   push %rbp
   movq %rsp, %rbp
   /* gcc will nicely put the contents of crtbegin.o's .fini section here. */
</syntaxhighlight>

and a simple implementation of crtn.s will be (x86_64):

<syntaxhighlight lang="asm">
.section .init
   /* gcc will nicely put the contents of crtend.o's .init section here. */
   popq %rbp
   ret

.section .fini
   /* gcc will nicely put the contents of crtend.o's .fini section here. */
   popq %rbp
   ret
</syntaxhighlight>

Finally, you simply need to assemble your crt0.o, crti.o, and crtn.o files and install them in your system library directory. Your _start function is now able to set up the standard library, call the global constructors, and call exit(main(argc, argv)). Don't forget to call your _fini function in your exit function, or the global destructors won't be run, leading to subtle bugs.

== Implementing ==
Most of this will be up to you. A good place to start are the mem* and str*-functions, as they are mostly independent of syscalls and other library routines and make unit testing a breeze. clang and gcc, being nice compilers, both provide a set of [https://gcc.gnu.org/onlinedocs/gcc/Other-Builtins.html builtin] functions that can be used to skip implementing some parts. This approach gives you working versions of these functions, with the tradeoff being that this method is compiler-specific to clang and gcc only. While this is probably fine for a kernel, your mileage may vary in userland.

== Standards ==

Creating your own C library has a number of advantages. For instance, you can finally add that useful fopen_http that the GNU libc maintainers rejected. You can also add a bunch of new, useful conversion specifiers to your printf. But before you get too carried away, you should spend a few minutes thinking about compatibility. For instance, if you do add your fopen_http and your user-space programs use it, then you just lost portability back to Linux. While you are still bootstrapping your operating system, it is very valuable to be able to run and test your user-space programs from your host operating system. It allows more rapid development and makes sure that your libc functions do the same as the ones on your host system. Rather than adding these new utility functions to libc, it may well be worth adding them to a new separate library, which is portable and works under your operating system and your host operating system.

Additionally, you should be very careful about changing semantics of existing libc functions or adding new functions to existing headers (namespace pollution). This may very well confuse third-party programs and cause massive porting problems. Whether you do choose to diverge from the conventional path, you should think twice: Is it really worth it? Can you make a new, nicer function and have the old one call the new one? Could your extension conflict with stuff from BSD, Plan 9, or another libc? Has anyone already decided that a given C interface is bad and made a sane replacement? Choosing to copy the interfaces of implementations can certainly help you port software.

Fortunately, you can find plenty of the relevant standards online. For instance, you can look at:
* The C Standard (2011), latest draft is available [http://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf here (PDF)].
* [http://pubs.opengroup.org/onlinepubs/9699919799/ POSIX (2008)].
* GNU libc documentation.

== Include Guards ==

C library headers conventionally use include guards to prevent multiple declarations. It may be wise to use the same format as GNU's libc as some programs (and GCC fixincludes) may erroneously rely on these macros to detect whether a given header has been included. GNU libc's headers usually follow this simple scheme:

<syntaxhighlight lang="c">
#ifndef _STDIO_H
#define _STDIO_H 1

/* ... */

#endif
</syntaxhighlight>

== Repeated Declarations ==

The occasional poor design of the C standard library has lead to the requirement that some declarations must be repeated in multiple header files. Worse, if a header needs the FILE declaration from stdio.h, it is not allowed to include stdio.h because of namespace polution. This leads to a multiple maintaince problem and can be dangerous if you update the declaration in one place and forget to do it in another. If you have a look at GNU libc's stdio.h header, you will discover a maze of #ifdefs and #define __need_FILE preprocessor magic that allows you to #define __need_FILE #include <stdio.h> and then get <em>only</em> the declaration of FILE. While this works, it is very ugly and grows horribly in complexity.

Another solution is to put the FILE in its own header <decl_FILE.H> with its own include guard. This is much cleaner and simpler to understand, but causes extra work for the preprocessor which can get comparably expensive if this is done for a lot of declarations. An alternative solution is to pre-preprocess the header files by writing a simple compiler that automatically finds these <decl_FILE.h> inclusions and inserts the contents into the header file. This trades some build-time-complexity for maintainability and runtime performance.

== C++ Compatibility ==

Traditionally C++ programs can use the C headers, even though such headers are written in C and have C linkage. Unless you specify otherwise, GCC will assume that header files found in your system include directory are written in C and have C linkage. This is done by GCC automatically inserting extern "C" { ... } around all included system headers. However, this may lead to strange linking-failures if you try to use C++ headers from the system include directory. The key solution is to make your C headers explicitly compatible with C++ and tell GCC that you understand C++.

'''For GCC versions < 9''', telling GCC that your headers understand C++ is done by having the following in your OS-specific gcc/gcc/config/myos.h:

<syntaxhighlight lang="c">
/* Don't assume anything about the header files.  */
#undef  NO_IMPLICIT_EXTERN_C
#define NO_IMPLICIT_EXTERN_C	1
</syntaxhighlight>

'''For GCC versions >= 9''', [https://patchwork.ozlabs.org/patch/934478/ this patch] changed the behaviour; they have made <tt>NO_IMPLICIT_EXTERN_C</tt> a so-called "poisoned identifier", so compiling your OS-specific toolchain will fail. If you read the linked patch above, you will realise that they decided to invert the behaviour, such that system headers are assumed to understand C++ by default. So, in theory, you do not need those two lines above at all, and it should Just Work (tm).

If, instead, your system headers indeed ''do not'' support C++ (why?), you can <tt>#define SYSTEM_IMPLICIT_EXTERN_C 1</tt>, as they have done for AIX.

Adding support for C++ in your C header files (such as stdio.h) is very simple. Previously GCC automatically added extern "C" around all headers if compiling C++, but now we'll need to do it ourselves. The key feature is that we don't need to do this on C++-only headers, meaning that C++ headers will now actually work, instead of GCC assuming they have C-linkage. For instance, you can change your stdio.h to be of this form:

<syntaxhighlight lang="c">
#ifndef _STDIO_H
#define _STDIO_H 1

#if defined(__cplusplus)
extern "C" {
#endif

int printf(const char* format, ...);

#if defined(__cplusplus)
} /* extern "C" */
#endif

#endif
</syntaxhighlight>

[[Category:C]]
[[Category:Standard Libraries]]
[[Category:Porting]]
[[Category:Tutorials]]
````

<a id="w003300"></a>

## W003300 — Talk:Creating a C Library

Bron-ID: W003300
Namespace: Talk (1)
Revisie: 26204 | Datum: 2021-07-25T03:41:08Z
Bron: https://wiki.osdev.org/Talk:Creating_a_C_Library
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ACreating_a_C_Library&oldid=26204

````text
The first paragraph claims this article is a tutorial on implementing a standard C library. However, it doesn't talk about standard C at all. If anything, it talks about GCC and the x86 SysV ABI. You should update the introduction to reflect this. --[[User:Love4boobies|Love4boobies]] 21:24, 19 December 2012 (CST)

Hmm, you're right. I simply added bits of information that helped me create my libc and connect my cross toolchain to it. Additions are very welcome. Eventually these SysV ABI details can be refactored into other articles linked from here. --[[User:Sortie|Sortie]] 16:42, 20 December 2012 (CST)

Is there a Intel/NASM syntax version of crt*.s files? --[[User:Leapofazzam|Leapofazzam]] 22:41, 24 July 2021 (CDT)
````

<a id="w003301"></a>

## W003301 — User:Bellezzasolo/BPDI

Bron-ID: W003301
Namespace: User (2)
Revisie: 14182 | Datum: 2012-12-26T23:54:51Z
Bron: https://wiki.osdev.org/User:Bellezzasolo/BPDI
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ABellezzasolo%2FBPDI&oldid=14182

````text
{{stub}}

=BPDI=
==Introduction==
Bellezzasolo's Polymorphic Driver Interface (BPDI) is a driver interface designed for the C++ world.
````

<a id="w003302"></a>

## W003302 — Low Pin Count

Bron-ID: W003302
Namespace: Main (0)
Revisie: 28064 | Datum: 2023-07-09T20:32:06Z
Bron: https://wiki.osdev.org/Low_Pin_Count
Revisie-URL: https://wiki.osdev.org/index.php?title=Low_Pin_Count&oldid=28064

````text
{{stub}}
The Low Pin Count bus, (or LPC bus,) is used to connect low bandwidth devices to the CPU such as [[Serial Ports|serial]] & [[Parallel port|parallel ports]], [[PS/2 Keyboard]], PS/2 Mice, [[Floppy Disk Controller]], and the [[Trusted Platform Module]]


== See Also ==
*[https://www.intel.com/content/dam/www/program/design/us/en/documents/low-pin-count-interface-specification.pdf Intel Low Pin Count Specification]
*[https://en.wikipedia.org/wiki/Low_Pin_Count Low Pin Count on Wikipedia]
[[Category:Buses]]
````

<a id="w003303"></a>

## W003303 — Enhanced Host Controller Interface

Bron-ID: W003303
Namespace: Main (0)
Revisie: 25813 | Datum: 2021-04-21T00:43:14Z
Bron: https://wiki.osdev.org/Enhanced_Host_Controller_Interface
Revisie-URL: https://wiki.osdev.org/index.php?title=Enhanced_Host_Controller_Interface&oldid=25813

````text
The '''Enhanced Host Controller Interface (EHCI)''' is the ''single'' method of interfacing with [[USB]] 2.0. This makes life a lot easier than with USB 1.0, where two incompatible standards ([[OHCI]] and [[UHCI]]) both survived. This is because the USB-IF insisted on a single implementation, produced by Intel.

== Interface ==

On a PC you will normally find the EHCI USB controller on the [[PCI]] bus - in fact it is the only access method specified in the specification. USB 2.0 supports interfacing with USB 1.0 devices. However, EHCI is NOT expected to support them. Instead, you will find an UHCI or OHCI ''companion controller''. Therefore you must implement [[OHCI]] and [[UHCI]] as well. Note there are NOT companions of both types, it is one type or another.

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
* [[XHCI]]

=== External Links ===
* [http://www.intel.com/technology/usb/download/ehci-r10.pdf EHCI Specification]
* [http://download.intel.com/technology/usb/EHCIv1-1_Addendum.pdf Addendum]
* [https://github.com/haiku/haiku/blob/master/src/add-ons/kernel/busses/usb/ehci.cpp Haiku's EHCI implementation]

[[Category:USB]]
[[de:Enhanced_Host_Controller_Interface]]
````

<a id="w003304"></a>

## W003304 — EHCI

Bron-ID: W003304
Namespace: Main (0)
Revisie: 14218 | Datum: 2013-01-10T12:33:12Z
Bron: https://wiki.osdev.org/EHCI
Revisie-URL: https://wiki.osdev.org/index.php?title=EHCI&oldid=14218
Doorverwijzing: Enhanced Host Controller Interface

````text
#REDIRECT [[Enhanced Host Controller Interface]]
````

<a id="w003305"></a>

## W003305 — Universal Host Controller Interface

Bron-ID: W003305
Namespace: Main (0)
Revisie: 29856 | Datum: 2026-01-11T05:17:33Z
Bron: https://wiki.osdev.org/Universal_Host_Controller_Interface
Revisie-URL: https://wiki.osdev.org/index.php?title=Universal_Host_Controller_Interface&oldid=29856

````text
Universal Host Controller Interface (UHCI) was created by Intel as an implementation of the [[USB]] 1.0 host controller interface.  Along with [[OHCI]], it makes up the USB 1.0 standard.

== Technical Details ==
The UHCI specification defines a set of I/O mapped registers that allow communication between the controller and the operating system.  The base address for these registers can be found by searching the PCI controller for a specific VendorID/DeviceID combination, or for a specific ClassID/SubclassID/Interface combination.  All UHCI [[PCI]] controllers will have a Class ID of 0x0C, a Subclass ID of 0x03, and an Interface value of 0x00.  The PCI Configuration space for this device will contain the I/O port address information in BAR4.

== I/O Registers ==
{| {{wikitable}}
! Offset (Hex)
! Name
! Description
! Length
|-
| 00 || USBCMD || USB Command || 2 bytes
|-
| 02 || USBSTS || USB Status || 2 bytes
|-
| 04 || USBINTR || USB Interrupt Enable || 2 bytes
|-
| 06 || FRNUM || Frame Number || 2 bytes
|-
| 08 || FRBASEADD || Frame List Base Address || 4 bytes
|-
| 0C || SOFMOD|| Start Of Frame Modify || 1 byte
|-
| 10 || PORTSC1 || Port 1 Status/Control || 2 bytes
|-
| 12 || PORTSC2 || Port 2 Status/Control || 2 bytes
|-
|}

=== Command Register ===
{| {{wikitable}}
! Bits
! Name
! Description
|-
| 15-8 || Reserved ||
|-
| 7 || Max Packet Size (MAXP) || 0 = Max Size is 32 bytes
1 = Max Size is 64 bytes
|-
| 6 || Configure Flag (CF)||Set by host driver after it configures the host controller. Has no effect on hardware
|-
| 5 || Software Debug (SWDBG) ||0 = Normal mode
1 = Debug mode (Host controller clears Run bit after each transaction)
|-
| 4 || Force Global Resume (FGR) ||1 = Host Controller sends Global Resume signal on the USB
|-
| 3 || Enter Global Suspend Mode (EGSM) ||1 = Host Controller enters Global Suspend mode, suspends USB transactions
|-
| 2 || Global Reset (GRESET) ||1 = Host Controller sends Global Reset signal on the USB and resets itself
|-
| 1 || Host Controller Reset (HCRESET) ||1 = Host Controller resets its internal state to initial values
This bit will clear after reset is done
|-
| 0 || Run/Stop (RS) || 1 = Host Controller execute Frame List Entries
0 = Host Controller halts execution
|-
|}

=== Status Register ===
These bits are set by the Host Controller to indicate interrupts and other states. You can clear those bits by writing 1 to them.
{| {{wikitable}}
! Bits
! Name
! Description
|-
| 15-6 || Reserved ||
|-
| 5 || HCHalted || Set to 1 by Host Controller after it stops executing the Frame List
|-
| 4 || Host Controller Process Error ||Set to 1 by Host Controller when it encounters a fatal error while processing a TD
|-
| 3 || Host System Error ||Set to 1 by Host Controller when a host system access error occurs, e.g. PCI parity error
|-
| 2 || Resume Detecte ||Set to 1 by Host Controller when it receives a RESUME signal from a USB device
|-
| 1 || USB Error Interrupt ||Set to 1 by Host Controller when a USB transaction results in an error
|-
| 0 || USB Interrupt ||Set to 1 by Host Controller after it completes a USB transaction whose TD has its IOC bit set
|-
|}

=== Interrupt Enable Register ===
If bit is set, it mean that this interrupt will be generated
{| {{wikitable}} cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
! Bits
! Name
! Description
|-
| 15-4 || Reserved ||
|-
| 3 || Short Packet Interrupt Enable || Interrupt when short packet was transferred
|-
| 2 || Interrupt on Complete (IOC) Enable || Interrupt after transfer of packet with IOC
|-
| 1 || Resume Interrupt Enable ||
|-
| 0 || Timeout/CRC Interrupt Enable ||
|-
|}

=== Frame Number ===
Number of the currently processed entry of the Frame List.

=== Frame List Base Address ===
32-bit physical address of Frame List. Address has to be aligned to 4 Kb (first 12 bits are zero). The Frame List must contain 1024 entries.

=== Start Of Frame ===
This port sets timing of frame. Should be 0x40.

=== Port 1/2 Status/Control Registers ===
Some bits are Read Only (RO), others can be Read and Written to (R/W). Bits that are marked R/WC are write-clear.
{| {{wikitable}}
! Bits
! Name
! Access
! Description
|-
| 15-13 || Reserved || ||
|-
| 12 || Suspend || R/W || 1 = Port suspended
0 = Port not suspended
|-
| 11-10 || Reserved || ||
|-
| 9 || Port Reset || R/W || 1 = Port in reset state
0 = Port not in reset state
|-
| 8 || Low Speed Device Attached || RO || 1 = Low Speed Device attached
0 = Full Speed Device attached
|-
| 7 || Reserved || RO || Always set, used to detect number of ports
|-
| 6 || Resume Detect || R/W || 1 = Resume detected on port
0 = No resume detected
|-
| 5-4 || Line Status || RO || Signal line logical levels. Used for fault detection and recovery
|-
| 3 || Port Enable/Disable Change || R/WC || 1 = Bit 2 was changed
0 = No change
|-
| 2 || Port Enabled/Disabled || R/W || 1 = Enable
0 = Disable
|-
| 1 || Connect Status Change || R/WC || 1 = Bit 0 was changed
0 = No change
|-
| 0 || Current Connect Status || RO || 1 = There is a device attached to port
0 = No device attached to port
|-
|}

== Memory structures ==

=== Frame List Entry ===
{| {{wikitable}}
! Bits
! Name
! Description
|-
| 31-4 || Physical Address to Memory Structure ||
|-
| 3-2 || Reserved ||
|-
| 1 || Memory Structure Type ||0 = Transfer Descriptor 1 = Queue Head
|-
| 0 || Frame Enable || 1 = Empty Frame
0 = Frame Is Valid 
|-
|}

=== UHCI Queue Head ===
{| class="wikitable"
|+
!Offset (Hex)
!Name
!Description
|-
|00
|Horizontal Pointer
|Same structure as Frame List Entry
|-
|04
|Vertical Pointer
|Same structure as Frame List Entry
|}

=== UHCI Transfer Descriptor ===
{| {{wikitable}} cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
! Offset (Hex)
! Name
! Description
|-
| 00 || Next Descriptor || See below 
|-
| 04 || Status || See below 
|-
| 08 || Packet Header || See below 
|-
| 12 || Buffer Address || 32-bit address of data buffer 
|-
| 16 || System Use || 128-bit area reserved for use by the system 
|-
|}

==== UHCI Transfer Descriptor Next Descriptor ====
{| class="wikitable"
|+
!Bits
!Name
!Description
|-
|31:4
|Physical Address of Memory Structure
|
|-
|3
|Reserved
|
|-
|2
|Depth First
|Controller will continue execution to next TD pointed by this TD
|-
|1
|Memory Structure Type
|1 = Queue Head
0 = Transfer Descriptor 
|-
|0
|Terminate
|1 = This TD do not point do anything
0 = Pointer is valid 
|}

==== UHCI Transfer Descriptor Status ====
{| {{wikitable}} cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; {{{1}}}"
! Bits
! Name
! Description
|-
| 31-30 || Reserved ||
|-
| 29 || Short Packet Detect || 1 = If SPD, continue execution from horizontal QH pointer
|-
| 28-27 || Error Counter ||
|-
| 26 || Low Speed || 1 = This is transfer to Low speed device
|-
| 25 || Is Isochronous ||
|-
| 24 || Interrupt On Complete ||
|-
| 23 || Active || This packet waits to be transferred by UHCI controller
|-
| 22 || Stalled ||
|-
| 21 || Data Buffer Error ||
|-
| 20 || Babble Detected ||
|-
| 19 || Non-Acknowledged ||
|-
| 18 || Timeout CRC ||
|-
| 17 || Bit Stuff Error ||
|-
| 16-11 || Reserved ||
|-
| 10-0 || Actual Length || Length of transferred bytes-1
|-
|}

==== UHCI Transfer Descriptor Packet Header ====
{| {{wikitable}}
! Bits
! Name
! Description
|-
| 31-21 || Maximum Length || (Length - 1) so 7 for low speed, 63 for full speed
|-
| 20 || Reserved ||
|-
| 19 || Data Toggle ||
|-
| 18-15 || Endpoint ||
|-
| 14-8 || Device ||
|-
| 7-0|| Packet Type || 0x69 = IN, 0xE1 = OUT, 0x2D = SETUP, other values are invalid
|-
|}

== Initalization ==
When you find UHCI controller, you need to initalize it by following steps:

* Disable BIOS Legacy USB Emulation by writing value 0x2000 to PCI register 0xC0
* Set I/O busmastering in PCI
* Reset controller by Host Controller Reset (bit will self-clear) and Global Reset (you need to clear bit after 10ms)
* Allocate memory for Frame List,  fill it with empty Frame Entries (every entry will have bit 0 set) and write it's physical address to FRBASEADD
* Enable interrupts by setting bits in USBINTR
* Find out number of ports on controller. This value is not saved anywhere, so you need to continue to access new port registers, and check if they have bit 7 set and their value is not 0xFFFF. If those conditions are met, it means that you find valid port register.
* Start controller by setting bit 0 and bit 7 (to enable transfers with 64 bytes) in USBCMD

When you initalized controller, you can start periodically check Connection Status Change of ports. If it is set, it means, that device was either connected, or disconnected. So you need to check Connection Status to determine what happened. In both cases, you will need to clear Connection Status Change. You have to do it in separate write beside any other action, so for example, you can not clear Connection Status Change and set Reset of port in same write. Then if there is connected new device, you need to enable it by following sequence:

* Set Reset bit
* Wait 100 ms
* Clear Reset bit
* Wait 50 ms
* Set Device Enable bit
* Wait until this bit becomes set

When Device Enable bit is set, it means that you have successfully enabled device that is in zero address mode, so you can start transfers to it. At first you would want to read USB Device Descriptor, and then Set Address to something other than 0. If Device Enable bit is not set after 100 ms, you can assume that there happened some error.

If you want to disable device (for example because it do not responds to transfers), you simply need to clear Device Enable bit.

== Transfers ==
At beginning of frame, UHCI controller looks at frame number at FRNUM and then looks at Frame Entry of this frame. If it is valid, it then continues to execute transfer structure it points to. If it moves on transfer descriptor, and this transfer descriptor has active bit set, controller will process this descriptor. If it moves on Queue Head, it will continue down by vertical pointer. When it will find first transfer descriptor with active bit, it will transfer it. Then it will look at Depth bit in transfer descriptor. If it is clear, it will move by horizontal pointer in Queue Head, if it is set, it will move to structure pointed to by descriptor. When last transfer descriptor is processed (terminate bit is set), controller continues through horizonal pointer of last loaded Queue Head.

Execution of one frame lasts 1 millisecond. If controller did not went through all memory structures, it will abandon execution of them and move on next frame. After every frame is number in FRNUM incremented, and rolls over after value 1023, because there are 1024 frame entries.

=== Control and bulk transfers ===
You need to create Queue Head which vertical pointer points to first Transfer Descriptor, and then all Transfer Descriptors are connected after one another. Then you need to insert Queue Head to be executed in every frame for fastest transfer. You will probably want interrupt fired after execution of last Transfer Descriptor in list.

Example of control transfer that transfers USB Descriptor with length 18 bytes from low speed device:<syntaxhighlight lang="text">
FRAME ENTRY: 0x10000 | (1 << 1) (Queue Head at 0x10000)

0x10000: QUEUE HEAD
Horizontal Pointer: 0x00000001 (invalid pointer)
Vertical Pointer: 0x20000 (Transfer Descriptor at 0x20000)

0x20000: TRANSFER DESCRIPTOR
Type: Setup
Toggle: 0
Length of transfer: 8 bytes
Pointer: 0x20020 | (1 << 2) (Transfer Descriptor at 0x20020, Depth bit set)

0x20020: TRANSFER DESCRIPTOR
Type: In
Toggle: 1
Length of transfer: 8 bytes
Pointer: 0x20040 | (1 << 2) (Transfer Descriptor at 0x20060, Depth bit set)

0x20040: TRANSFER DESCRIPTOR
Type: In
Toggle: 0
Length of transfer: 8 bytes
Pointer: 0x20060 | (1 << 2) (Transfer Descriptor at 0x20080, Depth bit set)

0x20060: TRANSFER DESCRIPTOR
Type: In
Toggle: 1
Length of transfer: 2 bytes
Pointer: 0x20080 | (1 << 2) (Transfer Descriptor at 0x20080, Depth bit set)

0x20080: TRANSFER DESCRIPTOR
Type: Out
Toggle: 1
Length of transfer: 0 bytes
Pointer: 0x00000001 (invalid pointer)
</syntaxhighlight>

==== Interrupt transfers ====
You need to create Queue Head that points to one Transfer Descriptor. Then you need to insert Queue Head to be executed every X millisecond as endpoint needs. It is okay to request transfer a bit earlier, e.g. endpoint require execution every 10ms, but you ask for it every 8ms. You will probably want interrupt fired after execution of this Transfer Descriptor. Unlike OHCI and EHCI, UHCI controller will not reverse toggle bit after transfer, so when you will reactivate Transfer Descriptor, you will need to do it by yourself.

==== Isochronous transfers ====
Isochronous transfers are organized as list of Transfer Descriptors that are directly pointed to from Frame Entry before any Queue Head is processed. It is because it is mandatory for controller to transfer all isochronous transfers in one frame to assure same data rate every millisecond.

== Frame Memory Structure ==
One way of managing transfers is to have few Queue Heads that will be transferred at 2<sup>X</sup> milliseconds, like Queue Head that will be transferred every millisecond, Queue Head that will be transferred every 2 milliseconds, Queue Head at every 4 milliseconds and so on. If you manage Queue Heads in this way, you can be sure that when any of them is called, in same frame should be also called all Queue Heads with lower millisecond time. So after you create them, you can connect them by horizontal pointers from slowest to fastest:<syntaxhighlight lang="text">
QH 32 ms -> QH 16 ms -> QH 8 ms -> QH 4 ms -> QH 2 ms -> QH 1 ms -> invalid pointer
</syntaxhighlight>All vertical pointers needs to be invalid. Now you can insert those Queue Heads to Frame List. To ensure that every Queue Head is called at right time, you should insert Queue Heads to frames that are divisible by their millisecond number. So for example frame 1 should point to QH 1 ms, frame 4 should point to QH 4 ms, QH 2 ms and QH 1 ms. Because you already connected slower Queue Heads to faster Queue Heads, you can insert slowest Queue Head to Frame list. Here is example how to do this:<syntaxhighlight lang="c">
#define UHCI_QUEUE_HEAD_POINTER (1 << 1)

uint32_t *frame_list = (uint32_t *) uhci_controller_frame_list_base;

for(int i = 0; i < 1024; i++) {
    if((i % 32) == 0) {
        frame_list[i] = (&queue_head_32_ms | UHCI_QUEUE_HEAD_POINTER);
    }
    else if((i % 16) == 0) {
        frame_list[i] = (&queue_head_16_ms | UHCI_QUEUE_HEAD_POINTER);
    }
    else if((i % 8) == 0) {
        frame_list[i] = (&queue_head_8_ms | UHCI_QUEUE_HEAD_POINTER);
    }
    else if((i % 4) == 0) {
        frame_list[i] = (&queue_head_4_ms | UHCI_QUEUE_HEAD_POINTER);
    }
    else if((i % 2) == 0) {
        frame_list[i] = (&queue_head_2_ms | UHCI_QUEUE_HEAD_POINTER);
    }
    else {
        frame_list[i] = (&queue_head_1_ms | UHCI_QUEUE_HEAD_POINTER);
    }
}
</syntaxhighlight>Now when you have Queue Head for transfer, you can easily insert it to transfer structure. For example if you want to insert new queue head to be transferred every 8 milliseconds, you simply do:<syntaxhighlight lang="c">
new_queue_head->horizontal_pointer = queue_head_8_ms->horizontal_pointer;
queue_head_8_ms->horizontal_pointer = (&new_queue_head | UHCI_QUEUE_HEAD_POINTER);
</syntaxhighlight>And when you need to remove Queue Head after transfer is complete, you simply do:<syntaxhighlight lang="c">
// previous_queue_head points to queue head which we want to remove
previous_queue_head->horizontal_pointer = queue_head_to_be_removed->horizontal_pointer;
</syntaxhighlight>When removing Queue Heads you need to be sure that controller is not now processing them. One way is to mark all Transfer Descriptor pointers as invalid, so even if controller is actually processing one of them, it will immediately stop, and move on to next Queue Head.

If you want to transfer isochronous transfer, you need to prepare all transfer descriptors for particular frame, then set value from that Frame Entry to last transfer descriptor pointer, and then insert pointer to first descriptor to Frame Entry. After frame is done, all isochronous descriptors should be transferred, and you will need to prepare new descriptors for new frame. It is good idea to set descriptors for at least two frames ahead, so when your interrupt routine updates new frame, you will not miss actually processed frame.

== See Also ==
===External Links===
* [https://ftp.netbsd.org/pub/NetBSD/misc/blymn/uhci11d.pdf Intel UHCI standard]

[[Category:USB]]
[[de:Universal_Host_Controller_Interface]]
````

<a id="w003306"></a>

## W003306 — Raspberry Pi

Bron-ID: W003306
Namespace: Main (0)
Revisie: 30457 | Datum: 2026-04-06T15:36:18Z
Bron: https://wiki.osdev.org/Raspberry_Pi
Revisie-URL: https://wiki.osdev.org/index.php?title=Raspberry_Pi&oldid=30457

````text
{{In Progress}}
{{FirstPerson}}
{{Sole Editor}}
This is a tutorial on bare-metal [OS] development on the Raspberry Pi. This tutorial is written specifically for the Raspberry Pi Model B Rev 2 because the author has no other hardware to test on. But so far the models are basically identical for the purpose of this tutorial (Rev 1 has 256MB ram, Model A has no ethernet).

This is the authors very first ARM system and we learn as we write without any prior knowledge about ARM. Experience in Linux/Unix ('''very''' important) and C/C++ language ('''incredibly''' important, including how to use inline assembler) is assumed and required. This is not a tutorial about how to build a kernel but a simple intro in how to get started on the RPi.

== Preparations ==

=== Materials ===

You will need a:
* '''Raspberry Pi''', RPi in short.
* [[SD Card]] to boot from.
* A SD Card reader so you can write to the SD Card from your developement system.
* A serial adaptor for the RPi.
* Power from an external Power Supply, USB or the Serial Adaptor.

=== Serial adaptor ===

The RPi has 2 serial ports ([[UART|UARTs]]). This tutorial only concerns itself with UART0, called simply UART or serial port. UART1 is ignored from now on. The basic UART onboard uses a 3.3V TTL and is connected to some of the GPIO pins labeled "P1" on the board. x86 PCs and MACs do use 5V TTL so you need some adaptor to convert the TTL. A '''USB to TTL Serial Cable - Debug/Console Cable for Raspberry Pi''' with separate connectors per lead, like [http://www.adafruit.com/products/954 commercial RPi serial adaptor], is recommended.. Which is then connected to the RPi [[Media:ARM_RaspberryPi_serial.jpg|like this]]. The slightly cheaper [http://www.ebay.com/itm/USB-To-RS232-TTL-PL2303HX-Auto-Converter-Module-Converter-Adapter-5V-3-3V-Output-/350568364250 PL2303HX adapter] was found usable, but seems to be unreliable if connected to an USB port with an extension cable (a USB 2.0 hub might remedy this).

Note: The serial adaptor I use provides both a 0V and 5V lead (black and red) which provide power to the RPi. No extra power supply is needed besides this.

Alternatively, you can use the FTDI chip on an Arduino (or clone thereof). Connect RX on the Arduino to RX on the Pi, TX to TX and GND to GND, then connect the Arduino's reset pin to ground to prevent code in flash from interfering. Due to reset being held low, the Arduino itself is entirely bypassed. If you have a clone that supports 3.3V operation (such as a Seeeduino) then you can enable it to be safe, but this approach works fine with 5V.

=== Testing your hardware/serial port ===

First things first, you're going to want to make sure all your hardware works. Connect your serial adaptor to the RPi and boot up the official Raspian image. The boot process will output to both the serial and the HDMI and will start a getty on the serial.  Set up your serial port, however yours works, and open up minicom. Make sure you have flow control turned off.
Ensure you can run at 115200 baud, 8N1, which is what the RPi uses.

If you get 'Permission Denied' '''do NOT become root!''' This is unnecessary. Instead do:
<syntaxhighlight lang="bash">
sudo adduser <user> dialout
</syntaxhighlight>
This will let your user use serial ports without needing root.

Or do ls -l /dev/ttyS* to find out the group that own the device, then add you into that group under /etc/group (normally the group is uucp)

If you started minicom only after the RPi has booted then simply press '''return''' in minicom so the getty will output a fresh login prompt. Otherwise wait for the boot messages to appear. If you don't get any output then connect the RPi to a monitor to check that it actually boots, check your connections and minicom settings.

=== Building a cross compiler ===

Like me you are probably using a x86 PC as main machine and want to edit and compile the source on that and the RPi is an ARM CPU so you absolutely need a cross compiler. But even if you are developing on an ARM system it is still a good idea to build a cross compiler to avoid accidentally mixing stuff from your developement system with your own kernel. Follow the steps from [[GCC Cross-Compiler]] to build your own cross compiler but use:

<syntaxhighlight lang="bash">
export TARGET=arm-none-eabi
</syntaxhighlight>

Now you are ready to start.

== Tutorials and examples ==
# [https://www.cl.cam.ac.uk/projects/raspberrypi/tutorials/os/ Tutorial in assembler (University of Cambridge)]
# [[Raspberry Pi Bare Bones|Tutorial in C]]
# [[Raspberry Pi Bare Bones Rust|Tutorial in Rust]]
# [https://github.com/dwelch67/raspberrypi Collection of examples and bootloader by dwelch67]
# [https://github.com/bztsrc/raspi3-tutorial Tutorials for AArch64 in C by bzt]
# [https://github.com/s-matyukevich/raspberry-pi-os OS tutorial for the Raspberry Pi by s-matyukevich]
# [[Detecting Raspberry Pi Board]]

== Booting the kernel ==

Do you still have the SD card with the original Raspian image on it from when you where testing the hardware above? Great. So you already have a SD card with a boot partition and the required files. If not then download one of the original Raspberry boot images and copy them to the SD card.

Now mount the first partition from the SD card and look at it:

<pre>
bootcode.bin  fixup.dat     kernel.img            start.elf
cmdline.txt   fixup_cd.dat  kernel_cutdown.img    start_cd.elf
config.txt    issue.txt     kernel_emergency.img
</pre>

When the RPi powers up the ARM CPU is halted and the GPU runs. The GPU loads the bootloader from rom and executes it. That then finds the SD card and loads the bootcode.bin. The bootcode handles the config.txt and cmdline.txt (or does start.elf read that?) and then runs start.elf. start.elf loads the kernel.img at 0x00008000, puts a few opcodes at 0x00000000 and the ATAGS at 0x00000100 and at last the ARM CPU is started. The CPU starts executing at 0x00000000, where it will initialize r0, r1 and r2 and jump to 0x00008000 where the kernel image starts.

So to boot your own kernel simply replace kernel.img with our own, umount, sync, stick the SD card into RPi and turn the power on.

Note: The GPU also initialized the video ouput, detecting the right resolution from the monitor (if hdmi) or from the config.txt and creates a 2x2 pixel framebuffer (red, yellow, blue and cyan pixels) that the hardware scales to fullscreen with color interpolation. So you get rectangle with a nice color fading.

== Boot from serial ==

The RPi boots the kernel directly from an SD card and only from an SD card. There is no other option. While developing this becomes tiresome since one has to constantly swap the SD card from the RPi to a SD card reader and back. Writing the kernel to the SD card over and over also wears out the card. Plus the SD card slot is somewhat fragile; several people have reported that they broke it accidentally. Overall not an ideal solution. So what can we do about that?

I've written a small bootloader named [https://github.com/mrvn/raspbootin Raspbootin] based on the Tutorial in C above that loads the real kernel from the serial port. Raspbootin is acompanied by Raspbootcom ([https://github.com/mrvn/raspbootin same repository]) that acts as a boot server and terminal program. Using the two I only need to reboot my RPi to get it to boot the latest kernel. This makes testing both faster and safer for the hardware.

Raspbootin is completely transparent for your kernel. It preserves the r0, r1 and r2 registers and ATAGs placed into memory by the GPU for your kernel. So whether you boot your kernel directly from an SD card or with Raspbootin via serial port makes no difference to your code.

=== Raspbootin serial protocol ===

You don't have to care about this unless you want to write your own boot server.

The boot protocol for Raspbootin is rather simple. Raspbootin first sends 3 breaks (<code>\x03</code>) over the serial line to signal that it is ready to receive a kernel. It then expects the size of the kernel as <code>uint32_t</code> in little endian byte order. After the size it replies with "<code>OK</code>" if the size is acceptable or "<code>SE</code>" if it is too large for it to handle. After "<code>OK</code>" it expects <code>size</code> many bytes representing the kernel. That's it.

== Parsing ATAGs ==

A good documentation for ATAGs can be found [http://www.simtec.co.uk/products/SWLINUX/files/booting_article.html#appendix_tag_reference here].

[http://web.archive.org/web/20120605001004/http://www.simtec.co.uk/products/SWLINUX/files/booting_article.html cached version from the Wayback Machine]

Note that later Raspberry Pis will pass you a device tree blob instead in R2. ATAGs still can be found at 0x100, if you disable device tree (r2 contains 0x0 in this case) - identifiable because they always start with an ATAG_CORE (0x54410001). In comparison, Device Tree is probably far more useful, but is more complex. A device tree starts with the uint32_t 0xd00dfeed '''(big-endian).''' Note the big endian - this applies to all values. ARM defaults to little endian, so you'll probably want to write some endian routines early!
[https://www.devicetree.org/ Device Tree Specification]

== Framebuffer support ==

On boot the RPi configures a display with a virtual resolution of 2x2 pixel scaled to full screen. Each pixel has a different color and the hardware scaling interpolates the colors to show a nice color fade. So before you do anything first connect a monitor and see to it that you get some output.

For a framebuffer, you need to learn how to
[https://github.com/raspberrypi/firmware/wiki/Accessing-mailboxes access mailboxes]. And then you have to send the GPU some [https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface mail]. Or, you could read about the framebuffer [http://elinux.org/RPi_Framebuffer here],

Start with a single simple querry at first and then build up more complex mails. If you get it working then I recommend just altering the virtual size and color depth and leaving the physical resolution as is. The RPi seems to do a fine job of detecting the monitor and the user can also configure a resolution in the boot config files on the SD card. Best to honor his wishes.

== Interrupts and exceptions ==

By default the exception vector table on ARM starts at 0x0. You can use that but there are better ways. You can set a flag to use a high vector at 0xffff0000 or set the exception vector base address register to point to your own table anywhere (32 byte aligned) you like.

Note: Interrupts are level triggered so you have to clear the source of an interrupt or mask it before returning from interrupt. The ARM CPU in the RPi also supports some extra instructions for storing registers on the stack of a different mode, switching modes and returning from interrupt. Those extensions are nicely described in the ARM arm.

Note: The return address in LR during an interrupt will be 0-8 byte, depending on the type of exception, offset to what it should be and needs to be adjusted before returning. Again look into the ARM arm for which offset applies to which exception.

Implementing and testing the software interrupt first is a good idea since you can trigger it in a controlled way.

When configuring some peripheral to send an interrupt it is a useful thing to have interrupts disabled in the CPSR, enable the interrupt you are interested in (or all) in the 3 interrupt enable registers and then poll the 3 pending registers in a tight loop and output changes. This allows you to see if the peripheral raises an interrupt and (if in doubt) which one. After that the real interrupt handler can be configured and tested. Gives you a nice half way point to test what you have so far.

== Floating point support ==

To be able to use any floating point operations, such as storing or loading floating point numbers, you need to enable the FPU before using it. To do this, you have to enable access to the coprocessor to whoever should be able to use it, and you have to enable the FPU itself.

<syntaxhighlight lang="asm">
# enable FPU in coprocessor enable register - this gives everybody access to both locations of coprocessor.
ldr r0, =(0xF << 20)
mcr p15, 0, r0, c1, c0, 2
</syntaxhighlight>

And then enable the FPU itself:

<syntaxhighlight lang="asm">
# enable FPU in FP exception register
MOV r3, #0x40000000
#  VMSR FPEXC, r3    # assembler bug
.long 0xeee83a10
</syntaxhighlight>

The third line is the actual instruction that you'd want to use, but due to a bug in Binutils 2.23 it does not assemble. The line below it is what it should assemble to, and replaces the opcode. After doing these two, it's possible to use the FPU.

== USB ==

A standalone BSD-licenced USB driver with support for keyboard and mouse is available here: https://github.com/Chadderz121/csud . This driver can be kept stand-alone, by editing the <code>/source/platform.c</code> file to interface the driver with your implementation of <code>malloc()</code> and similar functions, or you can integrate the driver more closely with your operating system.

== External references ==
# [https://www.scss.tcd.ie/~waldroj/3d1/arm_arm.pdf arm_arm.pdf] - General ARM Architecture Reference Manual v6
# [http://infocenter.arm.com/help/topic/com.arm.doc.ddi0301h/DDI0301H_arm1176jzfs_r0p7_trm.pdf DDI0301H_arm1176jzfs_r0p7_trm.pdf] - More specific ARM for the RPi
# [https://github.com/dwelch67/raspberrypi dwelch67 examples] - Basic toolchain + UART stuff
# [http://elinux.org/RPi_Hardware RPi_Hardware] - List of datasheets (and one manual about peripherals on the Broadcom chip)
# [https://github.com/raspberrypi/firmware/wiki GitHub Raspberry Pi firmware wiki] - For mailboxes and video stuff
# [http://www.raspberrypi.org/wp-content/uploads/2012/02/BCM2835-ARM-Peripherals.pdf BCM2835-ARM-Peripherals.pdf] - Datasheet for RPi peripherals
# [http://www.simtec.co.uk/products/SWLINUX/files/booting_article.html Booting ARM Linux] - Describes the generic bootloader interface to the ARM port of Linux which the RPi bootloader emulates
# [http://sourceforge.net/projects/rpiqemuwindows/ RPi Emulator] - A preconfigured QEMU RPi emulation environment for Windows.

[[Category:ARM]]
[[Category:Raspberry Pi]]
````

<a id="w003307"></a>

## W003307 — File:ARM RaspberryPi serial.jpg

Bron-ID: W003307
Namespace: File (6)
Revisie: 14231 | Datum: 2013-01-12T17:29:35Z
Bron: https://wiki.osdev.org/File:ARM_RaspberryPi_serial.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AARM_RaspberryPi_serial.jpg&oldid=14231

````text
Connecting to the serial port on the Raspberry Pi
````

<a id="w003308"></a>

## W003308 — UHCI

Bron-ID: W003308
Namespace: Main (0)
Revisie: 14235 | Datum: 2013-01-12T17:51:31Z
Bron: https://wiki.osdev.org/UHCI
Revisie-URL: https://wiki.osdev.org/index.php?title=UHCI&oldid=14235
Doorverwijzing: Universal Host Controller Interface

````text
#REDIRECT [[Universal Host Controller Interface]]
````

<a id="w003309"></a>

## W003309 — EXtensible Host Controller Interface

Bron-ID: W003309
Namespace: Main (0)
Revisie: 30300 | Datum: 2026-03-16T20:31:16Z
Bron: https://wiki.osdev.org/EXtensible_Host_Controller_Interface
Revisie-URL: https://wiki.osdev.org/index.php?title=EXtensible_Host_Controller_Interface&oldid=30300

````text
{{DISPLAYTITLE:eXtensible Host Controller Interface}}
eXtensible Host Controller Interface (xHCI) defines a register-level description of a Host Controller for Universal Serial bus ([[USB]]), which is capable of interfacing to USB 1.x, 2.0, and 3.0 compatible devices without the use of companion controllers.

== Technical Details ==

The xHCI controller communicates with the operating system using memory mapped registers that can be located by searching the [[PCI]] configuration space for a device with a specific Class ID, Subclass ID, and Interface number.  All xHCI controllers will have a Class ID of 0x0C, a Subclass ID of 0x03, and an Interface value of 0x30.  The configuration space for this device will contain two Base Address Registers: BAR0 and BAR1.  These two 32-bit address fields combine to create a single 64-bit address that points to the base address of the memory mapped registers for the controller.

== Capability Registers ==

The capability registers are located at the address specified by the PCI configuration space.

{| {{wikitable}}
! Offset (Hex)
! Name
! Description
|-
| 00 || CAPLENGTH || Capability Register Length
|-
| 01 || RSVD || Reserved
|-
| 02 || HCIVERSION || Interface Version Number
|-
| 04 || HCSPARAMS1 || Structural Parameters 1
|-
| 08 || HCSPARAMS2 || Structural Parameters 2
|-
| 0C || HCSPARAMS3 || Structural Parameters 3
|-
| 10 || HCCPARAMS1 || Capability Parameters
|-
| 14 || DBOFF || Doorbell Offset
|-
| 18 || RTSOFF || Runtime Registers Space Offset
|-
| 1C || HCCPARMS2 || Capability Parameters 2
|-
|}

== Operational Registers ==

The operational registers are located after the capability registers in memory, and can be found by adding the CAPLENGTH field to the base address specified in the PCI configuration space.

{| {{wikitable}}
! Offset (Hex)
! Name
! Description
|-
| 00 || USBCMD || USB Command
|-
| 04 || USBSTS || USB Status
|-
| 08 || PAGESIZE || Page Size
|-
| 14 || DNCTRL || Device Notification Control
|-
| 18 || CRCR || Command Ring Control
|-
| 30 || DCBAAP || Device Context Base Address Array Pointer
|-
| 38 || CONFIG || Configure
|-
|}

Reading CRCR (or bits of it) provides '0'. Therefore, keep your own track of this address. Bit 0 of CRCR is the Consumer Cycle State (CCS) flag.

== Port Registers ==

At the end of the operational registers ('''at offset 0x400!'''), each port on the root hub is assigned a set of registers.  The number of entries in the port registers table is determined by the MaxPorts value in the HCSPARAMS1 register.

{| {{wikitable}}
! Offset (Hex)
! Name
! Description
|-
| 00 || PORTSC || Port Status and Control
|-
| 04 || PORTPMSC || Port Power Management Status and Control
|-
| 08 || PORTLI || Port Link Info
|-
| 0C || reserved
|-
|}

== Runtime Registers ==

The runtime registers are located after the operational registers in memory, and can be found by adding the RTSOFF field to the base address specified in the PCI configuration space.

{| {{wikitable}}
! Offset (Hex)
! Name
! Description
|-
| 00 || MFINDEX || Microframe Index
|-
| 20 || IR0-1023 || Interrupter Register Sets
|-
|}

Starting at offset 0x20, each interrupter register set defines the event ring memory addresses needed to send and receive events and data to the USB bus.

{| {{wikitable}}
! Offset (Hex)
! Name
! Description
|-
| 00 || IMAN || Interrupter Management
|-
| 04 || IMOD || Interrupter Moderation
|-
| 08 || ERSTSZ || Event Ring Segment Table Size
|-
| 10 || ERSTBA || Event Ring Segment Table Base Address
|-
| 18 || ERDP || Event Ring Dequeue Pointer
|-
|}

== Doorbell Registers ==

The doorbell registers are located after the runtime registers in memory, and can be found by adding the DBOFF field to the base address specified in the PCI configuration space.  The length of the doorbell register table is based on the number of ports specified in the MaxSlots field in the HCSPARAMS1 register above.  Each doorbell register is 32-bits long, and is used to notify the controller that there are pending operations to be performed on a specific device slot.

== Virtual Registers ==

The xHCI specifications support "virtual" controllers that can be used to support multiple virtual machines running on a single physical machine.  These registers must be configured and managed by the VM host, and effectively duplicate the registers above for use by the guest VMs.

== Chipsets with both EHC and xHC ==

Some chipsets such as the Intel Panther Point (<code>8086:1e31</code>) feature both an EHC and xHC. Both controllers share the same set of ports and access to each port is determined by a hardware toggle (0 = EHC, 1 = xHC).

To switch a port to either the EHC or the xHC registers <code>USB3_PSSEN</code> (<code>0xd0</code>) and <code>XUSB2PR</code> (<code>0xd8</code>) must be written.

For example, to switch all available ports to the xHC (make sure the registers are actually present!):

<syntaxhighlight lang="C">
#define USB3_PSSEN 0xd0
#define XUSB2PR    0xd8

char *pci_header = ...;
(u32 *)(pci_header + USB3_PSSEN) = 0xffff_ffff; // Enable SuperSpeed on USB3 ports
(u32 *)(pci_header + XUSB2PR)    = 0xffff_ffff; // Switch over USB2 ports
</syntaxhighlight>

== External Links ==

*[https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf eXtensible Host Controller Interface Specification 1.2]
* Commit <code>69e848c2090aebba5698a1620604c7dccb448684</code> in the Linux source tree
*[https://github.com/haiku/haiku/blob/master/src/add-ons/kernel/busses/usb/xhci.cpp Haiku's XHCI implementation]

[[Category:USB]]
````

<a id="w003310"></a>

## W003310 — XHCI

Bron-ID: W003310
Namespace: Main (0)
Revisie: 14241 | Datum: 2013-01-12T19:50:45Z
Bron: https://wiki.osdev.org/XHCI
Revisie-URL: https://wiki.osdev.org/index.php?title=XHCI&oldid=14241
Doorverwijzing: EXtensible Host Controller Interface

````text
#REDIRECT [[EXtensible Host Controller Interface]]
````

<a id="w003311"></a>

## W003311 — Virtio

Bron-ID: W003311
Namespace: Main (0)
Revisie: 30307 | Datum: 2026-03-17T18:50:51Z
Bron: https://wiki.osdev.org/Virtio
Revisie-URL: https://wiki.osdev.org/index.php?title=Virtio&oldid=30307

````text
VirtIO is a standardized interface which allows virtual machines access to simplified "virtual" devices, such as block devices, network adapters and consoles.  Accessing devices through VirtIO on a guest VM improves performance over more traditional "emulated" devices, as VirtIO devices require only the bare minimum setup and configuration needed to send and receive data, while the host machine handles the majority of the setup and maintenance of the actual physical hardware.

== Technical Details ==

VirtIO devices appear, to the guest VM, to be normal [[PCI]] devices with a specific VendorID and DeviceID.  All VirtIO devices have a Vendor ID of 0x1AF4, and have a DeviceID between 0x1000 and 0x103F.  The type of VirtIO device (Network Adapter, Block Device, etc.) can be determined by the Subsystem ID field in the [[PCI#Configuration_Space|PCI Configuration Space]] for the device.  The currently defined types are:

{| {{wikitable}}
! Subsystem ID
! Name
|-
| 01 || Network Card
|-
| 02 || Block Device
|-
| 03 || Console
|-
| 04 || Entropy Source
|-
| 05 || Memory Ballooning
|-
| 06 || IO Memory
|-
| 07 || RPMSG
|-
| 08 || SCSI Host
|-
| 09 || 9P Transport
|-
| 10 || MAC802.11 WLAN
|-
|}

== I/O Registers ==

Based on the PCI subsystem ID, the I/O mapped registers for the device can be determined.  All devices have a common "header" block of registers:

{| {{wikitable}}
! Offset (Hex)
! Name
|-
| 00 || Device Features
|-
| 04 || Guest Features
|-
| 08 || Queue Address
|-
| 0C || Queue Size
|-
| 0E || Queue Select
|-
| 10 || Queue Notify
|-
| 12 || Device Status
|-
| 13 || ISR Status
|-
|}

The Device Features register is pre-configured by the device, and includes flags to notify the guest VM what features are supported by the device.  The Guest Features register is used by the guest VM to communicate the features that the guest VM driver supports.  This allows both the host and the guest to maintain both backward and forward compatibility.

The Device Status field is used by the guest VM to communicate the current state of the guest VM driver.  The flags in this register designate when the driver has found the device, when the driver has determined that the device is supported, and when the all of the necessary registers have been configured by the guest driver, and communication between the guest and host may begin.

{| {{wikitable}}
! Flags
! Description
|-
| 01 || Device Acknowledged
|-
| 02 || Driver Loaded
|-
| 04 || Driver Ready
|-
| 40 || Device Error
|-
| 80 || Driver Failed
|-
|}

== Device Specific Registers ==

Immediately after the common registers above, any device specific registers are located (at offset 0x14).

=== Network Device Registers ===

{| {{wikitable}}
! Offset (Hex)
! Name
|-
| 14 || MAC Address 1
|-
| 15 || MAC Address 2
|-
| 16 || MAC Address 3
|-
| 17 || MAC Address 4
|-
| 18 || MAC Address 5
|-
| 19 || MAC Address 6
|-
| 1A || Status
|-
|}

=== Block Device Registers ===

{| {{wikitable}}
! Offset (Hex)
! Name
|-
| 14 || Total Sector Count
|-
| 1C || Maximum Segment Size
|-
| 20 || Maximum Segment Count
|-
| 24 || Cylinder Count
|-
| 26 || Head Count
|-
| 27 || Sector Count
|-
| 28 || Block Length
|-
|}

== Configuration ==

All VirtIO devices use one or more ring buffers on the guest machine to communicate with the host machine.  These buffers are added to virtual queues in memory, and each device has a predefined number of queues.  For instance, the VirtIO Network device has 2 mandatory queues (the receive queue and the send queue), and one optional queue (the control queue).  The optional control queue must be supported by both the host and guest (i.e. feature bit must be set in both the Device Features register and the Guest Features register), and allows the guest to control network specific features on the host machine, like promiscuous mode and hardware checksum calculations.

Each queue must be configured by the guest operating system before the device can be enabled.  The guest can determine the necessary memory needed by the queue by setting the Queue Select register to the desired queue index, and then reading the Queue Size register.  Once the virtual queue has been created in memory, its address is written to the Queue Address register.  This value written to this register is the address of the queue divided by 4096, which means that the virtual queue must be aligned on a 4096 byte boundary.

=== Virtual Queue Descriptor ===

<syntaxhighlight lang="c">

struct VirtualQueue
{
  struct Buffers[QueueSize]
  {
    uint64_t Address; // 64-bit address of the buffer on the guest machine.
    uint32_t Length;  // 32-bit length of the buffer.
    uint16_t Flags;   // 1: Next field contains linked buffer index;  2: Buffer is write-only (clear for read-only).
                      // 4: Buffer contains additional buffer addresses.
    uint16_t Next;    // If flag is set, contains index of next buffer in chain.
  }
  
  struct Available
  {
    uint16_t Flags;             // 1: Do not trigger interrupts.
    uint16_t Index;             // Index of the next ring index to be used.  (Last available ring buffer index+1)
    uint16_t [QueueSize] Ring;  // List of available buffer indexes from the Buffers array above.
    uint16_t EventIndex;        // Only used if VIRTIO_F_EVENT_IDX was negotiated
  }
  
  uint8_t[] Padding;  // Reserved
  // 4096 byte alignment
  struct Used
  {
    uint16_t Flags;            // 1: Do not notify device when buffers are added to available ring.
    uint16_t Index;            // Index of the next ring index to be used.  (Last used ring buffer index+1)
    struct Ring[QueueSize]
    {
      uint32_t Index;  // Index of the used buffer in the Buffers array above.
      uint32_t Length; // Total bytes written to buffer.
    }
    uint16_t AvailEvent;       // Only used if VIRTIO_F_EVENT_IDX was negotiated
  }
}

</syntaxhighlight>

== Communication ==

To send data to a VirtIO device, the guest fills a buffer in memory, and adds that buffer to the Buffers array in the Virtual Queue descriptor.  Then, the index of the buffer is written to the next available position in the Available ring buffer, and the Available index field is incremented.  Finally, the guest writes the index of the Virtual Queue to the Queue Notify I/O register, in order to notify the device that the queue has been updated.  Once the buffer has been processed, the device will add the buffer index to the Used ring, and will increment the Used index field.  If interrupts are enabled, the device will also set the low bit of the ISR Status I/O register, and will trigger an interrupt.

To receive data from a VirtIO device, the guest adds an empty buffer to the Buffers array (with the Write-Only flag set), and adds the index of the buffer to the Available ring, and increments the Available index field, and writes the Virtual Queue index to the Queue Notify I/O register.  When the buffer has been filled, the device will write the buffer index to the Used ring and increment the Used index.  If interrupts are enabled, the device will set the low bit of the ISR Status field, and trigger an interrupt.

Once a buffer has been placed in the Used ring, it may be added back to the available ring, or discarded.

=== Network Packets ===

Each ethernet packet placed in a buffer must be immediately preceeded by a VirtIO Network Packet header.

<syntaxhighlight lang="c">

struct PacketHeader
{
  uint8_t Flags;                // Bit 0: Needs checksum; Bit 1: Received packet has valid data;
                                // Bit 2: If VIRTIO_NET_F_RSC_EXT was negotiated, the device processes
                                // duplicated ACK segments, reports number of coalesced TCP segments in ChecksumStart
                                // field and number of duplicated ACK segments in ChecksumOffset field,
                                // and sets bit 2 in Flags(VIRTIO_NET_HDR_F_RSC_INFO) 
  uint8_t SegmentationOffload;  // 0:None 1:TCPv4 3:UDP 4:TCPv6 0x80:ECN
  uint16_t HeaderLength;        // Size of header to be used during segmentation.
  uint16_t SegmentLength;       // Maximum segment size (not including header).
  uint16_t ChecksumStart;       // The position to begin calculating the checksum.
  uint16_t ChecksumOffset;      // The position after ChecksumStart to store the checksum.
  uint16_t BufferCount;         // Used when merging buffers.
}

</syntaxhighlight>

Note: Since the PacketHeader structure is Read-Only, and the incoming Network Packet is Write-Only (for incoming packets), you may need to separate these two areas into separate buffers, linked by the Next field.

=== Block Device Packets ===

Each block device request placed in a buffer must be filled with a VirtIO Block Request record.

<syntaxhighlight lang="c">

struct BlockRequest
{
  uint32_t Type;              // 0: Read; 1: Write; 4: Flush; 11: Discard; 13: Write zeroes
  uint32_t Reserved;
  uint64_t Sector;
  uint8_t Data[];             // Data's size must be a multiple of 512
  uint8_t Status;             // 0: OK; 1: Error; 2: Unsupported
}

</syntaxhighlight>

Note: Since the BlockRequest structure contains both read-only and write-only fields, it may be necessary to split this structure up into multiple buffers.  The Type, Priority and Sector fields should be placed in a Read-Only buffer, and the Data and Status fields should be placed in a Write-Only buffer.  The buffers will need to be linked together using the Next field.  For Write requests, the buffer containing the Data field should be Read-Only, and the Status field buffer should be Write-Only, which would require 3 buffer entries.

== See Also ==
===External Links===
* [https://docs.oasis-open.org/virtio/virtio/v1.1/virtio-v1.1.html VirtIO specification]
* [http://www.dumais.io/index.php?article=aca38a9a2b065b24dfa1dee728062a12 VirtIO Driver Example]
* [https://github.com/torokernel/torokernel/blob/7d6df4c40fa4cc85febd5fd5799404592ffdff53/rtl/drivers/VirtIONet.pas VirtIO Network Driver Example]
* [https://github.com/torokernel/torokernel/blob/7d6df4c40fa4cc85febd5fd5799404592ffdff53/rtl/drivers/VirtIOBlk.pas VirtIO Block Driver Example]
* [https://github.com/torokernel/torokernel/blob/master/rtl/drivers/VirtIOFS.pas VirtIO FS Driver Example]
* [https://github.com/torokernel/torokernel/blob/master/rtl/drivers/VirtIOVSocket.pas VirtIO VSocket Driver Example]
[[Category:Virtualization]]
[[Category:Standards]]
````

<a id="w003312"></a>

## W003312 — Category:Virtualization

Bron-ID: W003312
Namespace: Category (14)
Revisie: 30104 | Datum: 2026-03-03T15:03:21Z
Bron: https://wiki.osdev.org/Category:Virtualization
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AVirtualization&oldid=30104

````text
[[Category:Hardware]]
````

<a id="w003313"></a>

## W003313 — Intel High Definition Audio

Bron-ID: W003313
Namespace: Main (0)
Revisie: 30627 | Datum: 2026-05-26T15:09:57Z
Bron: https://wiki.osdev.org/Intel_High_Definition_Audio
Revisie-URL: https://wiki.osdev.org/index.php?title=Intel_High_Definition_Audio&oldid=30627

````text

{{In Progress}}

Intel High Definition Audio refers to the specification released by Intel for delivering high-definition audio that is capable of playing back more channels at higher quality than previous integrated audio codecs like [[AC97]].

== Identifying HDA on a machine ==

All HDA devices appear on the PCI bus with a specific VendorID and DeviceID.  Many HDA devices have a Vendor ID of 8086 (Intel), and a Device ID such as 2668 or 27D8, but other Vendor IDs are also in use, e.g. Vendor ID 1002 (AMD) and Device ID 4383. If you find any others, please add them into this page. The best way to identify HDA is probably just to look for the Class code (4h) and subclass (3h), but the HDA specification doesn't tell you to do this.

== Overview ==

The HDA specification (link at the bottom of this page) spells out how to set up devices at the two ends of a link and there is no substitute for working from it, but it takes several readings through its 200+ pages before a clear picture eventually begins to emerge from it, so this overview is aimed at making the most frustrating parts clear from the outset.

You can communicate with the HDA device via memory-mapped ports, but much of the action takes place further away in hardware codecs containing many widgets, and the communication link through to them is rather slow. Setting things up at the near end is relatively easy, part of it being done by keyhole surgery through a couple of ordinary ports in the PCI device configuration space, while the rest is done through the memory mapped ports located at an address found at index 10-17h (though the lowest four bits should be taken as zeros). You may not need to change anything in the PCI configuration space at all, but there's a fair bit to do with the memory mapped ports to set up and control several DMA engines. Setting things up at the codec end is much more complicated though as you have to interrogate them to find out what they are and what functionality they offer, and then you have to work out how to set them up correctly to create active paths between devices (speakers and mic.s) and DACs/ADCs. All your communications with the codecs and the many widgets they contain will be done by sending special commands via the link.

Data and commands are sent across the link in frames with strict timings and bit limits, but the work of packaging the different kinds of data into packets to go into these frames is all done for you, so all you need to do is set up a number of buffers. Two of these buffers are called CORB (command output ring buffer) and RIRB (response input ring buffer) - each buffer has a DMA engine dedicated to it which will in one case send commands from the CORB buffer across the link to codecs, and in the other case will write responses from codecs into the RIRB buffer. There will in many implementations also be an immediate command port which allows you to send commands to codecs/widgets and to receive responses from them without going through the CORB and RIRB mechanisms, but this route should not be used at the same time as CORB/RIRB as they may conflict, so it should really be reserved for initial exploration while designing your driver. The purpose of CORB and RIRB is to allow large numbers of these relatively slow communications to take place in the background while the processor goes off to do something else.

There are also buffers and DMA engines dedicated to four input streams and four output streams (or at least, it's four of each in current implementations, but your software ought to check the actual number), each stream needing a descriptor buffer which must contain two or more descriptors (up to 256) which define a list of data buffers used by that stream, and the data buffers which these descriptors define will contain the actual sound samples (or have samples written into them) structured like the content of .wav files (though 20 and 24-bit samples must be padded out with zeros at the LSB end to make them all 32-bits long). The combined length of the sound data buffers can be anything up to 4GiB, so you could set things up to play or record a very long sound file and leave it going all by itself. In reality though, you'll probably work with chunks of memory a just few megabytes in size (or smaller) as one megabyte gives you room for about six seconds of 16-bit stereo data at 41.1KHz. For performance reasons, making the length of these data buffers a multiple of 128 bytes is recommended.

With all these buffers, the DMA engines jump back to the start and carry on running from there infinitely until you stop them, although with CORB there is a register which stores the last valid command which the DMA engine must stop at (after sending that command) and it will only move on again when that register is modified (by you) to enable more commands to be sent. It is the job of your software to collect data from input buffers before they are overwritten on the next lap. The set of sound data buffers defined by descriptors for a single stream collectively comprise a cyclic stream buffer, but it's divided up into chunks defined by descriptors to enable an interrupt to be generated at the end of each chunk (interrupt optional) to help you write new data into an output buffer that's just been sent before the DMA engine returns to that buffer on the next lap, or to copy data out of an input buffer that's just been written by a DMA engine to make way for more data to be written on the next lap.

At the codec end, you will need to start out by interrogating the root node of each of 15 possible codecs. The STATESTS register at offset 0Eh indicates which codec addresses have codecs at the end of them. The verb F00h will then be used with an 8-bit parameter to request information such as vendor ID, device ID and the starting node number and number of nodes for the function groups in the codec. Having found the function groups, you can use the same verb to interrogate them to find out the starting node number and number of nodes of their widgets, and also the type of the function group itself - AFG (audio function group) is the one you want (unless you're looking for a modem). You can then interrogate each widget to ask it what its type is (e.g. output converter (DAC), input converter (ADC), mixer, selector, pin complex, power widget, volume knob). Another verb, F02h, allows you to get a connection list of other widgets in the same function group directly connected to the widget you're interrogating, though you need to use verb F00h first and the parameter at 0Eh to get the connection list length. On the Netbook I program on there is only one codec, one function group (AFG) and 37 widgets (about half of which are vendor defined audio widgets, though many of those don't exist on the codec manufacturer's datasheet as they are just holding places for real widgets used on more advanced sound cards). Don't be surprised if you don't find a volume control widget - the volume can also be controlled by setting different amplifier levels at the input and output controllers.

== Device Registers ==

Base address of these registers can be found in BAR0 in PCI. Full description of these registers can be found in specification, descriptions below do not aim to explain every register or every single bit, only relevant ones.

==== 0x00 (4 bytes) Global Capabilities ====

{| class="wikitable"
! Bits
! Description
|-
| 15:12 || Number of Output streams
|-
| 11:8 || Number of Input streams
|-
| 7:3 || Number of Bidirectional streams
|-
| 0 || 1 = 64 bit addressing is supported
|-
|}

==== 0x02 (1 byte) Minor Version / 0x03 (1 byte) Major Version ====

As names suggests, from these registers you can read version of High Definition Audio specification that is implemented by sound card.

==== 0x08 (2 bytes) Global Control ====

{| class="wikitable"
! Bits
! Description
|-
| 8 || 1 = Unsolicited responses from Codec are allowed
|-
| 0 || 0 = Controller is in reset state 1 = Controller is in operational state
|-
|}

==== 0x20 (4 bytes) Interrupt Control ====

{| class="wikitable"
! Bits
! Description
|-
| 31 || 0 = All interrupts are disabled 1 = Interrupts are enabled
|-
| 30 || 1 = Controller will generate interrupt due to Response interrupt, Response Buffer overrun and wake events
|-
| 29:0 || If is particular bit set, it mean that corresponding stream will be able to generate interrupts. Bits for streams are sorted in order: Input Streams, Output Streams, Bidirectional streams. So for example if there is one stream from each category, bit 0 will control Input Stream, bit 1 Output Stream and bit 2 Bidirectional Stream. Unused bits are reserved.
|-
|}

==== 0x34 (4 bytes) Stream Synchronization ====

Note: The HD Audio specifications list this register at offset 0x38, but this appears to be incorrect.

This register allows to start or stop more streams at same time. You can do it by setting bits of these streams what will stop them, then start DMA engine on each of them and finally clear their bits to allow DMA engines to start working with data.

{| class="wikitable"
! Bits
! Description
|-
| 29:0 || If is particular bit clear, it mean that corresponding stream will be able to sent or receive data. Bits for streams are sorted in same order as in register 0x20 Interrupt Control.
|-
|}

==== 0x40 (4 bytes) CORB Lower Base Address ====

{| class="wikitable"
! Bits
! Description
|-
| 31:7 || Lower Base Address of CORB ring
|-
| 6:0 || Reserved to force 1 KB alignment
|-
|}

==== 0x44 (4 bytes) CORB Upper Base Address ====

{| class="wikitable"
! Bits
! Description
|-
| 31:0 || Upper Base Address of CORB ring
|-
|}

==== 0x48 (2 bytes) CORB Write Pointer ====

{| class="wikitable"
! Bits
! Description
|-
| 7:0 || Number of last valid CORB entry
|-
|}

==== 0x4A (2 bytes) CORB Read Pointer ====

{| class="wikitable"
! Bits
! Description
|-
| 15 || 1 = Reset CORB read pointer (this bit is not self-clearing, you have to wait for it to be set, then clear it and wait for it to be clear)
|-
| 7:0 || Number of last CORB entry that controller processed
|-
|}

==== 0x4C (1 byte) CORB Control ====

{| class="wikitable"
! Bits
! Description
|-
| 1 || 0 = CORB is stopped 1 = CORB is running
|-
| 0 || 1 = Interrupt will be generated if there is memory error
|-
|}

==== 0x4E (1 byte) CORB Size ====

{| class="wikitable"
! Bits
! Description
|-
| 6 || 1 = CORB ring can be programmed to have 256 entries
|-
| 5 || 1 = CORB ring can be programmed to have 16 entries
|-
| 4 || 1 = CORB ring can be programmed to have 2 entries
|-
| 1:0 || Number of CORB ring entries. 0b00 = 2 entries 0b01 = 16 entries 0b10 = 256 entries 0b11 = Reserved
|-
|}

==== 0x50 (4 bytes) RIRB Lower Base Address ====

{| class="wikitable"
! Bits
! Description
|-
| 31:7 || Lower Base Address of RIRB ring
|-
| 6:0 || Reserved to force 1 KB alignment
|-
|}

==== 0x54 (4 bytes) RIRB Upper Base Address ====

{| class="wikitable"
! Bits
! Description
|-
| 31:0 || Upper Base Address of RIRB ring
|-
|}

==== 0x58 (2 bytes) RIRB Write Pointer ====

{| class="wikitable"
! Bits
! Description
|-
| 15 || 1 = Reset write pointer (note: this bit always reads as 0)
|-
| 7:0 || Number of last RIRB entry that was written by controller
|-
|}

==== 0x5A (2 bytes) RIRB Response Interrupt Count ====

{| class="wikitable"
! Bits
! Description
|-
| 7:0 || How many responses have to be written by RIRB to generate interrupt
|-
|}

==== 0x5C (1 byte) RIRB Control ====

{| class="wikitable"
! Bits
! Description
|-
| 0 || 1 = Interrupt will be generated if there is response overrun
|-
| 1 || 0 = RIRB is stopped 1 = RIRB is running
|-
| 0 || 1 = Interrupt will be generated if there is memory error
|-
|}

==== 0x5E (1 byte) RIRB Size ====

{| class="wikitable"
! Bits
! Description
|-
| 6 || 1 = RIRB ring can be programmed to have 256 entries
|-
| 5 || 1 = RIRB ring can be programmed to have 16 entries
|-
| 4 || 1 = RIRB ring can be programmed to have 2 entries
|-
| 1:0 || Number of RIRB ring entries. 0b00 = 2 entries 0b01 = 16 entries 0b10 = 256 entries 0b11 = Reserved
|-
|}

==== 0x70 (4 bytes) DMA Position Lower Base Address ====

{| class="wikitable"
! Bits
! Description
|-
| 31:7 || Lower Base Address of DMA position buffer
|-
| 6:1 || Reserved to force 1 KB alignment
|-
| 0 || 1 = DMA position buffer is enabled
|-
|}

==== 0x74 (4 bytes) DMA Position Upper Base Address ====

{| class="wikitable"
! Bits
! Description
|-
| 31:0 || Upper Base Address of DMA position buffer
|-
|}

=== 0x80+ Registers of Stream Descriptors ===

Sound card has as many Stream Descriptors as many streams it have. Registers of every Stream Control occupies 0x20 bytes. As first there are Stream Descriptors for Input Streams, then for Output Streams and then for Bidirectional Streams. Number of streams can be readed from 0x00 Global Capabilities register.

For example, if there are 4 Input Streams, 3 Output Streams and 1 Bidirectional Stream, base address for registers of first Stream Descriptor for Input Stream is at (mmio_base + 0x80), first Stream Descriptor for Output Stream is at (mmio_base + 0x80 + 0x20*4) and first Stream Descriptor for Bidirectional Stream is at (mmio_base + 0x80 + 0x20*4 + 0x20*3).

Offsets in table below should be added to particular Stream Descriptor base.

{| class="wikitable"
! Offset
! Size
! Description
|-
| 0x00 || 1 byte || Stream Control byte 0
|-
| 0x02 || 1 byte || Stream Control byte 2
|-
| 0x03 || 1 byte || Stream Status
|-
| 0x04 || 4 bytes || Link position in buffer entry
|-
| 0x08 || 4 bytes || Cyclic Buffer Length (size of all entries in Buffer Descriptor List added together)
|-
| 0x0C || 2 bytes || Last Valid Index (number of entries in Buffer Descriptor List + 1)
|-
| 0x12 || 2 bytes || Stream Format
|-
| 0x18 || 4 bytes || Lower Base Address of Buffer Descriptor List
|-
| 0x1C || 4 bytes || Upper Base Address of Buffer Descriptor List
|-
|}

==== Stream Control byte 0 ====

{| class="wikitable"
! Bits
! Description
|-
| 4 || 1 = Generate interrupt if there is Descriptor error
|-
| 3 || 1 = Generate interrupt if there is FIFO error
|-
| 2 || 1 = Generate interrupt if was transferred buffer with IOC bit set
|-
| 1 || 1 = Stream is running (transferring data)
|-
| 0 || 1 = Reset all registers of Stream Descriptor, must not be set when Stream is running (this bit is not self-clearing, you have to wait for it to be set, then clear it and wait for it to be clear)
|-
|}

==== Stream Control byte 2 ====

{| class="wikitable"
! Bits
! Description
|-
| 7:4 || Number of stream to which this Stream Descriptor will send data. Value 0 is reserved and indicate that this Stream Descriptor is unused.
|-
| 3 || Relevant only for Bidirectional streams to set their behaviour 0 = Select Input engine 1 = Select Output engine
|-
| 2 || This bit may be read-only. 0 = Stream will be handled on „best effort“ basis 1 = Stream will be transferred as preferred traffic
|-
|}

== CODECs and Nodes ==

Each HD Audio device contains one or more CODECs, and each CODEC contains a list of "nodes" that are connected together in a hierarchical structure.  You can find the available CODECs by sending a GetParameter command to the codec's root node (node 0) using the CORB ring buffer (or the Immediate Command Output Interface, if available). If response is not 0x00000000, it mean that CODEC is present.

== Node Commands ==

Each entry in the CORB ring buffer has the following structure:

{| class="wikitable"
! Bits
! Description
|-
| 31:28 || Codec Address
|-
| 27:20 || Node Index
|-
| 19:8 || Command
|-
| 7:0 || Data
|-
|}

The following list contain some useful verbs that could be send to CODEC nodes:

{| class="wikitable"
! Hex
! Command
! Notes
|-
| F00 || Get Parameter || Data is number of requested parameter
|-
| F02 || Get Connection List Entry ||
|-
| 701 || Set Selected Input ||
|-
| 705 || Set Power State || 0 in data mean full power
|-
| 706 || Set Converter Stream, Channel ||
|-
| 707 || Set Pin Widget Control || 
|-
| F09 || Get Pin Widget Sense || Used to detect if headphone/microphone is plugged into Pin
|-
| 70C || Set EAPD/BTL || 
|-
| F1C || Get Pin Widget Configuration Default || 
|-
| 72D || Set Output Converter Channel Count || 
|-
| 7FF || Audio Function Node Reset || Reset all nodes in particular Audio Function Group
|-
| 0x3 || Set Amplifier Gain || (16-bit payload)
|-
| 0x2 || Set Stream Converter Format || (16-bit payload) Data have same format as Stream Format register
|-
|}

== Node Parameters ==

The following parameters can be read using the Get Parameter verb above:

{| class="wikitable"
! Hex
! Description
! Notes
|-
| 00 || Vendor ID / Device ID || 
|-
| 02 || Revision ID || 
|-
| 04 || Node Count || 
|-
| 05 || Function Group Type || 
|-
| 08 || Audio Group Capabilities || 
|-
| 09 || Audio Widget Capabilities || 
|-
| 0A || Supported PCM Rates || 
|-
| 0B || Supported Formats || 
|-
| 0C || Pin Capabilities || 
|-
| 0D || Input Amplifier Capabilities || 
|-
| 12 || Output Amplifier Capabilities || 
|-
| 0E || Connection List Length || 
|-
| 0F || Supported Power States || 
|-
| 10 || Processing Capabilities || 
|-
| 11 || GPIO Count || 
|-
| 13 || Volume Capabilities || 
|-
|}

== Initalizing sound card ==

TODO

== Initalizing CODEC ==

=== Setting up the AFG codec ===

Each programmer will likely have their own way of going about this, but the main aim will be to set out to identify which pin complexes are connected to actual speakers, headphone sockets, microphones and microphone sockets before following the trail of connections back to the most appropriate DAC or ADC to handle the stream. How you end up connecting things up will depend on what you want to do: you could, for example, use the same DAC for the speakers as for the headphone jack, but there might be occasions when you want to send different sound streams to each. Similarly, you might use the same ADC for the built-in microphone as for the microphone jack, but you could use a different ADC for each if you need to collect different inputs from different microphones (perhaps using one to collect background noise to subtract from the other input). You might also want to send sound out through the microphone socket so that two people can listen to the same music at the same time on headphones, or you might want to use the headphone socket as an extra microphone socket, perhaps to set up a pair of stereo microphones in a tetrahedron arrangement in order to calculate the direction a sound is coming from. Try to make your driver flexible enough to cover all those cases, or at least to make it easy to adapt it to cover them later.

Here's an suggestion as to how the initial task might be broken down, but use it only as a rough guide:-

(1) Use verb F00h with parameter 4 and NID 0 (the root node) to find the number of function groups in the codec, then check those nodes using their NID and with verb F00h and parameter 5 to hunt for an AFG function group.

(2) Use verb F00h with parameter 4 and the correct NID for the AFG function group to get the start node (first widget) and number of nodes (widgets).

(3) Collect the following data for each widget: param 9h (primarily to get the widget type); Ch (pin capabilities); Dh (input amplifier details); 12h (output amplifier details); Eh (connection list length); 13h (volume knob capabilities). Also, use verb F1Ch to collect the configuration defaults, and verb F02h to collect the first few entries in the connection list. Don't worry about whether useful responses are available for each widget - they will send back all zeros wherever they are not relevant.

(4) Sort the results into groups of DACs, ADCs, mixers, pins, etc.

(5) Identify the pins with actual speakers, microphone and sockets for jacks by using the configuration defaults which you collected earlier. Bits 31-30 state which pins have devices attached to them, telling you whether they are integrated devices or sockets into which things can be plugged. Bits 23-20 tell you what the integrated device is or what kind of device will normally be plugged into the socket. Some machines may have multiple output sockets for sound and may in rare cases have more than one set of integrated speakers to play surround sound, so you might need to look at bits 7-4 (default-association) to find out which ones are grouped together as a set, and bits 3-1 (sequence) to determine which are the main speakers in that set. In a few badly configured machines which fail to provide correct information in some of these bits, EAPD capability may serve as an additional clue for tracking down the speakers.

(6) Make two lists of pins: one for those you want to send outputs to, and the other for inputs.

(7) Try to find the shortest paths from the speakers and headphone jacks back to DACs, though you may wish to use different DACs for different pins. See the section "Finding all useful paths through the codec" below for guidance on this, but the first connection in the connection list for a widgit should help you find the right paths for the most common uses, although you may need to explore more items in the connection lists for some of the widgets.

(8) Try to find the shortest paths from the ADCs to any built-in microphone and microphone jack. Again you may wish to use different ADCs for the different inputs or you may wish to share the same one.

(9) Set up the paths you want to use by setting an intitial volume/gain and turning off the mutes at every widget in the chain wherever there is one needing to be set. Use verb 2h to set Format for any DACs and ADCs you're going to use, and verb 706h to set the stream and the lowest channel. Verb 707 must be used to enable input/output at the pins (and read up on VrefEn for mic.s, if you can find any good information on this - I'm still searching for it). Remember to set Unsol for the jacks too if you want pins to report headphones/microphone being plugged in or removed, but you can poll them if you prefer.

(10) For each path, create a list of volume controls, mutes and the range of numbers that can be used with each volume/gain control to guide the creation of suitable controls for the user to manipulate in order to control each input/output.

(11) Set EAPD to enable the external amp for the speakers.

After you've done all that, get the near end of the link set up to get the DMA engines to handle the streams (read the specification carefully and act on it) and with luck you'll soon have sound coming in and out. If it doesn't happen that easily, you can set up the DMA position buffer to see if the DMA engines are actually running and looping correctly through the cyclic stream buffers. If all is well there, you might find it useful to use the codec ID to find the datasheet for it just to make sure you aren't missing anything out in the way you're setting it up. Also, feel free to improve the instructions on this page so that others can gain from your solutions to any problems you encounter.

=== Finding all useful paths through the codec ===

The paths that are potentially useful are the ones that lead from pins with output devices connected to them back to a DAC, plus all the paths from those same pins back to all other pins with an input device connected to them (some codecs allow inputs to be mixed into outputs for such purposes as karaoke, so your device driver might as well make this option available if the codec supports it), plus all the paths from ADCs to all pins with an input device connected. A socket for a jack can be both an output and input device, but not at the same time.

An algorithm guaranteed to find all useful paths is as follows. Write a routine to start at a pin with an attached output device and then call it for each such pin in turn. The routine will look at the association list for the pin it's starting at and follow the path to the first widget listed there. If it finds a DAC (or a pin with an input device attached), the path can be stored. Else, look at the association list for this widget and repeat the process to examine the next widget in the path. The search will backtrack to the previous widget in the chain once a DAC or pin has been reached, and then the next item in the association list for that previous widget tells you which widget to examine next. If there isn't a next item in the association list, backtrack again to the widget before it. In this way, all paths are explored, and the task is complete once you've run out of items in the association list to explore for the pin you started at. It may be wise to keep count of the number of links in the path just in case a codec sends you round in circles - you should abandon any paths that get too long (perhaps a ten link limit would be safe). For inputs the algorithm is much the same, but you start at each ACD and store any paths that link through to a pin which has an input device attached.

Once you have your list, you can select the first suitable one found for each purpose, while the rest can be offered to the user as alternative options accessed either via a user-interface for the device driver or from the interface of an advanced application which directly offers these other paths whenever they exist on the machine. An audio player could, for example, allow you to sing along with the music it's playing such that you hear your voice in the headphones (without any delay from the input being put through a conversion to digital and back).

== Playing Audio ==

If you successfully initalized all nodes that are in path, you can play audio by following steps:

==== Set Audio Output node ====

You have to set Audio Output node to format of your data by verb 0x2, and also set stream number and channel you will use to transfer data by verb 0x706. Node will remember these values, so you need to set them only when you want to change them.

==== Create Buffer Descriptor List ====

Buffer Descriptor List has to be 1 KB aligned, what mean that first seven bits must be zero. It has to contain at least 2 entries. Every entry is 16 bytes long and has following format:

{| class="wikitable"
! Offset
! Length
! Description
|-
| 0x00 || 8 bytes || Address of Audio Data
|-
| 0x08 || 4 bytes || Length of Audio Data in bytes
|-
| 0x0C || 4 bytes || Bit 0 = IOC (Interrupt On Completion), if set, interrupt will be generated after transferring this entry. Rest is reserved.
|-
|}

NOTE: Sound Card will read Buffer Descriptor List data directly from RAM memory, so you have to be sure, that you really wrote it into RAM memory and not only to processor cache. You can do it by flushing processor cache by assembly command „wbinvd“

==== Set Stream Descriptor ====

Now you have to select Stream Descriptor that you do not already use and program it to transfer your Buffer Descriptor List.

(1) Stop Stream Descriptor (clear bit 1 in Control register)

(2) Reset Stream Descriptor (set bit 0 in Control register, wait until it is set, then clear it and wait for it to be cleared)

(3) Write Buffer Descriptor List physical address

(4) Add up length of all entries in Buffer Descriptor List and write it to register

(5) Set number of Buffer Descriptor List entries

(6) Set Stream Format register by same format you sended to Audio Output node

(7) Set number of stream you want to use for streaming data (in byte 2 of Control register)

(8) Start transfer (set bit 1 in Control register)

If you configured everything properly, DMA engine should start transfer your audio data and you should hear sound from PIN(s) you enabled and connected them to Audio Output(s) that are receiving data from your stream number. The audio will play until the end of the last buffer in the buffer list, then it will begin playing the beginning of the first buffer again. You can enable the proper interrupt to be notified when a buffer has been played, so that you can refill the buffer with the next block of audio data.

== See Also ==
=== External links ===

* [http://www.intel.co.uk/content/www/us/en/standards/high-definition-audio-specification.html Intel High Definition Audio Specification]

* [https://forum.osdev.org/viewtopic.php?f=1&t=34525 Thread about playing sound through Intel HD Audio]

* https://forum.osdev.org/viewtopic.php?f=1&t=28944


[[Category:Sound]]
````

<a id="w003314"></a>

## W003314 — AC97

Bron-ID: W003314
Namespace: Main (0)
Revisie: 30980 | Datum: 2026-09-28T12:28:12Z
Bron: https://wiki.osdev.org/AC97
Revisie-URL: https://wiki.osdev.org/index.php?title=AC97&oldid=30980

````text
{{Historical}}

AC'97 is an audio codec standard developed by Intel Architecture Labs in 1997.

==Detecting AC'97 sound card==
Sound card is connected into [[PCI]] with class 0x04 and subclass 0x01. If you found sound card you can check VendorID and DeviceID if it is supporting AC'97. But, because today are mostly used Intel HD Audio cards which have subclass 0x03, you can probably expect that every sound card with subclass 0x01 is sound card compatible with AC'97.
AC'97 use four BARs, but we need only BAR0 - Native Audio Mixer Base Address and BAR1 - Native Audio Bus Master Base Address. Both these BARs are I/O. Before any work with card, you must set bit 0 and bit 2 in [https://wiki.osdev.org/PCI#Command_Register PCI Command Register] to enable I/O space and bus mastering.

==Native Audio Mixer registers==
Here is list of all useful registers of BAR0 base address.

{| class="wikitable"
! Offset
! Description
! Length
|-
| 0x00 || Reset Register/Capabilities || word
|-
| 0x02 || Set Master Output Volume || word
|-
| 0x04 || Set AUX Output Volume || word
|-
| 0x0E || Set Microphone Volume || word
|-
| 0x18 ||| Set PCM Output Volume || word
|-
| 0x1A || Select Input Device || word
|-
| 0x1C || Set Input Gain || word
|-
| 0x1E || Set Gain of Microphone || word
|-
| 0x28 || Extended capabilities || word
|-
| 0x2A || Control of extended capabilities || word
|-
| 0x2C || Sample rate of PCM Front DAC || word
|-
| 0x2E || Sample rate of PCM Surr DAC || word
|-
| 0x30 || Sample rate of PCM LFE DAC || word
|-
| 0x32 || Sample rate of PCM L/R ADC || word
|-
|}

Here is description for all registers:

===0x00 Reset Register===
Writing any value info this register will cause Register reset, what means that all NAM registers will be changed into their default value. When you read this register, it works as Capabilities register. Most useful is bit 4, that indicates, that sound card have headphone output.

===0x02 Master Volume / 0x04 AUX Output Volume===
{| class="wikitable"
! Bit
! Type
! Name
! Description
|-
| 4:0 / 5:0 || RW || Volume of right channel || Number*1,5=Volume in decibels
|-
| 7:6 || RO || Hardwired to 0 ||
|-
| 12:8 / 13:8 || RW || Volume of left channel || Number*1,5=Volume in decibels
|-
| 14 || RO || Hardwired to 0 ||
|-
| 15 || RW || Mute || 1=Mute Sound
|-
|}

0 db means loudest output, 34,5 / 94,5 db means quietest output, mute bit set means no sound. This register support either 32 or 64 steps of changing volume. You can find supported number of steps by setting bit 5 and bit 13. If these bits remains set, you can use 64 steps, if no, you can use only 32 steps.

Master Volume register controls volume of speaker.

AUX Output Volume register can control volume of one of three different types of devices (see part 5.2.1 in specification) which is hardwired by manufacturer. You have to find which one is it. If after register reset this register contain value 0x8000 and Capabilities register have set bit 4, it mean that this register controls output volume of headphone.

===0x0E Microphone Volume (gain)===
{| class="wikitable"
! Bit
! Type
! Name
! Description
|-
| 4:0 || RW || Microphone gain || Every step mean 1,5 db, see below
|-
| 5 || RO || Hardwired to 0 ||
|-
| 6 || RW || 20 db || 1=Add 20 db of volume(gain)
|-
| 14:7 || RO || Hardwired to 0 ||
|-
| 15 || RW || Mute || 1=Mute Sound
|-
|}

01000b means 0 db, so 00000b means 12 db, 11111b means -34,5 db and mute bit set means no sound. Default value of this register is 0x8008.

===0x18 PCM output volume===
{| class="wikitable"
! Bit
! Type
! Name
! Description
|-
| 4:0 || RW || Volume of right channel || Every step mean 1,5 db, see below
|-
| 7:5 || RO || Hardwired to 0 ||
|-
| 12:8 || RW || Volume of left channel || Every step mean 1,5 db, see below
|-
| 14:3 || RO || Hardwired to 0 ||
|-
| 15 || RW || Mute || 1=Mute Sound
|-
|}

01000b means 0 db, so 00000b means 12 db, 11111b means -34,5 db and mute bit set means no sound. Default value of this register is 0x8808. Volume set in this register will affect all outputs that plays PCM data.

===0x1A Select record input device===
Only usable values of this register are 0x0000 (inbuilt Microphone input) and 0x0303 (AUX input - external microphone).

===0x1C Record Gain / 0x1E Record Gain of Microphone===
{| class="wikitable"
! Bit
! Type
! Name
! Description
|-
| 3:0 || RW || Gain of right channel || Number*1,5=Volume in decibels
|-
| 7:4 || RO || Hardwired to 0 ||
|-
| 11:8 || RW || Gain of left channel || Number*1,5=Volume in decibels
|-
| 14:2 || RO || Hardwired to 0 ||
|-
| 15 || RW || Mute || 1=Mute Sound
|-
|}

0 db means max gain, 22,5 db means quietest gain, mute bit set means no gain. Default value of this register is 0x8000.

=== Extended capabilities registers ===
Some cards support more features. You have to check bits in Extended capabilities register to check whose features card support, and then enable them by setting same bits in Control of extended capabilities register.

Most useful feature is support for more sample rates than 48000. This feature have bit 0. If this bit is set in Extended capabilities register, you can activate it by setting bit 0 in Control of extended capabilities register. Then you can just write value of sample rate to four Sample rate registers. All card with this feature have to support sample rates 48000 and 44100 and optionally 32000, 22050, 16000, 11025 and 8000.

==Native Audio Bus Master registers==
Here is list of all useful registers of BAR1 base address.

{| class="wikitable"
! Offset
! Description
! Length
|-
| 0x00 || NABM register box for PCM IN  || below
|-
| 0x10 || NABM register box for PCM OUT || below
|-
| 0x20 || NABM register box for Microphone || below
|-
| 0x2C || Global Control Register || dword
|-
| 0x30 || Global Status Register || dword
|-
|}

Every NABM register box contain these registers:
{| class="wikitable"
! Offset
! Description
! Length
|-
| 0x00 || Physical Address of Buffer Descriptor List || dword
|-
| 0x04 || Number of Actual Processed Buffer Descriptor Entry || byte
|-
| 0x05 || Number of all Descriptor Entries || byte
|-
| 0x06 || Status of transferring Data || word
|-
| 0x08 || Number of transferred Samples in Actual Processed Entry || word
|-
| 0x0A || Number of next processed Buffer Entry || byte
|-
| 0x0B || Transfer Control || byte
|-
|}

Here are descriptions of registers:

===0x2C Global Control===
{| class="wikitable"
! Bit
! Type
! Name
! Description
|-
| 0 || RW || Global Interrupt Enable (GIE) || 0=Disable Interrupts 1=Enable Interrupts
|-
| 1 || RW  || Cold reset || 0=Device is in reset and can not be used 1=Resume to operational state
|-
| 2 || RW || Warm reset ||
|-
| 3 || RW || Shut down || 0=Device is powered 1=Shut down
|-
| 6:4 || RW || (you do not need this bits) ||
|-
| 19:7 || RO || Reserved ||
|-
| 21:20 || RW || Channels for PCM Output || 00=2 channels 01=4 channels 10=6 channels 11=Reserved
|-
| 23:22 || RW || PCM Output mode || 00=16 bit samples 01=20 bit samples
|-
|}

===0x30 Global Status===
{| class="wikitable"
! Bit
! Type
! Name
! Description
|-
| 21:20 || RO || Channel capabilities || 00=2 channels 01=4 channels 10=6 channels 11=Reserved
|-
| 23:22 || RO || Sample capabilities || 01=16 and 20 bit samples are supported Other=Reserved, only 16 bit samples supported
|-
|}

This register contains much more bits, but you probably do not need them.

===0x00 0x10 0x20 Buffer Descriptor Base Address===
{| class="wikitable"
! Bit
! Type
! Name
! Description
|-
| 2:0 || RO || Hardwired to 0 ||
|-
| 31:3 || RW || Base Address || Physical base address of Buffer Descriptor List
|-
|}

===0x04 0x14 0x24 Current Processed Entry===
{| class="wikitable"
! Bit
! Type
! Name
! Description
|-
| 4:0 || RO || Current Entry || Number of Buffer Entry which is actually processed
|-
| 7:5 || RO || Hardwired to 0 ||
|-
|}

===0x05 0x15 0x25 Last Valid Entry===
{| class="wikitable"
! Bit
! Type
! Name
! Description
|-
| 4:0 || RW || Last Entry || Number of last Buffer Entry which will be processed
|-
| 7:5 || RO || Hardwired to 0 ||
|-
|}

===0x06 0x16 0x26 Transfer Status===
{| class="wikitable"
! Bit
! Type
! Name
! Description
|-
| 0 || RO || DMA controller status || 0=Transferring data 1=Halted
|-
| 1 || RO || End of transfer || 0=Current Entry is not equal to Last Valid Entry 1=CE is equal to LVE
|-
| 2 || R/WC || Last Buffer Entry transferred Interrupt || Software clears this bit writing 1 to it 1=Interrupt is firing
|-
| 3 || R/WC || IOC Interrupt || Software clears this bit writing 1 to it 1=Interrupt is firing
|-
| 4 || R/WC || Fifo ERROR Interrupt || Software clears this bit writing 1 to it 1=Interrupt is firing
|-
| 15:5 || RO || Reserved ||
|-
|}

If you get some interrupt, you must clear this register by write value 0x1C otherwise the interrupt will be fired forever.

===0x08 0x18 0x28 Position in Current Entry===
{| class="wikitable"
! Bit
! Type
! Name
! Description
|-
| 15:0 || RO || Number of transferred data || Number of samples which were transferred from currently processed entry
|-
|}

===0x0A 0x1A 0x2A Prefetched Entry===
{| class="wikitable"
! Bit
! Type
! Name
! Description
|-
| 4:0 || RO || Prefetched Entry || Number of prefetched Buffer Entry
|-
| 7:5 || RO || Hardwired to 0 ||
|-
|}

===0x0B 0x1B 0x2B Transfer Control===
{| class="wikitable"
! Bit
! Type
! Name
! Description
|-
| 0 || RW || DMA controller control || 0=Pause transferring 1=Transfer sound data
|-
| 1 || RW || Reset || 0=Remove reset condition 1=Reset this NABM register box, this bit is cleared by card when reset is complete
|-
| 2 || RW || Last Buffer Entry Interrupt enable || 0=Disable interrupt 1=Enable interrupt
|-
| 3 || RW || IOC Interrupt enable || 0=Disable interrupt 1=Enable interrupt
|-
| 4 || RW || Fifo ERROR Interrupt enable || 0=Disable interrupt 1=Enable interrupt
|-
| 7:5 || RO || Reserved ||
|-
|}

For IOC interrupt see below part Buffer Descriptor List.

==Buffer Descriptor List==
When you have PCM sound data in memory, you must describe them in Buffer Descriptor List. Every entry of list have following format:

{| class="wikitable"
! Offset
! Description
! Length
|-
| 0x00 || Physical Address to sound data in memory || dword
|-
| 0x04 || Number of samples in this buffer  || word
|-
| 0x06 || Bit 15=Interrupt fired when data from this entry is transferred Bit 14=Last entry of buffer, stop playing  Other bits=Reserved || word
|-
|}

Buffer Description List has 32 entries. While adding entries, you have to roll over. Every entry can transfer up to 0xFFFE samples. If input contains multiple channels, all are counted into this value, so to play five frames from PCM data with 2 channels, you have to write value 10. If you have 2 channels 16 bit audio, value 0xFFFE means that (2 bytes per sample*0xFFFE) = 128 KB will be transferred. Programming number of samples with an odd number such as 0xFFFF is not allowed.

Default samples are 16 bit samples stored in words, but AC'97 can also support 20 bit samples that are stored in dwords in higher bytes. If entry have set bit 15 (called IOC), after successful transfer of all data interrupt will be fired. If you want use it, be sure that IOCE bit in BUFFER_CNT register is set. If is set bit 14 (called BUP), it means that this entry is last from this list and AC'97 will stop playing sound after transferring this entry.

===Theory of Operation===

During playing back, The BDL is a simple ring buffer between the producer (driver code running on the CPU) and the consumer (AC'97's DMA).

The producer keeps updating the contents of the sample buffers described by the BDL entries and optionally the BDL itself, and the write pointer (Last Valid Entry register at offset 0x5 in the BOX). 

The consumer keeps reading the BDL entries, the contents of the sample buffers, the read pointer (Current Processed Entry at offset 0x4 in the BOX), and a second read pointer (Prefetched Entry at offset 0xA in the BOX).

Unlike HDA's stream DMAs which can keep running in cycles by themselves, AC'97's DMA will stop if its read pointer catches up with the write pointer.

Thus the driver code should take care to keep the sample buffers updated and the write pointer sufficiently ahead of the DMA's read pointer to avoid any underrun or DMA stoppage. The driver should also take note of the auxiliary read pointer if it needs to update BDL entries, as the DMA can prefetch BDL entries and possible read stale sample buffers if the driver updated the BDL entries after the prefetching.

==Using AC'97 sound card==
In initalization of sound card you must resume card from cold reset and set power for it. It can be done by writing value 0x2 to Global Control register if you do not want to use interrupts, or 0x3 if you want to use interrupts. After this, you should write any value to NAM Reset register to reset all NAM registers. After this, you can read card capability info from Global Status register to found out if 20 bit audio samples are supported and also check out bit 4 in NAM Capabilites register and value in AUX Output to find out if this sound card support headphone output. As last thing, set maximal volume for PCM Output by writing value 0 to this register. Now sound card is ready to use.

===Playing sound===
* Set volume of output you want to use - Master Volume register (NAM register 0x02) for speaker and if supported, AUX Output register (NAM register 0x04) for headphone
* Load sound data to memory and describe it in Buffer Descriptor List
* Set reset bit of output channel (NABM register 0x1B, value 0x2) and wait for card to clear it
* Write physical position of BDL to Buffer Descriptor Base Address register (NABM register 0x10)
* Write number of last valid buffer entry to Last Valid Entry register (NABM register 0x15)
* Set bit for transferring data (NABM register 0x1B, value 0x1)

Now wait for interrupt if you use it and do not forget to clear status register by write value 0x1C to it. If you do not use interrupt, you can pool for bit 1 of NABM Transfer Status register. While playing, you have to fill entries that were already transferred and then update Last Valid Entry.

===Stop and resuming sound===
Stopping sound is easy, you only need to clear bit 0 in NABM transfer control register. If you want to resume playing, set this bit back. If you want absolutely stop sound or reprogram playing other sound, you must stop transferring data and reset this NABM register box (bit 1 in NABM Transfer Control register 0x1B). Resetting will set all registers in box to zero.

== See Also ==
=== Articles ===
* [[Intel High Definition Audio]] - superseding standard also made by Intel, implemented on most modern chipsets.

=== Drivers ===
* [https://github.com/VendelinSlezak/BleskOS/blob/master/source/drivers/sound/ac97.c BleskOS AC'97 driver (C code)]

=== External Links ===
* [https://web.archive.org/web/20180508155315id_/http://www-inst.eecs.berkeley.edu/~cs150/Documents/ac97_r23.pdf Audio Codec '97 Specification]
* [https://web.archive.org/web/20170226174953im_/http://www.intel.com/content/dam/doc/manual/io-controller-hub-7-hd-audio-ac97-manual.pdf Intel ICH7 HDA/AC'97 Programmer's Reference Manual]
* [https://web.archive.org/web/20210422185924/https://www.intel.com/assets/pdf/manual/252751.pdf Intel ICH5/ICH5R/ICH4 AC'97 Programmer's Reference Manual]

[[Category:Sound]]
[[de:AC97]]
````

<a id="w003315"></a>

## W003315 — Talk:Raspberry Pi

Bron-ID: W003315
Namespace: Talk (1)
Revisie: 26588 | Datum: 2021-10-24T10:48:56Z
Bron: https://wiki.osdev.org/Talk:Raspberry_Pi
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ARaspberry_Pi&oldid=26588

````text

== Tutorial format ==

Hi,

For the scope of this tutorial, I was wondering if C would be a better language than C++. It's not as if the OO paradigm comes particularly useful while writing a simple tutorial, and C *can* be accepted as a language the whole community is (mostly) expected to know.

As for the actual tutorial, I've been thinking of doing away with the "provide files with comments" ideology. Most of the article just provides the files, and expects the end-reader to understand what's going on. It does give some explanation for a few files, but it's mostly feeble, at the best. It also provides stuff like user-space utilities and Makefiles, which aren't strictly required in the context of the tutorial. Note that this isn't local to this article, but pertains across the entire wiki.

What I suggest & propose is a more "classic" JamesM style tutorial -- one where small blocks of code are followed by simple explanations, without grouping everything together so that the end-user/reader can grasp it rather than simply copy-pasting. Small challenges (such as implementation of memset in asm?) can be provided to the user/reader, rather than being given in black&white. At the end, tarballs of each tutorial "level" can be provided, so that if someone has any problems, s/he can refer to them.

For ARM, there seems to be a lack of comprehensive coverage, as I suppose the community expects whoever is starting with an ARM kernel to be well versed with a simple x86-based kernel? I hope more articles/tutorials pop up for ARM, so that new dev'ers could directly start with the RISC architecture, rather than delve with x86.

Since Mrvn seems to be the only contributor for this Wiki article, if he has no problems, I'd be willing to fix the above-mentioned issues in the article. For the entire wiki, I won't be able to do so -- but if everyone from the community does their small bit, we might be able to bring the standard of all articles a bit.

Regards,
--[[User:Shikhin|Shikhin]] 11:49, 18 January 2013 (CST)

NOTE: If the entire-wiki related talk could be moved to a better talk page, I've absolutely no problem with it. Also note that the above are my own humble views. I'm not trying to offend anyone, and if someone has a different view, we can talk it over (the author has clearly seen a lot of flame-talks ;).

I agree with the change of format. There is a lot of code, while simple memset()s can be implemented in C(++) and language independent. As for the language issue, any code snippets I feel should be C++. C++ simplifies code snippets because of the easy categorization by classes (or namespaces). I have written my OS in C++ and it makes code elegant. While there are good reasons to use C in an OS (compatibility), for tutorial snippets C++ won't come amiss, as it makes everything neater. It won't take much for anybody that knows C to understand C++.
--[[User:Bellezzasolo|Bellezzasolo]] 14:06, 19 January 2013 (CST)

I've already started to split the main article and moved the Hello World kernel into its own page (and changed it back to pure C). I also want to add a more object oriented C++ flavour of the same to show what you can do. This is mainly about programming style so it should be enough to demonstrate them with the Hello World kernel and UART.

Beyond that I want to step away from a step by step tutorial approach and concentrate on providing independent modules. Modules should be mostly standalone but probably will have some dependencies, e.g. nearly everything will use the UART.
Modules would be things like a kprintf(), a framebuffer, activating interrupts, creating and activating page tables, USB/keyboard input, SC card access, ... The idea is that you start with the Hello World tutorial to get of the ground. Then you decide what to do next and look the module for that. You don't have to do framebuffer, you could do paging first.

When I started the page I wanted to just give a few code sniplets and comments and upload a tarball of the full source. But the wiki doesn't allow tarballs. So I uploaded the full files so people can download something that simply works. Maybe the code should live on github? Other ideas?

--[[User:Mrvn|Mrvn]] 19:38, 20 January 2013 (CET)
````

<a id="w003316"></a>

## W003316 — Raspberry Pi Bare Bones

Bron-ID: W003316
Namespace: Main (0)
Revisie: 30867 | Datum: 2026-07-07T17:32:10Z
Bron: https://wiki.osdev.org/Raspberry_Pi_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Raspberry_Pi_Bare_Bones&oldid=30867

````text
{{BeginnersWarning}}
{{Rating|3}}
{{Template:Kernel designs}}

This is a tutorial on operating systems development on the [[Raspberry Pi]]. This will serve as an example of how to create a minimal system, but not as an example of how to properly structure your project.

There's a similar tutorial [[Raspberry Pi Bare Bones Rust]] which uses Rust instead of C.

== Prepare ==

You are about to begin development of a new operating system. Perhaps one day, your new operating system can be developed under itself. This is a process known as bootstrapping or going self-hosted. However, that is way into the future. Today, we simply need to set up a system that can compile your operating system from an existing operating system. This is a process known as cross-compiling and this makes the first step in operating systems development.

This article assumes you are using a Unix-like operating system such as Linux which supports operating systems development well. Windows users should be able to complete it from a MinGW or Cygwin environment. 

== Building a Cross-Compiler ==
:''Main article: [[GCC Cross-Compiler]], [[Why do I need a Cross Compiler?]]

The first thing you should do is set up a [[GCC Cross-Compiler]] for '''arm-none-eabi'''. You have not yet modified your compiler to know about the existence of your operating system, so we use a generic target called arm-none-eabi, which provides you with a toolchain targeting the [[System V ABI]].
You will ''not'' be able to correctly compile your operating system without a cross-compiler.

If you want a 64 bit kernel, you should set up '''aarch64-elf''' target instead. This provides the same System V ABI interface, but for 64 bit. Using elf is not mandatory, but simplifies things.

== Overview ==

By now, you should have set up your [[GCC Cross-Compiler|cross-compiler]] for the proper ABI (as described above). This tutorial provides a minimal solution for creating an operating system. It doesn't serve as a recommend skeleton for project structure, but rather as an example of a minimal kernel. In this simple case, we just need three input files:

* boot.S - kernel entry point that sets up the processor environment
* kernel.c - your actual kernel routines
* linker.ld - for linking the above files

== Booting the Operating System  ==

We will now create a file called boot.S and discuss its contents. In this example, we are using the GNU assembler, which is part of the cross-compiler toolchain you built earlier. This assembler integrates very well with the rest of the GNU toolchain. 

Each Pi Model requires different set up. In general, you must distinguish AArch32 and AArch64 mode, as they are booted differently. The latter only accessible from Pi 3 and upwards. Within a mode, you can [[Detecting_Raspberry_Pi_Board|detect the board]] in run-time, and set up mmio base address accordingly.

===Pi Model A, B, A+, B+, and Zero===

The environment is set up and execution to _start transferred from [https://github.com/raspberrypi/tools/blob/master/armstubs/armstub.S#L35 armstub.s].
<syntaxhighlight lang="asm">
// AArch32 mode

// To keep this in the first portion of the binary.
.section ".text.boot"

// Make _start global.
.globl _start

        .org 0x8000
// Entry point for the kernel.
// r15 -> should begin execution at 0x8000.
// r0 -> 0x00000000
// r1 -> 0x00000C42 - machine id
// r2 -> 0x00000100 - start of ATAGS
// preserve these registers as argument for kernel_main
_start:
	// Setup the stack.
	mov sp, #0x8000

	// Clear out bss.
	ldr r4, =__bss_start
	ldr r9, =__bss_end
	mov r5, #0
	mov r6, #0
	mov r7, #0
	mov r8, #0
	b       2f

1:
	// store multiple at r4.
	stmia r4!, {r5-r8}

	// If we are still below bss_end, loop.
2:
	cmp r4, r9
	blo 1b

	// Call kernel_main
	ldr r3, =kernel_main
	blx r3

	// halt
halt:
	wfe
	b halt
</syntaxhighlight>

The section ".text.boot" will be used in the linker script to place the boot.S as the very first thing in our kernel image. The code initializes a minimum C environment, which means having a stack and zeroing the BSS segment, before calling the kernel_main function. Note that the code avoids using r0-r2 so the remain valid for the kernel_main call.

You can then assemble boot.S using:

<syntaxhighlight lang="shell-session">
$ arm-none-eabi-gcc -mcpu=arm1176jzf-s -fpic -ffreestanding -c boot.S -o boot.o 
</syntaxhighlight>

===Pi 2===

With newer versions of the Pi, there's a bit more to be done. Raspberry Pis 2 and 3 (the first model that supports 64 bit) have 4 cores. On boot, all cores are running and executing the same boot code. Therefore you have to distinguish cores, and only allow one of them to run, putting the others in an infinite loop.

The environment is set up and execution to _start transferred from [https://github.com/raspberrypi/tools/blob/master/armstubs/armstub7.S#L167 armstub7.s].
<syntaxhighlight lang="asm">
// AArch32 mode

// To keep this in the first portion of the binary.
.section ".text.boot"

// Make _start global.
.globl _start

        .org 0x8000
// Entry point for the kernel.
// r15 -> should begin execution at 0x8000.
// r0 -> 0x00000000
// r1 -> 0x00000C42 - machine id
// r2 -> 0x00000100 - start of ATAGS
// preserve these registers as argument for kernel_main
_start:
	// Shut off extra cores
	mrc p15, 0, r5, c0, c0, 5
	and r5, r5, #3
	cmp r5, #0
	bne halt

	// Setup the stack.
	ldr r5, =_start
	mov sp, r5

	// Clear out bss.
	ldr r4, =__bss_start
	ldr r9, =__bss_end
	mov r5, #0
	mov r6, #0
	mov r7, #0
	mov r8, #0
	b       2f

1:
	// store multiple at r4.
	stmia r4!, {r5-r8}

	// If we are still below bss_end, loop.
2:
	cmp r4, r9
	blo 1b

	// Call kernel_main
	ldr r3, =kernel_main
	blx r3

	// halt
halt:
	wfe
	b halt
</syntaxhighlight>

You can assemble boot.S using:

<syntaxhighlight lang="shell-session">
$ arm-none-eabi-gcc -mcpu=cortex-a7 -fpic -ffreestanding -c boot.S -o boot.o 
</syntaxhighlight>

===Pi 3, 4===

It worth mentioning that Pi 3 and 4 normally boots kernel8.img into 64 bit mode, but you can still use AArch32 with kernel7.img for backward compatibility. Note that in 64 bit mode, the boot code is loaded at 0x80000 and not 0x8000. The boot code in AArch64 is exactly the same for Pi 3 and 4, however Pi 4 has a different peripheral base address (see below the C example code).

With the latest firmware, only the primary core runs (core 0), and the secondary cores are awaiting in a spin loop. To wake them up, write a function's address at 0xE0 (core 1), 0xE8 (core 2) or 0xF0 (core 3) and they will start to execute that function.

The environment is set up and execution to _start transferred from [https://github.com/raspberrypi/tools/blob/master/armstubs/armstub8.S#L154 armstub8.s].
<syntaxhighlight lang="asm">
// AArch64 mode

// To keep this in the first portion of the binary.
.section ".text.boot"

// Make _start global.
.globl _start

    .org 0x80000
// Entry point for the kernel. Registers:
// x0 -> 32 bit pointer to DTB in memory (primary core only) / 0 (secondary cores)
// x1 -> 0
// x2 -> 0
// x3 -> 0
// x4 -> 32 bit kernel entry point, _start location
_start:
    // set stack before our code
    ldr     x5, =_start
    mov     sp, x5

    // clear bss
    ldr     x5, =__bss_start
    ldr     w6, =__bss_size
1:  cbz     w6, 2f
    str     xzr, [x5], #8
    sub     w6, w6, #1
    cbnz    w6, 1b

    // jump to C code, should not return
2:  bl      kernel_main
    // for failsafe, halt this core
halt:
    wfe
    b halt

</syntaxhighlight>

Compile your code with:

<syntaxhighlight lang="shell-session">
$ aarch64-elf-as -c boot.S -o boot.o
</syntaxhighlight>

== Implementing the Kernel ==

So far we have written the bootstrap assembly stub that sets up the processor such that high level languages such as C can be used. It is also possible to use other languages such as C++.

=== Freestanding and Hosted Environments ===

If you have done C or C++ programming in user-space, you have used a so-called Hosted Environment. Hosted means that there is a C standard library and other useful runtime features. Alternatively, there is the Freestanding version, which is what we are using here. Freestanding means that there is no C standard library, only what we provide ourselves. However, some header files are actually not part of the C standard library, but rather the compiler. These remain available even in freestanding C source code. In this case we use <stddef.h> to get size_t & NULL and <stdint.h> to get the intx_t and uintx_t datatypes which are invaluable for operating systems development, where you need to make sure that the variable is of an exact size (if we used a short instead of uint16_t and the size of short changed, our code would break!). Additionally you can access the <float.h>, <iso646.h>, <limits.h>, and <stdarg.h> headers, as they are also freestanding. GCC actually ships a few more headers, but these are special purpose.

=== Writing a kernel in C ===

The following shows how to create a simple kernel in C. Please take a few moments to understand the code. To set the value for "int raspi" in run-time, see [[Detecting_Raspberry_Pi_Board|detecting the board type]].

<syntaxhighlight lang="cpp">
#include <stddef.h>
#include <stdint.h>

static uint32_t MMIO_BASE;

// The MMIO area base address, depends on board type
static inline void mmio_init(int raspi)
{
    switch (raspi) {
        case 2:
        case 3:  MMIO_BASE = 0x3F000000; break; // for raspi2 & 3
        case 4:  MMIO_BASE = 0xFE000000; break; // for raspi4
        default: MMIO_BASE = 0x20000000; break; // for raspi1, raspi zero etc.
    }
}

// Memory-Mapped I/O output
static inline void mmio_write(uint32_t reg, uint32_t data)
{
	*(volatile uint32_t*)(MMIO_BASE + reg) = data;
}

// Memory-Mapped I/O input
static inline uint32_t mmio_read(uint32_t reg)
{
	return *(volatile uint32_t*)(MMIO_BASE + reg);
}

// Loop <delay> times in a way that the compiler won't optimize away
static inline void delay(int32_t count)
{
	asm volatile("__delay_%=: subs %[count], %[count], #1; bne __delay_%=\n"
		 : "=r"(count): [count]"0"(count) : "cc");
}

enum
{
    // The offsets for reach register.
    GPIO_BASE = 0x200000,

    // Controls actuation of pull up/down to ALL GPIO pins.
    GPPUD = (GPIO_BASE + 0x94),

    // Controls actuation of pull up/down for specific GPIO pin.
    GPPUDCLK0 = (GPIO_BASE + 0x98),

    // The base address for UART.
    UART0_BASE = (GPIO_BASE + 0x1000), // for raspi4 0xFE201000, raspi2 & 3 0x3F201000, and 0x20201000 for raspi1

    // The offsets for reach register for the UART.
    UART0_DR     = (UART0_BASE + 0x00),
    UART0_RSRECR = (UART0_BASE + 0x04),
    UART0_FR     = (UART0_BASE + 0x18),
    UART0_ILPR   = (UART0_BASE + 0x20),
    UART0_IBRD   = (UART0_BASE + 0x24),
    UART0_FBRD   = (UART0_BASE + 0x28),
    UART0_LCRH   = (UART0_BASE + 0x2C),
    UART0_CR     = (UART0_BASE + 0x30),
    UART0_IFLS   = (UART0_BASE + 0x34),
    UART0_IMSC   = (UART0_BASE + 0x38),
    UART0_RIS    = (UART0_BASE + 0x3C),
    UART0_MIS    = (UART0_BASE + 0x40),
    UART0_ICR    = (UART0_BASE + 0x44),
    UART0_DMACR  = (UART0_BASE + 0x48),
    UART0_ITCR   = (UART0_BASE + 0x80),
    UART0_ITIP   = (UART0_BASE + 0x84),
    UART0_ITOP   = (UART0_BASE + 0x88),
    UART0_TDR    = (UART0_BASE + 0x8C),

    // The offsets for Mailbox registers
    MBOX_BASE    = 0xB880,
    MBOX_READ    = (MBOX_BASE + 0x00),
    MBOX_STATUS  = (MBOX_BASE + 0x18),
    MBOX_WRITE   = (MBOX_BASE + 0x20)
};

// A Mailbox message with set clock rate of PL011 to 3MHz tag
volatile unsigned int  __attribute__((aligned(16))) mbox[9] = {
    9*4, 0, 0x38002, 12, 8, 2, 3000000, 0 ,0
};

void uart_init(int raspi)
{
	mmio_init(raspi);

	// Disable UART0.
	mmio_write(UART0_CR, 0x00000000);
	// Setup the GPIO pin 14 && 15.

	// Disable pull up/down for all GPIO pins & delay for 150 cycles.
	mmio_write(GPPUD, 0x00000000);
	delay(150);

	// Disable pull up/down for pin 14,15 & delay for 150 cycles.
	mmio_write(GPPUDCLK0, (1 << 14) | (1 << 15));
	delay(150);

	// Write 0 to GPPUDCLK0 to make it take effect.
	mmio_write(GPPUDCLK0, 0x00000000);

	// Clear pending interrupts.
	mmio_write(UART0_ICR, 0x7FF);

	// Set integer & fractional part of baud rate.
	// Divider = UART_CLOCK/(16 * Baud)
	// Fraction part register = (Fractional part * 64) + 0.5
	// Baud = 115200.

	// For Raspi3 and 4 the UART_CLOCK is system-clock dependent by default.
	// Set it to 3Mhz so that we can consistently set the baud rate
	if (raspi >= 3) {
		// UART_CLOCK = 30000000;
		unsigned int r = (((unsigned int)(&mbox) & ~0xF) | 8);
		// wait until we can talk to the VC
		while ( mmio_read(MBOX_STATUS) & 0x80000000 ) { }
		// send our message to property channel and wait for the response
		mmio_write(MBOX_WRITE, r);
		while ( (mmio_read(MBOX_STATUS) & 0x40000000) || mmio_read(MBOX_READ) != r ) { }
	}

	// Divider = 3000000 / (16 * 115200) = 1.627 = ~1.
	mmio_write(UART0_IBRD, 1);
	// Fractional part register = (.627 * 64) + 0.5 = 40.6 = ~40.
	mmio_write(UART0_FBRD, 40);

	// Enable FIFO & 8 bit data transmission (1 stop bit, no parity).
	mmio_write(UART0_LCRH, (1 << 4) | (1 << 5) | (1 << 6));

	// Mask all interrupts.
	mmio_write(UART0_IMSC, (1 << 1) | (1 << 4) | (1 << 5) | (1 << 6) |
	                       (1 << 7) | (1 << 8) | (1 << 9) | (1 << 10));

	// Enable UART0, receive & transfer part of UART.
	mmio_write(UART0_CR, (1 << 0) | (1 << 8) | (1 << 9));
}

void uart_putc(unsigned char c)
{
	// Wait for UART to become ready to transmit.
	while ( mmio_read(UART0_FR) & (1 << 5) ) { }
	mmio_write(UART0_DR, c);
}

unsigned char uart_getc()
{
    // Wait for UART to have received something.
    while ( mmio_read(UART0_FR) & (1 << 4) ) { }
    return mmio_read(UART0_DR);
}

void uart_puts(const char* str)
{
	for (size_t i = 0; str[i] != '\0'; i ++)
		uart_putc((unsigned char)str[i]);
}

#if defined(__cplusplus)
extern "C" /* Use C linkage for kernel_main. */
#endif

#ifdef AARCH64
// arguments for AArch64
void kernel_main(uint64_t dtb_ptr32, uint64_t x1, uint64_t x2, uint64_t x3)
#else
// arguments for AArch32
void kernel_main(uint32_t r0, uint32_t r1, uint32_t atags)
#endif
{
	// initialize UART for Raspi2
	uart_init(2);
	uart_puts("Hello, kernel World!\r\n");

	while (1)
		uart_putc(uart_getc());
}
</syntaxhighlight>

The GPU bootloader passes arguments to the AArch32 kernel via r0-r2 and the boot.S makes sure to preserve those 3 registers. They are the first 3 arguments in a C function call. The argument r0 contains a code for the device the RPi was booted from.  This is generally 0 but its actual value depends on the firmware of the board.  r1 contains the 'ARM Linux Machine Type' which for the RPi is 3138 (0xc42) identifying the BCM2708 CPU. A full list of ARM Machine Types is available from [http://www.arm.linux.org.uk/developer/machines/ here].  r2 contains the address of the ATAGs.

For AArch64, the registers are a little bit different, but also passed as arguments to the C function. The first, x0 is the 32 bit address of the DTB (that is, [https://elinux.org/Device_Tree_Reference Device Tree Blob] in memory). Watch out, it is a 32 bit address, the upper bits may not be cleared. The other arguments, x1-x3 are cleared to zero for now, but reserved for future use. Your boot.S should preserve them.

Notice how we wish to use the common C function strlen, but this function is part of the C standard library that we don't have available. Instead, we rely on the freestanding header <stddef.h> to provide size_t and we simply declare our own implementation of strlen. You will have to do this for every function you wish to use (as the freestanding headers only provide macros and data types). 

The addresses for the GPIO and UART are offsets from the peripheral base address, which is 0x20000000 for Raspberry Pi 1 and 0x3F000000 for Raspberry Pi 2 and Raspberry Pi 3. For [[Raspberry Pi 4]] the base address is 0xFE000000. You can find the addresses of registers and how to use them in the BCM2835 manual. It is possible to detect the base address in run time by [[Detecting_Raspberry_Pi_Board|reading the board id]].

Compile using:

<syntaxhighlight lang="bash">
arm-none-eabi-gcc -mcpu=arm1176jzf-s -fpic -ffreestanding -std=gnu99 -c kernel.c -o kernel.o -O2 -Wall -Wextra
</syntaxhighlight>

or for 64 bit:

<syntaxhighlight lang="bash">
aarch64-elf-gcc -ffreestanding -c kernel.c -o kernel.o -O2 -Wall -Wextra
</syntaxhighlight>

Note that the above code uses a few extensions and hence we build as the GNU version of C99.

== Linking the Kernel ==

To create the full and final kernel we will have to link these object files into the final kernel program. When developing user-space programs, your toolchain ships with default scripts for linking such programs. However, these are unsuitable for kernel development and we need to provide our own customized linker script.

The linker script for 64 bit mode looks exactly the same, except for the starting address.

<syntaxhighlight lang="text">
ENTRY(_start)

SECTIONS
{
    /* Starts at LOADER_ADDR. */
    . = 0x8000;
    /* For AArch64, use . = 0x80000; */
    __start = .;
    __text_start = .;
    .text :
    {
        KEEP(*(.text.boot))
        *(.text)
    }
    . = ALIGN(4096); /* align to page size */
    __text_end = .;

    __rodata_start = .;
    .rodata :
    {
        *(.rodata)
    }
    . = ALIGN(4096); /* align to page size */
    __rodata_end = .;

    __data_start = .;
    .data :
    {
        *(.data)
    }
    . = ALIGN(4096); /* align to page size */
    __data_end = .;

    __bss_start = .;
    .bss :
    {
        bss = .;
        *(.bss)
    }
    . = ALIGN(4096); /* align to page size */
    __bss_end = .;
    __bss_size = __bss_end - __bss_start;
    __end = .;
}
</syntaxhighlight>

There is a lot of text here but don't despair. The script is rather simple if you look at it bit by bit.

ENTRY(_start) declares the entry point for the kernel image. That symbol was declared in the boot.S file. Since we are actually booting a binary image, the entry is completely irrelevant, but it has to be there in the elf file we build as intermediate file.

SECTIONS declares sections. It decides where the bits and pieces of our code and data go and also sets a few symbols that help us track the size of each section.

<syntaxhighlight lang=text>
    . = 0x8000;
    __start = .;
</syntaxhighlight>

The "." denotes the current address so the first line tells the linker to set the current address to 0x8000 (or 0x80000), where the kernel starts. The current address is automatically incremented when the linker adds data. The second line then creates a symbol "__start" and sets it to the current address.

After that sections are defined for text (code), read-only data, read-write data and BSS (0 initialized memory). Other than the name the sections are identical so lets just look at one of them:

<syntaxhighlight lang=text>
    __text_start = .;
    .text : {
        KEEP(*(.text.boot))
        *(.text)
    }
    . = ALIGN(4096); /* align to page size */
    __text_end = .;
</syntaxhighlight>

The first line creates a __text_start symbol for the section. The second line opens a .text section for the output file which gets closed in the fifth line. Lines 3 and 4 declare what sections from the input files will be placed inside the output .text section. In our case ".text.boot" is to be placed first followed by the more general ".text". ".text.boot" is only used in boot.S and ensures that it ends up at the beginning of the kernel image. ".text" then contains all the remaining code. Any data added by the linker automatically increments the current address ("."). In line 6 we explicitly increment it so that it is aligned to a 4096 byte boundary (which is the page size for the RPi). And last line 7 creates a __text_end symbol so we know where the section ends.

What are the __text_start and __text_end for and why use page alignment? The 2 symbols can be used in the kernel source and the linker will then place the correct addresses into the binary. As an example the __bss_start and __bss_end are used in boot.S. But you can also use the symbols from C by declaring them extern first. While not required I made all sections aligned to page size. This later allows mapping them in the page tables with executable, read-only and read-write permissions without having to handle overlaps (2 sections in one page).

<syntaxhighlight lang=text>
    __end = .;
</syntaxhighlight>

After all sections are declared the __end symbol is created. If you ever want to know how large your kernel is at runtime you can use __start and __end to find out.

With these components you can now actually build the final kernel. We use the compiler as the linker as it allows it greater control over the link process. Note that if your kernel is written in C++, you should use the C++ compiler instead.

You can then link your kernel using:

<syntaxhighlight lang="bash">
arm-none-eabi-gcc -T linker.ld -o myos.elf -ffreestanding -O2 -nostdlib boot.o kernel.o -lgcc
arm-none-eabi-objcopy myos.elf -O binary kernel7.img
</syntaxhighlight>

or for 64 bit:

<syntaxhighlight lang="bash">
aarch64-elf-gcc -T linker.ld -o myos.elf -ffreestanding -O2 -nostdlib boot.o kernel.o -lgcc 
aarch64-elf-objcopy myos.elf -O binary kernel8.img
</syntaxhighlight>

== Booting the Kernel ==

In a few moments, you will see your kernel in action. 

=== Testing your operating system (Real Hardware) ===

Do you still have the SD card with the original Raspbian image on it from when you where testing the hardware above? Great. So you already have a SD card with a boot partition and the required files. If not then download one of the original Raspberry boot images and copy them to the SD card.

Now mount the first partition from the SD card and look at it:

<pre>
bootcode.bin  fixup.dat     kernel.img            start.elf
cmdline.txt   fixup_cd.dat  kernel_cutdown.img    start_cd.elf
config.txt    issue.txt     kernel_emergency.img
</pre>

If you don't have a raspbian image, you can create a FAT32 partition, and [https://github.com/raspberrypi/firmware/tree/master/boot download the firmware files] from the official repository. You'll need only three files:

* bootcode.bin: this is the one that's loaded first, executed on the GPU (not needed on RPi4 as that model has bootcode.bin in a ROM)
* fixup.dat: this data file contains important hardware-related information, a must have
* start.elf: this is the RPi firmware (same as BIOS on IBM PC). This also runs on the GPU.

Simplified when the RPi powers up the ARM CPU is halted and the GPU runs. The GPU loads the bootloader from ROM and executes it. That then finds the SD card and loads the bootcode.bin (except RPi4 which has a big enough ROM to include bootcode.bin as well). The bootcode loads the firmware, start.elf which handles the config.txt and cmdline.txt. The start.elf loads the kernel*.img and at last the ARM CPU is started running that kernel image.

To switch among ARM modes, you have to rename your kernel.img file. If you rename it to '''kernel7.img''', that will be executed in AArch32 mode (ARMv7). For AArch64 mode (ARMv8) you'll have to rename it to '''kernel8.img'''.

So now we replace the original kernel.img with our own, umount, sync, stick the SD card into RPi and turn the power on.
Your Minicom should then show the following:

<pre>
Hello, kernel World!
</pre>

=== Testing your operating system (QEMU) ===

QEMU supports emulating Raspberry Pi 2 with the machine type "raspi2". At the time of writing this feature is not available in most package managers but can be found in the latest QEMU source found here: https://github.com/qemu/qemu

Check that your QEMU install has qemu-system-arm and that it supports the option "-M raspi2". When testing in QEMU, be sure to use the raspi2 base addresses noted in the source code.

With QEMU you do not need to objcopy the kernel into a plain binary; QEMU also supports ELF kernels:

<syntaxhighlight lang=bash>
$YOURINSTALLLOCATION/bin/qemu-system-arm -m 256 -M raspi2 -serial stdio -kernel kernel.elf
</syntaxhighlight>

==== Updated Support for AArch64 (raspi2, raspi3, raspi4) ====
As of QEMU 2.12 (April 2018), emulation for 64-bit ARM, ''qemu-system-aarch64'', now supports direct emulation of both Raspberry Pi 2 and 3 using the machine types '''raspi2''' and '''raspi3''', respectively. This should allow for testing of 64-bit system code.

<syntaxhighlight lang="bash">
qemu-system-aarch64 -M raspi3 -serial stdio -kernel kernel8.img
</syntaxhighlight>

As of QEMU 9.0 (April 2024), emulation for 64-bit ARM, ''qemu-system-aarch64'', now supports emulation of the Rasberry Pi 4 using the machine type '''raspi4b'''. 
<syntaxhighlight lang="bash">
qemu-system-aarch64 -M raspi4b -serial stdio -kernel kernel8.img
</syntaxhighlight>

Note that in most cases, there will be few if any differences between 32-bit ARM code and 64-bit ARM, but there can be difference in the way the code behaves, in particular regarding kernel memory management. Also, some AArch64 implementations may support features not found on any of their 32-bit counterparts (e.g., cryptographic extensions, enhanced NEON SIMD support).

Another very important note: starting from Raspberry Pi 3, the SoC is changed to [https://github.com/raspberrypi/documentation/files/1888662/ BCM2837] and PL011 clock (UART0) is not fixed any more, but derived from the system clock. Therefore to properly set up baud rate, first you have to set clock frequency. This can be done with a [https://github.com/raspberrypi/firmware/wiki/Mailboxes Mailbox call]. Or you can use AUX miniUART (UART1) chip, which is easier to program. The links below includes tutorials on how to do both.

== See Also ==

=== Articles ===
* [[Raspberry Pi Bare Bones Rust]]
* [[ARMv7-A Bare Bones]]

=== External Links ===
* [http://www.raspberrypi.org/wp-content/uploads/2012/02/BCM2835-ARM-Peripherals.pdf BCM2835 ARM Peripherals] (original Raspberry Pi)
* [https://github.com/raspberrypi/documentation/files/1888662/ BCM2837 ARM Peripherals] (latest Raspberry Pi 3)
* [http://infocenter.arm.com/help/topic/com.arm.doc.ddi0183g/DDI0183G_uart_pl011_r1p5_trm.pdf PrimeCell UART (PL011) Technical Reference Manual]
* [https://github.com/s-matyukevich/raspberry-pi-os Raspberry-Pi-OS a hobby OS tutorial for the Raspberry Pi] (details Linux drivers too, great source)
* [https://github.com/bztsrc/raspi3-tutorial Bare metal tutorial for AArch64]

[[Category:ARM]]
[[Category:AArch64]]
[[Category:Raspberry Pi]]
[[Category:Bare bones tutorials]]
[[Category:C]]
[[Category:C++]]
````

<a id="w003317"></a>

## W003317 — Uefi.inc

Bron-ID: W003317
Namespace: Main (0)
Revisie: 25442 | Datum: 2020-12-15T16:46:45Z
Bron: https://wiki.osdev.org/Uefi.inc
Revisie-URL: https://wiki.osdev.org/index.php?title=Uefi.inc&oldid=25442

````text
{{TutorialTone}}
<tt>uefi.inc</tt> a library for [[UEFI]] written in assembly for [[FASM]]. It's main goal was small code, and to achieve that it gives only limited access to UEFI. It was designed to fulfill all the needs of a boot loader, namely

* Provide user input interface.
* Exactly one way to load and save sectors.
* Framebuffer services for output.

It is not intend to be an universal UEFI library, because you won't need all UEFI functionality in a bootloader, just the ones listed above.

== Hello World from UEFI ==
This shows how to import and use the library. The macro hides UEFI ABI from the programmer, so you don't have to worry about number of arguments, stack alignment and in which registers to pass arguments etc.
<source lang="asm">
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
</source>

== Description ==

There's only two functions, as you can see in the Hello World example. The names came from the original [[GNU-EFI]] SDK.

* InitializeLib - initializes the library, you should call this as soon as possible.
* uefi_call_wrapper - provides a way to call an UEFI function with it's ABI

Other than these, you need structure definitions (like ConIn) to figure out the actual function addresses. These are also defined in the include file. As said earlier, those definitions are limited to Console, Framebuffer and Storage to keep the include file small.

If you want, you can add more structure definitions any time in a separate include file. The uefi_call_wrapper macro allows you to call those, but it also specifies shortcuts for the most common functions, so that you don't have to use BootServices or RuntimeServices prefixes.

== Case studies: usage in a boot loader ==
Please note that these are only tutorials. They focus on a specific topic only. They do not allocate buffers they use for example. You'll have to write that part on your own.
They were written for those who wants to roll their own UEFI boot loader in assembly. I hope it will save them some sleepless nights and headaches.
=== Reading a character from keyboard ===
<source lang="asm">
@@:		uefi_call_wrapper	ConIn, ReadKeyStroke, ConIn, key
		cmp			dword [key.scancode], 0
		jz			@b

;data
key:
key.scancode:	dw 			0
key.unicode:	du			0
</source>

=== Detecting Memory Map ===
<source lang="asm">
		mov			dword [memmapdescsize], 48
		uefi_call_wrapper	BootServices, GetMemoryMap, memmapsize, memmapbuff, memmapkey, memmapdescsize, memmapdescver
		cmp			dword [memmapdescsize], 0
		jnz			@f
		mov			dword [memmapdescsize], 48
@@:		clc
		cmp			rax, EFI_SUCCESS
		je			@f
		stc
@@:

;data
memmapsize:	dq			MEMMAP_BUFFSIZE
memmapkey:	dq			0
memmapdescsize:	dq			0
memmapdescver:	dq			0
memmapbuff:	rb			MEMMAP_BUFFSIZE
</source>
Note that memmapkey is needed for ExitBootServices. If you use memory allocation or free, you'll have to query again map to get a valid key.

=== Iterating on disks ===
<source lang="asm">
		; query device handles
		mov			dword [tmp], DEVICEHANDLE_BUFFSIZE
		uefi_call_wrapper	BootServices, LocateHandle, 2, blkiouuid, 0, tmp, devicehandle_buff
		cmp			rax, EFI_SUCCESS
		jne			.error

		; iterate on handles
		mov			r15, devicehandle_buff
		mov			r14, qword [tmp]
.nexthandle:	mov			rcx, qword [r15]
		or			rcx, rcx
		jz			.end
		; open handle
		push			r14
		push			r15
		uefi_call_wrapper	BootServices, OpenProtocol, rcx, blkiouuid, tmp, rcx, 0, 2
		pop			r15
		pop			r14
		cmp			rax, EFI_SUCCESS
		jne			.skipdev
		; save results
		mov			rax, qword [tmp]
		mov			qword [bootdiskblkio], rax
		; FIXME: do some checks here to decide if it's your device or not
		; I load GPT from each device and look for a specific UUID for example
		; cmp ismydevice, 1
		; je .end

.skipdev:	xor			rax, rax
		mov			qword [bootdiskblkio], rax
		pop			r15
		pop			r14
		add			r15, 8
		sub			r14, 8
		js			.error
		jnz			.nexthandle

.end:		clc
		ret

.error:		stc
		ret

;data
tmp:		dq			0
blkiouuid:	db			EFI_BLOCK_IO_PROTOCOL_UUID
bootdiskblkio:	dq			0
</source>

=== Loading a sector from device ===
<source lang="asm">
		mov			rax, qword [bootdiskblkio]
		stc
		or			rax, rax
		jz			@f
		mov			rbx, rax
		mov			rcx, rax
		mov			rax, qword [rax+EFI_BLOCK_IO_PROTOCOL.Media]
		or			rax, rax
		jz			@f
		mov			edx, dword [rax+EFI_BLOCK_IO_MEDIA.MediaId]
		or			edx, edx
		jz			@f
		xor			rax, rax
		uefi_call_wrapper	rbx, EFI_BLOCK_IO_PROTOCOL.ReadBlocks, rcx, rdx, qword [lba], qword [sizeinbytes], diskbuff
		stc
		cmp			rax, EFI_SUCCESS
		jne			@f
		clc
@@:

;data
lba:		dq			0
sizeinbytes:	dq			0
diskbuff:	rb			DISK_BUFFSIZE
</source>

=== Get sorted list of available video modes ===
For a detailed description, see the [[GOP]] article.
<source lang="asm">
		; installation check on GOP. This must return buffer too small if GOP supported
		xor			rax, rax
		mov			qword [tmp], rax
		uefi_call_wrapper	BootServices, LocateHandle, 2, gopuuid, 0, tmp, gop_handle
		cmp			rax, EFI_BUFFER_TOO_SMALL
		jne			.error
		; query available video modes
		mov			dword [tmp], MODE_BUFFSIZE
		uefi_call_wrapper	BootServices, LocateHandle, 2, gopuuid, 0, tmp, qword [tmpbuff]
		cmp			rax, EFI_SUCCESS
		jne			.error
		; iterate on each handle, store sorted result to gopmodes
		mov			rsi, qword [tmpbuff]
		lea			rdi, [gopmodes]
.getnextinfo:	lodsq
		or			rax, rax
		jz			.nomore
		mov			qword [tmp], rax
		mov			qword [rdi], rax
		push			rsi
		push			rdi
		uefi_call_wrapper	BootServices, HandleProtocol, qword [tmp], gopuuid, gopinterface
		pop			rdi
		pop			rsi
		cmp			rax, EFI_SUCCESS
		jne			.error
		; iterate on each mode for a handle
		; get number of modes
		mov			rcx, qword [gopinterface]
		mov			r10, qword [rcx + EFI_GRAPHICS_OUTPUT_PROTOCOL.Mode]
		mov			r10, qword [r10 + EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE.MaxMode]
		xor			rdx, rdx
.nextmode:	mov			rcx, qword [gopinterface]
		mov			qword [rdi+8], rcx
		push			rsi
		push			rdi
		mov			dword [tmp], 8
		lea			r8, [tmp]
		lea			r9, [gopinfo]
		push			rcx
		push			rdx
		push			r10
		uefi_call_wrapper	rcx, EFI_GRAPHICS_OUTPUT_PROTOCOL.QueryMode, rcx, rdx, r8, r9
		pop			r10
		pop			rdx
		pop			rcx
		pop			rdi
		pop			rsi
		cmp			rax, EFI_SUCCESS
		jne			.error
		; save info to gopmodes
		mov			r9, qword [gopinfo]
		mov			eax, dword [r9+EFI_GRAPHICS_OUTPUT_MODE_INFORMATION.HorizontalResolution]
		mov			word [rdi+16], ax
		mov			eax, dword [r9+EFI_GRAPHICS_OUTPUT_MODE_INFORMATION.VerticalResolution]
		mov			word [rdi+18], ax
		mov			eax, dword [r9+EFI_GRAPHICS_OUTPUT_MODE_INFORMATION.PixelsPerScanline]
		mov			word [rdi+20], ax
		mov			ax, word [r9+EFI_GRAPHICS_OUTPUT_MODE_INFORMATION.PixelFormat]
		mov			word [rdi+22], ax
		mov			dword [rdi+24], edx
		;bubble up mode
		mov			rbx, rdi
.bubble:		mov			ax, word [rdi+16]
		cmp			word [rdi+16-32], ax
		ja			.ok2
		jb			.switch
		mov			ax, word [rdi+18]
		cmp			word [rdi+18-32], ax
		jae			@f
.switch:		mov			rax, qword [rdi]
		xchg			qword [rdi-32], rax
		mov			qword [rdi], rax
		mov			rax, qword [rdi+8]
		xchg			qword [rdi-32+8], rax
		mov			qword [rdi+8], rax
		mov			rax, qword [rdi+16]
		xchg			qword [rdi-32+16], rax
		mov			qword [rdi+16], rax
		mov			rax, qword [rdi+24]
		xchg			qword [rdi-32+24], rax
		mov			qword [rdi+24], rax
		sub			rdi, 32
		cmp			rdi, gopmodes
		jae			.bubble
@@:		mov			rdi, rbx
		add			rdi, 32
.loop:		inc			rdx
		cmp			edx, r10d
		jb			.nextmode
		jmp			.getnextinfo
.error:		xor			rax, rax
		mov			qword [gopinterface], rax
		stc
.nomore:

;data
tmp:		dq			0
tmpbuff:	dq			0
gopuuid:	db			EFI_GRAPHICS_OUTPUT_PROTOCOL_UUID
gop_handle:	dq			0
gopinterface:	dq			0
gopinfo:	dq			0
;output, sorted list of 32 byte entries
;  8 bytes handle
;  8 bytes interface
;  2 bytes horizontal resolution
;  2 bytes vertical resolution
;  2 bytes scanline
;  2 bytes pixelformat
;  8 bytes mode number
gopmodes:	rb			MODE_BUFFSIZE
</source>
To access framebuffer, you'll have to get it's address. This code returns it in rbx, and buffer's size in rcx:
<source lang="asm">
		mov			rax, qword [gopinterface]
		mov			rax, qword [rax + EFI_GRAPHICS_OUTPUT_PROTOCOL.Mode]
		mov			rbx, qword [rax + EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE.FrameBufferBase]
		mov			rcx, qword [rax + EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE.FrameBufferSize]
</source>

== The include file ==
And finally the library that makes the magic alive:
<source lang="asm">
;*********************************************************************
;*                                                                   *
;*          UEFI library for fasm by bzt, Public Domain              *
;*                                                                   *
;*********************************************************************

; include x86asm.net's efi.inc
;include 'efi.inc'
struc int8 {
  . db ?
}
struc int16 {
  align 2
  . dw ?
}
struc int32 {
  align 4
  . dd ?
}
struc int64 {
  align 8
  . dq ?
}
struc intn {
  align 8
  . dq ?
}
struc dptr {
  align 8
  . dq ?
}

;symbols

EFIERR = 0x8000000000000000
EFI_SUCCESS			= 0
EFI_LOAD_ERROR			= EFIERR or 1
EFI_INVALID_PARAMETER		= EFIERR or 2
EFI_UNSUPPORTED 		= EFIERR or 3
EFI_BAD_BUFFER_SIZE		= EFIERR or 4
EFI_BUFFER_TOO_SMALL		= EFIERR or 5
EFI_NOT_READY			= EFIERR or 6
EFI_DEVICE_ERROR		= EFIERR or 7
EFI_WRITE_PROTECTED		= EFIERR or 8
EFI_OUT_OF_RESOURCES		= EFIERR or 9
EFI_VOLUME_CORRUPTED		= EFIERR or 10
EFI_VOLUME_FULL 		= EFIERR or 11
EFI_NO_MEDIA			= EFIERR or 12
EFI_MEDIA_CHANGED		= EFIERR or 13
EFI_NOT_FOUND			= EFIERR or 14
EFI_ACCESS_DENIED		= EFIERR or 15
EFI_NO_RESPONSE 		= EFIERR or 16
EFI_NO_MAPPING			= EFIERR or 17
EFI_TIMEOUT			= EFIERR or 18
EFI_NOT_STARTED 		= EFIERR or 19
EFI_ALREADY_STARTED		= EFIERR or 20
EFI_ABORTED			= EFIERR or 21
EFI_ICMP_ERROR			= EFIERR or 22
EFI_TFTP_ERROR			= EFIERR or 23
EFI_PROTOCOL_ERROR		= EFIERR or 24

;helper macro for definition of relative structure member offsets

macro struct name
{
  virtual at 0
    name name
  end virtual
}

;structures
EFI_SYSTEM_TABLE_SIGNATURE	equ	49h,42h,49h,20h,53h,59h,53h,54h
struc EFI_TABLE_HEADER {
 .Signature    		int64
 .Revision     		int32
 .HeaderSize   		int32
 .CRC32        		int32
 .Reserved     		int32
}
struct EFI_TABLE_HEADER

struc EFI_SYSTEM_TABLE {
 .Hdr		        EFI_TABLE_HEADER
 .FirmwareVendor        dptr
 .FirmwareRevision      int32
 .ConsoleInHandle       dptr
 .ConIn 	        dptr
 .ConsoleOutHandle      dptr
 .ConOut	        dptr
 .StandardErrorHandle   dptr
 .StdErr	        dptr
 .RuntimeServices       dptr
 .BootServices	        dptr
 .NumberOfTableEntries  intn
 .ConfigurationTable    dptr
}
struct EFI_SYSTEM_TABLE

struc SIMPLE_TEXT_OUTPUT_INTERFACE {
 .Reset 	    	dptr
 .OutputString	    	dptr
 .TestString	    	dptr
 .QueryMode	    	dptr
 .SetMode	    	dptr
 .SetAttribute	    	dptr
 .ClearScreen	    	dptr
 .SetCursorPosition 	dptr
 .EnableCursor	    	dptr
 .Mode		    	dptr
}
struct SIMPLE_TEXT_OUTPUT_INTERFACE

;---include ends

struc SIMPLE_INPUT_INTERFACE {
 .Reset			dptr
 .ReadKeyStroke		dptr
 .WaitForKey		dptr
}
struct SIMPLE_INPUT_INTERFACE

struc EFI_BOOT_SERVICES_TABLE {
 .Hdr		       	EFI_TABLE_HEADER
 .RaisePriority		dptr
 .RestorePriority	dptr
 .AllocatePages		dptr
 .FreePages		dptr
 .GetMemoryMap		dptr
 .AllocatePool		dptr
 .FreePool		dptr
 .CreateEvent		dptr
 .SetTimer		dptr
 .WaitForEvent		dptr
 .SignalEvent		dptr
 .CloseEvent		dptr
 .CheckEvent		dptr
 .InstallProtocolInterface dptr
 .ReInstallProtocolInterface dptr
 .UnInstallProtocolInterface dptr
 .HandleProtocol	dptr
 .Void			dptr
 .RegisterProtocolNotify dptr
 .LocateHandle		dptr
 .LocateDevicePath	dptr
 .InstallConfigurationTable dptr
 .ImageLoad		dptr
 .ImageStart		dptr
 .Exit			dptr
 .ImageUnLoad		dptr
 .ExitBootServices	dptr
 .GetNextMonotonicCount	dptr
 .Stall			dptr
 .SetWatchdogTimer	dptr
 .ConnectController	dptr
 .DisConnectController	dptr
 .OpenProtocol		dptr
 .CloseProtocol		dptr
 .OpenProtocolInformation dptr
 .ProtocolsPerHandle	dptr
 .LocateHandleBuffer	dptr
 .LocateProtocol	dptr
 .InstallMultipleProtocolInterfaces dptr
 .UnInstallMultipleProtocolInterfaces dptr
 .CalculateCrc32	dptr
 .CopyMem		dptr
 .SetMem		dptr
}
struct EFI_BOOT_SERVICES_TABLE

struc EFI_RUNTIME_SERVICES_TABLE {
 .Hdr		       	EFI_TABLE_HEADER
 .GetTime		dptr
 .SetTime		dptr
 .GetWakeUpTime		dptr
 .SetWakeUpTime		dptr
 .SetVirtualAddressMap	dptr
 .ConvertPointer	dptr
 .GetVariable		dptr
 .GetNextVariableName	dptr
 .SetVariable		dptr
 .GetNextHighMonoCount	dptr
 .ResetSystem		dptr
}
struct EFI_RUNTIME_SERVICES_TABLE

struc EFI_TIME {
 .Year			int16
 .Month			int8
 .Day			int8
 .Hour			int8
 .Minute		int8
 .Second		int8
 .Pad1			int8
 .Nanosecond		int32
 .TimeZone		int16
 .Daylight		int8
 .Pad2			int8
 .sizeof		rb 1
}
struct EFI_TIME

EFI_LOADED_IMAGE_PROTOCOL_UUID equ 0A1h,31h,1bh,5bh,62h,95h,0d2h,11h,8Eh,3Fh,0h,0A0h,0C9h,69h,72h,3Bh
struc EFI_LOADED_IMAGE_PROTOCOL {
 .Revision		int32
 .ParentHandle		int64
 .SystemTable		dptr
 .DeviceHandle		int64
 .FilePath		dptr
 .Reserved		int64
 .LoadOptionsSize	int32
 .ImageBase		dptr
 .ImageSize		int64
 .ImageCodeType		int32
 .ImageDataType		int32
 .UnLoad		dptr
}
struct EFI_LOADED_IMAGE_PROTOCOL

EFI_BLOCK_IO_PROTOCOL_UUID equ 21h,5bh,4eh,96h,59h,64h,0d2h,11h,8eh,39h,00h,0a0h,0c9h,69h,72h,3bh
struc EFI_BLOCK_IO_PROTOCOL {
 .Revision		int64
 .Media			dptr
 .Reset			dptr
 .ReadBlocks		dptr
 .WriteBlocks		dptr
 .FlushBlocks		dptr
}
struct EFI_BLOCK_IO_PROTOCOL

struc EFI_BLOCK_IO_MEDIA {
 .MediaId		int32
 .RemovableMedia	int8
 .MediaPresent		int8
 .LogicalPartition	int8
 .ReadOnly		int8
 .WriteCaching		int8
 .BlockSize		int32
 .IoAlign		int32
 .LastBlock		int64
}
struct EFI_BLOCK_IO_MEDIA

EFI_GRAPHICS_OUTPUT_PROTOCOL_UUID equ 0deh, 0a9h, 42h,90h,0dch,023h,38h,04ah,96h,0fbh,7ah,0deh,0d0h,80h,51h,6ah
struc EFI_GRAPHICS_OUTPUT_PROTOCOL {
 .QueryMode		dptr
 .SetMode		dptr
 .Blt			dptr
 .Mode			dptr
}
struct EFI_GRAPHICS_OUTPUT_PROTOCOL

struc EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE {
 .MaxMode		int32
 .CurrentMode		int32
 .ModeInfo		dptr
 .SizeOfModeInfo	intn
 .FrameBufferBase	dptr
 .FrameBufferSize	intn
}
struct EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE

struc EFI_GRAPHICS_OUTPUT_MODE_INFORMATION {
 .Version		int32
 .HorizontalResolution	int32
 .VerticalResolution	int32
 .PixelFormat		int32
 .RedMask		int32
 .GreenMask		int32
 .BlueMask		int32
 .Reserved		int32
 .PixelsPerScanline	int32
}
struct EFI_GRAPHICS_OUTPUT_MODE_INFORMATION
;---macros to make life easier---
;call it early, after entry point is the best
macro InitializeLib
{
		clc
		or			rdx, rdx
		jz			.badout
		cmp			dword [rdx], 20494249h
		je			@f
.badout:	  xor			rcx, rcx
		xor			rdx, rdx
		stc
@@:		mov			[efi_handler], rcx		; ImageHandle
		mov			[efi_ptr], rdx    		; pointer to SystemTable
}

;invoke an UEFI function
macro uefi_call_wrapper			interface,function,arg1,arg2,arg3,arg4,arg5,arg6,arg7,arg8,arg9,arg10,arg11
{
numarg = 0
if ~ arg11 eq
 numarg = numarg + 1
 if ~ arg11 eq rdi
		mov			rdi, arg11
 end if
end if
if ~ arg10 eq
 numarg = numarg + 1
 if ~ arg10 eq rsi
		mov			rsi, arg10
 end if
end if
if ~ arg9 eq
 numarg = numarg + 1
 if ~ arg9 eq r14
		mov			r14, arg9
 end if
end if
if ~ arg8 eq
 numarg = numarg + 1
 if ~ arg8 eq r13
		mov			r13, arg8
 end if
end if
if ~ arg7 eq
 numarg = numarg + 1
 if ~ arg7 eq r12
		mov			r12, arg7
 end if
end if
if ~ arg6 eq
 numarg = numarg + 1
 if ~ arg6 eq r11
		mov			r11, arg6
 end if
end if
if ~ arg5 eq
 numarg = numarg + 1
 if ~ arg5 eq r10
		mov			r10, arg5
 end if
end if
if ~ arg4 eq
 numarg = numarg + 1
 if ~ arg4 eq r9
		mov			r9, arg4
 end if
end if
if ~ arg3 eq
 numarg = numarg + 1
 if ~ arg3 eq r8
		mov			r8, arg3
 end if
end if
if ~ arg2 eq
 numarg = numarg + 1
 if ~ arg2 eq rdx
		mov			rdx, arg2
 end if
end if
if ~ arg1 eq
 numarg = numarg + 1
 if ~ arg1 eq rcx
  if ~ arg1 in <ConsoleInHandle,ConIn,ConsoleOutHandle,ConOut,StandardErrorHandle,StdErr,RuntimeServices,BootServices>
		mov			rcx, arg1
  end if
 end if
end if
		xor			rax, rax
		mov			al, numarg
if interface in <ConsoleInHandle,ConIn,ConsoleOutHandle,ConOut,StandardErrorHandle,StdErr,RuntimeServices,BootServices>
		mov			rbx, [efi_ptr]
		mov			rbx, [rbx + EFI_SYSTEM_TABLE.#interface]
else
 if ~ interface eq rbx
		mov			rbx, interface
 end if
end if
if arg1 in <ConsoleInHandle,ConIn,ConsoleOutHandle,ConOut,StandardErrorHandle,StdErr,RuntimeServices,BootServices>
		mov			rcx, rbx
end if
if defined SIMPLE_INPUT_INTERFACE.#function
		mov			rbx, [rbx + SIMPLE_INPUT_INTERFACE.#function]
else
 if defined SIMPLE_TEXT_OUTPUT_INTERFACE.#function
		mov			rbx, [rbx + SIMPLE_TEXT_OUTPUT_INTERFACE.#function]
 else
  if defined EFI_BOOT_SERVICES_TABLE.#function
		mov			rbx, [rbx + EFI_BOOT_SERVICES_TABLE.#function]
  else
   if defined EFI_RUNTIME_SERVICES_TABLE.#function
		mov			rbx, [rbx + EFI_RUNTIME_SERVICES_TABLE.#function]
   else
    if defined EFI_GRAPHICS_OUTPUT_PROTOCOL.#function
		mov			rbx, [rbx + EFI_GRAPHICS_OUTPUT_PROTOCOL.#function]
    else
     if defined EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE.#function
		mov			rbx, [rbx + EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE.#function]
     else
		mov			rbx, [rbx + function]
     end if
    end if
   end if
  end if
 end if
end if
		call			uefifunc
}

;*********************************************************************
;*                       Library functions                           *
;*********************************************************************

section '.text' code executable readable

uefifunc:	;save stack pointer
		mov			qword [uefi_rsptmp], rsp
		;set up new aligned stack
		and			esp, 0FFFFFFF0h
		;alignment check on arguments
		bt			eax, 0
		jnc			@f
		push			rax
		;arguments
@@:		cmp			al, 11
		jb			@f
		push			rdi
@@:		cmp			al, 10
		jb			@f
		push			rsi
@@:		cmp			al, 9
		jb			@f
		push			r14
@@:		cmp			al, 8
		jb			@f
		push			r13
@@:		cmp			al, 7
		jb			@f
		push			r12
@@:		cmp			al, 6
		jb			@f
		push			r11
@@:		cmp			al, 5
		jb			@f
		push			r10
@@:		;space for
		;r9
		;r8
		;rdx
		;rcx
		sub			rsp, 4*8
		;call function
		call			rbx
		;restore old stack
		mov			rsp, qword [uefi_rsptmp]
		ret

section '.data' data readable writeable
efi_handler:	dq			0
efi_ptr:	dq			0
uefi_rsptmp:	dq			0

</source>
[[Category:UEFI]][[Category:X86]]
````

<a id="w003319"></a>

## W003319 — XSDT

Bron-ID: W003319
Namespace: Main (0)
Revisie: 30273 | Datum: 2026-03-16T17:51:03Z
Bron: https://wiki.osdev.org/XSDT
Revisie-URL: https://wiki.osdev.org/index.php?title=XSDT&oldid=30273

````text
{{ACPI}}

'''eXtended System Descriptor Table (XSDT)''' - the 64-bit version of the ACPI [[RSDT]]
==Introduction==
''"the XSDT provides identical functionality to the RSDT but accommodates physical addresses of DESCRIPTION HEADERs that are larger than 32-bits."'' - ACPI Specification v5.0. This means all addresses are now 64 bits. If the pointer to the XSDT is valid, the OS '''''MUST''''' use the XSDT. All of the addresses will probably be 32 bits, and if you are not using PAE, you couldn't use higher than 32 bits, but the spec says you should use this anyway.
==Detecting the XSDT==
To find the XSDT you need first to locate and check the [[RSDP]], then use the XsdtPointer (if it exists).
==Validating the XSDT==
You only need to sum all the bytes in the table and compare the result to 0.

==Structure==
The XSDT is the main System Description Table. However there are many kinds of SDT. All the SDT may be split into two parts. One (the header) which is common to all the SDT and another (data) which is different for each table.
The structure of the header is:
<syntaxhighlight lang="c">
struct ACPISDTHeader {
  char Signature[4];
  uint32_t Length;
  uint8_t Revision;
  uint8_t Checksum;
  char OEMID[6];
  char OEMTableID[8];
  uint32_t OEMRevision;
  uint32_t CreatorID;
  uint32_t CreatorRevision;
};
</syntaxhighlight>
=== Header fields ===
==== Signature ====
All the ACPI tables have a 4 byte Signature field (except the RSDP which has an 8 byte one). Using the signature, you can determine what table are you working with.

==== Length ====
Total size of the table, inclusive of the header.
==== Checksum ====
A 8 bit checksum field of the whole table, inclusive of the header. All bytes of the table summed must be equal to 0 (mod 0x100). You should always do a checksum of the table before using it, even if you found the table linked by other tables.
<syntaxhighlight lang="c">
bool doChecksum(ACPISDTHeader *tableHeader)
{
    unsigned char sum = 0;

    for (int i = 0; i < tableHeader->Length; i++)
    {
        sum += ((char *) tableHeader)[i];
    }

    return sum == 0;
}
</syntaxhighlight>

=== Other fields ===
The XSDT contains pointers to other tables:
<syntaxhighlight lang="c">
struct XSDT {
  struct ACPISDTHeader h;
  uint64_t PointerToOtherSDT[(h.Length - sizeof(h)) / 8];
};
</syntaxhighlight>

'''Important''': Note that the 'PointerToOtherSDT' field as defined above should be aligned to a 4-byte boundary and not the default 8-byte alignment for a uint64_t.
The offset to the first pointer from the beginning of the XSDT table is 36 bytes.

== How to use ==
Well, suppose you want to find the [[FADT]]. The FADT, being a SDT has a ACPISDTHeader. It's signature is "FACP".
You may use this code:
<syntaxhighlight lang="c">
void *findFACP(void *RootSDT)
{
    XSDT *xsdt = (XSDT *) RootSDT;
    int entries = (xsdt->h.Length - sizeof(xsdt->h)) / 8;

    for (int i = 0; i < entries; i++)
    {
        ACPISDTHeader *h = (ACPISDTHeader *) xsdt->PointerToOtherSDT[i];
        if (!strncmp(h->Signature, "FACP", 4))
            return (void *) h;
    }

    // No FACP found
    return NULL;
}
</syntaxhighlight>

== What can you find? ==

=== Defined by ACPI ===

The ACPI Specification 5.0 has direct definitions for the following System Descriptor tables, identified by their signatures:

* "APIC": Multiple APIC Description Table ([[MADT]])
* "BGRT": Boot Graphics Resource Table ([[BGRT]]; only supported on UEFI systems)
* "BERT": Boot Error Record Table ([[BERT]])
* "CPEP": Corrected Platform Error Polling Table ([[CPEP]])
* "DSDT": Differentiated System Description Table ([[DSDT]])
* "ECDT": Embedded Controller Boot Resources Table ([[ECDT]])
* "EINJ": Error Injection Table ([[EINJ]])
* "ERST": Error Record Serialization Table ([[ERST]])
* "FACP": Fixed ACPI Description Table ([[FADT]])
* "FACS": Firmware ACPPI Control Structure ([[FACS]])
* "HEST": Hardware Error Source Table ([[HEST]])
* "MSCT": Maximum System Characteristics Table ([[MSCT]])
* "MPST": Memory Power State Table ([[MPST]])
* "OEMx": OEM Specific Information Tables (Any table with a signature beginning with "OEM" falls into this definition)
* "PMTT" Platform Memory Topology Table ([[PMTT]])
* "PSDT": Persistent System Description Table ([[PSDT]])
* "RASF": ACPI RAS FeatureTable ([[RASF]])
* "RSDT": Root System Description Table ([[RSDT]]; 32-bit version of the XSDT)
* "SBST": Smart Battery Specification Table ([[SBST]])
* "SLIT": System Locality System Information Table ([[SLIT]])
* "SRAT": System Resource Affinity Table ([[SRAT]])
* "SSDT": Secondary System Description Table ([[SSDT]])
* "XSDT": Extended System Description Table (This wiki page; included for completeness)

=== Reserved by ACPI ===

The ACPI specification maintains reservations for additional third party defined description tables.  Each third party submitting a request for a reservation also maintains a specification for these tables, located on individual sites.  See the ACPI spec ([https://uefi.org/specifications]) for more information.

== What's next? ==
You should now parse each table pointed by the XSDT. (Probably you will need only the [[FADT]], the [[SSDT]] and the [[MADT]])

[[Category:ACPI]]
````

<a id="w003320"></a>

## W003320 — User:Franciozzy

Bron-ID: W003320
Namespace: User (2)
Revisie: 14374 | Datum: 2013-01-22T14:15:58Z
Bron: https://wiki.osdev.org/User:Franciozzy
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AFranciozzy&oldid=14374

````text
Personal page: [http://www.paradoxo.org www.paradoxo.org]
````

<a id="w003321"></a>

## W003321 — Talk:Raspberry Pi Bare Bones

Bron-ID: W003321
Namespace: Talk (1)
Revisie: 22025 | Datum: 2018-01-04T05:27:49Z
Bron: https://wiki.osdev.org/Talk:Raspberry_Pi_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ARaspberry_Pi_Bare_Bones&oldid=22025

````text
Love4boobies, I needed to change instances of "asm volatile" to "__asm volaktile" even with "-std=c99". And yes, I did compile and test this before editing the wiki. I'm using arm-none-eabi version 2012.03.56 btw. --[[User:Andreasbwagner|Andreasbwagner]] 16:38, 26 January 2013 (CST)

: It should have been -std=gnu99 (or gnu89 even) -- "c99" effectively enables "-ansi" and so asm is indeed unavailable. I believe that the proper keyword to use here is __asm__, not __asm which works fine, but is described as a "compatibility" one in the GCC docs. Or just enable GNU C extensions and use plain "asm". [[User:Alexshpilkin|Alexshpilkin]] 08:54, 30 August 2013 (CDT)

== Issues with toolchain and ABI mentions ==
Correct me if I'm way off here, but are there a few inaccuracies regarding the ABI in this text?

The article states that:
: "QEMU you do not need to objcopy the kernel into a plain binary; QEMU also supports ELF kernels"
Since we're using the '''arm-none-eabi''' toolchain, we're not actually creating ELF executables in the first place. Maybe include some text surrounding what the purpose of using this particular toolchain is, and amend the text about the compiler targeting the '''System V ABI''', isn't the ARM EABI entirely different? Maybe readers might benefit from a minor introductory paragraph to why they're '''not''' using ELF in this application.

This line is also redundant:
  arm-none-eabi-objcopy myos.elf -O binary myos.bin
As well as the script line for Qemu referencing an incorrectly named file. If there are no objections I'd be happy to make the necessary edits to amend these issues!
-- [[User:Ajxs|Ajxs]] 23:27, 3 January 2018 (CST)
````

<a id="w003322"></a>

## W003322 — User:Andreasbwagner

Bron-ID: W003322
Namespace: User (2)
Revisie: 14401 | Datum: 2013-01-26T22:49:05Z
Bron: https://wiki.osdev.org/User:Andreasbwagner
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAndreasbwagner&oldid=14401

````text
[http://www.flyingsquirrelos.org/ FlyingSquirrelOS]
````

<a id="w003323"></a>

## W003323 — SYSENTER

Bron-ID: W003323
Namespace: Main (0)
Revisie: 30835 | Datum: 2026-07-02T03:22:15Z
Bron: https://wiki.osdev.org/SYSENTER
Revisie-URL: https://wiki.osdev.org/index.php?title=SYSENTER&oldid=30835

````text
==Introduction==
The SYSENTER/SYSEXIT instructions (and equivalent SYSCALL/SYSRET on AMD) enable fast entry to the kernel, avoiding interrupt overhead. This is the method used by Windows NT (XP/Vista/7/8) for its API. SYSCALL/SYSRET are covered here as well, but operate slightly differently.

==INTEL: SYSENTER/SYSEXIT==
===Description===
"Executes a fast call to a level 0 system procedure or routine. SYSENTER is a companion instruction to SYSEXIT. <br /> 
The instruction is optimized to provide the maximum performance for system calls from user code running at privilege level 3 to <br />
operating system or executive procedures running at privilege level 0." -- Intel IA-32 (64) programming manual, volume 2B.
===Registers===
====MSRs====
'''These must be accessed through rdmsr and wrmsr'''<br/>
*IA32_SYSENTER_CS (0x174) 
** Contains ring 0 code segment (CS). 
** Ring 0 data = CS + 8. 
** If REX.W prefix is used with SYSEXIT, ring 3 code = CS + 32 and ring 3 data = CS + 40. 
** Otherwise, ring 3 code = CS + 16 and ring 3 data = CS + 24.
<b>These values cannot be changed, therefore your [[GDT]] must be structured as such.</b> 
*IA32_SYSENTER_ESP (0x175) - The kernel's ESP for SYSENTER.
*IA32_SYSENTER_EIP (0x176) - The kernel's EIP for SYSENTER. This is the address of your SYSENTER entry point.

====CPU registers====
'''These must be set by the application, or the C library wrapper'''<br />
*ECX: Ring 3 Stack pointer for SYSEXIT
*EDX: Ring 3 Return address
===Operation===
When SYSENTER is called, CS is set to the value in IA32_SYSENTER_CS. SS is set to IA32_SYSENTER_CS + 8. EIP is loaded from IA32_SYSENTER_EIP and ESP is loaded from IA32_SYSENTER_ESP. The CPU is now in ring 0, with EFLAGS.IF=0, EFLAGS.VM=0, EFLAGS.RF=0.

When SYSEXIT is called, CS is set to IA32_SYSENTER_CS+16. EIP is set to EDX. SS is set to IA32_SYSENTER_CS+24, and ESP is set to ECX.

'''Notes:''' ECX and EDX are not automatically saved as the return address and Stack Pointer. These need to be saved in Ring 3.
====64 bit mode====
Operation in 64 bit mode is exactly the same. IA32_SYSENTER_ESP and IA32_SYSENTER_EIP are extended to 64 bits (get rid of reserved bits).
==AMD: SYSCALL/SYSRET==
===Description===
"SYSCALL and SYSRET are low-latency system call and
return instructions. These instructions assume the operating system implements a flat-memory model,
which greatly simplifies calls to and returns from the operating system. This simplification comes
from eliminating unneeded checks, and by loading pre-determined values into the CS and SS segment
registers (both visible and hidden portions). As a result, SYSCALL and SYSRET can take fewer than
one-fourth the number of internal clock cycles to complete than the legacy CALL and RET
instructions. SYSCALL and SYSRET are particularly well-suited for use in 64-bit mode, which
requires implementation of a paged, flat-memory model." -- AMD System programming

In legacy mode AMD CPUs support SYSENTER/SYSEXIT. However, in long mode only SYSCALL and SYSRET are supported.
===Registers===
====MSRs====
'''These must be accessed through rdmsr and wrmsr'''<br/>
*STAR (0xC0000081) - Ring 0 and Ring 3 Segment bases, as well as SYSCALL EIP. <br />
Low 32 bits = SYSCALL EIP, bits 32-47 are kernel segment base, bits 48-63 are user segment base.
*LSTAR (0xC0000082) - The kernel's RIP SYSCALL entry for 64 bit software.
*CSTAR (0xC0000083) - The kernel's RIP for SYSCALL in compatibility mode.
*SFMASK (0xC0000084) - The low 32 bits are the SYSCALL flag mask. If a bit in this is set, the corresponding bit in rFLAGS is cleared.
===Operation===
NOTE: these instructions assume a flat segmented memory model (paging allowed). They require that "the code-segment base, limit, and attributes (except for CPL) are consistent for all application and system processes." --AMD System programming

SYSCALL loads CS from STAR 47:32. It masks EFLAGS with SFMASK. Next it stores EIP in ECX. It then loads EIP from STAR 32:0 and SS from STAR 47:32 + 8. It then executes.

'''Note that the Kernel does not automatically have a kernel stack loaded. This is the handler's responsibility.'''

SYSRET loads CS from STAR 63:48. It loads EIP from ECX and SS from STAR 63:48 + 8. 

'''Note that the User stack is not automatically loaded. Also note that ECX must be preserved.'''

====64 bit mode====
The operation in 64 bit mode is the same, except that RIP is loaded from LSTAR, or CSTAR of in IA32-e submode (A.K.A. compatibility mode). It also respectively saves and loads RFLAGS to and from R11.

'''In Long Mode, userland CS will be loaded from STAR 63:48 + 16 and userland SS from STAR 63:48 + 8 on SYSRET. You may need to modify your GDT accordingly.'''

Moreover, SYSRET will return to compatibility mode if the operand size is set to 32 bits, which is, for instance, nasm's default. To explicitly request a return into long mode, set the operand size to 64 bits (e.g. "o64 sysret" with nasm).

==Compatibility across Intel and AMD==
For a 32bit kernel, SYSENTER/SYSEXIT are the only compatible pair.  For a 64bit kernel in Long mode only (not Long Compat mode), SYSCALL/SYSRET are the only compatible pair.  For Intel 64bit, IA32_EFER.SCE must be set, or SYSCALL will result in a #UD exception. IA32_EFER is an MSR at 0xC0000080, and SCE (SYSCALL Enable) is its 0th bit.

==Security of SYSRET==
For both AMD and Intel, it is up to the kernel to switch stack back to the userspace stack before executing SYSRET.  This opens a race condition where the NMIs and MCEs exception handlers will be executed on a guest controlled stack.  For 64bit mode, the kernel must use Interrupt Stack Tables to safely move NMIs/MCEs onto a properly designated kernel stack.  For 32bit mode AMD systems, the kernel must use Task Gates for NMIs and MCEs to switch stack.

All Intel CPUs to date (2013) have a silicon bug when executing the SYSRET instruction.  If a non-canonical address is present in RCX when executing SYSRET, a General Protection Fault will be taken in CPL0 with CPL3 registers.  See [http://lists.xen.org/archives/html/xen-announce/2012-06/msg00001.html Xen Security Advisory 7] for more details. 
[[Category:X86 CPU]]
[[de:Syscall/sysret]]

==See Also==

===External Links===

*https://cdrdv2.intel.com/v1/dl/getContent/671200 Page 1882 for SYSRET

[[Category:System Calls]]
````

<a id="w003324"></a>

## W003324 — SYSCALL

Bron-ID: W003324
Namespace: Main (0)
Revisie: 14427 | Datum: 2013-02-04T18:08:06Z
Bron: https://wiki.osdev.org/SYSCALL
Revisie-URL: https://wiki.osdev.org/index.php?title=SYSCALL&oldid=14427
Doorverwijzing: SYSENTER

````text
#REDIRECT [[SYSENTER#AMD: SYSCALL/SYSRET]]
````

<a id="w003325"></a>

## W003325 — SYSEXIT

Bron-ID: W003325
Namespace: Main (0)
Revisie: 14428 | Datum: 2013-02-04T18:09:10Z
Bron: https://wiki.osdev.org/SYSEXIT
Revisie-URL: https://wiki.osdev.org/index.php?title=SYSEXIT&oldid=14428
Doorverwijzing: SYSENTER

````text
#REDIRECT [[SYSENTER]]
````

<a id="w003326"></a>

## W003326 — SYSRET

Bron-ID: W003326
Namespace: Main (0)
Revisie: 14429 | Datum: 2013-02-04T18:09:45Z
Bron: https://wiki.osdev.org/SYSRET
Revisie-URL: https://wiki.osdev.org/index.php?title=SYSRET&oldid=14429
Doorverwijzing: SYSENTER

````text
#REDIRECT [[SYSENTER#AMD: SYSCALL/SYSRET]]
````

<a id="w003327"></a>

## W003327 — Sysenter

Bron-ID: W003327
Namespace: Main (0)
Revisie: 14446 | Datum: 2013-02-07T17:46:25Z
Bron: https://wiki.osdev.org/Sysenter
Revisie-URL: https://wiki.osdev.org/index.php?title=Sysenter&oldid=14446
Doorverwijzing: SYSENTER

````text
#REDIRECT [[SYSENTER]]
````

<a id="w003328"></a>

## W003328 — User:Intx13

Bron-ID: W003328
Namespace: User (2)
Revisie: 18482 | Datum: 2015-11-09T18:30:37Z
Bron: https://wiki.osdev.org/User:Intx13
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AIntx13&oldid=18482

````text
=Intx13's PCI tutorial=


'''Peripheral Component Interconnect''' (PCI) provides protocol, signaling, and physical standards that allow third-party components to be connected to a host system. In the x86 architecture, PCI-connected devices are software-accessible via a combination of port-mapped IO, memory-mapped IO, and interrupts.


==Basics==
===Definitions===
A '''bus''' is a collection of PCI signaling lines. Some of these lines are shared among all components on the bus, such as the address and data lines that carry traffic. Others come in sets with one line available for each possible component on the bus, such as request/grant lines that determine which component gets to use the shared lines and when.

A '''slot''' is a connection point at which a single component can be attached to a bus. A slot can be a slotted connector into which an ''expansion card'' plugs, such as a sound card. A slot can also be location on a motherboard where PCI signals terminate into a ''planar device'', such as an "on-board" network card.

A '''function''' is an independent hardware capability presented as a unique device to the host system. A planar device or expansion card can provide multiple functions simultaneously, which are treated as separate devices by the host system. For example, some manufacturers sell expansion cards that combine a sound card and a modem. The sound card would appear as one function and the modem as another. While they are attached to the same slot on the same bus, they are typically considered two distinct devices because they are accessed independently by software.

Processors do not have a native PCI interface; there is no "read-from-PCI-card" instruction in a processor's instruction set. Instead, a PCI '''host controller''' is integrated into the CPU at a level where it can interface with native IO mechanisms. Software communicates with the host controller using these mechanisms and the host controller takes the appropriate action on the PCI bus on behalf of the software. In the x86 architecture, the host controller is integrated into the Southbridge chip. Software uses a combination of port-mapped IO (the '''in''' and '''out''' instructions) and memory-mapped IO (indirect '''mov''' instructions) to access the host controller.

[[File:PCI-host-controller-in-x86-architecture.png|thumb|center|alt=caption|The PCI bus is quite a ways downstream from the CPU, but appears to be directly accessible to software via PMIO and MMIO.]]

===Geographical addressing===
A device's '''geographical address''' is the bus/slot/function triple that identifies a device with which software wishes to communicate. Buses are numbered '''0-255'''. Slots are numbered '''0-31'''. Functions are numbered '''0-7'''. This means there can be a maximum of 256*32 = 8192 planar devices/expansion cards attached to a single system. If each card utilizes all 8 functions, the system could have up to 8192*8 = 65536 distinct PCI devices. That's a lot!

When a vendor develops a new expansion card, they decide if and how to use function numbers. Function 0 must be used, but functions 1 through 7 are optional. Optional function numbers need not be used in order. For example, a card could implement functions 0, 3, and 7. The slot number assigned to a specific slot on a bus cannot be changed. If an expansion card is moved from one slot to another, its geographical address will change. The bus number assigned to a specific bus is fixed by the host controller and cannot be changed. x86 systems include a single host controller which has at least one bus. One of its buses will always be numbered 0, but the other numbers could be arbitrary.

===Bridges===
A '''bridge''' is a planar device/expansion card that exposes additional PCI buses to the host. For example, a host controller might only implement one bus with a few slots. A bridge (representing a single bus with its own set of slots) could be plugged into one of those slots, allowing for additional cards to be connected. It's not uncommon to find on-board bridges on modern desktop motherboards.

The host controller does not need to be configured by system software in order to know how to communicate across with the devices attached to its slots, because host controllers are integrated into the Southbridge and their bus/slot numbers are fixed at the time of manufacture. However, bridges must be configured at runtime with a bus number that doesn't conflict with any pre-existing bus. Bridges must also be "daisy-chained" to one another, with the host controller itself being daisy-chained to the first bridge. When software tells the host controller to communicate with a device on a bus that the host controller itself does not provide, it must know to which bridge device the traffic must be sent. The traffic "hops" along the chain until the bridge responsible for the bus number accepts it.

[[File:Correct-flow-of-PCI-traffic.png|thumb|center|alt=caption|Bridges daisy-chained from the host controller.]]

(Although it's of little relevance to the operating system developer, the "hopping" of traffic from one device to another doesn't actually require repeated transmission of data, due to the shared data lines that PCI uses.)

===The host controller as a PCI device===
As described previously, the host controller is a distinct hardware component that resides within the Southbridge to provide PCI bus access to software. However, the host controller also sits on the PCI bus like a device! This can be thought of as a "management interface", so that the host controller itself can be configured over PCI. Some host controllers might provide multiple buses. In this case, the host controller acts as ''multiple'' devices on the PCI bus: one for each bus.

A host controller always appears on slot 0 of bus 0. The function numbers of the host controller represent the bus numbers that the host controller supports. Note that this means a host controller can only provide 8 buses, numbered 0-7. The system can have up to 256 buses, so the other 248 can only be provided by bridges. Software can communicate with the host controller at geographical addresses 0/0/* to configure the individual buses the host controller supports.

[[File:Multi-bus-PCI-host-controller-with-multiple-bridges.png|thumb|center|alt=caption|Things get complicated when the PCI host controller supports multiple buses which might have multiple bridges.]]


===Thank goodness for BIOS===
Early configuration of the PCI bus includes assigning bus numbers to all bridges and daisy-chaining them together. It also includes BAR mapping, which we will get into later. Luckily, BIOS will do all of this at start-up. Operating systems may choose to reconfigure the PCI bus by reassigning bus numbers, re-daisy-chaining bridges, and remapping BARs, but if BIOS can be trusted this isn't necessary. Linux does not bother to reconfigure PCI bridges or remap BARs. TODO: What about BSD? Windows? Should be easy to check; compare their "lspci" equivalents to what Linux reports on the same system.

The remainder of this document will therefore not cover the configuration of bridges or host controllers or BAR remapping.


===Learn to love ''lspci''===
The '''lspci''' Linux program retrieves information from the kernel about the PCI buses in the system. If executed as '''lspci -n''', the program will display geographical addresses, class codes, and vendor/product/revision IDs for all configured PCI devices. Because Linux sticks with whatever bridge configuration and BAR mapping BIOS arranged, '''lspci''' tells you what your code ''should'' see. The output of '''lspci -n''' is formatted as below.

  xx:yy.z ccss: vvvv:pppp (rev rr)
  
  xx    Bus number
  yy    Slot number
  z     Function number
  
  cc    Class code
  ss    Sub-class code
  
  vvvv  Vendor ID
  pppp  Product ID
  rr    Revision ID

If you haven't read the rest of this document, class codes and vendor/product/revision IDs aren't something you're interested in just yet. However, you can learn some interesting information about the layout of the PCI buses just by looking at the geographical addresses.

  $ lspci -n
  00:00.0 0600 8086:7190 (rev 01)
  00:01.0 0604 8086:7191 (rev 01)
  ...
  02:00.0 0200 8086:100f (rev 01)

In the above example, we see right away that the system's host controller only provides one bus. We know this because the host controller will appear as a PCI device on bus 0 slot 0 with a function for every bus it supports, but this system only has a single device on bus 0 slot 0 (namely, geographical address 00:00.0). At the same time we see a device (02:00.0) whose geographical address indicates it is attached to bus 2. This means there must be a bridge in the system. As it happens, the second device listed (geographical address 00:01.0) is that bridge. Using only the geographical addresses we can put together a map of the host controller, the buses it provides, and any other buses that are provided by bridges, along with the number of slots and functions that are used on each bus. Until we learn a little more, however, we won't know which devices are those bridges and which devices are other useful things, like network cards.

==Configuration Space==
Every PCI device has a set of internal registers called its '''configuration space'''. The host controller provides port-mapped IO that allows software to read and write registers within each device's configuration space. This port-mapped IO is called the '''Configuration Access Mechanism''', or '''CAM'''. The contents of a device's configuration space includes many useful fields that describe the purpose of the device. The configuration space also includes '''BARs''', which tells us how to interact with the device beyond CAM. To put it shortly, CAM retrieves "header" information that can be parsed without knowing the details of the device, while BARs would be used by drivers to interact with the guts of the device.


===Configuration Access Mechanism===
The host controller implements CAM through PIO ports '''0x0CF8''' and '''0x0CFC'''. Writing a 32-bit value to port 0x0CF8 informs the host controller which device (by geographical address) and which register number within that device we are interested in reading or writing. Reading from port 0x0CFC will cause the host controller to read that register in that device's configuration space and return it to software. Writing to port 0x0CFC will cause the host controller to write to that register in that device's configuration space.

(figure here, illustrating this standard method of accessing "shadowed" registers)

The 32-bit value written to port 0x0CF8 breaks down as follows.

(figure here)

The following pseudocode selects a register within a device by geographical address.
  '''cam_select''': '''bus''', '''slot''', '''function''', '''register'''
  
  '''val32''' = 
       0x80000000
     | '''bus''' << 16
     | '''slot''' << 11
     | '''function''' << 8
     | '''register''' << 2
  
  out(0x0CF8, '''val32''')


The following pseudocode reads 32-bit, 16-bit, and 8-bit registers from within a device.
  '''cam_read_32''': '''bus''', '''slot''', '''function''', '''register''' -> '''val'''
  
  cam_select('''bus''', '''slot''', '''function''', '''register''')
  '''val''' = in(0x0CFC)

  '''cam_read_16_0''': '''bus''', '''slot''', '''function''', '''register''' -> '''val'''
  
  '''val32''' = cam_read_32('''bus''', '''slot''', '''function''', '''register''')
  '''val''' = '''val32'''(15:0)

  '''cam_read_16_1''': '''bus''', '''slot''', '''function''', '''register''' -> '''val'''
  
  '''val32''' = cam_read_32('''bus''', '''slot''', '''function''', '''register''')
  '''val''' = '''val32'''(31:16)

  '''cam_read_8_0''': '''bus''', '''slot''', '''function''', '''register''' -> '''val'''
  
  '''val32''' = cam_read_32('''bus''', '''slot''', '''function''', '''register''')
  '''val''' = '''val32'''(7:0)

  '''cam_read_8_1''': '''bus''', '''slot''', '''function''', '''register''' -> '''val'''
  
  '''val32''' = cam_read_32('''bus''', '''slot''', '''function''', '''register''')
  '''val''' = '''val32'''(15:8)

  '''cam_read_8_2''': '''bus''', '''slot''', '''function''', '''register''' -> '''val'''
  
  '''val32''' = cam_read_32('''bus''', '''slot''', '''function''', '''register''')
  '''val''' = '''val32'''(23:16)

  '''cam_read_8_3''': '''bus''', '''slot''', '''function''', '''register''' -> '''val'''
  
  '''val32''' = cam_read_32('''bus''', '''slot''', '''function''', '''register''')
  '''val''' = '''val32'''(31:24)


===A normal device's configuration space===
TODO


===A bridge's configuration space===
TODO


===A host controller's configuration space===
TODO


==Base Address Registers==
TODO
===Port-mapped IO with PCI devices===
TODO

===Memory-mapped IO with PCI devices===
TODO


==Interrupts==
TODO



=Recursive descent AML parser=
The latest ACPI specification defines TODO AML objects. Most of them are just wrappers/containers for other objects. For example, a TermObj is either a NameSpaceModifierObj, a NamedObj, a Type1Opcode, or a Type2Opcode. And a NameSpaceModifierObj is either a DefAlias, a DefName, or a DefScope. And so on.

The goal of an AML parser is to transform a blob of AML bytecode in memory into a parse tree, which can subsequently be walked to look for various types of objects. Pages 814 through 825 of the ACPI specification provide the formal grammar for every AML object. For example, a null-terminated ASCII string is defined as follows.

<source lang="c">
String        := StringPrefix AsciiCharList NullChar
StringPrefix  := 0x0D
AsciiCharList := Nothing | <AsciiChar AsciiCharList>
AsciiChar     := 0x01 - 0x7F
NullChar      := 0x00
</source>

You can read this out loud as below:
<blockquote>A String consists of three objects: a StringPrefix followed by an AsciiCharList followed by a NullChar. A StringPrefix is a single byte with value 0x0D. An AsciiCharList is either Nothing or an AsciiChar followed by another AsciiCharList. An AsciiChar is a single byte with value between 0x01 and 0x7F, inclusive. A NullChar is a single byte with value 0x00.</blockquote>

If you wanted to represent the string "cat" in AML, you would build a parse tree like this:
<source lang="c">
String
  StringPrefix (0x0D)
  AsciiCharList
    AsciiChar (0x63)
    AsciiCharList
      AsciiChar (0x61)
      AsciiCharList
        AsciiChar (0x74)
        AsciiCharList
          Nothing
  NullChar (0x00)
</source>

The resulting bytecode would look like this:
<source lang="c">
0D 63 61 74 00
</source>

The job of a parser (for the String object anyway) is to take that bytecode and produce the corresponding parse tree.

==Recursive descent parsing==
Many AML objects (like String) are defined in terms of other objects. Others (like AsciiChar and NullChar) are defined in terms of bytes consumed from a blob of AML. This suggests that a "recursive descent" parser might be a good choice for parsing AML. When writing a recursive descent parser you write a parsing function for every object. Many of those parsers (like the parser for String) just call other parsers and collect their results, while some parsers (like AsciiChar and NullChar) actually read bytes from a blob of AML.

Below is a pseudocode example of a recursive descent parser for String and related objects. In this example and throughout this document, ''blob'' is some global object from which bytes of AML can be read. We'll start by writing the simple parsers: the ones that aren't defined in terms of any other objects.

<source lang="c">
function ParseStringPrefix(blob):
  if blob.NextByte == 0x0D:
    blob.ConsumeByte()
    return StringPrefix
  else:
    return null

function ParseAsciiChar(blob):
  if     blob.NextByte >= 0x01
     and blob.NextByte <= 0x7F:
    byte x = blob.ConsumeByte()
    return AsciiChar(x)
  else:
    return null

function ParseNullChar(blob):
  if blob.NextByte == 0x00:
    blob.ConsumeByte()
    return NullChar
  else:
    return null

function ParseNothing(blob):
  return Nothing
</source>


Note how ParseStringPrefix and ParseNullChar just check whether the next byte in the blob is the value they're expecting, and if so, consume it and return the appropriate object. ParseAsciiChar does the same, but it also attaches the consumed byte to the object being returned, so that the parse tree will contain the characters from the string. The parser for Nothing is simple: it always returns a Nothing object, without consuming any bytes of AML. Now let's write a parser for AsciiCharList, which is a little trickier, because it's defined recursively in terms of itself.

<source lang="c">
function ParseAsciiCharList(blob):
  a = ParseAsciiChar(blob)
  if a:
    b = ParseAsciiCharList(blob)
    if b:
      return AsciiCharList(a, b)
    else:
      return null
  else:
    return ParseNothing(blob)
</source>

ParseAsciiCharList first calls ParseAsciiChar. If that parser succeeded (meaning that the next byte in the blob was an ASCII character and it's been returned inside of an AsciiChar object) then ParseAsciiCharList calls itself recursively. It then returns an AsciiCharList object consisting of the AsciiChar object and whatever the recursive call returns. If the ParseAsciiChar parser failed (meaning that the next byte in the blob was ''not'' an ASCII character) then ParseAsciiCharList calls the ParseNothing parser and returns whatever it returns. This is a tail-recursive function.

Finally, let's write the parser for the String object.

<source lang="c">
function ParseString(blob):
  a = ParseStringPrefix(blob):
  if a:
    b = ParseAsciiCharList(blob):
    if b:
      c = ParseNullChar(blob):
      if c:
        return String(a, b, c)
      else:
        return null
    else:
      return null
  else:
    return null
</source>

ParseString doesn't touch the contents of the AML blob at all. It just calls three other parsers and if they all succeed, it returns a String object containing them.

We could continue along this path, writing recursive descent parsers for every AML object. The syntax of the formal grammar by which AML objects are defined makes this very straightforward, although somewhat tedious. Unfortunately, there are two big problems with writing a recursive descent AML parser.

==Fixed, implicit and explicit-length objects==
AML objects can be sorted into three types: fixed-length, implicit-length, and explicit-length.

===Fixed-length objects===
A fixed-length object is an object that always consumes the same number of bytes of AML (if it exists in the AML, of course.) AsciiChar and NullChar are examples of fixed-length objects; if present, they each consume a single byte. Fixed-length objects are the easiest to parse, because you simply consume the required number of bytes and make sure they are correct for the type of object being parsed.

===Implicit-length objects===
Implicit-length objects can take up different numbers of bytes of AML. AsciiCharList and String are examples of implicit-length objects. When parsing a String, you don't know how many bytes of AML you are ultimately going to consume. When parsing an AsciiCharList you don't know how deep the call to ParseAsciiCharList is going to recurse. For some AML objects, like String and AsciiCharList, you don't ''care'' how many bytes you'll ultimately consume. You simply call the parsing function and it happily recurses until eventually the recursion terminates (for example, when ParseAsciiChar fails to find any more ASCII characters and so ParseAsciiCharList calls ParseNothing instead). These "don't-care" implicit-length objects are easy to parse using recursion.

For some implicit-length objects, however, you ''have'' to know when to stop parsing, because there's no "end point" or "terminating byte" that causes recursion to stop. The AML objects that behave like this are too complex to look at right now, so instead we'll make up an object that contains a list of zeroes. Here is the grammar for this made-up object.

<source lang="c">
ZeroList := Nothing | <Zero ZeroList>
Zero     := 0x00
</source>

Now suppose the AML bytecode that you wanted to parse consisted of two bytes:
<source lang="C">
00 00
</source>

Now ask yourself, is this a single ZeroList containing two Zero objects? Or is ''two'' ZeroLists, each containing ''one'' Zero object? Does the parse tree look like this:
<source lang="C">
ZeroList
  Zero
  ZeroList
    Zero
    ZeroList
      Nothing
</source>

Or this:
<source lang="C">
ZeroList
  Zero
  ZeroList
    Nothing
ZeroList
  Zero
  ZeroList
    Nothing
</source>

Both of those are perfectly valid ways to parse that AML bytecode. Note that AsciiCharList avoids this confusion because it only ever appears within a String, and it's book-ended by a StringPrefix on one side, and a NullChar on the other. But there are quite a few real AML objects that behave just like our ZeroList example.

When parsing these "must-know" implicit-length objects, the parser needs to know how many bytes it should parse before stopping. The parsers for Zero and ZeroList could look like this.
<source lang="C">
function ParseZero(blob)
  if blob.NextByte == 0x00:
    return Zero
  else:
    return null

function ParseZeroList(blob, count):
  if count == 0:
    return Nothing
  else:
    a = ParseZero(blob)
    if a:
      return ZeroList(a, ParseZeroList(blob, count - 1))
    else:
      return null
</source>

Notice how ParseZeroList decrements the "count" variable before recursing, and if count ever reaches zero it knows that it's time to apply the "end-of-recursion" case.

But where does the "count" variable come from? Well, "must-know" implicit-length objects always appear inside of explicit-length objects, which we will discuss next.

===Explicit-length objects===
Explicit-length objects are objects whose AML bytecode includes the actual total length of the object (and of any sub-objects it contains). The total length of an object is stored in a PkgLength object. We won't look at the encoding for a PkgLength object right now, but it's usually a single byte that says how many bytes are in the current object, from the PkgLength onwards. We'll make up another object to illustrate this.

<source lang="c">
Happy       := HappyPrefix PkgLength ZeroList
HappyPrefix := 0xFF
</source>

Like our made-up Happy object, real AML explicit-length objects always include a PkgLength object, typically as the second object, just after an "opcode" style object and just before a "must-know" implicit-length list. Suppose we were parsing the following AML.

<source lang="c">
FF 03 00 00 00
</source>

We begin executing a parsing function for Happy, which might look something like this:

<source lang="c">
function ParseHappy(blob):
  a = ParseHappyPrefix(blob)
  if a:
    b = ParsePkgLength(blob)
    count = b.Value - 1
    if b:
      c = ParseZeroList(count)
      if c:
        return Happy(a, b, c)
      else:
        return null
    else:
      return null
  else:
    return null
</source>

First we call ParseHappyPrefix, which consumes the first byte of the blob, 0xFF. Then we call ParsePkgLength, which consumes the second byte of the blob, 0x03. We take the value consumed and subtract one to account for the length of the PkgLength object itself. Then we call ParseZeroList, passing along the remaining number of bytes to be consumed.

With one exception, every "must-know" implicit-length AML object appears within an explicit-length object (although it might be nested rather deep within it). That one exception is the other big difficulty in parsing AML, to be addressed next.

==Object references==
AML objects are often referenced by-name within other AML objects. The MethodInvocation object is used to represent object references and calls to methods defined within the AML. (A simple object reference is the same as a method that doesn't take any arguments.) We're not going to look at the DefMethod object right now, which defines a method to be invoked, but it specifies the name of the method, the number of arguments it takes, and a list of objects - usually math or comparison operations - to be performed on them. A simplified version of the MethodInvocation object is defined below.

<source lang="c">
MethodInvocation := String TermArgList
TermArgList      := Nothing | <TermArg TermArgList>
TermArg          := MethodInvocation
...
</source>

From the grammar we can see that a MethodInvocation consists of the name of the method to be invoked, followed by a list of arguments passed to the method. Each argument is itself a MethodInvocation. A simple object reference has no arguments, so its TermArgList is Nothing.

Unfortunately, while TermArgList is a "must-know" implicit-length object, MethodInvocation is ''not'' an explicit-length object. For example, consider the following AML.
<source lang="c">
0D 63 61 74 00 0D 64 6F 67 00 0D 66 6F 6F 00 0D 62 61 72 00
</source>

This AML consists of four strings in a row: "cat", "dog", "foo", and "bar". So is this an invocation of the "cat" method, taking three arguments, which are references to the "dog", "foo", and "bar" objects...
<source lang="c">
cat(dog, foo, bar)
</source>

...or is it an invocation of the "cat" method, taking ''two'' arguments, the first of which is a reference to the "dog" object, and the second is an invocation of the "foo" method, which itself takes a single argument, the "bar" object...
<source lang="c">
cat(dog, foo(bar))
</source>

...or is it something else? There is no way to know how to parse a MethodInvocation without knowing whether an object is a simple object or a method, and if a method, how many arguments it requires. A parser might look like this.
<source lang="c">
function ParseMethodInvocation(blob):
  a = ParseString(blob)
  if a:
    if IsMethod(a):
      num_args = GetNumArgs(a)
      b = list[num_args]
      for i = 0 to num_arguments - 1:
        b[i] = ParseMethodInvocation(blob)
      return MethodInvocation(a, b)
    else:
      return MethodInvocation(a)
  else:
    return null
</source>

So where do the IsMethod and GetNumArgs functions get their information? The parser must store this information somewhere whenever a named object is parsed, as illustrated below for the DefMethod object.
<source lang="c">
function ParseDefMethod(blob):
  // Not actually showing how to parse a DefMethod, but it includes
  // the name of the method and the number of arguments the method
  // requires.
  SetIsMethod(name)
  StoreNumArgs(name, num_arguments)
  return DefMethod(...)
</source>

But what if the corresponding named object is defined ''after'' the MethodInvocation? These kinds of "forward references" mean that you cannot parse AML in a single pass. Whenever you encounter a MethodInvocation but have not yet encountered an object with the name of the method being invoked the parser must stop and backtrack all the way back to the nearest explicit-length object. All the AML for that object must be stored for later parsing.

An example will make this clearer. Consider the following grammar for some made-up objects.
<source lang="c">
Funky       := FunkyPrefix PkgLength Goofy
FunkyPrefix := 0xAA
Goofy       := GoofyPrefix MethodInvocation
GoofyPrefix := 0xBB
</source>

Consider the following AML.
<source lang="c">
AA 07 BB 0D 62 61 72 00 01 02 03 04...
</source>

Clearly we have a FunkyPrefix followed by a PkgLength with value 0x07, which tells us that the next 6 bytes are part of the Funky object. The partial parse tree might look like this.
<source lang="c">
Funky
  FunkyPrefix
  PkgLength (0x07)
  (Unparsed AML: BB 0D 62 61 72 00 01 02 03 04 ...)
</source>

Continuing on, we have a GoofyPrefix and we expect there to be a MethodInvocation afterwards. We parse the String and find that the method to be invoked is named "bar". Now our partial parse tree might look like this.
<source lang="c">
Funky
  FunkyPrefix
  PkgLength (0x07)
  Goofy
    GoofyPrefix
    MethodInvocation
      String "bar"
      (Unparsed AML: 01 02 03 04 ...)
</source>

Suppose we look up "bar" in our database of methods, but we don't find anything - clearly we haven't found an object with the name "bar" yet. This means we can't finish parsing the MethodInvocation. If we can't finish parsing the MethodInvocation, we can't finish parsing the Goofy object either. And if we can't finish parsing the Goofy object we can't finish parsing the Funky object. However, the Funky object is an explicit-length object, so we know how much AML is inside it. So we can wrap that AML up in a special object temporarily, and continue parsing. Now our parse tree looks like this.
<source lang="c">
Funky
  FunkyPrefix
  PkgLength (0x07)
  AML wrapper {BB 0D 62 61 72 00}
(Unparsed AML: 01 02 03 04 ...)
</source>

Suppose we continue parsing the unparsed AML and eventually we find a DefMethod object with name "bar", and it says that the "bar" method doesn't take any arguments. Now we can revisit the AML wrapped inside the Funky object and parse it again. We work our way back to the MethodInvocation and this time we know that there's nothing expected after the "bar" method name. Our completely parsed Funky object looks like this.
<source lang="c">
Funky
  FunkyPrefix
  PkgLength (0x07)
  Goofy
    GoofyPrefix
    MethodInvocation
      String "bar"
</source>

This isn't too bad, right? You just have to keep a registry of method and object names as you parse them, and wait until you've parsed all of them before you attempt to parse any MethodInvocations.

Unfortunately, in this document so far we've described method/object names as Strings, but that's not the case. Instead, method/object names are NameString objects, which are more like paths in a filesystem. To find the corresponding DefMethod (or other named object definition) for a particular MethodInvocation you have to understand the ACPI namespace and AML scoping. Have faith: this is the last AML concept needed to completely parse arbitrary AML with a recursive descent parser.

==ACPI namespace and scoping==
TODO

==AML errata==
The ACPI specification includes a number of mistakes in the AML grammar.
TODO: summarize all the mistakes I've found
````

<a id="w003329"></a>

## W003329 — File:Lspci-with-multifunction-device-highlighted.png

Bron-ID: W003329
Namespace: File (6)
Revisie: 14472 | Datum: 2013-02-20T20:06:32Z
Bron: https://wiki.osdev.org/File:Lspci-with-multifunction-device-highlighted.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ALspci-with-multifunction-device-highlighted.png&oldid=14472

````text
This is the output of the "lspci" command on a Debian installation. The highlighted entries illustrate a multi-function PCI device that uses functions 0 and 1 on bus 0, slot 7.
````

<a id="w003330"></a>

## W003330 — File:PCI-host-controller-as-software-interface.png

Bron-ID: W003330
Namespace: File (6)
Revisie: 14475 | Datum: 2013-02-20T20:28:04Z
Bron: https://wiki.osdev.org/File:PCI-host-controller-as-software-interface.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3APCI-host-controller-as-software-interface.png&oldid=14475

````text
In modern systems the PCI host controller is often integrated into the CPU and exposes PCI buses to software via some CPU-specific IO mechanism, such as memory-mapped IO. This is in contrast with USB, where the USB host controller is connected to the CPU via some other bus (such as PCI) and software communicates with it over that other bus as opposed to using native CPU IO.
````

<a id="w003331"></a>

## W003331 — File:PCI-host-controller-in-x86-architecture.png

Bron-ID: W003331
Namespace: File (6)
Revisie: 14476 | Datum: 2013-02-20T20:34:50Z
Bron: https://wiki.osdev.org/File:PCI-host-controller-in-x86-architecture.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3APCI-host-controller-in-x86-architecture.png&oldid=14476

````text
In the x86 architecture, the PCI host controller is integrated into the Southbridge. The Processor/Northbridge/Southbridge, produced by a single vendor, exposes the PCI host controller (and, by extension, the PCI buses) to software via port-mapped IO, memory-mapped IO, and interrupts.
````

<a id="w003332"></a>

## W003332 — File:PCI-host-controller-and-single-bridge.png

Bron-ID: W003332
Namespace: File (6)
Revisie: 14478 | Datum: 2013-02-20T20:48:19Z
Bron: https://wiki.osdev.org/File:PCI-host-controller-and-single-bridge.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3APCI-host-controller-and-single-bridge.png&oldid=14478

````text
In this illustration, a PCI host controller has three attached devices. One of those devices is a bridge, which has two attached devices. Traffic destined for one of the devices attached to the bridge flows from the host controller to the bridge and then to the device.
````

<a id="w003333"></a>

## W003333 — File:Incorrect-flow-of-PCI-traffic.png

Bron-ID: W003333
Namespace: File (6)
Revisie: 14479 | Datum: 2013-02-20T21:00:34Z
Bron: https://wiki.osdev.org/File:Incorrect-flow-of-PCI-traffic.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AIncorrect-flow-of-PCI-traffic.png&oldid=14479

````text
Sadly this is NOT how traffic flows from a PCI host controller to devices connected to bridges. However, it simplifies things to start with this figure when introducing bridges.
````

<a id="w003334"></a>

## W003334 — File:Correct-flow-of-PCI-traffic.png

Bron-ID: W003334
Namespace: File (6)
Revisie: 14480 | Datum: 2013-02-20T21:04:17Z
Bron: https://wiki.osdev.org/File:Correct-flow-of-PCI-traffic.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ACorrect-flow-of-PCI-traffic.png&oldid=14480

````text
This figure illustrates how traffic really flows from the host controller to bridges and the devices they host. Bridges are daisy-chained until there is a single path from the host controller through every bridge.
````

<a id="w003335"></a>

## W003335 — File:Multi-bus-PCI-host-controller-with-multiple-bridges.png

Bron-ID: W003335
Namespace: File (6)
Revisie: 14481 | Datum: 2013-02-20T21:15:27Z
Bron: https://wiki.osdev.org/File:Multi-bus-PCI-host-controller-with-multiple-bridges.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AMulti-bus-PCI-host-controller-with-multiple-bridges.png&oldid=14481

````text
This figure illustrates a system with a PCI host controller that supports two buses, one of which has two bridges. To ensure that the two bridged expansion cards are accessible, a daisy-chain would need to be set up from the PCI host controller (either its bus 0 or bus 1 function) to one of the bridges and then to the other.
````

<a id="w003336"></a>

## W003336 — Calling Global Constructors

Bron-ID: W003336
Namespace: Main (0)
Revisie: 28110 | Datum: 2023-07-09T22:43:14Z
Bron: https://wiki.osdev.org/Calling_Global_Constructors
Revisie-URL: https://wiki.osdev.org/index.php?title=Calling_Global_Constructors&oldid=28110

````text
This tutorial discusses how to correctly invoke global constructors, such as those on global C++ objects. These are supposed to have run before your main function, which is why the program entry point is normally a function called _start. This function has the responsibility of parsing the command line arguments, initializing the standard library (memory allocation, signals, ...), running the global constructors and finally exit(main(argc, argv)). Things may be different on your operating system if you change your compiler, but if you are using the GNU Compiler Collection (gcc) it may be wise to follow the System V ABI.

On most platforms, the global constructors/destructors are stored in a sorted array of function pointers and invoking these is as simple as traversing the array and running each element. However, the compiler does not always give access to this list, and some compilers consider this implementation details. In this case you will have to cooperate with the compiler - fighting the compiler will only cause trouble.

== GNU Compiler Collection - System V ABI ==

The System V ABI (as used by <tt>i686-elf-gcc</tt>, <tt>x86_64-elf-gcc</tt>, and other ELF platforms) specifies use of five different object files that together handle program initialization. These are traditionally called <tt>crt0.o</tt>, <tt>crti.o</tt>, <tt>crtbegin.o</tt>, <tt>crtend.o</tt>, and <tt>crtn.o</tt>. Together these object files implement two special functions: <tt>_init</tt> which runs the global constructors and other initialization tasks, and <tt>_fini</tt> that runs the global destructors and other termination tasks.

This scheme gives the compiler great control over program initialization and makes things easy for you, but you have to cooperate with the compiler, otherwise bad things will happen. Your cross-compiler will provide you with <tt>crtbegin.o</tt> and <tt>crtend.o</tt>. These files contain the internals that the compiler wishes to hide from you, but wants you to use. To get access to this information, you will need to provide your own implementation of <tt>crti.o</tt> and <tt>crtn.o</tt>. Fortunately, this is easy and is described in detail in this tutorial. The fifth file <tt>crt0.o</tt> contains the program entry point (normally <tt>_start</tt>) and calls the special <tt>_init</tt> function that runs the "program initialization tasks" that <tt>crti.o</tt>, <tt>crtbegin.o</tt>, <tt>crtend.o</tt>, and <tt>crtn.o</tt> together form, and your exit function will normally call the <tt>_fini</tt> function made by these objects. However, <tt>crt0</tt>.o is out of scope of this article. (Note that the object file that contains <tt>_start</tt> acts as <tt>crt0.o</tt> in a kernel.)

To understand this apparent complexity, consider a program consisting of <tt>foo.o</tt> and <tt>bar.o</tt> that is being linked:
<source lang="bash">i686-elf-gcc foo.o bar.o -o program</source>

The compiler will rewrite the command line and pass it to the linker as:

<source lang="bash">i686-elf-ld crt0.o crti.o crtbegin.o foo.o bar.o crtend.o crtn.o</source>

The idea is that the these files together form the <tt>_init</tt> and <tt>_fini</tt> functions during the linking. This is done by storing the <tt>_init</tt> function in the <tt>.init</tt> section, and the <tt>_fini</tt> function in the <tt>.fini section</tt>. Each file then contributes a bit to these sections and the linker makes glues together the fragments in the code specified on the command line. <tt>crti.o</tt> provides the function header, <tt>crtbegin.o</tt> and <tt>crtend.o</tt> provide the body, and <tt>crtn.o</tt> provide the footer (return statement). It is important to understand that the link order matters and strange things may happen if the objects is not exactly linked in this order.

=== Using global constructors from C ===
As a special extension, GCC allows C programs to run functions as global constructors. For more information, consult the compiler documentation. This is normally used as:

<source lang="c">
__attribute__ ((constructor)) void foo(void)
{
	printf("foo is running and printf is available at this point\n");
}

int main(int argc, char* argv[])
{
	printf("%s: main is running with argc=%i\n", argv[0], argc);
}
</source>

=== Using crti.o, crtbegin.o, crtend.o, and crtn.o in a Kernel ===

In a kernel, you are not using a user-space C library. You may be using a special kernel "C library", or none at all. The compiler always supplies <tt>crtbegin.o</tt> and <tt>crtend.o</tt>, but normally the C library supplies <tt>crti.o</tt> and <tt>crtn.o</tt>, but not in this case. The kernel should supply its own <tt>crti.o</tt> and <tt>crtn.o</tt> implementation (even if it would be otherwise identical to the user-space libc version). A kernel is linked with <tt>-nostdlib</tt> (which is the same as passing <tt>-nodefaultlibs</tt> and <tt>-nostartfiles</tt>) which disables the "start files" <tt>crt*.o</tt> that are normally automatically added to the link command line. By passing <tt>-nostartfiles</tt>, we promise to the compiler that we take on the responsibility ourselves to call the "program initialization tasks" in the <tt>crtbegin.o</tt> and <tt>crtend.o</tt> files. This means as we need to manually add <tt>crti.o</tt>, <tt>crtbegin.o</tt>, <tt>crtend.o</tt>, and <tt>crtn.o</tt> to the command line. Since we provide <tt>crti.o</tt> and <tt>crtn.o</tt> ourselves, that is trivial to add to the kernel command line. However, since <tt>crtbegin.o</tt> and <tt>crtend.o</tt> are installed inside a compiler-specific directory, we'll need to figure out the path. Luckily, gcc offers an option just to do this. If <tt>i686-elf-gcc</tt> is your cross-compiler and <tt>$CFLAGS</tt> is the flags you would normally provide to your compiler, then

<source lang="bash">i686-elf-gcc $CFLAGS -print-file-name=crtbegin.o</source>

will make the compiler print the path to the correct <tt>crtbegin.o</tt> file (that is ABI compatible with the $CFLAGS options) to the standard output. The same works with <tt>crtend.o</tt>. If you are using GNU Make, you can do it easily in your makefile assuming <tt>$(CC)</tt> is your cross-compiler and <tt>$(CFLAGS)</tt> is the flags you would normally pass it:

<source lang="make">
CRTBEGIN_OBJ:=$(shell $(CC) $(CFLAGS) -print-file-name=crtbegin.o)
CRTEND_OBJ:=$(shell $(CC) $(CFLAGS) -print-file-name=crtend.o)
</source>

You can then use them as such (adapted to your real build system):

<source lang="make">
OBJS:=foo.o bar.o

CRTI_OBJ=crti.o
CRTBEGIN_OBJ:=$(shell $(CC) $(CFLAGS) -print-file-name=crtbegin.o)
CRTEND_OBJ:=$(shell $(CC) $(CFLAGS) -print-file-name=crtend.o)
CRTN_OBJ=crtn.o

OBJ_LINK_LIST:=$(CRTI_OBJ) $(CRTBEGIN_OBJ) $(OBJS) $(CRTEND_OBJ) $(CRTN_OBJ)
INTERNAL_OBJS:=$(CRTI_OBJ) $(OBJS) $(CRTN_OBJ)

myos.kernel: $(OBJ_LINK_LIST)
	$(CC) -o myos.kernel $(OBJ_LINK_LIST) -nostdlib -lgcc

clean:
	rm -f myos.kernel $(INTERNAL_OBJS)
</source>

It is important to remember that the objects must be linked in this exact order, or you will experience strange bugs.

Your kernel will then have an <tt>_init</tt> and a <tt>_fini</tt> function linked in, which can be called from your <tt>boot.o</tt> (or what your kernel entry point object is called) before passing control to <tt>kernel_main</tt> (or what your kernel main routine is called). Please note that the kernel may not be initialized, at all, at this point in time and you can only do trivial things from your global constructors. In addition, <tt>_fini</tt> may not ever get called because your operating system will remain running, and when it is time to shut down, there is little worth doing that a processor reset won't do. It may be worth setting up a <tt>kernel_early_main</tt> function that initializes the heap, the log, and other core kernel features. Then your <tt>boot.o</tt> can call <tt>kernel_early_main</tt>, then call <tt>_init</tt>, and then finally pass control to the real <tt>kernel_main</tt>. This is analogous to how things work in user-space, where <tt>crt0.o</tt> calls <tt>_initialize_c_library</tt> (or what you call it), then <tt>_init</tt>, and finally <tt>exit(main(argc, argv))</tt>.

=== Using crti.o, crtbegin.o, crtend.o, and crtn.o in User-Space ===
{{Main|Creating a C Library}}

It is very easy to use these object files in user-space, as the cross-compiler automatically will link them in the right order into the final program. The compiler will, as always, provide <tt>crtbegin.o</tt> and <tt>crtend.o</tt>. Your C library will then provide <tt>crt0.o</tt> (program entry point file), <tt>crti.o</tt>, and <tt>crtn.o</tt>. If you have an [[OS Specific Toolchain]], you can change the name of the program entry point (normally _start), the path where the compiler searches for the <tt>crt{0,i,n}.o</tt> files, and what files is even used (possibly with other names) and what order, by modifying <tt>STARTFILE_SPEC</tt> and <tt>ENDFILE_SPEC</tt>. When you start creating a user-space, it may well be worth creating an OS Specific Toolchain because it allows you great control over exactly how all this works.

=== x86 (32-bit) ===
It is very simple to implement this under x86. You simply have to define the header of two functions in <tt>crti.o</tt> and the footer in <tt>crtn.o</tt> and use these objects in your C library or kernel. You can then simply call <tt>_init</tt> to perform the initialization tasks and call <tt>_fini</tt> to perform the termination tasks (normally done from a <tt>crt0.o</tt> or <tt>my-kernel-boot-object.o</tt>).

<pre>
/* x86 crti.s */
.section .init
.global _init
.type _init, @function
_init:
	push %ebp
	movl %esp, %ebp
	/* gcc will nicely put the contents of crtbegin.o's .init section here. */

.section .fini
.global _fini
.type _fini, @function
_fini:
	push %ebp
	movl %esp, %ebp
	/* gcc will nicely put the contents of crtbegin.o's .fini section here. */
</pre>

<pre>
/* x86 crtn.s */
.section .init
	/* gcc will nicely put the contents of crtend.o's .init section here. */
	popl %ebp
	ret

.section .fini
	/* gcc will nicely put the contents of crtend.o's .fini section here. */
	popl %ebp
	ret
</pre>

=== x86_64 (64-bit) ===

The system ABI on x86_64 is similar to its 32-bit counterpart and we also just need to provide function headers and function footers and the compiler will insert the rest of the <tt>_init</tt> and <tt>_fini</tt> functions through <tt>crtbegin.o</tt> and <tt>crtend.o</tt>.

<pre>
/* x86_64 crti.s */
.section .init
.global _init
.type _init, @function
_init:
	push %rbp
	movq %rsp, %rbp
	/* gcc will nicely put the contents of crtbegin.o's .init section here. */

.section .fini
.global _fini
.type _fini, @function
_fini:
	push %rbp
	movq %rsp, %rbp
	/* gcc will nicely put the contents of crtbegin.o's .fini section here. */
</pre>

<pre>
/* x86_64 crtn.s */
.section .init
	/* gcc will nicely put the contents of crtend.o's .init section here. */
	popq %rbp
	ret

.section .fini
	/* gcc will nicely put the contents of crtend.o's .fini section here. */
	popq %rbp
	ret
</pre>

=== ARM (BPABI) ===

In this case things are slightly different. The system ABI mandates the use of special sections called <tt>.init_array</tt> and <tt>.fini_array</tt>, rather than the common <tt>.init</tt> and <tt>.fini</tt> sections. This means that <tt>crtbegin.o</tt> and <tt>crtend.o</tt>, as provided by your cross-compiler, does not insert instructions into the <tt>.init</tt> and <tt>.fini</tt> sections. The result is that if you follow the method from Intel/AMD systems, your <tt>_init</tt> and <tt>_fini</tt> functions will do nothing. Your cross-compiler may actually come with default <tt>crti.o</tt> and <tt>crtn.o</tt> objects, however they also suffer from this ABI decision, and their <tt>_init</tt> and <tt>_fini</tt> functions will also do nothing.

The solution is to provide your own <tt>crti.o</tt> object that inserts a symbol at the start of <tt>.init_array</tt> and <tt>.fini_array</tt> sections, as well as your own <tt>crtn.o</tt> that inserts a symbol at the end of the sections. In this case it is actually possible to write <tt>crti.o</tt> and <tt>crtn.o</tt> in C, because we are not writing incomplete functions. These files should be compiled like the rest of your kernel and otherwise used normally as <tt>crti.o</tt> and <tt>crtn.o</tt>.

<source lang="c">
/* crti.c for ARM - BPABI - use -std=c99 */
typedef void (*func_ptr)(void);

extern func_ptr _init_array_start[0], _init_array_end[0];
extern func_ptr _fini_array_start[0], _fini_array_end[0];

void _init(void)
{
	for ( func_ptr* func = _init_array_start; func != _init_array_end; func++ )
		(*func)();
}

void _fini(void)
{
	for ( func_ptr* func = _fini_array_start; func != _fini_array_end; func++ )
		(*func)();
}

func_ptr _init_array_start[0] __attribute__ ((used, section(".init_array"), aligned(sizeof(func_ptr)))) = { };
func_ptr _fini_array_start[0] __attribute__ ((used, section(".fini_array"), aligned(sizeof(func_ptr)))) = { };
</source>

<source lang="c">
/* crtn.c for ARM - BPABI - use -std=c99 */
typedef void (*func_ptr)(void);

func_ptr _init_array_end[0] __attribute__ ((used, section(".init_array"), aligned(sizeof(func_ptr)))) = { };
func_ptr _fini_array_end[0] __attribute__ ((used, section(".fini_array"), aligned(sizeof(func_ptr)))) = { };
</source>

Additionally, if you use constructor/desctructor priorities, the compiler will append these priorities to the section name. The linker script is expected to expected to sort these, so you will have to add the following to your linker script. Note that we have to treat the <tt>crti.o</tt> and <tt>crtn.o</tt> objects specially because we need to put the symbols in the right order. Alternatively, you can emit the <tt>_init_array_start</tt>, <tt>_init_array_end</tt>, <tt>_fini_array_start</tt>, <tt>_fini_array_end</tt> symbols yourself from the linker script.

<pre>
/* Include the list of initialization functions sorted. */
.init_array :
{
    crti.o(.init_array)
    KEEP (*(SORT(EXCLUDE_FILE(crti.o crtn.o) .init_array.*)))
    KEEP (*(EXCLUDE_FILE(crti.o crtn.o) .init_array))
    crtn.o(.init_array)
}

/* Include the list of termination functions sorted. */
.fini_array :
{
    crti.o(.fini_array)
    KEEP (*(SORT(EXCLUDE_FILE(crti.o crtn.o) .fini_array.*)))
    KEEP (*(EXCLUDE_FILE(crti.o crtn.o) .fini_array))
    crtn.o(.fini_array)
}
</pre>

=== CTOR/DTOR ===

Another way to execute the global constructors / destructors is to execute the .ctors / .dtors symbols manually (assuming you have your own ELF loader, see [[ELF_Tutorial]]). Once you have loaded each ELF file into memory, and all of the symbols have been resolved and relocated you can use .ctors / .dtors to execute the global constructors / destructors manually (apparently, the same applies to .init_array and .fini_array). To do this, you must first locate the .ctors / .dtors section headers:

<source lang="c">
for (i = 0; i < ef->ehdr->e_shnum; i++)
{
    char name[250];
    struct elf_shdr *shdr;

    ret = elf_section_header(ef, i, &shdr);
    if (ret != ELF_SUCCESS)
        return ret;

    ret = elf_section_name_string(ef, shdr, &name);
    if (ret != BFELF_SUCCESS)
        return ret;

    if (strcmp(name, ".ctors") == 0)
    {
        ef->ctors = shdr;
        continue;
    }

    if (strcmp(name, ".dtors") == 0)
    {
        ef->dtors = shdr;
        continue;
    }
}
</source>

Now that you have the .ctors / .dtors section headers, you can resolve each constructor using the following. Note that .ctors / .dtors is a table of pointers (32bit for ELF32 and 64bit for ELF64). Each pointer is a function that must be executed.

<source lang="c">
typedef void(*ctor_func)(void);

for(i = 0; i < ef->ctors->sh_size / sizeof(void *); i++)
{
    ctor_func func;
    elf64_addr sym = 0;

    sym = ((elf64_addr *)(ef->file + ef->ctors->sh_offset))[i];
    func = ef->exec + sym;
    func();

    /* elf->file is the char * that stores the ELF file that your working with. Could be binary or shared library */
    /* elf->exec is the char * that stores the location in memory that the ELF file have been loaded to, and and reloacted */
}
</source>

Don't be surprised if you only have one entry in .ctors / .dtors. At least on x86_64, GCC appears to add a single entry to a set of functions called _GLOBAL__sub_I_XXX and _GLOBAL__sub_D_XXX which call _Z41__static_initialization_and_destruction_0ii that actually call each constructor for you. Adding more globally defined constructors / destructors will cause this function to grow, and not .ctors / .dtors. 

=== Stability Issues ===

If you don't call the constructors / destructors that GCC provides, GCC will generate code that will segfault under certain conditions with x86_64. Take this for example:

<source lang="cpp">
class A
{
    public: 
        A() {}
};

A g_a;

void foo(void)
{
    A *p_a = &g_a;
    p_a->anything();     // <---- segfault
}
</source>

It appears that GCC is using the constructor / destructor initialization routines to do more than simply call the constructors / destructors of each globally defined class. Executing the functions defined in .ctors / .dtors not only initializes all of the constructors / destructors, but resolves these types of segfaults (the above is only one example of many that are resolved). From what I can tell, when globally defined objects exist, GCC might also create ".data.rel.ro" which is another relocation table that GCC needs to process. It is marked as PROGBITS and not REL/RELA, which means that the ELF loader will not do the relocations for you. Instead, executing the functions defined in .ctors will execute _Z41__static_initialization_and_destruction_0ii which appears to perform the relocations for us. See the following for more info: [https://gcc.gnu.org/bugzilla/show_bug.cgi?id=68738]

== Clang ==
'''Note:''' Since Clang attemps to be largely compatible with GCC, the information listed there can possibly be adapted easily. If you give it a try, please document the findings here.

From what I can tell, clang doesn't require you to specify the correct crt{begin,end}.o in the objects you pass to your linker, provided you pass crt[in].o in correct place. clang, being able to output to many targets, doesn't always have a usable crt{begin,end}.o at hand, which it seems to somehow compile on demand. Calling _init doesn't seem to be a requirement, too. An additional call to _init doesn't execute it twice, so it's still safe to call manually.

In summary, in order to adapt this from GCC to clang, take out crt{begin,end}.o from your linker line and you should be fine.

== Other Compilers / Platforms ==
If your compiler or system ABI is not listed here, you will need to consult the appropriate documentation manually and hopefully document the information here if it is relevant.

== See Also ==

=== External links ===
* [https://gcc.gnu.org/onlinedocs/gccint/Initialization.html How Initialization Functions Are Handled] in the GCC Documentation.


[[Category:C]]
[[Category:C++]]
````

<a id="w003338"></a>

## W003338 — User:Combuster/Object File Architecture

Bron-ID: W003338
Namespace: User (2)
Revisie: 14521 | Datum: 2013-03-07T11:46:17Z
Bron: https://wiki.osdev.org/User:Combuster/Object_File_Architecture
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACombuster%2FObject_File_Architecture&oldid=14521

````text
When compiling source code in parts, the compiler will not be able to fill out all details at first, but rather delegates it to the linker, who is in fact aware of all the components, or even the runtime linker. Because the compiler won't know where data ends up, it has to attach extra data to the machine code, so that the missing information can be added later.

== Symbols & Relocations ==
The core mechanic of the intermediate representation depends on having two parts. Symbols represent locations in the code, and relocations represent bits in the code where a reference is made to such a symbol. After you link an application, most of the actual locations of each symbol will become known, and the linker can make sure each of the references to each symbol gets replaced with their correct values. Actually, a linker will throw an error if a symbol is referenced and that symbol can not be found.
Once linking is done, the executable has the correct values and both the symbols and relocations are no longer needed to run the application. That doesn't mean they are not useful: Having the symbols will allow you to addresses back to their original names. Having the relocations allows you to move the executable around in memory, which is essential in some implementations of shared libraries where you want to prevent errors caused by libraries liking to stay in the same address. Using position-independent methods is a common alternative and the trade-off is typicially decided between load-time overhead of relocating and runtime overhead of position-independent code. There's typically memory overhead as well, but these effects are mostly dictated by to the implementation details rather than the choice itself.

=== Relocation styles ===
The way relocations are provided depend largely on the file format and the architecture of the platform. These architectures typically have a few common grounds, and I will show some assembly examples.

===== Absolute references =====
The most simple form of using a symbol is to just point to the exact location where the data is stored. Most data references work in this fashion. For example:
<source lang="c">
extern int variable_not_defined_here;
extern int other_variable;
(...)
int value = variable_not_defined_here + other_variable;
</source>
On i386 the resulting machine code takes the following form
<source lang="asm">
mov eax, [variable_not_defined_here]
add eax, [other_variable]
</source>
When we assemble this, it doesn't know the locations yet, so it'll have to provide some substitute values
<pre>
   0:	a1 00 00 00 00       mov    eax, [0]
   5:	03 05 00 00 00 00    add    eax, [0]
</pre>
If we try to run that as is, we would be accessing a null pointer. Therefore we have to tell the linker to replace those values later:
<pre>
contents
   0:	a1 00 00 00 00       mov    eax, [0]
   5:	03 05 00 00 00 00    add    eax, [0]

relocations
   1:   absolute     variable_not_defined_here
   7:   absolute     other_variable
</pre>
Note that the relocations reference locations 1 and 7 - the linker doesn't want to do anything with assembly and especially not with having to disassemble to find out where the values should be put. Therefore, we simply give it the exact location of memory where the changed value is supposed to go. The linker then determines what the address should be and substitutes it:
<pre>
 8048080:	a1 90 90 04 08       	mov    eax, [0x8049090]
 8048085:	03 05 94 90 04 08    	add    eax, [0x8049094]
</pre>
The linker decides the code should be at 0x08048080, and the data at 0x0804090. If the application does indeed get loaded there, these instructions will now work as expected.
````

<a id="w003339"></a>

## W003339 — User:Kenny/Test

Bron-ID: W003339
Namespace: User (2)
Revisie: 14525 | Datum: 2013-03-11T11:10:13Z
Bron: https://wiki.osdev.org/User:Kenny/Test
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKenny%2FTest&oldid=14525

````text
= Test Article =

This article is long and sprawling, and covers multiple platforms.  To see only section specific to your platform, select the appropriate link.

<div class="mw-customtoggle-windowsonly">Windows</div>
<div class="mw-customtoggle-linuxonly">Linux</div>
<div class="mw-customtoggle-macosonly">MacOS</div>

<div class="mw-customtoggle-windowsonly mw-customtoggle-linuxonly">Windows & Linux</div>


<div class="mw-collapsible" id="mw-customcollapsible-windowsonly">
== Section For Windows ==
<div class="mw-collapsible-content">Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed dictum mi vitae tellus porta sed tincidunt eros porta. Pellentesque quis.</div>
</div>


<div class="mw-collapsible" id="mw-customcollapsible-linuxonly">
== Section For Linux ==
<div class="mw-collapsible-content">Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed dictum mi vitae tellus porta sed tincidunt eros porta. Pellentesque quis.</div>
</div>


<div class="mw-collapsible" id="mw-customcollapsible-macosonly">
== Section For MacOS ==
<div class="mw-collapsible-content">Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed dictum mi vitae tellus porta sed tincidunt eros porta. Pellentesque quis.</div>
</div>


<div class="mw-collapsible" id="mw-customcollapsible-windowsonly">
== More Windows Only ==
<div class="mw-collapsible-content">Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed dictum mi vitae tellus porta sed tincidunt eros porta. Pellentesque quis.</div>
</div>


<div class="mw-collapsible" id="mw-customcollapsible-windowsonly">
== More Windows Only ==
<div class="mw-collapsible-content">Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed dictum mi vitae tellus porta sed tincidunt eros porta. Pellentesque quis.</div>
</div>
````

<a id="w003340"></a>

## W003340 — User:Cwood

Bron-ID: W003340
Namespace: User (2)
Revisie: 15447 | Datum: 2013-12-06T14:47:16Z
Bron: https://wiki.osdev.org/User:Cwood
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACwood&oldid=15447

````text
== Articles Currently Being Written ==
Disk Image Management: [[User:CWood/Disk Image Management]]
````

<a id="w003342"></a>

## W003342 — Talk:Intel High Definition Audio

Bron-ID: W003342
Namespace: Talk (1)
Revisie: 18528 | Datum: 2015-11-11T19:09:14Z
Bron: https://wiki.osdev.org/Talk:Intel_High_Definition_Audio
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AIntel_High_Definition_Audio&oldid=18528

````text

== Identifying HDA ==


I suggested using the Class code (4h) and subclass (3h) to identify HDA, but the specification doesn't spell that out as the correct method. If there is any case where that doesn't work, please edit these pages to say so.


== Widget arrangement in codecs ==


To help work out the rules for connecting things up, it would be useful to have a better understanding of how variable different AFG codecs can be. My machine has a Realtek codec with Vendor ID 10ECh and Device ID 0662h, and that information led me to its datasheet and a map showing how its widgets are connected together. If anyone reading this can supply the Vendor ID and Device ID for the AFG codec in their machine, that'll help us all come up with better rules to follow so that our software will have more chance of working on any machine that supports Intel HD Audio.[[User:Davidcooper|Davidcooper]] 13:09, 11 November 2015 (CST)
````

<a id="w003343"></a>

## W003343 — User:Virtlink/common.css

Bron-ID: W003343
Namespace: User (2)
Revisie: 14583 | Datum: 2013-04-25T21:57:26Z
Bron: https://wiki.osdev.org/User:Virtlink/common.css
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AVirtlink%2Fcommon.css&oldid=14583

````text
table.wikitable,
table.prettytable {
	margin: 1em 1em 1em 0;
	background: #f9f9f9;
	border: 1px #aaaaaa solid;
	border-collapse: collapse;
	empty-cells: show;
}
table.wikitable th, table.wikitable td,
table.prettytable th, table.prettytable td {
	border: 1px #aaaaaa solid;
	padding: 0.2em 0.4em;
}
table.wikitable th, table.wikitable td.hl3, table.wikitable th.hl3,
table.prettytable th, table.prettytable td.hl3, table.wikitable th.hl3 {
	background: #8da7d6;
	text-align: center;
}
table.wikitable td.hl1, table.wikitable th.hl1,
table.prettytable td.hl1, table.wikitable th.hl1 {
	background: #c5d8fc;
	text-align: center;
}
table.wikitable td.hl2, table.wikitable th.hl2,
table.prettytable td.hl2, table.wikitable th.hl2 {
	background: #a7c1f2;
	text-align: center;
}
table.wikitable caption,
table.prettytable caption {
	margin-left: inherit;
	margin-right: inherit;
	font-weight: bold;
}
````

<a id="w003344"></a>

## W003344 — X86 Page Tables

Bron-ID: W003344
Namespace: Main (0)
Revisie: 30733 | Datum: 2026-06-12T16:13:00Z
Bron: https://wiki.osdev.org/X86_Page_Tables
Revisie-URL: https://wiki.osdev.org/index.php?title=X86_Page_Tables&oldid=30733

````text
{{Disputed}}
The x86 processors use '''page tables''' (or '''page map levels''') to map each virtual page to a corresponding physical page. Zero or more virtual pages can correspond to the same physical page, with more than 1 mapping of the same physical page allowed considering the same cache control bits are used for all of them. The size of a page depends on the processor mode (protected, compatibility or long mode), the extensions used (e.g. PAE) and the virtual address bits supported by the processor (current AMD64 processors support 48-bit and 57-bit virtual addresses, with LA57 extension).

== Determining your page size ==
To determine the ideal page size, you can't just look at the overhead of the paging structures alone.

Typically (for user space), each process has at least 3 areas with different characteristics: executable, read only no-execute and read/write no-execute. When paging is used to enforce this, you end up with padding from the actual end of each area up to the next page boundary. You can assume that on average this padding will be 50% of the page size. For example, with 4096 byte pages and 3 areas you'd expect 6 KiB of RAM wasted per process due to padding, and with 2 MiB pages you'd expect 3 MiB of RAM wasted per process.

Typically (for OSs like Windows and Linux) there's around 50 processes where most use very little RAM and some use lots (X, browser, etc). If you assume 50 processes that use an average of 10 MiB of RAM each; then (for 4 KiB pages in long mode) each process (on average) would use a PML4, PDPT, PD and 5 page tables; or 32 KiB for all paging structures. For 2 MiB pages each process (on average) would use a PML4, PDPT and a PD; or 12 KiB for all paging structures.

That gives us some rough figures for comparison. For 50 processes where each process is an average of 10 MiB and has 3 different areas:

* 4 KiB paging will cost about 6 KiB for padding and 32 KiB for paging structures, or about 38 KiB of overhead per process
* 2 MiB paging will cost about 3 MiB for padding and 12 KiB for paging structures, or about 3084 KiB of overhead per process
* 4 KiB paging will cost about 1.86 MiB of overhead for all 50 processes combined
* 2 MiB paging will cost about 150.6 MiB of overhead for all 50 processes combined
* With 500 MiB of RAM actually used by all processes; 1.86 MiB of total overhead works out to about 0.37% more, which is almost nothing
* With 500 MiB of RAM actually used by all processes; 150.6 MiB of total overhead works out to about 23.15% more, which is massive

For performance (e.g. TLB misses) it's much harder to estimate the likely cost, as it depends on how much of the paging structures remain in the cache (and how much has to be fetched from RAM), how large the TLBs are (for both small pages and large pages), if the CPU caches higher level structures (modern CPUs do), the working set of each process and its access pattern, how often you switch between processes, etc...

However (for typical OSs with typical loads), I'd assume it's very unlikely that the performance gains you get from using 2 MiB pages is going to justify having roughly 23% more RAM wasted.

Basically, 4 KiB pages (with 4 levels of paging structures) is starting to get a little small, but the next step up (2 MiB pages with 3 levels of paging structures) is far too big to be practical for most things.

To reduce the number of levels of paging structures, a better idea would be to also increase the size of page directories, PDPTs, etc. For example, for 55-bit virtual addresses you could have 64 KiB pages, 64 KiB page tables, 64 KiB page directories and 64 KiB PDPTs. Unfortunately we have to wait for Intel (or AMD) to do something like that though.

~ [http://forum.osdev.org/viewtopic.php?p=198045#p198045 Brendan]

Luckily though you can freely mix 4k, 2MiB and 1GiB pages. You don't have to use an uniform page size for everything. So a process having 9MB of data can get 4 2MiB pages and make up the rest with 4k pages. That saves paging structures, improves TLB usage and does not increase the overhead at all.

== Recursive mapping ==
To make it easier to change the current address space's page map, you can map an entry of the highest page map level into itself.

Recursive mapping wastes some virtual address space. This table shows the relative space used by the recursively mapped page table:
{| class="wikitable"
! Mode
! Page size
! Max page map size
! Used virtual space
! Total virtual space
! colspan="2" | Ratio
|-
| rowspan="2" | Protected mode (non-PAE)
| 4 KiB
| style="text-align: right;" | 4 MiB
| rowspan="2" style="text-align: right;" | 4 MiB
| rowspan="2" style="text-align: right;" | 4 GiB
| rowspan="2" style="text-align: right; border-right: none;" | 1/1024
| rowspan="2" style="text-align: left; border-left: none;"   | (0.1%)
|-
| 4 MiB
| style="text-align: right;" | 4 KiB
|-
| rowspan="2" | Protected mode (PAE)
| 4 KiB
| style="text-align: right;" | 8 MiB
| rowspan="2" style="text-align: right;" | 1 GiB
| rowspan="2" style="text-align: right;" | 4 GiB
| rowspan="2" style="text-align: right; border-right: none;" | 1/4
| rowspan="2" style="text-align: left; border-left: none;"   | (25%)
|-
| 2 MiB
| style="text-align: right;" | 16 KiB
|-
| rowspan="3" | Long mode (48-bit)
| 4 KiB
| style="text-align: right;" | 512 GiB
| rowspan="3" style="text-align: right;" | 512 GiB
| rowspan="3" style="text-align: right;" | 256 TiB
| rowspan="3" style="text-align: right; border-right: none;" | 1/512
| rowspan="3" style="text-align: left; border-left: none;"   | (0.2%)
|-
| 2 MiB
| style="text-align: right;" | 1 GiB
|-
| 1 GiB
| style="text-align: right;" | 2 MiB
|}

The ''Recursive mapping'' column in the tables below shows the base address and offset to use to get to a particular page map level when the page map is recursively mapped as the ''last'' entry.

== Protected/compatibility mode (32-bit) page map ==
In protected mode, the virtual address space is 32-bit (4 GiB) in size, regardless of whether PAE is enabled or not.

=== Non-PAE mode ===
Without enabling PAE, the page map can reference physical pages with addresses up to 32-bit (4 GiB).

==== 4 KiB pages ====
{| class="wikitable"
! Level
! Table
! colspan="2" | Size
! colspan="2" | Range
! Bits
! Entries
! colspan="2" | Pages
! colspan="2" | Recursive mapping
|-
| 0
| (page)
| style="text-align: right; border-right: none;" | -
| style="text-align: left; border-left: none;"   | &nbsp;
| style="text-align: right; border-right: none;" | 0x1000
| style="text-align: left; border-left: none;"   | (4 KiB)
| 12 bits
| -
| style="text-align: right; border-right: none;" | 0x1
| style="text-align: left; border-left: none;"   | (1)
| style="text-align: right; border-right: none;" | -
| style="text-align: left; border-left: none;"   | &nbsp;
|-
| 1
| PT
| style="text-align: right; border-right: none;" | 0x1000
| style="text-align: left; border-left: none;"   | (4 KiB)
| style="text-align: right; border-right: none;" | 0x40 0000
| style="text-align: left; border-left: none;"   | (4 MiB)
| 10 bits
| 1024
| style="text-align: right; border-right: none;" | 0x400
| style="text-align: left; border-left: none;"   | (1024)
| style="text-align: right; border-right: none;" | 0xFFC0 0000
| style="text-align: left; border-left: none;"   | + 0x1000 * PDi
|-
| 2
| PD
| style="text-align: right; border-right: none;" | 0x1000
| style="text-align: left; border-left: none;"   | (4 KiB)
| style="text-align: right; border-right: none;" | 0x10000 0000
| style="text-align: left; border-left: none;"   | (4 GiB)
| 10 bits
| 1024
| style="text-align: right; border-right: none;" | 0x10 0000
| style="text-align: left; border-left: none;"   | (1048576)
| style="text-align: right; border-right: none;" | 0xFFFF F000
| style="text-align: left; border-left: none;"   | &nbsp;
|}

==== 4 MiB pages ====
{| class="wikitable"
! Level
! Table
! colspan="2" | Size
! colspan="2" | Range
! Bits
! Entries
! colspan="2" | Pages
! colspan="2" | Recursive mapping
|-
| 0
| (page)
| style="text-align: right; border-right: none;" | -
| style="text-align: left; border-left: none;"   | &nbsp;
| style="text-align: right; border-right: none;" | 0x40 0000
| style="text-align: left; border-left: none;"   | (4 MiB)
| 22 bits
| -
| style="text-align: right; border-right: none;" | 0x1
| style="text-align: left; border-left: none;"   | (1)
| style="text-align: right; border-right: none;" | -
| style="text-align: left; border-left: none;"   | &nbsp;
|-
| 2
| PD
| style="text-align: right; border-right: none;" | 0x1000
| style="text-align: left; border-left: none;"   | (4 KiB)
| style="text-align: right; border-right: none;" | 0x10000 0000
| style="text-align: left; border-left: none;"   | (4 GiB)
| 10 bits
| 1024
| style="text-align: right; border-right: none;" | 0x400
| style="text-align: left; border-left: none;"   | (1024)
| style="text-align: right; border-right: none;" | 0xFFC0 0000
| style="text-align: left; border-left: none;"   | &nbsp;
|}

=== PAE mode ===
With PAE, the page map can reference physical pages with addresses up to 36-bit (64 GiB).

==== 4 KiB pages ====
{| class="wikitable"
! Level
! Table
! colspan="2" | Size
! colspan="2" | Range
! Bits
! Entries
! colspan="2" | Pages
! colspan="2" | Recursive mapping
|-
| 0
| (page)
| style="text-align: right; border-right: none;" | -
| style="text-align: left; border-left: none;"   | &nbsp;
| style="text-align: right; border-right: none;" | 0x1000
| style="text-align: left; border-left: none;"   | (4 KiB)
| 12 bits
| -
| style="text-align: right; border-right: none;" | 0x1
| style="text-align: left; border-left: none;"   | (1)
| style="text-align: right; border-right: none;" | -
| style="text-align: left; border-left: none;"   | &nbsp;
|-
| 1
| PT
| style="text-align: right; border-right: none;" | 0x1000
| style="text-align: left; border-left: none;"   | (4 KiB)
| style="text-align: right; border-right: none;" | 0x20 0000
| style="text-align: left; border-left: none;"   | (2 MiB)
| 9 bits
| 512
| style="text-align: right; border-right: none;" | 0x200
| style="text-align: left; border-left: none;"   | (512)
| style="text-align: right; border-right: none;" | 0xC000 0000
| style="text-align: left; border-left: none;"   | + 0x20 0000 * PDi + 0x1000 * PTi
|-
| 2
| PD
| style="text-align: right; border-right: none;" | 0x1000
| style="text-align: left; border-left: none;"   | (4 KiB)
| style="text-align: right; border-right: none;" | 0x4000 0000
| style="text-align: left; border-left: none;"   | (1 GiB)
| 9 bits
| 512
| style="text-align: right; border-right: none;" | 0x40000
| style="text-align: left; border-left: none;"   | (262144)
| style="text-align: right; border-right: none;" | 0xC060 0000
| style="text-align: left; border-left: none;"   | + 0x1000 * PDi
|-
| 3
| PDP
| style="text-align: right; border-right: none;" | 0x20
| style="text-align: left; border-left: none;"   | (32 bytes)
| style="text-align: right; border-right: none;" | 0x10000 0000
| style="text-align: left; border-left: none;"   | (4 GiB)
| 2 bits
| 4
| style="text-align: right; border-right: none;" | 0x10 0000
| style="text-align: left; border-left: none;"   | (1048576)
| style="text-align: right; border-right: none;" | 0xC060 3000
| style="text-align: left; border-left: none;"   | &nbsp;
|}

==== 2 MiB pages ====
{| class="wikitable"
! Level
! Table
! colspan="2" | Size
! colspan="2" | Range
! Bits
! Entries
! colspan="2" | Pages
! colspan="2" | Recursive mapping
|-
| 0
| (page)
| style="text-align: right; border-right: none;" | -
| style="text-align: left; border-left: none;"   | &nbsp;
| style="text-align: right; border-right: none;" | 0x20 0000
| style="text-align: left; border-left: none;"   | (2 MiB)
| 21 bits
| -
| style="text-align: right; border-right: none;" | 0x1
| style="text-align: left; border-left: none;"   | (1)
| style="text-align: right; border-right: none;" | -
| style="text-align: left; border-left: none;"   | &nbsp;
|-
| 2
| PD
| style="text-align: right; border-right: none;" | 0x1000
| style="text-align: left; border-left: none;"   | (4 KiB)
| style="text-align: right; border-right: none;" | 0x4000 0000
| style="text-align: left; border-left: none;"   | (1 GiB)
| 9 bits
| 512
| style="text-align: right; border-right: none;" | 0x200
| style="text-align: left; border-left: none;"   | (512)
| style="text-align: right; border-right: none;" | 0xC000 0000
| style="text-align: left; border-left: none;"   | + 0x20 0000 * PDi
|-
| 3
| PDP
| style="text-align: right; border-right: none;" | 0x20
| style="text-align: left; border-left: none;"   | (32 bytes)
| style="text-align: right; border-right: none;" | 0x10000 0000
| style="text-align: left; border-left: none;"   | (4 GiB)
| 2 bits
| 4
| style="text-align: right; border-right: none;" | 0x800
| style="text-align: left; border-left: none;"   | (2048)
| style="text-align: right; border-right: none;" | 0xC060 0000
| style="text-align: left; border-left: none;"   | &nbsp;
|}


== Long mode (64-bit) page map ==
In long mode, the virtual address space ''could in theory'' be 64-bit (16 EiB) in size, but individual processors allow only a portion of that space to be addressed. The currently most common processor implementations allow a 48-bit (256 TiB) address space. For such virtual addresses bits 48-63 must be a copy of bit 47 (similar to sign extension), and this splits the virtual address space in a higher half and a lower half.

=== 48-bit virtual address space ===

==== 4 KiB pages ====
{| class="wikitable"
! Level
! Table
! colspan="2" | Size
! colspan="2" | Range
! Bits
! Entries
! colspan="2" | Pages
! colspan="2" | Recursive mapping
|-
| 0
| (page)
| style="text-align: right; border-right: none;" | -
| style="text-align: left; border-left: none;"   | &nbsp;
| style="text-align: right; border-right: none;" | 0x1000
| style="text-align: left; border-left: none;"   | (4 KiB)
| 12 bits
| -
| style="text-align: right; border-right: none;" | 0x1
| style="text-align: left; border-left: none;"   | (1)
| style="text-align: right; border-right: none;" | -
| style="text-align: left; border-left: none;"   | &nbsp;
|-
| 1
| PT
| style="text-align: right; border-right: none;" | 0x1000
| style="text-align: left; border-left: none;"   | (4 KiB)
| style="text-align: right; border-right: none;" | 0x20 0000
| style="text-align: left; border-left: none;"   | (2 MiB)
| 9 bits
| 512
| style="text-align: right; border-right: none;" | 0x200
| style="text-align: left; border-left: none;"   | (512)
| style="text-align: right; border-right: none;" | 0xFFFF FF80 0000 0000
| style="text-align: left; border-left: none;"   | + 0x4000 0000 * PDPi + 0x20 0000 * PDi + 0x1000 * PTi
|-
| 2
| PD
| style="text-align: right; border-right: none;" | 0x1000
| style="text-align: left; border-left: none;"   | (4 KiB)
| style="text-align: right; border-right: none;" | 0x4000 0000
| style="text-align: left; border-left: none;"   | (1 GiB)
| 9 bits
| 512
| style="text-align: right; border-right: none;" | 0x40000
| style="text-align: left; border-left: none;"   | (262144)
| style="text-align: right; border-right: none;" | 0xFFFF FFFF C000 0000
| style="text-align: left; border-left: none;"   | + 0x20 0000 * PDPi + 0x1000 * PDi
|-
| 3
| PDP
| style="text-align: right; border-right: none;" | 0x1000
| style="text-align: left; border-left: none;"   | (4 KiB)
| style="text-align: right; border-right: none;" | 0x80 0000 0000
| style="text-align: left; border-left: none;"   | (512 GiB)
| 9 bits
| 512
| style="text-align: right; border-right: none;" | 0x800 0000
| style="text-align: left; border-left: none;"   | (134217728)
| style="text-align: right; border-right: none;" | 0xFFFF FFFF FFE0 0000
| style="text-align: left; border-left: none;"   | + 0x1000 * PDPi
|-
| 4
| PML4
| style="text-align: right; border-right: none;" | 0x1000
| style="text-align: left; border-left: none;"   | (4 KiB)
| style="text-align: right; border-right: none;" | 0x10000 0000 0000
| style="text-align: left; border-left: none;"   | (256 TiB)
| 9 bits
| 512
| style="text-align: right; border-right: none;" | 0x10 0000 0000
| style="text-align: left; border-left: none;"   | (68719476736)
| style="text-align: right; border-right: none;" | 0xFFFF FFFF FFFF F000
| style="text-align: left; border-left: none;"   | &nbsp;
|}


==== 2 MiB pages ====
{| class="wikitable"
! Level
! Table
! colspan="2" | Size
! colspan="2" | Range
! Bits
! Entries
! colspan="2" | Pages
! colspan="2" | Recursive mapping
|-
| 0
| (page)
| style="text-align: right; border-right: none;" | -
| style="text-align: left; border-left: none;"   | &nbsp;
| style="text-align: right; border-right: none;" | 0x20 0000
| style="text-align: left; border-left: none;"   | (2 MiB)
| 21 bits
| -
| style="text-align: right; border-right: none;" | 0x1
| style="text-align: left; border-left: none;"   | (1)
| style="text-align: right; border-right: none;" | -
| style="text-align: left; border-left: none;"   | &nbsp;
|-
| 2
| PD
| style="text-align: right; border-right: none;" | 0x1000
| style="text-align: left; border-left: none;"   | (4 KiB)
| style="text-align: right; border-right: none;" | 0x4000 0000
| style="text-align: left; border-left: none;"   | (1 GiB)
| 9 bits
| 512
| style="text-align: right; border-right: none;" | 0x200
| style="text-align: left; border-left: none;"   | (512)
| style="text-align: right; border-right: none;" | 0xFFFF FF80 0000 0000
| style="text-align: left; border-left: none;"   | + 0x4000 0000 * PDPi + 0x20 0000 * PDi
|-
| 3
| PDP
| style="text-align: right; border-right: none;" | 0x1000
| style="text-align: left; border-left: none;"   | (4 KiB)
| style="text-align: right; border-right: none;" | 0x80 0000 0000
| style="text-align: left; border-left: none;"   | (512 GiB)
| 9 bits
| 512
| style="text-align: right; border-right: none;" | 0x40000
| style="text-align: left; border-left: none;"   | (262144)
| style="text-align: right; border-right: none;" | 0xFFFF FFFF C000 0000
| style="text-align: left; border-left: none;"   | + 0x20 0000 * PDPi
|-
| 4
| PML4
| style="text-align: right; border-right: none;" | 0x1000
| style="text-align: left; border-left: none;"   | (4 KiB)
| style="text-align: right; border-right: none;" | 0x10000 0000 0000
| style="text-align: left; border-left: none;"   | (256 TiB)
| 9 bits
| 512
| style="text-align: right; border-right: none;" | 0x8000000
| style="text-align: left; border-left: none;"   | (134217728)
| style="text-align: right; border-right: none;" | 0xFFFF FFFF FFE0 0000
| style="text-align: left; border-left: none;"   | &nbsp;
|}


==== 1 GiB pages ====
{| class="wikitable"
! Level
! Table
! colspan="2" | Size
! colspan="2" | Range
! Bits
! Entries
! colspan="2" | Pages
! colspan="2" | Recursive mapping
|-
| 0
| (page)
| style="text-align: right; border-right: none;" | -
| style="text-align: left; border-left: none;"   | &nbsp;
| style="text-align: right; border-right: none;" | 0x4000 0000
| style="text-align: left; border-left: none;"   | (1 GiB)
| 30 bits
| -
| style="text-align: right; border-right: none;" | 0x1
| style="text-align: left; border-left: none;"   | (1)
| style="text-align: right; border-right: none;" | -
| style="text-align: left; border-left: none;"   | &nbsp;
|-
| 3
| PDP
| style="text-align: right; border-right: none;" | 0x1000
| style="text-align: left; border-left: none;"   | (4 KiB)
| style="text-align: right; border-right: none;" | 0x80 0000 0000
| style="text-align: left; border-left: none;"   | (512 GiB)
| 9 bits
| 512
| style="text-align: right; border-right: none;" | 0x200
| style="text-align: left; border-left: none;"   | (512)
| style="text-align: right; border-right: none;" | 0xFFFF FF80 0000 0000
| style="text-align: left; border-left: none;"   | + 0x4000 0000 * PDPi
|-
| 4
| PML4
| style="text-align: right; border-right: none;" | 0x1000
| style="text-align: left; border-left: none;"   | (4 KiB)
| style="text-align: right; border-right: none;" | 0x10000 0000 0000
| style="text-align: left; border-left: none;"   | (256 TiB)
| 9 bits
| 512
| style="text-align: right; border-right: none;" | 0x40 000
| style="text-align: left; border-left: none;"   | (262144)
| style="text-align: right; border-right: none;" | 0xFFFF FFFF C000 0000
| style="text-align: left; border-left: none;"   | &nbsp;
|}

== See Also ==
=== Articles ===
*[[Paging]]
*[[Page Frame Allocation]]

=== Forum ===
*[http://forum.osdev.org/viewtopic.php?f=1&t=24292 Time for 2 MB pages?]

<!-- === External Links === -->

[[Category:Memory management]]
[[Category:Paging]]
[[Category:Virtual Memory]]
[[Category:X86]]
````

<a id="w003346"></a>

## W003346 — VMWare

Bron-ID: W003346
Namespace: Main (0)
Revisie: 14601 | Datum: 2013-04-30T11:22:02Z
Bron: https://wiki.osdev.org/VMWare
Revisie-URL: https://wiki.osdev.org/index.php?title=VMWare&oldid=14601
Doorverwijzing: VMware

````text
#REDIRECT [[VMware]]
````

<a id="w003347"></a>

## W003347 — Talk:VMWare

Bron-ID: W003347
Namespace: Talk (1)
Revisie: 14603 | Datum: 2013-04-30T11:22:02Z
Bron: https://wiki.osdev.org/Talk:VMWare
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AVMWare&oldid=14603
Doorverwijzing: Talk:VMware

````text
#REDIRECT [[Talk:VMware]]
````

<a id="w003349"></a>

## W003349 — Template:Quote

Bron-ID: W003349
Namespace: Template (10)
Revisie: 14615 | Datum: 2013-04-30T11:46:58Z
Bron: https://wiki.osdev.org/Template:Quote
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AQuote&oldid=14615

````text
<blockquote style="margin-bottom: 10px; padding: 5px 10px; margin-left: 15px; border-left: 10px solid #eeeeee;">
{{{1}}}
</blockquote>
````

<a id="w003350"></a>

## W003350 — Multiboot

Bron-ID: W003350
Namespace: Main (0)
Revisie: 30712 | Datum: 2026-06-10T13:13:44Z
Bron: https://wiki.osdev.org/Multiboot
Revisie-URL: https://wiki.osdev.org/index.php?title=Multiboot&oldid=30712

````text

The '''Multiboot''' specification is an open standard, targeted towards free 32-bit x86 and MIPS operating systems, that provides kernels with a uniform way to be booted by Multiboot-compliant bootloaders. The reference implementation of the Multiboot specification is provided by [[GRUB]], and the protocol is also supported by some other popular [[Bootloaders]], including [[Limine]]. Many popular operating systems use Multiboot, including GNU/Hurd, NetBSD, Xen, and others (but notably not Linux, which uses [[Linux Boot Protocol|its own boot protocol]]), and many hobby operating systems, with abundance of information and [[:Category:Bare bones tutorials|tutorials]].


Since Multiboot is an older and simpler standard, which as the specification states, is targeted towards 32-bit operating systems, it does practically no CPU and hardware initialization, dropping the kernel (on x86) into protected mode with no paging, and providing the bare minimum for further initialization. Many newer [[:Category:Bootloaders|boot protocols]] are more ergonomic, especially for 64-bit kernels, providing bootstrap paging initialization and loading the kernel directly into its desired virtual memory location and final CPU mode (long mode on x86), filtering the [[Memory Map]], dealing with x86 tech debt, sometimes initializing the SMP, and other conveniences, among with supporting more architectures (requiring less work with supporting other protocols when porting to them). For these reasons, the other protocols (such as [[Limine Bare Bones|Limine]]) are now commonly recommended for beginners instead of Multiboot. However, Multiboot (version 2, the first version has problems with 64-bit hardware) is still perfectly fine, even for 64-bit kernels, provided that there is a specific need for it (need to use a specific bootloader, exotic memory layout, etc.) and/or the kernel is ok with doing extra work during the initialization. In the end, apart from extra potential work to initialize itself, the kernel does not gain nor lose anything major by choosing to use Multiboot or some other protocol.

== Multiboot 1 ==
The original Multiboot specification was created by Bryan Ford and Erich Stefan Boleyn in 1995. It has evolved and been updated by the Free Software Foundation ever since.

Multiboot 1 compliant kernels used the magic number 0x1BADB002, and Multiboot-compliant bootloaders report themselves with magic number 0x2BADB002.

You can check whether a program has a valid Multiboot 1 header by running <tt>grub-file</tt> which is quiet but will exit 0 if it finds a header, and 1 otherwise.
<syntaxhighlight lang="bash">grub-file --is-x86-multiboot myos.bin</syntaxhighlight>

You can boot a multiboot v1 kernel with the following command in your menuentry:
<pre>multiboot /boot/kernel-file</pre>

== Multiboot 2 ==
The newer Multiboot specification was created to solve some of the problems that the original specification had. It is not backwards compatible, uses different structures and different magic numbers.

Multiboot 2 compliant kernels used the magic number 0xE85250D6, and Multiboot-compliant bootloaders report themselves with magic number 0x36D76289. On x86, this magic number is stored in EAX just before the kernel is invoked.

You can check whether a program has a valid Multiboot 2 header by running <tt>grub-file</tt> which is quiet but will exit 0 if it finds a header, and 1 otherwise.
<syntaxhighlight lang="bash">grub-file --is-x86-multiboot2 myos.bin</syntaxhighlight>

Note: GNU LD has default page size of 0x00200000 for elf64, which will fail multiboot2 test. You can use "-z max-page-size=SIZE" to get around this.

GNU GRUB is compatible with the Multiboot2 specification as of version 2.02.

You can boot a multiboot v2 kernel with the following command in your menuentry:
<pre>multiboot2 /boot/kernel-file</pre>

=== Header Format ===

The first part of the Multiboot2 header is a struct aligned on an 8-byte boundary containing four "magic" 32-bit integers. These fields contain the magic number 0xE85250D6, flags describing the architecture, the total length of the header including the subsequent tags, and the checksum of the magic fields respectively.

The magic fields in the header are followed by a list of 8-byte aligned structures referred to in the specification as <tt>tags</tt>. The tags are used as hints by the bootloader to provide information about the kernel image and to request that it provide additional information in the boot information struct.

The available tags and their structure vary by platform and are documented in the Multiboot2 specification. See [[#External Links]] below for the latest draft. This list of tags must end with a tag of type 0 and size 8.

=== Information Structure ===

A Multiboot2-compliant bootloader provides an [https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Boot-information-format information structure] when the kernel boots. On x86, a physical pointer is stored in EBX, while the magic number is stored in EAX. It's a well-known practice to check the magic number to verify the bootloader passed the correct information. This physical pointer information structure is 8-byte aligned, and contains tags. These tags may contain useful information about the system, for example: [https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Memory-map the memory map], [https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#ELF_002dSymbols ELF sections of the image] and [https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Framebuffer-info information about the framebuffer].

<syntaxhighlight lang="c">
struct multiboot_tag {
    uint32_t type;
    uint32_t size;
};

struct multiboot_info {
    uint32_t total_size;
    uint32_t reserved;
    struct multiboot_tag tags[0];
};

</syntaxhighlight>

== Multiboot 2 modules ==
(Note that this section may or may not be 100% accurate, has yet to be tested.)

The Multiboot 2 specification supports boot modules, which are pieces of data loaded into memory when the bootloader boots your kernel. Normally these are just files residing on the boot media, which are loaded by your boot loader, but the exact implementation is up to your bootloader. For GRUB, these modules are loaded using the "module2" command before booting your kernel. 

Once your kernel is booted, you can use the multiboot information structure to locate and identify these modules. Note that in the case of GRUB, the module's string will also include the text after the module name, for example, if you load a module using "module video 1024x768", the string field of that module's structure will contain "video 1024x768". In this way, you can specify 'arguments' to modules when you load them. (This seems to be the behavior intended by the multiboot spec.)

GRUB may support decompressing modules before loading them, this seems to be driven by the extension of the file.

== Placing the Multiboot struct ==

As said before, the struct must be 8 byte aligned and placed in the first 8K of the executable. The simplest solution to assure this is to create a new section for the struct with an attribute:
<syntaxhighlight lang="c">
__attribute__((section(".multiboot"))) struct {
    ...
} multiboot_header;
</syntaxhighlight>
Then tell the [[LD|linker]] in the [[Linker Scripts|linker script]] to start the file with this section with proper alignment:
<pre>
...
.text: {
  /* link the multiboot struct here */
  . = ALIGN(8);
  KEEP(*(.multiboot))
  /* place all of your code afterwards */
  *(.text)
}
...
</pre>
The keyword ALIGN guarantees the alignment, and KEEP tells the linker to keep that section at the beginning of the text segment.

== See Also ==
=== Articles ===
* [[GRUB]]

=== Threads ===
* [[Topic:22481|Multiboot 2 Header layout]]

===External Links===
* [https://www.gnu.org/software/grub/manual/multiboot/multiboot.html Latest Multiboot 1 specification (HTML)]
* [http://git.savannah.gnu.org/cgit/grub.git/tree/doc/multiboot.texi?h=multiboot Latest Multiboot 1 specification]
* [https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html Latest Multiboot 2 specification (HTML)]
* [http://git.savannah.gnu.org/cgit/grub.git/tree/doc/multiboot.texi?h=multiboot2 Latest Multiboot 2 specification]
* [http://web.archive.org/web/*/http://grub.enbug.org/MultibootDraft Old Multiboot 2 drafts]
* [http://git.savannah.gnu.org/cgit/grub.git/tree/doc/multiboot.h?h=multiboot Multiboot 1 header file]
* [http://git.savannah.gnu.org/cgit/grub.git/tree/doc/multiboot2.h?h=multiboot2 Multiboot 2 header file]

[[Category:Booting]]
[[Category:Bootloaders]]
[[de:Multiboot]]
````

<a id="w003351"></a>

## W003351 — User:M12/Rolling Your Own Filesystem

Bron-ID: W003351
Namespace: User (2)
Revisie: 14627 | Datum: 2013-05-02T19:23:02Z
Bron: https://wiki.osdev.org/User:M12/Rolling_Your_Own_Filesystem
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AM12%2FRolling_Your_Own_Filesystem&oldid=14627

````text
This page is a test for an edit to [[File Systems#Rolling your own|this one]].

== Rolling your own ==
There are many different kinds of filesystems around, from the well-known to the more obscure ones. The most unfortunate thing about filesystems is that every hobbyist OS programmer thinks that the filesystem they design is the ultimate technology, when in reality it's usually just a copy of FAT with a change here and there, perhaps because it is one of the easiest to implement. The world doesn't need another FAT-like filesystem. Investigate all the possibilites before you decide to roll your own.

=== Guidelines if you do decide to roll your own ===
<br />
* Consider carefully what it will be used for.
* Use a program to figure out the layout.  I recommend a spreadsheet.  The basic areas needed are:
** Bootsector.  Unless you are booting with UEFI, this is a must.  Even then, I'd recommend including it in the specs for compatibility with older file systems.  This section should contain at a minimum the disk size, location of the file table, hidden sectors for multiple partition disks, and a version number.  I'd leaving plenty of reserved space for features you don't think of.  Don't forget to leave space for a jmp instruction and the boot code!.
** File table.  Don't think of this as just a simple table containing a list of files and their locations.  One idea I've had and never implemented is, instead of storing files, the system would store file parts, and the file table would list the parts in each file.  This would be useful for saving space if many files on the disk are the same or similar (for example, license agreements).
** Data area.  Files will be stored here.
* Create a program to read and write disk images with your filesystem.  Parts of this will be portable into the fs driver.
* Implement the fs into your OS.


== Original Text ==
There are many different kinds of filesystems around, from the well-known to the more obscure ones. The most unfortunate thing about filesystems is that every hobbyist OS programmer thinks that the filesystem they design is the ultimate technology, when in reality it's usually just a copy of FAT with a change here and there, perhaps because it is one of the easiest to implement. The world doesn't need another FAT-like filesystem. Investigate all the possibilites before you decide you roll your own.
````

<a id="w003352"></a>

## W003352 — User:M12

Bron-ID: W003352
Namespace: User (2)
Revisie: 14628 | Datum: 2013-05-02T19:25:47Z
Bron: https://wiki.osdev.org/User:M12
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AM12&oldid=14628

````text
My pages:
[[User:M12/Rolling_Your_Own_Filesystem]]
````

<a id="w003353"></a>

## W003353 — Creating an Operating System

Bron-ID: W003353
Namespace: Main (0)
Revisie: 29373 | Datum: 2025-02-05T22:41:52Z
Bron: https://wiki.osdev.org/Creating_an_Operating_System
Revisie-URL: https://wiki.osdev.org/index.php?title=Creating_an_Operating_System&oldid=29373

````text
{{Rating|4}}
Welcome to operating systems development! This tutorial will document the process of creating a new operating system from the baby steps to creating a self-hosting operating system. The path is long and hard, but also fun and rewarding. As you progress through the stages, you will slowly begin to diverge from the tutorial as you make your own decisions about the design of your operating system and you no longer need this guide. If you create a non-Unix-like operating system, you will diverge earlier and will have to fill in the missing gaps yourself.

== Phase 0: Introduction ==
=== Welcome to Operating Systems Development ===
{{Main|Introduction|Required Knowledge|Beginner Mistakes}}
You should consult all the basic documentation before starting writing an operating system.

=== Building the latest GCC ===
{{Main|Building GCC}}

You may wish to upgrade your system compiler to the latest version before you start out with operating systems development and build the cross-compiler.

== Phase I: Beginning ==
In this phase we will set up a toolchain and create a basic kernel that will become the core of the new operating system.

=== Setting up a Cross-Toolchain ===
{{Main|GCC Cross Compiler|Why do I need a Cross Compiler?}}

The first thing you will like to do is set up a cross-compiler for your operating system. The compiler on your local system is not able to produce programs for your operating system because it hasn't been invented yet. At first you would like to do is create a compiler that produces executables that will run directly on your target hardware.

=== Creating a Hello World kernel ===
{{Main|Bare Bones}}
:''See also [[:Category:Bare bones tutorials|Bare Bones for other platforms]]''

Your next task is to make a simple hello world kernel that is able to start up, print a message to the output device and then loop endlessly. While simple and useless, it will serve as a great example and starting point for a real system, as well as confirm that your testing environment works correctly.

=== Setting up a Project ===
{{Main|Meaty Skeleton}}

With a basic working example, your next task is to set up a build infrastructure using whatever build system you see fit. Be careful in your choices of technology, GNU Make is easier to port than Python.

=== Calling Global Constructors ===
{{Main|Calling Global Constructors}}

The compiler expects you to perform various program initialization tasks, such as calling the constructors on global C++ objects. Normally you would use a <tt>kernel_early_main</tt> function to set up the minimal kernel features, then perform all these initialization tasks, and then jump to the actual <tt>kernel_main</tt> function.

=== Terminal Support ===
{{Main|Formatted Printing}}

You will often need to debug your operating system. Your very best friend is a printf function that is able to print strings and integers to the screen onto a terminal-like buffer. It is crucial to add a printf function to your kernel early on as you will certainly need it later for debugging.

=== Stack Smash Protector ===
{{Main|Stack Smashing Protector}}

Early is not too soon to think about security and robustness. You can take advantage of the optional stack smash protector offered by modern compilers that detect stack buffer overruns rather than behaving unexpectedly (or nothing happening, if unlucky).

=== Multiboot ===
{{Main|Multiboot}}

It's useful to know what features and information the bootloader offers the kernel, as this may help you get memory maps, set video modes, and even kernel symbol tables.

=== Global Descriptor Table ===
{{Main|Global Descriptor Table}}

The Global Descriptor Table is an important part of the processor state and it should as such be one of the first things that are initialized. It probably makes a lot of sense to set up it even prior to kernel_early.

=== Memory Management ===
{{Main|Memory Management}}

Memory allocation and management is one of the most basic functions in an operating system. You need to keep track of physical page frames, what ranges of virtual memory are used, and implementing a heap (malloc, free) upon it for internal kernel use.

=== Interrupts ===
{{Main|Interrupts}}

Your kernel needs to handle asynchronous events sent by the hardware to function properly.

=== Multithreaded Kernel ===
{{Main|Multithreaded Kernel}}

It is best to go multithreaded early in the development of your kernel or you'll end up rewriting parts of your kernel. We'll certainly need this when we add processes later on.

=== Keyboard ===
{{Main|Keyboard}}

Your operating system will certainly need support for reading input from the computer operator so it can adapt its behavior to their wishes.

=== Internal Kernel Debugger ===
{{Main|Internal Kernel Debugger}}

It is very useful for a multithreaded kernel to have built-in debugging facilities early on. You could have a magic key that stops the entire kernel and dumps the user to a mini-shell for debugging. It could know the data structures used by the scheduler to list all the threads and perform call traces, provide debugging info and so on.

=== Filesystem Support ===
:''Main articles: [[Filesystem]], [[Initrd|Initialization Ramdisk]]

It'll be useful to have support for filesystems early on and transferring files onto your operating system using an initialization ramdisk.

== Phase II: User-Space ==
In this phase we'll expand your operating system into user-space and add support for programs, which is enough for your project to qualify as a small operating system. You need to work on system calls, program loading, memory management and rework parts of your system early in this phase.

=== User-Space ===
{{Main|User-Space}}

Your procesessor has until now run in kernel mode, where the code is able to do everything. Processes are normally run with no permissions at all, except being able to execute code and use its designated memory. The first part of implementing user-space is switching the processor to user mode.

=== Program Loading ===
{{Main|Dynamic Linker|ELF}}

The first task you will need to complete is loading a program into memory. This involves parsing the program headers, allocating memory at the right virtual addresses and copying the contents of the segments to the right virtual addresses. You'll also need to fill up entries in the GOT according to the relocation tables.

=== System Calls ===
{{Main|System Calls}}

You are now able to load programs into memory and switch to user mode. Processes communicate with the kernel using system calls, which is the next feature you will want to add.

=== OS Specific Toolchain ===
{{Main|OS Specific Toolchain}}

As your operating system is now becoming a real operating system, it is time to treat it as such. We'll like to teach the cross-compiler about your operating system and its conventions, so we can easily cross-compile programs.

=== Creating a C Library ===
{{Main|Creating a C Library}}

At this point, you can decide to use an existing C library or write your own C library. If you go the custom route, you will want to set up some basic features that the cross-compiler needs for libgcc. With this in place, you can now easily cross-compile programs.

=== Fork and Execute ===
{{Main|Fork}}

With basic program loading in place, we are almost ready to create a multitasking operating system. The missing primitives is allowing a process to create new processes. The standard Unix primitive is fork, which allows a process to create a perfect copy of itself. This copy is then able to call the program loader and replace its own memory with that of another program image.

=== Shell ===
{{Main|Shell}}

This is a very exciting point in your operating system. It is able to run programs and create new processes. So far, the behavior of the system has possibly been determined when it was compiled. By writing a shell, you can run multiple programs and decide which one to run at runtime. This is the point where you reach the level that many newcomers dream of: a basic operating system with a working command line.

== Phase III: Extending your Operating System ==
With these basic features in place, you can now start writing your operating system and all its wonderful features. You'll add games, editors, test programs, command line utilties, drivers and whatever you can imagine. Your skill and imagination is the limit here. You can delay many of these features until later in your operating system and make then in almost any order.

=== Time ===
{{Main|Time}}

Time is a complicated concept in computing, however modern operating systems are expected to have functions for converting timestamps to parsed time and back, as well as providing system clocks (real time, monotonic time, user CPU time, ..) and timers on these clocks with events happening on expiration.

=== Threads ===
{{Main|Thread}}

Operating systems should expose a threading API such as pthreads.

=== Thread Local Storage ===
{{Main|Thread Local Storage}}

Thread local variables require runtime support.

=== Symmetric Multiprocessing ===
{{Main|SMP}}

It's a very good idea to add support for multiple CPUs to your kernel early on, or you will likely need to redo a lot of your kernel because it wasn't SMP-ready in many places.

=== Secondary Storage ===
{{Main|Secondary}}

You will likely want to support common block devices such as harddisks, cdroms, floppies, and whatever storage devices your operating system needs support for.

=== Real Filesystems ===
{{Main|File Systems}}

It's a good idea to add proper filesystem support early on.

=== Graphics ===
{{Main|How do I set a graphics mode}}

Real operating systems don't operate in the basic text mode, but have bitmapped graphics. Writing real graphics drivers is a bunch of work, although some virtual machines offer some useful shortcuts.

=== User Interface ===
:''Main articles'': [[User Interface]], [[Compositing]]

You will certainly need to impress the operating systems development community with your flashy graphics and usable user-interface.

=== Networking ===
{{Main|Networking|Ethernet|IP|TCP}}

The uses for networking support are obvious, so you will likely want to do this.

=== Sound ===
{{Main|Sound}}

Sound is an important part of the computing experience and depending on your needs, you may well wish to support sound on common hardware.

=== Universal Serial Bus ===
{{Main|USB}}

If you need to communicate with modern peripherals, you will likely need a USB stack and support for the various common USB controllers.

== Phase IV: Bootstrapping ==
You now have your basic operating system in place and you are ready to move onto the next level. In this phase, we will start porting software onto your operating system such that you can become self-hosting. You already begun your effort toward being self-hosting when you set up your OS-specific toolchain and it pays off now.

=== Porting Software ===
{{Main|Cross-Porting Software}}

While not all pieces of software is easy to port, most Unix software comes with a autoconf-generated configure script. You can provide these scripts with the --host=mycpu-myos option and if your operating system offers the needed features, you can cross-compile the software onto your operating system. You already met examples of how to port software when setting up the OS-specific toolchain. While the difficulity of cross-compiling software differs greatly, you will like be using the same process for adapting new packages.

=== Porting GCC ===
{{Main|Porting GCC to your OS}}

You already began the work porting binutils and gcc when you set up the OS-specific toolchain. We'll finish the process and cross-compile them onto your operating system such that it can compile the C hello world program.

=== Compiling your OS under your OS ===
The next task is to port your entire build system. You may need to port GNU Make, port some command line utilites (coreutils) or write your own, port a real shell or finish yours and more. You may also need to fix a number of bugs in your operating system such that the compiler runs correctly. You will need to deal with how to transfer the newly compiled version onto permanent storage such that a reboot of the computer will run the next version. Your operating system will now quality as self-compiling.

=== Fully Self-hosting ===
Now that you can build your entire operating system under your operating system, you also need to be able to do the rest. You need to be able to also build your compiler under your operating system. You need to be able to develop under your operating system, so you'll port your favorite text editor or write one. You need networking so you can release the newest version (build on itself) onto the internet. You'll port lots of programs, libraries, games, and whatever else you desire, such that the entire development process can happen on your operating system. You can now uninstall your original operating system and replace it with your new glorious operating system.

== Phase V: Profit ==
You have now successfully created a real operating system that is fully self-hosting and the envy of the entire operating systems development community. You have ported quake, have OpenGL programs, a working browser, a thriving community of contributors, and much success. You can now start over and develop the next operating system from your own operating system.


[[Category:OS Development]]
````

<a id="w003354"></a>

## W003354 — Compositing

Bron-ID: W003354
Namespace: Main (0)
Revisie: 28115 | Datum: 2023-07-09T23:12:52Z
Bron: https://wiki.osdev.org/Compositing
Revisie-URL: https://wiki.osdev.org/index.php?title=Compositing&oldid=28115

````text
{{In Progress}}

'''Compositing''' is a method for providing a windowed GUI in which each application window has its own dedicated buffer - sometimes called a ''surface'' or ''canvas'' - in which it may draw as it pleases. These buffers are then blitted together by the ''compositor'' to produce a final image to display on the screen. The first major system to use a compositing ''window manager'' was the Amiga, which used a hardware blitter to composite windows. Notably, OS X was the first of the three mainstream OSes to ship with a compositing window manager, a feature it had since its first release. Other early mainstream compositors include Compiz (the first widely used dedicated X11 compositor) and DWM (the Windows compositor).

== Overview ==

Compositors are so called because they ''composite'' windows together to form the image shown on screen. In order to do this, each window needs a dedicated buffer in which to draw. As such, compositors require more memory (either system memory or graphics memory) for each new window that will be displayed. The primary benefit to compositing over traditional window management is comparmentalization. Since each window has its own buffer to draw into, and since the compositor has complete control over the final image, artifacts typical of traditional window rendering such as the ''"IE 6 effect"'' are not typically seen in compositing window managers as the window manager does not need to wait for  or request that windows repaint areas previously clipped or covered when they are moved. An additional benefit of compositing is that it allows the window manager to perform various transformations or rendering effects when compositing, such as alpha blending, rotation, scaling, duplication, and so on.

== Prerequisites ==

If you want to build a compositor, you will need a few things:

* '''Video memory access''' - The compositor itself needs to be able to access video memory or have some interface to draw to the screen.
* '''Shared memory''' - An interface to graphics memory is technically superior to system memory, but is significantly more difficult to implement and requires a thorough understanding of graphics hardware. Regardless, you will need an interface by which two processes can read and write the same region of memory (or at least, one side needs to be able to write at least one region, and other side needs to be able to read from many such regions). You will need to be able to support many regions up to, and even beyond, the size of your display in pixels. The format for these shared memory regions will be discussed later.
* '''Message passing, streams, or sockets''' - Some form of synchronous and speedy IPC is required to send messages between the server and the various clients. A system which is engineered to work with a single server and multiple (possibly malicious or ambivalent) clients is preferred. This interface will be used by the client to send commands like "give me a window" and by the server to send events like ''"Alt+F4 pressed"'' or ''"mouse moved to x=259 y=530"''.

== Design Decisions ==

There are two key design decisions to consider in a compositing window manager (though they can apply to other kinds of window managers as well):

* '''Canvas format''' - There are very many different pixel formats in use by graphics libraries, from RGBA32 (8 bits per pixel with an alpha channel) to YUV12. You can choose to support many of these or just one, but note that anything other than the native format you use to composite will require conversion somewhere.
* '''Window decorations''' - Most traditional window managers (both stacking and compositing) render decorations in the server, but a relatively new approach is to have clients render their own borders. The former moves complexity into the server, possibly requiring windows to have subparts and for the compositor to know about different "window types" (see various X11 compositors). Client-side decorations require clients to render on their own, which requires clients to somehow cooperate in determining how decorations should be themed (see Wayland, Quartz). Of course, an alternative approach is to not have decorations at all, a popular thematic decision amongst some technical Unix crowds.

[[Category:Graphical UI]]
````

<a id="w003355"></a>

## W003355 — Why do I need a Cross Compiler

Bron-ID: W003355
Namespace: Main (0)
Revisie: 14683 | Datum: 2013-05-15T06:22:27Z
Bron: https://wiki.osdev.org/Why_do_I_need_a_Cross_Compiler
Revisie-URL: https://wiki.osdev.org/index.php?title=Why_do_I_need_a_Cross_Compiler&oldid=14683
Doorverwijzing: Why do I need a Cross Compiler?

````text
#REDIRECT [[Why_do_I_need_a_Cross_Compiler%3F]]
````

<a id="w003357"></a>

## W003357 — Building Upon Existing Software

Bron-ID: W003357
Namespace: Main (0)
Revisie: 30831 | Datum: 2026-06-30T02:51:13Z
Bron: https://wiki.osdev.org/Building_Upon_Existing_Software
Revisie-URL: https://wiki.osdev.org/index.php?title=Building_Upon_Existing_Software&oldid=30831

````text
== Idea ==

Consider not writing your operating system from scratch! There are a lot of possibilities to create your own operating system without doing ''everything'' on your own. Great examples would be [https://en.wikipedia.org/wiki/OS_X Mac OS X] or [https://en.wikipedia.org/wiki/Android_%28operating_system%29 Android]. Just ask yourself: If even Apple - a great company with billions of turnaround and thousands of employees - decided to make an operating system based on an already existing kernel and userland tools, why would you decide not doing so? Do you think you can do everything better than all those programmers programming Linux, FreeBSD and the like? Sometimes people think they have to write an operating system because of very specific things they don't like. Depending on what you don't like in an existing operating system, there are a number of alternatives to writing your operating system from scratch. If choose one of them, you still can be sure that your task is challenging and you have plenty enough work to do.

=== Process Scheduler ===

If you think you can write a better process scheduler, there would be no need to write a complete new operating system. Instead, you could exchange the one of an existing kernel. An example of this would be the [http://en.wikipedia.org/wiki/Brain_Fuck_Scheduler Brainfuck Scheduler] that replaces the default scheduler used in the [https://en.wikipedia.org/wiki/Linux Linux] kernel. Most possibly, you will have to use the same programming language than an already existing implementation has.

=== GUI ===
 
If you don't like the [[GUI]] of your operating system, consider writing anything of these GUI components:

* Window Server: If you don't like, for example, X11, you could write a replacement for that. X11 is currently in use on majority of GNU/Linux operating systems. Examples of new window servers that replace X11 would be Wayland or Mir.
* Window Manager: If you don't like the way how GUI components are drawn and handled, you could write your own window manager.
* Display Manager: If you think the process of logging in could be made better, you could even write your own display manager or try to enhance an existing one (existing ones are XDM, GDM, KDM or LightDM).
* Desktop Manager: If you think your desktop environment sucks, create a new one or try to replace existing parts of your current desktop manager/environment.

=== Init System ===

Especially Unix-like systems use the System V method of system initialisation, which includes mounting volumes, starting deamons, bringing up network interfaces and similar tasks. If you don't like how your system is made ready for using, you could replace that init system. Existing alternatives are Upstart and systemd. When the kernel transfers control to the init system, a lot of essential work is already done, like hardware is initialised, you have a running process scheduler, memory management and all those things. But you still have a lot of control over the system and you get it very soonly, a few seconds after the system is powered on. The good thing is that you can use the programming language of your choice, even scripting languages like Python can be used at that point of time. To start your init program if you use Linux, supply "init=/path_to_your_init" as a start parameter to the Linux kernel.

[[Category:OS Development]]
````

<a id="w003358"></a>

## W003358 — Talk:Detecting CPU Speed

Bron-ID: W003358
Namespace: Talk (1)
Revisie: 14684 | Datum: 2013-05-15T18:31:57Z
Bron: https://wiki.osdev.org/Talk:Detecting_CPU_Speed
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ADetecting_CPU_Speed&oldid=14684

````text

== Dead link ==

The link at [http://www.sylvain-ulg.be.tf/resources/speed.c looping on a chain of 1550 XORs] is dead. [[User:M12|M12]] 13:31, 15 May 2013 (CDT)
````

<a id="w003359"></a>

## W003359 — UDI Device Enumeration

Bron-ID: W003359
Namespace: Main (0)
Revisie: 27950 | Datum: 2023-07-09T16:04:34Z
Bron: https://wiki.osdev.org/UDI_Device_Enumeration
Revisie-URL: https://wiki.osdev.org/index.php?title=UDI_Device_Enumeration&oldid=27950

````text
[[Category:Uniform Driver Interface]]

==Key structures==

<source lang="c">
typedef struct {
	udi_cb_t gcb;
	udi_ubit32_t child_ID;
	void *child_data;
	udi_instance_attr_list_t *attr_list;
	udi_ubit8_t attr_valid_length;
	const udi_filter_element_t *filter_list;
	udi_ubit8_t filter_list_length;
	udi_ubit8_t parent_ID;
} udi_enumerate_cb_t;</source>

==Key Service Calls==

<source lang="c">
void udi_enumerate_req(udi_enumerate_cb_t *cb, udi_ubit8_t enumeration_level);

/* Values for enumeration_level */
#define UDI_ENUMERATE_START 1
#define UDI_ENUMERATE_START_RESCAN 2
#define UDI_ENUMERATE_NEXT 3
#define UDI_ENUMERATE_NEW 4
#define UDI_ENUMERATE_DIRECTED 5
#define UDI_ENUMERATE_RELEASE 6

void udi_enumerate_ack (udi_enumerate_cb_t *cb, udi_ubit8_t enumeration_result, udi_index_t ops_idx);

/* Values for enumeration_result */
#define UDI_ENUMERATE_OK 0
#define UDI_ENUMERATE_LEAF 1
#define UDI_ENUMERATE_DONE 2
#define UDI_ENUMERATE_RESCAN 3
#define UDI_ENUMERATE_REMOVED 4
#define UDI_ENUMERATE_REMOVED_SELF 5
#define UDI_ENUMERATE_RELEASED 6
#define UDI_ENUMERATE_FAILED 255

</source>

==Overview==
After a [[Uniform Driver Interface|UDI]] driver is loaded by the [[UDI Environment | Management Agent]] (the operating system kernel), the MA proceeds to establish a specific IPC channel to the driver's [[primary region]] which will be used for driver control, called the "Management Metalanguage Channel". This particular UDI channel is unique, in that it is not strictly a two-way dialogue channel. Almost every other UDI channel specified by the core specification or by any metalanguage enables the driver to send messages over the channel, and enables the kernel to send messages to the driver. The Management Metalanguage only allows the MA to call into the driver, and forbids the driver to send messages to the MA unless first asked to do so.

The MA calls <tt>udi_enumerate_req()</tt> on a newly initiated device driver instance with an <tt>'enumeration_level'</tt> argument of <tt>UDI_ENUMERATE_START</tt> to begin obtaining information about child devices of that device. The MA will loop, calling into the driver instance repeatedly with an <tt>'enumeration_level'</tt> argument of <tt>UDI_ENUMERATE_NEXT</tt>, and the driver will respond to each call with a <tt>udi_enumerate_ack()</tt>, giving information on a new child device. This will continue until the driver responds with a <tt>udi_enumerate_ack()</tt> that has an <tt>'enumeration_result'</tt> argument of <tt>UDI_ENUMERATE_DONE</tt>.

For devices with no children, the standard thing is to either always respond to a call to <tt>udi_enumerate_req()</tt> with a <tt>udi_enumerate_ack()</tt> that gives an <tt>'enumeration_result'</tt> of <tt>UDI_ENUMERATE_LEAF</tt> (i.e.: "I am a leaf device, and I have no children"), or to make use of the MA-provided stub function that will do exactly the same thing on behalf of the driver.

Generally, bus drivers ([[PCI]], [[ISA]], [[ACPI]], etc) and root enumerators ([[IDE|PCI IDE Controller]], [[SATA]], etc) will respond to <tt>udi_enumerate_req()</tt> with child devices, and most other types of drivers will simply return no children, because they generally have none. Each distinct child device identified by a parent driver should have a unique <tt>child_ID</tt> in the <tt>udi_enumerate_cb_t</tt> IPC control block.

The purpose of the <tt>udi_enumerate_req()</tt> and <tt>udi_enumerate_ack()</tt> pair is to enable the host kernel to construct its internal tree of devices. As a result, the information returned about each child device is information that is relevant to the kernel's device tree's construction.

==Returning child device attributes==

===Key structures===
<source lang="c">
udi_enumerate_cb_t (See above).

typedef struct {
	char attr_name[32];
	udi_ubit8_t attr_value[64];
	udi_ubit8_t attr_length;
	udi_instance_attr_type_t attr_type;
} udi_instance_attr_list_t;
</source>
===Synopsis===

The <tt>'attr_name'</tt> and <tt>'attr_value'</tt> pairs are, as you have probably guessed, key-value pairs for child-device attribute identification. On every call to <tt>udi_enumerate_ack()</tt> the driver returns the <tt>udi_enumerate_cb_t</tt> message block to the MA, with an array of these key-value pairs.

The attribute keys are specified by each UDI Bus Metalanguage. For example, a child-device returned by a PCI bus driver would have the following attribute keys:
* bus_type (attr_type: STRING).
* pci_vendor_id (attr_type: UBIT32).
* pci_device_id (attr_type: UBIT32).
* pci_revision_id (attr_type: UBIT32).
* pci_baseclass (attr_type: UBIT32).
* pci_sub_class (attr_type: UBIT32).
* pci_prog_if (attr_type: UBIT32).
* pci_subsystem_vendor_id (attr_type: UBIT32).
* pci_subsystem_id (attr_type: UBIT32).
* pci_unit_address (attr_type: UBIT32).
* pci_slot (attr_type: UBIT32).

Note well: the information returned by a parent device is agnostic of the type of device that the child is -- the device can be a modem, a network card, a graphics card, or anything else. The parent bus driver is not required to know what type of device the child is. It must only return information about the child device that allows the MA (kernel) to search the disk for a driver to match that device. For the PCI bus, this information is the PCI "vendor ID" and PCI "device ID".

The kernel will take this information and create a new node in its device tree. Then, for each child device, it will search the disk for drivers that match the instance attributes of the child device in question. The following are examples to aid understanding:
* PCI bus driver enumerates a device with attributes "pci_vendor_id" and "pci_device_id".
** pci_vendor_id = 0x8086, pci_device_id = 0x2411:
*** Kernel searches disk for device drivers matching this combination of vendor and device.
*** If the correct driver is installed, it will find that the combination matches the driver for "IDE Controller (UltraATA/66)", an Intel IDE controller.
*** The kernel loads the driver for "IDE Controller (UltraATA/66)".

* PCI bus driver enumerates a device with attributes "pci_vendor_id" and "pci_device_id".
** pci_vendor_id = 0x1002, pci_device_id = 0x5159:
*** Kernel searches disk for device drivers matching this combination of vendor and device.
*** If the correct driver is installed, it will find that the combination matches the driver for "Radeon 7000 series AGP", an ATI/AMD graphics chip.
*** The kernel loads the driver for "Radeon 7000 series AGP".

==Hot-Plug enumeration==
===Normal enumeration sequence===
The normal enumeration sequence goes as follows:
* The UDI MA (kernel) first calls into the driver (<tt>udi_enumerate_req()</tt>) with an <tt>'enumeration_level'</tt> argument of <tt>UDI_ENUMERATE_START</tt>.
** The driver responds to each call until it has enumerated all child devices, at which point it returns <tt>UDI_ENUMERATE_DONE</tt>.
** The MA keeps calling <tt>udi_enumerate_req()</tt> with <tt>'UDI_ENUMERATE_NEXT'</tt> until it gets a <tt>UDI_ENUMERATE_DONE</tt> from the driver.

===Hot-plug enumeration===
When the MA is done doing the "first" enumeration loop, it may want to know about any child devices that get added (or removed) at runtime. To this end, the MA can make a call to <tt>udi_enumerate_req()</tt> with an <tt>'enumeration_level'</tt> argument of <tt>UDI_ENUMERATE_NEW</tt>. <tt>UDI_ENUMERATE_NEW</tt> will cause the driver to hold on to the IPC message, and not respond until a child device change has been detected, such as a hot-add or hot-removal of a child device. At that point, the driver will respond to the kernel with information about the event.
* If the event is the hot-addition of a new device, the driver will respond with <tt>UDI_ENUMERATE_OK</tt>
** The kernel will post a new <tt>UDI_ENUMERATE_NEW</tt> request, and the driver will hold on to it again until a new hot-plug event occurs.
* If the event is the hot-removal of a child device, the driver will respond with <tt>UDI_ENUMERATE_REMOVED</tt>, and set the child_ID member of the message block to the ID of the child that has been removed.
** The kernel will post a new <tt>UDI_ENUMERATE_NEW</tt> request, and the driver will hold on to it again until a new hot-plug event occurs.
* If the event is the hot-removal of the parent node itself, such as a bus being removed without warning, etc, the running driver will respond with <tt>UDI_ENUMERATE_REMOVED_SELF</tt>.
** The kernel will most likely terminate the parent driver at this point, as well as all of its children.

[[Category:Hardware Detection]]
````

<a id="w003361"></a>

## W003361 — Bare Bones with NASM

Bron-ID: W003361
Namespace: Main (0)
Revisie: 30237 | Datum: 2026-03-16T15:07:57Z
Bron: https://wiki.osdev.org/Bare_Bones_with_NASM
Revisie-URL: https://wiki.osdev.org/index.php?title=Bare_Bones_with_NASM&oldid=30237

````text
This article is an extension to the [[Bare Bones]] article and describes how to use [[NASM]] in a Hello World kernel. Mentally add the following changes to the base article.

==  Booting the Operating System  ==
=== Bootstrap Assembly (NASM) ===
We will now create a file called boot.asm and discuss its contents. In this example, we are using the [[NASM|Netwide Assembler]] which is not part of your previously built cross-compiler toolchain and you will have to install it separately.

<syntaxhighlight lang="asm">
; Declare constants for the multiboot header.
MBALIGN  equ  1 << 0            ; align loaded modules on page boundaries
MEMINFO  equ  1 << 1            ; provide memory map
MBFLAGS  equ  MBALIGN | MEMINFO ; this is the Multiboot 'flag' field
MAGIC    equ  0x1BADB002        ; 'magic number' lets bootloader find the header
CHECKSUM equ -(MAGIC + MBFLAGS) ; checksum of above, to prove we are multiboot
                                ; CHECKSUM + MAGIC + MBFLAGS should be Zero (0)

; Declare a multiboot header that marks the program as a kernel. These are magic
; values that are documented in the multiboot standard. The bootloader will
; search for this signature in the first 8 KiB of the kernel file, aligned at a
; 32-bit boundary. The signature is in its own section so the header can be
; forced to be within the first 8 KiB of the kernel file.
section .multiboot
align 4
	dd MAGIC
	dd MBFLAGS
	dd CHECKSUM

; The multiboot standard does not define the value of the stack pointer register
; (esp) and it is up to the kernel to provide a stack. This allocates room for a
; small stack by creating a symbol at the bottom of it, then allocating 16384
; bytes for it, and finally creating a symbol at the top. The stack grows
; downwards on x86. The stack is in its own section so it can be marked nobits,
; which means the kernel file is smaller because it does not contain an
; uninitialized stack. The stack on x86 must be 16-byte aligned according to the
; System V ABI standard and de-facto extensions. The compiler will assume the
; stack is properly aligned and failure to align the stack will result in
; undefined behavior.
section .bss
align 16
stack_bottom:
resb 16384 ; 16 KiB is reserved for stack
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
        ; note, that if you are building on Windows, C functions may have "_" prefix in assembly: _kernel_main
	extern kernel_main
	call kernel_main

	; If the system has nothing more to do, put the computer into an
	; infinite loop. To do that:
	; 1) Disable interrupts with cli (clear interrupt enable in eflags).
	;    They are already disabled by the bootloader, so this is not needed.
	;    Mind that you might later enable interrupts and return from
	;    kernel_main (which is sort of nonsensical to do).
	; 2) Wait for the next interrupt to arrive with hlt (halt instruction).
	;    Since they are disabled, this will lock up the computer.
	; 3) Jump to the hlt instruction if it ever wakes up due to a
	;    non-maskable interrupt occurring or due to system management mode.
	cli
.hang:	hlt
	jmp .hang
.end:
</syntaxhighlight>

You can then assemble boot.asm using:
<syntaxhighlight lang="bash">nasm -felf32 boot.asm -o boot.o</syntaxhighlight>

== Kernel ==

<syntaxhighlight lang="asm">
BITS 32

VGA_WIDTH equ 80
VGA_HEIGHT equ 25

VGA_COLOR_BLACK equ 0
VGA_COLOR_BLUE equ 1
VGA_COLOR_GREEN equ 2
VGA_COLOR_CYAN equ 3
VGA_COLOR_RED equ 4
VGA_COLOR_MAGENTA equ 5
VGA_COLOR_BROWN equ 6
VGA_COLOR_LIGHT_GREY equ 7
VGA_COLOR_DARK_GREY equ 8
VGA_COLOR_LIGHT_BLUE equ 9
VGA_COLOR_LIGHT_GREEN equ 10
VGA_COLOR_LIGHT_CYAN equ 11
VGA_COLOR_LIGHT_RED equ 12
VGA_COLOR_LIGHT_MAGENTA equ 13
VGA_COLOR_LIGHT_BROWN equ 14
VGA_COLOR_WHITE equ 15

global kernel_main
kernel_main:
    mov dh, VGA_COLOR_LIGHT_GREY
    mov dl, VGA_COLOR_BLACK
    call terminal_set_color
    mov esi, hello_string
    call terminal_write_string
    jmp $
    

; IN = dl: y, dh: x
; OUT = dx: Index with offset 0xB8000 at VGA buffer
; Other registers preserved
terminal_getidx:
    push ax; preserve registers

    shl dh, 1 ; multiply by two because every entry is a word that takes up 2 bytes

    mov al, VGA_WIDTH
    mul dl
    mov dl, al

    shl dl, 1 ; same
    add dl, dh
    mov dh, 0

    pop ax
    ret

; IN = dl: bg color, dh: fg color
; OUT = none
terminal_set_color:
    shl dl, 4

    or dl, dh
    mov [terminal_color], dl


    ret

; IN = dl: y, dh: x, al: ASCII char
; OUT = none
terminal_putentryat:
    pusha
    call terminal_getidx
    mov ebx, edx

    mov dl, [terminal_color]
    mov byte [0xB8000 + ebx], al
    mov byte [0xB8001 + ebx], dl


    popa
    ret

; IN = al: ASCII char
terminal_putchar:
    mov dx, [terminal_cursor_pos] ; This loads terminal_column at DH, and terminal_row at DL

    call terminal_putentryat
    
    inc dh
    cmp dh, VGA_WIDTH
    jne .cursor_moved

    mov dh, 0
    inc dl

    cmp dl, VGA_HEIGHT
    jne .cursor_moved

    mov dl, 0


.cursor_moved:
    ; Store new cursor position 
    mov [terminal_cursor_pos], dx

    ret

; IN = cx: length of string, ESI: string location
; OUT = none
terminal_write:
    pusha
.loopy:

    mov al, [esi]
    call terminal_putchar

    dec cx
    cmp cx, 0
    je .done

    inc esi
    jmp .loopy


.done:
    popa
    ret

; IN = ESI: zero delimited string location
; OUT = ECX: length of string
terminal_strlen:
    push eax
    push esi
    mov ecx, 0
.loopy:
    mov al, [esi]
    cmp al, 0
    je .done

    inc esi
    inc ecx

    jmp .loopy


.done:
    pop esi
    pop eax
    ret

; IN = ESI: string location
; OUT = none
terminal_write_string:
    pusha
    call terminal_strlen
    call terminal_write
    popa
    ret

; Exercises:
; - Newline support
; - Terminal scrolling when screen is full
; Note: 
; - The string is looped through twice on printing. 

hello_string db "Hello, kernel World!", 0xA, 0 ; 0xA = line feed


terminal_color db 0

terminal_cursor_pos:
terminal_column db 0
terminal_row db 0
</syntaxhighlight>

Similar as before, to assemble it:
<syntaxhighlight lang="bash">nasm -felf32 kernel.asm -o kernel.o</syntaxhighlight>

[[Category:Tutorials]]
[[Category:Bare bones tutorials]]
[[Category:Assembly]]
````

<a id="w003362"></a>

## W003362 — User talk:Sortie/Calling Global Constructors

Bron-ID: W003362
Namespace: User talk (3)
Revisie: 14723 | Datum: 2013-05-24T11:29:08Z
Bron: https://wiki.osdev.org/User_talk:Sortie/Calling_Global_Constructors
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ASortie%2FCalling_Global_Constructors&oldid=14723
Doorverwijzing: Calling Global Constructors

````text
#REDIRECT [[Calling Global Constructors]]
````

<a id="w003363"></a>

## W003363 — Building GCC

Bron-ID: W003363
Namespace: Main (0)
Revisie: 30240 | Datum: 2026-03-16T15:12:47Z
Bron: https://wiki.osdev.org/Building_GCC
Revisie-URL: https://wiki.osdev.org/index.php?title=Building_GCC&oldid=30240

````text
{{Rating|1}}

In this tutorial we upgrade your system [[GCC]] to the most recent version. This helps you build a [[GCC Cross-Compiler]], as it is recommended that you build the cross-compiler with the same compiler version. With some care, you don't have to bootstrap a new system compiler, but it is risky and can potentially create trouble. You should use the same major compiler release for your system compiler and your cross-compiler.

See also the [http://gcc.gnu.org/install/ official instructions for building GCC].

== Introduction ==
Compilers are upgraded through a process called bootstrapping. At first you have your old system compiler that produces slow code and doesn't support all the new language features. You then use this old system compiler to build the new version of the compiler, hoping that the old compiler is able to build the new compiler (it supports all needed features). This produces a new compiler that produces fast code and supports all the new features, however the compiler itself is slow, because it was compiled with a compiler that produces slow code. In addition, the new compiler may be buggy because your old compiler was buggy, or perhaps the new compiler release has a bug.

The next step is to use your new slow compiler that produces fast code, and then build the new compiler again. This produces a fast compiler that produces fast code. However, the first compiler we built could be buggy, and the compiler we just built using it may be defect. We need to verify the correctness of our new fast compiler that produces fast code.

To solve that problem, we build the compiler ''a third'' time. Once we have built the third compiler using the second compiler, it should produce the very same output as the first compiler building the second, as both times we are using compilers that produce fast code and use the same source code. The compiler build system will then verify that the second and third compilers are identical, which gives you confidence in the bootstrap. If the second and third compilers are not identical, the bootstrap failed and you have encountered a compiler bug. Bootstrapping takes three times as long as just building a regular compiler, but it makes sure your toolchain is stable.

The last thing to do is run the compiler test suite so you can verify that it works correctly.

Notice how the same problems could occur if you use GCC 4.4 to build a GCC 4.8 cross-compiler. For that reason, it is recommended to use GCC 4.8 to build the GCC 4.8 cross-compiler, so you know no such problems could occur. You can't bootstrap a cross-compiler as it doesn't produce programs for the local operating system.

=== Do I need to bootstrap? ===
If your system compiler is the same major version as the compiler version you wish to build (for instance, if you have 4.6.2 and want 4.6.3), then you don't need to bootstrap. The minor releases are usually very compatible. You may be able to build another major version (such as 4.6.3 building 4.7.3) without a bootstrap, but it may not produce the ideal compiler. As the gap widens between the compiler versions, you are very likely to run into trouble. Each major compiler release is always able to build the next using a bootstrap. If you have the patience, it is possible to find a very old Linux system with a very old GCC release, and then bootstrap all the way up to the most recent GCC release, simply by iteratively upgrading a few major compiler releases at a time.

You may also be able to upgrade your system compiler through other channels.

=== Which compiler version do I want? ===
The newest [[GCC]] is recommended as it is the latest and greatest release. At this time, GCC 4.9.2 is the newest release. We recommend that you use the latest compiler release for your cross-compiler. You can also use older releases as they are usually reasonably good. If your local system compiler isn't too terribly old (at least GCC 4.6.0), you may wish to safe yourself the trouble and just pick the latest minor release (such as 4.6.3 if your system compiler is 4.6.1) for your cross-compiler. In that case, you don't need to bootstrap and you can proceed directly to building your [[GCC Cross-Compiler]].

You can view your current compiler version by invoking:

<syntaxhighlight lang="bash">gcc --version</syntaxhighlight>

If you have patience and wish to build yourself the latest and greatest cross-compiler, you'll want to bootstrap your system compiler before building your [[GCC Cross-Compiler]].

=== Do I need a new Binutils as well? ===
It certainly couldn't hurt, but if your Binutils isn't too horribly out of date, it should be good enough for your new system compiler. You can view your current Binutils version by invoking:

<syntaxhighlight lang="bash">ld --version</syntaxhighlight>

You probably need at least Binutils 2.22, or preferably the latest 2.25 release.

== Preparing for the build ==
<!-- Note how this section ([[Preparing GCC Build]]) is shared by [[Building GCC]] and [[GCC Cross-Compiler]]. -->
{{:Preparing GCC Build}}

== The Build ==
You need to decide where to install your new compiler. It is dangerous to replace your current system compiler and installation into system directories is likely a very bad idea. You also need to decide whether the new compiler should be installed globally or just for you. If you want to install it just for you (recommended), installing into <tt>$HOME/opt/gcc-x.y.z</tt> is normally a good idea. If you want to install it globally, installing it into <tt>/usr/local/gcc-x.y.z</tt> is normally a good idea.

Please note that we build everything out of the source directory tree, as is considered good practice. Some packages only support building outside, some only inside and some both (but may not offer extensive checking with make). Building GCC inside the source directory tree fails miserably, at least for older versions.

As the build can take a long time, it is recommended to make use of make's "-jN" option. This will allow make to use multiple threads to compile the programs, which will speed up things a LOT. Substitute N with a number; a good guideline is the number of core you CPU has, plus one. So for a 4 core CPU, you'll want to use :
<syntaxhighlight lang="bash">
make -j5
</syntaxhighlight>

=== Preparation ===
<syntaxhighlight lang="bash">export PREFIX="$HOME/opt/gcc-x.y.z"</syntaxhighlight>

=== Binutils ===
<syntaxhighlight lang="bash">
cd $HOME/src
mkdir build-binutils
cd build-binutils
../binutils-x.y.z/configure --prefix="$PREFIX" --disable-nls --disable-werror
make
make install
</syntaxhighlight>

'''--disable-nls''' tells Binutils not not include native language support. This is basically optional, but reduces dependencies and compile time. It will also result in English-language diagnostics, which the people on the [http://forum.osdev.org/ Forum] understand when you ask your questions.

=== GCC ===
:''See also the [http://gcc.gnu.org/install/configure.html offical instructions for configuring GCC].''

<syntaxhighlight lang="bash">
cd $HOME/src


# If you wish to build these packages as part of GCC:
mv libiconv-x.y.z gcc-x.y.z/libiconv # Mac OS X users
mv gmp-x.y.z gcc-x.y.z/gmp
mv mpfr-x.y.z gcc-x.y.z/mpfr
mv mpc-x.y.z gcc-x.y.z/mpc

# Or in new GCC versions, you can ask gcc to download the prerequisites
cd gcc-x.y.z
./contrib/download_prerequisites
cd $HOME/src # Returning the main src folder

mkdir build-gcc
cd build-gcc
../gcc-x.y.z/configure --prefix="$PREFIX" --disable-nls --enable-languages=c,c++
make
make install
</syntaxhighlight>

'''--disable-nls''' is the same as for Binutils above.

'''--enable-languages''' tells GCC to not to compile all the other language frontends it supports, but only C and C++. Even if you aren't going to use C++ to make your OS, you'll need it later on to port GCC.

'''--disable-bootstrap''' tells the compiler to not bootstrap itself against the current system compiler. This results in a much quicker compilation, but if the current and the new compiler differ too much in version, you will get a less robust compiler or weird errors.

Building and bootstrapping GCC may take quite a while, so sit back and relax, and enjoy that you are about to use the latest and greatest version of the GNU Compiler Collection.

Before you run <tt>make install</tt> and install the new compiler, if you have some additional patience, read the [http://gcc.gnu.org/install/test.html official testing instructions] and test your compiler for defects.

== Using the new Compiler ==
You can now run your new compiler by invoking something like:

<syntaxhighlight lang="bash">$HOME/opt/gcc-x.y.z/bin/gcc --version</syntaxhighlight>

To use your new compiler simply by invoking <tt>gcc</tt>, add <tt>$HOME/opt/gcc-x.y.z/bin</tt> to your <tt>$PATH</tt> by typing:

<syntaxhighlight lang="bash">export PATH="$HOME/opt/gcc-x.y.z/bin:$PATH"</syntaxhighlight>

This command will add your new compiler to your PATH for this shell session. If you wish to use it permanently, add the PATH command to your <tt>~/.profile</tt> configuration shell script or similar. Consult your shell documentation for more information.

== Building the Cross-Compiler ==
{{Main|GCC Cross-Compiler}}

Now that you have a system compiler that is up to date, you can build your cross-compiler for your new operating system. You can use the latest and greatest version of GCC as your cross-compiler safely because you use the same version to build it. Once you have built the actual cross-compiler, you can uninstall the compiler we built in this tutorial by removing the installation directory. The cross-compiler will not depend on the compiler that built it, just like our new compiler doesn't depend on the old system compiler it replaced.

[[Category:Compilers]]
[[Category:Tutorials]]
````

<a id="w003364"></a>

## W003364 — Preparing GCC Build

Bron-ID: W003364
Namespace: Main (0)
Revisie: 30954 | Datum: 2026-09-17T10:48:55Z
Bron: https://wiki.osdev.org/Preparing_GCC_Build
Revisie-URL: https://wiki.osdev.org/index.php?title=Preparing_GCC_Build&oldid=30954

````text
{{Rating|1}}

<noinclude>'''Note that this article is meant to be included into the [[GCC Cross-Compiler]] and [[Building GCC]] as they share this common section.'''</noinclude>

The GNU Compiler Collection is an advanced piece of software with dependencies. You need the following in order to build GCC:

* A Unix-like environment (Windows users can use the Windows Subsystem for Linux or Cygwin)
* Enough memory and hard disk space (it depends, 256 MiB will not be enough).
* GCC (existing release you wish to replace), or another system C compiler
* G++ (if building a version of GCC >= 4.8.0), or another system C++ compiler
* Make
* Bison
* Flex
* GMP
* MPFR
* MPC
* Texinfo
* ISL (optional)

=== Installing Dependencies ===

{| {{wikitable}}
|-
! &darr; Dependency / OS &rarr;
| Source Code
| Debian (Ubuntu, Mint, WSL, ...)
| Gentoo
| Fedora
| Cygwin
| OpenBSD
| Arch
|-
|How to install
|Normally
|'''<tt>sudo apt install</tt> ''foo'''''
|'''<tt>sudo emerge --ask</tt> ''foo'''''
|'''<tt>sudo dnf install</tt> ''foo'''''
|Cygwin GUI setup
|'''<tt>doas pkg_add</tt> ''foo'''''
|'''<tt>pacman -Syu</tt> ''foo'''''
|-
|Compiler
|N/A
|'''<tt>build-essential</tt>'''
|'''<tt>sys-devel/gcc</tt>'''
|'''<tt>gcc gcc-c++</tt>'''
|'''<tt>mingw64-x86_64-gcc-g++</tt>''' / '''<tt>mingw64-i686-gcc-g++</tt>'''
|Preinstalled
|'''<tt>base-devel</tt>'''
|-
|Make
|N/A
|'''<tt>build-essential</tt>'''
|'''<tt>dev-build/make</tt>'''
|'''<tt>make</tt>'''
|'''<tt>make</tt>'''
|Preinstalled
|'''<tt>base-devel</tt>'''
|-
|[https://www.gnu.org/software/bison/ Bison]
|[https://ftp.gnu.org/gnu/bison/]
|'''<tt>bison</tt>'''
|'''<tt>sys-devel/bison</tt>'''
|'''<tt>bison</tt>'''
|'''<tt>bison</tt>'''
|?
|'''<tt>base-devel</tt>'''
|-
|[https://github.com/westes/flex Flex]
|[https://github.com/westes/flex/releases]
|'''<tt>flex</tt>'''
|'''<tt>sys-devel/flex</tt>'''
|'''<tt>flex</tt>'''
|'''<tt>flex</tt>'''
|?
|'''<tt>base-devel</tt>'''
|-
|[https://gmplib.org/ GMP]
|[https://ftp.gnu.org/gnu/gmp/]
|'''<tt>libgmp3-dev</tt>'''
|'''<tt>dev-libs/gmp</tt>'''
|'''<tt>gmp-devel</tt>'''
|'''<tt>libgmp-devel</tt>'''
|'''<tt>gmp</tt>'''
|'''<tt>gmp</tt>'''
|-
|MPC
|[https://ftp.gnu.org/gnu/mpc/]
|'''<tt>libmpc-dev</tt>'''
|'''<tt>dev-libs/mpc</tt>'''
|'''<tt>libmpc-devel</tt>'''
|'''<tt>libmpc-devel</tt>'''
|'''<tt>libmpc</tt>'''
|'''<tt>libmpc</tt>'''
|-
|[https://www.mpfr.org/ MPFR]
|[https://ftp.gnu.org/gnu/mpfr/]
|'''<tt>libmpfr-dev</tt>'''
|'''<tt>dev-libs/mpfr</tt>'''
|'''<tt>mpfr-devel</tt>'''
|'''<tt>libmpfr-devel</tt>'''
|'''<tt>mpfr</tt>'''
|'''<tt>mpfr</tt>'''
|-
|[https://www.gnu.org/software/texinfo/ Texinfo]
|[https://ftp.gnu.org/gnu/texinfo/]
|'''<tt>texinfo</tt>'''
|'''<tt>sys-apps/texinfo</tt>'''
|'''<tt>texinfo</tt>'''
|'''<tt>texinfo</tt>'''
|'''<tt>texinfo</tt>'''
|'''<tt>base-devel</tt>'''
|-
|[http://isl.gforge.inria.fr/ ISL] (Optional)
|[http://isl.gforge.inria.fr/]
|'''<tt>libisl-dev</tt>'''
|'''<tt>dev-libs/isl</tt>'''
|'''<tt>isl-devel</tt>'''
|'''<tt>libisl-devel</tt>'''
|N/A
|N/A
|}

You need to have Texinfo installed to build Binutils. You need to have GMP, MPC, and MPFR installed to build GCC. GCC optionally can make use of the ISL library.

For instance, you can install '''<tt>libgmp3-dev</tt>''' on Debian by running the shell command: '''<tt>sudo apt install libgmp3-dev</tt>'''

'''Note:''' GCC requires at least ISL version 0.15.

'''Note:''' If you are using '''Cygwin''', it is recommended to install the <tt>'''libintl-devel'''</tt> package (I couldn't build a cross-compiler without this package)

=== Downloading the Source Code ===

Download the needed source code into a suitable directory such as <tt>$HOME/src</tt>:

* You can download the desired Binutils release by visiting the [https://gnu.org/software/binutils/ Binutils website] or directly accessing the [https://ftp.gnu.org/gnu/binutils/ GNU main mirror].

* You can download the desired GCC release by visiting the [https://gnu.org/software/gcc/ GCC website] or directly accessing the [https://ftp.gnu.org/gnu/gcc/ GNU main mirror].

'''Note:''' The versioning scheme used is that each fullstop separates a full number, i.e. Binutils 2.20.0 is newer than 2.9.0. This may be confusing, if you have not encountered this (quite common) versioning scheme yet, when looking at an alphanumerically sorted list of tarballs: The file at the bottom of the list is not the latest version! An easy way of getting the latest version is to sort by the last modified date and scrolling to the bottom.

'''Note (16-bit C/C++):''' If you need to build 16-bit C/C++ code, don't use GCC versions 13.X.X and 14.X.X! When accessing global variables, an error occurs: collect2: fatal error: ld terminated with signal 11 [Segmentation fault], core dumped. You can build GCC version 12.1.0 and use it.

=== Linux Users building a System Compiler ===
Your distribution may ship its own patched GCC and Binutils that is customized to work on your particular Linux distribution. You should be able to build a working cross-compiler using the above source code, but you might not be able to build a new system compiler for your current Linux distribution. In that case, try a newer GCC release or get the patched source code.

=== Gentoo Users ===

Gentoo offers [https://wiki.gentoo.org/wiki/Crossdev crossdev] to set up a cross-development toolchain:

    emerge -av crossdev
    crossdev --help
    PORTDIR_OVERLAY="/usr/local/crossdev" crossdev --stage1 --binutils <binutils-version> --gcc <gcc-version> --target <target>

This will install a GCC cross-compiler into a "slot", i.e. alongside already-existing compiler versions. You can install several cross-compilers that way, simply by changing target designations. An unfortunate downside is that it will also pull in gentoo patches and pass additional configure options that differ from the official [[GCC Cross-Compiler]] setup, and they might behave differently.

After the compilation finishes successfully, your cross-compiler is callable via <target>-gcc. You can also use gcc-config to toggle between compiler versions should you need to do so. Don't replace your system compiler with a cross-compiler. The package manager will also suggest updates as soon as they become available. 

You can uninstall the cross-compiler by calling ''crossdev --clean <target>''. Read the [https://wiki.gentoo.org/wiki/Cross_build_environment cross-development] document for additional information.

Note that the version numbers to binutils and gcc are ''Gentoo package versions'', i.e. there might be a suffix to the "official" (GNU) version that addresses additional patchsets supplied by the Gentoo maintainers. (For example, ''--binutils 2.24-r3 --gcc 4.8.3'' is the latest stable package pair at the time of this writing.) You can omit the version numbers to use the latest package available.

Portage uses overlays to store packages that are not part of the original package management. Crossdev needs one overlay where it can store its binutils and gcc packages before building them. You can configure one properly, or you can use PORTDIR_OVERLAY to point at where it should keep its package manager files. Using PORTDIR_OVERLAY is not a good idea with existing overlays, but by then you should know how you have personally set them up earlier anyway and how to do it properly. See [https://wiki.gentoo.org/wiki/Custom_repository#Crossdev].

=== Nix(OS) Users===
<code>nixpkgs</code> provides cross-compiled versions of most packages and their dependencies/compilers, that includes pre-compiled build tools for cross-compilation for every available architecture in <code>pkgs.pkgsCross</code>. This code snippet declares a nix shell with <code>i686-elf-gcc</code> :<syntaxhighlight lang="nix" line="1">
{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs =
    with pkgs; [
        pkgsCross.i686-embedded.buildPackages.gcc
    ];
}

</syntaxhighlight>Read more at [https://nix.dev/tutorials/cross-compilation.html the official Nix documentation page for cross-compilation].

===macOS Users===
macOS users need a replacement libiconv because the system libiconv is seriously out of date. macOS users can download the latest libiconv release by visiting the [https://gnu.org/software/libiconv/ libiconv website] or directly accessing the [https://ftp.gnu.org/gnu/libiconv/ GNU main FTP mirror]. Otherwise you may  get unresolved symbol errors related to libiconv when compiling GCC 4.3 or higher on OS X 10.4 and 10.5.

Install a new version (compile it yourself or use MacPorts) and add <tt>--with-libiconv-prefix=/opt/local</tt> (or <tt>/usr/local</tt> if you compiled it yourself) to GCC's <tt>./configure</tt> line. Alternatively you may place the libiconv source as gcc-x.y.z/libiconv and it will be compiled as part of the GCC compilation process. (This trick also works for MPFR, GMP, and MPC).

The makefiles of Binutils and GCC use the <syntaxhighlight lang="make" enclose="none">$(CC)</syntaxhighlight> variable to invoke the compiler. On OS X, this resolves to <syntaxhighlight lang="bash" enclose="none">gcc</syntaxhighlight> by default, which is actually <syntaxhighlight lang="bash" enclose="none">clang</syntaxhighlight>. Prior to OS X 10.8, the Clang that came with Xcode's Command Line Tools package was not able to build a working GCC. Users running OS X 10.7 or below may need to find and install GCC, either from [https://brew.sh Homebrew], or from somewhere on Apple's website. You can try with the old GCC that comes preinstalled on some macOS versions.

<syntaxhighlight lang="bash">
# This is only necessary for OS X users running 10.7 or below.
export CC=/usr/bin/gcc-4.2
export CXX=/usr/bin/g++-4.2
export CPP=/usr/bin/cpp-4.2
export LD=/usr/bin/gcc-4.2
</syntaxhighlight>

You will want to unset these exports once you compiled and installed the cross compiler.

'''Note for Lion users:''' If you're on Lion (or above) chances are that you don't have the "real" GCC since Apple removed it from the Xcode package, but you can still install it. You can do it via Homebrew or by compiling from source, both are perfectly described on [https://apple.stackexchange.com/a/38247 a StackExchange answer].

'''Note for Maverick users:''' You can build binutils-2.24 and gcc-4.8.3 (possible other version) with Xcode 5.1.1. Note that building GCC with LLVM is not officially supported and may cause interesting bugs, if you are willing to take this risk and save time building host-gcc just to compile a cross-gcc, follow this.
Install GMP, MPFR, Mpc with [https://www.macports.org/ MacPorts].

 sudo port install gmp mpfr libmpc

 ../binutils-2.24/configure --prefix=$PREFIX \
 --target=$TARGET \
 --enable-interwork --enable-multilib \
 --disable-nls --disable-werror

 ../gcc-4.8.3/configure --prefix=$PREFIX \
 --target=$TARGET \
 --disable-nls \
 --enable-languages=c,c++ --without-headers \
 --enable-interwork --enable-multilib \
 --with-gmp=/usr --with-mpc=/opt/local --with-mpfr=/opt/local

'''Note:''' There is an issue with port's GMP, so the version from OS X from /usr is used instead.
'''Note2:''' If you still have some errors, try making a case-sensitive APFS disk image using disk utility app and build from there

===Windows Users===
Windows users need to set up a Unix-like environment such as [[MinGW]] or [[Cygwin]]. It may well be worth looking into systems such as Linux and see if they fit your needs, as you commonly use a lot of Unix-like tools in operating systems development and this is much easier from a Unix-like operating system. '''If you have just installed the basic [[Cygwin]] package, you have to run the setup.exe again and install the following packages:''' GCC, G++, Make, Flex, Bison, Diffutils, libintl-devel, libgmp-devel, libmpfr-devel, libmpc-devel, Texinfo

MinGW + MSYS is an option, and as it addresses the native Windows API instead of a POSIX emulation layer, results in a slightly faster toolchain. Some software packages will not build properly under MSYS as they were not designed for use with Windows. As far as this tutorial is concerned, everything that applies to Cygwin also applies to MSYS unless otherwise specified. Make sure you install the C and C++ compilers, and the MSYS Basic System.

The "Windows Subsystem for Linux (Beta)", released with the Windows 10 Anniversary update is also an option for using a cross compiler. (Tested 08/08/2016 with GCC 6.1.0 and Binutils 2.27) This cross-compiler works reasonably fast, although being in beta state, it may not be ideal permanent development platform. Even though your files are accessible via Windows Explorer, you may encounter errors if you use it to unzip or edit files related to these packages. Complete these activities inside your WSL distribution.

'''Cygwin note:''' Cygwin includes your Windows <tt>%PATH%</tt> in its bash <tt>$PATH.</tt> If you were using DJGPP before, this could result in confusion as e.g. calling <tt>GCC</tt> on the Cygwin bash command line would still call the DJGPP compiler. After uninstalling DJGPP, you should delete the DJGPP environment variable and clear the <tt>C:\djgpp</tt> entry (or wherever you installed it) from your <tt>%PATH%</tt>. Likewise, it might be a bad idea to mix build environments in your system PATH variable.

'''MinGW note:''' Some MinGW-specific information on building a cross-toolchain can be found on the [http://www.mingw.org/wiki/HostedCrossCompilerHOWTO hosted cross-compiler how-to page] on the MinGW homepage.

'''Windows Subsystem for Linux (Beta) Note:''' You cannot have your cross compiler in the /mnt/c/ (or /mnt/"x") areas, as trying to compile your cross-compiler there will generate errors, whereas building to $HOME/opt/cross works perfectly. This is fixed with Windows Update KB3176929

===OpenBSD Users===
OpenBSD users might need to install "gcc" package from ports because base system's GCC is very outdated. If you want to build GCC, try to use the ports' version instead of the latest version available and apply all patches from ports to your build. Also, if the build fails during compiling lto-plugin, a temporary solution is to disable LTO altogether during configure stage of building GCC by adding <tt>--disable-lto</tt>

[[Category:Compilers]]
````

<a id="w003365"></a>

## W003365 — UDI IRQ Model

Bron-ID: W003365
Namespace: Main (0)
Revisie: 27997 | Datum: 2023-07-09T17:41:37Z
Bron: https://wiki.osdev.org/UDI_IRQ_Model
Revisie-URL: https://wiki.osdev.org/index.php?title=UDI_IRQ_Model&oldid=27997

````text
=References=
UDI Bus/Bridge Metalanguage v1.1.,
UDI Physical I/O specification v1.1.

=Core interface=
==Service Calls==
<source lang="c">
/* Provided by bus drivers; enables child devices to request and enable their IRQs. */
void udi_intr_attach_req(udi_intr_attach_cb_t *intr_attach_cb);
/* Provided by bus drivers; enables child devices to disable their IRQs. */
void udi_intr_detach_req(udi_intr_detach_cb_t *intr_detach_cb);

/* Provided by child drivers; called to notify a child device when an IRQ occurs, after the preprocessing stage. */
void udi_intr_event_ind(
	udi_intr_event_cb_t *intr_event_cb,
	udi_ubit8_t flags);

/* Provided by bus drivers; called when by the child driver to signal that it can handle more IRQs. */
void udi_intr_event_rdy(udi_intr_event_cb_t *intr_event_cb);
</source>

==Control block messages==
<source lang="c">
/* Message/control-block type used by udi_intr_attach_req(). */
typedef struct {
	udi_cb_t gcb;
	udi_index_t interrupt_idx;
	udi_ubit8_t min_event_pend;
	udi_pio_handle_t preprocessing_handle;
} udi_intr_attach_cb_t;

/* Message/Control-block type used by udi_intr_detach_req(). */
typedef struct {
	udi_cb_t gcb;
	udi_index_t interrupt_idx;
} udi_intr_detach_cb_t;

/* Message/Control-block type used by udi_intr_event_ind() and udi_intr_event_rdy() */
typedef struct {
	udi_cb_t gcb;
	udi_buf_t *event_buf;
	udi_ubit16_t intr_result;
} udi_intr_event_cb_t;
</source>

=IRQ Routing, IRQ controllers and Bus-IRQ mapping=
==Types of IRQs==
Naturally, when talking about device driver IRQ management APIs, one would wonder how the API could be mapped to an existing kernel's IRQ infrastructure.

A well designed kernel designs its IRQ handling into several groups of related IRQs. Should your design not be as complete as the one presented here, you could consider this a useful bonus design guide for private implementation.
* Pin-based IRQs.
* Message signaled IRQs.
* Non-Maskable interrupts.

==The slow move from pin-based IRQs to MSI IRQs==
Most devices, due to the fact that the majority of chipsets available today only support pin-based IRQs, export only one "interrupt", in spite of the fact that many devices may actually be optimally manufactured with more than one; for example, a network card could be designed to have two interrupts: one for signaling frame receipt, and one for say, signaling that a DMA transfer from a send operation has been completed. It has been theorized that as message-signaled IRQs come into more common use, the trend will lean more toward devices exposing functions using more than one interrupt per device. UDI genericizes this possibility by allowing drivers to indicate which interrupt they are trying to attach to the system. This is the purpose of the <tt>'interrupt_idx'</tt> member of the control block messages.

==IRQ Routing modes==
The IBM-PC can use any of three different IRQ routing modes: Symmetric I/O mode, i8259a-PIC mode, and "mixed mode" which is explicitly not supported by the ACPI specification. Most kernels use either PIC or Symmetric I/O mode for IRQ routing. A kernel is expected to detect the IRQs associated with a bus and maintain this information in some form of IRQ information repository.

==Buses, IRQs and Bus <-> device IRQ mapping==
Each bus on a chipset may export zero or more IRQs of any kind, be they pin-based, message signaled or NMIs. For legacy buses like ISA, the device attached to each pin of the IRQ controllers is known. The i8254 PIT is assigned ISA-IRQ-0, and connected to pin-0 of the i8259a-PIC, and so forth. When the I/O APICs are in use, ISA IRQs are ID-mapped to the first 16 Global System IRQs, so generally, discovering where the ISA device IRQs are connected is very easy. ISA device detection is often very easily doable, and actually works well with a lot of assumptions.

But for other buses such as PCI, discovering which device is attached to which pin(s) on the i8259a-PICs or the I/O APICs is more difficult. PCI device <-> IRQ-pin mapping requires the reading of tables in RAM, and possibly support for ACPICA. Regardless, a kernel must detect and know the mappings of devices to their IRQ pins. It should be possible for a kernel to be given a request to enable "IRQ-3" on the "PCI Bus"; and the kernel should be able to translate that using something along these lines, until it arrives at the correct PIC or I/O APIC pin to handle the request:
* Request: "Enable PCI IRQ-3".
** Translate: "PCI IRQ-3" to "PCI-Link D".
** Look-up PCI IRQ steering information.
** Look-up current mapping for PCI-Link-D.
** Ensure that the kernel hasn't, for example, steered the PCI link to a non-default PIC or I/O-APIC pin.
** Get the correct PIC or I/O-APIC pin that corresponds to PCI-Link-D.
*** Enable the PIC or I/O APIC pin in question.

You can probably see that "PCI-IRQ-3" may actually correspond to I/O APIC pin 20. This is not information that PCI drivers should have to care about; a PCI driver only cares about PCI bus notation. This is the case with UDI drivers -- a UDI driver that uses the PCI bus will make service calls that will ask the kernel to enable <tt>'interrupt_idx'</tt> number <tt>3</tt> for that device. PCI does allow each device to have up to 4 interrupts, so a PCI device can ask the kernel to enable any of these 4 interrupts. The kernel must then know how to translate that PCI Interrupt number into a PCI Link number, and then furthermore into an I/O APIC or PIC pin number. The same goes for any other type of bus -- devices should not care about IRQ-controller pins. They should only care about their own private IRQ-ID-number which is relative to their parent bus.

=General sequence for managing IRQs=
In general, UDI drivers will just call <tt>udi_intr_attach_req()</tt>, passing the index of a device-specific interrupt that the driver wishes to enable, and it is left up to the kernel to translate this information to a pin-number or MSI-IRQ index or NMI, depending on what that device-specific IRQ ID maps to. Drivers do not know about or care about the particular IRQ routing scheme in use by the kernel (i8259a-PIC or Symmetric I/O mode), whether or not one of its IRQs has been steered to a non-default pin, or indeed the type of IRQ that is being enabled. It is up to the kernel to decide, for example, whether to use Message-Signaled IRQ functionality for a device if it supports it.

Upon a successful call to <tt>udi_intr_attach_req()</tt>, the driver is expected to supply a number of message control blocks to the kernel for the purpose of IRQ handling, and from that point on, the device should expect to receive any number of IRQ notifications (signaled via a call into the driver at its entry point for <tt>udi_intr_event_ind</tt>).

TODO: Discuss interrupt preprocessing here.

[[Category:Uniform Driver Interface]]
[[Category:Interrupts]]
````

<a id="w003367"></a>

## W003367 — UDI Regions

Bron-ID: W003367
Namespace: Main (0)
Revisie: 29082 | Datum: 2024-06-07T13:37:35Z
Bron: https://wiki.osdev.org/UDI_Regions
Revisie-URL: https://wiki.osdev.org/index.php?title=UDI_Regions&oldid=29082

````text
[[Category:Uniform Driver Interface]]

This article seeks to explain a foundational principle to UDI drivers: the UDI Region. This can be considered advanced reading, and is probably not useful to persons who do not understand asynchronous programming, or are not interested in implementing a UDI environment.

=Introduction to UDI driver regions=
A region is a set of readable/writable data. UDI drivers do not have global, writable variables or data structures. All writable data is to be kept within a region and allocated at runtime. All global data is to be purely read-only and treated as "constant" and immutable.

If there is a variable which is shared among two "threads" of execution ("thread" being used loosely here, since UDI does not dictate any particular threading model), this shared variable is to be modified via IPC between the two threads, and only one of the threads may "own" the variable. The other shall send IPC messages to the "owning" thread when it wishes to modify that data.

As a result of this, it is impossible for two threads to modify a single variable at the same time. The UDI "Region" is the ''unit of synchronization'' of UDI shared data.

=Threading models for UDI drivers=
In the above introduction, the article loosely refers to regions as having "threads" attached to them, as if a "region" implies a new "thread". This is not necessarily true, and is simply '''one''' model of implementation. The key requirement is that region data not be writable by two threads at the same time, regardless of the threading model used in a kernel.

For the purpose of being as accommodating as possible to the reader, the author has decided to discuss some of the conceivable threading models that can be used with UDI drivers. Note that for all of these cases, the driver does not know or care how the kernel decides to apply threading to it, if threads are used at all.

==One thread per region==
In this model, the requirement that ''no two threads be executing on the same region data at once'' is met by instantiating a thread for each region in the driver. Subsequently, the thread will de-queue all messages sent to that region (via channel IPC calls), and dispatch them one by one. In this model, it is impossible for another region's thread to attempt to read or write another region's data since each thread is only responsible for, and interested in its own data.

===Advantages===
* Very naturally supports asynchronous programming.
* Scales very well to compute clusters, NUMA machines, etc.
* Migrating regions as single units across NUMA nodes, machines in a cluster, etc is very simple.
* No locking is needed.
* Applies the full benefits of asynchronous programming while also eliminating the need for costly locking.

===Disadvantages===
* IPC messages between regions may incur overhead that can be avoided in many cases.


This approach is best suited to high scaling microkernels or hybrid kernels and distributed systems.

==One thread per channel==
In this model, every channel of IPC established between regions is given its own thread. Each IPC channel thread de-queues messages from its IPC queue, and attempts to modify the region data that the IPC request would require it to. In this model, when a channel thread attempts to read or write to region data, it may encounter another channel thread ''already accessing'' that data. Locking would be needed for each region data block to ensure that two channel thread do not access the region data at once.

===Advantages===
* Very naturally supports an asynchronous programming model.
* Conceptually (not practically) scales well to NUMA systems and clusters.
* Migrating regions to other nodes is "easy", though more difficult than the "One thread per region" approach.

===Disadvantages===
* Requires the use of locking on per-region data.
* Introduces a large amount of contention, as the number of IPC channels and the complexity of the driver increase.
* Introduces a large amount of kernel metadata overhead since Thread Control Blocks and Thread Stacks require kernel memory.
* Uses resources which could have been saved by a better design.


This approach is probably not a best fit for any implementation (at all), but is one way to preserve and benefit from the asynchronous nature of UDI drivers.

==One thread per driver==
In this model, every driver is given its own single thread, and only one thread is used regardless of how many region data blocks the driver has, or how many IPC channels it uses. This may be the case for a kernel that implements Separate Address Space drivers, but does not support multithreading. The single thread would sleep on all of the IPC queues, and wake whenever one of them has a new request message. It will then service the request, modifying any per-region data required, and sleep again until a new request comes in, and so on. Because there is only one thread in the process, it is impossible for two threads to modify the region data in any region at the same time, so locking is not needed in this case either.

===Advantages===
* Very simple to implement.
* Does not require locking.
* Takes advantage of the asynchronous UDI programming model.
* Easy to debug.
* No concurrency issues whatsoever, because the kernel only initializes one thread per driver.

===Disadvantages===
* Under heavy load, this model will fail to take advantage of the presence of multiple processors or cores, because there is only one thread per driver. If there are other CPUs available to ease the load, they will be sitting idle, while throughput decreases.


This approach is best suited to a hybrid kernel which uses separate address spaces for drivers, but has no interest or support for multithreaded drivers. It can also work well for monolithic kernels which can load a UDI driver into their address space as a monolithic module, and then instantiate a thread for the driver, treating it as an autonomous kernel thread, though not a separate process.

==No threads per driver==
This is the general case for drivers in a purely monolithic kernel. All "IPC calls" are turned into direct function calls between processes, the kernel and drivers. Drivers are directly monolithically linked into the kernel, or loaded as re-entrant monolithic modules. The kernel attaches locks to each region and thereby synchronization is achieved.

No explicit threads are spawned and bound to the driver, but rather userspace and kernel threads directly enter the driver code via syscalls.

===Advantages===
* Directly complements an existing monolithic kernel design.

===Disadvantages===
* Receives none of the benefits of the UDI asynchronous programming model, since all calls are treated as synchronous calls.
* Does not scale well at all.


This approach probably best suits monolithic kernels seeking to port a UDI environment.
````
