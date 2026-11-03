# AUDIT-06 — gegevensstromen en gebruikerscontrole

Datum: 3 november 2026, Europe/Amsterdam. Dit is een begrensde statische
privacy-inventaris door een AI, geen onafhankelijke menselijke audit,
AVG-verklaring, exploitonderzoek of productacceptatie.

## Opdracht, bron en afbakening

Letterlijke opdracht: "Inventariseer gegevensstromen, metadata, retentie,
toestemming, lokale verwerking, NAS-koppelingen en gebruikerscontrole."

| Onderdeel | Vaststelling |
|---|---|
| Repository | `simonvanieperen/nieuw-os` |
| Bevroren bronbranch | `copilot/add-os-specialist-agents` |
| Beoordeelde broncommit | `beafdedfc188154fe7285848fbd411e651013f54` |
| Eigen beginbranch | `simonvanieperen-friendly-pancake`, niet detached |
| Eigen begin-HEAD | `69809c2cb53dfb3bd77abe5352892810fba6e82d` |
| Beginwerkboom | Schoon; de gevraagde commit bestond lokaal als commitobject |
| Bronselectie | Na die controle `git switch --detach beafdedfc188154fe7285848fbd411e651013f54`; exit 0 |
| Werkstand tijdens onderzoek | Detached HEAD op de bevroren bron; niet werken op of schrijven naar de bronbranch |
| Eigen worktree | `/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-friendly-pancake` |
| Enig schrijfbestand | `docs/audits/subagents/06-privacy.md` |
| Rol | `.github/agents/os-privacy-reviewer.agent.md:12-31` gelezen als rolinstructie; geen technisch geladen customagent |

Gelezen instructies: root `AGENTS.md`, `docs/AGENTS.md` en de instructies voor
services, UI, kernel en tests. README, het bovenste checkpoint van
PROJECT-STATE, FULL-TEST-RELEASE, privacycontract revisie 4 en relevante
implementatiecontracten/besluiten zijn als broncontext gebruikt.
De huidige opdracht beperkt bewust de algemene checkpointregels: geen
productcode, score, roadmap, projectstatus, commit, push, PR of tweede rapport.
De integrator verzamelt de rapporten en verzorgt eventuele gezamenlijke opname.

Geen andere checkout gelezen. `git worktree list` is uitsluitend metadata.
Geen actuele GitHub-PR-/CI-status opgevraagd; lokale refs zijn geen live
remotecontrole. Geen CodeRabbit/API-review, externe broncontrole, download,
installatie, echte gegevens, account, NAS of apparaat gebruikt. Geen nieuwe
subagents, builds of tests gestart.

## Kernuitkomst

Het privacyontwerp begrenst inhoud, sleutels en netwerkrechten duidelijk,
maar de complete productketen is niet aangetoond. Er is echte clientencryptie
in een Node-hostprototype, Rust-code voor NAS-autoriteit en afzonderlijk
historisch native bewijs voor lokale NAS-tokenintrekking en klembordoverdracht.
Die bewijzen sluiten geen native gekoppelde client/NAS/provider-/AI-keten.

Voor gebruikerscontrole zijn drie grenzen doorslaggevend: intrekking haalt
eerder verstrekte kopieën niet terug; DELETE bewaart herstelbare versies;
encryptie verbergt geen account-, grootte-, timing- of verbindingsmetadata.
De onderzochte native sessie is bovendien geen authenticerend
gebruikersprofiel. Gevoelige echte gegevens op deze ontwikkelstand zijn
daarom buiten de aangetoonde productscope.

## Bewijsmodel

| Niveau | Betekenis in dit rapport |
|---|---|
| E0 | Ontbrekend of alleen ambitie |
| E1 | Ontwerp of contract |
| E2 | Hostprototype, hosttest of simulatie |
| E3 | Native broncode; compilatie alleen bewijst geen uitvoering |
| E4 | Historisch native positief resultaat, uitsluitend voor genoemde bron en profiel |
| E5 | Historisch native negatief resultaat met afgebakend geweigerd scenario |
| E6 | Brongebonden herhaalde CI-/releasegate; een workflowbestand alleen is geen E6 |
| E7 | Gedocumenteerde fysieke hardwaretest van de betreffende eigenschap |
| E8 | Onafhankelijke menselijke audit of formele evaluatie met expliciete scope |

Alle **eigen** controles hieronder zijn statische bron-/bestandscontroles.
E4–E6 worden uitsluitend aan reeds aanwezige historische rapporten ontleend;
zij zijn niet opnieuw uitgevoerd en niet naar `beafdedf` omgelabeld. Geen
E7/E8 voor de geïnventariseerde privacyketen vastgesteld. Dit rapport
controleert niet de volledige hashketen van alle historische artifacts.

## Dataflowregister

Een namespace is een afgebakende opslagruimte. Een tombstone is een bewaard
intrekkingsrecord dat voorkomt dat oude synchronisatie de ruimte opnieuw
aanmaakt. Een capability is een expliciet technisch recht, geen UI-checkbox.
Retentie is de bewaartermijn of de gebeurtenis waarna gegevens verdwijnen.

