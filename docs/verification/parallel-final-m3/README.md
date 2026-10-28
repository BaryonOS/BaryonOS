# I05 — volledige M3-regressie op de integratiebron

## Uitkomst en bronbinding

**Alle 22 groepen slagen: 19 werkelijke native VM-runs en drie afzonderlijke
host-/artifacttestgroepen.** Geen hosttest telt mee als een VM-run.

Broncommit: `3e41bbf93822f7044c6020259f4a752925e1966d`.
Source-fingerprint v3:
`307c828e9ce7bf49a4e809859ca77ecd332015572dd37e96fb580de98951aa60`.
HEAD en bronhash zijn voor en na de gehele suite gecontroleerd en gelijk.
Deze bron bevat de native quantumhooks, SDK en de gerepareerde scalargate.

| Native groep | Aantal | Wat daadwerkelijk is gecontroleerd |
|---|---:|---|
| Normale M3-run | 1 | Private adresruimtes, preëmptie, eerlijk plannen, opruimen |
| Toegangs- en uitvoeringsfouten | 12 | Peer-read/write, kernel-write, stack/heap-exec, beide guards, cli, x87, XMM, AVX en UD |
| Gecontroleerde exit/ongeldige stack | 3 | Exit, niet-nul-exit van proces B en invalid-rsp |
| Idle na beide exits | 1 | Werkelijke idle-timerinterrupts en resourcehergebruik |
| Grotere BSS-image | 1 | Meer dan de oude grens van 64 mappings, daarna dezelfde M3-acceptatie |
| M1 na alle M3-runs | 1 | Gewone boot zonder onbedoelde M3-/quantumfeature |

Alle 19 gasten eindigen met rawstatus 33, zonder time-out en met de voor hun
case vereiste kernelmarkers na `BOOT:EBS_OK`. De achttien M3-gasten tonen
positieve timer-/syscallaantallen en `M3:CLEANUP:PASS`. De drie hostgroepen
controleren kernel-core/ELF/returnflags, de gewone userspace-artifact en de
grotere BSS-artifact. Ruwe compiler- en testlogs zijn afzonderlijk bewaard.

## Extra controle van het werkelijke bewijs

Voor iedere VM zijn de werkelijke kernel-/loaderbytes opnieuw gehasht en
vergeleken met het bouwmanifest van precies de bijbehorende feature. Beide
firmwaresnapshots zijn opnieuw gehasht en gelijk aan het vastgelegde paar.
De volledige QEMU-commandoregel en profielstring zijn exact gecontroleerd,
evenals de bronhash, HEAD, echte seriële/stderrbytes en het aantal en de
volgorde van vereiste markers. Geen run bevat een QSIM-marker.

`acceptance.json` bevat deze extra koppeling per VM. Onder `runs/` staan de
ongewijzigde `result.json`, `serial.txt`, `stderr.txt` en het corresponderende
bouwmanifest. `m3-test-results.json` is de oorspronkelijke suite-uitkomst;
`i05-run-checks.json` bewaart commando, omgeving en broncontrole.
`build-and-host-logs.json` bewaart alle ruwe build-/hostlogteksten met hun
oorspronkelijke SHA256. Gegenereerde binaries staan niet in Git.

## Reproduceren

```bash
source /tmp/nieuw-os-par-tools/env.sh
export CARGO_HOME=/tmp/nieuw-os-par-tools/cargo-home
export CARGO_BUILD_JOBS=2
export NOS_QEMU_MACHINE=pc-q35-7.2
export NOS_SDK_RUNTIME=0
export NOS_BUILD_DIR="$PWD/build/m3-reproduce"
export NOS_TARGET_DIR=/tmp/nieuw-os-m3-reproduce-target
export CARGO_TARGET_DIR="$NOS_TARGET_DIR"
PYTHONPATH=tools python3 -B -c \
  'from test_milestones import test_m3; test_m3()'
```

De directe `test_m3()`-route voert precies de volledige M3-suite uit. De
CLI-optie `--through m3` zou daarnaast M2 uitvoeren en is hier niet gebruikt.
Alle VM's gebruiken expliciet `pc-q35-7.2/qemu64/TCG/1CPU/256MiB`, netwerk
uit, een eigen firmwarepaar en alleen-lezen ESP. De time-out is 60 seconden
per gast. De suite bouwt iedere feature en bewaart haar artifacts afzonderlijk.

## Betekenis en grenzen

Dit is regressiebewijs dat de bestaande M3-grenzen op de samengestelde bron
blijven werken. Het introduceert geen nieuwe syscall of bevoegdheid en is
geen bewijs voor fysiek hardwaregedrag, meerdere CPU's of realtimegaranties.
De quantumrun is afzonderlijk opgeslagen in `parallel-final-qsim`.
Scorevoorstel: bestaande procescriteria behouden; geen extra punten enkel
voor deze herhaalde acceptatie. De hoofdagent beheert het vaste scoreblad.
