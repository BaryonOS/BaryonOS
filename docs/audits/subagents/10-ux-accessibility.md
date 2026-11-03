# AUDIT-10 — gebruikerservaring en toegankelijkheid

Datum: 3 november 2026, Europe/Amsterdam. **Resultaat:** er is aantoonbaar
historisch native deelbewijs voor vensters, documentbediening, compositorherstel
en tekstklembord. Een volledige persoonlijke, toegankelijke en herstelbare
gebruikersreis is daarmee niet aangetoond. Hieronder staan zowel concrete
bronafwijkingen als ontbrekende productpoorten; ontbrekend bewijs wordt niet
als een uitgevoerde mislukte test gepresenteerd.

## 1. Opdracht, bron en grenzen

Letterlijke opdracht: **"Beoordeel installatie, eerste start, login, desktop,
instellingen, foutmeldingen, herstel, toegankelijkheid en dagelijkse bruikbaarheid."**

- Taak-ID: AUDIT-10. Rol gelezen uit
  `.github/agents/os-ux-accessibility-reviewer.agent.md:15-31`; gewone
  chatsessie, geen technisch geladen customagent en geen nieuwe deelagents.
- Bevroren bronbranch: `copilot/add-os-specialist-agents`.
  Broncommit: `beafdedfc188154fe7285848fbd411e651013f54`.
- Eigen werkmap:
  `/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-musical-spoon`.
  Beginbranch: `simonvanieperen-musical-spoon`;
  begin-HEAD: `69809c2cb53dfb3bd77abe5352892810fba6e82d`.
  Beginstatus schoon. Daarna uitsluitend deze eigen worktree detached op
  de bevroren bron gezet; huidige bron is dus niet de beginbranch.
- Hoofdcheckout niet geopend of gewijzigd. `git worktree list` las alleen
  Git-metadata. Geen fetch, externe dienst, installatie, download,
  credentialgebruik, echte gegevens, apparaat of destructieve proef.
- Alleen dit rapport geschreven. Geen productcode, scoreblad, roadmap,
  projectstatus, commit, push, PR of merge. De hoofdagent integreert de
  rapporten; de gelezen score is 41/100 en is hier niet herbeoordeeld of gewijzigd.
- Toepasselijke root-, docs- en UI-instructies gelezen. De os-dev-skill is
  gebruikt voor bewijsdiscipline. Geen CodeRabbit/API-review uitgevoerd:
  die zou de expliciete lokale/read-only onderzoeksgrens overschrijden.

**Methode:** statisch lezen van relevante ontwerpen, UI-bron, invoerbeleid,
controller, bestaande testbron en brongebonden verificatierapporten.
Dit is een begrensde AI-beoordeling, geen volledige codebase-audit,
menselijke usabilitytest, conformiteitsbeoordeling of releasevalidatie.
Er is geen UI gestart, screenshot opnieuw opgenomen of historische run herhaald.
Regelverwijzingen gelden voor het bevroren commit, niet voor latere branches.

## 2. Bewijsladder en behouden native resultaten

| Niveau | Betekenis in deze audit |
|---|---|
| E0 | Ontbrekend of alleen ambitie |
| E1 | Ontwerp/documentatie |
| E2 | Hostprototype, hosttest of simulatie |
| E3 | Native broncode; op zichzelf geen uitvoering |
| E4 | Historisch native positief uitvoeringsbewijs, gebonden aan oorspronkelijke bron |
| E5 | Historisch native negatief uitvoeringsbewijs, eveneens brongebonden |
| E6 | Historische herhaalde CI-/releasegate voor benoemde scope; niet automatisch volledige release |
| E7 | Gedocumenteerde fysieke hardwaretest van het eigen OS |
| E8 | Onafhankelijke menselijke audit of formele evaluatie |

Een niveau is geen globale eigenschap van het OS. De actuele broninspectie
is eigen werk; E4–E6 hieronder zijn **gelezen historische resultaten**,
geen verse uitvoering op `beafdedf`.

