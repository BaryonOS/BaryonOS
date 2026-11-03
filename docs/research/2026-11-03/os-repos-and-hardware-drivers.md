# Vergelijking van OS-repositories en driverstrategie

**Datum:** 3 november 2026
**Onderzochte Nieuw OS-bron:** `99d132768fd72276caa4a3b3e9a09f8f1909f0ed`
**Status:** documentatieonderzoek en voorstel; geen architectuurbesluit, codeport of hardwaretest.

## Conclusie

De onderzochte repositories bieden geen kant-en-klare verzameling breed
herbruikbare Linux-hardwaredrivers. De snelste passende route voor Nieuw OS is
de eigen kernel behouden, de vaste QEMU-doelen via VirtIO afmaken en GOP
(UEFI's framebufferinterface) gebruiken voor basale beeldweergave. Kies daarna
een concreet fysiek referentieprofiel en port alleen doelgericht geschikte
drivers of protocolcode. Een driver in een apart proces past bij de
capabilityarchitectuur, maar is pas werkelijk geïsoleerd als geheugen, MMIO,
interrupts en DMA begrensd zijn; voor DMA is een correct ingerichte IOMMU
relevant.

Dit versnelt de ondersteunde testmatrix en beperkt het aantal apparaatspecifieke
implementaties. Het belooft geen brede ondersteuning van willekeurige pc's,
wifi-adapters of GPU's.

## Wat de drie repositories daadwerkelijk laten zien

| Repository en onderzochte bron | Opbouw en driveraanpak | Wat Nieuw OS kan meenemen | Bewijsgrens |
|---|---|---|---|
| [Ginux](https://github.com/gmkurtzer/ginux/tree/c86945b1ae246a7c9e4bd831a66e5179d55de95e), commit `c86945b` (3 nov. 2026) | Eigen Rust x86-64-kernel, UEFI-image, eigen proces-/domeinmodel en Linux-userspace-ABI. De README noemt VirtIO-blok/netwerk op QEMU Q35. ADR 0002 stelt Linux `.ko`-compatibiliteit uit; de README zegt dat er geen stabiele third-party driver-ABI is en dat algemene hardwaredekking toekomstwerk is. | Heldere onderscheidingen tussen eigen kernel, Linux-programmacompatibiliteit en drivers; echte distributie-workloads als integratietest; beperkingen per ondersteund VM-profiel opschrijven. | De README noemt onder meer een Rocky Linux 10 systemd-pad, maar dat is Ginux' eigen gerapporteerde testbewijs. Geen onafhankelijke reproductie in dit onderzoek. De “11 dagen” beschrijven een agentische ontwikkelmethode, niet een universele driveroplossing. |
| [VibeOS](https://github.com/kaansenol5/VibeOS/tree/866bfb9b7fc554b5b8e0939b4626bcf0416b8b19), commit `866bfb9` (27 jan. 2026) | Zelfgebouwd AArch64-hobby-OS voor QEMU en volgens de README Raspberry Pi Zero 2W, met eigen USB DWC2/HID- en eMMC-paden. De README noemt op de Pi geen netwerk- of audiodriver en beschrijft USB-beperkingen, waaronder geen hotplug. | Kies een concreet board en maak een verticale doorsnede met echte apparaattests; documenteer ontbrekende functies en storingsgedrag even expliciet als de demo. | README-claims zijn niet opnieuw op QEMU of een Pi uitgevoerd. De bijbehorende `CLAUDE.md` vermeldt dat zijn status uit sessie 40 komt; die verouderde technische notities zijn niet gebruikt als actuele verificatie. |
| [vib-OS](https://github.com/viralcode/vib-OS/tree/73b64299bf6e591e738c58c9d1845f0babdd6f58), commit `73b6429` (29 jan. 2026) | Eigen C/assembly-kernel met eigen virtio-, video-, netwerk-, NVMe-, USB- en andere driverdirectories; richt zich vooral op ARM64 en claimt meerdere architecturen. De README bevat echter tegenstrijdige x86-64-status: de architectuursectie noemt een x86-64-build, terwijl dezelfde bron zegt dat die “will be released soon”; de meest recente commit verwijdert de x86-64-buildinstructies. | Houd een concrete architectuur-/device-matrix bij en scheid “code bestaat”, “buildt”, “boot in VM” en “getest op hardware”. | De README, screenshots en driverbestandsnamen zijn geen bewijs dat alle genoemde apparaten of platformen werken. Geen builds of hardwaretests uitgevoerd. |

De belangrijkste correctie op de aangehaalde Google-samenvatting: de huidige
Ginux-bron beschrijft Linux-*userspace*-compatibiliteit (Linux-programma's
draaien boven de eigen kernel), niet een werkende wrapper voor duizenden Linux
kerneldrivers. De eigen Ginux-ADR zegt expliciet dat binaire Linux-modules nog
zijn uitgesteld. Linux documenteert bovendien dat de kernel geen stabiele
interne module-/driver-API belooft; die is iets anders dan de veel stabielere
syscall-ABI voor gebruikersprogramma's. Een Linux-driver “foppen” vraagt dus
meer dan een dunne syscall-laag: bus-, geheugen-, DMA-, interrupt-, apparaat-
en subsystemcontracten moeten passen.

## Beoordeling van de voorgestelde shortcuts

| Voorstel | Beoordeling voor Nieuw OS |
|---|---|
| Linux-drivers wrappen | **Niet als snelle route.** Een Linux-userspace-ABI helpt applicaties, niet kerneldrivers. Een compatibiliteitslaag voor `.ko`-modules zou een omvangrijk Linux-kernel-/subsystemmodel en een nieuwe vertrouwensgrens introduceren. Onderzoek dit hoogstens later als afgebakende haalbaarheidsproef; neem geen bestaande kernel als basis. Elke codeport vereist afzonderlijke licentie-, herkomst-, API- en beveiligingsreview. |
| VirtIO en UEFI GOP | **Direct bruikbaar binnen hun grenzen.** VirtIO levert gestandaardiseerde apparaten aan VM's en past bij QEMU/CI; het is geen fysieke NIC-/NVMe-/GPU-driver voor gewone pc's. GOP laat de loader een firmwaremodus en framebuffer doorgeven, bruikbaar voor initiële pixels zonder GPU-versnelling. Het vervangt geen native GPU-driver, modesetting, audio, wifi of post-boot USB-stack. |
| AI als driver-portfabriek | **Alleen als hulpmiddel bij één port tegelijk.** Een agent kan specificaties, drivercode en tests voorbereiden, maar moet dezelfde bron-, licentie-, API-, DMA- en negatieve-testgates doorlopen als handgeschreven code. Automatisch herschrijven is geen bewijs van correctheid of compatibiliteit. |
| Drivers in gebruikersruimte | **Architectonisch passend, maar niet gratis of automatisch veilig.** Een procescrash kan lokaal worden herstart; eerder door het device uitgevoerde DMA kan alsnog geheugen beschadigen. Gebruik alleen beperkte capabilities, gepinde/afgeschermde buffers, reset/quiescencecontracten en waar nodig een correct ingerichte IOMMU. Een procesgrens alleen voorkomt geen DMA. |
| WASM als OS-basis | **Niet passend.** Dan levert een bestaand host-OS of browser de hardwaredrivers. Dat kan applicaties sandboxen, maar levert geen zelfstandig Nieuw OS met een eigen kernel op en voldoet dus niet aan de productbasis. |

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

### Technische specificaties en documentatie

- Linux Kernel Documentation, [The Linux Kernel Driver Interface](https://www.kernel.org/doc/html/latest/process/stable-api-nonsense.html) en [Driver implementer's API guide](https://docs.kernel.org/driver-api/index.html). Interne kernel-/driver-API en syscall-ABI zijn verschillende interfaces.
- OASIS, *Virtual I/O Device (VIRTIO) Version 1.2*, [specificatie](https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html).
- UEFI Forum, *UEFI Specification 2.11*, [hoofdstuk 12: Console Support, inclusief Graphics Output Protocol](https://uefi.org/specs/UEFI/2.11/12_Protocols_Console_Support.html).
- Nieuw OS, [G0 native grafische basis](../../implementation/GRAPHICS-G0.md), [VirtIO-net-taakcontract](../../implementation/VIRTIO-NET.md), [volledige release en driveracceptatie](../../FULL-TEST-RELEASE.md), [virtio-blk](../../../drivers/virtio-blk/README.md) en [virtio-snd](../../../drivers/virtio-snd/README.md).

De externe repositoryclaims zijn gelezen op 3 november 2026. De Linux-, OASIS-
en UEFI-bronnen zijn geraadpleegd als technische referenties; de UEFI-host
weigerde in deze sessie directe paginaophaling, zodat voor de GOP-beperking de
specificatie-inhoudsopgave en de bestaande Nieuw OS G0-specificatie zijn
gebruikt. Er zijn geen externe repositories gekloond of gebouwd, geen code
overgenomen en geen QEMU- of fysieke hardwaretests uitgevoerd.
