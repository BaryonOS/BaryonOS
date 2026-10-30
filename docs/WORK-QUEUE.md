# Werkqueue — volledige native testrelease, 30 oktober 2026

## Actueel mandaat- en claimregister — managementwijziging Simon, 30 oktober 2026

Dit register vervangt de **actuele werking** van alle oudere toewijzingen
hieronder; de historische inhoud en brongebonden bewijzen blijven behouden.
Managerbron `caa2c14da15c5ac82b5493b3b2407167163d00d9`, runtimebasis
`94c8908ee363d06a5be51417c5cae635549d566f`. Laatst door intake bevestigde
main is `ca45a7dabe6a040921de03bdc2fad3e5045a75ba`; finale kandidaatbasis
volgt van de integrator na #74. **34/100 blijft 34/100; alle 62 FR-rijen open.**
De volledige ROADMAP/FULL-TEST-RELEASE/CONTEXT en fysieke/menselijke gates
blijven bindend, inclusief honderd koude starts en 24 uur gecombineerd gebruik.

### Bevoegdheden en overdracht

**Overname voltooid op 30 oktober 2026, 19:18 Europe/Amsterdam.**
Donor `6e3bc3ff9922d947df8f5aa89a7bcd962da601fa`, gecontroleerd via
`git ls-remote`; uitsluitend queueblob `7c011de653103dc6614c90be50b61102c8d55af8`
is eerst bytegelijk overgenomen op schone basis `ca45a7dabe6a040921de03bdc2fad3e5045a75ba`.
Daarna onderhoudt uitsluitend **Teamleider werkqueue**,
app-ID `5d5f8513-2d31-41af-96ce-81c5ef56ef80`, dit register.
Hostcontext-ID `dcb34641-e6d7-483a-8b8b-21598205381f` is geen extra uitvoerder.
Geen donorstatus, scoreblad of runtimebron overgenomen. Teamleider89 bevestigt
dat zijn queuewrites gestopt zijn; onderstaande donorprotocoltekst is historie.

| Rol | Bevestigde identiteit | Bevoegdheid en schrijfgrens |
|---|---|---|
| Directeur Nieuw OS | `ef7a7596-8c3d-4705-ab69-36f8a2a1a1b6`, Directeur nieuw OS; eigen rolbevestiging ontvangen | Roadmap/kwaliteit en aansturing van beide teamleiders; geen extra taak-/PR-poort en geen centrale writes. Alleen richtingwijzigingen, gebiedsoverstijgende of met eigenaars onoplosbare besluiten met bewijs voorleggen |
| Operationeel teamleider | `89f84098-bd8c-4bc8-aeb2-0695517db26f` | Technische voortgang, afhankelijkheden, samenwerking en kandidaatkwaliteit. Blijft enige schrijver van PROJECT-STATE, progress.json, README, PROGRESS, progress.svg en eigen managerbewijs |
| Assistent-teamleider | `5d5f8513-2d31-41af-96ce-81c5ef56ef80`, Teamleider werkqueue; eigen rol-/schone-startbevestiging ontvangen | Operationele planning en **enige schrijver van `docs/WORK-QUEUE.md` na onderstaande bronvrijgave**. Mag gewone taken en daadwerkelijk vrije schrijfgebieden binnen vastgelegde prioriteiten zelfstandig toewijzen, testen en oplevering laten doorgaan; geen routinegoedkeuring van teamleider of Simon |
| PR-intake | `f2afe26e-5d6f-4948-89b8-0e2aeeb96225`, Pull requests binnenhalen | Behoudt oorspronkelijke PR-intake/routering/dossiers. Voorstel algemene werkverdeling ingetrokken; geen queuewriter. Feitelijke eerdere overdracht apart controleren en eventueel werk behouden |
| Integrator | `e8e471ce-0601-417b-b4fa-a04fd9c7b954` | Enige operationele integrator, conflictopname en guarded merges; branchregels blijven gelden |

**Donorprotocol:** teamleider rondt deze bestaande, nog ongepubliceerde
queuewijzigingen af in één persistente donorcommit, pusht en controleert
remote-SHA, en stuurt exacte commit plus expliciete vrijgave aan de assistent.
De assistent heeft bevestigd voordien geen queuewrites/commits te hebben en
tot die bronvrijgave niet te schrijven. Vanaf vrijgave schrijft de teamleider
de queue niet meer; ontvanger bevestigt bron/overname en onderhoudt daarna
alleen deze queue. Geen branch- of hele-bestandsovername van verouderde main.
Voor de oude PR-intakeoverdracht bevat het bewaarde register uitsluitend
PR-bevoegdheid. Aanvullende read-only checkoutcontrole: geen dirtyqueue en
laatste queuecommits zijn de gedeelde main-integraties94c8908/f52dab5/ad2b981,
geen eigen planningscommit gevonden. Persoonlijke bevestiging is opgevraagd;
stilte wordt niet als bewijs gebruikt en eventueel later geleverd werk blijft
behouden. Er is geen uitgevoerde exclusieve overdracht gevonden die eerst
moet worden teruggenomen.

**Continuïteit:** onderstaande bestaande taken en fileclaims blijven geldig
bij stilte, timeout of onbereikbaarheid. Binnen de vastgelegde opdracht mag
de eigenaar zelfstandig implementeren, testen en opleveren, zonder nieuw
manager-ACK. Nieuwe gewone verzoeken gaan naar de assistent. Een timeout
bewijst geen stop en maakt geen schrijfgebied vrij; overdracht vereist
expliciete vrijgave/ontvangst en behoud van onafgemaakt werk.
Nieuw bevoegdheids-/ABI-/trustbeleid eerst met betrokken eigenaars en
technisch teamleider oplossen; alleen de hierboven genoemde uitzonderingen
gaan naar de directeur. Simon is geen routinefallback.

**Technische prioriteit:** gewone interactieve DEV1 voor Windows/QEMU,
exact brongebonden bediening/opslag/herstel en eerlijke beperkingen;
parallel onafhankelijke ontwikkeling binnen bestaande claims.
Daarna concrete native blockers (TCP-CR3, issuer/provider/object-onlyREAD,
launch, NAS-lifecycle/persistentie) oplossen, geen tweede verifier/catalog/
crypto of fixture als productieprovider. Volledige roadmap/release blijft
bindend, niet vervangen door alleen de DEV-bundel.

Operationele integratie, basissein,
conflictopname en merges zijn **uitsluitend bij e8e** gedelegeerd. Release
beoordeelt de gecombineerde kandidaat en behoudt reeds toegewezen runtime-/ABI-
werk tot expliciete overdracht. Geen algemene domein- of kernelovername.
Een verdere centrale overdracht vereist opnieuw een expliciete exclusieve afspraak.

### Rollen en precies één primaire taak

**Nummering:** 1 teamleider, 2 beveiliging, 3 intake, 4 release, 5 AI/quantum,
6 netwerk, 7 review, 8 integratie, 9 hardware, 10 NAS, 11 apps/SDK,
12 build/test, 13 native UI, 14 native kern/opslag. De acht onderstaande
specialisten zijn door Simon geopende, via echte berichten geïdentificeerde
sessies; er is geen dubbele sessie gestart. Gebruik de app-routerings-ID's
in de tabel, niet een tweede interne project-/context-ID als extra agent.
Met de bevestigde directeur en assistent zijn er24 geregistreerde rollen;
dit is **geen** verklaring dat24 agents gelijktijdig uitvoeren.

| Rol / sessie | Primaire taak en actuele stand |
|---|---|
| Teamleider / `89f84098-bd8c-4bc8-aeb2-0695517db26f` | TASK-TL-HANDOFF: donorcheckpoint/technische samenhang en centraal scorebeheer; queue na bronvrijgave uitsluitend bij assistent, geen parallelle mergequeue |
| Assistent / `5d5f8513-2d31-41af-96ce-81c5ef56ef80` | TASK-ASSISTANT-QUEUE: donor6e3bc3f ontvangen en queueblob bytegelijk overgenomen op ca45; enige queuewriter. Planning/vrije gebieden zelfstandig, geen score-/status-/mergewrites |
| Directeur / `ef7a7596-8c3d-4705-ab69-36f8a2a1a1b6` | TASK-DIR-ROADMAP-QUALITY: richting/kwaliteit, alleen noodzakelijke beslisescalaties; rol bevestigd, geen extra bouwer/integrator of routinegoedkeuring |
| PR-intake / `f2afe26e-5d6f-4948-89b8-0e2aeeb96225` | TASK-PR-INTAKE: uitsluitend read-only dossiers per open PR en één primaire routering. #76 `8a14770d6a719c3a122c54cb774db58ce9b1a728` is overgedragen/frozen; geen producer-, rebase-, merge- of sluitingsclaim |
| Integratie / `e8e471ce-0601-417b-b4fa-a04fd9c7b954` | TASK-INTEGRATE-74: enige integrator. d7b7 had13groen/0threads; ready_for_review startte nieuwe verplichte CI, guarded squash correct geweigerd. Main blijftca45; alleen nieuwste check per context telt, geen oude allgreen overnemen |
| Release / `2ab17599-85c8-4409-ba06-a93c27485328` | TASK-REL-CANDIDATE: GEWONE INTERACTIEVE DEV1 gekozen op door8vrijgegevenca45, geenrootseambuildernu. Eigen offlinebuild/same-sourcepixelreferentie, daarna12ACK voor2seriële bestaande gates (referentie+7procespreview, max240s/elk), noggeenVM. Freeze bron/tool/imagehashes+NLbediening/stop/limits voor20/21. NAS#83/032 blijft apart bewijs; object-onlyREAD/provider/launch en CAPBENCHguard blijven rootdependencies |
| Reparatie/review / `7f895683-fc79-48d0-a98a-aa0a6ccfe8b9` | TASK-PR-REVIEW: R73-v2, R78 en R80 read-only afgerond zonder concrete mergeblokker; R68-v2 vindt twee rendererregressies, primaire reparateur UI. Wacht op diens fixfreeze voor begrensde herreview; geen eigen tests/native/audit/writes |
| Build/test / `b72f2536-b494-4a6b-a608-b6fd785e18ef` | TASK-RUNNER-POOL-HANDOFF: NAS1219-herreview af, Windowsroute formeel overgedragen aan21. Toolenv geleverd aan13/19/20/22; enige korte slot van4 aan19, daarna13 met headroom/4ACK. R77 FROZEN bffe11, geen coldtool-/packagewriter |
| Beveiliging / `081d6f31-7de1-4713-b466-d15adf51b2ef` | SEC72 af/frozen7f67471, df1-testhelper zonder proddelta,36host/releasewipe/12native/20locks groen gemeld. TASK-SEC-CRYPTO-AUTHORITY eindfreeze `c5fbb21a0e46e010d5e9b4748ae9b9307a67cef4`, één bestand; NET/NAS/REL-alignment af, geen runtimeprovider. B05menselijk intakepakket voorbereid, geen audit |
| Netwerk / `039dbef8-bc79-4480-b003-55b8eccba736` | Primair TASK-N01-NATIVE-CONFIG hieronder vrijgegeven; REL bevestigt geen actieve/gereserveerde/gedelegeerde Stack/configwrites. Broker #87/0962d02 frozen, eigen net-broker-quality SUCCESS gemeld op run41104644311, overige gates apart. Issuer/devicefence/TCP-CR3 blijven REL/SDK/HW-afhankelijkheden; geen nativeconfigclaim |
| NAS / `d7636e9c-fe84-4e63-8f37-4d08a5f50d0d` | Primair TASK-NAS-EXACT-REPLAY-PROBE payloadleaf hieronder vrijgegeven, rootuitvoering wacht REL-profilefreeze/handoff. RESUME-1 frozen19c85f34,16debug/17release en viercrategates gemeld; intakef2 gevraagd correcte draftleafhead/base te routeren, geen PR71mutatie. Typedcheckpoint geen duurzame queue; TLS/authority/lineage/persistentie open |
| Apps/SDK / `9db24a72-2ae7-4807-8e68-2fc88cb235d3` | Catalogusbc27/#80 en protocol `734c8fc587edffba746ef470212bb751e6537b29`/#84 af/frozen; #80 dedicatedquality werkelijk SUCCESS, overige gates apart. #84 op #80base na expliciete8-ghopdracht;11 blijft contract-/reviewafnemer viewer22, geen viewer-/SDK-/rootwriter |
| AI/quantum / `381d080e-be3d-4c78-ac72-b99b1de2b4e5` | TASK-L01-ROOT-ADAPTER af/frozen b51a8093, concept#82 op #78;4unit+13contract/lint/fmt/default+scalar-none en PR-CI groen door eigenaar gemeld. Geen provider/catalogus/nativeinference; ook #83 mist rootcontract/provider, dus geen integratieclaim. qsim-WIP behouden |
| Materiaal/toegankelijkheid / `764bc9fb-2036-4b13-bbff-368037f9b2d7` | TASK-UI-PRIMARY-HOVER opgeleverd/frozen29b79eae; eigenaar meldt rode oude renderer101,4pixel/24suite/lint/fmt/none groen, geenVM. Freeze naar7/15 voor review. IMAGE-0818 bewijsfreeze e37fa17b op0818/v326b41e55 blijft eigen bron, geen ordinaryDEV/assets/U-aftekening; wrapper89e geparkeerd |
| Native kern/opslag / `fbf60554-e98e-40bd-bd48-ee87a6498abe` | TL-VERIFY-SOURCE-RUNTIME eerst expliciet overdragen; laatst trace `79b956e`, nieuw uitvoeringsresultaat niet ontvangen. Concrete volgende K/S-opdracht TASK-KS-CR3-DIAG read-only verstuurd; S09-vervolg vanaf #76 daarna, nog geen producerfiles vrijgegeven |
| Hardware / `f5b20312-b6c5-46b0-bbf6-76970d3470a2` | Primair TASK-H03-TRANSPORT-MODEL-1 hieronder vrijgegeven. H05/#79, codec#85/72f en contract#88/823cba3 frozen; B1–B9-review14/2/4 nog apart, geen runtime/devicefencebewijs |
| 15 Productvormgeving/UX / `6117dd21-50d2-4418-b3b6-17048afd1180` | TASK-UX-SAVE-RECOVERY af/gepusht `d6b4e68380e910b0c9b560ffceb1470eb574d81e` na63af, twee spec/designbestanden,12UX-statechecks/bekeken hostrender en uitvoerbaar menselijk taakprotocol. Ontwerpaanspreekpunt15, wacht0818-runtimebeelden; geen native/humanresultaat |
| 16 Iconen/assets / `1bf8f308-d645-4632-b210-2258428f86b2` | TASK-ASSET-MATERIAL-FLOW af/frozen `d3547639be733aefcc1228edad37114980e5073c`; vier SVG-symbolen, XML/allowlist/hostrender24/48px licht/donker groen,15/13 dezelfde bron/controlebeeld beoordeeld. Geen native assetconsumptie |
| 17 OS-onderzoek/R&D / `ac0a0e5e-505e-4b77-b933-bcec2c636696` | TASK-RD-TLS-TIME af/frozen `4ffdde9f2762ff39de64eeca9940e8b3548d79e0`; één beslisnotitie/vijf primaire bronnen binnen90min, huidige fail-closed behouden. Aanspreekpunt17 voor18/19, geen zelf vastgestelde tijdbootstrap |
| 18 Invarianten/formele modellen / `018d2f82-0c7f-4f13-874e-5651c374ad50` | TASK-V18-TLS-TIME-FAILCLOSED af/frozen `60bab47377b45c478d59c47a57b313c12e25ce2a`;4tests,2380histories/6942calls en RTC-promotiemutant tegenvoorbeeld gemeld. Model-only, geen Rust-refinement/handshake/native bewijs; cap-model NIET actief |
| 19 Prestaties/betrouwbaarheid / `a398bc66-58a8-4ebe-9aa4-ed8ee2134fdb` | TASK-CAP-DIGEST-TCG-2 evidence `b715e362b04fcd65de79c8bf85bfa439add037eb`: preflight/build/18host groen, enige VMca45 raw35/runner1 CAPBENCHmanifestfail vóór meetcalls. Geen timings; slot vrij naar13. StaleB02-countguard hypothese, owner4/kernel; geen herhaling zonder nieuwebron/slot |
| 20 Onafhankelijke native QEMU-producttest / `1963283b-73f4-4520-83f3-e9cfb93b3693` | TASK-QEMU-PRODUCT-DEV1: vijf-casesprotocol gepusht `a49774beface0f332e9c7afc3931174eca64d2a5`, geen artifact/VM. Verwachte0818-v3/14ASCII-QMP-input onbevestigd tot run; stopcontroller apart. Vaste beperkte DEV mag vóór hele62FR; Full/Unknownfixtures ontbreken |
| 21 Windows/QEMU-aflevering / `d7cd7dda-ca14-4f03-8adb-275b5884acfb` | TASK-WINDOWS-DELIVERY-TAKEOVER: donor12 en21 wederzijds ACK, vijf bestaande routefiles exclusief overgedragen. Bewijs `0915828fcd40492f873dc9098ee445306f1f0638`:10host+5echteWindows-synthetische tests; nog geen nieuwe OS-kandidaat/ZIP/VM |
| 22 Extra native-appbouwer / `29d8407c-b3c5-4c7c-ba6b-c37ca39da332` | R22-IMAGE-VIEWER-LEAF-1 freeze concept#86 `f2c875a711f9e2d3d045de160163634bcbc8d80e`,code6b3fbf,15host/lint/fmt/scalar-none groen gemeld. Fit-fixregressie+afzonderlijkeAIherreview af, geen humaanaudit.11handoff gequeueëd na2timeouts; native objectREAD/launch/focus externblocked, geen volgend pakket |

