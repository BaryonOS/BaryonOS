# Materiaal: separate native app

Actual no_std CPL3 caller for existing DocumentSession, typed INP1 client and
SUR1 client. This package contains no display capability, raw PS/2 driver,
block I/O, host storage, networking, modelled reply or prefilled document.
The compositor, input source/policy and volume provider remain separate
processes with their own rights. A test backend under tests/ is only a
protocol peer for host experiments.

The startup page and channel assignment are in
[NATIVE-MATERIAAL.md](../../../docs/implementation/NATIVE-MATERIAAL.md).
The lead agent integrates kernel manifests and creates actual VM test images.
This source or a successful ELF alone does not establish a native user workflow.

## Building and checking

With the project toolchain and centrally integrated SDK ABI1.8:

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools python3 -B sdk/examples/materiaal/verify.py
```

The separate locked workspace uses exactly the existing libraries, font pin
and design tokens. build.py only builds, changes no shared source or disks
and grants no rights. The linker produces five monotonic LOAD segments.
R+NX read-only placement also consumes font-pointer tables before the writable
wildcard; three 64-KiB buffers reside in exclusive RW+NX BSS, outside the
64-KiB stack. No heap or host malloc fallback. The actual kernel user::parse
checks the built ELF; objdump/nm inspect the monomorphised scalar code.

verify.py stops if the central SDK still uses ABI1.7. For the first local
ELF experiment, only the temporary SDK1.8 patch supplied by the lead agent
was active; those shared files are not committed with this package. Exact
SDK source hashes and the patch hash remain in verification/pre-native.json.

## Interaction in the integrated image

Materiaal reads the actual volume list. Enter on Nieuw (New) opens an empty
document; typing edits actual UTF-8 draft bytes. Ctrl+S begins streaming
storage; Annuleren (Cancel) sends an actual ABORT while the upload runs.
Escape requests return to Materiaal; changed work offers save, continue editing
and explicit discard. Tab/Shift+Tab move between visible actions and objects.
Saved objects reopen the same bytes through the provider.

The input client receives global GOP coordinates. Compositor offsets are
subtracted only inside the assigned surface. Outside it, a click/drag ends;
returning with a held button does not start a new click. At 100/200%, the
renderer uses the same design tokens, focus and textual states. There is
not yet an integrated screen reader, complete Unicode input or multi-app session.

At most four PEEK/ACKs, one upload step and one tile transaction per iteration.
An old partial paint must not confirm a newer draft generation. Ready requires
an actual INP1 response and all tile ACKs for a current complete frame.
A lost reply triggers neither repeated mutation nor an invented ACK; storage
remains uncertain where necessary and the draft stays in the process.
Input loss withdraws Ready and shows actual state; surface loss stops
invisible new actions.

Supervisor STOP5 is accepted only on the previously granted control channel
with the expected initPID, exact tag, 1B payload and zero capabilities.
Dirty/saving returns STOP_REFUSED6, not autosave/discard/DONE. Only clean/idle
plus explicit cleanup of the three channel owners returns DONE3. Complete
session recovery, receiving grants again and safe persistent encrypted keys
remain separately open.

Twenty host-chain experiments plus one actual ELF-policy test are in
verification/. The review fix has separate red/green evidence: Ready is off
during every new paint; PresentedGeneration changes only after all current
ACKs. The built entry checks RSPmod16==8 and jumps directly to C. QMP interaction,
actual rendered native screens, cold reopening, latency/stack measurements
and all 62 full-release criteria remain open.
