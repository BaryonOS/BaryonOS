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

Native (in `<CHECKOUT>`, each build with its own `NOS_BUILD_DIR`, shared
`NOS_TARGET_DIR`/`CARGO_TARGET_DIR`, sequential):

| Build `python3 -B tools/build.py ...` | Run | Build / run exit |
| --- | --- | --- |
| `--kernel-feature native-virtio-snd-probe` | `tools/run_virtio_snd.py --case normal`, `--case no-device`, `--case access-platform` | 0 / 0, 0, 0 |
| `--kernel-feature test-snd-fence-reinit` | `tools/run_virtio_snd.py --case fence-reinit` | 0 / 0 |
| `--kernel-feature test-snd-quarantine` | `tools/run_virtio_snd.py --case quarantine` | 0 / 0 |
| (standard) | `tools/run.py --smoke` | 0 / 0 |
| `--kernel-feature milestone-m2` | `tools/run.py --smoke --milestone m2` | 0 / 0 |
| `--kernel-feature native-xhci-probe` | `tools/run_xhci.py --case noop` | 0 / 0 |
| `--kernel-feature native-usb-hid-probe` | `tools/run_usb_hid.py --case reports` | 0 / 0 |

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
hashes: the `-icount` profile output is deterministic. `report.json` holds the per-case command,
artifact hashes, guest SND lines and oracle values.

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

## Not run

- Hosted CI (`.github/**` unchanged; no workflow job runs these cases yet).
- Physical hardware, audible output, latency, any other QEMU version.
- F6 (control timeout with a blocked backend), F13-F16 (no broker), F3 in a
  guest (QEMU refuses `streams` outside 1..10).
- The other xHCI cases (`fault`, `no-dmar`, `readonly`); only `noop` and USB
  `reports` were rerun for the cfg rename.
