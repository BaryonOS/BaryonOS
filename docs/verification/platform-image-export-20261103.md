# TASK-PLATFORM-M7-IMAGE-EXPORT

Basis `d10b5cdaf56774a39e3ef10463f674c6188a463a` (gefetched main bij start),
branch `platform/image-bmp-export-20261103`.
Geteste code-/test-/README-/lockfilebron:
`26e6edeb55ade22b13449cf4d5f3fccea205ea6b`.
De vervolgcommit voegt uitsluitend deze receipt toe.
Scope: nieuwe `sdk/examples/image-export/**` plus dit bewijs;
geen codec-, viewer-, CSV-, General-, gedeelde workspace-, kernel-, driver-,
centrale status-/score- of autorisatiewijziging. Niet afhankelijk van #165/#169.

## Resultaat en ontwerpgrens

Een werkelijk uitgevoerde hostconsumer leest PNG/JPEG via stdin, gebruikt de
bestaande onderhouden decoder en `Image::rgba8_into`, en schrijft een
zelfstandig leesbaar ongecomprimeerd RGB24-BMP-bestand via stdout.
De verplichte expliciete RGB-achtergrond bepaalt alpha-afvlakking.
De uitvoer bevat displaygeorienteerde pixels, geen EXIF/ICC/andere metadata.
RGBA16 wordt met bestaande codecafronding naar 8 bit gebracht.
Originele bytes en raw samples blijven ongewijzigd.
Geen ICC-/gamma-/lineairlichttransformatie, metadata-/precisieroundtrip-
of fotogetrouwheidsclaim. Het profiel staat in de subtree-README.

Primaire bronnen: Microsoft, **BITMAPFILEHEADER** (5 december 2018),
<https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapfileheader>;
**BITMAPINFOHEADER** (26 april 2023),
<https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapinfoheader>.
Geraadpleegd op 3 november 2026. Positieve hoogte/bottom-up, BGR-triplets,
rijen op vier bytes, 24 bpp, BI_RGB, planes=1 en nulreserved/padding
zijn expliciet gecontroleerd. Geen nieuwe decoder of algemene beeldengine.

Invoer 4 MiB, 4 * 1024 * 1024 pixels, aslengte 16384, uitvoer 16 MiB;
caller kan uitvoerbudget verlagen. RGBA8- en BMP-werkbuffers zijn afzonderlijk
begrensd/fallible; samen maximaal circa 32 MiB naast decoder/raw samples.
Decoderwerkbudget 64 MiB is admission, geen harde heapquota. Upstream kan
infallible alloceren; native gebruik vereist een afzonderlijk begrensd proces.
Geen nieuwe registrydependency: lockfile hergebruikt de bestaande
image-codec-closure en gecontroleerde zune-JPEG-vendorpatch.

## Uitgevoerd bewijs

Werkdirectory
`/home/simon/.copilot/session-state/edf11e7c8b3e4c9ba1a91b7ef0779695/files/platform-bmp`.
Rust 1.90.0, bestaande `/tmp/nieuw-os-c-toolchain/cc-wrapper` en sysroot;
`CARGO_TARGET_DIR="$PWD/target/platform-bmp-host"`.
Geen installatie/download of wijziging aan bestaande lockfiles.

