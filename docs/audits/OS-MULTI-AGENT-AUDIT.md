# Nieuw OS - multidisciplinaire audit

Datum: 3 november 2026, Europe/Amsterdam.

**Resultaat:** alle zestien specialistische deelonderzoeken zijn opgeleverd.
Nieuw OS heeft een eigen kernel, native diensten en betekenisvolle historische
VM-deelbewijzen. Een volledige persoonlijke, verbonden, toegankelijke,
herstelbare en onderhoudbare productrelease is niet aangetoond. Er is één
concrete securitybronbevinding met hoog vertrouwen, naast productgaten en
specifieke archiefproblemen. Deze categorieën worden niet bij elkaar opgeteld
als kwetsbaarheden.

Dit is een gecoördineerde AI-audit met afzonderlijke onderzoekscontexten,
**geen onafhankelijke menselijke audit**, formeel veiligheidsbewijs,
juridische conformiteitsverklaring of releasevalidatie.

## 1. Repository, bron en werkwijze

| Gegeven | Waarde |
|---|---|
| Repository | `simonvanieperen/nieuw-os` |
| Onderzochte bronbranch | `copilot/add-os-specialist-agents` |
| Geauditeerde broncommit | `beafdedfc188154fe7285848fbd411e651013f54` |
| Audit-/rapportcommit | `6929b986440fe659fa1d45167f476313abb6c6d3` |
| Parent van auditbron | `69809c2cb53dfb3bd77abe5352892810fba6e82d` |
| Titel broncommit | `Add OS specialist agent profiles` |
| Auteur/organisatie en datum broncommit | S.W.J.B.M. van Ieperen, 3 november 2026 |
| Vastgepinde bron-URL | [GitHub-bronsnapshot](https://github.com/simonvanieperen/nieuw-os/tree/beafdedfc188154fe7285848fbd411e651013f54) |
| Source-v3 via bestaande `tools/common.py` | `fe0fe328ef3909e2dab4386c0ba77ebf3cf6a8244301e245863b21debfde4c99` |
| Integratiewerkmap | Eigen worktree; lokaal pad niet gepubliceerd |
| Wijzigingsscope | Alleen dit hoofdrapport en zestien deelrapporten |
| Productscore | Bestaand 41/100; ongewijzigd, niet opnieuw inhoudelijk geaccepteerd |

De technisch geladen OS-customagents konden niet worden gestart: de host
adverteerde hun selectors niet. Na expliciete toestemming van de gebruiker
zijn gewone projectsessies gebruikt die ieder hun eigen profiel als
instructietekst lazen. Dit is niet hetzelfde als technisch geladen
customagents. Er liepen maximaal drie inhoudelijke analyses tegelijk; iedere
sessie had uitsluitend schrijfrecht volgens opdracht op haar eigen rapport.
Deze gedragsafspraak is geen technisch afgedwongen sandboxgarantie.

Iedere kindworktree begon schoon op een eigen branch op parent `69809c2c`
en werd toegestaan detached op de exacte auditcommit gezet. **Commitinhoud
is gelijk; letterlijke branchgelijkheid is niet gerealiseerd.** Beginbranch,
bronselectie en eindstatus staan in ieder deelrapport. Geen bronresultaten uit
de beginbranches zijn stil samengevoegd. Security gebruikte bovendien een
ingebouwde algemene security-review-hulpreview; ook dat staat expliciet in
rapport 04 en is geen technisch geladen OS-profiel of menselijke audit.

Primaire bewijsbasis is uitsluitend de bestaande repository. Externe
standaarden, actuele juridische bronnen, advisories en CI-artifacts zijn niet
opnieuw opgehaald tijdens de oorspronkelijke audit. In deze PR-review is
alleen de publicatiemetadata van de officiële WCAG 2.2-pagina nagekeken om een
citatie te corrigeren; inhoudelijke EU-toepasselijkheid en andere externe
claims zijn niet gecontroleerd. Lokale OSDev-passages zijn aanvullende leerbronnen, geen
normatieve hardware- of beveiligingscertificering. Bij de oorspronkelijke
rapportintegratie op `6929b986440fe659fa1d45167f476313abb6c6d3` zijn de
rapporten hashgelijk overgenomen; noodzakelijke citaat-/scopepreciseringen
waren door hun auteurs gedaan. Deze gelijkheidsclaim is historisch en geldt
niet voor de na PR-review gewijzigde rapporten op de huidige head (zie §9).
Mechanische citatencontrole bevestigt vindbaarheid en regelbereik,
niet automatisch de inhoudelijke juistheid.

Alle technische bevindingen over Nieuw OS zijn repository-observaties tegen de
hierboven vastgepinde snapshot. Uitspraken over externe standaarden, hardware,
wetgeving of andere besturingssystemen zijn niet onafhankelijk geverifieerd
en gelden als `ONBEKEND` of als toetsvraag, niet als vastgestelde feiten.

### Remote drift tijdens de audit

De integrator controleerde met `gh pr view 150 --json
number,isDraft,state,baseRefName,headRefName,headRefOid,url` en
`git ls-remote --heads origin copilot/add-os-specialist-agents` de live
integratiecontext. Beide commando's slaagden. Op dat controlemoment wees de
remote branch naar `55da47347d4180101faac400c8aee4597182877a`, niet naar
de auditbron. PR #150 stond open naar `main`, maar niet meer als concept.
Eerder was hij in deze sessie wel als concept bevestigd.

Read-only `git log`/`git diff --stat` toonde latere profielwijzigingen,
een mainintegratie, artifact-storetests en status-/voortgangsmetadata.
**Die latere inhoud is niet geaudit, niet overgenomen en niet teruggedraaid.**
De gezamenlijke auditcommit blijft direct op de gepinde lokale bron.
Geen force-push, merge, PR-sluiting of overschrijving van ander werk; geen
claim dat deze audit de inmiddels gewijzigde PR-head beoordeelt. Push naar
die divergerende branch is daarom niet uitgevoerd.

## 2. Status en samenvatting van alle specialisten

Alle onderstaande rollen zijn als profieltekst gelezen in een gewone chat.
Alle zestien rapporten zijn gereed en geïntegreerd; geen specialist maakte
een commit of afzonderlijke PR.

| # | Profiel en deelrapport | Samenvatting en bewijsgrens |
|---|---|---|
| 01 | `os-architect` - [architectuur](subagents/01-architecture.md) | Eigen kernel/capability-IPC/diensten aanwezig. Accountautoriteit, koude sleutelactualiteit, TLS en geïntegreerd signed desktopprofiel open; single-core/DMA-aannames expliciet. |
| 02 | `os-builder` - [bouwstatus](subagents/02-build-status.md) | 84 niet-vendormanifests parseerbaar, twintig rootmembers en 53 lockfiles. Historische echte ELF/UEFI-builds en native deelgates; audit-HEAD niet gebouwd. Ontbrekende tools op zijn PATH zijn geen universele hostconclusie. |
| 03 | `os-verifier` - [verificatie](subagents/03-verification.md) | 102 manifests/1814 entries: 1788 matches, 25 ontbrekend, één mismatch. Vijftien hostharnastests geslaagd. Actuele native/releaseacceptatie niet vast te stellen; archiefcontrole gefaald. |
| 04 | `os-security-reviewer` - [security](subagents/04-security.md) | Legacy recovery-export via gewoon domeinendpoint retourneert geheim plus masterblob: HOOG, 9/10, E3. Geen exploit uitgevoerd; algemene hulpreview gebruikt. |
| 05 | `os-maturity-auditor` - [maturiteit](subagents/05-maturity.md) | Echte native deelresultaten, maar 62/62 volledige releasecriteria open. Taakgerichte vergelijking met acht referentie-OS'en, geen actuele externe featureclaims. TEST-keyrotatie na gerichte correctie erkend. |
| 06 | `os-privacy-reviewer` - [privacy](subagents/06-privacy.md) | Veertien dataflows; geïntegreerde private client/NAS/provider-/sleutelketen ontbreekt. Lokale encryptie, retentie, dashboard en AI-grenzen niet productbreed bewezen. |
| 07 | `os-eu-compliance-reviewer` - [EU-gaps](subagents/07-eu-compliance.md) | Dertien technische gaps met conditionele toepasselijkheid. Product-/marktrol, support/securityrespons, SBOM/licentie en toegankelijkheidsacceptatie open; juridische actualiteit niet geverifieerd. |
| 08 | `os-release-engineer` - [releaseketen](subagents/08-release-supply-chain.md) | Historische 18-stage TEST-policygates en 56 hashes erkend. Productieankers, firmware-loadertrust, hermetische dubbelbuild, complete SBOM/licentie en support ontbreken voor distributie. |
| 09 | `os-hardware-driver-architect` - [hardware](subagents/09-hardware-drivers.md) | Vaste QEMU/q35/BDF/single-core-profielen; pinnen/quarantaine geen IOMMU. ACPI-hostparser geen native poweroff; geen benoemd fysiek supportprofiel/E7. Netwerkmatrix gericht gepreciseerd. |
| 10 | `os-ux-accessibility-reviewer` - [UX/toegankelijkheid](subagents/10-ux-accessibility.md) | Twaalf taakpaden. Native U01/compositor/klembord behouden; persoonlijke installatie/login/lock/herstel en hulpmiddelketen niet compleet. F6/Alt+Tab, statussemantiek en invoergrenzen concreet E3. |
| 11 | `os-reliability-recovery` - [betrouwbaarheid/herstel](subagents/11-reliability-recovery.md) | Vroege bootvloer en terminale andere dienstfaults; onafhankelijke native backup/restore, key-write-ahead en duurzame syncqueue open. 35 synthetische harnastests en 85 historische hashes geslaagd, geen VM-herhaling. |
| 12 | `os-app-platform-reviewer` - [appplatform/SDK](subagents/12-app-platform-sdk.md) | Verifier/SDK/capabilities aanwezig; duurzame issuertrust/store/catalogus en dynamische verified-image-naar-procesketen ontbreken. ABI/features/bootstrap en documentatie driften. |
| 13 | `os-red-team` - [dreigingsmodel](subagents/13-red-team.md) | Acht defensieve AND/OR-aanvalsbomen met veilige toetsvoorstellen. Geen uitgevoerde exploit of nieuwe bevestigde exploiteerbare kwetsbaarheid; hypothesen blijven hypothesen. |
| 14 | `os-performance-energy` - [prestaties/energie](subagents/14-performance-energy.md) | Historische CPU-mediaandaling onafhankelijk exact herberekend: 88,2600281961856578%. Volledige latencyworkload, RAM/I/O/duur en fysieke energieacceptatie ontbreken. |
| 15 | `os-network-distributed-reviewer` - [netwerk/distributie](subagents/15-network-distributed.md) | Historische native ARP/UDP/DNS/TCP en NAS cold/replay erkend. Native TLS/tijd/peercontinuïteit, duurzame queue, dynamische policy, resolver en providerketen open. |
| 16 | `os-claims-auditor` - [claims](subagents/16-claims.md) | Bekende archiefgaten bevestigd; RTC-mismatch teruggevoerd op ADR-hernummering. Historische teksten onderrapporteren native voortgang. 21 accepted-criteria/41 punten versus 62 open releasecriteria correct gescheiden. |

## 3. Gecombineerde bewijs- en maturiteitsmatrix

Gedeelde orchestratorconventie, niet letterlijk in ieder profiel aanwezig:
E0 ontbrekend/ambitie; E1 ontwerp; E2 hostprototype/simulatie/hosttest;
E3 native broncode; E4 native positief; E5 native negatief; E6 herhaalde
CI-/releasegate; E7 gedocumenteerde fysieke hardwaretest; E8 onafhankelijke
menselijke audit/formele evaluatie. Deze legenda vervangt de bestaande
release-eisencodes niet. Een niveau geldt alleen voor genoemde eigenschap,
bron en profiel. E4 en E5 zijn verschillende dekkingen, geen automatische
goedkeuring van elkaar.

| Domein | Actuele bron / hostniveau | Historische uitvoering, niet op audit-HEAD herhaald | Ontbrekende productacceptatie |
|---|---|---|---|
| Kernel/processen/IPC | E3 eigen loader/kernel, paging, usercopy, capabilities en quota | E4/E5 en gerapporteerde E6 voor beperkte KS/K05/K07-profielen | Gecombineerde actuele gates, concurrency/SMP/shared-mappinginvarianten, E7/E8 |
| Opslag/sleutels | E3 volume/records/key-state; concrete legacy-exportbronbevinding | Native commit/corruptie/readonly/cold/snapshotdeelgates | Echte gebruikerssleutels, duurzame keymutatie vóór ACK, vertrouwde koude actualiteit, algemene encryptie en recovery |
| Desktop/sessies | E3 vensters/apps/semantiek; openbare VM-gast, geen persoonlijke broker | U01/compositorherstel en klembord E4/E5/E6 | Installatie, echte login/lock/wissel, instellingen, hulpmiddelketen, talen/IME/shaping en menselijke taken |
| Netwerk/NAS | E3 vaste app-/bestemmingsbinding en NAScore/probe; E2 host-TLS/crypto/clientmodel | ARP/UDP/DNS/TCP/client-exit en opaque NAS cold/tokenreplay E4/E5; historische gates apart | Native TLS/tijd/truststore, remote peercontinuïteit, E2EE-actualiteit, duurzame syncqueue, resolver/policy/provider |
| Updates/release | E3 signed profiel en TEST-keypolicyrotatie/intrekking | Historische 18-stage positieve/negatieve gates; geselecteerde updatebron gelijk aan testbron | Grafisch signed productimage, firmwareanker, productkeys/ceremonie, proefboot/recovery, SBOM/licentie/herkomst/support |
| Betrouwbaarheid | E3 COW/Unknown/readonly/compositor; E2 harnassen/kopiemodel | S06/S08/recovery/K07/U01/B03 deelgates | Algemene dienstrestart, onafhankelijke native backup/restore, RPO/RTO, configuratie/shutdown en duuracceptatie |
| Apps/SDK | E3 verifier/SDK/capabilities; E2 in-memory lifecyclemodellen | Vaste apps en beperkte processtart/klembordgates | Duurzame trust/store/catalogus, dynamische imageautorisatie, install/update/remove/grantlifecycle en oude-appcompatibiliteit |
| Hardware | E3 virtuele drivers; E2 USB/audio/ACPI-modellen/parsers | QEMU-framebuffer/RTC/blok/netwerkdeelproeven | Benoemde fysieke profielen, IOMMU/PCI/APIC/SMP, xHCI/HID, GPU/audio, native poweroff/suspend/batterij/thermiek |
| Performance/energie | E3 PIT/HLT/polling, E2 latencyrecorder | Zes historische WHPX-runs, drie per arm; diagnostische digestbenchmark/memorycorrectheid | Volledige 20-samples/richtingworkload, p95/framepacing, RAM/I/O/duur en fysieke watt/joule/thermiek |
| AI/quantum | Modelreader en software-qsim; afzonderlijke host/native artefacten | Native qsim/job/quotadeelbewijs, geen fysieke QPU | Werkelijke native AI-inferentie met taak-/retentie-/netwerkgrenzen; fysieke QPU afzonderlijk onderzoek |
| Compliance/onderhoud | E1 productcontracten en technische gapinventaris | Geen juridische of menselijke aftekening uit CI afgeleid | Productrol/toepasselijkheid, actuele primaire bronnen, support/incidentrespons, licentie/SBOM, menselijke en onafhankelijke evaluatie |

De bestaande 41/100 is een planningsmeter, geen veiligheidspercentage,
tijdsprognose of releasepariteit. Geen punten toegevoegd of ingetrokken:
de opdracht verbiedt score-/roadmap-/statuswijzigingen en deze audit levert
geen nieuwe native criteriumacceptatie. De 62 volledige releasecriteria
blijven open; historische deelacceptatie wordt niet gewist.

## 4. Conflicten en hun expliciete behandeling

| Conflict | Behandeling |
|---|---|
| TLS E2 versus E3 (01/02/15) | Hosthandshake E2, native adapterbron E3; geen native producthandshake E4. Geen heel subsysteem één niveau geven. |
| Oude NAS/privacykoppen versus latere native probe | Historische kop onderrapporteert; lokale opaque/cold/replaydeelbewijzen erkennen, geen complete E2EE/sync afleiden. |
| Apart key-admin-ingress versus legacy domeinexport (01/04) | Ontwerpintentie beschermt de concrete legacyroute niet. Securitybronbevinding heeft voorrang voor die onderzochte operatie; geen stil architectuurakkoord. |
| Positieve receipts versus 25 ontbrekende logs/RTC-mismatch | Intacte bindings én specifieke gaten vermelden. Niet alle historische proeven als gefaald of integraal betrouwbaar verklaren. |
| Zelfde branch gevraagd versus detached kinderen | Exacte commitinhoud gelijk; branchvoorwaarde letterlijk niet gehaald. Methodeafwijking blijft zichtbaar. |
| Gevraagde OS-customagent versus gewone chat/algemene hulpreview | Gebruiker stond chatroute toe; profiel gelezen, niet technisch geladen. Securityhulpreview expliciet; geen menselijke onafhankelijkheid. |
| EU-cloudverbod/revisie 3 versus leidende revisie 4 | Leidende keuze staat expliciete opaque provider toe. Documentdrift blijft bevinding; contract niet tijdens audit gewijzigd. |
| Maturity noemt rotatie ontbrekend versus latere TEST-next-gates | Door auteur 05 gericht gecorrigeerd. TEST-rotatie/intrekking aanwezig; productiepolicy/-custody en firmwaretrust blijven open. |
| Shell-/loginontwerp zegt geen code versus huidige E3/U01 | Historische ontwerpbasis onderscheiden van latere bron en deelacceptatie. Geen volledige account-/a11yacceptatie afleiden. |
| Hardware-netwerkmatrix E3 versus native integratie E5 | Auteur 09 preciseerde pakketproef versus latere native stackintegratie; E5 erkend buiten eigen hashvalidatie. Fysieke NIC/IOMMU nog open. |
| CPU 88,26002820 versus 88,2600281961856578 | Alleen afronding: verschil -0,0000000038143421678765 procentpunt. Geen nieuwe meting, geen fysieke energieclaim. |
| `NET_CACHE_INTACT` mogelijk als DNS-cache gelezen | Bron controleert ARP-burencache. Native A-query/TTLcheck wel aanwezig; geen volledige resolver/cache-expiryacceptatie. |
| Lokale auditbron versus gewijzigde livebranch/PR | Nieuwere inhoud niet geaudit; geen samenvoeging, terugdraaiing of force-push. Zie §1. |

REL-05 en RR-01 beschrijven hetzelfde bootvloerprobleem en tellen als één
risico. Privacy-, maturity-, release- en netwerkrapporten benoemen deels
dezelfde integratiegaten; frequentie van noemen is geen extra bewijs.
Red-teamhypothesen tellen niet als nieuwe bevestigde kwetsbaarheden.

## 5. Blokkerende risico's

### Concrete securitybronbevinding

| # | Severity | File | Lines | Vulnerability | Confidence |
|---|----------|------|-------|---------------|------------|
| 1 | 🟠 HIGH | `sdk/examples/keyring/src/service.rs` | 520-536 | Gewoon domeinendpoint staat legacy `EXPORT_RECOVERY` toe en retourneert geheim plus masterblob samen | 9/10 |

Dit is AUDIT-04-01, **E3-bronbevinding**, geen uitgevoerde exploit/E5.
De B01-acceptatiemanifest `kernel/src/m2/m3/system.rs` verleent de niet-admin
app SEND op het domeinendpoint; het admin-ingress blijft apart bij init.
HIGH/9 op 10 betreft dit profiel. Productie- of gewone-app-provisioning van
deze route is niet aangetroffen; productiebereikbaarheid en exploitability
zijn **ONBEKEND**. Zie rapport 04 voor de historische brontrace.
Geen automatische toegang tot alle ciphertext of kernelrechten bewezen.
Vervolg vraagt expliciete domeinallowlist, afwijzing van legacyherstel via
app-ingress en native adversariële deny/regressie; niets tijdens audit gefixt.

### Product-, herstel- en bewijsblockers

| Risico | Grens en gevolg |
|---|---|
| Identiteit en sleutel-/opslagcontinuïteit | UI-gastweigering is geen accountautoriteit; key-write-ahead/koude vloer en volledige versleutelde gebruikersroute ontbreken. |
| Bootvloer vóór gezonde kernel | REL-05/RR-01: een geldig getekende maar crashende nieuwe kernel kan de werkende oude slot onder de verhoogde vloer plaatsen. Zelfstandig brontrace, niet gereproduceerd. Antirollback niet uitschakelen. |
| Herstel niet generiek | Compositorherstel bestaat; andere kritieke window-session-dienstfaults zijn terminaal. Lokale historie is geen onafhankelijke native backup. Nieuwste-tabelcorruptie kan volgens bekend contract de laatste commit verliezen. |
| Verbonden productketen | Native TCP/UDP/DNS en lokale NASprobe zijn geen TLS/peer/E2EE/sync/providerproduct. RAMcheckpoint bewijst geen koude offline-editduurzaamheid. |
| Releaseautoriteit en onderhoud | Openbare TEST-ankers zijn bewust testmateriaal, geen gelekte productiekeys. Productanker-/firmwareketen, SBOM/licentie, herkomst, support en compromisherstel ontbreken voor distributie. |
| Dagelijkse bruikbaarheid/apps/toegankelijkheid | Installer, persoonlijke sessies, native hulpmiddelen en derdenapplifecycle ontbreken als complete reis; input/status/ABI/DX hebben concrete bron-/documentgaps. |
| Bewijsarchief | 25 oorspronkelijke logs ontbreken; RTC-README wijkt af van manifest. Dit blokkeert volledige lokale hercontrole van die archieven, niet automatisch al het bronwerk. |
| Finale kwalificatie | Actuele gecombineerde imagegates, duur/meting, fysieke ondersteuning en menselijke/onafhankelijke evaluatie ontbreken. Geen E7/E8 uit VM of AI afleiden. |

## 6. P0-, P1-, P2- en P3-prioriteiten

Prioriteiten zijn voorstellen voor vervolg, geen nieuwe roadmap of opdracht
om nu productcode te wijzigen. P0 is onmiddellijke autorisatie-/bewijshygiëne;
P1 de ontbrekende productketens; P2 verbreding en meetdekking; P3 finale
kwalificatie. Specialistprioriteiten die een productclaim blokkeren zijn
hier niet automatisch securityernst.

| Niveau | Werk |
|---|---|
| P0 | Legacy key-exportgrens dichten en native negatieve regressie; archiefgaten door bewijseigenaar herstellen of expliciet ongeldig/open afbakenen. Geen historische logs reconstrueren of nieuwe runs als oude bytes voorstellen. |
| P1 | Echte account/lock/key-unlock/versleutelde data/herstel; duurzame keymutaties en koude actualiteit; bootgezondheid/recovery boven vloer; dienstspecifiek herstel en onafhankelijke backup/restore. |
| P1 | Native TLS/tijd/policy/peercontinuïteit/E2EE met duurzame syncqueue en expliciete provideradapter; echte apptrust/store/catalogus/launch/lifecycle; toegankelijke installatie/sessie en bruikbare status-/keyboardpaden. |
| P1 | Eén productreleaseprofiel, firmware-/producttrust, SBOM/licentiebeslissing, gecontroleerde herkomst/dubbelbuild en operationeel security-/supporteigenaarschap. |
| P2 | ABI/bootstrap/features/compatibiliteit en documentdrift; resolver/DHCP en driveruitbreiding; volledige latency/RAM/I/O/idlemetingen; vooraf benoemd HW-profiel/IOMMU/PCI-/powercontracten. |
| P3 | Na geïntegreerde gates: herhaalde duur-/herstelkwalificatie, expliciet gemachtigde fysieke proeven, menselijke taken/hulpmiddelen, onafhankelijke audit, actuele juridische toepasselijkheid en formele releaseaftekening. |

## 7. Afhankelijkheden en aanbevolen uitvoeringsvolgorde

| Voorwaarde | Afhankelijk resultaat | Waarom |
|---|---|---|
| Domeinoperaties correct autoriseren | Persoonlijke sleutel-/identiteitsketen | Anders blijft gewoon app-ingress herstelautoriteit bereiken. |
| Identity/sessiebroker + duurzame key-state/actualiteit | Login/lock/opslag/herstel | UI-status en generatievloer nul leveren geen productieautoriteit. |
| Native RNG/tijd/TLS + caller-/devicefence | Peeridentiteit en NAS/provider | IP/PID/hostnaam of requestboolean bewijst geen remote identiteit. |
| Duurzame lokale documenten/queue + E2EE-versieankers | Offline sync/reconcile en disaster recovery | RAMqueue en serverchecksum geven geen koude clientduurzaamheid/actualiteit. |
| Signed desktopprofiel + producttrust + proefboot/migratie | Herstelbare productupdate | Losse componentgates vormen niet één veilig bijgewerkt image. |
| Duurzame issuertrust/store/catalogus + ABI/bootstrapcontract | Geautoriseerde appstart/update/remove | Verificatie geeft geen startrecht; manifest geeft geen capabilities. |
| Dienstspecifieke epochs/reset/ownership + onafhankelijke backup | Algemene herstelacceptatie | Compositorroute is niet toepasbaar op iedere dienstfault. |
| Native semantiekbroker/events/focus + hulpmiddel | Toegankelijke taken en menselijke acceptatie | Lokale tree of CSS-contrast is geen werkende schermlezer. |
| Bron/image/workload bevroren + aangesloten recorder | p95/RAM/I/O/duuracceptatie | Laatste toetsinterval en microbenchmark zijn geen productlatency. |
| Bewijsprovenance + passende toolchains | Actuele integrale native gates | Geen oude receipt of cache-hit als nieuwe uitvoering rekenen. |
| PCI/IOMMU/DMA/SMP-contract + apart hardwaremandaat | Fysieke support/E7 | QEMU/single-core/pinnen bewijst geen fysieke containment. |

Aanbevolen uitvoering: eerst P0-autorisatie en bewijsafbakening; daarna de
kleine gedeelde identiteit/key-/boot-/lifecyclecontracten vastleggen. Vervolgens
kunnen sessie/opslag/herstel, netwerk/NAS en appplatform parallel bouwen met
één eigenaar per gedeelde interface. Integreer op één exact image met
positieve én negatieve gebruikersroutes. Sluit daarna de volledige
performance-/duur-/recoverymatrix, en pas vervolgens de afzonderlijk
geautoriseerde fysieke, menselijke, onafhankelijke en releasegates.
Documentatieprecisering kan onafhankelijk, maar vervangt geen implementatie.

## 8. Ontbrekend bewijs en uitgevoerde verificatie

Ontbrekend zijn onder meer actuele bron-/binary-/imagegebonden geïntegreerde
gates, oorspronkelijke ontbrekende logs, volledige pixel-/diskhercontrole uit
historische artifacts, hermetische dubbelbouw, native product-TLS/sync/apps,
echte auth/keyrecovery, OS-brede hulpmiddelen, representatieve latency/duur,
fysieke hardware/energie, menselijke taken en onafhankelijke evaluatie.
Juridische actualiteit en product-/markttoepasselijkheid zijn niet vastgesteld.
Een ontbrekend bewijs is geen impliciete goedkeuring en ook niet automatisch
een uitgevoerde mislukte runtimeproef.

### Eigen integratorcontroles

| Exact commando / controle | Resultaat en grens |
|---|---|
| `git branch --show-current && git rev-parse HEAD && git status --short` | Exit 0; gepinde lokale branch/bron, alleen nieuwe auditmap; herhaald tijdens integratie. |
| `python3 -B -m unittest -v tools.tests.test_materiaal_host_count` | Exit 0; vijf hostharnastests, 0,001 s; E2. |
| `python3 -B tools/test_window_session.py -v` | Exit 0; tien hostharnastests, 0,045 s; E2. |
| Exact C06-manifestscript uit rapport 03, lokaal herhaald | Exit 1; 102/1814/1788/25/1. Volledige lokale uitvoer in sessieartefact `audit-manifest-check.log`; geen runtimefalen afgeleid. |
| `python3 -B tools/progress.py --check` | Exit 0; 41/100 structureel consistent; beoordeelt geen native bewijsinhoud. Geen `--write`. |
| `source_hash()` uit bestaande `tools/common.py` | Exit 0; source-v3 uit §1; geen supply-chaincertificering. |
| `unittest`-discovery van `test_volume_disk.py`, `test_update_harness.py`, `test_supervisor_harness.py` onder `tests` | Exit 0; 35 synthetische hostharnastests, 0,582 s runner/0,619 s totaal; geen native boot/hersteluitvoering. |
| `tomllib`-inventaris van gevolgde `Cargo.lock` | Exit 0; 53 locks, 338 externe entries niet ontdubbeld, alle met checksum. Geen advisory-/licentiescan. |
| `(cd docs/verification/b03-key-policy-mainca45 && sha256sum --check --quiet SHA256SUMS)` plus JSON-asserties | Exit 0; 56 hashes en opgeslagen 18/18 completed/passed-stages; geen nieuwe updateproef. |
| PyYAML `safe_load` op de zestien `.github/agents/os-*.agent.md`-frontmatters | Exit 0; zestien mappings met naam/beschrijving; geen profielbestanden gewijzigd. |
| Integratievalidator uit een lokaal sessieartefact (pad geredigeerd; geen repositorybestand) | Historisch exit 0: zestien auteurhashes, 826 volledige broncitaten via `git show`, rapportlinks, whitespace, codefences en Pythonbloksyntax; exact zeventien uitsluitend toegevoegde rapporten. Geen OS-test. |
| `git diff --cached --check` | Exit 0; staged auditdiff zonder whitespacefouten. |

De hosttestlogs `audit-host-count.log` en `audit-window-harness.log`, het
manifestlog en de integratievalidator staan buiten Git in een lokaal
sessieartefact; het privépad is niet gepubliceerd. Zij zijn sessieartefacten,
geen beloofd permanent CI-archief. De beslissende
manifest- en CPU-herhaalcommando's staan volledig in de deelrapporten;
overige inline controles en exitstatussen staan in het tooltranscript.

### Expliciet niet uitgevoerd

Geen nieuwe Cargo-/Clippy-/crossbuild, QEMU/OVMF/native gate, exploit,
fuzzing, koude productrecovery, echte installer/login, native TLS/NAS/provider,
nieuwe benchmark, honderd starts, 24-uursproef, fysieke disk-/firmwarehandeling,
hardware-/energieproef, menselijke gebruikerstest, onafhankelijke audit,
formele evaluatie, juridische aftekening, releaseondertekening of releasevalidatie.
Geen afhankelijkheden geïnstalleerd of externe reviewdienst gebruikt.
Geen actuele CI-run of artifactdownload; alleen beperkte PR-/remotecontext
door de integrator opgehaald zoals in §1.

## 9. Rapportherkomst en integratiegrens

Historische auteurhashes van de zestien oorspronkelijk overgenomen bestanden
in rapportcommit `6929b986440fe659fa1d45167f476313abb6c6d3`.
De tabel legt die oorspronkelijke bytes vast, **niet de huidige head**:
latere PR-reviewcorrecties wijzigen rapportbytes. Hashgelijkheid en de
historische integratievalidator uit §8 gelden uitsluitend voor die
oorspronkelijke integratie; zij valideren de latere wijzigingen niet.

| # | Historische SHA-256 op rapportcommit 6929b986 |
|---|---|
| 01 | `2d25a28fff2873e001799043d70b14f2d603d9c86d01d022927d0b6de3e473d5` |
| 02 | `d12e820e66c06073cf4050af81a757aa1e6c59c6e55f2f63f70235a55bf5562c` |
| 03 | `7bb18edcd064bc52c1fb730a451c9619cf839915f8eaf760d77b598b70af59fb` |
| 04 | `73f31ba51be30d50418dcfa01a15bfbc7c4e10d2e6b8159fa8ae51acc375b0e9` |
| 05 | `d4a5de360160e64dd08d016834c5b3dea318bc652d2651abc1ad1e016f9f36dd` |
| 06 | `0ac680cbec333555edef2019e350c1a19387d919d5b577c264589e30da648f25` |
| 07 | `2367e2ef7a4780246ee2e4e90562166e83329a66bc5647d4d5688d551bd693a3` |
| 08 | `6c79388211aa0fa397f3adfae3b8ec39a7fc43f4656dbf69ecef6f31b4a316b7` |
| 09 | `02bbbcc4f80c390673ab8fab3a4930fc742c650ad5656b7ac346bbdde6af24f3` |
| 10 | `0258040289f2a15c378e28758e570380999851e07937b487b3d18e3a921b2b67` |
| 11 | `ca7f89615ceee24c92efff21434761c80b63589788210fc5b741715e5b7a0e0b` |
| 12 | `c18e0bb05f39bc489536729f4de7161c111d1031e3b65e80859921f1a65f20a0` |
| 13 | `6653086a1d20e041a0fdcde23e202691c8dd9e48ed522ff60371da066a13fa74` |
| 14 | `b1a515bc72b016f21e3e13bd09d4de7ec04067977abdccce5058939d2b70f91c` |
| 15 | `b3119595fff31862afb5f5556a2eb25f61735e6016834fb752a3b0c7ffa67abb` |
| 16 | `3fa0fa9f0bc1774a459675513aec1b4fe23e2de406c94584f2e0c970b1a365fb` |

De oorspronkelijke gezamenlijke rapportcommit `6929b986440fe659fa1d45167f476313abb6c6d3`
is na alle rapporten en integratiechecks gemaakt, direct boven de auditbron
beafdedf. `git log -- docs/audits/OS-MULTI-AGENT-AUDIT.md` toont daarnaast
latere rapportcorrecties, geen nieuwe audit van de actuele productbron.
Bestaande `AGENTS.md`, `.codex/agents`, profielbestanden, productcode, roadmap,
voortgang en projectstatus bleven in de oorspronkelijke auditdiff ongewijzigd.
Geen reparatie, releasegoedkeuring of publicatietoestemming uit de
auditbevindingen afleiden.
