# TASK-UX-SAVE-RECOVERY — Materiaal: opslaan en herstellen

**Status:** ontwerppakket met uitgevoerde visuele deelbeoordeling van een native
probe; geen volledige runtime- of gebruikersacceptatie. **Ontwerpbasis:** checkout `ca45a7d` op
`simonvanieperen-bookish-invention`. De getekende toestanden staan in
[`ui/design/save-recovery.svg`](../../ui/design/save-recovery.svg).

## Doel en afbakening

Maak één dagelijks bruikbare, controleerbare native keten af: een document
maken/openen, tekst wijzigen, expliciet opslaan, terugkeren naar Materiaal en
hetzelfde object opnieuw openen. De dienst bepaalt of iets is opgeslagen; de
interface mag geen succes voorspellen of een onzekere mutatie automatisch
herhalen.

Dit pakket vervangt P2 niet, voegt geen route toe en wijzigt geen gedeelde
tokens, runtime, native UI-code of service-ABI. P2 blijft bevroren op zeven
schermen. De bestaande tokenbron blijft `ui/prototype/tokens.css`; bestaande
Materiaal-identiteit, Nederlandse labels en systeem-/statusgrenzen blijven
leidend. De schermschets is een ontwerpweergave, geen screenshot of claim van
een werkende native compositor.

**Acceptatie:** FR-U03 (werkelijke voortgang, annuleren en fouten), FR-U05
(toegankelijk bedienen), FR-S04 (open werk niet stil verliezen), FR-O01
(bevestigd werk na koude herstart terugvinden), FR-O04 (I/O-fouten zonder
valse ACK of replay), FR-U01 (focus en terugkeer) en rubric U03/U04/U06/R01.
Deze ontwerpspecificatie sluit geen van die criteria af.

## Hoofdtaak en toestandsovergangen

De flow gebruikt de echte `Controller`/`DocumentSession`-operaties uit
`ui/native`. UI-object-ID's zijn geen rechten; alleen reeds gevalideerde native
adapters mogen events of capabilities aanleveren.

| Gebruikershandeling | Werkelijke service-/controllerovergang | Zichtbare toestand |
|---|---|---|
| Materiaal vernieuwen | `refresh()` vraagt de echte objectlijst op. Alleen een geldige `DocumentSession::list`-respons zet `list_loaded=true`; bij fout blijft de lijst niet-geladen en wordt `Failed` gezet. | Onderscheid een geslaagde lege lijst van een mislukte/nog niet geladen lijst. Toon geen laadspinner voor een synchrone operatie. |
| Nieuw document | Zo nodig eerst echt vernieuwen; daarna eerste vrije objectslot via `DocumentSession::create`. De draft is nog niet opgeslagen. | Editor opent met focus in het document; status “Wijzigingen nog niet opgeslagen.” |
| Bewerken | Invoer wijzigt alleen de sessiedraft. Behoud UTF-8-bytes en selectie; verander geen opgeslagen revisie. | Zichtbare caret/selectie en onopgeslagen status; geen “opgeslagen” uiterlijk. |
| Opslaan | Bij gewijzigde draft: `begin_save()` start een echte stream. Iedere `advance_save()` voert maximaal één APPEND/COMMIT-uitwisseling uit. Alleen de echte `SaveProgress::Saved { revision }` zet `Status::Saved`. | Tijdens schrijven: label “Opslaan annuleren” en voortgang uit bevestigde bytes (`completed/total`), niet uit tijd. Na echte commit: “Op dit apparaat opgeslagen” plus revisienummer in de status/geschiedenisweergave zodra beschikbaar. |
| Annuleren tijdens opslaan | De knop vraagt de echte `DocumentSession::cancel_save()`/ABORT aan. | Alleen na bevestigde annulering: “Opslaan geannuleerd. Je tekst blijft open.” Draft blijft dirty; er wordt geen nieuwe revisie gemeld. Een mislukte ABORT blijft een fout. |
| Terug met dirty draft | `request_back()` opent `ConfirmClose`, standaardfocus `KeepEditing`; er wordt niet automatisch opgeslagen of weggegooid. | “Nog niet opgeslagen” met expliciete acties: “Bewaren”, “Verder bewerken” (standaardfocus), “Wijzigingen verwerpen”. Escape annuleert de overlay en behoudt de draft. |
| Bewaren en terug | Dezelfde saveketen als hierboven. Alleen na bevestigde commit gaat de controller terug naar Materiaal en vernieuwt de lijst. | Terugkeer pas na echte `Saved`; fout laat het document open. |
| Verwerpen | Alleen na expliciete keuze roept de controller `discard_changes()` aan en keert terug naar de echte lijst. | Benoem dat alleen de onopgeslagen wijzigingen worden verworpen; geen verborgen standaardkeuze. |
| Terug/open opnieuw | Na bevestigd opslaan vernieuwt de lijst, opent hetzelfde object-ID en leest de dienst de actuele revisie opnieuw. | Zelfde objectidentiteit en opgeslagen inhoud; de vorige draft is geen bewijs, de echte `open()`-respons wel. |

**Ongewijzigde inhoud:** `save()` kan na zijn beschikbaarheids-/rechtenchecks
`SaveOutcome::Unchanged { revision }` teruggeven. De controller gebruikt ook
daarvoor `Status::Saved`. Dit is de reeds geladen, bevestigde revisie, geen
nieuwe COMMIT, revisieverhoging of herbevestiging na `Uncertain`. Bij zo'n actie
luidt de bedoelde copy “Geen nieuwe wijzigingen. Revisie {n} op dit apparaat.”
De huidige `View` geeft dit onderscheid nog niet afzonderlijk door; rol 13
moet het operationele resultaat bewaren vóór deze verschillende copy mogelijk
is. Een extra klik mag nooit worden uitgelegd als nieuwe duurzaamheidsproef.

Bij `completed == total` zijn alleen de inhoudsbytes bevestigd: de volgende
COMMIT kan nog mislukken. Toon dan “Inhoud verzonden. Opslag bevestigen…”;
geen vinkje of succesanimatie. Bij een leeg document is er geen zinvol
bytepercentage; toon “Opslag bevestigen…” zonder determinate bytebalk. Een
cancelverzoek is niet zelf een terminale uitkomst. Als COMMIT al bevestigd is,
blijft het resultaat opgeslagen; meld niet alsnog “geannuleerd”. Een late of
verkeerd gebonden respons mag de actuele status niet overschrijven.

### Fouten, rechten en herstelgrenzen

