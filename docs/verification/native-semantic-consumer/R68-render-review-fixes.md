# R68 renderer review fixes

Status: host regressions and one source-bound native renderfixture pass.
Full compositor/session, assistive-service and human acceptance remain open.

## Source and scope

- Frozen review base: `e382a989d4b28a45a23e31ff4edef2426e9c2616`
- Work branch: `codex/native-render-review-fixes-20261101`
- Changed runtime: `ui/native/src/render.rs`
- Regression tests: `ui/native/tests/render_regressions.rs`
- Existing assertion updated only near line 597 in
  `ui/native/tests/interaction.rs`
- No Cargo manifests, lockfiles, kernel, SDK, compositor, shared tokens, or
  accessibility IPC changed.

## Findings and fixes

1. The semantic output buffer now has capacity for the volume-domain maximum
   plus the possible paging controls: `8 + PER_DOMAIN` nodes. `Frame::semantics`
   traverses the same bounded node builder once to count the exact result,
   rejects an oversized or undersized request before writing, and traverses
   again to fill the caller buffer. A short buffer therefore remains untouched.
   At 800×720, 100%, with eight objects, the regression asserts seven visible
   list items, both paging controls, the status node, and exactly 15 nodes.
   Fourteen output slots are rejected atomically. The existing 1920×1080
   interaction case separately asserts its actual 14 nodes; maximum capacity
   is not confused with the count for that viewport.
2. The Window semantic bounds are already in physical viewport coordinates.
   They are no longer scaled a second time; logical control bounds still receive
   exactly one scale conversion. Regressions assert 800×600 physical bounds at
   both 100% and 200%.

## Reproduction and verification

On the frozen unmodified source, the new test file failed both cases:

```text
semantics_budget_includes_paging_and_preserves_short_output
  panicked at src/render.rs:533:17: index 14 out of bounds for length 14
window_semantic_bounds_match_physical_viewport_at_100_and_200_percent
  expected width 800, received 1600 at 200%
```

Red log: `/tmp/nieuw-os-render-red.log` (test process exit 101).

The fixed source was verified with Rust 1.90.0, using the existing offline
Cargo cache and host linker; no packages were installed. Exact commands and
results:

| Command | Result | Log |
|---|---:|---|
| `python3 -B tools/dev.py cargo fmt --manifest-path ui/native/Cargo.toml -- --check` | exit 0 | `/tmp/nieuw-os-render-fmt.log` |
| `python3 -B tools/dev.py cargo test --offline --locked --manifest-path ui/native/Cargo.toml` | exit 0; 20 interaction tests + 2 renderer regressions | `/tmp/nieuw-os-render-tests.log` |
| `python3 -B tools/dev.py cargo check --offline --locked --manifest-path ui/native/Cargo.toml --tests` | exit 0 | `/tmp/nieuw-os-render-check.log` |
| `python3 -B tools/dev.py cargo clippy --offline --locked --manifest-path ui/native/Cargo.toml --all-targets -- -D warnings` | exit 0 | `/tmp/nieuw-os-render-clippy.log` |

SHA-256 of the fixed test inputs:

```text
52fce85611498d4d33f96a64d83205045ab6ccd9264c291227d3ff27f1a4e234  ui/native/src/render.rs
b2aa1a3e6629e0228cb61415e8397bd8420cc3cb72a585de00e0d5f7588f07d6  ui/native/tests/interaction.rs
27ed99f5a7cd74fdd79a21dd3461a148a46e9e05ae24e66cb520546273446a77  ui/native/tests/render_regressions.rs
```

## Evidence boundary

These are host tests of the scalar renderer and semantic-tree construction.
They do not execute the changed source in a guest, demonstrate a native
compositor, expose the app-local tree to a screen reader, or constitute a
human accessibility review. No U01–U06 criterion or progress point is closed
by this change.

## TASK-UI-IMAGE-0818 — subsequent native renderfixture

