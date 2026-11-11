# Host diagnostics only

This standalone helper deliberately uses the **unmodified registry version**
zune-jpeg0.5.16-rc3. The native decode component uses the verified local
vendor correction. This keeps the original YCbCr counterexample reproducible.
The helper is never linked into the OS or image-probe and is not native evidence.

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-image-diagnostics-target \
  python3 -B tools/dev.py cargo +1.90.0 run --locked --offline \
  --manifest-path services/image-codec/diagnostics/Cargo.toml -- \
  services/image-codec/fixtures/progressive.jpg /tmp/image-ycbcr.raw ycbcr
```

Replace `ycbcr` with `rgba` for real engine RGBA output or `planes` for the
real post-IDCT component planes. `planes` writes output prefix`.0`,`.1`,`.2`
and reports width/height/stride. Only the named synthetic fixture is
used; Pillow serves exclusively as an independent host oracle.

The causal comparison uses libjpeg-turbo3.1.4.1, as identified
by `PIL.features.version_feature('libjpeg_turbo')`. `version_codec('jpg')`
reports6.2 as the compatibility version, not the real turbo version. For YCbCr the
oracle uses `Image.open(...).draft('YCbCr', original_size)` and no RGB→YCbCr
round trip. The full production oracle tests remain in `tests/decode.rs`.
