# Native PNG/JPEG integration gate

IMAGE-HARNESS-1, base `4b638568e4dee62ac35f2a8f6407d7b05c7b770d`.
This gate binds the actual ring3 image probe to the original integrated
system/disk chain. It runs no host decoder and changes no PNG/JPEG boundary
to obtain a passing result. The
[image fixture](NATIVE-IMAGE-FIXTURE.md),
[codec component](../implementation/NATIVE-IMAGES.md) and
[FULL-TEST-RELEASE](../FULL-TEST-RELEASE.md) remain the substantive contracts.

## Profile and execution

The only profile is `native-system-images`; the CLI case is named `normal`.
Root supplies the existing four programs plus a fifth initial image probe
without capabilities, the opt-in memory/FPU/clock functions, the actual
bootstrap and shared build/run hooks. The app itself requests 1024 fresh
heap pages of its own and runs with the existing 64KiB stack. The harness
provides no pixels, heap or successful decoding outcome.

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools python3 -B tools/dev.py python3 -B tools/build.py --kernel-feature native-system-images
LOCALAPPDATA=/workspace/nieuw-os/.tools python3 -B tools/dev.py python3 -B tools/run_images.py --case normal
```

Use separate `NOS_BUILD_DIR`/`NOS_TARGET_DIR` paths within the checkout and
the specified QEMU/OVMF profile. The gate uses the existing guarded
`run_system`/`run_volume.execute`: TCG, one qemu64 CPU, 256MiB, no network or
host share, no snapshot, one new empty 1MiB disposable disk and the existing
virtio-rng for the Key Service. The image probe receives no storage, display,
network, key or entropy capability. There is no option to select a custom
disk or existing volume. Timeout is explicitly bounded to 180 seconds.

This gate is a functional test. Concurrent TCG runs on shared host CPUs
do not provide comparable performance results. The existing FR-I01 normal→
cold readonly recovery remains separately required regression evidence;
no image-recovery profile or durable pixel snapshot is assumed.

## Mandatory native evidence

The original boot, process-lifecycle, privilege, record, cleanup and disk
assertions remain active, including 2 seals/2 unseals/9 authentication
rejections and the exact 4096 document bytes. The actual QEMU exit must be
33 and timeout false. A missing probe, fault, panic, unexpected exit or a
PASS flag alone does not make the gate pass.

Before executing the probe, root bootstrap prints one twenty-digit
`SYSTEM:IMAGE_MANIFEST_PID:<actualpid>`, directly after `image_fixture::bind`
and before `SYSTEM:MANIFEST:PASS`. The final observation must report exactly
the same actual PID. The harness does not treat PID5 or a later user of
slot4 as identity. The kernel observer retains PID/boot slot, actual
launch/exit ticks and a snapshot before cleanup; it also checks ownership,
RW/NX and mappings/guards.

Then exactly the 20 specified `SYSTEM:IMAGE_*` fields in observer order
and one `SYSTEM:IMAGES:PASS` before `SYSTEM:BROKERS:PASS` are required.
All counters are exactly 20 decimal digits within u64. Unknown, duplicate,
missing, incorrectly ordered or overflowing fields are errors.
The required content is:

| Data | Requirement |
|---|---|
| PNG/JPEG |8/3, including successful redecoding after errors |
| Pixels / sample bytes |149647 /623808 |
| Negative outcomes / cancellation in advance |10 /1 |
| Pixel parity |548683896226622996 |
| App growth / total kernel heap |1024 /1040 pages |
| Arena peak / live allocations |actual >0 and ≤4194304 bytes /0 |
| Stack / guards |16 pages /1; existing 64KiB stack |
| Clock |start≤end≤exit; exit−start<20000 actual ticks |
| Specific JPEG question |QUALITY_OPEN=0 for the investigated, corrected fixture |
| Linked ELF |actual length and FNV compared with the reread built ELF |

The actual entropy broker must report the original 1024 bytes; this image
probe must cause no additional key/RNG operations. General JPEG quality
evidence is not equivalent to pixel parity using the same engine. The concrete
colour/PNG sample tests and original failing JPEG outcome are retained in
the codec contract.

## Artifact binding and result

`manifest.system` contains exactly `system_init`, `system_app`, `system_worker`,
`keyring_service`, `volume_service` and `image_probe`; the additional
`manifest.image_probe` must identify the same artifact. The builder uses
`TARGET/system-image-probe/x86_64-unknown-none/release/image-probe` and passes
`NOS_IMAGE_PROBE_ELF` to the kernel. The other five paths are the existing
system build targets.

The gate checks source-fingerprint version 3, the current feature and
manifest, SHA256 and fresh inspection of all six actual ELF files, and the
presence of their complete bytes in the loaded kernel. It compares both the
build ESP and actual QEMU ESP against the kernel/loader hashes, without extra
files or links. The linked ELF FNV is only a parity/drift fingerprint;
SHA256 binds the exact build bytes. **FNV grants no authenticity.**

The saved serial output is reread and strictly parsed. The generic system
parser does not recognise the numeric portion of `IMAGE_ELF_FNV64`; this
harness parses that field itself and records the complete image values
separately as `image_evidence`. Existing `evidence` remains equal to the
reevaluated original system data. Old or modified cached image/system
counters must not replace actual bytes.

The command profile, QEMU executable hash, selected OVMF code and preboot
VARS template hash must match and remain unchanged at the end. Actual writable
UEFI state is read under controlled access, not incorrectly equated with its
old bytes. A snapshot/network/different command line is not accepted as the
same test.

The independent disk parser checks the live disposable image and identical
post-run snapshot: the new volume started empty; the original document is
named `systeemproef`, contains 4096 exact fixture bytes at revision1 and
SAVE requestID3. Additional objects or collections written, incorrect canaries
or changed content cannot be concealed by a saved oracle success.

Symlinks, special files and hard-linked evidence/output files are rejected.
Only read-only consumption of actual Cargo output permits its internal
hard links. Publication uses new inodes and atomic replacement within owned
paths. The initially false `image_gate` becomes true only after all checks.
Every additional error also sets `passed=False`. Results are stored under
the dedicated build directory as `image-report.json` and with the actual run
as `result.json`; `last-system-stage.json` with this different feature does
not authorise ordinary FR-I01 cold recovery. This record is not signed
attestation against a malicious host that falsifies all source, binaries
and evidence files.

The current `image-report.json` is atomically replaced with `status=starting`,
`passed=False`, `image_gate=False` before further preflight and VM delegation.
A preflight/launch exception writes `status=failed` with false flags and is
propagated; an older passing index report therefore cannot conceal a new
failure. Historical per-run results are retained. Unsafely linked report
targets are rejected before publication without modifying external bytes.
Before shared copy/delegation, the source ESP itself is checked for exactly
the two hashed files, all ancestors and absence of links/special files.
The subsequent check of the actual guest ESP remains mandatory and does
not replace this early boundary.

## Host negative tests and open acceptance

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools python3 -B tools/dev.py python3 -B -m unittest discover -s tools/tests -p test_images.py
```

The explicitly synthetic host fixtures test only the harness: mixed PIDs,
incorrect bootstrap order, error/timeout with a PASS flag, different pixel/
heap/stack/clock figures, incorrect ELF FNV/length/SHA/inspection, profile/
source errors, snapshot/network commands, links, changed serial output/disk
and stale oracle results. They run no image decoder and constitute no native log.

Actual root VM execution on the frozen integration source and the existing
joint cold regressions remain mandatory. General viewer/format acceptance,
object RPC, Materiaal/compositor integration, zoom/pan, mid-decode cancellation,
actual native allocator OOM/restart and complete memory-safety/quality audit
remain open. A passing sample test on a 64KiB stack is not a maximum-stack
guarantee for all input. This gate does not independently sign off FR-A03
or the complete test release.
