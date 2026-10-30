# TASK-NATIVE-INPUT-SERVICE — echte bron en beleidsdienst

Dit uitvoeringscontract bouwt voort op input-core4/INP1 en de PS/2-adapter
`b2ad7c1`. Basis voor de eigen werkboom is `b6074c3`, daarna inhoudelijk
behouden dependency-integratie met `985ed4da`. De hoofdagent bezit bootstrap,
capabilitygrants, kernel-/SDK-hooks en de uiteindelijke native VM-gate.
Deze taak schrijft uitsluitend nieuwe `sdk/examples/input-service/**` en dit
document. Geen eigen kernelvervanging, nieuwe privileged parser of hostdecoder.

## Procesgrenzen

Twee nieuwe ring3-executables vormen de invoerketen; dit zijn geen twee totale
OS-processen:

| Proces | Benodigde eigen autoriteit | Gegevens en uitvoer |
| --- | --- | --- |
| input-source | Eén niet-delegeerbare InputSource, Source-ingressSEND, eigen replyRECV, beperkte RegistrySEND naar init | Werkelijke status/data uit broker, echte resetresultaten, INP1 Sync/Sample |
| input-policy | Exact vier RECV-ingresses: source, compositor, supervisor, apps; vooraf gekoppelde replySENDs; RegistrySEND | Bestaande input-core4, focus/generaties/cancel en PEEK/ACK |
| compositor, apart van beide diensten | DisplaySource en beperkte focusSEND/replyRECV | Werkelijke surfaces/focus; geen broninjectie |
| init/supervisor | Processtatus-/beheergrants en lifecycleSEND/replyRECV | Werkelijke procesdood naar INP1 lifecycle |
| gewone Materiaalapp | Document-, surface- en appINP1 SEND/replyRECV | Alleen eigen gefocuste events en eigen documentbytes |

Source-, compositor- en supervisor-PID blijven onderling verschillend zoals
`Service::new` vereist. Een gewone app krijgt geen ruwe input-/displaygrant.
De policy krijgt geen device-, volume-, netwerk- of entropycapability. Een
payload, geraden PID of lokale Handleconstructor verleent geen autoriteit.
Alle ontvangen `Received`-enveloppen komen in productcode uitsluitend uit de
werkelijke SDK `native::receive`; de core verifieert ingress en kernelafzender.

## Bootstrap en echte gereedheid

De hoofdagent legt de beperkte startmetadata vast vóór de ELF's worden
ingebed: schema/rol, eigen PID, service-epoch, source/policy/compositor/
supervisor-PIDs, vier verschillende policyRECV-handles, vaste replyroutes,
maximaal acht client-ID/PID/replySEND-bindingen, publieke breedte/hoogte en de
RegistrySEND naar init. Geen verzoek mag een willekeurige replyhandle kiezen.
De startpagina is eigen RAM, geen runtime-configuratierecht voor andere apps.
Het vaste IPD1-bootmanifest gebruikt de volgende u64-velden op DATA. Dit is geen
nieuwe algemene product-ABI. Prefix0–31 blijft de bestaande kernelbootstrap.

| Offset | Source | Policy |
| --- | --- | --- |
|32|`NOSISRC1` little-endian|`NOSIPOL1` little-endian|
|40|BootGeneration >0|BootGeneration >0|
|48|InputSource|sourceRECV|
|56|INP1 SourceSEND|focusRECV|
|64|eigen replyRECV|lifecycleRECV|
|72|PolicyPID|appsRECV|
|80|ServiceEpoch|SourcePID|
|88|RegistrySEND|CompositorPID|
|96|InitPID|Supervisor/InitPID|
|104|reserved0|ServiceEpoch|
|112|reserved0|breedte, hogere32bits nul|
|120|reserved0|hoogte, hogere32bits nul|
|128|reserved0|SourceReplySEND|
|136|reserved0|CompositorReplySEND|
|144|reserved0|SupervisorReplySEND|
|152|reserved0|RegistrySEND|
|160|reserved0|clientCount1–8|
|168+24×i|reserved0|client-ID, client-PID, replySEND in drie opeenvolgende u64's|

