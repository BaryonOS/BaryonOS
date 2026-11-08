# Nieuw OS — volledige testrelease

**Toestand: in uitvoering; volledige productacceptatie open.** Dit is de
bindende releaseopdracht en het acceptatieregister. Een documentatiecheckpoint,
een afzonderlijke mijlpaal en een groene VM-deelproef zijn geen oplevering.

## Doel, bron en voortgang

De vastgelegde `/goal` luidt:

> Realiseer de volledige native testrelease van Nieuw OS volgens docs/FULL-TEST-RELEASE.md. Blijf implementeren, integreren, testen en repareren tot alle verplichte functionele, visuele, beveiligings- en herstelcriteria aantoonbaar slagen op het geleverde releaseimage. Behoud de volledige productomvang en architectuur; vervang ontbrekende functies niet door mocks, verborgen bediening of uitstel. Gebruik beschikbare agents voor onafhankelijk werk. Bewaar bij een harde blokkade of limiet de exacte werkstand en hervatstap. Meld voltooiing uitsluitend met het complete testpakket en herleidbaar acceptatiebewijs.

De oorspronkelijke opdracht staat **onverkort** aan het einde van dit document.
De aanvullende instructie van 30 oktober 2026 is normatief verwerkt in
[uitvoerings- en productcontext](FULL-TEST-RELEASE-CONTEXT.md), inclusief concrete
gebruikstaken/compatibiliteitsgrenzen, formaatdekking, TCB en duuracceptatie.
Zij behoudt deze volledige opdracht, stabiele ID's, `/goal` en voortgangsrubric.
De aanvullende eis luidt: **“externe opslag van uitsluitend
versleutelde objecten moet kunnen”**. Daarom blijft een expliciet gekozen
externe provider toegestaan als onbetrouwbare opslaglaag voor uitsluitend
client-side versleutelde opaque objecten volgens
[privacycontract revisie 4](LOCAL-PRIVACY-CONTRACT.md) en
[het versleutelde-objectcontract](implementation/ENCRYPTED-OBJECT-STORAGE.md).
Geen inhoudssleutels, plaintext, verplichte leverancierscloud, stille fallback,
provider-key-escrow, OS-telemetrie of cloudinferentie. Deze verduidelijking
verandert de overige opdracht niet.

Er is één voortgangsketen; de bestanden hebben verschillende functies:

| Bron | Verantwoordelijkheid |
|---|---|
| Dit document | Bindende volledige opdracht, stabiele productacceptatie en aftekenregels |
| [progress.json](progress.json) en [PROGRESS.md](PROGRESS.md) | Bestaande vaste gewogen ontwikkelrubric; geen vervangende releasescore |
| [PROJECT-STATE.md](PROJECT-STATE.md) | Actuele bron, artifact, uitgevoerde tests, blokkades en eerstvolgende hervatstap |
| [WORK-QUEUE.md](WORK-QUEUE.md) | Uitvoerbare taken, actuele eigenaren, exclusieve schrijfgebieden, contracten en acceptatie-ID's |
| [ROADMAP.md](ROADMAP.md) | Bestaande technische/productroute en afzonderlijk vastgelegd onderzoek |

De hoofdagent onderhoudt deze koppelingen en de gedeelde voortgangsbestanden.
Een taak gebruikt de ID's hieronder en de bestaande rubric-ID's; zij introduceert
geen concurrerende roadmap. De opgeslagen beginscore is **32/100, rubric v1**.
Die score blijft staan zonder nieuw criteriumbewijs en is geen verklaring dat
32% van deze release al op één image is geaccepteerd. Historische native
resultaten blijven aan hun eigen bronversie gebonden. Alle productrijen hieronder
beginnen open; bestaand werk wordt hergebruikt en op de uiteindelijke combinatie
opnieuw beoordeeld.

### Gecontroleerde ontwikkelbasis

| Controle | Vaststelling en bewijsgrens |
|---|---|
| Ingangsbron | Branch `work`, commit `73ebc1e169b893eb5aba4b4f8ac06d7798136de3`; schone werkboom bij de repositorycontrole van 29 oktober 2026 |
| Rubriccontrole | `python3 tools/progress.py --check`: exit 0, 32/100, rubric v1 consistent; controleert berekening/verwijzingen/weergaven, geen OS-uitvoering |
| Historische K05-basis | Exact geteste bron `56b576b16da1e4503bc5f9167a476474a58c33cf`, vijf native gevallen; [K05-bewijs](verification/k05-startup-slots/README.md) |
| Actuele herhaling | De integrator rapporteert op bron `73ebc1e` vijf herhaalde K05-VM-runs en 238 root-Rust-hosttests, exit 0. Archief: `docs/verification/full-release-baseline/supervisor-gate.json` en `git-build-audit.json`; volledige raw hostlogs zijn niet opnieuw compleet gearchiveerd en een deel van de uitvoer is alleen tooltranscript. Geen productrij sluit hiermee; geen gecombineerd desktop-/opslag-/netwerkimage bewezen. De integrator legt de exacte huidige commandologs en artifactbinding vast. |
| Belangrijk open | Gecombineerd native profiel, B02 persistente geauthenticeerde opslag, volwaardige native UI/apps/netwerk/NAS/AI/updates en integrale releasevalidatie; fysieke/human-auditgates blijven op hun vereiste bewijsniveau open |

## Profielen, grenzen en bewijs

`VM-B0` is uitsluitend de bestaande smalle regressiebasis: x86_64,
`pc-q35-8.2`, `qemu64`, TCG, één CPU, 256 MiB, UEFI/OVMF 4M, eigen loader/kernel,
modern virtio-blk en virtio-rng waar de betreffende proef dat vereist. De
integrator registreert de werkelijk gebruikte QEMU-/OVMF-versies en hashes;
de historische basis gebruikte QEMU 10.0.13 en OVMF 2025.02-8+deb13u1.

`VM-FR` is het geplande geïntegreerde functionele releaseprofiel:
`pc-q35-8.2`, `Nehalem-v1`, één vCPU/TCG, 8 GiB RAM, een eigen sparse
32 GiB-disk, GOP 1280×720 plus lay-outcontrole op 1920×1080 en 200% schaal,
PS/2-invoer en virtio-blk/rng/net/snd. TPM 2.0 CRB gebruikt een aparte
swTPM-state. QEMU 10.0.13 en passende OVMF 4M zijn in de huidige
ontwikkelomgeving beschikbaar; dit is geen claim dat alle geplande devices
of componenten al native werken. `VM-AI` gebruikt dit profiel of een vooraf
vastgelegde passende uitbreiding met expliciet model-/resourcebudget; het
daadwerkelijke model en benodigde resources blijven een zichtbare gate.
De exacte configuratie en hashes worden vóór acceptatieruns bevroren.
Een profielnaam op zichzelf is geen ondersteuningsclaim. Bestaande kleine
proeflimieten zijn geen productlimieten. Het concrete ondersteunde
bestandsformaat-/capaciteitsregister en de prestatiegrenzen worden vóór de
bijbehorende implementatiegate vastgesteld; ontbrekende waarden blokkeren
aftekenen en mogen geen lagere productomvang verbergen.

`HW-FR` en `HW-AI` zijn afzonderlijke fysieke profielen. Hardware-identiteit,
onderdelen, ondersteunde functies, gecontroleerde image, instructies en
herstelroute moeten concreet zijn. Het fysieke apparaat is nog niet voor
schijfwrites, firmwarewijzigingen of aanpassing van bootconfiguratie
geautoriseerd. Dit blokkeert onafhankelijk VM-/bronwerk niet. Een bestaande
verplichte fysieke of menselijke gate blijft open; zij wordt geen `N/A`
omdat toegang, apparatuur of externe review ontbreekt.

Alle VM-disks zijn speciaal gemaakte wegwerpimages onder de aangewezen build-
en runpaden. Geen persoonlijke schijf, echte NAS-inhoud, account of fysiek
apparaat koppelen zonder gerichte autorisatie. Netwerk-/NAS-tegenpartijen zijn
afgebakende protocolconforme testservers met synthetische gegevens. Hosttools
mogen bouwen, observeren en emuleren; zij voeren geen ontbrekende OS-client,
crypto-, opslag-, UI- of inferentiefunctie namens Nieuw OS uit.

### Aftekenmodel

Elke afgetekende rij krijgt een compact rapport onder `docs/verification/`
met matrix-ID, exact broncommit en bronvingerafdruk, configuratie, toolversies,
exacte commando's, exitstatus, log-/artifactpaden en SHA-256 van geleverde
loader/kernel/images. Bewijs dat vóór een relevante wijziging is opgenomen,
wordt niet naar de nieuwe bron omgelabeld. Een mislukte run blijft mislukt
bewaard; een timeout telt alleen als geslaagd wanneer juist dat negatieve
scenario aantoonbaar wordt verwacht en gecontroleerd.