- **Lezen mislukt:** geen lege-lijstpresentatie alsof er nul documenten zijn.
  Toon oorzaak als de service die werkelijk geeft, laat de gebruiker op de
  lijst blijven en bied alleen een nieuwe leesactie aan wanneer die operationeel
  beschikbaar is.
- **Read-only of geweigerde grant:** `can_save=false`; maak de onmogelijke
  actie zichtbaar uitgeschakeld met tekstuele reden. Geen herhaalde grantprompt
  en geen verruiming van object-/netwerkrechten.
- **Conflict:** behoud de draft en meld dat een nieuwere revisie bestaat.
  Geen stil overschrijven. Vergelijk/samenvoeg/heropen pas aanbieden wanneer de
  dienst daarvoor echte operaties levert.
- **Vol/te grote inhoud/I/O-fout:** behoud de draft en toon het concrete gevolg
  en de eerstvolgende werkelijk ondersteunde handeling. Niet doen alsof er
  gedeeltelijk is opgeslagen.
- **Onzekere COMMIT-uitkomst (`Unknown`/`Uncertain`):** toon expliciet
  “Opslaguitkomst onbekend. Je tekst blijft op dit scherm.” Blokkeer save en
  automatische replay; de bewaarde status is niet “opgeslagen”. De huidige
  sessie sluit de serviceverbinding en behoudt de draft in het proces, maar
  bewijst geen herstel na app-/procesbeëindiging. Er is nog geen native
  status-opvraag/veilige-hervatoperatie vastgesteld. **Daarom mag de UI hier
  geen “Opnieuw proberen” tonen totdat een versiegebonden dienstoperatie de
  commitstatus veilig kan vaststellen.**
- **Invoer-/compositorverlies:** als de huidige sessie de draft nog bezit,
  behoud die en laat een verse, geauthenticeerde inputbinding de invoer
  herstellen. Dit is geen bewijs van opslag, apparaatreset of screenreader-
  herstel.

Geen retry wordt door de UI zelf als herstel geïmplementeerd. De asset
`retry` is alleen bedoeld voor een fouttoestand waarvan de service expliciet
een idempotente/veilige retry toestaat; `Unknown`/`Uncertain` is uitgesloten.

### Nederlandse copy en beschikbaarheid van herstelacties

Onderstaande teksten zijn doelcopy, niet allemaal huidige rendererlabels.
De foutcode blijft een optioneel diagnostisch detail; nooit de primaire uitleg.

| Dienstfeit | Primaire melding / gevolg | Werkelijk toegestane vervolgstap en grens |
|---|---|---|
| Geldige lege `list` | “Nog geen materiaal. Maak je eerste document.” | “Nieuw document”, alleen met bestaande schrijfrechten. Geen lege-lijstcopy na list-error. |
| `ReadOnly` | “Alleen lezen. Je kunt dit document hier niet wijzigen.” | Lees/navigatie blijven mogelijk. Geen knop die een grant simuleert. |
| Expliciete deny/revoke | “Geen toestemming om dit document op te slaan. Je tekst blijft open.” | Behoud de draft zolang het proces leeft. Een nieuwe gescopeerde toestemming vraagt een echte broker; die herstelroute is nog niet geleverd. Een generieke transportfout wordt niet tot deny omgelabeld. |
| `Service(Full)` / `Service(Quota)` | “Opslagruimte vol.” / “Je opslagruimte is opgebruikt.”, gevolgd door “Niet opgeslagen. Je tekst blijft open.” | Alleen bewezen ruimtebeheer/een toegestaan ander doel aanbieden zodra native gekoppeld; huidige save is geen capaciteitsherstel. |
| `ContentTooLarge` | “Deze tekst past niet in de beschikbare documentruimte.” | Invoer zichtbaar behouden; niets stil afkappen. Tekst verkorten kan alleen als de levende sessie nog bewerkbaar is. De proefcap wordt geen permanente productlimiet. |
| `Conflict` / verouderde revisie | “Er is een nieuwere versie. Je wijzigingen zijn niet opgeslagen.” | Geen blind overschrijven of reopen dat de draft wist. Vergelijking/herstel is afhankelijk van een gekoppeld revisieportaal. |
| Bekende I/O-fout | “Opslaan mislukt. Je wijzigingen zijn niet bevestigd.” | Draft behouden. Veilige retry alleen na expliciete dienstclassificatie en herstel; huidige generieke foutafhandeling levert geen onbeperkt retryrecht. |
| `Closed` | “Opslagverbinding verbroken. Je tekst blijft open.” | Nieuwe geauthenticeerde sessie vereist herstel en verse grants. De UI kan oude endpoints niet heropenen. |
| `Uncertain` | “Opslaguitkomst onbekend. Je tekst blijft op dit scherm.” | Geen automatische replay, geen “Opnieuw proberen”, geen claim dat appsluiting de draft behoudt. |
| Bevestigde ABORT | “Opslaan geannuleerd. Je tekst blijft open.” | Terug naar documentfocus; de draft blijft onopgeslagen en weer bewerkbaar als de sessie dat toestaat. |
| Bevestigde COMMIT | “Op dit apparaat opgeslagen. Revisie {n}.” | Terug naar lijst en echt heropenen. Geen NAS-/back-up-/encryptieclaim afleiden uit deze lokale ACK. |

## Schermopbouw en visuele regels

Gebruik de waarden uit `docs/implementation/PRODUCT-SPEC.md` §5 en de bestaande
CSS-tokenbron; introduceer geen afwijkende runtime- of design-tokenlijst.

- **Werkvlak:** materiaalinhoud blijft dominant. Een rustige, lichte
  achtergrond, één witte inhoudssurface, precieze uitlijning en een beperkte
  inspector zijn toegestaan; geen permanente Office-ribbon, dock, decoratieve
  statistiek of glaslaag.
- **Editor-header:** links “Terug naar materiaal”; titel als primaire context;
  rechts één primaire actie (“Opslaan”) die tijdens de echte save verandert in
  “Opslaan annuleren”. Houd de knop minimaal 44×44 CSS-pixels en zichtbaar bij
  200%.
- **Inhoud:** documenttitel en teksteditor blijven één objectoppervlak.
  Cursor, selectie en toetsenbordfocus zijn afzonderlijk zichtbaar; de editor
  heeft een duidelijke controlrand en een focusring uit de bestaande token.
- **Status:** blijvend tekstlabel nabij de editorvoet; voortgangsbalk alleen
  tijdens een actieve save en proportioneel aan echte byte-ACK's. Behoud ruimte
  voor twee regels status op smalle breedte; kleur is nooit de enige betekenis.
