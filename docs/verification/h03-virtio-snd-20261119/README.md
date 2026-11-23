# H03-T3a: kernel-only virtio-snd PCM output fixture (QEMU 8.2.2)

Evidence level: **native QEMU/TCG emulator output** of an IF0 kernel fixture.
Not audible or physical playback, not latency, not DMA isolation, not a ring-3
or capability path, not a human audit. H03 stays open.

## Source and environment

- Parent commit `9c707c8` (main `035b44e` plus the local H05 S3 commit); source
  fingerprint `9cb285c1740e14c93e2c0d2462ffaee32715dea2129fea646cff4cd1c2fdcc12`
  (`tools/common.py` `source_hash`, version 3). The fingerprint excludes `docs/`
  and `*/verification/`, so this directory does not change it.
- Rust `1.100.0`; QEMU `8.2.2 (Debian 1:8.2.2+ds-0ubuntu1.18)`; OVMF
  `/usr/share/OVMF/OVMF_{CODE,VARS}_4M.fd`; `NOS_QEMU_MACHINE=pc-q35-8.2`;
  TCG single-threaded with `-icount shift=0,sleep=off`; 4 host CPUs shared.
- `<CHECKOUT>` in `report.json` is an exact copy of the agent worktree outside
  the parent checkout, with the identical source fingerprint. Reason (environment
  defect, not part of this change): a worktree nested under the main checkout
  also loads the parent's `.cargo/config.toml`; cargo concatenates both
  `rustflags` arrays, the kernel is linked with `-Tkernel/linker.ld` twice, and
  `__text_end`/`__rodata_end`/`__kernel_end` resolve to `0x2000000`. Every M2+
  profile, plain `milestone-m2` included, then triple-faults after `KS-02:PASS`
  (raw exit 0). The same source built outside the parent tree passes.

## Commands and raw exit codes

Host (in the worktree):

| Command | Exit |
| --- | --- |
| `cargo +1.100.0 test --locked --manifest-path drivers/virtio-snd/Cargo.toml` (63 tests) | 0 |
| `cargo +1.100.0 clippy --locked --manifest-path drivers/virtio-snd/Cargo.toml --all-targets -- -D warnings` | 0 |
| `cargo +1.100.0 fmt --manifest-path drivers/virtio-snd/Cargo.toml -- --check` | 0 |
| `cargo +1.100.0 build --locked --release --manifest-path drivers/virtio-snd/Cargo.toml --target x86_64-unknown-none` | 0 |
| `cargo +1.100.0 fmt --all -- --check` | 0 |
| `python3 -B -m unittest discover -s tools/tests -p 'test_virtio_snd.py'` (11 tests) | 0 |
| `python3 -m compileall -q tools` | 0 |
| `python3 -B -m unittest discover -s tools/tests -p 'test_*.py'` (563 tests) | 1: one failure and six errors outside this change (below) |
| `cargo +1.100.0 check -p os-kernel --target x86_64-unknown-none --features <snd>,<other>` for `native-rtc`, `native-xhci-probe`, `native-acpi-s3`, and both test variants together | 101 each, by the new `compile_error!` guards (expected) |

The full host suite's failure `test_gs_discipline` also fails on the unchanged
parent tree (`sleep.rs` and `arch.rs` MSR writes from the H05 S3 commit); the six
errors (`test_agent_isolation`, `test_agent_pipeline`) are `Git environment
overrides require a verified controller adapter` from this session's git
environment.

Native, sequential, run from `<CHECKOUT>` with `NOS_QEMU_MACHINE=pc-q35-8.2`
and a shared `NOS_TARGET_DIR`/`CARGO_TARGET_DIR` (`<shared>`: the values were
not recorded). Each `NOS_BUILD_DIR` below is taken from the `run_directory`
fields of `report.json`; for the xHCI and USB HID regressions it was not
recorded:

| Command | Exit |
| --- | --- |
| `NOS_BUILD_DIR=build/final-snd-normal python3 -B tools/build.py --kernel-feature native-virtio-snd-probe` | 0 |
| `NOS_BUILD_DIR=build/final-snd-normal python3 -B tools/run_virtio_snd.py --case normal` | 0 (raw 33) |
| `NOS_BUILD_DIR=build/final-snd-normal python3 -B tools/run_virtio_snd.py --case no-device` | 0 (raw 33) |
| `NOS_BUILD_DIR=build/final-snd-normal python3 -B tools/run_virtio_snd.py --case access-platform` | 0 (raw 33) |
| `NOS_BUILD_DIR=build/final-snd-fence-reinit python3 -B tools/build.py --kernel-feature test-snd-fence-reinit` | 0 |
| `NOS_BUILD_DIR=build/final-snd-fence-reinit python3 -B tools/run_virtio_snd.py --case fence-reinit` | 0 (raw 33) |
| `NOS_BUILD_DIR=build/final-snd-quarantine python3 -B tools/build.py --kernel-feature test-snd-quarantine` | 0 |
| `NOS_BUILD_DIR=build/final-snd-quarantine python3 -B tools/run_virtio_snd.py --case quarantine` | 0 (raw 33) |
| `NOS_BUILD_DIR=build/final-standard python3 -B tools/build.py` | 0 |
| `NOS_BUILD_DIR=build/final-standard python3 -B tools/run.py --smoke` | 0 (raw 33) |
| `NOS_BUILD_DIR=build/final-m2 python3 -B tools/build.py --kernel-feature milestone-m2` | 0 |
| `NOS_BUILD_DIR=build/final-m2 python3 -B tools/run.py --smoke --milestone m2` | 0 (raw 33) |
| `python3 -B tools/build.py --kernel-feature native-xhci-probe` (`NOS_BUILD_DIR` not recorded) | 0 |
| `python3 -B tools/run_xhci.py --case noop` (same build) | 0 (raw 33) |
| `python3 -B tools/build.py --kernel-feature native-usb-hid-probe` (`NOS_BUILD_DIR` not recorded) | 0 |
| `python3 -B tools/run_usb_hid.py --case reports` (same build) | 0 (raw 33) |

All guest runs ended with raw QEMU exit 33. The xHCI and USB HID runs are the
regression for the `native-xhci-probe` to `if0-device-window` cfg rename in
`kernel/src/m2/paging.rs` and `kernel/src/m2.rs`.

## Native fixture results

| Case | Kernel feature | Guest | WAV (not committed) | Oracle |
| --- | --- | --- | --- | --- |
| normal | `native-virtio-snd-probe` | 100 TX completions, 48000 frames, RELEASE twice refused (not sent), status `0x0f`, quiesce acknowledged | 192044 B, sha256 `24c5e8d9398adb276c61c7cad0e16fb10f7480037f41e6de20a3d98dc1e12676` | 0 silent + 48000 A frames + 0 silent, byte-identical |
| no-device | same image, no `virtio-sound-pci` | `SND:ABSENT` | not created | no audio |
| access-platform | same image, `iommu_platform=on` | features `0x0000010379000000` (bit 33), `SND:OFFLINE:FEATURES`, BME 0, no DRIVER_OK | 44 B header only, sha256 `4872b61c768dff943f9e021453d816f06e35adc8edd88ef183301f03e31b94a5` | finalised, 0 frames |
| fence-reinit | `test-snd-fence-reinit` | 10 periods A, fence ACK, used rings stable, epoch 1, re-init, 100 periods B | 211244 B, sha256 `146822ccbfd8864189ba9bddca4f5d10ccc040c307cbc7cc8fe4b6ab62136ae3` | 4800 A (of 4800 submitted) + 0 silent + 48000 B |
| quarantine | `test-snd-quarantine` | 10 completions with TX outstanding (6240 frames submitted), zero-deadline fence ⇒ `SND:QUARANTINED`, publication refused, pool retained | 17324 B, sha256 `104558fede36242ed5eae065f097e1d358165c1f69f8c75f23b0b3e13fd3f381` | 4320 A frames, nothing else |

Runs from an earlier source fingerprint of this change produced the same WAV
hashes: the `-icount` profile output is deterministic. `report.json` holds the
per-case QEMU argv (not the harness command line), artifact hashes, guest SND
lines and oracle values. Its `ovmf_vars_sha256` (schema version 1) is the
post-run NVRAM state OVMF wrote during the boot, not the input template
(`OVMF_VARS_4M.fd` hashes to `5d2ac383…5d1e`); the raw report is kept as
recorded. Runner schema version 2 records the template as
`ovmf_vars_template_sha256` before launch and the post-run state separately as
`ovmf_vars_final_sha256`.

## Exploratory runs (non-acceptance, before the oracle was frozen)

1. PCM_INFO answered with used length 4 (the 36-byte records were written):
   the first build fenced with `SND:FAULT:USED:QUEUE=0:ID=0:LEN=4`. QEMU 8.2.2
   pushes `sizeof(virtio_snd_hdr)` for every control response. Changed once:
   that header-only length is accepted for PCM_INFO only (`TRANSPORT.md` §10).
2. Normal playback with the drain after STOP: 47520 of 48000 frames, exact
   and contiguous, the last period missing. QEMU disables the voice on STOP
   and drops buffered samples. Changed once: drain while RUNNING, then STOP.
   This run also confirmed that mixeng passes S16/48 kHz/stereo byte-identically
   and that underruns insert no silence. The oracle was then frozen unchanged
   from `TRANSPORT.md` §7.