### Afdelingen, aanspreekpunten en samenwerking

| Afdeling | Rollen | Aanspreekpunt en escalatie |
|---|---|---|
| Kern/hardware/netwerk | 14,9,6 | 14 (`fbf60554...`); zelf K/S-werk, geen algemene domeinovername |
| Beveiliging/NAS | 2,10 | 2 (`081d6f31...`); gedeelde crypto/identiteit/foutgrenzen, geen stille nieuwe autoriteit |
| Productontwerp/native UI | 15,16,13 | Ontwerp15 (`6117dd21...`);13 blijft code-/runtime-eigenaar,16 uitsluitend afgesproken assets |
| Apps/SDK/lokale intelligentie | 11,5,22 | 11 (`9db24a72...`); extra appbouwers alleen bij zelfstandige pakketten en passende review/integratiecapaciteit |
| Onderzoek/verificatie | 17,18,19 | 17 (`ac0a0e5e...`); concrete vraag/ronde/resultaat, geen routinebesluiten als centrale R&D-poort |
| Kwaliteit/aflevering | 4,12,20,21 | 4 (`2ab17599...`);12 plant zware runs,20 onafhankelijk gedrag,21 één Windowsroute |
| PR/integratie | 3,7,8 | 8 (`e8e471ce...`), uitsluitend operationeel integrator;3intake/7review read-only |

Aanspreekpunten blijven zelf ontwerpen/onderzoeken/bouwen/verifiëren.
Lees bij het eerstvolgende checkpoint deze samenwerking en de actuele
fileclaims; deel alleen nieuwe resultaten/bron/PR/bewijs, concrete blocker
met eigenaar en volgende uitvoerbare stap per afdeling. Details blijven
bij taak/PR. Nieuwe gewone taken/vrije gebieden gaan naar de assistent; technische
afhankelijkheidsconflicten naar manager1, alleen onoplosbare/gebiedsoverstijgende
of richtingwijzigende beslispunten naar de directeur.
Geen normale voortgangsberichten naar iedere rol tegelijk.

Werkstroom: ontwerp → implementeerbare specificatie/assets → native
implementatie → gerichte validatie/review → integratie op vaste kandidaat →
onafhankelijke productcontrole → aflevering met expliciete DEV-/acceptatiestatus.
Ontwikkelbranches mogen onafhankelijk doorgaan terwijl een vaste kandidaat
wordt beoordeeld. Eén passende geldige run mag verschillende beoordelaars
van bewijs voorzien; onafhankelijke herhaling blijft verplicht waar de
acceptatie die voorschrijft. Een testcontroller is geen gebruikersinvoer.

**Werkelijke middelen:** hostmomentopname32logischeCPU/30,8GiBRAM/
23,5GiBavailable/8GiBongebruikteswap/925GiBvrij, cgroupcpu/memory/pids
`max`; gedeelde omgeving, dus geen exclusieve resource- of platformgarantie.
12 heeft read-only toolenv/versies/firmwarehashes aan13/19/20/22 geleverd.
Het ene korte slot19 is daadwerkelijk één keer gebruikt en vrijgegeven;
4ACK voor opvolger13 gemeld,12 blijft headroom/registratie bewaken.
13 meldt één geslaagde probe en vrijgave;20 wacht definitief gedeeld artifact,
geen tweedeVM. Eigen targets/builds/disks/vars/swtpm-state.
Geen extra8GiBVMs of dubbele suites op basis van alleen dit getal.

**Aflevergrens:** volledige VM-FR/62FR/100cold/24uur/fysiek/menselijk blijven
open. Een bruikbare, expliciet beperkte Windows-DEV-build is wél een geldig
tussenresultaat, mits4/8 exacte bron/v3/image/kernel/loader/firmware/QEMU/
profiel/start-stop/limitations vrijgeven.13's0818-probe is alleen een
renderfixture en vervangt die gewone interactieve DEV-kandidaat niet.
Geen tweede launcher/packager en geen oude eabc-bundel herlabelen.
Windows11.1hostbewijs en Linux10.2.1bewijs apart houden.

Bereikbaarheid: directe prioriteitsbezorging aan8 faalde expliciet met
remote admission timeout; de inhoud is daarna bewust gequeueëd, geen
verwerking geclaimd.8 heeft inmiddels zijn exclusieve rol en hervatte
#74-verificatie terugbevestigd.13historische SUCCESS zijn ingehaald door
nieuwe ready_for_review-CI op dezelfde d7-head; guarded squash faalde correct
op strict host-quality/qemu-smoke. Definitieve main-/DEV-basis nog niet ontvangen.
14 is gericht benaderd, maar diens verifieroverdracht/CORE-ack blijft ontbreken;
dat wordt geen fictieve actieve build of onbeperkte writerclaim.

Iedere normale oplevering gaat compact via intake → integrator/release:
**taak, resultaatcommit/PR, werkelijk gedrag, bewijslink, blokkade, volgende stap.**
Intake levert het compacte PR-/blokkadeoverzicht aan de manager, zonder een
tweede centrale queue te schrijven. Domeinbesluiten en problemen die meerdere
verticale taken blokkeren komen rechtstreeks bij de teamleider.
Een sessie-ID, `is_running`, geaccepteerd bericht of `unknown`-activiteit bewijst
geen draaiende uitvoering. De laatste live snapshot had geen input-/planblokkade;
native/hardware/intake/reviewer waren toen niet-running. Hardware heeft
inmiddels de actuele bron en eigen probe teruggeleverd; native nog niet.
Nieuwe berichten zijn geaccepteerd, geen permanente bewaking of achtergrondwerk
beloofd. Intake heeft een volledig gepagineerd dossier van de toen tien open
PRs geleverd (`files/triage-current/dossiers.md` in zijn sessieartifacts);
#79/#80 zijn latere intakes, geen onderdeel van die oudere tien-PR-snapshot.

### Begrensde implementatiecontracten

**Besluit TASK-N01-NATIVE-CONFIG:** vrijgegeven aan NET039 na expliciete
REL-bevestiging dat beide gebieden geen actieve/gereserveerde/gedelegeerde
writes hebben. Bestaande NET-transportclaim uit donorqueue blijft behouden;
dit is geen hergoedkeuring van bestaand werk. Concrete scope hieronder.

**Open planning, alleen betrokken uitbreiding wacht:** SEC-publisher/sourcepersistentie
(geen tweede verifier) wacht op exacte nieuwe leafbestanden; gericht voorstel
bij de eigenaar opgevraagd.
NAS-rootbootstrap/runner/80B-handoff blijft uitsluitend REL en niet actief
naast DEV1; alleen de onderstaande NAS-payloadvoorbereiding is vrijgegeven.
Deze verzoeken gaan niet opnieuw via Simon wegens manageronbereikbaarheid.

Elke rij is één primaire implementatie of expliciet geparkeerde reservering.
Afgeronde/frozen bron kan worden gelezen/geïntegreerd, niet stil hervat.
Afzonderlijke worktrees, Cargo-targets, QEMU-images/OVMF/swtpm-state en buildroots
per uitvoerder; geen gelijktijdige writers van dezelfde testmiddelen.

Voor de coördinerende/read-only rollen gelden aanvullend:

| Eigenaar / opdracht | Schrijfgebied | Afhankelijkheid en acceptatie |
|---|---|---|
| Assistent5d5 / TASK-ASSISTANT-QUEUE | Alleen `docs/WORK-QUEUE.md`; eigen sessieartifacts | Exacte gepushte donorbron/vrijgave ontvangen en ACK; vorige schrijver stopt. Voor elke nieuwe uitvoerder vóór implementatie taak/eigenaar/files/dependencies/acceptatie registreren, conflicten/claims controleren. Routinekeuzes zelfstandig, geen score-/status-/ABI-/trustbeleidwrites |
| Teamleider89f / TASK-TL-HANDOFF | `docs/PROJECT-STATE.md`, `docs/progress.json`, `README.md`, `docs/PROGRESS.md`, `docs/assets/progress.svg`, eigen `docs/verification/teamleader-*`; queue alleen deze laatste donorwijziging | Gepushte bron/remote-SHA/ontvangst vastleggen, technisch geïntegreerde kwaliteit/afhankelijkheden bewaken; progress write/check en onveranderde criteria. Na vrijgave geen queuewrites |
| Directeuref7 / TASK-DIR-ROADMAP-QUALITY | Repository read-only, eigen sessieartifacts | Onderbouwde uitzonderingsbesluiten, geen routinegoedkeuring of extra bouwer; ontbrekende menselijke B05-reviewer/voorwaarden expliciet open houden, geen betaalde engagement zonder toestemming |
| Intakef2 / TASK-PR-INTAKE | Eigen sessiedossiers/PRmetadata binnen oorspronkelijke mandaat, geen centrale repositoryfiles | Bestaande taak/werk behouden; één primaire route, exacte head/base/stack/review/gates doorgeven aan8. Geen algemene werkverdeling/mergebevoegdheid |
| Reviewer7f / TASK-PR-REVIEW | Read-only bron/diff en eigen reviewartifacts | Eén begrensde review op vastgelegde bron, concrete bevindingen naar eigenaar/intake, geen parallelle reparateur of menselijke auditclaim |
| Integratore8 / TASK-INTEGRATE-74 | Alleen afgestemde integratiebranches/PRmetadata/conflictopname binnen eigenaarsmandaten | Actuele head/base/latest-per-contextgates/review en inhoudsbehoud; geen regels omzeilen. Centrale queue uitsluitend via assistent, score/status via teamleider |
| Build12 / TASK-RUNNER-POOL-HANDOFF | Eigen sessieartifacts, goedgekeurde lokale toolcache en eigen tijdelijke teststate; geen nieuwe tracked bronclaim | Tool-/firmware-identiteit, verse headroom, exclusief slot en vrijgave; Windowsroute bij21, geen dubbele suite/VM of overname door timeout. Nieuwe codeclaim eerst via assistent |

**Aanvullende vrijgaven en overdrachten (leidend boven afgeronde rijen
hieronder):**

