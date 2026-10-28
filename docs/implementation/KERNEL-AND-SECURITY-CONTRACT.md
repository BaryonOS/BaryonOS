# Uitvoerbaar kernel- en beveiligingscontract M2–M4

Datum: 26 oktober 2026. Status: **gekozen implementatiebasis, nog niet geïmplementeerd of in QEMU getest**. Dit document sluit de architectuurkeuzes voor de eerste isolatie- en capabilityproeven. De builder voert ze uit; opnieuw algemeen onderzoek doen is geen voorwaarde. Herziening vraagt een concrete tegenvoorbeeldtest, incompatibiliteit of gemeten probleem, plus een besluit met migratiepad.

Dit is een eigen Rust-microkernel. seL4 dient als openbare kennisbron voor capabilities, niet als onderliggende kernel, overgenomen ABI of veiligheidsbewijs. De bron beschrijft capabilities als rechten op kernelobjecten; de precieze tabellen, grenzen en protocollen hieronder zijn **eigen projectkeuzes** [K3]. Ook de seL4-bewijzen hebben expliciete aannames en gelden uitsluitend voor de daar beschreven configuraties [K4].

## 1. Vaste eerste configuratie

| Onderdeel | Besluit |
|---|---|
| Machine | Het M1-profiel: QEMU q35, TCG, x86_64, één CPU, 256 MiB RAM, geen netwerk of hostschijf |
| Kernel | Rust `no_std`, `panic=abort`, `x86_64-unknown-none`, `code-model=small`; minimale assembly |
| Bootcompatibiliteit | Bestaande M1 `BootInfo` v1.1, 128 bytes; fysieke pointers; kernel VA=PA=`0x02000000`, gereserveerd bereik 32–48 MiB |
| Paginaformaat | Eigen vierlaags paging, uitsluitend 4 KiB-pagina's; geen huge pages, PCID, demand paging of userspace-MMIO |
| Processen | Maximaal 16 processen; precies één thread per adresruimte; geen gedeeld gebruikersgeheugen |
| Executables | Eigen kleine statische ELF64-testprogramma's, System V-aanroepconventie, geen hostlibc, TLS of dynamische linker |
| Kernelobjecten M4 | Alleen endpoints en capabilitynodes; geen algemene object-, frame- of procesaanmaak door apps |
| Aanmaak | Een begrensd, ingebouwd testmanifest maakt processen, geheugen en endpoints; later vervangt een afzonderlijke beheerservice dit |
| API | Eigen ABI versie 1, registergebaseerde syscalls via `int 0x80`; begrensde, niet-blokkerende IPC |

De Rust-targetdocumentatie bevestigt dat deze target geen `std`/standaardallocator heeft, standaard geen vector- of floatingpointregisters gebruikt en standaard het `kernel`-codemodel kiest. Daarom is `small` hier een expliciete override voor de lage adressen [K2]. Kernel en eerste userprogramma's gebruiken dezelfde scalar-codegeneratie; extra target-features zijn niet toegestaan in deze fase.

M2 kopieert en valideert BootInfo en de complete UEFI-geheugenkaart vóór hergebruik van enig geheugen. Gebruik de descriptorstride uit BootInfo, niet `size_of` van een zelfbedachte UEFI-struct. Alle optellingen, aantallen en adresbereiken zijn gecontroleerd op overflow. Versie, flags en reserveringen volgen [BOOT-AND-TOOLCHAIN.md](BOOT-AND-TOOLCHAIN.md); verander de boot-ABI hier niet.

## 2. Geheugen, eigendom en exceptions

### Adresruimte

Alle bereiken zijn halfopen: het eindadres hoort er niet bij. Elke adresruimte heeft een eigen PML4 en PDPT. Onder PML4[0] verwijst PDPT[0] naar gedeelde supervisor-tabellen voor het gebied onder 1 GiB; PDPT[1] naar eigen usertabellen voor 1–2 GiB. Deel nooit een tabel met processpecifieke userentries tussen processen. De kernelmappings zijn identiek en uitsluitend supervisor; gebruikersmappings zijn per proces verschillend.

