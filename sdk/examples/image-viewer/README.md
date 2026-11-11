# R22-IMAGE-VIEWER-LEAF-1

App-specific `no_std` controller for **FR-A03**, rubric **A04/U03**.
FR-A04 is PDF and remains a different work package. Base:
`ca45a7dabe6a040921de03bdc2fad3e5045a75ba`.
TL89f released only this new subtree; Apps/SDK11 is the consumer.
No native executable, new SDK, wire, bootstrap, installer or shared UI.

## What the leaf actually does

A selected object/revision and an explicit byte/pixel budget start a
load ticket. `decode` processes real PNG/JPEG bytes with the existing
`services/image-codec`; `decoded` also accepts the result of a separate
decoder. Both check the selection and output shape before pixels become
available. Old tickets, a different revision and oversized/corrupted results
are refused. Starting on a new object removes the previous image from
the controller; errors never deliver substitute success pixels.

`Fit`, 100% and bounded zoom of 25–800%, arrow keys and dragging operate on
the assigned SUR1 geometry. 100% uses the specified 100/200% UI scale.
Fit fits both dimensions into the surface; extreme aspect ratios keep
at least one visible pixel. The simple nearest-neighbour rendering
selects real source samples, but is not a quality claim for photo scaling.
The viewer follows JPEG EXIF orientation 1–8 when selecting pixels, without allocating a second
full image. Fit, zoom, pan and `semantics().image_size` use
the oriented display dimensions. `decoded` refuses dimensions that do not
match the orientation before rendering becomes possible.
RGBA16 is rounded only for display; original and metadata remain
unchanged. Alpha is blended against the 24-bit background chosen by the caller.
No custom font, colour, focus or component system.

`render_tile` writes only a tight-packed BGRX tile of at most 32x8
into caller memory. Extra buffer space remains untouched. The adapter sends
those bytes with the existing SUR1 client and calls `acknowledge` **only after the real
positive blit/COMMIT reply**. Only all consecutive tiles of the same
generation set `presented`; changes withdraw this flag immediately.
This flag does not mean page flip, atomicity, durability or native evidence.

`input` processes the existing INP1 `Event` types. The adapter remains the owner
of authentication, epochs, ordering and PEEK/ACK. On a superseded offer it must
cancel the local pointer/focus state via the existing Cancel event;
this leaf is not an input broker. Outside the assigned rectangle, after overflow or
loss of focus a drag is dropped; coming back with the button held does not start a drag.
Keys: `+`/`-`, `0`/Home (fit), `1` (100%), arrows and Escape (close).
A key release performs no action; Ctrl/Alt/Super combinations are
not taken over from the shell.

`semantics` provides the real state, Dutch status text, image size, zoom,
focus and frame confirmation for the future shared chrome/a11y adapter.
This is **not** a registered accessibility tree or a working screen reader.
Error/empty/loading status is not drawn as pixels here; the existing
UI adapter must actually display this text. The leaf contains no window.

## Invariants and hard evidence boundaries

- `Selection`, `Load` and `observed` are correlation data, **not authority**.
  A native open portal/provider must bind the kernel sender to only the
  selected object, the revision and READ, including direct malicious
  IPC/LIST/mutations. `domain0 SEND` or `EditMode::ReadOnly` is not an alternative.
- The native launch slot, object-only READ, focus, surface and lifecycle seam
  has **not been released** on the base source. Root/Materiaal owns that integration.
  No existing Materiaal endpoints or bootstrap fields are taken over.
- The selection binds the byte and decoded pixel budget; the existing codec `Limits` additionally binds
  dimensions, metadata, scans/chunks and work admission. There are no
  extra controller/render allocations or full RGBA conversion buffers.
  The existing parser can allocate infallibly: a dedicated bounded arena and
  an isolated process with OOM/fault cleanup are **mandatory native preconditions**.
  `work_bytes` is not a hard heap limit. Being able to show an `OutOfMemory` error from a worker
  does not prove that every allocator failure is recoverable.
- `decode` is synchronous. A `Cancellation` cancelled beforehand really works;
  cancelling the ticket prevents late publication. A UI cancel cannot run by itself
  during a blocked single-threaded decode. Responsive native
  job/process cancellation remains open; there is no misleading cancel button here.
- `close`, cancel, replace and revoke drop the owned pixel buffers and
  invalidate future rendering/ACKs. Source bytes are borrowed and are not
  erased. Allocator deallocation is not a secure erase. The adapter must stop real
  jobs, close endpoints and have the old surface erased/retired;
  pixels already shown or copies obtained earlier are not recalled.
- The codec and the semantics flag of the same name report
  `metadata_transforms_applied=false` for the **raw samples**. The tile renderer
  applies EXIF orientation to the display; ICC/gamma colour management and PNG eXIf
  remain open. No full format/photo fidelity claim.
  Dependency choice/notices remain in `services/image-codec`; no new
  registry dependency, decoder or host execution added.

## Targeted acceptance

Use the existing pinned toolchain environment with a dedicated absolute
`CARGO_TARGET_DIR`; no shared Cargo target writer. From the repository root:

```sh
cargo +1.90.0 test --locked --offline --manifest-path sdk/examples/image-viewer/Cargo.toml
cargo +1.90.0 clippy --locked --offline --manifest-path sdk/examples/image-viewer/Cargo.toml --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path sdk/examples/image-viewer/Cargo.toml -- --check
cargo +1.90.0 build --locked --offline --release --lib --manifest-path sdk/examples/image-viewer/Cargo.toml --target x86_64-unknown-none
```

Tests use real decoder fixtures plus independent known
RGBA8/RGBA16/alpha/letterbox values and real existing input/surface types.
Four EXIF regressions test all eight asymmetric pixel matrices, real
JPEG decode to all BGRX tiles, invalid display dimensions and rotated
fit/zoom/pan axes at both UI scales.
Fault injections for service refusal/I/O/OOM are solely controller inputs,
not simulated kernel/service acceptance. Tests cover stale replies,
budgets, malformed decoded buffers, decode errors, cancellation, revocation,
closing, scale/zoom/pan limits, pointer cancel and old/incomplete frames.
The verification directory binds executed results to source/artifact hashes.

**Not executed:** native app start, object portal, QEMU operation,
independent framebuffer oracle, process OOM/cancel/revoke, cold storage,
complete native/hardware/accessibility and release integration gates.
The [EXIF consumer test](../../../docs/verification/platform-m7-exif-viewer-20261103.md)
separates the host evidence from the separately repeated native decoder regression.
FR-A03 remains open; this step yields no rubric points
(**41 -> 41/100**, assessment by the integrator).
Reverting the EXIF consumer step: revert the small codec/viewer diff;
no right, storage format or runtime profile has changed.