The host-only boundary above describes the regression tests. A subsequent
single native probe run actually compiled and executed the fixed renderer
inside the Materiaal app. The separate app-local action-wrapper WIP on
`89e2004` was not included. No runtime source, runner, manifest, lockfile,
shared token or central progress file was changed for this run.

- Exact tested commit: `0818f5ec651b72c32b8795ff0d3d9a6ebdc9783d`.
- Source fingerprint v3, rechecked by build, run and offline replay:
  `26b41e5586434ce51202a4e234ccaa99bad25c5a76ec3efdb3af9d7020c14f5f`.
- Profile: QEMU 10.2.1, `pc-q35-10.0`, `qemu64`, TCG single-thread,
  one vCPU, 256 MiB, headless VGA/PS2, private QMP pipe, separate OVMF
  VARS and generated 1 MiB test volume. No guest network.
- One build and one normal probe VM, after explicit release4 and registrar12
  slot acknowledgements. Runner exit 0, raw QEMU exit 33, 132.460607 seconds,
  no timeout. The slot was explicitly released; no automatic second run.
- Full generated report and frozen original inputs:
  `/tmp/nieuw-os-render-review-e382/build/ui-image-0818/run/materiaal-normal-ad23a2ee91`.
- Compact provenance, exact QEMU command, eight userspace ELF hashes,
  firmware hashes, stage bindings and disk results:
  [R68-ui-image-0818.json](R68-ui-image-0818.json).

The exact command sequence is saved in the read-only bundle as
`commands/run-ui-image-0818.sh`. It selects the existing read-only Rust
1.90.0/QEMU/OVMF environment supplied by build infrastructure, verifies
commit, clean worktree, source fingerprint and tool hashes, and then runs:

```sh
export NOS_BUILD_DIR="$PWD/build/ui-image-0818"
export NOS_TARGET_DIR="$PWD/target/ui-image-0818"
export CARGO_TARGET_DIR="$NOS_TARGET_DIR"
python3 -B -u tools/build.py --kernel-feature native-system-materiaal-probe
python3 -B -u tools/run_materiaal_probe.py --case normal \
  --build-dir "$NOS_BUILD_DIR" --machine pc-q35-10.0 \
  --vga-rom "$P/qemu/usr/share/seabios/vgabios-stdvga.bin" \
  --vga-rom-sha256 c944f5fd404a6553a32e1e0527081d40e6040d3ce1740d39766bff01871e4fde \
  --timeout 240
```

Both build and runner exited 0. `P` is the recorded existing tool root in
the bundled command script, not a download or installation instruction.
Complete build/run log: `/tmp/nieuw-os-ui-image-0818-attempt.log`.

Additional executed commands on the same runtime source:

| Command | Result | Log |
|---|---|---|
| `cargo +1.90.0 build --offline --locked --manifest-path ui/native/Cargo.toml --release --target x86_64-unknown-none` | exit 0 | `/tmp/nieuw-os-r68-none-build.log` |
| `python3 -B -m unittest tools.tests.test_materiaal_probe -v` | exit 0; 22 host-oracle tests, no VM | `/tmp/nieuw-os-r68-probe-host.log` |
| Existing `run_materiaal_probe.replay(report, run_directory, common.source_hash())` after VM exit | passed; serial/QMP/frame/disk chain recomputed | compact JSON and original run |

### Actual input, rendered output and storage

Readiness required actual source, policy, compositor and focus markers plus a
complete current ready frame. QMP then moved the real PS/2 pointer and clicked
Nieuw, typed exactly `nieuw os werkt` (14 ASCII bytes), sent Ctrl+S, Escape,
Tab, Tab, Enter, and finally F12. The six captured frames are ready, empty
editor, dirty, saved, list and reopened. Each frame is a real 800x600
framebuffer capture bound to its current serial prefix and QMP screendump.
F12 releases the diagnostic controller's existing InitStopRPC only after
the reopened capture. This stop orchestration is not product shutdown or
ordinary seven-process DEV1 acceptance.

