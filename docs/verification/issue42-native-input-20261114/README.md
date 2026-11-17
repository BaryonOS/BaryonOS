# H02 native USB and protected HID checkpoint

The native USB transport and DMA containment fixtures pass on the published
source below. The independent native input-policy fixture also passes, using
synthetic HID completions. H02 remains open, with the assessment at **41/100**.

## Source identity

- PR: [#333](https://github.com/BaryonOS/BaryonOS/pull/333).
- Verified native/standard source: `36f131fe3ce0fb5e4ad92ef4cf3cab4e86637694`.
- Git tree: `c5c02eae7619f105debb3a17fef02e2e552c022f`.
- Source-v3: `922c0e579e6fed6b03bf94ae320a01350504a334ff4d406a9fc9d050a37e25fe`.
- Host-test source: `b082f836cd8f3969fd144c3a6a9b01c8656d4122`, source-v3
  `606ad2bc02ceaff9ff46f20412c8b874954b6582c3a7d2452ec09f65e2b5e92c`.

The only committed changes between the host-test and final native sources are
eleven added lines in `sdk/input-client/Cargo.lock` and `ui/native/Cargo.lock`.
They complete the new local USB dependency without changing existing versions.
The [integrated receipt](integrated.json) checks this exact delta; both locked
offline dependency fetches pass on the final source. All six native cases and
the standard gates were rerun on that final published source. Subsequent
evidence/status changes do not change executable inputs.

PR #328 was squash-merged as `4cd444e08dac820cc66f2750c43845b4139901cc`.
Its implementation is retained. This continuation also preserves main's N04
helper-isolation follow-up at `ce47359fcd9c62abca50da87ac62183026aac91c`.

## Implementation results

DMA allocations now carry opaque lease/record ownership, preventing generic
free, reallocation and inappropriate aliases before the device lease releases
them. The original generic-free regression was reproduced before the fix.
Validated DMAR requester coverage supports an actual VT-d domain, hardware
readback and completed invalidation. General DMA inventory is still not an
authority grant; the ordinary bus-master model remains test-only.

A real QEMU capability layout reproduced an xHCI parser defect: an extended
capability can occupy the permitted gap before the operational registers.
The corrected parser retains the register-overlap and aperture checks.

The fixed USB profile now performs actual enumeration, configuration, boot
protocol selection and interrupt transfers. It validates every completion
before the existing USB Session sees the bytes. A real held-key unplug clears
the key exactly once and rejects the old token. Details and limitations are in
[the transport contract](../../implementation/NATIVE-USB-HID.md).

Protected HID ingress and `hid::Source` connect validated sessions to the input
policy. The separate own-kernel test verifies authentication, focus, reset,
detach/reattach and stale-message behavior through real IPC. Its HID reports
remain synthetic; it does not establish the production USB-to-UI connection.

## Native results

Local runtime: QEMU **10.0.13**, `pc-q35-8.2`, qemu64, TCG, one CPU, 256 MiB.
Every row below passed with raw QEMU exit **33**, no timeout, matching executed
artifacts and the final source fingerprint.

| Case | Observed result | Evidence |
|---|---|---|
| xHCI No-Op | Actual command fetch and matching event write through VT-d | [Report](reports/xhci-noop-report.json) |
| Unmapped write | Actual DMA write fault; independently initialized guard unchanged | [Report](reports/xhci-fault-report.json) |
| Missing DMAR | Actual execution denied; BME checked before cleanup | [Report](reports/xhci-no-dmar-report.json) |
| Read-only mapping | Actual VT-d write-permission fault, reason 5; full guard unchanged | [Report](reports/usb-hid-readonly-report.json) |
| USB reports/unplug | Keyboard, relative mouse, button state, held-key unplug and stale token | [Report](reports/usb-hid-reports-report.json) |
| Native HID IPC | Synthetic reports; 11 publications, 6 source rejections, 10 client events, 4 cancellations | [Report](reports/hid-input-report.json) |
| Standard smoke | Default OS boot with `M1:PASS`; no USB execution claim | [Report](reports/standard-smoke.json) |

Strict preflight, the standard build and smoke pass. Commands, exit codes,
compressed-log receipts and raw-log digests are retained in
[standard checks](standard-checks.json) and [native checks](native-checks.json).
The dedicated workflows select QEMU 8.2.2 separately; hosted observations are
recorded with their own run/commit identity in `hosted-checks.json`.

## Host results and regressions

All **213 Rust tests** pass: boot protocol 33, kernel core 59, xHCI 16,
USB 44, input core 30 and input daemon 31. Formatting and strict Clippy pass.
All **112 Python tests** pass across the three new runtime oracles, packaging,
guest harness, artifact binding, build paths and workflow preservation checks.
The exact commands and their source identity are in [host checks](host-checks.json).

Independent AI review found and prompted correction of a desktop report-shape
regression. Actual builder output now preserves ordinary desktop metadata; the
extra HID marker appears only in the opt-in profile. The new regression test
fails both ordinary profiles against the previous producer and passes after
the fix. Compact-only workflows retain explicit `full-evidence: 'false'` and
the existing retention policy; invalid or mixed preservation settings fail.

The first hosted run at `b082f836` exposed the two incomplete transitive locks.
The failure was corrected without disabling `--locked` or upgrading packages.
Expected-failure logs for DMA ownership, xHCI layout and desktop metadata, plus
the lock repair proof, are preserved under `regressions/`.

## Evidence and acceptance boundaries

`integrated.json` indexes preserved manifests, reports and raw text logs by
SHA-256 and size. Runtime reports bind the actual kernel, loader, firmware and
input actors. Kernel binaries, full VM disks and firmware images are not
archived in Git; independent execution requires rebuilding the recorded source
and providing the documented runtime. A report or hash alone is not a new run.

An [independent AI artifact replay](artifact-replay.json) verifies all 81 indexed
files, the six native oracles and standard smoke, the 213/112 test counts and
the exact lock-only source delta. It also rehashes 29 retained runtime artifacts
and rechecks the HID actors' metadata and actual ELF embedding. This is an
evidence-consistency review; it neither rebuilds nor reruns QEMU.

The reviews are AI reviews, not human hardware/security acceptance. The next
step is a bounded completion handoff from the long-lived native device owner
to `hid::Source`, with explicit authority and DMA coexistence for other devices.
Real IRQ handling, hardware reset/reattach, general hotplug and the authorized
physical reference matrix remain open. No criterion, weight or acceptance
boundary changes; issue #42 is not closed.
