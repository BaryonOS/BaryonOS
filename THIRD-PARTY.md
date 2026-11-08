# Externe onderdelen en hun licenties

Externe code en lettertypen behouden hun eigen licentievoorwaarden. De
MPL-toepassing op eigen software wijzigt die voorwaarden niet.
[LICENSING.md](LICENSING.md) en [MPL-2.0-FILES.txt](MPL-2.0-FILES.txt)
bakenen de projectlicentie af; dit register verleent geen extra rechten
op extern materiaal.

## Meegeleverde code en lettertypen

| Onderdeel | Voorwaarden en bestaand bewijs |
|---|---|
| Gepatchte zune-jpeg 0.5.16-rc3 | MIT, Apache-2.0 of Zlib; volledige upstreamteksten bij `services/image-codec/vendor/zune-jpeg-0.5.16-rc3/`; wijzigingen en herkomst in [image-codec/THIRD-PARTY.md](services/image-codec/THIRD-PARTY.md) |
| Gepatchte hkdf 0.13.0 en hmac 0.13.0 | MIT of Apache-2.0; volledige upstreamteksten bij de twee directories onder `services/keyring/vendor/`; wijzigingen en herkomst in [vendor/README.md](services/keyring/vendor/README.md) |
| Open Sans-atlas `ui/graphics/assets/open-sans-atlas.rle` en gegenereerde glyphmetadata `ui/graphics/src/atlas.rs` | Apache-2.0; beide blijven buiten de MPL-bestandslijst. [graphics/THIRD-PARTY.md](ui/graphics/THIRD-PARTY.md), oorspronkelijke herkomst/hashes in [font-provenance.json](ui/graphics/assets/font-provenance.json) en volledige licentietekst onder `ui/graphics/licenses/` |
| Noto Sans Mono-bitmapgegevens | OFL-1.1 voor de fontdata, MIT voor de Rust-cratecode; volledige notices in [native/THIRD-PARTY.md](ui/native/THIRD-PARTY.md) en [compositor/THIRD-PARTY.md](sdk/examples/compositor/THIRD-PARTY.md) en hun `licenses/`-directories |
| SplitMix64-algoritme | Publiek-domeinverklaring van Sebastiano Vigna; bronvermelding in `experiments/quantum/qsim-core/src/sampler.rs` en [quantum/THIRD-PARTY.md](experiments/quantum/THIRD-PARTY.md) |
| Kalenderberekening `days_from_civil` | Publiek-domeinverklaring van Howard Hinnant; bronvermelding in `drivers/rtc/src/lib.rs` |

## Geregistreerde dependencies

De afzonderlijke componentregisters beschrijven dependencies en de
geselecteerde noticevoorwaarden:

- [quantum](experiments/quantum/THIRD-PARTY.md)
- [heap-allocator](sdk/heap-allocator/THIRD-PARTY.md)
- [document-import](sdk/examples/document-import/THIRD-PARTY.md)
- [image-crop](sdk/examples/image-crop/THIRD-PARTY.md)
- [image-codec](services/image-codec/THIRD-PARTY.md)
- [net-stack](services/net-stack/THIRD-PARTY.md)
- [net-tls](services/net-tls/README.md)
- [r-efi](docs/third-party/README.md)

Een dependencyvermelding in `Cargo.toml` of `Cargo.lock` betekent niet dat
de externe broncode of iedere optionele dependency in deze repository of
in iedere binary wordt meegeleverd. De distributie van een binary vereist
de toepasselijke notices van de daadwerkelijk opgenomen componenten.

Let bij de dependency-inventaris in het bijzonder op de combinaties
**Apache-2.0 AND ISC** voor `ring` en **(MIT OR Apache-2.0) AND Unicode-3.0**
voor `unicode-ident`. De aanvullende fontvoorwaarden van Noto Sans Mono
worden niet volledig weergegeven door het MIT-label van de Rust-crate.

## Firmware en distributiepakketten

De bronrepository is geen generieke licentieverlening voor externe
QEMU-, OVMF-, SeaBIOS- of hostpakketten. Een daadwerkelijk samengesteld
runner- of releasepakket houdt de vereiste firmwarelicenties en
componentnotices bij volgens
[tools/windows-qemu/README.md](tools/windows-qemu/README.md).
