# AUDIT-11 - betrouwbaarheid, gegevensbehoud en herstel

**Bronmetadata:** primaire repositorybron: *Nieuw OS — “Add OS specialist
agent profiles”*, auteur/organisatie S.W.J.B.M. van Ieperen, 3 november 2026,
commit `beafdedfc188154fe7285848fbd411e651013f54`
([onveranderlijke snapshot](https://github.com/simonvanieperen/nieuw-os/tree/beafdedfc188154fe7285848fbd411e651013f54)).
Alle lokale `pad:regel`-citaten hieronder zijn repository-observaties uit die
snapshot. Claims over externe durability-, hardware- en recoverycontracten
zijn niet onafhankelijk geverifieerd en blijven `ONBEKEND`.

Datum: 3 november 2026, Europe/Amsterdam. Status: afgeronde, begrensde
repository-audit; geen productacceptatie of onafhankelijke menselijke audit.

## Opdracht, bron en bevoegdheid

Letterlijke opdracht: **"Beoordeel crashes, stroomuitval, opslagcorruptie, volle
resources, mislukte updates, backup, restore en recovery."**

- Repository: `simonvanieperen/nieuw-os`.
- Uitsluitende auditbron: `beafdedfc188154fe7285848fbd411e651013f54`,
  lokaal aanwezig op `copilot/add-os-specialist-agents`.
- Eigen worktree; lokaal pad niet gepubliceerd. Niet gewerkt in de
  hoofdcheckout.
- Begin: branch `simonvanieperen-redesigned-engine`, HEAD
  `69809c2cb53dfb3bd77abe5352892810fba6e82d`, schone beginboom.
  `git cat-file -t` bevestigde de vereiste commit. Daarna conform toestemming
  `git switch --detach beafdedfc188154fe7285848fbd411e651013f54`.
  **Afwijking van letterlijke branchgelijkheid:** detached HEAD, geen switch
  naar of wijziging van de elders uitgecheckte hoofdbranch. Alle inhoudelijke
  conclusies hieronder betreffen uitsluitend de exacte auditcommit.
- Gewone chatsessie. `.github/agents/os-reliability-recovery.agent.md:9-28`
  is expliciet gelezen als rol-/scenario-instructie, niet technisch geladen
  als customagent. Geen subagents gebruikt.
- Gelezen instructies: root `AGENTS.md`, `docs/AGENTS.md`,
  `boot/AGENTS.md`, `kernel/AGENTS.md`, `services/AGENTS.md`,
  `drivers/AGENTS.md`, `tests/AGENTS.md`; geen dieper docs-AGENTS gevonden.
  Ook README, actueel PROJECT-STATE-checkpoint, privacycontract,
  FULL-TEST-RELEASE, architectuur, beveiligingsmodel, teststrategie,
  relevante ADR's en implementatie-/bewijsbestanden gelezen.
- Alleen dit rapport mag wijzigen. Geen productcode, voortgangsmeter,
  projectstatus, fixes, commits, pushes, PR's, externe diensten, downloads,
  installaties, credentials, echte gegevens of hardwarehandelingen.
  CodeRabbit niet uitgevoerd: externe codeverwerking valt buiten deze opdracht.

## Uitkomst en bewijsladder

Er bestaan substantiële native deelroutes voor copy-on-write-opslag, koude
read-only recovery, compositorherstel, resource-uitputting, sleutelstatus en
getekende updates. Ze zijn geen aangetoond herstelproduct op één gecombineerd
image. De belangrijkste concrete bronrisico's zijn vroegtijdige verhoging van
de bootversievloer en terminale uitval bij andere sessiediensten dan de
compositor. Onafhankelijke back-up/restore, duurzame offline sync en volledig
sleutelherstel blijven product-/integratiegaten.

De volgende E0-E8-legenda is de **gedeelde integratieconventie van de
hoofdorchestrator**, ontvangen tijdens deze audit. Zij staat niet letterlijk
in het reliability-profiel. Niveaus gelden per eigenschap, bron en profiel,
niet als algemene OS-score:

| Niveau | Betekenis |
|---|---|
| E0 | Ontbrekend of alleen ambitie |
| E1 | Ontwerp/documentatie |
| E2 | Hostprototype, simulatie of hosttest |
| E3 | Native broncode |
| E4 | Native positief testbewijs |
| E5 | Native negatief testbewijs |
| E6 | Herhaalde CI-/releasegate voor benoemde scope |
| E7 | Gedocumenteerde fysieke hardwaretest |
| E8 | Onafhankelijke menselijke audit/formele evaluatie |

Op de audit-HEAD zijn hier alleen broninspectie en de hieronder vermelde
E2-controles uitgevoerd. Historische E4-E6 blijven historisch. Een workflow
definiëren is geen uitgevoerde CI-run; hashcontrole is geen VM-heruitvoering;
QEMU is geen E7 en deze AI-audit is geen E8. Releasevalidatie wordt evenmin
afgeleid uit een E6-deelgate.

## Aangetroffen dekking, inclusief positief bewijs

| Onderwerp | Actuele bron en historisch bewijs | Bewijsgrens |
|---|---|---|
| Opslagcommit en onzekerheid | `services/volume/src/service.rs:795-841`: tabel naar oudere slot, FLUSH voor live commit; fout maakt instantie `Unknown`, zonder automatische retry. `:142-153`: schrijfbare recovery flusht gekozen kopie vóór gebruik. `docs/verification/S06-ACCEPTANCE.md:56-78` bewaart negen VM-stappen, inclusief drie echte kill/recovery-paren, op ce181bf/bronfingerprint 528dfe4c. | E3 nu; historische E4/E5. Geen huidige native replay, fysiek stroomverlies of algemene schijfreparatie. |
| Corruptie | `services/volume/src/table.rs:806-853` kiest alleen toegestane tabelrelaties; `services/volume/src/service.rs:902-925` controleert chunks en padding. Historisch corruptiegeval faalt vóór READY: `docs/verification/S06-ACCEPTANCE.md:58-68`. | E3 plus historische E5; FNV is geen cryptografische authenticatie. Bekend verliesgeval staat apart in RR-05. |
| I/O-fouten en read-only herstel | `docs/verification/S08-ACCEPTANCE.md:14-31`: WRITE/FLUSH-fouten, time-outs, resetquarantaine, vol volume en corruptie; dezelfde image daadwerkelijk `readonly=on`, zonder write-/flushcapability. `docs/verification/native-recovery-hardening/README.md:34-65` bewaart 27 geïntegreerde historische VM-runs op 53e0d57. | Historische E4/E5 en gerapporteerde CI-herhaling/E6, niet audit-HEAD. M6c/S08 en volume-v2 zijn verschillende profielen. |
| Compositorcrash | `sdk/examples/window-session/src/recovery.rs:50-83` vereist pause, werkelijk geïnstalleerde verse groep en rebind-ACKs. `docs/verification/mature-session-20261102/U01-ACCEPTANCE.md:10-27` bewaart echte ud2, behoud van beide apps/draft, save/reopen, bytes en cleanup op 337f2f29. | E3; historische E4/E5/E6 voor compositor. Geen bewijs voor iedere driver-/servicedood, zie RR-02. |
| Volle resources | `tools/test_resource_pressure.py:47-112` omvat IPC-/handlequota en twee qsim-processen met private 64-KiB-arena's, deadlines en cleanup. `docs/verification/K07-ACCEPTANCE.md:9-34` bewaart brongebonden CI. Volume-Quota/Full/hergebruik: `docs/verification/M6E-ACCEPTANCE.md:63-75`. | E3 en historische E4-E6, afgebakende quota; geen algemene kernel-OOM-, SMP-, 24-uurs- of herstelduurclaim. |
| Getekende updates en TEST-policy | `crates/update-core/src/test_policy.rs:4-44` pint huidige, next, revoked en expired TEST-paren. `docs/verification/B03-KEY-POLICY-1.md:200-213` bewaart echte TEST-naar-TEST-next-installatie en koude boot, plus dubbele weigeringen. `:5-17` rapporteert opnieuw 18/18 QEMU-stages op 69d5c3ac. | E3 en historische E4/E5; historische herhalingen blijven brongebonden. **Rotatie is niet ontbrekend.** Productieceremonie, dynamische policydistributie en monotone policyopslag zijn apart open (`:252-258`). |
| Sleutelstatus | `docs/implementation/B02-SECURE-RECORDS.md:304-325` beschrijft native export naar object 8 en koude restore, verkeerde sleutel/vloer/tag en behouden intrekking; bewijs onder `docs/verification/full-release-b02-keystate-system/`. | E3 en historische E4/E5-slice, niet volledige B02. RR-04 beschrijft de expliciete integratiegrenzen. |
| Historie versus back-up | `docs/verification/S06-ACCEPTANCE.md:91-95`: RESTORE als nieuwe revisie en undelete bestaan native. `services/volume/tests/independent_copy.rs:1-9,50-111`: host-only onafhankelijke volumekopie overleeft verlies van bronobjecten/-disks in het hostmodel. | Historische E4/E5 voor lokale historie; E2-bron voor onafhankelijke kopie. Lokale historie is geen onafhankelijke back-up. Rusttests hier niet opnieuw uitgevoerd. |
| NAS-herstel | `services/nas/native-client/README.md:27-45` bewaart edits in RAM, dwingt reconciliatie af en weigert automatische replay. `services/nas/native-probe/CONTRACT.md:3-7,65-76,102-107` scheidt lokale native probe van geauthenticeerde externe NAS/E2EE. | E2 clientmodel en E3 native probe; geen volledige duurzame offline sync of disaster-recoveryclaim. |

## Bevindingen en risico's

Prioriteit hieronder is een herstelprioriteit: P1 = vóór gebruikersgegevens of
dagelijks gebruik oplossen/bewijzen; P2 = vóór brede herstelacceptatie.
Dit is geen lijst van bewezen exploits. Vertrouwen betreft de bronvaststelling,
niet de kans dat het scenario optreedt.

### RR-01 - P1, hoog: geldige nieuwe kernel kan vorige werkende boot uitsluiten

**Broninferentie, vertrouwen hoog.** In
`boot/loader/src/signed.rs:327-341` stijgt de persistente versievloer na
image-/ELFvalidatie maar vóór terugkeer naar de hoofdloader.
`boot/loader/src/main.rs:518-539,645-649` laadt daarna pas de kernel;
de daadwerkelijke sprong volgt op `:487-495`. De selector weigert ieder slot
onder de vloer (`crates/update-core/src/lib.rs:393-416`).

Tegenvoorbeeld: A/v1 werkte, B/v2 is geldig ondertekend en laadbaar maar
panickt of hangt na overdracht. B verhoogt eerst de vloer naar 2. A/v1 is
bij volgende boot niet meer toegestaan; B kan opnieuw gekozen worden.
Er is in dit pad geen succesvol-draait-bevestiging of pogingenteller die een
geautoriseerde herstelkeuze mogelijk maakt. Zelfs een latere loaderfout vóór
kernelentry kan na de vloerverhoging vallen. Dit is niet hetzelfde als de
gedekte onderbreking tijdens installatie van bytes naar de inactieve slot.

**Impact:** geen aangetoonde automatische terugkeer naar een werkende versie;
herstelduur onbegrensd zolang geen bruikbaar toegestaan image wordt geleverd.
Geen runtimeproef van dit tegenvoorbeeld uitgevoerd. De 18 historische
TEST-policystages blijven geldig binnen hun scope.

**Afhankelijkheid/acceptatie:** FR-B03/FR-K01; ontwerp voor proefboot,
gezondheidsbevestiging en geautoriseerde recovery dat antirollback behoudt.
Niet de vloer uitzetten of een willekeurige downgrade toestaan. Benodigd
deelbewijs: geldig getekende maar crashende/hangende nieuwe kernel,
behouden werkende herstelroute, dezelfde disk/NV-state en behouden data.

### RR-02 - P1, hoog: compositorherstel is geen algemeen dienstherstel

**Bronvaststelling, vertrouwen hoog.**
`kernel/src/m2/m3/window_session.rs:594-615` staat compositor-exit/fault toe,
maar andere kritieke exits geven `CRITICAL_SERVICE_EXIT` en faults buiten
COMP geven `CRITICAL_SERVICE_FAULT`. `:101-110` beëindigt via
`serial::exit(0x11)`; `kernel/src/serial.rs:55-64` eindigt anders in cli/hlt.
De roles KEY/VOLUME/SOURCE/POLICY staan op
`kernel/src/m2/m3/window_session.rs:14-22`.

Het restartpad vereist bovendien levende apps én inputbron/-policy
(`:479-489`); init bewaakt hier de compositor-Process-handle
(`sdk/examples/window-session/src/init.rs:185-204`).
Een afzonderlijke input-, sleutel- of volumedienstfault wordt dus niet door
dezelfde succesvolle compositorroute afgehandeld.

**Impact:** sessie/system stopt, ondanks procesisolatie; onopgeslagen
appgeheugen is niet duurzaam. Dit kan binnen het smalle huidige profiel een
bewuste fail-closed-grens zijn, maar voldoet niet aan algemeen herstel waarbij
overige apps bruikbaar blijven. Geen nieuwe faultinjectie uitgevoerd.

**Afhankelijkheid/acceptatie:** FR-O04/FR-H02/FR-K02; userspace
afhankelijkheden-/herstartbeleid, epochs, app-pause, blok/RNG-reset/quarantaine.
Per dienst een eigen fault, hang en herhaalde uitval toetsen; geen brede
kernelrechten toevoegen. Behoud het bestaande compositorbewijs.

### RR-03 - P1, hoog: onafhankelijke gebruikersback-up/restore niet aangetoond

**Product-/integratiegat, vertrouwen hoog binnen gelezen scope.**
`docs/progress.json:186-191` houdt S09 open. De host-only kopie is echt
aanwezig (`services/volume/tests/independent_copy.rs:1,50-111`) en leest eerst
de bron volledig vóór een bestemmingswrite. Native geschiedenis/undelete is
eveneens aanwezig. Geen van beide bewijst een door een gebruiker gemaakte,
geverifieerde, onafhankelijke herstelkopie met native restore na bronverlies.

`docs/FULL-TEST-RELEASE.md:203` vereist juist native import/export/restore met
corrupt/incompatibel formaat, naamconflict, vol doel, onderbreking en bronverlies.
Deze audit heeft geen bewijs daarvan gevonden in de onderzochte kopie-,
volume-, Materiaal- en NAS-routes; dit is geen uitputtend afwezigheidsbewijs
voor ieder repositorybestand.

**Impact:** lokale historie en dezelfde disk verliezen samen hun waarde bij
bron-/diskverlies. Zonder geslaagde onafhankelijke kopie is het verliesbudget
de volledige uitsluitend lokaal bewaarde dataset, niet alleen de laatste edit.

**Afhankelijkheid/acceptatie:** S09/FR-O06, formatspecificatie, exportportaal,
gescheiden doel en credentials, sleutelherstel en byte-oracle. Hergebruik het
bestaande kopiemodel; bewijs daarna native restore op een leeg nieuw doel
zonder bron, inclusief vol/corrupt/afgebroken doel en gecontroleerde conflicten.

### RR-04 - P1, hoog: sleutelstatusherstel heeft nog geen volledig duurzaam levenscycluscontract

**Gedeeltelijk geïmplementeerd; integratiegat, vertrouwen hoog.**
`sdk/examples/keyring/src/service.rs:499-501,524-526` antwoordt rechtstreeks op
GENERATE/REVOKE in de levende keyring. De bestaande native slice exporteert
later via init en bewaart de snapshot vóór het specifieke document
(`docs/implementation/B02-SECURE-RECORDS.md:304-317`).
Dezelfde specificatie noemt write-ahead per mutatie en de vertrouwde koude
generatievloer uitdrukkelijk onbewezen (`:326-330`).

`services/keyring/src/persistent.rs:111-156` authenticeert snapshots en
controleert een meegegeven minimumgeneratie. Die ondergrens moet onafhankelijk
vertrouwd zijn; de gedocumenteerde koude proef gebruikt 0. De echte
gebruikers-/herstelagent, per-gebruikerssleutels en wachtwoord-KDF ontbreken nog
volgens `docs/implementation/PROFILES-AND-USERS.md:64-71`.

**Impact:** uit de beperkte proef volgt geen garantie dat iedere bevestigde
allocatie/intrekking na ieder crashpunt behouden is, noch dat een oud geldig
snapshot na koude herstart wordt geweigerd. Verlies van het externe geheim
zonder tweede herstelkopie kan inhoud onherstelbaar maken. Geen claim dat
bestaande AEAD-authenticatie of levende tombstones ontbreekt.

**Afhankelijkheid/acceptatie:** B02/FR-O05/FR-S02, externe-secretvoorziening,
vertrouwd rollbackanker en transactionele write-ahead. Crash vóór/na iedere
keymutatie-ACK, verkeerd geheim, oud snapshot en herstel op nieuwe instantie
native toetsen. Geen escrow of herstel door authenticatie uit te schakelen.

### RR-05 - P2, middel: beschadigde nieuwste tabel kan laatste bevestigde commit verliezen

**Bestaand, expliciet vastgelegd ontwerprisico; vertrouwen hoog.**
`services/volume/tests/service.rs:397-407` legt vast dat één ongeldige sector in
de nieuwste kopie niet te onderscheiden is van een onderbroken rewrite:
recovery kiest generatie 1 in plaats van 2. De tabelkeuze volgt dit beleid
(`services/volume/src/table.rs:806-817`). Het oorspronkelijke contract noemt
het verlies van de laatste commit expliciet
(`docs/implementation/M6E-SCALABLE-VOLUME.md:94-99`).

Dit is niet hetzelfde als een volledig verzegelde maar niet-canonieke tabel:
die wordt juist geweigerd, niet stil teruggevallen. Evenmin is het gewone
power-cutbewijs fout: dat beschermt een andere foutklasse.

**Impact:** bij deze corruptieklasse kan een eerder bevestigde commit verloren
gaan; een commit kan een transactie met meerdere objecten omvatten. Bij
meervoudige inhouds-/tabelcorruptie is geen algemeen verliesmaximum aangetoond.
Chunkchecks detecteren schade, maar herstellen geen ontbrekende bytes.

**Afhankelijkheid/acceptatie:** FR-O04/S09/B02; productbesluit over
redundantie/controlepunten en zichtbare herstelstatus. Vereist oraclebewijs
met de laatst bevestigde generatie, beschadigde kopie en exacte verliesmelding.
Alleen readonly openen is geen garantie dat de nieuwste ACK wordt teruggevonden.

### RR-06 - P1, hoog: offline NAS-queue is nog geen duurzame lokale herstelkopie

**Product-/integratiegat, vertrouwen hoog.**
`services/nas/native-client/README.md:27-45` zegt expliciet dat Checkpoint
in-memory is, geen duurzaam schema of save-ACK, en dat restore eerst
`NeedsReconcile` oplevert. Conflicten en Unknown blokkeren replay terecht.
De open afhankelijkheden staan op `:58-78`: duurzame queue/versieankering,
echte peerbinding, key-lifecycle en native clientintegratie.

**Impact:** uit RAMqueue-tests volgt geen behoud van lokale offline wijzigingen
bij proces-/machineverlies; exacte verliesgrens hangt af van een nog te koppelen
duurzame lokale documentbron. Geen reden om bestaande NAS-serverpersistentie
of de lokale native probe als ontbrekend te presenteren.

**Afhankelijkheid/acceptatie:** FR-P02/FR-P03/FR-O06. Lokaal duurzaam opslaan
vóór een relevante gebruikersbevestiging, daarna herstart/reconcile met
dezelfde peeridentiteit. NAS/provider offline, ACK-verlies, intrekking en
conflict mogen geen andere provider of automatische overschrijving activeren
(`docs/LOCAL-PRIVACY-CONTRACT.md:14-18,54-68`).

### RR-07 - P2, middel: herstelduur en bruikbare configuratie-/afsluitroute niet geaccepteerd

**Bewijsgat, vertrouwen hoog.**
`docs/FULL-TEST-RELEASE.md:164-167` houdt instellingen, shutdown met open werk en
energieherstel open; `:230-233` houdt de gecombineerde sessie, honderd koude
starts, 24 uur duurgebruik en zelfstandige fysieke recovery open.
`docs/implementation/PROFILES-AND-USERS.md:231-235` vraagt native
onderbrekingsproeven voor inrichting, instellingen en sleutelherstel.

Er zijn wel beperkte deadlines en metingen: compositorherstel wordt begrensd
op 6000 ticks (`sdk/examples/window-session/src/init.rs:256-258`), terwijl
historische koude volumerecovery 3,8-41,1 s duurde in de genoemde kleine
S06-fixtures (`docs/verification/S06-ACCEPTANCE.md:58-68`).
Dit zijn geen gemeten p95-hersteldoelen voor VM-FR of een grote dataset.

**Impact:** onduidelijk wanneer een consument weer kan werken en hoeveel
onopgeslagen werk bij config-/shutdownfouten behouden blijft.
**Afhankelijkheid/acceptatie:** FR-S03/S04/R01/R02, echte instellingenopslag,
save/flush/cancelcontract en vooraf vastgelegde meetworkload. Geen oude
TCG-smoketijd als product-SLA presenteren.

## Scenarioregister: vereisten voor vervolgacceptatie

Deze tabel definieert per profielscenario het controlecontract. **Injecties zijn
voorstellen, niet in deze audit uitgevoerd.** Alleen synthetische wegwerpimages
en geïsoleerde VM's; geen fysieke power cuts, hostdisks of echte gebruikersdata.
RPO = maximaal gegevensverlies; RTO = tijd tot bruikbaar herstel. Waar geen
geaccepteerde productgrens bestaat, staat dat expliciet.

| Scenario | Invariant en verwacht gedrag | Veilige injectie | RPO / verliesbudget | RTO / herstelduur | Observatiepunt en onafhankelijke controle |
|---|---|---|---|---|---|
| Compositorcrash | Apps/draft behouden, oude kanalen ongeldig, verse groep pas na pause/rebind. | Bestaande ud2-probe; later herhaalde crashes en vastgelopen compositor apart. | 0 appdraftverlies bij alleen compositorfault; geen belofte bij hele VM-kill. | 6000 ticks is codegrens; product-p95 open. | Echte PIDs/epochs, appinhoud/pixels, save/reopen en externe diskdecoder; historisch deelbewijs aanwezig. |
| Inputdriver, policy of andere dienst dood | Isolatie behouden, andere apps blijven reageren of eerlijke beperkte herstelstand. | Elke rol afzonderlijk laten exit/fault/hangen in fixture. | Bevestigd diskwerk behouden; RAMdraftbudget open. | Open; huidige kritieke fout is terminaal (RR-02). | Seriële rol/fout, appvoortgang, capabilities en framebaseline; onafhankelijke runner eist geen valse reboot-/restartclaim. |
| Kernelpanic / boot failure | Geen ongecontroleerde voortzetting; toegestaan herstelimage zonder gegevenswissen. | Panic na eigen kernelentry, ontbrekend/corrupt image, geldige crashkernel. | Bevestigd werk behouden onder afgesproken opslagmodel; RAMverlies apart. | Open; RR-01 kan onbeperkt blokkeren. | Loader vloer/slot en kernelentry, dezelfde disk/NV-bytes; onafhankelijke slot- en datacontrole. |
| Abrupte VM-stop / power loss | Geen ACK voor onduurzame mutatie, prefix/atomische voor-of-na-staat. | Bestaande cuts tijdens data/tabel/transactie op wegwerpimage; fysieke power loss niet toegestaan. | Bij normale cut onder flushmodel: geen verlies van eerdere duurzame ACK; lopende mutatie kan oud/nieuw zijn. | Alleen historische kleine-fixturetijden bekend. | Nieuwe koude VM, identieke after/before-imagehash, orakel en readonly-hash na herstel. VM-kill is geen fysiek cachebewijs. |
| Metadata-/inhoudcorruptie | Geen stille verkeerde plaintext of formattering; fout/terugval exact benoemen. | Bitflip, gescheurde sector, verzegeld oncanoniek, chunkfout in kopie. | RR-05: laatste commit kan weg; bij meerdere fouten onbekend. | Detectie bij mount/lezen; bruikbaar herstel uit kopie nog open. | Tabelgeneratie, chunk/padding, foutcode; onafhankelijke decoder en laatste-ACK-register. |
| WRITE/FLUSH-fout, timeout, reset zonder ACK | Unknown is geen succes; geen replay; onbevestigde DMA niet hergebruiken. | Bestaande brokerfixtures, requestercrash/reset-ACK apart. | Onzekere mutatie oud/nieuw, eerdere ACK behouden onder model. | Devicequarantaine kan tot VM-stop duren; productherstel open. | Blokpublicaties/completions, nul calls na Unknown, peerprogressie en cold diskoracle. M6c-bewijs niet automatisch volume-v2. |
| Low-memory / CPU-druk | Expliciete limietfout, survivor blijft draaien, cleanup herstelt middelen. | Bestaande arena-/procesquota; globale frame-uitputting en gecombineerde druk apart. | Geen verlies van eerder opgeslagen werk; niet-opgeslagen toestand per app nog te contracteren. | Bestaande deadline/cancel; volledige product-RTO open. | Timer/voortgang, exacte fout, frame-/handlebaseline door native assertions en aparte runner. |
| Handle-/IPC-uitputting | EQUOTA/EAGAIN zonder rechtenlek, hergebruik na vrijgave. | test-m4-quotas en supervisorquota; later samen met GUI/recovery. | 0 duurzame datamutatie door afgewezen request. | Begrensde request; productresponsbudget open. | Queueaantal, send/receive, capability-invarianten en onafhankelijke tellercontrole. |
| Vol volume / quota | Full/Quota vóór publicatie; oud document intact, ruimte na vrijgave bruikbaar. | Vul synthetisch volume plus destination, inclusief retained historie. | 0 eerdere ACK verloren door weigering; nieuwe save niet als opgeslagen melden. | Fout per aanvraag; hervatten na vrije ruimte, producttijd open. | Writes vóór/na weigering, stat/revisie en diskoracle; herstelactie moet zelf kunnen falen zonder valse bevestiging. |
| NAS/netwerk weg / ACK kwijt | Lokaal bruikbaar, edit duurzaam voordat beloofd, dezelfde peer, reconciliatie zonder replay/fallback. | Stop alleen testpeer, verbreek testtransport, verlies ACK vóór/na commit. | Duurzame lokale kopie beoogd 0; huidige RAMqueue alleen geeft geen grens (RR-06). | Retrybeleid/product-RTO open, offline lokaal verderwerken vereist. | Persistente queue en revisies, serverbytes, autoriteitsgeneraties; onafhankelijke tegenpartij-/diskcontrole, niet een verified-boolean. |
| Verkeerd geheim / key-unseal / sleutelverlies | Auth-fout zonder plaintext of namespacewijziging; geen escrow/fail-open. | Verkeerd geheim, tag/volume gewijzigd, oude snapshot, herstelgeheim ontbreekt. | Bij verlies van alle sleutelkopieën kan alle betreffende inhoud verloren zijn; bevestigde intrekkingen moeten behouden blijven. | Auth-fout begrensd; herstel zonder sleutel niet beloofd. | Native fout en nul body, exacte key-state, cold oracle en herstel via onafhankelijke kopie; RR-04 open. |
| Onderbroken update / rollback | Actief werkend image/data intact; boot alleen geautoriseerd; geen stille downgrade. | Cuts bij image, manifest, pointer, elke FLUSH en NV-update; geldige crashkernel apart. | 0 gebruikersdataverlies als productdoel; huidige updatefixture bewijst geen datamigratie. | Historische updategates bestaan, algemene recovery-RTO open. | Zelfde slotdisk/NV-state, beide signatures, vloer, werkelijk werkende kernel en gebruikersdata-oracle. TEST-next-gate behouden. |
| Defecte configuratie / onderbroken inrichting / shutdown | Laatste geldige configuratie, geen half profiel, cancel/flushfout eerlijk. | Verkeerd schema, configbitflip, volle opslag, kill vóór/na configcommit of shutdownflush. | Geen verlies bevestigde instellingen; draft-/shutdownbudget nog niet geaccepteerd. | Open. | Koude VM, echte instellingen/UI, generatie-/bytecontrole; FR-S01/S03/S04. |
| Backup / restore / disaster recovery | Onafhankelijke kopie verifiëren en zonder bron terugzetten; bron niet overschrijven bij fout/conflict. | Bron uitsluitend uit testopstelling verwijderen; leeg nieuw doel, kapotte/afgebroken kopie, vol doel, verloren sleutel. | Alleen laatst geverifieerde onafhankelijke kopie; interval/retentie niet vastgesteld. | Open; hostmodel is geen native producttijd. | Gescheiden disk/keykopie, onafhankelijke hashes en formaatdecoder, native readback zonder bron; gebruikerstaak en schone tweede omgeving apart. |

De injecties delen interfaces met boot/update, volume/block, keyring,
session/input en NAS/transport. Eerst hun herstel-ACK-, sleutel- en
autoriteitscontracten vastleggen; geen testtool die een ontbrekende OS-functie
op de host uitvoert. Volledige FR-criteria blijven open
(`docs/FULL-TEST-RELEASE.md:129,201-204,226-233`).

## In deze audit daadwerkelijk uitgevoerde controles

Alleen repository-reads en hieronder vermelde host-/integriteitscontroles.
Volledige interactieve tooluitvoer zit in het sessietranscript; er is geen
apart commandolog of tweede rapport in de repository gemaakt.

| Commando / controle | Exitstatus en resultaat |
|---|---|
| `pwd && git branch --show-current && git rev-parse HEAD && git status --short && git worktree list --porcelain && git cat-file -t beafdedfc188154fe7285848fbd411e651013f54` | 0; eigen geïsoleerde worktree, beginbranch/HEAD hierboven, status leeg, commit aanwezig. Worktreelijst alleen metadata; geen bestanden uit andere checkouts gelezen. |
| `git switch --detach beafdedfc188154fe7285848fbd411e651013f54` | 0; exacte audit-HEAD, opnieuw lege status. |
| `git show -s --format='%H%n%P%n%s' HEAD && git branch --points-at HEAD && git status --porcelain=v1 --untracked-files=all` | 0; parent 69809c2c, vereiste bron, lokale bronbranch aanwezig, schone boom vóór rapport. |
| Bronlezen met `view`, `nl -ba`, `sed -n`, `rg -n` en `git ls-files` | Alleen reads; relevante exacte pad/regelcitaten hierboven. Eén samengestelde bronzoekopdracht eindigde 1 omdat het filter `storage\|volume\|supervisor` onder `services/storage/*`/`crates/*/src/*` geen pad vond; werkelijk volume staat onder `services/volume/`. Geen testfalen of bewijs van afwezigheid. Latere gerichte zoekopdrachten vonden de juiste bestanden. |
| Harnastestcommando hieronder | 0; **35 tests**, 0 failures/errors, unittest 0,562 s; totale scriptmeting 0,606 s. Alleen E2. |
| Archiefcontrole hieronder | 0; **56 B03-mainca45 + 29 S06 = 85 hashes** correct, 0,004 s. Alleen lokale integriteit van historisch bewijs; geen cryptografische manifestverificatie of native herhaling. |
| `git diff --quiet && git diff --cached --quiet && git status --porcelain=v1 --untracked-files=all` vóór schrijven | 0; geen bron- of gebruikerswijzigingen vóór rapport. |

Exact uitgevoerd hosttestcommando; synthetische tijdelijke fixtures staan in
door Python beheerde tijdelijke mappen, niet in andere repository-output:

```sh
python3 -B - <<'PY'
import time
import unittest
start = time.monotonic()
suite = unittest.TestSuite()
for pattern in ('test_volume_disk.py', 'test_update_harness.py', 'test_supervisor_harness.py'):
    suite.addTests(unittest.defaultTestLoader.discover('tests', pattern=pattern))
result = unittest.TextTestRunner(verbosity=1).run(suite)
print(f'AUDIT-11 host-harness: tests={result.testsRun} success={result.wasSuccessful()} seconds={time.monotonic()-start:.3f}')
raise SystemExit(0 if result.wasSuccessful() else 1)
PY
```

Exact uitgevoerde read-only archiefcontrole; manifestpaden worden vóór lezen
tot de genoemde bewijsroot beperkt:

```sh
python3 -B - <<'PY'
import hashlib
from pathlib import Path
import time
start = time.monotonic()
for folder in ('docs/verification/b03-key-policy-mainca45', 'docs/verification/s06'):
    root = Path(folder).resolve()
    count = 0
    for line in (root / 'SHA256SUMS').read_text().splitlines():
        digest, relative = line.split(maxsplit=1)
        relative = relative.removeprefix('*')
        path = (root / relative).resolve()
        if Path(relative).is_absolute() or not path.is_relative_to(root):
            raise ValueError(f'path outside evidence root: {relative}')
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != digest:
            raise ValueError(f'hash mismatch: {folder}/{relative}')
        count += 1
    print(f'{folder}: {count} hashes OK')
print(f'AUDIT-11 archive integrity seconds={time.monotonic()-start:.3f}; no native execution')
PY
```

De geselecteerde harnastests gebruiken gefabriceerde transcripten om het
**weigeren van fout bewijs** te testen; zij produceren geen echte bootlogs
of native succesbewijzen. De hashcontrole bewijst uitsluitend dat de
gecommitte archiefbytes bij hun manifest passen, niet dat iedere historische
interpretatie daarmee opnieuw bewezen is.

## Niet uitgevoerd, aannames en blockers

- **Niet uitgevoerd:** Rusttests/builds, preflight, QEMU/OVMF, VM-kills,
  nieuwe update-/herstel-/NAS-/keyringproeven, fysieke hardware/stroomuitval,
  live CI-/PR-inspectie, 24 uur duurtest, honderd starts, menselijke
  gebruikerstest, onafhankelijke menselijke audit/formele evaluatie of
  releasevalidatie. Geen externe service benaderd.
- Aanname bij interpretatie van historisch opslagbewijs: het beschreven
  single-CPU/QEMU/virtio-flushmodel; geen vertaling naar fysieke
  drivecache-, firmware-, DMA- of energiegaranties. De lokale OSDev-opslagroute
  is alleen als aanvullende leesindex bekeken, geen vervanging van de
  repo-contracten of extern gecontroleerde primaire specificatie.
- Onderzoek is substantieel maar begrensd tot de genoemde boot/update-,
  volume-, keyring-, resource-, window-session- en NAS-herstelpaden plus
  hun opgeslagen bewijzen. Geen volledige parser-, driver- of securityaudit.
- Audituitvoering had geen harde omgevingsblocker: commit beschikbaar,
  schone eigen worktree, veilige hostcontroles beschikbaar. De expliciete
  auditgrens verhinderde bewust native/releasevalidatie; dat is geen groene
  acceptatie en evenmin bewijs dat de native omgeving technisch ontbreekt.
- Productblockers: RR-01/02 voor algemeen boot-/dienstherstel; RR-03/04/06
  voor onafhankelijk data-/sleutel-/offlineherstel; RR-05/07 voor een
  overeengekomen verliesbudget, herstelduur en gebruikersroute. E7/E8 ontbreken
  voor deze onderzochte herstelketen. Productiekeybeheer blijft afzonderlijk
  van aanwezige TEST-policyrotatie en historische 18-stage-gates.
- Geen productscore herberekend of aangepast, conform uitsluitend-rapport-
  schrijfmandaat. Geen algemene veiligheids-, hardware- of productrijpheidsclaim.

## Oplevering

Enig schrijfbestand: `docs/audits/subagents/11-reliability-recovery.md`.
Exacte broncommit blijft `beafdedfc188154fe7285848fbd411e651013f54`.
De read-only rapportvalidator eindigde met exit 0: 44 geldige pad/regelcitaten,
E0-E8 aanwezig, zeven bevindingen, exacte HEAD en uitsluitend het toegestane
rapport als wijziging. Hij controleert bestaan en regelbereik; betekenis van
de citaten berust op de afzonderlijke bronlezing, niet op deze mechanische check.

`git diff --no-index --check /dev/null docs/audits/subagents/11-reliability-recovery.md`
eindigde met 1 zonder whitespace-diagnostiek: no-index rapporteert hier het
verschil met het lege bestand. Dit wordt niet als exit 0 gelabeld. De aanvullende
controle hieronder toetst expliciet trailing whitespace en eindnewline:

```sh
python3 -B - <<'PY'
from pathlib import Path
p = Path('docs/audits/subagents/11-reliability-recovery.md')
data = p.read_bytes()
assert data.endswith(b'\n')
assert not any(line.rstrip(b' \t') != line for line in data.splitlines())
print('AUDIT-11 whitespace: OK')
PY
```

Resultaat: exit 0. `git diff --check` voor de tracked boom: exit 0.
De rapport-SHA-256 wordt buiten dit bestand aan de opdrachtgever teruggegeven
om een zelfreferentiële hash te vermijden. Geen commit/push/PR of herstelwijziging.