| Virtueel bereik | Gebruik en toegangsbeleid |
|---|---|
| `0x00000000–0x00001000` | Ongemapt; null-page |
| `0x00001000–0x40000000` | Supervisor-only identitygebied; map uitsluitend werkelijk benodigde RAM-pagina's en expliciete bootreserveringen, nooit automatisch gaten of apparaten |
| `0x02000000–0x03000000` | M1-kernelreservering binnen dit supervisorgebied; map imagepagina's volgens ELF-segmentrechten |
| `0x40000000–0x60000000` | User-ELF-segmenten; eerste text op `0x40000000`; alleen werkelijk geladen segmenten aanwezig |
| `0x60000000–0x60010000` | Eerste vaste userheap, 64 KiB, read/write, NX |
| `0x7ffee000–0x7ffef000` | Ongemapte onderste stackguard |
| `0x7ffef000–0x7ffff000` | Userstack, 64 KiB, read/write, NX |
| `0x7ffff000–0x80000000` | Ongemapte bovenste stackguard |
| Overig | Ongemapt voor userspace |

Een user-ELF mag alleen niet-overlappende `PT_LOAD`-segmenten binnen het ELF-bereik leveren. Weiger writable+executable segmenten, ongelijke pagina-offsets van file/VA, overflow, ontbrekende executable entry, segmentoverlap op paginaniveau en onbekende verplichte features. Nul het volledige toegewezen geheugen vóór het kopiëren; `p_memsz - p_filesz` blijft nul. Start `RIP=e_entry`, `RSP=0x7ffff000-8`, met nul op de dummy returnpositie en overige algemene registers nul. `_start` retourneert niet.

Intel beschrijft dat U/S-toestemming door de pagingniveaus heen geldt en dat CR0.WP ook supervisorwrites aan schrijfrechten bindt; NX voorkomt uitvoering op daarvoor gemarkeerde pagina's. Interrupt-/exceptiongates en privileges vormen de andere helft van de grens [K1, hoofdstukken 4–6]. Onze gekozen configuratie: NX verplicht, `EFER.NXE=1`, `CR0.WP=1`; user-PTE's hebben U/S=1 op alle betrokken niveaus. Kernel-leaves blijven U/S=0, ook wanneer zij een hogere tabel met userpagina's delen.

Text is RO/X, rodata RO/NX, data/heap/stacks RW/NX. **Geen writable alias van een executable fysiek frame:** ook de supervisor-identityalias van usertext is RO/NX. Frameclassificatie en mappingwijzigingen controleren alle aliases. Een executable frame gaat pas terug naar de allocator nadat alle mappings verwijderd en TLB-invalidaties voltooid zijn. Aanvankelijk geen JIT of runtime-permissiewissel door apps.

### Framebeheer

Elk 4 KiB-frame heeft precies één eigenaar/status: `Reserved`, `Free`, `Kernel`, `PageTable(asid)` of `User(pid, kind)`. Geen gedeelde userframes in M3/M4. Een vaste bitmap plus metadata-array volstaat; grote/groeiende allocaties zijn niet nodig aan de syscallgrens.

1. Neem aanvankelijk alleen `EfiConventionalMemory` als kandidaat voor `Free`, zonder runtime-, nonvolatile- of bijzondere attributen; trek alle bootreserveringen, kernel, kaartbuffer, BootInfo, loader en stack expliciet af. Overlap of tegenstrijdige descriptors is een bootfout.
2. Bouw eigen GDT, TSS, IDT en pagetables, inclusief huidige code/stack/kaart, en wissel naar eigen CR3 voordat interrupts of usermode ingeschakeld worden. Firmwarehandlers en -services zijn geen herstelpad.
3. M2–M4 reclaimen geen `Loader*`, `BootServices*`, ACPI- of runtimegebieden. Dat kost tijdelijk RAM en vermijdt een impliciet levensduurcontract; later reclamation krijgt aparte tests.
4. Nul een frame vóór toekenning aan een ander proces en vóór terugkeer naar `Free`. Fysieke adressen komen nooit als onbeperkte apprechten naar buiten.
5. CR4.PCIDE en PGE blijven uit. Een proceswissel laadt CR3 opnieuw; iedere mappingwijziging krijgt de juiste lokale invalidatie vóór hergebruik. Geen remote-TLB-protocol zolang er één CPU is.

### CPU-context en foutbeleid

