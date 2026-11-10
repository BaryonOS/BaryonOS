# Ontwerpbesluiten

**Projecteigenaar:** Simon van Ieperen.
Dit document onderscheidt de besluiten en bevoegdheden van de projecteigenaar
van die van de uitvoerende rollen. Broncitaten behouden hun oorspronkelijke
tekst en spreker.

## ADR-074 — Ticketlock, irqsave en lockdep per CPU (K06 stap 5a)

6 november 2026; alleen in `native-smp`. Dit is de eerste stap naar een
SMP-scheduler voor processen (spec K06 §4). De crate `crates/smp-lock`
(no_std, host-testbaar) levert een FIFO-ticketlock (Mellor-Crummey & Scott)
en `Lockdep`. In `kernel/src/m2/lock.rs` staat `IrqLock`: die bewaart
RFLAGS.IF, doet `cli`, legt het lockniveau vast in de lockdepset van deze
CPU en spint daarna. Bij het vrijgeven komt eerst de release, dan het
wissen van het niveau, en pas daarna het herstel van IF. De BSP krijgt een
eigen GS-anker (CPU-index 0), los van de AP-ankers. AP-specifieke lookups
(preëmptie, IST-markers) blijven de BSP dus missen.

**Lockvolgorde (buiten → binnen):** KERNEL 10, ASPACE 20, FRAMES 30,
RUNQ 40, SHOOTDOWN 50, ICR 60, SERIAL 70. Een CPU neemt alleen een niveau
dat strikt hoger ligt dan alles wat hij al vasthoudt. Een schending stopt
de VM met `SMP:LOCKDEP:ORDER:<held>-><wanted>`. Lockdep is in elke
`native-smp`-build actief. De begrensde spin met `SMP:LOCK:STUCK` is alleen
actief in debugbuilds.

**Bewijs (stand van deze commit):** hosttests in `smp-lock`: uitsluiting
onder 8 threads (vangt een mutant zonder wachtlus), FIFO, wrap,
ordeschending, held-across-switch en een compile_fail-test voor een
`Send`-guard. De AP-stresstest gebruikt nu deze ticketlock. De serial-oracle
eist `SMP:LOCKDEP:PASS` op elk niet-terminaal pad. Het negatieve profiel
`test-smp-lockdep-order` moet `SMP:LOCKDEP:ORDER:30->10` geven. **Native
QEMU-uitvoering staat nog open** (workflow `native-smp.yml`).

**Open:** nog niets gebruikt `IrqLock` buiten de zelftest. De KERNEL-lock om
M3-syscalls en de scratchslot per CPU volgen in stap 5b; de herziening van
ADR-024 krijgt daarmee ADR-075, niet ADR-074 zoals de spec voorstelde.
Lockdep heeft één bit per niveau, dus twee RUNQ-locks tegelijk vragen in
stap 5d een nestteller. Spinnen met IF=1 op KERNEL/ASPACE (§4.3) volgt in 5b.

**Alternatieven overwogen.** De bestaande TTAS-lock: vervangen, omdat die
niet FIFO is en onder TCG kan uithongeren. Lockdep alleen in debugbuilds:
verworpen, omdat de native gate releasebuilds draait.

## ADR-073 — Gepreëmpteerde kernelthreads op AP's (K06 stap 4)

5 november 2026; alleen in `native-smp`, de BSP blijft IF=0. Iedere AP
krijgt een periodieke LAPIC-timer (vector 0xd0). De stub bewaart alle 15
GPR's op de onderbroken stack, en `smp_preempt` kiest round-robin tussen
de thuislus en twee kernelthreads, elk op een eigen stack met guards. De
kernel is soft-float zonder red zone; GPR's plus het CPU-interruptframe
vormen daarom de volledige threadcontext. Er is geen FPU-, XSAVE- of
per-thread-CR3-wissel.

**Contract.** Een thread draait alleen tussen RUN en STOP. Bij STOP maskeert
de eerstvolgende tick de timer en keert de AP terug naar zijn thuislus.
Pas daarna meldt hij RESUMED. De BSP eist per AP minstens 8 ticks en 4
contextwissels, en dat beide threads vooruitgang boeken. Een AP die dat
niet haalt, wordt behandeld als een dove AP: alle AP's krijgen INIT en de
BSP werkt alleen verder.

**Bewijs:** `docs/verification/k06-smp-preempt/`. **Open:** dit zijn
kernelthreads zonder eigen adresruimte, er is geen gedeelde runqueue of
migratie, en er zijn geen interrupts of preëmptie op de BSP. Er is geen
lockvolgorde voor kernelsubsystemen en geen gebruikersprocessen op AP's;
ADR-024 is ongewijzigd. Deze stap verandert niets aan andere profielen.

**Status:** geaccepteerd in het opt-in `native-smp`-profiel; K06 blijft open.

**Alternatieven overwogen.**
- TSC-deadline-timer: uitgesteld, omdat die CPUID-detectie en kalibratie
  vraagt; de periodieke LAPIC-timer volstaat voor dit bewijs.
- Coöperatieve threads (`yield`): verworpen, omdat ze geen preëmptie
  bewijzen.
- Een volledige XSAVE/FXSAVE-context: niet nodig, want de kernel is
  soft-float zonder red zone; voor gebruikersprocessen met FPU blijft dat
  verplicht.

**Primaire bronnen.** Intel SDM Vol. 3A: APIC-timer (LVT-timer, divide
configuration, initial/current count, periodieke modus) en
interruptafhandeling (interruptframe, uitlijning van RSP bij een
privilege-loze interrupt, IRETQ). De hoofdstuknummers verschillen per
editie.

## ADR-072 — TLB-shootdown vóór hergebruik van een fysieke pagina (K06 stap 3)

5 november 2026; aanvulling op ADR-070/071, alleen in `native-smp`. Op SMP
geldt het `Frames::unmap`-contract ("PTE wissen en TLB invalideren vóór het
vrijgeven van metadata") voor **iedere** CPU die de vertaling kan cachen.
Met één gedeelde CR3 kan een CPU iedere aanwezige vertaling (ook
speculatief) cachen. Daarom geldt dit voor **alle** aliassen van het frame,
en gaat het vrijgeven van een vervangen frame in deze volgorde:
1. Het frame wissen terwijl de eigen bootstrap-alias nog bestaat.
2. Beide PTE's lokaal weghalen (venster → nieuw frame, identity-alias weg)
   en lokaal `invlpg`. De alias-metadata blijft daarbij staan.
3. Shootdown-IPI (vector 0xe0) naar iedere AP: `invlpg` van beide
   gepubliceerde adressen, daarna een ack van de generatie van deze ronde.
4. Pas na álle acks: beide aliassen vrijgeven, `free` en een volledige
   paging-audit.

Ontbreekt een ack, dan blijft het oude frame in quarantaine (eigendom en
alias behouden, nooit hergebruikt) en reset de BSP alle AP's met INIT.
Dat is fail-closed, want een frame dat een andere CPU nog kan lezen, mag
nooit naar een nieuwe eigenaar.

**Bewijs:** iedere CPU ziet daarna aantoonbaar het nieuwe frame. Een
controleprofiel zonder shootdown laat deterministisch zien dat alle AP's
anders het vrijgegeven frame lezen: de test meet dus een echt gevaar. Zie
`docs/verification/k06-smp-shootdown/`. **Open:** dit is één venster in
een bootstrapprofiel. Een algemene shootdown voor procesadresruimten (per
ASID/CR3 en per range), integratie met M3-unmaps, interrupts op de BSP en
de SMP-scheduler volgen nog; ADR-024 blijft tot dan ongewijzigd geldig.

**Status:** geaccepteerd in het opt-in `native-smp`-profiel; K06 blijft open.

**Alternatieven overwogen.**
- Alleen het venster ongeldig maken en de identity-alias lokaal laten
  vallen: verworpen na review, omdat een AP met een gedeelde CR3 die alias
  speculatief kan cachen.
- CR3 herladen of een volledige TLB-flush per AP: is ook correct, maar
  grover; `invlpg` per adres volstaat hier en blijft per vertaling
  controleerbaar.
- PCID/INVPCID: niet gebruikt; PCID staat uit in dit profiel.
- Het frame vrijgeven en de shootdown "lazy" achteraf doen: verworpen,
  want dat is precies het gevaar dat het controleprofiel aantoont.

**Primaire bronnen.** Intel SDM Vol. 3A: TLB-onderhoud ("Invalidation of
TLBs and Paging-Structure Caches", "Propagation of Paging-Structure Changes
to Multiple Processors", INVLPG als serializing instruction) en APIC (ICR
en TPR). Ook hier verschillen de hoofdstuknummers per editie.

## ADR-071 — Eigen TSS/IST per AP en begrensde, gerichte IPI's (K06 stap 2)

5 november 2026; aanvulling op ADR-070, nog steeds alleen in `native-smp`.
Iedere AP krijgt een eigen GDT-kopie met eigen TSS (RSP0, IST1 voor #DF,
IST2 voor NMI). De stacks daarvan staan in het kernelimage, met
ongemapte guards. Een busy TSS wordt nooit gedeeld. IA32_GS_BASE wijst
naar een per-CPU-anker; dat is de basis voor latere per-CPU-staat.

**IPI-contract.** Vector 0xf0 is een supervisor-interruptgate. De stub telt
de interrupt in het per-CPU-anker, geeft EOI aan de eigen xAPIC en keert
terug. Vector 0xff is de spurious-gate. Pas nadat alle LVT's gemaskeerd
zijn, TPR 0 is en SVR aan staat, wacht een AP met `sti; hlt`. De BSP
verstuurt alleen unicast-IPI's. Broadcast-shorthands zijn verboden, omdat
die CPU's in firmwarebezit zouden bereiken (bijvoorbeeld na een
capaciteitsgrens). Per ronde moet iedere AP precies één levering tonen.
Neemt een AP een gerichte interrupt niet aan, dan reset de BSP alle AP's
met INIT en werkt alleen verder. Een CPU die niet op een IPI reageert, kan
later nooit deelnemen aan TLB-shootdown. Een mislukte INIT stopt de boot.

**Bewijs.** NMI en #DF op een AP komen native op de eigen IST-stack van die
AP terecht; voorheen was dat een triple fault. Zie
`docs/verification/k06-smp-ipi/`. **Open:** de BSP blijft IF=0, en er is
nog geen TLB-shootdown, SMP-scheduler of lockvolgorde voor
kernelsubsystemen. AP's draaien dus nog steeds geen kernelsubsystemen.
ADR-024 blijft ongewijzigd geldig. Fysieke hardware is niet getest.

**Status:** geaccepteerd in het opt-in `native-smp`-profiel; K06 blijft open.

**Alternatieven overwogen.**
- Eén gedeelde TSS: onmogelijk, want een busy TSS mag maar door één CPU geladen zijn.
- Broadcast-IPI-shorthands: verworpen, omdat die ook CPU's in firmwarebezit
  bereiken (bijvoorbeeld na een capaciteitsgrens).
- Een AP die niet reageert alleen loskoppelen in plaats van alle AP's te
  resetten: verworpen, omdat een AP die geen IPI aanneemt later geen
  shootdown kan volgen.
- x2APIC/MSR-IPI: uitgesteld, het profiel gebruikt xAPIC-MMIO.

**Primaire bronnen.** Intel SDM Vol. 3A: taakbeheer (64-bit TSS, IST en
de busy-vlag), interrupt- en exceptionafhandeling (IST-stackwissel,
interruptgates) en APIC (ICR, delivery modes, LVT, TPR/PPR,
spurious-vector, EOI). De SDM-hoofdstuknummers verschillen per editie;
zoek op sectietitel.

## ADR-070 — Begrensde AP-opstart als eerste K06-stap