Bewijscodes in de matrix gelden cumulatief:

| Code | Vereist bewijs |
|---|---|
| N | Echte native uitvoering via eigen kernel/diensten op het geïntegreerde image; waarneembare handeling plus normale en negatieve assertions, bovenstaande bron-/hashbinding |
| D | Onafhankelijke inspectie van werkelijk opgeslagen bytes, koude nieuwe VM na volledige stop en waar relevant crash-/onderbrekingsproef; geen geheugensnapshot als duurzaamheid |
| V | Werkelijk gerenderde interface én bedieningsproef; screenshots/video ondersteunen de assertions en vervangen de bediening niet |
| T | Verkeer/ciphertext bij het afgesproken observatiepunt, identiteit/rechten en negatieve protocolgevallen; externe testserver is alleen tegenpartij |
| P | Metingen met vooraf vastgelegde workload, herhalingen, p50/p95 of toepasselijke verdeling, geheugen/CPU/latency/energiegrenzen en fout-/duurcriteria |
| H | Werkelijk uitgevoerd op het exacte fysieke profiel, met onderdelenlijst en herstelroute; VM-resultaten afzonderlijk |
| A | Afgebakende menselijke/gebruiker-/externe-deskundigencontrole waar bestaande acceptatie dat vraagt; AI-review afzonderlijk en geen vervanging |
| X | Compleet controleerbaar opleverartifact, reproduceerbare procedure en controle door een andere schone omgeving waar vereist |

Toestanden: `open`, `in uitvoering`, `geblokkeerd`, `afgetekend`.
`afgetekend` vereist al het genoemde bewijs. Een geweigerde toestemming of
ontbrekend ondersteund apparaat is alleen een normale runtimeconditie als de
betreffende implementatie werkelijk bestaat. Ontbrekende implementatie blijft
een open releasevereiste. Iedere hieronder genoemde eigenaar is een
verantwoordelijk werkspoor, geen claim van een al actieve agent. De actuele
persoon/agent en exclusieve bestanden staan alleen in de werkqueue.

## Actuele deelbewijzen behouden de volledige acceptatie

Alle 62 verplichte rijen blijven open. FR-B01/FR-O05/FR-N03 hebben native RANDOM-deelbewijs: vier echte gevallen op 2b5 en 9aa, opnieuw alle vier uitgevoerd in CI op `844ec40`. Firmware, ELF's, schijfbytes en canaries zijn afzonderlijk gecontroleerd. Dit levert geen persistente sleutels, volledige TLS of VM-FR-aftekening.

FR-U01/FR-A02 tonen in GUI-bron `5e126b7` echte Materiaal- en editorframes; de vaste 200-secondenketen faalt nog vóór Saved/Reopen/afsluiten. Een hostoptimalisatie op `1fd879d` blijkt native trager; de variantkeuze volgt daarom een echte eigen-kernelmeting. Geen veiligheidscontrole of acceptatiegrens wordt verwijderd.

FR-A03 heeft op `844ec40` een geslaagde native PNG/JPEG-decoderproef na een gericht herstel van de echte JPEG-stackguardfout. Stack, heap en pixeloracles zijn behouden; een onafhankelijke artifactcontrole slaagt. Dit tekent geen beeldviewer, volledige formaatdekking of native OOM/herstel af. Alle negen CI-workflows op die exacte bron slagen, inclusief vijf verplichte native foundationstappen. Op de latere gezamenlijke bron `9596abc` slagen opnieuw alle negen workflows met de vijf echte foundationstappen; de afzonderlijke GUI-keten blijft na 20.000 PIT-ticks rood met twee werkelijk gepresenteerde frames. [Gezamenlijke CI](verification/full-release-ci-9596abc/README.md) en [rode GUI-uitvoer](verification/full-release-gui-9596abc/README.md). [Decoderbewijs](verification/full-release-images-844ec40/README.md) en [CI-bewijs](verification/full-release-ci-844ec40/README.md).

De volledige opdracht, stabiele ID's, vereiste profielen en bewijsniveaus blijven ongewijzigd. Exacte actuele uitvoering staat bovenaan PROJECT-STATE en in WORK-QUEUE; een beperkt fixtureprofiel vervangt VM-FR niet.

## Acceptatiematrix

Werkpakketten gebruiken in de actuele queue het voorvoegsel `TASK-` en verwijzen
expliciet naar onderstaande acceptatie-ID's. Oudere ontwikkelcontracten en
testlogs noemen die werkpakketten soms `FR-I01`, `FR-A03` of `FR-B02`; dat zijn
geen aftekeningen van gelijknamige matrixrijen. Zo blijft `FR-A03` de
afbeeldingsviewer en `FR-B02` de boot-/imageauthenticiteit. Een documentadapter
of recordcodec kan die criteria niet sluiten.

ID's `FR-<domein><nummer>` blijven stabiel bij herplanning. Splitsen voegt nieuwe
ID's toe en behoudt de oorspronkelijke eis; geen rij verwijderen, verbergen
of als niet-verplicht markeren om groen te krijgen. Iedere verwijzing in de
laatste kolom is naar het bestaande register in [progress.json](progress.json).
`nieuw` bij een interface betekent dat de native contractuitwerking nog open
is, niet dat een dienst al bestaat.

### Kern, drivers, sessie en dagelijkse bediening

| ID / gebruikershandeling | Native componenten / interfaces | Normaal + foutscenario's | Profiel | Bewijs | Afhankelijkheden / blokkades | Eigenaar | Status | Rubric |
|---|---|---|---|---|---|---|---|---|
| FR-K01 — start het eigen OS | Eigen UEFI-loader, BootInfo, Rust no_std-kernel | Normale overdracht; ontbrekende/corrupte kernel, panic en begrensde boot-timeout | VM-FR; HW-FR | N,H | Geïntegreerde bootketen en finale imagebinding | kern | open | K01,B04 |
| FR-K02 — gebruik meerdere apps zonder elkaar te beschadigen | Paging/W^X, scheduler, ring 3, generiek proces-/dienstbeheer | Appstart/exit/herstart; appfault, parent-exit, orphan/cleanup, resource-uitputting; andere apps blijven werken | VM-FR; HW-FR | N,P,H | Profielen combineren; SMP voor vereiste meerkernondersteuning | kern | open | K02,K03,K05,K06,K07 |
| FR-K03 — verleen, weiger en trek taakrechten in | Capability-engine, Message320-IPC, usercopy, generaties/epochs | Geldige beperkte overdracht; stale handles, foute types/rechten/lengtes/pointers en toekomstige acties na intrekking geweigerd | VM-FR; HW-FR | N,H | Alle productdiensten aansluiten; geen rechtenverruiming | kern/beveiliging | open | K04,A03,B06 |
| FR-H01 — bedien met toetsenbord, muis en ondersteunde USB-apparaten | Inputdrivers, devicebroker, native events naar sessie | Toetsen/aanwijzer/hotplug; unplug, malformed deviceantwoord, uitval/herstart en begrensde buffers | VM-FR; HW-FR | N,V,P,H | Native inputketen, benoemde fysieke onderdelen | drivers | open | H02,U02 |
| FR-H02 — zie beeld en hoor geluid | Framebuffer/GPU-/audiodriver, geïsoleerde compositor-/audiobroker | Render/audio onder belasting; deviceverlies, ongeldige buffers, clipping/underrun, driver-/compositorherstel | VM-FR; HW-FR | N,V,P,H | Native graphics/audio, latencybudgetten en fysieke meting | drivers/sessie | open | H03,U01 |
| FR-H03 — gebruik afgesproken verbindingen en apparaten | Netwerkdriver, Ethernet/wifi en afgesproken Bluetooth | Verbinden/verbreken/hervatten; hotplug, driverfout en ontbrekend onderdeel met ware reden | VM-FR; HW-FR | N,T,P,H | Concrete hardwarematrix en driverondersteuning | drivers/netwerk | open | H04,H06 |
| FR-S01 — richt eerste gebruik en lokaal profiel in | Native onboarding, profiel-/sessiedienst, keyring, instellingenopslag | Taal/bediening/profiel/opslag kiezen en lokaal beginnen; terug/annuleer, fout en onderbroken inrichting zonder half profiel | VM-FR; HW-FR | N,D,V,H | Native UI, sleutel-/profielpersistentie; geen verplicht leverancieraccount | sessie | open | U01,U05,B01 |
| FR-S02 — vergrendel en ontgrendel de sessie | Sessiebroker, authenticatie, keyring-/surfacegrenzen | Herhaald lock/unlock; fout geheim, afgebroken poging, inputrace en toegang vanuit vergrendelde apps | VM-FR; HW-FR | N,V,H | Werkelijke authenticatie-/sleutelgrens, geen decoratief slotscherm | sessie/beveiliging | open | U01,U05,B02 |
| FR-S03 — wijzig instellingen en vind ze na herstart terug | Instellingendienst, gebruikersscope, volume/keyring | Wijzigen/terugzetten; ongeldige waarde, volle opslag, I/O-fout en koude herstart zonder valse bevestiging | VM-FR; HW-FR | N,D,V,H | FR-O01–O05, profiel-/apparaatinstellingen | sessie | open | U05,S08 |
| FR-S04 — herstart of sluit af met open werk | Sessiebroker, app lifecycle, opslagflush, ACPI/power | Opgeslagen werk behouden; onopgeslagen werk bewaren/verwerpen/annuleren, niet-reagerende app en flushfout begrijpelijk afhandelen | VM-FR; HW-FR | N,D,V,H | Native shutdown/restartpad en app-savecontract | sessie/drivers | open | H05,U05,R01 |
| FR-S05 — hervat en gebruik energiebeheer volgens profiel | ACPI-/energiebeleid, device-/sessieherstel | Ondersteunde slaap/wek, batterij/thermiek; onderbreking, timeout en herstel zonder verlies/rechtenherleving | HW-FR; VM-FR waar toepasselijk | N,D,P,H | Exact fysiek energieprofiel en autorisatie; geen VM-naar-hardwareclaim | drivers | open | H05,R02 |
| FR-U01 — navigeer Materiaal, Bewerken, Gereedschap, Toegang, Apparaten en Assistent | Native compositor/sessie, surfaces en begrensde app-/dienstendpoints | Alle afgesproken routes terug/vooruit; appwisseling, focusherstel, compositorcrash en ontbrekende inhoud | VM-FR; HW-FR | N,V,H | Alle schermen native porteren; browserprototype alleen ontwerpbron | gebruikersomgeving | open | U01,U02,U03,U05 |
| FR-U02 — selecteer, kopieer/plak, gebruik contextmenu en sleep | Focus-/selectiemodel, klembord-/dragbroker, taakgrants | Toetsenbord/muis, meervoudige selectie waar afgesproken; geannuleerd slepen, ongeldige bestemming en geweigerde cross-app-data | VM-FR; HW-FR | N,V,H | Native input, veilige gegevensoverdracht en productspecificatie | gebruikersomgeving | open | U02,A03,B06 |
| FR-U03 — begrijp voortgang, annuleren en fouten | Gedeeld native component-/statusmodel, asynchrone operaties | Werkelijke loading/leeg/succes; cancel, timeout, stale response, retry en herstel zonder nepstatus | VM-FR; HW-FR | N,V,H | Werkende serviceoperaties achter iedere bediening | gebruikersomgeving | open | U02,U05,U06 |
| FR-U04 — gebruik een afgewerkte consistente interface | Onderhoudbare design tokens, typografie/iconen/spacing/motion en appcomponenten | Alle toestanden; hover/focus/selection/disabled, lange tekst, venstergrootte, schalen; geen onafgewerkt scherm of placeholder voor vereisten | VM-FR; HW-FR | N,V,A,H | Native renderketen; alle apps visueel beoordelen en bedienen | ontwerp/UI | open | U06,A04 |
| FR-U05 — bedien toegankelijk | Native accessibility-tree/-events, tekst/focus, schaal/zoom, contrast/reduced-motion | Keyboard-only, schermlezer, leesbaarheid, 200%/afgesproken schalen; focusval/afgekapte labels/statusverlies voorkomen, reducedmotion en informatiezonderkleur | VM-FR; HW-FR | N,V,A,H | Native hulpmiddelen en echte gebruikerstaken; web-WCAG-test geen native bewijs | ontwerp/UI | open | U04,U06 |