| Bron | Feit, oorspronkelijke versie en grens |
|---|---|
| `docs/verification/mature-session-20261102/U01-ACCEPTANCE.md:1-31` en `docs/verification/mature-session-20261102/U01-ACCEPTANCE.json:1-27` | Bron `337f2f29f1833ef4a8d2cd640b09419f85dbb83b`, source-v3 `3919403542c923bc86467db3184345159a5ce6881567a0e11baa85f2d39f1995`: twee eigen CPL3-apps, focus/slepen, echte compositorfault en behoud van draft/berekening, daarna save/list/reopen en diskoracle. E4/E5; gedocumenteerde CI-herhaling E6 binnen U01. Geen persoonlijke account- of a11yacceptatie. |
| `docs/verification/native-clipboard-20261102/README.md:11-58,60-101` en `docs/verification/native-clipboard-20261102/root-native-verification.json:1-34` | Rootbron `d0ae570bd4666626096f92e57acef981c0f0d7b6`, finale testbron `e3182f0bba72f300596dd81417fbbd371a8d5d79`, CI-merge `5754763ffb257d6420f74126cae7252e36afab1e`: 22 native fasen, kopiëren/plakken tussen Materiaal en Rekenmachine, geweigerde ongeschikte tekst, wissen en leeg klembord na compositorherstel. E4/E5/E6 voor die scope. Eerste capaciteit 192 UTF-8-bytes; geen knippen of algemene formaatintegratie. |
| `docs/verification/session-desktop-20261102/README.md:3-25,70-85,124-131` | Bron `344dc0afde893784503217ac676e154b13662507`: acht native stappen met synthetische persoonlijke poging, weigering/wissen en expliciete openbare gast. E4/E5 voor previewgedrag, niet voor veilige persoonlijke aanmelding. Materiaalovergang in die TCG-run 16,08 s. |
| `docs/verification/gui-idle-20261102/INTEGRATION.md:13-49` | Historische Windows/WHPX- en CI-routes behouden herstel/save/reopen. ProcesCPU daalt in de beschreven A/B-proef; geen hard interactiebudget, fysieke energieproef of E7-gastdriverbewijs. |
| `ui/prototype/verification/P02V-2026-10-27.md:1-56` | E2: browserfocus, reflow en beperkte contrastcontrole. Laagste gemeten CSS-tekstpaar 4,73:1 in die proef. Werkelijke 200%-browserzoom, schermlezer en menselijke taken bleven open. Niet overzetten naar native compositorpixels of native toegankelijkheid. |
| `ui/native/README.md:1-40,83-109` | E3: app-lokale semantiek en native frontend, geen werkende OS-brede schermlezer. Accountpreview wordt expliciet onderscheiden van echte authenticatie. |

De U01-full-result-hash is in deze audit opnieuw berekend en komt overeen met
`docs/verification/mature-session-20261102/U01-ACCEPTANCE.json:10-11`. Dit controleert bytes van het opgeslagen verslag,
niet de waarheid van iedere runtimeassertie of een huidige image.

## 3. Taakpadmatrix

Frictie en risico hieronder zijn statische beoordelingen, niet gemeten
gebruikersgedrag. Verbetercriteria zijn voorstellen voor de bestaande gates,
geen nieuwe scorecriteria.

