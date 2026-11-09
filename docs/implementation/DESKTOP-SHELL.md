# Bureaublad, taakbalk en zoeken — ontwerp

Original design: 30 October 2026 (ADR-058). Status as of 9 November 2026:
**design** (§§1–5); **host-rendered implementation** (§7, `ui/graphics` and
`ui/native`, PR #214). There is no new native QEMU evidence for the Horizon
appearance and no measured usability study. The accepted progress remains
**41/100**; no additional criterion is accepted by this change.
Design source: [DECISIONS.md](../DECISIONS.md), ADR-058.
Inloggen en gebruikers: [PROFILES-AND-USERS.md](PROFILES-AND-USERS.md).

## 1. Doel

Wie Windows, macOS of een gangbare Linux-desktop kent, kan Nieuw OS meteen
gebruiken: apps starten, zoeken, vensters schikken, bestanden terugvinden,
instellingen wijzigen en afsluiten, zonder uitleg. Daarbovenop biedt Nieuw OS
dingen die bekende platforms niet standaard zo bieden (§4). Die aanvullingen
mogen het bekende pad nooit vervangen of in de weg zitten.

Regels:

- **Herkenbaar eerst.** Een standaardhandeling gebruikt de plek, het gebaar en
  de sneltoets die consumenten verwachten.
- **Eigen huisstijl.** Geen gekopieerde iconen, afbeeldingen, lettertypen,
  merknamen of trade dress. De bestaande Materiaal-tokens (minerale
  achtergrond, diepe groentinten, duidelijke typografie) zijn het startpunt.
- **Rechten blijven rechten.** Een vertrouwd ogend venster of een bekende
  plek verleent geen capability. Toestemmingsvensters tekent de vertrouwde
  shell in een laag die apps niet kunnen overtekenen. Wachtwoord, pincode
  en beheerderswachtwoord worden alleen ingevoerd op een oppervlak van de
  sessiedienst, bereikt via het vergrendelscherm of Ctrl+Alt+Delete; de
  shell en apps krijgen die toetsaanslagen nooit
  ([PROFILES-AND-USERS.md](PROFILES-AND-USERS.md) §4, invariant 8). Een app
  kan pixels nabootsen; daarom vraagt het systeem buiten die route nooit om
  een geheim.

## 2. Onderdelen en gekozen conventies

| Onderdeel | Gedrag | Keuze waar platforms verschillen |
|---|---|---|
| Bureaublad | Achtergrond; optioneel snelkoppelingen; rechtermuisklik: nieuw document, achtergrond wijzigen, beeldscherminstellingen | Standaard leeg bureaublad met alleen achtergrond |
| Orbit (dock) | Zenit-knop (eigen horizon-merkteken, geen "Start"), Signaal (zoeken), vastgemaakte en geopende apps met indicator, systeemvak, klok/datum, meldingen | Compacte zwevende capsule onderaan, gecentreerd; plek blijft herkenbaar |
| Zenit (launcher) | Zoekveld bovenaan (direct typen zoekt), vastgemaakte apps, alle apps, recente bestanden, gebruiker en aan/uit-menu | Opent met de OS-toets of het merkteken in Orbit |
| Zoeken | Apps, instellingen en eigen bestanden, lokaal geïndexeerd; resultaten direct bij typen | Geen internetzoekopdracht standaard; alleen als de gebruiker dat aanzet en een zoekdienst kiest |
| Vensters | Titelbalk met minimaliseren, maximaliseren en sluiten; slepen; vergroten via randen; dubbelklik op titelbalk maximaliseert | Knoppen rechts |
| Venster schikken | Slepen naar rand of hoek, of OS-toets+pijl, zet venster op helft of kwart | — |
| Wisselen | Alt+Tab met miniaturen; klik op taakbalkknop | — |
| Systeemvak | Netwerk, geluid, batterij, privacy-indicator (§4); klik opent snelle instellingen (wifi, volume, helderheid, niet storen) | — |
| Meldingen | Meldingencentrum vanaf klok/datum; niet storen | — |
| Bestanden | Bestanden-app met zijbalk (Recent, Bureaublad, Documenten, Afbeeldingen, Downloads, NAS/apparaten) en adresbalk; Materiaal levert geschiedenis, rechten en opslagstatus | Bekende mappen als vaste ingangen |
| Instellingen | Categorieën: Systeem, Apparaten, Netwerk en internet, Persoonlijke instellingen, Apps, Accounts, Tijd en taal, Toegankelijkheid, Privacy en beveiliging, Update | Zoekveld in Instellingen |
| Prullenbak | Verwijderen gaat eerst naar de prullenbak; leegmaken is definitief | — |
| Schermafbeelding | Print Screen en OS-toets+Shift+S | — |

### Standaard sneltoetsen

| Toets | Werking |
|---|---|
| Ctrl+C / Ctrl+X / Ctrl+V | Kopiëren / knippen / plakken |
| Ctrl+Z / Ctrl+Y | Ongedaan maken / opnieuw |
| Ctrl+S, Ctrl+A, Ctrl+F, Ctrl+P | Opslaan, alles selecteren, zoeken, afdrukken |
| Alt+Tab, Alt+F4 | Wisselen, venster sluiten |
| OS-toets | Zenit met Signaal |
| OS-toets+E / +I / +L / +D | Bestanden / Instellingen / vergrendelen / bureaublad tonen |
| Ctrl+Alt+Delete | Vertrouwd systeemscherm (vergrendelen, wisselen, afmelden, taakbeheer) |
| Ctrl+Shift+Esc | Taakbeheer |

Met "OS-toets" wordt de toets met het logo bedoeld, ook wel Super-toets
genoemd. Op Mac-toetsenborden geldt Cmd als OS-toets.

## 3. Huidige native basis en wat ontbreekt

| Er is (native, in QEMU getest) | Ontbreekt voor deze shell |
|---|---|
| Ring-3-compositor met één toegewezen surface voor één app, focus en tegelblits ([NATIVE-COMPOSITOR.md](NATIVE-COMPOSITOR.md)) | Meerdere clients en surfaces, vensterbeheer (verplaatsen, vergroten, stapelen, schikken), een beschermde systeemlaag, taakbalk en startmenu als vertrouwde shell |
| Native invoerdienst met PS/2-bron en focusroute via de compositor ([NATIVE-INPUT-DAEMON.md](NATIVE-INPUT-DAEMON.md)) | Zichtbare muisaanwijzer, interruptgestuurde invoer, sneltoetsafhandeling door de shell, beveiligde aandachtsreeks naar de sessiedienst |
| Materiaal en documentsessie ([NATIVE-MATERIAAL.md](NATIVE-MATERIAAL.md)) | Bestanden-app met mappenweergave, zoekindex |
| Meerdere ring-3-processen ([K05](K05-PROCESS-MANAGEMENT.md)) | Generiek apps starten/stoppen vanuit het startmenu (FR-K02, FR-A-rijen) |
| UEFI-framebuffer, vast 800×600 ([DISPLAY-INPUT-NATIVE-PROBE.md](DISPLAY-INPUT-NATIVE-PROBE.md)) | Hogere resoluties, schaalfactor, een grafische driver |

## 4. Beter dan bekend (aanvullingen)

Deze aanvullingen komen bovenop §2, nooit in plaats ervan:

- **Lokale assistent in zoeken:** vragen over zelf gekozen bestanden,
  volledig op het apparaat zonder netwerkrecht (FR-L01/L02).
- **Privacy-indicator:** het systeemvak toont welke app nu netwerk,
  microfoon, camera of locatie gebruikt; één klik toont of trekt het recht in.
- **Duidelijke toestemmingsvensters:** wie, wat, hoelang, in gewone taal,
  getekend door de vertrouwde shell.
- **Geschiedenis voor ieder bestand:** eerdere versies terugzetten vanuit de
  Bestanden-app (Materiaal-revisies).
- **Verder werken op een tweede eigen apparaat** via de eigen NAS.
- **Afwerking:** vloeiende, korte animaties die uit kunnen; geen wachten op
  zichtbaar opbouwende schermen.

## 5. Stappen

Elke stap is pas klaar met native QEMU-bewijs, inclusief foutgevallen. Een
browsermock of schermontwerp is productbewijs, geen systeembewijs.

| Stap | Resultaat | Afhankelijk van | Acceptatie |
|---|---|---|---|
| D0 | Bediening bruikbaar (= stap 0 van PROFILES-AND-USERS): aanwijzer, toetsenbord, complete frames, IRQ-invoer | Compositor, input | Zie daar |
| D1 | Schermontwerp in eigen huisstijl voor bureaublad, taakbalk, startmenu, vensters, vergrendel- en inlogscherm; klikbaar prototype, gelabeld als simulatie | ADR-058 | Taakproef met mensen die geen beheerder zijn: app starten, bestand terugvinden, venster schikken, afsluiten zonder uitleg |
| D2 | Meerdere clients en surfaces in de compositor; vensterbeheer: titelbalk, verplaatsen, vergroten, stapelen, schikken, Alt+Tab | D0 | Twee apps tegelijk; focus, overlap en sluiten; een app kan geen ander venster verplaatsen of overtekenen |
| D3 | Taakbalk, startmenu en aan/uit-menu als vertrouwde shell; apps starten vanuit het menu | D2, generiek procesbeheer | Starten, sluiten, crash van een app laat de shell staan; crash van de shell herstelt zonder sessieverlies |
| D4 | Zoeken over apps, instellingen en eigen bestanden, lokaal | D3, catalogus, PROFILES-AND-USERS stap 3 | Alleen bestanden van de ingelogde gebruiker; geen netwerkverkeer zonder expliciete keuze |
| D5 | Bestanden-app en Instellingen-app | D3, PROFILES-AND-USERS stap 2 en 6 | FR-U- en FR-S03-acceptatie |
| D6 | Hogere resolutie en schaal, daarna grafische driver | Driverwerk | Afzonderlijke contracten |

## 6. Bewijsgrens

Sections 1–5 describe the intended interaction design. Section 7 records a
host-rendered desktop implementation. Its fixtures and unit tests do not
demonstrate native Horizon execution, consumer usability, or feature parity
with another operating system. Familiarity requires measured user-task trials
(U06).

## 7. Uiterlijk 2030 — "Horizon" (5 november 2026)

Status: **host-gerenderd** via de bestaande desktopfixtures in `ui/native`
(768×536, 1280×720, 1920×1080); geen nieuw native QEMU-bewijs en geen gemeten
gebruiksproef.

Gebruikersbesluit (Simon, 5 november 2026): geen "Start"-knop of een icoon
dat aan een bestaand OS-logo doet denken, en geen vorm die aanvoelt als een
opgepimpte klassieke taakbalk. De **plek** van de bediening blijft herkenbaar
(onderaan, zoeken, apps, afmelden); vorm, naam en lichtbeeld zijn eigen.

Ontwerptaal "Horizon":

- **Licht als materiaal.** Achtergrond: een donkere hemel boven de oplichtende
  rand van een planeet, met atmosfeerlicht, een iris-waas en spaarzame
  sterren. De rand wordt met 1/16-pixelprecisie berekend, zodat hij ook op
  FHD-, QHD- en 4K-schermen strak blijft. Alleen gehele getallen, dus elke
  tegel rendert identiek.
- **Zwevend glas met lichtrand.** Oppervlakken zijn donker doorschijnend glas
  met een fijne rand en een highlight van boven (`rim`). Er zijn geen witte
  formulierkaarten.
- **Eén commandodock** in plaats van een taakbalk: een gecentreerde capsule
  met het eigen merkteken (een lichtende eclips boven een horizon, label
  "Zenit"), een compacte Signaal-knop in plaats van een zoekbalk, vastgemaakte apps als
  lichtbollen met een lichtstreep zolang de app open is, privacylicht en
  aan/uit.
- **Privacy als licht, niet als pop-up.** Een rustige mintgroene gloed
  "Lokaal" in het dock. Hij is geen bedieningselement en vraagt niets. Hij zegt
  alleen wat waar is in deze VM-sessie: je werkruimte staat op dit apparaat.
  Mint betekent overal "blijft bij jou". De volledige indicator uit §4 wacht op
  een capabilitybron.
- **Gecentreerde compositie** voor grote schermen: "Verder waar je was" als
  eerste blik, in plaats van een paneel linksboven.

Bewust nog niet gedaan:

- een klok of datum: de UI heeft geen tijdbron;
- nep-apps in het overzicht;
- animatie;
- schaal boven 2×: 8K heeft een 4×-letteratlas nodig, en de renderer kent nu
  drie lettergroottes bij 1× en 2×;
- grote displaytypografie: die vraagt een extra atlasgrootte.

### Namen (gebruikersbesluit 5 november 2026, familie "Kosmos")

Alleen de zichtbare namen veranderen; code-identifiers en documentnamen
(`Materiaal`, `nieuw_os_materiaal`) blijven staan om churn te voorkomen.

| Bekend als | Nieuw OS | Status |
|---|---|---|
| Bureaublad / startscherm | **Horizon** | Zichtbaar in de VM-shell |
| Taakbalk / dock | **Orbit** | Zichtbaar (compacte capsule) |
| Start / launcher | **Zenit** | Zichtbaar (merkteken, label) |
| Verkenner / bestanden (was Materiaal) | **Nebula** | Zichtbaar in shell en app |
| Zoeken | **Signaal** | Zichtbaar (knop en zoekpaneel) |
| Instellingen | **Kern** | Nog geen app |
| Store | **Haven** | Nog geen app |

### Richting: wat een OS van 2030 de consument moet geven

| Wat mensen nu missen | Antwoord in Nieuw OS | Bouwsteen |
|---|---|---|
| Privacy kost tientallen instellingen en cookie-achtige pop-ups | Privacy staat standaard aan; rechten worden één keer per **object en gevolg** gekozen bij installeren of koppelen, daarna stil; een permanente lichtstatus in het dock in plaats van meldingen | Capabilities, §4 privacy-indicator |
| Werk is verspreid over apps, cloud en mappen | Eén commandoveld voor apps, materiaal en acties, lokaal geïndexeerd; "Verder waar je was" als startpunt | D4, Materiaal |
| "Waar is mijn versie van gisteren?" | Geschiedenis per bestand, terugzetten vanuit Bestanden | Materiaal-revisies |
| Een tweede apparaat betekent een cloudaccount | Verder werken via de eigen NAS, end-to-end versleuteld | NAS-sync |
| AI-assistenten sturen data naar de cloud | Lokale assistent in het commandoveld, zonder netwerkrecht | FR-L01/L02 |
| Hoge-resolutieschermen zien er wazig of te klein uit | Vectorachtige, integer-exacte rendering; schaal 1×/2× nu, 3×/4× (8K) als volgende stap | Letteratlas, D6 |
| Updates en fouten voelen eng | Ongedaan maken van updates en duidelijke herstelroutes | Signed updates, herstel |

"Voelt futuristisch" en gebruiksgemak blijven hypotheses tot de taakproef
(U06).
