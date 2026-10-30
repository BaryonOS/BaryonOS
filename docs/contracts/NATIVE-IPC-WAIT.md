# IPC-WAIT-1: begrensd wachten op bestaande IPC-rechten

Werkbasis: `9596abc576fceefe2b3ff2f3bd120bcd914eb04c`. Dit is een opt-in
mechanisme en SDK-contract, geen afgetekende GUI- of releaseacceptatie.
De componentbron is voorbereid voor integratiereview; uitgevoerde controles
en open poorten staan hieronder. Er is geen native proef uitgevoerd door
deze componentagent.

## Integratie op actuele main

De integratiebranch `simon/native-test-version` gebruikt basis `ad2b981`.
De hieronder beschreven historische component is overgenomen, niet opnieuw
ontworpen. De wachtinterface krijgt **ABI 1.10**: ABI 1.9 is inmiddels door
ADR-050 aan de niet-vertrouwde RTC-klok toegewezen. Die klok en de uitgebreidere
userreturn-/stackvalidatie uit main blijven behouden. De historische
componentmetingen hieronder gelden niet als nieuwe native acceptatie.
`native-ipc-wait` is een afzonderlijke tweeprocessenproef; productprofielen
blijven voorlopig ongewijzigd. De integrator voegt een native fixture toe
alvorens de bestaande GUI-gebruikersketen met wachten te verbinden.

## Invariant en ABI 1.10

Alle bestaande SEND/RECV-, copy-, capability-, generatie-, IPC- en
digestcontroles blijven behouden. Wachten reserveert of consumeert niets en
verstuurt geen bericht. Een ready-melding garandeert uitsluitend dat de queue
op dat moment ruimte respectievelijk data bevat. Een echte volgende send of
receive valideert opnieuw alles en kan EAGAIN, Quota of een andere fout geven.
Er ontstaat geen nieuw recht, object, node, fysieke mapping of autoriteit.

Kernelfeature `ipc-wait` is standaard uit en vereist milestone-m4 plus
monotonic-clock. Alleen die profielen rapporteren ABI 1.10. Bestaande profielen
behouden hun ABI, gedrag en terminale idleproef. ABI 1.0–1.9 blijven in de SDK
geaccepteerd; een ouder profiel ondersteunt syscall 0x22 niet.

```text
ipc_wait(items_pointer, count, absolute_tick, 0, 0, 0) // syscall 0x22
entry = { handle: little-endian u64, interest: little-endian u64 }
interest = 1 SEND of 2 RECV, nooit beide of andere bits
0 <= count <= 16; maximaal 256 gekopieerde bytes
count == 0 vereist items_pointer == 0
toekomstige afstand <= 1000 PIT-ticks, nominaal 10 seconden
RAX == 0, RDX == ready-mask; bit i hoort bij entry i
RDX == 0: deadline verstreken zonder ready entry
RAX < 0: bestaande kernelfout; RDX == 0
```

De complete invoer wordt vóór registratie gevalideerd en naar een begrensde
kernelkopie omgezet. Een CopyPlan, userpointer, userbuffer, message, objectindex
of nodereferentie wordt niet in een waitrecord bewaard. De normale bewaarde
gebruikersregisters hebben hun bestaande betekenis; zij worden bij hervatten
niet opnieuw als copy-invoer gedereferenceerd. De wachttabel bewaart uitsluitend
eigen PID, volledige handles met generatie, interests en absolute tijdwaarden.

Alle voorwaarden krijgen de bestaande check_endpoint-validatie: levende PID,
slothandle/generatie/node, Revoked, Closed, type, rechten en RECV-eigenaarschap.
Een eerder ready item verbergt nooit een fout in een later item. Fouten maken
de wachtende procescontext weer uitvoerbaar met die fout en nul als tweede
resultaat. De bestaande foutvolgorde blijft gelden. Dezelfde handle mag meer
dan eens voorkomen; ieder resultaatbit blijft aan zijn invoerpositie gebonden.

Na volledige capabilityvalidatie heeft daadwerkelijke readiness voorrang
op een inmiddels verstreken deadline. Is niets ready, dan geeft een verstreken
deadline direct nul. Alleen een nog toekomstige afstand groter dan 1000 wordt
als Invalid geweigerd; gewone syscallvertraging verandert een verstreken
korte wacht niet in Invalid. De deadline wordt nooit stil vernieuwd.
Een teruglopende kernelklok of onjuiste PID-binding geeft geen succes.

