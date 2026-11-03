# TASK-PLATFORM-M7-IMAGE-CROP

Basis `681f1137e411d6c6fe536bd2bf499c27b34832be` (gefetched main).
Branch `platform/image-crop-20261103`, geteste volledige bron
`9b4cca4ab4ea4a1c9e319ac9d99efc22ff599d04`.
Eerste actieve kerncheckpoint `419dd1f62db019876fd7148e8aa8c804c459eaf7`;
die was nadrukkelijk geen volledige milestoneoverdracht.
Deze vervolgcommit voegt alleen de eigen receipt toe.
Scope: nieuwe `sdk/examples/image-crop/**` en deze receipt.
Geen oude BMP-/EXIF-/viewerwrites, centrale status/score of General/Veld/K05.

## Resultaat

Echte hostbestandsbewerking: bestaande PNG/JPEG-decoder ->
displaygeoriënteerde RGBA8-pixels -> rechthoek -> onderhouden png-encoder.
Crop is byte-exact inclusief alpha en RGB onder transparante pixels.
Coördinaten gelden na de bestaande EXIF-oriëntatie; geen tweede mapping in
productcode. Geen clamp, resampling, matting, bronmutatie of stille 16->8
quantisatie. PNG16 wordt expliciet geweigerd.
Verplichte `--drop-metadata --unmanaged-colour`: metadata/ICC/gamma gaan
bewust niet mee, er wordt geen sRGB/gamma-profiel verzonnen. Output bevat
uitsluitend IHDR/IDAT/IEND. Samples behouden betekent niet kleurweergave
behouden op een bepaald display. Geen kleurbeheerclaim.

No_std-cropbibliotheek, optionele **std-hostencoder** via `host-cli`.
De encoder is niet naar Nieuw OS geport. Default no_std-tree bevat
geen png/crc32fast/flate2. Invoer uitsluitend stdin, uitvoer stdout/stderr;
geen disk-/netwerkautoriteit of nieuwe capability.
Alles vóór stdout wordt afgerond; errors geen successtdout.
Stdoutschrijffout kan gedeeltelijke output hebben gegeven.

README beschrijft exacte budgetten: 8 MiB input/output, 4096 per dimensie,
1048576 pixels, 128 MiB decoderadmission-work, 256 KiB metadata,
1024 PNG-chunks, 128 JPEG-scans en 8 MiB extra cropbudget.
Eigen buffers reserveren fallible; dit is geen native harde heap-/CPUcap.
Upstreamallocaties blijven infallible; native decodergebruik vereist de
bestaande aparte begrensde arena/procesgrens.

## Dependencycontrole vóór compilatie

`png = "=0.18.1"`, defaultfeatures uit, uitsluitend optioneel host-cli.
Voor toevoeging gelezen: primaire registry Cargo.toml.orig, MIT-licentie,
CHANGES.md, std-compatibiliteit en package-include zonder buildscript.
Na manifestwijziging generate-lockfile/fetch met tijdelijk
`CARGO_NET_OFFLINE=false`: exit 0.
Vóór eerste compilatie alle transitive registrymanifests en aanwezige
licentie-artifacts/metadata-buildtargets gecontroleerd.
`THIRD-PARTY.md` bevat de closure met exact gepinde Cargo.lock-checksums:
png0.18.1, bitflags2.13.2, crc32fast1.5.4, cfg-if1.0.5, fdeflate0.3.7,
flate2 1.1.10, miniz_oxide0.8.9/0.9.1, adler2 2.0.1 plus bestaande
codecdeps/simd-adler32. Geen C/zlib-rs/unstable backend geselecteerd.
Een ongebruikte optional zlib-rs-tarball opgehaald door Cargo fetch is
geen geselecteerd buildtarget; featuretree bevestigt miniz_oxide.

Expliciete buildscriptuitzondering: crc32fast/build.rs geheel gelezen;
alleen Cargo's `RUSTC --version` en emit van compiler-cfg/check-cfg,
geen C-compilatie/download/netwerk/extra scriptketen.
No_std-defaultclosure heeft dit buildscript niet.
Geen onafhankelijk securityauditclaim.

## Exacte uitgevoerde controles

Werkdirectory
`/home/simon/.copilot/session-state/edf11e7c8b3e4c9ba1a91b7ef0779695/files/platform-image-crop`.
Rust 1.90.0, bestaande cc-wrapper/sysroot uit random-environment.sh;
`CARGO_TARGET_DIR="$PWD/target/platform-image-crop-host"`.

