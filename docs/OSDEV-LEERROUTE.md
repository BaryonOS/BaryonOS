# OSDev als leerroute
Ik heb de [Expanded Main Page van OSDev](https://wiki.osdev.org/Expanded_Main_Page) voor het onderzoek aangeleverd. De hoofdpagina, Getting Started en Beginner Mistakes zijn geopend op 26 oktober 2026.

Gebruik OSDev als onderwerpindex en uitlegbron. De wiki noemt zelf dat zij geen volledige kopieerbare handleiding is. De eerste index verwijst onder meer naar vereiste voorkennis, booting, UEFI/BIOS, compilers, interrupts en debugging. Deze pagina's geven een route naar vragen; ze leveren geen beveiligingsbewijs voor onze code.

## Voorgestelde koppeling aan dit project
| Moment | Zoeken via de OSDev-index | Wat Codex vervolgens moet opleveren |
|---|---|---|
| M0 | Required Knowledge, Getting Started, Beginner Mistakes | Persoonlijke kennislacunes + gekozen target/toolchain |
| M1 | UEFI, Object Files, Boot Sequence, Kernel Debugging | Bootcontract onderbouwd met UEFI- en Rust-documentatie |
| M2 | Interrupts, geheugenbeheer en paging | Invarianten en grensgevallen, gecontroleerd tegen processorhandleiding |
| M3/M4 | Processen, scheduling, syscall- en IPC-onderwerpen | Afgebakend interfacecontract en negatieve isolatietests |
| M5+ | Driver- en bestandssysteemonderwerpen | Exacte device-/protocolspecificatie en foutafhandeling |

Niet alle genoemde subartikelen zijn in dit vooronderzoek inhoudelijk beoordeeld. Laat Codex het gekozen artikel openen wanneer die taak start en de exacte pagina/versie raadpleegbaar vastleggen.

## Instructie aan Codex
1. Lees alleen het OSDev-artikel dat de huidige mijlpaal helpt.
2. Noteer welke architectuur, bootmethode en programmeertaal het voorbeeld veronderstelt.
3. Toets details aan primaire documentatie: UEFI, Intel/AMD/Arm/RISC-V, Rust, QEMU en OASIS.
4. Vertaal principes naar ons eigen ontwerp; verander de target niet stilzwijgend om een tutorial te kunnen kopiëren.
5. Controleer herkomst en licentie vóór code wordt overgenomen.
6. Schrijf de eigen test vóór je een claim uit de tutorial als gerealiseerd behandelt.

OSDev is aanvullend. De officiële hardware-/protocolspecificatie is leidend bij conflicten. Bestaande tutorials kunnen een andere doelgroep of legacy-bootroute hebben; actualiteit en toepasbaarheid worden per pagina beoordeeld.

## Repositoryskill en lokale kennisbank
De repository bevat de Agent Skill `.agents/skills/os-dev/SKILL.md`. Die bundelt de nieuwste revisie per pagina uit de aangeleverde OSDev Wiki-snapshot (**3.054 wikipagina's**) en alle **78 fysieke pagina's** van *The little book about OS development*, samen met een lokale bronindex en zoekhelper. Gebruik bijvoorbeeld `python .agents/skills/os-dev/scripts/zoeken.py "paging"` of een bron-ID zoals `W001461`/ `B053`.

De skill vervangt geen primaire CPU-, firmware- of protocolspecificatie en het toevoegen van de kennisbank telt niet als gerealiseerde OS-functionaliteit.

## Bronnen
OSDev Wiki contributors. (z.d.). *Expanded Main Page*. Geraadpleegd op 26 oktober 2026, van https://wiki.osdev.org/Expanded_Main_Page
OSDev Wiki contributors. (z.d.). *Getting Started*. Geraadpleegd op 26 oktober 2026, van https://wiki.osdev.org/Getting_Started
OSDev Wiki contributors. (z.d.). *Beginner Mistakes*. Geraadpleegd op 26 oktober 2026, van https://wiki.osdev.org/Beginner_Mistakes
