# Clipping a PCM WAV fragment

TASK-PLATFORM-M7-WAV-CLIP, base `6d523964`. Concrete file operation:
import a real supported WAV, cut out a frame range, export it as a
standalone WAV file. No playback mock, audio broker,
generic codec engine or duplication of an existing driver.
The model is `no_std+alloc`; the CLI is solely a host program.

```sh
cargo +1.90.0 run --offline --locked --manifest-path sdk/examples/audio-clip/Cargo.toml --features host-cli --bin wav-clip -- --start-frame 48000 --frames 96000 < opname.wav > fragment.wav
```

A frame contains **all channels at one point in time**. At 48 kHz this
example selects the next 2 seconds starting at second 1. The selection is exact,
not rounded to timestamps or silently shortened to the end of the file.
Original PCM16 sample bytes, channel order and sample rate remain
the same. No conversion, resampling, gain, fade or dithering; a hard cut
may produce an audible click. Original input remains unchanged.

## Explicitly supported profile

- RIFF little-endian `WAVE`, PCM tag 1, signed 16-bit samples,
  mono or stereo, sample rate 8000 to 192000 Hz inclusive.
- Exactly one `fmt ` chunk of 16 bytes, or 18 bytes with cbSize 0,
  and exactly one `data` chunk. Both chunk orders are accepted.
  Byte rate, block alignment and complete frames are checked.
- At most 16 MiB complete file and 1024 chunks. The RIFF length must exactly
  cover the file; no hidden/trailing bytes or partial header.
  All chunks including padding and metadata after the audio are validated.
- No RIFX/RF64/WAVE64, PCM8/24/32, float, extensible, multichannel,
  ADPCM, MP3/FLAC or other compression. No complete WAV coverage claimed.

Extra chunks (including JUNK/LIST/cue/loop metadata) are counted, not executed.
If there are extra chunks, clipping requires the explicit choice
`--drop-metadata`. Without that choice the result is `MetadataChoiceRequired`, not
successful output. Marker/loop positions may be invalid after clipping;
the exporter therefore does not claim to preserve or recompute them.
`--drop-metadata` writes only the canonical fmt/data chunks and deliberately removes
all extra metadata. A clip without extra metadata needs no
repeated consent.

## Limits and errors

`Wave::parse` borrows the original samples without allocation.
`clip(start..end, drop_metadata, output_limit)` uses a half-open,
non-empty frame range and reserves the exact output fallibly.
The output limit is at most 16 MiB, and can be lowered by the caller.
CLI input at most 16 MiB+1 detection byte, output at most 16 MiB;
no native hard heap quota/OOM recovery claim.

No unsafe or dependency. New subtree `sdk/examples/audio-clip/**` plus
its own receipt, no changes to the existing SDK workspace, UI, driver, kernel, audio,
General, launch, storage, rights or score.
A native edit/save flow requires the existing portal/app owners;
sample-exact file editing itself does not require a native audio device.
The host CLI inherits stdio and grants no Nieuw OS resource capability.

All parse/selection/export errors give exit 1 and stderr without sample content.
No stdout until the complete operation succeeds. The stdout write is not atomic;
a write error can leave partial output behind. Never use the same
input/output path with shell redirection.

## Primary sources and acceptance

Microsoft, *Resource Interchange File Format (RIFF)*, 31 May 2018,
<https://learn.microsoft.com/en-us/windows/win32/xaudio2/resource-interchange-file-format--riff->;
*WAVEFORMATEX*, 5 December 2018,
<https://learn.microsoft.com/en-us/windows/win32/api/mmreg/ns-mmreg-waveformatex>.
Accessed 3 November 2026: RIFF lengths, WORD padding, fmt/data,
sample rate, block alignment and byte rate. No Windows runtime or kernel.

```sh
# Run from the repository root on a prepared Rust toolchain.
set -e
export CARGO_TARGET_DIR=target/platform-wav-host
mkdir -p build/platform-wav-evidence
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/audio-clip/Cargo.toml --features host-cli
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/audio-clip/Cargo.toml --features host-cli --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path sdk/examples/audio-clip/Cargo.toml -- --check
cargo +1.90.0 build --offline --locked --manifest-path sdk/examples/audio-clip/Cargo.toml --features host-cli --bin wav-clip
cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/audio-clip/Cargo.toml --target x86_64-unknown-none
python3 -B sdk/examples/audio-clip/tests/interop.py target/platform-wav-host/debug/wav-clip build/platform-wav-evidence/fragment.wav
file build/platform-wav-evidence/fragment.wav
```

Six Rust tests and 48 real CLI clip outputs are tested, with independent
Python `wave` as input writer/output reader and exact sample comparison.
Four sample rates, mono/stereo, start/end/full clip and explicit
metadata refusal/discard. Negative tests cover truncations, overflow/chunk
lengths, missing padding, duplicate/missing chunks, incomplete frames,
unsupported formats, inconsistent fmt, selection/CLI and budgets.
No native executable, GUI, actually audible output, audio device,
QEMU, hardware, human, independent audit or release acceptance.
