# Aanvullende uitvoerings- en productcontext — 30 oktober 2026

Deze normatieve aanvulling verwerkt de twintig secties van de aanvullende
gebruikersopdracht. Zij vult [FULL-TEST-RELEASE](FULL-TEST-RELEASE.md) aan;
de oorspronkelijke volledige opdracht, `/goal`, architectuur, 62 stabiele
acceptatie-ID's, uitgevoerde werkzaamheden en geldige brongebonden bewijzen
blijven behouden. Zij is geen afzonderlijke roadmap of lagere releaseomvang.

## Vergelijking met de actuele uitvoering

Al vastgelegd: eigen Rust-kernel/microkernelrichting, capabilities/IPC/isolatie,
Materiaal, volledige apps/NAS/AI/update/herstelomvang, optionele opaque externe
opslag, onderscheid tussen bewijsniveaus en VM-FR versus smalle regressies.
VM-FR is al pc-q35-8.2/Nehalem-v1/1vCPU/TCG/8GiB/32GiB sparse disk, met
vastgelegde GOP/PS2/virtio-apparaten en aparte TPM2-CRB/swtpm-state.

Werkelijk uitgevoerd: native diensten/documenten/recordcrypto, eigen NIC met
ARP/UDP/DNS, FPU/klok en nu TCP met protocoltegenpartij; recente bronversies
staan in PROJECT-STATE. Dit zijn begrensde deelprofielen met plaintext
testdisks/RAM-sleutels, geen veilige persistente productopslag of releaseimage.
Native desktop, complete gebruikstaken en het volledige testpakket blijven open.

Concreet aangevuld: onderstaand taak-/compatibiliteitsregister, minimale
formaatdekking, expliciete Chromium/Gecko-vergelijking, toegankelijkheidsdetails,
TCB per eigenschap en de vaste honderd koude starts/24uur-productsessie.
Lopende geheugen/allocator/TLS/entropy/RTC-taken nemen de relevante runtime-
en beveiligingsvoorwaarden mee. UI-/appwerk blijft een verplichte integratieroute.

## Vast product- en consumentencontract

Nieuw OS moet hoogwaardig dagelijks gebruik ondersteunen voor communicatie,
internet/dienstverlening, administratie/werk/studie/onderwijs, gaming/muziek/video,
fotografie/ontwerp/creatie en uiteenlopende digitale vaardigheden/toegankelijkheid.
Het Europese alternatief en de eigen kernel zijn strategische keuzes, geen
bewijs van consumentenvoorkeur, technische superioriteit of veiligheid.
Rust blijft hoofdtaal waar voorgeschreven. Geen bestaande hostkernel als vervanging.
Verantwoord engine-/libraryhergebruik en hostgereedschappen zijn toegestaan;
hostfunctionaliteit geldt niet als uitvoering door Nieuw OS.

Consumentenliteratuur levert hypothesen, geen reeds gemeten Nieuw-OS-gebruik:
noodzakelijke taken kunnen uitvoeren, merkbaar voordeel, begrijpelijk/herstelbaar
proberen en overstappen, waargemaakte dagelijkse verwachtingen, begrijpelijke
privacy/veiligheid en verzorging samen met betrouwbaarheid. Onderzoek afzonderlijk
installatie op bestaande hardware, gebruik naast een bestaande omgeving en
aanschaf van een ondersteunde computer: migratie, kosten, ondersteuning en herstel
verschillen. Neem tevreden niet-overstappers serieus. Belangstelling, willen
proberen, werkelijk kunnen overstappen en vrijwillig blijven gebruiken zijn
afzonderlijke uitkomsten; downloads/positieve reacties/AI-review bewijzen geen
retentie, marktaandeel, voorkeur of betalingsbereidheid. Geen verzonnen interviews.

## Complete gebruikstaken en harde belemmeringen

Alle rijen hieronder zijn open. VM-FR/HW-FR zijn doelen, geen actuele
ondersteuningsclaim. Elke uitvoering vult concrete app/site/account/bestand/device-
versies, beginsituatie, grenzen, configuratie en bewijs in. Producttests gebruiken
eigen fixtures/testtegenpartijen; echte accounts, inhoud, apparaten of ingrijpende
handelingen vereisen de bestaande gerichte autorisatie. Een noodzakelijke
ontbrekende app/game/device/toegankelijkheidsfunctie blijft een zichtbare harde
overstapbelemmering, ook als andere gemiddelden gunstig zijn.

