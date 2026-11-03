# AUDIT-14 - prestaties en energie

**Bronmetadata:** primaire repositorybron: *Nieuw OS — “Add OS specialist
agent profiles”*, auteur/organisatie S.W.J.B.M. van Ieperen, 3 november 2026,
commit `beafdedfc188154fe7285848fbd411e651013f54`
([onveranderlijke snapshot](https://github.com/simonvanieperen/nieuw-os/tree/beafdedfc188154fe7285848fbd411e651013f54)).
Alle lokale `pad:regel`-citaten hieronder zijn repository-observaties uit die
snapshot. Externe energie-/hardwareclaims en vergelijkingen zijn niet
onafhankelijk geverifieerd en blijven `ONBEKEND`.

Datum: 3 november 2026, Europe/Amsterdam. Audit-only, repository-only.
**Uitkomst:** bruikbare brongebonden microbenchmark- en idle-CPU-bewijzen,
maar geen aangetoonde volledige performance- of energieacceptatie van de
onderzochte bron. Geen optimalisatie of productcodewijziging uitgevoerd.

## Opdracht, bron en werkomgeving

Opdracht: "Beoordeel bestaande benchmarks, meetmethodes, latency, geheugen,
I/O, idlegedrag en energieverbruik. Voer geen ongecontroleerde optimalisaties uit."

- Repository: `simonvanieperen/nieuw-os`.
- Verplichte bronbranch: `copilot/add-os-specialist-agents`; de lokale
  `refs/remotes/origin/copilot/add-os-specialist-agents` wees exact naar
  `beafdedfc188154fe7285848fbd411e651013f54`. Geen fetch/API-opvraag uitgevoerd.
- Beginbranch: `simonvanieperen-sturdy-carnival`, HEAD
  `69809c2cb53dfb3bd77abe5352892810fba6e82d`, **schone** beginboom.
  Deze beginbranch was dus niet de gevraagde bronbranch.
- Eigen worktree; het lokale pad is niet gepubliceerd. De worktreeregistratie
  bevestigde de eigen werkplek. De hoofdcheckout is uitsluitend als
  Git-worktreemetadata gezien, niet als bestandsbron gelezen of beschreven.
- Na de toegestane `git switch --detach` is de audit geheel uitgevoerd op
  **detached HEAD `beafdedfc188154fe7285848fbd411e651013f54`**.
  Geen branchhernoeming, commit, push, merge of PR-actie.
- Dit is een gewone chat die
  `.github/agents/os-performance-energy.agent.md:9-25` expliciet heeft
  gelezen, geen technisch geladen customagent. Geen deelagents gestart.
- Exclusief schrijfbestand: `docs/audits/subagents/14-performance-energy.md`.
  Geen score-, roadmap-, status- of andere rapportbestanden gewijzigd.
  De opgeslagen 41/100 wordt niet opnieuw toegekend of aangepast.

Vooraf gekozen grens: statische broninspectie en eindige herberekening van
opgeslagen synthetische proefgegevens, zonder builds, nieuwe VM's, benchmarks
of tests. Acceptatie van deze audit is een herleidbaar rapport, correcte
historische rekencontrole, geldige citaten en uitsluitend het toegestane
schrijfbestand. Dit is geen acceptatie van het OS.

Gelezen instructies: root `AGENTS.md`, `docs/AGENTS.md`, `kernel/AGENTS.md`,
`drivers/AGENTS.md`, `services/AGENTS.md`, `tests/AGENTS.md`, `ui/AGENTS.md`,
het eigen profiel en `.agents/skills/os-dev/SKILL.md`. Ook gelezen: README,
het actuele checkpoint, privacycontract, relevante architectuur- en
beveiligingscontext, ADR-045/047/061, teststrategie, relevante workflowstappen
en uitvoeringshandleiding. De lokale OSDev-skillroute
`.agents/skills/os-dev/references/workflows/06-debugging-and-review.md:1-47`
is als werkwijze gebruikt, niet als hardwarebewijs. Geen externe bron
opgehaald; bestaande externe verwijzingen zijn niet opnieuw geverifieerd.
De beschikbare CodeRabbit-skill is gelezen, maar de externe dienst/CLI is
niet gebruikt vanwege de repository-only-grens.

## Bewijslegenda en afbakening

| Niveau | Betekenis in deze audit |
|---|---|
| E0 | Ontbrekend bewijs of ambitie |
| E1 | Ontwerp/contract |
| E2 | Hostprototype, simulatie of hosttest |
| E3 | Native broncode, zonder nieuwe uitvoering hier |
| E4 | Historisch native positief uitvoeringsbewijs, uitsluitend voor genoemde bron |
| E5 | Historisch native negatief uitvoeringsbewijs, uitsluitend voor genoemde bron |
| E6 | Brongebonden herhaalde CI/releasegate, niet automatisch de huidige bron |
| E7 | Fysieke hardware-uitvoering van Nieuw OS op exact ondersteund profiel |
| E8 | Onafhankelijke menselijke audit of formele evaluatie met eigen scope |

De niveaus zijn geen overdraagbaar keurmerk. Een opgeslagen `passed=true`
of CI-snapshot is een historisch artefact; deze audit herhaalt geen runtimegate.
Een hashcontrole bewijst bytebinding, niet waarheid van de oorspronkelijke
observatie. De Windowsproef draait Nieuw OS in QEMU/WHPX op een fysieke host:
dat is VM-bewijs E4/E5, geen bare-metal E7 en geen fysieke energiemeting.
Deze AI-audit is geen E8.

## Kernbevindingen en prioriteit

P1 betekent: blokkeert de bijbehorende product-/prestatieclaim.
P2 betekent: meetdekking verbeteren voordat een volgende optimalisatie wordt
geaccepteerd. Geen van deze prioriteiten is een opdracht om nu code te wijzigen.

| ID | Prioriteit | Bevinding, effect en exacte bronverwijzing |
|---|---|---|
| PE-01 | P1 | De volledige latencyworkload is niet aangesloten op de native acceptatieroute. `tools/window_latency.py:145-186` definieert plannen; `tools/window_latency.py:330-357` beoordeelt aantallen en p95. De werkelijke runner houdt alleen een interval vanaf de laatste toetsactie bij (`tools/run_window_session.py:268-306`). De CI controleert functionele uitkomsten, geen p95-plan (`.github/workflows/native-window-session.yml:118-139`). Het archief noemt de volledige workload expliciet niet uitgevoerd (`docs/verification/native-clipboard-20261102/README.md:93-100`). Dus E1 plus hostrecorder, geen aangetoonde p95-productacceptatie. |
| PE-02 | P1 | Energie, thermiek en slaap/wek hebben geen productmeting in het onderzochte bewijs. De ACPI-crate is alleen een parser zonder port-I/O of shutdownsyscall (`drivers/acpi-power/README.md:25-33`); firmwarecompatibiliteit is hostparsing, geen eigen-kernel-poweroff (`drivers/acpi-power/README.md:89-96`). FR-S05 en FR-R02 vereisen nog fysieke energie-/herstel- en duuracceptatie (`docs/FULL-TEST-RELEASE.md:167`, `docs/FULL-TEST-RELEASE.md:231`). Minder QEMU-CPU is geen gemeten watt/joule of accuduur. |
| PE-03 | P1 | Geheugenbewijzen dekken correctheid en herstel, niet het totale productgeheugen onder langdurige belasting. De validator controleert nulvulling, quota, fouten en cleanup (`tools/run_memory.py:61-100`); de historische vijf gasten hebben uitdrukkelijk geen product-/hardwareacceptatie (`docs/verification/full-release-memory-b6074c3/memory-gate.json:1-56`). Zonder werkset, piekgebruik, allocatietijden en groei per productworkload blijft het geheugenbudget voor FR-R02 open. |
| PE-04 | P2 | Opslag- en netwerkinterfaces zijn begrensd, maar geen representatieve I/O-prestatiebaseline gevonden in de onderzochte systeemharnassen. Virtio-blk schakelt interrupts uit en pollt (`drivers/virtio-blk/src/lib.rs:97-113`); een tweede pending aanvraag geeft Busy (`drivers/virtio-blk/src/lib.rs:288-307`). De NIC heeft een enkel pending TX-binding en vaste RX-buffer (`kernel/src/m2/m3/network.rs:10-33`). Functionele requesttellingen en volledige routeduur vervangen geen doorvoer-, wachtrij- of p95-flushmeting. |
| PE-05 | P2 | De idlewinst is reproduceerbaar uit historische tellers, maar slechts drie korte runs per arm op een headless WHPX-profiel. Het archief begrenst redraw-/counterclaims zelf (`docs/verification/gui-idle-20261102/windows-ab/README.md:15-23`). Geen lange idleduur, actief/idle-mix, gecontroleerde achtergrondlast, gastprocesattributie of temperatuur-/energiereeks aangetoond. De gemeten winst is echt binnen die historische scope, geen algemene platformclaim. |
| PE-06 | P2 | Kernel-idle blijft PIT-gedreven en GUI-invoer blijft polling. `kernel/src/m2/m3.rs:1153-1190` bevat resumable `sti; hlt; cli`; `kernel/src/m2/m3.rs:1901-1914` programmeert de PIT. Userspace wacht tot de volgende tick (`sdk/examples/window-session/src/idle.rs:24-47`); de compositor pollt op twee ticks (`sdk/examples/window-session/src/main.rs:49`, `sdk/examples/window-session/src/main.rs:275-291`). Dit is een implementatie met energie-/responstijdafweging, geen bewijs voor een harde 20ms-grens of diepe fysieke idletoestanden. |
| PE-07 | P2 | De digestmicrobenchmark is goed afgebakend, maar geen scheduler-, IPC-, contextswitch- of bootbenchmark. Hij meet voor timer/userspace met vaste engine en IF=0 (`kernel/src/m2/m3/cap_bench.rs:1-10`, `kernel/src/m2/m3/cap_bench.rs:52-69`). Varianten worden geroteerd en invarianten gecontroleerd (`kernel/src/m2/m3/cap_bench.rs:105-137`). Het resultaat heeft geen prestatiedrempel als PASS-voorwaarde (`tools/run_cap_bench.py:33-70`); historische PASS betekent correcte proef, niet snellere variant. |

### Wat al goed begrensd is

De idlewijziging laat ontvangstrechten, absolute transactiedeadlines en
herstelvoorwaarden intact. De compositor weigert rust bij pending replies,
inputbacklog, gedeeltelijke surfaces of damage:
`sdk/examples/window-session/src/main.rs:623-634`.
De idlehelper controleert klokterugloop, overflow en het wake-masker:
`sdk/examples/window-session/src/idle.rs:31-46`. Dit is E3-bron, geen
nieuwe negatieve uitvoering in deze audit.

De recorder bewaart mislukte en vervangen samples en staat geen
succes-door-herhalen toe: `tools/window_latency.py:272-357`. Hij verklaart
uitdrukkelijk dat aangeleverde hashes geen bewijs zijn dat de caller de
beeldcontrole uitvoerde: `tools/window_latency.py:1-13`. Deze scheiding moet
bij toekomstige aansluiting behouden blijven.

Het K07-resourceharnas controleert echte cancel/deadline/cleanup-markers en
voortgang van twee qsim-processen, maar zijn sleutel
`memory_cpu_budget_and_recovery` is geen gemeten volledig OS-CPU-/RAM-budget:
`tools/test_resource_pressure.py:82-120`. Geen negatief oordeel over K07;
wel een grens aan het hergebruik als performancebewijs.

## Onafhankelijke herberekening van historische idle-CPU

Invoer: `docs/verification/gui-idle-20261102/windows-ab/result.json:1-43`,
de oorspronkelijke `aggregate-original.json.gz` en de twaalf gecomprimeerde
native-/serialkopieën. De bindingsindex staat op
`docs/verification/gui-idle-20261102/windows-ab/index.json:1-19`.
Geen historisch absoluut Windows-pad uit die JSON is geopend.

Herberekend met `Decimal`, precisie 45, rechtstreeks uit ruwe
`kernel_100ns + user_100ns` en afzonderlijke monotone leesintervallen:

```text
cpu_s = (total_after - total_before) / 10000000
wall_s = (after_started + after_completed
          - before_started - before_completed) / 2
cpu_percent = 100 * cpu_s / wall_s
daling = 100 * (1 - mediaan(B) / mediaan(A))
```

Dit is een berekening over historische data, **geen nieuwe meting**.
De opgeslagen percentages of `root-cpu-recalculation.json` waren niet de
rekeninvoer. Die laatste diende alleen als vergelijking:
`docs/verification/gui-idle-20261102/root-cpu-recalculation.json:101-110`.

| Run | Arm | ProcesCPU, % van een logische CPU | vCPU-thread, % |
|---|---|---:|---:|
| 1 | A | 99,268908178307 | 99,060676292949 |
| 2 | B | 11,666287862300 | 11,041310842023 |
| 3 | B | 12,395408129516 | 11,874598124017 |
| 4 | A | 99,372366963517 | 98,955756195589 |
| 5 | A | 99,373577632859 | 98,852822927294 |
| 6 | B | 11,666233849391 | 11,353755660712 |

| Samenvatting | A | B | Relatieve medianedaling |
|---|---:|---:|---:|
| ProcesCPU-mediaan | 99,372366963517% | 11,666287862300% | **88,260028196186%** |
| ProcesCPU-minimum/maximum | 99,268908178307 / 99,373577632859% | 11,666233849391 / 12,395408129516% | Niet als intervalschatting bedoeld |
| vCPU-mediaan | 98,955756195589% | 11,353755660712% | 88,526432319641% |
| vCPU-minimum/maximum | 98,852822927294 / 99,060676292949% | 11,041310842023 / 11,874598124017% | Geen fysieke idle-residency |

De integrator/verifier gaf 88,26002820%. Onafhankelijk resultaat:
88,2600281961856578321234544035668928426679451%.
Verschil **-0,0000000038143421678765 procentpunt**: uitsluitend afronding.
Dit is een relatieve daling van de medianen; niet een daling van 88,26
CPU-procentpunten en niet 88,26% minder fysieke energie.

De gedecomprimeerde aggregate-SHA256 is gecontroleerd:
`26fc841be9ff0d1414db0eb23c245cb0a812504993cbd44ff432d213930f3637`.
Ook gecontroleerd: leesbare aggregatehash en JSON-gelijkheid met het origineel,
twaalf native-/serialhashes, vaste countercreatietijd, telleroptellingen en
overeenstemming met opgeslagen percentages binnen 0,000000001 procentpunt.
Alle zes native rapporten vermelden passed/native_gate, 13 fasen en raw exit 33.
Dit controleert hun inhoud; oorspronkelijke schermbestanden en diskvolumes
zijn hier niet opnieuw beschikbaar gesteld of geïnspecteerd.

Gecontroleerd identiek historisch profiel in alle zes native JSON's:
`pc-q35-8.2`, `whpx`, `qemu64`, 1 vCPU, 256M; QEMU-SHA256
`47d57a6072e0bb3bd98f87926eb129eb1736dfe818c67b3b81ef7ce4edd0b3cd`.
De methode vermeldt QEMU 11.1.0, 1280x720, 15 seconden per run,
beeldgereedheid voor de meting en volgorde A,B,B,A,A,B:
`docs/verification/gui-idle-20261102/windows-ab/README.md:13-23`.
Dat is geen vaste thermische warm-up; de kleine reeks rechtvaardigt geen
algemene p95- of betrouwbaarheidsintervalclaim.

Historische bron A: `337f2f29f1833ef4a8d2cd640b09419f85dbb83b`,
source-v3 `3919403542c923bc86467db3184345159a5ce6881567a0e11baa85f2d39f1995`.
Historische bron B: `1f2ac0ae7cf8944d77ca0656affb01fe9b086f1b`,
source-v3 `8faf3f7f47e3f26409e446b2311673416ddd70d5735ef106d94c9f30d2c6847a`.
Finale historische CI-head en CI-merge zijn weer andere bronnen:
`docs/verification/gui-idle-20261102/INTEGRATION.md:28-45`.
Geen van deze runs is omgelabeld naar `beafdedf`.

## Overige meetgebieden en bewijsdekking

| Gebied | Bestaand bewijs | Wat nog ontbreekt voor de betreffende claim |
|---|---|---|
| Boot, login en appstart | Historische WHPX/TCG-sessietijden met drie starts per backend; `docs/implementation/VM-HARDWARE-AND-PERFORMANCE.md:196-202`. Een harnastimeout is expliciet geen gemeten schermtijd (`docs/implementation/VM-HARDWARE-AND-PERFORMANCE.md:15-19`). | Actuele bron/image, onderscheiden koude boot/login/appstart-eindpunten, representatieve herhalingen, achtergrondlast en vooraf vastgelegde regressiedrempels. Loginflow is geen bewijs van echte authenticatie. |
| Input, focus, appwissel en plakken | E1-contract met p95-doelen 100 ms en volledig beeld/plakken 250 ms, per richting minstens 20 samples en drie koude starts (`docs/implementation/NATIVE-CLIPBOARD.md:77-96`). E2-recorder en historische native functionele beelden. | Werkelijke uitvoering van deze volledige workload. Niet de laatste toets van een burst als eerste invoertijd gebruiken. Volledig appbeeld is geen whole-screen/scanout; observerpolling van 20/50/200 ms is onderdeel van de bestaande hostintervallen (`tools/run_window_session.py:259-296`). |
| Frame pacing | Gevalideerde volledige appbeelden; protocol-/batchtellingen onderscheiden van snelheid (`docs/verification/perf-batch-review.md:35-47`). | Reeks presentatietijden, frameafstand/spreiding, gemiste deadlines en bewijs van echte scanout waar dat wordt geclaimd. Geen FPS afleiden uit FRAMES/COMMITS-snapshots. |
| Scheduler, contextswitch en IPC | E3-round-robin/idlebron; IPC-waitgate controleert queue/deadline/revocation en echte idle-IRQ-counters (`tools/run_ipc_wait.py:24-51`). Historische idle-gates omvatten herstelnegatieven. | Afzonderlijke latency-/fairnessmetingen onder representatieve concurrerende belasting, zonder veiligheidscontroles te verwijderen. Wachtcorrectheid is geen contextswitch-p95. |
| Allocaties en geheugen | Historisch vijf memory-gasten op `b6074c3176db2a6fbe2b6fe42f07f139f153a649`, met normaal/NX/guard/peer/OOM. Vijf rapporthashes opnieuw gecontroleerd. De normale run telt cumulatief 6030 gegroeide pagina's en 24698880 nulgevulde bytes (`docs/verification/full-release-memory-b6074c3/normal/memory-report.json:237-256`). | Cumulatief gereserveerde pagina's zijn geen gelijktijdige werkset of host-RSS. Geen allocator-p95/fragmentatieprofiel, totale desktopbaseline of 24-uursgeheugengroei bewezen. |
| Opslag en netwerk | Begrensde DMA-/requestinterfaces en functionele cold-recovery-/TCP-gates in `.github/workflows/native-system.yml:57-112`. Geen representatieve IOPS-/doorvoermeting in de gelezen systeemharnassen. | Dataset, blokgrootte, read/write/flush-verhouding, queue depth, cachetoestand, concurrency, foutlast, bytes/s en p50/p95. `cache=writeback`-VM-opslag apart houden van fysieke duurzaamheid/prestaties (`tools/run_window_session.py:233-234`). |
| Energie en thermiek | E1-producteis, E2-ACPI-parser/firmwarefixture, E3-HLT/PIT en historische VM-CPU. | Geen E7 watt/joule/temperatuur/batterij-/slaapwekmeting aangetoond; geen E8. Toestemming voor fysieke apparaten valt buiten deze audit. |

### Digestbenchmark: historische uitkomsten, geen snelheidsverhaal

Actuele bron gebruikt vier warm-upcalls per variant, negen geroteerde rondes
van zestien calls: `kernel/src/m2/m3/cap_bench.rs:8-10`,
`kernel/src/m2/m3/cap_bench.rs:105-131`.
De meetklok is diagnostische RDTSC:
`kernel/src/m2/m3.rs:1916-1924`. Geen omzetting naar fysieke cycles of
wandklokmicroseconden, geen bewijs van scheduler-/interruptbelasting.

Historisch rapport op `8365f14e1c3fae90a781b836405836906ee785ab`:
`docs/verification/full-release-pr58-v3-components/benchmark/cap-bench-report.json:1-19`,
`docs/verification/full-release-pr58-v3-components/benchmark/cap-bench-report.json:625-737`.
Negen rondes, totalen en minima/maxima zijn opnieuw uit de opgeslagen
rondetijden gecontroleerd. REF/DIRECT = 0,98089125920736; REF/ZERO =
0,49291684236814204. DIRECT en ZERO zijn dus in die proef niet sneller dan REF.
De REF-batchrange 6142890-16037866 TSC-eenheden laat bovendien forse spreiding
zien; een ratio van sommen is geen tail-latencysamenvatting.

Het oudere acht-rondenrapport op `48b51878c03b5b73449f563735c7854b24926fc4`
is een andere bron/workload:
`docs/verification/full-release-cap-bench-48b5187/result.json:355-414`.
Die resultaten mogen niet worden samengevoegd tot een actuele benchmark.
De huidige runner bevriest bron-/firmware-/binaryinputs
(`tools/run_cap_bench.py:169-213`), wat waardevolle provenance biedt, maar
deze audit heeft hem niet uitgevoerd.

## Vervolgprioriteiten en afhankelijkheden - voorstellen, niet uitgevoerd

1. **PE-01:** sluit de bestaande recorder aan op de echte native driver,
   nadat profiel, bron/imagehash, drie starts, warm-up, acties/richtingen en
   regressieregels zijn bevroren. Behoud volledige beeld-, epoch-, counter-,
   opslag-, herstel- en foutbinding; laat mislukkingen meetellen. Bewijsdoel:
   de bestaande E1-workload uitvoeren, niet de budgetten versoepelen.
2. **PE-03/04:** leg een kleine representatieve gecombineerde workload vast
   met vooraf afgesproken RAM-/allocatie-/I/O-grenzen en observatiekosten.
   Afhankelijk van bruikbare apps/opslag/netwerk en een vaste eigen-kernelimage;
   een testbackend of hosttool mag het ontbrekende productwerk niet overnemen.
3. **PE-05/06:** meet lange idle en actief/idle-overgangen, IRQ-/wakeaantallen
   en invoer na rust op dezelfde bron/configuratie. Bepaal pas daarna of een
   wijziging aan polling nodig is. Afhankelijk van correcte brokerwakeups,
   deadlines, control/STOP en herstel; geen ongecontroleerde tickless-ingreep.
4. **PE-02:** fysieke energie-/thermiek-/slaapwekacceptatie pas op expliciet
   geautoriseerde ondersteunde hardware met meetinstrument, kalibratie,
   workloadduur en herstelroute. ACPI-keten en deviceherstel zijn afzonderlijke
   implementatieafhankelijkheden. VM-CPU blijft een andere grootheid.
5. **PE-07:** behoud de digestbenchmark als diagnostiek; voeg pas voor een
   concrete wijziging een passend vooraf gekozen regressiebudget toe.
   Geen productiepad kiezen uitsluitend op deze microbenchmark.

Releaseblokkerende afhankelijkheden blijven de bindende productgates, niet
deze lijst als nieuwe roadmap. FR-R02 vraagt honderd koude starts en 24 uur
gecombineerd gebruik: `docs/FULL-TEST-RELEASE.md:231`.

## Uitgevoerde controles en commandoregistratie

De shellaanroepen in de onderstaande tabel eindigden met **exit 0**. De view/glob/rg-
tools leveren geen numerieke procesexitstatus; hun resultaten zijn als
lees-/zoekuitkomsten gebruikt, niet als geslaagde tests.

| Controle | Exacte shellopdracht of verwijzing naar hieronder |
|---|---|
| Begincontrole | `pwd && git branch --show-current && git rev-parse HEAD && git status --short && git worktree list --porcelain && git cat-file -t beafdedfc188154fe7285848fbd411e651013f54 && git show-ref --verify refs/remotes/origin/copilot/add-os-specialist-agents` |
| Pinning | `git status --porcelain=v1 && git worktree list --porcelain` (privé worktreepad niet gepubliceerd); de overige exacte stappen zijn in de lokale audituitvoer vastgelegd. |
| Historische CPU/hashcontrole | `python3 -B - <<'PY'` met uitsluitend standaardbibliotheek: gzip/SHA256/JSON/Decimal/statistics/Path; formule en resultaten staan hierboven. Asserties: aggregatehash, leesbare hash, JSON-gelijkheid, 12 gedecomprimeerde hashes, native gate/raw33, creatietijden, telleroptellingen en afwijking kleiner dan 1e-9. Geen outputbestand. |
| Historische profiel-/benchmark-/memorycontrole | `python3 -B - <<'PY'` met gzip/hashlib/json/Path; controleerde zes rapporten op native_gate/passed/raw33/13 fasen en identiek profiel/QEMU-hash; negen rondetotalen/min/max en vijf memoryrapporthashes. Gevolgd door `git status --short && git diff --stat && git rev-parse HEAD`. Geen outputbestand. |

De twee stdin-Pythonprogramma's en hun volledige stdout staan in het
sessietooltranscript (shell-ID's 1564 en 1582); geen losse scripts of logs in
de repository aangemaakt. Voor een compacte exact herhaalbare kernberekening
zonder de historische code te importeren:

```sh
python3 -B - <<'PY'
import gzip, json
from decimal import Decimal, getcontext
from pathlib import Path
from statistics import median
getcontext().prec = 45
p = Path("docs/verification/gui-idle-20261102/windows-ab/aggregate-original.json.gz")
runs = json.loads(gzip.decompress(p.read_bytes()), parse_float=Decimal)["runs"]
arms = {"A": [], "B": []}
for r in runs:
    c = r["measurement"]["cpu"]
    b, a = c["before"]["process"], c["after"]["process"]
    seconds = Decimal(a["total_100ns"] - b["total_100ns"]) / 10000000
    wall = (a["read_started"] + a["read_completed"]
            - b["read_started"] - b["read_completed"]) / 2
    arms[r["arm"]].append(100 * seconds / wall)
print(100 * (1 - median(arms["B"]) / median(arms["A"])))
PY
```

Deze compacte kernberekening is ook werkelijk uitgevoerd: exit 0, uitvoer
`88.2600281961856578321234544035668928426679451`.

De eerste rapportcontrole ontdekte vier buiten-EOF-regelverwijzingen
(47 versus 55 regels in de skillroute, twee 357 versus 369 in de recorder,
56 versus 60 in het memory-gaterapport). Het eerste Pythondeel faalde met
AssertionError; de samengestelde shell eindigde desondanks op 0 doordat de
volgende opdrachten doorgingen. De vervolgaanroep die alle vier opsomde
eindigde expliciet met **exit 1**. Die fouten zijn hersteld; bovendien is
een inhoudelijk verkeerd sessietijdencitaat naar de juiste regels 196-202
verplaatst. Deze gefaalde rapportchecks zijn geen OS-testfouten en worden
niet als geslaagde controles gepresenteerd.

Finale rapportcontrole gebruikt exact:

```sh
python3 -B - <<'PY'
import re, subprocess
from pathlib import Path
report = Path("docs/audits/subagents/14-performance-energy.md")
refs = re.findall(r"`([.\w/-]+\.(?:md|rs|py|json|yml)):(\d+)(?:-(\d+))?`",
                  report.read_text())
for name, first, last in refs:
    assert 1 <= int(first) <= int(last or first) <= len(Path(name).read_text().splitlines())
assert subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip() == "beafdedfc188154fe7285848fbd411e651013f54"
assert subprocess.check_output(["git", "diff", "--name-only"], text=True) == ""
assert subprocess.check_output(["git", "diff", "--cached", "--name-only"], text=True) == ""
assert subprocess.check_output(["git", "ls-files", "--others", "--exclude-standard"], text=True).splitlines() == [str(report)]
assert not any(line.rstrip() != line for line in report.read_text().splitlines())
print("OK:", len(refs), "geldige pad/regelcitaten; alleen auditrapport; bron gepind")
PY
git diff --check
sha256sum docs/audits/subagents/14-performance-energy.md
```

Uitkomst van de finale citaten-/schrijfgrenscontrole: **exit 0**, 53 geldige
pad/regelcitaten, exact gepinde HEAD, geen tracked of staged wijzigingen en
uitsluitend dit untracked rapport. `git diff --check`: **exit 0**; de aanvullende
whitespacecontrole leest ook het untracked rapport. SHA256-berekening:
**exit 0**; de definitieve hash wordt buiten het rapport overgedragen om een
zelfreferente bestandshash te vermijden.

Niet-bestaande zoekpaden zijn niet als ontbrekende implementatie uitgelegd:
eerste gerichte zoekpogingen naar `tools/run_network.py`,
`kernel/src/m2/m3/{block,net,block_driver,arch}.rs` en
`sdk/examples/window-session/src/{compositor.rs,bin}` gaven skipmeldingen
of een zoekfout. Daarna zijn de echte `network.rs`, `system.rs`,
`drivers/virtio-blk/src/lib.rs`, `m3.rs` en window-session `main.rs` gelezen.
Een zoekopdracht over beide ontbrekende block/net-paden gaf expliciet een fout,
geen groen resultaat. De werkelijke bestaande systeemharnassen
`tools/test_system.py`, `tools/test_net_system.py`, `tools/test_tcp_system.py`,
`tools/test_memory_system.py`, `tools/test_resource_pressure.py` gaven geen
matches voor `benchmark|throughput|iops|p95|p99|latency`; dit is begrensde
zoekdekking, geen bewijs dat nergens in de repository een meting bestaat.

**Niet uitgevoerd:** builds, Rust-/Python-testsuites, nieuwe benchmarks,
QEMU-/WHPX-/TCG-runs, actuele CI/PR-opvragingen, fysieke hardware-/energie-/
thermiekmetingen, gebruikersproeven, formele evaluatie of menselijke audit.
Geen downloads, installaties, externe diensten, credentials, persoonlijke data,
devices of destructieve proeven gebruikt. Historische absolute padstrings
zijn uitsluitend metadata gebleven.

## Aannames, risico's en eindgrens

- De gehashte historische gegevens worden als opgeslagen proefverslag
  beoordeeld. Hashgelijkheid controleert niet opnieuw pixels, diskbytes of
  waarheid van Win32-proceswaarnemingen.
- De onderzochte native bronnen zijn aanwezig, maar hun actuele gedrag op
  `beafdedf` is in deze audit niet uitgevoerd. Historische groene gates blijven
  bij hun eigen commits; geen runtimegroen voor deze bron geclaimd.
- De bron bevat voldoende aanknopingspunten voor gerichte vervolgmetingen.
  Ontbrekende latency-/duur-/energieacceptatie blijft een echte blocker voor
  dagelijkse bruikbaarheid en releaseclaims, niet voor voltooiing van dit
  afgebakende auditrapport.
- Geen blocker voor het read-only onderzoek gevonden. De toegestane
  opdrachtgrens verbiedt hier de noodzakelijke nieuwe product- en fysieke
  metingen. Geen optimalisatie, fix of toestemming daarvoor gevraagd.
