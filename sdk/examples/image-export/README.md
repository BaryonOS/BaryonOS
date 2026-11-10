# Export PNG/JPEG to BMP

TASK-PLATFORM-M7-IMAGE-EXPORT, baseline `d10b5cda`. A concrete host-only file
consumer for the existing image codec. No native export dialogue, new decoder,
extra rendering engine, app installer or resource broker.

```sh
cargo +1.90.0 run --offline --locked --manifest-path sdk/examples/image-export/Cargo.toml --features host-cli --bin image-bmp -- --background ffffff < foto.png > foto.bmp
```

The background is **required** and consists of six hexadecimal RGB digits.
This avoids silently flattening transparency against a hidden default colour.
No stdout until all decoding/export checks succeed; errors return exit 1 and
stderr without image contents. A failure during the actual stdout write may
leave partial output. No atomic saving; never use the same input/output
filename in shell redirection.

## Profile and loss boundary

Supported PNG/JPEG input follows only the existing `services/image-codec`
profiles and dependency ledger. Existing `Image::rgba8_into` applies JPEG
EXIF orientation and reduces RGBA16 to RGBA8. Alpha is then blended using
existing sample values against the selected colour. No ICC/gamma/linear-light
transformation or photographic-fidelity claim. The result is a separate
**uncompressed RGB24 BMP**: BGR bytes, positive height/bottom-up, 40-byte DIB,
54-byte pixel offset, four-byte row alignment with zero padding, no palette
or colour profile. EXIF orientation is incorporated in pixels; original
EXIF/ICC/metadata are not copied into the derived BMP. Original input and
raw pixels remain unchanged. This is not a lossless metadata/precision roundtrip.

Primary format sources: Microsoft,
*[BITMAPFILEHEADER](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapfileheader)*
(document version 5 December 2018) and
*[BITMAPINFOHEADER](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapinfoheader)*
(26 April 2023), accessed 3 November 2026.
Only the standard file format is used; no Windows API/kernel.

## Resource and authority boundary

Input at most 4 MiB, 4 megapixels (`4 * 1024 * 1024`), axis length 16384,
output at most 16 MiB. A caller may lower the output budget. A temporary
oriented RGBA8 buffer and the BMP buffer are each bounded and fallibly
reserved; approximately 32 MiB combined maximum, in addition to decoder/raw
pixels. The codec work limit controls admission, not a hard heap quota.
Upstream may allocate infallibly; native use requires a separately bounded
decoder process. This host consumer does not demonstrate native OOM/crash recovery.

No new registry dependency or unsafe. The new lockfile uses the existing
exactly pinned codec closure and JPEG vendor patch. No changes to codec,
viewer, shared workspace, native IPC, authorisation, storage or networking.
The host CLI inherits stdio; a native export must use the existing save portal
and an actually authorised volume client.

## Acceptance

```sh
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/image-export/Cargo.toml --features host-cli
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/image-export/Cargo.toml --features host-cli --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path sdk/examples/image-export/Cargo.toml -- --check
cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/image-export/Cargo.toml --target x86_64-unknown-none
```

Tests check actual output headers, bottom-up row order, BGR, all padding
lengths, exact alpha/RGBA16 values, EXIF rotation, actual PNG/JPEG fixtures
and CLI processes. Malformed samples, dimensions, background, missing choices
and insufficient output budgets produce explicit errors. No native executable,
GUI, saved Baryon OS object, hardware or release acceptance; a no_std build
is target compilation only.
