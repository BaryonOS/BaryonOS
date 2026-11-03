# AUDIT-05 — productvolwassenheid en functionele probleemoplossing

**Bronmetadata:** primaire repositorybron: *Nieuw OS — “Add OS specialist
agent profiles”*, auteur/organisatie S.W.J.B.M. van Ieperen, 3 november 2026,
commit `beafdedfc188154fe7285848fbd411e651013f54`
([onveranderlijke snapshot](https://github.com/simonvanieperen/nieuw-os/tree/beafdedfc188154fe7285848fbd411e651013f54)).
Alle lokale `pad:regel`-citaten hieronder verwijzen naar die snapshot.
De referenties naar andere besturingssystemen zijn productdoelen en
toetsvragen, geen geverifieerde beschrijvingen van hun gedrag.

## Managementsamenvatting

Nieuw OS heeft een aantoonbare eigen technische basis en beperkte native
gebruiksketens, maar het beschikbare bewijs rechtvaardigt geen dagelijks
bruikbare productrelease. Het is meer dan een webdesktop: eigen loader/kernel,
ring-3-apps, capability-IPC, opslag, vensterherstel en netwerkdienst hebben
opgeslagen VM-deelbewijs. Het is evenmin een aantoonbaar alternatief voor het
volledige dagelijkse takenpakket van volwassen besturingssystemen.

De zwaarste producthiaten zijn niet cosmetisch: echte accountauthenticatie en
sleutelvrijgave, complete versleutelde gebruikersopslag met onafhankelijk
herstel, geauthenticeerd internet/NAS, toegankelijke dagelijkse apps en één
geïntegreerd, onderhoudbaar releasepakket. Een ondertekende updateproef en
herstel van een compositor zijn belangrijke deelresultaten, geen vervanging
voor die keten. Alle **62 unieke verplichte FR-rijen zijn nog open**, statisch
nageteld in deze audit. De bestaande **41/100 ontwikkelpunten** zijn eveneens
nagerekend, niet opnieuw toegekend; zij zijn geen gereedheidspercentage.

De juiste vergelijking met referentie-OS'en is daarom: kan een gebruiker een
taak zelfstandig afronden, fouten herkennen en data herstellen? Niet: lijkt
het scherm erop, gebruikt het dezelfde architectuur of heeft het veel tests?
Er is geen vergelijkende prestatietest, menselijke taakstudie, fysieke
hardwaretest of onafhankelijke menselijke audit door deze sessie uitgevoerd.

## 1. Opdracht, bron en bevoegdheidsgrens

- Letterlijke opdracht: “Beoordeel de productvolwassenheid van Nieuw OS en
  vergelijk functionele probleemoplossing met volwassen besturingssystemen.”
- Uitvoering: gewone chatsessie die het profiel
  `.github/agents/os-maturity-auditor.agent.md:10–27` als rolinstructie leest;
  **geen technisch geladen customagent** en geen nieuwe subagents.
- Datum: 3 november 2026, Europe/Amsterdam.
- Opgegeven bronbranch: `copilot/add-os-specialist-agents`.
- Bevroren, lokaal aanwezige en werkelijk gelezen bron:
  `beafdedfc188154fe7285848fbd411e651013f54`.
- Eigen beginbranch: `simonvanieperen-ubiquitous-barnacle`; begin-HEAD
  `69809c2cb53dfb3bd77abe5352892810fba6e82d`; werkboom schoon.
  Na de toegestane switch: **detached HEAD op beafdedf**, niet een checkout van
  de bronbranch. De opgegeven branchnaam is context, niet live remote-verificatie.
- Alleen de eigen worktree is als bestandsbron gebruikt; het lokale pad is
  niet gepubliceerd.
  Lokale worktree-metadata is gelezen; andere checkouts zijn niet onderzocht.
- Alleen dit rapport wordt geschreven. Geen productcode, scoreblad,
  projectstatus, roadmap, commit, push, PR of merge. De integrator verzamelt
  de zestien rapporten; dit rapport introduceert geen tweede actieve werkqueue.
- Instructies gelezen: root-AGENTS, `docs/AGENTS.md`, profiel en relevante
  boot/kernel/drivers/services/UI-instructies. Onder `docs` is alleen
  `docs/AGENTS.md` gevonden. De expliciete rapportopdracht begrenst de
  algemene instructies om status/score bij te werken en te publiceren.

Dit is een begrensde productmaturiteitsanalyse, geen exploitonderzoek,
volledige codeaudit, conformiteitsoordeel of algemene veiligheidsverklaring.
De voorbeelden hieronder zijn hiaten in implementatie of bewijs, niet
vastgestelde exploiteerbare kwetsbaarheden.

## 2. Bewijstaal en herleidbaarheid

| Niveau | Betekenis in deze audit |
|---|---|
| E0 | Bewijs ontbreekt of alleen ambitie voor de bedoelde productfunctie |
| E1 | Ontwerp, interfacecontract of acceptatie-eis |
| E2 | Hostprototype, hosttest of simulatie; geen native productbewijs |
| E3 | Native broncode; aanwezigheid is geen geslaagde uitvoering |
| E4 | Opgeslagen positieve uitvoering op het eigen OS |
| E5 | Opgeslagen negatieve uitvoering op het eigen OS |
| E6 | Opgeslagen herhaalde CI-/acceptatiegate, uitsluitend binnen beschreven bron en scope |
| E7 | Gedocumenteerde uitvoering op een benoemd fysiek doelapparaat |
| E8 | Onafhankelijke menselijke audit of formele evaluatie met afgebakende scope |

Niveaus worden niet gemiddeld. E5 voor een capabilityweigering bewijst geen
E5 voor wachtwoordherstel. Een `no_std`-parser zonder device-I/O is niet
automatisch E3 voor een werkende driver. Een machinegecontroleerd deelmodel
zou evenmin een onafhankelijke menselijke audit van het hele product zijn.

**Huidige bron:** zelf statisch gelezen bestanden op beafdedf. **Historisch:**
uitvoeringsrapporten die in die boom zijn opgenomen, met hun eigen commits,
vingerafdrukken en profielen. **Eigen controle:** uitsluitend Git-, tekst-,
JSON- en hashinspectie; geen nieuwe E4–E8-productuitvoering. E6 hieronder
betekent niet dat CI op beafdedf live is gecontroleerd.

### Bewijsregister

| ID | Concrete vindplaats in de bevroren boom | Bewijs en begrenzing |
|---|---|---|
| B1 | `kernel/src/main.rs:1–44,65–79`; `boot/loader/src/main.rs:507–525`; `crates/capability-core/src/lib.rs:1–44,53–73` | Eigen `no_std`-kernel, BootInfo-validatie, opt-in signed-boot en getypeerde rechten. Capaciteiten zoals 16 processen en 320-byte IPC zijn ontwikkelgrenzen, geen gemeten dagelijks productbudget. |
| B2 | `docs/verification/K05-ACCEPTANCE.md:3–15,21–42,68–85` | Historisch native start/exit/crash/herstart/deny/quota en cleanup; aanvullende bron `56b576b16da1e4503bc5f9167a476474a58c33cf`. Niet opnieuw op beafdedf uitgevoerd. |
| B3 | `docs/verification/mature-session-20261102/U01-ACCEPTANCE.md:3–26,37–56,58–70`; gelijknamige JSON `:3–14,29–57` | Bron `337f2f29f1833ef4a8d2cd640b09419f85dbb83b`: echte aparte apps, focus/slepen, compositorcrash, rebind, bewaren/heropenen, schijforakel; root- en CI-uitvoering. Eigen hashcontrole bevestigt alleen de opgeslagen full-resultbytes. |
| B4 | `docs/verification/S08-ACCEPTANCE.md:3–37`; `services/volume/src/lib.rs:1–11,23–38` | Historische 13 VM-invocaties, I/O-/flush-/timeout-/volle-opslaggevallen en koude read-only recovery, bron db1b62e. Volumechecksom is expliciet geen authenticatie/encryptie. |
| B5 | `docs/verification/B01-ACCEPTANCE.md:5–40`; `docs/implementation/B02-SECURE-RECORDS.md:3–24,242–264` | B01-native entropy en sleutelgrenzen op 53e0d57; B02 kent echte record-/persistentieslices, maar koude rollbackankering/write-ahead en volledige acceptatie blijven open. Niet als “geen crypto aanwezig” classificeren. |
| B6 | `docs/verification/B03-ACCEPTANCE.md:9–43,67–88,109–117`; `crates/update-core/src/policy.rs:19–63`; `crates/update-core/src/test_policy.rs:1–44`; `docs/verification/B03-KEY-POLICY-1.md:3–46,200–219,245–258` | Signed slots, onderbreking/kill en downgrade, aangevuld met aanwezige native TEST-policyrotatie naar vooraf vastgepind TEST-next en statische intrekking/geldigheidsvensters. Latere historische herhaling op `69d5c3acce8a97e38895f284f13d3de3dbf4a5d9`: 18/18 QEMU-stages, exit 0. Firmware→loadervertrouwen, productiekeycustody/-ceremonie, veilige beleidsdistributie/-antirollback en intrekking van een gecompromitteerde oude releasekey blijven open. |
| B7 | `docs/verification/n01-native-service-20261102/README.md:3–42,51–77`; `services/net-tls/src/lib.rs:1–9` | N01-bron f152aec: echte native TCP/UDP/HTTP met synthetische Ethernetpeer en appafwijzingen. TLS-crate noemt zichzelf uitsluitend portvoorwaarden; geen crypto-provider of transportdienst. |
| B8 | `services/nas/README.md:3–26,55–65,81–95`; `services/nas/native-client/src/lib.rs:1–5`; `docs/verification/nas-replay-110/README.md:3–31,33–47` | Echte hostencryptie/loopback-TLS, native lokale bouwstenen en historisch native tokenreplay na koude boot. Bronfingerprint 7255534f… en latere CI-binding afzonderlijk; geen volledige E2EE-multidevice-NAS. |
| B9 | `ui/native/src/session.rs:1–6,146–169`; `docs/DECISIONS.md:39–63,118–145` | Native sessie-interactie met standaard `UnavailableBroker`; echte authenticatie weigert. Openbare synthetische VM-gast is geen ingelogde gebruiker of vrijgegeven gebruikerssleutel. ADR-058 eist herkenbare bediening. |
| B10 | `drivers/usb/src/lib.rs:1–4`; `drivers/acpi-power/src/lib.rs:1–7`; `drivers/virtio-snd/src/lib.rs:1–8` | USB-framing, ACPI-extractie en geluidscodec noemen expliciet ontbrekende device-I/O/transport/playback. Bestandsnamen zijn geen bewijs van USB, slaapstand of hoorbaar geluid. |
| B11 | `docs/verification/A01-ACCEPTANCE.md:3–17,30–32`; `docs/verification/L02-ACCEPTANCE.md:3–19` | Native SDK-app met positieve/negatieve endpoints, bron 0d68762; native lokale qsim-jobdienst, bron 433bab9. Geen algemene installer, AI-inferentie, fysieke QPU of quantumversnelling. |
| B12 | `.github/workflows/native-window-session.yml:20–48,96–126`; `.github/workflows/native-foundation.yml:20–31,73–120` | Brongebonden CI-gates en host/native scheiding bestaan als configuratie. Geen uitvoering van deze workflowbestanden tijdens deze audit. |
| B13 | `docs/FULL-TEST-RELEASE.md:57–117,127–144,157–233`; `docs/PROGRESS.md:7–48`; `README.md:35–72` | Native/hardware/human/artifactgates, alle FR-rijen open; 41/100 betreft afzonderlijke ontwikkelrubric. VM-FR is gepland, geen bewezen totaalimage. |
| B14 | `docs/audit/os-completeness/README.md:19–34,70–77`; `source-baseline.md:3–17,61–80`; `completeness-matrix.md:15–49` in dezelfde map | Eerdere audit op 380111b; secundaire inventaris van 39 groepen. Niet als actuele branch-/PR-/CI-status overgenomen. |
| B15 | `docs/audit/os-completeness/accessibility-gaps.md:5–31`; `compatibility-gaps.md:3–27`; `matrix.json:723–751,803–830,1342–1368,1593–1625` in dezelfde map | Historische hiateninventaris voor a11y, migratie, browser/media, AI en governance; gebruikt naast huidige bron en nog open FR-eisen, niet als uitvoeringsbewijs. |
| B16 | `SECURITY.md:1–8`; `docs/PRODUCT-AND-SOVEREIGNTY.md:3–11,45–46,81–98`; `docs/DESIGN-AMBITION.md:5–22,26–42` | Expliciet geen ondersteunde release/securitycontact/responstermijn/productiekeys. Europese productrichting en taakonderzoek zijn ontwerp, geen conformiteit of menselijke validatie. |
| B17 | `docs/LOCAL-PRIVACY-CONTRACT.md:1–25,35–48`; `docs/implementation/ENCRYPTED-OBJECT-STORAGE.md:3–5,13–39,43–60`; `docs/SECURITY-AND-QUANTUM.md:19–59` | Normatieve trust boundaries: optionele opaque provideropslag, geen stille fallback, lokale AI, Security Floor. Eisen zijn geen bewijs van volledige native afdwinging. |

## 3. Maturiteitsmatrix

E4/E5/E6 zijn hier steeds **historisch opgeslagen**, tenzij anders vermeld.
Voor geen domein is E7/E8 op deze bevroren bron door deze audit vastgesteld.
Een Windows-host met QEMU/WHPX uit B3 is geen fysieke Nieuw-OS-hardwaretest.

| Domein / te lossen gebruikersprobleem | Huidige bron en niveau | Hoogste relevante deelbewijs | Open productgrens / gevolg |
|---|---|---|---|
| Kernel en isolatie — apps werken naast elkaar | E3, eigen kernel/capabilities (B1) | E4/E5, herhaalde gates historisch E6 (B2/B12) | Volledige productdiensten, SMP en gecombineerde druk/herstel zijn niet daarmee afgetekend. Fundamenteel integratiegat. |
| Boot en platformvertrouwen — bedoeld OS start | E3, opt-in signed boot en TEST-keypolicy (B1/B6) | E4/E5; historisch herhaalde update-/TEST-policygate (B6) | TEST-opvolging/statische intrekking bestaan; firmware→loadervertrouwen, productiekeybeheer/beleidsdistributie en finale artifactbinding blijven open. Fundamenteel. |
| Accounts/login/lock — juiste persoon krijgt data | E1 voor echte auth; E3 voor gast-/interactiepad (B9) | Geen E4/E5 voor echte accountauth vastgesteld | `UnavailableBroker` weigert. Een loginvenster bewijst geen account- of key-unlockketen. Fundamenteel. |
| Materiaalopslag — bewaren en later terugvinden | E3, volume/appketen (B3/B4) | E4/E5 en historische CI-herhaling (B3/B4) | Smalle data-/omvangprofielen; volledige CRUD/zoeken/import/export/duuracceptatie open. Productgat. |
| Versleutelde gebruikersdata — verlies/diefstal begrenzen | E1 plus E3 record-/keystateslices (B5) | E4/E5 voor begrensd sleutel-/persistentiewerk (B5) | Geen complete accountgebonden gebruikersopslag met koude rollback- en herstelketen. Fundamenteel. |
| Backup/herstel — eigen fout of apparaatverlies overleven | E1 totaalroute; E3 deelherstel (B4/B17) | E4/E5 koude read-only recovery van dezelfde image (B4) | Dezelfde image heropenen is geen onafhankelijke backup of herstel na verlies van die image. Fundamenteel/product. |
| Netwerk — toegestane apps communiceren | E3 dienst en NIC-rechten (B1/B7) | E4/E5 normale/exit/foutscenario's (B7) | Vaste bootstrapbestemmingen/synthetische peer; dynamische issuer/intrekking en algemeen internet open. Product/fundamenteel. |
| Internet/TLS — vertrouwde site of peer benaderen | E1 plus E3 portvoorwaarden, geen native TLS-dienst (B7) | E2 geauthenticeerde hostloopback (B8) | Certificate-time/identity, DNS/configuratie en onderhoudbare native TLS/browserketen open. Productgat. |
| NAS/multidevice/provider — werk hervatten op eigen apparaten | E1/E2 totaal; E3 lokale cores/client (B8/B17) | E4/E5 lokale replay na koude boot; historische CI-binding (B8) | Geen volledige pairing, persistente clientkeys, conflict/sync, native E2EE of provideradapter aangetoond. Productgat. |
| Drivers/platform — benodigde apparaten gebruiken | E3 voor begrensde virtuele paden; E1/E2 voor genoemde parsercores (B3/B7/B10) | E4/E5 voor specifieke virtuele devices (B3/B4/B7) | Geen E7-doelconfiguratie, brede USB/GPU/wifi/audio- of DMA/IOMMU-acceptatie. Fundamenteel/platform. |
| Energie — rust, slapen, hervatten en accugebruik | E1/E2 voor ACPI-extractie (B10) | GUI-idlemeting gerapporteerd in `README.md:92–96`, host/VM-context | Geen fysiek verbruik, slaap-/ontwaakroute of accuduur vastgesteld. Geen “energiezuinig OS”-claim. |
| Desktop/vensters — vertrouwd multitasken | E3 native compositor/sessie (B3/B9) | E4/E5/E6 voor beperkte U01-keten (B3) | Gastpreview, volledige instellingen, login/lock, brede interactiebudgetten en U02 blijven open. Productgat, niet louter styling. |
| Toegankelijkheid — zelfstandig bedienen met hulpmiddelen | E1; beperkte E2-prototypechecks (B15/B16) | Geen E4/E5 voor complete native hulptechnologieketen vastgesteld | Native semantische boom/events, screenreader, schaal en menselijke taken open. Productblokker, geen cosmetiek. |
| Taal/invoer — eigen taal en tekst correct verwerken | E1/E2; beperkte native tekstbouwstenen | U01 toont beperkte tekstinvoer, niet volledige i18n (B3/B15) | IME, RTL, shaping, fallback en locale-/hulpmiddelketen niet bewezen. Productgat. |
| Apps/SDK/rechten — gereedschap installeren en beheersen | E3 SDK en beperkte apps (B11); pakketontwerp aanwezig | E4/E5 en historische CI-acceptatie voor beperkte SDK-catalogusroute (B11) | Algemene installatie/update/verwijdering, portalen en dagelijkse appset niet geaccepteerd. Productgat. |
| Web/PDF/media/migratie — bestaand werk blijven doen | E1/E2 browser; afzonderlijke native decoderbouwstenen (B13/B15) | Native PNG/JPEG-deelproef historisch in `FULL-TEST-RELEASE.md:133` | Geen algemene browser-, PDF-, audio/video- of migratieclaim; parser of screenshot is geen complete app. Productgat. |
| Updates/recovery — na fout verder kunnen | E3 signed slot-/updatercode en TEST-keypolicy (B1/B6) | E4/E5 installatie/kill/rollback en TEST-rotatie/statische intrekking; historisch herhaalde gate (B6) | Productiekeycustody, veilige distributie/antirollback van nieuw beleid, app-/datamigratie, delivery en consumentenherstel blijven open. Fundamenteel/product. |
| Beheer/instellingen — begrijpen en veilig aanpassen | E1/E2 totaal, E3 beperkte shellbouwstenen (B9/B15) | Geen integrale native beheeracceptatie vastgesteld | Account-, netwerk-, update-, opslag- en rechtenbeheer ontbreken als afgetekende dagelijkse keten. Productgat. |
| Privacy — alleen gekozen gegevens delen | E1 contract; E2 hostpolicy; E3 primitives (B1/B8/B17) | E4/E5 voor specifieke deny/revokegrenzen (B2/B5/B7) | Geen volledige LP-regressie met echte apps/NAS/AI en uitgeschakelde uitleg op finaleimage. Geen algemene privacygarantie. |
| Documentatie/support/onderhoud — fouten melden en oplossen | E1 documentatie/voorlopig beleid (B13/B16) | Brongebonden bouw-/testdocumentatie bestaat; geen E6 voor complete releasevoering | Ondersteunde versies, eigenaar, contact, patchtermijn en response-oefening open. Productvoeringgat. |
| Integrale release — geleverd image klopt en blijft werken | E1 FR-register; E3 deeltooling (B12/B13) | E6 voor afzonderlijke historische gates, niet FR-R01–R04 | 62 open rijen; geen finale image met 100 starts/24 uur, hardware, mensen en schone reproductie. BLOKKEREND. |
| Europese marktvoorbereiding — bruikbaar en verantwoord aanbieden | E1 productrichting (B16/B17) | Geen E7/E8 of toepasselijkheidsevaluatie vastgesteld | Juridische toepasselijkheid, toegankelijkheid, licenties/SBOM, support en exporteerbaarheid moeten concreet worden beoordeeld. ONBEKEND, geen conformiteitsclaim. |
| Lokale AI — optioneel nuttige hulp zonder upload | E1 en E2 simulatie (B15/B17) | Geen echte native inferentie vastgesteld | Model/engine/resources/contextbroker/deny/cancel open; ontbrekende inferentie bewijst geen netwerkloze werkende AI. Productgat. |
| Quantum — begrensde lokale rekentaak | E3 eigen-OS qsim-port (B11) | E4/E5 en historische CI-acceptatie voor L02-scope (B11) | FR-L04-integratieregressie open; fysieke QPU/hybride hardware E0/E1 in dit bewijs. Geen versnellingsclaim. |

## 4. Vergelijking met referentie-OS'en

Onderstaande tabel gebruikt Windows, macOS, Linux, Android, iOS, ChromeOS,
Qubes OS en Fuchsia uitsluitend als **productdoelen en toetsvragen**, niet als
geverifieerde actuele featurelijsten of technische beschrijvingen. Geen externe
diensten zijn geraadpleegd; versies, edities, hardware en beleid zijn
`ONBEKEND`. De genoemde taken zeggen niet dat een bestaand systeem ze gelijk
of foutloos oplost.

| Referentiekader | Productdoel/toetsvraag (geen beschrijving van bestaand OS) | Nieuw OS: bewijs tegenover ontbrekende keten |
|---|---|---|
| Windows | Toetsvraag: kunnen gewone taken met bestaand werk, meerdere apps en herstel na app-/updatefout zonder gegevensverlies worden uitgevoerd? | U01/K05/B03 leveren deelbewijs; import/export, installer, dagelijkse apps, updates met gebruikersdata en support nog niet integraal bewezen (B2–B6/B13). |
| macOS | Toetsvraag: is setup/login/document-/apparaatgebruik herkenbaar en blijft na onderbreking duidelijk waar werk en rechten staan? | ADR-058 en Materiaal passen als ontwerp; U01 is echt. Auth, multidevice, volledige afwerking en menselijke continuïteitstaken zijn open (B3/B8/B9/B16). |
| Linux | Toetsvraag: kunnen gebruikers gereedschap kiezen, configureren en onderhouden met begrijpelijke diagnose, drivers en herstel? | Eigen kernel blijft verplicht; “Linux-achtig instelbaar” vereist niet Linux als basis. SDK/documentatie bestaan, apparaat-/pakket-/supportketen is beperkt (B1/B10/B11/B16). |
| Android | Toetsvraag: krijgen apps alleen benodigde rechten en hervatten ze veilig na onderbreking, resourcegebrek of netwerkverlies? | Capability- en quota/faultproeven zijn relevante basis; algemene package lifecycle, deviceportalen en dagelijkse achtergrondwerking zijn niet bewezen (B1/B2/B7/B13). |
| iOS | Toetsvraag: zijn maken, delen, terugvinden en veilig vergrendelen toegankelijk en met beperkte handelingen uitvoerbaar? | Begrensde tekst-/venstertaken bestaan; trusted lock, portals, native semantiek en menselijke taakkwaliteit ontbreken als totaalbewijs (B3/B9/B15). |
| ChromeOS | Toetsvraag: zijn webtaken, downloads en offline werk uitvoerbaar zonder handmatige transport-/updateafhandeling? | Geen native browseracceptatie; synthetische HTTP-peer is geen webplatform. TLS, engineport, downloads, offline/dataherstel en updatevoering zijn afhankelijkheden (B6/B7/B15). |
| Qubes OS | Toetsvraag: zijn contexten gescheiden en overdrachten expliciet en begrijpelijk, ook bij fouten? | Capabilityprocessen zijn geen bewijs van dezelfde compartiment-/VM-grenzen. Toets eigen document-, klembord-, download- en netwerkgrenzen met negatieve taken; kopieer niet de architectuur (B1–B3/B17). |
| Fuchsia | Toetsvraag: blijven rechten en servicelevenscyclus controleerbaar tijdens starten, falen, vervangen en opruimen? | Getypeerde handles, K05 en U01-herbinding zijn relevante eigen deelresultaten; ze erven geen Zircon/Fuchsia-bewijs of volledige platformvolwassenheid (B1–B3/B11). |

Een latere eerlijke vergelijking gebruikt dezelfde synthetische dataset,
concrete systeemversies, vergelijkbare hardware en gebruikerservaring.
Meet taakvoltooiing, fouten, herstel, hulpbehoefte, gegevensbehoud,
toestemmingsbegrip en latency per taak. Een nieuw eigen OS hoeft niet alle
legacy-apps te draaien, maar mag overstapproblemen niet als opgelost tellen
zonder een benoemde formaat-/app-/taakroute. Materiaal kan voordeel bieden,
maar daarvoor ontbreekt representatief menselijk vergelijkingsbewijs
(`docs/DESIGN-AMBITION.md:26–34`).

## 5. Geprioriteerde hiaten en afhankelijkheden

Prioriteit betreft uitvoeringsvolgorde, niet een nieuwe productscore.
**BLOKKEREND** verhindert finale aftekening; **KRITIEK** raakt
gegevensveiligheid/herstel of onderhoud; **HOOG** verhindert dagelijks gebruik;
**MIDDEL/LAAG** betreft vervolgens bruikbaarheid/afwerking. **ONBEKEND**
betekent dat impact of toepasselijkheid eerst afgebakend moet worden.
Ook lager geprioriteerde verplichte FR-eisen mogen niet worden weggelaten.

| ID | Ernst / type / prioriteit | Concreet hiaat, impact en bewijs | Afhankelijkheden en eerstvolgende bewijsgrens |
|---|---|---|---|
| M01 | BLOKKEREND / integratie / P0→P2 | Geen finale bron-/imagecombinatie met volledige producttaken; 62 open FR-rijen (B13). Losse profielen mogen niet worden opgeteld tot release. | Afhankelijk van M02–M10 en alle FR-rijen; integrator legt bron, configuratie, artifacts en toepasselijke gates op één combinatie vast. |
| M02 | KRITIEK / fundamenteel / P0 | Auth→key-unlock→accountdata→lock/revoke niet af; gast en `UnavailableBroker` zijn bewust geen auth (B9). | Identiteit/sessionbroker, B5, trusted input/compositor; positief login/unlock plus verkeerde/replayed/stale antwoorden, lock en koude reboot op eigen OS. |
| M03 | KRITIEK / fundamenteel / P0 | Volledige versleutelde gebruikersopslag, verse rollbackkennis na koude start en onafhankelijk herstel ontbreken (B4/B5/B17). Zelfde image recovery is geen backup. | M02, native crypto/key-status, opslagtransacties; offline byte-inspectie, restore naar nieuwe instantie, onderbreking, verlies/intrekking en zichtbaar foutgedrag. |
| M04 | KRITIEK / fundamenteel/product / P0 | Native TEST-policyrotatie naar vooraf vastgepinde opvolger en statische intrekking zijn aanwezig en historisch positief/negatief getest. Geen volledige productiekeycustody/-ceremonie, veilige beleidsdistributie/-antirollback of producttrust-/migratie-/recoveryketen bewezen (B6). | M03, bootvertrouwen en releasebeheer; bestaande TEST-gates behouden, daarna herstel na gecompromitteerde oude releasekey, veilig nieuw beleid, kritieke cuts en gebruikersdatabehoud. Geen productiekeys gebruiken in deze audit. |
| M05 | HOOG / product / P1 | Geen compleet internet/DNS/TLS/browser/downloadpad en native peerpolicy (B7/B15). NAS-onboarding/sync volgt daar niet automatisch uit (B8). | Entropy, identiteit/tijdmodel, native transport en rechtenissuer; daarna engineport, downloadportaal, E2EE-sync/provider en multidevice. Test deny/revoke/redirect/outage zonder stille fallback. |
| M06 | HOOG / product / P1 | Toegankelijkheid, i18n en herkenbare beheer-/dagelijkse appketen niet integraal bruikbaar (B9/B13/B15). | Native semantische API, tekst/font/input, sessie-/objectportalen; keyboard/screenreader/200%-taken, fout-/focusherstel en mensen met hulpmiddelen. |
| M07 | HOOG / platform / P1→P2 | Virtuele devices en codec/parsers bewijzen geen fysieke USB/audio/GPU/wifi, DMA-veiligheid of energiebeheer (B10/B13). | Eerst profiel en gerichte hardwareautorisatie; afzonderlijke E7 device-/IOMMU-/sleep-resume-/herstelproeven. Geen hardwareactie nu. |
| M08 | HOOG / product/ecosysteem / P1 | Appinstallatie/update/verwijdering, defaults, migratie en formaten niet compleet (B11/B13/B15). | M03/M04, package/authenticiteit zonder auto-grants, open/saveportalen; benoemde formaten en gewone externe app met fout-/onderbrekingsroute. |
| M09 | KRITIEK / productvoering / P0 ontwerp, P2 aftekening | Geen ondersteunde releases, securitycontact, responstermijn of productieondertekeningsbeheer (B16). Geen verantwoord dagelijks onderhoud aangetoond. | Benoemde maintainers, keys/channels, dependency-/licentie/SBOM-register en patch/recoveryprocedure; response- en reproduceerbaarheidsoefening vóór publieke release. |
| M10 | BLOKKEREND / bewijs / P2 | FR-R02–R04 vragen 100 starts, 24 uur gecombineerd gebruik, schone reproductie, fysieke en menselijke gates (B13). | M01–M09 functioneel gereed; vooraf grenzen/workload vastzetten, onafhankelijke controle en artifacts bewaren. Een tweede AI is geen E8-menselijke audit. |
| M11 | ONBEKEND / markt/toepasselijkheid / P2 | Europese ambitie is geen wettelijke conformiteit; doelgroep, distributie, licenties en toepasselijke eisen moeten beoordeeld (B16). | Concreet releaseprofiel en verantwoordelijke deskundigen; expliciete eisen-bewijsmapping, geen certificeringsclaim uit deze audit. |
| M12 | MIDDEL / productkwaliteit / P1→P2 | Interactiebudgetten en taakbegrip niet representatief gemeten; U01/WHPX- en idlemetingen hebben beperkte scope (B3/README:92–96). | Stabiele taken en resources; p50/p95/input-feedback, prompting en menselijke fout-/herstelmeting. Geen universele snelheid/energiewinst afleiden. |
| M13 | LAAG / cosmetisch / P3 | Consistente iconen, details en visuele verfijning blijven nuttig maar lossen M02–M10 niet op (B16). | Onderliggend taak-/a11ypad stabiel; beoordeel binnen dezelfde taak, niet alleen via screenshots. |
| M14 | MIDDEL / differentiatie / P3, geen releasevrijstelling | Materiaalvoordeel, echte lokale AI en hybride-QPU-route zijn niet gelijk aan bewezen consumentenwaarde; qsim-port bestaat wel (B11/B15). | AI-model/runtime/context-deny/cancel en taakonderzoek; behoud L02-regressie. Verplichte FR-L01–L04 blijven vóór finale release vereist, fysieke QPU blijft apart onderzoek. |

### P0–P3 als uitvoeringsadvies, niet nieuwe roadmap

1. **P0 — gegevensveiligheid en herstel:** behoud bestaande kernel-/rechten-
   en opslaggates; voltooi identiteit, key-unlock, versleutelde duurzaamheid,
   onafhankelijke backup en signed update-/trustherstel. Richt ondertussen
   onderhoudseigenaarschap en bewijsbinding in. Nog geen persoonlijke data.
2. **P1 — dagelijks functioneren:** verbind desktop, toegankelijkheid,
   instellingen, appportalen, gangbare taken/formaten, internet/TLS/browser
   en E2EE-NAS. Werk mag parallel waar interfaces/eigenaren gescheiden zijn;
   M05 mag de rechten-/sleutelgrenzen van M02/M03 niet omzeilen.
3. **P2 — productvoering en acceptatie:** voer de volledige FR-matrix uit op
   dezelfde finale combinatie, inclusief duur, fysieke doelprofielen,
   menselijke taak-/deskundigencontroles, releasebundle, licenties/SBOM,
   ondersteuning en toepasselijkheidsbeoordeling. Niet aftekenen op deel-CI.
4. **P3 — waarde en verfijning:** verbeter aantoonbaar taakcontinuïteit,
   productafwerking en onderscheidende lokale functies; behoud de verplichte
   AI-/qsim-releasegates. Extra QPU-/hardwareonderzoek staat afzonderlijk.
   P3 is geen toestemming om afgesproken functionaliteit uit release te schrappen.

## 6. Risicoregister en tegenstrijdig/ontbrekend bewijs

| Risico of inconsistentie | Vaststelling | Behandeling |
|---|---|---|
| Historische rode GUI-tekst lijkt huidige regressie | `FULL-TEST-RELEASE.md:129–133` noemt oude rode 200-secondenroutes; B3 bevat later geslaagde U01 op 337f2f29. | Niet “bewaren werkt niet” concluderen; behoud beide bronbindingen. Geen van beide bewijst finale FR-acceptatie op beafdedf. |
| Eerdere audit lijkt huidige bron te beschrijven | B14 noemt main/PR-status op 380111b; huidige checkpoint is documentcombinatie met latere bronnen (`PROJECT-STATE.md:3–46`). | Alleen lokale beafdedf-inhoud beoordelen; geen oude live-status naar nu promoveren. Geen remote-GitHubcontrole uitgevoerd. |
| Privacycontract/README blijven achter op deelimplementatie | `LOCAL-PRIVACY-CONTRACT.md:4` noemt native netwerk nog open; B7 bewijst beperkte native netwerkdienst. `services/nas/README.md:17–25` zegt nog geen QEMU-proef; B8 bevat lokale native replay. | Specificatie blijft normatief, maar “geen native code/proef” is te absoluut. Deelproef sluit internet/TLS/NAS niet. |
| Begrippen B02/FR-B02 lopen uiteen | `FULL-TEST-RELEASE.md:137–144` waarschuwt dat taaknamen niet gelijk zijn aan matrixrijen; B5 betreft keystatus, FR-B02 betreft imageauthenticiteit. | Eis altijd matrix-ID plus criterium/bron/scope; geen punt of rij sluiten op naamovereenkomst. |
| Checksum wordt cryptografische gegevensbescherming | Volumebron zegt expliciet dat FNV geen authenticatie/encryptie/rollbackbescherming is (B4). | M03 blijft open, ook wanneer opslagtests groen zijn. |
| VM-/hostsucces wordt hardware- of energieclaim | B3 scheidt TCG, CI en gerapporteerde WHPX-data; B10 bevat pure parsers. | Geen E7 uit host-CPUmetingen; fysieke profielen en energieproeven apart. |
| TEST-ankers worden productvertrouwen | B6 bevat native TEST-policyrotatie/statische intrekking, maar geen productiekeycustody, veilige beleidsdistributie/-antirollback of firmware→loaderacceptatie. De oudere B03-tekst alleen is te beperkt als actuele rotatie-inventaris. | TEST-deelbewijs erkennen zonder productieclaim; M04/M09 blijven productvoering blokkeren. |
| AI/qsim/veiligheidsbewijs worden vermengd | B11 bewijst software-qsim; B15 beschrijft gesimuleerde AI; geen menselijke audit vastgesteld. | AI-inferentie, postkwantumcrypto, simulator en fysieke QPU apart labelen; E8 niet uit AI-review afleiden. |
| Brede scope veroorzaakt onderhouds-/integratieoverbelasting | Browser, codecs, hardware, NAS, AI en eigen kernel vragen verschillende ketens (B13/B15/B16). | Onderhouden standaarden/componenten onderzoeken zonder kernelwissel; één eigenaar per gedeelde interface, afgebakende gates en geen stille scopeverlaging. |
| Juridische of maatschappelijke belofte wordt conformiteit | Europese producttekst noemt zichzelf ontwerp en geen certificering (B16). | Toepasselijkheid en deskundigenbewijs ontbreken; status ONBEKEND, niet “voldoet” of “voldoet niet”. |

## 7. Eigen controles, commando's en exitstatus

Alle onderstaande commando's zijn in deze sessie uitgevoerd. Er zijn geen
builds, tests, installaties, downloads, netwerkproeven of externe
statusopvragen gestart. Historische testcommando's in B2–B11 zijn **gelezen,
niet uitgevoerd**. Uitvoer staat in het sessietranscript; er is geen nieuw
logbestand of tweede rapport in de repository gemaakt.

| Controle | Exact commando | Exit / uitkomst |
|---|---|---|
| Ingang | `git branch --show-current && git rev-parse HEAD && git status --short && git cat-file -t beafdedfc188154fe7285848fbd411e651013f54` | 0; beginbranch/HEAD zoals §1, status leeg, type `commit`. |
| Toegestane bronwissel | `git switch --detach beafdedfc188154fe7285848fbd411e651013f54` | 0; detached op broncommit, status daarna leeg. |
| Bronhercontrole vóór schrijven | `git branch --show-current && git rev-parse HEAD && git status --short && git diff --quiet && git diff --cached --quiet && git log -1 --format='%H%n%P%n%s' && git ls-files 'AGENTS.md' 'docs/**/AGENTS.md' '.github/AGENTS.md' '.github/agents/AGENTS.md'` | 0; lege branchnaam, beafdedf, geen wijzigingen; parent 69809c2 en onderwerp `Add OS specialist agent profiles`. De globlijst vond root-AGENTS; aparte `find`/`git ls-files '*AGENTS.md'` leverden de geneste instructies. |
| Docs-instructies | `find docs -name AGENTS.md -print` | 0; uitsluitend `docs/AGENTS.md`. |
| Eigen metadata | `git worktree list --porcelain \| grep -A 4 -B 1 'simonvanieperen-ubiquitous-barnacle'` | 0; eigen pad, beafdedf, `detached`; geen andere checkoutinhoud gelezen. |
| FR-telling | Exact Pythonblok A hieronder | 0; 62 rijen, 62 unieke ID's, 62 open, regels 157–233. |
| Opgeslagen U01-hash | Exact Pythonblok B hieronder | 0; full-result-SHA klopt, historische `native_gate=true`, raw 33 en 13 framehashes. Geen native herhaling. |
| Roadmaptelling | Exact Pythonblok C hieronder | 0; 41 van 100 punten, 21 van 51 gates accepted. Geen bestand geschreven. |

**A — statische acceptatietelling**

```sh
python3 -B -c 'from pathlib import Path; import re; p=Path("docs/FULL-TEST-RELEASE.md"); rows=[(n,s) for n,s in enumerate(p.read_text().splitlines(),1) if re.match(r"\| FR-[A-Z][0-9]{2} ",s)]; print("FR rows:",len(rows)); print("open:",sum("| open |" in s for _,s in rows)); print("unique:",len({re.search(r"FR-[A-Z][0-9]{2}",s)[0] for _,s in rows})); print("range:",rows[0][0],rows[-1][0])'
```

**B — opgeslagen bewijsbytes, geen nieuwe VM-test**

```sh
python3 -B - <<'PY'
import json, hashlib, gzip
from pathlib import Path
root=Path('docs/verification/mature-session-20261102')
s=json.loads((root/'U01-ACCEPTANCE.json').read_text())
p=root/s['full_result']
actual=hashlib.sha256(p.read_bytes()).hexdigest()
print('U01 source:',s['source_commit'])
print('full_result hash:',actual,'matches:',actual==s['full_result_sha256'])
assert actual==s['full_result_sha256']
print('native_gate:',s['native_gate'],'raw_exit:',s['raw_qemu_exit'])
print('frame hashes:',len(s['frame_sha256']))
a=json.loads(Path('docs/progress.json').read_text())
print('progress top-level keys:',', '.join(a))
p=Path('docs/audit/os-completeness/matrix.json')
m=json.loads(p.read_text())
print('historical completeness basis:',m['audit_basis'])
print('historical groups:',len(m['items']))
for i in m['items']:
    if i['id'] in ('GOVERNANCE','AI-LOCAL','BROWSER','MEDIA','UPDATES-RECOVERY'):
        print(i['id'],i['status'],i['evidence_levels'])
PY
```

Hash: `8892530c0642475fec6c930ccb173ae168919c2391cc91a6baab0cb746f04d94`.
Dit controleert geen volledige bewijsbundel, bronvingerafdruk of bootimage;
`gzip` was geïmporteerd maar de serial is niet gedecomprimeerd/gevalideerd.

**C — bestaande rubric, zonder progress-tool te starten**

```sh
python3 -B - <<'PY'
import json
from pathlib import Path
p=json.loads(Path('docs/progress.json').read_text())
gates=[g for t in p['tracks'] for g in t['gates']]
print('accepted points:',sum(g['points'] for g in gates if g['status']=='accepted'))
print('total points:',sum(g['points'] for g in gates))
print('accepted gates:',sum(g['status']=='accepted' for g in gates),'of',len(gates))
PY
```

**Leescontroles:** `nl -ba`, `head`, `sed -n`, `rg`, `git ls-files` en
JSON-inspectie zijn gebruikt voor de hierboven geciteerde bronranges;
die leverden geen productuitvoering op. Enkele grote shelluitvoeren werden
door de tool afgekapt; beslissende ranges zijn daarna kleiner gelezen.
Een initiële instructiebatch eindigde met **exit 1**, uitsluitend doordat
`find docs/audits .github/agents -name AGENTS.md -print` de nog niet bestaande
map `docs/audits` aantrof. De bronwissel daarvoor was geslaagd en schoon.
Een zoekopdracht gebruikte ten onrechte
`docs/implementation/PRODUCT-AND-SOVEREIGNTY.md`: `rg` meldde bestandsfout
(exit 2 binnen een shellbatch die door latere commando's eindstatus 0 had).
Het juiste `docs/PRODUCT-AND-SOVEREIGNTY.md` is vervolgens gevonden en gelezen.
Een glob naar niet-bestaande sessiesubmappen had eveneens geen betekenis als
productcheck; het bestaande `ui/native/src/session.rs` is expliciet gelezen.
Deze zoekfouten zijn geen mislukte of geslaagde OS-tests.

**Rapportcontrole:** `git diff --check` gaf exit 0.
`git diff --no-index --check /dev/null docs/audits/subagents/05-maturity.md`
gaf exit 1 zonder whitespace-diagnostiek: het nieuwe bestand verschilt van
`/dev/null`. In een eerste `&&`-keten hield dit de daaropvolgende
Python-documentcontrole tegen; die was dus nog niet uitgevoerd. De aparte
documentcontrole las daarna 337 regels en 37 unieke letterlijke bronpaden.
Zij gaf exit 1 omdat het hierboven bewust als fout beschreven
`docs/implementation/PRODUCT-AND-SOVEREIGNTY.md` niet bestaat. Dat is geen
nieuwe ontbrekende bewijsbron: de juiste bron is al expliciet gelezen.
De definitieve controle hieronder behandelt precies dat ene gedocumenteerde
foutpad als verwachte ontbrekende verwijzing; andere ontbrekende paden blijven
een fout. Een eerste definitieve controle gaf intern exit 1 doordat zij alle
codefence-substrings telde, inclusief die in haar eigen geciteerde broncode.
De shellbatch eindigde alsnog met 0 door de volgende Git-opdracht; dat is geen
geslaagde Python-controle. Onderstaande versie telt alleen fence-regels.

```sh
python3 -B - <<'PY'
from pathlib import Path
import re
p=Path('docs/audits/subagents/05-maturity.md')
s=p.read_text()
assert s.startswith('# AUDIT-05')
assert sum(line.startswith("```") for line in s.splitlines()) % 2 == 0
refs=set()
for text in re.findall(r'`([^`\n]+)`',s):
    candidate=re.split(r':\d',text)[0]
    if candidate.startswith(('docs/','kernel/','boot/','services/','crates/','drivers/','ui/','.github/')) and not any(x in candidate for x in (' ', '*','…')):
        refs.add(candidate)
missing=sorted(x for x in refs if not Path(x).exists())
assert missing == ['docs/implementation/PRODUCT-AND-SOVEREIGNTY.md'], missing
assert not any(line.rstrip() != line for line in s.splitlines())
print('documentcontrole OK; uitsluitend het beschreven foutpad ontbreekt')
PY
```

Deze gecorrigeerde documentcontrole is afzonderlijk uitgevoerd: **exit 0**.

### Gerichte verduidelijking na AUDIT-07-signaal

Op 3 november 2026 is uitsluitend de te brede rotatie-afwezigheidsclaim
gecorrigeerd, zonder heraudit, codewijziging, nieuwe test of commit.
`git branch --show-current && git rev-parse HEAD && git status --short --untracked-files=all`
gaf exit 0: nog detached op beafdedf, alleen dit bestaande eigen rapport
ongevolgd. De aanvullende reads waren `nl -ba crates/update-core/src/policy.rs`
(regels 1–85), `nl -ba crates/update-core/src/test_policy.rs` (regels 1–120),
en `nl -ba docs/verification/B03-KEY-POLICY-1.md` (regels 1–100 en 180–285);
de leesbatches gaven exit 0. B6, de twee betrokken maturiteitsrijen, M04 en
de TEST-ankerrisicorij onderscheiden nu expliciet E3-bron en historische
E4/E5/herhaalde gate voor native **TEST-policyrotatie/statische intrekking**
van ontbrekend bewijs voor **productiekeycustody/-ceremonie,
beleidsdistributie/-antirollback en volledige producttrust**.
De 18-stagesrapportage op 69d5c3 is gelezen, niet opnieuw uitgevoerd of naar
beafdedf herlabeld. Broncommit, productconclusie, overige hiaten en score
blijven ongewijzigd.

## 8. Aannames, niet-uitgevoerd en overdracht

Opgeslagen rapporten zijn bruikbare brongebonden claims, maar niet allemaal
onafhankelijk herberekend. Alleen het genoemde U01-full-result heeft een eigen
hashcontrole gekregen. Oude GitHub-URLs/runstatussen zijn niet bezocht; er
is geen actuele PR-/remote-headclaim. De bevroren commit is lokaal aanwezig;
de opgegeven bronbranch is niet live tegen een remote gecontroleerd.

Niet uitgevoerd: preflight/progress write/check, Cargo/Python/Node-tests,
builds, QEMU, netwerkobservatie, volledige artifact-/fingerprintcontrole,
duur-/performance-/energieproeven, fysieke hardware, echte accounts/data,
menselijke gebruikerstests, onafhankelijke securityaudit/formele evaluatie,
juridische/licentieaudit en releasevalidatie. Die ontbreken niet “omdat alles
goed was”, maar omdat deze opdracht uitsluitend statische inspectie toestaat.

De bestaande meter is **41 → 41, ongewijzigd**; dit rapport kent geen punten
toe en wijzigt geen assessment. Geen nieuwe architectuurkeuze is aangenomen.
Resterend risico is vooral dat losse succesvolle bouwstenen als complete
gebruikservaring worden gepresenteerd of dat oud bewijs naar een nieuwe
integratie wordt doorgeschoven.

Opleverbestand: `docs/audits/subagents/05-maturity.md`; het oorspronkelijke
lokale worktreepad is niet gepubliceerd.
Integrator: behoud de bronbinding beafdedf en de historische scopes, verzamel
de overige rapporten en leg pas gezamenlijk vast. M01–M10 zijn geen
audituitvoeringsblockers, maar open product-/acceptatieblockers.
