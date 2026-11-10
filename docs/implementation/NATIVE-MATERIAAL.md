# TASK-NATIVE-MATERIAAL — first native frontend chain

## REV-APP-1 — current paint and correct C entry

The independent app review reproduced a real failure: after a completed
paint, Ready400 remained positive when only tile 0 of a newer generation
had completed. Controller.presented was itself bounded, but the old
measurement flag was incorrect. Every new paint now withdraws Ready;
evidence also checks the current dirty state/generation.
LastPresentedGeneration changes only after that same current generation/epoch
reaches Controller.presented through all actual tile ACKs. Ready400 is
published last, after first writing 0 and the other measurement words, so
kernel pre-emption cannot read a half-published group as successful.

The other targeted review fix replaces the extra entry CALL with CLD, an
actual RSPmod16==8 guard and JMP to the non-returning C function; the kernel
already supplies SysV C-entry alignment. The actual ELF is checked for guard
bytes and a direct JMP without an extra CALL. Nineteen host scenarios pass,
including the previously failing counterexample. Compilation/ELF inspection
remain distinct from native execution.

Additional observer words 504 screen (1 Materiaal/2 Editor/3 Confirm),
512 draftDirty, 520 draftLength, 528 draftFNV, 536 objectID (MAX if absent),
624 LastPresentedGeneration, 632 scale (1 or 2) were agreed with root and
derived from the actual Controller/DocumentSession. This changes no grants,
input policy, draft content or storage. The native build uses a private source
copy with hashed central SDK1.8 inputs; no shared SDK files were changed for
this fix. The separate
`sdk/examples/materiaal/verification/review-fixed.json` binds the failing/
passing evidence and ELF. The older pre-native.json remains identifiable as
the pre-review checkpoint.

The bounded GUI trial also publishes 640 F12PressCount: only an actual
authenticated INP1 event PhysicalKey::F12/Press increments this checked
counter. Repeat/Release or other keys do not count; exhaustion returns an
error and does not wrap. This is not a shutdown function or new grant. Only
after actual Saved→List→Reopened and a real QMP screenshot may the probe use
a new F12Press to let the separate CPL3 controller execute its existing
StopRPC. A late Enter release therefore cannot set the shutdown barrier early.

## TASK-MATERIAAL-NATIVE-APP — separate native app, SUR1 freeze

31ab300 build baseline: main 985ed4d plus frontend be0/d5 and display/input
b2ad. Owner native_ui_apps: ui/surface-protocol, sdk/input-client, new example
sdk/examples/materiaal and this document. Root owns the separate compositor,
bootstrap/central ABI/build/workspace locks. Additional targeted allocation:
ui-core mouse.rs for a public checked button constructor and the app's
Controller for a pointer-only cancellation hook. No other existing files.

App slot 3 receives no DISPLAY/INPUT/Block/Entropy/Key/Network capability.
Compositor 4 has DISPLAY, source 5 INPUT and policy 6 only its own IPC;
actual PIDs remain distinct and come from kernel metadata/startup. Init 0
remains supervisor. The app has three task-channel pairs (volume, INP1,
surface) plus the existing supervisor controlRECV/registrySEND, within the
existing quota of four receive endpoints.

The agreed [SUR1 contract](../../ui/surface-protocol/README.md) has a 48-B
header, 208-B chunk data, at most five chunks per tile and one transaction ID
for BEGIN/CHUNK/COMMIT. Only a complete validated COMMIT plus an actual
DisplayClient blit may receive a positive acknowledgement. Root builds the
ring-3 compositor against the pure
Receiver/CommitReady/commit_with(token,now_tick,callback) interface; no kernel
rendering/GUI policy. INFO reports public screen geometry and the assigned
Rect, not a physical base/stride. The app draws locally; the compositor
validates/adds the assigned offset and protects its mandatory system border.
Canonical row-major grid indices, request/epoch/frame/offset/length/pixel
checks and timeout bound staging.

Input remains **global** over actual GOP width/height. The native adapter
validates the separate 96-B INP1 contract, not a 32-B hardware record. Within
assignedRect it only subtracts the x/y offset; outside it, cancel_pointer
clears only pressed, drag and hover/cursor, without changing draft, keyboard
focus or generation. A release outside Save is not clamped into a click;
re-entry with a button down creates no new press. MouseButtons::checked
rejects unknown bits while retaining the existing private routing constructor.