- **Bevestiging:** ondoorzichtige lokale dialoog, geen misleidende systeem-
  privilegeclaim. Toon gevolg vóór de destructieve actie. “Verder bewerken”
  krijgt initiële focus; “Wijzigingen verwerpen” is duidelijk destructief en
  vraagt een bewuste activatie.
- **Typografie:** productdoel body 16/24, metadata 14/20, sectie 20/28,
  paginatitel 32/40, gewicht 400 of hoger. De huidige native raster gebruikt
  begrensde bitmap-monospaceglyphs; dat is geen definitieve producttypografie.
  Proportionele shaping, volledige Unicode-/graphemeondersteuning en IME zijn
  expliciete render-/inputafhankelijkheden, geen reden om de ambitie te
  verlagen.
- **Motion:** huidige native leaf animeert niet. Een toekomstige compositor mag
  alleen korte, functionele overgang gebruiken volgens de bestaande
  120/180/220-ms-waarden; bij reduced motion geen verplaatsing en maximaal
  50-ms-fade. Status-/savefeedback mag nooit van animatie afhangen.
- **Iconen:** gebruik de assetset van rol 16 alleen naast tekst of semantische
  naam. Er is nog geen bevestigde native SVG-rasterroute; geen iconen in de
  native build claimen voordat die route bestaat.

### Exacte lay-out voor rol 13

Maten hieronder zijn logische pixels (fysieke pixels gedeeld door de
schaalfactor). De SVG is een gereduceerd overzicht van toestanden, **niet** een
op schaal getekend 800/1280/1920-runtimelayout. Bepaal breekpunten uit de
logische vensterbreedte; een 1920-scherm op 200% heeft 960 logische pixels.

| Logische viewport | Lay-outregel | Lange tekst en ruimteverdeling |
|---|---|---|
| 800×600 / 1280×720 / 1920×1080 op 100% | Buitenmarge 24; werkvlak maximaal 1120 breed, gecentreerd. De bestaande shellrail volgt PRODUCT-SPEC: 224 vanaf 1200, 72 vanaf 768. Editor binnen de beschikbare contentbreedte. Headercontrols minimaal 44 hoog; 24 tussen header en inhoud; editorpadding 16. | Geen nieuwe rail voor deze slice introduceren als de native shell nog ontbreekt. Toon dan hetzelfde werkvlak, maar registreer die ontbrekende shell apart. Titel krijgt minimaal twee regels; documentpaneel vult overige hoogte. |
| 1280×720 op 200% (640×360) | Eén kolom, rail/inspector gesloten maar bereikbaar via benoemde actie zodra shell gekoppeld is; geen verkleinde labels. Header: terug links en save rechts met minimaal 8 tussenruimte. | Buitenmarge 16, header 44, titel maximaal twee regels van 24; editor minimaal twee tekstregels; status standaard 48, bij lange fout/progress 72 of meer uit gemeten teksthoogte. Geen vaste statushoogte die tekst afsnijdt. |
| 800×600 op 200% (400×300); aanvullende 320-breedte | Compacte buitenmarge 12; header 44; titelregel wrappend; geen zijpaneel. Dialog volledig scherm met verticale, volle-breedte-acties van minimaal 44 hoog en 8 tussenruimte. | Bij hoge status/titel past het document in een eigen scrollgebied; controles/status blijven bereikbaar. Wanneer niet alles tegelijk past, gebruik expliciete verticale schermscroll, nooit verbergen, overlap of horizontale paginaschroll. |
| 1920×1080 op 200% (960×540) | Middelbrede shell: 72-rail indien aanwezig; inspector opvraagbaar, niet permanent. Overige maten schalen niet nogmaals: zij zijn al logisch. | Zelfde status-/titelmeetregels. Tekstschaling niet vervangen door alleen grotere rectangles of dubbele bitmap zonder geometrieaanpassing. |

Focus krijgt aan alle zijden 6 px vrije ruimte (3 stroke + 3 offset).
Een zichtbare selectie gebruikt de bestaande `--accent-soft`-achtergrond en
`--text`-tekst; bij focusverlies blijft selectie herkenbaar, maar de actieve
focusring verhuist. Een disabled control verliest geen label of reden. Hover
verandert alleen bestaande-tokenkleuren of randen en veroorzaakt geen
layoutverschuiving. Een primaire knop behoudt de accentvulling en haar
actiehiërarchie; de secundaire surface mag naar accent-soft veranderen.
Normale tekstcontrastgrens 4,5:1; grote tekst en betekenis-
dragende UI-randen/staten minimaal 3:1. Beoordeel gerenderde kleurparen, niet
alleen de vaste waarden in dit pakket.

## Toetsenbord, focus, schaal en semantiek

- Tab/Shift+Tab door alle acties; in de editor: Terug → Document → Opslaan
  (tijdens save: Annuleren). Enter activeert een gefocuste control; in de
  documenttekst voegt Enter een regeleinde in. Ctrl+S vraagt de echte save aan.
  Escape bij dirty werk opent de expliciete keuze, niet discard.
- De focusring is 3 px met 3 px offset, volledig zichtbaar en niet uitsluitend
  op kleur gebaseerd. Pointer hover vervangt focus niet. Focus keert na sluiten
  van de niet-destructieve dialoog terug naar de actie die haar opende.
- Op 200% herschalen zowel tekst als geometrie; geen verborgen primaire actie,
  overlap of horizontale pagina-scroll. Lange UTF-8-titels en statussen wrappen
  zonder over documenttekst of controls heen te tekenen.
- De huidige `Frame::semantics` biedt app-lokale rollen, namen, focus,
  read-only/enabled, tekstwaarde, selectie en echte savevoortgang. Dit is nog
  geen accessibility-tree-/event-IPC of werkende schermlezer. Screenreader-
  labels, statusaankondiging en modal focus trap moeten in de compositor/
  accessibilitybroker echt zijn gekoppeld vóór FR-U05.
- Toetsenbordbediening wordt in dezelfde flow als muisbediening getest;
  reduced motion is een systeemvoorkeur, niet een app-specifieke security
  instelling.

## Controleerbare acceptatie voor de volgende native build

1. **Echte bediening:** op exact brongebonden build nieuw/open → wijzig tekst
   → save → echte APPEND-voortgang → bevestigde commit → terug → lijst
   vernieuwen → hetzelfde object heropenen. Vergelijk object-ID, revisie en
   inhoud met onafhankelijke opslaginspectie.