### Basisprogramma's en appbeheer

| ID / gebruikershandeling | Native componenten / interfaces | Normaal + foutscenario's | Profiel | Bewijs | Afhankelijkheden / blokkades | Eigenaar | Status | Rubric |
|---|---|---|---|---|---|---|---|---|
| FR-A01 — maak, zoek en orden materiaal/bestanden | Native Materiaal-app, catalog-/volumeclient, zoek-/metadatadienst | Maken/openen/zoeken/ordenen/hernoemen/verplaatsen; geen resultaat, naamconflict, ingetrokken recht, stale revisie en volle opslag | VM-FR; HW-FR | N,D,V,H | FR-O01–O07, native zoekindex en bruikbare capaciteit | apps/opslag | open | U03,U05,A04,S06 |
| FR-A02 — schrijf en bewerk tekst | Native teksteditor, document-session, opslag-/klembordportaal | Nieuw/open/bewerk/selecteer/undo/opslaan/weer openen; grote UTF-8-invoer, extern gewijzigde revisie, crash, onopgeslagen sluiting en I/O-fout | VM-FR; HW-FR | N,D,V,H | Editor en concrete formaat-/omvangmatrix; geen voorgevuld scherm | apps | open | A04,U03,S05 |
| FR-A03 — bekijk afbeeldingen | Native image-app, begrensde decoder, object-/renderbroker | Open ondersteunde echte afbeeldingen, zoom/pan/schalen; beschadigd, te groot, unsupported formaat en geweigerd object | VM-FR; HW-FR | N,V,P,H | Vastgezette formaten/decoders, dependency-/licentiecontrole | apps | open | A04,H03 |
| FR-A04 — lees PDF | Native PDF-app/onderhouden engine, object-/renderbroker | Echte meerdere pagina's, navigatie/zoom/zoeken waar afgesproken; corrupt/beschermd/te groot document en parser/resourcegrens | VM-FR; HW-FR | N,V,P,H | PDF-engineport, feature-/formaatmatrix en sandbox | apps | open | A04,U03 |
| FR-A05 — browse en download binnen verleende rechten | Native browser/onderhouden engine, netwerk/TLS/DNS, downloadportaal | Navigeren, link/formulier, eigen testsite en downloaden/openen; DNS/time-out/TLS-fout, geweigerde bestemming/bestandsactie en intrekking | VM-FR; HW-FR | N,D,V,T,H | Chromium/Gecko-vergelijking vóór enginebesluit, engineport, FR-N01–N04, app-/objectrechten | apps/netwerk | open | A04,N01,N02,N03,N04,A03 |
| FR-A06 — speel audio af | Native player, codec, audiobroker en objectclient | Echte gangbare afgesproken bestanden afspelen/pauze/seek/volume; corrupt/unsupported, einde, deviceverlies en resourcegrens | VM-FR; HW-FR | N,V,P,H | Audio-/codecport en concrete formaten; gemeten hoorbare uitvoer | apps/drivers | open | A04,H03 |
| FR-A07 — speel video af | Native player, demux/codec, render-/audiobroker | Echte afgesproken video met beeld/geluidsync, pauze/seek; corrupt/unsupported, dropped frames en CPU/geheugendruk | VM-FR; HW-FR | N,V,P,H | Video-/audio-/renderketen en concrete codecprofielen | apps/drivers | open | A04,H03,R02 |
| FR-A08 — reken met de rekenmachine | Native app, begrensde parser/rekenkern | Werkelijke invoer, bewerken en resultaten; delen door nul, grote waarden, ongeldige expressie en herstel van invoer | VM-FR; HW-FR | N,V,H | Native app en gedocumenteerde reken-/precisiegrenzen | apps | open | A04 |
| FR-A09 — maak en bewaar een screenshot | Capturebroker, compositor, object-/saveportaal | Scherm/afgesproken selectie vastleggen, opslaan en heropenen; cancel, geweigerde capture, volle opslag en lockscreenbescherming | VM-FR; HW-FR | N,D,V,H | Screenshot is gekozen toegestane minimale capturefunctie; native capturegrens | apps/UI | open | A04,A03,U01 |
| FR-A10 — bekijk systeeminformatie, diagnose en Nederlandse hulp | Native diagnose-/hulpapp, gescopete status/log-API | Echte configuratie/versie/resourcegegevens, foutuitleg/logexport; ontbrekend device, redacties, geweigerde privileged data en offline hulp | VM-FR; HW-FR | N,V,X,H | Werkelijke meetbronnen, geen hardcoded systeemstatus of secrets in logs | apps/release | open | A04,U05,R03 |
| FR-A11 — installeer een app uit afgesproken bronnen | Pakketdienst, manifest/handtekening, appimage-loader, capabilitybeleid | Geldig pakket installeren en echt uitvoeren; ongeldige signatuur/hash/manifest, disk vol, onderbreking en geweigerde installatiebron | VM-FR; HW-FR | N,D,V,T,H | Pakketformaat, meerdere bronnen, geen onbeperkte installscripts of automatische grants | appbeheer/beveiliging | open | A01,A02,B04,B06 |
| FR-A12 — werk apps bij en verwijder ze | Pakkettransactie-/versiebeheer, proces-/objectrechten | Bijwerken/verwijderen/weer openen waar passend; draaiende app, onderbreking, ongeldig pakket, behoud-/wisgedrag en ingetrokken oude autoriteit | VM-FR; HW-FR | N,D,V,H | FR-A11, opslagtransacties en expliciet gebruikersdatacontract | appbeheer | open | A02,A03 |
| FR-A13 — kies standaardgereedschap en open/bewaar samenhangend | MIME/formaatregister, open/saveportaal, SDK, taakbroker | Bestand naar passende app, default bewaren en beperkte grants; onbekend formaat, verwijderde default, deny/revoke en kwaadaardige helper | VM-FR; HW-FR | N,D,V,H | Concrete formaten en portalen; ondertekening is herkomst, geen toestemming | appbeheer/UI | open | A01,A03,A04,B06 |
| FR-A14 — maak en bewerk tabelmateriaal | Native tabelgereedschap volgens P3/P4, document-/objectmodel en gedeelde UI-/savebroker | Realistische tabel maken/openen/bewerken/bewaren/heropenen; ongeldige invoer, grote tabel, conflict en I/O-fout; afgesproken import/export per feature toetsen | VM-FR; HW-FR | N,D,V,P,H | Bestaand PRODUCT-SPEC-productiviteitsmodel; concrete tabel-/formaatfeatures blijven zichtbaar open tot vastlegging/bewijs | apps | open | A04,U03,S05,U06 |
| FR-A15 — maak en bewerk presentatiemateriaal | Native presentatiegereedschap volgens P3/P4, object-/render-/savebroker | Realistische presentatie maken/openen/bewerken/bewaren/weergeven; beschadigde inhoud, schalen/lange tekst, conflict en I/O-fout; afgesproken import/export per feature toetsen | VM-FR; HW-FR | N,D,V,P,H | Bestaand PRODUCT-SPEC-productiviteitsmodel; geen ongefundeerde PPTX-round-tripclaim | apps | open | A04,U03,S05,U06 |

