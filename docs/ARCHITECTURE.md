# Architectuur en benodigde kennis

> Update revisie 5: de eerste bouwkeuzes staan vast in [implementation/KERNEL-AND-SECURITY-CONTRACT.md](implementation/KERNEL-AND-SECURITY-CONTRACT.md). Gebruik dat document voor directe implementatie; dit bestand bewaart de ambitie en achtergrond voor vervolgfases.

Historische status van dit achtergrondbestand: onderzoeksvoorstel van 26 oktober 2026. De eigen kernel is inmiddels tot en met M4 geïmplementeerd en in het vastgelegde QEMU-profiel getest; zie PROJECT-STATE.md en verification/m2-m4/. Onderstaande vervolgfasekeuzes blijven voorstellen tenzij het actuele bewijs ze uitdrukkelijk afdekt.

## 1. Wat ‘een eigen OS’ hier betekent

Het doel is een eigen kernel, eigen systeemservices en eigen gebruikerservaring, zonder Linux-, BSD-, XNU-, Zircon- of seL4-kernel als onderlaag. Het wordt dus geen Linux-distributie met een andere desktop.

Een eigen OS hoeft niet elk hulpmiddel opnieuw uit te vinden. Rust/LLVM, Git, QEMU, UEFI-firmware, openbare hardwarestandaarden en zorgvuldig gekozen bibliotheken zijn toegestaan. Ook een ontwikkelcomputer met Windows of Linux maakt het doel-OS geen afgeleide van dat host-OS. Houd gebruikte onderdelen, versies, licenties en herkomst bij. Neem geen bestaande kernelcode stilzwijgend over.

Principes als processen, virtueel geheugen, microkernels en capabilities zijn bestaande technische kennis. Originaliteit moet vooral ontstaan uit een coherent product, een eigen implementatie en eigen interactieontwerp. Een nieuw ongetoetst beveiligingsmechanisme is geen voordeel op zichzelf.

Er bestaat spanning tussen ‘alles zelf’ en ‘zo snel mogelijk hoge zekerheid’. Een nieuwe kernel begint zonder de bewezen eigenschappen en jarenlange foutcorrectie van volwassen projecten. seL4 kan een onderzoeksreferentie zijn; zijn bewijzen gelden niet voor onze code [A7]. Als hoge zekerheid uiteindelijk belangrijker wordt dan een eigen kern, verdient die scopekeuze een expliciete heroverweging.

## 2. Aanbevolen eerste platform

| Onderdeel | Eerste keuze | Reden en grens |
|---|---|---|
| Processor | x86_64, één virtuele CPU | Een afgebakende eerste implementatie. Een apart opt-in profiel start AP's begrensd (ADR-070); SMP-scheduling volgt later. |
| Machine | QEMU, een vastgelegde q35-machineversie | Herhaalbare virtuele hardware; geen afhankelijkheid van mijn specifieke laptop. |
| Accelerator | TCG als CI-baseline; lokale acceleratie optioneel | TCG vereist geen beschikbare hardwarevirtualisatie. Gebruik geen TCG-resultaten als bewijs voor prestaties op echte hardware. |
| Firmware | OVMF/UEFI, versies en hashes vastgelegd | Standaard bootinterface. Firmware is een afhankelijkheid en onderdeel van het dreigingsmodel. |
| Loader | Kleine eigen UEFI-loader in Rust | Laadt de eigen kernel en geeft gecontroleerd geheugenkaart en bootinformatie door. |
| Loader-target | `x86_64-unknown-uefi` | Dit produceert een UEFI-programma; het is niet op zichzelf een onafhankelijke kernel. |
| Kernel-target | `x86_64-unknown-none`, `#![no_std]` | Eigen bare-metal ELF-kernel, zonder host-systeemaanroepen. |
| Eerste uitvoer | Seriële log met vaste succes-/foutcodes | Automatisch toetsbaar zonder grafische desktop. |
| Apparaten | Eerst timer/interruptcontroller; daarna beperkte virtio-apparaten | Kleiner oppervlak dan meteen echte wifi-, GPU- en USB-hardware. |
| Opslag | Alleen gegenereerde testimages | Geen echte schijven, partities, USB-flash of telefoon ontgrendelen in de eerste fasen. |
| Netwerk | Uit in de eerste boot- en isolatiefasen | Netwerk pas toevoegen met eigen dreigingsmodel en tests. |