2. **Negatieven:** ABORT vóór COMMIT; dirty-close en Escape; deny/read-only;
   volle opslag; conflict; I/O-fout; onbekende COMMIT. Geen geval toont
   bevestigde opslag zonder bevestiging, geen onbekende uitkomst wordt replayed,
   en de editor verliest geen draft tijdens de levende sessie.
3. **Vaste rasteropnamen:** werkelijk gerenderde editor/dirty/saving/saved/
   conflict/uncertain-states op 800×600, 1280×720 en 1920×1080; 100%/200% en
   320 CSS-pixels logische breedte. Leg bron-, image- en framehash vast.
4. **Lay-out/asserties:** geen clipping/overlap; zichtbare focus; minimaal
   44×44 targets; statuslabel volledig; lange Nederlandse tekst en
   UTF-8-tekst blijven leesbaar; empty-list is onderscheidbaar van list-error.
5. **Bediening/toegankelijkheid:** dezelfde assertions via PS/2/QMP keyboard
   only en pointer; semantische rollen/namen/status/progress controleren; echte
   screenreader-/accessibilitybrokerproef apart registreren.
6. **Bewijsgrens:** host-raster/hostharnas is ontwerpreview; targetbuild is
   geen native uitvoering; native VM-QMP-render en bediening zijn pas bewijs
   voor hun exacte bron/image; menselijke taakproef blijft afzonderlijk.

### Gerichte statechecks voor 13/20

Dit zijn vooraf vastgelegde verwachte uitkomsten, **niet** uitgevoerde tests.
Bij iedere case worden input, startrevisie, echte serviceuitkomst, zichtbare
status en focus vastgelegd. Opslagcases krijgen daarnaast een onafhankelijke
byte-/revisiecontrole; snapshots of UI-copy zijn niet voldoende.

| Case | Prikkel | Vereiste uitkomst |
|---|---|---|
| UX-SR01 | Echte lege lijst tegenover list-error/geen geldige respons | Alleen een succesvolle lege lijst toont leegte; fout heeft benoemde oorzaak. Geen fictieve objecten. |
| UX-SR02 | Nieuw → UTF-8-tekst → Ctrl+S | Focus eerst Document, tijdens save CancelSave, na bevestigde COMMIT Document; draft/inhoud/revisie exact behouden en echte voortgang zichtbaar. |
| UX-SR03 | Laatste APPEND bevestigd, COMMIT nog niet bevestigd; ook leeg document | Geen Saved/vinkje; bytebalk is geen commitsucces. Leeg document heeft geen verzonnen percentage. |
| UX-SR04 | ABORT tijdens stream; mislukte ABORT; cancel na voltooide COMMIT | Alleen bevestigde ABORT geeft Cancelled en dirty draft. Fout blijft fout; bevestigde COMMIT wordt niet als annulering getoond. |
| UX-SR05 | Dirty-close; focus achtereenvolgens op KeepEditing, SaveAndBack en Discard, plus Tab/Shift+Tab/Escape | Elk focusdoel zichtbaar/semantisch correct, achtergrond niet bedienbaar. Escape behoudt draft/caret/selectie. Verlies alleen na expliciete Discard. |
| UX-SR06 | ReadOnly, verse deny/revoke, Closed en Conflict bij openen/opslaan/confirm-close | Geen werkende Save/Bewaren-claim waar dienst weigert. Defaultfocus blijft veilig; draft blijft zichtbaar. Focus niet achterlaten op onbereikbaar geworden actie. |
| UX-SR07 | Full/Quota/ContentTooLarge en bekende I/O-fout | Geen truncatie, geen valse ACK; oorzaak, gevolg en ondersteunde vervolgstap correct. Geen generieke fout als bewijs van Full/deny presenteren. |
| UX-SR08 | Verloren COMMIT-reply / Uncertain | Geen Saved, geen automatische replay en geen retryknop. Ook na inputrebind of terugediten naar oorspronkelijke bytes blijft de onbekende uitkomst herkenbaar. |
| UX-SR09 | Unchanged op een geldige sessie; daarna dezelfde poging op Closed/Uncertain | Geen nieuwe revisie/COMMIT claimen; afgesloten/onzekere sessie geeft geen nieuw succes. |
| UX-SR10 | Late respons, oude generation/epoch of stale actie na navigatie | Geen status-/focusoverschrijving of nieuwe mutatie door verouderde input. Native adapter moet de binding afdwingen, UI-label doet dat niet. |
| UX-SR11 | Lange titel (inclusief accenten), lang foutlabel en saveprogress op alle drie fysieke maten, 100/200%, 320-logische-breedte en keyboard-only | Geen label buiten knop/status, geen overlap of verloren primary action; 44×44 targets en 3+3-focusruimte. Saveprogress en lange fout afzonderlijk én foutovergang uit progress inspecteren. |
| UX-SR12 | Save → lijst → hetzelfde object open → stop → koude nieuwe VM → hetzelfde object open | ID/revisie/UTF-8-bytes gelijk aan bevestigde opslag; verse Ready-status hoeft niet identiek te zijn aan de oude Saved-status. Geen oude draft als cold bewijs. |

## Werkelijk waargenomen basis en afwijkingen

`ui/native/verification/checks.json` rapporteert host/scalar checks met
`native_execution=false`, `screenreader=false` en `physical_hardware=false`.
De opgeslagen `host-raster-review.json` beschrijft werkelijk bekeken host-
rasters van 800×600 en 640×480: de minimale lijst toont een echt object,
de editor-/dirty-close-states zijn gerasterd op 100%/200%, en een opaque
lokale backdrop voorkomt half zichtbare modaltekst. Dat brongebonden
hostbewijs is niet opnieuw opgebouwd in deze checkout.

Rol 13 leverde de render-/state-specificatie op reviewbasis `e382a989` en
meldde de afzonderlijke host-rendererfix `0818f5e` en generation-bound
semantiekconsumer `89e2004`. Geen van die losse commits is daarmee bewezen
op één gastimage. De specificatiereview vindt de bestaande save-/ABORT-/
Uncertain-scheiding passend; de `Unchanged`-nuance, lange status/progress op
200% en readonly/closed/conflict/deny bij alle drie confirm-close-focusdoelen
blijven concrete te controleren integratiepunten. De nieuwe host-PPM's
waren niet beschikbaar in deze checkout en zijn hier niet bekeken.

