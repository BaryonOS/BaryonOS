# #117 — SUR1 sparse frames: native Materiaal-probe naast main

31 oktober 2026, integratie 3 (branch `ccr-4bb5b18c-t9u3cv`). Uitgevoerd door
de integrerende Claude Code-sessie; geen menselijke beoordeling.

## Wat is vergeleken

Twee schone worktrees, sequentieel gedraaid in dezelfde omgeving:

| Run | Bron | Bronfingerprint v3 | Resultaat | Raw exit | Duur |
|---|---|---|---|---|---|
| #117 | `3f88a7f4` (PR-head) | `5d04e68b62a0db2a…` | `passed=True`, alle zes fasen | 33 | 178,8 s |
| main | `e08fb0e4` | `9add25cdb2dfc4cb…` | `passed=False` | 35 | 205,5 s |

Omgeving: cloudcontainer, 4 vCPU, QEMU 8.2.2 (Ubuntu-pakket
`1:8.2.2+ds-0ubuntu1.18`), `pc-q35-8.2`, TCG, 1 CPU, 256 MiB, OVMF
`OVMF_CODE_4M.fd`/`OVMF_VARS_4M.fd` uit het Ubuntu-pakket, VGA-ROM
`/usr/share/seabios/vgabios-stdvga.bin` (SHA-256 `4eecd8e7…8bf8a0`).
Volledige paden, QEMU- en firmwarehashes staan in de `*-result.json`.

Commando's per worktree (elk vanuit de worktree-root):

```sh
export OS_OVMF_CODE=/usr/share/OVMF/OVMF_CODE_4M.fd OS_OVMF_VARS=/usr/share/OVMF/OVMF_VARS_4M.fd
NOS_BUILD_DIR="$PWD/build/probe-<n>" NOS_TARGET_DIR="$PWD/target/probe-<n>" \
  python3 -B -u tools/build.py --kernel-feature native-system-materiaal-probe
python3 -B -u tools/run_materiaal_probe.py --build-dir "$PWD/build/probe-<n>" \
  --machine pc-q35-8.2 --vga-rom /usr/share/seabios/vgabios-stdvga.bin \
  --vga-rom-sha256 4eecd8e752efc02c6c0fe87eb92bf66934cb9ea67b0e1bc41e86f8430c8bf8a0 --timeout 240
```

Build: beide exit 0 (22 s). Probe: #117 exit 0, main exit 1.

## Gemeten tellers (uit de gastsnapshots)

| Fase | #117 FRAMES / TILES | main FRAMES / TILES |
|---|---|---|
| ready | 1 / 1608 | 1 / 1608 |
| editor | 2 / 2887 | 2 / 3221 |
| dirty (28 toetsen getypt) | 3 / 2896 | 3 / 4840 |
| saved | 4 / 2913 | — (niet bereikt) |
| list | 5 / 4039 | — |
| reopened | 6 / 5179 | — |

Het frame na het typen kostte met #117 9 tegels; op main 1619 tegels
(4840 − 3221), elk met een eigen IPC-roundtrip. Main kwam na de
dirty-fase niet verder: de gast meldde acht keer `MATERIAAL:TIMEOUT_SLOT`
en eindigde met raw 35. De host-runner herkent dat record niet en meldt
`Unknown native record`; het is een gastzijdige time-out, geen hostcrash.

## Bewijsgrens

- Dit bewijst op deze omgeving dat de sparse-route native werkt (exacte
  pixelcontrole van de probe, alle zes fasen, raw 33) en dat main hier op
  traagheid vastloopt. Op Simons eerdere QEMU 10-omgeving slaagde main wel
  (136 s, zie `full-release-interactive-native/`); de main-mislukking is dus
  omgevingsafhankelijk en geen regressie van main.
- Eén run per bron: geen herhaalbaarheid, geen p50/p95, geen meting van de
  zichtbare latentie per toets. De ~5 s per letter uit de PR-tekst is niet
  opnieuw gemeten.
- Geen CI-workflow draait deze probe; dit is lokaal bewijs, geen
  CI-gate. Geen FR-, fysieke of menselijke acceptatie.

`SHA256SUMS` bindt de vier gekopieerde bestanden (serial en result per run).
QMP-traces, PPM's en disks zijn niet in Git opgenomen.
