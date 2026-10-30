# RTC als niet-geauthenticeerde UTC-bron

`drivers/rtc` is een zelfstandige allocatorvrije `no_std`-crate met
`forbid(unsafe_code)` en zonder dependencies. Basiscommit:
`cda42f2d4017e65bd8ba89399cb1633f85a1c2da`. De crate levert alleen de
CMOS-leesprocedure, datumcodec en expliciete bronkwaliteit. Het opt-in
`native-rtc`-profiel voegt een native portbackend en bootstrap-acceptatie toe.
Het aparte profiel `native-system-clock` voegt ABI 1.9-syscall `0x73` toe:
een ring-3-app vraagt een actuele UTC/Unix-secondewaarde op, die de kernel
rechtstreeks en alleen-lezen uit CMOS leest. De SDK markeert elk resultaat
`UnauthenticatedRtc`. Dit is geen vertrouwde TimeProvider of
certificaatintegratie. Sys3 blijft daarnaast de afzonderlijke boot-monotone
deadlineklok; ticks worden niet in een kalenderdatum omgerekend.

## Eigenschappen en vertrouwensbasis

| Eigenschap | Wat deze crate afdwingt | Vereiste aanname/open grens |
| --- | --- | --- |
| Afgebakende I/O | Alleen getypeerde indexselectie, settle en dataread; geen CMOS-datawrite. | De opt-in native backend gebruikt uitsluitend poorten 0x70/0x71; geen algemene app-I/O. |
| NMI/index | Iedere selectie gebruikt dezelfde expliciete NMI-policy; de oorspronkelijke index wordt op succes en fout hersteld. | De platformeigenaar levert een betrouwbare index/NMI-shadow en sluit alle andere CPU-/IRQ-/NMI-gebruikers uit. |
| Samenhangende datum | Twee volledige gelijke snapshots, UIP vóór/na, constante mode en geldige velden. | Correct devicegedrag en begrensde adapteroperaties; dit authenticeert de klok niet. |
| Begrensde uitvoering | Eén globaal register-readbudget en een begrensd aantal snapshotpogingen. | Native I/O en settle zelf moeten begrensd terugkeren; dit is geen gemeten tijdsdeadline. |
| UTC en eeuw | Alleen expliciete UTC-platformbasis plus expliciet eeuwregister. | De RTC bevat werkelijk UTC; geen detectie van lokale tijd of authenticatie van configuratie. |
| Unixconversie | Geldige Gregoriaanse datum, jaar 1–9999, gecontroleerde i64-bewerkingen. | Geen leap-secondrepresentatie; seconde 60 wordt geweigerd. |
| Terugloop | Vergelijking met eerdere geaccepteerde lezing of callerfloor; een lagere tijd geeft fout. | In-memory toestand gaat verloren bij herstart. De crate authenticeert of bewaart een externe floor niet. |
| Vertrouwde certificaattijd | **Niet geleverd**: de SDK geeft altijd `UnauthenticatedRtc` terug. | Een geautoriseerde TimeProvider, versheid, onzekerheidsmarge en rollbackanker ontbreken. |

Een stabiele klok kan verkeerd zijn, stilstaan of door een aanvaller worden
teruggezet. Een gelijke waarde wordt geaccepteerd; grote sprongen vooruit
worden niet als aanval herkend. De aanwezigheid van een timestamp geeft dus
geen toestemming om certificaatdatums over te slaan, een fout certificaat te
accepteren, TOFU te gebruiken of deze bron als betrouwbare tijd te presenteren.
Bij ontbrekende of onvoldoende tijdkwaliteit moet de toekomstige TLS-grens
een concrete tijd-/validiteitsfout teruggeven. Dat beleid wordt hier niet
gesimuleerd als reeds geïmplementeerde TLS-beveiliging.

## Interface en levensduur

`Rtc::new(io, initial_selection, configuration)` neemt exclusieve toegang tot
het index-/datapaar over, zonder I/O. `CmosIo` exposeert:

```rust,ignore
fn select(&mut self, selection: IndexSelection) -> Result<(), Self::Error>;
fn settle(&mut self) -> Result<(), Self::Error>;
fn read_data(&mut self) -> Result<u8, Self::Error>;
```

`select` schrijft uitsluitend de indexpoort 0x70 met `selection.encoded()`;
`read_data` leest uitsluitend 0x71. Er is geen datawrite-interface. `settle`
verzorgt alleen de begrensde vertraging, zonder extra registerread of wijziging
van index/NMI; ook herstel naar register C mag geen interrupt bevestigen.
Het hele lezen, inclusief herstel, wordt
geserialiseerd tegenover alle andere gebruikers van het paar. Een Rust
`&mut` of trait is geen kernelrecht en sluit een andere CPU/NMI-handler niet
zelf uit.

Port 0x70 wordt niet teruggelezen om de NMI-toestand te raden. Bootstrap moet
de oorspronkelijke index en de expliciete NMI-policy kennen/bezitten en een
`IndexSelection` leveren. De lezer verandert de NMI-maskerbit niet tijdens
de procedure. Als indexherstel mislukt, bevat `ReadError` zowel de primaire
fout als de herstelfout en wordt de instance poisoned. Volgende reads doen
geen I/O. `into_io()` draagt het paar over voor gecontroleerd adapterherstel;
er is geen verborgen I/O in Drop.

Configuratie kiest een eeuwregister vanaf 0x0e tot en met 0x7f. De positie
moet uit gecontroleerde platformconfiguratie/FADT komen. De eerste native
QEMU-proef zal expliciet 0x32 gebruiken. Ontbrekende eeuw of onbekende
tijdbasis geeft een fout vóór I/O; er bestaat geen pivot op bouwjaar,
hosttijd, huidige datum, lokale timezone of een vooraf ingevulde timestamp.

`read_utc()` geeft `Reading { utc, date, source, quality, rollback }` terug of
een fout. `UnixUtc` bevat getekende i64 Unixseconden voor jaar 1–9999;
`date` bewaart de daadwerkelijk gedecodeerde velden. `source=CmosRtc` en
`quality=UnauthenticatedRtc` zijn vaste, zichtbare waarden. Een externe
minimumtijd verschijnt als `FloorSource::CallerSupplied`; de crate claimt
daarmee geen geauthenticeerde opslag. Een succesvolle read werkt de eigen
in-memory floor bij; mislukte reads en mislukt indexherstel doen dat niet.

De crate heeft een eigen `[workspace]` en lockfile. De latere integrator
moet de aansluiting op de gedeelde workspace expliciet regelen; er zijn nu
geen gedeelde Cargo-, kernel-, SDK- of buildbestanden gewijzigd.

## Hardwarecodec en foutpaden

Het oorspronkelijke core-registercontract is het QEMU 10.0.13
MC146818-model; de native proef hieronder meet expliciet QEMU 10.2.1 bij
het q35-profiel. Dit is geen steun voor ieder fysiek RTC-model.
Register A moet de lopende 32,768-kHz-divider aangeven; UIP mag tijdens het
lezen niet veranderen. SET in B of een gestopte/ononderzochte divider geeft
een fout. Automatische DST in B wordt geweigerd bij dit UTC-profiel. D moet
precies de geldige-batterij/tijdwaarde 0x80 hebben. Een zwevende 0xff-bus of
expliciete offline-I/O geeft geen tijd; een nulwaarde is conservatief
`BatteryInvalid` en bewijst niet afzonderlijk of een device ontbreekt.
Register C wordt niet gelezen, zodat lezen geen RTC-interrupt bevestigt.

