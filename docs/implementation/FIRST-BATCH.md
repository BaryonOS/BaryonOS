# Eerste uitvoerbatch — onderzoek uitgevoerd, nu implementeren
Revisie 5, 26 oktober 2026. Deze opdrachten gebruiken de gekozen implementatiespecificaties onder docs/implementation/. Zij herhalen geen brede architectuur- of conceptverkenning.

## Werkverdeling
De hoofdagent leest START-CODING.md, controleert bestaande gebruikerswijzigingen en beheert gedeelde configuratie. Daarna kunnen maximaal drie builders onafhankelijk werken:
- B01: boot/ en kernel/ plus crates/boot-protocol/ en crates/elf-loader-core/; wijzigingen in rootbuildbestanden en tools/ via de hoofdagent.
- P02: uitsluitend ui/prototype/.
- Q01: uitsluitend experiments/quantum/ met eigen Cargo-workspace; de rootworkspace sluit deze map uit.

Nieuwe broncode ontbreekt nog; deze paden zijn te creëren. De hoofdagent bezit root Cargo.toml, rust-toolchain.toml, root Cargo.lock en tools/. B01 bezit de boot-protocol-/ELF-parsercrates volgens de overeengekomen ABI. Q01 bezit alleen zijn geneste Cargo.lock. Elk gedeeld bestand heeft één eigenaar. Gebruik geen gelijktijdige schrijvers voor die bestanden. Review en test na integratie, niet alleen per agent.

## B00 — werkelijke ontwikkelomgeving gereedmaken
**Input:** BOOT-AND-TOOLCHAIN.md en tools/preflight.py.
**Actie:** voer de read-only inventaris uit; vergelijk versies, voeg ontbrekende tools toe binnen de beschikbare bevoegdheden en verifieer de gekozen targets/firmware. Leg echte versies en firmwarehash vast. Geen bestaande gebruikersconfig vervangen.

**Klaar:** buildgereedschap werkt of er is een exacte concrete blokkade met bewijs. Een blokkade voor QEMU blokkeert P02 niet en hoeft Q01 niet te blokkeren als een Rust-hosttoolchain beschikbaar is.

## B01 — eigen loader draagt over aan eigen kernel
**Input:** BOOT-AND-TOOLCHAIN.md, kernelcontract en toepasselijke AGENTS.md.
**Actie:** implementeer de afgesproken ELF-loader, BootInfo, overdracht na firmwarebootservices, kernelentry en seriële rooktest. Volg de vaste baseline; open alleen gerichte documentatie bij een concrete implementatiefout.

**Klaar:** kernelmarker én correcte emulatorstatus op een echte run; corrupte/ontbrekende kernel en timeout worden als fout gemeld. Firmwarebanner of alleen cross-compilatie telt niet als geslaagde kernelboot. Logs vermelden commit/basis, commands, versies en resultaat.

## P02 — beleefbaar prototype van de gekozen productrichting
**Input:** PRODUCT-SPEC.md.
**Actie:** codeer de daarin gekozen schermen, toestandsovergangen, tokens en toetsenbordbediening. Geen nieuwe ronde conceptselectie. Gebruik lokale fictieve fixtures. Geen cloud, tracking, externe fonts of echte NAS-bestanden.

**Klaar:** de afgesproken kerntaken zijn interactief doorlopen; fout-/lege-/laadtoestanden en toegankelijkheidschecks zijn aanwezig. Test desktop en het kleinere viewport uit de specificatie. Label dit als productprototype; het draait nog niet op de eigen kernel.

## Q01 — lokale quantumproef
**Input:** QUANTUM-SPEC.md.
**Actie:** implementeer het vaste kleine simulatorprofiel, de jobinterface en limieten. Geen nieuwe taal-/backendselectie en geen cryptografische implementatie in deze taak. Houd simulatorcore los van host-JSON/CLI.

**Klaar:** referentiecircuits, ongeldige requests, quota en annulering doorstaan de gespecificeerde tests. Rapporteer hostsimulatie afzonderlijk van de toekomstige OS-port/QPU. Geen netwerk of quantumcloud.

## Integratie en afsluiting
1. Hoofdagent controleert daadwerkelijke gewijzigde bestanden en shared-interfaceversies.
2. Laat een reviewer concrete fouten en ontbrekend bewijs zoeken; review maakt niet-uitgevoerde tests niet groen.
3. Voer relevante checks op de gecombineerde toestand uit, herstel fouten en herhaal alleen betrokken tests.
4. Werk PROJECT-STATE.md bij met wat geïmplementeerd/gedraaid/niet gedraaid is.
5. Sluit deze batch af met de volgende afgebakende implementatietaak; genereer geen nieuwe algemene onderzoeksopdracht.

Geen publicatie, echte schijven, firmwareflash, NAS-configuratieveranderingen of releasekeys in deze batch. Het gaat om lokale broncode, prototypes en geïsoleerde tests.

## Concrete ontwerpwijziging tijdens bouwen
Als een gekozen contract aantoonbaar fout blijkt: leg de fout, bron/test en kleinste herstelkeuze vast; coördineer afhankelijke bestanden en ga door. Een goede implementatie volgt geen fout document blind. 'Ik wil eerst alle opties opnieuw vergelijken' is geen blokkerende reden.
