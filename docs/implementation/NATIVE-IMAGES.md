# Native PNG/JPEG decoding component

TASK-IMAGE-CODEC-NATIVE, baseline `985ed4da0ead6865c8d1fd0d4deb34ef7f97f3eb`.
This is a partial component for **FR-A03/UC04**, not an image viewer or
approval of PNG/JPEG in their entirety. The project's kernel/capabilities
remain authoritative; parser code receives bytes and returns pixels, without
a display, storage, network, key or entropy capability.

## Dependency decision before inclusion

The official crate archives were first downloaded outside the repository and
checked for crates.io SHA256, source, licences, features, MSRV, allocations,
unsafe code and build scripts. The five exact development pins were then
added. [dependency-ledger.json](../../services/image-codec/dependency-ledger.json)
records all hashes, source commits from `.cargo_vcs_info.json`, dates and
versions; [THIRD-PARTY.md](../../services/image-codec/THIRD-PARTY.md) retains
the MIT notices.

| Crate | Pin / MSRV | Active choice |
|---|---|---|
| zune-png | 0.5.2 / 1.87 | no_std+alloc; defaults, SSE and portable-SIMD disabled |
| zune-jpeg | **0.5.16-rc3** / 1.77.1 | no_std+alloc; std/x86/neon/arith/portable-SIMD disabled; explicit development RC |
| zune-core | 0.5.3 / 1.75 | Byte cursor/options; no std/log/serde |
| zune-inflate | 0.2.54 / no declared MSRV | zlib only, no gzip/std; actually built with Rust 1.90 |
| simd-adler32 | 0.3.10 / no declared MSRV | Defaults/std/nightly disabled; unknown-none has no SSE2 target feature and selects scalar |

The active dependency closure contains exactly these five engine crates:
four registry crates and one locally verified JPEG vendor copy. The internal
`zune-core feature default` edge is empty and activates no std. No crate has
a build script, C/host decoder or downloader. Zune crates are MIT OR Apache2
OR Zlib; Adler is MIT. The inflate crate archive omits the full licence text;
the pinned repository explicitly offers MIT. That copyright notice and the
complete MIT text from the same project have therefore been retained. Only
the local JPEG correction described below differs from upstream.