De lezer verwerkt BCD/binaire waarden en 12-/24-uursmode zonder modewijziging.
AM 12 wordt 00, PM 12 wordt 12. Niet-decimale BCD-nibbles, ongeldige velden,
kalenderdagen, eeuw, schrikkeldagen of een PM-bit in 24-uursmode worden
geweigerd. Een maand-/jaarovergang of gewijzigde mode geeft een nieuwe poging.
Maximaal 65.536 registerreads en 32 snapshotpogingen zijn configureerbaar;
een kleinere configuratie kan terecht al vóór een volledige snapshot falen.
Elk foutpad probeert het oorspronkelijke index/NMI-woord te herstellen.

Er worden geen alarmen, IRQ-modes, RTC-datum, CMOS-inhoud, firmware of fysieke
klokken gewijzigd. De pure RTC-core verricht zelf geen echte port-I/O;
alleen de opt-in bootstrapadapter hieronder voert die uit.

## Bronnen, hergebruik en licenties

- QEMU, *MC146818 RTC emulation*, versie **10.0.13**:
  <https://github.com/qemu/qemu/blob/v10.0.13/hw/rtc/mc146818rtc.c> en
  <https://github.com/qemu/qemu/blob/v10.0.13/include/hw/rtc/mc146818rtc_regs.h>.
  Dit is de primaire bron voor het voorlopige emulatorprofiel, niet een
  hardwarecertificering. Bronbestanden zijn gelezen; er is geen emulatorcode
  overgenomen.
- Howard Hinnant, *chrono-Compatible Low-Level Date Algorithms*, 1 september
  2021, `days_from_civil`:
  <https://howardhinnant.github.io/date_algorithms.html#days_from_civil>.
  De March-/400-jaar-formule is hergebruikt met gevalideerde invoer en checked
  i64-bewerkingen. De auteur doneert de formules expliciet aan het public
  domain. De gelezen bron is via de upstream GitHub-mirror verkregen;
  <https://github.com/HowardHinnant/HowardHinnant.github.io/blob/master/date_algorithms.html>.
- ACPICA, `ACPI_TABLE_FADT.Century`, gelezen 30 oktober 2026:
  <https://github.com/acpica/acpica/blob/master/source/include/actbl.h>.
  Het veld levert een registerindex; deze crate implementeert geen FADT-parser.
- Aanvullende leesindex: OSDev, *CMOS*, **W001717**, revisie 30263 van
  16 maart 2026:
  <https://wiki.osdev.org/index.php?title=CMOS&oldid=30263>.
  De PC-NMI/indexconventie is aan de expliciete adaptergrens vastgelegd.

De fabrikantdatasheet kon vanuit deze omgeving niet worden opgehaald (HTTP
403). Daarom is geen fysieke RTC-ondersteuning afgetekend. De daadwerkelijk
gelezen bronnen zijn met SHA-256 vastgelegd in het verificatierapport. Er
zijn geen nieuwe libraries, build scripts of externe dependencyversies.

## Native bootstrap-contract RTC-NATIVE-1

Basis `5c334f9`, branch `simonvanieperen-expert-disco`. Dit is een directe
technische afhankelijkheid van FR-S03/FR-N03, geen volledige aftekening.
De uitsluitend diagnostische `native-rtc`-feature draait na eigen GDT/IDT,
pagetables en M2-geheugencontrole, op de ene CPU met IF=0. Combineren met
ring 3 of framebuffer wordt bij compilatie geweigerd. Geen productieprofiel
schakelt de feature impliciet in; geen proces krijgt nieuwe I/O-rechten.
De kleine codec draait in deze terminale proef in de kernel om de echte
hardwareadapter te toetsen.