Startup remains compatible: docEpoch32, docRecv72, docSend80, registrySend88,
controlRecv96, readonly112, volumePID168, appPID176, initPID184. New fields:
surfaceSend544, reply552, compositorPID560, surfaceEpoch568, inputSend576,
inputReply584, policyPID592, inputEpoch600, inputClientID608, scale616.
Counters400..512/failure240 are updated only after actually observed
operations, not as a prepopulated Saved/Presented/Ready trial.

Three 64-KiB caller buffers reside in owned BSS/RW, not on the 64-KiB stack.
The new five-segment linker retains startup40010000 and places
rodata+.data.rel.ro before the RW wildcard in a separate R+NX area 40020000
(max 256 KiB), RW/BSS40070000 and code400b0000. This remains within existing
user-image/quota/W^X boundaries. An actually built ELF and user::parse are
mandatory; the native QMP/image/disk oracle follows only at root integration.
There is no claim of persistent encrypted keys/B02, complete Materiaal, a
screen reader or a full test release.

SUR1 review found and reproduced a known expired Ready token after malformed
traffic. The regression failed on old source; expiry now precedes decoding,
and a fresh native monotonic tick is mandatory at commit admission. Without
a valid deadline the callback must not run, and no positive reply or frame
completion may follow. The deadline concerns admission before blit, not a
vsync promise across pre-emption or supposed rollback of written pixels.
Source, failing/passing evidence and independent re-review are at
`ui/surface-protocol/verification/`.

### Separate app adapter and built ELF

`sdk/examples/materiaal` now contains the actual no_std native caller. All
three task pairs are transferred once to their typed owner; app startup checks
valid, distinct handles and distinct expected PIDs. The app owns no hardware
capability and fabricates no Received envelopes. Each bounded iteration
performs at most four PEEK/ACK pairs, one actual upload step and one tile
transaction. A snapshot is presented only after all ACKs for the same current
generation/epoch. On input/surface loss, Ready is cleared; draft bytes are
preserved and a lost COMMIT reply does not produce Saved.

The supervisor protocol remains the existing SYSTEM tag: READY1 is only
metadata/ELF ready, GO2 starts service use, STOP5 is an explicit supervisor
request, STOP_REFUSED6 replies when dirty/saving, and DONE3 follows only
clean/idle plus cleanup. The exact kernel sender, zero capabilities and a
1-B payload are mandatory. No silent discard, autosave or change of PID/epoch
authority through payloads.

Evidence fields: 400 current UIReady (complete current paint + actual INP1
response), 408 complete frames, 416 actual tile ACKs, 424 delivered keys,
432 mouse events, 440 last actually acknowledged saveRevision, 448 bytes,
456 FNV checksum, 464 actual input ACKs, 472 errors, 480 actual save COMMIT
ACKs, 488 supersededOffers, 496 stop refusals. Heartbeat8/failure240 remain;
an unchanged save is not a new COMMIT ACK. These are measurement fields,
not authority or cryptographic properties.

Twenty combined host scenarios use one actual capability engine, the existing
volume-v7 service, input-core and SUR1. Tested cases are create/edit/save/
reopen, cancellation before and after APPEND, global pointer offset/outside/
held re-entry, 100/200%, read-only, focus regrant with held Control, large
input, I/O, lost input/tile/COMMIT replies, stale half-frames and explicit
cleanup. Ordered RAM sectors and the display recorder are host counterparts;
no native input/display/disk evidence. One additional test applies the actual
kernel user::parse to the actually built x86_64-unknown-none executable,
not a fixture ELF.

This ELF has five monotonic W^X LOADs and 119 image pages. Three 64-KiB buffers
are at 0x40071000 in BSS/RW+NX, not on the stack; read-only data including
font relro is 191684 B, code 84089 B at this checkpoint. The monomorphised
19781 instructions contain no SIMD/FPU or host malloc backend. Exact hashes,
command logs and the actual temporary central SDK1.8 input are in
`sdk/examples/materiaal/verification/pre-native.json`. The temporary patch
is removed from shared SDK files before commit; root must actually integrate
the same central SDK1.8 source and rebuild. user::parse/Clippy/build do not
prove native execution, maximum stack depth or GUI latency. QMP, the actually
rendered UI, cold reopening and encrypted key persistence remain open.

30 October 2026; baseline `61ea727`. Owner `native_ui_apps`: only
`ui/native/**` and this contract. Root owns kernel/ABI/startup capabilities/
build/integration; `git_build_audit` investigates display/PS2 transport;
the memory agent owns m3.rs. Allocator commit `42d4bc9` remains separately
preserved. All 62 full-release criteria remain open; this slice is not a
complete Materiaal environment.

