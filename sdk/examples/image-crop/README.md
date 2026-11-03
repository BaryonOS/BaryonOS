# PNG/JPEG uitsnijden naar PNG

Werkelijke bestandsbewerking: bestaande decoder -> displaygeoriënteerde
RGBA8-pixels -> rechthoek -> onderhouden PNG-encoder.
Cropcoördinaten gelden **na** bestaande JPEG EXIF-oriëntatie, niet voor
de ruwe samplevolgorde. Alpha, inclusief RGB onder volledig transparante
pixels, blijft byte-exact. Geen clamp, resampling, matting of bronmutatie.
RGBA16 wordt geweigerd in plaats van stil naar acht bits af te ronden.

```sh
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/image-crop/Cargo.toml --features host-cli
cargo +1.90.0 run --offline --locked --manifest-path sdk/examples/image-crop/Cargo.toml --features host-cli --bin image-crop -- --x 10 --y 20 --width 100 --height 80 --drop-metadata --unmanaged-colour < bron.jpg > uitsnede.png
```

De twee verplichte keuzes bevestigen verlies van metadata en ontbreken
van ICC/gamma-kleurbeheer. De uitvoer bewaart geselecteerde **samples**,
niet de exacte kleurweergave op een bepaald scherm. Geen sRGB-profiel of
gamma verzonnen. EXIF, auteurs-/GPS-/tekstmetadata en ICC blijven niet in
de uitvoer; orientation is in de pixelvolgorde verwerkt en wordt niet
opnieuw als metadata meegegeven. De bron blijft intact.
Uitvoer is statische PNG RGBA8 zonder ancillary chunks, geen APNG/JPEG-
encoding of volledig PNG/JPEG-profielclaim. Ondersteunde invoer volgt
de bestaande codec; onder meer CMYK/animatie/unsupported precisie faalt.

`image-crop` leest uitsluitend stdin en schrijft stdout/stderr. Geen
bestandsopeningen, netwerkrechten of metadatarelay. Alle import/crop/PNG-
encoding voltooit vóór stdout; fouten geven exit 1 en uitleg. Stdoutwrite
kan gedeeltelijk mislukken; geen atomaire save-/durabilityclaim.
Onbekende/dubbele argumenten, overflow, negatieven, lege/buitenbeeld-
rechthoeken en ontbrekende expliciete keuzes worden geweigerd.

## Architectuur en grenzen

Nieuwe `sdk/examples/image-crop/**` op mainbasis
`681f1137e411d6c6fe536bd2bf499c27b34832be`.
Geen writes aan overgedragen BMP/EXIF/viewer of General/Veld/K05.
No_std-kern hergebruikt `Image::rgba8_into`; geen gekopieerde EXIFmapping.
Caller ontvangt eigen cropbuffer, bezit daarnaast de oorspronkelijke Image.
Extra geheugenbudget omvat de volledige georiënteerde tijdelijke buffer
en cropresultaat. Gemalformeerde caller-Image geeft codec-fout.

Host-CLI: maximaal 8 MiB input, 4096 per dimensie, 1048576 pixels,
128 MiB decoderadmission-work, 256 KiB metadata, 1024 PNG-chunks,
128 JPEG-scans, 8 MiB extra cropbudget en 8 MiB encoded-outputbudget.
Limieten zijn geen native harde heap-/CPUquota. Upstreamallocaties
blijven infallible; native decodergebruik vereist afzonderlijke
begrensde arena/proces volgens het bestaande codeccontract.

`png = "=0.18.1"` is optioneel achter `host-cli`. De daadwerkelijke
PNG-encoding is **host/std**, niet naar Nieuw OS geport.
Default no_std-build bevat geen png/crc32fast/flate2. Herkomst/pins/
licenties/buildscriptuitzondering staan in THIRD-PARTY.md en Cargo.lock.
Geen nieuwe XML/PNG-/compressie-engine of algemene graphicsfoundation.

Primaire formaatbron: W3C PNG Working Group,
*Portable Network Graphics (PNG) Specification, Third Edition*,
Recommendation 24 juni 2025,
<https://www.w3.org/TR/2025/REC-png-3-20250624/>.
Onderhouden encoder-API:
<https://docs.rs/png/0.18.1/png/struct.Encoder.html>.

## Acceptatie en bewijsgrens

Gerichte Rusttests plus echte CLI-input/output. Onafhankelijk Python-
zlib/PNG-orakel controleert chunks/CRC/filters/dimensies/alle RGBA-bytes,
inclusief alpha. Bestaande JPEG-referenties zijn afzonderlijk met
Pillow/libjpeg gegenereerd: maximaal drie niveaus sampleverschil;
crop en alle acht oriëntaties zijn vervolgens byte-exact tegen de
volledige gedecondeerde samplebuffer getoetst. Geen fixturegeneratie
of oude consumerhertests. No_std-kerncompile, geen native PNG-encoder.

Geen native window/open/save/launch, VM, fysiek apparaat, menselijke
gebruikertest, onafhankelijke audit of releasevalidatie geclaimd.
Eigen receipt bevat uitgevoerde commands en hashes.