De nieuwe read-only corequery kijkt bij SEND naar `len < 16` en bij RECV naar
`len > 0`. Dezelfde bevoegdheidschecks worden bij iedere schedulerbeoordeling
opnieuw uitgevoerd. Close, revoke, process-exit en queueonderhoud kunnen dus
ook wekken zonder send/receive. Intrekking kan queued capberichten verwijderen
en SEND-ruimte vrijmaken. Laatste SEND-close is geen EOF; bestaande berichten
zonder overgedragen caps blijven geldig volgens het bestaande contract.

## Scheduler, echte idle en interruptgrens

De scheduler heeft optioneel een Waiting-status met een eigen gekopieerd
waitrecord. De huidige syscall maakt het proces pas Waiting nadat registratie
is voltooid. De iteratieve selectielus beoordeelt alle wachtenden vóór het
kiezen van een Ready-proces. Een signaal vóór registratie wordt in de huidige
queue gezien; tussen controle en parkeren kan op één CPU met IF=0 geen andere
Engine-mutatie optreden. Er is geen losse check-then-enable-wakeupregistratie.

Als uitsluitend wachtende processen overblijven, wordt de bestaande
supervisor-root en permanente servicestack gebruikt met één aaneengesloten
`sti; hlt; cli`. De bestaande CPL0 IRQ0-route raakt alleen raw idlecounters en
het interruptframe, nooit de uitgeleende State of Engine. Na CLI wordt de
idleroute uitgezet en de daadwerkelijk getelde PIT-delta precies één keer
bij de monotone klok, IRQ-counter en idle-metriek opgeteld. Geen schatting van
verloren ticks, wallclock of certificaattijd. Daarna lopen dezelfde brokers en
observers, waarna de selectielus opnieuw beoordeelt. Geen recursieve
idle→select→idle-lus. Een wake met nul PIT-delta vervalt niet automatisch tot
een geslaagde timerwacht; die deadline blijft gelden.

Dit profiel schakelt geen driver-IRQ in. De huidige PIC laat alleen IRQ0 toe;
PS/2 IRQ1/12 zijn bovendien in het gecontroleerde apparaatprofiel uitgeschakeld.
Er wordt dus geen fictieve input- of virtio-interrupt als wakebron gebruikt.
De Source mag alleen na echte RawIdle tot de volgende PIT-tick slapen, en
moet beschikbare bytes blijven verwerken tot het apparaat werkelijk leeg is.
Pending reset- of brokerwerk behoudt de bestaande begrensde voortgangspaden.

Processen behouden hun eigen mappings, kernelstack en 22-woordencontext terwijl
zij slapen. Er komt geen userpointer- of DMA-lifetimeverlenging bij. Bestaande
FPU-entry-save en terminale restore blijven ongewijzigd; idle-IRQ0 vanuit CPL0
voert geen gebruikers-FPU-restore uit. Kill en cleanup blijven mogelijk voor
Waiting; Process::new wist het waitrecord bij dood en slothergebruik.

De bestaande m4-foutcontrole blijft exact staan. De nieuwe waitregistratie
vergelijkt zelf de volledige Engine-Stats vóór en na het geserialiseerde
registratiesegment, ook bij succes. Er wordt **geen** globale digest over de
slaaptijd vergeleken: andere processen mogen daarin legitiem de Engine wijzigen.
Readinessqueries hebben alleen `&Engine` en veranderen geen Engine-veld.

## SDK en integratiegrens

```rust
WaitCondition::send(SendEndpoint)
WaitCondition::receive(ReceiveEndpoint)
native::ipc_wait(&[WaitCondition], absolute_deadline) -> Result<u16, Error>
```

De SDK begrenst count, maakt de wirekopie en weigert resultaatbits buiten count.
Een lege lijst is alleen een eigen timerwacht. Consumers behouden hun eigen
absolute protocoldeadline en kiezen bijvoorbeeld de kleinste van die deadline
en de eerstvolgende noodzakelijke poll. Geen automatische IPC-replay,
deadlineverlenging, gegokte Received of hostuitvoering. Zestien voorwaarden
dekken onder meer de vier Policy-ingangen, elf begrensde antwoordroutes en
RegistrySEND. Een daemon mag niet slapen terwijl hij nog lokaal render-,
wis-, decodeer- of ander direct uitvoerbaar werk heeft.

