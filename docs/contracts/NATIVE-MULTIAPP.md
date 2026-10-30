# Native wisselen tussen Materiaal en de rekenmachine

MULTIAPP-CONTRACT-1, **voorstel voor de volgende implementatiestap**.
Broncontrole: GUI `48b51878c03b5b73449f563735c7854b24926fc4` en
rekenmachine `0254206f9b44c75499cf55a9c3716fdb363372ce`. Dit document voegt
geen uitvoerende code of acceptatiebewijs toe. Root heeft de richting van
afzonderlijke owners, Pause/Resume en echte focus-/framebevestiging aanvaard
en de64B-APPCTRL2/APC2-v1-codec voor implementatie goedgekeurd. Metadata en
consumer-bronverdeling moeten vóór hun implementatie gezamenlijk worden
bevroren. Root-product integreert
intussen GUI en foundation; die nieuwere bron moet bij hervatten opnieuw
worden gecontroleerd.

Doel is één bruikbare verticale keten: een document bewerken, via de echte
launcher de echte rekenmachine gebruiken en naar hetzelfde onopgeslagen
document terugkeren; daarna opslaan en koud heropenen. Beide apps draaien
als afzonderlijke CPL3-processen op de eigen kernel. De bestaande
[Materiaalproef](NATIVE-MATERIAAL-PROBE.md),
[init](NATIVE-DESKTOP-INIT.md),
rekenmachinecontract `docs/contracts/NATIVE-CALCULATOR.md` op0254206 en
[volledige acceptatie](../FULL-TEST-RELEASE.md) blijven verplicht.
Dit tekent FR-U01/U02/U04/U05, FR-A02/A08 of de volledige release niet af.

## Werkelijke basis en gerichte afhankelijkheden

| Onderdeel op de gecontroleerde bron | Hergebruik | Nog noodzakelijk |
| --- | --- | --- |
| `kernel/src/m2/m3/desktop.rs` | Echte endpointleaves, afzonderlijke deviceprincipals, private startupmetadata, gecontroleerde ELF-inbedding | Eén extra app, eigen kanalen/identiteit, nieuwe bounded manifestvariant en echte lifecyclemeldingen |
| `sdk/examples/desktop-init/src/{boot,wire,supervisor}.rs` | Ready-gates, geauthenticeerde focus-RPC, STOP/refusal en deadlines | App-tabel en wisseltoestanden; huidige bootdecoder bindt één app en exacte PID4/5/6/7 |
| `sdk/examples/compositor/src/{boot,control,surface,main}.rs` | DISPLAY uitsluitend hier, echte blit, deadline, sendercontrole en vaste replyroutes | Twee oppervlaktes, één gedeelde display-eigenaar en één foregroundcoördinator |
| `ui/surface-protocol/src/{lib,client}.rs` | Bounded SUR1-assembler, privé CommitReady, echte ACK na callback, strikte framegrid | Geen wirewijziging nodig; wel buiten de codec een foregroundtoelatingsgrens |
| `services/input-core` en `sdk/examples/input-service` | Reeds maximaal8 PID-gebonden clients, afzonderlijke bron/focus/lifecycle-ingress, PEEK/ACK, held-key-/buttonrouting | Manifest bindt nu1 client; launcherentry en werkelijke ClientDead/SourceDead/CompositorDead-afhandeling aansluiten |
| `sdk/examples/materiaal` en `ui/native` | Echte DocumentSession, drie64KiB-buffers, UTF-8-concept en actuele PresentedGeneration | Expliciete pause/resume-adapter, behoud van model, opnieuw volledig schilderen |
| Calculator `0254206` | Echte decimale kern, model, renderaar, INP1/SUR1 en zes beperkte handles | Integreren, pause/resume-adapter en native bediening bewijzen |