| Taakpad / doel | Stappen | Frictie | Fout en herstel | Toegankelijkheidsrisico | Bewijs / verbetering |
|---|---|---|---|---|---|
| Installeren met gegevensbehoud | Medium → taal/hulpmiddelen → invoertest → proberen/installeren → doel/plan → account/herstel → bevestigen → uitvoering → eerste boot | Geen uitvoerbare volledige installer aangetoond; consument kan de beschreven route niet als product doorlopen | Ontwerp invalideert gewijzigd doel/plan; stoppen vóór writes en faserecovery erna vereist | Hulpmiddelen moeten vóór schijfkeuze en geheimen bereikbaar zijn, niet pas op desktop | E1, `docs/product/new-os-installer-spec.md:14-75,92-104`; IN-01–07 en R01 open. Eerst volledige route op wegwerpimages inclusief onderbreking en begrip van wisgevolgen. |
| Eerste start en eigen profiel | Taal/regio → layout testen → netwerk overslaan/kiezen → account → herstelmiddel → desktop | Meerdere ontwerpen, nog geen bewezen persistente eigen profielroute | Onderbroken inrichting mag geen half account opleveren; first boot controleert feitelijke staat | Keuze van taal/keyboard/toegankelijkheid moet behouden blijven | E1, `docs/implementation/PROFILES-AND-USERS.md:101-122`; `docs/product/new-os-onboarding-spec.md:13-24`. FR-S01: onderbreken, terug/annuleren en koude start aantonen. |
| Aanmelden / lock / wisselen | Account kiezen → methode → bevestigen → sessieactivatie; later lock en opnieuw aanmelden | Alleen openbare VM-gast bruikbaar; persoonlijke verifier en veilige lock ontbreken | Persoonlijke poging wordt geweigerd; terug wist invoer. Geen echte herstel-/key-unlockroute | Niet-beschikbare methoden lijken bedienbaar; status niet in semantiek | E3 plus historische E4/E5 preview; `ui/native/src/desktop.rs:315-373`, `ui/native/src/session.rs:158-169`; FR-S02 pas sluiten na echte broker en negatieve lock-/wisselproeven. |
| App openen en lokaal vinden | Preview: gast → desktop → zoeken → Materiaal; vensterprofiel: dock of Ctrl+K-palet | Twee opt-in paden met verschillende zoek-/shellbediening; preview zoekt één vaste app, geen systeeminstellingen of bestandenindex | Geen resultaat blijft een lege uitkomst, geen alternatieve werkende service | Vaste ASCII-invoer, geen IME; palet en semantiek niet OS-breed verbonden | E3, `ui/native/src/desktop.rs:84-109,353-365`; `sdk/examples/window-session/src/scene.rs:240-299`. ON-03/U05: beschikbare doelen en ontbrekendstatus, alle benoemde aliassen en focus toetsen. |
| Vensters wisselen en schikken | Dockklik/F6 → focus; titelbalk slepen; Ctrl+K → app kiezen | F6 wijkt af van ontworpen Alt+Tab; geen bewezen volledige min/max/resize/snap/close-route | Echte compositorherstart behoudt apps; geen bewijs van iedere venster-/inputfout | Muisloze positionering en vensterbeheer niet aangetoond | E3 en historische E4/E5/E6 U01; `sdk/examples/window-session/src/main.rs:318-352`, `sdk/examples/window-session/src/scene.rs:375-462`. D2/FR-U01–02 met keyboard-only uitvoeren. |
| Tekst maken, opslaan en terugvinden | Nieuw → typen/selecteren → Ctrl+S of Bewaren → bevestigde commit → lijst → hetzelfde object openen | Native smalle keten bestaat; algemene bestandsordening/zoekindex en formaatdekking niet bewezen | Dirty-terug opent keuze; bewaren mislukt niet als succes; expliciet verwerpen | Appfocus/semantiek voorbereid, nog geen hoorbare status in hulpmiddel | E3 en historische E4/E5 voor hoofdketen; `ui/native/src/controller.rs:645-725`, U01-bewijs. FR-A01/A02 vereisen bredere echte workload, fouten en zelfstandig terugvinden. |
| Kopiëren/plakken tussen apps | Selecteer → Ctrl+C → andere app → Ctrl+V; Ctrl+Shift+Delete wist | 192-bytegrens en alleen tekst; knippen ontbreekt | Te lange selectie zichtbaar geweigerd, ongeschikte Calculator-invoer atomair geweigerd; herstel begint leeg | Feedback moet ook hoorbaar zijn; grens mag geen stil afkappen worden | Historische E4/E5/E6, klembordrapport `README.md:60-101`; U02 blijft open tot ruime afgesproken capaciteit en interactiebudgetten. |
| Instellingen wijzigen | Zoek categorie → wijzig → bevestig waar nodig → koude herstart → controleer | Uniforme native instellingen en persistente profielvoorkeuren niet bewezen | Vol/ongeldige waarde/I/O en onleesbare displayinstelling vereisen echte terugval | Geen bewezen systeemkeuze voor schaal, contrast, toetsenbord of motion | E1, `docs/implementation/DESKTOP-SHELL.md:35-48,99-113`; FR-S03/U05 open. Iedere instelling moet echte toestand tonen, fouten bewaren en herstart overleven. |
| Fout begrijpen en doorgaan | Probleem zien/horen → gevolgen begrijpen → ondersteunde herstelactie | Concrete labels voor conflict/closed/uncertain, maar generieke restfout zonder handelingsperspectief | Draft blijft in levend proces; onbekende commit niet replayen. Geen algemene rescue-/reconnectactie aangetoond | Fouttekst mag niet alleen in pixels staan; compact login mist brede toelichting | E3, `ui/native/src/render.rs:1106-1139,1500-1512`; `docs/implementation/UX-MATERIAAL-SAVE-RECOVERY.md:60-105`. FR-U03: gebruikers laten herstellen zonder terminal of blind retry. |
| Herstellen na crash of verlies | Compositorfault → verse kanalen → draft/berekening behouden; opslag-/sleutelverlies apart | Presentatieherstel bestaat, geen volledige gebruikersgerichte sleutel-/appcrash-/OS-recovery | Onzekere save bewaart alleen de levende draft; dat is geen herstel na app-exit of stroomverlies | Focus én status van hulpmiddel moeten na herstel terugkomen | Historische E4/E5 compositor; E1 voor bredere herstelroute. `docs/implementation/UX-MATERIAAL-SAVE-RECOVERY.md:60-88`; FR-O/FR-S04/R01 blijven apart. |
| Afmelden / herstarten met werk | Aan/uit-actie → bewaren/verwerpen/annuleren → flush → afsluiten | Vensterprofiel heeft F10-stopmechaniek, geen volwaardige zichtbare shutdownbegeleiding aangetoond | STOP weigert dirty werk: géén aangetoond stil weggooien. Weigering terug naar gebruiker nog onvoldoende onderbouwd | Verborgen functietoets en ontbreken van begrijpelijke melding beperken zelfstandigheid | E3, `sdk/examples/materiaal/src/main.rs:329-342`, `sdk/examples/window-session/src/init.rs:146-160`; FR-S04 vereist zichtbare keuze plus hang/flushfout. |
| Toegankelijk dagelijks werken | Vanaf boot hulpmiddel kiezen → keyboard-only/spraak/schaal → document/appwissel/fout/herstel | Geen bewezen end-to-end hulpmiddelroute, brede locale/IME of meerbeeldschermroute | Focus/status na modal, herstart en intrekking moeten beschikbaar blijven | Blind/slechtziend, motorisch beperkt of niet-US-keyboard kan taak niet zelfstandig voltooien | E1–E3 deelbouwstenen; U04/U06 en FR-U05 open. E7/E8 ontbreken binnen de beoordeelde scope. Native taakproeven met daadwerkelijke hulpmiddelen en mensen vereist. |