| Taak / rubric / release / basis / afnemer | Exclusieve bestanden | Contract en observeerbare acceptatie |
|---|---|---|
| TASK-NAS-EXACT-REPLAY-PROBE / P01,P04,B06 / FR-P01,P04(bijdrage) / ce52, #83source1219 frozen / REL4,SEC2 | NAS10: alleen `services/nas/native-probe/src/main.rs`, `services/nas/native-probe/CONTRACT.md`, `services/nas/native-probe/tests/**`, `services/nas/native-probe/verification/replay/**` | REL bevestigt geen actieve/gereserveerde/gedelegeerde NASleafwrites; #83-runtime/evidence ongewijzigd. Aparte branchce52: alleen payload/startuprequirements en hostnegatieven voor werkelijk boot1-issued Session40+Upload40 na diskcoldboot weigeren. Geen fictieve tokenbron: native exactreplay BLOCKED tot REL na eigen profilefreeze nieuwe root/runner/privé80B-handoff daadwerkelijk levert. Testartifact bindt originrun/role/stage/complete/lengte/hash en immutable coldinput, tokens nooit serial/gewonelog/Git. Geen nieuwe crypto/wire/ABI/productrestore/duurzaamheidsclaim of VMslot. Volgende stap: vereisten en begrensde payload/negatieve hostfixtures opleveren; daarna bronfreeze, geen runtimehulp overnemen |
| TASK-N81-TLS-CI / N03 / FR-N03(bijdrage) / frozen4a0/#81 / integrator8,BUILD12 | NET6: uitsluitend nieuw `.github/workflows/net-tls-quality.yml`, eigen nieuw `services/net-tls/verification/ci-only/**` | Build12 bevestigt geen actieve/gereserveerde/gedelegeerde writer op filename; werkelijke net-tls-crate op4a0 via git ls-tree bevestigd. Korte opleverblokker naast primaire N01, geen TLScode/provider/grantwijziging. Manifest-/pathbinding controleren, locked hosttest/Clippy/fmt/none en echte dedicatedCI; geen conditionalskip of oude groene CI herlabelen. Frozen4a0-code behouden, CI-delta apart naar8 |
| TASK-H03-TRANSPORT-MODEL-1 / H03,B06 / FR-H02,B04(bijdrage) / contract#88 `823cba3eb38786d39726ecfb98d6ce7f258754f7` / kern14,SEC2,REL4,intake3 | HW9: nieuw `drivers/virtio-snd/src/{transport_model.rs,transport_model_tests.rs}`, alleen test-only modulekoppeling in `drivers/virtio-snd/src/lib.rs`, eigen `drivers/virtio-snd/verification/{README.md,SHA256SUMS,model-*}` | Donorregister/HW bevestigen eigen codec/contract frozen; geen tweede runtimewriter. Pure bounded hostmodeltoestanden Absent/Probed/Ready/Streaming/Fencing/Quiesced/Quarantined/Released, geen exports/productiecode/deps/manifest/lock/unsafe/I/O. Modelleer revoke, checked epoch/slot, stale/doublecompletion, double release, RXverbod, timeoutquarantaine en readbackvoorwaarden. Status0+BME-off zijn modelaannames, GEEN bewijs voor echte DMA-quiescentie. Minstens een aantoonbaar rode mutant per I1–I6, incl dubbelrelease/completion; exacte mutatiedelta/commando/exit/restore bewaren. Locked tests/fmt/Clippy/none, geen VM/hardware/H03punten; B1–B9-read-onlyafstemming14/2/4 blijft apart. Volgende stap: modelinvarianten+negatieve mutantcases, daarna zelfstandig leafoplevering aan3 |
| TASK-UI-PRIMARY-HOVER / U01,U04,U06 / FR-U04,U05(bijdrage) / code0818f5e, bewijse37fa17b / ontwerp15, review7, REL4 | UI13: `ui/native/src/render.rs`, bestaande `ui/native/tests/render_regressions.rs`, uitsluitend eigen `docs/verification/native-semantic-consumer/UX-V01-primary-hover.md` | Vorige renderfix/IMAGE-0818 frozen; donorregister bevestigt UI als renderowner, geen gedeelde writer overgenomen. Aparte leaf vanaf0818, oude PR68/DEV1-kandidaat niet wijzigen. Bestaande primaryaccenttokens bij hover behouden; eerst falende pixelregressie, minimale fix, gerichte/bestaande rendertests/fmt/Clippy/none. Geen token/API/runner/kernel/manifestwrites of VMslot. Wrapper89e blijft geparkeerd; geen native/human-aftekening. Volgende stap: rode primaryhoverpixelcase en minimale rendererfix, daarna bronfreeze voor7/15 en zelfstandige oplevering |
| TASK-N01-NATIVE-CONFIG / N01,N02 / FR-N01,N02 / Stack4a0, DHCPclient16adfe5, codecmainca45 / REL4,NAS10 | NET039: `services/net-stack/**`, eventueel nieuw `services/net-config/**`, eigen verificatie daarbinnen; bronleafs `services/net-protocol/**` alleen inhoudsbehoudend hergebruiken, geen nieuwe protocolwrites | Eén bestaande Stack/PacketDevice-RX/TX/completionroute. DHCP-adres/default-route/DNS en lease/T1/T2/expiry/link-down/reconnect; seconds naar100Hz checked/overflow. CheckingAddress wordt pas Bound na echte RFC5227-adresconflictproef, geen bevestigingsboolean als nativebewijs. Begin met geïntegreerde lifecyclehosttests incl malformed/stale/NAK/expiry/conflict/overflow/backpressure, dan locked test/Clippy/fmt/none op exacte bron; runner/root/SDK/driver en native issuer/fence/TCP-CR3 blijven bij hun eigenaren. Geen tweede stack/ABI, geen frozen #81/#87 muteren, geen nativeclaim zonder eigen-kernelproef. Volgende stap: aparte leafbasis met alleen benodigde DHCPbron, contract en eerste rode lifecyclecase; daarna zelfstandig implementeren/testen/opleveren |
| TASK-H03-TRANSPORT-CONTRACT-1 / H03,A04 / FR-H02,A06,A07 /72f /14,2,4 | Alleen nieuw `drivers/virtio-snd/TRANSPORT.md` | Eén brongebonden state-/failuretabel voor PCI/queue/buffer/DMA/lease/revoke/reset/quiescence/stalecompletion. Eén read-only afstemming14/2/4; bronpins/links/diffcheck en minimale runtime-experimentgate. Geen code/IRQ/MMIO/mic/crypto/ABI; epoch alleen bewijst geen devicefence/DMA-isolatie |
| TASK-REL-CANDIDATE-DEV1 / R01,R03 / FR-U03,O01,R01,R04(bijdrage) /ca45 /20,21,Simon | Bestaande gewone Materiaalbuild/previewgate alleen uitvoeren, eigen geïsoleerde build/source/target/VMstate en eigen releasebewijs; geen nieuwe rootbronclaim | Eerst offlinebuild/pixelreferentie,12headroom/ACK, maximaal2seriële bestaande gates/referentie en gewone7procespreview met echteQMPinput+diskorakel. Bron/v3/kernel/loader/image/tool/firmware/profiel/start-stop/limits vastzetten; geen13fixture als gewone build, geen volledige FR/Windowsclaim vóór echte uitvoering |
| TASK-UX-SAVE-RECOVERY / U03,U04,U06,R01 / FR-U03,U05,S04,O01,O04 / ca45 /13,20,21 | `docs/implementation/UX-MATERIAAL-SAVE-RECOVERY.md`, `ui/design/save-recovery.svg` | Maak/bewerk→Save→lijst→zelfdeobject koud/reopen, dirty/saving/confirmed/Unknown/fout/vol/deny/stale/cancel, keyboard/focus en800/1280/1920op100/200%. Geen saved zonder ACK, geen replay bijUnknown, Unchanged geen nieuwe commitclaim.12statechecks en hostrender;63afgeleverd, geen native/human gate |
| TASK-ASSET-MATERIAL-FLOW / U01,U04,U06 / FR-U01,U05,U06 / ca45 /15,13 | Alleen `ui/assets/material-flow-icons.svg` | Vier document/history/saved/retry-symbolen24×24,currentColor/1.75ronde stroke, metadata als compact manifest/herkomst/licentiegrens. XML/gesloten allowlist/geen externe refs/scripts en bekeken24/48px licht/donker;d354geleverd. Native vector/rasterinput ontbreekt, geen rendererwijziging |
| TASK-RD-TLS-TIME / N03,B04 / FR-N03 / ca45+CORE9af/NET4a /2,6,4 | Alleen `docs/research/TLS-CERTIFICATE-TIME-BOUNDARY.md` | Max90min/5primairebronnen+gerichte inspectie; uitvoerbare tijdbootstrap/tegenvoorbeeld zonder RTC-upgrade of stille fallback.4ffgeleverd: PKIXtijd, NTSbootstrapcirkel/delay, rawkey-enrollmentgrens, dezelfde keten moet toekomstig interval dekken. Advies is geen aangenomen ADR/productieprovider; vervolgvoorstel eerst besluit |
| TASK-V18-TLS-TIME-FAILCLOSED / B05,N03 / FR-N03,B05(bijdrage) / eigen ca45, CORE`9afabbd8266cd1230b7b89b6c70a56caa22f88ce`+NET`4a0ce99ff6b88adcd0e2c6be0571b30390d8d1c8` /17,2,6 | Nieuw `experiments/verification/tls-time-gate/{model.py,test_model.py,README.md}` | Alleen MissingSource/UnauthenticatedRtc→geen CertificateTime; bronregels/hashes/modelcorrespondentie, eindige histories en echte negatieve RTC-promotiemutant.60babgeleverd4tests/2380histories/6942calls; geen handshake-entrypoint dus geen authenticated-success/Rust-refinement/nativebewijs. Geen toekomstige trust/freshness/rollbacksemantiek; CORE-ack ontbreekt als gelabelde beperking |
| TASK-CAP-DIGEST-TCG-2 / K07,R02 / FR-K02,R02(bijdrage) / ca45 /17,12,4 | Alleen `docs/verification/performance-19/**`; bestaande diagnostiek gebruiken, geen sourceoptimalisatie | Bestaand CAP-DIAGNOSTIC-DIGEST REF/DIRECT/ZERO-protocol; één kort4slot door12 geregistreerd. Eerst goedgekeurde env/branchpreflight/eigen target, dan brongebonden begrensde run; TCG-RDTSC geen fysiekeCPU/GUI-/VM-FR-meting. Geen installatie/extraVM/herhalingsbatch of optimalisatie zonder baseline |
| TASK-QEMU-PRODUCT-DEV1 / U03,U04,R01,R03 / FR-U03,U05,O01,R04 / eigen ca45, kandidaat nog niet vrij /4,Simon | Alleen `docs/verification/product-test-20/**` | Vijf observeerbare cases create/edit/Save/reopen,cold,deny/revoke,Full,Unknown met inputherkomst/provenance/start-stop. EchteUI/QMP-input onderscheiden van testcontroller; faultfixtures vooraf goedkeuren. Protocol voorbereid, uitvoering NOT RUN/BLOCKED tot vaste DEVartifact+12slot/toolroute; geen ongetest gedrag als FAIL/PASS |
| TASK-WINDOWS-DELIVERY-TAKEOVER / R01,R03 / FR-R01,R04 / ca45, donor bffe11 /4,20,Simon | `tools/package_windows_qemu.py`, `tools/windows-qemu/{Start-NieuwOS.ps1,Test-Launcher.ps1,README.md}`, `tools/tests/test_package_windows_qemu.py`, `docs/verification/windows-delivery-21/**` |12→21formele wederzijdseACK, vijf routeblobs donor/ca45 gelijk. Bestaande route behouden,10host+5echteWindows-synthetische tests op0915828f, geen nieuwe OS-ZIP/VM. Na4/8DEVvrijgave packagen/hashcheck/Windowsstart-bediening/herstel en eerlijke limitations; Linux/Windowsbewijs apart, geen policyverruiming/persoonlijke disks |
| R22-IMAGE-VIEWER-LEAF-1 / A04,U03 / FR-A03 / ca45 /11 | Alleen nieuw `sdk/examples/image-viewer/**` incl eigen manifest/lock/src/tests/README/verification | Hergebruik image-codec en bestaande input/surfacetypes; decodedImage→checkedviewport/zoom/pan en load/stale/deny/corrupt/cancel/OOM/cleanup. Geen nativeentry/wire/bootstrap/sharedSDK/renderer/rootwrites. ReadOnlywoord112/domain0SEND geen afgedwongen object-onlyREAD; provider moet object/revision/READ+maliciousIPC/LIST/mutatieweigering afdwingen, launch/focus door4/13. Codec19host+nonebaseline is geen viewerresultaat; onafhankelijke leaf mag nu bouwen |
| TASK-SEC-CRYPTO-AUTHORITY / B01–B06 / FR-B01,B02,B04,N03,P01–P06(afhankelijkheden) / ca45+SEC7f/CORE9af/NET4a /6,10,4,17,18 | Alleen nieuw `docs/contracts/NATIVE-CRYPTO-AUTHORITY.md` | Bestaande crypto/identiteit/tijd/entropy/fout/recoverycontracten en ontbrekende providers/grants brongebonden; geen eigen KDF/signer/tijdbron/policy/ADR/SDK/rootwrites. Eén gerichte read-only afstemming metNET/NAS/release; SEC72 afgerond, geen tweede producer |
| TASK-H03-VIRTIO-SND-CODEC / H03,A04 / FR-H02,A06,A07 / ca45 /14,4,2,11 | Alleen nieuw `drivers/virtio-snd/**` incl eigen manifest/lock/tests/contract/verification | OASISvirtio1.3§5.14 boundedconfig/PCM_INFO/status, output-onlyS16/48k/stereo; exacte lengte/count/status/features/direction/range/padding/overflow-negatieven. Bestaand6testprototype als gerichte crate-tests, Clippy/fmt/none. DeviceIDdetectie geen audio; geen mic/transportqueues/DMA/MMIO/broker/kernel/root/runners |
| TASK-UI-IMAGE-0818 / U01,U04,U06 / FR-U01,U05,U06(bijdrage) /0818f5ec /15,16,20,4 | Bestaande probe/runner uitsluitend uitvoeren, eigen build/target/VMstate/eigen bewijs; geen runnerbronwrites | Na12/4slot native-system-materiaal-probe + run_materiaal_probe.py op exact0818; één immutable bron/v3/ELF/ESP/firmware/QEMU/profiel/start-stop/frames voor alle beoordelaars. Alleen echte ondersteunde inputstappen als bediening, testcontroller apart. Geen gewone interactieve/fullVM-FR-/assetconsumptieclaim; geparkeerde AppWIP niet aanraken |

De onderstaande eerdere taakcontracten bewaren hun interfaces/bewijs, maar
hun **afgeronde** writerstatus wordt uitsluitend door de actuele rolrijen en
bovenstaande overdrachten bepaald. Geen afgeronde SDK-/lock-/runnerclaim
blijft stil voor nieuwe taken gereserveerd.

| Taak; rubric; release-ID's | Exacte schrijfgebieden | Afhankelijkheid, interfacecontract en observeerbare acceptatie |
|---|---|---|
| TASK-NAS-NATIVE; P01–P04/B01/S08; FR-P01–P06/K03/O01/O04/B01 | `kernel/Cargo.toml`, `kernel/src/m2/m3.rs`, nieuwe `kernel/src/m2/m3/nas.rs`, `tools/build.py`, `tools/build_nas.py`, `tools/test_nas.py`, `tests/test_nas_build.py`, `tests/test_nas_runner.py`, `docs/verification/nas-release-intake/**` | Basis37b09c75, freeze808f5391 op ca45+NASce52+oracle7890ea8. Twee opt-in normal/cold-profielen, bestaande ELF-envs/launch/IPC/block/RNGgrants. Bestaande `system.rs`, `kernel/build.rs`, loader/SDK/keyring/volume bytegelijk. Normaal/koud caller-, deny/stale/exit/poll- en onafhankelijk diskbewijs; root hosttests en relevante native regressies. Geen TLS/E2EE- of volledige P-acceptatie |
| TASK-SEC72-DROP; B01/B06/A03; FR-B01/B04/K03 | `sdk/runtime/{Cargo.toml,README.md,src/message.rs,src/native.rs}`, `sdk/examples/keyring/src/service.rs`, `sdk/keyring-random-client/src/lib.rs`, `sdk/examples/random-probe/src/main.rs`; precies de onderstaande twintig consumerlocks; eigen IPC-dropbewijs | Geteste coded4677211, bewijs9e7b1b92. RAII-Drop na capabilitycleanup, geen Unknown/no-retry/lease/callerdelta. Eigenaar meldt36host/Clippy/fmt, echte geoptimaliseerde320bytewipe en12native(system2/B015/WAIT1/RANDOM4),66Gitblobhashes. Review/handoff via intake, oud82b-bewijs blijft eigen bron; geen tweede crypto/identiteit/tijdcontract vóór afsluiting en nieuwe fileclaim |
| TASK-N04-NET-BROKER; N04/A03/B06; FR-N01/N04/K03 | Nieuw `services/net-broker/{Cargo.toml,Cargo.lock,README.md,src/lib.rs,tests/rights.rs,verification/**}` en nieuwe `.github/workflows/net-broker-quality.yml` | Basis4a0ce99f na TLS-handoff. Bestaande Stack hergebruiken; contract in eigen README vóór interfacevastlegging read-only met SEC afstemmen. Authentieke kernel/capabilitymetadata, nooit aanvraag-PID/route/boolean als identiteit; begrensde sockets/bestemmingen/grants, generaties, revoke op bestaande/nieuwe sockets, cleanup/Unknown zonder ACK/replay. Normaal/deny/crosscaller/stale/revoke/helperroute/resource/timeout-tests, locked host/Clippy/fmt/none/echte CI. Ontbrekende issuer/SDK-seam expliciet open; geen tweede wire-ABI of native afdwingingsclaim |
| TASK-NAS-RESUME-1; P02/P03/S09; FR-P03/P04/P05 | Nieuw `services/nas/native-client/{Cargo.toml,Cargo.lock,README.md,src/lib.rs,tests/resume.rs,verification/**}`; uitsluitend `.github/workflows/nas-core.yml` voor vierde echte crate | Basis86a9600e/ce52. Bestaande opaque authority/ticket/revisie/generatie hergebruiken, geen tweede wire-/crypto-/identiteitscontract of PID/route/boolean als autoriteit. Begrensde offlinequeue; conflict bewaart beide zijden; revoke/stale/Unknown geeft geen ACK/replay zonder expliciete reconcile. Normaal/outage/conflict/revoke/Unknown/crashstate/quota/overflow hostnegatieven, Clippy/fmt/none en echte vierde CI-crate zonder skip. Ontbrekende shared seam eerst melden; hoststate geen duurzame sync/E2EE/native claim |
| TASK-R73-PROTOCOL-CONSISTENCY; A02/A03; FR-A11/A12/A13 | `sdk/app-management/src/protocol.rs` met inline tests en eigen provenancezin in `docs/implementation/APP-PACKAGES.md`; nieuw extern testpad eerst melden | Aparte leaf/worktree vanaf bc27ed0f, frozen #73/#80-head niet wijzigen. Encoder weigert OKCommit/None/catalogrev0 conform eigen decoder; alle geldige replyvarianten roundtrip en malformednegatieven. Descriptortekst storeRef/len/digest onderscheiden van publisher/source/versiontrust. Eerst rode gevallen, minimale validatorfix, bestaande18+8tests/Clippy/fmt/none. Geen manifests/locks/root/ABI; één definitieve basis-/conflictopname door e8 |
| TASK-L01-ROOT-ADAPTER; L01; FR-L01/L02/L03 | Uitsluitend `experiments/ai/model-store-contract/**`, incl. eigen manifest/lock voor relatieve root-pathdependency | Basise57e7d70+behouden test/README-vervolg, eigen worktree op PR78-rootcontract7f4f7bfe. RootArtifactStore/ArtifactRef/descriptor/error direct gebruiken, geen tweede trait/ID-afleiding/provider. Bestaande3unit+13contract, rootadapter shortread/exactEOF/stale-na-verify/cleanup/digesttamper, Clippy/fmt/none. Eerdere LLVM-ICE101 bewaren; twee nieuwe default-none Cargo-exits0 melden als non-reproduction, wrapperexit1 niet als Cargo-falen. Geen inference/download/netwerkweigering |
| TASK-R68-RENDER-FIX; U01/U04/U06; FR-U01/U05/U06 | `ui/native/src/render.rs`, nieuw `ui/native/tests/render_regressions.rs`, `docs/verification/native-semantic-consumer/R68-render-review-fixes.md` | Aparte worktree vanaf exact e382a989d4b28a45a23e31ff4edef2426e9c2616b. Eerst rood:800×720@100/8objecten semantiekbudget, fysieke rootbounds bij100/200%. Minimaal worstcasebudget/vooraf atomiccount en één schaalconversie; regressies/Clippy/fmt/none, daarna7f-herreview. Geen Cargo/locks/kernel. SDKlib+nieuw `sdk/examples/materiaal/tests/semantic_actions.rs` op89e20045 onaangeroerd/geparkeerd; geen screenreader/nativeacceptatie |
| TASK-H05-PARSER; H05; FR-S04/S05 | `drivers/acpi-power/src/lib.rs`, `drivers/acpi-power/README.md`, `drivers/acpi-power/verification/**`; gerichte inline parsertests | Basis4af90fcd; alleen H05-commits0ce565a/ff6e4c4/50e5954/4af90fc intake, niet de hele #79 met historische USB/display/statusdiff. Bounded FADT/DSDT-S5subset: checksum/lengte/range/overflow/unsupportedAML weigeren, UEFI-pointer en legacy-scanalignment onderscheiden, geen willekeurige I/O-port/caller-powerplan. 22hosttests/Clippy/fmt/none groen gemeld, firmware-only RSDP gevonden maar nog geen volledige DSDT-parserrun. Geen eigen-OS/fysiek/poweroffbewijs. Release behoudt snapshot-ABI; virtio-snd pas na deze overdracht een aparte taak |
| TL-VERIFY-SOURCE-RUNTIME → TASK-KS-CR3-DIAG; K02/K05/S08; FR-K02/O04/I01 | Alleen lezen in kernel/loader/ABI; eigen bestaande verifierevidence/artifacts, geen runtimewrites | Eerst lopende bron-/hashgebonden verifier expliciet overdragen. Dan NET4a+CORE94-trace en eigen fresh artifacts vergelijken: niet-present instructionfetch0x200130c direct naCR3=1000 → DF/triplefault. Eén concrete oorzaak/repro of ontbrekende voorwaarde+commando, geen brede audit. Exacte minimale patchclaim daarna met release/TL afstemmen, geen parallelle kernelwriter. S09-vervolg vanaf #76/S06 en K06 blijven open, geen domeinovername |