| Exact commando | Finale uitkomst |
|---|---|
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/image-crop/Cargo.toml --features host-cli` | exit 0; 5 werkelijk geregistreerde tests (1 hostsink + 4 crop), 0,350 s |
| `cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/image-crop/Cargo.toml --features host-cli --all-targets -- -D warnings` | exit 0, 0,113 s |
| `cargo +1.90.0 fmt --manifest-path sdk/examples/image-crop/Cargo.toml -- --check` | exit 0, 0,045 s |
| `cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/image-crop/Cargo.toml --target x86_64-unknown-none` | exit 0, 0,127 s; kerncompile, geen native encoderuitvoering |
| `python3 -B sdk/examples/image-crop/tests/interop.py target/platform-image-crop-host/debug/image-crop build/platform-image-crop-evidence/crop.png` | exit 0, 0,870 s; 325 echte PNG-outputs, 111 afwijzingen |
| `file build/platform-image-crop-evidence/crop.png` | exit 0, 0,004 s; PNG 19x23 RGBA8 non-interlaced |
| `git diff --check` | exit 0, 0,007 s |
| `git diff --cached --check` | exit 0 voor broncommit |
| `cargo +1.90.0 tree --offline --locked --manifest-path sdk/examples/image-crop/Cargo.toml --features host-cli -e features` | exit 0; uitsluitend miniz_oxide-backend voor host |
| `cargo +1.90.0 tree --offline --locked --manifest-path sdk/examples/image-crop/Cargo.toml --target x86_64-unknown-none -e normal` | exit 0; uitsluitend bestaande image-codecclosure, geen png-hostdeps |

Eerste acceptatieketen: tests exit 0 maar Clippy exit 101 omdat de net
toegevoegde malformed-Image-test per ongeluk genest stond en niet
geregistreerd was. Naar moduleniveau verplaatst, geen lintallow; finale
keten voert de test daadwerkelijk uit. Geen sample-/formatbeleid verruimd.

Rust: displaycoördinaten/oriëntatie, alpha/inputbehoud, exact 39/40-byte
extra werkbudget, overflow/leeg/buitenbeeld, malformed callerpixels/
displaydimensies, geen precisionreductie en exacte 8 MiB encoded-output
grens versus één byte extra.
Python: eigen zlib/PNG-orakel controleert chunkvolgorde/CRC/no-ancillary,
IHDR-format/dimensies, zlibstream en alle vijf mogelijke PNG-filters;
alle RGBA-bytes vergeleken. 24 PNG-exports met alpha/tekst/gamma-metadata,
12 oorspronkelijke JPEG-fullframeoutputs met bestaande onafhankelijk
gegenereerde Pillow/libjpeg-referenties (maximaal 3 sampleverschil),
288 crops over alle acht EXIF-oriëntaties byte-exact tegen de volledige
decoded samplebuffer en één daadwerkelijke 1048576-pixelbudgetacceptatie.
111 denials controleren exit!=0, stderr en lege stdout: alle truncaties,
ongeldige CLI/overflow/keuzes, formaat, input/dimensie/pixelgrenzen,
unsupported PNG16 en CMYK. Geen bestaande fixtures geregenereerd of
oude consumenten opnieuw bewezen.

SHA-256 van buiten Git bewaarde artifacts en de geteste bron:

```text
97f8bb5548b0f4ce48ff980a21840689ec537c262639a00d373cc595479471dc build/platform-image-crop-evidence/accepted-gates.log
d9c89c8d8b78bec74edefaf3f075a037b299299ab02669452c1a2a49f66a9e21 build/platform-image-crop-evidence/first-gates.log (Clippy101)
00ada48035b8adafc4198617f50aa0d67b49018904a473915d192a241cedd280 build/platform-image-crop-evidence/crop.png
f831d52bd5928fe2940cae591de2e40921fc7bef50ca0cb54053834726e77811 target/platform-image-crop-host/debug/image-crop
715acc49e7b2daa9fe6a3432e61bbfd2265895c9ae0c351259dcb7bf1045010a target/platform-image-crop-host/x86_64-unknown-none/release/libnieuw_os_image_crop.rlib
2b7f6f3fa00eb3a3fb5d8e6579ca42cd4172147eb9a7ecd4b84c323fa506036d src/lib.rs
4e44dab45d7fabe3e2d51f6ad86cd4a47371a0c90845d4062ec91543e7694637 src/main.rs
4309ffa8ff44e9a97f786fbeb9ec1e5eb17f4e1de854ffe7dec082bce5d84041 tests/crop.rs
d20952f8cb79286bfefb12a9735efea2cdd6ee8ce3096a6fa83a17ddeb655cb9 tests/interop.py
```

## Queue en resterende grenzen

**QUEUED-FOR-INTEGRATION**, geen nieuwe PR zonder OPEN-PR-LEASE.
Hostbestandsfunctionaliteit plus no_std-kerncompile, geen native
PNG-encoding/window/open/save/launch. VM, hardware, menselijke tests,
onafhankelijke audit en releasevalidatie niet uitgevoerd.
Geen volledig beeldbewerkingsprogramma, color-management of volledige
formatconformance geclaimd. Native arena-/launch-/surface-/saveportaal
blijft bij General/Veld/K05; deze slice verruimt geen rechten.
Scoreadvies **41 -> 41/100**; centrale beoordeling bij General.
Rollback verwijdert uitsluitend nieuwe image-crop-subtree en receipt.
