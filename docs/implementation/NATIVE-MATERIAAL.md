# TASK-NATIVE-MATERIAAL — eerste native frontendketen

## REV-APP-1 — huidige paint en juiste C-entry

De onafhankelijke appreview reproduceerde echt rood dat Ready400 na een
voltooide paint nog groen bleef bij slechts tile0 van een nieuwere generatie.
Controller.presented was zelf begrensd, maar de oude meetvlag was onjuist.
Iedere nieuwe paint trekt Ready nu in; evidence controleert ook de huidige
dirty/generatie. LastPresentedGeneration verandert uitsluitend nadat dezelfde
actuele generatie/epoch via alle echte tileACKs Controller.presented bereikt.
Ready400 wordt als laatste gepubliceerd, na eerst0 en de overige meetwoorden,
zodat een kernelpreëmptie geen halfgepubliceerde groep groen kan lezen.

De andere gerichte reviewfix vervangt de extra entry-CALL door CLD, een echte
RSPmod16==8-guard en JMP naar de nonreturning C-functie; de kernel levert al
SysV C-entryalignment. Het werkelijke ELF wordt op guardbytes en directe JMP
zonder extra CALL gecontroleerd. Negentien hostscenario's zijn groen, inclusief
de eerst rode tegenproef. Compilatie/ELFinspectie blijven geen native uitvoering.

Extra observerwoorden504screen(1Materiaal/2Editor/3Confirm),512draftDirty,
520draftLength,528draftFNV,536objectID(MAXbijgeen),624LastPresentedGeneration,
632scale(1of2) zijn door root afgestemd en uit werkelijke Controller/DocumentSession
afgeleid. Dit verandert geen grants, inputpolicy, draftinhoud of opslag.
Native build gebruikt een private bronkopie met gehashte centrale SDK1.8-inputs;
geen gedeelde SDKfiles zijn voor deze fix gewijzigd. Het afzonderlijke
`sdk/examples/materiaal/verification/review-fixed.json` bindt rood/groen/ELF.
De oudere pre-native.json blijft herkenbaar het vóór-reviewcheckpoint.

De afgebakende GUI-proef publiceert tevens640F12PressCount: uitsluitend een echt
geauthenticeerd INP1-event PhysicalKey::F12/Press verhoogt deze checked teller.
Repeat/Release of andere keys tellen niet; uitputting geeft een fout en wrapt
niet. Dit is geen shutdownfunctie of nieuwe grant. De probe mag na werkelijk
Saved→List→Reopened en een echte QMP-screenshot pas een nieuwe F12Press gebruiken
om de afzonderlijke CPL3-controller zijn bestaande StopRPC te laten uitvoeren.
Een late Enterrelease kan de afsluitbarrier daardoor niet voortijdig zetten.

## TASK-MATERIAAL-NATIVE-APP — gescheiden native app, SUR1-freeze

31ab300 bouwbasis: main985ed4d plus frontend be0/d5 en display/input b2ad.
Eigenaar native_ui_apps: ui/surface-protocol, sdk/input-client, nieuw voorbeeld
sdk/examples/materiaal en dit document. Root bezit afzonderlijke compositor,
bootstrap/centraleABI/build/workspace-locks. Gericht extra toegewezen:
ui-core mouse.rs voor een publieke checked buttonconstructor en de eigen
Controller voor een pointer-only cancelhook. Geen andere bestaande files.

Appslot3 krijgt geen DISPLAY/INPUT/Block/Entropy/Key/Network-cap. Compositor4
heeft DISPLAY, source5 INPUT en policy6 alleen eigen IPC; echte PIDs blijven
onderscheiden en komen uit kernelmetadata/startup. Init0 blijft supervisor.
App heeft drie taak-kanaalparen (volume, INP1, surface) plus bestaande supervisor-
controlRECV/registrySEND, binnen het bestaande quotum van vier receive-endpoints.

Het afgestemde [SUR1-contract](../../ui/surface-protocol/README.md) is 48B-header,
208B-chunkdata, maximaal vijf chunks per tile en één transaction-ID voor
BEGIN/CHUNK/COMMIT. Alleen complete gevalideerde COMMIT plus echte DisplayClient-
blit mag positief bevestigen. Root bouwt de ring3compositor tegen de pure
Receiver/CommitReady/commit_with(token,now_tick,callback)-interface; geen kernel-render-/GUIpolicy.
INFO meldt publieke schermgeometry en toegewezen Rect, geen fysieke base/stride.
De app tekent lokaal; compositor valideert/telt de toegewezen offset op en
beschermt zijn verplichte systeemrand. Canonieke row-major gridindices,
request/epoch/frame/offset/length/pixelchecks en timeout begrenzen staging.

