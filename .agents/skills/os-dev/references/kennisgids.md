# OS Development — thematische kennisgids

Deze gids is een Nederlandstalige navigatie- en samenvattingslaag boven de volledig opgenomen bronnen. Het is geen vervanging van de broninhoud. De onderwerpen zijn redactioneel gegroepeerd; de gelinkte artikelen behouden hun eigen terminologie, ontwerpkeuzes, voorbeelden en revisiedatum. Onzekerheden, mogelijke fouten en historische aannames zijn niet stilzwijgend gecorrigeerd.

## Bronnen lezen

Gebruik een W-nummer voor een wiki-pagina en een B-nummer voor een fysieke PDF-pagina. Zoek hetzelfde nummer via de pagina-index en de volledige bronbestanden of open de lokale bron. Een bronverwijzing is een vindplaats, geen garantie dat alle code of uitspraken daarin juist zijn. Raadpleeg bij een inhoudelijke tegenspraak de originele tekst, niet alleen deze samenvatting.

## Start, voorkennis en ontwikkelvolgorde

Begin bij de vereiste programmeer- en hardwarekennis, de keuze van doelplatform en de eerste controleerbare boot. De wiki heeft zowel minimale bare-bonesvoorbeelden als uitgebreidere projectskeletten. De volgorde van het boek loopt van boot en uitvoer via interrupts en geheugen naar gebruikersprogramma’s, bestandssystemen, systeemaanroepen en multitasking.

Broningangen: [W001748: Getting Started]; [W003073: Required Knowledge]; [W001879: Beginner Mistakes]; [W001577: What Order Should I Make Things In?]; [W002497: Bare bones]; [W003448: Meaty Skeleton].

Boek: p. 7–17, p. 19–22.

## Kernelontwerp en systeemarchitectuur

De wiki onderscheidt onder meer monolithische, modulaire, microkernel-, hybride en exokernelontwerpen. De microkernelpagina bespreekt diensten in gebruikersruimte, IPC, contextwissels, bootstrapafhankelijkheden en expliciet het punt dat herstelbaarheid extra ontwerpwerk vereist. Een model is een ontwerpkeuze, geen automatisch bewijs van veiligheid.

Broningangen: [W002697: Kernel]; [W001411: Microkernel]; [W002298: Monolithic kernel]; [W001906: Modular Kernel]; [W002695: Hybrid Kernel]; [W001465: Exokernel]; [W003680: Object oriented OS].

## Programmeertalen en runtime

Dit deel bundelt de talenoverzichten, assembly, freestandingprogrammering, runtimeondersteuning en standaardbibliotheken. Het boek kiest C met NASM en gebruikt assembly voor processorgerichte handelingen; die keuze is een eigenschap van de bron, niet een verplichting voor een ander OS.

Broningangen: [W001680: Languages]; [W001425: Assembly]; [W001803: C]; [W001528: C++]; [W003415: Rust]; [W003725: Implications of writing a freestanding C project]; [W003192: C Library].

Boek: p. 11–12, p. 19–22, p. 67–68.

## Compilers, assemblers, linkers en builds

De bronpagina’s verbinden cross-compilatie met het gewenste doelplatform, objectbestanden, ABI’s, linkerscripts en builds. Het boek laat zien hoe een entrysymbol, secties en geheugenadressen samen het kernelbestand vormen. Bewaar het onderscheid tussen hostgereedschap en het besturingssysteem dat het geproduceerde bestand gaat uitvoeren.

Broningangen: [W001448: GCC Cross-Compiler]; [W002968: LLVM Cross-Compiler]; [W002604: Linker Scripts]; [W001759: Calling Conventions]; [W003507: System V ABI]; [W002864: Object Files]; [W001785: Makefile].

Boek: p. 13–16, p. 19–22, p. 57–58, p. 61–62.

## Bootproces, firmware en bootloaders

De kennisbank bevat afzonderlijke routes voor BIOS, UEFI, bestaande bootloaders, eigen loaders en bootprotocollen. Het boek gebruikt BIOS, GRUB Legacy, Multiboot en een 32-bit kernel; de UEFI-, Limine- en 64-bitpagina’s staan als aparte bronnen naast die leerlijn, niet als stilzwijgende vervanging ervan.

Broningangen: [W003050: Boot sequence]; [W001682: Bootloader]; [W001484: BIOS]; [W002924: UEFI]; [W002803: GRUB]; [W004530: Limine]; [W003350: Multiboot]; [W004562: OVMF].