### Gedeelde bestanden en afgeronde claims

- Release bezit alleen bovengenoemde actieve NAS-kernelsubset/`tools/build.py`.
  Reeds toegewezen runtime-/ABI-verantwoordelijkheid blijft behouden, maar
  BootInfo/ACPI is nu een geparkeerde vervolgtaak, geen tweede actieve builder.
  IPC-WAIT, TPM/NV0041 en TLS-time zijn afgeronde bronclaims, geen blokkade.
- Root `Cargo.toml`/`Cargo.lock` en centrale docs in frozen PR73/78 worden
  uitsluitend inhoudsbehoudend met e8 afgestemd; geen vrije producerwriter.
  Na donorbronvrijgave bezit uitsluitend assistent5d5f8513 `docs/WORK-QUEUE.md`;
  manager houdt `docs/PROJECT-STATE.md`, `docs/progress.json`, `README.md`,
  `docs/PROGRESS.md`, `docs/assets/progress.svg` en eigen managerbewijs.
  Directeur/intake/uitvoerders schrijven deze niet als routinecheckpoint.
- SEC72 is de enige eigenaar van precies deze twintig consumerlocks:
  `sdk/Cargo.lock`; `experiments/quantum/native/job/Cargo.lock`;
  `sdk/{display-client,input-client,keyring-client}/Cargo.lock`;
  `sdk/examples/{calculator,compositor,desktop-init,display-input-probe,image-probe,input-service,ipc-wait-probe,materiaal,memory,net-probe,random-probe,tcp-probe}/Cargo.lock`;
  `services/input-core/Cargo.lock`; `ui/{native,surface-protocol}/Cargo.lock`.
  Eigen bewijs: `docs/verification/B02-IPC-ZEROIZATION.md` en de bestaande
  `ipc-main94-*`, `full-release-ipc-drop-system`, `ipc-drop-keyring`,
  `ipc-drop-random`-mappen. Geen NV/TPM/kernelCargo/tpm_fixture/buildwriter.
  NET heeft de twee random-client/probefiles expliciet voor SEC vrijgegeven.
- R77-coldtools/tests/bewijs zijn frozen op bffe11. B72 schrijft geen
  NAS-runner/build.py/kernel/SEC-locks; beoordeelt onafhankelijk. Bestaande
  workflows in PR73/78 zijn frozen; NAS krijgt de genoemde vierde-
  crate-uitbreiding, NET uitsluitend zijn nieuwe brokerworkflow. Geen generieke
  runnerclaim. Frozen catalogus#80 geeft zijn producerclaim terug; alleen de
  nieuwe protocolleaf bezit de eigen documentsectie. UI-App-WIP is geparkeerd,
  niet opnieuw aan een ander toegewezen.

### Eerdere claimherstelmomentopname vóór specialistentoetreding

De actuele rolrijen, overdrachten en bronnen hierboven gaan voor onderstaande
eerdere tussenstanden (bijvoorbeeld queued-CI en nog open NAS-herreview).

1. **e8/#74:** d7b7fcce heeft volgens eigenaar18native+15Rust/12harnas/24preflight
   en56hashgebonden bewijsblobs; laatste finale CI pending/onleesbaar door
   GitHub502/504. Geen merge/bypass/ready-eventtruc/extra push of VM-herhaling
   vanwege alleen een API-storing. De definitieve basis komt alleen van e8.
2. **Release/NAS:** freeze808,8roothosttests/fmt/diff exit0 gemeld, kernel/
   loaderbuilds op eerdere37b/455 exit0. Oorspronkelijke normale/koude pair
   positief bewaard/gepusht als5a9e71e3; rapport
   `build/nas-release-808f539/nas-system-gate.json`, log
   `/tmp/nieuw-os-nas-808f539-native.log`. B72 vond twee MEDIUM-bindinggaten:
   geen postexitESP/immutableCODE-herhash en onvolledige parentmachine/QEMU/
   firmware/werkelijke-implementatiebinding. Release reproduceerde zes
   assertionfailures exit1, log `/tmp/nieuw-os-nas-review-bindings-red-confirmed.log`;
   minimale runnerfix, hostregressies en nieuwe echte pair met directe ELF
   volgen. B72 wacht op fixfreeze, geen eigen VM/reproductie geclaimd.
   Coldtoken[42;40] is synthetische fixture, geen echte boot1-tokenreplay.
   Daarna één geïntegreerde kandidaat,
   gewone interactieve variant en volledig R01–R03-testpakket, geen losse
   featureimages als totaalbewijs.
3. **R77 geleverd, geen herreviewloop:** bffe11 op code7d66/v3949ce3:
   nieuwe echte reference133.939s/raw33, parent93.551s/raw0, cold37.543s/raw0,
   alle build/runexit0 gemeld. Zestien artifact-/claimnegatieven, waaronder
   vier firmwarewissels en latePANIC/FAIL met bijgewerkte serialhash;66oude
   blobs intact. Bewijs `docs/verification/native-cold-acceptance/review-7d66/README.md`,
   coldreport `db052373c440823fa9954298f81f076eb9a323025deddc3864e11e4e135dbb67`.
   Beide bronbevindingen gesloten; geen main-, cleanstop-, powerloss-,
   encryptie-, Windowscold- of volledige-FR-claim.
4. **Reviewcapaciteit naar afronding:** intake routeerde R73-v2, R78, R68-v2
   en R80 exclusief/sequentieel naar7f. R68 is **niet mergeklaar**:
   HIGH-correctheidspanic door14nodes zonder paging/statusruimte
   (800×720@100,8objecten), MEDIUM-dubbele fysieke rootschaling
   (800×600@200→1600×1200). Alleen UI repareert, App-wrapperwerk geparkeerd;
   geen eigen reviewerreproductie of nativefailureclaim.
   R73 heeft twee
   niet-blokkerende consistentiepunten (replyencoder accepteert ondecodeerbare
   Commit/None/catalogrev0; provenance in contracttekst anders dan descriptor).
   Geen verzonnen client-falsesuccess. Catalogus#80/bc27 heeft18+8hosttests/
   Clippy/fmt/none exit0 gemeld, dedicatedCI queued; twee rode ABA/cleanup-
   regressies gefixt, R80 zonder concrete blocker, geen nativeinstall.
   Afzonderlijke protocolleaf nu vrijgegeven zonder frozenheads te muteren.
   Na74 één conflictresolutie van de vijf
   bekende shared paden en echte CI; bronownerapps blijft enige reparateur.
   #78read-onlycontract zonder backend/nativecallerbinding, CI laatst queued.
5. **SEC72/AI/UI/NAS-leaf kunnen onafhankelijk door:** concrete claims hierboven,
   geen wachten op ongerelateerde oude rootclaims. SEC72-review/handoff sluit
   de huidige stap vóór nieuwe centrale crypto-/identiteitscontractimplementatie.
   Native levert ontbrekende overdracht en onderzoekt daarna de concrete
   CR3-fout read-only; hardware heeft actuele bron/firmwareprobe geleverd,
   geen ontbrekende algemene inventarisatie. TLS4a heeft13host/Clippy/fmt/none
   exit0 gemeld maar TCPnormal bereikt cold niet door PF/DF; trusted calendar/
   signer/provider/TLS-procesgrant blijven open. Niet-verifieerbare naam
   “Copilot A” is aan geen sessie gemapt.

## Historische teamleiding — oorspronkelijke bronbasis `0ade8014`, 30 oktober 2026

Startbasis `0ade8014`; actuele managerintegratiebasis `94c8908` na #66.
De hieronder genoemde oorspronkelijke branchechecks zijn momentopnamen;
latere expliciete mijlpalen in dit hoofdstuk gaan voor oude headvermeldingen.

Simon heeft de manager opgedragen de twaalf bestaande sessies aan de volledige
ROADMAP en FULL-TEST-RELEASE te toetsen en uitvoerbaar werk te laten voortzetten.
Onderstaande verdeling vervangt de historische agentclaims verderop, niet de
62 productcriteria, vaste rubric of bewijsniveaus. **34/100 blijft 34/100.**
Alle twaalf eigenaren hebben een concrete voortzettingsopdracht ontvangen en
hun stand teruggestuurd. Berichtacceptatie is geen bewijs van een draaiende
build: de app meldde activiteit `unknown`, geen observeerbare `busy`-status.
Alle hieronder genoemde deelresultaten zijn eigenaarrapportages, tenzij de
managercontrole uitdrukkelijk is vermeld. Geen nieuwe native manager-run.

| Taak / sessie | Bevestigde bron en stand | Exclusieve uitvoering / eerstvolgend resultaat | Open bewijsgrens |
|---|---|---|---|
| TL-REL / `2ab17599-85c8-4409-ba06-a93c27485328` | `simonvanieperen-refactored-pancake`, `0ade8014`; 10 updateharnastests groen, nog geen native run | Finale productintegratie, gedeelde manifest-/kernelinterfaces en gecombineerde image; ontvangt buildteambron en herhaalt bediende gebruikers-/herstelketen | Geen tweede B03-herhaling als hoofdtaak; FR-R01–R04 en finale VM-FR open |
| TL-AIQ / `381d080e-be3d-4c78-ac72-b99b1de2b4e5` | `simonvanieperen-crispy-tribble`, basis `0ade8014`, nog ongecommit; 9 buildgatetests en 6 qsim-hosttests groen; FPU-artifactgate exit 1 op `xorpd`, geen VM-run | FPU-WIP als mislukt checkpoint behouden, niet als geaccepteerde integratie; daarna L01-model/licentie/resource-/native-interfacestap. Toolroute/preflight nu groen | `kernel/Cargo.toml`/`tools/build.py`-WIP gemeld aan release; allowlist niet versoepeld; L02 al accepted, FR-L01–L04 open |
| TL-ISSUES / `7f895683-fc79-48d0-a98a-aa0a6ccfe8b9` | `simonvanieperen-urban-robot`, `0ade8014`, schoon; read-only #66-waitreview en issue39–43-triage geleverd | Gerichte read-only PR-controles van intake, issue/bewijskoppeling; geen merges of gedeelde writes | Geen eigen nieuwe tests/audit; issues #39–43 blijven open |
| TL-PRS / `e8e471ce-0601-417b-b4fa-a04fd9c7b954` | Reviewfixbranch `d104fd9`; #62 nu `9f69e1c`, Windows-linkertestregressie hersteld | Enige PR-lifecycle-/mergecoördinator; #62 actuele CI/review, #66-fixopname met buildteam | Oud groen op `f6d3a10` is historisch; #65 pas sluiten na aantoonbaar inhoudsbehoud |
| TL-HW / `f5b20312-b6c5-46b0-bbf6-76970d3470a2` | `simonvanieperen-friendly-guacamole`, `6daadbb`; USB-descriptorcore `f8e7490`, 7 hosttests/lint/target groen | Na concrete xHCI-bronblokkade doorgestuurd naar publieke ACPI/QEMU-shutdownslice, uitsluitend nieuwe `drivers/acpi-power/**`, contract/tests/eigen bewijs; release reserveert native broker na v3freeze | Nog geen xHCI-I/O/enumeratie/native profiel; H02–H06 blijven open, fysieke/hostpoweracties niet geautoriseerd |
| TL-NAS / `d7636e9c-fe84-4e63-8f37-4d08a5f50d0d` | `109c9f0`, concept-PR #71; no_std NAS-core/VolumeStorage host-/targetbewijs | `services/nas/**`: echte ring-3-service/admin-/gastprobe met callerbinding en volume/RNG; native scopevraag beantwoord, gedeelde files vooraf afstemmen | P01–P04 0/5; geen native NAS/transport/multideviceacceptatie; FR-P01–P06 open |
| TL-APPS / `9db24a72-2ae7-4807-8e68-2fc88cb235d3` | `codex/apps-sdk-packages`, `c7b30f0`, concept-PR #73; 8 hosttests/lint/target groen | `crates/app-package-core/**`, appcontract en eigen bewijs; native installer/persistentie/geautoriseerde launch met release afstemmen | Geen native install/update/remove; FR-A01–A15 en appportalen blijven verplicht |
| TL-SEC / `081d6f31-7de1-4713-b466-d15adf51b2ef` | Nieuwe concept-PR #74 `cad3b55`, definitief geteste bron `fecc4b9`, 18 B03-stages/capture groen; bewijsnotitie door manager uit Git gelezen | Updatepolicycheckpoint persistent; nu B02/NV-rollbackanker, gedeelde TPM-grens vooraf melden. Intake routeert nieuwe #74 naar vrije reviewer | Releaseversievenster is geen vertrouwde kalender; beleidsdistributie/antirollback/ceremonie open; #72 is ander IPC-WIP |
| TL-NET / `039dbef8-bc79-4480-b003-55b8eccba736` | #70 nu `5d95ea9`, 300-byte-BOOTP-padding gerepareerd; 58 hosttests. DHCP-client `16adfe5`, 69 hosttests incl. 11 lifecyclegevallen groen gemeld | `services/net-protocol/**`-checkpoint persistent; nieuwe onafhankelijke branch `simonvanieperen-tcp-listen`, uitsluitend `services/net-stack/**` voor NAS-passive TCP; native hook met release | Oud UDP/DNS normal+cold blijft op `9ddc6b6`; geen native DHCP/listen/TLS/app-bestemmingsgrants; N01–N04 0/10 |
| TL-UI / `764bc9fb-2036-4b13-bbff-368037f9b2d7` | #68 `2f41a47`; 29 host-/QMP-replaytests groen; eigen native normal op `1ffd271` rood | `tools/run_materiaal_probe.py`, `tools/tests/test_materiaal_probe.py`, UI-contract/eigen bewijs; keyboardgate na exacte cold-recoveryintegratie, daarna accessibility-servicegrens | App-local semantiek is geen schermlezer; gewone cold recovery op deze bron niet bewezen; U01–U06 0/15 |
| TL-CORE / `fbf60554-e98e-40bd-bd48-ee87a6498abe` | TLS-time leaf `9afabbd` gepusht; 27 runtime-/81 SDK-hosttests en 3 negatieve compileproeven/lint/target/fmt groen gemeld. TL-VERIFY-IMAGE op exact `3d2e1ab`: twee builds exit 0 maar andere ELF-hashes | Native verifier gestopt op bytebindingmismatch; nu begrensde read-only toolchain/flags/ELF-sectie-/padtrace met builderinput, alleen eigen bewijswrites | Geen eigen voltooide native gate. Opaque certificaattijd weigert None/RTC; vertrouwde provider/freshness/anker open. Geen menselijke audit |
| TL-BUILD / `b72f2536-b494-4a6b-a608-b6fd785e18ef` | #66-hoofd `c532ba88` bevroren; `simon/native-interactive-cold` checkpoint `3d2e1ab`, uitvoeringsbron `74d15ac`, v3 `24d5f4c1…` | Beide echte native gates groen gemeld en bewijsnotitie door manager uit Git gelezen; nu aparte INTERACTIVE-COLD-1-branch met nieuwe `tools/materiaal_cold.py`, `tools/run_materiaal_cold.py`, tests/eigen bewijs | Nieuwe Windowsgast/ZIP en gewone cold recovery nog open; geen oude Windowsbundel herlabelen; release owns eventuele runtime-/manifestfix |