De huidige `Surface<D>` bezit tegelijk één `DisplayClient<D>`, één
`Receiver`, één `peer` en één replypaar. Twee exemplaren mogen dus niet door
het kopiëren van hetzelfde DISPLAY-handle worden gemaakt. Verplaats de ene
display-eigenaar naar de compositorloop; iedere app-entry bezit alleen haar
eigen receiver, kanaalpaar, pendingreply en tellers. De loop leent de display
uitsluitend gedurende een gevalideerde foreground-COMMIT-callback.
De huidige loop roept `surface.step` ook in State::Inactive aan; die status is
dus op zichzelf nog geen nieuwe foregroundschrijfgrens. Die toelatingscontrole
moet werkelijk vóór iedere displaycallback komen. Iedere iteratie heeft een
vast control-/IPC-/tegelbudget en eerlijke rotatie; achtergrondflood mag de
launcher, foregroundapp of kritieke focusreply niet onbeperkt verdringen.

`Receiver::retire()` sluit permanent; `Surface::retire()` sluit ook de twee
endpoints. Het is geen tijdelijk verbergen. De typed SUR1-client sluit bij
een negatieve of onzekere reply. Beide huidige apps schilderen zelfstandig
zodra hun model dirty is. Alleen de inputfocus veranderen en daarna gewone
achtergrondtiles weigeren zou hun sessies verbreken. Daarom is het volgende
pauzeprotocol een noodzakelijke integratievoorwaarde.
Ook `Surface::clear` eist nu een retired owner. Gewone wissels hebben daarom
een afzonderlijk compositor-eigen, bounded wipepad via de ene DisplayClient
nodig; de appreceiver wordt daarvoor niet definitief gesloten.

## Begrensd ontwikkelmanifest en bevoegdheden

Het eerste nieuwe profiel start acht echte processen: init0, Key1, Volume2,
Materiaal3, compositor4, inputsource5, inputpolicy6 en calculator7, waarbij
de getallen **slots zijn, geen runtime-identiteiten**. Een optionele eigen
proefcontroller krijgt slot8. De bestaande kernlimiet is16; geen quota- of
adresruimteverruiming is hiervoor toegestaan zonder afzonderlijke reden en
bewijs. Het oude zeven-/achtprocessenprofiel blijft afzonderlijk testbaar.

Deze vaste lijst is een geïntegreerde ontwikkelstap, geen uiteindelijke
appbeheerlimiet, installatiebron of vervanging van het verplichte pakket-
en rechtenbeheer. Het activeren van een reeds gestart proces is nog geen
bewijs van algemene appinstallatie of dynamisch starten.

| Principal | Precies benodigde taakrechten |
| --- | --- |
| Materiaal | Bestaand eigen volume-SEND/reply-RECV; eigen SUR1-SEND/reply-RECV; eigen INP1-SEND/reply-RECV; registry-SEND en control-RECV |
| Calculator | Eigen SUR1-SEND/reply-RECV; eigen INP1-SEND/reply-RECV; registry-SEND en control-RECV, dus de bestaande zes leaves |
| Compositor | Eén DISPLAY; eigen per-app SUR1-RECV/reply-SEND; bestaande policy-Focus-SEND/reply-RECV; eigen launcher-INP1-SEND/reply-RECV; Init-control-RECV en registry-SEND |
| Inputsource | Uitsluitend INPUT en bestaande beperkte IPC-kanalen |
| Inputpolicy | Bestaande vier ontvangstroutes en vaste reply-SENDs; geen DISPLAY/INPUT/devicecapability |
| Init | Registry-RECV; twee app-control-SENDs; compositor-control-SEND; eigen policy-lifecycle-SEND/reply-RECV; alleen bij het proefprofiel het afzonderlijke controllerpaar |

Geen app krijgt Focus-, lifecycle-, launch-, clipboard-, DISPLAY-, INPUT-,
Key-, entropy-, netwerk- of blokrechten door het wisselen. Calculator krijgt
geen documentgrants; zijn bekende documentstartupwoorden blijven0. Gedeelde
INP1-appingress mag twee afzonderlijk verleende SEND-leaves ontvangen, maar
de kernelafzender bepaalt de clientbinding en iedere client heeft een eigen
reply-RECV. Geen app ontvangt de RECV-leaf of reply-SEND van de andere app.