Boek: p. 11–17, p. 47–49.

## x86, processormodi en descriptoren

Dit deel bevat registers, modi, descriptoren, segmentatie, processoreigenschappen en uitbreidingen. Het boek beschrijft selector, descriptor, basisadres, limiet en lineair adres voor zijn 32-bitvoorbeeld. De wiki behandelt daarnaast x86-64; bewaar steeds welke processormodus en datastructuurlay-out een bron bedoelt.

Broningangen: [W001897: IA32 Architecture Family]; [W001447: X86-64]; [W001537: Global descriptor table]; [W001488: Segmentation]; [W002018: Task State Segment]; [W002814: Setting Up Long Mode]; [W001450: CPUID].

Boek: p. 33–37, p. 65–66.

## Exceptions, interrupts en timers

De bronnen behandelen interrupttabellen, handlers, processor-exceptions, interruptcontrollers en tijdbronnen. Het boek bouwt een IDT, bespreekt handlerstubs en foutcodes, en gebruikt PIC en PIT. De wiki bevat aanvullende controller- en architectuurvarianten; verwissel deze niet alsof ze dezelfde registers en regels hebben.

Broningangen: [W001548: Interrupt descriptor table]; [W001719: Interrupt Service Routines]; [W002400: Exceptions]; [W001478: 8259 PIC]; [W002956: Apic]; [W002434: IOAPIC]; [W001561: Programmable interval timer]; [W003378: HPET].

Boek: p. 39–45, p. 74–75.

## Geheugenkaarten, paging en allocatie

Maak onderscheid tussen fysieke frames, virtuele adresruimten en kleinere heapallocaties. Het boek behandelt adresvertaling, paginatabellen, toegangsvlaggen, identity mapping, higher-halfmapping, gereserveerde gebieden, frameadministratie en tijdelijke mappings. De wikipagina’s voegen alternatieve allocatoren en architectuurspecifieke uitwerkingen toe.

Broningangen: [W001536: Memory Management]; [W002278: Memory Map (x86)]; [W002279: Detecting Memory (x86)]; [W002122: X86 Paging]; [W001461: Page Frame Allocation]; [W001460: Memory Allocation]; [W003452: Heap]; [W001868: Higher Half Kernel].

Boek: p. 51–59, p. 61–63.

## Processen, threads en scheduling

De procespagina’s onderscheiden een programma-in-uitvoering met zijn resources van de afzonderlijke uitvoeringsstromen. Het boek gaat van een geladen module naar gebruikersruimte, daarna naar coöperatieve en preëmptieve scheduling. Registers, adresruimte en stacks horen bij de toestand die de implementatie moet beheren.

Broningangen: [W001415: Processes and Threads]; [W004247: Thread]; [W002603: Getting to Ring 3]; [W001418: Context Switching]; [W001477: Scheduling Algorithms]; [W001797: Blocking Process]; [W001888: Multiprocessor Scheduling].

Boek: p. 47–49, p. 65–68, p. 73–75.

## Synchronisatie, SMP en geheugenordening

De wiki groepeert wederzijdse uitsluiting, rendezvous, readers/writerstoegang, semaforen, spinlocks, atomaire bewerkingen en geheugenbarrières. Zij bespreekt naast correctheid ook fairness en starvation. Het boek behandelt geen multicore-OS; zijn uitleg van preëmptie is daarom geen volledige SMP-handleiding.

Broningangen: [W001889: Synchronization Primitives]; [W001766: Atomic operation]; [W002494: Spinlock]; [W001752: Semaphore]; [W001729: Deadlock]; [W002499: Memory barriers]; [W002671: Multiprocessing].

Boek: p. 73–75.

## Systeemaanroepen en communicatie tussen processen

Dit deel bundelt systeemaanroepen, message passing, gedeeld geheugen en RPC. Het boek bespreekt een syscall-API, een software-interrupt als overdracht en een TSS voor de overgang naar een kernelstack. De wiki bevat andere overdrachtsmechanismen en IPC-ontwerpen; hun ABI’s blijven afzonderlijke keuzes.

Broningangen: [W001961: System Calls]; [W003327: Sysenter]; [W001711: Message Passing]; [W003424: IPC Data Copying methods]; [W001765: Shared Memory]; [W001763: Remote Procedure Call]; [W001890: Signals].

Boek: p. 71–72.

