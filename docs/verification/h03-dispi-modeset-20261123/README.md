# H03 DISPI mode-set evidence, 23 November 2026

Source commit: `576fe7ffb9c31507d5ded4b216b3a1f4e7102dc3` on
`claude/h03-dispi-20261123` (base `origin/main` `035b44e5`). Source
fingerprint (v3) of the native runs:
`bdfbfb1db1601c1f99cc13b3027770096a951a60432ed048c462348eeb00825e`.
Specification: `docs/implementation/NATIVE-DISPI-MODESET.md`.

Environment: local QEMU 10.2.1 (Debian 1:10.2.1+ds-1ubuntu3.3) with machine
type `pc-q35-8.2`, TCG single thread, `qemu64`, 1 CPU, 256 MiB; OVMF
`/usr/share/OVMF/OVMF_CODE_4M.fd` (sha256 `874f15a7…58aa7`). Hosted CI pins
QEMU 8.2 and has **not** run this fixture. Rust 1.90.0 for build and fmt.
`NOS_QEMU_MACHINE=pc-q35-8.2`, `CARGO_TARGET_DIR=$HOME/.cache/baryon-h03t4/target`.

## Step-1 probe (before the full fixture)

`probe-step1/serial.txt` was produced by a minimal, uncommitted probe build of
the same feature pairing (desktop-mode loader, `-device VGA,vgamem_mb=16`),
raw exit 33:

| Quantity | Measured |
| --- | --- |
| class-0x03 1234:1111 functions | 1 (class/subclass 0x0300) |
| GOP base / byte length | 0x80000000 / 0x384000 (1280×720×4) |
| Raw BAR0 (no sizing, no write) | 0x80000008: memory, 32-bit, prefetchable; address 0x80000000 = GOP base |
| DISPI ID (index 0) | 0xB0C5 |
| VRAM (index 0xA) | 0x100 × 64 KiB = 16 MiB |

The ports answered and BAR0 decodes to the GOP base, so the full fixture
proceeded. The committed kernel keeps these lines (`DISPI:GOP_BASE`,
`DISPI:BAR0_RAW`, `DISPI:ID`, `DISPI:VRAM`) so every run re-records them.

## Native runs on the source commit

| Case | Command | Raw exit | Result |
| --- | --- | --- | --- |
| Positive (stdvga) | `python3 -B tools/run_dispi.py --case positive` | 33 | passed, `positive/` |
| Negative (`bochs-display`) | `python3 -B tools/run_dispi.py --case negative` | 35 | passed, `negative/` |

Positive read-back per mode (`XRES:YRES:BPP:ENABLE:VIRT_WIDTH:VIRT_HEIGHT:X_OFF:Y_OFF`):
`640:480:32:65:640:6553:0:0`, `800:600:32:65:800:5242:0:0`,
`1024:768:32:65:1024:4096:0:0`, `1280:720:32:65:1280:3276:0:0`.
VIRT_HEIGHT is device-derived (16 MiB / line length).

QMP screendumps (not committed; dimensions and every pixel matched the host
oracle exactly):

| Mode | PPM sha256 |
| --- | --- |
| 640×480 | `64f91fcd230d0390dc5be22d586e84e60d823c4ff1d9dd54c9302be206064e4f` |
| 800×600 | `0c8ed05ba015cd22f3aaa37edcb37529e2e1c36a8c14bdc64572c3940b41dd89` |
| 1024×768 | `3d4ef2a8bfc35efcc1d53965b9a3a65b1ab614219c1e89b010f7880c7d3be1fe` |
| 1280×720 | `f4a21f12c7e0caa18a574058918ed0fab5910bb845f8dbb6677f3349aa015f27` |

Negative: `bochs-display` is 1234:1111 with a non-VGA subclass and no VGA I/O
ports; OVMF still set the 1280×720 GOP. The kernel printed
`DISPI:REFUSED:DEVICE` and `M1:FAIL:DISPI_REFUSED` after `KS-03:PASS`, with no
BAR0, ID, READBACK or MODE line, i.e. before any 0x1CE/0x1CF access.

## Regression and host checks

Text files are stored with LF line endings (the repository normalises
`.txt`); `SHA256SUMS` covers the committed bytes. The SHA-256 of each raw
CRLF `serial.txt` as written by QEMU is in the run's `result.json` `artifacts`.

`checks/exit-codes.txt` lists every command with its raw exit code; each log
is beside it. All ran sequentially on the source commit.

| Check | Exit |
| --- | --- |
| `python3 -B -m unittest tools.tests.test_dispi` (10 tests) | 0 |
| `python3 -B -m unittest discover -s tests -p test_ci_workflows.py` | 0 |
| `python3 -B -m unittest discover -s tests -p test_graphics_capture.py` | 0 |
| `cargo +1.90.0 fmt --all -- --check` | 0 |
| `python3 -B tools/build.py` (default) + `python3 -B tools/run.py --smoke` | 0 / 0, raw 33, `M1:PASS` (`smoke2.log`) |
| `python3 -B tools/build.py --kernel-feature graphics-g0` + `python3 -B tools/run_graphics.py` | 0 / 0, raw 33 |
| `python3 -B -u tools/build.py --kernel-feature native-display-probe` + `python3 -B -u tools/run_display_probe.py --qmp-transport pipe --vga-rom /usr/share/seabios/vgabios-stdvga.bin` | 0 / 0, raw 33 (`*2.log`) |
| `python3 -B tools/build.py --kernel-feature native-dispi-probe` | 0 |
| `python3 -B -m unittest discover -s tools/tests -p 'test_*.py'` | 1: 6 errors, all `Git environment overrides require a verified controller adapter` caused by the session's `GIT_EDITOR`; `env -u GIT_EDITOR` rerun of `test_agent_*.py` exits 0 (`host-agent-no-giteditor.log`) |

`smoke.log`, `build-display-probe.log` and `run-display-probe.log` (exit 1)
are the first attempts with `NOS_BUILD_DIR` outside the checkout, which
`run.py` and the FPU artifact builder reject by design. The `*2.log` reruns
used the ignored in-checkout `build/h03t4`.

## Not exercised

Physical hardware; other display adapters (virtio-vga, QXL, ramfb); hosted CI
on QEMU 8.2; byte-for-byte comparison of other profiles' kernel ELFs against
the base commit (only their native gates were rerun); 1920×1080; mode changes
requested by a ring-3 display service; EDID and monitor timing. A QEMU pass is
not H03 hardware acceptance.