## 4. Bevindingen en prioriteit

**BLOKKEREND** betekent hier: blokkeert acceptatie van het genoemde taakpad,
niet dat verder onafhankelijk ontwikkelwerk moet stoppen. **KRITIEK** zou een
concreet aangetoond ernstig schadelijk pad vereisen; dat is in deze beperkte
read-only audit niet vastgesteld. Prioriteiten zijn geen kwetsbaarheidsscores.

### UX10-01 — BLOKKEREND: persoonlijk beginnen en veilig terugkeren ontbreken als complete reis

`ui/native/src/session.rs:158-169` weigert echte authenticatie standaard;
`ui/native/src/desktop.rs:315-373` biedt de expliciete VM-gast als bruikbare
route. `docs/implementation/SESSION-AND-DESKTOP.md:132-149,185-196`
onderscheidt appstaat van echte lock/revoke en noemt ontbrekend enrollment,
herstel en limitering. De installer is een voorstel
(`docs/product/new-os-installer-spec.md:1-12,78-90`).

**Impact:** er is geen bewezen pad van installatie naar eigen account, behouden
instellingen, veilige vergrendeling en zelfstandig herstel. De zichtbare
accountkaart is geen registratie. **Bewijs:** E1/E3, historische preview-E4/E5
uit §2; geen productiecredentials gebruikt. **Benodigd:** echte account-/sleutel-,
profiel-/instellingen- en sessiecomponenten, daarna FR-S01–04 en IN-acceptatie.
Een decoratief inlogscherm of expliciet veilige weigering sluit die poort niet.

### UX10-02 — BLOKKEREND: geen native end-to-end schermlezer/toegankelijkheidsketen bewezen

De frontend heeft echte lokale semantiek, acties en selectie, maar noemt zelf
dat dit geen accessibility-IPC of werkende schermlezer is
(`ui/native/README.md:35-40`). De releasematrix vereist juist native
tree/events, hulpmiddelen, keyboard, 200%, contrast en reduced motion
(`docs/FULL-TEST-RELEASE.md:168-172`); rubric U04/U06 staan open
(`docs/progress.json:294-315`).

**Impact:** visuele bediening en app-lokale testbomen bewijzen geen
zelfstandige taak voor een blinde gebruiker of bruikbaarheid vanaf boot.
**Benodigd:** benoemde broker-/event-/focusroute en daadwerkelijk hulpmiddel,
inclusief foutaankondiging, modal, appwisseling en herstel; daarna menselijke
taakproeven. Geen WCAG-/wettelijke conformiteitsclaim uit deze audit.

### UX10-03 — HOOG: vensterwisseling gebruikt F6 in plaats van het afgesproken Alt+Tab

Het ontwerp noemt Alt+Tab, Alt+F4, OS-toets en vensters schikken
(`docs/implementation/DESKTOP-SHELL.md:35-65`). De gelezen compositorroute
reserveert F6 voor `scene.cycle()` en stuurt andere toetsen naar de app
(`sdk/examples/window-session/src/main.rs:318-352`); het palet gebruikt Ctrl+K
(`sdk/examples/window-session/src/scene.rs:240-299`). Titelbalkinvoer verplaatst
het venster (`sdk/examples/window-session/src/scene.rs:404-462`), maar vormt geen aangetoonde volledige
resize/min/max/snap/close-interface.

**Impact:** overdracht van bekende sneltoetsen faalt in dit profiel;
muisloze vensterbediening is onvolledig aangetoond. Niet als regressie van
reeds geaccepteerd volledig vensterbeheer presenteren: U01 had een smallere
grens. **Benodigd:** één consequent shellcontract, gelabelde beschikbare
acties en native keyboard-only acceptatie; voorkom dat een app-toets F6
onbedoeld als globale wisseltoets wordt opgeëist.

### UX10-04 — HOOG: loginsemantiek meldt onbeschikbaarheid en fout niet mee

