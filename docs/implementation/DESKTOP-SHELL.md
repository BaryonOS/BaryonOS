# Bureaublad, taakbalk en zoeken — ontwerp

Datum: 30 oktober 2026. Status: **ontwerp** volgens gebruikersbesluit
ADR-058 in [DECISIONS.md](../DECISIONS.md). Er is nog geen code voor deze
shell en niets native getest. Bronbasis: main `c0b7147b`.
Voortgangsvoorstel: **34% → 34%**; dit document sluit geen criterium.
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
  plek verleent geen capability. Systeemvensters (toestemming,
  beheerderswachtwoord, Ctrl+Alt+Delete) tekent alleen de vertrouwde shell;
  apps kunnen ze niet nabootsen of overtekenen.

## 2. Onderdelen en gekozen conventies

| Onderdeel | Gedrag | Keuze waar platforms verschillen |
|---|---|---|
| Bureaublad | Achtergrond; optioneel snelkoppelingen; rechtermuisklik: nieuw document, achtergrond wijzigen, beeldscherminstellingen | Standaard leeg bureaublad met alleen achtergrond |
| Taakbalk | Startknop, zoekveld, vastgemaakte en geopende apps met indicator, systeemvak, klok/datum, meldingen | Onderaan, apps gecentreerd; links uitlijnen instelbaar |
| Startmenu | Zoekveld bovenaan (direct typen zoekt), vastgemaakte apps, alle apps, recente bestanden, gebruiker en aan/uit-menu | Opent met de OS-toets of de startknop |
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
| OS-toets | Startmenu met zoeken |
| OS-toets+E / +I / +L / +D | Bestanden / Instellingen / vergrendelen / bureaublad tonen |
| Ctrl+Alt+Delete | Vertrouwd systeemscherm (vergrendelen, wisselen, afmelden, taakbeheer) |
| Ctrl+Shift+Esc | Taakbeheer |

Met "OS-toets" bedoelen we de toets met het logo, ook wel Super-toets
genoemd. Op Mac-toetsenborden geldt Cmd als OS-toets.

## 3. Huidige native basis en wat ontbreekt

| Er is (native, in QEMU getest) | Ontbreekt voor deze shell |
|---|---|
| Ring-3-compositor met surfaces, focus en tegelblits ([NATIVE-COMPOSITOR.md](NATIVE-COMPOSITOR.md)) | Vensterbeheer (verplaatsen, vergroten, stapelen, schikken), taakbalk en startmenu als vertrouwde shell |
| Inputbron/-router met focus via de compositor ([NATIVE-INPUT.md](NATIVE-INPUT.md)) | Zichtbare muisaanwijzer, interruptgestuurde invoer, sneltoetsafhandeling door de shell |
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
| D2 | Vensterbeheer in de compositor: titelbalk, verplaatsen, vergroten, stapelen, schikken, Alt+Tab | D0 | Twee apps tegelijk; focus, overlap en sluiten; een app kan geen ander venster verplaatsen of overtekenen |
| D3 | Taakbalk, startmenu en aan/uit-menu als vertrouwde shell; apps starten vanuit het menu | D2, generiek procesbeheer | Starten, sluiten, crash van een app laat de shell staan; crash van de shell herstelt zonder sessieverlies |
| D4 | Zoeken over apps, instellingen en eigen bestanden, lokaal | D3, catalogus | Alleen bestanden van de ingelogde gebruiker; geen netwerkverkeer zonder expliciete keuze |
| D5 | Bestanden-app en Instellingen-app | D3, PROFILES-AND-USERS stap 2 en 6 | FR-U- en FR-S03-acceptatie |
| D6 | Hogere resolutie en schaal, daarna grafische driver | Driverwerk | Afzonderlijke contracten |

## 6. Bewijsgrens

Dit is een ontwerp. Het bewijst geen werkende shell, geen gebruiksgemak en
geen gelijkwaardigheid met bestaande platforms. Herkenbaarheid wordt pas
geclaimd na gemeten taakproeven (U06).
