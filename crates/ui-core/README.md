# ui-core — graphics bootstrap

FR-U01 adds separate physical-input building blocks: `Set1Keyboard`, `Ps2Mouse`,
`Cursor` and `InputRouter`. They do not replace the G0 ASCII API below.
The router retains focus generations and prevents held keys, modifiers or
buttons from transferring to newly focused content. This is pure `no_std`
logic; actual device I/O, capability enforcement and native compositor
acceptance remain separate steps. Contract and reproducible checks:
[NATIVE-INPUT.md](../../docs/implementation/NATIVE-INPUT.md).

Independent, dependency-free safe Rust `no_std` code. No pointers, firmware,
MMIO, allocator, file access or app permissions. This is a software building
block for a graphical OS experiment; not a compositor, desktop, storage
interface or native M7.

`Geometry` checks 32bpp RGB/BGR, a maximum of 1920×1080, stride of at most
4096 pixels and a complete buffer. `Canvas` draws only in private RAM and
leaves scanline padding and bytes outside its validated slice untouched.
Signed rectangles are clipped before drawing. The original 5×7 debug glyphs
render lowercase as uppercase; unsupported characters become `?`. This is
not a complete Unicode, font, scaling or accessibility implementation.

`render_debug_row` uses only `width*4` bytes (maximum 7680). The hardware
adapter must first render a row in private RAM and then copy it with suitable
volatile writes to the separately validated framebuffer. Creating a RAM slice
pointing to MMIO is not a permitted substitute. `draw_material_debug` renders
the same scene on a RAM canvas. Both explicitly display **GRAFISCHE BASIS -
GEEN DESKTOP** (graphics foundation, not a desktop) and **ALLEEN RAM - NIET
OPGESLAGEN** (RAM only, not saved). There are no fictitious file actions or buttons.

`Keyboard` decodes the bounded Set1 QEMU experiment: make/break, both Shift/Ctrl/Alt,
CapsLock, E0 navigation and PrintScreen; E1/Pause consumes six bytes without text.
The host adapter checks controller status/parity/timeout/AUX and calls `reset()`
on loss. No USB, locale, input-method or hardware claim. `TextLine<N>` stores
only ASCII 32–126 in a fixed array; releases and Ctrl/Alt insert nothing.
A full buffer returns `Full`; Enter returns `Submit` without claiming storage.

Host tests check geometry, overflow rejection, guards/padding, clipping,
glyph landmarks, bounded changed screen regions, modifier/prefix sequences,
text limits and random scan bytes. They do not demonstrate hardware execution.