Nieuwe bootdecoders krijgen een herkenbare versie2 met bounded tabellen:
init en compositor precies2 apprecords; policy precies3 clientrecords
(Materiaal1, calculator2 en systeemlauncher3). Ieder apprecord bindt AppID,
werkelijke PID, instancegeneratie, surface-epoch en vaste kanaalleaves.
De compositor is als launcherclient dezelfde PID als de compositorrol,
maar heeft een **andere ingressrol en andere requestteller**. De bestaande
inputcore onderscheidt die rollen al op endpoint plus kernelafzender.
Layout/offsets en codec-eigendom worden door root vóór de codefreeze vastgelegd;
ongebruikte/reserved woorden blijven0. Bestaande magic1-decoders worden niet
stil verruimd om nieuwe willekeurige metadata te accepteren.

Per live appinstantie is er een afzonderlijk niet-nul surface-epoch. Alle
bindings omvatten ook de echte PID en instancegeneratie; een AppID is slechts
een index in een vooraf geautoriseerde tabel. Herstart krijgt nieuwe PID,
instancegeneratie, surface-epoch en endpointpaar. Een policyherstart krijgt
een nieuw policy-epoch en nieuwe inputparen; oude PEEK/ACK wordt niet herhaald.
Tellers lopen nooit rond. Uitputting sluit de betreffende overgang zichtbaar
af. Een koude boot maakt alle oude lokale handles/queues ongeldig; een op
schijf bewaard numeriek epoch verleent geen nieuwe bevoegdheid.

## Echte launcher, toetsenbord én muis

De compositor tekent een echte `Gereedschappen`-knop in haar eigen chrome en
een selector met precies de twee beschikbare programma's. De selector heeft
Tab/ShiftTab, zichtbare focus, Enter/Space, Escape en dezelfde echte down/up-
hitregels als de apps. De gekozen appnaam en actieve toestand zijn leesbaar
zonder kleur. Een label is geen actief bedienelement voordat zijn pad werkt.

De huidige policy heeft nog geen systeemlauncherroute. Voeg die alleen toe
aan het nieuwe expliciet geconfigureerde profiel:

1. De launcherroute wordt pas armed na de eerste geauthenticeerde, geslaagde
   compositor-Focus-aanvraag na de bestaande Source/Policy/Comp-ready-gates.
2. Een echte F10-**Press** uit de gevalideerde PS/2-bron, of een echte nieuwe
   linkermuis-Down binnen de vaste trusted launcherknop, verplaatst focus naar
   systeemclient3. Die private bronroute cancelt het oude token met de bestaande
   held-key-/buttoninvarianten en levert het oorspronkelijke event aan client3.
   Repeat, een oude release, payload van een app of een startupwoord kan dit
   niet aanvragen. Geen nieuwe algemene Focus-operatie voor apps.
3. De compositor ontvangt uitsluitend haar eigen gewone typed INP1-offers,
   niet een kopie van alle invoer. Zij ACKt die werkelijk. F10-Press opent via
   dezelfde Init-wisselroute als de muisknop; muisopening vereist daarna een
   echte Up op dezelfde knop. Up buiten de knop cancelt die actie. Een drag
   die in een app begon, reentry met heldbutton of alleen hover opent niets.
4. Een keuze in de selector levert een geauthenticeerde intent van de
   compositor aan Init met alleen AppID1of2. Init gebruikt de vaste tabel;
   de payload mag geen PID, endpoint, programmapad of nieuwe rechten kiezen.
   Escape gebruikt dezelfde route om naar de vorige levende app terug te gaan.

De policy krijgt de trusted knoprect uit het gecontroleerde bootmanifest,
controleert overflow/GOP-grenzen en dat deze buiten beide apprects ligt.
Appcode kan deze rect niet zetten. Uitbreiding naar andere trusted
launcheracties gebruikt exact dezelfde intent- en overgangscontrole; F10 is
een extra bediening, geen vervanging van verplichte muisbediening.

