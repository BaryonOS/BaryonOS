# Ontwerpbesluiten
Status revisie 5: onderstaande principes zijn projectbesluiten. Concrete keuzes voor de eerste bouwfase staan in implementation/START-CODING.md en bijbehorende specificaties. Bouwuitvoerbaarheid is nog niet getest.

| ID | Besluit | Reden | Herzien als |
|---|---|---|---|
| ADR-001 | Eigen kernel; geen bestaande kernel als basis | Expliciete gebruikerswens | Simon bewust de projectscope verandert |
| ADR-002 | Rust no_std + minimaal assembly | Kleine expliciete onveilige grens | Target/tooling dit aantoonbaar verhindert |
| ADR-003 | x86_64 QEMU q35, UEFI, software-emulatie baseline | Controleerbare eerste testomgeving | Praktische toolchainbeperking wordt aangetoond |
| ADR-004 | Kleine capability-gebaseerde microkernel als richting | Autoriteit expliciet en TCB klein houden | Prototypes aantoonbare problemen met complexiteit/latency tonen |
| ADR-005 | Standaarden en gecontroleerde bibliotheken toestaan | Zelf crypto/alle tools bouwen vergroot risico en scope | Herkomst/licentie/onderhoud onacceptabel is |
| ADR-006 | Lokaal account, geen externe OS-telemetrie, OS-data alleen apparaat/eigen NAS op eigen LAN | Expliciete gebruikerscorrectie | Alleen na expliciete gewijzigde gebruikersopdracht |
| ADR-007 | PQC-voorbereiding vanaf ontwerp; integratie pas met echte use case | Protocol- en sleutelmigratie voorkomen | Relevante standaarden/implementaties wijzigen |
| ADR-008, herzien | Product-/interactieonderzoek parallel vanaf M0; echte systeemintegratie volgt bewezen kernfuncties | Originaliteit en hoogwaardige UX moeten het systeem mede sturen | Bijstelling op experimenten, geen stille reductie tot standaarddesktop |
| ADR-009 | Geen kernelbrede claims over formeel bewijs | Eigen code erft geen seL4-bewijs | Afgebakend daadwerkelijk geverifieerd resultaat bestaat |
| ADR-010 | AI-inferentie uitsluitend op apparaat; geen netwerkrecht of cloudfallback | Expliciete gebruikerscorrectie | Alleen na expliciete gewijzigde gebruikersopdracht |
| ADR-011 | Installeerbare derdenapps met gescheiden bestands-, NAS- en netwerkrechten | Toegankelijk ecosysteem zonder impliciet vertrouwen | Acceptatietests tonen ontbrekende grens |
| ADR-012 | Quantum als derde onderzoeksspoor vanaf M0: PQC, lokale QPU-uitvoering en hybride systeemport | Oorspronkelijke ambitie expliciet behouden | Bijstelling alleen met onderbouwd onderzoek; privacycontract blijft gelden |

## Gekozen uitvoeringsbesluiten — 26 oktober 2026

| ID | Besluit | Bewijsgrens / herziening |
|---|---|---|
| ADR-013 | Eigen UEFI-loader met r-efi 7.1.0; Rust 1.90.0; x86_64/QEMU TCG | Versies onderzocht; combinatie nog niet gebouwd. Exact bootcontract in BOOT-AND-TOOLCHAIN.md |
| ADR-014 | Eerste kernel: vaste ELF-layout en gedeelde BootInfo; daarna begrensde processen, IPC en capabilities | Details en negatieve tests in KERNEL-AND-SECURITY-CONTRACT.md; geen isolatiebewijs vóór uitvoering |
| ADR-015 | Materiaal als eerste productrichting; P2 lokale HTML/CSS/JS-proef | Vergelijking uitgevoerd, geen gebruikersproef. Herzie op concrete gebruiksobservaties |
| ADR-016 | Q1 safe Rust statevector-core met qsim-v0, maximaal 12 qubits, hostadapter apart | Analytische fixtures gecontroleerd; geen Rust-/QPU-/OS-portbewijs |
| ADR-017 | Later ML-KEM-768 en ML-DSA-65 via beoordeelde implementaties/protocolbinding | Geen eigen cryptoprimitives, geen volledige bootketenclaim |
| ADR-018 | EU-01 t/m EU-08 als technische acceptatie-eisen; toepasselijkheid wetgeving per release | EUROPE-AND-RELEASE.md bevat conditionele matrix, geen conformiteitsclaim |

## Template voor nieuw besluit
ID, datum, status, context, alternatieven, besluit, onderbouwing/bronnen, gevolgen, security-invariant, testbewijs, migratie/rollback en eigenaar. Schrijf wat de keuze zou weerleggen; vermijd besluiten zonder toetsbare aannames.