Bronbasis: Rust documenteert aparte UEFI- en bare-metal-targets. Bij `x86_64-unknown-none` ontbreken `std` en een standaardallocator; `alloc` vraagt een eigen allocator [A1–A2]. QEMU biedt systeememulatie, verschillende accelerators en virtio-apparaten; machine- en architectuurkeuzes zijn expliciet en niet onderling uitwisselbaar [A3–A4]. Virtio is een gestandaardiseerde interface voor virtuele apparaten [A5].

De loader moet de overgang na `ExitBootServices` correct afhandelen. Geheugen dat de firmware eerder beheerde, mag alleen volgens het overdrachtscontract worden hergebruikt. Een screenshot met ‘Hello world’ terwijl firmware-services nog al het werk doen, mag niet als zelfstandig werkende kernel worden gerapporteerd. De exacte boot-ABI, ELF-segmentvalidatie en geheugenkaartsemantiek krijgen vóór implementatie een ADR.

## 3. Architectuurrichting: kleine capability-microkernel

Houd het privilegieerde hart klein: adresruimten, threads/scheduling, interrupts, beperkte IPC en afdwingbare toegang tot kernelobjecten. Bestandssystemen, netwerkdiensten, pakketbeheer, grafische compositie en zoveel mogelijk drivers draaien als geïsoleerde processen. Dit is een onderzoeksrichting; IPC-complexiteit en prestaties moeten worden gemeten.

Een capability is hier een door de kernel gecontroleerd recht op een specifiek object, met expliciete bewerkingen. De gebruiker geeft bijvoorbeeld één gekozen bestand vrij aan een app; de app krijgt niet automatisch toegang tot de hele thuismap. Een waarde raden of kopiëren uit een ander proces mag geen toegang opleveren. Fuchsia illustreert procesgebonden handles met afzonderlijke rechten; het is een referentie, geen over te nemen ABI [A6].

Ontwerpregels:

1. Standaard geen globale bestands-, netwerk-, camera- of microfoontoegang voor applicaties.
2. Rechten doorgeven kan alleen expliciet en nooit met meer bevoegdheden dan de afzender bezit.
3. Leg objectlevensduur, rechtenvermindering, handlehergebruik en intrekking vast. Intrekking kan toekomstige toegang stoppen, maar reeds gelezen gegevens niet terughalen.
4. IPC heeft begrensde berichtgrootte, gevalideerde lengtes, quota, tijdslimieten en duidelijke eigendomsoverdracht.
5. Een systeemservice is niet automatisch vertrouwd voor alle gegevens. Analyseer welke componenten voor welke veiligheidsclaim vertrouwd moeten worden.
6. Een driver in gebruikersruimte is nog niet volledig geïsoleerd wanneer DMA buiten zijn geheugen kan komen. IOMMU-beheer en toegestane buffers moeten onderdeel zijn van echte hardwareondersteuning.
7. Geen ‘root’-achtig algemeen recht in een normale app. Een beheerhandeling loopt via een beperkte beheerinterface; fysieke eigenaarcontrole wordt apart ontworpen.
8. Rust vermindert bepaalde geheugenfouten maar bewijst geen correct privilegebeleid. Elk `unsafe`-blok beschrijft invariant, aannames, aanroepvoorwaarden en controlebewijs.

De complete beveiligingsuitwerking staat in [SECURITY-AND-QUANTUM.md](SECURITY-AND-QUANTUM.md).

## 4. Productrichting die meer is dan een andere desktop