Alle ongebruikte bootstrapwoorden tot1024 zijn aanvankelijk nul. De daemon
leest die één keer vóór hij diagnostiek schrijft. Geef source dus geen oude
generieke ownPID op176; dat is hier een gereserveerd woord. Policy176 is de
eerste client-PID. Root levert de werkelijke identiteiten en publieke
GOP-geometrie, zonder handle-/PID-opzoeking op basis van een appverzoek.

READY gebruikt tag `u64::from_le_bytes(*b"IPDREAD1")` en precies32 payloadbytes:
`IPD1` op0, versie u16=1 op4, rol u16 op6 (1source,2policy), ServiceEpoch u64
op8, BootGeneration op16 en RawEpoch op24. Source meldt de werkelijk bevestigde
rawEpoch; policy meldt0. Init matcht kernelafzender met vaste rol, generatie en
epoch. Een tag of payload verleent geen identiteit. Registry-sendEAGAIN bewaart
dezelfde message; een geslaagde send betekent queued, geen InitACK. De lokale
handshake heeft een deadline van200 echte PIT-ticks, zonder polltellerfallback.

Policy meldt pas gereed na geldige configuratie en werkelijke validatie van
zijn ontvangstrechten. Source meldt pas gereed nadat de echte PS/2-broker
Ready heeft gemeld én de policy een echte geauthenticeerde INP1 Sync heeft
bevestigd. Registrygereedheid gaat als echte IPC naar init; een privaat
diagnostiekwoord of hostsucces vervangt die dependencyhandshake niet.

De typed InputClient gebruikt uitsluitend de vijf reeds afgesproken
SDKdisplay/input-wrappers, waarvan source alleen input0x82–0x84 nodig heeft.
ResetBegin geeft een ticket en betekent geen Ready. Ready volgt pas na echte
ACK/BAT/ID0/configuratiereadback door de afzonderlijke hardwareadapter.
De 32-byte rawEpoch is een ander domein dan de 96-byte INP1 serviceEpoch.
INP1 source-requestnummers blijven strikt monotoon over hardware-resets;
raw sequence start opnieuw op1 binnen de nieuwe rawEpoch.

## Bronloop en fouten

Na iedere werkelijke Ready stuurt source Sync(profile1) en wacht op het juiste
policy-PID, tag, opcode/replybit, request-ID, serviceEpoch, status en volledig
canonieke reply. Daarna wordt precies één echt rawsample vastgehouden totdat
de policy het heeft beantwoord. Het volgende rawsample wordt niet gelezen
terwijl dit verzoek wacht.

Request-sendEAGAIN betekent dat dezelfde eigen bytes met hetzelfde nummer
later opnieuw mogen worden aangeboden. Na sendOk betekent receiveEAGAIN
uitsluitend wachten: nooit het verzoek opnieuw publiceren. Bij reply-send-
EAGAIN kan de policy het sample al verwerkt hebben; dat wordt niet nogmaals
gedecodeerd. Time-outs gebruiken de werkelijke 100-Hz bootmonotone SDK-klok,
geen pollteller of wallclock.

Een echte parity/timeoutstatus gaat ongewijzigd naar de core. QueueFull en
decode-/controllerfouten worden expliciet bevestigd als fout, waarna source
een echte brokerreset en nieuwe Sync uitvoert. Geen fictieve foutbytes,
vooraf geschreven toetsen of Sync vóór hardwareReady. Onzekere publicatie,
verkeerde/ontbrekende reply-identiteit en niet verklaarbare rawepoch/sequence-
fouten beëindigen de source failclosed; de echte supervisor meldt SourceDead
en vernieuwt zo nodig de service-epoch en grants. Tussen detectie zonder sample
en die echte lifecyclemelding bestaat een expliciet supervisorvenster; de
core kan zonder nieuwe verlies-opcode niet eerder hiervan weten. Geen directe
synchrone corecancelclaim voor dat venster. Een los nieuw bron-PID kan
niet stil de oude lifetimebinding overnemen.

