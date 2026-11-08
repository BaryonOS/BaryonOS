# Uitgevoerde controles en bewijsgrenzen
Onderzoeksdatum: 26 oktober 2026. Dit overzicht betreft revisie 5 van het startpakket, geen geïmplementeerd OS.

## Onderzoek

Vijf afgebakende onderzoeken uitgevoerd: boot/toolchain, kernelisolatie en rechten, productconcept, lokale quantumuitvoering/PQC, Europese eisen. Primaire bronnen en precieze raadpleegbeperkingen staan onderaan elk bijbehorend bestand. OSDev is de leerroute; mechanismen/standaarden zijn waar mogelijk tegen documentatie van de makers gecontroleerd.

Er is een concrete productvergelijking gemaakt, geen gebruikersproef. De gekozen kernel en simulator zijn ontworpen, geen overgenomen implementatie. Onbereikbare exacte versiepagina's en nog niet opgehaalde packages zijn in de specificaties herkenbaar benoemd; bekende versiegegevens worden niet als succesvolle build gepresenteerd.

## Deze controles zijn echt uitgevoerd

| Controle | Resultaat | Wat dit bewijst |
|---|---|---|
| `python3 tools/preflight.py` | Exit 0, valide JSON | Inventaris werkt in de onderzoeksomgeving |
| `python3 tools/preflight.py --strict` | Exit 2 | Ontbrekende Rust-/QEMU-/firmwarebasis wordt zichtbaar als ongereed gemeld; dit is het verwachte resultaat |
| Python `ast.parse` op preflight | Geslaagd | Syntactisch geldige Python; niet alle toekomstige hostvarianten getest |
| Python `tomllib` op vijf TOML-bestanden | Geslaagd | Syntaxis agentconfiguratie; geen bewijs dat iedere Codex-client deze rollen laadt |
| Python `ctypes` BootInfo | 128 bytes, alignment 8; alle 19 offsets gelijk aan spec | Rekenkundig voorgeschreven C-layout; Rust compile-time asserts nog uitvoeren |
| Python `ctypes` IPC | 320 bytes; caps@32, data@64 | Rekenkundig berichtformaat; syscall-/copyveiligheid nog testen |
| Zelfstandige SplitMix64/Bell-referentieberekening | seed0: 8 shots `00:5,11:3`; 4096 shots `00:2082,11:2014` | Deterministische fixtureverwachting; geen werkende Rust-engine/QPU |
| JSON-parse drie quantumvoorbeelden | Geslaagd | Voorbeelden zijn syntactisch JSON; toekomstige typed validator nog testen |
| sRGB-contrastberekening zes kleurparen | 13,56; 5,15; 7,65; 6,75; 5,81; 3,77 | De vaste kleurparen kloppen; geen gerenderde WCAG-audit |

De genoemde Pythonberekeningen beëindigden met exit 0. De hoofdcontrole heeft de ABI-/IPC-layout, Bellfixtures en kleurverhoudingen opnieuw zelfstandig berekend. Een extra AI-review is documentcontrole, geen onafhankelijke menselijke securityaudit.

## Overdracht en pakketcontrole

44 projectbestanden, 30 lokale Markdownverwijzingen gecontroleerd; geen ontbrekend doel en alle codefences gesloten. De aparte documentreview vond een dubbele README-alinea en onduidelijk eigenaarschap van buildtools; beide zijn gecorrigeerd. P02 gebruikt overal ui/prototype/. De hoofdagent beheert rootconfiguratie en tools; B01 de loader/kernel/protocol-/parsercrates; Q01 zijn eigen geneste workspace.

## Omgeving van dit onderzoek

Linux x86_64, Python 3.12.14, Git 2.51.1, host-C-compiler GCC 13.3.0. `rustc`, `cargo`, `rustup`, `qemu-system-x86_64`, `clang` en `ld.lld` ontbreken in PATH; de onderzochte standaardlocaties bevatten geen OVMF. Er zijn geen binaries, toolchaincomponenten of firmware geïnstalleerd tijdens deze onderzoeksopdracht. Inventariseer de uitvoeromgeving bij een volgende ontwikkelsessie opnieuw. Dit onderzoek beschrijft de Linux-onderzoeksomgeving; mijn eigen computer is niet geïnspecteerd.

Het preflightscript zoekt alleen naar aanwezigheid en versies. Het installeert niets, draait geen VM, zoekt niet naar NAS-apparaten en verandert geen firmware. `RUSTUP_AUTO_INSTALL=0` voorkomt impliciete toolchaindownload via rustup-proxies. Bron: https://rust-lang.github.io/rustup/environment-variables.html .

## Niet uitgevoerd

Rust-dependencyresolutie, compiler-/linkertests, OVMF/QEMU-boot, EBS-retryproef, echte procesisolatie, capabilitytests, UI-browserproeven, schermlezerproeven, gebruiksonderzoek, simulator-Rusttests, crypto-integratie, fysieke QPU-aansturing, hardwareports en juridische conformiteitsbeoordeling.

Dit zijn uitvoeropdrachten met eigen acceptatiecriteria, geen redenen om de algemene onderzoeksronde opnieuw te starten. De eerstvolgende resultaten zijn code en echte testlogs volgens FIRST-BATCH.md.
