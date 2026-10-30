# Native RANDOM-proef naast de geïntegreerde systeemketen

TASK-RANDOM-NATIVE-PROBE, basis985ed4d met de twee beoordeelde KEYRANDOM-
commits. Deze stap levert probe/observer/dienstadapter en componentbuilds;
**nog geen uitgevoerde native RANDOM-gate**. De integrator levert de echte
bootstrap, feature, images, koude keten en runtimebewijzen. Alle eisen uit
[FULL-TEST-RELEASE](../FULL-TEST-RELEASE.md) blijven verplicht.

## Grens en procesopbouw

`native-system-random` bouwt voort op `native-system,monotonic-clock`.
Init, volume en de bestaande v7document-/cryptoproef behouden hun gedrag;
Key krijgt de afzonderlijke RND-ingress en de app alleen de hieronder
beschreven opt-in STATUS-barrière. Een vijfde initieel proces voert de RANDOM-proef uit. Zijn echte
PID wordt opgeslagen; slot4 mag na exit door een worker worden hergebruikt.
De observer leest de probegegevens vóór cleanup en bewaart een snapshot.

De probe krijgt exact twee handles: RANDOM-SEND en eigen reply-RECV.
Geen rawEntropySource, sleutelingress, recovery/snapshot, opslag, netwerk,
procesbeheer, DERIVE, TRANSFER of REVOKE. De bestaande Key Service bezit twee
sleutelingresses; RANDOM wordt zijn derde van maximaal vier endpoints.
De probe bezit één reply-endpoint. Geen endpointquota worden verhoogd.

Bootstrap behoudt bij Key een RANDOM-root en een daarvan afgeleide
SEND|DERIVE|TRANSFER|REVOKE-ancestor. Alleen een SEND-leaf gaat naar de probe.
De Key-RECV is een afzonderlijke tak. Intrekking van de SEND-ancestor raakt
die RECV niet; de ancestor zelf wordt wel ingetrokken. De rebindfase gebruikt
de nog geldige ancestor vóór intrekking. De reply-root wordt na afleiding van
de unieke probe-RECV en Key-SEND gesloten. Er is geen verborgen RECV-alias.

De fixture verleent alleen expliciete testrebind/revoke binnen deze scope.
Zij is geen product-API of implementatie van account-/NAS-onboarding. Kernel
en capability-engine krijgen geen cryptografie, RNG of protocolparser.

## Vier gevallen

| Case | Werkelijke handelingen en aftekenbewijs |
|---|---|
| 0 RAW | Native RND1-IPC voor1/32/64/128/224bytes; alle9oude opcodes en9foute requestvelden geweigerd;64calls binnen een echt100tickvenster en quotaantwoord; wachten op echte ticks; call- én4096byte-reseed; nativeclose→nieuweSEND-generatie zonder principalreset; oudeIDgeweigerd; gesloten unieke replyRECV gevolgd door echte aanvraag; Key blijft werken, pending wordt gewist/ingress retired; daarna SEND-subtreeintrekking en daadwerkelijke Revoked-fout. |
| 1 SDK/cancel | De echte SDK/NativeTransport vraagt1/32/64/128bytes; daarna wordt een werkelijk geënqueuede32byteaanvraag geannuleerd. Beide handles sluiten; latere begin/poll publiceren niets. De probe blijft leven tot de dienst de aanvraag werkelijk verwerkt heeft. |
| 2 SDK/uncertain | De testtransportadapter voert echte native.send uit en geeft alleen ná succes opzettelijk InvalidReturn terug. Dit is expliciete caller-foutinjectie. De SDK moet Unknown melden, beide handles sluiten, geen bytes publiceren en niet opnieuw verzenden. Geen vervanging van een OS-dienst door een hostresultaat. |
| 3 Verkeerde PID | De RND-descriptor verwacht de bestaande app-PID; de probe verstuurt echt als eigen PID. Seen=1/badCaller=1 en calls/bytes/seeds=0 bewijzen daadwerkelijk consume+weigering. Daarna bevestigt de caller minstens10echte ticks zonder antwoord; stilte alleen volstaat niet. |

Bij cancel/uncertain mag de reply vóór de annulering al geënqueued zijn, of
pas bij Closed falen. Intrekking kan reeds rechtmatig geproduceerde bytes niet
ongedaan maken. De SDK geeft de onbeantwoorde output niet aan de caller.
Case0 veroorzaakt een gesloten-replyfout deterministisch, zonder die race.
De bestaande systeemapp moet in ieder geval zijn2seals/2unseals/9authweigeringen
en de4096byte plaintext-diskfixture blijven doorlopen. Deze diskfixture bewijst
geen versleutelde opslag.
Na de terminale RND-observatie voert de bestaande app bovendien een **nieuwe
echte legacy-STATUS-RPC** uit met haar al bestaande, enige levende KeySession.
De ontvangen toestand moet Ready zijn. Er worden geen extra KEY-rechten aan
de RANDOM-probe verleend en STATUS mag geen entropy consumeren.