3. In-flight fence (pattern A with TX outstanding, then re-init): after
   re-initialisation QEMU completed used id 0 with length 8 while head 0 was
   not outstanding; the driver failed closed (`USED_RING_VIOLATION`, raw exit
   35). The WAV showed A frames 0..5279, then B frames 1440..1919, 0..479 and
   0..671: popped elements that `virtio_snd_reset` keeps were replayed from
   reused slots. The accepted fence-reinit case therefore fences with no TX
   outstanding.

## Integration re-verification by the lead

The builder branch (`b033335`, base `9c707c8`) was merged into
`ccr-38238c58-g8y1l6` as `167c2bc` (additive conflicts in
`kernel/src/main.rs` and `tools/build.py`). The lead reran the native gates
on that exact commit, from a detached worktree outside the main checkout.
The source-v3 fingerprint was
`aadc6b7b0d7eb50a…` (full value in the local reports), with
`NOS_QEMU_MACHINE=pc-q35-8.2`. Every build exited 0, and every run exited 0
with its oracle passing:

- virtio-snd `normal`, `no-device` and `access-platform` (one
  `native-virtio-snd-probe` build), `fence-reinit` and `quarantine`: raw
  exit 33 each. The WAV SHA-256 values are byte-identical to the builder's
  runs (`24c5e8d9…`, `146822cc…`, `104558fe…`), so the `-icount` oracle is
  deterministic across the two source versions.
- Regressions for the shared `if0-device-window` gates and the merged guards:
  xHCI `noop`, USB HID `reports` and `reattach`, S3 `cycles`, the standard
  smoke and the `milestone-m2` smoke.

The lead also checked the PCM_INFO deviation against the pinned QEMU 8.2.2
source,
<https://gitlab.com/qemu-project/qemu/-/blob/v8.2.2/hw/audio/virtio-snd.c>,
file sha256 `99a13ef25d0729dd46e9947dea12986598b5b9242422a052fb05214f22e7794d`,
identical to the copy in the `qemu-8.2.2.tar.xz` release tarball (sha256
`847346c1b82c1a54b2c38f6edbd85549edeb17430b7d4d3da12620e2962bc4f3`) pinned in
`drivers/virtio-snd/TRANSPORT.md` §1. Line numbers refer to that file hash:

- `virtio_snd_handle_pcm_info` copies the records after the header with
  `iov_from_buf(..., sizeof(virtio_snd_hdr), pcm_info,
  sizeof(virtio_snd_pcm_info) * count)` at lines 247-251;
- `process_cmd` then calls `virtqueue_push(..., sizeof(virtio_snd_hdr))` at
  line 752, for every control request whose 4-byte header it could read (a
  request with a short header gets no response).

The deviation is recorded in ADR-H03-SND-FIXTURE-20261119.

`native-xhci.yml` now runs the five cases, the crate tests and clippy, and
`test_virtio_snd.py`, and preserves the five compact reports. The hosted run
of that job is the authoritative CI result.

## Review-fix re-verification (20 November 2026)

This section is separate from the results above, which stay as recorded. The
review fixes on branch `h03-review-fixes` (base `3ef2a4d`) change the kernel
fixture, the `virtio-snd` crate and the runner:

- every publication first checks the device used index against the requests
  outstanding before it (F8);
- the fence baseline is bracketed around the acknowledging status/BME
  read-back, the acknowledgement marker is written only after that check, and
  `finish_fence` needs a final matching observation after the 50 ms window
  (F11);
- the quarantine case requires TX to be driver-outstanding at the
  zero-deadline fence, in the kernel and in the runner (1-3 periods);
- an event buffer with a used length below 8 is counted, not fenced (F17), and
  one poll handles at most four event completions;
- `pcm::MAX_PERIOD_BYTES` is 2044, so a period always fits one TX slot.

The runner now pins `pc-q35-8.2`, refuses non-literal QEMU option paths,
fails output beyond 4 MiB, and writes report schema version 2.

Source fingerprint `56a20ca6658d56c93caeef839fc167852d8373a5e2d4cf8b7f716502dfff226b`
(`tools/common.py` `source_hash`, version 3), working tree on `3ef2a4d` before
the commit that records it. Same host, QEMU 8.2.2 and OVMF as above. The runs
were sequential from the worktree, which lies outside the parent checkout,
with `NOS_QEMU_MACHINE=pc-q35-8.2` unless stated, and a shared scratch
`NOS_TARGET_DIR`/`CARGO_TARGET_DIR` outside the worktree. Each `NOS_BUILD_DIR`
was set as an absolute path; it is shown relative to the worktree root:

