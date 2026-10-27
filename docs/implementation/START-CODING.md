# Bouwbesluiten — Codex kan nu implementeren
Revisie 5, 26 oktober 2026. De eerste onderzoeken zijn uitgevoerd en keuzes zijn vastgelegd. Dit is de korte ingang voor de volgende ontwikkelchat. Gebruik [FIRST-BATCH.md](FIRST-BATCH.md) als uitvoeropdracht.

## Wat is besloten?

| Spoor | Gekozen bouwbasis | Exact contract |
|---|---|---|
| Boot | Eigen Rust-kernel plus eigen kleine UEFI-loader; Rust 1.90.0, r-efi 7.1.0, x86_64/QEMU TCG | [BOOT-AND-TOOLCHAIN.md](BOOT-AND-TOOLCHAIN.md): toolchain, ELF, BootInfo, entry, runcommando en fouttests |
| Kernveiligheid | Kleine capability-gebaseerde kern; eerst één CPU en geïsoleerde gebruikersprocessen, begrensde IPC | [KERNEL-AND-SECURITY-CONTRACT.md](KERNEL-AND-SECURITY-CONTRACT.md): geheugen, syscalls, rechten, quota en negatieve tests |
| Product | **Materiaal**: jouw documenten/collecties staan centraal, apps zijn verwisselbare gereedschappen met begrensde toegang | [PRODUCT-SPEC.md](PRODUCT-SPEC.md): vergelijking van drie concepten, zeven schermen, toestanden, vormgeving en toegankelijkheid |
| Quantum | Lokale Rust-statevector-simulator, qsim-v0, maximaal 12 qubits; core los van hostadapter | [QUANTUM-SPEC.md](QUANTUM-SPEC.md): gates, JSON, rekenregels, limieten, fixtures en 17 tests; daarnaast PQC-keuzes |
| Europa en publicatie | Concrete privacy-, toegankelijkheids-, export-, supplychain- en herstelvoorwaarden | [EUROPE-AND-RELEASE.md](EUROPE-AND-RELEASE.md): toepasselijkheidsmatrix en EU-01 t/m EU-08 |

Dit is een **gekozen en onderzochte basis**, geen reeds geteste OS-implementatie. Het nieuwe OS neemt geen bestaande kernel over. De ontwikkelhost, QEMU, UEFI-standaarden en ondersteunende bibliotheken zijn gereedschappen. Een UI-prototype in een browser en een quantumproef op de ontwikkelhost zijn afzonderlijke hulpmiddelen; hun uiteindelijke OS-port blijft een echte vervolgstap.

## De eerste handeling

Open de projectroot en voer uit:

```bash
python3 tools/preflight.py
```

Dit script bestaat en inventariseert alleen. Bestaande build-/runbestanden worden eerst gecontroleerd; ontbreken ze, dan maakt de hoofdagent deze gedeelde root-/tools-bestanden voor B01 volgens de bootspecificatie. Begin daarna direct met B01, P02 en Q01. De hoofdagent bezit gedeelde rootconfiguratie; builders hebben exclusieve paden uit FIRST-BATCH.md. Geen onbeperkte nieuwe architectuurverkenning.

Referentieprofiel: QEMU 10.0.14, machine pc-q35-10.0, CPU qemu64, TCG, één CPU en 256 MiB. Het bootspecificatiedocument beschrijft de praktische route met beschikbare compatibele QEMU/OVMF-pakketten en het exacte referentieprofiel afzonderlijk. Log afwijkingen en echte firmwarehashes. Ontbrekende hardwareversnelling is geen blocker voor de TCG-proef.

## Voorrang en bewijs

Expliciete gebruikersvereisten en [../LOCAL-PRIVACY-CONTRACT.md](../LOCAL-PRIVACY-CONTRACT.md) blijven leidend: internet met afzonderlijke rechten, E2EE-opslag op expliciet gekoppelde eigen NAS of NAS van vrienden/familie, remote toegang alleen na peergebonden toestemming, admin-verwijderrecht zonder inhoudssleutel, AI op het apparaat, geen leverancierscloud of telemetrie. De implementatiespecificaties concretiseren oudere onderzoeksvoorstellen; zij herroepen geen privacy- of productambitie.

Alle bronnen, gekozen grenzen en nog onbewezen eigenschappen staan per specificatie. [VERIFICATION.md](VERIFICATION.md) registreert wat in dit onderzoek daadwerkelijk is gecontroleerd. Heropen een besluit alleen bij een concrete technische tegenstrijdigheid, aangetoonde fout of nieuwe gebruikersinstructie. Repareer die gericht, leg haar vast en ga door met bouwen.

## Eerst nog te realiseren

- De loader/kernel en echte boottests; het aanwezig zijn van deze documentatie is geen bootbewijs.
- Het interactieve Materiaal-prototype en eerste gebruiks-/toegankelijkheidsproeven.
- De Rust-simulator en CLI-tests; de analytische fixtures zijn wel gecontroleerd.
- Daarna de volgende kleine kernel-, native-product- en quantumhardwarestappen uit [../ROADMAP.md](../ROADMAP.md).

PQC, quantumprogramma's uitvoeren en het OS op een hybride klassiek/QPU-systeem laten draaien zijn drie verschillende doelen. De eerste simulator wist die verdere ambitie niet, maar bewijst haar ook niet. Hetzelfde geldt voor brede apparaatondersteuning: de eerste x86_64-boot is één testplatform, geen beperking van het einddoel.

