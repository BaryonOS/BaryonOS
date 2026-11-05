# PNG/JPEG naar BMP-export

TASK-PLATFORM-M7-IMAGE-EXPORT, basis `d10b5cda`. Een concrete host-only
bestandsconsumer voor de bestaande beeldcodec. Geen native exportdialoog,
nieuwe decoder, extra renderengine, appinstaller of resourcebroker.

```sh
cargo +1.90.0 run --offline --locked --manifest-path sdk/examples/image-export/Cargo.toml --features host-cli --bin image-bmp -- --background ffffff < foto.png > foto.bmp
```

De achtergrond is **verplicht** en bestaat uit zes hexadecimale RGB-cijfers.
Zo wordt transparantie niet stil tegen een verborgen standaardkleur afgevlakt.
Geen stdout tot alle decode-/exportcontroles slagen; fouten: exit 1 en stderr
zonder beeldinhoud. Een fout tijdens de eigenlijke stdoutwrite kan gedeeltelijke
output achterlaten. Geen atomair opslaan; gebruik bij shellredirectie nooit
dezelfde invoer-/uitvoerbestandsnaam.

## Profiel en verliesgrens

Ondersteunde PNG/JPEG-invoer volgt uitsluitend de bestaande
`services/image-codec`-profielen en dependencyledger. De bestaande
`Image::rgba8_into` past JPEG-EXIF-orientatie toe en rondt RGBA16 naar RGBA8 af.
Daarna wordt alpha in de bestaande samplewaarden tegen de gekozen kleur
gemengd. Geen ICC-/gamma-/lineairlichttransformatie of fotogetrouwheidsclaim.
Het resultaat is een afzonderlijke **ongecomprimeerde RGB24 BMP**:
BGR-bytes, positieve hoogte/bottom-up, 40-byte DIB, 54-byte pixeloffset,
vierbyte-rijuitlijning met nulpadding, geen palet of kleurprofiel.
EXIF-orientatie zit in de pixels; oorspronkelijke EXIF/ICC/metadata worden
niet in de afgeleide BMP gekopieerd. Originele invoer en ruwe pixels blijven
ongewijzigd. Dit is geen verliesvrije metadata-/precisieroundtrip.

Primaire formaatbronnen: Microsoft,
*[BITMAPFILEHEADER](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapfileheader)*
(documentversie 5 december 2018) en
*[BITMAPINFOHEADER](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapinfoheader)*
(26 april 2023), geraadpleegd 3 november 2026.
Alleen het standaardbestandsformaat wordt gebruikt; geen Windows-API/-kernel.

## Resource- en authoritygrens

Invoer maximaal 4 MiB, 4 megapixels (4 * 1024 * 1024), aslengte 16384,
uitvoer maximaal 16 MiB. Een caller kan het uitvoerbudget verlagen.
Een tijdelijke georienteerde RGBA8-buffer en de BMP-buffer zijn elk begrensd
en fallible gereserveerd; maximaal circa 32 MiB samen, naast decoder/raw pixels.
De codecwerkgrens is toelating, geen harde heapquota. Upstream kan infallible
alloceren; native gebruik vereist een afzonderlijk begrensd decoderproces.
Deze hostconsumer bewijst geen native OOM-/crashherstel.

Geen nieuwe registrydependency of unsafe. Het nieuwe lockfile gebruikt de
bestaande exact gepinde codecclosure en JPEG-vendorpatch. Geen wijzigingen aan
codec, viewer, gedeelde workspace, native IPC, autorisatie, opslag of netwerk.
De host-CLI erft stdio; een native export moet via het bestaande saveportaal en
een daadwerkelijk geautoriseerde volumeclient lopen.

## Acceptatie

```sh
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/image-export/Cargo.toml --features host-cli
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/image-export/Cargo.toml --features host-cli --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path sdk/examples/image-export/Cargo.toml -- --check
cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/image-export/Cargo.toml --target x86_64-unknown-none
```

Tests controleren de daadwerkelijke uitvoerheaders, onderste-bovenrijvolgorde,
BGR, alle paddinglengtes, exacte alpha- en RGBA16-waarden, EXIF-rotatie,
echte PNG/JPEG-fixtures en CLI-processen. Malformed samples, afmetingen,
background, ontbrekende keuze en te klein uitvoerbudget geven expliciete fouten.
Geen native executable, GUI, opgeslagen Nieuw-OS-object, hardware- of
releaseacceptatie; een no_std-build is uitsluitend targetcompilatie.