Aanvullend meldt intake `f2afe26e-5d6f-4948-89b8-0e2aeeb96225` een nieuwe
expliciete opdracht van Simon: PRs vergelijken en naar beide controlesessies
doorzetten. TL-INTAKE doet daarom uitsluitend read-only vergelijking en
niet-overlappende reviewoverdracht naar TL-ISSUES/TL-PRS; geen concurrerende
merge-/code-eigenaar. Dit is een aanvulling op de twaalf opgegeven sessies.

Intake heeft alle acht open PRs met echte Git-bestandssets/ancestry vergeleken:
de API-bestandenlijst van #66 stopt op 100, maar de volledige diff bevat
171 bestanden. #72 overlapt #66 in SDK-native/keyring-service (wait versus
zeroisatie); #73/#66 delen Cargo-manifest/lock, #62 heeft bovendien eigen
preflightwijzigingen. Geen PR is op alleen ancestry als duplicaat afgeschreven.
#71-review op `109c9f0` is zonder high-confidence codebevinding afgerond;
gerichte CI voor de twee standalone NAS-workspaces ontbreekt en is door de
lifecyclecoördinator als mergeblokkade gemeld. NAS stemt workflowwrites af.
#68-review op `2f41a47` is nu eveneens zonder high-confidence nieuwe
codebevinding afgerond; eigen keyboardroute nog niet native uitgevoerd.
#73-review op `c7b30f0` is zonder
high-confidence fout in de beperkte packagecore afgerond; geen tests door
reviewer heruitgevoerd. Actuele reviewbesluiten, CI, echte installtransacties
en native launch blijven open. VerifiedPackage verleent geen launchcapability;
de consument moet brongeneratie bij gebruik opnieuw controleren.
Eigenaren actualiseren hun PR-templatebody, niet meerdere schrijvers tegelijk.

### Integratieafspraken en ontbrekende roadmapdekking

- De manager bezit de centrale WORK-QUEUE/PROJECT-STATE en
  progress.json/README/PROGRESS/SVG. Alle twaalf eigenaren zijn geïnformeerd;
  reeds gecommitteerde branchcheckpoints blijven historisch behouden bij
  integratie. Geen twee schrijvers van deze bestanden.
- Release is productintegrator en eigenaar van de finale manifest-/kernel-/
  ABI-besluiten. Buildteam voltooit eerst zijn al lopende #66-integratie;
  die technische verantwoordelijkheid wordt met exacte bron/hashes
  overgedragen. Apps bezit app/SDK-contracten, netwerk transport, NAS de
  NAS-dienst, security sleutel-/updatebeleid, hardware input/devices en UI
  de gebruikersomgeving. Gedeelde filewijzigingen vragen een concrete
  writerafspraak, geen algemene claim op elkaars domein.
- Kritiek pad: reviewfixes en gewone zevenprocesbron → werkelijk bediende
  create/edit/save/reopen/afsluitketen → koude recovery met onafhankelijk
  diskoracle → verticale integratie van veilige opslag, configuratie/TLS,
  appinstallatie, NAS en lokale inference → finaal VM-FR-image met alle
  gebruikstaken. Deze tussenstappen verlagen de volledige eindscope niet.
- K06/SMP en S09/import/export/back-up hebben nog geen afzonderlijke actuele
  builder. Release houdt deze gaten én FR-O01–O08 zichtbaar en wijst een
  concrete kern-/opslagstap toe zodra een niet-overlappend schrijfgebied
  beschikbaar is. Geen implementatie of actieve capaciteit verzinnen.
  De TLS-tijdstap vervangt SMP niet.
- Apps heeft de concrete opslagblokkade gemeld: pakketfixtures tot 2 MiB
  passen niet in de huidige 64 KiB/object, 256 KiB-volume en 8 objecten/domein.
  Een kleine A/B-installerfixture mag tussenbewijs leveren, maar is geen
  productcapaciteit. Release heeft geautoriseerde dynamic launch en het
  package-storecapaciteitscontract na v3freeze op zijn kritieke pad gezet.
  Appteam werkt onafhankelijk aan install/update/remove-transacties en
  Full/Unknown/afbreek/herstartgevallen. De RAM-only SourceRegistry mag niet
  als onbeveiligde publishertrust worden gepersisteerd; B02/B04-anker blijft
  een securityafhankelijkheid.
- Apps behoudt ook de nog niet begonnen browser/PDF/audio/video/tabel-/
  presentatietaken; UI eerste gebruik/sessie/lock/instellingen/capture en
  volledige toegankelijkheid; hardware audio/energie en concrete devices.
  Netwerk behoudt DNS/TLS/grants naast DHCP; NAS ook provider/conflicten/
  multidevice; security persistente keys/ankers/boot/privacy naast updates.
  Release bewaakt duurtest (100 cold boots/24 uur), reproduceerbaarheid,
  licenties/onderhoud/hulp en finale acceptatie. Eigenaarschap is geen
  bewijs dat al deze nog ontbrekende implementaties gelijktijdig draaien.
- M0–M8/P2–P4 en Q1-native blijven verbonden met bovenstaande werkpakketten.
  M9/fysieke profieltests en menselijke toegankelijkheids-/expertreview
  blijven expliciet open tot passende toestemming/uitvoering. M10/extra
  ISA, CHERI en fysieke QPU/Q2–Q4 behouden hun afzonderlijke onderzoeksspoor;
  zij worden niet stilzwijgend geschrapt of als VM-productbewijs opgevoerd.

### Managercontrole en hervatten

Live GitHub-controle op basis `0ade8014`: #66 `c532ba88` heeft 13 groene
checks; #71 `109c9f0` 12 groen; #65 alleen CodeRabbit groen. #62 `9f69e1c`,
#68 `2f41a47`, #70 `1bb8acf` en #73 `c7b30f0` hadden nog lopende checks.
#72 `56e76b1` had geen checks. Dit is een tijdgebonden remote-waarneming,
geen reviewgoedkeuring, mergebesluit of native productacceptatie.
Issue39–43 zijn open. Branch-, code-, bewijs- en app-sessienamen kunnen
verschillen; gebruik de door de eigenaar gemelde werkelijke Git-bron.

Actualisatie na de eerste push van managercheckpoint `f404792`: buildteam
heeft de gewone gates op `74d15ac` afgerond, bewijscheckpoint `3d2e1ab`.
Referentie 139,078 s/raw 33, gewone variant 94,889 s/raw 0, zes bytegelijke
frames en post-exit-diskoracle; manager las het brongebonden rapport via
`git show`. Dit is Linux-QEMU 10.2.1/`pc-q35-10.0`/256 MiB, geen VM-FR of
nieuwe Windowsacceptatie. De aparte cold-tools zijn buildteameigendom;
release bezit eventuele runtimefix en finale integratie. TL-VERIFY-IMAGE
heeft twee eigen builds uitgevoerd (exit 0), maar is conform stopregel
gestopt op een concrete bytebindingmismatch ondanks gelijke bronvingerafdruk:
gewone kernel `c91f39aa…` versus builder `f2e9cc4a…`, loader `e7aa259a…`
versus `7e55ab0f…`; ook de zeven app-ELFs verschillen. De eigen referentie-VM
is gestopt, gewone VM niet gestart; geen geslaagde onafhankelijke native
herhaling. De manager heeft een begrensde toolchain/flags/ELF-sectie-/padtrace
toegewezen. Oorzaak onbekend, geen assertion versoepeld. Dit blokkeert de
byte-reproduceerbaarheidsclaim, niet de geldigheid van builders eerdere run
op diens eigen gehashte artifacts of onafhankelijk bronwerk.

Lifecyclecoördinator meldt #62 regulier gesquashmerged naar
`748e29b7455ca4213a6bf3ba3808c3097458263d`: 12/12 checks op `9f69e1c`,
9/9 threads opgelost en volledige mergeboom bytegelijk aan de PR-head.
De manager heeft merge/state/head/SHA met `gh pr view 62` en remote-main
met `git ls-remote` bevestigd (exit 0); de gerapporteerde volledige
boomvergelijking niet zelf herhaald. Postmerge-main-CI nog niet aangetoond.
Buildteam integreert die main met behoud van reviewfixes, locks/preflight
en Windowsbewijs; nieuwe gezamenlijke gates blijven een herhaalplicht.
#66 oude head wordt niet op alleen oude groene checks gemerged.
Verse sessiecontrole: geen wachtende input/plan, activiteit nog `unknown`;
afgeronde reviewers/intake krijgen alleen concrete nieuwe reviewpakketten.
#74 is als nieuw pakket naar de vrijgekomen reviewer gerouteerd.

Laatste interfacebesluiten: TLS-component `34191bb` uit de historische queue
is niet beschikbaar (`git show`: exit 128; GitHub-lookup: HTTP 422/exit 1).
Remote TLS-branchinventaris toont alleen tijdpoort `9afabbd`. Netwerk is
gecorrigeerd: geen bestaande native TLS-port claimen of erop blijven wachten;
na TCP-listen volgt een gecontroleerde onderhouden-libraryportproef.
AI mag alleen het experimentele, gepinde Qwen2.5-0.5B-kandidaatprofiel en
nieuwe `experiments/ai`-reader-/loadernegatieven uitwerken; modeldownload,
echte inference en Nederlandse taakkwaliteit zijn niet bewezen. De circa
988 MB-modelartifact en appfixtures vereisen een gezamenlijk door release
gecoördineerd streaming artifact-storecontract buiten het kleine documentvolume.
Packagepublishertrust blijft een afzonderlijke authority, geen hergebruik van
B03-updatekeys of een caller-`verified`-boolean.

NAS krijgt een begrensde nieuwe NAS-specifieke workflow na filenameafstemming
met buildteam, voor de twee standalone workspaces; geen native NAS-CI-claim.
Tijdelijke centrale conflictvrijgave voor #70/#74 geldt alleen de noodzakelijke
base-update op hun eigen branch, met behoud van beide historische kanten en
34 punten, niet als overdracht van de teamroadmap. #70 meldt nu mergehead
`0078c3b` (padding `5d95ea9` + main `748e29b`), eigen gerichte gates groen,
actuele CI nog open. #74-herhaling volgt op zijn nieuwe base. Buildteam heeft
main opgenomen, nieuwe v3 `c20a3300…` en gerichte hostgates groen gemeld,
nieuwe native referentiegate liep nog. De oude `74d15ac`-proeven blijven oud.

### Vervolgronde na hernieuwde uitvoeringsopdracht

De manager heeft de werkelijk afgeleverde oudere Windows-ZIP en alle twaalf
uitgepakte bestanden zelf read-only gecontroleerd; SHA-256/CRC/bytevergelijking
slagen, geen teststate aanwezig, QEMU-executable bestaat. Compact bewijs:
`verification/teamleader-delivery-20261030.json`. Geen nieuwe guestuitvoering
of claim dat die bundel al de latere code bevat.

Actuele #66-head is `c0962e0`, volgens builder code `d9f8c216` met main748
en v3 `c20a3300…`; beide native gates zijn nu groen gerapporteerd.
Lifecycle heeft summaryhashes/bronbinding zelf gecontroleerd; manager zag
actuele head met nog lopende CI. Mergevolgorde: eerst #66 na passende checks/
review, daarna inhoudsbehoudende base-updates van #70/#74 en standalone
coverage. Centrale #75 wordt uitsluitend door manager bijgewerkt; oude
assessments niet boven de teamstand zetten.

Reproduceerbaarheidstrace `79b956e` is opgeleverd: vier apps hebben gelijke
geladen segmenten maar andere symboltabellen, Materiaal bevat werkelijk
andere geladen instructies, key/volume onder meer absolute bronpaden en
verschoven geladen data, loader uitsluitend timestamp/debug-ID-verschillen.
Geen bewezen complete oorzaak of semantische gelijkheid. TL-CORE mag een
afzonderlijke fresh-source-runtime uitvoeren met eigen hashes, nieuwe
referentie en ongewijzigde gates; het open reproduceerbaarheidsverdict blijft
behouden, geen image-identiteitsclaim.

UI heeft `f6ec180`/bewijs `0276678` gepusht: begrensde app-local semantiekboom,
host/scalar-/interactiongates groen, native_execution=false; keyboard-QEMU
volgt op de exacte eigen artifactbinding. Security heeft #74 bijgewerkt naar
`c5b20b6`, herhaling op `933591d`/main748, 18 echte stages groen gemeld,
CI nog open. B02/NV-voorloper `9ae314d` is gepusht met 23 driver-/6 harnastests;
nieuwe native NV-proef was nog niet uitgevoerd. Dit is geen KEY1-lineage/
write-ahead-/productie-authenticatie.

Apps krijgt exclusief nieuwe `.github/workflows/app-management-quality.yml`
na buildmelding. Verplicht zowel de bestaande standalone
`crates/app-package-core/Cargo.toml` als de nieuwe app-managementcode testen
zodra aanwezig: een workflow alleen voor een toekomstige crate lost de
actuele #73-dekkingsblokkade niet op. Geen conditionalskip of bestaand
rootworkflowconflict. NAS houdt zijn eigen nieuwe CI-workflowclaim.

H05-rootbesluit is door manager bevestigd: versiegebonden begrensde,
gevalideerde ACPI-snapshot via loader/BootInfo, geen langdurige rauwe
firmwarepointer; oude boot/framebuffer-ABI behouden. Release is enige schrijver
van dit gedeelde contract/ABI en legt eerst ADR/invarianten/negatieven vast.
Hardware parsers krijgen alleen begrensde FADT/DSDT en een opaque plan,
geen willekeurige port-I/O of algemene AML-interpreter. Security/hardware
beginnen read-only op dit ontwerp; uitsluitend wegwerp-QEMU, geen fysieke
poweractie of nieuwe H05-acceptatie. Dit is nog een ontwerpbesluit, geen
geïmplementeerde BootInfo-uitbreiding of clean-shutdownclaim.