Losse nulbytes en herhaalde korte waarden zijn geldige randomoutput. De proef
eist alleen voor samples vanaf32bytes een aanvullende niet-allesnul/
verschilcontrole. Dat is geen entropieschatting, statistische veiligheidstest
of fysieke RNG-kwaliteitsclaim. De implementatie moet de echte virtio-rng en
onderhouden ChaCha20Rng gebruiken; er is geen host-/CPU-RNG-fallback in de probe.

## Startup, hooks en tellers

De probe gebruikt de bestaande private1024byte-startuppagina:
32=versie1,40=resultaat,48=RND-SEND,56=reply-RECV,112=case0..3,
160=KeyPID,176=eigenPID. Resultaat1 plus nativeexit0 betekent probegeslaagd;
240 geeft bij falen de stap. Meetwaarden staan in400..584.
Fase640=request,648=ack,656=nieuwSEND; deze cellen zijn uitsluitend de
opt-in testfixture. Hun definities staan in
[`contract.rs`](../../sdk/examples/random-probe/src/contract.rs).

Key houdt de eerder vastgelegde descriptor:
120=`NOSRNG01`,128=RND-RECV,136=reply-SEND,144=callerPID,152=domain0.
Oude profielen zonder descriptor blijven zonder RND-Sys3-aanroepen werken.
Aanvullende Key-observatiewoorden:

| Woord | Betekenis |
|---|---|
| 272/280/288 | werkelijke corecalls/bytes/seeds |
| 296/304/312 | native ingressmessages gezien / verkeerdecaller / verkeerdeopcode |
| 320/328/336 | RNDingress retired / replypending / fout requestformaat |
| 344/352/360 | werkelijk geënqueuede replies / Closed-fouten / pendingdeadline/budgetfouten |
| 368/376 | geslaagde legacyreceive/poll-returns(Ok/EAGAIN) / onverwachte legacypollfouten |

De requestcounters stijgen pas na echte native.receive. Pending wordt vóór
de bijbehorende outputcounters gepubliceerd. Observerfasen wachten op een
volledige terminale combinatie; losse userlandwoorden zijn niet atomair ten
opzichte van timerpreëmptie.
Na de terminale RND-observatie wacht de fixture bovendien op minstens twee
nieuwe echte legacypoll-returns en nul fouten. Zo bewijst de native trace
voortgang na Closed/cancel/uncertain/verkeerdecaller; een heartbeat in een
RND-retrylus telt niet. Een nieuwe succesvolle legacy-RPC-roundtrip ná die
fout wordt afzonderlijk door de bestaande app uitgevoerd; haar eerdere2/2/9
of alleen deze pollcounter is daarvoor geen vervangend bewijs.

Alleen dit profiel vult in de bestaande app624=`NOSRNGT1`,632=release,
640=STATUS-ACK,648=begin- en656=eindticks. Magic0 houdt alle oude gevallen
zonder extraRPC/Sys3-pad. De fixture zet release pas na daadwerkelijk
geconsumeerde/verwerkte RND-input of daadwerkelijke verkeerdecallerweigering.
De app wacht begrensd met Sys3, gebruikt begin/poll op haar bestaande sessie
voor STATUS met een echte tickdeadline en sluit die sessie pas daarna.
De fase-ACK naar de probe volgt pas op de werkelijke STATUS-ACK van de app;
de oude app kan daarbij al zijn afgesloten, waarna de observer haar snapshot gebruikt.
Documentopslag, keyrevoke,2/2/9 en diskrequestID's blijven gelijk.
De observer bewaart dit bewijs ook vóór appcleanup en vereist
`terminalTick <= STATUSbegin <= STATUSeinde <= appExitTick` en ACK1.

Bootstrap roept `random_fixture::bind(s,e,Binding{case,key_pid,probe_pid,
random_root,send_ancestor,send_leaf,reply_receive})` na registratie/grants/
startup. Daarna zijn nodig: `poll(s)`, `is_probe(s,slot)`,
`observe_exit(s,slot)->bool` vóór cleanup bij **alle** exits: de appstatus wordt
opgeslagen met returnfalse, de probe met returntrue. Gebruik `finished()`,
dat beide snapshots vereist, als extra finishvoorwaarde en
`check_and_print(s)` vóór Keycleanup. De root houdt module-/ELFkeuze,
vijf-initiëleprocessen, featureconflicten, timeoutdiagnose, bouwmanifesten,
gedeelde locks en run-/capturevalidatie in één integratiepatch.