### Opslag, netwerk, NAS en lokale AI

| ID / gebruikershandeling | Native componenten / interfaces | Normaal + foutscenario's | Profiel | Bewijs | Afhankelijkheden / blokkades | Eigenaar | Status | Rubric |
|---|---|---|---|---|---|---|---|---|
| FR-O01 — bewaar en vind gegevens na koude herstart | Virtio-blokbroker, volume/catalogus, READ/WRITE/FLUSH, document-session | Bevestigde writes teruglezen in nieuwe VM; kill vóór/tijdens/na commit en geen bevestigde data verliezen binnen contract | VM-FR; HW-FR | N,D,H | Gecombineerd native profiel, onafhankelijke diskoracle en echte powerroute | opslag | open | S01,S02,S03,S07 |
| FR-O02 — werk met bruikbare volumes en documenten | Volumeallocator/compactie, streams, quota en bounded buffers | Normaal lang gebruik/hergebruik/grote invoer; Full/Quota/deelwrite/overflow en afgebroken compactie | VM-FR; HW-FR | N,D,P,H | Proefcaps verantwoord uitbreiden; productomvang/limieten exact vastzetten | opslag | open | S04,S05,K07 |
| FR-O03 — organiseer metadata en gebruik geschiedenis/herstel | NOSVOL02/IPC v6 of versieerbare opvolger, collecties/revisies | Namen/collecties/metadata/revisies wijzigen en herstel als nieuwe revisie; conflict, delete/recover en onderbroken transactie | VM-FR; HW-FR | N,D,V,H | Native appkoppeling, herstelbeleid en indexconsistentie | opslag/UI | open | S03,S06,U03 |
| FR-O04 — herstel bij opslagfouten | Supervisor, block-/volumeservice, epochs/reset/quarantaine | Dienstherstart/corruptie/timeout/diskuitval; Unknown blijft geen ACK, geen write-replay, geen stille herinitialisatie en overige apps reageren | VM-FR; HW-FR | N,D,V,P,H | FR-I01, requestercrash/reset-zonder-ACKbewijs, algemeen herstelpad | opslag/kern | open | S07,S08,K05,K07 |
| FR-O05 — bescherm inhoud, metadata en sleutels | Keyring/entropy, geauthenticeerde persistente volume-/objectlaag | Persistente seal/unseal/recovery; manipulatie/tag/header/replay/rollback volgens threat model, sleutelverlies/intrekking en cold reopen | VM-FR; HW-FR | N,D,H,A | B02 open; geen zelfbedachte crypto, koude antirollback expliciet | beveiliging/opslag | open | B01,B02,B06 |
| FR-O06 — importeer/exporteer en maak herstelkopieën | Formaatadapters, export-/backupportaal, onafhankelijke kopie | Echte ondersteunde bestanden import/export/restore; corrupt/incompatibel, naamconflict, vol doel, afgebroken transfer en bronverlies | VM-FR; HW-FR | N,D,V,X,H | S09-bron eerst terugvinden/hergebruiken; concrete formaten/compatibiliteitsgrenzen | opslag/apps | open | S09,A04,R01 |
| FR-O07 — zoek lokale inhoud en metadata | Gescopeerde lokale index-/zoekdienst, object/revisie-identiteit | Nieuwe/geüpdatete/verwijderde inhoud vinden; indexcrash/rebuild, deny/revoke, geen vreemde domeinresultaten en geen indexlek | VM-FR; HW-FR | N,D,V,P,H | Index persistentie, werkelijke queries en toegangsgrens | opslag/UI | open | U05,S06,B06 |
| FR-O08 — initialiseer nieuwe opslag veilig | Volumedetectie/-initialisatie, format-/versiemarker, expliciete herstelkeuze | Nieuw aangewezen leeg testvolume gebruiken; onbekend/niet-leeg/corrupt/ouder volume weigeren of gecontroleerd migreren; onderbreking bewaart eerdere bevestigde data | VM-FR; HW-FR | N,D,V,H | Nooit automatisch een niet-herkend bestaand volume overschrijven; fysieke target pas na gerichte autorisatie | opslag/release | open | S02,S04,S08,R01 |
| FR-N01 — configureer netwerk en maak verbinding | Native netdriver/-dienst, IP/TCP/UDP, adres-/routebeheer | Echte packets/configuratie/reconnect; malformed packets, backpressure, linkverlies, timeout en bounded resources | VM-FR; HW-FR | N,T,P,H | Native stack en benoemde deviceprofielen | netwerk | open | N01,N02,H04 |
| FR-N02 — gebruik naamresolutie | Native DNS/resolver-/netwerkdienst, procesbestemmingsrechten | Geldige naam/response/cache; NXDOMAIN, timeout, malformed/truncated, stale antwoord en weigering | VM-FR; HW-FR | N,T,H | FR-N01, werkelijk protocolconforme testresolver | netwerk | open | N02,N04 |
| FR-N03 — gebruik geauthenticeerd beveiligd transport | Onderhouden TLS/crypto, keyring/entropy, hybride PQC-contract | Authentieke peer/hostname/ALPN; verlopen/verkeerd certificaat, mismatch, malformed handshake, geen stille klassieke/onveilige fallback | VM-FR; HW-FR | N,T,P,H,A | Native crypto/TLS-port; host-N02 is voorbereidend bewijs | netwerk/beveiliging | open | N03,B01,B04 |
| FR-N04 — sta netwerk per app/bestemming toe en trek in | Kernelcapabilities, netwerkbroker, grants/generaties | Browser mag bedoelde bestemming; deny/revoke op actieve/nieuwe verbinding, helperroute en onverwachte luisterpoort | VM-FR; HW-FR | N,T,H | Socket-/bestemmingsafdwinging; LP-08–10/23/30/31 | netwerk/beveiliging | open | N04,A03,B06 |
| FR-P01 — koppel eigen of gedeelde NAS | Native client/NAS-rol, identiteitscontrole, transport en uitnodiging | Gecontroleerd koppelen/ontvangen/aanbieden; verkeerde peer, oude/hergebruikte/verlopen invite en afzonderlijk LAN/internetrecht | VM-FR met test-NAS; HW-FR | N,V,T,H | FR-N01–N04, native key lifecycle; LP-02/03/19/24 | NAS | open | P01,P04,B06 |
| FR-P02 — bewaar versleuteld op NAS en werk offline | Client-side encrypted objects, keyring, lokale volume-/synclaag | Namen/inhoud vóór verzending versleuteld; NAS-state+admincredentials geven geen plaintext/keys, outage/timeout behoudt lokale bruikbaarheid | VM-FR met test-NAS; HW-FR | N,D,V,T,H,A | FR-O05, geen hostclient of NAS-inferentie; LP-01/14/15/20/22 | NAS/beveiliging | open | P01,B02,B06 |
| FR-P03 — synchroniseer, hervat en los conflicten op | Native syncengine, revisies, tickets en peergebonden autoriteit | Offline edits, herverbinden/LAN-internetwisseling; concurrente schrijvers, gedeeltelijke upload, conflict, crash en geen stille dataverliezen | VM-FR, meerdere testclients; HW-FR | N,D,V,T,P,H | Persistente sync-/recoverykeys en begrensde queues | NAS/opslag | open | P02,P03,S09 |
| FR-P04 — nodig uit, deel, beheer quota en trek ruimte in | Afzonderlijke NAS-admincapability, quota/reserveringen, tombstones | Ruimte aanbieden/ontvangen/delen/revoke/evict; volle quota, abort/retry, offline/weigerende gast, oude sessie/upload na herstart geen herleving | VM-FR met test-NAS; HW-FR | N,D,V,T,H | Beheerder kan eigen kopie wissen zonder gebruikerssleutel/consent; LP-16–18/21/25 | NAS | open | P04,P01,B06 |
| FR-P05 — voeg apparaten toe, roteer/herstel gebruikerssleutels | Deviceidentiteit, keyring, geauthenticeerde multidevice-recovery | Nieuw geautoriseerd apparaat en rotatie; intrekking, verloren sleutel, gelijktijdige schrijvers en geen provider-key-escrow | VM-FR, meerdere testclients; HW-FR | N,D,V,T,H,A | Threat model/rollbackgrenzen en onafhankelijk herstelmedium | NAS/beveiliging | open | P03,B01,B02 |
| FR-P06 — kies optionele externe ciphertextopslag | Provider-onafhankelijke native encrypted-objectadapter, providernamespace/-credentials en netwerkbroker | Alleen gekozen ciphertextnamespace; credentials zonder content key, manipulatie/rollback, outage/delete en revoke zonder stille fallback | VM-FR met testprovider; HW-FR | N,D,V,T,H,A | Expliciete keuze, FR-O05/N04; LP-26–28; provider is geen OS-vertrouwensanker | NAS/provider | open | P01,B02,B06,S09 |
| FR-L01 — activeer/verwijder lokale AI en kies een model | Native modelbeheer, geïsoleerde downloader, pinned model/licentie/formaat | Opt-in/download/install/remove; gewijzigde modelbytes, unsupported parser, geen model en te weinig opslag/RAM; OS blijft bruikbaar | VM-AI; HW-AI | N,D,V,T,P,H | Model/engineport en profiel; downloader krijgt geen persoonlijke context/keys | AI | open | L01,A02,B06 |
| FR-L02 — gebruik AI met expliciet geselecteerde context | Echte lokale inference op CPU/GPU/NPU van profiel, contextbroker | Afgesproken taken op nieuwe invoer, echte verwerking/antwoord; ontbrekend model/capaciteit en onleesbare/niet-geselecteerde context | VM-AI; HW-AI | N,V,T,P,H | Geen vaste antwoorden; netwerkloze inference; context-/uitvoergrenzen | AI | open | L01,B06 |
| FR-L03 — annuleer AI en controleer toegestane acties | Begrensde job-/actiebroker, quota en lifecycle | Cancel tijdens verwerking en gescopeerde actie; deny/revoke, geheugen-/CPUdruk, modelcrash en helper-/socket-/uploadweigering | VM-AI; HW-AI | N,V,T,P,H | FR-K03, geen algemene shellbroker/cloudfallback; LP-04–07 | AI/beveiliging | open | L01,K07,A03,B06 |
| FR-L04 — behoud native lokale quantumfunctie | Bestaande lokale simulator/jobinterface met quota/annuleren | Referentiecircuits en annulering na integratie; fout circuit/resourcegrens en geen externe quantumcloud | VM-FR; benoemd HW-profiel | N,P,H waar vereist | Bestaand L02 niet verliezen; fysieke QPU blijft apart onderzoek | quantum/integratie | open | L02 |

