# Inputdiensten wachten op werkelijk ontbrekend werk

INPUT-IDLE-WAIT-1, implementatie op basis9596abc. Deze wijziging verlaagt de
runnable pollbelasting van de bestaande aparte CPL3-inputsource en -policy.
Zij decodeert geen nieuwe invoer, verandert geen token/ACK/IPC-contract en
vergroot geen kernel- of apparaatbevoegdheid. Hosttests en doelbuilds zijn
geen native prestatie- of GUIacceptatie; root voert die gate afzonderlijk uit.

## ABI en eigenaarschap

Alleen de exacte gerapporteerde ABI1.8 en1.9 worden geaccepteerd.1.8 behoudt
de bestaande nonblocking-IPC/yield-lus. Alleen bekende1.9 gebruikt
`native::ipc_wait(&[WaitCondition], absolute_tick)`. Andere minors/majors en
onbekende versiewaarden falen; geen numerieke toekomstcompatibiliteitsclaim.
De SDK en kernel implementeren die boundary elders, niet in de inputdiensten.

`WaitCondition::send(SendEndpoint)` en `::receive(ReceiveEndpoint)` lenen
uitsluitend bestaande grants. Maximaal16 voorwaarden, maximaal1000 ticks
vooruit; een lege lijst is een timer-only wait. Alle voorwaarden worden
opnieuw gevalideerd. Closed/Revoked/BadHandle zijn fouten, geen gereedbits;
ze wekken fail-closed en verbergen ook eventuele andere readybits. Een bit
reserveert of consumeert niets. Gewone IPC na wake valideert opnieuw.
De caller houdt geen verwijzing naar een andere adresruimte vast.

De read-only plannen staan in `source::wait` (de module staat fysiek in
`sdk/examples/input-service/src/wait.rs`). Geen SDK/kernel/lib/Cargo-interface
wordt door deze taak gewijzigd. Exact-dubbele interesses worden samengevoegd,
verschillende send/receive-interesses niet. Capaciteit/klokoverflow falen;
er is geen heap of onbeperkte verzameling. De native adapter voert hoogstens
één waitcall uit en behandelt diens mask uitsluitend als wakeuphint.

## Source: lokaal werk vóór wachten

| Werkelijke toestand na één bestaande bounded step | Plan |
| --- | --- |
| Begin; reset start/poll Pending; nieuw SYNC/Sample; echte Ready/SampleAccepted/Recovering | Runnable/yield: volgende lokale overgang uitvoeren, geen slaap |
| Unsent request en werkelijk send-EAGAIN | Alleen request-SEND; dezelfde outgoingbytes en oorspronkelijke exchange-deadline |
| Request echt queued en receive-EAGAIN | Alleen response-RECV; nooit opnieuw send omdat een wakebit niets ACKt |
| Werkelijke raw `read()` geeft Idle/None | Geen IPC-interesse; timer tot oorspronkelijke waarnemingstick+1 |
| RegistryReady nieuw gemaakt | Runnable; eerst echt send proberen |
| RegistryReady send-EAGAIN | Alleen registry-SEND, originele ready-deadline |
| Fataal | Bestaande cleanup/exit; nooit wachten of retry |

IRQ1/12 wekt deze wait nog niet. De rawPS2-Idlewacht duurt daarom hoogstens
tot de volgende PITtick; geen bewering van interruptgestuurde input. Een
reset-Pending is geen Idle en krijgt deze timeroptimalisatie niet. Is de
tick tijdens een syscall/preëmptie al verstreken, voer dan meteen de volgende
bounded step uit en voeg geen nieuwe tick toe.

Exchange en Ready bewaren hun bestaande `now-started > 200` grens: de
wakegrens is checked `started+200+1`. De bestaande resetgrens blijft>500.
Geen wake, EAGAIN, nieuwe planberekening of preëmptie vernieuwt started of
deze absolute deadline. Een verlopen deadline leidt eerst weer naar de echte
toestandmachine, niet naar een verse wachtperiode.

## Policy: alleen bruikbare routes

Een bestaande pass controleert eerst lifecycle, focus en source-ingress,
daarna maximaal2 apprequests en iedere retained reply precies één keer.
Pas na die volledige pass is wachten toegestaan. Tot de vier echte
receive-rechten gecontroleerd zijn blijft het proces runnable voor de
bestaande Ready-preflight.

Het plan bevat source-/compositor-/supervisor-RECV alleen als de betreffende
route geen behouden reply heeft en dus weer ingestbaar is. App-RECV behoudt
zijn bestaande bounded drain-/afwijsbeleid. Voeg uitsluitend SENDs toe voor
de werkelijk retained replies (maximaal11 bij8clients), niet voor alle
mogelijke replyendpoints. Een stale reply die lokale herberekening nodig
heeft houdt het proces runnable. Een nieuw bericht dat na de laatste check
aankomt wordt door de atomische level-triggered wait direct gezien; er is
geen lost-wakeupvenster tussen hostcheck en slaap.

De policy introduceert geen nieuwe RPC-timeout. Een idle-wacht heeft een
bounded1000tickhorizon; bestaande protocoldeadlines van de peers blijven
leidend. Indien eigen RegistryReady werkelijk EAGAIN gaf, voeg diens SEND
toe en verkort uitsluitend tot de oorspronkelijke ready-deadline. Alle
plannen blijven binnen16voorwaarden. Een beurtbudget dat bereikt is kan geen
lokaal behouden antwoord verbergen: alle retained replies zijn al geprobeerd
en alle overige kernelqueues staan als eligible receiveconditie in het plan.