De bestaande bedieningstest `real_ps2_edit_stream_commit_list_and_reopen`
doorloopt via het hostharnas echte PS/2-bytes, draft, echte saveprogress,
commit, lijst en reopen; andere hostgevallen testen ABORT, onbekende commit,
conflict, intrekking, I/O, readonly en focus. Deze tests bewijzen niet de
QEMU-presentatie of invoer van de samengestelde 13/20-build.

Concrete open afwijkingen/afhankelijkheden:

1. De renderer is een scalar 32×8 private-RAM-tegelrenderer; compositor,
   display-presentatie en echte QMP-render/bediening moeten aan de exacte
   13/20-releasecandidate worden gekoppeld.
2. Semantiek is app-lokaal; screenreader-IPC en daadwerkelijke aankondiging/
   focusintegratie ontbreken.
3. De huidige monospaced bitmapfont en codepointbewerkingen zijn een
   begrensde eerste implementatie, niet de uiteindelijke typografische of
   Unicode-kwaliteit.
4. Er is nog geen versiegebonden veilige statusopvraag voor `Unknown`; de UI
   mag deze toestand niet “herstellen” met retry of opgeslagen-succes.
5. De design-assets zijn geen runtime-assets zolang native vector/raster-
   consumptie niet beschikbaar is.
6. De huidige testbron heeft geen in deze checkout opnieuw geproduceerde
   rasters of native gecombineerde build. De release-integrator levert de
   exacte kandidaat voor vervolgbeoordeling; geen ontbrekend bewijs wordt
   afgeleid uit oudere screenshots.

**Menselijke gebruiksproef:** nog niet uitgevoerd. Bereid een taak voor een
gebruiker zonder beheerderskennis voor: “Maak een korte notitie, bewaar haar,
ga terug en open dezelfde notitie opnieuw; annuleer daarna een wijziging en
leg uit wat wel/niet is opgeslagen.” Noteer taakvoltooiing, hulp, begrip van
opslagstatus, foutverwachting en herstelsucces; verzin geen deelnemer of
resultaat.

### Uitvoerbaar draaiboek voor de menselijke taakproef

Dit is een voorbereid protocol, geen werving, deelnemersregistratie of
uitgevoerde proef. Rol 20/21 gebruikt uitsluitend een vastgelegde synthetische
wegwerpimage. Deelnemers werken niet met persoonlijke documenten, accounts,
NAS of sleutels. Deelnemerstoestemming en een geschikte lokale opslagplaats
voor notities worden vóór de proef geregeld; geen automatische opname,
telemetrie of publicatie. Vooraf wordt expliciet uitgelegd dat dit een
ontwikkelversie is en dat de beoordeling het systeem betreft, niet de deelnemer.

**Startvoorwaarden:** exact broncommit, kernel/app-/diskhashes, profiel,
runner en inputmethode invullen. De checker bevestigt dat de taak en
foutinjectie werkelijk werken, zonder het resultaat vooraf aan de deelnemer te
tonen. Geen mens laten beoordelen of niet-geïmplementeerde bediening “werkt”.
Wanneer de native taak niet beschikbaar is, mag een gelabelde ontwerpstudie
worden uitgevoerd, maar die krijgt geen native gebruiksacceptatie.

| Stap / neutrale opdracht | Te observeren zonder antwoord voor te zeggen | Onafhankelijke controle |
|---|---|---|
| “Maak een korte notitie over een afspraak en zorg dat je haar later terugvindt.” | Vindt de deelnemer maken, schrijven en bewaren? Welke zichtbare status geeft volgens hem/haar zekerheid? | Echte commit, object-ID, revisie en bytes vastleggen; geen taaksucces op basis van alleen vertrouwen of een klik. |
| “Ga terug naar je materiaal en ga verder met dezelfde notitie.” | Wordt hetzelfde object gevonden, zonder onbedoelde kopie? Blijft de relatie tussen object en gereedschap duidelijk? | Echte list/open, zelfde ID en bevestigde bytes; geen vorige draft als bewijs. |
| “Voeg iets toe en ga terug. Je wilt die toevoeging toch niet bewaren.” | Begrip van Bewaren / Verder bewerken / Wijzigingen verwerpen, standaardfocus en Escape; onbedoeld verlies of hulp tellen. | Alleen expliciete discard verwijdert de toevoeging; eerdere bevestigde revisie blijft intact. |
| “Bewaar deze langere notitie, maar stop de opslag terwijl die bezig is.” | Vindt en begrijpt de deelnemer annuleren? Verwacht die tekstverlies, ongedaan maken of alleen stoppen? | Echte ABORT en behoud dirty draft; als commit eerder voltooid is, die uitkomst eerlijk registreren en niet als annuleerfout classificeren. |
| Gecontroleerde onzekere uitkomst: “Wat is er volgens jou gebeurd, en wat zou je nu doen?” | Begrip van onbekend tegenover mislukt/opgeslagen; gevaarlijke verwachting van retry, afsluiten of back-up noteren. Geen herstel beloven dat niet bestaat. | Fault/COMMIT-binding en no-replay door checker bevestigd; open herstelafhankelijkheid blijft zichtbaar. |
| “Ga na het opnieuw starten verder met dezelfde notitie.” | Kan de deelnemer het bevestigde werk terugvinden? Onderscheidt die vorige onopgeslagen wijziging van de laatste opgeslagen versie? | Koude nieuwe VM, zelfstandig diskorakel en geopende objectbytes; alleen uitvoeren wanneer die native route beschikbaar is. |

Herhaal de relevante taken keyboard-only en op 200% met geschikte deelnemers
voor die bediening; een ziende muisgebruiker die een schermlezer beluistert
vervangt geen toets met daadwerkelijke hulptechnologiegebruikers. Forceer geen
tweede ronde als vermoeidheid of de ontwikkelbuild de resultaten vertekent.
De probeleider geeft tijdens de primaire poging geen navigatiehint; gevraagde
hulp wordt pas gegeven nadat het hulpmoment en de oorspronkelijke uitkomst
zijn vastgelegd.

Per taak registreert de tester: begonnen/voltooid/geblokkeerd/afgebroken,
start/eindtijd, aantal en aard van hints, onbedoelde acties/kopieën,
verwachte versus werkelijke opgeslagen versie, focus-/leesbaarheidsprobleem,
gebruikersuitleg in eigen woorden en hersteluitkomst. Tijd en subjectieve
waardering apart rapporteren; geen gemiddelden gebruiken om verlies,
onbegrepen Unknown, ontbrekende accessibility of een niet-uitvoerbare taak
te verbergen. Stop de taak bij onverwacht gegevensverlies, onveilige
retry/rechtenverruiming of niet-herstelbare focusval; bewaar het defect en
herhaal pas na herstel op een nieuwe gebonden kandidaat.

