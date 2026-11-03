# Native venstersessie — U01 bouwcontract

## Veld-02: objectgebonden gereedschappen en venstertoestanden

3 november 2026, expliciete vervolgopdracht; baseline `264ad5b0`.
Werkende native UI-code is leidend; aanvullend visueel onderzoek volgt alleen
een concrete ontbrekende implementatiebeslissing. De volledige gevraagde
omgeving blijft actief, niet afgedaan met deze deelstap.

**Eerste contract:** Schrijven en Lezen gebruiken dezelfde `DocumentSession`
in Materiaal. F4, een echte knop en een semantische actie wisselen zonder
opslag-RPC, kopie, heropenen of rechtenwijziging. Titel, object, tekst,
selectie/caret, scroll, dirty-state en opslagstatus blijven behouden.
Lezen staat navigeren/selecteren/kopiëren toe maar geen typen, verwijderen,
plakken of semantisch vervangen. Terug naar Schrijven verleent geen
schrijfrecht aan een read-only sessie. Wisselen tijdens opslaan of vanuit
een modal is geweigerd. Expliciet opslaan blijft de enige bewaarrichting;
dit is geen gereedschapswisseling tussen afzonderlijke processen.

**Schrijfbestanden:** `ui/native/src/{controller,render}.rs`, gerelateerde
host-/nativefixtures, `tools/{run_window_session,test_window_session}.py`,
de bestaande CI-gate en deze contract-/status-/bewijsbestanden. Vensterstates
worden onafhankelijk gebouwd in `window-session/src/scene.rs` en één
testbestand; de hoofdagent integreert input en native acceptatie.

**Acceptatie:** native UI- en beide app-hosttests, Clippy/fmt; hele
ongemaskerde documentbeelden voor Lezen, geweigerde bewerking en hersteld
Schrijven; originele volledige klembord-, isolatie-, verplaatsings-,
compositorherstel-, schijf- en cleanupgates blijven verplicht. Schaal 200%
blijft onderdeel van hostpixels en een afzonderlijk te draaien VM-profiel.
Een hosttest sluit die native schaalgate niet.

**Venstertoestanden:** Normal/Minimized met behoud van proces, appbuffers en
opslag. Zichtbare titelknop of Alt+F9 minimaliseert; werkbalk/zoeken/F6
herstelt het doel. Alt+Space of de titelmenuknop opent een echt modaal
venstermenu voor Minimaliseren, Ordenen en Dit werk/Alles tonen. Tab/pijlen,
Enter en Escape bedienen het zonder muis. Geen nep-maximaliseer/sluitknop.
Bij nul zichtbare vensters krijgt geen app toetsen of muishits. Een nieuw
shellpaneel annuleert oude appinvoercapture via INP1; verborgen vensters
verliezen ook bestaande invoer en klembordtickets. Native acceptatie toetst
menu-pixels, muisminimaliseren, toetsenminimaliseren, weigering van
invoer bij twee verborgen vensters en herstel van de echte documentbody.

**Native 200%:** `native-window-session-hidpi-probe` combineert uitsluitend
de bestaande hidpi- en opt-in fault-probe-features; geen nieuw kernelbeleid,
capability of grenswijziging. De loader kiest 1920x1080 en de bestaande
schaal2. Dezelfde runner draait met `--scale 2`, volledige afzonderlijke
1120x660-/640x608-apporakels, geschaalde invoer en expliciete shellgebieden.
Manifest, echte GOP-afmeting en profiel moeten overeenkomen; een 100%-beeld
kan geen 200%-gate openen. CI krijgt beide schalen, niet een vervangende
lichtere hidpi-smokeproef. Verplichte uitvoeringscommando's:
`python3 -B -u tools/build.py --kernel-feature native-window-session-hidpi-probe`
en de volledige runneropdracht hierboven aangevuld met `--scale 2`.
Normale distributieprofielen blijven zonder foutinjectie.

**Documentcontextmenu (Veld-03):** Menu, Shift+F10, rechterklik en een zichtbare
knop openen een modaal appmenu: Alles selecteren, Lezen/Schrijven, Opslaan,
Annuleren. Pijlen/Tab, Enter en Escape zijn voldoende. Onderliggende semantische
acties verdwijnen zolang het menu open is; oude generaties en read-only writes
blijven geweigerd. Geen nagebootste klembordgebaren of opslagrechten.
Schrijfgrens: native controller/render, menutests, bestaande native fixtures,
runner en CI. Geen kernel-/opslag-/capabilitywijziging.

Acceptatie: volledige UI-/app-hosttests en strikte Clippy/fmt; hele appbeelden
op beide schalen voor open menu, navigatie, echte toolkeuze, geblokkeerde
mutatie, annuleren en de zichtbare muisknop. Alle bestaande native gates
blijven vereist. `document_menu_gate` wordt alleen groen na de volledige
workspacegate inclusief echte muisbediening; diagnostische runs voldoen niet.
Veld-03-bewijs behoudt de toen werkelijk achtergebleven knophover.

