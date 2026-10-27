# Actuele projectstatus
Bijgewerkt: 26 oktober 2026, revisie 5. De gebruiker vroeg de onderzoeken nu uit te voeren zodat Codex meteen kan coderen. Die voorbereiding is afgerond in docs/implementation/.
Git-commit: niet beschikbaar; er is geen nieuwe remote repository aangemaakt of gepubliceerd.

## Geleverd en besloten

- Boot/toolchain: eigen loader/kernel, vastgelegde ELF-layout, BootInfo, overdracht en testcommando's.
- Kern M2–M4: adresruimten, syscallgrens, capabilities, IPC, quota en 16 acceptatiegevallen.
- Product P0/P1: drie concepten vergeleken; Materiaal gekozen; zeven schermen en P2-bouwpad gespecificeerd.
- Quantum Q0: simulatorprofiel gekozen; exacte JSON/gates/limieten/fixtures en Q1-bouwpad; PQC-keuzes apart.
- Europese toepasselijkheidsmatrix en acht engineeringvoorwaarden; geen conformiteitsverklaring.
- Startprompt omgezet naar implementatie, exclusieve agenttaken en read-only tools/preflight.py toegevoegd.

## Werkelijk gecontroleerd

Zie [implementation/VERIFICATION.md](implementation/VERIFICATION.md). Onder meer: primaire bronnen, Python/TOML-syntaxis, documentconsistentie, BootInfo-/IPC-layout met Python ctypes, quantumfixtures en kleurcontrastberekeningen. Preflight uitgevoerd in deze onderzoeksomgeving: Python/Git aanwezig; Rust/rustup/Cargo/QEMU/OVMF ontbreken. Dit zegt niets over Simons eigen computer.

## Nog niet geïmplementeerd of getest

Geen eigen loader, kernel, gebruikersproces, capability-engine, UI-prototype of Rust-quantumsimulator. Geen Rust-build, echte QEMU-boot, hardwarecompatibiliteit, gebruikerstest of securityaudit. De geselecteerde compiler/dependency/firmwarecombinatie is nog niet samen uitgevoerd. Nooit voorbereiding als een behaalde M1 melden.

## Eerstvolgende taak

Voer PROMPT-START.md en [implementation/FIRST-BATCH.md](implementation/FIRST-BATCH.md) uit:
B00 inventariseert en bereidt de werkelijke ontwikkelomgeving voor; B01 schrijft de bootproef; P02 schrijft ui/prototype/; Q01 schrijft experiments/quantum/. Start geen nieuwe algemene concept- of toolchainstudie. Ontbrekende bootsystemen blokkeren onafhankelijk productwerk niet.

## Bindende grenzen en latere open punten

OS-gegevens alleen apparaat/eigen NAS binnen eigen LAN; AI-inferentie alleen apparaat; geen externe OS-cloud/relay/telemetrie; afzonderlijke apprechten. Deze eisen zijn nog niet geïmplementeerd. De ambitie voor een eigen product en lokale hybride quantumhardware blijft behouden.

MPL-2.0 is onderzocht en aanbevolen; officiële projectlicentie en licentie-inventaris vóór publieke publicatie afronden. Menselijke OS-/securityreview, concrete QPU/controller en fysieke referentiehardware nog niet geregeld. Die latere keuzes blokkeren de eerste lokale codetaak niet.

## Na iedere mijlpaal bijwerken

Taak-ID; Git-basis/eindcommit; gewijzigde bestanden; wat werkt; werkelijk uitgevoerde commando's en exitstatus; log-/artifactpaden; niet uitgevoerde controles; besluiten; resterende risico's; eerstvolgende kleine taak.