After complete VM exit, the separate disk oracle confirmed object 0,
revision 1, name `Nieuw document`, exactly 14 bytes, content SHA-256
`9a45a00fa9ff69c17e124f54c14bb14cf2c7650e8cf1c272f6631f618a19a5a5`,
generation 1, clocks `[1, 0]`, intact canaries and unchanged other domains.
This is in-session save/reopen, not a cold new VM.

Existing `capture_graphics.encode_png` converted the six original PPMs
without changing samples; `decode_png(png) == run_graphics.ppm(ppm_bytes)`
was asserted for each. The ready/dirty/saved/reopened PNGs were actually
viewed by the UI owner. Roles 15, 16 and 20 received the same read-only
bundle and frame directory for their separate reviews; delivery is not
proof that those reviews passed.

### Frozen handoff and remaining gates

Bundle path:
`/home/simon/.copilot/session-state/1b87793863ad4dcc8f16a0bd10884654/files/ui-image-0818/native-materiaal-probe-0818f5e-ad23a2ee91.tar.gz`.

Bundle: 2,682,413 bytes, SHA-256
`3c476387f7aa7657207e2a0ee96b7d1b159695de1716305cdbefa795b26447b2`.
All 40 member content hashes were rechecked after archive creation.
The archive, original run files and lossless PNGs are read-only; these are
permissions and hash bindings, not a claim that the owner cannot mutate them.
The original canonical run path is retained because the existing replay
checks absolute QMP screendump paths. Do not start a VM in the frozen run
directory; any future run requires a fresh runner-created state and slot.

No separate disk-image container was produced: the actual boot artifact is
the hashed ESP file set (loader and kernel), bound to the eight embedded
userspace ELFs. The archive is a delivery container, not a boot-image hash.

Not executed here: ordinary seven-process preview, cold recovery, native
deny/revoke/full/Unknown cases, dirty-close modal, intermediate APPEND versus
COMMIT screenshot, 200% or other resolutions, external semantic consumer or
screenreader, SVG asset consumption, human testing or physical hardware.
The two semantic-node regressions remain host assertions; this normal native
flow does not invoke an external accessibility consumer.

Score proposal to the central owner remains **34/100 -> 34/100** and
**U01-U06: 0/15 -> 0/15**. Next: review the same frozen frames and evidence
with roles 15/16/20; release owns selection of an ordinary DEV candidate.
No runtime rollback is needed for this evidence-only update.

### Same-artifact reviews received

- Role 15, source review `eb70accf56017b3f795cb4c463d4794811eb2010`:
  all six PNGs actually viewed; archive members, pixel roundtrips, UI/token
  source bindings and QMP replies independently checked. No clipping seen
  in the short text/focus/footer states. Concrete open findings are
  UX-V01 (Primary loses its accent fill on hover), UX-V02 (limited bitmap
  typography hierarchy) and UX-V03 (confirmed revision not visible in
  the shown UI). These are not fixed or waived by this evidence.
- Role 16, read-only review at
  `/home/simon/.copilot/session-state/add48d11fd504cf4aba91efd78f5c873/files/native-baseline-0818-review.md`:
  all six PNGs viewed and hashes checked. Future icons need a real
  icon/text layout slot, not an overlay on the current text baseline.
  A saved icon must use the relevant confirmed document status, not a
  historical global save counter. No supplied SVG is consumed here.
- Role 20, evidence commit `97b73e496d57c577978671d90aa392fa2c86fa7e`,
  reported successful read-only verification of the same bundle,
  40 member hashes, canonical files, tools, existing replay, 31 QMP
  requests/six phases, PNG/PPM bindings and the independent disk oracle.
  Four negative report mutations were rejected. Its report, verifier and logs
  are under `docs/verification/product-test-20/`. The first verifier attempt
  had a host tuple-versus-JSON-list comparison error after successful
  replay; role 20 retained that failure and corrected its verifier.
  This was not attributed to the guest or concealed as a first-pass result.

None of these peers started another VM. Their AI reviews are separate from
human usability/accessibility acceptance. No ordinary DEV1, cold recovery,
deny/full/Unknown, screenreader or complete release pass follows from them.
