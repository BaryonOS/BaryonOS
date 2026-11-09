# Verification

Basis `ca45a7dabe6a040921de03bdc2fad3e5045a75ba`, Rust 1.90.0, crate-local
lockfile, own `target/`. Logs and exit codes live next to this file;
`SHA256SUMS` binds sources, lockfile and logs (`sha256sum -c SHA256SUMS`
from `drivers/virtio-snd`).

```sh
cargo +1.90.0 fmt --check
cargo +1.90.0 clippy --offline --locked --all-targets -- -D warnings
cargo +1.90.0 test --offline --locked
cargo +1.90.0 build --offline --locked --release --target x86_64-unknown-none
```

Evidence boundary: host unit tests and no_std crosscompile. No QEMU guest,
audio output, transport, physical hardware or human audit. The 6 prototype
tests are carried over (annotated in `src/tests.rs`) and extended with
feature, range/overflow, multi-record and reader-bounds negatives.

## TASK-H03-TRANSPORT-MODEL-1 (basis `823cba3eb38786d39726ecfb98d6ce7f258754f7`)

`src/transport_model.rs` + `src/transport_model_tests.rs`: pure, bounded host
model of `TRANSPORT.md` §4–§6, linked only under `#[cfg(test)]` (no
production export; the `x86_64-unknown-none` rlib does not contain it). No
deps, Cargo or lock change, no unsafe, no I/O, no VM.

Invariants (each with a red mutant, `model-mutants.py` → `model-mutants.log`,
7/7 caught, source sha256 restored):

| Inv | Rule | Mutant |
| --- | --- | --- |
| I1 | slot reuse only after fence = status 0 **and** BME off read back; frozen (no submit/control/selective completion) while `Fencing` | M1 |
| I2 | stale lease (epoch/PID/cap generation) refused without state change; completion or used-index change after both ACKs ⇒ `Quarantined` | M2 |
| I3 | fence deadline miss, unarmed internal fence, contradicting read-back, deadline/epoch overflow ⇒ `Quarantined`, pool stays pinned | M3 |
| I4 | no submit/start after revoke; reinit needs a fresh lease (higher cap generation, current epoch) | M4 |
| I5 | rx queue never gets buffers | M5 |
| I6 | double completion never frees twice (⇒ fence); double RELEASE refused (F19) | M6, M7 |

Gates (logs/exit `model-{fmt,clippy,tests,build,mutants}`): fmt --check,
clippy `--all-targets -D warnings`, `test --offline --locked` (32 = 11 codec +
21 model), none release build, `python3 verification/model-mutants.py`; all
exit 0.

Evidence boundary: status 0 and BME-off read-back are **model assumptions**
about a conforming device, not proven DMA quiescence or a device fence. No
production lease/fence/IRQ/MMIO/DMA/root contract is adopted; B1–B9 review
stays separate. Not H03 acceptance, no points.

## PCM lifecycle codec (basis `8bb153092e967aae314ecde35bcf963565a87ac7`)

`src/pcm.rs` + `src/pcm_tests.rs`: production (`no_std`, `forbid(unsafe_code)`,
no heap, no deps, Cargo/lock unchanged) PCM control/TX/event codec with a
per-stream lifecycle. `src/lib.rs` only gains `pub mod pcm`, five `StreamInfo`
getters and imports `S_OK` from `pcm`. No I/O, VM, transport or integration.

Primary sources used for the wire constants and lifecycle (fetched 4 Nov
2026, not vendored):

- QEMU `v10.2.1` `include/standard-headers/linux/virtio_snd.h` (request codes
  `0x0101..0x0105`, status `0x8000..0x8003`, event codes `0x1000/0x1001/
  0x1100/0x1101/0x1200`, struct layouts, `FMT_S16 = 5`, `RATE_48000 = 7`,
  `PCM_F_*` bits 0..4).
- OASIS virtio-spec `device-types/sound/description.tex` (lifecycle
  "SET PARAMETERS → set parameters, prepare; PREPARE → set parameters,
  prepare, start, release; START → stop; STOP → start, release; RELEASE →
  set parameters, prepare"; `buffer_bytes % period_bytes == 0`; no undefined
  or unadvertised feature/format/rate; padding 0; TX status `OK`/`IO_ERR`;
  device completes pending I/O before RELEASE completes).

Correction carried into the test-only model: `transport_model.rs` used
`0x8002` as `IO_ERR`; that value is `NOT_SUPP`, `IO_ERR` is `0x8003`. The
model and its tests now use `0x8003`, and a TX completion with `NOT_SUPP`
fences (`bad_len_or_status_fences_io_err_stops`). `model-mutants.py` still
catches 7/7 (`model-mutants.log`).

Tests (`pcm::tests`, 18): exact constants and layouts; SET_PARAMS encoding,
feature selection and bound table; eight-byte simple requests; capture and
non-stereo-S16-48k streams refused at construction; the full 5×5
state×request matrix (allowed transitions encode and mark `pending`, every
other is `BadState` without state change); `Busy` until completion; F19
double RELEASE refused before encoding and a new lifetime after SET_PARAMS;
control completion needs `pending`, exactly 4 bytes, `NOT_SUPP` keeps state,
other statuses are `Status`; TX header only in `Prepared|Running` with frame
and period bounds; TX status F9 (`IO_ERR` ⇒ `Faulted` until device-confirmed
STOP), every other status/length fences, completion after RELEASE is late;
event decode bounds; per-stream event routing; F17 budget; the fence policy
table; reader bounds.

Gates (crate-local, Rust 1.90.0; logs `fmt`, `clippy`, `tests`, `build` and
`model-mutants`, all exit 0; `model-{fmt,clippy,tests,build}` are byte-equal
copies of that single run because the model has no separate gate any more):

```sh
cargo +1.90.0 fmt --check
cargo +1.90.0 clippy --offline --locked --all-targets -- -D warnings
cargo +1.90.0 test --offline --locked
cargo +1.90.0 build --offline --locked --release --target x86_64-unknown-none
python3 verification/model-mutants.py
```

Evidence boundary: host unit tests and a no_std crosscompile only. No QEMU
guest, no audio, no transport, no control time-out, no physical hardware, no
human audit. H03 stays open; this closes no FR row and earns no points.
