# Presentatiedia's met de bestaande native rasterbibliotheek

TASK-PLATFORM-M11-SLIDE-RENDER-CONSUMER, basis `ab85d633`.
Een echte hostconsumer: lokaal tekstdeck importeren, dia selecteren,
volgende/vorige, pixels renderen met `ui/graphics`, PPM-frame uitvoeren.
Geen HTML/browser, nieuwe font-/graphicsengine of los model zonder afnemer.
Het model/renderpad is `no_std+alloc`; de CLI is een hostprogramma.

```sh
cargo +1.90.0 run --offline --locked --manifest-path sdk/examples/presentation/Cargo.toml --features host-cli --bin slide-preview -- --slide 2 --scale 2 < sdk/examples/presentation/fixtures/deck.txt > dia.ppm
```

`--slide` is een 1-gebaseerd dianummer; optionele `--scale` is 1 of 2.
Frame 640x360 of 1280x720, 24-bit RGB Netpbm P6, geen alpha.
De gedeelde MATERIAAL-kleuren en bestaande Font-presets worden hergebruikt.
Schaal 2 gebruikt de bestaande grotere fontpresets, geen claim van exacte
tweemaal alle glyphafmetingen of gevalideerde high-DPI-/accessibility-UX.

## Klein expliciet tekstprofiel

UTF-8 met **LF**-regels; eerste regel van iedere dia is de titel,
volgende regels zijn body. De exacte regel `---`, met een LF ervoor en erna,
scheidt dia's. Geen Markdown, PPTX, ODP of Microsoft-compatibiliteitsclaim.
Voorbeeld:

```text
Eerste titel
Een regel inhoud.
---
Tweede titel
Meer inhoud.
```

Maximaal 32 KiB invoer, 32 dia's, 256 bytes/titel, twaalf bodyregels,
512 bytes/bodyregel. Geen lege titels/dia's of controletekens behalve LF.
CRLF, tabs en NUL worden expliciet geweigerd, niet stil herschreven.
Het model leent de oorspronkelijke UTF-8-bron zonder kopie/verlies;
beperkte atlasglyphdekking gebruikt een **zichtbaar vervangingsteken**.
Dat behoudt bronbytes, maar is geen volledige Unicode-shaping/fontondersteuning.
Geen tekstedit/exportroundtrip of inhoudelijke interpretatie.

## Render- en resource-invariant

Voor de geselecteerde dia worden uitvoerschaal, buffer, volledige titel-
en bodybreedte en totale teksthoogte **voor iedere pixelwrite** gecontroleerd.
Te brede/lange inhoud geeft `TextOverflow`; geen ellipsis, stille clipping,
verborgen diaregels of succesoutput. Het is geen automatische layout-/
wordwrap-engine; de gebruiker kan regels verdelen of een extra dia maken.
Een niet-passende dia blijft selecteerbaar en de bron blijft intact.
Navigeren voorbij begin/einde verandert de selectie niet.

De deckvector reserveert fallible maximaal 32 entries; bronbytes blijven
bij de caller. Canvasbuffergrootte maximaal 921600 u32-pixels (circa 3,52 MiB),
PPM-uitvoer circa 2,64 MiB plus header. De hostconsumer reserveert fallible;
geen native harde heapquota/OOM-herstel-/performanceclaim.
De CLI schrijft pas stdout nadat import/selectie/render/outputallocatie slagen.
Stdoutwrite zelf is niet atomair en kan bij een I/O-fout gedeeltelijk zijn.

Alleen een pathdependency op de bestaande `ui/graphics`, geen nieuwe
registrydependency of unsafe. Geen bestaande grafische/shared workspace-,
General-editor-, kernel-, driver-, input-, window-, B02- of launchwrites.
Geen bestands-, netwerk-, launch- of trusted-UI-bevoegdheid verkregen.
Een native slideshow vereist de bestaande app-/surface-/input-/portalowners;
dit bouwt geen parallelle broker of proceslifecycle.

## Uitgevoerde acceptatie

```sh
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/presentation/Cargo.toml --features host-cli
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/presentation/Cargo.toml --features host-cli --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path sdk/examples/presentation/Cargo.toml -- --check
cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/presentation/Cargo.toml --target x86_64-unknown-none
python3 -B sdk/examples/presentation/tests/interop.py target/platform-presentation-host/debug/slide-preview build/platform-presentation-evidence
```

Zeven Rusttests, waaronder twee werkelijke processen, controleren import,
navigatie, bronbehoud, twee echte diaframes op beide schalen, limieten,
glyphfallback en pixelbuffers die bij fouten onveranderd blijven.
Het zelfstandige Python-PPM-orakel controleert vier echte uitvoerframes,
RGB/header/pixelregio's en genereert uitsluitend buiten Git PNG-bewijsbeelden.
Dat PNG-hulppad is geen productencoder en valideert niet zelfstandig de
volledige typografie. Agentinspectie van die beelden is geen menselijke test.
Geen native executable, GUI-bediening, saveportaal, presentatie-editor,
QEMU, fysieke, menselijke of releaseacceptatie.
