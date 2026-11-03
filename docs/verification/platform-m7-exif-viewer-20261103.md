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

## Open bij checkpoint

no_std-crossbuilds en de bestaande native decoderregressie volgen na dit
broncheckpoint. Nog geen nieuw native bewijs; een decoder-VM zal evenmin
een native EXIF-viewer bewijzen. FR-A03/A04 blijven open; voorstel score
**41 -> 41/100**, beoordeling bij de integrator. Geen volledige
testreleaseclaim, menselijke audit, hardwaretest of native vieweracceptatie.
Rollback: deze kleine codec-/viewerwijziging terugdraaien; geen opslagformaat,
recht, bootstrap of ABI gewijzigd.
