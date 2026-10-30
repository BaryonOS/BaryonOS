# R77 reviewfix: nieuwe same-source native keten

Vaste codebron `7d66dc2744604d8119b8366a283dc10ea1a76000`;
v3 `949ce3abc8df4af3ed4cc29fd41e10ee0621e4f270ddf4ac2aafb3bacba9a901`.
Dit is eigen reviewfixacceptatie, geen acceptatie op een nieuwere main.
Geen base-/kernel-/preview-/SDK-/lock-/workflowwijziging. Bestaande geldige
eigen targetcache hergebruikt; nieuwe buildmanifests en schone runimages.
Historisch code420fac5-bewijs is ongewijzigd behouden in de bovenliggende
map, niet naar deze bron gelabeld.

## Exacte uitgevoerde commandoreeks

Alle onderstaande commando's exit0, behalve de eerder apart beschreven
rode reproductie in [review-fixes.md](../review-fixes.md).

```sh
export LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu"
export NOS_TARGET_DIR="$PWD/target/native-cold-acceptance/gate1"
export OS_OVMF_CODE="$PWD/.tools/nieuw-os/qemu/usr/share/OVMF/OVMF_CODE_4M.fd"
export OS_OVMF_VARS="$PWD/.tools/nieuw-os/qemu/usr/share/OVMF/OVMF_VARS_4M.fd"
NOS_BUILD_DIR="$PWD/build/native-cold-acceptance/review-reference" python3 -B -u tools/build.py --kernel-feature native-system-materiaal-probe
python3 -B -u tools/run_materiaal_probe.py --build-dir "$PWD/build/native-cold-acceptance/review-reference" --machine pc-q35-10.0 --vga-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/vgabios-stdvga.bin" --vga-rom-sha256 c944f5fd404a6553a32e1e0527081d40e6040d3ce1740d39766bff01871e4fde
NOS_BUILD_DIR="$PWD/build/native-cold-acceptance/review-parent" python3 -B -u tools/build.py --kernel-feature native-system-materiaal
python3 -B -u tools/run_materiaal_preview.py --build-dir "$PWD/build/native-cold-acceptance/review-parent" --interactive-reference-run "$PWD/build/native-cold-acceptance/review-reference/run/materiaal-normal-3cd55ccf2e" --timeout 240 --machine pc-q35-10.0 --bios-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/bios-256k.bin" --vga-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/vgabios-stdvga.bin"
python3 -B -u tools/run_materiaal_cold.py --build-dir "$PWD/build/native-cold-acceptance/review-parent" --parent-run "$PWD/build/native-cold-acceptance/review-parent/preview-run/materiaal-88844885f4" --reference-run "$PWD/build/native-cold-acceptance/review-reference/run/materiaal-normal-3cd55ccf2e" --timeout 240 --machine pc-q35-10.0 --bios-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/bios-256k.bin" --vga-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/vgabios-stdvga.bin"
python3 -B docs/verification/native-cold-acceptance/tamper_replay.py --build-dir "$PWD/build/native-cold-acceptance/review-parent"
python3 -B -m unittest tools.tests.test_materiaal_cold
python3 -B -m unittest discover -s tools/tests
python3 -B tools/progress.py --check
git diff --check
```

| Stap | Rapportduur | Commandowall | Exit |
|---|---:|---:|---:|
| Referentiebouw | N/A | 2.10s | 0 |
| Referentie-VM 3cd55ccf2e | 133.939s | 134.09s | 0, gast raw33 |
| Gewone parentbouw | N/A | 2.00s | 0 |
| Gewone parent 88844885f4 | 93.551s | 93.72s | 0, gast raw0 |
| Cold-child 880aa274b6 | 37.543s | 37.68s | 0, gast raw0 |
| Offline replay, bestaande negatieven | N/A | 1.13s | 0 |
| Uitgebreide echte firmware/serialnegatieven | N/A | 1.21s | 0 |

