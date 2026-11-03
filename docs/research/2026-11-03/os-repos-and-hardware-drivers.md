# Vergelijking van OS-repositories en driverstrategie

**Datum:** 3 november 2026
**Onderzochte Nieuw OS-basis:** `99d132768fd72276caa4a3b3e9a09f8f1909f0ed`
**Status:** documentatieonderzoek en voorstel; geen architectuurbesluit, codeport of hardwaretest.

Aanvulling na twee extra door de gebruiker genoemde repositories: Nieuw OS-bron
`9e13e7be17845e03256dd451de57419eb242e376` (3 november 2026).
Aanvulling na de organisatieverwijzing `rust-osdev`: Nieuw OS-bron
`47cc895b4de6bfde02f294399ebc777c67575d24` (3 november 2026).

## Conclusie

De onderzochte repositories bieden geen kant-en-klare, brede verzameling
Linux-kerneldrivers die zonder port in Nieuw OS werkt. ODI heeft wel enkele
zelfstandige C-driverimplementaties; `device-driver` is een actieve Rust-toolkit
om apparaatspecifieke register-/commandocode te genereren. Beide zijn mogelijke
bronnen voor gericht vervolgwerk, niet een oplossing voor de hele hardwarematrix.
De voorgestelde passende route voor Nieuw OS is
de eigen kernel behouden, de vaste QEMU-doelen via VirtIO afmaken en GOP
(UEFI's framebufferinterface) gebruiken voor basale beeldweergave. Kies daarna
een concreet fysiek referentieprofiel en port alleen doelgericht geschikte
drivers of protocolcode. Een driver in een apart proces past bij de
capabilityarchitectuur, maar is pas werkelijk geïsoleerd als geheugen, MMIO,
interrupts en DMA begrensd zijn; voor DMA is een correct ingerichte IOMMU
relevant.

Dit beoogt de testmatrix en het aantal apparaatspecifieke implementaties
beheersbaar te houden; er is geen versnelling gemeten. Het belooft geen brede ondersteuning van willekeurige pc's,
wifi-adapters of GPU's.

## Wat de vijf repositories daadwerkelijk laten zien

| Repository en onderzochte bron | Opbouw en driveraanpak | Wat Nieuw OS kan meenemen | Bewijsgrens |
|---|---|---|---|
| [Ginux](https://github.com/gmkurtzer/ginux/tree/c86945b1ae246a7c9e4bd831a66e5179d55de95e), commit `c86945b` (3 nov. 2026) | Eigen Rust x86-64-kernel, UEFI-image, eigen proces-/domeinmodel en Linux-userspace-ABI. De README noemt VirtIO-blok/netwerk op QEMU Q35. ADR 0002 stelt Linux `.ko`-compatibiliteit uit; de README zegt dat er geen stabiele third-party driver-ABI is en dat algemene hardwaredekking toekomstwerk is. | Heldere onderscheidingen tussen eigen kernel, Linux-programmacompatibiliteit en drivers; echte distributie-workloads als integratietest; beperkingen per ondersteund VM-profiel opschrijven. | De README noemt onder meer een Rocky Linux 10 systemd-pad, maar dat is Ginux' eigen gerapporteerde testbewijs. Geen onafhankelijke reproductie in dit onderzoek. De “11 dagen” beschrijven een agentische ontwikkelmethode, niet een universele driveroplossing. |
| [VibeOS](https://github.com/kaansenol5/VibeOS/tree/866bfb9b7fc554b5b8e0939b4626bcf0416b8b19), commit `866bfb9` (27 jan. 2026) | Zelfgebouwd AArch64-hobby-OS voor QEMU en volgens de README Raspberry Pi Zero 2W, met eigen USB DWC2/HID- en eMMC-paden. De README noemt op de Pi geen netwerk- of audiodriver en beschrijft USB-beperkingen, waaronder geen hotplug. | Kies een concreet board en maak een verticale doorsnede met echte apparaattests; documenteer ontbrekende functies en storingsgedrag even expliciet als de demo. | README-claims zijn niet opnieuw op QEMU of een Pi uitgevoerd. De bijbehorende `CLAUDE.md` vermeldt dat zijn status uit sessie 40 komt; die verouderde technische notities zijn niet gebruikt als actuele verificatie. |
| [vib-OS](https://github.com/viralcode/vib-OS/tree/73b64299bf6e591e738c58c9d1845f0babdd6f58), commit `73b6429` (29 jan. 2026) | Eigen C/assembly-kernel met eigen virtio-, video-, netwerk-, NVMe-, USB- en andere driverdirectories; richt zich vooral op ARM64 en claimt meerdere architecturen. De README bevat echter tegenstrijdige x86-64-status: de architectuursectie noemt een x86-64-build, terwijl dezelfde bron zegt dat die “will be released soon”; de meest recente commit verwijdert de x86-64-buildinstructies. | Houd een concrete architectuur-/device-matrix bij en scheid “code bestaat”, “buildt”, “boot in VM” en “getest op hardware”. | De README, screenshots en driverbestandsnamen zijn geen bewijs dat alle genoemde apparaten of platformen werken. Geen builds of hardwaretests uitgevoerd. |
| [Open Driver Interface (ODI)](https://github.com/Open-Driver-Interface/odi/tree/ef58952c4a7b27338cb47bdb9401d4b6f970143c), commit `ef58952` (1 dec. 2023) | C-framework met PCI-/ACPI-discovery en enkele driverimplementaties, waaronder AHCI, XHCI, E1000 en serieel. De quickstart vereist portering van OS-afhankelijke functies en beschrijft callbacks met generieke pointers. README noemt MIT. | Een specifieke driver, bijvoorbeeld E1000 of AHCI, kan later als lees-/portkandidaat dienen wanneer het gekozen fysieke profiel dit apparaat bevat. Vergelijk protocolkeuzes en hardwarevolgorde met onze eigen begrensde drivercontracten. | Dit is geen stabiele universele driver-ABI en geen Linux-`.ko`-wrapper. Repositoryhoofd is al sinds de vermelde commit uit 2023 niet bijgewerkt. De documentatie noemt zowel `source/` als testomgeving, terwijl de bekeken tree de implementatie in `src/` plaatst. Geen build, code-audit of compatibiliteitstest uitgevoerd; MIT-vermelding is geen kwaliteits- of veiligheidsbewijs. |
| [diondokter/device-driver](https://github.com/diondokter/device-driver/tree/036fb189f382d1ced62fa2695d43b3a69dcdaa73), commit `036fb18` (24 okt. 2026) | Actieve Rust-toolkit met `no_std`-runtime en DDSL: een apparaatspecificatie kan getypeerde register-, veld-, buffer- en commandotoegang/API genereren. De hoofdcrate is dual MIT/Apache-2.0; macro en `embedded-io`-integratie zijn optionele features. | Mogelijke latere registergenerator, niet direct compatibel met de projectpin: het upstream-workspacemanifest vereist Rust 1.95 en de runtime erft die eis. | Genereert geen volledige PCI-/USB-driver, busenumeratie, interruptservice, DMA-beheer, resetbeleid of capabilitygrens. VirtIO-queues en netwerkprotocollen worden er niet automatisch mee geïmplementeerd. De vereiste Rust 1.95 wijkt af van Nieuw OS' 1.90.0; dependency-policy, buildketen en concrete `no_std`-target zijn daarnaast niet getest. Geen dependency toegevoegd. |

Primaire toolchainbronnen, gecontroleerde bronversies op 3 november 2026:
diondokter/device-driver, *workspace Cargo.toml*, versie `036fb189`,
[upstream rust-version](https://github.com/diondokter/device-driver/blob/036fb189f382d1ced62fa2695d43b3a69dcdaa73/Cargo.toml);
Nieuw OS, *rust-toolchain.toml*, versie `1c687206`,
[projectpin Rust 1.90.0](https://github.com/simonvanieperen/nieuw-os/blob/1c6872063fab055211eff48fce11496db39728be/rust-toolchain.toml).
Dit is een manifestvergelijking, geen uitgevoerde port of compatibiliteitsproef.

De belangrijkste correctie op de aangehaalde Google-samenvatting: de huidige
Ginux-bron beschrijft Linux-*userspace*-compatibiliteit (Linux-programma's
draaien boven de eigen kernel), niet een werkende wrapper voor duizenden Linux
kerneldrivers. De eigen Ginux-ADR zegt expliciet dat binaire Linux-modules nog
zijn uitgesteld. Linux documenteert bovendien dat de kernel geen stabiele
interne module-/driver-API belooft; die is iets anders dan de veel stabielere
syscall-ABI voor gebruikersprogramma's. Een Linux-driver “foppen” vraagt dus
meer dan een dunne syscall-laag: bus-, geheugen-, DMA-, interrupt-, apparaat-
en subsystemcontracten moeten passen.

## Wat de rust-osdev-organisatie bijdraagt

De organisatie is een verzameling losse Rust-bibliotheken, geen geïntegreerde
driverstack. De nuttigste projecten voor de huidige x86-64/UEFI-route zijn:

| Project | Wat de bron daadwerkelijk levert | Relevantie en beperking voor Nieuw OS |
|---|---|---|
| [acpi](https://github.com/rust-osdev/acpi/tree/386035c1badb74e8bd5f3a63b385e77c60d79b1f) `386035c` | Rust-parser/API voor ACPI-tabellen en AML; een beperkte API werkt zonder allocator, rijkere functies/AML vereisen alloc. Licentie MIT/Apache-2.0; huidige manifest is versie `7.0.0-pre1`, Rust 2024 edition. | Kansrijke bouwsteen voor platform-/interrupt-/energie-inventarisatie, maar geen PCI- of apparaatspecifieke driver. Pre-release, dependencyboom en firmware-afwijkingen vragen een concrete test vóór adoptie. |
| [pci_types](https://github.com/rust-osdev/pci_types/tree/eff856b6bab81adbe1bda0ab750009f3786cb8d3) `eff856b` | `pci_types` 0.10.1: `no_std`-types en helpers voor PCI-adressen, configuratieregisters, capabilities en deviceklassen; geen PCI-configuratie-accessbackend of driverbinding. MIT/Apache-2.0. | Mogelijk nuttig bij toekomstige generieke PCI-enumeratie. Het vervangt geen veilige config-space access, BAR-toekenning, interrupt- of DMA-policy. |
| [xhci](https://github.com/rust-osdev/xhci/tree/f2254c86d5ba12754259bf0ae483c6c8dc821aad) `f2254c8` | `xhci` 0.9.2: `no_std`-register-, context-, ring- en extended-capabilitytypes voor xHCI. De README zegt expliciet dat een OS hiermee zelf een USB-device-driver kan implementeren. MIT/Apache-2.0; laatste commit in de onderzochte geschiedenis is uit 2024. | Bruikbaar als laag onder een toekomstige eigen xHCI-hostcontrollerdriver, niet als complete controller-, DMA-, interrupt-, enumeratie- of USB-HID-driver. Activiteit en API moeten vóór gebruik opnieuw worden beoordeeld. |
| [usb](https://github.com/rust-osdev/usb/tree/75e219179df7e0961aa42e18774beb8878765341) `75e2191` | Losse 0.1.0-hulppakketten, waaronder `usb_request` en `usb_hid_item`, voor USB-requests en HID-item-/usagegegevens; geen hostcontroller. De laatst gevonden commit dateert uit 2022. | Kan eventueel parser-/descriptorwerk versnellen nadat de USB-transportgrens bestaat. Oudere crates en parsergedrag eerst op malformed/truncated input testen; het is geen vervanging van onze USB-enumeratie of xHCI. |
| [uart_16550](https://github.com/rust-osdev/uart_16550/tree/770cbd28330de1cc461d62d8fc8ac5814c673304) `770cbd2` | Echte `no_std`, allocatievrije 16550-driver voor x86 port-I/O, MMIO en aangepaste backends, inclusief configuratie, RX/TX en interrupts; MIT/Apache-2.0. | Technisch concreter dan de meeste repos, maar manifest `0.9.0` vereist `rust-version = 1.91.0`; Nieuw OS staat vast op Rust 1.90.0. Dus de huidige upstream is niet direct in onze gepinde toolchain te gebruiken zonder expliciet toolchain-/versiebesluit en compatibiliteitstests. |
| [virtio-spec-rs](https://github.com/rust-osdev/virtio-spec-rs/tree/ad565bd701e93fa47bc288d45701e1ae53ac3c12) `ad565bd` | `virtio-spec` 0.4.0: Rust-definities uit de VirtIO-specificatie; README zegt expliciet dat de crate geen mening heeft over of de drivers erboven implementeert. MIT/Apache-2.0. | Mogelijk referentie-/typebron, maar geen VirtIO-driver en geen snellere route naar fysieke apparaten. Nieuw OS heeft al een eigen `virtio-core` en gerichte VirtIO-drivers; geen tweede protocolbron invoeren zonder concrete behoefte en contractvergelijking. |

De organisatie bevat daarnaast onder meer `bootloader`, `uefi-rs`,
`x86_64`, `pic8259` en `apic`. Dat zijn respectievelijk boot-, firmware-,
architectuur- en interruptbouwstenen, niet een algemene verzameling moderne
GPU-, wifi-, NVMe- of laptopdrivers. Een `xhci`-registercrate of
`virtio-spec`-definitie mag niet als een werkende driver worden geteld.

**Praktisch oordeel:** dit is de waardevolste bron die tot nu toe is bekeken
voor herbruikbare Rust-bouwstenen die bij onze taal en `no_std`-richting passen.
Het kan later parser-, type- en registerwerk beperken. De beste kandidaat hangt
af van de eerstvolgende concrete gate: PCI/ACPI voor platforminventarisatie,
xHCI-types voor een USB-hostcontrollerport, of een seriële driver zodra de
toolchaingrens is opgelost. Geen van deze crates levert op zichzelf een
geïntegreerde, capability-begrensde driverstack. Voeg niets toe voordat
herkomst, exacte versie, licentie, `no_std`/MSRV, build scripts, `unsafe`-
grenzen en negatieve native QEMU-tests zijn gecontroleerd.

## Beoordeling van de voorgestelde shortcuts

| Voorstel | Beoordeling voor Nieuw OS |
|---|---|
| Linux-drivers wrappen | **Niet als snelle route.** Een Linux-userspace-ABI helpt applicaties, niet kerneldrivers. Een compatibiliteitslaag voor `.ko`-modules zou een omvangrijk Linux-kernel-/subsystemmodel en een nieuwe vertrouwensgrens introduceren. Onderzoek dit hoogstens later als afgebakende haalbaarheidsproef; neem geen bestaande kernel als basis. Elke codeport vereist afzonderlijke licentie-, herkomst-, API- en beveiligingsreview. |
| rust-osdev-bibliotheken | **Gerichte versneller voor bouwstenen.** `acpi`, `pci_types`, `xhci` en USB-hulppakketten kunnen bepaalde parsing-/type-/registertaken schelen, maar zijn geen kant-en-klare drivers. `uart_16550` is een concretere driver, maar upstream vereist Rust 1.91 en past dus niet bij de huidige Rust 1.90-pin. `virtio-spec-rs` bevat specificatiedefinities, geen VirtIO-driver. |
| ODI gebruiken | **Mogelijk per driver, na port- en securityreview.** De C-code kan concrete controllerlogica bieden, maar verwacht OS-bindingen; de generieke pointer/callback-interface is geen Nieuw OS-capabilitymodel. Selecteer pas een driver na keuze van het apparaat en test die op het exacte QEMU-/hardwareprofiel. |
| `device-driver` gebruiken | **Mogelijk als latere codegenerator, niet als driverbibliotheek voor alle hardware.** De onderzochte versie vereist Rust 1.95 tegenover projectpin 1.90.0. Eerst die incompatibiliteit expliciet oplossen; niet stil de projecttoolchain verhogen. DDSL lost geen bus, interrupts, DMA, device lifecycle of protocolstack op. Daarna blijven een geïsoleerde `no_std`-proef en dependency-/feature-/buildscriptreview vereist vóór eventuele adoptie. |
| VirtIO en UEFI GOP | **Direct bruikbaar binnen hun grenzen.** VirtIO levert gestandaardiseerde apparaten aan VM's en past bij QEMU/CI; het is geen fysieke NIC-/NVMe-/GPU-driver voor gewone pc's. GOP laat de loader een firmwaremodus en framebuffer doorgeven, bruikbaar voor initiële pixels zonder GPU-versnelling. Het vervangt geen native GPU-driver, modesetting, audio, wifi of post-boot USB-stack. |
| AI als driver-portfabriek | **Alleen als hulpmiddel bij één port tegelijk.** Een agent kan specificaties, drivercode en tests voorbereiden, maar moet dezelfde bron-, licentie-, API-, DMA- en negatieve-testgates doorlopen als handgeschreven code. Automatisch herschrijven is geen bewijs van correctheid of compatibiliteit. |
| Drivers in gebruikersruimte | **Architectonisch passend, maar niet gratis of automatisch veilig.** Een procescrash kan lokaal worden herstart; eerder door het device uitgevoerde DMA kan alsnog geheugen beschadigen. Gebruik alleen beperkte capabilities, gepinde/afgeschermde buffers, reset/quiescencecontracten en waar nodig een correct ingerichte IOMMU. Een procesgrens alleen voorkomt geen DMA. |
| Host-/browsergebaseerde WASM-oplossing als OS-basis | **Niet passend.** Wanneer een bestaand host-OS of browser de hardwaredrivers levert, is dit geen zelfstandig Nieuw OS met eigen kernel. Dit sluit een later expliciet ontworpen WASM-appruntime boven de eigen kernel niet op voorhand uit. |

## Passende route voor Nieuw OS

1. **Rond eerst de VM-hardwaredoorsnede af.** Behoud x86-64/UEFI/QEMU als ontwikkeldoel en bouw voort op de bestaande VirtIO-blok-, RNG-, netwerk- en geluidsbouwstenen. Volg de bindende `VM-FR`-matrix; rapporteer per device apart of alleen een parser, hostmodel, crosscompile of echte QEMU-I/O is getest.
2. **Gebruik firmware alleen als startvoorziening.** GOP kan het eerste scherm leveren; de bestaande G0-specificatie begrenst al framebuffer, geometrie en post-`ExitBootServices`-gedrag. Voeg geen firmwareafhankelijkheid toe voor latere rendering of invoer zonder expliciet contract.
3. **Kies vóór fysieke driverportering één referentieapparaat.** Inventariseer UEFI/ACPI, PCI-ID's, opslag, bedraad netwerk, USB-controller en IOMMU. Begin met een beperkte set standaardinterfaces. Selecteer een wifi-chip of GPU pas wanneer het exacte onderdeel, firmwarepad, licentie en herstel-/testprofiel bekend zijn. Geen fysieke installatie of flash volgt uit dit voorstel.
4. **Isoleer apparaatdiensten via expliciete bevoegdheden.** Begin met een virtueel device dat in QEMU reproduceerbaar is; test ook malformed descriptors, ontbrekende devices, timeout, reset en herstart. Voeg gebruikersruimtedrivers pas toe wanneer capability- en DMA-eigendom in de kernel werkelijk worden afgedwongen.
5. **Port gericht, kopieer niet blind.** Hergebruik eerst standaardspecificaties en kleine, portable algoritmen. Voor drivercode: bepaal licentie en herkomst, vervang Linux-specifieke koppelingen door een smalle Nieuw OS-interface en test de resulterende implementatie tegen echte VM-I/O. Een drivernaam of geslaagde build is geen hardwareacceptatie.

Deze route verandert de eigen kernel, de Security Floor, de privacy-eisen of de
vaste acceptatiematrix niet. In de actuele voortgangsmeter is H01 geaccepteerd,
maar de volledige releasecriteria FR-H01 tot en met FR-H03 blijven zelfstandige
acceptatiegates; gedeeltelijk driverwerk kent geen nieuwe punten toe.

## Bronnen

### Primaire repositorybronnen

- Ginux, `README.md`, commit [`c86945b`](https://github.com/gmkurtzer/ginux/blob/c86945b1ae246a7c9e4bd831a66e5179d55de95e/README.md), en [ADR 0002: Unified Boot Image and Driver Boundary](https://github.com/gmkurtzer/ginux/blob/c86945b1ae246a7c9e4bd831a66e5179d55de95e/docs/decisions/0002-unified-image-and-driver-boundary.md).
- VibeOS, `README.md`, commit [`866bfb9`](https://github.com/kaansenol5/VibeOS/blob/866bfb9b7fc554b5b8e0939b4626bcf0416b8b19/README.md).
- vib-OS, `README.md` en wijzigingsgeschiedenis, commit [`73b6429`](https://github.com/viralcode/vib-OS/commit/73b64299bf6e591e738c58c9d1845f0babdd6f58).
- Open Driver Interface, [README](https://github.com/Open-Driver-Interface/odi/blob/ef58952c4a7b27338cb47bdb9401d4b6f970143c/README.md), [quickstart/portcontract](https://github.com/Open-Driver-Interface/odi/blob/ef58952c4a7b27338cb47bdb9401d4b6f970143c/docs/quickstart.md) en [dependency-/portnotities](https://github.com/Open-Driver-Interface/odi/blob/ef58952c4a7b27338cb47bdb9401d4b6f970143c/docs/dependencies.md).
- Dion Dokter, [device-driver-toolkit README](https://github.com/diondokter/device-driver/blob/036fb189f382d1ced62fa2695d43b3a69dcdaa73/README.md), [runtimemanifest](https://github.com/diondokter/device-driver/blob/036fb189f382d1ced62fa2695d43b3a69dcdaa73/device-driver/Cargo.toml) en [repositorylicentie](https://github.com/diondokter/device-driver/blob/036fb189f382d1ced62fa2695d43b3a69dcdaa73/LICENSE-MIT).
- rust-osdev, vastgezette [ACPI README](https://github.com/rust-osdev/acpi/blob/386035c1badb74e8bd5f3a63b385e77c60d79b1f/README.md) en [Cargo.toml](https://github.com/rust-osdev/acpi/blob/386035c1badb74e8bd5f3a63b385e77c60d79b1f/Cargo.toml), [PCI Cargo.toml](https://github.com/rust-osdev/pci_types/blob/eff856b6bab81adbe1bda0ab750009f3786cb8d3/Cargo.toml), [xHCI README](https://github.com/rust-osdev/xhci/blob/f2254c86d5ba12754259bf0ae483c6c8dc821aad/README.md) en [Cargo.toml](https://github.com/rust-osdev/xhci/blob/f2254c86d5ba12754259bf0ae483c6c8dc821aad/Cargo.toml), [USB-repository](https://github.com/rust-osdev/usb/tree/75e219179df7e0961aa42e18774beb8878765341), [uart_16550 Cargo.toml](https://github.com/rust-osdev/uart_16550/blob/770cbd28330de1cc461d62d8fc8ac5814c673304/Cargo.toml), [virtio-spec-rs README](https://github.com/rust-osdev/virtio-spec-rs/blob/ad565bd701e93fa47bc288d45701e1ae53ac3c12/README.md) en [Cargo.toml](https://github.com/rust-osdev/virtio-spec-rs/blob/ad565bd701e93fa47bc288d45701e1ae53ac3c12/Cargo.toml), en [organisatie-repositories](https://github.com/orgs/rust-osdev/repositories).

### Technische specificaties en documentatie

- Linux Kernel Documentation, [The Linux Kernel Driver Interface](https://www.kernel.org/doc/html/latest/process/stable-api-nonsense.html) en [Driver implementer's API guide](https://docs.kernel.org/driver-api/index.html). Interne kernel-/driver-API en syscall-ABI zijn verschillende interfaces.
- OASIS, *Virtual I/O Device (VIRTIO) Version 1.2*, [specificatie](https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html).
- UEFI Forum, *UEFI Specification 2.11*, [hoofdstuk 12: Console Support, inclusief Graphics Output Protocol](https://uefi.org/specs/UEFI/2.11/12_Protocols_Console_Support.html).
- Nieuw OS, [G0 native grafische basis](../../implementation/GRAPHICS-G0.md), [VirtIO-net-taakcontract](../../implementation/VIRTIO-NET.md), [volledige release en driveracceptatie](../../FULL-TEST-RELEASE.md), [virtio-blk](../../../drivers/virtio-blk/README.md) en [virtio-snd](../../../drivers/virtio-snd/README.md).

De externe repositoryclaims zijn gelezen op 3 november 2026; bij aanvullende
broncode zijn bovenstaande commit-ID's vastgezet. De Linux-, OASIS- en UEFI-
bronnen zijn geraadpleegd als technische referenties; de UEFI-host
weigerde in de eerdere sessie directe paginaophaling, zodat voor de GOP-grens
de specificatie-inhoudsopgave en de bestaande Nieuw OS G0-specificatie zijn
gebruikt. Er zijn geen externe repositories gekloond of gebouwd, geen code
overgenomen, geen toolkit als dependency toegevoegd en geen QEMU- of fysieke
hardwaretests uitgevoerd. De huidige toolchain is Rust 1.90.0; in het bijzonder
is `uart_16550` 0.9.0 met Rust 1.91.0 daarom niet als compatibel behandeld.
