# M1: besloten bootcontract en bouwprofiel

Status: implementatiespecificatie, 26 oktober 2026. Dit document sluit de bootkeuzes voor M1; de volgende ontwikkelchat implementeert ze. Het bevat geen gecompileerde loader, kernel of geslaagde VM-test. De huidige onderzoeksomgeving heeft geen `rustc`, `cargo`, `rustup`, QEMU, Clang of `ld.lld` in PATH. Er is dus geen bootbewijs. De broncontrole hieronder is wel uitgevoerd. Lees voor M2–M4 ook [KERNEL-AND-SECURITY-CONTRACT.md](KERNEL-AND-SECURITY-CONTRACT.md).

## Uitvoeringscorrecties — 27 oktober 2026

De daadwerkelijke Windows-run gebruikt QEMU11.1.0 uit het vastgelegde ontwikkelpakket met bijpassende firmware. De oorspronkelijke 2–18MiB-reservering overlapt daarin ACPI NVS en BootServicesData; de loader wees deze terecht af. Het vaste profiel is daarom gewijzigd naar32–48MiB en BootInfo ABI1.1. De headerlayout is ongewijzigd; ABI1.0 wordt afgewezen. De alleen-lezen ESP is aangesloten via virtio, omdat IDE de alleen-lezen blocknode weigert. Een compilergegenereerde statische GOT is expliciet in .data opgenomen; runtime-relocaties blijven verboden. Zie ADR-019 t/m ADR-021 en PROJECT-STATE.md voor echt testbewijs. Oorspronkelijke onderzoeksclaims hieronder zijn historische context, geen nieuwe testuitslag.

## 1. Vast profiel

| Onderdeel | Besluit voor M1 | Bewijsstatus |
|---|---|---|
| Architectuur | x86_64, little endian, 4 KiB-pagina's, één CPU | Ontwerpkeuze |
| VM | `pc-q35-10.0`, `qemu64`, TCG, 256 MiB RAM | Ontwerpkeuze; nog geen boot |
| Rust | exact **1.90.0**, edition 2021, `panic=abort` | Officiële release van 18 september 2025 bevestigd [B1]; bewust een vaste bekende baseline, niet een claim dat dit de nieuwste compiler is |
| Loader-target | `x86_64-unknown-uefi`; eigen PE32+ UEFI-app | Target en calling convention bevestigd [B2] |
| Kernel-target | `x86_64-unknown-none`; eigen statische ELF64 `ET_EXEC` | Target bevestigd [B3] |
| Enige externe loadercrate | `r-efi = { version = "=7.1.0", default-features = false }` | Gepubliceerde versie 10 augustus 2026 en MSRV 1.68 bevestigd [B4–B5]; combinatie met onze code nog niet gecompileerd |
| Kerneldependencies | alleen eigen `boot-protocol`; `core` van gepinde target | Geen externe kernelimplementatie |
| QEMU-referentie | **10.0.14** | Op officiële downloadpagina bevestigd [B6] |
| Firmware-referentie | EDK II **edk2-stable202508**, `OvmfPkg/OvmfPkgX64.dsc`, X64/RELEASE/GCC5 | Release en officiële bouwroute bevestigd [B7–B8]; eigen binaries en hashes nog niet geproduceerd |
| Opslag | gegenereerde ESP-directory via QEMU's read-only FAT-disk | Alleen synthetische projectbestanden |
| Netwerk | `-nic none`; geen netwerkdevice, hostshare, agent, clipboard of gast-netwerkstack | Testprofiel; geen bewijs van latere appisolatie |

**Bouw meteen de eigen code.** Een bestaande of distributie-QEMU mag voor ontwikkeling worden gebruikt als `pc-q35-10.0` beschikbaar is. Gebruik dan OVMF_CODE en OVMF_VARS uit precies dezelfde package/build; leg versie, herkomst en SHA-256 van beide vast. Deze variant moet dezelfde tests halen, maar is niet byte-identiek aan de referentie. Bouw QEMU en EDK II niet eerst vanaf bron als deze ontwikkelvariant beschikbaar is. Van de VARS-template wordt voor iedere test een eigen wegwerpkopie gemaakt; CODE blijft read-only. Secure Boot staat in M1 uit; M1 bewijst geen geauthenticeerde bootketen.

`r-efi` levert alleen specificatietypen en constanten, geen bootloader, allocator, driver of OS-kernel. Het project schrijft de laadlogica zelf. De geraadpleegde Cargo-manifestbron toont geen standaard ingeschakelde dependency; `rustc-std-workspace-core` is optioneel. Licentiekeuze voor deze dependency: MIT (de upstream biedt MIT OR Apache-2.0 OR LGPL-2.1-or-later). Bewaar notice en registrychecksum in de dependencyadministratie. Geen `uefi-services`, `bootloader`-crate of overgenomen kernel erbij voegen. De high-level `uefi`-crate is hier bewust niet geselecteerd: een kleine lokale wrapper rond ruwe bindings maakt de laatste allocaties en de overdracht expliciet controleerbaar.

