# Nieuw OS — eigen kernel en native documentopslag

## Actuele bouwstap: M6b

De eigen Rust-kernel heeft nu een native objectdienst in ring 3. Twee geïsoleerde
testapps kunnen via capability-IPC objecten aanmaken, lezen, bijwerken en
verwijderen. De dienst schrijft via echte virtio-I/O; revisies en verwijderrecords
blijven behouden na een koude herstart. Apps krijgen uitsluitend hun eigen
domeinendpoint, geen rechtstreekse blok- of DMA-toegang.

De volledige M6b-gate op broncheckpoint `20f7ed2` is geslaagd: **18 controlegroepen
en 14 echte VM-invocaties**, inclusief zes koude recoveries en twee externe
QEMU-kills. Ook 22 Rust-tests en 26 M6b-harnastests slagen. Ruwe uitvoer, hashes en
de onafhankelijke schijfcontrole staan in [verification/m6b](docs/verification/m6b/).
Deze afgebakende proef gebruikt vier objecten van maximaal 128 bytes en een
journal van 32 records. Namen, ruimtehergebruik, encryptie en native NAS/desktop
volgen als aparte integratiestappen; dit is nog geen consumentensysteem.

```sh
python3 -B tools/build.py --kernel-feature milestone-m6b
python3 -B tools/run_m6b.py --case normal --smoke
```

De runner gebruikt een eigen synthetisch wegwerpvolume en sluit na de
acceptatieproef af. Volledig contract: [M6B-OBJECT-STORAGE.md](docs/implementation/M6B-OBJECT-STORAGE.md).
Voortgang, regressies en resterende grenzen: [PROJECT-STATE.md](docs/PROJECT-STATE.md).

## Nieuwe richting: internet en persoonlijke NAS-opslag

**Vervolg N02:** [hybride TLS-transportproef](services/nas/TRANSPORT.md) met echte wederzijdse authenticatie en X25519MLKEM768 op loopback. De gecontroleerde [beveiligings-/PQC-blauwdruk](docs/implementation/VERIFIABLE-SECURITY-AND-PQC.md) onderscheidt formele eigenschappen, eigen kernel, CHERI-onderzoek en native integratie. Dit is hostbewijs; geen native internetstack, formeel bewezen compleet OS of volledige postkwantumauthenticatie. Uitvoering en actuele testresultaten: [PROJECT-STATE.md](docs/PROJECT-STATE.md).

De gebruikerswijziging van 27 oktober 2026 voegt veilige internettoegang, eigen NAS en gedeelde ruimte bij vrienden/familie toe. Inhoud en documentnamen worden op de gebruikersclient versleuteld; de NAS-beheerder kan ruimte intrekken en zijn kopie verwijderen zonder gebruikerssleutel of toestemming. De uitvoerbare hostproef staat in [services/nas/](services/nas/README.md); het browserprototype blijft een simulatie. Contract en native integratiecriteria: [NAS-SHARING-AND-INTERNET.md](docs/implementation/NAS-SHARING-AND-INTERNET.md). Dit voegt nog geen netwerkstack aan de kernel toe en verandert de M6a-bewijsstatus niet.

De eigen Rust-kernel start via de eigen UEFI-loader in QEMU, draait geïsoleerde gebruikersprocessen en verwerkt capabilities/IPC. M5 voegt een echte ring-3-blokdienst met alleen-lezen virtio-I/O toe. De definitieve M1–M5-suites slagen: **77 controlegroepen en 64 echte VM-runs**, plus een normale M5-eindstart. Materiaal en qsim blijven afzonderlijke hostprototypes. Dit is een experimentele ontwikkelversie met synthetische testprogramma's.

- **Actuele voortgang en open controles:** [PROJECT-STATE.md](docs/PROJECT-STATE.md).
- **Starten en reproduceren:** [DEVELOPMENT.md](docs/DEVELOPMENT.md).
- **Opgeslagen testbewijs:** [docs/verification/](docs/verification/).
- **Productprototype:** [handleiding](ui/prototype/README.md).
- **Quantumsimulator:** [handleiding](experiments/quantum/README.md).