Input blijft **globaal** op werkelijke GOPwidth/height. Native adapter valideert
het afzonderlijke 96B INP1-contract, geen 32Bhardware-record. Binnen assignedRect
trekt hij alleen x/y-offset af; erbuiten wist cancel_pointer alleen pressed,
drag en hover/cursor, zonder draft/keyboardfocus/generatie te wijzigen. Geen
clamping van een release buiten Save naar een klik; reentry met buttondown
maakt geen nieuwe press. MouseButtons::checked weigert onbekende bits, terwijl
de private bestaande routingconstructor behouden blijft.

Startup blijft compatibel: docEpoch32, docRecv72, docSend80, registrySend88,
controlRecv96, readonly112, volumePID168, appPID176, initPID184. Nieuwe velden:
surfaceSend544, reply552, compositorPID560, surfaceEpoch568, inputSend576,
inputReply584, policyPID592, inputEpoch600, inputClientID608, scale616.
Counters400..512/failure240 worden uitsluitend na werkelijk waargenomen
handelingen bijgewerkt, geen vooraf ingevulde Saved/Presented/Ready-proef.

Drie64KiB-callerbuffers komen in eigen BSS/RW en niet op64KiB-stack. De nieuwe
vijfsegmentlinker houdt startup40010000 en plaatst rodata+.data.rel.ro vóór de
RWwildcard in afzonderlijk R+NXgebied40020000(max256KiB), RW/BSS40070000 en
code400b0000. Dit blijft binnen bestaande userimage-/quota-/W^X-grenzen. Werkelijk
gebouwde ELF en user::parse zijn verplicht; native QMP/image/diskoracle komt
pas bij rootintegratie. Er is geen claim van persistente encrypted keys/B02,
volledig Materiaal, schermlezer of volwaardige testrelease.

SUR1-review vond en reproduceerde een bekend verlopen Ready-token na malformed
traffic. De regressie was rood op oude bron; expiry staat nu vóór decode en
een verse native monotone tick is verplicht bij commit-admission. Zonder
geldige deadline mag de callback niet lopen, geen positieve reply of frame-
completion volgen. De deadline betreft admission vóór blit, geen vsyncbelofte
over preëmptie of vermeende rollback van geschreven pixels. Bron-/red-/green-
en onafhankelijke herreview staan bij `ui/surface-protocol/verification/`.

### Afzonderlijke app-adapter en gebouwd ELF

`sdk/examples/materiaal` bevat nu de echte no_std native caller. Alle drie
taakparen worden eenmaal aan hun typed eigenaar overgedragen; app-startup
controleert geldige, verschillende handles en onderscheiden verwachte PIDs.
De app bezit geen hardwarecap en fabriceert geen Received-envelopes. Iedere
begrensde iteratie doet hoogstens vier PEEK/ACK-paren, één echte uploadstap en
één tiletransactie. Een snapshot wordt alleen gepresenteerd na alle ACKs van
dezelfde actuele generatie/epoch. Bij input-/surfaceverlies gaat Ready uit;
draftbytes blijven bewaard en een verloren COMMIT-reply geeft geen Saved.

Supervisorprotocol blijft het bestaande SYSTEM-tag: READY1 is uitsluitend
metadata/ELFready, GO2 start servicegebruik, STOP5 is expliciet supervisorverzoek,
STOP_REFUSED6 antwoordt bij dirty/saving, DONE3 volgt alleen clean/idle plus
cleanup. Exacte kernelafzender, nul caps en1Bpayload zijn verplicht. Geen stil
verwerpen, autosave of wijziging van PID-/epochautoriteit via payloads.

Bewijsvelden:400 huidige UIReady (volledig huidig paint + werkelijke INP1-response),
408 volledige frames,416 echte tileACKs,424 geleverde keys,432 muisevents,
440 laatst werkelijk bevestigde saveRevision,448 bytes,456 FNVchecksum,
464 echte inputACKs,472 fouten,480 echte saveCOMMIT-ACKs,488 supersededOffers,
496 stopweigering. Hartslag8/failure240 blijven; een unchanged-save is geen
nieuwe COMMIT-ACK. Dit zijn meetvelden, geen autoriteits- of crypto-eigenschappen.

