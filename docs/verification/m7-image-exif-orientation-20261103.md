# M7 JPEG EXIF-orientatie — host- en targetbewijs

Datum: 3 november 2026. Implementatiebron: `e5cded7e2426fa47999cc3e279cad9bb358b5fc3`
op branch `simonvanieperen-vigilant-robot`. Dit is een hosttestbare
codecfunctionaliteit, geen afbeeldingsviewer of aftekening van FR-A03.

## Resultaat en grenzen

De JPEG-wrapper leest de EXIF-orientatietag uit begrensde APP1/TIFF-IFD0-data.
Waarden 1–8 wijzigen alleen de RGBA8-displayuitvoer van `rgba8_into` en de
bijbehorende displayafmetingen. Originele objectbytes, ruwe decoderpixels,
bronafmetingen en `pixel_hash` blijven ongewijzigd. Onbruikbare TIFF-offsets
en ongeldige waarden worden genegeerd; EXIF-payload groter dan
`Limits.metadata_bytes` geeft `MetadataLimit`.

Regressies controleren alle acht mappings met een asymmetrische 3×2-pixelmatrix,
beide TIFF-bytevolgordes, de bestaande baselinefixture met oriëntatie 6
(257×129 bron, 129×257 display), ongeldige waarden/offsets en metadata-overlimiet.
De verwachte oriëntatie is getoetst aan CIPA DC-008-2012,
*Exchangeable image file format for digital still cameras: Exif Version 2.3*:
<https://www.cipa.jp/std/documents/e/DC-008-2012_E.pdf>.

ICC/gamma-kleurbeheer en PNG eXIf blijven buiten scope. Er is geen
native/QEMU-, viewer-, fysieke-, menselijke of releaseproef uitgevoerd.
FR-A03 blijft open; deze slice levert geen nieuw rubriccriterium op
(41/100 → 41/100).

## Uitgevoerde controles

De standaardhost-preflight meldde exit 2 omdat `cc` niet op `PATH` staat;
Cargo is daarom via zijn absolute toolchainpad aangeroepen. De hosttests zijn
succesvol gelinkt met de reeds aanwezige, geïsoleerde GCC 15.2/sysroot onder
`/tmp/nieuwos-terminal-toolchain`; er is niets systeembreed geïnstalleerd.
Rust 1.90.0, offline en de bestaande lockfile zijn gebruikt.
Omgevingsinstellingen voor de hostlink:

```sh
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/tmp/nieuwos-terminal-toolchain/root/usr/bin/x86_64-linux-gnu-gcc-15
export COMPILER_PATH=/tmp/nieuwos-terminal-toolchain/root/usr/libexec/gcc/x86_64-linux-gnu/15
export LIBRARY_PATH=/tmp/nieuwos-terminal-toolchain/root/usr/lib/x86_64-linux-gnu:/tmp/nieuwos-terminal-toolchain/root/usr/lib/gcc/x86_64-linux-gnu/15
export LD_LIBRARY_PATH=/tmp/nieuwos-terminal-toolchain/root/usr/lib/x86_64-linux-gnu
export RUSTFLAGS='-C link-arg=--sysroot=/tmp/nieuwos-terminal-toolchain/root'
export CARGO_TARGET_DIR=/tmp/nieuw-os-image-core-target
export LOCALAPPDATA=/workspace/nieuw-os/.tools
```

| Exact commando | Resultaat | Log en SHA-256 |
|---|---|---|
| `python3 -B tools/dev.py /home/simon/.cargo/bin/cargo +1.90.0 test --locked --offline --manifest-path services/image-codec/Cargo.toml -- --nocapture` | exit 0; 15 decode-, 5 snapshot/OOM- en 3 upsampletests geslaagd | `/tmp/nieuwos-image-exif/test-final.log` — `01769c0fcb9311b7c965a4a93805f1bf13611d4f6d7a6971f4de283f284bd8c1` |
| `python3 -B tools/dev.py /home/simon/.cargo/bin/cargo +1.90.0 clippy --locked --offline --manifest-path services/image-codec/Cargo.toml --all-targets -- -D warnings` | exit 0 | `/tmp/nieuwos-image-exif/clippy-final.log` — `c9c1b9a1de0d07f5d05a96105b5a5709bc52bb446e56d5942ca078a3d08b238f` |
| `python3 -B tools/dev.py /home/simon/.cargo/bin/cargo +1.90.0 check --locked --offline --manifest-path services/image-codec/Cargo.toml --target x86_64-unknown-none` | exit 0 | `/tmp/nieuwos-image-exif/none-final.log` — `02db5a66fa0c60ca8618bd08aac556eac4f3ec863e8630a0d353dba6a0f6e126` |
| `python3 -B tools/dev.py /home/simon/.cargo/bin/cargo +1.90.0 fmt --manifest-path services/image-codec/Cargo.toml -- --check` | exit 0 | `/tmp/nieuwos-image-exif/fmt-final.log` — `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `python3 -B tools/preflight.py --strict-host-tests` | exit 2; `cc` ontbreekt in standaardomgeving | `/tmp/nieuwos-image-exif/preflight.log` — `25fd87c7c77ac6e0dee4b08bf48dbae69ce872725b46afce3d825c1b81d23564` |
| `git diff --check` | exit 0 | `/tmp/nieuwos-image-exif/diff-check.log` — `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `python3 -B tools/progress.py --check` | exit 0; bestaande rubricweergave consistent op 41/100 | `/tmp/nieuwos-image-exif/progress-check.log` — `1963f64b94a52a78464f272cac286fcafea13fea2314c5ea8cddbd65dd78be33` |

`--write` is niet uitgevoerd: de gedeelde assessment- en voortgangsbestanden
hebben een exclusieve eigenaar (Manager 89) en zijn conflicterend op PR #156.
De score blijft inhoudelijk ongewijzigd; de controle hierboven beoordeelt geen
nieuw acceptatiebewijs.

## Bronvingerafdrukken

| Bestand | SHA-256 |
|---|---|
| `services/image-codec/src/lib.rs` | `b30a9102aa6ef836adcac29a1fdb09649ee78827a2b63f7d0ab74e07b1918d7c` |
| `services/image-codec/tests/decode.rs` | `7623bf1d46120a8657208c1640f1f5bad700470836e1d9a1e03e7f3119d8dca3` |
| `docs/implementation/NATIVE-IMAGES.md` | `b5241a547c04756030fdd1da488b7b8e86884620b6dd572c577d71353ead1b64` |