## 2. Te maken bestanden en vaste commando's

| Pad | Verantwoordelijkheid |
|---|---|
| `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.cargo/config.toml` | Integratie door hoofdagent; workspace bevat uitsluitend hieronder genoemde Rust-packages |
| `crates/boot-protocol/src/lib.rs` | `#![no_std]`; BootInfo, offsets, veilige header-/mapvalidatie |
| `boot/loader/Cargo.toml`, `boot/loader/src/main.rs` | Package `os-loader`; UEFI-entry, beperkte firmware-wrapper, bestandslezing, overdracht |
| `boot/loader/src/handoff.S` | EFIAPI naar SysV, eigen stack, interrupts uit, lege IDT |
| `crates/elf-loader-core/src/lib.rs` | Eigen `no_std` byteparser; hosttestbare ELF-validatie en laadplan zonder fysieke writes |
| `kernel/Cargo.toml`, `kernel/src/main.rs` | Package `os-kernel`; entry, eigen COM1-uitvoer, BootInfo-controle en testexit |
| `kernel/linker.ld` | Expliciete ELF-segmenten en vaste adressen |
| `tools/build.py` | Beide targets bouwen, ELF controleren en ESP-directory maken |
| `tools/run.py` | QEMU-argumenten, private VARS-kopie, logs, timeout en rooktest |
| `tools/test.py` | Hosttests, corrupte ELF-images, EBS-retryproef en VM-rooktest |
| `build/esp/EFI/BOOT/BOOTX64.EFI`, `build/esp/kernel.elf` | Gegenereerde wegwerpartifacts, niet handmatig aanpassen |
| `build/logs/`, `build/manifest.json` | Exacte commando's, hashes, versies, stdout/stderr en ruwe exitcodes |

Geen algemene target instellen voor de hele workspace: hosttests moeten op de hosttarget draaien. Product- en quantumprototypes zijn geen root-workspacemembers. Voeg eventuele geneste Rust-workspaces expliciet aan `exclude` toe.

```toml
# rust-toolchain.toml
[toolchain]
channel = "1.90.0"
profile = "minimal"
components = ["rustfmt", "clippy"]
targets = ["x86_64-unknown-uefi", "x86_64-unknown-none"]
```

```toml
# .cargo/config.toml — uitsluitend de kernel-target krijgt ELF-linkopties.
[target.x86_64-unknown-none]
linker = "rust-lld"
rustflags = [
  "-C", "code-model=small",
  "-C", "relocation-model=static",
  "-C", "no-redzone=yes",
  "-C", "link-arg=-Tkernel/linker.ld",
  "-C", "link-arg=--no-pie",
  "-C", "link-arg=--build-id=none",
  "-C", "link-arg=-zmax-page-size=0x1000",
]
```

De linker zit in de Rust-toolchain. `tools/build.py` vindt hem via `rustc +1.90.0 --print sysroot` plus de hosttriple uit `rustc +1.90.0 -vV`, en voegt `<sysroot>/lib/rustlib/<host>/bin` aan PATH van het subprocess toe. Zo vereist de naam `rust-lld` geen afzonderlijk geïnstalleerde host-LLD. De ingebouwde target selecteert de ELF-linkerflavour. Voor UEFI blijven de ingebouwde PE/COFF-linkerinstellingen behouden; stuur de ELF-scriptopties nooit naar de loader. De kern-target heeft standaard code-model `kernel`; onze lage plaatsing vereist de bovenstaande bewuste override naar `small` [B3]. Beide crates gebruiken `#![no_std]`, `#![no_main]`; geen nightly, custom-target JSON of `-Zbuild-std` nodig voor M1.

Rootprofielen: `panic = "abort"` in dev/release; release `codegen-units = 1`, `lto = false`. Gebruik geen unwindpad, globale allocator, TLS, floats/SIMD, C-runtime of host-syscalls. Als de linker geheugenintrinsics nodig blijkt te hebben, lever minimale eigen `memcpy`/`memmove`/`memset`/`memcmp` in een afzonderlijk gereviewd `unsafe`-module; voorkom optimalisatie tot recursieve aanroepen en controleer het resultaat. Voeg niet stilzwijgend host-libc toe.

Na installatie van rustup via de officiële procedure zijn dit de operationele ingangen:

```sh
rustup toolchain install 1.90.0 --profile minimal --component rustfmt --component clippy
rustup target add --toolchain 1.90.0 x86_64-unknown-uefi x86_64-unknown-none
python3 tools/preflight.py
python3 tools/build.py
python3 tools/run.py --smoke
python3 tools/test.py
```