## Begrensde policyloop

Er zijn elf vaste routeslots: drie actoren en maximaal acht cliënten.
Ieder pending slot bezit de werkelijk ontvangen envelope, ingress en één
coreReply. Per peer is maximaal één onbeantwoord verzoek toegestaan.
Lifecycle, focus en source worden iedere ronde afzonderlijk bediend wanneer
hun eigen replieslot vrij is; gedeelde apps krijgen een vast klein budget.
Een tweede verzoek voor een bezet slot vervangt het eerste niet en bereikt
de core niet. Appfouten sluiten de gedeelde appRECV niet.

`handle` wordt eenmaal aangeroepen. Gewone corefouten worden waar mogelijk
met `error_reply` voor dezelfde gebonden canonieke envelope beantwoord, nooit
als Empty of succes. Onverwachte gedelegeerde caps worden op ieder pad precies
eenmaal gesloten. Een ongebonden/niet-canoniek verzoek krijgt geen verzonnen
antwoord. Alle outputhandles volgen uitsluitend uit bootstraproutebinding.

Iedere ronde probeert ieder pending slot maximaal eenmaal te verzenden, in
roterende volgorde. Een verouderd event wordt met de oorspronkelijke envelope
omgezet naar een negatieve StaleRequest. `reply_is_current` wordt direct vóór
de echte send opnieuw gecontroleerd zonder tussenliggende coremutatie.
EAGAIN bewaart het slot; alleen sendOk wist het. Een dode clientroute wordt
afgesloten; een gestopte core beëindigt de epoch. Er is geen herhaling van
`handle` om een acknowledgement opnieuw te maken. Alleen een echte geldige
ACK consumeert een aangeboden event; PEEK en mislukte sends doen dat niet.

Deze middelen bewijzen geen perfecte multiapp-toelatingsfairness: met één
gedeelde kernel-FIFO kan een floodende app een vrije positie steeds bezetten.
Een bevoegde supervisor moet daarvoor afzonderlijke ingressgrants kunnen
intrekken. De eerste native vertical slice gebruikt één Materiaalclient;
algemene resource-/floodisolatie blijft zichtbaar open. Ook kan de policy een
eerder succesvol verzonden event niet uit de kernelqueue terughalen na een
latere focuswisseling. De appadapter moet generatie/cancelwatermarks en
duplicatePEEK-na-mislukteACK correct toepassen en geen event dubbel uitvoeren.

## Acceptatie en bewijsgrens

Hosttests gebruiken echte capability-engine-enveloppen en begrensde
transportfouten voor de adapterlogica. Ze bewijzen geen hardware of native
afzenderherkomst. Targetbuild moet de echte SDKcalls bevatten; geen aparte
ASM-syscallimplementatie. Kernel-/sharedbuild-/workspacewijzigingen blijven
eigendom van de hoofdagent en worden niet in deze daemoncommit opgenomen.

De native poort vereist de echte bron- en policyprocessen, werkelijke
bootgrants/PIDs/ingresses, succesvolle hardwareReady+Sync en geauthenticeerde
PEEK→send→ACK; QMP-toetsen en muis moeten de gefocuste Materiaalapp bedienen.
Weigering van ruwe input voor de app, bronstatusfout/werkelijke reset,
focuscancel, replybackpressure, bekende procesdood en begrensde cleanup worden
met exact commit/configuratie/imagehash vastgelegd. Dit is nog geen complete
desktop-, invoerapparaat-, toegankelijkheids- of volledige releaseacceptatie.

## Bouwinterface en private diagnostiek

