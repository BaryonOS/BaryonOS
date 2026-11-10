# OSDev as a learning route
The [OSDev Expanded Main Page](https://wiki.osdev.org/Expanded_Main_Page) was supplied as a research source. The main page, Getting Started and Beginner Mistakes were opened on 26 October 2026.

Use OSDev as a topic index and explanatory source. The wiki itself states that it is not a complete guide to copy. Its initial index links to prerequisite knowledge, booting, UEFI/BIOS, compilers, interrupts and debugging, among other topics. These pages provide a route to questions; they do not provide security evidence for the project code.

## Proposed mapping to this project
| Stage | Search through the OSDev index | What Codex must then deliver |
|---|---|---|
| M0 | Required Knowledge, Getting Started, Beginner Mistakes | Individual knowledge gaps plus the chosen target/toolchain |
| M1 | UEFI, Object Files, Boot Sequence, Kernel Debugging | Boot contract supported by UEFI and Rust documentation |
| M2 | Interrupts, memory management and paging | Invariants and edge cases checked against the processor manual |
| M3/M4 | Processes, scheduling, syscall and IPC topics | Bounded interface contract and negative isolation tests |
| M5+ | Driver and filesystem topics | Exact device/protocol specification and error handling |

Not all referenced subarticles were substantively reviewed during this preliminary research. Have Codex open the selected article when that task begins and record the exact page/version in a retrievable form.

## Instructions for Codex
1. Read only the OSDev article relevant to the current milestone.
2. Record the architecture, boot method and programming language assumed by the example.
3. Check details against primary documentation: UEFI, Intel/AMD/Arm/RISC-V, Rust, QEMU and OASIS.
4. Adapt principles to the project's own system design; do not silently change the target to copy a tutorial.
5. Check provenance and licensing before adopting code.
6. Write the project's own test before treating a tutorial claim as implemented.

OSDev is supplementary. Official hardware/protocol specifications prevail in conflicts. Existing tutorials may target a different audience or legacy boot route; currency and applicability are assessed for each page.

## Repository skill and local knowledge base
The repository contains the Agent Skill `.agents/skills/os-dev/SKILL.md`. It bundles the latest revision of each page from the supplied OSDev Wiki snapshot (**3,054 wiki pages**) and all **78 physical pages** of *The little book about OS development*, together with a local source index and search helper. For example, use `python .agents/skills/os-dev/scripts/zoeken.py "paging"` or a source ID such as `W001461`/`B053`.

The skill does not replace primary CPU, firmware or protocol specifications, and adding the knowledge base does not count as implemented OS functionality.

## Sources
OSDev Wiki contributors. (n.d.). *Expanded Main Page*. Retrieved October 26, 2026, from https://wiki.osdev.org/Expanded_Main_Page
OSDev Wiki contributors. (n.d.). *Getting Started*. Retrieved October 26, 2026, from https://wiki.osdev.org/Getting_Started
OSDev Wiki contributors. (n.d.). *Beginner Mistakes*. Retrieved October 26, 2026, from https://wiki.osdev.org/Beginner_Mistakes