| ID en doel/beginsituatie | Benodigdheden en normale uitkomst | Fouten/herstel | Bevoegdheden | Configuratie/bewijs en FR-koppeling |
|---|---|---|---|---|
| UC01 document ontvangen→bewerken→school/werk sturen | Concrete communicatiebron, editor, UTF8/afgesproken uitwisselformaat; inhoud/metadata behouden, bewerken/bewaren/exporteren/verzenden | Corrupt/groot bestand, conflict, offline, onbekende opslaguitkomst; herstel zonder valse ACK | Alleen gekozen bron/object/save/export/bestemming | VM-FR/HW-FR; native gehele keten, onafhankelijke bytes/coldreopen; A01,A02,A13,O01–O07,N01–N04 |
| UC02 bankieren/overheidsdienst | Benoemde ondersteunde site/browser, testaccount en toepasselijke authenticatie; complete representatieve taak | TLS/identiteit/certtijd/verbinding/cancel; geen onbedoelde transactie of stille retry | Benoemde bestemming, expliciete credential-/apparaatgrants; geen echte financiële handeling als automatische test | Native browser/transport/taakbewijs, externe compatibiliteit afzonderlijk; A05,N01–N04,B04 |
| UC03 videobellen | Benoemde app/site/versie, camera/mic/audio en gecontroleerde gesprekspartner | Rechtenweigering/intrekking, deviceverlies, mute/stop/offline/herstel | Afzonderlijke camera/mic/audio/netwerkgrants | VM-FR waar beschikbaar plus HW-FR; hoorbare/zichtbare uitvoering, latency en intrekking; H01–H03,A05,A06,N04 |
| UC04 foto's importeren→selecteren→bewerken→delen→terugvinden | Concrete bron/apparaat, PNG/JPEG, Materiaal en editor; kwaliteit/metadata/ordening behouden | Grote/corrupte foto, vol volume, verkeerde revisie, offline delen, herstel | Gekozen importbron, objecten, export/deelbestemming | Native hele keten, onafhankelijke referentiebeelden en coldreopen; A01,A03,A13,O01–O07 |
| UC05 creatief project correct opleveren | Benoemde toepassing/versie/project, fonts/plugins/gekoppelde bestanden, collega's en export/printdoel | Ontbrekende afhankelijkheid, getrouwheids-/kleur-/exportfout en herstel | Geselecteerd project, graphics/audio/devices en gecontroleerde uitwisseling | Benoemde combinatie/taakgetrouwheid/prestatie; geen algemene suiteclaim; A13–A15,H02,O07 |
| UC06 specifieke game met vrienden | Titel/versie/launcher/account/install/aankopen/saves/multiplayer/chat/controller/headset | Install-/save-/verbinding-/anti-cheat-/DRM-probleem; geen hostbeveiliging verlagen | Begrensde eigen opslag/netwerk/venster/input/audio/devices | Exacte combinatie en voorwaarden, FPS/framepacing/inputlatency/stabiliteit; compatibiliteitsregister, H01–H03,A11–A12,N04 |
| UC07 bestanden overzetten→coldreopen | Benoemde bron/bestanden/formaten, import/export/volume; exacte inhoud terugvinden | Deelwrite, vol, corruptie, offline, conflict/onzekere ACK; bevestigde bytes behouden | Alleen geselecteerde bron/doel/objecten | Koude nieuwe VM, onafhankelijk bytes/orakel; O01–O07,R01 |
| UC08 fout herstellen zonder onduidelijk verlies | Benoemde fout, historie/back-up/recovery/update en bekende gegevens | Stroomonderbreking, sleutel-/anker-/slotverlies, mislukte update | Gescopeerde herstelautoriteit, expliciet downgradebeleid | Wegwerpimages/disk+afzonderlijkeTPM-state, echte recovery en gebruikersdatacontrole; B01–B03,O04–O06,R01 |
| UC09 zelfstandig toegankelijk werken | Keyboard-only of echte schermlezer, native semantiek/status, 200% en passende input | Focusval, verloren status, kleurafhankelijkheid, schaal/lange tekst, herstel | Afgebakende accessibility-/focus-/surfacegrants | Werkelijk gerenderd en bediend, menselijke beoordeling waar vereist; U01–U05,R01 |

## Materiaal, formats en compatibiliteit