Deze laatste drie tools worden door de implementatieopdracht gemaakt; dit document beweert niet dat ze al bestaan of zijn uitgevoerd. Eenmalig maakt `build.py` bij een ontbrekende lockfile met expliciet gepinde dependencies de `Cargo.lock`; daarna gebruiken alle builds `--locked`. Downloaden van publieke ontwikkeldependencies hoort bij de ontwikkelomgeving, niet bij het geïnstalleerde OS. Vervolgens kan een gevulde/vendored dependencycache offline worden gebruikt.

Onderliggend bouwt de tool vanuit de repositoryroot:

```sh
cargo +1.90.0 build --locked --release -p os-loader --target x86_64-unknown-uefi
cargo +1.90.0 build --locked --release -p os-kernel --target x86_64-unknown-none
```

## 3. ELF en linkerlayout

De kernel gebruikt fysieke adressen gelijk aan virtuele adressen. Reserveer **`[0x02000000, 0x03000000)`**, exact 16 MiB, via UEFI `AllocatePages(AllocateAddress, EfiLoaderCode, 4096, &base)` voordat andere grote buffers worden gealloceerd. M1 is specifiek voor het vastgelegde VM-profiel. Wanneer deze reservering niet lukt: expliciete fout; geen willekeurig ander laadadres. De linker begrenst de gebruikte kernel tot dit interval.

Startpunt voor `kernel/linker.ld`:

```ld
OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)
PHDRS {
  text   PT_LOAD FLAGS(5);
  rodata PT_LOAD FLAGS(4);
  data   PT_LOAD FLAGS(6);
}
SECTIONS {
  . = 0x02000000;
  __kernel_start = .;
  .text : ALIGN(4096) { KEEP(*(.text.entry)) *(.text .text.*) } :text
  . = ALIGN(4096);
  .rodata : { *(.rodata .rodata.*) } :rodata
  . = ALIGN(4096);
  .data : { *(.data .data.*) *(.got .got.*) } :data
  .bss (NOLOAD) : ALIGN(16) { *(.bss .bss.*) *(COMMON) } :data
  __kernel_end = ALIGN(4096);
  /DISCARD/ : { *(.eh_frame*) *(.comment) *(.note*) }
  ASSERT(__kernel_end <= 0x03000000, "kernel exceeds reserved span")
}
```

`build.py` inspecteert de echte ELF-output; het script alleen is geen bewijs voor de segmenten. Verwerp onverwachte allocatable orphan-secties, dynamische metadata of relocaties die runtimeverwerking vereisen. Bewaar een machineleesbaar segmentoverzicht. `p_paddr == p_vaddr` is verplicht. RX, R en RW krijgen aparte, op 4 KiB uitgelijnde pagina-intervallen; geen paginaoverlap door afronding toestaan.

De loader behandelt het gehele ELF-bestand als onbetrouwbare bytes. Een pure parser produceert eerst een begrensd laadplan; pas na volledige validatie volgen fysieke writes. Accepteer uitsluitend ELF64, little-endian, huidige ELF-versie, `EM_X86_64=62`, `ET_EXEC=2`, headergrootte 64 en programheadergrootte 56. Bestand maximaal 32 MiB, `1 <= e_phnum <= 64`. Alle optellingen/vermenigvuldigingen en omzettingen naar `usize` checked. Volledige programheadertabel binnen bestand; geen extended programheader numbering.

Per `PT_LOAD`: `filesz <= memsz`; `offset + filesz` binnen bestand; niet-lege geheugenrange volledig binnen de gereserveerde 16 MiB; `p_align` macht van twee en minstens 4096 voor dit profiel; `offset % p_align == vaddr % p_align`; `paddr == vaddr`; alleen R/RX/RW-vlagcombinaties, nooit W+X. Niet-lege loadranges overlappen niet, ook niet na afronding naar pagina's. Entry ligt binnen de daadwerkelijk uit bestand geladen bytes van precies één RX-segment. Nul-lange segments tellen niet als entry of geldige laadinhoud.

Voor dit kleine profiel: weiger `PT_INTERP`, `PT_DYNAMIC`, `PT_TLS`, en onbekende programheadertypen. `PT_NULL`, `PT_NOTE`, `PT_PHDR` en een niet-uitvoerbare `PT_GNU_STACK` mogen als metadata worden geaccepteerd, maar worden niet uitgevoerd. Minstens één RX-segment en één niet-leeg `PT_LOAD` vereist. Sectionheaders zijn geen laadbron; optionele sectioninformatie is alleen build-time controle. Eerst het gereserveerde gebied nulmaken, daarna uitsluitend goedgekeurde `filesz` kopiëren; daarmee is `.bss` nul. Deze projectrestricties zijn bewust strenger dan algemene ELF-mogelijkheden [B11].

