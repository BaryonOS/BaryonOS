# Native Bochs DISPI mode-set fixture (H03)

This fixture is an execution test for criterion H03 (native device and display
modes, usable resolutions) on one named development profile: QEMU stdvga
(`-device VGA,vgamem_mb=16`, PCI 1234:1111 class 0x0300), `pc-q35-8.2`,
qemu64/TCG/1 CPU/256 MiB, OVMF, and the paired 1280×720 `desktop-mode` GOP
loader. It is QEMU-only evidence. It is not a display driver service, a
compositor mode switch, physical-hardware acceptance or acceptance of the
mandatory VM-FR and 1920×1080/200 % full-release profiles.

## Profile and ownership

| Item | Value |
| --- | --- |
| Kernel feature | `native-dispi-probe = ["graphics-g0", "dep:pci-core"]` |
| Loader feature | `desktop-mode` (bounded GOP QueryMode selection of 1280×720) |
| Source | `kernel/src/dispi.rs`; additive `mode_extent`/`fill_mode` in `kernel/src/framebuffer.rs` |
| Runner | `python3 -B tools/run_dispi.py --case positive|negative` |
| Host tests | `python3 -B -m unittest tools.tests.test_dispi` |

`tools/build.py --kernel-feature native-dispi-probe` selects the `desktop-mode`
loader and rejects any other loader. `kernel/src/main.rs` has a
`compile_error!` that excludes the profile together with `milestone-m3`,
`native-smp`, `native-smp-legacy`, `native-rtc`, `native-acpi-s5`,
`native-xhci-probe` and `display-input`. `m2.rs` dispatches to
`dispi::run()` in place of the G0 renderer; every other profile's dispatch,
GOP descriptor and mapping are unchanged.

## Invariants

- IF0, one CPU, CPL0 only: no syscall, no ring 3, no interrupt handler.
- PCI access is the read-only `pci_core::enumerate` scan plus one raw read of
  BAR0 at offset 0x10 and one raw read of the command register at offset
  0x04. There is no BAR sizing, no command-register write, no bus mastering
  and no DMA.
- Exactly one class-0x03 function may exist, and it must be 1234:1111,
  subclass 0, programming interface 0, segment 0, bus 0. Otherwise the kernel
  prints `DISPI:REFUSED:DEVICE` before any access to ports 0x1CE/0x1CF.
- BAR0 must be a 32-bit memory BAR (bits 2:0 zero) whose address bits
  (`BAR0 & !0xf`) equal the GOP `frame_buffer_base`. Bit 3 (prefetchable) is
  ignored: it is allowed but not required. The measured value was
  0x80000008. The GOP mode must be 1280×720 with pixel format 1 (BGRX).
- The bound function's command register must already have I/O-space decode
  (bit 0, ports 0x1CE/0x1CF) and memory-space decode (bit 1, linear
  framebuffer) enabled by firmware; the kernel prints `DISPI:COMMAND:` and
  otherwise refuses with `DISPI:REFUSED:DECODE` before any 0x1CE/0x1CF
  access. It never sets these bits itself.
- The DISPI ID must lie in 0xB0C0..=0xB0C5. VRAM (index 0xA × 64 KiB) and the
  existing GOP mapping must both cover every allowlisted mode at 32 bpp.
- Modes come only from the const allowlist 640×480, 800×600, 1024×768,
  1280×720 (in that order, so the first mode differs from the GOP mode).
- Mode set: ENABLE=0, XRES, YRES, BPP=32, VIRT_WIDTH=XRES, X/Y offset 0,
  ENABLE=0x41 (enabled + linear framebuffer; NOCLEARMEM is not requested, so
  QEMU is expected to clear VRAM, which this fixture does not measure). Every
  written register is read back; any mismatch, or VIRT_HEIGHT below YRES,
  ends with `M1:FAIL:DISPI_READBACK` (exit 0x11). VIRT_HEIGHT is derived by
  the device (VRAM / line length) and is checked only as a lower bound.
- Pixel stores use `framebuffer::fill_mode`: the extent width×height×4 is
  validated once against both the GOP `mapped_len` and `byte_len` before the
  first store, and each store's offset+4 is checked again. No page-table
  entry is created; the existing supervisor RW/NX/UC GOP window is reused.
  Before the first store the kernel checks counterexamples (1281×720,
  1280×721, zero width, zero height) and fails if any is accepted.

## Pattern and handshake

For mode index `m` (0..3) and pixel `(x, y)` the 0x00RRGGBB colour is:

```text
R = (x*3 + m*0x40) & 0xff
G = (y*5 + m*0x25) & 0xff
B = ((x>>8)*0x40 + (y>>8)*0x10 + (x ^ y)) & 0xff
```

The runner computes the same pattern independently and compares every pixel
of a QMP `screendump` PPM, after checking that its dimensions equal the mode.
A wrong stride, a stale frame of another mode or one changed pixel fails.

After drawing a mode the kernel prints `DISPI:MODE:<w>x<h>:SET` and polls the
i8042 (reusing the G0 `initialize_input`/`poll_scancode`) until the set-1
make code 0x31. The runner sends a real QMP `n` key only after its screendump.
After the last mode the kernel prints `DISPI:PASS` and `M1:PASS` and exits
with 0x10 (raw QEMU exit 33).

## Negative case

`--case negative` boots the same images with `-device bochs-display`
(1234:1111; QMP `query-pci` measured class 0x0380 and memory BARs only; it is
not a VGA device, so the legacy 0x1CE/0x1CF ports are not expected to decode). OVMF still provides the
1280×720 GOP. The runner requires the exact sequence `KS-03:PASS`, the GOP
lines, `DISPI:REFUSED:DEVICE`, `M1:FAIL:DISPI_REFUSED`, no READBACK or MODE
line, and raw exit 35.

## Evidence boundary

Evidence for one source commit is in
`docs/verification/h03-dispi-modeset-20261123/`. The local runs used QEMU
10.2.1 with the `pc-q35-8.2` machine type; hosted CI pins QEMU 8.2 and has not
run this fixture. Not exercised: physical hardware, other display adapters,
EDID or monitor timing, mode switching by ring-3 services, the compositor and
1920×1080.

## Primary sources

- QEMU project, `hw/display/vga.c` and `include/hw/display/bochs-vbe.h`, version
  v8.2.2: the DISPI index/data ports, the ID range 0xB0C0–0xB0C5, the VRAM
  register (64 KiB units), `VBE_DISPI_ENABLED`/`VBE_DISPI_LFB_ENABLED` and the
  derived VIRT_HEIGHT. <https://gitlab.com/qemu-project/qemu/-/blob/v8.2.2/hw/display/vga.c>,
  <https://gitlab.com/qemu-project/qemu/-/blob/v8.2.2/include/hw/display/bochs-vbe.h>.

These sources describe the emulated device; they are not execution evidence.
Execution evidence is recorded in docs/verification/h03-dispi-modeset-20261123/: the committed artifacts are local, and the hosted run is recorded there separately.

## Refusal grammar

A refusal is exactly one line `DISPI:REFUSED:<TOKEN>` followed by
`M1:FAIL:DISPI_REFUSED` (exit 0x11). The closed token set, in `bind()` order,
is `GOP`, `PCI_SCAN`, `DEVICE`, `PCI_BAR`, `BAR0`, `PCI_COMMAND`, `DECODE`,
`ID`, `EXTENT`. `tools/run_dispi.py::REFUSALS` holds the same tuple; a host
test compares it with the kernel source, and the runner rejects any other
token.