M6b staat op [codex/m6b-object-storage](https://github.com/simonvanieperen/nieuw-os/tree/codex/m6b-object-storage), boven op `codex/pqc-transport-foundation` ([PR #8](https://github.com/simonvanieperen/nieuw-os/pull/8)). De status staat per spoor in PROJECT-STATE.md. [PR #3](https://github.com/simonvanieperen/nieuw-os/pull/3) bewaart de M5-historie; de oude testmatrix staat in [ACCEPTANCE.md](docs/verification/m5/ACCEPTANCE.md).

De oudere M5-leesproef blijft selecteerbaar via `python tools/build.py --kernel-feature milestone-m5` en `python tools/run.py --smoke --milestone m5`. Er is nog geen interactieve native desktop. Kerneltransport en QEMU-device zijn vertrouwd; geen IOMMU- of fysieke-hardwarebewijs. De uitvoerroute staat in DEVELOPMENT.md. Publieke publicatie en definitieve licentie blijven een afzonderlijk besluit volgens LICENSE-PLAN.md.

## Oorspronkelijk startpakket (historische context)

Onderstaande tekst beschrijft de aangeleverde voorbereiding van26 oktober2026. Uitspraken daarin over ontbrekende code zijn historisch; de actuele status staat hierboven.

# Een zelfstandig OS — startpakket voor Codex
Onderzoeksdatum: 26 oktober 2026. Revisie 5: de eerste onderzoeken zijn uitgevoerd; concrete bouwspecificaties voor kernel, product, quantum en Europese eisen staan klaar. Werknaam: `nieuw-os`; geen merkonderzoek uitgevoerd.

Dit pakket is een voorbereid project met gekozen oplossingen, exacte interfaces, instructies en een uitvoerbare eerste ontwikkelopdracht. Begin met [docs/implementation/START-CODING.md](docs/implementation/START-CODING.md). Het bevat **geen werkend besturingssysteem** en geen bewezen veilige kernel. Er is nog geen boot-, prestatie- of beveiligingstest uitgevoerd.

## Begin
1. Lees [START-HIER.md](START-HIER.md).
2. Open de uitgepakte projectmap als werkproject in Codex.
3. Plak [PROMPT-START.md](PROMPT-START.md) in de eerste ontwikkelchat.
4. Bewaar actuele voortgang in [docs/PROJECT-STATE.md](docs/PROJECT-STATE.md), niet alleen in de chat.

## De gekozen richting
Een eigen Rust-kernel met een kleine vertrouwde kern, expliciete bevoegdheden (capabilities) en afzonderlijke systeemdiensten. Eerst x86_64 in QEMU met UEFI; later één aantoonbaar ondersteund apparaat tegelijk. De eerste architectuur is als bouwbasis gekozen, maar nog niet geïmplementeerd. Privacygrens herzien op 27 oktober 2026: OS-data lokaal of end-to-end versleuteld op een expliciet gekoppelde eigen NAS of NAS van vrienden/familie; internet naar die NAS vereist afzonderlijke toestemming. De beheerder mag opslag intrekken/verwijderen zonder de inhoud te kunnen ontsleutelen. AI-inferentie blijft op het apparaat zonder cloudfallback. Derdenapps blijven installeerbaar met afzonderlijk begrensde rechten.

Eigen kernel en eigen productervaring betekenen niet: zelf een compiler, encryptiealgoritme of netwerkstandaard uitvinden. Bestaande ontwikkelgereedschappen, formele methoden, open standaarden en zorgvuldig gekozen bibliotheken mogen worden gebruikt. Een bestaande Linux-, BSD-, XNU- of seL4-kernel als basis valt buiten de huidige opdracht.

## Inhoud
| Bestand | Functie |
|---|---|
| START-HIER.md | Project, Git, GitHub/alternatieven en modelkeuze |
| PROMPT-START.md | Volledige kopieerbare startopdracht |
| PROMPT-VERVOLG.md | Herstarten zonder belangrijke context te verliezen |
| AGENTS.md | Projectbrede werkinstructies |
| boot/, kernel/, drivers/, services/, ui/, tests/, docs/AGENTS.md | Regels per map; nog geen subsystemen |
| .codex/agents/*.toml | Optionele rollen volgens geraadpleegde actuele Codex-documentatie |
| .codex/config.example.toml | Voorbeeld voor maximaal drie subagents; geen automatische installatie |
| docs/implementation/START-CODING.md | Gekozen bouwbasis en ingang naar de vijf concrete specificaties |
| docs/implementation/FIRST-BATCH.md | Directe opdrachten B00/B01/P02/Q01 met exclusieve schrijfgebieden |
| docs/implementation/VERIFICATION.md | Werkelijk uitgevoerde controles en nog ontbrekend bewijs |
| tools/preflight.py | Read-only inventarisatie van de ontwikkelomgeving |
| docs/ARCHITECTURE.md | Systeemontwerp, interfaces en hardwarekeuzes |
| docs/ROADMAP.md | Kleine mijlpalen met controleerbare acceptatiecriteria |
| docs/SECURITY-AND-QUANTUM.md | Dreigingsmodel, isolatie, updateketen en postkwantumstrategie |
| docs/QUANTUM-RESEARCH.md | Quantumprogramma’s, lokale QPU en eigen OS op hybride quantumhardware |
| docs/LOCAL-PRIVACY-CONTRACT.md | Bindende NAS-, lokale AI- en derdenapp-eisen met acceptatietests |
| docs/DESIGN-AMBITION.md | Eigen interactiemodel, hoogwaardige afwerking en parallel productonderzoek |
| docs/PRODUCT-AND-SOVEREIGNTY.md | Privacy, Europese autonomie, originaliteit en gebruikerservaring |
| docs/KNOWLEDGE-MAP.md | Benodigde kennis, bronnen en leerroute |
| docs/OSDEV-LEERROUTE.md | Jouw OSDev-link, gekoppeld aan de mijlpalen |
| docs/CODEX-WORKFLOW.md | Coördinatie, rollen, review en bewijsvoering |
| docs/TEST-STRATEGY.md | Testlagen en betrouwbaar rapporteren |
| docs/DECISIONS.md | Eerste ontwerpbesluiten en open keuzes |
| docs/DEPENDENCIES.md | Inventaris en herkomstcontrole |
| docs/TASK-TEMPLATE.md | Afgebakende opdracht voor één agent |
| SECURITY.md | Voorlopig beveiligingsbeleid |
| LICENSE-PLAN.md | Licentiekeuze vóór publicatie |

De eerste productkeuze (Materiaal), het bootcontract, de kernelgrenzen en de lokale quantumproef zijn besloten. Een inventarisatiescript staat in `tools/preflight.py`. Nog niet aanwezig: loader/kernel, productprototype en Rust-quantumsimulator. De startprompt laat Codex deze bouwen; hij vraagt geen nieuwe algemene onderzoeksronde.

Bronnen staan per onderzoeksbestand in APA-achtige bibliografische vorm met directe URL's en raadpleegdatum. De implementatiespecificaties bevatten gekozen projectbesluiten; achtergrondbestanden bevatten ook voorstellen voor latere fasen. Keuzes en budgetten zijn geen door bronnen bewezen prestaties. De documentatie dekt de start; zij vervangt geen processorhandleiding, securityreview of ervaren OS-ontwikkelaar.