Platformbezit wordt expliciet gevestigd door register D met NMI Enabled te
selecteren. Dit is een eigen bekende startselectie, **geen reconstructie van
de firmwareselectie/NMI-policy**. De proef maskeert NMI niet; de eigen
NMI-handler is terminaal en gebruikt geen CMOS. Er zijn geen firmwarecalls,
andere CPU's, IRQ-/userspace-CMOS-gebruikers of gedeelde portreferenties.
De backend heeft alleen `out8(0x70)` en `in8(0x71)`, geen datawrites, algemene
portargumenten of port-0x80-vertraging. QEMU verwerkt de indexwrite synchroon;
de begrensde scalar settle-loop is geen timingcontract voor fysieke hardware.
Register C wordt niet gelezen. De q35-platformconfiguratie kiest eeuw 0x32
en UTC expliciet; geen FADT-parser of automatische platformdetectie geclaimd.

Acceptatie: twee echte UTC-readings met zichtbare `UnauthenticatedRtc`,
datum/Unix-gelijkheid en in-memory floor; een werkelijk uitgeput readbudget,
weigering van een onhaalbare callerfloor, onbekende tijdbasis/ontbrekende eeuw
zonder I/O, daarna opnieuw succesvol lezen. Na succes en beide I/O-foutpaden
wordt de startselectie hersteld: software-indexcontrole plus een echte
dataread **zonder reselectie** moet register D=0x80 opleveren. Dat controleert
de registerselectie op het device; NMI-policy is een expliciete
code-/platforminvariant, geen onafhankelijk gemeten hardwaregarantie.
De pure hosttests behouden de corrupte/offline/UIP-/mode-/poison-tegenproeven;
de native negatieve gevallen injecteren geen fictieve device-antwoorden.

## ABI 1.9-proef NATIVE-CLOCK-1

`native-system-clock` is een afzonderlijk opt-in single-CPU systeemprofiel,
geen algemene hardwareondersteuning. De app roept syscall `0x73` aan zonder
argumenten of pointers. Bij succes retourneert `RAX=0` en `RDX` de signed
Unixseconden als two's-complementwaarde; ongeldige extra argumenten geven
`-22`, een onbruikbare RTC geeft `-5`, en profielen zonder de syscall blijven
de normale onbekende-syscallfout geven (`-2`). De SDK biedt
`native::wall_clock_utc()` als `WallClockReading { unix_seconds, quality }`;
de enige kwaliteit is `UnauthenticatedRtc`.

Iedere aanroep doet een begrensde, read-only snapshot, controleert de RTC-kern
en het register-D-herstel en geeft de oorspronkelijke indexselectie terug.
De aanroep heeft geen gebruikerspointer, capability of bestandstoegang.
Dit is een publiek lokaal tijdgegeven, geen nieuw recht op andere resources.
De adapter veronderstelt nog steeds exclusief CMOS-bezit op één CPU met IF=0,
een expliciete UTC-basis, eeuwregister 0x32, geen IRQ-/NMI-/andere
CMOS-gebruiker en een NMI-handler die zelf geen CMOS leest. Er is geen
persistente antirollback, authenticatie, versheids-/onzekerheidsbewijs,
time-sync, suspendcorrectie of TLS-certificaatbeleid. De monotone PIT-ticks
worden niet gebruikt om wall-clock te schatten of te corrigeren.

De native proef laat de bestaande ring-3-systeemapp twee tijden lezen,
controleert hun volgorde en de monotone-ticks, en test een syscall met een
niet-nulargument. De onafhankelijke runner vergelijkt de gerapporteerde
seconden met een expliciete QEMU RTC-basistijd. Zie
`docs/verification/system-clock/README.md` zodra die native uitvoering is
opgeslagen. Dit sluit geen FR-S03/FR-N03-productcriterium af.

```sh
NOS_QEMU_MACHINE=pc-q35-8.2 python3 -B tools/build.py --kernel-feature native-rtc
NOS_QEMU_MACHINE=pc-q35-8.2 python3 -B tools/run_rtc.py --case all
python3 -B -m unittest tools.tests.test_system_clock
python3 -B -u tools/run_system_clock.py
```