Voorgesteld nieuw schaalbaar chrome: boven64, onder16 en links/rechts16
**logische** pixels; assignedRect is fysiek
`(16s,64s,W-32s,H-80s)`, s=1of2. Een44-logische-pixelknop met zichtbare
focusring past zo echt. Zowel compositor, policyhitrects, renderer als
app-INFO moeten dezelfde gecontroleerde geometrie gebruiken. De oude
48-physical-pixeltopbar is daarvoor bij200% ontoereikend.
800×600/s1 is het eerste ontwikkelprofiel; 200% vereist minstens de werkelijke
viewportgrenzen van beide apps en dus een passende GOP-mode. De huidige
800×600-loader bewijst de voorgestelde200%-geometrie niet. VM-FR1280×720 en
1920×1080/200% blijven zichtbaar open totdat echte selectie en bediening
zijn bewezen; geen fallback naar100% of verborgen controls om te slagen.

## Overgangsprotocol en echte voltooiing

Init serialiseert maximaal één wissel of STOP. De voorgestelde nieuwe
app-controlcodec is64B, caploos, tag `APPCTRL2`: `APC2`@0, versionu16=1@4,
opcodeu16@6, requestIDu64@8, BootGeneration@16, AppInstanceGeneration@24,
ActivationGeneration@32, SurfaceEpoch@40, FrameGeneration@48, statusu64@56.
Alles little-endian. RequestID is niet0, strikt monotoon per controlpaar en
wordt niet hergebruikt. ActivationGeneration is niet0 en stijgt bij een
nieuwe wissel; een vervolgfase binnen dezelfde wissel echoot die generatie.
Pause1 en Resume2 requests hebben frame/status0;
replies echoën alle bindings met opcode|0x8000. Pause-reply heeft frame0;
Resume-reply vermeldt het werkelijk volledig gepresenteerde nieuwe frame.
Status0 betekent alleen de hieronder omschreven voltooiing. Gereserveerde
opcodes/statussen of verkeerd gebonden replies falen. Init en app gebruiken
hun vooraf verleende control-/registryroute en echte kernelafzender.
De eerste codecversie reserveert alle APP-statussen buiten0; een adapterfout
wordt geen positieve completion. Tellers gebruiken1..=u64::MAX-1 en lopen
niet rond. De typed replyvalidator krijgt de werkelijke eerdere frame- en
activationwatermarks van deze binding mee en weigert een oude completion,
ook wanneer de overige velden kloppen. De codec beheert geen transport of
werkelijke blits; die noodzakelijke adaptervoorwaarden blijven afzonderlijk.
De gedeelde registry demultiplext eerst op echte kernelafzender/tag/fase;
een bericht van appB wordt niet bij appA's uitstaande reply ingeleverd.
Malformed verkeer van de verwachte peer kan diens eigen lane afsluiten,
maar een andere principal krijgt daarmee geen globale annulering.

De concrete launcherintentcodec voor compositor→Init is eveneens64B, tag
`DESKSWC1`: `DSW1`@0, versionu16=1@4, opcodeu16=Switch1@6,
requestIDu64@8, BootGeneration@16, TargetAppID@24,
ActivationGeneration@32, SurfaceEpoch@40, FrameGeneration@48, status@56.
RequestID en BootGeneration zijn niet0; TargetAppID is uitsluitend1of2 uit
de vooraf verleende boottabel; de overige vier requestwoorden zijn0.
Init controleert de verwachte compositor-kernelafzender, tag, caps0, exacte
lengte, bootgeneratie en replayteller. De request kiest geen nieuwe rechten.
Een canonieke reply heeft opcode0x8001 en echoot request/boot/target.
Status0 bevat uitsluitend de werkelijk voltooide activation/surface/frame-
generaties, alle niet0 en tegen de geselecteerde appbinding gecontroleerd.
Busy1, Denied2 en Failed3 hebben die drie resultaatwoorden0. Andere status,
een gemengde positieve/foutvorm of een reply uit een andere fase faalt.

De Init↔compositorcodecs krijgen een afzonderlijke versie met dezelfde
request/boot/activation-binding en AppID uit de vaste tabel: Quiesce,
ShowLauncher, Activate en Retire. Details van die ene gedeelde codec worden
door de protocolowner bevroren vóór implementatie. Geen twee schrijvers die
afzonderlijk opcodes of metadata verzinnen. Het bestaande CMP1 Retire-antwoord
wordt niet als een nieuw appselectieantwoord geïnterpreteerd.