RAW verwacht app89succesantwoorden/4737bytes,110enqueues/109gevalideerde
replies; variables5,legacy9,invalid9,quota1,rebind/stale/replay/revoke ieder1,
capabilityweigeringen6. De Key-vector is exact:
`[90,4769,3,110,0,9,1,0,9,109,1,0]` volgens de twaalf servicewoorden.
SDKcancel verwacht5corecalls/257bytes/1seed,4positieve outputs/225bytes,
5nativeenqueues,2closes en één cancel zonderretry. SDKuncertain verwacht
1call/64bytes/1seed,1enqueue,0output en2closes zonderretry.
WrongPID verwacht exact `[0,0,0,1,1,0,0,0,0,0,0,0]`.
`RANDOM_PROBE_PID`, `RANDOM_KEY_PID`, `RANDOM_APP_PID` en
`RANDOM_EXPECTED_CALLER_PID` binden de meetwaarden aan de opgeslagen echte
procesidentiteiten; alleen WrongPID verwacht de app als caller.
Alle gevallen vergelijken echte brokerbytes met1024+256×seeds én Key264;
de oorspronkelijke crypto-, opslag-, lifecycle- en cleanupgates blijven vereist.

## Begrensde RANDOM-replies

De oude gedeelde sendhelper kon200.000EAGAIN-yields uitvoeren voordat legacy
weer aan de beurt kwam. Alleen de RANDOM-replyroute is daarom veranderd:
één nonblocking sendpoging per hoofdloop, maximaal één beheerde256bytepending
reply,4096pogingen en een geverifieerde100tickdeadline na EAGAIN. Iedere
hoofdloop behandelt eerst de bestaande sleutelingresses. Closed, andere fatale
transportfouten, tijd-/budgetfouten beëindigen alleen RNDingress en wissen
pending/outputenvelope; master, Ready en legacy-RPC's worden niet gereset.
Een geretireerd ingress wordt niet stil opnieuw toegekend.

De RAW-Closedproef en de SDKcancelcases moeten dit native uitvoeren.
Een echt volle replyqueue met EAGAIN en zijn deadlinepad zijn aanvullend nog
native te controleren; source-/hostcontrole maakt die runtime-eis niet groen.
Ontbrekende/achteruitgaande tijd en bronfalen blijven ook afzonderlijke
negatieve runtimevereisten; dit profiel maakt de echte monotone PIT-tijd niet
opzettelijk defect om daar een resultaat voor te simuleren.

## Componentbuild en bewijsgrens

`LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=<eigenmap>
python3 -B tools/dev.py python3 -B sdk/examples/random-probe/build.py`
bouwt `random-probe` metRust1.90 voorx86_64-unknown-none. De kernel ontvangt
`NOS_RANDOM_PROBE_ELF=<gecontroleerd ELF>`. Argument`clippy` voert dezelfde
native selectie met`-D warnings` uit. De probe heeft een eigen lock;
geen root-/SDK-lock is voor deze componentbuild gewijzigd.

Hostchecks vergelijken alle1..224RAW-codeclengtes met de werkelijke Key-codec,
weigeren gewijzigde replyvelden/caps en gebruiken de echte capability-engine
voor Closed/rebind/subtreeintrekking. Zij bewijzen geen native RNG-uitvoering.
Het componentrapport staat in
[`verification/component.json`](../../sdk/examples/random-probe/verification/component.json):
4 nieuwe hostproeven plus61 bestaande SDK- en15 Key-clientproeven slagen;
probe, Keydienst en systeemapp bouwen en voldoen aan Clippy voor het scalar
native doel. De disassemblies bevatten geen FP-/SIMD-register- of CPU-RNG-
instructiematches. Dit is een begrensde codegeneratiecontrole, geen uitvoeringsbewijs.
De observer wordt daarnaast in een private kopie van de echte985kernel
getypecheckt, met alleen een tijdelijke moduleverklaring en gecontroleerde
bestaande985ELFs; bootstrap/hooks worden daarmee nog niet uitgevoerd.

Voor native aftekening blijven de rootintegratie, alle vier werkelijke
gevallen, koude normale/herstelketen, exacte image-/bronhashes en regressies
verplicht. Geen volledige testrelease-, TLS-, identiteit-, certificaattijd-,
PQC-, entropiekwaliteits- of volledige geheugenerasureclaim uit deze deelstap.
