# PCM-WAV-fragment uitsnijden

TASK-PLATFORM-M7-WAV-CLIP, basis `6d523964`. Concrete bestandsbewerking:
een echte ondersteunde WAV importeren, een framebereik uitsnijden, als
zelfstandig WAV-bestand exporteren. Geen afspeelmock, audiobroker,
generieke codecengine of duplicatie van een bestaande driver.
Het model is `no_std+alloc`; de CLI is uitsluitend een hostprogramma.

```sh
cargo +1.90.0 run --offline --locked --manifest-path sdk/examples/audio-clip/Cargo.toml --features host-cli --bin wav-clip -- --start-frame 48000 --frames 96000 < opname.wav > fragment.wav
```

Een frame bevat **alle kanalen op een tijdstip**. Bij 48 kHz kiest dit
voorbeeld vanaf seconde 1 de volgende 2 seconden. De selectie is exact,
niet afgerond naar tijdstempels of stil ingekort tot het bestandseinde.
Originele PCM16-samplebytes, kanaalvolgorde en samplefrequentie blijven
gelijk. Geen conversie, resampling, gain, fade of dithering; een harde knip
kan hoorbaar klikken. Originele invoer blijft ongewijzigd.

## Expliciet ondersteund profiel

- RIFF little-endian `WAVE`, PCM-tag 1, signed 16-bit samples,
  mono of stereo, samplefrequentie 8000 t/m 192000 Hz.
- Exact een `fmt `-chunk van 16 bytes, of 18 bytes met cbSize 0,
  en exact een `data`-chunk. Beide chunkvolgordes worden geaccepteerd.
  Byte rate, block alignment en complete frames worden gecontroleerd.
- Maximaal 16 MiB volledig bestand en 1024 chunks. RIFF-lengte moet exact
  het bestand dekken; geen verborgen/trailing bytes of gedeeltelijke header.
  Alle chunks inclusief padding en metadata na de audio worden gevalideerd.
- Geen RIFX/RF64/WAVE64, PCM8/24/32, float, extensible, multichannel,
  ADPCM, MP3/FLAC of andere compressie. Geen volledige WAV-dekking geclaimd.

Extra chunks (ook JUNK/LIST/cue/loopmetadata) worden geteld, niet uitgevoerd.
Als er extra chunks zijn, vereist uitsnijden de expliciete keuze
`--drop-metadata`. Zonder die keuze volgt `MetadataChoiceRequired`, geen
succesoutput. Marker-/loopposities kunnen na uitsnijden ongeldig zijn;
de exporter beweert die daarom niet te behouden of opnieuw te berekenen.
`--drop-metadata` schrijft alleen de canonieke fmt/data-chunks en verwijdert
alle extra metadata bewust. Een clip zonder extra metadata heeft geen
herhaalde toestemming nodig.

## Grenzen en fouten

`Wave::parse` leent de oorspronkelijke samples zonder allocatie.
`clip(start..end, drop_metadata, output_limit)` gebruikt een halfopen,
niet-leeg framebereik en reserveert de exacte output fallible.
De uitvoerlimiet is maximaal 16 MiB, door caller te verlagen.
CLI-invoer maximaal 16 MiB+1 detectiebyte, uitvoer maximaal 16 MiB;
geen native harde heapquota/OOM-herstelclaim.

Geen unsafe of dependency. Nieuwe subtree `sdk/examples/audio-clip/**` plus
eigen receipt, geen bestaande SDK-workspace-, UI-, driver-, kernel-, audio-,
General-, launch-, storage-, rechten- of scorewijzigingen.
Een native edit/saveflow vereist de bestaande portal-/appowners;
sample-exacte bestandsbewerking zelf vereist geen native audiodevice.
De host-CLI erft stdio en verleent geen Nieuw-OS-resourcecapability.

Alle parse/selectie/exportfouten geven exit 1 en stderr zonder sampleinhoud.
Geen stdout tot de volledige operatie slaagt. Stdoutwrite is niet atomair;
een writefout kan gedeeltelijke output achterlaten. Gebruik nooit hetzelfde
invoer-/uitvoerpad bij shellredirectie.

## Primaire bronnen en acceptatie

Microsoft, *Resource Interchange File Format (RIFF)*, 31 mei 2018,
<https://learn.microsoft.com/en-us/windows/win32/xaudio2/resource-interchange-file-format--riff->;
*WAVEFORMATEX*, 5 december 2018,
<https://learn.microsoft.com/en-us/windows/win32/api/mmreg/ns-mmreg-waveformatex>.
Geraadpleegd 3 november 2026: RIFF-lengtes, WORD-padding, fmt/data,
samplerate, block alignment en byte rate. Geen Windows-runtime of -kernel.

```sh
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/audio-clip/Cargo.toml --features host-cli
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/audio-clip/Cargo.toml --features host-cli --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path sdk/examples/audio-clip/Cargo.toml -- --check
cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/audio-clip/Cargo.toml --target x86_64-unknown-none
python3 -B sdk/examples/audio-clip/tests/interop.py target/platform-wav-host/debug/wav-clip build/platform-wav-evidence/fragment.wav
file build/platform-wav-evidence/fragment.wav
```

Zes Rusttests en 48 echte CLI-clipoutputs worden getoetst, met onafhankelijk
Python `wave` als inputwriter/outputreader en exacte samplevergelijking.
Vier samplefrequenties, mono/stereo, begin/einde/volledige clip en expliciete
metadataweigering/-discard. Negatieven dekken truncaties, overflow/chunk-
lengtes, ontbrekende padding, dubbele/missende chunks, incomplete frames,
unsupported formaten, inconsistent fmt, selectie/CLI en budgetten.
Geen native executable, GUI, daadwerkelijk hoorbare uitvoer, audiodevice,
QEMU, hardware-, menselijke, onafhankelijke audit- of releaseacceptatie.