### Beveiliging, updates, integrale validatie en oplevering

| ID / gebruikershandeling | Native componenten / interfaces | Normaal + foutscenario's | Profiel | Bewijs | Afhankelijkheden / blokkades | Eigenaar | Status | Rubric |
|---|---|---|---|---|---|---|---|---|
| FR-B01 — vertrouw entropy en sleutelbescherming | Virtio-rng/geschikte fysieke bron, broker, keyring met onderhouden crypto | Gezonde bron/key lifecycle/recovery; stuck/no-device/time-out, monotone intrekking, requestercrash en reset zonder ACK/quarantaine | VM-FR; HW-FR | N,D,H,A | B01 historisch bewijs scopebewust hergebruiken; fysieke RNG/TPM niet verzinnen | beveiliging/drivers | open | B01,B02 |
| FR-B02 — boot/verifieer alleen toegestane images | Boot-/signature-/PQC-contract, sleutelrotatie en distributieketen | Geldige testrelease met herkenbare testsleutels; gewijzigde kernel/image/signatuur, onbekende key, rotatie en trust failure | VM-FR; HW-FR | N,D,H,A,X | Native bootauthenticiteit, afgebakende testsleutels; geen productieclaim | beveiliging/boot | open | B04,B03 |
| FR-B03 — update het systeem en herstel na mislukking | Native updateplanner/installatie, slots/transacties, authenticiteit/integriteit | Geldige update en toegestane vorige versie; corrupt/verkeerd ondertekend, onderbreking elke kritieke fase, downgradeaanval en gebruikersdatabehoud | VM-FR; HW-FR | N,D,V,H,A | Update-/rollback-/migratiecontract en echte recoveryboot | updates/opslag | open | B03,R01 |
| FR-B04 — behoud privacy en Security Floor | Afdwinging op kernel-/dienstgrenzen, lokale logs, download-/appgrants | Alle toepasselijke LP-01–31; schone installatie/periodieke diensten/privacypopup uit zonder capabilitydelta; geen OS-telemetrie of AI-fallback | VM-FR/VM-AI; HW-FR/HW-AI | N,D,V,T,H,A | Alle relevante native componenten en negatieve helperroutes | beveiliging | open | B06,A03,N04 |
| FR-B05 — laat beveiligingsgrenzen onafhankelijk controleren | Kernel-/dienst-/DMA-/parser-/capabilitycontracten, gekozen formele modellen | Afgebakende deskundigenreview en machinegecontroleerde gekozen invarianten; blockers herstellen en opnieuw testen, beperkingen publiceren | Exacte releasebron/profielen | A,N,H waar vereist,X | Menselijke externe deskundigen nog niet uitgevoerd; AI-review geen vervanging | beveiliging/release | open | B05,B04,R03 |
| FR-R01 — doorloop het dagelijkse product als geheel | Alle productcomponenten op één finale image | Eerste start, sessie/apps, materiaal CRUD/restore, cold save, browser/download/media, appbeheer/instellingen, NAS/AI/rechten/update en herstel; fouten uit elke betrokken rij | VM-FR/VM-AI; HW-FR/HW-AI | N,D,V,T,P,H,A | Alle functionele rijen; geen losse featureimages als totaalbewijs | integrator/verifier | open | U01–U06,A02–A04,P01–P04,L01,R01 |
| FR-R02 — start herhaald en werk langdurig gecombineerd | Productworkload, resource-/performanceobserver met lokale logs | Honderd koude starts en24uur gecombineerde productsessie op stabiele finaleimage; geheugengroei, CPU/IPCdruk, volle opslag en netwerkstoringen; dataintegriteit behouden | VM-FR/VM-AI; HW-FR/HW-AI | N,D,P,H | Duur/workload/p50/p95/energie/geheugen/latencygrenzen vooraf vastzetten, niet achteraf versoepelen | release/verifier | open | R02,K07,H05 |
| FR-R03 — installeer/herstel op ondersteunde fysieke configuraties | Gecontroleerde image, installatie-/herstelprocedure en hardwaredrivers | Praktijkproef met behoud-/wisopties; mislukte start/update en zelfstandige recovery op benoemde onderdelen | HW-FR/HW-AI | N,D,V,H,A,X | Concrete apparaatidentiteit en gerichte toestemming voor fysieke handelingen ontbreken; meerdere configuraties waar H06 vereist | hardware/release | open | H06,R01,H02–H05 |
| FR-R04 — reproduceer en beoordeel het complete testpakket | Broncommit, build/start/recoverytooling, handleiding, matrix en evidencebundle | Schone build/controle van hashes, Nederlandse taakhandleiding/testplan, log-/foutmeldroute; ontbrekende dependency/licentie/SBOM en onherleidbaar artifact blokkeren | Schone bouwomgeving + alle releaseprofielen | X,N,H,A waar vereist | Volledig pakket uit §14, onderhouds-/patchbeleid en reproduceerbaarheid | integrator/release | open | R03,A01,B04 |

De rubriclink verwijst naar bestaande criteria; een samengestelde verwijzing
zoals `U01–U06` omvat alle tussenliggende bestaande ID's. De provideradapter
uit de privacyroadmap blijft expliciet zichtbaar ook al heeft de huidige
vaste rubric geen afzonderlijk P05-punt. Dat verandert geen gewichten of
acceptatieverplichting. Volledige formele verificatie, CHERI, extra ISA's en
fysieke QPU-besturing behouden hun al afzonderlijk vastgelegde onderzoeksroute;
zij zijn geen reden bestaande releasevereisten uit deze matrix te halen.