Primary sources: [zune-image](https://github.com/etemesi254/zune-image),
[zune-png0.5.2](https://crates.io/crates/zune-png/0.5.2),
[zune-jpeg0.5.16-rc3](https://crates.io/crates/zune-jpeg/0.5.16-rc3),
[simd-adler32](https://github.com/mcountryman/simd-adler32).
The upstream project had recent JPEG commits on 24 October 2026. Targeted
RustSec source searches for zune-png/jpeg/inflate returned no entries; this
is neither a statement that no vulnerabilities exist nor a complete audit.

Stable JPEG 0.5.15 was deliberately **not** selected: upstream
[#409](https://github.com/etemesi254/zune-image/issues/409) and
[#382](https://github.com/etemesi254/zune-image/issues/382) show
scalar/progressive panics, and
[#496](https://github.com/etemesi254/zune-image/issues/496) incorrect colours,
with fixes in newer source. The selected RC is not production approval.
Open upstream variants remain visible: APP14/greyscale
[#444](https://github.com/etemesi254/zune-image/issues/444), DRI interval
[#509](https://github.com/etemesi254/zune-image/issues/509), YCCK
[#442](https://github.com/etemesi254/zune-image/issues/442). Their impact must
be resolved for each supported profile with real fixtures before format
acceptance.

## API, formats and resource boundary

`services/image-codec` is a standalone no_std workspace. `decode` receives
immutable selected object bytes, explicit `Limits` and cancellation. A
successful result contains owned `Rgba8` or **owned `Rgba16`** and retains a
borrow of all original bytes including metadata; no re-encoding or object
modification occurs. `rgba8_into` performs a checked display conversion with
rounding to eight bits; full samples are preserved.

PNG uses the maintained `decode_into(&mut[u8])` API with big-endian samples.
The wrapper constructs u16 values safely and iteratively. Consequently, the
known `convert_u16_to_u8_slice` alias cast from upstream's other `decode()`
path is not executed; the engine is not incorrectly described as entirely
free of unsafe code. Filters, palette/tRNS, DEFLATE, CRC and Adam7 remain in
the maintained engine.

The PNG resource envelope checks only chunk bounds/count, dimensions, metadata
quantity and the prohibited animation path. iCCP and zTXt are prechecked by
the maintained inflater with limited output: upstream's own metadata inflater
otherwise uses a 1-GiB default. CRC+Adler and strict mode are enabled. The
inflater limit is best effort and can temporarily require overshoot/padding;
it does **not** replace the hard process arena. JPEG bounds dimensions,
pixels, scans and work admission before pixel decoding. Input length bounds
metadata.

The wrapper reserves its own output fallibly, but upstream has infallible
allocations. The caller must therefore use a separate parser process with a
bounded private arena. OOM/panic there is terminal; no host allocator or
partial image is published as success. A real native OOM/restart/peer-isolation
trial remains open. Current 8-MiB process quotas and the 256-MiB allocator
pool are not elevated into product policy; large photographs remain an
explicit product requirement.

| Profile | Actually tested / current boundary |
|---|---|
| PNG | RGB8, RGBA8/transparency, palette4+tRNS, grayscale1, gray-alpha8, Adam7RGB8 and RGBA16 with exact sample preservation. No claim that all bit-depth/colour/metadata combinations are approved. |
| JPEG | Huffman baselineRGB4:4:4, progressiveYCbCr4:2:0 and grayscale8; pixels are actually decoded. Valid EXIF orientation 1–8 controls the display pixels from `rgba8_into`; see the metadata rule. The original progressive quality error is demonstrated below as failing→passing, without changing the threshold. |
| Still open | PNG2/other combinations, APNG composition/animation, CMYK/YCCK, arithmetic/lossless/high-precision JPEG, DRI/APP14 and further corpus coverage. `UnsupportedAnimation/Color/Precision` is an explicit implementation boundary, not a hidden hardware state. |
| Metadata | Original bytes including EXIF/ICC and raw decoded pixels remain unchanged; source dimensions and `pixel_hash` remain in decoder order. For JPEG, `rgba8_into` applies valid EXIF orientation 1–8 to display pixels and dimensions. An EXIF APP1 payload larger than `Limits::metadata_bytes` rejects the image with `Error::MetadataLimit`, without falling back to absent orientation. Invalid EXIF structure or values are ignored (no orientation). `metadata_transforms_applied=false` describes the raw samples; ICC/gamma colour management and PNG eXIf remain outside scope. The EXIF tag follows CIPA DC-008-2012, *Exchangeable image file format for digital still cameras: Exif Version 2.3* ([primary specification](https://www.cipa.jp/std/documents/e/DC-008-2012_E.pdf)). The app-local viewer now also processes orientation in tiles/zoom/pan; the native viewer flow, export and editor integration remain open. |
| Cancellation | Before decoding/at PNG chunk boundaries/after decoding; JPEG also has the actual upstream MCU check hook. The native trial tests cancellation beforehand, not measured mid-decode cancellation latency. PNG mid-decode cancellation still requires process cancellation or a maintained incremental path. |

## Stack boundary and owned arena

**Historical implementation:** Baryon OS, *image-viewer: render_tile*, version
`d269f9b4` (3 November 2026), still draws raw decoded pixels
([primary renderer source](https://github.com/simonvanieperen/nieuw-os/blob/d269f9b45a54576b39629b36de94f9cdbb9f6f71/sdk/examples/image-viewer/src/lib.rs)).
EXIF correction in `rgba8_into` therefore does not prove an oriented viewer
or native user flow.

**Current viewer consumer:** `Orientation::display_dimensions` and the checked
`source_position` connect EXIF to the existing tile renderer without full
image conversion. Display dimensions control Fit, zoom, pan and semantics.
Source pixels, metadata and selected object bytes remain unchanged. The
[consumer trial](../verification/platform-m7-exif-viewer-20261103.md) describes
the new host evidence and separate native decoder regression; native viewer
startup, the object portal and user flow remain open.

**Test evidence:** Baryon OS, *viewer fixture and small image integration slice*,
version `d269f9b4` (3 November 2026): the fixture was updated for the new
`Image` fields and tested separately
([primary test source](https://github.com/simonvanieperen/nieuw-os/blob/d269f9b45a54576b39629b36de94f9cdbb9f6f71/sdk/examples/image-viewer/tests/viewer.rs),
[commands and results](https://github.com/simonvanieperen/nieuw-os/blob/d269f9b45a54576b39629b36de94f9cdbb9f6f71/docs/verification/integration-sprint-20261103-image.md)).

An initial instruction screen found only individual stack adjustments up to
4 KiB. That was insufficient: the actual decoder frame reserved a cumulative
30760 bytes. Together with a statically reachable scan/DHT chain, 77232 bytes
were visible, exceeding the current 64-KiB stack. This was a code-generation
finding, not an executed stack fault or complete branch trace.

The wrapper now constructs the large JPEG decoder in a separate
`#[inline(never)]` helper and moves it into an owned Box slice in the private
arena. Allocation first uses stable `Vec::try_reserve_exact(1)`; capacity 1
is checked so the subsequent Box conversion needs no shrinking allocation.
A construction allocation error is fallible; later infallible upstream
allocations remain terminal on OOM. PNG/JPEG stack frames remain separate.
Kernel stack and process quotas were not increased.

[stack-codegen.json](../../services/image-codec/verification/stack-codegen.json)
records actual none ELF and LLVM IR: dispatch frame 8 bytes, JPEG caller 616,
construction helper 30216. The 30056-byte object slot exists only in that
helper, which returns before parser work. The same inspected DHT call chain
now reserves 47104 bytes, approximately 18 KiB below 64 KiB. The guest entry
additionally reserves 248 bytes. A separate static progressive→marker→Huffman
chain is also in the report: 17664 bytes before guest entry/other unmodelled
frames; the actual progressive fixture contains DHT markers after the first
SOS. This remains static reachability, not an executed branch trace or a
complete maximum call chain. This is not a maximum-stack proof for all
branches or a native OOM/guard trial; actual VM execution remains mandatory.
All image hashes remain unchanged.

The next real VM on `9aa74870de55387094414c3ec1a909fbeec53c81` found another
previously unexamined chain: PID5 received a page fault in
`capture_scan_header_state`, RIP `0x40118aaa`, address `0x7ffee9e8` below
the stack guard `0x7ffef000`. The large inline snapshot plus clone reserved
approximately 60 KiB there before caller frames. The original fault remains
failing evidence; the previously selected DHT chains were not a complete
maximum-stack analysis.

The second local vendor patch now preserves exactly four DC and four AC
table slots, including `None`, in fallibly reserved Vec buffers. Recorded
quantisation, arithmetic-conditioning/statistics fields, restart interval,
colour space, Adobe and mjpeg status remain preserved. Capture and both
replay copies return `AllocationFailed` on failed reservation; the wrapper
returns `OutOfMemory`. All new reservations succeed before their local
parser/checkpoint mutation. `headers_decoded` is published only after a
successful snapshot. Recovery copies individual tables; EOF/cancellation
rollback and SOS checkpoints retain their existing behaviour. There is no
new unsafe code or codec algorithm. This does not make all existing small
`Box::new` owners or other upstream allocations fallible; native OOM
isolation remains separately required.

[scan-snapshot-codegen.json](../../services/image-codec/verification/scan-snapshot-codegen.json)
binds the new ELF and selected cumulative prologues/call paths. This remains
static code-generation evidence, not an executed stack or maximum-call-chain
trial. The arena remains 4 MiB, the user stack 64 KiB and the native fixture/
hash contract unchanged. The next integrated VM must still approve actual
decoding.

## Executed evidence and the actual failing quality question

Fixtures are original patterns of varying dimensions, not prepopulated success
images. [generate.py](../../services/image-codec/fixtures/generate.py) is
only a host encoder/oracle; Pillow 12.3/libjpeg and known 16-bit samples
provide independent pixels. OS code contains no Pillow/std/host decoding
path. The manifest records input/reference hashes. The PNG16 preview rounds
full 16-bit samples; Pillow's eight-bit PNG view truncates. The full16
reference therefore comes from the fixture encoder's known source values
and is checked sample by sample; Pillow also reads the actual PNG file.

The original [failing report](../../services/image-codec/verification/host-tests-before-h2v2.json)
is retained at checkpoint 5910204: 10 groups, 8 passed/2 failed, exit 101.
Both failing groups contain the same progressive 4:2:0 comparison: maximum
4 instead of the preregistered 3, two of 87236 channels, both blue at
(176,17)/(175,18). Total absolute deviation 27763; squared deviation 43153.
The threshold remains 3.

The causal host diagnosis reads real component planes from the unchanged
decoder. Pillow/libjpeg-turbo **3.1.4.1** supplies an independent reference
through JPEG draft(YCbCr, original size), without an RGB round trip. The old
two-step model exactly reproduces all 65427 original YCbCr bytes. The affected
pixels have the same Y and a Cb higher by 2; this explains the blue deviation
of 4. Standard 9:3:3:1 interpolation with a single rounding reduces this
chroma error. [Component evidence](../../services/image-codec/verification/jpeg-components.json)
retains the measurements; the host diagnostic is host tooling only.

The local correction changes **only scalar `upsample_hv`**. It expresses the
mathematical triangle weights and alternating 8/7 tie bias in original safe
Rust code; no C source was incorporated. Primary reference:
[libjpeg-turbo jdsample.c, commit 9217719d…](https://github.com/libjpeg-turbo/libjpeg-turbo/blob/9217719d3a58633923b096af4c1d50d304768a64/src/jdsample.c#L362).
No upstream acceptance is claimed. The original crate archive, SHA,
[exact small diff](../../services/image-codec/vendor/upstream/upsample-hv.patch)
and [all source hashes](../../services/image-codec/vendor/upstream/source-verification.json)
remain preserved. This first patch changes only `upsample_hv`. The separate
[snapshot patch](../../services/image-codec/vendor/upstream/scan-header-heap.patch)
changes `decoder.rs`, `mcu.rs` and the explicit allocation error in `errors.rs`;
the other 37 upstream files and all licence texts are byte-identical. Only
this literal upstream source and unified-diff context lines have a local
whitespace-style exception; they are not rewritten to remove formatting
warnings. Wrapper/probe/tests remain subject to normal checks.
`verify_vendor.py` checks that complete file set, archive hash, patch hash
and unchanged upsampler prefix/suffix before every native build; the second
patch must be the exact unified diff of only its three source files. Four
host trials reject changed source/archive, an additional build script and
a changed snapshot patch or one that does not match the source.

Five new host trials inject real allocation failures into the production
engine, check both public OOM error paths and reuse the same decoder after
snapshot/checkpoint errors. The synthetic
[three-SOS fixture](../../services/image-codec/fixtures/scan-rollback/oracle.json)
with DQT/DHT/DRI/COM/APP changes is test data only; Pillow/libjpeg serves as
the independent host reference. EOF and cancellation between scans must allow
recovery with exactly the same pixels. This additional host fixture changes
neither the ten native input images, the quality tolerance nor the mandatory
format scope.

The [original report](../../services/image-codec/verification/host-tests.json)
contains **14 passing test groups, exit 0**: 11 decoding groups plus 3 groups
on the actually patched upsampler. PNG pixels and full 16-bit samples match
exactly. Baseline JPEG maximum 3, greyscale 1, progressive now 3; progressive
total absolute deviation 1375 and squared deviation 1903. The same previously
failing tests remain active. Twelve additional real baseline/progressive JPEGs
check 1×1, 1×17, 17×1, 3×5, 5×3, 17×19; direct upsampler trials check
minimum/maximum samples, odd widths, tie bias, capacity tails and invalid
geometry before any output write.

Other trials check a real decompression bomb, truncation, CRC, dimension/input/
work bounds, cancellation, unchanged source/output with wrong capacity and
reuse after errors; 512 deterministic malformed mutations cause no panic.
This is not an unlimited fuzz/security audit. Clippy with `-D warnings` was
executed and passes. A none check and an actual static native ELF were built.
Miri was not executed; these trials do not replace a complete memory-safety
audit. **At component checkpoint `9ae1c73`, QEMU execution had not yet occurred.**
The first integrated image trial on source
`2b5d69c69eaf4196eb00a8488c88273defe14770` stopped with raw exit 35 and
error step 1 before decoding: startup word 0 is the boot mode, not the ABI
version. The probe now requests the ABI from the running kernel through
`native::check_abi()`; the old failed trial remains preserved. A new native
decoding trial remains open.

## Native process contract for integration

`sdk/examples/image-probe` builds with the same Rust 1.90 and a dedicated
target directory:

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-image-core-target \
  python3 -B tools/dev.py cargo +1.90.0 test --locked --offline \
  --manifest-path services/image-codec/Cargo.toml -- --nocapture
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-image-probe-target \
  python3 -B sdk/examples/image-probe/build.py
```

The first command passes after the local rounding correction; the original
exit-101 report and unchanged quality threshold remain preserved. The second
returns `extra_env.NOS_IMAGE_PROBE_ELF`, ELF inspection and SHA256; this
means a build, not native success. Root later connects opt-in
`native-system-images=[native-system,process-memory,fpu-context]`, ABI1.7,
Sys3/Sys4 and only a boot-manifest slot 4 without additional capabilities.
`_start` performs CLD+JMP and preserves the loader's SysV stack alignment.
The guest itself requests 1024 fresh heap pages and transfers them exclusively
to UserAllocator. Kernel/harness provide no heap, decoded pixels or successful
reply.

| Startup offset | Native fact / expected parity |
|---|---|
| 40/240 | done1 / errorstep0 only after all assertions |
| 400/408 | PNG8 (seven variants+redecode after errors), JPEG3 |
| 416/424 | 149647 actually processed pixels /10 checked negative outcomes |
| 432/440 | Actual FNV-1a64 pixel aggregation `079d5118b890ae14` /623808 decoded sample bytes |
| 448/456/464 | 1024 heap-growth pages /actual arena high-water >0≤4MiB /zero remaining allocations |
| 472/480/488 | 1 cancellation /actual monotonic start/end ticks |
| 496 | **QUALITY_OPEN=0** for the resolved specific JPEG-oracle question; general format/viewer acceptance remains open |

FNV is only a parity checksum, not a cryptographic authenticity claim. PNG
checksums correspond to independent exact samples. JPEG checksums compare
the same engine configuration between host and guest; their independent
image quality remains separately tested. The root observer must retain the
actual boot PID so that later worker-slot reuse does not accept the wrong
identity. Exact source/image binding and a cold joint regression then follow,
real OOM and process cancellation, object RPC and compositor/Materiaal
integration. A full-screen preview, ELF or isolated decoding trial does not
approve FR-A03.
