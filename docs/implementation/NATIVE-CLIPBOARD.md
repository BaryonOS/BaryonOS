# Native sharing of selected text — U02 clipboard

Implementation contract, 2 November 2026. Baseline
41d02693083aa5262eee435c583680b566808489, branch
codex/native-clipboard-20261102. This does not close U02: fully measured
interaction budgets and everyday format/capacity coverage remain necessary.

## Use and authority

Ctrl+C copies only explicitly selected text from the active Materiaal or
Calculator window. Ctrl+V replaces the selection or inserts at the cursor.
No whole-document, old-result or background-read fallback.
Ctrl+Shift+Delete clears the volatile clipboard through the trusted
compositor. The first bounded implementation supports 192 UTF-8 bytes in one
message; oversized selections produce visible feedback, without truncation.
This does not reduce the full product ambition; larger streaming transfers
follow separately.

The compositor is already the trusted owner of global input and focus. It
mediates CLP1 over existing app-INP endpoints. InputSession remains the sole
owner of inputreplyRX; no second session or copy of the handles is created.
No new kernel permission, syscall, process, host clipboard, file/network
access or general clipboard read. Ordinary input roles remain unchanged;
CLP1 exists only on the explicit window-compositor route.

A request requires the actual kernel PID, current app input epoch, exact
sequence/generation of the still-offered trusted CtrlC/CtrlV Press event
and current window focus. Control requires no Alt/Super/Shift;
repeat/release events are not user authorisation. One offer may authorise at
most one clipboard action. Apps cannot create authorisation with a
self-written key label. ACK, cancellation, focus loss, queue overflow, source
loss, retirement and recovery invalidate tickets. Trusted CtrlC clears the
old content beforehand, so a failed copy does not silently reuse old text.

## Data and transactions

CLP1 has a fixed 64-byte header plus at most 192 text bytes within the existing
256-byte IPC payload. UTF-8 and lengths/padding/versions are checked strictly;
no capabilities in messages. The broker retains only the selected text, with
no history/persistence/logging. Owned buffers are erased using the existing
pinned zeroize 1.9.1. No plaintext Debug/Clone.

A delayed paste response retains only a ticket; the compositor rechecks that
ticket and focus at every actual flush. No plaintext remains in pending
replies after permission expires. Text successfully queued to the app before
revocation cannot be retrieved; the app does not apply that text on a
Superseded ACK and erases its temporary copy. The app applies received text
only after the original input ACK Acknowledged; Superseded erases the temporary
text and does not change the document. Calculator validates the complete
text and remaining capacity before any mutation; invalid or excessive text
does not change the expression/cursor/selection or previous calculation.
Pasting does not perform a calculation.

Compositor crash/restart clears its clipboard and revokes all tickets; the
app document and expression remain according to the existing recovery
contract. This is not an account/authentication boundary. Other applications,
password fields, media formats, cutting and clipboard history are not implied.

## Write scopes and acceptance

The broker builder owns only input-client and window-session plus their
lockfiles in a dedicated worktree. Root owns ui/native, both app adapters,
calculator-model, native fixtures/runner and central documentation/score.
The latency researcher starts read-only; no shared concurrent writer.

Targeted Rust tests use actual protocol/Queue/capability engines for forged
PID, epoch, generation, sequence, wrong focus, replay, after ACK,
cancellation/backpressure, invalid UTF-8/oversize and atomic paste. Then
Clippy/fmt, freestanding builds and ELF bounds. The native gate retains all
existing 13 phases/pixels/ACKs/fault/save/disk/cleanup/raw33 and adds a real
Materiaal→Calculator→Materiaal copy/paste route, clearing, an error scenario
and recovery without clipboard content. No masks or relaxed deadlines.
For latency, the measurement profile, workload, repetitions and budgets are
fixed before measurement; input ACK is not a completed image.

Evidence remains bound to each source hash/build/profile. No U02 points for
host tests alone, an additional UI or new counters. The existing 41/100
remains during implementation.

## Preregistered interaction measurement

The first real clipboard route uses the existing Linux/TCG configuration,
one vCPU/256 MiB, 1280×720 and a bounded timeout of 1800 s (equal to the
existing CI gate). It retains all original thirteen phases and adds clipboard
steps. Additional app-image intervals are recorded from the last QMP key
action to an exactly confirmed complete app image. They include QMP, host
observation and captures; no guest timestamp or actual display latency.
This small number of measurements does not close a p50/p95 budget.

The separate recorder tools/window_latency.py requires a previously saved
plan bound to source, profile, workload, complete image hash and context.
For the later WHPX reference, the product target is: p95 input/focus/app
switching at most 100 ms, complete image/paste at most 250 ms. For this
recorder that is a conservative upper bound for the complete result, not a
first-pixel measurement. Each direction is a separate stratum with at least
20 samples, three cold starts, no input bursts and no next input before an
exactly stable target image. Workloads: A/Backspace in Materiaal,
Tab/Shift-Tab, F6 and dock clicks to both apps with a targeted key to check
routing. TCG results are not WHPX acceptance. No new measurement counter
replaces the native image/storage/recovery gate.
