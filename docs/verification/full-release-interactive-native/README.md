# Interactieve native zevenprocessenketen

Geteste bron: `fc40347b250c544ab9838cf89a62df16b32a797b`, v3
`61f58e2e7c7a64e911883c64de5b2d416f7085af94123699fd206c7448ea2b5e`.
Deze bewijsbron wordt niet naar een later formatteringscheckpoint omgelabeld.
De summaries zijn gegenereerde selecties van originele rapporten, met hun
SHA256 en originele lokale pad. De volledige QMP-trace, PPM's en disks staan
in de genoemde gegenereerde runmappen, niet in Git. Firmwarelogs zijn exact
gekopieerd; binaries en caches zijn niet opgenomen.

Gemeenschappelijke omgeving:

```sh
export LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu"
export NOS_TARGET_DIR="$PWD/target/native-test-version/gui-baseline"
```

Uitgevoerde commando's, elk exit0:

```sh
NOS_BUILD_DIR="$PWD/build/native-test-version/interactive-reference" python3 -B -u tools/build.py --kernel-feature native-system-materiaal-probe
python3 -B -u tools/run_materiaal_probe.py --build-dir "$PWD/build/native-test-version/interactive-reference" --machine pc-q35-10.0 --vga-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/vgabios-stdvga.bin" --vga-rom-sha256 c944f5fd404a6553a32e1e0527081d40e6040d3ce1740d39766bff01871e4fde
NOS_BUILD_DIR="$PWD/build/native-test-version/interactive-current" python3 -B -u tools/build.py --kernel-feature native-system-materiaal
OS_OVMF_CODE="$PWD/.tools/nieuw-os/qemu/usr/share/OVMF/OVMF_CODE_4M.fd" OS_OVMF_VARS="$PWD/.tools/nieuw-os/qemu/usr/share/OVMF/OVMF_VARS_4M.fd" python3 -B -u tools/run_materiaal_preview.py --build-dir "$PWD/build/native-test-version/interactive-current" --interactive-reference-run "$PWD/build/native-test-version/interactive-reference/run/materiaal-normal-224bfa2ade" --timeout 240 --machine pc-q35-10.0 --bios-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/bios-256k.bin" --vga-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/vgabios-stdvga.bin"
```

Referentie: 136.326s, raw33; gewone variant: 92.774s, QMP-quit/raw0.
Alle zes volledige 800x600-PPM's exact gelijk, geen pixelmaskers of
fallback. Alle zeven apprecords, loader, bron, machine, firmware en
QEMU-implementatie zijn gebonden. Na QEMU-exit zelfstandig diskorakel:
object0/rev1, `Nieuw document`, `nieuw os werkt`, generation1/clocks[1,0],
andere domeinen leeg en canaries intact.

Eerste previewaanroep zonder de twee expliciete OVMF-variabelen: exit1,
`Invalid native preview input: OVMF_CODE.fd`, voor VM-start. Automatisch
gevonden firmwarepad was geen regulier single-link bestand. De hierboven
gebruikte reguliere bestanden hebben exact de referentiehashes; geen
validatie versoepeld. Foutlog:
`/tmp/nieuw-os-interactive-current-run.log`.

Succeslogs: `/tmp/nieuw-os-interactive-reference-{build,run}.log`,
`/tmp/nieuw-os-interactive-current-build.log` en
`/tmp/nieuw-os-interactive-current-run-explicit.log`.
Hostvalidatie op dezelfde bron: 49 gerichte en 132 tools-tests exit0,
logs `/tmp/nieuw-os-interactive-integrated-{host,tools}.log`.

Actuele CI op fc40347 faalt op rustfmt van drie overgenomen cfg-hooks:
host-quality41091759159, display41091759171, foundation41091759214,
RTC41091759087. Dit is een integratiefout, geen native acceptatiefout;
reparatie en nieuwe same-source gates volgen. Andere lopende checks zijn
geen succesclaim. Geen Windows-run, ZIP, koudherstel van de gewone variant,
cleanup, menselijke visuele acceptatie, volledige release of extra
rubricpunten. **34% blijft34%; minstens50% is niet bereikt.**
