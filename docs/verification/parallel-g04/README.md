# G04 — gerichte review van native quantumuitvoering

## Uitkomst en afbakening

De native rekenproef heeft daadwerkelijk in twee ring-3-processen op de eigen
kernel gedraaid. Deze review vond één concrete fout in de scalar-instructiegate
(`G04-01`, hieronder). Er is in de gecontroleerde allocator, invoerbegrenzing,
echte core-uitputting, kernel-entry, frameadministratie en opruiming geen tweede
blokkerende fout gevonden. Dit is een gerichte AI-code- en bewijsreview, geen
menselijke audit of formeel veiligheidsbewijs.

De beoordeelde integratiecheckout was gebaseerd op
`41bac3c173d464a286017d31558335d70899c434`, met nog niet gecommitteerde integratie.
Daarom bindt `review.json` de daadwerkelijk gelezen bestanden afzonderlijk met
SHA-256. `directed-result.json` is een ongewijzigde kopie van de reeds door de
integrator uitgevoerde run, geen extra VM-run van deze reviewer. Het totale
bronfingerprint van die run is
`c447a518bb034d9f76e283b252ffe68a90cc16d6ef8c5f7e15aeff21c09e71ad`.

## G04-01 — prefixes konden de scalar-gate omzeilen

`experiments/quantum/native/build.py`, SHA-256
`7b2e171d8418ab838736b8c995029f565c5aa16b06879ae415a2e79b8a992a5f`,
behandelde het eerste token na het adres als de instructie. GNU `objdump`
decodeert de echte bytes `66 db e3` als `data16 fninit` en `f3 db e3` als
`repz fninit`. Beide werden geaccepteerd met `extended_registers_found=false`;
dezelfde `fninit` zonder prefix werd wel geweigerd. De reproductie voegt
`cd 80` toe om ook de verplichte bestaande syscall-site aanwezig te hebben.
Geen van deze fixturebytes is uitgevoerd.

Dit is een fout in de controle die het ontbreken van uitgebreide CPU-context
moet onderbouwen. De reproductie toont niet aan dat de huidige qsim-ELF een
FPU-/SIMD-instructie bevat. De kleinste reparatie is prefixes volledig te
verwerken vóór classificatie, onbekende instructievormen gesloten af te wijzen,
echte gedisassembleerde prefixgevallen als negatieve controles toe te voegen
en de gate opnieuw op dezelfde ELF uit te voeren. De hoofdagent heeft dit als
I04 toegewezen aan de eigenaar van het native quantumonderdeel; deze review
wijzigt geen implementatie.

Reproduceer de gevonden fout of controleer de reparatie:

```sh
python3 docs/verification/parallel-g04/prefix-repro.py /pad/naar/nieuw-os
```

`prefix-repro.json` bewaart de oorspronkelijke foutuitkomst. Een gerepareerde
gate moet `all_negative_controls_rejected=true` geven. Deze historische
reproductie mag niet achteraf als geslaagde oorspronkelijke controle worden
omgelabeld.

De latere I04-kandidaat is afzonderlijk gecontroleerd in `fix-check.json`.
Met gatehash `269d0d6abdfba5385554ec40a992cf7acfcc3de5b70251c58cef8babc59a78fa`
worden alle drie oorspronkelijke negatieve controles geweigerd. Dezelfde
gerepareerde gate accepteert ook de oorspronkelijke directed-run-disassembly
van 4.096 instructies en vier `int 0x80`-sites, gebonden aan ELF `3a20a6e8…`.
De code verwerkt bekende prefixketens vóór de expliciete opcode-allowlist en
faalt gesloten bij onbekende vormen. G04-01 is daarmee opgelost in deze
specifieke kandidaatbytes; opname van die bytes en de definitieve native
bronbinding blijven bij de integrator. De oorspronkelijke foutregistratie
en reproductie blijven ongewijzigd bewaard.

## Getoetste grenzen

| Grens | Beoordeling en concrete betekenis |
|---|---|
| Allocator en levensduur | De private heap begint uitgelijnd op `0x60000000` en omvat 64 KiB. `allocation_range` controleert powers of two, overflow, gebruikte ruimte, uitlijning en eindpositie. De atomische cursor stijgt alleen; er is geen reset of hergebruik zolang verwijzingen kunnen leven. `dealloc` doet bewust niets. Voor deze eindige proef ontstaan daardoor geen overlappende geldige allocaties. Dit is geen herbruikbare dienstallocator. |
| Allocatiefouten | De proef weigert eerst een aanvraag van een hele arena nadat ruimte in gebruik is, zonder cursorwijziging. Daarna reserveert zij de resterende ruimte en roept `execute(1, &[], 1, ...)` aan. De lege gatekopie vraagt geen buffer; de bestaande core probeert werkelijk zijn statevector te reserveren en geeft `ResourceExhausted`. Alleen een zelfgemaakte foutstatus of hostinjectie zou deze controle niet vervangen. |
| Invoer en resultaat | Maximaal 6 qubits, 64 gates en 4.096 shots; target/control-validatie vóór allocatie; vaste uitvoer van maximaal 64 bins. De native adapter bezit geen bestand-, netwerk-, klok- of deviceautoriteit. |
| Annulering | Stops op Control-aanroep 1 en 8 geven exact `Aborted(Cancelled)` en geen histogram. Dit controleert afbreking vóór allocatie en tijdens de berekening. Het bewijst geen extern native cancelobject. |
| Deadline | Stop op Control-aanroep 80 geeft exact `Aborted(DeadlineExceeded)`. Dit is de in de specificatie toegestane deterministische injectie; er is geen vergelijking met een native monotone klok en geen gemeten realtimegarantie. |
| Entry en onderbreking | De bestaande IDT gebruikt interruptgates `0xee`/`0x8e`; de entry wist DF, gebruikt de permanente servicestack en kopieert alle 22 contextwoorden vóór mogelijke vrijgave. De observer draait met IF=0 en kernel-CR3, kopieert 112 bytes uit gevalideerde private mappings en houdt geen gebruikersreferentie over herplanning vast. |
| Procesisolatie | Het manifest vereist verschillende PID, patroon, pagetableroot, rapportpagina en heapmapping. De bestaande `State::audit` controleert alle gebruikersmappings, rechten en frame-eigenaars; de proef vereist dat beide processen geselecteerd zijn en ten minste één timerinterrupt is verwerkt. |
| Opruiming | De baseline is vrije frames plus exact getelde user-, kernelstack- en pagetableframes. Vóór vrijgave wordt de permanente root/stack vastgehouden; bestaande cleanup zero/free-controles lopen per eigenaar. Het resultaat vereist de oorspronkelijke baseline, `Dead`, geen pagetables en nul mappings voor beide processen. |
| Betekenis van PASS | De kernel controleert volledige rapportvelden en sluit foutstatus, onverwachte exit/fault, timeout en afwijkende numerieke resultaten uit. De berekening blijft in de brongebonden probe. Het rapport is geen bewijs dat een willekeurig onbetrouwbaar programma nooit constanten zou kunnen fabriceren; de vaste bron en het geladen artifact horen bij de claim. |

