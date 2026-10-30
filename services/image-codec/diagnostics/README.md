# Alleen hostdiagnose

Deze zelfstandige helper gebruikt bewust de **ongewijzigde registryversie**
zune-jpeg0.5.16-rc3. De native decodecomponent gebruikt de geverifieerde lokale
vendorcorrectie. Zo blijft het oorspronkelijke YCbCr-tegenvoorbeeld reproduceerbaar.
De helper wordt nooit in het OS of image-probe gelinkt en is geen native bewijs.

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-image-diagnostics-target \
  python3 -B tools/dev.py cargo +1.90.0 run --locked --offline \
  --manifest-path services/image-codec/diagnostics/Cargo.toml -- \
  services/image-codec/fixtures/progressive.jpg /tmp/image-ycbcr.raw ycbcr
```

Vervang `ycbcr` door `rgba` voor echte engine-RGBA-output of `planes` voor de
echte post-IDCT-componentplanes. `planes` schrijft outputprefix`.0`,`.1`,`.2`
en meldt breedte/hoogte/stride. Alleen de benoemde synthetische fixture wordt
gebruikt; Pillow fungeert uitsluitend als onafhankelijk hostorakel.

De causale vergelijking gebruikt libjpeg-turbo3.1.4.1, zoals geïdentificeerd
door `PIL.features.version_feature('libjpeg_turbo')`. `version_codec('jpg')`
meldt6.2 als compatibilityversion, niet de echte turboversie. Voor YCbCr gebruikt
het orakel `Image.open(...).draft('YCbCr', original_size)` en geen RGB→YCbCr-
roundtrip. De volledige productie-orakelproeven blijven in `tests/decode.rs`.