**W^X-status:** ELF-vlaggen vormen in M1 een gecontroleerde intentie, nog geen afgedwongen geheugenbescherming. M1 erft tijdelijk UEFI-pagetables. M2 maakt eigen tables, supervisor-only mappings, read-only code en NX-data en test de echte faults. Dit M1-profiel heeft nog geen userspace, ASLR, guardpages, DMA-isolatie of Secure Boot. Claim daarvoor geen beveiliging op basis van een succesvol bootbericht.

## 4. BootInfo v1.1: exact overdrachtsformaat

Alle meerbytevelden zijn little-endian. Fysieke adressen worden als `u64` opgeslagen; geen Rust-references, `usize`, `bool`, enums, packed structs of slice-fat-pointers in de ABI. `#[repr(C)]`, alignment 8, totale grootte **128 bytes**. De structuur staat aan het begin van een eigen nulgevulde 4096-byte `EfiLoaderData`-allocatie. Alle pointers worden pas gebruikt na bounds-/overlapcontrole.

```rust
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BootInfo {
    pub magic: [u8; 8],
    pub abi_major: u16,
    pub abi_minor: u16,
    pub header_size: u32,
    pub flags: u64,
    pub memory_map_addr: u64,
    pub memory_map_len: u64,
    pub descriptor_size: u32,
    pub descriptor_version: u32,
    pub kernel_reserved_start: u64,
    pub kernel_reserved_len: u64,
    pub stack_start: u64,
    pub stack_len: u64,
    pub bootinfo_start: u64,
    pub bootinfo_len: u64,
    pub loader_start: u64,
    pub loader_len: u64,
    pub memory_map_capacity: u64,
    pub reserved0: u64,
}
```

| Byteoffset | Veld | Waarde/semantiek |
|---:|---|---|
| 0 | `magic`, 8 bytes | `b"NOSBOOT\0"` |
| 8 | `abi_major`, u16 | 1 |
| 10 | `abi_minor`, u16 | 1 |
| 12 | `header_size`, u32 | 128 |
| 16 | `flags`, u64 | bit0: EBS geslaagd; bit1: firmware-paging nog actief; bit2: M1-QEMU-testprofiel; overige bits 0 |
| 24 | `memory_map_addr`, u64 | Fysiek begin van aparte, behouden pageallocatie |
| 32 | `memory_map_len`, u64 | Werkelijke bytes van de laatste succesvolle `GetMemoryMap` |
| 40 | `descriptor_size`, u32 | Firmwarestride, minstens 40, veelvoud van 8, maximaal 4096 in dit profiel |
| 44 | `descriptor_version`, u32 | 1; onbekende versie wordt voor M1 geweigerd |
| 48 | `kernel_reserved_start`, u64 | 0x02000000 |
| 56 | `kernel_reserved_len`, u64 | 0x01000000 |
| 64 | `stack_start`, u64 | Eigen pagealigned stackbasis, geen firmwarestack |
| 72 | `stack_len`, u64 | 65536 bytes |
| 80 | `bootinfo_start`, u64 | Fysiek adres van deze structuur en pageallocatie |
| 88 | `bootinfo_len`, u64 | 4096 bytes |
| 96 | `loader_start`, u64 | `LoadedImage.ImageBase`, omlaag afgerond naar 4 KiB |
| 104 | `loader_len`, u64 | Hele geladen PE-image tot omhoog afgerond `ImageBase + ImageSize` |
| 112 | `memory_map_capacity`, u64 | Volledige gereserveerde mapbuffer in bytes; kan groter zijn dan len |
| 120 | `reserved0`, u64 | 0 |

M1 accepteert exact major1/minor1/header128 en flags7. Wijzig semantiek of vereiste velden alleen met een afgesproken ABI-versie; laat loader en kernel dezelfde crate gebruiken. Compile-time assert voor `size_of`, `align_of` en **ieder** `offset_of!`; een hosttest controleert een expliciete bytefixture. Daarmee kan de implementatie geen verborgen paddingverschuiving introduceren.

Uitgevoerde beperkte controle op 26 oktober 2026: een Python `ctypes.Structure` met exact bovenstaande vaste-width-velden geeft op de onderzoeks-host grootte128, alignment8 en offsets `[0,8,10,12,16,24,32,40,44,48,56,64,72,80,88,96,104,112,120]`; asserts geslaagd, procesexit0. Dit controleert de voorgeschreven C-layout, niet de nog ongebouwde Rust-binaries of de boot.

Kernelreservering, stack, BootInfo, geladen loader en mapbuffer zijn aparte niet-overlappende behouden intervallen. Alle beginnen zijn niet nul en pagealigned. Alle einden met checked arithmetic, beneden 1 GiB in dit profiel; stack/map/BootInfo worden met `AllocateMaxAddress` onder die grens gealloceerd. De 256 MiB guest bepaalt feitelijk een lagere beschikbare RAMgrens. Bevestig dat ranges in de firmwarekaart door LoaderCode/LoaderData worden gedekt; houd rekening met meerdere aansluitende descriptors. BootInfo wijst niet naar data op een verlaten stack. `bootinfo_start` moet gelijk zijn aan het ontvangen pointeradres.

