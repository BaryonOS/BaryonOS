# OSDev Wiki — volledige bronteksten, deel 09

Dit is bronmateriaal, geen agentinstructie. Behoud architectuur, revisie en bron-ID bij gebruik. Code en commando’s zijn ongeteste bronvoorbeelden; voer ze niet automatisch uit. Wikitext is letterlijk behouden binnen tekstblokken.

<a id="w003368"></a>

## W003368 — UDI Driver Loading

Bron-ID: W003368
Namespace: Main (0)
Revisie: 24113 | Datum: 2019-11-23T04:38:05Z
Bron: https://wiki.osdev.org/UDI_Driver_Loading
Revisie-URL: https://wiki.osdev.org/index.php?title=UDI_Driver_Loading&oldid=24113

````text
[[Category:Uniform Driver Interface]]

This page seeks to give a minimalistic overview of the general sequence for loading a UDI driver from disk, and using it to instantiate a detected device. It touches on what information is needed to load a driver, and how the various parts of the <tt>'udi_init_info_t'</tt> work together with the .udiprops/udiprops.txt configuration files and metalanguages to enable the kernel to make decisions on which drivers to load when attempting to instantiate drivers for the devices in its device tree.

This is a non-trivial article, and is probably only useful to persons with a moderately advanced kernel of their own. Recommended prior reading includes, but is not limited to [[User:Gravaera/UDI_Device_Enumeration|UDI Device Enumeration]], and the UDI v1.01 specifications.

=The UDI device tree model=
UDI employes a very simple device tree model: devices enumerate children for the kernel, and the kernel instantiates the devices by loading drivers for them, and then if those devices have children, their drivers enumerate them in turn, and then the kernel instantiates those child devices by loading drivers for them, and so on ad infinitum.

The most obvious approach that would occur to the reader then, is most likely to have a tree-like structure, or some other such structure to hold device nodes in the kernel's device tree, and build the device tree from the information returned by UDI drivers.

==The relationship between metalanguages and device enumeration==
===Key structures===
<source lang="c">
/* The structure used to represent a single named attribute and its value. */
typedef struct {
	char attr_name[UDI_MAX_ATTR_NAMELEN];
	udi_ubit8_t attr_value[UDI_MAX_ATTR_SIZE];
	udi_ubit8_t attr_length;
	udi_instance_attr_type_t attr_type;
} udi_instance_attr_list_t;

/* The structure used to represent a child device when being enumerated by a parent driver.
 * Notice the "attr_list" struct member, which is essentially an array of named
 * attributes.
 **/
typedef struct {
	udi_cb_t gcb;
	udi_ubit32_t child_ID;
	void *child_data;
	udi_instance_attr_list_t *attr_list;
	udi_ubit8_t attr_valid_length;
	const udi_filter_element_t *filter_list;
	udi_ubit8_t filter_list_length;
	udi_ubit8_t parent_ID;
} udi_enumerate_cb_t;
</source>

===Device Instance Attributes===
When a UDI driver enumerates its child devices to the kernel, it does so using a series of named "instance attributes". The particular attributes returned by the driver for the child device depend on the type of device that the parent is: for example, if the parent is a PCI bus, then it would return attributes that describe a PCI device. If the parent device is an ISA root bus device, it would return attributes that best describe an ISA device.

Who decides what attributes "best" describe a PCI child device? Or what guarantee is there that different drivers for the same bus won't return different named attributes, or employ custom naming schemes for attributes? The answer is quite simple: the attributes returned by enumerator drivers for a particular bus, are determined by the UDI metalanguage for that bus. For example, the UDI PCI Metalanguage specifies the bindings for PCI drivers, relative to the core specification. In more detail, we can examine the PCI Metalanguage's actual attributes, as it defines them:
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

These attributes "best" describe a PCI child device to the kernel. This information is complete, and more than sufficient for enabling the kernel to add this new child device to its device tree. The <tt>'pci_vendor_id'</tt> and <tt>'pci_device_id'</tt> can then be used to search the disk for a driver that matches those attributes (PCI device/vendor ID).

Naturally, this is the recommended, and officially standardized Metalanguage for the PCI bus, but the UDI specification is generic in nature, and new metalanguages can be developed with ease; it is perfectly possible for a <del>retarded</del> rogue party to choose to define a custom PCI Metalanguage binding. Thankfully, even this would not cause any problems with interoperability: the rogue party's custom PCI metalanguage would actually seamlessly fit into existing UDI environments without any modification. The reasons for this are beyond the scope of this article -- suffice it to say that competing standards are not an issue for, or a threat to UDI compliant kernels. Drivers will "just work".

===Synopsis so far===
Parent drivers enumerate their child devices to the kernel, and they tell the kernel about the attributes of their child devices using attributes specified by a standardized UDI committee sanctioned metalanguage, or a custom metalanguage. The kernel can then use these attributes as criteria when searching the disk for the correct or preferred driver for a device.

==The relationship between Device Instance Attributes and udiprops.txt/.udiprops==
===Introduction to Udiprops===
Much like Microsoft Windows' .INI configuration information files that come with drivers, UDI uses a text based configuration approach to distributing drivers. This configuration shall always be distributed with drivers in one of the following forms:
* For binary-only drivers, the configuration information '''shall''' be embedded into the driver's executable binary object in an executable section called "'''.udiprops'''".
* For source distributions of drivers, the configuration information '''shall''' be found in a file called "'''udiprops.txt'''" within the source distribution.

===What is udiprops used for?===
Udiprops gives information about how to compile the driver (for source distributions), the metalanguages used in the driver binaries, and it also gives information about '''how to match the driver against instance attributes''' when a kernel is searching the disk for the correct driver to use to instantiate a given device.

Naturally, it would occur to any fast-thinking reader that this means that the kernel would need to search the udiprops of each driver when trying to find the correct driver for a device. Naturally, it would also subsequently occur to a faster-thinking reader that a more mature implementation could have a database or registry of sorts with the instance attribute information stripped out to enable fast index lookups for driver attributes. The actual method used to search the drivers available on disk is not specified by the UDI Core Specification. A kernel can choose to slowly read each driver's udiprops, or compile the information in a database, or even not to do either of these, and just load a fixed set of drivers that it knows it needs. All of this is up to the implementor.

===Matching device instance attributes against udiprops===
<source lang="text">
message 1001 Fake driver written by DeceptionCorp Inc.
message 1002 DeceptionCorp Inc.
message 1003 PO-BOX #123 Fake Valley, Not Real City, USA; Tel 765-4321;

name     1002
supplier 1002
contact  1003

requires udi_pci 0x101
requires udi_sysbus 0x101

meta 1 udi_pci
meta 2 udi_sysbus

message 200 Example VGA graphics device, model 1
message 201 Example VGA graphics device, model 2

device 200 1 \
   bus_type      string pci \
   pci_vendor_id ubit32 0x8086 \
   pci_device_id ubit32 0x1234

device 201 1 \
   bus_type      string pci \
   pci_vendor_id   ubit32 0x8086 \
   pci_device_id   ubit32 0x2345 \
   pci_revision_id ubit32 0x02

device 201 2 \
   bus_type           string system \
   sysbus_mem_addr_lo 0xA0000 \
   sysbus_mem_addr_hi 0x0 \
   sysbus_mem_size    0x20000 \
   sysbus_io_addr_lo  0x3C0 \
   sysbus_io_addr_hi  0x0 \
   sysbus_io_size     0x8 \
   sysbus_intr0       16
</source>

The above is an example udiprops which has been constructed on the fly to meet the needs of this article. It is simplified to the extent of being non-compliant, so that the bare minimum information needed to enable a kernel to match a driver against instance attributes can be explained. That said, a few extra lines have been added (name, supplier, contact) to make it look more wholesome. It demonstrates declaration of a single driver that supports multiple different devices (model 1 and model 2), and shows how to declare a single device that can be found on multiple buses (Model 2 can be found on the PCI bus or the ISA bus, as is very common for VGA devices).

===="Device" lines in udiprops====
Each "device" line in a udiprops configuration describes one set of attribute match values which are meant to be compared against a given device in the kernel's device tree. In most cases, these will describe separate devices, but it is also possible for a device to be configurable on multiple different buses. For example, a VGA graphics controller may be enumerated as part of the ISA bus, or it may be enumerated from the PCI bus. The I/O ports and memory locations will usually remain the same regardless of which bus it is exposed on, but naturally, each device is different, and this may not hold true for every device.
<source lang="text">
device <device_name_string_index> <bus_metalanguage_index> \
   <attribute_name> <attribute_type> <attribute_value>
</source>

"Device" declaration lines with the same <tt>'<device_name_string_index>'</tt> number refer to the same device. "Device" declaration lines with differing <tt>'<device_name_string_index>'</tt> numbers refer to different devices. As can be seen in the example udiprops above, there are two distinct devices which our example driver claims to support:
* Example VGA graphics device, model 1; message-string-index 200.
* Example VGA graphics device, model 2; message-string-index 201.


The <tt>'<bus_metalanguage_index>'</tt> enables the kernel to know what Metalanguage libraries need to be loaded to properly start the driver process when the driver is loaded for a match against this "device" line. For example, the first two devices would require the <tt>'udi_pci'</tt> bus metalanguage libraries to be loaded, while the third device declaration would instead require the <tt>'udi_sysbus'</tt> library. Our fake driver's udiprops claims to enumerate devices on two buses:
* udi_pci (metalanguage index 1).
* udi_sysbus (metalanguage index 2).


The named device instance attributes which follow the <tt>'<bus_metalanguage_index>'</tt> all follow the attributes mandated by the particular UDI bus Metalanguage that the device declaration uses. For example, the UDI PCI Metalanguage uses <tt>'bus_type'</tt>, <tt>'pci_vendor_id'</tt> and <tt>'pci_device_id'</tt> among other attributes for device attributes. ''A driver need not specify every named attribute in a "device" declaration, but supplying a large number of attributes will make locating the correct driver easier.'' For the PCI bus though, drivers really need only specify <tt>'pci_vendor_id'</tt> and <tt>'pci_device_id'</tt>, because as the user most likely already knows, these two are for the most part all that are needed to match a PCI device against its driver.

A sharp reader would also have noticed that the named instance attributes listed for the ISA sysbus "device" declaration of "Model 2" '''do not have''' the same attributes as a PCI device. This is because the attributes of a PCI device differ from the attributes of an ISA device, and the UDI Sysbus Metalanguage defines different attributes for sysbus devices. Different bus metalanguages use different attributes to describe child devices because different buses work differently. The same would apply to say, IDE devices enumerated by an IDE driver. IDE/PATA disks obviously cannot rationally be expected to be identified by <tt>'pci_vendor_id'</tt> and <tt>'pci_device_id'</tt>.

==Conclusion==
So in conclusion, our fake driver above declares two distinct devices. The first can only be found on the PCI bus, and the second can be found on both the PCI and the ISA buses. The kernel would match the device tree instance attributes against the udiprops of each driver on disk, or against its database, or whatever its search method may be, and find the best driver to use, and load that driver to service the device.
````

<a id="w003371"></a>

## W003371 — User:Blacklight

Bron-ID: W003371
Namespace: User (2)
Revisie: 14818 | Datum: 2013-06-04T04:38:21Z
Bron: https://wiki.osdev.org/User:Blacklight
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ABlacklight&oldid=14818

````text
I do stuff. My OS is called '''BlacklightEVO'''.
````

<a id="w003372"></a>

## W003372 — How Do I Use A Debugger With My OS

Bron-ID: W003372
Namespace: Main (0)
Revisie: 14839 | Datum: 2013-06-05T16:45:48Z
Bron: https://wiki.osdev.org/How_Do_I_Use_A_Debugger_With_My_OS
Revisie-URL: https://wiki.osdev.org/index.php?title=How_Do_I_Use_A_Debugger_With_My_OS&oldid=14839
Doorverwijzing: Kernel Debugging

````text
#REDIRECT [[Kernel Debugging]]
[[Category:FAQ]]
````

<a id="w003373"></a>

## W003373 — Talk:How Do I Use A Debugger With My OS

Bron-ID: W003373
Namespace: Talk (1)
Revisie: 14822 | Datum: 2013-06-04T05:11:40Z
Bron: https://wiki.osdev.org/Talk:How_Do_I_Use_A_Debugger_With_My_OS
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AHow_Do_I_Use_A_Debugger_With_My_OS&oldid=14822
Doorverwijzing: Talk:Kernel Debugging

````text
#REDIRECT [[Talk:Kernel Debugging]]
````

<a id="w003374"></a>

## W003374 — UDI Metalanguages

Bron-ID: W003374
Namespace: Main (0)
Revisie: 17189 | Datum: 2014-12-01T20:57:29Z
Bron: https://wiki.osdev.org/UDI_Metalanguages
Revisie-URL: https://wiki.osdev.org/index.php?title=UDI_Metalanguages&oldid=17189

````text
UDI Metalanguages can be described succinctly as doing two things:
* Defining APIs for IPC between drivers (and the host kernel's userspace).
* Defining extensions to the Core UDI Specification to enable it to meet the needs of special device driver types.

Metalanguages are used in every UDI driver, since they are essentially APIs. Any API call that you use to call into a UDI driver is defined by a Metalanguage -- Metalanguages ''define UDI APIs''. For example, if you were to call <tt>udi_devmgmt_req()</tt> to ask a driver to "Prepare to Suspend", you would be calling a function that is part of the UDI Management Metalanguage API. This Management Metalanguage API has other functions grouped under it as well, and together these functions make up the Management Metalanguage API:
<source lang="c">
typedef const struct {
  udi_usage_ind_op_t *usage_ind_op;
  udi_enumerate_req_op_t *enumerate_req_op;
  udi_devmgmt_req_op_t *devmgmt_req_op;
  udi_final_cleanup_req_op_t *final_cleanup_req_op;
} udi_mgmt_ops_t;
</source>

Metalanguage APIs form the basis of device-class APIs in UDI.

== Metalanguage Operations Vectors ==
In the above example for the UDI Management Metalanguage, all of the Management Metalanguage API functions were grouped into one function-pointer struct. This is because the Management Metalanguage is a very '''simple''' API, so it can't really be separated into groups of related functions. By all means, every API can be exposed this way -- as a single congested block of functions with no separation or organization.

But it is beneficial to allow for the APIs to be broken into sub-groups of functions. Consider for example, yet again, a network card's driver. Most network card drivers can be split up into several groups of functions:
* Functions related to device '''control''' and '''status''' management.
* Functions related to '''sending''' network frames.
* Functions related to '''receiving''' network frames from the network.

A naive implementation of a Network Metalanguage API could kludge all of its function together, regardless of their logical sub-grouping, similarly to this example below, where a naive Network Metalanguage groups all of its control, send and receive operations together indiscriminately:
<source lang="c">
typedef const struct {
/* Control and status related functions */
	udi_nd_bind_req_op_t *nd_bind_req_op;
	udi_nd_unbind_req_op_t *nd_unbind_req_op;
	udi_nd_enable_req_op_t *nd_enable_req_op;
	udi_nd_disable_req_op_t *nd_disable_req_op;
	udi_nd_ctrl_req_op_t *nd_ctrl_req_op;
	udi_nd_info_req_op_t *nd_info_req_op;
/* Send functions */
	udi_nd_tx_req_op_t *nd_tx_req_op;
	udi_nd_exp_tx_req_op_t *nd_exp_tx_req_op;
/* Frame receipt notification functions. */
	udi_nd_rx_rdy_op_t *nd_rx_rdy_op;
} udi_nd_ctrl_ops_t;
</source>

And it would be fine, and functional -- there is nothing technically wrong with this: the host kernel can call into the driver just fine, and the driver can respond to the requests just fine as well. However, if instead the Metalanguage API was to be redesigned, and all of these functions broken into groups of related logical sub-functions:
<source lang="c">
typedef const struct {
	udi_channel_event_ind_op_t *channel_event_ind_op;
	udi_nd_rx_rdy_op_t *nd_rx_rdy_op;
} udi_nd_rx_ops_t;

typedef const struct {
	udi_channel_event_ind_op_t *channel_event_ind_op;
	udi_nd_tx_req_op_t *nd_tx_req_op;
	udi_nd_exp_tx_req_op_t *nd_exp_tx_req_op;
} udi_nd_tx_ops_t;

typedef const struct {
	udi_channel_event_ind_op_t *channel_event_ind_op;
	udi_nd_bind_req_op_t *nd_bind_req_op;
	udi_nd_unbind_req_op_t *nd_unbind_req_op;
	udi_nd_enable_req_op_t *nd_enable_req_op;
	udi_nd_disable_req_op_t *nd_disable_req_op;
	udi_nd_ctrl_req_op_t *nd_ctrl_req_op;
	udi_nd_info_req_op_t *nd_info_req_op;
} udi_nd_ctrl_ops_t;
</source>

Then the kernel would be calling into a different struct for each of the groups of functions (control, send and receive). This would also enable the kernel to make assumptions about the grouping of these functions. Whereas before, the kernel could not assume that any function within the "big kludge of functions" was functionally distinct from the others, now it can assume that each individual structure containing function pointers ''implies'' a logically separate flow of signaling into the driver.

This enables the kernel to create separate IPC request queues (a.k.a., '''UDI Channels''') for each group of functions (if it so desires); in the case of our example Network Metalanguage API, separate channels could be created for control IPC messages, send IPC messages and receive IPC notifications. Of course, this only applies to microkernels and hybrid kernels, because monolithic kernels call directly into their drivers without using any messages at all. A monolithic kernel would just ignore all of this.

Any single Metalanguage API can specify that its API be broken up into multiple function-groups. That is, any Metalanguage can specify ''<N>'' different functions, grouped into ''<M>'' different groupings, as seen fit by that Metalanguage. Drivers just provide their entry points for the functions, grouped as recommended by the Metalanguage specification.

== Conclusion ==
Metalanguages extend the UDI Core Specification by introducing new APIs for adoption and standardization. Driver vendors can also choose to provide new metalanguages if they see fit. And drivers can provide any number of APIs as needed.

For example, a graphics driver can expose an OpenGL API via an OpenGL Metalanguage, and also expose a Direct 3D API via a D3D Metalanguage.

[[Category:Uniform Driver Interface]]
````

<a id="w003375"></a>

## W003375 — Category:Drivers

Bron-ID: W003375
Namespace: Category (14)
Revisie: 14846 | Datum: 2013-06-13T07:46:48Z
Bron: https://wiki.osdev.org/Category:Drivers
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ADrivers&oldid=14846

````text
Drivers are at the core of any operating system. Without them, the kernel (and OS) could not interface with hardware. Some drivers are built into the kernel, while others can be loaded from external files.

[[Category:Kernel]]
````

<a id="w003376"></a>

## W003376 — Category:Driver Interfaces

Bron-ID: W003376
Namespace: Category (14)
Revisie: 14847 | Datum: 2013-06-13T07:49:31Z
Bron: https://wiki.osdev.org/Category:Driver_Interfaces
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ADriver_Interfaces&oldid=14847

````text
Driver interfaces are the way drivers inergrate with the kernel. Without them, the kernel could not load, control, or service drivers.

[[Category:Drivers]]
````

<a id="w003377"></a>

## W003377 — Category:Uniform Driver Interface

Bron-ID: W003377
Namespace: Category (14)
Revisie: 27019 | Datum: 2022-03-28T21:23:01Z
Bron: https://wiki.osdev.org/Category:Uniform_Driver_Interface
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AUniform_Driver_Interface&oldid=27019

````text
Uniform Driver Interface, or UDI, is a driver interface created by several large corporations. Although Windows can have any driver it wants and GNU discourage it for philosophical reasons, the advantages it presents to a hobbyist developer are obvious, although it has fallen dormant.

[[Category:Driver Interfaces]]
````

<a id="w003378"></a>

## W003378 — HPET

Bron-ID: W003378
Namespace: Main (0)
Revisie: 30935 | Datum: 2026-09-01T14:51:09Z
Bron: https://wiki.osdev.org/HPET
Revisie-URL: https://wiki.osdev.org/index.php?title=HPET&oldid=30935

````text
:''This page is not meant as a full description of HPET, only as a lightweight introduction. If you need any information not covered by this article, consult the [http://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/software-developers-hpet-spec-1-0a.pdf HPET specification].''

'''HPET''', or High Precision Event Timer, is a piece of hardware designed by Intel and Microsoft to replace older [[PIT]] and [[RTC]]. It consists of a (usually 64-bit) main counter (which counts up), as well as from 3 to 32 32-bit or 64-bit wide comparators. The HPET is programmed using memory mapped IO, and the base address of HPET can be found using [[ACPI]].

==Detecting HPET using ACPI==

The [http://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/software-developers-hpet-spec-1-0a.pdf HPET specification] defines an ACPI 2.0 table that is to be used to detect presence, address and capabilities of HPET present in the system. If this table doesn't exist, you should assume there is no HPET and you should fall back to [[PIT]] or the [[APIC timer]].

<source lang="cpp">
struct address_structure
{
    uint8_t address_space_id;    // 0 - system memory, 1 - system I/O
    uint8_t register_bit_width;
    uint8_t register_bit_offset;
    uint8_t reserved;
    uint64_t address;
} __attribute__((packed));

struct description_table_header
{
    char signature[4];    // 'HPET' in case of HPET table
    uint32_t length;
    uint8_t revision;
    uint8_t checksum;
    char oemid[6];
    uint64_t oem_tableid;
    uint32_t oem_revision;
    uint32_t creator_id;
    uint32_t creator_revision;
} __attribute__((packed));

struct hpet : public description_table_header
{
    uint8_t hardware_rev_id;
    uint8_t comparator_count:5;
    uint8_t counter_size:1;
    uint8_t reserved:1;
    uint8_t legacy_replacement:1;
    uint16_t pci_vendor_id;
    address_structure address;
    uint8_t hpet_number;
    uint16_t minimum_tick;
    uint8_t page_protection;
} __attribute__((packed));
</source>

==HPET - timer vs comparators==

There is just one up counting main counter in the timer, but interrupt generation is handled by *comparators*. There're from 3 to 32 comparators, and the exact amount is indicated by <code>comparator_count</code> field in the above <code>hpet</code> structure. Keep in mind you have to initialize both the main counter and all of the comparators. Also, the routing as well as '''allowed''' routing of comparator interrupts is independent, so you have to detect and set it up for each of them individually. More information on this procedure is provided further in the text.

==HPET operating modes==

HPET offers two operating modes: one-shot (also called "non-periodic" by the specification) and periodic mode.

===One-shot mode===

In non-periodic mode, the OS programs one of timer's comparator registers with value of main counter that is to trigger an interrupt. If the timer is set to 32 bit mode, it will also generate an interrupt when the counter wraps around. The comparator register's value is never written by the hardware, you are free to write to it and read from it at any time, therefore you can change at what value in the main counter the interrupt will be generated.

Every comparator in HPET '''must support''' non-periodic mode.

===Periodic mode===

Periodic mode is more tricky than non-periodic mode. For periodic mode, similarly to one-shot mode, you write a value at which an interrupt shall be generated to the comparator register. When the interrupt is generated, however, the hardware will '''increase the value in comparator register by the last value written to it'''! This is a consequence of HPET's main counter being up-counting.

So, if the main counter's value is 12345 when we set the timer up, and we write 12456 to comparator (i.e. the interrupt should trigger 111 time units from now), when the interrupt triggers, 12456 will be ''added'' to the comparator register, so it will become 24912, which is 12456 time units from the first interrupt. There are two techniques to deal with this problem; they will both be described in later part of the article.

Comparators are '''NOT required''' to support this mode; you must detect this capability when initializing a comparator. More information on this is provided further in the article.

==Interrupt routing==

HPET supports three interrupt mapping options: "legacy replacement" option, standard option, and FSB option. 

==="Legacy replacement" mapping===

In this mapping, HPET's timer (comparator) #0 replaces PIT interrupts, whereas timer #1 replaces RTC's interrupts (in other words, PIC and RTC will no longer cause interrupts). While the HPET specification provides the following table describing the routing of timers #0 and #1 in this mapping, it is recommended to at least check the routing of IRQ0 and IRQ8 to I/O APIC in ACPI tables.

{| {{wikitable}}
!Timer
!PIC mapping
!IOAPIC mapping
|-
|0
|IRQ0
|IRQ2
|-
|1
|IRQ8
|IRQ8
|-
|N
|As per IRQ Routing Field
|As per IRQ Routing Field
|}

===Standard mapping===

In standard mapping, each timer has its own interrupt routing control. Allowed I/O APIC inputs are found by reading timer's capabilities register.

===FSB mapping===

This mapping is almost identical to PCI's Message Signaled Interrupts. The "Timer N FSB Interrupt Route Register" which defines how FSB interrupts are configured can be found in the specification. FSB interrupts are enabled using "Tn_FSB_EN_CNF" field in timer's configuration register. This mapping mode will not be further discussed in this article.

==HPET registers==

The following table and field descriptions can also be found in the specification. "Offset" means offset from the address defined in <code>address</code> field of <code>hpet</code> struct. The following table skips reserved registers defined in the specification.

{| {{wikitable}}
!Offset
!Register
!Type
|-
|0x000 - 0x007
|General Capabilities and ID Register
|Read only
|-
|0x010 - 0x017
|General Configuration Register
|Read/write
|-
|0x020 - 0x027
|General Interrupt Status Register
|Read/write clear
|-
|0x0F0 - 0x0F7
|Main Counter Value Register
|Read/write
|-
|(0x100 + 0x20 * N) - (0x107 + 0x20 * N)
|Timer N Configuration and Capability Register
|Read/write
|-
|(0x108 + 0x20 * N) - (0x10F + 0x20 * N)
|Timer N Comparator Value Register
|Read/write
|-
|(0x110 + 0x20 * N) - (0x117 + 0x20 * N)
|Timer N FSB Interrupt Route Register
|Read/write
|}

===General Capabilities and ID Register===

{| {{wikitable}}
!Bits
!Name
!Description
|-
|63:32
|COUNTER_CLK_PERIOD
|Main counter tick period in femtoseconds (10^-15 seconds). Must not be zero, must be less or equal to 0x05F5E100, or 100 nanoseconds.
|-
|31-16
|VENDOR_ID
|This field should be interpreted similarly to PCI's vendor ID.
|-
|15
|LEG_RT_CAP
|If this bit is 1, HPET is capable of using "legacy replacement" mapping.
|-
|14
|Reserved
| -
|-
|13
|COUNT_SIZE_CAP
|If this bit is 1, HPET main counter is capable of operating in 64 bit mode.
|-
|12:8
|NUM_TIM_CAP
|The amount of timers - 1.
|-
|7:0
|REV_ID
|Indicates which revision of the function is implemented; must not be 0.
|}

===General Configuration Register===

{| {{wikitable}}
!Bits
!Name
!Description
|-
|63:2
|Reserved
| -
|-
|1
|LEG_RT_CNF
|0 - "legacy replacement" mapping is disabled
1 - "legacy replacement" mapping is enabled
|-
|0
|ENABLE_CNF
|Overall enable.
0 - main counter is halted, timer interrupts are disabled

1 - main counter is running, timer interrupts are allowed if enabled
|}

===General Interrupt Status Register===

{| {{wikitable}}
!Bits
!Name
!Description
|-
|63:32
|Reserved
| -
|-
|n
|Tn_INT_STS
|The functionality is dependent of whether edge or level-triggered mode is used for timer #n.

'''For level-triggered:''' the default value is 0. When a corresponding timer interrupt is active, this bit is set. If it is set, software can clear it by writing 1 to this bit. Writes of 0 have no effect.

'''For edge-triggered:''' this bit should be ignored. It is always set to 0.
|}

===Main Counter Value Register===

Bits 63:0 of this register are called MAIN_COUNTER_VAL. Writes to this register should only be done when the counter is halted (ENABLE_CNF = 0). Reads will return current value of the main counter. 32-bit counters will always return 0 for the upper 32 bits. If 32 bit reads are performed on 64 bit counter, consult 2.4.7 in the specification for instructions how to do it safely. It is recommended to use 32 bit counter when on 32-bit only software.

===Timer N Configuration and Capability Register===

{| {{wikitable}}
!Bits
!Name
!Description
|-
|63:32
|Tn_INT_ROUTE_CAP
|Timer n Interrupt Routing Capability. If bit X is set in this field, it means that this timer can be mapped to IRQX line of I/O APIC.
|-
|31:16
|Reserved
| -
|-
|15
|Tn_FSB_INT_DEL_CAP
|If this read-only bit is 1, this timer supports FSB interrupt mapping.
|-
|14
|Tn_FSB_EN_CNF
|If this bit is set to 1, this timer will use FSB interrupt mapping.
|-
|13:9
|Tn_INT_ROUTE_CNF
|This field indicates I/O APIC routing. Allowed values can be determined using Tn_INT_ROUTE_CAP. If an illegal value is written, then value read back from this field will not match the written value.
|-
|8
|Tn_32MODE_CNF
|For 64-bit timer, if this field is set, the timer will be forced to work in 32-bit mode. Otherwise it has no effect.
|-
|7
|Reserved
| -
|-
|6
|Tn_VAL_SET_CNF
|This field is used to allow software to directly set periodic timer's accumulator. Detailed explanation is provided further in the article.
|-
|5
|Tn_SIZE_CAP
|If this read-only bit is set to 1, the size of the timer is 64-bit. Otherwise, it's 32-bit.
|-
|4
|Tn_PER_INT_CAP
|If this read-only bit is set to 1, this timer supports periodic mode.
|-
|3
|Tn_TYPE_CNF
|If Tn_PER_INT_CAP is 1, then writing 1 to this field enables periodic timer and writing 0 enables non-periodic mode. Otherwise, this bit will be ignored and reading it will always return 0.
|-
|2
|Tn_INT_ENB_CNF
|Setting this bit to 1 enables triggering of interrupts. Even if this bit is 0, this timer will still set Tn_INT_STS.
|-
|1
|Tn_INT_TYPE_CNF
|0 - this timer generates edge-triggered interrupts.
1 - this timer generates level-triggered interrupts. When the interrupt is generated, Tn_INT_STS is set. If another interrupt occurs before that bit is cleared, the interrupt will remain active.
|-
|0
|Reserved
| -
|}

===Timer N Comparator Value Register===

Bits 63:0 (or 31:0, if the timer operates in 32 bit mode) are used to compare with main counter to check if an interrupt should be generated.

==Initialization==

The following is the procedure you need to perform to initialize main counter and comparators in order to receive interrupts.

General initialization:

1. Find HPET base address in 'HPET' ACPI table.

2. Calculate HPET frequency (f = 10^15 / period).

3. Save minimal tick (either from ACPI table or configuration register).

4. Initialize comparators.

5. Set ENABLE_CNF bit.

Timer N initialization:

1. Determine if timer N is periodic capable, save that information to avoid re-reading it every time.

2. Determine allowed interrupt routing for current timer and allocate an interrupt for it.

I am enabling the timers only when I actually use them, so there's no "real" initialization of comparators here.

Keep in mind that allowed interrupt routing '''may be insane'''. Namely, you probably want to use some of ISA interrupts - or, at very least, be able to use them at one point unambiguously. Last time I checked VirtualBox allowed mappings for HPET, it allowed every timer to be routed to ''any'' of 32 I/O APIC inputs present on the system. Knowing how buggy hardware can be, I wouldn't be too surprised if there exists a PC with HPET claiming that input #31 is allowed, when there are only 24 I/O APIC inputs. Be aware of this when choosing interrupt routing for timers.

==Using timers==

===One-shot mode===

To enable one-shot mode:

<source lang="cpp">
// "time" is time in femtoseconds from now to interrupt
if (time < COUNTER_CLK_PERIOD)
{
    time = adjust_time(time);
}

write_register_64(timer_configuration(n), (ioapic_input << 9) | (1 << 2));
write_register_64(timer_comparator(n), read_register(main_counter) + time);
</source>

I hope the above code is obvious. If it's not, please analyze the meaning of specific fields in registers used above.

===Periodic mode===

To enable periodic mode:

<source lang="cpp">
// "time" is time in femtoseconds from now to interrupt
if (time < COUNTER_CLK_PERIOD)
{
    time = adjust_time(time);
}

write_register_64(timer_configuration(n), (ioapic_input << 9) | (1 << 2) | (1 << 3) | (1 << 6));
write_register_64(timer_comparator(n), read_register(main_counter) + time);
write_register_64(timer_comparator(n), time);
</source>

This snippet requires some more comments.

Bit 2 is the same as above, Interrupt Enable. Bit 3 is also quite straightforward - 1 means periodic timer. But we've also set bit 6. Why?

Let's take a look at quote from the HPET specification:

<blockquote>Timer n Value Set: [...] Software uses this read/write bit only for timers that have been set to periodic mode. By writing this bit to a 1, the software is then allowed to directly set a periodic timer’s accumulator.</blockquote>
<blockquote>Software does NOT have to write this bit back to 0 (it automatically clears). Software should not write a 1 to this bit position if the timer is set to non-periodic mode.</blockquote>

This means that next write to timer N comparator register will have the usual meaning, while ''second'' next write will write directly to the accumulator. I believe that the wording could've been much better.

==See also==
* [[IOAPIC]]
* [[APIC]]
* [[APIC timer]]
* [[PIT]]
* [[RTC]]

==External links==
* [http://en.wikipedia.org/wiki/High_Precision_Event_Timer HPET article on Wikipedia]
* [http://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/software-developers-hpet-spec-1-0a.pdf Intel's HPET Specification v1.0a]


[[Category:Common Devices]]
[[Category:Interrupts]]
[[Category:Timers]]
````

<a id="w003379"></a>

## W003379 — Talk:Descriptor

Bron-ID: W003379
Namespace: Talk (1)
Revisie: 14915 | Datum: 2013-07-08T17:40:07Z
Bron: https://wiki.osdev.org/Talk:Descriptor
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ADescriptor&oldid=14915

````text
In reference to the [Paging] references made on the page:
AFAIK (Intel Documentation) there is no relation to Paging in desctiptors.

The P (Present) bit for selectors when 0 can generate a #NP (Segment Not Present),
whereas the P (Present) bit in PDE/PTE (page directory/table entries), when 0,
can generate a #PF (Page Fault).
[[User:Forge|Forge]] 17:06, 5 July 2013 (CDT)
: Thanks.  Fixed.  [[User:Jnc100|John]] 05:58, 6 July 2013 (CDT)

== Renaming or splitting article ==

The term descriptor is kinda ill suited as the name of this article. The term is easily confused with file descriptors and surely a better x86-specific name can be given to this article. --[[User:Sortie|Sortie]] 18:05, 5 July 2013 (CDT)
: I vote for merging into the appropriate articles ([[GDT]] and [[IDT]]) and then remove this.  A quick browse of the [[GDT]] article reveals that it only deals with code/data descriptors (not TSS etc) which we could expand with information from this article.  Unfortunately, most of the information on the GDT article is in vector graphics format, so we presumably need the original author to step forward with the pictures to edit? [[User:Jnc100|John]] 05:54, 6 July 2013 (CDT)
: I vote for removal of the [[GDT]] and the [[IDT]] articles as well. In place, we should have a [[Segmentation]] and [[Interrupts]] page, with a x86 section which deals with GDT/IDT. Since GDT and IDT are often the first thing one does when writing a hobby kernel for x86, it has been given a lot of special attention. I don't see us having articles about "Page Directory" or "Page Table" -- it all comes under virtual memory/paging. --[[User:Shikhin|Shikhin]] 04:56, 8 July 2013 (CDT)
: Note that it would be good to keep the x86-specific names for this information (GDT, IDT) because they contain x86-specific information, unlike "Segmentation" and "Interrupts" that by nature should have generic information about multiple architectures. Likewise, "Page Table" and "Page Directory" would be bad names for articles, because then we would have articles on "Page Directory Table" and "Page Map Level 4" as well, when it really should be part of "Paging (x86)". --[[User:Sortie|Sortie]] 05:49, 8 July 2013 (CDT)
:: Hmm, maybe. I'm just suggesting that as a community, we place a bit too much emphasis on x86. OSDev on ARM actually has quite some benefits, and I find it equally fun and interesting. I was thinking that maybe we have all ARCH-specific topics under one page, rather than spread them out. Then, again, that's not been done yet, so maybe there is some valid reason? How about a quick vote here? (oh, and as per your analogy, we can just as well have GDT under Segmentation (x86) -- mostly because I suppose it's only on x86 -- to have a neat hierarchy) :-) --[[User:Shikhin|Shikhin]] 09:53, 8 July 2013 (CDT)
::: No need to vote, just make the restructuring changes yourself if you have the time and you think it is an improvement. If anyone strongly disagrees, they can make a forum topic or another talk page discussion. --[[User:Sortie|Sortie]] 12:40, 8 July 2013 (CDT)
````

<a id="w003380"></a>

## W003380 — IDE

Bron-ID: W003380
Namespace: Main (0)
Revisie: 14930 | Datum: 2013-07-12T15:14:16Z
Bron: https://wiki.osdev.org/IDE
Revisie-URL: https://wiki.osdev.org/index.php?title=IDE&oldid=14930
Doorverwijzing: PCI IDE Controller

````text
#REDIRECT [[PCI IDE Controller]]
````

<a id="w003381"></a>

## W003381 — Talk:IDE

Bron-ID: W003381
Namespace: Talk (1)
Revisie: 14932 | Datum: 2013-07-12T15:14:16Z
Bron: https://wiki.osdev.org/Talk:IDE
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AIDE&oldid=14932
Doorverwijzing: Talk:PCI IDE Controller

````text
#REDIRECT [[Talk:PCI IDE Controller]]
````

<a id="w003382"></a>

## W003382 — User:Nessphoro

Bron-ID: W003382
Namespace: User (2)
Revisie: 14954 | Datum: 2013-07-24T06:04:56Z
Bron: https://wiki.osdev.org/User:Nessphoro
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANessphoro&oldid=14954

````text

== On Memory Allocators ==
''Disclaimer: This is a work in progress.(This is essentially documenting the heap allocator in my OS)''

''See Also: [[Memory Allocation]] ''

=== Overview ===
In a nutshell, the memory allocator described here is a simplification of jemalloc and posted for reasons of creating diversity.
The language I will use is C++, however it will look more like C with classes and namespaces. Also, the naming standard will be the same Microsoft uses in C# API's due to its appealing neatness.

=== Quick Look at jemalloc ===
jemalloc is described in a paper by Jason Evans and is freely available on the Internet. Its purpose is to provide a concurrent and scalable malloc for general use. The algorithm achieves that by heavily utilizing TLS (Thread Local Storage) so that threads need not use locks to allocate small to mid sized object. If, however, TLS does not have enough caches paged, the algorithm simply asks the global page manager to allocate more space and maps it to the TLS. The global page manager is also responsible to allocate space for large and huge objects which are mapped at page boundaries. Please note, that jemalloc does not use a 4 byte header before each pointer to store size, instead it uses spans which are described further on. Spans are ''spans'' of objects with a fixed size. The size is predetermined and simplifies the algorithm in several ways. By utilizing fixed sized pooled object, the algorithm seeks to minimize fragmentation which leads to better performance and stability in long running systems. Each span in 2MiB long with all metadata stored at the beginning. Within the span, bitmaps are utilized to store information about allocated/free objects. Size's for most small objects is determined by the equation 16n where n is a number from 1 to 32.

----
Note from the author:

Bitmaps are actually quite efficient as inline structures. Their worst case is equivalent to linked list with O(n). Nevertheless, caution must be exercised especially when implement bit-checking. It is often a good idea to avoid such consturcts: 

<source lang="cpp">
for(int i=0;i<32;i++)
{
    if(intToTest>>i && 1)
        DoSomething();
}
</source>
Instead, implementation of a fast-path in assembly is advised, particularly due to x86 instructions such as BSF which allow the execution of the above code in one operation.
----


As for large objects, the algorithm uses red-black self balancing binary trees to store the metadata which allows very efficient look up in O(log n). As a possible optimization, it might be sound to use hash-tables to achieve constant time speed, this is however out-of-scope of this discussion.

=== Quick Overview of Hydra's Heap Allocator ===
Hydra being an OS in its early life naturally doesn't have many features compared to FreeBSD (the platform jemalloc originally targeted) hence I decided to not use things such a TLS as the kernel does not perform many allocations in concurrency and is not expected to compete with user-space programs due to the separation of allocators and space. In addition, it is expected that kernel drivers needing a fast/concurrent malloc will implement it themselves - for now at least. In conclusion, this approach has many perks:

1. If you allocate many objects but don't actually touch them - no physical memory is consumed.

2. No extra metedata is needed, which increases memory utilization.

3. Guaranteed memory alignment on the object's size boundary.

4. Subjectively quick allocations.

5. Reduced fragmentation.
````

<a id="w003386"></a>

## W003386 — Template:OSProject

Bron-ID: W003386
Namespace: Template (10)
Revisie: 28881 | Datum: 2024-05-28T08:16:07Z
Bron: https://wiki.osdev.org/Template:OSProject
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AOSProject&oldid=28881

````text
<div style="margin: 0 0 10px 0; background-color: #FAFAFA; color: #000000; border: 1px solid #CEDFF2; padding:0.5em 0.5em;">
{| style="width: 100%; margin: 0; padding: 0; border: 0; border-collapse: collapse;"
| style="padding: 0; width: 70%; vertical-align: top;" |
<div style="float:right;padding-left:4px;padding-right:4px;">{{if|{{{Pic|}}}|{{{Pic|}}} }}</div>
<h2 style="margin:0;font-size:120%;border:none;font-weight:bold;text-align:left;color:#000;padding:0.0em 0.0em;">{{{OSName}}}</h2>
{{{OSDescription|''No description provided.''}}}
| style="padding: 0 0 0 0px; width: 30%; vertical-align: top; font-size: 11px;" |
[[Image:Icon_phone.png]] ''{{{Contact|No Contact Information}}}''<br>

[[Image:Icon_license.png]] ''{{{License|No License Information}}}''<br>

[[Image:Icon_site.png]] ''{{if|{{{URL|}}}|[{{{URL}}} {{{URL}}}]|No URL}}'' {{{URLEx|}}}<br>

[[Image:Icon_floppy.png]] ''{{{BootMedia|Unknown}}}''<br>

[[Image:Icon_clock.png]] ''{{Month|{{{LastReleaseMonth}}}}} {{if|{{{LastReleaseDay|{{{LastReleaseMonth|}}}}}}|{{{LastReleaseDay}}},&nbsp;}}{{{LastReleaseYear|}}}'' {{if|{{{LastReleaseDescription|}}}|-- ''{{{LastReleaseDescription}}}''}} {{if|{{{Status|}}}| ''{{{Status|}}}''}}

{{if|{{{CloserInspection|}}}|<br>{{{CloserInspection|}}}}}
|}
</div>
````

<a id="w003387"></a>

## W003387 — Template:CloserInspectionValidityNew

Bron-ID: W003387
Namespace: Template (10)
Revisie: 14990 | Datum: 2013-08-03T22:05:57Z
Bron: https://wiki.osdev.org/Template:CloserInspectionValidityNew
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ACloserInspectionValidityNew&oldid=14990

````text
[[Image:Icon_magnify_warning.png]] '''The validity of this entry is questionable;'''<br \> ''{{{1}}}''
````

<a id="w003388"></a>

## W003388 — File:Toaruos screenshot.png

Bron-ID: W003388
Namespace: File (6)
Revisie: 26667 | Datum: 2021-11-09T10:16:28Z
Bron: https://wiki.osdev.org/File:Toaruos_screenshot.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AToaruos_screenshot.png&oldid=26667

````text

````

<a id="w003389"></a>

## W003389 — User:Sortie/OS Specific Toolchain

Bron-ID: W003389
Namespace: User (2)
Revisie: 16781 | Datum: 2014-09-21T14:31:33Z
Bron: https://wiki.osdev.org/User:Sortie/OS_Specific_Toolchain
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASortie%2FOS_Specific_Toolchain&oldid=16781
Doorverwijzing: OS Specific Toolchain

````text
#REDIRECT [[OS Specific Toolchain]]
````

<a id="w003390"></a>

## W003390 — File:Drama.png

Bron-ID: W003390
Namespace: File (6)
Revisie: 15028 | Datum: 2013-08-09T05:54:14Z
Bron: https://wiki.osdev.org/File:Drama.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ADrama.png&oldid=15028

````text

````

<a id="w003391"></a>

## W003391 — Template:Humor

Bron-ID: W003391
Namespace: Template (10)
Revisie: 28993 | Datum: 2024-06-06T17:27:27Z
Bron: https://wiki.osdev.org/Template:Humor
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AHumor&oldid=28993

````text
<center>
{|style="border: 1px solid #cfcfbf; margin-top: 25px; margin-bottom: 25px; background-color: #f0f0ff; text-align: center;"
|
[[image:Drama.png|Comedy]]
This page is a work of humor, it is not meant to be serious.
|
|}
</center>

<includeonly>[[Category:Humor]]</includeonly>
````

<a id="w003392"></a>

## W003392 — You Know Your OS Is Advanced When

Bron-ID: W003392
Namespace: Main (0)
Revisie: 29644 | Datum: 2025-09-09T13:26:09Z
Bron: https://wiki.osdev.org/You_Know_Your_OS_Is_Advanced_When
Revisie-URL: https://wiki.osdev.org/index.php?title=You_Know_Your_OS_Is_Advanced_When&oldid=29644

````text
{{Humor}}

<span style="font-size: 16px;">"Hmm... I don't think I have mmap... oh wait, I changed the binary loader to use it months ago!"</span>
- thePowersGang

<span style="font-size: 16px;">... you have a problem with bash and find the answer on Stack Overflow with an Ubuntu tag.</span>
- klange

<span style="font-size: 16px;">... people don't believe it when you show them screenshots with `uname -a` output.</span>
- pcmattman

<span style="font-size: 16px;">... your compositor has features that can't be implemented with X11.</span>
- klange

<span style="font-size: 16px;">... you consult the Linux From Scratch guide more often than the Intel/AMD documentation.</span>
- sortie

<span style="font-size: 16px;">... you require at least 1 GB of main memory just to run it.</span>
- glauxosdever

<span style="font-size: 16px;">... windows decides to wipe your hard drive 35 times.</span>
- bellezzasolo

<span style="font-size: 16px;">... you require a bunch of 3rd-party drivers for the system to even boot.</span>
- Coconut9

<span style="font-size: 16px;">... your Desktop is full of PDFs for weird devices.</span>
- superleaf1995

<span style="font-size: 16px;">... you ask questions on Stack Overflow with the Linux tag.</span>
- Moldytzu

<span style="font-size: 16px;">... when it takes more than 5 seconds to compile</span>
- Anotheridiot
````

<a id="w003393"></a>

## W003393 — Template:Infobox Project

Bron-ID: W003393
Namespace: Template (10)
Revisie: 15085 | Datum: 2013-08-10T06:19:03Z
Bron: https://wiki.osdev.org/Template:Infobox_Project
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AInfobox_Project&oldid=15085

````text
<table align="right" cellpadding="4" cellspacing="0" style="margin-top:0.5em; margin-bottom:0.5em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; width: 30em; font-size: 90%; text-align: left;">
<tr><td colspan="2" style="text-align: center; font-size: 130%; font-weight: bold;">
{{{name}}}
</td></tr>
{{if | {{{image|}}} | <tr><td colspan{{=}}2><center>[[Image:{{{image}}}|300px]]<br><span style{{=}}"font-size: 10px;">''{{{caption|}}} ''</span></center></td></tr>}}
{{if | {{{developers|}}} | <tr><td>'''Developed by:'''</td><td>{{{developers}}}</td></tr>}}
{{if | {{{kerneltype|}}} | <tr><td>'''Kernel:'''</td><td>{{{kerneltype}}}</td></tr>}}
{{if | {{{taskmodel|}}} | <tr><td>'''Task Model:'''</td><td>{{{taskmodel}}}</td></tr>}}
{{if | {{{platform|}}} | <tr><td>'''Platform(s):'''</td><td>{{{platform}}}</td></tr>}}
{{if | {{{language|}}} | <tr><td>'''Language(s):'''</td><td>{{{language}}}</td></tr>}}
{{if | {{{website|}}} | <tr><td>'''Website:'''</td><td>{{{website}}}</td></tr>}}
{{if | {{{github|}}} | <tr><td>'''Github:'''</td><td>[https://github.com/{{{github}}} {{{github}}}]</td></tr>}}
{{if | {{{license|}}} | <tr><td>'''License:'''</td><td>{{{license}}}</td></tr>}}
</table>
````

<a id="w003394"></a>

## W003394 — User:Klange/ToaruOS

Bron-ID: W003394
Namespace: User (2)
Revisie: 26673 | Datum: 2021-11-13T00:02:40Z
Bron: https://wiki.osdev.org/User:Klange/ToaruOS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKlange%2FToaruOS&oldid=26673

````text
{{Infobox Project
|name=ToaruOS
|image=Toaruos_screenshot.png
|caption=ToaruOS desktop with Yutani compositor
|developers=[[User:Klange|K. Lange]]
|kerneltype=Hybrid
|platform=x86-64
|taskmodel=Preemptive multitasking
|language=C, [https://kuroko-lang.github.io/ Kuroko]
|website=[http://toaruos.org toaruos.org]
|github=klange/toaruos
|license=NCSA
}}

'''ToaruOS''' is an SMP-capable, graphical operating system with a 64-bit hybrid kernel, written by [[User:Klange|K. Lange]].

== History ==

ToaruOS began as a university project in January 2011. It was briefly managed by the [https://en.wikipedia.org/wiki/University_of_Illinois_Urbana-Champaign University of Illinois] [https://en.wikipedia.org/wiki/Association_for_Computing_Machinery ACM] [https://en.wikipedia.org/wiki/ACM_SIGOPS SIGOps] student chapter until 2012, where its original GUI was developed.

A stable release of ToaruOS was made available in 2015. In 2018, the project underwent a major rewrite of its userspace, eliminating third-party components. In 2021, a new 64-bit kernel was written to replace the original 32-bit kernel.

ToaruOS 2.0 is slated to be released in December of 2021.


== Kernel ==

Previously, ToaruOS employed a 32-bit, modular kernel which provided multitasking, VFS, various IPC mechanisms, and shared memory between processes.

In 2021, a new kernel, ''Misaka'', was written, targeting x86-64 and supporting SMP.

== Kuroko ==

ToaruOS includes an implementation of a dialect of Python called ''Kuroko''.

== Yutani ==

ToaruOS's GUI is powered by '''Yutani''', a 2D compositor and window server. Yutani uses shared memory to store window textures and bidirectional pipes to communicate commands events. Yutani provides a number of features including an exposé mode with live previews, alpha window, window rotation, and animations.
````

<a id="w003395"></a>

## W003395 — Template:=

Bron-ID: W003395
Namespace: Template (10)
Revisie: 15064 | Datum: 2013-08-10T05:54:42Z
Bron: https://wiki.osdev.org/Template:%3D
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3A%3D&oldid=15064

````text
=
````

<a id="w003396"></a>

## W003396 — User:Dozniak

Bron-ID: W003396
Namespace: User (2)
Revisie: 15089 | Datum: 2013-08-10T09:12:21Z
Bron: https://wiki.osdev.org/User:Dozniak
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADozniak&oldid=15089

````text
Omnia mea mecum porto. See [[User:Dozniak/Metta|Metta]].
````

<a id="w003397"></a>

## W003397 — User:Dozniak/Metta

Bron-ID: W003397
Namespace: User (2)
Revisie: 15091 | Datum: 2013-08-10T09:24:50Z
Bron: https://wiki.osdev.org/User:Dozniak/Metta
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADozniak%2FMetta&oldid=15091

````text
{{Infobox Project
|name=Metta OS

|image=Metta_bootup.png
|caption=Boot up of type system

|developers=[[User:Dozniak|Berkus]]

|kerneltype=Exokernel
|platform=x86,x86-64,arm
|taskmodel=Preemptive multitasking
|language=C++

|website=[http://exocortex.madfire.net exocortex]
|github=berkus/metta

|license=BOOST
}}

'''Metta OS''' is a multi-tasking operating system with a 32-bit modular x86 exokernel and vertically integrated userspace. The project is maintained by single developer with occasional contributions from third parties.

== Kernel ==

Metta employs a 32-bit kernel which only provides MMU hardware access, interrupt processing stubs and a single synchronization primitive. The rest of the kernel runs in userspace as a privileged domain.

The vertical integration builds upon interfaces and components, composed together using resolution and binding services. This allows to securely colocate speed-critical components without any extra context switching. Synchronization primitives all build upon '''event''' primitive supported by the kernel.
````

<a id="w003398"></a>

## W003398 — File:Metta bootup.png

Bron-ID: W003398
Namespace: File (6)
Revisie: 15092 | Datum: 2013-08-10T09:30:06Z
Bron: https://wiki.osdev.org/File:Metta_bootup.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AMetta_bootup.png&oldid=15092

````text

````

<a id="w003399"></a>

## W003399 — Accelerated Graphics Cards

Bron-ID: W003399
Namespace: Main (0)
Revisie: 15098 | Datum: 2013-08-12T08:56:15Z
Bron: https://wiki.osdev.org/Accelerated_Graphics_Cards
Revisie-URL: https://wiki.osdev.org/index.php?title=Accelerated_Graphics_Cards&oldid=15098
Doorverwijzing: Accelerated Graphic Cards

````text
#REDIRECT [[Accelerated Graphic Cards]]
````

<a id="w003400"></a>

## W003400 — Talk:APIC Timer

Bron-ID: W003400
Namespace: Talk (1)
Revisie: 28148 | Datum: 2023-07-10T14:11:19Z
Bron: https://wiki.osdev.org/Talk:APIC_Timer
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AAPIC_Timer&oldid=28148

````text
In Bochs 2.4 there is a bug related to APIC causing code on this page not to work (APIC timer fires only one time and then stops). But in Debian (and especially Ubuntu) only Bochs version in repo is 2.4.6 (but Debian experimental has bochs-2.6). Should it be noted on page? I think, yes.
````

<a id="w003401"></a>

## W003401 — File:ISBN 0596009658.jpg

Bron-ID: W003401
Namespace: File (6)
Revisie: 15108 | Datum: 2013-08-19T23:54:48Z
Bron: https://wiki.osdev.org/File:ISBN_0596009658.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0596009658.jpg&oldid=15108

````text
Learning the bash Shell, 3rd ed.
````

<a id="w003402"></a>

## W003402 — Portability

Bron-ID: W003402
Namespace: Main (0)
Revisie: 29011 | Datum: 2024-06-06T17:31:33Z
Bron: https://wiki.osdev.org/Portability
Revisie-URL: https://wiki.osdev.org/index.php?title=Portability&oldid=29011

````text

== Theory ==
While you might be content to write a kernel that functions on only x86 IBM compatible computers, the truth is there are many other platforms on which to write a kernel. These range from the long dead, such as Amiga and Alpha Processor-based computers, to the relevant ARM architectures. Traditionally, the operating system was specific to an architecture, which allowed user programs to take full advantage of their hardware designs. UNIX however, changed this with the introduction of the C programming language, which allowed UNIX (And other programs) to be ported to almost every system ever conceived with only minor adjustments. An operating system's core, the kernel is designed to deal with the hardware, so the Operating System can be oblivious to hardware changes. However, most kernel code can be reused on different architectures that share similar hardware, such as USB, and PCI buses.

== Practical ==
=== Prep Work ===
C and C++ code can be crossed compiled onto many systems, so chances are if you can target gcc to run on it, you can write a kernel for it. Some hardware will be easier and more thoroughly documented than others, but in any case, you should fully understand the system in which you're developing for, and read all it's manuals and specifications. Once you feel confident with it, you can begin.

=== Separate Sources ===
Your kernel will have functions that contain what is called '''generic''' and  '''targeted''' code. For example: a puts() implementation will be generic, while the screen driver that puts() uses to draw the string to the output device would be targeted to a specific implementation. Before you begin trying to code for more than one platform, separate your sources so any targeted functions are in a different location than generics. This creates an abstraction layer between hardware drivers and pure software. That way, all you need to do to port your kernel to a new platform is re-write the targeted functions for the platform, and the generic code can run on top of it. 

Targeted code can be further divided into two categories: architecture-specific code and machine-specific code. Architecture-specific code is code that differs between different processor architectures, like I/O, trap dispatching, and paging. Machine-specific code is code that differs between machines with the same processor architectures, like IRQ controllers, system timers, real time clocks, and multiprocessor information. The NT kernel has a module containing machine-specific code called the Hardware Abstraction Layer, or HAL.

=== Code ===
After the targeted files are separated, you can make new targeted functions for each platform you want to port to. The generic code will (usually) run without issue on top of the targeted code. A brief side note, make sure to delete the object files after you re-target, as object files are not cross compatible.

=== Next Steps ===
Once your architecture-specific code has been separated, you need to begin writing drivers for the new platform. Even though you've modified your kernel to be able to run on top of any hardware, you still need to write the drivers that have code specific to your platform.  For example, if you had an x86-only kernel before, and you separated out the targeted code and wrote targeted code for the Raspi, in order for your OS to actually interface with the user, you have to write device drivers for hardware like the screen and audio controllers.

== Example Targeted Functions ==
* Booting (usually assembly stubs)
* CPU Specific functions (I/O, Trap Dispatching, Paging, Context Switching, SMP)
* Timers (PIT, RTC, APIC Timer, HPET)

== Example Generic Functions ==
* malloc() (NOT paging!)
* Task Scheduler
* String parsing functions
* Buffered I/O (gets, putchar, etc)

[[Category:Porting]]
[[Category:OS theory]]
[[Category:FAQ]]
````

<a id="w003403"></a>

## W003403 — User:Love4boobies/Cygwin as UNIX

Bron-ID: W003403
Namespace: User (2)
Revisie: 19905 | Datum: 2016-10-24T01:59:32Z
Bron: https://wiki.osdev.org/User:Love4boobies/Cygwin_as_UNIX
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALove4boobies%2FCygwin_as_UNIX&oldid=19905

````text
{{stub}}

''TODO: Go through libraries as well.''

In order to bring your Cygwin environment as close as possible to the UNIX System, you want to install at least the following packages, their dependencies, as well as the '''Base''' packages, and to set the <tt>POSIXLY_CORRECT</tt> environment variable. Note that some of these may be missing from the 64-bit distribution; you can find a full list [http://www.cygwin.com/cygwin-64bit-missing here]. Furthermore, not all of these packages are necessarily useful for the purpose of OS development.

* base-cygwin
* base-files
* bash (utilities: <tt>alias</tt><ref name=builtin>This is a Bash built-in command. You can find a link to the full list of the Bash built-in commands at the bottom of the page.</ref>, <tt>sh</tt>, <tt>cd</tt><ref name=builtin />, <tt>command</tt><ref name=builtin />, <tt>getopts</tt><ref name=builtin />, <tt>hash</tt><ref name=builtin />, <tt>read</tt><ref name=builtin />, <tt>type</tt><ref name=builtin />, <tt>ulimit</tt><ref name=builtin />, <tt>umask</tt><ref name=builtin />, <tt>unalias</tt><ref name=builtin />)
* bc (utilities: <tt>bc</tt>)
* binutils (linker; utilities: <tt>ar</tt>, <tt>nm</tt>, <tt>strings</tt>, <tt>strip</tt>)
* bison (utilities: <tt>yacc</tt>)
* catgets (utilities: <tt>gencat</tt>)
* coreutils (utilities: <tt>basename</tt>, <tt>cat</tt>, <tt>chgrp</tt>, <tt>chmod</tt>, <tt>chown</tt>, <tt>cksum</tt>, <tt>comm</tt>, <tt>cp</tt>, <tt>csplit</tt>, <tt>cut</tt>, <tt>date</tt>, <tt>dd</tt>, <tt>df</tt>, <tt>dirname</tt>, <tt>du</tt>, <tt>echo</tt>, <tt>env</tt>, <tt>expand</tt>, <tt>expr</tt>, <tt>false</tt>, <tt>fold</tt>, <tt>head</tt>, <tt>id</tt>, <tt>join</tt>, <tt>link</tt>, <tt>ln</tt>, <tt>logname</tt>, <tt>ls</tt>, <tt>mkdir</tt>, <tt>mkfifo</tt>, <tt>mv</tt>, <tt>nice</tt>, <tt>nl</tt>, <tt>nohup</tt>, <tt>od</tt>, <tt>paste</tt>, <tt>pathchk</tt>, <tt>pr</tt>, <tt>printf</tt>, <tt>pwd</tt>, <tt>rm</tt>, <tt>rmdir</tt>, <tt>sleep</tt>, <tt>sort</tt>, <tt>split</tt>, <tt>stty</tt>, <tt>tail</tt>, <tt>tee</tt>, <tt>test</tt>, <tt>touch</tt>, <tt>tr</tt>, <tt>true</tt>, <tt>tsort</tt>, <tt>tty</tt>, <tt>uname</tt>, <tt>unexpand</tt>, <tt>uniq</tt>, <tt>unlink</tt>, <tt>wc</tt>, <tt>who</tt>)
* cron (utilities: <tt>cron</tt>)
* ctags (utilities: <tt>ctags</tt>)
* cygutils (utilities: <tt>lp</tt>)
* cygwin (utilities: <tt>getconf</tt>, <tt>kill</tt>, <tt>locale</tt>, <tt>ps</tt>)
* diffutils (utilities: <tt>cmp</tt>, <tt>diff</tt>)
* ed (utilities: <tt>ed</tt>)
* file (utilities: <tt>file</tt>)
* findutils (utilities: <tt>find</tt>, <tt>xargs</tt>)
* flex (utilities: <tt>lex</tt>)
* gawk (utilities: <tt>awk</tt>)
* gcc-core (utilities: <tt>c99</tt>)
* gcc-fortran (utilities: <tt>fort77</tt>)
* grep (utilities: <tt>grep</tt>)
* gzip (utilities: <tt>zcat</tt>)
* inetutils (utilities: <tt>talk</tt>)
* ipc-utils (utilities: <tt>ipcrm</tt>, <tt>ipcs</tt>)
* libiconv (utilities: <tt>iconv</tt>)
* m4 (utilities: <tt>m4</tt>)
* man-db (utilities: <tt>man</tt>)
* make (utilities: <tt>make</tt>)
* mintty (terminal)
* ncompress (utilities: <tt>compress</tt>, <tt>uncompress</tt>)
* ncurses (utilities: <tt>tabs</tt>, <tt>tput</tt>)
* patch (utilities: <tt>patch</tt>)
* pax (utilities: <tt>pax</tt>)
* psmisc (utilities: <tt>fuser</tt>)
* sed (utilities: <tt>sed</tt>)
* sharutils (utilities: <tt>uudecode</tt>, <tt>uuencode</tt>)
* sysvinit (utilities: <tt>mesg</tt>)
* time (utilities: <tt>time</tt>)
* util-linux (utilities: <tt>cal</tt>, <tt>logger</tt>, <tt>more</tt>, <tt>renice</tt>)
* vim-minimal (utilities: <tt>ex</tt>, <tt>vi</tt>)

{{reflist}}

The following utilities cannot be found in any of the packages: <tt>admin</tt>, <tt>asa</tt>, <tt>at</tt>, <tt>batch</tt>, <tt>bg</tt>, <tt>cflow</tt>, <tt>cxref</tt>, <tt>delta</tt>, <tt>fc</tt>, <tt>fg</tt>, <tt>get</tt>, <tt>jobs</tt>, <tt>localedef</tt>, <tt>mailx</tt>, <tt>newgrp</tt>, <tt>prs</tt>, <tt>qalter</tt>, <tt>qdel</tt>, <tt>qhold</tt>, <tt>qmove</tt>, <tt>qmsg</tt>, <tt>qreturn</tt>, <tt>qrls</tt>, <tt>qselect</tt>, <tt>qsig</tt>, <tt>qstat</tt>, <tt>qsub</tt>, <tt>rmdel</tt>, <tt>sact</tt>, <tt>sccs</tt>, <tt>unget</tt>, <tt>uucp</tt>, <tt>uustat</tt>, <tt>uux</tt>, <tt>val</tt>, <tt>wait</tt>, <tt>what</tt>, and <tt>write</tt>.

Here are some possible starting points for these utilities:

* [http://sccs.berlios.de/ SCCS] (utilities: <tt>admin</tt>, <tt>delta</tt>, <tt>get</tt>, <tt>prs</tt>, <tt>rmdel</tt>, <tt>sact</tt>, <tt>sccs</tt>, <tt>unget</tt>, <tt>val</tt>, <tt>what</tt>)
* [ftp://sunsite.unc.edu/pub/Linux/devel/lang/fortran Asa] (utilities: <tt>asa</tt>)
* [http://www.gnu.org/software/cflow/ GNU cflow] (utilities: <tt>cflow</tt>)
* [http://www.gedanken.org.uk/software/cxref/ Cxref] (utilities: <tt>cxref</tt>)
* [http://mailutils.org/ GNU Mailutils] (utilities: <tt>mailx</tt>)
* [http://www.gnu.org/software/uucp/ GNU UUCP] (utilities: <tt>uucp</tt>, <tt>uux</tt>)

= See Also =

* [http://pubs.opengroup.org/onlinepubs/9699919799/idx/head.html POSIX headers]
* [http://pubs.opengroup.org/onlinepubs/9699919799/idx/utilities.html POSIX utilities]
* [http://cygwin.com/packages/ Cygwin packages]
* [http://www.gnu.org/software/bash/manual/html_node/Shell-Builtin-Commands.html Bash built-in commands]
````

<a id="w003404"></a>

## W003404 — User:H0bby1

Bron-ID: W003404
Namespace: User (2)
Revisie: 17233 | Datum: 2014-12-02T22:26:53Z
Bron: https://wiki.osdev.org/User:H0bby1
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AH0bby1&oldid=17233

````text
{{Lovecraftian}}
{{Disputed|Topic:27147}}



==how to prevent dependence on build environment in C==

When a program is compiled to produce executable machine code, there is some part of the building process that is not described in the C language, but are left to the build chain to generate the executable code for a particular architecture.

==what part of the build can be specific==


*'''calling convention and data types''': Calling convention are the way function are called, that specify how return address and parameters are passed to the function, as well as how the return value will be passed back to the caller. There are some calling convention that most compiler will recognize, but they are cpu dependent, and compilers might not use them by default in all cases, some compiler can allow for compiler specific calling conventions, using register instead of the stack to pass parameters for example. The way the compiler will deal with structure can also vary, regarding member alignment, and packing of members. Some compiler can also use specific data type declaration, for example for 64 bit integer, or have specific 16 bit char type for unicode strings.

*'''the C runtime''' : (crt, libgcc for linux), They are functions that are not part of the C standard, often made in assembler for the specific cpu target.The compiler uses them to perform an operation with an internal routine specific to the compiler and to the target architecture. It is also used to perform system specific application initialization before the 'main' function is entered and after the main function return to handle system specific process shutdown.

*'''the standard C library ''': (stdc, glibc for linux), The C Library is used to provide a set of standard functions that C program can use to access system specific resources. It is part of the build environment for the particular target, all C compilers who want to enable program to use the set of standard C functions must use an implementation of the C Library for the target they want to produce working executable for. This include the string and memory management functions (strcpy,memcpy,strtol,itoa) , memory allocation functions (malloc,calloc,realloc,free), floating point math and the ALU, i/o function (printf/sscanf/fopen/fread/fwrite/fclose) , as well as functions used to program sockets. The C Library is generally build on top of the kernel, and is used to provide a standard interface to make in sort C program are independent of any kernel specific functions.


Only the C runtime is generally linked statically because it's small enough, and those functions are totally specific to the compiler. The C library is generally linked dynamically, and has to be present and loaded by the target system at runtime. 

The C library can also be linked statically alongside with the executable, in which case the exe will depend only on lower level system api or kernel that the C Library used is programmed for.As the C Library is made for the purpose to avoid direct kernel dependencies from the executable, and to provide a standard API to C program, it's more meant to be linked dynamically, to make it possible for the executable to be run on different system that implement the set of standard C functions. If it's linked statically, the executable will then depend on any lower level functions that the C Library use internally.

*On windows before visual studio 2005, the C library of visual studio was shipped in windows by default, so you didn't have to worry about it, but since vc2005, applications have to distribute the C library (msvc.lib) with them, either as a dll or linked statically in the application if they want to use visual studio C library.

*On linux and bsd, application are generally compiled and linked on the host plateform before to be used, so it make sure all the functions definition are compiled according to the host configuration.

----

==different configurations==


====programming regular application====
If you are programming an application, what you want is to take advantage of a particular development environment , and to use a maximum of compiler specific features and rely entirely on the C building suit to compile optimized code for the platform targeted by the build. The application need to be linked statically with the compiler's C runtime, and to either be distributed with the C library used to to build it, or to rely on the C Library implementation present on the target system to be compatible with the definitions of the C Function from the C Library used to build it.

====programming a shared library====
If you are programming a library, or a framework, the 'guest' C program (the application using the library) will have to use shared data structures passed as function arguments with the framework, then you want to make sure the compiler use the specified calling convention to use the functions of the framework as well as identical structure packing if shared structure are involved.

Eventually you may want to avoid using Standard C functions to avoid the client application to have to include the C Library used to compile the framework as a dependence. 

If the framework has to be linked statically, the C runtime can create conflicts between the framework library and the C runtime of the application's compiler, because both the framework and the application need their respective compiler's version of the runtime to be linked statically in the executable.

====programming a kernel====

A kernel generally cannot use the standard C library of the compiler, but some of the functions can be linked statically or implemented fully in the kernel code, in which case it shouldn't be a problem because it's unlikely the kernel's C routines will be compiled with a different building environment than the kernel itself.

However if those kernel functions have to be used as 'functions of the standard C library' by other programs, need to make sure the declaration of the standard C functions contained in the standard C header (stddef.h, string.h, math.h) used to build the client program are consistent with how the functions are defined in the kernel.

If the client program's compiler need to use kernel specific declaration of the function of the C Library, need to make sure it treat the call to those functions according to the explicit declaration of the header used , compilers often expects a specific implementation of the standard C library functions, so need to make sure the compiler follow the explicit declaration of the functions present in the header files and not the implicit declaration that the compiler might expect. 

====programming a operating system====

Applications will have to use some data type and functions defined in the os source code, and to be able to use them in it's own source code. The calling convention attribute and parameters of how structure are aligned and packed, and how parameters and return address are expected to be passed to function of the os need to be specified explicitly in the shared function and data type declarations. 

The compiler used to build the 'guest' application will expect that his own version of the standard C library will be used at runtime on the target system. So either the os will have to load the application's version of the C Library to run it, or either assume that the definitions of the C functions used by the application are consistent with the C library used at runtime. 

If specific declaration of the function of the C Library are used, need to be sure the guest compiler use those definitions to compile the application, sometime compilers can override the definitions of the standard C functions with the type it expects from the standard C library used to build it, because it has certain specific way to optimize or deal with the call to those functions.

The configuration for an operating system is similar to the one of a framework, but need to take extra care about compiler specifics, because several version of a framework can be distributed to fit a particular building environment, but it's much harder to ship a version of the OS for each building environment that can be used to build an application or a component/module for it.

----

==how C compiler deal with call to the standard lib C==

As mentioned previously, compilers can sometime have a specific behavior when they encounter a call to a functions of the standard C library, and won't always behave exactly as expected from function declaration. This behavior is very compiler specific, and depend on options used for compilation, but it can either totally inline code, or use specific functions of the runtime to replace the call, or even optimize successive call in function or whole program level.

For example, given the following code

<source lang="c">
float c,s,a;
a=1.0;

c=cos(a);
s=sin(a);
</source>

The compiler can optimize into

<source lang="asm">
fld [a]
fsincos
fstp [c]
fstp [s]
</source>

And use the fsincos instruction instead of two successive call to the standard C sin and cos functions.

The compiler can just inline some assembler routine to optimize call at function or whole program level, and/or insert call to its own C runtime library. It can use implicit declaration of the standard C function instead of the one explicitly defined in the header , or issue warnings if some of it's internal functions are redefined. If the program compiled has to be inter - compiler compatible on a binary level with the OS implementation of the C Library, it's safer to use functions with a different name than the functions of the compiler's standard C library.

There are many problems that can arise on the assembly level regarding what a compiler will do when encountering a call to a function of the C library. What it will expect of the C library code is not always defined explicitly, and that can break compatibility between an application and the host system due to the C library of the host used at runtime being incompatible with what the compiler expects.Even if the application use functions declarations consistent with the definition of the function in the host system.
----

==how to avoid problems==

====always using the headers and C library specific to a build environment to build everything ====

The system will contain a single C library, and all programs compiled must use the corresponding C headers to declare function of the C library present in the system.

*pro
#Can use the C library and headers of the building environment.
#Can take full advantage of compiler and building options.
#The compiler will be able to optimize the code that use functions of the C Library.
*con
#C library headers often contain compiler specific directive, so it's most likely you need to use the same compiler to use the same headers file to declare the functions of the C library.
#Errors or inconsistencies can happen if two programs that need to use the same C library use different C Library headers, or use same C Library headers but are compiled by different compilers or with different options.
#Need to make sure the C Library used at runtime is compatible with the one that is used to build it.

====not using the build environment specific headers and C library nor runtime====
Not using any functions or data type declaration that depend on the build environment specific implementation of the standard C function.
*pro 
#all programs can be compiled with any compilers without having inconsistencies and can include the same cross compiled headers to declare the equivalent of the C functions.
#Executable file will be smaller because they don't need to incorporate the runtime
#Two program compiled with different compilers can be linked statically with each others
#Can control entierly the way the compiler will process calls to the C library functions
*con
#Compiler can't optimize the calls to the function of the C library.
#Need to rewrite the function of the C library that program uses and eventually part of the runtime of each compilers that you want to be compatible with, or making explicit call to specific function to perform the operation instead of using the compiler runtime library implicitly.
#Compiler's C library are generally more optimized, and can be adapted automatically by compiler depending on compilation options.
#Can make the port of application a bit harder, but most of the job can be done using preprocessor directives, and lot of application and library that are 'portable' give an easy way to use system specific function to replace the standard C library.


----

== the make up of a typical C Library==

Even if you want to get rid of building environment dependence, you still want to implement functions with an identical behavior to be able to port easily applications that use the standard C library to your os, but without depending on anything specific to the build environment for declaring , implementing and compiling them.

It is also preferable to declare the replacement for the standard C functions in header file which name are different from the standard library include file name (stdlib.h,string.h,math.h etc) , because the wrong one could be selected if include path are not configured properly, so it's better to use a different name for both the name of the header file in which the function are declared, and a different name for the C function themselves, to make sure the compiler will effectively deal with the functions as defined explicitly in their declarations and that they will not conflict with compiler internal/built in definitions.

====the standard C library====

======stddef.h======

This file is mainly used to define basic type that other functions definitions will use, it must be included first as all the declaration of the C standard library depend on them.

It often include compiler and system detection using preprocessor conditional compilation.

Each compiler will define a certain number of preprocessor variables automatically, and some code can be compiled conditionally depending on the target arch, operating system, version of the compiler , that are expected to be set by the compiler or the building environment. Like this, the same header files can be used to compile for different platforms.

The exact interpretation of the C library header depend on many variables that are expected to be set by the compilers and that are specific to each compiler.

======malloc.h======

This file contain declaration for functions related to memory allocation, such as malloc, calloc, realloc, and free. According to the C standard, this file should not be included directly, and the declaration of those function should be used by including stdlib.h, as malloc.h will not always be present in all C Library. 

======string.h======

This file contain declaration for functions related to string and memory manipulation, such as memcpy, strcpy, memset, strcat, strtol, itoa etc


Most C Library will use a very complex syntax to declare these functions, because the good declaration depend on variable set for a specific build, and will contain lot of compiler specific directive that will often be hard to use with other compilers. It can be use to toggle feature used for debugging, or feature of the compiler's C runtime, which tend to make it hard to have standard C functions declaration that are consistent across different version of C headers from different implementation of the C Library, and it's most likely each building environment will require his own version of the C function header declaration to function properly with all possible compilations options.

This is an example of string.h from android glibc2.7-4.6

<source lang="c">
#define strcpy(dest, src) \
  ((__ssp_bos (dest) != (size_t) -1)                                    \
   ? __builtin___strcpy_chk (dest, src, __ssp_bos (dest))               \
   : __strcpy_ichk (dest, src))
static inline __attribute__((__always_inline__)) char *
__strcpy_ichk (char *__restrict__ __dest, const char *__restrict__ __src)
{
  return __builtin___strcpy_chk (__dest, __src, __ssp_bos (__dest));
}



</source>

======math.h======

This file contain declaration for functions related to math functions, either to deal with floating point numbers, trigonometric functions, or arithmetic/logical operations

======stdio.h======

This file contain declaration for functions related to i/o functions, mostly used to deal with file and text formatting functions, such as fopen, fread, fwrite, fclose, printf, sscanf etc




====the C runtime====

It often take the form of a static library that will be linked statically into the executable file generated. Any specific functions the compiler need to generate executable code for a particular plateform that is not part of the set of standard C functions will be part of the compiler's C runtime.

Features of C compilers that will require some functions from the C runtime:

*memory and stack runtime security check
*performance profiling
*RTTI (run time type information for c++ class using virtual functions)
*64 bit integer arithmetic or logical operation on 32 bit arch
*system specific functions to initialize and terminate the application before and after the execution of the 'main' function.

to have compiler independent executable either :

*Those feature must be disabled.
*Write an implementation of the compiler's runtime, the source of the runtime is often shipped with compilers, so you can copy / paste the code of the runtime function used, and declare them either statically in the exe/shared lib, or resolve reference to them at runtime as imported function.
*Make your own version of the functions to replace the compiler's specific runtime function in sort that it doesn't have to generate any call to its runtime during compilation.


It's not absolutely necessary to avoid dependence on a C Compiler runtime for executable or shared library, as the code will be linked statically with them, but they can include code that reference system specific functions, specially for the code that relate to application initialization and termination, and it can be a bit of overhead in the exe file. Some of the runtime function can be ok, but it's hard to make sure which one the compiler will use unless you prevent it to link it's runtime in the exe, in which case you'll see the reference to the runtime as unresolved symbol at link time. And you might not want to have to deal with specific initialization code that compiler's runtime may need to run the executable.

For static library, it is necessary if the static library has to be linked with a program compiled with a different compiler. Because the runtime of the library's compiler will conflict with the runtime of the other program's compiler.

==how to write build environment independent code==

Most program made in C will use some functions of the standard C Library, the policy can either be :

*Making it easy to port dependence on the functions of the C library in existing application. Then need to implement a library that is as compatible as possible with other C Libraries, but without using anything specific to the build environment to implement it.
*To provide a totally different internal api to manipulate memory, strings , math and file, and then need to totally rewrite code that use them in an application to port it.
*To provide a specific internal api, but also providing the functions compliant with standard C library.

The simplest is to declare functions with the same parameters and return type than the ones of the standard C Library, but with a different name. Using a suffix or prefix in the name of the function declaration is generally convenient, application will have to use this name instead of the standard C function name.

Those functions should be defined in header files that have the same organisation than the standard C library to make porting easier.

The header file of the standard C library used to compile the applications will not be the ones the C compiler expect as the C Library header, but an os specific version of the header containing the declaration of the standard C functions replacement, and all dependences to the functions of Standard C Library in the application will be switched to this specific library that the compiler will not recognize as functions of the standard C Library.

====intel calling conventions====

Each cpu architecture have their own set of calling convention, most intel compiler will support any of those in a standard manner:

*for intel i386
#__cdecl : arguments are passed on the stack, and the stack is restored by the caller after the call, integer return value in eax
#__stdcall : the argument are passed on the stack, and the stack is restored by the function ,integer return value in eax

__cdecl is safer, because the functions have no way to know how the caller passed the argument, but __stdcall is faster and more compatible even with older compilers. on i386 linux, gcc will use a calling convention identical to __cdecl by default.

*for intel x64
#sysv : calling convention used by default in intel 64 bit sysV platform.
#microsoft : calling convention supported by microsoft 64 bit compilers and 64 bit system.


====creating simple header file to replace the standard C library declarations for i386====

so this declaration in string.h from gcc C Library header file

<source lang="c">
#define strcpy(dest, src) \
  ((__ssp_bos (dest) != (size_t) -1)                                    \
   ? __builtin___strcpy_chk (dest, src, __ssp_bos (dest))               \
   : __strcpy_ichk (dest, src))
static inline __attribute__((__always_inline__)) char *
__strcpy_ichk (char *__restrict__ __dest, const char *__restrict__ __src)
{
  return __builtin___strcpy_chk (__dest, __src, __ssp_bos (__dest));
}

</source>

would become this in build independent os_string.h

<source lang="c">
char *os_strcpy (char *dest,  const char *src);
</source>



====adding compiler specific directive====


now it still need to specify the functions attribute like calling convention , if the function is local, imported or exported, in a way that every compilers will understand and apply, so you need the equivalent of an stddef.h 

os_def.h

if compiling for x64, the calling convention for x64 need to be used instead of __cdecl. the Microsoft calling convention will be supported as well by gcc, so it would be preferable to use, as Microsoft compilers do not support the sysV calling convention.

<source lang="c">
#ifdef MSVC_VER
#define OS_API_CALL __cdecl
#endif

#ifdef GNU_C
#define OS_API_CALL attribute('__cdecl')
#endif
</source>

The further declaration of the C functions then use preprocessor variable to generate the code specific to the compiler used to compile it.

os_string.h

<source lang="c">
char * OS_API_CALL os_strcpy (char *dest,  const char *src);
</source>


====implementing the function====

Compiler specific attributes doesn't need to be specified in the function implementation, because the header file containing its declaration is included so the compiler will consider automatically that this function is the implementation of the one being declared in the header, and apply automatically all the attribute present in the declaration. If the return type, name and argument type of the function definition match the declaration, the compiler will apply automatically all the attributes of the declaration to the implementation.


os_string.c
<source lang="c">
#include "os_def.h" 
#include "os_string.h"

char * os_strcpy (char *dest,  const char *src)
{
      //copy the string and return;
}

</source>


====using it in an application====

main.c

<source lang="c">
#include "os_def.h" 
#include "os_string.h"

int main(int argc,char **argv)
{
   char my_string[32];

   os_strcpy(my_string,argv[0]);

}
</source>


Like this, it make sure the compiler will use the good declaration of os_strcpy , and any specific syntax needed by the compiler for the particular build is handled in os_def.h.

Then program should be compiled with '-nodefaultlibs' with gcc, or 'nodefaultlibrary' (ignore all default library in the linker settings) with visual studio , in sort that it doesn't attempt to link either the runtime or any build specific library in the generated executable code, which make the executable generated as independent as possible on the build environment, and dependent explicitly on your os specific implementation of the C Library functions.


====making port easier====
Some preprocessor directives can be used to automatically transform call to the functions of the standard C library to call made to the os specific function.

<source lang="c">
char * OS_API_CALL os_strcpy (char *dest,  const char *src);

#define strcpy os_strcpy
</source>

Like this, the preprocessor will transform any direct call to strcpy to os_strcpy automatically before the compiler start to parse the file, so all references to standard C functions can easily be translated to call to the os specific functions to make the port of application that make direct call to function of the C library easier.

Using a preprocessor definition is not always reliable, because some library or other programs can redefine the C functions as macros, but normally it should issue a warning if that happen, and either the redefinition in the library should be ignored, or either the definition of those macros should be set to use the os function. 

====handling import/export====

Imports and exports are not essentially handled by the compiler, they are handled by the linker and depend on the ABI used, but compilers often have a way to tell the linker how the functions should be resolved at runtime. 

When using imported function, the os will be responsible for resolving the address of all imported symbols at runtime.

If the C library is to be compiled as a shared Library, then the function need to be declared as local exported functions when compiling the C library, and as imported function in all the programs that will be linked with it.

os_def.h

<source lang="c">
#ifdef MSVC_VER

  #define OS_API_CALL __cdecl

  #define OS_SHARED __decslspec(dllimport) 
  #define OS_EXPORT __decslspec(dllexport) 

#endif

#ifdef __GNU_C__
  #define OS_API_CALL attribute('__cdecl')

  /* 
   All symbols are exported by default in elf and symbols are just set as unresolved at link time if they are not defined, so empty declaration. 
  */
  #define OS_SHARED                                  
  #define OS_EXPORT 
#endif


#ifndef OS_LIBC_FUNC
  #define OS_LIBC_FUNC OS_SHARED
#endif

</source>

os_string.h
<source lang="c">

OS_LIBC_FUNC char * OS_API_CALL os_strcpy (char *dest,  const char *src);

#define strcpy os_strcpy
</source>


os_string.c
<source lang="c">

#define OS_LIBC_FUNC OS_EXPORT

#include "os_def.h" 
#include "os_string.h"

char * os_strcpy (char *dest,  const char *src)
{
      //copy the string and return;
}

</source>



main.c

The source of the application shouldn't change, excepted the standard C function name can be used due to the preprocessor definition. 



<source lang="c">
#include "os_def.h" 
#include "os_string.h"
 
int main(int argc,char **argv)
{
   char my_string[32];
 
   strcpy(my_string,argv[0]);
 
}
</source>


==implementing the base C functions for intel cpu==
````

<a id="w003405"></a>

## W003405 — Code Management

Bron-ID: W003405
Namespace: Main (0)
Revisie: 25368 | Datum: 2020-11-19T19:25:42Z
Bron: https://wiki.osdev.org/Code_Management
Revisie-URL: https://wiki.osdev.org/index.php?title=Code_Management&oldid=25368

````text
An Operating System is, by its very nature, a quite large project. As such, it requires proper code management, otherwise your code base is likely to turn into a big mess that will only keep getting harder to maintain as the project grows, and you will end up rewriting your code base every month because it has become impossible to maintain, instead of actually implementing new features. On the other hand, code management must be scaled to the project: a project that doesn't mean to grow very big does not need a complicated organization.

== Consistency in the codebase ==
The first step in writing a codebase that is easy to maintain whatever the count of people working on it, is to ensure consistency. This applies to various fields, which will be detailed within this section.

=== At the source level ===
In a large project, it is important that all source files be written the same way. It starts with the coding style. Everybody has his own and likes to use it, but large projects need to impose one coding style for the whole project, or people willing to contribute will spend all their time deciphering the coding styles of the others, and lose any will to help out, resulting on your project to be abandoned. Most projects also impose that each source file contains a heading comments which typically contains a description of the file's purpose, its filename, and its licensing conditions (or a reference to the license, in case of long licenses like the GPL). Additionally, projects should impose other aspects of the coding standard, such as the tab policy (the way indentation is performed through tab characters or spaces, and how many spaces per indentation level), the preferred comment style (either C++-style ''//'' or C-style ''/* ... */'' for which purpose), or the encoding (pure ASCII, UTF-8, etc.) of all source files.

Perhaps the most essential thing to define is the naming convention (that is, how functions, structures, types, macros, variables etc. are named) in the interface (see [[#Interfaces,  Implementations, and Black Boxes|Interfaces, Implementations and Black Boxes]] below for more details). This is less important for internal (e.g. ''static'' in C) functions which are not exposed, though it's good practice to keep the same standards everywhere.

The last concern is the method that is used to accomplish certain tasks that are used in multiple places across the OS. For example, the C++ language allows two ways of allocating memory (''new'' and the C standard ''malloc''). A good project has well-defined conventions that are followed throughout the project.

Fortunately, with the evolution of modern IDEs, following the coding conventions of a project has become a simple matter of setting up the IDE to use the correct encoding/tab policy/headers, and using the IDE's automated source formatting utilities to produce correctly formatted source files. Some IDEs can even create the heading comment block directly at the start of each source file you create, so that you only have to fill in the gaps.

=== In the source tree ===
The other place where it is essential to have a consistent organization is in the project tree.  In other words, the way files are classified in subdirectories '''must be consistent'''. This is more of a concern in big projects, and becomes essential when developing an operating system. Because one day or another you will want to port your operating system to other architectures, it is necessary to place source files that are architecture-dependent (e.g. x86-specific paging management) apart from architecture-independent ones (e.g. your command-line shell) (see [[Portability]])). [[GRUB]] does that with an "''arch''" directory aside from the source files containing subdirectories for different platforms (e.g. x86, ARM, SPARC, MIPS...). Then, you should name your source files in a uniform manner. For example, C++ headers must have the same extension (''.h'', ''.hpp'', ''.hxx'') throughout the project. Here's another example: let's say you have an extensible command-line shell, and have commands implemented in different source files. You must decide whether you're going to create a subdirectory specifically for each commands (which would be better, e.g. "cmd/xxxxxx.c" and "cmd/yyyyy.c") or name them ''cmdxxxxx.c'' and "cmdyyyyy.c", or some other naming scheme.

=== In the versioning scheme ===
Although this is usually less important than the previous concern, you need to keep the way different versions are numbered consistent. Some number versions sequentially (e.g. ''0.1'', then ''0.2'', then ''0.3''...) while others increment the minor version (e.g. the ''2'' in ''1.2'') number sequentially for small updates and only increment the major version number (the first number in the version) only when they add a major improvement to the projects. Some also add revision and build numbers, which are incremented by some random number every time a change is made to a file, which gives weird version numbers such as ''2.2.11127.56150'', which are hard to remember.

Additionally, you may give each release a special name.  Projects that use that strategy include Mac OS X (e.g. Leopard, Lion, Mountain Lion...), Android (e.g. Gingerbread, Jelly Bean, Ice Cream Sandwich...), Windows (e.g. Millennium, XP, Vista), and probably others I don't know about. Just pick the one you prefer, or something completely different, or even more simple, such as ''MyOS 1'', then ''MyOS 2'', then ''MyOS 3''... it's your project, after all!

[http://semver.org/ "Semantic Versioning"] is an attempt to unify versioning schemes. You probably already use a scheme that is close to this one.

== Interfaces, Implementations and Black Boxes ==
When dealing with a large codebase, such as that of an operating system, code can quickly turn into a big mess. In order to avoid that, you need proper code organization. Note that it's still perfectly possible to code without using this organization, but organizing your codebase highly increases the chances that your project to succeed, and be stable.

It's all about splitting the ''interface'' from the ''implementation''. The ''interface'' is made of function prototypes, structure definitions, typedefs, structures and possibly preprocessor macros in C, plus classes in C++. Note that different languages support different interfaces: while some languages such as [[D]] or [[Java]] provide built-in support for that through special keywords and constructs, while other languages don't (assembly is an example, though few OS's are written completely in assembly).

=== Interface ===
The interface is the part that is visible to the user. It is usually present in header files, and consists, for the C language at least, of function prototypes, structure and type definitions, and preprocessor symbols and macros. This part should not contain any code, except for small inline functions that usually defer the call to other functions, with different arguments formatting. An example of that are the ubiquitous utility ''inb/inw/inl/outb/outw/outl'' inline assembly functions.

When writing an operating system kernel, one usually wants the interface to be platform-independent. This can be achieved by using ''typedef''s to represent each element. For example, when implementing paging, you may use a specific typedef for physical addresses, and another for linear addresses. If you refer to physical addresses as "phys_addr_t"'s, you will be able to reuse the same interface when adapting your kernel to x86_64 (assuming a start with the x86) by simply changing the typedef of "phys_addr_t" from ''uint32_t'' to ''uint64_t''.  You can also just use conditional compilation, or some similar feature your language of choice provides.

It is very important that the interface be completely independent of the implementation. That will allow you to write a second implementation of your Hardware Abstraction Layer (the part of a kernel that abstracts the hardware, for beginners) for a different platform later, without having to change the interface. And assuming that your interface is platform-independent enough, and your kernel uses it '''and only it''' to access the hardware, then porting to a new platform will simply be a matter of writing a new HAL, and the rest will magically compile and work perfectly on the new architecture (for more info, see [[Portability]]).

For this to work, you also have to ensure that the rest of the kernel is really platform-independent. In fact, the interface of your kernel should never directly use fixed-width types (such as ''uint32_t''). If the fixed-width is required by the platform (such as special registers), then you should have a special typedef for it (e.g. special_register_t). Otherwise, if you just want to ensure a given capacity, then use the standard type that is closest to what you want. A good example of that is the C standard library, which uses special type for file offsets, sizes, times, etc, but uses (unsigned) char/int/long when necessary. And as you can see, the interface to the C library remains the same through countless different platforms. A well-designed kernel interface allows the same thing to apply to... your kernel.

=== Implementation ===
The implementation is the opposite of the interface. That is, it contains the actual ''implementation'' of the functions defined in the interface. The implementation is allowed (and in fact, has to, in the context of a kernel) depend on the platform. It is in the implementation that you write code that directly accesses specific hardware (most notably inline assembly). (Code that addresses specific hardware should be placed in separate drivers, according to your kernel design). However, if you want your code to remain readable, I suggest that you still use the C preprocessor for ''magic'' values, such as fixed addresses of memory-mapped devices (e.g. the 0xB8000 address of the VGA's text mode video memory), with explicit names (e.g. VGA_TEXT_MEMORY).

The implementation can be as ugly as it wants or needs to be, as long as it properly implements the interface. However, to achieve proper isolation between interface and implementation, it is important that functions that are internal to the implementation ''not'' be accessible from outside it. The C language provides the ''static'' keyword for that.

A special case are bigger components, whose implementation can be considered as a separate library. In that case, a common practice is to have an internal implementation-specific interface. However, and since internal functions can't be declared ''static'', it's up to the developer to ensure that they are not used from outside the implementation. There are countless professional libraries using such techniques. Good examples are the industry-standard ''boost C++ libraries'' with their "''detail''" directories. An example of such components are complicated device drivers, such as those for hardware-accelerated video cards, or complicated filesystems.

=== Black Boxes ===
In time, you will probably want to extend your development team, if not already done. But once you will have more people working on the codebase, you will face a problem: the more they are, the longer it takes to develop. The reason for that is that each developer in charge of one part of the kernel needs to take in account the code written by other developers in charge of other parts of the kernel. That's when black boxes come into play.
The ''Black Boxes'' approach consists in treating your kernel as a set of separate libraries (the Hardware Abstraction Layer is a library, the VFS is another etc.). Once your project is properly divided into pieces, you can give each developer his library, and no longer worry about conflicts. Combined to interface and implementation isolation, you will have each team try to implement its part of the interface as well as possible. With a properly-designed interface, this will make the development process much faster and easier to manage, for the project leader.

That approach will also help you during the recruitment process of your development team. In effect, you will realize that there are many people who decided to start their own OS project with the goal of making the best possible GUI, but lose any will to continue that project when they realize that they have to implement countless other things before. If you announce on the forum that you are looking for people to design a GUI under an already mature kernel, these people will be happy to join your project, since it allows them to work only on what they like, without to worry about the rest.

It will also make it easier to fix bugs. Because each part of the kernel is clearly separate from the rest, it's easy to locate an individual problem with the code, and fix it, without also having to modify a lot of other parts of the kernel which relied on it, due to a lack of precise interface. Due to that, your kernel has chances to be more stable than average in the end.

Now comes the problem of the interface. Since every component is neatly isolated from the rest, and implements the interface, you need to have a well-designed interface, and clear rules about who can modify it. If anybody can change the interface, it is useless to have one, because your team will spend its time adapting the implementation to suit the new interface, instead of actually coding things. A (good) scheme that proved to work is to have, on your project page, a forum with a topic dedicated to discussing the interface. Then, when somebody wants to change the interface, he will need to present his change to the whole developer community, with arguments, and then it will be up to you to decide whether the change should be done or not.

== Working across changes ==
It is obvious that your code will have to change, and sometimes a change is not as good as you thought it would be. In this case, you will want to roll back the change. You may also encounter situations where you remove a part of your code (e.g. you want to completely change your scheduler) and later want to see that code again (e.g. your new scheduler supports different methods and you now want to re-include your old scheduler's code).
:'''Note:''' ''If you are already using a version control system, you'd better skip this section :)''

=== Version Control Systems ===
A [https://en.wikipedia.org/wiki/Version_control Version Control system] (VCS) is a program that manages changes to your source files (for OSX users, this is much like Time Machine, but enhanced for source code). Concretely, you perform a ''commit'' operation after each significant change  (e.g. after you rewrote your scheduler, or added a new driver, don't commit after each modified line), and the VCS will remember the state of the files before and after the changes. This means that if you change your mind and want to see your code again, then the VCS will be able to give it back. A VCS also enables you to generate ''patches'', which are files that contain only the differences between two versions of a same file.

But the main point in using a VCS is that it enables two (or more) persons to work on the same codebase at once without interfering with each other. A VCS also allows you to place your source code on the Internet and have your whole team co-operate using a single server. Many source code hosting websites (such as Google Code) support accessing your codebase through a VCS, and using such tools for your code gives you more chances to get people to contribute to your project.

[[Category:Organization]]
[[Category:OS Development]]
````

<a id="w003406"></a>

## W003406 — File:Compilers1.png

Bron-ID: W003406
Namespace: File (6)
Revisie: 15210 | Datum: 2013-09-29T22:49:17Z
Bron: https://wiki.osdev.org/File:Compilers1.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ACompilers1.png&oldid=15210

````text
Overall view of what a compiler does, in most general terms.
````

<a id="w003407"></a>

## W003407 — File:Compilers2.png

Bron-ID: W003407
Namespace: File (6)
Revisie: 15211 | Datum: 2013-09-29T22:50:06Z
Bron: https://wiki.osdev.org/File:Compilers2.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ACompilers2.png&oldid=15211

````text
The internals of your average compiler.
````

<a id="w003408"></a>

## W003408 — Making a Compiler

Bron-ID: W003408
Namespace: Main (0)
Revisie: 29987 | Datum: 2026-02-12T17:19:55Z
Bron: https://wiki.osdev.org/Making_a_Compiler
Revisie-URL: https://wiki.osdev.org/index.php?title=Making_a_Compiler&oldid=29987

````text
{{Template:In_Progress}}

So you want to make your own [[compiler]], from scratch?

That won't be easy. Some compilers are comparable in complexity to entire simple operating systems.

Still, it can be a fun and/or educational project. Here are some things you should know before you get started:

==Pros and Cons==
Why write a compiler instead of using an existing one?

Pros:
* A compiler for the language you want and the target you want might simply does not exist.
* Educational, you get a better idea of [[How kernel, compiler, and C library work together|how the kernel, compiler, and the run-time libraries work together]], and what it takes to implement a language.
* Speaking of, you can implement your own new programming language, right then and there! How cool is that?
* You can do just about anything with the program being compiled:
** Add debugging data in your favorite format
** Add profiling aids
** Re-write and re-arrange parts of the program
** Optimize for something specific
** Throw it out and replace it with another program
* The compiled programs would fit your machine or OS like a glove, even if it's a new OS you're developing.

Cons:
* Complete understanding of the language required. Always keep the official language spec at hand.
* Understanding of assembly specific to your target is also required.
* Most of the notes on [[Getting Started#Getting motivated|OSDev motivation]] apply to compilers as well.
* It will be tricky to fully comply with the language spec.
* You would have to be really, really good at this if you wish to compete with GCC in terms of size/speed optimization of the compiled code.

==Terminology==
Well, if you're still up to it, let's now look at how compilers work:

[[Image:compilers1.png]]

Terminology:
* The "Host" is the environment where the compiler itself runs.
* The "Target" is where the compiled program is meant to run.
* The "run-time" is a set resources (libraries, processes, etc) that exist on the Target. Two machines with identical hardware that differ in availability of run-time resources are sometimes thought of as different targets, as a program might run on one but not the other.
* The "[[Executable Formats|executable]]" is a file containing information necessary for the Target to launch the program. It can be a flat binary, but is usually more elaborate, containing for example linking and re-location information.
* The "link" signifies that the program knows how to interface with the run-time, and relies on the run-time for some of it's functionality. It is created by the [[Linkers|linker]], and does not exist in free-standing programs (i.e. OS kernel).

If the host and the target are the same the product is a "native executable". This is what you see 99% of the time.
If the compiler is capable of producing an executable for a target different from the host, it's a "cross-compiler".
If the compiler is capable of compiling itself, it's "self-hosting".

==Internals==
[[Image:compilers2.png|right]]
Compilers also have a peculiar internal structure. You do not have to replicate the one described here exactly, but it's a good road map for now. Generally, the task of compiling a program is subdivided into several individual tasks:

* The '''Front End''' takes the source code for some specific language, and outputs a universal "intermediate representation" (IR).
* The '''Middle End''' (optional) takes the intermediate representation, changes it in some way, and outputs it back.
* The '''Back End''' takes the IR and outputs an executable for a specific target.

The idea for this division is that by having a collection of different front-ends, and different back ends, you can compile a program in any language, to any target, because the IR is the same for all of them.

===Front End===

*First of all, the front end usually includes some sort of user interface, so that files and options can be accepted. Then, there are several stages of processing. 
* For C-like languages, a '''pre-processor''' copies and pastes the code from header files, removes comments performs macro expansion and does other relatively simple tasks.
* Then, a '''scanner''' (or '''tokenizer''') breaks up the source file into a series of tokens, representing basic words of a language (i.e. identifiers, number and string constants, keywords like "if" and "while", punctuation marks and semicolons)
* Then, a '''parser''' reads the stream of tokens and constructs a '''parse tree''', a tree-like structure that represents the structure of the source code. In terms of language-making, each source code file is a "sentence" of the language, and every sentence, if it is valid, follows a specific syntax.
* Next, a '''semantic analyzer''' walks the parse tree and determines the specific meaning of each part of that "sentence". For instance, if a statement is a declaration, it adds an entry to a symbol table, and if it is an expression involving a variable, it substitutes an address of the known variable from the symbol table for the variable inside the expression.
* Finally, the Front End generates an '''Intermediate Representation''', which captures all the information specific to the source code of the program. A good IR says everything that the source says and more, and also captures additional requirements that the language places on the program. It can be in the form of a '''parse tree''', '''abstract syntax tree''', '''three address code''' or something more exotic.

It's worth noting that this break-up is purely for convenience of the compiler developer. Most types of '''parsers''' can easily read a stream of characters as opposed to a stream of tokens, and so do not need a separate scanner. Some very powerful classes of parsers can even determine the semantic meaning of statements ''before they're even done reading'', and so do not need a semantic analyzer. They might be more difficult to understand and work with, though.

===Middle End===
Applies a whole bunch of algorithms meant to try and make the program more efficient. I'm not really familiar with this part, so I won't go into details. Since all it does is optimization, you could omit it for simplicity.

===Back End===
* First, the '''code generator''' reads an intermediate representation, and emits assembly-like code, the only difference being that it assumes a machine with infinite number of registers (or some other resource) for simplicity.
* Second, a '''register allocator''' juggles available registers on the CPU by deciding when a variable is to be stored in a register or in memory. This part has ''major'' impact on performance. Once it's done, the assembly-like code becomes true '''assembly'''.
* Then, an '''assembler''' reads the assembly and outputs '''machine code'''. Usually, the code still has some unresolved references, so the machine code is put inside an '''object file'''.
* Next, a '''linker''' satisfies every unresolved reference in an object file by linking it against a library and substituting a pointer to the resource being referred to inside the library. The linker also supplies necessary information to locate the relevant library when linking dynamically, or, when linking statically, some or all of a particular library might be embedded in the object file. The linker is sometimes thought of as being separate from the back end or even the compiler, but the truth is, the operation of a particular linker is specific to the target (or a family of targets) that it links for, so I included it in this category.

*Finally, the back end converts the object file with all references resolved into an executable by adding meta-data necessary for the Target's program loading mechanism to recognize the program, load it, dynamically link it if necessary, and run it.


Soon, I'll go over how to actually write each of those. [editor's note: "soon" doesn't seem to be that soon after all]

==See Also==
===Articles===
* [[Books#Compiler_Theory|Books, Compiler Theory section]]

===External Links===

* [https://craftinginterpreters.com/ Crafting Interpreters] - has a very good and approachable description of lexing, parsing, and ASTs. Also covers compiling to bytecode, with techniques that can be adapted to emit unoptimized machine code.
* [https://www.cs.cornell.edu/courses/cs6120/2020fa/self-guided/ CS 6120: Advanced Compilers: The Self-Guided Online Course] - an introduction to various code optimization techniques
* [https://cfallin.org/blog/2023/01/20/cranelift-isle/ Cranelift's Instruction Selector DSL, ISLE: Term-Rewriting Made Practical] - an example of how instruction selection can be accomplished
* [https://cfallin.org/blog/2022/06/09/cranelift-regalloc2/ Cranelift, Part 4: A New Register Allocator] - an example of a non-trivial register allocation technique
* [https://github.com/bytecodealliance/rfcs/blob/main/accepted/cranelift-egraph.md Cranelift: Using E-Graphs for Verified, Cooperating Middle-End Optimizations] - describes an interesting technique for solving the phase-ordering problem

===Bibliography===

* ''Engineering a Compiler'' by Keith D. Cooper, Linda Torczon
* ''Advanced Compiler Design and Implementation'' by Steven S. Muchnick
* ''Compilers: Principles, Techniques, & Tools'' by Alfred V. Aho, Monica S. Lam, Ravi Sethi, and Jeffrey D. Ullman

[[Category:Compiler Development]]
````

<a id="w003409"></a>

## W003409 — Talk:Making a Compiler

Bron-ID: W003409
Namespace: Talk (1)
Revisie: 15214 | Datum: 2013-09-30T19:52:20Z
Bron: https://wiki.osdev.org/Talk:Making_a_Compiler
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMaking_a_Compiler&oldid=15214

````text
This looks interesting. I'll read it more carefully later. Quick thing: The platform on which programs are built is usually called the 'build platform', while the platform on which programs run is called the 'host platform' (if the program creates output for some particular platform, that's the 'target platform'). It's probably best to stick to this terminology as its standardized on Unix and helps lessen confusion when people use the same word for different things. --[[User:Sortie|Sortie]] 11:35, 30 September 2013 (CDT)
: Agreed.  Also, I'd probably separate the linker from the compiler back-end, primarily as otherwise someone reading this article might also expect it to describe how to create a linker in the 'Back-end' section. [[User:Jnc100|John]] 14:52, 30 September 2013 (CDT)
````

<a id="w003410"></a>

## W003410 — Device Driver Interfaces

Bron-ID: W003410
Namespace: Main (0)
Revisie: 15223 | Datum: 2013-10-02T22:07:49Z
Bron: https://wiki.osdev.org/Device_Driver_Interfaces
Revisie-URL: https://wiki.osdev.org/index.php?title=Device_Driver_Interfaces&oldid=15223
Doorverwijzing: Device Management

````text
#REDIRECT [[Device Management]]
````

<a id="w003411"></a>

## W003411 — Talk:Device Driver Interfaces

Bron-ID: W003411
Namespace: Talk (1)
Revisie: 15225 | Datum: 2013-10-02T22:07:49Z
Bron: https://wiki.osdev.org/Talk:Device_Driver_Interfaces
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ADevice_Driver_Interfaces&oldid=15225
Doorverwijzing: Talk:Device Management

````text
#REDIRECT [[Talk:Device Management]]
````

<a id="w003412"></a>

## W003412 — Talk:ISO 9660

Bron-ID: W003412
Namespace: Talk (1)
Revisie: 15275 | Datum: 2013-10-18T08:55:35Z
Bron: https://wiki.osdev.org/Talk:ISO_9660
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AISO_9660&oldid=15275

````text
== Proposals for enhancements ==

I am the developer of xorriso, which produces bootable
ISO 9660 images. Here are some proposals after reading
this wiki page.
I am ready to implement them here. But first i ask
whether such changes would be appreciated.

------------------------

"Filenames must be in the 8.3 format":

ECMA-119 filenames may be longer than 8.3 format.
And they should bear ";" and a File Version Number.

ECMA-119 7.5 "File Identifier" says:
  The sum of the following shall not exceed 30:
  * if there is a File Name, the length of the File Name;
  * if there is a File Name Extension, the length of the File Name Extension.

ECMA-119 10 "Levels of Interchange", prescribes a
maximum of 8 characters for File Name and 3 for
File Name Extension only for Level 1. Levels 2 and 3
allow the sum of 30.

------------------------

"The Boot Record" should refer to http://wiki.osdev.org/El-Torito
See its Talk page for my proposals there.

Currently the OSDev wiki has a gap between Boot Record and
the boot images which lead the firmware into boot loaders
resp. operating systems. The Boot Catalog should be described
in the wiki page about El-Torito.

But one should mention here that bytes 71 to 74 tell the
2 kB block number of the Boot Catalog as little-endian
32 bit number. (There is no big-endian copy of that number.)

------------------------

One should mention that the first 32 KiB of an ISO 9660
image are called System Area and may store arbitrary data.
For example it is possible to have a Master Boot Record
(MBR, for BIOS) and a GUID Partition Table (GPT, for EFI)
for booting the ISO image from USB stick or hard disk.

------------------------

One should mention the existence of Rock Ridge (for POSIX
and more) and Joliet (for MS-Windows names in UCS-2 characters).
It seems better to keep ISO 9660:1999 a secret ...

Rock Ridge (RRIP) is implemented as ECMA-119 System Use
data in ECMA-119 Directory Records. Its general format
is specified by the System Use Sharing Protocol (SUSP).
  ftp://ftp.ymi.com/pub/rockridge/susp112.ps
  ftp://ftp.ymi.com/pub/rockridge/rrip112.ps

Joliet is implemented as separate tree of Directory
Records. Its root Directory Record is stored in a 
Supplementary Volume Descriptor.
  http://www.buildorbuy.org/pdf/joliet.pdf

[[User:Scdbackup|Scdbackup]] 03:55, 15 October 2013 (CDT)

: Thanks for the info.  If you get a chance, please could you go ahead and make the appropriate changes to this page (and any others) as you see fit as you are presumably one of the people with the most experience of CD filesystems around here.  [[User:Jnc100|John]] 00:18, 18 October 2013 (CDT)

:: Done now. I broke it down into several smaller changes.
:: I propose to remove the "In Progress" mark at the start of the page.
:: It is now quite comprehensive. Deeper details would make it
:: a tutorial for implementing producers and readers of ISO 9660.
::
:: The article about Joliet is a stub with a link that leads to
:: a russian language site. I will think about how to connect it
:: to this article about ISO 9660.
:: [[User:Scdbackup|Scdbackup]] 03:55, 18 October 2013 (CDT)
````

<a id="w003413"></a>

## W003413 — User:Scdbackup

Bron-ID: W003413
Namespace: User (2)
Revisie: 26675 | Datum: 2021-11-13T19:24:01Z
Bron: https://wiki.osdev.org/User:Scdbackup
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AScdbackup&oldid=26675

````text
My name is Thomas Schmitt, <scdbackup@gmx.net>.

I am developer of free software for producing [[ISO 9660]] filesystems
and for operating [[Optical_Drive|optical drives]] with CD, DVD, or BD media.
Available as three libraries from
[http://libburnia-project.org/ Libburnia project]
or as all-in-one source tarball [http://www.gnu.org/software/xorriso/ GNU xorriso].

The former are intended for distro packagers, the latter is intended
for users who do not necessarily have sysadmin privileges.

My reason to get an account here was to re-furbish the articles
about [[ISO 9660]], [[El-Torito]], and [[Mkisofs|mkisofs compatible programs]].
Further i attached a new article [[Optical_Drive]] to article [[ATAPI]],
in order to encourage OS developers to care for CD drivers and burn programs.

[[User:Scdbackup|Scdbackup]] 20:22, 13 November 2021 (CEST)
````

<a id="w003414"></a>

## W003414 — Talk:Mkisofs

Bron-ID: W003414
Namespace: Talk (1)
Revisie: 15332 | Datum: 2013-10-20T10:07:54Z
Bron: https://wiki.osdev.org/Talk:Mkisofs
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMkisofs&oldid=15332

````text
== Proposal for a Re-Write ==
I am the developer of xorriso, a competitor to mkisofs which is compatible
for the purposes discussed by OSDev.org.

Meanwhile i have refurbished [[ISO 9660]] and a pending proposal for [[El-Torito]]
on my user page [[User:Scdbackup]]. Since that page is occupied, i post my
proposal for [[Mkisofs]] here. 

[[User:Scdbackup|Scdbackup]] 07:50, 19 October 2013 (CDT) (actually done one day earlier)

: The proposal is now implement on the wiki page, and thus removed from here.
: [[User:Scdbackup|Scdbackup]] 05:07, 20 October 2013 (CDT)
````

<a id="w003415"></a>

## W003415 — Rust

Bron-ID: W003415
Namespace: Main (0)
Revisie: 30713 | Datum: 2026-06-10T18:12:11Z
Bron: https://wiki.osdev.org/Rust
Revisie-URL: https://wiki.osdev.org/index.php?title=Rust&oldid=30713

````text
'''Rust''' is a systems language sponsored by Mozilla, focused on three things: safety, speed, and concurrency. It accomplishes many of these goals through strong compile-time checks, allowing for very little overhead at runtime. Performance is comparable to C or C++, while being free of many of the problems caused by things like dangling pointers, buffer overflows, and iterator invalidation.

While Rust is very much a "curly-brace" language, it also takes many cues from ML. Almost everything is an expression, and there is a very strong type system including sum types and powerful generics.

Rust provides a mechanism to subvert its safety checks when necessary, by adding an 'unsafe' annotation to portions of your code. This allows you full access to raw pointers, while controlling exactly how they are used. Often, Rust code that uses unsafe will do so as an implementation detail, allowing for them to expose a safe interface.

== Operating System Development ==

Rust has a comparable amount of runtime to C and C++, and has set up its standard library to be amenable towards OS development. Specifically, the standard library (std) consists of three smaller parts: '''std''', '''core''', and '''alloc'''. '''core''' consists of the primitive, platform-independent components of the standard library. It doesn't include things like allocation, threading, and other higher-level features. '''alloc''' is used for managing heap allocations, smart pointers, and simple data collections. '''std''' builds on top of '''core''' and '''alloc''' to add high-level, OS-dependent features, such as files and processes. Using only '''core''' instead of std is as easy as adding the '''#![no_std]''' annotation to your '''lib.rs''' or '''main.rs''' file. To use '''alloc''' as well as '''core''' in a '''no_std''' environment, you add the statement '''extern crate alloc;''' in your '''lib.rs''' or '''main.rs''' file as well.

Every Rust compiler is a cross-compiler (utilizing LLVM behind the scenes), which makes setup easier as you don't have to compile anything extra.

== Active Projects ==

* [https://github.com/Andy-Python-Programmer/aero A mature POSIX OS]
* [https://os.phil-opp.com/ "Writing an OS in Rust"]
* [https://osblog.stephenmarz.com/ Tutorial: RISC-V OS using Rust]
* [https://github.com/thepowersgang/rust-barebones-kernel Meaty Bare-Bones]
* [https://github.com/skyzh/core-os-riscv core-os-riscv]
** An xv6-like operating system
** RISC-V based with multi-core support
* [https://github.com/thepowersgang/rust_os Tifflin]
** Multiboot image for x86_64
** has VFS and userland
* [http://www.redox-os.org Redox]
** One of the largest and most active Rust OS projects
* [https://www.tockos.org Tock]
** optimized for embedded devices
** isolates drivers and applications
* [http://intermezzos.github.io/ intermezzOS]
** Small kernel + book, specifically for learning OSDev through Rust
** on hiatus indefinitely
* [https://github.com/ryanra/RustOS Ryanra's RustOS]
** Uses libcore/collections/alloc
* [https://github.com/QuiltOS/QuiltOS QuiltOS]
** Forked from Ryanra's RustOS
** Main goal is demoing OS Dev libraries written in Rust.
* [https://source.that.world/source/rux Rux]
* [https://github.com/nebulet/nebulet Nebulet]
** Has an end goal of running [[WebAssembly]] in the kernel for a performance increase
* [https://github.com/theseus-os/Theseus Theseus]
** An experimental OS written in Rust
* [https://github.com/vinc/moros MOROS]
** A simple hobby OS with a text-based user interface
* [https://github.com/nanvix Nanvix]
** A microkernel-based research operating system written in Rust.

== Past Projects ==

Rust went through a long and public development process and used to have a significantly larger runtime. There were lots of experiments pre-1.0, thus these projects are of historical interest, but aren't guaranteed to run or even build.

* [https://github.com/charliesome/rustboot RustBoot] 
* [https://github.com/LeoTestard/Quasar Quasar]
* [https://github.com/miselin/rustic Rustic]
* [https://github.com/wbthomason/ironkernel IronKernel]
* [https://github.com/thiagopnts/rustico Rustico]
* [https://github.com/Arcterus/kRnel kRnl]
* [https://github.com/mahrz/rv6 rv6]
* [https://github.com/hackndev/zinc Zinc]


== Libraries ==

* [https://github.com/gz/rust-x86 libx86]: Library to program x86 hardware.
* [https://github.com/rust-osdev/x86_64 x86_64]: Library to program x86_64 hardware.
* [https://github.com/rust-osdev/bootloader bootloader]: A rust bootloader.
* <strike>[https://github.com/Tobba/libcpu libcpu]: Library to program CPUs.</strike> Now absorbed into libx86.
* [https://github.com/gz/rust-slabmalloc slabmalloc]: Low-level memory allocator for liballoc.
* [https://github.com/gz/rust-multiboot multiboot]: Library to read multiboot layout.
* [https://github.com/mvdnes/spin-rs spin]: Various synchronization primitives implemented with spinning. Closely mimics `std::sync`'s interface. 
* [https://github.com/nathan7/libfringe fringe]: Context switching agnostic to how stacks are allocated.

== See Also ==

* [http://www.rust-lang.org/ Rust Website]
* [https://github.com/rust-lang/rust-wiki-backup/blob/master/Operating-system-development.md Operating Systems Development in Rust]
* [http://scialex.github.io/reenix.pdf Reenix: Implementing a Unix-like OS in Rust]
* [http://jvns.ca/blog/2014/03/12/the-rust-os-story/ Writing an OS in Rust in tiny steps (Steps 1-5)]
* [[Rust Bare Bones]]
* [http://os.phil-opp.com/ Blog Series: Writing an OS in Rust]
* [http://www.randomhacks.net/2015/11/11/bare-metal-rust-custom-target-kernel-space/ Retarget your compiler so interrupts are not evil]
* [[BOOTBOOT]] loader has an example 64 bit higher half kernel in Rust
* [https://github.com/rust-osdev rust-osdev on GitHub]

[[Category:Languages]]
[[Category:Rust]]
````

<a id="w003416"></a>

## W003416 — Optical Drive

Bron-ID: W003416
Namespace: Main (0)
Revisie: 21487 | Datum: 2017-09-06T18:02:13Z
Bron: https://wiki.osdev.org/Optical_Drive
Revisie-URL: https://wiki.osdev.org/index.php?title=Optical_Drive&oldid=21487

````text
This article is about the SCSI commands for optical drives.
They are governed mainly by SCSI part MMC, but also by SPC,
and by the 0x1B START/STOP UNIT command out of SBC.

Regrettably these specs are not officially available for free.
One has to pay 30 USD per part as PDF.
Many old links, which originally pointed to free-of-charge
drafts of the specs, now lead to the new info PDFs of
[http://www.t10.org/ T10],
which just tell where to buy the specs.

==Transport==
The communication between computer and optical drive can be
done by various types of controllers and cabling. Like [[ATAPI]],
[[SATA]], or [[USB]]. This task of transport is not in the
scope of this article.

Many operating systems offer some kind of generic SCSI driver
interface which abtracts the various transports to a single 
transaction API. These APIs are also available in userspace.
Linux has ioctl(SG_IO), FreeBSD has CAM, Solaris has uscsi, ...

Your operating system should have such a thing too. If not for
internal purposes, then for enabling the porting of burn programs,
which usually run in userspace.

==SCSI Transactions==
A driver of optical drives will have to compose SCSI commands
(Control Data Block, CDB) and provide payload data buffers.
It will then hand them over to the transport drivers which
bring them to the optical drive.

The drive will return a status indication and payload data,
if appropriate. The transport driver will interpret the status
and return it to the caller together with the payload data.
It is nice of the transport driver if it also fetches diagnostic
Sense Data if the status indicates their availability.
This is to be done by command 0x03 REQUEST SENSE.

==Readable Disc Content==
Optical media and drives are structured in sessions and tracks.
The readable entities are called Logical Track. They are contiguous
strings of blocks. Some media types can bear several sessions with
several tracks each, others bear only one session with one track.

The MMC specs mention with DVD and BD only block size 2048.
These media can be read by the same SCSI commands as hard disks,
e.g. by 0x28 READ(10).

The block sizes and formats on CD can be quite strange. Data CDs
(CD-ROM) bear block size 2048 and can be read by the same SCSI
commands as hard disks. Audio CDs (CD-DA) bear block size 2352
and can only be read by special SCSI commands like 0xBE READ CD.
Further there are other CD block formats defined in MMC.

The track content of data CD, DVD, or BD is not restricted to
particular filesystem formats. It may well be raw data blocks
or tape archives.

Nevertheless, most data CD bear an [[ISO 9660]] filesystem, often
with [[ISO 9660#Rock_Ridge_and_Joliet|Rock Ridge and Joliet]]
extensions.
DVD and BD for home entertainment video players bear [[UDF]]
filesystems. For data exchange they often bear [[ISO 9660]].

DVD, BD, and data CD may be prepared as boot medium by
[[ISO 9660]] with additional [[El-Torito]] data.

===Getting an Overview of the Medium===
The type of optical media can be deduced from the Current
Profile as reported by command 0x46 GET CONFIGURATION.
Actually this tells not the media type but the feature set
of the medium. In a read-only drive everything might be
reported as -ROM media profile. Writer drives are more
accurate with their reports.

The medium can be either Blank, Appendable, Closed, or Overwritable.
This can be inquired by command 0x51 READ DISC INFORMATION as
"Empty Disc", "Incomplete Disc", "Finalized Disc", "Others".

'''Blank''' means that the medium contains no readable data and may
be written. Some media types need to be formatted first.

'''Appendable''' means that the medium contains readable data, but
may also be written in yet unreadable blocks.

'''Closed''' means that there are readable data and no more can be
written.

'''Overwritable''' is the state of formatted re-usable media.
Data can be read and written like with a hard disk.
Formatted DVD-RW demand write transactions to be aligned
to multiples of 16 blocks.

The first three states are assumed by '''sequential media''':
CD-R, CD-RW, DVD-R, unformatted DVD-RW, DVD+R, BD-R.
Written CD-RW and DVD-RW can be brought back to state Blank.

CD-ROM, DVD-ROM, BD-ROM are always in state Closed.

The family of '''overwritable media''' may be in state Blank
when sold: DVD+RW, DVD-RW, DVD-RAM, BD-RE. In this case it is
necessary to format them to Overwritable, before write and
read transactions be performed.
DVD+RW must be entirely written for once to guarantee that
formatting has reached all block addresses.

Sequential media in states Appendable and Closed can bear more
than one session with more than one track each.
The number of sessions and tracks can be inquired by
0x51 READ DISC INFORMATION, Data Type 000b.
On appendable media, the free area is counted as last track
of the last session. So in this case the number of readable
sessions and tracks is one less.
Track numbering usually starts at 1, but at least on CD
the burn program can choose a higher start number.

Knowing the number range of readable tracks, their start address
and size can be inquired by command 52h READ TRACK INFORMATION.
This also tells the session number to which the track belongs.

There is another command 0x43 READ TOC/PMA/ATIP which gives
complete information only about CD media. DVD and BD are at
best represented by coarse fabricated information.
So this command is mainly of interest if one wants to examine
CD-specific features like block formats or CD-TEXT.

Command 0x25 READ CAPACITY tells the number of the last 
readable data block. Between block 0 and this block there 
may be unwritten blocks though, which are located outside 
any track.

==Random Access Operation of DVD-RAM, DVD+RW, DVD-RW, BD-RE==
Writing to sequential media types can demand quite tricky
preparations. Therefore this is usually done by burn programs
which live in userspace and use the SCSI transaction API to
operate the drive.

But overwritable thoroughly formatted media are similar enough
to hard disks so that they can easily be presented as block
devices. Just use commands like 0xAA WRITE(12) or 0x28 READ(10).

Formatting is done by command 0x04 FORMAT UNIT. But it is
not trivial with DVD+RW and DVD-RW. So it seems wise to
format those media by help of a burn program like
[http://www.gnu.org/software/xorriso/ xorriso]
or a formatter program like
[http://fy.chalmers.se/~appro/linux/DVD+RW/tools/ dvd+rw-format out of dvd+rw-tools].
After formatting it is wise to let the burn program write
the medium up to the last block and to check whether
they all are readable afterwards.
In case of dvd+rw-format the burn program would be growisofs.

== See Also ==
=== Articles ===
* [[ATAPI]] - Transport protocol for SCSI over [[ATA]]
* [[ISO 9660]] - Technical properties of ISO 9660 filesystems
* [[UDF]] - The filesystem used for video DVD and BD

=== External Links ===
* [http://www.t10.org/ T10 home page] - Where the SCSI specs are made
* [http://www.techstreet.com/incits/? INCITS Online Standards Store] - Where to buy SCSI specs MMC-6, SPC-3, and SBC-2
* [https://dev.lovelyhq.com/libburnia/libburn/raw/HEAD/doc/cookbook.txt libburn cookbook] - Detailed description how libburn operates the various media types. You need SCSI specs for understanding this. The links to free drafts are semi-dead. If link is dead: see libburn-*/doc/cookbook.txt in any libburn tarball.

[[Category:Storage]]
````

<a id="w003417"></a>

## W003417 — Posting Checklist

Bron-ID: W003417
Namespace: Main (0)
Revisie: 30408 | Datum: 2026-03-24T22:15:47Z
Bron: https://wiki.osdev.org/Posting_Checklist
Revisie-URL: https://wiki.osdev.org/index.php?title=Posting_Checklist&oldid=30408

````text
Many questions on the forums are not answerable straight away due to a wide variety of reasons. It is in part a nature of the subject - remember that in OS development everything depends on everything and small errors in the foundations can mean huge untraceable problems in later stages. A lot of the queries are however quite preventable.

== Trivialities that get bugs fixed ==
Things that give apparently random results are typically the result of one of the following:
# Your build calls "gcc" and/or "ld". That's not [[GCC Cross-Compiler|how you should use a compiler]]. Sadly, this practice also breaks several tutorials in the wild.
# You are using outdated versions of [[Bochs]] or [[QEMU]]. In particular, do not use the versions bundled with your Linux distribution.
# You forgot to check the VM log. Especially in the case of Bochs, the exact error cause is often printed in there.

== When you ask ==
# Currently, one out of five new people blatantly violates the [http://forum.osdev.org/viewtopic.php?f=1&t=16944 forum rules] on the first post, and for roughly half the newcomers it is questionable if they have actually read it instead of acting on autopilot. Don't waste our time, or more importantly your own time: it contains all sorts of pointers, including a few that might answer the question already.
# Don't guess ''anything''. Stick to the facts. If you have pending questions that prevents you from understanding what you are doing, consider making the initial problem the background story to your lack of understanding.
# Include the name and version of the emulator(s) and virtual machine(s) you are using. Mention your architecture (protected mode, long mode, raspberry etc), and whatever bootloader you are using.
# Do mention any noticeable differences between different VMs and real hardware. Your OS should work the same everywhere and these things are important cues.
# If you run your code in a VM, show any log lines produced by your code.
# Reduce the code to the smallest set that exhibits the bug, and post it. Don't waste a day for a query from us to "post your code" when you could have done it straight away. Translate where applicable.

== After replies ==
# Don't blindly follow suggestions. Learn what they are meant to do.
# There might be multiple issues with your code. '''Fix them all''', and ask for elaboration on all comments you don't understand. Sometimes you need to fix more than one thing to get it working, and sometimes one thing might not contribute right now, but it might be an issue later. This also has another effect, caused by the fact that many software developers are inherently trained to stop reading after the first bug they encounter. If your code has several (possibly minor) issues, you'll only get the first few issues, and not the ones that matter the most at that time. until you do follow up and explain what you did and that it still doesn't work. 
# There are a lot of people trying to help, but you'll find that some people post arbitrary suggestions based on their own lack of understanding. Again, if you don't understand why, you are better to question the suggestion for elaboration and understand what's really happening.
# Be courteous and write a brief summary of what you've learned.

[[Category:FAQ]]
````

<a id="w003418"></a>

## W003418 — Talk:Libsupcxx

Bron-ID: W003418
Namespace: Talk (1)
Revisie: 15363 | Datum: 2013-11-04T15:52:20Z
Bron: https://wiki.osdev.org/Talk:Libsupcxx
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ALibsupcxx&oldid=15363

````text
== *-elf and --disable-hosted-libstdcxx are not compatible ==

After examining libsupc++ and attempting to build a freestanding libsupc++ for i586-elf using gcc 4.8.2, I must unfortunately announce that I have failed. It would appear that i586-elf is indeed [i]too[/i] freestanding for libsupc++, or rather the libstdc++-v3 build system. The configure script for libstdc++-v3 scans through headers such as <stdio.h> and searches for dlopen (and other things) in various system libraries, however, the configure script correctly recognizes it should not do such tests and simply decides to error out upon encountering this paradox. I was unable to work-around these issues, indeed the approach suggested by the older libstdc++ article appears to have bitrotted away. The --disable-hosted-libstdcxx configure option that was supposed to save the day only appears to work on actually hosted targets. I successfully used this --disable-hosted-libstdcxx option when targeting my native x86_64-linux-gnu platform and it actually only installed a libsupc++.a into the compiler prefix and only a small subset of freestanding C++ headers - so there's hope. After examining the libsupc++.a for x86_64-linux-gnu using nm(1), I saw a few references to libc (strcmp, malloc, free), a bunch of references to libgcc and a few problematic pthread_ calls. As a work-around you can probably use a i386 Linux libsupc++.a library in your kernel, but the pthread symbol references suggest you might be pulling in some bad Linux stuff.

Anyways, my recommendation is that you don't use libsupc++ and the provided C++ headers in a kernel built with i586-elf. You can probably get it working if you add a working libc into your system root, and you use an OS-Specific Toolchain that has a libstdc++-v3 port, and if you link your libc (a special kernel version) into your kernel. The real solution here is to get the GCC developers (or better yet - do it yourself) to fix --disable-hosted-libstdcxx not working properly on *-elf platforms. I might just look into this if I get some time, or perhaps just send a bug report to the GCC team.

I took the liberty of adding the disputed statement to this article because gcc 4.2 is ancient and the information is not reliable. Ideally, if we can get this working for i586-elf, we should merge this information into [[GCC Cross-Compiler]]. --[[User:Sortie|Sortie]] 09:52, 4 November 2013 (CST)
````

<a id="w003419"></a>

## W003419 — Talk:Win95 DOS Call

Bron-ID: W003419
Namespace: Talk (1)
Revisie: 15399 | Datum: 2013-11-11T07:13:08Z
Bron: https://wiki.osdev.org/Talk:Win95_DOS_Call
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AWin95_DOS_Call&oldid=15399

````text
I don't like the last sentence of the introduction ''"This might make Win95 a better choice for an OS development platform than Win98."'' for two reasons:
* It's ''almost'' recommending using Win95
* It's using Windows 98 as a sort of benchmark
Is there a reason this is worded this way?
-- [[User:Phillid|Phillid]]
````

<a id="w003420"></a>

## W003420 — Target Triplet

Bron-ID: W003420
Namespace: Main (0)
Revisie: 30316 | Datum: 2026-03-17T19:01:58Z
Bron: https://wiki.osdev.org/Target_Triplet
Revisie-URL: https://wiki.osdev.org/index.php?title=Target_Triplet&oldid=30316

````text
Target Triplets describe a platform on which code runs and are a core concept in the GNU build system. They contain three fields: the name of the CPU family/model, the vendor, and the operating system name. You can view the unambiguous target triplet for your current system by running:

<syntaxhighlight lang="bash">
gcc -dumpmachine
</syntaxhighlight>

== Structure ==

Target triplets have this simple structure:

 machine-vendor-operatingsystem

For instance, a FreeBSD system could be written as:

 x86_64-unknown-freebsd

Notice how the vendor field is mostly irrelevant and is usually '<tt>pc</tt>' for 32-bit x86 systems or '<tt>unknown</tt>' or '<tt>none</tt>' for other systems. The simple three-field target triplet we have seen so far is unambiguous and easy to parse. However, since the vendor field is mostly unused, the GNU build system allows you to leave out the vendor field; the build system will automatically insert a default vendor part when it disambiguates your target triplet. For instance, this allows you to type:

 x86_64-freebsd

The build system will then automatically deduce that the vendor is the default (<tt>unknown</tt>) if it wishes to know the unambiguous target triplet. Note that parsing target triplets are a bit more tricky, as sometimes the operating system field can be two fields:

 x86_64-unknown-linux-gnu

This gets a bit worse since the vendor field can be left out:

 x86_64-linux-gnu

This is most definitely ambiguous. Most autoconf-based packages ship with a huge shell script called <tt>config.sub</tt> whose function is to disambiguate target triplet using a long list of known CPUs and known operating systems.

== Rationale ==

Target triplets are intended to be systematic unambiguous platform names (well, after disambiguation). They lets build systems understand exactly which system the code will run on and allows enabling platform-specific features automatically. In any compilation setting, there are usually three platforms involved (which might be the same three ones):

* '''Build Platform''': This is the platform on which the compilation tools are executed.
* '''Host Platform''': This is the platform on which the code will eventually run.
* '''Target Platform''': If this is a compiler, this is the platform that the compiler will generate code for.

This means that up to three differently-targeted compilers might be in play (if you are building a GCC on platform A, which will run on platform B, which produces executables for platform C). This problem is solved by simply prefixing the compilation tools with the target triplet. When you build a cross-compiler, the installed executables will be prefixed with the specified target triplet:

 i686-elf-gcc

This prevents the wrong compiler from being used (and prevents things from the build machine leaking onto the target machine) if build systems are careful to prefix all compilation tools with the target prefix.

== Target Triplets for Operating Systems Development ==

For instance, if you develop your own operating system and [[OS Specific Toolchain|modify GCC to add a new target triplet]], yours could be:

 x86_64-myos

However, when starting out you simply wish to use the generic targets that are well-suited for early OS development:

 i686-elf
 x86_64-elf
 arm-none-eabi
 aarch64-none-elf
 riscv64-none-elf

These are bare targets meant for freestanding programs (bootloaders and kernels) that don't have a user-space.

[[Category:FAQ]]
[[Category:OS_Development]]
[[Category:OS_theory]]
````

<a id="w003421"></a>

## W003421 — User:Griwes/HPET

Bron-ID: W003421
Namespace: User (2)
Revisie: 15430 | Datum: 2013-11-24T13:32:34Z
Bron: https://wiki.osdev.org/User:Griwes/HPET
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGriwes%2FHPET&oldid=15430
Doorverwijzing: HPET

````text
#REDIRECT [[HPET]]
````

<a id="w003422"></a>

## W003422 — User:Max

Bron-ID: W003422
Namespace: User (2)
Revisie: 27467 | Datum: 2022-10-11T19:02:11Z
Bron: https://wiki.osdev.org/User:Max
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMax&oldid=27467

````text
Hello! :-)

My name is Max. I'm a software developer from Germany and the maker of [[User:Max/Ghost_Kernel|Ghost OS]].

If you have questions, feel free to write me an [mailto:lokoxe@gmail.com e-mail].

Greets,
Max
````

<a id="w003423"></a>

## W003423 — User:CWood/Disk Image Management

Bron-ID: W003423
Namespace: User (2)
Revisie: 15448 | Datum: 2013-12-07T00:24:56Z
Bron: https://wiki.osdev.org/User:CWood/Disk_Image_Management
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACWood%2FDisk_Image_Management&oldid=15448

````text
Inevitably, in the early days of operating system development, you will use a virtual machine. Indeed, you may need to use several. Disk images are, thus, an inevitability, and the only viable option for using virtual machines (see below). This tutorial is meant more for people who are not so familiar with the terminal, and the Linux kernel internals (some of these will definitely work in BSD etc, others I have no idea, as I'm not familiar with other *NIXen)
== Comparison of Alternatives ==
=== USB Flash Drives ===
Many of these commands can be avoided, by using 'tricks' such as using a bootable USB drive as your disk image(s). But, this comes with its own set of issues, which are much more difficult to address. Namely:
* '''Loss of portability''' - If you plan on releasing your OS, it will entail more work for people to build your OS. This is because not everybody will have a USB flash drive present, and many will be unwilling to look for one.
* '''Loss of convenience''' - If you are developing away from your own dev box, it is not enough to have the source files, and prerequisite toolchain any more; you will need to remember to have the correct flash drive present. This gets even more unwieldy if you use the flash drive for other things as well, especially if you take it with you to places other than your dev box(en).
* '''Lack of size''' - Many flash drives, especially the cheaper ones, are limited in capacity. While it may be unlikely, it is entirely possible that your OS, especially as it matures, will outgrow your chosen flash drive, meaning a new one, with more capacity, must be obtained, and set up.
* '''Difficulties with version control''' - As stated above, it is possible you may need more than one bootable image. There are two ways to go about this with USB drives:
** Multiple drives. This '''will''' get unwieldy. Fast.
** Multiple kernels on one drive. This requires use of a multiboot bootloader, or similar. However, if one kernel runs rogue and starts modifying things on the disk, there is nothing to prevent this. Furthermore, file management will get messy as more versions are added.
** Multiple partitions on one drive. This still requires a multiboot loader, and still has write protection issues. However, file management is simplified, at the expense of (possibly) having to store duplicate data.

That is not to say, however, that bootable USB flash memory is not without its benefits. 
* When you get to the point wherein you are ready to test on real hardware, there is no work involved in setting up a hard disk/floppy disk for boot. Simply boot straight from your development drive of choice. This is also convenient, as it is easier to test on a multitude of hardware using this method.

== Issues with Disk Images ==
=== Partition Management ===
==== Mounting Partitions in a Clean, Portable Way ====
One of the main issues with disk images, is partition management. If you wish to have a partitioned image (which is most people, especially if HDD support is needed), you will run into issues at some point, the main one being mounting in the host OS.

If your host OS is a *NIX, you may find yourself doing the following (or something similar) to mount a particular partition on an image.
 losetup /dev/loop0 disk.img -o 1048576
Clearly, this uses a magic number, that is completely devoid of context. For reference, the magic number refers to the offset, in bytes, to the start of the partition. There is, however, a much better way.

If you have used fdisk to edit disk images, you will have seen it refer to things like /dev/loop0p1. A common source of confusion is why fdisk refers to this, but the loop module doesn't implement them. Well, as it turns out, it does, at least in Linux. It's probably just turned off in your kernel, however. To activate this behaviour, run:
 sudo rmmod loop
 sudo modprobe loop max_part=63
 losetup /dev/loop0 disk.img
 mount /dev/loop0p1 /mnt

No magic numbers, and a much neater command history. This is much easier to remember, but the ramifications go much deeper than that. Portability is hugely improved by this, as the shell script/makefile is no longer dependent on the size of the partitions in your disk image. Indeed, by adding a parameter to the configuration of your makefile (through whatever means this is achieved in your OS), it can be made to be independent of partition order as well.

==== Finding the Correct Loopback Device ====
This one is fairly easy, and will be somewhat obvious to people who have used Bash for any length of time. But, its a time saver nonetheless.

Setting up your loopback device is easy, using:
 sudo losetup /dev/loop0 disk.img
However, this is not portable. If /dev/loop0 is being used by something else on the system, we cannot claim it for ourselves, and there is no guarantee that it is not being used by something else. Therefore, it is better to use:
 sudo losetup -f disk.img

However, after running this, we no longer know where disk.img is mounted. So, to find this, we can run:
 losetup -a | grep disk.img | cut -d: -f1

What this does is list all of the loopback devices, grep for our image, and chop out which specific device this belongs to. This can be put inside a mount command, like so (Makefile):
 mount $(shell losetup -a | grep disk.img | cut -d: -f1)p1 /mnt

(Shell script):
 mount `losetup -a | grep disk.img | cut -d: -f1`p1 /mnt

This will find our loopback device, and mount it to /mnt, so that we can perform file operations from the host OS. The ''p1'' is explained in the previous section, Mounting Partitions in a Clean, Portable Way.
````

<a id="w003424"></a>

## W003424 — IPC Data Copying methods

Bron-ID: W003424
Namespace: Main (0)
Revisie: 28094 | Datum: 2023-07-09T21:29:39Z
Bron: https://wiki.osdev.org/IPC_Data_Copying_methods
Revisie-URL: https://wiki.osdev.org/index.php?title=IPC_Data_Copying_methods&oldid=28094

````text
=Prologue=
IPC ([https://en.wikipedia.org/wiki/Inter-process_communication Inter Process Communication]) is a well known kernel-provided abstraction, which is core to almost any type of kernel design; whether multi-address space, single-address space, microkernel, monolithic kernel, or otherwise, a kernel will generally have to provide IPC data transfer mechanisms as long as it supports multiple processes.

Since various higher IPC abstractions exist, and many APIs and frameworks have been introduced into the body of known "IPC mechanisms" throughout history it is generally not difficult to obtain information on IPC APIs in use contemporarily. However, the distinction between an IPC framework, and the underlying copying and data transfer method may not be well understood even by someone who has read on the topic of IPC. This article seeks to go one layer below IPC, and examine the four general strategies which will be available to a kernel for the transfer of data across process boundaries.

Caveat lector: not every methodology can be applied to every design. Additionally, this article does not take asynchronous design into consideration; persons wishing to implement asynchronous designs should bear that in mind, substituting the appropriate messaging infrastructure where necessary as they read.

=Multi-copy, Single-copy, and Zero-copy=

The number of data copies required for any IPC abstraction or implementation is reliant on the underlying data transfer technique being used. For the techniques described in this article, the classifications are as follows:
* Multi-copy: Kernel-relayed buffers (hereafter: "Kernel-buffered").
* Single-copy: Map-shared and copy (hereafter: "map and copy").
* Zero-copy: Map-shared and read (hereafter: "map and read"); Page-table displacement (hereafter: "pagetable-displacement").

==Kernel-buffered data transfer==

In this approach, the data sender process initiates an IPC transaction, and gives the kernel the address of a buffer which it will wishes to have transferred to another process. The kernel copies that buffer into its own address space region and sends a message or signal of some sort to the target process, and returns to the sender.

When the target process "picks up" the signal or message from the kernel, it will generally allocate memory to contain the data from the sending process, and then ask the kernel to fill the data into the allocated memory.

This approach does not require the kernel to block the sending process, because the changes to the sending process' copy of the data will not affect the fidelity of the data that the target process receives. The sending process does not strictly need to be informed when the target process is given a copy of the data; but a particular implementation may choose to give such a notification for a "handshake" roundtrip.

As a keen reader can probably see, there were two copies involved in this data transfer, along with ''potentially'' two address space switches (a switch into the target process' address space, and then back to the sender's address space), and any number of privilege barrier crossings (userspace to kernelspace, and back) which are mandated by the particular IPC API implementation that uses the technique.

==Map and copy==

This data transfer method makes use of shared memory mapping to reduce the amount of copying done. It holds the following advantages over kernel-buffered data transfer:
* Large data transfers do not exhaust the kernel virtual address space.
* Data transfer is done directly between the sender and the target process, without needing an intermediate copy into a buffer in the kernel.
* For large data transfers, the reduction from two copies to one copy, and the elimination of the need to allocate a kernel buffer brings a performance benefit.

In brief, the sender process asks the kernel to send a designated packet of data to a target process. The kernel must at this point put the sender to sleep, and then post a message or signal to the target process.

The sender process must be put to sleep because the kernel does not create a copy of the data to be sent; therefore if the sender process is not forced to sleep, it could modify the data before the target process "picks it up". More elaborate synchronization schemes can be constructed around this method to ensure better serialization of access to the data buffer. Consider for example, the scenario where the sender process is multithreaded, and another thread within the sender process modifies the data buffer before the target process is able to create a copy of it. Clearly, sleeping the sender thread is not adequate. And putting '''all''' of the threads within the sending process to sleep just to transfer one buffer of data is also obviously not justifiable for most designs.

Continuing: Eventually, the target process receives the message or signal from the kernel that data is available to be "picked up", and it allocates enough memory to hold the data. The target process will then ask the kernel to copy the data into the memory area it has allocated. The kernel will at this point, create a temporary shared mapping between the sender process (which is asleep) and the target process, and use that temporary shared mapping to access the data from within the target's address space to facilitate a fast, single copy into the allocated receipt memory area.

After the copy, the kernel will tear the shared memory mapping down and wake up the sending process.

==Map and read==

This technique is usable only if the sender can guarantee that it does not need to write to the data it intends to send, until the target process has read it and finished using it.

The advantage of this approach over "map and copy" is that this approach does not require copying of data; it simply maps the data from the sender into the target process' address space, and lets the target process use the shared memory '''directly'''. The steps are essentially the same as "map and copy":

First, the sender prepares a buffer of read-only data which it intends to send to a target. The sender asks the kernel to transfer this data to its target using some IPC abstraction. The kernel will then put the sender to sleep, and post a message or signal of some kind to the target process.

The target process eventually picks up the message or signal which tells it that data is available from some other process. The target process asks the kernel to set up the shared memory mapping to the data from the sender, and the kernel does so. The target process reads the data and acts on it in one manner or another, preserving it and treating it as read-only.

When the target process is done working with the data, the target process must tell the kernel so; the kernel will then unmap the shared mapping and wake up the sender process.

==Pagetable displacement==

This is an IPC technique which, rather than copy data, moves data from one process to another; or more precisely, it moves data from the scope of access of one process to another, by unmapping the data from the view of the sender process, and mapping it into the address space of the target process.

No copying is done, and only the page tables of the two processes are modified, essentially making this method as fast as a few page-table modifications, plus some MMU translation flushing.

The data is not copied, but moved. Naturally, the implications should be understood by the sender before the method is employed.

=Epilogue=

In synopsis, most IPC data transfer techniques will employ one of the above in some way or another. There are variations, and further synchronization considerations to be made. Each technique can be implemented as a synchronous operation, or an asynchronous operation; the implementation details only serve to add or diminish from the flexibility and throughput characteristics of each method.

Naturally, while this article postulates that for example, the map-and-copy and map-and-read techniques disallow the sender to modify the data buffer while it is "in transit", it is not inconceivable that one or more kernel designs may prefer to '''allow''' such behaviour, and see such behaviour as desirable.

[[Category:IPC]]
````

<a id="w003425"></a>

## W003425 — Talk:IPC Data Copying methods

Bron-ID: W003425
Namespace: Talk (1)
Revisie: 26401 | Datum: 2021-09-15T17:43:26Z
Bron: https://wiki.osdev.org/Talk:IPC_Data_Copying_methods
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AIPC_Data_Copying_methods&oldid=26401

````text
Please review and post comments. In N days (where N is defined at my own discretion), I'll move this out of my namespace and into the public namespace if I believe it is in decent form. Feel free to critique as deemed fit.
[[User:Gravaera|Gravaera]] 19:52, 12 December 2013 (CST)

== ASID/PCID ==

Shouldn't stuff about ASID/PCID be added here?
````

<a id="w003426"></a>

## W003426 — User:Gravaera/IPC Data Copying methods

Bron-ID: W003426
Namespace: User (2)
Revisie: 15458 | Datum: 2013-12-14T23:11:23Z
Bron: https://wiki.osdev.org/User:Gravaera/IPC_Data_Copying_methods
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FIPC_Data_Copying_methods&oldid=15458
Doorverwijzing: IPC Data Copying methods

````text
#REDIRECT [[IPC Data Copying methods]]
````

<a id="w003427"></a>

## W003427 — User talk:Gravaera/IPC Data Copying methods

Bron-ID: W003427
Namespace: User talk (3)
Revisie: 15460 | Datum: 2013-12-14T23:11:23Z
Bron: https://wiki.osdev.org/User_talk:Gravaera/IPC_Data_Copying_methods
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AGravaera%2FIPC_Data_Copying_methods&oldid=15460
Doorverwijzing: Talk:IPC Data Copying methods

````text
#REDIRECT [[Talk:IPC Data Copying methods]]
````

<a id="w003428"></a>

## W003428 — Rust Bare Bones

Bron-ID: W003428
Namespace: Main (0)
Revisie: 30342 | Datum: 2026-03-17T21:03:52Z
Bron: https://wiki.osdev.org/Rust_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Rust_Bare_Bones&oldid=30342

````text
{{In Progress}}
{{Stub}}
{{Rating|2}}
{{BeginnersWarning}}
{{Template:Kernel designs}}

== Cross Compiling ==
Just like when building a OS in another language, such as C, you need to build a cross compiler. In contrast to GCC, this is much easier with Rust and its surrounding tools. Here's a list of the tools you'll need:

* Rustup for managing your toolchain (https://rustup.rs/).
* Rustc (installable via rustup)
* Cargo (installable via rustup)

The cross compiler and core library will automatically be fetched, built, and installed when you build the project ''the right way''.

== Creating a target ==
In order to specify how to build the cross compiler and what features to include, you have to make a ''target.json'' file. The specification can be found here: https://book.avr-rust.org/005.1-the-target-specification-json-file.html.

An example might be as follows (for an x86_64 platform):
<syntaxhighlight lang="json">
{
  "llvm-target": "x86_64-unknown-none",
  "data-layout": "e-m:e-i64:64-f80:128-n8:16:32:64-S128",
  "arch": "x86_64",
  "target-endian": "little",
  "target-pointer-width": "64",
  "target-c-int-width": "32",
  "os": "none",
  "executables": true,
  "linker-flavor": "ld.lld",
  "linker": "rust-lld",
  "panic-strategy": "abort",
  "disable-redzone": true,
  "features": "-mmx,-sse,+soft-float",
  "dynamic-linking": false,
  "relocation-model": "pic",
  "code-model": "kernel",
  "exe-suffix": ".elf",
  "has-rpath": false,
  "no-default-libraries": true,
  "position-independent-executables": false,
  "pre-link-args": {
    "ld.lld": ["--script=<your linker script>"]
  }
}
</syntaxhighlight>

Please note that you also need to provide a linker script. This is more or less analogous to what you could provide for a kernel built in a more conventional language like C. This is pretty dependent on which executable format and which bootloader you use (among other things), so this isn't listed here.

== A minimal executable ==
This assumes you have at least some background with Rust (e.g. you've built some command line tools or crates before). It does not assume you know about the internals of the Rust language. It also assumes you will be using Cargo for your build system (you can call it from a Makefile if you need to do other things).

Please note that this depends on which bootloader you are using. For the purposes of this example, we assume you are working with Limine, over the Limine protocol, with an ELF binary. This will be different if you are using a different setup.

First, you must prepare your source. In general, rust tries to avoid command line flags in favour of in-source changes. As such, there is not Cargo or Rust flag to disable the standard library and add a custom panic handler. The below source takes care of that:

<syntaxhighlight lang="rust">
#![no_std]

#[no_mangle]
fn _start() {
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
</syntaxhighlight>

The above code tells the compiler to not include the Rust standard library (the core library is kept), not to mangle the _start symbol (Rust uses name mangling by default), and to use the panic() function as the panic handler. This approach by the Rust team generally keeps your rustc invocations simpler and easier to debug if anything goes wrong.

Another thing to keep in mind is that Rust uses stack unwinding by default, however, this is dependent on an underlying OS, which we obviously don't have (hey, you're here for a reason). To disable stack unwinding and just abort on panic (after the panic handler is called, of course), modify your Cargo.toml:

<syntaxhighlight lang="toml">
[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
</syntaxhighlight>

Run cargo build to build your kernel, and do whatever you wish with the resulting ELF binary (look at your bootloaders instructions on how to build a bootable image).

== Notes ==
* There aren't as many resources for using rust to build a kernel. Be aware of this if this is your first attempt.
* The rust libcore can easily be included in a kernel, it's analogous to C's freestanding headers
* Enable `--gc-sections` in lld to avoid having to define symbols you don't need (e.g. floating point math functions)
* Always use a target json file (Flexible Target Specifications)
* If you're building a UEFI app, disable function sections as well
** rustc puts every item in its own section by default. The PE format has a limit of 96 sections. So unless you disable function sections, your project will most likely fail to load.


== Examples ==
* https://github.com/thepowersgang/rust-barebones-kernel - A  framework written in rust 1.0 alpha, supporting x86 and x86-64, boots off multiboot.
* https://github.com/charliesome/rustboot - VERY old (rust 0.7), includes a bootsector.
* https://github.com/redox-os/redox
* http://os.phil-opp.com/ - New Rust OS Tutorial/Blog by Philipp Oppermann

[[Category: Bare bones tutorials]]
[[Category: Rust]]
````

<a id="w003429"></a>

## W003429 — ELF Tutorial

Bron-ID: W003429
Namespace: Main (0)
Revisie: 30303 | Datum: 2026-03-16T20:34:39Z
Bron: https://wiki.osdev.org/ELF_Tutorial
Revisie-URL: https://wiki.osdev.org/index.php?title=ELF_Tutorial&oldid=30303

````text
{{In_Progress}}
{{Rating|2}}{{File formats}}
This tutorial describes the steps to loading ELF files targeting the i386 (32-bit architecture, little-endian byte order). All code in the tutorial is in the form of C compatible C++ and strives to teach by example, by using simplified (and sometimes naive), neat, and functional snippets of code. It may later be expanded to cover other types of ELF files, or formats targeting other architectures or machine types.

==ELF Data Types==

<syntaxhighlight lang="cpp">
# include <stdint.h>

typedef uint16_t Elf32_Half;	// Unsigned half int
typedef uint32_t Elf32_Off;	    // Unsigned offset
typedef uint32_t Elf32_Addr;	// Unsigned address
typedef uint32_t Elf32_Word;	// Unsigned int
typedef int32_t  Elf32_Sword;	// Signed int
</syntaxhighlight>

The ELF file format is made to function on a number of different architectures, many of which support different data widths. For support across multiple machine types, the ELF format provides a set of guidelines for fixed width types that make up the layout of the section and data represented within object files. You may choose to name your types differently or use types defined in stdint.h directly, but they should conform to those shown above.

==The ELF Header==

The ELF file format has only one header with fixed placement: the ELF header, present at the beginning of every file. The format itself is extremely flexible as the positioning, size, and purpose of every header (save the ELF header) is described by another header in the file.

<syntaxhighlight lang="cpp">
# define ELF_NIDENT	16

typedef struct {
	uint8_t		e_ident[ELF_NIDENT];
	Elf32_Half	e_type;
	Elf32_Half	e_machine;
	Elf32_Word	e_version;
	Elf32_Addr	e_entry;
	Elf32_Off	e_phoff;
	Elf32_Off	e_shoff;
	Elf32_Word	e_flags;
	Elf32_Half	e_ehsize;
	Elf32_Half	e_phentsize;
	Elf32_Half	e_phnum;
	Elf32_Half	e_shentsize;
	Elf32_Half	e_shnum;
	Elf32_Half	e_shstrndx;
} Elf32_Ehdr;
</syntaxhighlight>

The ELF header is the first header in an ELF file and it provides important information about the file (such as the machine type, architecture and byte order, etc.) as well as a means of identifying and checking whether the file is valid. The ELF header also provides information about other sections in the file, since they can appear in any order, vary in size, or may be absent from the file altogether. Universal to all ELF files are the first 4 bytes (the magic number) which are used identify the file. When working with the file through the '''Elf32_Ehdr''' type defined above, these 4 bytes are accessible from indexes 0 - 3 of the field '''e_ident'''.

<syntaxhighlight lang="cpp">
enum Elf_Ident {
	EI_MAG0		= 0, // 0x7F
	EI_MAG1		= 1, // 'E'
	EI_MAG2		= 2, // 'L'
	EI_MAG3		= 3, // 'F'
	EI_CLASS	= 4, // Architecture (32/64)
	EI_DATA		= 5, // Byte Order
	EI_VERSION	= 6, // ELF Version
	EI_OSABI	= 7, // OS Specific
	EI_ABIVERSION	= 8, // OS Specific
	EI_PAD		= 9  // Padding
};

# define ELFMAG0	0x7F // e_ident[EI_MAG0]
# define ELFMAG1	'E'  // e_ident[EI_MAG1]
# define ELFMAG2	'L'  // e_ident[EI_MAG2]
# define ELFMAG3	'F'  // e_ident[EI_MAG3]

# define ELFDATA2LSB	(1)  // Little Endian
# define ELFCLASS32	(1)  // 32-bit Architecture
</syntaxhighlight>

The first field in the header consists of 16 bytes, many of which provide important information about the ELF file such as the intended architecture, byte order, and ABI information. Since this tutorial focuses on implementing an x86 compatible loader, only relevant value definitions have been included.

<syntaxhighlight lang="cpp">
enum Elf_Type {
	ET_NONE		= 0, // Unkown Type
	ET_REL		= 1, // Relocatable File
	ET_EXEC		= 2  // Executable File
};

# define EM_386		(3)  // x86 Machine Type
# define EV_CURRENT	(1)  // ELF Current Version
</syntaxhighlight>

The file header also provides information about the machine type and file type. Once again, only the relevant definitions have been included above.

===Checking the ELF Header===

Before an ELF file can be loaded, linked, relocated or otherwise processed, it's important to ensure that the machine trying to perform the aforementioned is able to do. This entails that the file is a valid ELF file targeting the local machine's architecture, byte order and CPU type, and that any operating system specific semantics are satisfied.

<syntaxhighlight lang="cpp">
bool elf_check_file(Elf32_Ehdr *hdr) {
	if(!hdr) return false;
	if(hdr->e_ident[EI_MAG0] != ELFMAG0) {
		ERROR("ELF Header EI_MAG0 incorrect.\n");
		return false;
	}
	if(hdr->e_ident[EI_MAG1] != ELFMAG1) {
		ERROR("ELF Header EI_MAG1 incorrect.\n");
		return false;
	}
	if(hdr->e_ident[EI_MAG2] != ELFMAG2) {
		ERROR("ELF Header EI_MAG2 incorrect.\n");
		return false;
	}
	if(hdr->e_ident[EI_MAG3] != ELFMAG3) {
		ERROR("ELF Header EI_MAG3 incorrect.\n");
		return false;
	}
	return true;
}
</syntaxhighlight>

Assuming that an ELF file has already been loaded into memory (either by the bootloader or otherwise), the first step to loading an ELF file is checking the ELF header for the magic number that should be present at the begining of the file. A minimal implementation of this could simply treat the image of file in memory as a string and do a comparison against a predefined string. In the example above, the comparision is done byte by byte through the ELF header type, and provides detailed feedback when the method encounters an error.

<syntaxhighlight lang="cpp">
bool elf_check_supported(Elf32_Ehdr *hdr) {
	if(!elf_check_file(hdr)) {
		ERROR("Invalid ELF File.\n");
		return false;
	}
	if(hdr->e_ident[EI_CLASS] != ELFCLASS32) {
		ERROR("Unsupported ELF File Class.\n");
		return false;
	}
	if(hdr->e_ident[EI_DATA] != ELFDATA2LSB) {
		ERROR("Unsupported ELF File byte order.\n");
		return false;
	}
	if(hdr->e_machine != EM_386) {
		ERROR("Unsupported ELF File target.\n");
		return false;
	}
	if(hdr->e_ident[EI_VERSION] != EV_CURRENT) {
		ERROR("Unsupported ELF File version.\n");
		return false;
	}
	if(hdr->e_type != ET_REL && hdr->e_type != ET_EXEC) {
		ERROR("Unsupported ELF File type.\n");
		return false;
	}
	return true;
}
</syntaxhighlight>

The next step to loading an ELF object is to check that the file in question is intended to run on the machine that has loaded it. Again, the ELF header provides the necessary information about the file's intended target. The code above assumes that you have implemented a function called '''elf_check_file'''() (or used the one provided above), and that the local machine is i386, little-endian and 32-bit. It also only allows for executable and relocatable files to be loaded, although this can be changed as necessary.

====Loading the ELF File====

<syntaxhighlight lang="cpp">
static inline void *elf_load_rel(Elf32_Ehdr *hdr) {
	int result;
	result = elf_load_stage1(hdr);
	if(result == ELF_RELOC_ERR) {
		ERROR("Unable to load ELF file.\n");
		return NULL;
	}
	result = elf_load_stage2(hdr);
	if(result == ELF_RELOC_ERR) {
		ERROR("Unable to load ELF file.\n");
		return NULL;
	}
	// TODO : Parse the program header (if present)
	return (void *)hdr->e_entry;
}

void *elf_load_file(void *file) {
	Elf32_Ehdr *hdr = (Elf32_Ehdr *)file;
	if(!elf_check_supported(hdr)) {
		ERROR("ELF File cannot be loaded.\n");
		return;
	}
	switch(hdr->e_type) {
		case ET_EXEC:
			// TODO : Implement
			return NULL;
		case ET_REL:
			return elf_load_rel(hdr);
	}
	return NULL;
}
</syntaxhighlight>

==The ELF Section Header==

The ELF format defines a lot of different types of section and their relevant headers, not all of which are present in every file, and there's no guarantee on which order they appear in. Thus, in order to parse and process these sections the format also defines section headers, which contain information such as section names, sizes, locations and other relevant information. The list of all the section headers in an ELF image is referred to as the section header table.

<syntaxhighlight lang="cpp">
typedef struct {
	Elf32_Word	sh_name;
	Elf32_Word	sh_type;
	Elf32_Word	sh_flags;
	Elf32_Addr	sh_addr;
	Elf32_Off	sh_offset;
	Elf32_Word	sh_size;
	Elf32_Word	sh_link;
	Elf32_Word	sh_info;
	Elf32_Word	sh_addralign;
	Elf32_Word	sh_entsize;
} Elf32_Shdr;
</syntaxhighlight>

The section header table contains a number of important fields, some of which have different meanings for different sections. Another point of interest is that the '''sh_name''' field does not point directly to a string, instead it gives the offset of a string in the section name string table (the index of the table itself is defined in the ELF header by the field '''e_shstrndx'''). Each header also defines the position of the actual section in the file image in the field '''sh_offset''', as an offset from the beginning of the file.

<syntaxhighlight lang="cpp">
# define SHN_UNDEF	(0x00) // Undefined/Not Present
# define SHN_ABS    0xFFF1 // Absolute symbol

enum ShT_Types {
	SHT_NULL	= 0,   // Null section
	SHT_PROGBITS	= 1,   // Program information
	SHT_SYMTAB	= 2,   // Symbol table
	SHT_STRTAB	= 3,   // String table
	SHT_RELA	= 4,   // Relocation (w/ addend)
	SHT_NOBITS	= 8,   // Not present in file
	SHT_REL		= 9,   // Relocation (no addend)
};

enum ShT_Attributes {
	SHF_WRITE	= 0x01, // Writable section
	SHF_ALLOC	= 0x02  // Exists in memory
};
</syntaxhighlight>

Above are a number of constants that are relevant to the tutorial (a good deal more exist). The enumeration '''ShT_Types''' defines a number of different types of sections, which correspond to values stored in the field '''sh_type''' in the section header. Similarly, '''ShT_Attributes''' corresponds to the field '''sh_flags''', but are bit flags rather than stand-alone values.

===Accessing Section Headers===

Getting access to the section header itself isn't very difficult: It's position in the file image is defined by '''e_shoff''' in the ELF header and the number of section headers is in turn defined by '''e_shnum'''. Notably, the first entry in the section header is a NULL entry; that is to say, fields in the header are 0. The section headers are continuous, so given a pointer to the first entry, subsequent entries can be accessed with simple pointer arithmetic or array operations.

<syntaxhighlight lang="cpp">
static inline Elf32_Shdr *elf_sheader(Elf32_Ehdr *hdr) {
	return (Elf32_Shdr *)((int)hdr + hdr->e_shoff);
}

static inline Elf32_Shdr *elf_section(Elf32_Ehdr *hdr, int idx) {
	return &elf_sheader(hdr)[idx];
}
</syntaxhighlight>

The two methods above provide convenient access to section headers on a by-index basis using the principles noted above, and they will be used frequently in the example code that follows.

===Section Names===

One notable procedure is accessing section names (since, as mentioned before, the header only provides an offset into the section name string table), which is also fairly simple. The whole operation can be broken down into a simple series of steps:

# Get the section header index for the string table from the ELF header (stored in '''e_shstrndx'''). Make sure to check the index against '''SHN_UNDEF''', as the table may not be present.
# Access the section header at the given index and find the table offset (stored in '''sh_offset''').
# Calculate the position of the string table in memory using the offset.
# Create a pointer to the name's offset into the string table.

An example of the process is shown in the two convenience methods below.

<syntaxhighlight lang="cpp">
static inline char *elf_str_table(Elf32_Ehdr *hdr) {
	if(hdr->e_shstrndx == SHN_UNDEF) return NULL;
	return (char *)hdr + elf_section(hdr, hdr->e_shstrndx)->sh_offset;
}

static inline char *elf_lookup_string(Elf32_Ehdr *hdr, int offset) {
	char *strtab = elf_str_table(hdr);
	if(strtab == NULL) return NULL;
	return strtab + offset;
}
</syntaxhighlight>

Note that before you attempt to access the name of a section, you should first check that the section has a name (the offset given by sh_name is not equal to '''SHN_UNDEF''').

==ELF Sections==

ELF object files can have a very large number of sections, however, it is important to note that only some sections need to be processed during program loading, and not all of them may exist within the object file itself (i.e. the BSS). This segment will describe a number of sections that should be processed during program loading (given they are present).

===The Symbol Table===

The symbol table is a section (or a number of sections) that exist within the ELF file and define the location, type, visibility and other traits of various symbols declared in the original source, created during compilation or linking, or otherwise present in the file. Since an ELF object can have multiple symbol tables, it is necessary to either iterate over the file's section headers, or to follow a reference from another section in order to access one.

<syntaxhighlight lang="cpp">
typedef struct {
	Elf32_Word		st_name;
	Elf32_Addr		st_value;
	Elf32_Word		st_size;
	uint8_t			st_info;
	uint8_t			st_other;
	Elf32_Half		st_shndx;
} Elf32_Sym;
</syntaxhighlight>

Each symbol table entry contains a number of notable bits of information such as the symbol name ('''st_name''', may be '''STN_UNDEF'''), the symbol's value ('''st_value''', may be absolute or relative address of value), and the field '''st_info''' which contains both the symbol type and binding. As an aside, the first entry in each symbol table is a NULL entry, so all of it's fields are 0.

<syntaxhighlight lang="cpp">
# define ELF32_ST_BIND(INFO)	((INFO) >> 4)
# define ELF32_ST_TYPE(INFO)	((INFO) & 0x0F)

enum StT_Bindings {
	STB_LOCAL		= 0, // Local scope
	STB_GLOBAL		= 1, // Global scope
	STB_WEAK		= 2  // Weak, (ie. __attribute__((weak)))
};

enum StT_Types {
	STT_NOTYPE		= 0, // No type
	STT_OBJECT		= 1, // Variables, arrays, etc.
	STT_FUNC		= 2  // Methods or functions
};
</syntaxhighlight>

As mentioned above, '''st_info''' contains both the symbol type and biding, so the 2 macros above provide access to the individual values. The enumeration '''StT_Types''' provides a number of possible symbol types, and '''StB_Bindings''' provides possible symbol bindings.

====Accessing the Value of a Symbol====

Some operation such as linking and relocation require the value of a symbol (or rather, the address thereof). Although the symbol table entries do define a field '''st_value''', it may only contain a relative address. Below is an example of how to compute the absolute address of the value of the symbol. The code has been broken up into multple smaller section so that it is easier to understand.

<syntaxhighlight lang="cpp">
static int elf_get_symval(Elf32_Ehdr *hdr, int table, uint idx) {
	if(table == SHN_UNDEF || idx == SHN_UNDEF) return 0;
	Elf32_Shdr *symtab = elf_section(hdr, table);

	uint32_t symtab_entries = symtab->sh_size / symtab->sh_entsize;
	if(idx >= symtab_entries) {
		ERROR("Symbol Index out of Range (%d:%u).\n", table, idx);
		return ELF_RELOC_ERR;
	}

	int symaddr = (int)hdr + symtab->sh_offset;
	Elf32_Sym *symbol = &((Elf32_Sym *)symaddr)[idx];
</syntaxhighlight>

The above performs a check against both the symbol table index and the symbol index; if either is undefined, 0 is returned. Otherwise the section header entry for the symbol table at the given index is accessed. It then checks that the symbol table index is not outside the bounds of the symbol table. If the check fails an error message is displayed and an error code is returned, otherwise the symbol table entry at the given index is is retrieved.

<syntaxhighlight lang="cpp">
	if(symbol->st_shndx == SHN_UNDEF) {
		// External symbol, lookup value
		Elf32_Shdr *strtab = elf_section(hdr, symtab->sh_link);
		const char *name = (const char *)hdr + strtab->sh_offset + symbol->st_name;

		extern void *elf_lookup_symbol(const char *name);
		void *target = elf_lookup_symbol(name);

		if(target == NULL) {
			// Extern symbol not found
			if(ELF32_ST_BIND(symbol->st_info) & STB_WEAK) {
				// Weak symbol initialized as 0
				return 0;
			} else {
				ERROR("Undefined External Symbol : %s.\n", name);
				return ELF_RELOC_ERR;
			}
		} else {
			return (int)target;
		}
</syntaxhighlight>

If the section to which the symbol is relative (given by '''st_shndx''') is equal to '''SHN_UNDEF''', the symbol is external and must be linked to its definition. The string table is retrieved for the current symbol table (the string table for a given symbol table is available in the table's section header in '''sh_link'''), and the symbol's name is found in the string table. Next the function '''elf_lookup_symbol'''() is used to find a symbol definition by name (this function is not provided, a minimal implementation always return NULL). If the symbol definition is found, it is returned. If the symbol has the '''STB_WEAK''' flag (is a weak symbol) 0 is returned, otherwise an error message is displayed and an error code returned.

<syntaxhighlight lang="cpp">
	} else if(symbol->st_shndx == SHN_ABS) {
		// Absolute symbol
		return symbol->st_value;
	} else {
		// Internally defined symbol
		Elf32_Shdr *target = elf_section(hdr, symbol->st_shndx);
		return (int)hdr + symbol->st_value + target->sh_offset;
	}
}
</syntaxhighlight>


If the value of '''sh_ndx''' is equal to '''SHN_ABS''', the symbol's value is absolute and is returned immediately. If '''sh_ndx''' doesn't contain a special value, that means the symbol is defined in the local ELF object. Since the value given by '''sh_value''' is relative to a section defined '''sh_ndx''', the relevant section header entry is accessed, and the symbol's address is computed by adding the address of the file in memory to the symbol's value with its section offset.

===The String Table===

The string table conceptually is quite simple: it's just a number of consecutive zero-terminated strings. String literals used in the program are stored in one of the tables. There are a number of different string tables that may be present in an ELF object such as .strtab (the default string table), .shstrtab (the section string table) and .dynstr (string table for dynamic linking). Any time the loading process needs access to a string, it uses an offset into one of the string tables. The offset may point to the beginning of a zero-terminated string or somewhere in the middle or even to the zero terminator itself, depending on usage and scenario. The size of the string table itself is specified by '''sh_size''' in the corresponding section header entry.

The simplest program loader may copy all string tables into memory, but a more complete solution would omit any that are not necessary during runtime such, notably those not flagged with '''SHF_ALLOC''' in their respective section header (such as .shstrtab, since section names aren't used in program runtime).

===The BSS and SHT_NOBITS===

The BSS (the section named ".bss") is in the simplest way of describing it: A block of memory which has been zeroed. The BSS is the area in memory where variables with global lifetime that haven't been initialized (or have been initialized to 0 or NULL) are stored. The section header for the BSS defines its '''sh_type''' as '''SHT_NOBITS''', which means that it isn't present in the file image, and must be allocated during runtime. A simple and naive way of allocating a BSS is to malloc some memory and zero it out with a memset. Failing to zero the BSS can cause unexpected behavious from any loaded programs. Another thing to note is that the BSS should be allocated before performing any operation that relies on relative addressing (such as relocation), as failing to do so can cause code to reference garbage memory or fault.

While the BSS is one specific example, any section that is of type '''SHT_NOBITS''' and has the attribute '''SHF_ALLOC''' should be allocated early on during program loading. Since this tutorial is intended to be general and unspecific, the example below will follow the trend and use the simplest example for allocating sections.

<syntaxhighlight lang="cpp">
static int elf_load_stage1(Elf32_Ehdr *hdr) {
	Elf32_Shdr *shdr = elf_sheader(hdr);

	unsigned int i;
	// Iterate over section headers
	for(i = 0; i < hdr->e_shnum; i++) {
		Elf32_Shdr *section = &shdr[i];

		// If the section isn't present in the file
		if(section->sh_type == SHT_NOBITS) {
			// Skip if it the section is empty
			if(!section->sh_size) continue;
			// If the section should appear in memory
			if(section->sh_flags & SHF_ALLOC) {
				// Allocate and zero some memory
				void *mem = malloc(section->sh_size);
				memset(mem, 0, section->sh_size);

				// Assign the memory offset to the section offset
				section->sh_offset = (int)mem - (int)hdr;
				DEBUG("Allocated memory for a section (%ld).\n", section->sh_size);
			}
		}
	}
	return 0;
}
</syntaxhighlight>

The example above allocates as much memory as necessary for the section, described by the '''sh_size''' field of the section's header. Although the function in the example only seeks out sections that needs to be allocated, it can be modified to perform other operation that should be performed early on into the loading process.

===Relocation Sections===

Relocatable ELF files have many uses in kernel programming, especially as modules and drivers that can be loaded at startup, and are especially useful because they are position independent, thus can easily be placed after the kernel or starting at some convenient address, and don't require their own address space to function. The process of relocation itself is conceptually simple, but may get more difficult with the introduction of complex relocation types.

Relocation starts with a table of relocation entries, which can be located using the relevant section header. There are actually two different kinds of relocation structures; one with an explicit added (section type '''SHT_RELA'''), one without (section type '''SHT_REL'''). Relocation entires in the table are continuous and the number of entries in a given table can be found by dividing the size of the table (given by '''sh_size''' in the section header) by the size of each entry (given by '''sh_entsize'''). Each relocation table is specific to a single section, so a single file may have multiple relocation tables (but all entries within a given table will be the same relocation structure type).

<syntaxhighlight lang="cpp">
typedef struct {
	Elf32_Addr		r_offset;
	Elf32_Word		r_info;
} Elf32_Rel;

typedef struct {
	Elf32_Addr		r_offset;
	Elf32_Word		r_info;
	Elf32_Sword		r_addend;
} Elf32_Rela;
</syntaxhighlight>

The above are the definitions for the different structure types for relocations. Of note if the value stored in '''r_info''', as the upper byte designates the entry in the symbol table to which the relocation applies, whereas the lower byte stores the type of relocation that should be applied. Note that an ELF file may have multiple symbol tables, thus the index of the section header table that refers to the symbol table to which these relocation apply can be found in the '''sh_link''' field on this relocation table's section header. The value in '''r_offset''' gives the relative position of the symbol that is being relocated, within its section.

<syntaxhighlight lang="cpp">
# define ELF32_R_SYM(INFO)	((INFO) >> 8)
# define ELF32_R_TYPE(INFO)	((uint8_t)(INFO))

enum RtT_Types {
	R_386_NONE		= 0, // No relocation
	R_386_32		= 1, // Symbol + Offset
	R_386_PC32		= 2  // Symbol + Offset - Section Offset
};
</syntaxhighlight>

As previously mentioned, the '''r_info''' field in '''Elf32_Rel'''('''a''') refers to 2 separate values, thus the set of macro functions above can be used to attain the individual values; '''ELF32_R_SYM'''() provides access to the symbol index and '''ELF32_R_TYPE'''() provides access to the relocation type. The enumeration '''RtT_Types''' defines the relocation typs this tutorial will encompass.

====Relocation Example====

Loading a relocatable ELF file entails processing all relocation entries present in the file (Remember to alloc allocate all '''SHT_NOBITS''' sections first!). This process starts with finding all the relocation tables in the file, which is done in the example code below.

<syntaxhighlight lang="cpp">
# define ELF_RELOC_ERR -1

static int elf_load_stage2(Elf32_Ehdr *hdr) {
	Elf32_Shdr *shdr = elf_sheader(hdr);

	unsigned int i, idx;
	// Iterate over section headers
	for(i = 0; i < hdr->e_shnum; i++) {
		Elf32_Shdr *section = &shdr[i];
		
		// If this is a relocation section
		if(section->sh_type == SHT_REL) {
			// Process each entry in the table
			for(idx = 0; idx < section->sh_size / section->sh_entsize; idx++) {
				Elf32_Rel *reltab = &((Elf32_Rel *)((int)hdr + section->sh_offset))[idx];
				int result = elf_do_reloc(hdr, reltab, section);
				// On error, display a message and return
				if(result == ELF_RELOC_ERR) {
					ERROR("Failed to relocate symbol.\n");
					return ELF_RELOC_ERR;
				}
			}
		}
	}
	return 0;
}
</syntaxhighlight>

Note that the code above only processes '''Elf32_Rel''' entries, but it can be modified to process entries with explicit addends as well. The code also relies on a function called '''elf_do_reloc''' which will be shown in the next example. This example function stops, displays an error message, and returns an error code if it's unable to process a relocation.

====Relocating a Symbol====

As the following function is fairly complex, it's been broken up into smaller manageable chunks and explained in detail. Note that the code shown below assumes that the file being relocated is a relocatable ELF file (ELF executables and shared objects may also contain relocation entries, but are processed somewhat differently). Also note that '''sh_info''' for section headers of type '''SHT_REL''' and '''SHT_RELA''' stores the section header to which the relocation applies.

<syntaxhighlight lang="cpp">
# define DO_386_32(S, A)	((S) + (A))
# define DO_386_PC32(S, A, P)	((S) + (A) - (P))

static int elf_do_reloc(Elf32_Ehdr *hdr, Elf32_Rel *rel, Elf32_Shdr *reltab) {
	Elf32_Shdr *target = elf_section(hdr, reltab->sh_info);

	int addr = (int)hdr + target->sh_offset;
	int *ref = (int *)(addr + rel->r_offset);
</syntaxhighlight>

The above code defines the macro functions that are used to complete relocation calculations. It also retrieves the section header for the section wherein the symbol exists and computes a reference to the symbol. The variable '''addr''' denotes the start of the symbol's section, and '''ref''' is created by adding the offset to the symbol from the relocation entry.

<syntaxhighlight lang="cpp">
	// Symbol value
	int symval = 0;
	if(ELF32_R_SYM(rel->r_info) != SHN_UNDEF) {
		symval = elf_get_symval(hdr, reltab->sh_link, ELF32_R_SYM(rel->r_info));
		if(symval == ELF_RELOC_ERR) return ELF_RELOC_ERR;
	}
</syntaxhighlight>

Next the value of the symbol being relocated is accessed. If the symbol table index stored in '''r_info''' is undefined, then the value defaults to 0. The code also references a function called '''elf_get_symval'''(), which was implemented previously. If the value returned by the function is equal to '''ELF_RELOC_ERR''', relocation is stopped and said error code is returned.

<syntaxhighlight lang="cpp">
	// Relocate based on type
	switch(ELF32_R_TYPE(rel->r_info)) {
		case R_386_NONE:
			// No relocation
			break;
		case R_386_32:
			// Symbol + Offset
			*ref = DO_386_32(symval, *ref);
			break;
		case R_386_PC32:
			// Symbol + Offset - Section Offset
			*ref = DO_386_PC32(symval, *ref, (int)ref);
			break;
		default:
			// Relocation type not supported, display error and return
			ERROR("Unsupported Relocation Type (%d).\n", ELF32_R_TYPE(rel->r_info));
			return ELF_RELOC_ERR;
	}
	return symval;
}
</syntaxhighlight>

Finally, this segment of code details the actual relocation process, performing the necessary calculating the relocated symbol and returning it's value on success. If the relocation type is unsupported an error message is displayed, relocation is stopped and the function returns an error code. Assuming no errors have occurred, relocation is now complete.

==The ELF Program Header==

The program header is a structure that defines information about how the ELF program behaves once it's been loaded, as well as runtime linking information. ELF program headers (much like section headers) are all grouped together to make up the program header table.

<syntaxhighlight lang="cpp">
typedef struct {
	Elf32_Word		p_type;
	Elf32_Off		p_offset;
	Elf32_Addr		p_vaddr;
	Elf32_Addr		p_paddr;
	Elf32_Word		p_filesz;
	Elf32_Word		p_memsz;
	Elf32_Word		p_flags;
	Elf32_Word		p_align;
} Elf32_Phdr;
</syntaxhighlight>

The program header table contains a continuous set of program headers (thus they can be accessed as if they were an array). The table itself can be accessed using the '''e_phoff''' field defined in the ELF header, assuming that it is present. The header itself defines a number of useful fields like '''p_type''' which distinguishes between headers, '''p_offset''' which stores the offset to the segment the header refers to, and '''p_vaddr''' which defines the address at which position-dependent code should exist.

For '''statically linked ELF's,''' the most important segment type is the '''PT_LOAD''' type. These denote sections in the ELF file that must be loaded into memory. <syntaxhighlight lang="c">
// mem is the location of the elf header
void *load_segment_to_memory(void *mem, Elf64_Phdr *phdr, int elf_fd) {
    size_t mem_size = phdr->p_memsz;
    off_t mem_offset = phdr->p_offset;
    size_t file_size = phdr->p_filesz;
    void *vaddr = (void *)(phdr->p_vaddr);
    // mmap the memory region with the correct protections
    int prot = 0;
    if (phdr->p_flags & PF_R) prot |= PROT_READ;
    if (phdr->p_flags & PF_W) prot |= PROT_WRITE;
    if (phdr->p_flags & PF_X) prot |= PROT_EXEC;
</syntaxhighlight>Note that there are two fields to denote the size. '''mem_size''' is the total amount of memory that the segment will occupy in memory, and '''file_size''' is the total amount of memory that has to be copied in. Now to actually map the memory to a certain virtual address, you must use ''mmap''. ''mmap'' will require page aligned accesses, so we will need to do some calculations in order to get the right addresses.<syntaxhighlight lang="c">
    off_t page_offset = (uint64_t)vaddr % PAGE_SIZE;
    void *aligned_vaddr = (void *)((uint64_t)vaddr - page_offset);
    size_t aligned_size = mem_size + page_offset;
</syntaxhighlight>Now can can call mmap. Note that there are potentially two mmap calls. This will read immediately from the file given that the proper file descriptor is given.<syntaxhighlight lang="c">
    mmap(aligned_vaddr, file_size + page_offset, prot, MAP_PRIVATE | MAP_ANONYMOUS, fd, mem_offset - page_offset);
    // technically we can just have vaddr as the first argument as mmap will 
    // automatically truncate to the start of the page
    if (mem_size > file_size) {
        void *page_break = ((uint64_t)vaddr + mem_offset + PAGE_SIZE - 1) & ~(PAGE_SIZE - 1);
        memset((uint64_t)vaddr + file_size, 0, (uint64_t)page_break - (uint64_t)vaddr - file_size);
        if (mem_size > page_break - (uint64_t)vaddr) {
            mmap(page_break, mem_size - (page_break - (uint64_t)vaddr), flags, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
            memset(page_break, 0, mem_size - (page_break - (uint64_t)vaddr));
        }
    }
}
</syntaxhighlight>

TODO : Expand and Detail.

==See Also==
===Articles===
*[[ELF]]
*[[Modular Kernel]]
*[[System V ABI]]

===External Links===
*[http://docs.oracle.com/cd/E23824_01/html/819-0690/chapter6-46512.html#scrolltoc ELF Format Specifications] Detailed and up-to-date ELF information (including SPARC in depth) by Oracle.
*[http://www.sco.com/developers/gabi/latest/contents.html System V ABI] about ELF
*[http://www.linuxfoundation.org/en/Specifications LSB specifications]<br />See (generic or platform-specific) 'Core' specifications for additional ELF information.
*[https://github.com/Bareflank/hypervisor/tree/master/bfelf_loader] Example ELF loader
*[https://code.google.com/p/elf-loader/] Example ELF loader

[[Category:Executable Formats]]
````

<a id="w003430"></a>

## W003430 — User:Gravaera/UDI Regions

Bron-ID: W003430
Namespace: User (2)
Revisie: 15484 | Datum: 2013-12-26T21:46:57Z
Bron: https://wiki.osdev.org/User:Gravaera/UDI_Regions
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FUDI_Regions&oldid=15484
Doorverwijzing: UDI Regions

````text
#REDIRECT [[UDI Regions]]
````

<a id="w003431"></a>

## W003431 — User:Gravaera/UDI Channels

Bron-ID: W003431
Namespace: User (2)
Revisie: 15486 | Datum: 2013-12-26T21:47:39Z
Bron: https://wiki.osdev.org/User:Gravaera/UDI_Channels
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FUDI_Channels&oldid=15486
Doorverwijzing: UDI Channels

````text
#REDIRECT [[UDI Channels]]
````

<a id="w003433"></a>

## W003433 — User:Gravaera/UDI Device Enumeration

Bron-ID: W003433
Namespace: User (2)
Revisie: 15490 | Datum: 2013-12-26T21:48:10Z
Bron: https://wiki.osdev.org/User:Gravaera/UDI_Device_Enumeration
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FUDI_Device_Enumeration&oldid=15490
Doorverwijzing: UDI Device Enumeration

````text
#REDIRECT [[UDI Device Enumeration]]
````

<a id="w003434"></a>

## W003434 — User:Gravaera/UDI Driver Loading

Bron-ID: W003434
Namespace: User (2)
Revisie: 15492 | Datum: 2013-12-26T21:48:29Z
Bron: https://wiki.osdev.org/User:Gravaera/UDI_Driver_Loading
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FUDI_Driver_Loading&oldid=15492
Doorverwijzing: UDI Driver Loading

````text
#REDIRECT [[UDI Driver Loading]]
````

<a id="w003435"></a>

## W003435 — User:Gravaera/UDI IRQ Model

Bron-ID: W003435
Namespace: User (2)
Revisie: 15494 | Datum: 2013-12-26T21:48:43Z
Bron: https://wiki.osdev.org/User:Gravaera/UDI_IRQ_Model
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FUDI_IRQ_Model&oldid=15494
Doorverwijzing: UDI IRQ Model

````text
#REDIRECT [[UDI IRQ Model]]
````

<a id="w003436"></a>

## W003436 — User:Gravaera/UDI Metalanguages

Bron-ID: W003436
Namespace: User (2)
Revisie: 15496 | Datum: 2013-12-26T21:49:19Z
Bron: https://wiki.osdev.org/User:Gravaera/UDI_Metalanguages
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FUDI_Metalanguages&oldid=15496
Doorverwijzing: UDI Metalanguages

````text
#REDIRECT [[UDI Metalanguages]]
````

<a id="w003437"></a>

## W003437 — Inline assembly

Bron-ID: W003437
Namespace: Main (0)
Revisie: 15500 | Datum: 2013-12-30T15:54:03Z
Bron: https://wiki.osdev.org/Inline_assembly
Revisie-URL: https://wiki.osdev.org/index.php?title=Inline_assembly&oldid=15500
Doorverwijzing: Inline Assembly

````text
#REDIRECT [[Inline Assembly]]
````

<a id="w003438"></a>

## W003438 — Talk:VESA Video Modes

Bron-ID: W003438
Namespace: Talk (1)
Revisie: 18660 | Datum: 2015-11-28T17:30:38Z
Bron: https://wiki.osdev.org/Talk:VESA_Video_Modes
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AVESA_Video_Modes&oldid=18660

````text
All the ''vbeInfoBlock'' should be replaced by ''vbeControllerInfo''.
(Or am I mistaken? I didn't want to just edit since I'm new here.)

Edit: also ''videomodes'' and ''VideoModePtr'' should be the same I think. It looks like two codes have been incompletely merged for this page?
: - Tang (please sign your posts using four ~'s)

: Fixed as appropriate - [[User:Combuster|Combuster]] 09:32, 7 January 2014 (CST)
````

<a id="w003439"></a>

## W003439 — User:Lionel/OSX

Bron-ID: W003439
Namespace: User (2)
Revisie: 15848 | Datum: 2014-04-06T05:34:13Z
Bron: https://wiki.osdev.org/User:Lionel/OSX
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALionel%2FOSX&oldid=15848

````text
{{In Progress}}

Developing on Mac OSX is a pretty hard thing. Apple's toolchain is broken and will not create ELF executables, only MACH-O. This is problematic, so here I will list my experiences on trying to fix it or port stuff to OSX.

== What this is NOT ==
This is NOT a roadmap on how to develop on OSX, but a list of experiences I had, and hint's and notes that might help you with OS Development. Feel free to contribute.

= Porting =

== Clang ==

Clang is pretty easy to port. And I mean a full clang, not just the apple version. The apple version of clang appears to be able to only output Mach-O binaries, and ignores any options to create ELF output, or cross compile. However, building a full version of clang is pretty easy. You require Subversion and cmake. Apple make and binutils work well enough to produce a working binary. Just follow the tutorial on compiling clang.
== libiconv ==
Building an updated version of libiconv was easy. Here's my process:
<source lang="bash">
function grab () {
    echo "Pulling $1..."
    if [ ! -f "$3" ]; then
        wget "$2/$3"
    else
        echo "Already have $1"
    fi
} #Credit Kevin Lange (klange) for grab function
##TODO:COMPLETE
</source>
== gcc ==
<source lang="bash">
VERSION=4.7.0
PREFIX=/usr/gcc-4.7.0
LANGUAGES=c,c++
MAKE=make
# Or
# MAKE='make -j 4' # to compile using four cores

brew-path() { brew info $1 | head -n3 | tail -n1 | cut -d' ' -f1; }

# Prerequisites

brew install gmp
brew install mpfr
brew install libmpc

# Download & install the latest GCC

mkdir -p /usr/gcc-4.7.0
mkdir temp-gcc
cd temp-gcc
wget ftp://ftp.gnu.org/gnu/gcc/gcc-4.7.0/gcc-4.7.0.tar.gz
tar xfz gcc-4.7.0.tar.gz
rm gcc-4.7.0.tar.gz
cd gcc-4.7.0

mkdir build
cd build

../configure \
   --prefix=/usr/gcc-4.7.0 \
   --with-gmp=$(brew-path gmp) \
   --with-mpfr=$(brew-path mpfr) \
   --with-mpc=$(brew-path libmpc) \
   --program-suffix=-4.7.0 \
   --enable-languages=c,c++ \
   --with-system-zlib \
   --enable-stage1-checking \
   --enable-plugin \
   --enable-lto \
   --disable-multilib

sudo make -j 4 bootstrap

sudo make -j 4 install

</source>
== binutils (cross compiling) ==
Porting binutils was A HUGE PAIN. First of all, you need to compile a working GCC. Then, you need to run the following commands.
<source lang="bash">
export PREFIX=/usr/local/cross
export TARGET=i586-elf
export MAKEFLAGS="-j 4"
export CC=/usr/gcc-4.7.0/bin/gcc-4.7.0
export CXX=/usr/gcc-4.7.0/bin/g++-4.7.0
export CPP=/usr/gcc-4.7.0/bin/cpp-4.7.0
export LD=/usr/gcc-4.7.0/bin/gcc-4.7.0
mkdir -p ~/src/build-binutils/
curl ftp://ftp.gnu.org/gnu/binutils/binutils-2.22.tar.bz2 | tar -x -f - -C ~/src/
cd ~/src/build-binutils/
../binutils-2.22/configure --prefix=$PREFIX --target=$TARGET --disable-nls
</source>
Now STOP. There is a bug in gcc that makes -Werror turn on for each component, and yet it wont be disabled by NO_WERROR.
Enter build-binutils, and in each subfolder do a search for -Werror inside of Makefile, and delete it from CFLAGS.
Then, finally you can:
<source lang="bash">
make all
sudo make install
</source>
To install binutils to /usr/local/cross
````

<a id="w003440"></a>

## W003440 — User:Bellezzasolo/Windows

Bron-ID: W003440
Namespace: User (2)
Revisie: 15512 | Datum: 2014-01-04T17:29:56Z
Bron: https://wiki.osdev.org/User:Bellezzasolo/Windows
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ABellezzasolo%2FWindows&oldid=15512

````text
== Microsoft Windows ==
=== Introduction ===
OS development on windows isn't the easiest. The most used toolchain, binutils, isn't native.

== Visual Studio ==
=== Introduction ===
Visual Studio only outputs portable executables. This is incompatible with GRUB.

=== GRUB ===
Youy can however, create a multiboot header. This bypasses the format barrier.

==== 64 bit ====
You can't embed a 32 bit stub in a PE32+. Instead, load a 32 bit binary (with multiboot header), with the 64 bit kernel loaded as a module. Relocate the kernel, set up long mode, then call it.
````

<a id="w003441"></a>

## W003441 — User talk:Bellezzasolo/Windows

Bron-ID: W003441
Namespace: User talk (3)
Revisie: 15524 | Datum: 2014-01-08T08:52:37Z
Bron: https://wiki.osdev.org/User_talk:Bellezzasolo/Windows
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ABellezzasolo%2FWindows&oldid=15524

````text
Did you know there's an entire page dedicated to [[Visual Studio]] as well as [[Cygwin]]? Given this page's rough layout it seems that you might want to consider editing those instead. - [[User:Combuster|Combuster]] 16:26, 7 January 2014 (CST)

yes, I will once I finish it here --[[User:Bellezzasolo|Bellezzasolo]] 02:52, 8 January 2014 (CST)
````

<a id="w003443"></a>

## W003443 — User:Klange/Room-full-of-painters

Bron-ID: W003443
Namespace: User (2)
Revisie: 15533 | Datum: 2014-01-13T09:05:52Z
Bron: https://wiki.osdev.org/User:Klange/Room-full-of-painters
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKlange%2FRoom-full-of-painters&oldid=15533

````text
The '''''room-full-of-painters''''' is an explanation for how [[compositing]] window managers operate. Particularly, the form of the explanation below covers a simple, naïve compositor. It is not a coincidence that the metaphor resembles the process of an animation house, as many of the terms used in compositing are derived from animation.

== The Room ==

Imagine, if you will, a '''room full of people'''. Now, imagine that one of these people has a large '''canvas''' in front of him; we will call him the '''Master Artist'''. Many of the other people in the room also have their own, '''smaller canvases''' and various painting tools; we will refer to them as '''painters'''. There is also a man standing near the door of the room, looking very auspicious - he is the '''Curator'''.

The '''Master Artist''' has many tools, but they may seem peculiar for a painter: a '''scanner/copier''', a '''small knife''', some '''glue''', and a box of little '''stickers''' shaped variously as hands and arrows and I-bars. He also keeps a '''notebook''', filled with tables listing coordinates and the names of his apprentice painters.

The '''painters''' have their '''small canvases''', and each has a box of painter tools containing whatever they please.

The '''painters''' take their canvases and draw whatever they like on them. They then give the canvases to the '''Master Artist''', who places each canvas into the copier and produces a copy. He then takes the copies and places them upon his canvas, gluing each into place as it should be from his notebook, in order as prescribed in the same. When a copy extends beyond his canvas, he takes the knife and cuts off the excess. Content with his work, he then takes a sticker from his box and places it on top, as also prescribed by his notebook. The '''Master Artist''' then takes his completed canvas and gives it another person, the '''Curator''', who promptly gives the master a new canvas. The '''Master Artist''' then returns all of his apprentices' canvas to the various painters, and the process repeats.

Now while all of this is happening, the '''Master Artist''' has some assistants running around. One of them, '''Assistant A''', runs in and out of the room, brushing dangerously close to the Curator. When he returns, he always goes straight to the '''Master Artist''' and gives him a note. Scrawled on it are such things as "left 1mm" and "key 12 down". The '''Master Artist''' looks at the note, then looks at his notebook, does some quick math, jots down a note of his own, and calls upon '''Assistant B'''. '''Assistant B''' takes the note and runs off to one of the apprentice '''painters''' to deliver it. The '''painter''' then does as he pleases with it and '''Assistant B''' returns to his post.

Sometimes, a '''painter''' will decide that he is '''done painting''', or that he would like a '''larger canvas''' (or perhaps '''another canvas'''!). When this happens, the painter jots down a note and runs off to the '''Master Artist''' to deliver it. The '''Master Artist''' can then do what was asked, or refuse. If the painter says he is '''done''', the Master Artist will throw that painter's canvas in the garbage and erase the line in his notebook for it. If the painter would like an additional canvas, the Master Artist can give them one, or refuse.

Additionally, sometimes a new painter will enter the room, or perhaps another one of the people in the room (who was not previously a painter) will pull out a brush and request a canvas from the Master Artist.

== Explanation ==

The people represent ''programs''. The '''Master Artist''' represents the ''compositor'' and the various '''painters''' represent ''graphical applications''. The '''Master Artist's canvas''' represents ''the framebuffer'', on the computer. The '''Curator''' represents the ''graphics card'', and each canvas brought to them is shown on screen. Since ''graphical applications'' may have multiple windows, '''painters''' may have multiple '''canvases''', which they draw on together. The process by which the '''Master Artist''' uses his copier and knives and glue sums up ''bit-blitting'' windows to the framebuffer. The '''Master Arist''''s assistants represent communication, '''A''' representing ''messages from input drivers'' - such as the mouse and keyboard - and '''B''' representing ''messages from the compositor to graphical applications'', particularly input events; the lack of an assistant passing messages from the painters to the Master Artist is representative of the common synchronous nature of messages from graphical apps back to the compositor (and saves the need for having many assistants to run messages for all of the various painters). The '''Master Artist's notebook''' represents the compositor's internal memory of where windows should be, as well as where things like the pointer should be. The '''Master Artist's box of stickers''' represents mouse cursors, which are usually drawn by the compositor itself, though they could simply be yet another canvas drawn by an application.

== Supplement ==

=== Double Buffering ===
Each '''painter''', including the '''Master Artist''', has two canvases instead of just the one. This is true for each canvas that has been requested. Whenever the '''Master Artist''' collects one canvas, the '''painter''' continues drawing on the other.

=== Damage Regions / Masking ===
Involves more creative uses of the knife, and usually also involves keeping the canvas instead of giving it to the Curator. When the '''Master Artist''' collects the canvases, he copies only the updated portions (which are noted on a slip of paper accompanying the canvas), cutting off the rest, and places the new scraps atop their older counterparts.
````

<a id="w003444"></a>

## W003444 — User:Turdus/Universal Driver Block Format

Bron-ID: W003444
Namespace: User (2)
Revisie: 15546 | Datum: 2014-01-20T21:25:48Z
Bron: https://wiki.osdev.org/User:Turdus/Universal_Driver_Block_Format
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATurdus%2FUniversal_Driver_Block_Format&oldid=15546

````text
[[Category:Universal Driver Block Format]]
[[Category:Drivers]]
[[Category:Driver Interfaces]]

If it's okay, I would like to have this wiki page as the primary source for the UDRV Specification. Modify and expand only along the given lines.
Please note that currently it's nothing more than a DRAFT.

=Disclaimer=
UDRV stands for '''U'''niversal '''DR'''i'''V'''er, which is the name of
* a block format,
* a library interface
* and a CLI tool
as well.

This specification aims to provide a way to share drivers among hobby kernels not source but binary level. Inspired by [[UDI]], a dead project sadly, but it's message should not be lost, hence this spec.

UDRV takes less effort to understand and to implement than UDI. Adopting should be straightforward once you have user space libraries and hopefully will spread among hobby kernels eventually. It's needless to say what benefits that would have.

As opposite to [[UDI]], UDRV is quite simple, yet robust and can be used with any existing executable format. To UDRV point of view, driver is a (usually small) binary blob that has to be linked with the rest of the system (or directly with the kernel if you're using [[monolithic kernel]] design, just like Linux kernel modules). In order to achieve linkage, UDRV specifies a data block, that has to be inlined in the binary's first 64k on a 16 aligned boundary. It's very similar to [[Multiboot]] block. This block describes the driver, and also holds pointers to it's functions.

=Terms and phrases=
* block - usually about 512 bytes, can be larger, a non-kernel-specific way to describe the driver's interface.
* driver interface - set of pointers to well defined functions using well defined ABI
* device instance - memory area that holds the current state of the device (implementation is kernel specific)
* device path - an UEFI generic device path binary string (see UEFI specification 9.3.1)
* device class - represents class, see [http://pci-ids.ucw.cz/read/PD/ PCI Device class]. Note that UDRV uses this to categorize drivers, it does not mean UDRV is limited to PCI drivers in any way.
* device subclass - see PCI Device SubClass
* device interface - see PCI Device Interface. Do not confuse with driver interface.
* driver major version - reflects the software version, not mknod's major & minor
* driver minor version - patch level
* offset - pointer to UDRV block's data area, relative to UDRV magic+256
* pointer - pointer to code, relative to the beginning of the binary file
* properties - part of device instance memory, user tunable arguments
* newline - (NL) single ASCII character 10
* string end - single ASCII character 0
* CLI tool - a command to interact with UDRV compatible devices from shell
* UDRV compatible driver - executable binary blob image with UDRV Block in it
* UDRV compatible device - a device driver that implements UDRV Driver Interface 
* UDRV compatible OS - can dynamically link UDRV Driver Interface and has at least one UDRV CLI tool
* UDRV compatible software - any program that can read and understand UDRV Blocks. Linkers, configuration tools

=Universal Driver Specification DRAFT=
==UDRV Block Format==
===Executable driver binary===
There's no restriction on it's format (dll, so, kext etc. use whatever you like, see [[shared library]]).

''A word on function entry points in executable''
Obviously a kernel that understands the format can use it natively and there's no need for any additional pointers. But for kernels that don't, or formats that do not support other than entry point (such as [[a.out]]), the UDRV Block can be used.

===Detecting UDRV===
An UDRV compatible driver should have an UDRV Block in it's first 65536 bytes aligned on a 16 byte boundary. UDRV compatible software (like a linker or the CLI tool, or even a GUI configuration tool if you implement it) will look for this block and use it's contents.

===Header===
The block starts with a 256 byte long header, which is followed by initialized data, terminated by a zero.

This block (naturally since it's in a binary executable) holds information in binary format, but some are in human readable format for comfort.

<source lang="c">
struct udrv_header {
	uint32	magic;		//characters 'UDRV'
	uint16	size;		//UDRV Block size in bytes counting from magic
	uint8	major_ver;	//driver major version
	uint8	minor_ver;	//driver minor version
	uint8	type;		//driver type
	uint8	flags;		//UDRV flags for compatible software
	uint8	device_class;	//PCI Device Class
	uint8	device_subclass;
	uint8	device_interface;
	uint8	chksum;		//add bytes in block and check for zero
	uint16	offs_name;	//offset to an UTF-8 ASCIIZ string (used by dependency check)
	uint16	offs_meta;	//Meta Info string
	uint16	offs_prop;	//property definitions for instance memory
	char[16] arch;		//architecture, x86_32, x86_64, ARMv8 etc.
	uint32	func_reset;
	uint32	func_getcapabilty;
	uint32	func_cmd;
	uint32	func_open;
	uint32	func_read;
	uint32	func_write;
	uint32	func_close;
  union {

    //character devices
    struct {
	uint32	func_seek;
    } chr;

    //block devices
    struct {
	uint32	func_seek;
	uint32	func_read;
	uint32	func_write;
	uint32	func_getblocksize;
	uint32	func_setblocksize;
    } block;

    //timers
    struct {
	uint32	func_resetcounter;
	uint32	func_oneshot;
	uint32	func_periodic;
	uint32	func_getquantum;
	uint32	func_setquantum;
    } timer;

    //line buffer devices (like serial)
    struct {
	uint32	func_seek;
	uint32	func_fgets;
	uint32	func_fputs;
	uint32	func_getbuffsize;
	uint32	func_setbuffsize;
    } linebuff;

    //devices that can hold sub-devices
    struct {
	uint32	func_list;
	uint32	func_add;
	uint32	func_remove;
    } bus;

    //virtual file system driver hooks
    struct {
	uint32	func_fopen;
	uint32	func_fread;
	uint32	func_fwrite;
	uint32	func_fclose;
	uint32	func_fstat;
	uint32	func_opendir;
	uint32	func_readdir;
	uint32	func_closedir;
	uint32	func_mkdir;
	uint32	func_rmdir;
	uint32	func_link;
	uint32	func_symlink;
	uint32	func_unlink;
	uint32	func_mount;
	uint32	func_umount;
    } vfs;
  }
} udrv_header_t;

</source>
===Data area===
It's nothing more than an initialized data area to hold stuff like name, meta info etc.

====Default values====
If properties offset (offs_prop) is not zero, it selects a Property definitions string within data area. The first column defines size, sum it up to get default values size. Normally you don't need this information, since you convert offsets. By default, all values are zero, unless you define them at compile time (eg.: pskeyboard should have 1 in "irq" property).

If properties offset is zero, then the device is not user tunable and there's no default value after the header at all (meaning other offsets can use zero as valid offset).

====Meta Info string====
<pre>
#comment
author (author(s))
supports (comma ',' separated list of PCI IDs in hex)
depends (comma ',' separated list of other UDRV device drivers this driver relies on
conflicts (comma ',' separated list of other UDRV drivers that this driver cannot run with
homepage (homepage of the author)
hardware (homepage of the device's manufacturer)
license (licensing information)
download (url to the driver download directory)
device (device file path and name regexp string, not including "/dev/", like "disk[0-9](p[0-9])?","net[0-9]","uuid/[0-9a-f]{16}")

Example:
author Me<a@myproject.tld>
depends hpet, pci>=1.2
license LGPL
device eth[0-9]
#this will look for "http://myproject.tld/public/drivers/" (category) "/" (udrvname) "-" (major ver) "." (minor ver) ".udrv"
#which can be a single binary or either cpio or tar (optionally compressed with gzip, bzip2)
#you must provide (udrvname) "-latest.udrv" symlink there. Redirects in http responses are likely too
#to handle dependencies to foreign drivers without mirroring.
download http://myproject.tld/public/drivers/
</pre>

====Property definitions====
<pre>
Each line in property string looks like:
(number of bytes occupied) ' ' (name of the property) ' ' (type) NL
Ends in a zero byte.
Example:
1 irq int(min=1,max=16)
4 ioapic hexint(min=0xF0000000,max=0xFFF00000)
2 language optionlist('en','hu')
1 boolean optionlist('0':'false','1':'true')
</pre>

==UDRV Library Interface==
Drivers compiled for x86_32 use [[stdcall]], x86_64 use [http://www.x86-64.org/documentation/abi.pdf‎ fastcall] [[calling convention]].
<source lang="c">
#define UDRV_MAGIC "UDRV"
enum { UDRV_TYPE_CHAR, UDRV_TYPE_BLOCK, UDRV_TYPE_TIMER, UDRV_TYPE_LINEBUFF, UDRV_TYPE_BUS, UDRV_TYPE_VFS };
#define UDRV_FLAG_64BIT 1
enum { UDRV_CLASS_UNCLASSIFIED, UDRV_CLASS_STORAGE, UDRV_CLASS_NETWORK... };

int reset (void* instance, void* malloc, void* free, void* sendsignal);
char* getcapability(void * instance);
//similar to ioctl, but suitable for kernels without vargs too
int cmd(void *instance, int cmd, void* arg1, void* arg2, void* arg3, void* arg4, void* arg5, void* arg6);

...soon
</source>

===Linking the library===
For 64 bit systems you'll have to expand pointers to 8 bytes. Once the driver loaded, your linker should adjust the pointers with the memory location it was loaded to (uint32 -> void*).

===Instance memory===
Device instance memory has a two pointer header. One for the name, one for the path. It's 8 bytes for 32 bit, 16 for 64 bit.
<pre>
struct udrv_instance {
	unsigned char*	name; //UTF-8 ASCIIZ, without "/dev/", like "disk1p3"
	unsigned char*	path; //UEFI generic device path (note, 0xff terminated!)
} udrv_instance_t;
</pre>
That header is followed by device properties (current configuration) and optionally local variables (current state).
On creation device properties are filled in with default values from UDRV Block.

Not counting the header the structure of instance memory is totally up to kernel. It can be placed in kernel memory,
user memory (for microkernels, either in fs server or driver address space), even in fs inode, etc.

==UDRV CLI tool==
Probably most hobby OS is some kind of UN*X, it's safe to assume to have a shell. Therefore it's highly recommended to implement this command on your OS.
If you do some extreme stuff without shell, you'll need to provide a way to communicate with the UDRV compatible devices (voice command, GUI whatever).

===Commands===
<pre>
udrv reset device
udrv (get|set) (device|default device|driver) property [value]
  device - query/modify device instance
  driver - query/modify default values in UDRV block
  default device - get device's driver name and query/modify it's UDRV block
udrv path device          - print generic device path
udrv list device          - list sub-devices (only for bus-type devices)
udrv update driver        - not required for basic compatibility
</pre>

====Reset device====
<pre>
udrv reset /dev/serial0
</pre>
Resets the device to it's initial state.

====Query device====
<pre>
udrv get /dev/eth0 netmask
</pre>
====Configure device====
<pre>
udrv set /dev/eth0 ip 1.2.3.4/24
</pre>
====Query defaults====
<pre>
udrv get default /dev/eth0 mtu
udrv get /lib/mykernel/network/ns2k.o mtu
</pre>
====Configure defaults====
<pre>
udrv set default /dev/eth0 mtu 1490
udrv set /lib/mykernel/network/ns2k.o mtu 1490
</pre>
====Query sub-devices====
<pre>
udrv list /dev/pci
</pre>
====Query UEFI generic device path====
<pre>
udrv path /dev/disk0
</pre>
Returns UEFI generic device path (see UEFI spec 9.3.1) in human readable format. For devices that does not have one (or you are not using UEFI at all), your kernel should create one, it's quite easy to put a byte one after another.

====Update driver====
<pre>
udrv update /lib/mykernel/network/ne2k.o
</pre>
Once you have networking, you can use the download url given in Meta Info string to refresh your drivers on the fly. UDRV does not have a central repository by design, the drivers expected to be distributed on hobby kernel's download sites. For list of available drivers, this page is the primary source.

This function is not required to be fully UDRV compatible, but highly recommended (once you have networking). In lack of native networking, you can use your build system to refresh the driver binaries.

===Porting===
To make porting this command really easy, it's source is a single file in ANSI C. You should be able to cross-compile (or compile :-) ) it with "cc udrv.c -o udrv", where "cc" is the C compiler of your choice (GCC, TCC, LLVM whatever).

Alternatively if you don't use C at all, you can write your own version of the command expecting the same arguments as this reference implementation.

udrv.c:
<source lang="c">
#include <stdio.h>
#include <stdlib.h>
#include <udrv.h>

int main(int argc, char** argv)
{
   //soon
}
</source>
===Tested compilers===
List of C compilers known to compile the above code correctly:

=List of UDRV Device Drivers=
List of download urls to driver binaries.
==01 Mass storage controller==
==02 Network controller==
==03 Display controller==
==04 Multimedia controller==
==05 Memory controller==
==06 Bridge==
==07 Communication controller==
==08 Generic system peripheral==
==09 Input device controller==
==0a Docking station==
==0b Processor==
It's very unlikely to have such a driver, for performance reasons and hardware issues CPU resource management usually implemented natively.
==0c Serial bus controller==
==0d Wireless controller==
==0e Intelligent controller==
==0f Satellite communications controller==
==10 Encryption controller==
==11 Signal processing controller==

=List of UDRV Compatible Operating Systems=
List of hobby operating systems that have implemented UDRV.
==Monolithic kernels==
==Micro kernels==
````

<a id="w003445"></a>

## W003445 — Universal Driver Block Format

Bron-ID: W003445
Namespace: Main (0)
Revisie: 15540 | Datum: 2014-01-17T13:19:12Z
Bron: https://wiki.osdev.org/Universal_Driver_Block_Format
Revisie-URL: https://wiki.osdev.org/index.php?title=Universal_Driver_Block_Format&oldid=15540
Doorverwijzing: User:Turdus/Universal Driver Block Format

````text
#REDIRECT [[User:Turdus/Universal Driver Block Format]]
````

<a id="w003446"></a>

## W003446 — User talk:Turdus/Universal Driver Block Format

Bron-ID: W003446
Namespace: User talk (3)
Revisie: 15541 | Datum: 2014-01-17T13:26:44Z
Bron: https://wiki.osdev.org/User_talk:Turdus/Universal_Driver_Block_Format
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ATurdus%2FUniversal_Driver_Block_Format&oldid=15541

````text
Looks like a nice interface.
It would need some size_t s for 64 bit expansion.
If in a native format, you could add a function that registered the interface in the entry point.
It would also allow non-natives.
It might even be possible to wrap classes around it.
Keep up the good work!
--[[User:Bellezzasolo|Bellezzasolo]] 07:26, 17 January 2014 (CST)
````

<a id="w003448"></a>

## W003448 — Meaty Skeleton

Bron-ID: W003448
Namespace: Main (0)
Revisie: 30797 | Datum: 2026-06-19T11:17:13Z
Bron: https://wiki.osdev.org/Meaty_Skeleton
Revisie-URL: https://wiki.osdev.org/index.php?title=Meaty_Skeleton&oldid=30797

````text
{{FirstPerson}}
{{BeginnersWarning}}
{{Rating|1}}
{{Kernel designs}}

This tutorial continues from [[Bare Bones]] and creates a minimal template
operating system in the [[Stan Dard]] style suitable for further modification or
as inspiration for your initial operating system version. The [[Bare Bones]]
tutorial only gives you the absolutely minimal code to demonstrate how to
correctly cross-compile a kernel, however this is unsuitable as an example
operating system. Additionally, this tutorial implements neccesary ABI features
needed to satisfy the ABI and compiler contracts to prevent possible mysterious
errors.

This tutorial also serves as the initial template tutorial on how to
[[Creating_a_C_Library|create your own libc]] (Standard C Library). The GCC
documentation explicitly states that libgcc requires the freestanding
environment to supply the <tt>memcmp</tt>, <tt>memcpy</tt>, <tt>memmove</tt>,
and <tt>memset</tt> functions, as well as <tt>abort</tt> on some platforms. We
will satisfy this requirement by creating a special kernel C library (libk) that
contains the parts of the user-space libc that are ''freestanding'' (doesn't
require any kernel features) as opposed to ''hosted'' libc features that need to
do system calls.

== Preface ==

This tutorial is an example on how you could structure your operating system in
a manner that will continue to serve you well for the foreseeable future. This
serves as both inspiration and as an example for those that wish to something
different, while as a base for the rest. The tutorial does embed a few important
concepts into your operating system such as the existence of a libc, as well as
indirectly other minor Unix and ABI semantics. Adapt what you wish from this
tutorial. Note that the shell script and Make-based build system constructed in
this tutorial is meant for Unix systems. There
is no need to make this tutorial portable across all operating systems as the
provided code is just an example.

We will name this new example operating system <tt>myos</tt>. This is just a
placeholder and you should replace all occurrences of <tt>myos</tt> with what you
decide to call your operating system.

== Bare Bones ==
{{Main|Bare Bones}}

You are expected to have completed the [[Bare Bones]] tutorial before continuing
to this tutorial. It is not strictly neccesary to have completed [[Bare Bones]],
but doing so confirms that your development environment works as well as
explaining a number of core things.

You should probably discard the code you got from toying around with
[[Bare Bones]] and start over with this tutorial as your basis.

== Building a Cross-Compiler ==
:''Main article: [[GCC Cross-Compiler]], [[Why do I need a Cross Compiler?]]

You ''must'' use a [[GCC Cross-Compiler]] in this tutorial as in the
[[Bare Bones]] tutorial. You should use the '''i686-elf''' target in your
cross-compiler, though any <tt>ix86-elf</tt> target (but no less than i386) will
do fine for our purposes here.

You ''must'' configure your cross-binutils with the <tt>--with-sysroot</tt> option, otherwise linking will mysteriously fail with the ''this linker was not configured to use sysroots'' error message. If you forgot to configure your cross-binutils with that option, you'll have to rebuild it, but you can keep your cross-gcc.

== Dependencies ==

You will need these dependencies in order to complete this tutorial:

* i686-elf toolchain, as discussed above.
* GRUB, for the grub-mkrescue command, along with the appropriate runtime files.
* Xorriso, the .iso creation engine used by grub-mkrescue.
* GNU make 4.0 or later.
* Qemu, optionally for testing the operating system.

This tutorial requires a GNU/Linux system, or a similar enough system. The BSD systems may almost work. OS X is not supported but can possibly be made to work with some changes. Windows is not supported, but Windows environments like Cygwin and Windows Subsystem For Linux (WSL) might work.

=== Debian-family Users ===

Install the i686-elf toolchain as described above and then install the packages <tt>xorriso grub-pc-bin</tt>.

== System Root ==

Normally when you compile programs for your local operating system, the compiler locates development files such as headers and libraries in system directories such as:

:<tt>/usr/include</tt>
:<tt>/usr/lib</tt>

These files are of course not usable for your operating system. Instead you want to have your own version of these directories that contains files for your operating system:

:<tt>/home/bwayne/myos/sysroot/usr/include</tt>
:<tt>/home/bwayne/myos/sysroot/usr/lib</tt>

The <tt>/home/bwayne/myos/sysroot</tt> directory acts as a fake root directory for your operating system. This is called a system root, or ''sysroot''.

You can think of the sysroot as the root directory for your operating system. Your build process will build each component of your operating system (kernel, standard library, programs) and gradually install them into the system root. Ultimately the system root will be a fully functional root filesystem for your operating system, you format a partition and copy the files there, add the appropriate configuration files, configure a bootloader to load the kernel from there, and use your harddisk driver and filesystem driver to read the files from there. The system root is thus a temporary directory that will ultimately become the actual root directory of your operating system.

In this example the cross system root is located as <tt>sysroot/</tt>, which is
a directory created by the build scripts and populated by the
<tt>make install</tt> targets. The makefiles will install the system headers
into the <tt>sysroot/usr/include</tt> directory, the system libraries into the
<tt>sysroot/usr/lib</tt> directory and the kernel itself into the
<tt>sysroot/boot</tt> directory.

We already use system roots because it will make it smoother to add a user-space when you get that far. This scheme is very convenient when you later [[Cross-Porting_Software|Port Third-Party Software by Cross-Compiling It]].

The <tt>-elf</tt> targets have no user-space and are incapable of having one. We configured the compiler with system root support, so it will look in <tt>${SYSROOT}/usr/lib</tt> as expected. We prevented the compiler from searching for a standard library using the --without-headers option when building <tt>i686-elf-gcc</tt>, so it will ''not'' look in <tt>${SYSROOT}/usr/include</tt>. (Once you add a user-space and a libc, you will configure your custom cross-gcc with <tt>--with-sysroot</tt> and it will look in <tt>${SYSROOT}/usr/include</tt>. As a temporary work-around until you get that far, we fix it by passing <tt>-isystem=/usr/include</tt>).

You can change the system root directory layout if you wish, but you will have to modify some Binutils and GCC source code and tell them [[OS Specific Toolchain|what your operating system is]]. This is advanced and not worth doing until you add a proper user-space. Note that the cross-linker currently looks in <tt>/lib</tt>, <tt>/usr/lib</tt> and <tt>/usr/local/lib</tt> by default, so you can move files there without changing Binutils. Also note that we use the <tt>-isystem</tt> option for GCC (as it was configured without a system include directory), so you can move that around freely.

== System Headers ==

The <tt>./headers.sh</tt> script simply installs the headers for your libc and
kernel (system headers) into <tt>sysroot/usr/include</tt>, but doesn't actually
cross-compile your operating system. This is <tt>useful</tt> as it allows you to
provide the compiler a copy of your headers before you actually compile your
system. You will [[Hosted_GCC_Cross-Compiler#Sysroot_Headers|need to provide the
standard library headers]] when you build a [[Hosted GCC Cross-Compiler]] in the 
future that is capable of an user-space.

Note how your cross-compiler comes with a number of fully freestanding headers
such as <tt>stddef.h</tt> and <tt>stdint.h</tt>. These headers simply declare
types and macros that are useful. Your kernel standard library will supply a
number of useful functions (such as <tt>strlen</tt>) that doesn't require system
calls and are freestanding except they need an implementation somewhere.

== Makefile Design ==

The makefiles in this example respect the environment variables (such as
<tt>CFLAGS</tt> that tell what default compile options are used to compile C
programs). This lets the user control stuff such as which optimization levels
are used, while a default is used if the user has no opinion. The makefiles also
make sure that particular options are always in CFLAGS. This is done by having
two phases in the makefiles: one that sets a default value and one that adds
mandatory options the project makefile requires:

<syntaxhighlight lang="make">
# Default CFLAGS:
CFLAGS?=-O2 -g

# Add mandatory options to CFLAGS:
CFLAGS:=$(CFLAGS) -Wall -Wextra
</syntaxhighlight>

== Architecture Directories ==

The projects in this example (libc and kernel) store all the architecture
dependent source files inside an <tt>arch/</tt> directory with their own
sub-makefile that has special configuration. This cleanly separates the systems
you support and will make it easier to port to other systems in the future.

== Kernel Design ==

We have moved the kernel into its own directory named <tt>kernel/</tt>. It would
perhaps be better to call it something else if your kernel has another name than
your full operating system distribution, though calling it <tt>kernel/</tt>
makes it easier for other hobbyist developers to find the core parts of your new
operating system.

The kernel installs its public kernel headers into
<tt>sysroot/usr/include/kernel</tt>. This is useful if you decide to create a
kernel with modules, where modules can then simply include the public headers
from the main kernel.

GNU GRUB is used as the bootloader and the kernel uses Multiboot as in the
[[Bare Bones]] tutorial.

The kernel implements the correct way of [[Calling_Global_Constructors|invoking
global constructors]] (useful for C++ code and C code using
<tt>__attribute__((constructor))</tt>. The bootstrap assembly calls <tt>_init</tt>
which invokes all the global constructors. These are invoked very early in the
boot without any specific ordering. You should only use them to initialize
global variables that could not be initialized at runtime.

The special <tt>__is_kernel</tt> macro lets the source code detect whether
it is part of the kernel.

== libc and libk Design ==

The libc and libk are actually two versions of the same library, which is stored
in the directory <tt>libc/</tt>. The standard library is split into two
versions: freestanding and hosted. The difference is that the freestanding
library (libk) doesn't contain any of the code that only works in user-space,
such as system calls. The libk is also built with different compiler options,
just like the kernel isn't built like normal user-space code.

You are not required to have a libk. You could just as easily have a regular
libc and a fully seperate minimal project inside the kernel directory. The libk
scheme avoids code duplication, so you don't have to maintain multiple versions
of <tt>strlen</tt> and such.

This example doesn't come with a usable libc. It compiles a libc.a that is
entirely useless, except being a skeleton we can build on when we add user-space
in a later tutorial.

Each standard function is put inside a file with the same name as the function
inside a directory with the name of the header. For instance, <tt>strlen</tt>
from <tt>string.h</tt> is in <tt>libc/string/strlen.c</tt> and <tt>stat</tt>
from <tt>sys/stat.h</tt> would be in <tt>libc/sys/stat/stat.c</tt>.

The standard headers use a BSD-like scheme where <tt>sys/cdefs.h</tt> declares
a bunch of useful preprocessor macros meant for internal use by the standard
library. All the function prototypes are wrapped in
<tt>extern "C" {</tt> and <tt>}</tt> such that C++ code can correctly
link against libc (as libc doesn't use C++ linkage). Note also how the compiler
provides the internal keyword __restrict unconditionally (even in C89) mode,
which is useful for adding the restrict keyword to function prototypes even when
compiling code in pre-C99 or C++ mode.

The special <tt>__is_libc</tt> macro lets the source code detect whether
it is part of the libc and <tt>__is_libk</tt> lets the source code detect
whether it's part of the libk binary.

This example comes with a small number of standard functions that serve as
examples and serve to satisfy ABI requirements. Note that the <tt>printf</tt>
function included is very minimal and intentionally doesn't handle most common
features.

== Source Code ==

<!---

Greetings, editors! It's not desirable to have parts of this wiki not actually be on this wiki, e.g. a code dump hosted on another service. To mitigate this, the source code is hosted here primarily. It's inconvenient and error-prone to manually copy it, though, so I've set up a git repository that people can easily clone. If you make any changes to the article, ping me, I'll update the git repository ASAP and update the git revision hash below. If you wish to pre-empt control (perhaps if I become unavailable) you can simply fork the git repository and update the links accordingly. --~~~~

--->

You can easily download the source code using [[Git]] from the [https://gitlab.com/sortie/meaty-skeleton Meaty Skeleton Git repository]. This is preferable to doing a manual error-prone copy, as you may make a mistake or whitespace may get garbled due to bugs in our syntax highlighting. To clone the git repository, do:

<syntaxhighlight lang="bash">
git clone https://gitlab.com/sortie/meaty-skeleton.git
</syntaxhighlight>

Check for differences between the git revision used in this article and what you cloned (empty output means there is no difference):
<syntaxhighlight lang="bash">
git diff 084d1624bedaa9f9e395f055c6bd99299bd97f58..master
</syntaxhighlight>

Operating systems development is about being an expert. Take the time to read the code carefully through and understand it. Please seek further information and help if you don't understand aspects of it. This code is minimal and almost everything is done deliberately, often to pre-emptively solve future problems.

=== kernel ===

==== kernel/include/kernel/tty.h ====

<syntaxhighlight lang="c">
#ifndef _KERNEL_TTY_H
#define _KERNEL_TTY_H

#include <stddef.h>

void terminal_initialize(void);
void terminal_putchar(char c);
void terminal_write(const char* data, size_t size);
void terminal_writestring(const char* data);

#endif
</syntaxhighlight>

==== kernel/Makefile ====

<syntaxhighlight lang="make">
DEFAULT_HOST!=../default-host.sh
HOST?=DEFAULT_HOST
HOSTARCH!=../target-triplet-to-arch.sh $(HOST)

CFLAGS?=-O2 -g
CPPFLAGS?=
LDFLAGS?=
LIBS?=

DESTDIR?=
PREFIX?=/usr/local
EXEC_PREFIX?=$(PREFIX)
BOOTDIR?=$(EXEC_PREFIX)/boot
INCLUDEDIR?=$(PREFIX)/include

CFLAGS:=$(CFLAGS) -ffreestanding -Wall -Wextra
CPPFLAGS:=$(CPPFLAGS) -D__is_kernel -Iinclude
LDFLAGS:=$(LDFLAGS)
LIBS:=$(LIBS) -nostdlib -lk -lgcc

ARCHDIR=arch/$(HOSTARCH)

include $(ARCHDIR)/make.config

CFLAGS:=$(CFLAGS) $(KERNEL_ARCH_CFLAGS)
CPPFLAGS:=$(CPPFLAGS) $(KERNEL_ARCH_CPPFLAGS)
LDFLAGS:=$(LDFLAGS) $(KERNEL_ARCH_LDFLAGS)
LIBS:=$(LIBS) $(KERNEL_ARCH_LIBS)

KERNEL_OBJS=\
$(KERNEL_ARCH_OBJS) \
kernel/kernel.o \

OBJS=\
$(ARCHDIR)/crti.o \
$(ARCHDIR)/crtbegin.o \
$(KERNEL_OBJS) \
$(ARCHDIR)/crtend.o \
$(ARCHDIR)/crtn.o \

LINK_LIST=\
$(LDFLAGS) \
$(ARCHDIR)/crti.o \
$(ARCHDIR)/crtbegin.o \
$(KERNEL_OBJS) \
$(LIBS) \
$(ARCHDIR)/crtend.o \
$(ARCHDIR)/crtn.o \

.PHONY: all clean install install-headers install-kernel
.SUFFIXES: .o .c .S

all: myos.kernel

myos.kernel: $(OBJS) $(ARCHDIR)/linker.ld
	$(CC) -T $(ARCHDIR)/linker.ld -o $@ $(CFLAGS) $(LINK_LIST)
	grub-file --is-x86-multiboot myos.kernel

$(ARCHDIR)/crtbegin.o $(ARCHDIR)/crtend.o:
	OBJ=`$(CC) $(CFLAGS) $(LDFLAGS) -print-file-name=$(@F)` && cp "$$OBJ" $@

.c.o:
	$(CC) -MD -c $< -o $@ -std=gnu11 $(CFLAGS) $(CPPFLAGS)

.S.o:
	$(CC) -MD -c $< -o $@ $(CFLAGS) $(CPPFLAGS)

clean:
	rm -f myos.kernel
	rm -f $(OBJS) *.o */*.o */*/*.o
	rm -f $(OBJS:.o=.d) *.d */*.d */*/*.d

install: install-headers install-kernel

install-headers:
	mkdir -p $(DESTDIR)$(INCLUDEDIR)
	cp -R --preserve=timestamps include/. $(DESTDIR)$(INCLUDEDIR)/.

install-kernel: myos.kernel
	mkdir -p $(DESTDIR)$(BOOTDIR)
	cp myos.kernel $(DESTDIR)$(BOOTDIR)

-include $(OBJS:.o=.d)
</syntaxhighlight>

==== kernel/kernel/kernel.c ====

<syntaxhighlight lang="c">
#include <stdio.h>

#include <kernel/tty.h>

void kernel_main(void) {
	terminal_initialize();
	printf("Hello, kernel World!\n");
}
</syntaxhighlight>

==== kernel/arch/i386/tty.c ====

<syntaxhighlight lang="c">
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include <kernel/tty.h>

#include "vga.h"

static const size_t VGA_WIDTH = 80;
static const size_t VGA_HEIGHT = 25;
static uint16_t* const VGA_MEMORY = (uint16_t*) 0xB8000;

static size_t terminal_row;
static size_t terminal_column;
static uint8_t terminal_color;
static uint16_t* terminal_buffer;

void terminal_initialize(void) {
	terminal_row = 0;
	terminal_column = 0;
	terminal_color = vga_entry_color(VGA_COLOR_LIGHT_GREY, VGA_COLOR_BLACK);
	terminal_buffer = VGA_MEMORY;
	for (size_t y = 0; y < VGA_HEIGHT; y++) {
		for (size_t x = 0; x < VGA_WIDTH; x++) {
			const size_t index = y * VGA_WIDTH + x;
			terminal_buffer[index] = vga_entry(' ', terminal_color);
		}
	}
}

void terminal_setcolor(uint8_t color) {
	terminal_color = color;
}

void terminal_putentryat(unsigned char c, uint8_t color, size_t x, size_t y) {
	const size_t index = y * VGA_WIDTH + x;
	terminal_buffer[index] = vga_entry(c, color);
}

void terminal_putchar(char c) {
	unsigned char uc = c;
	terminal_putentryat(uc, terminal_color, terminal_column, terminal_row);
	if (++terminal_column == VGA_WIDTH) {
		terminal_column = 0;
		if (++terminal_row == VGA_HEIGHT)
			terminal_row = 0;
	}
}

void terminal_write(const char* data, size_t size) {
	for (size_t i = 0; i < size; i++)
		terminal_putchar(data[i]);
}

void terminal_writestring(const char* data) {
	terminal_write(data, strlen(data));
}
</syntaxhighlight>

==== kernel/arch/i386/crtn.S ====

<pre>
.section .init
	/* gcc will nicely put the contents of crtend.o's .init section here. */
	popl %ebp
	ret

.section .fini
	/* gcc will nicely put the contents of crtend.o's .fini section here. */
	popl %ebp
	ret
</pre>

==== kernel/arch/i386/vga.h ====

<syntaxhighlight lang="c">
#ifndef ARCH_I386_VGA_H
#define ARCH_I386_VGA_H

#include <stdint.h>

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

static inline uint8_t vga_entry_color(enum vga_color fg, enum vga_color bg) {
	return fg | bg << 4;
}

static inline uint16_t vga_entry(unsigned char uc, uint8_t color) {
	return (uint16_t) uc | (uint16_t) color << 8;
}

#endif
</syntaxhighlight>

==== kernel/arch/i386/make.config ====

<syntaxhighlight lang="make">
KERNEL_ARCH_CFLAGS=
KERNEL_ARCH_CPPFLAGS=
KERNEL_ARCH_LDFLAGS=
KERNEL_ARCH_LIBS=

KERNEL_ARCH_OBJS=\
$(ARCHDIR)/boot.o \
$(ARCHDIR)/tty.o \
</syntaxhighlight>

==== kernel/arch/i386/crti.S ====

<pre>
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

==== kernel/arch/i386/linker.ld ====

<pre>
/* The bootloader will look at this image and start execution at the symbol
   designated at the entry point. */
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

	/* The compiler may produce other sections, put them in the proper place in
	   in this file, if you'd like to include them in the final kernel. */
}
</pre>

==== kernel/arch/i386/boot.S ====

<pre>
# Declare constants for the multiboot header.
.set ALIGN,    1<<0             # align loaded modules on page boundaries
.set MEMINFO,  1<<1             # provide memory map
.set FLAGS,    ALIGN | MEMINFO  # this is the Multiboot 'flag' field
.set MAGIC,    0x1BADB002       # 'magic number' lets bootloader find the header
.set CHECKSUM, -(MAGIC + FLAGS) # checksum of above, to prove we are multiboot

# Declare a header as in the Multiboot Standard.
.section .multiboot
.align 4
.long MAGIC
.long FLAGS
.long CHECKSUM

# Reserve a stack for the initial thread.
.section .bss
.align 16
stack_bottom:
.skip 16384 # 16 KiB
stack_top:

# The kernel entry point.
.section .text
.global _start
.type _start, @function
_start:
	movl $stack_top, %esp

	# Call the global constructors.
	call _init

	# Transfer control to the main kernel.
	call kernel_main

	# Hang if kernel_main unexpectedly returns.
	cli
1:	hlt
	jmp 1b
.size _start, . - _start
</pre>

==== kernel/.gitignore ====

<pre>
*.d
*.kernel
*.o
</pre>

=== libc and libk ===

==== libc/include/string.h ====

<syntaxhighlight lang="c">
#ifndef _STRING_H
#define _STRING_H 1

#include <sys/cdefs.h>

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

int memcmp(const void*, const void*, size_t);
void* memcpy(void* __restrict, const void* __restrict, size_t);
void* memmove(void*, const void*, size_t);
void* memset(void*, int, size_t);
size_t strlen(const char*);

#ifdef __cplusplus
}
#endif

#endif
</syntaxhighlight>

==== libc/include/stdio.h ====

<syntaxhighlight lang="c">
#ifndef _STDIO_H
#define _STDIO_H 1

#include <sys/cdefs.h>

#define EOF (-1)

#ifdef __cplusplus
extern "C" {
#endif

int printf(const char* __restrict, ...);
int putchar(int);
int puts(const char*);

#ifdef __cplusplus
}
#endif

#endif
</syntaxhighlight>

==== libc/include/sys/cdefs.h ====

<syntaxhighlight lang="c">
#ifndef _SYS_CDEFS_H
#define _SYS_CDEFS_H 1

#define __myos_libc 1

#endif
</syntaxhighlight>

==== libc/include/stdlib.h ====

<syntaxhighlight lang="c">
#ifndef _STDLIB_H
#define _STDLIB_H 1

#include <sys/cdefs.h>

#ifdef __cplusplus
extern "C" {
#endif

__attribute__((__noreturn__))
void abort(void);

#ifdef __cplusplus
}
#endif

#endif
</syntaxhighlight>

==== libc/Makefile ====

<syntaxhighlight lang="make">
DEFAULT_HOST!=../default-host.sh
HOST?=DEFAULT_HOST
HOSTARCH!=../target-triplet-to-arch.sh $(HOST)

CFLAGS?=-O2 -g
CPPFLAGS?=
LDFLAGS?=
LIBS?=

DESTDIR?=
PREFIX?=/usr/local
EXEC_PREFIX?=$(PREFIX)
INCLUDEDIR?=$(PREFIX)/include
LIBDIR?=$(EXEC_PREFIX)/lib

CFLAGS:=$(CFLAGS) -ffreestanding -Wall -Wextra
CPPFLAGS:=$(CPPFLAGS) -D__is_libc -Iinclude
LIBK_CFLAGS:=$(CFLAGS)
LIBK_CPPFLAGS:=$(CPPFLAGS) -D__is_libk

ARCHDIR=arch/$(HOSTARCH)

include $(ARCHDIR)/make.config

CFLAGS:=$(CFLAGS) $(ARCH_CFLAGS)
CPPFLAGS:=$(CPPFLAGS) $(ARCH_CPPFLAGS)
LIBK_CFLAGS:=$(LIBK_CFLAGS) $(KERNEL_ARCH_CFLAGS)
LIBK_CPPFLAGS:=$(LIBK_CPPFLAGS) $(KERNEL_ARCH_CPPFLAGS)

FREEOBJS=\
$(ARCH_FREEOBJS) \
stdio/printf.o \
stdio/putchar.o \
stdio/puts.o \
stdlib/abort.o \
string/memcmp.o \
string/memcpy.o \
string/memmove.o \
string/memset.o \
string/strlen.o \

HOSTEDOBJS=\
$(ARCH_HOSTEDOBJS) \

OBJS=\
$(FREEOBJS) \
$(HOSTEDOBJS) \

LIBK_OBJS=$(FREEOBJS:.o=.libk.o)

#BINARIES=libc.a libk.a # Not ready for libc yet.
BINARIES=libk.a

.PHONY: all clean install install-headers install-libs
.SUFFIXES: .o .libk.o .c .S

all: $(BINARIES)

libc.a: $(OBJS)
	$(AR) rcs $@ $(OBJS)

libk.a: $(LIBK_OBJS)
	$(AR) rcs $@ $(LIBK_OBJS)

.c.o:
	$(CC) -MD -c $< -o $@ -std=gnu11 $(CFLAGS) $(CPPFLAGS)

.S.o:
	$(CC) -MD -c $< -o $@ $(CFLAGS) $(CPPFLAGS)

.c.libk.o:
	$(CC) -MD -c $< -o $@ -std=gnu11 $(LIBK_CFLAGS) $(LIBK_CPPFLAGS)

.S.libk.o:
	$(CC) -MD -c $< -o $@ $(LIBK_CFLAGS) $(LIBK_CPPFLAGS)

clean:
	rm -f $(BINARIES) *.a
	rm -f $(OBJS) $(LIBK_OBJS) *.o */*.o */*/*.o
	rm -f $(OBJS:.o=.d) $(LIBK_OBJS:.o=.d) *.d */*.d */*/*.d

install: install-headers install-libs

install-headers:
	mkdir -p $(DESTDIR)$(INCLUDEDIR)
	cp -R --preserve=timestamps include/. $(DESTDIR)$(INCLUDEDIR)/.

install-libs: $(BINARIES)
	mkdir -p $(DESTDIR)$(LIBDIR)
	cp $(BINARIES) $(DESTDIR)$(LIBDIR)

-include $(OBJS:.o=.d)
-include $(LIBK_OBJS:.o=.d)
</syntaxhighlight>

==== libc/stdlib/abort.c ====

<syntaxhighlight lang="c">
#include <stdio.h>
#include <stdlib.h>

__attribute__((__noreturn__))
void abort(void) {
#if defined(__is_libk)
	// TODO: Add proper kernel panic.
	printf("kernel: panic: abort()\n");
        asm volatile("hlt");
#else
	// TODO: Abnormally terminate the process as if by SIGABRT.
	printf("abort()\n");
#endif
	while (1) { }
	__builtin_unreachable();
}
</syntaxhighlight>

==== libc/string/memmove.c ====

<syntaxhighlight lang="c">
#include <string.h>

void* memmove(void* dstptr, const void* srcptr, size_t size) {
	unsigned char* dst = (unsigned char*) dstptr;
	const unsigned char* src = (const unsigned char*) srcptr;
	if (dst < src) {
		for (size_t i = 0; i < size; i++)
			dst[i] = src[i];
	} else {
		for (size_t i = size; i != 0; i--)
			dst[i-1] = src[i-1];
	}
	return dstptr;
}
</syntaxhighlight>

==== libc/string/strlen.c ====

<syntaxhighlight lang="c">
#include <string.h>

size_t strlen(const char* str) {
	size_t len = 0;
	while (str[len])
		len++;
	return len;
}
</syntaxhighlight>

==== libc/string/memcmp.c ====

<syntaxhighlight lang="c">
#include <string.h>

int memcmp(const void* aptr, const void* bptr, size_t size) {
	const unsigned char* a = (const unsigned char*) aptr;
	const unsigned char* b = (const unsigned char*) bptr;
	for (size_t i = 0; i < size; i++) {
		if (a[i] < b[i])
			return -1;
		else if (b[i] < a[i])
			return 1;
	}
	return 0;
}
</syntaxhighlight>

==== libc/string/memset.c ====

<syntaxhighlight lang="c">
#include <string.h>

void* memset(void* bufptr, int value, size_t size) {
	unsigned char* buf = (unsigned char*) bufptr;
	for (size_t i = 0; i < size; i++)
		buf[i] = (unsigned char) value;
	return bufptr;
}
</syntaxhighlight>

==== libc/string/memcpy.c ====

<syntaxhighlight lang="c">
#include <string.h>

void* memcpy(void* restrict dstptr, const void* restrict srcptr, size_t size) {
	unsigned char* dst = (unsigned char*) dstptr;
	const unsigned char* src = (const unsigned char*) srcptr;
	for (size_t i = 0; i < size; i++)
		dst[i] = src[i];
	return dstptr;
}
</syntaxhighlight>

==== libc/stdio/puts.c ====

<syntaxhighlight lang="c">
#include <stdio.h>

int puts(const char* string) {
	return printf("%s\n", string);
}
</syntaxhighlight>

==== libc/stdio/putchar.c ====

<syntaxhighlight lang="c">
#include <stdio.h>

#if defined(__is_libk)
#include <kernel/tty.h>
#endif

int putchar(int ic) {
#if defined(__is_libk)
	char c = (char) ic;
	terminal_write(&c, sizeof(c));
#else
	// TODO: Implement stdio and the write system call.
#endif
	return ic;
}
</syntaxhighlight>

==== libc/stdio/printf.c ====

<syntaxhighlight lang="c">
#include <limits.h>
#include <stdbool.h>
#include <stdarg.h>
#include <stdio.h>
#include <string.h>

static bool print(const char* data, size_t length) {
	const unsigned char* bytes = (const unsigned char*) data;
	for (size_t i = 0; i < length; i++)
		if (putchar(bytes[i]) == EOF)
			return false;
	return true;
}

int printf(const char* restrict format, ...) {
	va_list parameters;
	va_start(parameters, format);

	int written = 0;

	while (*format != '\0') {
		size_t maxrem = INT_MAX - written;

		if (format[0] != '%' || format[1] == '%') {
			if (format[0] == '%')
				format++;
			size_t amount = 1;
			while (format[amount] && format[amount] != '%')
				amount++;
			if (maxrem < amount) {
				// TODO: Set errno to EOVERFLOW.
				return -1;
			}
			if (!print(format, amount))
				return -1;
			format += amount;
			written += amount;
			continue;
		}

		const char* format_begun_at = format++;

		if (*format == 'c') {
			format++;
			char c = (char) va_arg(parameters, int /* char promotes to int */);
			if (!maxrem) {
				// TODO: Set errno to EOVERFLOW.
				return -1;
			}
			if (!print(&c, sizeof(c)))
				return -1;
			written++;
		} else if (*format == 's') {
			format++;
			const char* str = va_arg(parameters, const char*);
			size_t len = strlen(str);
			if (maxrem < len) {
				// TODO: Set errno to EOVERFLOW.
				return -1;
			}
			if (!print(str, len))
				return -1;
			written += len;
		} else {
			format = format_begun_at;
			size_t len = strlen(format);
			if (maxrem < len) {
				// TODO: Set errno to EOVERFLOW.
				return -1;
			}
			if (!print(format, len))
				return -1;
			written += len;
			format += len;
		}
	}

	va_end(parameters);
	return written;
}
</syntaxhighlight>

==== libc/arch/i386/make.config ====

<syntaxhighlight lang="make">
ARCH_CFLAGS=
ARCH_CPPFLAGS=
KERNEL_ARCH_CFLAGS=
KERNEL_ARCH_CPPFLAGS=

ARCH_FREEOBJS=\

ARCH_HOSTEDOBJS=\
</syntaxhighlight>

==== libc/.gitignore ====

<pre>
*.a
*.d
*.o
</pre>

=== Miscellaneous ===

These files go into the root source directory.

==== build.sh ====

<syntaxhighlight lang="bash">
#!/bin/sh
set -e
. ./headers.sh

for PROJECT in $PROJECTS; do
  (cd $PROJECT && DESTDIR="$SYSROOT" $MAKE install)
done
</syntaxhighlight>

You should make this executable script executable by running:
<syntaxhighlight lang="bash">
chmod +x build.sh
</syntaxhighlight>

==== clean.sh ====

<syntaxhighlight lang="bash">
#!/bin/sh
set -e
. ./config.sh

for PROJECT in $PROJECTS; do
  (cd $PROJECT && $MAKE clean)
done

rm -rf sysroot
rm -rf isodir
rm -rf myos.iso
</syntaxhighlight>

You should make this executable script executable by running:
<syntaxhighlight lang="bash">
chmod +x clean.sh
</syntaxhighlight>

==== config.sh ====

<syntaxhighlight lang="bash">
SYSTEM_HEADER_PROJECTS="libc kernel"
PROJECTS="libc kernel"

export MAKE=${MAKE:-make}
export HOST=${HOST:-$(./default-host.sh)}

export AR=${HOST}-ar
export AS=${HOST}-as
export CC=${HOST}-gcc

export PREFIX=/usr
export EXEC_PREFIX=$PREFIX
export BOOTDIR=/boot
export LIBDIR=$EXEC_PREFIX/lib
export INCLUDEDIR=$PREFIX/include

export CFLAGS='-O2 -g'
export CPPFLAGS=''

# Configure the cross-compiler to use the desired system root.
export SYSROOT="$(pwd)/sysroot"
export CC="$CC --sysroot=$SYSROOT"

# Work around that the -elf gcc targets doesn't have a system include directory
# because it was configured with --without-headers rather than --with-sysroot.
if echo "$HOST" | grep -Eq -- '-elf($|-)'; then
  export CC="$CC -isystem=$INCLUDEDIR"
fi
</syntaxhighlight>

==== default-host.sh ====

<syntaxhighlight lang="bash">
#!/bin/sh
echo i686-elf
</syntaxhighlight>

You should make this executable script executable by running:
<syntaxhighlight lang="bash">
chmod +x default-host.sh
</syntaxhighlight>

==== headers.sh ====

<syntaxhighlight lang="bash">
#!/bin/sh
set -e
. ./config.sh

mkdir -p "$SYSROOT"

for PROJECT in $SYSTEM_HEADER_PROJECTS; do
  (cd $PROJECT && DESTDIR="$SYSROOT" $MAKE install-headers)
done
</syntaxhighlight>

You should make this executable script executable by running:
<syntaxhighlight lang="bash">
chmod +x headers.sh
</syntaxhighlight>

==== iso.sh ====

<syntaxhighlight lang="bash">
#!/bin/sh
set -e
. ./build.sh

mkdir -p isodir
mkdir -p isodir/boot
mkdir -p isodir/boot/grub

cp sysroot/boot/myos.kernel isodir/boot/myos.kernel
cat > isodir/boot/grub/grub.cfg << EOF
menuentry "myos" {
	multiboot /boot/myos.kernel
}
EOF
grub-mkrescue -o myos.iso isodir
</syntaxhighlight>

You should make this executable script executable by running:
<syntaxhighlight lang="bash">
chmod +x iso.sh
</syntaxhighlight>

==== qemu.sh ====

<syntaxhighlight lang="bash">
#!/bin/sh
set -e
. ./iso.sh

qemu-system-$(./target-triplet-to-arch.sh $HOST) -cdrom myos.iso
</syntaxhighlight>

You should make this executable script executable by running:
<syntaxhighlight lang="bash">
chmod +x qemu.sh
</syntaxhighlight>

==== target-triplet-to-arch.sh ====

<syntaxhighlight lang="bash">
#!/bin/sh
if echo "$1" | grep -Eq 'i[[:digit:]]86-'; then
  echo i386
else
  echo "$1" | grep -Eo '^[[:alnum:]_]*'
fi
</syntaxhighlight>

You should make this executable script executable by running:
<syntaxhighlight lang="bash">
chmod +x target-triplet-to-arch.sh
</syntaxhighlight>

==== .gitignore ====

<pre>
*.iso
isodir
sysroot
</pre>

== Cross-Compiling the Operating System ==

The system is cross-compiled in the same manner as [[Bare Bones]], though with
the complexity of having a system root with the final system and using a libk.
In this example, we elected to use shell scripts to to the top-level build
process, though you could possibly also use a makefile for that or a wholly
different build system. Though, assuming this setup works for you, you can clean
the source tree by invoking:

<syntaxhighlight lang="bash">
./clean.sh
</syntaxhighlight>

You can install all the system headers into the system root without relying on
the compiler at all, which will be useful later on when switching to a
[[Hosted GCC Cross-Compiler]], by invoking:

<syntaxhighlight lang="bash">
./headers.sh
</syntaxhighlight>

You can build a bootable cdrom image of the operating system by invoking:

<syntaxhighlight lang="bash">
./iso.sh
</syntaxhighlight>

It's probably a good idea to create a quick ''build-and-then-launch'' short-cut
like used in this example to run the system in your favorite emulator quickly:

<syntaxhighlight lang="bash">
./qemu.sh
</syntaxhighlight>

== Troubleshooting ==

If you receive odd errors during the build, you may have made a mistake during manual copying, perhaps missed a file, forgot to make a file executable, or bugs in the highlighting software we use cause unintended whitespace to appear. Perform a git repository clone as described above, and use that code instead, or compare the two directory trees with the <tt>diff(1)</tt> diff command line utility. If you made personal changes to the code, those may be at fault.

== Moving Forward ==

You should adapt this template to your needs. There's a number of things you
should consider doing now:

=== Renaming MyOS to YourOS ===

Certainly you wish to name your operating system after your favorite flower,
hometown, boolean value, or whatever marketing told you. Do a search and replace
that replaces <tt>myos</tt> with whatever you wish to call it. Keep in mind that
the name is deliberately lower-case in a few places for technical reasons.

=== Improving the Build System ===
{{Main|Hard Build System}}

It is probably worth improving the build system. For instance, it could be
useful if <tt>build.sh</tt> accepted command-line options, or perhaps if it used
<tt>make</tt>'s important <tt>-j</tt> option for concurrent builds.

It's worth considering how contributors will build your operating system. It's an easy trap to fall into thinking you can make super script that does everything. This will end up complex and insufficiently flexible; or it will be flexible and even more complex. It's better to document what the user should do to prepare a cross toolchain and what prerequisite programs to install. This tutorial shows an example hard build system that merely builds the operating system. You can complete it by documenting how to build a cross-compiler and how to use it.

=== Stack Smash Protector ===
{{Main|Stack Smashing Protector}}

Early is not too soon to think about security and robustness. You can take advantage of the optional stack smash protector offered by modern compilers that detect stack buffer overruns rather than behaving unexpectedly (or nothing happening, if unlucky).

=== Going Further ===
{{Main|Going Further on x86}}
This guide is meant as an overview of what to do, so you have a kernel ready for more features, without actually redesigning it radically when adding them.

=== User-Space ===

A later tutorial in this series will extend this template with a proper
user-space and an [[OS Specific Toolchain]] that fully utilizes the system root.

== Forum Posts ==

* [[topic:36584|A link error found & fixed]]

[[Category:Tutorials]]
[[Category:C]]
````

<a id="w003449"></a>

## W003449 — UEFI ISO Bare Bones

Bron-ID: W003449
Namespace: Main (0)
Revisie: 25174 | Datum: 2020-09-17T15:46:02Z
Bron: https://wiki.osdev.org/UEFI_ISO_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=UEFI_ISO_Bare_Bones&oldid=25174
Doorverwijzing: UEFI App Bare Bones

````text
#REDIRECT [[UEFI App Bare Bones#Creating and running the CD image]]
````

<a id="w003450"></a>

## W003450 — ARM TargetingMultipleDevices

Bron-ID: W003450
Namespace: Main (0)
Revisie: 20525 | Datum: 2017-02-20T14:13:49Z
Bron: https://wiki.osdev.org/ARM_TargetingMultipleDevices
Revisie-URL: https://wiki.osdev.org/index.php?title=ARM_TargetingMultipleDevices&oldid=20525

````text
= Targeting Multiple ARM Based Devices =
This page was written to help shed a little light on the issues of targeting multiple ARM based devices. I hope that this page does not make you feel like this task is impossible, but I think that you should be prepared with the reality of the task ahead of you. There are software out there that target many devices such as routers and phones, but the maintainers and authors of this software have more problems to overcome than software targeting a standard PC type x86/x64 device with BIOS and bus technologies with plug and play support.

A many readers of this page are likely coming from a background of the x86/x64 platform and will be accustomed to having a BIOS and the lack of software, hardware, and physical limitations to replacing storage devices and manipulating these storage devices. This is partially due to the small size of many ARM devices which creates physical limitations. The lack of BIOS type services and other architectures such as plug and play technologies which are not present on an ARM device. And, also software limitations such as image verification problems that prevent unauthorized images from being uploaded or placed on storage mediums in the device or even the absence of methods to replace images and system files on the device.

== ARM Devices ==
The ARM processor is used in smartphones, laptops, tablet and notepad computers, and other embedded systems. 
== Booting Of ARM Processor ==
It boots by starting at some specific address. This address is usually located on ROM/FLASH that has been 
mapped onto the memory bus. This piece of code starts the booting process. In some small device is may be the only
code that is executed. On more complex devices such as tablets and smart phones for instance it simply boot straps
the system to load from other storage mediums such SD cards both internal and external for one example.
== No BIOS On ARM ==
There is no industry standard software for booting an ARM based system such as BIOS like on the x86/x64 platforms. There
could exist software that is similar or just like BIOS but this will likely be customized and in many cases not exist at
all. So to work with hardware your going to have to communicate directly with it from the beginning.

== Three Major Problems ==
The ability to develop system software to target a broad range of ARM devices is problematic in three different ways:
# Different hardware and memory with no industry standard enumeration or inspection functionality
# ROM/FLASH protection mechanisms enforced in hardware and/or in software to prevent unauthorized code from being flashed onto the boot ROM/FLASH memory
# Physical and software (but generally not hardware) protection mechanisms to replace secondary boot image (or kernel)

== Different Hardware And Memory ==
A lot of ARM based devices have custom chips which contain hardware that will vary from chip to chip, device to device, and board to board. This is exactly the same as a device built on the standard x86/x64 platform except there is no industry standard method to enumerate hardware on an ARM based device like what exists on the x86/x64. A good example is the amount of memory which is normally different from device to device. This means that your system software will have to know ahead of time how much memory is on the device, where the memory is located, and the different types of memory on the device. It will also need to know what hardware is on the device there is no industry standard method to request a list of hardware and where it is located on the board. There is no plug and play support for devices on most ARM boards.

//It is possible that a range of ARM based devices may support a field that identifies the amount of memory in the device, but there is no industry standard location of this field or it's format, and it may or may not exist on one of your target devices.//

== ROM/FLASH Protection Mechanisms ==
Most devices employ at least a software security mechanism to prevent you from uploading custom ROM images to the device that are not authorized. If all devices used the exact same mechanism then this would not be a problem because you could employ a one solution for all, but in reality all devices have different mechanisms. Some may even use some type of hardware protection requiring you to interface electrically with the device to enable flashing of a custom ROM.

== Secondary Image (Kernel) Protection Mechanisms ==
Not all devices have a kernel, but many advanced devices do. And, some devices just have a secondary boot image which was too large to fit onto the boot ROM/FLASH. This is generally
stored in secondary storage which is loaded into memory by the boot code on the ROM/FLASH. Since the boot code is normally quite custom to the device you might find a better route to developing for a wide range of devices by targeting the secondary image.

There exist two major issues:
* physical protection mechanisms or access issues
* software protection mechanisms or access issues 

=== Physical Protection Mechanism Or Access Issues ===
You may not be able to physically access the storage in such a way that would make deployment a reality for many devices for the users that wish to install your system software. If the storage
was accessible for example by USB cable and allowed you to write a new secondary boot image then this would be a potential device that you could support. However, if you required the user to electrically interface with the storage by soldering wires or performing some other task of advanced skill then you are going to have issues deploying your software to casual and basic users of the device. 

=== Software Protection Mechanism Or Access Issues ===
Some devices may either not provide a mechanism to replace the secondary boot image, or actually protect it from being replaced in much the same way the boot ROM may be protected such as requiring the image to have some special code in it at a specific location, or even be signed using public key cryptography. Of course if you could circumvent this then you could deploy your software to a wide range of devices but consideration must be taken that many devices may have completely different methods for verification of the image to be replaced. So you would have to have some method of testing each device to ensure that your methods works and each method would have to be maintained adding to a lot of overhead.

//However, this is quite possible. You just must be aware of these issues.//

[[Category:ARM]]
````

<a id="w003451"></a>

## W003451 — User:Pancakes/SimpleHeapImplementation

Bron-ID: W003451
Namespace: User (2)
Revisie: 16351 | Datum: 2014-05-07T18:54:02Z
Bron: https://wiki.osdev.org/User:Pancakes/SimpleHeapImplementation
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APancakes%2FSimpleHeapImplementation&oldid=16351

````text
== Simple Heap Implementation ==
There are a few simple heap implementations for you to examine. I want to come back and
write a little tutorial on each page to kind of walk the reader through how they work. 

If you want to copy them straight out and use them in your kernel then that is fine. I put these
up here so someone who might not be that interested in implementing a heap can get it done and
move on to better things.

The bitmap implementation has been fairly well optimized. I can not find any major performance improvements to make. It still needs some optimization for multiple blocks such as moving full blocks to the end of the linked list maybe. Also, the entry could perhaps use a double chain where one chain links free entries and the other links used entries therefore making allocation quicker. At the moment it just jumps through all of them. Also, deallocation could be faster on the entry implementation.

I have tested both implementations quite well for corruption internally, and checked a 32-bit block at the beginning and end of the blocks for under or over runs and so far have found no corruption even after a long period of running.

''If you find any bugs, problems, or have success using them just let me know at [mailto:kmcg3413@gmail.com kmcg3413@gmail.com].''

== Simulation ==
I wrote a page that lets you simulate a bitmap heap with a view of memory and play back of memory accesses. It relates directly to the bitmap algorithm except it uses only 16-bit fields. You can find it
at [http://kmcg3413.net/jsbmheap.htm http://kmcg3413.net/jsbmheap.htm]. It also contains a link back to the bitmap implementation page so you can reference the actual code and structures.

== Implementations ==
These are the implementations. I recommend the bitmap because of it's reliability, decent performance, simplicity, and small code size make it a very attractive choice for a beginning heap.

{| class="wikitable"
|-
! Page
! Brief Description
|-
| [[User:Pancakes/BitmapHeapImplementation|Bitmap Based]]
| A very simple and decently performing implementation.
|-
| [[User:Pancakes/EntryBasedHeapImplementation|Entry Based]] 
| Not very good but shows an alternative way.
|-
| [[User:Pancakes/StackBasedHeapImplementation|Stack Based]]
| Very fast but only supports one block size, and if data is smaller it wastes memory depending on stack entry size. Could be chained together to provide different sizes like a SLAB allocator.
|-
| [[User:Pancakes/BitmapHeapImplementationEnhanced|Bitmap Based With Enhanced Functionality]]
| The bitmap implementation but supports managing physical memory and provides data aligned on arbitrary boundaries.
|-
| [[User:mrvn/LinkedListBucketHeapImplementation|Linked List Bucket Heap]]
| A very fast general purpose allocator. It can work well with large or small allocations.
|}

==== Performance ====

The ''Enhanced Bitmap'' is noted as ''bm'' in green, and was initialized with 16-byte blocks. The ''Linked List Bucket Heap'' is noted as ''mrvn'' and is shown in light red. As you can see the performance of the bitmap implementation decreases as allocation size increases. The darkish purple is the entry based. The numbers across the bottom denote maximum allocation size. For example 256 means allocations were between 1 and 256 while 4096 means allocations where between 1 byte and 4096 bytes. Each implementation was run for a lengthy period of time and had allocations and frees made randomly. Each implementation was allowed to use more memory if it ran out.

<html><body><svg height="300" width="700">
<text style="font-size: 8pt;" x="0" y="271.4770860505417" fill="rgb(0,0,0)">eb</text>
<text style="font-size: 8pt;" x="0" y="260.5681727301949" fill="rgb(0,0,0)">bm</text>
<text style="font-size: 8pt;" x="0" y="201.04345987460465" fill="rgb(0,0,0)">sb</text>
<text style="font-size: 8pt;" x="0" y="238.8811744121518" fill="rgb(0,0,0)">mrvn</text>
<line x1="40" y1="271.4770860505417" x2="39.0" y2="271.4770860505417" style="stroke:rgb(153,0,255);stroke-width:2"/>
<text style="font-size: 8pt;" x="40" y="271.4770860505417" fill="rgb(153,0,255)">0.3</text>
<line x1="40" y1="260.5681727301949" x2="39.0" y2="260.5681727301949" style="stroke:rgb(153,255,153);stroke-width:2"/>
<text style="font-size: 8pt;" x="40" y="260.5681727301949" fill="rgb(153,255,153)">0.4</text>
<line x1="40" y1="201.04345987460465" x2="39.0" y2="201.04345987460465" style="stroke:rgb(0,255,255);stroke-width:2"/>
<text style="font-size: 8pt;" x="40" y="201.04345987460465" fill="rgb(0,255,255)">1.1</text>
<line x1="40" y1="238.8811744121518" x2="39.0" y2="238.8811744121518" style="stroke:rgb(255,153,153);stroke-width:2"/>
<text style="font-size: 8pt;" x="40" y="238.8811744121518" fill="rgb(255,153,153)">0.7</text>
<text style="font-size: 8pt;" x="40" y="300" fill="rgb(0,0,0)">16</text>
<line x1="80" y1="271.59097505774" x2="40" y2="271.4770860505417" style="stroke:rgb(153,0,255);stroke-width:2"/>
<text style="font-size: 8pt;" x="80" y="271.59097505774" fill="rgb(153,0,255)">0.3</text>
<line x1="80" y1="266.1457872407004" x2="40" y2="260.5681727301949" style="stroke:rgb(153,255,153);stroke-width:2"/>
<text style="font-size: 8pt;" x="80" y="266.1457872407004" fill="rgb(153,255,153)">0.3</text>
<line x1="80" y1="197.9335788586706" x2="40" y2="201.04345987460465" style="stroke:rgb(0,255,255);stroke-width:2"/>
<text style="font-size: 8pt;" x="80" y="197.9335788586706" fill="rgb(0,255,255)">1.1</text>
<line x1="80" y1="243.85223645946513" x2="40" y2="238.8811744121518" style="stroke:rgb(255,153,153);stroke-width:2"/>
<text style="font-size: 8pt;" x="80" y="243.85223645946513" fill="rgb(255,153,153)">0.6</text>
<text style="font-size: 8pt;" x="80" y="300" fill="rgb(0,0,0)">32</text>
<line x1="120" y1="273.3687218988938" x2="80" y2="271.59097505774" style="stroke:rgb(153,0,255);stroke-width:2"/>
<text style="font-size: 8pt;" x="120" y="273.3687218988938" fill="rgb(153,0,255)">0.3</text>
<line x1="120" y1="273.2187003317216" x2="80" y2="266.1457872407004" style="stroke:rgb(153,255,153);stroke-width:2"/>
<text style="font-size: 8pt;" x="120" y="273.2187003317216" fill="rgb(153,255,153)">0.3</text>
<line x1="120" y1="184.0375802848887" x2="80" y2="197.9335788586706" style="stroke:rgb(0,255,255);stroke-width:2"/>
<text style="font-size: 8pt;" x="120" y="184.0375802848887" fill="rgb(0,255,255)">1.3</text>
<line x1="120" y1="247.36927461158245" x2="80" y2="243.85223645946513" style="stroke:rgb(255,153,153);stroke-width:2"/>
<text style="font-size: 8pt;" x="120" y="247.36927461158245" fill="rgb(255,153,153)">0.6</text>
<text style="font-size: 8pt;" x="120" y="300" fill="rgb(0,0,0)">64</text>
<line x1="160" y1="280.0267366728067" x2="120" y2="273.3687218988938" style="stroke:rgb(153,0,255);stroke-width:2"/>
<text style="font-size: 8pt;" x="160" y="280.0267366728067" fill="rgb(153,0,255)">0.2</text>
<line x1="160" y1="278.3306191945452" x2="120" y2="273.2187003317216" style="stroke:rgb(153,255,153);stroke-width:2"/>
<text style="font-size: 8pt;" x="160" y="278.3306191945452" fill="rgb(153,255,153)">0.2</text>
<line x1="160" y1="193.80312762756847" x2="120" y2="184.0375802848887" style="stroke:rgb(0,255,255);stroke-width:2"/>
<text style="font-size: 8pt;" x="160" y="193.80312762756847" fill="rgb(0,255,255)">1.2</text>
<line x1="160" y1="250.4264809169783" x2="120" y2="247.36927461158245" style="stroke:rgb(255,153,153);stroke-width:2"/>
<text style="font-size: 8pt;" x="160" y="250.4264809169783" fill="rgb(255,153,153)">0.5</text>
<text style="font-size: 8pt;" x="160" y="300" fill="rgb(0,0,0)">128</text>
<line x1="200" y1="283.2616949646197" x2="160" y2="280.0267366728067" style="stroke:rgb(153,0,255);stroke-width:2"/>
<text style="font-size: 8pt;" x="200" y="283.2616949646197" fill="rgb(153,0,255)">0.1</text>
<line x1="200" y1="283.15126703525505" x2="160" y2="278.3306191945452" style="stroke:rgb(153,255,153);stroke-width:2"/>
<text style="font-size: 8pt;" x="200" y="283.15126703525505" fill="rgb(153,255,153)">0.1</text>
<line x1="200" y1="179.8772379141531" x2="160" y2="193.80312762756847" style="stroke:rgb(0,255,255);stroke-width:2"/>
<text style="font-size: 8pt;" x="200" y="179.8772379141531" fill="rgb(0,255,255)">1.3</text>
<line x1="200" y1="247.57494334396884" x2="160" y2="250.4264809169783" style="stroke:rgb(255,153,153);stroke-width:2"/>
<text style="font-size: 8pt;" x="200" y="247.57494334396884" fill="rgb(255,153,153)">0.6</text>
<text style="font-size: 8pt;" x="200" y="300" fill="rgb(0,0,0)">256</text>
<line x1="240" y1="284.98919175692185" x2="200" y2="283.2616949646197" style="stroke:rgb(153,0,255);stroke-width:2"/>
<text style="font-size: 8pt;" x="240" y="284.98919175692185" fill="rgb(153,0,255)">0.1</text>
<line x1="240" y1="288.2320350909907" x2="200" y2="283.15126703525505" style="stroke:rgb(153,255,153);stroke-width:2"/>
<text style="font-size: 8pt;" x="240" y="288.2320350909907" fill="rgb(153,255,153)">0.1</text>
<line x1="240" y1="115.37561513335092" x2="200" y2="179.8772379141531" style="stroke:rgb(0,255,255);stroke-width:2"/>
<text style="font-size: 8pt;" x="240" y="115.37561513335092" fill="rgb(0,255,255)">2.1</text>
<line x1="240" y1="241.2029598079109" x2="200" y2="247.57494334396884" style="stroke:rgb(255,153,153);stroke-width:2"/>
<text style="font-size: 8pt;" x="240" y="241.2029598079109" fill="rgb(255,153,153)">0.6</text>
<text style="font-size: 8pt;" x="240" y="300" fill="rgb(0,0,0)">512</text>
<line x1="280" y1="287.3975513306974" x2="240" y2="284.98919175692185" style="stroke:rgb(153,0,255);stroke-width:2"/>
<text style="font-size: 8pt;" x="280" y="287.3975513306974" fill="rgb(153,0,255)">0.1</text>
<line x1="280" y1="293.3565931075834" x2="240" y2="288.2320350909907" style="stroke:rgb(153,255,153);stroke-width:2"/>
<text style="font-size: 8pt;" x="280" y="293.3565931075834" fill="rgb(153,255,153)">0.0</text>
<line x1="280" y1="93.56361255075586" x2="240" y2="115.37561513335092" style="stroke:rgb(0,255,255);stroke-width:2"/>
<text style="font-size: 8pt;" x="280" y="93.56361255075586" fill="rgb(0,255,255)">2.4</text>
<line x1="280" y1="243.0694390692868" x2="240" y2="241.2029598079109" style="stroke:rgb(255,153,153);stroke-width:2"/>
<text style="font-size: 8pt;" x="280" y="243.0694390692868" fill="rgb(255,153,153)">0.6</text>
<text style="font-size: 8pt;" x="280" y="300" fill="rgb(0,0,0)">1024</text>
<line x1="320" y1="289.8395296655668" x2="280" y2="287.3975513306974" style="stroke:rgb(153,0,255);stroke-width:2"/>
<text style="font-size: 8pt;" x="320" y="289.8395296655668" fill="rgb(153,0,255)">0.1</text>
<line x1="320" y1="291.4247206595326" x2="280" y2="293.3565931075834" style="stroke:rgb(153,255,153);stroke-width:2"/>
<text style="font-size: 8pt;" x="320" y="291.4247206595326" fill="rgb(153,255,153)">0.0</text>
<line x1="320" y1="91.37795492547124" x2="280" y2="93.56361255075586" style="stroke:rgb(0,255,255);stroke-width:2"/>
<text style="font-size: 8pt;" x="320" y="91.37795492547124" fill="rgb(0,255,255)">2.4</text>
<line x1="320" y1="240.37311671765553" x2="280" y2="243.0694390692868" style="stroke:rgb(255,153,153);stroke-width:2"/>
<text style="font-size: 8pt;" x="320" y="240.37311671765553" fill="rgb(255,153,153)">0.6</text>
<text style="font-size: 8pt;" x="320" y="300" fill="rgb(0,0,0)">2048</text>
<line x1="360" y1="291.82936406358374" x2="320" y2="289.8395296655668" style="stroke:rgb(153,0,255);stroke-width:2"/>
<text style="font-size: 8pt;" x="360" y="291.82936406358374" fill="rgb(153,0,255)">0.0</text>
<line x1="360" y1="291.685536015328" x2="320" y2="291.4247206595326" style="stroke:rgb(153,255,153);stroke-width:2"/>
<text style="font-size: 8pt;" x="360" y="291.685536015328" fill="rgb(153,255,153)">0.0</text>
<line x1="360" y1="84.43282646493722" x2="320" y2="91.37795492547124" style="stroke:rgb(0,255,255);stroke-width:2"/>
<text style="font-size: 8pt;" x="360" y="84.43282646493722" fill="rgb(0,255,255)">2.5</text>
<line x1="360" y1="242.36747130138255" x2="320" y2="240.37311671765553" style="stroke:rgb(255,153,153);stroke-width:2"/>
<text style="font-size: 8pt;" x="360" y="242.36747130138255" fill="rgb(255,153,153)">0.6</text>
<text style="font-size: 8pt;" x="360" y="300" fill="rgb(0,0,0)">4096</text>
<line x1="400" y1="293.64362664521053" x2="360" y2="291.82936406358374" style="stroke:rgb(153,0,255);stroke-width:2"/>
<text style="font-size: 8pt;" x="400" y="293.64362664521053" fill="rgb(153,0,255)">0.0</text>
<line x1="400" y1="291.0443879550437" x2="360" y2="291.685536015328" style="stroke:rgb(153,255,153);stroke-width:2"/>
<text style="font-size: 8pt;" x="400" y="291.0443879550437" fill="rgb(153,255,153)">0.1</text>
<line x1="400" y1="75.78456882308708" x2="360" y2="84.43282646493722" style="stroke:rgb(0,255,255);stroke-width:2"/>
<text style="font-size: 8pt;" x="400" y="75.78456882308708" fill="rgb(0,255,255)">2.6</text>
<line x1="400" y1="244.82608599404003" x2="360" y2="242.36747130138255" style="stroke:rgb(255,153,153);stroke-width:2"/>
<text style="font-size: 8pt;" x="400" y="244.82608599404003" fill="rgb(255,153,153)">0.6</text>
<text style="font-size: 8pt;" x="400" y="300" fill="rgb(0,0,0)">8192</text>
<line x1="440" y1="292.83027753490535" x2="400" y2="293.64362664521053" style="stroke:rgb(153,0,255);stroke-width:2"/>
<text style="font-size: 8pt;" x="440" y="292.83027753490535" fill="rgb(153,0,255)">0.0</text>
<line x1="440" y1="289.20005644117305" x2="400" y2="291.0443879550437" style="stroke:rgb(153,255,153);stroke-width:2"/>
<text style="font-size: 8pt;" x="440" y="289.20005644117305" fill="rgb(153,255,153)">0.1</text>
<line x1="440" y1="0.0" x2="400" y2="75.78456882308708" style="stroke:rgb(0,255,255);stroke-width:2"/>
<text style="font-size: 8pt;" x="440" y="0.0" fill="rgb(0,255,255)">3.4</text>
<line x1="440" y1="240.93068766785797" x2="400" y2="244.82608599404003" style="stroke:rgb(255,153,153);stroke-width:2"/>
<text style="font-size: 8pt;" x="440" y="240.93068766785797" fill="rgb(255,153,153)">0.6</text>
<text style="font-size: 8pt;" x="440" y="300" fill="rgb(0,0,0)">16384</text>
</svg></body></html>


The ''Linked List Bucket Heap'' is a great generic heap, but it is slightly more complicated and a little harder to follow code wise. The bitmap heap also consumes significant memory by meta-data of it's bitmap tables, but code wise is fairly straight forward and simple.

''The numbers represent the speed gain compared with the standard malloc and free provided by my C library on the machine I produced these numbers on. It represents how many times faster, and it combines the speed of both malloc and free for an overall speed. Some implements have ultra fast free like entry based and bitmap which beat all others, but their initial allocation speed is just too slow to make them fast.''
````

<a id="w003452"></a>

## W003452 — Heap

Bron-ID: W003452
Namespace: Main (0)
Revisie: 15620 | Datum: 2014-02-21T23:16:16Z
Bron: https://wiki.osdev.org/Heap
Revisie-URL: https://wiki.osdev.org/index.php?title=Heap&oldid=15620

````text
{{Stub}}
[[Category:Memory management]]

= Heap =
A heap is a vital component of both application programs and the kernel. It is also generally superseded by a higher level of memory management that deals with larger chunks of memory. For most operating systems memory will be allocated based on pages or other large chunks. A page on the X86 and X64 architectures is generally 4KB, but can be larger. However, for smaller allocations the entire page would be wasted. For example if you only needed 24 bytes and you allocate an entire 4KB page you will have wasted a lot of memory. So most applications and kernels will implemented a second memory management scheme that uses memory allocated in 4KB chunks (or larger) and break these strips of pages or individual pages into smaller parts as they are requested. 

=== Standard Library, Applications, And Kernel ===
Applications in most operating systems have their own heap implementation as a part of the standard library and do not use the implementation that the kernel uses. So therefore not only is the heap for the kernel separate from the heap of each application, but it may very well use a different heap implementation. Also, this allows each application to use it's own implementation depending on which standard library is was built against or with. However, there is no reason why a kernel could not provide heap services directly
to applications. It must be noted too that there is no way to enforce an application to use a specific heap or implementation because a clever programmer could simply use the memory allocated to supply it's own implementation.

''If you are looking for information about designing a memory system in regard to paging or handing out larger chunks of memory (such as 4KB) and aligned on a large boundaries then this is not the correct page. Try looking at [[Memory_management|Memory Management]].''

=== Tutorials/Implementations/Articles ===
{| class="wikitable"
|-
! Page
! Brief Description
|-
| [[User:Pancakes/SimpleHeapImplementation|Simple Heap Implementation]]
| A simple, portable, and fairly efficient memory wise implementation. 
|-
| [[Writing_a_memory_manager|Writing A Heap Implementation]]
| It's not that hard to implement a basic first fit MM... (continued).
|}
````

<a id="w003453"></a>

## W003453 — User:Pancakes/SimpleKernelModuleAttachment

Bron-ID: W003453
Namespace: User (2)
Revisie: 15622 | Datum: 2014-02-22T21:34:11Z
Bron: https://wiki.osdev.org/User:Pancakes/SimpleKernelModuleAttachment
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APancakes%2FSimpleKernelModuleAttachment&oldid=15622

````text
= Simple Kernel Module Attachment =

== What About GRUB ==
Most people are going to ask why not just use GRUB, and if you are working on a platform that supports it then this could be
a good idea. However, if you are on a platform that does not support GRUB or does not need it then you are going to have to
find an alternative way. And, your kernel might be cross-platform meaning it might be loaded by GRUB on one platform, and not
on the other. So to keep from writing complex code you might just bundle all the needed modules in the same file with the kernel.

''This method lets you bundle the modules in the kernel file, but also still allows your image to be loaded by GRUB or loaded from ROM on another platform.''

== Overview ==
Have you ever wanted a simple way to load files into memory with your kernel. Essentially, allowing to you strap on arbitrary drivers, data, or libraries which may be required to boot strap the system. Well, I came up with a decent method to do so that you might wish to employ on your own kernel or system software.

It requires the usage of Python and C/C++. Basically, the Python code writes the module to the end of the kernel image with a special
header. Once the kernel image is flashed onto the ROM, or loaded into memory it can scan it's self and enumerate these modules so that you can handle loading them or calling code in them.

{| class="wikitable"
|-
! Needed
! Description
|-
| GCC
| Or, a compiler/linker (C++) that has similar functionality; Or you can adapt the code too. I use some non-standard features such as ''__attribute__ ((section (".MODHDR")))''.
|-
| Python
| I use Python3, but the code may work on earlier versions. Also, you can code the Python part in any language you like. The point is for you to understand how it works.
|-
| Flat Binary Kernel
| If not your going to have to find a way to modify the sections and add extra data. There may be an easy way to append data to a section. But, having a flat binary format (or one that supports loading anything appended to the end) is of course the easiest.
|}

=== Example Enumeration Code ===
<pre>
        KATTMOD     *mod;

        /* each module is enumerated that is attached to the end of our image */
	for (mod = k_imgPkgGetFirstMod(); mod; mod = k_imgPkgGetNextMod(mod)) {
		k_printf("mod iface:%x\n", mod->interfaceID);
		k_printf("    sigA: %x\n", mod->signatureA);
		k_printf("    sigB: %x\n", mod->signatureB);
	}
</pre>

=== Example Calling A Procedure ===
The module here that I am calling a procedure in was compiled with position independent code. You have to make sure
that your executable module can handle being called in this way. If not or your compiler and platform can not handle
it you could instead use a object file format like ELF or PE* for example and write a loader to load and relocate them
in memory.

<pre>
typedef int(*PFN)(void);        /* function pointer */
typedef unsigned int uintptr;   /* needs to be the size of a pointer on your platform */

k_printf("ret:  %x\n", ((PFN) ((uintptr)mod->ptr[0] + (uintptr)mod + 4))());
</pre>

You can notice I add 4 bytes. This is because the ''size'' field was created by ''attachmod.py'', and the ''mod'' structure
starts their which means the ''ptr'' table is 4 bytes off. So I have to make that adjustment here. Also, the module was compiled
with a base address of 0x0 which means I can make relative offsets from the beginning of the module, this is why you see my using
''(uintptr)mod + 4'' which is the address of the first byte of the module and then ''(uintptr)mod->ptr[0]'' is simply added to that giving us the absolute address of the procedure.

=== Python Code (attachmod.py) ===
For example, ''./attachmod.py <module> <kernel-binary-image>''.

<pre>
#!/usr/bin/python3

import sys
import struct

srcf = sys.argv[1]
dstf = sys.argv[2]

sfd = open(srcf, "rb")
sfd.seek(0, 2)
sz = sfd.tell()
sfd.seek(0, 0)
sd = sfd.read(sz)
sfd.close()

print('[ATTACHMOD] %s [%s bytes] ----> %s' % (srcf, sz, dstf))

dfd = open(dstf, "r+b")
dfd.seek(0, 2)

-- TODO: Need to ensure that 'sd' is padded so that it is aligned on a 
--       4-byte boundary for processors that do not support unaligned 
--       memory access. May need additional field to specify actual module
--       size and then actual size to jump to next module.
dfd.write(struct.pack('I', sz))
dfd.write(sd)
dfd.close()
</pre>

=== Module Enumeration Code ===
<pre>
typedef struct _KATTMOD {
	uint32			size;
	uint32			signatureA;
	uint32			signatureB;
	uint32			interfaceID;
	uint32			ptr[];
} KATTMOD;

/*
	Get the next module after the specified module.
*/
KATTMOD *k_imgPkgGetNextMod(KATTMOD *mod) {
	KATTMOD		*n;

	n = (KATTMOD*)((uintptr)mod + mod->size);
	
	/* make sure there actually exists another module after this one */
	if (n->signatureA != 0x12345678 || ~n->signatureA != n->signatureB) {
		return 0;
	}
	
	return n;
}

/*
        Needed when no modules are attached. Keeps it from running forever in memory.
*/
extern uint8 _EOI;

/*
	Locate first module, if any exists, at end of our image.
*/
KATTMOD *k_imgPkgGetFirstMod() {
	uint32		*p;
	uint32		x;
	
	/* get end of our image */
	p = (uint32*)&k_imgPkgGetNextMod;
	
	/* find signature of first module */
	for (x = 0; x * 4 < ((uintptr)&_EOI) + 4 * 4); ++x) {
		/*
			This is a very tricky situation. We do not want the compiler
			to emit these sequence of bytes neither through an immediate
			operand or through optimization.
		*/
		if (p[x] == 0x12345678 && p[x + 1] == ~p[x]) {
			/* backup 32-bits (4 bytes) so we grab the size placed by attachmod.py */
			return (KATTMOD*)&p[x - 1];
		}
	}
	
	return 0;
}
</pre>

=== Linker Script ===
You should notice the ''_EOI'' symbol. This is not mandatory, but if your kernel supports being executed with
no attached modules then this is needed so it can detect that no modules exist and no get stuck into an infinite
loop looking for a module and/or eventually crash.

You do not have to copy this linker script, but just make sure to put in an ''_EOI'' symbol or modify the code
to detect the end of the kernel image differently.
<pre>
ENTRY (entry)

SECTIONS
{
    . = 0x10000;
    .text : { *(.text*) *(.rodata*) *(.data*) *(.bss*) }
    _EOI = .;
}
</pre>

=== Example Module ===
The module is compiled with -fPIC on the ARM platform. Depending on your target processor you may be able to use -fPIC or require actually doing a relocation on any executable code that is being loaded. Your module do not have to consist of executable code, but
at least require the header so you can locate them. You could modify the ''attachmod.py'' to automatically append a header if it is
needed, and specify a custom ID field.

<pre>
void start(void);

typedef unsigned int uint32;

int testA();
int testB();

const void __attribute__ ((section (".MODHDR"))) *___MODHDR[] = {
	(void*)0x12345678,
	(void*)~0x12345678,
	(void*)0x11223344,       /* custom ID field, can be anything */
	testA,                   /* does not have to be a function pointer */
	testB                    /* .. anything .. */
};

 
 int testA() {
	return 95;
 }
 
 int testB() {
	return testA() + 1;
 }
</pre>

Here is the linker script I used to link this module:
<pre>
SECTIONS
{
    . = 0x0;
    .text : { *(.MODHDR*) *(.text*) *(.rodata*) *(.data*) *(.bss*) *(COMMON) }
}
</pre>

=== Authors/Editors ===
<pre>
    Leonard Kevin McGuire Jr (www.kmcg3413.net) (kmcg3413@gmail.com) 2014
</pre>
````

<a id="w003454"></a>

## W003454 — User:Pancakes/BitmapHeapImplementation

Bron-ID: W003454
Namespace: User (2)
Revisie: 25752 | Datum: 2021-03-12T08:53:31Z
Bron: https://wiki.osdev.org/User:Pancakes/BitmapHeapImplementation
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APancakes%2FBitmapHeapImplementation&oldid=25752

````text
= Bitmap Heap Implementation =
This implementation of a heap uses a bitmap per block instead of a header for each allocation (and empty allocation) per block.

It provides data aligned on the block size specified for the block size per block. So if you use multiple block sizes then you could end up with data aligned on different boundaries. This might be a feature you wish to improve upon if you use this heap.

''Also do not be confused with block and block size. You add blocks to the heap which is were the allocations happen (inside each block), but inside each block you have a parameter that is specified that tells what size to chop the block into. For instance you could specify 16 which is a 16 byte blocks inside the block. The 16 is also the alignment, and the bitmap represents each of these 16 byte blocks inside the block.''

It has also been slightly optimized using the ''lfb'' field of each KHEAPBLOCKBM structure. This basically points to the space after the most recent allocation in hopes that during it allocation it will be closer to any free blocks. In certain situations this technique may not work well but it will provide some decent performance needed during early development. So as your kernel, user applications, or what not mature you may have to look for a better implementation.

== Simulation ==
I wrote a okay page that lets you simulate the heap with a view of memory and play back of memory accesses. It relates directly to this algorithm except it uses only 16-bit fields. You can find it
at [https://web.archive.org/web/20150810195624/http://www.kmcg3413.net/jsbmheap.htm] (original link dead).

== Example Usage ==
It might be easier also to use a ''#define'' directive to change the function names to something less
descriptive such as ''kmalloc'' or ''kfree''. Like, ''#define kmalloc k_heapBMAlloc''. Or, you could
just directly change the procedure names.

The ''16'' passed to the add block function specifies the default block size (very much like a hard disk). So something that was 9 bytes long would actually take up 16 bytes, or something that was 17 bytes would take up 32 bytes.
<source lang="c">
    KHEAPBM     kheap;
    char        *ptr;

    k_heapBMInit(&kheap);                              /* initialize the heap */
    k_heapBMAddBlock(&kheap, 0x100000, 0x100000, 16);  /* add block to heap 
                                                       (starting 1MB mark and length of 1MB) 
                                                       with default block size of 16 bytes
                                                       */
    ptr = (char*)k_heapBMAlloc(&kheap, 256);           /* allocate 256 bytes (malloc) */
    k_heapBMFree(&kheap, ptr);                         /* free the pointer (free) */
</source>

Now, using this you will have one problem. It will return a null pointer when there is no memory left which can be a problem. To get around this you can write a wrapper function and call it ''kmalloc'' or something, and have it check for a null pointer and if found allocate some memory from your higher level memory manager then add a block with ''k_heapBMAddBlock'' then attempt to allocate again. You will also want to add a check that your adding a large enough block for the allocation. You know if you adding 4K blocks to the heap and something allocates 16K well your going to need to add a block larger than 16K to have room for the bitmap and header. I will try to add a function later to tell you how much memory you need to handle a certain sized allocation later, but until now I suppose that is an exercise for you!

== Code ==
<source lang="c">
/*
    2014 Leonard Kevin McGuire Jr (www.kmcg3413.net) (kmcg3413@gmail.com)
    2016 Clément Gallet (provided bug fixes)
*/
typedef struct _KHEAPBLOCKBM {
	struct _KHEAPBLOCKBM	                *next;
	uint32					size;
	uint32					used;
	uint32					bsize;
        uint32                                  lfb;
} KHEAPBLOCKBM;

typedef struct _KHEAPBM {
	KHEAPBLOCKBM			*fblock;
} KHEAPBM;

void k_heapBMInit(KHEAPBM *heap) {
	heap->fblock = 0;
}

int k_heapBMAddBlock(KHEAPBM *heap, uintptr addr, uint32 size, uint32 bsize) {
	KHEAPBLOCKBM		*b;
	uint32				bcnt;
	uint32				x;
	uint8				*bm;
	
	b = (KHEAPBLOCKBM*)addr;
	b->size = size - sizeof(KHEAPBLOCKBM);
	b->bsize = bsize;
	
	b->next = heap->fblock;
	heap->fblock = b;

	bcnt = b->size / b->bsize;
	bm = (uint8*)&b[1];
	
	/* clear bitmap */
	for (x = 0; x < bcnt; ++x) {
			bm[x] = 0;
	}

	/* reserve room for bitmap */
	bcnt = (bcnt / bsize) * bsize < bcnt ? bcnt / bsize + 1 : bcnt / bsize;
	for (x = 0; x < bcnt; ++x) {
			bm[x] = 5;
	}
	
	b->lfb = bcnt - 1;
	
	b->used = bcnt;
	
	return 1;
}

static uint8 k_heapBMGetNID(uint8 a, uint8 b) {
	uint8		c;	
	for (c = a + 1; c == b || c == 0; ++c);
	return c;
}

void *k_heapBMAlloc(KHEAPBM *heap, uint32 size) {
	KHEAPBLOCKBM		*b;
	uint8				*bm;
	uint32				bcnt;
	uint32				x, y, z;
	uint32				bneed;
	uint8				nid;

	/* iterate blocks */
	for (b = heap->fblock; b; b = b->next) {
		/* check if block has enough room */
		if (b->size - (b->used * b->bsize) >= size) {
			
			bcnt = b->size / b->bsize;		
			bneed = (size / b->bsize) * b->bsize < size ? size / b->bsize + 1 : size / b->bsize;
			bm = (uint8*)&b[1];
			
			for (x = (b->lfb + 1 >= bcnt ? 0 : b->lfb + 1); x < b->lfb; ++x) {
				/* just wrap around */
				if (x >= bcnt) {
					x = 0;
				}		

				if (bm[x] == 0) {	
					/* count free blocks */
					for (y = 0; bm[x + y] == 0 && y < bneed && (x + y) < bcnt; ++y);
					
					/* we have enough, now allocate them */
					if (y == bneed) {
						/* find ID that does not match left or right */
						nid = k_heapBMGetNID(bm[x - 1], bm[x + y]);
						
						/* allocate by setting id */
						for (z = 0; z < y; ++z) {
							bm[x + z] = nid;
						}
						
						/* optimization */
						b->lfb = (x + bneed) - 2;
						
						/* count used blocks NOT bytes */
						b->used += y;
						
						return (void*)(x * b->bsize + (uintptr)&b[1]);
					}
					
					/* x will be incremented by one ONCE more in our FOR loop */
					x += (y - 1);
					continue;
				}
			}
		}
	}
	
	return 0;
}

void k_heapBMFree(KHEAPBM *heap, void *ptr) {
	KHEAPBLOCKBM		*b;	
	uintptr				ptroff;
	uint32				bi, x;
	uint8				*bm;
	uint8				id;
	uint32				max;
	
	for (b = heap->fblock; b; b = b->next) {
		if ((uintptr)ptr > (uintptr)b && (uintptr)ptr < (uintptr)b + sizeof(KHEAPBLOCKBM) + b->size) {
			/* found block */
			ptroff = (uintptr)ptr - (uintptr)&b[1];  /* get offset to get block */
			/* block offset in BM */
			bi = ptroff / b->bsize;
			/* .. */
			bm = (uint8*)&b[1];
			/* clear allocation */
			id = bm[bi];
			/* oddly.. GCC did not optimize this */
			max = b->size / b->bsize;
			for (x = bi; bm[x] == id && x < max; ++x) {
				bm[x] = 0;
			}
			/* update free block count */
			b->used -= x - bi;
			return;
		}
	}
	
	/* this error needs to be raised or reported somehow */
	return;
}
</source>
````

<a id="w003455"></a>

## W003455 — User:Pancakes/EntryBasedHeapImplementation

Bron-ID: W003455
Namespace: User (2)
Revisie: 16348 | Datum: 2014-05-07T06:46:37Z
Bron: https://wiki.osdev.org/User:Pancakes/EntryBasedHeapImplementation
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APancakes%2FEntryBasedHeapImplementation&oldid=16348

````text
== Simple Heap Implementation ==
This is a fairly simple heap implementation. It has not been well tested so it could have a bug somewhere, but so far it seems to work well and I have not run into any problems. It basically works by you assigning blocks of memory to the heap. Each block of
memory is separate. So the maximum allocation you can make is the size of the largest block minus the header. Also, over time the
heap can become kind of fragmented from calls to alloc and free which eventually create spots between allocations and this can
reduce your maximum allocation size. A good fallback would be for you to add another block to the heap if an allocation can not be
made in the event the heap becomes highly fragmented. I do not add another block here because I wanted to keep this implementation with out any dependencies. The best way would likely be to wrap these calls in another layer than can handle the failed call to
the allocation function and try to add another block (and also check adding another block will work).

Also, I forgot to mention there is a maximum block size set at 2,147,483,647 bytes (31-bit). This is because each header/chunk inside the block only has 31 bits to specify it's size (free or used). The highest bit (32nd bit) is used to flag if the header/chunk is used or free. However, the implementation is so short and simple that if you needed to support bigger blocks and allocations you could simply expand the header to handle any. I just settled for 2GB because I think that is good for most hobby projects, while still keeping the overhead low on memory consumption and still being quite fast.

A good idea is to build a wrapper around these calls (not only for your kernel to call k_heapAddBlock) to provide a debugging ability. What you do is catch all alloc calls and add some extra bytes onto the size. Then place a special signature at the
beginning and end to catch under-flows and overflows for writing. It is not easy to catch over reading or under reading, but
at least you can catch when your heap has become corrupted!

=== Gotcha ===
I forgot to make it align on at least a specific boundary such as 4-byte boundary. So, I think it would be good practice for whomever if using this to code that in! At least until I get around to fixing it. The fix should be fairly simple, and mainly just involve some code on the allocation function.

=== k_heapInit ===
This initializes the heap structure. Nothing too complex.
=== k_heapAddBlock ===
You use this to add a block to the heap. Like, h_heapAddBlock(&heap, 0x1000, 0x2000) would add 2KB starting at 1KB.
=== k_heapAlloc ===
This will allocate a chunk which can be used.
=== k_heapFree ===
This will free a chunk.
=== Example Usage ===
<pre>
    KHEAP       kheap;
    char        *ptr;

    k_heapLCABInit(&kheap);                          /* initialize the heap */
    k_heapLCABAddBlock(&kheap, 0x100000, 0x100000);  /* add block to heap (starting 1MB mark and length of 1MB) */
    ptr = (char*)k_heapLCABAlloc(&kheap, 256);               /* allocate 256 bytes (malloc) */
    k_heapLCABFree(&kheap, ptr);                             /* free the pointer (free) */
</pre>
Also, you could wrap the functions so that ''kheap'' does not need to be passed thus making the calls
much more simple and easy to write. I thought it was better to give the programmer the ability to have
multiple heaps than to restrict them. So if you only want to use one heap then create an abstraction
layer that hands in the ''kheap'' argument.

=== C/C++ Code ===
<pre>
/*
    2014 Leonard Kevin McGuire Jr (www.kmcg3413.net) (kmcg3413@gmail.com)
*/
typedef unsigned int uint32;
typedef unsigned char uint8;
typedef unsigned int uintptr;

#define KHEAPFLAG_USED			0x80000000

typedef struct _KHEAPHDRLCAB {
	uint32				prevsize;
	uint32				flagsize;
} KHEAPHDRLCAB;

typedef struct _KHEAPBLOCKLCAB {
	uint32					size;
	uint32					used;
	struct _KHEAPBLOCKLCAB	                *next;
	uint32					lastdsize;
	KHEAPHDRLCAB			        *lastdhdr;
} KHEAPBLOCKLCAB;

typedef struct _KHEAPLCAB {
	KHEAPBLOCKLCAB		       *fblock;
	uint32				bcnt;
} KHEAPLCAB;

void k_heapLCABInit(KHEAPLCAB *heap) {
		heap->fblock = 0;
		heap->bcnt = 0;
}

int k_heapLCABAddBlock(KHEAPLCAB *heap, uintptr addr, uint32 size) {
	KHEAPBLOCKLCAB			*hb;
	KHEAPHDRLCAB			*hdr;
	
	//printf("add block addr:%p size:%x\n", addr, size);
	
	hb = (KHEAPBLOCKLCAB*)addr;
	hb->size = size;
	hb->used = 0;
	hb->next = heap->fblock;
	heap->fblock = hb;
	
	hdr = (KHEAPHDRLCAB*)&hb[1];
	hdr->flagsize = hb->size - (sizeof(KHEAPBLOCKLCAB) + 32);
	
	++heap->bcnt;
	
	hb->lastdsize = 0;
	hb->lastdhdr = 0;
	
	//printf("added block; block-count:%u\n", heap->bcnt);
	
	//printf("added block hb:%p hb->next:%p\n", hb, hb->next);
	//sleep(10);
	return 1;
}

/*
	Look behind and forward to see if we can merge back into some chunks.
*/
void k_heapLCABFree(KHEAPLCAB *heap, void *ptr) {
	KHEAPHDRLCAB				*hdr, *phdr, *nhdr;
	KHEAPBLOCKLCAB				*hb;
	uint32						sz;
	uint8						fg;
	
	//printf("lab pre-free ptr:%p\n", ptr);
	
	hdr = (KHEAPHDRLCAB*)ptr;
	//printf("GGGG\n");
	hdr[-1].flagsize &= ~0x80000000;
		
	//printf("lcab free\n");
		
	phdr = 0;
	/* find the block we are located in */
	for (hb = heap->fblock; hb; hb = hb->next) {
		//printf("lcab free looking at block:%p next:%p ptr:%p end:%p\n", hb, hb->next, ptr, (uintptr)hb + hb->size);
		if (((uintptr)ptr > (uintptr)hb) && ((uintptr)ptr < (uintptr)hb + hb->size)) {
			/* we have found the heap block */
			
			/* get header */
			hdr = (KHEAPHDRLCAB*)((uintptr)ptr - sizeof(KHEAPHDRLCAB));
			
			/* set to free */
			hdr->flagsize &= ~0x80000000;
			
			hb->used -= hdr->flagsize;
			
			/* get previous header if it exists */
			if (hdr->prevsize) {
				phdr = (KHEAPHDRLCAB*)((uintptr)&hdr - (sizeof(KHEAPHDRLCAB) + hdr->prevsize));
			} else {
				phdr = 0;
			}
			
			//printf("hdr:%p hdr->flagsize:%x hdr->prevsize:%x\n", hdr, hdr->flagsize, hdr->prevsize);
			/* get next header */
			nhdr = (KHEAPHDRLCAB*)((uintptr)&hdr[1] + hdr->flagsize);
			if ((uintptr)nhdr >= ((uintptr)hb + hb->size)) {
				nhdr = 0;
			}
						
			//fg = hdr->flagsize >> 31;
			//sz = hdr->flagsize & 0x7fffffff;			
			
			//printf("hdr:%p phdr:%p nhdr:%p phdr->flagsize:%x hdr->flagsize:%x\n", hdr, phdr, nhdr, phdr->flagsize, hdr->flagsize);
			if (nhdr) {
				if (!(nhdr->flagsize & 0x80000000)) {
					/* combine with it */
					hdr->flagsize += sizeof(KHEAPHDRLCAB) + nhdr->flagsize;
					hb->used -= sizeof(KHEAPHDRLCAB);
					/* set next header prevsize */
					nhdr = (KHEAPHDRLCAB*)((uintptr)&hdr[1] + hdr->flagsize);
					nhdr->prevsize = hdr->flagsize;
				}
			}
			
			//printf("here hdr:%p prevsize:%x\n", hdr, hdr->prevsize);
			
			/* can we combine with previous header? */
			if (phdr) {				
				if (!(phdr->flagsize & 0x80000000)) {
					//printf("combine backward\n");
					phdr->flagsize += sizeof(KHEAPHDRLCAB) + hdr->flagsize;
					hb->used -= sizeof(KHEAPHDRLCAB);
					hdr = phdr;
					/* set next header prevsize */
					nhdr = (KHEAPHDRLCAB*)((uintptr)&hdr[1] + hdr->flagsize);
					if ((uintptr)nhdr < (uintptr)hb + sizeof(KHEAPBLOCKLCAB) + hb->size) {
						nhdr->prevsize = hdr->flagsize;
					}
				}
			}
			
			/* optimization */
			if (hdr->flagsize > hb->lastdsize) {
				hb->lastdsize = hdr->flagsize;
				hb->lastdhdr = hdr;
			}
			
			return;
		}
	}
	
	printf("uhoh ptr:%p\n", ptr);
	for (hb = heap->fblock; hb; hb = hb->next) {
		printf("lcab free looking at block:%p next:%p ptr:%p end:%p\n", hb, hb->next, ptr, (uintptr)hb + hb->size);
		if (((uintptr)ptr > (uintptr)hb)) {
			printf("above\n");
			if (((uintptr)ptr < ((uintptr)hb + hb->size))) {
				printf("found\n");
			}
		}
	}
	for (;;);
	/* uhoh.. this should not happen.. */
	return;
}

/*
	This will allocate a chunk of memory by the specified size, and if
	no memory chunk can be found it will return zero.
*/
void* k_heapLCABAlloc(KHEAPLCAB *heap, uint32 size) {
	KHEAPBLOCKLCAB		*hb;
	KHEAPHDRLCAB		*hdr, *_hdr, *phdr;
	uint32				sz;
	uint8				fg;
	uint32				checks;
	uint32				bc;
	
	checks = 0;
	bc =0;
	
	//printf("lcab alloc\n");
	/* first find heap block with enough space */
	for (hb = heap->fblock; hb; hb = hb->next) {
		if ((hb->size - hb->used) >= (size + sizeof(KHEAPHDRLCAB))) {
			++bc;
			/* optimization */
			//if (hb->lastdsize >= size) {
				/* optimization (use block) */
				//hdr = hb->lastdhdr;
				//hb->lastdhdr = 0;
				//hb->lastdsize = 0;
			//} else {
			hdr = (KHEAPHDRLCAB*)&hb[1];
			//}
			
			//printf("enter loop\n");
			phdr = 0;
			while ((uintptr)hdr < ((uintptr)hb + hb->size)) {
				++checks;
				//printf("lcab alloc found\n");
				fg = hdr->flagsize >> 31;
				sz = hdr->flagsize & 0x7fffffff;
				//printf("hdr:%p fg:%x sz:%x\n", hdr, fg, sz);
				if (!fg) {
					//printf("lcab alloc got chunk\n");
					if (sz >= size) {
						//printf("lcab alloc thinking of splitting\n");
						// else take whole chunk
						if (sz > (size + sizeof(KHEAPHDRLCAB) + 16)) {
							//printf("lcab alloc splitting\n");
							/* has enough left over (break into two) */
							_hdr = (KHEAPHDRLCAB*)((uintptr)&hdr[1] + size);
							//printf("AA\n");
							/* take out data size and header size */
							_hdr->flagsize = sz - (size + sizeof(KHEAPHDRLCAB));
							_hdr->prevsize = size;
							//printf("BB\n");
							/* set to used and set new size */
							hdr->flagsize = 0x80000000 | size;
							//printf("CC\n");
							hb->used += sizeof(KHEAPHDRLCAB);
							//printf("DD\n");
						} else {
							/* simply set to used */
							hdr->flagsize |= 0x80000000;
						}
						hb->used += size;
						
						//printf("ptr:%p\n", &hdr[1]);
						//printf("alloced checks:%u blocks-checked:%u\n", checks, bc);
						
						return &hdr[1];
					}
				}
				phdr = hdr;
				hdr = (KHEAPHDRLCAB*)((uintptr)&hdr[1] + sz);
			}
			//printf("exit loop\n");
		}
	}
	
	//printf("return null\n");
	return 0;
}
</pre>

== Problems/Suggestions/Feedback ==
I rarely get any feedback from a lot of stuff. So even negative feedback is good. If you want to tell me something just drop a message at, [mailto:kmcg3413@gmail.com kmcg3413@gmail.com]. I am always curious who actually find usage for things that I put up on the internet.
````

<a id="w003456"></a>

## W003456 — Talk:Exceptions

Bron-ID: W003456
Namespace: Talk (1)
Revisie: 25246 | Datum: 2020-10-10T12:32:30Z
Bron: https://wiki.osdev.org/Talk:Exceptions
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AExceptions&oldid=25246

````text
From IRC:

<tt><lightburst> processors after 386 apparently dont generate that coprocessor segment overrun interrupt. someone should highlight that in the wiki.</tt>

<tt><lightburst> great. :) if citations are necessary, the info is a simple footnote in the interrupt table thing in the intel manual.</tt> 

--[[User:Sortie|Sortie]] 09:05, 25 February 2014 (CST)


==Security Exception==
I was looking at the intel manual referenced at the bottom of the page I didn't see a reference to the security exception. Is there a source for this? Also even though it is a link it doesn't go anywhere. I would be happy to write that if I could find the source.

[[User:Gwoplock|Gwoplock]] 18:04, 27 March 2018 (CDT)

== Questionable edits cleaned up ==

I've cleaned up some questionable edits that have been lurking here since 10 July 2017. Could anyone please review my changes?

Other pages may be also affected by similarly questionable edits by the same person. But unfortunately I don't have the time to look through everything affected, and it is made more complicated by the fact that not everything they wrote is questionable.

[[User:Testjz|Testjz]] 05:07, 10 October 2020 (CDT)

Also, I forgot, [https://forum.osdev.org/viewtopic.php?t=37280 this is the forum topic that prompted me to do this].

[[User:Testjz|Testjz]] 07:32, 10 October 2020 (CDT)
````

<a id="w003457"></a>

## W003457 — Libgcc

Bron-ID: W003457
Namespace: Main (0)
Revisie: 30323 | Datum: 2026-03-17T19:08:33Z
Bron: https://wiki.osdev.org/Libgcc
Revisie-URL: https://wiki.osdev.org/index.php?title=Libgcc&oldid=30323

````text
The [[GCC|GNU Compiler Collection]] uses a special library called <tt>libgcc</tt> during code generation, which contains shared code that would be inefficient to duplicate every time as well as auxiliary helper routines and runtime support. Its exact contents depends on the particular target, configuration and even command line options. GCC unconditionally assumes it can safely emit calls to <tt>libgcc</tt> symbols as it sees fit, thus all code compiled by GCC must be linked with <tt>libgcc</tt>. The library is automatically included by default when you link with GCC and you need to do nothing further to use it. However, kernels usually don't link with the standard user-space libc for obvious reasons and are linked with the <tt>-nodefaultlibs</tt> (implied by <tt>-nostdlib</tt>) which disables the automatic linking with libc and <tt>libgcc</tt>. This is a problem as gcc still thinks it can use <tt>libgcc</tt> and you need to link with it.

== How to build libgcc ==
{{Main|GCC Cross-Compiler}}

You need to invoke the <tt>all-target-libgcc</tt> and <tt>install-target-libgcc</tt> make targets while building your [[GCC Cross-Compiler]] to build and install <tt>libgcc</tt> along with your cross-compiler. A static library called <tt>libgcc</tt>.a is installed into your compiler prefix along with the other compiler-specific files. Note that some architectures, such as ARM, have multiple kinds of ABIs and instruction sets: these targets will thus need multiple versions of <tt>libgcc</tt> depending on which particular compile options you use and they all go into compiler-specific subdirectories.

== How to link with libgcc ==
{{Main|Bare Bones}}

You can link with <tt>libgcc</tt> by passing <tt>-lgcc</tt> when linking your kernel with your compiler. You don't need to do this unless you pass the <tt>-nodefaultlibs</tt> option (implied by <tt>-nostdlib</tt>). For instance:

<syntaxhighlight lang="bash">i686-elf-gcc -T linker.ld -o myos.kernel -ffreestanding boot.o kernel.o -nostdlib -lgcc</syntaxhighlight>

Note how <tt>libgcc</tt> is installed in a compiler-specific directory known to your compiler, but not to your linker. Thus you must use the compiler as a linker, rather than invoking <tt>ld</tt> directly, or you'll need to tell the linker where to find <tt>libgcc</tt>. Note also how you must be certain to give the machine compile options you compile with when you link (the <tt>-mfoo</tt> and <tt>-fbar</tt> options, among others) or you may get the wrong <tt>libgcc</tt>. If you want to know the full path of <tt>libgcc</tt> (if you insist on linking with <tt>ld</tt> rather than your compiler), you can do so:

<syntaxhighlight lang="bash">i686-elf-gcc $CFLAGS -print-libgcc-file-name</syntaxhighlight>

You naturally also need to pass the machine compile options when using the <tt>-print-libgcc-file-name</tt> option. Make your build scripts or Makefile locate <tt>libgcc</tt> rather than hard-coding the path somewhere or other people will have a harder time building your operating system.

== Frequently Asked Questions ==

=== When do I need to link with <tt>libgcc</tt>? ===

All code compiled with GCC must be linked with <tt>libgcc</tt>.

=== I link with <tt>libgcc</tt> and nothing changes? ===

The compiler is not required to emit calls to <tt>libgcc</tt> and on some platforms <tt>libgcc</tt> is a small library and you may be able to write complex programs that never need <tt>libgcc</tt>. The compiler will only link in the specific parts of <tt>libgcc</tt> that are needed, since it is a static library. However, the compiler is free to change its mind and suddenly emit calls to <tt>libgcc</tt> if it is a Tuesday and the stock prices have gone up (or you upgraded your compiler to a newer version).

=== I don't link with <tt>libgcc</tt> and it works? ===

See the above question. The compiler doesn't care if you actually link with <tt>libgcc</tt>, it will emit calls just the same. You will never get <tt>libgcc</tt>-related linking errors if you link with <tt>libgcc</tt>, which is simple and makes your kernel compilation process reliable.

Think of it this way: you went into the compiler source code and randomly commented out a few randomly chosen functions. The compiler still happens to work for your kernel, but will it always work for all the possible kernel source codes you can write? The <tt>libgcc</tt> library is part of the compiler and it would be foolish to unsafely disable parts of the compiler.

=== Can I use the Linux <tt>libgcc</tt>? ===

You must use the correct <tt>libgcc</tt> that came with your cross-compiler. Any other <tt>libgcc</tt> you found likely has a different target, was built with different machine compile options, has dependencies on the standard library, is part of a different compiler revision (your distribution may have patched its <tt>gcc</tt>, even). It is possible that using a different <tt>libgcc</tt> will work, but it might not be reliable.

=== Can I copy the <tt>libgcc</tt> functions I need into my OS? ===

You really shouldn't. These functions are compiler revision specific and the set of needed functions can potentially change when you upgrade your compiler. You'll be playing catch-up with the compiler developers and you might have even more trouble if your other contributors use different compiler versions. There is also the issue of whether you have copied enough of <tt>libgcc</tt>, if you don't have all of it, things will not work reliably.

=== Can I just re-implement <tt>libgcc</tt>? ===

You really shouldn't. See the above question. It's a lot of effort and very tricky to get right, you will likely get an inefficient 64-bit division function that doesn't work properly for all values. It's much simpler to just link with <tt>libgcc</tt> and get it over with.

=== Does <tt>libgcc</tt> have dependencies? ===

It usually doesn't, the <tt>-elf</tt> target used for tutorials here certainly doesn't. If you have a hosted user-space, it may enable additional parts of itself and have special symbols that rely on particular user-space functions it found in your system headers. These are common functions that you'd want to implement in your kernel standard library regardless. Some dependencies are only relevant if you use the relevant <tt>libgcc</tt> parts that trigger them.

=== What is the <tt>libgcc</tt> license? ===

The libgcc library is licensed under the GNU GPL plus the GCC Runtime Library Exception (see <tt>COPYING.RUNTIME</tt> in your GCC source tree). This roughly means that you are allowed to link in <tt>libgcc</tt> into your software even if it would normally violate the GNU GPL, as long as you used a non-proprietary version of GCC. This is nothing unusual, such licensed code is linked into everything GCC creates, especially user-space programs and you can legally compile and distribute proprietary programs with GCC because of the runtime library exception.

=== It's too big! ===

It isn't. The static archive contains a lot of object files that each contain debugging symbols. You'll find the archive gets considerably smaller if you attempt to strip it. Furthermore, it's not even linked yet and a lot of that information is simply overhead from the file format and linking information. Finally, it's a static archive which means that only the relevant parts actually get linked into your binary.

=== What if I am careful? ===

On some platforms the compiler doesn't generate calls to <tt>libgcc</tt> for most common code except in particular circumstances. You can avoid particular features and be mostly safe. But I'll bet you that you haven't actually read into the GCC source code and found the exact rules for when calls to <tt>libgcc</tt> are emitted. The rules may change when you update your compiler. Indeed, tomorrow may be a Tuesday. The library provides useful features and it's simple to link with it and enjoy the features. If the rules change and you cannot build your kernel with the new compiler, but you fix your kernel, then you still cannot build older versions of your operating system (which could have been useful for debugging or historical purposes).

=== I don't want it because I want control! ===

You didn't write the compiler you are using. Ponder that for a few moments and let the Not-Invented-Here kick in.

=== I really don't want <tt>libgcc</tt>! ===

The above rule is:

:All code compiled with GCC must be linked with <tt>libgcc</tt>.

But that's okay, you can just change what GCC is and get rid of the rule. Modify the GCC source code such that it never emits calls to <tt>libgcc</tt> (perhaps only if <tt>-ffreestanding</tt>) but generates code directly instead. You don't have to link with it then.

=== I'm using clang ===

This is an article about GCC, you need to do different things if you are using another compiler.

=== I saw someone not link with <tt>libgcc</tt>! ===

Report them to the local compiler crime police.

==See Also==
===Articles===
* [[GCC]]
* [[GCC Cross-Compiler]]
* [[Bare Bones]]
* [[Building libgcc for mcmodel=kernel]]

[[Category:C]]
[[Category:C++]]
[[Category:Compilers]]
[[Category:FAQ]]
````

<a id="w003458"></a>

## W003458 — Host Triplet

Bron-ID: W003458
Namespace: Main (0)
Revisie: 15660 | Datum: 2014-02-26T19:06:33Z
Bron: https://wiki.osdev.org/Host_Triplet
Revisie-URL: https://wiki.osdev.org/index.php?title=Host_Triplet&oldid=15660
Doorverwijzing: Target Triplet

````text
#REDIRECT [[Target Triplet]]
````

<a id="w003459"></a>

## W003459 — James Molloy's Tutorial Known Bugs

Bron-ID: W003459
Namespace: Main (0)
Revisie: 25869 | Datum: 2021-05-18T22:17:05Z
Bron: https://wiki.osdev.org/James_Molloy%27s_Tutorial_Known_Bugs
Revisie-URL: https://wiki.osdev.org/index.php?title=James_Molloy%27s_Tutorial_Known_Bugs&oldid=25869

````text
Several sources - including this Wiki - point to [http://www.jamesmolloy.co.uk/tutorial_html/ James Molloy's Roll your own toy UNIX-clone OS] Tutorial as a starting point. This is fine, but the tutorial has some well-known weak points that cause trouble for people again and again. It's not uncommon that well-established members traced back mysterious bugs to early parts of their operating systems based on this tutorial. Nonetheless, it's one of the best introductory tutorials out there even if it has the occasional landmine. This article is meant to preempt issues arising from following the tutorial and to aid those that have encountered such problems. It is generally recommended to be sceptical of its advise on how to design your kernel and compare its information against this wiki. Some issues are quite subtle and only experts will recognize them.

== Before you follow the tutorial ==
{{Main|Bare Bones}}

It is recommended that you follow the this wiki's standard tutorial [[Bare Bones]] before you begin with the tutorial. This ensures you get the a proper cross-compiler and use the proper compilation options. If you have already followed the tutorial, please compare your current build environment against the recommended practices covered by [[Bare Bones]].

== Problem: Not using a cross-compiler ==
{{Main|GCC Cross-Compiler}}

This tutorial was written years before it was recognized as standard practice to use a cross-compiler. As such, you should disregard most of the build instructions in Chapter 1 Environment Setup and instead follow [[Bare Bones]]. You should use the [[Bare Bones]] linker script instead as well as the the boot assembly from [[Bare Bones]]. Floppies are an obsolete technology and it is advisable to create a bootable cdrom image instead.

== Problem: __cdecl calling convention ==

The tutorial states that the <tt>__cdecl</tt> calling convention is used. This is, however, a Windows term. Your cross-compiler uses a similar calling convention but it is called the System V ABI for i386. It is advisable to understand this calling convention in depth, especially parts about how the parameters on the stack are clobbered and how structure parameters are passed. This will be very useful later and will help you avoid a later subtle bug. The function call example in 2.3 neglects to add 12 to esp following the call instruction, so the three parameters are never popped.

== Problem: CFLAGS ==

The tutorial recommends using these compilation options <tt>-nostdlib -nostdinc -fno-builtin -fno-stack-protector</tt>, but this is not the recommended practice. The tutorial neglects to pass the important <tt>-ffreestanding</tt> option. See [[Bare Bones]] on how to correctly compile C kernel files and how to correctly link the kernel.

== Problem: Not using libgcc ==

{{Main|libgcc}}

The tutorial disables libc and [[libgcc]] through the <tt>-nodefaultlibs</tt> option (implied by <tt>-nostdlib</tt>) but neglects to add back libgcc during the link.

== Problem: Not setting a stack ==
{{main|Stack#Setup the stack}}

The tutorial neglects to set a stack in the initial boot file and relies on the bootloader using an appropriate [[stack]]. You should instead declare your own stack as an array and use that instead, such that you have control of the situation.

== Problem: main function ==

This isn't a regular <tt>main</tt> function: The name <tt>main</tt> is actually a special case in C and it would be inadvisable to call it that. You should call it something like <tt>kernel_main</tt> instead.

== Problem: Data types ==

The tutorial uses non-standard data types such as <tt>u32int</tt> while the international C standard (99 revision) introduces standard fixed-width data types like <tt>uint32_t</tt> that you should use instead. Simply include <tt><stdint.h></tt> which comes with your cross-compiler and works even in freestanding mode. This is the reason you should not pass the <tt>-nostdinc</tt> option.

== Problem: Inline Assembly ==

{{Main|Inline Assembly/Examples}}

The tutorial uses inline assembly, which is notoriously hard to get exactly correct. The smallest error can emit assembly that fully works until one day the optimizer does things differently. While the inline assembly looks correct to me at a glance, please compare the inline assembly with the [[Inline Assembly/Examples|examples on this wiki]].

== Problem: Missing functions ==

The GCC documentation mentions that the <tt>memset</tt>, <tt>memcpy</tt>, <tt>memmove</tt> and <tt>memcmp</tt> functions must always be present. The compiler uses these automatically for certain optimization purposes and even code that doesn't use them can automatically generate calls to them. You should add them at your earliest convenience.

== Problem: Interrupt handlers corrupt interrupted state ==

This article previously told you to know the ABI. If you do you will see a huge problem in the interrupt.s suggested by the tutorial: It breaks the ABI for structure passing! It creates an instance of the <tt>struct registers</tt> on the stack and then passes it by value to the <tt>isr_handler</tt> function and then assumes the structure is intact afterwards. However, the function parameters on the stack belongs to the function and it is allowed to trash these values as it sees fit (if you need to know whether the compiler actually does this, you are thinking the wrong way, but it actually does).
There are two ways around this. The most practical method is to pass the structure as a pointer instead, which allows you to explicitly edit the register state when needed - very useful for system calls, without having the compiler randomly doing it for you. The compiler can still edit the pointer on the stack when it's not specifically needed. The second option is to make another copy the structure and pass that.

== Problem: ISR 17 and 21 have error codes ==

The interrupt handling code in the downloadable code have a bug where it handles ISR 17 and 21 by pushing a fake error code, but the CPU does push error codes here.

== Problem: struct registers::esp is useless ==

The <tt>struct registers</tt> structure has a esp member that is one of the values pushed by pusha. This value is, however, ignored by popa for obvious reasons. You should rename it to <tt>useless_value</tt> and rename <tt>useresp</tt> to <tt>esp</tt> instead. The value is useless because it has to do with the current stack context, not what was interrupted.

== Problem: __attribute__((packed)) ==

This attribute packs the associated structure. This is useful in a few cases, such as the IDT and GDT code (actually just the IDT, GDT and TSS pointers). However, the tutorial tends to randomly attach it to every struct parameter, even where it isn't even needed. It is only needed where you badly want aligned structure members, it doesn't do anything if all the structure members were already naturally aligned. Otherwise, the compiler will automatically insert gaps between structure members so each begins at its own natural alignment.

== Problem: cli and sti in interrupt handlers ==

The interrupt.s file invokes the <tt>cli</tt> and <tt>sti</tt> in the interrupt handler to disable and enable interrupts, as if the author didn't know whether the interrupt handlers were run with interrupts on or off. You can control whether they are run with interrupts on and off by simply deciding it in your IDT entry for the particular interrupt. The <tt>sti</tt> during the interrupt handler end is also useless as <tt>iret</tt> loads the eflags value from the stack, which contains a bit telling whether interrupts are on or off; in other words the interrupt handler automatically restores interrupts whether or not interrupts were enabled before this interrupt.

== Problem: kmalloc isn't properly aligned ==

Each data type in C has its own natural alignment. For instance, on the ABI that you are using an int is a signed 32-bit value that must be 32-bit aligned in memory (4 byte alignment). The same applies for structures, where the alignment of the whole structure is the maximum alignment of all its members. It is undefined behavior to access an unaligned value. For instance, you could decide you want an int at a particular unaligned (for an int) memory address and construct a pointer to it. When you attempt to write an int value to that pointer, undefined behavior happens. Furthermore, SIMD registers have alignment needs that are bigger than their individual components.

The <tt>kmalloc</tt> function in 6.4.1 only 1-byte aligns or page-aligns its memory address. This means you can only reliably use it allocate memories for chars (size 1), but not any larger types unless you use page-alignment. A proper malloc implementation returns pointers that are aligned such that they are suitable for all the common types, for instance it could be 64-bit (8-byte) aligned. You'll also want to modify the parameters such that it uses <tt>size_t</tt> appropriately rather than <tt>u32int</tt>.

Additionally the check if the address is page aligned is wrong.
 <nowiki>
if (align == 1 && (placement_address & 0xFFFFF000)) // If the address is not already page-aligned</nowiki>
should be
 <nowiki>
if (align == 1 && (placement_address & 0x00000FFF)) // If the address is not already page-aligned</nowiki>

== Problem: Paging Code ==

The paging code isn't terribly good and it is worth it to fully understand paging and design it all yourself. Paging code tends to be quite ugly, but it'll probably be decent after your fifth design revision. There is no need to always re-enable paging in <tt>switch_page_directory</tt>, it is likely best to have a special function the first time paging is enabled. The Inline Assembly in 6.4.5. doesn't need to be volatile as it is simply reading a memory value, which has no side-effects and it is acceptable if the compiler optimizes it away if the value is never used.

== Problem: Heap Code ==

It is probably best that you write your own heap implementation.

There is an operator precedence bug in <tt>find_smallest_hole()</tt> that will cause bad allocations and memory overwrites if attempting to fork in user mode later on.

To fix the problem, change this:
 <nowiki>
if ((location+sizeof(header_t)) & 0xFFFFF000 != 0)</nowiki>

to this:
 <nowiki>
if ((location+sizeof(header_t) & 0xFFFFF000) != 0)</nowiki>

See the section on user mode below for more details.

== Problem: VFS Code ==

The name of files on Unix are stored in the directory entries rather than the inode itself (<tt>struct fs_node</tt> here), this allows Unix files to have multiple names and even none if the file is deleted but the inode is not yet closed in all programs.

== Problem: multiboot.h ==

It's advisable to get a copy of <tt>multiboot.h</tt> from the GRUB source code rather than copied from the tutorial. Beware, the copy in the GRUB documentation is out of date, use one from an official release.


== Problem: Multitasking ==

It is strongly recommended that you write your own implementation of this and disregard the tutorial. The tutorial attempts to implement forking kernel threads by searching for magic values on the stack, which is insanity. If you wish to create a new kernel thread, simply decide which registers it should have and point its stack pointer at its freshly allocated stack. It will then start executing at your desired entry point. The part where it disables paging is bad and you should just map the source and destination physical frames at appropriate virtual addresses and memcpy with paging on at all times. Section 9.3 in particular is insanity and has blown up at least one well-established hobby operating system.


=== Inline Assembly optimiser problem with GCC 4.8 ===
As mentioned above, writing Inline Assembly can be tricky. The original Inline Assembly is this:

 <nowiki>
   asm volatile("         \
     cli;                 \
     mov %0, %%ecx;       \
     mov %1, %%esp;       \
     mov %2, %%ebp;       \
     mov %3, %%cr3;       \
     mov $0x12345, %%eax; \
     sti;                 \
     jmp *%%ecx           "
                : : "r"(eip), "r"(esp), "r"(ebp), "r"(current_directory->physicalAddr)); </nowiki>

Everything works fine when using gcc-4.2.4. However, the gcc-4.8.4 optimizer produces the following assembly (produced with '''objdump -d src/kernel'''):
 <nowiki>
  10387c:   fa                      cli
  10387d:   89 c1                   mov    %eax,%ecx
  10387f:   89 d4                   mov    %edx,%esp
  103881:   89 cd                   mov    %ecx,%ebp
  103883:   0f 22 db                mov    %ebx,%cr3
  103886:   b8 45 23 01 00          mov    $0x12345,%eax
  10388b:   fb                      sti
  10388c:   ff e1                   jmp    *%ecx</nowiki>

Note how the EAX register is assigned to the ECX register. However, later on the ECX register is assigned to EBP register. The reason for this is that the optimizer was using the EAX register to store the EIP variable and the ECX register to store the EBP variable. This results in the EIP variable being assigned to the ECX register ''as well as'' the EBP register. This leads to a subsequent '''ret''' statement sending the CPU to some invalid memory location.

A way to fix this is to remove the Inline Assembly by, for example adding this to '''process.s''':
 <nowiki>
; Here we:
; * Stop interrupts so we don't get interrupted.
; * Temporarily put the new EIP location in ECX.
; * Temporarily put the new page directory's physical address in EAX.
; * Set the base and stack pointers
; * Set the page directory
; * Put a dummy value (0x12345) in EAX so that above we can recognize that we've just
;   switched task.
; * Restart interrupts. The STI instruction has a delay - it doesn't take effect until after
;   the next instruction.
; * Jump to the location in ECX (remember we put the new EIP in there).

[GLOBAL perform_task_switch]
perform_task_switch:
     cli;
     mov ecx, [esp+4]   ; EIP
     mov eax, [esp+8]   ; physical address of current directory
     mov ebp, [esp+12]  ; EBP
     mov esp, [esp+16]  ; ESP
     mov cr3, eax       ; set the page directory
     mov eax, 0x12345   ; magic number to detect a task switch
     sti;
     jmp ecx</nowiki>

Then edit '''task.c''' and add this to the top of the file:
 <nowiki>
extern void perform_task_switch(u32int, u32int, u32int, u32int);</nowiki>
and replace the Inline Assembly with:
 <nowiki>
perform_task_switch(eip, current_directory->physicalAddr, ebp, esp);</nowiki>

== Problem: User mode ==

There are several problems. The downloadable code has everything fixed except a <tt>find_smallest_hole()</tt> page-aligned heap allocation bug.

=== Problem 1: nasm byte keyword causing <tt>0x80</tt> to become <tt>0xffffff80</tt> ===

This code is at fault:
 <nowiki>
 %macro ISR_NOERRCODE 1  ; define a macro, taking one parameter
  [GLOBAL isr%1]        ; %1 accesses the first parameter.
  isr%1:
    cli
    push byte 0
    push byte %1
    jmp isr_common_stub
%endmacro</nowiki>

It should be:

 <nowiki>
%macro ISR_NOERRCODE 1  ; define a macro, taking one parameter
  [GLOBAL isr%1]        ; %1 accesses the first parameter.
  isr%1:
    cli
    push byte 0
    push %1
    jmp isr_common_stub
%endmacro</nowiki>

=== Problem 2: Don't forget to allow interrupts in user mode in idt_set_gate ===

Find this comment in chapter 4 and uncomment the code:
 <nowiki>
   // We must uncomment the OR below when we get to using user-mode.
   // It sets the interrupt gate's privilege level to 3.
   idt_entries[num].flags   = flags /* | 0x60 */;</nowiki>

=== Problem 3: regs var must called by reference instead of by value in the irq and isr handlers ===
Various changes are needed.

Change this in <tt>page_fault()</tt>:
 <nowiki>
void page_fault(registers_t regs)
// to
void page_fault(registers_t *regs)</nowiki>

And in
<tt>isr_handler()</tt> and <tt>irq_handler()</tt>, change

 <nowiki>
handler(regs);</nowiki>
to
 <nowiki>
handler(&regs);</nowiki>

This fixes a problem where the syscall hander won't get called.

=== Problem 4: Missing documentation around set_kernel_stack ===

<tt>KERNEL_STACK_SIZE</tt> needs defining in <tt>task.h</tt>:
 <nowiki>
#define KERNEL_STACK_SIZE 2048       // Use a 2kb kernel stack.</nowiki>

Also, some code needs to be added to four sections in <tt>task.c</tt>

In <tt>initialise_tasking()</tt>
 <nowiki>
 current_task->kernel_stack = kmalloc_a(KERNEL_STACK_SIZE);</nowiki>

In <tt>fork()</tt>
 <nowiki>
current_task->kernel_stack = kmalloc_a(KERNEL_STACK_SIZE);</nowiki>

In <tt>switch_task()</tt>
 <nowiki>
set_kernel_stack(current_task->kernel_stack+KERNEL_STACK_SIZE);</nowiki>

In <tt>switch_to_user_mode()</tt>
 <nowiki>
set_kernel_stack(current_task->kernel_stack+KERNEL_STACK_SIZE);</nowiki>

=== Problem 5: find_smallest_hole() bug in heap code causing fork() to page fault ===

This bug from the heap chapter may not hit you until now. The bug results in the newly allocated <tt>kernel_stack</tt> messing up the page directory, causing <tt>clone_directory()</tt> to fail in <tt>fork()</tt>. See the heap section above for the details.

== Conclusion ==

The tutorial isn't bad as an example, but its design is not optimal and some parts of it are just plain bad (see multitasking). Indeed, you should just use it to get started and diverge from it as fast as possible, only using the tutorial when you need an example or can't implement it yourself. You should prefer consulting information on this wiki if possible. I haven't yet located all the problems in the tutorial, and some are quite minor and not technically problems but just small subjective design flaws. It is worth anticipating whether your future self will be removing tutorial code from your operating system and thus saving effort by never putting it there in the first place.

[[Category:OS Development]]
[[Category:Troubleshooting]]
[[Category:FAQ]]
````

<a id="w003460"></a>

## W003460 — Talk:Introduction

Bron-ID: W003460
Namespace: Talk (1)
Revisie: 15681 | Datum: 2014-03-01T18:55:09Z
Bron: https://wiki.osdev.org/Talk:Introduction
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AIntroduction&oldid=15681

````text
== Bran's tutorial? ==

This introductory article suggests following Bran's tutorial, which is ancient and known to be buggy rather than Bare Bones and the newer things that is being contributed to this site. It's likely a good idea to fix that soon. Indeed, it doesn't directly link to the classic bare bones tutorial. --[[User:Sortie|Sortie]] 16:44, 26 February 2014 (CST)

:There isn't any tutorial out there that actually produces a kernel of appreciable complexity, without it's accompanying share of bugs. Barebones is meant to be a project template that doesn't actually do anything interesting, and both Bran's and James' have documented bugs in quantities comparable to their size. That really leaves two options: You can trim out the tutorial option because suggesting barebones is already done and there's no better replacement, or secondly, making a blatant statement/warning along the lines that there aren't actually any tutorials out there because nobody succeeds at getting it right :). - [[User:Combuster|Combuster]] 12:55, 1 March 2014 (CST)
````

<a id="w003461"></a>

## W003461 — James Molloy's Known Bugs

Bron-ID: W003461
Namespace: Main (0)
Revisie: 15695 | Datum: 2014-03-07T07:16:33Z
Bron: https://wiki.osdev.org/James_Molloy%27s_Known_Bugs
Revisie-URL: https://wiki.osdev.org/index.php?title=James_Molloy%27s_Known_Bugs&oldid=15695
Doorverwijzing: James Molloy's Tutorial Known Bugs

````text
#REDIRECT [[James Molloy's Tutorial Known Bugs]]
````

<a id="w003462"></a>

## W003462 — User:Lionel/James Molloy's Kernel Development Tutorials

Bron-ID: W003462
Namespace: User (2)
Revisie: 15704 | Datum: 2014-03-10T02:15:44Z
Bron: https://wiki.osdev.org/User:Lionel/James_Molloy%27s_Kernel_Development_Tutorials
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALionel%2FJames_Molloy%27s_Kernel_Development_Tutorials&oldid=15704

````text
{{In Progress}}
James Molly's Kernel Development Tutorials are a set of 10 tutorials written in C that teach and guide an individual on how to create a unix-style kernel. This tutorial, while notorious for several bad design decisions, is still a great learning resource. It can be seen as a successor to Bran's Kernel Development Tutorials.

== Design Choices ==
The kernel created with this tutorial has some... interesting design choices. In this context "interesting" equals "potentially destructive". http://wiki.osdev.org/James_Molloy%27s_Tutorial_Known_Bugs points out most of them.
== The Tutorial==
The tutorial can be accessed at http://www.jamesmolloy.co.uk/tutorial_html/index.html
=== Google Code Version ===
There is an experimental version of the tutorial avalable at https://code.google.com/p/jamesm-tutorials/ . The tutorial seems to be more advanced, to the point of even including a working printf implementation.


== Analysis of the tutorial ==
This tutorial implements:
* A bootable kernel
* A text mode terminal
* Segmentation
* Interrupts & (x86) Exception support
* The PIT
* Paging
* A kernel heap
* A Virtual Filesystem & Initrd
* Preemptive Multitasking
* Usermode (Ring 3)


== Customising the Kernel ==
=== The Screen ===
I would recommend changing the function names to your liking. I would also recommend implementing printf().
=== Paging ===
Create your own paging implementation, as this one makes several bad design decisions which can cripple a kernel in the long run.
=== The heap ===
Create your own memory allocation system or use a premade one.
=== VFS & Initrd ===
Create your own Initrd filesystem.
````

<a id="w003463"></a>

## W003463 — User:Pancakes/StackBasedHeapImplementation

Bron-ID: W003463
Namespace: User (2)
Revisie: 15713 | Datum: 2014-03-13T21:05:11Z
Bron: https://wiki.osdev.org/User:Pancakes/StackBasedHeapImplementation
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APancakes%2FStackBasedHeapImplementation&oldid=15713

````text
== Stack Based ==
This uses a stack of 32-bit entries. I used 32-bit to keep alignment which I felt was important at the time that I wrote this. In any case it should still serve as an example of a way to implement a heap like this. Also, there is unused space at the start of the stack that could have been used for data if I improved the way it calculates the stack size and how it calculates it which I felt which over complicate the code. So this keeps it simple and I think if someone wanted to improve it then that would be a possible place. 

''I did think about using 16-bit entries and using an index in them which then could be multiplied with the block size to get an offset in the block. So this could be an option to reduce the size of the stack. However, it would limit the number of allocations per block to 0xffff which might not be a bad idea. I am just not sure at the moment. I might look into it further later.''

== Code ==
<pre>
typedef struct _KHEAPBLOCKSS {
	struct _KHEAPBLOCKSS	*next;
	uint32					top;
	uint32					max;
	uintptr					size;			/* total size in bytes including this header */
} KHEAPBLOCKSS;

typedef struct _KHEAPSS {
	KHEAPBLOCKSS			*fblock;
	uint32					bsize;
} KHEAPSS;

void k_heapSSInit(KHEAPSS *heap, uint32 bsize) {
	heap->fblock = 0;
	heap->bsize = bsize;
}

int k_heapSSAddBlock(KHEAPSS *heap, uintptr addr, uint32 size) {
	KHEAPBLOCKSS		*b;
	uint32				x;
	uint32				*stack;
	uint32				stacksz;
	uint32				slotres;
	
	b = (KHEAPBLOCKSS*)addr;
	b->next = heap->fblock;
	heap->fblock = b;
	
	b->size = size;
	
	size = size - sizeof(KHEAPBLOCKSS);
	
	b->max = size / (heap->bsize);
	
	stacksz = b->max * 4;
	slotres = (stacksz / heap->bsize) * heap->bsize < stacksz ? (stacksz / heap->bsize) + 1 : stacksz / heap->bsize;
	
	b->top = slotres;
	
	stack = (uint32*)&b[1];
	for (x = slotres; x < b->max; ++x) {
		stack[x] = x * heap->bsize;
	}
		
	return 1;
}

void *k_heapSSAlloc(KHEAPSS *heap, uint32 size) {
	KHEAPBLOCKSS		*b;
	uintptr				ptr;
	uint32				*stack;
	
	/* too big */
	if (size > heap->bsize) {
		return 0;
	}
	
	for (b = heap->fblock; b; b = b->next) {
		if (b->top != b->max) {
			stack = (uint32*)&b[1];
			ptr = stack[b->top++];
			ptr = (uintptr)&b[1] + ptr;
			return (void*)ptr;
		}
	}
	
	/* no more space left */
	return 0;
}

void k_heapSSFree(KHEAPSS *heap, void *ptr) {
	KHEAPBLOCKSS		*b;
	uintptr				_ptr;
	uint32				*stack;
	
	/* find block */
	_ptr = (uintptr)ptr;
	for (b = heap->fblock; b; b = b->next) {
		if (_ptr > (uintptr)b && _ptr < ((uintptr)b + b->size))
			break;
	}
	
	/* might want to catch this somehow or somewhere.. kinda means corruption */
	if (!b)
		return;
	
	/* get stack */
	stack = (uint32*)&b[1];
	/* normalize offset to after block header */
	/* decrement top index */
	/* place entry back into stack */
	stack[--b->top] = _ptr - (uintptr)&b[1];
	return;
}
</pre>
````

<a id="w003464"></a>

## W003464 — User:Johnburger

Bron-ID: W003464
Namespace: User (2)
Revisie: 23183 | Datum: 2019-01-08T09:27:21Z
Bron: https://wiki.osdev.org/User:Johnburger
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger&oldid=23183

````text
__TOC__
[http://stackexchange.com/users/8605261/john-burger http://stackexchange.com/users/flair/8605261.png]
== Who is John Burger? ==
I'm an Australian citizen that has been interested in software programming ever since I crashed my first mainframe at the age of 12 (1980). It was running a Tic&#8209;Tac&#8209;Toe program that refused to believe I could beat it three times in a row, and 'crashed' in a sulk when I did. '''''OBVIOUSLY''''' looking back on it I now realize that it was programmed that way - but at the time the feeling of terror, angst and panic made me the Ethical Hacker I am today.

== What is John's history? ==
I first started writing code in 1981. BASIC, Pascal and Forth were my first languages. It was only when I crashed a mini-computer's Pascal compiler that I realized that a compiler was merely yet another program, prone to the same foibles that beset other mere programs I myself had written.

I started learning assembly in 1984 on my [http://en.wikipedia.org/wiki/TRS-80_Color_Computer TRS-80 Color Computer] with a 6809 processor. When I joined the workforce in 1987 I soon learned x86 assembler, and wrote some TSRs and application libraries that were used for decades to come.

In 1990 I decided to write my first OS. The 386SX had come out, and was cheaper than other PCs at the time. The Olivetti PCS-386SX was a sturdy little workhorse, and its BIOS came with a ROM monitor! I could access RAM and the hard disk directly, without booting into an OS first! I still have three of these systems today.

My first tools were Borland's products (I bought TASM and preferred it over MASM), and started to hack and experiment. It formed the basis of my (abandoned) Masters project - alas, work took priority.

== What about now? ==
Most of my career has since been in 3GL - Pascal, C/C++ and Java - rather than the higher-level languages and frameworks that litter the IT landscape now. I guess you could call me a dinosaur - except that I'm currently writing C for x86, ARM and PowerPC embedded systems that keep me close to the hardware, where I feel most in control.

== So what happened to your OS? ==
I've been lurking on a number of websites (including this one) for a while now, and I've finally decided that I want to resurrect where I got up to. Most of my last couple of decades have included a teaching component, so I fully expect to write a [[User:Johnburger/Tutorials|couple of tutorials]] about low-level OS implementation. Look for some interesting content!

But step one was to resurrect my development environment - and it sucks! I use 64-bit systems now, and they refuse to run the old ('''OLD!''') tools that I grew up and got comfortable with. That's OK: surely the new tools are better now? '''WRONG!'''

Most of them refuse to produce for anything other than existing OSes like Windows or Linux. I want complete control. <em>Raw. Binary.</em> 16-bit for boot sectors, and transitioning to 32- and 64-bit as necessary. Nope: not happening. And unfortunately the ones that '''DO''' work aren't large-system friendly. Oh sure, they can assemble code, but they <em>don't</em> assume you've got a large code base, and they <em>do</em> assume that you are good enough to keep the '''ENTIRE''' source code in your head at the same time. Hey! I'm getting old! And I expect the tools to '''TELL''' me when I'm doing something foolish.

To this day, to my mind TASM is still the best assembler out there. I'd love to be proven wrong, but I have looked at quite a few... Luckily Borland produced 32-bit versions of their tools just before they imploded (Inprised?), so I'll just use those - except that most people don't have TASM anymore. It seems that the Netwide Assembler [http://www.nasm.us/ NASM] is the assembler-of-choice, so [[User:Johnburger/NASMCritique|as much as I don't like it]] my code here will be in NASM format.

== All right. But what can you do for us? ==
Hopefully, a lot. I've resurrected some of my early work, updated it for today's development environments, and have re-published my work for your edification. Please take a look at [[User:Johnburger/Demo|my latest demo]] - or indeed [[User:Johnburger/|any other of my Wiki pages]].
````

<a id="w003465"></a>

## W003465 — User:Johnburger/Tutorials

Bron-ID: W003465
Namespace: User (2)
Revisie: 21350 | Datum: 2017-08-28T21:03:17Z
Bron: https://wiki.osdev.org/User:Johnburger/Tutorials
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FTutorials&oldid=21350

````text
{{In Progress}}
= [[User:Johnburger|John's]] Tutorials =

== Ummm... ==
OK, I admit it. [[User:Johnburger|I'm]] a narcissist who wants to wax lyrical about my experiences. But if I can teach a couple of whippersnappers while I'm about it, who's to complain?

=== So, what's going here? ===
I've been working on the x86 platform since before there was the 80286. I've done embedded '186 work, as well as 8086-specific work on PCs practically since they came out. I've even worked on the 80376 ''(no Real Mode! Yay!)'' But most of the following will be about PC-specific things. It may be covered under other topics on this Wiki, but another perspective can always be useful... right?

= Demo =
Of course, having words to read is useful, but seeing <code>CODE</code> is sublime. To that end I've also put up a [[User:Johnburger/Demo|Demonstration System]] that implements many of these discussion topics.

= Booting =

= PC Hardware =
* [[User:Johnburger/PIC|Priority Interrupt Controller]]
* [[User:Johnburger/Timer|(Standard) Timer]]
* [[User:Johnburger/Keyboard|Keyboard Controller]]
* [[User:Johnburger/SerialPort|Serial Port]]
* [[User:Johnburger/RTC|Real-Time Clock]]

= 16-bit Protected Mode =
This is the mode invented for the 80286, which is still supported by its successors in 32-bit mode. Frankly, it's too limited to be of much interest to me: maximum of 64K segments; maximum of 16MB address space; why bother?

= 32-bit Protected Mode =

== Addressing ==

== Segmentation ==

== Paging ==

= 64-bit Protected Mode =
This is the "way of the future!" - according to everyone. Well, sort of. Why are Intel® still producing chips that don't have 64-bit support then? My biggest sorrow is that Intel didn't continue [[Segment Limits#Segmentation|Segmentation]] in the 64-bit version: it's still there in a truncated form, but all of the advantages of Segmentation have been lost.

I agree that for full-blown computers (workstations, desktops, laptops and even tablets) 64-bit is best - usually because of the bloat that GUIs impose! But for embedded systems, or simpler computers, 32-bit is still more than adequate.

== Addressing ==

== Segmentation ==

== Paging ==
````

<a id="w003466"></a>

## W003466 — User:Johnburger/PIC

Bron-ID: W003466
Namespace: User (2)
Revisie: 21379 | Datum: 2017-08-28T21:22:46Z
Bron: https://wiki.osdev.org/User:Johnburger/PIC
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FPIC&oldid=21379

````text
{{In Progress}}
= Priority Interrupt Controller (PIC) =

== Introduction ==
The 8086/8088 CPU used in the original IBM PC had a single External Interrupt input called INTR. If an attached device signalled this line, the CPU would stop what it was doing, remember where it was up to (saving certain registers on the stack), and start executing the interrupt handler. At the end, the interrupt handler would execute an IRET instruction and the CPU would return to what it was doing before it was interrupted. However, if a number of attached devices wanted to produce interrupts, they would have to share the single INTR pin and let the single interrupt handler sort out which device(s) were interrupting.

The Intel 8259 PIC was originally designed as a support chip for the 8086/8088 (and 8080/8085 - never mind!), to extend the CPU's single interrupt pin and allow a number of different devices to interrupt it. The PIC had 8 inputs, and it would prioritise these for the CPU to ensure that the more important interrupts were serviced before - or even during - less important ones. Of course, this complicated interrupt handling for all hardware interrupt handling code, but this complexity was seen as necessary for a true computer like the IBM PC.

== System design ==

{{Main |User:Johnburger/PIC/Background |l1=PIC Background }}

== Initialisation ==
The PIC by default provides access to only two of its registers: the Command register and the Interrupt Mask register. You can Command it to read its Interrupt Request Register or In-Service Register, but that's a two-step operation. The Master PIC uses I/O port addresses 020h and 021h respectively; the Slave uses I/O port addresses 0A0h and 0A1h respectively.

To initialise a PIC from scratch is a four-step operation: an explicit sequence of writes needs to be performed, and this sequence is different between the Master and the Slave:

<source lang="asm">
    MOV    AL, 00010001b    ; b4=1: Init; b3=0: Edge; b1=0: Cascade; b0=1: Need 4th init step
    OUT    020h, AL         ; Tell Master
    OUT    0A0h, AL         ; Tell Slave

    MOV    AL, 08h          ; Master's IRQ0 shoud be on INT 08h
    OUT    021h, AL         ; Tell Master
    MOV    AL, 70h          ; Slave's IRQ8 should be on INT 70h
    OUT    0A1h, AL         ; Tell Slave

    MOV    AL, 00000100b    ; Master has a Slave on input #2
    OUT    021h, AL         ; Tell Master
    MOV    AL, 02h          ; Slave is on Master's input #2
    OUT    0A1h, AL         ; Tell Slave

    MOV    AL, 00000001b    ; b4=0: FNM; b3-2=00: Master/Slave set by hardware; b1=0: Not AEOI; b0=1: x86 mode
    MOV    021h, AL         ; Tell Master
    MOV    0A1h, AL         ; Tell Slave
</source>

Note that at the end of the above initialisation sequence, all interrupts are masked off. You can read 021h or 0A1h and observe the bits which are set (this should be all of them!). Clear the appropriate bit (b0=IRQ0 or IRQ8) to enable the corresponding interrupt through to the CPU.

== Servicing an Interrupt ==

=== CPU Actions ===
When an external interrupt occurs, and the CPU's Interrupt flag is set, the CPU will start the Interrupt handling mechanism:
* If it is in Real Mode, it will:
** Push a minimal state onto the stack;
** Disable interrupts;
** Load the appropriate address from the Interrupt Vector Table (IVT at 0000h:0000h);
** Start executing at the new address.
* If it is in 32-bit Protected Mode, it will:
** Reference the appropriate entry in the Interrupt Descriptor Table;
** If necessary switch stacks or tasks to make sure it will be running in Ring 0 / Supervisor Mode;
** Push a minimal state onto the stack;
** If defined in the IDT, disable interrupts;
** Start executing at the new address.

=== Interrupt Handler actions ===
At this point the CPU is the programmer's responsibility. There is a fundamental minimum that the programmer <em>must</em> do: if they don't happen, the whole system will crash:
* Any registers used by the interrupt handler must be saved before being changed, and of course restored to their original values before the end of the interrupt handler. This amounts to all of them: the "minimal state" saved above is essentially only the CPU flags and the address of the interrupted instruction;
* Before the end of the interrupt, the appropriate PIC(s) <em>must</em> be sent an EOI command. Failure to do so will prevent this interrupt - and all lower-priority ones - from ever happening again.
* At the end of the interrupt, the interrupt handler must end with some form of the IRET command. This will fully restore the CPU's state before the interrupt occurred.

Apart from the above, there's something else that the programmer can <em>consider</em> doing: re-enable the CPU's interrupt flag. Doing so will allow the PICs' priority mechanisms to work again, allowing interrupts with a higher priority through.

==== Interrupt Handler considerations ====
Given these two independent actions, what (and when!) is the correct order to do them in?
* If interrupts are re-enabled too early, more important interrupts may interrupt this one before critical information can be retrieved from the device. This runs the risk of important information being lost;
* If the EOI is commanded before the device has been told that its interrupt has been serviced:
** The device may immediately re-signal the interrupt;
** The interrupt handler would immediately restart;
** It would again re-command the EOI;
** Which would re-signal the interrupt;
** Which would start the interrupt handler again...
In short, the CPU will be locked up until the stack runs out of room, crashing the system.
* If both the Master and the Slave PICs need an EOI sent to them, what is the correct order?
** If the Master is commanded first, it may immediately signal a low-priority interrupt that has been waiting for a while. This would leave the Slave PIC unacknowledged and unable to forward other interrupts until it was finally acknowledged when the CPU resumed the previous interrupt handler;
** If the Slave is commanded first, it may immediately signal a new low-priority interrupt of its own - but the Master PIC will ignore it until it has had its acknowledgement. At least in this scenario the Master will behave correctly.

It is obvious in the final example that it is critical that a Slave interrupt handler acknowledge the PICs in the order of Slave first, <em>then</em> Master.

==== Interrupt Handler EOI Command ====
To actually acknowledge that an interrupt has been handled, the interrupt handler has to send an EOI command to the appropriate PIC(s)' Command registers. As described above, this is likely to be a Non-specific EOI, which has the value 20h:

Known Master-only EOI command:
<source lang="asm">
    MOV    AL, 20h    ; EOI
    OUT    020h, AL   ; Tell Master
</source>

Known Slave-only EOI command:
<source lang="asm">
    MOV    AL, 20h    ; EOI Command
    OUT    0A0h, AL   ; Note Slave first!
    OUT    020h, AL   ; Then Master
</source>

If the same interrupt handler is used for both a potential Master and Slave interrupt, it is <em>very</em> important that only the relevant PIC(s) get commanded - failure to do so may crash the system.

= The new APIC - Advanced Priority Interrupt Controller =

== Legacy Mode ==

== See Also ==

=== Articles ===
* [[PIC]]
````

<a id="w003467"></a>

## W003467 — User:Johnburger/NASMCritique

Bron-ID: W003467
Namespace: User (2)
Revisie: 21143 | Datum: 2017-08-28T20:15:52Z
Bron: https://wiki.osdev.org/User:Johnburger/NASMCritique
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FNASMCritique&oldid=21143

````text
== Critiques from a [[TASM]] user ([[User:Johnburger|John Burger]] 19 March 2014) ==
NASM has some advantages over Borland's TASM (or TASM32 for running on 64-bt Windows), but a number of disadvantages which can lead to dangerous code:
=== Pros ===
* TASM is closed source, and will only run on Microsoft operating systems. NASM is open and portable.
* TASM only understands mnemonics up to around the Pentium Pro. NASM understands all of them.
* TASM can only produce OBJ files, which need to be linked into a final binary with a separate linker. NASM can also produce OBJ files, along with a number of other intermediate formats, but it can also produce final binary forms such as BIN, COM and even EXE (with a hack), obviating the need for a linker.
* NASM has a really neat structure definition syntax, meaning that you don't need to worry about structure fields being globally defined. In TASM and NASM, the following definition 'pollutes' the global namespace:
<source lang="asm">
         STRUC         Point
X        RESW          1
Y        RESW          1
         ENDSTRUC
</source>
After this definition, X and Y are forevermore defined. For unambiguity, you are forced to change your names to something like:
<source lang="asm">
         STRUC         Point
Pt_X     RESW          1
Pt_Y     RESW          1
         ENDSTRUC
</source>
But, NASM offers a solution: by leveraging the 'local variable' concept, you can define structure fields that require you to nominate the containing structure in future references:
<source lang="asm">
         STRUC         Point
.X       RESW          1
.Y       RESW          1
         ENDSTRUC

Cursor   ISTRUC        Point
         ENDISTRUC

         MOV           AX,[Cursor+Point.X]
         MOV           DX,[Cursor+Point.Y]
</source>
Personally, I'd prefer the following for the last lines - but NASM doesn't keep the type of variables so that cannot work:
<source lang="asm">
         MOV           AX,[Cursor.X]
         MOV           DX,[Cursor.Y]
</source>

=== Cons ===
* There is no type safety at all. If you define a variable
<source lang="asm">
Variable DW            1234h
</source>
then later accesses still don't know what size it is. The following will produce an 'Unknown size' error:
<source lang="asm">
         MOV           [Variable],5678h
</source>
The solution is that you MUST provide an override. But what if you provide the wrong one?
<source lang="asm">
         MOV           [Variable],EAX
         MOV DWORD PTR [Variable],12345678h
</source>
It will assemble fine, and your code will overwrite adjacent memory.

* There is no mechanism for structures within structures. If you define a Point structure:
<source lang="asm">
         STRUC         Point
.X       RESW          1
.Y       RESW          1
         ENDSTRUC
</source>
then there is no mechanism to include that structure within another one. Only fundamental types are allowed.

* There is no mechanism for bitfields, with their attendant <code>WIDTH</code>, <code>MASK</code> and <code>SHIFT</code> operators.

In short, they've provided a basic assembler with VERY powerful macro features - but their fundamental assumption that types are 'not important' allow the programmer using it to make very dangerous code.

Note that the [http://www.asmcommunity.net/projects/nasmx/ NASMX project] addresses some of these shortfalls (in particular the nested <code>STRUC</code> one) with some very sophisticated macros. Using this though defines practically a whole new language, necessitating the religious use of their macros over the fundamental types defined by the assembler.
````

<a id="w003468"></a>

## W003468 — ARM Integrator-CP IRQTimerAndPIC

Bron-ID: W003468
Namespace: Main (0)
Revisie: 30232 | Datum: 2026-03-16T15:01:28Z
Bron: https://wiki.osdev.org/ARM_Integrator-CP_IRQTimerAndPIC
Revisie-URL: https://wiki.osdev.org/index.php?title=ARM_Integrator-CP_IRQTimerAndPIC&oldid=30232

````text
{{FirstPerson}}

== ARM Integrator-CP IRQ, Timer, And PIC ==
This demonstrates initializing the exceptions vector table, timer 0, and the programmable interrupt controller.

{| class="wikitable"
! Tool
|-
| GCC
|-
| LD
|-
| OBJCOPY
|}

''I used GCC 4.4.5 and GCC 4.8.2 targeting the ARM EABI. Also, Binutils 2.23.91.20131118 and 2.20.1.''

=== Author ===
I [[User:Pancakes|Pancakes]] wrote this to help jump start you into developing for the ARM using QEMU or even a real piece of hardware. I have wrote software for both emulators and real hardware, but this has only been tested on QEMU so far. Please make any needed changes if you find problems. Also let me know at [mailto:kmcg3413@gmail.com kmcg3413@gmail.com] if you find this useful, have comments, or suggestions.

''You might want to double check the exceptions other than FIQ, IRQ, and SWINT because I may not be manipulating the LR register for a correct return if needed.''

''Also keep in mind that on a real board you may be flashing this code directly to FLASH (or ROM) (not RAM) and therefore you will not be able to write to global variables in your code (only read). You can avoid this by (1) not using global variables except only as read only. (2) Set the .bss and .data in the linker script to RAM, but then you will lose any initializers on that data. (3) Write a boot loader that loads this image in ELF (or similar) format. If it was placed in RAM there would be no problem.''

QEMU will load an ELF, but I designed this to be used not only with an emulator but also with real hardware. With QEMU you can actually write to your global variables but on real hardware you will not if they reside in FLASH.

Another trick to maintain a global state. Is to designate an area of memory to serve as a header for instance:
<syntaxhighlight lang="c">
#define KERNEL_HEAP    0x8000
#define KERNEL_HEAPSZ  0x1000

typedef struct _KERNEL_STATE {
    HEAP        heap;
    uint32      variable1;
    uint32      variable2;
} KERNEL_STATE;

KERNEL_STATE      *ks;

ks = (KERNEL_STATE*)KERNEL_HEAP;

heapInit(&ks->heap);
heapAddBlock(&ks->heap, KERNEL_HEAP + sizeof(KERNEL_STATE), KERNEL_HEAPSZ - sizeof(KERNEL_STATE));

/* load other areas of memory if needed */
heapAddBlock(&ks->heap, OTHERAREA, OTHERAREASZ);

</syntaxhighlight>

This allows you to designate an area of memory as part of your heap, while designating a structure to
exist at the beginning. Basically, KERNEL_STATE becomes your structure of global variables.

=== Source ===
<syntaxhighlight lang="c">
#ifdef B64
typedef unsigned long long  uintptr;
#else
typedef unsigned int		uintptr;
#endif
typedef unsigned long long  uint64;
typedef unsigned int		uint32;
typedef unsigned char		uint8;
typedef unsigned short		uint16;

#define ARM4_XRQ_RESET   0x00
#define ARM4_XRQ_UNDEF   0x01
#define ARM4_XRQ_SWINT   0x02
#define ARM4_XRQ_ABRTP   0x03
#define ARM4_XRQ_ABRTD   0x04
#define ARM4_XRQ_RESV1   0x05
#define ARM4_XRQ_IRQ     0x06
#define ARM4_XRQ_FIQ     0x07

#define CTRL_ENABLE			0x80
#define CTRL_MODE_FREE		0x00
#define CTRL_MODE_PERIODIC	0x40
#define CTRL_INT_ENABLE		(1<<5)
#define CTRL_DIV_NONE		0x00
#define CTRL_DIV_16			0x04
#define CTRL_DIV_256		0x08
#define CTRL_SIZE_32		0x02
#define CTRL_ONESHOT		0x01

#define REG_LOAD		0x00
#define REG_VALUE		0x01
#define REG_CTRL		0x02
#define REG_INTCLR		0x03
#define REG_INTSTAT		0x04
#define REG_INTMASK		0x05
#define REG_BGLOAD		0x06

#define PIC_IRQ_STATUS			0x0
#define PIC_IRQ_RAWSTAT			0x1
#define PIC_IRQ_ENABLESET		0x2
#define PIC_IRQ_ENABLECLR		0x3
#define PIC_INT_SOFTSET			0x4
#define PIC_INT_SOFTCLR			0x5

#define PIC_FIQ_STATUS			8
#define PIC_FIQ_RAWSTAT			9
#define PIC_FIQ_ENABLESET		10
#define PIC_FIQ_ENABLECLR		11

#define KSTACKSTART 0x2000
#define KSTACKEXC   0x4000

void start(void);

/*
    This could be non-standard behavior, but as long as this resides at the top of this source and it is the
    first file used in the linking process (according to alphanumerical ordering) this code will start at the
    beginning of the .text section.
*/
void __attribute__((naked)) entry()
{
	asm("mov sp, %[ps]" : : [ps]"i" (KSTACKSTART));
	/* send to serial output */
	asm("mov r1, #0x16000000");
	asm("mov r2, #65");
	asm("str r2, [r1]");
	/* call main kernel function */
	asm("bl	start");
}

#define KEXP_TOPSWI \
	uint32			lr; \
	asm("mov sp, %[ps]" : : [ps]"i" (KSTACKEXC)); \
	asm("push {lr}"); \
	asm("push {r0,r1,r2,r3,r4,r5,r6,r7,r8,r9,r10,r11,r12}"); \
	asm("mov %[ps], lr" : [ps]"=r" (lr));

#define KEXP_TOP3 \
	uint32			lr; \
	asm("mov sp, %[ps]" : : [ps]"i" (KSTACKEXC)); \
	asm("sub lr, lr, #4"); \
	asm("push {lr}"); \
	asm("push {r0,r1,r2,r3,r4,r5,r6,r7,r8,r9,r10,r11,r12}"); \
	asm("mov %[ps], lr" : [ps]"=r" (lr));
	
#define KEXP_BOT3 \
	asm("pop {r0,r1,r2,r3,r4,r5,r6,r7,r8,r9,r10,r11,r12}"); \
	asm("LDM sp!, {pc}^")

#define SERIAL_BASE 0x16000000
#define SERIAL_FLAG_REGISTER 0x18
#define SERIAL_BUFFER_FULL (1 << 5)

void k_serdbg_putc(char c)
{
    while (*(volatile unsigned long*)(SERIAL_BASE + SERIAL_FLAG_REGISTER) & (SERIAL_BUFFER_FULL));
	
    *(volatile unsigned long*)SERIAL_BASE = c;
}
	
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
	
void arm4_xrqenable_fiq()
{
    arm4_cpsrset(arm4_cpsrget() & ~(1 << 6));
}

void arm4_xrqenable_irq()
{
    arm4_cpsrset(arm4_cpsrget() & ~(1 << 7));
}
	
void k_exphandler(uint32 lr, uint32 type) {
	uint32			*t0mmio;
	uint32			swi;
	
	k_serdbg_putc('H');
	
	/*      clear interrupt in timer HW so it will lower its line to the PIC
                and therefore allow the PIC to lower its line to the CPU
	
		if you do not clear it, an interrupt will
		be immediantly raised apon return from this
		interrupt
	*/
	t0mmio = (uint32*)0x13000000;
	
	t0mmio[REG_INTCLR] = 1;
	
	/*
		Get SWI argument (index).
	*/
	if (type == ARM4_XRQ_SWINT) {
		swi = ((uint32*)((uintptr)lr - 4))[0] & 0xffff;
		
		if (swi == 4) {
			k_serdbg_putc('@');
		}
	}
	
	if (type != ARM4_XRQ_IRQ && type != ARM4_XRQ_FIQ && type != ARM4_XRQ_SWINT) {
		/*
			Ensure, the exception return code is correctly handling LR with the
			correct offset. I am using the same return for everything except SWI, 
			which requires that LR not be offset before return.
		*/
		k_serdbg_putc('!');
		for(;;);
	}
	
	return;
}
	
void __attribute__((naked)) k_exphandler_irq_entry() { KEXP_TOP3; k_exphandler(lr, ARM4_XRQ_IRQ); KEXP_BOT3; }
void __attribute__((naked)) k_exphandler_fiq_entry() { KEXP_TOP3;  k_exphandler(lr, ARM4_XRQ_FIQ); KEXP_BOT3; }
void __attribute__((naked)) k_exphandler_reset_entry() { KEXP_TOP3; k_exphandler(lr, ARM4_XRQ_RESET); KEXP_BOT3; }
void __attribute__((naked)) k_exphandler_undef_entry() { KEXP_TOP3; k_exphandler(lr, ARM4_XRQ_UNDEF); KEXP_BOT3; }	
void __attribute__((naked)) k_exphandler_abrtp_entry() { KEXP_TOP3; k_exphandler(lr, ARM4_XRQ_ABRTP); KEXP_BOT3; }
void __attribute__((naked)) k_exphandler_abrtd_entry() { KEXP_TOP3; k_exphandler(lr, ARM4_XRQ_ABRTD); KEXP_BOT3; }
void __attribute__((naked)) k_exphandler_swi_entry() { KEXP_TOPSWI;   k_exphandler(lr, ARM4_XRQ_SWINT); KEXP_BOT3; }

void arm4_xrqinstall(uint32 ndx, void *addr) {
	char buf[32];
    uint32      *v;
    
    v = (uint32*)0x0;
	v[ndx] = 0xEA000000 | (((uintptr)addr - (8 + (4 * ndx))) >> 2);
}

void start() {
	uint32		*t0mmio;
	uint32		*picmmio;

	arm4_xrqinstall(ARM4_XRQ_RESET, &k_exphandler_reset_entry);
	arm4_xrqinstall(ARM4_XRQ_UNDEF, &k_exphandler_undef_entry);
	arm4_xrqinstall(ARM4_XRQ_SWINT, &k_exphandler_swi_entry);
	arm4_xrqinstall(ARM4_XRQ_ABRTP, &k_exphandler_abrtp_entry);
	arm4_xrqinstall(ARM4_XRQ_ABRTD, &k_exphandler_abrtd_entry);
	arm4_xrqinstall(ARM4_XRQ_IRQ, &k_exphandler_irq_entry);
	arm4_xrqinstall(ARM4_XRQ_FIQ, &k_exphandler_fiq_entry);
	
	k_serdbg_putc('Z');
	
	asm("swi #4");
	
	/* enable IRQ */
	arm4_cpsrset(arm4_cpsrget() & ~(1 << 7));
	
	/* initialize timer and PIC 
	
		The timer interrupt line connects to the PIC. You can make
		the timer interrupt an IRQ or FIQ just by enabling the bit
		needed in either the IRQ or FIQ registers. Here I use the
		IRQ register. If you enable both IRQ and FIQ then FIQ will
		take priority and be used.
	*/
	picmmio = (uint32*)0x14000000;
	picmmio[PIC_IRQ_ENABLESET] = (1<<5) | (1<<6) | (1<<7);
	
	/*
		See datasheet for timer initialization details.
	*/
	t0mmio = (uint32*)0x13000000;
	t0mmio[REG_LOAD] = 0xffffff;
	t0mmio[REG_BGLOAD] = 0xffffff;			
	t0mmio[REG_CTRL] = CTRL_ENABLE | CTRL_MODE_PERIODIC | CTRL_SIZE_32 | CTRL_DIV_NONE | CTRL_INT_ENABLE;
	t0mmio[REG_INTCLR] = ~0;		/* make sure interrupt is clear (might not be mandatory) */
	
	k_serdbg_putc('K');
	k_serdbg_putc('\n');
	/* infinite loop */
	for(;;);
}
</syntaxhighlight>

=== Linker Script (file: link.ld) ===
This is saved in a file named ''link.ld'' which is referenced by the linker. QEMU loads the image to 0x10000, but on a real piece of hardware it might be different. This load position by QEMU may have something to do with the integrator-cp board, but I am not sure. Most ARM code is actually position independent code because the branch instructions do not address 32-bits and this is because of the 32-bit size of all instructions in ARM mode. But, when your code references symbols in the other sections it will use absolute addressing. It is possible in theory for GCC to emit position independent data references but I have found no standard option. I have heard of patches that add this ability to GCC, but just keep this in mind. I also yank out the text section in the build scripts below to make a flat binary and this is why I place everything into the text section.
<syntaxhighlight lang="text">
ENTRY (entry)

SECTIONS
{	
    . = 0x10000;
    _BOI = .;
    .text : { *(.text*) *(.rodata*) *(.data*) *(.bss*)}
    _EOI = .;
}
</syntaxhighlight>

=== Compile And Link ===
VERY IMPORTANT - I do not include linking with ''LIBGCC'' in this example because I know some of you may not be using a proper cross-compiler setup. BUT, you SHOULD link against ''LIBGCC'' because it is part of the GCC compiler. See [[ARM_Overview#Missing_Division_Functions_.28_aeabi_uidivmod.2C_aeabi_idiv.29_.2F_No_Division_Support|ARM Overview (Missing Division Functions / LIBGCC)]] and also [[Libgcc]].

''I only left it out because I would rather wet your appetite instead of run you away with this being difficult to compile or link.''

<syntaxhighlight lang="bash">
arm-eabi-gcc -s -I./inc -nostdlib -nostartfiles -ffreestanding -std=gnu99 -c *.c
arm-eabi-ld -T link.ld -o __arm.bin *.o
arm-eabi-objcopy -j .text -O binary __arm.bin arm.bin
</syntaxhighlight>

=== Test ===
<syntaxhighlight lang="bash">
qemu-system-arm -m 8 -kernel arm.bin -serial stdio
</syntaxhighlight>

=== Going Further ===
I would take a good look at all the parts of this demonstration and ensure you understand what each part does. I would recommend you pay careful attention to the exception entry and exit routines and make sure that you understand each instruction and how it works. This will save you a headache later on when things
are not acting quite right. But, here are some links to continue with. These are ''not'' the only links, but are just a few that I thought would be quite helpful.

{| class="wikitable"
! Page
! Description
|-
| [[Heap]] 
| Information about implementing heap
|-
| [[Memory_management|Memory Management]] 
| Information about higher level memory management
|-
| [[ARM_Overview|ARM Overview]] 
| See more tutorials and starting points for other things.
|-
| [http://infocenter.arm.com/help/topic/com.arm.doc.dui0159b/DUI0159B_integratorcp_1_0_ug.pdf Integrator Datasheet]
| Datasheet for the Integrator
|-
| [[ARM_Integrator-CP_IRQTimerAndPICAndTaskSwitch|IRQ, Timer, PIC, And Task Switch]]
| This page is a continuation of the above source code. It adds in a very simple task switch.
|}

[[Category:ARM]]
[[Category:Interrupts]]
````

<a id="w003469"></a>

## W003469 — ARM Integrator-CP IRQTimerAndPICAndTaskSwitch

Bron-ID: W003469
Namespace: Main (0)
Revisie: 28004 | Datum: 2023-07-09T17:47:33Z
Bron: https://wiki.osdev.org/ARM_Integrator-CP_IRQTimerAndPICAndTaskSwitch
Revisie-URL: https://wiki.osdev.org/index.php?title=ARM_Integrator-CP_IRQTimerAndPICAndTaskSwitch&oldid=28004

````text
{{FirstPerson}}

== ARM Integrator-CP IRQ, Timer, PIC, And Task Switching ==
This demonstrates initializing the exceptions vector table, timer 0, and the programmable interrupt controller. Then performing task switching between different tasks. It shows a simple method to switch
between different tasks.

=== Author ===
I [[User:Pancakes|Pancakes]] wrote this to help jump start you into developing for the ARM using QEMU or even a real piece of hardware. I have wrote software for both emulators and real hardware, but this has only been tested on QEMU so far. Please make any needed changes if you find problems. Also let me know at [mailto:kmcg3413@gmail.com kmcg3413@gmail.com] if you find this useful, have comments, or suggestions.

=== Notes ===
On ARM the SWI instruction disables IRQ interrupts on entry which is what I am using for task switching. So no switching occurs during an SWI which keeps the code simpler, and easy to understand. When the CPU switches into IRQ mode (interrupt) it is non-re-entrant unless an FIQ happens. I use no FIQ interrupts below.

The following points should be taken into account, and I think it would be good to leave it as an exercise to the reader. The fixes are simple, but I just wanted to make it clear what trouble you will run into when improving it to suit your needs.

''Also, the SWI instruction can not handle the same routine for swapping the thread state. It does not push the SPSR onto the stack like the KEXP_TOP3 and KEXP_BOT3 macros. So you will have to make the change there to allow it to save and restore the SPSR to change tasks during an SWI.''

''And, when I switch modes to grab the hidden registers I have it hard coded to switch back to IRQ mode which will be incorrect when switching from an SWI exception. So you need to save the current mode then restore it.''

=== Tools ===

{| class="wikitable"
! Tool
|-
| GCC
|-
| LD
|-
| OBJCOPY
|}

''I used GCC 4.4.5 and GCC 4.8.2 targeting the ARM EABI. Also, Binutils 2.23.91.20131118 and 2.20.1.''

=== Author ===
This is a continuation of the [[ARM_Integrator-CP_IRQTimerAndPIC|IRQ, Timer, And PIC]] demonstration. It may be easier to refer to that page and then come to this page as the source has been extended.

=== Source ===
<source lang="c">
#ifdef B64
typedef unsigned long long  uintptr;
#else
typedef unsigned int		uintptr;
#endif
typedef unsigned long long  uint64;
typedef unsigned int		uint32;
typedef unsigned char		uint8;
typedef unsigned short		uint16;

#define ARM4_XRQ_RESET   0x00
#define ARM4_XRQ_UNDEF   0x01
#define ARM4_XRQ_SWINT   0x02
#define ARM4_XRQ_ABRTP   0x03
#define ARM4_XRQ_ABRTD   0x04
#define ARM4_XRQ_RESV1   0x05
#define ARM4_XRQ_IRQ     0x06
#define ARM4_XRQ_FIQ     0x07

#define ARM4_MODE_USER   0x10
#define ARM4_MODE_FIQ	 0x11
#define ARM4_MODE_IRQ	 0x12
#define ARM4_MODE_SUPER  0x13
#define ARM4_MODE_ABORT	 0x17
#define ARM4_MODE_UNDEF  0x1b
#define ARM4_MODE_SYS    0x1f
#define ARM4_MODE_MON    0x16

#define CTRL_ENABLE			0x80
#define CTRL_MODE_FREE		0x00
#define CTRL_MODE_PERIODIC	0x40
#define CTRL_INT_ENABLE		(1<<5)
#define CTRL_DIV_NONE		0x00
#define CTRL_DIV_16			0x04
#define CTRL_DIV_256		0x08
#define CTRL_SIZE_32		0x02
#define CTRL_ONESHOT		0x01

#define REG_LOAD		0x00
#define REG_VALUE		0x01
#define REG_CTRL		0x02
#define REG_INTCLR		0x03
#define REG_INTSTAT		0x04
#define REG_INTMASK		0x05
#define REG_BGLOAD		0x06

#define PIC_IRQ_STATUS			0x0
#define PIC_IRQ_RAWSTAT			0x1
#define PIC_IRQ_ENABLESET		0x2
#define PIC_IRQ_ENABLECLR		0x3
#define PIC_INT_SOFTSET			0x4
#define PIC_INT_SOFTCLR			0x5

#define PIC_FIQ_STATUS			8
#define PIC_FIQ_RAWSTAT			9
#define PIC_FIQ_ENABLESET		10
#define PIC_FIQ_ENABLECLR		11

/*
	============== CONFIGURATION =============
*/
/* the intial kernel stack and the exception stack */
#define KSTACKSTART 0x2000
#define KSTACKEXC   0x3000
/* somewhere to place the kernel state structure */
#define KSTATEADDR	0x1000

/*
	============================================
*/

void start(void);

/*
    This could be non-standard behavior, but as long as this resides at the top of this source and it is the
    first file used in the linking process (according to alphanumerical ordering) this code will start at the
    beginning of the .text section.
*/
void __attribute__((naked)) entry()
{
	asm("mov sp, %[ps]" : : [ps]"i" (KSTACKSTART));
	/* send to serial output */
	asm("mov r1, #0x16000000");
	asm("mov r2, #65");
	asm("str r2, [r1]");
	/* call main kernel function */
	asm("bl	start");
}

#define KEXP_TOPSWI \
	uint32			lr; \
	asm("mov sp, %[ps]" : : [ps]"i" (KSTACKEXC)); \
	asm("push {lr}"); \
	asm("push {r0,r1,r2,r3,r4,r5,r6,r7,r8,r9,r10,r11,r12}"); \
	asm("mov %[ps], lr" : [ps]"=r" (lr));	

#define KEXP_BOTSWI \
	asm("pop {r0,r1,r2,r3,r4,r5,r6,r7,r8,r9,r10,r11,r12}"); \
	asm("LDM sp!, {pc}^")
		 
#define KEXP_TOP3 \
	uint32			lr; \
	asm("mov sp, %[ps]" : : [ps]"i" (KSTACKEXC)); \
	asm("sub lr, lr, #4"); \
	asm("push {lr}"); \
	asm("push {r0,r1,r2,r3,r4,r5,r6,r7,r8,r9,r10,r11,r12}"); \
	asm("mrs r0, spsr"); \
	asm("push {r0}"); \
	asm("mov %[ps], lr" : [ps]"=r" (lr));
	
#define KEXP_BOT3 \
	asm("pop {r0}"); \
	asm("msr spsr, r0"); \
	asm("pop {r0,r1,r2,r3,r4,r5,r6,r7,r8,r9,r10,r11,r12}"); \
	asm("LDM sp!, {pc}^")

#define SERIAL_BASE 0x16000000
#define SERIAL_FLAG_REGISTER 0x18
#define SERIAL_BUFFER_FULL (1 << 5)

void kserdbg_putc(char c)
{
    while (*(volatile unsigned long*)(SERIAL_BASE + SERIAL_FLAG_REGISTER) & (SERIAL_BUFFER_FULL));
	
    *(volatile unsigned long*)SERIAL_BASE = c;
}

void kserdbg_puts(const char * str)
{
    while (*str != 0) {
		kserdbg_putc(*str++);
	}
}

static char* itoh(int i, char *buf)
{
	const char 	*itoh_map = "0123456789ABCDEF";
	int			n;
	int			b;
	int			z;
	int			s;
	
	if (sizeof(void*) == 4)
		s = 8;
	if (sizeof(void*) == 8)
		s = 16;
	
	for (z = 0, n = 8; n > -1; --n)
	{
		b = (i >> (n * 4)) & 0xf;
		buf[z] = itoh_map[b];
		++z;
	}
	buf[z] = 0;
	return buf;
}

void ksprintf(char *buf, const char *fmt, ...)
{
	const char 				*p;
	__builtin_va_list		argp;
	int 					i;
	char 					*s;
	char 					fmtbuf[256];
	int						x, y;

	__builtin_va_start(argp, fmt);
	
	x = 0;
	for(p = fmt; *p != '\0'; p++)
	{
		if (*p == '\\') {
			switch (*++p) {
				case 'n':
					buf[x++] = '\n';
					break;
				default:
					break;
			}
			continue;
		}
	
		if(*p != '%')
		{
			buf[x++] = *p;
			continue;
		}

		switch(*++p)
			{
			case 'c':
				i = __builtin_va_arg(argp, int);
				buf[x++] = i;
				break;
			case 's':
				s = __builtin_va_arg(argp, char *);
				for (y = 0; s[y]; ++y) {
					buf[x++] = s[y];
				}
				break;
			case 'x':
				i = __builtin_va_arg(argp, int);
				s = itoh(i, fmtbuf);
				for (y = 0; s[y]; ++y) {
					buf[x++] = s[y];
				}
				break;
			case '%':
				buf[x++] = '%';
				break;
		}
	}
	
	__builtin_va_end(argp);
	buf[x] = 0;
}
	
uint32 arm4_cpsrget()
{
    uint32      r;
    
    asm("mrs %[ps], cpsr" : [ps]"=r" (r));
    return r;
}

uint32 arm4_spsrget()
{
    uint32      r;
    
    asm("mrs %[ps], spsr" : [ps]"=r" (r));
    return r;
}

void arm4_cpsrset(uint32 r)
{
    asm("msr cpsr, %[ps]" : : [ps]"r" (r));
}
	
void arm4_xrqenable_fiq()
{
    arm4_cpsrset(arm4_cpsrget() & ~(1 << 6));
}

void arm4_xrqenable_irq()
{
    arm4_cpsrset(arm4_cpsrget() & ~(1 << 7));
}

typedef struct _KTHREAD {
	uint8			valid;
	uint32			r0, r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12, sp, lr, cpsr, pc;
} KTHREAD;

typedef struct _KSTATE {
	KTHREAD			threads[0x10];
	uint8			threadndx;
	uint8			iswitch;
} KSTATE;

void k_exphandler(uint32 lr, uint32 type) {
	uint32			*t0mmio;
	uint32			*picmmio;
	uint32			swi;
	char			buf[128];
	KSTATE			*ks;
	int				x;
	KTHREAD			*kt;
	uint32			__lr, __sp, __spsr;
	
	ks = (KSTATE*)KSTATEADDR;
		
	kserdbg_putc('H');
	
	/*  clear interrupt in timer so it will lower its INT line
	
		if you do not clear it, an interrupt will
		be immediantly raised apon return from this
		interrupt
	*/
	
	if (type == ARM4_XRQ_IRQ) {
		picmmio = (uint32*)0x14000000;
		
		ksprintf(buf, "picmmio[PIC_IRQ_STATUS]:%x\n", picmmio[PIC_IRQ_STATUS]);
		kserdbg_puts(buf);
		
		/*
			It is possible that other pins are activated so we just check
			this one bit.
		*/
		if (picmmio[PIC_IRQ_STATUS] & 0x20) {
			t0mmio = (uint32*)0x13000000;
			t0mmio[REG_INTCLR] = 1;			/* according to the docs u can write any value */
			
			/* dont store registers on first switch */
			if (!ks->iswitch) {
				/*
					1. store register on stack in thread struct
					2. access hidden registers and store in thread struct
				*/
				kt = &ks->threads[ks->threadndx];
				kt->pc = ((uint32*)KSTACKEXC)[-1];
				kt->r12 = ((uint32*)KSTACKEXC)[-2];
				kt->r11 = ((uint32*)KSTACKEXC)[-3];
				kt->r10 = ((uint32*)KSTACKEXC)[-4];
				kt->r9 = ((uint32*)KSTACKEXC)[-5];
				kt->r8 = ((uint32*)KSTACKEXC)[-6];
				kt->r7 = ((uint32*)KSTACKEXC)[-7];
				kt->r6 = ((uint32*)KSTACKEXC)[-8];
				kt->r5 = ((uint32*)KSTACKEXC)[-9];
				kt->r4 = ((uint32*)KSTACKEXC)[-10];
				kt->r3 = ((uint32*)KSTACKEXC)[-11];
				kt->r2 = ((uint32*)KSTACKEXC)[-12];
				kt->r1 = ((uint32*)KSTACKEXC)[-13];
				kt->r0 = ((uint32*)KSTACKEXC)[-14];
				kt->cpsr = ((uint32*)KSTACKEXC)[-15];
				/*
				stack viewer
				
				for (x = 0; x < 16; ++x) {
					ksprintf(buf, "stack[%x]:%x\n", x, ((uint32*)KSTACKEXC)[-x]);
					kserdbg_puts(buf);
				}
				*/
				ksprintf(buf, "kt->lr:%x threadndx:%x\n", kt->lr, ks->threadndx);
				kserdbg_puts(buf);
				/* switch to system mode get registers then switch back */
				asm("    mrs r0, cpsr \n\
					 bic r0, r0, #0x1f \n\
					 orr r0, r0, #0x1f \n\
					 msr cpsr, r0 \n\
					 mov %[sp], sp \n\
					 mov %[lr], lr \n\
					 bic r0, r0, #0x1f \n\
					 orr r0, r0, #0x12 \n\
					 msr cpsr, r0 \n\
					 " : [sp]"=r" (__sp), [lr]"=r" (__lr));
				kt->sp = __sp;
				kt->lr = __lr;
				ksprintf(buf, "<---threadndx:%x kt->sp:%x kt->pc:%x kt->lr:%x\n", ks->threadndx, kt->pc, kt->lr);
				kserdbg_puts(buf);
			}
			/* store registers on next switch */
			
			/*
				get next thread (if not initial switch)
			*/
			if (!ks->iswitch) {
				for (ks->threadndx = (ks->threadndx + 1) & 0xf; !ks->threads[ks->threadndx].valid; ks->threadndx = (ks->threadndx + 1) & 0xf);
			}
			
			ks->iswitch = 0;
			
			kt = &ks->threads[ks->threadndx];
			ksprintf(buf, "--->threadndx:%x kt->pc:%x kt->lr:%x\n", ks->threadndx, kt->pc, kt->lr);
			kserdbg_puts(buf);
			/*
				load registers
			*/
			((uint32*)KSTACKEXC)[-1] = kt->pc;
			((uint32*)KSTACKEXC)[-2] = kt->r12;
 			((uint32*)KSTACKEXC)[-3] = kt->r11;
			((uint32*)KSTACKEXC)[-4] = kt->r10;
			((uint32*)KSTACKEXC)[-5] = kt->r9;
			((uint32*)KSTACKEXC)[-6] = kt->r8;
 			((uint32*)KSTACKEXC)[-7] = kt->r7;
			((uint32*)KSTACKEXC)[-8] = kt->r6;
			((uint32*)KSTACKEXC)[-9] = kt->r5;
			((uint32*)KSTACKEXC)[-10] = kt->r4;
			((uint32*)KSTACKEXC)[-11] = kt->r3;
			((uint32*)KSTACKEXC)[-12] = kt->r2;
			((uint32*)KSTACKEXC)[-13] = kt->r1;
			((uint32*)KSTACKEXC)[-14] = kt->r0;
			((uint32*)KSTACKEXC)[-15] = kt->cpsr;
			/* switch into system mode restore hidden registers then switch back */
			asm("    mrs r0, cpsr \n\
                                 bic r0, r0, #0x1f \n\
				 orr r0, r0, #0x1f \n\
				 msr cpsr, r0 \n\
				 mov sp, %[sp] \n\
				 mov lr, %[lr] \n\
                                 bic r0, r0, #0x1f \n\
				 orr r0, r0, #0x12 \n\
				 msr cpsr, r0 \n\
				 " : : [sp]"r" (kt->sp), [lr]"r" (kt->lr));
			/* go back through normal interrupt return process */
			return;
		}
	}
	
	/*
		Get SWI argument (index).
	*/
	if (type == ARM4_XRQ_SWINT) {
		swi = ((uint32*)((uintptr)lr - 4))[0] & 0xffff;
		
		if (swi == 4) {
			ksprintf(buf, "SWI cpsr:%x spsr:%x code:%x\n", arm4_cpsrget(), arm4_spsrget(), swi);
			kserdbg_puts(buf);
			kserdbg_putc('@');
		}
		
		return;
	}
	
	if (type != ARM4_XRQ_IRQ && type != ARM4_XRQ_FIQ && type != ARM4_XRQ_SWINT) {
		/*
			Ensure, the exception return code is correctly handling LR with the
			correct offset. I am using the same return for everything except SWI, 
			which requires that LR not be offset before return.
		*/
		kserdbg_putc('!');
		for(;;);
	}
	
	return;
}
	
void __attribute__((naked)) k_exphandler_irq_entry() { KEXP_TOP3; k_exphandler(lr, ARM4_XRQ_IRQ); KEXP_BOT3; }
void __attribute__((naked)) k_exphandler_fiq_entry() { KEXP_TOP3;  k_exphandler(lr, ARM4_XRQ_FIQ); KEXP_BOT3; }
void __attribute__((naked)) k_exphandler_reset_entry() { KEXP_TOP3; k_exphandler(lr, ARM4_XRQ_RESET); KEXP_BOT3; }
void __attribute__((naked)) k_exphandler_undef_entry() { KEXP_TOP3; k_exphandler(lr, ARM4_XRQ_UNDEF); KEXP_BOT3; }	
void __attribute__((naked)) k_exphandler_abrtp_entry() { KEXP_TOP3; k_exphandler(lr, ARM4_XRQ_ABRTP); KEXP_BOT3; }
void __attribute__((naked)) k_exphandler_abrtd_entry() { KEXP_TOP3; k_exphandler(lr, ARM4_XRQ_ABRTD); KEXP_BOT3; }
void __attribute__((naked)) k_exphandler_swi_entry() { KEXP_TOPSWI;   k_exphandler(lr, ARM4_XRQ_SWINT); KEXP_BOTSWI; }

void arm4_xrqinstall(uint32 ndx, void *addr) {
	char buf[32];
    uint32      *v;
    
    v = (uint32*)0x0;
	v[ndx] = 0xEA000000 | (((uintptr)addr - (8 + (4 * ndx))) >> 2);
}

void thread2() {
	int			x;
	
	for (;;) {
		for (x = 0; x < 0xfffff; ++x);
		kserdbg_putc('B');
	}
}

void thread1() {
	int			x;

	for (;;) {
		for (x = 0; x < 0xfffff; ++x);
		kserdbg_putc('A');
		asm("swi #4");
	}
}

void start() {
	uint32		*t0mmio;
	uint32		*picmmio;
	KSTATE		*ks;
	int			x;
	
	ks = (KSTATE*)KSTATEADDR;
	
	kserdbg_putc('Y');	
	
	/*
		============ SCHEDULER TASK SETUP =============
	*/
	/* lets scheduler know this is going to be the first switch */
	ks->iswitch = 1;
	
	for (x = 0; x < 0x10; ++x) {
		ks->threads[x].valid = 0;
	}
	
	/* this currently executing thread and stack will be discarded when these run */
	ks->threads[0].pc = (uint32)&thread1;
	ks->threads[0].valid = 1;
	ks->threads[0].cpsr = 0x60000000 | ARM4_MODE_USER;
	ks->threads[0].sp = 0x5000;
	ks->threads[0].ksp = 0x6000;
	
	ks->threads[1].pc = (uint32)&thread2;
	ks->threads[1].valid = 1;
	ks->threads[1].cpsr = 0x60000000 | ARM4_MODE_USER;
	ks->threads[1].sp = 0x7000;
	ks->threads[1].ksp = 0x8000;
	
	/* the first thread to run */
	ks->threadndx = 0x0;

	arm4_xrqinstall(ARM4_XRQ_RESET, &k_exphandler_reset_entry);
	arm4_xrqinstall(ARM4_XRQ_UNDEF, &k_exphandler_undef_entry);
	arm4_xrqinstall(ARM4_XRQ_SWINT, &k_exphandler_swi_entry);
	arm4_xrqinstall(ARM4_XRQ_ABRTP, &k_exphandler_abrtp_entry);
	arm4_xrqinstall(ARM4_XRQ_ABRTD, &k_exphandler_abrtd_entry);
	arm4_xrqinstall(ARM4_XRQ_IRQ, &k_exphandler_irq_entry);
	arm4_xrqinstall(ARM4_XRQ_FIQ, &k_exphandler_fiq_entry);
	
	kserdbg_putc('Z');
	
	/* enable IRQ */
	arm4_cpsrset(arm4_cpsrget() & ~(1 << 7));
	
	/* initialize timer and PIC 
	
		The timer interrupt line connects to the PIC. You can make
		the timer interrupt an IRQ or FIQ just by enabling the bit
		needed in either the IRQ or FIQ registers. Here I use the
		IRQ register. If you enable both IRQ and FIQ then FIQ will
		take priority and be used.
	*/
	picmmio = (uint32*)0x14000000;
	picmmio[PIC_IRQ_ENABLESET] = (1<<5) | (1<<6) | (1<<7);
	
	/*
		See datasheet for timer initialization details.
	*/
	t0mmio = (uint32*)0x13000000;
	t0mmio[REG_LOAD] = 0xffffff;
	t0mmio[REG_BGLOAD] = 0xffffff;			
	t0mmio[REG_CTRL] = CTRL_ENABLE | CTRL_MODE_PERIODIC | CTRL_SIZE_32 | CTRL_DIV_NONE | CTRL_INT_ENABLE;
	t0mmio[REG_INTCLR] = ~0;		/* make sure interrupt is clear (might not be mandatory) */
	
	kserdbg_putc('K');
	kserdbg_putc('\n');
	/* infinite loop */
	for(;;);
}
</source>

=== Linker Script (file: link.ld) ===
This is saved in a file named ''link.ld'' which is referenced by the linker. QEMU loads the image to 0x10000, but on a real piece of hardware it might be different. This load position by QEMU may have something to do with the integrator-cp board, but I am not sure. Most ARM code is actually position independent code because the branch instructions do not address 32-bits and this is because of the 32-bit size of all instructions in ARM mode. But, when your code references symbols in the other sections it will use absolute addressing. It is possible in theory for GCC to emit position independent data references but I have found no standard option. I have heard of patches that add this ability to GCC, but just keep this in mind. I also yank out the text section in the build scripts below to make a flat binary and this is why I place everything into the text section.
<source lang="text">
ENTRY (entry)

SECTIONS
{	
    . = 0x10000;
    _BOI = .;
    .text : { *(.text*) *(.rodata*) *(.data*) *(.bss*)}
    _EOI = .;
}
</source>

=== Compile And Link ===
VERY IMPORTANT - I do not include linking with ''LIBGCC'' in this example because I know some of you may not be using a proper cross-compiler setup. BUT, you SHOULD link against ''LIBGCC'' because it is part of the GCC compiler. See [[ARM_Overview#Missing_Division_Functions_.28_aeabi_uidivmod.2C_aeabi_idiv.29_.2F_No_Division_Support|ARM Overview (Missing Division Functions / LIBGCC)]] and also [[Libgcc]].

''I only left it out because I would rather wet your appetite instead of run you away with this being difficult to compile or link.''

<source lang="bash">
arm-eabi-gcc -s -I./inc -nostdlib -nostartfiles -ffreestanding -std=gnu99 -c *.c
arm-eabi-ld -T link.ld -o __arm.bin *.o
arm-eabi-objcopy -j .text -O binary __arm.bin arm.bin
</source>

=== Test ===
<source lang="bash">
qemu-system-arm -m 8 -kernel arm.bin -serial stdio
</source>

=== Next Phase ===
Now, that we have a preemptive multitasking system you might be wondering how we can improve? Well, for starters we could actually add in memory management and virtual memory.

With, [[User:Pancakes/ARM_Integrator-CP_IRQTimerPICTasksAndMM|IRQ, Timer, PIC, Tasks, And MM]].

=== Going Further ===
I would take a good look at all the parts of this demonstration and ensure you understand what each part does. I would recommend you pay careful attention to the exception entry and exit routines and make sure that you understand each instruction and how it works. This will save you a headache later on when things
are not acting quite right. But, here are some links to continue with. These are ''not'' the only links, but are just a few that I thought would be quite helpful.


{| class="wikitable"
! Page
! Description
|-
| [[ARM_Integrator-CP_IRQTimerAndPIC|IRQ, Timer, And PIC]]
| The starting base for this demonstration. It does not include task switching.
|-
| [[Heap]] 
| Information about implementing heap
|-
| [[Memory_management|Memory Management]] 
| Information about higher level memory management
|-
| [[ARM_Overview|ARM Overview]] 
| See more tutorials and starting points for other things.
|-
| [http://infocenter.arm.com/help/topic/com.arm.doc.dui0159b/DUI0159B_integratorcp_1_0_ug.pdf Integrator Datasheet]
| Datasheet for the Integrator
|-
| [[ARM_Integrator-CP_PL110_Dirty|PL110 - Color Display|QEMU PL110 Color Display]]
| Take your system to the next level with color graphics!
|-
| [[User:Pancakes/ARM_Integrator-CP_IRQTimerPICTasksAndMM|IRQ, Timer, PIC, Tasks, And MM]]
| Adds memory management including virtual memory.
|}


[[Category:ARM]]
[[Category:Interrupts]]
````

<a id="w003470"></a>

## W003470 — User:Johnburger/PIC/Background

Bron-ID: W003470
Namespace: User (2)
Revisie: 21342 | Datum: 2017-08-28T20:54:03Z
Bron: https://wiki.osdev.org/User:Johnburger/PIC/Background
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FPIC%2FBackground&oldid=21342

````text
== General PIC capabilities ==
The PIC is very flexible in its usage. As an example, I once programmed a system that had an 80188 CPU, an 8259 PIC, and 8x 16450 UART serial chips. After one of the UARTs interrupted the CPU, I had configured the PIC to "rotate" the priority of the interrupts to demote that UART to the bottom of the list, allowing others to get a fairer service.

But it would be true to say that well over 99% of all 8259 PICs (or their equivalents) are installed in PCs, which uses the PIC in only one of its many modes. Today the PIC's functionality, along with that of many other previously separate chips, has been moved into a single all-in-one support chip called the Southbridge.

== The PC: IBM made a Whoops! (or two... or...) ==
IBM's decision in 1980 to base its new Personal Computer on Intel's architecture arguably made Intel the world's largest chip manufacturer today. Not only the processor but also the support chips (the PIC was just one of the many Intel chips used) all worked together, were readily available, and were cheap. IBM just had to put them all together and make some design decisions: but at the time they weren't aware of how fundamentally some of those decisions were going to affect the future.

=== Intel specification ===
Intel's 8086/8088 processor supported up to 256 interrupts, up to 64 of which could be external interrupts (called Interrupt ReQuests or IRQs) using the 8259 PIC (see below). Intel predefined the first few interrupts (INTs) of the CPU to signal internal exceptions, such as Division by Zero or Debug, and quietly documented that they reserved the first 32 interrupts for future use - even though only the first 5 were currently defined.

IBM either missed the documented reservation, or ignored it since they weren't actually in use. They promptly and arbitrarily assigned various interrupts from 5 upwards for their own use: system calls, hardware interrupts (IRQs), and even simple pointers to data tables. (Tip: Don't ever make an INT call to one of those ones - unless you want to crash your computer?) For example, INT 5, the first one not in use, was adopted to perform a Screen Print (that smacks of an early debugging requirement to me...) At least when Microsoft added their own interrupts in MS-DOS they started from 32 (20h) - but that may be because IBM had already used most of the lower ones!

The effect of this decision became apparent when the PC was upgraded to use the 80286 and 80386. These newer processors used more of those Intel-reserved interrupts, which meant that executing a simple BOUND instruction on a PC could now cause the printer to burst into life! Worse, the IRQs (assigned to INTs 8 through 15 on the PC) interfered with more of the CPU's internal exceptions, complicating their handlers. Was that INT 13 caused by an internal General Protection Fault, or the Ethernet card?

This is why one of the first things that an OS writer on the modern PC platform needs to do is reprogram the PIC to get the IRQs <em>away</em> from the Intel-reserved exceptions! Blame it on IBM...

=== IRQ sharing ===
Another design decision - or at least a decision not to be more proscriptive in their documentation - also had an effect on the future of the new PC. The PC had an expansion bus where extra circuit boards could be installed, adding features such as communications or storage to the basic PC. Some of these expansion cards may need to interrupt the CPU, so the bus carried IRQs 2 through 7 (IRQs 0 and 1 were already dedicated to the timer and keyboard on the motherboard). Properly designed cards could use these IRQs in a cooperative manner, sharing the same interrupt with each other.

Unfortunately, most cards weren't properly designed in this respect - they assumed that they were the only one using a particular IRQ. That very quickly used up all the available IRQs, making adding new cards a tricky proposition for unskilled users. Moveable jumpers were required on expansion cards to move between available IRQs - and then the drivers for those cards needed to be informed of the jumper position. The invention of Plug-and-Play (PnP) could arguably be ascribed to this situation.

=== Level- versus Edge-triggered Interrupts ===
The final design compromise - with respect to IRQs, anyway - was actually one without a correct answer. The PIC supports two kinds of Interrupt modes: level-triggered and edge-triggered. Think of a pupil sitting in a classroom wanting to attract the attention of the teacher. She could raise her hand and keep it up until the teacher acknowledged her, or she could raise it and quickly lower it again, hoping to catch the teacher's eye. If the teacher had his back turned (busy doing something else) he may completely miss the second type, but with the first type the raised hand could obscure someone sitting behind the girl.

A level-triggered interrupt is raised for as long as the hardware is requesting service for the interrupt. If the CPU doesn't service the interrupt, which would allow the hardware to lower the interrupt line, the CPU cannot be told about other interrupts that may be pending.

However, an edge-triggered interrupt merely pulses the interrupt line, and if the CPU misses the pulse the device may go unserviced. Also, if the device decides it doesn't need an interrupt after all, there's no way for it to "take back" its interrupt pulse.

The designers of the original IBM PC decided to go for edge-triggered interrupts. Later designers of the PCI bus decided to go for level-triggered interrupts. Either way, the programmer who is writing the interrupt handler (you!) needs to carefully handle the device(s) assigned to an interrupt to discover if it actually needs servicing: never assume that the interrupt came from 'your' device, and always assume that there may be other devices hanging on the same interrupt.

One more issue to do with interrupts is the concept of a "spurious" interrupt. If a PIC sees an interrupt, it will immediately pass it on to the CPU - which may have interrupts disabled. By the time the CPU gets back to the PIC, the interrupt may have gone away (maybe the edge-triggered pulse was too short, or maybe the level-triggered interrupt simply stopped, or maybe electrical noise made a phantom spike). But the PIC is committed: it has to tell the CPU something. What it does is to signify an IRQ 7. The IRQ 7 handler then has to handle the possibility that 'its' associated device may not have been the cause of the interrupt: it must examine the PIC to see if there really was an interrupt, and simply ignore it if there wasn't. (Also, see "Working with the Slave" below).

== Adding more Interrupts - the IBM PC/AT ==
When IBM started work on the PC/AT, they had to increase the size of the expansion bus to accommodate the new 80286's 24-bit address space, so that gave them the opportunity to add more interrupts to the system. Luckily, the PIC was already designed to support extra interrupts by sacrificing one or more of its interrupt lines so that a new PIC could be wired on in a Master/Slave cascade arrangement - adding eight more interrupts each!

This immediately gave the designers a problem though. In the few years that the original IBM PC had been sold, various board manufacturers had already utilised all of the available IRQ lines, and removing one of the used lines would orphan a section of the market. They wanted to support all of the existing boards - so they devised a clever trick:

They decided to sacrifice IRQ 2, but then designate IRQ 9 as its replacement. They wired IRQ 9 where IRQ 2 used to be on the expansion bus (renaming it IRQ 2/9 in the process), then wrote a default IRQ 9 handler that would do the housekeeping necessary for the second PIC, and then simply jump to the original IRQ 2 handler. This meant that old IRQ 2 devices could work transparently on the new PC/AT, without any changes to the device driver, and without the device realising that it was actually using a new IRQ (let alone a different PIC).

Their other decision was which interrupts to assign the new IRQs to. Many of the interrupts had already been given well-known functions by various companies, and IBM needed a block of eight of them! They ended up choosing 70h-77h - at least it was out of the Intel-reserved space!

=== Working with the Master ===
Adding a new PIC didn't require much of a change to the code that handled the existing PIC. The initialisation code during system startup had to change, telling the old PIC that it was now a Master and where the Slave was, but all of the rest of the code could stay the same - none of the existing code for any of the current boards would have to change to work with the new PC/AT.

=== Working with the Slave ===
New code would have to be added to the system startup sequence to prepare the new PIC, telling it that it was a Slave and where it was connected to the Master.

And of course, the device drivers for any new boards that wanted to use the new IRQs would have to handle the new PIC. If they were on one of the original IRQs, they would have to perform housekeeping on just the old PIC as before. If they were on one of the new IRQs, they would have to perform housekeeping on both the old and the new PICs at startup, as well as during the interrupt handler.

== Interrupt Prioritisation ==
As described above, the PIC arbitrates which interrupt to pass on to the CPU in priority order - by default it designates the first input (IRQ0) as the highest priority. If a middle-priority interrupt occurs, the PIC will interrupt the CPU which will begin to execute the relevant interrupt handler. If that handler hasn't finished when a lower-priority interrupt comes in, the PIC will 'save' that interrupt until the CPU has finished with the earlier one. But if a higher-priority one comes in, the PIC will immediately attempt to interrupt the CPU.

That of course raises the question: how does the PIC know that the CPU has finished with the current interrupt so that it can pass on a lower-priority one? To resolve this, the PIC has two different modes:
* It can be set to deem that the interrupt is handled as soon as it successfully interrupts the CPU. Known as the Automatic End Of Interrupt (AEOI) mode, it means that the PIC can immediately signal the CPU on the next interrupt.
* Or it can be set to require the interrupt handler to send a command to the PIC when it has reached a suitable point in its processing - typically right at the end. This command is generically known as the End Of Interrupt (EOI) command, although the commonly used one is the Non-specific EOI: the PIC knows which interrupt it last gave the CPU, so it knows which one to complete when a (non-specific) EOI command is given, and the interrupt handler doesn't have to issue a Specific EOI.
The IBM PC designers decided that the AEOI mode was too dangerous and that they wanted explicit control over interrupts, so they only used the standard EOI mode.

But there is one independent issue that affects the PIC's prioritisation mechanism. When the CPU receives an interrupt, it saves its current state (represented by a few registers) onto the stack, clears the "Interrupt Enabled" flag (disabling future interrupts), and starts to execute the interrupt handler. When that handler is finished, the previous CPU state is restored (including the Interrupt flag state), and the normal processing resumes. However, for the period of the interrupt's execution, by default all future interrupts are disabled, defeating the PIC's priority handling mechanism. Unless the interrupt handler explicitly re-enabled interrupts, no new interrupts could be handled regardless of their priority. (Note that as of the 80286, the default "disable interrupts" mechanism can be changed on a per-interrupt basis.)

There's also a second issue that affects the prioritisation of interrupts, but this time only for the Slave PIC. If a low-priority interrupt occurs on the Slave PIC, it signals the Master PIC, which in turn interrupts the CPU. If a higher-priority interrupt were to then occur on the Slave PIC before the previous one was acknowledged, the Slave would again signal the Master PIC - which would then postpone it. The new interrupt wouldn't be of a higher priority than the one currently being serviced ("equal" is not "higher"), so the Master PIC would postpone it until the CPU was ready, again defeating the Slave PIC's priority system.

To combat this, the PIC designers provided another mode that could be enabled on the Master PIC:
* The standard cascade mode is named the "Fully Nested Mode" (FNM). When a Slave interrupt handler needed to acknowledge its interrupt, it would always send an EOI command to both the Master and Slave PICs.
* A second cascade mode was named the "Special Fully Nested Mode" (SFNM). In this mode, the Master PIC would allow interrupts from the <em>same</em> priority through, but <em>only</em> if they came from a Slave. In this mode though, a Slave interrupt handler <em>must not</em> acknowledge the Slave interrupt on the Master <em>unless</em> the Slave had finished all its interrupts. This requirement meant that all Slave interrupt handlers were more complicated than before, but worse, they introduced an uncertainty: although the Slave interrupt handler could determine that the Slave PIC was indeed finished, before the handler could get around to acknowledging the interrupt on the Master, another Slave interrupt could arrive.

The uncertainty of SFNM versus the lack of prioritisation of FNM meant that the IBM PC/AT designers decided to use the standard Fully Nested Mode to initialise their Master PIC.

==See Also==
===Articles===
* [[PIC|Priority Interrupt Controller]]
````

<a id="w003471"></a>

## W003471 — User:Pancakes/BitmapHeapImplementationEnhanced

Bron-ID: W003471
Namespace: User (2)
Revisie: 15824 | Datum: 2014-03-29T09:34:30Z
Bron: https://wiki.osdev.org/User:Pancakes/BitmapHeapImplementationEnhanced
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APancakes%2FBitmapHeapImplementationEnhanced&oldid=15824

````text
== Bitmap Heap Implementation Enhanced ==
I was working on another project and realized that I could use the same code for a heap as I could to manage physical memory. I did not want to clutter that page with lots of code so I placed it here.

The original page is [[User:Pancakes/BitmapHeapImplementation|here]] and contains a lot of information about using the heap.

This heap provides four extra functions.
<source lang="c">
int k_heapBMAddBlockEx(KHEAPBM *heap, uintptr addr, uint32 size, uint32 bsize, KHEAPBLOCKBM *b, uint8 *bm, uint8 isBMInside);
void *k_heapBMAllocBound(KHEAPBM *heap, uint32 size, uint32 mask);
uintptr k_heapBMGetBMSize(uintptr size, uint32 bsize);
void k_heapBMSet(KHEAPBM *heap, uintptr ptr, uintptr size, uint8 rval);
</source>

The ''k_heapBMAddBlockEx'' allows you to set the actual block management structure and ''bitmap'' in a different memory location. The is useful because normally the header will offset the actual data onto a CPU word sized boundary. While a CPU word sized (for example 4-byte/32-bit) boundary is useful for a normal heap, when working with say 4KB or 16KB boundary you will have problems and additional complications trying to make it work. For example if you wanted to manage memory from address 0 to the 8MB mark and make the bitmap handle chunks in 4KB sections all aligned on a boundary then the header is going to cause that to be impossible.

==== Two Ways To Initialize That Are Equivalent ====

To explain let me show you two ways to initialize the bitmap that are equivalent:
<source lang="c">
KHEAPBM       bmh;

#define SIZE  0x100000
#define BSIZE 16

/* the simple and normal way */
k_heapBMInit(&bmh);
k_heapBMAddBlock(&bmh, (uintptr)malloc(SIZE), SIZE, BSIZE);

/* the advanced way */
uintptr       addr;
uint8         *bm;
KHEAPBLOCKBM  *block;

k_heapBMInit(&bmh);
addr = (uintptr)malloc(SIZE);
b = (KHEAPBLOCKBM*)addr;
bm = (uint8*)(addr + sizeof(KHEAPBLOCKBM));
k_heapBMAddBlockEx(&bmh, addr + sizeof(KHEAPBLOCKBM), SIZE - sizeof(KHEAPBLOCKBM), BSIZE, b, bm, 1);
</source>

Both, produce the same result. In actuality when you call ''k_heapBMAddBlock'' it does this then it calls ''k_heapBMAddBlockEx'' where ''Ex'' stands for extension. Now, let me show you an advantage of using the extension function.

==== Allocating Header And Bitmap Outside ====

<source lang="c">
uintptr       addr;
uint8         *bm;
KHEAPBLOCKBM  *block;

k_heapBMInit(&bmh);
addr = (uintptr)malloc(SIZE);
b = (KHEAPBLOCKBM*)malloc(sizeof(KHEAPBLOCKBM));
bm = (uint8*)malloc(k_heapBMGetBMSize(SIZE, BSIZE));
k_heapBMAddBlockEx(&bmh, addr, SIZE, BSIZE, b, bm, 0);
</source>

Now, the bitmap and block header are stored outside of the originally allocated data. This is useful because if you want to alloc data aligned on a boundary beyond the CPU word size it is now possible because you have removed the block structure from offsetting your data. But, what about the bitmap? I mean the bitmap would have used whole blocks anyway right? Well, true, but the bitmap might have wasted some precious memory in a machine with a limited amount of memory to begin with because the bitmap will normally occupy whole blocks. So if your block size was 4K and you have 8MB of memory then the bitmap would have used an entire 4K (one block) instead of only 2K which might be kind of wasteful.

==== Allocate Header Outside With Bitmap At Beginning ====

My point is now you not only have control over removing the header, but also the ability to save a little space by also allocating the bitmap outside of the heap data and not have it subject to using whole blocks and that can save a little space.

Let me also show you how to leave the bitmap but allocate the header outside:
<source lang="c">
k_heapBMInit(&bmh);
addr = (uintptr)malloc(SIZE);
b = (KHEAPBLOCKBM*)malloc(sizeof(KHEAPBLOCKBM));
bm = (uint8*)addr;
k_heapBMAddBlock(&bmh, addr, SIZE, BSIZE, b, bm, 1);
</source>

You can notice the ''bm'' is set to the beginning and at the moment the implementation does not support it being set anywhere else unless you use the ''k_heapBMSet'' which is explained below.

==== Example To Manage Physical Memory Using Two Heaps ====

Let me show an example of using the bitmap implementation to manage physical memory.
<source lang="c">
        #define KRAMADDR    0x0               
        #define KCHKIOFF    (0x1000 * 4)       /* 16KB */
        #define KCHKISIZE   0x1000             /* 4KB */
        #define KRAMSIZE    (1024 * 1024 * 8)  /* 8 MB */

        KHEAPBM      hphy, hchk;

	k_heapBMInit(&hphy);
	k_heapBMInit(&hchk);
	/* get a bit of memory to start with for small chunk */
	k_heapBMAddBlock(&hchk, KCHKIOFF, KCHKISIZE, 16);
	/* add block but place header in chunk heap to keep alignment */
        /* also allocate bitmap inside chunk heap */
	k_heapBMAddBlockEx(&hphy, KRAMADDR, KRAMSIZE, 0x1000, (KHEAPBLOCKBM*)k_heapBMAlloc(&hchk, sizeof(KHEAPBLOCKBM)), (uint8*)k_heapBMAlloc(&hchk, k_heapBMGetBMSize(KRAMSIZE, 0x1000)), 0);
        k_heapBMSet(&hphy, KCHKIOFF, KCHKISIZE, 5);
</source>

Now, we used ''k_heapBMSet'' to remove the region ''KCHKIOFF'' of size ''KCHKISIZE'' by setting it as used inside the heap basically removing it from being allocated. Now, you can allocate on a 16KB boundary from the beginning of RAM by removing the header and bitmap, and still keeping track of them by allocating an initial small size heap.

''You want a 4KB page then use hphy. You need to allocate a structure then use hchk. You want 16KB then allocate from either hphy and hchk (you need support to code to make hchk allocate more memory if it does not have enough or runs out see blow).''

''You need 16KB allocated on a 16KB boundary then see one of the sections below.''

You may notice the last argument of ''k_heapBMSet'' is ''5''. Try not to use the value ''1'', ''0'', or ''2'' as the implementation uses that to designate allocations. If you use any of those you will find your region getting freed or allocated. So use something ''> 2''. You can use anything between ''3'' and ''255''.

==== Feeding Heap With Pages/Chunks/Sections ====

From this point you could keep your small chunk heap (''hchk'') feed by handing allocates from your larger size heap (''hphy''). The ''hphy'' stands for physical page heap and ''hchk'' for chunk heap.

<source lang="c">
void *kalloc(uintptr size) {
     void *ptr;
     /* access hphy and hchk... */
     ptr = k_heapBMAlloc(hchk, size);
     if (!ptr) {
         /* try allocation again... */
         if (size > 0x500) {
             k_heapBMAddBlock(hchk, k_heapAlloc(hphy, size * 2), size * 2);
         } else {
             k_heapBMAddBlock(hchk, k_heapAlloc(hphy, 0x1000), 0x1000);
         }
         ptr = k_heapBMAlloc(hchk, size);
     }
     /* either it worked or it did not */
     return ptr;
}
</source>

The above is a rough way to feed your small chunk heap with your larger physical page heap. It can be optimized. For example you might be able to allocate less than ''size * 2'' but I just used that because its whole number integer multiplication and simple to understand.

==== Allocating On Boundaries ===
<source lang="c">
uintptr   ptr;
ptr = k_heapBMAllocBound(hphy, 1024 * 16, 14);
</source>

Here we request a pointer to a memory region 16KB in size that is allocated on a 16KB boundary. The 16KB boundary is specified by asking for the bottom 14 bits to be zero. As 16KB is ''0x4000'' which has the bottom 14 bits set to zero. If it is unable to find a region that satisfies this condition then you get a null/zero pointer.

== Code ==
Here is the updated bitmap implementation providing the new extensions:
<source lang="c">
/* Leonard Kevin McGuire Jr (kmcg3413@gmail.com) (www.kmcg3413.net) */
typedef struct _KHEAPBLOCKBM {
	struct _KHEAPBLOCKBM	*next;
	uint32					size;
	uint32					used;
	uint32					bsize;
	uint32					lfb;
	uintptr					data;
	uint8					*bm;
} KHEAPBLOCKBM;

typedef struct _KHEAPBM {
	KHEAPBLOCKBM			*fblock;
} KHEAPBM;

void k_heapBMInit(KHEAPBM *heap);
int k_heapBMAddBlock(KHEAPBM *heap, uintptr addr, uint32 size, uint32 bsize);
int k_heapBMAddBlockEx(KHEAPBM *heap, uintptr addr, uint32 size, uint32 bsize, KHEAPBLOCKBM *b, uint8 *bm, uint8 isBMInside);
void *k_heapBMAlloc(KHEAPBM *heap, uint32 size);
void k_heapBMFree(KHEAPBM *heap, void *ptr);
uintptr k_heapBMGetBMSize(uintptr size, uint32 bsize);
void *k_heapBMAllocBound(KHEAPBM *heap, uint32 size, uint32 mask);
void k_heapBMSet(KHEAPBM *heap, uintptr ptr, uintptr size, uint8 rval);

void k_heapBMInit(KHEAPBM *heap) {
	heap->fblock = 0;
}

int k_heapBMAddBlock(KHEAPBM *heap, uintptr addr, uint32 size, uint32 bsize) {
	KHEAPBLOCKBM			*b;
	uintptr					bmsz;
	uint8					*bm;

	b = (KHEAPBLOCKBM*)addr;
	bmsz = k_heapBMGetBMSize(size, bsize);
	bm = (uint8*)(addr + sizeof(KHEAPBLOCKBM));
	/* important to set isBMInside... (last argument) */
	return k_heapBMAddBlockEx(heap, addr + sizeof(KHEAPBLOCKBM), size - sizeof(KHEAPBLOCKBM), bsize, b, bm, 1);
}

uintptr k_heapBMGetBMSize(uintptr size, uint32 bsize) {
	return size / bsize;
}

int k_heapBMAddBlockEx(KHEAPBM *heap, uintptr addr, uint32 size, uint32 bsize, KHEAPBLOCKBM *b, uint8 *bm, uint8 isBMInside) {
	uint32				bcnt;
	uint32				x;
	
	b->size = size;
	b->bsize = bsize;
	b->data = addr;
	b->bm = bm;
	
	b->next = heap->fblock;
	heap->fblock = b;

	bcnt = size / bsize;
	
	/* clear bitmap */
	for (x = 0; x < bcnt; ++x) {
		bm[x] = 0;
	}

	bcnt = (bcnt / bsize) * bsize < bcnt ? bcnt / bsize + 1 : bcnt / bsize;
	
	/* if BM is not inside leave this space avalible */
	if (isBMInside) {
		/* reserve room for bitmap */
		for (x = 0; x < bcnt; ++x) {
			bm[x] = 5;
		}
	}
	
	b->lfb = bcnt - 1;
	
	b->used = bcnt;
	
	return 1;
}

static uint8 k_heapBMGetNID(uint8 a, uint8 b) {
	uint8		c;	
	for (c = a + 1; c == b || c == 0; ++c);
	return c;
}

void *k_heapBMAlloc(KHEAPBM *heap, uint32 size) {
	return k_heapBMAllocBound(heap, size, 0);
}

void *k_heapBMAllocBound(KHEAPBM *heap, uint32 size, uint32 bound) {
	KHEAPBLOCKBM		*b;
	uint8				*bm;
	uint32				bcnt;
	uint32				x, y, z;
	uint32				bneed;
	uint8				nid;
	uint32				max;

	bound = ~(~0 << bound);
	/* iterate blocks */
	for (b = heap->fblock; b; b = b->next) {
		/* check if block has enough room */
		if (b->size - (b->used * b->bsize) >= size) {
			bcnt = b->size / b->bsize;		
			bneed = (size / b->bsize) * b->bsize < size ? size / b->bsize + 1 : size / b->bsize;
			bm = (uint8*)b->bm;
			
			for (x = (b->lfb + 1 >= bcnt ? 0 : b->lfb + 1); x != b->lfb; ++x) {
				/* just wrap around */
				if (x >= bcnt) {
					x = 0;
				}		
				
				/*
					this is used to allocate on specified boundaries larger than the block size
				*/
				if ((((x * b->bsize) + b->data) & bound) != 0)
					continue;	
				
				if (bm[x] == 0) {	
					/* count free blocks */
					max = bcnt - x;
					for (y = 0; bm[x + y] == 0 && y < bneed && y < max; ++y);

					/* we have enough, now allocate them */
					if (y == bneed) {
						/* find ID that does not match left or right */
						nid = k_heapBMGetNID(bm[x - 1], bm[x + y]);
						
						/* allocate by setting id */
						for (z = 0; z < y; ++z) {
							bm[x + z] = nid;
						}
						/* optimization */
						b->lfb = (x + bneed) - 2;

						/* count used blocks NOT bytes */
						b->used += y;
						return (void*)((x * b->bsize) + b->data);
					}
					
					/* x will be incremented by one ONCE more in our FOR loop */
					x += (y - 1);
					continue;
				}
			}
		}
	}
	
	return 0;
}

void k_heapBMSet(KHEAPBM *heap, uintptr ptr, uintptr size, uint8 rval) {
	KHEAPBLOCKBM		*b;	
	uintptr				ptroff, endoff;
	uint32				bi, x, ei;
	uint8				*bm;
	uint8				id;
	uint32				max;
	
	for (b = heap->fblock; b; b = b->next) {
		/* check if region effects block */
		if (
			/* head end resides inside block */
			(ptr >= b->data && ptr < b->data + b->size) ||
			/* tail end resides inside block */
			((ptr + size) >= b->data && (ptr + size) < b->data + b->size) ||
			/* spans across but does not start or end in block */
			(ptr < b->data && (ptr + size) > b->data + b->size)
		) {
			/* found block */
			if (ptr >= b->data) {
				ptroff = ptr - b->data;  /* get offset to get block */
				/* block offset in BM */
				bi = ptroff / b->bsize;
			} else {
				/* do not start negative on bitmap */
				bi = 0;
			}
			
			/* access bitmap pointer in local variable */
			bm = b->bm;

			ptr = ptr + size;
			endoff = ptr - b->data;
			
			/* end index inside bitmap */
			ei = (endoff / b->bsize) * b->bsize < endoff ? (endoff / b->bsize) + 1 : endoff / b->bsize;
			++ei;
			
			/* region could span past end of a block so adjust */
			max = b->size / b->bsize;
			max = ei > max ? max : ei;
			
			/* set bitmap buckets */
			for (x = bi; x < max; ++x) {
				bm[x] = rval;
			}
			
			/* update free block count */
			if (rval == 0) {
				b->used -= ei - bi;
			} else {
				b->used += ei - bi;
			}
			
			/* do not return as region could span multiple blocks.. so check the rest */
		}
	}
	
	/* this error needs to be raised or reported somehow */
	return;
}

void k_heapBMFree(KHEAPBM *heap, void *ptr) {
	KHEAPBLOCKBM		*b;	
	uintptr				ptroff;
	uint32				bi, x;
	uint8				*bm;
	uint8				id;
	uint32				max;
	
	for (b = heap->fblock; b; b = b->next) {
		if ((uintptr)ptr > b->data && (uintptr)ptr < b->data + b->size) {
			/* found block */
			ptroff = (uintptr)ptr - b->data;  /* get offset to get block */
			/* block offset in BM */
			bi = ptroff / b->bsize;
			/* .. */
			bm = b->bm;
			/* clear allocation */
			id = bm[bi];

			max = b->size / b->bsize;
			for (x = bi; bm[x] == id && x < max; ++x) {
				bm[x] = 0;
			}
			/* update free block count */
			b->used -= x - bi;
			return;
		}
	}
	
	/* this error needs to be raised or reported somehow */
	return;
}
</source>

==== Example Of Making The Most Of Memory On An ARM System ====
Just for an example. This is how I might start my memory management on an ARM based platform, but 
it could be very similar for the X86/X64 or another architecture. I like the ARM because it is a
bit simpler. The X86 has many more memory areas that need to be protected.

<source lang="c">
/* the intial kernel stack and the exception stack */
#define KSTACKSTART 0x2000		/* descending */
#define KSTACKEXC   0x3000		/* descending */
/* somewhere to place the kernel state structure */
#define KSTATEADDR	0x3000
/* the address of the start of usable RAM and it's length in bytes */
#define KRAMADDR	0x4000
#define KRAMSIZE	(1024 * 1024 * 8)
/* the size of a physical memory page */
#define KPHYPAGESIZE		4096
/* the block size of the chunk heap (kmalloc, kfree) */
#define KCHKHEAPBSIZE		16

/* KSTATE is a structure that serves as a global variable area for the kernel */
ks = (KSTATE*)KSTATEADDR;

k_heapBMInit(&ks->hphy);
k_heapBMInit(&ks->hchk);
/* get a bit of memory to start with for small chunk */
k_heapBMAddBlock(&ks->hchk, 4 * 7, KRAMADDR - (4 * 7), 16);

/* state structure */
k_heapBMSet(&ks->hchk, KSTATEADDR, sizeof(KSTATE), 5);
/* stacks (can free KSTACKSTART later) */
k_heapBMSet(&ks->hchk, KSTACKSTART - 0x1000, 0x1000, 6);
k_heapBMSet(&ks->hchk, KSTACKEXC - 0x1000, 0x1000, 7);

/* add block but place header and bitmap in chunk heap to keep alignment */
k_heapBMAddBlockEx(&ks->hphy, KRAMADDR, KRAMSIZE - KRAMADDR, 0x1000, (KHEAPBLOCKBM*)k_heapBMAlloc(&ks->hchk, sizeof(KHEAPBLOCKBM)), (uint8*)k_heapBMAlloc(&ks->hchk, k_heapBMGetBMSize(KRAMSIZE - KRAMADDR, 0x1000)), 0);
	
/* remove kernel too (you might have to remove it from hchk or hphy), in my case hphy */
k_heapBMSet(&ks->hphy, 0x10000, 0x4000, 8);
</source>

Something to handle is the amount of memory needed to map the maximum memory installed. You may
need to dynamically calculate ''KRAMADDR'' so you have enough space to allocate for the ''KHEAPBLOCKBM'' structure and the bitmap. For example for 4GB of memory and a 4K page size your bitmap would be 1MB in size. A simple check that displays a message may be sufficient. Such as a 
warning message that says the system was unable to initialize and why. But, actually increasing KRAMADDR if needed should work fine.

My chunk heap (small sized allocation heap) uses 16-byte blocks and is initial setup between 
memory address ''4*7'' and ''KRAMSTART''. The RAM actually starts at 0x0 so it is a bit misleading. I then
allocate the memory needed for the physical page heap from my chunk heap and set it to go from ''KRAMADDR'' with size ''KRAMADDR - KRAMSIZE'' since I used a bit out of the total RAM size.

Then I set each region as used in my small chunk heap where the initial kernel stack resides and the exception stack. I also skipped past the exception table with ''4*7''. This helps to make the most of my potentially limited memory by allocating 16-byte blocks to live between those areas. Once task switching is up and running and I can free the area occupied by the initial kernel stack.

Also, notice I use a ''6'' for the initial kernel stack and a ''7'' for the exception stack. This is because if I used the same number and tried to free the initial kernel stack it would free both. Unless, I used the k_heapBMSet and gave it a ''0'' which is also another way, but I just preferred to 
keep them separate so I did not have to reference the ''KSTACKSTART'' again when releasing the memory.

==== Original Implementation ====

For the original implementation see [[User:Pancakes/BitmapHeapImplementation|Bitmap Heap Implementation]]. I kept these separate to reduce the complexity of people just looking for a basic heap implementation with only the minimal necessary functionality. I would recommend that check out that page just to get an idea of what has changed.

[[Category:Memory management]]
[[Category:Heap]]
````

<a id="w003472"></a>

## W003472 — User:Pancakes/ARM Integrator-CP IRQTimerPICTasksAndMM

Bron-ID: W003472
Namespace: User (2)
Revisie: 16198 | Datum: 2014-04-15T17:49:08Z
Bron: https://wiki.osdev.org/User:Pancakes/ARM_Integrator-CP_IRQTimerPICTasksAndMM
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APancakes%2FARM_Integrator-CP_IRQTimerPICTasksAndMM&oldid=16198

````text
== IRQ, Timer, PIC, Tasks, And MM On The Integrator-CP ==
Well, hopefully, you are coming from some of the earlier examples. But, if not here are the links.

{| class="wikitable"
! Page
! Description
|-
| [[ARM_Integrator-CP_IRQTimerAndPIC|IRQ, Timer, And PIC]]
| This demonstration just uses the IRQ, Timer, And PIC.
|-
| [[ARM_Integrator-CP_IRQTimerAndPICAndTaskSwitch|IRQ, Timer, PIC, And Tasks]]
| This shows how to switch between tasks using the timer, and builds from the previous page.
|}

=== Author ===
I [[User:Pancakes|Pancakes]] wrote this to help jump start you into developing for the ARM using QEMU or even a real piece of hardware. I have wrote software for both emulators and real hardware, but this has only been tested on QEMU so far. Please make any needed changes if you find problems. Also let me know at [mailto:kmcg3413@gmail.com kmcg3413@gmail.com] if you find this useful, have comments, or suggestions.

=== Note ===
''If you are working with the VMSAv7 such as with the cortex-a9 then there is no bit to disable sub-pages (no backwards compatibility). In the code below I disable sub-pages. So you should omit setting bit 23 of the control register when enabling paging on VMSAv7 systems.''

=== About ===

I am going to extend the later demonstration of task switching. But, first we are going to need to add in some basic memory management, and then progress to virtual memory afterwards.

==== Adding A Heap Implementation ====
I am going to use a fairly simple heap implementation, [[User:Pancakes/BitmapHeapImplementationEnhanced|Bitmap Heap Implementation Enhanced]]. I would recommend that you read about it because we will be using all of it's features to implement a memory management system, and it will be needed to implement virtual memory. Also, even if you use your own implementation you might find it helpful to have a general idea of what the codes does. It might even be self explanatory for some readers.

To compile and link I would recommend that you place the structures and prototypes in a header called ''kheap_bm.h'' and place the implementation into ''kheap_bm.c''. Then when compiling the kernel code just include the header and link with the ''kheap_bm.o'' that you should produce and everything should work great.

=== Adding The Basic Memory Initialization ===
We are going to be basing everything from one of the previous demonstrations shown in the links at the top of this page. To make this page easier to read I am only going to duplicate the code needed to demonstrate. The first modification we are going to make is to the ''KSTATE'' structure.

<source lang="c">
typedef struct _KSTATE {
	KTHREAD			threads[0x10];
	uint8			threadndx;	
	uint8			iswitch;
	KHEAPBM			hphy;		/* physical page heap */
	KHEAPBM			hchk;		/* data chunk heap */
} KSTATE;
</source>

I have added two new fields ''hphy'' and ''hchk''. The ''hphy'' will serve as our physical memory manager working with 4KB pages while ''kchk'' will work with smaller allocations. If you are using the older demonstration you may be missing the ''KSTATE'' structure completely. 

Lets make some defines:
<source lang="c">
/* the intial kernel stack and the exception stack */
#define KSTACKSTART 0x2000		/* descending */
#define KSTACKEXC   0x3000		/* descending */
/* somewhere to place the kernel state structure */
#define KSTATEADDR	0x3000
/* 
	RAM is assumed to start at 0x0, but we have to leave room for a little setup code, and
	depending on how much physical memory (KRAMSIZE) we are using we might have to adjust
	KRAMADDR to give a bit more. For example if KRAMSIZE is 4GB then KRAMADDR needs to be
	larger than 1MB, perferrably 2MB at least.
*/
#define KRAMADDR	0x200000
#define KRAMSIZE	(1024 * 1024 * 8)
/* 
	kernel virtual memory size 
	
	KMEMINDEX can be 1 through 7
	1 = 2GB
	2 = 1GB
	3 = 512MB
	4 = 256MB
	5 = 128MB
	6 = 64MB
	7 = 32MB
	
	This is the maximum amount of virtual memory per kernel space.
*/
#define KMEMINDEX   3
#define KMEMSIZE	(0x1000000 << (8 - KMEMINDEX))
/* the size of a physical memory page */
#define KPHYPAGESIZE		4096
/* the block size of the chunk heap (kmalloc, kfree) */
#define KCHKHEAPBSIZE		16
/* minimum block size for chunk heap */
#define KCHKMINBLOCKSZ		(1024 * 1024)
</source>

You may already have some defines. Just replace them and add any new ones. One important one is ''KSTATEADDR'' which defines where our kernel state resides.

Now, let us add some entries into our ''main'' function of our play kernel.
<source lang="c">
void start() {
	.........
	KSTATE      *ks;
	uint8       *bm;

	ks = (KSTATE*)KSTATEADDR;

	/* create physical page heap */
	k_heapBMInit(&ks->hphy);
	k_heapBMInit(&ks->hchk);
	k_heapBMInit(&ks->husr);
	/* get a bit of memory to start with for small chunk */
	k_heapBMAddBlock(&ks->hchk, 4 * 7, KRAMADDR - (4 * 7), KCHKHEAPBSIZE);
	
	/* state structure */
	k_heapBMSet(&ks->hchk, KSTATEADDR, sizeof(KSTATE), 5);
	/* stacks (can free KSTACKSTART later) */
	k_heapBMSet(&ks->hchk, KSTACKSTART - 0x1000, 0x1000, 6);
	k_heapBMSet(&ks->hchk, KSTACKEXC - 0x1000, 0x1000, 7);
	
	/*
		I like to identity map my page tables in kernel space because it makes them
		easy to walk. To do so I have to try to reserve physical memory in the lower
		half. To do this I have two heaps of physical 4K pages. 
		
		The user heap is any memory past the KMEMSIZE, and anything below is the kernel
		heap. Pages are first allocated out of the user heap for any user space process,
		and if that fails then they are allocated from the kernel heap.
	*/
	if (KRAMSIZE > KMEMSIZE) {
		/* let hphy represent lower memory for kernel so it can be identity mapped */
		bm = (uint8*)k_heapBMAlloc(&ks->hchk, k_heapBMGetBMSize(KMEMSIZE - KRAMADDR, KPHYPAGESIZE));
		k_heapBMAddBlockEx(&ks->hphy, KRAMADDR, KMEMSIZE - KRAMADDR, KPHYPAGESIZE, (KHEAPBLOCKBM*)k_heapBMAlloc(&ks->hchk, sizeof(KHEAPBLOCKBM)), bm, 0);
		/* let husr represent upper memory which does not need to be identity mapped */
		bm = (uint8*)k_heapBMAlloc(&ks->hchk, k_heapBMGetBMSize(KRAMSIZE - KMEMSIZE, KPHYPAGESIZE));
		k_heapBMAddBlockEx(&ks->husr, KMEMSIZE, KRAMSIZE - KMEMSIZE, KPHYPAGESIZE, (KHEAPBLOCKBM*)k_heapBMAlloc(&ks->hchk, sizeof(KHEAPBLOCKBM)), bm, 0);		
	} else {
		/* add block but place header in chunk heap to keep alignment */
		bm = (uint8*)k_heapBMAlloc(&ks->hchk, k_heapBMGetBMSize(KRAMSIZE - KRAMADDR, KPHYPAGESIZE));
		k_heapBMAddBlockEx(&ks->hphy, KRAMADDR, KRAMSIZE - KRAMADDR, KPHYPAGESIZE, (KHEAPBLOCKBM*)k_heapBMAlloc(&ks->hchk, sizeof(KHEAPBLOCKBM)), bm, 0);
	}
		
	/* 
		remove kernel image region 
		
		This ensures it does not reside in either one. Because, KRAMADDR can change we can not
		be sure if it resides in which one or if it spans both somehow so to be safe this works
		quite well.
	*/
	k_heapBMSet(&ks->hphy, (uintptr)&_BOI, (uintptr)&_EOI - (uintptr)&_BOI, 8);
	k_heapBMSet(&ks->hchk, (uintptr)&_BOI, (uintptr)&_EOI - (uintptr)&_BOI, 8);
	k_heapBMSet(&ks->husr, (uintptr)&_BOI, (uintptr)&_EOI - (uintptr)&_BOI, 8);
    ..............
</source>

I prefer to have the exception handlers setup before this code. I do not show that above, but you can simply move their initialization code to above this code block. The first objective is to create a small allocation heap, ''hchk'', to handle small allocations then place the remaining RAM as 4KB pages into our second heap known as ''hphy''. The ''phy'' part stands for physical, and the ''chk'' stands for chunk. The ''hchk'' heap uses 16-byte blocks and the ''hphy'' uses 4K blocks. 

You should notice that I first:
<source lang="c">
	k_heapBMAddBlock(&ks->hchk, 4 * 7, KRAMADDR - (4 * 7), KCHKHEAPBSIZE);
</source>
This adds a region of memory starting ''after'' the exception table (''4*7'') and extends to the beginning of what I define as remaining RAM with ''KRAMADDR''. RAM actually starts at 0x0, so do not let it be misleading. It is just that I had no better name for it.

Next:
<source lang="c">
	/* state structure */
	k_heapBMSet(&ks->hchk, KSTATEADDR, sizeof(KSTATE), 5);
	/* stacks (can free KSTACKSTART later) */
	k_heapBMSet(&ks->hchk, KSTACKSTART - 0x1000, 0x1000, 6);
	k_heapBMSet(&ks->hchk, KSTACKEXC - 0x1000, 0x1000, 7);
</source>
Here we set some regions as used in the bitmap. They contain the kernel's initial stack and the exception stack. My exceptions are non-re-entrant so I can get away with one stack for now. Also notice that our heap grows downward which is why I subtract 0x1000. It just makes the code loading the ''sp'' register simpler. Also, I consider each stack to be 4KB, but you could change this to a lower amount if you desired. Also, if you have read the [[User:Pancakes/BitmapHeapImplementationEnhanced|Bitmap Heap Implementation Enhanced]] page you will know that I am distinguishing each specific region with a different identifier in the bitmap by using ''5'', ''6'', and ''7''.
I only remove them from ''hchk'' because I think it is fairly safe to assume ''KRAMADDR'' will not be lower than ''KSTACKEXC'', but you could add some
type of check to be sure someone compiling your kernel does not happen to make ''KRAMADDR'' too low.

Finally:
<source lang="c">
	if (KRAMSIZE > KMEMSIZE) {
		/* let hphy represent lower memory for kernel so it can be identity mapped */
		bm = (uint8*)k_heapBMAlloc(&ks->hchk, k_heapBMGetBMSize(KMEMSIZE - KRAMADDR, KPHYPAGESIZE));
		k_heapBMAddBlockEx(&ks->hphy, KRAMADDR, KMEMSIZE - KRAMADDR, KPHYPAGESIZE, (KHEAPBLOCKBM*)k_heapBMAlloc(&ks->hchk, sizeof(KHEAPBLOCKBM)), bm, 0);
		/* let husr represent upper memory which does not need to be identity mapped */
		bm = (uint8*)k_heapBMAlloc(&ks->hchk, k_heapBMGetBMSize(KRAMSIZE - KMEMSIZE, KPHYPAGESIZE));
		k_heapBMAddBlockEx(&ks->husr, KMEMSIZE, KRAMSIZE - KMEMSIZE, KPHYPAGESIZE, (KHEAPBLOCKBM*)k_heapBMAlloc(&ks->hchk, sizeof(KHEAPBLOCKBM)), bm, 0);		
	} else {
		/* add block but place header in chunk heap to keep alignment */
		bm = (uint8*)k_heapBMAlloc(&ks->hchk, k_heapBMGetBMSize(KRAMSIZE - KRAMADDR, KPHYPAGESIZE));
		k_heapBMAddBlockEx(&ks->hphy, KRAMADDR, KRAMSIZE - KRAMADDR, KPHYPAGESIZE, (KHEAPBLOCKBM*)k_heapBMAlloc(&ks->hchk, sizeof(KHEAPBLOCKBM)), bm, 0);
	}
	
	/* remove kernel image region */
	k_heapBMSet(&ks->hphy, (uintptr)&_BOI, (uintptr)&_EOI - (uintptr)&_BOI, 8);
	k_heapBMSet(&ks->hchk, (uintptr)&_BOI, (uintptr)&_EOI - (uintptr)&_BOI, 8);
	k_heapBMSet(&ks->husr, (uintptr)&_BOI, (uintptr)&_EOI - (uintptr)&_BOI, 8);
</source>
One very important difference is I allocate the block header and bitmap outside of that memory region using ''k_heapBMAddBlockEx''. 
The ''Ex'' function adds more control over where the structures are allocated.
I also allocate three heaps. The ''hphy'' is physical memory that can be used by the kernel for identity mapping, and 'husr'' 
is memory that can be used for either user space or kernel space. I
make ''hphy'' for allocating memory for the page tables because they need to be identity mapped in my kernel just to make 
it easier to walk the tables. The ''husr'' is for any pages not inside kernel space. The ''husr'' heap could be empty, and
this is okay since we will just use pages from ''hphy''.

Then I remove the region of memory where the kernel image is loaded. You could manually enter values here, but I used a symbol
created by the linker script to mark the beginning and end of the kernel. I also remove it from ''hphy'', ''hchk'', and ''husr'' because
KRAMADDR could change and it is actually possible it could cause the kernel to span any of them. So just to be safe I try to remove it
from both heap.
==== kmalloc and kfree ====
At this point you can now implement a ''kmalloc'' and ''kfree'' if you like:
<source lang="c">
void kfree(void *ptr) {
	KSTATE			*ks;
	
	ks = (KSTATE*)KSTATEADDR;

	k_heapBMFree(&ks->hchk, ptr);
}

void* kmalloc(uint32 size) {
	void			*ptr;
	KSTATE			*ks;
	uint32			_size;	
	
	ks = (KSTATE*)KSTATEADDR;
	
	/* attempt first allocation try (will fail on very first) */
	ptr = k_heapBMAlloc(&ks->hchk, size);
	
	/* try adding more memory if failed */
	if (!ptr) {
		if (size < KCHKMINBLOCKSZ / 2) {
			/* we need to allocate blocks at least this size */
			_size = KCHKMINBLOCKSZ;
		} else {
			/* this is bigger than KCHKMINBLOCKSZ, so lets double it to be safe */
			/* round up allocation to use all the blocks taken */
			_size = size * 2;
			_size = (_size / KPHYPAGESIZE) * KPHYPAGESIZE < _size ? _size / KPHYPAGESIZE + 1 : _size / KPHYPAGESIZE;
			_size = _size * KPHYPAGESIZE;
		}
		ptr = k_heapBMAlloc(&ks->hphy, _size);
		if (!ptr) {
			/* no more physical pages */
			return 0;
		}
		/* try allocation once more, should succeed */
		k_heapBMAddBlock(&ks->hchk, (uintptr)ptr, _size, KCHKHEAPBSIZE);
		ptr = k_heapBMAlloc(&ks->hchk, size);
	}
	
	return ptr;
}
</source>

=== A Quick Primer On Virtual Memory ===
If your brand new to virtual memory you might have a difficul time grasping what in the word is going on. So let me
try to explain and then give you some links.

Basically, for starters the connection between CPU and memory looks kinda like this:
<pre>
        CPU <-----> MEMORY
</pre>
Now, that is simple right? The CPU accessess RAM directly. Well, lets add something else in there.
<pre>
        CPU <-----> MMU/TLB <-----> MEMORY
</pre>
Basically, the MMU has all addresses lines (at least) pass through it. This gives it the ability to modify
the address and change it. The TLB is the actual buffer that holds either the whole table in memory or parts
of it. When you set the address of a TLB (or likely more accurately a table) it uses this to translate the
addresses. You are basically working with virtual memory addresses when you enable the MMU and it translates
them into physical addresses. This process is transparent to the CPU. The CPU has no idea the address is being
changed which is why you have to invalidate caches and even invalidate the tables the MMU has cached when you
change the mapping through your tables.

Virtual memory allows you to do things such as disk swapping where you pretend you have more memory in the
machine than you really do by swapping unused sections to disk, and replacing them with newly accessed
section. A section can be any size and depends on your MMU configuration and your architecture. It also
allows you to present each application with a different view of memory and can handles fragmentation of
your physical memory by arranging the physical pages that may be seperated into a contigious range using
virtual memory mapping. Also, on most architectures virtual memory allows you to create security by restricting
access to certain areas of memory with different modes such as read only or read/write only for privileged code.

Your best place for further reading is using Google to search for virtual memory and read up on it. I am afraid
I can only explain so much here, but I hope I have basically given you the basics to get you on your feet a 
little instead of feeling completely lost.

<pre>TODO: add links to virtual memory in our wiki</pre>

=== Tinkering With Virtual Memory On the ARM complying with VMSA v6 ===
I comform to the VMSA v6 specification. This deals with 16MB super-sections, 1MB sections, 64K entries, and 
4K entries. If your coming from an X86/X64 background your going to find it to be almost identical, except
instead of having 4MB entries on your first level table you have 1MB. The most flexible configuration is 
having your first level table map ''whole'' 1MB sections or use a coarse table to map 4KB pages in that 1MB
region. 64K entries and 16MB entries are essentially similar except one is used on the first level table, and
the other on the second level table.

Now that we have some basic memory management we can proceed to the fun stuff. Lets start really small until we get a grasp of what we are working with.

We need a few utility functions to make life easier:
<source lang="c">
void arm4_tlbset0(uint32 base) {
	asm("mcr p15, 0, %[tlb], c2, c0, 0" : : [tlb]"r" (base));
}

void arm4_tlbset1(uint32 base) {
	asm("mcr p15, 0, %[tlb], c2, c0, 1" : : [tlb]"r" (base));
}

void arm4_tlbsetmode(uint32 val) {
	asm("mcr p15, 0, %[tlb], c2, c0, 2" : : [tlb]"r" (val));
}

void arm4_tlbsetdom(uint32 val) {
	asm("mcr p15, 0, %[val], c3, c0, 0" : : [val]"r" (val));
}

uint32 arm4_tlbgetctrl() {
	uint32			ctrl;
	asm("mrc p15, 0, r0, c1, c0, 0 \n\
	     mov %[ctrl], r0" : [ctrl]"=r" (ctrl));
	return ctrl;
}

void arm4_tlbsetctrl(uint32 ctrl) {
	asm("mcr p15, 0, %[ctrl], c1, c0, 0" : : [ctrl]"r" (ctrl));
}
</source>

==== Defines ====
<source lang="c">
	/* first level table */
	#define TLB_FAULT			0x000		/* entry is unmapped (bits 32:2 can be used for anything) but access generates an ABORT */
	#define TLB_SECTION			0x002		/* entry maps 1MB chunk */
	#define TLB_COARSE			0x001		/* sub-table */
	#define TLB_DOM_NOACCESS	0x00		/* generates fault on access */
	#define TLB_DOM_CLIENT		0x01		/* checked against permission bits in TLB entry */
	#define TLB_DOM_RESERVED	0x02		/* reserved */
	#define TLB_DOM_MANAGER		0x03		/* no permissions */
	#define TLB_STDFLAGS		0xc00		/* normal flags */
	/* second level coarse table */
	#define TLB_C_LARGEPAGE		0x1			/* 64KB page */
	#define TLB_C_SMALLPAGE		0x2			/* 4KB page */
	/* AP (access permission) flags for coarse table [see page 731 in ARM_ARM] */
	#define TLB_C_AP_NOACCESS	(0x00<<4)	/* no access */
	#define TLB_C_AP_PRIVACCESS	(0x01<<4)	/* only system access  RWXX */
	#define TLB_C_AP_UREADONLY	(0x02<<4)	/* user read only  RWRX */
	#define TLB_C_AP_FULLACCESS	(0x03<<4)	/* RWRW */	
	/* AP (access permission) flags [see page 709 in ARM_ARM; more listed] */
	#define TLB_AP_NOACCESS		(0x00<<10)	/* no access */
	#define TLB_AP_PRIVACCESS	(0x01<<10)	/* only system access  RWXX */
	#define TLB_AP_UREADONLY	(0x02<<10)	/* user read only  RWRX */
	#define TLB_AP_FULLACCESS	(0x03<<10)	/* RWRW */
</source>

==== Example Enabling Paging ====
You might notice the word TLB and this stands for transition look-aside buffer. Which, is what we will be
working with to implement paging. If you are coming from X86/X64 architecture you might have already read
or tinkered with paging and it is basically the same on the ARM. One major difference is ARM can work with
two paging tables at the same time. This is intended for one to be used for the kernel and another one to
be process specific. I am going to use them backwards (''in this example'') because the docs state for 
TTBR0 to be process specific
and TTBR1 to be operating system specific, but TTBR1 gets mapped into the upper 2GB of the 32-bit address
space and TTBR0 in the lower and since our kernel lives low in this code example I am going to use TTBR0 for it,
and it keeps the demonstration simple.

''You might want to consider higher half kernel. Meaning, your kernel is linked for
in upper memory not lower memory because it provides a number of advantages. My example
below is reversed to keep things simple since we are already loaded low in memory.''

You can also further divide the space as follows (not evenly):
{| class="wikitable"
! TTBR0
! TTBR1
|-
| 2GB
| 2GB
|-
| 1GB
| 3GB
|-
| 512MB
| 3584MB
|-
| ...
| ...
|}

It can actually reach 32MB for the TTBR0 and 4064MB for TTBR1. We are just going to divide it with 2GB for our initial tinkering.

===== Code =====
<source lang="c">
	/*
		This says:
		1. Use the hphy heap.
		2. Allocate 16KB (1024 * 16)
		3. Make sure the 16 least significant bits are zero.
	*/
	ktlb = k_heapBMAllocBound(&ks->hphy, 1024 * 16, 14);
	utlb = k_heapBMAllocBound(&ks->hphy, 1024 * 16, 14);
	
	/* 
		enable the usage of utlb when bit 31 of virtual address is not zero 
	*/
	arm4_tlbsetmode(1);
	
	/* 
           We are going to identity map 2GB of space.

           We only need to set 2048 entries, because the remaining entries
           are ignored in the ktlb. But AFAIK the utlb still needs an entire 16KB which
           appears to me to be proved by the code below...
        */
	for (x = 0; x < 2048; ++x) {
		ktlb[x] = (x << 20) | TLB_ACCESS | TLB_SECTION;
	}
	
	/* 
		Do a little trick to map the same memory twice but in both
		page tables (TTBR0 and TTBR1)
	*/
	utlb[0x800] = (1 << 20) | TLB_ACCESS | TLB_SECTION;
	utlb[0x801] = (1 << 20) | TLB_ACCESS | TLB_SECTION;
	ktlb[1] = (2 << 20) | TLB_ACCESS | TLB_SECTION;
	ktlb[2] = (2 << 20) | TLB_ACCESS | TLB_SECTION;
	
	kserdbg_puts("OK\n");
	
	/* load location of TLB */
	arm4_tlbset1((uintptr)utlb);
	arm4_tlbset0((uintptr)ktlb);
	/* set that all domains are checked against the TLB entry access permissions */
	arm4_tlbsetdom(0x55555555);
	/* enable TLB (0x1) and disable subpages (0x800000) */
	arm4_tlbsetctrl(arm4_tlbgetctrl() | 0x800001);
	
	kserdbg_puts("OK2\n");
	
	/* check trick - as you can see the same 1MB chunk is mapped twice */
	a = (uint32*)0x80000000;
	b = (uint32*)0x80100000;
	a[0] = 0x12345678;
	ksprintf(buf, "utlb:%x\n", b[0]);
	kserdbg_puts(buf);

	a = (uint32*)0x100000;
	b = (uint32*)0x200000;
	a[0] = 0x87654321;
	ksprintf(buf, "ktlb:%x\n", b[0]);
	kserdbg_puts(buf);

	kserdbg_puts("DONE\n");
	for(;;);
</source>
We have mapped the one region twice at 1MB and 2MB, and mapped the another region twice at 2048MB and 2049MB. We verify it by writing to one region and reading it from another region. ''Now, this has been tested in QEMU only. On read hardware we could have missed a thing or two. But, this is the basics and if you have done it correctly it should work! Congratulations, you have enabled paging and not crashed your emulator (or maybe real hardware)!

The CPU actually supports second level tables for a finer grain of mapping, and even some other configurations that provide mapping of 16MB regions. I just used 1MB sections because it is easier to understand at first.

==== arm4_tlbsetmode(1) ====
''This value can be between 0 and 7. I am using a 1 in the above code.''

Basically, 0 disables the usage of the TTBR1 which means we only use a single TLB. 
In this case your would have to not only swap the TLB on each process change, but 
also update changes to any kernel space on all TLBs for every process.

Any value greater than 0 enables the usage of TTBR1. But, it also represents
the number of highest bits in the address that are used to select between TTBR1
and TTBR0. For example:
<pre>
VALUE		BITS IF ZERO USE TTBR0      TTBR0 ADDRESSABLE SPACE     TTBR0 SIZE IN ENTRIES (1MB SECTIONS)
0			-----						4GB (TTBR0)					4096
1			31							2GB (TTBR0)					2048
2			31:30						1GB (TTBR0)					1024
3			31:29						512MB (TTBR0)				512
4			31:28						256MB (TTBR0)				256
5			31:27						128MB (TTBR0)				128
6			31:26						64MB (TTBR0)				64
7			31:25						32MB (TTBR0)				32
</pre>
If you set the appropriate mode and all the bits in the range are zero then it will
use TTBR0, and if any are set it will use TTBR1. If you set the value to 0 then it
will ''ONLY'' use TTBR0.

Each value also effects how much of the TTBR0 table we have to initialize. So if we use
the value 2 like below. Then we only have to have 1024 entries (of 1MB sections).
However, TTBR1 will always need the full number of entries.

From my research it seems that even though the ARM manual recommends TTBR0 be used for
user space and TTBR1 be used for kernel space you can swap them if desired. The major down
side is that your will be allocating a full 16KB for each process at a minimum if using
1MB sections with (or with out) second level tables (second level table further divides the 1MB).

On the upside you can give more address space to user space. Of course you could use some of
the TTBR0 for user space, but in that case you would either have to share that space with all
processes (or groups of processes) or swap out both TTBR0 and TTBR1 which would seem to be a 
disadvantage because you would lose the ability to keep a portion of memory (kernel space) validated
in the TLB for a performance increase.

==== arm4_tlbsetdom(0x55555555); ====
Each entry can be assigned a domain value. There are only 16 possible domains. Each domain can be assigned
a rough permission, and this can be changed on the fly.
<source lang="c">
	#define TLB_DOM_NOACCESS	0x00		/* generates fault on access */
	#define TLB_DOM_CLIENT		0x01		/* checked against permission bits in TLB entry */
	#define TLB_DOM_RESERVED	0x02		/* reserved */
	#define TLB_DOM_MANAGER		0x03		/* no permissions */
</source>
Each domain is represented by 2 bits. With domain zero starting with the least significant bits.
''See page 742 in the ARM architecture manual''. 
<source lang="c">
	#define TLB_DOM(x)			(x<<5)		/* domain value (16 unique values) */
	#define DOM_TLB(x,y)		(x<<(y*2))	/* helper macro for setting appropriate bits in domain register */
</source>
The ''TLB_DOM'' macro allows you to set the domain value in a TLB entry, while ''DOM_TLB'' lets you set the
appropriate bits in the domain register. For example:
<source lang="c">
	arm4_tlbsetdom(DOM_TLB(TLB_DOM_CLIENT, 0));
</source>
The above code would set ''domain 0'' to use the permission bits specified in the TLB entry.
<source lang="c">
	arm4_tlbsetdom(DOM_TLB(TLB_DOM_CLIENT, 0) | DOM_TLB(TLB_DOM_MANAGER, 1));
</source>
The above code would set ''domain 0'' to use the permission bits specified in the TLB entry, and
''domain 1'' to overide the permission bits in the TLB entry and allow any CPU mode to access the
memory specified by the entries in the TLB that are marked as ''domain 1''.
==== Access Bits For TLB Entry ====
<source lang="c">
	uint32			*tlb = .....;
	
	tlb[0] = (1 << 20) | TLB_AP_NOACCESS | TLB_SECTION;
	tlb[1] = (1 << 20) | TLB_AP_PRIVACCESS | TLB_SECTION;
	tlb[2] = (1 << 20) | TLB_AP_UREADONLY | TLB_SECTION;
	tlb[3] = (1 << 20) | TLB_AP_FULLACCESS | TLB_SECTION;
</source>
Let me give a table of how this would effect memory access:
{| class="wikitable"
! Virtual Address Range
! Physical Address Range
! Access Description
|-
| 00000000 - 000FFFFF
| 00100000 - 001FFFFF
| Any access by privileged CPU mode or user mode will generate a fault.
|-
| 00100000 - 001FFFFF
| 00100000 - 001FFFFF
| Any access by user mode will generate a fault.
|-
| 00200000 - 002FFFFF
| 00100000 - 001FFFFF
| The user mode can read but not write.
|-
| 00300000 - 003FFFFF
| 00100000 - 001FFFFF
| Any mode can read and write.
|}

Now, I mapped each 1MB of virtual address section to the same physical address.

==== Coarse Table (Sub-Page) ====
In this example I create a coarse table with two 4KB entries, and I test that they
are mapped at the end. I have a few debug printing statements littered throughout
and decided to leave them in as an example.
<source lang="c">
	ktlb = (uint32*)k_heapBMAllocBound(&ks->hphy, 1024 * 16, 14);
	utlb = (uint32*)k_heapBMAllocBound(&ks->hphy, 1024 * 16, 14);

	ksprintf(buf, "ktlb:%x utlb:%x\n", ktlb, utlb);
	kserdbg_puts(buf);
	
	arm4_tlbsetmode(2);	
	/* 
		map 1GB of space - each entry is 1MB (we are setting N to 2 above) 
	*/
	for (x = 0; x < 1024; ++x) { // 0xc02
		ktlb[x] = (x << 20) | TLB_AP_PRIVACCESS | TLB_SECTION;
	}
	
	/* 
		do a little trick to map the same memory twice but in both
		page tables (TTBR0 and TTBR1)
	*/
	utlb[0x800] = (1 << 20) | TLB_AP_PRIVACCESS | TLB_SECTION;
	utlb[0x801] = (1 << 20) | TLB_AP_PRIVACCESS | TLB_SECTION;
	ktlb[1] = (2 << 20) | TLB_AP_PRIVACCESS | TLB_SECTION;
	ktlb[2] = (2 << 20) | TLB_AP_PRIVACCESS | TLB_SECTION;
	
	/* the coarse table must be aligned on a 1KB boundary */
	tlbsub = (uint32*)k_heapBMAllocBound(&ks->hphy, 1024, 10);
	ksprintf(buf, "tlbsub:%x\n", tlbsub);
	kserdbg_puts(buf);
	
	/* link the coarse table to our main table */
	ktlb[3] = (uint32)tlbsub | TLB_COARSE;
	
	/* clear coarse table (sub-table) */
	for (x = 0; x < 256; ++x) {
		tlbsub[x] = 0;
	}
	
	tlbsub[0] = TLB_C_SMALLPAGE | ((1024 * 1024) << 12) | TLB_C_AP_PRIVACCESS;
	tlbsub[1] = TLB_C_SMALLPAGE | ((1024 * 1024) << 12) | TLB_C_AP_PRIVACCESS;
	
	kserdbg_puts("OK\n");
	
	/* load location of TLB */
	arm4_tlbset1((uintptr)utlb);
	arm4_tlbset0((uintptr)ktlb);
	/* set that all domains are checked against the TLB entry access permissions */
	arm4_tlbsetdom(0x55555555);
	/* enable TLB 0x1 and disable subpages 0x800000 */
	arm4_tlbsetctrl(arm4_tlbgetctrl() | 0x800001);
	
	kserdbg_puts("OK2\n");
	
	/* check sub-pages */
	a = (uint32*)0x300000;
	b = (uint32*)0x301000;
	a[0] = 0x22334411;
	ksprintf(buf, "small:%x\n", b[0]);
	kserdbg_puts(buf);
	
	/* check trick - as you can see the same 1MB chunk is mapped twice */
	a = (uint32*)0x80000000;
	b = (uint32*)0x80100000;
	a[0] = 0x12345678;
	ksprintf(buf, "utlb:%x\n", b[0]);
	kserdbg_puts(buf);

	a = (uint32*)0x100000;
	b = (uint32*)0x200000;
	a[0] = 0x87654321;
	ksprintf(buf, "ktlb:%x\n", b[0]);
	kserdbg_puts(buf);

	kserdbg_puts("DONE\n");
	for(;;);
</source>

==== Using 64K Entries In Coarse Table ====
I have done some experimenting with 64K page entries in the coarse table. Essentially, you have to create
16-64K entries to map 64K. 

Take the normal way to map 64K of memory in a coarse table with 4K entries:
<source lang="c">
	uint32			*ctbl = ....;
	uint32			offset;
	uint32			x;
	
	for (x = 0; x < 16; ++x) {
		ctbl[x] = offset + (x * 4096) | FLAGS | SMALLPAGE4K;
	}
</source>

Let us see the mapping of 64K using 64K entries.
<source lang="c">
	uint32			*ctbl = ....;
	uint32			offset;
	uint32			x;

	for (x = 0; x < 16; ++x) {
		ctbl[x] = offset | FLAGS | LARGEPAGE64K;
	}
</source>
Now, QEMU will not enforce the entry to be replicated 16 times.

''It appears that real hardware may enforce the 16 count replication in order to reduce memory access and
space used by the TLB cache/memory. QEMU, does not enforce this and will happily work just fine with blank
entries in the 64K mapping or even 4K mappings mixed in. Also, it states in the manual that the entry must
be replicated 16 times. So the safe way is to always follow this rule and it should work with any MMU no
matter if it is software emulated or hardware.''

==== Globally Tagged Entries ====
''...shared''

==== Protection And Cache Behavior ====
''...''

=== Working With Tables Per Process ===
Let us now see about working with page tables per process. We need some utility functions. Then we will take a look at actually initializing the page tables per process, and finally the code in our
scheduler that will switch out tables for each process.

==== Utility Functions ====
To work with pages tables I have come up with a few utility functions to make life easier. These just make
working with a TLB table much easier. These functions only work with 4K coarse table mapping. One of the
functions can understand 1MB sections, but thats it. They are just basic utility functions to allow you to
have an idea of how you could work with page tables. 
<source lang="c">
#define KVMM_SUCCESS		1
#define KVMM_FAILURE		0
#define KVMM_UNMAP			0x80000000
#define KVMM_USER			0x40000000
#define KVMM_KERNEL			0x20000000

/*
	kernel kernel page alloc
	
	- only alloc from hphy
*/
void *kkpalloc() {
	KSTATE			*ks;
	int				x;
	
	ks = (KSTATE*)KSTATEADDR;
	return k_heapBMAlloc(&ks->hphy, KPHYPAGESIZE);
}

/*
	kernel user page alloc
	
	- first try to alloc from husr then hphy
*/
void *kupalloc() {
	KSTATE			*ks;
	int				x;
	void			*p;
	char			buf[128];
	
	ks = (KSTATE*)KSTATEADDR;
	p = k_heapBMAlloc(&ks->husr, KPHYPAGESIZE); 
	if (!p) {
		p = k_heapBMAlloc(&ks->hphy, KPHYPAGESIZE);
		ksprintf(buf, "p:%x\n", p);
		kserdbg_puts(buf);
	}
	
	return p;
}

/* Initialize the paging table structure. */
int kvmm_init(KVMMTABLE *table) {
	uint32		x;
	KSTATE		*ks;
	char		buf[128];
	
	ks = (KSTATE*)KSTATEADDR;	
	table->table = (uint32*)k_heapBMAllocBound(&ks->hphy, 1024 * 16, 14);
	
	kserdbg_putc('&');
	
	ksprintf(buf, "table.table:%x\n", table->table);
	kserdbg_puts(buf);
	
	for (x = 0; x < 1024 * 4; ++x) {
		table->table[x] = 0;
	}
}

/* Map contigious range starting at a virtual address and physical of count */
int kvmm_map(KVMMTABLE table, uintptr virtual, uintptr physical, uintptr count, uintptr flags) {
	uintptr		x, v, p, y;
	uint32		*t, *st;
	KSTATE		*ks;
	uint8		unmap;
	
	ks = (KSTATE*)KSTATEADDR;
	
	unmap = 0;
	if (flags & KVMM_UNMAP) {
		/* remove special flags */
		flags = flags & (~0 >> 1);
		unmap = 1;
	}
	
	/* this entire loop could be optimized, but it is simple and straight forward */
	v = virtual >> 12;
	p = physical >> 12;
	t = table.table;
	for (x = 0; x < count; ++x) {
		if ((t[v >> 8] & 3) == 0) {
			/* create table (alloc on 1KB boundary) */
			t[v >> 8] = (uint32)k_heapBMAllocBound(&ks->hphy, 1024 * 16, 10);
			if (!t[v >> 8]) {
				return 0;
			}
			/* get sub-table */
			st = (uint32*)t[v >> 8];
			t[v >> 8] |= TLB_COARSE;
			/* clear table (all entries throw a fault) */
			for (y = 0; y < 256; ++y) {
				st[y] = 0;
			}
		} else {
			/* get sub-table */
			st = (uint32*)(t[v >> 8] & ~0x3ff);
		}
		
		if ((st[v & 0xff] & 3) == 0) {
			/* map 4K page */
			if (!unmap) {
				/* map page */
				if (p + 1 == 0) {			
					st[v & 0xff] = (p << 12) | flags | TLB_C_SMALLPAGE;
				} else {
					st[v & 0xff] = (p << 12) | flags | TLB_C_SMALLPAGE;
				}
			} else {
				/* unmap page */
				st[v & 0xff] = 0;
			}
		}
		
		++v;
		++p;
	}
	
	return 1;
}

/* Find a region of size within specified boundries. */
int kvmm_getunusedregion(KVMMTABLE table, uintptr *virtual, uintptr count, uintptr lowerLimit, uintptr upperLimit) {
	uintptr		x, v, c, s;
	uint32		*t, *st;
	KSTATE		*ks;
	
	ks = (KSTATE*)KSTATEADDR;
	
	/* this entire loop could be optimized, but it is simple and straight forward */
	s = 0;	/* start */
	c = 0;  /* count */
	v = lowerLimit >> 12;
	t = table.table;
	for (x = 0; x < count; ++x) {
		if (t[v >> 8] & 3 == 0) {
			c++;
		} else {
			/* get sub-table */
			st = (uint32*)(t[v >> 8] & ~0x3ff);
			
			if (st[v & 0xff] & 3 == 0) {
				c++;
			} else {
				s = v << 12;
				c = 0;
			}
		}
		
		/* if we found a region of size */
		if (c >= count) {
			*virtual = s;
			return 1;
		}
		
		++v;
	}	
	
	*virtual = 0;
	return 0;
}

/*
	This does not support 64K or 16MB mappings, only 1MB and 4K.
*/
int kvmm_getphy(KVMMTABLE table, uintptr virtual, uintptr *out) {
	uint32			*t;
	char			buf[128];
	
	/* not mapped */
	if ((table.table[virtual >> 20] & 3) == 0) {
		*out = 0;
		return 0;
	}
	
	ksprintf(buf, "table.table[virtual >> 20]:%x\n", table.table[virtual >> 20]);
	kserdbg_puts(buf);
	
	/* get 1MB section address */
	if ((table.table[virtual >> 20] & 3) == TLB_SECTION) {
		*out = table.table[virtual >> 20] & ~0xFFFFF;
		return 1;
	}
	
	/* get level 2 table */
	t = (uint32*)(table.table[virtual >> 20] & ~0x3ff);
	
	virtual = (virtual >> 12) & 0xFF;

	ksprintf(buf, "t[%x]:%x\n", virtual, t[virtual]);
	kserdbg_puts(buf);
	
	/* not mapped on level 2 */
	if ((t[virtual] & 3) == 0) {
		*out = 0;
		return 0;
	}
	
	/* get 4K mapping */
	*out = t[virtual] & ~0xfff;
	return 1;
}

/* Map fresh pages from physical page accounting system.*/
int kvmm_allocregion(KVMMTABLE table, uintptr virtual, uintptr count, uintptr flags) {
	uintptr		x, v;
	uint32		*t, *st;
	KSTATE		*ks;
	uint8		kspace;
	uint32		y;

	/*
		If we are mapping in user space then lets try to use pages from the
		kusr map, and if for kernel then pull from kphy. 
	*/
	kspace = 1;
	if (flags & KVMM_USER) {
		kspace = 0;
	} else {
		if (!(flags & KVMM_KERNEL)) {
			/* force specification of either kernel space or user space mapping */
			return 0;
		}
	}
	/* remove flags */
	flags = flags & ~KVMM_KERNEL;
	flags = flags & ~KVMM_USER;
	
	ks = (KSTATE*)KSTATEADDR;
	/* this entire loop could be optimized, but it is simple and straight forward */
	v = virtual >> 12;
	t = table.table;
	for (x = 0; x < count; ++x) {
		if ((t[v >> 8] & 3) == 0) {
			/* create table (alloc on 1KB boundary) */
			t[v >> 8] = (uint32)k_heapBMAllocBound(&ks->hphy, 1024 * 16, 10);
			if (!t[v >> 8]) {
				/* memory failure */
				return 0;
			}
			st = (uint32*)t[v >> 8];
			
			t[v >> 8] |= TLB_COARSE;
			
			/* clear table (all entries throw a fault) */
			for (y = 0; y < 256; ++y) {
				st[y] = 0;
			}
		} else {
			/* get sub-table */
			st = (uint32*)(t[v >> 8] & ~0x3ff);
		}
		
		if ((st[v & 0xff] & 3) != 0) {
			/* return failure because we are mapping over already mapped memory */
			return 0;
		}
		
		/* map a page */
		if (kspace) {
			kserdbg_putc('1');
			st[v & 0xff] = (uintptr)kkpalloc() | flags | TLB_C_SMALLPAGE;
		} else {
			kserdbg_putc('2');
			st[v & 0xff] = (uintptr)kupalloc() | flags | TLB_C_SMALLPAGE;
		}
		/* increment to next virtual address */
		++v;
	}	
	return 1;	
}
/* Unmap pages and hand them back to the physical page accounting system. */
int kvmm_freeregion(KVMMTABLE table, uintptr virtual, uintptr count) {
	uintptr		x, v, p;
	uint32		*t, *st;
	KSTATE		*ks;
	
	ks = (KSTATE*)KSTATEADDR;
	
	/* this entire loop could be optimized, but it is simple and straight forward */
	v = virtual >> 12;
	t = table.table;
	for (x = 0; x < count; ++x, ++v) {
		if ((t[v >> 8] & 3) != 0) {
			/* get sub-table */
			st = (uint32*)(t[v >> 8] & ~0x3ff);
			
			/* unmap a page */
			if ((st[v & 0xff] & 3) != 0) {
				/* determine if it is a kernel or user page */
				p = (st[v & 0xff] >> 12 << 12);
				if (p > KMEMSIZE) {
					/* user */
					if (!k_heapBMFree(&ks->husr, (void*)p)) {
						return 0;
					}
				} else {
					/* kernel */
					if (!k_heapBMFree(&ks->hphy, (void*)p)) {
						return 0;
					}
				}
			}
		}
	}	
	
	return 1;
}
</source>

==== Initializing Page Tables Per Process ====
Our code, if you have been following from [[ARM_Integrator-CP_IRQTimerAndPICAndTaskSwitch|IRQ, Timer, PIC, And Tasks]], included 
a simple scheduler that swapped
state for each thread. What we want to do now is include a virtual address space for each thread. And, 
eventually upgrade the system to support the design of processes and threads. For
now just consider each thread a process, and later we will implement actual threads and processes. This
just helps keep the code simple at first which makes reading and understanding easier.

Our first step is to expand our thread structure with a field for the level one page table.
<source lang="c">
typedef struct _KVMMTABLE {
	uint32			*table;
} KVMMTABLE;
	
typedef struct _KTHREAD {
	uint8			valid;
	uint32			r0, r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12, sp, lr, cpsr, pc;
	KVMMTABLE		vmm;
} KTHREAD;
</source>

Next, lets add a field for the kernel level one page table.
<source lang="c">
typedef struct _KSTATE {
	KTHREAD			threads[0x10];
	uint8			threadndx;	
	uint8			iswitch;
	KHEAPBM			hphy;		/* kernel physical page heap */
	KHEAPBM			hchk;		/* data chunk heap */
	KHEAPBM			husr;		/* user physical page heap */
	KVMMTABLE		vmmk;		/* kernel virtual memory map */
} KSTATE;
</source>

Now, we need to initialize the kernel page table.
<source lang="c">
	/* initialize the kernel memory map */
	kvmm_init(&ks->vmmk);
	
	/* identity map kernel memory (may use all of RAM or not may) */
	for (x = 0; x < 4096; ++x) {
		// TLB_AP_FULLACCESS
		ks->vmmk.table[x] = (x << 20) | TLB_AP_PRIVACCESS | TLB_SECTION;
	}
</source>
This just identity maps all of RAM. That keeps everything simple at the moment. We can go back later and actually
make the kernel page in what it needs. But, that would complicate our existing code and I want to either
do that later or leave it as an excercise for you. Also, worth nothing, our utility functions will be unable to
work with the kernel table because it uses ''1MB sections'' and they do not handle those properly.

Also, we used ''KMEMINDEX'' to calculate ''KMEMSIZE'' which defines the maximum size of the kernel space. So, make sure
that we use it to divide the address space.
<source lang="c">
arm4_tlbsetmode(KMEMINDEX);
</source>

Here we initialize the address space of a thread, and we copy some executable code to it.
<source lang="c">
	ks->threads[0].pc = 0x80000000;         /* set program counter */
	ks->threads[0].valid = 1;
	ks->threads[0].cpsr = 0x60000000 | ARM4_MODE_USER;
	ks->threads[0].sp = 0x90001000;         /* set stack pointer */
	ks->threads[0].r0 = 0xa0000000;		/* argument to function we copied */
	
        /* initialize the level one table */
	kvmm_init(&ks->threads[0].vmm);
        /* map in the serial hardware MMIO */
	kvmm_map(ks->threads[0].vmm, 0xa0000000, 0x16000000, 1, TLB_C_AP_FULLACCESS);
        /* alloc and map a 4K page for the code */
	kvmm_allocregion(ks->threads[0].vmm, 0x80000000, 1, KVMM_USER | TLB_C_AP_FULLACCESS);
        /* alloc and map a 4K page for the stack */
	kvmm_allocregion(ks->threads[0].vmm, 0x90000000, 1, KVMM_USER | TLB_C_AP_FULLACCESS);
        /* get the physical page (going to be mapped in kernel space) */
	kvmm_getphy(ks->threads[0].vmm, 0x80000000, &page);
	/* copy some code there */
	for (x = 0; x < 1024; ++x) {
		((uint8*)page)[x] = ((uint8*)&thread1)[x];
	}
</source>
ARM passes the first few arguments by register so this enables us to pass an argument in ''R0''. We also
map in the serial hardware MMIO so the thread can send some letters to the output to demonstrate that it
is running. The argument is the address of the serial device's memory mapped output register for sending
a character. ''We are supposed to check if it is ready to recieve another character, but QEMU does not seem
to be effected so this should work fine on it. It is mainly just to demonstrate it working and not for
production usage.''

One ''VERY'' important part to understand is that we get the physical page (which we assume is mapped into kernel space), since I am assuming your running with less than 32MB of RAM. This is 
just to keep it simple. We can go back later and improve on this, but for now it will work enough
for you to move forward. If you have more memory than ''KMEMSIZE'' then this would not work because
it would allocate a page from the ''husr'' heap and that would be above ''KMEMSIZE'' meaning the
kernel will not have it identity mapped. I will cover additional ways to access the memory of a
thread/process later. For now just make QEMU have less than ''KMEMSIZE'' of RAM.

Here is the thread.
<source lang="c">
void thread1(uintptr serdbgout) {
	int			x;
	
	for (;;) {
		for (x = 0; x < 0xfffff; ++x);
		((uint32*)serdbgout)[0] = 72;
	}
}
</source>

Next, we need some code in our scheduler to do this before it returns (after loading the new thread's state).
<source lang="c">
            ......
			/* set TLB table for user space (it can be zero for kernel) */
			arm4_tlbset1((uintptr)kt->vmm.table);
			/* 
				Invalidate all unlocked entries...
				
				..according to the manual there may be a better way to invalidate,
				only some entries per process. But, for now this should work.
				
				If you do not do this then the TLB does not flush and old entries
				from the previous process will still be in the TLB cache.
			*/
			asm("mcr p15, #0, r0, c8, c7, #0");
			/* go back through normal interrupt return process */
			return;

</source>

The ''above'' happens after we have restored the registers on the exception stack and the two hidden registers, and is right
before we return from the IRQ generated by the timer hardware. We also invalidate the entire TLB. I assume this invalidates
both TTBR0 and TTBR1 tables.

=== Lets Page Out The Kernel Space ===
Ok, you now should have a minimaly functional kernel with user application isolation, memory management, and tasking. Now, you
can continue on with your current memory management system and go quite far, or you could tear it down and build something better
but more complicated. The previous design has some limitations mainly if a user applications exhaust the ''husr'' memory and start
grabbing memory from ''hphy''. A large app could essentially end up hogging all the memory the kernel can use and render the system
unusable even if you terminated other tasks. One method to handle this case is to take each page of memory used from ''hphy'' 
that has been allocated to user space and copy it (while the system is interrupted) into a page from ''husr'' then change the
mapping to point to the ''husr'' page. Essentially, swapping pages from ''kphy'' to ''husr''.

''Also having a unneeded memory mapped into kernel space poses a hazard from bugs. Mainly, your kernel could have a bug that makes stray writes and corrupts other processes causing really hard to find bugs, and by not mapping unneeded memory you make it more likely that you will catch bugs writing by them writing to invalid memory locations.''

If you want to explore different solution to managing virtual memory with out identity mapping
the kernel space then checkout the page [[User:Pancakes/ARM_Integrator-CP_VMM2|VMM2]] which builds from this page.
````

<a id="w003473"></a>

## W003473 — OSDev Wiki:Sandbox/Subpage

Bron-ID: W003473
Namespace: OSDev Wiki (4)
Revisie: 15809 | Datum: 2014-03-27T07:45:42Z
Bron: https://wiki.osdev.org/OSDev_Wiki:Sandbox/Subpage
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDev_Wiki%3ASandbox%2FSubpage&oldid=15809

````text
== Sub-page ==
This should be a Subpage to the Sandbox page.

Are there breadcrumbs at the top?

[[OSDev_Wiki:Sandbox/Subpage/Subsubpage|Sub-sub-page]]
````

<a id="w003474"></a>

## W003474 — OSDev Wiki:SubSubPage

Bron-ID: W003474
Namespace: OSDev Wiki (4)
Revisie: 15806 | Datum: 2014-03-27T07:41:49Z
Bron: https://wiki.osdev.org/OSDev_Wiki:SubSubPage
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDev_Wiki%3ASubSubPage&oldid=15806

````text
== Sub-sub-page ==
This should be a Sub-sub-page to the Sandbox sub-page.

Are there breadcrumbs at the top?
````

<a id="w003475"></a>

## W003475 — OSDev Wiki:Sandbox/Subpage/Subsubpage

Bron-ID: W003475
Namespace: OSDev Wiki (4)
Revisie: 15807 | Datum: 2014-03-27T07:42:49Z
Bron: https://wiki.osdev.org/OSDev_Wiki:Sandbox/Subpage/Subsubpage
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDev_Wiki%3ASandbox%2FSubpage%2FSubsubpage&oldid=15807

````text
== Sub-sub-page ==
This should be a Sub-sub-page to the Sandbox sub-page.

Are there breadcrumbs at the top?
````

<a id="w003476"></a>

## W003476 — User:Johnburger/Demo

Bron-ID: W003476
Namespace: User (2)
Revisie: 22422 | Datum: 2018-06-03T05:56:28Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo&oldid=22422

````text
= John Burger's Demo =

== Introduction ==
In 1995, I had been tinkering with the '386 for a few years, learning about different aspects of it and the PC that used it. Back then the World Wide Web had barely been invented, and there certainly weren't dedicated sites like this one for discussing arcane or even beginner questions on the operation of the PC.

== UseNet ==
But back then there was UseNet, a way of sending messages under different categories to other readers interested in that category. I laugh when I think about how it was considered "rude" to use UseNet to advertise products: that was considered "spam", and people were shunned when they tried. Alas...

Today Google has archives of those newsgroups, and you can see most of the old posts today. Some have been lost - one of the failings of UseNet - but new posts are being added all the time.

Two groups that I followed carefully were <code>comp.lang.asm.x86</code> and <code>alt.lang.asm</code> - the former more serious, while the latter more alternative (hence the designators). But then as now there were newbies that asked the same questions over and over, so I decided to do something about it.

== First example ==
I posted a five-part demo (https://groups.google.com/d/topic/comp.lang.asm.x86/RmpXt19VGHk/), in x86 assembly, showing one way to perform the following steps:
# Set up the GDT
# Set up the IDT
# Set up the interrupt and fault handlers
# Initialise the A20 gate
# Initialise the PICs to revector the IRQs
# Modify the timer to interrupt more often
# Switch to protected mode
# Set up some on-screen (text) tasks to execute
# Switch between the different tasks
# When the <Esc> key was pressed, return to real mode
# Restore the timer
# Restore the PICs
# Restore the A20 gate
# Exit the program

This demo was useful, since it allowed me to make changes to the code, try them out immediately, and when I pressed <Esc> it would return to DOS (gasp!). I could then reload the IDE and try the next thing. I didn't have to set up a boot floppy every time (USB stick? Back then? Nope!) and reboot every time to try it.

== Updated demo, for today's environment ==
Reading various pages on this Wiki and Forum, it's apparent that people are still interested in the steps I demonstrated above. The problem is that today, the development environment is a whole lot more sophisticated, and it's a <em>lot</em> harder to do the code/assemble/test/code merry-go-round that was so easy before.

Nevertheless, in the following pages I present a working demonstration of the steps necessary to get a '386 into protected mode, and what to do once you get there. It won't demonstrate some sophisticated features, like Paging, various device drivers, or modules like networking or file systems, but it will provide a testbed to explore various aspects of low-level OS development:
* What happens when a stack overflows?
* What does an Invalid TSS exception need to do?
* What is the effect of slowing down or speeding up the clock?
* How can I get information between different Tasks?

=== The Demonstrator ===
The 80386 (and later) is able to run in 32-bit mode in what is commonly called the "Flat" Memory Model, with none of that 'pesky' Segmentation. Where's the fun in that? Personally, I find the extra protection offered by [[Segment Limits#Segmentation|Segments]] a benefit while programming and debugging, and well worth the extra considerations.

I want to demonstrate many things with this code:
* Supervisor vs User modes. This requires at least one TSS;
* Segmentation. This requires more GDT entries for system tables such as LDTs and TSSs;
* Local Descriptor Tables. Allow each Task their own private segments;
* Task State Segments. Why not demo the CPU's native switching mechanism?

= [[User:Johnburger/Demo/Overview|John's Demo Overview]] =

[[Category:Kernel]]
````

<a id="w003477"></a>

## W003477 — User:Johnburger/Demo/Overview

Bron-ID: W003477
Namespace: User (2)
Revisie: 21351 | Datum: 2017-08-28T21:05:39Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Overview
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FOverview&oldid=21351

````text
The following is an overview of all the steps I took for this demonstration. Each section has a short description, and a link to a page covering the details of that section along with the assembly code to implement it.

To use this example, set up the Development Environment as detailed below (basically, install [http://www.nasm.us/ NASM]) and download the [https://github.com/JohnBurger/OSDev-Demo/archive/Wiki.zip GitHub ZIP file] containing all of the source. Assemble the source as described, and enjoy!

The ZIP file also has a fully-assembled (46 kiB) [https://github.com/JohnBurger/OSDev-Demo/raw/Wiki/Demo.iso ISO image] that you should be able to record straight to CD, or use with your favourite virtualisation program. Of course, this makes it difficult to experiment...

= Development Environment =

== Development Host ==
{{Main|User:Johnburger/Demo/Host}}
I wanted this example to be available to as broad an audience as possible, so I've made very few assumptions. The development machine's OS can be Windows (tested on WinXP 32-bit and Win7 64-bit) or Linux (tested on Ubuntu 12.04), and I presume Macintosh as well.

Of course, to assemble the source code requires an assembler. I don't particularly like NASM ([[User:Johnburger/NASMCritique|here's why]]), but it ''is'' portable and it ''does'' produce raw binary files, which is just perfect for what I need!

== Target Machine ==
{{Main|User:Johnburger/Demo/Target}}
The target machine needs to satisfy all of the following criteria:
* Is at least a '386 (tested on an Olivetti 386SX-16, and Asus Core i7);
* Has at least 640K of RAM (Bill Gates was right!);
* Obeys the BIOS Boot Specification 1.01 (i.e. not EFI/UEFI);
* Can boot off a Floppy, Hard Disk, USB, CD or ISO image (Yep! MultiBoot at its best!).

Note that I've tested this example with both VirtualPC and VMWare Player on Windows, and VirtualBox on Linux (Ubuntu 12.04), and it worked fine.

= Coding Style =

== Hierarchical Code ==
It must be my mindset: I think in hierarchies, and I've structured the source code that way too. Not only the code, but even the <em>identifiers</em>! NASM lets me use dots to categorise (and sub-categorise, and sub-sub-...) identifiers - it sometimes makes for some long names, but I <em>hope</em> it will make it clearer for readers too. (It certainly did for me!) Besides, it also let me do some interesting macro tricks.

The identifiers all start with a category: <code>x86</code> (definitions), <code>BIOS</code> (definitions), <code>Dev</code> (definitions), <code>Boot</code> (definitions and code), <code>Ints</code> (definitions and code), etc. These categories correspond to a top-level directory of the hierarchy. The second level is the subject within the category being defined: maybe a component of the x86; or a particular device; or a phase of the boot process. Of course, these correspond to sub-directories of the hierarchy.

Finally, you'll note that there is only one <code>.asm</code> file in the whole system. That is because it <code>%include</code>s all the other files, that themselves won't assemble standalone. They've all got the <code>.inc</code> extension to indicate this.

== Use the Assembler as Much as Possible ==
Any calculations that can be performed at assembly time are CPU cycles that don't have to be performed at run-time. To that end, I've defined constants, and expressions, and tables, and even <em>checksums</em> so that as much as possible only raw numbers need to be output to the binary image. One of the advantages of NASM is that it has powerful macro facilities - hey, being able to write a checksum algorithm in macros to make our job easier has to be a good thing!

== Define System Tables at Assembly Time ==
You'll notice that many tables have been built in source code, rather than at run time. Not all: some require the environment to be running to generate the correct values. To allow building tables in source code, many symbols and labels are defined to keep track of where the assembler is up to; and these symbols are often inter-related, and build upon each other such that changing one will change others.

To that end, I've also used macros to hide as many implementation details as possible at the time of invocation. Of course, you can look at the magic behind the scenes (<code>Demo.inc</code>, <code>x86/Desc.inc</code> and <code>Pad/ISO/Defn.inc</code> are the main files here), but it makes for concise one-liners at the point of invocation rather than convoluted NASM code to instantiate the various structures.

= The Source Code =
{{Main|User:Johnburger/Demo/Demo}}
The main code simply <code>%include</code>s all the other code, in a hierarchical fashion to make each aspect (hopefully) easily digestible. The different sections (and their potential sub-sections) are outlined below.

== Configurable settings ==
{{Main|User:Johnburger/Demo/Demo.inc}}
The modifiable parameters that the rest of the code uses are embodied in this file. By all means experiment - but don't blame me when weird things happen! If you need help understanding what happened, however, please feel free to drop me a message.

== Global Data ==
{{Main|User:Johnburger/Demo/Data.inc}}
Although this Demonstrator uses Local Descriptor Tables, some data still needs to be Global. For example, interrupt handlers that need to store data, such as the Keyboard handler, can only be guaranteed access to a Data Segment if it resides in the Global Descriptor Table.

= x86 Definitions =
{{Main|User:Johnburger/Demo/x86/x86}}

The designers of the '386 did more than just define opcodes to execute; they also defined a large number of system structures that define how the system will work in some pretty sophisticated scenarios. When coding for those, it pays to '''''NOT''''' use "magic numbers" - identifiers not only help comment the code, but make it less likely that you'll overlook the bug that has caused your masterpiece to triple-fault for the 1,847th time!

== EFlags ==
{{Main|User:Johnburger/Demo/x86/EFlags}}
It's not often that you need to access individual flags in the <code>EFlags</code> register: most of the time you can use <code>Jcc</code> to do the tests; or for the more sophisticated programmer, <code>SETcc</code>. But <code>EFlags</code> is 32 bits, and some of the flags are <em>very</em> useful - in particular, <code>TF</code> (the Trace Flag) calls the debugger after Every. Single. Instruction. Set this, and you can step through your code!

To that end, I've defined every flag in <code>EFlags</code> as a bit-mask, using the Intel-defined name.

== Segment ==
{{Main|User:Johnburger/Demo/x86/Seg}}
Once you're in Protected Mode, a Segment Register takes on a whole new personality - that deserve its own set of definitions.

== Control Register 0 (CR0) ==
{{Main|User:Johnburger/Demo/x86/CR0}}
Control Register 0 covers such things as whether the CPU is in Protected Mode, or in Paging Mode, or expecting Aligned accesses, or ... In short, you can change the entire architecture of your system by changing a couple of bits. Better define them, using Intel's defined names...

== Descriptors ==
{{Main|User:Johnburger/Demo/x86/Desc}}
Segment Descriptors reside in Descriptor Tables. There are three types: The Global Descriptor Table, the Local Descriptor Table, and the Interrupt Descriptor Table. Each have their functions and limitations, but all have eight-byte Descriptors in them with a common structure. That means that not only do they deserve definitions, but since they're going to be used a <em>lot</em> throughout the system, some macros would be useful too...

== Task State Segment (TSS) ==
{{Main|User:Johnburger/Demo/x86/TSS}}
The largest structure defined by Intel (actually, that's not true - the current state of the Floating Point Unit, with its <strong>huge</strong> registers, is bigger...) is the Task State Segment (TSS). It is used for hardware Task Switching, but it is also used for some other mechanisms of the CPU (e.g. switching privilege levels...) For a long time the Intel documentation actually stated that TSSes weren't required for systems that didn't use hardware switching - I disaverred them of that quickly enough...

== Interrupt Vector Table (IVT) ==
{{Main|User:Johnburger/Demo/x86/IVT}}
While the TSS is a single structure, the Interrupt Vector Table is merely a collection of 256 <code>FAR</code> pointers - larger, true, but merely a collection. In Real Mode, the IVT is <em>always</em> at 0000h:0000h, and <em>always</em> contains pointers to routines - although IBM and Microsoft subverted this '''''horribly''''' when they started using various interrupts to point to data tables... Can you imagine what would happen if code did an <code>INT 41h</code> or <code>INT 46h</code>? Executing data as code should send shudders through any programmer...

= BIOS Definitions =
{{Main|User:Johnburger/Demo/BIOS/BIOS}}
Whereas Intel defined the hardware silicon of the '386, IBM and other BIOS vendors added software extensions in the form of system calls, vectored through the <code>INT</code> instruction. Don't get me <em>started</em> on IBM not following Intel's mandate to reserve the first 32 interrupts for themselves - IBM's first interrupt (after Intel's last-defined one on the 8086) that they decided to use was to dump the current contents of the screen to a printer. Sounds like a debug function to me...

Regardless, the fact that the BIOS defined the operation of the PC gave it gravitas, and as the industry evolved various manufacturers published standards and specifications to make it easy for programmers to use their services. In short, their efforts required knowledge and information for programmers to use. In other words: definitions!

== A20 Gate ==
{{Main|User:Johnburger/Demo/BIOS/A20}}

== RAM Map ==
{{Main|User:Johnburger/Demo/BIOS/RAMMap}}

== Keyboard ==
{{Main|User:Johnburger/Demo/BIOS/Key}}

== Video Graphics Array (VGA) ==
{{Main|User:Johnburger/Demo/BIOS/VGA}}

== Disk ==
{{Main|User:Johnburger/Demo/BIOS/Disk}}

== Master Boot Record (MBR) ==
{{Main|User:Johnburger/Demo/BIOS/MBR}}

== Memory Map ==
{{Main|User:Johnburger/Demo/BIOS/MemMap}}

= Device Definitions =
{{Main|User:Johnburger/Demo/Dev/Dev}}
To write a device driver to control a peripheral requires detailed knowledge of the programming interface to that device. Some are straightforward. Others are highly complex. And there are different manufacturers out there that each use their own proprietary interface to get their devices to behave like others of the same type.

One of the best things that has happened since the development of the original IBM PC was the definition of the [[Universal Serial Bus]] (USB) standard. There was <em>one</em> programming interface that all USB controller manufacturers had to implement, which meant that there was only one USB driver that needed to be written to work with all controllers! Of course, then USB 1.1 came out, requiring changes. Then USB 2.0. Then USB 3.0...

Regardless, even before that there were some industry-standard interfaces that all manufacturers adhered to, almost by necessity: back in the DOS days installing device drivers to handle proprietary formats wasn't very user friendly, with <code>CONFIG.SYS</code> editing and arcane IRQ, I/O and memory parameters. Far better to use the default pre-prepared interfaces that were already coded in the BIOS.

The following files have definitions for the industry-standard implementations of the devices. Note that the code that uses these definitions might be in the [[User:Johnburger/Demo/Boot/Boot|Boot]] section or the [[User:Johnburger/Demo/Exec/Exec|Executive]] section, or even the [[User:Johnburger/Demo/Ints/Ints|Interrupt handlers]].

== A20 Gate ==
{{Main|User:Johnburger/Demo/Dev/A20}}

== Priority Interrupt Controller (PIC) ==
{{Main|User:Johnburger/Demo/Dev/PIC}}

== Periodic Interval Timer (PIT) ==
{{Main|User:Johnburger/Demo/Dev/Timer}}

== Keyboard ==
{{Main|User:Johnburger/Demo/Dev/Key}}

== Video Graphics Array (VGA) ==
{{Main|User:Johnburger/Demo/Dev/VGA}}

= Boot Sequence =
{{Main|User:Johnburger/Demo/Boot/Boot}}
When a PC first turns on, the familiar BIOS screen is shown while it initialises and tests various aspects of the system. It then determines which device to boot from, loads the boot sector (usually the first sector on the device, but for CDs it's the 17th - go figure, but I've used that to our advantage!), and then jumps to the loaded code.

Note that one of the first things that this boot code needs to do is load ''more'' code - the 512 or 2,048 bytes that the BIOS loads isn't very much! But before it can do anything, the code first needs to understand the starting point that it is working with.

== Real Mode Entry ==
{{Main|User:Johnburger/Demo/Boot/Entry}}
When the BIOS jumps to the loaded code, the CPU is in Real Mode. Given that the ultimate destination of this demonstration is Protected Mode, this boot code obviously needs to perform the switch. But once the switch is performed, the BIOS routines are no longer available. One way to handle this is to switch back whenever the BIOS is to be called. Another is to do the calls up front. The latter is what this Demonstrator does.

== Test CPU ==
{{Main|User:Johnburger/Demo/Boot/CPU}}
The first thing to do is confirm that the CPU is indeed a '386 or better. If not, there's no point in continuing!

== Load Rest of Code ==
{{Main|User:Johnburger/Demo/Boot/Load}}
The next thing is to load the rest of the code. From where, and to what address though? Luckily the BIOS Boot Specification helps us with the From question: it provides the device number, and the sector is the "next" one. Where in memory is arbitrary: just make sure you avoid anything important - including the code currently being executed! Check the [[User:Johnburger/Demo/Boot/Boot#Memory_Map|memory map]].

Of course, ideally all the Real Mode stuff will fit in the first 510 bytes. (Why not 512? We need the 0xAA55 BIOS Signature at the end.) If we can manage that, then the rest of the code can be loaded anywhere we like, and this code can be abandoned - tossed away like a booster rocket on a multi-stage orbital insertion.

== Position IDT ==
{{Main|User:Johnburger/Demo/Boot/IDT}}
Part of the design of this Demonstrator is to define some startup system tables at assemble time. One particular table is the default Interrupt Descriptor Table - it costs more in assembly instructions to initialize the IDT than it does to simply (effectively) hard-code it. The problem is that hard-coding it means that it's part of the binary image just loaded - but it's not where we want it. Solution? Move it!

Although you ''can'' set up the IDT once you've entered Protected Mode, I found that having it set up from the very first Protected Mode instruction made for better fault debugging. Of course it took a '''long''' time even then before I had the fault handlers themselves working - but as soon as I did, I could see where the problem instruction was and development proceeded ''much'' faster.

== Get RAM Size ==
{{Main|User:Johnburger/Demo/Boot/RAM}}
We should have asked this question before loading more code - if there's not enough RAM, there's no point in continuing. But '386s were always delivered with at least 1 MB of RAM, which is more than enough for our purposes!

But it is important to find out not only what RAM there is, but what RAM there isn't - amount of Extended Memory, where memory holes might exist, what memory the BIOS is using, and even what RAM was detected as bad during boot. To find all this out requires the help of the BIOS, so we need to save this information away for later use.

== Switch A20 Gate ==
{{Main|User:Johnburger/Demo/Boot/A20}}
As a legacy of the original IBM PC, this one is never going away! To protect against Real Mode Wrap, at boot time the A20 line is gated off. This needs to be gated on before Extended Memory can be used - and that requires the BIOS. There are a couple of other techniques tried if the BIOS doesn't support it.

== Set up Keyboard ==
{{Main|User:Johnburger/Demo/Boot/Key}}
The keyboard doesn't need much initializing - the BIOS initialization works well enough. However, any special setup can be done here.

== Switch Video Mode ==
{{Main|User:Johnburger/Demo/Boot/VGA}}
For the purposes of this Demonstrator, I didn't want to have to deal with the complexities of pixel-addressable graphics. The PC's video card has a number of different text modes, and that's sufficient for displaying stuff for the user in a quick and convenient manner.

By default I use the 80x50 text mode, but for experimentation I've also provided the code to switch to 90x60 mode. Not every PC or Virtualisation program supports this, so if you do enable it prepare to revert if it doesn't work! Not having it also saves a large number of bytes in the initial boot sector...

== Switch to Protected Mode ==
{{Main|User:Johnburger/Demo/Boot/PM}}
Switching to protected mode is three instructions - four if you include disabling interrupts first (a ''very'' smart move!) After all, all the previous code has set up all of the system structures...

You can then <code>JMP</code> to the Protected Mode code - and considering that the first thing that needs to happen is for the Segment Registers to be updated, we might as well start with the Code Segment Register (<code>CS</code>) first: make the first <code>JMP</code> far. And since we're using LDTs for this Demonstrator, why not <code>JMP</code> into one?

= Interrupt Handlers =
{{Main|User:Johnburger/Demo/Ints/Ints}}
While developing, any unexpected interrupt is a calamity! To that end, I decided to simply dump the current state onto the screen, overwriting whatever was there already. This required all sorts of labels and register-to-hex display routines, but the result is unambiguous: something horrible has happened, and ''this'' is where it happened.

What then? All we can really do is reboot the computer, but it'd be nice to wait until you can write down the critical areas. (Screen dump? Don't make me laugh!) &lt;Ctrl&gt;&lt;Alt&gt;&lt;Del&gt; is a time-honoured tradition - let's make it that!

Of course, it'd also be nice to find out exactly which ''flavour'' of interrupt actually occurred. It'd be nice to point all Fault vectors to the same handler, but a little extra sophistication allows us to display the interrupt number as well. To do that, we need separate entry points for each vector: but that's OK, there's a complicating factor anyway.

A half-dozen of the Intel-defined exceptions don't follow the standard rules. (Typical hardware engineers. Sheesh!) As well as doing the ''normal'' things of pushing onto the stack <code>EFLAGS</code>, then <code>CS</code>, then <code>EIP</code>, ''some'' interrupts also push an Error Code. And that Error Code can be useful. That means that either those "special" interrupts need special handling - or that we can modify all the ''other'' interrupts to pretend that they push Error Codes too...

I decided to do just that: provide separate entry points for each Interrupt handler, and for those that didn't have an Error Code, push a Zero onto the Stack as though it did. From there, all Interrupt handlers would push their number onto the stack, and call the default Fault handler.

== Interrupt Descriptor Table (IDT) ==
{{Main|User:Johnburger/Demo/Ints/IDT}}
It was less code to embed the initial entries of the starting IDT as data than to write code to reproduce it. In fact, given that I wanted the IDT to be available from the very first Protected Mode instruction, that IDT setup code had to be in Real Mode, where there was even less room! So I deliberately put the IDT at the beginning of the block loaded by the Real Mode loader for it to relocate to its final position.

== Vectors ==
{{Main|User:Johnburger/Demo/Ints/Vectors}}
These are the entry points for each of the Intel-reserved exception handlers.

== Default Fault Handler ==
{{Main|User:Johnburger/Demo/Ints/Fault}}
The default Fault handler basically smears the screen with a '''huge''' amount of information: the hex representation of every register I can get my hands on - ''with labels''. In a nice bright-white-on-red-background, in case anyone misses the point. Start copying...

Needless to say, as a default handler it is ''less'' than subtle. Especially since it doesn't try to correct the problem - however, after displaying the registers it returns to the faulting instruction. That should immediately re-raise the exception, [[User:Johnburger/Dingle|Dingling(tm)]] the interrupt location on the screen...

Note that some of the Fault sources are not simple interrupts: they're full-blown Task switches. I still wanted to display that information too, since not doing so is ''very'' misleading - it would indicate the current environment, not the faulting one. For those I need to extract the information, not from the Stack, but from the relevant TSS. Luckily the Backlink field of the current TSS holds the calling TSS, so I can use that to extract the relevant fields - just as soon as I work out which Segment Descriptor to load!

Incidentally: even though a fault has been displayed, you should still be able to invoke the [[User:Johnburger/Demo/Ints/Debug/Debug|debugger]]...

=== Single Step Handler ===
{{Main|User:Johnburger/Demo/Ints/Single}}
To experiment with debugging, I've hooked the Single Step Interrupt (INT 1) to make it simply wait in a tight loop until either <Space> or <Enter> is pressed:
* While waiting for a keypress, I decided to indicate that it was waiting by incrementing a screen location. That tight loop is very tight indeed...
* If <Space> is pressed, the routine simply returns. After the next instruction is executed, this same Handler will be re-entered, waiting for the next keypress. Single Step!
* If <Enter> is pressed, then it also returns: but before doing so, it turns off Single Stepping - for this Task at least!

To turn on Single Stepping in the first place, simply add the <code>TRACE</code> macro to the desired place. This sets <code>TF</code>, the Trace Flag, in the <code>[[User:Johnburger/Demo/x86/EFlags|EFLAGS]]</code> register.

One cute place to add <code>TRACE</code> is just at the start of <code>User</code>'s <code>DrawFrame</code> routine. That will mean that '''all''' the Tasks will be in Single Step mode, and you won't see anything until you hold down the Space Bar for long enough that the borders will start to be drawn. From there you can press <Enter> to continue individual Tasks until only a few "targets" remain.

=== Debug Handler ===
{{Main|User:Johnburger/Demo/Ints/Debug/Debug}}
I really wanted to see the memory that I was so carefully constructing. System tables, stack depths, and even the screen - I wanted to look at them. But I also wanted to be able to interact with it: in short, I wanted a memory viewer that I could invoke at run-time. There's a key on the keyboard labeled <Break>. Let's see what pressing it does...

==== Initialisation ====
{{Main|User:Johnburger/Demo/Ints/Debug/Init}}
First, I need to initialise the current run-time context. Prevent the code from switching to a different Task, set up some variables, then...

==== Show Registers ====
{{Main|User:Johnburger/Demo/Ints/Debug/Regs}}
Display the saved registers. Luckily, there's a table already set up to define where the registers are saved in memory during an interrupt - just need to use the funky screen location parameter...

==== Show Memory ====
{{Main|User:Johnburger/Demo/Ints/Debug/Show}}
Then we can display the current memory contents. We wouldn't want to raise an exception trying to access inaccessible memory, so pay careful attention to the current Segment's limits. Oh, and if the Segment happens to be an [[Expand Down]] segment, we need to totally invert the memory pointers for where we're looking at. In fact: let's take the opportunity to [[User:Johnburger/Demo/x86/Desc|highlight]] the fact that it is an [[Expand Down]] Segment by displaying it at the bottom of the screen!

==== Keyboard ====
{{Main|User:Johnburger/Demo/Ints/Debug/Key}}
Finally, a debugger without controls is not very useful. Let's look at the keypresses stored in the Global Data Segment and act on various direction arrows and context changing keys.

(Uh oh: what happens if we invoke the debugger while the debugger is running. Let's try it... Wow!)

=== Segment Not Present Handler ===
{{Main|User:Johnburger/Demo/Ints/NoSeg}}
Just to show what could happen in a full-blown Operating System, I decided to implement a quick-'n'-dirty Segment Not Present exception handler. Of course, it assumes that the segment referenced really ''is'' present, but that the Segment Present bit in the descriptor is merely off.

This handler discovers the offending segment in the appropriate Descriptor Table, enables the Present bit, and returns. The faulting instruction will be re-executed, and (hopefully!) things will proceed as normal!

== User System Calls ==
{{Main|User:Johnburger/Demo/Ints/User}}
IBM used the <code>INT</code> paradigm to invoke system calls on the first PC. Microsoft continued that practice with MS-DOS. As more sophisticated techniques came about - especially <code>SYSCALL</code> - using <code>INT</code> has become less common. Indeed, Intel suggests using Call Gates since they're more Protected - they can automatically copy parameters between stacks.

But for purposes of this Demonstrator, I've added my own two <code>INT</code> calls, to change the way that the individual Tasks behave. Experiment!

== Default IRQ Handler ==
{{Main|User:Johnburger/Demo/Ints/IRQs}}
An Interrupt ReQuest (IRQ) is different from an internal exception or fault. For one thing, there's an (impatient) device out there! If we're not careful, the device will continue to interrupt - if we simply returned, the unacknowledged interrupt could immediately re-interrupt the CPU, effectively locking it up forever.

The correct procedure on an externel interrupt is to "placate" the device to stop it from raising more interrupts for the same event, and then to acknowledge the interrupt on the PIC(s) that formed part of the process (in the order Slave then Master - this is important). If you then want to continue to process the interrupt, that's your lookout.

The problem is: what should the ''default'' IRQ handler do? It won't know how to "placate" every device out there - it can't. So I don't even try. I ''could'' acknowledge the PIC(s) and simply return, at the risk of entering an infinite interrupt loop - but at another level: '''''WHY is the PIC interrupting me for an unknown IRQ anyway?'''''

When the PIC is initialised, all interrupts are turned off. That means that it shouldn't be generating interrupts for unknown sources. Therefore, quite simply, the default IRQ handler will simply use the default Fault handler, and put an error code of <del>C0DE</del> (I decided to use 0 again - 0C0DEh used too many unnecessary bytes in the code!)

=== Timer IRQ Handler ===
{{Main|User:Johnburger/Demo/Ints/Timer}}
A Timer interrupt is quite expected: in fact, my default rate is 1,000 times a second. Why? I'm not doing much other than updating the screen, so you get smoother animations...

So what does the Timer interrupt handler do? Apart from housekeeping, it simply examines the GDT looking for TSSes that are not marked Busy, starting ''after'' the current one (which is, of course, Busy!). If it finds one, it checks to see if it's currently Active (that complication comes about because of the Intel suggestion to use TSSes to handle certain exceptions.) If it's Active, it <code>JMP</code>s to it. If it isn't, it keeps looking, wrapping around to the beginning of the GDT after the last known one. If it comes all the way back to the current TSS, it simply returns - it's the only one runnable!

The effect of a <code>JMP</code> to a TSS is that the CPU performs a Task Switch. It stores the current CPU state in the current TSS, marks it as not Busy, then updates TR and loads the new state from the new TSS. If the system has been running for a while, odds on the new TSS is pointing to the instruction ''after'' the Timer handler's <code>JMP</code> instruction, so the code will resume by returning from the Timer IRQ handler.

=== Keyboard IRQ Handler ===
{{Main|User:Johnburger/Demo/Ints/Key}}
A keyboard interrupt is also quite expected. The handler doesn't have to do much: simply get the received byte from the keyboard (which incidentally acknowledges the interrupt) and store it away somewhere. Where?
* With DOS, the answer is inside a 16-byte buffer in the BIOS Data Area (BDA) to be sampled by future calls to various "Get Next Key" BIOS calls. If the buffer fills up, the interrupt handler causes the Speaker to bleat - sorry, no Speaker code available in this Demonstrator (another exercise for the reader?)
* With Windows, the key press (or key release - they both cause interrupts, thankfully!) generates yet another Event for the Event Queue: say no more.

For this demo, I don't envisage a signficant delay between the key press and any Task waiting to handle it. After all, there ''are'' only two types of Tasks waiting for key presses: the SingleStep handler waiting for a &lt;Space&gt; or &lt;Enter&gt;; and any of the Fault handlers waiting for a &lt;Ctrl&gt;&lt;Alt&gt;&lt;Del&gt;. Therefore, I've decided to merely store the last keycode received.

It's that multiple key combination that deserves a description: when a key is pressed, the keyboard controller generates an interrupt to provide a unique(ish) code for the key. The top bit (0x80) of that code is zero. If that key is released, ''another'' interrupt is generated for the same key - but this time that top bit is set. Also note that if a key is held down, after a delay a succession of interrupts are generated, all with the same keycode, and all with their top bit zero.

Using that "make/break" keycode high-bit flag, it is possible to determine which keys are currently held down. As well as storing the last keycode found, the Key interrupt handler will also keep track of which of the "Shift" style of keys are currently pressed: &lt;Shift&gt;, &lt;Ctrl&gt;, or &lt;Alt&gt;. Note that since some keyboards have two copies of these keys (left and right, explaining the "(ish)" above - they generate the same keycodes, but with a prefix...), there will be multiple flags to represent the multiple keys.

== Utility Functions ==
{{Main|User:Johnburger/Demo/Ints/Util}}

Different interrupt handlers need to perform similar functions, to display Hex values or switch to the next Task. This module holds those common routines.

= User Task =
{{Main|User:Johnburger/Demo/User/User}}
Given that this Demonstrator will be running a number of User Tasks, and that it doesn't have a Task Loader (other than the one at [[User:Johnburger/Demo/Boot/Load|Boot]]), I need a Task to run.

I decided to make the User Task run in User Mode instead of Supervisor Mode. This demonstrates the extra work that needs to be done to accommodate the new mode, and also provides a platform from which to test various User Mode features: protection from Privileged instructions (such as <code>HLT</code>), as well as the complexities of making a <code>CALL</code> to the Kernel (an exercise for the reader!).

Note that it is easiest to make each User Task the same. This isn't of course necessary, but it provides a number of tasks without having to write a new one every time.

Each User Task is simple - it has been given an area of the screen to work with, and works out what to do with the area that it has been given:
* It draws a border in the allocated area - if one will fit;
* If the area is only 1x1, it simply continually increments the screen location. This has the effect of showing that the Task is actually running.
* Otherwise, it draws a "ball" using the defined character, and "bounces" it around in its allotted space. This may not cover every location in the defined area - a square area, for example, would only bounce the ball from the top left corner to the bottom right and back again.

== User LDT ==
{{Main|User:Johnburger/Demo/User/LDT}}
Each Task will have its own context: not only its own Task State Segment (TSS), but also its own Data, Stack, and Code. No, wait... since Code in Protected Mode is always Read-only, we can share the code between the Tasks.

Finally, they will each have their own Local Descriptor Table.

This actually makes the shared code easier - I can use constants for the different Segments, rather than organise new Segments for every new Task.

== User Data ==
{{Main|User:Johnburger/Demo/User/Data}}
The User code as given doesn't really need data: it can keep everything in its registers. However, as a Demonstrator it makes sense to show how it's done, so this is the Data segment - every Task will have its own copy.

== User Code ==
{{Main|User:Johnburger/Demo/User/Code}}
The User Task is given its starting parameters in registers, set up by the Executive when the TSS was created. It needs to use these parameters to work out how it is going to behave:
* Whether to draw a frame;
* Whether it is going to be bouncing or incrementing the ball.

== Draw Frame ==
{{Main|User:Johnburger/Demo/User/Frame}}
If the width is greater than 2, and the height is greater than 2, there's room for a border. This uses the high half of the ASCII character table to draw a border around the Task's allocated screen area. It then shrinks the allocated area to exclude the border from future calculations.

= Executive =
{{Main|User:Johnburger/Demo/Exec/Exec}}
Now that we're in Protected Mode, we have the full protection of the CPU for all of the code we write. Of course, that means that we need to be careful - but at least we get the benefit of the CPU for detecting code problems!

Before we can start the Executive proper, which creates the User Tasks, we really need to finish setting up Protected Mode: install better Fault Handlers; set up some devices with hardware interrupts; and so on.

Some of these routines will be calling utility functions, to allocate RAM and Descriptor Table entries. These routines are [[#Allocator_Functions|described below]].

== Update Registers ==
{{Main|User:Johnburger/Demo/Exec/Init}}
The Flags register and all of the Segment Registers need to be initialised now that we're in Protected Mode. <code>CS</code> was initialised during the <code>JMP</code> to here, so it's just the Stack and Data segment registers to do.

But, since we'll be using Supervisor and User modes, not to mention the CPU's Task-switching mechanism, it's also necessary to set up the Task Register - and a TSS to store stuff into.

== Install Fault Handlers ==
{{Main|User:Johnburger/Demo/Exec/Ints/Ints}}
Intel recommends that certain Faults should be handled by a full Task Switch, rather than merely an interrupt routine. For example, the following code while in Supervisor mode ''will'' halt the CPU:
<source lang="asm">
    MOV   ESP,1
    PUSH  EAX
</source>
This code will:
# Cause the CPU to underflow the stack, causing a Stack Fault.
# That will cause the CPU to push the current <code>EFLAGS</code>, <code>CS</code>, <code>EIP</code> and Error Code onto the stack... Oops! Invalid Stack! Now it's a Double Fault!
# That will cause the CPU to push the current <code>EFLAGS</code>, <code>CS</code>, <code>EIP</code> and Error Code onto the stack... Oops! Invalid Stack! Now it's a Triple Fault!
# That will cause the CPU to shut down.<br>The PC often has hardware to detect this condition, and will simply reboot the computer when it sees the CPU has shut down. Fun fun fun!

At least, the above scenario will halt the CPU unless the Stack Fault handler is a separate Task - or at least the Double Fault handler is one. That will switch in a known Stack at Fault time, avoiding the above scenario.

Therefore, we initialise a number of TSSes to take over the Intel-suggested Fault handlers in the IDT:
* Int 8 - Double Fault
* Int 10 - Invalid TSS
* Int 12 - Stack Fault

And we add some other interrupt handlers, for Debugging support and demonstration purposes.

=== Install Trace Handler ===
{{Main|User:Johnburger/Demo/Exec/Ints/Trace}}
The [[User:Johnburger/Demo/Ints/Single|Trace]] handler has already been written: it's just a matter of installing it.

=== Install Debug Handler ===
{{Main|User:Johnburger/Demo/Exec/Ints/Debug}}
The [[User:Johnburger/Demo/Ints/Debug/Debug|Debug]] handler has already been written: it's just a matter of installing it.

=== Install No Segment Handler ===
{{Main|User:Johnburger/Demo/Exec/Ints/NoSeg}}
The [[User:Johnburger/Demo/Ints/NoSeg|No Segment]] handler has already been written: it's just a matter of installing it.

=== Install Double Fault Handler ===
{{Main|User:Johnburger/Demo/Exec/Ints/Double}}
The generic [[User:Johnburger/Demo/Ints/Fault|Fault]] handler has been written, and it is currently pointed to by the Double Fault IDT entry, but that's not good enough. It really ought to have its own context - especially its own Stack - so this module will set one up for it, and fix the IDT entry.

Given that it's not the only Fault handler that needs this special treatment, it will use Executive's more sophisticated generic [[#Generic_Fault_Handler_Task|Fault]] handler, which will finally <code>CALL</code> the above generic handler to give the same appearance as before, but in a safer context.

=== Install Stack Fault Handler ===
{{Main|User:Johnburger/Demo/Exec/Ints/BadStack}}
The generic [[User:Johnburger/Demo/Ints/Fault|Fault]] handler has been written, and it is currently pointed to by the Stack Fault IDT entry, but that's not good enough. It really ought to have its own context - especially its own Stack - so this module will set one up for it, and fix the IDT entry.

Given that it's not the only Fault handler that needs this special treatment, it will use Executive's more sophisticated generic [[#Generic_Fault_Handler_Task|Fault]] handler, which will finally <code>CALL</code> the above generic handler to give the same appearance as before, but in a safer context.

=== Install Invalid TSS Handler ===
{{Main|User:Johnburger/Demo/Exec/Ints/BadTSS}}
The generic [[User:Johnburger/Demo/Ints/Fault|Fault]] handler has been written, and it is currently pointed to by the Invalid TSS Fault IDT entry, but that's not good enough. It really ought to have its own context - especially its own Stack - so this module will set one up for it, and fix the IDT entry.

Given that it's not the only Fault handler that needs this special treatment, it will use Executive's more sophisticated generic [[#Generic_Fault_Handler_Task|Fault]] handler, which will finally <code>CALL</code> the above generic handler to give the same appearance as before, but in a safer context.

=== Generic Fault Handler Task ===
{{Main|User:Johnburger/Demo/Exec/Ints/Fault}}
This is the more sophisticated generic Fault handler, that runs in its own Task context. That means that it needs to point to the registers in the faulting Task (using the [[User:Johnburger/Demo/x86/TSS|TSS Back link]] field) - which means even more complex code!

== Initialise PICs ==
{{Main|User:Johnburger/Demo/Exec/PICs}}
Another IBM PC legacy that won't go away! IBM decided to ignore Intel's reservation of the first 32 CPU interrupts for themselves, and mapped the hardware Interrupt ReQuest (IRQ) vectors for IRQs 0 to 7 to Interrupts 8 to 15.

When Intel used some of those interrupts in its 80286, every system from then on either had to deal with the overlaying interrupt causes, or had to reprogram the PICs to change their mapping.

I've decided to use Interrupts 32 to 47 for IRQs 0-7 and 8-15.

== Initialise Timer ==
{{Main|User:Johnburger/Demo/Exec/Timer}}
At boot time, the BIOS programs the Timer to interrupt as slow as possible, which ends up being 18.2 times a second. Given that the BIOS only uses this Timer to update the current date and time (with a resolution of only one second), and to turn off the Floppy Drive motor a little while after the last access (''not'' a time-critical operation!), this seems like a reasonable choice.

Of course, if a multi-tasking system wants to give the appearance of doing lots of things at once, 18.2 times a second is rather slow. So what's better? 60 times? 100 times? 1,000 times? More? Experiment! I chose 1,000 times, just for the giggle factor.

And of course the Timer Interrupt handler needs to be installed, which will switch between the available tasks.

== Initialise Keyboard ==
{{Main|User:Johnburger/Demo/Exec/Key}}
There's not a lot that you ''need'' to do for the keyboard. You could instruct it to use a different mapping, but the BIOS-defined default works well enough for our purposes.

We just need to set up a Keyboard Interrupt handler to save away keypresses as they occur.

== Create User Tasks ==
{{Main|User:Johnburger/Demo/Exec/User/User}}
Well, everything that we want initialised is now ready. There's nothing more for it but to let the system begin! And that's as simple as merely enabling interrupts (actually, that was done earlier...): the Timer handler will cycle through the list of active Tasks. All we need to do is create them!

=== Create User's LDT ===
{{Main|User:Johnburger/Demo/Exec/User/LDT}}
The User's LDT is defined in the [[User:Johnburger/Demo/User/LDT|User's code]], but that is so that it can access its symbols for the Selector offsets. It actually has to be populated by the Executive before use - which is this code!

=== Create User's TSS ===
{{Main|User:Johnburger/Demo/Exec/User/TSS}}
Similarly, the TSS has to be populated by the Executive before the User code can start - which is this code!

== Allocator Functions ==
{{Main|User:Johnburger/Demo/Exec/Alloc/Alloc}}
Some utility functions are needed by both the Initialisation code and the Main Task itself, to allocate generic system structures for tailoring by the rest of the code. These are provided here.

=== RAM Allocator ===
{{Main|User:Johnburger/Demo/Exec/Alloc/RAM}}
Keeping track of what RAM has been used and what is still available is important on any dynamic system. Since this demonstration will be allocating RAM but never returning it, it certainly makes the job easy! All we have to do is avoid allocating addresses that are already in use, or actually missing!

Of course, that's only the start of the problem. Once the RAM is allocated, it then needs to be assigned to a segment: without a segment descriptor, we can't access the RAM we've just allocated!

=== DT Allocator ===
{{Main|User:Johnburger/Demo/Exec/Alloc/DT}}
If RAM has been allocated, or a system table needs to be created, a descriptor for it needs to be added to a Descriptor Table. There are actually ''three'' descriptor tables that need updating: the GDT, the IDT, and the current LDT.

=== IDT Allocator ===
{{Main|User:Johnburger/Demo/Exec/Alloc/IDT}}

=== GDT Allocator ===
{{Main|User:Johnburger/Demo/Exec/Alloc/GDT}}

=== LDT Allocator ===
{{Main|User:Johnburger/Demo/Exec/Alloc/LDT}}

=== TSS Allocator ===
{{Main|User:Johnburger/Demo/Exec/Alloc/TSS}}

== Executive's LDT ==
{{Main|User:Johnburger/Demo/Exec/LDT}}
The Executive's LDT can be used for a number of things other than just holding the Segments needed to run it: it is also the perfect place to put the Segments required by the Executive's more sophisticated Fault handlers. Rather than cluttering the GDT with various non-global structures, I have defined this LDT as the LDT for the Fault handlers' TSSes.

== Global Descriptor Table ==
{{Main|User:Johnburger/Demo/Exec/GDT}}
And finally, last thing, (arguably) the most important structure in the entire system: the Global Descriptor Table.

I've coded it last to allow it to grow, without having to reposition it at run-time first. Where it is is good enough - it's (up to) 64 kiB in size, so a healthy chunk of memory has been reserved for it, even though only the first few entries are defined at startup.

= Padding =
{{Main|User:Johnburger/Demo/Pad/Sizes}}
The binary output from the assembler is exactly as large as it needs to be - which isn't likely to be a perfect size for any storage media. For this reason we need to Pad the binary output to round it up to the next Sector size. But here's also a perfect opportunity to Pad it with the structures and tables necessary to create an ISO image - without using a post-assemble tool!

Before we can do any Padding, though, we need a definitive size for each of the different Segments in the code. This module simply re-opens each Segment and defines a symbol for its <code>.Base</code>, <code>.Size</code> and <code>.Limit</code>.

== HardDisk ==
{{Main|User:Johnburger/Demo/Pad/HardDisk}}
Padding for the Hard Disk is not difficult, since the hard part was the Boot record - and that was handled in [[User:Johnburger/Demo/Boot/Boot|Demo/Boot/Boot]]. All we have to do here is Pad to the next Sector size.

== ISO ==
{{Main|User:Johnburger/Demo/Pad/ISO/ISO}}
[[ISO 9660]] is the detailed specification for the format recorded on a CD-ROM. Further, [[El-Torito]] is the specification for making a CD-ROM bootable. Due to a quirk in these specifications, it's possible to "Pad" a binary image and turn it into a bootable CD-ROM image! As long as the original binary obeys some restrictions, at least...

I have tried to make these files as stand-alone as possible, so that they can be easily added to your own project. The main article describes the required definitions.

=== Definitions ===
{{Main|User:Johnburger/Demo/Pad/ISO/Defn}}
[[ISO 9660]] defines a number of small and large structures, so I've defined most of them here, along with some helper macros to make my life easier. You will (probably) not need to modify any definitions here for your own use.

=== Primary Volume ===
{{Main|User:Johnburger/Demo/Pad/ISO/Primary}}
All CD-ROMs have a Primary Volume.

=== Boot Volume ===
{{Main|User:Johnburger/Demo/Pad/ISO/Boot}}
Only Boot CD-ROMs have a Boot Volume. This is defined by the [[El-Torito]] specification.

=== Terminator Volume ===
{{Main|User:Johnburger/Demo/Pad/ISO/Terminator}}
At the end of the list of volumes there needs to be a Terminator Volume.

=== Catalog Sector ===
{{Main|User:Johnburger/Demo/Pad/ISO/Catalog}}
The [[El-Torito]] specification requires a Catalog of Boot entries. For simplicity, I only define one.

=== Path Table(s) ===
{{Main|User:Johnburger/Demo/Pad/ISO/PathTable}}
They designed the CD File System in such a way that the same CD could be used by both little- and big-endian systems. Most of the time that is accomplished by recording the same multi-byte value twice, first in little-endian, then in big. They refer to this as "both-endian" format, and the reading software merely picks out the representation that it prefers.

But there's one structure, the Path Table, where they decided that it was too difficult to maintain, so they defined that the entire structure be recorded twice, once in each format. Since for this Demonstrator there is practically no difference between the two versions, I have written the source code for the structure generically, and used macros where the endian-ness is important.

Then it is simply a matter of:
* <code>%define</code>ing the macro as <code>Little</code>;
* <code>%include</code>ing this file once;
* Re<code>%define</code>ing the macro as <code>Big</code>;
* Re<code>%include</code>ing this file a second time.

I love macros!

=== Root Directory ===
{{Main|User:Johnburger/Demo/Pad/ISO/RootDir}}
This ISO image is a rarity, and could be mistaken for a coaster: it doesn't have any files on it! The important stuff is in the Boot sector and following loaded binary; there are no files ''per se'' required. Unfortunately, [[ISO 9660]] requires a root directory - even if it's empty.

== Floppy ==
{{Main|User:Johnburger/Demo/Pad/Floppy}}
Old versions of VMware wouldn't allow you to use a file image instead of a physical Floppy Disk, unless the file image precisely matched a standard Floppy size. I suspect the file size was used internally by VMware to work out what drive geometry to use (sectors per track etc.) So for years I added a '''''huge''''' number of zeroes onto the end of my experiments to make them 720 kiB, 1,200 kiB or 1,440 kiB in size.

One day, after a few VMware upgrades, I forgot - and it worked anyway. So I offer this code here in case it will be of any use. Personally, I doubt it...

= Experiments =
As well as adding <code>MOV ESP,1</code> somewhere, there are a number of other experiments you can do with this code: some for the giggle factor, while others to explore the functionality of the CPU.

== <code>HLT</code> vs <code>Yield</code> vs <code>LOOP</code> ==
In [[User:Johnburger/Demo/User/Code|Demo/User/Code]] there are four different ways of getting the User-mode code to wait between each update of the ball position:
<source lang="asm">
.Wait:
;                HLT                             ; Illegal User-mode instruction!

                  User.Halt                       ; So, use system-provided one

;                User.Yield                      ; This has a different effect

; This is an alternative to using HLT
;                PUSH            ECX
;                MOV             ECX,User.Delay  ; Wait for a little while
;                LOOP            $               ; About this long...
;                POP             ECX
</source>
You can uncomment each of them and see how it affects the behaviour of the individual Tasks - or comment them all out! I'm sorry I don't have a CPU usage display, but if you use a laptop and leave it running for a little while, pretty soon the CPU fan will become very noisy!

== <code>System.TSS</code> ==
I defined the <code>System.TSS</code> flag to both prevent switching ''to'' a System Task, and to prevent switching ''from'' one. I'm not suggesting that you do it this way in your system, but you will need something! Take a look at what happens if you don't put it in:
* In [[User:Johnburger/Demo/Ints/Debug/Init|Demo/Ints/Debug/Init]], comment out the <code>CALL Ints.System.Init</code> and <code>CALL Ints.System.Done</code> lines;
* Assemble and start the system;
* Invoke the Debugger by pressing <Break>;<br>Since the Debugged Task is no longer marked as special, the Task Switcher happily switches away from it and bounces balls through the display.
* Select a different Segment with the <Right> arrow key;
* Press <Break> again;<br>In all likelihood, you'll start Debugging a second Task, rather than Debugging the Debugger. You'll see the system try to display two different Segments, as well as bouncing balls.
* Press <Break> a few more times, and press <Right> and <Left> arrow a bunch of times too;<br>Now you'll see fewer bouncing balls, but more and different Segments.
* Keep pressing <Break> until you finally recurse one Task's Debugger deep enough to underflow its Stack.

Now the system ''is'' inside a <code>System.TSS</code>, courtesy of the Stack Fault (<code>Int 0Ch</code>) Task, and the Task Switcher stops the madness. You can now invoke the Debugger and examine the memory without being disturbed.

== Default bit in Code Descriptor ==
The assembler has been told that User-mode code is USE32, so it uses the 32-bit forms for all the instructions. What happens if we get this wrong? Let's find out!

In [[User:Johnburger/Demo/Exec/User/LDT|Demo/Exec/User/LDT]], there are some blocks of code to initialise the different User LDT entries. Go to the block that initialises the Code Segment:
<source lang="asm">
                MOV             EAX,User.Base           ; Assembled User code
                MOV             ECX,User.Size
                MOV             DL,Type.Mem(Code, DPL3, NoRW) & ~x86.Desc.Type.Present
                MOV             DH,Gran.Mem(Byte, Def32)
                CALL            Exec.Alloc.LDT.Mem      ; Allocate LDT Entry
</source>
and change <code>Def32</code> to <code>Def16</code>. This now tells the CPU to execute User code as 16-bit code - but we haven't told the assembler that!

Assemble and run the code. It should crash instantly with a GPF (<code>Int 0D</code>) in User Code (<code>CS=002F</code>), at (<code>EIP</code>) address <code>0000_0005</code>. Take a look at the first few lines of User Code in the assembly listing:
<source lang="asm">
                             <2>                 SEGMENT         User  VSTART=0  ALIGN=16
                             <2> 
                             <2> User.ColouredBall EQU           (User.BallColour << 8) | User.Ball
                             <2> 
                             <2> User.Entry:
00000000 FC                  <2>                 CLD                             ; Work forwards
00000001 891D[04000000]      <2>                 MOV             [User.Data.Row],EBX
00000007 668915[00000000]    <2>                 MOV             [User.Data.Left],DX  ; Left and Top
0000000E 66890D[02000000]    <2>                 MOV             [User.Data.Width],CX ; Width and Height
                             <2> 
</source>
There isn't an instruction at <code>0000_0005</code>! But if you look at the instruction at <code>0000_0001</code> you'll see the op-code <code>89 1D</code>. This means "Move the contents of <code>EBX</code> into the following memory location" - followed by a 32-bit address. Only, the CPU is actually in 16-bit mode, so it will only move <code>BX</code> and only expects a 16-bit address: then expects the next instruction to be at - you guessed it! - <code>0000_0005</code>. And what's there? The op-code <code>00 00</code>, which decodes as <code>ADD [BX+SI],AL</code> - which is ''way'' past <code>DS</code>'s Limit, and causes the GPF.
````

<a id="w003479"></a>

## W003479 — System Management Mode

Bron-ID: W003479
Namespace: Main (0)
Revisie: 30131 | Datum: 2026-03-04T22:15:31Z
Bron: https://wiki.osdev.org/System_Management_Mode
Revisie-URL: https://wiki.osdev.org/index.php?title=System_Management_Mode&oldid=30131

````text
[[System Management Mode]] (SMM) is an operating mode on x86 and x86-64 processors, intended for use by Firmware/BIOS to perform low-level system management operations while an OS is running.

== The role of SMM ==
During platform initialization the PC's firmware ([[BIOS]], [[UEFI]]) has complete control of the system and can perform whatever configuration operations are required to prepare the system for an OS to take over. Once an OS is running it expects that ''it'' has complete control of the system. If the firmware developer would like the firmware to continue to have some control over the system, SMM is used.

SMM is intended to be completely transparent to the OS. When the system enters SMM, the firmware preserves the state of the CPU in a region of RAM designated as "SMRAM". During SMM the firmware performs low-level management operations like changing fan speeds, checking thermal zones, adjusting the CPU speed, etc. Before leaving SMM, the firmware restores the state of the CPU from SMRAM. From the perspective of the OS, those low-level management operations are happening atomically and automatically in the background.

== Triggering SMM ==
SMM is triggered through a System Management Interrupt (SMI), a signal sent from the [[chipset]] to the CPU. During platform initialization, the firmware configures the chipset to cause a System Management Interrupt for various events that the firmware developer would like the firmware to be made aware of. The firmware also designates the region of RAM that should be used as SMRAM and specifies where the CPU should jump when an SMI occurs. During operation, the chipset detects the configured events and signals an SMI, triggering the CPU to enter SMM by jumping to the SMM entry point. The OS has no way of knowing when the chipset might signal an SMI and cannot "catch" System Management Interrupts like other interrupts. SMIs are not routed through the [[IVT]]/[[IDT]].

An OS ''can'' ask the chipset to signal an SMI, although the handling of the SMI will still be transparent. This is performed by writing to a particular [[Port IO|port]] (determined via [[ACPI]]). If the OS asks the chipset to signal an SMI and there is no reason for an SMI to be triggered at that particular moment, the firmware will not have much to do and the OS will regain control almost immediately.

== SMM and ACPI ==
SMM predates [[ACPI]]. Without ACPI, it makes sense that the firmware would want to stay resident in memory even once an OS is running, because the OS may not have drivers for all of the thermal sensors, fans, and CPU performance settings that are available on that particular motherboard/chipset/CPU combination. With ACPI, however, the firmware stores [[AML]] code for the OS to interpret and execute; that AML tells the OS how to handle all those things. Theoretically, this means that SMM is not nearly as critical on a modern system as on a pre-ACPI system. However, at boot time the firmware has no way of knowing whether or not the OS includes a complete ACPI implementation or whether the OS is willing to take responsibility to managing power, thermal properties, and CPU performance. So by default, the firmware will still use SMM to handle all those things itself.

If, however, at some point the OS indicates to firmware that it ''does'' have a complete ACPI implementation, the firmware will stop performing those low-level management operations and will expect the OS to perform them instead. Even once the OS takes over, the chipset will still occasionally signal SMIs and the CPU will enter SMM, but the firmware will not have very much to do.

The firmware developer decides on the division of responsibilities between firmware and the OS. The firmware could store AML code that tells the OS exactly how to handle every single thing that might happen with the system, leaving the firmware with nothing to do and no real reason for SMM to be used. On the other hand, the AML code could simply tell the OS to tell the chipset to signal an SMI whenever anything happens, at which point the firmware will directly handle the event itself.

A simple example of SMM and ACPI is the power button. Pressing the power button causes the chipset to signal an SMI, sending the CPU into SMM and jumping to the SMM entry point. The firmware determines that the power button was pressed. By default, before the OS takes over low-level system management responsibilities, the firmware will handle the event by telling the chipset to tell the power switching component of the motherboard to turn off, and the system powers off immediately. If, however, the OS has indicated that ''it'' will handle low-level system management responsibilities, the firmware ignores the button press and updates the ACPI structures in memory to note that the event occurred. The OS can detect the event by examining the ACPI structures and perform any clean-up operations (like saving files) before ''it'' tells the chipset to tell the power switching component of the motherboard to turn off.

== Transparency ==
TODO - write a bit about how SMM implements its transparent behavior, SMRAM, the lock bits in the MMU, etc.

== What can I do with system management mode? ==
SMM is designed so that OS developers do not need to be aware of it, other than understanding its relationship with ACPI. Since code executing in SMM cannot be detected by the OS under normal circumstances, SMM is an attractive vector for advanced malware. CPU implementations of SMM and BIOS implementations of SMM firmware that can be detected, abused, or interfered with are significant security issues. TODO - include some links to Invisible Things and some SMM hacking papers.

[[Category:X86]] [[Category:Operating Modes]]
[[de:System_Management_Mode]]
````

<a id="w003480"></a>

## W003480 — SMM

Bron-ID: W003480
Namespace: Main (0)
Revisie: 15827 | Datum: 2014-03-30T11:36:46Z
Bron: https://wiki.osdev.org/SMM
Revisie-URL: https://wiki.osdev.org/index.php?title=SMM&oldid=15827
Doorverwijzing: System Management Mode

````text
#REDIRECT [[System Management Mode]]
````

<a id="w003481"></a>

## W003481 — What do I need to know about SMM

Bron-ID: W003481
Namespace: Main (0)
Revisie: 15828 | Datum: 2014-03-30T11:37:33Z
Bron: https://wiki.osdev.org/What_do_I_need_to_know_about_SMM
Revisie-URL: https://wiki.osdev.org/index.php?title=What_do_I_need_to_know_about_SMM&oldid=15828
Doorverwijzing: System Management Mode

````text
#REDIRECT [[System Management Mode]]
[[Category:FAQ]]
````

<a id="w003482"></a>

## W003482 — User:Pancakes/ARM Integrator-CP VMM2

Bron-ID: W003482
Namespace: User (2)
Revisie: 16276 | Datum: 2014-04-24T14:16:15Z
Bron: https://wiki.osdev.org/User:Pancakes/ARM_Integrator-CP_VMM2
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APancakes%2FARM_Integrator-CP_VMM2&oldid=16276

````text
== MM2 ==
Well, hopefully, you are coming from some of the earlier examples. But, if not here are the links.

{| class="wikitable"
! Page
! Description
|-
| [[ARM_Integrator-CP_IRQTimerAndPIC|IRQ, Timer, And PIC]]
| This demonstration just uses the IRQ, Timer, And PIC.
|-
| [[ARM_Integrator-CP_IRQTimerAndPICAndTaskSwitch|IRQ, Timer, PIC, And Tasks]]
| This shows how to switch between tasks using the timer, and builds from the previous page.
|-
| [[User:Pancakes/ARM_Integrator-CP_IRQTimerPICTasksAndMM|IRQ, Timer, PIC, Tasks, And MM]]
| This shows how to integrate virtual memory, and builds from the previous page.
|}

=== Author ===
I [[User:Pancakes|Pancakes]] wrote this to help jump start you into developing for the ARM using QEMU or even a real piece of hardware. I have wrote software for both emulators and real hardware, but this has only been tested on QEMU so far. Please make any needed changes if you find problems. Also let me know at [mailto:kmcg3413@gmail.com kmcg3413@gmail.com] if you find this useful, have comments, or suggestions.

=== Intro ===

I am going to extend the page [[User:Pancakes/ARM_Integrator-CP_IRQTimerPICTasksAndMM|IRQ, Timer, PIC, Tasks, And MM]].
On that page we build a functional virtual memory management system. It had a limitation and that was that
the kernel space was identity mapped. This can create problems especially for MMIO access since if the
kernel space is less than the address of a MMIO device then you will have to map it into every process's
virtual memory just to access it which is much more difficult than just mapping it into kernel space. It
also lets us catch kernel bugs by having gaps so stray memory accesses might throw an exception, and it
helps to prevent kernel bugs from corrupting the user space and introducing very hard to track down bugs.

''So this design basically gives more protection and flexibility at a cost of complexity and slightly more
memory consumption.''

===== About Previous Design =====
Lets try something a bit more robust and to eliminate the need for 2 page heaps, and
instead have a single page heap that can service both kernel and user space. The problem is this design is a little more complex
and has a lot more moving parts, but I am going to try to gently lower you down into it in hopes you come out with a better memory 
management system. And as you know the more moving parts the more that can break!

===== Major Factor =====
The main problem with doing this is that once we go into paging mode we need memory to map more memory. A good example would
be imagine having your level one paging table with only one table entry, and that table having all 256 entries filled. You 
need to allocate and map at least 1k to make another table. Well, you will have no way to make that happen if your not using
1MB sections (maps 1MB with out second level table). And, the only other way would be to disable paging which could cause problems if you need to access data that is not identity mapped including your kernel's instruction stream. Also, in my code I am not going to use 1MB sections. So you would be unable to alloc and map another 4K to at least make a table with 1K.

''Disabling paging might not incur any performance penalty as I am not sure if it would flush the TLB, but if you need to access data that is not identity mapped then you are going to run into problems.''

''I decided not to use 1MB because it would complicate existing
code further, and I wanted this to be portable to the X86 by simply changing the shift operations and flags.''

==== Source ====
You will also need the source for the new VMM functions. Since the source is quite large I have uploaded it elsewhere. You can also find the complete kernel (''for this page'') here also: [http://kmcg3413.net/arm_icp_vmm2 http://kmcg3413.net/arm_icp_vmm2]

===== Updated Source =====
''If you inspect the above directory you will find the original functions. I have since made minor bug fixes and added more functionality. So I have left the above links for historical purposes, but the links below have the most recent source.''

These functions have been fairly well tested, and should be bug free. I have have very few problems. I hope these can be used as a guide on how you might want to construct your own VM functions for manipulating VM tables. If your looking for the complete source then consider cloning the repository and pulling out what you need. The links are [http://kmcg3413.net/vmm.c vmm.c] and [http://kmcg3413.net/vmm.h vmm.h].

=== Extracting Out Just The VMM ===
If you want to use the VMM in your own project you will need ''vmm.c'', ''vmm.h'', ''stdtypes.h'', ''main.h'', ''kheap.h'', and ''kheap_bm.c''. You major core files are just ''vmm.c'' and ''kheap_bm.c'' which contain all the implementation code.

The ''stdtypes.h'' is a small header that just defines the types. The ''kheap.h'' and ''heap_bm.c'' are also only need ''stdtypes.h''. The ''main.h'' contains the kernel state structure, which you
could modify and also modify in the ''vmm.c'' to work a different way if needed.

==== New Kernel State Structure ====
We need to add and remove a few fields from our kernel structure. One big change is the
removal of ''husr'', and the changing of the ''vmmk'' field to ''vmm''. I change it to ''vmm'' so any old code will break during compilation allowing you to remove it if you basing everything on
the previous page.

<source lang="c">
typedef struct _KSTATE {
	/* process/thread support */
	KTHREAD			threads[0x10];
	uint8			threadndx;	
	uint8			iswitch;
	
	/* physical and heap memory management */
	KHEAPBM			hphy;			/* kernel physical page heap */
	KHEAPBM			hchk;			/* data chunk heap */
	
	/* virtual memory management */
	KVMMTABLE		vmm;			/* kernel virtual memory map */
	uint32			vmm_ucte;		/* unused coarse table entries */
	KSTACK			tstack;			/* 1K table stack */
	uint32			*vmm_rev;		/* reverse map */
} KSTATE;
</source>

==== Overview Of Bootstrapping Before We Enable Paging ====
This should look familiar. It is basically the same process except we are using the functions
with ''kvmm2_'' prefix and whom work differently. The ''kvmm2_baseinit'' prepares a few critical
structures and can be found in ''vmm.c''. You should also notice we map the kernel code, exception table, and the entire ''hchk'' heap. The heap initialization code is basically the same as the previous page.
<source lang="c">
        ......

	arm4_xrqinstall(ARM4_XRQ_RESET, &k_exphandler_reset_entry);
	arm4_xrqinstall(ARM4_XRQ_UNDEF, &k_exphandler_undef_entry);
	arm4_xrqinstall(ARM4_XRQ_SWINT, &k_exphandler_swi_entry);
	arm4_xrqinstall(ARM4_XRQ_ABRTP, &k_exphandler_abrtp_entry);
	arm4_xrqinstall(ARM4_XRQ_ABRTD, &k_exphandler_abrtd_entry);
	arm4_xrqinstall(ARM4_XRQ_IRQ, &k_exphandler_irq_entry);
	arm4_xrqinstall(ARM4_XRQ_FIQ, &k_exphandler_fiq_entry);

	/* create physical page heap */
	k_heapBMInit(&ks->hphy);
	k_heapBMInit(&ks->hchk);
	/* get a bit of memory to start with for small chunk */
	k_heapBMAddBlock(&ks->hchk, 4 * 7, KRAMADDR - (4 * 7), KCHKHEAPBSIZE);
	
	/* state structure */
	k_heapBMSet(&ks->hchk, KSTATEADDR, sizeof(KSTATE), 5);
	/* stacks (can free KSTACKSTART later) */
	k_heapBMSet(&ks->hchk, KSTACKSTART - 0x1000, 0x1000, 6);
	k_heapBMSet(&ks->hchk, KSTACKEXC - 0x1000, 0x1000, 7);
        /* remove kernel image from hchk (if it is there) */
	k_heapBMSet(&ks->hchk, (uintptr)&_BOI, (uintptr)&_EOI - (uintptr)&_BOI, 8);
	
	/* add block but place header in chunk heap to keep alignment */
	bm = (uint8*)k_heapBMAlloc(&ks->hchk, k_heapBMGetBMSize(KRAMSIZE - KRAMADDR, KPHYPAGESIZE));
	k_heapBMAddBlockEx(&ks->hphy, KRAMADDR, KRAMSIZE - KRAMADDR, KPHYPAGESIZE, (KHEAPBLOCKBM*)k_heapBMAlloc(&ks->hchk, sizeof(KHEAPBLOCKBM)), bm, 0);
	
        /* remove kernel image from hphy (if it is there) */
	k_heapBMSet(&ks->hphy, (uintptr)&_BOI, (uintptr)&_EOI - (uintptr)&_BOI, 8);

	/* vmm */
	kvmm2_baseinit(KRAMADDR);
	
	/* map kernel image */
	kvmm2_mapmulti(&ks->vmm, 
					(uintptr)&_BOI, (uintptr)&_BOI,
					kvmm2_rndup((uintptr)&_EOI - (uintptr)&_BOI), 
					TLB_C_AP_PRIVACCESS | KVMM_DIRECT
	);
	/* map reverse table (ALREADY MAPPED WITH HCHK BELOW) */
	/* map interrupt table, and chunk heap (hchk) */
	kvmm2_mapmulti(&ks->vmm, 0, 0, kvmm2_rndup(KRAMADDR), TLB_C_AP_PRIVACCESS | KVMM_DIRECT | KVMM_SKIP);

	/* map serial out register, PIC, and timer */
	kvmm2_mapsingle(&ks->vmm, 0x16000000, 0x16000000, TLB_C_AP_PRIVACCESS | KVMM_DIRECT);
	kvmm2_mapsingle(&ks->vmm, 0x14000000, 0x14000000, TLB_C_AP_PRIVACCESS | KVMM_DIRECT);
	kvmm2_mapsingle(&ks->vmm, 0x13000000, 0x13000000, TLB_C_AP_PRIVACCESS | KVMM_DIRECT);

	arm4_tlbsetmode(KMEMINDEX);
	/* load location of TLB */
	arm4_tlbset1((uintptr)ks->vmm.table);	/* user space */
	arm4_tlbset0((uintptr)ks->vmm.table);	/* kernel space */
	/* set that all domains are checked against the TLB entry access permissions */
	arm4_tlbsetdom(0x55555555);
	/* enable TLB 0x1 and disable subpages 0x800000 */
	arm4_tlbsetctrl(arm4_tlbgetctrl() | 0x1 | (1 << 23));

        ......
</source>
At the end we enable paging, and if all is well execution should resume. That in my opinion is not a heavily complex or long section of code if you consider you now have a paged kernel. Also, you should note the usage of ''KVMM_DIRECT''. This tells the implementation that paging is not yet enabled and all references can be followed directly. This happens when walking the tables to perform the various operations. Once paging has been enabled you simply omit that flag from the calls and they will perform the translation using what I call the ''reverse lookup table''. 

''The reverse lookup table just translates physical addresses to virtual ones.''

==== Creating A Thread And Mapping It ====
The next part is mapping an address space for a user level thread.
<source lang="c">
	ks->threads[0].pc = 0x80000000;
	ks->threads[0].valid = 1;
	ks->threads[0].cpsr = 0x60000000 | ARM4_MODE_USER;
	ks->threads[0].sp = 0x90001000;
	ks->threads[0].r0 = 0xa0000000;
	
	kvmm2_init(&ks->threads[0].vmm);
	kvmm2_mapsingle(&ks->threads[0].vmm, 0xa0000000, 0x16000000, TLB_C_AP_FULLACCESS);
	kvmm2_allocregionat(&ks->threads[0].vmm, 1, 0x80000000, TLB_C_AP_FULLACCESS);
	kvmm2_allocregionat(&ks->threads[0].vmm, 1, 0x90000000, TLB_C_AP_FULLACCESS);
	kvmm2_getphy(&ks->vmm, (uintptr)ks->threads[0].vmm.table, &page);
	arm4_tlbset1(page);
	asm("mcr p15, #0, r0, c8, c7, #0");
	/* copy some code there */
	for (x = 0; x < 1024; ++x) {
		((uint8*)0x80000000)[x] = ((uint8*)&thread1)[x];
	}
</source>

I try to flush the TLBs, but QEMU seems to work fine with out it, but I left it in just in case. Since GCC by default emits position independent code (-fPIC) for ARM due to the nature of ARM instructions so therefore I am able to just plainly copy the ''thread1'' function. However, any references to segments other than ''.text'' by it may not be. But, this is just a demonstration. By using this method you could load an ELF image into the address space for example.

Because, GRUB may not be possible to use I would like to introduce you to a method to not only attach modules to your kernel but enumerate them in the next page, [[ARM_Integrator-CP_IRQTimerPICTasksMMAndMods|IRQ, Timer, PIC, Tasks, MM, And Modules]].
````

<a id="w003483"></a>

## W003483 — ARM Integrator-CP IRQTimerPICTasksMMAndMods

Bron-ID: W003483
Namespace: Main (0)
Revisie: 28001 | Datum: 2023-07-09T17:45:27Z
Bron: https://wiki.osdev.org/ARM_Integrator-CP_IRQTimerPICTasksMMAndMods
Revisie-URL: https://wiki.osdev.org/index.php?title=ARM_Integrator-CP_IRQTimerPICTasksMMAndMods&oldid=28001

````text
{{FirstPerson}}
== ARM Integrator-CP IRQ, Timer, PIC, Tasks, Modules, And ELF32 ==
{| class="wikitable"
! Page
! Description
|-
| [[ARM_Integrator-CP_IRQTimerAndPIC|IRQ, Timer, And PIC]]
| This demonstration just uses the IRQ, Timer, And PIC.
|-
| [[ARM_Integrator-CP_IRQTimerAndPICAndTaskSwitch|IRQ, Timer, PIC, And Tasks]]
| This shows how to switch between tasks using the timer, and builds from the previous page.
|-
| [[User:Pancakes/ARM_Integrator-CP_IRQTimerPICTasksAndMM|IRQ, Timer, PIC, Tasks, And MM]]
| This shows how to integrate virtual memory, and builds from the previous page.
|-
| [[User:Pancakes/ARM_Integrator-CP_VMM2|Virtual Memory With Paged Kernel]]
| This shows a method to have kernel space paged.
|}

=== Author ===
I [[User:Pancakes|Pancakes]] wrote this to help jump start you into developing for the ARM using QEMU or even a real piece of hardware. I have wrote software for both emulators and real hardware, but this has only been tested on QEMU so far. Please make any needed changes if you find problems. Also let me know at [mailto:kmcg3413@gmail.com kmcg3413@gmail.com] if you find this useful, have comments, or suggestions.


=== Intro ===

You should be coming from one of the previous pages. I prefer the last one if you have been
following along. Well, the moment has arrived. You are now ready to see a method of attaching
some modules and even a demonstration of loading a ELF32 module into user space. 

First we will cover the method to attach files to the end of the kernel image. Then we will look
at loading an ELF32 module into user space.

=== Attaching A Module ===
==== Python Script ====
This scripts pads the module to a 32-bit boundary so reads will be aligned, and inserts the
special header so it can be located. The header contains a few fields, see C code further down to see a structure defined for the header. The module can present it's own fields in the header through the ''slots'' field.
<source lang="python">
#!/usr/bin/python3
import sys
import struct

def attach(srcf, dstf, typf):
	sfd = open(srcf, "rb")
	sfd.seek(0, 2)
	sz = sfd.tell()
	sfd.seek(0, 0)
	sd = sfd.read(sz)
	sfd.close()

	pad = len(sd) & 3
	if pad > 0:
		pad = 4 - pad
	else:
		pad = 0

	dfd = open(dstf, "r+b")
	dfd.seek(0, 2)
	psz = dfd.tell()
	ppad = psz & 3
	if ppad > 0:
		ppad = 4 - ppad
	else:
		ppad = 0
	dfd.write(b'B' * ppad)
	dfd.write(struct.pack('IIII', sz + pad, 0x12345678, 0xedcba987, int(typf)))
	dfd.write(sd)
	#print('sz+pad=%s' % (sz + pad))
	if pad > 0:
		dfd.write(b'A' * pad)
	#print('[ATTACHMOD] spad:%s tpad:%s %s [%s bytes] ----> %s (old sz:%s new sz:%s)' % (pad, ppad, srcf, sz, dstf,  hex(psz), hex(dfd.tell())))
	dfd.close()
	return sz

srcf = sys.argv[1]
dstf = sys.argv[2]
typf = sys.argv[3]

attach(srcf, dstf, typf)
</source>
To attach a module you could do something like this:
<pre>
    ./attachmod.py mymodule mykernelimage 1
</pre>

Where 1 is a 32-bit integer value. I use the value one in the kernel code further down to
determine if it is an ELF32 module and if so I load it into a newly created user address space.

==== Module Enumeration Code ====
<source lang="c">
#ifndef KMOD_H
#define KMOD_H
#include "stdtypes.h"

typedef struct _KATTMOD {
	uint32			size;
	uint32			signatureA;
	uint32			signatureB;
	uint32			type;
	uint32			slot[];
} KATTMOD;

KATTMOD *kPkgGetNextMod(KATTMOD *mod);
KATTMOD *kPkgGetFirstMod();
uintptr kPkgGetTotalLength();
#endif

#include "kmod.h"

extern uint8 _EOI;

/*
	Get the next module after the specified module.
*/
KATTMOD *kPkgGetNextMod(KATTMOD *mod) {
	KATTMOD		*n;
	
	n = (KATTMOD*)((uintptr)mod + mod->size + sizeof(KATTMOD));
	
	/* make sure there actually exists another module after this one */
	if (n->signatureA != 0x12345678 || ~n->signatureA != n->signatureB) {
		kprintf("next mod not found\n");
		return 0;
	}
	
	return n;
}

uintptr kPkgGetTotalLength() {
	KATTMOD			*m, *lm;
	
	lm = 0;
	for (m = kPkgGetFirstMod(); m; m = kPkgGetNextMod(m)) {
		lm = m;
	}
	
	if (!lm) {
		return (uintptr)&_EOI;
	}
	
	return lm->size + (uintptr)lm + sizeof(KATTMOD) + 1;
}

/*
	Locate first module, if any exists, at end of our image.
*/
KATTMOD *kPkgGetFirstMod() {
	uint32		*p;
	uint32		x;
	uintptr		_p;
	
	_p = (uintptr)&_EOI - 4 * 20;
        /* OR... _p = (uintptr)&kPkgGetFirstMod - 4 * 20.. if not _EOI symbol 
      
          With GCC and -Os the code below might emit the signature. So you might
          need to create a dummy function below this one (should come after this
          one in text section), and reference that instead. With _EOI symbol you
          should have no problems unless this function appears with in the range
          and your using GCC -Os (optimize for size)
        */
	_p = _p & ~0x3;
	
	/* find signature of first module */
	for (p = (uint32*)_p, x = 0; x < 256; ++x) {
		/*
			This is a very tricky situation. We do not want the compiler
			to emit these sequence of bytes neither through an immediate
			operand or through optimization.
		*/
		if (p[x] == 0x12345678 && p[x + 1] == ~p[x]) {
			/* backup 32-bits (4 bytes) so we grab the size placed by attachmod.py */
			return (KATTMOD*)&p[x - 1];
		}
	}
	
	return 0;
}
</source>

To enumerate modules attached to the image:
<source lang="c">
        KATTMOD   *m;

	for (m = kPkgGetFirstMod(); m; m = kPkgGetNextMod(m)) {
        }
</source>

=== Loading An Elf Module ===
The code below is the absolute minimal needed to load an ELF32 into memory. It skips a lot of
checks and other advanced features.

==== ELF Binary Code ====
<source lang="c">
int _start(unsigned int *smmio) {
	int		x;

	for (;;) {
		for (x = 0; x < 0xfffff; ++x);
		smmio[0] = 'G';
		smmio[0] = 'K';
	}
	return 0;
}
</source>

To compile:

<pre>
arm-eabi-gcc -nostdlib -nostartfiles -ffreestanding -std=gnu99 main.c -o main 
arm-eabi-ld main.o -o main -Ttext 0x80000000 -N
</pre>


''For the -N flag. LD produces large files where the file offset of sections are equal to the virtual address. See https://sourceware.org/ml/binutils/2009-04/msg00099.html''

==== Kernel Code ====
The first part of the kernel code here assumes you have been following along with the other pages
in this series of pages. So I am not going to go too deep to explain much of it. But, essentially I check that the module is of type ''KMODTYPE_ELFUSER'' and if so then I find a free task slot and
hand that to a function that will do the actual loading of the image.

<source lang="c">
	#define KMODTYPE_ELFUSER			1
	/*
		create a task for any attached modules of the correct type
	*/
	kprintf("looking at attached modules\n");
	for (m = kPkgGetFirstMod(); m; m = kPkgGetNextMod(m)) {
		kprintf("looking at module\n");
		if (m->type == KMODTYPE_ELFUSER) {
			/* find free task structure */
			for (x = 0; x < 0x10; ++x) {
				if (!ks->threads[x].valid) {
					break;
				}
			}
			
			if (x >= 0x10) {
				PANIC("out-of-task-slots");
			}
			
			kelfload(&ks->threads[x], (uintptr)&m->slot[0], m->size);
		}
	}
</source>

And, of course the ''kelfload'' implementation. This is just a very minimal implementation intended to just get the image into memory as a demonstration and nothing advanced.

<source lang="c">
typedef uint16 Elf32_Half;
typedef uint32 Elf32_Word;
typedef uint32 Elf32_Off;
typedef uint32 Elf32_Addr;

#define EI_NIDENT 16
typedef struct {
        unsigned char   e_ident[EI_NIDENT];
        Elf32_Half      e_type;
        Elf32_Half      e_machine;
        Elf32_Word      e_version;
        Elf32_Addr      e_entry;
        Elf32_Off       e_phoff;
        Elf32_Off       e_shoff;
        Elf32_Word      e_flags;
        Elf32_Half      e_ehsize;
        Elf32_Half      e_phentsize;
        Elf32_Half      e_phnum;
        Elf32_Half      e_shentsize;
        Elf32_Half      e_shnum;
        Elf32_Half      e_shtrndx;
} ELF32_EHDR;

#define EM_ARM			40

typedef struct {
       Elf32_Word      sh_name;
       Elf32_Word      sh_type;
       Elf32_Word      sh_flags;
       Elf32_Addr      sh_addr;
       Elf32_Off       sh_offset;
       Elf32_Word      sh_size;
       Elf32_Word      sh_link;
       Elf32_Word      sh_info;
       Elf32_Word      sh_addralign;
       Elf32_Word      sh_entsize;
} ELF32_SHDR;

int kelfload(KTHREAD *th, uintptr addr, uintptr sz) {
	ELF32_EHDR			*ehdr;
	ELF32_SHDR			*shdr;
	uint32				x, y;
	uintptr				page, oldpage;
	KSTATE				*ks;
	uint8				*fb;
	
	kprintf("loading elf into memory space\n");
	
	ks = (KSTATE*)KSTATEADDR;
	
	ehdr = (ELF32_EHDR*)addr;
	if (ehdr->e_machine != EM_ARM) {
		kprintf("kelfload: not ARM machine!\n");
		return 0;
	}
	
	if (ehdr->e_ident[4] != 0x1) {
		kprintf("kelfload: not ELF32 object\n");
		return 0;
	}
	
	kvmm2_init(&th->vmm);
	th->pc = ehdr->e_entry;
	th->valid = 1;
	th->cpsr = 0x60000000 | ARM4_MODE_USER;
	/* set stack */
	th->sp = 0x90001000;
	/* pass address of serial output as first argument */
	th->r0 = 0xa0000000;
	/* map serial output mmio */
	kvmm2_mapsingle(&th->vmm, 0xa0000000, 0x16000000, TLB_C_AP_FULLACCESS);
	/* map stack page (4K) */
	kvmm2_allocregionat(&th->vmm, 1, 0x90000000, TLB_C_AP_FULLACCESS);

	/* map address space so we can work directly with it */
	kvmm2_getphy(&ks->vmm, (uintptr)th->vmm.table, &page);
	oldpage = arm4_tlbget1();
	arm4_tlbset1(page);
	/* flush TLB */
        asm("mcr p15, #0, r0, c8, c7, #0");

	// e_shoff - section table offset
	// e_shentsize - size of each section entry
	// e_shnum - count of entries in table
	for (x = 0; x < ehdr->e_shnum; ++x) {
		shdr = (ELF32_SHDR*)(addr + ehdr->e_shoff + x * ehdr->e_shentsize);
		if (shdr->sh_addr != 0) {
			/* load this into memory */
			// sh_offset - byte offset in module
			// sh_size - size of section in module 
			// sh_addr - address to load at
			kvmm2_allocregionat(&th->vmm, kvmm2_rndup(shdr->sh_size), shdr->sh_addr, TLB_C_AP_FULLACCESS);
			fb = (uint8*)(addr + shdr->sh_offset);
			/* copy */
			for (y = 0; y < shdr->sh_size; ++y) {
				((uint8*)shdr->sh_addr)[y] = fb[y];
			}
		}
	}
	
	/* restore previous address space */
	arm4_tlbset1(oldpage);
        /* flush the TLB */
        asm("mcr p15, #0, r0, c8, c7, #0");
}
</source>

Of course it is just minimal but it should be enough for you to get started with and improve it!

== Full Source ==
You can access the full source with:
<pre>
    git clone http://kmcg3413.net/armthin.git
    git checkout origin/bINT_TIMER_PIC_TASKS_VMM_MOD_ELF
</pre>

''The head (master branch) may contain more work beyond this page.''

== Going Further ==
At this point you have almost a fully functional kernel. Your missing better support for device drivers, services, and such things. Your also at the point where you can build different kernel types depending on your design. Your missing system calls for user space for memory allocation. Also, you need processes and threads instead of the more primitive task like design we have now. Also, you need to handle exceptions for user space correctly such as terminating the offending process or thread. Also, synchronization primitives and common services like timers and sleeping are missing. And, then again maybe you do not need these and you have some really awesome design!? Well, code to your heart's content is my suggestion!

''You can continue along with me or branch out in your own direction if you like, but in the next page we are going to create actual processes and threads, improve our exception handling, and add just a few basic services through system calls to threads.''

For lack of a better name, continue onward to, [[ARM_Integrator-CP_ITPTMME_Phase2|Phase 2]].

[[Category:ARM]]
````

<a id="w003484"></a>

## W003484 — User:Johnburger/Demo/Demo

Bron-ID: W003484
Namespace: User (2)
Revisie: 21353 | Datum: 2017-08-28T21:10:56Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Demo
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FDemo&oldid=21353

````text
The following is the complete source code. Not very big, is it? In fact, I'm proud of the fact that the assembled output is a mere 4.5kiB in size - not including any padding for Disk Image formats such as ISO.

Of course, I'm cheating. There are a number of <code>%include</code>s - which in turn often have their own <code>%include</code>s. In fact, I'm <em>also</em> proud of the fact that there are more '''''lines''''' in the resultant listing than there are '''''bytes''''' in the assembled output!

== <code>[[User:Johnburger/Demo/Overview#The_Source_Code|Demo]]/Demo.asm</code> ==
<source lang="asm">
;
; Demo.asm
;

%define         Version.Program   "Demonstrator"
%define         Version.Author    "John Burger"
%define         Version.Name      Version.Author, "'s 80386 ", Version.Program
%define         Version.Copyright "(c)2014"

%define         Version.Major   1
%define         Version.Minor   0
%define         Version.Build   1000

%defstr         Version.String  %[Version.Major].%[Version.Minor].%[Version.Build]

;*******************************************************************************

; This file is a complete example of the steps necessary to get a '386 (or
; later) PC to boot into Protected Mode, start some Tasks, and switch between
; them. It is completely written in assembly language, and uses no extra
; libraries.
;
; It has been written to be assembled with the Netwide Assembler NASM, which
; is a free download (http://www.nasm.us/) and will run on a number of different
; development platforms - Windows, Linux and OS-X.
;
; The NASM command line to assemble this source depends on the desired format:
; For CD:        nasm -o Demo.iso Demo.asm
; For USB:       nasm -o Demo.usb Demo.asm
; For Floppy:    nasm -o Demo.flp Demo.asm
; For Hard Disk: nasm -o Demo.bin Demo.asm
;
; Because NASM (by default) is a linear assembler, it is easy to use it to
; produce binary output. And since most Disk Image file formats are straight-
; forward binary, it is also easy to get NASM to produce these formats. Finally,
; most Disk Image formats can use the same binary, with extra 'decorations',
; so the same output can be used for multiple formats - simultaneously!
;
; By default, the binary output for this program can be used for any standard
; disk format - Floppy, USB, CD or even Hard Drive. You have two options;
; 1) Write it to physical media. Not as a file, but at a sector-by-sector level
;    starting at the first sector, using some form of low-leve drive writing
;    program (dd under Linux). Needless to say, this will effectively DESTROY
;    the existing data on the media! The easiest is probably to produce an ISO
;    file and use an ISO Burner to burn it to a CD.
;    Note that some USB sticks, and all hard drives, have a Partition Table
;    which divides the Drive into one or more Volumes. For these, you should
;    write the output to the sectors at the start of the Volume, not the Drive.
;    You can write it to the beginning of the Drive, but you'll DESTROY the
;    defined Partitions and everything in them!
; 2) Give the output file an appropriate extension (.flp, .img or .iso) and use
;    the output as a Floppy or CD stand-in with a virtualisation program like
;    VMWare. Note that in this case, the output cannot be used for a Hard Drive:
;    it is not in the .vmdk or .vdi form that these programs use.
;===============================================================================

; Define .map file output. Map files are your friend! They can help you work out
; whether the assembler understands what you thought you told it...
                [map all Demo.map]

; You can tailor the output (make it smaller) by modifying these %defines,
; depending on just what the final desired output format will be. See the usage
; of the IMAGE.* defines at the bottom of this file.
%define         IMAGE.ISO       ; Write as bootable CD (ISO-9660 and El Torito)
;%define         IMAGE.FLOPPY    ; Write as full Floppy image

%assign         Demo.Size       0     ; Starting size. See Sizes.inc below

;===============================================================================

; The following are just definitions. Lots and lots of definitions...
; I hate "magic" numbers. Only 0 and 1 are numbers; the rest need labels!
; And comments. Lots and lots of comments...
%include        "x86/x86.inc"   ; Definitions for CPU
;===============================================================================
%include        "BIOS/BIOS.inc" ; Definitions for BIOS
;===============================================================================
%include        "Dev/Dev.inc"   ; Definitions for other Devices
;===============================================================================
%include        "Demo.inc"      ; Definitions for the rest of the program

;*******************************************************************************

; This is the BIOS Boot entry point. It runs in Real Mode, and assumes the
; standard BIOS Boot Specification 1.01.

                USE16           ; Start in 16-bit Real Mode

%include        "Boot/Boot.inc" ; Real Mode bootstrap

;*******************************************************************************

; From here down is Protected Mode code. It is loaded by the above Boot code
; to the defined location, "jettisoning" the above code as it's no longer needed

                USE32           ; The rest is 32-bit Protected Mode

%include        "Data.inc"      ; Global Data
;===============================================================================
%include        "Ints/Ints.inc" ; Interrupt handlers
;===============================================================================
%include        "User/User.inc" ; User Mode program
;===============================================================================
%include        "Exec/Exec.inc" ; Supervisor Mode executive

;*******************************************************************************

; Finally, Pad the assembled image to make it usable as a Disk Image file.

%include        "Pad/Sizes.inc"    ; Calculate the end-of-Segment sizes

;===============================================================================
; First, Pad it up to the next Hard Disk size.
%include        "Pad/HardDisk.inc" ; Perform padding to generate Hard Disk image

; Can now calculate the final Load Size
Load.Size       EQU             Demo.Size - Real.Size

; See Boot/Load.inc for why
%if Load.Size > Load.Max
%error "Demo image too large for loader!"
%endif

;===============================================================================
; For any other Disk Image format, extra work is needed.

; If you're going to write it to a Floppy, USB or Hard Disk, the following isn't
; required.
; If you want a CD-Rom image (.iso), you don't need the Floppy stuff.
; If you want a Floppy image (.flp or .img), you don't need the ISO stuff.
; Leaving them in won't hurt, it just may be a larger file with LOTS of 0s.

%ifdef IMAGE.ISO
%assign         ISO.Start.Size    Demo.Size
%include        "Pad/ISO/ISO.inc" ; Perform padding to generate ISO image
%assign         Demo.Size         Demo.Size + ISO.Size
%endif ; IMAGE.ISO

;===============================================================================
%ifdef IMAGE.FLOPPY
%include        "Pad/Floppy.inc" ; Perform padding for Floppy image for VMWare
%endif ; IMAGE.FLOPPY
</source>
````

<a id="w003485"></a>

## W003485 — User:Johnburger/Demo/Demo.inc

Bron-ID: W003485
Namespace: User (2)
Revisie: 21406 | Datum: 2017-08-29T09:12:50Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Demo.inc
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FDemo.inc&oldid=21406

````text
This file contains most of the configurable values used by the Demonstrator. You can modify other values, but you need a deeper understanding of what they are, and how they affect other things in the system, before you can change them willy-nilly. (If you only <em>willy</em> change them, they should work. Go to the full <em>nilly</em>, and I'm not responsible...)

{| class="wikitable"
|-
! Identifier
! Description
|-
| <code>BIOS.VGA.90x60</code>
| Defining this will make the Boot Code change the screen to 90x60 text mode instead of 80x50 (the default). Note that not every system supports this mode - the versions of VMware that I tried did, while my late-generation laptops didn't.
|-
| <code>Timer.Tick</code>
| Changing this changes how many times a second the Timer will interrupt the PC. The Demonstrator changes tasks on every Timer Tick (there's also a <code>Yield</code> function...), so you can change this value and watch what happens.
|-
| <code>Window.Width</code><br><code>Window.Height</code>
| Changing these values has the effect of changing how many Tasks are created at run-time. Since each Task bounces a ball around its individual screen area, a larger area means fewer Tasks. A smaller area is intriguing: if it's too small to even draw a border, it doesn't bother; and if the area is 1x1, rather than bounce the ball it simply sits there and "rotates" - increments to show you that something is happening.

An interesting demonstration is to enable <code>BIOS.VGA.90x60</code> (above), and set these to 1x1. The result is so many tasks that it overflows the maximum number of '386 hardware Tasks - at least, it does the way I've set them up. Each Task has its own LDT, which means there's only room for a little less than 4,100 Tasks.
|-
| <code>User.Ball</code><br><code>User.BallColour</code><br><code>User.FrameColour</code>
| These are merely:
* The character representing the Ball that bounces around;
* The value for the colour to use for the Ball;
* The value for the colour to use for the Frame
|-
| <code>User.Delay</code>
| Changing this value merely alters the delay between updates of the Ball position. The code puts the CPU in a busy-waiting loop (which could be time-sliced away at any moment) before returning to work out the next position for the ball.

There are two other scheduling techniques you can try, and they're both commented out in the [[User:Johnburger/Demo/User/Code|User code]]:
* The Halt technique tells the CPU to Halt the current Task until the next interrupt. This is (of course) <em>very</em> unfair to the other Tasks:
** "Hey! If you're not using the CPU, why can't I?"
** The alternate argument is: "I'd use all of my timeslice anyway, just spinning my wheels. At least this way I'm not using CPU power..."
* The Yield technique tells the system to immediately switch to the next waiting Task, and don't wait for the Timer tick.
|}


== <code>[[User:Johnburger/Demo/Demo|Demo]]/Demo.inc</code> ==
<source lang="asm">
;
; Demo.inc
;

; These are definitions for the rest of the program. You can experiment by
; changing them. Some expect to be changed (these first few), while others
; will have far-reaching impacts.
;

; This %define changes things to invoke the BIOS to show more text:
;%define         BIOS.VGA.90x60 ; Allow use of 90x60 VGA (not universal!)

; Change these to (almost) whatever you'd like. If you're too extreme, you'll
; get assemble-time errors. Others you won't know until you try...
; For example, Timer.Tick is limited on virtualisation programs like VMware,
; so that even programming ludicrous values will result in staid performance.
; However, on real hardware there isn't that sort of control. Go on, set the
; Timer to 1,000,000 times a second - I dare you!
Timer.Tick      EQU             1000    ; Number of Timer Interrupts a second
Window.Width    EQU             8       ; Width of each Task's Window
Window.Height   EQU             6       ; Height of each Task's Window

User.Ball       EQU             1       ; Try 3: it is an OEM 'Heart' symbol
User.BallColour EQU             Dev.VGA.BlueBack  | Dev.VGA.White
User.FrameColour EQU            Dev.VGA.GreenBack | Dev.VGA.White
User.Delay      EQU             0FFFFh  ; How long to 'idle'the Bounce animation

; A useful pseudo-instruction to invoke the built-in Debugger - after it's set up!
%define         DEBUG           DB 0CCh ; INT 3 - NASM assembles this as 0CD 03h

; Another macro for debugging. Enable Single Step debugging - after it's set up!
%macro          TRACE           0
                PUSHFD
                OR      WORD    [ESP],x86.EFlags.TF
                POPFD
%endmacro

; The following is the memory map for this Demonstration. Note that some of the
; areas are left over from the BIOS - I'm simply working around those.
IVT.Base        EQU             x86.IVT.Base
IVT.Size        EQU             x86.IVT.Size
BDA.Base        EQU             BIOS.BDA.Base
BDA.Size        EQU             BIOS.BDA.Size
Data.Base       EQU             BDA.Base + BDA.Size
Data.Size       EQU             IDT.Base - Data.Base

IDT.Base        EQU             Code.Base - IDT.Max ; Final address of IDT
IDT.Max         EQU             x86.Ints * x86.Desc_size ; Full size of IDT

Code.Base       EQU             1000h

Load.Base       EQU             IDT.Start       ; Load from start of defined IDT
Load.Max        EQU             8 * BIOS.Disk.Sector.Size ; See Boot/Load.inc for why

; This will be the first-available RAM after everything else is initialised
Demo.First      EQU             Load.Base + Load.Size + GDT.Size + Exec.Stack.Size

Exec.Stack.Top  EQU             Demo.First      ; This is the TOP of an Expand-Down stack
Exec.Stack.Size EQU             1000h

; The PICs need to be re-based to get them away from the Intel-reserved interrupts
PIC.A.Base      EQU             20h
PIC.B.Base      EQU             28h

; See Ints/Timer.inc for a discussion on this (significant!) definition
System.TSS      EQU             x86.Desc.Mem.Gran.Avail

; This is the Descriptor for the Alias for the Descriptor Table that holds it
DT.Alias        EQU             x86.Desc_size

; The following structure is held in the first (NULL) GDT entry. It holds the
; information about the current state of the GDT: how big it is, and which
; entries have been returned.
; It is designed to enable the following instruction to be used:
;         LGDT  [GDT.Alloc.Pseudo]
; This is accomplished by:
; 1) Storing the linear base address of the GDT in .Pseudo.Base
; 2) Storing the current GDT size in .Pseudo.Size
; 3) As entries are freed, adding them to the .Free list (this uses the entries
;    themselves to store the next Free one, or 0 to end the list).
                STRUC           GDT.Alloc
.Free           RESW            1       ; Index to first Free Descriptor, or 0
.Pseudo         EQU             $       ; You can do LGDT [GDT.Alloc.Pseudo]
.Pseudo.Limit   EQU             $
.Limit          RESW            1       ; Last allocated Descriptor
.Pseudo.Base    RESD            1       ; GDT.Base, as part of a pseudo-descriptor
                ENDSTRUC

; Like the GDT, the following structure is held in the first LDT entry - even
; though this is a valid entry according to Intel's definition.
; But by putting it there, and setting it up so that the important fields
; overlap the same fields in GDT.Alloc, the same code can be used to
; allocate Descriptors in either table.
; The "important fields" are simply .Free and .Limit
                STRUC           LDT.Alloc
.Free           RESW            1       ; Index to first Free Descriptor or 0
.Limit          RESW            1       ; Last (ever) allocated Descriptor
                RESB            1       ; Reserved
.Type           RESB            1       ; Make sure any attempt at access fails!
.GDT            RESW            1       ; This is the LDT's entry in the GDT
                ENDSTRUC

; As stated above, the two ?DT.Alloc structures have common elements:
                STRUC           DT.Alloc
.Free           RESW            1       ; Index to first Free Descriptor, or 0
.Limit          RESW            1       ; Last allocated Descriptor
                ENDSTRUC
</source>
````

<a id="w003486"></a>

## W003486 — User:Johnburger/Demo/Boot/Boot

Bron-ID: W003486
Namespace: User (2)
Revisie: 21355 | Datum: 2017-08-28T21:11:16Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Boot/Boot
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FBoot%2FBoot&oldid=21355

````text
The following code is the entire Real Mode code in the system. It prepares the system for Protected Mode by setting up relevant tables, but it also calls the BIOS to perform tasks, or get information to save away for later. The final thing it does is "stamp" the code with a BIOS signature, so that the various boot loaders will recognize it as a valid - albeit never-seen-before - Boot Sector.

== Boot Sector size ==
One thing to note is that we only have 512 bytes to do everything. If all options are enabled (<code>BIOS.VGA.90x60</code> takes a <em>lot</em> of work!) there are only a handful of bytes left. For this reason the following code doesn't use the formal, correct, and highly-laudable modular programming technique of defined subroutines that are individually <code>CALL</code>ed - I don't have the space to waste on three bytes for every call, and one byte for every return.

=== Fall-through Code ===
Instead, everything is written as a stream-of-consciousness - sorry, -execution - paradigm where the tail end of one module (yes, there are still modules!) simply falls through to the start of the next one. There are places where <code>CALL</code>/<code>RET</code> are used - whenever the same bunch of code needs to be executed more than once, I put it into a subroutine. But that pesky <code>RET</code> at the end interferes with the stream-of-cons... er... -execution paradigm, and I have to <code>JMP</code> over it.

But sometimes I use a different technique. If I'm about to fall into the subroutine with carefully-set-up register values, and when I've finished I'm about to <code>JMP</code> past the subroutine anyway, instead I cheat. Instead of
<source lang="asm">
             CALL        .Sub
             JMP         .Past
.Sub:
; Do stuff here
             RET
.Past:
</source>
I use an assembly trick and code the following:
<source lang="asm">
             PUSH        .Past   ; Fake a CALL from Past
.Sub:
; Do stuff here
             RET
.Past:
</source>
When the <code>RET</code> is executed, it <code>POP</code>s off the return address - which is Past where I want to be!

I don't normally condone this kind of coding, but sometimes it's necessary in constrained environments - and this is very constrained indeed!

=== Common <code>JMP</code> Code ===
Having justified my use of a coding technique that would have [http://en.wikipedia.org/wiki/Niklaus_Wirth| Niklaus Wirth] turning in his (hopefully long-postponed!) grave, I need to mention a more common coding technique that I also use, this time in code that is less constrained. I have some code that is repeated a number of times, but is extremely similar. Given a little initialization code, the rest is identical. In those cases, I do use the following construct:

<source lang="asm">
Entry1:
          ; Perform setup for situation #1
          JMP       CommonCode
Entry2:
          ; Perform setup for situation #2
          JMP       CommonCode
Entry3:
          ; Perform setup for situation #3
          JMP       CommonCode
Entry4:
          ; Perform setup for situation #4
;         JMP       CommonCode
CommonCode:
          ; Do what's common
</source>

For <em>this</em> technique, I am not apologetic. In fact, the above also demonstrates another technique I use: sometimes code is "obvious", usually by pattern with other code around it, but the effect of that obvious code is a no-effect - in other words, a waste of space. I have three choices:
# Leave the code in, taking up some measly number of bytes;
# Remove the code completely, meaning that in the future I might just notice the break in the pattern and unthinkingly add it in again; or
# Leave the code in, but comment it out to prevent it from being assembled.
Needless to say, I chose the last option.

== Memory Map ==
At this point in time (BIOS Boot complete), the Memory Map looks something like:
{| class="wikitable"
! Address
! Usage
|-
| <code>0000_0000h</code>
| Interrupt Vector Table
|-
| <code>0000_0400h</code>
| <code>SS:SP</code> Stack Top <code>*</code>
|-
| <code>0000_0400h</code>
| BIOS Data Area
|-
| <code>0000_0500h</code>
| Available
|-
| <code>0000_0600h</code>
| Master Boot Record (MBR) <code>**</code>
|-
| <code>0000_0800h</code>
| Available
|-
| <code>0000_7C00h</code>
| BIOS-loaded Boot Sector
|-
| <code>0000_7E00h</code>
| Available
|-
| <code>0009_F???h</code> <code>***</code>
| Extended BIOS Data Area
|-
| <code>000A_0000h</code>
| Adapter / ROM Area
|-
| <code>000B_8000h</code>
| Text Video Memory
|-
| <code>000F_0000h</code>
| BIOS ROM
|-
| <code>0010_0000h</code>
| High Memory <code>****</code>
|}
<code>*</code> Yes! Believe it or not, the BIOS initializes the stack to point to the end of the Interrupt Vector table. Every <code>CALL</code> or <code>PUSH</code> you make adds a new, interesting vector to the IVT...
<br>
<code>**</code> The MBR only exists here if the PC has booted from a Hard Drive.
<br>
<code>***</code> An INT 12h will help identify this value.
<br>
<code>****</code> Note that this memory is only accessible in Protected Mode - except for the first 64 kiB (less 16 bytes), which you can access with a Segment Register set to <code>0FFFFh</code> as long as the A20 Gate is off. By the way: <strong>DON'T DO THIS!</strong>

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Overview#Boot_Sequence|Boot]]/Boot.inc</code> ==
<source lang="asm">
;
; Boot/Boot.inc
;

; This is the BIOS Boot entry point. It runs in Real Mode, and assumes the
; standard BIOS Boot Specification 1.01.
;
; It performs the following actions:
; 1) Loads the rest of the code into RAM;
; 2) Gets some information from the soon-to-be-unusable BIOS;
; 3) Sets up the PC for Protected Mode;
; 4) JMPs to the freshly-loaded Protected Mode code
; After that, the memory used to hold this code is avaiable to the system.
;
; Note that one of the goals of the Boot sector is to jettison itself:
; 7C00h is such an awkward location, so ideally after doing all the Real Mode
; initialisation, we can JMP to Protected Mode and simply leave this code behind.

; To save space, I take advantage of some HORRIBLE things you can do with
; assembly language: PUSHing values to fake CALLs, or worse, SELF-MODIFYING CODE!
; In Protected Mode, this last is very difficult to organise - and unreliable
; with later processors that have on-board caches - so not worth it.
; In Real Mode, this can be very useful as a register- and code-saving device!

; To save space, I've done away with byte-wasting CALLs and RETs for "modular"
; programming. Not that modules aren't in and of themselves useful - indeed,
; they're still modularised within their own little files - but the result is
; simply a stream of bytes to execute. Where I have defined subroutines to be
; CALLed, the stream of code has to either JMP over the subroutine's RET, or
; a CALL is faked by PUSHing the desired continuation address onto the stack.

;-------------------------------------------------------------------------------
                SEGMENT         Real  VSTART=0  ALIGN=1

                USE16                      ; Real Mode? 16 bits!

; At this point, DL is probably the drive number of the Boot Drive.
; ES:SI is possibly a pointer to the MBR Partition table entry (See Load.inc).
; In any case: don't clobber DL, ES or SI before Load.inc!

%include        "Boot/Entry.inc" ; Entry code to initialise important registers
;-------------------------------------------------------------------------------
%include        "Boot/CPU.inc"  ; Check CPU is (at least) a '386
;-------------------------------------------------------------------------------
%include        "Boot/Load.inc" ; Load Protected Mode code
;-------------------------------------------------------------------------------
%include        "Boot/IDT.inc"  ; Move IDT to final location
;-------------------------------------------------------------------------------
%include        "Boot/RAM.inc"  ; Get RAM size
;-------------------------------------------------------------------------------
%include        "Boot/A20.inc"  ; Enable A20 gate
;-------------------------------------------------------------------------------
%include        "Boot/Key.inc"  ; Speed up keyboard
;-------------------------------------------------------------------------------
%include        "Boot/VGA.inc"  ; Switch Video mode
;-------------------------------------------------------------------------------
%include        "Boot/PM.inc"   ; Switch to Protected Mode
;-------------------------------------------------------------------------------

; At this point, we still must be inside the first 512-byte Boot sector.
; We need to confirm to the BIOS that this sector is a valid Boot sector by
; having BIOS.Sig.Value at the correct place.
; Note that if the previous code was too large, the following Pad calculation
; will underflow and generate an error.

Real.Padding    EQU             BIOS.Sig.Pos - ($ - $$)
TIMES           Real.Padding    DB 00h

                DW              BIOS.Sig.Value
</source>
````

<a id="w003487"></a>

## W003487 — User:Johnburger/Demo/x86/x86

Bron-ID: W003487
Namespace: User (2)
Revisie: 28462 | Datum: 2023-11-15T08:40:19Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/x86/x86
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2Fx86%2Fx86&oldid=28462

````text
The following is a large number of definitions, all for the '386. There's no code for any of this: it's all just definitions. The designers of the '386 have burned silicon that strictly defines its operation - indeed, you could almost consider it the DNA of the processor.

As well as defining mere constants, though, an even better practice that I have followed is to define <code>%macro</code>s - and use them. For example: in Real Mode, a Segment Register holds a value that is 4 right-shifts from the memory that it references. For any code that loads a Segment Register, it is (relatively) obvious what it's going to do (you <em>have</em> defined labels for the Segment values for <code>Interrupt Vector Table</code>, <code>BIOS Data Area</code>, and <code>Screen</code>, haven't you?)

But in Protected Mode, a Segment Register holds a selector to a Segment Descriptor that is comprised of three parts - and it is inordinately easy to forget to load all the parts, causing a General Protection Fault, or worse!

<pre>
       1111110000000 0 00
       FEDCBA9876543 2 10
       ^^^^^^^^^^^^^ ^ ^^
       ||||||||||||| | `+-- Requested Privilege Level: (00-03)
       ||||||||||||| `----- Table Indicator:            0=GDT, 1=LDT
       `++++++++++++------- Actual selector into Indicated Descriptor Table
</pre>

To save myself lots of grief (can you tell I learned this the hard way?), I defined a <code>%macro</code> called <code>Selector</code> with all three parameters:
<source lang="asm">
             MOV     AX,Selector(Index,LDT,RPL3)
</source>

That way the assembler can warn me that I've forgotten one of them - now all I have to do is:
# Remember to use it (easy!); and
# '''''Think''''' when I use it, that I have every parameter correct!

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Overview#x86_Definitions|x86]]/x86.inc</code> ==
<source lang="asm">
;
; x86/x86.inc
;

; The following are definitions for the 80386 and later CPUs.
;
; Note that they're defined by Intel as part of the 'DNA' of the chip: almost
; literally, if you think of the silicon as defining the CPU's operation.

;-------------------------------------------------------------------------------
%include        "x86/EFlags.inc"        ; Definitions for EFlags register
;-------------------------------------------------------------------------------
%include        "x86/Seg.inc"           ; Definitions for Segment register
;-------------------------------------------------------------------------------
%include        "x86/CR0.inc"           ; Definitions for CR0 register
;-------------------------------------------------------------------------------
%include        "x86/Desc.inc"          ; Definitions for Descriptors
;-------------------------------------------------------------------------------
%include        "x86/TSS.inc"           ; Definitions for Task State Segment
;-------------------------------------------------------------------------------
%include        "x86/IVT.inc"           ; Definitions for Interrupt Vector Table
</source>
````

<a id="w003488"></a>

## W003488 — User:Johnburger/Demo/x86/Seg

Bron-ID: W003488
Namespace: User (2)
Revisie: 21407 | Datum: 2017-08-29T09:13:02Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/x86/Seg
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2Fx86%2FSeg&oldid=21407

````text
At the risk of [[User:Johnburger/Demo/x86/x86|repeating myself]], a Segment is defined by Intel to point into a Descriptor Table - either the GDT, LDT or IDT.

<pre>
       1111110000000 0 00
       FEDCBA9876543 2 10
       ^^^^^^^^^^^^^ ^ ^^
       ||||||||||||| | `+-- Requested Privilege Level: (00-03)
       ||||||||||||| `----- Table Indicator:            0=GDT, 1=LDT
       `++++++++++++------- Actual selector into Indicated Descriptor Table
</pre>
To make it easy on myself, I defined some constants, and some macros, that let me do the following:
<source lang="asm">
             MOV     AX,Selector(Index,LDT,RPL3)
</source>

At least now I can write code that <em>warns</em> me if I forget a critical parameter - like the fact that the Index is in the Local Descriptor Table, rather than the Global one!

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/x86/x86|x86]]/Seg.inc</code> ==
<source lang="asm">
;
; x86/Seg.inc
;

; These are the definitions for the Segment selector registers

x86.Seg.Index   EQU            1111_1111_1111_1000b  ; Index into Table
x86.Seg.TI      EQU            0000_0000_0000_0100b  ; Table Indicator (GDT/LDT)
x86.Seg.RPL     EQU            0000_0000_0000_0011b  ; Requested Privilege Level

x86.Seg.LDT     EQU            x86.Seg.TI       ; Add this in to every LDT seg
x86.Seg.GDT     EQU            0

x86.Seg.RPL3    EQU            0000_0000_0000_0011b
x86.Seg.RPL2    EQU            0000_0000_0000_0010b
x86.Seg.RPL1    EQU            0000_0000_0000_0001b
x86.Seg.RPL0    EQU            0000_0000_0000_0000b

; A helper macro, making it easy to set all the fields of a Selector.
; The parameters are: Table Index, GDT or LDT, and RPL0-3
%define         Selector(Index, TI, RPL) ( Index + (x86.Seg.%+ TI | x86.Seg.%+ RPL) )
</source>
````

<a id="w003489"></a>

## W003489 — User:Johnburger/Demo/x86/EFlags

Bron-ID: W003489
Namespace: User (2)
Revisie: 21408 | Datum: 2017-08-29T09:13:15Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/x86/EFlags
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2Fx86%2FEFlags&oldid=21408

````text
Personally, I prefer Borland's TASM and Microsoft's MASM syntax for bitfields: you can define a <code>RECORD</code> for a bitfield, and then each individual bit or range of bits are accessible through the use of keywords like <code>SHIFT</code>, <code>MASK</code> and <code>WIDTH</code> to reference the fields.

You want to isolate x86.EFlags.IOPL?
<source lang="asm">
x86.EFlags      RECORD
                .NT : 1         ; Nested Task Flag
                .IOPL : 2       ; I/O Privilege Level
                .OF : 1         ; Parity Flag
                .DF : 1         ; Direction Flag
                .IF : 1         ; Interrupt Flag
                .TF : 1         ; Trace Flag
                .SF : 1         ; Sign Flag
                .ZF : 1         ; Zero Flag
                    : 1         ; Reserved
                .AF : 1         ; Arithmetic Flag
                .0  : 1         ; Always 0
                .PF : 1         ; Parity Flag
                .1  : 1         ; Always 1
                .CF : 1         ; Carry Flag
x86.EFlags      ENDS

                PUSHFD
                POP             EAX
                AND             EAX,MASK x86.EFlags.IOPL
                SHR             EAX,SHIFT x86.EFlags.IOPL
                MOV             CL,WIDTH x86.EFlags.IOPL
</source>
(Of course, I've muddied the example by using NASM's sophisticated local-label syntax that TASM doesn't support, and I've used a non-supported "missing bit" syntax extension...)

I've seen posts asking why the whole <code>RECORD</code>, <code>MASK</code>, <code>SHIFT</code>, and <code>WIDTH</code> infrastructure exists: '''''THIS''''' is why! NASM doesn't support the keywords: I miss them...

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/x86/x86|x86]]/EFlags.inc</code> ==
<source lang="asm">
;
; x86/EFlags.inc
;

; The following are the bitfields in the EFlags register
x86.EFlags.ID   EQU             0000_0000_0010_0000_0000_0000_0000_0000b
x86.EFlags.VIP  EQU             0000_0000_0001_0000_0000_0000_0000_0000b
x86.EFlags.VIF  EQU             0000_0000_0000_1000_0000_0000_0000_0000b
x86.EFlags.AC   EQU             0000_0000_0000_0100_0000_0000_0000_0000b
x86.EFlags.VM   EQU             0000_0000_0000_0010_0000_0000_0000_0000b
x86.EFlags.RF   EQU             0000_0000_0000_0001_0000_0000_0000_0000b
x86.EFlags.NT   EQU             0000_0000_0000_0000_0100_0000_0000_0000b
x86.EFlags.IOPL EQU             0000_0000_0000_0000_0011_0000_0000_0000b
x86.EFlags.OF   EQU             0000_0000_0000_0000_0000_1000_0000_0000b
x86.EFlags.DF   EQU             0000_0000_0000_0000_0000_0100_0000_0000b
x86.EFlags.IF   EQU             0000_0000_0000_0000_0000_0010_0000_0000b
x86.EFlags.TF   EQU             0000_0000_0000_0000_0000_0001_0000_0000b
x86.EFlags.SF   EQU             0000_0000_0000_0000_0000_0000_1000_0000b
x86.EFlags.ZF   EQU             0000_0000_0000_0000_0000_0000_0100_0000b
x86.EFlags.AF   EQU             0000_0000_0000_0000_0000_0000_0001_0000b
x86.EFlags.PF   EQU             0000_0000_0000_0000_0000_0000_0000_0100b
x86.EFlags.1    EQU             0000_0000_0000_0000_0000_0000_0000_0010b
x86.EFlags.CF   EQU             0000_0000_0000_0000_0000_0000_0000_0001b
</source>
````

<a id="w003490"></a>

## W003490 — User:Johnburger/Demo/x86/CR0

Bron-ID: W003490
Namespace: User (2)
Revisie: 21409 | Datum: 2017-08-29T09:13:26Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/x86/CR0
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2Fx86%2FCR0&oldid=21409

````text
Along the lines of [[User:Johnburger/Demo/x86/EFlags|EFlags]], this is the bitfield definition for Control Register 0 (CR0):

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/x86/x86|x86]]/CR0.inc</code> ==
<source lang="asm">
;
; x86/CR0.inc
;

; The following are the bitfields in the CR0 register
x86.CR0.PG      EQU             1000_0000_0000_0000_0000_0000_0000_0000b
x86.CR0.CD      EQU             0100_0000_0000_0000_0000_0000_0000_0000b
x86.CR0.NW      EQU             0010_0000_0000_0000_0000_0000_0000_0000b
x86.CR0.AM      EQU             0000_0000_0000_0100_0000_0000_0000_0000b
x86.CR0.WP      EQU             0000_0000_0000_0001_0000_0000_0000_0000b
x86.CR0.NE      EQU             0000_0000_0000_0000_0000_0000_0010_0000b
x86.CR0.ET      EQU             0000_0000_0000_0000_0000_0000_0001_0000b
x86.CR0.TS      EQU             0000_0000_0000_0000_0000_0000_0000_1000b
x86.CR0.EM      EQU             0000_0000_0000_0000_0000_0000_0000_0100b
x86.CR0.MP      EQU             0000_0000_0000_0000_0000_0000_0000_0010b
x86.CR0.PE      EQU             0000_0000_0000_0000_0000_0000_0000_0001b
</source>
````

<a id="w003491"></a>

## W003491 — User:Johnburger/Demo/x86/Desc

Bron-ID: W003491
Namespace: User (2)
Revisie: 21410 | Datum: 2017-08-29T09:13:39Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/x86/Desc
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2Fx86%2FDesc&oldid=21410

````text
This is both the most involved x86 definition file and the most useful. Since I decided to define as many system tables as possible at assembly time rather than run time, defining Descriptors became a frequent occurrence. And it didn't take long for NASM's structure instantiation mechanism (<code>ISTRUC</code> - '''''*shudder*''''') to muddy the code enough to warrant a <code>%macro</code> to help.

I'm actually quite proud of the result - take a look at the [[User:Johnburger/Demo/Ints/IDT|Interrupt Descriptor Table definition]] for an example of how clean the resultant definition really is.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/x86/x86|x86]]/Desc.inc</code> ==
<source lang="asm">
;
; x86/Desc.inc
;

; Descriptor Tables hold Descriptor Table Entries, or Descriptors for short.
; To select a Descriptor, you load a Segment register with a Selector (see Seg.inc).

; The following is the generic structure of a Descriptor (not very useful!)
                STRUC           x86.Desc
                RESD            1
                RESB            1
.Type           RESB            1       ; All Descriptors have a Type field here
                RESW            1
                ENDSTRUC                ; All Descriptors have the same Desc_size

; The following is the structure of a System Descriptor
                STRUC           x86.Desc.Sys
.OffsetLo       RESW            1
.Selector       RESW            1
.Args           RESB            1
.Type           RESB            1
.OffsetHi       RESW            1
                ENDSTRUC

; The following is the structure of a Memory (Code/Data) Descriptor
                STRUC           x86.Desc.Mem
.LimitLo        RESW            1
.BaseLo         RESW            1
.BaseMid        RESB            1
.Type           RESB            1
.Gran           RESB            1
.BaseHi         RESB            1
                ENDSTRUC

; The .Type field in both .Sys and .Mem structures above have the same format.
; These are the bitfields of the .Type field
x86.Desc.Type.Present EQU       1000_0000b

; These are the different Descriptor Privilege Levels
x86.Desc.Type.DPL3    EQU       0110_0000b
x86.Desc.Type.DPL2    EQU       0100_0000b
x86.Desc.Type.DPL1    EQU       0010_0000b
x86.Desc.Type.DPL0    EQU       0000_0000b

x86.Desc.Type.Sys     EQU       0000_0000b
x86.Desc.Type.Mem     EQU       0001_0000b

x86.Desc.Type.Type    EQU       0000_1111b

;-------------------------------------------------------------------------------

; These are the .Type.Type field values and modifiers for System Descriptors
x86.Desc.Sys.Type.TSS   EQU     0000_0001b      ; Task State Segment
x86.Desc.Sys.Type.LDT   EQU     0000_0010b      ; Local Descriptor Table
x86.Desc.Sys.Type.Busy  EQU     0000_0011b      ; Busy TSS
x86.Desc.Sys.Type.Gate  EQU     0000_0111b      ; One of a number of Gates
x86.Desc.Sys.Type.Call  EQU     0000_0100b      ; Call Gate (switches Priv)
x86.Desc.Sys.Type.Task  EQU     0000_0101b      ; Task Gate (switches TSS)
x86.Desc.Sys.Type.Int   EQU     0000_0110b      ; Int Gate (disables interrupts)
x86.Desc.Sys.Type.Trap  EQU     0000_0111b      ; Trap Gate
x86.Desc.Sys.Type.386   EQU     0000_1000b      ; '386 variant of above
x86.Desc.Sys.Type.286   EQU     0000_0000b      ; '286 is the original ones
; The .386 modifier is used for 32-bit variants of the original '286 16-bit ones.
; Note it cannot be used for either .LDT or .Task

;-------------------------------------------------------------------------------

; These are the .Type.Type field values and modifiers for Memory Descriptors
x86.Desc.Mem.Type.Code     EQU  0000_1000b
x86.Desc.Mem.Type.Data     EQU  0000_0000b
x86.Desc.Mem.Type.Conform  EQU  0000_0100b ; Code can be executed at different PLs
x86.Desc.Mem.Type.Expand   EQU  0000_0100b ; Base is TOP, Limit is size BELOW that
x86.Desc.Mem.Type.Stack    EQU  0000_0100b ; (Best used for Stacks)
x86.Desc.Mem.Type.RW       EQU  0000_0010b ; Readable for Code, Writable for Data
x86.Desc.Mem.Type.NoRW     EQU  0000_0000b ; No Read for Code, no Write for Data
x86.Desc.Mem.Type.Accessed EQU  0000_0001b ; Set when Descriptor loaded

; These are the bitfields of the .Gran field in Memory Descriptors
x86.Desc.Mem.Gran.LimitHi EQU   0000_1111b

x86.Desc.Mem.Gran.Gran  EQU     1000_0000b ; Use Page-granular Limit
x86.Desc.Mem.Gran.Byte  EQU     0000_0000b ; Use byte-granular Limit

x86.Desc.Mem.Gran.Big   EQU     0100_0000b ; Use EIP and ESP for Code and Stacks
x86.Desc.Mem.Gran.Small EQU     0000_0000b ; Use IP and SP for Code and Stacks

x86.Desc.Mem.Gran.Def32 EQU     0100_0000b ; Default to 32-bit instructions
x86.Desc.Mem.Gran.Def16 EQU     0000_0000b ; Default to 16-bit instructions

x86.Desc.Mem.Gran.Avail EQU     0001_0000b ; Available for system use

;-------------------------------------------------------------------------------

; Helper macros for each of the .Types defined above. There are a number of
; fields that need to be set, making for long source lines, and it's easy to
; forget one! Each of these assume only one thing: they're .Present

; Parameters are: TSS or LDT or Call or Task or Int or Trap, DPL 0-3, 286 or 386
%define         Type.Sys(Type, DPL, x86) (\
                x86.Desc.Type.Present | x86.Desc.Type.Sys | \
                x86.Desc.Type.%+ DPL | x86.Desc.Sys.Type.%+ Type | x86.Desc.Sys.Type.%+ x86 \
                )

; Parameters are: Code or Data or Stack, DPL 0-3, and RW or NoRW
%define         Type.Mem(Type, DPL, RW) (\
                x86.Desc.Type.Present | x86.Desc.Type.Mem | \
                x86.Desc.Type.%+ DPL | x86.Desc.Mem.Type.%+ Type | x86.Desc.Mem.Type.%+ RW \
                )

; Parameters are: Gran or Byte, and Big or Small
%define         Gran.Mem(Gran, Size) (\
                x86.Desc.Mem.Gran.%+ Gran | x86.Desc.Mem.Gran.%+ Size \
                )

; A helper macro, making it easy to allocate a System Descriptor in a Table.
; Unfortunately this macro requires that the Offset is less than 64K.
; Tne parameters are: Selector, Offset, Type
%macro          Desc.Sys                   3
%00             ISTRUC x86.Desc.Sys
AT x86.Desc.Sys.OffsetLo,  DW              %2
AT x86.Desc.Sys.Selector,  DW              %1
AT x86.Desc.Sys.Args,      DB              0
AT x86.Desc.Sys.Type,      DB              %3
AT x86.Desc.Sys.OffsetHi,  DW              0
                IEND
%endmacro

; A helper macro, making it easy to allocate a Memory Descriptor in a Table.
; The parameters are: Base, Limit, Type, and Gran
%macro          Desc.Mem                   4
%00             ISTRUC x86.Desc.Mem
AT x86.Desc.Mem.LimitLo,   DW              %2
AT x86.Desc.Mem.BaseLo,    DW              (%1 >> 0) & 0FFFFh
AT x86.Desc.Mem.BaseMid,   DB              (%1 >> 16) & 0FFh
AT x86.Desc.Mem.Type,      DB              %3
AT x86.Desc.Mem.Gran,      DB              %4
AT x86.Desc.Mem.BaseHi,    DB              (%1 >> 24) & 0FFh
                IEND
%endmacro
</source>
````

<a id="w003492"></a>

## W003492 — User:Johnburger/Demo/x86/TSS

Bron-ID: W003492
Namespace: User (2)
Revisie: 21411 | Datum: 2017-08-29T09:13:52Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/x86/TSS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2Fx86%2FTSS&oldid=21411

````text
Although the Task State Segment is (as described) a Segment, it's actually merely the definition for the <em>beginning</em> of a Segment. Intel wisely decided to let (nearly<code>*</code>) the rest of the Segment be used for other stuff relating to the Task that it holds the state for - a perfect example is the current Floating Point Unit state.

This Demonstrator doesn't handle the Floating Point execution Unit (FPU) at all - adding that is a project for the reader! There are two strategies:
* On every task switch, just before executing the <code>JMP</code> that switches Tasks, first dump the current FPU's state to RAM just above the TSS structure. And on Task resume, recover the FPU state and let it resume.<br>
This has the advantage of simplicity, but a bigger disadvantage: many Floating Point operations take a while to complete, and before an FPU store operation can start it needs to finish what it's doing. Worse, often there aren't many Tasks even using the FPU - it may very well be that the Task resuming operation of the FPU was the very one that dumped its state a few Task Switches ago!
* To help this latter case, Intel defined a Flag (<code>TS</code> in [[User:Johnburger/Demo/x86/CR0|<code>CR0</code>]]) and an Exception. On every hardware Task Switch, the Flag is set to indicate that a Task has been switched. On every Floating Point operation that flag is tested: if it is set, the CPU raises an <code>INT 7</code> - Coprocessor Not Available exception. That fault handler can quickly determine if it's inside a different Task than what was previously using the FPU: if not, do nothing; if so, only <em>now</em> save the FPU state and recover the new Task's FPU state.

----
<code>*</code> And what did I mean by (nearly) above? One of the things that a TSS can do is to define which I/O ports a Task can access. If it is a Supervisor Task, it can (and should) access all available ports - for example, the Interrupt Acknowledge ports on the PICs. A User Task, however, can do immeasurable damage if it could access any port it liked. To that end, a TSS can have a bitmap (<code>.IOMap</code> below) that defines which bits the Task is allowed to access - anything else will result in a General Protection Fault. That bitmap is defined as part of the TSS - an array of bits from <code>.IOMap</code> to the Limit of the TSS.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/x86/x86|x86]]/TSS.inc</code> ==
<source lang="asm">
;
; x86/TSS.inc
;

; This module defines the Task State Segment (the 32-bit one, anyway).

x86.TSS.Flags.Trap  EQU         0000_0000_0000_0001b

                STRUC           x86.TSS
.Back           RESW            1               ; Back-link to calling TSS
                RESW            1
.ESP0           RESD            1               ; Stack 0 pointer
.SS0            RESW            1
                RESW            1
.ESP1           RESD            1               ; Stack 1 pointer
.SS1            RESW            1
                RESW            1
.ESP2           RESD            1               ; Stack 2 pointer
.SS2            RESW            1
                RESW            1
.CR3            RESD            1               ; Page Directory Base Register
.EIP            RESD            1
.EFlags         RESD            1
.EAX            RESD            1
.ECX            RESD            1
.EDX            RESD            1
.EBX            RESD            1
.ESP            RESD            1
.EBP            RESD            1
.ESI            RESD            1
.EDI            RESD            1
.ES             RESW            1
                RESW            1
.CS             RESW            1
                RESW            1
.SS             RESW            1
                RESW            1
.DS             RESW            1
                RESW            1
.FS             RESW            1
                RESW            1
.GS             RESW            1
                RESW            1
.LDT            RESW            1
                RESW            1
.Flags          RESW            1
.IOMap          RESW            1
                ENDSTRUC
</source>
````

<a id="w003493"></a>

## W003493 — User:Johnburger/Demo/x86/IVT

Bron-ID: W003493
Namespace: User (2)
Revisie: 21412 | Datum: 2017-08-29T09:14:41Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/x86/IVT
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2Fx86%2FIVT&oldid=21412

````text
This Demonstrator doesn't need to modify the Real-mode Interrupt Vector Table - in fact, it avoids it completely! While setting up Protected Mode it issues various BIOS calls (which uses the IVT to get to the handler), and it carefully avoids using that 1 kiB of memory for anything else.

It avoids the BIOS Data Area and Extended BIOS Data Area too, just in case you ever want to try going back to Real Mode (or even Unreal Mode).

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/x86/x86|x86]]/IVT.inc</code> ==
<source lang="asm">
;
; x86/IVT.inc
;

; These are the definitions for the (Real Mode) Interrupt Vector Table.

x86.Ints        EQU             256

x86.IVT.Base    EQU             0000h         ; Interrupt Vector Table
x86.IVT.Size    EQU             x86.Ints * 4  ; This many DWORD vectors
</source>
````

<a id="w003494"></a>

## W003494 — User:Johnburger/Demo/Boot/Entry

Bron-ID: W003494
Namespace: User (2)
Revisie: 21394 | Datum: 2017-08-29T09:06:34Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Boot/Entry
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FBoot%2FEntry&oldid=21394

````text
On entry to my code, different BIOSes have different values for <code>CS:IP</code>. I want to use <code>CS</code>-relative addressing, so the very first thing I do is make sure that <code>CS</code> is where I want it!

Then I fix the Stack so that it is no longer clobbering the Real Mode [[User:Johnburger/Demo/Boot/Boot#Memory_Map|IVT]]...

== Memory Map ==
After this code, the Memory Map will look like this:
{| class="wikitable"
! Address
! Usage
|-
| <code>0000_0000h</code>
| Interrupt Vector Table
|-
| <code>0000_0400h</code>
| BIOS Data Area
|-
| <code>0000_0500h</code>
| Available
|-
| <code>0000_0600h</code>
| Master Boot Record (MBR) <code>*</code>
|-
| <code>0000_0800h</code>
| Available
|-
| <strong><code>0000_7C00h</code></strong>
| <strong><code>SS:SP</code> Stack Top</strong>
|-
| <code>0000_7C00h</code>
| BIOS-loaded Boot Sector
|-
| <code>0000_7E00h</code>
| Available
|-
| <code>0009_F???h</code> <code>**</code>
| Extended BIOS Data Area
|-
| <code>000A_0000h</code>
| Adapter / ROM Area
|-
| <code>000B_8000h</code>
| Text Video Memory
|-
| <code>000F_0000h</code>
| BIOS ROM
|-
| <code>0010_0000h</code>
| High Memory <code>***</code>
|}
<code>*</code> The MBR only exists here if the PC has booted from a Hard Drive.
<br>
<code>**</code> An INT 12h will help identify this value.
<br>
<code>***</code> Note that this memory is only accessible in Protected Mode - except for the first 64 kiB (less 16 bytes), which you can access with a Segment Register set to <code>0FFFFh</code> as long as the A20 Gate is off. By the way: <strong>DON'T DO THIS!</strong>

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Boot/Boot|Boot]]/Entry.inc</code> ==
<source lang="asm">
;
; Boot/Entry.inc
;

                ORG             0000h

; On Entry, different BIOSes have different values for CS:IP.
; I want to use CS-relative addressing, so make sure CS is where I want it!
Boot.Entry:
                JMP             (BIOS.Entry>>4) : .Start ; Go to known location
.Start:
                XOR             AX,AX           ; Initialise the stack
                MOV             SS,AX           ; The BIOS's place is ludicrous
                MOV             ESP,BIOS.Entry  ; So put ESP somewhere sane
; Note we can now use 32-bit addressing with [ESP] to access the stack
</source>
````

<a id="w003495"></a>

## W003495 — User:Johnburger/Demo/Boot/CPU

Bron-ID: W003495
Namespace: User (2)
Revisie: 21395 | Datum: 2017-08-29T09:06:44Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Boot/CPU
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FBoot%2FCPU&oldid=21395

````text
The following code checks to make sure that we're not about to start executing '386-specific instructions on a previous-generation CPU. It uses quirks in the different processors to identify which is actually running.

Note that this code isn't strictly necessary, since there's nothing we can really do if the CPU isn't a '386 or above anyway: a <code>JMP $</code> loop is hardly user friendly! So this will be the first code up against The Wall if I need more room in the Boot sector...

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Boot/Boot|Boot]]/CPU.inc</code> ==
<source lang="asm">
;
; Boot/CPU.inc
;

; Test whether CPU is a '386 (minimum)
Boot.CPU:

                CPU 8086                  ; Assume lowest possible option
; The 8086 pushes a different SP to the others
                PUSH            SP
                POP             AX

                CMP             SP,AX     ; Comparison designed to set Carry too
                JNE             $         ; Uh oh! ***STOP!***

                CPU 286                   ; At least we know it's this good!
; The '286 always clears IOPL
                PUSHF
                POP             AX

                OR              AX,x86.EFlags.IOPL

                PUSH            AX
                POPF

                PUSHF
                POP             CX

                CMP             CX,AX     ; Comparison designed to set Carry too
                JNE             $         ; Uh oh! ***STOP!***

                CPU 386                   ; From now on, we're confident!
</source>
````

<a id="w003496"></a>

## W003496 — User:Johnburger/Demo/Boot/Load

Bron-ID: W003496
Namespace: User (2)
Revisie: 21396 | Datum: 2017-08-29T09:06:57Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Boot/Load
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FBoot%2FLoad&oldid=21396

````text
The code to load the rest of the system uses the BIOS to avoid dealing with the different possible boot media. It either starts loading from where the Master Boot Record indicates (assuming a HardDisk boot), or from the second sector onwards.

It needs to deal with all sorts of intricacies: not the least of which is disk errors! (The code tries a few times before giving up.) But it also has to handle not being able to load all the sectors in one operation: it merely moves the pointers along and tries again.

== Memory Map ==
After this code, the Memory Map will look like this:
{| class="wikitable"
! Address
! Usage
|-
| <code>0000_0000h</code>
| Interrupt Vector Table
|-
| <code>0000_0400h</code>
| BIOS Data Area
|-
| <code>0000_0500h</code>
| Available
|-
| <code>0000_0600h</code>
| Master Boot Record (MBR) <code>*</code>
|-
| <code>0000_0800h</code>
| Available
|-
| <strong><code>0000_0E70h</code></strong>
| <strong>Loaded Initial IDT values</strong>
|-
| <strong><code>0000_1000h</code></strong>
| <strong>Loaded Protected Mode code</strong>
|-
| <strong><code>0000_2070h</code></strong>
| <strong>Available</strong>
|-
| <code>0000_7C00h</code>
| <code>SS:SP</code> Stack Top
|-
| <code>0000_7C00h</code>
| BIOS-loaded Boot Sector
|-
| <code>0000_7E00h</code>
| Available
|-
| <code>0009_F???h</code> <code>**</code>
| Extended BIOS Data Area
|-
| <code>000A_0000h</code>
| Adapter / ROM Area
|-
| <code>000B_8000h</code>
| Text Video Memory
|-
| <code>000F_0000h</code>
| BIOS ROM
|-
| <code>0010_0000h</code>
| High Memory <code>***</code>
|}
<code>*</code> The MBR only exists here if the PC has booted from a Hard Drive.
<br>
<code>**</code> An INT 12h will help identify this value.
<br>
<code>***</code> Note that this memory is only accessible in Protected Mode - except for the first 64 kiB (less 16 bytes), which you can access with a Segment Register set to <code>0FFFFh</code> as long as the A20 Gate is off. By the way: <strong>DON'T DO THIS!</strong>

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Boot/Boot|Boot]]/Load.inc</code> ==
<source lang="asm">
;
; Boot/Load.inc
;

; This module loads the rest of the system - all the Protected Mode stuff.
; It loads it into RAM way down low, leaving the rest of RAM for the system's
; use - and allowing this boot code to be jettisoned.

; The number of Sectors to load. Note that the resultant value may exceed what
; the BIOS can handle: the typical maximum is "number of sectors per track".
; Different floppy formats use different numbers: 8, 9, 15, 18, and 36 are known.
; Since the lowest value is 8, that's the "safest" value. 1.44MB floppies use 18
Load.Sectors    EQU             Load.Size / BIOS.Disk.Sector.Size

; Upon entry, DL is BIOS drive number, ES:SI is Hard Disk Partition entry (if any)
Boot.Load:
                MOV             AX,Load.Base>>4    ; Segment to load to
;               MOV             ES,AX              ; Not yet! Need BIOS value
                XOR             BX,BX              ; Offset to load to

                TEST            SI,SI              ; Sensible value?
                JZ              .Beginning         ; Not really...

; Assume pointing to MBR Table entry
                MOV             CX,[ES:SI+BIOS.MBR.Entry.StartCylSect]
                MOV             DH,[ES:SI+BIOS.MBR.Entry.StartHead]

; Now check if above assumption is true!
                OR              SI,BIOS.Disk.Sector.Size-1; Round UP to BIOS.Sig
                CMP             [ES:SI-1],WORD BIOS.Sig.Value ; (Well, nearly!)
                JE              .Continue          ; All that hard work panned out!

; Assumption NOT true: no help from BIOS! Just load from first drive sector
.Beginning:
                MOV             CX,00_01h          ; Cyl 0, Sect #1
                MOV             DH,0               ; Head Start
.Continue:
                INC             CX                 ; Next sector
;               MOV             DL,                ; DriveNum provided by BIOS
                MOV             SI,3               ; Try this many times
                MOV             ES,AX              ; Can load buffer segment now!

.Attempt:
                MOV             AH,BIOS.Disk.Fn.Read ; Read sectors
                MOV             AL,Load.Sectors    ; Number sectors
.NumToRead      EQU             $-1                ; GASP! Self-modifying code!
.Read:
                INT             BIOS.Disk.Int      ; BIOS Read
                JNC             .ReadSome          ; Carry set on error
                CMP             AL,1               ; Tried single-sector reads?
                JE              .Reset             ; Yep! Better reset drive
                MOV             AL,1               ; No. Try that then.
                JMP             .Read              ; And retry command
.ReadSome:
                SUB             [CS:.NumToRead],AL ; Read enough?
                JZ              .Done              ; Success!

; Note that the next calculation only works for a few number of sectors.
; If a larger number were required, the "Sectors per Track" would overflow,
; and we'd have to instead increase the Head count and zero the Sector count.
; If it overflowed again, we'd have to increase the Cylinder count and zero the
; Head and Sector counts. All too hard if we don't really need to do it,
; especially since determining what constitutes "overflow" is tricky!
; Finally, the "memory address to load to" calculation would overflow, limiting
; the final result to 64K maximum anyway
                ADD             CL,AL              ; Fewer sectors to read...

; Turn the Number Sectors Read value into a pointer offset:
                CBW
                IMUL            AX,BIOS.Disk.Sector.Size
                ADD             BX,AX              ; Further in buffer...
.Reset:
                MOV             AH,BIOS.Disk.Fn.Reset
                INT             BIOS.Disk.Int      ; BIOS Reset
                DEC             SI                 ; One less try...
                JNS             .Attempt           ; (Leaves Z clear on last error!)
                JMP             $                  ; ***STOP!***
.Done:
</source>
````

<a id="w003497"></a>

## W003497 — User:Johnburger/Demo/Boot/IDT

Bron-ID: W003497
Namespace: User (2)
Revisie: 21397 | Datum: 2017-08-29T09:07:08Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Boot/IDT
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FBoot%2FIDT&oldid=21397

````text
The IDT to use in Protected Mode is defined in the source code, and loaded as part of the Protected Mode image. Note, though, that only the first 50 entries are defined, leaving 206 zero. That's a lot of wasted space! So rather than code the full IDT, I only code the first 50, and get the Real Mode code to relocate it to the desired position - and zero out the rest of the Table.

== Memory Map ==
After this code, the Memory Map will look like this:
{| class="wikitable"
! Address
! Usage
|-
| <code>0000_0000h</code>
| Interrupt Vector Table
|-
| <code>0000_0400h</code>
| BIOS Data Area
|-
| <code>0000_0500h</code>
| Available
|-
| <code>0000_0600h</code>
| Master Boot Record (MBR) <code>*</code>
|-
| <strong><code>0000_0800h</code></strong>
| <strong>Moved IDT-to-be</strong>
|-
| <code>0000_1000h</code>
| Loaded Protected Mode code
|-
| <code>0000_2070h</code>
| Available
|-
| <code>0000_7C00h</code>
| <code>SS:SP</code> Stack Top
|-
| <code>0000_7C00h</code>
| BIOS-loaded Boot Sector
|-
| <code>0000_7E00h</code>
| Available
|-
| <code>0009_F???h</code> <code>**</code>
| Extended BIOS Data Area
|-
| <code>000A_0000h</code>
| Adapter / ROM Area
|-
| <code>000B_8000h</code>
| Text Video Memory
|-
| <code>000F_0000h</code>
| BIOS ROM
|-
| <code>0010_0000h</code>
| High Memory <code>***</code>
|}
<code>*</code> The MBR only exists here if the PC has booted from a Hard Drive.
<br>
<code>**</code> An INT 12h will help identify this value.
<br>
<code>***</code> Note that this memory is only accessible in Protected Mode - except for the first 64 kiB (less 16 bytes), which you can access with a Segment Register set to <code>0FFFFh</code> as long as the A20 Gate is off. By the way: <strong>DON'T DO THIS!</strong>

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Boot/Boot|Boot]]/IDT.inc</code> ==
<source lang="asm">
;
; Boot/IDT.inc
;

; This system's IDT is not fully populated - only the first 48 of 256 entries
; are used. Originally, I generated the IDT at run-time, but that used up a LOT
; of the boot sector. Since the IDT is not important until Protected Mode,
; loading it as part of the image makes sense, and is effectively free of
; run-time overhead (we're loading everything else anyway).
;
; But the result was 1,664 bytes of zeroes stored in the disk image. So this is
; the compromise: the used part of the IDT is stored in the image, then this
; code moves the IDT to its final resting home, zeroing out the unused part.

Boot.IDT:
                ; First, turn off hardware interrupts
                MOV             AL,0FFh
                OUT             Dev.PIC.A.Mask,AL
                OUT             Dev.PIC.B.Mask,AL

                MOV             AX,IDT.Start >> 4 ; Start segment
                MOV             DS,AX

                MOV             AX,IDT.Base >> 4 ; Destination segment
                MOV             ES,AX

                CLD                               ; Work forwards

                MOV             CX,(IDT.Limit + 1) / 4 ; Num DWORDs to move
                XOR             SI,SI             ; From the beginning
                XOR             DI,DI             ; Of both segments
                REP             MOVSD             ; Do it

                XOR             EAX,EAX           ; Now zero the rest of it
                MOV             CX,(IDT.Max - (IDT.Limit + 1)) / 4
                REP             STOSD             ; Do it
</source>
````

<a id="w003498"></a>

## W003498 — User:Johnburger/Demo/Boot/RAM

Bron-ID: W003498
Namespace: User (2)
Revisie: 21398 | Datum: 2017-08-29T09:07:19Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Boot/RAM
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FBoot%2FRAM&oldid=21398

````text
Another piece of information needed by the Protected Mode code is just exactly how much RAM the system has - and where. At Boot time, the BIOS knows, so now would be a good time to get it and save it away for future reference.

Of course, as the PC evolved the question "How much memory is there anyway?" got harder and harder to answer, so a number of methods evolved with it. And of course, being implemented by various manufacturers, they each had their own idiosyncrasies...

I use two:
* The first one is the original. It merely gives the top of conventional RAM (less the Extended BIOS Data Area - woohoo! I now have a value for <code>0009_F???h</code> in my Memory Map!).
* The second is the most up-to-date one. It categorises the different blocks of address space into different types (Reserved, Present, Missing, Faulty etc.)

Having gone to all that effort, however, in the rest of the Demonstrator I ignore the values for the RAM Map returned by the BIOS in the second step above. I'll leave it as an exercise for the reader...

== Memory Map ==
After this code, the Memory Map will look like this:
{| class="wikitable"
! Address
! Usage
|-
| <code>0000_0000h</code>
| Interrupt Vector Table
|-
| <code>0000_0400h</code>
| BIOS Data Area
|-
| <strong><code>0000_0500h</code></strong>
| <strong>Protected Mode Data</strong>
|-
| <strong><code>0000_0518h</code></strong>
| <strong>RAM Map from BIOS</strong>
|-
| <code>0000_0800h</code>
| Moved IDT-to-be
|-
| <code>0000_1000h</code>
| Loaded Protected Mode code
|-
| <code>0000_2070h</code>
| Available
|-
| <code>0000_7C00h</code>
| <code>SS:SP</code> Stack Top
|-
| <code>0000_7C00h</code>
| BIOS-loaded Boot Sector
|-
| <code>0000_7E00h</code>
| Available
|-
| <strong><code>0009_F###h</code></strong>
| <strong>Extended BIOS Data Area</strong>
|-
| <code>000A_0000h</code>
| Adapter / ROM Area
|-
| <code>000B_8000h</code>
| Text Video Memory
|-
| <code>000F_0000h</code>
| BIOS ROM
|-
| <code>0010_0000h</code>
| High Memory <code>*</code>
|}
<code>*</code> Note that this memory is only accessible in Protected Mode - except for the first 64 kiB (less 16 bytes), which you can access with a Segment Register set to <code>0FFFFh</code> as long as the A20 Gate is off. By the way: <strong>DON'T DO THIS!</strong>

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Boot/Boot|Boot]]/RAM.inc</code> ==
<source lang="asm">
;
; Boot/RAM.inc
;

; This module uses the BIOS to determine how much RAM the system has.
; It saves the information away for the system to process after it has entered
; Protected Mode - BIOS functions aren't available then.

Boot.RAM:
                XOR             EAX,EAX           ; Zero high part of EAX
                MOV             AX,Data.Base >> 4 ; Point to global Data
                MOV             DS,AX
                MOV             ES,AX             ; With both Segments

                INT             BIOS.RAMTop.Int   ; Get size of Low RAM in kiB
                SHL             EAX,10            ; Turn into number of bytes
                MOV             [Data.RAMTop],EAX ; Top of (Real) RAM

                XOR             EBX,EBX           ; Start with beginning of RAM
                MOV             DI,Data.RAMMap    ; Store records here
.Loop:
                MOV             EAX,BIOS.RAMMap.Fn ; Function call
                MOV             CX,BIOS.RAMMap_size
                MOV             EDX,BIOS.RAMMap.Magic
                MOV             [DI+BIOS.RAMMap.Flags],BYTE BIOS.RAMMap.Flags.DontIgnore
                INT             BIOS.RAMMap.Int ; Call BIOS
                JC              .End            ; Carry? Finished

                CMP             EAX,BIOS.RAMMap.Magic ; BIOS function supported?
                JNE             .End            ; Nope.

                CMP             CL,BIOS.RAMMap.Flags ; Don't mind optional Flags DWORD
                JB              .Skip           ; Ignore if too small
                JE              .Test           ; No .Flags entry, so valid

                TEST            [DI+BIOS.RAMMap.Flags],BYTE BIOS.RAMMap.Flags.DontIgnore
                JZ              .Skip           ; Ignore if told to
.Test:
                MOV             EAX,[DI+BIOS.RAMMap.Length+0]
                OR              EAX,[DI+BIOS.RAMMap.Length+4]
                JZ              .Skip           ; Zero-length entry?

                INC     WORD    [Data.RAMMap.Num]  ; One more Map entry

                ADD             DI,BIOS.RAMMap_size ; Next record
                CMP             DI,Data.Size-BIOS.RAMMap_size ; Any more room?
                JA              .End            ; Nope. Leave
.Skip:
                TEST            EBX,EBX         ; End of sequence?
                JNZ             .Loop           ; Not yet...
.End:
</source>
````

<a id="w003499"></a>

## W003499 — User:Johnburger/Demo/Boot/A20

Bron-ID: W003499
Namespace: User (2)
Revisie: 21399 | Datum: 2017-08-29T09:07:30Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Boot/A20
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FBoot%2FA20&oldid=21399

````text
The 8086 has a 20-bit address bus. If you loaded a segment register with <code>0FFFFh</code> and accessed <code>0FFFFh:0FFFFh</code>, the Segment:Offset addition would wrap and you'd actually be accessing <code>0000_FFEFh</code>. The BIOS would "take advantage" of this quirk and use it to access both top-of-Address ROM and botttom-of-Address RAM with the same segment register.

Of course, then the '286 PC/AT came out with a 24-bit bus - and the wrap no longer occurred. You could now use it to access the (nearly) 64K of High Memory while still in Real Mode! But what about all the legacy code that used the trick? That was now broken.

To "fix" this, the designers of the PC/AT added a Gate to the A20 line, to hold A20 on the memory bus at zero regardless of what the CPU wanted. This emulated the wrap-around "feature", but made the high memory inaccessible.

So to fix '''''that''''', they found a spare output pin that they could program at run-time to turn the Gate on or off. Where was this spare pin? The keyboard controller, of course...

As PCs became more sophisticated, the A20 Gate stayed, but the programming mechanism to turn it on or off changed - and it wasn't really standardised. Since this program wants to access high memory, it has to turn off the Gate - and it uses a number of techniques to attempt this, as recommended by [[A20_Line#Recommended_Method|this page]] in the Wiki.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Boot/Boot|Boot]]/A20.inc</code> ==
<source lang="asm">
;
; Boot/A20.inc
;

; The 8086 has a 20-bit address bus. If you loaded a segment register with
; 0FFFFh and accessed 0FFFFh:0FFFFh, the Segment:Offset addition would wrap
; and you'd actually be accessing 0000_FFEFh. The BIOS would "take advantage"
; of this quirk and use it to access both top-of-Address BIOS and botttom-of-
; Address RAM with the same segment register.
;
; Of course, then the '286 PC/AT came out with a 24-bit bus - and the wrap no
; longer occurred. You could use it to access the (nearly) 64K of High Memory!
; But what about all the legacy code? That was now broken.
;
; To "fix" this, the designers of the PC/AT added a Gate to the A20 line, to
; hold A20 on the memory bus at zero regardless of what the CPU wanted. This
; emulated the wrap-around "feature", but made the high memory inaccessible.
;
; So to fix that, they found a spare output that they could program at run-time
; to turn the Gate on or off. Where was this spare pin? The keyboard controller,
; of course...
;
; As PCs became more sophisticated, the A20 Gate stayed, but the programming
; mechanism to turn it on or off changed - and it wasn't really standardised.
; Since this program wants to access high memory, it has to turn off the Gate -
; and it uses a number of techniques to attempt this.
;
; Of course, this means finding out whether A20 is off, both up front and after
; each technique. How? Go to a known memory location, get its value, and compare
; it with its high-memory "alias". If they're different, then the Gate is off.
; If they're the same, then maybe it was a coincidence? Change the original
; location, and check the alias again. If they're now different (they were the
; same, but now theyre different), the Gate must be off.

A20.Enable EQU             (BIOS.A20.Fn << 8) | BIOS.A20.Enable

; [0000h:BIOS.Entry + BIOS.Sig.Pos] holds the value BIOS.Sig.Value
; [FFFFh:BIOS.Entry + BIOS.Sig.Pos + 10h] is its alias, if the A20 Gate is on.
Sig.Alias  EQU             BIOS.Entry + BIOS.Sig.Pos + 10h

Boot.A20:
; CS:BIOS.Sig.Pos has a known, safely-modifable value.
; Prepare for testing, by comparing CS:BIOS.Sig.Pos with 0FFFFh:Sig.Alias
                MOV             SI,BIOS.Sig.Pos

                MOV             AX,0FFFFh       ; End of 1 MB
                MOV             ES,AX           ; In ES
                MOV             DI,Sig.Alias

; First things first: has it already been done for us?
                CALL            .Test           ; Test if A20 already on
                JE              .End            ; Yep! Nothing to do

; No. OK, maybe the BIOS can help.
                MOV             AX,A20.Enable
                INT             BIOS.A20.Int    ; Using BIOS

                CALL            .Test           ; Did it work?
                JE              .End            ; Yep! Nothing more to do

; No. OK: the traditional ways are always the best?
                CALL            Boot.A20.Switch ; Switch A20 myself
;               JNZ             $               ; Uh oh! ***STOP!***

                CALL            .Test           ; Did it work?
                JE              .End            ; Yep! Nothing more to do

; No. Maybe it's a PS/2?
                IN              AL,Dev.A20.Fast.Port ; Try Fast A20
                TEST            AL,Dev.A20.Fast.A20 ; Enabled?
                JNZ             $               ; Uh oh! Isn't working! ***STOP!***
                OR              AL,Dev.A20.Fast.A20    ; Enable it
                AND             AL,~Dev.A20.Fast.Reset ; But NOT Fast Reset!
                OUT             Dev.A20.Fast.Port,AL

                CALL            .Test           ; Did it work?
                JE              .End            ; Yep! Finally!

                JMP             $               ; No reaction! ***STOP!***
Boot.A20.Test:
                MOV             DX,[CS:SI]      ; Get current value
                CMP             DX,[ES:DI]      ; Same at end of RAM?
                JNE             .Good           ; No, so no problem!

                NOT     WORD    [CS:SI]         ; Change current value
                CMP             DX,[ES:DI]      ; Still same as old?
                JMP             .Bad            ; No, so problem!
.Good:
                XOR             AX,AX           ; Set Z flag
.Bad:
                RET

;-------------------------------------------------------------------------------

Boot.A20.Switch:
                CALL            .Wait          ; Wait for 8042 to be ready
                JNZ             .Dead          ; Uh oh...

                MOV             AL,Dev.A20.Ctrl.Write
                OUT             Dev.A20.Ctrl.Port,AL ; Tell 8042 we're writing to it
                CALL            .Wait          ; Wait for acknowledge
                JNZ             .Dead          ; Uh oh...

                MOV             AL,Dev.A20.Data.Enable
                OUT             Dev.A20.Data.Port,AL ; Tell 8042 to Enable A20
;...............................................................................
.Wait:
                XOR             CX,CX           ; Try a LARGE number of times!
.Loop:
                IN              AL,Dev.A20.Stat.Port ; Wait to write
                TEST            AL,Dev.A20.Stat.InFull ; YES, In! It's backwards
                LOOPNZ          .Loop           ; Nope. Still full
.Dead:
                RET

;-------------------------------------------------------------------------------

Boot.A20.End:
</source>
````

<a id="w003500"></a>

## W003500 — User:Johnburger/Demo/Boot/Key

Bron-ID: W003500
Namespace: User (2)
Revisie: 21400 | Datum: 2017-08-29T09:07:41Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Boot/Key
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FBoot%2FKey&oldid=21400

````text
The keyboard doesn't need much setting up - the BIOS defaults are pretty good to my way of thinking. On real hardware, it would be nice to speed up the keyboard's repeat rate to be more responsive; and there's a BIOS function for that. Better call it before leaving Real Mode...

Unfortunately, this code doesn't work well under VMware. To the end I've commented it out.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Boot/Boot|Boot]]/Key.inc</code> ==
<source lang="asm">
;
; Boot/Key.inc
;

; Use the BIOS to speed up the keyboard to as fast as it will go

Key.SetRate     EQU             (BIOS.Key.SetRate.Fn << 8) | BIOS.Key.SetRate.Set
Key.Rate        EQU             (BIOS.Key.SetRate.Delay.250 << 8) | BIOS.Key.SetRate.Rate.300

Boot.Key:
; The following doesn't work as intended under VMWare
;               MOV             AX,Key.SetRate
;               MOV             BX,Key.Rate
;               INT             BIOS.Key.Int
</source>
````

<a id="w003501"></a>

## W003501 — User:Johnburger/Demo/Boot/VGA

Bron-ID: W003501
Namespace: User (2)
Revisie: 21401 | Datum: 2017-08-29T09:07:59Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Boot/VGA
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FBoot%2FVGA&oldid=21401

````text
With the vast number of video adapters on the market, all with their proprietary (and often secret!) access methods, video drivers are the biggest problem for PC system developers. There are certain industry-standard modes and conventions, but the really clever stuff needs drivers provided by the video card manufacturers.

Before leaving Real Mode, now would be the perfect time to set the desired Video Mode using the BIOS. All VGAs support 80x50 mode, so that will be the lowest common denominator for this Demonstrator. Some systems support 90x60 too - VMware is a perfect example! That gives me more experimenting room; but I need a way to disable it in the source code. The easiest is to use [[User:Johnburger/Demo/Demo.inc#Demo.inc|<code>%define BIOS.VGA.90x60</code>]].

Note though that the 90x60 code takes up a fair bit of space. That will probably be the [[User:Johnburger/Demo/Boot/CPU|second code against The Wall]] if I run out of room...

Oh, and set the cursor to invisible too.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Boot/Boot|Boot]]/VGA.inc</code> ==
<source lang="asm">
;
; Boot/VGA.inc
;

; Use the BIOS to set the desired video mode (80x50 by default: 90x60 if enabled)

VGA.Set80x25    EQU             (BIOS.VGA.SetMode.Fn << 8) | BIOS.VGA.SetMode.80x25
VGA.Ext80x50    EQU             (BIOS.VGA.ExtMode.Fn << 8) | BIOS.VGA.ExtMode.80x50

; Note: DS and ES are 0000h
Boot.VGA:
                MOV             AX,VGA.Set80x25 ; Set 80x25 mode, 8x16 font
                INT             BIOS.VGA.Int

                MOV             AX,VGA.Ext80x50 ; Ext 80x50 mode, 8x8 font
                MOV             BL,0
                INT             BIOS.VGA.Int

%ifndef BIOS.VGA.90x60

VGA.Rows        EQU             50
VGA.Cols        EQU             80

                MOV             AH,BIOS.VGA.Cursor.Fn   ; Set cursor size
                MOV             CX,BIOS.VGA.Cursor.None ; To invisible
                INT             BIOS.VGA.Int

%else ; BIOS.VGA.90x60

VGA.Rows        EQU             60
VGA.Cols        EQU             90

;-------------------------------------------------------------------------------

; I know I said I don't like "magic numbers", but the following is esoteric!
                MOV             AL,0E7h
                MOV             DX,Dev.VGA.Misc.Port
                OUT             DX,AL

                CLD                             ; Work forwards
                MOV             SI,Boot.VGA.Seq

                MOV             CX,.Seq.Num     ; This many Seq registers
                MOV             DX,Dev.VGA.Seq.Base
                CALL            Boot.VGA.Write

                MOV             AL,017h         ; Enable regs 0-7
                MOV             DX,Dev.VGA.CRTC.Base
                OUT             DX,AL
                INC             DX              ; Go to Data port
                IN              AL,DX
                AND             AL,~80h         ; Enable registers
                OUT             DX,AL
                DEC             DX              ; Back to Index port

                MOV             AL,003h         ; Enable vertical retrace
                OUT             DX,AL
                INC             DX              ; Go to Data port
                IN              AL,DX
                OR              AL,80h          ; Enable registers
                OUT             DX,AL
                DEC             DX              ; Back to Index port

                MOV             CX,.CRTC.Num    ; This many CRTC registers
                PUSH            Boot.VGA.End    ; Fake a call from .End
;-------------------------------------------------------------------------------

.Write:
                MOV             AH,0            ; Start with register 0
.Write.Loop:
                CS                              ; Load from Code Segment!
                LODSB
                XCHG            AL,AH           ; Make sure Index is in AL
                OUT             DX,AX           ; 16-bit write
                MOV             AH,AL           ; But need AL for LODSB
                INC             AH              ; Next index
                LOOP            .Write.Loop
                RET

.Seq            DB              003h, 001h, 003h, 000h, 002h ; Sequence registers
.Seq.Num        EQU             $-.Seq
.CRTC           DB              06Bh, 059h, 05Ah, 082h, 060h, 08Dh, 00Bh, 03Eh
                DB              000h, 047h, 020h, 000h, 000h, 000h, 000h, 000h
                DB              0EAh, 00Ch, 0DFh, 02Dh, 008h, 0E8h, 005h, 0A3h
                DB              0FFh                         ; CRTC registers
.CRTC.Num       EQU             $-.CRTC

%endif ; BIOS.VGA.90x60

.End:
</source>
````

<a id="w003502"></a>

## W003502 — User:Johnburger/Demo/Boot/PM

Bron-ID: W003502
Namespace: User (2)
Revisie: 28460 | Datum: 2023-11-15T08:33:00Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Boot/PM
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FBoot%2FPM&oldid=28460

````text
Well, it's been a long road. From the original BIOS Boot Sector load, a lot of work has been done to get things ready for Protected Mode. There's nothing left for it: time to load some registers and leap into the abyss!

The two most important registers that need to be set are the Global Descriptor Table Register (GDTR) and Control Register 0 ([[User:Johnburger/Demo/x86/CR0|CR0]]). The former defines all the Global Descriptors, while the latter has the bit that sets the CPU into Protected Mode. But in my experience a third register needs to be set before entering Protected Mode: the Interrupt Descriptor Table Register (IDTR). That will ensure that there are immediate handlers for any fault that might occur during the transition.

Once in Protected Mode I made it difficult for myself because I didn't want to "clutter" the GDT with bootstrap code. So I made the Executive (the next port of call) have its own LDT, and put its code there. That means that as soon as the code enters Protected Mode, it needs to load the Local Descriptor Table Register (LDTR) with the correct LDT selector using the <code>LLDT</code> instruction. As soon as that happens though, it can <code>JMP</code> to the new code.

And by <code>JMP</code>ing, we say goodbye to this Real Mode code. It has served its purpose and no longer has any use. It also happens to be laying slap bang in the middle of the (now) GDT - it needs to be jettisoned and (ultimately) overwritten.

== Memory Map ==
After this code, the Memory Map will look like this:
{| class="wikitable"
! Address
! Usage
|-
| <code>0000_0000h</code>
| Interrupt Vector Table
|-
| <code>0000_0400h</code>
| BIOS Data Area
|-
| <code>0000_0500h</code>
| Protected Mode Data
|-
| <code>0000_0518h</code>
| RAM Map from BIOS
|-
| <strong><code>0000_0800h</code></strong>
| <strong>Interrupt Descriptor Table</strong>
|-
| <code>0000_1000h</code>
| Loaded Protected Mode code
|-
| <strong><code>0000_1D70h</code></strong>
| <strong>Local Descriptor Table</strong>
|-
| <strong><code>0000_1DE0h</code></strong>
| <strong>Global Descriptor Table</strong>
|-
| <strong><code>0001_2E70h</code></strong>
| <strong><code>SS:SP</code>-to-be Stack Top</strong>
|-
| <strong><code>0001_2E70h</code></strong>
| <strong>Available</strong>
|-
| <strong><code>0009_F###h</code></strong>
| <strong>Extended BIOS Data Area</strong>
|-
| <code>000A_0000h</code>
| Adapter / ROM Area
|-
| <code>000B_8000h</code>
| Text Video Memory
|-
| <code>000F_0000h</code>
| BIOS ROM
|-
| <code>0010_0000h</code>
| High Memory
|}

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Boot/Boot|Boot]]/PM.inc</code> ==
<source lang="asm">
;
; Boot/PM.inc
;

; This code does the final preparations before launching into Protected Mode.
; It loads GDTR, IDTR, switches to Protected Mode, loads LDTR then JMPs to the
; Executive.

Boot.LDT.Code   EQU             Selector(Exec.LDT.Code, LDT, RPL0)

Boot.PM:
                MOV             AX,GDT.Base >> 4 ; Point to GDT
                MOV             DS,AX
                LGDT            [GDT.Alloc.Pseudo]

                PUSH   DWORD    IDT.Base    ; Put on stack as Pseudo-descriptor
                PUSH    WORD    IDT.Limit   ; Only this many entries in IDT
                LIDT            [ESP]
; We're about to abandon the stack, so don't bother fixing it up!

; Can only load the LDTR after we've entered Protected Mode: this is the value
                MOV             BX,Selector(GDT.Exec.LDT, GDT, RPL0)

; Turn on Protected Mode (cross your fingers!)
                CLI                             ; Disable interrupts!
                MOV             EAX,CR0
                OR              AL,x86.CR0.PE
                MOV             CR0,EAX

; First, load the LDT
                LLDT            BX
; Then, launch code
                JMP             Boot.LDT.Code : DWORD Exec.Code

; This code is now unneeded. It can be overwritten by whatever the system wants
</source>
````

<a id="w003503"></a>

## W003503 — ARM Integrator-CP ITPTMME Phase2

Bron-ID: W003503
Namespace: Main (0)
Revisie: 28043 | Datum: 2023-07-09T19:00:12Z
Bron: https://wiki.osdev.org/ARM_Integrator-CP_ITPTMME_Phase2
Revisie-URL: https://wiki.osdev.org/index.php?title=ARM_Integrator-CP_ITPTMME_Phase2&oldid=28043

````text
{{FirstPerson}}
== Processes & Threads, Exception Handling, And System Calls ==
In this phase we are going to improve our exception handling, rework out tasking system to support processes and threads (instead of tasks), and implement some basic system calls for demonstration.

I make a few minor changes and one or two major changes. You can find the full source with:
<pre>
    git clone http://kmcg3413.net/armthin.git
    git checkout origin/bPHASE1
</pre>

This should give you the source with all the changes below. If you have been following along from the previous pages then you will only have to cover these simple changes. If you are new then I recommend you start from [[ARM_Integrator-CP_IRQTimerAndPIC|IRQ, Timer, And PIC]].

=== New Define For Timer Support ===
<source lang="c">
/* max number of ticks for a task */
#define KTASKTICKS 10000
</source>
This sets the time in which threads are interrupted for a context switch. The lower the
higher the frequency or maximum time a thread can execute.

=== Basic Linked List Utility Functions ===
I like these functions because they can work with ''any'' data structure that has the fields
at the top. It can be cast to type ''LL'' and used that way.
<source lang="c"> 
typedef struct _LL {
	struct _LL			*next;
	struct _LL			*prev;
} LL;

void ll_add(void **p, void *i) {
	LL		*_i;
	
	_i = (LL*)i;
	
	_i->next = *p;
	if (*p) {
		_i->prev = ((LL*)(*p))->prev;
	} else {
		_i->prev = 0;
	}
	
	if (p) {
		*p = _i;
	}
}

void ll_rem(void **p, void *i) {
	LL			*_i;
	
	_i = (LL*)i;

	if (_i->prev) {
		_i->prev->next = _i->next;
	}
	
	if (_i->next) {
		_i->next->prev = _i->prev;
	}
	
	if (p) {
		if (*p == i) {
			if (_i->prev) {
				*p = _i->prev;
			} else {
				*p = _i->next;
			}
		}
	}
}
</source>
=== Separate Scheduler Function ===
I have added support for sleeping threads, and a wake up flag. This is by no means a correct design, but rather just a demonstration of how you could put threads to sleep and wake them up. It also supports a timeout which allows a thread to not only wake up with an external signal but also after a specified amount of time.
<source lang="c">
void ksched() {
	KSTATE			*ks;
	KTHREAD			*kt;
	uint32			__lr, __sp, __spsr;
	uintptr			page;
	
	ks = (KSTATE*)KSTATEADDR;

	/* if valid process and thread then store */
	if (ks->cproc && ks->cthread) {
		/*
			1. store register on stack in thread struct
			2. access hidden registers and store in thread struct
		*/
		kt = ks->cthread;
		kt->pc = ((uint32*)KSTACKEXC)[-1];
		kt->r12 = ((uint32*)KSTACKEXC)[-2];
		kt->r11 = ((uint32*)KSTACKEXC)[-3];
		kt->r10 = ((uint32*)KSTACKEXC)[-4];
		kt->r9 = ((uint32*)KSTACKEXC)[-5];
		kt->r8 = ((uint32*)KSTACKEXC)[-6];
		kt->r7 = ((uint32*)KSTACKEXC)[-7];
		kt->r6 = ((uint32*)KSTACKEXC)[-8];
		kt->r5 = ((uint32*)KSTACKEXC)[-9];
		kt->r4 = ((uint32*)KSTACKEXC)[-10];
		kt->r3 = ((uint32*)KSTACKEXC)[-11];
		kt->r2 = ((uint32*)KSTACKEXC)[-12];
		kt->r1 = ((uint32*)KSTACKEXC)[-13];
		kt->r0 = ((uint32*)KSTACKEXC)[-14];
		kt->cpsr = ((uint32*)KSTACKEXC)[-15];
		/* switch to system mode get hidden registers then switch back */
		asm("mrs r0, cpsr \n\
			 mov r1, r0 \n\
			 bic r0, r0, #0x1f \n\
			 orr r0, r0, #0x1f \n\
			 msr cpsr, r0 \n\
			 mov %[sp], sp \n\
			 mov %[lr], lr \n\
			 msr cpsr, r1 \n\
			 " : [sp]"=r" (__sp), [lr]"=r" (__lr));
		kt->sp = __sp;
		kt->lr = __lr;
	}

	if (!ks->cproc) {
		/* initial start */
		ks->cproc = ks->procs;
		ks->cthread = ks->procs->threads;
	}
	
	while (1) {
		if (ks->cthread) {
			/* get next thread */
			ks->cthread = ks->cthread->next;
		}
		
		/* if none get next process */
		if (!ks->cthread) {
			/* get next process */
			if (ks->cproc) {
				ks->cproc = ks->cproc->next;
			}
			/* if none get first process */
			if (!ks->cproc) {
				ks->cproc = ks->procs;
			}
			/* get first thread */
			if (ks->cproc) {
				ks->cthread = ks->cproc->threads;
			} else {
				ks->cthread = 0;
			}
		}
		
		if (!ks->cthread) {
			break;
		}
		
		/* if current thread is sleeping and current thread equals last thread */
		if ((ks->cthread->flags & KTHREAD_SLEEPING) && (ks->cthread == kt)) {
			PANIC("all-threads-sleeping");
		}
		
		/* only wakeup if it is sleeping */
		if (ks->cthread->flags & KTHREAD_SLEEPING) {
			if (ks->cthread->timeout > 0 && ks->ctime > ks->cthread->timeout) {
				//kprintf("WOKE UP (timeout) %x\n", ks->cthread);
				/* wake up thread if passed timeout */
				ks->cthread->flags &= ~KTHREAD_SLEEPING;
				ks->cthread->r0 = 0;
				break;
			}
			
			/* wakeup thread is set to be woken up */
			if (ks->cthread->flags & KTHREAD_WAKEUP) {
				//kprintf("WOKE UP (signal) %x\n", ks->cthread);
				ks->cthread->flags &= ~(KTHREAD_WAKEUP | KTHREAD_SLEEPING);
				ks->cthread->r0 = ks->ctime - ks->cthread->timeout;
				break;
			}
		} else {
			/* run this thread */
			break;
		}
		/* thread is sleeping or not able to run */
	}
	
	/* hopefully we got something or the system should deadlock */
	kt = ks->cthread;
	/*
		load registers
	*/
	((uint32*)KSTACKEXC)[-1] = kt->pc;
	((uint32*)KSTACKEXC)[-2] = kt->r12;
	((uint32*)KSTACKEXC)[-3] = kt->r11;
	((uint32*)KSTACKEXC)[-4] = kt->r10;
	((uint32*)KSTACKEXC)[-5] = kt->r9;
	((uint32*)KSTACKEXC)[-6] = kt->r8;
	((uint32*)KSTACKEXC)[-7] = kt->r7;
	((uint32*)KSTACKEXC)[-8] = kt->r6;
	((uint32*)KSTACKEXC)[-9] = kt->r5;
	((uint32*)KSTACKEXC)[-10] = kt->r4;
	((uint32*)KSTACKEXC)[-11] = kt->r3;
	((uint32*)KSTACKEXC)[-12] = kt->r2;
	((uint32*)KSTACKEXC)[-13] = kt->r1;
	((uint32*)KSTACKEXC)[-14] = kt->r0;
	((uint32*)KSTACKEXC)[-15] = kt->cpsr;
	/* switch into system mode restore hidden registers then switch back */
	asm("mrs r0, cpsr \n\
		 mov r1, r0 \n\
		 bic r0, r0, #0x1f \n\
		 orr r0, r0, #0x1f \n\
		 msr cpsr, r0 \n\
		 mov sp, %[sp] \n\
		 mov lr, %[lr] \n\
		 msr cpsr, r1 \n\
		 " : : [sp]"r" (kt->sp), [lr]"r" (kt->lr));
	/* set TLB table for user space (it can be zero for kernel) */
	kvmm2_getphy(&ks->vmm, (uintptr)ks->cproc->vmm.table, &page);
	arm4_tlbset1(page);
	/* 
		Invalidate all unlocked entries...
		
		..according to the manual there may be a better way to invalidate,
		only some entries per process. But, for now this should work.
		
		If you do not do this then the TLB does not flush and old entries
		from the previous process will still be in the TLB cache.
	*/
	asm("mcr p15, #0, r0, c8, c7, #0");
	
	//kprintf("SWITCH-TO thread:%x cpsr:%x fp:%x sp:%x pc:%x\n", kt, kt->cpsr, kt->r11, kt->sp, kt->pc);
	
	uint32			*p;
	
	if (!kvmm2_getphy(&ks->cproc->vmm, 0x90000000, (uintptr*)&p)) {
		//kprintf("NO STACK EXISTS??\n");
	} else {
		//kprintf("writing to stack..%x\n", kt->sp);
		//((uint32*)kt->sp)[-1] = 0xbb;
	}
	
	if (kvmm2_getphy(&ks->cproc->vmm, 0x80000000, (uintptr*)&p)) {
		uint32			x;
		
		//kprintf("CODE PAGE :%x\n", p);
		
		p = (uint32*)(0x80000000);

		//((uint32*)KSTACKEXC)[-1] = 0x80000800;
	
		//for (x = 0; x < 1024; ++x) {
		//	p[x] = 0xeafffffe;
		//}
	
	} else {
		//kprintf("CODE PAGE????\n");
	}
}
</source>
Here we have the basic save thread state and load thread state blocks. These possibly could be implemented in a much faster way, but I choose to avoid premature optimization for the sake of being straight forward. In the middle between the save and load state blocks you have the code to choose the next thread. In it's simplest form it simply grabs the next thread in the current process, and if no thread left it grabs the next process and the first thread. It continues this until it has a thread to run. In the form above it does the exact same except it checks if the thread is sleeping, then it checks the ''timeout'' (minimum time to sleep for) and if it is expired it sets the thread as awake and runs it. It also checks if any signal has been asserted to the thread and if so (and only if) then the thread is woken if it is sleeping. You might be wondering why I remove the wake signal (bit) only if the thread is sleeping. This has to do with my future design of the IPC for the system.


=== Added Idle Thread And Kernel Thread ===
I added an ''idle'' thread that does nothing but yield, and a kernel work thread. The idle thread keeps the scheduler with something to switch too when no other threads are running (supposed to be), but in reality in the current code it just switches to it and it yields. Needs some improvement, but it is simple for now.
<source lang="c">
int ksleep(uint32 timeout) {
	int			result;
	
	asm("	mov r0, %[in] \n\
			swi #101 \n\
			mov %[result], r0 \n\
		" : [result]"=r" (result) : [in]"r" (timeout));
	/* convert from ticks */
	return result;
}

void kthread(KTHREAD *th) {
	uint32			x;
	
	for (;;) {
		ksleep(0xffff);
		kserdbg_putc('$');
	}
}

void kidle() {
	for (;;) {
		asm("swi #102");
	}
}
The ''kidle'' immediantly yields. The scheduler could be coded differently where it only executes the ''kidle'' if there are no other threads to run, but I decided to keep it simple at this stage.
</source>

=== Creation Of Kernel And Idle Thread ===
<source lang="c">
        .... in main ....
	process = (KPROCESS*)kmalloc(sizeof(KPROCESS));
	memset(process, 0, sizeof(KPROCESS));
	kvmm2_init(&process->vmm);
	ll_add((void**)&ks->procs, process);
	
	th = (KTHREAD*)kmalloc(sizeof(KTHREAD));
	memset(th, 0, sizeof(KTHREAD));
	ll_add((void**)&process->threads, th);
	th->pc = (uintptr)&kthread;
	th->flags = 0;
	th->cpsr = 0x60000000 | ARM4_MODE_SYS;
	/* set stack */
	th->sp = (uintptr)kmalloc(1024 * 2) + 1024 * 2 - 8;
	th->r0 = (uint32)th;

	th = (KTHREAD*)kmalloc(sizeof(KTHREAD));
	memset(th, 0, sizeof(KTHREAD));
	ll_add((void**)&process->threads, th);
	th->pc = (uintptr)&kidle;
	th->flags = KTHREAD_KIDLE;
	th->cpsr = 0x60000000 | ARM4_MODE_SYS;
	/* set stack (dont need anything big for idle thread at the moment) */
	th->sp = (uintptr)kmalloc(128) + 128 - 8;
	th->r0 = (uint32)th;
	ks->idleth = th;
	ks->idleproc = process;
        ....
</source>
Here both the kernel thread and ''idle'' thread are created under the same process. You could have created a separate process if you like.

=== New Exception Handlers ===
<source lang="c">
void k_exphandler(uint32 lr, uint32 type) {
	uint32			*t0mmio;
	uint32			*picmmio;
	uint32			swi;
	KSTATE			*ks;
	int				x;
	KTHREAD			*kt;
	uintptr			out;
	uint32			r0, r1;
	KPROCESS		*proc;
	KTHREAD			*th;
	
	ks = (KSTATE*)KSTATEADDR;
		
	//kserdbg_putc('H');
	//kserdbg_putc('\n');
	
	/*  clear interrupt in timer so it will lower its INT line
	
		if you do not clear it, an interrupt will
		be immediantly raised apon return from this
		interrupt
	*/
	
	if (type == ARM4_XRQ_IRQ) {
		picmmio = (uint32*)0x14000000;
		
		//kprintf("picmmio[PIC_IRQ_STATUS]:%x\n", picmmio[PIC_IRQ_STATUS]);
		/*
			It is possible that other pins are activated so we just check
			this one bit.
			
			1 << 4
		*/
		if (picmmio[PIC_IRQ_STATUS] & (1<<6)) {
			t0mmio = (uint32*)0x13000100;
			t0mmio[REG_INTCLR] = 1;			/* according to the docs u can write any value */
			
			//kprintf("t0mmio[REG_BGLOAD]:%x ks->ctime:%x\n", t0mmio[REG_BGLOAD], ks->ctime);
			ks->ctime += t0mmio[REG_BGLOAD];
			
			ksched();
			//kprintf("time:%x\n", t0mmio[REG_VALUE]);
			/* go back through normal interrupt return process */
			return;
		}
	}
	
	/*
		Get SWI argument (index).
	*/
	if (type == ARM4_XRQ_SWINT) {
		swi = ((uint32*)((uintptr)lr - 4))[0] & 0xffff;
		
		//kprintf("SWI thread:%x code:%x\n", ks->cthread, swi);
		
		//((uint32*)KSTACKEXC)[-14] = R0;
		//((uint32*)KSTACKEXC)[-13] = R1;
		
		switch (swi) {
			case KSWI_WAKEUP:
				/* wake up thread function */
				r0 = ((uint32*)KSTACKEXC)[-14];
				r1 = ((uint32*)KSTACKEXC)[-13];
				for (proc = ks->procs; proc; proc = proc->next) {
					if ((uint32)proc == r0) {
						for (th = proc->threads; th; th = th->next) {
							if ((uint32)th == r1) {
								/* wake up thread */
								th->flags |= KTHREAD_WAKEUP;
							}
						}
					}
				}
				break;
			case KSWI_GETTICKPERSECOND:
				((uint32*)KSTACKEXC)[-14] = ks->tpers;
				break;
			case KSWI_SLEEP:
				/* thread sleep function */
				r0 = ((uint32*)KSTACKEXC)[-14];
				
				//kprintf("SLEEPING thread:%x timeout:%x\n", ks->cthread, r0);

				if (ks->cthread) {
					ks->cthread->flags |= KTHREAD_SLEEPING;
					ks->cthread->timeout = r0 + ks->ctime;
				}
				ksched();
				break;
			case KSWI_YEILD:
				ksched();
				break;
			default:
				break;
		}
		return;
	}
	
	if (type != ARM4_XRQ_IRQ && type != ARM4_XRQ_FIQ && type != ARM4_XRQ_SWINT) {
		/*
			Ensure, the exception return code is correctly handling LR with the
			correct offset. I am using the same return for everything except SWI, 
			which requires that LR not be offset before return.
		*/
		KTHREAD			*tmp;
		
		tmp = ks->cthread;
		
		kprintf("!EXCEPTION\n");
		kprintf("type:%x cproc:%x cthread:%x lr:%x\n", type, ks->cproc, ks->cthread, lr);

		ll_rem((void**)&ks->cproc->threads, ks->cthread);
		ks->cthread = ks->cthread->next;
		
		ksched();
		
		kdumpthreadinfo(tmp);
	}
	
	return;
}
</source>
The biggest difference here is the addition of systems call and the termination of a thread when it has an exception. 


=== Added memset ===
I added a useful utility function ''memset''.
<source lang="c">
void memset(void *p, uint8 v, uintptr sz) {
	uint8			*_p;
	uintptr			x;
	
	_p = (uint8*)p;
	
	for (x = 0; x < sz; ++x) {
		_p[x] = v;
	}
}
</source>

=== Changes In kelfload ===
<source lang="c">
int kelfload(KPROCESS *proc, uintptr addr, uintptr sz) {
	ELF32_EHDR			*ehdr;
	ELF32_SHDR			*shdr;
	uint32				x, y;
	uintptr				page, oldpage;
	KSTATE				*ks;
	uint8				*fb;
	KTHREAD				*th;
	
	kprintf("loading elf into memory space\n");
	
	ks = (KSTATE*)KSTATEADDR;
		
	ehdr = (ELF32_EHDR*)addr;
	if (ehdr->e_machine != EM_ARM) {
		kprintf("kelfload: not ARM machine!\n");
		return 0;
	}
	
	if (ehdr->e_ident[4] != 0x1) {
		kprintf("kelfload: not ELF32 object\n");
		return 0;
	}
	
	if (!proc->vmm.table) {
		kvmm2_init(&proc->vmm);
	}

	th = (KTHREAD*)kmalloc(sizeof(KTHREAD));
	memset(th, 0, sizeof(KTHREAD));
	
	ll_add((void**)&proc->threads, th);

	th->pc = ehdr->e_entry;
	th->flags = 0;
	th->cpsr = 0x60000000 | ARM4_MODE_USER;
	/* set stack */
	th->sp = 0x90001000;
	/* pass address of serial output as first argument */
	th->r0 = 0xa0000000;
	/* map serial output mmio */
	kvmm2_mapsingle(&proc->vmm, 0xa0000000, 0x16000000, TLB_C_AP_FULLACCESS);
	/* map stack page (4K) */
	kvmm2_allocregionat(&proc->vmm, 1, 0x90000000, TLB_C_AP_FULLACCESS);

	/* map address space so we can work directly with it */
	kvmm2_getphy(&ks->vmm, (uintptr)proc->vmm.table, &page);
	oldpage = arm4_tlbget1();
	arm4_tlbset1(page);
        /* flush the TLB */
	asm("mcr p15, #0, r0, c8, c7, #0");

	// e_shoff - section table offset
	// e_shentsize - size of each section entry
	// e_shnum - count of entries in table
	for (x = 0; x < ehdr->e_shnum; ++x) {
		shdr = (ELF32_SHDR*)(addr + ehdr->e_shoff + x * ehdr->e_shentsize);
		if (shdr->sh_addr != 0) {
			/* load this into memory */
			// sh_offset - byte offset in module
			// sh_size - size of section in module 
			// sh_addr - address to load at
			kvmm2_allocregionat(&proc->vmm, kvmm2_rndup(shdr->sh_size), shdr->sh_addr, TLB_C_AP_FULLACCESS);
			fb = (uint8*)(addr + shdr->sh_offset);
			/* copy */
			for (y = 0; y < shdr->sh_size; ++y) {
				((uint8*)shdr->sh_addr)[y] = fb[y];
			}
		}
	}
	
	/* restore previous address space */
 	arm4_tlbset1(oldpage);
        /* flush the TLB */
        asm("mcr p15, #0, r0, c8, c7, #0");
}
</source>
Here we simple allocate memory for the sections and then copy them into memory. I switch address spaces and flush the TLB to make this easier to perform. It could be faster just to map the memory into kernel space and copy to it in certain situations, but I decided that this method was the simplest. 

=== Changed To Creation Of Process From Modules ===
<source lang="c">
        .... 
	#define KMODTYPE_ELFUSER			1
	/*
		create a task for any attached modules of the correct type
	*/
	kprintf("looking at attached modules\n");
	for (m = kPkgGetFirstMod(); m; m = kPkgGetNextMod(m)) {
		kprintf("looking at module\n");
		if (m->type == KMODTYPE_ELFUSER) {
			/* create new process */
			process = (KPROCESS*)kmalloc(sizeof(KPROCESS));
			memset(process, 0, sizeof(KPROCESS));
			ll_add((void**)&ks->procs, process);
			/* will create thread in process */
			kelfload(process, (uintptr)&m->slot[0], m->size);
		}
	}
        .....
</source>
I simply changed from creating of a task to the creation of a process, and ''kelfload'' was changed to create a thread of the passed process instead of a task.

=== SWI Exception Entry And Exit Changed ===
These were changed to push ''SPSR'' so it can be saved and restored easily from inside
our new ''sched()'' function.
<source lang="c">
#define KEXP_TOPSWI \
	uint32			lr; \
	asm("mov sp, %[ps]" : : [ps]"i" (KSTACKEXC)); \
	asm("push {lr}"); \
	asm("push {r0,r1,r2,r3,r4,r5,r6,r7,r8,r9,r10,r11,r12}"); \
	asm("mrs r0, spsr"); \
	asm("push {r0}"); \
	asm("mov %[ps], lr" : [ps]"=r" (lr));	

#define KEXP_BOTSWI \
	asm("pop {r0}"); \
	asm("msr spsr, r0"); \
	asm("pop {r0,r1,r2,r3,r4,r5,r6,r7,r8,r9,r10,r11,r12}"); \
	asm("LDM sp!, {pc}^")
</source>
The ''SWI'' entry and exit assembly now saves everything the same way as the other exception handlers allowing the scheduler to be called to save the current thread and load the next meaning system calls can switch threads for ''sleep'' and ''yield''.

=== Added System Call Numbers ===
<source lang="c">
#define KSWI_WAKEUP				100
#define KSWI_SLEEP				101
#define KSWI_YEILD				102
#define KSWI_GETTICKPERSECOND	103
</source>

=== New KPROCESS And Updated KTHREAD ===
<source lang="c">
typedef struct _KTHREAD {
	struct _KTHREAD		*next;
	struct _KTHREAD		*prev;
	
	uint64				timeout;			/* when to wakeup */
	uint8				flags;
	uint32				r0, r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12, sp, lr, cpsr, pc;
} KTHREAD;

typedef struct _KPROCESS {
	struct _KPROCESS	*next;
	struct _KPROCESS	*prev;
	
	KVMMTABLE			vmm;
	KTHREAD				*threads;
} KPROCESS;
</source>
The ''KTHREAD'' was updated with linked list fields added, ''timeout'', and ''flags''. The ''KPROCESS'' is new and was added to provide a process like structure to tasking. The ''vmm'' field was moved from the thread structure to the process structure since all threads share the same
address space.
 
=== Changes To KSTATE ===
<source lang="c">
 typedef struct _KSTATE {
-	/* process/thread support */
-	KTHREAD			threads[0x10];
-	uint8			threadndx;	
-	uint8			iswitch;
-	
 	/* new process/thread support */
+	KPROCESS		*procs;
+	KPROCESS		*cproc;
+	KTHREAD			*cthread;
 	
 	/* physical and heap memory management */
 	KHEAPBM			hphy;			/* kernel physical page heap */
 	KHEAPBM			hchk;			/* data chunk heap */
 	
+	/* time management */
+ 	uint64			ctime;
+	uint32			tpers;			/* ticks per second */
+	
 	/* virtual memory management */
 	KVMMTABLE		vmm;			/* kernel virtual memory map */
 	uint32			vmm_ucte;		/* unused coarse table entries */
</source>
The old thread/task structures removed and the new field added. A pointer to the first process entry (linked list) was added, and a pointer to the current process and thread.

=== Second Module Added ===
<source lang="c">
int _start(unsigned int *smmio) {
	int		x;
	int		y;
	
	for(;;);
	
	for (;;) {
		for (x = 0; x < 0xfffff; ++x);
		smmio[0] = 'A';
	}
	return 0;
}
</source>

=== First Module ===
This module demonstrates going to sleep.
<source lang="c">
unsigned int getTicksPerSecond() {
	unsigned int			out;
	asm("	swi #103 \n\
			mov %[out], r0 \n\
		" : [out]"=r" (out));
	return out;
}

void sleep(unsigned int timeout) {
	asm("	mov r0, %[in] \n\
			swi #101 \n\
		" : : [in]"r" (timeout));
}

int _start(unsigned int *smmio) {
	int				x;
	int				y;
	unsigned int	tps;
	
	tps = getTicksPerSecond();
	
	for (;;) {
		smmio[0] = 'G';
		smmio[0] = 'K';
		sleep(tps);
	}
	return 0;
}
</source>
This module demonstrates going to sleep instead of busy looping which wastes CPU that could be used for another thread.

=== Change In Timer Initialization ===
<source lang="c">
	t0mmio = (uint32*)0x13000100;
	t0mmio[REG_LOAD] = KTASKTICKS;
	t0mmio[REG_BGLOAD] = KTASKTICKS;			
	t0mmio[REG_CTRL] = CTRL_ENABLE | CTRL_MODE_PERIODIC | CTRL_SIZE_32 | CTRL_DIV_NONE | CTRL_INT_ENABLE;
	t0mmio[REG_INTCLR] = ~0;		/* make sure interrupt is clear (might not be mandatory) */
	ks->tpers = 1000000;
</source>
We now use ''KTASKTICKS'' and initialize ''ks->tpers'' which is read by threads using a software interrupts (''see exphandler''). This allows them to adjust their ticks for the sleep call to sleep for a certain amount of actual real time (''see exphandler'' SWI handler).

I also switched to using a 1MHZ timer just because I have not yet figured out how to detect the system clock speed. I have a few ideas but I am currently working on more important things, but it might be a good exercise for the reader to try to do this. The board may allow you to determine the system clock speed. Also, an idea is to use the 1MHZ timer to estimate the system clock synchronized timer 0. This might yield a different value on different systems depending on how the system clock timer is implemented in QEMU, but on real hardware it should always be the same.

[[Category:ARM]]
````

<a id="w003504"></a>

## W003504 — ARM Integrator-CP ITPTMME Main

Bron-ID: W003504
Namespace: Main (0)
Revisie: 28085 | Datum: 2023-07-09T21:14:02Z
Bron: https://wiki.osdev.org/ARM_Integrator-CP_ITPTMME_Main
Revisie-URL: https://wiki.osdev.org/index.php?title=ARM_Integrator-CP_ITPTMME_Main&oldid=28085

````text
== ARM Tutorial Series By Demonstration And Explanation ==
This consists of a series of pages building a minimal kernel and OS specifically for the ARM architecture. Each page builds on the previous state of the kernel. By following each step you should eventually reach a point of a functional kernel. The design is of a micro-kernel and follows a non-standard design which aims to keep the reader interested and challenges them to solve non-standard problems. Each step does not provide a linear progression, but instead demonstrates minimal and sometimes non-optimal solutions in order to slowly increase the functionality and performance of the kernel. At specific points a system may be replaced with a better one leaving the reader with the choice of replacing something and continuing forward or keeping the original design and deviating from the series or allows them to integrate back into the series at some point if they deviate.

Most explanations are provided in code if they are simplistic, and more complicated systems or code blocks are described both in code and text. The reader is assumed to have an adequate understanding of assembly and the C language. Also GCC, binutils, and QEMU are expected. 

''I use some specific features of GCC to produce code and I apologize if you use a different compiler, but GCC is ported to many operating systems and is widely available making it an attractive choice. It is also full featured.''

Different hardware than that provided by the Integrator-CP board can be used but will require slightly changing hardware implementation code. I expect to completely remove the hardware dependent code from the kernel at later stages and incorporate modules to provide hardware in a common interface for the kernel to use (example of PIC and timers used for scheduling). However, this would complicate the demonstrations too early in the beginning stages so it is delayed until later. It should also be noted that the hardware dependent code is very minimal and localized.

''The only hardware used by the kernel is the PIC and timer for task switching. Just about any board or SoC should come with some kind of timer but it may be slightly different in how you program it. The PIC can also be different, but generally its going to be very similar and only a few lines of code are needed to talk to it (same with timer). So do not become discouraged if you use a different board. As I stated I will eventually move the code external to the kernel sometime after module loading, and may incorporate both the PIC and timer into one module. Normally, user applications do not care what timer is used as long as it provides the required accuracy and hardware really does not care how interrupts are mapped as long as it can handle the interrupt making the PIC and timer a nice set to just combine into a single module for the SoC or board.''

Later parts of the series provide full code from a GIT repository, but are designed with the hope that everything can be implemented by reading the page alone, and the repository is provided instead for the reader to see a working implementation if they run into difficulties in implementing the functionality displayed by the code or explained by the text. This allows the reader to download a working copy and actually see it working and then inspect it.

The reader is also expected to not rely solely on the methods and designs used but instead supplement their learning by using other sources of development and design material such as other tutorials, books, courses, and readings. This is because the methods and designs used in the series are not expected to be optimal, viable, or realistic but instead be adequate to demonstrate a kernel and operating system that has a sufficient amount of functionality to be usable in an educational or research type manner.

''I might make mistakes in my implementations and I want you the reader to understand that you need to consult external sources and do your own thinking. My goal is simply to get you into it and to a point where you feel like branching off and doing your own design. Also part of that goal is providing you with a hard example of doing something in the event you run into trouble getting something to work such as exception tables, interrupts, task switching, virtual memory, hardware, and many other things. I also encourage you to put your own twist on things but if you like to copy and paste then you are welcome to do so.''

If you are looking for the standard, accepted, normal, correct, or proven designs and methods then this serial of tutorials may not be what you want. If you are looking for an interesting and different approach to development of a kernel suited to someone who does this as a hobby then this is the right place for you.

''Many people frown on using code to explain things, but I am a code kind of guy. I can read code faster than an explanation and maybe that is because I have been coding in C for many years. I can also read documentation too, but when I see an example in code it just clicks for me faster because I literally see a practical implementation of something. Since this is being written in assembly and C it is just natural that anyone who is going to follow along and comprehend be proficient in C (or at least willing to learn). By all means if your new to C and assembly do not let me discourage you because learning to read code now is better than later, and it actually helps you learn to read your own code well which helps with debugging.''

{| class="wikitable"
! Step
! Page
! Description
|-
| 1
| [[ARM_Integrator-CP_IRQTimerAndPIC|IRQ, Timer, And PIC]]
| Basic system software demonstrating handling IRQs with timer and PIC.
|-
| 2
| [[ARM_Integrator-CP_IRQTimerAndPICAndTaskSwitch|IRQ, Timer, PIC, And Tasks]]
| Added ability to run tasks using timer.
|-
| 3
| [[User:Pancakes/ARM_Integrator-CP_IRQTimerPICTasksAndMM|IRQ, Timer, PIC, Tasks, And MM]]
| Added memory management and virtual memory support. Identity mapped kernel.
|-
| 4
| [[User:Pancakes/ARM_Integrator-CP_VMM2|IRQ, Timer, PIC, Tasks, MM (Paged Kernel)]]
| More advanced and flexible virtual memory support with paged kernel.
|-
| 5
| [[ARM_Integrator-CP_IRQTimerPICTasksMMAndMods|Attached Modules And Minimal ELF Loading]]
| Usage of modules attached to kernel image, and loading of ELF32 modules into protected user address space.
|-
| 6
| [[ARM_Integrator-CP_ITPTMME_Phase2|Tasks To Processes And Threads With System Calls]]
| Addition of process and thread design with minimal system calls for demonstration.
|}

[[Category:ARM]]
````

<a id="w003505"></a>

## W003505 — Executable and Linkable Format

Bron-ID: W003505
Namespace: Main (0)
Revisie: 15936 | Datum: 2014-04-08T18:03:07Z
Bron: https://wiki.osdev.org/Executable_and_Linkable_Format
Revisie-URL: https://wiki.osdev.org/index.php?title=Executable_and_Linkable_Format&oldid=15936
Doorverwijzing: ELF

````text
#REDIRECT [[ELF]]
````

<a id="w003506"></a>

## W003506 — Executable and Linking Format

Bron-ID: W003506
Namespace: Main (0)
Revisie: 15937 | Datum: 2014-04-08T18:26:53Z
Bron: https://wiki.osdev.org/Executable_and_Linking_Format
Revisie-URL: https://wiki.osdev.org/index.php?title=Executable_and_Linking_Format&oldid=15937
Doorverwijzing: ELF

````text
#REDIRECT [[ELF]]
````

<a id="w003507"></a>

## W003507 — System V ABI

Bron-ID: W003507
Namespace: Main (0)
Revisie: 29723 | Datum: 2025-10-20T19:38:00Z
Bron: https://wiki.osdev.org/System_V_ABI
Revisie-URL: https://wiki.osdev.org/index.php?title=System_V_ABI&oldid=29723

````text
The '''System V Application Binary Interface''' is a set of specifications that detail [[Calling Conventions|calling conventions]], [[Object Files|object file formats]], [[Executable Formats|executable file formats]], dynamic linking semantics, and much more for systems that complies with the ''X/Open Common Application Environment Specification'' and the ''System V Interface Definition''. It is today the standard ABI used by the major Unix operating systems such as Linux, the BSD systems, and many others. The [[Executable and Linkable Format]] (ELF) is part of the System V ABI. 

The ABI is organized as a portable base document and platform-specific supplements that fill in the blank gaps. Unofficial new architecture processor supplements have been published as the format has been adapted to new platforms such as [[X86-64]]. The standard is extensible and the format continues to evolve as Unix vendors add new features. Due to the many unofficial supplement specifications and the chaotic history of the Unix operating systems, the current situation is that the System V ABI has become a family of unofficial draft specifications with no real central governing body.

Many of the advanced feature such as dynamic linking are optional and loading a simple statically linked [[ELF]] program is straightforward. Earlier versions of the standard were more ambitious and attempted to standardize software package installation formats and X11 details, while these obsolete details are disregarded today. The ABI is well-understood by common operating system development tools like [[Binutils]] and [[GCC]]. Toolchains such as <tt>i686-elf-gcc</tt> generate code and executable files according to this ABI.

== Executable and Linkable Format ==
{{Main|Executable and Linkable Format}}

The [[Executable and Linkable Format]] is standardized as an adaptable file format in the System V ABI. Each processor supplement subtly changes the file format by declaring the size of abstract types used in the ELF format structures as well as the endianness. This allows the skeleton file format to be adapted to multiple processor architectures, where the difference between 32-bit and 64-bit systems are handled by simply increasing the size of various header fields. The format is powerful enough to contain auxiliary information such as debugging information, relocations for dynamic libraries and other vendor-specific miscellaneous information. This allows using the same format for both object files and linked executables.

== Calling Convention ==

This is a short overview of the important [[Calling Conventions|calling convention]] details for the major System V ABI architectures. This is an incomplete account and you should consult the relevant processor supplement document for the details. Additionally, you can use the <tt>-S</tt> compiler option to stop the compilation process before the assembler is invoked, which lets you study how the compiler translates code to assembly following the relevant calling convention.

=== i386 ===

This is a 32-bit platform. The stack grows downwards. Parameters to functions are passed on the stack in reverse order such that the first parameter is the last value pushed to the stack, which will then be the lowest value on the stack. Parameters passed on the stack may be modified by the called function. Functions are called using the <tt>call</tt> instruction that pushes the address of the next instruction to the stack and jumps to the operand. Functions return to the caller using the <tt>ret</tt> instruction that pops a value from the stack and jump to it. The stack is 4-byte aligned all the time, on older systems and those honouring the SYSV psABI. On some newer systems, the stack is additionally 16-byte aligned just before the call instruction is called (usually those that want to support SSE instructions); consult your manual (GNU/Linux on i386 has recently become such a system, but code mixing with 4-byte stack alignment-assuming code is possible).

Functions preserve the registers <tt>ebx</tt>, <tt>esi</tt>, <tt>edi</tt>, <tt>ebp</tt>, and <tt>esp</tt>; while <tt>eax</tt>, <tt>ecx</tt>, <tt>edx</tt> are scratch registers. The return value is stored in the <tt>eax</tt> register, or if it is a 64-bit value, then the higher 32-bits go in <tt>edx</tt>. Functions push <tt>ebp</tt> such that the <tt>caller-return-eip</tt> is 4 bytes above it, and set <tt>ebp</tt> to the address of the saved ebp. This allows iterating through the existing stack frames. This can be eliminated by specifying the <tt>-fomit-frame-pointer</tt> GCC option.

As a special exception, GCC assumes the stack is not properly aligned and realigns it when entering <tt>main</tt> or if the <tt>attribute ((force_align_arg_pointer))</tt> is set on the function.

The direction flag '''must be clear''' before calling a function and is expected clear on exit too. This allows more efficient forward byte string operations. [1] [2] '''The System V ABI has the same requirement for 64-bit.'''

[1]: <nowiki>https://www.sco.com/developers/devspecs/abi386-4.pdf</nowiki> (page 38)

[2]: <nowiki>https://learn.microsoft.com/en-us/cpp/c-runtime-library/direction-flag?view=msvc-170</nowiki>

=== x86-64 ===

This is a 64-bit platform. The stack grows downwards. Parameters to functions are passed in via the registers <tt>rdi</tt>, <tt>rsi</tt>, <tt>rdx</tt>, <tt>rcx</tt>, <tt>r8</tt>, and <tt>r9</tt>. Floating-point parameters are passed in via <tt>xmm0</tt> through <tt>xmm7</tt>. Unlike the standard parameter passing registers, floating point registers are not preserved between function calls. Any additional arguments that do not fit in these registers are passed on the stack in reverse order. Parameters passed via the stack may be modified by the called function. Functions are called using the <tt>call</tt> instruction, which pushes the address of the next instruction onto the stack and jumps to the operand. Functions return to the caller using the <tt>ret</tt> instruction, which pops the return address from the stack and jumps to it. The stack is 16-byte aligned just before the <tt>call</tt> instruction is executed.

Functions preserve the registers <tt>rbx</tt>, <tt>rsp</tt>, <tt>rbp</tt>, <tt>r12</tt>, <tt>r13</tt>, <tt>r14</tt>, and <tt>r15</tt>; while <tt>rax</tt>, <tt>rdi</tt>, <tt>rsi</tt>, <tt>rdx</tt>, <tt>rcx</tt>, <tt>r8</tt>, <tt>r9</tt>, <tt>r10</tt>, <tt>r11</tt> are scratch registers. The return value is stored in the <tt>rax</tt> register, or if it is a 128-bit value, then the higher 64-bits go in <tt>rdx</tt>. Optionally, functions push <tt>rbp</tt> such that the <tt>caller-return-rip</tt> is 8 bytes above it, and set <tt>rbp</tt> to the address of the saved rbp. This allows iterating through the existing stack frames. This can be eliminated by specifying the <tt>-fomit-frame-pointer</tt> GCC option.

If the return value does not fit in <tt>rax</tt> and <tt>rdx</tt>, then the caller must reserve storage space and pass a pointer in <tt>rdi</tt> as-if it was the first argument. The callee must return the same pointer in <tt>rax</tt>.

Signal handlers are executed on the same stack, but 128 bytes known as the red zone is subtracted from the stack before anything is pushed to the stack. This allows small leaf functions to use 128 bytes of stack space without reserving stack space by subtracting from the stack pointer. The red zone is well-known to cause problems for x86-64 kernel developers, as the CPU itself doesn't respect the red zone when calling interrupt handlers. This leads to a subtle kernel breakage as the ABI contradicts the CPU behavior. The solution is to build all kernel code with <tt>-mno-red-zone</tt> or by handling interrupts in kernel mode on another stack than the current (and thus implementing the ABI).

==See Also==

=== Documents ===
'''TODO:''' ''Ensure whether these are the latest official links. These documents are simply what I could find through a quick online search.''
* base document and addons
** [http://www.sco.com/developers/gabi/latest/contents.html System V ABI - Latest Base Document]
** [http://www.sco.com/developers/devspecs/gabi41.pdf System V ABI - Older Base Document]
** [http://www.akkadia.org/drepper/tls.pdf ELF Handling For Thread-Local Storage]
* x86 (i8086, i386, amd64, x32, k1om)
** [https://gitlab.com/x86-psABIs/x86-64-ABI System V ABI - AMD64 Architecture Processor Supplement]
** [http://www.sco.com/developers/devspecs/abi386-4.pdf System V ABI - Intel386 Architecture Processor Supplement]
** [http://software.intel.com/sites/default/files/forum/278102/k1om-psabi-1.0.pdf System V ABI - K1OM Architecture Processor Supplement]
* MIPS
** [http://math-atlas.sourceforge.net/devel/assembly/mipsabi32.pdf System V ABI - MIPS RISC Processor Supplement]
** [http://math-atlas.sourceforge.net/devel/assembly/mipsabi64.pdf System V ABI - MIPSpro™ 64-Bit]
** [http://math-atlas.sourceforge.net/devel/assembly/007-2816-005.pdf System V ABI - MIPSpro™ N32 ABI Handbook]
* PowerPC
** [http://math-atlas.sourceforge.net/devel/assembly/elfspec_ppc.pdf System V ABI - PowerPC Processor Supplement]
** [http://math-atlas.sourceforge.net/devel/assembly/PPC-elf64abi-1.7.pdf System V ABI - 64-bit PowerPC]
* SPARC
** [http://math-atlas.sourceforge.net/devel/assembly/abi_sysV_sparc.pdf System V ABI - SPARC® Processor Supplement]
** [http://math-atlas.sourceforge.net/devel/assembly/64.psabi.1.33.ps.Z System V ABI - SPARC® Version 9 Processor Supplement]
* Itanium
** [http://www.csee.umbc.edu/portal/help/architecture/24537001.pdf System V ABI - IA-64 Architecture Processor Supplement]
* ⚠ not SYSV ABI, but at least the architecture-official calling standards:
** [http://bitsavers.trailing-edge.com/pdf/dec/alpha/Alpha_Calling_Standard_Rev_2.0_19900427.pdf DEC Alpha calling standard]

===External Links===
* [http://www.sco.com/developers/gabi/ System V ABI Website]
* [https://refspecs.linuxfoundation.org/ Linux Foundation — Referenced Specifications]
* [https://course.ece.cmu.edu/~ece447/s15/doku.php?id=techdocs Introduction to Computer Architecture — Technical Documents]

[[Category:ABI]]
[[Category:Executable Formats]]
[[Category:Object Files]]
[[Category:Standards]]
````

<a id="w003508"></a>

## W003508 — Category:ABI

Bron-ID: W003508
Namespace: Category (14)
Revisie: 15943 | Datum: 2014-04-08T19:10:34Z
Bron: https://wiki.osdev.org/Category:ABI
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AABI&oldid=15943

````text
This is a category of ABI-specific articles.
````

<a id="w003509"></a>

## W003509 — User:Johnburger/Demo/Data.inc

Bron-ID: W003509
Namespace: User (2)
Revisie: 21404 | Datum: 2017-08-29T09:09:37Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Data.inc
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FData.inc&oldid=21404

````text
Device interrupts may need access to a Data Segment - for example, the Keyboard handler needs to put the keycodes somewhere! This Data segment cannot be in a Local Descriptor Table, since the LDT may not be the one that is active when the interrupt occurs - unless the interrupt handler is a full-blown TSS that switches in its own LDT. That may not be a good idea for a frequent interrupt, but may be OK for something as slow as a keyboard.

So this code is for the global Data Segment. The Keyboard handler uses it, as does the Memory Allocator - for which the RAMMap has already been filled in at boot time by [[User:Johnburger/Demo/Boot/RAM|Boot/RAM]].

== <code>[[User:Johnburger/Demo/Demo|Demo]]/Data.inc</code> ==
<source lang="asm">
;
; Data.inc
;

; This is the definition for the global Data Segment. Note that it is
; mostly populated at Boot by Boot/RAM.inc

                 SEGMENT        Data  START=0  ALIGN=16  NOBITS

                 USE32

Data.RAM         RESD           1       ; Beginning of available RAM
Data.RAMTop      RESD           1       ; Top of RAM as reported by BIOS

Data.Key.Code    RESB           1       ; Storage for key scan-code
Data.Key.Shifts  RESB           1       ; Storage for current Key Shift states

                 RESB           12      ; Reserved

Data.RAMMap.Num  RESW           1       ; Number of BIOS.RAMMap entries

Data.RAMMap      RESB           Data.Size - ($-$$) ; RAMMap
Data.RAMMap.Size EQU            $ - Data.RAMMap
Data.RAMMap.Max  EQU            Data.RAMMap.Size / BIOS.RAMMap_size
</source>
````

<a id="w003510"></a>

## W003510 — User:Johnburger/Demo/BIOS/BIOS

Bron-ID: W003510
Namespace: User (2)
Revisie: 21357 | Datum: 2017-08-28T21:11:43Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/BIOS/BIOS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FBIOS%2FBIOS&oldid=21357

````text
Although the BIOS includes a huge number of functions, this Demonstrator only uses a few, so only those are defined herein.

Each BIOS handler has its own <code>BIOS.*.Int</code> number, and just about all of them expect <code>AH</code> to hold the requested <code>BIOS.*.Fn</code>. From there other registers may be used to provide more information.

One guarantee is that BIOS functions will not modify any registers that aren't expressly mentioned in their description, such as for returning results. In particular, <code>ESI</code> and <code>EDI</code> are practically never used as input or output parameters.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Overview#BIOS_Definitions|BIOS]]/BIOS.inc</code> ==
<source lang="asm">
;
; BIOS/BIOS.inc
;

; The original IBM PC BIOS, and various successors, have all embodied relatively
; arbitrary design decisions that have become industry standards. Of course, not
; every BIOS uses the same definitions, and others had bugs which were fixed in
; later versions, but those arbitrary decisions need to be either utilised or
; accommodated by any system wanting to work on the PC.

BIOS.RAMTop.Int EQU             12h

;-------------------------------------------------------------------------------
%include        "BIOS/A20.inc"          ; Definitions for A20 Gate controller
;-------------------------------------------------------------------------------
%include        "BIOS/RAMMap.inc"       ; Definitions for RAMMap information
;-------------------------------------------------------------------------------
%include        "BIOS/Key.inc"          ; Definitions for Keyboard
;-------------------------------------------------------------------------------
%include        "BIOS/VGA.inc"          ; Definitions for VGA video
;-------------------------------------------------------------------------------
%include        "BIOS/Disk.inc"         ; Definitions for Disk controller
;-------------------------------------------------------------------------------
%include        "BIOS/MBR.inc"          ; Definitions for Master Boot Record
;-------------------------------------------------------------------------------
%include        "BIOS/MemMap.inc"       ; Definitions for boot-time memory map
</source>
````

<a id="w003511"></a>

## W003511 — User:Johnburger/Demo/BIOS/A20

Bron-ID: W003511
Namespace: User (2)
Revisie: 21387 | Datum: 2017-08-29T09:00:28Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/BIOS/A20
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FBIOS%2FA20&oldid=21387

````text
The A20 gate has a defined BIOS handler for it - but surprisingly it is not always implemented by BIOS manufacturers. [[A20_Line#Recommended_Method|This page]] in the Wiki describes the recommended sequence of steps to perform - this is Step 2.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/BIOS/BIOS|BIOS]]/A20.inc</code> ==
<source lang="asm">
;
; BIOS/A20.inc
;

; These are the definitions for the BIOS A20 controller
BIOS.A20.Int    EQU             15h
BIOS.A20.Fn     EQU             24h
BIOS.A20.Enable EQU             01h
BIOS.A20.Disable EQU            00h
</source>
````

<a id="w003512"></a>

## W003512 — User:Johnburger/Demo/BIOS/RAMMap

Bron-ID: W003512
Namespace: User (2)
Revisie: 21388 | Datum: 2017-08-29T09:00:51Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/BIOS/RAMMap
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FBIOS%2FRAMMap&oldid=21388

````text
The BIOS is about the only thing that can know how the PC's RAM is set up, since it's the code that set it up in the first place. The RAMMap BIOS function reports the memory address space layout through a series of calls.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/BIOS/BIOS|BIOS]]/RAMMap.inc</code> ==
<source lang="asm">
;
; BIOS/RAMMap.inc
;

; These are the definitions for the BIOS RAM Map information handler

BIOS.RAMMap.Int EQU             15h
BIOS.RAMMap.Fn  EQU             0000_E820h
BIOS.RAMMap.Magic EQU           "PAMS"          ; NASM reverses "SMAP"

; This is the BIOS RAMMap structure
                STRUC           BIOS.RAMMap
.Base           RESQ            1
.Length         RESQ            1
.Type           RESD            1
.Flags          RESD            1 ; Added in later BIOSes
                ENDSTRUC

BIOS.RAMMap.Flags.DontIgnore EQU 0000_0000_0000_0001b
</source>
````

<a id="w003513"></a>

## W003513 — User:Johnburger/Demo/BIOS/Key

Bron-ID: W003513
Namespace: User (2)
Revisie: 21389 | Datum: 2017-08-29T09:01:11Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/BIOS/Key
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FBIOS%2FKey&oldid=21389

````text
There isn't much that the bootstrap needs the BIOS for in the way of keyboard input, but there is one setting that is nice to improve. Here are the definitions to set the typematic delay and rate.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/BIOS/BIOS|BIOS]]/Key.inc</code> ==
<source lang="asm">
;
; BIOS/Key.inc
;

; These are the definitions for the BIOS keyboard controller

BIOS.Key.Int    EQU             16h

BIOS.Key.SetRate.Fn       EQU   03h
BIOS.Key.SetRate.Default  EQU   00h ; Set back to default
BIOS.Key.SetRate.Increase EQU   01h
BIOS.Key.SetRate.Decrease EQU   02h ; Slow down by half
BIOS.Key.SetRate.Off      EQU   04h ; Turn off
BIOS.Key.SetRate.Set      EQU   05h ; Set to below values (BH:BL)

; Delay before typematic starts, in milliseconds (in BH)
BIOS.Key.SetRate.Delay.250  EQU 00h
BIOS.Key.SetRate.Delay.500  EQU 01h
BIOS.Key.SetRate.Delay.750  EQU 02h
BIOS.Key.SetRate.Delay.1000 EQU 03h

; Rate at which typematic occurs, in chars per 10 seconds (in BL)
BIOS.Key.SetRate.Rate.300  EQU  00h
BIOS.Key.SetRate.Rate.267  EQU  01h
BIOS.Key.SetRate.Rate.240  EQU  02h
BIOS.Key.SetRate.Rate.218  EQU  03h
BIOS.Key.SetRate.Rate.200  EQU  04h
BIOS.Key.SetRate.Rate.185  EQU  05h
BIOS.Key.SetRate.Rate.171  EQU  06h
BIOS.Key.SetRate.Rate.160  EQU  07h
BIOS.Key.SetRate.Rate.150  EQU  08h
BIOS.Key.SetRate.Rate.133  EQU  09h
BIOS.Key.SetRate.Rate.120  EQU  0Ah
BIOS.Key.SetRate.Rate.109  EQU  0Bh
BIOS.Key.SetRate.Rate.100  EQU  0Ch
BIOS.Key.SetRate.Rate.92   EQU  0Dh
BIOS.Key.SetRate.Rate.86   EQU  0Eh
BIOS.Key.SetRate.Rate.80   EQU  0Fh
BIOS.Key.SetRate.Rate.75   EQU  10h
BIOS.Key.SetRate.Rate.67   EQU  11h
BIOS.Key.SetRate.Rate.60   EQU  12h
BIOS.Key.SetRate.Rate.55   EQU  13h
BIOS.Key.SetRate.Rate.50   EQU  14h
BIOS.Key.SetRate.Rate.46   EQU  15h
BIOS.Key.SetRate.Rate.43   EQU  16h
BIOS.Key.SetRate.Rate.40   EQU  17h
BIOS.Key.SetRate.Rate.37   EQU  18h
BIOS.Key.SetRate.Rate.33   EQU  19h
BIOS.Key.SetRate.Rate.30   EQU  1Ah
BIOS.Key.SetRate.Rate.27   EQU  1Bh
BIOS.Key.SetRate.Rate.25   EQU  1Ch
BIOS.Key.SetRate.Rate.23   EQU  1Dh
BIOS.Key.SetRate.Rate.21   EQU  1Eh
BIOS.Key.SetRate.Rate.20   EQU  1Fh
</source>
````

<a id="w003514"></a>

## W003514 — User:Johnburger/Demo/BIOS/VGA

Bron-ID: W003514
Namespace: User (2)
Revisie: 21390 | Datum: 2017-08-29T09:01:30Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/BIOS/VGA
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FBIOS%2FVGA&oldid=21390

````text
Most of the screen handling is done by directly accessing the memory, so [[User:Johnburger/Demo/Dev/VGA|Dev/VGA]] defines a large number of symbols for use.

The BIOS implements the tricky (and usually proprietary) part of changing the current display mode. Here are the definitions for that.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/BIOS/BIOS|BIOS]]/VGA.inc</code> ==
<source lang="asm">
;
; BIOS/VGA.inc
;

; These are the definitions for the BIOS video controller

BIOS.VGA.Int    EQU             10h

BIOS.VGA.SetMode.Fn    EQU      00h
BIOS.VGA.SetMode.80x25 EQU      03h

BIOS.VGA.ExtMode.Fn    EQU      11h
BIOS.VGA.ExtMode.80x50 EQU      12h

BIOS.VGA.Cursor.Fn    EQU       01h
BIOS.VGA.Cursor.None  EQU       0010_0000_0000_0000b
BIOS.VGA.Cursor.Under EQU       06_07h
BIOS.VGA.Cursor.Block EQU       00_07h
</source>
````

<a id="w003515"></a>

## W003515 — User:Johnburger/Demo/BIOS/Disk

Bron-ID: W003515
Namespace: User (2)
Revisie: 21391 | Datum: 2017-08-29T09:01:46Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/BIOS/Disk
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FBIOS%2FDisk&oldid=21391

````text
Accessing various types of disk drives - floppies, IDE, SATA, SCSI, and even USB - takes a lot of code. Luckily the BIOS hides that from us, and lets the 512-byte boot sector do it in a handful of instructions.

Here are the definitions to access all of these different kinds of disks.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/BIOS/BIOS|BIOS]]/Disk.inc</code> ==
<source lang="asm">
;
; BIOS/Disk.inc
;

; These are the definitions for the BIOS Disk controller

BIOS.Disk.Sector.Size EQU       512

BIOS.Disk.Int      EQU          13h
BIOS.Disk.Fn.Reset EQU          00h
BIOS.Disk.Fn.Read  EQU          02h
</source>
````

<a id="w003516"></a>

## W003516 — User:Johnburger/Demo/BIOS/MBR

Bron-ID: W003516
Namespace: User (2)
Revisie: 21392 | Datum: 2017-08-29T09:02:02Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/BIOS/MBR
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FBIOS%2FMBR&oldid=21392

````text
Strictly speaking, the Master Boot Record (MBR) is not actually a BIOS structure. It's defined by itself - its own code defines and accesses the Partition Table entries embodied within itself.

It could be that this boot code has been loaded from the MBR, so we need to access the entry that defines the Partition to continue to load the rest of the system.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/BIOS/BIOS|BIOS]]/MBR.inc</code> ==
<source lang="asm">
;
; BIOS/MBR.inc
;

; These are the definitions for the standard BIOS Master Boot Record

; If we booted from a hard disk, ES:SI will be pointing to the relevant Partition
                STRUC           BIOS.MBR.Entry
.Active         RESB            1
.StartHead      RESB            1
.StartCylSect   RESW            1
.Type           RESB            1
.EndHead        RESB            1
.EndCylSect     RESW            1
.Start          RESD            1
.Size           RESD            1
                ENDSTRUC

; The two CylSect entries above have the following format. But don't be fooled!
; The HIGH bits of the Cylinder are in the LOW byte  - it's weirdly reversed!
BIOS.MBR.Entry.Cyl  EQU         1111_1111_1100_0000b
BIOS.MBR.Entry.Sect EQU         0000_0000_0011_1111b
</source>
````

<a id="w003517"></a>

## W003517 — User:Johnburger/Demo/BIOS/MemMap

Bron-ID: W003517
Namespace: User (2)
Revisie: 21393 | Datum: 2017-08-29T09:02:22Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/BIOS/MemMap
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FBIOS%2FMemMap&oldid=21393

````text
The system needs to know where some things in memory are, and sometimes what values they have - or it needs to put them there.

To that end, I need symbols for various areas of the BIOS Memory Map.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/BIOS/BIOS|BIOS]]/MemMap.inc</code> ==
<source lang="asm">
;
; BIOS/MemMap.inc
;

; This is the Memory Map of when the BIOS first boots.

; You'll notice that the Stack is on top of the Interrupt Vector Table!
BIOS.Stack.Base EQU             0000h      ; BIOS Boot stack
BIOS.Stack.Size EQU             0400h

BIOS.BDA.Base   EQU             0400h      ; BIOS Data Area
BIOS.BDA.Size   EQU             0100h

BIOS.MBR.Base   EQU             0600h      ; BIOS Master Boot Record load area
BIOS.MBR.Size   EQU             0200h

BIOS.Entry      EQU             7C00h      ; BIOS Boot Entry point

BIOS.Sig.Value  EQU             0AA55h     ; BIOS Signature flag
BIOS.Sig.Pos    EQU             BIOS.Disk.Sector.Size - 2
</source>
````

<a id="w003518"></a>

## W003518 — User:Johnburger/Demo/Host

Bron-ID: W003518
Namespace: User (2)
Revisie: 21358 | Datum: 2017-08-28T21:11:59Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Host
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FHost&oldid=21358

````text
The development environment for a software engineer is a personal choice. I've used PCs most of my professional life, but I have used mainframes, minicomputers, SGI/HP/Sun Unix workstations and Macs too.

On the PCs I've used DOS, Linux and Windows, and I probably miss DOS the most. Hey, I'm an embedded engineer - I want to be <em>close</em> to the machine I'm working on! By using various virtualization programs I can still get the runtime environment I need, as described in [[User:Johnburger/Demo/Target|Target Machine]].

For this project, I did most of the development work under Windows 7 and VMware player.

== Assembler ==
I wrote all the code in assembly. The assembler choice needed to be made up front, since different assemblers have different syntax or directives. I wanted you to be able to work with the code, so I also wanted to use a readily-available one. But since I've used Intel syntax for all my x86 life, I didn't want to make the mind-switch to Gnu's assembler - I'd be sure to make mistakes with the reversed opcodes!

So I've used [http://www.nasm.us/ NASM] - and taken full advantage of two of my [[User:Johnburger/Demo/Overview#Coding_Style|favourite features]] that it offers: local/hierarchical identifiers and macros.

The command line to assemble the source depends on how you want to use the output:
{|
| For CD:
| <code>nasm -o Demo.iso Demo.asm</code>
|-
| For USB:
| <code>nasm -o Demo.usb Demo.asm</code>
|-
| For Floppy:
| <code>nasm -o Demo.flp Demo.asm</code>
|-
| For Hard Disk:
| <code>nasm -o Demo.bin Demo.asm</code>
|}

No, I'm not using some kind of fancy NASM macro to examine the output file name: this source code generates a single binary that can be used for any of the above media! You can tailor the code to invalidate any of the output types: for example, if you comment out the following line in [[User:Johnburger/Demo/Demo|Demo/Demo]], you can still ''output'' a <code>.iso</code> file, it just won't work like one!
<source lang="asm">
;%define         IMAGE.ISO       ; Write as bootable CD (ISO-9660 and El Torito)
</source>

== IDE ==
I don't want to start a Holy War about the relative merits of different IDEs - I've used around a dozen in my decades of programming, and none are perfect. For this project I used a freeware editing environment called [http://www.pspad.com/ PSPad]. Some of its advantages include:
* Simple text-based editing environment;
* Hex editor for binary files;
* Simple text-based project file (see below);
* Syntax highlighting;
* Invoke a build command, and capture and process the output for one-click jumps to errors;
* Above all: [[User:Johnburger/Demo/Overview#Hierarchical_Code|hierarchical]] project specification.

The following is the PSPad Project (<code>.ppr</code>) file. It's fairly straightforward in its layout: after a few project-specific settings, the file layout is there for all to see. You've probably guessed that there's a Wiki page for each of the files...

== <code>[[User:Johnburger/Demo/Overview#Development_Host|Demo]]/Demo.ppr</code> ==
<pre>
[Config]
Compilator.FileName=C:\NASM\NASM.exe
Compilator.PSPar=-l %Name%.lst %Name%%Ext%
Compilator.Log=%Name%.lst
Compilator.Run=CMD /c COPY %Name% %Name%.iso && COPY %Name% %Name%.flp
Compilator.SaveAll=1
Compilator.Capture=1
Compilator.ParsLog=%F:%L:* ; NASM
Compilator.HideOutput=1
Compilator.LogType=0
DefaultCPIndex=0
LogtoEnd=0
DontOpen=0
AbsolutePath=0
FileFormat=0
ProjectFilesOnly=0
[Project tree]
Demo
	Demo.asm
	Demo.inc
	Demo.lst
	Demo.map
	Demo
	+x86
		x86\x86.inc
		x86\EFlags.inc
		x86\Seg.inc
		x86\CR0.inc
		x86\Desc.inc
		x86\TSS.inc
		x86\IVT.inc
	+BIOS
		BIOS\BIOS.inc
		BIOS\A20.inc
		BIOS\RAMMap.inc
		BIOS\Key.inc
		BIOS\VGA.inc
		BIOS\Disk.inc
		BIOS\MBR.inc
		BIOS\MemMap.inc
	+Dev
		Dev\Dev.inc
		Dev\A20.inc
		Dev\PIC.inc
		Dev\Timer.inc
		Dev\Key.inc
		Dev\VGA.inc
	+Boot
		Boot\Boot.inc
		Boot\Entry.inc
		Boot\CPU.inc
		Boot\Load.inc
		Boot\IDT.inc
		Boot\RAM.inc
		Boot\A20.inc
		Boot\Key.inc
		Boot\VGA.inc
		Boot\PM.inc
	Data.inc
	+Ints
		Ints\Ints.inc
		Ints\IDT.inc
		Ints\Fault.inc
		Ints\Vectors.inc
		Ints\User.inc
		Ints\Single.inc
		+Debug
			Ints\Debug\Debug.inc
			Ints\Debug\Init.inc
			Ints\Debug\Key.inc
			Ints\Debug\Show.inc
			Ints\Debug\Regs.inc
		Ints\NoSeg.inc
		Ints\IRQs.inc
		Ints\Timer.inc
		Ints\Key.inc
		Ints\Util.inc
	+User
		User\User.inc
		User\LDT.inc
		User\Data.inc
		User\Code.inc
		User\Frame.inc
	+Exec
		Exec\Exec.inc
		Exec\Init.inc
		+Ints
			Exec\Ints\Ints.inc
			Exec\Ints\Debug.inc
			Exec\Ints\Trace.inc
			Exec\Ints\NoSeg.inc
			Exec\Ints\Double.inc
			Exec\Ints\BadStack.inc
			Exec\Ints\BadTSS.inc
			Exec\Ints\Fault.inc
		Exec\PICs.inc
		Exec\Timer.inc
		Exec\Key.inc
		+Start
			Exec\Start\Start.inc
			Exec\Start\LDT.inc
			Exec\Start\TSS.inc
		+Alloc
			Exec\Alloc\Alloc.inc
			Exec\Alloc\RAM.inc
			Exec\Alloc\DT.inc
			Exec\Alloc\IDT.inc
			Exec\Alloc\GDT.inc
			Exec\Alloc\LDT.inc
			Exec\Alloc\TSS.inc
		Exec\LDT.inc
		Exec\GDT.inc
	+Pad
		Pad\Sizes.inc
		Pad\HardDisk.inc
		+ISO
			Pad\ISO\ISO.inc
			Pad\ISO\Defn.inc
			Pad\ISO\Primary.inc
			Pad\ISO\Boot.inc
			Pad\ISO\Terminator.inc
			Pad\ISO\Catalog.inc
			Pad\ISO\PathTable.inc
			Pad\ISO\RootDir.inc
		Pad\Floppy.inc
[Open project files]
[Selected Project Files]
Main=Demo.asm
</pre>
````

<a id="w003519"></a>

## W003519 — User:Johnburger/Demo/Target

Bron-ID: W003519
Namespace: User (2)
Revisie: 21155 | Datum: 2017-08-28T20:21:29Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Target
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FTarget&oldid=21155

````text
To test the code, it is easiest to use a virtualization environment such as VMware, Virtual PC or VirtualBox - I've used all of them. Each has their own idiosyncrasies, but if I can get the same code to work on all of them, I know I have a general solution.
__TOC__
=== Virtualisation Environment ===
The virtualization environment was the same in every case:
* 4 MiB RAM
* Single 1 GB hard disk (not used for the Demonstrator as presented here - maybe later!)
* Floppy / CD-ROM as required for boot media.
Note that VMware has an "Install from disc" option. This is '''NOT''' the option to choose - use "I will install the OS later."

=== Physical Environment ===
Of course, a virtualization environment is not the real thing. Also, I wanted to try the various boot media solutions to confirm that they worked correctly. Below is a complete list of what I tested on:
* Windows 7 and VMware
** Floppy image
** ISO image
** Physical floppy
** Physical CD
* Windows 7 and Virtual PC
** Floppy image
** ISO image
* Windows 7 and VirtualBox
** ISO image
* Ubuntu 12.04 and VirtualBox
** Floppy image
** ISO image
* Olivetti PCS/386SX with 2 MiB RAM
** Floppy
** Hard disk (MBR)
** Hard disk (Partition)
* Asus Core i7 laptop
** USB Floppy
** USB CD
** USB memory stick
** Hard disk (partition)
* Samsung Core i7 laptop
** USB Floppy
** USB CD
** USB memory stick
````

<a id="w003520"></a>

## W003520 — User:Johnburger/Demo/Dev/Dev

Bron-ID: W003520
Namespace: User (2)
Revisie: 21359 | Datum: 2017-08-28T21:12:21Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Dev/Dev
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FDev%2FDev&oldid=21359

````text
Although a PC has a large number of devices, this Demonstrator only uses a few of them - and not their full capability in each case.

No code is embodied in these files - except for some macro definitions, maybe. These definitions are merely convenient symbols for the actual device drivers to use.

If the definition is for an I/O Port, or the individual values that it can use, they're usually grouped together to make it obvious. Also, since all (standard) I/O ports are in the range <code>000h</code>-<code>3FFh</code>, I use three-digit hex numbers and a <code>.Port</code> suffix to represent them. Bit-fields are represented in binary, while command values are represented in hex.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Overview#Device_Definitions|Dev]]/Dev.inc</code> ==
<source lang="asm">
;
; Dev/Dev.inc
;

; Different devices have different definitions: for their registers, their
; assumed structures, and for their access protocols.

;-------------------------------------------------------------------------------
%include        "Dev/A20.inc"   ; Definitions for the A20 Gate controller
;-------------------------------------------------------------------------------
%include        "Dev/PIC.inc"   ; Definitions for the Priority Interrupt Controllers
;-------------------------------------------------------------------------------
%include        "Dev/Timer.inc" ; Definitions for the Programmable Interval Timer
;-------------------------------------------------------------------------------
%include        "Dev/Key.inc"   ; Definitions for the Keyboard
;-------------------------------------------------------------------------------
%include        "Dev/VGA.inc"   ; Definitions for the VGA video controller
</source>
````

<a id="w003521"></a>

## W003521 — User:Johnburger/Demo/Dev/A20

Bron-ID: W003521
Namespace: User (2)
Revisie: 21413 | Datum: 2017-08-29T09:14:51Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Dev/A20
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FDev%2FA20&oldid=21413

````text
There are actually two (potential) A20 Gate controllers on the PC:
* One is the "fast" controller defined on the IBM PS/2;
* The other is the traditional one that is accessed through the keyboard controller.

By giving the latter its own definitions, as though it was a separate controller to the keyboard (albeit with the same values), risks writing the same access routines for both A20 and the keyboard, instead of using common code. For example, very similar things need to be done to command the A20 gate versus telling the keyboard to light the Lock LEDs.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Dev/Dev|Dev]]/A20.inc</code> ==
<source lang="asm">
;
; Dev/A20.inc
;

; These are the definitions for the A20 Gate registers
Dev.A20.Fast.Port  EQU          092h
Dev.A20.Fast.A20   EQU          0000_0010b      ; Set this bit to Enable A20
Dev.A20.Fast.Reset EQU          0000_0001b      ; Set this bit to Reset PC

Dev.A20.Ctrl.Port  EQU          064h
Dev.A20.Ctrl.Read  EQU          0D0h
Dev.A20.Ctrl.Write EQU          0D1h

Dev.A20.Stat.Port  EQU          064h
Dev.A20.Stat.InFull  EQU        0000_0010b
Dev.A20.Stat.OutFull EQU        0000_0001b

Dev.A20.Data.Port  EQU          060h
Dev.A20.Data.Enable EQU         0DFh
</source>
````

<a id="w003522"></a>

## W003522 — User:Johnburger/Demo/Dev/PIC

Bron-ID: W003522
Namespace: User (2)
Revisie: 21414 | Datum: 2017-08-29T09:15:01Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Dev/PIC
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FDev%2FPIC&oldid=21414

````text
There are two Priority Interrupt Controllers on the IBM PC/AT and later computers - a Master and a Slave that I've named PIC.A and PIC.B respectively.

The PICs need to be initialized to get them away from the <em>stupid</em> defaults, but they also need to be reconfigured as new device drivers are installed.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Dev/Dev|Dev]]/PIC.inc</code> ==
<source lang="asm">
;
; Dev/PIC.inc
;

; These are the definitions for the Priority Interrupt Controllers
Dev.PIC.A.Cmd   EQU             020h
Dev.PIC.A.Mask  EQU             021h

Dev.PIC.B.Cmd   EQU             0A0h
Dev.PIC.B.Mask  EQU             0A1h

Dev.PIC.Cmd.EOI   EQU           20h
Dev.PIC.Cmd.IRR   EQU           0Ah             ; Access Interrupt Request Register
Dev.PIC.Cmd.ISR   EQU           0Bh             ; Access In-Service Register

Dev.PIC.Cmd.Init  EQU           11h
Dev.PIC.Init.8086 EQU           01h

Dev.PIC.A.Cascade EQU           0000_0100b      ; PIC.A has slave on IRQ2
Dev.PIC.B.Cascade EQU           02h             ; PIC.B is slaved on IRQ2

Dev.PIC.A.Timer   EQU           0000_0001b
Dev.PIC.A.Key     EQU           0000_0010b
Dev.PIC.A.COM2    EQU           0000_1000b
Dev.PIC.A.COM1    EQU           0001_0000b
Dev.PIC.A.Floppy  EQU           0100_0000b
Dev.PIC.A.Printer EQU           1000_0000b

Dev.PIC.B.RTC     EQU           0000_0001b
Dev.PIC.B.Mouse   EQU           0001_0000b
Dev.PIC.B.IDE0    EQU           0100_0000b
Dev.PIC.B.IDE1    EQU           1000_0000b
</source>
````

<a id="w003523"></a>

## W003523 — User:Johnburger/Demo/Dev/Timer

Bron-ID: W003523
Namespace: User (2)
Revisie: 21415 | Datum: 2017-08-29T09:15:10Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Dev/Timer
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FDev%2FTimer&oldid=21415

````text
There's not much to the Timer chip. The most important thing is the frequency that it is clocked at - the values programmed into it are based off the crystal connected to it.

To save costs, the original IBM PC was designed around a single clock that was used for multiple purposes: the CPU, this timer, and even the video. 4.77 MHz was an odd number, but it could be factored down into the different rates required. This is where the final 18.2 times-a-second value came from in DOS to <em>really</em> complicate keeping track of the current Time-of-Day. And once this value was chosen, all updates to the PC had to perpetuate the odd value.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Dev/Dev|Dev]]/Timer.inc</code> ==
<source lang="asm">
;
; Dev/Timer.inc
;

; These are the definitions for the Programmable Interval Timer

Dev.Timer.Port  EQU             040h

Dev.Timer.ClockFreq EQU         4_772_728 / 4
</source>
````

<a id="w003524"></a>

## W003524 — User:Johnburger/Demo/Dev/Key

Bron-ID: W003524
Namespace: User (2)
Revisie: 21416 | Datum: 2017-08-29T09:15:20Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Dev/Key
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FDev%2FKey&oldid=21416

````text
Programmers are often surprised to find out that the IBM keyboard doesn't send ASCII characters to the PC.

== Key Codes ==
Instead, it sends byte-wide scan codes in a particular format:
* The high bit is <code>0</code> to indicate that the key has been pressed ("make"), and <code>1</code> to indicate that it was released ("break");
* The remaining seven bits are simply a number that superficially resembles its location on the keyboard, with <code>01h</code> being the top left key and (around) <code>50h</code> being the bottom right key. As keyboard layouts changed, these starting values became muddied: it was more important that a moved key kept its original scan code than that it held a rigorous positional value!

== Shift Keys ==
This encoding has the effect of not caring about the current values of the <Shift> keys - the same letter or number key produces the same code regardless of whether <Shift> or <Caps Lock> is currently down. Indeed, these keys send their individual scan codes just like any other key on the keyboard. It is up to the system to keep track of what Shift/Ctrl/Alt/Lock keys are currently pressed, and to interpret the subsequent keypresses appropriately.

== Extended Keys ==
The original IBM PC keyboard has 84 keys. Each was given its own scan code, so for example the Left and Right <Shift> keys each had their own value (<code>2Ah</code> and <code>36h</code> respectively). When the 104-key Extended keyboard came out, they had duplicated some keys and added new ones - for example they added an extra <Ctrl> and <Alt> key on the right hand side, and provided dedicated arrow keys rather than overlaying the numeric keypad. Even later, completely new keys were added that had no equivalents, such as the <Windows> and <Menu> keys.

=== New Keys ===
These latter keys could be assigned brand new scan codes, since they didn't have to be backward compatible - if an old system didn't understand the scan code it could just ignore it. But the repeated keys caused backward compatibility issues that were resolved in an interesting way - but in a way that makes our job harder.

=== Multi-code Keys ===
The duplicated keys couldn't be given new scan codes, since existing software wouldn't understand the codes and that would cripple the new keys' functionality. But giving them the same code meant that new software couldn't distinguish between the original and the duplicate. So instead the designers made the new keys send a double scan code when they were pressed or released. The first byte was a special magic value of 0E0h: it looks like a break code (with the high bit set), but for an unknown key, so old software would ignore it. Then the normal scan code for the old key was sent. For example, the Left <Ctrl> key has a scan code of <code>1Dh</code>, so the new Right <Ctrl> key would send the double-byte sequence <code>0E0h 1Dh</code> (and <code>0E0h 9Dh</code> on release). Old code would only recognize the second byte, while new code could take advantage of the prefix byte to distinguish the duplicate keys.

=== One-key Keys ===
Some of the added keys were designed to be one-key replacements for functions that previously required two keystrokes to invoke: for example, on the original IBM to get the screen to dump to a printer required the user to press <Shift><PrScn>, and to Pause or Break the current program required <Ctrl><NumLock>. When the one-key replacements were added, they distinguished themselves with the magic value as well, but now they generated extended sequences by double-coding the <Shift> key too - Prt Sc generated <code>0E0h 2Ah 0E0h 37h</code> on press and <code>0E0h 0B7h 0E0h 0AAh</code> on release, which corresponded to the original key sequence.

=== Pause/Break ===
For some reason though, <Pause/Break> has its own magic value: <code>0E1h</code>. This value means that there will be <em>two</em> extended scan codes sent - <code>0E1h 1Dh 45h</code> and <code>0E1h 09Dh 0C5h</code> for make and break respectively. This magic value also seems to indicate that the codes won't be auto-repeated, unlike every other key on the keyboard.

== Definitions ==
The following definitions are for the keyboard controller, but they also give scan codes for many of the special keys - not all of which are used by this Demonstrator. The system does display all keypresses though, so it should be easy to add new definitions for values as required.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Dev/Dev|Dev]]/Key.inc</code> ==
<source lang="asm">
;
; Dev/Key.inc
;

; These are the definitions for the Keyboard

Dev.Key.Data    EQU             060h
Dev.Key.Ctrl    EQU             064h

Dev.Key.Make    EQU             000h            ; "Make" codes have high bit off
Dev.Key.Break   EQU             080h            ; "Break" codes have high bit on

Dev.Key.Ext1    EQU             0E0h            ; This is the Extended Key prefix
Dev.Key.Ext2    EQU             0E1h            ; This one means TWO following

; Scan codes for various keys
Dev.Key.Esc     EQU             01h

Dev.Key.Enter   EQU             1Ch
Dev.Key.Space   EQU             39h

Dev.Key.Pause   EQU             45h

Dev.Key.Home    EQU             47h
Dev.Key.Up      EQU             48h
Dev.Key.PgUp    EQU             49h

Dev.Key.Left    EQU             4Bh
Dev.Key.Right   EQU             4Dh

Dev.Key.End     EQU             4Fh
Dev.Key.Down    EQU             50h
Dev.Key.PgDn    EQU             51h

Dev.Key.Ins     EQU             52h
Dev.Key.Del     EQU             53h

Dev.Key.F1      EQU             3Bh
Dev.Key.F2      EQU             3Ch
Dev.Key.F3      EQU             3Dh
Dev.Key.F4      EQU             3Eh
Dev.Key.F5      EQU             3Fh
Dev.Key.F6      EQU             40h
Dev.Key.F7      EQU             41h
Dev.Key.F8      EQU             42h
Dev.Key.F9      EQU             43h
Dev.Key.F10     EQU             44h
Dev.Key.F11     EQU             57h
Dev.Key.F12     EQU             58h

Dev.Key.Window  EQU             5Bh
Dev.Key.Menu    EQU             5Dh
</source>
````

<a id="w003525"></a>

## W003525 — User:Johnburger/Demo/Dev/VGA

Bron-ID: W003525
Namespace: User (2)
Revisie: 21417 | Datum: 2017-08-29T09:15:33Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Dev/VGA
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FDev%2FVGA&oldid=21417

````text
The memory arrangement of the Text mode of the IBM PC is a linear array of two-byte locations. The first memory location is the top left of the screen, followed by 79 (typically) more locations for the first row, immediately followed by the memory for the second row, until all the rows are displayed. Each screen location is two bytes:
* The first byte is the character code, which (luckily) is ASCII for the characters #32-#126 - most text is easy to display. The other codes in the #0-#255 range are often called "OEM" codes - different countries use different codes. The default mapping is used in this system.
* The second byte is that character's attribute - typically its colour. The low nybble is the foreground colour, while the high nybble is the background colour - except for the highest bit, which is often set to flash the character. Many systems though don't do this, instead substituting a wider range of shades of colours.

The colours available to display are essentially the primary RGB and the secondary CMY, with the necessary White and Black added in.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Dev/Dev|Dev]]/VGA.inc</code> ==
<source lang="asm">
;
; Dev/VGA.inc
;

; These are the definitions for the Video Graphics Array (VGA) Controller.
;
; The memory arrangement of the Text mode of the IBM PC is a linear array of
; two-byte locations. The first memory location is the top left of the screen,
; followed by 79 (typically) more locations for the first row, immediately
; followed by the memory for the second row, until all the rows are displayed.
; Each screen location is two bytes:
;
; The first byte is the character code, which (luckily) is ASCII for the
; characters #32-#126 - most text is easy to display. The other codes in the
; #0-#255 range are often called "OEM" codes - different countries use different
; codes. The default mapping is used in this system.
;
; The second byte is that character's attribute - typically its colour. The low
; nybble is the foreground colour, while the high nybble is the background
; colour - except for the highest bit, which is often set to flash the
; character. Many systems though don't do this, instead substituting a wider
; range of shades of colours.
;
; The colours available to display are essentially the primary RGB and the
; secondary CMY, with the necessary White and Black added in. The table for the
; default colour mappings is provided below.

Dev.VGA.Base      EQU           000B_8000h      ; Base memory address

Dev.VGA.Foregruond EQU          0000_1111b
Dev.VGA.Bright     EQU          0000_1000b
Dev.VGA.Background EQU          0111_0000b
Dev.VGA.Blink      EQU          1000_0000b

Dev.VGA.Black     EQU           0_0h
Dev.VGA.Blue      EQU           0_1h
Dev.VGA.Green     EQU           0_2h
Dev.VGA.Cyan      EQU           0_3h
Dev.VGA.Red       EQU           0_4h
Dev.VGA.Magenta   EQU           0_5h
Dev.VGA.Brown     EQU           0_6h
Dev.VGA.Grey      EQU           0_7h

Dev.VGA.Dark          EQU       Dev.VGA.Bright | Dev.VGA.Black
Dev.VGA.BrightBlue    EQU       Dev.VGA.Bright | Dev.VGA.Blue
Dev.VGA.BrightGreen   EQU       Dev.VGA.Bright | Dev.VGA.Green
Dev.VGA.BrightCyan    EQU       Dev.VGA.Bright | Dev.VGA.Cyan
Dev.VGA.BrightRed     EQU       Dev.VGA.Bright | Dev.VGA.Red
Dev.VGA.BrightMagenta EQU       Dev.VGA.Bright | Dev.VGA.Magenta
Dev.VGA.Yellow        EQU       Dev.VGA.Bright | Dev.VGA.Brown
Dev.VGA.White         EQU       Dev.VGA.Bright | Dev.VGA.Grey

Dev.VGA.BlackBack     EQU       Dev.VGA.Black   << 4
Dev.VGA.BlueBack      EQU       Dev.VGA.Blue    << 4
Dev.VGA.GreenBack     EQU       Dev.VGA.Green   << 4
Dev.VGA.CyanBack      EQU       Dev.VGA.Cyan    << 4
Dev.VGA.RedBack       EQU       Dev.VGA.Red     << 4
Dev.VGA.MagentaBack   EQU       Dev.VGA.Magenta << 4
Dev.VGA.YellowBack    EQU       Dev.VGA.Brown   << 4
Dev.VGA.WhiteBack     EQU       Dev.VGA.Grey    << 4

Dev.VGA.Misc.Port EQU           3C2h            ; VGA miscellaneous port

Dev.VGA.Seq.Base  EQU           3C4h            ; VGA Sequencer base port

Dev.VGA.CRTC.Base EQU           3D4h            ; VGA CRTC base port
</source>
````

<a id="w003526"></a>

## W003526 — User:Johnburger/Demo/Ints/Ints

Bron-ID: W003526
Namespace: User (2)
Revisie: 21360 | Datum: 2017-08-28T21:12:40Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Ints/Ints
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FInts%2FInts&oldid=21360

````text
It turned out that the interrupt handler section had more code, and took longer, than the rest of the Demonstrator. That's indicative of how much effort I suggest you should put into handling faults, especially early on while you're getting your startup code finalised.

Displaying the fault code and the register contents is very helpful, so getting that done first proved useful. But then I wanted to be able to see the in-memory tables, to prove to myself that I'd set them up correctly. To that end I decided to write a "debugger" - not for code, but to display the memory contents. It turned out that simply working out what the data was showing me helped me understand how Protected Mode operates - and how [[Expand Down]] Segments work!

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Overview#Interrupt_Handlers|Ints]]/Ints.inc</code> ==
<source lang="asm">
;
; Ints/Ints.inc
;

;-------------------------------------------------------------------------------
%include        "Ints/IDT.inc"          ; First, define an IDT
;-------------------------------------------------------------------------------
%include        "Ints/Vectors.inc"      ; Vectors for each interupt
;-------------------------------------------------------------------------------
%include        "Ints/Fault.inc"        ; Default Fault handling code
;-------------------------------------------------------------------------------
%include        "Ints/Single.inc"       ; Example single-step handler
;-------------------------------------------------------------------------------
%include        "Ints/Debug/Debug.inc"  ; Example debug handler
;-------------------------------------------------------------------------------
%include        "Ints/NoSeg.inc"        ; Example No Segment handler
;-------------------------------------------------------------------------------
%include        "Ints/User.inc"         ; Demonstrate User System Services
;-------------------------------------------------------------------------------
%include        "Ints/IRQs.inc"         ; Vectors for each IRQ
;-------------------------------------------------------------------------------
%include        "Ints/Timer.inc"        ; Example Timer interrupt handler
;-------------------------------------------------------------------------------
%include        "Ints/Key.inc"          ; Example Keyboard interrupt handler
;-------------------------------------------------------------------------------
%include        "Ints/Util.inc"         ; Utility functions
</source>
````

<a id="w003527"></a>

## W003527 — User:Johnburger/Demo/Ints/IDT

Bron-ID: W003527
Namespace: User (2)
Revisie: 21403 | Datum: 2017-08-29T09:09:02Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Ints/IDT
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FInts%2FIDT&oldid=21403

````text
Hard-coding the Interrupt Descriptor Table in data rather than in code saved me vital space in the [[User:Johnburger/Demo/Boot/IDT|Boot]] sector, since I wanted the IDT fully set up (to at the very least display Faults) upon the switch to Protected Mode.

The below table makes extensive use of the [[User:Johnburger/Demo/x86/Desc|Desc]] macros, making for very clean and straightforward source code. It's not so obvious with the [[User:Johnburger/Demo/Exec/GDT|GDT]] code, but they're used there too. As for the [[User:Johnburger/Demo/User/LDT|LDT]]s... nope! Too hard!

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Ints/Ints|Ints]]/IDT.inc</code> ==
<source lang="asm">
;
; Ints/IDT.inc
;

; This is the Interrupt Descriptor Table for the system.
; For each potential interrupt, it points to the relevant routine.
; Note that only the first 32 Intel-reserved interrupts, and the 16 Interrupt
; ReQuest IRQs, are set up. Although sufficient memory has been allocated for
; all 256 interrupts, the IDT is only initialised with these first 48.
;
; EDIT: I decided to add two more, to be called from User mode:
; * INT 48 is a User-mode HLT replacement;
; * INT 49 is a Yield call - go to next Task.

                SEGMENT         IDT  VSTART=0  ALIGN=16

                USE32

IDT.Trap        EQU             Type.Sys(Trap, DPL0, 386)
IDT.Int         EQU             Type.Sys(Int,  DPL0, 386)
IDT.User        EQU             Type.Sys(Trap, DPL3, 386)

IDT:
IDT.Divide      Desc.Sys        GDT.Ints, Ints.Int0, IDT.Trap     ; Int 00h
IDT.Single      Desc.Sys        GDT.Ints, Ints.Int1, IDT.Trap     ; Int 01h
IDT.NMI         Desc.Sys        GDT.Ints, Ints.Int2, IDT.Trap     ; Int 02h
IDT.Debug       Desc.Sys        GDT.Ints, Ints.Int3, IDT.Trap     ; Int 03h
IDT.INTO        Desc.Sys        GDT.Ints, Ints.Int4, IDT.Trap     ; Int 04h
IDT.BOUND       Desc.Sys        GDT.Ints, Ints.Int5, IDT.Trap     ; Int 05h
IDT.BadOp       Desc.Sys        GDT.Ints, Ints.Int6, IDT.Trap     ; Int 06h
IDT.No387       Desc.Sys        GDT.Ints, Ints.Int7, IDT.Trap     ; Int 07h
IDT.Double      Desc.Sys        GDT.Ints, Ints.Int8, IDT.Trap     ; Int 08h
IDT.Bad387      Desc.Sys        GDT.Ints, Ints.Int9, IDT.Trap     ; Int 09h
IDT.BadTSS      Desc.Sys        GDT.Ints, Ints.Int10, IDT.Trap    ; Int 0Ah
IDT.NoSeg       Desc.Sys        GDT.Ints, Ints.Int11, IDT.Trap    ; Int 0Bh
IDT.BadStack    Desc.Sys        GDT.Ints, Ints.Int12, IDT.Trap    ; Int 0Ch
IDT.GPF         Desc.Sys        GDT.Ints, Ints.Int13, IDT.Trap    ; Int 0Dh
IDT.Page        Desc.Sys        GDT.Ints, Ints.Int14, IDT.Trap    ; Int 0Eh
IDT.15          Desc.Sys        GDT.Ints, Ints.Int15, IDT.Trap    ; Int 0Fh
IDT.Float       Desc.Sys        GDT.Ints, Ints.Int16, IDT.Trap    ; Int 10h
IDT.Align       Desc.Sys        GDT.Ints, Ints.Int17, IDT.Trap    ; Int 11h
IDT.Machine     Desc.Sys        GDT.Ints, Ints.Int18, IDT.Trap    ; Int 12h
IDT.SIMD        Desc.Sys        GDT.Ints, Ints.Int19, IDT.Trap    ; Int 13h
IDT.VM          Desc.Sys        GDT.Ints, Ints.Int20, IDT.Trap    ; Int 14h
IDT.21          Desc.Sys        GDT.Ints, Ints.Int21, IDT.Trap    ; Int 15h
IDT.22          Desc.Sys        GDT.Ints, Ints.Int22, IDT.Trap    ; Int 16h
IDT.23          Desc.Sys        GDT.Ints, Ints.Int23, IDT.Trap    ; Int 17h
IDT.24          Desc.Sys        GDT.Ints, Ints.Int24, IDT.Trap    ; Int 18h
IDT.25          Desc.Sys        GDT.Ints, Ints.Int25, IDT.Trap    ; Int 19h
IDT.26          Desc.Sys        GDT.Ints, Ints.Int26, IDT.Trap    ; Int 1Ah
IDT.27          Desc.Sys        GDT.Ints, Ints.Int27, IDT.Trap    ; Int 1Bh
IDT.28          Desc.Sys        GDT.Ints, Ints.Int28, IDT.Trap    ; Int 1Ch
IDT.29          Desc.Sys        GDT.Ints, Ints.Int29, IDT.Trap    ; Int 1Dh
IDT.Security    Desc.Sys        GDT.Ints, Ints.Int30, IDT.Trap    ; Int 1Eh
IDT.31          Desc.Sys        GDT.Ints, Ints.Int31, IDT.Trap    ; Int 1Fh

IDT.Timer       Desc.Sys        GDT.Ints, Ints.IRQ0, IDT.Int      ; Int 20h
IDT.Key         Desc.Sys        GDT.Ints, Ints.IRQ1, IDT.Int      ; Int 21h
IDT.Cascade     Desc.Sys        GDT.Ints, Ints.IRQ2, IDT.Int      ; Int 22h
IDT.COM2        Desc.Sys        GDT.Ints, Ints.IRQ3, IDT.Int      ; Int 23h
IDT.COM1        Desc.Sys        GDT.Ints, Ints.IRQ4, IDT.Int      ; Int 24h
IDT.IRQ5        Desc.Sys        GDT.Ints, Ints.IRQ5, IDT.Int      ; Int 25h
IDT.Floppy      Desc.Sys        GDT.Ints, Ints.IRQ6, IDT.Int      ; Int 26h
IDT.Printer     Desc.Sys        GDT.Ints, Ints.IRQ7, IDT.Int      ; Int 27h
IDT.RTC         Desc.Sys        GDT.Ints, Ints.IRQ8, IDT.Int      ; Int 28h
IDT.IRQ9        Desc.Sys        GDT.Ints, Ints.IRQ9, IDT.Int      ; Int 29h
IDT.IRQ10       Desc.Sys        GDT.Ints, Ints.IRQ10, IDT.Int     ; Int 2Ah
IDT.IRQ11       Desc.Sys        GDT.Ints, Ints.IRQ11, IDT.Int     ; Int 2Bh
IDT.Mouse       Desc.Sys        GDT.Ints, Ints.IRQ12, IDT.Int     ; Int 2Ch
IDT.IRQ13       Desc.Sys        GDT.Ints, Ints.IRQ13, IDT.Int     ; Int 2Dh
IDT.IDE0        Desc.Sys        GDT.Ints, Ints.IRQ14, IDT.Int     ; Int 2Eh
IDT.IDE1        Desc.Sys        GDT.Ints, Ints.IRQ15, IDT.Int     ; Int 2Fh

IDT.Halt        Desc.Sys        GDT.Ints, Ints.Halt, IDT.User     ; Int 30h
IDT.Yield       Desc.Sys        GDT.Ints, Ints.Yield, IDT.User    ; Int 31h
</source>
````

<a id="w003528"></a>

## W003528 — User:Johnburger/Demo/Ints/Vectors

Bron-ID: W003528
Namespace: User (2)
Revisie: 21418 | Datum: 2017-08-29T09:15:44Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Ints/Vectors
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FInts%2FVectors&oldid=21418

````text
These are the interrupt entry points for each of the Intel-reserved exception handlers. Each entry point does the same thing:
* <code>PUSH</code>es a pseudo-fault code on the stack (if the exception didn't put a real one there already);
* <code>PUSH</code>es the interrupt number onto the stack for display purposes;
* <code>JMP</code>s to the same [[User:Johnburger/Demo/Ints/Fault|Fault handler]] to save the current state and display it on the screen.

Note that this best-effort initial setup for the IDT doesn't cater for the special interrupts that Intel recommends be handled by full-blown Tasks. Those are set up during Protected Mode initialization, replacing the appropriate ones of these. Until that happens, the system is not able to handle all fault conditions.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Ints/Ints|Ints]]/Vectors.inc</code> ==
<source lang="asm">
;
; Ints/Vectors.inc
;

; This file holds the entry points for all of the Intel-reserved interrupts.
; They all jump to the same handler, but not before saving away the number
; of the interrupt - oh, and a pseudo-fault code if one isn't already generated.

; To save space, the handler has been put in the middle of the vectors, to
; allow the assembler to use the shorter form of the JMP instruction.

                SEGMENT    Ints  VSTART=0  ALIGN=16

                USE32

Ints.Int0:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    0       ; Interrupt number
                JMP             Ints.Vector
Ints.Int1:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    1       ; Interrupt number
                JMP             Ints.Vector
Ints.Int2:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    2       ; Interrupt number
                JMP             Ints.Vector
Ints.Int3:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    3       ; Interrupt number
                JMP             Ints.Vector
Ints.Int4:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    4       ; Interrupt number
                JMP             Ints.Vector
Ints.Int5:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    5       ; Interrupt number
                JMP             Ints.Vector
Ints.Int6:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    6       ; Interrupt number
                JMP             Ints.Vector
Ints.Int7:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    7       ; Interrupt number
                JMP             Ints.Vector
Ints.Int8:
;               PUSH   DWORD    0       ; Pseudo-fault code (not required)
                PUSH   DWORD    8       ; Interrupt number
                JMP             Ints.Vector
Ints.Int9:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    9       ; Interrupt number
                JMP             Ints.Vector
Ints.Int10:
;               PUSH   DWORD    0       ; Pseudo-fault code (not required)
                PUSH   DWORD    10      ; Interrupt number
                JMP             Ints.Vector
Ints.Int11:
;               PUSH   DWORD    0       ; Pseudo-fault code (not required)
                PUSH   DWORD    11      ; Interrupt number
                JMP             Ints.Vector
Ints.Int12:
;               PUSH   DWORD    0       ; Pseudo-fault code (not required)
                PUSH   DWORD    12      ; Interrupt number
                JMP             Ints.Vector
Ints.Int13:
;               PUSH   DWORD    0       ; Pseudo-fault code (not required)
                PUSH   DWORD    13      ; Interrupt number
                JMP             Ints.Vector
Ints.Int14:
;               PUSH   DWORD    0       ; Pseudo-fault code (not required)
                PUSH   DWORD    14      ; Interrupt number
                JMP             Ints.Vector
Ints.Int15:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    15      ; Interrupt number
;               JMP             Ints.Vector

Ints.Vector     JMP             Ints.Fault

Ints.Int16:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    16      ; Interrupt number
                JMP             Ints.Vector
Ints.Int17:
;               PUSH   DWORD    0       ; Pseudo-fault code (not required)
                PUSH   DWORD    17      ; Interrupt number
                JMP             Ints.Vector
Ints.Int18:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    18      ; Interrupt number
                JMP             Ints.Vector
Ints.Int19:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    19      ; Interrupt number
                JMP             Ints.Vector
Ints.Int20:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    20      ; Interrupt number
                JMP             Ints.Vector
Ints.Int21:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    21      ; Interrupt number
                JMP             Ints.Vector
Ints.Int22:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    22      ; Interrupt number
                JMP             Ints.Vector
Ints.Int23:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    23      ; Interrupt number
                JMP             Ints.Vector
Ints.Int24:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    24      ; Interrupt number
                JMP             Ints.Vector
Ints.Int25:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    25      ; Interrupt number
                JMP             Ints.Vector
Ints.Int26:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    26      ; Interrupt number
                JMP             Ints.Vector
Ints.Int27:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    27      ; Interrupt number
                JMP             Ints.Vector
Ints.Int28:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    28      ; Interrupt number
                JMP             Ints.Vector
Ints.Int29:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    29      ; Interrupt number
                JMP             Ints.Vector
Ints.Int30:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    30      ; Interrupt number
                JMP             Ints.Vector
Ints.Int31:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    31      ; Interrupt number
                JMP             Ints.Vector
</source>
````

<a id="w003529"></a>

## W003529 — User:Johnburger/Demo/Ints/Fault

Bron-ID: W003529
Namespace: User (2)
Revisie: 21419 | Datum: 2017-08-29T09:15:54Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Ints/Fault
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FInts%2FFault&oldid=21419

````text
The following code uses the <code>[[User:Johnburger/Demo/Ints/Util|Ints.Names]]</code> table to display all the register names, fetch their values from the relevant locations, and display them in Hex.

When the register data is on the stack, it is called a "Frame". It was quite complicated to establish the Frame in a standardized manner, and some hoops were jumped through to get the memory layout consistent so that different display code could use the same <code>Ints.Names</code> table. To allow that to happen, some extra routines were provided that other code could call to ensure this consistency: see <code>.Frame.Create</code> and <code>.Frame.Destroy</code>.

And then <code>.ShowRegs</code> itself had to handle fetching the register values from the correct location: either the stack frame or from a TSS alias. The <code>Ints.Names</code> table has both indexes.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Ints/Ints|Ints]]/Fault.inc</code> ==
<source lang="asm">
;
; Ints/Fault.inc
;

; This is the general-purpose Fault handler. It simply writes out as much
; information as it can find onto the screen, to allow poor you to work out
; where the problem is!

Ints.Fault:
                PUSHAD                          ; Save all registers
                CALL            .Frame.Create   ; Create Stack Frame for display

                XOR             AX,AX           ; No TSS!
                MOV             FS,AX

                MOV             EDI,[ESP+64]    ; Load Interrupt number
                CALL            .ShowRegs

                CALL            .Frame.Destroy  ; Destroy created Stack Frame
                POPAD                           ; Note this doesn't replace ESP!
                ADD             ESP,8           ; Ignore Int and Error values
                IRETD                           ; And return
;...............................................................................
; This function pushes registers on the stack for display purposes - oh, and so
; I can use some of them! To standardise the stack layout and make the Ints.Names
; table consistent, other code may call it.
;
; Of course, the trick with using a function to save registers is that you call
; the function, then return from it, popping the values you just saved. But with
; assembly, you can perform a trick: POP the return address first, save what
; needs saving, PUSH the return address, and then RET. Actually, the last two
; steps can be done away with: just JMP to the saved register!

Ints.Fault.Frame.Create:
                POP             EDX             ; Pop off return address

                PUSH            DS              ; So these values can go there
                PUSH            ES
                PUSH            FS
                PUSH            GS
                PUSH            SS

                TEST    WORD    [ESP+64],x86.Seg.RPL3 ; From different Privilege Level?
                JZ              .SamePL         ; No

                MOV             EAX,[ESP+76]    ; Yes, so get old SS
                MOV             [ESP],EAX       ; Replace my value
                MOV             EAX,[ESP+72]    ; Get old ESP
                MOV             [ESP+32],EAX    ; Replace my value
.SamePL:
                STR             AX              ; Include faulting task
                PUSH            EAX
                SLDT            AX              ; And faulting LDT
                PUSH            EAX
                MOV             EAX,CR3         ; And Page Directory Base Register
                PUSH            EAX

                JMP             EDX             ; Return
;...............................................................................
Ints.Fault.Frame.Destroy:
                POP             EDX             ; Pop off return address

                ADD             ESP,16          ; Ignore saved CR3, LDT, TR and SS
                POP             GS              ; Pop everything else
                POP             FS
                POP             ES
                POP             DS

                JMP             EDX             ; Return
;...............................................................................
; This function is called externally (FAR), so that it can perform the NEAR call

Ints.Fault.External:
                CALL            Ints.Fault.ShowRegs
                RETF
;...............................................................................
Ints.Fault.ShowRegs:
                MOV             EBP,ESP         ; Save for display purposes

                CALL            Ints.System.Init
                PUSH            EAX

                PUSH            CS
                POP             DS              ; Point to code, for Ints.Names

                MOV             AX,Selector(GDT.VGA, GDT, RPL0)
                MOV             ES,AX           ; Point to screen with ES

                MOV             ESI,Ints.Names  ; Point to Names
                SHL             EDI,1           ; Turn IntNum into screen address
                INC     BYTE    [ES:EDI]        ; Dingle(tm) that this INT happened

                MOV             AH,Dev.VGA.RedBack | Dev.VGA.White ; White-on-red!
                ADD             EDI,VGA.Cols * 2 ; On next row

                CLD                             ; Work forwards
.Dump:
                MOV             ECX,Ints.Names.Label ; Width of label; zero high
                INC             ESI             ; Ignore Debug pos
.Name:
                LODSB                           ; Get character
                STOSW                           ; Store attrib+char
                LOOP            .Name           ; For each character
                MOV             AL,':'
                STOSW

                LODSB                           ; Get size of register
                MOV             CL,AL           ; Into counter

                MOV             EDX,FS          ; From TSS?
                TEST            EDX,EDX         ; Defined?
                JNZ             .InTSS          ; Yes. Go look at it
.OnStack:
                LODSB                           ; Get position on stack
                INC             ESI             ; Ignore position in TSS
                MOVZX           EDX,AL          ; Into memory offset
.FromStack:
                MOV             EDX,[EBP+EDX]   ; Get value from stack
                JMP             .Show
.InTSS:
                INC             ESI             ; Ignore Stack position
                LODSB                           ; Get position in TSS (FS)
                MOVZX           EDX,AL          ; Into memory offset
                CMP             AL,x86.TSS.CR3  ; Down low?
                JB              .FromStack      ; Yes. Must be on stack instead
                MOV             EDX,[FS:EDX]    ; No. So get value to show
.Show:
                PUSH            EDI             ; Save starting location
                CALL            Ints.Hex        ; Display as Hex
                POP             EDI
                ADD             EDI,(VGA.Cols-Ints.Names.Label-1)*2 ; Next row

                CMP             ESI,Ints.Names.End ; End of registers?
                JB              .Dump           ; No, so continue

                HLT                             ; Nothing more to do
; If interrupts are disabled, the HLT will "zombie" the PC, requiring a power
; off. VMware detects this condition, but it lets you look at the screen first!
; If interrupts are enabled, one will wake us up and continue from here, which
; will then return and resume the faulting instruction (presumably). Note that
; it may also invoke a fault-"storm", generating new faults that will overwrite
; what's just been written. In other words, what you see is the last thing that
; happened, not the first! To only see the first, uncomment the following line;
;               JMP             $-1
; Here's an example of what could happen: Say the User Code forgets to RET from
; a function, and that function is at the end of the Code Segment. The next
; instruction fetch will be past the end of CS, so a GPF will be raised. All of
; this .Fault code will display the problem (it might take you a bit to work out
; that CS has overflowed, but anyway...) and then this code will clean up and
; IRET to the problem area. But that's past CS, so now it is the .Fault's IRET
; that's the culprit! A new GPF will be generated, displaying the new details,
; blaming this very code!
                POP             EBX
                CALL            Ints.System.Done
                RET
</source>
````

<a id="w003530"></a>

## W003530 — User:Johnburger/Demo/Ints/Single

Bron-ID: W003530
Namespace: User (2)
Revisie: 21420 | Datum: 2017-08-29T09:16:11Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Ints/Single
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FInts%2FSingle&oldid=21420

````text
Single-stepping is a very useful debugging technique, but it's really only useful when you can see what's happening - or even better, what's about to happen. So the following code is only provided as an example of using and invoking the [[User:Johnburger/Demo/x86/EFlags|Trace Flag]].

More sophisticated code would reference the <code>PUSH</code>ed <code>CS:EIP</code>, prepare a Data alias for the Code segment, and actually present disassembled code. That is left as an exercise for the reader!

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Ints/Ints|Ints]]/Single.inc</code> ==
<source lang="asm">
;
; Ints/Single.inc
;

; This module provides an example Single-Step handler interrupt.
; To invoke it, enable Single Step Interrupt with the "TRACE" macro.
; It waits for a keypress:
; If it's a Space, it simply returns - the next instruction will stop again.
; If it's Enter, it returns too - but first, it disables the Trace Flag to
; stop future interrupts.
;
; A good place to put the "TRACE" macro is inside the User code, just as the
; Frame is being drawn. Then, EVERY Task will be in Single Step mode! To
; continue the Tasks, merely hold down the Space key, and watch them slowly
; build their Frames, and perhaps start to bounce their balls.
;
; If you press Enter, one of the Tasks will resume as normal. If you hold down
; Enter, more and more of the Tasks will resume...

Ints.Single:

.Key.Continue   EQU             Dev.Key.Space
.Key.Stop       EQU             Dev.Key.Enter

                PUSH            EAX             ; Need these registers
                PUSH            DS
                PUSH            ES

                MOV             AX,Selector(GDT.Data, GDT, RPL0) ; Point to Data
                MOV             DS,AX

                MOV             AX,Selector(GDT.VGA,  GDT, RPL0) ; Point to Screen
                MOV             ES,AX
.Loop:
                INC     BYTE    [ES:0002h]       ; Dingle(tm) screen location

                HLT                              ; Wait for interrupt

                MOV             AL,0             ; Consume any keypress
                XCHG            [Data.Key.Code],AL
                CMP             AL,.Key.Continue ; Continue?
                JE              .Finish          ; Yes, so leave
                CMP             AL,.Key.Stop     ; Stop?
                JNE             .Loop            ; No, so keep waiting

                AND     WORD    [ESP+14h],~x86.EFlags.TF; Yes, so stop tracing
.Finish:
                POP             ES
                POP             DS
                POP             EAX
                IRETD
</source>
````

<a id="w003531"></a>

## W003531 — User:Johnburger/Demo/Ints/NoSeg

Bron-ID: W003531
Namespace: User (2)
Revisie: 21421 | Datum: 2017-08-29T09:16:21Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Ints/NoSeg
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FInts%2FNoSeg&oldid=21421

````text
This is a simple example of what can be done with various Fault handlers in the system.
__TOC__
In this case, if a Descriptor in one of the Descriptor tables has its <code>.Type.Present</code> bit zero, the CPU will raise a Segment Not Present exception so that the system can do something about it.

=== Testing ===
I have marked some of the [[User:Johnburger/Demo/User/LDT|User Task]] segments as Not Present, to invoke this handler. They're fully set up, so all that this handler has to do is mark the Descriptor as Present and return.

=== Not Stack Segments ===
A caution: this handler is '''''not''''' called for Stack Segments that are marked as Not Present. Since the fault revolves around a Stack, the CPU doesn't trust the stack it currently has and calls the Stack Fault exception handler instead. Intel recommends that that fault be handled by a full-blown Task rather than a mere routine.

=== Surprise! ===
And ''this'' is why I wrote this Demonstrator! Different environments (unsurprisingly) behave differently. It works in VMware, but not in VirtualBox, nor on a Core i7 directly. What happens is this:
* The TSS has one Descriptor marked as Not Present - say <code>CS</code>.
* When the TSS is switched to, the CPU starts to load and validate the registers from the TSS.
* It notes that <code>CS</code> is Not Present, and calls this handler with an Error Code for the Selector.
* This handler sets the Descriptor's <code>.Present</code> flag.
* As this handler returns, it <code>POP</code>s <code>CS</code>, which re-loads the (now-present) Descriptor.
* The new Task resumes - but there's a problem! The TSS load was incomplete: it was interrupted by the Not Present Fault. <code>ES</code> in the TSS has a valid Selector (in the User Task case, a Selector to the Screen segment), but the actual Segment Register still has a zero in it!
* The first attempt to use ES faults with a GPF.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Ints/Ints|Ints]]/NoSeg.inc</code> ==
<source lang="asm">
;
; Ints/NoSeg.inc
;

; This module gives an example of a "No Segment" Fault handler.
; Of course, a real handler would have to load the missing Segment from
; somewhere - this one merely assumes that we "forgot" to mark it as .Present,
; fixes it, and resumes.

Ints.NoSeg:                                      ; Oops, forgot to mark as Present!
;               PUSH            0                ; PUSH Error Code - no, already there!
                PUSH            EBX
                PUSH            DS

                MOV             BX,Selector(GDT.VGA, GDT, RPL0)
                MOV             DS,BX
                INC     BYTE    [0016h]          ; Dingle(tm) on screen - show we've done it1

                MOV             EBX,[ESP+8]      ; Get error code
                AND             EBX,x86.Seg.TI   ; Isolate which DT it's in
                OR              BX,DT.Alias      ; Same Alias position in all DTs
                MOV             DS,BX            ; Point to relevant DT Alias

                MOV             EBX,[ESP+8]      ; Get error code (again)
                AND             BX,x86.Seg.Index ; Ignore extra bits
                OR      BYTE    [BX+x86.Desc.Type],x86.Desc.Type.Present ; 16-bit index!

                POP             DS
                POP             EBX
                ADD             ESP,4           ; Skip Error code
                IRETD                           ; And restart instruction
</source>
````

<a id="w003532"></a>

## W003532 — User:Johnburger/Demo/Ints/User

Bron-ID: W003532
Namespace: User (2)
Revisie: 21443 | Datum: 2017-08-29T09:31:55Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Ints/User
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FInts%2FUser&oldid=21443

````text
To demonstrate one technique of allowing User Tasks to request functions from the Executve, I have added two <code>INT</code>s:
* A <code>HLT</code> implementation, since <code>HLT</code> is a Supervisor-only instruction, and User Mode code gets a GPF if it tries it;
* A <code>Yield</code> implementation, allowing the User Mode code to give up the rest of its time-slice.

Note that these two functions are small, since they're essentially only calling functions that are already implemented. Also note that they're single-use INTs: I could have used a register as a Function holder, similar to the way that BIOS calls use <code>AH</code>, but I've left that as an exercise for the reader!

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Ints/Ints|Ints]]/User.inc</code> ==
<source lang="asm">
;
; Ints/User.inc
;

; This module demonstrates providing System Services to a User-mode program.
; The interrupts have been given a DPL of 3, to allow the User program access.

;...............................................................................
; HLT is a privileged instruction, since it can lock up the PC. But for demo
; purposes, an interrupt has been set up for the User program to call.

Ints.Halt:
                HLT
                IRETD                   ; That was easy!
;...............................................................................
; This interrupt does the same Task Switch that happens on the Timer call.
Ints.Yield:
                CLI                     ; Disable interrupts!
                PUSH            EAX
                CALL            Ints.Switch
                POP             EAX
                IRETD
</source>
````
