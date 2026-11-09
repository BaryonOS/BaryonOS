# AUDIT-09 — Hardwareabstractie, drivers en energiebeheer

**Bronmetadata:** primaire repositorybron: *Nieuw OS — “Add OS specialist
agent profiles”*, auteur/organisatie S.W.J.B.M. van Ieperen, 3 november 2026,
commit `beafdedfc188154fe7285848fbd411e651013f54`
([onveranderlijke snapshot](https://github.com/simonvanieperen/nieuw-os/tree/beafdedfc188154fe7285848fbd411e651013f54)).
Alle lokale `pad:regel`-citaten hieronder zijn repository-observaties uit die
snapshot. Claims over virtio, DMA/IOMMU, ACPI, interrupts en power states zijn
niet onafhankelijk getoetst aan primaire hardwarebronnen en blijven
`ONBEKEND`/onderzoeksvraag.

**Opdracht:** “Beoordeel hardwareabstractie, drivers, DMA/IOMMU, ACPI,
energiebeheer en de route van QEMU naar echte ondersteunde hardware.”

**Bron:** `simonvanieperen/nieuw-os`, commit
`beafdedfc188154fe7285848fbd411e651013f54`. Deze worktree begon schoon op
branch `simonvanieperen-miniature-memory`; de commit is lokaal aanwezig en de
worktree staat voor deze beoordeling detached op die bevroren bron. Dit rapport
is begrensd tot hardware en levert geen algemene veiligheids-, conformiteits-,
productrijpheids- of hardwarecompatibiliteitsclaim.

## Conclusie

**BLOKKEREND voor een claim van fysieke hardwareondersteuning:** de code en
testen vormen een substantiële, eigen x86_64/UEFI/QEMU-ontwikkeling, maar de
hardwarecontracten zijn nog op vaste virtuele apparaten afgestemd. Er is geen
benoemd fysiek referentieprofiel, geen fysieke acceptatie en geen DMA-afscherming
door een werkende IOMMU. De scorekaart registreert H01 als geaccepteerd voor
virtio-bloktransport in QEMU; H02–H06 blijven open. Dat onderscheidt een reëel
native VM-deelbewijs van ondersteuning van een pc
(`docs/progress.json:199-249`).

Er zijn nuttige bestaande veiligheidsmaatregelen: kernel-owned DMA-pools,
gevalideerde descriptorplannen, geen appadressen als DMA-doel, reset/timeout-
afhandeling en gepinde/quarantainebuffers. Deze beheersen de **protocolconforme
virtio/QEMU-profielen**; ze zijn geen vervanging voor een IOMMU en beschermen
niet tegen een apparaat dat buiten gepubliceerde descriptors DMA uitvoert
(`docs/DECISIONS.md:636-646`; `drivers/virtio-blk/src/lib.rs:3-5,135-165,475-484`).

## Bevindingen

### 1. BLOKKEREND — geen vastgesteld fysiek supportprofiel

De vaste hardware-roadmap zegt expliciet dat virtio, PS/2, framebuffer en RTC
virtuele profielen zijn en dat fysieke referentiehardware, IOMMU/DMA-bewijs en
brede driverdekking ontbreken. H06 vereist meerdere vooraf benoemde pc-
configuraties; de huidige matrix noemt nog geen modellen, firmwareversies of
ondersteunde onderdelen (`docs/audit/os-completeness/matrix.json:218-248`;
`docs/progress.json:244-249`). De bestaande `HW-FR`/`HW-AI`-route vereist ook
een concreet profiel, gecontroleerde image, instructies en herstelroute
(`docs/FULL-TEST-RELEASE.md:81-83,630`).

Dit is een release-/supportblokkade, niet de bewering dat de QEMU-tests
waardeloos zijn. QEMU-proeven kunnen de eigen kernel en virtuele transports
valideren; ze demonstreren geen werking met echte firmware, controllers,
interrupt-routes, apparaatfouten of herstel op een fysieke machine.

### 2. BLOKKEREND — DMA is begrensd in software, niet geïsoleerd met IOMMU

ADR-026 benoemt de grens rechtstreeks: M5 bevat geen IOMMU en bewijst geen
containment van kwaadwillige DMA; de driver en protocolconforme QEMU-backend
blijven vertrouwd. De vijfpagina-pool blijft gepind; zonder reset-ACK blijft
deze in quarantaine (`docs/DECISIONS.md:636-646`). De blokdriver gebruikt
vaste BDF `00:05.0`, maximaal 256 MiB adresruimte en expliciet door de kernel
geëigende, identity-mapped DMA-pagina's
(`drivers/virtio-blk/src/lib.rs:3-5,13-14,135-165`).

De netwerk- en RNG-drivers wijzen `ACCESS_PLATFORM` af en vragen dus geen
virtio-platform/IOMMU-DMA-pad aan (`drivers/virtio-net/src/lib.rs:9,90-93`;
`drivers/virtio-rng/src/lib.rs:3-5,58-60`). Pinnen, descriptorvalidatie en
uitschakelen van PCI bus mastering beperken softwarefouten en veilig
hergebruik na reset, maar een actieve bus-master kan zonder IOMMU nog steeds
andere fysieke RAM-adressen benaderen. Verleen daarom geen fysieke
bus-masterdevice-supportclaim op basis van deze VM-contracten.

**Wat het huidige ontwerp wel afdekt:** de app krijgt geen willekeurige
fysieke adressen of raw DMA-recht; kernelcode bouwt descriptors en kopieert
data tussen de driverpool en begrensde clientbuffers. Device-reset en
quarantaine beschermen tegen voortijdig hergebruik volgens de gedocumenteerde
status/ACK-aannames. De bron stelt niet dat die maatregelen fysieke
DMA-isolatie zijn.

### 3. HOOG — drivers en machineprofiel zijn vast aan QEMU-indeling gekoppeld

Het M1-contract is QEMU q35, TCG, x86_64, één CPU en 256 MiB RAM. De kernel
beheert met `FRAME_COUNT = 65536` precies een 256-MiB-framebereik; hoger RAM
blijft gereserveerd (`docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md:11`;
`kernel/src/m2.rs:1-14`). Virtio-blok, RNG en netwerk zoeken vaste PCI BDF's
(`00:05.0`, `00:06.0`, `00:07.0`) in plaats van een gedocumenteerde
hardwarecompatibiliteits-/enumeratiematrix
(`drivers/virtio-blk/src/lib.rs:13`; `drivers/virtio-rng/src/lib.rs:17`;
`drivers/virtio-net/src/pci.rs:6,10`).

Dit is passend voor een kleine reproduceerbare emulatorbasis, maar de
veronderstellingen zijn wezenlijk: vaste virtio-ID's/capabilities, BAR- en
MMIO-profiel, adresbereik, één core, beschikbaarheid van legacy poorten en
QEMU-firmware. Er is geen aangetoonde algemene PCI/PCIe-bus-, bridge-,
interruptvector-, hotplug- of resourceallocatieroute. De QEMU-ontwikkel- en
hostacceleratievarianten zijn bovendien niet onderling uitwisselbaar bewijs;
de hardware/performance-specificatie waarschuwt dat CPU-acceleratie geen
apparaatdoorvoer of fysieke driveracceptatie oplevert
(`docs/implementation/VM-HARDWARE-AND-PERFORMANCE.md:11-24,85-103`).

### 4. HOOG — ACPI-parser is geen ACPI-boot- of energie-integratie

De huidige `BootInfo` is 128 bytes en bevat geheugenkaart-/reserveringsvelden
en één `reserved0`-veld; de loader vult daarin alleen de optionele
framebufferinformatie, niet een ACPI-root pointer of gevalideerde
FADT/DSDT-snapshot (`crates/boot-protocol/src/lib.rs:20-39`;
`boot/loader/src/main.rs:599-623`). De nieuwe `drivers/acpi-power`-crate is
een begrensde parser: hij kan uit gevalideerde FADT/DSDT-bytes een beperkte
legacy x86 S5-instelling halen, maar voert zelf geen port-I/O of
shutdownactie uit. De kernel moet handoff, geheugenreservering, tabelchecks,
autorisatie en brokeractie nog zelfstandig integreren
(`drivers/acpi-power/README.md:1-32,90-112`).

Historisch bewijs is nadrukkelijk beperkter dan een OS-poweroff: volledige
QEMU/OVMF-tabellen zijn buiten de Nieuw-OS-gast uitgelezen en op de host
geparseerd. De QEMU-run eindigde via QMP `quit`, niet via guest-shutdown; de
loaderhandoff, native poweroff en fysieke apparatuur bleven open
(`drivers/acpi-power/verification/README.md:9-14,75-93,137-160`). Dit is
E2-bewijs voor parserlogica/firmwarefixtures, niet E4-bewijs van ACPI-
integratie.

### 5. HOOG — interrupt-, klok- en SMP-contract is single-core/legacy

De kernel- en schedulercode documenteert één CPU en IF-uitgeschakelde
geserialiseerde mutaties. De timer initialiseert de legacy PIC/PIT, met
ongeveer 100 Hz; de bron noemt deze route expliciet een vast QEMU-profiel
(`kernel/src/m2.rs:1-3`; `kernel/src/m2/m3.rs:1755-1756,1896-1910`).
Er is geen aangetoonde APIC/IOAPIC/MSI-driverroute, SMP-start, inter-processor
interrupt, remote-TLB-invalidatie, timerkalibratie of suspend-aware
monotone-klok. De kernelregels stellen ook dat SMP pas na aantoonbare
single-core-invarianten aan de orde is (`kernel/AGENTS.md:4-6`;
`docs/DECISIONS.md:332-334`;
`docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md:208-212`).

Dat begrenst zowel hardwarekeuze als claims over timers, prestaties en
energieverbruik. De PIT-tick is geen wall-clock-, realtime- of
energiebeheergarantie; de code documenteert expliciet dat suspend niet in de
tickbasis wordt verwerkt (`kernel/src/m2/m3.rs:1755-1756`). De expliciete
herzieningspoort vóór SMP staat ook in `docs/DECISIONS.md:332-334`;
het beveiligingscontract markeert DMA/IOMMU en SMP als niet aanwezig
(`docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md:208-212`).

### 6. HOOG — USB, audio en GPU zijn geen complete apparaatpaden

- **USB/invoer:** de USB-crate controleert alleen framing van reeds begrensde
  descriptors. Er is geen USB-enumeratie, HID-driver of xHCI-hostcontroller,
  dus geen USB-toetsenbord-/muis- of hotplugacceptatie
  (`docs/implementation/USB-DESCRIPTOR-CORE.md:1-14,25-42`). De aanwezige
  PS/2/i8042-code is een afgebakend legacy/QEMU-pad, geen USB-vervanger op een
  pc zonder PS/2-emulatie (`kernel/src/m2/m3/ps2.rs:14-36`;
  `docs/progress.json:212-218`).
- **Audio:** `virtio-snd` heeft hosttests en een freestanding build, maar geen
  audio-uitvoer, device-I/O of kernel-/image-integratie
  (`drivers/virtio-snd/README.md:1-5,23-29`).
- **Grafisch:** er is een werkende native VM-framebuffer/shellroute, maar dat
  is geen GPU-driver, hardwareacceleratie of fysieke displaymatrix. H03 blijft
  open en vraagt fysieke referentiehardware
  (`README.md:84-95`; `docs/progress.json:220-226`;
  `docs/implementation/VM-HARDWARE-AND-PERFORMANCE.md:33-35,90`).

### 7. HOOG — slaap, hervatten, batterij en thermiek zijn niet aangetoond

De power-roadmap noemt ACPI/RTC-proeven geen bewijs van laptop-slaap,
batterij, thermiek of betrouwbaar hervatten; fysieke slaap-/power-cycleproeven
ontbreken. H05 eist herhaalde slaap/wek-, klep-, batterij-/thermiek- en
shutdowntransities met behoud van opgeslagen data en apparaat-/rechtenstatus
(`docs/audit/os-completeness/matrix.json:1299-1333`;
`docs/progress.json:236-242`). De parser hierboven is alleen voor de beperkte
S5-shutdowninstelling; S3/S0ix, wakebronnen, batterijstatus, thermische
begrenzing, CPU-frequentie, ventilatorbeleid en resume-herinitialisatie zijn
niet in de onderzochte hardwarepaden aangetoond. Noem deze functies niet
ondersteund op basis van QEMU-shutdown- of RTC-tests.

## Apparaat- en bewijsvolwassenheid

Legenda: E0 ontbrekend/ambitie; E1 ontwerp; E2 hostprototype/simulatie/hosttest;
E3 native broncode; E4 native positief; E5 native negatief; E6 herhaalde
CI-/releasegate; E7 gedocumenteerde fysieke hardwaretest; E8 onafhankelijke
menselijke audit/formele evaluatie. “Maximaal” hieronder is het hoogste niveau
dat uit de geïnspecteerde bron en de daarbij horende historische rapporten
volgt; het is geen nieuwe test op de bevroren commit. Historische rapporten
blijven aan hun eigen broncommit gebonden.

| Pad | Huidige implementatie / historisch bewijs | Max. niveau en grens |
|---|---|---|
| UEFI-loader, kernelhandoff, geheugen/CPU | Eigen Rust-loader en kernel; vaste x86_64/QEMU/OVMF-profielen met native VM-positieve en negatieve runs. | **E5**, voor de gerapporteerde VM-profielen. Geen fysieke firmware. |
| Virtio-blok | Kernel-owned pool, vaste `00:05.0`, kernelbroker; M5-rapport beschrijft 64 echte VM-runs met normale en foutcases en externe wegwerpimages. H01 staat geaccepteerd. | **E5** in QEMU voor het vaste profiel; geen fysieke drive of IOMMU-containment. Het archief is geen herhaling op deze HEAD. |
| Virtio-RNG / entropy | Vaste `00:06.0`, één QEMU-virtqueue en permanent gepinde DMA-pagina's; code zegt protocolconforme QEMU te vertrouwen. | **E3** in deze beoordeling; geen fysieke entropybron-/DMA-proef. |
| Virtio-net en netwerkstack | De afzonderlijke driverpakketproef bleef bij bron/hostmodellen: daarin zijn native PCI-discovery en NIC-RX/TX niet uitgevoerd (`drivers/virtio-net/verification/driver-acceptance.json`). Latere, apart aan hun eigen bron gebonden integratiebewijzen rapporteren echte QEMU virtio-net-pakketten voor ARP/UDP/DNS en native TCP (normaal/herstel met peertranscript): `docs/verification/full-release-fwcfg-causal/result.json`, `docs/verification/full-release-tcp-1b6a308/tcp-system-gate.json` en `docs/verification/full-release-tcp-1b6a308/0-normal/network-peer.json`. | **E5** voor de genoemde historische native VM-integraties; die latere bewijzen vallen buiten de rapport-/artefact-hashvalidatie van deze audit en zijn geen runs op deze audit-HEAD. Fysieke NIC, IOMMU/DMA-isolatie, Wi-Fi/Bluetooth en H04-hardwareacceptatie blijven onbewezen/open. |
| PS/2-invoer | QEMU-gerichte native i8042-code; geen fysieke HID-controller- of USB-bewijs. | **E3** broncode; H02 blijft open. |
| Framebuffer/shell | Native VM-framebuffer en desktopervaring, zonder fysieke GPU-acceptatie. | **E4** voor de gebonden VM-weergave; geen GPU-acceleratieclaim. |
| RTC/CMOS | Historische native QEMU-runs omvatten echte port-reads, herstelcontrole en negatieve kalender-/firmwaregevallen. | **E5** voor de gebonden QEMU/OVMF-proef; geen hardwareklok-, suspend- of monotoniciteitsgarantie. |
| USB/xHCI/HID | Alleen descriptorframing-core met hosttests/no_std-build; geen controller of device-I/O. | **E2**. |
| Audio/virtio-snd | Codec- en transportmodeltests/no_std-build; geen audiohardware-uitvoer of integratie. | **E2**. |
| ACPI S5/poweroff | Begrensde hostparser en hostparse van QEMU/OVMF-tabellen; geen BootInfo-handoff/native shutdown. | **E2**. |
| IOMMU, fysieke DMA-afscherming | Ontwerp-/risicotekst; geen IOMMU-driver, DMA-domainopzet of echte apparaatfaultproef. | **E0–E1**. |
| Suspend/resume, batterij, thermiek | Open acceptatiecriteria; geen geïmplementeerd/bewezen fysiek lifecyclepad in de onderzochte bronnen. | **E0–E1**. |
| Echte ondersteunde pc-configuratie | Geen benoemd model/firmware met hardware-uitvoering in de bron. | **E0**. Geen E7/E8 voor de beoordeelde hardwarepaden. |

De status- en bewijskaders zijn consistent met de onafhankelijke
completeness-matrix: `K-DRIVERS` is `PROTOTYPE`, `POWER-ENERGY` is `PARTIAL`;
beide noemen fysieke hardware-/DMA-grenzen als open
(`docs/audit/os-completeness/matrix.json:218-248,1299-1333`). Binnen deze audit zijn de
historische runs niet opnieuw uitgevoerd, hun hashes niet opnieuw gevalideerd
en is geen GitHub-CI-status opgevraagd.

## Aannames en resterende risico’s

- De vaste BDF-, RAM-, virtqueue- en single-CPU-contracten worden gelezen als
  bewuste emulatorprofielen, niet als universele PCI-/hardwareabstracties.
- Resetstatus en bus-master-disable readback gelden alleen binnen de
  beschreven virtio-devicecontracten; dat is geen bewijs dat fysieke hardware
  daadwerkelijk stopt met DMA.
- In-repo test- en acceptatierapporten zijn historische bronclaims. De
  rapporten zijn niet opnieuw gereproduceerd; hun vermelde commits mogen niet
  stilzwijgend als uitvoering op `beafdedf` worden gepresenteerd.
- Geen externe specificaties, websites, diensten of hardware zijn tijdens deze
  opdracht geraadpleegd of gebruikt. De repositorydocumenten verwijzen naar
  primaire standaarden; die verwijzingen zijn hier niet onafhankelijk
  geverifieerd.
- Dit is geen onafhankelijke menselijke securityaudit, formele verificatie,
  volledige drivercode-review of inventaris van ieder apparaat in de tree.

## Voorgestelde kleine hardwarebasis — voorstel, niet uitgevoerd

Maak eerst één `HW-01`-referentieprofiel met **exact** fabrikant/model,
boardrevisie, CPU/CPUID-featurebaseline, UEFI/firmwareversie, RAM-layout,
PCI-ID's, IOMMU-capabilities/-groepen en herstel-/bootroute. Als eerste
selectiecriterium: een x86_64 UEFI-desktop met aantoonbaar inschakelbare
VT-d/AMD-Vi, geïntegreerde UEFI-framebuffer, één bekabelde Ethernetcontroller,
NVMe en xHCI met standaard USB-toetsenbord/muis. Sluit in die eerste matrix
Wi-Fi, Bluetooth, audio, discrete GPU, slaap/wek en batterijclaims uit; dat
zijn latere aparte paden. **Er is in deze bron geen concreet model gekozen.**

Voorgestelde volgorde:

1. Eerst een niet-destructieve boot-/CPU-/UEFI-GOP-proef zonder hostschijf of
   bus-masterdevice; controleer BootInfo/eigenaarschap en de exacte
   firmware-/imagehashes.
2. Vóór NVMe- of NIC-DMA: PCI-inventarisatie, IOMMU-initialisatie met
   deny-by-default domains, alleen expliciete buffervertalingen en negatieve
   DMA-buiten-domainproeven; reset, bus-master-uitschakeling, fault reporting
   en quarantaineroute op het exacte apparaat.
3. Voeg daarna één controllerklasse tegelijk toe (xHCI/HID, opslag, Ethernet),
   elk met ontbrekend/stuck/malformed/hot-unplug/resetgevallen, herhaalde
   boots en reproduceerbaar bewijs. Pas later ACPI-wake, slaap/hervatten,
   batterij en thermische transitions toe met dataflush- en rechtenherstel.
4. Pas na herhaalde exacte-image-/CI-gates en fysiek uitgevoerde matrixruns
   hardwareondersteuning benoemen; H06 vraagt bovendien meerdere benoemde
   representatieve configuraties.

Dit voorstel schrijft geen driver, selecteert geen apparaat namens de
gebruiker en machtigt geen fysieke handeling.

## Uitgevoerde controles

Exacte shellcommando's en resultaten in deze worktree:

| Commando | Exit | Uitkomst |
|---|---:|---|
| `pwd && git branch --show-current && git rev-parse HEAD && git status --short && git rev-parse --show-toplevel` | 0 | Eigen worktree; beginbranch `simonvanieperen-miniature-memory`, HEAD `69809c2cb53dfb3bd77abe5352892810fba6e82d`, schoon. |
| `git cat-file -e beafdedfc188154fe7285848fbd411e651013f54^{commit} && git switch --detach beafdedfc188154fe7285848fbd411e651013f54 && git status --short && git rev-parse HEAD` | 0 | Broncommit aanwezig; detached HEAD `beafdedfc188154fe7285848fbd411e651013f54`, schoon vóór rapportage. |
| `mkdir -p docs/audits/subagents` | 0 | Alleen de toegewezen rapportmap aangemaakt. |
| `git status --short --untracked-files=all && git rev-parse HEAD && git branch --show-current && awk '/[ \t]+$/ { print "trailing whitespace at line " NR; bad=1 } END { exit bad }' docs/audits/subagents/09-hardware-drivers.md && sha256sum docs/audits/subagents/09-hardware-drivers.md` | 0 | Alleen het aangewezen rapport is nieuw; HEAD blijft `beafdedfc188154fe7285848fbd411e651013f54` (detached; branchnaam leeg); trailing-whitespacecontrole en hashberekening slagen. |

**Niet uitgevoerd:** builds, tests, QEMU-runs, firmware-/device-I/O, fysieke
hardwareproeven, externe diensten/downloads en nieuwe agents. Na het schrijven
zijn alleen de rapporttekst op trailing whitespace en de werkboomstatus
gecontroleerd; ook is de SHA-256 lokaal berekend.
