# TASK-CAP-DIGEST-TCG-2 — rol 19: digestprestaties

**Status:** de ene gereserveerde native run stopte vóór de meetcalls op de
manifestpreconditie. Er zijn dus geen nieuwe TSC-prestatiedata. Dit rapport
bewaart het vooraf vastgelegde protocol, de mislukte gate, ruwe outputs,
historische baseline en hervatstap.
Manager bevestigde deze afbakening op branchbasis
`ca45a7dabe6a040921de03bdc2fad3e5045a75ba`; schrijfgebied is uitsluitend
`docs/verification/performance-19/**`. Taak voedt K07/R02 en FR-K02/R02
als prestatie-/duuracceptatiebewijs, maar deze diagnostiek op zichzelf sluit
geen criterium en kent geen scorepunten toe.

## Meetvraag en grens

De concrete vraag is of de directe berichtwoord- of nulreeksvariant de
kernel-digestkosten verlaagt ten opzichte van de referentie in de bestaande
Materiaal-prestatieonderzoekslijn. Dit is een read-only diagnose vóór PIT en
userspace, niet de input-naar-frame-latency, GUI-taakduur of productacceptatie.
De eerdere rode GUI-keten blijft afzonderlijk rood; de digestmeting alleen kan
de oorzaak van die volledige vertraging niet vaststellen.

De huidige werkboom was schoon op branch
`simonvanieperen-shiny-waffle`, commit
`ca45a7dabe6a040921de03bdc2fad3e5045a75ba`. Voor deze bron is de
benchmarkimage gebouwd en één keer gestart, maar de meetpoort faalde vóór de
eerste timingcall. De bestaande meetdefinitie staat in
[`CAP-DIAGNOSTIC-DIGEST.md`](../../contracts/CAP-DIAGNOSTIC-DIGEST.md).

## Vooraf vastgelegd protocol

Gebruik het bestaande `native-cap-digest-bench`-profiel, zonder de workload,
warmup, aantal rondes of GUI-timeout te wijzigen:

| Onderdeel | Vastgelegde instelling |
|---|---|
| Gastwerk | Eén echte vier-processenmanifestgraaf: 21 handles, 29 nodes, 9 open endpoints |
| Varianten | REF, DIRECT en ZERO; dezelfde Engine, rotatie van meetvolgorde per ronde |
| Warmup/herhaling | 4 warmupcalls per variant; 9 rondes × 16 calls = 144 gemeten calls per variant |
| Meetpunt | Ongeserialiseerde RDTSC-batchverschillen; vóór timer en userspace; geen andere gastactoren |
| Profiel | Bestaand runnerprofiel: `pc-q35-8.2`, `qemu64`, één vCPU, single-thread TCG, 256 MiB, UEFI/OVMF en eigen lege wegwerpdisk |
| Meetuitvoer | Totaal, minimum en maximum per 16-callbatch; opgeslagen waarden per ronde en werkelijke variantvolgorde |
| Correctheidscriteria | Exacte manifest/workload, hashpariteit, ongewijzigde volledige Stats/invarianten, complete eindmarkers, raw QEMU-exit 33 en geen timeout |
| Tijdsdoel | Contract noemt minder dan 5 s gastmeetwerk als doel; de runner rapporteert duur maar valideert deze grens niet automatisch |

TSC-waarden zijn emulator-/TCG-diagnostiek, geen fysieke CPU-cycli, exacte
per-calllatentie of realtimegarantie. Voor een herhaling moeten hostbelasting,
gelijktijdige builds en andere VM's vóór en na worden geregistreerd. De runner
moet eigen geïsoleerde `NOS_BUILD_DIR`, `NOS_TARGET_DIR` en
`CARGO_TARGET_DIR` gebruiken. Er zijn geen aparte koude/warme productruns;
deze diagnose gebruikte de vier interne warmupcalls en één verse QEMU-boot.
De hostbelasting en aantallen relevante
concurrente processen zijn vlak vóór en na de ene gaststart vastgelegd in de
contextbestanden. Dit blijft een eenmalige QEMU/TCG-run, geen onafhankelijk
herhaalde acceptatietest.