**Pointer-leave (Veld-04):** de compositor meldt een werkelijk waargenomen
body-exit eenmaal als INP1-eventtype4, zonder buitenbodycoordinaten. De app
wist uitsluitend pointer/hover/press, niet menu, document, selectie, focus
of opslagstatus. Oude generaties en replay blijven geweigerd; het event
verleent geen klembordautoriteit en herstelt op zichzelf geen verloren input.
Een actieve drag blijft bij body-exit de bestaande Resync-annulering krijgen.
Shelloverlays, focusverlies en verborgen vensters behouden hun strengere
annulering; ook hun oude cursor verdwijnt.

Schrijfgrens: input-core enum/wire, SDK-decoder, controller/desktop, beide
appadapters, compositorrouter, gerichte protocol-/autoriteits-/pixeltests,
bestaande native fixtures/runner/CI en deze contract-/bewijsbestanden.
Geen nieuwe capability, kernelbeleid, opslag-RPC of buitenbody-muispacket.
Acceptatie: betrokken hosttests/Clippy/fmt en beide volledige native gates,
met werkelijk verschillende hoverpixels voor/na leave en behoud van het
geopende menu. Pinned apps en SDK worden samen gebouwd; oudere derdenapps
zijn geen bewezen compatibiliteitsdoel van deze tweevensterproef.

## Veld: eerste native uitvoering

**Opdracht/basis:** uitvoering na het visuele onderzoeksplan,
`d6f8e98178d73d6aeda0abe84ca404dfde03123c`, branch
`simonvanieperen-fictional-disco`. Dit is de twee-appstap van Veld,
niet het volledige ontwerp, dagelijks gebruik of nieuwe U02–U06-acceptatie.
Het oorspronkelijke U01-contract hieronder blijft gelden.

**Doel en schrijfgrens:** gedeelde visuele rollen in `ui/prototype/tokens.css`,
`ui/graphics` en de native designbridge; werkgerichte shellcompositie,
zoektermen, focus-/ordeningacties in `sdk/examples/window-session`; bijbehorende
tests en uitbreiding van de bestaande runner/CI. Een aangetoonde observatiefout
in `kernel/src/m2/m3/window_session.rs` vereist daarnaast uitsluitend een
opt-in proefdiagnostiekwijziging: de al gelogde inputteller triggert voortaan
ook een snapshot zonder kunstmatige appframecommit. Geen productkernelbeleid,
capabilities, authenticatie, cryptografie of opslagprotocollen gewijzigd.

**Bediening:** de werkbalk toont de bestaande collectie en berekening, telkens
met het echte gereedschap. Ctrl+K en de zoekknop vinden deze twee bestaande
doelen; ook document-/gereedschapstermen en begrensde lokale Finder-, Files-,
Verkenner-, Explorer-, Nautilus-, Dolphin- en Calculator-aliassen werken.
Typen voert niets uit en verleent geen recht. Dit is geen algemene
bestandzoekdienst of systeembrede synoniemenregistratie.
F7 / "Dit werk" toont alleen het actieve venster; het andere proces en zijn
werk blijven bestaan. Wisselen blijft via de werkbalk, zoeken en F6 mogelijk.
F8 / "Ordenen" herstelt de oorspronkelijke bekende posities. Beide acties
zijn ook met de muis bereikbaar; herhaalde keypresses schakelen niet
onbedoeld heen en weer en keyreleases gaan niet naar apps.

**Invarianten/fouten:** buffers, appafmetingen, offsets en presentatie-epochs
veranderen niet door deze acties. Verborgen vensters krijgen geen muishit.
Beweging en modewisseling annuleren uitsluitend een aanwezige appmuiscapture
via de bestaande invoerwachtrij. Zonder grab blijft gewone appinvoer intact.
Volledige
oude/nieuwe schaduwgrenzen, focus en paletopen/sluiten doen mee aan damage.
Onjuiste schaduwspreiding/lege geometrie faalt vóór schrijven. De achtergrond
is een statisch RGB-verloop; schaduwen hebben begrensde afstandsafname,
geen backdrop blur. Geen zelfbedachte authenticatie of nieuwe apprechten.

**Acceptatiecommando's:** de bestaande hosttests, Clippy en fmt voor graphics,
native UI, compositor en beide apps; protocolclient- en runnertegenvoorbeelden;
de bestaande prototypekleur-/toestandstests. Daarna:
`python3 -B -u tools/build.py --kernel-feature native-window-session-probe`
en `python3 -B -u tools/run_window_session.py --clipboard --workspace --timeout 1800 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"`.
De extra workspaceproef controleert exact benoemde shellgebieden, waaronder
het volledige verborgen venstergebied en het opaque zoekpaneelinterieur.
Haar nul-appbuffers leveren uitdrukkelijk geen appbeeldbewijs. Alle bestaande
ongemaskeerde volledige appbeeld-, isolation-, herstel-, schijf- en
cleanupcontroles blijven daarnaast verplicht. De verplichte CI draait beide.

