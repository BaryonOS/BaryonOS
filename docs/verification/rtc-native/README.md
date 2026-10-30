# RTC-NATIVE-1 — echte CMOS-reads op de eigen kernel

30 oktober 2026. **Begrensde native afhankelijkheid afgerond; volledige
testrelease en productie-TimeProvider blijven open.** Basis `5c334f9`,
uitvoeringsbron `c02675e21cbbaf32294298f46ce1cb394b360895`,
branch `simonvanieperen-expert-disco`. Het volgende bewijscheckpoint verandert
uitsluitend documentatie; code-/artifactbinding gebeurt via source inputs v3:
`0ef69cbf5d371a809f16fc1ad0e35e9cc535acfb73de058bdc4f14c88348d8a6`.
De score blijft **34% -> 34%**; geen FR- of rubriccriterium gesloten.

## Profiel en grenzen

Opt-in `native-rtc`, eigen x86_64-loader/kernel, eigen IDT en pagetables,
IF=0, 1 CPU, `pc-q35-8.2/qemu64/TCG/256MiB`, disposable ESP/firmwarekopie.
Rust `1.90.0 (1159e78c4 2025-09-14)`, QEMU
`10.2.1 (Debian 1:10.2.1+ds-1ubuntu3.3)`, OVMF-paar uit Ubuntu
`ovmf-generic 2025.11-3ubuntu7.3`. Dit is een gemeten ontwikkelvariant,
niet het historische 10.0.13-profiel of het volledige VM-FR-image.

| Gebonden invoer | SHA-256 |
|---|---|
| RTC-kernel | `f37880e68bed5e27f9056e7b58899ee1a530c5cf7ab20d9157b1c0373c2af921` |
| Loader | `7fe38cf8c04faee27595bde879243ee801c8420a8143cbf6c77c2a8c9917a3de` |
| OVMF CODE 4M | `874f15a7c9110e78a89db159f0f68fc97e9076b74777520de1c235426f758aa7` |
| OVMF VARS-template 4M | `5d2ac383371b408398accee7ec27c8c09ea5b74a0de0ceea6513388b15be5d1e` |

De gast schrijft alleen de CMOS-indexpoort 0x70 en leest data uit 0x71.
Geen tijd-/modewrite, register-C-read, netwerk, hostschijf of firmware-runtimecall.
De D/NMI-Enabled-startselectie is expliciet eigen platformbeleid, geen
teruggelezen firmwaretoestand. Herstel wordt met de echte D-dataread zonder
reselectie gecontroleerd; NMI-policy is een code-invariant, geen fysiek gemeten
garantie. Er is geen algemene I/O-capability of productiekloksyscall toegevoegd.
Zie [contract](../../implementation/RTC-WALLTIME.md) en ADR-057.

## Werkelijk uitgevoerde controles

De lokale geïsoleerde tools staan onder `.tools/nieuw-os/`. De gebruikte shell
laadde `. .tools/nieuw-os/rtc-env.sh`; de relevante instellingen zijn:

```sh
export LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/emulator/usr/lib/x86_64-linux-gnu"
export PATH="$PWD/.tools/nieuw-os/emulator/usr/bin:$PATH"
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$PWD/.tools/nieuw-os/cc"
export LIBRARY_PATH="$PWD/.tools/nieuw-os/emulator/usr/lib/x86_64-linux-gnu:$PWD/.tools/nieuw-os/emulator/usr/lib/gcc/x86_64-linux-gnu/15"
export OS_OVMF_CODE="$PWD/.tools/nieuw-os/emulator/usr/share/OVMF/OVMF_CODE_4M.fd"
export OS_OVMF_VARS="$PWD/.tools/nieuw-os/emulator/usr/share/OVMF/OVMF_VARS_4M.fd"
export NOS_QEMU_MACHINE=pc-q35-8.2
export NOS_BUILD_DIR="$PWD/build/rtc-native"
export NOS_TARGET_DIR="$PWD/target/rtc-native"
export CARGO_TARGET_DIR="$PWD/target/rtc-host"
```

`tools/dev.py` ontdekt de lokale rustup/cargo-installatie zelf. De lokale
`cc`-wrapper roept de uitgepakte Ubuntu-GCC15 aan met de eigen `--sysroot`;
de usr-merge-aliassen `lib -> usr/lib` en `lib64 -> usr/lib64` zijn lokaal
aangelegd. De QEMU-link in `.tools/nieuw-os/qemu/` heeft `share` naar de
uitgepakte ROMs. Geen sudo, globale configuratie of hosttoolinstallatie
gewijzigd. Een gewone CI-host gebruikt zijn eigen geïnstalleerde tools.