Herziene privacygrens (27 oktober 2026): opslag lokaal of end-to-end versleuteld op een expliciet gekoppelde eigen NAS of NAS van vrienden/familie. Internet is ondersteunde productfunctionaliteit met afzonderlijke rechten; remote NAS-verkeer vereist toestemming gebonden aan de geverifieerde peer. NAS-beheer geeft ruimte-/verwijderrecht, geen inhoudssleutel. AI-inferentie blijft uitsluitend op het apparaat. Leverancierscloud, stille relays en cloudfallback zijn geen OS-opties. Zie [LOCAL-PRIVACY-CONTRACT.md](LOCAL-PRIVACY-CONTRACT.md). Derdenapps mogen via begrensde rechten internet gebruiken; dat verleent geen automatische toegang tot OS- of NAS-gegevens.

Aanbevolen eerste producthypothese: een lokaal werkend persoonlijk systeem met begrijpelijke, tijdelijke toegang per taak. Eén consistent permissiemodel voor apps én toekomstige AI-assistenten, met een zichtbaar overzicht ‘welke app kan nu bij welke gegevens?’. Dit is een ontwerpvoorstel, geen claim van wereldwijde nieuwheid.

| Doel | Concreet productgedrag | Te onderzoeken bewijs |
|---|---|---|
| Gebruiksvriendelijk | Werkt zonder online account; gewone taken via consistente instellingen; goede herstelroute | Gebruikersonderzoek met taakvoltooiing, fouten en herstelbaarheid |
| Privacyvriendelijk | Geen verplichte leverancierscloud of OS-telemetrie; E2EE naar persoonlijke NAS en optioneel opaque client-side versleutelde objecten bij een expliciet gekozen externe provider; beheer/provider krijgt geen inhoudssleutel; AI zonder netwerkrecht | Verkeersmetingen, NAS-/provideruitval, vals endpoint, providercredentials-zonder-key, manipulatie/rollback en stille uitwijkpogingen testen |
| Zelf in te regelen | Verwisselbare shell, sneltoetsen en services; exporteerbare configuratie | Veilige standaardconfiguratie blijft intact bij normale personalisatie |
| Europese autonomie | Open formaten, lokale taal/locale, onafhankelijke updatesleutels en verwisselbare diensten | Werkt zonder verplicht Apple-, Google- of Microsoft-account of centrale dienst |
| Toegankelijk | Toetsenbordbediening, schaalbare tekst, hoog contrast, later semantische toegankelijkheidsboom en schermlezer | Tests met gebruikers en ondersteunende technologie |
| Licht en snel | Beperkte achtergrondtaken, begrensd geheugengebruik, responsieve invoer | Vastgelegde RAM-, CPU-, latentie- en energieprofielen op referentiehardware |
| Breed inzetbaar | Gedeelde kern en API, aparte desktop/touch/embedded-profielen | Dezelfde contracttests per ondersteunde platformport |

Geen kopie van vensterdecoraties is nodig om vertrouwde handelingen te ondersteunen. Laat vormgeving volgen uit taken. Een complete browserengine, kantooromgeving en appwinkel zijn geen eerste kernelmijlpaal. Hun kwaliteit en compatibiliteit kunnen later bepalender worden voor adoptie dan de kernel.

## 5. ‘Op elk device’ vertalen naar toetsbare ondersteuning

Eén broncodebasis kan meerdere architecturen ondersteunen; één ongewijzigd kernelbestand draait niet vanzelf op alle processoren. Bovendien verschilt hardware binnen dezelfde ISA. Een ARM64-kernel is daarmee nog geen bruikbaar telefoon-OS.

