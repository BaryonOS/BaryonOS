# NOSVOL02 toelating van vreemde volumes

## Integratorvervolg op a5b14596

De slot1-review heeft een aanvullende echte fout gevonden: niet-nuldata in
alleen de tweede tabelkopie werd zonder marker als lege opslag toegelaten.
Rust en het onafhankelijke Python-orakel eisen nu nulbytes in de volledige
ongemarkeerde volume-invoer. De22 service- en25 orakeltests zijn echt groen,
inclusief alle68 metadatasectoren en gemarkeerd herstel; exacte rood/groenlogs:
[nieuwe receipt](../pr148-slot1-review.json).

De eerste-markercrash is daarna apart getest: 516 beelden en 11.699 beelden
in de onverkorte documentcrashmatrix groen; 26 Python-tests groen.
<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:c4ea8517e8f409aa7689c5b5312311cc41893fd13ff5578bf4a242f917ed2fa9 -->

[Primaire receipt](../pr148-marker-crash-review.json). De `volume-core` host-suite
is inmiddels ook groen op deze branch:
[host-suite receipt](../pr148-volume-core-host-suite.json).

<!-- /coderabbit-ai-generated:v1 -->

Alleen de native
metadata-only/QEMU-regressie blijft open. Dit vervolg maakt de
oorspronkelijke niet-uitgevoerde tests hieronder niet retrospectief groen.

## Historisch implementatiecheckpoint

Implementatiecheckpoint op basis `cbbc9c9941b08ea5edfbd3637d494635b4fb1df0`,
branch `simonvanieperen-upgraded-disco`. De lege-volumecontrole en onafhankelijke
hostorakeltests zijn bijgewerkt. De native foreign-volume-proef is **niet
uitgevoerd**: deze WSL-omgeving heeft geen `qemu-system-x86_64`, OVMF of
`cc`; `sudo`-installatie is geblokkeerd door ontbrekende interactieve
authenticatie. Dit rapport is dus geen native bewijs.

## Uitgevoerd

| Exact commando | Exit | Resultaat |
|---|---:|---|
| `$HOME/.cargo/bin/cargo +1.90.0 fmt --all -- --check` | 0 | Rust-opmaak consistent |
| `$HOME/.cargo/bin/cargo +1.90.0 check --locked -p volume-core --all-targets` | 0 | Productiecode en hosttestcode typechecken; tests niet uitgevoerd |
| `$HOME/.cargo/bin/cargo +1.90.0 clippy --locked -p volume-core --all-targets -- -D warnings` | 0 | Geen Clippy-waarschuwingen |
| `python3 -B -m unittest discover -s tests -p 'test_volume_*.py'` | 0 | 23 hosttests van het onafhankelijke schijforakel |
| `$HOME/.cargo/bin/cargo +1.90.0 test --locked -p volume-core` | 101 | Hosttests niet gelinkt: linker `cc` ontbreekt |
| `CARGO_TARGET_DIR=/tmp/nieuw-os-foreign-volume-native-final-20261103 PATH="$HOME/.cargo/bin:$PATH" python3 -B sdk/examples/volume-docs/build.py` | 0 | Native freestanding volume-app en volume-service gebouwd; geen kernel of image |
| `python3 -B tools/preflight.py --strict` | 2 | QEMU, OVMF en `cc` ontbreken in PATH |
| `NOS_BUILD_DIR=/tmp/nieuw-os-foreign-volume-build-20261103 NOS_TARGET_DIR=/tmp/nieuw-os-foreign-volume-target-20261103 PATH="$HOME/.cargo/bin:$PATH" python3 -B tools/build.py --kernel-feature test-volume-foreign` | 1 | Volledige featurebuild bereikte kernel-buildscript; linker `cc` ontbreekt; geen kernel/UEFI image |

Volledige uitvoer staat buiten Git. SHA-256 per log en native app/service-binary:

```text
fmt-final.log                 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
check-final.log               6d77f64dfc8c5325b90fdbe05d8be367bdd578bcccec24f275c5ffee7d03dd6e
clippy-final.log              02db5a66fa0c60ca8618bd08aac556eac4f3ec863e8630a0d353dba6a0f6e126
python-final.log               100b74df08b009685a01c2654d8523a753904ad9e6dae448c764e4bf8347035b
cargo-test-final.log           d97a5f67a00c3e168537315b876560d9fbc00650230f47a10fb67ac8ce46b3c2
volume-native-final.log        94917ee1aef6282ee23483d764b3609075f540d361061339136415f4cb76f781
kernel-build-final.log         ab13be145cc80ee9a0ad5f623960a60aaa936142e5907f17c2c2cb9db9ff39b2
preflight-final.log            54e5cdd7251147f1075aec189573939cd123d2aa67869a22dbb5b04ab57cddba
volume-service                 5b5e1f821932d435796c58918189344819942de3a5c538c183bbe89052296543
volume-app                     05deb058e9fec9dc0647742c4e8c11827be55909588f85e55fa8af2b82cdbbb8
```

De logbestanden en binaries staan onder `/tmp/nieuw-os-foreign-`; de
volledige featurebuild bewaart zijn detailuitvoer onder
`/tmp/nieuw-os-foreign-volume-build-20261103/logs/`.

## Implementatie en bewijsgrens

Als beide NOSVOL02-tabelkopieën ontbreken, controleert herstel nu ook de
volledige experimentele gegevensregio. Zonder geldige `NOSFMT02`-marker wordt
alleen volledig nulgevulde opslag als leeg geaccepteerd. De eerste mutationele
upload of metadata-commit schrijft en flusht de marker vóór de eerste
inhouds-/tabelwrite; zo blijft een onderbroken eerste upload als lege logische
toestand herstelbaar. De onafhankelijke Python-orakeltest gebruikt een
synthetische Btrfs-achtige magicwaarde op de 64-KiB-offset. Dat is geen geldig
Btrfs-volume. Een direct geconstrueerde service valideert vóór haar eerste
mutatie ook een eventueel meegegeven niet-nul-generatie tegen de werkelijke
hersteltoestand; alleen succesvolle `Service::recover` slaat die controle over.

De native volume-app/service zijn alleen gecompileerd. Native kernel/QEMU-
uitvoering is nog te doen op uitsluitend een wegwerpimage; QEMU en OVMF
ontbreken in deze omgeving:

```sh
python3 -B tools/build.py --kernel-feature test-volume-foreign
python3 -B tools/run_volume.py --case foreign
```

Verwacht: de eigen kernel en volume-service weigeren het onherkende,
niet-nulgevulde volume vóór servicebeschikbaarheid met `Corrupt`; de image
blijft bytegelijk en er zijn geen schrijf-, flush- of app-reply-operaties.
Tot uitvoering blijven de Rust-hosttests en de volledige `tools/test_volume.py`
regressiegate open. Dit bewijst geen detectie/import van alle bestandssystemen,
geen fysieke mediumveiligheid en geen eigen filesystemdriver.
