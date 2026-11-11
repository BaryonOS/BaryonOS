# Native window session — U01 build contract

Source base: `0fe648b`, working branch `codex/mature-native-session-20261102`.
This contract describes ongoing work; it does not close U01 and changes no points.

## Result and boundaries

A native compositor shows Materiaal and Calculator as two separate
CPL3 processes. Only the compositor owns DisplaySource; only the input source
owns InputSource. Applications get exclusively their own SUR1/INP1 channels.
The existing document, key and volume rights remain unchanged. The new
graphics library uses bounded pre-generated glyph images and
own vector icons, without a runtime font parser or new privileged code.

The compositor manages window positions, stacking order and input focus. An app cannot
overwrite a system border or claim input from another window. The
kernel supplies the real sender of every message; a client ID or coordinate
never grants rights. Per window there is a bounded backing store, one
SUR1 receiver and a bounded INP1 offer/ACK queue. In total at most
1,400,000 pixels of backing store, two windows, no heap or shared app memory.

## Recovery

The user-space supervisor decides on recovery. A compositor error must not destroy a
document draft or calculation. The supervisor first retires
exclusively the four recorded old presentation handles per app (`0x53`, phase
`a2=1` of this unreleased ABI1.11 profile). An app-owned reply receiver
otherwise stays open after peer exit and an ongoing RPC cannot read its pause request.
Retiring closes those old handles without allocations, new processes,
epochs or changes to storage, control or the old process capability. The apps
thereby return from their old RPC, pause presentation and confirm that
via their existing control route. Only after both real pause confirmations does
init request the construction phase (`a2=0`); that requires prior retirement. The
supervisor can then restart the recorded compositor/input service group with fresh process identities,
capabilities and epochs. Apps bind those new channels explicitly and draw
again. Storage and document channels are not replaced in the process. The old
channels no longer deliver success. Recovery does not widen device rights.

The explicit app cleanup after retirement treats only `BadHandle` per
individual old handle as already closed. Other close errors remain errors,
both owners are tried and no input or frame ACK arises.
Normal `close()` semantics and the existing RPC budgets remain unchanged.

The init policy then separately guards Wait-for-Pause, Wait-for-fresh-READY
and Wait-for-Rebind-ACK. The old readiness mask is cleared at the start of recovery
down to the two persistent app bits. Only a successful construction syscall plus
validated fresh boot identities/generation/epochs opens the READY phase; that
starts again with exclusively app bits. Rebind requires all three current
service notifications in that phase. ACK7 and ACK8 apply exclusively in their own phase;
old READY notifications cannot open the rebind gate.

The kernel contains only the bounded process/manifest mechanics, no rendering,
input focus policy or font parser. Partial initialization, wrong caller,
living old service, stale process capability, endpoint/frame shortage and wrong
generations must fail closed. Existing diagnostic profiles remain
separately usable.

## Acceptance

First host tests for geometry, bounds, focus, offer/ACK, malformed messages
and recovery binding checks; then no_std build/ELF inspection. Native acceptance
requires actually two apps, keyboard/mouse focus, window movement, negative
surface/caller trials and a real compositor error while preserving draft and
calculation. Then real save/list/reopen, independent disk check
and resource cleanup. The physical GPU, real account authentication, screen reader,
full text shaping and human user tests fall outside this gate.

N01 is additionally integrated in a separate native profile; a
network service compile is not native network acceptance. U01/N01 only get
their fixed points if the full existing criterion boundary demonstrably passes.