| Exact commando, na bovenstaande exports | Resultaat | Bewijs |
|---|---|---|
| `python3 -B tools/preflight.py --strict` | exit 0 | `preflight.json` |
| `python3 -B tools/dev.py cargo +1.90.0 test --locked --manifest-path drivers/rtc/Cargo.toml` | exit 0; 1 unit + 10 integratie | `logs/rtc-host-final.txt` |
| `python3 -B tools/dev.py cargo +1.90.0 clippy --locked --manifest-path drivers/rtc/Cargo.toml --all-targets -- -D warnings` | exit 0 | `logs/rtc-clippy-final.txt` |
| `python3 -B tools/dev.py cargo +1.90.0 fmt --all -- --check` | exit 0 | `logs/format-final.txt` |
| `python3 -B -m unittest discover -s tools/tests -v` | exit 0; 95 hosttests, waarvan 9 RTC | `logs/tools-tests-final.txt` |
| `python3 -B -m unittest discover -s tests -v` | exit 0; 414 hosttests | `logs/tests-host-final.txt` |
| `python3 -B tools/build.py --kernel-feature native-rtc` | exit 0; echte ELF/PE-build/inspectie | `build-manifest.json`, `logs/build-rtc-final.txt` |
| `python3 -B tools/run_rtc.py --case all` | exit 0; 5 echte VM's; raw33, geen timeout | `rtc-report.json`, `rtc-runs/` |
| `CARGO_TARGET_DIR="$PWD/target/rtc-exclusivity" python3 -B tools/dev.py cargo +1.90.0 check --locked --release -p os-kernel --target x86_64-unknown-none --features native-rtc,framebuffer` | verwachte exit 101; expliciete exclusiviteitsfout | `logs/exclusivity-negative.txt` |
| `python3 -B tools/run_rtc.py --case epoch` | exit 1; echte firmwaredatum 2070, niet 1970 | `epoch-rejected-report.json`, `logs/epoch-firmware-rejected.txt` |
| `python3 -B tools/run_rtc.py --case century` | exit 1; echte firmwaredatum 2000, niet 2100 | `century-rejected-report.json`, `logs/century-firmware-rejected.txt` |

De laatste RTC-host/lint/fmt/tools-bundel duurde 5 hele seconden; de brede
414-testsuite 4,954 s. De laatste bewaarde vijf VM's duurden samen
9,156 s (afzonderlijke monotone durations in `rtc-report.json`).
Geen cache-hit als testresultaat aangemerkt.

| Geval | Werkelijk UTC gelezen | Werkelijk Unix |
|---|---|---|
| millennium | 2000-01-01 00:00:01 | 946684801 |
| leap | 2000-02-29 12:34:57 | 951827697 |
| current | 2026-10-30 12:00:01 | 1793361601 |
| y2038 | 2038-01-19 03:14:09 | 2147483649 |
| future | 2096-02-29 00:00:01 | 3981312001 |

Iedere VM doet twee echte readings met RAM-floor, readbudget-uitputting na
precies een dataread, callerfloor `UnixUtc::MAX`-weigering, onbekende basis/
ontbrekende eeuw zonder I/O en een succesvolle reread na de fouten.
Alle vijf melden zichtbaar `UnauthenticatedRtc`, `RTC:NEGATIVE:PASS`,
`RTC:RESTORE:PASS` en raw33. Python datetime controleert onafhankelijk zowel
de datum/Unixconversie als base..base+60 seconden. Dit is geen bewezen live
middernachtovergang of realtime-nauwkeurigheid. De hostfixture-uitvoer in de
Python-testlogs is uitsluitend parserbewijs, niet een extra VM.

## Integratieregressies en onafhankelijke uitvoering

RTC-V-M2 is door een aparte verifieragent uitgevoerd, zonder codewijzigingen,
in eigen build-/targetmappen. Geen menselijke audit. Exact:

```sh
export NOS_BUILD_DIR="$PWD/build/rtc-m2-verifier"
export NOS_TARGET_DIR="$PWD/target/rtc-m2-verifier"
export CARGO_TARGET_DIR="$PWD/target/rtc-m2-verifier-host"
python3 -B tools/test_milestones.py --through m2
```

Exit 0, 17,457 s, acht groepen inclusief 38 bestaande Rusttests.
Zeven echte VM's: eigen CR3/WX/frames, RO-, NX- en absent-pagefout, gecontroleerde
kernelfout, doublefault via IST en gewone M1-nacontrole. Vijf raw33 en twee
**verwachte** raw35; geen timeout. De M2:FAIL-marker in de gecontroleerde
kernelfout wordt niet tot een normale succesboot omgelabeld.
`m2-regression.json`, `m2-runs/`, `build-*-manifest.json` en
`logs/m2-{verifier,host-tests.*}.txt` bewaren de observaties.
De verifier verwijderde aanvankelijk vijf nog ongecommitteerde
hoofdagent-archiefbestanden buiten zijn schrijfgebied. Geen bron/buildbewijs
ging verloren; het archief is daarna opnieuw uit echte buildrapporten
samengesteld en beide firmwarediagnostieken plus de vijf RTC-boots zijn opnieuw
uitgevoerd. De bewaarde reports zijn de herstelde, gecontroleerde uitvoering.