| Toestand/handeling | Verplichte uitvoering vóór volgende toestand |
| --- | --- |
| Running(A) → launcherintent | Policy geeft systeemclient focus en echte cancel aan A; Init start één Quiesce/Pause(A), nieuwe activationgeneratie |
| Pausing(A) | A rondt hoogstens de al gepubliceerde bounded SUR1-tile-RPC werkelijk af en ACKt echte canceloffers; daarna geen nieuwe invoer-/render-RPC. Compositor blijft alleen die lopende tile afhandelen vóór de pausebarrière |
| Paused(A) | App-Pause-ACK én compositor heeft geen staged tile/pendingreply van A; begin/chunk/commit uit A kunnen daarna nooit een displaycallback krijgen. INFO zonder write mag alleen volgens de vaste eigen binding |
| Launcher | Compositor wist werkelijk het appgebied en tekent selector; geen pixelcache van een app als nieuwe app presenteren. Systeemclient bezit de inputfocus |
| Activate(B) | Init valideert keuze en levende binding; compositor activeert alleen B, zet echte policyFocus(B) en wacht op de geauthenticeerde ACK; daarna Init-Resume(B), of eerste GO voor nog niet gestarte frontend |
| Resuming(B) | B maakt een nieuwe rendergeneratie, begint tile0 en schildert het hele eigen rect via de bestaande SUR1-client. Compositor laat alleen B's callbacks toe en controleert de volledige nieuwe framegrid |
| Running(B) | B's gebonden Resume-ACK noemt het actuele volledige frame én compositor bevestigt echt `completed_frame > completed_at_activation` na alle blits. Pas daarna toont chrome B als gereed en volgt switch-success |

De laatste rij geldt ook voor eerste GO: StartupReady/SYSTEM1 of sendOk van
GO bewijst geen geschilderd frame. Een half frame geeft alleen werkelijke
voortgang, geen Ready/Presented. Surface-ACKs volgen nog steeds uitsluitend
na `DisplayClient::blit`; `commit_with` krijgt een nieuwe echte monotone tick
vlak vóór de callback. Geen positieve ACK voor verborgen opslag, verworpen
tegels, een verlopen token of een anders geselecteerde app. Geen vsync- of
atomaire-frameclaim: de overgang mag zichtbare bounded tekenvoortgang hebben.

SUR1-sessie en requestteller blijven bij een gewone switch leven. Resume moet
de model-/rendergeneratie strikt verhogen: dezelfde oude framegeneratie opnieuw
bij tile0 aanbieden wordt door de huidige client en receiver terecht geweigerd.
Voeg hiervoor een expliciete gecontroleerde render-invalidationhook toe.
Misbruik geen documentedit, resize naar andere maten, gefabriceerd inputevent,
nieuwe SaveACK of input-epochwissel om repaint af te dwingen.

EAGAIN vóór enqueue bewaart dezelfde bytes en bounded deadline; na enqueue
wordt uitsluitend de reply ontvangen, nooit automatisch opnieuw gepubliceerd.
Onzekere levering, verkeerde afzender, stale binding, ontvangen caps,
timeout, klokterugloop of telleruitputting kan geen switch-success opleveren.
Sluit onverwachte ontvangen caps. Bij onzekerheid: geen foregroundapp-write
toestaan, systeemstatus/fout via echte compositor en concepten behouden.
Niet vanzelf de oude app hervatten alsof een onzekere overgang niet plaatsvond.

Een offer kan al door de oude app verwerkt zijn voordat een focuswissel de
latere ACK met StaleAck/Superseded weigert. Dat is geen retroactieve undo van
de al verwerkte toetsaanslag of bewerking. De adapter telt geen ACK, verwerkt
dat offer niet opnieuw, cancelt captures en consumeert daarna de werkelijke
nieuwe cancellation. Een verloren reeds geconsumeerde ACK sluit het eigen
paar zonder replay. De nieuwe app krijgt nooit de oude key/buttonactie.

## Concepten, afsluiten en definitieve levensduur