## Uitvoering en integratie

De cyclus blijft onderzoeken → implementeren → bouwen → uitvoeren → beoordelen
→ repareren → integreren. Gebruik bestaande kernel-, service-, SDK- en
toolimplementaties; los gerichte fouten op zonder algemene architectuurherstart.
Bibliotheken/engines zijn toegestaan na herkomst-, versie-, onderhoud-,
licentie-, no_std-/ABI-/buildscript- en sandboxbeoordeling. De kernel blijft
klein: productbeleid, documentformaten, rendering, cryptobeleid en inferentie
horen in geïsoleerde diensten/apps. Een noodzakelijke grenswijziging krijgt
motivatie, contract en negatieve tests vóór integratie.

### Eerste concrete verticale integratie: FR-I01

**Doel:** het generieke supervisor-/procespad, echte blok-I/O en de bestaande
entropy/keyringbron tegelijk op één eigen-kernelprofiel laten functioneren.
Deze stap is een afhankelijkheid van FR-K02, FR-O04 en FR-B01; zij voltooit
geen desktop, B02 of volledige release.

- De integrator bezit de gedeelde launch-/startupcapability-/syscallcontracten,
  featureselectie en finale profielconfiguratie. Behoud `proc_spawn`-posities,
  beperkte Launch/Process-rechten, Message320 en bestaande block-/entropy-API's.
- Gewone appprocessen krijgen taakendpoints, geen block-/entropy-/MMIO-/DMA-
  of onbeperkt startrecht. De supervisor mag uitsluitend de bedoelde diensten
  beheren; geen algemene rootcapability toevoegen om integratie te versnellen.
- Bevestig echte blokbytes/WRITE/FLUSH en echte entropycompletion via hun
  bestaande brokers op hetzelfde image. Laat client en onafhankelijke
  voortgangsprocessen doorwerken bij dienstcrash, timeout, deny/revoke en
  resource-uitputting. Oude epochs/handles worden geweigerd; onbekende I/O
  krijgt geen succes-ACK of replay. DMA-pools blijven gepind tot reset-ACK
  of gaan in quarantaine.
- Acceptatie: gerichte hosttests, freestanding build, echte normale en negatieve
  VM-cases, onafhankelijk diskoracle, bron-/artifacthashes en relevante K05,
  volume/storage-resilience en keyringregressies op de samengevoegde bron.
  Fixtureinjecties expliciet labelen; een testfeature bewijst alleen haar
  onderzochte eigenschap. Het uitvoerbare commando en werkelijke uitslagen
  komen in de actuele werkqueue/status, niet als verzonnen uitvoering hier.

### Parallel werk met één integratie-eigenaar

Gebruik alle werkelijk beschikbare onafhankelijke agentplaatsen nuttig, met
eigen worktrees/buildmappen waar nodig. Alleen een geslaagde agentstart telt
als actieve capaciteit. De taakbrief bevat ID, doel, exclusieve schrijfbestanden,
contractversie, afhankelijkheden, invarianten, foutgevallen, acceptatiecommando's
en oplevervorm. Eén schrijver per gedeeld bestand en één eigenaar per
kernel-/wire-/SDK-interface. Een reviewer begint read-only. Een verifier
controleert de werkelijk geïntegreerde bron en artifacts onafhankelijk van
de builder; AI-verificatie blijft als zodanig gelabeld.

Onafhankelijke sporen zijn kern/IPC, drivers, opslag/herstel, UI/ontwerp,
apps/appbeheer, netwerk/NAS/provider, lokale AI en beveiliging/update/release.
Beschikbare plaatsen worden op concrete afhankelijkheden verdeeld; niet ieder
spoor hoeft tegelijk actief te zijn. De hoofdagent integreert conflicten met
inhoudsbehoud, controleert actuele Git-/PR-status en regressies, onderhoudt
score/status/queue en levert het ene werkende product op. Een afgeronde
agenttaak is geen productaftekening.

### Integrale test- en aftekenvolgorde

1. Bevries profiel, bestandsformaten, capaciteit en meetbare prestatie-/duur-
   en herstelcriteria vóór de relevante gates. Bewaar baseline en foutgevallen.
2. Integreer elke verticale functie met normale/negatieve native proeven;
   herhaal relevante bestaande regressies na gedeelde grenswijzigingen.
3. Bouw het finale image reproduceerbaar en doorloop FR-R01/FR-R02 als
   samenhangend product, met koude opslagcontrole en netwerkobservatie.
4. Voer vereiste menselijke/accessibility-/deskundigenproeven en afzonderlijke
   geautoriseerde fysieke profieltests werkelijk uit. Bereid bij ontbrekende
   autorisatie het concrete pakket voor; werk onafhankelijk verder.
5. Herstel blockers en test opnieuw op de definitieve bron. Vul iedere
   matrixrij met bron-/hashgebonden bewijs en lever het volledige pakket.

**“Volwaardige testrelease gereed” is uitsluitend toegestaan wanneer alle
verplichte matrixrijen slagen op de exact geleverde geïntegreerde image(s),
alle bestaande vereisten op hun voorgeschreven bewijsniveau zijn aangetoond,
geen blokkerende defecten resteren en het complete testpakket reproduceerbaar
is.** Bestaande verplichte fysieke/human-auditcriteria blijven open tot echt
bewijs; geen VM-only voltooiing en geen onbevoegde uitvoering van externe gates.
Een volledig geslaagde VM-functiepoort met open externe gates is een
tussentijdse geïntegreerde VM-validatie, geen afgeronde opdracht.

## Oplevering, blokkades en hervatten

Het testpakket bevat geïntegreerde broncode/commit; het werkelijk geteste
releaseimage met hashes; reproduceerbare build; start-/herstelprocedures;
toegankelijke Nederlandse gebruikershandleiding; ondersteunde configuraties;
een productbreed beoordelingsplan; ingevulde matrix/bewijs; uitsluitend
niet-blokkerende bekende beperkingen; en een duidelijke foutmeld-/logroute.
Development-testsleutels zijn herkenbaar als testsleutels. Compatibiliteits-,
hardware-, security- en prestatieclaims blijven beperkt tot echt onderzochte
eigenschappen en aannames.

Bij een blokkade blijft de relevante rij open/geblokkeerd met reden en
afhankelijkheid. Een autorisatiegrens wordt niet omzeild. Fysieke schrijftests,
firmware/bootwijzigingen, publicatie, betaalde infrastructuur en externe
opdrachten volgen alleen hun concrete bevoegdheidsgrens. De integrator maakt
de concrete image, instructies en herstelroute eerst reviewbaar.

Bij iedere onderbreking bewaart het actuele `NEXT AGENT HANDOFF` in
PROJECT-STATE: exacte branch/commit en lokale wijzigingen; laatste bruikbare
artifact met hashes/configuratie; open matrix-/rubric-ID's; uitgevoerde
commando's/exitstatus/logs en niet-uitgevoerde tests; concrete blokkades; en
één eerstvolgende uitvoerbare hervatstap. Lees bij hervatten dit document,
actuele status/queue/AGENTS en de werkelijke bron opnieuw op verschillen.
De opdracht hieronder en de expliciete ciphertextverduidelijking blijven
inhoudelijk intact bij samenvatting, compaction en overdracht.

## Oorspronkelijke opdracht — onverkort

De onderstaande kopie heeft 14 genummerde secties. De SHA-256 van de exacte
UTF-8-tekst tussen de markers, zonder omringende marker-newlines, is
`e47f7a5885fd1f45d29a1edec7883a3dbaa8fdfba97f396fd1ebda9549d1eea2`.
Dit maakt onbedoelde latere wijziging van de bewaarde opdracht controleerbaar;
het is geen claim van externe authenticatie of releaseacceptatie.

<!-- FULL-RELEASE-REQUEST:BEGIN -->
ROL EN HOOFDOPDRACHT

Je bent de hoofdontwikkelaar en integratieverantwoordelijke van Nieuw OS.

Mijn opdracht is om door te ontwikkelen tot een volwaardige, samenhangende, professioneel afgewerkte testrelease van het besturingssysteem. Ik wil het complete product kunnen beoordelen en gebruiken op het vastgelegde testplatform.

Het einddoel omvat werkende systeemfuncties, een verzorgde gebruikersomgeving, bruikbare basisprogramma’s, instellingen, opslag, netwerk, lokale AI, NAS-functionaliteit, beveiliging, updates en herstel.

Een bootdemo, notitieproef, technische preview, browserprototype, verzameling losse componenten of interface met gesimuleerde functies voldoet niet.

Leg deze opdracht vast in docs/FULL-TEST-RELEASE.md. Verbind dit document met de bestaande projectstatus en werkqueue, zodat er één consistente bron voor voortgang ontstaat. Behoud de inhoud van deze opdracht bij samenvatten en hervatten.