## Daadwerkelijk bekeken native bewijs

De bestaande directed-run eindigde zonder timeout met QEMU-exit **33** en de
vereiste boot-, Bell-, zes-qubit-, cancel-, deadline-, resource-, isolatie- en
cleanupmarkers. De kernel was
`4c0937a6ce257c0d5ebe9a27f9df00a2700da92d67cb6faef5d1e0c37d43ce47`;
de qsim-ELF was
`3a20a6e8566b899ffec4939a03a9a7e5f970ca2f5b0c7d84fd85f5dccc860a5a`.
Per proces rapporteerde de uitvoering 253 checkpoints, 7.216 nuttige arenabytes
en maximaal 448 werkeenheden tussen checkpoints. Beide Bellhistogrammen zijn
`2082/2014` bij 4.096 shots. Daarna werd de volledige 65.536-byte-arena uitgeput.
De kernel rapporteerde drie timerinterrupts, 505 syscalls en 96 teruggegeven
frames. Dat zijn observaties van deze TCG-run op het expliciete
`pc-q35-7.2/qemu64/1CPU/256MiB`-profiel, geen fysieke hardware- of prestatieclaim.

De run noemt zichzelf terecht een directed-integratieproef; de definitieve
exacte quantumharness was nog open. Deze review promoveert dat bewijs niet tot
een volledige eindacceptatie. De originele run-, artifact- en bronbindingen
staan in de JSON-bestanden naast dit document. Latere integratiewijzigingen
kunnen het totale v3-fingerprint veranderen; de gelezen bronhashes en de
oorspronkelijke runidentiteit blijven afzonderlijk zichtbaar.

## Vast roadmapcriterium L02

Ongewijzigde acceptatietekst uit `docs/progress.json`:

> De lokale simulator/jobinterface draait native met quota, annulering en
> referentiecircuits; hostsimulatie en fysieke QPU-ondersteuning blijven
> afzonderlijk gelabeld.

**Advies: L02 blijft open, delta 0 punten.** Het simulatorgedeelte en de
begrensde callable reken-API werken native. De huidige executable biedt alleen
gecompileerde proefjobs; zij roept uitsluitend `abi_info` en `yield` aan.
Er is nog geen native pad om een job van een ander proces aan te bieden,
het resultaat terug te ontvangen en die job gericht te annuleren.
`QUANTUM-SPEC.md` §1 en §5 beschrijven de userspace-dienst als integratiedoel;
de native README benoemt joblevensduur, begrensde IPC-aanvoer en een
cancelobject expliciet als vervolg.

Dit oordeel voegt geen verplicht transportformaat aan de vaste tekst toe.
Een publieke Rust-functie is een bruikbare native library-interface, maar
de getoonde integratie bewijst alleen de vaste proefaanroeper. De kleinste
open poort is een enkel, begrensd native jobpad met resultaat en gerichte
annulering onder de bestaande procesautoriteit. In de gekozen architectuur
kan dat bestaande Message320-IPC gebruiken, met een expliciete eindige
joblevensduur zodat geheugen terugkomt. Een algemene workerpool, 12 qubits,
JSON in de kernel, nieuwe syscalls, wallclock-real-timegaranties of een fysieke
QPU zijn daarvoor niet nodig. Een gewone native aanvrager moet een geldig
referentiecircuit kunnen aanbieden, foutieve/overgrote jobs moeten begrensd
falen en een voortijdige annulering mag geen voltooid deelresultaat leveren.

De nieuwe native rekenmogelijkheid verdient wel een zichtbare beschrijving
in de projectstatus. Het ontbreken van een volledig punt maakt deze port en
de bijbehorende native resourceproeven niet ongedaan.

## Uitvoering en eigenaarschap

Deze taak heeft alleen reviewbewijs geschreven. Bron- en testwijzigingen voor
G04-01 blijven van I04/root. De rootagent blijft eigenaar van de vaste score,
integratie en uiteindelijke native acceptatie. De opgeslagen bronhashes,
commando's en exitstatussen staan in `review.json` en `prefix-repro.json`.