`ui/native/src/render.rs:701-753` publiceert in desktopoverlay alleen het
venster en controles, alle met `enabled: true`; er wordt vóór de gewone
semantiek teruggekeerd. `desktop.unavailable()` wordt daar niet als status
doorgegeven. Vingerafdruk en FIDO2 zetten alleen die vlag
(`ui/native/src/desktop.rs:349-351`), terwijl de renderer ze als actieve
knoppen tekent (`ui/native/src/render.rs:1452-1493`).
De algemene uitleg is bovendien afhankelijk van `!compact`
(`ui/native/src/render.rs:1496-1509`).

**Impact:** zelfs een toekomstige correct gekoppelde schermlezer zou uit deze
tree de geweigerde aanmelding en onbeschikbare methode niet kunnen afleiden.
Dit is een concrete E3-afwijking, los van het nog ontbrekende hulpmiddel.
Positief: wachtwoordbytes worden niet in de tree opgenomen.
**Benodigd:** expliciete beschikbaarheidsreden en benoemde status/event,
ook compact; geen werkende biometrie suggereren. Toets weigering, fout,
terugkeerfocus en hoorbare tekst zonder een geheim bekend te maken.

### UX10-05 — HOOG: onzeker opslaan heeft geen bewezen zelfstandige uitweg buiten de levende draft

De copy `"Opslag onzeker. Je tekst blijft open."` staat in
`ui/native/src/render.rs:1133`. De ontwerpspecificatie legt terecht vast
dat de draft alleen in het proces behouden blijft en dat nog geen veilige
statusopvraag/hervatting bestaat
(`docs/implementation/UX-MATERIAAL-SAVE-RECOVERY.md:60-88`).
Controller/testbron beschermt tegen blind opslaan en replay
(`ui/native/tests/interaction.rs:487-550`); die testbron is hier niet uitgevoerd.

**Impact:** de gebruiker weet onvoldoende wat te doen vóór appsluiting,
herstart of stroomverlies. Compositorherstel lost dit andere foutpad niet op.
Geen aantoonbaar dataverlies in deze audit; wel een open herstel- en
begrijpelijkheidsgrens. **Benodigd:** servicegedreven uitkomstcontrole of
afzonderlijk geautoriseerd draftbehoud/export, met expliciete gevolgen.
Geen algemene retry toevoegen aan een onbekende commit.

### UX10-06 — HOOG: dagelijkse responsiviteit nog niet op het vereiste budget bewezen

De klembordrapportage noemt TCG-opbouw te traag en de vereiste
minimaal twintig samples per stratum nog niet uitgevoerd
(`docs/verification/native-clipboard-20261102/README.md:91-101`).
Het native verslag bewaart bijvoorbeeld 77,27 s van laatste chord tot
geverifieerd editorbeeld, uitdrukkelijk als conservatief host/QMP/capture-
interval (`docs/verification/native-clipboard-20261102/root-native-verification.json:46-59`).

**Impact:** de beschreven profielervaring ondersteunt geen claim van vloeiende
dagelijkse bediening. Dit is geen exacte gastlatentie, p95, algemene
hardwaregrens of bewijs dat ieder frame even traag is. De CPU-idleverbetering
is reëel historisch deelbewijs, geen vervanging van interactiemeting.
**Benodigd:** brongebonden workload, tijdige zichtbare ontvangstfeedback,
metingen en foutregistratie volgens bestaande U02-gate; niet veiligheidschecks
verwijderen om een beter cijfer te krijgen.

### UX10-07 — MIDDEL: toetsenbordinvoergrenzen worden stil genegeerd in de preview

`ui/native/src/desktop.rs:425-443` negeert de `Result` van
`push_password_byte`; zoekinvoer boven de vaste buffer wordt niet verwerkt
en krijgt daar geen foutstatus. De flow zelf kent wel `PasswordFull`
(`ui/native/src/session.rs:278-287`); de querybuffer is 64 bytes
(`ui/native/src/desktop.rs:40-46,99-109`).

**Impact:** de gebruiker typt zonder te weten dat niet alles geaccepteerd is.
Vandaag levert dit geen geslaagde foutieve authenticatie op: de persoonlijke
broker weigert sowieso. Het is wel een fout in de voorbereide invoerervaring.
**Benodigd:** zichtbare en semantische capaciteitsmelding, behoud van geldige
invoer en herstel met Backspace. Test grens-1/grens/grens+1 en meerdere UTF-8-
bytes via de daarvoor bedoelde invoerroute, zonder inhoud te loggen.

### UX10-08 — HOOG: taal, keyboard en tekstweergave zijn smaller dan het productpad