Logs `/tmp/nieuw-os-cold-review-{reference,parent}-{build,run}.log`,
`/tmp/nieuw-os-cold-review-child-run.log` en
`/tmp/nieuw-os-cold-review-tamper-{baseline,expanded}.{json,log}`.
Dezelfde prefixes met `.time` bewaren gemeten wall/user/sys.
Grote originele QMP-/PPM-/diskbestanden blijven in de genoemde runmappen
buiten Git. `reports.json` bindt compacte gegenereerde summaries aan de
volledige originele rapporthashes; ruwe serial/stderr zijn bytegelijk bewaard.

## Feitelijke acceptatie

Gewone parent: zes volledige 800x600-referentiebeelden, zeven app/ELF-records,
bron/kernel/loader/machine/QEMU/firmware en onafhankelijk diskorakel groen.
Cold-child: volledige lijst/heropenbeelden zonder maskers, definitieve
seriële validatie na procesexit, immutable parentfirmware opnieuw gehasht
en exact onveranderde diskbytes. QMP-trace bevat 398 gecorreleerde opdrachten;
alle input/response-IDs en fasevolgorde zijn offline opnieuw gecontroleerd.

Kernel `f2e9cc4aad87ad2b9fbc8b498ab146db2ab0df4a77b904eecf81c3a9beb198b3`;
loader `f90bc8f0452e13bafcc8ed1a2afc24d8e0f20573e5a381dfb0ca90fdc0ca1025`.
Cold-before/after beide
`246c8f9773533796718a729389c26d6103f45f04e7a83b4d06503e17c6d4e07e`.
Zelfstandige decoder: object0/rev1, `Nieuw document`, `nieuw os werkt`
(14 bytes), generatie1/klokken[1,0], andere domeinen leeg, canaries intact.
Volledig koude lijst-PPM
`ebc10cc5ac84ff5ac80796e42de4261a26fcb5faaa021b8a11e8a14460555eaa`;
heropen-PPM
`dfca8c62b1e10b99a6561098b56be8749102d69dca390257717935686fb19099`.
Originele coldreport-SHA256
`db052373c440823fa9954298f81f076eb9a323025deddc3864e11e4e135dbb67`.
De summaries bevatten tevens alle zeven apps en de QEMU-/firmwarehashes.

Na de nieuwe geslaagde keten is de kleinste resterende negatieve stap
uitgevoerd op tijdelijke kopieën van deze echte parent: alle vier immutable
firmwarebestanden afzonderlijk gemuteerd, plus late PANIC/FAIL toegevoegd
met eveneens bijgewerkte serialhash. Alle zes geweigerd op hun inhoudelijke
validatie, niet op alleen een stale hash. Met de bestaande acht artifact-
en twee claimnegatieven zijn alle zestien geweigerd. De positieve kopie
is vóór en na de mutaties opnieuw geaccepteerd; alle 66 originele
artifact-/rapportbestanden zijn ongewijzigd. Resultaten `tamper-results.json`.

Na archivering opnieuw 16 gerichte tests/3.528s en 154 tools-tests/9.099s,
exit0; progresscheck/diffcheck eveneens exit0. Logs
`/tmp/nieuw-os-cold-review-final-{targeted,tools}.log` en
`/tmp/nieuw-os-cold-review-progress.log`. Alle drie nieuwe rapporten
afzonderlijk op codecommit7d66/v3949ce3/rapport-SHA256 gecontroleerd.

## Open grenzen

Host-QMP-stop na opslaan is geen guest clean shutdown en kan QEMU-caches
flushen; geen abrupt stroomverlies tijdens een write geaccepteerd.
Geen productie-encryptie, Windowscold, menselijke audit, CORE-bitreproduceer-
baarheid, nieuwere-mainacceptatie of volledige FR-claim.
Scorevoorstel **34% -> 34%**; centrale status/assessment bij de teamleider.