Twintig gecombineerde hostscenario's gebruiken één echte capability-engine,
de bestaande volume-v7-service, input-core en SUR1. Getest zijn maken/bewerken/
opslaan/heropenen, annuleren vóór en na APPEND, globaal pointeroffset/outside/
heldreentry,100/200%, readonly, focusregrant met heldControl, grote invoer, I/O,
verloren input-/tile-/COMMIT-replies, stale halve frames en expliciete cleanup.
Ordered RAMsectoren en de displayrecorder zijn hosttegenpartijen; geen native
input-/scherm-/diskbewijs. Eén extra test past de echte kernel user::parse toe
op het werkelijk gebouwde x86_64-unknown-none executable, niet een fixtureELF.

Dit ELF heeft vijf monotone W^X LOADs en119 imagepagina's. Drie64KiB-buffers
staan op0x40071000 in BSS/RW+NX, niet op de stack; readonly inclfontrelro is
191684B, code84089B op dit checkpoint. De gemonomorfiseerde19781 instructies
bevatten geen SIMD/FPU of hostmallocbackend. Exacte hashes, commandologs en
werkelijk gebruikte tijdelijke centrale SDK1.8-input staan in
`sdk/examples/materiaal/verification/pre-native.json`. De tijdelijke patch wordt
vóór commit uit gedeelde SDKfiles verwijderd; root moet dezelfde centrale
SDK1.8-bron echt integreren en opnieuw bouwen. user::parse/Clippy/build bewijzen
geen native uitvoering, maximale stackdiepte of GUI-latency. QMP, daadwerkelijk
gerenderde UI, koude heropening en encrypted keypersistentie blijven open.

30 oktober 2026; basis `61ea727`. Eigenaar `native_ui_apps`: uitsluitend
`ui/native/**` en dit contract. Root bezit kernel/ABI/startupcaps/build/integratie;
`git_build_audit` onderzoekt display-/PS2-transport; geheugenagent bezit m3.rs.
Allocatorcommit `42d4bc9` blijft afzonderlijk behouden. Alle 62 volledige
releasecriteria blijven open; deze slice is geen complete Materiaalomgeving.

## Gecontroleerde basis en grens

G0 heeft echte GOP/framebufferkernelmapping en keyboardpolling, maar is een
kernelproef met uppercase debugfont, niet een gebruikersomgeving. Graphics-
bootstrap is op deze basis incompatibel met m3. `graphics::write_pixels` is al
begrensd op 80 canonieke pixels en herbruikbaar in de hardwareadapter.
`ui-core` levert echte Set1/PS2-ID0-events en focusrouting; `input-core` levert
authenticeerbare INP1-policy maar nog geen native dienstloop. Volume
DocumentSession v7 is native getest; deze frontend gebruikt die bestaande
implementatie, geen browserbroker of eigen opslagsimulatie.

De frontend bezit precies één DocumentSession en zijn endpointpaar. Geen
block-/entropy-/key-/netwerkcapabilities, geen firmware/device-I/O en geen
framebufferpointer. Een bevoegde compositoradapter bezit de displaycap en
vertaalt alleen private RAM-tegels. De UI verleent geen bevoegdheden. Een
renderer-/controller-hosttest bewijst geen native input-/displaygrens.

## Afgestemde adaptervorm (nog geen vastgesteld kernel-ABI)

- Publieke viewport: pixelbreedte, pixelhoogte en niet-nul display-epoch;
  hoogstens de bestaande 1920×1080-basis. Geen fysieke base of hardwarestride.
- Tegels maximaal 32×8, 256 little-endian u32's / 1024 bytes. Ieder pixel is
  `0x00RRGGBB`; payloadbytes B,G,R,0. Laatste rij/kolom zijn korter. De
  kerneladapter controleert het volledige rect, omvang en bovenbytes vóór de
  eerste hardwarewrite. Geen MMIO-slice naar Rust Canvas.
- Reële inputadapter valideert INP1-afzender uit kernelmetadata, tag, epoch,
  client, request-ID, canonieke velden en focusgeneratie. Pas daarna worden
  typed input-core-events aan deze frontend doorgegeven. Apps kunnen geen
  bron-/focus-/lifecyclebericht namaken. PEEK wordt pas na verwerking met exact
  het ontvangen eventnummer geACKt; een onsuccesvolle send consumeert niets.
- Focus-/bron-/queuecancel beëindigt drag/modifierinteractie, maar bewaart draft
  en selectie. Een nieuwe focusgeneratie erft geen vastgehouden actie. Een
  service- of display-epochwisseling vereist nieuwe grants/volledige repaint;
  een oud gepresenteerd frame mag nieuwe damage niet wissen.