## Programmaformaten, loaders en bibliotheken

De bronnen onderscheiden uitvoerbare formaten, objectbestanden, relocatie, loaders en bibliotheken. Het boek begint met een flat binary en laat ELF-ondersteuning voor gebruikersprogramma’s als uitbreiding open. De latere ELF- en loaderpagina’s leveren de informatie om die stap gericht uit te werken.

Broningangen: [W001542: ELF]; [W003429: ELF Tutorial]; [W001954: Executable Formats]; [W001892: PE]; [W002811: COFF]; [W003824: Execution Loader]; [W002367: Loading a Process]; [W004000: Dynamic Linker].

Boek: p. 47–49, p. 67–68.

## Apparaatbeheer, buses, DMA en IOMMU

De wiki behandelt detectie, apparaatinterfaces, port-I/O, memory-mapped I/O, PCI, DMA en IOMMU-varianten. Het boek introduceert drivers via scherm- en seriële uitvoer: overige code gebruikt een driverinterface in plaats van rechtstreeks alle registers te kennen.

Broningangen: [W002355: Device Management]; [W002037: Hardware Abstraction Layer]; [W003794: Pci]; [W002829: PCI Express]; [W003135: I/O ports]; [W003198: Memory mapped registers in C/C++]; [W001467: ISA DMA]; [W005001: IOMMU]; [W003632: VirtIO].

Boek: p. 23–31.

## Platformdetectie, ACPI en energiebeheer

Platformtabellen en firmwarebeschrijvingen staan hier bij elkaar: ACPI, AML, relevante tabellen, devicetree, detectie en uitschakelen of herstarten. Dit is een wiki-uitbreiding op het boek; de hoofdstukken van het boek vormen geen uitgewerkt modern energiebeheer- of firmwareframework.

Broningangen: [W002946: Acpi]; [W003184: AML]; [W001440: RSDP]; [W002712: RSDT]; [W003319: XSDT]; [W002812: FADT]; [W003259: MADT]; [W004422: Devicetree]; [W001845: Shutdown]; [W001844: Reboot].

## Blockapparaten en schijfstructuren

Dit onderdeel scheidt block-I/O en schijfindeling van bestandssystemen. De wiki bevat ATA/ATAPI, AHCI, NVMe, SCSI, sectoradressering, partities en diskimages. Een bestandssysteem bovenop een apparaat en het protocol waarmee blokken worden gelezen zijn verschillende lagen.

Broningangen: [W002239: ATA PIO Mode]; [W002241: ATA/ATAPI using DMA]; [W002674: AHCI]; [W004432: NVMe]; [W004521: SCSI]; [W002970: Logical Block Addressing]; [W002868: Partition table]; [W003126: GPT]; [W001606: Disk Images].

## Bestandssystemen, VFS en consistentie

Het boek begint met een read-onlybestandssysteem in een RAM-image en introduceert daarna inodes en VFS. De wiki bespreekt concrete formaten, vrije ruimte, naamgeving, metadata, caches, journaling en eigen bestandssystemen. De tekst blijft per formaat herkenbaar: specificaties van verschillende formaten worden niet samengevoegd.

Broningangen: [W001505: File Systems]; [W001859: File Management]; [W002957: Hierarchical VFS Theory]; [W003194: Initrd]; [W001506: FAT]; [W001509: Ext2]; [W003000: Journaling]; [W003638: Roll Your Own Filesystem].

Boek: p. 69–70.

## Netwerkstack, protocollen en netwerkkaarten

De wiki bevat zowel netwerkhardware als de opbouw van een netwerkstack en afzonderlijke protocollen. Ze zijn hier gezamenlijk vindbaar maar blijven aparte bronartikelen. Het boek bevat geen eigen hoofdstuk over een netwerkstack; de inhoud van dit onderdeel komt daarom uit de wiki.

Broningangen: [W002538: Network Stack]; [W002539: Address Resolution Protocol]; [W002979: Internet Control Message Protocol]; [W002362: Ethernet devices]; [W003751: Socket]; [W002626: 802.11]; [W002789: Intel 8254x]; [W002176: RTL8139].

## USB, toetsenbord, muis en seriële I/O

Het boek gebruikt toetsenbordscancodes, PIC-interrupts en seriële uitvoer. De wiki vult dit aan met PS/2-controllerdetails, muizen, USB en hostcontrollervarianten. Scancodes, tekens en invoerinterfaces blijven afzonderlijke begrippen in de bronstructuur.

