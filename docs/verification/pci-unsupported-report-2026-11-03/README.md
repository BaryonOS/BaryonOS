# PCI-mismatchrapportage

**Status:** implementatie compileert en gerichte hostcontroles slagen op de
geharde bronboom. Geen QEMU-runtime-, fysieke hardware- of onafhankelijke
menselijke review.

- Gecontroleerde bronboom: `6eeae37cef88d61d71905f03291014793304f470`
- PCI-rapportage-implementatie: `340ecd12938f3915140315051aa2ca4bbcacd005`
- Constructiebeveiliging en cataloguscorrectie: `cace93cc2a24ada0d8088c0b782ee0e75443104b`
- Branch: `copilot/universal-driver-framework`
- Gepushte head na checkpoint: `6eeae37cef88d61d71905f03291014793304f470`
- PR #153: open, `CHANGES_REQUESTED`; geen CI-checks gerapporteerd.
- Voortgang: `41/100 → 41/100`; compilatie en hosttests voldoen niet aan een
  nieuwe native acceptatiepoort.

## Controles

De hosttests en Clippy gebruikten de reeds aanwezige tijdelijke compiler onder
`/tmp/nieuw-os-hostcc-2026-11-03`; er is geen systeem- of repositorydependency
geïnstalleerd. De omgeving voor beide Cargo-commando's was:

```sh
export PATH="/home/simon/.cargo/bin:/tmp/nieuw-os-hostcc-2026-11-03/bin:$PATH"
export LD_LIBRARY_PATH=/tmp/nieuw-os-hostcc-2026-11-03/root/usr/lib/x86_64-linux-gnu
export GCC_EXEC_PREFIX=/tmp/nieuw-os-hostcc-2026-11-03/root/usr/lib/gcc/x86_64-linux-gnu/15/
export COMPILER_PATH=/tmp/nieuw-os-hostcc-2026-11-03/root/usr/libexec/gcc/x86_64-linux-gnu/15
export LIBRARY_PATH=/tmp/nieuw-os-hostcc-2026-11-03/linker-script:/tmp/nieuw-os-hostcc-2026-11-03/root/usr/lib/x86_64-linux-gnu:/tmp/nieuw-os-hostcc-2026-11-03/root/usr/lib/gcc/x86_64-linux-gnu/15
export CC=/tmp/nieuw-os-hostcc-2026-11-03/bin/cc
```

| Controle | Exact commando | Exit | Duur | Log | SHA-256 |
|---|---|---:|---:|---|---|
| Hosttests | `cargo test --locked -p driver-core -p pci-core -p virtio-blk-driver -p virtio-net-driver -p virtio-rng-driver` | 0 | 641 ms | `/tmp/nieuw-os-pci-report-2026-11-03/recheck-cace93/host-test.log` | `23a9d7b048ee9eb377f6548e87d11b52fd4e05bb9de477f1c94cc7e4fb3d1066` |
| Clippy | `cargo clippy --locked -p driver-core -p pci-core -p virtio-blk-driver -p virtio-net-driver -p virtio-rng-driver --all-targets -- -D warnings` | 0 | 275 ms | `/tmp/nieuw-os-pci-report-2026-11-03/recheck-cace93/clippy.log` | `37235b84edcac5d91879142e6766abdb467f273f613023d367fca4e62b73d096` |
| Format | `cargo fmt --all -- --check` | 0 | <1 s | `/tmp/nieuw-os-pci-report-2026-11-03/recheck-cace93/fmt.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| Whitespace | `git diff --check` | 0 | <1 s | `/tmp/nieuw-os-pci-report-2026-11-03/recheck-cace93/diff-check.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| M5-kernelbuild | `python3 -B tools/build.py --kernel-feature milestone-m5` | 0 | 1.635 s | `/tmp/nieuw-os-pci-report-2026-11-03/recheck-cace93/m5.log` | `1d18e7aa8aadc5bbebab1c7c627ad5ce097051cebc0b12b25e4bbb4f5f667d72` |
| Systeem/netwerk-kernelbuild | `python3 -B tools/build.py --kernel-feature native-system-net` | 0 | 2.129 s | `/tmp/nieuw-os-pci-report-2026-11-03/recheck-cace93/native-system-net.log` | `36e3c8bbfbb7260b0a0f3e2521cf54486fb95225342162d714c5a3d5f5163aa0` |

De hosttestuitvoer bevat 40 geslaagde tests: 11 `driver-core`, 8 `pci-core`,
1 blockdriver, 16 netwerkdriver en 4 RNG-driver. De buildscriptcommando's
gebruikten `PATH="/home/simon/.cargo/bin:$PATH"`. De hosttest- en
Clippy-commando's gebruikten bovenstaande tijdelijke compileromgeving.

## Niet geslaagd of niet uitgevoerd

- Eerdere pogingen zonder tijdelijke compiler of Cargo-bin-directory op
  `PATH` faalden (`linker cc not found` respectievelijk `rustc` niet gevonden);
  de herhaalde controles hierboven met aanwezige tools slaagden.
- `qemu-system-x86_64` ontbreekt. De seriële `UNSUPPORTED:PCI`-uitvoer is
  daarom niet op een VM gecontroleerd; fysieke hardware is niet getest.

De implementatie is een read-only diagnose vóór M5-deviceactivatie. De scan
matcht metadata voor de driverfamilies in het actieve profiel, meldt alleen
type-0-endpoints als unsupported en telt niet-type-0-functies apart. Een
onvolledige scan wordt expliciet gemeld. Dit bewijst geen driveractivatie,
resource-isolatie of volledigheid van alle toekomstige registries.
