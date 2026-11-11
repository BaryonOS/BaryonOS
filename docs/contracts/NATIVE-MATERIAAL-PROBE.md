# GUI-QMP-1 — real native control chain

Owner native_ui_apps: exclusively tools/run_materiaal_probe.py,
tools/tests/test_materiaal_probe.py and this contract. Base d528821, own
worktree materiaal-qmp. Root owns feature/ABI/manifest/build/kernel observer;
kernel_storage builds the separate desktop-init/controller. This runner is
a bounded integration trial, not full Materiaal or release acceptance.

## Execution and evidence boundary

The runner starts only a hashed own-kernel QEMU with eight real CPL3 images:
desktop-init, keyring-service, volume-service, materiaal-app, native-compositor,
input-source, input-policy, desktop-controller. The normal product composition
has seven processes; the eighth process is exclusively the limited test
counterpart with two IPC grants for a real Init StopRPC. It has no
display, input, block or memory-write grants to another process.

QMP delivers real PS/2 keys/mouse movements and reads real framebuffer images.
The runner writes no guest RAM, draft, result word or storage content during
the trial. It performs no missing OS function on the host. Protocol fixtures
in the Python unit tests are explicitly host negative trials for the validator.

The fixture is the actually typed ASCII/UTF-8 `nieuw os werkt` (14 bytes),
object 0, name `Nieuw document` ("New document"), revision 1. It is plaintext test data on a
new 1MiB throwaway image. The existing Key Service is present, but this chain
proves no persistent encrypted storage, key recovery or product security.
FNV is only a check on the exact bytes here; no authentication claim.

This first development trial is explicitly 800×600, qemu64/TCG/1 CPU/256MiB, VGA/PS2.
It does not replace the larger release profiles, keyboard/mouse acceptance,
200% accessibility, screen reader and physical hardware validation.

## Linked build

Features: native-system-materiaal-probe for normal and
test-system-materiaal-recover for the separate cold readonly VM, always
the actually linked framebuffer loader. The feature/build helper must have been
integrated by root; absence or an old source hash gives an explicit
failed preflight, no pre-filled PASS or fallback to a browser trial.
native-system-materiaal and test-system-materiaal-scale200 are separate root profiles;
this CLI does not yet sign those off.

`BUILD/manifest.json` contains source_fingerprint_version 3, current source_sha256,
kernel_feature, loader_feature=framebuffer, SDK/document_session trial flags False
and `materiaal` exactly equal to `users/materiaal/build-report.json`:

```json
{
  "passed": true,
  "native_runtime_performed": false,
  "probe_controller": true,
  "apps": [
    {"name": "desktop-init", "sha256": "64 lowercase hex", "elf": {}},
    {"name": "keyring-service", "sha256": "64 lowercase hex", "elf": {}},
    {"name": "volume-service", "sha256": "64 lowercase hex", "elf": {}},
    {"name": "materiaal-app", "sha256": "64 lowercase hex", "elf": {}},
    {"name": "native-compositor", "sha256": "64 lowercase hex", "elf": {}},
    {"name": "input-source", "sha256": "64 lowercase hex", "elf": {}},
    {"name": "input-policy", "sha256": "64 lowercase hex", "elf": {}},
    {"name": "desktop-controller", "sha256": "64 lowercase hex", "elf": {}}
  ]
}
```

`elf` appears above as an empty object only for schema illustration; actual
evidence requires exactly the recomputed inspect_user result. Every
image must also be literally embedded in the kernel bytes. The build PASS is
exclusively inspection, no runtime claim. Kernel and loader must be exactly the
ESP copy; extra ESP files, modified bytes, symlinks and hardlinks are
rejected. Root determines the NOS_DESKTOP_INIT_ELF/NOS_MATERIAAL_ELF/
NOS_COMPOSITOR_ELF/NOS_INPUT_SOURCE_ELF/NOS_INPUT_POLICY_ELF and controller grants.

## Native serial grammar

The existing seven BOOT/EBS/kernel/KS/M2 markers must appear exactly in order,
followed by MATERIAAL:MANIFEST:PASS. Then each exactly once:

- MATERIAAL:SOURCE:READY — actual hardware reset plus source SYNC confirmed.
- MATERIAAL:POLICY:READY — real service loop/IPC ingress ready.
- MATERIAAL:COMPOSITOR:READY — real bootstrap blit and valid constructors.
- MATERIAAL:FOCUS:READY — real current policy FocusACK, after the three service gates.

Init's SYSTEM:READY 1 is exclusively StartupReady and is not accepted as paint or
hardware evidence. App 400 requires a complete current frame plus a real
INP1 response; Empty is not by itself source/focus evidence. Therefore all
four separate gates are required before the runner gives any user input.

The observer emits for each new complete current frame:

```text
MATERIAAL:SNAPSHOT:BEGIN
MATERIAAL:READY:<u64 exact20 decimal digits>
MATERIAAL:PRESENTED_GEN:<u64>
MATERIAAL:FRAMES:<u64>
MATERIAAL:TILES:<u64>
MATERIAAL:KEYS:<u64>
MATERIAAL:MOUSE:<u64>
MATERIAAL:INPUT_ACKS:<u64>
MATERIAAL:SAVE_ACKS:<u64>
MATERIAAL:SAVED_REVISION:<u64>
MATERIAAL:SAVED_BYTES:<u64>
MATERIAAL:SAVED_SUM:<u64>
MATERIAAL:SCREEN:<u64>
MATERIAAL:DRAFT_DIRTY:<u64>
MATERIAAL:DRAFT_BYTES:<u64>
MATERIAAL:DRAFT_SUM:<u64>
MATERIAAL:OBJECT:<u64>
MATERIAAL:SCALE:<u64>
MATERIAAL:ERRORS:<u64>
MATERIAAL:SNAPSHOT:END
```

All other u64s are also exactly 20 digits and at most u64::MAX. SCREEN 1/2/3
is Materiaal/Editor/Confirm; no draft gives OBJECT=u64::MAX and length/sum 0.
An empty existing draft has FNV_OFFSET, so that no draft and an empty document
remain distinguishable. The observer publishes no prescribed host phase.
The host selects the phases from actually observed states/generations.
Complete frames and generation must strictly increase; tiles cover at least
all 32×8 tiles of every frame. Expired/half/duplicate frames do not count.

After the last real screenshot QMP sends one F12. App 640 counts exclusively
actually authenticated INP1 F12/Press, no Enter release, F12 Repeat or Release.
The kernel probe may only set its own controller barrier after actual
Saved→later List→later Reopened with equal object/length/FNV and a subsequently
newer F12 Press. Cold recovery requests list→actually opened document,
without a synthetic save. This is trial orchestration, not a product shutdown button.

Then follow MATERIAAL:F12_BARRIER:<1 as 20 digits> and
MATERIAAL:CONTROLLER:RELEASED. The controller performs the real Init StopRPC;
Init sends app STOP 5, refuses dirty/saving via STOP_REFUSED 6 and only expects
DONE 3 after real client cleanup. Compositor retire follows after a real Focus 0 ACK and
a real clearing blit. EXIT_SLOT/EXIT_CODE/EXIT_STEP triplets for app 3/init 0/controller 7
must each report code 0/step 0 once. Their mutual completion order may vary.

Then exactly these 20-digit fields: DISK_READS, DISK_WRITES, DISK_FLUSHES,
LIVE_PROCESSES, LIVE_HANDLES, FRAMES_RECOVERED. Normal requires writes>0/flushes≥2;
recover writes/flushes 0. Live processes/handles 0 and frame recovery 1 are mandatory,
followed by CLEANUP:PASS, CASE:PASS, M1:PASS. Raw QEMU exit 33 and no timeout are
mandatory. A host QMP quit or kill cannot replace this completion.

## Real actions and screen verification

Normal: complete ready capture → mouse Nieuw ("New") → complete empty editor capture →
14 real keys → complete dirty capture → Ctrl+S → Saved capture → Escape →
List capture → Tab,Tab,Enter → Reopened capture → F12 → real clean shutdown.
The source-checked ui-core cursor starts at (0,0); bounded relative movements
go to Nieuw without a pressed button, followed by real Down/Up. The app
receives global GOP coordinates, not injected local UI events.

Recover: new QEMU from the same source version, private copy of a fully
recomputed successful normal-volume.after.img, QEMU disk readonly=on plus real
readonly grants. Ready capture → Tab,Enter → real Open capture → Ctrl+S →
understandable readonly error/capture with 0 save ACK → F12 and clean shutdown.
All disk bytes before/after must remain exactly equal.

Every capture moment keeps the SHA256/length of the actually observed
serial prefix and the precise screendump command number. Replay binds those
data again to the raw bytes, the complete ordered QMP transcript and
PPM. Images must really be 800×600 and change between every action.
Pixels outside assignedRect(16,48,768,536) remain equal; saving keeps the
actually rendered text/caret area while status/control pixels change.
The font/layout/usability inspection additionally remains a real visual assessment;
`visual_quality_acceptance=false` reports that this full release gate is still open.

The independent disk oracle reuses volume_disk.choose/content (both
table copies, generations, checksums, padding and chunks) and compares the real
saved name/revision/bytes with the 14 typed bytes. Canaries outside the grant,
other objects/domains, history and collections must be untouched.
The old volume_disk.analyse accepts exclusively its binary generated
fixture; this trial does not change that function and does not disguise the typed text.

## Report and path boundaries

All writes stay in an explicitly chosen ROOT/build[/subdir]/run/materiaal-*
child. Normal always creates a new throwaway image and takes no disk argument.
Recover accepts only a generated parent run dir, recomputes its
complete evidence and takes exclusively a new copy of its after image.
No personal disks, NAS, accounts, shares, host network or physical writes.
A private QMP pipe is the default; a Unix socket is only allowed within its own short run directory.