Steekproefomvang en taaktijd-/hulpbudget worden vóór de echte proef door het
onderzoeksspoor vastgelegd volgens PRX/DV-01, niet achteraf uit gunstige
uitkomsten gekozen. Dit pakket claimt geen representativiteit, tevredenheid,
adoptie of conformiteit. Native technische assertions, AI-ontwerpreview en
menselijke observaties houden elk een eigen resultaatkolom.

## Assetoverdracht en lokale controle

Rol 16 leverde alleen `ui/assets/material-flow-icons.svg`, definitieve
bron `d3547639be733aefcc1228edad37114980e5073c`. Vier symbols:
`document`, `history`, `saved`, `retry`; 24×24 viewBox, monochrome currentColor,
1,75 stroke en ronde uiteinden/hoeken. Het bestand is een sprite, geen direct
zichtbaar iconenblad. Rol 15 heeft de werkelijk met systeem-librsvg/Cairo
gerenderde contact sheet op 24/48 px en lichte/donkere achtergrond bekeken:
geen zichtbare clipping, de document-/klok-/vink-/herhaalcontouren zijn
onderscheidbaar. Dit is AI-hostvisualisatiecontrole, geen menselijke
herkenningsproef of native consumptie. De sprite wordt niet door dit pakket
geïntegreerd; intake/rol 8 behoudt afzonderlijke bron en eigenaarschap.

Lokale uitvoering op 30 oktober 2026:

| Commando / controle | Resultaat en grens |
|---|---|
| `python3 -B ui/native/verify.py` | Exit 1: `FileNotFoundError: cargo`. Geen hosttests, Clippy, build of nieuwe native rasterbeelden uitgevoerd. Bestaande bewijsbestanden niet herschreven. |
| SVG-parsercontrole met `xml.etree.ElementTree` | Exit 0: namespace/viewBox correct. Eerste shellvariant had een Python-quote-syntaxfout (exit 1); gecorrigeerde parsercontrole slaagde. |
| `python3 -B /home/simon/.copilot/session-state/038b6cdc1bb74465872d0b93e6a6e0cd/files/render-design.py ui/design/save-recovery.svg /home/simon/.copilot/session-state/038b6cdc1bb74465872d0b93e6a6e0cd/files/save-recovery.png` | Systeem-librsvg/Cairo, zonder installatie; daadwerkelijk ontwerp gerenderd en bekeken. Geen native runtime of bedieningsproef. |
| `git diff --check` | Exit 0; alleen toegewezen ontwerppakketbestanden in de worktree toegevoegd. |
| `python3 -B tools/progress.py --check` | Exit 0; bestaande rubric 34/100 consistent. Ontwerp/assetreview sluit U03/U04/U06/R01 niet af: voorstel **34% → 34%**. |

De eerste visuele ontwerpcontrole vond de te smalle knop “Opslaan annuleren”.
Die knop is verbreed; de definitieve schets is opnieuw gerenderd en bekeken,
zonder de gevonden labeloverloop. Dit valideert de illustratie, niet de native
responsive lay-out.
Geen runtime-token gewijzigd of gereserveerd. Centrale PROJECT-STATE,
WORK-QUEUE en scoreassessment blijven conform managerafspraak bij rol 1.

Bronbinding van deze visuele controle:

- Ontwerp-SVG SHA-256:
  `57a69211e476a51ccd924d492a3047a9e5931d2a5b93fe2bb74e94b0430b7858`.
- Ontwerp-PNG (1440×1110) SHA-256:
  `d513ce5cb053f662400b79423ead043802d3c8784ce206bed9a8e5ac09fde24f`.
- Rol-16-asset SHA-256:
  `391f75bcfdf5f7cd37532b66531a52646a650e3bc3d7c7b2a811f2d60b2dea26`;
  contact sheet:
  `f92b214bfcbeb2dded9c135232fe7c7518acc8886a0f969f092dd9ce3a839d1b`.

De PNG's blijven sessieartifacts, buiten Git. De SVG bewaart bronvorm en
expliciet simulatie-/ontwerplabel; geen gegenereerde binaries in deze commit.

**Historische hervatstap vóór ontvangst van de probe:** rol 4/12 levert één fixed candidate met broncommit,
image-/apphashes, UI-/tokenbronbinding, profiel, bestaande runner en bijbehorende
13/20-claim. Voer daarop UX-SR02/03/05/06/08/11/12 uit en beoordeel de echte
frames én invoertransities. Tot dan is runtime-/gebruikersacceptatie open;
de lokale nieuwe hostrasterrun blijft geblokkeerd door ontbrekende Cargo.

## Native deelbeoordeling — werkelijk bediende probe 0818f5e

**30 oktober 2026, rol 15.** De hiervoor beschreven ontbrekende probe is nu
ontvangen en read-only beoordeeld. Dit vervangt uitsluitend die oude
beschikbaarheidsblokkade. Het is geen nieuwe VM-run door rol 15 en geen
menselijke audit. Ontwerpbron en onderzochte runtimebron blijven verschillend.

| Binding | Exacte waarde / grens |
|---|---|
| Native bron | `0818f5ec651b72c32b8795ff0d3d9a6ebdc9783d`, v3 `26b41e5586434ce51202a4e234ccaa99bad25c5a76ec3efdb3af9d7020c14f5f` |
| Probe | `native-system-materiaal-probe`, normal; `pc-q35-10.0/qemu64/TCG/1CPU/256MiB/VGA/PS2`, 800×600, schaal 100%. Dit is niet VM-FR of de gewone dev1-preview. |
| Uitgevoerde run door 13/12 | 132,4606 s; ruwe QEMU-exit 33, runner exit 0. Duur van de hele proef is geen meting van interactieve responstijd. |
| Kernel / loader | `48acb20ca8e12e0418f21c3790361c2a3ac8c935cc76de0ad6ddb977bfd10896` / `9686ab3b85eaeedc334053cc9f583f6459c99e2779673416eacac319c68aa387` |
| Materiaal-app | `d84f5333ad11d91b44ea822382562d78a508ec12794518cc70a0a47b583c06db` |
| Tokenbron | `ui/prototype/tokens.css`: `0c588509e0cb0f94bbda2c6969da8d79d463359bfc9684687a684210a54aadcf`, onveranderd |
| Read-only bundle | `native-materiaal-probe-0818f5e-ad23a2ee91.tar.gz`, 2.682.413 bytes, SHA-256 `3c476387f7aa7657207e2a0ee96b7d1b159695de1716305cdbefa795b26447b2` |
| Vindplaats | `/home/simon/.copilot/session-state/1b87793863ad4dcc8f16a0bd10884654/files/ui-image-0818/`; naastliggende `handoff.json` bevat runcommando, bron-, tool-, userspace-, firmware-, input- en framebinding |