Fysieke login-/zoekinvoer vertaalt via `us_character`
(`ui/native/src/desktop.rs:425-443`); query-invoer via setter accepteert alleen
ASCII (`ui/native/src/desktop.rs:99-109`). Tekstnavigatie werkt op codepoints, niet op
volledige zichtbare tekenclusters (`ui/native/src/text.rs:1-25`).
De native UI-atlas heeft vaste Latin-glyphs en vervangingsglyph
(`ui/graphics/src/text.rs:1-2,62-66`).

**Impact:** een gekozen locale/layout, samengestelde tekens, RTL of IME kan
niet uit UTF-8-opslagondersteuning worden afgeleid. Onleesbare vervanging
hoeft de opgeslagen bytes niet te wijzigen, maar verhindert betrouwbaar
begrijpen en bewerken. **Benodigd:** echt layout-/inputmethodemodel,
tekstshaping en afspraken over ondersteunde talen; taakproeven met langere
vertalingen en samengestelde tekens. Datum, tijdzone, getalformaten en
meerdere schermen zijn binnen deze audit ONBEKEND qua native productdekking.

### UX10-09 — MIDDEL: generieke foutcopy mist oorzaak en uitvoerbare herstelactie

De renderer heeft nuttige afzonderlijke labels voor conflict, read-only,
onzeker en verbroken opslag, maar laat overige fouten vallen op
`"Handeling mislukt. Je tekst blijft open."`
(`ui/native/src/render.rs:1106-1139`). Het gewenste foutencontract benoemt
juist full/quota/I/O, gevolgen en alleen werkelijk toegestane vervolgstappen
(`docs/implementation/UX-MATERIAAL-SAVE-RECOVERY.md:90-105`).

**Impact:** documentbehoud binnen het proces is positief, maar helpt een
consument niet zelfstandig verder bij ruimtegebrek of een dienstfout.
**Benodigd:** getypeerde oorzaken waar de dienst die levert, heldere grens
van behouden werk en een echte herstelactie. Geen verzonnen detailoorzaak,
providerfallback of success-shaped bevestiging.

### UX10-10 — MIDDEL: oude ontwerpstatus kan native vooruitgang onjuist laten lijken

`docs/implementation/DESKTOP-SHELL.md:3-6,71-82` zegt dat er nog geen shellcode
is en noemt één surface/800×600 als basis; `docs/implementation/PROFILES-AND-USERS.md:3-18`
zegt dat er geen inlogscherm of profielgerelateerde code is. Dat zijn
historische ontwerpbases op `c0b7147b`, terwijl de bron nu `desktop.rs`,
`session.rs` en de historische venster-/clipboardacceptatie bevat.

**Impact:** reviewers kunnen native vensters/Calculator/clipboard ten onrechte
tot hostprototype reduceren, of ontwerp en uitvoering juist verwarren.
**Benodigd:** later door eigenaar een expliciet historisch statuskader en
verwijzing naar actuele deelacceptaties toevoegen; oorspronkelijke bron en
ontwerpeisen bewaren. Dit rapport verandert die bestanden niet.

## 5. Overkoepelende toegankelijkheids- en verliescontrole

| Onderwerp | Vaststelling en open grens |
|---|---|
| Focus en keyboard | Positief: Tab/Shift+Tab/Enter en Esc; dirty-close begint op Verder bewerken (`ui/native/src/controller.rs:653-660,943-988`). Modal-/stale-actiontests bestaan (`ui/native/tests/interaction.rs:623-795`). Geen eigen runtimecontrole; globaal vensterbeheer blijft UX10-03. |
| Schermlezer | Semantische rollen, namen en acties zijn concrete bouwstenen, geen spraakoutput. Loginstatus mist zelfs in lokale tree (UX10-04). Focus-/statusherstel bij compositorfault moet ook door het toekomstige hulpmiddelpad lopen. |
| Contrast / informatie zonder kleur | Native status bevat tekst naast foutkleur (`ui/native/src/render.rs:1106-1146`). Kleurwaarden zijn beschikbaar (`ui/graphics/src/theme.rs:16-31`), maar geen eigen pixel-/contrastmeting uitgevoerd. Historische CSS-ratio's niet naar native antialiasing, focus, selectie of alle states overzetten. Geen vastgestelde native contrastafkeur of -goedkeuring. |
| Schaal / beeldschermen | `ui/native/src/lib.rs:20-31` heeft 100/200%; de app accepteert vanaf 320×240 logisch (`ui/native/src/lib.rs:49-55`). Venstersessie vereist minstens 800×500 logisch (`sdk/examples/window-session/src/scene.rs:61-68`): 1280×720 op 200% voldoet daar niet aan; 1920×1080 op 200% wel aan deze minimumgeometrie. Geen algemene schaalacceptatie, dynamische schaalkeuze, mixed-DPI of meerdere-schermenbewijs uit die code afleiden. |
| Beweging / alternatieve invoer | De frontend introduceert geen animatie (`ui/native/src/design.rs:76-81`); dat is niet hetzelfde als een persistente OS-brede reduced-motionvoorkeur. Touch, vergroter, schakelaarbediening en brede inputmethoden niet uitgevoerd of als werkend aangetoond. |
| Promptmoeheid | Productcontract vraagt één bewuste koppeling en daarna automatisch werken binnen bestaande scope (`docs/LOCAL-PRIVACY-CONTRACT.md:21-27`). Dat is E1. Browserflows zijn E2, geen bewijs van native promptfrequentie. Meet beslissingen/herhalingen per taak; nieuwe peer/rechten vragen wel een nieuwe beslissing. Privacyuitleg uit mag geen capability verruimen. |
| Gegevensverlies | Dirty-terug, expliciet verwerpen en STOP-weigering beschermen binnen de geleverde grenzen. Compositorcrash behoudt historisch draft en berekening. Niet gelijkstellen aan appcrash, reboot, veilige backup of sleutelherstel. NAS-beheerder kan zijn kopie verwijderen; dat gevolg en noodzaak eigen backup horen vóór koppelen begrijpelijk te zijn (`docs/LOCAL-PRIVACY-CONTRACT.md:61-81`). |
| Menselijke beoordeling | Geen deelnemers, hulpmiddelengebruikers, taakvoltooiing of begrip gemeten. Screenshots, rasteroracles en AI-beoordelingen leveren geen E8-usabilitybewijs of brede productkwaliteitsclaim. |