| ID / bron → doel | Gegevens en verwerkingsdoel | Ontvanger / zichtbaarheid | Retentie | Autoriteit / bewijs |
|---|---|---|---|---|
| D01 native Materiaal → catalogus/volume | Documentinhoud, naam, object/revisie, collectie, tags, logische aanmaak-/wijzigstempels; bewaren en terugvinden | Lokale app en toegewezen opslagdomein; volumeformaat bevat leesbare namen en inhoud, geen algemene encryptieclaim | Actuele versie plus twee eerdere versies; DELETE is herstelbaar; PURGE verwijdert historie uit de actieve tabel, geen fysieke wisgarantie | Domein-/objectroute; `services/volume/src/lib.rs:1-44`, `services/volume/src/table.rs:11-40,63-75`, `services/volume/src/service.rs:639-652`; E3 |
| D02 native app → compositor → andere app | Alleen expliciet geselecteerde UTF-8-tekst; kopiëren/plakken | Vertrouwde compositor en doelapp; geen hostklembord of netwerk toegevoegd | Vluchtig; handmatig wissen en compositorherstart wissen brokerinhoud; reeds afgeleverde appkopie niet terughaalbaar | Focus plus één actueel invoergebeurtenis/ticket; `docs/implementation/NATIVE-CLIPBOARD.md:9-55`; E3, historisch E4/E5/E6 voor beperkte route |
| D03 gebruiker → native aanmeldmodel | Accountkeuze, wachtwoordbytes, verzoek-/sessiegeneratie en ticket; aanmeldinteractie | Lokale interactiestaat, toekomstige AuthBroker; geen werkende persoonlijke authenticatie afgeleid | Eigen wachtwoordbuffer wordt gewist; tijdelijke compiler-/registerkopieën en crashbeelden vallen buiten die wisclaim | Vertrouwde adapter moet identiteit leveren; VM-gast is expliciet geen geauthenticeerde gebruiker; `ui/native/src/session.rs:1-10,177-214`; E3 |
| D04 gebruiker → browserdemo → sessionStorage | Profielalias, documentnamen/tekst/versies, concept, laatste object, toestemmingshistorie, NAS-alias/quotum, geselecteerde AI-context en huidig demoantwoord; interactie onderzoeken | Browsercontext onder sleutel `nieuw-os-prototype-v1`; geen OS-isolatie of encryptie | Hele state wordt bij dispatch geserialiseerd; vervalt/reset binnen browserbeheer, geen betrouwbare schijfwisclaim. Hervatten neemt niet alle velden terug en expireert grants | Geen echte OS-autoriteit; `ui/prototype/app.js:56-63,348-352`, `ui/prototype/state.js:239-294`; E2 |
| D05 browserdemo → voorbeeldassistent | Geselecteerde object-ID's, titel en eerste twee niet-lege tekstregels; vaste samenvattingssimulatie | Dezelfde browserstate; geen inferentiemodel | Antwoord tot wijziging/annulering/reset; context wordt deels hervat; gehele state kan vóór refresh in sessionStorage staan | Maximaal drie objecten, geen echte contextbroker; `ui/prototype/broker.js:129-140`, `ui/prototype/state.js:208-231,289-291`; E2 |
| D06 toekomstige persoonlijke context → lokale AI | Prompts, documenten, embeddings, zoekindex, historie en resultaten; lokale ondersteuning | Alleen gekozen lokale inferentiedienst/contextbroker; géén NAS-inferentie of clouduitwijk | Geen geïmplementeerde volledige retentie-/verwijderketen vastgesteld; versleutelde back-up alleen expliciet gekozen | Bindend verbod op AI-netwerk, aparte downloader; `docs/LOCAL-PRIVACY-CONTRACT.md:82-93`; E1, complete inferentie E0 |
| D07 modelbron → modeldownloader → artifactreader | Publieke modelbytes, herkomst/revisie/licentie, lengte, SHA-256, opaque artifact-ID/generatie; modelintegriteit | Aparte downloader mag geen persoonlijke context zien; reader ontvangt geen document-/netwerkcapability | Readerhandle eindigt bij abort/drop via adapter; modelinstallatie, opslagprovider, catalogus en uninstall ontbreken in deze leaf | `experiments/ai/model-store-contract/README.md:13-33,54-79`; E2-contract/hostbewijs, geen native modeluitvoering |
| D08 clientcipher → Node NAS-store | Documentnaam en inhoud samen AES-256-GCM-versleuteld; envelope bevat namespace, object-ID, revisie, nonce, tag en ciphertext | Synthetische hoststore ontvangt envelope, niet de inhoudssleutel; beheer ziet operationele metadata | Ciphertext tot verwijderen; revoke alleen stopt toegang; sessies/staging in RAM; gecommitteerde metadata op disk | Afzonderlijke admin-/toegangscapabilities binnen host-API; `services/nas/crypto.mjs:9-15,44-61,73-95`, `services/nas/store.mjs:275-311`; E2 |
| D09 NAS-gast/beheer → Rust NAS-core → volume | Owner/adminfingerprints, namespace, status, epoch, quota, uitnodiging/verval, opaque bytes/revisies; allocatie, publicatie en intrekking | NAS-service; beheer-API toont ruimte/status, geen documentinhoud; on-disk snapshot kent owneridentiteit | Sessies/uploads niet persistent; revoke bewaart inhoud, evict verwijdert beheerde objecten maar behoudt tombstone en owner; geen tijdslimiet daarvoor | Authentieke ingress is vereiste, geen callerboolean; `services/nas/native-core/src/lib.rs:99-121,678-695,740-778`, `src/snapshot.rs:75-89`; E3, lokale probe historisch E4/E5 |
| D10 clientqueue → staging/reconcile | Maximaal vier opaque edits, base-/remoteversie en conflictbytes; offline werk bewaren | Aanroeper en gekozen NAS-core; geen sleutels of providercredentials | RAM/checkpoint tot expliciete overdracht/afhandeling; checkpoint is geen duurzame lokale opslag | Bestaande caller/namespace/epoch/CAS, geen eigen identiteit; `services/nas/native-client/README.md:11-45,58-78`; E2 |
| D11 object-/keyservice → optionele provideradapter | Uitsluitend versleutelde inhoud, namen/manifests; provider ziet account, IP, opaque ID's, omvang, aantallen en timing | Expliciet gekozen provider, onbetrouwbare opslag; géén Nieuw-OS-inhoudssleutel | Providerretentie en verwijderbevestiging onbekend; provider kan historische ciphertext bewaren | Afzonderlijke provider/namespace/bestemmingsrechten; geen stille fallback; `docs/implementation/ENCRYPTED-OBJECT-STORAGE.md:186-228,250-273`; E1, productadapter niet aangetoond |
| D12 app/downloader → netwerkbestemming | Appverkeer of update-/modelbestanden; bestemming ziet noodzakelijke verbindingsmetadata | Alleen toegekende bestemming; systeemdownload geen recht op persoonlijke data | Volledige native verbindings-/diagnostiekretentie niet vastgesteld | Hostpolicy en Rust-brokercore bestaan; native caller-/issuerbinding en TLS-NAS-peerbinding zijn aparte open grenzen; `services/nas/README.md:81-95`, `services/net-broker/README.md:17-42`, `services/net-tls/README.md:8-45` |
| D13 sleutelservice → persistent key-state/opslag/herstel | Master, allocaties/intrekkingen versleuteld in blob; leesbare header bevat suite/formaat/volume-ID; extern herstelgeheim | Geautoriseerde key-/herstelroute, niet NAS-admin/provider; hostdemo kan expliciet ruwe recoverykey exporteren naar caller | Sleutelstatus blijft bewaard; lifetime/rotatie/herstel van volledige productketen niet bewezen; koude antirollback vereist apart anker | `services/keyring/src/persistent.rs:1-6,70-108,111-121`, `docs/implementation/B02-SECURE-RECORDS.md:84-117`, `services/nas/crypto.mjs:44-45`; E3 plus beperkte historische deelproeven |
| D14 ontwikkelproef → logs/screenshots/diskimage → CI-artifact | Synthetische invoer, PID's, counters, bron-/beeld-/diskhashes, serial/stderr, PPM's en volume.img; reproduceerbaar ontwikkelbewijs | Lokale onderzoekers en GitHub Actions-artifacttoegang volgens repository-/dienstinstellingen | In onderzochte vensterworkflow 14 dagen ingesteld; ingecheckte rapporten blijven Gitgeschiedenis. Geen gecontroleerde totale retentie van mirrors/caches/back-ups | `.github/workflows/native-window-session.yml:179-203`; geen OS-runtime-telemetrie. `docs/LOCAL-PRIVACY-CONTRACT.md:172-174` vereist synthetische data |