Materiaal houdt objecten herkenbaar; direct apps openen blijft mogelijk. Namen,
ordening, openen/bewerken/bewaren/delen, locatie/historie/rechten, voortgang en
werkelijke annulering zijn consequent. Opgeslagen/bezig/mislukt blijven verschillend;
geen stil verlies. Meldingen/onderbrekingen zijn regelbaar; kernfuncties werken
zonder AI, NAS of internet. Vernieuwing maakt gewone taken niet onnodig moeilijk.
Eén onderhoudbare bron voor native tokens/componenten inclusief toegankelijkheids-
semantiek, typografie, iconen, spacing, focus/selectie, fouten en beweging.
Iconen blijven herkenbaar. Keyboardfocus/schermlezerstatus, contrast/leesbaarheid,
lange teksten, 200%, reduced motion en informatie zonder kleur zijn releasevoorwaarden.
Toegankelijkheidsrechten en de beveiligingsgrens worden samen ontworpen.

Alle bestaande basisapps plus tabel-/presentatiemateriaal blijven verplicht.
Minimale formaatdekking: UTF8, PNG/JPEG, PDF, WAV/FLAC/MP3, MP4(H264/AAC) en
WebM(VP9/Opus). Per formaat worden lezen/schrijven/bewerkingen/profielen,
inhoud/metadata, corruptie/grootte/unsupportedvarianten en referentieresultaten
concreet vastgelegd vóór aftekening. Een extensie of startend venster bewijst
geen volledige ondersteuning. Parsers/codecs/engines krijgen onderhouds-,
licentie-, dependency/buildscript-, target-, resource- en isolatiecontrole.

| Compatibiliteitsregister | Verplichte vastlegging vóór een claim | Toestand |
|---|---|---|
| COMP01 benoemde gamecombinatie | UC06 plus native port/API-ABIlaag/runtime/gastkeuze, titel/versie, voorwaarden en alle functionele/prestatieblokkades | Gewenst concreet doel nog te benoemen; geen universele gamestore/anti-cheatbelofte |
| COMP02 benoemde professionele creatieve combinatie | UC05 plus projectgetrouwheid/fonts/plugins/links/kleurbeheer/GPU/randapparaten/collega-export/print | Gewenst concreet doel nog te benoemen; geen Adobe/Windows-suitecompatibiliteitsclaim |
| COMP03 compatibiliteitsomgeving | Werkelijk afgedwongen grens, onderhoud/runtime, onderlinge apps in gedeelde gast, geselecteerde portals/netwerk/surfaces/input/audio/devices/print | Optioneel ontwerpbesluit vereist; gastkernel vervangt hostkernel niet |

Gaming/professionele creatie blijven expliciet zichtbaar; de prioriteit is het
bruikbare OS. Geen automatische opdracht om volledige creatieve suites opnieuw
te bouwen of onbeperkte universele compatibiliteit af te tekenen. Een laag is
geen isolatiebewijs; gedeelde gast kan een gezamenlijke vertrouwensruimte zijn.
Geen algemene hostrechten of hostbeveiligingsverlaging voor compatibiliteit.
Gegevensoverdracht verloopt via begrensde open/save/klembord/drag/printportalen.

## Veiligheid, vertrouwen en gegevensstromen

Definieer de TCB per eigenschap: boot/initialisatie/kernel, policybroker,
sleutelbeheer, compositor en relevante hardware kunnen mede vertrouwd zijn.
Kleine kernel/LOC/Rust bewijzen geen veiligheid; controleer unsafe/FFI/veilige
abstracties en logica/races/deadlocks/autorisatie. Formele eigenschappen/aannames,
tests en review blijven onderscheiden; geen geërfde seL4-bewijzen of absoluteclaims.
Een overgenomen component blijft binnen expliciete begin-, object-, domein-,
operatie-, delegatie-/intrekkings-/lifetime- en middelenrechten. IPC/sharedmemory
controleert grenzen/overflow/pointers/alignment/ownership/TOCTOU/replybinding/
cancel/deadline/cleanup. Zero-copy alleen met bewezen passende lifetimes.
W^X/geheugen/proces-/FPU/SSE-isolatie en begrensde wachtrijen/quota blijven aan.
Lezen plus netwerk kan toegestane verzending mogelijk maken: leg gecombineerde
rechten uit. Intrekking blokkeert toekomstige toegang, haalt oude kopieën niet terug.

