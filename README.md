# Een zelfstandig OS — startpakket voor Codex
Onderzoeksdatum: 26 oktober 2026. Revisie 5: de eerste onderzoeken zijn uitgevoerd; concrete bouwspecificaties voor kernel, product, quantum en Europese eisen staan klaar. Werknaam: `nieuw-os`; geen merkonderzoek uitgevoerd.

Dit pakket is een voorbereid project met gekozen oplossingen, exacte interfaces, instructies en een uitvoerbare eerste ontwikkelopdracht. Begin met [docs/implementation/START-CODING.md](docs/implementation/START-CODING.md). Het bevat **geen werkend besturingssysteem** en geen bewezen veilige kernel. Er is nog geen boot-, prestatie- of beveiligingstest uitgevoerd.

## Begin
1. Lees [START-HIER.md](START-HIER.md).
2. Open de uitgepakte projectmap als werkproject in Codex.
3. Plak [PROMPT-START.md](PROMPT-START.md) in de eerste ontwikkelchat.
4. Bewaar actuele voortgang in [docs/PROJECT-STATE.md](docs/PROJECT-STATE.md), niet alleen in de chat.

## De gekozen richting
Een eigen Rust-kernel met een kleine vertrouwde kern, expliciete bevoegdheden (capabilities) en afzonderlijke systeemdiensten. Eerst x86_64 in QEMU met UEFI; later één aantoonbaar ondersteund apparaat tegelijk. De eerste architectuur is als bouwbasis gekozen, maar nog niet geïmplementeerd. Harde privacygrens: OS-data uitsluitend op het apparaat of eigen NAS op eigen LAN; AI-inferentie op het apparaat zonder cloudfallback. Derdenapps blijven installeerbaar met afzonderlijk begrensde rechten.

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