GDT-selectors: kernelcode `0x08`, kerneldata `0x10`, userdata `0x1b`, usercode `0x23`, 64-bit TSS `0x28`. TSS.RSP0 wijst bij elke switch naar een eigen 16 KiB-kernelstack met ongemapte guards; double fault en NMI hebben afzonderlijke IST-stacks. Sla alle algemene registers op. DS/ES/FS/GS worden gecontroleerd geïnitialiseerd; FS/GS-bases zijn nul; userinstelling daarvan en FSGSBASE zijn uit. Debugregisters worden niet gedeeld.

Eerste usermode ondersteunt geen x87/MMX/SIMD-registertoestand: `CR0.EM=1`, `TS=1`, `MP=1`; `CR4.OSFXSR=0`, `OSXSAVE=0`. Een poging die deze registertoestand gebruikt eindigt in een userfault; de handler schakelt de functie niet alsnog in. Scalar instructies zoals fences zijn hiervan niet automatisch uitgesloten. Dit is een tijdelijke uitvoeringsprofielkeuze, geen beperking van de productambitie. SIMD/AI komt later met volledige contextopslag en afzonderlijke lektests [K1, hoofdstuk 13].

Normale interruptgates zijn DPL0; alleen `0x80` is DPL3. Gebruik `iretq`, geen `sysret`. Bij terugkeer: gecontroleerde userselectors, user-RIP in executable pagina, user-RSP in toegestane stack, gereserveerde RFLAGS-bits correct; behoud alleen rekenflags CF/PF/AF/ZF/SF/OF en zet vaste bit1+IF. IOPL/NT/VM/DF/TF/AC zijn nul. De entry-stub voert `cld` uit vóór Rust. NMI voert geen scheduler-, allocator- of capabilitymutaties uit.

Een userfault (`#PF`, `#GP`, `#UD`, `#NM`, enz.) beëindigt dat proces, noteert begrensde lokale metadata en laat andere processen verdergaan. Een kernelfault of double fault stopt gecontroleerd; probeer geen mogelijk beschadigde kernel voort te zetten. Faultlogs bevatten geen paginainhoud of secrets. Bij procesexit: markeer `Dying`, stop scheduling, sluit eigen endpoints, ruim caprechten op, wissel naar een veilige adresruimte/stack en geef daarna frames/tables vrij. Nooit de actieve stack of actieve CR3-tabellen vrijgeven.

## 3. Scheduler met aantoonbare voortgang

Gebruik het legacy PIC/PIT-pad van dit QEMU-profiel: remap IRQ's naar `0x20/0x28`, maskeer alles behalve PIT IRQ0, PIT channel 0 periodiek met divisor 11932 (ongeveer 100 Hz), geen gelijktijdige APIC-timerroute. Initialisatie en EOI-code blijven in de kleine arch-laag; fysieke hardwareondersteuning volgt hier niet uit.

Eén FIFO-round-robinqueue, gelijke prioriteit, quantum één timertick. Statussen: `Ready`, `Running`, `Dying`, `Dead`. `yield` plaatst het proces achteraan. Lege runqueue: `sti; hlt`-idle. Aan het einde van een IRQ of syscall vindt de eventuele contextwissel plaats, nadat kernelmutaties afgerond zijn. Kernelwerk tijdens M3/M4 is niet preëmptief; interrupts zijn gemaskeerd tijdens korte, begrensde kritieke secties. Geen wachten op I/O, lockspin, rekursieve foutafhandeling of onbeperkte allocatie in een syscall.

**Fairnesscontract:** bij maximaal 16 permanent runnable processen bereikt ieder proces binnen 16 schedulerselecties de CPU, mits de begrensde kerneloperaties eindigen. Dit is geen realtimegarantie in milliseconden. Meet maximale syscallduur en vertraagde ticks apart; tel timerverlies niet weg als “snelle kernel”. Geen gebruikersprioriteiten of onbeperkte busy-wait binnen de kernel.

## 4. Syscall-ABI v1 en veilige kopieën

`RAX`=nummer; argumenten in `RDI, RSI, RDX, R10, R8, R9`. Bij terugkeer: `RAX`=0 of negatieve foutcode, `RDX`=resultaat of nul bij fout. Overige algemene registers blijven behouden; RFLAGS volgt het bovenstaande masker. Niet-gebruikte argumenten moeten nul zijn. Alle ABI-velden zijn little-endian, vaste integerbreedtes; geen Rust-`usize`, references, `bool` of enums in onbetrouwbare bytes.