**Niet geleverd:** objectgebonden gereedschapswisseling, vier vrije
werkzaamheden, resize/snapping, algemene documentmetadata in de shell,
GPU/blur, dark-/hoogcontrastmodus, motioninstellingen, OS-brede schermlezer,
installer en fysieke/menselijke acceptatie. Deze stap is geen conventionele
desktop als eindproduct: de materiële continuïteitsketen blijft noodzakelijk.
Actuele uitvoering: [bewijsverslag](../verification/veld-native-20261103/README.md).

## Oorspronkelijk U01-contract

Bronbasis: `0fe648b`, werkbranch `codex/mature-native-session-20261102`.
Dit contract beschrijft lopend werk; het sluit U01 niet en wijzigt geen punten.

## Resultaat en grenzen

Een native compositor toont Materiaal en Rekenmachine als twee afzonderlijke
CPL3-processen. Alleen de compositor bezit DisplaySource; alleen de inputbron
bezit InputSource. Applicaties krijgen uitsluitend hun eigen SUR1-/INP1-kanalen.
De bestaande document-, sleutel- en volumerechten blijven ongewijzigd. De nieuwe
grafische bibliotheek gebruikt begrensde vooraf gegenereerde letterbeelden en
eigen vectorpictogrammen, zonder runtime-fontparser of nieuwe privileged code.

De compositor beheert vensterposities, stapelvolgorde en invoerfocus. Een app kan
geen systeemrand overschrijven of invoer uit een ander venster opeisen. De
kernel levert de echte afzender van ieder bericht; een client-ID of coördinaat
verleent nooit rechten. Per venster bestaan een begrensde backing store, één
SUR1-ontvanger en een begrensde INP1-aanbod/ACK-wachtrij. Totaal maximaal
1.400.000 pixels backing store, twee vensters, geen heap of gedeeld appgeheugen.

## Herstel

De gebruikersruimtesupervisor beslist over herstel. Een compositorfout mag een
documentdraft of berekening niet vernietigen. De supervisor retireert eerst
uitsluitend de vier vastgelegde oude presentatiehandles per app (`0x53`, fase
`a2=1` van dit onuitgebrachte ABI1.11-profiel). Een app-eigen reply-ontvanger
blijft na peer-exit anders open en een lopende RPC kan zijn pauzeverzoek niet
lezen. Retireren sluit die oude handles zonder allocaties, nieuwe processen,
epochs of wijzigingen aan opslag, control of de oude procescapability. De apps
keren daardoor uit hun oude RPC terug, pauzeren presentatie en bevestigen dat
via hun bestaande controlroute. Alleen na beide echte pauzebevestigingen vraagt
init de constructiefase (`a2=0`) aan; die vereist voorafgaande retirement. De
supervisor kan dan de vastgelegde compositor/inputdienstgroep herstarten met verse procesidentiteiten,
capabilities en epochs. Apps binden die nieuwe kanalen expliciet en tekenen
opnieuw. Opslag- en documentkanalen worden daarbij niet vervangen. De oude
kanalen leveren geen succes meer. Herstel verruimt geen apparaatrechten.

De expliciete app-cleanup na retirement behandelt alleen `BadHandle` per
afzonderlijke oude handle als al gesloten. Andere sluitfouten blijven fouten,
beide eigenaars worden geprobeerd en er ontstaat geen invoer- of frame-ACK.
Normale `close()`-semantiek en de bestaande RPC-budgetten blijven ongewijzigd.

De init-policy bewaakt daarna afzonderlijk Wacht-op-Pause, Wacht-op-verse-READY
en Wacht-op-Rebind-ACK. Het oude gereedheidsmasker wordt bij herstelbegin gewist
tot de twee blijvende appbits. Alleen een geslaagde constructiesyscall plus
gevalideerde verse bootidentiteiten/generatie/epochs opent de READY-fase; die
begint opnieuw met uitsluitend appbits. Rebind vereist alle drie actuele
dienstmeldingen in die fase. ACK7 en ACK8 gelden uitsluitend in hun eigen fase;
oude READY-meldingen kunnen de herbindpoort niet openen.

De kernel bevat alleen de begrensde proces-/manifestmechaniek, geen rendering,
invoerfocuspolicy of fontparser. Partial initialization, onjuiste aanroeper,
levende oude dienst, stale procescapability, endpoint-/frametekort en foute
generaties moeten gesloten falen. Bestaande diagnostische profielen blijven
apart bruikbaar.

## Acceptatie

Eerst hosttests voor geometrie, bounds, focus, aanbod/ACK, malformed berichten
en herstelbindingscontrole; daarna no_std-build/ELF-inspectie. Native acceptatie
vereist werkelijk twee apps, toetsenbord/muisfocus, vensterbeweging, negatieve
surface-/callerproeven en een echte compositorfout met behoud van draft en
berekening. Vervolgens echt bewaren/lijsten/heropenen, onafhankelijke diskcontrole
en resourcecleanup. De fysieke GPU, echte accountauthenticatie, schermlezer,
volledige tekstshaping en menselijke gebruikerstests vallen buiten deze gate.

N01 wordt daarnaast in een afzonderlijk native profiel geïntegreerd; een
netwerkdienstcompile is geen native netwerkacceptatie. U01/N01 krijgen alleen
hun vaste punten als de volledige bestaande criteriumgrens aantoonbaar slaagt.