De hoofdagent voerde de bestaande CI-M6c-smoke uit met:

```sh
export NOS_BUILD_DIR="$PWD/build/rtc-m6c-regression"
export NOS_TARGET_DIR="$PWD/target/rtc-m6c-regression"
export CARGO_TARGET_DIR="$PWD/target/rtc-host"
python3 -B tools/build.py --kernel-feature milestone-m6c
python3 -B tools/run_m6c.py --case normal --smoke
```

Exit 0, build/run samen 5 hele seconden, één echte VM met raw33 en echte
disk-oraclecontrole. Bewijs: `m6c-run/`, `m6c-build-manifest.json`,
`logs/m6c-{build,native}.txt`. Alle bewaarde RTC-/M2-/M6c-runs dragen dezelfde
uitvoeringscommit en v3-bronfingerprint. De hoofdagent controleerde hun
rapporten/hashbindingen en de uiteindelijke diff.

## Niet-groene controles, aannames en vervolg

De twee firmwarediagnostieken blijven rood, ondanks een geslaagde kale
read-/restoreproef met raw33: het onafhankelijke orakel weigert de verkeerde
eeuw. De native backend compenseert dit niet door datawrites of eeuwgokken.
Primair brononderzoek: TianoCore EDK2 `edk2-stable202511`,
[PcRtc.c](https://github.com/tianocore/edk2/blob/edk2-stable202511/PcAtChipsetPkg/PcatRealTimeClockRuntimeDxe/PcRtc.c),
`ConvertRtcTimeToEfiTime` en `PcRtcAcpiTableChangeCallback`.
De eerste reconstrueert de eeuw uit de PCD-minimumjaarperiode, de tweede
schrijft die naar het FADT-eeuwregister. Dit verklaart de echte observatie;
geen brede native datumrange of firmware-onafhankelijkheid geclaimd.
Hosttests omvatten wel epoch-/eeuwvectoren en alle 146097 dagen van een
Gregoriaanse 400-jaarcyclus.

De extra poging
`CARGO_TARGET_DIR="$PWD/target/rtc-native" python3 -B tools/dev.py cargo +1.90.0 clippy --locked --release -p os-kernel --target x86_64-unknown-none --features native-rtc -- -D warnings`
gaf exit 101: vier bestaande bevindingen in ongewijzigde M2-pagingregels en
de bestaande main-pointerread, geen RTC-locatie. `logs/kernel-clippy.txt`.
Dit is **geen groene kernelclippy**; de nieuwe workflow claimt dat niet.
Geen unrelated lintreparaties of blanket-allow toegevoegd.

Nieuwe bronreview omvatte adapter-ownership/unsafe, feature-isolatie,
read-only-portgrens, faalpaden, runnerargumenten, bron-/artifactbindingen en
behoud van normale bootsemantiek; geen open RTC-codeblocker gevonden.
De oorspronkelijke werkruimte-doctestblokkade is van de andere sessie en
hier niet gewijzigd. Geen externe dependencyversies veranderd.

**Niet uitgevoerd:** volledige GitHub-CI op het uiteindelijke PR-image,
alle andere subsystem-/GUI-/NAS-/TLS-/updategates, echte RTC-devicefouten,
live kalenderovergangen, IRQ-/SMP-gebruik, userspace-TimeProvider,
authentieke of persistente antirollback-tijd, fysieke hardware en menselijke
audit. Het archief bevat geen firmware-, disk- of ELF-binaries.
`SHA256SUMS` bindt alle bewaarde tekstbestanden:
`sha256sum -c docs/verification/rtc-native/SHA256SUMS` (exit 0 bij checkpoint).
De afgebakende Git-attributen behouden ook firmware-CRLF bytegetrouw;
de hashes worden tegen de opgeslagen Git-blobs gecontroleerd, niet alleen
tegen de lokale werkboom.
De nieuwe workflow voert de vijfkalenderpoort bij PR-uitvoering opnieuw uit;
een lokale test vervangt geen verplichte CI-check.

Na openen is de echte GitHub-RTC-workflow uitgevoerd en groen op
`295b4db9ce978d6b020effce08abdc7535554625` (de runtimebron is gelijk aan
`c02675e`): job 47 s, hosttests/codec/formatter, eigen loader/kernel,
vijf echte kalenderboots en artifactupload allemaal success.
`ci-native-rtc.json` bewaart de API-run-/stepstatus; controlecommando:
`gh run watch 41064380898 --exit-status --interval 10`, exit 0.
De overige verplichte PR-checks liepen bij overdracht nog, niet als groen
aangemerkt. Dit bewijscheckpoint wijzigt geen uitvoerbare code.

Vervolg blijft de geautoriseerde geisoleerde TimeProvider en een expliciet
vertrouwd tijdpad voor certificaatvalidatie. Rollback: revert de opt-in
feature/adapter/runner/workflow met dependencytoevoeging. Standaard M1/M2,
productiediensten, capabilityrechten en historische releasecriteria behouden.
