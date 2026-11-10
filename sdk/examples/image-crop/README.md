# Crop PNG/JPEG to PNG

Actual file processing: existing decoder → display-oriented RGBA8 pixels →
rectangle → maintained PNG encoder. Crop coordinates apply **after** existing
JPEG EXIF orientation, not to raw sample order. Alpha, including RGB beneath
fully transparent pixels, remains byte-exact. No clamping, resampling,
matting or source mutation. RGBA16 is rejected rather than silently reduced
to eight bits.

```sh
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/image-crop/Cargo.toml --features host-cli
cargo +1.90.0 run --offline --locked --manifest-path sdk/examples/image-crop/Cargo.toml --features host-cli --bin image-crop -- --x 10 --y 20 --width 100 --height 80 --drop-metadata --unmanaged-colour < bron.jpg > uitsnede.png
```

The two mandatory choices acknowledge metadata loss and absence of ICC/gamma
colour management. Output preserves selected **samples**, not exact colour
appearance on a particular display. No sRGB profile or gamma is invented.
EXIF, author/GPS/text metadata and ICC are not retained in output; orientation
is incorporated in pixel order and not added again as metadata. The source
remains intact. Output is static PNG RGBA8 without ancillary chunks, not
APNG/JPEG encoding or a full PNG/JPEG-profile claim. Supported input follows
the existing codec; CMYK, animation and unsupported precision, among others,
fail.

`image-crop` reads source content through stdin and writes stdout/stderr.
Its CLI opens no files or network connections and transmits no metadata.
This does **not** automatically remove the host process's disk/network rights;
ordinary host execution provides no sandbox or permission restriction here.
Restricting host rights requires a separately enforced process boundary.
All import/crop/PNG encoding completes before stdout; errors return exit 1
and an explanation. Stdout writes may fail partially; no atomic save or
durability claim. Unknown/duplicate arguments, overflow, negatives, empty/
out-of-image rectangles and missing explicit choices are rejected.

## Architecture and limits

New `sdk/examples/image-crop/**` on main baseline
`681f1137e411d6c6fe536bd2bf499c27b34832be`.
No writes to handed-over BMP/EXIF/viewer or General/Veld/K05 work.
The no_std core reuses `Image::rgba8_into`; no copied EXIF mapping.
The caller receives an owned crop buffer and also owns the original Image.
The extra memory budget includes the complete oriented temporary buffer
and crop result. A malformed caller Image produces a codec error.

Host CLI: at most 8 MiB input, 4096 per dimension, 1048576 pixels,
128 MiB decoder-admission work, 256 KiB metadata, 1024 PNG chunks,
128 JPEG scans, 8 MiB extra crop budget and 8 MiB encoded-output budget.
Limits are not native hard heap/CPU quotas. Upstream allocations remain
infallible; native decoder use requires a separate bounded arena/process
under the existing codec contract.

`png = "=0.18.1"` is optional behind `host-cli`. Actual PNG encoding is
**host/std**, not ported to Baryon OS. The default no_std build contains no
png/crc32fast/flate2. Provenance/pins/licences/build-script exception are in
THIRD-PARTY.md and Cargo.lock. No new XML/PNG/compression engine or general
graphics foundation.

Primary format source: W3C PNG Working Group,
*Portable Network Graphics (PNG) Specification, Third Edition*,
Recommendation 24 June 2025,
<https://www.w3.org/TR/2025/REC-png-3-20250624/>.
Maintained encoder API:
<https://docs.rs/png/0.18.1/png/struct.Encoder.html>.

## Acceptance and evidence limits

Targeted Rust tests plus actual CLI input/output. An independent Python
zlib/PNG oracle checks chunks/CRC/filters/dimensions/all RGBA bytes, including
alpha. Existing JPEG references were separately generated with Pillow/libjpeg:
at most three levels of sample difference; crop and all eight orientations
were then checked byte-exactly against the full decoded sample buffer.
No fixture generation or retesting of old consumers. No_std core compilation,
not a native PNG encoder.

No native window/open/save/launch, VM, physical device, human user test,
independent audit or release validation is claimed. The component receipt
contains executed commands and hashes.