### Geheugenkaartcontract

Het formaat is de ruwe, behouden UEFI-map van de succesvolle EBS-overgang. Itereer met `descriptor_size`, **nooit** met `size_of::<MemoryDescriptor>()`. Vereis `0 < len <= capacity <= 1 MiB`, `len % stride == 0`. De gebruikte lengte heeft geen padding aan het einde; bytes daarbuiten zijn geen descriptors. Elk record heeft de UEFI-v1-prefix: `type:u32@0`, gereserveerde padding@4, `physical_start:u64@8`, `virtual_start:u64@16`, `number_of_pages:u64@24`, `attribute:u64@32`. Lees prefixvelden als bytes/unaligned integerwaarden; vorm geen typed slice met de verkeerde stride.

`physical_start` is pagealigned; lengte is `number_of_pages * 4096` met overflowcontrole. Nul-paginarecords worden geweigerd. Descriptorvolgorde is niet vertrouwd; detecteer overlap zonder een firmware-allocator te gebruiken. Onbekende typen/attributen worden gereserveerd behandeld. `virtual_start` is geen vrijbrief om die pointer te volgen. MMIO-ranges mogen buiten de 256 MiB RAM liggen; map of gebruik ze niet automatisch.

**M1 hergebruikt geen enkel firmwaregeheugen.** De loaderimage, stack, map, BootInfo, runtime-/ACPI-reservaties en bestaande pagetables blijven dus bestaan. M2 begint alleen met geschikt `EfiConventionalMemory` (type7), met expliciete aftrek van behouden ranges en uitsluiting van runtime/nonvolatile/speciale eigenschappen. EfiBootServicesCode/Data worden niet automatisch vrijgegeven terwijl firmware-CR3 nog actief is. Eerst eigen tables/GDT/IDT/stack en een gecontroleerd reclamationbesluit. Er staan geen UEFI-servicepointers in BootInfo. De raw map volgt de UEFI-definitie; onze allocatiegrenzen en reclamationregels zijn projectbeleid [B9].

## 5. UEFI-loader en ExitBootServices

Gebruik `use r_efi::efi;` en de upstream protocoltypen; definieer niet zelf een gedeeltelijke `BootServices`-struct met mogelijk verkeerde offsets. Entry heeft `extern "efiapi"`, `efi::Handle` en `*mut efi::SystemTable`, en retourneert vóór de overdracht `efi::Status`. Iedere firmwarecall zit achter één kleine wrapper met geldigheids-/levensduurcontract. `extern "C"` wordt niet als universeel UEFI-ABI gebruikt [B2].

De volgorde ligt vast:

1. Controleer image/system/boot-services-pointers en tabelrevisie; blijf op `TPL_APPLICATION`. Schakel watchdog met `SetWatchdogTimer(0, 0, 0, null)` uit. Maak geen eigen UEFI-events aan. Bewaar imagehandle; `ExitBootServices` moet dat oorspronkelijke handle krijgen.
2. Lees via Loaded Image Protocol de imagebasis/omvang en het bootvolume. Open de Simple File System op **datzelfde** device en lees `\kernel.elf` met File Protocol. Geen netwerkboot of doorzoeken van alle volumes.
3. Reserveer de vaste kernelspan vóór grote filebuffers. Lees het bestand in een eigen, begrensde maximaal-32-MiB loaderdatabuffer; accepteer short reads, EOF en expliciete foutstatus, nooit een ongecontroleerde file size. Valideer volledig en laad volgens §3. Sluit file/root handles en eventueel geopende protocolrelaties. Geef tijdelijke file-/infobuffers terug zolang bootservices nog volledig beschikbaar zijn.
4. Alloceer eigen stack (16 pages) en BootInfo (1 page) als LoaderData. De definitieve mapbuffer volgt in stap5. Gebruik geen `Vec`, logger met allocaties of impliciete `Drop` die later firmware kan aanroepen. Bewaar loaded-imagegegevens als gewone integers; geen afhankelijke protocolobjecten na deze fase.
5. Vraag de benodigde mapgrootte op met `GetMemoryMap` en nulcapaciteit. Alloceer `round_up(max(256 KiB, needed + 8 * descriptor_size), 4096)`, met checked arithmetic en hard maximum 1 MiB. Dit is capaciteit, niet de uiteindelijke map. Als de eerste echte map nog niet past, mag **vóór de allereerste EBS-poging** gecontroleerd worden vergroot. Maximaal drie voorbereidingspogingen; anders fout.
6. Initialiseer BootInfo met alle reserveringen, bit0 nog nul. Rond alle firmwarediagnostiek en resource-drops af. Vanaf nu is geen allocation/free, protocolcall, consolecall, callbackregistratie of firmwarelog toegestaan. De laatste memory-mapbufferallocatie zelf moet dus al in de daarna opgevraagde kaart staan.
7. Zet bij **iedere** `GetMemoryMap` de in/out `MemoryMapSize` opnieuw op buffer-capacity; haal map, stride, versie en mapkey op. Controleer in eigen code metadata zonder allocaties. Geef **deze mapkey** direct aan `ExitBootServices(image_handle, map_key)`. CPU-writes naar reeds gereserveerde BootInfo veranderen de firmwarekaart niet.
8. `SUCCESS`: onmiddellijk eigen `cli`/`cld` en lege-IDT-bootstrap; markeer de bootservices-wrapper als verbruikt. Geen destructor/terugkeerpad met firmwarecalls. Bewaar precies deze laatste maplen/stride/versie. Zet succesbit, log `BOOT:EBS_OK` met eigen COM1 en ga naar de handoff. Ook het codepad tussen successtatus en shim mag geen firmware-interrupthandler meer gebruiken.
9. `INVALID_PARAMETER`: mapkey is niet meer bruikbaar. Herhaal alleen `GetMemoryMap` in **dezelfde bestaande buffer** en daarna EBS. Maximaal drie EBS-pogingen totaal. Dit ontwerp gebruikt ook tijdens retry geen nieuwe allocatie. Als buffer dan te klein is, metadata fout is of een andere status volgt: definitieve testfout; geen terugkeer naar firmwaremenu of oude key hergebruiken.