## Checked baseline and boundary

G0 has actual GOP/framebuffer kernel mapping and keyboard polling, but is a
kernel trial with an uppercase debug font, not a user environment. Graphics
bootstrap is incompatible with m3 on this baseline. `graphics::write_pixels`
is already bounded to 80 canonical pixels and reusable in the hardware
adapter. `ui-core` provides actual Set1/PS2-ID0 events and focus routing;
`input-core` provides authenticatable INP1 policy but no native service loop
yet. Volume DocumentSession v7 has been tested natively; this frontend uses
that existing implementation, not a browser broker or separate storage
simulation.

The frontend owns exactly one DocumentSession and its endpoint pair. No
block/entropy/key/network capabilities, firmware/device I/O or framebuffer
pointer. An authorised compositor adapter owns the display capability and
translates only private RAM tiles. The UI grants no authority. A renderer/
controller host test does not prove a native input/display boundary.

## Agreed adapter form (not yet an established kernel ABI)

- Public viewport: pixel width, pixel height and a non-zero display epoch;
  at most the existing 1920×1080 baseline. No physical base or hardware stride.
- Tiles at most 32×8, 256 little-endian u32s / 1024 bytes. Each pixel is
  `0x00RRGGBB`; payload bytes B,G,R,0. The last row/column is shorter. The
  kernel adapter checks the complete rectangle, size and upper bytes before
  the first hardware write. No MMIO slice to Rust Canvas.
- The real input adapter validates the INP1 sender from kernel metadata, tag,
  epoch, client, request ID, canonical fields and focus generation. Only then
  are typed input-core events passed to this frontend. Apps cannot forge a
  source/focus/lifecycle message. PEEK is ACKed only after processing, with
  exactly the received event number; an unsuccessful send consumes nothing.
- Focus/source/queue cancellation ends drag/modifier interaction but preserves
  the draft and selection. A new focus generation inherits no held action.
  A service or display epoch change requires new grants/a complete repaint;
  an old presented frame must not clear new damage.
- `Controller::rebind_input(new_epoch)` is an explicit adapter step after new,
  independently authenticated input-service grants. It retains DocumentSession,
  draft, selection and display epoch, clears only old input numbering/click
  drag, and waits for an actually received valid event from the new epoch.
  This creates no endpoint/capability; old PEEK/ACKs remain closed. Retention
  of physical held-key knowledge during a real service restart remains
  separate source/supervisor acceptance, not a claim of this method.
- A frame reads only its own immutably borrowed view data and produces private
  tiles. Only acknowledgement of the same frame generation **and** viewport
  epoch can approve damage as handled. A failed/interrupted blit leaves damage
  pending.

## First user chain and actual document states

Materiaal displays results from the actual volume list, opens the same object
in the text editor and saves through DocumentSession begin_save/save_next.
Three caller buffers remain exclusively owned by that session. The first
native fixture uses 4096 bytes; the core adds no permanently smaller limit
to the existing 64-KiB volume boundary. Larger caller buffers require a
confirmed owned arena and cannot silently be placed on the 64-KiB user stack.

Caret/selection move at UTF-8 code-point boundaries; document bytes are neither
normalised nor replaced by font fallbacks. The first physical layout is
US-Set1; input methods/grapheme shaping/other layouts remain open. Only
working actions are drawn: open, new, back, save and actual cancel. No
preloaded object list or success based solely on a click. Closing a dirty
document asks to save/discard/cancel. During streaming save, edits are
blocked, progress comes from actual APPEND acknowledgements and cancellation
uses ABORT. Only Saved{revision} may confirm storage status. Conflict, full
storage, denial and uncertain commit preserve the draft. Uncertain work is
not automatically resent. Cold encrypted storage remains open until the
separate persistent-key/B02 chain is ready.

## Maintainable design, text and accessibility

Native colour tokens/radii/spacing read the single existing source
`ui/prototype/tokens.css` at compile time; no conflicting second colour table
or font network request. Typography sizes and control/focus rules follow
PRODUCT-SPEC. Scaling to 100/200% changes geometry and text together;
focus/status receive semantic roles and visible labels, so colour is not
the only meaning. This slice does not animate. That respects reduced motion
but does not yet prove a system-wide setting or screen-reader operation.