## Gebruikerscontrole en standaardinstellingen

| Controle | Huidige implementatie / grens | Wat de gebruiker nog niet aantoonbaar krijgt |
|---|---|---|
| Lokale start zonder leveranciersaccount | Productnorm; native openbare VM-gast en gesimuleerde onboarding bestaan | Persoonlijk geauthenticeerd profiel, veilige lock/unlock en sleutelvrijgave |
| Apprechten toestaan/weigeren | Demo toont concreet object, actie, duur en gecombineerd document-/internetrisico; annuleren heeft autofocus (`ui/prototype/app.js:173-178`) | Native privacydashboard met authentieke, actuele grant-/bestemmingsgegevens en bedienbare intrekking over alle diensten |
| Intrekken | Core-revoke invalidereert toekomstige handelingen en bewaart intrekking; klembordtickets vervallen | Terughalen van reeds afgeleverde tekst/export of door een app gemaakte kopie; dat is ook niet beloofd in het contract |
| Verwijderen/herstellen | Volume DELETE en PURGE bestaan als verschillende bewerkingen; NAS-admin mag eigen ciphertext verwijderen zonder gasttoestemming | Eén duidelijk native overzicht van actuele data, historie, staging, zoekindex, AI-data, lokale back-ups en externe kopieën met gescheiden bevestigingen |
| Export/dataportabiliteit | Node-recoveryexport en lezen van eigen objecten zijn bouwstenen; native kopiëren/plakken bestaat beperkt | Volledige gebruikersgerichte export/import inclusief formaat, metadata, sleutelherstel en provideronafhankelijke herstelproef |
| NAS koppelen/internet | Hostpolicy kiest binnen bestaand peerrecht LAN, anders toegestaan direct internet; geen herhaalde toestemming bedoeld | Productketen die identiteit, duurzame keuze, echte sessie en actuele intrekkingsgeneratie over reboot samenbindt |
| AI opt-in en verwijderen | Contract maakt AI optioneel/verwijderbaar; demo labelt vaste respons en ontbrekend model | Native install/remove, contextgebruiksoverzicht, retentiebediening en bewezen geen directe of gedelegeerde netwerkroute |
| Privacyuitleg uitschakelen | ADR-038 en LP-29 scheiden uitleg van capabilitybeslissing | Native aantoonbaar dezelfde rechten vóór/na toggle; een uitgeschakelde uitleg is niet "alles toestaan" |
| Beheer en sleutels scheiden | Hostcipher/clientkey en NAS-beheercapability zijn apart; core geeft geen inhoudssleutel aan admin | Volledige multi-user-/multi-device-key-lifecycle met onafhankelijke herstel- en intrekkingscontrole |

Dit is geen dark-patternvrijverklaring. De demo maakt simulatie, gevolgen van
intrekken en risico van lezen-plus-internet zichtbaar. De standaard
AI-context is echter al ingevuld (`ui/prototype/fixtures.js:42`) en wordt
bij hervatten behouden of opnieuw voorgekozen
(`ui/prototype/state.js:289-291`). Of gebruikers dit begrijpen en echt
vrijwillig kiezen is niet menselijk onderzocht. Normale automatische sync
binnen bestaande rechten is een producteis, geen reden om dagelijks opnieuw
consent te vragen.