Consumerwijzigingen, exacte ABI-checks in bestaande daemon-ELFs, profielaliases,
builder, bootstrap, native fixture en runner behoren aan de integrator. Dit
component levert geen reeds geïntegreerde of geteste daemonversnelling.
De bestaande max_syscall_cycles-metriek omvat bij een blokkerende syscall ook
het opzettelijke wachten; dat mag niet als actieve kernelrekentijd worden
gepresenteerd. De nieuwe interne waitmetrics onderscheiden calls, registratie,
directe resultaten, wake-resultaten en werkelijk getelde idle-IRQ0's.

## Verplichte negatieve en native acceptatie

- Pure core- en waittests: leeg/vol/wrap, verkeerde rechten/type/PID/owner,
  stale handles, intrekking, laatste RECV-close, process-exit, hergebruik,
  alle items gevalideerd, geen queue/node/handle-mutatie, ready versus Quota.
- Wire/SDK: count 0/16/17, reserved args, vreemde interests, hoge handles,
  te verre/verstreken deadlines, klokterugloop, te hoge resultaatbits en
  volledige pointervalidatie inclusief een onleesbare tweede invoerpagina.
- Native lege RECV→send→wake en volle SEND→drain→wake, met echte bytes en
  daadwerkelijke syscallresultaten; onmiddellijke readiness vóór registratie.
- Native volle SEND-queue met wachtende sender: owner trekt SEND afgeleid van
  zijn root in terwijl de queue vol blijft. Vereist Revoked vóór de deadline,
  zonder queueprogress en zonder false-success/replay.
- Native alle processen tegelijk geblokkeerd, waaronder count0-slaap:
  echte STI/HLT, PIT-telling, Sys3-voortgang en correcte schedulerhervatting.
- Native kill van een wachtend proces, echte cleanup en slothergebruik zonder
  oud resultaat; FPU- en IPC-isolatie blijven in relevante regressies getoetst.
- Daarna de ongewijzigde volledige GUI-handeling en legacy gates op exacte
  geïntegreerde commit, configuratie en artifacts. Geen timeoutverruiming.

De afzonderlijke native fixture vereist vooraf contractreview en wordt door
root gewired en uitgevoerd. Tot dat bewijs blijft de native wait- en
performanceacceptatie open, met nul zelfstandige releasepunten.

## Uitgevoerde componentcontrole

De aparte lifecyclecase `test-supervisor-wait` gebruikt uitsluitend de
bestaande K05 Launch/Process/SEND/KILL/OBSERVE-rechten. Zij moet twee echte
Waiting-kinderen beëindigen, Process-derive zonder DERIVE en een endpoint-kill
weigeren en na schoon
PID/slothergebruik de oude status-/killhandle weigeren. De nieuwe wacht
moet voorbij de bovengrens van de oude deadline intact blijven. De observer
verandert geen processtatus of wake: hij controleert de gewone spawn/kill/
cleanup-uitkomst en verlangt lege Engine en volledig frameherstel.
Alle zes K05-cases en hun hostorakels blijven vereist. Native uitvoering
wordt pas afgetekend met de brongebonden rapporten onder
`verification/native-test-version/`.

De integratie voegt daarnaast een afzonderlijke `test-fpu-wait`-proef toe:
de bestaande echte x87/SSE-apps vervangen alleen op ABI1.10 hun yield door
een begrensde count0-deadlinewait. Alle registervergelijkingen,
timerpreëmpties, AVX-appfault, survivor, PID3/slothergebruik en cleanup blijven
vereist. Het native orakel eist echte blokkeringen en idle-IRQ's, uitsluitend
deadlinewakes en exact één wachtcall per geverifieerde lus. Het normale
ABI1.6-profiel blijft apart. Dit test contextbehoud rond blokkeren; het is
geen bewijs voor kill van een nog Waiting proces. Uitvoeringsstatus en
exacte bron staan in `verification/native-test-version/`.

Alle 47 oorspronkelijke coretests blijven behouden. Uitgevoerd:

| Controle | Werkelijke uitkomst |
| --- | --- |
| Core standaard | 57 geslaagd, 0 mislukt, 1 expliciete benchmark ignored |
| Core diagnostic-bench | 58 geslaagd, 0 mislukt, 1 ignored |
| SDK-runtime | 22 geslaagd: 5 unit-, 14 message- en 3 procescases |
| Werkelijke pure WaitSet-decoder/evaluator | 7 hosttests geslaagd |
| Clippy core, SDK en WaitSet-hostharnas | Exit 0, alle targets, `-D warnings` |
| SDK scalar-none releasebuild en Clippy | Exit 0 |
| Kernel native-system + ipc-wait + fpu-context | Scalar-none releasebuild exit 0 |
| Kernel native-system + fpu-context, zonder ipc-wait | Scalar-none releasebuild exit 0 |
| Kernel Clippy, beide configuraties | Exit 101, exact dezelfde 12 bestaande meldingen |

