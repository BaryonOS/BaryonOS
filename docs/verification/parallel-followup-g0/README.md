# B08 — echte G0-run en gerepareerde herbruikbare capture

## Resultaat

**Geslaagd:** een nieuwe gepaarde graphics-g0-build, één echte geïsoleerde
QEMU-run met ruwe exitstatus 33 en alle tien G0-checks, gevolgd door de
ongewijzigde `tools/capture_graphics.py` inclusief B09-reparatie. De capture
heeft deze nieuwe bewijsmap zelf atomisch gepubliceerd. Daarna zijn alleen
de begeleidende uitvoerlogs, metadata en dit document toegevoegd; de elf
oorspronkelijk geïndexeerde archiefbestanden zijn ongewijzigd.

Uitvoering: 28 oktober 2026, 16:20:48–16:20:57 UTC.
Broncommit: `dbd35b04d742b18d66b40fdd65d8a9b900656cac`.
Bron-tree: `421bb96102779d01d9cb4e6231d686aea417d63a`.
Source-v3: `c5f3d9316433ba09ba4b886985ffa8b28643d67c72a9ebb0e2b0136362dc7a9f`.
HEAD en source-v3 zijn vóór en na iedere stap gecontroleerd.

Werkelijke runmap:
`/tmp/nieuw-os-followup-g0/graphics-g0-15e24b60b7`.
Buildmap: `build/parallel-followup-g0`.
Zowel `NOS_TARGET_DIR` als `CARGO_TARGET_DIR`:
`/tmp/nieuw-os-par-followup-g0-target`.
De test gebruikte twee Cargo-buildjobs en `NOS_SDK_RUNTIME=0`.

## Wat daadwerkelijk werkt

De eigen kernel toont de G0-scène op een framebuffer van **800×600 pixels**.
De runner stuurt via private QMP-pipes achtereenvolgens `a`, `b`, backspace en
`c` naar de PS/2-invoer van de VM. De vijf originele PPM-screenshots tonen de
bijbehorende wijzigingen binnen de invoerregel. Backspace herstelt het gehele
eerdere `a`-beeld byte voor byte. Escape beëindigt de gast normaal met
`G0:PASS`, `M1:PASS` en ruwe QEMU-exitstatus **33**.

De totale gewijzigde rechthoek is `[48, 192, 74, 212]`. De opeenvolgende
wijzigingen zijn `[48, 192, 62, 212]` en driemaal `[60, 192, 74, 212]`.
De seriële uitvoer bevat **425 bytes**, SHA256
`f61350dd1041ed8736a36452f1f4e56821f05f4f11008c9036e327913bfe2596`.
Die ruwe bytes zijn exact gelijk aan het archief en de gerapporteerde stdout.

Het gastprofiel is expliciet `pc-q35-7.2`, QEMU **7.2.22**, TCG met één CPU,
256 MiB RAM, zonder netwerk. De officiële SeaBIOS VGA-ROM is gekopieerd en
gecontroleerd tegen de expliciete SHA256-pin
`cc2f735f19b6318922ac3de9506dee498f149a6b75534f7e5c176d4441a7fa4a`.
De exacte commandoregel, firmwarehashes en elf gekoppelde QMP-uitwisselingen
staan in de oorspronkelijke result-, manifest- en tracebestanden.

## Uitgevoerde controles

| Stap | Uitkomst | Bewijs |
|---|---|---|
| Verse gepaarde loader/kernel-build en artifactinspectie | CLI-exit 0, 2,48 s | `build-logs/`, `build.stdout.txt`, `build.stderr.txt` |
| Echte G0-VM met private QMP-pipe | Ruwe exit 33, tien checks waar, geen fout; CLI 5,58 s | `result.json`, `serial.txt`, `qmp-trace.json`, vijf beelden |
| Gerepareerde herbruikbare capture | CLI-exit 0, 0,47 s, nieuwe bestemming gepubliceerd | `capture.json`, `capture.stdout.txt` |
| Directe archiefcontrole | Alle 16 oorspronkelijke inputhashes en elf archiefhashes gelijk | `execution.json` |
| Onafhankelijke PNG-reconstructie met Pillow 12.3.0 | Alle vijf volledige PPM's byte-exact teruggelezen | `execution.json`, `image-roundtrip.json` |
| Latere read-only observatie | Bron, ruwe inputs en oorspronkelijke archiefbytes opnieuw vergeleken | `final-observation.json` |