| Nr. | Naam / argumenten | Resultaat bij succes |
|---|---|---|
| `0x00` | `abi_info()` | RDX=`(1 << 32)` voor major 1/minor 0 |
| `0x01` | `yield()` | RDX=0 |
| `0x02` | `exit(code)` | Keert niet terug; code is lokale procesmetadata |
| `0x10` | `cap_close(handle)` | Sluit lokale slot, ook als object gesloten/node revoked is |
| `0x11` | `cap_derive(handle, rights)` | Nieuwe lokale handle met verminderde rechten |
| `0x12` | `cap_revoke(handle)` | Intrekking van alle afstammelingen; eigen node blijft geldig |
| `0x20` | `ipc_send(endpoint, message_ptr, size)` | Eén bericht gequeued, RDX=0 |
| `0x21` | `ipc_recv(endpoint, message_ptr, size)` | Eén bericht atomisch ontvangen, RDX=payloadlengte |

M3 implementeert alleen `0x00–0x02`. M4 voegt de overige toe. Onbekende syscall geeft `ENOSYS`. Er is geen userspace debugpoort, algemene kernelpointer, arbitraire fysieke mapping of globale objectopzoekfunctie.

**Copy-regel:** behandel een userpointer eerst als `u64`. Controleer canoniek adres, userbereik, `checked_add(len)`, maximumlengte, aanwezigheid/eigendom van iedere geraakte pagina en effectieve U/S- en R/W-rechten. Uitvoer vereist writable pagina's; een alleen-lezen input mag read-only zijn. Maak pas daarna via een kleine gedocumenteerde unsafe-routine een bytekopie naar een volledig geïnitialiseerde kernelbuffer. Geen `&UserStruct`, geleende userslice of ongevalideerde enum construeren. Rust maakt ook binnen `unsafe` misalignments, ongeldige waarden en aliasingschendingen niet geldig [K5].

De kernel leest invoer precies eenmaal, valideert de kopie en bewaart nooit de oorspronkelijke pointer in een queue. In dit profiel kunnen dezelfde mapping en bytes tijdens kopiëren niet door een tweede userthread, gedeeld frame, DMA of user-unmap veranderen: die mogelijkheden bestaan niet, en er vindt geen processwitch in de kopie plaats. Bij toekomstige threads/SMP/shared memory moet dit contract eerst vervangen worden door vastgezette mappings en synchronisatie; “pointer vooraf gecontroleerd” alleen voldoet dan niet.

Voor IPC is `size` exact 320; anders `EMSGSIZE` vóór kopiëren. Kopieer bij ontvangst eerst naar een kernelbuffer, valideer alle outputpagina's, reserveer alle handles en commit dan in één niet-onderbreekbare operatie. Een mislukte ontvangst verandert userbuffer, queue en captable niet. Alle niet-gebruikte outputbytes zijn nul. Het vaste 320-byte formaat voorkomt lengtes die tijdens een tweede userread veranderen.

| Code | Betekenis |
|---|---|
| `-1 EINVAL` | Onbekende flags/rechten, niet-nul reserved, ongeldige combinatie |
| `-2 ENOSYS` | Syscall onbekend of niet in deze mijlpaal |
| `-3 EFAULT` | Userbereik/pagina/effectieve rechten ongeldig |
| `-4 EBADHANDLE` | Slot buiten tabel, leeg of generation mismatch |
| `-5 EACCESS` | Benodigd recht ontbreekt of delegatie probeert rechten te vergroten |
| `-6 ETYPE` | Verkeerd kernelobjecttype |
| `-7 EQUOTA` | Proces- of globale resourcegrens bereikt |
| `-8 EAGAIN` | Endpointqueue vol bij send of leeg bij receive |
| `-9 ECLOSED` | Endpoint is door eigenaar gesloten/gestopt |
| `-10 EREVOKED` | Bestaande lokale slot verwijst naar ingetrokken node |
| `-11 EMSGSIZE` | Verkeerde ABI-size, payload >256 of cap_count >2 |

Valideervolgorde: syscall/ongebruikte args → ABI-size → endpointhandle/generation/node/object/recht → userbuffer → berichtvelden → resources → commit. Returnwaarden lekken geen objectgegevens vóór de rechtencontrole. Een fout verandert geen rechten of resource-eigendom.