Na een eerste mislukte EBS-call kan firmware gedeeltelijk zijn afgebouwd. Daarom is het retrygebied bovenstaande kleine lus; geen bestands-I/O of logging via firmware. Na succes worden ook Runtime Services niet gebruikt. Failures loggen via de eigen vaste QEMU-COM1-driver en testexit; dit directe preboot-I/O is uitsluitend de terminale foutdiagnostiek in ons expliciete QEMU-profiel. Algemene portable UEFI-I/O loopt vóór de finalisatie via firmware. De EBS-statusmachine volgt de actuele-mapregel uit UEFI §7.4.6; de retrylimiet en vooraf gereserveerde buffer zijn eigen beleid [B9].

Voor een reproduceerbare retrytest: een **alleen test-build** haalt vóór eerste EBS een mapkey op, alloceert vervolgens één extra testpagina en bevriest daarna alle allocaties. De eerste poging met de oude key moet `INVALID_PARAMETER` krijgen; de retry haalt de verse kaart op. Die testpagina blijft gereserveerd in de laatste kaart. De productiecode krijgt geen fault-injectionflag of verouderde-keypad.

## 6. CPU-handoff: twee verschillende calling conventions

De loader is een UEFI-app met de x64 EFIAPI-conventie. De ELF-kernel gebruikt expliciet `extern "sysv64" fn _start(info: *const BootInfo) -> !`. Verander niet met een Rust-`transmute` alleen het functietype en veronderstel dat de registers dan correct zijn.

De losse assemblyshim is vanuit Rust gedeclareerd als `extern "efiapi" fn handoff(entry: u64, info: u64, stack_top: u64) -> !`. Hij ontvangt dus RCX=entry, RDX=info, R8=stack_top. Hij bevestigt de direct na EBS ingestelde toestand met `cli` en `cld`, laadt een IDT-descriptor met limit0/base0, neemt de eigen stack (`RSP=R8`, naar 16 bytes uitlijnen), wist RBP, zet RDI=RDX en doet `call RCX`. Daardoor is RSP bij de SysV-entry congruent aan 8 modulo16. Er is geen Windows-shadowspace nodig voor die laatste SysV-call. De shim keert niet terug; onverwachte kernelreturn geeft failurecode0x11 en daarna `cli; hlt` in een lus.

De lege IDT is uitsluitend een M1-fail-closed bootstrap: een exception/NMI leidt tot shutdown/triplefault en dus een mislukte rooktest, niet tot een oude firmwarehandler. M2 vervangt dit door eigen exceptionhandlers, GDT/TSS en IDT voordat interrupts of userspace worden toegestaan. M1 herlaadt geen firmwaresegmenten; bestaande segmentcaches en CR3 blijven tijdelijk behouden als bootstraptoestand. Er wordt geen firmwarefunctie meer aangeroepen en geen onderliggend geheugen gerecycled. Dit is een bewuste korte bootfase, geen eindarchitectuur.

Maak deze stackwissel niet in een gewone Rustfunctie waarvan de compiler nog een epiloog uitvoert. Gebruik een apart assemblysymbool via `global_asm!`/`include_str!`, met COFF-compatibele directives in de loader. Kernelentry krijgt `#[no_mangle]` en `.text.entry`. Geen SIMD/FPU-activering nodig voor M1: beide targets genereren standaard geen gebruik van die registers; introduceer geen target-feature of floating-point-dependency [B2–B3].

## 7. Seriële output en machine-testcontract