The CPL3 Materiaal app exposes a bounded semantic tree of its own current
controller state through `App::semantics`. Node IDs remain stable for screen
controls and use the volume object ID for list items; parent relationships
identify the window, list and dirty-close dialogue. Roles, labels, physical
bounds, focus, editability, text selection and actual storage status come
from the same frame source. The controller also offers `activate_semantic`
and `set_semantic_value`: before every action they rebuild the current tree
and reject missing, disabled or non-actionable nodes. The caller must also
provide the controller generation from the visible snapshot; a stale
generation is rejected before dispatch. Existing Enter/Space, PageUp/PageDown
and Escape actions use the same checked route. A semantic text replacement
changes only the current draft; saving remains a separate explicit action.
These methods remain inside the app and grant no capability. An undersized
output buffer fails without a partial snapshot. This is still an app-local
API, not IPC to a screen reader, speech output or an OS-wide accessibility
interface. Host tests check keyboard dispatch, stale/disabled actions,
save/reopen, tree structure, maximum list page, buffer boundary, editor and
dirty-close dialogue; they do not prove a native screen reader or user
acceptance.

The separate `--case keyboard` QMP route was executed as a baseline on frozen
source `74d15ac09ac61c8d47a882c3c982711af1a9d891`, fingerprint v3
`24d5f4c112924cfe45681de3fcf3a92f62087e29524f15c74e14d83620891abd`.
In QEMU 10.2.1 (`pc-q35-10.0`, TCG, `qemu64`, 1 CPU, 256 MiB), the run
completed six rendered phases in 136.8 s: ready, editor, dirty, saved, list
and reopened. The QMP trace contains 21 `send-key` requests and six screendumps;
the guest acknowledged 42 keys and zero pointer events. Save/reopen remained
within the same VM session. The disk oracle found object 0, revision 1,
generation 1, 14 bytes with the expected hash and intact canaries. QEMU raw
exit 33 was accepted as normal probe termination (`passed=true`, no timeout,
M1 and cleanup passed).

This is baseline operation only, not execution of the new semantic tree from
`f6ec180103779dfdb1f50088c7a9cfd7338fb955`. The frozen
`ui/native/src/render.rs` hash was
`89c7c99010aa1ca284fdf914543d38ffd6ea0c1891f4c07102c78925d13ce2d9` and does
not contain the newly added stable `SemanticId`, parent and action fields;
the current leaf has hash
`fc2682ebedafe92e4c99501f7d2a12a49f145c7bec92208db2e2fc789fea9434`.
Those new changes were therefore not linked into this guest image or executed
natively. The old source did contain a flat app-local `semantics()` method,
but this run demonstrates no consumer or screen-reader route. Complete
source, disk, QMP and frame hashes are in
[`qmp-keyboard-v3.json`](../verification/native-materiaal-accessibility-snapshot/qmp-keyboard-v3.json);
generated binaries/images exist only in the ignored local
`build/native-materiaal-accessibility-v3/`.

No cold restart/recovery, screen reader, speech output or human accessibility/
UX acceptance was performed. This baseline closes no U criterion and does
not validate the new semantic tree. Release must first integrate the leaf
UI code into one current Materiaal runtime and provide exact source/artifact
binding; this native gate must then be rerun on that source.

The explicitly approved backend is `noto-sans-mono-bitmap = 0.3.2`, with
defaults disabled; regular, size16/32, Basic Latin, Latin1 Supplement and
Specials. No runtime dependencies, build script, allocator or floats.
Rust MSRV 1.56.1; published 7 January 2026. The actual tarball checksum is
`f102c477fd4ec864c149803bf606ecc23bc9329602ec0f99073a728abf454abc`.
Library MIT; the original Noto Sans Mono Regular 2.014 is OFL1.1,
copyright 2022 Noto Project Authors. Font SHA:
`56134342e3d5137fbfcb25361039d4a98def7df75db7dd821ef47c3ae03127b6`,
traceable to upstream Git `636fb222507237549c20e9b23dae54fc9d639ae3`.
Both notices remain under ui/native/licenses. This is a bounded first font
backend, not a definitive visual identity or complete Unicode engine.
Missing glyphs are recognisable replacement characters; stored UTF-8 remains
intact. The old uppercase G0 font is not used for user documents.

