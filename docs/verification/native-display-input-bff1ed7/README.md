# Native display/input-proef — bff1ed7

**Reikwijdte:** gehashte eigen Nieuw OS UEFI-loader/kernel en drie CPL3-
proefprogramma's; één QEMU-VM met twee input-/displaygeneraties. Dit is
emulatorbewijs voor de bestaande PS/2/display/input-slice, geen fysieke
hardwareacceptatie, geen USB/xHCI- of audioproef en geen energie-/prestatiemeting.

## Profiel en artefactbinding

- Broncommit: `bff1ed76a8ace8617dee040770bcea98bb39fe55`.
- Profiel: `pc-q35-8.2`, `qemu64`, TCG, één CPU, 256 MiB, VGA en PS/2; netwerk
  uitgeschakeld. Displayprofiel: 800×600.
- QEMU: `11.1.0 (v11.1.0-12130-ge470268ff4)`. De SHA-256 van de werkelijk
  meegeleverde `C:\Program Files\qemu\qemu-system-x86_64.exe` is
  `47d57a6072e0bb3bd98f87926eb129eb1736dfe818c67b3b81ef7ce4edd0b3cd`;
  deze komt overeen met de runner-ingang. WSL geeft voor
  `/proc/<pid>/exe` de proxy `/init`; de aparte
  `qemu_implementation_path`-/`qemu_implementation_sha256`-velden in
  `native-result.json` gelden daarom niet als QEMU-binary-identiteit.
- Kernel SHA-256:
  `1b20fd1e487f052cb4e7d2144596c334a2ec923e02c3a5a7eb95daed61ffb1c5`.
- UEFI-loader SHA-256:
  `1c82e84d21bbeead563c16fc123a310ed87e41eab8069a3b33e1d33e19f4be28`.
- Run: raw QEMU-exit 33, geen timeout; rapportcheck en artifactbinding slagen.
  Twee inputgeneraties leverden toetsenbord-/muisbytes. De framebufferpoort
  vergeleek 768 pixelposities, zag 512 unieke verwachte pixels, valideerde
  behoud van het eerste frame en weigerde ongeautoriseerde zichtbare pixels.
  De guest controleerde capability-/displayweigeringen, reset, lifecycle en
  process-slot-hergebruik.

## Uitgevoerde commando's

De hosttests en preflight draaiden vanuit de projectcheckout; de native
build-/QEMU-commando's draaiden vanuit een tijdelijke detached worktree op de
Windows-schijf. De projectlokale Rust 1.90.0-toolchain en niet-globale
host-C-linker stonden vooraan op `PATH`. `NOS_BUILD_DIR`, `NOS_TARGET_DIR` en
`CARGO_TARGET_DIR` voor de native run wezen naar unieke build-/targetmappen
onder die worktree; het QEMU Vars-bestand was een wegwerpkopie. Voor Cargo en
preflight stonden
`.tools/nieuw-os/host-toolchain/bin` en `.tools/nieuw-os/cargo/bin` vooraan op
`PATH`; niets is systeembreed geïnstalleerd. Exitstatus:

| Commando | Exit |
|---|---:|
| `python3 -B tools/test_display_probe.py` | 0 — 13 tests |
| `cargo +1.90.0 test --offline --locked --manifest-path drivers/ps2/Cargo.toml` | 0 — 1 unit-, 7 controller-tests |
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/display-input-probe/Cargo.toml` | 0 — 2 fixture-tests |
| `python3 -B tools/preflight.py --strict` | 0 |
| `python3 -B -u tools/build.py --kernel-feature native-display-probe` | 0 |
| `python3 -B -u tools/run_display_probe.py --machine pc-q35-8.2 --qmp-transport pipe --vga-rom '/mnt/c/Program Files/qemu/share/vgabios-stdvga.bin'` | 0 — native gate passed |

## Bestanden

- `native-result.json`: volledige runnerstatus, ruwe seriële transcriptie,
  checks, observaties en hashbinding.
- `build-manifest.json`, `build-report.json`: buildinputs en ELF-/artifactclaims.
- `qmp-trace.json`: vastgelegde, geslaagde toetsenbord-, muis- en screendump-
  opdrachten en reacties.
- `build.log`, `native-run.log`: relevante uitvoer.
- `SHA256SUMS`: hashes van dit archief.

De PPM-screenshots, VM-outputs, wegwerp-OVMF-vars en gegenereerde binaries zijn
niet in Git opgenomen. Er zijn geen fysieke disks, apparaten of firmware
gewijzigd. H02–H06 blijven open; deze slice voegt geen roadmappunten toe.