COM1 base0x3f8, polling, 115200 baud, 8N1. Eigen minimale port-I/O-module: interrupts uit op UART, DLAB aan, divisor1/0, LCR0x03, FIFO0xc7, MCR0x0b; poll LSR-bit5 vóór ieder byte. Per byte maximaal 1.000.000 polls; daarna failure-exit, geen eindeloze wachtlus. Gebruik `out dx, al`/`in al, dx` voor UART en `out dx, eax` naar 0xf4 voor debugexit. Een uitgeschakelde/ontbrekende UART mag een negatieve test niet oneindig laten hangen.

Na succesvolle EBS logt eigen code, in deze volgorde:

```text
BOOT:EBS_OK
KERNEL:ENTRY
BOOTINFO:OK
M1:PASS
```

De eerste regel ontstaat in de loader na EBS; de volgende drie in de echte ELF-kernel. Alleen de kernel mag `M1:PASS` en de succesexit uitsturen. Een panic of validatiefout schrijft waar mogelijk `M1:FAIL:<vaste_code>` en exit0x11; formatteer geen secrets of onbeperkte ontvangen tekst. De kernel controleert ook `.bss`-nulinitialisatie en een bekende `.data`-waarde vóór succes.

`isa-debug-exit` gebruikt de projectwaarden **0x10 = pass**, **0x11 = fail**. De hoststatus is `(value << 1) | 1`: dus **33** respectievelijk **35**; een gewone hostexit0 is geen testpass [B10]. Alleen `run.py` vertaalt (33 én alle verwachte markers in volgorde én geen failmarker) naar harness-exit0. Een fout met status35, signal, firmwareprompt, triplefault, QEMU-launchfout of ontbrekende marker blijft fail. Ruwe QEMU-status blijft altijd in het log.

`run.py --smoke` voert semantisch dit commando uit (concrete absolute paden via subprocess-argumentlijst, geen shellinterpolatie):

```sh
qemu-system-x86_64 \
  -machine pc-q35-10.0 \
  -accel tcg,thread=single \
  -cpu qemu64 -smp 1 -m 256M \
  -nodefaults -display none -monitor none -serial stdio \
  -nic none -no-reboot \
  -drive if=pflash,unit=0,format=raw,readonly=on,file=build/firmware/OVMF_CODE.fd \
  -drive if=pflash,unit=1,format=raw,file=build/run/OVMF_VARS.fd \
  -drive if=virtio,index=0,format=raw,readonly=on,file=fat:build/esp \
  -device isa-debug-exit,iobase=0xf4,iosize=0x04
```

De tool zet de werkdirectory naar de repositoryroot; `fat:build/esp` is QEMU's read-only virtuele FAT-schijf, niet een echte hostdisk. De ESP-directory is een stabiele kopie voor de duur van de VM-run; tijdens uitvoering geen builds die haar muteren. FAT bevat de standaardbootnaam `EFI/BOOT/BOOTX64.EFI` en `kernel.elf`; nog geen diskpartitioneringstool nodig. Later krijgt een echt installatiemedium een eigen geteste GPT/ESP-builder. Geen `-no-shutdown`: dit kan de verwachte testexit verstoren bij QEMU-implementaties die shutdownrequests gebruiken.

Harde timeout **60 seconden** vanaf QEMU-start. `subprocess.run(..., timeout=60)` of equivalent beëindigt een vastgelopen proces en bewaart beschikbare stdout/stderr; een time-out is expliciet mislukt, nooit overgeslagen. Bewaar starttijd, duur, versie, argumenten, returncode, alle artifacthashes en het gebruikte ontwikkel-/referentieprofiel. De tijdslimiet is een CI-budget, geen prestatiebelofte voor het OS.

## 8. Acceptatiepoort: wanneer M1 klaar is

| Test | Vereist bewijs |
|---|---|
| Toolchain/target | Beide binaries bouwen met Rust1.90.0 en `--locked`; loader is PE32+, kernel ELF64 ET_EXEC; geen host-OS imports |
| ABI | Compilerasserts voor alle offsets/grootte/alignment; hostbytefixture; foutieve magic/major/flags/range/stride afgewezen |
| ELF-parser | Truncaties, overflow, verkeerde machine/type, filesz>memsz, ongeldige entry, overlapping pagina's, RWX en DYNAMIC/INTERP/TLS afgewezen vóór writes |
| Rooktest | Eigen ELF-entry na EBS, BootInfo/BSS/data goed, markers in volgorde, QEMU33 en harness0 binnen60s |
| Afwijzen image | Gemuteerde `kernel.elf` weigert vóór kernelentry, failmarker en QEMU35; geen `M1:PASS` |
| Retry | Test met expres oude mapkey faalt eerste EBS, slaagt met nieuwe kaart zonder nieuwe allocatie; normale build bevat injectie niet |
| Foutpad | Expliciete kernelpanic bereikt failurecode; een hang en afwijkende exit worden door harness als fail behandeld |
| Onafhankelijkheid | Na eerste EBS geen protocol-/console-/filecalls; na succes geen enkele UEFI-call, geen `Drop` met firmware-effecten |
| Herhaalbaarheid | Drie opeenvolgende cold boots, iedere keer verse VARS-kopie; hashes en ruwe logs bewaard |