De runner gebruikt bestaande `run_guest` met een eigen ESP/VARS/CODE-kopie
per echte VM en `-rtc base=<expliciete UTC>,clock=vm,driftfix=none`. Vijf
kalenders: millennium 2000, schrikkeldag 2000, 2026, grens na signed-i32-2038
en schrikkeldag 2096. Python datetime controleert onafhankelijk zowel
de zes velden als de Unixconversie en het venster base..base+60 seconden;
dit venster dekt firmware/boot en bewijst geen realtime-nauwkeurigheid.
Geen hosttijd als OS-fallback, geen netwerk, geen echte disk en geen writes
naar RTC-data. Raw exit 33, volledige geordende uitvoer, bronfingerprint,
manifest- en artifacthashes zijn vereist. Timeout/foute datum/duplicaten/
gewijzigde bron falen; een oude laatste rapportage wordt eerst ingetrokken.
CI heeft een eigen verplichte uitvoerende workflow `native-rtc.yml`.
`--case epoch` en `--case century` blijven expliciete firmwarediagnostiek,
niet onderdeel van de normale vijfkalenderacceptatie: op het gemeten OVMF-paar
werd 1970 werkelijk 2070 en 2100 werkelijk 2000. Het onverzwakte orakel
weigert beide met exit 1, ondanks raw33 en geslaagde read-/restoremarkers.
EDK2 `edk2-stable202511`, `PcAtChipsetPkg/PcatRealTimeClockRuntimeDxe/PcRtc.c`,
`ConvertRtcTimeToEfiTime` reconstrueert de eeuw uit de PCD-minimumjaarperiode;
`PcRtcAcpiTableChangeCallback` schrijft die naar het FADT-eeuwregister.
Dit verklaart het gemeten gedrag; de native backend corrigeert of raadt
geen andere eeuw. Core-Gregoriaanse hosttests blijven het brede datumbewijs;
native 1970/2100-acceptatie en een volledige firmware-onafhankelijke datumrange
blijven open. Nieuwe firmwarepakketten worden opnieuw werkelijk getoetst,
niet automatisch als gelijkwaardig aan deze ontwikkelvariant beschouwd.
Geen authentieke tijd, persistente antirollback, NTP, TLS, userspacegebruik,
fysiek apparaat of volledige productacceptatie door deze proef.

## Historisch corebewijs en volgende productiepoort

Met Rust 1.90.0 en aparte targetmappen uitgevoerd:

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-rtc-target \
  python3 -B tools/dev.py cargo +1.90.0 test \
  --manifest-path drivers/rtc/Cargo.toml --locked --offline
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-rtc-target \
  python3 -B tools/dev.py cargo +1.90.0 clippy \
  --manifest-path drivers/rtc/Cargo.toml --locked --offline --all-targets -- -D warnings
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-rtc-none-target \
  python3 -B tools/dev.py cargo +1.90.0 build \
  --manifest-path drivers/rtc/Cargo.toml --locked --offline \
  --release --target x86_64-unknown-none --lib
```

Elf tests slagen zonder skips: overflow, vaste epochankers en alle 146.097
dagen van een Gregoriaanse 400-jaarcyclus, BCD/binaire uurmodi, foutieve
datums, update-/mode-/middernachtovergangen, vastgelopen UIP, verdwenen device,
readbudget, herstel/poison en zichtbare rollbackfloor. Clippy en de
freestanding librarybuild slagen. `drivers/rtc/verification/` bevat het
afgebakende bron-/artifacthashbewijs. Dit is hostbewijs en een targetbuild,
geen native uitvoering van de RTC-driver.

De volgende productiepoort blijft open: een minimaal geautoriseerde
broker/TimeProvider, gedeeld CMOS/NMI-bezit bij IRQ/SMP, echte overgangen/
devicefouten en vervolgens tijdkwaliteit bij certificaatvalidatie. Fysieke
klok, beveiligde tijd en persistent rollbackanker zijn nog niet bewezen.
Deze stap verandert de releaseomvang en voortgangspunten niet zelfstandig.