De twaalf kernelmeldingen zitten in ongewijzigde keyring-, volume-, paging-,
main- en drie bestaande m3-regels. De gestructureerde lintsets zijn met elkaar
vergeleken; geen nieuwe waitmelding en geen suppressie om deze controle groen
te verklaren. Deze algemene kernelcontrole blijft rood. Core- en SDK-Clippy
zijn afzonderlijk werkelijk groen. De kleine `never_loop`-annotatie op select
geldt alleen wanneer ipc-wait uitstaat: dan blijft de oude terminale idle-uitgang
bestaan; zij onderdrukt geen van deze twaalf meldingen of beveiligingscheck.

De WaitSet-tests compileren de echte pure decoder/evaluator uit wait.rs in een
apart hostharnas, zonder native adapter. Zij voeren geen IRQ, scheduling of
usercopy uit. De testgevallen omvatten alle zestien interests, scalargrenzen,
absolute deadline/klokterugloop, latere ongeldige caps na eerdere readiness,
gekopieerde invoer, echte Engine-queuewijzigingen, intrekking terwijl vol en
meerdere senders zonder reservering. Native adapter, idle, PID-lifetime en
contextwisseling zijn daarnaast werkelijk voor de freestanding kernel gebouwd.

De kernelcompile gebruikt vijf gecontroleerde, lokaal gekopieerde ELF-inputs
uit de bestaande cap-bench-bundle. Dit zijn alleen link-/typecheckinputs; hun
oude ABI wordt niet als geslaagde 1.9-dienstuitvoering gepresenteerd. Er is
geen QEMU gestart. De integrator bouwt de echte nieuwe fixture/consumer-ELFs.

De bronreview ontdekte een concrete observerrecursie: sdk_app en qsim_job
riepen vanuit poll opnieuw select aan. De vier nauw goedgekeurde returns
(sdk_app twee, qsim_job één, m4 lifecycle één) geven nu None terug en laten de
enige m3-event/selectlus plannen. Hun M4-profielen passeren geen oude
M3-fixturetak. qsim.rs blijft ongewijzigd; diens combinatie met ipc-wait wordt
door de bestaande milestone-m4-compilevoorwaarde geweigerd. Gerichte herlezing
vond geen resterende concrete blocker. Dit is AI-bronreview, geen menselijke
audit of native acceptatie.

De volledige productie-digestbody, Stats-body, m4-syscallbody en arch.rs zijn
letterlijk gelijk gebleven aan basis 9596abc. De bestaande 22-woordenlayout,
entry-save, userreturnvalidatie en terminale FPU-restore blijven intact.
In de gebouwde kernel staan STI/HLT/CLI daadwerkelijk direct naast elkaar;
de herstelbare idleroute wist de idleflag direct na CLI. De terminale oude
idleroute blijft apart aanwezig. Dat is instructie-inspectie, geen uitvoering.

```text
componentbron v3 2c793f9314f8a47ca4d4b784fff2fa84509a3c860722ede8c4d301c227690289
core lib.rs SHA256 ce50c2e7838e07da07523952b2749872ea154127930d4248c3f2827e673e8546
m3.rs SHA256 7a4b67cfcfb5111d9facb5f80d9c7b2f93c0d38ed4bc0f5803d2c2fb63a534d9
wait.rs SHA256 678709d0d822f4c21a3bba76f691cb2905f00635b96c1797fe61d0a60b20e88d
compile-only kernel SHA256 78762586bab6dc7ff362afe66afcdc5470b5cab0a577f1ceb29939d3c65c9d45
```

Eigen logs en compacte bronbinding: `build/ipc-wait-checks/host-report.json`,
`target-report.json`, `final-target-report.json`, `source-binding.json` en de
bijbehorende afzonderlijke stdout/stderr. Alle uitvoer ligt onder de eigen
worktree/target. Rust 1.90.0, offline/locked waar een lock bestaat. Scalarflags:
code-model=kernel, relocation-model=static, no-redzone, SSE/SSE2/SSE3/SSSE3/
SSE4.1/SSE4.2/AVX/AVX2 uit, expliciete padremap en bestaande linkergrenzen.
De nieuwe public SDK-signatures zijn beschikbaar voor onafhankelijke consumers;
hun eigen host-, target- en native acceptatie blijft afzonderlijk.
