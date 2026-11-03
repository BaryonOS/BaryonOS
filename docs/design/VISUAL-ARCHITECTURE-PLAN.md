# Visueel ontwerpplan: Materiaal als blijvende werkomgeving

**Datum:** 3 november 2026, Europe/Amsterdam.
**Bronbasis:** `380111b68688c981eec6abb8fab29174a29f5601`,
branch `simonvanieperen-fictional-disco`.
**Status:** onderzoeksvoorstel, geen gekozen definitief designsysteem,
productimplementatie, renderbenchmark of menselijke beoordeling.

**Vervolg 3 november:** Simon heeft daarna "Voer het uit" opgedragen.
Veld wordt daarom als eerste native richting uitgevoerd, zonder het volledige
ontwerp of de alternatieven al gevalideerd te noemen. De oorspronkelijke
onderzoeksafbakening hieronder beschrijft de eerdere opdracht, niet een verbod
op deze expliciet gevraagde vervolgstap. Huidige implementatie en open grenzen:
[native venstersessie](../implementation/NATIVE-WINDOW-SESSION.md#veld-eerste-native-uitvoering).

## 1. Uitkomst en afbakening

Onderzoek twee richtingen verder: **Veld**, waarin materiaal een blijvende
plaats krijgt en gereedschappen eraan worden gekoppeld, en **Werkstuk**,
waarin meerdere gereedschappen hetzelfde materiaal gelijktijdig vanuit
verschillende kanten tonen. Veld is de eerste aanbeveling voor een algemeen
desktop-OS; Werkstuk is de serieuze tegenkandidaat, niet een extra modus die
zonder gebruiksonderzoek in Veld wordt gestopt.

De eigenheid moet uit drie samenhangende eigenschappen komen:
**het werk blijft zichtbaar wanneer een gereedschap wisselt; bediening verschijnt
bij haar onderwerp; voortgang en toegang horen bij datzelfde onderwerp.**
Een mooie taakbalk, glaslaag of kleurverloop alleen realiseert dit niet.

De actuele vraag van Simon is leidend: **geen gegenereerde afbeeldingen**,
wel een concreet designplan en onderzoek naar hoe andere OS'en hun
vormgeving in code realiseren. Daarom levert dit document uitgeschreven
composities, geometrie, toestanden en bewegingsscenario's; geen
high-fidelitybeelden en geen claim dat de uitstraling al visueel is gevalideerd.
De eerdere bijlagen zijn probleem- en acceptatiecontext. Hun verzoek om
beelden en om direct een definitief designsysteem te kiezen wordt hier niet
uitgevoerd.

Geen productcode, tokens in de runtime, kernelcontracten of P2-taakflows
wijzigen. Materiaal blijft de gekozen productlogica uit
[PRODUCT-SPEC](../implementation/PRODUCT-SPEC.md). De vijf richtingen onderzoeken
de **presentatiearchitectuur**, niet een vervangend bestandseigenaarschap.
Het volle releasecontract en de privacygrenzen blijven bestaan.

Onderbouwing en bronbeperkingen staan in
[VISUAL-RENDERING-RESEARCH](../research/VISUAL-RENDERING-RESEARCH.md).
Dit is geen volledige historische monografie of onafhankelijke usabilitystudie.

## 2. Waarom opnieuw drie stijlen maken niet helpt

De daadwerkelijke oude A/B/C-ontwerpbestanden of screenshots staan niet in deze
bronboom en zijn niet bijgevoegd. Wel staan hun beschrijvingen en Simons kritiek
in de aangeleverde tekst. Onderstaande kritiek betreft die beschreven
architectuur; details van ongeziene beelden worden niet verzonnen.

| Beschreven richting | Kernprobleem | Wat wel bruikbaar blijft | Waarom geen overtuigende 2035-identiteit |
|---|---|---|---|
| A: rustig, architecturaal | Appvensters, titelbalk en systeemstrook blijven het skelet | Uitlijning, leesbare neutralen, terughoudende ornamenten | Andere kleuren wijzigen niet wat de gebruiker als hoofdonderwerp ziet |
| B: gelaagd, adaptief | Glas en laagvorming kunnen oude containers verfraaien zonder hun relatie te veranderen | Context behouden door overlap; tijdelijke bediening onderscheiden | Een effect is nog geen nieuwe organisatie van werk |
| C: information-first | Informatiedichtheid kan klassieke beheersoftware opleveren | Compacte lijsten en voorspelbare navigatie | Minder versiering kan juist het oude raamwerk sterker zichtbaar maken |

### Audit van de desktopgrammatica

**LEGACY:** historisch overgenomen uitvoering zonder hier aangetoond voordeel.
**FAMILIAR:** bekende interactie met blijvende praktische waarde.
**EVOLVABLE:** waardevol concept, andere uitvoering onderzoeken.
**FUTURE:** nieuwe combinatie met toetsbare maar nog onbewezen meerwaarde.
Deze labels zijn ontwerpbeoordelingen, geen historische of gebruikersmetingen.

| Element | Label | Ontwerpactie |
|---|---|---|
| Desktop als lege wallpaper met appiconen | EVOLVABLE | Leegte mag; bij werk bevat de omgeving objecten en hervatpunten, geen verplicht dashboard |
| Venster als begrensde werkeenheid | FAMILIAR | Behouden voor afbakening, focus en externe appinhoud |
| Appnaam als grootste vaste venstertitel | EVOLVABLE | Materiaaltitel eerst; gereedschapidentiteit herkenbaar en vertrouwd ernaast |
| Permanent dubbele titelbalk en appwerkbalk | LEGACY | Combineer waar de veiligheidsgrens dat toelaat; geen twee contextloze rijen |
| Sluiten, vergroten, verplaatsen en herstellen | FAMILIAR | Zichtbaar bereikbaar houden; geen uitsluitend verborgen hoeken of gestures |
| Dock als lijst van draaiende appmerken | EVOLVABLE | Werkwissel toont object/taak met actieve tool; apps apart vindbaar |
| Launcher als verplicht appgrid | LEGACY | Eén zoekingang met gelabelde resultaten voor materiaal, tools en systeemacties |
| Tijd, verbinding en opname-indicator | FAMILIAR | Vaste positie, geen wisselende indeling tijdens lezen of klikken |
| Permanente globale navigatiesidebar | EVOLVABLE | Alleen waar categorieën echt navigatie geven; in werk contextueel paneel |
| Contextmenu bij selectie | FAMILIAR | Blijft; toont acties voor precies die selectie en dezelfde keyboardroute |
| Cards voor ieder klein gegeven | LEGACY | Rijen voor vergelijkbare data; cards alleen voor zelfstandige werk-/keuze-eenheden |
| Schaduw en overlap | EVOLVABLE | Eén dieptegrammatica, niet per app een andere elevation |
| Typografie als hiërarchie | FAMILIAR | Titel, inhoud en status krijgen verschillende rollen, geen overal displayletters |
| Permanente lage informatiedichtheid | LEGACY | Dichtheid door gebruiker/taak, niet steeds automatische herschikking |
| Universele afgeronde pilvorm | LEGACY | Vorm volgt rol: werkvlak, bediening, tijdelijke keuze, status |
| Accentkleur voor alles wat actief is | EVOLVABLE | Focus, selectie, bewerking en systeemactiviteit apart coderen |
| Motion als open-/sluitspektakel | LEGACY | Beweging verklaart herkomst en bestemming |
| Dezelfde objectanker door tool- en apparaatwissel | FUTURE | Objectidentiteit blijft; rechten en werkelijke versie opnieuw controleren |
| Toegang en opslaan als objectgebonden status | FUTURE | Niet afhankelijk van een app die zichzelf vertrouwd noemt |

**Screenshotvraag:** zonder logo moet een combinatie van objectankers,
lokale toolaansluitingen en de eigen werkcompositie herkenbaar zijn.
Of mensen dat daadwerkelijk herkennen blijft een afkeur-/doorgaanproef,
niet een eigenschap die we in een tabel kunnen bewijzen.

## 3. Eerst informatie, dan vorm

| Vraag | Primaire zichtbare informatie | Alleen op verzoek |
|---|---|---|
| WORK: waar ben ik mee bezig? | Titel van huidige werkcontext; andere open werkzaamheden | Projectstructuur en ordeningsregels |
| OBJECT: wat bewerk ik? | Naam, type en huidige weergave van materiaal | Object-ID, formaat, volledige metadata |
| CONTEXT: wat hoort erbij? | Door gebruiker gekoppelde referenties en passende tools | Relaties en herkomst in een inspectiepaneel |
| ACTION: wat kan ik doen? | Acties op object of selectie, met duidelijke scope | Alle commando's en sneltoetsen |
| SYSTEM: wat doet het apparaat? | Tijd, verbinding, energie waar aanwezig, gevoelig apparaatgebruik | Achtergrondprocessen en resourcegrafieken |
| HISTORY: wat is gebeurd? | Opgeslagen/niet opgeslagen/onzeker; laatste bevestigde versie | Revisiegeschiedenis, fouten en herstelpreview |
| CONTINUITY: hoe ga ik verder? | Dit apparaat; gekozen gekoppelde bestemming en overdrachtstatus | Conflicten, afzonderlijke kopieën en routegegevens |
| TRUST: wie mag wat? | Toolidentiteit en relevante toegang; nieuwe aanvraag expliciet | Alle verleende rechten en intrekkingshistorie |

Een relatie voorstellen is niet automatisch toestemming. Een bestand naar een
tool slepen kan een toegangsaanvraag starten, maar deelt nooit stil de hele
collectie, lokale index, NAS of netwerkmachtiging.

### Lagen die niet allemaal tegelijk zichtbaar zijn

| Laag | Ruimtelijke functie | Verschijnen/verdwijnen |
|---|---|---|
| Background | Stabiele omgeving en lege ruimte | Altijd; geen live inhoud achter de aanmelding |
| System | Globale toestand en vaste ingang | Essentiële status blijft, details op klik |
| Workspace | Plaats en relaties van huidig werk | Zichtbaar in werk/overzicht; niet een tweede dashboard |
| Content | Document, code, media, appinhoud | Dominant; leesvlakken blijven ondoorzichtig |
| Transient | Menu, gereedschapskeuze, melding | Verankerd aan bron; weg op sluiten/Escape waar veilig |
| Assistive | Zoeken en lokale hulp | Op aanvraag; selectie en invoerscope zichtbaar |
| Security | Nieuwe autoriteit of ingrijpende handeling | Vertrouwd systeemvlak; geen app-effecten erdoorheen |

Semantische lagen zijn geen vrijbrief voor extra processen, algemene capabilities
of gedeelde appbuffers. Technische toedeling volgt pas na ontwerpkeuze.

## 4. Vijf werkelijk verschillende architecturen

Alle vijf gebruiken dezelfde voorbeeldtaken: **Tuinplan** schrijven,
**Begroting** bewerken, **Foto's** vergelijken en een **Webreferentie**
bekijken. Een browser is een eigen tool/proces met eigen netwerkrechten;
vier werkzaamheden betekenen niet dat vier volwaardige native tools al bestaan.

Compositiereferentie: 1920 x 1080, logische schaal 100%, oorsprong linksboven.
`(x,y,w,h)` beschrijft ontwerpgeometrie, geen native ondersteunde buffer.
Elk concept krijgt ook compact, donker, high-contrast en keyboard-only.
Verschillen in kleur alleen mogen de vergelijking niet sturen.

### R1 — Veld: objecten hebben een plek, tools sluiten erop aan

**Organisatie:** meerdere blijvende materiaalvlakken in één werkveld.
Een vlak hoort bij het materiaal; een gereedschap kan worden vervangen zonder
dat naam, plaats, referenties of geschiedenis verdwijnen. Gewone vrije
vensters blijven beschikbaar voor apps die dit objectcontract niet ondersteunen.

**Concrete compositie:** systeemlijn `(24,16,1872,32)`;
Tuinplan `(64,96,1040,760)`, Referentie `(1136,96,720,352)`,
Begroting `(1136,480,720,376)`. Foto's heeft een hervatanker in de
werkstrook `(64,920,1792,112)`, geen onleesbare vierde miniatuur.
De strook bevat vier objecttitels plus toolnaam en opslagstatus.
Alle vier blijven rechtstreeks bereikbaar; via 'Vergelijk' kan een 2 x 2
split met echte leesbare werkvlakken worden gekozen.
Die expliciete vierwerkcompositie gebruikt Tuinplan `(64,96,880,392)`,
Begroting `(976,96,880,392)`, Foto's `(64,520,880,392)` en Webreferentie
`(976,520,880,392)`. De objectkoppen en werkstrook blijven bereikbaar.
Dit onderzoekt echt simultaan werk, niet alleen een editor en rekenmachine.

**Eigen visuele signatuur:** materiaalnaam en versiestatus op een uitgespaarde
koprand; toolbediening sluit als een smallere band aan de geselecteerde rand aan.
Dat is geen zwevende losse dock. Inhoud blijft op hetzelfde anker bij toolwissel.
Inactief werk houdt volle tekstleesbaarheid, met neutralere rand.

**Vorm:** krijtachtige neutralen, inkttekst, irisaccent als eerste studiekleur;
Open Sans 400/600 als vergelijkingsbasis. Vlakke inhoud, lokale matte
translucency boven referentiegebieden, compacte eigen 24-unit-iconen.
Motion koppelt toolband aan object, niet appicoon aan taakbalk.

**Sterk:** sluit direct aan op Materiaal; bekende sleep/split/wisselacties.
**Zwak:** kan alsnog een restyled desktop worden als objectanker slechts
een hernoemde titelbalk is. Toolwissel moet echt objectcontinuïteit laten zien.
**Risico:** vrije plaatsing vraagt overzicht en leesvolgorde; expliciete
ordening en een lineaire toegankelijke equivalent zijn verplicht.
**Kosten:** meerdere backing stores en damage bij overlap; compositie kan
zonder blur functioneren. Vier werkzaamheden overschrijden de huidige
twee-appfixture en vragen later een afzonderlijk resourcecontract.
**2035-hypothese:** objectcontinuïteit blijft nuttig; randlicht en glasmode zijn
vervangbaar. Geen wereldprimeurclaim voor documentgerichte omgevingen.

### R2 — Lens: dezelfde taak door verschillende contextlenzen

**Organisatie:** één hoofdonderwerp, wisselen tussen lezen, vergelijken,
bewerken en organiseren. Het OS presenteert niet vier losse vensters, maar
een taakcompositie die relevante referenties tijdelijk bijhaalt.
Geen AI die zelfstandig beslist wat belangrijk is: gebruiker kiest de lens.

**Concrete compositie:** hoofdinhoud `(320,88,1280,848)`;
vaste taakselectie `(24,88,264,848)`. In 'Vergelijk' wordt het hoofdvlak
twee kolommen van 624 px met 32 px tussenruimte. Contextlade
`(1320,128,544,704)` verschijnt alleen na 'Referenties'; geen automatische
herindeling onder de pointer. Een zichtbaar lenslabel blijft op vaste positie.

**Vorm:** strakke typografische hoofdvlakken; saliegroene studieaccenten,
weinig randen, referenties in gedempte matte laag; identieke UI-typebasis.
Diepte is hoofdzakelijk schaal/uitsnede en contrast, niet schaduw.
Motion laat een referentie uit zijn zichtbare anker komen.

**Sterk:** rustig lezen en schrijven; context dichtbij zonder permanente
bedieningswanden. **Zwak:** minder geschikt voor willekeurige multitasking.
**Risico:** modi kunnen verwarren; elk lenslabel en backpad blijft zichtbaar.
**Kosten:** minder simultane grote vlakken, wel meer layout- en focuslogica.
**2035-hypothese:** taakgerichte composities zijn duurzaam; te slimme
automatische contextselectie zou snel hinderlijk worden.

### R3 — Ritme: werk is een hervatbare tijdlijn

**Organisatie:** activiteiten met resultaten en onderbrekingen staan centraal.
Niet de vensterpositie maar het laatste bevestigde werkmoment is de ingang.
Een activiteit mag meerdere materiaalobjecten bevatten; het is geen nieuw
bestandseigenaarschap en geen tweede verplichte projectadministratie.

**Concrete compositie:** vier activiteiten als horizontale lanes binnen
`(64,88,1792,224)`; actieve stap `(64,344,1224,624)`;
resultaat/historie `(1320,344,536,624)`. Tijdlijn is opvraagbaar en blijft
geen live ticker. De vier lanes heten 'Tuinplan', 'Begroting', 'Foto's',
'Webreferentie', niet de namen van apps.

**Vorm:** donkerblauwe inkt en warm lichtvlak; stapmarkeringen en lijnen
in plaats van een muur met cards; compacte statuspictogrammen.
Motion schuift een bevestigde versie naar het historievlak, nooit een
nog onzekere write naar 'klaar'.

**Sterk:** onderbreken, later hervatten, fouten verklaren.
**Zwak:** urgent werk zonder natuurlijke stappen wordt kunstmatig.
**Risico:** tijdlijn kan gevoelige activiteit onthullen; lokaal, gebruikersscope,
bewaarbeleid en duidelijke wis-/uitzetroute vereist.
**Kosten:** index en herstelmetadata belangrijker dan renderbelasting.
**2035-hypothese:** hervatten blijft relevant; chronologie als universeel
organisatiemodel is mogelijk een trend of verkeerde abstractie.

### R4 — Werkstuk: één object, meerdere gereedschapsweergaven

**Organisatie:** het materiaal vormt één samenhangend werkstuk met meerdere
gekoppelde views: tekst, gegevens, afbeelding, inspectie of geschiedenis.
Tools openen niet een eigen hoofdvenster, maar een begrensde weergave van
hetzelfde geselecteerde object. Meerdere onafhankelijke werkstukken blijven
via overzicht en split bereikbaar.

**Concrete compositie:** objectkop `(48,64,1824,56)`; tekstview
`(48,144,1144,760)`; inspectie/referentie `(1224,144,648,760)`;
toolaansluitingen `(48,928,1824,96)`. Vier gelijktijdige werkzaamheden
worden in een expliciete 2 x 2 werkstukindeling getoond, niet als tabs
waarvan drie verdwijnen. Iedere view toont toolidentiteit en toegewezen scope.

**Vorm:** paper/ink als Veld, maar geen individuele schaduwen rond views:
gemeenschappelijke buitenrand en heldere interne splitlijnen.
Een amber studieaccent markeert geselecteerde view, niet de volledige app.
Iconen hebben objecttype als basis en een kleine toolmarkering.
Motion laat een view naast zijn object ontstaan; vervangen verandert
de weergave, niet de objecttitel.

**Sterk:** sterk materiaalgericht; geschikt voor professionele vergelijking.
**Zwak:** arbitraire apps, browsers en games passen niet vanzelf in een
multiview-objectcontract. Gewone appvlakken mogen blijven bestaan.
**Risico:** simultaan bewerken vraagt expliciete revisies en conflictgedrag;
visuele nabijheid mag geen gedeelde schrijfrechten suggereren.
**Kosten:** view-/revisiebrokers en resourcebudgetten zijn de zware vraag,
niet blur. Dit kan niet alleen door de compositor te restylen.
**2035-hypothese:** meerdere perspectieven op hetzelfde materiaal zijn duurzaam;
een overal verplicht multiview-framework zou te rigide zijn.

### R5 — Verbinding: relaties en bestemmingen als hoofdstructuur

**Organisatie:** materiaal, mensen, apparaten en resultaten vormen een
expliciet navigeerbare relatiekaart. De gebruiker kiest een verbinding,
bijvoorbeeld 'Tuinplan op dit apparaat -> gekoppelde NAS', en opent het
bijbehorende werk. Geen zichtbare netwerkkaart vol technische nodes.

**Concrete compositie:** inhoud `(48,96,1152,832)`; relatiepaneel
`(1232,96,640,832)` met maximaal vijf zichtbare, gelabelde relaties.
Onder `(48,952,1824,80)` staan vier lopende werkzaamheden als
'onderwerp -> resultaat/bestemming'. Een lineaire lijst is de standaard
alternatieve representatie, niet een achteraf aangebouwde noodmodus.

**Vorm:** scherpe contentblokken met verbindingen alleen bij selectie;
petrolaccent, normale UI-letters, duidelijke type-/locatie-iconen.
Motion volgt de gekozen route maar eindigt pas bij echte ontvangstbevestiging.
Geen permanente deeltjes, gloeiende lijnen of wereldbol.

**Sterk:** laat continuïteit en toegang begrijpelijk samen zien.
**Zwak:** veel gewone taken hebben helemaal geen interessante relatiekaart.
**Risico:** onterecht vertrouwen door nabijheid of een lockicoon;
identiteit/inhoudstoegang/opslagbeschikbaarheid apart labelen.
**Kosten:** relatie-index, rechtenfiltering en stale-stateafhandeling domineren.
**2035-hypothese:** multi-devicecontext blijft nuttig; een grafische kaart als
primaire desktop kan juist een demonstratie-interface blijken.

## 5. Dezelfde twintig toestanden, vijf verschillende composities

Dit zijn **tekstuele compositiestudies**, niet geproduceerde beelden of
werkende schermen. Ze leggen vast hoe een compleet OS in elke architectuur
kan passen. Bootdiagnostiek, herstel en rechten behouden expliciete bediening.

| Toestand | Veld | Lens | Ritme | Werkstuk | Verbinding |
|---|---|---|---|---|---|
| 01 Boot | Vast materiaalanker met fase eronder | Centrale status, nog geen lens | Alleen huidige bootfase, geen gebruikershistorie | Neutrale objectloze basis | Alleen dit apparaat, geen ongeauthenticeerde peers |
| 02 Installer | Doelschijf als keuzevlak, stappen eraan gekoppeld | Eén stap per lens met vaste terugactie | Uitgevoerde/volgende stap naast inhoud | Installatie als systeemwerkstuk met samenvatting | Bron, doelschijf en gevolg; geen onduidelijke pijlen |
| 03 Setup | Profiel/opslag/bediening als opeenvolgende keuzes | Taal en bediening in dezelfde vaste compositie | Drie omkeerbare inrichtingsstappen | Profielwerkstuk met controle vóór voltooiing | Begin lokaal; koppelen is optionele volgende relatie |
| 04 Login | Neutraal anker, profielkeuze en geheimveld | Eén duidelijke aanmeldtaak | Geen eerdere persoonlijke activiteiten zichtbaar | Geen objectpreviews vóór authenticatie | Apparaat/profiel; geen document- of contactnamen |
| 05 Lege desktop | Ruim veld, 'Nieuw' en 'Open'; geen nepactiviteit | Kies/open onderwerp; taaklijst echt leeg | 'Begin werk'; geen gefabriceerde tijdlijn | Maak/open werkstuk | Dit apparaat, 'Open materiaal'; geen verplichte kaart |
| 06 Dagelijks werk | Groot object en twee referentievlakken | Lees-/bewerkcompositie met expliciete lens | Actieve stap met laatste bevestigde resultaat | Actieve tekstview en inspectie van hetzelfde object | Inhoud met geselecteerde relevante relaties |
| 07 Vier werkzaamheden | Drie leesvlakken en vier hervatankers; 2 x 2 op verzoek | Vier vaste taakingangen plus gekozen split | Vier lanes met één of gesplitste actieve stap | Vier werkstukken in 2 x 2 met objectkoppen | Vier onderwerpen; relaties van alleen de selectie |
| 08 Materiaal | Objecten als werkvlakken; lijst bij veel objecten | Organisatielens met rijen en preview | Materiaal blijft lijst; filter op activiteit mogelijk | Object openen met aangeboden views | Objectlijst plus optioneel relatiepaneel |
| 09 Zoeken/command | Tijdelijk veld bij globale ingang; object/tools/acties apart | Zoeklens behoudt onderwerp en scope | Zoekt materiaal én hervatmomenten met labels | Object zoeken of view/action toevoegen | Zoekt object, bestemming of concrete relatie |
| 10 Settings | Eigen systeemvlak met categorieën en rijen | Systeemlens, vaste terugkeer naar werk | Systeemtaak, geen appactiviteit in historie mengen | Systeemwerkstuk met categorieviews | Dit apparaat -> instelling; gewone lijst blijft hoofdvorm |
| 11 Quick controls | Paneel uit vaste statushoek | Tijdelijke systeemlaag, lens verandert niet | Kleine statuslade buiten tijdlijn | Systeemlaag buiten toolband | Apparaatpaneel met concrete verbindingsstatus |
| 12 Notifications | Bij betrokken object, elders gelabelde inbox | Rustige melding naast onderwerp, niet over selectie | Inbox; pas echte resultaten worden historiestappen | Aan betrokken view/object, niet alle views tegelijk | Bron/gevolg in inbox; geen knipperende routekaart |
| 13 Tool switching | Toolband vervangen, objectanker blijft | Gereedschap binnen huidige lens wisselen | Gereedschap voor huidige stap, resultaat blijft | View toevoegen/vervangen met zichtbare scope | Tool op geselecteerd object, niet op alle verbonden data |
| 14 Files | Mappen/lijst/contextmenu blijven; drop bij object | Organisatielens met dezelfde conventies | Bestanden zijn objecten, geen verplichte taakregistratie | Import/open maakt werkstuk; origineel en export apart | Verplaatsen geeft benoemde bestemming en werkelijke status |
| 15 Multitasking | Slepen/split/overzicht, apps ook rechtstreeks | Expliciete split, geen automatische focuswissel | Parallelle lanes; stappen pauzeerbaar | Meerdere werkstukken en views met afzonderlijke focus | Parallelle inhoud, kaart volgt alleen bewuste selectie |
| 16 Toestemming | Vertrouwd vlak gekoppeld aan verzoekend object | Beveiligingslaag boven lens, context leesbaar | Aanvraag noemt stap en object, niet hele activiteit | Tool/view en objectrechten exact genoemd | Peer, object en actie; pijl is geen toestemming |
| 17 Error | Foutstrook op object; niet-opgeslagen werk blijft | Fout in taak; terug en herstel zonder lensverlies | Mislukte stap blijft mislukt, retry apart | Fout in betrokken view; object blijft toegankelijk waar toegestaan | Onderbroken relatie, lokale kopie blijft; geen uitwijkroute |
| 18 Update | Systeemvlak naast werk; save/restart apart | Update als expliciete systeemtaak | Download/controle/herstart aparte ware stappen | Systeemwerkstuk met impact op open werk | Gekozen updatebron, geen persoonlijke context gekoppeld |
| 19 Recovery | Dezelfde type/rand/actietaal; technische details op verzoek | Eén herstelkeuze per stap, duidelijke terugweg | Werkelijk hersteljournaal, geen aangenomen succes | Herstelpreview en nieuw resultaat, origineel zichtbaar | Beschikbare kopieën met geverifieerde identiteit en grenzen |
| 20 Dark | Donker veld, heldere inhoudslagen, geen zwarte gaten | Donkere lens met volwaardige tekstcontrastrollen | Zelfde lanes en statusbetekenis | Gemeenschappelijke donkere rand/viewregels | Zelfde relaties, geen neonnetwerk |

Installer en recovery hebben geen dure compositor nodig om dezelfde identiteit
te dragen: letterhiërarchie, contouren, focus, spacing en actietaal zijn de
basis. Vóór font-/grafische initialisatie mag een eenvoudige diagnostische
fallback zichtbaar zijn. Niet doen alsof die dezelfde rijke renderer heeft.

## 6. Concreet visueel proefrecept voor Veld en Werkstuk

De waarden hieronder zijn **studiewaarden** voor vergelijkbaar vervolgonderzoek,
geen definitieve tokenbron of opdracht om huidige code te wijzigen.
Veld en Werkstuk moeten eerst met dezelfde kleuren en letters worden beoordeeld,
anders testen we smaak in plaats van architectuur.

### Kleurrollen

| Tokenrol | Light | Dark | High-contrast donker |
|---|---|---|---|
| environment | `#F4F3EF` | `#15171D` | `#000000` |
| content/surface | `#FFFFFF` | `#242730` | `#000000` |
| text.primary | `#202229` | `#F1F0F5` | `#FFFFFF` |
| text.secondary | `#595B66` | `#B6B7C2` | `#FFFFFF` |
| boundary.interactive | `#7B7D88` | `#858898` | `#FFFFFF` |
| accent/focus | `#5144BA` | `#B6ACFF` | `#00FFFF` |
| accent.on-fill | `#FFFFFF` | `#1C1640` | `#000000` |
| selection.fill | `#E8E4FA` | `#393052` | `#FFFF00` |
| selection.text | `#202229` | `#F1F0F5` | `#000000` |
| danger.text | `#A32C3B` | `#FFB0BA` | `#FFFFFF` + fouticoon |
| warning.text | `#855000` | `#FFD998` | `#FFFFFF` + waarschuwingsicoon |
| success.text | `#14624F` | `#93D6BF` | `#FFFFFF` + bevestigingsicoon |

Semantische tekstkleuren worden op content/surface gebruikt, niet op willekeurig
glas of accent-fill. High-contrast licht is dezelfde binaire constructie met
witte vlakken en zwarte tekst/randen; focus `#002D72` op wit heeft een
berekend sRGB-contrast van circa 12,98:1. Selectie is zwart met witte tekst,
accentknoppen gebruiken witte tekst op `#002D72`.
Hoog contrast gebruikt geen blur, tintafhankelijkheid of schaduw als grens.
De 33 ondoorzichtige kleurparen uit bovenstaande drie modi zijn berekend:
geen paar onder de voorgestelde minimumgrens. Dit bewijst geen contrast van
blur/alpha, antialiased randpixels of werkelijk native gerenderde schermen.

**Dynamic color:** optionele, lokaal afgeleide accentfamilie uit een expliciet
gekozen achtergrond, niet uit open documenten. Perceptuele tonale rollen
zoals in Material Color Utilities zijn bruikbaar; de Android-stijl overnemen
is niet nodig. Palette genereren bij wijziging, niet ieder frame.
Focus- en semantische kleuren krijgen contrastvalidatie; fout/warningbetekenis
blijft vast. Gebruiker kan een vaste kleur kiezen. Geen achtergrondafhankelijke
verandering tijdens slepen, geen inferentie van werkcontext uit foto's.

### Typografie, ruimte en geometrie

| Onderdeel | Studiewaarde | Reden en open grens |
|---|---|---|
| UI-family | Open Sans 400/600; geen systeemfontfallback in native claims | Sluit aan bij huidige gepinde atlas; uitbreiding vraagt licentie/herkomstcontrole |
| Kleine metadata | 13 px / 18 px regelhoogte | Niet voor essentiële lange uitleg; huidige atlas heeft eigen discrete maten |
| Controls/lijsten | 15 px / 22 px | Compact maar niet flinterdun |
| Body/tekst | 16 px / 24 px | Rustige leestekst; document mag eigen font hebben |
| Objecttitel | 22 px / 28 px, 600 | Materiaal vóór appmerk |
| Pagina-/inrichtingstitel | 32 px / 40 px, 600 | Alleen echte hoofdhiërarchie |
| Code/getallen | Monospaced documentfont; tabulaire cijfers waar ondersteund | Geen proportionele caret zonder overeenkomstige selectielogica |
| Spacing | 4, 8, 12, 16, 24, 32, 48, 64 px | Eén scale, geen component-eigen willekeur |
| Controls | 40 px normaal; 32 px dense; 48 px touch | Hitgebied minimaal 44 x 44 in normale/touchproef; dense keyboardroute apart toetsen |
| Radius | control 6; menu 10; werkvlak 12; dialog 14 px | Geen universele pillen; splitranden recht |
| Rand | 1 px structureel; 2 px actief; 3 px focus | Focus heeft 2 px tussenruimte en blijft zichtbaar op beide lagen |
| Iconen | 24-unit-grid, 20/24/32 optische uitvoeringen | Niet simpelweg dunne 24 px-versie verkleinen |

Het ontwerpdoel omvat locale, Unicode, bidi en complexe tekstvorming. De huidige
Open Sans-atlas doet dat **niet** algemeen. Geen SF Pro/Segoe als vrij
herdistribueerbare native standaard aannemen. Internationaal testen met lange
Nederlandse/Duitse labels, Arabisch/RTL, CJK en niet-Latijnse invoer.
UI schalen en tekstgrootte zijn afzonderlijke voorkeuren; 200% mag inhoud
herordenen, niet labels afsnijden of essentiële acties verwijderen.

Iconen: normale 24-unit-symbolen krijgen een 1,75-unit stroke, open vormen
waar herkenning dat toelaat, gesloten fill voor compacte opname-/foutstatus.
Op 20 px de stroke optisch verzwaren, op 32 px detail niet automatisch
vermenigvuldigen. Ronde lijnuiteinden, 2-unit geometrische hoekbasis; document-
en mapvormen behouden hun herkenbare silhouet. Actieve status heeft ook een
label of vormwijziging; geen ingevuld icoon als enig focussignaal.
Appiconen mogen een eigen merk dragen, systeemiconen vormen één familie.
Richtingsiconen spiegelen alleen wanneer de betekenis met RTL meedraait;
afspelen, hardware en logo's niet automatisch spiegelen.

### Diepte, translucency en illumination

Illumination betekent hier **luminantie als toestand**, niet decoratief licht.
Een opgelichte rand heeft altijd een icoon/label of contour-equivalent.

| Niveau | Taak | Studiebehandeling |
|---|---|---|
| L0 environment | Ruimte en rustige oriëntatie | Ondoorzichtig; hoogstens statische tintverdeling, geen doorlopende animatie |
| L1 workspace | Relaties en inactief werk | Neutralere koprand; content blijft leesbaar, geen hele-app-dimming |
| L2 active content | Actief bewerk-/leesvlak | Ondoorzichtig; 2 px lokale actieve contour en toolidentiteit |
| L3 transient | Command/menu/quick controls | 88-96% tintdekking; lokale blurproef met radius 12 logische px; rand plus schaduw |
| L4 intervention | Toegang/herstel/ingrijpende actie | Ondoorzichtig vertrouwd vlak; contextscrim, geen refractie of doorschijnende documenten |

De 12 px blur is een onderzoeksparameter, geen benchmarkresultaat. Studie voor
L3-schaduw: offset `(0,8)`, radius 24, max alpha 0,16 light / 0,32 dark,
plus contactschaduw `(0,2)`, radius 6. Geen stapel van vijf schaduwen per card.
Zonder effect blijft dezelfde geometrie staan met opaque fill en contour.

| Toestand | Functioneel licht-/vormsignaal | Niet doen |
|---|---|---|
| Focus | Vaste iriscontour en actieve toolnaam | Het hele venster laten gloeien |
| Bewerken/niet opgeslagen | Klein blijvend markertje + 'Niet opgeslagen' | Kleur alleen of pulserende documentachtergrond |
| Opslaan | Korte toestandsovergang bij objectstatus | Succesanimatie vóór echte duurzame bevestiging |
| Sync | Label 'Synchroniseren' en begrensde voortgang; rust na afronding | Permanente netwerklichtshow of 'veilig' uit animatie afleiden |
| Camera/microfoon | Blijvende systeemindicator met appidentiteit en stoproute | Alleen een tijdelijk groen stipje dat vanzelf verdwijnt |
| Lokale AI | Bij selectie: 'Lokaal bezig', annuleren, scope | Glow die intelligentie of netwerkloze afdwinging pretendeert |
| Beveiligingsaanvraag | Duidelijke vaste contour + systeemidentiteit | App mag zelf trusted effect of schermlaag aanvragen |

### Window chrome en system surface

Bij Veld krijgt een objectvlak een **40 px kopzone**: objecttitel links,
kleine toolnaam ernaast, sluiten en vergrotings-/layoutactie rechts.
Sluiten blijft standaard zichtbaar. Verplaatsen kan aan de lege kopzone;
resizen aan zichtbare begrenzing; een layoutmenu biedt split en herstellen.
Minimaliseren betekent 'naar hervatstrook', niet verwijderen van materiaal.
Alt+Tab wisselt tussen open werkzaamheden; een gelabelde alternatieve
appwisselroute blijft beschikbaar. Alt+F4 sluit de tool/view met bestaande
save-/annuleerregels, niet stil het materiaalobject.

Pointer proximity mag extra split-/verplaatskeuzes tonen, maar verandert nooit
het sluitdoel onder de pointer. Tab/focus en touch maken dezelfde opties
expliciet bereikbaar. Geen verplichte nieuwe gebaren of verborgen randtrucs.
Werkstuk gebruikt één objectkop en kleinere gelabelde viewkoppen; afsluiten van
een view heeft een ander label dan afsluiten van het hele werkstuk.

De **werkstrook** is niet slechts een taakbalk met nieuwe vorm:
zij noemt werkzaamheden, betrokken materiaal en tool, met hervatstatus.
Processen zonder materiaal, zoals games, krijgen een herkenbare tooltaak.
Fasen worden door expliciete input gekozen en hebben stabiele hitgebieden:

| Fase | Zichtbaar | Trigger |
|---|---|---|
| Passive/Calm | Huidig onderwerp, 'Werk', zoekingang; essentiële systeemstatus elders vast | Gebruiker kiest Calm; geen automatische verdwijning tijdens invoer |
| Work/Normal | Open werkzaamheden met objecttitels en toolmarkering | Normaal werken |
| Switch | Overzicht van werkvlakken, actieve tool per taak | Klik Werk of sneltoets |
| System | Audio, netwerk, apparaten, energie, toegang; werkstrook niet herschikken | Klik vaste statushoek |
| Command | Zoeken met scope, object/tools/acties als aparte groepen | Zoekknop of instelbare sneltoets |

**Calm/Normal/Dense** zijn gekozen dichtheden, geen autonome persoonsinschatting.
Dense voegt kolommen en rijen toe in Materiaal, Settings en professioneel werk;
de vaste system-/veiligheidsingangen blijven op dezelfde plaats.
Transient controls mogen contextueel zijn, hoofdnavigatie mag niet onverwacht
van betekenis wisselen.

### Componenttaal: geen losse kit

| Begrip | Definitie | Voorbeeld |
|---|---|---|
| TOKEN | Semantische, herbruikbare ontwerpwaarde | `text.primary`, `spacing.4`, `motion.attach` |
| COMPONENT | Eén bediening met staten en focusgedrag | Button, field, menuitem |
| PATTERN | Samenhangende handeling met fout-/herstelroute | Tool koppelen, object bewaren, toegang aanvragen |
| SURFACE | Begrensd compositor-/navigatievlak | Objectvlak, commandlaag, systeeminterventie |
| APPLICATION | Tool met eigen inhoud, lifecycle en rechten | Tekstbewerker, browser, rekenmachine |

| Component/patroon | Concreet voorschrift |
|---|---|
| Buttons | Werkwoordlabels; één primaire actie per beslisgroep; default/hover/pressed/focus/disabled/loading afzonderlijk |
| Fields | Blijvend label, caret en selectie; fout nabij invoer; disabled blijft leesbaar |
| Menus | Anker bij bron, 8 px binnenpad, 40 px rijen normaal; acties gegroepeerd, shortcuts rechts |
| Dialogs/sheets | Benaming en gevolg eerst; objectgebonden sheet waar niet globaal; focus terug naar bron |
| Cards/lists | Cards voor zelfstandige onderwerpen; lijsten voor scannen/vergelijken; randen niet ieder detail omlijsten |
| Toolbars | Selectiegebonden, beperkte primaire acties, overflow gelabeld; geen dubbele permanente ribbon |
| Materiaal | Lijst/grid naar taak, naam/type/locatie/ware revisiestatus; geschiedenis bij object |
| Settings | Categorieën plus rijen; search; huidige waarde en gevolg; destructive actiegroep afzonderlijk |
| Quick controls | Belangrijkste status direct; meer instellingen via gelabelde route; geen knop zonder backend |
| Notifications | Bron en gevolg, niet alleen appmerk; groeperen op onderwerp; niet-urgent respecteert focus |
| Installer/setup | Dezelfde typen/randen/focus; doelschijf en wisgevolg expliciet, lokaal beginnen mogelijk |
| Login | Dezelfde kop-/invoertaal; geen persoonlijke previews zonder autoriteit; echte authenticatie blijft vereiste |
| Recovery/update | Werkelijke fase, beschikbaar resultaat en herstelactie; details op verzoek, fouten niet verbergen |

### Motion: ruimtelijke continuïteit zonder spektakel

| Gebeurtenis | Studiebeweging | Reduced motion |
|---|---|---|
| Menu/command openen | 140 ms ease-out, 8 px vanuit anker, geen schaal van tekst | Direct, hoogstens 80 ms opacity |
| Menu sluiten | 100 ms ease-in terug naar bron | Direct |
| Tool koppelen | 180 ms band aan objectrand; objectnaam blijft staan | Directe plaatsing + 'Tool actief' |
| Tool vervangen | 140 ms inhoudscrossfade; geen verplaatsing objectanker | Directe vervanging |
| Split/maximaliseren | 220 ms geometrie ease-in-out; geen veerovershoot | Directe eindlayout |
| Werk wisselen | 220 ms horizontale continuïteit in overzicht, stationaire systeemlijn | Direct, naam en focus expliciet |
| Drag/drop | Pointer volgt zonder easing; ghost en geldig doel; annuleren keert terug | Geen reisanimatie na loslaten |
| Bewaren | 120 ms statusovergang pas na echte bevestiging | Tekst/icoonwissel |
| Overdracht naar apparaat | 220 ms van objectanker naar benoemde bestemming na bevestigde fase | Fase/voortgangstekst |
| Melding | 160 ms uit inboxanker; nooit over actief invoerdoel | Direct in inbox |
| Fout/toestemming | Geen shake, bounce of pulserende scrim; duidelijke focus | Dezelfde directe presentatie |
| Loading | Geen eindeloze ambient animatie; indeterminate alleen bij echte onbekende duur | Statische 'Bezig', waar mogelijk voortgangtekst |

Tijden zijn te testen parameters. Invoer wordt niet 220 ms geblokkeerd omdat
een animatie loopt. Bij interruptie opnieuw vanaf de huidige geometrie naar
de nieuwe eindstaat; geen rij achterstallige animaties.

### Responsive en toegankelijkheid

1920 x 1080 ondersteunt meerdere leesvlakken. Op 1280 x 720 krijgen primaire
inhoud en één referentie prioriteit; overige werkzaamheden blijven zichtbaar
in de werkwissel. Bij 200% is 1280 x 720 effectief een compacte omgeving:
één werkvlak, context als bereikbare sheet, nooit simpelweg vier miniaturen.
Lay-out volgt beschikbare logische breedte en fontmetrics, niet apparaattype.

Geen overname van desktop-naar-touch door alles alleen te vergroten:
touch heeft zichtbare layoutacties, grotere doelen en geen hoververeiste.
Toetsenbord heeft vaste leesvolgorde, selecteerbaar overzicht, Escape/terug,
geen focusval en duidelijke active/inactive versus keyboardfocus.
De semantische boom weerspiegelt object -> view -> controls; ruimtelijke plaatsing
vervangt geen logische ordening. Schermlezer meldt opslaan/fouten bij echte
statuswissel, niet bij elk animatieframe.

## 7. Vergelijking zonder schijnbewijs

Schaal 1-5: ontwerpinschatting, **niet** gemeten gebruikerswaardering.
Een 5 betekent sterk ontwerpargument, niet gerealiseerde acceptatie.
Geen totaalscore: falende toegankelijkheid, vertrouwde bediening of general-purpose
gebruik mag niet door een mooiere screenshot worden gecompenseerd.

| Criterium | Veld | Lens | Ritme | Werkstuk | Verbinding |
|---|---|---|---|---|---|
| Herkenbaar als OS / overdraagbare bediening | 5 | 4 | 3 | 4 | 3 |
| Conceptueel onderscheid | 4 | 4 | 5 | 5 | 5 |
| Eigen compositie zonder logo, hypothese | 4 | 3 | 4 | 4 | 4 |
| Onderscheid Windows/macOS/GNOME/KDE | 4 | 4 | 4 | 4 | 4 |
| Visuele coherentie | 4 | 5 | 4 | 5 | 3 |
| Informatiehiërarchie en rust | 4 | 5 | 3 | 4 | 3 |
| Acht uur productief gebruik, hypothese | 4 | 4 | 3 | 4 | 2 |
| General-purpose / externe tools | 5 | 3 | 3 | 3 | 3 |
| Materiaalcontinuïteit | 5 | 4 | 4 | 5 | 4 |
| Toegankelijke lineaire representatie | 4 | 4 | 4 | 4 | 3 |
| Internationalisatie | 4 | 4 | 4 | 4 | 3 |
| Compactheid en professionele dichtheid | 4 | 4 | 3 | 5 | 3 |
| Touchgeschiktheid / schaalbaarheid | 4 | 4 | 3 | 4 | 3 |
| Renderkosten zonder dure effecten | 4 | 4 | 4 | 4 | 3 |
| Haalbaarheid met gefaseerde Nieuw-OS-uitbouw | 4 | 3 | 3 | 2 | 2 |
| Duurzaamheid richting 2035, hypothese | 4 | 4 | 3 | 4 | 3 |

**Waarom Veld eerst:** het maakt de bestaande Materiaalbelofte zichtbaar en laat
tegelijk gewone onafhankelijke apps bestaan. Het vraagt geen volledig nieuw
mentaal model vóór iemand kan schrijven of browsen.
**Waarom Werkstuk daarnaast:** het toetst of gereedschapsuitwisselbaarheid
sterker zichtbaar wordt door gemeenschappelijke objectkoppen en multiview.
**Waarom niet de andere drie:** Lens wordt snel modaal, Ritme maakt chronologie
te belangrijk, Verbinding maakt uitzonderlijke multi-devicevragen te centraal.
Hun goede ideeën mogen later gericht worden getest, niet nu in één hybride
met alle mogelijkheden worden samengevoegd.

## 8. Current / Keep / Evolve / Replace / New

Deze bevindingen zijn gelezen broncode op bovengenoemde basis, geen nieuwe
VM-test of oordeel over ongeziene screenshots.

| Surface / laag | CURRENT | KEEP | EVOLVE / REPLACE | NEW / open |
|---|---|---|---|---|
| Browser P2 | Groen/petrol CSS, system-ui, zeven materiaalflows | Taakvolgorde en scope-/foutflows | Studiewaarden niet automatisch producttokens; geen webdesktop als runtime | Zelfde gekozen semantische taal na beoordeling |
| Native graphics | Eigen scalar Canvas, Open Sans-atlas, vectoriconen | Clipping, privébuffer, proportionele labelmetrics | Palette als rollen, niet hardcoded oppervlakstijl | Algemene shaping, fallbackfonts, nieuwe maten, later GPU |
| Materiaal native | Graphics-palet voor hoofdkleuren; mono documentselectie | Opslag-/selectiegedrag en echte foutstatus | Visuele objectanker/toolrelatie onderzoeken | Volwaardige objectcatalogus en continuïteit |
| Venstersessie | Twee appvlakken; titel op appnaam; topbar en appdock; opaque kopiepad | Vertrouwde chrome, focus, damage/retained en herstel | Appcentrische kop/strook naar materiaalgerichte presentatie | Vier werkzaamheden, layoutbeheer en resourcecontract |
| Loginpreview | Afzonderlijke account/home/search-pages; guest geen echte auth | Geen nep-auth of persoonlijke vrijgave | Zelfde component-/surfacetaal als rest | Werkelijke authenticatie/keyring/lock, niet alleen mooi slot |
| Launcher | Native palet op twee appnamen; previewsearch is andere flow | Lokale zoekingang en toetsenbord | Eén resultaatgrammatica object/tools/systeem | Native zoekbackend en juiste objectrechten |
| Settings/installer/recovery | Producteisen en beperkte bestaande deelroutes | Eerlijke fout- en herstelgrenzen | Geen nieuwe fictieve complete schermclaims | Complete native surfaces met dezelfde visuele familie |

Concreet conflict: `ui/prototype/tokens.css` heeft groene accent-/tekstrollen;
`ui/graphics/src/theme.rs` heeft paper/ink/iris met een vlak graphitebureaublad.
`ui/native/src/design.rs` gebruikt graphics voor hoofdkleuren, maar nog CSS voor
onder meer danger, warning, geometrie en desktopwaarden.
`scene.rs::render_reference` heeft bovendien lokale shellkleuren.
Er is dus bestaande hergebruiklogica, maar nog niet één volledige semantische
tokenbron voor de OS-familie. Niet alles is los of dubbel; niet alles is uniform.

## 9. Hoe we technisch bij het ontwerp komen

**Geen uitvoeringsopdracht in deze fase.** Dit is een haalbaarheidsroute voor
de twee te beoordelen concepten, geen nieuwe concurrerende roadmap.
De huidige renderer bepaalt niet hoe het eindproduct eruit mag zien; hij bepaalt
wel welke beweringen we vandaag kunnen doen.

| Vervolgproef na beoordeling | Eigenaar / grens | Acceptatie vóór verder bouwen |
|---|---|---|
| Semantische rollen en componentstaten | UI-library, buiten kernel; één token-eigenaar | Dezelfde rollen voor app, shell, login/recovery; contrast en alle states |
| Veld tegenover Werkstuk | Ontwerp-only, dezelfde fixtures en kleuren | Toolwissel/hervat/opslagbegrip; geen automatische keuze door rendergemak |
| Object-/toolanker en windowbeheer | Vertrouwde sessie, apps blijven privé | Focus, close/save, split, keyboard, stale input en compositorherstel behouden |
| Lokale matte transient-laag | Compositor-private compositie | Blur/caching/damage inclusief leesradius; geen andere app ontvangt backdroppixels |
| Tekstuitbouw | Gebruikersruimte, parser-/fontbudget apart | Shaping, selectie/caret, locale en 200%; unsupported zichtbaar, geen silent fallback |
| GPU/rendergraph | Graphics-/drivercontract apart | Begrensde buffers, sync/fences, deviceverlies/reset/DMA en scalar referentiepad |
| Volledige OS-familie | Native surfaces op echte services | Function/usability/visual/accessibility én bestaande FR-gates |

Een rendergraph is een lijst van afhankelijke tekenstappen. Voor functioneel
glas: achtergrondregio -> lokale downsample -> blur -> tint -> contrastcontrole
-> tekst/controls -> display. Bronregio moet met de filterradius worden vergroot.
Verandert iets eronder, dan is ook het doorschijnende vlak beschadigd
(`damage` = opnieuw te tekenen gebied). Alleen de gewijzigde menuletters
hertekenen is dan onjuist.

CPU-proef kan later een begrensde separabele blur of downsample/upsample
vergelijken. Niet nu blind een KWin-shader of Flutter-engine porten.
GPU-versnelling vraagt echte drivers en geheugen-/commandogrens, niet slechts
een shaderbestand. Linear-light/sRGB-gedrag en premultiplied-alpha moeten als
rendercontract worden gekozen; huidige integer RGB-coverage is geen bewijs
dat dat al bestaat.

### Vooraf meetbare experimentgrenzen

| Proef | Te vergelijken | Voorstel voor doorgaan/afkeuren; nog niet gemeten |
|---|---|---|
| VD-P1 concept | Veld vs Werkstuk; 8-12 gebruikers uit meerdere OS-gewoonten | >=80% voert open/bewerk/toolwissel/hervat zonder hulp uit; geen onbedoeld sluiten/data delen |
| VD-P2 begrip | Zelfde object, andere tool; opslaan/sync/onzeker apart | >=80% herkent objectbehoud en ware opslagstatus; iedere gevaarlijke misinterpretatie eerst oplossen |
| VD-P3 identiteit | Logo en merk verwijderd, dezelfde kleurcontrole | Deelnemers herkennen terugkerende familiekenmerken op installer/login/werk/settings/recovery; geen herkenningsclaim vóór proef |
| VD-P4 toegankelijkheid | Keyboard, reduced motion, opaque/high-contrast, lange labels, RTL, 200% | Alle taken beschikbaar; text 4,5:1, betekenisvolle grenzen/focus 3:1 als minimale ontwerpgrens; native AT afzonderlijk testen |
| VD-P5 render | Opaque vs lokaal glas, vier vlakken, 1080p en 200% | Geheugen vooraf begrensd; 0 doorlopende frames wanneer idle; geen verslechtering van bestaande correcte herstel-/pixel-/sendbudgetgates |
| VD-P6 snelheid | Input-to-present en effectkosten in vastgelegde VM plus later fysiek profiel | Studieambitie p95 <=50 ms gewone input; effecten <=20% extra p95 t.o.v. opaque bij identieke workload; geen 60fps-claim zonder meting |
| VD-P7 duur | Twee uur echt lezen/schrijven/vergelijken, later langere dagelijkse proef | Vermoeidheid, onrust, gemiste status en fouten vastleggen; geen acht-uurclaim uit een screenshot |

Steekproef en percentages zijn verkennende beslisregels, geen representatief
marktbewijs. Subjectieve voorkeur én taaksucces vastleggen; lager aantal
participants niet ometiketteren als volwaardige acceptatie.
Een prototypeproef is geen native product-/securitybewijs. Bestaande
releasegrenzen mogen niet door deze nieuwe proefwaarden worden verlaagd.

## 10. Stop- en hervatpunt

Deze opdracht eindigt bij het designplan, de bronvergelijking en maximaal
twee aanbevolen richtingen. Geen definitief designsysteem, afbeeldingen,
redesignimplementatie, merge of release.

Een toekomstige interface is pas af wanneer **FUNCTION** (ware werking),
**USABILITY** (begrijpelijke taak), **VISUAL** (consistent met beoordeelde taal)
en **ACCESSIBILITY** (waarneembaar en bedienbaar) samen slagen op het passende
bewijsniveau. Dit operationaliseert FR-U04/FR-U05; het sluit die rijen niet.

Herbeoordeling: **41% -> 41%**. Dit ontwerp en externe broncode realiseren
geen nieuw native criterium. U02-U06, hardware-/menselijke beoordeling en
volledige releasevalidatie blijven op hun bestaande bewijsgrenzen open.