Primary sources: [crate](https://crates.io/crates/noto-sans-mono-bitmap/0.3.2),
[upstream](https://github.com/phip1611/noto-sans-mono-bitmap-rs),
[Noto OFL](https://github.com/notofonts/latin-greek-cyrillic/blob/main/OFL.txt),
[existing product agreements](PRODUCT-SPEC.md),
[input contract](NATIVE-INPUT-SERVICE.md) and [GOP baseline](GRAPHICS-G0.md).
Source checking on 30 October 2026 is not an audit.

## Evidence plan and open follow-up

Targeted host tests: UTF-8/range/selection, actual service + capability engine
behind DocumentSession, save/ABORT/conflict/uncertain commit/I/O error, no
false Saved, guards/clipping/colour order of actual rendered tiles, focus
and 100/200% layout. Then Clippy/fmt and an actual scalar none build. Root
integrates actual ring-3 processes/capabilities, QMP keys/mouse, rendered
before/after images **and** operation assertions, disk oracle/cold reopen
and exact image evidence. Screen reader, all seven routes, compositor
recovery, complete editor features, cross-app clipboard, persistent keys
and the rest of the release remain open.

## First executed checkpoint

`python3 -B ui/native/verify.py` was executed with Rust 1.90.0: 14 host
scenarios, Clippy, fmt, feature graph and an actual scalar
`x86_64-unknown-none` build pass. The nine built rlibs are bound through actual
Cargo artifact records and checked with objdump/nm; no SIMD/FPU instructions
or host malloc backend in those objects. Generic controller methods are
fully monomorphised only in the final native caller; this is not ring-3
execution. Exact source/input/artifact binding and commands:
[`ui/native/verification/checks.json`](../../ui/native/verification/checks.json).

The large existing capability/volume host harness initially exceeded the small
default test-thread stack. The same explicit 16-MiB host-test stack as the
SDK harness resolves that; the native 64-KiB stack is neither enlarged nor
approved. An initial compilation required type/API corrections in the new
tests. At the checkpoint above, the targeted checks actually passed again.

Native integration requires a dedicated app linker: font pointer tables reside
in `.data.rel.ro` and must remain R+NX. The old system-app linker places
`.data.*` in one RW page and allows only 48 KiB of rodata; that does not fit.
Measured before final linking: approximately 146 KiB relro +35 KiB rodata
+27 KiB frontend code. A bounded dedicated read-only segment of, for example,
256 KiB fits within the already permitted user-image window and requires no
kernel/capability expansion. Final residency/stack/latency remain to be
measured on the actual linked UI app.

The general `tools/common.py::source_hash` does not yet track ui/. The
component's own evidence therefore additionally binds ui/native and
ui/prototype/tokens.css. Root must extend native image/capture input binding
before UI acceptance; otherwise old UI code could incorrectly appear current
under an unchanged general source hash.

## Follow-up: actually inspected rasters and input rebinding

After checkpoint `be0dc462`, the actual host pixels for Materiaal, editing
and dirty close at 100/200% and 320 logical pixels were inspected with
view_image. The same renderer feeds private tiles; the fixture uses actual
volume/capability-engine requests with synthetic test documents. These are
not native screenshots, a user trial or product acceptance.

Inspection found a missing first list row at minimum height and a modal with
partly obscured background text. The compact header now shows actual list
content, long titles retain space without overlapping metadata, and an opaque
local backdrop visually isolates confirmation. Primary, secondary, danger
and hover states use the same token source. Pointer and PageUp/PageDown can
actually paginate through the volume list; small viewports do not make later
objects reachable only through Tab.

The first controller/text/snapshot source was reviewed read-only by
kernel_storage: 14 scenarios repeated in a dedicated target, five source
hashes unchanged, no concrete blocker within that scope. Its rebind API
finding was addressed in the follow-up described above. That AI review on
`be0dc462` is not automatically a review of all subsequent code. Eighteen
targeted host scenarios, Clippy/fmt and the new scalar build now pass;
`verification/checks.json` binds the current source.
`verification/host-raster-review.json` contains only hashes/observations of
the host images actually inspected and the evidence boundary.

# Integration addition, 30 October 2026: native IPC wait

The experimental `native-system-materiaal` profiles now enable `ipc-wait`
(ABI 1.10). The existing input source and input policy park only when work
is demonstrably absent, preserving their original absolute deadlines.
Desktop-init, controller and compositor recognise exactly ABI 1.8 and 1.10;
ABI 1.9 remains the separate RTC clock and is not recognised as wait support.
The app, SUR1, token, ACK, focus and save contracts do not change.

The independent two-process trial demonstrated queue, deadline, revoke and
PIT-idle execution. That does not yet prove a working Materiaal chain: the
baseline without wait and the input-only wait trial both exceeded the fixed
20,000 PIT ticks after three actual frames. The next built version also parks
keyring/volume when no work is pending and desktop-init only in a quiet Running
phase. The local controller trigger is not hidden in a long receive wait.
The complete QMP operation and cold reopening must still pass. No enlarged
budgets, host-render fallback or new progress points.