## 5. Capability- en IPC-model

### Handles en rechten

Een handle is `u64 = (generation:u32 << 32) | slot:u32`. Iedere procestabel heeft 256 slots; slot 0 is altijd ongeldig. Generation start op 1, stijgt bij sluiten en wordt nooit 0. Een slot met maximale generation wordt bij sluiten permanent gepensioneerd. Nooit wraparound/ABA. Lookup gebruikt **alleen de captable van de aanroepende procescontext**, nooit een door userspace opgegeven PID.

Een handle is geen cryptografisch geheim. Een getal uit proces B geeft A geen toegang tot B's tabel. Valt dat getal toevallig samen met een geldige eigen slot van A, dan krijgt A uitsluitend het recht dat A al bezat. De autoriteit zit in kernelmetadata, niet in moeilijk te raden cijfers; quantumcomputers veranderen dat lookupcontract niet.

Rechtenbits voor `Endpoint`: `SEND=1`, `RECV=2`, `DERIVE=4`, `TRANSFER=8`, `REVOKE=16`; alle andere bits ongeldig. `derive` vereist DERIVE, en `new_rights & !old_rights == 0`. Masker 0 is toegestaan maar geeft geen operatierecht. RECV blijft bij de endpoint-eigenaar: overdracht met RECV wordt geweigerd; een eigen lokale afleiding mag RECV houden. SEND zegt niets over bevoegdheid om bestanden, netwerk of apparaten te gebruiken.

Elke capslot verwijst naar een kernelnode met object-ID/generation, rechten, oudernode-ID/generation, issuer-PID en status. Nodes vormen een boom met maximale diepte 8, waarbij een root diepte 0 heeft; maximaal 1024 nodes totaal. Object- en node-ID's gebruiken intern ook generationtellers zonder wraparound. Parentnodes blijven bestaan zolang kinderen bestaan, ook na sluiten van hun laatste lokale handle. Refcounts omvatten lokale slots, kinderen en queueverwijzingen; reclaim alleen bij nul.

`cap_revoke` vereist REVOKE en markeert alle afstammelingen revoked, ook elders in andere processen of nog in een queue. Scan maximaal 1024 nodes met per node maximaal 8 ouderstappen, en maximaal 1024 queueplaatsen; geen afhankelijkheid van tabelvolgorde of onbegrensde boomrecursie. Revoked slots blijven herkenbaar tot `cap_close`, maar geven geen toegang. `cap_close` alleen trekt afstammelingen niet in. Procesexit trekt alle nodes in zijn slots én door dat proces uitgegeven nodes/afstammelingen in; issuer-PID blijft daarvoor bewaard. Er ontstaan geen blijvende rechten uit een gestorven delegator in dit eerste profiel.

### Berichtformaat en transactie

Dit is een formaatbeschrijving, geen reeds aanwezige implementatie:

```rust
#[repr(C)]
struct IpcCapV1 { handle: u64, rights: u64 }
#[repr(C)]
struct IpcMessageV1 {
    sender_pid: u64,  // offset 0; send moet 0 zijn; receive vult kernel in
    tag: u64,         // offset 8; vrij appgegeven, geen autorisatie
    len: u32,         // offset 16; 0..=256
    cap_count: u32,   // offset 20; 0..=2
    reserved: u64,    // offset 24; 0
    caps: [IpcCapV1; 2], // offset 32; 32 bytes
    data: [u8; 256],  // offset 64; totaal 320 bytes, alignment 8
}
```

Niet-gebruikte capdescriptors en bytes na `len` moeten bij send nul zijn; daardoor ontstaat een canonieke, controleerbare representatie. `sender_pid` komt uit de kernelcontext en is nooit een bewijs van inhoudelijke bevoegdheid voor een dienst. PID's zijn monotone `u64` zonder hergebruik/wrap in dezelfde boot.

Send maakt een snapshot; valideert endpoint-SEND, alle over te dragen caps, TRANSFER en rechtenvermindering; reserveert daarna alle node- en queueplaatsen. Overdracht is **kopie via een nieuwe afgeleide node**, geen move. De bronhandle blijft geldig. Een mislukte send laat niets achter. Per endpoint maximaal 16 berichten, FIFO, geen wachtende kernelthreads.