- `Controller::rebind_input(new_epoch)` is een expliciete adapterstap na nieuwe,
  onafhankelijk geauthenticeerde inputdienstgrants. Zij behoudt DocumentSession,
  draft, selectie en display-epoch, wist alleen oude inputnummering/klikdrag en
  wacht op een werkelijk ontvangen geldig event uit de nieuwe epoch. Geen
  endpoint/capability wordt hierdoor gemaakt; oude PEEK/ACKs blijven afgesloten.
  Behoud van fysieke held-keykennis bij een echte dienstherstart blijft de
  afzonderlijke source-/supervisoracceptatie, niet een claim van deze methode.
- Een frame leest alleen eigen, onveranderlijk geleende viewdata en maakt
  private tegels. Alleen bevestiging van dezelfde framegeneratie én viewport-
  epoch kan damage aftekenen. Een gefaalde/onderbroken blit houdt damage open.

## Eerste gebruikersketen en echte documenttoestanden

Materiaal toont resultaten van de werkelijke volumelijst, opent hetzelfde
object in de tekstbewerker en bewaart via DocumentSession begin_save/save_next.
Drie callerbuffers blijven exclusief eigendom van die sessie. De eerste native
fixture gebruikt 4096 bytes; de core voegt geen permanente kleinere grens toe
aan de bestaande 64-KiB-volumegrens. Grotere callerbuffers vereisen een
bevestigde eigen arena en kunnen niet stil op de 64-KiB-userstack worden gezet.

Caret/selectie bewegen op UTF-8-codepointgrenzen; documentbytes worden niet
genormaliseerd of vervangen door fontfallbacks. De eerste fysieke layout is
US-Set1; invoermethoden/grapheme shaping/andere layouts blijven open. Alleen
werkende acties worden getekend: openen, nieuw, terug, opslaan en echte cancel.
Geen voorgeladen objectlijst of succes op basis van alleen een klik.
Dirty sluiten vraagt bewaren/verwerpen/annuleren. Tijdens streaming-save zijn
edits geblokkeerd, voortgang komt uit echte APPEND-bevestigingen, annuleren
gebruikt ABORT. Alleen Saved{revision} mag de opslagstatus bevestigen. Conflict,
volle opslag, denial en onzekere commit behouden de draft. Onzeker werk wordt
niet automatisch opnieuw verzonden. Koude versleutelde opslag blijft open tot
de afzonderlijke persistente sleutel-/B02-keten gereed is.

## Onderhoudbaar ontwerp, tekst en toegankelijkheid

Native kleurtokens/radii/spacing lezen compile-time de ene bestaande bron
`ui/prototype/tokens.css`; geen tegenstrijdige tweede kleurentabel of font-
netwerkverzoek. Typografiematen en control-/focusregels volgen PRODUCT-SPEC.
100/200% schalen veranderen geometrie en tekst samen; focus/status krijgen
semantische rollen en zichtbare labels, dus kleur is niet de enige betekenis.
Deze slice animeert niet. Dat respecteert reduced-motion, maar bewijst nog
geen systeembrede instelling of screenreaderwerking.

De expliciet goedgekeurde backend is `noto-sans-mono-bitmap = 0.3.2`, defaults
uit; regular, size16/32, Basic Latin, Latin1 Supplement en Specials. Geen
runtime-dependencies, buildscript, allocator of floats. Rust-MSRV 1.56.1;
gepubliceerd 7 januari 2026. De daadwerkelijke tarballchecksum is
`f102c477fd4ec864c149803bf606ecc23bc9329602ec0f99073a728abf454abc`.
Library MIT; de oorspronkelijke Noto Sans Mono Regular 2.014 is OFL1.1,
copyright 2022 Noto Project Authors. Font-SHA:
`56134342e3d5137fbfcb25361039d4a98def7df75db7dd821ef47c3ae03127b6`,
herleidbaar tot upstreamgit `636fb222507237549c20e9b23dae54fc9d639ae3`.
Beide notices blijven onder ui/native/licenses behouden. Dit is een begrensde
eerste fontbackend, geen definitieve huisstijl of volledige Unicode-engine.
Ontbrekende glyphs zijn herkenbare vervangingstekens; opgeslagen UTF-8 blijft
intact. Het oude uppercase G0-font wordt niet voor gebruikersdocumenten benut.