Pause bewaart Materiaal's DocumentSession, drie eigen buffers, object/revisie,
UTF-8-concept, caret/selectie en dirty/uncertain/conflictstatus. Het start geen
save, abort, discard, refresh of close. Een al door de gebruiker gestarte save
mag in de achtergrond bounded verdergaan; de echte uitkomst blijft bij dezelfde
sessie en krijgt alleen na een echte commitreply SaveACK. Een update daarvan
wordt bij terugkeer volledig opnieuw gerenderd. Calculator bewaart expressie,
selectie, vorige uitkomst en fout-/afrondingstoestand. Beide clearen captures,
drag en hover; held modifiers worden niet tot nieuwe appacties vertaald.

STOP is afzonderlijk, via de geselecteerde app's echte controlroute en de
bestaande fasegebonden SYSTEM5/6/3-regels. Dirty/saving Materiaal weigert met6;
dat is geen DONE, Retire, discard of toestemming om het proces te doden.
Een onzekere/conflicterende opslaguitkomst mag evenmin impliciet verdwijnen;
de nieuwe stopadapter controleert die toestand expliciet en vraagt eerst de
werkelijke gebruikers-/herstelafhandeling.
Init kan het document via dezelfde focusroute terugbrengen voor de echte
Save/Discard/Cancel-bediening. Annuleren behoudt het proces en concept. Een
achtergrond-STOP controleert dezelfde appstatus; geen eigenaarschap uit de
toevallig actieve app overnemen. Calculator sluit eerst de vier leaves van
haar twee typed sessies, stuurt DONE via de nog levende registry-SEND en
sluit daarna registry/control vóór exit0. Iedere close wordt geprobeerd.
DONE beweert dus geen reeds gesloten registry; definitieve procescleanup
vereist ook de echte exit en de kernelcontrole op overgebleven handles.

Na echte DONE of geobserveerde death: blokkeer per-app displaycallbacks,
retire alleen die SUR1-owner, sluit/revoke de betreffende eigen leaves, stuur
via Init's lifecycle-ingress werkelijk ClientDead en wacht op de policy-ACK.
`ClientDead` maakt de binding definitief dood; voor herstart zijn nieuwe
bindings nodig. De andere app en haar volume-/replykanalen blijven intact.
De bestaande single-app Coordinator stuurt bij Retire altijd Focus0; dat mag
niet ongewijzigd voor een achtergrond-app worden hergebruikt. Achtergrond-
Retire laat de andere focus en zichtbare pixels intact; alleen foreground-
Retire vereist echte Focus0 en een wipe van het getroffen zichtbare gebied.
Bij compositor-/source-/policyverlies: focus weg, captures cancelen,
verouderde staged tokens/replies ongeldig en geen verborgen hergrant.
Het huidige manifest maakt een lifecycle-RECV maar bewaart de bijbehorende
Init-SEND niet; die echte route moet dus worden aangesloten, niet uit een
RAMwoord als reeds uitgevoerd worden afgeleid.

Geometrie of verplaatsing kan alleen de compositor besluiten. Apps krijgen
geen framebufferadres/stride of globale tekenrechten. Gewone foregroundwissel
behoudt hetzelfde assignedRect; er is nog geen vrije vensterverplaatsing.
Bij echte herplaatsing/resize: pauze, oude SUR1-eigenaar sluiten, nieuwe
surface-epoch/kanalen en gevalideerde INFO, nieuwe viewport en volledige
repaint. Inputcursor blijft globaal binnen GOP; alleen inside-coördinaten
worden apprelatief. Buitenrect-Up wordt nooit naar een knop geclamped.
De toekomstige drag-/vensterbediening blijft een expliciete open FR-U02-taak.

Deze stap voert geen gedeeld klembord in. De gecontroleerde Materiaalcontroller
heeft nog geen cross-app-clipboardbroker; calculator heeft geen clipboardgrant.
Een switch kopieert dus geen concept, selectie of uitkomst naar de andere app.
Kopieer/plak en slepen tussen apps blijven verplicht open. Een toekomstige
broker moet expliciete gebruikersactie, datatype/omvang, bron/doel en ingetrokken
grants afdwingen; de launcherroute verleent daarvoor geen impliciet recht.

## Exclusieve bronverdeling en uitvoeringsvolgorde

Onderstaande verdeling is toewijzingsvoorstel; root bevestigt werkers voordat
zij schrijven. Iedere schrijver gebruikt een eigen worktree/targetmap. De
hoofdagent behoudt root/SDK-locks, kernel/ABI, buildhooks, status en rubric.

