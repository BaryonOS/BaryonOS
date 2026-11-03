# TASK-PLATFORM-M11-SLIDE-RENDER-CONSUMER

Basis `ab85d6339ce5bdddb10a2de171d0c1b04e4b62ff` (gefetched main).
Branch `platform/presentation-preview-20261103`, code-/testbron
`5e3baed2d189de37005aa1c85ec6071fe36d9dce`.
Vervolgcommit wijzigt uitsluitend README-bewijsgrens en deze receipt.
Scope: nieuwe `sdk/examples/presentation/**` plus dit bewijs;
geen General-, graphics-, workspace-, kernel-, driver-, input-, window-,
B02-, launch- of centrale score/statuswrites.
Geen afhankelijkheid van nog niet geintegreerde #165/#169/#173/#176.
De main-delta sinds vorige milestone was uitsluitend #151-auditdocumentatie;
gebruikte instructies/interfaces waren ongewijzigd.

## Resultaat en profiel

Een werkelijk uitgevoerde hostconsumer importeert een klein UTF-8-LF-tekstdeck,
selecteert een dia en rendert titel/body/paginateller met de bestaande
`ui/graphics`-atlas/Canvas en gedeeld MATERIAAL-palet.
Het model kan volgende/vorige/select en leent de originele UTF-8-bron.
De host-CLI levert een daadwerkelijk rasterframe, geen DOM/HTML/mock.
Het no_std-model/renderpad heeft daarmee een concrete afnemer.

640x360 of 1280x720, RGB24 P6-**variant met ongewijzigde UI-RGB-samples**.
Geen BT.709-transferfunctie- of kleurgetrouwheidsclaim.
Primaire bron: Netpbm, *PPM Format Specification*, 7 november 2025,
<https://netpbm.sourceforge.net/doc/ppm.html>, geraadpleegd 3 november 2026.
P6/header/rijvolgorde/samplelengte zijn gecontroleerd. De README verduidelijkt
de samplevariant naar aanleiding van deze primaire bron, zonder codewijziging.

Maximaal 32 KiB invoer, 32 dia's, 256 bytes/titel, twaalf bodyregels,
512 bytes/bodyregel. Alleen het expliciete eigen tekstprofiel, geen
Markdown/PPTX/ODP. Schaal, volledige titel-/bodybreedte, totale teksthoogte
en uitvoerbuffer worden voor iedere pixelwrite gecontroleerd:
overflow is een fout, niet clipping/ellipsis/stille inhoudsverwijdering.
Zichtbare atlasfallback behoudt bronbytes, maar is geen volledige
Unicode-fontdekking/shaping. Geen nieuwe tekst-/graphicsengine.
Geen nieuwe registrydependency of unsafe.

## Uitgevoerd bewijs

Werkdirectory
`/home/simon/.copilot/session-state/edf11e7c8b3e4c9ba1a91b7ef0779695/files/platform-presentation`.
Rust 1.90.0, bestaande `/tmp/nieuw-os-c-toolchain/cc-wrapper` en sysroot;
`CARGO_TARGET_DIR="$PWD/target/platform-presentation-host"`.
Geen installatie/download.

