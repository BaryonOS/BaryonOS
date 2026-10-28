# I11 — actuele native qsim-smoke

## Uitkomst en bronbinding

Eén echte QEMU-VM slaagt met ruwe exitcode `33` en de volledige strikte reeks van 28 regels vanaf `BOOT:EBS_OK`. De runner accepteert beide gescheiden ring-3-processen, Bell- en zesqubittest, de bestaande annulering- en deadlinezelftests, resourcegrenzen, procesisolatie en werkelijke framevrijgave.

- Commit: `dbd35b04d742b18d66b40fdd65d8a9b900656cac`.
- Tree: `421bb96102779d01d9cb4e6231d686aea417d63a`.
- Source fingerprint v3: `c5f3d9316433ba09ba4b886985ffa8b28643d67c72a9ebb0e2b0136362dc7a9f`.
- HEAD, tree en source fingerprint zijn vóór build en na VM gelijk; iedere fase controleert opnieuw de bron.
- Werkelijke native VMs in deze bundel: **1**. Er is geen volledige M3-suite herhaald.
- Run: `build/parallel-followup-qsim/run/native-qsim-e3745ad261`.
- Profiel: `development-package; pc-q35-7.2/qemu64/TCG/1CPU/256MiB`.
- PID 1: 253 checkpoints, 7216 nuttige arenabytes, maximale werkkloof 448.
- PID 2: 253 checkpoints, 7216 nuttige arenabytes, maximale werkkloof 448.
- 4 echte timerinterrupts, 504 syscalls en 96 vrijgegeven frames.

Alle zeven strict-runnercontroles slagen. De controle bindt de werkelijk gebruikte kernel, loader, OVMF CODE/VARS-template, qsim-user-ELF, scalarbuildrapport en disassembly aan bron en hashes; het exacte user-ELF is in de kernel aanwezig. De privésnapshot behoudt de onveranderde inputbestanden. `serial.txt` en `stderr.txt` zijn ruwe kopieën van de VM-uitvoer en zijn bij archivering opnieuw met het geaccepteerde rapport vergeleken.

## Vergelijking met de eerdere acceptatie

De kernel en qsim-user-ELF zijn byte voor byte gelijk aan de artifacts uit de eerdere native acceptatie op 3e41bbf. Alleen de loaderhash verschilt. Directe vergelijking vindt precies 12 afwijkende bytes, uitsluitend in de COFF- en debugtimestamps en de CodeView RSDS-buildidentifier. De timestamps zijn 2026-10-28 15:36:50 UTC en 2026-10-28 16:21:30 UTC, passend bij beide builds. De volledige uitvoerbare `.text`-sectie en alle overige bytes buiten die benoemde metadatavelden zijn identiek. `loader-comparison.json` bewaart de offsets, sectiehashes, volledige onafhankelijke `objdump -x`-uitvoer en de controle dat beide echte bestanden ongemoeid bleven. Alleen kopieën in het geheugen zijn voor deze vergelijking van metadata ontdaan; geen gehasht artifact is gewijzigd. De nieuwe loaderhash is rechtstreeks gebonden aan de actuele VM. De bredere bronvingerafdruk is veranderd door andere geïntegreerde bron- en harnesswijzigingen; de 18 qsim-inputs zijn ongewijzigd.

| Artifact | SHA-256 huidige run |
| --- | --- |
| Kernel | `84c75a9504e923426db1f86ac6d1e24b62d70024e4bebfe52b571b67106ac72d` |
| Qsim-user-ELF | `3a20a6e8566b899ffec4939a03a9a7e5f970ca2f5b0c7d84fd85f5dccc860a5a` |
| UEFI-loader | `1e13496735d3e2cbc80fac646379dc3c335c40a127b82aec22bf887535e24640` |

`execution.json` bevat zowel de oude als nieuwe hashes en de 18 afzonderlijke qsim-inputvergelijkingen. De eerdere artifacts zijn uitsluitend gelezen en zijn na deze run opnieuw onveranderd bevonden.

## Reproduceren

Gebruik Rust 1.90.0 met de targets `x86_64-unknown-none` en `x86_64-unknown-uefi`, de gepinde selectorcompatibele launchers en de vastgelegde QEMU/OVMF-installatie. De launchercompatibiliteit is nodig omdat de bestaande buildketen `+1.90.0` gebruikt.

```bash
source /tmp/nieuw-os-par-tools/env.sh
export CARGO_HOME=/tmp/nieuw-os-par-tools/cargo-home
export CARGO_BUILD_JOBS=2
export NOS_BUILD_DIR=/workspace/scratch/debaa3b6a81c/nieuw-os/build/parallel-followup-qsim
export NOS_TARGET_DIR=/tmp/nieuw-os-followup-qsim-target
export CARGO_TARGET_DIR=/tmp/nieuw-os-followup-qsim-target
export NOS_QEMU_MACHINE=pc-q35-7.2
export NOS_SDK_RUNTIME=0
python3 -B tools/build.py --kernel-feature native-qsim
python3 -B tools/run_qsim.py
```

Voer dit uit vanuit de repository op de genoemde commit. Gebruik voor een nieuwe onafhankelijke bewijscapture een eigen leeg build- en targetpad. De oorspronkelijke I11-capture weigert bestaand bewijs te overschrijven; de opgeslagen scripts documenteren die uitvoering en de daaropvolgende archivering.

## Bewijs en grenzen

`acceptance.json` bevat de stricte checks en waarnemingen; `result.json` bevat de echte QEMU-commandoregel, raw exit, timing en artifactbinding. `execution.json` legt de bovenliggende commands, toolwrapper- en firmwarehashes, versie-uitvoer, broncontrole en artifactvergelijking vast. `raw-streams.json` bewaart de exacte UTF-8-bytes van buildlogs, disassembly en symbooluitvoer inclusief oorspronkelijke eindregels, met lengte en SHA-256. De native binaries blijven buiten deze Git-bundel.

Dit bewijst de bestaande begrensde qsim-referentietaken in twee native gebruikersprocessen. Het is geen algemene quantum-jobdienst, geen hardware-QPU, geen GUI-flow en geen fysieke hardwaretest. Het nieuwe job-requestcodec wordt hierdoor niet als actieve dienst geaccepteerd. De disassemblycontrole is een gerichte scalarinstructiegate, geen formeel bewijs. Er wordt geen roadmapcriterium of scorewijziging uitsluitend op deze smoke geclaimd.