### Wat werkelijk is bekeken en gecontroleerd

Alle zes weergegeven PNG's (`ready`, `editor`, `dirty`, `saved`, `list`,
`reopened`) zijn geopend en visueel beoordeeld. Hun hashes zijn tegen het
handoff gecontroleerd; de gedecodeerde RGB-pixels zijn voor ieder beeld exact
vergeleken met de oorspronkelijke PPM in de bundle. Alle 40 manifestbestanden,
15 bevroren artifacts en vijf UI-/tokenbronbestanden zijn hashgecontroleerd;
de UI-bronnen zijn daarnaast vergeleken met de Git-objecten op `0818f5e`.
Deze controle certificeert binding en pixelgelijkheid, niet alle gedragsclaims.

De 31 gearchiveerde QMP-verzoeken en hun geslaagde antwoord-ID's zijn
geïnspecteerd: pointer naar Nieuw en knopdruk/-release, 14 echte ASCII-
toetsen (`nieuw os werkt`), Ctrl+S, Escape, Tab, Tab, Enter en zes screendumps.
Geen gastgeheugenwrites in die trace. F12 en de testcontroller verzorgen alleen
de proefstop: geen gewone productafsluiting, shutdown- of coldreopenbewijs.

De overgedragen snapshots tonen voor `dirty` 14 draftbytes en nul save-ACK's,
voor `saved` één save-ACK, revisie 1 en een schone draft, en na list/open
hetzelfde object 0 met dezelfde 14 bytes. De diskoracle-uitkomst in het
handoff is object 0 / revisie 1 / 14 bytes / canaries intact. De onafhankelijke
runtime-/diskreplay is door 13/12 aangeleverd als geslaagd; rol 15 heeft de
opslagoracle hier niet opnieuw uitgevoerd en claimt geen tweede VM-proef.

### Concrete visuele bevindingen voor rol 13

| ID / prioriteit | Waarneming op exact deze frames | Implementeerbare vervolgactie en acceptatie |
|---|---|---|
| UX-V01 — verbetering vóór visuele productacceptatie | Op `ready` is Nieuw een diepe groene primaire knop; op `list` verliest diezelfde actie haar accentvulling wanneer de pointer erop staat en lijkt zij op de secundaire Vernieuwen-knop. De focusrand blijft wel zichtbaar. | Behoud de primaire actiehiërarchie bij hover, bijvoorbeeld dezelfde accentvulling met een bestaande-tokenrand/focusbehandeling. Geen nieuw token zonder claim. Vergelijk native ready/list met pointer op en naast dezelfde control; tekstcontrast en focus moeten blijven voldoen. Dit blokkeert de normale saveketen niet. |
| UX-V02 — open productkwaliteit | Titel “Nieuw document”, documenttekst, knoppen en status gebruiken een gelijkmatige kleine bitmap-monospacepresentatie. De titel van het geopende object krijgt nauwelijks meer nadruk dan metadata; de grote inhoudssurface benadrukt het verschil met de kleine labels. Geen clipping zichtbaar bij deze korte ASCII-inhoud. | Behoud de huidige functionele bitmapbackend als gelabelde tussenstap; plan de vastgelegde body/metadata/titelhiërarchie met echte tekstmetrics en proportionele shaping. Controleer gekozen nominale maten en gerenderde glyphs op de doelprofielen. Deze waarneming bewijst niet dat een specifieke fontgrootte fout is, noch dat tekst voor alle gebruikers leesbaar is. |
| UX-V03 — open objectcontinuïteit | `saved` zegt alleen “Op dit apparaat opgeslagen”; `list` toont “Nieuw document” / “Op dit apparaat”, en `reopened` zegt “Document geopend”. De nieuwe revisie is correct in de snapshots maar niet zichtbaar in deze UI. | Maak de bevestigde revisie samen met objectidentiteit opvraagbaar/zichtbaar in de status of geschiedeniscontext; `Unchanged` blijft afzonderlijk van een nieuwe commit. Geen nieuwe versie uit alleen tekstwijziging afleiden. Verifieer de uiteindelijke labels tegen de echte documentrespons. |
| UX-V04 — positieve deelwaarneming | Dirty en Saved zijn tekstueel verschillend; de focusring blijft volledig binnen de toegewezen surface zichtbaar. Documentinhoud `nieuw os werkt` verschijnt in dirty, saved en reopened, met caret/focus op het document. Systeemchrome blijft buiten het appvlak. | Behoud deze status-/focus-/surfacegrenzen bij vervolgwerk. Niet uitbreiden naar grantprompt-authenticiteit, schermlezer, selectiecontrast of crashherstel: die zijn niet bediend in deze proef. |

De betekenisvolle inhoud heeft geen zichtbare overlap met controls/footer op
deze zes 800×600-beelden. Dit sluit lange tekst, alle fouttoestanden of de
andere viewports niet af. Geen wijziging aan runtimecode, compositor, glyphs,
tokens of gedeelde interface door rol 15.

UX-V01 is ook brongebonden: `render.rs` op `0818f5e`, regels 1064–1077,
geeft Primary alleen bij `!hovered` de accentvulling; hover valt terug op
dezelfde stijl als Secondary. De gewenste aanpassing is hierboven aangescherpt
in de hoverregel; de huidige afwijking wordt niet als foutloze uitvoering van
de eerdere, te algemene surface/accent-soft-regel gepresenteerd.

Rol 16 meldt op dezelfde fixture de nog ontbrekende 24-px-iconenslots bij
lijst-/editortitel en status. Dat is een toekomstige integratieafhankelijkheid,
geen defect van de geleverde symbolen: reserveer een gezamenlijk
icon-/tekstvak met 8 px tussenruimte, meet de volledige 24-px-hoogte en plaats
het niet op de bestaande tekstbaseline over de editorrand. Schaal beide samen
op 200%. Alleen de actuele bevestigde documentstatus mag `saved` tonen;
een oude teller `SAVE_ACKS=1` na heropenen is daarvoor geen grond. Geen
history-/retry-/200%-assetpresentatie in deze run aftekenen.