Aanvullende gerichte H05-trace door TL-ISSUES op main `748e29b`, zonder
writes/tests/VM: vaste BootInfo-prefix 128/align8, `reserved0` op offset120,
GOP-extensie op 128..192. Bestaande minor1/2-validators weigeren minor3.
Alleen loader-versiebump, snapshot op128 of hergebruik van reserved0
zou huidige consumenten breken. Release ontving de concrete bronpaden/
regels en moet eerst producer/consumer-dispatch voor 1.1/1.2/1.3 en
graphics, snapshotspan/ownership/retained mapping en negatieve fixtures
vastleggen. Hardware is geïnformeerd; geen nieuw ABI-formaat door manager
of reviewer geïmplementeerd.

UI bevestigt keyboard-QMP-pass op baseline74d/v324d5; de nieuwe semantiek
f6ec180 is niet in die guest gelinkt. Release kreeg als volgende integratiestap
de leafcode inhoudsbehoudend opnemen, daarna nieuwe bron/artifacts bevriezen
en UI de gate laten herhalen. Geen screenreader/U04-acceptatie afgeleid.
NAS kreeg expliciete vrijgave native-probe als derde werkelijke crate aan
zijn eigen `nas-core.yml` toe te voegen, met hosttests/fmt/lint/targetbuild.
Target-ELFs zijn geen uitgevoerde NAS-guestprobe. Actuele #73 ae9 heeft bij
managercontrole nog geen checks, oude #71 109 heeft twaalf groene maar
ongerelateerde checks; de nieuwe standalone jobs blijven een mergegate.

Laatste live GitHubcontrole bevestigt #66 merged naar
`94c8908ee363d06a5be51417c5cae635549d566f` op 17:24:22 UTC.
Lifecycle rapporteert 13/13 SUCCESS vóór merge en hele boom identiek aan
c096. De eerdere 12/13-waarneming is daarmee ingehaald; postmerge-CI is
nog niet als groen vastgesteld. Enige lifecyclecoördinator blijft eigenaar
van de mergevolgorde; manager integreert uitsluitend zijn centrale #75.
Live appsteekproef
meldde elf van veertien sessies running, geen input-/planblokkades,
activiteit unknown. Buildteam is gericht hervat op de aparte cold-toolstap;
de niet-running reviewer heeft de H05-trace inmiddels opgeleverd.

Eerstvolgende manageractie: ontvang de exacte up-to-date buildteambron en
koude-recovery-/verifiergates, controleer dat release/UI daarop voortbouwen
en geef de kleinste nog ontbrekende gedeelde interface aan één schrijver.
Blijf mijlpalen/blockers verwerken; een ontvangen statusrapport is geen
stopopdracht aan de teams. Er is geen periodieke achtergrondautomatisering
ingesteld en een verzonden bericht garandeert geen blijvende uitvoering na
sessieonderbreking; hervatten begint met een verse sessie-/GitHub-controle.

Laatste eigenaarsmijlpalen bij main94-integratie:
Lifecycle meldt inmiddels main94-postmerge 10/10 SUCCESS; #70 `68f4dd99`
heeft voorrang, #74 `798ef70` heeft 18 nieuwe native policystages groen
gemeld maar eigen CI/integratie blijft apart. Centrale #75 is op main94
gepusht als `41885e7`, inclusief letterlijk bewaarde PR65-conflictparagraaf/
assessment; #65 pas sluiten na werkelijke mainopname en latestheadcontrole.

Releasevoorstel `43b5670` is gepusht: immutable storedescriptor/opaque
readhandle gescheiden van write-transacties en publisher-/modelprovenance.
AI heeft directe ArtifactRef zonder u64-mapping afgetekend, shortreadfix
volgt in eigen leaf. Manager las het contract en verifieerde rootpayload256;
capacityscope/nul-lengthbeleid moeten expliciet, pure validators zijn geen
kernelcallerbinding. Release bouwt de kleine afgesproken no_std-contractcrate.
Gedeelde descriptor is nu door Apps/AI afgetekend; datum/capacityscope/
nul-length Invalid gecorrigeerd. Release bezit concreet
`crates/artifact-store-core/**`, root-workspacemember en de vrijgegeven
nieuwe `artifact-store-quality.yml` na melding aan build. Hosttest faalt
eerst exit101 door cc; UI geeft bestaande werkende read-only hosttoolenv
door. Compile-only vervangt niet de uit te voeren hosttests.

Nieuwe #73-head `97c26c88` bevat de twee echte cratejobs in één expliciete
workflow; manager las workflow/manifests. GitHub meldt nog geen checks/runs:
uitvoeringsgate blijft open. Zeven bestanden/2280 regels zijn ook nieuwe
protocol-/storecode, dus oude packagecorereview niet hergebruiken.
Lifecycle routeert een nieuwe begrensde review na freeze.

SDK-zeroize in aparte #72-bron4de veroorzaakt een bewezen standalone
ipc-wait-probe-lockbuildfout; security doet alleen de noodzakelijke
consumerlock-updates en daarna relevante nativeherhaling. Build weet dat
security deze locks beheert; geen dubbele schrijvers of locked-bypass.

Buildteam meldt echte reference/gewone/coldchild-gates groen op
`420fac5783fdf86ed6d6464a8f1498190fdf78ef`/v3 `f6d4ca78…`, aparte
cold-toolbranch/buildroots, 14 gerichte en 152 tools-hosttests groen.
Volledige lijst/heropendbeelden en onveranderd diskoracle, archivering/
tamper-negatieven volgen. Expliciet QMPquit-na-save, geen guestcleanstop,
abrupte writepowerloss, Windowscold of byte-reproduceerbaarheid. Release
heeft de concrete integratie-/herhaalopdracht.
Archivering/tamperstap inmiddels opgeleverd als concept-PR #77,
`bef94615244f639a4df3934f54565551a6f0dc37`: alleen drie nieuwe coldtool/
testbestanden en eigen bewijs, main94 inhoudsbehoudend opgenomen.
Offline replay plus acht parenttamper-/twee scope-stopnegatieven slagen
volgens eigenaar, 66 bewijsfiles hashgelijk. CI blijft afzonderlijk;
native uitvoeringsbron420fac en alle genoemde bewijsgrenzen behouden.
Manager draaide aanvullend zelf exact bef9461 in aparte checkout:
14 gerichte cold-hosttests exit0/2,776s, schone checkout daarna verwijderd;
geen VM/artifactreplay. Rapport `teamleader-cold77-host.json`.
R77-review door7f vond twee MEDIUM-brongaten buiten die testdekking:
late post-exitserial PANIC/FAIL-validatie en opnieuw hashen van immutable
parentfirmware. Beide direct bij build voor minimale fix+regressies,
intake bevestigt routering. Mergeblokkade tot herstel/review/gates, geen
claim van een bewezen fout in de opgeslagen run. Lifecyclemelding bewust
gequeueëd na lopend mergewerk nadat priorityadmission NOTSENT gaf.
Opvolging #77: fix `7d66dc2`, producent eerst rood3/6failures, daarna
16gerichte/154tools groen gemeld; 7f sluit beide bevindingen in de beperkte
brondelta. Manager draaide zelf de 16 gerichte tests, exit0/3,772s,
aparte schone checkout daarna verwijderd. Geen nieuwe VM. Build herhaalt
nu exact nieuwe v3949ce3 reference/parent/cold/replay zonder oude bewijsbron
te herlabelen; geen mergevrijgave op alleen bronreview/hosttests.

App-CI-diagnose is inmiddels concreet: eigen REST-read van #73 op9d8a119
geeft mergeable=false/dirty/null-mergecommit; eerdere UNKNOWN ingehaald.
Eén basisupdate na74 voor echte conflictresolutie/CI, geen event-/ready-
bypass of aanname dat nieuwe workflow eerst op main moet staan. Owner
meldt8+10hosttests/lint/fmt/none groen, geen GitHub-run; nieuwe brondelta
blijft afzonderlijk te reviewen. #72-zeroize is door owner geparkeerd na
ReceivedDrop/needs_drop compilefout; #74-prioriteit niet verdringen.
Eigen voorbereidende merge-tree ca45/9d8a119 bevestigt vijf conflicten
(Cargo.toml, README, DECISIONS, PROJECT-STATE, progress.json), exit1;
geen branch-/werkboommutatie. Apps heeft de concrete resolutieoppervlakte.

Release levert daadwerkelijke read-only crate0816a53/bewijs7ab88e1/
ca45-integratie53c2f14, zeven hosttests/lint/fmt/none groen op0816 gemeld.
Manager las de volledige crate/tests/workflow op53c, geen concrete
bevinding in beperkte delta; geen eigen Rusttest, cargo/cc niet op PATH.
Hergates/remote53c en GitHub-CI nog open bij overdracht. Daarna concrete
NAS-probeprofielintegratie, geen backend/authority/productacceptatie.
UI-consumer op aparte branch met exclusieve controller/interaction/directe
docsectie, release afgestemd; generation-bound actiondispatch nog in werk.
Bevroren #68-evidence/main94-merge is niet die nieuwe consumer.
Artifact-store is nu conceptPR #78/head7f4f7bf, ca45; zeven hosttests/
lint/fmt/none ook op53c herhaald, eigen en bestaande CI queued.

