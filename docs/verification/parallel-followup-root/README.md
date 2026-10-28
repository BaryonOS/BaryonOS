# Actuele M6b-opslag en hostgrenzen

Uitvoeringsbron: `dbd35b04d742b18d66b40fdd65d8a9b900656cac`.
Bronfingerprint v3: `c5f3d9316433ba09ba4b886985ffa8b28643d67c72a9ebb0e2b0136362dc7a9f`.

Twee echte, afzonderlijke QEMU-runs slagen met ruwe exit 33: normale CRUD op
een vers synthetisch volume, gevolgd door lezen/herstel in een nieuwe koude
VM met hetzelfde volume. De bestaande volledige per-run capturevalidator
controleert beide runs; 18 werkelijke artifacthashes zijn daarnaast opnieuw
gelezen. De koude before- en after-image zijn bytegelijk aan de eerdere
after-image. De volledige historische 32-groepenpoort is niet herhaald.

Gerichte huidige hostcontroles: 34 grafische testmethoden, 21 preflighttests,
3 buildpadtests (elk over vier runners) en 3 bronfingerprinttests. De echte
`tools/preflight.py --strict` slaagt. Alle opdrachten, uitvoer, foutuitvoer,
exitstatussen en broncontroles staan in `execution.json` en de bijbehorende
ongewijzigde tekstlogs. `verification.json` bevat de twee echte VM-paden,
zeven gezamenlijke herstelcontroles en de 18 hergehashte artifacts.

## Reproduceren

Gebruik Rust 1.90 met de bestaande gepinde launchers, beide native targets,
QEMU 7.2.22 en OVMF. Kies verse, afzonderlijke paden; de buildmap moet binnen
de checkout liggen. Stel `OS_OVMF_CODE` en `OS_OVMF_VARS` op de werkelijke
firmwarebestanden in en gebruik `NOS_QEMU_MACHINE=pc-q35-7.2`.

```python
# Na instellen van NOS_BUILD_DIR, NOS_TARGET_DIR en CARGO_TARGET_DIR;
# voeg de checkout/tools-map aan sys.path toe.
from build import build
from run_m6b import run_m6b
from capture_m6b import check_run
from common import source_hash
from pathlib import Path
frozen = source_hash()
build(kernel_feature="milestone-m6b", label="normal")
normal, volume = run_m6b("normal", label="normal")
check_run(Path(normal["run_directory"]), normal, frozen)
build(kernel_feature="test-m6b-recover", label="recover")
cold, _ = run_m6b("recover", volume=volume, label="cold")
check_run(Path(cold["run_directory"]), cold, frozen)
```

`execute.py` bewaart de exact uitgevoerde launcher en heeft bewust deze
werkruimte/freeze vastgelegd. Voor een andere checkout zijn de bovenstaande
bestaande build/run/capturefuncties de reproduceerroute. Geen echte schijf,
netwerk, algemene appstart, fysieke stroomuitval of grafische documentflow
is onderdeel van deze gerichte proef. Roadmap: 14% naar 14%.