Maak vervolgens een uitvoerbaar plan en begin direct met de uitvoering. Een plan opleveren is geen afronding van de opdracht.


1. CONTROLEER DE WERKELIJKE PROJECTSTAND

Lees de actuele AGENTS.md-bestanden, README, projectstatus, architectuurbesluiten, productspecificaties, privacycontracten, beveiligingscontracten, buildinstructies en testprocedures.

Controleer de actieve branch, commits, relevante ontwikkelbranches, open pull requests en bestaande lokale wijzigingen. Gebruik de actuele implementatie en herleidbaar testbewijs als uitgangspunt.

Oude chatclaims, voortgangspercentages, documentatie en geslaagde tests van andere commits gelden niet automatisch als bewijs voor de huidige release.

Bouw voort op bruikbare bestaande implementaties. Hergebruik en integreer goed werk. Los conflicten, dubbele implementaties en tegenstrijdige interfaces op zonder bestaand gebruikerswerk te verliezen.

Voer geen algemene architectuurherstart uit wanneer een gerichte verbetering volstaat.


2. LEG DE VOLLEDIGE ACCEPTATIE VAST

Maak een acceptatiematrix voor de volledige testrelease.

Vermeld per verplichte functie:
- Welke concrete gebruikershandeling moet werken.
- Welke native componenten en interfaces daarvoor nodig zijn.
- Welke normale en foutscenario’s worden getest.
- Op welk hardware- of VM-profiel de functie wordt ondersteund.
- Welk bewijs nodig is om de functie af te tekenen.
- Welke afhankelijkheden of blokkades nog openstaan.

Gebruik de bestaande productafspraken én de onderstaande eisen als basis. Deze opdracht vervangt het eerdere beperkte einddoel van alleen opstarten en een korte notitie opslaan.

Verlaag de productomvang niet om sneller een groen resultaat te krijgen.

Verwijder, verberg of deactiveer geen verplichte functie om de acceptatie te laten slagen. Verplaats moeilijke releasevereisten niet zelfstandig naar “later”. Als een eis nog niet uitvoerbaar is, blijft deze zichtbaar open.

Maak onderscheid tussen bestaande releasevereisten en onderzoek dat in de vastgelegde roadmap al een afzonderlijk traject heeft. Gebruik dat onderscheid niet om nieuwe uitzonderingen te creëren.


3. BEHOUD DE EIGEN ARCHITECTUUR

Nieuw OS blijft een zelfstandig besturingssysteem met een eigen kernel en eigen productervaring.

Behoud de afgesproken microkernelrichting, capabilitybeveiliging, procesisolatie, IPC-contracten en scheiding tussen kernel, drivers en systeemdiensten.

Vergroot de vertrouwde kern niet uitsluitend om integratie gemakkelijker te maken. Motiveer noodzakelijke wijzigingen en controleer de gevolgen.

Gebruik toegestane standaarden, bibliotheken, engines en ontwikkelgereedschappen wanneer die het project helpen. Controleer daarbij compatibiliteit, onderhoud, licenties en beveiligingsgrenzen. Een browserengine of andere bibliotheek hoeft niet opnieuw te worden uitgevonden als verantwoord hergebruik mogelijk is.

Vervang de eigen kernel niet door Linux, BSD, XNU, Zircon of een andere bestaande kernel.

Native functionaliteit moet via de eigen kernel en systeemdiensten werken. Hostgereedschappen mogen bouwen, testen en emuleren, maar mogen ontbrekende OS-functionaliteit niet uitvoeren alsof die al in Nieuw OS bestaat.

Testgegevens en protocolconforme testservers zijn toegestaan. Gesimuleerde succesmeldingen en hostprototypes gelden niet als implementatiebewijs.


4. LEVER EEN COMPLETE GEBRUIKERSOMGEVING

Werk Materiaal uit tot een samenhangende, native gebruikersomgeving.

Zorg voor:
- Een verzorgde opstart- en eerstegebruikservaring.
- Gebruikersprofiel, sessie, vergrendeling en ontgrendeling.
- De afgesproken schermen voor materiaal, bewerken, gereedschappen, toegang, apparaten en assistent.
- Betrouwbare navigatie, focus en bediening met toetsenbord en muis.
- Werkende selectie, contextmenu’s, klembord en slepen waar de productspecificatie dit vraagt.
- Begrijpelijke meldingen, voortgang, annulering en foutafhandeling.
- Consistente instellingen die na herstart behouden blijven.
- Correct herstarten en afsluiten, met voorafgaande afhandeling van opgeslagen en onopgeslagen werk.
- Energie- en hervatgedrag volgens de vastgelegde eisen van het ondersteunde profiel.

Iedere zichtbare bediening moet de bedoelde handeling werkelijk uitvoeren.

Voorwaardelijk onbeschikbare functies, bijvoorbeeld door ontbrekende hardware of geweigerde toestemming, moeten hun werkelijke toestand en reden tonen. Een ontbrekende implementatie mag niet als zo’n normale toestand worden vermomd.


5. MAAK VISUELE KWALITEIT EEN RELEASEVOORWAARDE

Het OS moet coherent, rustig, herkenbaar en professioneel aanvoelen.

Gebruik één samenhangend ontwerpsysteem voor typografie, kleuren, iconen, afstanden, componenten, animaties en interactietoestanden. Bewaar de gekozen eigen identiteit en de Materiaal-richting.

Ontwerp en controleer ook:
- Laden, leegte, fouten, bevestiging en herstel.
- Focus, selectie, hover en uitgeschakelde bediening.
- Lange teksten, verschillende venstergroottes en beeldschalen.
- Toetsenbordnavigatie, leesbaarheid, contrast en toegankelijkheid.
- Samenhang tussen systeemschermen en meegeleverde programma’s.

Geen lorem ipsum, onafgewerkte standaardvensters, kapotte lay-outs, onverklaarde technische meldingen of “coming soon” voor verplichte functies.

Houd vormgeving en assets onderhoudbaar, zodat later aangeleverde ontwerpen consistent kunnen worden geïntegreerd.

Controleer de daadwerkelijke gerenderde interface. Screenshots ondersteunen die controle, maar vervangen geen bedieningsproef.


6. LEVER BRUIKBARE BASISPROGRAMMA’S EN APPBEHEER

Implementeer de basisprogramma’s uit de productspecificatie. Neem voor deze testrelease minimaal op:
- Materiaal-/bestandsbeheer met zoeken en ordenen.
- Een bruikbare teksteditor.
- Afbeeldingsweergave.
- PDF-weergave.
- Een webbrowser.
- Audio- en videoweergave voor vastgelegde gangbare formaten.
- Een rekenmachine.
- Schermopname of screenshotfunctionaliteit.
- Systeeminformatie, diagnose en toegankelijke hulp.
- Appinstallatie, bijwerken en verwijderen.

Leg ondersteunde bestandsformaten en gedrag concreet vast. Maak openen, opslaan, standaardgereedschappen en toegangsrechten samenhangend.

Een programma dat alleen opent of een vooraf gevuld scherm toont, voldoet niet.

Appinstallatie moet via een werkende pakket- en rechtenstructuur verlopen. Ondersteun de afgesproken installatiebronnen. Behandel een geldige handtekening als bewijs van herkomst, niet als algemene toestemming.

Claim compatibiliteit met programma’s van andere besturingssystemen alleen wanneer die daadwerkelijk is geïmplementeerd en getest.


7. MAAK OPSLAG GESCHIKT VOOR WERKELIJK GEBRUIK

Werk de opslaglaag uit tot een bruikbare basis voor de volledige testrelease.

Ondersteun de afgesproken functies voor namen, metadata, organiseren, zoeken, wijzigen, verwijderen, geschiedenis en herstel.

Verwijder experimentele capaciteitsbeperkingen die normaal gebruik verhinderen door de onderliggende implementatie verantwoord uit te breiden.

Zorg voor:
- Duurzame opslag na volledig afsluiten en opnieuw starten.
- Correcte grenzen bij grote invoer, volle opslag en mislukte I/O.
- Behoud van eerder bevestigde gegevens bij relevante fouten.
- Veilige initialisatie, ruimtebeheer en herstel.
- De afgesproken versleuteling en bescherming van inhoud en metadata.
- Betrouwbare import en export in de ondersteunde formaten.

Een herstelde geheugensnapshot is geen bewijs van duurzame opslag.

Voer herstelproeven uit op wegwerpimages en controleer de werkelijk opgeslagen gegevens onafhankelijk van alleen succesmeldingen in de interface.


8. INTEGREER NETWERK, NAS EN LOKALE AI

Deze onderdelen behoren tot deze testrelease en mogen niet worden teruggebracht tot losse demonstraties.

NETWERK
Lever werkende netwerkconfiguratie, verbinding, naamresolutie en beveiligd transport voor het gekozen profiel. Controleer netwerkuitval, herstel, time-outs, certificaatfouten en geweigerde rechten.

