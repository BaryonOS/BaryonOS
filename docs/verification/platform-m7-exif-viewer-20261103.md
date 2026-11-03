# TASK-PLATFORM-M7-EXIF-VIEWER

## Contract en bron

Basis `78ff7c03b015ad16fe665ae7df9fe239d078e08d` (main), eigen worktree
`/home/simon/.copilot/session-state/edf11e7c8b3e4c9ba1a91b7ef0779695/files/platform-next`,
branch `platform/terminal-next-20261103`.

Doel: bestaande JPEG-decode en EXIF-orientatie verbinden met de bestaande
image-viewer, van geselecteerde bytes naar werkelijk berekende BGRX-tegels.
Geen nieuwe decoder, terminalcore, browserengine, rechtenlaag of kernel-ABI.
Schrijfbestanden: codecbron en decode-tests, viewerbron/tests/README,
de direct gerelateerde EXIF-consumentparagraaf in NATIVE-IMAGES en dit bewijs.
Geen centrale score/statuswrites of wijziging aan andere workstreams.

Terminal-dependency op deze basis: `sdk/runtime/src/process.rs::LaunchImage`
en `native::spawn` starten alleen een vooraf toegekende ingebouwde image.
`sdk/examples/supervisor` gebruikt een vaste bootstrap; de gearchiveerde
terminal-core is nog niet op main geintegreerd en er is geen vrijgegeven
Terminal-launch-/I/O-consumer. Dit wordt niet opgelost met een derde core
of ongeautoriseerde kernelbootstrap. De browserroute heeft evenmin een
geintegreerde upstream-engine-, process- en surfaceketen; geen vervolg van
het afgeronde scraperexperiment. Deze onafhankelijke bestaande renderconsumer
is daarom de gekozen kleine productstap, geen nieuwe algemene foundation.

## Implementatie en invariant

- De codec biedt de bestaande displayafmetingen en een gecontroleerde inverse
  coordinaatmapping aan. Alle acht EXIF-orientaties, lege dimensies en
  ongeldige coordinaten zijn expliciet afgedekt.
- De viewer controleert dat displayafmetingen en orientatie overeenkomen
  voordat een decoded resultaat gepubliceerd wordt.
- Fit, zoom, pan, semantische grootte en pixelindexering gebruiken displayassen.
  Een tegel leest rechtstreeks de juiste raw RGBA8/RGBA16-samples; geen nieuwe
  beeldbuffer of renderallocatie.
- Originele objectbytes, metadata en raw pixels blijven ongewijzigd.
  `metadata_transforms_applied` blijft de bestaande raw-samplevlag, geen
  ICC-kleurbeheerclaim. De bestaande afronding/alpha/background/SUR1-tegelgrens
  en generatie-/ACK-regels veranderen niet.

Dit is een hostgeteste app-local renderconsumer; geen native viewerwindow,
objectportal, resourceautorisatie of gebruikerstest. Correlatiegegevens zijn
geen capability. Geen nieuwe dependency, lockfile, unsafe of platformrecht.

## Rode en groene hostproef

Omgeving: bestaande Rust 1.90.0 en
`/tmp/nieuw-os-c-toolchain/cc-wrapper`, met bijbehorende bestaande
`COMPILER_PATH`, `LIBRARY_PATH` en `LD_LIBRARY_PATH`; geen installatie.
`CARGO_TARGET_DIR="$PWD/target/platform-viewer-host"`.