## Historische native baseline — andere bron en protocolversie

`CAPABILITY-DIGEST.md` documenteert een echte run op commit
`48b51878c03b5b73449f563735c7854b24926fc4`, source fingerprint v3
`d54262733b9e2aca9b6ec1385d2c4ba7f0087d91cabab96d3f2a722edffca60d`,
QEMU 10.0.13 / pc-q35-8.2 / qemu64 / één CPU / TCG / 256 MiB. Die oudere run
had **8** geroteerde rondes (128 calls per variant), niet de huidige 9 rondes
(144 calls). Daarom is dit context, geen baseline die de huidige bron of het
huidige protocol aftekent.

| Variant | Som batch-TSC | Minimum | Maximum |
|---|---:|---:|---:|
| REF | 36.998.312 | 4.203.914 | 5.101.902 |
| DIRECT | 43.931.758 | 5.070.624 | 6.485.388 |
| ZERO | 77.735.944 | 8.745.048 | 10.404.420 |

Op die ene historische TCG-run lag DIRECT circa 18,7% en ZERO circa 110,1%
hoger dan REF in totaalsom. De vastgelegde rapportage bevat min/max en totaalsom,
maar geen per-ronde ruwe reeks om p50/p95 te berekenen. De broncontracten
rapporteren bovendien geen hostbelasting of concurrerende workloads voor die
run. Geen percentiel of spreidingsclaim wordt hier uit gemiddelden afgeleid.
Dit ondersteunt alleen dat de eerdere varianten in die specifieke emulatorrun
geen snelheidswinst lieten zien. Het stelt niet vast waarom de GUI-keten faalde.

## Uitvoering op de huidige bron

| Controle | Resultaat |
|---|---|
| `python3 -B tools/preflight.py --strict` zonder gedeelde toolroute | **Exit 2.** De standaardworktree-inventory ontdekte Rust, QEMU/OVMF, `ld.lld` en host `cc` niet. |
| `python3 -B tools/dev.py python3 -B tools/preflight.py --strict` met de afgestemde read-only toolroute | **Exit 0.** Inventory vond pinned Rust/Cargo 1.90.0, beide targets, QEMU 10.2.1, OVMF, `ld.lld` en GCC 15.2.0. Dit valideert inventarisatie, geen build of ABI. Ruwe JSON: [`preflight.json`](preflight.json). |
| `python3 -B tools/build.py --kernel-feature native-cap-digest-bench` | **Exit 0, 8,15 s** (`CARGO_NET_OFFLINE=true`, Cargo `--locked`; eigen build-/targetmappen). Een `timer_init`-dead-codewaarschuwing hoort bij het profiel dat vóór de timer meet. Gearchiveerd: [`build.log`](build.log), [`build-wall-time.txt`](build-wall-time.txt). |
| `python3 -B -m unittest tools.tests.test_cap_bench` | **Exit 0:** 18 hostharnastests, 0,497 s. Dit test de rapportparser en input/artifactbinding met HOST_ONLY-fixtures en mock-QEMU; geen benchmarkcode is in een gast uitgevoerd. |
| `python3 -B tools/run_cap_bench.py` | **Runner exit 1; QEMU raw exit 35; `timed_out=false`; runnerduur 2,408981447 s, buitenste walltijd 2,66 s.** De eigen kernel startte, maar stopte bij `CAPBENCH:FAIL:FOUR_PROCESS_MANIFEST`. Er volgden geen benchmarkcalls of TSC-uitvoer. De doelduur <5 s is hierdoor niet geaccepteerd of gemeten. |

