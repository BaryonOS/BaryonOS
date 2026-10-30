# Baseline voor de volledige testrelease

Werkelijk gecontroleerde bron: `73ebc1e169b893eb5aba4b4f8ac06d7798136de3`,
bronvingerafdruk v3 `cccf0e3e7fcec439428208c4458a0194ec23c6120e1f03d96abe65b0711631b7`.
Dit is bestaand proces-/kernelbewijs, geen acceptatie van de volledige release.
De nieuwe opdracht verandert de vaste score niet: **32% → 32%**.

`git-build-audit.json` bewaart de read-only repositorycontrole en hostchecks.
238 Rust-hosttests en 21 preflighttests zijn uitgevoerd met exit 0; de volledige
Rust-stdout was in het tooltranscript zichtbaar en is niet apart gearchiveerd.
Dit wordt niet als volledige reproduceerbare native bewijslast voorgesteld.
Remote main en de checkout waren gelijk; geen open PR's. De drie genoemde
main-workflows zijn op hun exacte SHA via de API gecontroleerd.

`supervisor-gate.json` en de vijf submappen bevatten een nieuwe volledige K05-
uitvoering: normal, crash, denied, parent-exit, quota, ieder werkelijk QEMU
raw exit 33 en `passed=true`. De gate eindigt `completed/passed=true` en
bevat ook 40 capability-, 17 SDK-, één voorbeeld- en zeven harnastests.
Firmware-/image-/programmafiles blijven als gegenereerde lokale artifacts buiten
Git; de resultaatbestanden leggen de bron-/ELF-/ESP-hashes en QEMU-opdracht vast.

Exacte geslaagde hoofdopdracht vanuit de repositoryroot:

```sh
PATH=/workspace/.nieuw-os-env/bin:$PATH \
OS_OVMF_CODE=/workspace/.nieuw-os-env/sysroot/usr/share/OVMF/OVMF_CODE_4M.fd \
OS_OVMF_VARS=/workspace/.nieuw-os-env/sysroot/usr/share/OVMF/OVMF_VARS_4M.fd \
NOS_QEMU_MACHINE=pc-q35-8.2 \
NOS_BUILD_DIR=/workspace/nieuw-os/build/plan-baseline-k05 \
NOS_TARGET_DIR=/workspace/nieuw-os/target/plan-baseline \
CARGO_TARGET_DIR=/workspace/nieuw-os/target/plan-baseline \
python3 -B tools/dev.py python3 -B -u tools/test_supervisor.py
```

Exit 0, ruwe uitvoer `supervisor-command.txt`. `initial-environment-error.txt`
behoudt de eerste mislukte start zonder `tools/dev.py`: cargo stond niet op het
subprocesspad. Deze start heeft geen VM gedraaid en is geen kernelregressie.
De herhaling gebruikt de bestaande toolchain; er is niets hergeïnstalleerd.

De gate bouwt aan het einde de gewone M1-probe. Dat bestand is geen geleverd
releaseimage. Native grafiek, beveiligde persistente opslag, netwerken, apps,
NAS/providers, lokale AI, updates en integrale herstelacceptatie blijven open.
VM-uitvoering vervangt geen fysieke test of menselijke beveiligingsaudit.