Werkelijk gelezen kernel-ELF:
`1c5467ce79246bad48edc9c9f065b86ffb28261835c9a94ff3dffb9de0dc869d`.
Werkelijk gelezen EFI-loader:
`40a39d98edf2b266ffb40f93cef6907897f845201936bf3e9cb96ed1627f2a10`.
De bewaarde buildmanifest-hashes komen overeen met die bestanden in de
private runmap. De binaries, firmware en oorspronkelijke PPM's blijven
buiten Git; hun hashes en de exacte PPM-header/RGB-reconstructie zijn bewaard.

## Zelf reproduceren

Kies nieuwe build-, target-, run- en archiefnamen. De capture weigert terecht
een bestaande bestemming. Gebruik de actuele checkout met passende Rust1.90-,
QEMU7.2- en OVMF-tools:

```sh
source /tmp/nieuw-os-par-tools/env.sh
export CARGO_HOME=/tmp/nieuw-os-par-tools/cargo-home
export NOS_BUILD_DIR="$PWD/build/g0-reproduce-1"
export NOS_TARGET_DIR=/tmp/nieuw-os-g0-reproduce-1-target
export CARGO_TARGET_DIR="$NOS_TARGET_DIR"
export CARGO_BUILD_JOBS=2
export NOS_SDK_RUNTIME=0
python3 -B tools/build.py --kernel-feature graphics-g0
python3 -B tools/run_graphics.py --qmp-transport pipe --machine pc-q35-7.2 \
  --run-root /tmp/nieuw-os-g0-reproduce-1 \
  --vga-rom /tmp/nieuw-os-par-tools/qemu/usr/share/seabios/vgabios-stdvga.bin \
  --vga-rom-sha256 cc2f735f19b6318922ac3de9506dee498f149a6b75534f7e5c176d4441a7fa4a
python3 -B - <<'PY'
import json, os, pathlib, subprocess, sys
result = pathlib.Path(os.environ['NOS_BUILD_DIR']) / 'graphics-g0-result.json'
run = json.loads(result.read_text())['run_directory']
subprocess.run([sys.executable, '-B', 'tools/capture_graphics.py',
               '--run-dir', run,
               '--destination', 'docs/verification/g0-reproduce-1'], check=True)
PY
```

De gearchiveerde `execute_gate.py` is het exacte gebruikte B08-stuurprogramma
vanuit de buildmap, inclusief de onafhankelijke PNG-controle. De oorspronkelijke
opdrachten, omgeving, tijdstippen en exitstatussen staan in `execution.json`.

## Grenzen en behouden bewijs

Dit bevestigt de bestaande native G0-scène en RAM-invoer op het vastgelegde
QEMU-profiel, plus de nieuwe herbruikbare capture op de gecombineerde bron.
Het bewijst geen nieuwe native documentbewerking, persistentie via G0,
desktop, fysieke hardware of UNIX-socketroute. Er is geen testresultaat
gerepareerd. De eerder bevestigde native run en de afzonderlijk ongeldig
verklaarde oorspronkelijke run blijven intact onder hun eigen bronbinding.

`i10-readback.json` is een afzonderlijke read-only controle van de
document-sessionreview: 37 bron-/bewijshashchecks, geen nieuwe testuitvoering
of VM. Zij hoort bij G07/I10 en maakt geen deel uit van de native G0-claim.

Roadmapvoorstel: **0 punten wijziging**. Deze taak bevestigt bestaande G0-
functionaliteit op de nieuwe bron en maakt de capture reproduceerbaar.