NAS
Voer het actuele privacycontract uit voor expliciet gekoppelde eigen of gedeelde NAS-opslag:
- Werkelijke koppeling en identiteitscontrole.
- Versleuteling voordat persoonlijke inhoud wordt verzonden.
- Sleutels bij de gebruiker volgens het afgesproken model.
- Werkende opslag, synchronisatie en foutafhandeling.
- Quota, uitnodigen, delen en intrekken.
- Het afgesproken onderscheid tussen gebruiker en NAS-beheerder.
- Lokale bruikbaarheid wanneer de NAS onbereikbaar is.
- Afzonderlijk toegestane verbindingen volgens de actuele afspraken.

Een protocolconforme test-NAS mag als externe tegenpartij dienen. De OS-client, rechtencontrole en lokale verwerking moeten werkelijk in Nieuw OS functioneren.

LOKALE AI
Lever een optioneel te activeren, daadwerkelijk werkende lokale AI-functie op een passend ondersteund profiel:
- Werkelijke modeluitvoering op het eigen apparaat.
- Expliciete selectie van context.
- Begrensde toegang en acties via de afgesproken broker.
- Annulering en zichtbare verwerkingstoestand.
- Beheersing van geheugen- en rekengebruik.
- Begrijpelijk gedrag bij ontbrekend model of onvoldoende capaciteit.

Gebruik geen vooraf geschreven antwoorden als bewijs dat AI werkt.

Behoud de afgesproken uitsluiting van leverancierscloud voor persoonlijke OS-opslag, verplichte leverancieraccounts, externe OS-telemetrie en cloudfallback voor systeem-AI.


9. LEVER WERKENDE BEVEILIGING, UPDATES EN HERSTEL

Beveiligingsbeleid moet daadwerkelijk worden afgedwongen door de betreffende systeemgrenzen.

Controleer onder meer:
- Proces- en dienstisolatie.
- Rechtenverlening, weigering en intrekking.
- Gevalideerde communicatie tussen processen.
- Bescherming van opslag en sleutels.
- Veilige entropy en correcte cryptografische integratie.
- De vastgelegde boot-, ondertekenings- en PQC-contracten.
- Foutisolatie en begrensde resourceconsumptie.

Schakel controles niet uit om een demonstratie te laten slagen.

Lever een werkend update- en herstelpad:
- Controle van authenticiteit en integriteit.
- Correcte installatie.
- Herstel bij een mislukte of onderbroken update.
- Een gecontroleerde terugkeer naar een toegestane werkende versie.
- Behoud van gebruikersgegevens volgens het herstelcontract.

Gebruik duidelijk afgebakende testsleutels waar dat voor ontwikkeling nodig is. Presenteer die niet als productie-infrastructuur.

Claim geen onkraakbaarheid, universele hardwareondersteuning of volledige formele verificatie. Koppel iedere beveiligingsclaim aan de werkelijk onderzochte eigenschap en aannames.


10. WERK MET MAXIMALE NUTTIGE PARALLELLISATIE

Gebruik werkelijk beschikbare agents om onafhankelijke onderdelen parallel uit te voeren.

Verdeel taken bijvoorbeeld over:
- Kernel, processen en IPC.
- Drivers en hardwareondersteuning.
- Opslag en herstel.
- Gebruikersomgeving en ontwerp.
- Basisprogramma’s en appbeheer.
- Netwerk en NAS.
- Lokale AI.
- Beveiliging, updates en releasevalidatie.

Stem de verdeling af op beschikbare capaciteit en concrete afhankelijkheden.

Leg eigenaarschap, interfacecontracten en acceptatie per taak vast. Gebruik afzonderlijke worktrees waar nodig. Voorkom gelijktijdige ongecoördineerde wijzigingen aan dezelfde bestanden.

De hoofdagent blijft verantwoordelijk voor samenvoeging, conflictoplossing, regressies en de werking van het gehele OS.

Het afronden van een subtaak betekent niet dat het product klaar is.


11. TEST HET SAMENHANGENDE PRODUCT

Voer echte gebruikersscenario’s uit op het uiteindelijke releaseimage.

Controleer minimaal:
- Eerste start en inrichting.
- Dagelijks starten, vergrendelen, ontgrendelen en afsluiten.
- Meerdere programma’s gebruiken en ertussen wisselen.
- Materiaal maken, openen, bewerken, zoeken, verwijderen en herstellen.
- Opslaan en terugvinden na koude herstart.
- Browsergebruik en downloaden binnen verleende rechten.
- Media openen en afspelen.
- Apps installeren, bijwerken en verwijderen.
- Instellingen wijzigen en na herstart terugvinden.
- NAS koppelen, gebruiken, onderbreken en opnieuw verbinden.
- Lokale AI met geselecteerde context gebruiken en annuleren.
- Toegang weigeren en intrekken.
- Update, mislukte update en herstel.
- Relevante resource-, opslag- en netwerkfouten.

Test daarnaast herhaald opstarten en langere gecombineerde gebruikssessies met vooraf vastgelegde criteria.

Leg voor prestaties meetbare grenzen vast op het referentieprofiel. Meet in plaats van alleen te schrijven dat iets “snel” of “soepel” voelt.

Koppel bewijs aan de exacte commit, configuratie en hashes van de geleverde images. Een geslaagde hosttest of compilatie vervangt geen native uitvoering.

Verklaar de release pas klaar wanneer alle verplichte acceptatiecriteria slagen en bekende blokkerende defecten zijn opgelost.


12. BESCHERM DE TESTCOMPUTER

Bouw en valideer eerst in een gecontroleerde virtuele omgeving met eigen testimages.

Gebruik voor netwerk- en NAS-proeven afgebakende testnetwerken, testgegevens en toegestane testservers.

Koppel geen persoonlijke schijven, echte NAS-inhoud, accounts of fysieke apparaten zonder gerichte autorisatie. Gebruik geen fysieke schijfwrites, partitionering, firmwareflash of wijziging van de bestaande bootconfiguratie als automatische vervolgstap.

Beperk bouw-, start- en resetscripts tot hun aangewezen bestanden en paden.

Lever voor fysieke hardware een apart profiel met ondersteunde onderdelen, gecontroleerde image, concrete instructies en herstelroute. Een geslaagde VM-test mag niet als geslaagde test op mijn laptop worden gepresenteerd.

Ontbrekende informatie over mijn fysieke apparaat mag onafhankelijk ontwikkelwerk en VM-validatie niet stilleggen.


13. BLIJF UITVOEREN TOT HET EINDDOEL IS BEREIKT

Werk voortdurend in de cyclus:
onderzoeken → implementeren → bouwen → uitvoeren → beoordelen → repareren → integreren.

Neem normale technische beslissingen zelfstandig binnen de opdracht en bestaande bevoegdheden.

Stop niet na:
- Een plan.
- Een documentatie-update.
- Een deelcommit of pull request.
- Een geslaagde compilatie.
- Een afzonderlijke werkende functie.
- Een tussentijdse mijlpaal.

Geef korte voortgangsberichten met afgeronde acceptatiecriteria, werkelijk bewijs, eerstvolgende werk en concrete blokkades. Ga daarna verder.

Behoud de bestaande voortgangsrubric. Verhoog percentages alleen wanneer de vereiste bewijzen bestaan.

Stop bij volledige acceptatie, mijn pauze/stop, een harde systeem- of budgetlimiet, of een externe blokkade waarvoor geen toegestane vervolgstap bestaat.

Werk bij een gedeeltelijke blokkade de onafhankelijke onderdelen af. Omzeil geen toestemmingen, limieten of beveiligingsregels.

Bewaar bij onderbreking:
- De exacte werkstand.
- Het laatste bruikbare artifact.
- Open acceptatiecriteria.
- Uitgevoerde tests en hun uitkomsten.
- Concrete blokkades.
- De eerstvolgende uitvoerbare hervatstap.

Noem een onderbroken of geblokkeerde opdracht niet voltooid.


14. LEVER EEN COMPLEET TESTPAKKET OP

De oplevering bevat:
- De geïntegreerde broncode en bijbehorende commit.
- Het werkelijk geteste releaseimage.
- Een reproduceerbare bouwprocedure.
- Start- en herstelprocedures.
- Een toegankelijke Nederlandse gebruikershandleiding.
- De ondersteunde configuraties.
- Een testplan waarmee ik het gehele product kan beoordelen.
- Het ingevulde acceptatieoverzicht met bewijs.
- Bekende beperkingen en resterende niet-blokkerende bevindingen.
- Een duidelijke route voor foutmeldingen en logs.

De eindverklaring “volwaardige testrelease gereed” is alleen toegestaan als het geleverde pakket de volledige verplichte acceptatie doorstaat.

Begin nu met de actuele repositorycontrole, leg het volledige werk vast en ga aansluitend door met implementatie.
<!-- FULL-RELEASE-REQUEST:END -->