Drivers zijn volgens architectuur geïsoleerd; voorlopige kerneltransporten en
resterend vertrouwen expliciet. Userspacedriver betekent geen DMA-isolatie:
IOMMU/groepen/topologie/IRQ/bufferlevensduur, crash/reset/no-ACK/revoke/reuse
zijn concrete gates. Ontbrekende isolatie betekent zichtbare beperking/weigering,
geen garantie. Softwarecompositie blijft basis; GPUversnelling is afzonderlijke
technische opgave, geen verplichte tweeGPU-consumentenconfiguratie. Meet juiste
hardware; beveilig focus/input/surfaces/capture/klembord/overdracht. Vertrouwde
prompts zijn herkenbaar buiten kleur. Isolatie van onbetrouwbare media/documenten
en opstart/cleanupkosten meetbaar; geen onbewezen onmiddellijke restvernietiging.
Timing/cache/SMT/hardwarezijkanalen zijn afzonderlijke vragen, geen algemene garanties.

## Opslag, netwerk, crypto, AI en herstel

Lokaal/eigenNAS/gedeeldeNAS blijven mogelijk; externeprovider alleen expliciet
gekozen client-side opaque ciphertext, zonder inhoudssleutels/plaintext/gevoelige
manifests. Encrypt ook namen/gevoelige metadata vóór provideradapter; credentials
gescheiden. Geen verplichte leverancieraccount/cloud, escrow/stille fallback/
OStelemetrie/externe systeeminferentie. Eigenprovideraccount verleent geen OSrechten.
Client/NASrollen: beheerder beheert quota/blokkeert/verwijdert eigen gastkopieën
zonder gastdecryptiesleutel/toestemming; geen inhoudslezen of wissen op andere
apparaten. Duurzame keys/unlock/rotatie/herstel, contextgeauthenticeerde objecten/
manifests, transacties/ACKs, offline/resume/conflicten, gedeeltelijke uploads/
revoke, import/export/onafhankelijke back-up blijven verplicht. Encryptie garandeert
geen beschikbaarheid/anonimiteit/herstel; grootte/timing/verbindingsmetadata kunnen
zichtbaar zijn, sync is geen back-up. LAN/naam/privateIP is geen authenticatie.
Herhaal binnen verleende rechten automatisch; nieuw vertrouwen/ruimere rechten/
ingrijpende handeling vraagt opnieuw de relevante autorisatie.

Netwerk omvat driver/config/routes/IP/TCP/UDP/DNS/tijd/geauthenticeerdtransport/
proces-bestemmingsrechten. Host is uitsluitend tegenpartij. Geschikte entropy en
onderhouden crypto, geen voorspelbare productseed/eigenprimitief. Scheid inhouds-
encryptie/KEM/signatuur/keybeheer/bootverify/meting. ML-KEM is geheimovereenkomst,
ML-DSA handtekening, geen toegangspolitiek. Hybride/PQC/bootcontract blijft aan;
bij vereiste dubbele signatures is één onvoldoende. Crypto blijft waar passend
buiten kernel, boot in expliciete boot-TCB. Updates: authenticiteit/integriteit,
key/algoritmerotatie, slots/atomische activatie, elke onderbreking, expliciete
downgrade en offlineherstel met gegevensbescherming. Antirollback en herstel
samen testen: TPM2-CRB met eigen swtpm-state, disk/ankertransacties, ankerverlies/
herstel. Hostrollback van zowel disk als TPM valt buiten deze VMgarantie.
Geen secrets in bronnen/gewone logs/foutmeldingen/testarchieven.

AI is optioneel, werkelijk lokaal, zonder direct LAN/internet; expliciete context,
CPU/geheugen/uitvoerquota, annulering en beperkte actiebrokers. Document/modeluitvoer
verleent geen autoriteit. Afzonderlijke downloader alleen gekozen modellen, geen
persoonlijke documenten/index/prompt/NASkeys. Herkomst/licentie/integriteit/parser/
resources/target van model en engine toetsen. Ontbrekend model/capaciteit:
begrijpelijke beperking/queue/kleiner ondersteund lokaal model/AIuit; geen externe
inferencefallback. Kernfuncties blijven bruikbaar. Expliciete versleutelde AIback-up
mag volgens opslagcontract; NAS/provider worden daardoor geen inferentiedienst.

## Uitvoering, bewijs en eindpakket