TL-REL-NAS-PROFILE rootclaim is concreet toegekend:
release enige schrijver tools/build.py, kernel/Cargo.toml, m3.rs en
benodigde m3/**-profielwiring, nieuwe build_nas/test_nas/directe tests/bewijs.
Geen boot/paging/SDK/ABI/grantverruiming; exacte subset vóór edit melden.
SEC bevestigt geen gelijktijdige NV/kernelCargo/tpm_fixture/buildwrites;
AI behoudt oudWIP maar geen actieve rootwriter. NAS levert bevroren ce52
plus endpoints/grants/footprint; concrete normaal/koud guestauthority-/
diskgate volgt, geenTLS/E2EE. Dit heft de writerblokkade op, geen runtimegate.
Exacte subset bevestigd: Cargo opt-in normal/cold, m3 cfg-dispatch,
nieuwe m3/nas.rs en nieuwe build/testtools; bestaande ELF-envs volstaan.
Geen kernel/build.rs/system.rs/SDK/loaderwijziging. Intakeb6ec464/ca45+ce52
en bestaande builder/core21/volume9/probe5/lint/fmt groen gemeld, geen guest.
Door brede GitHub502/504 zijn finale74checks niet betrouwbaar leesbaar.
Geen merge-/eventbypass of onnodige testrerun. Eerdere wachtgrens voor
NAS-edits is opgeheven voor alleen deze aparte ca45-integratiebranch:
lokale opt-inimplementatie/gates nu, één definitieve basisopname/hergates na74.

Apps-catalogleaf c8c68ee apart gepusht met15hosttests/lint/fmt/none en
drie mutanten gevangen gemeld; geen persistentie/nativeinstall/GitHubCI.
Nieuwe code apart beoordelen, #73freeze9d8 niet door leafwerk muteren.
UI89e200workingconsumer gepusht,21interacties/fmt/check/lint groen gemeld;
nu alleen App-local SDK-Materiaal libwrapper+directe tests vrijgegeven.
Geen externe broker/IPC/surfacecapability/screenreaderauthority; generatie/
node/action/read-only hercheck, draftunsavedtotSave, release geïnformeerd.

AIe57e7d70 gepusht, alleen vijf model-store-contractleafbestanden:
store-issued ArtifactRef/generation direct,3unit+12contract/lint/fmt/scalar-
none groen gemeld. Defaultsha2-none-build LLVM-ICE exit101 blijft expliciet,
scalarroute niet als defaultsucces voorstellen. Geen rootprovider/catalog/
nativeinference/netwerkweigering. Volgende leafstap stale-na-verify/cleanup;
oude acht qsim-rootWIPfiles behouden/nietgestaged/nietverder als writer.

NAS `ba557473` heeft drie werkelijke matrixcrates, host/target/stackfaultgates
groen gemeld; GitHub-CI en echte NAS-QEMU blijven open. Security NV
`0041a741` bewaart zes echte proeven op `9ae314d`/c532 inclusief counter2
na volledige QEMU+swtpm-stop en wrongauth/missing; geen lineage/powercut/
write-ahead/main94-claim. Security werkt eerst #74 inhoudsbehoudend op main94
bij en herhaalt daar de policygates.

Netwerk listen `91cda779`/code `c643b2d`: 22 hostmodellen/fmt/lint/target
groen gemeld, actieve native TCP-gate faalt vóór probe op kandidaat én
baseline0ade. Geen native listenacceptatie; rootbootblocker bij release.
TLS/no_std-feasibility gaat onafhankelijk verder, geen fictieve oude port.

TL-INTAKE heeft de ontbrekende S09-vindplaats onderzocht: claim06d46151
historisch aangetroffen, geen implementatie/capture in geraadpleegde
actuele boom/refs/gerichte historie. Geen totale-afwezigheidsclaim.
Concrete vervolgbuildertaak TL-S09-COPY-TEST: exclusief nieuw
`services/volume/tests/independent_copy.rs`, twee onafhankelijke SimDisk/
Service-instanties, vaste revisie kopiëren, bron laten vallen, doel koud
recoveren, stale/vol/quota negatieven. Bestaande Client/SimDisk hergebruiken;
geen common/client/service/kernel/SDK/format-/voortgangswrites.
Gerichte cargo-test + bestaande history-/volume-coregates, commit/push
met host-only bewijsgrens. Echte tweede-volumeportals/grants bij release;
geen S09-acceptatiepunten voor deze hergebruikregressie.
Opgeleverd: `8a14770d6a719c3a122c54cb774db58ce9b1a728` op main94,
uitsluitend deze 181-regelige testleaf, drie nieuwe/40 totale volume-core-
hosttests en history/Clippy/fmt groen gemeld, remote exact/schoon.
Manager las de volledige test rechtstreeks uit Git: aparte doeldisk,
8365 bytes, koude recovery na bronverlies en onveranderde doelbytes bij
stale/quota/full. Geen eigen testherhaling of native/productclaim.
Lifecycle routeert dit apart, niet via centrale documentatie-PR #75.

## Historische uitvoeringsregistratie vóór de managerverdeling

## Historische integratie en schrijfgebieden uit #66

Integrator `simon/native-test-version`: gepusht bewijscheckpoint `c532ba88`,
geteste code `eabc27ba879fd3b5960863ae8eede800c5045a57`, runtime v3
`a1998a126a3a1ccc18523925a422912d934c14bab865b3d7df346862a2c251a6`.
De onderstaande oudere toewijzingen zijn historie, geen actieve reserveringen.
IPC-wachten/consumenten, B02 en native TPM zijn geïntegreerd; normale GUI,
save/list/reopen en koude probeheropening zijn op hun eigen bron bewezen.
Alle zes volledige gewone zevenprocessenframes zijn nu op eabc27b native
bytegelijk aan de opnieuw uitgevoerde referentie, met diskorakel na raw0.
Bewijs: `verification/full-release-interactive-fixed/`.

| Taak | Eigenaar / exclusieve writes | Toestand en bewijsgrens |
|---|---|---|
| WINDOWS-INTERACTIVE-1 | peer `Keuze native os-functionaliteit`; alleen eigen `docs/verification/windows-qemu-interactive-1/`, eigen tijdelijke adapter/bundel/state | Vaste c532ba8-kopie in eigen worktree. Lokale ZIP en echte Windows CheckOnly/start/readiness gemeld; bediening loopt nog, geen geverifieerde eindclaim of publicatie |
| INTERACTIVE-COLD-1 | integrator; voorlopig alleen sessieartifacts en eigen gegenereerde cold-experiment-runs, daarna zo nodig `tools/run_materiaal_preview.py` en directe tests | Eerste exacte lijstvergelijking rood: verse Ready-status verschilt terecht van vorige Saved-status. Tweede verkennende run4019c620f8 bewijst volledig bytegelijk reopened-frame en onveranderde disk na QMP-quit/raw0, maar certificeert de initiële lijst niet (expliciete vaste wachttijd). Nog geen volledige koude acceptatie; renderer-afgeleid lijstorakel en strikt parent/artifact/trace-replay vereist. Geen kernel-/app-/SDK-writes |

De Windowspeer heeft de vaste build read-only gekopieerd en alle bron/copy-
hashes gecontroleerd; verder integratorwerk verandert haar vaste bron niet.
Geen parallelle schrijvers voor de gedeelde voortgangsmeter/status.
De integrator heeft de Materiaal-/toegankelijkheidssessie expliciet van deze
runnerreservering op de hoogte gebracht. Score blijft34%;50% open.

## Uitvoeringshistorie

Afgeronde fundamentstap `TPM-NATIVE-1`, eigenaar `simon/native-test-version`, vanaf
`cf02055`: alleen vaste supervisor-MMIO-adapter en read-only TPM2-property-
proef met echte PIT-polls, afzonderlijk absent-geval en bestaande volledige
CPL3-wachtacceptatie. Exclusieve writes: root/kernel Cargo-manifest/lock,
exacte pagingseam, M3-TPM-fixture/testhook, waitfixture-observer,
`tools/build.py`, `tools/run.py`, nieuwe `run_tpm.py`/hosttest en bewijsdocs.
Geen keyring/system-opslagwijzigingen; actieve PR #64 bezit dat gebied.
23 coretests, 4 harnastests en scalarbuild/ELF-inspectie groen. Beide echte
native VM's slagen op `4b09869`: propertyquery/release en nul-write-absent-
quarantaine, met volledige CPL3-wachtacceptatie. Eerste rode poging behouden;
`verification/native-test-version/tpm-evidence.json`. ADR-052; fundament
voor FR-B01/FR-O05, geen gesloten criterium. Integrator herhaalt nu de
gedeelde gates/proefversie; nieuwe CI-step nog niet uitgevoerd.
Voorafgaande sessiestap: alle zes echte K05-cases, inclusief twee Waiting-
kills en schoon hergebruik, groen op `586c33c`; brongebonden bewijs in
`verification/native-test-version/wait-lifecycle-evidence.json`.
De normale/koude native GUI en FPU/wacht zijn eveneens inmiddels groen op
hun eigen bron; oude rode queue-omschrijvingen hieronder zijn historisch.

## Peerhistorie — Windows-QEMU-testversie, nu geïntegreerd

`simon/windows-qemu-test-version`, basis `ad2b981`; codecheckpoint `f777f7a`.
`TASK-WINDOWS-QEMU-TEST-VERSION` voegt een gecontroleerde ZIP-packager,
PowerShell-launcher en bron-/hashgebonden QMP-previewrunner toe voor het echte
zeven-app `native-system-materiaal`-profiel. De opgegeven map
`C:\Program Files\qemu` is het ingebouwde standaarddetectiepad. De launcher
vereist QEMU 10.2.1+, SDL en `rng-builtin`; QEMU zelf wordt niet meegeleverd.

Hosttests (31: packager, transcript/frame-orakel en bestaande Materiaal-probe),
strikte preflight en normale Materiaal-build slagen; de laatste build is
waarschuwingvrij. De eerste QEMU-preview faalde door een onterechte vaste
Source/Policy-volgorde in het hostorakel. Na correctie slaagt de echte run
in 3,874 s: QEMU exit 0 na QMP-quit, 800×600 en 310 kleuren.
Bewijs: `verification/windows-qemu-preview/`. Bundelgeneratie, GitHub-artifactjob en
Windows-uitvoering staan nog open; de uiteindelijke bron wordt eerst met
`simon/native-test-version` afgestemd. De lokale QEMU-build heeft QMP en
`rng-builtin`, maar uitsluitend de `none`-displaybackend, zodat dit geen
Windows-UI- of SDL-bewijs is. Geen volledige-release- of roadmapcriteria
afgetekend; rubric blijft 34%.

Windows-preflightaanvulling: echte PowerShell bereikt de opgegeven
QEMU-installatie (11.1.0), met SDL en `rng-builtin`. Vijf launchertests met
synthetische bestanden slagen zonder VM of state; `-CheckOnly` valideert
bundelgrootte/SHA-256. Bewijs: `verification/windows-qemu-preview/windows-launcher.md`.
Dit is geen Windows-gaststart of handmatige bediening. PR #65 is concept.

Packager verifieert ook de ingelezen kernel-/loaderbytes en bron opnieuw
vlak vóór archivering. 33 hosttests groen, inclusief artifactwissel na
initiële validatie: fout zonder ZIP. Geen OS-bundel gemaakt.
Historische CI41089090576 op f777 is rood (herstelde readinessvolgorde);
huidige-head-CI blijft open.

Eerstvolgende gate:
`. .tools/nieuw-os/rtc-env.sh && PATH="$PWD/.tools/nieuw-os:$PATH" NOS_BUILD_DIR="$PWD/build/windows-qemu" NOS_TARGET_DIR="$PWD/target/windows-qemu" CARGO_TARGET_DIR="$PWD/target/windows-qemu" python3 -B -u tools/run_materiaal_preview.py --bios-rom .tools/nieuw-os/emulator/usr/share/seabios/bios-256k.bin --vga-rom .tools/nieuw-os/emulator/usr/share/seabios/vgabios-stdvga.bin`.

Afgeronde deelstap `RTC-NATIVE-1`: `simonvanieperen-expert-disco`, basis
`5c334f9ac3100f1c0e503e4f0ac4353a4a35154e`. De opt-in `native-rtc`-bootstrap,
`kernel/src/m2/rtc.rs`, RTC-buildkeuze, RTC-runner/hosttests/workflow en
`RTC-WALLTIME.md` zijn op die bron bewezen. Uitvoeringsbron `c02675e`: vijf
RTC-kalenderboots, zeven M2-regressie-VM's, M6c-smoke en hostgates slagen;
firmware-epoch/2100 blijft expliciet geweigerd.

Afgeronde deelstap `TASK-NATIVE-CLOCK-1`: opt-in ABI 1.9 syscall `0x73`,
SDK-type met expliciet `UnauthenticatedRtc`, en echte ring-3-positief- en
negatiefproef in de bestaande systeemapp. De native QEMU-gate slaagde met
gecontroleerde UTC-basis, twee echte reads, monotone tickcontrole en
`-22`-argumentweigering; brongebonden bewijs staat onder
`verification/system-clock/`. Het is een directe FR-N03/FR-S03-afhankelijkheid,
geen vertrouwde tijd, TLS-klok of gesloten roadmapcriterium.
[Brongebonden bewijs](verification/rtc-native/README.md); concept-PRreview,
geen samenvoeging of volledige productrelease geclaimd. De andere
zichtbare sessie beheert uitsluitend een root-workspace-doctestblokkade.
De oudere agentnamen hieronder zijn historische toewijzingen, geen actuele
claim dat hun agents draaien. `main` op deze basis bevat PR #58/#60/#61;
de actuele gewogen score is 34%, niet de historische 32% hieronder.

Integratie-eigenaar root, branch `codex/full-test-release`; laatste native foundationbron `844ec40`, gezamenlijke bron `9596abc` (negen CI groen, GUI rood) in `codex/native-product-integration`, PR #60.
[Volledige opdracht/matrix](FULL-TEST-RELEASE.md) → [actuele uitvoering en exacte hervatstap](PROJECT-STATE.md).
Werkpakketten zijn implementatiestappen; de 62 vaste `FR-*`-criteria blijven open. De gewogen ontwikkelscore blijft 32%.

| Werkpakket | Toestand | Eigenaar / exclusieve writes | Acceptatie | Interface en bewijsgrens |
|---|---|---|---|---|
| B02-PERSIST-1 | geïmplementeerd; native FR-I01-gate (normaal + koude herstart) zie PROJECT-STATE | root; keyring wire/adapter, system init/app/protocol, system.rs manifest, run_system/volume_disk/capture | FR-O05, FR-B02 (deel), FR-O01 | KEY1-opcodes 11/12 alleen via apart admin-ingress (ADR-051); keystate-commit vóór ciphertextcommit; vloer is gedemonstreerd mechanisme, geen vertrouwd anker; B02 blijft open |
| TASK-NATIVE-CLOCK-1 | implementatie gebouwd; native QEMU-gate volgt | root; RTC-bridge, M3 syscall, SDK, systeem-app en `run_system_clock.py` | FR-S03, FR-N03 | ABI 1.9 syscall 0x73; echte ring-3-reads en foutargument; uitsluitend `UnauthenticatedRtc`, geen TLS-/productklokclaim |
| TASK-DOC / TASK-CONTEXT | opdracht en aanvullende context vastgelegd | root; FULL/status/queue/rubric | alle | oorspronkelijke scope, stabiele ID's, formats/compatibiliteit en 100 cold/24 uur behouden |
| TASK-I01 / TASK-INT/GIT | foundation geïntegreerd op 844ec40 | root; gedeelde ABI/build/locks/Git | alle geraakt | negen CI groen op exact die bron; geen volledige productacceptatie |
| TASK-A03/R1 / TASK-KEYSDK-NATIVE | document-v7 en keyclient geïntegreerd | root | FR-A02, FR-A13, FR-O01–FR-O05, FR-B01 | native deelproeven; lost-BEGIN, owner-death en veilige disk-/sleutelpersistentie open |
| TASK-TCP | eigen native TCP-keten en cold pair groen | root | FR-N01, FR-N03, FR-H03 | HTTP/FIN/refused/reset/retry/timeout en strict capture; configuratie/TLS/appgrants open |
| TASK-FPU/CLOCK / TASK-MEMORY / TASK-USER-ALLOC | geïntegreerd en native deelproeven groen | root | FR-K02, FR-K03, FR-L02 | eigen heap/guard/NX/OOM/FPU/context/clock; geen SMP- of algemeen hardwarebewijs |
| TASK-DISPLAY-INPUT / TASK-DISPLAY-PROBE / TASK-DISPLAY-CI | geïntegreerd en native CI groen | root | FR-U01, FR-K03, FR-H01 | echte PS/2/QMP, pixels en negatieve cap/lifetimegevallen; volledige desktop open |
| TASK-INPUT / TASK-INPUT-DAEMON / TASK-SURFACE/APP / TASK-GUI-INIT | componenten in GUI-branch | root; integratie shared files | FR-U01–FR-U05, FR-A02, FR-H01 | echte Source/Policy/Comp/Init/App-processen, begrensde IPC; brede sessie en crashherstart open |
| TASK-GUI-QMP / TASK-MATERIAAL-NATIVE | 5e echte Materiaal-/editorframes; 1fd regressie; keten rood | root; root-materiaal | FR-U01–FR-U05, FR-A02, FR-O01 | 200 seconden vast; Save/Reopen/afsluiten nog niet bewezen |
| TASK-REV-APP | gevonden stale-ready/entryfouten hersteld | gerichte onafhankelijke review afgerond | FR-U01, FR-A02, FR-K03 | host-/bronreview; volledige native GUI-keten blijft vereist |
| TASK-RANDOM-NATIVE / TASK-KEYRANDOM | geïntegreerd, vier echte gevallen opnieuw in CI op 844 | root | FR-B01, FR-N03 | onderhouden CSPRNG, echte virtio-RNG, quota/clock/cancel/revoke; persistente productkeys/TLS open |
| TASK-IMAGES | decoder native groen op 844, onafhankelijk hergevalideerd | root; image-codec component ade6 | FR-A03 | PNG/JPEG, heap/stack/pixels/negatieven; viewer, volledige formaten, native OOM en mid-decode cancel open |
| TASK-FOUNDATION-CI | alle vijf native stappen echt groen, 844 | root | FR-B01, FR-A03 | afzonderlijke builds, geen skips of genegeerde fouten; raw artifactmetadata bewaard |
| TASK-TPM-CRB | geïntegreerde pure transport-/commandocore | root; component 6bed190 | FR-B01, FR-B03, FR-O05 | host-/targetbewijs; native MMIO, virtuele TPM, ankertransacties en sleutelherstel open |
| TASK-RTC / TASK-RTC-V | core geïntegreerd, review afgerond | root | FR-N03, FR-S03 | 11 hosttests en onafhankelijke datumvectoren; UnauthenticatedRtc is geen vertrouwde certificaattijd |
| TASK-TLS-PORT | component 34191bb in eigen tls-port worktree | wachtrij voor verdere integratie | FR-N03, FR-P01–FR-P06 | no_std rustls/ML-KEM/mTLS host en target; native random/signer/trusted-time/TCP-keten open |
| TASK-BROWSER-PORT | concreet Chromium/Gecko-onderzoek a2085b9 gereed | wachtrij voor rootintegratie | FR-A05 | echte portproeven geblokkeerd, engine_selected null; verplichte browser blijft open |
| TASK-CAP-PERF / CAP-DIGEST-TCG-2 | productieherstel geïntegreerd; gezamenlijke GUI-keten blijft rood | kernel_storage; capability-core en digestcontract, basis 952e | FR-K03, FR-U01 | 5337500 geïntegreerd: reference sneller onder TCG; alle controles behouden; echte gezamenlijke GUI-herhaling volgt |
| GUI-BENCH-REVIEW-1 | P2-harnasfix geïntegreerd en onafhankelijk groen | security_release; run_cap_bench.py en test_cap_bench.py, basis 342387e | FR-K03, FR-U01 | exacte native artifacts correct; stale-index/inputdrift/initial-VARS/disk-readorder hersteld; native herhaling volgt |
| TASK-CALCULATOR / CALCULATOR-REVIEW-1 | component 0254206 geïntegreerd, onafhankelijke review groen | root; shared multiappgrants nog open | FR-A08 | 17+4 IPC-hostgroepen en 26424 exacte oraclecases; target groen; echte multiapp-/QMP-bediening open |
| MULTIAPP-CONTRACT-1 | actief | native_ui_apps; nieuw NATIVE-MULTIAPP.md, basis 48b | FR-U01–FR-U05, FR-A08 | twee echte CPL3-apps, minimale grants, focus/dirty/retire/lifetime; root bezit gedeelde code |
| PRODUCT-MERGE-REVIEW-1 | afgerond; configuratie-P2 gericht hersteld | git_build_audit; c91714b | alle gedeelde interfaces | gerichte controle foundation/GUI/build/manifest, geen root-WIP-writes |
| IPC-WAIT-1 | in uitvoering; WIP nog niet native getest | kernel_storage; core/SDK/m3/waitmodule/feature, basis 9596 | FR-K03, FR-U01 | 16 endpoints, absolute deadline, veilige wake/revoke/lifetime en terugkeerbare IRQ-idle; rootnativegate vereist |
| WAIT-CONSUMERS-IDLE-1 | in uitvoering; SDK-afhankelijkheid open | security_release; Key/Volume service en nieuw contract | FR-K03, FR-B01, FR-O01 | alleen echte idlepass; oude ABI/devicepolling en no-replay/deadlines behouden |
| MULTIAPP-CODEC-1 | hostchecks groen, target en integratie volgen | native_ui_apps; nieuw sdk/desktop-control en contract | FR-U01, FR-A08 | APPCTRL2 en beperkte Switch, geen native Pause/Resume-blitclaim |
| NATIVE-TLS-PROBE-1 | wachtrij, threadlimiet verhinderde start | nog niet actief | FR-N03, FR-P01 | volgende vrije worker: echte native TLS-/signerketen, geen voorspelbare RNG of hostsucces |

Volgend verplicht werk blijft zichtbaar: volledige compositor/sessie/toegankelijkheid,
alle apps en appbeheer, veilige persistente opslag en TPM-ankering, netwerkconfiguratie
met TLS, NAS/providersynchronisatie, lokale inference, boot/PQC, updates/herstel en
integrale productvalidatie. Dit zijn geen naar later verplaatste releasevereisten.
Fysieke en menselijke gates blijven open; onafhankelijk bron-/VM-werk gaat door.
Uit historische queues hieronder mag geen actieve agentcapaciteit worden afgeleid.

## Historische registratie

# Werkqueue — 29 oktober 2026

Actuele geteste bron `53e0d57af99c9673c97bc900fae15b98e08c8ca3`, PR #54.
Root integreert; onafhankelijke worktrees en buildmappen zijn gebruikt.
Historische registraties hieronder zijn geen actieve capaciteitsclaims.

| ID | Toestand | Eigenaar / exclusief gebied | Resultaat / afhankelijkheid |
|---|---|---|---|
| B01-R1 | afgerond, geïntegreerd getest | keyring_repairs: services/keyring/** | Blijvende Health, monotone intrekking; 15 tests en extra tegenvoorbeelden groen |
| B01-R2 | afgerond, geïntegreerd getest | entropy_timeout: keyringbroker/app/harnas | Echte timeout/reset-ACK, aparte herstart, native oude-blobintrekking; 5/5 gate |
| B01-V | afgerond | security_verifier: alleen lezen | Geen blocker; 615 hosttests plus lint/fmt/voortgang groen |
| S08-R | afgerond, geïntegreerd getest | storage_resume: m6c-/volumebroker | Err-completionmeting; 13/13 gezamenlijke gate; aparte targeted cases groen |
| ENV | afgerond | emulator_setup: uitsluitend .tools/** | Officiële lokale QEMU10.0.13/OVMF, preflight groen |
| INT | lokale controles afgerond | root: docs/voortgang/bewijs/GitHub | 27 VM-runs groen; geschikte merge uitsluitend na finale PR-checks |
| K05-SLOTS | lokaal afgerond, eindintegratie volgt | root + storage_resume, security_verifier | 5/5 native, 21 negatieve controles; exact56b576b, geen nieuw punt |
| B02 | gereed om te bouwen | volgende eigen branch | Persistente geauthenticeerde sleutel-/volumeopslag; live herstel is geen koude antirollback |
| S09 | wacht op bron of nieuwe implementatie | niet toegewezen | Historische lokale branch niet remote gevonden; geen geïntegreerde implementatie claimen |

## Vroege productvalidatie — consumentenadoptie

De literatuurverkenning in
[CONSUMER-VALUE-2026-10-28.md](research/CONSUMER-VALUE-2026-10-28.md)
levert vier onderzoeksstappen naast de native bouw. Deze taken gebruiken geen
agentcapaciteit totdat zij werkelijk worden uitgevoerd en tellen niet als
roadmappunt zonder het bewijs van het betreffende acceptatiecriterium.

| ID | Toestand | Onderzoeksdoel | Afhankelijkheid / grens |
|---|---|---|---|
| PRX1 | gereed voor onderzoek | 12–18 taakinterviews/-observaties om twee of drie kerntaken en overstapvoorwaarden vast te leggen | Praktische pilotomvang; geen statistische representativiteitsclaim |
| PRX2 | wacht op PRX1 en bruikbaar DV-01-prototype | circa 24–36 deelnemers; terugvinden, twee gereedschappen, herstel en rechten vergelijken | Taakprestatie en aantrekkelijkheid apart meten; definitieve omvang na pilot/poweranalyse |
| PRX3 | wacht op stabiele kernflow | eerlijke productbeloftes vergelijken op zelfstandig uitgevoerde taak, niet alleen klikgedrag | Zelfde prototypekwaliteit, prijs-/beschikbaarheidsinformatie en vooraf gekozen primaire uitkomst |
| PRX4 | wacht op bruikbare afgebakende workflow | vierweekse vroege veldproef met vrijwillig hergebruik, fouten, herstel, hulp en terugvalredenen | Geen claim van langdurige retentie; 90-/180-dagenvalidatie blijft latere marktproef |

DV-01 in [PRODUCT-SPEC.md](implementation/PRODUCT-SPEC.md) blijft het
geïntegreerde Materiaal-scenario. U06 blijft open totdat representatieve
gebruikers de afgesproken kerntaken daadwerkelijk uitvoeren en relevante
frictie binnen productbudgetten is opgelost.

## Historische registratie

# Werkqueue — native M6b-opslagfouten

28 oktober 2026, basis `1f24f99`, branch `codex/m6b-storage-errors`.

| ID | Eigenaar | Exclusieve writes | Status |
|---|---|---|---|
| K07 | m6a_architect_review | kernel/Cargo.toml, kernel/src/m2/m3/m6b.rs, userspace/m6b_probe.rs | generieke Unknown-observerreparatie en twee statussnapshotinjecties klaar |
| G07 | root | tools/tests/docs/Git | 31 hosttests, 22 groepen/18 VM en capture exit 0; nieuw bewijs apart gearchiveerd |
| R07/V07 | m6a_environment | alleen lezen | bronreview geen blockers; onafhankelijke hash-/schijf-/recoveryaudit |

Vereist: geen mutatie-ACK of live-stateclaim na onzekere I/O, geen replay,
GET/PUT/DELETE fail-closed, canoniek foutantwoord en overlevende processen.
Twee gerichte native foutcases en de volledige M6b-gate met beide koude
foutrecoveries slagen. Historisch bewijs behouden. Geen nieuwe rechten,
dependency, schijfformaat of architectuur. Bron `592dccf`; bewijs en nog open
backend-/timeout-/LIMIT-grenzen staan in PROJECT-STATE.md.

## Oudere werkqueue — historisch
# Werkqueue — M6d native foutafhandeling en productvoortgang

## Afgeronde uitvoering M6d — 28 oktober 2026

Uitvoeringsbron `88e6c04be218b8b09ea4f0a226f6b9719da706a6`, branch
`codex/m6d-io-recovery-roadmap`, PR #11. Contract:
[M6D-IO-UNCERTAINTY.md](implementation/M6D-IO-UNCERTAINTY.md).

| ID | Werkverdeling | Afgerond resultaat |
|---|---|---|
| A01/A02 | architecture_next_step / kernelobserver, native probe, integratie twee hosttools | Unknown zonder valse ACK; acht native foutfeatures; upstream hostlogbinding behouden |
| A03 | architecture_next_step / M6b-capture en bijbehorende tests | begrensde alternatieve bestemming; drie negatieve/positieve bestemmingschecks |
| D01/D02/D03 | progress_roadmap / scoremodel en historische documentintegratie; later alleen advies | vaste 100 punten, bron-/bewijsgrenzen en concreet herstelvervolg |
| R08/R09 | native_review / aanvankelijk alleen lezen, daarna zelfstandige audit buiten de repo | twee vroege harnasbevindingen hersteld; 31 M6d-runs, 274 artifacthashes en 16 images gecontroleerd |
| A04 | architecture_next_step / afzonderlijke read-only M6b-bewijsaudit | 14 M6b/M1-runs, 121 artifacthashes, zeven volumes en zes cold-links gecontroleerd |
| G08/V08 | root / gedeelde interfaces, integratie, uitvoering, score en Git | 35/31 plus 18/14, beide captures en normale eindbuild geslaagd; S07 geaccepteerd, 13% → 14% |

Eén schrijver per bestand en één build/QEMU-slot. Geen nieuwe OS-kernelbasis,
capabilityverruiming, schijfformaat of externe dependency. Exacte uitslagen en
bewijs: [M6D-ACCEPTANCE.md](verification/M6D-ACCEPTANCE.md). Iedere volgende
opdracht herbeoordeelt de vaste meter volgens AGENTS.md.

### Eerstvolgende native stap — begrensde catalogusrecovery

Voorgesteld, nog niet gebouwd of als punt geteld:

1. Eén geautoriseerde supervisor mag alleen deze catalogusdienst vervangen;
   gewone apps krijgen geen reset-, processtart- of blokrechten.
2. Oude servicegeneratie, endpoints en blokhandles intrekken vóór vervanging.
   DMA-geheugen pas hergebruiken na bevestigde reset; anders quarantaine.
3. Nieuwe instantie herstelt de echte journal zonder write-replay. Apps blijven
   actief, krijgen binnen hun bestaande rechten nieuwe endpoints en lezen de
   actuele revisie terug; Unknown wordt nooit alsnog een verzonnen succes-ACK.
4. Herstartbudget begrenzen; stale caps, onbevoegde supervisor, corrupte journal,
   herhaalde uitval en ontbrekende reset-ACK als negatieve gevallen opnemen.
5. In dezelfde VM testen en de relevante M6d-regressie herhalen. Deze beperkte
   proef voltooit niet automatisch algemeen dienstbeheer K05 of opslagherstel S08.

Native Materiaal, grotere documenten en ruimtehergebruik houden daarnaast hun
eigen productcriteria. Geen sleutelgeneratie zonder native entropybron.

## Historische M6c-queue

### Historische M6c-uitvoering — 28 oktober 2026

Basis `bb22fdb`, branch `codex/m6c-named-catalog`. M6a en M6b zijn al afgerond
volgens de versiegebonden bewijsarchieven in PROJECT-STATE.md. De oudere queue
hieronder bewaart uitsluitend de toenmalige werkverdeling.

| ID | Eigenaar / schrijfgebied | Afgerond resultaat |
|---|---|---|
| C07-core | catalog_core / services/catalog core en tests | namen, v4-wire, v2-journal, catalogus; 30 hosttests en no_std groen |
| C07-native | catalog_native / nieuwe serviceadapter, probe en kernelobserver | finale native gate: 19 groepen / 15 VM-invocaties inclusief M1 |
| G07 | root / gedeelde config, tools, harnas en docs | onafhankelijk schijforakel; 33 hosttests; checkpoint bc4cd104 vóór lange controles |
| R07 | context_review / alleen lezen | 15 scenario's van echte adapter via host-ABI-shim; geen native devicefoutinjectie |
| V07 | root + reviewers | M6c-capture, M6b 18 groepen / 14 VM, acht volumes en 130 artifacthashes geaudit; brongebonden integratiecontrole |

Alle eindgates en captures hebben exit 0. Bron, exacte commando's, host-/native
grenzen en bewijslinks: [M6C-ACCEPTANCE.md](verification/M6C-ACCEPTANCE.md).
De M6c-eindbuild is bytegelijk aan de twee normale eindgate-runs. De reviews zijn
AI-reviews, geen menselijke audit of formeel bewijs.

Vervolg: native Materiaal-koppeling en beveiligingsafhankelijkheden
uit VERIFIABLE-SECURITY-AND-PQC blijven afzonderlijke stappen; geen automatische
sleutelgeneratie zonder entropy of grafische/native NAS-claim door deze catalogus.

## Historische queue vóór afgeronde M6b

## Parallel N01 — internet en persoonlijke NAS

Gebruikersuitbreiding 27 oktober 2026; privacycontract revisie 3. Bron vanaf `a96ad45`, branch `codex/nas-private-sharing`.

| ID | Gebied | Status/bewijs |
|---|---|---|
| N01-C | services/nas/ | echte hostencryptie, ciphertextstore, quota, revoke/evict en netwerkbeleidsmodel; 50 tests groen |
| N01-P | ui/prototype/ | drie NAS-flows en afzonderlijke lokale beheerrol; 39 tests groen, nieuwe visuele browsercontrole open |
| N01-R | alleen-lezen review | vier concrete bevindingen hersteld; gerichte hercontroles groen |
| N01-G | root/docs/Git | herziene productgrens, ADR-027, integratiebewijs 89 tests + demo |
| N02 | services/nas/transport.mjs | begrensde echte hybride TLS-hostproef; actuele resultaten onder verification/pqc-transport/ en PROJECT-STATE.md |
| N03 | toekomstige native integratie | netwerkstack/TLS-port/identiteit, clientkeys/multi-device, NAS-rol en socketcapabilities nog open |

Resultaten tellen niet als M6a-/M8-QEMU- of echte-internetbewijs. Eerste kernelvervolg blijft de hieronder beschreven M6a-poort.

De aangeleverde formele-/PQC-blauwdruk is verwerkt in `implementation/VERIFIABLE-SECURITY-AND-PQC.md`: V01 voor machinegecontroleerde capability-invarianten, P01–P04 voor native build/entropy/image-/opslagintegratie en Q02 voor een afzonderlijke Grover-onderwijsproef. Deze taken zijn gespecificeerd, niet reeds uitgevoerd. N02 voegt geen eigen cryptografische tunnel of bestaande OS-kernel toe.

27 oktober 2026, basis `448974b`, branch `codex/m6a-write-recovery`. Gekozen invariant/ABI/cases vóór implementatie: implementation/M6A-WRITE-RECOVERY.md.

| ID | Eigenaar | Exclusieve writes | Status |
|---|---|---|---|
| K06 | kernel_m2 | kernel/,kernel-core/,drivers/virtio-blk/ | bron bevroren; 16 hosttests en gerichte QEMU/cut/recoveryproeven groen |
| C06 | quantum_contract | capability-core/,virtio-core/ | 30 + 21 tests/fmt/Clippy/no_std exit 0; bron gepauzeerd |
| U06 | product_acceptance | services/block/,nieuwe M6-userspace | 20 hosttests + 12 artifacttests exit 0; echte dienstbron gepauzeerd |
| G06 | root | config/tools/tests/docs/Git | guarded writable image/crashharnas/capture,19 hostcontroles groen; volledige poort gereed |
| R06 | C06/U06 | alleen lezen | gerichte transport- en zes harnas/capturebevindingen hersteld/gesloten |
| V06 | later verifier | alleen lezen | finale M1–M6-poorten, echte cuts/recovery en hashbinding |

Eén schrijver per bestand, maximaal drie agents en één gedeeld build/QEMU-slot. Geen nieuwe externe dependency. Na elke afgebakende stap bron/bewijs/open punten privépushen; noreply-afzender alleen per Git-commando, geen globale configwijziging. Gerichte M6-proeven groen; finale 54 groepen / 49 VM-poort en regressie nog uit te voeren. QEMU-procescrash is geen fysieke stroomuitvalclaim.

## M5 — behaalde basis (historisch)

27 oktober 2026. Bron `7af544a`, basis `c630c59`, branch `codex/m5-block-service`, concept-PR #3. Definitieve fingerprint v3: c7455ebe2dfcd0e4337d058f440712d60b3ae600d2953c385ee81ba6d69a0307.

| ID | Eigenaar | Gebied | Status / bewijs |
|---|---|---|---|
| G05 | root | config/tools/docs/Git | geïntegreerd; preflight exit 0, private checkpoints, einddocs |
| K05 | kernel_m2 | kernel/,kernel-core/,drivers/virtio-blk/ | gereed; M5 25 groepen/20 VM exit 0 |
| C05 | quantum_contract | virtio-core/,capability-core/ | gereed; 12/26 tests, fmt/Clippy/no_std exit 0 |
| U05 | product_acceptance | services/block/,M5-userspace | gereed; 9 protocol- en 11 artifacttests exit 0, echte gastdienst |
| R05 | twee reviewers | alleen lezen | herstelpunten gesloten; ongeïnjecteerde randen expliciet |
| V05 | root/verifier | ruwe evidence/hashbinding | 77 groepen/64 VM plus M5-eindstart; alle 65 VM en 279 artifacthashes geaudit |

Opdrachten, resultaten, grenzen en hervatroute: PROJECT-STATE.md en verification/m5/ACCEPTANCE.md. Eerste mislukte gate blijft failed bewaard; de finale bron is opnieuw getest. Bestaande UI/quantum en M1–M4-baseline behouden.

Volgende stap: de M6a-bronpoort en regressie hervatten vanaf deze checkpoint. De M6a-specificatie, huidige teststatus en open punten staan in PROJECT-STATE.md. Geen echte hardware/disks/NAS/keys, betaalde infrastructuur, publieke release of merge zonder passende opdracht. Licentie vóór publicatie blijft open.
