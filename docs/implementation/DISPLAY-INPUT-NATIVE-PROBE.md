# Native grensproef voor display en PS/2

Deze afzonderlijke fixture is een uitvoeringstoets voor ABI 1.8 op het benoemde
ontwikkelprofiel `display-input-dev-800x600`: QEMU 10.0.13, `pc-q35-8.2`,
qemu64/TCG/1CPU/256MiB, UEFI-framebuffer strikt 800×600, eigen kernel en drie
werkelijke CPL3-programma's. Het is geen acceptatie van de Materiaal-desktop,
inputdaemon, focusbroker of basisprogramma's. De productomvang en alle open
criteria uit `FULL-TEST-RELEASE.md` blijven gelden.

De primaire geometriebron is de al bestaande
`boot/loader/src/graphics.rs::prepare`: de begrensde GOP-QueryMode-selectie
vereist `horizontal_resolution == 800`, `vertical_resolution == 600` en
`pixel_format <= 1`; ontbreekt die modus, dan volgt `GOP_800_600`.
De runner legt dit bronpad en de werkelijke bronhash vast. Dit kleine
ontwikkelprofiel verlaagt de verplichte volledige releaseprofielen (VM-FR,
1280×720 en 1920×1080/200%, met hun eigen resource-eisen) niet.

De eerste echte run op `fc9515d66d6d65662de14860f7dc1e3173d05465`
bereikte beide werkelijke Ready-markers van generatie 1 en leverde een echte
800×600-PPM. Het oorspronkelijke harnas verwachtte foutief 1280×720 en brak
de hostrun af met raw -9/`passed=false` vóór QMP-invoer. Dat was een
harnasfout. Dat mislukte bewijs wordt bewaard; een nieuwe run moet de volledige
pixel-, invoer-, rechten-, sequence- en cleanupcontrole alsnog doorstaan.

## Eigenaarschap en integratie

De nieuwe bronbestanden zijn de standalone crate
`sdk/examples/display-input-probe/**` en `kernel/src/m2/m3/display_fixture.rs`.
De hoofdagent bezit kernel-Cargo, gedeelde M3/M4-haken, SDK ABI/wrappers,
imagebouwer en QMP-runner. De fixture verleent geen algemene nieuwe rechten.
De bestaande `DISPLAY`/`INPUT`-objecten blijven uitsluitend manifestrechten.

Het nieuwe profiel heet `native-display-probe = ["display-input"]`. Het mag
niet tegelijk met een andere native fixture worden gekozen. De M4
capabilityengine, IPC-dispatch en usercopy blijven werkelijk actief; de oude
M4-proefmanifesten, waarnemers en hun tellers worden voor dit profiel
uitgesloten. De systeemproef wordt niet als alternatief programma ingebed.

`process_elf(slot)` gebruikt uitsluitend deze werkelijk gebouwde ELF's:

| Slot / eerste PID | Programma | Compile-time pad |
| --- | --- | --- |
| 0 / 1 | `display-probe` | `NOS_DISPLAY_PROBE_ELF` |
| 1 / 2 | `input-probe` | `NOS_INPUT_PROBE_ELF` |
| 2 / 3 | `display-intruder-probe` | `NOS_DISPLAY_INTRUDER_ELF` |

De kernel roept de nieuwe observer aan met `manifest(&State)`,
`observe_exit(&State, usize, u64)`,
`observe_fault(&State, usize, &[u64;22], u64)` en `poll(&mut State)`.
Na de bestaande framebuffer-voorbereiding blijven `display::prepare()` en
`ps2::prepare()` staan; de bestaande cleanup roept beide `detach(pid)` en
`m4::process_exit(pid)` aan. De observer doet zelf geen beeld- of poort-I/O.