## 6. Eigen controles, commando's en niet-uitgevoerde werkzaamheden

Alle onderstaande shellaanroepen zijn daadwerkelijk uitgevoerd; exitstatus is
de status van de volledige getoonde aanroep. Bestandslezingen via `view`,
`glob` en `rg` zijn zoek-/leesacties, geen testresultaten. Eén gerichte `rg`
waarschuwde dat `sdk/examples/materiaal/src/app.rs` niet bestaat; vervolgens
zijn de bestaande `main.rs` en relevante bronverwijzingen gelezen.

```sh
# C01 — exit 0; schone eigen branch, commit aanwezig; tooluitvoer deels afgekapt
pwd && git branch --show-current && git rev-parse HEAD && git status --short && git worktree list && git cat-file -t beafdedfc188154fe7285848fbd411e651013f54

# C02 — exit 0; detached bron beafdedf, schoon; instructiepaden en remote metadata
git switch --detach beafdedfc188154fe7285848fbd411e651013f54 && git rev-parse HEAD && git status --short && git ls-files '*AGENTS.md' && git remote get-url origin

# C03 — exit 0; HEAD en aanwezige bronrefs, aantallen en exacte bewijsregels
git branch --show-current && git rev-parse HEAD && git status --short && git for-each-ref --format='%(refname:short) %(objectname)' --points-at HEAD && wc -l docs/product/new-os-installer-spec.md docs/product/new-os-onboarding-spec.md docs/verification/mature-session-20261102/U01-ACCEPTANCE.md docs/verification/native-clipboard-20261102/README.md docs/verification/session-desktop-20261102/README.md ui/native/src/desktop.rs ui/native/src/render.rs sdk/examples/window-session/src/main.rs && nl -ba ui/native/src/desktop.rs | sed -n '375,446p' && nl -ba ui/native/src/render.rs | sed -n '701,753p' && nl -ba sdk/examples/window-session/src/main.rs | sed -n '318,352p'

# C04 — exit 0; opgeslagen verslaghashes, geen productdiff vóór rapport
sha256sum docs/verification/mature-session-20261102/window-337-complete.json docs/verification/mature-session-20261102/U01-ACCEPTANCE.json docs/verification/native-clipboard-20261102/root-native-verification.json && git --no-pager diff --stat && git status --short
```

Bij de rapportcontrole gaf
`git diff --no-index --check /dev/null docs/audits/subagents/10-ux-accessibility.md`
exit **1**, zonder whitespace-uitvoer. De eerste ermee geketende leesbatch
stopte daarom vóór de vervolglezingen. Bij een tweede aanroep bleef de
no-index-status 1, terwijl de los volgende scope-/regelleesbatch exit 0 gaf
en uitsluitend dit ongetrackte rapport meldde. Dit is niet als groene
`git diff --check` geregistreerd. De onderstaande aparte read-only controle
controleert trailing whitespace en alle expliciete bestands-/regelranges.
Een eerste verwijzingsscan vond 19 onvolledige/te ruime verwijzingen; die
zijn hersteld. De definitieve uitvoering gaf exit **0**,
`references=68 invalid=[] trailing_whitespace=[]`:

```sh
python3 -B - <<'PY'
from pathlib import Path
import re
report = Path('docs/audits/subagents/10-ux-accessibility.md')
text = report.read_text()
references = re.findall(r'`([^`\n]+\.(?:md|rs|json)):(\d[\d,\-]*)`', text)
errors = []
for name, ranges in references:
    path = Path(name)
    if not path.is_file():
        errors.append((name, 'missing'))
        continue
    length = len(path.read_text().splitlines())
    for value in ranges.split(','):
        bounds = [int(n) for n in value.split('-')]
        if not 1 <= bounds[0] <= bounds[-1] <= length:
            errors.append((name, value, length))
whitespace = [n for n, line in enumerate(text.splitlines(), 1) if line.rstrip() != line]
print(f'references={len(references)} invalid={errors} trailing_whitespace={whitespace}')
raise SystemExit(bool(errors or whitespace))
PY
```

Dit is documentreferentiecontrole, geen OS-test. Aanvullende `nl`/`sed`/`wc`
waren uitsluitend leesnavigatie naar de geciteerde bronnen. Bij overdracht
worden `git rev-parse HEAD`, `git status --short --untracked-files=all` en
`sha256sum docs/audits/subagents/10-ux-accessibility.md` nog afzonderlijk
uitgevoerd; de definitieve hash gaat mee in de overdracht.

Gelezen bewijsbestanden, eigen SHA-256-controle:

| Bestand | SHA-256 |
|---|---|
| `docs/verification/mature-session-20261102/window-337-complete.json` | `8892530c0642475fec6c930ccb173ae168919c2391cc91a6baab0cb746f04d94` |
| `docs/verification/mature-session-20261102/U01-ACCEPTANCE.json` | `a5382ac427dd5330dfb8cef0d9d4ffb04d40a736f7345e107d7232aeb62b01a4` |
| `docs/verification/native-clipboard-20261102/root-native-verification.json` | `fb579e9b930396adac03ce642697b502038190537a1caafe67ef557dbea7f640` |

De grote eerste tooluitvoer werd door de tool opgeslagen onder
`/tmp/1793704742350-copilot-tool-output-1102-19dafc55-80ce-4f04-98bd-117fb24552ce.txt`;
dit is geen duurzaam repositorybewijs. Het rapport bevat de noodzakelijke
bronstatus zelf. Overige eigen controles hebben alleen toolstdout, geen nieuw
logarchief. Bestaande runtime-log-/artifactverwijzingen blijven historisch
in de in §2 genoemde rapporten; externe artifacts en Windowsbestanden zijn
niet opnieuw opgehaald of geverifieerd.

**Niet uitgevoerd:** builds, tests, preflight, progress write/check,
native/QEMU/UI-runs, installatie- of crashproeven, koude herstart, screenshot-
en pixel-/contrastmeting, werkelijke 200%-bediening, schermlezer, menselijke
taakproeven, hardwaretest, live PR-/CI-inspectie, formele/auteursrechtelijke/
wettelijke conformiteitscontrole, externe AI-review. Dit volgt de expliciete
auditgrens en is geen groen resultaat voor die poorten.

## 7. Aannames, afhankelijkheden en overdracht

- De bevroren repository is de enige primaire bewijsbasis. Een genoemd
  historisch rapport is gelezen als verslag van die eigen bron, niet als
  automatisch gereproduceerde waarneming op `beafdedf`.
- Native bronpaden zijn niet als één momenteel gecombineerd releaseimage
  beschouwd. Accountpreview en venstersessie zijn verschillende profielen.
  De historische ruimere native prestaties blijven wel behouden als deelbewijs.
- Concrete bronbevindingen UX10-03/04/07/08/09 vragen gerichte implementatie
  en positieve/negatieve acceptatie. Ontbrekende end-to-end poorten
  UX10-01/02/05/06 hangen af van echte diensten, hulpmiddelen en metingen;
  een extra ontwerp of hosttest sluit die poorten niet.
- Contrast, meerdere schermen, overige inputmethoden, tijdzone-/locale-
  persistentie en menselijke begrijpelijkheid blijven onbekend buiten de
  genoemde gelezen scope. Afwezigheid van eigen onderzoek is geen bewijs
  dat elke mogelijke implementatie ontbreekt.
- Prioriteit voor de integrator: persoonlijke sessiegrens en toegankelijke
  basisroute expliciet open houden, concrete bediening/statusfouten begrenzen,
  daarna dezelfde native taakreeks met fouten/herstel en menselijke observatie.
  Geen algemene veiligheids-, hardware-, conformiteits- of productrijpheidsclaim.
- Oplevering: uitsluitend
  `docs/audits/subagents/10-ux-accessibility.md`. Geen wijzigingen aan centrale
  voortgang; geen toekenning of intrekking van rubricpunten door deze deelreview.
  Bestandshash wordt na de laatste rapportcontrole afzonderlijk teruggegeven
  zodat geen zelfverwijzende hash in het bestand nodig is.