## Geprioriteerde bevindingen

Ernst betreft de privacy-/acceptatie-impact binnen deze opdracht, niet een
bevestigde exploiteerbare kwetsbaarheid. BLOKKEREND blokkeert een genoemde
productclaim; HOOG vraagt eerst een sluitende grens; MIDDEL/LAAG vragen
gerichte product-/documentcorrectie. ONBEKEND is geen geslaagd resultaat.
Geen afzonderlijke KRITIEK-bevinding onderbouwd binnen deze statische scope.

### P06-01 — BLOKKEREND — geen geïntegreerde private NAS-/providerketen

- **Bron/doel/ontvanger/retentie:** D08–D13; persoonlijke inhoud naar gekoppelde opslag met een blijvend sleutel-/versie-/intrekkingsmodel.
- **Bewijs:** `services/nas/native-client/README.md:58-78` noemt ontbrekend gedeeld transport, object/keymapping, duurzame queue en native bediening. `services/net-tls/README.md:8-11,36-45` levert geen handshake/NAS-principal; `services/nas/native-probe/CONTRACT.md:3-7,71-76` beperkt de probe tot lokale processen. Providerontwerp is E1.
- **Product-/rechtsvraag:** welke gekozen identiteit, namespace, sleutel en opslaglocatie autoriseert de gebruiker werkelijk, en blijft die keuze geldig zonder stille verruiming?
- **Risico:** losse E2/E3-onderdelen of lokale E5-replay kunnen ten onrechte als privé multideviceproduct worden aangeboden; daarmee ontbreken controle over ontvanger, key recovery en duurzame intrekking.
- **Minimale maatregel:** één brongebonden native keten samenstellen met echte peerbinding, sleutelcontext, duurzame keuze/queue en expliciete providerkeuze; LP-01–03/15–28 aan die keten toetsen. Tot die tijd geen echte persoonlijke opslag claimen.
- **Menselijke beoordeling:** onboarding, begrip van beheerdersmacht, sleutelverlies en juiste ontvanger; deskundige beoordeling van de samengestelde grens. Geen algemene conformiteitsuitspraak.

### P06-02 — HOOG — lokale data en herstelhistorie zijn niet algemeen versleuteld

- **Bron/doel/ontvanger/retentie:** D01/D03/D13; documenten en namen op lokaal volume, wachtwoord-/key-state naar toekomstige sessie-/herstelagent.
- **Bewijs:** `services/volume/src/lib.rs:10-11` sluit encryptie/authenticatie uit; `table.rs:63-75` encodeert namen. `ui/native/src/session.rs:1-6` implementeert geen authenticatie. Historische sessieproef weigert persoonlijke invoer en opent alleen openbare gast (`docs/verification/session-desktop-20261102/README.md:3-9`).
- **Product-/rechtsvraag:** kan een andere lokale gebruiker of iemand met de testimage de inhoud zien, en welke bescherming denkt de gebruiker te hebben?
- **Risico:** proces-/domeinisolatie of een aanmeldscherm verwarren met bescherming van opgeslagen inhoud of echte multi-userauthenticatie.
- **Minimale maatregel:** openbare proef expliciet houden; gebruikersauthenticatie, sleutelvrijgave en versleutelde inhoud/naam-/indexrecords als één aantoonbare productroute verbinden.
- **Menselijke beoordeling:** lock/unlock, gedeeld apparaat, gastverwijdering en herstel zonder beheerdersleesrecht. Dit rapport stelt geen bypass vast; het stelt de ontbrekende productgrens vast.

### P06-03 — HOOG — wissen, historie en back-ups hebben geen gezamenlijke gebruikersgrens

- **Bron/doel/ontvanger/retentie:** D01/D08–D11/D13; inhoud, oudere versies, staging en onafhankelijke kopieën.
- **Bewijs:** `services/volume/src/service.rs:639-652` bewaart bij DELETE de inhoud als historie; `services/nas/native-volume/README.md:17-37` vereist PURGE maar sluit fysieke COW-sectorwis en diskrollback uit. `services/nas/store.mjs:290-311` onderscheidt revoke, evict en object-delete. Externe kopieën vallen buiten de wisclaim (`docs/LOCAL-PRIVACY-CONTRACT.md:59-65`).
- **Product-/rechtsvraag:** betekent "verwijderen" naar herstelbak, historie purgen, deze NAS-kopie verwijderen of sleutels vernietigen; wat blijft waar achter?
- **Risico:** verwijderde gegevens blijven herstelbaar of elders bewaard terwijl een interface voltooiing suggereert. De code onderscheidt de stappen, maar de volledige native bediening/retentiecontrole is niet aangetoond.
- **Minimale maatregel:** per stap scope, achterblijvende kopieën, fout en resultaat tonen; automatische bewaartermijnen en handmatige purge/export samen ontwerpen. Geen valse secure-erase- of providerretentieclaim.
- **Menselijke beoordeling:** herstelbaarheid versus definitief verwijderen, onverwacht dataverlies, woorden en bevestigingen; back-upbeleid apart beoordelen.

### P06-04 — HOOG — privacydashboard en gecombineerde netwerkrechten nog geen productcontrole