Receive vereist RECV en de eigenaar-PID. Reserveer alle benodigde lokale slots vóór aflevering. Vervang bericht-handles door handles uit de ontvangers eigen tabel; rapporteer de werkelijke gedelegeerde rechten. Heeft ontvanger geen slots, geef EQUOTA en laat het bericht staan. Voor een ingetrokken cap in een nog niet afgeleverd bericht wordt het **hele bericht geannuleerd** en de reservering teruggegeven; nooit gedeeltelijk payload/rechten afleveren. Revocation ruimt zulke berichten op binnen de vaste globale scanlimieten. Reeds afgeleverde bytes kunnen niet worden teruggehaald.

Een endpoint hoort bij precies één eigenaar. Sluiten of intrekken van diens laatste actieve RECV-handle, of zijn exit, sluit het endpoint; pending berichten worden geannuleerd. Alle overige geldige caphandles naar het object geven ECLOSED totdat gesloten; een reeds revoked node behoudt volgens de valideervolgorde EREVOKED. Objectmetadata blijft als tombstone bestaan zolang verwijzingen bestaan, zodat hergebruik geen ander endpoint bereikbaar maakt.

### Vaste quotas

| Resource | Grens M3/M4 |
|---|---|
| Processen / threads | 16 / één per proces |
| Userframes per proces | 2048 = 8 MiB, inclusief image/heap/stack |
| User virtual mappings | Maximaal 2048; geen automatische uitbreiding |
| Capslots | 256 per proces, waarvan slot 0 niet bruikbaar |
| Capnodes / boomdiepte | 1024 globaal / 8 |
| Endpoints | 64 globaal, maximaal 4 eigendom per proces |
| Queue | 16 berichten per endpoint; vaste 320-byte berichtstruct plus begrensde kernelmetadata |
| IPC-transfer | Maximaal 2 caps per bericht |

Reserveer tabellen/queueplaatsen vooraf; geen allocator op basis van een userlengte. Een manifest dat grenzen overschrijdt wordt vóór processtart geweigerd. Rekenkundige quota-overschrijding en echte allocatoruitputting hebben hetzelfde gecontroleerde foutpad; nooit panic als normale resourceafhandeling. Sluiten/exit geeft capaciteit terug, behalve gepensioneerde generation-slots.

## 6. Acceptatiegevallen die de builder uitvoert

Maak pure hosttests voor intervalrekenen, frameboekhouding, captransities en IPC-transacties. Maak daarnaast QEMU-tests die de echte privilegeovergangen uitvoeren. De volgende ID's zijn opdrachten, **geen geslaagde testresultaten**. De kernelharness schrijft alleen `KS-xx:PASS` nadat de beschreven toestand is vastgesteld; userprogramma's kunnen geen succesmarker rechtstreeks afdrukken. Bewaar echte log, commit, profiel, toolversies, timeout en exitstatus volgens het bootcontract.