`python3 -B sdk/examples/input-service/build.py` bouwt met vastgezette Rust1.90.0
de echte `input-source` en `input-policy` voor `x86_64-unknown-none`.
De caller stelt een eigen CARGO_TARGET_DIR in. JSON geeft
`NOS_INPUT_SOURCE_ELF`, `NOS_INPUT_POLICY_ELF` en per ELF hash/segmentinspectie;
`native_runtime_performed` blijft false. Beide entrypoints zijn directe
`extern "C" _start`; de compiler verzorgt de callsitealignment vanaf de
bestaande kernel-RSP mod16=8. Er is geen parallelle ASM-syscalllaag.

Source schrijft alleen private tellers:400 resetStarted,408 resetReady,
416 SyncACKs,424 gelezen echte samples,432 SampleACKs,440 sendEAGAIN,
448 bevestigde fout/resetherstel,456 eerste RegistryReady queued,
464 laatste bevestigde rawEpoch,472 fatale stapcode en480 laatste echte tick.
Policy schrijft400 ontvangen,408 succesvol corehandled,416 afgewezen,
424 pipelined,432 replyEAGAIN,440 replies queued,448 stale replies vervangen,
456 clientroutes retired,464 werkelijke RECVcheckmask15,472 RegistryReady queued
en480 fatale stapcode. Dit zijn geen autoriteitsvelden of zelfstandige
acceptatie: init moet echte IPC ontvangen en native/provenancebewijs blijft
aan de volledige hoofdagentimage gebonden. Geen ruwe toetsen of documentinhoud
worden gelogd.

## Eerste uitgevoerde componentcontrole

Er zijn daadwerkelijk13 hostscenario's geslaagd: drie bootstrap-/registry-
bytecontracten en tien adapter-/lifetimegevallen met de echte capabilityengine.
Ze toetsen onder meer echte FIFO-EAGAIN, behoud van een pendingPEEK over
focuswijziging, geen herpublicatie na sendOk, verkeerde kernelafzender, twee
werkelijk gedelegeerde caps eenmaal sluiten, foutstatus/reset/monotone nummers,
terminalrawloss en deadline. De host-Rawfixture bewijst uitsluitend sequencing;
zij vervangt geen PS/2-broker. De bestaande grote host-engine krijgt dezelfde
expliciete16MiB-testthreadstack als andere SDK-hosttests; de native64KiB-stack
blijft ongewijzigd.

Host- en native-targetClippy met `-D warnings`, fmt en beide daadwerkelijke
`x86_64-unknown-none`-ELF-builds slagen. De linked executables zijn verder met
objdump/nm onderzocht: source2370 en policy4719 instructies, geen SIMD/FPU,
ruwe in/out, hostsyscall of onopgelost symbool. De directe C-entry bevat de
compiler-push vóór CALL en behoudt de bestaande SysV-stackalignment.
De grote initialisatieprologues tellen circa46KiB bij elkaar op; dit is een
gerichte artifactinspectie, geen volledige statische stackbewijzing of native
stackacceptatie.

De eerste Clippy-run meldde de grote sourceState-enumvariant. De wachtende
owned message is daarop als één vast optioneel veld buiten de kleine fase-
enum gezet, zonder heap of budgetverruiming; alle13tests en beide lints zijn
daarna opnieuw geslaagd. Geen veiligheidscontrole is uitgezet.

Voor de targetcontrole is de door root geleverde tijdelijke SDKABI1.8-wrapper-
patch gebruikt en daarna volledig hersteld. Die gedeelde dependencies worden
afzonderlijk geïntegreerd; deze daemoncommit bevat ze niet. Exacte commands,
bron-/dependency-/patch-/ELFhashes en de beperkte bronreview staan onder
`sdk/examples/input-service/verification/`. Er is nog geen daemon-QEMU-run
uitgevoerd: echte grants, hardware, registry/init, lifecycle en Materiaal-
bediening blijven de volgende geïntegreerde native poort.