| Exact commando | Resultaat |
|---|---|
| `cargo +1.90.0 generate-lockfile --offline --manifest-path sdk/examples/presentation/Cargo.toml` | exit 0; alleen nieuwe dependencyvrije registryclosure, path naar bestaande graphics |
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/presentation/Cargo.toml --features host-cli` | exit 0; 5 model/render- + 2 echte CLI-procestests, 0,584 s |
| `cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/presentation/Cargo.toml --features host-cli --all-targets -- -D warnings` | exit 0, 0,345 s |
| `cargo +1.90.0 fmt --manifest-path sdk/examples/presentation/Cargo.toml -- --check` | exit 0, 0,051 s |
| `cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/presentation/Cargo.toml --target x86_64-unknown-none` | exit 0, 0,329 s; bibliotheek, geen uitvoering |
| `python3 -B sdk/examples/presentation/tests/interop.py target/platform-presentation-host/debug/slide-preview build/platform-presentation-evidence` | exit 0; vier echte CLI-rasteroutputs geinspecteerd |
| `target/platform-presentation-host/debug/slide-preview --slide 1 < sdk/examples/presentation/fixtures/deck.txt > build/platform-presentation-evidence/first.ppm` | exit 0 |
| `target/platform-presentation-host/debug/slide-preview --slide 2 --scale 2 < sdk/examples/presentation/fixtures/deck.txt > build/platform-presentation-evidence/second-2x.ppm` | exit 0 |
| `file build/platform-presentation-evidence/first.ppm build/platform-presentation-evidence/second-2x.ppm` | exit 0; Netpbm raw pixmap, 640x360 en 1280x720 |
| `git diff --check` | exit 0 |

De eerste uitgevoerde keten was groen; geen rood-naar-groenregressie geclaimd.
Tests: exacte bronbinding/Unicodebehoud, selectiebounds, next/previous,
einde/begin, echte verschillende diaframes op beide schalen, deterministisch
terugnavigeren, ongewijzigde buffertail en nul writes na geometry-/overflowfout.
Malformed/te grote invoer, lege titels/dia's, CRLF/tabs/NUL, ongeldige UTF-8/
CLI/schaal/selectie en 32/33-diagrens leveren expliciete fouten.
Een onbekende glyph rendert exact als de gedeelde zichtbare fallback,
terwijl bronbytes gelijk blijven.

Python inspecteert P6-header, RGB-lengte, achtergrond/accent en niet-lege
titel-/body-/footerregio's van vier echte CLI-frames. Dit is geen onafhankelijk
volledig typografisch pixelorakel. De standaard-Python-PNG-serialisatie is
uitsluitend een testhulppad naar beelden buiten Git, geen productencoder.
Twee werkelijk gerenderde PNG's zijn door de agent visueel geopend:
dia 1 op 640x360 en dia 2 op 1280x720 tonen de bedoelde titel, twee bodyregels,
accent en paginateller zonder zichtbare afsnijding. Dit is agentinspectie,
geen onafhankelijke menselijke gebruikerstest of native bedieningsproef.

Logs onder `build/platform-presentation-evidence/`, SHA-256:

```text
5d76b40eec5426a89b66e4de72fb0e70a3216a0b0c817c3bf8cc0584a6a71f9e gates.log
d9ac6965fb44bf43dc78aeb5c5129112770bfa770dc9c6ca61afac3228d9ea0a raster-oracle.log
```

Artifacts:

```text
84005cedb5f6c8513f3021755be58dd8e21de83345b41fa0e7d0520e653dc120 target/platform-presentation-host/debug/slide-preview
5298c424e181ba0ab9b00549b04ddd80f9617ff2e0fae6b289d10afd8132dd4e target/platform-presentation-host/x86_64-unknown-none/release/libnieuw_os_presentation.rlib
26f6f56b5506aaf4a37c7c51a17cb32f0e83b51ea4cc08e9c72bcb33702debcb slide-1-scale-1.png
93326efadafe0dd06a899c55aacdce60504646189c60ce43a4b5c1207bbce32d slide-1-scale-2.png
96ed87e63efebbcfb4c79219ebd25ed3f32a684c2f34439c1f7b01a155be0a99 slide-2-scale-1.png
badcbe40de3d190bdc76f58cb9f40251545d1ac347998416cc273f5926d28f98 slide-2-scale-2.png
```

Bronhashes:

```text
da6eec19ef20bb9d25f0793af11bb7ef1d98ffefbba8d30859d1a2864d625d86 src/lib.rs
a5b16f050d2e0dc4e59434b49c2d86992f1e6a67f8b0b2e226d149d95b7f65c1 src/main.rs
ba4cfe442ced695bcc419d1499fbbe8cd50eef158028e8ee82af5e7482f862fd tests/cli.rs
c392582821528844ea8a4f83e170a5fd69c798e95939c7c76ab32d34392c0215 tests/deck.rs
1254d624376a5d72cfe6e2e7e6eee4f3e1a38451310f512d45c4c4c60767492d tests/interop.py
```

## Integratiequeue en open grenzen

**QUEUED-FOR-INTEGRATION**, geen nieuwe PR volgens de laatste
backpressure-instructie zolang de integrator geen publicatieslot vrijgeeft.
Push is daarom niet als volledige PR-overdracht of MERGED gepresenteerd.
De branch/head en eventuele app-autocreatie worden afzonderlijk gemeld.
Technische schrijfclaim vervalt na deze queuelevering; geen wachten op CI/merge.

Hostfunctionaliteit plus no_std-compilatie. Native app/venster/input/surface,
saveportaal, presentatie-editor, PPTX/ODP, gedeeld Work-documentmodel,
QEMU, hardheapquota/OOM-herstel, accessibility-/high-DPI-/performance-UX,
hardware-, menselijke en releaseacceptatie **niet uitgevoerd/niet bewezen**.
De bestaande app-/surface-/input-/portalowners bezitten die native koppeling;
geen parallelle broker, rootprofiel of launchregistratie gebouwd.
Volwaardige M11 en FR-A15 blijven open. Scoreadvies **41 -> 41/100**;
centrale beoordeling/status bij General, geen centrale write.

Rollback: verwijder uitsluitend nieuwe presentation-subtree en receipt;
geen bestaande grafische API, ABI of opslagformaat verandert.

## Integratiereview PR182, 3 november 2026

Exact reviewhead `9171756ab5864c09bbcfb6b784e73dae2fee9f58`, main
`ab85d6339ce5bdddb10a2de171d0c1b04e4b62ff`.
De bestaande PR182 is door de hoofdcoordinator expliciet voor integratie
vrijgegeven; de historische QUEUED-FOR-INTEGRATION-tekst hierboven is geen
actuele publicatieblocker of opdracht om een nieuwe PR te maken.

De begrensde inhoudelijke review controleerde stdin-/selectie-/schaalgrenzen,
volledige titel-/bodypreflight voor pixelwrites, bufferlengtes, paginateller,
werkelijk hergebruik van `ui/graphics::Canvas`/`Font`, gedeelde kleuren en
PPM-uitvoergrootte. Geen concrete productcodeblocker gevonden; geen onafhankelijke
audit of native presentatieclaim. Een verkeerd argument kon wel de CLI-testhelper
met BrokenPipe laten panikeren wanneer het kind stdin vroeg sloot. Een nieuwe
geregistreerde 128-KiB-regressie faalde op de oude helper (Cargo exit 101).
De helper accepteert nu uitsluitend BrokenPipe bij een daadwerkelijk falend
kindproces; andere schrijffouten en onverwacht succes blijven testfouten.
Geen productcode of gedeelde renderer gewijzigd.

Met Rust 1.90.0, eigen targets `target/pr-agent-c-host` en
`target/pr-agent-c-none` opnieuw uitgevoerd, alle exit 0:

```text
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/presentation/Cargo.toml --features host-cli
# 5 model/render- en 3 werkelijke CLI-procestests
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/presentation/Cargo.toml --features host-cli --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path sdk/examples/presentation/Cargo.toml -- --check
python3 -B sdk/examples/presentation/tests/interop.py target/pr-agent-c-host/debug/slide-preview build/pr-agent-c-182
# vier echte PPM-frames, header/lengte/accent/titel-/body-/footerregio's
cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/presentation/Cargo.toml --target x86_64-unknown-none
git diff --check
```

Lokale logs onder `build/pr-agent-c-182/`, SHA-256:

```text
c70a04462e227850228839c6af44bc32ca23b78007562fdbd13e2e42c97afd60 pipe-red.log
387b0e8514d7650cf8c2edc0480ad405e9360102bf1db22e7281a53133901a4d green.log
2c129809a99e173e3ceb01fb951f255ff02d0109145030a01940b993f38d6e9a tests/cli.rs
```

Oude hashes en aantallen hierboven beschrijven uitsluitend de oorspronkelijke
bron. De vier gemeten PPM-fingerprints staan in green.log; de PNG's zijn
alleen lokaal testbewijs. Geen nieuwe visuele/humanetest of vollediger
typografisch pixelorakel geclaimd. Geen native runtimeconsumer gewijzigd;
QEMU, PPTX/ODP, native venster/input/surface, opslag en M11/FR-A15 blijven
open. Verse vereiste CI op het finale head is een aparte integratievoorwaarde.
Scorevoorstel **41 -> 41/100**, centrale metadata bij de hoofdcoordinator.