Primaire bronnen: [crate](https://crates.io/crates/noto-sans-mono-bitmap/0.3.2),
[upstream](https://github.com/phip1611/noto-sans-mono-bitmap-rs),
[Noto-OFL](https://github.com/notofonts/latin-greek-cyrillic/blob/main/OFL.txt),
[bestaande productafspraken](PRODUCT-SPEC.md), [invoercontract](NATIVE-INPUT-SERVICE.md)
en [GOP-basis](GRAPHICS-G0.md). Broncontrole op 30 oktober 2026 is geen audit.

## Bewijsplan en open vervolg

Gerichte hosttests: UTF-8/range/selection, echte service + capability-engine
achter DocumentSession, save/ABORT/conflict/onzekere commit/I/O-fout, geen valse
Saved, guards/clipping/kleurvolgorde van werkelijke rendertegels, focus en
100/200%-layout. Daarna Clippy/fmt en daadwerkelijke scalar none-build.
Root integreert echte ring3processen/caps, QMP-toets/muis, gerenderde voor/na-
beelden én bedieningsassertions, diskoracle/coldreopen en exact imagebewijs.
Schermlezer, alle zeven routes, compositorherstel, volledige editorfuncties,
cross-appklembord, persistente keys en de rest van de release blijven open.

## Eerste uitgevoerd checkpoint

`python3 -B ui/native/verify.py` is uitgevoerd met Rust1.90.0: 14 hostscenario's,
Clippy, fmt, featuregraph en daadwerkelijke scalar `x86_64-unknown-none`-build
slagen. De negen gebouwde rlibs zijn via echte Cargo-artifactrecords gebonden
en met objdump/nm gecontroleerd; geen SIMD/FPU-instructies of hostmallocbackend
in die objecten. Generieke controllermethoden worden pas in de definitieve
native caller volledig gemonomorfiseerd; dit is geen ring3-uitvoering.
Exacte bron-/input-/artifactbinding en commando's:
[`ui/native/verification/checks.json`](../../ui/native/verification/checks.json).

De grote bestaande capability/volume-hostharness overschreed eerst de kleine
standaard testthreadstack. Dezelfde expliciete 16-MiB-hostteststack als het SDK-
harnas verhelpt dat; de native 64-KiB-stack wordt niet verruimd of afgetekend.
Een eerste compile vereiste type-/APIcorrecties in de nieuwe tests. Op het
bovenstaande checkpoint zijn de gerichte controles werkelijk opnieuw geslaagd.

Native integratie vraagt een eigen app-linker: de fontpointertabellen staan in
`.data.rel.ro` en moeten R+NX blijven. De oude systemapp-linker zet `.data.*`
in één RW-pagina en laat maar48KiB rodata toe; dat past niet. Gemeten vóór
definitieve link: circa146KiB relro +35KiB rodata +27KiB frontendcode. Een
begrensd eigen readonlysegment van bijvoorbeeld256KiB past binnen het reeds
toegestane userimagewindow en vraagt geen kernel-/capabilityverruiming. Definitieve
residentie/stack/latency blijven te meten op de echte gekoppelde UIapp.

De algemene `tools/common.py::source_hash` volgt ui/ nog niet. Eigen bewijs
bindt ui/native en ui/prototype/tokens.css daarom aanvullend. Root moet de
native image-/capture-inputbinding uitbreiden vóór UI-acceptatie; anders kan
oude UIcode onder een ongewijzigde algemene bronhash onterecht actueel lijken.

## Vervolg: werkelijk bekeken rasters en inputherbinding

Na checkpoint `be0dc462` zijn de daadwerkelijke hostpixels van Materiaal,
bewerken en dirty-close op 100/200% en 320 logische pixels bekeken met view_image.
Dezelfde renderer voedt daarbij private tegels; de fixture gebruikt echte
volume-/capability-engine-aanvragen met synthetische testdocumenten. Dit zijn
geen native screenshots, geen gebruikersproef en geen productacceptatie.

De inspectie vond een verdwenen eerste lijstregel op minimale hoogte en een
modal met halfbedekte achtergrondtekst. De compacte header laat nu echte
lijstinhoud zien, lange titels houden ruimte zonder overlappende metadata en
een opaque lokale backdrop isoleert de bevestiging visueel. Primaire,
secundaire, gevaar- en hoverstaten gebruiken dezelfde tokenbron. Pointer en
PageUp/PageDown kunnen echt door de volumelijst pagineren; kleine viewports
maken latere objecten niet uitsluitend via Tab bereikbaar.

De eerste controller-/text-/snapshotbron is read-only door kernel_storage
beoordeeld: 14 scenario's in een eigen target herhaald, vijf bronhashes bleven
gelijk, geen concrete blocker binnen die scope. Zijn rebind-APIbevinding is
in het hierboven beschreven vervolg verwerkt. Die AI-review op `be0dc462`
is niet automatisch een review van alle vervolgcode. Nu slagen 18 gerichte
hostscenario's, Clippy/fmt en de nieuwe scalarbuild; `verification/checks.json`
bindt de actuele bron. `verification/host-raster-review.json` bevat alleen
hashes/waarnemingen van de werkelijk bekeken hostbeelden en de bewijsgrens.
