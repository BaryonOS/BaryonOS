# Vaste interactieve bron na formatteringsherstel

Bron `eabc27ba879fd3b5960863ae8eede800c5045a57`, runtime v3
`a1998a126a3a1ccc18523925a422912d934c14bab865b3d7df346862a2c251a6`.
De summaries/seriële bytes volgen dezelfde archiefmethode als
`../full-release-interactive-native/`. Originele rapporthashes, QMP-tracehashes,
framehashes, apprecords en platformbinding staan in de summaries.
Volledige gegenereerde artifacts blijven lokaal in de genoemde runmappen.

Uitgevoerde commando's, elk exit0:

```sh
export LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu"
export NOS_TARGET_DIR="$PWD/target/native-test-version/gui-baseline"
export OS_OVMF_CODE="$PWD/.tools/nieuw-os/qemu/usr/share/OVMF/OVMF_CODE_4M.fd"
export OS_OVMF_VARS="$PWD/.tools/nieuw-os/qemu/usr/share/OVMF/OVMF_VARS_4M.fd"
NOS_BUILD_DIR="$PWD/build/native-test-version/interactive-reference" python3 -B -u tools/build.py --kernel-feature native-system-materiaal-probe
python3 -B -u tools/run_materiaal_probe.py --build-dir "$PWD/build/native-test-version/interactive-reference" --machine pc-q35-10.0 --vga-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/vgabios-stdvga.bin" --vga-rom-sha256 c944f5fd404a6553a32e1e0527081d40e6040d3ce1740d39766bff01871e4fde
NOS_BUILD_DIR="$PWD/build/native-test-version/interactive-current" python3 -B -u tools/build.py --kernel-feature native-system-materiaal
python3 -B -u tools/run_materiaal_preview.py --build-dir "$PWD/build/native-test-version/interactive-current" --interactive-reference-run "$PWD/build/native-test-version/interactive-reference/run/materiaal-normal-73611f4dbf" --timeout 240 --machine pc-q35-10.0 --bios-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/bios-256k.bin" --vga-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/vgabios-stdvga.bin"
```

Referentie: 133.320s/raw33; gewone zevenprocessenvariant:
92.390s/QMP-quit/raw0. Alle zes volledige native frames bytegelijk,
geen maskers/fallback. Zelfstandige diskcontrole na exit:
object0/rev1, naam `Nieuw document`, tekst `nieuw os werkt`, generation1,
clocks[1,0], andere domeinen leeg en canaries intact.
Gewone kernelhash:
`f2e9cc4aad87ad2b9fbc8b498ab146db2ab0df4a77b904eecf81c3a9beb198b3`.
Loaderhash:
`7e55ab0fe6c232380f52dc5be1334b41388b70d80e0ebe2b0072de7e12373a09`.

Logs `/tmp/nieuw-os-interactive-fixed-{reference,current}-{build,run}.log`.
Een aanvankelijk host-shellcommando zocht na de geslaagde referentie/build
ten onrechte `interactive-reference/materiaal-result.json` (bestaat niet):
exit1, voor interactieve VM-start. Daarna is de werkelijk door de runner
gerapporteerde referentiemap gebruikt; geen gastorakel gewijzigd.

De concrete CI-formatfout is hersteld: op eabc27b zijn host-quality,
native-display-input, native-foundation en native-rtc groen. CI was bij
controle nog niet volledig klaar: onder meer qemu-smoke, native-system en
m6e-native-volume pending. Actuele required-set toont host-quality groen
en qemu-smoke pending; oudere observatie "geen required checks" is dus niet
meer actueel. Geen blocking watch gebruikt.

Dit is een vaste bron voor een afzonderlijke Windows-QEMU-proef.
Windows gebruikt zijn eigen QEMU-versie/machine/RNG- en displaybackend:
Linuxbewijs is daarvoor geen vervangende acceptatie. Nog geen ZIP of
Windows-gastbediening uitgevoerd in deze sessie. Geen koudrecovery-,
cleanup-, menselijke visuele, volledige product- of hardwareacceptatie.
**34% blijft34%; 50% is niet bereikt.**