Behoud de Chromium/Gecko-vergelijking vóór enginebesluit. Concrete browser- en
inferenceversies moeten runtime/toolchain/geheugen/threading/FS/net/graphics/CPU/
sandbox/resources/onderhoud aantonen. Targetproeven tonen echte lokale pagina of
modeluitvoering; hostproef apart. Vroege keten boot→nativeUI→input→app→veilig
bewaren→afsluiten→coldreopen is tussenmijlpaal, geen lagere eindscope. Native
quantumfunctie behouden; fysiekeQPU/CHERI/extraISA afzonderlijk bestaand onderzoek.

Maximale nuttige echte agents met exacte basis, exclusieve files/worktree/outputs,
interface/afhankelijkheden/acceptatie/oplevering. Root bezit sharedABI/workspaces/
build/status/Git en integratie. Hergebruik caches, behoud werk, geen algemene
heraudit/reinstall zonder concrete reden. Workerpass is geen geïntegreerdepass.
Git volgens bestaande autorisatie/branchregels; controleer daadwerkelijke remote.

Bewijsniveaus: ontwerp, hostprototype, hosttest, targetbuild, eigenkernel-QEMU,
geïntegreerde gebruikershandeling, fysieke test, menselijk onderzoek/review en
formeel bewijs. Geen onderlinge vervanging. Regressieprofiel behouden; VM-FR
alleen wijzigen met gemotiveerd besluit, éénCPU bewijst geen SMP, TCG geen fysieke
game-/AI-prestatie. Test verkeerde/grants/context/generatie/identiteit, corruptie/
overflow/grootte/vol/OOM, proces/driver/servicecrash, I/O/resetnoACK, netwerk/cert,
transactie/powercut/staleaanvraag, key-/updateverlies, conflicten/Unknown-ACK.
Timeout alleen groen als het bedoelde negatieve gedrag echt is vastgesteld.
Werkelijk renderen én bedienen, schalen/accessibility/fout/herstel. Budgetten vóór
aftekenen vastzetten; geen gunstige metingen of achteraf verschoven grenzen.
**Honderd koude starts en 24uur gecombineerde productsessie** zodra stabiele
integratedimage beschikbaar; de verplichting blijft nu open, nooit verkorten
tot snelle deelproeven. Exact commit/config/tools/deps/model/assets/commando/exit/
loader-kernel-imagehash/log/meting binden, na laatste relevante wijziging herhalen.
Freeze tijdens tests; CI geldt alleen eigencommit, overgeslagenreview is geenreview,
AIreview vervangt geen menselijke gate.

Eén voortgangsketen, ongewijzigde rubric/eindscope/gewichten; defecten zichtbaar,
extra tests verdienen niet automatisch punten. Autonoom doorgaan met volgende
onafhankelijke stap; maximaal één noodzakelijke productverduidelijking wanneer
werkelijk blokkerend. Fysieke/destructieve/productiedata/credentials/betaalde/
publicatiegrenzen respecteren, reviewbaar pakket voorbereiden en onafhankelijk
doorwerken. Bij echte sessie/context/tool/quotalimiet exacte commit/worktrees/dirt/
artifacts/bewijs/tests/actieve/open taken/hervatcommando bewaren; geen latere
uitvoering beloven. Alleen compleet geslaagd eindpakket heet volwaardige testrelease:
bron, werkelijk geteste image, reproduceerbare bouw, start/herstel/update, Nederlandse
handleiding, configuraties, matrix/bewijs/beperkingen en fout-/logroute.

Primaire beslisankers blijven seL4 proofs/assumptions/capabilities, Rust Nomicon
races, Linux VFIO/IOMMU, Qubes FAQ, LionsOS verificatiestatus en NIST FIPS203/204
zoals in de gebruikersinstructie. Raadpleeg toepasselijke versies gericht; deze
bronnen leveren geen implementatie, kernelvervanging of geërfd bewijs op.


- https://sel4.systems/Verification/proofs
- https://sel4.systems/Verification/assumptions.html
- https://docs.sel4.systems/Tutorials/capabilities.html
- https://doc.rust-lang.org/nomicon/races.html
- https://kernel.org/doc/html/latest/driver-api/vfio.html
- https://doc.qubes-os.org/en/latest/introduction/faq.html
- https://lionsos.org/
- https://csrc.nist.gov/pubs/fips/203/final
- https://csrc.nist.gov/pubs/fips/204/final