- **Bron/doel/ontvanger/retentie:** D04/D12; objectrechten, ontvangers/bestemmingen en gebruikshistorie.
- **Bewijs:** `ui/prototype/app.js:168-178` noemt het toegangsscherm expliciet simulatie; `services/net-broker/README.md:17-42` heeft geen authentieke native issuer. LP-29–31 blijven verplichte scenario's (`docs/LOCAL-PRIVACY-CONTRACT.md:160-164`).
- **Product-/rechtsvraag:** kan de gebruiker zien welke app welke inhoud naar wie mag sturen en toekomstig gebruik daadwerkelijk beëindigen?
- **Risico:** een checkbox/dashboard geeft schijncontrole zonder actuele dienst-/kernelfeiten. Lezen plus internet kan toegestane inhoud laten vertrekken; intrekken wist geen eerdere kopieën.
- **Minimale maatregel:** native overzicht uit gezaghebbende grant-/bestemmingstoestand, concrete gecombineerde waarschuwing en revoke naar alle betrokken diensten, inclusief helpers en bestaande verbindingen.
- **Menselijke beoordeling:** begrijpelijke bestemmingen, vindbare weigering, toegankelijke bediening en geen ongewenste toestemming door herhaaldruk.

### P06-05 — HOOG — lokale AI heeft nog geen volledige retentie- of netwerkgrens

- **Bron/doel/ontvanger/retentie:** D05–D07; context, prompts, model, afgeleide data en antwoord.
- **Bewijs:** `experiments/ai/model-store-contract/README.md:54-79` implementeert geen downloader, parsing of inferentie. `ui/prototype/broker.js:129-140` is vaste simulatie. Privacycontract `:82-91` eist lokale, optionele inferentie en weigering van helper-/socketomwegen.
- **Product-/rechtsvraag:** welke context wordt gebruikt, welke afgeleide gegevens blijven achter, hoe verdwijnen die, en kan een helper ze alsnog versturen?
- **Risico:** lokaal model-/readeronderzoek als bewijs presenteren dat prompts/indexen lokaal blijven of dat verwijderen/annuleren alle afgeleide data afhandelt.
- **Minimale maatregel:** context-/resultaat-/cache-/indexretentie vastleggen en native implementeren; inferentie en downloader scheiden; cancel/remove en directe plus gedelegeerde netwerkweigering apart aantonen.
- **Menselijke beoordeling:** vrijwillige modelkeuze, contextbegrip, foutuitleg en taakuitvoering zonder AI. Geen modeldownload of inferentie door deze audit uitgevoerd.

### P06-06 — MIDDEL — blijvende NAS-tombstones bevatten meer dan alleen intrekkingsstatus

- **Bron/doel/ontvanger/retentie:** D09; ownerfingerprint, namespace, epoch, quota en status blijven op NAS.
- **Bewijs:** `services/nas/native-core/src/lib.rs:99-109,742-778` wist bij revoke de uitnodiging/vervaltijd en bij evict de objecten, maar niet owner, namespace of quota. `src/snapshot.rs:75-89` bewaart/vereist owner ook in dit schema. Het bewaren van tombstones is expliciet anti-herlevingsontwerp (`native-core/README.md:58-70`).
- **Product-/rechtsvraag:** welke identifiers zijn blijvend noodzakelijk voor anti-herleving, en hoeft de eerdere eigenaar blijvend koppelbaar te blijven?
- **Risico:** blijvende relatie-/gebruiksmetadata na verwijdering zonder expliciete minimale veldenset of gebruikersuitleg; geen documentplaintextlek vastgesteld.
- **Minimale maatregel:** motiveer per veld de noodzaak en retentietrigger; onderzoek of een minimale tombstone voldoende is zonder eigenaarrelatie. Niet zomaar tombstones verwijderen en daarmee oude rechten laten herleven.
- **Menselijke beoordeling:** proportionaliteit van koppelbare metadata, bewaardoel en uitleg aan gast/beheerder; zo nodig juridisch toetsen, niet door deze AI aftekenen.

### P06-07 — MIDDEL — demo bewaart volledige state en kiest AI-context vooraf

- **Bron/doel/ontvanger/retentie:** D04/D05; ingevoerde namen, tekst/concepten, rechtenhistorie en voorbeeldantwoord in browser-sessionStorage.
- **Bewijs:** `ui/prototype/app.js:56-63,348-352`, `fixtures.js:1,42`, `state.js:239-291`. De begincontext is `obj-001`, geen lege context. Refresh expireert grants maar verwijdert niet daarmee alle document-/conceptgegevens. Reset is geen bewijs van browser-/back-upwis.
- **Product-/rechtsvraag:** begrijpt een proefpersoon dat eigen invoer in browseropslag kan belanden en dat een vooraf geselecteerd document niet dezelfde handeling is als zelf context kiezen?
- **Risico:** echte proefdata of onbewust geselecteerde context gebruiken ondanks het synthetische karakter. Dit is een prototypebevinding, geen native datalek.
- **Minimale maatregel:** uitsluitend synthetische input blijven gebruiken; bewaarvelden/retentie zichtbaar maken, duidelijke verwijderactie, en lege begincontext overwegen vóór menselijke proeven.
- **Menselijke beoordeling:** contextselectie en opslagverwachtingen; niet zonder gebruikersonderzoek concluderen dat de huidige UI misleidt.

### P06-08 — MIDDEL — diagnostiek en bewijsarchieven hebben verschillende retentiegrenzen

- **Bron/doel/ontvanger/retentie:** D14; serial, screenshots, schijfimages en hashes naar lokaal archief/Git/Actions.
- **Bewijs:** `.github/workflows/native-window-session.yml:179-203` uploadt ook volume.img en PPM's, 14 dagen ingesteld. `docs/verification/nas-replay-110/README.md:24-27,49-78` onderscheidt privé ruwe tokens en gehashte archiefbinding; de lekdetectie sluit kortere/anders gecodeerde fragmenten niet volledig uit.
- **Product-/rechtsvraag:** mogen toekomstige diagnosebundels inhoud of capabilitymateriaal bevatten, wie krijgt ze en wanneer verdwijnen lokale, ingecheckte en gehoste kopieën?
- **Risico:** ontwikkelbewijs later hergebruiken met echte data of globale logs presenteren als inhoudsvrije productdiagnostiek. Een ingesteld artifactverval verwijdert geen Gitgeschiedenis.
- **Minimale maatregel:** synthetische gate behouden; voor productdiagnostiek afzonderlijke minimale velden, toegangs-/exportkeuze, redactie en retentie per opslaglaag vastleggen.
- **Menselijke beoordeling:** welke bundel een gebruiker veilig kan delen; artifacttoegang en echte dienstretentie buiten deze lokale audit controleren.