| Werkpakket, één eigenaar | Exclusieve schrijfbestanden | Afhankelijkheid en betekenisvolle controle |
| --- | --- | --- |
| MA-01 protocolowner | Nieuw `sdk/desktop-control/**` en dit contract | Eerst de bounded codecs en tabellen bevriezen; malformed/opcode/padding/PID/epoch/replay/overflow/deadline en lostreplytests |
| MA-02 compositorowner | `sdk/examples/compositor/**` | MA-01; één echte display-owner, twee assemblers, foregroundbarrière, trusted launcher/hittest, echte framecomplete en per-app-retire; hostile dubbele/verborgen COMMIT en callbackfailure |
| MA-03 initowner | `sdk/examples/desktop-init/**` | MA-01/02; twee-app-toestandmachine, geen GO/Resume vóór echte focusACK, dirtyrefusal, geen success bij halve fase, tests met echte capability-engine |
| MA-04 inputowner | `services/input-core/**`, `sdk/examples/input-service/**` | Bevroren launcherpolicy/bootmap; drie echte clientrollen, F10/knop-originbinding, hold/release/overflow/reset/FocusLost/StaleAck; bestaande legacytests behouden |
| MA-05 Materiaalowner | `sdk/examples/materiaal/**`, `ui/native/src/controller.rs` en eigen gerichte tests | MA-01; Pause/Resume zonder draft-/buffer-/documentmutatie, nieuwe rendergeneratie, save-inflight/uncertain, geen false Ready |
| MA-06 calculatorowner | `sdk/examples/calculator/**` | Geïntegreerde/reviewde0254206 plus MA-01; modelbehoud, true calc na echte input, geen nieuwe opslaggrants; eigen formattering raakt geen gepinde vendorbron |
| MA-07 rootintegratie | `kernel/Cargo.toml`, `kernel/src/m2/m3.rs`, `kernel/src/m2/m3/desktop*.rs`, `tools/build*.py`, gedeelde Cargo/locks | Alle freezes; nieuwe slot7/8rollen en echte PIDbinding, alleELFs inspecteren, minimale grants/lifecycle, oude profielen behouden |
| MA-08 verifier | Nieuw `tools/run_multiapp_probe.py`, `tools/tests/test_multiapp_probe.py` | Nieuwe gate na werkende eigen adapters; QMP, echte pixels/serial/volume, negatieve replaygevallen; geen wijziging van componenten door de verifier |

Er is geen gedeelde ui-core-herschrijving of kernel-renderaar nodig. De
inputowner behoudt de bestaande physical-down/owner-invarianten; alleen een
concreet bewezen API-gat rechtvaardigt een afzonderlijke, exclusief toegewezen
ui-core-wijziging. Een componentcommit is geen productmijlpaal op zichzelf.

## Commando's, gates en bewijsgrens

Eerst bestaande componentchecks op de samengevoegde bron; calculatorpad bestaat
pas na integratie van0254206. Gebruik de aanwezige beheerde Rust1.90-omgeving,
offline locked closures en een eigen `CARGO_TARGET_DIR` onder de checkout:

```sh
cargo +1.90.0 test --offline --locked --manifest-path ui/surface-protocol/Cargo.toml --features client
cargo +1.90.0 test --offline --locked --manifest-path services/input-core/Cargo.toml
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/input-service/Cargo.toml
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/compositor/Cargo.toml
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/desktop-init/Cargo.toml
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/calculator/Cargo.toml
python3 -B -m unittest discover -s tools/tests -p test_materiaal_probe.py -v
```

Daarbij horen gerichte Clippy/fmt, de bestaande scalar/native builders met hun
eigen linkercontract, actuele ELF/W^X/stack/quota-inspectie en de bestaande
native display-, Materiaal-, opslag-, RNG- en foundationregressies. Een
hosttransport of een doelbuild is geen native switchbewijs.