Pure parser-/protocoltests draaien op de host en bewijzen uitsluitend die logica. QEMU-tests moeten daadwerkelijk de gebouwde loader en kernel uitvoeren; een Pythonmodel of nagebootst seriëel log telt niet. Er is na dit document **geen open architectuurkeuze nodig om M1 te schrijven**. Resterend werk is concrete hostpreflight, implementatie, compilerdiagnostiek, het werkelijk verkrijgen/hashen van firmware en het meten van de bovenstaande tests. Een mislukte test wordt een klein geregistreerd implementatieprobleem, geen aanleiding om zonder bewijs opnieuw de hele OS-richting te onderzoeken.

## 9. Bronnen en controlegrenzen (APA 7)

Geraadpleegd op 26 oktober 2026. Ontwerpgetallen, laadadres, schema, limieten en testmarkers zijn eigen projectbesluiten. Bronnen bevestigen de gebruikte mechanismen; ze bewijzen niet dat onze nog ongeschreven code correct is.

- **B1.** The Rust Project. (2025, 18 september). *Announcing Rust 1.90.0*. https://blog.rust-lang.org/2025/09/18/Rust-1.90.0/
- **B2.** The Rust Project. (z.d.). *\*-unknown-uefi*. The rustc book. https://doc.rust-lang.org/rustc/platform-support/unknown-uefi.html
- **B3.** The Rust Project. (z.d.). *x86_64-unknown-none*. The rustc book. https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-none.html
- **B4.** r-efi Contributors. (2026, 10 augustus). *r-efi 7.1.0*. Docs.rs. https://docs.rs/crate/r-efi/latest (geraadpleegde versiepagina vermeldt expliciet 7.1.0 en releasehistorie; exact-version-URL kon via de onderzoekstool niet worden opgehaald).
- **B5.** r-efi Contributors. (2026). *Cargo.toml* [Broncode; versieveld 7.1.0, rust-version 1.68]. https://github.com/r-efi/r-efi/blob/main/Cargo.toml — de release staat ook op https://github.com/r-efi/r-efi . `main` is veranderlijk; de implementatie downloadt exact7.1.0, controleert het verpakte manifest en bewaart de Cargo-registrychecksum. Die download/controle is hier niet uitgevoerd.
- **B6.** QEMU Project. (2026). *Download QEMU*. https://www.qemu.org/download/ — vermeldt versie10.0.14; distributieoverzicht https://download.qemu.org/ . Invocation: https://www.qemu.org/docs/master/system/invocation.html . De CLI is onderzocht; exacte lokale machine-/firmwarecombinatie nog niet uitgevoerd.
- **B7.** TianoCore. (2025, 22 augustus). *edk2-stable202508* [Release]. https://github.com/tianocore/edk2/releases/tag/edk2-stable202508
- **B8.** TianoCore. (z.d.). *How to build OVMF*; *How to run OVMF*. https://www.tianocore.org/tianocore-wiki.github.io/development/tutorials-howto/how_to_build_ovmf.html ; https://www.tianocore.org/tianocore-wiki.github.io/development/tutorials-howto/how_to_run_ovmf.html
- **B9.** UEFI Forum. (2022). *UEFI specification 2.10: 7. Services—Boot Services*, §§7.2.1, 7.2.3, 7.4.4 en 7.4.6. https://uefi.org/specs/UEFI/2.10/07_Services_Boot_Services.html — hoofdstuk inhoudelijk geraadpleegd, inclusief EBS-retry en descriptorstride. De 2.11-URL gaf een toegangsblokkade; daarom wordt hier expliciet de bereikbare 2.10-tekst gebruikt, niet verzonnen 2.11-inhoud.
- **B10.** QEMU Contributors. (z.d.). *hw/misc/debugexit.c* [Broncode]. https://github.com/qemu/qemu/blob/master/hw/misc/debugexit.c — geraadpleegde source bevat `(val << 1) | 1`. Controle van de gekozen lokale binary gebeurt aanvullend door de rooktest; dit is geen claim dat master hetzelfde bronbestand is als tag10.0.14.
- **B11.** Xinuos. (z.d.). *ELF object file format: 7. Program loading* (System V gABI4.2). https://gabi.xinuos.com/v42/elf/07-pheader.html — primaire ELF64-PHDR-layout, segmenttypen, filesz/memsz, alignment en permissies inhoudelijk geraadpleegd. Het gebruik van een publiek bestandsformaat maakt onze kernel geen afgeleide van een bestaand OS.