### P06-09 — LAAG — algemene statuszinnen zijn verouderd ten opzichte van deelbewijs

- **Bron/doel/ontvanger/retentie:** documentatie naar ontwikkelaars en gebruikers; geen nieuwe gegevensstroom.
- **Bewijs:** privacycontract `:4` noemt native netwerk/sleutelservice breed open; `services/nas/README.md:17-25` meldt nog geen QEMU-uitvoer. Daartegenover staan de opgeslagen lokale NAS-run en sessie-/klembordproeven. Profielenplan `:14-18` beschrijft een oudere main zonder sessiepreview.
- **Product-/rechtsvraag:** welke bronversie en welk bewijsniveau hoort bij een uitspraak over huidige bescherming?
- **Risico:** bestaande native deelimplementatie ten onrechte als geheel ontbrekend voorstellen, of juist een lokale probe als complete NAS presenteren.
- **Minimale maatregel:** integrator laat historische beweringen herkenbaar en verwijst bij algemene status naar actuele brongebonden deelbewijzen; geen historische uitslag naar deze auditbron promoveren.
- **Menselijke beoordeling:** productteksten en beperkinglabels op duidelijkheid laten beoordelen.

### P06-10 — ONBEKEND — export, brede retentie en afwezigheid van OS-telemetrie niet integraal bewezen

- **Bron/doel/ontvanger/retentie:** alle stromen, vooral zoekindexen, AI-geschiedenis, crashdata, herstelmedia en toekomstige provideraccounts.
- **Bewijs:** contractuele scheiding bestaat; volledige native componenten en FR-B04/FR-P05/FR-P06/FR-L01–03 zijn open (`docs/FULL-TEST-RELEASE.md:210-228`). Deze audit is geen volledige code-/verkeersscan.
- **Product-/rechtsvraag:** welke overige persoonsgegevens ontstaan werkelijk, kan de gebruiker ze exporteren/wissen en geldt geen telemetrie ook tijdens installatie, periodieke taken en foutpaden?
- **Risico:** afwezigheid van een gevonden zender verwarren met bewezen geen verbinding of volledige dataminimalisatie.
- **Minimale maatregel:** componentinventaris en retentie-eigenaar aanvullen zodra implementaties bestaan; gecontroleerde native verkeer-/opslagobservatie over normale en foutlevenscyclus, plus export/herstelproef.
- **Menselijke beoordeling:** onafhankelijke technische beoordeling en waar nodig juridische beoordeling van doeleinden, rollen en bewaartermijnen.

## Bewijsmatrix: huidige bron versus historische uitvoering

| Eigenschap | Huidige statische bron | Historische uitkomst / grens | Niet bewezen op auditbron |
|---|---|---|---|
| Beschermende defaults, geen verplichte cloud | E1; privacycontract `:8-19,95-124` en ADR-035/036 | Niet door deze audit uitgevoerd | Alle native LP-scenario's, dagelijkse bruikbaarheid |
| Hostencryptie naam/inhoud, aparte admin | E2; `services/nas/crypto.mjs:44-95`, store `:275-311` | README `:57-79` beschrijft echte hostencryptie en kills; demo bevat beide rollen in hetzelfde proces | Native E2EE en fysieke/key-isolatie; geen nieuw hosttestresultaat |
| Lokale native NAS-revoke/replay | E3; core en lokale probe bestaan | E4/E5: `docs/verification/nas-replay-110/README.md:3-31`: twee runs, raw33, koude READ/COMMIT met exacte oude tokens `Stale`, 0 writes/flushes. Metadata noemt basis `e08fb0e4`; on-gecommitteerde bron gebonden via v3 `7255534f462eceba32f6d5059f2fc3b2219a8ea5e4ca18b1bca7626940ea648b` | Netwerkpeer, E2EE, gekoppelde apparaten, definitieve fysieke wis |
| NAS herhaalde gate | `.github/workflows/nas-core.yml:74-85` configureert native job | E6, alleen historisch gerapporteerd: NAS-replay-README `:33-47` noemt CI op `fa97c692` en integratie `562be4d`; latere lekcontrole is apart gewijzigd | Geen actuele run/CI-/artifactcontrole op `beafdedf` |
| Ciphertexthistorie opruimen | E3; native-volume-adaptercontract `:17-37` | Hostadapterbewijs, geen productwisclaim; historische lokale NAS-probe betreft smalle synthetische snapshot | COW-remanentie, externe snapshots, echte schijf/secure erase |
| Klembord minimaal en vluchtig | E3 plus protocolcontract | E4/E5/E6: `docs/verification/native-clipboard-20261102/README.md:13-56,58-89`: bron `d0ae570bd4666626096f92e57acef981c0f0d7b6`, finaletestbron `e3182f0bba72f300596dd81417fbbd371a8d5d79`, CI-merge `5754763ffb257d6420f74126cae7252e36afab1e`; gerapporteerde 22 fasen, clear/herstart en weigeringen | Alle apps/formaten, passwordvelden, lock/wissel en deze auditbron |
| Persoonlijk profiel en sleutelontgrendeling | E1/E3-interactiemodel, geen authenticerende productketen | E4/E5 smalle preview op `344dc0afde893784503217ac676e154b13662507`: persoonlijke synthetische poging geweigerd, expliciete openbare gast toegelaten | Echt account, multi-user, veilige vergrendeling en key-unlock |
| Sleutel-/IPC-bufferhygiëne | E3, begrensde Zeroizing-/Drop-implementaties | `docs/verification/B02-IPC-ZEROIZATION.md:25-56` rapporteert 12 native regressiegasten op `df1ea1d10e50345bb410c985e5994c12f699350b`, alle raw33 | Alle geheugenkopieën/registers/crashbeelden; menselijke audit/formele UB-evaluatie |
| Persistente key-state/antirollback | E3-codec, expliciet extern geheim en onafhankelijke minimumgeneratie | B02-specificatie `:3-7` meldt beperkte native persistentieslice, volledige acceptatie open | Productherstelagent, koude onafhankelijke versieankering en volledige rotatie |
| AI-context/downloadscheiding | E1; E2-modelreadercontract | Modelreader-README `:74-110` beschrijft synthetische hosttests, geen inferentie/QEMU | Native inferentie, helpernetwerkweigering, retentie/remove |
| Provideronafhankelijke ciphertextopslag | E1; EO-contract `:4,212-273` | Geen echte provideradapter-/accountproef in deze inventaris vastgesteld | EO-01–14, ontvanger-/credentialgrens, providerretentie |
| Privacydashboard/export | E2-demo, E1-producteis | Geen native integrale bediening of menselijke gebruikerstest vastgesteld | Actueel netwerkgebruik, rechtenoverzicht, export en wisstatus over alle lagen |