| ID | Uitvoerbare opzet | Vereiste assertie |
|---|---|---|
| KS-01, M2 | Parse kaart met overlappende descriptor, overflow in pages×4096 en gereserveerd frame in ConventionalMemory | Eerste twee geweigerd; reservering nooit gealloceerd; alle succesvolle allocaties uniek |
| KS-02, M2 | Schrijf patroon in frame, free, alloceer aan andere eigenaar; herhaal tot quota | Nieuw userframe volledig nul; geen dubbele eigenaar; uitputting gecontroleerd |
| KS-03, M2 | Loop door alle pagetables plus fysieke aliasindex | Geen userpad naar kernelpagina; geen writable alias van executable frame; NX op heap/stack |
| KS-04, M3 | A/B krijgen dezelfde user-VA met verschillende fysieke frames en verschillende patronen; switch herhaaldelijk | A ziet alleen A, B alleen B; PTE-frameverzamelingen disjunct; geen stale TLB |
| KS-05, M3 | A leest/schrijft B's fysieke frame via het supervisor-identityadres en probeert kerneltext te schrijven | A krijgt userfault; B's bytes onveranderd; B blijft tellen |
| KS-06, M3 | A springt naar stack/heap, raakt stackguard, voert `cli`, x87/XMM/AVX-state-instructies uit in aparte boots | Elk verboden geval stopt alleen A; kernel blijft bestuurbaar; geen FPU-state toegankelijk |
| KS-07, M3 | 16 workers draaien zonder yield; meet per worker een privé voortgangsteller vanuit testharness | Iedereen geselecteerd in iedere ronde van 16 selecties; geen gedeeld userframe als meetshortcut |
| KS-08, M3 | A crasht/exit tijdens belasting; controleer tabellen vóór/na en laat B doorlopen | Geen actieve-stack-free; frames/caps gereclaimed volgens contract; B maakt voortgang |
| KS-09, M4 | IPC-pointer null, supervisoradres, noncanonical, laatste pagina+overflow, cross-page met tweede pagina afwezig/RO | EFAULT of vooraf EMSGSIZE; geen kernelcrash, gedeeltelijke uitvoer of queueverandering |
| KS-10, M4 | Fuzz len=`257`, cap_count=`3`, size=`0/319/321/u64::MAX`, reservedbits en misaligned geldige buffer | Vastgelegde foutcodes; misalignment van bytebuffer op zichzelf toegestaan; geen UB door structcast |
| KS-11, M4 | Testhook verandert originele inputlengte/payload ná snapshot; probeer gedeelde mapping/tweede thread/user-unmap aan te vragen | Operatie gebruikt alleen snapshot; onbeschikbare API ENOSYS; bouw-invariant één thread blijft waar |
| KS-12, M4 | Kopieer B's numerieke handle naar A; varieer slot/gen; sluit en hergebruik eigen slot | Geen toegang tot B; stale handle EBADHANDLE; collision levert hoogstens eigen bestaande autoriteit |
| KS-13, M4 | DERIVE/SEND zonder benodigd recht; derive met extra bit; transfer zonder TRANSFER of met RECV | EACCESS/EINVAL volgens volgorde; geen node, handle of bericht aangemaakt |
| KS-14, M4 | A→B→C-delegatie; revoke bij A vóór en na aflevering; houd oude handles vast | Nog geen aflevering: hele bericht geannuleerd; afgeleverd: cap EREVOKED, reeds gekopieerde data blijven bestaan |
| KS-15, M4 | Vul 16 queueplaatsen; vul ontvangers 255 slots; forceer capnodegrens/diepte 8; simuleer maximale generation in hostmodel | EAGAIN/EQUOTA zonder gedeeltelijke transacties; capaciteit keert terug na close; generation wrap onmogelijk |
| KS-16, M4 | Eigenaar sluit laatste RECV; sender sterft met queued delegated caps; ontvanger crasht | ECLOSED of annulering volgens lifecycle; geen verweesde rechten, refcount-underflow of hergebruikte-objecttoegang |

KS-11 test de concrete single-thread-aanname en snapshotsemantiek, **geen echte SMP-racebestendigheid**. De latere introductie van concurrerende threads vereist een nieuwe testcase waarin unmap/rewrite werkelijk parallel gebeurt. Hetzelfde geldt voor toekomstige DMA.

## 7. Wat iedere mijlpaal wel en niet levert

| Beveiligingscontrole | M1 | M2 | M3 | M4 / vervolg |
|---|---|---|---|---|
| Eigen MMU-/framebeheer | Ontbreekt; firmwarepaging geërfd | Implementeren en KS-01–03 bewijzen | Gebruikt voor isolatie | Behouden |
| Afzonderlijke userprocessen | Ontbreekt | Ontbreekt | KS-04–08 | Behouden |
| Gecontroleerde syscallgrens | Ontbreekt | Ontbreekt | ABI-info/yield/exit | Copyregels en IPC-negatieve tests |
| Caprechten/delegatie/revocatie | Ontbreekt | Ontbreekt | Ontbreekt | KS-09–16 |
| CPU-preëmptie en quotas | Ontbreekt | Alleen vaste kernelresources | Scheduler/userframes | Caps/queues begrensd |
| W^X | Geen claim | Kernelmappingcontrole | Ook userframes/aliases | Behouden |
| ASLR, SMEP/SMAP, exploitmitigaties | Niet aanwezig | Niet aanwezig | Niet aanwezig in dit profiel | Afzonderlijke uitbreiding; vaste adressen blijven zichtbaar |
| DMA/IOMMU, SMP, sidechannelbescherming | Niet aanwezig | Niet aanwezig | Niet aanwezig | Niet bewezen door deze tests |
| Secure Boot, ondertekende updates, schijfcrypto/PQC | Niet aanwezig | Niet aanwezig | Niet aanwezig | M6/M8-contracten en quantumonderzoek |
| NAS en lokale AI | Niet aanwezig | Niet aanwezig | Niet aanwezig | Dienstintegratie na relevante drivers/netwerk; privacygates blijven bindend |

