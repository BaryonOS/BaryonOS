# TASK-PLATFORM-M7-WAV-CLIP

Basis `6d523964ab85638d4dd981bcdb19580d18646fab` (gefetched main).
Branch `platform/wav-clip-20261103`, geteste code/test/READMEbron
`8540828f2f1e07f3d020bc68a9c08b7f8250f6f3`.
Vervolgcommit voegt alleen deze receipt toe.
Scope: nieuwe `sdk/examples/audio-clip/**` plus eigen bewijs;
geen oude Platform-consumers, General-, driver-, kernel-, audio-, workspace-,
native ABI-, rechten- of centrale status/scorewrites.

## Resultaat en profiel

Werkelijke fileconsumer: RIFF/WAVE PCM16 mono/stereo -> expliciet
framebereik -> nieuw WAV met exact dezelfde geselecteerde samplebytes,
kanaalvolgorde en samplefrequentie. Geen decoderframework/afspeelmock:
ongecomprimeerde PCM heeft hier geen codecengine nodig.
Bestandsbewerking is onafhankelijk van ontbrekende native playback.
Geen nieuw device-/netwerk-/bestandsrecht; no_std-API ontvangt callerbytes.

Profiel: little-endian RIFF/WAVE, PCM-tag 1, 16 bits, 1/2 kanalen,
8000..192000 Hz, fmt 16 bytes of 18 bytes met cbSize 0,
exact een fmt/data in willekeurige volgorde. Maximaal 16 MiB/1024 chunks;
volledig bestand inclusief tail/padding gevalideerd. Geen RF64/RIFX,
WAVE64, PCM8/24/32, float/extensible/ADPCM/MP3/FLAC-claim.
Onbekende/extra chunks vereisen expliciete `--drop-metadata`, omdat
cue-/loopposities na uitsnijden fout kunnen zijn. Zonder keuze geen
succesoutput. Oorspronkelijke input blijft intact; geen stille clamp,
resampling, gain of fade. Harde knip kan klikken.

Geen unsafe of dependency. Primaire bronnen Microsoft,
*Resource Interchange File Format (RIFF)*, 31 mei 2018,
<https://learn.microsoft.com/en-us/windows/win32/xaudio2/resource-interchange-file-format--riff->;
*WAVEFORMATEX*, 5 december 2018,
<https://learn.microsoft.com/en-us/windows/win32/api/mmreg/ns-mmreg-waveformatex>.
Geraadpleegd 3 november 2026; profiel/budget/privacygrens in de subtree-README.

## Exacte uitgevoerde controles

Werkdirectory
`/home/simon/.copilot/session-state/edf11e7c8b3e4c9ba1a91b7ef0779695/files/platform-wav`.
Rust 1.90.0, bestaande `/tmp/nieuw-os-c-toolchain/cc-wrapper` en sysroot;
`CARGO_TARGET_DIR="$PWD/target/platform-wav-host"`. Geen installatie/download.