Broningangen: [W001900: PS/2 Keyboard]; [W003157: PS/2 Mouse]; [W003160: I8042 PS/2 Controller]; [W002840: Serial ports]; [W001725: Universal Serial Bus]; [W003309: EXtensible Host Controller Interface]; [W003303: Enhanced Host Controller Interface].

Boek: p. 23–31, p. 43–45.

## Video, grafische uitvoer, fonts en GUI

De wiki behandelt tekstuitvoer, lineaire framebuffers, videomodi, fonts, buffering, compositing en GUI’s, plus specifieke GPU’s. Het boek noemt zijn VGA-tekstuitvoer een framebuffer; die terminologie blijft in de bron behouden. Een tekstconsole en een volledige desktopomgeving zijn verschillende ontwikkelniveaus.

Broningangen: [W001697: VGA Hardware]; [W004597: Drawing In a Linear Framebuffer]; [W004512: GOP]; [W002489: Double Buffering]; [W003354: Compositing]; [W004024: Graphics stack]; [W002346: GUI]; [W003999: PC Screen Font].

Boek: p. 23–26.

## Audio en geluidshardware

Audio is in deze verzameling een wiki-onderwerp: algemene geluidsuitvoer en concrete hardwarefamilies staan naast elkaar. De beschikbaarheid van een driverartikel betekent niet dat een doelcomputer die chip heeft of dat een programma voor een andere chip zonder aanpassing werkt.

Broningangen: [W002238: Sound]; [W002248: PC Speaker]; [W003314: AC97]; [W003313: Intel High Definition Audio]; [W002237: Sound Blaster 16]; [W003840: Ensoniq AudioPCI].

## Beveiliging, isolatie en betrouwbaarheid

De beveiligingspagina’s verbinden privilegegrenzen met geheugenbescherming en hogere beveiligingskeuzes. De microkernelbron benadrukt dat herstel en behoud van data aanvullende maatregelen vragen. Deze kennisbank is bronmateriaal en levert geen bewijs dat een specifieke implementatie foutvrij, onkraakbaar of formeel geverifieerd is.

Broningangen: [W002262: Security]; [W001411: Microkernel]; [W005001: IOMMU]; [W002935: Random Number Generator]; [W002628: Stack Smashing Protector]; [W001452: CPU Bugs].

Boek: p. 65–66, p. 71–72.

## Testen, debuggen en foutdiagnose

Het boek werkt in kleine stappen en controleert de kernel in een emulator; later wordt seriële uitvoer een logkanaal. De wiki bevat emulatorgebruik, GDB, unit tests en bekende fouten in tutorials. Neem loguitvoer, symbolen, gekozen machineconfiguratie en exacte bronrevisie mee bij een reproduceerbare foutbeschrijving.

Broningangen: [W001946: Testing]; [W001918: Unit Testing]; [W001947: Kernel Debugging]; [W001544: GDB]; [W002941: Qemu]; [W001556: Bochs]; [W001540: Troubleshooting]; [W003459: James Molloy's Tutorial Known Bugs].

Boek: p. 8–8, p. 16–17, p. 26–31.

## ARM, AArch64, RISC-V en overige platforms

Niet-x86-architecturen blijven volwaardig opgenomen: onder andere ARM/AArch64, RISC-V, PowerPC, MIPS, SPARC en S390. Hun handleidingen, interruptmodellen, geheugenbeheer en bare-bonesprojecten zijn niet afgeleid van de 32-bit x86-code in het boek.

Broningangen: [W002332: ARM Overview]; [W004267: AArch64]; [W004033: RISC-V]; [W003306: Raspberry Pi]; [W001834: PowerPC Overview]; [W001832: MIPS Overview]; [W003045: Sparc Hardware Docs]; [W005085: IBM Mainframe architecture].

## Naslag, hulpmiddelen, porting en community

Hier staan naslagwerken, projectvoorbeelden, porting, aanvullende computerwetenschap en community-informatie. Verwijzingen naar externe handleidingen zijn behouden als verwijzingen; de volledige externe documenten zijn niet automatisch onderdeel van een wiki-export.

Broningangen: [W001368: Resources]; [W001572: Books]; [W003903: Academic Papers]; [W001885: Licensing]; [W003405: Code Management]; [W003402: Portability]; [W003628: Cross-Porting Software]; [W001367: Projects].