| Command | Exit |
| --- | --- |
| `NOS_BUILD_DIR=build/rf-snd-normal python3 -B tools/build.py --kernel-feature native-virtio-snd-probe` | 0 |
| `NOS_BUILD_DIR=build/rf-snd-normal python3 -B tools/run_virtio_snd.py --case normal` | 0 (raw 33) |
| `NOS_BUILD_DIR=build/rf-snd-normal python3 -B tools/run_virtio_snd.py --case no-device` | 0 (raw 33) |
| `NOS_BUILD_DIR=build/rf-snd-normal python3 -B tools/run_virtio_snd.py --case access-platform` | 0 (raw 33) |
| `NOS_BUILD_DIR=build/rf-snd-normal env -u NOS_QEMU_MACHINE python3 -B tools/run_virtio_snd.py --case normal` (default machine) | 0 (raw 33, `pc-q35-8.2`) |
| `NOS_BUILD_DIR=build/rf-snd-fence-reinit python3 -B tools/build.py --kernel-feature test-snd-fence-reinit` | 0 |
| `NOS_BUILD_DIR=build/rf-snd-fence-reinit python3 -B tools/run_virtio_snd.py --case fence-reinit` | 0 (raw 33) |
| `NOS_BUILD_DIR=build/rf-snd-quarantine python3 -B tools/build.py --kernel-feature test-snd-quarantine` | 0 |
| `NOS_BUILD_DIR=build/rf-snd-quarantine python3 -B tools/run_virtio_snd.py --case quarantine` | 0 (raw 33) |
| `NOS_BUILD_DIR=build/rf-xhci-noop python3 -B tools/build.py --kernel-feature native-xhci-probe` | 0 |
| `NOS_BUILD_DIR=build/rf-xhci-noop python3 -B tools/run_xhci.py --case noop` | 0 (raw 33) |

Every virtio-snd report has `passed` true, `schema_version` 2, `qemu_machine`
`pc-q35-8.2`, `ovmf_code_sha256` `ad23261d…b083`, `ovmf_vars_template_sha256`
`5d2ac383…5d1e` (the `OVMF_VARS_4M.fd` template), `ovmf_vars_final_sha256`
`bf62103e…780f` (post-run NVRAM, equal to the historical `ovmf_vars_sha256`)
and `output_truncated` false. The WAV outputs are byte-identical to the
historical runs:

| Case | Guest | WAV sha256 | Oracle |
| --- | --- | --- | --- |
| normal (both runs) | 100 TX completions, second RELEASE refused, status `0x0f`, `SND:QUIESCE:ACK`, `SND:USED:STABLE` | `24c5e8d9…2676` (192044 B) | 48000 A frames, no silence |
| no-device | `SND:ABSENT` | not created | no audio |
| access-platform | `SND:OFFLINE:FEATURES`, BME 0, no DRIVER_OK | `4872b61c…94a5` (44 B) | finalised, 0 frames |
| fence-reinit | `SND:FENCE:ACK`, `SND:USED:STABLE`, `SND:EPOCH:1`, re-init, 100 periods B | `146822cc…6ae3` (211244 B) | 4800 A + 0 silent + 48000 B |
| quarantine | `SND:SUBMITTED_FRAMES:6240` (3 periods driver-outstanding), `SND:QUARANTINED`, `SND:PUBLISH:REFUSED` | `104558fe…f381` (17324 B) | 4320 A frames |

The raw reports remain in the ignored `build/rf-*` directories of that
worktree; they are not committed. This re-verification exercises the new
checks only on their passing paths: QEMU 8.2.2 cannot inject a stale used
index, a completion racing the fence acknowledgement, a short event or an
event storm, so those negative paths are host tests only (`queue_tests`,
`pcm_tests`, `tools/tests/test_virtio_snd.py`). Hosted CI for this source was
not run here. H03 stays open.

## Not run

- Hosted CI at the builder's commit (the job was added at integration).
- Physical hardware, audible output, latency, any other QEMU version.
- F6 (control timeout with a blocked backend), F13-F16 (no broker), F3 in a
  guest (QEMU refuses `streams` outside 1..10).
- F10 (`DEVICE_NEEDS_RESET` leading to a fence) was not induced natively or in
  host tests; QEMU 8.2.2 answers a repeated valid RELEASE with `OK`.
- Builder checkpoint (`b033335`): xHCI `fault` and `no-dmar`, and USB HID
  `readonly`, were not run; only xHCI `noop` and USB HID `reports` were rerun
  for the cfg rename. No S3 case was run. (USB HID `reattach` did not exist
  at that base; it arrived with H02-T3.)
- Lead integration (`167c2bc`): xHCI `fault` and `no-dmar`, USB HID
  `readonly`, and S3 `corrupt-facs` and `stale-token` were not run.
- `native-xhci.yml` runs all three xHCI cases and all three USB HID cases, and
  `native-power.yml` runs all three S3 cases. No hosted result for these is
  recorded here.