De private startup-/bewijspagina gebruikt `src/contract.rs`. Deze pagina is
geen gepubliceerde applicatie-ABI. `SOURCE=48`, `WRONG_TYPE=56`,
`PEER_DISPLAY=64`, `PEER_INPUT=72`, `GENERATION=80`, `CONTROL=88` en
`OLD_SOURCE=96`. Resultaten beginnen bij 400; foutstap staat op 240.
Ieder proces heeft een eigen endpoint als verkeerd objecttype. De inputbron
heeft daarnaast een tweede endpoint zodat haar bronhandle een andere slotindex
heeft dan de displayhandle. Een gekopieerd getal mag geen bronrecht worden.

## Werkelijke handelingen en tegenproeven

De display-eigenaar vraagt via de getypeerde SDK de echte geometrie op en
tekent één tegel van 32×8 pixels. Daarna gaan doelbewust ongeldige descriptors,
epoch, geometrie, alpha, lengtes, overige registers en pointers als ruwe
`int 0x80`-aanroepen door de echte kernelgrens. Zo kan lokale SDK-validatie
een ontbrekende kernelcontrole niet maskeren. Ieder foutresultaat vereist
ook `RDX=0`.

Voor de laatste alpha-byte zijn de overige pixels een andere kleur. Voor de
copy-grens wijst de payload naar `0x60010000-320`: de eerste 320 bytes behoren
tot de echte private heap, de volgende pagina is ongemapt. De fout moet vóór
iedere framebufferstore optreden. De kernelobserver leest geen verzonnen
pixels terug; de externe runner vergelijkt de echte QMP-screenshot.

Ronde 1 tekent op (64,64), ronde 2 op (112,80). Voor generatie `g`, lokale
pixel `(x,y)` is de kleur:

```text
((0x20 + g*0x20 + x*3) << 16) | ((0x30 + y*0x13) << 8) | (0x70 + x + y)
```

Dat zijn gepakte `0x00RRGGBB`-pixels. De onafhankelijke runner berekent dezelfde
gepubliceerde patroondefinitie zonder de appresultaten als beeldbron te nemen.
Hij controleert de eerste tegel vóór nieuwe invoer, daarna beide tegels. De
tweede ronde overschrijft de eerste tegel niet.

De inputbron start zelf een reset en wacht op de echte ACK/BAT/ID0/Set2- en
controllertranslatieketen. Een startticket is geen Ready. Voor de eerste
reset-poll krijgt een ongeldige outputpointer een fout zonder prefixwijziging.
Pas de `Ready`-reply met de echte epoch maakt de bron actief. De lege Ready
wachtrij geeft EAGAIN zonder de outputsentinel te veranderen.

De observer meldt per generatie `DISPINPUT:DISPLAY_READY:<20 cijfers>` en
`DISPINPUT:INPUT_READY:<20 cijfers>`. De QMP-runner wacht op beide stappen en
maakt de screenshot. Daarna stuurt hij een echte `a` make/break, een
`input-send-event`-batch met relatieve x=7/y=-3 en linkerknop neer, gevolgd
door linkerknop op. De bron vereist precies deze acht werkelijke bytes:

```text
1e 9e 09 07 03 08 00 00
```

De eerste twee hebben keyboardstatus, de laatste zes AUX-status; OBF moet aan
en parity/timeout moeten uit zijn. De ruwe samples blijven ruwe bytes; deze
proef is geen hostdecoder of native inputpolicy. De bron vereist echte
sequences 1..8 en de actuele resetepoch. Vóór **iedere** geldige read probeert
zij een outputbuffer over de heapguard: die weigering mag geen FIFO-byte of
sequence verbruiken. Alle acht status/bytecombinaties worden bij echte exit
als `DISPINPUT:RAW:<20 cijfers>` uitgegeven, telkens voorafgegaan door
`DISPINPUT:SEQ:<20 cijfers>` uit de werkelijk ontvangen `Sample.sequence`.
De private sequencevelden staan op 640..696. De host valideert beide
transcripten afzonderlijk als sequence 1..8; hij construeert geen sequence uit
alleen de positie in de uitvoer.