| Exact commando | Resultaat |
|---|---|
| `cargo +1.90.0 generate-lockfile --offline --manifest-path sdk/examples/audio-clip/Cargo.toml` | exit 0; nieuwe dependencyvrije workspace |
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/audio-clip/Cargo.toml --features host-cli` | exit 0; 6 geregistreerde tests, 0,319 s |
| `cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/audio-clip/Cargo.toml --features host-cli --all-targets -- -D warnings` | exit 0, 0,125 s |
| `cargo +1.90.0 fmt --manifest-path sdk/examples/audio-clip/Cargo.toml -- --check` | exit 0, 0,043 s |
| `cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/audio-clip/Cargo.toml --target x86_64-unknown-none` | exit 0, 0,136 s; bibliotheek, geen uitvoering |
| `python3 -B sdk/examples/audio-clip/tests/interop.py target/platform-wav-host/debug/wav-clip build/platform-wav-evidence/fragment.wav` | exit 0, 0,075 s; 48 echte clipoutputs onafhankelijk met Python wave |
| `file build/platform-wav-evidence/fragment.wav` | exit 0; RIFF/WAVE Microsoft PCM, 16 bit, stereo 192000 Hz |
| `git diff --check` | exit 0 |

De eerste testreeks slaagde met vijf tests, maar Clippy faalde exit 101
op de bewust omgekeerde letterlijke Range in een negatieve test.
Dezelfde grens is daarna via start/end-variabelen getest zonder lintallow.
Een aanvullende volledige-tail-/paddingtest is toegevoegd; de finale keten
voert alle zes tests uit en slaagt. Geen productpolicy of foutacceptatie verruimd.

Testdekking: signed sample-extremen/alle kanaalbytes, begin/einde/volledige
clip, inputbehoud, data-first/fmt18, odd metadata, expliciete discard,
exact budget 47/48, omgekeerd/leeg/overflowbereik, lege data, alle
bestandstruncaties, forged chunklengtes, ontbrekende padding en malformed
tail na geldige audio, dubbele/missende fmt/data, incomplete frames,
unsupported compressed/float/multichannel, inconsistent byterate/blockalign,
16 MiB+1 en 1024/1025 chunks.
Python maakt bronbestanden met de onafhankelijke wave-writer, voert echte
CLI-processen uit en leest de output opnieuw met wave:
mono/stereo, 8/44,1/48/192 kHz, vijf framebereiken en expliciete metadataflow.
Ongeldige CLI/bron/overflow geeft geen successtdout.

Logs/artifacts buiten Git onder `build/platform-wav-evidence/`, SHA-256:

```text
f55ecf2806e4f48853cc1a7f7a012fc1a5df75787d32c88c190a364aee24518c gates.log (Clippy-fout, exit 101)
ca81eab0ca9758e17f87a7e63a7609a0c60f22ec102800d4bbad26c86263feba accepted-gates.log (finale volledige keten, exit 0)
d134ca1832285abb2ea884207e1e4bc44bf0d0d35134305b8781b5e66df09e14 fragment.wav
d442b12eb469edc97a3a5e875dc8aa0fa050d4a626622b5a9d8271c7488bcc29 target/platform-wav-host/debug/wav-clip
6dc815a1a50fc89b4f3710d52c373df040d3234d1e7b791ed804b520b51d6351 target/platform-wav-host/x86_64-unknown-none/release/libnieuw_os_audio_clip.rlib
cf6cb6b569f6674d161b34d5e3ad5bd3a42a3a4c0fade9fdc7da19a4036c2513 src/lib.rs
b7aab7568889fd6261a33af4b3c70e879390e21264d550518a166a5377908ed5 src/main.rs
820a8a50eebee6ce6869152e96ed84da6c7acd0fd122ad439758288b9a646e30 tests/clip.rs
0d43c36939d83e39ef202c511cf5c9c68e28f7760fb044ab68a183d5a346c0c8 tests/interop.py
```

## Queue en resterende grenzen

**QUEUED-FOR-INTEGRATION**, geen nieuwe PR zonder OPEN-PR-LEASE.
Eventuele app-autocreatie afzonderlijk controleren/melden; geen duplicaat.
Nieuwe functionality en eigen bewijs, geen herhaling van oude slices.

Hostfunctionaliteit plus no_std-compilatie. Native app/venster/saveportaal,
afspelen/hoorbare uitvoer, audiodevice, native hardheapquota/OOM-herstel,
QEMU, fysieke hardware, menselijke tests, onafhankelijke audit en
releaseacceptatie **niet uitgevoerd/niet bewezen**. Playback blijft
afhankelijk van bestaande native app-/audiobroker-/driverowners.
Stdoutwrite is niet atomair; geen storage-durabilityclaim.
FR-A06/minimale WAV-dekking blijft als native productcriterium open.
Scoreadvies **41 -> 41/100**, centrale beoordeling/status bij General.
Rollback: verwijder uitsluitend nieuwe audio-clip-subtree en receipt;
geen bestaande ABI, audioketen of opslagformaat verandert.