| Platform | Plan | Voorwaarden |
|---|---|---|
| QEMU x86_64 | Eerste ondersteunde doel | Vastgelegde machine, firmware, CPU-features en apparaten |
| Eén echte x86_64-pc | Na VM-isolatie en hersteltests | Gedocumenteerde hardware, aparte testschijf, veilige boot-/herstelroute, DMA-beleid |
| QEMU AArch64 | Eerste portabiliteitsproef na kerncontracten | Eigen interrupt-, MMU-, timer-, contextwissel- en bootcode |
| RISC-V 64-bit | Mogelijke latere port | Expliciete ISA-extensies, privilegeomgeving, boardinterface en testset |
| Smartphones/tablets | Alleen concreet gekozen modellen | Ontgrendelbaarheid, bootketen, SoC-/GPU-/touch-/camera-/modemdrivers, firmware en energiebeheer |
| Microcontrollers zonder MMU | Ander beveiligingsprofiel of buiten scope | Geen identieke procesisolatie beloven; eventueel aparte MPU/RTOS-benadering |
| Quantumhardware | Vanaf ontwerp Q0–Q4 onderzoeken: lokale uitvoering en eigen OS op hybride quantumcomputer | Exact uitvoeringsmodel en lokale hardware vereist; zie QUANTUM-RESEARCH.md |

Android documenteert dat veel toestellen vergrendeld worden geleverd en dat ontgrendeling apparaatbeleid volgt en gegevens wist [A8]. De HAL-documentatie maakt expliciet dat leveranciers apparaatspecifieke functies moeten implementeren [A9]. **Ondersteuning van mijn OnePlus 13 is nog niet vastgesteld.** Hiervoor ontbreekt een modelspecifieke port, driverinventaris en geteste boot-/herstelroute. Flash het toestel nu niet.

Compatibiliteit met bestaande apps is een afzonderlijk productproject. Native apps vragen een SDK, API/ABI en distributiepad. Webapps vragen een port van een grote browserengine. Windows-/Android-/Linux-apps vragen compatibiliteitslagen of gast-VM’s, licentiecontrole en extra isolatie; ondersteuning volgt niet uit de keuze Rust of microkernel. Een optionele Linux-gast is een andere keuze dan een Linux-kernel als basis, en wordt pas na expliciete scopebeslissing toegevoegd.

## 6. Welke kennis Codex nodig heeft

Codex heeft geen grote kennisdump nodig die alle boeken vervangt. Het heeft de juiste versie van primaire documentatie, repo-contracten, meetbare eisen en de mogelijkheid nodig om elke aanname te testen. Elk taakbriefje noemt welke bron en welk hoofdstuk relevant zijn.

