# VirtIO RNG generieke PCI-start

**Checkpoint, geen native acceptatie.** Branch `copilot/universal-driver-framework`,
broncommit `af00927da73dcdc26007802e612e7ecc2a1c5f40`, basis
`377446b0c9b8168e2c13c55fc09e373ccefa9d15`. De remote branch is gecontroleerd
op exact de broncommit.

## Resultaat

VirtIO RNG gebruikt segment-0-PCI-enumeratie, registrymatching, een gebonden
configadapter, kernel-owned `PciBarGrant`-mappings en een statisch gekozen
initializerfactory. De keyring reserveert de drie DMA-pagina's pas nadat een
RNG-device is gevonden. Bij afwezigheid blijven de broker en services offline
zonder devicepagina's te alloceren. De bestaande queue-, entropy-, timeout- en
reset/quarantainepaden zijn behouden. `kernel/src/m2/m3.rs` is niet gewijzigd.

## Commando's en uitkomsten

| Commando | Exit | Uitkomst / log |
|---|---:|---|
| `$HOME/.cargo/bin/cargo +1.90.0 fmt -p virtio-rng-driver -- --check` | 0 | Formattercheck geslaagd |
| `CARGO_TARGET_DIR=build/virtio-rng-generic-pci $HOME/.cargo/bin/cargo +1.90.0 check -p pci-core -p virtio-rng-driver --all-targets --offline` | 0 | Driver- en testcode typechecken; log `/tmp/nieuw-os-virtio-rng-check.log`, SHA-256 `6dc2bd10b9d398256a5846dbc011aba51a5fccede85cdd8ff62f4a18a04319b8` |
| `CARGO_TARGET_DIR=build/virtio-rng-generic-pci $HOME/.cargo/bin/cargo +1.90.0 check -p virtio-rng-driver --target x86_64-unknown-none --offline --locked` | 0 | Freestanding drivercheck geslaagd; log `/tmp/nieuw-os-virtio-rng-none-check.log`, SHA-256 `79db3f7d72af1108d1988b4038d203fd523fa2151a1dda35b8192661c4087a32` |
| `CARGO_TARGET_DIR=build/virtio-rng-generic-pci $HOME/.cargo/bin/cargo +1.90.0 clippy -p virtio-rng-driver --all-targets --offline --locked -- -D warnings` | 0 | Strikte Clippy geslaagd; log `/tmp/nieuw-os-virtio-rng-clippy.log`, SHA-256 `3550a9330a36addcff3434788dc28016dafce67027751b4199c414014ae5c8f1` |
| `PATH=/tmp/nieuw-os-hostcc-2026-11-03/bin:$HOME/.cargo/bin:$PATH CARGO_TARGET_DIR=build/virtio-rng-generic-pci $HOME/.cargo/bin/cargo +1.90.0 test -p pci-core -p virtio-rng-driver --offline` | 101 | Hosttests niet uitgevoerd: linker mist `/usr/lib/x86_64-linux-gnu/libc_nonshared.a`. De eerste poging vond daarnaast een typefout in `reset_acknowledged`; die is hersteld. Eindlog `/tmp/nieuw-os-virtio-rng-pci-tests-final.log`, SHA-256 `92e9bdbb762b5ed427bd1036e71592224af8945386bf615ed9a33936ff56e59f`; eerste foutlog `/tmp/nieuw-os-virtio-rng-pci-tests.log`, SHA-256 `8e3962207b0f7179351d0b2b615b9f0407b28cedcbc35df230a99bab3c3251ee` |
| `PATH=/tmp/nieuw-os-hostcc-2026-11-03/bin:$HOME/.cargo/bin:$PATH CARGO_TARGET_DIR=build/virtio-rng-generic-pci $HOME/.cargo/bin/cargo +1.90.0 check -p os-kernel --features native-keyring --target x86_64-unknown-none --offline --locked` | 101 | Stopte vóór kernelcompilatie bij linken van `os-kernel`'s host-buildscript; dezelfde `libc_nonshared.a` ontbreekt. Log `/tmp/nieuw-os-keyring-kernel-check.log`, SHA-256 `cf46b1fd3c5d59ff2b55b12dd94935a7f179c2178025b2f59681388c19871a71` |
| `git diff --check` | 0 | Geen whitespacefouten |

De eerste hosttestcompilefout is hersteld; de herhaling bevestigt uitsluitend de
omgevingblokkade bij het linken. De volledige
`python3 -B tools/build.py --kernel-feature native-keyring`-build moet nog
worden geprobeerd. `qemu-system-x86_64` ontbreekt, zodat VM-runtime en
foutscenario's niet zijn uitgevoerd. Fysieke hardware, DMA-isolatie en
RNG-kwaliteit zijn niet getest.

Geen nieuw voortgangspunt: code-, typecheck- en lintresultaten sluiten geen
native hardwarecriterium.