De onbevoegde derde app probeert beide gekopieerde bronintegers, verkeerde
objecttypen en nulhandles. De twee eigenaars proberen afleiden, nulrechten
afleiden, intrekken en IPC-overdracht van hun bron; die blijven geweigerd.
Foutieve outputaanroepen behouden hun sentinels. Beide eigenaars sluiten hun
bron en controleren daarna de stale handle via een echte syscall.
Voor DERIVE/TRANSFER/REVOKE is de bestaande foutcode `Invalid` (-1): deze
operaties behoren niet tot de ondersteunde rechten van het bronobjecttype.
Twee gerichte hostcontracttests gebruiken de echte capabilityengine om deze
foutcodes, volledige foutatomiciteit en de gekopieerde/gesloten/oude handle-
verwachtingen vóór de native proef te controleren.

Wanneer alle resultaten klaarstaan, geeft de observer CONTROL=1, observeert
werkelijke exits en laat de normale cleanup lopen. Pas nadat alle processen
Dead zijn, controleert hij engine-invarianten, lege object-/queue-/handle- en
procesaantallen en herstelde fysieke frames, rekening houdend met expliciet
behouden kernelpagetabellen. Hij hergebruikt dan alle drie slots met nieuwe
PID's 4..6 en nieuwe manifestcapabilities. Oude bronintegers blijven ongeldig;
displayepoch, inputepoch en ticket moeten stijgen. De nieuwe inputbron mag
geen oude Ready erven en voert de hardware-initialisatie opnieuw uit.

## Tellers en bewijsgrens

Na twee rondes zijn de vaste getallen: BLITS=2, PIXELS=512,
DISPLAY_DENIED=22, CAP_DENIED=44, RESETS=2, READY=2, SAMPLES=16, CLOSED=4,
OLD_DENIED=2, EXITS=6, SLOT_REUSES=3, LIVE_PROCESSES=0 en
FRAMES_RECOVERED=1. POINTER_DENIED is minstens 18; OUTPUT_UNCHANGED is
POINTER_DENIED+27. RESET_PENDING is minstens 2. Meer lege reads zijn toegestaan,
maar iedere extra pointerweigering moet ook een ongewijzigde prefix opleveren.
De onafhankelijke adaptertellers zijn display `[2,512,35]` en input
`[2,2,16,POINTER_DENIED+17,werkelijk_discarded]`.

Alleen na werkelijke uitvoering, exact gevalideerde marker-/tellertranscriptie,
QMP-invoer en onafhankelijke pixelcontrole mogen de afsluitende markers
`DISPINPUT:DISPLAY:PASS`, `INPUT:PASS`, `ISOLATION:PASS`, `CLEANUP:PASS` en
`M1:PASS` als native bewijs gelden. Ruwe VM-exit moet 33 zijn. Een compile,
hosttest, ingevulde startupwaarde of vooraf getoonde screenshot vervangt dat
niet. De observer begrenst de proef op 10000 ticks en 2000000 syscalls.

## Uitgevoerde voorbereidende controle

`LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-display-probe-target python3 -B sdk/examples/display-input-probe/build.py`
bouwde de drie ELF's met Rust 1.90.0, eigen linker, offline lockfile en
uitgeschakelde SSE/AVX-codegeneratie. `objdump -d` vond in die echte binaries
geen FPU/SIMD-instructies of registers. De entry is een gewone extern-C-functie:
de compiler zet een `push rax` vóór de call naar main, passend bij de bestaande
kernel-entry met RSP mod 16 = 8.

De nieuwe crate is met native feature en none-target door Clippy gecontroleerd
met `-D warnings`; fmt en Python-syntaxcontrole slagen. De actuele bestaande
PS/2-testset (8 tests) en display-clienttestset (5 tests) slagen in eigen targets.
Dit voorbereidende bewijs is gebouwd met de door de hoofdagent aangeleverde
tijdelijke ABI1.8-SDK-patch; die patch wordt niet in de fixturecommit opgenomen.
De gezamenlijke native kernelbuild en QMP-bediening volgen in de integratietak.
Deze tekst claimt die uitvoering nog niet.