result.json contains schema_version 1, passed, scope, case, source_commit/source_sha256,
exact command/cwd/profile, real QEMU entry/implementation hashes, frozen_artifacts,
before/after SHA256, stages and artifact_sha256. `stages` is an ordered list
of `{phase,snapshot,serial_size,serial_sha256,qmp_index}`. `observed` comes from
recomputation of raw serial/QMP/pixels/disk. Every error keeps passed False;
a new failed preflight first revokes an old summary PASS.
The run directory keeps exact ESP/ELF/build/firmware inputs, raw serial/stderr/QMP,
real PPMs and before/after disk. No generated binaries in Git.

## Check commands and remaining plan

Host validation (no native claim):

```sh
python3 -B -m unittest discover -s tools/tests -p test_materiaal_probe.py -v
```

After root integration, first build the current dedicated probe image with existing
tools/dev.py/tools/build.py and its own NOS_BUILD_DIR/NOS_TARGET_DIR. Then:

```sh
python3 -B tools/run_materiaal_probe.py --case normal --build-dir build/materiaal-probe --vga-rom /explicit/existing/VGA.rom --vga-rom-sha256 EXACT_SHA256
python3 -B tools/run_materiaal_probe.py --case recover --build-dir build/materiaal-probe --parent build/materiaal-probe/run/materiaal-normal-UNIQUE --vga-rom /explicit/existing/VGA.rom --vga-rom-sha256 EXACT_SHA256
```

The second command first requires a new recover build with the same frozen
source and the linked recover feature; do not reuse a normal kernel as
readonly evidence. The ROM/SHA/UNIQUE above are explicit placeholders for
existing checked artifacts, not a delivered or tested native outcome.

Still to be performed after the first native chain, visibly open:

| Scenario | Real action and expected independent evidence |
|---|---|
| Cold read-view | New VM after a fully terminated normal VM; the same typed bytes actually opened via the provider, hashes and readonly I/O 0. |
| Dirty-close/cancel | Type new work, Escape shows a real confirmation; continuing keeps the draft, explicitly cancelling the upload sends ABORT, no extra revision. |
| Readonly | Open real bytes, Ctrl+S is understandably refused; no COMMIT-ACK, no disk mutation; keep checking the native capability boundary separately. |
| 200% | Dedicated scale200 image, real font/focus/pointer check on the minimum supported viewport; no fallback to 100% to get green. |
| Save pointer outside | Down on Save, movement/release outside assignedRect and held re-entry; no BEGIN/COMMIT. Only a new real Down/Up inside may save. |
| Error/recovery | Source/policy/surface loss, lost ACK, full volume and failed I/O keep the draft and report the real state; no invisible action or false Saved/Presented. |

At this source checkpoint the root bootstrap/observer/controller are still external
integration dependencies. The runner has not claimed a native VM PASS. All 62
release criteria and the existing progress rubric continue to apply undiminished.

## Targeted retest after candidate 6a9292d

Own and independent host negative trials found three gaps in recomputing
stored evidence. A valid after image was not re-bound to the active
QEMU volume.img; too-late valid serial prefixes could certify an old
phase; offline recover replay did not require a recomputed normal parent.
These were evidence validator errors, not evidence that the real runner executed host functions
as OS functions. The first two regressions were actually recorded red on the
old runner and green after the targeted fix.

Replay now requires byte equality of the active QEMU disk and after image. Every
camera prefix is strictly longer than the previous one, lies before F12/controller release/
exits/final records and binds the last complete current snapshot in the correct
phase. A half following record or a historically matching phase gives no capture.
This rejects all prefixes on the complete final serial, all on Reopened before
F12, as well as a Ready prefix whose last snapshot is already Editor.

Cold replay accepts exclusively one owned normal parent from the same BUILD/run,
fully recomputes its raw 33/serial/QMP/pixels/disk/input hashes and binds its
after bytes to the recover before bytes. A recover parent or a normal parent
with its own parent is rejected: the check cannot recurse without bound.
Source hash, machine/QEMU entry/implementation, loader, firmware/ROM and all eight
userspace images must match; the kernel may exclusively have the recorded
recover feature. The disk remains readonly in QEMU and in the byte check.

On recognisable native TIMEOUT/FAIL/PANIC diagnostics the already failed VM gets
at most 1 s to write its real dump and raw exit before QMP close/kill.
This does not extend acceptance time and does not turn a failure into a PASS. The measured
first native candidate failed around 203 s: Ready frame after about 106 s, one
complete frame and 3161 tile ACKs before the fixed PIT boundary. No tolerance
was lowered to make this run green. Root is investigating the measured diagnostic
observer overhead; the APP/UI source stays the same for that targeted repetition.

Nineteen host negative trials pass after these fixes. Exact commands/logs are
under its own build/logs/materiaal-qmp; native repetition and independent
review of the repaired runner remain separate evidence steps.