5 november 2026. De kernel start applicatieprocessoren (AP's) uitsluitend in
het opt-in M2-profiel `native-smp`, met IF=0. Dat profiel sluit bij
compilatie M3, framebuffer, native-rtc en graphics-g0 uit. Alle bestaande
één-CPU-invarianten (ADR-024 scratchslot, frame-allocator, paging, serieel,
IPC, scheduler) blijven onveranderd geldig, omdat AP's die code nooit
uitvoeren. De BSP is de enige eigenaar van frames, paging, PIT, ICR en
seriële uitvoer.

**Ontwerp.** De MADT is onbetrouwbare firmware-invoer. Alleen ingeschakelde,
unieke processors tellen mee (maximaal 32). Een kapotte, dubbele of te grote
MADT maakt alleen SMP ongeldig, niet de MCFG/DMAR-inventaris.
IA32_APIC_BASE moet xAPIC-modus, ingeschakeld en BSP aangeven en gelijk zijn
aan de MADT-basis. Alleen die exacte pagina krijgt een UC/NX-mapping, die
`audit` net als het framebuffervenster afzonderlijk controleert.
De SIPI-pagina is het eerste vrije ConventionalMemory-frame onder 0xA0000:
permanent Kernel-eigendom, één keer geschreven vóór de CR3-wissel en nooit in
de eigen paginatabellen gemapt. Zij bevat alleen 16-bitcode en een CR3-veld. De 32/64-bit-
stap, de trampoline-GDT en AP-stacks van 16 KiB met guardpagina's staan in het
identity-gemapte kernelimage. W^X verandert dus niet.

**Gelijktijdigheid.** Een AP kiest zijn context op hardware-APIC-ID en claimt
die met één `lock cmpxchg` STARTING→CLAIMED. Bij een time-out pakt de BSP
dezelfde CAS STARTING→ABANDONED en reset de CPU met INIT. Een late AP draait
dus nooit op een opgegeven stack. Iedere fout stopt verdere opstart, zonder
paniek: het systeem blijft met de reeds gestarte CPU's werken. Kan een
gefaalde AP niet met INIT worden gereset, dan stopt de boot als fout: er
draait nooit een onbeheerste CPU naast de kernel. Een CPU-exception op een
AP komt in de gedeelde M2-handlers en stopt de VM terminaal. Een MADT-
leesfout maakt eveneens alleen de MADT ongeldig. Een AP neemt
CR0/CR3/CR4/EFER/PAT van de BSP exact over en controleert die. Daarna voert
hij een begrensde atomic-plus-spinlocktest uit en parkeert met `cli; hlt`.

**Bewijsgrens.** QEMU q35/OVMF/TCG met 1–9 vCPU's plus negatieve gevallen;
zie `docs/verification/k06-smp-bringup/`. Dit is geen SMP-scheduling, geen
IPI/TLB-shootdown en geen per-CPU TSS/IST: een NMI of #DF op een AP stopt de
VM. Fysieke hardware is niet getest en K06 is niet geaccepteerd.
**Herzien** vóórdat een AP kernelsubsystemen, gebruikersprocessen of
interrupts uitvoert. Dan zijn per-CPU-staat, TSS/IST, lockvolgorde,
TLB-shootdown en de herziening van ADR-024 verplicht.

**Status:** geaccepteerd als opt-in bringupprofiel (`native-smp`), niet als
productprofiel; K06 blijft open.

**Alternatieven overwogen.**
- x2APIC-modus: uitgesteld. xAPIC-MMIO dekt het q35-profiel en firmware in
  x2APIC-modus valt gesloten terug naar één CPU.
- Een trampoline volledig in de lage pagina (ook 32/64-bit): verworpen. Die
  pagina zou een uitvoerbare mapping in de kerneltabellen nodig hebben,
  terwijl de huidige vorm haar nooit mapt (W^X ongewijzigd).
- De LAPIC-basis uit de MADT halen: verworpen. De basis komt uitsluitend uit
  IA32_APIC_BASE; de MADT-waarde wordt alleen vergeleken (onbetrouwbare
  firmware-invoer).
- Een broadcast INIT/SIPI ("all excluding self"): verworpen. Die bereikt
  ook CPU's buiten de MADT of buiten de capaciteit, en geeft geen
  per-CPU-time-out of quarantaine.

**Primaire bronnen.**
- Intel SDM Vol. 3A, §9.4 (MP-initialisatieprotocol, INIT-SIPI-SIPI en de
  bijbehorende wachttijden) en §11.6 (ICR, delivery modes, SIPI-vector als
  paginanummer onder 1 MiB).
- Intel SDM Vol. 3A, §11.4 (IA32_APIC_BASE: BSP-, EN- en EXTD-vlaggen).
- ACPI 6.5, §5.2.12 (MADT, structuurtypen en -lengtes, Enabled en Online
  Capable, waarbij Online Capable gereserveerd-nul is als Enabled gezet is)
  en §5.2.12.8 (Local APIC Address Override).
- Vindplaatsen: *Intel® 64 and IA-32 Architectures Software Developer's
  Manual*, Volume 3A (Intel Corporation; actuele editie,
  https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html),
  voor INIT-SIPI-SIPI, ICR en IA32_APIC_BASE (xAPIC/x2APIC-vlaggen),
  MTRR/PAT en XCR0. *Advanced Configuration and Power Interface (ACPI)
  Specification*, versie 6.5 (UEFI Forum, augustus 2022,
  https://uefi.org/specs/ACPI/6.5/), §5.2.12 voor de MADT. *UEFI
  Specification* 2.10 (UEFI Forum, https://uefi.org/specifications) voor
  het geheugenmodel bij overdracht. De identity-map onder 1 MiB is
  geverifieerd OVMF-gedrag en geen UEFI-garantie; daarom staat ze in de
  bewijsgrens als open punt.
- Lokaal OSDev-snapshot W003259 (MADT), uitsluitend als routekaart.
- De SDM-hoofdstuknummers volgen recente edities. In oudere edities heten
  dezelfde secties §8.4 (MP-initialisatie) en §10.4/§10.6 (APIC); zoek
  bij twijfel op de sectietitel.

**Uitgevoerd testbewijs:** `docs/verification/k06-smp-bringup/README.md`.
## ADR-SESSION-20261102 — herkenbare bediening en expliciete VM-sessiegrens

De huidige projectopdracht vereist herkenbare sessie-, zoek-, status- en
vensterbediening rond Materiaal. Een eigen productidentiteit verbiedt geen
bekende conventies. De nieuwe `native-session-route` blijft een openbare,
geïsoleerde VM-proef: persoonlijke credentials worden niet verzameld zolang
een echte auth-/sessie-/keyringdienst ontbreekt. Lock bedekt beeld en blokkeert
appinput, maar mag niet als sleutel- of opslagcapability-intrekking worden
gepresenteerd. FR-S02 en U02 blijven open; de bestaande meter blijft 41.

Shellbeleid blijft in de userspace-compositor. Een inputoffer wordt bevestigd
vóór een systeemactie. Appqueues en clipboardoffers worden bij afschermen
geannuleerd; apps kunnen de vertrouwde voorgrond niet tekenen. Desktop wordt
pas zichtbaar na beide complete SUR-frames. Diagnostische gereedheid wordt
als laatste gepubliceerd, na een voorafgaande 0, zodat timerpreëmptie geen
nieuwe pagina met oude gereedheid kan laten observeren.

SUR1 behoudt een volledig eerste frame en actor/epoch/bounds/COMMIT/deadline-
controles; latere sparse frames gebruiken strikt oplopende tegels en een
verplichte eindtegel. Rebind maakt de cache ongeldig. Een opaque renderpad en
oude/nieuwe paneelunion beperken werk, met exacte rastervergelijking tegen het
behouden referentiepad. Geen nieuwe kernel-ABI, crypto of apparaatrechten.

De geteste codebron is 703cc07; afzonderlijke route- en regressieprofielen
slagen. Onderzoek, bronhashes, screenshots, meetgrenzen en open werk staan in
[het uitvoeringsrapport](verification/session-quality-20261102/README.md).
VMware blijft een aparte platformproef volgens
[de concrete overdracht](implementation/VMWARE-WORKSTATION-TRIAL.md).
Herzien bij echte account-/sessierechten, resize/rebind, GPU-transport of
een gewijzigde trusted-inputgrens; een hostbeeld vervangt geen native bewijs.

<a id="adr-057--vreemde-opslag-nooit-als-leeg-volume-toelaten"></a>
## ADR-064 — Vreemde opslag nooit als leeg volume toelaten

3 november 2026; correctie op herstel van het experimentele NOSVOL02-volume.
De eerdere herstelroute controleerde bij twee lege tabelkopieën alleen de
tabelsectoren. Daardoor kon een niet-herkend volume met gegevens buiten die
regio als generatie 0 worden behandeld. Formatteer- of schrijfgedrag mag niet
uit een ontbrekende herkenning worden afgeleid.

Een onherkend volume is uitsluitend leeg wanneer de volledige huidige
volumegrens nul bevat. Voor een klein eerste schrijfresultaat legt de dienst
vóór inhouds- of tabelwrites een versie-/geometriegebonden `NOSFMT02`-marker
vast en flusht die. Daardoor blijven ongecommitte eerste uploads herstelbaar
als logische lege toestand, zonder dat vreemde niet-nuldata wordt geaccepteerd.
Een marker is formaatidentificatie en foutdetectie, geen authenticatie.
Bestaande geldige NOSVOL02-tabelkopieën blijven de primaire recoverybron.

De eerste markerwrite en -flush zijn een aparte initialisatiegrens. Bij een
crash vóór een geldige duurzame marker mag een onherkenbare niet-nulrest niet
als eigen opslag worden behandeld: herstel stopt met `Corrupt`, zonder writes
of stil wissen. Alleen volledig nul of een volledige geldige marker levert
hier generatie0. De before/after-invariant voor documenttransacties begint
na duurzame formaatidentificatie en blijft onverkort gelden; afzonderlijke
crashproeven controleren ieder byteprefix van de eerste marker en het eerste
datawritepunt. Expliciete herstel-/formatteerbediening blijft apart open.

De kernel blijft een generieke, begrensde blokbroker; de herkenning en
admissie blijven in `volume-core`. Er wordt geen bestandssysteemdriver,
importpad of automatische formatter toegevoegd. Een niet-gemarkeerd volume
met niet-nuldata geeft `Corrupt` en krijgt geen schrijf-/flushverzoek.
Een synthetische Btrfs-achtige fixture test de nul-tabel/niet-nul-data-grens;
zij is geen volledig Btrfs-bestandssysteem.

Herzien bij wijziging van de volumegeometrie of introductie van expliciete
formatteer-/importbediening. Terugzetten naar alleen tabelcontrole is niet
toegestaan. De marker kan bij een overgang naar een andere persistente
geometrie worden vervangen, mits de volledige media-admissie en crashproeven
behouden blijven. Native QEMU-resultaat en exacte bronbinding staan in
`docs/verification/foreign-volume-admission-20261103/README.md`.

**Ontwerpregel:** zonder geldige NOSVOL02-tabel of volledige `NOSFMT02`-marker
is alleen volledig nul over de huidige volumegrens `[0,580)` toelaatbaar.

**Implementatiestatus:** geïmplementeerd in `services/volume/src/table.rs`
(`choose` en `recover_with_state`) en het onafhankelijke hostorakel
`tools/volume_disk.py`.

**Testbewijs:** host-regressies en crashgrenzen zijn vastgelegd in
`docs/verification/pr148-slot1-review.json`,
`docs/verification/pr148-marker-crash-review.json` en
`docs/verification/pr148-volume-core-host-suite.json`.

**Onbekend / open:** native metadata-only fixtureacceptatie en native QEMU
volume-admissie op deze branch.

**Primaire bronmetadata:** Nieuw OS repository (`simonvanieperen/nieuw-os`),
branch `simonvanieperen-upgraded-disco`, commit
`c6b25873c915a98f8c0b2bb7400395e01ba44741`,
https://github.com/simonvanieperen/nieuw-os/tree/c6b25873c915a98f8c0b2bb7400395e01ba44741 .

## Nummerbehoud bij desktopintegratie — 2 november 2026

De desktopbranches gebruikten al bezette ADR-nummers. Bestaande IPC- en
appbesluiten behouden hun nummer; de samengevoegde UI-besluiten krijgen
unieke nummers. Oude volledige titelankers blijven als alias bestaan.
Historische bewijsbestanden behouden hun oorspronkelijke nummervermelding.

| Oud nummer / onderwerp | Uniek nummer |
|---|---|
| 053, native vensterbeleid | ADR-059 |
| 054, tijdelijke autoriteitsvergelijking | ADR-060 |
| 055, begrensde GUI-rust | ADR-061 |
| 056, eenmalig klembord | ADR-062 |
| 056, herkenbare VM-sessieroute | ADR-063 |
| 058, positieve volume-identiteit (historische titel) | ADR-064 |
| 057, vreemde opslag nooit als leeg volume toelaten | ADR-064 |

## Onderzoeksbesluit J-20261102 — herkenbare basis, expliciete installatiegrens

**Datum:** 2 november 2026. **Status:** ontwerp-/onderzoekskeuze, geen
gerealiseerde architectuur of native acceptatie. Na de
[volledige OS-journeyvergelijking](research/2026-11-02/journey/README.md)
is de voorkeursrichting één kerninterface met voorspelbare basisacties,
lokale zoekaliassen en optionele herkomstgerichte hulp. Geen vier shells,
herkomst als machtiging of vrije AI-uitvoering. De voorgestelde installer
toont één compleet wijzigingsplan vóór iedere opslagmutatie, ook Advanced;
onbekende schijf is niet leeg en een gewijzigd doel maakt autorisatie ongeldig.

**Reden:** gedocumenteerde overeenkomsten helpen kennis overdragen, terwijl
app-/vensterbetekenis, zoekbereik, installerfasen en vroege schrijfgrenzen
daadwerkelijk verschillen. Een exacte nieuw-os-volgorde of betere bruikbaarheid
is daarmee niet bewezen. **Gevolg:** afzonderlijke IN-/ON-/UX-criteria en
service-/veiligheidsgates vóór uitvoering; P2, kernelkeuze, privacycontract en
vaste scorecriteria blijven behouden. **Herzien:** op taakobservatie,
actuele bron-/hardwareverschillen of een aantoonbare technische veiligheidsgrens.
Details en begrensd vervolg: [PRODUCT-SPEC.md](implementation/PRODUCT-SPEC.md).

<a id="adr-056--herkenbare-sessieroute-met-expliciete-vm-bewijsgrens"></a>
## ADR-063 — herkenbare sessieroute met expliciete VM-bewijsgrens

2 november 2026. De sessieroute moet de bekende stappen uit macOS,
Windows en Linux volgen, binnen de eigen architectuur. Accountkeuze, credentialinvoer,
verificatie, sessieactivatie en desktop worden afzonderlijke toestanden.
Een app-lokale UI of een wachtwoordvergelijking in de renderer verleent geen
kernelrechten. Alleen een toekomstige geauthenticeerde auth-/sessiebroker mag
keys/grants vrijgeven en lock/revoke afdwingen. Annuleren, wisselen en lock
maken oude brokerantwoorden ongeldig; authenticatie én activatie moeten slagen.

Het nieuwe opt-in profiel `native-system-session-preview` gebruikt daarom
expliciet een openbare VM-gast met uitsluitend synthetische testopslag. Het
formulier voor een eigen gebruiker is een interactieproef; de standaardbroker
weigert ieder wachtwoord en iedere nog niet geïmplementeerde hardwaremethode.
Er komt geen gedeeld demowachtwoord en geen heimelijke auto-login voor echte
accounts. Bestaande native Materiaal-probes blijven hun eigen profiel behouden.
Taakbalk en programmasearch verlenen geen aanvullende opslag-/netwerkrechten.

Windows mag via een gepauzeerde probe zonder schijven/devices/netwerk WHPX
onderzoeken. Auto meldt TCG-terugval; strikt Whpx faalt bij ontbrekende backend.
Dit verandert geen hostinstellingen of beveiligingsmitigaties. CPU-versnelling
is geen GPU-passthrough of fysieke driveracceptatie. Herzien bij echte
accountpersistentie, auth-IPC, compositor-lock, USB/CTAP of gewijzigd rendertransport.
Bronnen, contract en open gates: [sessieonderzoek](implementation/SESSION-AND-DESKTOP.md)
en [hardwareonderzoek](implementation/VM-HARDWARE-AND-PERFORMANCE.md).


## OP-001 — Schrijvers, integratoroverdracht en Windows-afleverroute

30 oktober 2026; vastlegging van de operationele besluiten uit PR #75.
Dit beschrijft de toen geregistreerde rollen en overdrachten, geen bewezen
identiteit van huidige sessies. Actuele expliciete opdrachten van de projecteigenaar
gaan vóór routineafspraken tussen agents. De huidige opdracht om issues en pull
requests op te lossen en geschikt werk te squash-mergen geeft het
integratiemandaat; actuele broncontrole, repositoryregels en relevante gates
blijven gelden. Dit besluit kondigt geen nieuwe mergepauze of vrijgave af.

- **Queue-eigenaarschap:** de geregistreerde overdracht van manager
  `89f84098` aan `5d5f8513` maakt de laatste de enige schrijver van
  `docs/WORK-QUEUE.md`; centrale status en score bleven bij de manager.
  Gevolg: één register voor taak- en fileclaims, behoud van bestaande opdrachten
  en geparkeerd werk; stilte of een timeout maakt een claim niet vrij.
  Herzien bij een expliciete nieuwe opdracht of aantoonbaar overlappende
  schrijvers. De oorspronkelijke eigenaarverklaringen blijven historische
  rapportage. [OP1]
- **Integratoroverdracht e8→7f:** de queue registreert expliciete donorstop
  en vrijgave, geen lopende/verzonden mutatie, ontvangst van het volledige
  dossier met claims en WIP en een operatiecontrole door de ontvanger.
  Gevolg: één operationele integrator binnen de overgedragen scope; actuele
  head/base/gates opnieuw controleren bij iedere integratiemutatie en bestaand
  werk inhoudsbehoudend verenigen. Een accountnaam of verstreken tijd bewijst
  geen sessie-identiteit of overdracht. Herzien bij een nieuw mandaat van de projecteigenaar,
  een aantoonbare tweede schrijver of een gewijzigde inhoudelijke scope.
  De afzonderlijke incident-/pauzeregistratie in de bron wordt hier bewaard
  als historische status, niet opnieuw uitgevaardigd. [OP1] [OP2]
- **Windows-route 12→21:** de wederzijds bevestigde overdracht betreft
  uitsluitend `tools/package_windows_qemu.py`,
  `tools/windows-qemu/{Start-NieuwOS.ps1,Test-Launcher.ps1,README.md}`,
  `tools/tests/test_package_windows_qemu.py` en eigen Windowsbewijs.
  Gevolg: de bestaande route blijft leidend; geen tweede launcher/packager of
  impliciete kernel-, CI- of runtimeclaim. Route- en synthetisch testbewijs
  leveren geen nieuwe OS-bundel of gastacceptatie op. Een ontwikkelkandidaat
  moet exact aan bron, manifest, artifacts, beperkingen en slotafspraak zijn
  gebonden. Herzien bij expliciete wijziging van route-eigenaar/filegrens of
  kandidaat-/slotcontract. [OP3]

**Primaire bronnen:** Nieuw OS-project, 30 oktober 2026.

- **[OP1]** *Operationele werkqueue — mandaat, bron en continuïteit*,
  vaste PR #75-bron `c26f57b60d4eaffbb57c0eb5114fa00bd7254eb6`:
  [werkqueue](https://github.com/simonvanieperen/nieuw-os/blob/c26f57b60d4eaffbb57c0eb5114fa00bd7254eb6/docs/WORK-QUEUE.md).
- **[OP2]** *Actuele projectstatus — integratoroverdracht en uitgebreid
  overdrachtsverslag*, dezelfde vaste bron:
  [projectstatus](https://github.com/simonvanieperen/nieuw-os/blob/c26f57b60d4eaffbb57c0eb5114fa00bd7254eb6/docs/PROJECT-STATE.md).
- **[OP3]** S.W.J.B.M. van Ieperen, *Rol 21: Windows/QEMU-overdracht en
  routebewijs*, oorspronkelijke overdrachtbron
  `0915828fcd40492f873dc9098ee445306f1f0638`:
  [routeverslag](https://github.com/simonvanieperen/nieuw-os/blob/0915828fcd40492f873dc9098ee445306f1f0638/docs/verification/windows-delivery-21/README.md).

## ADR-058 — Herkenbare consumentenbediening op de eigen kernel

**Datum/status:** 30 oktober 2026; besluit van de projecteigenaar, vastgelegd
in PR #114. Leidend boven de eerdere eisen dat het hoofdscherm geen
bestaande desktop mag nabootsen en dat er geen dock of taakbalk komt
(PRODUCT-SPEC §1 en §5, het verbod op interactioneel nabouwen in
PRODUCT-SPEC *Aanvulling P3/P4* voor gangbare indeling en bediening,
DESIGN-AMBITION, PRODUCT-AND-SOVEREIGNTY §2, AGENTS.md *Productambitie*).

**Besluit:** consumenten hoeven Nieuw OS niet opnieuw te leren. Inrichting,
inloggen, vergrendelen, gebruikersbeheer, bureaublad, taakbalk, startmenu met
zoeken, vensterbediening, bestandsbeheer, instellingen en gangbare
sneltoetsen volgen de conventies die Windows, macOS en gangbare
Linux-desktops gemeen hebben. Waar die verschillen, kiest het ontwerp één
variant en legt die vast. Vernieuwing ("futuristisch") is een aanvulling op
het bekende pad, nooit een vervanging ervan.

**Gevolgen:**

- De eigen kernel, microkernel-/capabilityarchitectuur, eigen code en de
  Security Floor blijven ongewijzigd. Herkenbare bediening verleent geen
  bredere rechten: een app krijgt nog steeds alleen expliciet verleende
  capabilities, ook als de interface op bekende platforms lijkt.
- Herkenbaar inloggen verandert de beveiliging niet: elke gebruiker heeft
  eigen sleutelmateriaal (geen gedeelde volumesleutel), inloggegevens worden
  alleen in de sessiedienst ingevoerd via het vergrendelscherm of
  Ctrl+Alt+Delete, en de pincode beschermt nooit sleutels op schijf
  (PROFILES-AND-USERS §4).
- Materiaal blijft het gegevens-, geschiedenis- en rechtenmodel. Het wordt
  zichtbaar via de Bestanden-app, recente items in het startmenu en
  toestemmingsdialogen; het is niet langer het verplichte hoofdscherm.
  P2 blijft historisch productbewijs, geen te bouwen eindlay-out.
- Eigen visuele identiteit: geen gekopieerde iconen, afbeeldingen, merknamen
  of trade dress van andere platforms. Herkenbaar is de indeling en
  bediening, niet de huisstijl.
- De functies uit FR-U01 blijven verplicht, maar worden bereikt vanuit het
  bureaublad en startmenu. De tekst van de matrix in FULL-TEST-RELEASE wordt
  door de integrator in een vervolgwijziging aangepast; dit besluit schrapt
  geen rij en sluit geen criterium. Score blijft 34%.
- Uitwerking: [PROFILES-AND-USERS.md](implementation/PROFILES-AND-USERS.md)
  en [DESKTOP-SHELL.md](implementation/DESKTOP-SHELL.md).

**Herzien** na gebruikersproeven (U06) die aantonen dat een conventie voor de
doelgroep slechter werkt, of bij een nieuw expliciet besluit van de projecteigenaar.
Rollback: dit ADR intrekken en de eerdere teksten weer leidend maken.

# ADR-054 — Gedeeld immutable artifact-storecontract

**Datum/status/eigenaar:** 30 oktober 2026; contractvoorstel van
release-integratie, afgestemd door Apps en AI. Security-/integratiereview en
native uitvoering staan open. De eerste code-uitwerking mag uitsluitend een
zuivere contractcrate met negatieve hosttests zijn.

**Context:** Apppakketverificatie en het model-store-experiment hebben beide
een grote, immutable, bounded-read artifact nodig. Het bestaande
documentvolume is hiervoor aantoonbaar ongeschikt; documentquota worden niet
tot productstore-capaciteit verheven. Apps heeft een kandidaatdescriptor en
transactie-state-machine; AI gebruikt een afzonderlijke readerproef met 4 KiB
read-at. Message320 blijft een aparte wiregrens van maximaal 256 payloadbytes.
[AS-IPC1] [AS-IPC2]

**Besluit:** één provider-onafhankelijk contract met immutable
descriptor, opaque procesgebonden read-handle en read-at van maximaal 4096
bytes. Stale epoch, revocation, short/zero reads, EOF, out-of-range, overflow,
corruptie en backendprotocolfouten zijn expliciet. Capacity/quota zijn
providergerapporteerde `u64`-waarden met checked arithmetic. Writable
install/publish is een aparte transactie: volledige reservering bij `begin`,
exacte chunks, volledige digest-/lengtecontrole vóór atomische publicatie, en
expliciete `Unknown`/cancel/outcome-semantiek. Reservering is geen
duurzaamheidsgarantie. SHA-256 is contentintegriteitsmetadata, geen
publishervertrouwen of modelveiligheid. Geen dynamic launch volgt uit
opslagpublicatie.

**Afgewezen:** documentvolume hergebruiken (onvoldoende capaciteit en andere
semantiek); 4 KiB als Message320-payload behandelen (overschrijdt de
wirelimiet); AI- en Apps-opslag-ABI's onafhankelijk vastleggen (twee
incompatibele authorities); vaste model-/app-proefgrootte als productlimiet
nemen; caller-`verified`-boolean of gedeelde B02/B03-authority gebruiken;
`Unknown` als mislukking, succes of automatische retry verbergen.

**Gevolgen/herziening:** contractdocument
`implementation/ARTIFACT-STORE-CONTRACT.md` is leidend voor de gezamenlijke
interface na Apps-/AI-ownerafstemming. De descriptorlayout is nog geen gedeelde
geserialiseerde ABI. Hostcontracttests kunnen alleen reken-/state-machine-
invarianten toetsen. Een native backend vereist later eigen foutinjectie,
flush-/duurzaamheidsdefinitie, koude recovery, proces-/capabilitybinding en
afzonderlijke publisher- en modeltrustbesluiten. Herzie vóór wire-encoding,
kernel/SDK-integratie, persistent store, modeldownload of dynamic launch.

**Primaire bronnen voor de IPC-grens:**

- **[AS-IPC1]** Nieuw OS-project. (2026, 26 oktober). *Uitvoerbaar kernel- en beveiligingscontract M2–M4*, sectie Berichtformaat en transactie (bronversie `7f4f7bfe28fe9b2314bc4e4192e43942838d5dcb`). [Kernel- en beveiligingscontract](https://github.com/simonvanieperen/nieuw-os/blob/7f4f7bfe28fe9b2314bc4e4192e43942838d5dcb/docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md).
- **[AS-IPC2]** Nieuw OS-project. (z.d.). *SDK Message320-envelope: MESSAGE_SIZE, MAX_PAYLOAD en lengtevalidatie* (bronversie `7f4f7bfe28fe9b2314bc4e4192e43942838d5dcb`). [SDK-implementatie](https://github.com/simonvanieperen/nieuw-os/blob/7f4f7bfe28fe9b2314bc4e4192e43942838d5dcb/sdk/runtime/src/message.rs).

## ADR-055 — Canoniek app-pakket zonder impliciete rechten

Bij integratie hernummerd vanaf de branchlokale ADR-052 uit PR #73
(`dce3b05`); ADR-052 over de CRB-apertureproef behoudt zijn bestaande ID.

30 oktober 2026; A02-verificatiekern, boven op `origin/main` na PR #67.
Een pakket bevat in versie 1 één vast manifest en exact één ELF-image.
Metadata wordt ondertekend met beide al gebruikte algoritmen uit ADR-049
(Ed25519 en ML-DSA-65); de caller bindt verificatie aan ankers van de
geselecteerde bron. De signature bewijst herkomst onder die sleutel, niet
betrouwbaarheid. App-ID, ABI-interval, versie en image-digest zijn metadata;
scripts, dependencies en capabilities ontbreken. Installatie verleent dus
geen algemene rechten.

De packageverificatie is een zelfstandige `no_std`-crate en weigert
niet-canonieke padding, onbekend formaat, ABI-mismatch, onbekende ankers, een
van beide ongeldige handtekeningen, trailing pakketbytes, afwijkende
`image_length`-framing en digestfouten. `verify_package` inspecteert geen
ELF-structuur. Alleen `verify_executable_package` voert aanvullend
`kernel_core::user::parse` uit en weigert images die niet aan het bestaande
user-ELF-beleid voldoen.
Het pakketformaat hergebruikt geen kernel-update-target of updatebeleid.
Alternatief: de kernel-update-manifestcode uitbreiden zou applicatiesemantiek
vermengen met de boot-/slottrustgrens; de bestaande verifier accepteert
uitsluitend `TARGET_KERNEL`. Dit besluit introduceert geen appinstallatie of
nieuwe capabilities.

De executable-entrypoint roept aanvullend het bestaande
`kernel_core::user::parse`-beleid aan in plaats van een tweede user-ELF-policy
te introduceren. De resulterende laadbeschrijving is preflight, geen
frameallocatie, processtart of `LaunchImage`-autorisatie. De installer-/launch-
grens blijft een afzonderlijke open stap.

De 2 MiB imagegrens is een tijdelijke parser-/fixturegrens, geen productlimiet.
De ML-DSA-stackbehoefte moet vóór native integratie door de service-/stackquota
worden gedragen. Herzien vóór installerintegratie: bron- en sleutelrotatie,
ABI-policy, opslag-/update-transacties, appimage-loader en herstelgedrag.
Terugdraaien kan door de nieuwe crate/workspace-entry te verwijderen; B03 en
bestaande app-launch ABI blijven ongewijzigd. Geen A02-punt toegekend.

## ADR-056 — Bronintrekking blokkeert toekomstige pakketten

Bij integratie hernummerd vanaf de branchlokale ADR-053 uit PR #73
(`dce3b05`); ADR-053 over IPC-wachten behoudt zijn bestaande ID.

30 oktober 2026; bronbeheer-policy-core voor A02.
De verificatiekern ontvangt bij elke verificatie een expliciete `SourceId` en
verwachte bron-generatie; de appbundel kiest nooit zelf haar trust anchor.
Keyrotatie en intrekking zijn compare-and-swap-operaties op die generatie.
Intrekking blokkeert nieuwe pakketverificaties/updates maar verwijdert of
blokkeert reeds geïnstalleerde apps niet; dit gedrag is expliciet door de projecteigenaar vastgesteld.
Verwijdering van apps/data is een afzonderlijke gebruikershandeling.

`SourceRegistry` is momenteel een vaste in-memory kern van acht slots.
Gebruikersconsent, capability-afdwinging, duurzame bronstatus, crashherstel en
de koppeling aan een installer blijven vereist. De acht slots zijn geen
productlimiet en leveren geen A02-acceptatiebewijs op. Terugdraaien kan de
registry en haar tests verwijderen zonder het packageformaat of kernel-ABI te
wijzigen.

## ADR-B03K1 — Gedeelde, vastgepinde sleutelautoriteit per releasevenster

30 oktober 2026; B03-KEY-POLICY-1, basis `0ade8014`. Loader en update-dienst
gebruiken één `update-core::policy::TrustPolicy` met maximaal acht exacte
hybride sleutelparen, inclusieve releaseversievensters en onvoorwaardelijke
intrekking. Ongeldige of dubbele tabelentries geven geen autoriteit. De
ondertekende versie wordt pas na beide handtekeningcontroles tegen het venster
getoetst. Geen nieuw cryptoalgoritme, capability of dependency.

Alternatief kalenderverval is niet gekozen: de RTC levert geen vertrouwde
kalendertijd. Een manifest mag evenmin zijn eigen vertrouwde sleutels of
intrekkingsbeleid aanleveren. De QEMU-beleidstabel behoudt het bestaande
TEST-paar en pint apart next/revoked/expired-fixtures; deze openbare sleutels
zijn geen productieankers. Vooraf vastgepinde rotatie bewijst geen veilige
distributie/rollbackbescherming van nieuwe beleidsversies.

Herzieningscriteria: beoordeel dit besluit opnieuw zodra een productiebeleid
voor sleuteldistributie of rollbackbescherming van de beleidstabel wordt
vastgesteld, of een geautoriseerde vertrouwde kalendertijdbron beschikbaar
komt. Beoordeel dan opnieuw de ankerdistributie, policyversie-/intrekkingsgrens
en keuze tussen releaseversievensters en eventueel kalenderbeleid; deze
triggers wijzigen het huidige beleid niet automatisch.

Contract, negatieve gevallen, open productie-/tijdgrenzen en rollback:
`implementation/B03-SIGNED-UPDATES.md`, B03-KEY-POLICY-1. Terugname is een
gerichte revert naar het eerdere enkelvoudige TEST-verificatiepad, zonder
slot-/manifestformaatmigratie; terugname verwijdert wel lifecycleafdwinging.
Native resultaat en bronbinding staan afzonderlijk in PROJECT-STATE; B04,
B05, B06 en de aanvullende gates van #41 blijven open.

## ADR-B03K2 — TEST-ankers uitsluitend via expliciete testfeatures

3 november 2026; basis `d686bd54`. `update-core::test_policy` is niet langer
onderdeel van de standaardbibliotheekbuild en vereist de feature
`test-signing-anchors`. De native fixturedienst en de UEFI-fixtureloader
activeren dit alleen expliciet; het loaderprofiel heet `signed-boot-test`.
De kale feature `signed-boot` en een native update-servicebuild zonder
policykeuze falen tijdens compilatie in plaats van stilzwijgend de openbare
TEST-ankers te gebruiken.

Dit is een fail-closed buildgrens, geen productie-key lifecycle. Er zijn geen
productieankers, releaseconfiguratie, ondertekeningsceremonie of
intrekkingsdistributie toegevoegd. #41 blijft daarom open; geen nieuw rubricpunt
of releaseclaim. Terugdraaien herstelt de ongeconditioneerde
`test_policy`-module en de oude profielnaam, maar verwijdert deze
build-time-bescherming.

Herzieningscriteria: herzie deze testfeaturegrens zodra productiebeleid voor
sleuteldistributie, releaseconfiguratie of distributie van intrekkingen wordt
ingevoerd. TEST-ankers worden daarbij niet stilzwijgend productiebeleid.

## ADR-053 — Wachten op IPC behoudt bestaande rechten en krijgt ABI 1.10

Bij main-integratie hernummerd vanaf de branchlokale ADR-051; de
main-ADR-051 over sleutelstatus hieronder behoudt zijn bestaande ID.

30 oktober 2026; integratie `NATIVE-TEST-VERSION`, basis `ad2b981`.
De bestaande component `IPC-WAIT-1` wordt overgenomen voor de rode native
Materiaal-gebruikersketen. Zestien gekopieerde handle/interestparen met een
absolute PIT-deadline mogen een proces parkeren; readiness reserveert noch
consumeert iets. Iedere herbeoordeling controleert opnieuw de volledige
capability en PID. Geen userpointer, copyplan of objectreferentie blijft
over een contextwisseling bewaard. Nieuwe rechten worden niet toegekend.

De componentbranch gebruikte nog ABI 1.9, maar main heeft die inmiddels
aan de expliciet niet-vertrouwde RTC-klok toegewezen. Daarom gebruikt alleen
`ipc-wait` ABI 1.10; de RTC-syscall en oude profielen behouden hun gedrag.
De integratie behoudt ook de latere stackbereikcontrole bij userreturn.
Lege uitvoerbare runqueue plus wachtenden mag via de bestaande permanente
servicestack een `sti; hlt; cli` uitvoeren; alleen werkelijk getelde IRQ0's
verhogen de monotone klok. De volledige Engine-digestchecks blijven staan.
Consumenten mogen uitsluitend parkeren zonder lokaal uitvoerbaar werk.
De legacy-keyringlus wacht daarom niet wanneer RANDOM-ingress bestaat;
de volume-adapter wacht pas na zijn synchrone disk- en replywerk.
Desktop-init wacht alleen in rustige Running-fase op registry/controller;
de aparte testcontroller blijft zijn lokale kerneltrigger pollen.

Eerst wordt een aparte native tweeprocessenproef gebouwd, inclusief echte
queue-, intrekkings-, deadline-, pointer- en idlecontroles. Daarna volgt
de bestaande GUI-gate zonder verruimde deadline. Hosttests of een gebouwde
ELF bewijzen geen native voortgang, bruikbaarheid of volledige release.
Herzien vóór SMP of nieuwe IRQ-wakebronnen: het huidige invariant geldt
uitsluitend voor één CPU en geserialiseerde IF0-kernelmutaties.
Rollback is het uitschakelen van de opt-in feature en consumenten; ABI
1.10 blijft een gereserveerde interfaceversie.

## ADR-051 — Persistente sleutelstatus via een afzonderlijk administratief ingress

30 oktober 2026; uitvoering B02-PERSIST-1, basis `ad2b981`. De bestaande
specificatie is [B02 — geauthenticeerde records en exacte sleutelstatus](implementation/B02-SECURE-RECORDS.md),
gedateerd 29 oktober 2026 en aangevuld 30 oktober 2026 (FR-B02P). Het
[native profielverslag](verification/full-release-b02-review-followup/system-gate.json)
en de [eindmanifest](verification/full-release-b02-review-followup/final-manifest.json)
leggen één `native-system`-QEMU-normal/recover-run vast; dit is geen bewijs voor
koude rollbackankering, productiegebruik of volledige B02-acceptatie. De native
sender-control-test is niet uitgevoerd en blijft open.
B02-codec `export_persistent`/`restore_persistent` krijgt KEY1-opcodes 11/12
(`EXPORT_PERSISTENT`, `RESTORE_PERSISTENT`). Zij zijn uitsluitend bereikbaar via
een derde, apart gegrant kernel-endpoint van de Key Service dat alleen de
supervisor/herstelagent (in het FR-I01-profiel: init) bezit. Domein-endpoints
en het RANDOM-ingress weigeren deze opcodes vóór cryptografische verwerking;
het administratieve ingress weigert omgekeerd iedere sleutel-, record-,
recovery- en RANDOM-bewerking. Het externe 32-bytegeheim reist in de
aanvraag en wordt door de dienst niet bewaard; een tweede service-adapterpad
voor "admin" bestaat niet, alleen `wire::dispatch_persistent`.

Het geïntegreerde profiel legt de volgorde vast: de app verzegelt eerst in
RAM, meldt `KEYS_READY`, init exporteert en commit het 120-byteblob via de
crashconsistente volumetransactie in volumedomein 1, en pas na `PERSISTED`
commit de app het ciphertextrecord als gewoon document. Zo komt in dit
profiel geen ciphertext op schijf onder een niet-duurzaam vastgelegde
sleutelallocatie. Die volgorde wordt door het init/app-protocol geborgd, niet
door de volume- of Key Service: een niet-coöperatieve app kan nog steeds eerder
committen. Afdwinging hoort bij de nog open write-ahead in de Key Service.
Specificatie: [B02-SECURE-RECORDS.md](implementation/B02-SECURE-RECORDS.md);
native deelbewijs (QEMU, gedefinieerde profielen, geen fysieke hardware):
`docs/verification/full-release-b02-keystate-*` en
[full-release-b02-review-followup](verification/full-release-b02-review-followup/).
Bij koude herstart op de ongewijzigde read-only image herstelt init de exacte
toestand vóór enig sleutelwerk; een verkeerd geheim, een hogere generatievloer
en een sleutelopcode op het admin-ingress worden geweigerd en geteld.

Afgewezen alternatieven: (a) de Key Service zelf blok-/volumerechten geven —
vergroot de TCB van de sleuteldienst en vermengt opslagherstel met
sleutelbeheer; (b) persistente opcodes op de bestaande domein-endpoints —
elke app met een sleutelendpoint zou dan snapshotbevoegdheid krijgen.
Het herstelgeheim van init is in dit profiel een compile-time testconstante
in de init-ELF van de geteste wegwerpimage; wie die image heeft, kent het
geheim. Dat is uitsluitend acceptatiefixture, geen productpad.
Autoriteit: een SEND naar het admin-ingress is gelijkwaardig aan de hele
sleutelnamespace, want EXPORT levert de master versleuteld onder een door de
houder gekozen geheim; het mag uitsluitend aan de supervisor-/herstelprincipal
toebehoren en nooit worden gedelegeerd. Voor een koude, ongebruikte instantie
is het externe geheim bovendien de enige lineage-authenticatie: wie het geheim
kent én object 8 kan schrijven, kiest de master van de volgende start, niet
alleen een oude snapshot. Een productherstelagent heeft daarom een
onafhankelijk verankerde lineage-/generatiecontrole nodig.
Grenzen: de generatievloer in dit profiel is een gedemonstreerd mechanisme,
geen vertrouwd anker; koude rollbackankering, write-ahead per GENERATE/REVOKE
in de dienst zelf en een echte volume-identiteit (nu de profielconstante
`NOS-SYSTEM-VOL01`) blijven open. De SDK-eigen Message320-envelope-/stagingbytes
worden sinds de B02-IPC-hygiënecheck wel gewist; caller-owned request-arrays en
compiler-/registerkopieën vallen daar niet onder.
Dat de app geen SEND naar het admin-ingress bezit, controleert het manifest
sinds INT3-FU-116 op het manifestmoment over de volledige app-handletabel:
objectvergelijking per handle (`APP_ADMIN_HANDLE`) plus een exacte telling van
de bezette app-slots (`APP_HANDLE_COUNT`). De afzendercontrole van het
admin-ingress in de Key Service heeft in dit profiel nog geen uitgevoerd native
tegenvoorbeeld (geen ander proces bezit een SEND); dat is een open bewijs, geen
impliciete goedkeuring. B02 blijft daarom open; de score verandert niet. Rollback: verwijder de twee
opcodes, het admin-ingress en de init-/appstappen; bestaande KEY1-codes,
formaten en het B01-profiel zijn ongewijzigd.

## ADR-050 — Opt-in wall-clock syscall blijft zichtbaar niet-vertrouwd

30 oktober 2026; uitvoering NATIVE-CLOCK-1, boven op RTC-NATIVE-1.
`native-system-clock` voegt ABI 1.9 syscall `0x73` toe voor een directe,
read-only UTC-snapshot uit CMOS. De gebruikers-SDK retourneert een expliciet
`UnauthenticatedRtc`-type. De klok is openbare procesmetadata; er worden geen
CMOS-poorten, capabilities, pointers, opslag- of netwerkrechten aan apps
gegeven. Ongeldige niet-nulargumenten geven `-22`, mislukte of onbruikbare
hardware geeft `-5`, zonder fallback of verzonnen tijd.

Deze scope kiest voor een werkende, testbare systeemfunctie in het bestaande
single-CPU IF0-profiel, niet voor een vertrouwde productie-TimeProvider. De
adapter verifieert de native RTC-invarianten en indexherstel per aanroep.
Unix-tijd en boot-monotone PIT-ticks blijven gescheiden. Geen
certificaatvalidatie, persistente antirollback, synchronisatie, slaap-/wekcorrectie,
fysiek hardwarebewijs of nieuwe voortgangspunten. De echte ring-3-app en
onafhankelijke QEMU-basistijd-orakel zijn de acceptatiepoort.

Herzien vóór SMP, andere firmware/hardware of TLS-gebruik: dan zijn
geserialiseerd apparaatbezit, geautoriseerde TimeProvider, versheid,
onzekerheidsmarge en rollbackanker nodig. Rollback: verwijder uitsluitend de
opt-in feature, syscall, SDK-bindings en acceptatie; ABI 1.9 blijft als
documenteerde compatibiliteitsrevisie, zonder invloed op oude profielen.

## ADR-057 — Exclusieve native CMOS-bootstrap, geen vertrouwde tijd

Bij integratie hernummerd vanaf het dubbel gebruikte ADR-049 in main
(`9a6703d`). ADR-049 over hybride kernelupdates behoudt zijn bestaande ID;
historische verwijzingen naar de CMOS-bootstrap met ADR-049 horen bij ADR-057.

30 oktober 2026; implementatie RTC-NATIVE-1, basis `5c334f9`.
De bestaande dependencyvrije RTC-codec wordt alleen voor een opt-in
`native-rtc`-diagnostiek aan de eigen kernel gekoppeld. Na eigen IDT/paging,
op een CPU met IF=0, bezit de proef uitsluitend CMOS-index/data 0x70/0x71.
Een expliciete D/NMI-Enabled-startselectie vervangt geen firmware-shadow;
de readprocedure herstelt deze eigen selectie. Geen RTC-datawrites,
register-C-ACK, app-I/O, nieuwe syscall of productiefeature-implicatie.
Ring-3/framebuffercombinaties worden geweigerd. De kleine privileged codec
is uitsluitend hardwareacceptatie; een productie-TimeProvider blijft een
geisoleerde dienst met afzonderlijk minimaal broker-/rechtencontract.

Alternatief: direct een wandkloksyscall of TLS-klok publiceren zou
niet-geauthenticeerde tijd tot een impliciet vertrouwensanker maken.
De uitvoer houdt daarom `UnauthenticatedRtc`, callerfloor en vluchtige
terugloopcontrole expliciet. Vijf onafhankelijke kalender-oracles plus echte
budget-/floorweigering en device-indexherstel vormen de native poort; de
bestaande hosttests blijven vereist. Exacte uitvoering: PROJECT-STATE en
RTC-WALLTIME. Geen fysiek-, persistent antirollback-, TLS- of productbewijs.
Primair emulatorcontract: QEMU `v10.2.1`, `hw/rtc/mc146818rtc.c`; aanvullend
OSDev CMOS W001717 (revisie 30263). Geen emulatorcode overgenomen.
Rollback: opt-in feature/adapter/runner revert; bestaande M2 en alle
productiediensten blijven ongewijzigd. Herzien bij IRQ/SMP of productgebruik.

## ADR-031 — Native I/O-onzekerheid zonder observerpanic

**Datum/status/eigenaar:** 28 oktober 2026; geïmplementeerd en geaccepteerd op
bron 88e6c04. Native builder A01, integratie root, AI-reviews R08/R09 en A04.

**Context en besluit:** de dienst kende Unknown al, maar de M6c-observer eiste
bij een voorbereide mutatie altijd Ok. Bind de eerste onzekere reply aan een
werkelijk aan de dienst teruggegeven terminale muterende syscallfout; Pending
is geen fout. Latch de I/O-tellingen, commit geen kandidaat, tel geen ACK en
controleer dat beide appdomeinen daarna geen nieuwe I/O veroorzaken. Validatie
en autorisatie blijven vóór de gesloten diensttoestand. Alleen volledige koude
recovery bepaalt de werkelijk opgeslagen versie.

**Alternatieven en gevolgen:** de succesasserties verwijderen zou fouten maskeren;
automatische write-replay kan een onzekere mutatie verdubbelen. Het gekozen pad
houdt normale WRITE+FLUSH-voorwaarden streng en verandert geen capability,
syscall, wire-/journalformaat, dienstimplementatie of externe dependency.
Testinjecties lopen via bestaande driversnapshot-, deadline- en resetpaden.
Geen automatische dienstherstart; historische M6b-observer apart behouden.

**Bewijs en herziening:** de volledige 35-groepen/31-VM-gate en M6b-regressie
(18 groepen/14 VM) slagen, met beide captures en afzonderlijke bewijsaudits.
30 Rust/48 Python-tests voor M6c, 22 Rust/30 Python-tests voor M6b; alle exit 0.
Exacte scope en bron: `verification/M6D-ACCEPTANCE.md`; S07 levert 13% → 14%.
Contract en primaire VIRTIO-bron:
`implementation/M6D-IO-UNCERTAINTY.md`. Geen fysieke devicefout, stroomuitval,
menselijke audit of formeel bewijs. Herzie bij algemene servicesupervisie,
parallelle mutaties, nieuwe I/O-foutmodellen of andere hardware.

## ADR-032 — Vaste 100-puntenroadmap met verplichte herbeoordeling

**Datum/status/eigenaar:** 28 oktober 2026; aanvaard en uitgevoerd op expliciete
opdracht van de projecteigenaar. De hoofdagent bezit de gedeelde voortgangsbestanden.

**Besluit:** `docs/progress.json` bevat vaste, binaire acceptatiecriteria met
gewichten die samen 100 vormen. De generator maakt de README, criteriatabel
en lokale SVG. 0% is nog niet gerealiseerd; 100% is een volwaardig zelfstandig
dagelijks OS op aantoonbaar ondersteunde hardware. De eigen kernel blijft
verplicht. De beginscore van de M1–M6c-bewijsstand is 13%.

**Alternatieven/gevolgen:** commits, regels code, testaantallen of M-nummers
meten geen productvoltooiing. Daarom leveren hostprototypes of alleen plannen
geen native punten op. Iedere opdracht actualiseert beoordeling, bron en datum;
een score mag gelijk blijven of dalen. Gewichten zijn een expliciete
planningskeuze, geen wetenschappelijke meting of resterende-tijdsraming.

**Bewijs en herziening:** `tools/progress.py --check` controleert som, unieke
criteria, bestaande bewijsverwijzingen en gelijkheid van gegenereerde weergaven.
Het beoordeelt de bewijsinhoud niet en voert geen native tests uit. Wijzigingen
aan eindscope/gewichten vereisen rubricversie, reden en vergelijking met de oude
score. Zie `PROGRESS.md`; nooit de lat verlagen om een hoger getal te krijgen.

## ADR-030 — Native labels en een catalogus per appdomein

**Datum/status:** 28 oktober 2026; geïmplementeerd en native eindgate geslaagd
op broncommit bc4cd104, basis bb22fdb. Exact contract: implementation/M6C-NAMED-CATALOG.md.
Op deze historische M6c-uitvoeringsbron heeft het native `Unknown`-foutpad een
bekende, ongerepareerde observerpanic, ook met `milestone-m6c`. De afzonderlijke
reparatiebron en haar afzonderlijke acceptatie staan bovenaan in ADR-031.

**Context en besluit:** afgeronde M6b heeft anonieme objectslots. Materiaal heeft
benoemde objecten en list/open nodig. Voeg een aparte dependencyvrije catalog-core
met expliciete IPC v4/journal v2 toe. Namen zijn UTF-8-labels, geen autoriteit of
pad; object-ID en endpoint/caller blijven de toegangsgrens. LIST retourneert één
begrensde snapshot van uitsluitend eigen live objecten met eigen revisiesom.
PUT/RENAME zijn één record met naam/inhoud onder één CAS-revisie. RENAME behoudt
inhoud; WRITE- en FLUSH-completion gaan vóór ACK. Het servicecontract sluit bij
`Unknown` ook LIST af; dit foutpad is alleen via host-ABI getest.

**Alternatieven/gevolgen:** stil hergebruik van v3/v1-padding zou oude readers
misleiden; een losse naamtabel vereist meerdere atomische writes. Automatische
migratie is niet nodig voor deze synthetische proef: v1 weigert vóór READY en de
M6b-reader blijft behouden. Duplicaatlabels zijn toegestaan; geen directoryboom,
normalisatie, encryptie of fysieke secure erase toegevoegd. Kleinere grenzen zijn
proefbudgetten, geen vaste consumentenproductkeuze.

**Bewijs en herziening:** de finale M6c-gate/capture slaagt met 19 groepen en
15 VM-invocaties: 14 M6c plus één gewone M1. M6b-regressie slaagt met 18 groepen
en 14 VM-invocaties op dezelfde fingerprint. De 30 coretests, 33 harnastests en
15 host-ABI-adapterproeven behouden hun eigen scope. De native observer in
`kernel/src/m2/m3/m6c.rs` kan bij WRITE-/FLUSH-fouten of time-outs panikeren
vóór de client `Unknown` ontvangt; gecontroleerde native foutafhandeling was
op die bron niet gerealiseerd. Een afzonderlijke AI-audit
controleert alle 141 bronbestanden, 130 geregistreerde artifacthashes en acht
volumes, inclusief twee externe rename-kills en zes koude recoveries. Details:
`verification/M6C-ACCEPTANCE.md`; geen menselijke audit of formeel bewijs.
De nieuwe kernelmodule bevat een expliciete fixtureobserver plus de bestaande blokbroker; het
catalogusbeleid draait in ring 3. Herstel van het bekende `Unknown`-defect
vereist een afzonderlijke native reparatie met WRITE-/FLUSH-fout- en
time-outinjectie, controle van het werkelijk ontvangen clientantwoord en
nieuwe brongebonden native verificatie. Herzie formaat/migratie vóór echt gebruikersdata,
meer slots, grotere documenten of compositorintegratie. Terugdraaien kan via de
bestaande M6b-build en zijn eigen volumes; geen bestaande image overschrijven.

**Bronnen:** huidige M6B-OBJECT-STORAGE.md/PROJECT-STATE.md op bb22fdb; Rust
bare-metal targetdocumentatie in het M6c-contract. Dit is een projectbesluit.

## ADR-028 — Bewijs per eigenschap en standaard hybride NAS-transport

**Datum/status:** 27 oktober 2026; ontwerp aanvaard en N02-hostproef getest vanaf `ece94a5`. **Context:** de gebruikersblauwdruk vraagt formele verificatie, kleine TCB, capabilities, geheugenbescherming en PQC. De richting past bij het project; de absolute productclaims en het eigen KDF-/tunnelvoorstel zijn niet onderbouwd.

**Besluit:** eigen Rust-kern behouden, formele eigenschappen met expliciete aannames en implementatiebinding uitwerken, CHERI als aparte platformproef. Asymmetrische/PQC-code blijft buiten `kernel/`. Voor de eerste uitvoerbare NAS-transportproef TLS 1.3 met uitsluitend de gestandaardiseerde groep X25519MLKEM768 (RFC 10024), wederzijdse certificaatcontrole/pinning en bestaande peergebonden rechten. Geen eigen combiner/recordcrypto. De optionele-PSK-framing uit het aangeleverde fragment is ambigu. Een klassieke-only peer krijgt geen fallback in dit proefprofiel.

**Alternatieven/gevolgen:** onmiddellijk een ongeteste native crate toevoegen past niet bij de huidige directe-rustc-build, vaste ELF-limieten, kleine IPC, ontbrekende entropycapability en uitgeschakelde SIMD. Een hele kernel vervangen door seL4 is geen gegeven opdracht. De begrensde hostproef geeft werkelijk protocolbewijs zonder een native port te claimen. De proef gebruikt klassieke Ed25519-authenticatie; geen volledige postkwantum-authenticatie-/bootketenclaim. LAN/WAN-labels in loopbacktests zijn expliciete harnasaannames.

**Invariant:** toestemming vóór verbinden; echte peer-authenticatie vóór data; actuele bevoegdheid bij iedere applicatiewrite/response; close/error trekt transportfeit in. Fact bindt aan app, peerpin-generatie en voor internet de intrekkingsgeneratie; oude geauthenticeerde verbindingen herleven niet na hernieuwde toestemming. Intrekking werkt bij de volgende gecontroleerde handeling, geen proactieve idle-socketpush. Client-only documentkeys en het bestaande beheerderverwijderrecht blijven gelden. Een geldig ondertekend pakket verleent op zichzelf geen capabilities. Eenmalig verleende beperkte autoriteit mag automatisch gebruikt worden zonder herhaalde prompts.

**Bewijs/status:** geïntegreerd 124/124 tests (85 service, 39 UI), beide demo's exit 0, 20 syntaxchecks en diffcontrole geslaagd. Echte negatieve TLS-handshakes, geauthenticeerde kwaadaardige serverantwoorden, resourcegrenzen, intrekking en regrant getest. Twee AI-reviews en gerichte onafhankelijke hercontrole van de generatiecorrectie. Commando's/bronhashes/resultaten in `verification/pqc-transport/`. Het eerste 16-policytestcheckpoint is historisch bewaard en niet bij de 124 opgeteld. Geen formeel bewijs of menselijke audit.

**Bronnen:** RFC 10024; seL4 verified configurations/Proofcraft nieuws 2026; CHERI FAQ/CHERI-D; NIST FIPS 203/204/205. Precieze URLs, correcties en native taken staan in `implementation/VERIFIABLE-SECURITY-AND-PQC.md`.

**Herziening/rollback:** herzie bij incompatibele standaarden, library-/bewijsproblemen, native port of nieuwe productinstructie. N02 is een afzonderlijke werkbranch boven PR #7; verwijderen ervan vereist geen schijf-, sleutel- of firmwaremigratie. De nieuwe proef opent uitsluitend loopback. Bestaande N01/M6a-code en bewijsarchieven behouden.

## ADR-027 — Internet en versleutelde persoonlijke NAS-ruimte

**Datum/status:** 27 oktober 2026; aanvaard op expliciete opdracht van de projecteigenaar, met geteste hostproef N01 en browsermock. Native integratie staat open. **Eigenaar:** hoofdagent voor contract/integratie; afzonderlijke service- en productbuilders.

**Context:** internettoegang, een NAS-rol binnen hetzelfde OS en het aanbieden van opslagruimte aan vrienden en familie behoren tot de producteisen. De host moet zijn ruimte kunnen terugnemen zonder toestemming van de gast, terwijl de inhoud voor hem versleuteld blijft. Dit herziet ADR-006 en de eerdere beperking tot eigen NAS/eigen LAN.

**Alternatieven:** alleen eigen LAN handhaven voldoet niet aan de nieuwe opdracht; een leverancierscloud of beheerderssleutel voor gastinhoud schendt de privacygrens; gastconsent als vereiste voor verwijderen ontneemt de hardware-eigenaar zijn beheerrecht.

**Besluit/invariant:** client-only inhoudssleutels; versleuteling van documentnaam en inhoud; namespace/object/revisie geauthenticeerd. NAS-beheer beheert quota, intrekking en verwijdering maar krijgt geen decryptiesleutel. Revoke/evict invalideert oude sessies en lopende uploads; tombstones verhinderen herleving. Internet is een afzonderlijk apprecht; NAS-internet is expliciet peergebonden, lokale AI blijft netwerkloos. Lokale NAS-beheerrol staat los van een internet-datakoppeling; extern beheer wordt niet stilzwijgend toegevoegd. Een routeplanner kiest automatisch geverifieerd LAN of eerder toegestaan direct internet naar dezelfde peer, zonder herhaalde consentvragen of nieuwe autoriteit. Privacy en gebruiksgemak zijn samen vereist; tijdelijke hostproefcaps zijn geen productkeuze.

**Uitvoering:** dependencyvrije Node-hostproef in services/nas/, zonder sockets of echte NAS. Eigen synthetische ciphertextstore met één schrijver en atomische snapshots. Browserflows Mijn NAS / Ruimte ontvangen / Ruimte aanbieden zijn mocks. Deze ontwikkelmiddelen worden geen verplichte runtime van het eigen OS.

**Testbewijs:** gezamenlijke suite 89/89 (50 service, 39 UI) en eindige demo exit0; bronhashes/commando's in verification/nas/report.json. Echte childproceskills na bevestigde opslagmutaties, geen fysieke stroomuitvaltest. Vier concrete reviewbevindingen zijn hersteld: noncebudget bij staging, tijdelijke ciphertextsnapshots bij wissen, race bij writerlockherstel en ongewenst beheerrecht via internet.

**Grenzen/gevolgen:** hostidentiteit en transportfacts zijn synthetisch; geen TLS-/native netwerkbewijs. Herstelde inhoudssleutel kan alleen lezen, niet veilig blijven schrijven op meerdere apparaten. Browserinternet is in N01 appgebonden; bestemmingsgrants voor gewone apps/downloaders volgen nog. Geen volledige snapshot-rollbackbescherming, veilige fysieke erase of plug-and-play-NAT-claim. Native clientdistributie/key lifecycle, netwerkcapabilities, beheeropslag, onboarding en menselijke audit blijven afzonderlijke integratievoorwaarden.

**Bronnen:** Node.js v24.22.1 Crypto (https://nodejs.org/docs/latest-v24.x/api/crypto.html), RFC 8446 (https://www.rfc-editor.org/rfc/rfc8446.html) en NIST SP 800-207 (https://doi.org/10.6028/NIST.SP.800-207), geraadpleegd 27 oktober 2026. Functionele keuze en adminverwijderrecht zijn projectbesluiten. Exact contract: implementation/NAS-SHARING-AND-INTERNET.md en LOCAL-PRIVACY-CONTRACT.md revisie 3.

**Migratie/rollback/herziening:** geen bestaande echte NAS, router, kernel-ABI of M6a-image wordt gewijzigd. Bewaar oudere M1–M6a-resultaten. N01 kan als afzonderlijke featurecommit worden teruggedraaid; geen destructieve reset van gebruikerswerk. Herzie bij bewijs van key-/nonceproblemen, multi-writerintegratie, andere transport-/opslaglaag of een nieuwe instructie van de projecteigenaar. Oude LAN-only teksten zijn op de actieve leesroutes bijgewerkt.
De oorspronkelijke revisie-5-principes hieronder zijn behouden als projectbesluiten. Uitvoeringsbesluiten vanaf ADR-019 beschrijven de gebouwde richting; actuele M1–M5-testuitkomsten en grenzen staan in PROJECT-STATE.md.

| ID | Besluit | Reden | Herzien als |
|---|---|---|---|
| ADR-001 | Eigen kernel; geen bestaande kernel als basis | Expliciet besluit van de projecteigenaar | De projectscope bewust door de projecteigenaar wordt gewijzigd |
| ADR-002 | Rust no_std + minimaal assembly | Kleine expliciete onveilige grens | Target/tooling dit aantoonbaar verhindert |
| ADR-003 | x86_64 QEMU q35, UEFI, software-emulatie baseline | Controleerbare eerste testomgeving | Praktische toolchainbeperking wordt aangetoond |
| ADR-004 | Kleine capability-gebaseerde microkernel als richting | Autoriteit expliciet en TCB klein houden | Prototypes aantoonbare problemen met complexiteit/latency tonen |
| ADR-005 | Standaarden en gecontroleerde bibliotheken toestaan | Zelf crypto/alle tools bouwen vergroot risico en scope | Herkomst/licentie/onderhoud onacceptabel is |
| ADR-006, herzien door ADR-027 | Lokaal account, geen OS-telemetrie, lokale data of E2EE naar gekoppelde eigen/gedeelde persoonlijke NAS; internet per peer expliciet | Expliciete uitbreiding van de projectopdracht, 27 oktober 2026 | Alleen na een expliciete opdrachtwijziging door de projecteigenaar |
| ADR-007 | PQC-voorbereiding vanaf ontwerp; integratie pas met echte use case | Protocol- en sleutelmigratie voorkomen | Relevante standaarden/implementaties wijzigen |
| ADR-008, herzien | Product-/interactieonderzoek parallel vanaf M0; echte systeemintegratie volgt bewezen kernfuncties | Originaliteit en hoogwaardige UX moeten het systeem mede sturen | Bijstelling op experimenten, geen stille reductie tot standaarddesktop |
| ADR-009 | Geen kernelbrede claims over formeel bewijs | Eigen code erft geen seL4-bewijs | Afgebakend daadwerkelijk geverifieerd resultaat bestaat |
| ADR-010 | AI-inferentie uitsluitend op apparaat; geen netwerkrecht of cloudfallback | Expliciete correctie van de projectopdracht | Alleen na een expliciete opdrachtwijziging door de projecteigenaar |
| ADR-011 | Installeerbare derdenapps met gescheiden bestands-, NAS- en netwerkrechten | Toegankelijk ecosysteem zonder impliciet vertrouwen | Acceptatietests tonen ontbrekende grens |
| ADR-012 | Quantum als derde onderzoeksspoor vanaf M0: PQC, lokale QPU-uitvoering en hybride systeemport | Oorspronkelijke ambitie expliciet behouden | Bijstelling alleen met onderbouwd onderzoek; privacycontract blijft gelden |

## Gekozen uitvoeringsbesluiten — 26 oktober 2026

| ID | Besluit | Bewijsgrens / herziening |
|---|---|---|
| ADR-013 | Eigen UEFI-loader met r-efi 7.1.0; Rust 1.90.0; x86_64/QEMU TCG | Versies onderzocht; combinatie nog niet gebouwd. Exact bootcontract in BOOT-AND-TOOLCHAIN.md |
| ADR-014 | Eerste kernel: vaste ELF-layout en gedeelde BootInfo; daarna begrensde processen, IPC en capabilities | Details en negatieve tests in KERNEL-AND-SECURITY-CONTRACT.md; geen isolatiebewijs vóór uitvoering |
| ADR-015 | Materiaal als eerste productrichting; P2 lokale HTML/CSS/JS-proef | Vergelijking uitgevoerd, geen gebruikersproef. Herzie op concrete gebruiksobservaties |
| ADR-016 | Q1 safe Rust statevector-core met qsim-v0, maximaal 12 qubits, hostadapter apart | Analytische fixtures gecontroleerd; geen Rust-/QPU-/OS-portbewijs |
| ADR-017 | Later ML-KEM-768 en ML-DSA-65 via beoordeelde implementaties/protocolbinding | Geen eigen cryptoprimitives, geen volledige bootketenclaim |
| ADR-018 | EU-01 t/m EU-08 als technische acceptatie-eisen; toepasselijkheid wetgeving per release | EUROPE-AND-RELEASE.md bevat conditionele matrix, geen conformiteitsclaim |

## Template voor nieuw besluit
ID, datum, status, context, alternatieven, besluit, onderbouwing/bronnen, gevolgen, security-invariant, testbewijs, migratie/rollback en eigenaar. Schrijf wat de keuze zou weerleggen; vermijd besluiten zonder toetsbare aannames.

## Uitvoeringsbesluiten — 27 oktober 2026

**ADR-019 — Windows-ontwikkelpakket en alleen-lezen virtio-ESP.** Rust1.90.0/MSVC gebruikt de bestaande Visual Studio2026/SDK. QEMU11.1.0 en het bijpassende CODE/VARS-paar komen uit dezelfde vastgelegde Windows-package; hashes staan in docs/verification/toolchain-provenance.json. Dit is de toegestane ontwikkelvariant, niet de byte-identieke referentiebuild. De oorspronkelijke IDE-drive gaf vóór loaderuitvoering `Block node is read-only`; de ESP gebruikt daarom `if=virtio,readonly=on`. Iedere run krijgt een eigen ESP-/VARS-kopie, zonder netwerk of hostdisk. Werkelijke cold-boots en foutproeven staan in docs/verification/boot-runs.json. Herzie bij andere firmware/platformen; verwijdering van de afzonderlijke toolmap herstelt de host zonder globale PATH-wijziging.

**ADR-020 — Vaste kernelspan32–48MiB en BootInfo1.1.** In de echte OVMF-kaart overlapt het oorspronkelijke interval2–18MiB ACPI NVS en BootServicesData. AllocatePages wees de reservering af met EFI_NOT_FOUND; diagnose in lokaal build/run/smoke-fe68c261dd/result.json. Het profiel gebruikt nu `[0x02000000,0x03000000)`, dezelfde16MiB. Loader, parser, linker en fixtures zijn samen gewijzigd; de header is nog128bytes met dezelfde offsets, maar vereist expliciet ABI1.1. ABI1.0 wordt afgewezen. Er wordt geen bezet firmwaregeheugen hergebruikt. De volledige acceptatiereeks met deze waarden slaagt. Terugdraaien vereist gezamenlijk aanpassen van beide binaries/validator/linker en een aantoonbaar passende firmwarekaart; nooit alleen een adresfallback invoeren.

**ADR-021 — Statische GOT in .data.** De compiler produceert vijf statisch opgeloste pointers. Die staan expliciet in de bestaande RW-.data-outputsectie; er komt geen runtime-relocatieverwerker of extra segment. De buildinspectie blijft dynamische metadata, relocaties en onbekende allocatable secties weigeren. De drie ELF-laadsegmenten en hashes staan in docs/verification/boot-manifest.json. M1 controleert segmentintentie; echte W^X-afdwinging blijft M2.

**ADR-022 — Voortgang per stap naar GitHub.** Het project moet bij verbruikslimieten volledig te hervatten zijn. Broncode, lockfiles, compacte bewijsrapporten en open punten gaan daarom na elke afgebakende stap en vóór lange controles naar de privéwerkbranch. Een checkpoint mag onvoltooide controles bevatten mits expliciet gemarkeerd. Codespaces is niet vereist voor deze bewaarroutine; een Codespace is niet gestart. Main blijft de startspecificatie totdat de concept-PR is beoordeeld.

**ADR-023 — Alleen WB-capabele normale ConventionalMemory, cachebits gecombineerd.** De eerste M2-run faalde vóór KS-02 doordat OVMF `0x0f` meldt en de allocator uitsluitend exact WB accepteerde. UEFI-attributen beschrijven mogelijkheden; UC/WC/WT/WB mogen samen voorkomen. De allocator vereist WB, staat uitsluitend deze vier cachebits toe en weigert overige/bijzondere attributen. Bron: TianoCore/EDK II, `edk2-stable202508`, [UefiSpec.h, EFI_MEMORY_DESCRIPTOR en cachebits](https://github.com/tianocore/edk2/blob/edk2-stable202508/MdePkg/Include/Uefi/UefiSpec.h), gecontroleerd 27 oktober 2026. De huidige package is daarmee niet als exact die firmwareversie geïdentificeerd. Regressie test zowel `0x0f` als geweigerde special/unknown-bits; echte M2-poort slaagt. Andere platformprofielen vereisen opnieuw beoordeling van geheugenattributen; geen stil reclaim van Loader/BootServices/runtime/ACPI-geheugen.

**ADR-024 — Statische2048-mappingarena en één tijdelijk scratchslot.** Iedere procescontext heeft de contractcapaciteit2048 mappings; grote metadata staat statisch buiten de16KiB-kernelstack. PID is monotoneu64; interne ASID is afzonderlijk begrensd. Allocatorzeroing/tabeltoegang gebruikt een tijdelijk supervisor/NX-slot, met invalidatie vóór gebruik en na clear. Een writable scratchalias naar een executable fysiek frame wordt geweigerd; readonly input heeft een aparte RO/NX-helper. Geen willekeurige identitymap van vrije RAM/gaten/MMIO. Bewijs: kernelbron ed09818, volle M2/M3-gate, echte67-userpage/BSS-zero-regressie en per-processcleanupbalans. Herzie bij SMP/DMA/nieuwe mapping-API; de serialisatie geldt alleen in het huidige één-CPU/één-threadprofiel. Herstel via een consistente eerdere bronversie in een afzonderlijke checkout, zonder huidige wijzigingen te wissen.

**ADR-025 — Safe vaste capability-engine met afzonderlijke usercopygrens.** `capability-core` is eigen dependencyvrije no_std-code zonder unsafe/heap; de kernel bewaakt userpointers, syscallvolgorde en320-byte-kopieën. Engine-receive commit pas na volledige outputvalidatie, daarna volgt de in dit profiel onfeilbare bytecopy. Rootautoriteit komt uitsluitend uit het statische testmanifest, niet uit een algemene userspace-objectlookup. Revoke trekt afstammelingen in; exit ook eigen/issued autoriteit. Gesloten objecttombstones blijven bestaan tot laatste ref. Hosttests vergelijken de volledige Engine-toestand; kernelchecksums zijn diagnostiek. Bewijs:21coretests,alle acht echte M4-cases en onafhankelijke gerichte review. Herzie vóór threads/sharedmemory/DMA/dynamische applicatie- of objecttypen; geen extrapolatie naar zulke garanties. Geen boot-ABI-wijziging of hostconfigmigratie.

**ADR-026 — M5: geïsoleerde READ-dienst met vertrouwd virtio-transport.** De eerste systeemdienst draait als eigen Rust-programma in ring 3. De kernel bewaakt de getypeerde READ-capability, tickets, usercopy, PCI/MMIO en een vaste vijfpagina-DMA-pool. Het device op `00:05.0` gebruikt één splitqueue van acht plaatsen en maximaal één gelijktijdige sectorread. De dienst kan geen fysieke adressen, descriptors, PCI-configuratie of MMIO kiezen. Het protocol en de 17 foutcases staan vóór uitvoering vast in [M5-BLOCK-SERVICE.md](implementation/M5-BLOCK-SERVICE.md). Alleen eigen lokale crates zijn toegevoegd; geen nieuwe externe dependency.

**Status:** aanvaard en geïmplementeerd voor het vaste M5-QEMU-profiel. **Alternatieven:** alle dienstlogica in de kernel (grotere vertrouwde basis), of directe userspace-DMA (vereist aanvullend isolatieontwerp, buiten M5). **Eigenaar:** hoofdagent voor integratie en besluitbeheer; `kernel_m2` voor transport en mappinggrens.

**Onderbouwing/bronnen:**

- OASIS Open, *[Virtual I/O Device (VIRTIO) Version 1.2](https://docs.oasis-open.org/virtio/virtio/v1.2/cs01/virtio-v1.2-cs01.html)*, Committee Specification 01, 1 juli 2022: §2.4 (resetbevestiging), §2.7 en §2.7.8 (splitqueue en used-entry zonder epoch), §4.1 (PCI-transport) en §5.2 (blokdevice). Queuegrootte, één gelijktijdige read en de gepinde pool zijn projectkeuzes.
- Intel Corporation, *[Intel® 64 and IA-32 Architectures Software Developer’s Manual, Volume 3A: System Programming Guide, Part 1](https://www.intel.sg/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3a-part-1-manual.pdf)*, Order No. 253668-060US, september 2016: §4.1.4 en §4.5 (CPUID/MAXPHYADDR en gereserveerde fysieke adresbits), §§11.12.2–11.12.3 (IA32_PAT en selectie met PAT/PCD/PWT). `map_mmio` controleert de CPU-adresbreedte en vereist UC in PAT-entry 3 vóór de 4KiB-mapping met PAT=0, PCD=1 en PWT=1; de firmware-PAT blijft ongewijzigd. Bronnen gecontroleerd op 27 oktober 2026.
De vertrouwde basis omvat het kerneltransport en het protocolconforme QEMU-device/backend. M5 bevat geen IOMMU: CPU-isolatie en begrensde descriptoropbouw bewijzen geen containment van kwaadwillige DMA. De pool blijft gepind tot VM-stop; hergebruik vereist completion of bevestigde reset. Zonder resetack blijft zij in quarantaine en het device offline. Een used-entry heeft geen epoch; softwaretickets vervangen geen hardware-resetgarantie. Herzie dit besluit vóór directe userdriver-DMA, SMP, fysieke hardware of een andere devicebackend.

De eerste scope is alleen lezen op een synthetische 64KiB-wegwerpimage. Schrijven, persistentie en herstel blijven M6. Herstart gebruikt een begrensde testcoördinator en echte nieuwe dienst-PID/epoch plus IPC-capoverdracht; geen algemene spawn-API. De client blijft leven. Het externe testorakel controleert sectorchecksums op twee verschillende images bij dezelfde kernel en controleert dat alle imagebytes ongewijzigd blijven. Uitgevoerde resultaten en open punten staan uitsluitend in PROJECT-STATE.md en verification/m5/. Herstel: een afzonderlijke checkout van M1–M4 op `c630c59`; geen hostdisk- of firmwaremigratie.


## ADR-029 — Native objectdienst met revisies en begrensd journal

**Datum/status/eigenaar:** 27 oktober 2026, geïmplementeerd en volledige native gate geslaagd binnen het begrensde QEMU-profiel. Het native `Unknown`-foutpad heeft een bekende, ongerepareerde observerpanic. Hoofdagent integreert, afzonderlijke builders en AI-reviewers controleren service, broker en harnas.

**Context en besluit:** M6a bewaart één vaste waarde. M6b voegt een echte ring-3-objectdienst toe boven dezelfde begrensde READ/WRITE/FLUSH-ABI: vier objecten in twee endpointgebonden domeinen, GET/PUT/DELETE, maximaal 128 bytes en 32 append-only records. De limieten zijn proefgrenzen. De object-core is eigen dependencyvrije safe Rust; er worden geen nieuwe externe crates toegevoegd. De kernel houdt de bestaande DMA-broker plus een expliciete testobserver; objectpolicy draait in de dienst.

**Alternatieven en gevolgen:** een volledig bestandssysteem met namen, garbage collection en encryptie ineens toevoegen maakt de herstel- en bevoegdheidsgrens veel groter. Een extra hoststore levert geen native vooruitgang. Dit begrensde native fundament maakt meerdere objecten en blijvende verwijdering nu toetsbaar, met expliciete latere uitbreidingen voor ruimtehergebruik, namen, crypto en de desktop. Geen migratie van bestaande M6a-images; M6b gebruikt uitsluitend eigen wegwerpvolumes.

**Invariant:** de ontvangende endpointcapability bindt het domein. Geen appvelden als blok- of domeinautoriteit. Mutaties vergelijken de actuele revisie, en alleen een echte WRITE-completion gevolgd door FLUSH-completion mag een succesvolle reply opleveren. Een tombstone behoudt de revisie, zodat oude verzoeken geen verwijdering ongedaan maken. Corruptie geeft geen stilzwijgende terugval. Het servicecontract vereist dat onzekere mutaties `Unknown` opleveren en verdere geldige dienstoperaties blokkeren zonder automatische replay. Dit `Unknown`-contract is alleen op host-ABI-niveau getest en is voor native WRITE-/FLUSH-fouten niet gerealiseerd door de bekende observerpanic. Full verschijnt vóór muterende I/O. Verwijderen is logisch; oude payloadbytes blijven in het journal en zijn niet cryptografisch beschermd.

**Onderbouwing en bewijs:** eigen contract `implementation/M6B-OBJECT-STORAGE.md`, eerdere virtio/capability-ABI uit ADR-026/M6a; wire- en journalindeling zijn interne projectbesluiten. De volledige eindgate op `1e61a20` slaagt: 18 groepen/14 echte VM-runs, 22 Rust-hosttests en 26 harnastests. Daaronder vallen zes koude recoveries, uitputting, corruptie en twee echte externe kills. Capture en een aparte audit van 121 artifacthashes/zeven schijfvolumes slagen; een extra Python-parser controleert onafhankelijk de echte schijfbytes zonder de Rust-journalcode te delen. Zeven host-ABI-adapterproeven controleren `Unknown`-uitkomsten. Native WRITE-/FLUSH-fouten kunnen via `sent()` in `kernel/src/m2/m3/m6b.rs` een kernel panic veroorzaken: de observer vergelijkt de reply eerst met een verwachte `Ok`-reply. Dit defect is niet gerepareerd; gecontroleerde end-to-end-afhandeling als `Unknown` ontbreekt ook in `milestone-m6b`. De native gate bevat geen devicefoutinjectie voor dit pad. Exacte bron, regressies en bewijsgrenzen staan in `PROJECT-STATE.md` en `verification/m6b/`.

**Herziening en rollback:** een herziening van het native `Unknown`-pad vereist een native reparatie, gerichte WRITE-/FLUSH-devicefoutinjectie, controle van het daadwerkelijke clientantwoord en nieuwe native verificatie met bijbehorende bronbinding. Herzie ook bij meer objecten, compaction, meerdere schrijvers, sleutels, native UI of andere devices. M6b heeft een eigen feature/ELF/volumeformaat; bestaande M1–M6a-profielen blijven selecteerbaar. Een aparte werkbranch maakt terugname mogelijk zonder hostschijven of echte gebruikersdata te wijzigen. Tests in QEMU bewijzen geen fysieke stroomuitval, IOMMU-containment, formele correctheid of productierijpheid.

Aanvulling 28 oktober 2026 op ADR-029: de terminale WRITE-/FLUSH-observerfout
is op `592dccf` hersteld en native getest met twee statussnapshotinjecties;
de historische beweringen hierboven gelden voor hun oude bron. Verder herstel
van lokale poll-LIMIT, echte brokerdeadline en resetquarantaine is de actieve
vervolgstap K08; uitgevoerde resultaten blijven in PROJECT-STATE.md.

## ADR-033 — opt-in native grafische bootstrap vóór productintegratie

Datum 28 oktober 2026, aanvaard als begrensde ontwikkelstap; root integreert,
G09 beheert core/loader/adapter. Het browserprototype geeft geen bewijs voor
native scherm- en invoerondersteuning. Daarom komt eerst een afzonderlijke
G0-feature met gevalideerde GOP-metadata, supervisor framebuffer en begrensde
PS/2-polling. Alternatieven zijn een volledige compositor/driver/IPC-broker
ineens bouwen (te grote eerste hardwarestap), of een browser op de host als
native presenteren (geen voortgang op eigen hardwaregrens).

Invariant: opt-in ABI 1.2 met vaste descriptor in de behouden BootInfo-pagina,
geen firmwarecalls na EBS, geen framebufferalias naar RAM/user/executable,
strikt begrensde RW/NX/UC-devicepagina's en safe `no_std`-render/invoercore.
De kernelweergave is tijdelijke diagnostiek, geen app-, object- of opslagpolicy.
Het scherm labelt de proef. Geen rechten voor willekeurige apps en geen nieuwe
extern gepubliceerde productie-ABI.

Contract/bronnen/acceptatie: [GRAPHICS-G0.md](implementation/GRAPHICS-G0.md).
Testbewijs wordt afzonderlijk opgeslagen; dit besluit is geen boottest.
Herzie vóór ring-3-compositor, IRQ-invoer, DMA/GPU, SMP, Unicode/IME of echte
hardware. Herstel via aparte checkout `7b4945f`; standaard BootInfo 1.1 blijft
selecteerbaar, geen echte schijf- of firmwaremigratie.


## ADR-034 — Native SDK en begrensde quantumport als afzonderlijke profielen

Datum 28 oktober 2026; geïmplementeerd als ontwikkelprofielen, eindacceptatie
op de gecombineerde bron nog uit te voeren. Root beheert ABI/bootconfig;
G01–G03 bouwen SDK/client, I01–I04 de port en controles.

De SDK hergebruikt de bestaande capability-ABI en 320-byte IPC-envelop.
Canonicalisatie, endpointrechten, antwoordherkomst en één uitstaand verzoek
zijn expliciet begrensd. Alleen EAGAIN vóór enqueue mag opnieuw worden
geprobeerd. Na enqueue, Unknown, ongeldige reply of deadline sluit de sessie
zonder automatische mutatieherhaling. De catalogus bepaalt revisies/namen;
de kernel krijgt geen app- of documentbeleid. De bestaande M6c-probe kan het
SDK-transport selecteren, terwijl ruwe negatieve ABI-proeven hun oude pad
behouden. Een apart native voorbeeld gebruikt de echte catalogusclient;
compilatie is nog geen bewijs van algemene appstart.

De quantumport gebruikt de ongewijzigde eigen qsim-core met maximaal zes
qubits, 64 gates, 4096 shots en een vaste arena van 64 KiB per proces. Een
monotone allocator vermijdt onbewezen hergebruik; de proef beëindigt de
processen waarna de bestaande kernelcleanup de pagina's terugneemt. Software-
float uit de gepinde Rust-toolchain maakt de eerste port mogelijk zonder
schedulerwijziging voor FPU/SIMD-context. Een disassemblygate controleert het
concrete ELF inclusief instructieprefixes; deze controle is geen formeel
bewijs. Een nieuw opt-in profiel bevat twee userprocessen en uitsluitend een
vaste kerneltestobserver. Het profiel is exclusief ten opzichte van M4/G0.

De alternatieven — nieuwe syscalls, onbegrensde allocatie, FPU-state support
of een volledige jobdienst tegelijk — vergroten deze eerste wijzigingsgrens.
Gevolg: native referentieberekeningen en resourcegrenzen zijn nu uitvoerbaar,
maar willekeurige jobaanvoer/resultaten/annulering buiten het gecompileerde
proefprogramma blijven een eigen volgende stap. De Control-fixtures bewijzen
geen externe cancel of klokdeadline. L02 krijgt daarom nog geen punt.

Eigen contracten: SDK README/ABI, `implementation/QUANTUM-SPEC.md` en de
bestaande capability/M3-contracten. Gerichte native uitvoering, exacte
opdrachten, hashes, reviewbevindingen en grenzen worden afzonderlijk bewaard
in `verification/parallel-*`. Herzie vóór dynamische apps, meer qubits,
allocatiehergebruik, FPU/SIMD, SMP of een echte jobdienst. Beide profielen zijn
opt-in; herstel door hun eerdere bron in een aparte checkout te bouwen.
Geen migratie van echte data, nieuwe externe crate of fysieke hardwareactie.


### ADR-034: afzonderlijke invarianten en weerlegging

**SDK — security-invariant:** gewone apps verkrijgen alleen reeds toegekende
procesgebonden endpointrechten. Elke reply wordt canoniek en op herkomst
gecontroleerd. Na enqueue en verlies van zekerheid blijft een mutatie zonder
automatische replay; een ongeldige reply geeft geen extra rechten of succes.
**Weerlegd als:** dezelfde mutatie na Unknown/verloren reply opnieuw wordt
verstuurd, een vreemde afzender wordt geaccepteerd of een ongeldig capability-
bericht als geldig doorgaat. **Eigenaar:** G01–G03 voor SDK, root voor ABI en
integratie. **Testbewijs:** parallel-g01/g03 en parallel-final-sdk-m6c (deze
laatste toetst de runtimebridge, niet de losse catalogusvoorbeeldapp).
**Migratie/rollback:** bestaande ABI en journal blijven gelijk; SDKselectie is
opt-in, terugname via eerdere bron in aparte checkout zonder datamigratie.

**Quantum — security-invariant:** elk proefproces heeft private mappings en
maximaal 64 KiB arena; input blijft binnen 6 qubits / 64 gates / 4096 shots. SIMD/FPU en
vreemde syscall/hardware-instructies zijn niet toegestaan in het geselecteerde
scalarartifact. Simulatorbeleid blijft userspace; opruiming gebruikt echte
procesframes. **Weerlegd als:** grenzen kunnen worden overschreden, processen
elkaars geheugen bereiken, ongewenste instructies door de artifactgate komen
of cleanup toegewezen frames achterlaat. **Eigenaar:** I01–I04 voor port/gate,
root voor de testobserver en integratie. **Testbewijs:** parallel-i04,
parallel-final-qsim en parallel-final-m3. Dit is geen formeel bewijs.
**Migratie/rollback:** exclusief opt-in profiel; eerdere bron afzonderlijk
bouwen, zonder opgeslagen gebruikersdata of hardwarefirmware te wijzigen.


## ADR-035 — Security Floor blijft actief bij vrijwillig minder privacy

**Datum/status/eigenaar:** 28 oktober 2026; aanvaard als product- en
architectuurinvariant, nog niet volledig native gerealiseerd. Hoofdagent bewaakt
integratie; security-eigenaars toetsen de enforcementgrenzen.

**Context:** Nieuw OS moet privacybeschermende standaarden hebben, maar de
gebruiker mag bewust online diensten, telemetrie, microfoon, cloudsync of
andere gegevensdeling toestaan. Een dergelijke keuze mag niet veranderen in
een algemene “trusted app”- of “minder veilige modus”.

**Besluit:** privacykeuzes verruimen alleen expliciete capabilities en
gegevensstromen. Kernel-/procesisolatie, capabilitycontrole, geheugen- en
sleutelisolatie, pakket-/update-authenticiteit, bescherming van niet-gedeelde
data en toekomstige DMA/IOMMU-grenzen blijven buiten die privacykeuze.

**Security-invariant:** toestemming vergroot functionaliteit, niet vertrouwen.
Een internetgrant alleen geeft geen document-, key-, camera-, microfoon-,
NAS-, proces- of kernelrecht.

**Weerlegd als:** een privacytoggle, providerkoppeling, telemetry-opt-in of
focus/gamingmodus een bredere privilegeklasse activeert of bestaande
isolatiechecks omzeilt.

**Bewijs/migratie:** toekomstige native LP-/capabilitytests; geen nieuw
acceptatiepunt door dit document. Bestaande grants blijven semantisch gelijk.
Rollback: herstel dit ADR en de gekoppelde contracten gezamenlijk; geen
datamigratie in deze documentatiestap.

## ADR-036 — Externe provider mag alleen opaque ciphertextopslag zijn

**Datum/status/eigenaar:** 28 oktober 2026; aanvaard architectuurdoel, nog geen
native provideradapter. Opslag-/securityeigenaar beheert formaat en keygrens.

**Context:** de eerdere revisie verbood leverancierscloud volledig. Optioneel
gebruik van bestaande algemene opslag, zoals OneDrive en Google Drive, moet mogelijk
zijn zonder de provider leesrecht op de opgeslagen inhoud te geven.

**Alternatieven:** absolute provideruitsluiting beperkt keuze; provider-native
encryptie/key escrow verschuift vertrouwen naar de provider; zelfgemaakte
cryptografie is onaanvaardbaar.

**Besluit:** een expliciet gekozen externe provider mag uitsluitend opaque,
client-side versleutelde Nieuw-OS-objecten opslaan. De provider krijgt geen
inhoudssleutel, documentplaintext, gevoelige manifestplaintext, lokale zoekindex
of AI-context. Geen stille providerfallback. Het OS mag een eigen versieerbaar
container-/objectformaat definiëren boven onderhouden standaardcryptografie.

**Security-invariant:** bezit van alleen providerstate/credentials levert geen
geaccepteerde documentplaintext op binnen het gedocumenteerde threat model.

**Weerlegd als:** providercredentials of opgeslagen bytes zonder een geldige
Nieuw-OS-key voldoende zijn voor plaintext; een provideradapter andere lokale
objecten/keys kan lezen; of een outage stil een andere provider activeert.

**Bewijs/migratie:** [implementation/ENCRYPTED-OBJECT-STORAGE.md](implementation/ENCRYPTED-OBJECT-STORAGE.md)
definieert EO-acceptatie. De bestaande NAS-route blijft geldig. Geen echte
provideraccount, productiekey of gebruikersdata wordt door dit besluit
geconfigureerd.

## ADR-037 — Eigen premium native productiviteitsfamilie, geen Office-ripoff

**Datum/status/eigenaar:** 28 oktober 2026; productbesluit voor P3/P4/M7,
nog niet native gerealiseerd. Product-/UI-eigenaar beheert designsysteem;
graphics-eigenaar beheert later renderer/compositorgrenzen.

**Context:** privacyvriendelijke alternatieven mogen niet aanvoelen als
verouderde systeemsoftware. Tegelijk zou een directe kopie van Office,
Windows of macOS de eigen productidentiteit ondermijnen.

**Besluit:** Materiaal blijft het objectmodel. Eigen tekst-, tabel-,
presentatie-, PDF/media- en latere productiviteitstools delen één designsysteem,
contextuele gereedschappen, geschiedenis, delen, herstel, motion en
toegankelijkheid. Geen permanente Office-ribbonkopie als basis. Een gedeelde
native GPU-geaccelereerde UI-/renderlaag is het langetermijndoel zodra de
graphicsbasis dat toelaat; exacte renderer/API blijft een later technisch
besluit. DOCX/XLSX/PPTX/PDF zijn interoperabiliteitsdoelen die per feature
moeten worden getest.

**Productinvariant:** herkenbare basishandelingen blijven voorspelbaar; de
eigenheid komt uit Materiaal, objectcontinuïteit, contextuele tools en
capabilitybewust delen, niet uit onbekende bediening omwille van originaliteit.

**Weerlegd als:** kernapps eindigen als inconsistente losstaande webviews,
de interface structureel een bestaande suite kopieert, of privacy alleen via
beheerachtige UX bruikbaar is.

**Bewijs/migratie:** PX-acceptatie in PRODUCT-SPEC.md en latere gebruikers-/
native tests. Het huidige P2-browserprototype blijft ontwerpgereedschap en
wordt niet door dit ADR tot productruntime verklaard.

## ADR-038 — Privacyuitleg is optioneel; capabilitybeslissingen blijven echt

**Datum/status/eigenaar:** 28 oktober 2026; aanvaard UX-/securitycontract,
nog niet native afgedwongen.

**Context:** begrijpelijke voorbeelden en kritische vragen kunnen privacykeuzes
duidelijker maken, maar herhaalde waarschuwingen kunnen de ervaring verstoren.
Educatieve privacyuitleg moet volledig uit te schakelen zijn.

**Besluit:** de UI scheidt (a) noodzakelijke beslissingen over nieuwe
capabilities van (b) educatieve privacyuitleg. Educatieve uitleg kan globaal
of per app uit. Een reeds geldige grant wordt niet bij iedere normale
handeling opnieuw gevraagd. Focus/gaming onderdrukt niet-urgente educatieve
meldingen; een bewust gestarte nieuwe gevoelige functie mag nog wel een
compacte noodzakelijke grantvraag tonen.

**Security-invariant:** het uitschakelen van privacyuitleg wijzigt geen
capabilityset en creëert geen impliciete “allow all”.

**Weerlegd als:** dezelfde statewijziging zowel de uitleg als toegangsrechten
uitzet/verruimt, of het systeem op basis van “uitleg uit” een nog niet verleende
capability als toegestaan behandelt.

**Bewijs/migratie:** LP-29/PX-04 en toekomstige native UI-/brokerproeven. Bestaande
grants migreren niet; alleen de presentatievoorkeur wordt afzonderlijk bewaard.

## ADR-039 — DocumentSession native controleren via de bestaande catalogus

**Datum:** 28 oktober 2026.  
**Status:** implementatie geïntegreerd in de auditbranch; host-/crossbuildbewijs en een
actuele twee-VM normal/koude-recovery-proef zijn uitgevoerd, maar dit verandert
geen bestaand roadmapcriterium.  
**Eigenaar:** app/SDK voor de gebruikersproef en library, runtime voor de
acceptatiepoort, root voor selectie en integratie.

**Context:** de bestaande M6c-probe gebruikte de catalogus via directe helpers,
terwijl de echte `DocumentSession`-library al bestaat. De integratiestap moet
die echte library in ring 3 gebruiken zonder kernelobserver, capabilityrechten,
IPC-envelop of opslagformaat te verruimen.

**Alternatieven:** (1) de directe catalogushelpers blijven gebruiken, waardoor
de appbibliotheek niet native wordt getoetst; (2) een afzonderlijke user-ELF met
gekopieerde cataloguscode bouwen, wat duplicatie en bewijsoppervlak vergroot;
(3) de bestaande M6c-userflow opt-in de echte library laten gebruiken en de
bestaande catalogusmodules linken. Gekozen is optie 3.

**Besluit/implementatie:** `DocumentSession<NativeTransport>` draait opt-in in
de bestaande M6c-gebruikersprocessen. `NOS_DOCUMENT_SESSION=1` vereist
`NOS_SDK_RUNTIME=1` en is beperkt tot normal/recover. Openen, lokaal bewerken,
CAS-save, opnieuw openen en LIST blijven userspace. `list()` houdt één eigenaar
van de endpoints; service en typed client delen de bestaande catalogusmodules.

**Security-invariant:** de route gebruikt uitsluitend reeds toegekende
procesgebonden endpoints. Replies blijven canoniek gevalideerd; LIST mag lokaal
concept/conflict/onzekerheid niet wissen en een gesloten sessie niet heropenen.
Onzekere saves worden niet automatisch herhaald. Syscall-, IPC-, geheugen- en
capabilitygrenzen worden niet verruimd. ADR-035 (Security Floor) blijft volledig
van toepassing.

**Onderbouwing/bronnen:** `docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md`,
`sdk/document-session/README.md`, `docs/verification/parallel-g11/`,
`docs/verification/parallel-i13/` en `docs/verification/parallel-i15/`.
Voor de afzonderlijke netwerkparser gelden RFC 768, RFC 791 en RFC 1071, zoals
vastgelegd in `services/net-protocol/README.md`.

**Testbewijs:** de repositorybrede audit-CI voert DocumentSession-hosttests,
SDK-/compilerbindingcontroles, native M6c-build, een baseline M6c-QEMU-smoke en
twee afzonderlijke DocumentSession-VM's (normal en koude recovery) uit.
Historische bewijsbundels blijven aan hun oorspronkelijke bron gekoppeld.

**Weerlegd als:** de proef nieuwe rechten nodig heeft, een draft via LIST
verliest, een onzekere save herhaalt, een gesloten endpoint opnieuw bruikbaar
maakt, de observer/oracle moet versoepelen of een andere binary als documentapp
kan meetellen.

**Migratie/rollback:** standaardprofielen behouden hun bestaande selectie. De
opt-in kan worden uitgezet; er is geen migratie van echte gebruikersdata,
firmware of fysieke apparaten. De headless documentproef is geen Materiaal-GUI,
algemene appstarter, grotere-documentondersteuning of ruimtehergebruik.

## ADR-040 — Copy-on-write volume (M6e) in plaats van append-only journal

**Datum:** 28 oktober 2026. M6b/M6c/M6d blijven ongewijzigd en afzonderlijk
bouwbaar.

**Status (gescheiden):** *ontwerp* geaccepteerd voor het opt-in profiel
`native-volume`. *Implementatie*: `services/volume`, de native dienst/app in
`sdk/examples/volume-docs`, de SDK-blok-API en kernelprofiel `native-volume` op
PR #47. *Testbewijs*: zie hieronder; GitHub Actions moet op iedere nieuwe
PR-head opnieuw groen zijn. *Onbekend/niet getest*: fysieke schijven en
stroomuitval, authenticatie/encryptie, back-up/export en grotere volumes.

**Onderbouwing/bronnen:** OASIS, *Virtual I/O Device (VIRTIO) Version 1.2*,
Committee Specification 01, 1 juli 2022, §5.2 Block Device, §5.2.6.2:
een write is stabiel wanneer na de voltooide write een `VIRTIO_BLK_T_FLUSH`
wordt verstuurd en zelf voltooid is
(https://docs.oasis-open.org/virtio/virtio/v1.2/cs01/virtio-v1.2-cs01.html,
geraadpleegd 28 oktober 2026). De volgorde inhoud → FLUSH → tabel → FLUSH →
ACK steunt op die garantie; een emulator die haar niet nakomt valt buiten dit
bewijs.

**Testbewijs:** hosttests `services/volume/tests/` (incl. uitputtende
crashsimulatie en de test met twee gemengde schrijfpogingen),
`tests/test_volume_disk.py`, en de native gate `tools/test_volume.py` met
bewijs in `docs/verification/m6e/` en
[M6E-ACCEPTANCE](verification/M6E-ACCEPTANCE.md). Wat daar niet staat, is
niet uitgevoerd.

**Besluit:** grote documenten en ruimtehergebruik (S04/S05) krijgen een nieuw
userspace-volumeformaat met twee alternerende, per sector verzegelde
tabelkopieën en 4 KiB-inhoudschunks. Een mutatie schrijft alleen naar chunks
die de duurzame tabel niet refereert; daarna FLUSH, de volgende tabel in de
oudere slot, FLUSH, ACK. Vrijgekomen chunks worden door latere commits
hergebruikt. Er is geen journal en dus geen recordlimiet; een aparte
compactiestap is niet nodig, omdat er geen append-only log is om te
compacteren. Crashconsistente vrijgave vervult hier de rol die "compactie" in
criterium S04 beschrijft. Deze interpretatie wordt hier expliciet vastgelegd,
niet stilzwijgend toegepast.

**Security-invariant:** de kernel blijft een generieke blokbroker zonder
formaat- of domeinkennis (kernel/AGENTS.md: geen opslagpolicy in privileged
code). Domeinautoriteit komt uitsluitend van het kernelgebonden endpoint. Apps
krijgen geen blokrecht. Read-only recovery wordt afgedwongen door emulator,
virtio-onderhandeling, capability (alleen READ) en dienst. Een fout tijdens de
tabelwrite geeft `Unknown` zonder replay. Een niet-canonieke, volledig
verzegelde nieuwste kopie stopt herstel.

**Weerlegd als:** recovery na een crashpunt een andere staat oplevert dan
vóór of na de onderbroken stap; een write buiten de scope of naar een
gerefereerde chunk plaatsvindt; guest en onafhankelijk orakel verschillende
digests vinden; quota/Full pas ná I/O worden gemeld; of een read-only
recovery de image wijzigt.

**Reviewcorrectie (CodeRabbit, PR #47):** de oorspronkelijke verzegeling bond
een sector alleen aan slot, index en generatie. Twee onderbroken pogingen voor
dezelfde generatie konden daardoor samen een schijnbaar volledige kopie vormen
en een teruggedraaide mutatie laten herleven. Iedere sector draagt nu ook een
digest over de payload van alle sectoren van zijn kopie; recovery accepteert
alleen een kopie waarvan alle sectoren dezelfde, herberekende digest hebben.
Een gemengde kopie geldt als gescheurd.

**Restrisico:** één onleesbare sector in de nieuwste tabelkopie leidt tot
terugval naar de vorige generatie (verlies van de laatste commit). FNV is geen
authenticatie (B02 open). Proefgeometrie: 16 objecten, 64 KiB per document,
256 KiB inhoud.

**Migratie/rollback:** opt-in feature; geen automatische migratie van M6c-
volumes en geen gebruik van echte schijven. Terugdraaien = feature niet
selecteren of de M6e-commits gericht reverten.

**Eigenaar:** hoofdagent voor formaat, contract en integratie
(`services/volume`, `tools/volume_disk.py`, gate en bewijs); `kernel_m2` voor de
generieke blokbroker in `kernel/src/m2/m3/volume.rs`.

## ADR-041 — Volume v2: collecties, metadata, begrensde historie en transacties (S06)

**Datum/status:** 29 oktober 2026, geaccepteerd voor het opt-in profiel
`native-volume`. Formaat v2 (`NOSVOL02`) en IPC v6 vervangen v1/IPC v5 binnen
dat profiel; er is geen migratie, omdat het profiel alleen wegwerpimages in
QEMU gebruikt. M6b/M6c/M6d blijven ongewijzigd.

**Status:** ontwerp geaccepteerd. Implementatie: `services/volume` (v2),
`sdk/examples/volume-docs` (dienst en app), kernelgeval `test-volume-cut-txn`,
orakel en gate. Testbewijs: zie
[S06-ACCEPTANCE](verification/S06-ACCEPTANCE.md). Onbekend: fysieke schijven en
stroomuitval, authenticatie/encryptie (B02), back-up/export (S09), grotere
historie en de native Materiaal-interface.

**Context:** Materiaal werkt vanuit objecten met collecties, type, labels en
geschiedenis; herstel maakt een nieuwe revisie (PRODUCT-SPEC §2, P2-05). M6e had
alleen naam en actuele inhoud.

**Alternatieven:** (a) een journal of log van bewerkingen naast de tabel:
verworpen, omdat het de recordlimiet en compactie terugbrengt die ADR-040
juist verwijderde; (b) onbeperkte historie via een aparte versieketen op schijf:
uitgesteld, omdat begrensde opslag en quota dan nog niet te bewijzen zijn;
(c) collecties als gewone objecten: verworpen, omdat lidmaatschap dan niet
atomair met objectmetadata te controleren is.

**Besluit:** iedere objectentry krijgt een tweede tabelsector met twee
historische versies; de kop bewaart `tags`, `kind`, collectie en logische
tijden. Die tijden komen uit een klok per domein in de tabelheader: het aantal
commits van dat domein (de klokken tellen op tot de generatie). Collecties
staan in een eigen tabelsector per volume (4 per domein) en zijn leesbaar met
COLLECTION_STAT, ook read-only. RESTORE maakt van een historische versie een
nieuwe revisie zonder inhoud te herschrijven (chunks worden opnieuw
gerefereerd); de teruggezette versie verlaat de historie en de vervangen
actuele versie neemt haar plaats in, zodat geen andere versie wegvalt. DELETE
bewaart de laatste versie als historie; PURGE geeft historie vrij. TRANSACT
voert maximaal vier META/DELETE/PURGE-stappen op verschillende objecten van
één domein uit in één tabelcommit. Alle wijzigingen gebruiken het
ongewijzigde M6e-commitprotocol. Een schrijfbare nieuwe instantie flusht na
recovery vóór het eerste verzoek; mislukt dat, dan start zij niet.

**Onderbouwing/bronnen:** ADR-040 (commitvolgorde, VIRTIO 1.2 §5.2.6.2) en
PRODUCT-SPEC §2/§6 voor de takenlijst. Geen nieuwe externe bron nodig.

**Security-invariant:** domein komt uitsluitend van het kernelgebonden
endpoint; een transactie kan alleen objectindices binnen het eigen domein
noemen, en een object kan alleen in een collectie van het eigen domein staan.
Quota telt behouden historie mee, dus historie kan geen ruimte buiten de quota
vasthouden. Leesantwoorden (STAT, LIST, COLLECTION_STAT, READ) en de revisies,
stempels en tellers in mutatieantwoorden bevatten alleen toestand van het eigen
domein (tijdstempels gebruiken de eigen domeinklok; LIST geeft een
domeinteller). Twee kanalen tussen domeinen zijn bewust restrisico: `Full`
hangt af van de gedeelde, overboekte volumeruimte (2 × 40 van 64 chunks), en
een mislukte tabelwrite van één domein sluit de instantie, waarna ook het
andere domein `Unknown` krijgt. De kernel blijft formaatloos.

**Weerlegd als:** een gedeeltelijk toegepaste transactie na een crash of fout
zichtbaar is; RESTORE inhoud wijzigt of andere historie wist; een object naar
een collectie van een ander domein kan; historie quota omzeilt; of guest en
onafhankelijk orakel verschillende domeindigests (inclusief collecties)
vinden.

**Reviewcorrecties (29 oktober 2026, vóór acceptatie):** een onafhankelijke
review vond vijf punten die in dit besluit zijn verwerkt, elk met een
regressietest: (1) de globale generatie in STAT
liet een domein de commits van het andere tellen; (2) collecties waren via IPC
alleen schrijfbaar; (3) RESTORE van de nieuwste versie bij volle historie wiste
de oudste; (4) een nieuwe instantie kon schrijven terwijl de gekozen kopie nog
alleen in de schijfcache stond; (5) de gate kon een kill vóór de tabelwrite
niet onderscheiden van een gescheurde tabelwrite. De runner eist nu een echt
gescheurde poging (prefix van g+1-sectoren boven de vorige kopie).

Een tweede review (van die correcties) vond geen fout in de productiecode,
maar wel tests die de bedoelde fout niet vingen en twee te ruime claims. Ook
die zijn gecorrigeerd:
- de test op ongerelateerde generaties werd vacuüm door de klokken;
- de LIST-cursortest bereikte de overloop niet;
- COLLECTION_STAT had geen negatieve tests;
- de losse klokgrenzen waren niet afzonderlijk getest;
- invariant 7 claimde volledige non-interferentie;
- een read-only antwoord op een `Unknown` commit werd als definitief
  gepresenteerd.

Sindsdien is de mutatiecontrole herhaalbaar en vastgelegd.
`tools/mutate_volume.py` schakelt per mutant precies één regel uit in een
kopie van de bron en eist dat ten minste één test faalt. Het rapport met
commando, exitstatus en falende tests staat in
`docs/verification/s06/volume-mutants.json`.

**Gevolgen:** v1-images worden door v2 als corrupt geweigerd (andere magic);
het M6e-bewijs blijft gebonden aan zijn eigen bron, en de S06-gate herhaalt de
S04/S05-scenario's op v2.

**Restrisico:** historie is begrensd tot twee versies (de oudste valt weg bij
een nieuwe versie); geen zoekindex; FNV is geen authenticatie. `Full` en de
instantiebrede `Unknown` na een mislukte tabelwrite zijn zichtbaar over
domeinen heen. Een read-only antwoord op een `Unknown` commit is voorlopig.

**Migratie/rollback:** geen migratie van v1-volumes; terugdraaien = de
S06-commits gericht reverten (dan geldt weer v1 met zijn eigen bewijs).

**Eigenaar:** hoofdagent voor formaat, contract, orakel en gate; `kernel_m2`
voor het kernelgeval `test-volume-cut-txn`.

## ADR-042 — Procesbeheer als capabilities: `Launch`- en `Process`-objecten, spawn/kill/status (K05)

**Datum:** 29 oktober 2026. Profiel `native-supervisor`, opt-in; alle
bestaande profielen blijven ongewijzigd bouwbaar.

**Status (gescheiden):** *ontwerp* vastgelegd in
[K05-PROCESS-MANAGEMENT](implementation/K05-PROCESS-MANAGEMENT.md).
*Implementatie*: `crates/capability-core` (objecttypen, rechten 128/256/512),
`kernel/src/m2/m3/supervisor.rs` (syscalls 0x30, 0x50–0x52, orphan-reaping),
`sdk/runtime` (ABI 1.3-wrappers), `sdk/examples/supervisor` (supervisor,
worker, client). *Testbewijs*: hosttests en de gate `tools/test_supervisor.py`;
zie [K05-ACCEPTANCE](verification/K05-ACCEPTANCE.md). Wat daar niet staat, is
niet uitgevoerd. *Onbekend*: SMP, fysieke hardware, gebruikersgeleverde images
(A02), beleid voor een systeembrede dienstbeheerder.

**Besluit:**
1. Starten van een proces vereist een `Launch`-capability op een ingebouwde
   image. Zulke objecten maakt alleen het bootmanifest aan. Een proces zonder
   `Launch` krijgt `EBADHANDLE`, `ETYPE` of `EACCESS` en er ontstaat niets.
2. `proc_spawn` geeft het kind alleen de maximaal twee expliciet meegegeven,
   verminderde capabilities (dezelfde regels als IPC-overdracht: `TRANSFER`
   vereist, nooit `RECV`, geen rechtenvergroting). Alle fallibele controles
   gaan vooraf aan iedere toewijzing.
3. De starter ontvangt de enige handle op een `Process`-object met
   `OBSERVE|KILL`. Het objecttype kent geen `DERIVE`, `TRANSFER` of `REVOKE`:
   deze autoriteit kan niet worden gedeeld of gestolen.
4. Dood (exit, fault, kill) loopt via het bestaande `cleanup`-pad; het
   `Process`-object bewaart toestand en code tot de eigenaar zijn laatste handle
   sluit. Een lopend kind zonder handles blijft als object bestaan, zodat de
   kernel de band met de starter kan blijven afdwingen.
5. Kinderen van een gestorven starter worden in dezelfde kernelgebeurtenis
   beëindigd, iteratief (hoogstens 16 rondes), nooit recursief op de
   kernelstack. Er blijft geen proces over zonder verantwoordelijke.
6. `endpoint_create` (0x30) wordt een gewone syscall binnen de bestaande
   quota (4 eigen endpoints per proces, 64 totaal). Dit herziet de M4-keuze
   "alleen het manifest maakt endpoints" bewust, omdat herstartbare diensten
   een eigen ingress nodig hebben; de RECV-eigendomsregel blijft onveranderd.

**Security-invariant:** autoriteit over processen zit uitsluitend in
kernelmetadata van het aanroepende proces; een getal uit een ander proces
levert hoogstens eigen bestaande autoriteit op (KS-12). Geen gedeeltelijke
staat na een geweigerde of mislukte spawn: de vrije framestand is voor en na
gelijk, anders stopt de run met `PARTIAL_SPAWN`. `check_invariants` eist dat
een lopend `Process`-object naar een levend proces wijst en dat ieder kind
door precies één object wordt genoemd.

**Weerlegd als:** een proces zonder `Launch` een kind kan starten; een
`Process`-handle via IPC of `derive` bij een ander proces belandt; na exit,
fault of kill frames, nodes, endpoints of gequeued capabilities achterblijven;
na de dood van een starter een kind blijft draaien; of een geweigerde spawn de
vrije framestand verandert.

**Restrisico:** een supervisor kan zijn eigen kinderen misbruiken binnen de
rechten die hij heeft; er is nog geen beleid voor wie supervisor mag zijn.
Exitcodes worden modulo 2^56 gerapporteerd. De ABI-minor stijgt naar 1.3;
oudere profielen melden 1.0–1.2 en kennen de nieuwe syscalls niet (`ENOSYS`).

**Migratie/rollback:** opt-in feature; terugdraaien = profiel niet selecteren
of de K05-commits gericht reverten. De rechtenbits 128/256/512 zijn in
`KNOWN_RIGHTS` opgenomen; oudere profielen wijzen ze bij endpoint- en
blokobjecten met `EINVAL` af, zoals voorheen.

**Eigenaar:** hoofdagent (engine, kernelprofiel, SDK, voorbeeld, gate, bewijs).

## ADR-043 — Entropy als capability en een ring-3 Key Service met onderhouden crypto-crates (B01)

**Datum:** 29 oktober 2026. Profiel `native-keyring`, opt-in; bestaande
profielen blijven ongewijzigd.

**Status (gescheiden):** *ontwerp* in
[B01-ENTROPY-AND-KEYS](implementation/B01-ENTROPY-AND-KEYS.md). *Implementatie*:
`drivers/virtio-rng`, `Entropy`-objecttype in `capability-core` (recht 1024),
kernelbroker `kernel/src/m2/m3/keyring.rs` (syscalls 0x60/0x61),
`services/keyring` (`keyring-core`), SDK-runtime ABI 1.4,
`sdk/examples/keyring`. *Testbewijs*: [B01-ACCEPTANCE](verification/B01-ACCEPTANCE.md);
wat daar niet staat, is niet uitgevoerd. *Onbekend*: echte hardware-RNG-
kwaliteit, TPM/hardwarebescherming, wachtwoordherstel, zijkanalen.

**Besluit:**
1. De kernel bezit het virtio-rng-apparaat en levert ruwe bytes alleen aan de
   houder van een `Entropy`-capability (manifest-only objecttype zonder
   `DERIVE`/`TRANSFER`/`REVOKE`). De kernel mengt, cachet of hergebruikt niets
   en bevat geen cryptografie (kernel/AGENTS.md).
2. Gereedheid en falen zijn expliciet: 1024 bytes opstartentropy door de
   SP 800-90B-tests Repetition Count (cutoff 6) en Adaptive Proportion
   (venster 512, cutoff 62; conservatief H = 4 bit/byte, α = 2⁻²⁰). Afwezig,
   vastgelopen of kort leverend apparaat → `BLOCKED` met reden; geen
   terugval, geen testseed in het productpad. Latere reseeds doorlopen
   dezelfde tests; falen → `DEGRADED` en weigering.
3. Alleen onderhouden, vastgepinde RustCrypto-crates met portable backends:
   `chacha20poly1305` 0.10.1 (XChaCha20-Poly1305), `chacha20` 0.9.1,
   `hkdf` 0.12.4, `sha2` 0.10.9, `zeroize` 1.8/1.9. De DRBG is de
   fast-key-erasure-constructie op de ChaCha20-keystream (Bernstein, 2017),
   geen eigen primitief. Dependencycontrole: licenties MIT/Apache-2.0,
   buildscripts alleen `generic-array` (`version_check`) en `zerocopy`-vrije
   boom; geen SIMD-instructies in de gegenereerde code (objdump-controle).
4. Sleutels verlaten de Key Service nooit; apps krijgen (domein, index)-
   handles; het domein volgt uit het kernelgebonden endpoint. AEAD bindt
   formaat, domein en index in de AAD. Per domein 16 sleutels; intrekking is
   permanent en reist mee in de herstelblob.
5. Herstel zonder escrow: een willekeurig 256-bit herstelgeheim voor de
   aanvragende app en een blob (hoofdsleutel + intrekkingsbitmaps, 76 bytes)
   onder dat geheim. RESTORE in een andere instantie herleidt alle
   domeinsleutels; de dienst bewaart het geheim niet.

**Security-invariant:** geen sleutelgeneratie vóór `READY`; geen bytes uit
een geweigerde, kort leverende of time-out-aanvraag; `entropy_poll` schrijft
pas na validatie van de hele uitvoerbuffer en wist de brokerkopie; een
gestorven aanvrager krijgt zijn antwoord niet alsnog via een ander proces.
Cross-domain- en manipulatiepogingen leveren `Auth`/`BadHandle` zonder
gedeeltelijke plaintext.

**Weerlegd als:** een proces zonder `Entropy` bytes krijgt; een dienst
sleutels maakt zonder geslaagde opstarttests; gemanipuleerde ciphertext,
tag of nonce plaintext oplevert; een hersteld domein een ingetrokken sleutel
weer bruikbaar maakt; of de kernel na afloop frames, nodes of objecten
achterlaat.

**Restrisico:** QEMU levert host-`/dev/urandom`; de tests toetsen gedrag,
niet de werkelijke min-entropie van fysieke hardware. De hoofdsleutel staat
onbeschermd in het dienstgeheugen (geen TPM). Zeroize is hygiëne, geen
garantie over kopieën in registers of crashgeheugen. Eén broker-aanvraag
tegelijk: diensten herhalen bij `EAGAIN`.

**Migratie/rollback:** opt-in; terugdraaien = profiel niet selecteren of de
B01-commit reverten. Rechtenbit 1024 wordt bij andere objecttypen met
`EINVAL` afgewezen.

**Eigenaar:** hoofdagent (driver, engine, kernelbroker, core, SDK, voorbeeld,
gate, bewijs).

## ADR-049 — Hybride ondertekende kernelupdates met slotopslag, versievloer en terugval (B03)

Bij integratie van PR #61 met main `6d3bbf0` hernummerd van ADR-044 naar
ADR-049: ADR-044 op main beschrijft het netwerk. Beide besluiten blijven
inhoudelijk behouden; historisch bewijs wordt niet herschreven.

**Datum:** 29 oktober 2026. Loaderfeature `signed-boot` en kernelprofiel
`native-update`, beide opt-in; loaders zonder de feature (alle andere
profielen) houden het bestaande ESP-pad (`kernel.elf`) ongewijzigd. Een
`signed-boot`-loader valt nooit op dat pad terug.

**Status (gescheiden):** *ontwerp* in
[B03-SIGNED-UPDATES](implementation/B03-SIGNED-UPDATES.md). *Implementatie*:
`crates/update-core` (manifest `NOSMAN01`, slotopslag `NOSSLOT1`, hybride
verificatie, selectie- en installatiebeleid), `boot/loader/src/signed.rs`,
`kernel/src/m2/m3/update.rs` (generieke blokbroker over de slotopslag, één
gelabelde crashfixture), `sdk/examples/update` (ring-3 update-dienst),
`tools/update-tool` (keygen/sign/pack/verify), `tools/update_disk.py`
(onafhankelijk hostorakel), gate `tools/test_update.py`. *Testbewijs*:
[B03-ACCEPTANCE](verification/B03-ACCEPTANCE.md); wat daar niet staat, is niet
uitgevoerd. *Onbekend*: firmwareverificatie van de loader zelf (Secure Boot,
B04), sleutelrotatie en intrekking in productie, netwerkdistributie,
fysieke schijven en stroomuitval, gedrag met een aanvaller die OVMF_VARS kan
schrijven.

**Besluit:**
1. Een kernelimage wordt alleen gestart uit een slot waarvan het manifest
   met **beide** vastgepinde ankers verifieert, Ed25519 (`ed25519-dalek`
   2.2.0, serial backend) én ML-DSA-65 (RustCrypto `ml-dsa` 0.1.1, FIPS 204),
   over dezelfde 84 canonieke bytes, en waarvan het image exact de lengte en
   SHA-256 (`sha2` 0.10.9, `force-soft`) uit dat manifest heeft. Eén geldige
   handtekening is onvoldoende; er is geen terugval naar één algoritme.
2. De loader bewaart de hoogste gestarte versie in de UEFI-NV-variabele
   `NieuwOsMinVersion` (vendor-GUID `50402e54-a387-47bc-8f75-dcd79cd0a603`,
   NV+BS) en weigert ieder slot met een lagere versie als `DOWNGRADE`, ook
   als de header ernaar wijst. De vloer stijgt alleen.
3. De header van de slotopslag is een onondertekende hint voor de slotvolgorde.
   Faalt het aangewezen slot (formaat, sleutel-ID, handtekening, digest, vloer,
   lezen), dan wordt het andere slot geprobeerd; falen beide, dan stopt de
   boot gecontroleerd (`BOOT:FAIL:NO_VALID_SLOT`, exit 0x11); ontbreekt de
   slotopslag, dan `BOOT:FAIL:NO_STORE`. Een lege, ongeldige of ontbrekende
   slotopslag valt nooit terug op een onondertekende ESP-kernel, en de build
   legt voor dit profiel geen `kernel.elf` op de ESP. Een correct ondertekend
   maar onlaadbaar image (ELF-parser) is een `FORMAT`-weigering van dat slot;
   de vloer stijgt pas voor een image dat de loader ook werkelijk kan laden.
4. De update-dienst schrijft uitsluitend naar het slot dat **niet** draait.
   Het draaiende slot wordt bepaald door de geverifieerde manifestversie die
   gelijk is aan de door de kernel gemelde eigen imageversie, niet door de
   header. Volgorde: image, FLUSH, manifest, FLUSH, header, FLUSH; een
   onderbreking laat het actieve slot en de header intact en de installatie
   is idempotent en hervatbaar.
5. De dienst verifieert de inboxbundel met dezelfde ankers, controleert de
   digest vóór de eerste schrijfactie en weigert `version ≤ huidige` als
   up-to-date respectievelijk downgrade. Loader en dienst controleren
   onafhankelijk; de kernel kent geen manifest- of slotformaat en bevat
   geen cryptografie (kernel/AGENTS.md).
6. De ankers in de repository zijn **TEST-ankers** (`tools/update-keys/TEST-*`,
   gegenereerd met `update-tool keygen`), inclusief seeds, zodat CI
   reproduceerbaar ondertekent. Zij beschermen niets buiten de gate. Een
   releaseprofiel vervangt de ankers in een afzonderlijke, buiten CI gehouden
   ondertekenstap; sleutelrotatie en intrekking horen bij B04.
7. Dependencycontrole (`cargo tree -p update-core`): `ed25519-dalek` 2.2.0 en
   `curve25519-dalek` 4.1.3 (BSD-3-Clause), `ml-dsa` 0.1.1, `module-lattice`
   0.2.3, `hybrid-array` 0.4.15, `keccak`/`shake`/`sponge-cursor`,
   `signature` 2.2/3.0, `sha2` 0.10.9, `digest`, `crypto-common` (MIT OR
   Apache-2.0), `subtle` 2.6.1 (BSD-3-Clause). Buildscripts: alleen
   `curve25519-dalek` (platformdetectie voor de woordbreedte, geen netwerk) en
   `generic-array` (`version_check`); één proc-macro (`curve25519-dalek-derive`,
   via `syn`/`quote`). De SIMD-backends zijn uitgeschakeld
   (`curve25519_dalek_backend="serial"`, sha2 `force-soft`); `objdump` van
   loader en dienst toont 0 xmm/ymm- en 0 x87-instructies.
   De ML-DSA-verificatie houdt ongeveer 200 KiB stack vast: de loader draait de
   selectie op een eigen 1 MiB-allocatie en het profiel `native-update` geeft
   de dienst 512 KiB gebruikersstack in plaats van 64 KiB.

**Security-invariant:** geen byte van een kernelimage wordt uitgevoerd
voordat formaat, sleutel-ID's, beide handtekeningen, lengte, digest en de
versievloer zijn gecontroleerd; het laatst geldig gestarte slot blijft bij
iedere onderbreking van een installatie ongewijzigd beschikbaar.

**Gevolgen:** de slotopslag kost 6 MiB per apparaat (drie slots van 2 MiB);
`image_length` is begrensd op 2 MiB. Pakket- en app-updates (A02),
netwerkdistributie (N-spoor) en de verificatie van de loader door de firmware
(B04) blijven open. De `KERNEL:IMAGE_VERSION`-marker en de startpagina-
woorden 120/192–400 zijn profielgebonden testinterfaces, geen OS-ABI.
### Aanvulling ADR-043 — monotone herstelstatus en doorlopende bewaking (29 oktober 2026)

Concrete post-merge tegenvoorbeelden uit #52: een oudere recoveryblob wist
lokale tombstones en iedere reseed maakte een nieuw gezondheidstestvenster.
De herstelwijziging behoudt Health vanaf startup en verenigt tombstones bij
herstel van dezelfde master. Een andere master mag uitsluitend naar een nog
ongebruikte instantie; een gebruikte instantie weigert de wissel zonder
wijziging. Geen wire-format- of cryptoprimitiefwijziging. Na koude herstart
blijft rollbackdetectie afhankelijk van toekomstige vertrouwde persistente
status (B02); een recoveryblob alleen bewijst geen actualiteit.

De broker bevestigt virtio-reset vóór hergebruik; bij ontbrekende reset-ACK
blijft DMA gepind en de broker offline. Eigenaarsverlies stopt het apparaat en sluit de broker af; de DMA-pool
blijft in quarantaine tot reboot. Twee aparte native fixtures bewijzen een echt stille
bron en een normale reset/herstart; de oude `stuck` is uitsluitend een
constante-bytefixture. Geen fysieke entropy- of hardwareherstelclaim.

Eigenaar: hoofdagent; wijzigingen en bewijs in native-recovery-hardening.
Herziening: persistent antirollback-/recoverycontract in B02, of een concreet
tegenvoorbeeld tegen de levende-instantie-invariant. Terugdraaien: revert
herstelcommits en B01 open houden, geen onzeker bewijs accepteren.

## ADR-044 — Native netwerk via één begrensde capabilitybroker

**Datum:**30oktober2026. In uitvoering als deel van de volledige release;
geen N01/NAS/TLSacceptatie. NetworkSource is uitsluitend aan het geconfigureerde
ring3netwerkproces toegekend; apps krijgen geen MMIO/DMA of algemeen netwerkrecht.
Eigen kernel valideert handles, volledige usercopy en lengtes, beheert gepinde
RX/TX-buffers, completion/timeout en reset-ACK/quarantaine. ARP/DNS/TCP en later
TLS/configuratie blijven in userspace. ABI1.5:0x70send/0x71completion/0x72receive;
sluiten/dood beëindigt de bron. Geen automatische pakket-replay. Ondersteund
proefapparaat: modern virtio-net00:07.0, één CPU/QEMU256MiB. Werkelijke native
ARP/UDP/DNS-counters en onafhankelijke8/7testpeer slagen op3879c32, met dezelfde
koude volumerecovery. Volledige app-/bestemmingsbroker blijft verplicht open.

Testfirmware is geen OS-netwerkclient. Een echte OVMF-MLDrepro leidde tot de
EDK2-runtimeopties VirtioNetSupport/IPv4Support/IPv6Support=no en romfile=,
met intacte PCI-NIC en strikte peer. Causale oude-kernelA/Bproef bevestigt dit.
Terugdraaien: netwerkfeature uit de integratiebranch revert, N01 open houden;
geen prebootframe negeren of driver-/rechtencontrole uitschakelen.

## ADR-045 — Eager x87/SSE-context en echte boot-monotone tijd

**Datum:**30oktober2026. Opt-inmechanismen fpu-context en monotonic-clock;
behoud van microkernel/procesisolatie vereist afzonderlijke CPU-status per
proces. Kernel beheert16uitgelijnde512byte FXSAVE-beelden; save bij CPL3-entry
vóórRust, restore pas na alleRust/CR3selectie direct vóórGPRpops/iret. Verse
PID/slot start met gewiste registerpayloads, vaste FCW/MXCSR en lege x87tags;
cleanup wist alles. CPUIDcontrole, EM/TSuit, MP/NEaan, OSFXSR/OSXMMEXCPTaan,
AMDFFXSRuit/teruggelezen; XSAVE/AVXblijven uit. Geen GUI/crypto/netpolicy verhuist
naar de vertrouwde kern. Kernel gebruikt zelf geen FP/SIMD en de disassembly
wordt gecontroleerd. Eén CPU; fysiekeAMDerrata/uitgebreideFPexcepties open.

Sys3 levert actuele100HzPITticks sinds boot, uitsluitend bij zes nulargumenten,
met saturatie. SDK kent ABI1.6; oudere profielen behouden hun eerdere ABI.
Geen wandklok/RTC/hervatcontract of realtimegarantie. De native-fpu-fixture heeft
echte syscall/timerwissels, drie schone levens, een echte unsupported-AVX#UD,
overlevend proces en slotreuse. Een afzonderlijke illegale-MXCSRproef onthult
TCG's ontbrekende#GP; dat falende bewijs blijft bewaard en is niet groen verklaard.
Hostartifacthelpers mogen geen symlink/hardlink buiten de checkout overschrijven.
Rootintegratie wordt apart native herhaald. Terugdraaien: opt-inmechanismen
revert en FP-afhankelijkeproductfuncties zichtbaar open houden.

## ADR-046 — Onderhouden TCP-library in een ring3app

**Datum:**30oktober2026. Ontwerpbesluit, implementatie/nativeacceptatie loopt.
Gebruik smoltcp=0.14.0(0BSD), minimale no_stdEthernet/IPv4/TCPfeatures,
integerReno, vaste1RX/1TX1514byte slots en maximaal twee TCP-sockets met8KiB
perbuffer. Geen std/alloc/libc/TUN, geen pakketparser in de kernel en geen
kunstmatig voortgezette tijd. TXslot blijft bezet tot echte brokercompletion.
Timeout/retransmissie gebruikt Sys3, naast hostlifetimebounds.

De actuele library vereist Rust1.91.1: alleen dit apppackage/alle lokale
Rust-afhankelijkheden worden daarmee gebouwd en als eigen ELF geladen.
Kernel/loader/oudere apps blijven gepind op1.90; geen gemengde rlibs of globale
defaultwissel. Matchende rust-lld en gelockte dependencyketen; buildscripts en
acht registrycrates zijn op herkomst/licentie/buildgedrag gecontroleerd.
0.12 is afgewezen wegens latere TCP-/IPv4-validatiefixes. Bekende upstreamissues
voor groteRXwindows en luisteraars blijven zichtbaar; de begrensdeclientproef
bewijst die algemene server/NASgevallen niet. De vaste proef-ISNseed is alleen
voor testdata; productie-entropy en TLS blijven releaseblokkerend open.
EOF, refused/reset, echt verloren eersteSYN/retransmissie en blackholetimeout
worden tegen een protocolconforme loopbacktegenpartij native getoetst.


## ADR-047 — Begrensde eigen heap en onderhouden gebruikersallocator

**Datum:** 30 oktober 2026. Implementatie in uitvoering; geen volledige geheugen-,
engine- of productacceptatie. Sys4 (opt-in ABI 1.7) groeit uitsluitend de eigen
contigue RW/NX-heap met verse nulgevulde pagina's. Bestaande totale procesquota
blijft 2048 frames inclusief programma en stack. Geen shrink, fysieke mapping,
uitvoerbaar heapgeheugen of delegatie. Fallible batchreservering en volledige
user- én kernelpaginatabelpreflight gaan vóór publicatie; fouten behouden oude
mappings en ruimen reserveringen op. Het eerste profiel blijft één CPU met IF=0.

De aparte userspace allocator gebruikt exact `linked_list_allocator` 0.10.6
(MIT/Apache-2.0, defaults uit), een expliciet exclusief arena en eigen begrensde
administratie. Geen hostfallback of stil automatisch groeien. `OwnedHeapPages`
bewijst geen veilige Rust-aliasing: de initialisatiecontrole blijft expliciet.
Onafhankelijke review, native nulvulling, quotum/OOM, NX, isolatie, hergebruik en
cleanup zijn afzonderlijke gates. De 128 MiB-OOM-proef verandert het bestaande
256 MiB-regressieprofiel of 8 GiB-productdoel niet. Terugdraaien: opt-in features
revert; allocator-/engineafhankelijke productcriteria blijven zichtbaar open.

## ADR-048 — Native display en invoer via minimale transportcapabilities

**Datum:** 30 oktober 2026. Contract goedgekeurd, implementatie in uitvoering.
De bestaande gevalideerde GOP-header/mapping en begrensde pixelhelper worden
losgemaakt van de G0-kernelrenderlus. De afzonderlijke `framebuffer`-feature
bevat alleen hardwaremetadata en supervisor-RW/NX/UC-transport. G0 blijft zijn
aparte IF=0-proef; Materiaal, fonts, documenten en invoerbeleid horen in ring 3.
Geen VRAM-frame wordt als gebruikers-RAM toegewezen of door heapcleanup vrijgegeven.

ABI 1.8 wordt opt-in gereserveerd: DISPLAY=4096 en INPUT=8192 zijn alleen door
het manifest uitgegeven, PID-/generatiegebonden en niet overdraagbaar/afleidbaar.
0x80/0x81 leveren openbare geometrie en een synchrone tileblit van maximaal
32×8 pixels (LE `0x00RRGGBB`); geen fysieke basis/stride of vsyncgarantie.
Alle usercopy, descriptorvelden, pixels en volledige doelrijen moeten vóór de
eerste hardwarewrite geldig zijn. Alleen de vertrouwde compositor krijgt display-
autoriteit; onbetrouwbare apps krijgen gecontroleerde oppervlakken/portalen.

0x82–0x84 vormen een begrensde PS/2-resetstatusmachine en ruwe bronlezing voor
uitsluitend de geïsoleerde invoerdienst. Werkelijke ACK/BAT/identiteit en
Set2→Set1-configuratie gaan vóór Ready; beide kanalen en foutstatus blijven
zichtbaar. Gevalideerde uitvoer gaat vóór FIFO-consumptie. Intrekking, dood,
stale tickets, tijdslimieten en generatie-uitputting mogen geen toegang of vals
Ready opleveren. Dit is geen algemene driver-/DMA-/IOMMU-garantie. Details en
uitvoeringsbewijs volgen in het transportcontract en native QMP-bedieningsproef.
Terugdraaien: opt-in transport revert, G0-regressie behouden en productcriteria open.

## ADR-052 — Afzonderlijke native CRB-apertureproef, geen product-TPM-beleid

**Datum:** 30 oktober 2026. `native-tpm-crb` combineert uitsluitend de bestaande
echte IPC-wachtproef met de bestaande safe TPM-transport-/commandocore.
De kernel bezit één vaste supervisor-RW/NX/UC-pagina op VA `0x3f180000`
voor QEMU locality0 `0xfed40000`; geen willekeurige fysieke mapping, DMA,
nieuwe gebruikersgrant of publieke syscall. De private niet-Copy-adapter
doet begrensde volatile-kopieën, expliciete compiler-/CPU-fences en sticky
quarantaine die geen nieuwe claim toestaat. Alleen de bootstrapfixture
codeert één read-only GetCapability(TPM_PT_FAMILY_INDICATOR); geen crypto,
NV-mutatie, sleutels of opslagpolicy in de kernel.

De bestaande scheduler levert echte PIT-ticks aan iedere begrensde poll.
Geen poll-countklok, drukke verborgen waitlus, automatische replay of
fallback. Readiness, exacte response, bufferwissen, Idle en relinquish-ACK
gaan vóór succes. Een aparte absent-case vereist weigering vóór enige
hardwarewrite. De volledige bestaande CPL3-wacht-/rechten-/cleanupcontrole
blijft gelden. Beide echte VM-runs zijn bij dit ontwerpcheckpoint nog open.

Dit is een eerste MMIO-integratie, geen kernelgrant voor productapps,
TPM-discovery, fysieke ondersteuning, vertrouwde boot, persistente
sleutel-lineage of onafhankelijk rollbackanker. PR #64 houdt zijn eigen
keyring/system-opslagwrites; dit experiment verandert die interface niet.
QEMU v10.2.1 `hw/tpm/tpm_crb.c` is als primaire implementatie gelezen:
https://github.com/qemu/qemu/blob/v10.2.1/hw/tpm/tpm_crb.c
Register/buffer/Idle/locality-completion passen bij het bestaande contract.
OSDev: *Memory mapped registers in C/C++*, W003198, revisie30402
(24 maart2026), is alleen aanvullende uitleg; geen normatieve TCG-controle.
Herzien vóór een productbroker, andere hardware of NV-commits. Rollback:
opt-in features/fixture verwijderen; legacy profielen blijven ongewijzigd.

<a id="adr-053--native-vensterbeleid-en-vervangbare-presentatiediensten"></a>
## ADR-059 — Native vensterbeleid en vervangbare presentatiediensten

**Datum:** 2 november 2026. De nieuwe opt-in venstersessie gebruikt dezelfde
eigen kernel, capability-engine, INP1 en SUR1. Twee geïsoleerde applicaties
krijgen ieder een eigen surface en invoerkanaal; de gebruikersruimtecompositor
bezit de enige DisplaySource. Alleen de bestaande inputbron bezit InputSource.
De compositor ontvangt gedecodeerde input via de inputdienst en beheert globale
focus, vensterpositie en vertrouwde titelbalken. Caller-PID komt uit kernel-IPC,
nooit uit een berichtveld. Geen gedeelde framebuffer of gedeeld appgeheugen.

Een nieuwe no_std grafische bibliotheek gebruikt vooraf gerasterde proportionele
Open Sans-glyphs met vastgelegde Apache-licentie en eigen vectorpictogrammen.
Er komt geen onbetrouwbare fontparser in de kernel. Backing stores zijn privé,
begrensd tot twee vensters en 1.400.000 pixels binnen de bestaande procesquota.
De oude 800×600-diagnostische profielen blijven bestaan; de venstersessie vraagt
expliciet een passende UEFI GOP-modus van 1280×720 of 1920×1080.

ABI 1.11 reserveert syscall 0x53 voor uitsluitend de manifest-supervisor met de
bijbehorende Launch- en overleden Process-capability. De supervisor bepaalt
het herstelbeleid en laat apps eerst hun oude presentatiekanalen sluiten.
De kernel speelt alleen de vaste compositor/inputdienstmanifestgroep opnieuw
af, na een volledige capaciteitscontrole, met verse PIDs/caps/epochs. App-,
document-, volume- en sleuteldienststaat blijven bestaan. Dit is geen algemene
apparaatdelegatie en geen kernel-GUI-beleid. Zie het exacte bouw-/testcontract
in `docs/implementation/NATIVE-WINDOW-SESSION.md`; native acceptatie is bij dit
ontwerpcheckpoint nog open. Geen U01-punten voor alleen deze bouwstap.

<a id="adr-054--exacte-tijdelijke-autoriteitsvergelijking-op-de-syscallgrens"></a>
## ADR-060 — Exacte, tijdelijke autoriteitsvergelijking op de syscallgrens

**Datum:** 2 november 2026. De volledige verse `Stats`- en digestfuncties
blijven de bestaande diagnostische orakels. Op de M4-foutgrens wordt
hun herhaalde hashscan vervangen door een volledige getypeerde kopie van alle elf
autoriteitsvelden en bestaande exacte `Engine::eq`. IPC-wait vergelijkt dit
op ieder resultaat. De afzonderlijke useroutput-, capability-, pointer-,
return-, paginaownership- en W^X-controles blijven intact. De snapshot wordt
nooit teruggezet in de actieve engine en verleent geen rechten.

Eén vooraf gereserveerde private kernelbuffer van461.816bytes voorkomt een
grote stacktemporary of allocator. Het single-CPU/IF0-contract wordt expliciet
gecontroleerd; een niet-blokkerende lease weigert herintreding. Exhaustieve
velddekking laat een toekomstige Engine-uitbreiding compilatie falen totdat de
kopie bewust is bijgewerkt. Vergelijking gebeurt op Rust-velden, niet padding.
Alle1024 extra berichtkopieën worden met getypeerde volatile writes gewist vóór
vrijgave en vóór een eventuele foutassertie; er blijft geen extra IPC-inhoud in
de vrijgegeven buffer staan. Metadata blijft alleen private kerneldata.

Dit is geen mutatiecache en geen uitschakeling van een audit. Native IPC-wait
en gerichte host-/negatieve tests zijn uitgevoerd op de deelbranch; de volledige
samengevoegde M4-, venster- en netwerkgates volgen. Hostmicrobenchmarks geven
geen native latentiewaarborg of U02-punten. SMP of herintredende kernelcallbacks
vragen een nieuw ontwerp. Details en bewijsgrenzen:
`docs/implementation/EXACT-AUTHORITY-SNAPSHOTS.md`.

<a id="adr-055--begrensde-rust-in-gui-processen-via-bestaande-ipc_wait"></a>
## ADR-061 — Begrensde rust in GUI-processen via bestaande IPC_WAIT

**Datum:** 2 november 2026. Materiaal, Rekenmachine, compositor en window-init
parkeren uitsluitend na een echte lege ontvang-/invoercontrole en volledig
afgehandeld eigen werk. De wachtroute gebruikt een bestaand ontvangstrecht en
een checked deadline van één volgende PIT-tick. Wake-bits zijn hints: de
oorspronkelijke receive/PEEK/antwoordvalidatie blijft verplicht. Control blijft
bereikbaar; opslaan, onvolledige frames, backpressure, fouten, herstel en STOP
blokkeren rust. Geen nieuw recht, kernel-ABI of gewijzigde transactiedeadline.

Een leeg pull-inputantwoord is geen toekomstige pushbelofte; daarom is alleen
op inputantwoord wachten onvoldoende. De korte deadline voorkomt dat nieuwe
invoer onbeperkt blijft liggen. De compositor behoudt zijn tweepits-poldrempel;
een harde20ms-/p95-garantie volgt daar niet uit. Zes volledige Windows/WHPX-gates
en TCG-CI slagen; drie15smetingen per arm geven88,26%lagere mediane procesCPU
bij hetzelfde volledige beeld op één pc. Dit is geen energie- of algemene
prestatieclaim. Herzien bij een event-driven inputprotocol, SMP of gewijzigde
herstelstaten; U02 vereist nog klembord en volledige interactiebudgetten.
Contract en brongebonden bewijs: implementation/GUI-IDLE-WAIT.md en
verification/gui-idle-20261102/INTEGRATION.md.

<a id="adr-056--klembord-als-eenmalige-focusgebonden-tekstoverdracht"></a>
## ADR-062 — Klembord als eenmalige, focusgebonden tekstoverdracht

**Datum:** 2 november2026. Het bestaande vertrouwde venstercompositorproces
bemiddelt een vluchtig CLP1-klembord voor expliciet geselecteerde tekst. De
bestaande app-inputendpoints en hun enige InputSession-eigenaar worden
hergebruikt; geen nieuwe kernel-ABI, proces of apparaat-/opslag-/netwerkcap.
Werkelijke PID, inputepoch, aangeboden CtrlC/CtrlV-Presssequence/generation,
actuele focus en eenmalige brokerclaim zijn gezamenlijk vereist. Een
appgeschreven intentlabel verleent niets. Pendingbackpressure bewaart alleen
het ticket; iedere flush toetst opnieuw voordat plaintext wordt opgebouwd.

Na een geldige leveringsresponse past de app tekst pas toe bij de echte
inputACKAcknowledged. Superseded/transportfalen wist de tijdelijke kopie zonder
edit. Reeds geënqueuede tekst kan de broker niet terughalen; intrekken stopt
toekomstige grenspassages en is geen informatieflowgarantie. Copy wist oude
clipboardinhoud al bij de vertrouwde toets, zodat lokale oversize/lege selectie
geen stille oude-tekstfallback geeft. CtrlShiftDelete wist en trekt tickets in;
compositorherstart begint leeg. Tekst staat nooit in logs/Debug of op schijf en
eigen buffers worden met de bestaande gepinde zeroize gewist.

De eerste versie heeft192 UTF-8-bytes zonder afkappen, geen historie, hele-doc-/
antwoordfallback of automatisch uitvoeren. Dit blijft een afgebakende stap:
U02 en bredere capaciteit/formaten/interactiebudgetten blijven open. Een
verloren DeleteRelease kan na een clear conservatief één Delete-cyclus inslikken;
de latch voorkomt dat een oude clear-repeat documenttekst wist. Herzien bij
streaming, meer apps, veilige accountvelden of een echte sessiegrens.
Contract, bron en nog lopende native gate: implementation/NATIVE-CLIPBOARD.md
en verification/native-clipboard-20261102/.


## ADR-MATERIAL-LIBRARY-20261102 — bestaande volumeobjecten native bedienen

De native werkruimte ontsluit bestaande volume-v7-namen, collecties, metadata
en twee historieplaatsen; geen nieuw wireformaat, opslagformaat of kernelrecht.
Hernoemen blijft draft tot expliciet bewaren. Ordenen gebruikt CAS, historie
herstelt als nieuwe revisie. Onzekere publicatie wordt niet herhaald; een
bevestigde restore met mislukte herlezing blokkeert bewaren tot expliciet heropenen.
Een begrensde tekstpreview valideert eerst de volledige historische versie en
UTF-8 in de bestaande stagingbuffer. Labels maken verkorting en locatie duidelijk.
Proportionele tekst deelt metrics/caret/selection/wrap/hittest en bewaart bytes.

Het eigen scalar appspecifieke RX-budget wordt na echte131177-byte-linkermeting
128→144KiB; vaste adressen, vijf segmenten, W^X, R/RW/stack/guard/kernelquota en
capabilities blijven gelijk. Zie `implementation/NATIVE-MATERIAL-LIBRARY.md`.
De native gate gebruikt volledige toegewezen appbeelden en een onafhankelijke
schijfdecoder; koude heropening kopieert uitsluitend een opnieuw gevalideerde
bron-/profielgebonden wegwerprun. U03 blijft open totdat de volledige route en
regressie werkelijk slagen. Geen authenticatie-, encryptie- of hardwareclaim.

<a id="adr-058--positieve-volume-identiteit-vóór-toelating-als-lege-opslag"></a>

## ADR-CI-RUNNERS-20261106 — Linux-CI alleen op de Unraid-runners

Status: runnerkeuze vervangen door ADR-CI-HOSTED-20261108; overige afspraken
over artifacts en verplichte checks blijven gelden. Oorspronkelijk aangenomen
op 6 november 2026 (opdracht van de projecteigenaar, PR #241).

**Besluit:**
- Elke workflowjob draait op `[self-hosted, Linux, X64, unraid, build]`.
- Persoonlijke machines (`MSIMON-wsl`, de Windows-host `MSIMON`) voeren geen PR-code uit.
- GitHub-hosted runners worden vanwege het budget niet zonder expliciete toestemming gebruikt.
- Artifacts zijn kortlevend (3 dagen). Bewijs van record blijft compact in `docs/verification/`.
- De verplichte checks `host-quality` en `qemu-smoke` (ci.yml) krijgen geen padfilter.
  Een overgeslagen workflow zou anders een eeuwig wachtende verplichte check opleveren.
- Niet-verplichte gates slaan PR's over die alleen `docs/**` of `*.md` in de root wijzigen.
- Een push naar main draait alles.

**Reden:**
- Het label `[self-hosted, Linux]` matchte ook een WSL-runner die als de gewone gebruiker
  draaide, met toegang tot diens credentials.
- Zo'n 6 GB aan artifacts liet groene gates rood eindigen op de uploadstap.

**Grenzen:**
- De Unraid-runners en hun containerisolatie, sudo-rechten en mounts zijn hier niet
  beoordeeld.
- De twee Tower-runners delen één fysieke host. Daardoor draaien er maximaal twee zware jobs
  tegelijk, maar CPU, RAM en I/O van die host zijn niet gemeten.

## ADR-CI-HOSTED-20261108 — Standaard GitHub-hosted Ubuntu-runners

Status: aangenomen, 8 november 2026, op expliciete opdracht van de projecteigenaar om de
workflows van self-hosted naar GitHub-hosted runners om te zetten.

**Besluit:** alle 24 jobdefinities in de 21 bestaande workflows gebruiken
`runs-on: ubuntu-24.04`. De component- en toolchainmatrices leveren samen
42 jobs op. Nieuwe of nog open workflowwijzigingen moeten dezelfde runnerkeuze
aanhouden; een oude branch mag de self-hosted selectie niet opnieuw invoeren.
Dit vervangt de runnerkeuze en de eerdere budgetbeperking in
ADR-CI-RUNNERS-20261106. Het vaste Ubuntu-image behoudt de Linux/x64-omgeving
voor de bestaande `pc-q35-8.2`-proeven, ook voor het op Linux gebouwde
Windows-QEMU-testpakket.

De bestaande Rust-pins, testcommando's, time-outs, rechten, verplichte checks,
padfilters en artifactretentie van drie dagen blijven behouden. De QEMU-/
firmwarestappen verwijzen inmiddels naar `.github/actions/setup-qemu` volgens
ADR-CI-PINS-20261108; die gedeelde, gepinde setup vervangt de oorspronkelijke
losse apt-installatie van ontbrekende pakketten. De host-Rust-buildcache
krijgt een nieuwe namespace met `ubuntu-24.04`, zodat oude self-hosted
buildresultaten niet worden hersteld. Cargo-downloadcaches blijven bruikbaar;
native images en acceptatieresultaten worden niet gecachet.

**Kosten en publicatie:** standaard GitHub-hosted runners zijn gratis en
onbeperkt te gebruiken voor openbare repositories, binnen de uitvoerings- en
concurrencylimieten. Voor private repositories geldt het inbegrepen
computebudget en eventuele extra facturering. Grotere runners vallen niet
onder deze keuze. De GitHub-API meldde bij deze wijziging `private: true`;
deze opdracht verandert de zichtbaarheid of betaalinstellingen niet.
Artifact- en cacheopslag hebben afzonderlijke limieten.

**Bewijsgrens en herstel:** de configuratiecontrole staat in
`verification/github-hosted-runners-20261108.json`. Een runnerlabel of geldige
YAML is geen native bootbewijs; daarvoor blijven de bestaande CI-gates op de
nieuwe PR-head nodig. Terugdraaien kan met een gerichte revert van deze
migratie, zonder andere bronwijzigingen of branchregels aan te passen.

**Primaire bronnen, geraadpleegd 8 november 2026:**
- GitHub. (z.d.). *GitHub-hosted runners reference*.
  https://docs.github.com/en/actions/reference/runners/github-hosted-runners
- GitHub. (z.d.). *GitHub Actions billing*.
  https://docs.github.com/en/billing/concepts/product-billing/github-actions
- GitHub. (z.d.). *Actions limits*.
  https://docs.github.com/en/actions/reference/limits


## ADR-CI-PINS-20261108 — Vaste tools voor reguliere hosted CI

Status: aangenomen, 8 november 2026, op vervolgopdracht van de projecteigenaar. Dit besluit
vult ADR-CI-HOSTED-20261108 aan en vervangt daar de losse installatie van
ontbrekende emulatorpakketten. Het betreft 21 reguliere workflows; de
later toegevoegde private publicatie-audit behoudt zijn afzonderlijke scope.

**Besluit:** gedeelde lokale Actions installeren Python 3.12.12 en QEMU/
firmware uit Ubuntu-snapshot 20261108T000000Z. Alle pakketversies zijn exact;
APT verifieert de ondertekende indexen en payloads. Na installatie worden
versies, TCG, machine pc-q35-8.2 en firmwarehashes gecontroleerd. PyYAML,
reeds gebruikt door de host-gate, krijgt een exact gehasht 6.0.3-wheel.
Rust 1.90.0, bestaande 1.91.1-controles en Node 24.22.1 blijven behouden.

**Tests en gevolgen:** 99 bestaande runner-/preflight-/M6c-hosttests komen
in host-quality. De bestaande zes native rookteststarts, harnasvoorwaarden,
jobtijdslimieten, rechten en triggers blijven gelijk. Een aanvullende
compacte QEMU-diagnose-upload is best effort en wijzigt de verplichte
testuitslag niet. Lokale agent-gates slaan uitsluitend GitHub-setup en
runnerdiagnostiek over; strikte preflight en native proeven blijven nodig.

**Bewijsgrens:** de runnerdistributie en tools zijn geselecteerd, maar de
complete VM en vooraf geïnstalleerde gedeelde bibliotheken zijn niet
immutable. Nieuwe-head Rust/OS-gates blijven afzonderlijk bewijs. Een
hosttest of QEMU-toolprobe bewijst geen productacceptatie. Score 41/100
blijft gelijk. Pinupdates vereisen herhaalde controle; herstel gebeurt via
een gerichte revert zonder branchregels te versoepelen.

Bronnen, versies, uitvoering en herzieningsroute: [CI.md](CI.md),
[controleverslag](verification/ci-standard-runners-20261108.json) en
[pakketprovenance](verification/ci-packages-20261108.json).

## ADR-PUBLIC-DEVELOPMENT-20261108 — Openbare ontwikkeling met MPL-2.0 als controlebasis

**Status:** aangenomen als projectrichting op 8 november 2026, op expliciete
opdracht van de projecteigenaar. Licentiecontrole en daadwerkelijke publicatie blijven open.

**Besluit:** Nieuw OS kiest voor openbare ontwikkeling. Het bestaande
[LICENSE-PLAN.md](../LICENSE-PLAN.md) blijft de basis voor de licentiecontrole,
met MPL-2.0 als beoogde licentie voor de eigen OS-code. De opdracht bevestigt
de richting en het controlekader; zij past geen projectlicentie toe en
verklaart de repository niet publicatieklaar.

**Reden:** de ontwikkeling moet inspecteerbaar en geschikt voor samenwerking
zijn. Openbaarheid wordt bewust aanvaard als onderdeel van de projectrichting.
Het plan wordt verder uitgevoerd vanuit de bestaande MPL-keuze, zonder nu
een nieuwe algemene licentievergelijking te beginnen.

**Consequenties:**
- Eigen code, bijdragen, documentatie, assets, modellen en externe onderdelen
  krijgen een expliciete rechten- en licentieafbakening volgens het plan.
  Bestaande externe voorwaarden blijven behouden.
- Het meegebundelde OSDev-/boekmateriaal krijgt een afzonderlijke
  publicatiecontrole volgens het eigen bronregister; het valt niet
  automatisch onder de MPL-richting voor eigen OS-code.
- De vijf bestaande publicatievoorwaarden blijven open tot de daadwerkelijke
  inventaris, compatibiliteit, notices, licentietekst en bronlevering kloppen.
  Openbaarheid betekent niet dat derden de broncode niet kunnen kopiëren.
- De overgang naar GitHub-hosted runners wordt afzonderlijk behandeld in
  [PR #249](https://github.com/simonvanieperen/nieuw-os/pull/249). Dit besluit
  wijzigt geen workflows, repositoryzichtbaarheid of branchregels.
- Bronpublicatie en een volwaardige productrelease hebben verschillende
  acceptatiegrenzen. Er wordt geen OS-criterium afgetekend; de score blijft
  41/100 en R03 blijft open.

**Uitvoeringsstand bij broncontrole:** `main` op `363b486b06023d3d578f8eca2b23ff49045a1757`
was privé; de root bevatte `LICENSE-PLAN.md` en nog geen toegepaste
projectlicentie in `LICENSE`. Deze observatie is geen permanente
repositoryinstelling. De afzonderlijke publicatiecontroles blijven nodig.

**Herzieningscriteria:** heropen de MPL-reikwijdte of een componentkeuze als
rechthebbenden, daadwerkelijke dependencies of distributievoorwaarden dat
vereisen. Een wijziging van de openbare projectrichting vraagt een nieuw
expliciet projectbesluit. Het terugzetten van documentatie trekt reeds
verleende rechten of verspreide kopieën niet automatisch in.

**Bronnen:** de opdracht van de projecteigenaar van 8 november 2026; het bestaande
licentieplan; Mozilla's officiële MPL-2.0-tekst en FAQ, met volledige
bronvermeldingen in [LICENSE-PLAN.md](../LICENSE-PLAN.md).

## ADR-CI-EVIDENCE-20261108 — testuitslag los van artifactopslag

**Status:** besloten en geïmplementeerd op `ci/artifact-storage-20261108`
in PR #251; opname in main volgt de bestaande checks en reviewregels.
**Eigenaar:** Simon van Ieperen.

**Datum:** 8 november 2026. Artifactopslag mag het bewaren
van testbewijs en de workflow niet blijven blokkeren. Op main `363b486b`
slagen de onderzochte native tests, maar upload-artifact weigert nieuwe
archieven wegens quota. Drie dagen retentie was al ingesteld.

**Alternatieven:** alleen retentie verkorten helpt bestaande quota niet
direct en stond al op drie dagen. Gerichte opruiming kan ruimte vrijmaken,
maar is geen structurele bescherming tegen opnieuw volle opslag en vraagt
eerst een gecontroleerde inventaris van te behouden bewijs. Alleen de oude
uploads niet-blokkerend maken laat bij uitval geen compact brongebonden
rapport achter. Externe opslag introduceert een nieuwe opslag- en
toegangsgrens en is voor deze reparatie niet nodig.

**Besluit:** alle vijftien bewijsuploads gebruiken één lokale composite
action. Die schrijft eerst begrensde rapporten, bronbinding en ontbrekende
bestanden naar runlog/summary en een compact receipt. Alleen de uploads
zijn best effort; tests, builds, validators en de collector blijven
verplicht. De oorspronkelijke error/warn-keuze voor geheel ontbrekend
bewijs blijft behouden. Werkelijke upload-outcomes zijn apart zichtbaar.

Normale rapportarchieven zijn begrensd en blijven drie dagen beschikbaar.
Ruwe images, frames en buildmappen worden uitsluitend op handmatig verzoek
als volledig archief geüpload, met één dag retentie. Windows-test-ZIPs
blijven apart beschikbaar na geslaagde controles als hun upload lukt, ook
één dag. Bij een volledig archief is geen tweede ZIP-upload nodig.

Een rapport met hash vervangt geen ruwe bytes voor onafhankelijke
hercontrole. Native acceptatievoorwaarden, downloadstatus en vaste
voortgangscriteria worden niet versoepeld. De runnerkeuze, toolversies,
checknamen, betaalinstellingen en opslagrechten wijzigen door dit besluit
niet. Geen artifacts of workflowruns worden ongericht verwijderd.

**Security-invariant:** de compacte collector leest uitsluitend de expliciete
selectie binnen de checkout, volgt geen symlinks en weigert verborgen paden.
Inhoud blijft bytegelijk in het compacte pakket; workflowcommando's en
controlbytes worden alleen voor de runlogweergave inert gemaakt. Bestands-,
scan- en uitvoerbudgetten zijn begrensd. Het Windows-pakket moet een echt
bestand binnen de checkout zijn; beide package-workflows selecteren die ZIP
ook expliciet of via haar bovenliggende directory in het volledige archief.
De configuratieverifier controleert dit inclusief weggelaten-packagegevallen.
Dit is geen algemene secretscan; full-evidence behoudt de oorspronkelijke
workflowselectie en is geen door de compacte collector gefilterd archief.

**Testbewijs:** de oorspronkelijke verificatie omvat 15 collector-hosttests
en 151 lokale YAML-/shellcontroles, vastgelegd in
`docs/verification/ci-artifact-storage-20261108.json`. De vervolgcontrole
staat in `docs/verification/ci-artifact-storage-20261108/review-followup.json`.
Op mergecheckout `89722c5f` van PR-head `5c291cff` zijn bovendien de echte native-display-input- en
B03-jobs geslaagd terwijl de rapportupload wegens artifactquota faalde;
bronbinding en gelabeld rapportbewijs bleven in de runlog staan. Dit bewijst
de geteste terugval bij uploadfalen; geen geslaagde artifactdownload,
Windows-uitvoering, volledige raw-hercontrole of nieuwe productacceptatie.

**Migratie/rollback:** opname van PR #251 activeert de gedeelde action voor
nieuwe runs. Bestaande artifacts, bewaartermijnen en runlogs worden niet
gewijzigd. Terugdraaien gebeurt met een gerichte revert-PR van deze wijziging;
dan keren de oorspronkelijke blokkerende uploadstappen terug. Behoud
wijzigingen van andere branches en bestaand acceptatiebewijs.

**Herzien wanneer:** een volgende job artifacts moet downloaden, een echte
releaselevering verplicht wordt, of het meetbare rapportbudget structureel
te klein blijkt. Een benodigde artifactoverdracht krijgt dan een eigen
verplichte leveringsgate. Werkwijze, bronnen en rollback: [CI-EVIDENCE.md](CI-EVIDENCE.md).

## ADR-CI-PUBLIC-20261108 — publieke CI zonder eigen runners

**Status:** besloten en geïmplementeerd in bestaande PR #249; nieuwe GitHub-
uitvoering en samenvoeging worden apart vastgelegd. **Eigenaar:** Simon van
Ieperen. **Datum:** 8 november 2026. De GitHub-repository is aantoonbaar public.

**Besluit:** integreer de vaste hosted toolchain uit #249 met de compacte
bewijsroute uit #251. Alle jobselectors, ook de private historische audit en
Copilot-setup, zijn `ubuntu-24.04`. Geen nieuwe self-hosted infrastructuur.
De private audit behoudt zijn voorwaarde en publiceert niet automatisch
historische auditinhoud op de publieke repository.

Verwijder de dubbele `ready_for_review`-trigger; test concept-PRs al op
openen/wijzigen/heropenen. Spiegel bestaande documentfilters naar main-pushes
van specialistische workflows. Behoud de ongefilterde verplichte checks
`host-quality` en `qemu-smoke`, die onafhankelijk parallel kunnen draaien.
Behoud bestaande concurrencygroepen en tijdslimieten. De componentmatrix
verliest twee dubbele entries; hun checks blijven in de gespecialiseerde
display/input- en artifact-store-workflows staan, inclusief scalar build.
Maximaal vier matrixjobs tegelijk. De hostbuildcache krijgt een bron-SHA
plus een compatibele herstelprefix; geen gecachte native acceptatie.

**Alternatieven:** een tweede migratie-PR zou dezelfde configuratie opnieuw
laten botsen. Iedere native gate door een nieuw centraal padclassificatie-
script laten overslaan vergroot de kans op ontbrekende required checks.
Generieke self-hosted labels of betaalde grotere runners passen niet bij
de expliciete keuze voor de standaard publieke GitHub-runners.

**Security-invariant:** PR-code behoudt een read-only token, externe Actions
blijven op commit-SHA gepind en checkout bewaart geen credentials. Geen
`pull_request_target`-uitvoering van PR-code. Alleen artifactuploads mogen
falen zonder de testuitslag te vervangen. De YAML-regressietests bewaken
deze grenzen en het behoud van de vereiste checks. Native orakels, negatieve
gevallen, markers en productcriteria blijven behouden.

**Testbewijs:** lokale hosttests, configuratie-/shellcontrole, bronbehoud en
nieuwe GitHub-runs worden brongebonden vastgelegd in
`verification/ci-public-runners-20261108.json`. Historische testuitslagen
uit #249/#251 blijven bij hun oorspronkelijke bron. Geen productpunt: 41/100.

**Migratie/rollback:** samenvoegen van de bestaande PR #249 activeert de
gecombineerde wijziging; #251 pas als ingehaald behandelen na bewezen
inhoudsopname. Gerichte revert via PR, zonder andere code of opgeslagen
bewijsarchieven te verwijderen. De oude Copilot-WSL-keuze uit #245 is
vervangen door een hosted setup; die branch mag geen self-hosted selector
terugbrengen. **Herzien wanneer:** echte runnerduur, foutlogs of benodigde
checks een aantoonbare uitvoeringsaanpassing vragen.

Bronnen en uitvoeringsroute: [CI.md](CI.md).

## ADR-CODERABBIT-20261108 — gerichte reviews zonder extra automatische afkeuring

**Status:** geïmplementeerd op vervolgbranch `ci/coderabbit-review-policy-20261108`
na merge van PR #249; effectieve CodeRabbit-review en integratie worden
afzonderlijk gecontroleerd.
**Eigenaar:** Simon van Ieperen. **Datum:** 8 november 2026.

**Aanleiding:** CodeRabbit dient ook bij kleine documentatiepunten een
`CHANGES_REQUESTED`-review in. Die kan blijven staan nadat een nieuwe review
als taak succesvol eindigt. De actieve main-regel vereist al geslaagde
`host-quality`/`qemu-smoke`, actuele branches en opgeloste reviewgesprekken;
er is geen vereiste botapproval. Oudere PRs hebben daarnaast conflicten en
CI die nog verwijderde runners selecteert.

**Besluit:** leg de repo-instellingen vast in `.coderabbit.yaml`: `quiet`
plaatst de belangrijkste feedback inline en bundelt andere suggesties;
`auto_approvals.enabled: false` stopt nieuwe automatische afkeuringen en
goedkeuringen. Automatische incrementele reviews blijven aan en
docstringdekking blijft een waarschuwing. Overige configuratie wordt geërfd
om bestaande tools, padinstructies en extra reviewbranches te behouden.

**Alternatieven:** de app verwijderen verliest reviewfeedback. Alle oude
afkeuringen of threads tegelijk sluiten negeert nog onbeoordeelde fouten.
De GitHub-regel voor opgeloste gesprekken uitzetten raakt ook andere
reviewers. Alleen de statuscheck aanpassen helpt niet: `CodeRabbit` is
in de gecontroleerde main-regel geen vereiste check.

**Behoud en verificatie:** geen bronpad uitgesloten, testgate gewijzigd of
branchregel omzeild. Herstel concrete bevindingen vóór het oplossen van
bijbehorende threads. Een dismissal krijgt een inhoudelijke reden en is
geen nieuw codebewijs. Schema-, host- en workflowcontrole staan met hashes
in `verification/coderabbit-unblock-20261108.json`. Schema-validatie bewijst
geen effectieve UI-/globale configuratie; controleer de volgende botreview.
De OS-score blijft 41/100.

**Migratie/rollback:** CodeRabbit gebruikt de YAML uit de beoordeelde
PR-branch. Nieuwe branches krijgen de configuratie na integratie; oudere
branches moeten die samen met de hosted workflows opnemen. Bestaande
afkeuringen blijven afzonderlijk af te handelen. Een gerichte revert herstelt
de eerdere reviewconfiguratie; opgeloste gesprekken blijven traceerbaar.
**Herzien wanneer:** belangrijke fouten alleen nog in samenvattingen
verschijnen, inherited/global settings de bedoeling veranderen, of de
vereiste menselijke reviewafspraken wijzigen. Primaire bronnen: [CI.md](CI.md).

## ADR-BARYON-NAME-20261108 — naamverklaring in de README

**Datum:** 8 november 2026. **Status:** redactioneel besluit voor de
projectpresentatie; geen wijziging van architectuur of productacceptatie.

**Besluit:** gebruik Baryon OS in de README en leg de naam uit vanuit
samengestelde deeltjes met drie valentiequarks. De koppeling met een kleine
eigen kernel, afzonderlijke systeemdiensten en een eigen gebruikersomgeving
is een metafoor voor samenhang. Processorringen behouden hun technische
betekenis. Foutisolatie, herstel, veiligheid en prestaties blijven claims
die passend implementatie- en testbewijs vereisen.

**Bronnen en gevolgen:** de [naamverklaring](../README.md#waarom-de-naam-baryon-os)
verwijst naar CERN/CMS en ESA. De oorspronkelijke repositorynaam `nieuw-os`,
technische identifiers, historische bewijsbestanden en bronverwijzingen
blijven behouden. Deze opdracht registreert geen domein en stelt geen
merkbeschikbaarheid vast. De voortgang blijft 41/100.

**Herzien wanneer:** primaire bronnen, de gekozen projectnaam of de
feitelijke architectuur een inhoudelijke aanpassing vereisen. Een gerichte
revert van de naamgevingspassages herstelt de vorige presentatie zonder
code of technische contracten terug te draaien.


## ADR-LICENSE-APPLICATION-20261109 — Apply MPL-2.0 to explicitly scoped project software

**Date:** 9 November 2026  
**Status:** accepted  
**Authority:** project owner.

**Context:** the repository is public, and the decision of 8 November 2026 established MPL-2.0 as the licensing baseline without yet applying it. This left the legal scope of project-owned software unnecessarily ambiguous.

**Decision:** include the official, unmodified MPL-2.0 text as `LICENSE`. Project-owned software covered by MPL-2.0 is listed explicitly in `MPL-2.0-FILES.txt`. `LICENSING.md` defines the scope; third-party code and assets retain their own terms through `THIRD-PARTY.md` and component-specific notices.

**Consequences:**

- the software license is auditable without implying that every repository file has the same license;
- documentation, historical evidence files, third-party assets, and project marks are not placed under MPL-2.0 solely by the root license;
- new project-owned software files must be deliberately added to the MPL scope and receive an appropriate notice;
- distribution still requires a separate release/SBOM/notice review.

**Review criterion:** changing the primary license, scope, or exclusions requires a separate decision that addresses migration consequences for already covered files and contributions.

## ADR-K06-5F-20261109 — Bound and measure SMP acceptance without widening authority

**Date:** 9 November 2026. **Status:** implementation in progress; native acceptance open.

**Source:** `main` `1db01c4a1fe9e433396c58af4e6b3732034ee2e9`, with the selected kernel, scheduler and test changes from `feat/k06-5e-proc-shootdown` `f141995ce6d5a0032cb90dea1466a24dc71b2f4d`. The latter includes the step-5d-2/FPU checkpoint `4030c4f4`. Unrelated runner migrations are excluded. The normative acceptance requirements remain [L0 step 5f](https://github.com/simonvanieperen/BaryonOS/blob/docs/k06-step-specs/docs/implementation/k06-steps/09-5f-acceptance.md).

**Decision:** complete cross-CPU IPC, address-space-locked usercopy and remote process lifecycle handling before accepting migration or SMP quota evidence. Process state remains protected by KERNEL. Usercopy and mapping changes use the process address-space lock. A remote wake publishes runnable state before sending vector `0xe8`; an interrupt is a hint, and the receiver rechecks state under KERNEL. A process must leave its old CPU and address space before its storage, stack or FPU image can be reassigned or freed.

The existing KERNEL lock-stuck bound is retained at 400 ms. The maximum deferred ticks during one acquisition is bounded at 40, derived from that bound and the existing 10 ms process timer period. Per-CPU maxima and both limits are emitted before CPUs are parked. These are acceptance limits, not measured results; they will not be enlarged in response to a failing test without a separate decision. TSC timing is restricted to the existing calibrated QEMU/TCG model. Host overcommit and ticket-lock contention remain R02 risks.

One shootdown generation has one 100-ms calibrated-TSC deadline, independent
of the number of target CPUs; it is not restarted per acknowledgement. The
ten-minute four-vCPU soak uses one scheduler manifest with four real M4 quota
processes, four real qsim pressure processes, a cross-CPU IPC pair and eight
additional CPU-bound processes. M4 authority is cleaned before the IPC pair
is admitted so unrelated queued messages cannot satisfy or perturb the quota
oracle. The qsim result is accepted only after the M4 proof, while all eighteen
processes remain subject to the final frame/FPU/scheduler cleanup balance.

Each matrix case requires 50 successful repetitions. Any failed repetition leaves that case unaccepted; source or artifact drift terminates the run and preserves the partial report. A one-vCPU run is regression evidence. Host watchdog ratios and guest deadline ratios are separate measurements. Mocked harness tests never count as native runs.

**Open acceptance:** complete L0 §9 workloads, migration with GPR/x87/SSE canaries and `active_cpus` checks, remote kill and exact cleanup, real K07 pressure across four CPUs, the 30-second concurrent workload, the ten-minute soak, negative controls and unchanged regressions. The current host checks and source integration do not accept K06 or extend K07 to SMP. No physical-hardware, speculative-execution, x2APIC or human-audit claim is made.

**Recovery:** retain the original worktrees and source branches. The 5f work is isolated on `codex/k06-5f-acceptance-20261109`; revert its reviewed commits to remove the change. No host disk, firmware, NAS or production key is part of the test configuration.


### K06 step 5f measurement and gate repair — 10 November 2026

Run 42723127310 on `f808d591` is rejected because `nine-capacity` iteration 16 fails the unchanged clock-drift gate (49/50). The old evidence logs only the median and does not distinguish IRQ loss from observation jitter. An idle return can observe more ticks than its requested target. Clock samples therefore retain a coherent IRQ0 tick/TSC snapshot with IF=0 and normalize elapsed TSC against the actual tick delta. Five target segments, their median, the observation deadline and the exact 1% drift limit remain fixed. Failure logs retain calibration plus each tick/TSC delta. Missing IRQs are not synthesized and sustained drift remains a failure.

The final repository gate previously required a missing `tools/qemu-smoke-test.sh`. The executable wrapper now runs the existing M2, K05, single-CPU K07 and RTC regression gates sequentially, with fresh isolated output directories and source checks. Host oracles, Clippy and formatting checks remain required. The workflow preserves nested reports and raw logs. The entire 64-by-50 matrix must restart from zero on the corrected source; no earlier passing repetition is reused.


### K06 step 5f FPU-canary readiness — 10 November 2026

The original run rejects `m3-four-single` iteration 33 with an FPU image mismatch. Its general loop-progress word is incremented before register initialization/checks, and the readiness flag was published before initialization. The fixture stop predicate is strengthened to require completed canary readiness for every live canary; the probe publishes it only after successful GPR/x87/SSE checks. The existing non-canary M4/qsim soak slots remain governed by their own oracles. Final FPU-image comparisons, negative controls, 1000-switch minimum, quotas and deadlines are unchanged. Failure diagnostics retain the slot, expected pattern, actual registers, TOP/tag and saved RIP. The old log lacks those fields and does not establish exact historical causality.

A host artifact/mutation check validates the actual probe RX instruction order and failure branches. CI executes it before relevant M3 cases and soak runs and retains its log. The two distinct unmap executables are excluded from that probe-specific check. A complete 64-by-50 restart is required on the corrected source; the earlier pending source is not reused.


### K06 step 5f bounded observer cost — 10 November 2026

The original eight-CPU single-TCG base case is 15/50 and migration case is 17/50, including ten unchanged 60-second watchdog failures in total. Per-process/register timeout diagnostics are absent, so exact historical timeout causality remains unproven. Source review identifies an unnecessary all-process physical-pattern scan on every M3 entry: sixteen reads each acquire ASPACE and map/unmap a scratch page, adding 32 INVLPG instructions per entry under KERNEL. Only the entering process is now sampled before cleanup; all live processes are scanned after STOP before any release. Observer sampling frequency changes to each process's own entries. The actual ring-3 pattern check remains on every loop. All genuine user-return permission walks and usercopy, shootdown, lock and resource enforcement remain unchanged. No workload, 1000-switch minimum, watchdog, quota or safety limit is reduced or increased. A full restart must establish runtime results; static work reduction is not a measured speedup.

## K06 step 5f — per-AP clock-load observation — 10 November 2026

Restart 42731994218 on `59a30f10` is rejected: `resched-deaf` 46/50, with four median clock-drift failures and exactly 1000 IRQ0 ticks in every sample. This evidence excludes the previously corrected observation-edge overshoot for these failures, without proving why IRQ0 falls behind its reference.

The clock fixture's single shared progress counter forced a locked RMW from every busy AP. Progress observation now uses independent 64-byte-separated single-writer counters and requires nonzero work from every online AP. The APs retain their continuously busy loops and PAUSE; separate synchronization tests, PIT/IRQ0, five samples, exact 1% and deadlines are unchanged. The fixture loses shared observer contention, as explicitly recorded. QEMU's use of host atomics does not establish a global QEMU lock or a causal explanation of the recorded drift. Host/crossbuild and object-code checks are not native acceptance. A new full 64×50 run and executed repository gate remain mandatory.

## K06 step 5f — syscall quantum and audit boundaries — 10 November 2026

L0 section 6.3 makes timer, explicit yield, blocking and reschedule the switch causes. Treating every completed nonblocking syscall as yield needlessly forfeits the caller's remaining quantum. The shared selector may now continue that caller only when its CPU's actual deferred-timer and reschedule generations have not changed. The complete existing return/ownership path remains mandatory; continuation acknowledges no new quantum and preserves every peer's FIFO order. Wake remains FIFO. This is not proof of the IPC deadline.

Engine audits move from repeated observations of unchanged state to every completed capability syscall, the existing bootstrap/exit boundaries and final quiescent STOP. Exact authority snapshots, rollback/output checks, usercopy leases and observable erasure remain unchanged. An independent source review and host models support this candidate; full native acceptance remains mandatory.

A separate manual diagnostic mode shares the full matrix concurrency lock, rejects simultaneous matrix input and retains raw executed trace commands. It neither relaxes native limits nor supplies acceptance repetitions. HTTP 503 snapshot failures receive three APT retries with the same signed snapshot, pins and hashes; persistent setup failure remains failure and supplies no native evidence. ADR-053's counted-IRQ0 rule is unchanged; elapsed-counter compensation requires an explicit normative decision and is not implemented.

## K06 step 5f — ordinary canary CPU hint — 10 November 2026

The complete mode19 canary-check path gains one PAUSE before its normal loop back-edge. Mode20 IPC, forced migration-Yield and legacy mode0 paths bypass it. PAUSE is a guest CPU hint; it neither blocks/yields the process nor increments the actual context-switch evidence. All 1000-switch, FPU, workload, clock and deadline checks remain.

The specified TCG profiles may benefit from finer host-vCPU interleaving: QEMU 10.2.1 exits its CPU loop on PAUSE, while its ordinary round-robin kick timer is 100 ms. Infinite starvation is not established, and the five historical eight-vCPU watchdogs are not causally explained by source inspection alone. Host/crossbuild and real-ELF checks pass; native performance and MTTCG BQL overhead remain to be measured. The full 64×50 acceptance restarts on the new frozen source.
