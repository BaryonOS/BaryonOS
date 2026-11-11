# Native Materiaal — first frontend building block

`no_std`, allocator-free and safe Rust. One real volume DocumentSession for
list/open/create/UTF-8 edits and streaming save; no browser broker or fake
storage status. Renderer with private 32×8 tiles, proportional antialiased system text,
100/200% layout, keyboard/mouse focus and a bounded app-local semantics tree
with stable node IDs, parent relations and explicit action descriptions.

This library contains no native display/input transport, compositor process,
screen reader or complete Materiaal environment. Only the kernel adapter may confirm input
origin/capabilities. All complete release criteria remain open.
Contract and adapter order: [NATIVE-MATERIAAL.md](../../docs/implementation/NATIVE-MATERIAAL.md).

A native caller constructs `Controller::new` with one `VolumeSession` and three
exclusive caller buffers; no copied active endpoint pairs. `refresh`
fetches the real list; `open`/`create` open a draft. `deliver(InputEnvelope)`
processes only INP1 events authenticated by the adapter. `save` starts;
`advance_save` performs at most one APPEND/COMMIT; `cancel_save` requests a real ABORT.
`close_clean` refuses dirty/busy and explicitly tries to close both endpoints.
`rebind_input(new_epoch)` preserves document/caret/selection and cancels old
click/drag state. The adapter may do this exclusively after new authenticated
input service grants; it is no capability grant or proven hardware reset.
Old PEEK/ACKs remain with the old endpoints and are never forwarded to the new
service. The first valid new input lifts the input warning;
an uncertain save outcome remains visible.

`Frame::new(controller.view())` borrows an immutable snapshot. Iterate
`frame.tiles()`, render each tile into its own `[u32; 256]` and blit only the
returned tight-packed rect. Record generation/epoch, drop frame, and call
`controller.presented(generation, epoch)` exclusively after confirmed transfer
of **all** tiles. Error/interruption/new generation leaves repaint open. The
first version redraws the full frame; there is no latency claim or
atomic page flip. Root must measure/integrate this on the reference profile.

`Frame::semantics` provides a bounded tree with roles, stable IDs,
parent relations, physical bounds, focus, read-only/enabled, text value,
byte selection and real save progress. `activate` and `set_value` only describe
which UI action fits a node; they call nothing and grant no
rights. This is preparation for a native accessibility broker, not
IPC, a working screen reader or an OS-wide accessibility interface.
Font fallback never changes stored UTF-8. Code point navigation is not yet
grapheme shaping, IME or general language/font support.

From the repository root with the existing toolchain environment:

```sh
python3 -B ui/native/verify.py
```

That runs the real host scenarios, Clippy, fmt, scalar-none build and object scan.
`verification/checks.json` binds files, the shared CSS token source and
all other existing source-v3 inputs to the built archives. The root image harness includes ui/native, ui/surface-protocol and tokens.css
in source-v3. This source binding does not replace native execution acceptance.

The host harness uses the actual capability engine and volume Service on
ordered RAM sectors. 18 scenarios cover, among other things, real PS2 bytes,
commit/reopen, ABORT, unknown commit outcome, deny/revoke, conflict/I/O, UTF-8,
full/capacity64KiB, dialog focus, stale input/epoch and actual raster buffers.
The large existing engine/service host structures get an explicit
16-MiB test thread stack; this does not widen any native user stack. The raster fixture test
writes exclusively synthetic host images under
`build/native-materiaal-visual/`; those images have been viewed by this agent and
stay outside Git. They are not QEMU screenshots or a user test. Compact
list pagination has real pointer/keyboard actions, visible focus and
semantic names; even at 320 logical pixels material remains visible.
Native stack/arena
budgets, UI operation and cold encrypted storage have not yet been demonstrated.

Font source, chosen features and notices are in [THIRD-PARTY.md](THIRD-PARTY.md).
Native colours, vector icons and system titles share the standalone
[graphics layer](../graphics/README.md) with the compositor. Existing compact
spacing/radius tokens continue to come from [tokens.css](../prototype/tokens.css).
The prototype is not used as runtime and does no work on behalf of the OS.
Document text deliberately keeps its existing monospaced grid and caret metrics;
proportional labels never change the document bytes or selection positions.

The shared font is a bounded, offline-generated Open Sans atlas with
real 100/200% rasters. The native app loads no TTF files or host fonts.
The RO segment limit is explicitly 512 KiB; startup, RW and code segments remain
separately bounded and W^X. `set_compositor_cursor(true)` exclusively suppresses
the local pointer drawing; hover/focus and app input keep their actual
positions. The compositor remains the owner of the trusted window border.

The additional test `render_desktop_foundation_at_reference_scales` draws
1280×720 at 100% and 1920×1080 at 200% via the same tile renderer. With
`OS_FOUNDATION_FIXTURE_DIRECTORY` that test writes inspectable PPM images
to an existing empty directory. These are host fixtures, not native evidence.

## Session route and desktop (2 November 2026)

The opt-in kernel profile `native-system-session-preview` gives the native app
startup word648=1. `Controller::enable_vm_desktop` shows user choice,
a recognisable sign-in form and an explicit public VM guest session,
followed by taskbar, local program search and real Materiaal navigation.
Esc in the Materiaal list returns to the desktop; editor/dirty dialog
keep their existing protection of unsaved work. Existing native
Materiaal gates do not use this profile.

`session.rs` binds replies to request/epoch/account/method and separates
authentication from activation. The native default `UnavailableBroker` refuses
every real password; fingerprint/FIDO2 are not connected. This is no
secured account management or compositor lock: the public guest remains the same
synthetic app domain with VM rights granted in advance. Do not use real data
or production credentials in this preview. See the
[session contract](../../docs/implementation/SESSION-AND-DESKTOP.md).

The start surface has a clear header, a recognisable guest status and one
primary action to Materiaal. The card shows the real library state:
a not-yet-loaded or empty library, a storage error or an existing
document.
It invents no recent documents, network status or energy data. The
compact layout keeps the card clear of the navigation bar; the wide bar names
the Start action explicitly. This remains a VM preview on top of the existing
Materiaal app, not a complete desktop or authenticated user session.

The compact layout also applies at less than640 logical pixels width,
not only at less than420 pixels height. Short card texts remain within
the card at the minimum viewport; a full object list or other
non-storage error is not presented as a storage failure.

Additional learning source: OSDev, *Drawing In a Linear Framebuffer*, W004597,
revision30599 (21 May 2026). The existing UEFI/GOP and capability transports
remain authoritative; no tutorial code has been copied.