## Behoud en verificatie

Plannen lezen status; zij alloceren geen requestID, wijzigen geen frame,
token/cancelgeneratie of pendingreply en fabriceren geen input/ACK. Send-EAGAIN
is niet enqueue; na echte enqueue wordt niet opnieuw gepubliceerd. Wake kan
weer EAGAIN opleveren. Een sourcewachtfout inABI1.9 gaat naar bestaande cleanup
en zichtbare failure. Policy behoudt de bestaande per-app-isolatie: uitsluitend
Closed/Revoked/BadHandle uit de waitcall worden eenmaal via de gewone bounded
IPC-pass hergecontroleerd. Alleen een daadwerkelijk nieuw geretirede app-lane
verklaart zo'n fout en laat verder wachten toe. Kritieke, onverklaarde, klok-,
yield-, masker- of andere waitfouten blijven fataal; geen stille fallback naar
pollsuccess ofABI1.8.1.8 krijgt geen nieuwe extra waitklokvoorwaarden.

Acceptatie per component: echte Engine-envelopes plus Ipc-fouttegenpartijen
toetsen eligible RECV, volle replyqueue/SEND, nieuw lokaal werk, stale/cancel,
replyverlies/Closed, cleanup en oorspronkelijke deadlines; malformed ABI,
max16/duplicaten/overflow krijgen negatieve checks. Daarna bestaande
inputdaemon-hosttests, Clippy/fmt en echte scalar freestanding-ELFs met de
bevrorenSDK1.9. De hoofdgate meet pas op één geïntegreerd image echte
QMP-keyboard/mouse, ongewijzigde focus/Save/reopen/cleanup, absolute200s-grens
en poll-/waitbelasting. Geen hosttest of minder syscallclaims vervangt die
native uitvoering; alle volledige releasecriteria blijven open.
## Historische componentcontrole, geen native integratiebewijs

Bronfreeze `2b286d2837b1a8760865c6c2c39bd2e1ea0389a5`, dependency exact
`8a8dbe53acf5c8c7882244ddf7f753a8645a1d0a` (lokaal als ongewijzigde
dependency-cherry-pick `ae3f022`). Bestaande root-/SDK-/kernelbestanden zijn
uitsluitend uit die dependency overgenomen; de consumercommit schrijft ze niet.

| Controle op deze consumerbron | Uitkomst |
| --- | --- |
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/input-service/Cargo.toml` | Exit0,21 scenario's:8 nieuwe waitplancases en13 bestaande boot/daemoncases |
| `cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/input-service/Cargo.toml --all-targets -- -D warnings` | Exit0 |
| Zelfde Clippy voor `--release --target x86_64-unknown-none --features native --bins` | Exit0, beide mains werkelijk typechecked |
| `cargo +1.90.0 fmt --manifest-path sdk/examples/input-service/Cargo.toml -- --check` | Exit0 |
| `python3 -B sdk/examples/input-service/build.py` met eigen targetmap | Exit0, beide echte scalar-ELFs plus bestaande `inspect_user`-controle |
| Disassembly van die twee ELFbestanden | Source2936/policy5299 instructies, geen FP/SIMD-instructies gevonden |

De source-ELF heeft SHA256
`129556644b7edb2862bd02920799d637072da1e7cd110dcecae111c65a25ff6a`;
policy `0caeab494a672cb628da87c134566aa0df6a26ff585df8269778c6ea085d6fdc`.
De builder bewaart de bestaande64KiB-stack; de bounded planwaarde is272bytes
en motiveert de plaatselijke `large_enum_variant`-lintuitzondering zonder
allocator of algemene uitschakeling van checks.

Exacte commandologs, bron-/ELFhashes en inspectie staan in de eigen
`build/input-idle-wait/logs/{final-checks.json,component.json}`; binaries
uitsluitend in `target/input-wait-native`, niet in Git. Twee vroege
hostpogingen hadden een ongeldige fixture-registratievolgorde (Engine vereist
monotone PIDs); de fixture is gecorrigeerd, niet de productie-engine.
Onafhankelijke consumerreview en native wait-/GUI-uitvoering blijven open.

## Gerichte correctie: foutisolatie van appreplies

Eigen bronherlezing van2b286 vond dat een lifetimefout uit de samengestelde
wait alle policyclients fataal zou maken, terwijl de oude IPC-flush alleen
een werkelijk defecte appreplylane retireert. De opvolgbron behoudt die
isolatie met één bounded hercontrole. Klok-, yield- en maskfouten hebben
afzonderlijke fouttypen en kunnen die herstelroute niet gebruiken.

De hertoets telt23 hostscenario's: de13 bestaande en10 waitplancases. De twee
nieuwe cases toetsen echte gesloten, revoked en stale-handle appreplyleaves
en nog werkende tweede-app-IPC; een onverklaarde lifetimefout, ander fouttype
of kapotte kritieke ingress wordt geen pollsuccess. Host- en beide native-
target-Clippychecks slagen. Nieuwe executablehashes volgen afzonderlijk;
de bovenstaande2b286-hashes blijven uitsluitend bij hun oorspronkelijke bron.