## Afhankelijkheden en ontbrekend/tegenstrijdig bewijs

1. **Identiteit en sessie:** native authenticatie-/herstelagent, per-gebruiker
   sleutelnamespace, lock/wissel en vertrouwde systeemdialoog. Het plan
   `PROFILES-AND-USERS.md:64-72` is een ontwerp-/historische lijst, geen
   actuele volledige code-inventaris.
2. **Netwerk en NAS:** authentieke caller-/issuerbinding, geauthenticeerde
   TLS-peer naar NAS-principal, duurzame grants/queue en onafhankelijke
   revisieankering. Een socketbestemming is geen geauthenticeerde NAS.
3. **Opslag en sleutels:** namen/manifests/indexen onder dezelfde object-
   en sleutelgrens als inhoud, write-ahead/key-state, herstel en rotatie;
   backup-/purge-/replaybeleid samen toetsen.
4. **UI en AI:** native privacydashboard met gezaghebbende dienstfeiten,
   opt-in, contextverwijdering, cancel/remove en gescheiden modeldownloader;
   daarna pas menselijke taak-/toestemmingsproeven.
5. **Acceptatie:** LP-01–31 en EO-scenario's zijn normatief, geen afgeronde
   testlijst. FR-P01–P06, FR-L01–L03 en FR-B04/B05 blijven open. Oude
   werkpakketnamen FR-B02 in sleutelonderzoek zijn niet de release-matrixrij
   FR-B02 voor bootauthenticiteit (`FULL-TEST-RELEASE.md:139-145`).

Het privacycontract sluit externe OS-telemetrie uit, maar staat bewust
toegestane apptelemetrie toe (`:15,114-124,160-162`). Dat zijn verschillende
principals, geen toestemming voor een algemeen OS-telemetriepad.
Providerprivacyprofielen met padding/batching zijn onderzoek, niet bestaande
metadata-afscherming (`ENCRYPTED-OBJECT-STORAGE.md:186-210`).
De versleutelde-objectstatus "nog niet native geïmplementeerd" betekent de
volledige productketen; native recordcode is niet daardoor afwezig.

## Eigen controles en uitvoeringsadministratie

Alle onderstaande commando's zijn werkelijk uitgevoerd in de eigen worktree.
Zij zijn bron-/administratieve controles, geen build-, hosttest- of VM-run.
De losse navigatiecommando's (`nl -ba ... | sed -n ...`, `rg`, `ls`, `cat`)
zijn uitsluitend gebruikt voor de hierboven geciteerde bronpassages. De
commandotranscriptie staat in de sessie; er is geen tweede logbestand in de
repository geschreven. Onderstaande lijst bevat de exacte beslissende
bron-/integriteitscontroles, niet iedere herhaalde regellezing.

### C01 — beginstatus en aanwezigheid broncommit — exit 0

```sh
git branch --show-current && git rev-parse HEAD && git status --short && git worktree list && git cat-file -t beafdedfc188154fe7285848fbd411e651013f54
```

Resultaat: beginbranch/HEAD zoals boven, leeg short-statusresultaat,
commitobject aanwezig. De andere worktreepaden uit de metadata zijn niet
geopend.

### C02 — toegestane bronselectie en instructielocaties — exit 0

```sh
git switch --detach beafdedfc188154fe7285848fbd411e651013f54 && git rev-parse HEAD && git status --short && find . -name AGENTS.md -not -path './.git/*' -not -path './target/*' -not -path './build/*'
```

Resultaat: juiste detached bron, schone werkboom; root-, docs-, services-,
UI-, kernel-, tests-, boot- en driversinstructies gelokaliseerd. Onder
`docs/audits/` werd geen extra AGENTS.md gevonden.

### C03 — lokale refs op broncommit — exit 0

```sh
git for-each-ref --format='%(refname) %(objectname)' --points-at HEAD
```

Als laatste deel van een leesaanroep uitgevoerd. Lokale bronbranch en haar
origin-trackingref wijzen naar `beafdedf`; dit is geen live remote-/PR-check.

### C04 — schone bron vóór rapport en concrete retentieplaatsen — exit 0