De gastuitvoer bevat `BOOT:EBS_OK`, kernel-/BootInfo-/KS-markers en
`SYSTEM:MANIFEST:PASS`, gevolgd door `CAPBENCH:FAIL:FOUR_PROCESS_MANIFEST`.
De QEMU-runner heeft de fout als `Native benchmark did not exit successfully`
opgeslagen. QEMU stderr bevat daarnaast een niet-fatale melding over het niet
vinden van `kvmvapic.bin`; de guest bereikte aantoonbaar de kernel en de
expliciete benchmarkguard. De ruwe gegevens staan in
[`run-result.json`](run-result.json), [`raw-serial.txt`](raw-serial.txt) en
[`qemu-stderr.txt`](qemu-stderr.txt); de hostconsole is
[`runner-console.txt`](runner-console.txt). De machine-/hostcontext vóór en na
staat in [`run-context-before.json`](run-context-before.json) en
[`run-context-after.json`](run-context-after.json).

**Waarschijnlijke oorzaak, geen gemeten counterwaarden:** de actuele
`kernel/src/m2/m3/system.rs`-manifest maakt naast de domeinendpoints ook twee
B02-persistencekanalen (`key_admin_ingress` en `init_key_reply`). Elk kanaal
maakt één endpoint met send-/receiveafleidingen. De benchmarkguard verwacht
nog de oudere vaste waarden 21 handles, 29 nodes en 9 endpoints; de extra
kanalen zouden statisch +4 live handles, +6 nodes en +2 endpoints verklaren.
Dit is een bronafgeleide hypothese, niet een runtime-uitlezing: de guard stopt
voordat die tellers worden uitgeprint. De fout betekent dus dat de fixture
niet overeenkomt met de huidige geïntegreerde procesgraaf; ze is geen
digestprestatieregressie. Er is niets aan productiecode, rechten of meetgrens
geoptimaliseerd.

De eerste, losse QEMU-versieprobe vóór deze run faalde exit 127 doordat die
probe de afgesproken `LD_LIBRARY_PATH` niet meekreeg (`libjpeg.so.8` ontbrak).
Er was toen geen VM-start. Dezelfde bestaande QEMU werkte daarna met de
goedgekeurde librarypaden; de geslaagde preflight en latere gueststart
bevestigen dat dit niet de oorzaak van raw exit 35 was.

Voor de ene gaststart registreerde de host 32 CPU's, loadavg `3.53 3.42 2.89`,
23.531.552 KiB beschikbaar geheugen en nul QEMU-, Cargo-, Rustc- of GCC15-
processen. Na afloop: loadavg `2.41 2.98 2.80`, 23.514.972 KiB beschikbaar
geheugen en opnieuw nul van die processen. Dit beperkt bewijs voor de host-
interferentie; het sluit andere systeemactiviteit niet uit. De run gebruikte
Linux/WSL2 + QEMU TCG, niet een Windows-native runner of fysieke CPU.

## Volgende stap

De ene release-4-slotrun is beëindigd; na release-4-ACK is het enige slot
doorgeschoven naar agent 13 voor diens ene probe. Ik heb geen herhaling
gestart. Volgende stap voor de kernel-/harnaseigenaar: vergelijk
de exacte runtime-Stats van de huidige procesgraaf met de historische guard en
werk de diagnostische fixture/verwachting bij op basis van de geïntegreerde
B02-kanalen, met alle invariant- en rechtenchecks intact. Leg daarvoor opnieuw
protocol/bron vast en reserveer een nieuw slot vóór een nieuwe VM. Totdat die
poort slaagt is er op `ca45a7d` geen digest-baseline en kan REF/DIRECT/ZERO niet
worden gerangschikt. Een product-UI-run blijft een afzonderlijke acceptatie
met input→status/frame en taakvoltooiing.

Scorevoorstel via R&D 17 / releaseagent 4: **geen rubricwijziging**. Deze
mislukte manifestpoort en de hosttests leveren geen prestatie-/duuraftekening;
voortgang blijft **34%**. Het onderzoek is een bijdrage aan K07/R02 en
FR-K02/R02, geen fysieke score of criteriumsluiting. Zie ook
[`FULL-TEST-RELEASE.md`](../../FULL-TEST-RELEASE.md) voor de blijvende
performance- en duuracceptatiegates.