| Exact commando | Resultaat |
|---|---|
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/image-viewer/Cargo.toml` (alleen regressies toegevoegd) | exit 101: bestaande 15 groen, vier nieuwe tests rood |
| hetzelfde commando na fix | exit 0: 19 tests (1 unit, 18 integratie) |
| `cargo +1.90.0 test --offline --locked --manifest-path services/image-codec/Cargo.toml` | exit 0: 24 tests (16 decode, 5 snapshot/OOM, 3 upsample) |
| `cargo +1.90.0 clippy --offline --locked --manifest-path services/image-codec/Cargo.toml --all-targets -- -D warnings` | exit 0 |
| `cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/image-viewer/Cargo.toml --all-targets -- -D warnings` | exit 0 |
| `cargo +1.90.0 fmt --manifest-path services/image-codec/Cargo.toml -- --check` | exit 0 |
| `cargo +1.90.0 fmt --manifest-path sdk/examples/image-viewer/Cargo.toml -- --check` | exit 0 |
| `git diff --check` | exit 0 |

De vier rode tests tonen: spiegeling tekent raw pixels; geroteerde
beeldafmetingen/letterboxing zijn fout; malformed displayafmetingen worden
geaccepteerd; echte EXIF-JPEG-tegels wijken af van de codecdisplaypixels.
De matrix gebruikt onafhankelijke 3x2-verwachtingen voor alle acht orientaties,
RGBA8/RGBA16 en alpha. De echte JPEG-proef decodeert acht EXIF-varianten en
controleert iedere pixel van iedere tegel tegen `rgba8_into` (voorwaartse
mapping, terwijl de viewer de inverse mapping gebruikt).
Geroteerde Fit/zoom/pan zijn bij beide UI-schalen gecontroleerd.

Logs onder `build/platform-m7-evidence/`, SHA-256:

```text
729fdf5832e6633f988deaa36b406c64819b33c45916fd6e484ba1221b9b5234 viewer-red.log
21cf3a20c0e2d670df7117e14e69be88ef33df7a5760a562edd7fe9c3a7d86c0 viewer-green.log
a0ceb417522ef29956c9e41003789cff643828b592a5af43c04c38aabb49f220 host-gates.log
```

Bronhashes van de geteste code:

```text
0a2580486b51d90d190e368736348e5a28ba464ae5101597a3981e787975dcd2 sdk/examples/image-viewer/src/lib.rs
564100162b4d663d3f9b546c2a227f960d8663b420dab4eca59fbb442c538989 sdk/examples/image-viewer/tests/viewer.rs
c65b0e00315aaaa85905433ad2dda7b76c90428bff62b1f35af350dec1c77f72 services/image-codec/src/lib.rs
dd2ba0da7fbf3c19f7acb463b6534cc293b2b86c3b6f115cd18c3e2b1e5f4f2b services/image-codec/tests/decode.rs
```

## Finale target- en native decoderregressie

Codecommit: `30c4effe1d6656c971e150dfcb0ab381442a84b8`.
Na dit codecheckpoint veranderen alleen documentatie en het ontvangstbewijs,
geen Rust-bron of tests. De SDK-README telt wel mee in de algemene
bronfingerprint; het native resultaat hieronder blijft daarom expliciet aan
de volledige codecheckpointbron gebonden, niet aan de latere documenthead.

Gebruikte bestaande omgeving:

```sh
export PATH=/tmp/nieuw-os-c-toolchain/bin:/home/simon/.cargo/bin:/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-studious-sniffle/.tools/nieuw-os/qemu:$PATH
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/tmp/nieuw-os-c-toolchain/cc-wrapper
export COMPILER_PATH=/tmp/nieuw-os-c-toolchain/root/usr/libexec/gcc/x86_64-linux-gnu/15
export LIBRARY_PATH=/tmp/nieuw-os-c-toolchain/root/usr/lib/x86_64-linux-gnu:/tmp/nieuw-os-c-toolchain/root/usr/lib/gcc/x86_64-linux-gnu/15
export LD_LIBRARY_PATH=/tmp/nieuw-os-c-toolchain/root/usr/lib/x86_64-linux-gnu
export OS_OVMF_CODE=/tmp/nieuw-os-qemu/root/usr/share/OVMF/OVMF_CODE_4M.fd
export OS_OVMF_VARS=/tmp/nieuw-os-qemu/root/usr/share/OVMF/OVMF_VARS_4M.fd
export CARGO_NET_OFFLINE=true
```

Finale commando's, allemaal exit 0:

| Exact commando | Resultaat |
|---|---|
| `CARGO_TARGET_DIR="$PWD/target/platform-viewer-none" cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/image-viewer/Cargo.toml --target x86_64-unknown-none` | no_std-viewerbibliotheek gebouwd |
| `CARGO_TARGET_DIR="$PWD/target/platform-viewer-none" cargo +1.90.0 check --offline --locked --manifest-path services/image-codec/Cargo.toml --target x86_64-unknown-none` | codec groen |
| `NOS_BUILD_DIR="$PWD/build/platform-m7-native" NOS_TARGET_DIR="$PWD/target/platform-native" python3 -B -u tools/build.py --kernel-feature native-system-images` | kernel/loader/diensten/probe gebouwd |
| `NOS_BUILD_DIR="$PWD/build/platform-m7-native" NOS_TARGET_DIR="$PWD/target/platform-native" python3 -B -u tools/run_images.py --case normal` | echte decoder-VM: gate True, QEMU 33, 6,023 s |
| `python3 -B -m unittest discover -s tools/tests -p test_images.py` | 20 harnastests groen; getoonde `gate=False`/`ValueError` zijn verwachte negatieve fixtures |
| `python3 -B tools/progress.py --check` | 41/100, consistent; alleen lezen, geen nieuwe beoordeling |

QEMU 10.2.1, `pc-q35-10.0`, TCG single-thread, qemu64, 256 MiB,
eigen wegwerpvolume en gekopieerde OVMF CODE/VARS; geen fysieke schijf.
Resultaat:
`build/platform-m7-native/run/system-normal-d6602082874e47fc807c075fd5264bff/`.
`result.json` bindt commit `30c4effe1d6656c971e150dfcb0ab381442a84b8`
en sourcefingerprint v3
`24a83ee4be9a2cbf1f56f4def29a2ed5655de79e2417c1206148d135d8142b9d`.
`passed=true`, `image_gate=true`, `status=completed`, `timed_out=false`.
Werkelijke probe: acht PNG, drie JPEG, 149647 pixels, tien negatieve gevallen,
een annulering, nul live allocaties en ongewijzigd pixelorakel
`548683896226622996`. De stackguard en 16 stackpagina's blijven behouden.

SHA-256:

```text
2b2a1e998ffc27d3a6c8a98573187a46e4876de19f95d3557cadf2b31c527288 result.json
9e1d3c3544948c8d21ec0b8fc8c7119110265b34b3b7bf07255054260033733b serial.txt
3b8c1093a73b12c3c556a69d37ab242c4c3b28bf55b2bb004907b583a5e19983 build/platform-m7-native/manifest.json
681394d2a4f4c9c7099730f15c578690dfb170feabb3a08cfc81fa570c8bf653 image-probe ELF
d52aea63c6d893023ea68a58392fdb1d397b2586c1cb91d7291cc9a445bbb5e0 kernel ELF
8e7ad96089d890fcc5a04065ecd57affa89f2a901ecfe0ebf8531b929902b24a UEFI loader
874f15a7c9110e78a89db159f0f68fc97e9076b74777520de1c235426f758aa7 OVMF CODE
5d2ac383371b408398accee7ec27c8c09ea5b74a0de0ceea6513388b15be5d1e OVMF VARS-template
```

Aanvullende logs onder `build/platform-m7-evidence/`, SHA-256:

```text
f476b470ebf72c0d114cfcadc2d466c35a6ed2c7057a9df2b619d9298577f684 none-build.log
befd2102db0ee72d8c373bcd48c1db78a782bc1d270e4f1fe82ed3d2945a48b6 native-build.log
b4c02be39c07fe463f101692a3b65e430745957fe5ea145b7d6c85f267262a4c native-run.log
6e9b0229e42cbebc5e2cad01db7487236c0e46a9c596de4bcc30490c1fdfb6fb image-harness.log
1963f64b94a52a78464f272cac286fcafea13fea2314c5ea8cddbd65dd78be33 progress-check.log
```

## Resterende grenzen en overdracht

De echte VM toetst de bestaande decoderketen, **niet** de app-local viewer.
De viewer is hostgetest en voor de eigen-kerneltarget gebouwd; native
appstart, objectportal, SUR1-presentatie, bediening, OOM/cancel/revoke en
menselijke/hardware-/accessibility-/releasevalidatie blijven open.
CI op deze nieuwe branch is niet als uitgevoerd geclaimd.

FR-A03/A04 blijven open; voorstel score **41 -> 41/100**, beoordeling bij de
integrator. Geen centrale status-/scorewrites, eigen cleanup of merge.
De Terminal- en browserdependencies zijn hierboven geregistreerd, niet met
nieuwe foundations omzeild. De volgende verticale integratie vraagt een
afzonderlijk vrijgegeven native launch-/object-/surfacecontract.
Rollback: deze kleine codec-/viewerwijziging terugdraaien; geen opslagformaat,
recht, bootstrap of ABI gewijzigd.

## Integratiereview PR165, 3 november 2026

PR-head `b14bdde173a558f8465408f8f05c9e8f6e06028f` is in de eigen
integratieworktree samengevoegd met main
`d10b5cdaf56774a39e3ef10463f674c6188a463a` tot
`a3e017d3c7f1d458f5a6960b56b832dc8992ce43`. Main voegde uitsluitend
onderzoeksdocumentatie toe; de vier hierboven gehashte Rustbestanden zijn
ongewijzigd. De begrensde inhoudelijke review controleerde inverse mapping,
afmetingsvalidatie, sample-indexering, Fit/zoom/pan en bestaande generatie-/ACK-
regels. Geen concrete codeblocker gevonden; dit is geen onafhankelijke audit.

Op deze samengevoegde bron opnieuw uitgevoerd, alle exit 0:

- Voor beide manifests `sdk/examples/image-viewer/Cargo.toml` en
  `services/image-codec/Cargo.toml`:
  `cargo +1.90.0 test --offline --locked --manifest-path <manifest>`,
  `cargo +1.90.0 clippy --offline --locked --manifest-path <manifest> --all-targets -- -D warnings`
  en `cargo +1.90.0 fmt --manifest-path <manifest> -- --check`.
  Viewer: 19 tests; codec: 24 tests.
- `cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/image-viewer/Cargo.toml --target x86_64-unknown-none`
- `cargo +1.90.0 check --offline --locked --manifest-path services/image-codec/Cargo.toml --target x86_64-unknown-none`
- `git diff --check`

Eigen targetmappen: `target/pr-agent-c-host` en `target/pr-agent-c-none`;
lokale logs: `build/pr-agent-c-165/host.log` en `none.log`.
Geen native VM-herhaling voor deze document-only main-synchronisatie.
De native decoderproef hierboven blijft uitsluitend bewijs op `30c4effe`;
native vieweracceptatie blijft open. Verse vereiste GitHub-checks worden
afzonderlijk op het definitieve PR-head beoordeeld, niet uit oud groen afgeleid.
Centrale voortgang en metadata zijn eigendom van de hoofdcoordinator;
voorstel blijft **41 -> 41/100**, zonder nieuw native productbewijs.