| Domein | Benodigde kennis | Te bewaren projectkennis | Aantoonbaar resultaat |
|---|---|---|---|
| Computerarchitectuur | Privilegeniveaus, MMU, interrupts, TLB, caches, atomics, DMA, geheugenvolgorde | CPU-/boardprofiel, featurebaseline, handleidingverwijzingen | Positieve en negatieve hardwaretests |
| Boot/linking | UEFI, ELF/PE, linkerlayout, calling conventions, boot-services-overgang | Boot-ABI, segmentregels, geheugenkaartcontract | Boot na firmware-overdracht, corrupte image geweigerd |
| Rust/systemen | Ownership, lifetimes, `no_std`, allocators, FFI, inline assembly, UB | Toolchainpin, dependencybeleid, `unsafe`-register | Hosttests voor pure logica plus doeltests |
| Kernelgeheugen | Pageframes, mappings, W^X, user/kernel-grens, objectlevensduur | Invarianten en adresruimte-indeling | Geen overlap, toegangsfouten geïsoleerd |
| Scheduling | Threads, contextwisseling, timers, prioriteiten, fairness, later SMP | Schedulercontract en quota | Voortgang en herstel onder foutbelasting |
| Bevoegdheden/IPC | Capabilities, delegatie, revocatie, confused deputy, TOCTOU | Syscall-/IPC-/rechtencontracten | Ongeautoriseerde toegang faalt aantoonbaar |
| Drivers | PCI, virtio, DMA, interrupts, buffers, reset/power | Ondersteunde device-ID’s en features | Malformed input en drivercrash begrensd |
| Opslag | Block I/O, metadata, crashconsistentie, flush, herstel | On-diskformaat en migratiebeleid | Power-cut/fault-injection met consistente uitkomst |
| Netwerk | Ethernet/IP, TCP/UDP, DNS, TLS, resource-exhaustion | Stackkeuze, poortbeleid, parsercontracten | Protocoltests, fuzzing, netwerk uit standaard |
| Cryptografie | Entropie, sleutelbeheer, actuele standaarden, PQC, replay/downgrade | Crypto-inventaris en library-/parameterbesluiten | Bekende testvectoren en foutpaden |
| Updates | Verificatie, sleuteldraaiing, A/B-slots, rollback versus anti-rollback | Updatemanifest en herstelontwerp | Onderbroken/malafide update laat herstel toe |
| Grafisch/invoer | Compositor, fonts, Unicode, touch, accessibility, GPU | UI-/inputcontracten en dreigingsmodel | Responsiviteit, invoerisolatie, toegankelijke taken |
| Apps/ecosysteem | Executable loader, SDK, ABI-versies, packages, signing | Appmanifest en compatibiliteitsbeleid | Kleine native app zonder verborgen hostafhankelijkheid |
| Kwaliteit | Fuzzing, modelchecking, propertytests, traceanalyse, reproduceerbare builds | Acceptatiecriteria, testlogs, regressiebaselines | Andere bouwer reproduceert uitkomst |
| Governance | Licenties, herkomst, maintainerrollen, vulnerability disclosure | Licentiebeleid, bijdragenbeleid, beslislog | Controleerbare herkomst en releaseprocedure |

Rust’s bare-metal-target ondersteunt geen normale uitvoering van de Rust-testsuite [A1]. Ontwerp dus twee testlagen: host-uitvoerbare pure algoritmen en integratietests die de echte kernel in QEMU starten. Een geslaagde hosttest bewijst niet dat de kernelboot werkt.

## 7. Primaire bronnen (APA 7)

Raadpleegdatum voor veranderlijke documentatie: 26 oktober 2026. Dit bestand bevat eigen ontwerpkeuzes op basis van deze bronnen; zij onderschrijven dit project niet.

- **A1.** The Rust Project. (z.d.). *x86_64-unknown-none*. The rustc book. https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-none.html
- **A2.** The Rust Project. (z.d.). *\*-unknown-uefi*. The rustc book. https://doc.rust-lang.org/rustc/platform-support/unknown-uefi.html
- **A3.** QEMU Project. (z.d.). *Introduction*. QEMU documentation. https://www.qemu.org/docs/master/system/introduction.html
- **A4.** QEMU Project. (z.d.). *Invocation*. QEMU documentation. https://www.qemu.org/docs/master/system/invocation.html
- **A5.** OASIS. (2022, 1 juli). *Virtual I/O Device (VIRTIO) version 1.2: Committee specification 01*. https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html
- **A6.** Fuchsia Authors. (z.d.). *Zircon handles*. https://fuchsia.dev/fuchsia-src/concepts/kernel/handles
- **A7.** seL4 Foundation. (z.d.). *Verification*. https://sel4.systems/Verification/
- **A8.** Android Open Source Project. (z.d.). *Lock and unlock the bootloader*. https://source.android.com/docs/core/architecture/bootloader/locking_unlocking
- **A9.** Android Open Source Project. (z.d.). *Hardware abstraction layer (HAL) overview*. https://source.android.com/docs/core/architecture/hal
- **A10.** UEFI Forum. (2024). *UEFI specification 2.11*. https://uefi.org/specs/UEFI/2.11/

De UEFI-inhoudsopgave is geraadpleegd; het specifieke hoofdstuk over boot-services kon in dit onderzoek niet volledig worden opgehaald. De implementatieagent moet dat hoofdstuk daadwerkelijk lezen vóór bootcode te schrijven.