```sh
git log -1 --format='%H%n%s%n%cI' && git diff --quiet && git diff --cached --quiet && git status --porcelain=v1 && git ls-files docs/audits/subagents/06-privacy.md && rg -n 'fn delete|fn purge|pub const HISTORY|pub fn revoke|pub fn evict|sessionStorage.setItem|case .resetDemo|context: .obj-001|retention-days: 14' services/volume/src/{lib.rs,service.rs} services/nas/native-core/src/lib.rs ui/prototype/{app.js,fixtures.js} .github/workflows/native-window-session.yml
```

Resultaat: juiste bron; geen tracked/staged wijziging en rapport nog niet
tracked. Matches voor historie, delete/purge, revoke/evict, browseropslag,
reset en 14 dagen. Het zoekpatroon voor voorgekozen context leverde zelf geen
match; die constatering komt uit de afzonderlijke regellezing van
`ui/prototype/fixtures.js:42`, niet uit deze zoekuitkomst.

### Onvolledige zoekaanroepen, niet als groen bewijs gebruikt

Drie verkennende aanroepen noemden niet-bestaande paden:
`verification/`, `services/nas/client.mjs` en
`.github/workflows/native-system-foundation.yml`. `rg` meldde die expliciet
als fout. De omringende pipelines/latere leescommando's eindigden met exit 0;
dat maakt het ontbrekende pad niet aanwezig of doorzocht. Relevante bronnen
zijn daarna onder de bestaande `docs/verification/`, `crypto.mjs` en de
bestaande NAS-/vensterworkflows gelezen. Te grote tooluitvoer is gericht
in kleinere regelbereiken herlezen; niet-zichtbare uitvoer is niet als
inhoudelijk bewijs gebruikt.

### C05 — afsluitende schrijfgrens — exit 0

Uitgevoerd en na de redactionele eindcorrectie herhaald:

```sh
git rev-parse HEAD && git status --short --untracked-files=all && git diff --name-only && git diff --cached --name-only && git diff --check && sha256sum docs/audits/subagents/06-privacy.md && wc -l docs/audits/subagents/06-privacy.md
```

Resultaat: HEAD gelijk aan de bevroren bron; alleen dit nieuwe rapport staat
als untracked in status; tracked/staged diffs zijn leeg.
De inhoudshash is een overdrachtshash van dit rapport, geen bewijs van
OS-functionaliteit. Het resultaat en de hash staan in de begeleidende
sessie-uitvoer, niet zelfreferentieel in het gehashte bestand.
Omdat `git diff --check` untracked inhoud niet beoordeelt, is aanvullend
de volgende read-only witruimtecontrole gebruikt:

```sh
git diff --no-index --check /dev/null docs/audits/subagents/06-privacy.md
```

Uitkomst: exit 1, zonder witruimtefoutmelding. De no-indexvergelijking ziet
het nieuwe bestand als verschil met `/dev/null`; de niet-nulstatus is
expliciet behouden en wordt niet als geslaagde test gerapporteerd.

## Niet uitgevoerd, aannames en resterend risico

- Geen `cargo`, Node-tests, `preflight.py`, `build.py`, QEMU, packetcapture,
  runtime-/wis-/export-/crashproef of volledige secret-/telemetriescan
  uitgevoerd. Geen testresultaat van `beafdedf` toegevoegd.
- Geen CI-run/API, artifactdownload, volledige historische SHA256SUMS-
  hercontrole, externe primaire specificatie of juridische bron geraadpleegd.
  Vermelde crypto-/bibliotheekeigenschappen zijn repositorykeuzes, geen
  nieuwe externe goedkeuring door deze audit.
- Geen hardwaretest, echte NAS/providerkoppeling, modeldownload, menselijke
  gebruikerstest, onafhankelijke menselijke audit of releasevalidatie.
- Bewaartermijnen van browserherstel, hostback-ups, Git-mirrors en
  providerlogs zijn onbekend. Noodzakelijke metadata is niet automatisch
  onschadelijk of anoniem; ook hashes en opaque ID's kunnen koppelbaar zijn.
- Aangenomen is uitsluitend het contractuele gebruik van synthetische
  fixtures. Dit rapport bevestigt niet dat alle repositorybestanden of
  toekomstige diagnosebundels zonder persoonlijke gegevens zijn.
- De onderzochte stukken zijn gericht geselecteerd; afwezigheid van een
  implementatie in deze inventaris is geen uitputtend bewijs dat nergens
  code bestaat. Waar een component zelf expliciet ontbrekende integratie
  benoemt, is die concrete grens wel als bevinding vastgelegd.
- Herbeoordeling door de integrator is nodig wanneer runtimecode, identiteit,
  sleutelcontext, ontvanger, retentie of bewijsbron verandert. Geen algemene
  uitspraak over veiligheid, AVG-conformiteit, productrijpheid of hardware.

## Overdracht

Volledig rapportpad:
`/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-friendly-pancake/docs/audits/subagents/06-privacy.md`.
Broncommit: `beafdedfc188154fe7285848fbd411e651013f54`, detached.
Geen commit/push/PR of gedeelde voortgangswijziging.

De primaire inhoudelijke blocker is P06-01: authentieke native
client/NAS/provider-/sleutelbinding ontbreekt als complete productketen.
P06-02–05 blokkeren eveneens brede claims over persoonlijke lokale data,
verwijdering, privacybediening en AI. Lokale NAS-replay en klembord hebben
wel historisch afgebakend native bewijs en mogen niet als geheel ontbrekend
worden voorgesteld. De inventarisopdracht zelf had geen uitvoeringsblocker;
de genoemde grenzen zijn product-/bewijsblockers voor de integrator.