| Exact commando | Resultaat |
|---|---|
| `cargo +1.90.0 generate-lockfile --offline --manifest-path sdk/examples/image-export/Cargo.toml` | exit 0; alleen nieuw consumerlockfile |
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/image-export/Cargo.toml --features host-cli` | exit 0; 6 exporttests + 2 echte CLI-procestests |
| `cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/image-export/Cargo.toml --features host-cli --all-targets -- -D warnings` | exit 0 |
| `cargo +1.90.0 fmt --manifest-path sdk/examples/image-export/Cargo.toml -- --check` | exit 0 |
| `cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/image-export/Cargo.toml --target x86_64-unknown-none` | exit 0; alleen targetbibliotheek, geen uitvoering |
| `python3 -B sdk/examples/image-export/tests/interop.py target/platform-bmp-host/debug/image-bmp` | exit 0; 30 werkelijke CLI-exports, zelfstandig BMP-reader/orakel |
| `target/platform-bmp-host/debug/image-bmp --background ffffff < services/image-codec/fixtures/baseline.jpg > build/platform-bmp-evidence/baseline.bmp` | exit 0 |
| `file build/platform-bmp-evidence/baseline.bmp` | exit 0; PC bitmap, Windows 3.x, 129 x 257 x 24, size 99770, offset 54 |
| `target/platform-bmp-host/debug/image-bmp --background 123456 < services/image-codec/fixtures/rgba16.png > build/platform-bmp-evidence/rgba16.bmp` | exit 0 |
| `file build/platform-bmp-evidence/rgba16.bmp` | exit 0; PC bitmap, Windows 3.x, 97 x 65 x 24, size 19034, offset 54 |
| `git diff --check` | exit 0 |

De eerste zes tests waren direct groen; dit is nieuwe functionaliteit,
geen geclaimde rood-naar-groenregressie. Na toevoeging van twee tests
voerde de finale run alle acht geregistreerde tests uit.
Gerichte negatieve tests: nul/overflow/te grote afmetingen, afwijkende
displaygeometrie, onjuiste RGBA8/16-samplelengte, ongeldige achtergrond,
ontbrekende expliciete keuze, ongeldige opties, ongeldige/te grote invoer,
exacte uitvoergrens 57/58 bytes. Fouten geven geen succesoutput.
Geen nieuwe trust-/bevoegdheidsgrens; geen native autorisatietest geclaimd.

De onafhankelijke Python-reader gebruikt bestaande fixturehashes en
Pillow-/synthetische referentiepixels, niet dezelfde decode/exportfunctie.
Tien werkelijke PNG/JPEG-fixtures maal drie achtergronden: PNG exact,
JPEG maximaal de bestaande tolerantie 3 (gray maximaal 1); geen verruiming.
Ook RGBA16-afronding en echte EXIF6-JPEG met verwisselde assen.
CMYK blijft door de bestaande decoder geweigerd en wordt niet als ondersteund
geclaimd. Rust controleert daarnaast alle acht orientaties tegen expliciete
asymmetrische 3x2-matrices, alle vier rijpaddinglengtes en bottom-up-BGR.
`file` identificeert de echte hostoutput onafhankelijk, maar is geen
volledige externe applicatieroundtrip- of compatibiliteitscertificering.

Logs onder `build/platform-bmp-evidence/`, buiten Git:

```text
ecb328a384bba238ba1b631d26bd9d0111f73714eaa15a646792cbf9cf3d5e95 final-gates.log
```

Finale gemeten commandoduren: Clippy 0,133 s, fmt 0,043 s,
no_std-build 0,062 s, Python-orakel 0,437 s. Het orakel voert werkelijk
30 processen uit; buildcachegebruik wordt niet als uitvoeringsbewijs geteld.
De native-targetbuild bewijst uitsluitend compilatie.

SHA-256 van gebruikte artifacts:

```text
11740bbf555eba508bf76abe9ec96467da9cd0f4b315ef7bfc5299be02456ece baseline.bmp
240c3d72524d5bad18ab3b7e801d746d2ff9a745df9d89955433a8a22d133ce3 rgba16.bmp
8ec50e09646d3f49d06c21ab2bc27578e52329423aa3a0da7e723687b332afbf target/platform-bmp-host/debug/image-bmp
0259594cbca8d20b84815cea41e79207da27afa06a52bf11f272b31aa558e816 target/platform-bmp-host/x86_64-unknown-none/release/libnieuw_os_image_export.rlib
```

Bronhashes ten tijde van de uitvoering:

```text
b63589367bbce1a6a1e898a1792b7c8009fe8834eb22573c987e5535d9580385 src/lib.rs
ed34c2d6850ef831f8f75c5fae515bc32a653f4bd74a87b1046c04b79e99b28d src/main.rs
ba3a0b524fde47121651ec8f5bc4be20a6aac624ff33536e8450c723cedae522 tests/cli.rs
74427ead3c6f46f8dc6ddc90ab4e9108dbfc775f0fe732c6005789b7870c3ecb tests/export.rs
7bec716dfeba8c67006ed7a41fc513b157f13bd8d82556078694edfe3f205129 tests/interop.py
```

## Overdracht, blockers en rollback

Hostfunctionaliteit plus no_std-compilatie. Native executable, exportdialoog,
geautoriseerde volume-save/objectportal, GUI, native OOM-/crashherstel, QEMU,
fysieke hardware, menselijke gebruikerstest, onafhankelijke securityaudit
en releaseacceptatie **niet uitgevoerd/niet bewezen**. Host-CLI erft stdio;
geen kernelcapability of resourcegrant. Uitvoer is niet atomair: stdout-
writefouten kunnen gedeeltelijke output opleveren.
Die native consumerpaden blijven bij bestaande app-/portal-/launch-eigenaars;
geen parallelle broker gebouwd. Integratiereview/nieuwe-head-CI blijven open.

Scorevoorstel **41 -> 41/100** (FR-A03/image-interoperabiliteit blijft native
open); centrale score/status bij General, geen schrijfclaim daarop.
Na push, bestaande/nieuwe draft-PR en overdracht vervalt deze actieve claim.
Volgende milestone start onafhankelijk op opnieuw opgehaalde main.
Rollback: verwijder uitsluitend nieuwe image-export-subtree en receipt;
geen bestaande ABI, brondecoder of opslagformaat verandert.

## Integratiereview PR173, 3 november 2026

Reviewbasis `2d19690b9fa273be8232701d5ceb379f5d83539c`, main
`d10b5cdaf56774a39e3ef10463f674c6188a463a`. De begrensde inhoudelijke
review controleerde BMP-headers, rijuitlijning, bottom-up-BGR, alpha,
RGBA16/EXIF, afmetings-/uitvoergrenzen en fouten. Geen productcodeblocker
gevonden. Wel dezelfde echte CLI-testhelperrace als bij CSV: vroeg
afgewezen argumenten sluiten stdin voordat de helper klaar is met schrijven.
Een geregistreerde 128-KiB-regressie reproduceerde BrokenPipe op de oude
helper (Cargo exit 101). Een eerdere poging registreerde deze test niet
correct en telt niet als bewijs; dit is hersteld voor de rode en groene run.

De helper accepteert nu alleen BrokenPipe bij een daadwerkelijk falend
kindproces. Andere schrijffouten en een onverwacht succesvolle exit blijven
testfouten. Geen productcode, dependency, formaat of decoder veranderd.

Met Rust 1.90.0 en eigen targets `target/pr-agent-c-host` en
`target/pr-agent-c-none` opnieuw uitgevoerd, alle exit 0:

```text
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/image-export/Cargo.toml --features host-cli
# 6 exporttests en 3 werkelijke CLI-procestests
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/image-export/Cargo.toml --features host-cli --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path sdk/examples/image-export/Cargo.toml -- --check
python3 -B sdk/examples/image-export/tests/interop.py target/pr-agent-c-host/debug/image-bmp
# 30 werkelijke outputs; PNG exact, JPEG bestaande toleranties 3/1
cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/image-export/Cargo.toml --target x86_64-unknown-none
git diff --check
```

Lokale logs onder `build/pr-agent-c-173/`, SHA-256:

```text
9ed38f185e83588211ba70243091bfec1d4f276028a492ca8d6875ba94ae8ede pipe-red.log
d3056d6fd5179e50206a8920dae5d54003d741fc0410134e4f07c7bc5dbaa18f green.log
85a729dd5f760ef71d3a38f2aae67cc2fb32da5cbdaa9860c175ad6edd677eef tests/cli.rs
```

Oude hashes en aantallen hierboven blijven uitsluitend brongebonden aan
`26e6edeb`. Geen native runtimeconsumer gewijzigd of toegevoegd; QEMU,
hardware-, menselijke en releasevalidatie niet uitgevoerd. Verse vereiste
GitHub-checks worden apart op het definitieve PR-head beoordeeld.
Scorevoorstel blijft **41 -> 41/100**, centrale metadata bij de hoofdcoordinator.
