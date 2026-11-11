# G0 — first native graphics foundation

28 October 2026, follow-up to `7b4945f`. This contract precedes execution.
G0 is an opt-in hardware/input trial, not a full M7 desktop or compositor.
The existing native storage and the browser prototype remain separate tracks.

## Result and invariant

An own kernel runs after ExitBootServices on its own M2 page tables and draws
via the framebuffer designated by UEFI. A bounded PS/2 decoder processes
actual QEMU keys into a local RAM line. The screen explicitly states
that this is the graphics foundation and not yet a desktop. No simulated files,
storage confirmation, apps, NAS or AI in this trial.

- Standard boot keeps the exact BootInfo 1.1 contract. Only coupled
  `graphics-g0` features in loader and kernel use opt-in BootInfo 1.2:
  header 128 bytes, flags 15 and a fixed validated descriptor from byte
  128 within the same preserved BootInfo page. Old profiles get no GOP.
- The loader configures/reads GOP before the last memory map/freeze; after
  EBS there are no GOP, console or other firmware calls.
- The kernel copies metadata before use and checks pixel type, geometry,
  stride, length, physical address width, overflow and overlap. Only RGB/BGR
  32-bit pixels; no blt-only or arbitrary bitmask implementation.
- Framebuffer pages get exclusively a separate, bounded supervisor
  RW/NX/UC mapping. No broadening of the virtio MMIO API and no mapping of RAM
  or reserved normal frames as framebuffer. PAT is checked.
- `ui-core` is own safe Rust `no_std`, without heap or external dependencies.
  Clipping and typed pixel geometry bound every write position. A
  private row buffer avoids a large backbuffer; only visible pixels
  go to the framebuffer via volatile stores. Padding stays intact.
- PS/2 Set1 input is a bounded polling trial on one QEMU CPU, without IRQ1
  modification. Releases, modifiers and extended prefixes produce no arbitrary
  text. The first text line is bounded ASCII; no Unicode/IME claim.
- ESC ends the test with its own G0 marker and raw exit 33. No automatic
  green test based solely on a firmware image or a captured screenshot.

## Acceptance and boundaries

Host tests check invalid framebuffer geometry/overflow, clipping,
padding/canaries, colour order, invalid scancodes, modifiers, backspace and
a full text line. The native trial must deliver real post-EBS/M2/G0 markers, injected
QMP keystrokes, framebuffer images before/after input and a correct exit.
QMP exclusively controls the own QEMU child without network to the guest.

A visibly typed line is not storage, an isolated app, an input broker,
an accessible full shell or a user desktop. That product integration
then requires a separate ring-3 display/input service and capability boundary.
The temporary kernel bootstrap contains exclusively bounded diagnostic display;
object, app and storage policy do not go into it.

Owner of pure core/loader/boot metadata/native adapter: G09. Root owns
workspace/lockfiles, build/QMP harness, documentation and Git. Kernel Cargo goes
via K08, so that one writer manages that shared interface. Rollback: separate
checkout of `7b4945f`; existing boot profiles remain selectable.

Primary sources: UEFI Forum, *UEFI Specification 2.11*, §12.9 Graphics Output
Protocol: https://uefi.org/specs/UEFI/2.11/12_Protocols_Console_Support.html;
Intel, *64 and IA-32 Architectures SDM*, page tables/PAT according to ADR-026.
Consulted source versions, actual tests and still-missing verification go
in PROJECT-STATE.md. Project limits are not requirements from the UEFI specification.