**Voorgestelde nieuwe entrypoints, nog niet aanwezig/uitgevoerd:**
`native-system-multiapp` (interactief), `native-system-multiapp-probe`
(aparte beperkte controller), `test-system-multiapp-recover` (echt readonly)
en `test-system-multiapp-scale200`. Na rootwiring wordt de normalgate:

```sh
NOS_BUILD_DIR="$PWD/build/multiapp-normal" NOS_TARGET_DIR="$PWD/target/multiapp-native" python3 -B -u tools/build.py --kernel-feature native-system-multiapp-probe
NOS_BUILD_DIR="$PWD/build/multiapp-normal" NOS_TARGET_DIR="$PWD/target/multiapp-native" python3 -B -u tools/run_multiapp_probe.py --case normal
```

Recover gebruikt een afzonderlijke build/runmap en uitsluitend de opnieuw
gevalideerde succesvolle normalouder onder die checkout. Geen willekeurige
schijfparameter, guest-RAM-injectie, hostdecoder of hostberekening als antwoord.
De bestaande200s-Materiaalbenchmark wordt niet verruimd door dit contract;
de multiappscenario's krijgen vóór hun eerste uitvoering een eigen vaste
deadline en gemeten frame-/wissellatentiegrenzen. Grenzen blijven bij falen
zichtbaar staan en worden niet tijdens een run gewijzigd.

| Native scenario op het uiteindelijke image | Noodzakelijk positief én foutbewijs |
| --- | --- |
| Materiaalconcept → launcher → calculator → Materiaal | Beide launchermethoden bedienen; echte expressie `(12,5+7,5)/4` geeft5, delen door0 geeft fout en correctie werkt; concept/caret/selectie/object/dirty ongewijzigd bij terugkeer; nieuwe volledige frames en echte focusACK per wissel |
| HeldControl/letter/muis tijdens wissel | Geen tekst, shortcut, knopklik of modifier lekt naar het nieuwe token; repeat/release/outside/reentry veroorzaakt geen launcher-/Save-/calculatoractie; echte volgende press werkt |
| Oude of onzekere IPC | Verkeerde app/epoch/request, andere replyroute, extra caps, StaleAck, volle queue, timeout, verloren ACK en stale CommitReady geven geen switch-/frame-/save-success |
| Achtergrond en afscherming | Achtergrondapp kan geen displaycallback laten uitvoeren, eigen frames kunnen trusted chrome niet overschrijven, calculator heeft ook na herhaalde wissel geen document/clipboard/device/launchrecht |
| Dirty en saving STOP | Dirty weigert zonder DONE/Retire; gebruiker kan echt annuleren of opslaan; reeds begonnen save blijft dezelfde transactie, disk-I/O-fout wordt geen ACK; andere app blijft bruikbaar |
| Death/retire/revoke | Alleen getroffen owner ongeldig, echte policy-lifecycleACK, nieuwe principal krijgt geen oud token/reply/frame; andere app/data behouden; veilige focus0 bij kritieke dienstuitval |
| Save → volledig afsluiten → nieuwe readonly-VM | Exacte echte documentbytes/revisie/naam uit onafhankelijke diskparser, normalouder/bron/tool/imagehashes herberekend; calculator krijgt ook daar geen opslaggrants |
| Visueel100/200%, keyboard en muis | Werkelijk gerenderde selector/appframes bekijken én bedienen; focus, contrast, lange tekst, fout-/achtergrondstatus en globale/apprelatieve hitgeometrie; geen aanname dat een screenshot bediening bewijst |

De verifier archiveert exacte commit, bronvingerafdruk, config, alle eigen
ELFs/kernel/loader, QEMU/firmware, raw exit/serial, geordende QMP-responses,
causale screenshotprefixen en echte volumevoor-/nabeelden. Per app bindt hij
PID/instance/surface-epoch/inputclient, activation- en PresentedGeneration;
een oude Ready of alleen de laatste succesfoto kan geen nieuwe wissel aftekenen.
Alle62 releasecriteria en32% blijven door dit contract ongewijzigd. Native
GUI-prestaties, versleutelde persistente sleutels/opslag, volledige launcher/
appbeheer, vensters/clipboard, toegankelijkheid, andere basisapps, NAS/AI en
updates/herstel blijven verplicht tot hun eigen werkelijke acceptatie.