“Niet aanwezig” betekent dat een M1-boot of M4-test geen dergelijke productclaim rechtvaardigt. De gekozen bescherming richt zich eerst op directe CPU-geheugentoegang en expliciete autoriteit. Speculatieve/timingkanalen, firmwarecompromis en fysieke aanvallen zijn hiermee niet opgelost.

## 8. Aansluiting op lokaal privégebruik

De capabilitylaag is het afdwingmechanisme waarop de latere services bouwen. Zij beslist niet zelf of een IP-adres de eigen NAS is. Maak daarvoor aparte objecten met concrete scope: gekozen bestand, benoemde NAS-dataset, goedgekeurde LAN-context, afzonderlijk internetrecht en een lokale AI-taak. Een algemeen endpoint naar een machtige helper mag deze grenzen niet omzeilen.

De NAS-dienst krijgt uitsluitend rechten op gekoppelde datasets en een geauthenticeerd kanaal naar de geselecteerde NAS-identiteit; LAN standaard, internet afzonderlijk expliciet toegestaan (privacycontract revisie 3). De transportdienst ziet alleen ciphertext; de client beheert de inhoudssleutels. Een NAS-admincapability mag ruimte blokkeren en verwijderen zonder gebruikersconsent, maar verleent geen decryptie- of documentleesrecht. Intrekking moet lopende uploads en herhaalde oude verzoeken bij de commitgrens stoppen. De inferentiedienst krijgt taakcontext en CPU/GPU/NPU-resources, **geen netwerkendpoint**. De modeldownloader krijgt modelopslag plus downloadrecht, geen documenten, prompts, NAS-credentials of inferentiegeschiedenis. Beperk ook delegatie en helperroutes: ontbreken van een socketrecht alleen is onvoldoende als IPC alsnog een machtige uploadhelper bereikbaar maakt.

Later moet de policyservice per aanvraag opnieuw de capability, objectscope en geldige machtiging controleren; userinhoud of modeltekst verleent geen rechten. Netwerkverlies, ontbrekend model of intrekking veroorzaakt een lokale fout, geen cloudfallback. De LP-01–14-tests in [LOCAL-PRIVACY-CONTRACT.md](../LOCAL-PRIVACY-CONTRACT.md) blijven afzonderlijke acceptatie-eisen; M4 kan die nog niet bewijzen. PQC beschermt later sleutels/protocollen, maar vervangt deze isolatie en autorisatie niet.

## 9. Primaire bronnen (APA 7)

Geraadpleegd op 26 oktober 2026. De geraadpleegde Intel-PDF is expliciet de editie 2016; de gebruikte paging-/privilegebasis is geen claim dat moderne processors geen errata of extra beveiligingsvereisten hebben. De numerieke limieten, ABI, layout en tests hierboven zijn projectbesluiten.

- **K1.** Intel Corporation. (2016, september). *Intel® 64 and IA-32 architectures software developer’s manual: Volume 3A, system programming guide, part 1* (Order No. 253668-060US; hoofdstukken 4–6 en 13). https://www.intel.sg/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3a-part-1-manual.pdf
- **K2.** The Rust Project. (z.d.). *x86_64-unknown-none*. The rustc book. Geraadpleegd op 26 oktober 2026, van https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-none.html
- **K3.** seL4 Project. (z.d.). *Capabilities*. seL4 Documentation. Geraadpleegd op 26 oktober 2026, van https://docs.sel4.systems/Tutorials/capabilities.html
- **K4.** seL4 Project. (z.d.). *What the proofs assume*. Geraadpleegd op 26 oktober 2026, van https://sel4.systems/Verification/assumptions.html
- **K5.** The Rust Project. (z.d.). *Behavior considered undefined*. The Rust Reference. Geraadpleegd op 26 oktober 2026, van https://doc.rust-lang.org/reference/behavior-considered-undefined.html