### Acceptatie per daadwerkelijk beschikbare case

| UX-SR | Beoordeling op deze ene normale probe |
|---|---|
| SR01 | Geslaagde lege lijst zichtbaar. List-error niet uitgevoerd; case niet volledig afgetekend. |
| SR02 | Nieuw via pointer, ASCII-invoer, Ctrl+S en bevestigde Saved/reopen zichtbaar en aan trace gekoppeld. Tussenliggende saveprogress/CancelSave-focus niet vastgelegd; daarom slechts deelbewijs. |
| SR03 / SR04 | Geen tussen-APPEND-frame, leeg-documentcommit of ABORT-/cancelrace. Niet uitgevoerd. |
| SR05 / SR06 / SR07 / SR08 / SR09 / SR10 | Geen dirty-close-modal, deny/read-only/closed/conflict/full/I/O/Unknown/Unchanged of stale-inputcase in deze fixture. Niet uitgevoerd; positieve normal-run vervangt negatieve tests niet. |
| SR11 | Alleen 800×600 op 100%, korte ASCII-inhoud. Geen 200%, andere maten, lange fout/progress of keyboard-only vanaf eerste start. Niet afgetekend. |
| SR12 | Zelfde object binnen dezelfde VM opnieuw geopend. Geen koude nieuwe VM; cold-deel niet uitgevoerd. |

De native bron heeft aantoonbaar echte frames én een bedieningsspoor voor dit
smalle normale pad. Alle samengestelde FR-U03/U05/S04/O01/O04 en
rubric U03/U04/U06/R01 blijven open; voorstel **34% → 34%**. Assets van rol 16
zijn in deze runtime niet geconsumeerd. Schermlezer-/menselijke/fysieke
acceptatie blijft niet uitgevoerd.

### Commando, resultaat en eerstvolgende hervatstap

Vanuit de eigen repositoryroot:

```sh
python3 -B /home/simon/.copilot/session-state/038b6cdc1bb74465872d0b93e6a6e0cd/files/verify-native-fixture.py /home/simon/.copilot/session-state/1b87793863ad4dcc8f16a0bd10884654/files/ui-image-0818
```

Exit 0: bundlehash/-grootte, manifestleden, zes lossless pixelroundtrips,
15 artifacthashes, vijf Git-UI-bronnen en 31 geslaagde begrensde QMP-verzoeken
gecontroleerd. Geen gast gestart, geen bundelcode uitgevoerd en geen
bewijsbestand gewijzigd. Het verificatiescript is een sessieartifact; de
bestaande repositoryhelpers `capture_graphics.decode_png` en
`run_graphics.ppm` leveren de pixelcontrole. De volledige JSON-uitvoer staat in
`/home/simon/.copilot/session-state/038b6cdc1bb74465872d0b93e6a6e0cd/files/native-fixture-verification.json`.
Ook `git diff --check` en `python3 -B tools/progress.py --check` gaven exit 0.
De hoofdagent/manager blijft eigenaar van de centrale assessmentbestanden;
rol 15 levert alleen dit 34% → 34%-voorstel met bewijs.

**Volgende afgebakende flow:** dirty-close met alle drie focusdoelen en
Escape (SR05), gevolgd door de echte fout-/Unknown-presentatie (SR06/08).
Rol 13 bezit implementatie; 4/12/20 wijzen de nieuwe, brongebonden cases en
het runnerslot toe. Beoordeel vervolgens de werkelijk gerenderde modal en
foutframes plus de bijbehorende bedieningsasserties; geen extra VM door rol 15
zonder die afstemming. UX-V01–03 zijn concrete productafwijkingen voor dat
vervolg, geen opdracht om zonder claim de gedeelde tokenbron te wijzigen.

## UX-V01 — gerichte bronreview van de hoverfix

**30 oktober 2026, Europe/Amsterdam.** Beoordeelde leaf:
`0818f5ec651b72c32b8795ff0d3d9a6ebdc9783d` →
`29b79eae19de616be415cfbd7b7cc313766546d6`
(`origin/codex/native-primary-hover`). Alleen de rendererdelta, bijbehorende
pixelregressies en `UX-V01-primary-hover.md` zijn beoordeeld; geen algemene
herreview of nieuwe test-/VM-run.

**Uitkomst: geen blokkerende ontwerpafwijking in deze delta.** De primaire
knop houdt bij hover `ACCENT` als vulling en `SURFACE` als labelkleur; alleen
de rand verandert naar de bestaande `TEXT`-kleur. Dat voldoet aan de hierboven
aangescherpte hoverregel, zonder nieuwe tokens of geometriewijziging. De
disabled-tak blijft vóór de stijlkeuze staan; Secondary en Danger zijn
ongewijzigd. De bestaande focuscode blijft na het schilderen van de knop staan.

De toegevoegde tests bemonsteren echte tilepixels op 100/200% voor Nieuw,
Opslaan en de primaire modalbewaaractie: vulling, bovenrand, focusrand en
ongewijzigde gerenderde labelpixels. De aanwijzer ligt buiten die samples.
Een afzonderlijke case controleert disabled Nieuw en secondary-hover
Vernieuwen. Dit is gericht regressiebewijs, geen volledige oppervlakvergelijking
of native modal-/toegankelijkheidsacceptatie.

Uitgevoerde reviewcontroles: `git diff --name-only 0818f5e 29b79eae`
toont uitsluitend de drie toegewezen leafbestanden; de volledige renderer-/
testdiff en bewijsnotitie zijn gelezen. Een read-only Python-stdlibcontrole
vergeleek alle twaalf SHA-256-vermeldingen uit die notitie: drie bron-/token-
bestanden tegen Git-objecten en negen logs tegen de aangeleverde logmap.
Exit 0. `red.log` toont de verwachte oude fout bij Material/100%;
`regressions.log` toont 4/4 groen. De overige gates zijn door rol 13 uitgevoerd
en beschreven, niet door rol 15 opnieuw gedraaid. Geen bundelcode uitgevoerd.

**UX-V01 is op bron-/hostregressieniveau afgehandeld; native herbeoordeling
blijft open.** De zes 0818-beelden worden niet naar 29b79eae herlabeld.
UX-V02/03, SR05/06/08, native 200%, integratie-CI en menselijke acceptatie
blijven buiten deze review. Scorevoorstel blijft **34% → 34%**; centrale
assessment bij de manager. Integratie en passende gecombineerde gates blijven
bij de toegewezen integrator; rol 15 wijzigt geen runtimebestanden.
